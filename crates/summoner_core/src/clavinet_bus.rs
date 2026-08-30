// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Lock-Free Clavinet Bus Parameter Routing (Milestone 23).
//!
//! Provides thread-safe lock-free exchange of continuous physical modeling electromechanical
//! Clavinet parameters (Pickup Selection, Blend Ratio, Phase Angle, Rubber Anvil Hardness,
//! Yarn Damping, 4-Way Rocker Tone Switches, and Dynamic Auto-Wah Envelope Follower Parameters)
//! across audio, sequencer, and GUI threads.
//!
//! Enforces zero heap allocations in real-time audio threads under `AllocGuard`.

use crate::param_bus::{ParamBus, ParamId};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};

/// Classical and modern electromechanical Clavinet articulation techniques.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u32)]
pub enum ClavinetArticulation {
    /// Classic Stevie 1970s Funk with punchy neck/bridge bite and snappy rubber anvil.
    #[default]
    StevieFunk = 0,
    /// Deep hollow Funk "Quack" with out-of-phase pickup cancellation and Brilliant filter.
    OutPhaseQuack = 1,
    /// Clean and articulate D6 studio setup with balanced tone and natural yarn decay.
    ClassicD6Clean = 2,
    /// Mellow Chamber Clavinet with Soft tone switch and gentle velvet attack.
    MellowChamber = 3,
    /// Screaming Funk Auto-Wah with dynamic envelope follower sweep and sharp resonance.
    ScreamingAutoWah = 4,
    /// Twangy Bridge Lead with aggressive treble emphasis and fast staccato damping.
    TwangyBridge = 5,
    /// Tight percussive staccato funk chop with heavy yarn damping and fast release.
    StaccatoChop = 6,
    /// Vocal formant dynamic auto-wah sweep with peaking resonance.
    VowelWahSweep = 7,
    /// Custom multi-keyframe Catmull-Rom spline trajectory.
    CustomSpline = 8,
}

/// Full nominal tuple parameters for Clavinet articulation.
pub type ClavinetNominalParameters = (
    ClavinetPickupMode,
    f32,  // pickup_blend
    f32,  // pickup_phase_deg
    f32,  // anvil_hardness
    f32,  // yarn_damping
    bool, // brilliant
    bool, // treble
    bool, // medium
    bool, // soft
    bool, // auto_wah_enabled
    f32,  // auto_wah_sensitivity
    f32,  // auto_wah_freq_hz
    f32,  // auto_wah_resonance_q
    f32,  // auto_wah_mix
);

impl ClavinetArticulation {
    pub fn from_u32(val: u32) -> Self {
        match val {
            0 => Self::StevieFunk,
            1 => Self::OutPhaseQuack,
            2 => Self::ClassicD6Clean,
            3 => Self::MellowChamber,
            4 => Self::ScreamingAutoWah,
            5 => Self::TwangyBridge,
            6 => Self::StaccatoChop,
            7 => Self::VowelWahSweep,
            8 => Self::CustomSpline,
            _ => Self::StevieFunk,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::StevieFunk => "Stevie 1970s Superstition Funk",
            Self::OutPhaseQuack => "Out-of-Phase Funk Quack",
            Self::ClassicD6Clean => "Classic D6 Clean Studio",
            Self::MellowChamber => "Mellow Chamber Clavinet",
            Self::ScreamingAutoWah => "Screaming Resonant Auto-Wah",
            Self::TwangyBridge => "Twangy Bridge Lead",
            Self::StaccatoChop => "Tight Staccato Funk Chop",
            Self::VowelWahSweep => "Vocal Formant Auto-Wah Sweep",
            Self::CustomSpline => "Custom Spline Gesture",
        }
    }

    /// Returns nominal default parameters tuple.
    pub fn nominal_parameters(&self) -> ClavinetNominalParameters {
        match self {
            Self::StevieFunk => (ClavinetPickupMode::ParallelInPhase, 0.50, 0.0, 0.85, 0.75, true, true, true, false, false, 0.70, 350.0, 6.0, 0.85),
            Self::OutPhaseQuack => (ClavinetPickupMode::ParallelOutOfPhase, 0.50, 180.0, 0.90, 0.85, true, true, false, false, false, 0.80, 400.0, 7.5, 0.90),
            Self::ClassicD6Clean => (ClavinetPickupMode::ParallelInPhase, 0.55, 0.0, 0.70, 0.65, true, false, true, false, false, 0.60, 300.0, 5.0, 0.75),
            Self::MellowChamber => (ClavinetPickupMode::NeckOnly, 0.0, 0.0, 0.45, 0.50, false, false, false, true, false, 0.40, 250.0, 3.5, 0.60),
            Self::ScreamingAutoWah => (ClavinetPickupMode::ParallelOutOfPhase, 0.50, 180.0, 0.90, 0.80, true, true, false, false, true, 0.88, 420.0, 9.5, 0.95),
            Self::TwangyBridge => (ClavinetPickupMode::BridgeOnly, 1.0, 0.0, 0.95, 0.90, true, true, false, false, false, 0.75, 450.0, 6.0, 0.80),
            Self::StaccatoChop => (ClavinetPickupMode::ParallelInPhase, 0.60, 0.0, 0.92, 0.95, true, true, true, false, false, 0.75, 380.0, 5.5, 0.80),
            Self::VowelWahSweep => (ClavinetPickupMode::ParallelInPhase, 0.50, 0.0, 0.80, 0.70, true, false, true, false, true, 0.85, 300.0, 10.0, 0.90),
            Self::CustomSpline => (ClavinetPickupMode::ParallelInPhase, 0.50, 0.0, 0.85, 0.75, true, true, true, false, false, 0.70, 350.0, 6.0, 0.85),
        }
    }
}

/// Dual electromagnetic pickup selection mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u32)]
pub enum ClavinetPickupMode {
    NeckOnly = 0,
    BridgeOnly = 1,
    #[default]
    ParallelInPhase = 2,
    ParallelOutOfPhase = 3,
}

impl ClavinetPickupMode {
    pub fn from_u32(val: u32) -> Self {
        match val {
            0 => Self::NeckOnly,
            1 => Self::BridgeOnly,
            2 => Self::ParallelInPhase,
            3 => Self::ParallelOutOfPhase,
            _ => Self::ParallelInPhase,
        }
    }
}

/// Instantaneous lock-free snapshot of Clavinet bus parameters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ClavinetBusSnapshot {
    pub articulation: ClavinetArticulation,
    pub pickup_mode: ClavinetPickupMode,
    pub pickup_blend: f32,
    pub pickup_phase_deg: f32,
    pub anvil_hardness: f32,
    pub yarn_damping: f32,
    pub tone_switches_mask: u32,
    pub auto_wah_enabled: bool,
    pub auto_wah_sensitivity: f32,
    pub auto_wah_freq_hz: f32,
    pub auto_wah_resonance_q: f32,
    pub auto_wah_mix: f32,
    pub gesture_progress: f32,
    pub is_active: bool,
}

impl Default for ClavinetBusSnapshot {
    fn default() -> Self {
        let (mode, blend, phase, hardness, damping, br, tr, md, sf, wah_en, wah_sens, wah_freq, wah_res, wah_mix) =
            ClavinetArticulation::StevieFunk.nominal_parameters();

        let mut mask = 0u32;
        if br { mask |= 1; }
        if tr { mask |= 2; }
        if md { mask |= 4; }
        if sf { mask |= 8; }

        Self {
            articulation: ClavinetArticulation::StevieFunk,
            pickup_mode: mode,
            pickup_blend: blend,
            pickup_phase_deg: phase,
            anvil_hardness: hardness,
            yarn_damping: damping,
            tone_switches_mask: mask,
            auto_wah_enabled: wah_en,
            auto_wah_sensitivity: wah_sens,
            auto_wah_freq_hz: wah_freq,
            auto_wah_resonance_q: wah_res,
            auto_wah_mix: wah_mix,
            gesture_progress: 0.0,
            is_active: false,
        }
    }
}

/// Lock-free electromechanical Clavinet parameter routing bus.
pub struct ClavinetBus {
    articulation_raw: AtomicU32,
    pickup_mode_raw: AtomicU32,
    pickup_blend_bits: AtomicU32,
    pickup_phase_bits: AtomicU32,
    anvil_hardness_bits: AtomicU32,
    yarn_damping_bits: AtomicU32,
    tone_switches_mask: AtomicU32,
    auto_wah_flags: AtomicU32, // bit 0: enabled
    auto_wah_sens_bits: AtomicU32,
    auto_wah_freq_bits: AtomicU32,
    auto_wah_res_bits: AtomicU32,
    auto_wah_mix_bits: AtomicU32,
    progress_bits: AtomicU32,
    flags: AtomicU32,
}

impl Default for ClavinetBus {
    fn default() -> Self {
        Self::new()
    }
}

impl ClavinetBus {
    /// Creates a new ClavinetBus initialized to StevieFunk defaults.
    pub fn new() -> Self {
        let (mode, blend, phase, hardness, damping, br, tr, md, sf, wah_en, wah_sens, wah_freq, wah_res, wah_mix) =
            ClavinetArticulation::StevieFunk.nominal_parameters();

        let mut mask = 0u32;
        if br { mask |= 1; }
        if tr { mask |= 2; }
        if md { mask |= 4; }
        if sf { mask |= 8; }

        Self {
            articulation_raw: AtomicU32::new(ClavinetArticulation::StevieFunk as u32),
            pickup_mode_raw: AtomicU32::new(mode as u32),
            pickup_blend_bits: AtomicU32::new(blend.to_bits()),
            pickup_phase_bits: AtomicU32::new(phase.to_bits()),
            anvil_hardness_bits: AtomicU32::new(hardness.to_bits()),
            yarn_damping_bits: AtomicU32::new(damping.to_bits()),
            tone_switches_mask: AtomicU32::new(mask),
            auto_wah_flags: AtomicU32::new(if wah_en { 1 } else { 0 }),
            auto_wah_sens_bits: AtomicU32::new(wah_sens.to_bits()),
            auto_wah_freq_bits: AtomicU32::new(wah_freq.to_bits()),
            auto_wah_res_bits: AtomicU32::new(wah_res.to_bits()),
            auto_wah_mix_bits: AtomicU32::new(wah_mix.to_bits()),
            progress_bits: AtomicU32::new(0.0f32.to_bits()),
            flags: AtomicU32::new(0),
        }
    }

    /// Sets active articulation and loads nominal parameters.
    pub fn set_articulation(&self, articulation: ClavinetArticulation) {
        self.articulation_raw.store(articulation as u32, Ordering::Release);
        let (mode, blend, phase, hardness, damping, br, tr, md, sf, wah_en, wah_sens, wah_freq, wah_res, wah_mix) =
            articulation.nominal_parameters();

        let mut mask = 0u32;
        if br { mask |= 1; }
        if tr { mask |= 2; }
        if md { mask |= 4; }
        if sf { mask |= 8; }

        self.pickup_mode_raw.store(mode as u32, Ordering::Release);
        self.pickup_blend_bits.store(blend.to_bits(), Ordering::Release);
        self.pickup_phase_bits.store(phase.to_bits(), Ordering::Release);
        self.anvil_hardness_bits.store(hardness.to_bits(), Ordering::Release);
        self.yarn_damping_bits.store(damping.to_bits(), Ordering::Release);
        self.tone_switches_mask.store(mask, Ordering::Release);
        self.auto_wah_flags.store(if wah_en { 1 } else { 0 }, Ordering::Release);
        self.auto_wah_sens_bits.store(wah_sens.to_bits(), Ordering::Release);
        self.auto_wah_freq_bits.store(wah_freq.to_bits(), Ordering::Release);
        self.auto_wah_res_bits.store(wah_res.to_bits(), Ordering::Release);
        self.auto_wah_mix_bits.store(wah_mix.to_bits(), Ordering::Release);
    }

    /// Gets active articulation.
    pub fn get_articulation(&self) -> ClavinetArticulation {
        ClavinetArticulation::from_u32(self.articulation_raw.load(Ordering::Acquire))
    }

    /// Sets electronic pickup and tone switch properties.
    pub fn set_electronics(&self, mode: ClavinetPickupMode, blend: f32, phase_deg: f32, switches_mask: u32) {
        self.pickup_mode_raw.store(mode as u32, Ordering::Release);
        self.pickup_blend_bits.store(blend.clamp(0.0, 1.0).to_bits(), Ordering::Release);
        self.pickup_phase_bits.store(phase_deg.clamp(0.0, 180.0).to_bits(), Ordering::Release);
        self.tone_switches_mask.store(switches_mask & 0x0F, Ordering::Release);
    }

    /// Sets mechanical properties.
    pub fn set_mechanics(&self, hardness: f32, damping: f32) {
        self.anvil_hardness_bits.store(hardness.clamp(0.0, 1.0).to_bits(), Ordering::Release);
        self.yarn_damping_bits.store(damping.clamp(0.0, 1.0).to_bits(), Ordering::Release);
    }

    /// Sets auto-wah properties.
    pub fn set_auto_wah(&self, enabled: bool, sens: f32, freq: f32, res: f32, mix: f32) {
        self.auto_wah_flags.store(if enabled { 1 } else { 0 }, Ordering::Release);
        self.auto_wah_sens_bits.store(sens.clamp(0.0, 1.0).to_bits(), Ordering::Release);
        self.auto_wah_freq_bits.store(freq.clamp(50.0, 4000.0).to_bits(), Ordering::Release);
        self.auto_wah_res_bits.store(res.clamp(0.5, 25.0).to_bits(), Ordering::Release);
        self.auto_wah_mix_bits.store(mix.clamp(0.0, 1.0).to_bits(), Ordering::Release);
    }

    /// Sets gesture progress $[0.0 ..= 1.0]$.
    pub fn set_progress(&self, progress: f32, is_active: bool) {
        self.progress_bits.store(progress.clamp(0.0, 1.0).to_bits(), Ordering::Release);

        let mut curr = self.flags.load(Ordering::Acquire);
        loop {
            let next = if is_active { curr | 1 } else { curr & !1 };
            match self.flags.compare_exchange_weak(curr, next, Ordering::Release, Ordering::Acquire) {
                Ok(_) => break,
                Err(actual) => curr = actual,
            }
        }
    }

    /// Captures a complete atomic snapshot of the Clavinet bus.
    pub fn snapshot(&self) -> ClavinetBusSnapshot {
        let articulation = self.get_articulation();
        let mode = ClavinetPickupMode::from_u32(self.pickup_mode_raw.load(Ordering::Acquire));
        let blend = f32::from_bits(self.pickup_blend_bits.load(Ordering::Acquire));
        let phase = f32::from_bits(self.pickup_phase_bits.load(Ordering::Acquire));
        let hardness = f32::from_bits(self.anvil_hardness_bits.load(Ordering::Acquire));
        let damping = f32::from_bits(self.yarn_damping_bits.load(Ordering::Acquire));
        let switches_mask = self.tone_switches_mask.load(Ordering::Acquire);
        let wah_flags = self.auto_wah_flags.load(Ordering::Acquire);
        let wah_sens = f32::from_bits(self.auto_wah_sens_bits.load(Ordering::Acquire));
        let wah_freq = f32::from_bits(self.auto_wah_freq_bits.load(Ordering::Acquire));
        let wah_res = f32::from_bits(self.auto_wah_res_bits.load(Ordering::Acquire));
        let wah_mix = f32::from_bits(self.auto_wah_mix_bits.load(Ordering::Acquire));
        let progress = f32::from_bits(self.progress_bits.load(Ordering::Acquire));
        let flags = self.flags.load(Ordering::Acquire);

        ClavinetBusSnapshot {
            articulation,
            pickup_mode: mode,
            pickup_blend: blend,
            pickup_phase_deg: phase,
            anvil_hardness: hardness,
            yarn_damping: damping,
            tone_switches_mask: switches_mask,
            auto_wah_enabled: (wah_flags & 1) != 0,
            auto_wah_sensitivity: wah_sens,
            auto_wah_freq_hz: wah_freq,
            auto_wah_resonance_q: wah_res,
            auto_wah_mix: wah_mix,
            gesture_progress: progress,
            is_active: (flags & 1) != 0,
        }
    }

    /// Applies a complete snapshot into this bus.
    pub fn apply_snapshot(&self, snap: &ClavinetBusSnapshot) {
        self.articulation_raw.store(snap.articulation as u32, Ordering::Release);
        self.pickup_mode_raw.store(snap.pickup_mode as u32, Ordering::Release);
        self.pickup_blend_bits.store(snap.pickup_blend.to_bits(), Ordering::Release);
        self.pickup_phase_bits.store(snap.pickup_phase_deg.to_bits(), Ordering::Release);
        self.anvil_hardness_bits.store(snap.anvil_hardness.to_bits(), Ordering::Release);
        self.yarn_damping_bits.store(snap.yarn_damping.to_bits(), Ordering::Release);
        self.tone_switches_mask.store(snap.tone_switches_mask, Ordering::Release);
        self.auto_wah_flags.store(if snap.auto_wah_enabled { 1 } else { 0 }, Ordering::Release);
        self.auto_wah_sens_bits.store(snap.auto_wah_sensitivity.to_bits(), Ordering::Release);
        self.auto_wah_freq_bits.store(snap.auto_wah_freq_hz.to_bits(), Ordering::Release);
        self.auto_wah_res_bits.store(snap.auto_wah_resonance_q.to_bits(), Ordering::Release);
        self.auto_wah_mix_bits.store(snap.auto_wah_mix.to_bits(), Ordering::Release);
        self.progress_bits.store(snap.gesture_progress.to_bits(), Ordering::Release);
        self.flags.store(if snap.is_active { 1 } else { 0 }, Ordering::Release);
    }

    /// Dispatches current Clavinet bus snapshot values to central `ParamBus`.
    pub fn dispatch_to_param_bus(&self, bus: &ParamBus, base_id: ParamId) {
        let snap = self.snapshot();
        bus.set(ParamId(base_id.0), snap.articulation as u32 as f32);
        bus.set(ParamId(base_id.0 + 1), snap.pickup_mode as u32 as f32);
        bus.set(ParamId(base_id.0 + 2), snap.pickup_blend);
        bus.set(ParamId(base_id.0 + 3), snap.pickup_phase_deg);
        bus.set(ParamId(base_id.0 + 4), snap.anvil_hardness);
        bus.set(ParamId(base_id.0 + 5), snap.yarn_damping);
        bus.set(ParamId(base_id.0 + 6), snap.tone_switches_mask as f32);
        bus.set(ParamId(base_id.0 + 7), if snap.auto_wah_enabled { 1.0 } else { 0.0 });
        bus.set(ParamId(base_id.0 + 8), snap.auto_wah_sensitivity);
        bus.set(ParamId(base_id.0 + 9), snap.auto_wah_freq_hz);
        bus.set(ParamId(base_id.0 + 10), snap.auto_wah_resonance_q);
        bus.set(ParamId(base_id.0 + 11), snap.auto_wah_mix);
        bus.set(ParamId(base_id.0 + 12), snap.gesture_progress);
    }

    /// Pulls updated parameter values from a central `ParamBus`.
    pub fn sync_from_param_bus(&self, bus: &ParamBus, base_id: ParamId) {
        let art_idx = bus.get(ParamId(base_id.0)).unwrap_or(0.0).max(0.0).round() as u32;
        let mode_idx = bus.get(ParamId(base_id.0 + 1)).unwrap_or(2.0).round() as u32;
        let blend = bus.get(ParamId(base_id.0 + 2)).unwrap_or(0.50);
        let phase = bus.get(ParamId(base_id.0 + 3)).unwrap_or(0.0);
        let hardness = bus.get(ParamId(base_id.0 + 4)).unwrap_or(0.85);
        let damping = bus.get(ParamId(base_id.0 + 5)).unwrap_or(0.75);
        let mask = bus.get(ParamId(base_id.0 + 6)).unwrap_or(3.0).round() as u32;
        let wah_en = bus.get(ParamId(base_id.0 + 7)).unwrap_or(0.0) > 0.5;
        let wah_sens = bus.get(ParamId(base_id.0 + 8)).unwrap_or(0.70);
        let wah_freq = bus.get(ParamId(base_id.0 + 9)).unwrap_or(350.0);
        let wah_res = bus.get(ParamId(base_id.0 + 10)).unwrap_or(6.0);
        let wah_mix = bus.get(ParamId(base_id.0 + 11)).unwrap_or(0.85);
        let prog = bus.get(ParamId(base_id.0 + 12)).unwrap_or(0.0);

        self.articulation_raw.store(art_idx, Ordering::Release);
        self.set_electronics(ClavinetPickupMode::from_u32(mode_idx), blend, phase, mask);
        self.set_mechanics(hardness, damping);
        self.set_auto_wah(wah_en, wah_sens, wah_freq, wah_res, wah_mix);
        self.set_progress(prog, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::allocator::AllocGuard;

    #[test]
    fn test_clavinet_bus_snapshot_and_zero_allocation() {
        let bus = ClavinetBus::new();
        bus.set_articulation(ClavinetArticulation::OutPhaseQuack);
        bus.set_auto_wah(true, 0.90, 450.0, 8.5, 0.95);
        bus.set_progress(0.65, true);

        {
            let _guard = AllocGuard::new();
            let snap = bus.snapshot();
            assert_eq!(snap.articulation, ClavinetArticulation::OutPhaseQuack);
            assert_eq!(snap.pickup_mode, ClavinetPickupMode::ParallelOutOfPhase);
            assert!(snap.auto_wah_enabled);
            assert!((snap.auto_wah_freq_hz - 450.0).abs() < 1e-4);
            assert!(snap.is_active);
        }
    }

    #[test]
    fn test_clavinet_bus_param_bus_round_trip() {
        let clav_bus = ClavinetBus::new();
        let mut param_bus = ParamBus::new();
        for i in 0..20 {
            param_bus.register(ParamId(980 + i), 0.0);
        }

        clav_bus.set_articulation(ClavinetArticulation::ScreamingAutoWah);
        clav_bus.set_electronics(ClavinetPickupMode::ParallelOutOfPhase, 0.55, 180.0, 3);
        clav_bus.dispatch_to_param_bus(&param_bus, ParamId(980));

        let sync_bus = ClavinetBus::new();
        sync_bus.sync_from_param_bus(&param_bus, ParamId(980));

        let snap = sync_bus.snapshot();
        assert_eq!(snap.articulation, ClavinetArticulation::ScreamingAutoWah);
        assert_eq!(snap.pickup_mode, ClavinetPickupMode::ParallelOutOfPhase);
        assert!((snap.pickup_blend - 0.55).abs() < 1e-4);
    }
}
