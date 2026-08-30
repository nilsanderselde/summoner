// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Lock-Free Formant Bus & Real-Time Phonetic Parameter Routing (Milestone 15).
//!
//! Provides thread-safe lock-free exchange of International Phonetic Alphabet (IPA)
//! vowel formant frequencies ($F_1, F_2, F_3, F_4$), bandwidths, articulatory tongue/lip/velum
//! geometry, and glottal flow properties across audio, sequencer, and GUI threads.
//!
//! Enforces zero heap allocations in real-time audio threads under `AllocGuard`.

use crate::param_bus::{ParamBus, ParamId};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};

/// Standard International Phonetic Alphabet (IPA) vowel symbols for atomic encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u32)]
pub enum IpaVowel {
    #[default]
    CloseFrontI = 0,    // /i/ (beet)
    CloseMidFrontE = 1, // /e/ (bait)
    OpenMidFrontEps = 2,// /ɛ/ (bet)
    NearOpenFrontAsh = 3,// /æ/ (bat)
    OpenFrontA = 4,     // /a/ (father)
    OpenBackA = 5,      // /ɑ/ (palm)
    OpenMidBackO = 6,   // /ɔ/ (bought)
    CloseMidBackO = 7,  // /o/ (boat)
    CloseBackU = 8,     // /u/ (boot)
    MidCentralSchwa = 9,// /ə/ (sofa)
    CloseFrontY = 10,   // /y/ (French "tu")
    CloseMidFrontOe = 11,// /ø/ (French "feu")
}

impl IpaVowel {
    pub fn from_u32(val: u32) -> Self {
        match val {
            0 => Self::CloseFrontI,
            1 => Self::CloseMidFrontE,
            2 => Self::OpenMidFrontEps,
            3 => Self::NearOpenFrontAsh,
            4 => Self::OpenFrontA,
            5 => Self::OpenBackA,
            6 => Self::OpenMidBackO,
            7 => Self::CloseMidBackO,
            8 => Self::CloseBackU,
            9 => Self::MidCentralSchwa,
            10 => Self::CloseFrontY,
            11 => Self::CloseMidFrontOe,
            _ => Self::CloseFrontI,
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Self::CloseFrontI => "/i/",
            Self::CloseMidFrontE => "/e/",
            Self::OpenMidFrontEps => "/ɛ/",
            Self::NearOpenFrontAsh => "/æ/",
            Self::OpenFrontA => "/a/",
            Self::OpenBackA => "/ɑ/",
            Self::OpenMidBackO => "/ɔ/",
            Self::CloseMidBackO => "/o/",
            Self::CloseBackU => "/u/",
            Self::MidCentralSchwa => "/ə/",
            Self::CloseFrontY => "/y/",
            Self::CloseMidFrontOe => "/ø/",
        }
    }

    pub fn nominal_formants(&self) -> (f32, f32, f32, f32) {
        match self {
            Self::CloseFrontI => (270.0, 2290.0, 3010.0, 3500.0),
            Self::CloseMidFrontE => (390.0, 2300.0, 2850.0, 3500.0),
            Self::OpenMidFrontEps => (530.0, 1840.0, 2480.0, 3500.0),
            Self::NearOpenFrontAsh => (660.0, 1720.0, 2410.0, 3500.0),
            Self::OpenFrontA => (730.0, 1090.0, 2440.0, 3500.0),
            Self::OpenBackA => (700.0, 1220.0, 2600.0, 3500.0),
            Self::OpenMidBackO => (570.0, 840.0, 2410.0, 3500.0),
            Self::CloseMidBackO => (460.0, 1100.0, 2500.0, 3500.0),
            Self::CloseBackU => (300.0, 870.0, 2240.0, 3500.0),
            Self::MidCentralSchwa => (500.0, 1500.0, 2500.0, 3500.0),
            Self::CloseFrontY => (270.0, 1900.0, 2100.0, 3500.0),
            Self::CloseMidFrontOe => (400.0, 1600.0, 2250.0, 3500.0),
        }
    }
}

/// Instantaneous lock-free snapshot of formant bus parameters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FormantBusSnapshot {
    pub active_vowel: IpaVowel,
    pub f1_hz: f32,
    pub f2_hz: f32,
    pub f3_hz: f32,
    pub f4_hz: f32,
    pub tongue_position: f32,
    pub tongue_height: f32,
    pub lip_opening: f32,
    pub velum_opening: f32,
    pub glottal_f0_hz: f32,
    pub aspiration_level: f32,
    pub trajectory_progress: f32,
    pub morph_active: bool,
}

impl Default for FormantBusSnapshot {
    fn default() -> Self {
        Self {
            active_vowel: IpaVowel::CloseFrontI,
            f1_hz: 270.0,
            f2_hz: 2290.0,
            f3_hz: 3010.0,
            f4_hz: 3500.0,
            tongue_position: 0.70,
            tongue_height: 0.85,
            lip_opening: 1.0,
            velum_opening: 0.0,
            glottal_f0_hz: 140.0,
            aspiration_level: 0.04,
            trajectory_progress: 0.0,
            morph_active: false,
        }
    }
}

/// Lock-free phonetic formant parameter bus.
pub struct FormantBus {
    active_vowel: AtomicU32,
    f1_bits: AtomicU32,
    f2_bits: AtomicU32,
    f3_bits: AtomicU32,
    f4_bits: AtomicU32,
    tongue_pos_bits: AtomicU32,
    tongue_height_bits: AtomicU32,
    lip_opening_bits: AtomicU32,
    velum_opening_bits: AtomicU32,
    glottal_f0_bits: AtomicU32,
    aspiration_bits: AtomicU32,
    progress_bits: AtomicU32,
    flags: AtomicU32,
}

impl Default for FormantBus {
    fn default() -> Self {
        Self::new()
    }
}

impl FormantBus {
    /// Creates a new FormantBus initialized to default vowel /i/.
    pub fn new() -> Self {
        let (f1, f2, f3, f4) = IpaVowel::CloseFrontI.nominal_formants();
        Self {
            active_vowel: AtomicU32::new(IpaVowel::CloseFrontI as u32),
            f1_bits: AtomicU32::new(f1.to_bits()),
            f2_bits: AtomicU32::new(f2.to_bits()),
            f3_bits: AtomicU32::new(f3.to_bits()),
            f4_bits: AtomicU32::new(f4.to_bits()),
            tongue_pos_bits: AtomicU32::new(0.70f32.to_bits()),
            tongue_height_bits: AtomicU32::new(0.85f32.to_bits()),
            lip_opening_bits: AtomicU32::new(1.0f32.to_bits()),
            velum_opening_bits: AtomicU32::new(0.0f32.to_bits()),
            glottal_f0_bits: AtomicU32::new(140.0f32.to_bits()),
            aspiration_bits: AtomicU32::new(0.04f32.to_bits()),
            progress_bits: AtomicU32::new(0.0f32.to_bits()),
            flags: AtomicU32::new(0),
        }
    }

    /// Sets the active IPA vowel phoneme.
    pub fn set_active_vowel(&self, vowel: IpaVowel) {
        self.active_vowel.store(vowel as u32, Ordering::Release);
        let (f1, f2, f3, f4) = vowel.nominal_formants();
        self.set_formants(f1, f2, f3, f4);
    }

    /// Gets the active IPA vowel phoneme.
    pub fn get_active_vowel(&self) -> IpaVowel {
        IpaVowel::from_u32(self.active_vowel.load(Ordering::Acquire))
    }

    /// Sets formant frequencies $F_1, F_2, F_3, F_4$ in Hz.
    pub fn set_formants(&self, f1: f32, f2: f32, f3: f32, f4: f32) {
        self.f1_bits.store(f1.to_bits(), Ordering::Release);
        self.f2_bits.store(f2.to_bits(), Ordering::Release);
        self.f3_bits.store(f3.to_bits(), Ordering::Release);
        self.f4_bits.store(f4.to_bits(), Ordering::Release);
    }

    /// Gets formant frequencies $(F_1, F_2, F_3, F_4)$ in Hz.
    pub fn get_formants(&self) -> (f32, f32, f32, f32) {
        (
            f32::from_bits(self.f1_bits.load(Ordering::Acquire)),
            f32::from_bits(self.f2_bits.load(Ordering::Acquire)),
            f32::from_bits(self.f3_bits.load(Ordering::Acquire)),
            f32::from_bits(self.f4_bits.load(Ordering::Acquire)),
        )
    }

    /// Sets articulatory parameters: tongue position $[0..1]$, tongue height $[0..1]$, lip opening $[0.1..2.5]$, velum $[0..1]$.
    pub fn set_articulators(&self, tongue_pos: f32, tongue_height: f32, lip_opening: f32, velum: f32) {
        self.tongue_pos_bits.store(tongue_pos.clamp(0.0, 1.0).to_bits(), Ordering::Release);
        self.tongue_height_bits.store(tongue_height.clamp(0.0, 1.0).to_bits(), Ordering::Release);
        self.lip_opening_bits.store(lip_opening.clamp(0.1, 2.5).to_bits(), Ordering::Release);
        self.velum_opening_bits.store(velum.clamp(0.0, 1.0).to_bits(), Ordering::Release);
    }

    /// Gets articulatory parameters $(pos, height, lip, velum)$.
    pub fn get_articulators(&self) -> (f32, f32, f32, f32) {
        (
            f32::from_bits(self.tongue_pos_bits.load(Ordering::Acquire)),
            f32::from_bits(self.tongue_height_bits.load(Ordering::Acquire)),
            f32::from_bits(self.lip_opening_bits.load(Ordering::Acquire)),
            f32::from_bits(self.velum_opening_bits.load(Ordering::Acquire)),
        )
    }

    /// Sets glottal fundamental frequency ($F_0$) in Hz and turbulent aspiration noise level.
    pub fn set_glottal_source(&self, f0_hz: f32, aspiration: f32) {
        self.glottal_f0_bits.store(f0_hz.clamp(20.0, 2000.0).to_bits(), Ordering::Release);
        self.aspiration_bits.store(aspiration.clamp(0.0, 1.0).to_bits(), Ordering::Release);
    }

    /// Sets trajectory morph progress $[0.0 ..= 1.0]$.
    pub fn set_trajectory_progress(&self, progress: f32, morph_active: bool) {
        let p = progress.clamp(0.0, 1.0);
        self.progress_bits.store(p.to_bits(), Ordering::Release);

        let mut curr = self.flags.load(Ordering::Acquire);
        loop {
            let next = if morph_active { curr | 1 } else { curr & !1 };
            match self.flags.compare_exchange_weak(curr, next, Ordering::Release, Ordering::Acquire) {
                Ok(_) => break,
                Err(actual) => curr = actual,
            }
        }
    }

    /// Captures a complete atomic snapshot of the formant bus.
    pub fn snapshot(&self) -> FormantBusSnapshot {
        let active_vowel = self.get_active_vowel();
        let (f1_hz, f2_hz, f3_hz, f4_hz) = self.get_formants();
        let (tongue_position, tongue_height, lip_opening, velum_opening) = self.get_articulators();
        let flags = self.flags.load(Ordering::Acquire);

        FormantBusSnapshot {
            active_vowel,
            f1_hz,
            f2_hz,
            f3_hz,
            f4_hz,
            tongue_position,
            tongue_height,
            lip_opening,
            velum_opening,
            glottal_f0_hz: f32::from_bits(self.glottal_f0_bits.load(Ordering::Acquire)),
            aspiration_level: f32::from_bits(self.aspiration_bits.load(Ordering::Acquire)),
            trajectory_progress: f32::from_bits(self.progress_bits.load(Ordering::Acquire)),
            morph_active: (flags & 1) != 0,
        }
    }

    /// Dispatches formant bus states into `ParamBus` handles.
    pub fn dispatch_to_param_bus(&self, bus: &ParamBus, base_param_id: ParamId) {
        let snap = self.snapshot();
        bus.set(ParamId(base_param_id.0), snap.active_vowel as u32 as f32);
        bus.set(ParamId(base_param_id.0 + 1), snap.f1_hz);
        bus.set(ParamId(base_param_id.0 + 2), snap.f2_hz);
        bus.set(ParamId(base_param_id.0 + 3), snap.f3_hz);
        bus.set(ParamId(base_param_id.0 + 4), snap.f4_hz);
        bus.set(ParamId(base_param_id.0 + 5), snap.tongue_position);
        bus.set(ParamId(base_param_id.0 + 6), snap.tongue_height);
        bus.set(ParamId(base_param_id.0 + 7), snap.lip_opening);
        bus.set(ParamId(base_param_id.0 + 8), snap.velum_opening);
        bus.set(ParamId(base_param_id.0 + 9), snap.glottal_f0_hz);
    }

    /// Synchronizes formant bus states from `ParamBus` handles.
    pub fn sync_from_param_bus(&self, bus: &ParamBus, base_param_id: ParamId) {
        let vowel_idx = bus.get(ParamId(base_param_id.0)).unwrap_or(0.0).max(0.0).round() as u32;
        let f1 = bus.get(ParamId(base_param_id.0 + 1)).unwrap_or(270.0);
        let f2 = bus.get(ParamId(base_param_id.0 + 2)).unwrap_or(2290.0);
        let f3 = bus.get(ParamId(base_param_id.0 + 3)).unwrap_or(3010.0);
        let f4 = bus.get(ParamId(base_param_id.0 + 4)).unwrap_or(3500.0);
        let pos = bus.get(ParamId(base_param_id.0 + 5)).unwrap_or(0.70);
        let height = bus.get(ParamId(base_param_id.0 + 6)).unwrap_or(0.85);
        let lip = bus.get(ParamId(base_param_id.0 + 7)).unwrap_or(1.0);
        let velum = bus.get(ParamId(base_param_id.0 + 8)).unwrap_or(0.0);
        let f0 = bus.get(ParamId(base_param_id.0 + 9)).unwrap_or(140.0);

        self.active_vowel.store(vowel_idx, Ordering::Release);
        self.set_formants(f1, f2, f3, f4);
        self.set_articulators(pos, height, lip, velum);
        self.set_glottal_source(f0, self.snapshot().aspiration_level);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::allocator::AllocGuard;

    #[test]
    fn test_formant_bus_atomic_read_write_zero_allocation() {
        let bus = FormantBus::new();
        bus.set_active_vowel(IpaVowel::OpenFrontA);
        bus.set_articulators(0.25, 0.40, 1.8, 0.15);
        bus.set_glottal_source(220.0, 0.08);
        bus.set_trajectory_progress(0.75, true);

        {
            let _guard = AllocGuard::new();
            let snap = bus.snapshot();
            assert_eq!(snap.active_vowel, IpaVowel::OpenFrontA);
            assert_eq!(snap.f1_hz, 730.0);
            assert_eq!(snap.f2_hz, 1090.0);
            assert!((snap.tongue_position - 0.25).abs() < 1e-4);
            assert!((snap.lip_opening - 1.8).abs() < 1e-4);
            assert!((snap.glottal_f0_hz - 220.0).abs() < 1e-4);
            assert!(snap.morph_active);
        }
    }

    #[test]
    fn test_formant_bus_param_bus_round_trip() {
        let formant_bus = FormantBus::new();
        let mut param_bus = ParamBus::new();
        for i in 0..15 {
            param_bus.register(ParamId(400 + i), 0.0);
        }

        formant_bus.set_active_vowel(IpaVowel::CloseBackU);
        formant_bus.set_articulators(0.15, 0.90, 0.35, 0.0);
        formant_bus.dispatch_to_param_bus(&param_bus, ParamId(400));

        let sync_bus = FormantBus::new();
        sync_bus.sync_from_param_bus(&param_bus, ParamId(400));

        let snap = sync_bus.snapshot();
        assert_eq!(snap.active_vowel, IpaVowel::CloseBackU);
        assert_eq!(snap.f1_hz, 300.0);
        assert_eq!(snap.f2_hz, 870.0);
        assert!((snap.tongue_position - 0.15).abs() < 1e-4);
    }
}
