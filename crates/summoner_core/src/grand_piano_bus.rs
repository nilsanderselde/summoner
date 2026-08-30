// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Lock-Free Concert Grand Piano Bus Parameter Routing (Milestone 26).
//!
//! Provides thread-safe lock-free exchange of continuous physical modeling grand piano
//! parameters (Hammer Strike Velocity, Damper Lift Position / Half-Pedaling, Una Corda Soft Pedal Shift,
//! Sostenuto Latch Bitmasks, Bridge Coupling Bleed, Unison Detune Cents, Hammer Hardness,
//! String Inharmonicity B, Soundboard Decay Scale, and Master Gain) across audio, sequencer, and GUI threads.
//!
//! Enforces zero heap allocations in real-time audio threads under `AllocGuard`.

use crate::param_bus::{ParamBus, ParamId};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};

/// Classical and modern Concert Grand Piano performance styles and gesture articulations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u32)]
pub enum GrandPianoArticulation {
    /// 9ft Steinway D Concert Recital (balanced prompt/aftersound, open rich projection).
    #[default]
    ConcertRecital = 0,
    /// 9.5ft Bösendorfer Imperial 290 (deep sub-bass resonance, dark warm sustain).
    BösendorferWarmth = 1,
    /// 9ft Yamaha CFX (bright, crisp prompt attack, crystalline clarity).
    YamahaPopTreble = 2,
    /// Soft pedal Impressionist style (delicate ethereal shimmer, harmonic softening).
    ImpressionistUnaCorda = 3,
    /// Half-pedal & Sostenuto study (precise damper release and latching).
    HalfPedalSostenutoStudy = 4,
    /// Intimate close-mic felted piano (mellow felt, soft percussive clatter).
    IntimateFelted = 5,
    /// Prepared avant-garde (muted string junctions, metallic boundary reflections).
    PreparedAvantGarde = 6,
    /// Custom multi-keyframe Catmull-Rom spline trajectory.
    CustomSpline = 7,
}

/// Full nominal tuple parameters for Grand Piano articulation:
/// (strike_velocity, damper_lift_pos, una_corda_shift, sostenuto_low, sostenuto_mid, sostenuto_high,
///  bridge_bleed, unison_detune_cents, hammer_hardness, inharmonicity_b, soundboard_decay, master_gain)
pub type GrandPianoNominalParameters = (
    f32, // strike_velocity [0.0 .. 1.0]
    f32, // damper_lift_pos [0.0 .. 1.0] (sustain/half-pedal)
    f32, // una_corda_shift [0.0 .. 1.0] (soft pedal)
    u32, // sostenuto_latch_low (keys 21..52)
    u32, // sostenuto_latch_mid (keys 53..84)
    u32, // sostenuto_latch_high (keys 85..108)
    f32, // bridge_bleed [0.0 .. 1.0]
    f32, // unison_detune_cents [0.0 .. 5.0]
    f32, // hammer_hardness [0.0 .. 1.0]
    f32, // inharmonicity_b [0.00001 .. 0.001]
    f32, // soundboard_decay [0.1 .. 3.0]
    f32, // master_gain [0.0 .. 2.0]
);

impl GrandPianoArticulation {
    pub fn from_u32(val: u32) -> Self {
        match val {
            0 => Self::ConcertRecital,
            1 => Self::BösendorferWarmth,
            2 => Self::YamahaPopTreble,
            3 => Self::ImpressionistUnaCorda,
            4 => Self::HalfPedalSostenutoStudy,
            5 => Self::IntimateFelted,
            6 => Self::PreparedAvantGarde,
            7 => Self::CustomSpline,
            _ => Self::ConcertRecital,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::ConcertRecital => "Steinway D Concert Recital",
            Self::BösendorferWarmth => "Bösendorfer Imperial Warmth",
            Self::YamahaPopTreble => "Yamaha CFX Pop Clarity",
            Self::ImpressionistUnaCorda => "Impressionist Una Corda Shimmer",
            Self::HalfPedalSostenutoStudy => "Half-Pedal & Sostenuto Study",
            Self::IntimateFelted => "Intimate Felted Studio Grand",
            Self::PreparedAvantGarde => "Prepared Avant-Garde Clatter",
            Self::CustomSpline => "Custom Spline Gesture",
        }
    }

    /// Returns nominal default parameters tuple.
    pub fn nominal_parameters(&self) -> GrandPianoNominalParameters {
        match self {
            Self::ConcertRecital => (0.75, 0.0, 0.0, 0, 0, 0, 0.35, 0.75, 0.65, 0.00018, 1.0, 0.85),
            Self::BösendorferWarmth => (0.70, 0.0, 0.0, 0, 0, 0, 0.45, 0.60, 0.50, 0.00012, 1.35, 0.82),
            Self::YamahaPopTreble => (0.85, 0.0, 0.0, 0, 0, 0, 0.28, 0.90, 0.80, 0.00025, 0.85, 0.88),
            Self::ImpressionistUnaCorda => (0.50, 0.80, 1.0, 0, 0, 0, 0.50, 1.10, 0.30, 0.00015, 1.20, 0.80),
            Self::HalfPedalSostenutoStudy => (0.65, 0.50, 0.0, 0xFF, 0, 0, 0.35, 0.75, 0.60, 0.00018, 1.0, 0.85),
            Self::IntimateFelted => (0.55, 0.0, 0.4, 0, 0, 0, 0.20, 0.50, 0.35, 0.00010, 0.75, 0.80),
            Self::PreparedAvantGarde => (0.90, 0.0, 0.0, 0, 0, 0, 0.15, 2.50, 0.90, 0.00060, 0.40, 0.90),
            Self::CustomSpline => (0.75, 0.0, 0.0, 0, 0, 0, 0.35, 0.75, 0.65, 0.00018, 1.0, 0.85),
        }
    }
}

/// Instantaneous lock-free snapshot of Grand Piano bus parameters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GrandPianoBusSnapshot {
    pub articulation: GrandPianoArticulation,
    pub strike_velocity: f32,
    pub damper_lift_pos: f32,
    pub una_corda_shift: f32,
    pub sostenuto_latch_low: u32,
    pub sostenuto_latch_mid: u32,
    pub sostenuto_latch_high: u32,
    pub bridge_bleed: f32,
    pub unison_detune_cents: f32,
    pub hammer_hardness: f32,
    pub inharmonicity_b: f32,
    pub soundboard_decay: f32,
    pub master_gain: f32,
}

/// Default base ParamId for Grand Piano synthesis.
pub const GRAND_PIANO_BASE_PARAM_ID: ParamId = ParamId(1150);

/// Helper to pre-register all Grand Piano parameters into a ParamBus.
pub fn register_grand_piano_params(param_bus: &mut ParamBus, base_id: ParamId) {
    let (vel, damp, una, s_low, s_mid, s_high, bleed, detune, hard, b, decay, gain) =
        GrandPianoArticulation::ConcertRecital.nominal_parameters();

    param_bus.register(ParamId(base_id.0), GrandPianoArticulation::ConcertRecital as u32 as f32);
    param_bus.register(ParamId(base_id.0 + 1), vel);
    param_bus.register(ParamId(base_id.0 + 2), damp);
    param_bus.register(ParamId(base_id.0 + 3), una);
    param_bus.register(ParamId(base_id.0 + 4), s_low as f32);
    param_bus.register(ParamId(base_id.0 + 5), s_mid as f32);
    param_bus.register(ParamId(base_id.0 + 6), s_high as f32);
    param_bus.register(ParamId(base_id.0 + 7), bleed);
    param_bus.register(ParamId(base_id.0 + 8), detune);
    param_bus.register(ParamId(base_id.0 + 9), hard);
    param_bus.register(ParamId(base_id.0 + 10), b);
    param_bus.register(ParamId(base_id.0 + 11), decay);
    param_bus.register(ParamId(base_id.0 + 12), gain);
}

/// Complete thread-safe lock-free Grand Piano parameter bus.
#[derive(Debug)]
pub struct GrandPianoBus {
    pub articulation: AtomicU32,
    pub strike_velocity: AtomicU32,
    pub damper_lift_pos: AtomicU32,
    pub una_corda_shift: AtomicU32,
    pub sostenuto_latch_low: AtomicU32,
    pub sostenuto_latch_mid: AtomicU32,
    pub sostenuto_latch_high: AtomicU32,
    pub bridge_bleed: AtomicU32,
    pub unison_detune_cents: AtomicU32,
    pub hammer_hardness: AtomicU32,
    pub inharmonicity_b: AtomicU32,
    pub soundboard_decay: AtomicU32,
    pub master_gain: AtomicU32,
}

impl Default for GrandPianoBus {
    fn default() -> Self {
        Self::new()
    }
}

impl GrandPianoBus {
    /// Create a new grand piano parameter bus with default ConcertRecital settings.
    pub fn new() -> Self {
        let (vel, damp, una, s_low, s_mid, s_high, bleed, detune, hard, b, decay, gain) =
            GrandPianoArticulation::ConcertRecital.nominal_parameters();

        Self {
            articulation: AtomicU32::new(GrandPianoArticulation::ConcertRecital as u32),
            strike_velocity: AtomicU32::new(vel.to_bits()),
            damper_lift_pos: AtomicU32::new(damp.to_bits()),
            una_corda_shift: AtomicU32::new(una.to_bits()),
            sostenuto_latch_low: AtomicU32::new(s_low),
            sostenuto_latch_mid: AtomicU32::new(s_mid),
            sostenuto_latch_high: AtomicU32::new(s_high),
            bridge_bleed: AtomicU32::new(bleed.to_bits()),
            unison_detune_cents: AtomicU32::new(detune.to_bits()),
            hammer_hardness: AtomicU32::new(hard.to_bits()),
            inharmonicity_b: AtomicU32::new(b.to_bits()),
            soundboard_decay: AtomicU32::new(decay.to_bits()),
            master_gain: AtomicU32::new(gain.to_bits()),
        }
    }

    /// Read an instantaneous atomic snapshot. Safe for real-time audio threads.
    #[inline]
    pub fn snapshot(&self) -> GrandPianoBusSnapshot {
        let art_u32 = self.articulation.load(Ordering::Relaxed);
        let vel_bits = self.strike_velocity.load(Ordering::Relaxed);
        let damp_bits = self.damper_lift_pos.load(Ordering::Relaxed);
        let una_bits = self.una_corda_shift.load(Ordering::Relaxed);
        let s_low = self.sostenuto_latch_low.load(Ordering::Relaxed);
        let s_mid = self.sostenuto_latch_mid.load(Ordering::Relaxed);
        let s_high = self.sostenuto_latch_high.load(Ordering::Relaxed);
        let bleed_bits = self.bridge_bleed.load(Ordering::Relaxed);
        let detune_bits = self.unison_detune_cents.load(Ordering::Relaxed);
        let hard_bits = self.hammer_hardness.load(Ordering::Relaxed);
        let b_bits = self.inharmonicity_b.load(Ordering::Relaxed);
        let decay_bits = self.soundboard_decay.load(Ordering::Relaxed);
        let gain_bits = self.master_gain.load(Ordering::Relaxed);

        GrandPianoBusSnapshot {
            articulation: GrandPianoArticulation::from_u32(art_u32),
            strike_velocity: f32::from_bits(vel_bits),
            damper_lift_pos: f32::from_bits(damp_bits),
            una_corda_shift: f32::from_bits(una_bits),
            sostenuto_latch_low: s_low,
            sostenuto_latch_mid: s_mid,
            sostenuto_latch_high: s_high,
            bridge_bleed: f32::from_bits(bleed_bits),
            unison_detune_cents: f32::from_bits(detune_bits),
            hammer_hardness: f32::from_bits(hard_bits),
            inharmonicity_b: f32::from_bits(b_bits),
            soundboard_decay: f32::from_bits(decay_bits),
            master_gain: f32::from_bits(gain_bits),
        }
    }

    /// Update articulation and set nominal parameters atomically.
    pub fn set_articulation(&self, art: GrandPianoArticulation) {
        let (vel, damp, una, s_low, s_mid, s_high, bleed, detune, hard, b, decay, gain) =
            art.nominal_parameters();

        self.articulation.store(art as u32, Ordering::Relaxed);
        self.strike_velocity.store(vel.to_bits(), Ordering::Relaxed);
        self.damper_lift_pos.store(damp.to_bits(), Ordering::Relaxed);
        self.una_corda_shift.store(una.to_bits(), Ordering::Relaxed);
        self.sostenuto_latch_low.store(s_low, Ordering::Relaxed);
        self.sostenuto_latch_mid.store(s_mid, Ordering::Relaxed);
        self.sostenuto_latch_high.store(s_high, Ordering::Relaxed);
        self.bridge_bleed.store(bleed.to_bits(), Ordering::Relaxed);
        self.unison_detune_cents.store(detune.to_bits(), Ordering::Relaxed);
        self.hammer_hardness.store(hard.to_bits(), Ordering::Relaxed);
        self.inharmonicity_b.store(b.to_bits(), Ordering::Relaxed);
        self.soundboard_decay.store(decay.to_bits(), Ordering::Relaxed);
        self.master_gain.store(gain.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_strike_velocity(&self, vel: f32) {
        self.strike_velocity.store(vel.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_damper_lift(&self, pos: f32) {
        self.damper_lift_pos.store(pos.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_una_corda_shift(&self, shift: f32) {
        self.una_corda_shift.store(shift.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_sostenuto_latch(&self, low: u32, mid: u32, high: u32) {
        self.sostenuto_latch_low.store(low, Ordering::Relaxed);
        self.sostenuto_latch_mid.store(mid, Ordering::Relaxed);
        self.sostenuto_latch_high.store(high, Ordering::Relaxed);
    }

    #[inline]
    pub fn set_bridge_bleed(&self, bleed: f32) {
        self.bridge_bleed.store(bleed.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_unison_detune_cents(&self, cents: f32) {
        self.unison_detune_cents.store(cents.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_hammer_hardness(&self, hardness: f32) {
        self.hammer_hardness.store(hardness.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_inharmonicity_b(&self, b: f32) {
        self.inharmonicity_b.store(b.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_soundboard_decay(&self, decay: f32) {
        self.soundboard_decay.store(decay.to_bits(), Ordering::Relaxed);
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
    fn test_grand_piano_bus_articulation_round_trip() {
        let bus = GrandPianoBus::new();
        let snap = bus.snapshot();
        assert_eq!(snap.articulation, GrandPianoArticulation::ConcertRecital);
        assert!((snap.strike_velocity - 0.75).abs() < 1e-4);

        bus.set_articulation(GrandPianoArticulation::ImpressionistUnaCorda);
        let snap2 = bus.snapshot();
        assert_eq!(snap2.articulation, GrandPianoArticulation::ImpressionistUnaCorda);
        assert!((snap2.una_corda_shift - 1.0).abs() < 1e-4);
    }
}
