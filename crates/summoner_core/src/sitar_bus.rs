// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Lock-Free Sitar Bus Parameter Routing (Milestone 27).
//!
//! Provides thread-safe lock-free exchange of continuous physical modeling sitar
//! parameters (Mizrab Strike Velocity, Lateral Meend Pull Distance [0..5 st],
//! Jawari Bridge Clearance Gap $h_0$, Jiva Cotton Thread Position, Tarab Sympathetic
//! Coupling Bleed, Chikari Rhythmic Strum Trigger, Raga Scale Selection, Gourd Body
//! Decay Scale, Body Resonance Gain, and Master Gain) across audio, sequencer, and GUI threads.
//!
//! Enforces zero heap allocations in real-time audio threads under `AllocGuard`.

use crate::param_bus::{ParamBus, ParamId};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};

/// Classical and modern Sitar performance styles and gesture articulations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u32)]
pub enum SitarArticulation {
    /// Meditative Raga Yaman Alap (slow dynamic unfolding, deep microtonal Meend pulls).
    #[default]
    RagaYamanAlap = 0,
    /// Vilayat Khan Gayaki vocal style (intricate vocal-style bends, crisp camel-bone Jawari).
    VilayatKhanGayaki = 1,
    /// Ravi Shankar Kharaj Pancham style (deep bass drone, rich deer-horn Jawari sustain).
    RaviShankarKharaj = 2,
    /// Surbahar deep alap (slow, heavy brass string glissandi, 1-octave sub-bass resonance).
    SurbaharAlap = 3,
    /// Raga Bhairav Jor & Jhala (accelerating rhythmic pulse with active Chikari strumming).
    BhairavJorJhala = 4,
    /// Electric Sitar Jhajhar (sizzling flat Jawari buzz, sustained lead voice).
    ElectricSitarJhajhar = 5,
    /// Sympathetic Tarab Drone Focus (extended modal ringing across 13 Tarab strings).
    TarabSympatheticDrone = 6,
    /// Custom multi-keyframe Catmull-Rom spline trajectory.
    CustomSpline = 7,
}

/// Full nominal tuple parameters for Sitar articulation:
/// (mizrab_velocity, meend_pull_semitones, jawari_gap_mm, jiva_thread_pos, tarab_bleed,
///  chikari_trigger, raga_scale_id, gourd_decay, body_gain, master_gain)
pub type SitarNominalParameters = (
    f32, // mizrab_velocity [0.0 .. 1.0]
    f32, // meend_pull_semitones [0.0 .. 5.0]
    f32, // jawari_gap_mm [0.01 .. 1.5]
    f32, // jiva_thread_pos [0.0 .. 1.0]
    f32, // tarab_bleed [0.0 .. 1.0]
    f32, // chikari_trigger [0.0 .. 1.0]
    u32, // raga_scale_id [0..6] (0=Yaman, 1=Bhairav, 2=Bilawal, 3=Darbari, 4=Todi, 5=Kafi, 6=Bhairavi)
    f32, // gourd_decay [0.1 .. 3.0]
    f32, // body_gain [0.0 .. 2.0]
    f32, // master_gain [0.0 .. 2.0]
);

impl SitarArticulation {
    pub fn from_u32(val: u32) -> Self {
        match val {
            0 => Self::RagaYamanAlap,
            1 => Self::VilayatKhanGayaki,
            2 => Self::RaviShankarKharaj,
            3 => Self::SurbaharAlap,
            4 => Self::BhairavJorJhala,
            5 => Self::ElectricSitarJhajhar,
            6 => Self::TarabSympatheticDrone,
            7 => Self::CustomSpline,
            _ => Self::RagaYamanAlap,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::RagaYamanAlap => "Raga Yaman Alap (Meditative)",
            Self::VilayatKhanGayaki => "Vilayat Khan Gayaki (Vocal Style)",
            Self::RaviShankarKharaj => "Ravi Shankar Kharaj Pancham",
            Self::SurbaharAlap => "Surbahar Bass Deep Alap",
            Self::BhairavJorJhala => "Raga Bhairav Jor & Jhala",
            Self::ElectricSitarJhajhar => "Electric Sitar Jhajhar",
            Self::TarabSympatheticDrone => "Tarab Sympathetic Drone Focus",
            Self::CustomSpline => "Custom Spline Gesture",
        }
    }

    /// Returns nominal default parameters tuple.
    pub fn nominal_parameters(&self) -> SitarNominalParameters {
        match self {
            Self::RagaYamanAlap => (0.65, 1.5, 0.18, 0.45, 0.40, 0.0, 0, 1.1, 0.85, 0.85),
            Self::VilayatKhanGayaki => (0.75, 3.0, 0.08, 0.50, 0.35, 0.0, 0, 0.95, 0.80, 0.88),
            Self::RaviShankarKharaj => (0.80, 2.0, 0.20, 0.40, 0.45, 0.3, 1, 1.25, 0.90, 0.85),
            Self::SurbaharAlap => (0.70, 4.0, 0.35, 0.35, 0.50, 0.0, 3, 1.75, 0.95, 0.82),
            Self::BhairavJorJhala => (0.88, 0.5, 0.15, 0.48, 0.38, 0.8, 1, 1.0, 0.85, 0.90),
            Self::ElectricSitarJhajhar => (0.85, 2.5, 0.05, 0.60, 0.20, 0.5, 2, 0.80, 0.70, 0.88),
            Self::TarabSympatheticDrone => (0.50, 0.0, 0.22, 0.40, 0.75, 0.2, 0, 1.50, 0.95, 0.80),
            Self::CustomSpline => (0.65, 1.5, 0.18, 0.45, 0.40, 0.0, 0, 1.1, 0.85, 0.85),
        }
    }
}

/// Instantaneous lock-free snapshot of Sitar bus parameters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SitarBusSnapshot {
    pub articulation: SitarArticulation,
    pub mizrab_velocity: f32,
    pub meend_pull_semitones: f32,
    pub jawari_gap_mm: f32,
    pub jiva_thread_pos: f32,
    pub tarab_bleed: f32,
    pub chikari_trigger: f32,
    pub raga_scale_id: u32,
    pub gourd_decay: f32,
    pub body_gain: f32,
    pub master_gain: f32,
}

/// Default base ParamId for Sitar synthesis.
pub const SITAR_BASE_PARAM_ID: ParamId = ParamId(1200);

/// Helper to pre-register all Sitar parameters into a ParamBus.
pub fn register_sitar_params(param_bus: &mut ParamBus, base_id: ParamId) {
    let (miz, meend, gap, jiva, bleed, chik, raga, decay, bgain, mgain) =
        SitarArticulation::RagaYamanAlap.nominal_parameters();

    param_bus.register(ParamId(base_id.0), SitarArticulation::RagaYamanAlap as u32 as f32);
    param_bus.register(ParamId(base_id.0 + 1), miz);
    param_bus.register(ParamId(base_id.0 + 2), meend);
    param_bus.register(ParamId(base_id.0 + 3), gap);
    param_bus.register(ParamId(base_id.0 + 4), jiva);
    param_bus.register(ParamId(base_id.0 + 5), bleed);
    param_bus.register(ParamId(base_id.0 + 6), chik);
    param_bus.register(ParamId(base_id.0 + 7), raga as f32);
    param_bus.register(ParamId(base_id.0 + 8), decay);
    param_bus.register(ParamId(base_id.0 + 9), bgain);
    param_bus.register(ParamId(base_id.0 + 10), mgain);
}

/// Complete thread-safe lock-free Sitar parameter bus.
#[derive(Debug)]
pub struct SitarBus {
    pub articulation: AtomicU32,
    pub mizrab_velocity: AtomicU32,
    pub meend_pull_semitones: AtomicU32,
    pub jawari_gap_mm: AtomicU32,
    pub jiva_thread_pos: AtomicU32,
    pub tarab_bleed: AtomicU32,
    pub chikari_trigger: AtomicU32,
    pub raga_scale_id: AtomicU32,
    pub gourd_decay: AtomicU32,
    pub body_gain: AtomicU32,
    pub master_gain: AtomicU32,
}

impl Default for SitarBus {
    fn default() -> Self {
        Self::new()
    }
}

impl SitarBus {
    /// Create a new sitar parameter bus with default RagaYamanAlap settings.
    pub fn new() -> Self {
        let (miz, meend, gap, jiva, bleed, chik, raga, decay, bgain, mgain) =
            SitarArticulation::RagaYamanAlap.nominal_parameters();

        Self {
            articulation: AtomicU32::new(SitarArticulation::RagaYamanAlap as u32),
            mizrab_velocity: AtomicU32::new(miz.to_bits()),
            meend_pull_semitones: AtomicU32::new(meend.to_bits()),
            jawari_gap_mm: AtomicU32::new(gap.to_bits()),
            jiva_thread_pos: AtomicU32::new(jiva.to_bits()),
            tarab_bleed: AtomicU32::new(bleed.to_bits()),
            chikari_trigger: AtomicU32::new(chik.to_bits()),
            raga_scale_id: AtomicU32::new(raga),
            gourd_decay: AtomicU32::new(decay.to_bits()),
            body_gain: AtomicU32::new(bgain.to_bits()),
            master_gain: AtomicU32::new(mgain.to_bits()),
        }
    }

    /// Read an instantaneous atomic snapshot. Safe for real-time audio threads.
    #[inline]
    pub fn snapshot(&self) -> SitarBusSnapshot {
        let art_u32 = self.articulation.load(Ordering::Relaxed);
        let miz_bits = self.mizrab_velocity.load(Ordering::Relaxed);
        let meend_bits = self.meend_pull_semitones.load(Ordering::Relaxed);
        let gap_bits = self.jawari_gap_mm.load(Ordering::Relaxed);
        let jiva_bits = self.jiva_thread_pos.load(Ordering::Relaxed);
        let bleed_bits = self.tarab_bleed.load(Ordering::Relaxed);
        let chik_bits = self.chikari_trigger.load(Ordering::Relaxed);
        let raga = self.raga_scale_id.load(Ordering::Relaxed);
        let decay_bits = self.gourd_decay.load(Ordering::Relaxed);
        let bgain_bits = self.body_gain.load(Ordering::Relaxed);
        let mgain_bits = self.master_gain.load(Ordering::Relaxed);

        SitarBusSnapshot {
            articulation: SitarArticulation::from_u32(art_u32),
            mizrab_velocity: f32::from_bits(miz_bits),
            meend_pull_semitones: f32::from_bits(meend_bits),
            jawari_gap_mm: f32::from_bits(gap_bits),
            jiva_thread_pos: f32::from_bits(jiva_bits),
            tarab_bleed: f32::from_bits(bleed_bits),
            chikari_trigger: f32::from_bits(chik_bits),
            raga_scale_id: raga,
            gourd_decay: f32::from_bits(decay_bits),
            body_gain: f32::from_bits(bgain_bits),
            master_gain: f32::from_bits(mgain_bits),
        }
    }

    /// Update articulation and set nominal parameters atomically.
    pub fn set_articulation(&self, art: SitarArticulation) {
        let (miz, meend, gap, jiva, bleed, chik, raga, decay, bgain, mgain) =
            art.nominal_parameters();

        self.articulation.store(art as u32, Ordering::Relaxed);
        self.mizrab_velocity.store(miz.to_bits(), Ordering::Relaxed);
        self.meend_pull_semitones.store(meend.to_bits(), Ordering::Relaxed);
        self.jawari_gap_mm.store(gap.to_bits(), Ordering::Relaxed);
        self.jiva_thread_pos.store(jiva.to_bits(), Ordering::Relaxed);
        self.tarab_bleed.store(bleed.to_bits(), Ordering::Relaxed);
        self.chikari_trigger.store(chik.to_bits(), Ordering::Relaxed);
        self.raga_scale_id.store(raga, Ordering::Relaxed);
        self.gourd_decay.store(decay.to_bits(), Ordering::Relaxed);
        self.body_gain.store(bgain.to_bits(), Ordering::Relaxed);
        self.master_gain.store(mgain.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_mizrab_velocity(&self, vel: f32) {
        self.mizrab_velocity.store(vel.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_meend_pull(&self, semitones: f32) {
        self.meend_pull_semitones.store(semitones.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_jawari_gap(&self, gap_mm: f32) {
        self.jawari_gap_mm.store(gap_mm.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_jiva_thread_pos(&self, pos: f32) {
        self.jiva_thread_pos.store(pos.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_tarab_bleed(&self, bleed: f32) {
        self.tarab_bleed.store(bleed.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_chikari_trigger(&self, trig: f32) {
        self.chikari_trigger.store(trig.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_raga_scale_id(&self, scale_id: u32) {
        self.raga_scale_id.store(scale_id, Ordering::Relaxed);
    }

    #[inline]
    pub fn set_gourd_decay(&self, decay: f32) {
        self.gourd_decay.store(decay.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_body_gain(&self, gain: f32) {
        self.body_gain.store(gain.to_bits(), Ordering::Relaxed);
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
    fn test_sitar_bus_articulation_round_trip() {
        let bus = SitarBus::new();
        let snap = bus.snapshot();
        assert_eq!(snap.articulation, SitarArticulation::RagaYamanAlap);
        assert!((snap.meend_pull_semitones - 1.5).abs() < 1e-4);

        bus.set_articulation(SitarArticulation::VilayatKhanGayaki);
        let snap2 = bus.snapshot();
        assert_eq!(snap2.articulation, SitarArticulation::VilayatKhanGayaki);
        assert!((snap2.meend_pull_semitones - 3.0).abs() < 1e-4);
        assert!((snap2.jawari_gap_mm - 0.08).abs() < 1e-4);
    }
}
