// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Dynamic Asian Zither & Koto Gesture Timeline & Articulation Engine (Milestone 29).
//!
//! Provides multi-dimensional continuous physical modeling trajectory generation for
//! Tsume strike velocities, Oshi-Ite left-hand press forces [0..30 N] yielding microtonal
//! pitch bends up to +400 cents, Hiki-Iro releases, movable Ji bridge offsets,
//! traditional modal tuning scales (Hirajoshi, Kokin-joshi, In-sen, Kumoi-joshi, Ryukyu, Guzheng),
//! behind-the-bridge sympathetic bleeds, and Paulownia wood resonances with Catmull-Rom spline
//! smoothing, TOML serialization, and sample-accurate lock-free parameter dispatch into
//! `KotoBus` and `ParamBus`.
//!
//! Enforces zero heap allocations in real-time execution loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use summoner_core::koto_bus::{
    KotoArticulation, KotoBus, KotoBusSnapshot,
};
use summoner_core::param_bus::{ParamBus, ParamId};

/// Interpolation curve between discrete Koto keyframe waypoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum KotoInterpolationCurve {
    #[default]
    CubicCatmullRom,
    CubicBezier,
    Linear,
    Hold,
    Exponential,
}

/// A single keyframed multi-dimensional Koto articulation waypoint.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct KotoWaypoint {
    /// Timeline beat position in quarter note beats.
    pub beat: f64,
    /// Tsume pluck velocity $[0.01 ..= 1.0]$.
    pub tsume_velocity: f32,
    /// Left-hand Oshi-Ite press force in Newtons $[0.0 ..= 30.0]$ (yielding up to +400 cents).
    pub oshi_ite_force_n: f32,
    /// Left-hand Hiki-Iro release fraction $[0.0 ..= 1.0]$.
    pub hiki_iro_release: f32,
    /// Ji bridge position offset $[-0.20 ..= +0.20]$.
    pub ji_bridge_offset: f32,
    /// Active tuning schema index $[0..=5]$.
    pub tuning_schema_idx: u32,
    /// Behind-the-bridge sympathetic resonance bleed fraction $[0.0 ..= 1.0]$.
    pub behind_bridge_bleed: f32,
    /// Paulownia soundboard wood cavity resonance gain $[0.0 ..= 1.0]$.
    pub body_wood_resonance: f32,
    /// Master output gain $[0.0 ..= 2.0]$.
    pub master_gain: f32,
    /// Articulation technique.
    pub articulation: KotoArticulation,
    /// Interpolation curve leading to this waypoint.
    pub curve: KotoInterpolationCurve,
}

impl Default for KotoWaypoint {
    fn default() -> Self {
        let (vel, oshi, hiki, offset, tuning, bleed, wood, mgain) =
            KotoArticulation::HirajoshiMeditativeAlap.nominal_parameters();

        Self {
            beat: 0.0,
            tsume_velocity: vel,
            oshi_ite_force_n: oshi,
            hiki_iro_release: hiki,
            ji_bridge_offset: offset,
            tuning_schema_idx: tuning,
            behind_bridge_bleed: bleed,
            body_wood_resonance: wood,
            master_gain: mgain,
            articulation: KotoArticulation::HirajoshiMeditativeAlap,
            curve: KotoInterpolationCurve::CubicCatmullRom,
        }
    }
}

/// Pre-configured algorithmic Koto gesture trajectory patterns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KotoGesturePattern {
    /// Traditional Meditative Alap with gentle yuri vibrato in Hirajoshi tuning.
    HirajoshiMeditativeAlap {
        phrase_length_beats: f64,
        max_oshi_ite_n: f32,
    },
    /// Kokin-joshi Fast Miyako-bushi with crisp staccato tsume attacks.
    KokinJoshiFastMiyako {
        phrase_length_beats: f64,
    },
    /// In-sen Contemporary Dramatic with wide dynamic oshi-ite microtonal tension bends.
    InSenDramaticGendai {
        phrase_length_beats: f64,
        max_pitch_bend_n: f32,
    },
    /// Kumoi-joshi Spring Rain with delicate tremolo and sukui-tsume sweeps.
    KumoiJoshiSpringRain {
        phrase_length_beats: f64,
    },
    /// Ryukyu Festive Island Swell with bright Okinawan pentatonic glissandi.
    RyukyuIslandSwell {
        phrase_length_beats: f64,
    },
    /// Guzheng Virtuoso Waterfall with rapid 21-string harp-like cascading flow.
    GuzhengVirtuosoWaterfall {
        phrase_length_beats: f64,
        arpeggio_rate_hz: f32,
    },
}

/// Dynamic Koto articulation timeline and parameter trajectory engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KotoGestureEngine {
    pub waypoints: Vec<KotoWaypoint>,
    pub loop_length_beats: f64,
    pub is_looping: bool,
}

impl Default for KotoGestureEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl KotoGestureEngine {
    pub fn new() -> Self {
        Self {
            waypoints: Vec::new(),
            loop_length_beats: 4.0,
            is_looping: true,
        }
    }

    pub fn with_pattern(pattern: KotoGesturePattern) -> Self {
        let mut engine = Self::new();
        engine.load_pattern(pattern);
        engine
    }

    pub fn add_waypoint(&mut self, waypoint: KotoWaypoint) {
        self.waypoints.push(waypoint);
        self.sort_waypoints();
    }

    pub fn sort_waypoints(&mut self) {
        self.waypoints.sort_by(|a, b| a.beat.partial_cmp(&b.beat).unwrap_or(std::cmp::Ordering::Equal));
    }

    pub fn clear(&mut self) {
        self.waypoints.clear();
    }

    pub fn load_pattern(&mut self, pattern: KotoGesturePattern) {
        self.clear();

        match &pattern {
            KotoGesturePattern::HirajoshiMeditativeAlap { phrase_length_beats, max_oshi_ite_n } => {
                self.loop_length_beats = *phrase_length_beats;
                let max_f = *max_oshi_ite_n;

                self.waypoints.push(KotoWaypoint {
                    beat: 0.0,
                    tsume_velocity: 0.70,
                    oshi_ite_force_n: 1.0,
                    hiki_iro_release: 0.05,
                    ji_bridge_offset: 0.0,
                    tuning_schema_idx: 0, // Hirajoshi
                    behind_bridge_bleed: 0.25,
                    body_wood_resonance: 0.85,
                    master_gain: 0.88,
                    articulation: KotoArticulation::HirajoshiMeditativeAlap,
                    curve: KotoInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(KotoWaypoint {
                    beat: phrase_length_beats * 0.35,
                    tsume_velocity: 0.82,
                    oshi_ite_force_n: max_f * 0.7, // Deep oshi-ite press
                    hiki_iro_release: 0.15,
                    ji_bridge_offset: 0.0,
                    tuning_schema_idx: 0,
                    behind_bridge_bleed: 0.28,
                    body_wood_resonance: 0.88,
                    master_gain: 0.90,
                    articulation: KotoArticulation::HirajoshiMeditativeAlap,
                    curve: KotoInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(KotoWaypoint {
                    beat: phrase_length_beats * 0.65,
                    tsume_velocity: 0.60,
                    oshi_ite_force_n: max_f * 0.3, // Yuri release
                    hiki_iro_release: 0.35,
                    ji_bridge_offset: 0.0,
                    tuning_schema_idx: 0,
                    behind_bridge_bleed: 0.25,
                    body_wood_resonance: 0.85,
                    master_gain: 0.85,
                    articulation: KotoArticulation::HirajoshiMeditativeAlap,
                    curve: KotoInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(KotoWaypoint {
                    beat: *phrase_length_beats,
                    tsume_velocity: 0.70,
                    oshi_ite_force_n: 1.0,
                    hiki_iro_release: 0.05,
                    ji_bridge_offset: 0.0,
                    tuning_schema_idx: 0,
                    behind_bridge_bleed: 0.25,
                    body_wood_resonance: 0.85,
                    master_gain: 0.88,
                    articulation: KotoArticulation::HirajoshiMeditativeAlap,
                    curve: KotoInterpolationCurve::CubicCatmullRom,
                });
            }

            KotoGesturePattern::KokinJoshiFastMiyako { phrase_length_beats } => {
                self.loop_length_beats = *phrase_length_beats;

                self.waypoints.push(KotoWaypoint {
                    beat: 0.0,
                    tsume_velocity: 0.90,
                    oshi_ite_force_n: 3.0,
                    hiki_iro_release: 0.05,
                    ji_bridge_offset: 0.0,
                    tuning_schema_idx: 1, // Kokin-joshi
                    behind_bridge_bleed: 0.20,
                    body_wood_resonance: 0.80,
                    master_gain: 0.90,
                    articulation: KotoArticulation::KokinJoshiFastMiyako,
                    curve: KotoInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(KotoWaypoint {
                    beat: phrase_length_beats * 0.5,
                    tsume_velocity: 0.95,
                    oshi_ite_force_n: 5.0,
                    hiki_iro_release: 0.10,
                    ji_bridge_offset: 0.0,
                    tuning_schema_idx: 1,
                    behind_bridge_bleed: 0.22,
                    body_wood_resonance: 0.82,
                    master_gain: 0.92,
                    articulation: KotoArticulation::KokinJoshiFastMiyako,
                    curve: KotoInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(KotoWaypoint {
                    beat: *phrase_length_beats,
                    tsume_velocity: 0.90,
                    oshi_ite_force_n: 3.0,
                    hiki_iro_release: 0.05,
                    ji_bridge_offset: 0.0,
                    tuning_schema_idx: 1,
                    behind_bridge_bleed: 0.20,
                    body_wood_resonance: 0.80,
                    master_gain: 0.90,
                    articulation: KotoArticulation::KokinJoshiFastMiyako,
                    curve: KotoInterpolationCurve::CubicCatmullRom,
                });
            }

            KotoGesturePattern::InSenDramaticGendai { phrase_length_beats, max_pitch_bend_n } => {
                self.loop_length_beats = *phrase_length_beats;
                let max_b = *max_pitch_bend_n;

                self.waypoints.push(KotoWaypoint {
                    beat: 0.0,
                    tsume_velocity: 0.85,
                    oshi_ite_force_n: 2.0,
                    hiki_iro_release: 0.10,
                    ji_bridge_offset: 0.0,
                    tuning_schema_idx: 2, // In-sen
                    behind_bridge_bleed: 0.30,
                    body_wood_resonance: 0.90,
                    master_gain: 0.90,
                    articulation: KotoArticulation::InSenDramaticGendai,
                    curve: KotoInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(KotoWaypoint {
                    beat: phrase_length_beats * 0.40,
                    tsume_velocity: 0.96,
                    oshi_ite_force_n: max_b, // Major 3rd bend (~25-30 N)
                    hiki_iro_release: 0.20,
                    ji_bridge_offset: 0.0,
                    tuning_schema_idx: 2,
                    behind_bridge_bleed: 0.35,
                    body_wood_resonance: 0.95,
                    master_gain: 0.95,
                    articulation: KotoArticulation::InSenDramaticGendai,
                    curve: KotoInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(KotoWaypoint {
                    beat: phrase_length_beats * 0.70,
                    tsume_velocity: 0.75,
                    oshi_ite_force_n: max_b * 0.4,
                    hiki_iro_release: 0.40,
                    ji_bridge_offset: 0.0,
                    tuning_schema_idx: 2,
                    behind_bridge_bleed: 0.30,
                    body_wood_resonance: 0.90,
                    master_gain: 0.90,
                    articulation: KotoArticulation::InSenDramaticGendai,
                    curve: KotoInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(KotoWaypoint {
                    beat: *phrase_length_beats,
                    tsume_velocity: 0.85,
                    oshi_ite_force_n: 2.0,
                    hiki_iro_release: 0.10,
                    ji_bridge_offset: 0.0,
                    tuning_schema_idx: 2,
                    behind_bridge_bleed: 0.30,
                    body_wood_resonance: 0.90,
                    master_gain: 0.90,
                    articulation: KotoArticulation::InSenDramaticGendai,
                    curve: KotoInterpolationCurve::CubicCatmullRom,
                });
            }

            KotoGesturePattern::KumoiJoshiSpringRain { phrase_length_beats } => {
                self.loop_length_beats = *phrase_length_beats;

                self.waypoints.push(KotoWaypoint {
                    beat: 0.0,
                    tsume_velocity: 0.65,
                    oshi_ite_force_n: 2.5,
                    hiki_iro_release: 0.10,
                    ji_bridge_offset: 0.0,
                    tuning_schema_idx: 3, // Kumoi-joshi
                    behind_bridge_bleed: 0.22,
                    body_wood_resonance: 0.75,
                    master_gain: 0.85,
                    articulation: KotoArticulation::KumoiJoshiSpringRain,
                    curve: KotoInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(KotoWaypoint {
                    beat: phrase_length_beats * 0.5,
                    tsume_velocity: 0.78,
                    oshi_ite_force_n: 6.0,
                    hiki_iro_release: 0.20,
                    ji_bridge_offset: 0.0,
                    tuning_schema_idx: 3,
                    behind_bridge_bleed: 0.25,
                    body_wood_resonance: 0.78,
                    master_gain: 0.88,
                    articulation: KotoArticulation::KumoiJoshiSpringRain,
                    curve: KotoInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(KotoWaypoint {
                    beat: *phrase_length_beats,
                    tsume_velocity: 0.65,
                    oshi_ite_force_n: 2.5,
                    hiki_iro_release: 0.10,
                    ji_bridge_offset: 0.0,
                    tuning_schema_idx: 3,
                    behind_bridge_bleed: 0.22,
                    body_wood_resonance: 0.75,
                    master_gain: 0.85,
                    articulation: KotoArticulation::KumoiJoshiSpringRain,
                    curve: KotoInterpolationCurve::CubicCatmullRom,
                });
            }

            KotoGesturePattern::RyukyuIslandSwell { phrase_length_beats } => {
                self.loop_length_beats = *phrase_length_beats;

                self.waypoints.push(KotoWaypoint {
                    beat: 0.0,
                    tsume_velocity: 0.80,
                    oshi_ite_force_n: 4.0,
                    hiki_iro_release: 0.08,
                    ji_bridge_offset: 0.0,
                    tuning_schema_idx: 4, // Ryukyu
                    behind_bridge_bleed: 0.20,
                    body_wood_resonance: 0.82,
                    master_gain: 0.88,
                    articulation: KotoArticulation::RyukyuIslandSwell,
                    curve: KotoInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(KotoWaypoint {
                    beat: phrase_length_beats * 0.5,
                    tsume_velocity: 0.92,
                    oshi_ite_force_n: 8.0,
                    hiki_iro_release: 0.15,
                    ji_bridge_offset: 0.0,
                    tuning_schema_idx: 4,
                    behind_bridge_bleed: 0.24,
                    body_wood_resonance: 0.85,
                    master_gain: 0.90,
                    articulation: KotoArticulation::RyukyuIslandSwell,
                    curve: KotoInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(KotoWaypoint {
                    beat: *phrase_length_beats,
                    tsume_velocity: 0.80,
                    oshi_ite_force_n: 4.0,
                    hiki_iro_release: 0.08,
                    ji_bridge_offset: 0.0,
                    tuning_schema_idx: 4,
                    behind_bridge_bleed: 0.20,
                    body_wood_resonance: 0.82,
                    master_gain: 0.88,
                    articulation: KotoArticulation::RyukyuIslandSwell,
                    curve: KotoInterpolationCurve::CubicCatmullRom,
                });
            }

            KotoGesturePattern::GuzhengVirtuosoWaterfall { phrase_length_beats, arpeggio_rate_hz: _ } => {
                self.loop_length_beats = *phrase_length_beats;

                self.waypoints.push(KotoWaypoint {
                    beat: 0.0,
                    tsume_velocity: 0.92,
                    oshi_ite_force_n: 5.0,
                    hiki_iro_release: 0.10,
                    ji_bridge_offset: 0.0,
                    tuning_schema_idx: 5, // Guzheng
                    behind_bridge_bleed: 0.40,
                    body_wood_resonance: 0.95,
                    master_gain: 0.95,
                    articulation: KotoArticulation::GuzhengVirtuosoWaterfall,
                    curve: KotoInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(KotoWaypoint {
                    beat: phrase_length_beats * 0.5,
                    tsume_velocity: 0.98,
                    oshi_ite_force_n: 12.0,
                    hiki_iro_release: 0.18,
                    ji_bridge_offset: 0.0,
                    tuning_schema_idx: 5,
                    behind_bridge_bleed: 0.45,
                    body_wood_resonance: 0.98,
                    master_gain: 0.96,
                    articulation: KotoArticulation::GuzhengVirtuosoWaterfall,
                    curve: KotoInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(KotoWaypoint {
                    beat: *phrase_length_beats,
                    tsume_velocity: 0.92,
                    oshi_ite_force_n: 5.0,
                    hiki_iro_release: 0.10,
                    ji_bridge_offset: 0.0,
                    tuning_schema_idx: 5,
                    behind_bridge_bleed: 0.40,
                    body_wood_resonance: 0.95,
                    master_gain: 0.95,
                    articulation: KotoArticulation::GuzhengVirtuosoWaterfall,
                    curve: KotoInterpolationCurve::CubicCatmullRom,
                });
            }
        }
    }

    /// Evaluate timeline trajectory at arbitrary beat and return interpolated snapshot.
    pub fn evaluate_at_beat(&self, beat: f64) -> KotoBusSnapshot {
        if self.waypoints.is_empty() {
            let (vel, oshi, hiki, offset, tuning, bleed, wood, mgain) =
                KotoArticulation::HirajoshiMeditativeAlap.nominal_parameters();
            return KotoBusSnapshot {
                articulation: KotoArticulation::HirajoshiMeditativeAlap,
                tsume_velocity: vel,
                oshi_ite_force_n: oshi,
                hiki_iro_release: hiki,
                ji_bridge_offset: offset,
                tuning_schema_idx: tuning,
                behind_bridge_bleed: bleed,
                body_wood_resonance: wood,
                master_gain: mgain,
            };
        }

        if self.waypoints.len() == 1 {
            let wp = &self.waypoints[0];
            return KotoBusSnapshot {
                articulation: wp.articulation,
                tsume_velocity: wp.tsume_velocity,
                oshi_ite_force_n: wp.oshi_ite_force_n,
                hiki_iro_release: wp.hiki_iro_release,
                ji_bridge_offset: wp.ji_bridge_offset,
                tuning_schema_idx: wp.tuning_schema_idx,
                behind_bridge_bleed: wp.behind_bridge_bleed,
                body_wood_resonance: wp.body_wood_resonance,
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
            return KotoBusSnapshot {
                articulation: wp.articulation,
                tsume_velocity: wp.tsume_velocity,
                oshi_ite_force_n: wp.oshi_ite_force_n,
                hiki_iro_release: wp.hiki_iro_release,
                ji_bridge_offset: wp.ji_bridge_offset,
                tuning_schema_idx: wp.tuning_schema_idx,
                behind_bridge_bleed: wp.behind_bridge_bleed,
                body_wood_resonance: wp.body_wood_resonance,
                master_gain: wp.master_gain,
            };
        }

        if idx1 >= n {
            let wp = &self.waypoints[n - 1];
            return KotoBusSnapshot {
                articulation: wp.articulation,
                tsume_velocity: wp.tsume_velocity,
                oshi_ite_force_n: wp.oshi_ite_force_n,
                hiki_iro_release: wp.hiki_iro_release,
                ji_bridge_offset: wp.ji_bridge_offset,
                tuning_schema_idx: wp.tuning_schema_idx,
                behind_bridge_bleed: wp.behind_bridge_bleed,
                body_wood_resonance: wp.body_wood_resonance,
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

        let interp = |v0: f32, v1: f32, v2: f32, v3: f32, curve: KotoInterpolationCurve| -> f32 {
            match curve {
                KotoInterpolationCurve::Hold => v1,
                KotoInterpolationCurve::Linear => v1 + t * (v2 - v1),
                KotoInterpolationCurve::Exponential => {
                    let min_v = v1.min(v2).max(1e-5);
                    let ratio = (v2.max(1e-5) / min_v).max(1e-5);
                    v1 * ratio.powf(t)
                }
                KotoInterpolationCurve::CubicBezier => {
                    let t2 = t * t;
                    let t3 = t2 * t;
                    let b = 3.0 * t2 - 2.0 * t3;
                    v1 + b * (v2 - v1)
                }
                KotoInterpolationCurve::CubicCatmullRom => {
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
        let vel = interp(wp0.tsume_velocity, wp1.tsume_velocity, wp2.tsume_velocity, wp3.tsume_velocity, curve).clamp(0.01, 1.0);
        let oshi = interp(wp0.oshi_ite_force_n, wp1.oshi_ite_force_n, wp2.oshi_ite_force_n, wp3.oshi_ite_force_n, curve).clamp(0.0, 30.0);
        let hiki = interp(wp0.hiki_iro_release, wp1.hiki_iro_release, wp2.hiki_iro_release, wp3.hiki_iro_release, curve).clamp(0.0, 1.0);
        let offset = interp(wp0.ji_bridge_offset, wp1.ji_bridge_offset, wp2.ji_bridge_offset, wp3.ji_bridge_offset, curve).clamp(-0.20, 0.20);
        let bleed = interp(wp0.behind_bridge_bleed, wp1.behind_bridge_bleed, wp2.behind_bridge_bleed, wp3.behind_bridge_bleed, curve).clamp(0.0, 1.0);
        let wood = interp(wp0.body_wood_resonance, wp1.body_wood_resonance, wp2.body_wood_resonance, wp3.body_wood_resonance, curve).clamp(0.0, 1.0);
        let mgain = interp(wp0.master_gain, wp1.master_gain, wp2.master_gain, wp3.master_gain, curve).clamp(0.0, 2.0);

        KotoBusSnapshot {
            articulation: if t > 0.5 { wp2.articulation } else { wp1.articulation },
            tsume_velocity: vel,
            oshi_ite_force_n: oshi,
            hiki_iro_release: hiki,
            ji_bridge_offset: offset,
            tuning_schema_idx: if t > 0.5 { wp2.tuning_schema_idx } else { wp1.tuning_schema_idx },
            behind_bridge_bleed: bleed,
            body_wood_resonance: wood,
            master_gain: mgain,
        }
    }

    /// Dispatch interpolated parameters into `KotoBus` and `ParamBus`.
    pub fn dispatch_to_bus(
        &self,
        beat: f64,
        bus: &KotoBus,
        param_bus: Option<&ParamBus>,
        base_id: ParamId,
    ) {
        let snap = self.evaluate_at_beat(beat);

        bus.set_tsume_velocity(snap.tsume_velocity);
        bus.set_oshi_ite_force(snap.oshi_ite_force_n);
        bus.set_hiki_iro_release(snap.hiki_iro_release);
        bus.set_ji_bridge_offset(snap.ji_bridge_offset);
        bus.set_tuning_schema_idx(snap.tuning_schema_idx);
        bus.set_behind_bridge_bleed(snap.behind_bridge_bleed);
        bus.set_body_wood_resonance(snap.body_wood_resonance);
        bus.set_master_gain(snap.master_gain);

        if let Some(pb) = param_bus {
            pb.set(ParamId(base_id.0), snap.articulation as u32 as f32);
            pb.set(ParamId(base_id.0 + 1), snap.tsume_velocity);
            pb.set(ParamId(base_id.0 + 2), snap.oshi_ite_force_n);
            pb.set(ParamId(base_id.0 + 3), snap.hiki_iro_release);
            pb.set(ParamId(base_id.0 + 4), snap.ji_bridge_offset);
            pb.set(ParamId(base_id.0 + 5), snap.tuning_schema_idx as f32);
            pb.set(ParamId(base_id.0 + 6), snap.behind_bridge_bleed);
            pb.set(ParamId(base_id.0 + 7), snap.body_wood_resonance);
            pb.set(ParamId(base_id.0 + 8), snap.master_gain);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_koto_gesture_patterns_and_catmull_rom_evaluation() {
        for pattern in [
            KotoGesturePattern::HirajoshiMeditativeAlap { phrase_length_beats: 4.0, max_oshi_ite_n: 15.0 },
            KotoGesturePattern::KokinJoshiFastMiyako { phrase_length_beats: 4.0 },
            KotoGesturePattern::InSenDramaticGendai { phrase_length_beats: 6.0, max_pitch_bend_n: 25.0 },
            KotoGesturePattern::KumoiJoshiSpringRain { phrase_length_beats: 4.0 },
            KotoGesturePattern::RyukyuIslandSwell { phrase_length_beats: 4.0 },
            KotoGesturePattern::GuzhengVirtuosoWaterfall { phrase_length_beats: 8.0, arpeggio_rate_hz: 12.0 },
        ] {
            let engine = KotoGestureEngine::with_pattern(pattern);
            assert!(!engine.waypoints.is_empty());

            // Evaluate at multiple beat positions
            for beat in [0.0, 0.5, 1.2, 2.0, 3.5, 4.0, 5.8] {
                let snap = engine.evaluate_at_beat(beat);
                assert!(snap.tsume_velocity > 0.0);
                assert!(snap.oshi_ite_force_n >= 0.0);
                assert!(snap.hiki_iro_release >= 0.0);
                assert!(snap.behind_bridge_bleed >= 0.0);
                assert!(snap.body_wood_resonance >= 0.0);
                assert!(snap.master_gain > 0.0);
            }
        }
    }

    #[test]
    fn test_koto_gesture_dispatch_to_bus() {
        let engine = KotoGestureEngine::with_pattern(KotoGesturePattern::InSenDramaticGendai {
            phrase_length_beats: 4.0,
            max_pitch_bend_n: 20.0,
        });
        let bus = KotoBus::new();
        let mut param_bus = ParamBus::new();
        summoner_core::koto_bus::register_koto_params(&mut param_bus, summoner_core::koto_bus::KOTO_BASE_PARAM_ID);

        engine.dispatch_to_bus(0.0, &bus, Some(&param_bus), summoner_core::koto_bus::KOTO_BASE_PARAM_ID);

        let snap = bus.snapshot();
        assert!((snap.tsume_velocity - 0.85).abs() < 1e-3);
    }
}
