// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Lock-Free Mallet Bus & Real-Time Percussion Articulation Parameter Routing (Milestone 19).
//!
//! Provides thread-safe lock-free exchange of continuous percussion physical strike vectors
//! (Radial Strike Position $r \in [0.0, 1.0]$, Strike Velocity $v_s$, Mallet Hardness $h$,
//! Rimshot Damping, Roll Tremolo Rate, Kettle Tension / Pitch Bend, Air Cavity Depth, and
//! Technique Enums) across audio, sequencer, and GUI threads.
//!
//! Enforces zero heap allocations in real-time audio threads under `AllocGuard`.

use crate::param_bus::{ParamBus, ParamId};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};

/// Classical and contemporary percussive strike articulation techniques.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u32)]
pub enum MalletArticulation {
    /// Standard sweet spot strike (rich harmonic modal balance).
    #[default]
    EdgeSweetSpot = 0,
    /// Center strike (strong fundamental axisymmetric modes, hollower tone).
    CenterStrike = 1,
    /// Rimshot impact (simultaneous membrane and rim hoop hit).
    Rimshot = 2,
    /// Rapid continuous roll / tremolo oscillation (2 .. 40 Hz).
    MalletRollTremolo = 3,
    /// Dead stroke (mallet held firmly against membrane to suppress sustain).
    DeadStrokeMuted = 4,
    /// Stick shot (one stick pressed on skin, struck by second stick).
    StickShot = 5,
    /// Wire brush stirring sweep on textured coated head.
    BrushSweep = 6,
    /// Cross-stick / side-stick click on hoop.
    CrossStick = 7,
    /// Timpani kettle pedal pitch bend / talking drum tension squeeze.
    KettlePitchBend = 8,
}

impl MalletArticulation {
    pub fn from_u32(val: u32) -> Self {
        match val {
            0 => Self::EdgeSweetSpot,
            1 => Self::CenterStrike,
            2 => Self::Rimshot,
            3 => Self::MalletRollTremolo,
            4 => Self::DeadStrokeMuted,
            5 => Self::StickShot,
            6 => Self::BrushSweep,
            7 => Self::CrossStick,
            8 => Self::KettlePitchBend,
            _ => Self::EdgeSweetSpot,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::EdgeSweetSpot => "Sweet Spot Strike",
            Self::CenterStrike => "Center Strike",
            Self::Rimshot => "Rimshot",
            Self::MalletRollTremolo => "Mallet Roll / Tremolo",
            Self::DeadStrokeMuted => "Dead Stroke (Muted)",
            Self::StickShot => "Stick Shot",
            Self::BrushSweep => "Brush Sweep",
            Self::CrossStick => "Cross-Stick",
            Self::KettlePitchBend => "Kettle Pitch Bend",
        }
    }

    /// Returns nominal default $(r, \text{velocity}, \text{hardness}, \text{rim\_damping}, \text{roll\_hz}, \text{tension})$ parameters.
    pub fn nominal_parameters(&self) -> (f32, f32, f32, f32, f32, f32) {
        match self {
            Self::EdgeSweetSpot => (0.70, 0.75, 0.50, 0.05, 0.0, 0.0),
            Self::CenterStrike => (0.15, 0.85, 0.40, 0.02, 0.0, 0.0),
            Self::Rimshot => (0.95, 0.95, 0.85, 0.75, 0.0, 0.0),
            Self::MalletRollTremolo => (0.65, 0.60, 0.45, 0.05, 12.0, 0.0),
            Self::DeadStrokeMuted => (0.50, 0.80, 0.60, 0.90, 0.0, 0.0),
            Self::StickShot => (0.60, 0.90, 0.80, 0.40, 0.0, 0.0),
            Self::BrushSweep => (0.45, 0.40, 0.20, 0.10, 0.0, 0.0),
            Self::CrossStick => (0.90, 0.70, 0.75, 0.85, 0.0, 0.0),
            Self::KettlePitchBend => (0.70, 0.75, 0.50, 0.05, 0.0, 300.0),
        }
    }
}

/// Instantaneous lock-free snapshot of mallet bus parameters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MalletBusSnapshot {
    pub articulation: MalletArticulation,
    pub radial_strike_pos: f32,
    pub strike_velocity: f32,
    pub mallet_hardness: f32,
    pub rimshot_damping: f32,
    pub roll_tremolo_rate_hz: f32,
    pub kettle_tension_cents: f32,
    pub air_cavity_depth: f32,
    pub membrane_tension: f32,
    pub gesture_progress: f32,
    pub is_active: bool,
}

impl Default for MalletBusSnapshot {
    fn default() -> Self {
        let (r, vel, hard, rim, roll, tens) = MalletArticulation::EdgeSweetSpot.nominal_parameters();
        Self {
            articulation: MalletArticulation::EdgeSweetSpot,
            radial_strike_pos: r,
            strike_velocity: vel,
            mallet_hardness: hard,
            rimshot_damping: rim,
            roll_tremolo_rate_hz: roll,
            kettle_tension_cents: tens,
            air_cavity_depth: 0.75,
            membrane_tension: 0.50,
            gesture_progress: 0.0,
            is_active: false,
        }
    }
}

/// Lock-free physical percussion mallet parameter routing bus.
pub struct MalletBus {
    articulation_raw: AtomicU32,
    radial_pos_bits: AtomicU32,
    velocity_bits: AtomicU32,
    hardness_bits: AtomicU32,
    rim_damping_bits: AtomicU32,
    roll_rate_bits: AtomicU32,
    tension_cents_bits: AtomicU32,
    cavity_depth_bits: AtomicU32,
    membrane_tension_bits: AtomicU32,
    progress_bits: AtomicU32,
    flags: AtomicU32,
}

impl Default for MalletBus {
    fn default() -> Self {
        Self::new()
    }
}

impl MalletBus {
    /// Creates a new MalletBus initialized to standard EdgeSweetSpot defaults.
    pub fn new() -> Self {
        let (r, vel, hard, rim, roll, tens) = MalletArticulation::EdgeSweetSpot.nominal_parameters();
        Self {
            articulation_raw: AtomicU32::new(MalletArticulation::EdgeSweetSpot as u32),
            radial_pos_bits: AtomicU32::new(r.to_bits()),
            velocity_bits: AtomicU32::new(vel.to_bits()),
            hardness_bits: AtomicU32::new(hard.to_bits()),
            rim_damping_bits: AtomicU32::new(rim.to_bits()),
            roll_rate_bits: AtomicU32::new(roll.to_bits()),
            tension_cents_bits: AtomicU32::new(tens.to_bits()),
            cavity_depth_bits: AtomicU32::new(0.75f32.to_bits()),
            membrane_tension_bits: AtomicU32::new(0.50f32.to_bits()),
            progress_bits: AtomicU32::new(0.0f32.to_bits()),
            flags: AtomicU32::new(0),
        }
    }

    /// Sets active articulation and loads nominal default parameters.
    pub fn set_articulation(&self, articulation: MalletArticulation) {
        self.articulation_raw.store(articulation as u32, Ordering::Release);
        let (r, vel, hard, rim, roll, tens) = articulation.nominal_parameters();
        self.set_gesture_vector(r, vel, hard, rim, roll, tens);
    }

    /// Gets active articulation.
    pub fn get_articulation(&self) -> MalletArticulation {
        MalletArticulation::from_u32(self.articulation_raw.load(Ordering::Acquire))
    }

    /// Sets continuous mallet gesture vector $(r, v_s, h, d_{\text{rim}}, f_{\text{roll}}, \Delta\text{cents})$.
    pub fn set_gesture_vector(
        &self,
        radial_strike_pos: f32,
        strike_velocity: f32,
        mallet_hardness: f32,
        rimshot_damping: f32,
        roll_tremolo_rate_hz: f32,
        kettle_tension_cents: f32,
    ) {
        self.radial_pos_bits.store(radial_strike_pos.clamp(0.0, 1.0).to_bits(), Ordering::Release);
        self.velocity_bits.store(strike_velocity.clamp(0.0, 1.0).to_bits(), Ordering::Release);
        self.hardness_bits.store(mallet_hardness.clamp(0.0, 1.0).to_bits(), Ordering::Release);
        self.rim_damping_bits.store(rimshot_damping.clamp(0.0, 1.0).to_bits(), Ordering::Release);
        self.roll_rate_bits.store(roll_tremolo_rate_hz.clamp(0.0, 60.0).to_bits(), Ordering::Release);
        self.tension_cents_bits.store(kettle_tension_cents.clamp(-1200.0, 1200.0).to_bits(), Ordering::Release);
    }

    /// Gets continuous mallet gesture vector $(r, v_s, h, d_{\text{rim}}, f_{\text{roll}}, \Delta\text{cents})$.
    pub fn get_gesture_vector(&self) -> (f32, f32, f32, f32, f32, f32) {
        (
            f32::from_bits(self.radial_pos_bits.load(Ordering::Acquire)),
            f32::from_bits(self.velocity_bits.load(Ordering::Acquire)),
            f32::from_bits(self.hardness_bits.load(Ordering::Acquire)),
            f32::from_bits(self.rim_damping_bits.load(Ordering::Acquire)),
            f32::from_bits(self.roll_rate_bits.load(Ordering::Acquire)),
            f32::from_bits(self.tension_cents_bits.load(Ordering::Acquire)),
        )
    }

    /// Sets air cavity depth $[0.0 ..= 1.0]$.
    pub fn set_air_cavity_depth(&self, depth: f32) {
        self.cavity_depth_bits.store(depth.clamp(0.0, 1.0).to_bits(), Ordering::Release);
    }

    /// Gets air cavity depth.
    pub fn get_air_cavity_depth(&self) -> f32 {
        f32::from_bits(self.cavity_depth_bits.load(Ordering::Acquire))
    }

    /// Sets base membrane tension $[0.0 ..= 1.0]$.
    pub fn set_membrane_tension(&self, tension: f32) {
        self.membrane_tension_bits.store(tension.clamp(0.0, 1.0).to_bits(), Ordering::Release);
    }

    /// Gets base membrane tension.
    pub fn get_membrane_tension(&self) -> f32 {
        f32::from_bits(self.membrane_tension_bits.load(Ordering::Acquire))
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

    /// Captures a complete atomic snapshot of the mallet bus.
    pub fn snapshot(&self) -> MalletBusSnapshot {
        let articulation = self.get_articulation();
        let (r, vel, hard, rim, roll, tens) = self.get_gesture_vector();
        let cavity = self.get_air_cavity_depth();
        let tension = self.get_membrane_tension();
        let progress = f32::from_bits(self.progress_bits.load(Ordering::Acquire));
        let flags = self.flags.load(Ordering::Acquire);

        MalletBusSnapshot {
            articulation,
            radial_strike_pos: r,
            strike_velocity: vel,
            mallet_hardness: hard,
            rimshot_damping: rim,
            roll_tremolo_rate_hz: roll,
            kettle_tension_cents: tens,
            air_cavity_depth: cavity,
            membrane_tension: tension,
            gesture_progress: progress,
            is_active: (flags & 1) != 0,
        }
    }

    /// Applies a complete snapshot into this bus.
    pub fn apply_snapshot(&self, snap: &MalletBusSnapshot) {
        self.articulation_raw.store(snap.articulation as u32, Ordering::Release);
        self.radial_pos_bits.store(snap.radial_strike_pos.to_bits(), Ordering::Release);
        self.velocity_bits.store(snap.strike_velocity.to_bits(), Ordering::Release);
        self.hardness_bits.store(snap.mallet_hardness.to_bits(), Ordering::Release);
        self.rim_damping_bits.store(snap.rimshot_damping.to_bits(), Ordering::Release);
        self.roll_rate_bits.store(snap.roll_tremolo_rate_hz.to_bits(), Ordering::Release);
        self.tension_cents_bits.store(snap.kettle_tension_cents.to_bits(), Ordering::Release);
        self.cavity_depth_bits.store(snap.air_cavity_depth.to_bits(), Ordering::Release);
        self.membrane_tension_bits.store(snap.membrane_tension.to_bits(), Ordering::Release);
        self.progress_bits.store(snap.gesture_progress.to_bits(), Ordering::Release);
        self.flags.store(if snap.is_active { 1 } else { 0 }, Ordering::Release);
    }

    /// Dispatches current mallet bus snapshot values to `ParamBus`.
    pub fn dispatch_to_param_bus(&self, bus: &ParamBus, base_id: ParamId) {
        let snap = self.snapshot();
        bus.set(ParamId(base_id.0), snap.articulation as u32 as f32);
        bus.set(ParamId(base_id.0 + 1), snap.radial_strike_pos);
        bus.set(ParamId(base_id.0 + 2), snap.strike_velocity);
        bus.set(ParamId(base_id.0 + 3), snap.mallet_hardness);
        bus.set(ParamId(base_id.0 + 4), snap.rimshot_damping);
        bus.set(ParamId(base_id.0 + 5), snap.roll_tremolo_rate_hz);
        bus.set(ParamId(base_id.0 + 6), snap.kettle_tension_cents);
        bus.set(ParamId(base_id.0 + 7), snap.air_cavity_depth);
        bus.set(ParamId(base_id.0 + 8), snap.membrane_tension);
        bus.set(ParamId(base_id.0 + 9), snap.gesture_progress);
    }

    /// Pulls updated parameter values from a central `ParamBus`.
    pub fn sync_from_param_bus(&self, bus: &ParamBus, base_id: ParamId) {
        let art_idx = bus.get(ParamId(base_id.0)).unwrap_or(0.0).max(0.0).round() as u32;
        let r = bus.get(ParamId(base_id.0 + 1)).unwrap_or(0.70);
        let vel = bus.get(ParamId(base_id.0 + 2)).unwrap_or(0.75);
        let hard = bus.get(ParamId(base_id.0 + 3)).unwrap_or(0.50);
        let rim = bus.get(ParamId(base_id.0 + 4)).unwrap_or(0.05);
        let roll = bus.get(ParamId(base_id.0 + 5)).unwrap_or(0.0);
        let tens = bus.get(ParamId(base_id.0 + 6)).unwrap_or(0.0);
        let cav = bus.get(ParamId(base_id.0 + 7)).unwrap_or(0.75);
        let memb = bus.get(ParamId(base_id.0 + 8)).unwrap_or(0.50);
        let prog = bus.get(ParamId(base_id.0 + 9)).unwrap_or(0.0);

        self.articulation_raw.store(art_idx, Ordering::Release);
        self.set_gesture_vector(r, vel, hard, rim, roll, tens);
        self.set_air_cavity_depth(cav);
        self.set_membrane_tension(memb);
        self.set_progress(prog, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::allocator::AllocGuard;

    #[test]
    fn test_mallet_bus_snapshot_and_zero_allocation() {
        let bus = MalletBus::new();
        bus.set_articulation(MalletArticulation::Rimshot);
        bus.set_gesture_vector(0.95, 0.90, 0.85, 0.80, 0.0, 0.0);
        bus.set_air_cavity_depth(0.40);
        bus.set_membrane_tension(0.70);
        bus.set_progress(0.60, true);

        {
            let _guard = AllocGuard::new();
            let snap = bus.snapshot();
            assert_eq!(snap.articulation, MalletArticulation::Rimshot);
            assert!((snap.radial_strike_pos - 0.95).abs() < 1e-4);
            assert!((snap.strike_velocity - 0.90).abs() < 1e-4);
            assert!((snap.rimshot_damping - 0.80).abs() < 1e-4);
            assert!(snap.is_active);
        }
    }

    #[test]
    fn test_mallet_bus_param_bus_round_trip() {
        let mallet_bus = MalletBus::new();
        let mut param_bus = ParamBus::new();
        for i in 0..12 {
            param_bus.register(ParamId(900 + i), 0.0);
        }

        mallet_bus.set_articulation(MalletArticulation::CenterStrike);
        mallet_bus.set_gesture_vector(0.15, 0.85, 0.40, 0.02, 0.0, 0.0);
        mallet_bus.dispatch_to_param_bus(&param_bus, ParamId(900));

        let sync_bus = MalletBus::new();
        sync_bus.sync_from_param_bus(&param_bus, ParamId(900));

        let snap = sync_bus.snapshot();
        assert_eq!(snap.articulation, MalletArticulation::CenterStrike);
        assert!((snap.radial_strike_pos - 0.15).abs() < 1e-4);
        assert!((snap.strike_velocity - 0.85).abs() < 1e-4);
    }
}
