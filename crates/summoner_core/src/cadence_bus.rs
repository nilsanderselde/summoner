// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Lock-Free Cadence Bus & Real-Time Harmonic Parameter Routing (Milestone 14).
//!
//! Provides thread-safe lock-free exchange of chord progression states, active root pitch,
//! chord qualities, harmonic tension scores, voice-leading transition costs, and cadence
//! resolution predictions across audio, sequencer, and GUI threads.
//!
//! Enforces zero heap allocations in real-time audio threads under `AllocGuard`.

use crate::param_bus::{ParamBus, ParamId};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicI32, AtomicU32, Ordering};

/// Harmonic chord quality enum for atomic encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u32)]
pub enum ChordQuality {
    #[default]
    Major = 0,
    Minor = 1,
    Dominant7 = 2,
    Major7 = 3,
    Minor7 = 4,
    Diminished = 5,
    HalfDiminished = 6,
    Augmented = 7,
    Suspended4 = 8,
    Suspended2 = 9,
    Diminished7 = 10,
    AlteredDominant = 11,
}

impl ChordQuality {
    pub fn from_u32(val: u32) -> Self {
        match val {
            0 => Self::Major,
            1 => Self::Minor,
            2 => Self::Dominant7,
            3 => Self::Major7,
            4 => Self::Minor7,
            5 => Self::Diminished,
            6 => Self::HalfDiminished,
            7 => Self::Augmented,
            8 => Self::Suspended4,
            9 => Self::Suspended2,
            10 => Self::Diminished7,
            11 => Self::AlteredDominant,
            _ => Self::Major,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Major => "Major",
            Self::Minor => "Minor",
            Self::Dominant7 => "Dominant 7th",
            Self::Major7 => "Major 7th",
            Self::Minor7 => "Minor 7th",
            Self::Diminished => "Diminished",
            Self::HalfDiminished => "Half-Diminished 7th",
            Self::Augmented => "Augmented",
            Self::Suspended4 => "Sus4",
            Self::Suspended2 => "Sus2",
            Self::Diminished7 => "Diminished 7th",
            Self::AlteredDominant => "Altered Dominant",
        }
    }
}

/// Cadence resolution type for atomic encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u32)]
pub enum CadenceResolutionType {
    #[default]
    Authentic = 0,
    Plagal = 1,
    Deceptive = 2,
    Half = 3,
    Modulatory = 4,
}

impl CadenceResolutionType {
    pub fn from_u32(val: u32) -> Self {
        match val {
            0 => Self::Authentic,
            1 => Self::Plagal,
            2 => Self::Deceptive,
            3 => Self::Half,
            4 => Self::Modulatory,
            _ => Self::Authentic,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Authentic => "Authentic (V -> I)",
            Self::Plagal => "Plagal (IV -> I)",
            Self::Deceptive => "Deceptive (V -> vi)",
            Self::Half => "Half (I -> V)",
            Self::Modulatory => "Modulatory (Pivot -> Key)",
        }
    }
}

/// Instantaneous lock-free snapshot of current cadence bus state.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CadenceBusSnapshot {
    pub current_root_step: i32,
    pub current_quality: ChordQuality,
    pub target_root_step: i32,
    pub target_quality: ChordQuality,
    pub harmonic_tension: f32,
    pub resolution_urgency: f32,
    pub voice_leading_cost: f32,
    pub predicted_cadence: CadenceResolutionType,
    pub path_progress: f32,
    pub path_length: u32,
    pub auto_progression_active: bool,
}

impl Default for CadenceBusSnapshot {
    fn default() -> Self {
        Self {
            current_root_step: 0,
            current_quality: ChordQuality::Major,
            target_root_step: 0,
            target_quality: ChordQuality::Major,
            harmonic_tension: 0.0,
            resolution_urgency: 0.0,
            voice_leading_cost: 0.0,
            predicted_cadence: CadenceResolutionType::Authentic,
            path_progress: 0.0,
            path_length: 0,
            auto_progression_active: false,
        }
    }
}

/// Lock-free harmonic cadence parameter bus.
pub struct CadenceBus {
    current_root_step: AtomicI32,
    current_quality: AtomicU32,
    target_root_step: AtomicI32,
    target_quality: AtomicU32,
    harmonic_tension_bits: AtomicU32,
    resolution_urgency_bits: AtomicU32,
    voice_leading_cost_bits: AtomicU32,
    predicted_cadence: AtomicU32,
    path_progress_bits: AtomicU32,
    path_length: AtomicU32,
    flags: AtomicU32,
}

impl Default for CadenceBus {
    fn default() -> Self {
        Self::new()
    }
}

impl CadenceBus {
    /// Creates a new CadenceBus initialized to tonic C Major.
    pub fn new() -> Self {
        Self {
            current_root_step: AtomicI32::new(0),
            current_quality: AtomicU32::new(ChordQuality::Major as u32),
            target_root_step: AtomicI32::new(0),
            target_quality: AtomicU32::new(ChordQuality::Major as u32),
            harmonic_tension_bits: AtomicU32::new(0.0f32.to_bits()),
            resolution_urgency_bits: AtomicU32::new(0.0f32.to_bits()),
            voice_leading_cost_bits: AtomicU32::new(0.0f32.to_bits()),
            predicted_cadence: AtomicU32::new(CadenceResolutionType::Authentic as u32),
            path_progress_bits: AtomicU32::new(0.0f32.to_bits()),
            path_length: AtomicU32::new(0),
            flags: AtomicU32::new(0),
        }
    }

    /// Sets the active current chord root step and chord quality.
    pub fn set_current_chord(&self, root_step: i32, quality: ChordQuality) {
        self.current_root_step.store(root_step, Ordering::Release);
        self.current_quality.store(quality as u32, Ordering::Release);
    }

    /// Gets the current chord root step and quality.
    pub fn get_current_chord(&self) -> (i32, ChordQuality) {
        let root = self.current_root_step.load(Ordering::Acquire);
        let quality_idx = self.current_quality.load(Ordering::Acquire);
        (root, ChordQuality::from_u32(quality_idx))
    }

    /// Sets the target cadence resolution chord.
    pub fn set_target_chord(&self, root_step: i32, quality: ChordQuality) {
        self.target_root_step.store(root_step, Ordering::Release);
        self.target_quality.store(quality as u32, Ordering::Release);
    }

    /// Gets the target cadence resolution chord.
    pub fn get_target_chord(&self) -> (i32, ChordQuality) {
        let root = self.target_root_step.load(Ordering::Acquire);
        let quality_idx = self.target_quality.load(Ordering::Acquire);
        (root, ChordQuality::from_u32(quality_idx))
    }

    /// Sets the current harmonic tension score [0.0 ..= 1.0].
    pub fn set_harmonic_tension(&self, tension: f32) {
        let clamped = tension.clamp(0.0, 1.0);
        self.harmonic_tension_bits.store(clamped.to_bits(), Ordering::Release);
    }

    /// Gets the current harmonic tension score.
    pub fn get_harmonic_tension(&self) -> f32 {
        f32::from_bits(self.harmonic_tension_bits.load(Ordering::Acquire))
    }

    /// Sets resolution urgency [0.0 ..= 1.0].
    pub fn set_resolution_urgency(&self, urgency: f32) {
        let clamped = urgency.clamp(0.0, 1.0);
        self.resolution_urgency_bits.store(clamped.to_bits(), Ordering::Release);
    }

    /// Gets resolution urgency.
    pub fn get_resolution_urgency(&self) -> f32 {
        f32::from_bits(self.resolution_urgency_bits.load(Ordering::Acquire))
    }

    /// Sets voice-leading transition cost.
    pub fn set_voice_leading_cost(&self, cost: f32) {
        let non_neg = cost.max(0.0);
        self.voice_leading_cost_bits.store(non_neg.to_bits(), Ordering::Release);
    }

    /// Gets voice-leading transition cost.
    pub fn get_voice_leading_cost(&self) -> f32 {
        f32::from_bits(self.voice_leading_cost_bits.load(Ordering::Acquire))
    }

    /// Sets predicted cadence resolution type.
    pub fn set_predicted_cadence(&self, cadence: CadenceResolutionType) {
        self.predicted_cadence.store(cadence as u32, Ordering::Release);
    }

    /// Gets predicted cadence resolution type.
    pub fn get_predicted_cadence(&self) -> CadenceResolutionType {
        CadenceResolutionType::from_u32(self.predicted_cadence.load(Ordering::Acquire))
    }

    /// Sets path resolution progress [0.0 ..= 1.0] and total path length.
    pub fn set_path_status(&self, progress: f32, path_length: u32) {
        let p = progress.clamp(0.0, 1.0);
        self.path_progress_bits.store(p.to_bits(), Ordering::Release);
        self.path_length.store(path_length, Ordering::Release);
    }

    /// Sets auto-progression active flag.
    pub fn set_auto_progression_active(&self, active: bool) {
        let mut curr = self.flags.load(Ordering::Acquire);
        loop {
            let next = if active { curr | 1 } else { curr & !1 };
            match self.flags.compare_exchange_weak(curr, next, Ordering::Release, Ordering::Acquire) {
                Ok(_) => break,
                Err(actual) => curr = actual,
            }
        }
    }

    /// Captures a complete atomic snapshot of the cadence bus.
    pub fn snapshot(&self) -> CadenceBusSnapshot {
        let (current_root_step, current_quality) = self.get_current_chord();
        let (target_root_step, target_quality) = self.get_target_chord();
        let flags = self.flags.load(Ordering::Acquire);

        CadenceBusSnapshot {
            current_root_step,
            current_quality,
            target_root_step,
            target_quality,
            harmonic_tension: self.get_harmonic_tension(),
            resolution_urgency: self.get_resolution_urgency(),
            voice_leading_cost: self.get_voice_leading_cost(),
            predicted_cadence: self.get_predicted_cadence(),
            path_progress: f32::from_bits(self.path_progress_bits.load(Ordering::Acquire)),
            path_length: self.path_length.load(Ordering::Acquire),
            auto_progression_active: (flags & 1) != 0,
        }
    }

    /// Dispatches cadence bus states into `ParamBus` handles.
    pub fn dispatch_to_param_bus(&self, bus: &ParamBus, base_param_id: ParamId) {
        let snap = self.snapshot();
        bus.set(ParamId(base_param_id.0), snap.current_root_step as f32);
        bus.set(ParamId(base_param_id.0 + 1), snap.current_quality as u32 as f32);
        bus.set(ParamId(base_param_id.0 + 2), snap.harmonic_tension);
        bus.set(ParamId(base_param_id.0 + 3), snap.resolution_urgency);
        bus.set(ParamId(base_param_id.0 + 4), snap.voice_leading_cost);
        bus.set(ParamId(base_param_id.0 + 5), snap.predicted_cadence as u32 as f32);
        bus.set(ParamId(base_param_id.0 + 6), snap.path_progress);
    }

    /// Synchronizes cadence bus states from `ParamBus` handles.
    pub fn sync_from_param_bus(&self, bus: &ParamBus, base_param_id: ParamId) {
        let root = bus.get(ParamId(base_param_id.0)).unwrap_or(0.0).round() as i32;
        let quality_idx = bus.get(ParamId(base_param_id.0 + 1)).unwrap_or(0.0).max(0.0).round() as u32;
        let tension = bus.get(ParamId(base_param_id.0 + 2)).unwrap_or(0.0);
        let urgency = bus.get(ParamId(base_param_id.0 + 3)).unwrap_or(0.0);
        let cost = bus.get(ParamId(base_param_id.0 + 4)).unwrap_or(0.0);
        let cadence_idx = bus.get(ParamId(base_param_id.0 + 5)).unwrap_or(0.0).max(0.0).round() as u32;
        let progress = bus.get(ParamId(base_param_id.0 + 6)).unwrap_or(0.0);

        self.set_current_chord(root, ChordQuality::from_u32(quality_idx));
        self.set_harmonic_tension(tension);
        self.set_resolution_urgency(urgency);
        self.set_voice_leading_cost(cost);
        self.set_predicted_cadence(CadenceResolutionType::from_u32(cadence_idx));
        self.set_path_status(progress, self.path_length.load(Ordering::Relaxed));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::allocator::AllocGuard;

    #[test]
    fn test_cadence_bus_lock_free_reads_and_writes_zero_alloc() {
        let bus = CadenceBus::new();
        bus.set_current_chord(7, ChordQuality::Dominant7);
        bus.set_target_chord(0, ChordQuality::Major);
        bus.set_harmonic_tension(0.85);
        bus.set_resolution_urgency(0.92);
        bus.set_voice_leading_cost(2.4);
        bus.set_predicted_cadence(CadenceResolutionType::Authentic);
        bus.set_path_status(0.66, 3);
        bus.set_auto_progression_active(true);

        {
            let _guard = AllocGuard::new();
            let snap = bus.snapshot();
            assert_eq!(snap.current_root_step, 7);
            assert_eq!(snap.current_quality, ChordQuality::Dominant7);
            assert_eq!(snap.target_root_step, 0);
            assert_eq!(snap.target_quality, ChordQuality::Major);
            assert!((snap.harmonic_tension - 0.85).abs() < 1e-4);
            assert!((snap.resolution_urgency - 0.92).abs() < 1e-4);
            assert!((snap.voice_leading_cost - 2.4).abs() < 1e-4);
            assert_eq!(snap.predicted_cadence, CadenceResolutionType::Authentic);
            assert!(snap.auto_progression_active);
        }
    }

    #[test]
    fn test_cadence_bus_param_bus_round_trip() {
        let cadence_bus = CadenceBus::new();
        let mut param_bus = ParamBus::new();
        for i in 0..10 {
            param_bus.register(ParamId(100 + i), 0.0);
        }

        cadence_bus.set_current_chord(5, ChordQuality::Minor7);
        cadence_bus.set_harmonic_tension(0.45);
        cadence_bus.set_resolution_urgency(0.30);
        cadence_bus.set_voice_leading_cost(1.5);
        cadence_bus.set_predicted_cadence(CadenceResolutionType::Plagal);

        cadence_bus.dispatch_to_param_bus(&param_bus, ParamId(100));

        let sync_bus = CadenceBus::new();
        sync_bus.sync_from_param_bus(&param_bus, ParamId(100));

        let snap = sync_bus.snapshot();
        assert_eq!(snap.current_root_step, 5);
        assert_eq!(snap.current_quality, ChordQuality::Minor7);
        assert!((snap.harmonic_tension - 0.45).abs() < 1e-4);
        assert_eq!(snap.predicted_cadence, CadenceResolutionType::Plagal);
    }
}
