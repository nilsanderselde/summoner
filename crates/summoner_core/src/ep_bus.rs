// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Lock-Free Electric Piano Bus Parameter Routing (Milestone 22).
//!
//! Provides thread-safe lock-free exchange of continuous physical modeling electromechanical
//! electric piano parameters (Model Mode: Rhodes Tine / Wurlitzer Reed, Hammer Hardness,
//! Pickup Air-Gap Distance, Inductive Barking Overdrive, Resonant Tonebar Coupling,
//! Damper Release Clunk Volume, Stereo Optical Tremolo Rate/Depth/Stereo, Tube Preamp Drive,
//! Bass/Treble EQ, and Articulation Enums) across audio, sequencer, and GUI threads.
//!
//! Enforces zero heap allocations in real-time audio threads under `AllocGuard`.

use crate::param_bus::{ParamBus, ParamId};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};

/// Classical and modern electromechanical electric piano articulation techniques.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u32)]
pub enum EpArticulation {
    /// Classic Rhodes Suitcase with stereo optical ping-pong tremolo and bell chime.
    #[default]
    ClassicSuitcase = 0,
    /// High-energy Dyno-My-Piano with bright high-end, close pickup air-gap, and heavy barking bite.
    BarkingDyno = 1,
    /// Warm Mellow Rhodes Stage 73 with deep tonebar resonance and gentle tube warmth.
    MellowStage = 2,
    /// Classic Wurlitzer 200A Reed with gritty midrange growl and subtle optical tremolo.
    ClassicWurli = 3,
    /// High-gain Soul & Funk Wurlitzer with overdriven tube preamp and biting reed crunch.
    SoulOverdrive = 4,
    /// Ambient Spatial Rhodes with lush stereo panning and extended tonebar sustain.
    BelledAmbient = 5,
    /// Gentle soft hammer ballad with rich bell transient.
    BalladChime = 6,
    /// Percussive fast attack funk rhythm with biting reed bark and tight damper.
    FunkGroove = 7,
    /// Custom multi-keyframe Catmull-Rom spline trajectory.
    CustomSpline = 8,
}

/// Full nominal tuple parameters for electric piano articulation.
pub type EpNominalParameters = (
    EpModelType,
    f32,
    f32,
    f32,
    f32,
    f32,
    bool,
    f32,
    f32,
    bool,
    f32,
    f32,
    f32,
);

impl EpArticulation {
    pub fn from_u32(val: u32) -> Self {
        match val {
            0 => Self::ClassicSuitcase,
            1 => Self::BarkingDyno,
            2 => Self::MellowStage,
            3 => Self::ClassicWurli,
            4 => Self::SoulOverdrive,
            5 => Self::BelledAmbient,
            6 => Self::BalladChime,
            7 => Self::FunkGroove,
            8 => Self::CustomSpline,
            _ => Self::ClassicSuitcase,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::ClassicSuitcase => "Classic Rhodes Suitcase (Ping-Pong)",
            Self::BarkingDyno => "Dyno-My-Rhodes Barking Lead",
            Self::MellowStage => "Mellow Rhodes Stage 73",
            Self::ClassicWurli => "Classic Wurlitzer 200A Reed",
            Self::SoulOverdrive => "Soul Overdrive Wurlitzer",
            Self::BelledAmbient => "Belled Ambient Spatial Swell",
            Self::BalladChime => "Gentle Ballad Bell Chime",
            Self::FunkGroove => "Percussive Funk Reed Groove",
            Self::CustomSpline => "Custom Spline Gesture",
        }
    }

    /// Returns nominal default (model_type, hammer_hardness, air_gap_mm, bark_drive, tonebar_coupling, damper_clunk, trem_en, trem_rate, trem_depth, trem_stereo, drive_db, bass_db, treble_db).
    pub fn nominal_parameters(&self) -> EpNominalParameters {
        match self {
            Self::ClassicSuitcase => (EpModelType::RhodesTine, 0.50, 1.8, 0.55, 0.70, 0.35, true, 5.2, 0.65, true, 4.0, 1.5, 2.0),
            Self::BarkingDyno => (EpModelType::RhodesTine, 0.85, 1.0, 0.90, 0.85, 0.40, false, 4.0, 0.0, false, 8.0, -1.0, 6.5),
            Self::MellowStage => (EpModelType::RhodesTine, 0.30, 2.4, 0.30, 0.60, 0.25, false, 3.5, 0.0, false, 2.0, 3.0, -1.5),
            Self::ClassicWurli => (EpModelType::WurlitzerReed, 0.60, 1.2, 0.75, 0.35, 0.45, true, 6.0, 0.50, false, 6.0, 0.0, 3.0),
            Self::SoulOverdrive => (EpModelType::WurlitzerReed, 0.80, 0.9, 0.95, 0.40, 0.50, true, 7.2, 0.40, false, 14.0, 2.0, 5.0),
            Self::BelledAmbient => (EpModelType::RhodesTine, 0.40, 2.0, 0.40, 0.95, 0.20, true, 2.8, 0.80, true, 2.0, 2.5, 3.5),
            Self::BalladChime => (EpModelType::RhodesTine, 0.35, 2.2, 0.35, 0.80, 0.20, true, 3.8, 0.45, true, 3.0, 2.0, 1.0),
            Self::FunkGroove => (EpModelType::WurlitzerReed, 0.75, 1.1, 0.85, 0.30, 0.55, false, 5.0, 0.0, false, 9.0, 1.0, 4.0),
            Self::CustomSpline => (EpModelType::RhodesTine, 0.50, 1.8, 0.55, 0.70, 0.35, true, 5.2, 0.65, true, 4.0, 1.5, 2.0),
        }
    }
}

/// Electromechanical physical resonator model type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u32)]
pub enum EpModelType {
    #[default]
    RhodesTine = 0,
    WurlitzerReed = 1,
}

impl EpModelType {
    pub fn from_u32(val: u32) -> Self {
        if val == 1 {
            Self::WurlitzerReed
        } else {
            Self::RhodesTine
        }
    }
}

/// Instantaneous lock-free snapshot of electric piano parameters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EpBusSnapshot {
    pub articulation: EpArticulation,
    pub model_type: EpModelType,
    pub hammer_hardness: f32,
    pub air_gap_mm: f32,
    pub bark_drive: f32,
    pub tonebar_coupling: f32,
    pub damper_clunk_volume: f32,
    pub tremolo_enabled: bool,
    pub tremolo_rate_hz: f32,
    pub tremolo_depth: f32,
    pub tremolo_stereo: bool,
    pub tube_drive_db: f32,
    pub bass_db: f32,
    pub treble_db: f32,
    pub gesture_progress: f32,
    pub is_active: bool,
}

impl Default for EpBusSnapshot {
    fn default() -> Self {
        let (model, hardness, air_gap, bark, tonebar, damper, trem_en, trem_rate, trem_depth, trem_stereo, drive, bass, treble) =
            EpArticulation::ClassicSuitcase.nominal_parameters();
        Self {
            articulation: EpArticulation::ClassicSuitcase,
            model_type: model,
            hammer_hardness: hardness,
            air_gap_mm: air_gap,
            bark_drive: bark,
            tonebar_coupling: tonebar,
            damper_clunk_volume: damper,
            tremolo_enabled: trem_en,
            tremolo_rate_hz: trem_rate,
            tremolo_depth: trem_depth,
            tremolo_stereo: trem_stereo,
            tube_drive_db: drive,
            bass_db: bass,
            treble_db: treble,
            gesture_progress: 0.0,
            is_active: false,
        }
    }
}

/// Lock-free electromechanical electric piano parameter routing bus.
pub struct EpBus {
    articulation_raw: AtomicU32,
    model_type_raw: AtomicU32,
    hammer_hardness_bits: AtomicU32,
    air_gap_bits: AtomicU32,
    bark_drive_bits: AtomicU32,
    tonebar_coupling_bits: AtomicU32,
    damper_clunk_bits: AtomicU32,
    tremolo_flags: AtomicU32, // bit 0: enabled, bit 1: stereo
    tremolo_rate_bits: AtomicU32,
    tremolo_depth_bits: AtomicU32,
    tube_drive_bits: AtomicU32,
    bass_bits: AtomicU32,
    treble_bits: AtomicU32,
    progress_bits: AtomicU32,
    flags: AtomicU32,
}

impl Default for EpBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EpBus {
    /// Creates a new EpBus initialized to ClassicSuitcase defaults.
    pub fn new() -> Self {
        let (model, hardness, air_gap, bark, tonebar, damper, trem_en, trem_rate, trem_depth, trem_stereo, drive, bass, treble) =
            EpArticulation::ClassicSuitcase.nominal_parameters();

        let mut trem_flags = 0u32;
        if trem_en { trem_flags |= 1; }
        if trem_stereo { trem_flags |= 2; }

        Self {
            articulation_raw: AtomicU32::new(EpArticulation::ClassicSuitcase as u32),
            model_type_raw: AtomicU32::new(model as u32),
            hammer_hardness_bits: AtomicU32::new(hardness.to_bits()),
            air_gap_bits: AtomicU32::new(air_gap.to_bits()),
            bark_drive_bits: AtomicU32::new(bark.to_bits()),
            tonebar_coupling_bits: AtomicU32::new(tonebar.to_bits()),
            damper_clunk_bits: AtomicU32::new(damper.to_bits()),
            tremolo_flags: AtomicU32::new(trem_flags),
            tremolo_rate_bits: AtomicU32::new(trem_rate.to_bits()),
            tremolo_depth_bits: AtomicU32::new(trem_depth.to_bits()),
            tube_drive_bits: AtomicU32::new(drive.to_bits()),
            bass_bits: AtomicU32::new(bass.to_bits()),
            treble_bits: AtomicU32::new(treble.to_bits()),
            progress_bits: AtomicU32::new(0.0f32.to_bits()),
            flags: AtomicU32::new(0),
        }
    }

    /// Sets active articulation and loads nominal parameters.
    pub fn set_articulation(&self, articulation: EpArticulation) {
        self.articulation_raw.store(articulation as u32, Ordering::Release);
        let (model, hardness, air_gap, bark, tonebar, damper, trem_en, trem_rate, trem_depth, trem_stereo, drive, bass, treble) =
            articulation.nominal_parameters();

        self.model_type_raw.store(model as u32, Ordering::Release);
        self.hammer_hardness_bits.store(hardness.to_bits(), Ordering::Release);
        self.air_gap_bits.store(air_gap.to_bits(), Ordering::Release);
        self.bark_drive_bits.store(bark.to_bits(), Ordering::Release);
        self.tonebar_coupling_bits.store(tonebar.to_bits(), Ordering::Release);
        self.damper_clunk_bits.store(damper.to_bits(), Ordering::Release);

        let mut trem_flags = 0u32;
        if trem_en { trem_flags |= 1; }
        if trem_stereo { trem_flags |= 2; }
        self.tremolo_flags.store(trem_flags, Ordering::Release);

        self.tremolo_rate_bits.store(trem_rate.to_bits(), Ordering::Release);
        self.tremolo_depth_bits.store(trem_depth.to_bits(), Ordering::Release);
        self.tube_drive_bits.store(drive.to_bits(), Ordering::Release);
        self.bass_bits.store(bass.to_bits(), Ordering::Release);
        self.treble_bits.store(treble.to_bits(), Ordering::Release);
    }

    /// Gets active articulation.
    pub fn get_articulation(&self) -> EpArticulation {
        EpArticulation::from_u32(self.articulation_raw.load(Ordering::Acquire))
    }

    /// Sets mechanical properties.
    pub fn set_mechanics(&self, hardness: f32, air_gap: f32, bark: f32, tonebar: f32, damper: f32) {
        self.hammer_hardness_bits.store(hardness.clamp(0.0, 1.0).to_bits(), Ordering::Release);
        self.air_gap_bits.store(air_gap.clamp(0.4, 6.0).to_bits(), Ordering::Release);
        self.bark_drive_bits.store(bark.clamp(0.0, 1.0).to_bits(), Ordering::Release);
        self.tonebar_coupling_bits.store(tonebar.clamp(0.0, 1.0).to_bits(), Ordering::Release);
        self.damper_clunk_bits.store(damper.clamp(0.0, 1.0).to_bits(), Ordering::Release);
    }

    /// Sets tremolo properties.
    pub fn set_tremolo(&self, enabled: bool, rate_hz: f32, depth: f32, stereo: bool) {
        let mut flags = 0u32;
        if enabled { flags |= 1; }
        if stereo { flags |= 2; }
        self.tremolo_flags.store(flags, Ordering::Release);
        self.tremolo_rate_bits.store(rate_hz.clamp(0.2, 20.0).to_bits(), Ordering::Release);
        self.tremolo_depth_bits.store(depth.clamp(0.0, 1.0).to_bits(), Ordering::Release);
    }

    /// Sets tone properties.
    pub fn set_tone(&self, drive_db: f32, bass_db: f32, treble_db: f32) {
        self.tube_drive_bits.store(drive_db.clamp(0.0, 24.0).to_bits(), Ordering::Release);
        self.bass_bits.store(bass_db.clamp(-12.0, 12.0).to_bits(), Ordering::Release);
        self.treble_bits.store(treble_db.clamp(-12.0, 12.0).to_bits(), Ordering::Release);
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

    /// Captures a complete atomic snapshot of the electric piano bus.
    pub fn snapshot(&self) -> EpBusSnapshot {
        let articulation = self.get_articulation();
        let model = EpModelType::from_u32(self.model_type_raw.load(Ordering::Acquire));
        let hardness = f32::from_bits(self.hammer_hardness_bits.load(Ordering::Acquire));
        let air_gap = f32::from_bits(self.air_gap_bits.load(Ordering::Acquire));
        let bark = f32::from_bits(self.bark_drive_bits.load(Ordering::Acquire));
        let tonebar = f32::from_bits(self.tonebar_coupling_bits.load(Ordering::Acquire));
        let damper = f32::from_bits(self.damper_clunk_bits.load(Ordering::Acquire));
        let trem_flags = self.tremolo_flags.load(Ordering::Acquire);
        let trem_rate = f32::from_bits(self.tremolo_rate_bits.load(Ordering::Acquire));
        let trem_depth = f32::from_bits(self.tremolo_depth_bits.load(Ordering::Acquire));
        let drive = f32::from_bits(self.tube_drive_bits.load(Ordering::Acquire));
        let bass = f32::from_bits(self.bass_bits.load(Ordering::Acquire));
        let treble = f32::from_bits(self.treble_bits.load(Ordering::Acquire));
        let progress = f32::from_bits(self.progress_bits.load(Ordering::Acquire));
        let flags = self.flags.load(Ordering::Acquire);

        EpBusSnapshot {
            articulation,
            model_type: model,
            hammer_hardness: hardness,
            air_gap_mm: air_gap,
            bark_drive: bark,
            tonebar_coupling: tonebar,
            damper_clunk_volume: damper,
            tremolo_enabled: (trem_flags & 1) != 0,
            tremolo_rate_hz: trem_rate,
            tremolo_depth: trem_depth,
            tremolo_stereo: (trem_flags & 2) != 0,
            tube_drive_db: drive,
            bass_db: bass,
            treble_db: treble,
            gesture_progress: progress,
            is_active: (flags & 1) != 0,
        }
    }

    /// Applies a complete snapshot into this bus.
    pub fn apply_snapshot(&self, snap: &EpBusSnapshot) {
        self.articulation_raw.store(snap.articulation as u32, Ordering::Release);
        self.model_type_raw.store(snap.model_type as u32, Ordering::Release);
        self.hammer_hardness_bits.store(snap.hammer_hardness.to_bits(), Ordering::Release);
        self.air_gap_bits.store(snap.air_gap_mm.to_bits(), Ordering::Release);
        self.bark_drive_bits.store(snap.bark_drive.to_bits(), Ordering::Release);
        self.tonebar_coupling_bits.store(snap.tonebar_coupling.to_bits(), Ordering::Release);
        self.damper_clunk_bits.store(snap.damper_clunk_volume.to_bits(), Ordering::Release);

        let mut trem_flags = 0u32;
        if snap.tremolo_enabled { trem_flags |= 1; }
        if snap.tremolo_stereo { trem_flags |= 2; }
        self.tremolo_flags.store(trem_flags, Ordering::Release);

        self.tremolo_rate_bits.store(snap.tremolo_rate_hz.to_bits(), Ordering::Release);
        self.tremolo_depth_bits.store(snap.tremolo_depth.to_bits(), Ordering::Release);
        self.tube_drive_bits.store(snap.tube_drive_db.to_bits(), Ordering::Release);
        self.bass_bits.store(snap.bass_db.to_bits(), Ordering::Release);
        self.treble_bits.store(snap.treble_db.to_bits(), Ordering::Release);
        self.progress_bits.store(snap.gesture_progress.to_bits(), Ordering::Release);
        self.flags.store(if snap.is_active { 1 } else { 0 }, Ordering::Release);
    }

    /// Dispatches current electric piano bus snapshot values to central `ParamBus`.
    pub fn dispatch_to_param_bus(&self, bus: &ParamBus, base_id: ParamId) {
        let snap = self.snapshot();
        bus.set(ParamId(base_id.0), snap.articulation as u32 as f32);
        bus.set(ParamId(base_id.0 + 1), snap.model_type as u32 as f32);
        bus.set(ParamId(base_id.0 + 2), snap.hammer_hardness);
        bus.set(ParamId(base_id.0 + 3), snap.air_gap_mm);
        bus.set(ParamId(base_id.0 + 4), snap.bark_drive);
        bus.set(ParamId(base_id.0 + 5), snap.tonebar_coupling);
        bus.set(ParamId(base_id.0 + 6), snap.damper_clunk_volume);
        bus.set(ParamId(base_id.0 + 7), if snap.tremolo_enabled { 1.0 } else { 0.0 });
        bus.set(ParamId(base_id.0 + 8), snap.tremolo_rate_hz);
        bus.set(ParamId(base_id.0 + 9), snap.tremolo_depth);
        bus.set(ParamId(base_id.0 + 10), if snap.tremolo_stereo { 1.0 } else { 0.0 });
        bus.set(ParamId(base_id.0 + 11), snap.tube_drive_db);
        bus.set(ParamId(base_id.0 + 12), snap.bass_db);
        bus.set(ParamId(base_id.0 + 13), snap.treble_db);
        bus.set(ParamId(base_id.0 + 14), snap.gesture_progress);
    }

    /// Pulls updated parameter values from a central `ParamBus`.
    pub fn sync_from_param_bus(&self, bus: &ParamBus, base_id: ParamId) {
        let art_idx = bus.get(ParamId(base_id.0)).unwrap_or(0.0).max(0.0).round() as u32;
        let model = bus.get(ParamId(base_id.0 + 1)).unwrap_or(0.0).round() as u32;
        let hardness = bus.get(ParamId(base_id.0 + 2)).unwrap_or(0.5);
        let air_gap = bus.get(ParamId(base_id.0 + 3)).unwrap_or(1.8);
        let bark = bus.get(ParamId(base_id.0 + 4)).unwrap_or(0.55);
        let tonebar = bus.get(ParamId(base_id.0 + 5)).unwrap_or(0.70);
        let damper = bus.get(ParamId(base_id.0 + 6)).unwrap_or(0.35);
        let trem_en = bus.get(ParamId(base_id.0 + 7)).unwrap_or(1.0) > 0.5;
        let trem_rate = bus.get(ParamId(base_id.0 + 8)).unwrap_or(5.2);
        let trem_depth = bus.get(ParamId(base_id.0 + 9)).unwrap_or(0.65);
        let trem_stereo = bus.get(ParamId(base_id.0 + 10)).unwrap_or(1.0) > 0.5;
        let drive = bus.get(ParamId(base_id.0 + 11)).unwrap_or(4.0);
        let bass = bus.get(ParamId(base_id.0 + 12)).unwrap_or(1.5);
        let treble = bus.get(ParamId(base_id.0 + 13)).unwrap_or(2.0);
        let prog = bus.get(ParamId(base_id.0 + 14)).unwrap_or(0.0);

        self.articulation_raw.store(art_idx, Ordering::Release);
        self.model_type_raw.store(model, Ordering::Release);
        self.set_mechanics(hardness, air_gap, bark, tonebar, damper);
        self.set_tremolo(trem_en, trem_rate, trem_depth, trem_stereo);
        self.set_tone(drive, bass, treble);
        self.set_progress(prog, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::allocator::AllocGuard;

    #[test]
    fn test_ep_bus_snapshot_and_zero_allocation() {
        let bus = EpBus::new();
        bus.set_articulation(EpArticulation::BarkingDyno);
        bus.set_tone(10.5, 2.0, 5.0);
        bus.set_progress(0.75, true);

        {
            let _guard = AllocGuard::new();
            let snap = bus.snapshot();
            assert_eq!(snap.articulation, EpArticulation::BarkingDyno);
            assert_eq!(snap.model_type, EpModelType::RhodesTine);
            assert!((snap.tube_drive_db - 10.5).abs() < 1e-4);
            assert!(snap.is_active);
        }
    }

    #[test]
    fn test_ep_bus_param_bus_round_trip() {
        let ep_bus = EpBus::new();
        let mut param_bus = ParamBus::new();
        for i in 0..20 {
            param_bus.register(ParamId(950 + i), 0.0);
        }

        ep_bus.set_articulation(EpArticulation::SoulOverdrive);
        ep_bus.set_tone(16.0, 3.0, 4.5);
        ep_bus.dispatch_to_param_bus(&param_bus, ParamId(950));

        let sync_bus = EpBus::new();
        sync_bus.sync_from_param_bus(&param_bus, ParamId(950));

        let snap = sync_bus.snapshot();
        assert_eq!(snap.articulation, EpArticulation::SoulOverdrive);
        assert_eq!(snap.model_type, EpModelType::WurlitzerReed);
        assert!((snap.tube_drive_db - 16.0).abs() < 1e-4);
    }
}
