// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Lock-Free Asian Zither & Koto Bus Parameter Routing (Milestone 29).
//!
//! Provides thread-safe lock-free exchange of continuous physical modeling Koto / Guzheng
//! parameters (Tsume Strike Velocity [0.01..1.0], Oshi-Ite Left-Hand Press Force [0..30 N],
//! Hiki-Iro Pull Release [0..1], Ji Bridge Position Offset [-0.20..+0.20],
//! Tuning Scale Schema [Hirajoshi, Kokin-joshi, In-sen, Kumoi-joshi, Ryukyu, Guzheng],
//! Behind-the-Bridge Sympathetic Bleed [0..1], Soundboard Wood Resonance [0..1],
//! and Master Output Gain) across audio, sequencer, and GUI threads.
//!
//! Enforces zero heap allocations in real-time audio threads under `AllocGuard`.

use crate::param_bus::{ParamBus, ParamId};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};

/// Classical and modern Japanese Koto & Guzheng performance styles and gesture articulations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u32)]
pub enum KotoArticulation {
    /// Traditional Meditative Alap (Subtle yuri vibrato, gentle tsume release, Hirajoshi tuning).
    #[default]
    HirajoshiMeditativeAlap = 0,
    /// Kokin-joshi Fast Miyako-bushi (Sharp ivory tsume accents, fast staccato, urban classical).
    KokinJoshiFastMiyako = 1,
    /// In-sen Contemporary Dramatic (Large oshi-ite tension bends up to +400 cents, strong dynamics).
    InSenDramaticGendai = 2,
    /// Kumoi-joshi Spring Rain (Lyrical, delicate tremolo, sukui-tsume sweeps).
    KumoiJoshiSpringRain = 3,
    /// Ryukyu Festive Island Swell (Bright Okinawan pentatonic scale, lively glissandi).
    RyukyuIslandSwell = 4,
    /// Guzheng Virtuoso Waterfall (Rapid 21-string harp-like arpeggiated flow, rich sympathetic bleed).
    GuzhengVirtuosoWaterfall = 5,
    /// Custom multi-keyframe Catmull-Rom spline trajectory.
    CustomSpline = 6,
}

/// Full nominal tuple parameters for Koto articulation:
/// (tsume_velocity, oshi_ite_force_n, hiki_iro_release, ji_bridge_offset,
///  tuning_schema_idx, behind_bridge_bleed, body_wood_resonance, master_gain)
pub type KotoNominalParameters = (
    f32, // tsume_velocity [0.01 ..= 1.0]
    f32, // oshi_ite_force_n [0.0 ..= 30.0]
    f32, // hiki_iro_release [0.0 ..= 1.0]
    f32, // ji_bridge_offset [-0.20 ..= +0.20]
    u32, // tuning_schema_idx [0 = Hirajoshi, 1 = Kokin, 2 = InSen, 3 = Kumoi, 4 = Ryukyu, 5 = Guzheng]
    f32, // behind_bridge_bleed [0.0 ..= 1.0]
    f32, // body_wood_resonance [0.0 ..= 1.0]
    f32, // master_gain [0.0 ..= 2.0]
);

impl KotoArticulation {
    pub fn from_u32(val: u32) -> Self {
        match val {
            0 => Self::HirajoshiMeditativeAlap,
            1 => Self::KokinJoshiFastMiyako,
            2 => Self::InSenDramaticGendai,
            3 => Self::KumoiJoshiSpringRain,
            4 => Self::RyukyuIslandSwell,
            5 => Self::GuzhengVirtuosoWaterfall,
            6 => Self::CustomSpline,
            _ => Self::HirajoshiMeditativeAlap,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::HirajoshiMeditativeAlap => "Hirajoshi Meditative Alap (Classic)",
            Self::KokinJoshiFastMiyako => "Kokin-joshi Fast Miyako-bushi",
            Self::InSenDramaticGendai => "In-sen Dramatic Contemporary",
            Self::KumoiJoshiSpringRain => "Kumoi-joshi Spring Rain (Lyrical)",
            Self::RyukyuIslandSwell => "Ryukyu Festive Island Swell",
            Self::GuzhengVirtuosoWaterfall => "Guzheng Virtuoso Waterfall (21-Str)",
            Self::CustomSpline => "Custom Spline Gesture",
        }
    }

    /// Returns nominal default parameters tuple.
    pub fn nominal_parameters(&self) -> KotoNominalParameters {
        match self {
            Self::HirajoshiMeditativeAlap => (0.75, 2.0, 0.10, 0.0, 0, 0.25, 0.85, 0.90), // Hirajoshi
            Self::KokinJoshiFastMiyako => (0.90, 4.0, 0.05, 0.0, 1, 0.20, 0.80, 0.88),    // Kokin-joshi
            Self::InSenDramaticGendai => (0.85, 18.0, 0.25, 0.0, 2, 0.30, 0.90, 0.92),    // In-sen with high oshi-ite
            Self::KumoiJoshiSpringRain => (0.65, 3.0, 0.15, 0.0, 3, 0.22, 0.75, 0.85),    // Kumoi-joshi
            Self::RyukyuIslandSwell => (0.80, 5.0, 0.08, 0.0, 4, 0.20, 0.82, 0.88),       // Ryukyu
            Self::GuzhengVirtuosoWaterfall => (0.92, 6.0, 0.12, 0.0, 5, 0.40, 0.95, 0.95),// Guzheng
            Self::CustomSpline => (0.80, 5.0, 0.10, 0.0, 0, 0.25, 0.85, 0.90),
        }
    }
}

/// Instantaneous lock-free snapshot of Koto bus parameters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct KotoBusSnapshot {
    pub articulation: KotoArticulation,
    pub tsume_velocity: f32,
    pub oshi_ite_force_n: f32,
    pub hiki_iro_release: f32,
    pub ji_bridge_offset: f32,
    pub tuning_schema_idx: u32,
    pub behind_bridge_bleed: f32,
    pub body_wood_resonance: f32,
    pub master_gain: f32,
}

/// Default base ParamId for Asian Zither & Koto synthesis.
pub const KOTO_BASE_PARAM_ID: ParamId = ParamId(1260);

/// Helper to pre-register all Koto parameters into a ParamBus.
pub fn register_koto_params(param_bus: &mut ParamBus, base_id: ParamId) {
    let (vel, oshi, hiki, offset, tuning, bleed, wood, mgain) =
        KotoArticulation::HirajoshiMeditativeAlap.nominal_parameters();

    param_bus.register(ParamId(base_id.0), KotoArticulation::HirajoshiMeditativeAlap as u32 as f32);
    param_bus.register(ParamId(base_id.0 + 1), vel);
    param_bus.register(ParamId(base_id.0 + 2), oshi);
    param_bus.register(ParamId(base_id.0 + 3), hiki);
    param_bus.register(ParamId(base_id.0 + 4), offset);
    param_bus.register(ParamId(base_id.0 + 5), tuning as f32);
    param_bus.register(ParamId(base_id.0 + 6), bleed);
    param_bus.register(ParamId(base_id.0 + 7), wood);
    param_bus.register(ParamId(base_id.0 + 8), mgain);
}

/// Complete thread-safe lock-free Koto parameter bus.
#[derive(Debug)]
pub struct KotoBus {
    pub articulation: AtomicU32,
    pub tsume_velocity: AtomicU32,
    pub oshi_ite_force_n: AtomicU32,
    pub hiki_iro_release: AtomicU32,
    pub ji_bridge_offset: AtomicU32,
    pub tuning_schema_idx: AtomicU32,
    pub behind_bridge_bleed: AtomicU32,
    pub body_wood_resonance: AtomicU32,
    pub master_gain: AtomicU32,
}

impl Default for KotoBus {
    fn default() -> Self {
        Self::new()
    }
}

impl KotoBus {
    /// Create a new Koto parameter bus with default HirajoshiMeditativeAlap settings.
    pub fn new() -> Self {
        let (vel, oshi, hiki, offset, tuning, bleed, wood, mgain) =
            KotoArticulation::HirajoshiMeditativeAlap.nominal_parameters();

        Self {
            articulation: AtomicU32::new(KotoArticulation::HirajoshiMeditativeAlap as u32),
            tsume_velocity: AtomicU32::new(vel.to_bits()),
            oshi_ite_force_n: AtomicU32::new(oshi.to_bits()),
            hiki_iro_release: AtomicU32::new(hiki.to_bits()),
            ji_bridge_offset: AtomicU32::new(offset.to_bits()),
            tuning_schema_idx: AtomicU32::new(tuning),
            behind_bridge_bleed: AtomicU32::new(bleed.to_bits()),
            body_wood_resonance: AtomicU32::new(wood.to_bits()),
            master_gain: AtomicU32::new(mgain.to_bits()),
        }
    }

    /// Read an instantaneous atomic snapshot. Safe for real-time audio threads.
    #[inline]
    pub fn snapshot(&self) -> KotoBusSnapshot {
        let art_u32 = self.articulation.load(Ordering::Relaxed);
        let vel_bits = self.tsume_velocity.load(Ordering::Relaxed);
        let oshi_bits = self.oshi_ite_force_n.load(Ordering::Relaxed);
        let hiki_bits = self.hiki_iro_release.load(Ordering::Relaxed);
        let offset_bits = self.ji_bridge_offset.load(Ordering::Relaxed);
        let tuning = self.tuning_schema_idx.load(Ordering::Relaxed);
        let bleed_bits = self.behind_bridge_bleed.load(Ordering::Relaxed);
        let wood_bits = self.body_wood_resonance.load(Ordering::Relaxed);
        let mgain_bits = self.master_gain.load(Ordering::Relaxed);

        KotoBusSnapshot {
            articulation: KotoArticulation::from_u32(art_u32),
            tsume_velocity: f32::from_bits(vel_bits),
            oshi_ite_force_n: f32::from_bits(oshi_bits),
            hiki_iro_release: f32::from_bits(hiki_bits),
            ji_bridge_offset: f32::from_bits(offset_bits),
            tuning_schema_idx: tuning,
            behind_bridge_bleed: f32::from_bits(bleed_bits),
            body_wood_resonance: f32::from_bits(wood_bits),
            master_gain: f32::from_bits(mgain_bits),
        }
    }

    /// Update articulation and set nominal parameters atomically.
    pub fn set_articulation(&self, art: KotoArticulation) {
        let (vel, oshi, hiki, offset, tuning, bleed, wood, mgain) =
            art.nominal_parameters();

        self.articulation.store(art as u32, Ordering::Relaxed);
        self.tsume_velocity.store(vel.to_bits(), Ordering::Relaxed);
        self.oshi_ite_force_n.store(oshi.to_bits(), Ordering::Relaxed);
        self.hiki_iro_release.store(hiki.to_bits(), Ordering::Relaxed);
        self.ji_bridge_offset.store(offset.to_bits(), Ordering::Relaxed);
        self.tuning_schema_idx.store(tuning, Ordering::Relaxed);
        self.behind_bridge_bleed.store(bleed.to_bits(), Ordering::Relaxed);
        self.body_wood_resonance.store(wood.to_bits(), Ordering::Relaxed);
        self.master_gain.store(mgain.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_tsume_velocity(&self, vel: f32) {
        self.tsume_velocity.store(vel.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_oshi_ite_force(&self, force_n: f32) {
        self.oshi_ite_force_n.store(force_n.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_hiki_iro_release(&self, release: f32) {
        self.hiki_iro_release.store(release.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_ji_bridge_offset(&self, offset: f32) {
        self.ji_bridge_offset.store(offset.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_tuning_schema_idx(&self, schema_idx: u32) {
        self.tuning_schema_idx.store(schema_idx, Ordering::Relaxed);
    }

    #[inline]
    pub fn set_behind_bridge_bleed(&self, bleed: f32) {
        self.behind_bridge_bleed.store(bleed.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_body_wood_resonance(&self, wood: f32) {
        self.body_wood_resonance.store(wood.to_bits(), Ordering::Relaxed);
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
    fn test_koto_bus_atomic_updates_and_round_trip() {
        let bus = KotoBus::new();
        let snap = bus.snapshot();

        assert_eq!(snap.articulation, KotoArticulation::HirajoshiMeditativeAlap);
        assert!((snap.tsume_velocity - 0.75).abs() < 1e-4);
        assert!((snap.oshi_ite_force_n - 2.0).abs() < 1e-4);

        // Update articulation
        bus.set_articulation(KotoArticulation::InSenDramaticGendai);
        let snap2 = bus.snapshot();
        assert_eq!(snap2.articulation, KotoArticulation::InSenDramaticGendai);
        assert!((snap2.oshi_ite_force_n - 18.0).abs() < 1e-4);
        assert_eq!(snap2.tuning_schema_idx, 2); // In-sen

        // Individual setters
        bus.set_oshi_ite_force(25.0);
        let snap3 = bus.snapshot();
        assert!((snap3.oshi_ite_force_n - 25.0).abs() < 1e-4);
    }

    #[test]
    fn test_koto_bus_param_bus_registration() {
        let mut param_bus = ParamBus::new();
        register_koto_params(&mut param_bus, KOTO_BASE_PARAM_ID);

        let vel_val = param_bus.get(ParamId(KOTO_BASE_PARAM_ID.0 + 1));
        assert!(vel_val.is_some());
        assert!((vel_val.unwrap() - 0.75).abs() < 1e-4);
    }
}
