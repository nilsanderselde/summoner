// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Lock-Free Rotary & Tonewheel Bus Parameter Routing (Milestone 21).
//!
//! Provides thread-safe lock-free exchange of continuous electromechanical tonewheel organ
//! and Leslie rotary speaker parameters (Speed Mode, Horn/Drum RPM, 9-Drawbar Levels,
//! Harmonic Percussion, 6550 Tube Drive Saturation, Scanner Vibrato/Chorus, Key Click,
//! and Articulation Technique Enums) across audio, sequencer, and GUI threads.
//!
//! Enforces zero heap allocations in real-time audio threads under `AllocGuard`.

use crate::param_bus::{ParamBus, ParamId};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};

/// Classical and contemporary rotary speaker and tonewheel organ articulation techniques.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u32)]
pub enum RotaryArticulation {
    /// Slow chorale spatial modulation (~40 RPM horn).
    #[default]
    ChoraleSlow = 0,
    /// Fast tremolo intense Doppler shimmer (~400 RPM horn).
    TremoloFast = 1,
    /// Mechanical brake friction spin-down to stationary stop.
    BrakeStop = 2,
    /// Continuous 9-drawbar harmonic morphing gesture.
    DrawbarMorph = 3,
    /// Non-legato 2nd/3rd harmonic percussion accent.
    PercussionHit = 4,
    /// High-gain 6550 power amp tube overdrive saturation boost.
    TubeDriveBoost = 5,
    /// Gospel crescendo swell with drawbar pull and speed toggle.
    GospelSwell = 6,
    /// Rock roaring solo registration (88 8888 888) with C3 chorus.
    RockOverdrive = 7,
    /// Full stop stationary cabinet acoustic reflection.
    FullStop = 8,
}

impl RotaryArticulation {
    pub fn from_u32(val: u32) -> Self {
        match val {
            0 => Self::ChoraleSlow,
            1 => Self::TremoloFast,
            2 => Self::BrakeStop,
            3 => Self::DrawbarMorph,
            4 => Self::PercussionHit,
            5 => Self::TubeDriveBoost,
            6 => Self::GospelSwell,
            7 => Self::RockOverdrive,
            8 => Self::FullStop,
            _ => Self::ChoraleSlow,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::ChoraleSlow => "Chorale Slow Rotation",
            Self::TremoloFast => "Tremolo Fast Rotation",
            Self::BrakeStop => "Mechanical Brake Stop",
            Self::DrawbarMorph => "9-Drawbar Harmonic Morph",
            Self::PercussionHit => "Harmonic Percussion Hit",
            Self::TubeDriveBoost => "Tube Preamp Overdrive Boost",
            Self::GospelSwell => "Gospel Organ Swell",
            Self::RockOverdrive => "Rock Full Drawbar Roar",
            Self::FullStop => "Stationary Cabinet Position",
        }
    }

    /// Returns nominal default (speed_state, drive_db, horn_drum_balance, [drawbar_0..8]).
    pub fn nominal_parameters(&self) -> (u32, f32, f32, [f32; 9]) {
        match self {
            Self::ChoraleSlow => (1, 4.0, 60.0, [8.0, 8.0, 8.0, 0.0, 0.0, 0.0, 0.0, 0.0, 8.0]),
            Self::TremoloFast => (2, 6.5, 60.0, [8.0, 8.0, 8.0, 0.0, 0.0, 0.0, 0.0, 0.0, 8.0]),
            Self::BrakeStop => (3, 3.0, 50.0, [8.0, 8.0, 8.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            Self::DrawbarMorph => (1, 5.0, 60.0, [8.0, 6.0, 8.0, 4.0, 2.0, 0.0, 0.0, 4.0, 8.0]),
            Self::PercussionHit => (2, 5.5, 65.0, [8.0, 8.0, 8.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            Self::TubeDriveBoost => (2, 16.0, 60.0, [8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0]),
            Self::GospelSwell => (2, 8.0, 65.0, [8.0, 8.0, 8.0, 4.0, 4.0, 4.0, 4.0, 6.0, 8.0]),
            Self::RockOverdrive => (2, 14.0, 60.0, [8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0]),
            Self::FullStop => (0, 2.0, 50.0, [8.0, 0.0, 8.0, 0.0, 0.0, 8.0, 0.0, 0.0, 0.0]),
        }
    }
}

/// Instantaneous lock-free snapshot of rotary speaker and tonewheel organ parameters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RotaryBusSnapshot {
    pub articulation: RotaryArticulation,
    /// Speed state (0=Stop, 1=Chorale, 2=Tremolo, 3=Brake).
    pub speed_state: u32,
    pub horn_rpm: f32,
    pub drum_rpm: f32,
    /// 9 Drawbar levels $[0.0 ..= 8.0]$.
    pub drawbars: [f32; 9],
    pub percussion_enabled: bool,
    /// Percussion harmonic (2 for 2nd / 4', 3 for 3rd / 2-2/3').
    pub percussion_harmonic: u32,
    pub percussion_fast: bool,
    pub percussion_soft: bool,
    pub tube_drive_db: f32,
    /// Vibrato/Chorus mode index (0=Off, 1=V1, 2=C1, 3=V2, 4=C2, 5=V3, 6=C3).
    pub vibrato_mode: u32,
    pub key_click_amount: f32,
    pub crosstalk_amount: f32,
    pub horn_drum_balance_pct: f32,
    pub mic_distance_m: f32,
    pub mic_spread_deg: f32,
    pub gesture_progress: f32,
    pub is_active: bool,
}

impl Default for RotaryBusSnapshot {
    fn default() -> Self {
        let (speed, drive, bal, dbs) = RotaryArticulation::ChoraleSlow.nominal_parameters();
        Self {
            articulation: RotaryArticulation::ChoraleSlow,
            speed_state: speed,
            horn_rpm: 40.0,
            drum_rpm: 36.0,
            drawbars: dbs,
            percussion_enabled: true,
            percussion_harmonic: 3,
            percussion_fast: true,
            percussion_soft: false,
            tube_drive_db: drive,
            vibrato_mode: 6, // C3
            key_click_amount: 0.35,
            crosstalk_amount: 0.08,
            horn_drum_balance_pct: bal,
            mic_distance_m: 0.65,
            mic_spread_deg: 120.0,
            gesture_progress: 0.0,
            is_active: false,
        }
    }
}

/// Lock-free electromechanical rotary speaker and tonewheel parameter routing bus.
pub struct RotaryBus {
    articulation_raw: AtomicU32,
    speed_state_raw: AtomicU32,
    horn_rpm_bits: AtomicU32,
    drum_rpm_bits: AtomicU32,
    drawbar_bits: [AtomicU32; 9],
    percussion_flags: AtomicU32, // bit 0: enabled, bit 1: harmonic(0=2nd, 1=3rd), bit 2: fast, bit 3: soft
    tube_drive_bits: AtomicU32,
    vibrato_mode_raw: AtomicU32,
    key_click_bits: AtomicU32,
    crosstalk_bits: AtomicU32,
    balance_bits: AtomicU32,
    mic_dist_bits: AtomicU32,
    mic_spread_bits: AtomicU32,
    progress_bits: AtomicU32,
    flags: AtomicU32,
}

impl Default for RotaryBus {
    fn default() -> Self {
        Self::new()
    }
}

impl RotaryBus {
    /// Creates a new RotaryBus initialized to ChoraleSlow defaults.
    pub fn new() -> Self {
        let (speed, drive, bal, dbs) = RotaryArticulation::ChoraleSlow.nominal_parameters();

        Self {
            articulation_raw: AtomicU32::new(RotaryArticulation::ChoraleSlow as u32),
            speed_state_raw: AtomicU32::new(speed),
            horn_rpm_bits: AtomicU32::new(40.0f32.to_bits()),
            drum_rpm_bits: AtomicU32::new(36.0f32.to_bits()),
            drawbar_bits: [
                AtomicU32::new(dbs[0].to_bits()),
                AtomicU32::new(dbs[1].to_bits()),
                AtomicU32::new(dbs[2].to_bits()),
                AtomicU32::new(dbs[3].to_bits()),
                AtomicU32::new(dbs[4].to_bits()),
                AtomicU32::new(dbs[5].to_bits()),
                AtomicU32::new(dbs[6].to_bits()),
                AtomicU32::new(dbs[7].to_bits()),
                AtomicU32::new(dbs[8].to_bits()),
            ],
            percussion_flags: AtomicU32::new(1 | (1 << 1) | (1 << 2)), // Enabled, 3rd, Fast
            tube_drive_bits: AtomicU32::new(drive.to_bits()),
            vibrato_mode_raw: AtomicU32::new(6), // C3
            key_click_bits: AtomicU32::new(0.35f32.to_bits()),
            crosstalk_bits: AtomicU32::new(0.08f32.to_bits()),
            balance_bits: AtomicU32::new(bal.to_bits()),
            mic_dist_bits: AtomicU32::new(0.65f32.to_bits()),
            mic_spread_bits: AtomicU32::new(120.0f32.to_bits()),
            progress_bits: AtomicU32::new(0.0f32.to_bits()),
            flags: AtomicU32::new(0),
        }
    }

    /// Sets active articulation and loads nominal parameters.
    pub fn set_articulation(&self, articulation: RotaryArticulation) {
        self.articulation_raw.store(articulation as u32, Ordering::Release);
        let (speed, drive, bal, dbs) = articulation.nominal_parameters();
        self.speed_state_raw.store(speed, Ordering::Release);
        self.tube_drive_bits.store(drive.to_bits(), Ordering::Release);
        self.balance_bits.store(bal.to_bits(), Ordering::Release);
        for (i, db_val) in dbs.iter().enumerate() {
            self.drawbar_bits[i].store(db_val.to_bits(), Ordering::Release);
        }
    }

    /// Gets active articulation.
    pub fn get_articulation(&self) -> RotaryArticulation {
        RotaryArticulation::from_u32(self.articulation_raw.load(Ordering::Acquire))
    }

    /// Sets the 9 drawbars levels $[0.0 ..= 8.0]$.
    pub fn set_drawbars(&self, drawbars: &[f32; 9]) {
        for (i, db_val) in drawbars.iter().enumerate() {
            self.drawbar_bits[i].store(db_val.clamp(0.0, 8.0).to_bits(), Ordering::Release);
        }
    }

    /// Gets the 9 drawbar levels.
    pub fn get_drawbars(&self) -> [f32; 9] {
        let mut dbs = [0.0f32; 9];
        for (i, db_slot) in dbs.iter_mut().enumerate() {
            *db_slot = f32::from_bits(self.drawbar_bits[i].load(Ordering::Acquire));
        }
        dbs
    }

    /// Sets speed mode (0=Stop, 1=Chorale, 2=Tremolo, 3=Brake) and drive dB.
    pub fn set_speed_and_drive(&self, speed: u32, drive_db: f32) {
        self.speed_state_raw.store(speed.clamp(0, 3), Ordering::Release);
        self.tube_drive_bits.store(drive_db.clamp(0.0, 24.0).to_bits(), Ordering::Release);
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

    /// Captures a complete atomic snapshot of the rotary bus.
    pub fn snapshot(&self) -> RotaryBusSnapshot {
        let articulation = self.get_articulation();
        let speed = self.speed_state_raw.load(Ordering::Acquire);
        let horn_rpm = f32::from_bits(self.horn_rpm_bits.load(Ordering::Acquire));
        let drum_rpm = f32::from_bits(self.drum_rpm_bits.load(Ordering::Acquire));
        let drawbars = self.get_drawbars();
        let p_flags = self.percussion_flags.load(Ordering::Acquire);
        let drive = f32::from_bits(self.tube_drive_bits.load(Ordering::Acquire));
        let vib_mode = self.vibrato_mode_raw.load(Ordering::Acquire);
        let key_click = f32::from_bits(self.key_click_bits.load(Ordering::Acquire));
        let crosstalk = f32::from_bits(self.crosstalk_bits.load(Ordering::Acquire));
        let balance = f32::from_bits(self.balance_bits.load(Ordering::Acquire));
        let mic_dist = f32::from_bits(self.mic_dist_bits.load(Ordering::Acquire));
        let mic_spread = f32::from_bits(self.mic_spread_bits.load(Ordering::Acquire));
        let progress = f32::from_bits(self.progress_bits.load(Ordering::Acquire));
        let flags = self.flags.load(Ordering::Acquire);

        RotaryBusSnapshot {
            articulation,
            speed_state: speed,
            horn_rpm,
            drum_rpm,
            drawbars,
            percussion_enabled: (p_flags & 1) != 0,
            percussion_harmonic: if (p_flags & 2) != 0 { 3 } else { 2 },
            percussion_fast: (p_flags & 4) != 0,
            percussion_soft: (p_flags & 8) != 0,
            tube_drive_db: drive,
            vibrato_mode: vib_mode,
            key_click_amount: key_click,
            crosstalk_amount: crosstalk,
            horn_drum_balance_pct: balance,
            mic_distance_m: mic_dist,
            mic_spread_deg: mic_spread,
            gesture_progress: progress,
            is_active: (flags & 1) != 0,
        }
    }

    /// Applies a complete snapshot into this bus.
    pub fn apply_snapshot(&self, snap: &RotaryBusSnapshot) {
        self.articulation_raw.store(snap.articulation as u32, Ordering::Release);
        self.speed_state_raw.store(snap.speed_state, Ordering::Release);
        self.horn_rpm_bits.store(snap.horn_rpm.to_bits(), Ordering::Release);
        self.drum_rpm_bits.store(snap.drum_rpm.to_bits(), Ordering::Release);
        self.set_drawbars(&snap.drawbars);

        let mut p_flags = 0u32;
        if snap.percussion_enabled { p_flags |= 1; }
        if snap.percussion_harmonic == 3 { p_flags |= 2; }
        if snap.percussion_fast { p_flags |= 4; }
        if snap.percussion_soft { p_flags |= 8; }
        self.percussion_flags.store(p_flags, Ordering::Release);

        self.tube_drive_bits.store(snap.tube_drive_db.to_bits(), Ordering::Release);
        self.vibrato_mode_raw.store(snap.vibrato_mode, Ordering::Release);
        self.key_click_bits.store(snap.key_click_amount.to_bits(), Ordering::Release);
        self.crosstalk_bits.store(snap.crosstalk_amount.to_bits(), Ordering::Release);
        self.balance_bits.store(snap.horn_drum_balance_pct.to_bits(), Ordering::Release);
        self.mic_dist_bits.store(snap.mic_distance_m.to_bits(), Ordering::Release);
        self.mic_spread_bits.store(snap.mic_spread_deg.to_bits(), Ordering::Release);
        self.progress_bits.store(snap.gesture_progress.to_bits(), Ordering::Release);
        self.flags.store(if snap.is_active { 1 } else { 0 }, Ordering::Release);
    }

    /// Dispatches current rotary bus snapshot values to central `ParamBus`.
    pub fn dispatch_to_param_bus(&self, bus: &ParamBus, base_id: ParamId) {
        let snap = self.snapshot();
        bus.set(ParamId(base_id.0), snap.articulation as u32 as f32);
        bus.set(ParamId(base_id.0 + 1), snap.speed_state as f32);
        bus.set(ParamId(base_id.0 + 2), snap.tube_drive_db);
        bus.set(ParamId(base_id.0 + 3), snap.horn_drum_balance_pct);
        for i in 0..9 {
            bus.set(ParamId(base_id.0 + 4 + i as u32), snap.drawbars[i]);
        }
        bus.set(ParamId(base_id.0 + 13), if snap.percussion_enabled { 1.0 } else { 0.0 });
        bus.set(ParamId(base_id.0 + 14), snap.percussion_harmonic as f32);
        bus.set(ParamId(base_id.0 + 15), snap.vibrato_mode as f32);
        bus.set(ParamId(base_id.0 + 16), snap.gesture_progress);
    }

    /// Pulls updated parameter values from a central `ParamBus`.
    pub fn sync_from_param_bus(&self, bus: &ParamBus, base_id: ParamId) {
        let art_idx = bus.get(ParamId(base_id.0)).unwrap_or(0.0).max(0.0).round() as u32;
        let speed = bus.get(ParamId(base_id.0 + 1)).unwrap_or(1.0).round() as u32;
        let drive = bus.get(ParamId(base_id.0 + 2)).unwrap_or(6.0);
        let bal = bus.get(ParamId(base_id.0 + 3)).unwrap_or(60.0);
        let mut dbs = [8.0f32; 9];
        for (i, db_slot) in dbs.iter_mut().enumerate() {
            *db_slot = bus.get(ParamId(base_id.0 + 4 + i as u32)).unwrap_or(8.0);
        }
        let perc_en = bus.get(ParamId(base_id.0 + 13)).unwrap_or(1.0) > 0.5;
        let perc_harm = bus.get(ParamId(base_id.0 + 14)).unwrap_or(3.0).round() as u32;
        let vib = bus.get(ParamId(base_id.0 + 15)).unwrap_or(6.0).round() as u32;
        let prog = bus.get(ParamId(base_id.0 + 16)).unwrap_or(0.0);

        self.articulation_raw.store(art_idx, Ordering::Release);
        self.speed_state_raw.store(speed, Ordering::Release);
        self.tube_drive_bits.store(drive.to_bits(), Ordering::Release);
        self.balance_bits.store(bal.to_bits(), Ordering::Release);
        self.set_drawbars(&dbs);

        let mut p_flags = 0u32;
        if perc_en { p_flags |= 1; }
        if perc_harm == 3 { p_flags |= 2; }
        p_flags |= 4; // fast
        self.percussion_flags.store(p_flags, Ordering::Release);
        self.vibrato_mode_raw.store(vib, Ordering::Release);
        self.set_progress(prog, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::allocator::AllocGuard;

    #[test]
    fn test_rotary_bus_snapshot_and_zero_allocation() {
        let bus = RotaryBus::new();
        bus.set_articulation(RotaryArticulation::GospelSwell);
        bus.set_speed_and_drive(2, 12.5);
        bus.set_progress(0.60, true);

        {
            let _guard = AllocGuard::new();
            let snap = bus.snapshot();
            assert_eq!(snap.articulation, RotaryArticulation::GospelSwell);
            assert_eq!(snap.speed_state, 2);
            assert!((snap.tube_drive_db - 12.5).abs() < 1e-4);
            assert!(snap.is_active);
        }
    }

    #[test]
    fn test_rotary_bus_param_bus_round_trip() {
        let rotary_bus = RotaryBus::new();
        let mut param_bus = ParamBus::new();
        for i in 0..20 {
            param_bus.register(ParamId(900 + i), 0.0);
        }

        rotary_bus.set_articulation(RotaryArticulation::TubeDriveBoost);
        rotary_bus.set_speed_and_drive(2, 18.0);
        rotary_bus.dispatch_to_param_bus(&param_bus, ParamId(900));

        let sync_bus = RotaryBus::new();
        sync_bus.sync_from_param_bus(&param_bus, ParamId(900));

        let snap = sync_bus.snapshot();
        assert_eq!(snap.articulation, RotaryArticulation::TubeDriveBoost);
        assert_eq!(snap.speed_state, 2);
        assert!((snap.tube_drive_db - 18.0).abs() < 1e-4);
    }
}
