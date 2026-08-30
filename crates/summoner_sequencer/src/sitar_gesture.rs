// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Dynamic Sitar Articulation & Meend Pitch Gesture Timeline (Milestone 27).
//!
//! Provides multi-dimensional continuous physical modeling trajectory generation for
//! Sitar Mizrab stroke velocities, continuous lateral Meend pitch deflection [0..5 st],
//! Jawari bridge clearance gaps, Jiva thread damping, Tarab sympathetic coupling bleed,
//! Chikari drone strum triggers, and Raga scale selections with Catmull-Rom spline smoothing,
//! TOML serialization, and sample-accurate lock-free parameter dispatch into
//! `SitarBus` and `ParamBus`.
//!
//! Enforces zero heap allocations in real-time execution loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use summoner_core::param_bus::{ParamBus, ParamId};
use summoner_core::sitar_bus::{
    SitarArticulation, SitarBus, SitarBusSnapshot,
};

/// Interpolation curve between discrete Sitar keyframe waypoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SitarInterpolationCurve {
    #[default]
    CubicCatmullRom,
    CubicBezier,
    Linear,
    Hold,
    Exponential,
}

/// A single keyframed multi-dimensional Sitar articulation waypoint.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SitarWaypoint {
    /// Timeline beat position in quarter note beats.
    pub beat: f64,
    /// Mizrab strike velocity $[0.0 ..= 1.0]$.
    pub mizrab_velocity: f32,
    /// Lateral Meend pull pitch deflection $[0.0 ..= 5.0]$ in semitones.
    pub meend_pull_semitones: f32,
    /// Jawari clearance gap $h_0$ in mm $[0.01 ..= 1.5]$.
    pub jawari_gap_mm: f32,
    /// Jiva cotton thread position $[0.0 ..= 1.0]$.
    pub jiva_thread_pos: f32,
    /// Tarab sympathetic coupling bleed $[0.0 ..= 1.0]$.
    pub tarab_bleed: f32,
    /// Chikari rhythmic strum trigger $[0.0 ..= 1.0]$.
    pub chikari_trigger: f32,
    /// Raga scale selection ID [0..6].
    pub raga_scale_id: u32,
    /// Gourd body decay scale $[0.1 ..= 3.0]$.
    pub gourd_decay: f32,
    /// Body resonance gain $[0.0 ..= 2.0]$.
    pub body_gain: f32,
    /// Master gain $[0.0 ..= 2.0]$.
    pub master_gain: f32,
    /// Articulation technique.
    pub articulation: SitarArticulation,
    /// Interpolation curve leading to this waypoint.
    pub curve: SitarInterpolationCurve,
}

impl Default for SitarWaypoint {
    fn default() -> Self {
        let (miz, meend, gap, jiva, bleed, chik, raga, decay, bgain, mgain) =
            SitarArticulation::RagaYamanAlap.nominal_parameters();

        Self {
            beat: 0.0,
            mizrab_velocity: miz,
            meend_pull_semitones: meend,
            jawari_gap_mm: gap,
            jiva_thread_pos: jiva,
            tarab_bleed: bleed,
            chikari_trigger: chik,
            raga_scale_id: raga,
            gourd_decay: decay,
            body_gain: bgain,
            master_gain: mgain,
            articulation: SitarArticulation::RagaYamanAlap,
            curve: SitarInterpolationCurve::CubicCatmullRom,
        }
    }
}

/// Pre-configured algorithmic Sitar gesture trajectory patterns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SitarGesturePattern {
    /// Meditative Raga Yaman Alap with wide, slow Meend glissandi and soft plucks.
    RagaYamanAlap {
        phrase_length_beats: f64,
        max_meend_pull: f32,
    },
    /// Intricate Vilayat Khan vocal Gayaki with quick microtonal ornamentation.
    VilayatKhanGayaki {
        phrase_length_beats: f64,
        max_velocity: f32,
    },
    /// Ravi Shankar rhythmic Jhala with high-density Chikari drone strumming.
    RaviShankarKharajJhala {
        phrase_length_beats: f64,
        chikari_density: f32,
    },
    /// Surbahar deep alap with low bass resonance and heavy Jawari buzz.
    SurbaharDeepMeditation {
        phrase_length_beats: f64,
    },
    /// Extended Tarab sympathetic resonance cascade across 13 sympathetic strings.
    TarabSympatheticCascade {
        phrase_length_beats: f64,
    },
}

/// Dynamic Sitar articulation timeline and parameter trajectory engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SitarGestureEngine {
    pub waypoints: Vec<SitarWaypoint>,
    pub loop_length_beats: f64,
    pub is_looping: bool,
}

impl Default for SitarGestureEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SitarGestureEngine {
    pub fn new() -> Self {
        Self {
            waypoints: Vec::new(),
            loop_length_beats: 4.0,
            is_looping: true,
        }
    }

    pub fn with_pattern(pattern: SitarGesturePattern) -> Self {
        let mut engine = Self::new();
        engine.load_pattern(pattern);
        engine
    }

    pub fn add_waypoint(&mut self, waypoint: SitarWaypoint) {
        self.waypoints.push(waypoint);
        self.sort_waypoints();
    }

    pub fn sort_waypoints(&mut self) {
        self.waypoints.sort_by(|a, b| a.beat.partial_cmp(&b.beat).unwrap_or(std::cmp::Ordering::Equal));
    }

    pub fn clear(&mut self) {
        self.waypoints.clear();
    }

    pub fn load_pattern(&mut self, pattern: SitarGesturePattern) {
        self.clear();

        match &pattern {
            SitarGesturePattern::RagaYamanAlap { phrase_length_beats, max_meend_pull } => {
                self.loop_length_beats = *phrase_length_beats;

                self.waypoints.push(SitarWaypoint {
                    beat: 0.0,
                    mizrab_velocity: 0.55,
                    meend_pull_semitones: 0.0,
                    jawari_gap_mm: 0.18,
                    jiva_thread_pos: 0.45,
                    tarab_bleed: 0.40,
                    chikari_trigger: 0.0,
                    raga_scale_id: 0, // Yaman
                    gourd_decay: 1.1,
                    body_gain: 0.85,
                    master_gain: 0.85,
                    articulation: SitarArticulation::RagaYamanAlap,
                    curve: SitarInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(SitarWaypoint {
                    beat: phrase_length_beats * 0.4,
                    mizrab_velocity: 0.70,
                    meend_pull_semitones: *max_meend_pull * 0.8,
                    jawari_gap_mm: 0.16,
                    jiva_thread_pos: 0.48,
                    tarab_bleed: 0.45,
                    chikari_trigger: 0.0,
                    raga_scale_id: 0,
                    gourd_decay: 1.2,
                    body_gain: 0.88,
                    master_gain: 0.88,
                    articulation: SitarArticulation::RagaYamanAlap,
                    curve: SitarInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(SitarWaypoint {
                    beat: phrase_length_beats * 0.75,
                    mizrab_velocity: 0.75,
                    meend_pull_semitones: *max_meend_pull,
                    jawari_gap_mm: 0.14,
                    jiva_thread_pos: 0.50,
                    tarab_bleed: 0.50,
                    chikari_trigger: 0.0,
                    raga_scale_id: 0,
                    gourd_decay: 1.3,
                    body_gain: 0.90,
                    master_gain: 0.90,
                    articulation: SitarArticulation::RagaYamanAlap,
                    curve: SitarInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(SitarWaypoint {
                    beat: *phrase_length_beats,
                    mizrab_velocity: 0.50,
                    meend_pull_semitones: 0.0,
                    jawari_gap_mm: 0.18,
                    jiva_thread_pos: 0.45,
                    tarab_bleed: 0.40,
                    chikari_trigger: 0.0,
                    raga_scale_id: 0,
                    gourd_decay: 1.1,
                    body_gain: 0.85,
                    master_gain: 0.85,
                    articulation: SitarArticulation::RagaYamanAlap,
                    curve: SitarInterpolationCurve::CubicCatmullRom,
                });
            }

            SitarGesturePattern::VilayatKhanGayaki { phrase_length_beats, max_velocity } => {
                self.loop_length_beats = *phrase_length_beats;

                self.waypoints.push(SitarWaypoint {
                    beat: 0.0,
                    mizrab_velocity: *max_velocity * 0.6,
                    meend_pull_semitones: 0.5,
                    jawari_gap_mm: 0.08,
                    jiva_thread_pos: 0.50,
                    tarab_bleed: 0.35,
                    chikari_trigger: 0.0,
                    raga_scale_id: 1, // Bhairav
                    gourd_decay: 0.95,
                    body_gain: 0.80,
                    master_gain: 0.88,
                    articulation: SitarArticulation::VilayatKhanGayaki,
                    curve: SitarInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(SitarWaypoint {
                    beat: phrase_length_beats * 0.5,
                    mizrab_velocity: *max_velocity,
                    meend_pull_semitones: 3.5,
                    jawari_gap_mm: 0.07,
                    jiva_thread_pos: 0.52,
                    tarab_bleed: 0.40,
                    chikari_trigger: 0.1,
                    raga_scale_id: 1,
                    gourd_decay: 1.0,
                    body_gain: 0.85,
                    master_gain: 0.90,
                    articulation: SitarArticulation::VilayatKhanGayaki,
                    curve: SitarInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(SitarWaypoint {
                    beat: *phrase_length_beats,
                    mizrab_velocity: *max_velocity * 0.5,
                    meend_pull_semitones: 0.5,
                    jawari_gap_mm: 0.08,
                    jiva_thread_pos: 0.50,
                    tarab_bleed: 0.35,
                    chikari_trigger: 0.0,
                    raga_scale_id: 1,
                    gourd_decay: 0.95,
                    body_gain: 0.80,
                    master_gain: 0.88,
                    articulation: SitarArticulation::VilayatKhanGayaki,
                    curve: SitarInterpolationCurve::CubicCatmullRom,
                });
            }

            SitarGesturePattern::RaviShankarKharajJhala { phrase_length_beats, chikari_density } => {
                self.loop_length_beats = *phrase_length_beats;

                self.waypoints.push(SitarWaypoint {
                    beat: 0.0,
                    mizrab_velocity: 0.75,
                    meend_pull_semitones: 0.0,
                    jawari_gap_mm: 0.20,
                    jiva_thread_pos: 0.40,
                    tarab_bleed: 0.45,
                    chikari_trigger: 0.2,
                    raga_scale_id: 0,
                    gourd_decay: 1.25,
                    body_gain: 0.90,
                    master_gain: 0.85,
                    articulation: SitarArticulation::BhairavJorJhala,
                    curve: SitarInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(SitarWaypoint {
                    beat: phrase_length_beats * 0.5,
                    mizrab_velocity: 0.90,
                    meend_pull_semitones: 1.0,
                    jawari_gap_mm: 0.18,
                    jiva_thread_pos: 0.42,
                    tarab_bleed: 0.50,
                    chikari_trigger: *chikari_density,
                    raga_scale_id: 0,
                    gourd_decay: 1.30,
                    body_gain: 0.92,
                    master_gain: 0.90,
                    articulation: SitarArticulation::BhairavJorJhala,
                    curve: SitarInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(SitarWaypoint {
                    beat: *phrase_length_beats,
                    mizrab_velocity: 0.78,
                    meend_pull_semitones: 0.0,
                    jawari_gap_mm: 0.20,
                    jiva_thread_pos: 0.40,
                    tarab_bleed: 0.45,
                    chikari_trigger: 0.2,
                    raga_scale_id: 0,
                    gourd_decay: 1.25,
                    body_gain: 0.90,
                    master_gain: 0.85,
                    articulation: SitarArticulation::BhairavJorJhala,
                    curve: SitarInterpolationCurve::CubicCatmullRom,
                });
            }

            SitarGesturePattern::SurbaharDeepMeditation { phrase_length_beats } => {
                self.loop_length_beats = *phrase_length_beats;

                self.waypoints.push(SitarWaypoint {
                    beat: 0.0,
                    mizrab_velocity: 0.65,
                    meend_pull_semitones: 0.0,
                    jawari_gap_mm: 0.35,
                    jiva_thread_pos: 0.35,
                    tarab_bleed: 0.50,
                    chikari_trigger: 0.0,
                    raga_scale_id: 3, // Darbari
                    gourd_decay: 1.75,
                    body_gain: 0.95,
                    master_gain: 0.82,
                    articulation: SitarArticulation::SurbaharAlap,
                    curve: SitarInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(SitarWaypoint {
                    beat: phrase_length_beats * 0.5,
                    mizrab_velocity: 0.75,
                    meend_pull_semitones: 4.0,
                    jawari_gap_mm: 0.30,
                    jiva_thread_pos: 0.38,
                    tarab_bleed: 0.55,
                    chikari_trigger: 0.0,
                    raga_scale_id: 3,
                    gourd_decay: 1.90,
                    body_gain: 0.98,
                    master_gain: 0.85,
                    articulation: SitarArticulation::SurbaharAlap,
                    curve: SitarInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(SitarWaypoint {
                    beat: *phrase_length_beats,
                    mizrab_velocity: 0.60,
                    meend_pull_semitones: 0.0,
                    jawari_gap_mm: 0.35,
                    jiva_thread_pos: 0.35,
                    tarab_bleed: 0.50,
                    chikari_trigger: 0.0,
                    raga_scale_id: 3,
                    gourd_decay: 1.75,
                    body_gain: 0.95,
                    master_gain: 0.82,
                    articulation: SitarArticulation::SurbaharAlap,
                    curve: SitarInterpolationCurve::CubicCatmullRom,
                });
            }

            SitarGesturePattern::TarabSympatheticCascade { phrase_length_beats } => {
                self.loop_length_beats = *phrase_length_beats;

                self.waypoints.push(SitarWaypoint {
                    beat: 0.0,
                    mizrab_velocity: 0.50,
                    meend_pull_semitones: 0.0,
                    jawari_gap_mm: 0.22,
                    jiva_thread_pos: 0.40,
                    tarab_bleed: 0.75,
                    chikari_trigger: 0.2,
                    raga_scale_id: 0,
                    gourd_decay: 1.50,
                    body_gain: 0.95,
                    master_gain: 0.80,
                    articulation: SitarArticulation::TarabSympatheticDrone,
                    curve: SitarInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(SitarWaypoint {
                    beat: phrase_length_beats * 0.5,
                    mizrab_velocity: 0.65,
                    meend_pull_semitones: 1.0,
                    jawari_gap_mm: 0.20,
                    jiva_thread_pos: 0.42,
                    tarab_bleed: 0.90,
                    chikari_trigger: 0.4,
                    raga_scale_id: 0,
                    gourd_decay: 1.65,
                    body_gain: 0.98,
                    master_gain: 0.82,
                    articulation: SitarArticulation::TarabSympatheticDrone,
                    curve: SitarInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(SitarWaypoint {
                    beat: *phrase_length_beats,
                    mizrab_velocity: 0.48,
                    meend_pull_semitones: 0.0,
                    jawari_gap_mm: 0.22,
                    jiva_thread_pos: 0.40,
                    tarab_bleed: 0.75,
                    chikari_trigger: 0.2,
                    raga_scale_id: 0,
                    gourd_decay: 1.50,
                    body_gain: 0.95,
                    master_gain: 0.80,
                    articulation: SitarArticulation::TarabSympatheticDrone,
                    curve: SitarInterpolationCurve::CubicCatmullRom,
                });
            }
        }
    }

    /// Evaluate timeline trajectory at arbitrary beat and return interpolated snapshot.
    pub fn evaluate_at_beat(&self, beat: f64) -> SitarBusSnapshot {
        if self.waypoints.is_empty() {
            let (miz, meend, gap, jiva, bleed, chik, raga, decay, bgain, mgain) =
                SitarArticulation::RagaYamanAlap.nominal_parameters();
            return SitarBusSnapshot {
                articulation: SitarArticulation::RagaYamanAlap,
                mizrab_velocity: miz,
                meend_pull_semitones: meend,
                jawari_gap_mm: gap,
                jiva_thread_pos: jiva,
                tarab_bleed: bleed,
                chikari_trigger: chik,
                raga_scale_id: raga,
                gourd_decay: decay,
                body_gain: bgain,
                master_gain: mgain,
            };
        }

        if self.waypoints.len() == 1 {
            let wp = &self.waypoints[0];
            return SitarBusSnapshot {
                articulation: wp.articulation,
                mizrab_velocity: wp.mizrab_velocity,
                meend_pull_semitones: wp.meend_pull_semitones,
                jawari_gap_mm: wp.jawari_gap_mm,
                jiva_thread_pos: wp.jiva_thread_pos,
                tarab_bleed: wp.tarab_bleed,
                chikari_trigger: wp.chikari_trigger,
                raga_scale_id: wp.raga_scale_id,
                gourd_decay: wp.gourd_decay,
                body_gain: wp.body_gain,
                master_gain: wp.master_gain,
            };
        }

        let effective_beat = if self.is_looping && self.loop_length_beats > 0.0 {
            beat.rem_euclid(self.loop_length_beats)
        } else {
            beat
        };

        // Find surrounding waypoints for Catmull-Rom spline evaluation
        let n = self.waypoints.len();
        let mut idx1 = 0;
        while idx1 < n && self.waypoints[idx1].beat <= effective_beat {
            idx1 += 1;
        }

        if idx1 == 0 {
            let wp = &self.waypoints[0];
            return SitarBusSnapshot {
                articulation: wp.articulation,
                mizrab_velocity: wp.mizrab_velocity,
                meend_pull_semitones: wp.meend_pull_semitones,
                jawari_gap_mm: wp.jawari_gap_mm,
                jiva_thread_pos: wp.jiva_thread_pos,
                tarab_bleed: wp.tarab_bleed,
                chikari_trigger: wp.chikari_trigger,
                raga_scale_id: wp.raga_scale_id,
                gourd_decay: wp.gourd_decay,
                body_gain: wp.body_gain,
                master_gain: wp.master_gain,
            };
        }

        if idx1 >= n {
            let wp = &self.waypoints[n - 1];
            return SitarBusSnapshot {
                articulation: wp.articulation,
                mizrab_velocity: wp.mizrab_velocity,
                meend_pull_semitones: wp.meend_pull_semitones,
                jawari_gap_mm: wp.jawari_gap_mm,
                jiva_thread_pos: wp.jiva_thread_pos,
                tarab_bleed: wp.tarab_bleed,
                chikari_trigger: wp.chikari_trigger,
                raga_scale_id: wp.raga_scale_id,
                gourd_decay: wp.gourd_decay,
                body_gain: wp.body_gain,
                master_gain: wp.master_gain,
            };
        }

        let i1 = idx1 - 1;
        let i2 = idx1;
        let i0 = if i1 > 0 { i1 - 1 } else { i1 };
        let i3 = if i2 + 1 < n { i2 + 1 } else { i2 };

        let wp0 = &self.waypoints[i0];
        let wp1 = &self.waypoints[i1];
        let wp2 = &self.waypoints[i2];
        let wp3 = &self.waypoints[i3];

        let span = (wp2.beat - wp1.beat).max(1e-6);
        let t = ((effective_beat - wp1.beat) / span).clamp(0.0, 1.0) as f32;

        let interp = |v0: f32, v1: f32, v2: f32, v3: f32, curve: SitarInterpolationCurve| -> f32 {
            match curve {
                SitarInterpolationCurve::Hold => v1,
                SitarInterpolationCurve::Linear => v1 + t * (v2 - v1),
                SitarInterpolationCurve::Exponential => {
                    let min_v = v1.min(v2).max(1e-5);
                    let ratio = (v2.max(1e-5) / min_v).max(1e-5);
                    v1 * ratio.powf(t)
                }
                SitarInterpolationCurve::CubicBezier => {
                    let t2 = t * t;
                    let t3 = t2 * t;
                    let b = 3.0 * t2 - 2.0 * t3;
                    v1 + b * (v2 - v1)
                }
                SitarInterpolationCurve::CubicCatmullRom => {
                    let t2 = t * t;
                    let t3 = t2 * t;
                    0.5 * ((2.0 * v1)
                        + (-v0 + v2) * t
                        + (2.0 * v0 - 5.0 * v1 + 4.0 * v2 - v3) * t2
                        + (-v0 + 3.0 * v1 - 3.0 * v2 + v3) * t3)
                }
            }
        };

        let curve = wp2.curve;
        let miz = interp(wp0.mizrab_velocity, wp1.mizrab_velocity, wp2.mizrab_velocity, wp3.mizrab_velocity, curve).clamp(0.0, 1.0);
        let meend = interp(wp0.meend_pull_semitones, wp1.meend_pull_semitones, wp2.meend_pull_semitones, wp3.meend_pull_semitones, curve).clamp(0.0, 5.0);
        let gap = interp(wp0.jawari_gap_mm, wp1.jawari_gap_mm, wp2.jawari_gap_mm, wp3.jawari_gap_mm, curve).clamp(0.01, 1.5);
        let jiva = interp(wp0.jiva_thread_pos, wp1.jiva_thread_pos, wp2.jiva_thread_pos, wp3.jiva_thread_pos, curve).clamp(0.0, 1.0);
        let bleed = interp(wp0.tarab_bleed, wp1.tarab_bleed, wp2.tarab_bleed, wp3.tarab_bleed, curve).clamp(0.0, 1.0);
        let chik = interp(wp0.chikari_trigger, wp1.chikari_trigger, wp2.chikari_trigger, wp3.chikari_trigger, curve).clamp(0.0, 1.0);
        let decay = interp(wp0.gourd_decay, wp1.gourd_decay, wp2.gourd_decay, wp3.gourd_decay, curve).clamp(0.1, 3.0);
        let bgain = interp(wp0.body_gain, wp1.body_gain, wp2.body_gain, wp3.body_gain, curve).clamp(0.0, 2.0);
        let mgain = interp(wp0.master_gain, wp1.master_gain, wp2.master_gain, wp3.master_gain, curve).clamp(0.0, 2.0);

        SitarBusSnapshot {
            articulation: if t > 0.5 { wp2.articulation } else { wp1.articulation },
            mizrab_velocity: miz,
            meend_pull_semitones: meend,
            jawari_gap_mm: gap,
            jiva_thread_pos: jiva,
            tarab_bleed: bleed,
            chikari_trigger: chik,
            raga_scale_id: if t > 0.5 { wp2.raga_scale_id } else { wp1.raga_scale_id },
            gourd_decay: decay,
            body_gain: bgain,
            master_gain: mgain,
        }
    }

    /// Dispatch interpolated parameters into `SitarBus` and `ParamBus`.
    pub fn dispatch_to_bus(
        &self,
        beat: f64,
        bus: &SitarBus,
        param_bus: Option<&ParamBus>,
        base_id: ParamId,
    ) {
        let snap = self.evaluate_at_beat(beat);

        bus.set_mizrab_velocity(snap.mizrab_velocity);
        bus.set_meend_pull(snap.meend_pull_semitones);
        bus.set_jawari_gap(snap.jawari_gap_mm);
        bus.set_jiva_thread_pos(snap.jiva_thread_pos);
        bus.set_tarab_bleed(snap.tarab_bleed);
        bus.set_chikari_trigger(snap.chikari_trigger);
        bus.set_raga_scale_id(snap.raga_scale_id);
        bus.set_gourd_decay(snap.gourd_decay);
        bus.set_body_gain(snap.body_gain);
        bus.set_master_gain(snap.master_gain);

        if let Some(pb) = param_bus {
            pb.set(ParamId(base_id.0), snap.articulation as u32 as f32);
            pb.set(ParamId(base_id.0 + 1), snap.mizrab_velocity);
            pb.set(ParamId(base_id.0 + 2), snap.meend_pull_semitones);
            pb.set(ParamId(base_id.0 + 3), snap.jawari_gap_mm);
            pb.set(ParamId(base_id.0 + 4), snap.jiva_thread_pos);
            pb.set(ParamId(base_id.0 + 5), snap.tarab_bleed);
            pb.set(ParamId(base_id.0 + 6), snap.chikari_trigger);
            pb.set(ParamId(base_id.0 + 7), snap.raga_scale_id as f32);
            pb.set(ParamId(base_id.0 + 8), snap.gourd_decay);
            pb.set(ParamId(base_id.0 + 9), snap.body_gain);
            pb.set(ParamId(base_id.0 + 10), snap.master_gain);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sitar_gesture_engine_yaman_evaluation() {
        let engine = SitarGestureEngine::with_pattern(
            SitarGesturePattern::RagaYamanAlap {
                phrase_length_beats: 8.0,
                max_meend_pull: 4.0,
            },
        );

        let snap_start = engine.evaluate_at_beat(0.0);
        assert_eq!(snap_start.articulation, SitarArticulation::RagaYamanAlap);
        assert!((snap_start.meend_pull_semitones - 0.0).abs() < 1e-3);

        let snap_mid = engine.evaluate_at_beat(4.0);
        assert!(snap_mid.meend_pull_semitones > 2.0);
        assert!(snap_mid.mizrab_velocity > 0.60);
    }

    #[test]
    fn test_sitar_gesture_engine_toml_roundtrip() {
        let engine = SitarGestureEngine::with_pattern(
            SitarGesturePattern::VilayatKhanGayaki {
                phrase_length_beats: 4.0,
                max_velocity: 0.95,
            },
        );

        let toml_str = toml::to_string(&engine).expect("Failed to serialize SitarGestureEngine");
        let deserialized: SitarGestureEngine =
            toml::from_str(&toml_str).expect("Failed to deserialize SitarGestureEngine");
        assert_eq!(engine.waypoints.len(), deserialized.waypoints.len());
    }
}
