// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Isomorphic Microtonal Voice Router & Hexagonal Grid Note Coordinate Mapping (Milestone 13).
//!
//! Provides a 2D isomorphic hexagonal pitch lattice coordinate system with multiple
//! historical and microtonal layout generators (Wicki-Hayden, Gerhard, Bosanquet,
//! Janko, Harmonic Table, Rank-2 Generalized), scale degree filtering, and real-time
//! polyphonic microtonal voice allocation with sample-accurate frequency output.

use serde::{Deserialize, Serialize};
use crate::scale::Scale;

/// Standard isomorphic layout topologies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum IsomorphicLayoutType {
    /// Wicki-Hayden layout (horizontal +2 whole tones, diagonal +7 / +11 fifths).
    #[default]
    WickiHayden,
    /// Gerhard layout (horizontal +3 minor thirds, diagonal +4 major thirds).
    Gerhard,
    /// Bosanquet generalized microtonal keyboard.
    Bosanquet,
    /// Janko keyboard layout (alternating whole-tone scales).
    Janko,
    /// Harmonic Table layout (triadic lattice: root, minor third, major third, fifth).
    HarmonicTable,
    /// Arbitrary Rank-2 Temperament defined by generator and period steps.
    Rank2Temperament {
        generator_steps: i32,
        period_steps: i32,
    },
    /// Custom user-defined coordinate displacement vector.
    Custom {
        col_steps: i32,
        row_steps: i32,
    },
}

impl IsomorphicLayoutType {
    /// Computes (col_step_delta, row_step_delta) in scale degree units for the given EDO.
    pub fn step_generators(&self, edo: u16) -> (i32, i32) {
        match self {
            Self::WickiHayden => {
                let fifth = if edo == 19 { 11 } else if edo == 31 { 18 } else { 7 };
                let wholetone = if edo == 19 { 3 } else if edo == 31 { 5 } else { 2 };
                (wholetone, fifth)
            }
            Self::Gerhard => {
                let m3 = if edo == 19 { 5 } else if edo == 31 { 8 } else { 3 };
                let m3_maj = if edo == 19 { 6 } else if edo == 31 { 10 } else { 4 };
                (m3, m3_maj)
            }
            Self::Bosanquet => (1, if edo == 19 { 7 } else { 4 }),
            Self::Janko => (2, 1),
            Self::HarmonicTable => {
                let m3 = if edo == 19 { 5 } else if edo == 31 { 8 } else { 3 };
                let m3_maj = if edo == 19 { 6 } else if edo == 31 { 10 } else { 4 };
                (m3_maj, m3)
            }
            Self::Rank2Temperament {
                generator_steps,
                period_steps,
            } => (*generator_steps, *period_steps),
            Self::Custom {
                col_steps,
                row_steps,
            } => (*col_steps, *row_steps),
        }
    }
}

/// 2D Hexagonal axial coordinate (q, r) with implicit cube coordinate s = -q - r.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HexCoordinate {
    pub q: i32,
    pub r: i32,
}

impl HexCoordinate {
    pub const fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    /// Third redundant cube coordinate satisfying q + r + s = 0.
    pub const fn s(&self) -> i32 {
        -self.q - self.r
    }

    /// Hexagonal grid distance to another coordinate.
    pub fn distance_to(&self, other: &Self) -> i32 {
        let dq = (self.q - other.q).abs();
        let dr = (self.r - other.r).abs();
        let ds = (self.s() - other.s()).abs();
        (dq + dr + ds) / 2
    }
}

/// Polyphonic voice state tracked by the isomorphic router.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IsomorphicVoiceState {
    pub voice_id: usize,
    pub coord: HexCoordinate,
    pub step_index: i32,
    pub frequency_hz: f32,
    pub velocity: f32,
    pub active: bool,
    pub age_samples: u64,
}

/// Polyphonic Isomorphic Voice Router for real-time microtonal performance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsomorphicRouter {
    pub layout: IsomorphicLayoutType,
    pub edo: u16,
    pub root_freq_hz: f32,
    pub root_step_offset: i32,
    pub active_scale: Option<Scale>,
    pub max_voices: usize,
    pub voices: Vec<IsomorphicVoiceState>,
}

impl Default for IsomorphicRouter {
    fn default() -> Self {
        Self::new(IsomorphicLayoutType::WickiHayden, 19, 440.0, 16)
    }
}

impl IsomorphicRouter {
    pub fn new(
        layout: IsomorphicLayoutType,
        edo: u16,
        root_freq_hz: f32,
        max_voices: usize,
    ) -> Self {
        let n_voices = max_voices.clamp(1, 64);
        let mut voices = Vec::with_capacity(n_voices);
        for id in 0..n_voices {
            voices.push(IsomorphicVoiceState {
                voice_id: id,
                coord: HexCoordinate::new(0, 0),
                step_index: 0,
                frequency_hz: root_freq_hz,
                velocity: 0.0,
                active: false,
                age_samples: 0,
            });
        }

        Self {
            layout,
            edo: edo.max(1),
            root_freq_hz: root_freq_hz.max(10.0),
            root_step_offset: 0,
            active_scale: None,
            max_voices: n_voices,
            voices,
        }
    }

    /// Converts hexagonal coordinate (q, r) into linear EDO step index relative to root.
    pub fn coord_to_step(&self, coord: HexCoordinate) -> i32 {
        let (col_gen, row_gen) = self.layout.step_generators(self.edo);
        coord.q * col_gen + coord.r * row_gen + self.root_step_offset
    }

    /// Converts linear EDO step index to exact frequency in Hertz.
    pub fn step_to_frequency(&self, step: i32) -> f32 {
        let edo_f = self.edo as f32;
        let exponent = step as f32 / edo_f;
        self.root_freq_hz * 2.0_f32.powf(exponent)
    }

    /// Converts hexagonal coordinate directly into frequency in Hertz.
    pub fn coord_to_frequency(&self, coord: HexCoordinate) -> f32 {
        let step = self.coord_to_step(coord);
        self.step_to_frequency(step)
    }

    /// Checks if a hexagonal coordinate falls within the active scale (if scale filtering is enabled).
    pub fn is_coord_in_scale(&self, coord: HexCoordinate) -> bool {
        if let Some(ref scale) = self.active_scale {
            let step = self.coord_to_step(coord);
            let oct_step = step.rem_euclid(self.edo as i32) as u16;
            scale.contains_step(oct_step)
        } else {
            true
        }
    }

    /// Triggers a note at hexagonal coordinate (q, r) with given velocity.
    /// Returns the assigned voice index.
    pub fn note_on(&mut self, coord: HexCoordinate, velocity: f32) -> Option<usize> {
        let step = self.coord_to_step(coord);
        let freq = self.step_to_frequency(step);

        // If this coordinate is already held, retrigger the same voice
        if let Some(voice_idx) = self.voices.iter().position(|v| v.active && v.coord == coord) {
            self.voices[voice_idx].velocity = velocity;
            self.voices[voice_idx].active = true;
            self.voices[voice_idx].age_samples = 0;
            return Some(voice_idx);
        }

        // Find inactive voice or oldest voice to steal
        let mut target_idx = None;
        let mut oldest_age = 0u64;
        let mut oldest_idx = 0;

        for (idx, voice) in self.voices.iter().enumerate() {
            if !voice.active {
                target_idx = Some(idx);
                break;
            }
            if voice.age_samples >= oldest_age {
                oldest_age = voice.age_samples;
                oldest_idx = idx;
            }
        }

        let voice_idx = target_idx.unwrap_or(oldest_idx);

        self.voices[voice_idx].coord = coord;
        self.voices[voice_idx].step_index = step;
        self.voices[voice_idx].frequency_hz = freq;
        self.voices[voice_idx].velocity = velocity;
        self.voices[voice_idx].active = true;
        self.voices[voice_idx].age_samples = 0;

        Some(voice_idx)
    }

    /// Releases note at hexagonal coordinate (q, r).
    pub fn note_off(&mut self, coord: HexCoordinate) -> Option<usize> {
        if let Some(voice_idx) = self.voices.iter().position(|v| v.active && v.coord == coord) {
            self.voices[voice_idx].active = false;
            Some(voice_idx)
        } else {
            None
        }
    }

    /// Releases all active notes.
    pub fn all_notes_off(&mut self) {
        for voice in &mut self.voices {
            voice.active = false;
        }
    }

    /// Advances voice age by given number of audio frames.
    pub fn advance_frames(&mut self, frames: u64) {
        for voice in &mut self.voices {
            if voice.active {
                voice.age_samples = voice.age_samples.saturating_add(frames);
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isomorphic_coordinate_step_math() {
        let router_19 = IsomorphicRouter::new(IsomorphicLayoutType::WickiHayden, 19, 440.0, 8);
        let root_step = router_19.coord_to_step(HexCoordinate::new(0, 0));
        assert_eq!(root_step, 0);

        let horiz_step = router_19.coord_to_step(HexCoordinate::new(1, 0));
        assert_eq!(horiz_step, 3); // 19-EDO wholetone is 3 steps

        let diag_step = router_19.coord_to_step(HexCoordinate::new(0, 1));
        assert_eq!(diag_step, 11); // 19-EDO fifth is 11 steps

        let freq_root = router_19.coord_to_frequency(HexCoordinate::new(0, 0));
        assert!((freq_root - 440.0).abs() < 1e-4);

        let freq_fifth = router_19.coord_to_frequency(HexCoordinate::new(0, 1));
        let expected_fifth = 440.0 * 2.0_f32.powf(11.0 / 19.0);
        assert!((freq_fifth - expected_fifth).abs() < 1e-4);
    }

    #[test]
    fn test_isomorphic_polyphonic_voice_allocation_and_stealing() {
        let mut router = IsomorphicRouter::new(IsomorphicLayoutType::WickiHayden, 19, 440.0, 2);

        let v0 = router.note_on(HexCoordinate::new(0, 0), 0.8).unwrap();
        let v1 = router.note_on(HexCoordinate::new(1, 0), 0.9).unwrap();
        assert_ne!(v0, v1);

        router.advance_frames(100);
        // Exceed voice limit to trigger voice stealing
        let v2 = router.note_on(HexCoordinate::new(0, 1), 0.7).unwrap();
        assert!(v2 == v0 || v2 == v1);

        router.note_off(HexCoordinate::new(0, 1));
        assert!(!router.voices[v2].active);
    }

    #[test]
    fn test_isomorphic_router_toml_roundtrip() {
        let router = IsomorphicRouter::new(IsomorphicLayoutType::Gerhard, 31, 261.63, 16);
        let toml_str = router.to_toml_string().expect("TOML serialization failed");
        let restored = IsomorphicRouter::from_toml_str(&toml_str).expect("TOML deserialization failed");

        assert_eq!(router.edo, restored.edo);
        assert_eq!(router.layout, restored.layout);
        assert_eq!(router.max_voices, restored.max_voices);
    }
}
