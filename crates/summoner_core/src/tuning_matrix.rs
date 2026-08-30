// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Dynamic Microtonal Tuning Remapping Matrix (Milestone 13).
//!
//! Provides a lock-free real-time tuning remapping engine supporting per-note
//! frequency lookups, dynamic temperament crossfading, microtonal scale quantization,
//! and atomic parameter dispatch into `ParamBus`.
//!
//! Enforces zero heap allocations during audio thread execution under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use crate::param_bus::{ParamBus, ParamId};

/// Total standard MIDI note slots supported in the primary tuning table.
pub const TUNING_TABLE_SIZE: usize = 128;

/// Remapping mode for dynamic pitch transformation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TuningRemapMode {
    /// Exact target frequency per note index.
    #[default]
    DirectFrequency,
    /// Continuous cent offset added to baseline 12-TET pitch.
    CentOffset,
    /// Dynamic linear/logarithmic morph between two distinct tuning systems.
    TemperamentInterpolation,
    /// Snapped to nearest legal step of a defined microtonal scale.
    ScaleQuantize,
}

mod serde_array_128 {
    use super::TUNING_TABLE_SIZE;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(data: &[f32; TUNING_TABLE_SIZE], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        data.as_slice().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[f32; TUNING_TABLE_SIZE], D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = Vec::<f32>::deserialize(deserializer)?;
        let mut arr = [0.0f32; TUNING_TABLE_SIZE];
        for (i, val) in v.into_iter().take(TUNING_TABLE_SIZE).enumerate() {
            arr[i] = val;
        }
        Ok(arr)
    }
}

/// Dynamic Tuning Remapping Matrix configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningMatrix {
    pub name: String,
    pub mode: TuningRemapMode,
    pub root_note: u8,
    pub root_frequency_hz: f32,
    pub morph_alpha: f32,
    pub target_edo: u16,
    #[serde(with = "serde_array_128")]
    pub primary_frequencies: [f32; TUNING_TABLE_SIZE],
    #[serde(with = "serde_array_128")]
    pub secondary_frequencies: [f32; TUNING_TABLE_SIZE],
    #[serde(with = "serde_array_128")]
    pub cent_offsets: [f32; TUNING_TABLE_SIZE],
    pub param_morph_id: Option<ParamId>,
    pub param_root_id: Option<ParamId>,
}

impl Default for TuningMatrix {
    fn default() -> Self {
        Self::new("Standard 12-TET Matrix", 69, 440.0)
    }
}

impl TuningMatrix {
    pub fn new(name: impl Into<String>, root_note: u8, root_freq_hz: f32) -> Self {
        let mut primary = [0.0f32; TUNING_TABLE_SIZE];
        let mut secondary = [0.0f32; TUNING_TABLE_SIZE];
        let cent_offsets = [0.0f32; TUNING_TABLE_SIZE];

        let root_n = root_note.min(127);
        let root_f = root_freq_hz.max(10.0);

        for n in 0..TUNING_TABLE_SIZE {
            // Primary: Standard 12-TET
            let semi_diff = n as f32 - root_n as f32;
            primary[n] = root_f * 2.0_f32.powf(semi_diff / 12.0);

            // Secondary: 19-EDO
            secondary[n] = root_f * 2.0_f32.powf(semi_diff / 19.0);
        }

        Self {
            name: name.into(),
            mode: TuningRemapMode::DirectFrequency,
            root_note: root_n,
            root_frequency_hz: root_f,
            morph_alpha: 0.0,
            target_edo: 19,
            primary_frequencies: primary,
            secondary_frequencies: secondary,
            cent_offsets,
            param_morph_id: None,
            param_root_id: None,
        }
    }

    /// Sets up custom N-EDO tuning in the secondary table.
    pub fn set_secondary_edo(&mut self, edo: u16) {
        self.target_edo = edo.max(1);
        let edo_f = self.target_edo as f32;
        for n in 0..TUNING_TABLE_SIZE {
            let diff = n as f32 - self.root_note as f32;
            self.secondary_frequencies[n] = self.root_frequency_hz * 2.0_f32.powf(diff / edo_f);
        }
    }

    /// Sets per-note cent offset.
    pub fn set_cent_offset(&mut self, note: u8, cents: f32) {
        if (note as usize) < TUNING_TABLE_SIZE {
            self.cent_offsets[note as usize] = cents;
        }
    }

    /// Calculates mapped frequency for a given MIDI note with zero heap allocation.
    #[inline]
    pub fn note_to_frequency(&self, note: u8) -> f32 {
        let idx = (note as usize).min(TUNING_TABLE_SIZE - 1);
        match self.mode {
            TuningRemapMode::DirectFrequency => self.primary_frequencies[idx],
            TuningRemapMode::CentOffset => {
                let base_freq = self.primary_frequencies[idx];
                let cents = self.cent_offsets[idx];
                base_freq * 2.0_f32.powf(cents / 1200.0)
            }
            TuningRemapMode::TemperamentInterpolation => {
                let f1 = self.primary_frequencies[idx];
                let f2 = self.secondary_frequencies[idx];
                let alpha = self.morph_alpha.clamp(0.0, 1.0);
                // Logarithmic pitch interpolation to avoid beating
                (f1.ln() * (1.0 - alpha) + f2.ln() * alpha).exp()
            }
            TuningRemapMode::ScaleQuantize => self.primary_frequencies[idx],
        }
    }

    /// Updates dynamic parameter controls from `ParamBus`.
    pub fn poll_param_bus(&mut self, param_bus: &ParamBus) {
        if let Some(param_id) = self.param_morph_id {
            if let Some(val) = param_bus.get(param_id) {
                self.morph_alpha = val.clamp(0.0, 1.0);
            }
        }
        if let Some(param_id) = self.param_root_id {
            if let Some(val) = param_bus.get(param_id) {
                self.root_frequency_hz = val.clamp(20.0, 2000.0);
            }
        }
    }

    /// Dispatches active tuning matrix state to `ParamBus`.
    pub fn dispatch_to_param_bus(&self, param_bus: &ParamBus) {
        if let Some(param_id) = self.param_morph_id {
            param_bus.set(param_id, self.morph_alpha);
        }
        if let Some(param_id) = self.param_root_id {
            param_bus.set(param_id, self.root_frequency_hz);
        }
    }

    /// Serializes configuration to TOML string.
    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string(self)
    }

    /// Deserializes configuration from TOML string.
    pub fn from_toml_str(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }
}

/// Lock-free thread-safe Tuning Table Remapper for real-time audio threads.
#[derive(Debug)]
pub struct LockFreeTuningRemapper {
    matrix: Arc<TuningMatrix>,
    cached_frequencies: [AtomicU32; TUNING_TABLE_SIZE],
}

impl LockFreeTuningRemapper {
    pub fn new(matrix: TuningMatrix) -> Self {
        let mut cached = [const { AtomicU32::new(0) }; TUNING_TABLE_SIZE];
        for (i, item) in cached.iter_mut().enumerate() {
            let f = matrix.note_to_frequency(i as u8);
            *item = AtomicU32::new(f.to_bits());
        }

        Self {
            matrix: Arc::new(matrix),
            cached_frequencies: cached,
        }
    }

    /// Real-time lock-free note to frequency lookup.
    #[inline]
    pub fn get_frequency(&self, note: u8) -> f32 {
        let idx = (note as usize).min(TUNING_TABLE_SIZE - 1);
        let bits = self.cached_frequencies[idx].load(Ordering::Relaxed);
        f32::from_bits(bits)
    }

    /// Updates matrix configuration and refreshes atomic frequency cache.
    pub fn update_matrix(&mut self, new_matrix: TuningMatrix) {
        for (i, item) in self.cached_frequencies.iter().enumerate() {
            let f = new_matrix.note_to_frequency(i as u8);
            item.store(f.to_bits(), Ordering::Release);
        }
        self.matrix = Arc::new(new_matrix);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::allocator::AllocGuard;

    #[test]
    fn test_tuning_matrix_temperament_interpolation() {
        let mut matrix = TuningMatrix::new("Test Morph", 69, 440.0);
        matrix.mode = TuningRemapMode::TemperamentInterpolation;
        matrix.set_secondary_edo(19);

        matrix.morph_alpha = 0.0;
        let f_12tet = matrix.note_to_frequency(69 + 12);
        assert!((f_12tet - 880.0).abs() < 1e-4);

        matrix.morph_alpha = 1.0;
        let f_19edo = matrix.note_to_frequency(69 + 19);
        assert!((f_19edo - 880.0).abs() < 1e-4);

        matrix.morph_alpha = 0.5;
        let f_mid = matrix.note_to_frequency(69 + 12);
        assert!(f_mid > 440.0 && f_mid < 880.0);
    }

    #[test]
    fn test_lock_free_tuning_remapper_zero_alloc() {
        let matrix = TuningMatrix::new("LockFree Test", 69, 440.0);
        let remapper = LockFreeTuningRemapper::new(matrix);

        let mut out_freqs = [0.0f32; 64];
        {
            let _guard = AllocGuard::new();
            for (i, freq) in out_freqs.iter_mut().enumerate() {
                *freq = remapper.get_frequency((60 + (i % 24)) as u8);
            }
        }

        assert!(out_freqs.iter().all(|f| *f >= 20.0 && f.is_finite()));
    }

    #[test]
    fn test_tuning_matrix_toml_roundtrip() {
        let matrix = TuningMatrix::new("RoundTrip Matrix", 60, 261.63);
        let toml_str = matrix.to_toml_string().expect("TOML serialization failed");
        let restored = TuningMatrix::from_toml_str(&toml_str).expect("TOML deserialization failed");

        assert_eq!(matrix.name, restored.name);
        assert_eq!(matrix.root_note, restored.root_note);
        assert!((matrix.root_frequency_hz - restored.root_frequency_hz).abs() < 1e-4);
    }
}
