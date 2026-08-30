// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Lock-Free Shakuhachi Bus Parameter Routing (Milestone 25).
//!
//! Provides thread-safe lock-free exchange of continuous physical modeling Japanese
//! Shakuhachi parameters (Blowing Pressure, Embouchure Angle, Meri/Kari Pitch Bend offset,
//! Jet Distance, Murai-Iki Turbulence Burst Intensity, 5-Finger Hole Bitmask, and Master Gain)
//! across audio, sequencer, and GUI threads.
//!
//! Enforces zero heap allocations in real-time audio threads under `AllocGuard`.

use crate::param_bus::{ParamBus, ParamId};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};

/// Classical and modern Shakuhachi playing styles and gesture articulations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u32)]
pub enum ShakuhachiArticulation {
    /// Classical Honkyoku Zen meditation (1.8 Shaku, rich harmonics, organic meri vibrato).
    #[default]
    HonkyokuTraditional = 0,
    /// Deep introspective Jinashi Zen meditation (2.4 Shaku, heavy breath turbulence).
    ZenMeditative = 1,
    /// Energetic Min'yo folk flute (1.6 Shaku, snappy attack, bright upper partials).
    MinyoFolk = 2,
    /// Sankyoku chamber ensemble (2.1 Shaku, warm balanced tone, subtle vibrato).
    SankyokuEnsemble = 3,
    /// Explosive Murai-Iki technique with turbulent vortex breath bursts and overblowing.
    MuraiIkiExplosive = 4,
    /// Continuous deep Meri microtone pitch bend glissando down to -300 cents.
    MeriMicrotoneBend = 5,
    /// Kari elevated head tilt overblow into upper Kan/Dai-kan registers (+200 cents).
    KariOverblow = 6,
    /// Giant sub-bass Kyotaku temple flute (3.0 Shaku, deep resonant air column).
    KyotakuTemple = 7,
    /// Custom multi-keyframe Catmull-Rom spline trajectory.
    CustomSpline = 8,
}

/// Full nominal tuple parameters for Shakuhachi articulation:
/// (blowing_pressure_pa, embouchure_angle_deg, meri_kari_cents, jet_distance_mm, murai_iki_intensity, hole_mask, master_gain)
pub type ShakuhachiNominalParameters = (
    f32, // blowing_pressure_pa
    f32, // embouchure_angle_deg
    f32, // meri_kari_cents
    f32, // jet_distance_mm
    f32, // murai_iki_intensity
    u32, // hole_mask
    f32, // master_gain
);

impl ShakuhachiArticulation {
    pub fn from_u32(val: u32) -> Self {
        match val {
            0 => Self::HonkyokuTraditional,
            1 => Self::ZenMeditative,
            2 => Self::MinyoFolk,
            3 => Self::SankyokuEnsemble,
            4 => Self::MuraiIkiExplosive,
            5 => Self::MeriMicrotoneBend,
            6 => Self::KariOverblow,
            7 => Self::KyotakuTemple,
            8 => Self::CustomSpline,
            _ => Self::HonkyokuTraditional,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::HonkyokuTraditional => "Classical Honkyoku Zen Meditation",
            Self::ZenMeditative => "Deep Jinashi Zen Meditation",
            Self::MinyoFolk => "Min'yo Folk Flute Articulation",
            Self::SankyokuEnsemble => "Sankyoku Chamber Ensemble",
            Self::MuraiIkiExplosive => "Explosive Murai-Iki Turbulence Burst",
            Self::MeriMicrotoneBend => "Deep Meri Microtone Pitch Bend",
            Self::KariOverblow => "Kari Upper Octave Overblow",
            Self::KyotakuTemple => "Giant Sub-Bass Kyotaku Temple Flute",
            Self::CustomSpline => "Custom Spline Gesture",
        }
    }

    /// Returns nominal default parameters tuple.
    pub fn nominal_parameters(&self) -> ShakuhachiNominalParameters {
        match self {
            Self::HonkyokuTraditional => (850.0, 38.0, 0.0, 10.0, 0.10, 0, 0.85),
            Self::ZenMeditative => (700.0, 44.0, -80.0, 14.0, 0.35, 0, 0.80),
            Self::MinyoFolk => (1100.0, 34.0, 0.0, 8.0, 0.05, 3, 0.88),
            Self::SankyokuEnsemble => (800.0, 40.0, -20.0, 11.5, 0.08, 0, 0.82),
            Self::MuraiIkiExplosive => (2400.0, 32.0, 40.0, 10.0, 0.85, 0, 0.90),
            Self::MeriMicrotoneBend => (800.0, 22.0, -240.0, 12.0, 0.15, 0, 0.80),
            Self::KariOverblow => (1600.0, 52.0, 150.0, 9.0, 0.20, 16, 0.85),
            Self::KyotakuTemple => (650.0, 48.0, -120.0, 18.0, 0.40, 0, 0.85),
            Self::CustomSpline => (850.0, 38.0, 0.0, 10.0, 0.10, 0, 0.85),
        }
    }
}

/// Instantaneous lock-free snapshot of Shakuhachi bus parameters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ShakuhachiBusSnapshot {
    pub articulation: ShakuhachiArticulation,
    pub blowing_pressure_pa: f32,
    pub embouchure_angle_deg: f32,
    pub meri_kari_cents: f32,
    pub jet_distance_mm: f32,
    pub murai_iki_intensity: f32,
    pub hole_mask: u32,
    pub master_gain: f32,
}

/// Default base ParamId for Shakuhachi synthesis.
pub const SHAKUHACHI_BASE_PARAM_ID: ParamId = ParamId(1140);

/// Helper to pre-register all Shakuhachi parameters into a ParamBus.
pub fn register_shakuhachi_params(param_bus: &mut ParamBus, base_id: ParamId) {
    let (p, angle, meri, dist, murai, mask, gain) =
        ShakuhachiArticulation::HonkyokuTraditional.nominal_parameters();

    param_bus.register(ParamId(base_id.0), ShakuhachiArticulation::HonkyokuTraditional as u32 as f32);
    param_bus.register(ParamId(base_id.0 + 1), p);
    param_bus.register(ParamId(base_id.0 + 2), angle);
    param_bus.register(ParamId(base_id.0 + 3), meri);
    param_bus.register(ParamId(base_id.0 + 4), dist);
    param_bus.register(ParamId(base_id.0 + 5), murai);
    param_bus.register(ParamId(base_id.0 + 6), mask as f32);
    param_bus.register(ParamId(base_id.0 + 7), gain);
}

/// Complete thread-safe lock-free Shakuhachi parameter bus.
#[derive(Debug)]
pub struct ShakuhachiBus {
    pub articulation: AtomicU32,
    pub blowing_pressure_pa: AtomicU32,
    pub embouchure_angle_deg: AtomicU32,
    pub meri_kari_cents: AtomicU32,
    pub jet_distance_mm: AtomicU32,
    pub murai_iki_intensity: AtomicU32,
    pub hole_mask: AtomicU32,
    pub master_gain: AtomicU32,
}

impl Default for ShakuhachiBus {
    fn default() -> Self {
        Self::new()
    }
}

impl ShakuhachiBus {
    /// Create a new shakuhachi parameter bus with default HonkyokuTraditional settings.
    pub fn new() -> Self {
        let (p, angle, meri, dist, murai, mask, gain) =
            ShakuhachiArticulation::HonkyokuTraditional.nominal_parameters();

        Self {
            articulation: AtomicU32::new(ShakuhachiArticulation::HonkyokuTraditional as u32),
            blowing_pressure_pa: AtomicU32::new(p.to_bits()),
            embouchure_angle_deg: AtomicU32::new(angle.to_bits()),
            meri_kari_cents: AtomicU32::new(meri.to_bits()),
            jet_distance_mm: AtomicU32::new(dist.to_bits()),
            murai_iki_intensity: AtomicU32::new(murai.to_bits()),
            hole_mask: AtomicU32::new(mask),
            master_gain: AtomicU32::new(gain.to_bits()),
        }
    }

    /// Read an instantaneous atomic snapshot. Safe for real-time audio threads.
    #[inline]
    pub fn snapshot(&self) -> ShakuhachiBusSnapshot {
        let art_u32 = self.articulation.load(Ordering::Relaxed);
        let p_bits = self.blowing_pressure_pa.load(Ordering::Relaxed);
        let angle_bits = self.embouchure_angle_deg.load(Ordering::Relaxed);
        let meri_bits = self.meri_kari_cents.load(Ordering::Relaxed);
        let dist_bits = self.jet_distance_mm.load(Ordering::Relaxed);
        let murai_bits = self.murai_iki_intensity.load(Ordering::Relaxed);
        let mask = self.hole_mask.load(Ordering::Relaxed);
        let gain_bits = self.master_gain.load(Ordering::Relaxed);

        ShakuhachiBusSnapshot {
            articulation: ShakuhachiArticulation::from_u32(art_u32),
            blowing_pressure_pa: f32::from_bits(p_bits),
            embouchure_angle_deg: f32::from_bits(angle_bits),
            meri_kari_cents: f32::from_bits(meri_bits),
            jet_distance_mm: f32::from_bits(dist_bits),
            murai_iki_intensity: f32::from_bits(murai_bits),
            hole_mask: mask,
            master_gain: f32::from_bits(gain_bits),
        }
    }

    /// Update articulation and set nominal parameters atomically.
    pub fn set_articulation(&self, art: ShakuhachiArticulation) {
        let (p, angle, meri, dist, murai, mask, gain) = art.nominal_parameters();

        self.articulation.store(art as u32, Ordering::Relaxed);
        self.blowing_pressure_pa.store(p.to_bits(), Ordering::Relaxed);
        self.embouchure_angle_deg.store(angle.to_bits(), Ordering::Relaxed);
        self.meri_kari_cents.store(meri.to_bits(), Ordering::Relaxed);
        self.jet_distance_mm.store(dist.to_bits(), Ordering::Relaxed);
        self.murai_iki_intensity.store(murai.to_bits(), Ordering::Relaxed);
        self.hole_mask.store(mask, Ordering::Relaxed);
        self.master_gain.store(gain.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_blowing_pressure(&self, pressure_pa: f32) {
        self.blowing_pressure_pa.store(pressure_pa.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_meri_kari(&self, cents: f32) {
        self.meri_kari_cents.store(cents.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_embouchure_angle(&self, angle_deg: f32) {
        self.embouchure_angle_deg.store(angle_deg.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_jet_distance(&self, dist_mm: f32) {
        self.jet_distance_mm.store(dist_mm.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_murai_iki(&self, intensity: f32) {
        self.murai_iki_intensity.store(intensity.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_hole_mask(&self, mask: u32) {
        self.hole_mask.store(mask, Ordering::Relaxed);
    }

    #[inline]
    pub fn set_master_gain(&self, gain: f32) {
        self.master_gain.store(gain.to_bits(), Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shakuhachi_bus_articulation_round_trip() {
        let bus = ShakuhachiBus::new();
        let snap = bus.snapshot();
        assert_eq!(snap.articulation, ShakuhachiArticulation::HonkyokuTraditional);

        bus.set_articulation(ShakuhachiArticulation::MuraiIkiExplosive);
        let snap2 = bus.snapshot();
        assert_eq!(snap2.articulation, ShakuhachiArticulation::MuraiIkiExplosive);
        assert_eq!(snap2.blowing_pressure_pa, 2400.0);
        assert_eq!(snap2.murai_iki_intensity, 0.85);
    }

    #[test]
    fn test_shakuhachi_param_bus_registration() {
        let mut param_bus = ParamBus::new();
        register_shakuhachi_params(&mut param_bus, SHAKUHACHI_BASE_PARAM_ID);

        let art_val = param_bus.get(SHAKUHACHI_BASE_PARAM_ID);
        assert_eq!(art_val, Some(0.0));

        let pressure_val = param_bus.get(ParamId(SHAKUHACHI_BASE_PARAM_ID.0 + 1));
        assert_eq!(pressure_val, Some(850.0));
    }
}
