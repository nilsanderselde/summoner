// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Lock-Free Plectrum Bus & Real-Time Plucked Articulation Parameter Routing (Milestone 18).
//!
//! Provides thread-safe lock-free exchange of continuous physical plucking and striking articulation vectors
//! (Pluck Position $\beta$, Strike Velocity $v_s$, Hammer Hardness, Plectrum Angle $\theta_p$, Palm Mute Damping,
//! Sympathetic Bleed Ratio, String Stiffness, and Technique Enums) across audio, sequencer, and GUI threads.
//!
//! Enforces zero heap allocations in real-time audio threads under `AllocGuard`.

use crate::param_bus::{ParamBus, ParamId};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};

/// Classical and modern extended plucking and striking articulation techniques.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u32)]
pub enum PluckArticulation {
    /// Standard plectrum pick or fingertip pluck.
    #[default]
    PluckNormal = 0,
    /// Classical Spanish Apoyando (Rest Stroke) with solid fleshy attack.
    FingerRestStroke = 1,
    /// Classical Spanish Tirando (Free Stroke) with lighter transient.
    FingerFreeStroke = 2,
    /// Struck felt hammer contact (Piano, Dulcimer, Rhodes).
    HammerStruck = 3,
    /// Palm muted dampened pick stroke (Heavy rock chug, muffled nylon).
    PalmMute = 4,
    /// Rapid continuous tremolo plectrum alternation (Mandolin, Flamenco).
    TremoloPluck = 5,
    /// Bartók snap pizzicato / Slap bass popping off the fretboard.
    SnapPizzicato = 6,
    /// Harpsichord quill pluck with sharp release.
    HarpsichordQuill = 7,
    /// Sitar / Koto flat bridge jawari buzzing articulation.
    SitarJawariBuzz = 8,
}

impl PluckArticulation {
    pub fn from_u32(val: u32) -> Self {
        match val {
            0 => Self::PluckNormal,
            1 => Self::FingerRestStroke,
            2 => Self::FingerFreeStroke,
            3 => Self::HammerStruck,
            4 => Self::PalmMute,
            5 => Self::TremoloPluck,
            6 => Self::SnapPizzicato,
            7 => Self::HarpsichordQuill,
            8 => Self::SitarJawariBuzz,
            _ => Self::PluckNormal,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::PluckNormal => "Normal Pluck",
            Self::FingerRestStroke => "Apoyando (Rest Stroke)",
            Self::FingerFreeStroke => "Tirando (Free Stroke)",
            Self::HammerStruck => "Hammer Struck",
            Self::PalmMute => "Palm Mute",
            Self::TremoloPluck => "Tremolo Pluck",
            Self::SnapPizzicato => "Snap / Slap Pop",
            Self::HarpsichordQuill => "Harpsichord Quill",
            Self::SitarJawariBuzz => "Jawari Buzz",
        }
    }

    /// Returns nominal default $(\beta, \text{velocity}, \text{hardness}, \text{palm\_mute}, \text{sympathetic\_bleed})$ parameters.
    pub fn nominal_parameters(&self) -> (f32, f32, f32, f32, f32) {
        match self {
            Self::PluckNormal => (0.15, 0.75, 0.50, 0.00, 0.15),
            Self::FingerRestStroke => (0.20, 0.85, 0.35, 0.00, 0.10),
            Self::FingerFreeStroke => (0.18, 0.65, 0.40, 0.00, 0.12),
            Self::HammerStruck => (0.12, 0.80, 0.60, 0.00, 0.35),
            Self::PalmMute => (0.10, 0.90, 0.70, 0.80, 0.05),
            Self::TremoloPluck => (0.14, 0.70, 0.55, 0.00, 0.18),
            Self::SnapPizzicato => (0.08, 1.00, 0.90, 0.00, 0.25),
            Self::HarpsichordQuill => (0.10, 0.70, 0.85, 0.00, 0.12),
            Self::SitarJawariBuzz => (0.22, 0.75, 0.65, 0.00, 0.30),
        }
    }
}

/// Instantaneous lock-free snapshot of plectrum & hammer bus parameters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PlectrumBusSnapshot {
    pub articulation: PluckArticulation,
    pub pluck_position_beta: f32,
    pub strike_velocity: f32,
    pub hammer_hardness: f32,
    pub plectrum_angle_rad: f32,
    pub palm_mute_damping: f32,
    pub sympathetic_bleed: f32,
    pub string_stiffness: f32,
    pub gesture_progress: f32,
    pub is_active: bool,
}

impl Default for PlectrumBusSnapshot {
    fn default() -> Self {
        let (beta, vel, hard, mute, bleed) = PluckArticulation::PluckNormal.nominal_parameters();
        Self {
            articulation: PluckArticulation::PluckNormal,
            pluck_position_beta: beta,
            strike_velocity: vel,
            hammer_hardness: hard,
            plectrum_angle_rad: 0.0,
            palm_mute_damping: mute,
            sympathetic_bleed: bleed,
            string_stiffness: 0.25,
            gesture_progress: 0.0,
            is_active: false,
        }
    }
}

/// Lock-free physical plucked & struck string parameter routing bus.
pub struct PlectrumBus {
    articulation_raw: AtomicU32,
    beta_bits: AtomicU32,
    velocity_bits: AtomicU32,
    hardness_bits: AtomicU32,
    angle_bits: AtomicU32,
    mute_bits: AtomicU32,
    bleed_bits: AtomicU32,
    stiffness_bits: AtomicU32,
    progress_bits: AtomicU32,
    flags: AtomicU32,
}

impl Default for PlectrumBus {
    fn default() -> Self {
        Self::new()
    }
}

impl PlectrumBus {
    /// Creates a new PlectrumBus initialized to standard PluckNormal defaults.
    pub fn new() -> Self {
        let (beta, vel, hard, mute, bleed) = PluckArticulation::PluckNormal.nominal_parameters();
        Self {
            articulation_raw: AtomicU32::new(PluckArticulation::PluckNormal as u32),
            beta_bits: AtomicU32::new(beta.to_bits()),
            velocity_bits: AtomicU32::new(vel.to_bits()),
            hardness_bits: AtomicU32::new(hard.to_bits()),
            angle_bits: AtomicU32::new(0.0f32.to_bits()),
            mute_bits: AtomicU32::new(mute.to_bits()),
            bleed_bits: AtomicU32::new(bleed.to_bits()),
            stiffness_bits: AtomicU32::new(0.25f32.to_bits()),
            progress_bits: AtomicU32::new(0.0f32.to_bits()),
            flags: AtomicU32::new(0),
        }
    }

    /// Sets the active plucking articulation and loads nominal default parameters.
    pub fn set_articulation(&self, articulation: PluckArticulation) {
        self.articulation_raw.store(articulation as u32, Ordering::Release);
        let (beta, vel, hard, mute, bleed) = articulation.nominal_parameters();
        self.set_gesture_vector(beta, vel, hard, 0.0, mute, bleed);
    }

    /// Gets the active plucking articulation.
    pub fn get_articulation(&self) -> PluckArticulation {
        PluckArticulation::from_u32(self.articulation_raw.load(Ordering::Acquire))
    }

    /// Sets continuous pluck gesture vector $(\beta, v_s, \text{hardness}, \theta_p, \text{mute}, \text{bleed})$.
    pub fn set_gesture_vector(
        &self,
        pluck_position_beta: f32,
        strike_velocity: f32,
        hammer_hardness: f32,
        plectrum_angle_rad: f32,
        palm_mute_damping: f32,
        sympathetic_bleed: f32,
    ) {
        self.beta_bits.store(pluck_position_beta.clamp(0.05, 0.95).to_bits(), Ordering::Release);
        self.velocity_bits.store(strike_velocity.clamp(0.0, 1.0).to_bits(), Ordering::Release);
        self.hardness_bits.store(hammer_hardness.clamp(0.0, 1.0).to_bits(), Ordering::Release);
        self.angle_bits.store(plectrum_angle_rad.clamp(-0.5, 0.5).to_bits(), Ordering::Release);
        self.mute_bits.store(palm_mute_damping.clamp(0.0, 1.0).to_bits(), Ordering::Release);
        self.bleed_bits.store(sympathetic_bleed.clamp(0.0, 0.50).to_bits(), Ordering::Release);
    }

    /// Gets continuous pluck gesture vector $(\beta, v_s, \text{hardness}, \theta_p, \text{mute}, \text{bleed})$.
    pub fn get_gesture_vector(&self) -> (f32, f32, f32, f32, f32, f32) {
        (
            f32::from_bits(self.beta_bits.load(Ordering::Acquire)),
            f32::from_bits(self.velocity_bits.load(Ordering::Acquire)),
            f32::from_bits(self.hardness_bits.load(Ordering::Acquire)),
            f32::from_bits(self.angle_bits.load(Ordering::Acquire)),
            f32::from_bits(self.mute_bits.load(Ordering::Acquire)),
            f32::from_bits(self.bleed_bits.load(Ordering::Acquire)),
        )
    }

    /// Sets string stiffness dispersion parameter $[0.0 ..= 1.0]$.
    pub fn set_string_stiffness(&self, stiffness: f32) {
        self.stiffness_bits.store(stiffness.clamp(0.0, 1.0).to_bits(), Ordering::Release);
    }

    /// Gets string stiffness dispersion parameter.
    pub fn get_string_stiffness(&self) -> f32 {
        f32::from_bits(self.stiffness_bits.load(Ordering::Acquire))
    }

    /// Sets gesture trajectory progress $[0.0 ..= 1.0]$.
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

    /// Captures a complete atomic snapshot of the plectrum bus.
    pub fn snapshot(&self) -> PlectrumBusSnapshot {
        let articulation = self.get_articulation();
        let (beta, vel, hard, angle, mute, bleed) = self.get_gesture_vector();
        let stiffness = self.get_string_stiffness();
        let progress = f32::from_bits(self.progress_bits.load(Ordering::Acquire));
        let flags = self.flags.load(Ordering::Acquire);

        PlectrumBusSnapshot {
            articulation,
            pluck_position_beta: beta,
            strike_velocity: vel,
            hammer_hardness: hard,
            plectrum_angle_rad: angle,
            palm_mute_damping: mute,
            sympathetic_bleed: bleed,
            string_stiffness: stiffness,
            gesture_progress: progress,
            is_active: (flags & 1) != 0,
        }
    }

    /// Applies a complete snapshot into this bus.
    pub fn apply_snapshot(&self, snap: &PlectrumBusSnapshot) {
        self.articulation_raw.store(snap.articulation as u32, Ordering::Release);
        self.beta_bits.store(snap.pluck_position_beta.to_bits(), Ordering::Release);
        self.velocity_bits.store(snap.strike_velocity.to_bits(), Ordering::Release);
        self.hardness_bits.store(snap.hammer_hardness.to_bits(), Ordering::Release);
        self.angle_bits.store(snap.plectrum_angle_rad.to_bits(), Ordering::Release);
        self.mute_bits.store(snap.palm_mute_damping.to_bits(), Ordering::Release);
        self.bleed_bits.store(snap.sympathetic_bleed.to_bits(), Ordering::Release);
        self.stiffness_bits.store(snap.string_stiffness.to_bits(), Ordering::Release);
        self.progress_bits.store(snap.gesture_progress.to_bits(), Ordering::Release);
        self.flags.store(if snap.is_active { 1 } else { 0 }, Ordering::Release);
    }

    /// Dispatches plectrum bus values into `ParamBus` handles.
    pub fn dispatch_to_param_bus(&self, bus: &ParamBus, base_param_id: ParamId) {
        let snap = self.snapshot();
        bus.set(ParamId(base_param_id.0), snap.articulation as u32 as f32);
        bus.set(ParamId(base_param_id.0 + 1), snap.pluck_position_beta);
        bus.set(ParamId(base_param_id.0 + 2), snap.strike_velocity);
        bus.set(ParamId(base_param_id.0 + 3), snap.hammer_hardness);
        bus.set(ParamId(base_param_id.0 + 4), snap.plectrum_angle_rad);
        bus.set(ParamId(base_param_id.0 + 5), snap.palm_mute_damping);
        bus.set(ParamId(base_param_id.0 + 6), snap.sympathetic_bleed);
        bus.set(ParamId(base_param_id.0 + 7), snap.string_stiffness);
        bus.set(ParamId(base_param_id.0 + 8), snap.gesture_progress);
    }

    /// Synchronizes plectrum bus values from `ParamBus` handles.
    pub fn sync_from_param_bus(&self, bus: &ParamBus, base_param_id: ParamId) {
        let art_idx = bus.get(ParamId(base_param_id.0)).unwrap_or(0.0).max(0.0).round() as u32;
        let beta = bus.get(ParamId(base_param_id.0 + 1)).unwrap_or(0.15);
        let vel = bus.get(ParamId(base_param_id.0 + 2)).unwrap_or(0.75);
        let hard = bus.get(ParamId(base_param_id.0 + 3)).unwrap_or(0.50);
        let angle = bus.get(ParamId(base_param_id.0 + 4)).unwrap_or(0.0);
        let mute = bus.get(ParamId(base_param_id.0 + 5)).unwrap_or(0.0);
        let bleed = bus.get(ParamId(base_param_id.0 + 6)).unwrap_or(0.15);
        let stiff = bus.get(ParamId(base_param_id.0 + 7)).unwrap_or(0.25);
        let prog = bus.get(ParamId(base_param_id.0 + 8)).unwrap_or(0.0);

        self.articulation_raw.store(art_idx, Ordering::Release);
        self.set_gesture_vector(beta, vel, hard, angle, mute, bleed);
        self.set_string_stiffness(stiff);
        self.set_progress(prog, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::allocator::AllocGuard;

    #[test]
    fn test_plectrum_bus_snapshot_and_zero_allocation() {
        let bus = PlectrumBus::new();
        bus.set_articulation(PluckArticulation::PalmMute);
        bus.set_gesture_vector(0.10, 0.95, 0.80, 0.15, 0.75, 0.05);
        bus.set_string_stiffness(0.35);
        bus.set_progress(0.45, true);

        {
            let _guard = AllocGuard::new();
            let snap = bus.snapshot();
            assert_eq!(snap.articulation, PluckArticulation::PalmMute);
            assert!((snap.pluck_position_beta - 0.10).abs() < 1e-4);
            assert!((snap.strike_velocity - 0.95).abs() < 1e-4);
            assert!((snap.palm_mute_damping - 0.75).abs() < 1e-4);
            assert!(snap.is_active);
        }
    }

    #[test]
    fn test_plectrum_bus_param_bus_round_trip() {
        let plectrum_bus = PlectrumBus::new();
        let mut param_bus = ParamBus::new();
        for i in 0..12 {
            param_bus.register(ParamId(800 + i), 0.0);
        }

        plectrum_bus.set_articulation(PluckArticulation::SnapPizzicato);
        plectrum_bus.set_gesture_vector(0.08, 1.0, 0.90, -0.10, 0.0, 0.25);
        plectrum_bus.dispatch_to_param_bus(&param_bus, ParamId(800));

        let sync_bus = PlectrumBus::new();
        sync_bus.sync_from_param_bus(&param_bus, ParamId(800));

        let snap = sync_bus.snapshot();
        assert_eq!(snap.articulation, PluckArticulation::SnapPizzicato);
        assert!((snap.strike_velocity - 1.0).abs() < 1e-4);
        assert!((snap.hammer_hardness - 0.90).abs() < 1e-4);
    }
}
