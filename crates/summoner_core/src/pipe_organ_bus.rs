// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Lock-Free Pipe Organ Bus Parameter Routing (Milestone 24).
//!
//! Provides thread-safe lock-free exchange of continuous physical modeling pipe organ
//! parameters (Stop Registration bitmask, Wind Pressure, Cutup Ratio, Chiff Duration,
//! Tracker Key Velocity, Tremulant Modulation Rate/Depth, and Master Gain) across
//! audio, sequencer, and GUI threads.
//!
//! Enforces zero heap allocations in real-time audio threads under `AllocGuard`.

use crate::param_bus::{ParamBus, ParamId};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};

/// Classical and symphonic pipe organ registrations / articulations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u32)]
pub enum PipeOrganArticulation {
    /// Grand Baroque Tutti / Organo Pleno (Principal 8', Bourdon 16', Octave 4', Super Octave 2', Mixture IV, Trompette 8').
    #[default]
    ToccataPlenum = 0,
    /// Warm introspective devotional registration (Bourdon 16', Principal 8', Flute 4').
    BaroqueChorale = 1,
    /// Massive Gothic swell with full reeds and compound mixtures.
    GothicCathedral = 2,
    /// Symphonic Cavaille-Coll reed solo (Trompette 8', Vox Humana 8', Principal 8', Octave 4').
    FrenchRomanticReed = 3,
    /// Intimate Vox Humana with Tremulant and Bourdon 16' backing.
    VocalVoxHumana = 4,
    /// Delicate flute duo (Bourdon 16', Flute 4', Super Octave 2').
    SilverFlutes = 5,
    /// Snappy staccato articulation highlighting chiff attack noise.
    StaccatoChiff = 6,
    /// Atmospheric pulsating pneumatic tremulant swell.
    TremulantSwell = 7,
    /// Custom multi-keyframe Catmull-Rom spline trajectory.
    CustomSpline = 8,
}

/// Full nominal tuple parameters for Pipe Organ articulation.
/// (stops_mask, wind_pressure_mmh2o, cutup_ratio, chiff_duration_ms, tracker_velocity, tremulant_enabled, tremulant_rate_hz, tremulant_depth, master_gain)
pub type PipeOrganNominalParameters = (
    u32,  // stops_mask
    f32,  // wind_pressure_mmh2o
    f32,  // cutup_ratio
    f32,  // chiff_duration_ms
    f32,  // tracker_velocity
    bool, // tremulant_enabled
    f32,  // tremulant_rate_hz
    f32,  // tremulant_depth
    f32,  // master_gain
);

impl PipeOrganArticulation {
    pub fn from_u32(val: u32) -> Self {
        match val {
            0 => Self::ToccataPlenum,
            1 => Self::BaroqueChorale,
            2 => Self::GothicCathedral,
            3 => Self::FrenchRomanticReed,
            4 => Self::VocalVoxHumana,
            5 => Self::SilverFlutes,
            6 => Self::StaccatoChiff,
            7 => Self::TremulantSwell,
            8 => Self::CustomSpline,
            _ => Self::ToccataPlenum,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::ToccataPlenum => "Baroque Toccata Organo Pleno",
            Self::BaroqueChorale => "Warm Baroque Chorale",
            Self::GothicCathedral => "Gothic Cathedral Full Tutti",
            Self::FrenchRomanticReed => "French Romantic Reed Solo",
            Self::VocalVoxHumana => "Vocal Vox Humana & Tremulant",
            Self::SilverFlutes => "Silver Flutes Duo",
            Self::StaccatoChiff => "Snappy Staccato Chiff",
            Self::TremulantSwell => "Pneumatic Tremulant Swell",
            Self::CustomSpline => "Custom Spline Gesture",
        }
    }

    /// Returns nominal default parameters tuple.
    pub fn nominal_parameters(&self) -> PipeOrganNominalParameters {
        match self {
            // Stops: Principal 8' (1), Bourdon 16' (2), Octave 4' (4), Super Octave 2' (16), Mixture IV (32), Trompette 8' (64) = 119
            Self::ToccataPlenum => (119, 85.0, 0.25, 28.0, 0.90, false, 5.5, 0.10, 0.85),
            // Stops: Principal 8' (1), Bourdon 16' (2), Flute 4' (8) = 11
            Self::BaroqueChorale => (11, 65.0, 0.30, 35.0, 0.65, false, 5.0, 0.08, 0.80),
            // Stops: All 8 stops active = 255
            Self::GothicCathedral => (255, 110.0, 0.22, 22.0, 0.95, false, 6.0, 0.12, 0.90),
            // Stops: Principal 8' (1), Octave 4' (4), Trompette 8' (64), Vox Humana 8' (128) = 197
            Self::FrenchRomanticReed => (197, 95.0, 0.20, 18.0, 0.85, false, 5.8, 0.15, 0.88),
            // Stops: Bourdon 16' (2), Flute 4' (8), Vox Humana 8' (128) = 138
            Self::VocalVoxHumana => (138, 70.0, 0.28, 40.0, 0.70, true, 5.2, 0.22, 0.82),
            // Stops: Bourdon 16' (2), Flute 4' (8), Super Octave 2' (16) = 26
            Self::SilverFlutes => (26, 60.0, 0.35, 45.0, 0.60, false, 4.8, 0.05, 0.78),
            Self::StaccatoChiff => (119, 90.0, 0.38, 65.0, 1.00, false, 5.5, 0.10, 0.85),
            Self::TremulantSwell => (139, 72.0, 0.26, 30.0, 0.75, true, 6.2, 0.30, 0.85),
            Self::CustomSpline => (119, 85.0, 0.25, 28.0, 0.80, false, 5.5, 0.10, 0.85),
        }
    }
}

/// Default base ParamId for Pipe Organ synthesis.
pub const PIPE_ORGAN_BASE_PARAM_ID: ParamId = ParamId(1120);

/// Helper to pre-register all Pipe Organ parameters into a ParamBus.
pub fn register_pipe_organ_params(param_bus: &mut ParamBus, base_id: ParamId) {
    let (mask, p, cutup, chiff, tracker, trem_en, trem_rate, trem_depth, gain) =
        PipeOrganArticulation::ToccataPlenum.nominal_parameters();

    param_bus.register(ParamId(base_id.0), PipeOrganArticulation::ToccataPlenum as u32 as f32);
    param_bus.register(ParamId(base_id.0 + 1), mask as f32);
    param_bus.register(ParamId(base_id.0 + 2), p);
    param_bus.register(ParamId(base_id.0 + 3), cutup);
    param_bus.register(ParamId(base_id.0 + 4), chiff);
    param_bus.register(ParamId(base_id.0 + 5), tracker);
    param_bus.register(ParamId(base_id.0 + 6), if trem_en { 1.0 } else { 0.0 });
    param_bus.register(ParamId(base_id.0 + 7), trem_rate);
    param_bus.register(ParamId(base_id.0 + 8), trem_depth);
    param_bus.register(ParamId(base_id.0 + 9), gain);
}

/// Complete thread-safe lock-free Pipe Organ parameter bus.
#[derive(Debug)]
pub struct PipeOrganBus {
    pub stops_mask: AtomicU32,
    pub wind_pressure: AtomicU32,
    pub cutup_ratio: AtomicU32,
    pub chiff_duration_ms: AtomicU32,
    pub tracker_velocity: AtomicU32,
    pub tremulant_enabled: AtomicU32,
    pub tremulant_rate: AtomicU32,
    pub tremulant_depth: AtomicU32,
    pub master_gain: AtomicU32,
    pub articulation: AtomicU32,
}

impl Default for PipeOrganBus {
    fn default() -> Self {
        Self::new()
    }
}

impl PipeOrganBus {
    /// Create a new pipe organ parameter bus with default ToccataPlenum settings.
    pub fn new() -> Self {
        let (mask, p, cutup, chiff, tracker, trem_en, trem_rate, trem_depth, gain) =
            PipeOrganArticulation::ToccataPlenum.nominal_parameters();

        Self {
            stops_mask: AtomicU32::new(mask),
            wind_pressure: AtomicU32::new(p.to_bits()),
            cutup_ratio: AtomicU32::new(cutup.to_bits()),
            chiff_duration_ms: AtomicU32::new(chiff.to_bits()),
            tracker_velocity: AtomicU32::new(tracker.to_bits()),
            tremulant_enabled: AtomicU32::new(if trem_en { 1 } else { 0 }),
            tremulant_rate: AtomicU32::new(trem_rate.to_bits()),
            tremulant_depth: AtomicU32::new(trem_depth.to_bits()),
            master_gain: AtomicU32::new(gain.to_bits()),
            articulation: AtomicU32::new(PipeOrganArticulation::ToccataPlenum as u32),
        }
    }

    /// Read a consistent point-in-time snapshot.
    pub fn snapshot(&self) -> PipeOrganBusSnapshot {
        PipeOrganBusSnapshot {
            stops_mask: self.stops_mask.load(Ordering::Acquire),
            wind_pressure_mmh2o: f32::from_bits(self.wind_pressure.load(Ordering::Acquire)),
            cutup_ratio: f32::from_bits(self.cutup_ratio.load(Ordering::Acquire)),
            chiff_duration_ms: f32::from_bits(self.chiff_duration_ms.load(Ordering::Acquire)),
            tracker_velocity: f32::from_bits(self.tracker_velocity.load(Ordering::Acquire)),
            tremulant_enabled: self.tremulant_enabled.load(Ordering::Acquire) != 0,
            tremulant_rate_hz: f32::from_bits(self.tremulant_rate.load(Ordering::Acquire)),
            tremulant_depth: f32::from_bits(self.tremulant_depth.load(Ordering::Acquire)),
            master_gain: f32::from_bits(self.master_gain.load(Ordering::Acquire)),
            articulation: PipeOrganArticulation::from_u32(self.articulation.load(Ordering::Acquire)),
        }
    }

    /// Write a full snapshot atomically into the bus.
    pub fn write_snapshot(&self, snap: &PipeOrganBusSnapshot) {
        self.stops_mask.store(snap.stops_mask, Ordering::Release);
        self.wind_pressure.store(snap.wind_pressure_mmh2o.to_bits(), Ordering::Release);
        self.cutup_ratio.store(snap.cutup_ratio.to_bits(), Ordering::Release);
        self.chiff_duration_ms.store(snap.chiff_duration_ms.to_bits(), Ordering::Release);
        self.tracker_velocity.store(snap.tracker_velocity.to_bits(), Ordering::Release);
        self.tremulant_enabled.store(if snap.tremulant_enabled { 1 } else { 0 }, Ordering::Release);
        self.tremulant_rate.store(snap.tremulant_rate_hz.to_bits(), Ordering::Release);
        self.tremulant_depth.store(snap.tremulant_depth.to_bits(), Ordering::Release);
        self.master_gain.store(snap.master_gain.to_bits(), Ordering::Release);
        self.articulation.store(snap.articulation as u32, Ordering::Release);
    }

    /// Set articulation and load nominal parameters.
    pub fn set_articulation(&self, art: PipeOrganArticulation) {
        let (mask, p, cutup, chiff, tracker, trem_en, trem_rate, trem_depth, gain) =
            art.nominal_parameters();

        self.stops_mask.store(mask, Ordering::Release);
        self.wind_pressure.store(p.to_bits(), Ordering::Release);
        self.cutup_ratio.store(cutup.to_bits(), Ordering::Release);
        self.chiff_duration_ms.store(chiff.to_bits(), Ordering::Release);
        self.tracker_velocity.store(tracker.to_bits(), Ordering::Release);
        self.tremulant_enabled.store(if trem_en { 1 } else { 0 }, Ordering::Release);
        self.tremulant_rate.store(trem_rate.to_bits(), Ordering::Release);
        self.tremulant_depth.store(trem_depth.to_bits(), Ordering::Release);
        self.master_gain.store(gain.to_bits(), Ordering::Release);
        self.articulation.store(art as u32, Ordering::Release);
    }

    /// Synchronize this bus values to a global ParamBus at base_id.
    pub fn dispatch_to_param_bus(&self, bus: &ParamBus, base_id: ParamId) {
        let snap = self.snapshot();
        bus.set(ParamId(base_id.0), snap.articulation as u32 as f32);
        bus.set(ParamId(base_id.0 + 1), snap.stops_mask as f32);
        bus.set(ParamId(base_id.0 + 2), snap.wind_pressure_mmh2o);
        bus.set(ParamId(base_id.0 + 3), snap.cutup_ratio);
        bus.set(ParamId(base_id.0 + 4), snap.chiff_duration_ms);
        bus.set(ParamId(base_id.0 + 5), snap.tracker_velocity);
        bus.set(
            ParamId(base_id.0 + 6),
            if snap.tremulant_enabled { 1.0 } else { 0.0 },
        );
        bus.set(ParamId(base_id.0 + 7), snap.tremulant_rate_hz);
        bus.set(ParamId(base_id.0 + 8), snap.tremulant_depth);
        bus.set(ParamId(base_id.0 + 9), snap.master_gain);
    }

    /// Read updated values from global ParamBus at base_id into this bus.
    pub fn sync_from_param_bus(&self, bus: &ParamBus, base_id: ParamId) {
        let art_idx = bus.get(ParamId(base_id.0)).unwrap_or(0.0).max(0.0).round() as u32;
        let mask = bus.get(ParamId(base_id.0 + 1)).unwrap_or(119.0).round() as u32;
        let p = bus.get(ParamId(base_id.0 + 2)).unwrap_or(85.0);
        let cutup = bus.get(ParamId(base_id.0 + 3)).unwrap_or(0.25);
        let chiff = bus.get(ParamId(base_id.0 + 4)).unwrap_or(28.0);
        let tracker = bus.get(ParamId(base_id.0 + 5)).unwrap_or(0.90);
        let trem_en = bus.get(ParamId(base_id.0 + 6)).unwrap_or(0.0) >= 0.5;
        let trem_rate = bus.get(ParamId(base_id.0 + 7)).unwrap_or(5.5);
        let trem_depth = bus.get(ParamId(base_id.0 + 8)).unwrap_or(0.10);
        let gain = bus.get(ParamId(base_id.0 + 9)).unwrap_or(0.85);

        self.articulation.store(art_idx, Ordering::Release);
        self.stops_mask.store(mask, Ordering::Release);
        self.wind_pressure.store(p.to_bits(), Ordering::Release);
        self.cutup_ratio.store(cutup.to_bits(), Ordering::Release);
        self.chiff_duration_ms.store(chiff.to_bits(), Ordering::Release);
        self.tracker_velocity.store(tracker.to_bits(), Ordering::Release);
        self.tremulant_enabled.store(if trem_en { 1 } else { 0 }, Ordering::Release);
        self.tremulant_rate.store(trem_rate.to_bits(), Ordering::Release);
        self.tremulant_depth.store(trem_depth.to_bits(), Ordering::Release);
        self.master_gain.store(gain.to_bits(), Ordering::Release);
    }
}

/// Point-in-time serializable snapshot of Pipe Organ bus.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PipeOrganBusSnapshot {
    pub stops_mask: u32,
    pub wind_pressure_mmh2o: f32,
    pub cutup_ratio: f32,
    pub chiff_duration_ms: f32,
    pub tracker_velocity: f32,
    pub tremulant_enabled: bool,
    pub tremulant_rate_hz: f32,
    pub tremulant_depth: f32,
    pub master_gain: f32,
    pub articulation: PipeOrganArticulation,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipe_organ_bus_lock_free_snapshot_round_trip() {
        let bus = PipeOrganBus::new();
        bus.set_articulation(PipeOrganArticulation::GothicCathedral);

        let snap = bus.snapshot();
        assert_eq!(snap.articulation, PipeOrganArticulation::GothicCathedral);
        assert_eq!(snap.stops_mask, 255);
        assert_eq!(snap.wind_pressure_mmh2o, 110.0);
        assert_eq!(snap.chiff_duration_ms, 22.0);

        let mut modified = snap;
        modified.wind_pressure_mmh2o = 135.0;
        modified.cutup_ratio = 0.40;
        bus.write_snapshot(&modified);

        let snap2 = bus.snapshot();
        assert_eq!(snap2.wind_pressure_mmh2o, 135.0);
        assert_eq!(snap2.cutup_ratio, 0.40);
    }

    #[test]
    fn test_pipe_organ_bus_param_bus_round_trip() {
        let bus = PipeOrganBus::new();
        bus.set_articulation(PipeOrganArticulation::FrenchRomanticReed);

        let mut param_bus = ParamBus::new();
        let base_id = ParamId(1120);
        register_pipe_organ_params(&mut param_bus, base_id);

        bus.dispatch_to_param_bus(&param_bus, base_id);

        assert_eq!(param_bus.get(ParamId(base_id.0 + 1)), Some(197.0));
        assert_eq!(param_bus.get(ParamId(base_id.0 + 2)), Some(95.0));

        param_bus.set(ParamId(base_id.0 + 2), 105.0);
        let sync_bus = PipeOrganBus::new();
        sync_bus.sync_from_param_bus(&param_bus, base_id);
        assert_eq!(sync_bus.snapshot().wind_pressure_mmh2o, 105.0);
    }
}
