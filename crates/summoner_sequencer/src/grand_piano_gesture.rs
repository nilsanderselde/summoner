// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Dynamic Concert Grand Piano Articulation & Pedaling Timeline (Milestone 26).
//!
//! Provides multi-dimensional continuous physical modeling trajectory generation for
//! Grand Piano hammer strike velocities, continuous damper lift (Half-Pedaling),
//! Una Corda soft-pedal shifts, Sostenuto latch bitmasks, bridge coupling bleed,
//! unison detuning, and soundboard decay scaling with Catmull-Rom spline smoothing,
//! TOML serialization, and sample-accurate lock-free parameter dispatch into
//! `GrandPianoBus` and `ParamBus`.
//!
//! Enforces zero heap allocations in real-time execution loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use summoner_core::grand_piano_bus::{
    GrandPianoArticulation, GrandPianoBus, GrandPianoBusSnapshot,
};
use summoner_core::param_bus::{ParamBus, ParamId};

/// Interpolation curve between discrete Grand Piano keyframe waypoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum GrandPianoInterpolationCurve {
    #[default]
    CubicCatmullRom,
    CubicBezier,
    Linear,
    Hold,
    Exponential,
}

/// A single keyframed multi-dimensional Grand Piano articulation waypoint.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GrandPianoWaypoint {
    /// Timeline beat position in quarter note beats.
    pub beat: f64,
    /// Strike velocity $[0.0 ..= 1.0]$.
    pub strike_velocity: f32,
    /// Damper lift position $[0.0 ..= 1.0]$ (Sustain / Half-Pedaling).
    pub damper_lift_pos: f32,
    /// Una Corda soft pedal shift $[0.0 ..= 1.0]$.
    pub una_corda_shift: f32,
    /// Sostenuto latch mask low (keys 21..52).
    pub sostenuto_latch_low: u32,
    /// Sostenuto latch mask mid (keys 53..84).
    pub sostenuto_latch_mid: u32,
    /// Sostenuto latch mask high (keys 85..108).
    pub sostenuto_latch_high: u32,
    /// Bridge sympathetic energy bleed $[0.0 ..= 1.0]$.
    pub bridge_bleed: f32,
    /// Unison detune cents $[0.0 ..= 5.0]$.
    pub unison_detune_cents: f32,
    /// Hammer felt hardness $[0.0 ..= 1.0]$.
    pub hammer_hardness: f32,
    /// String inharmonicity $B$ parameter $[0.00001 ..= 0.001]$.
    pub inharmonicity_b: f32,
    /// Soundboard decay scale $[0.1 ..= 3.0]$.
    pub soundboard_decay: f32,
    /// Master gain $[0.0 ..= 2.0]$.
    pub master_gain: f32,
    /// Articulation technique.
    pub articulation: GrandPianoArticulation,
    /// Interpolation curve leading to this waypoint.
    pub curve: GrandPianoInterpolationCurve,
}

impl Default for GrandPianoWaypoint {
    fn default() -> Self {
        let (vel, damp, una, s_low, s_mid, s_high, bleed, detune, hard, b, decay, gain) =
            GrandPianoArticulation::ConcertRecital.nominal_parameters();

        Self {
            beat: 0.0,
            strike_velocity: vel,
            damper_lift_pos: damp,
            una_corda_shift: una,
            sostenuto_latch_low: s_low,
            sostenuto_latch_mid: s_mid,
            sostenuto_latch_high: s_high,
            bridge_bleed: bleed,
            unison_detune_cents: detune,
            hammer_hardness: hard,
            inharmonicity_b: b,
            soundboard_decay: decay,
            master_gain: gain,
            articulation: GrandPianoArticulation::ConcertRecital,
            curve: GrandPianoInterpolationCurve::CubicCatmullRom,
        }
    }
}

/// Pre-configured algorithmic Grand Piano gesture trajectory patterns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GrandPianoGesturePattern {
    /// Romantic Chopin Nocturne recital with expressive pedaling and dynamic velocity swell.
    ChopinRecitalNocturne {
        phrase_length_beats: f64,
        max_velocity: f32,
    },
    /// Impressionist Debussy passage with heavy Una Corda soft-pedal coloring and shimmer.
    DebussyImpressionistUnaCorda {
        phrase_length_beats: f64,
    },
    /// Dynamic Half-Pedal atmospheric study with continuous damper hovering.
    HalfPedalAtmosphere {
        phrase_length_beats: f64,
        min_damper: f32,
        max_damper: f32,
    },
    /// Contrapuntal Sostenuto study with bass pedaled drone and staccato upper voices.
    SostenutoContrapuntal {
        phrase_length_beats: f64,
    },
    /// Virtuoso Liszt bravura with high strike velocities and crisp prompt attacks.
    LisztVirtuosoBravura {
        phrase_length_beats: f64,
        peak_velocity: f32,
    },
}

/// Dynamic Grand Piano articulation timeline and parameter trajectory engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GrandPianoGestureEngine {
    pub waypoints: Vec<GrandPianoWaypoint>,
    pub loop_length_beats: f64,
    pub is_looping: bool,
}

impl Default for GrandPianoGestureEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl GrandPianoGestureEngine {
    pub fn new() -> Self {
        Self {
            waypoints: Vec::new(),
            loop_length_beats: 4.0,
            is_looping: true,
        }
    }

    pub fn with_pattern(pattern: GrandPianoGesturePattern) -> Self {
        let mut engine = Self::new();
        engine.load_pattern(pattern);
        engine
    }

    pub fn add_waypoint(&mut self, waypoint: GrandPianoWaypoint) {
        self.waypoints.push(waypoint);
        self.sort_waypoints();
    }

    pub fn sort_waypoints(&mut self) {
        self.waypoints.sort_by(|a, b| a.beat.partial_cmp(&b.beat).unwrap_or(std::cmp::Ordering::Equal));
    }

    pub fn clear(&mut self) {
        self.waypoints.clear();
    }

    pub fn load_pattern(&mut self, pattern: GrandPianoGesturePattern) {
        self.clear();

        match &pattern {
            GrandPianoGesturePattern::ChopinRecitalNocturne { phrase_length_beats, max_velocity } => {
                self.loop_length_beats = *phrase_length_beats;

                self.waypoints.push(GrandPianoWaypoint {
                    beat: 0.0,
                    strike_velocity: 0.45,
                    damper_lift_pos: 0.0,
                    una_corda_shift: 0.0,
                    sostenuto_latch_low: 0,
                    sostenuto_latch_mid: 0,
                    sostenuto_latch_high: 0,
                    bridge_bleed: 0.35,
                    unison_detune_cents: 0.75,
                    hammer_hardness: 0.55,
                    inharmonicity_b: 0.00018,
                    soundboard_decay: 1.0,
                    master_gain: 0.85,
                    articulation: GrandPianoArticulation::ConcertRecital,
                    curve: GrandPianoInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(GrandPianoWaypoint {
                    beat: phrase_length_beats * 0.25,
                    strike_velocity: *max_velocity * 0.75,
                    damper_lift_pos: 0.95,
                    una_corda_shift: 0.0,
                    sostenuto_latch_low: 0,
                    sostenuto_latch_mid: 0,
                    sostenuto_latch_high: 0,
                    bridge_bleed: 0.40,
                    unison_detune_cents: 0.80,
                    hammer_hardness: 0.65,
                    inharmonicity_b: 0.00018,
                    soundboard_decay: 1.1,
                    master_gain: 0.88,
                    articulation: GrandPianoArticulation::ConcertRecital,
                    curve: GrandPianoInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(GrandPianoWaypoint {
                    beat: phrase_length_beats * 0.65,
                    strike_velocity: *max_velocity,
                    damper_lift_pos: 1.0,
                    una_corda_shift: 0.0,
                    sostenuto_latch_low: 0,
                    sostenuto_latch_mid: 0,
                    sostenuto_latch_high: 0,
                    bridge_bleed: 0.45,
                    unison_detune_cents: 0.85,
                    hammer_hardness: 0.75,
                    inharmonicity_b: 0.00020,
                    soundboard_decay: 1.2,
                    master_gain: 0.90,
                    articulation: GrandPianoArticulation::ConcertRecital,
                    curve: GrandPianoInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(GrandPianoWaypoint {
                    beat: *phrase_length_beats,
                    strike_velocity: 0.40,
                    damper_lift_pos: 0.0,
                    una_corda_shift: 0.0,
                    sostenuto_latch_low: 0,
                    sostenuto_latch_mid: 0,
                    sostenuto_latch_high: 0,
                    bridge_bleed: 0.35,
                    unison_detune_cents: 0.75,
                    hammer_hardness: 0.50,
                    inharmonicity_b: 0.00018,
                    soundboard_decay: 1.0,
                    master_gain: 0.85,
                    articulation: GrandPianoArticulation::ConcertRecital,
                    curve: GrandPianoInterpolationCurve::CubicCatmullRom,
                });
            }

            GrandPianoGesturePattern::DebussyImpressionistUnaCorda { phrase_length_beats } => {
                self.loop_length_beats = *phrase_length_beats;

                self.waypoints.push(GrandPianoWaypoint {
                    beat: 0.0,
                    strike_velocity: 0.40,
                    damper_lift_pos: 0.70,
                    una_corda_shift: 0.85,
                    sostenuto_latch_low: 0,
                    sostenuto_latch_mid: 0,
                    sostenuto_latch_high: 0,
                    bridge_bleed: 0.50,
                    unison_detune_cents: 1.10,
                    hammer_hardness: 0.30,
                    inharmonicity_b: 0.00015,
                    soundboard_decay: 1.3,
                    master_gain: 0.80,
                    articulation: GrandPianoArticulation::ImpressionistUnaCorda,
                    curve: GrandPianoInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(GrandPianoWaypoint {
                    beat: phrase_length_beats * 0.5,
                    strike_velocity: 0.55,
                    damper_lift_pos: 0.95,
                    una_corda_shift: 1.0,
                    sostenuto_latch_low: 0,
                    sostenuto_latch_mid: 0,
                    sostenuto_latch_high: 0,
                    bridge_bleed: 0.55,
                    unison_detune_cents: 1.25,
                    hammer_hardness: 0.28,
                    inharmonicity_b: 0.00015,
                    soundboard_decay: 1.4,
                    master_gain: 0.82,
                    articulation: GrandPianoArticulation::ImpressionistUnaCorda,
                    curve: GrandPianoInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(GrandPianoWaypoint {
                    beat: *phrase_length_beats,
                    strike_velocity: 0.35,
                    damper_lift_pos: 0.60,
                    una_corda_shift: 0.80,
                    sostenuto_latch_low: 0,
                    sostenuto_latch_mid: 0,
                    sostenuto_latch_high: 0,
                    bridge_bleed: 0.50,
                    unison_detune_cents: 1.10,
                    hammer_hardness: 0.30,
                    inharmonicity_b: 0.00015,
                    soundboard_decay: 1.3,
                    master_gain: 0.80,
                    articulation: GrandPianoArticulation::ImpressionistUnaCorda,
                    curve: GrandPianoInterpolationCurve::CubicCatmullRom,
                });
            }

            GrandPianoGesturePattern::HalfPedalAtmosphere { phrase_length_beats, min_damper, max_damper } => {
                self.loop_length_beats = *phrase_length_beats;

                self.waypoints.push(GrandPianoWaypoint {
                    beat: 0.0,
                    strike_velocity: 0.60,
                    damper_lift_pos: *min_damper,
                    una_corda_shift: 0.0,
                    sostenuto_latch_low: 0,
                    sostenuto_latch_mid: 0,
                    sostenuto_latch_high: 0,
                    bridge_bleed: 0.38,
                    unison_detune_cents: 0.75,
                    hammer_hardness: 0.60,
                    inharmonicity_b: 0.00018,
                    soundboard_decay: 1.0,
                    master_gain: 0.85,
                    articulation: GrandPianoArticulation::HalfPedalSostenutoStudy,
                    curve: GrandPianoInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(GrandPianoWaypoint {
                    beat: phrase_length_beats * 0.5,
                    strike_velocity: 0.65,
                    damper_lift_pos: *max_damper,
                    una_corda_shift: 0.20,
                    sostenuto_latch_low: 0,
                    sostenuto_latch_mid: 0,
                    sostenuto_latch_high: 0,
                    bridge_bleed: 0.42,
                    unison_detune_cents: 0.80,
                    hammer_hardness: 0.58,
                    inharmonicity_b: 0.00018,
                    soundboard_decay: 1.05,
                    master_gain: 0.85,
                    articulation: GrandPianoArticulation::HalfPedalSostenutoStudy,
                    curve: GrandPianoInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(GrandPianoWaypoint {
                    beat: *phrase_length_beats,
                    strike_velocity: 0.58,
                    damper_lift_pos: *min_damper,
                    una_corda_shift: 0.0,
                    sostenuto_latch_low: 0,
                    sostenuto_latch_mid: 0,
                    sostenuto_latch_high: 0,
                    bridge_bleed: 0.38,
                    unison_detune_cents: 0.75,
                    hammer_hardness: 0.60,
                    inharmonicity_b: 0.00018,
                    soundboard_decay: 1.0,
                    master_gain: 0.85,
                    articulation: GrandPianoArticulation::HalfPedalSostenutoStudy,
                    curve: GrandPianoInterpolationCurve::CubicCatmullRom,
                });
            }

            GrandPianoGesturePattern::SostenutoContrapuntal { phrase_length_beats } => {
                self.loop_length_beats = *phrase_length_beats;

                self.waypoints.push(GrandPianoWaypoint {
                    beat: 0.0,
                    strike_velocity: 0.70,
                    damper_lift_pos: 0.0,
                    una_corda_shift: 0.0,
                    sostenuto_latch_low: 0b0000_1111, // Latch low drone notes
                    sostenuto_latch_mid: 0,
                    sostenuto_latch_high: 0,
                    bridge_bleed: 0.40,
                    unison_detune_cents: 0.75,
                    hammer_hardness: 0.65,
                    inharmonicity_b: 0.00018,
                    soundboard_decay: 1.1,
                    master_gain: 0.86,
                    articulation: GrandPianoArticulation::HalfPedalSostenutoStudy,
                    curve: GrandPianoInterpolationCurve::Hold,
                });

                self.waypoints.push(GrandPianoWaypoint {
                    beat: *phrase_length_beats,
                    strike_velocity: 0.70,
                    damper_lift_pos: 0.0,
                    una_corda_shift: 0.0,
                    sostenuto_latch_low: 0b0000_1111,
                    sostenuto_latch_mid: 0,
                    sostenuto_latch_high: 0,
                    bridge_bleed: 0.40,
                    unison_detune_cents: 0.75,
                    hammer_hardness: 0.65,
                    inharmonicity_b: 0.00018,
                    soundboard_decay: 1.1,
                    master_gain: 0.86,
                    articulation: GrandPianoArticulation::HalfPedalSostenutoStudy,
                    curve: GrandPianoInterpolationCurve::Hold,
                });
            }

            GrandPianoGesturePattern::LisztVirtuosoBravura { phrase_length_beats, peak_velocity } => {
                self.loop_length_beats = *phrase_length_beats;

                self.waypoints.push(GrandPianoWaypoint {
                    beat: 0.0,
                    strike_velocity: 0.70,
                    damper_lift_pos: 0.30,
                    una_corda_shift: 0.0,
                    sostenuto_latch_low: 0,
                    sostenuto_latch_mid: 0,
                    sostenuto_latch_high: 0,
                    bridge_bleed: 0.30,
                    unison_detune_cents: 0.85,
                    hammer_hardness: 0.75,
                    inharmonicity_b: 0.00022,
                    soundboard_decay: 0.90,
                    master_gain: 0.88,
                    articulation: GrandPianoArticulation::YamahaPopTreble,
                    curve: GrandPianoInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(GrandPianoWaypoint {
                    beat: phrase_length_beats * 0.5,
                    strike_velocity: *peak_velocity,
                    damper_lift_pos: 0.85,
                    una_corda_shift: 0.0,
                    sostenuto_latch_low: 0,
                    sostenuto_latch_mid: 0,
                    sostenuto_latch_high: 0,
                    bridge_bleed: 0.35,
                    unison_detune_cents: 0.95,
                    hammer_hardness: 0.85,
                    inharmonicity_b: 0.00025,
                    soundboard_decay: 0.95,
                    master_gain: 0.92,
                    articulation: GrandPianoArticulation::YamahaPopTreble,
                    curve: GrandPianoInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(GrandPianoWaypoint {
                    beat: *phrase_length_beats,
                    strike_velocity: 0.72,
                    damper_lift_pos: 0.20,
                    una_corda_shift: 0.0,
                    sostenuto_latch_low: 0,
                    sostenuto_latch_mid: 0,
                    sostenuto_latch_high: 0,
                    bridge_bleed: 0.30,
                    unison_detune_cents: 0.85,
                    hammer_hardness: 0.75,
                    inharmonicity_b: 0.00022,
                    soundboard_decay: 0.90,
                    master_gain: 0.88,
                    articulation: GrandPianoArticulation::YamahaPopTreble,
                    curve: GrandPianoInterpolationCurve::CubicCatmullRom,
                });
            }
        }
    }

    /// Evaluate timeline trajectory at arbitrary beat and return interpolated snapshot.
    pub fn evaluate_at_beat(&self, beat: f64) -> GrandPianoBusSnapshot {
        if self.waypoints.is_empty() {
            let (vel, damp, una, s_low, s_mid, s_high, bleed, detune, hard, b, decay, gain) =
                GrandPianoArticulation::ConcertRecital.nominal_parameters();
            return GrandPianoBusSnapshot {
                articulation: GrandPianoArticulation::ConcertRecital,
                strike_velocity: vel,
                damper_lift_pos: damp,
                una_corda_shift: una,
                sostenuto_latch_low: s_low,
                sostenuto_latch_mid: s_mid,
                sostenuto_latch_high: s_high,
                bridge_bleed: bleed,
                unison_detune_cents: detune,
                hammer_hardness: hard,
                inharmonicity_b: b,
                soundboard_decay: decay,
                master_gain: gain,
            };
        }

        if self.waypoints.len() == 1 {
            let wp = &self.waypoints[0];
            return GrandPianoBusSnapshot {
                articulation: wp.articulation,
                strike_velocity: wp.strike_velocity,
                damper_lift_pos: wp.damper_lift_pos,
                una_corda_shift: wp.una_corda_shift,
                sostenuto_latch_low: wp.sostenuto_latch_low,
                sostenuto_latch_mid: wp.sostenuto_latch_mid,
                sostenuto_latch_high: wp.sostenuto_latch_high,
                bridge_bleed: wp.bridge_bleed,
                unison_detune_cents: wp.unison_detune_cents,
                hammer_hardness: wp.hammer_hardness,
                inharmonicity_b: wp.inharmonicity_b,
                soundboard_decay: wp.soundboard_decay,
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
            return GrandPianoBusSnapshot {
                articulation: wp.articulation,
                strike_velocity: wp.strike_velocity,
                damper_lift_pos: wp.damper_lift_pos,
                una_corda_shift: wp.una_corda_shift,
                sostenuto_latch_low: wp.sostenuto_latch_low,
                sostenuto_latch_mid: wp.sostenuto_latch_mid,
                sostenuto_latch_high: wp.sostenuto_latch_high,
                bridge_bleed: wp.bridge_bleed,
                unison_detune_cents: wp.unison_detune_cents,
                hammer_hardness: wp.hammer_hardness,
                inharmonicity_b: wp.inharmonicity_b,
                soundboard_decay: wp.soundboard_decay,
                master_gain: wp.master_gain,
            };
        }

        if idx1 >= n {
            let wp = &self.waypoints[n - 1];
            return GrandPianoBusSnapshot {
                articulation: wp.articulation,
                strike_velocity: wp.strike_velocity,
                damper_lift_pos: wp.damper_lift_pos,
                una_corda_shift: wp.una_corda_shift,
                sostenuto_latch_low: wp.sostenuto_latch_low,
                sostenuto_latch_mid: wp.sostenuto_latch_mid,
                sostenuto_latch_high: wp.sostenuto_latch_high,
                bridge_bleed: wp.bridge_bleed,
                unison_detune_cents: wp.unison_detune_cents,
                hammer_hardness: wp.hammer_hardness,
                inharmonicity_b: wp.inharmonicity_b,
                soundboard_decay: wp.soundboard_decay,
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

        let interp = |v0: f32, v1: f32, v2: f32, v3: f32, curve: GrandPianoInterpolationCurve| -> f32 {
            match curve {
                GrandPianoInterpolationCurve::Hold => v1,
                GrandPianoInterpolationCurve::Linear => v1 + t * (v2 - v1),
                GrandPianoInterpolationCurve::Exponential => {
                    let min_v = v1.min(v2).max(1e-5);
                    let ratio = (v2.max(1e-5) / min_v).max(1e-5);
                    v1 * ratio.powf(t)
                }
                GrandPianoInterpolationCurve::CubicBezier => {
                    let t2 = t * t;
                    let t3 = t2 * t;
                    let b = 3.0 * t2 - 2.0 * t3;
                    v1 + b * (v2 - v1)
                }
                GrandPianoInterpolationCurve::CubicCatmullRom => {
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
        let strike_vel = interp(wp0.strike_velocity, wp1.strike_velocity, wp2.strike_velocity, wp3.strike_velocity, curve).clamp(0.0, 1.0);
        let damper = interp(wp0.damper_lift_pos, wp1.damper_lift_pos, wp2.damper_lift_pos, wp3.damper_lift_pos, curve).clamp(0.0, 1.0);
        let una = interp(wp0.una_corda_shift, wp1.una_corda_shift, wp2.una_corda_shift, wp3.una_corda_shift, curve).clamp(0.0, 1.0);
        let bleed = interp(wp0.bridge_bleed, wp1.bridge_bleed, wp2.bridge_bleed, wp3.bridge_bleed, curve).clamp(0.0, 1.0);
        let detune = interp(wp0.unison_detune_cents, wp1.unison_detune_cents, wp2.unison_detune_cents, wp3.unison_detune_cents, curve).clamp(0.0, 5.0);
        let hard = interp(wp0.hammer_hardness, wp1.hammer_hardness, wp2.hammer_hardness, wp3.hammer_hardness, curve).clamp(0.0, 1.0);
        let b = interp(wp0.inharmonicity_b, wp1.inharmonicity_b, wp2.inharmonicity_b, wp3.inharmonicity_b, curve).clamp(0.00001, 0.001);
        let decay = interp(wp0.soundboard_decay, wp1.soundboard_decay, wp2.soundboard_decay, wp3.soundboard_decay, curve).clamp(0.1, 3.0);
        let gain = interp(wp0.master_gain, wp1.master_gain, wp2.master_gain, wp3.master_gain, curve).clamp(0.0, 2.0);

        GrandPianoBusSnapshot {
            articulation: if t > 0.5 { wp2.articulation } else { wp1.articulation },
            strike_velocity: strike_vel,
            damper_lift_pos: damper,
            una_corda_shift: una,
            sostenuto_latch_low: if t > 0.5 { wp2.sostenuto_latch_low } else { wp1.sostenuto_latch_low },
            sostenuto_latch_mid: if t > 0.5 { wp2.sostenuto_latch_mid } else { wp1.sostenuto_latch_mid },
            sostenuto_latch_high: if t > 0.5 { wp2.sostenuto_latch_high } else { wp1.sostenuto_latch_high },
            bridge_bleed: bleed,
            unison_detune_cents: detune,
            hammer_hardness: hard,
            inharmonicity_b: b,
            soundboard_decay: decay,
            master_gain: gain,
        }
    }

    /// Dispatch interpolated parameters into `GrandPianoBus` and `ParamBus`.
    pub fn dispatch_to_bus(
        &self,
        beat: f64,
        bus: &GrandPianoBus,
        param_bus: Option<&ParamBus>,
        base_id: ParamId,
    ) {
        let snap = self.evaluate_at_beat(beat);

        bus.set_strike_velocity(snap.strike_velocity);
        bus.set_damper_lift(snap.damper_lift_pos);
        bus.set_una_corda_shift(snap.una_corda_shift);
        bus.set_sostenuto_latch(snap.sostenuto_latch_low, snap.sostenuto_latch_mid, snap.sostenuto_latch_high);
        bus.set_bridge_bleed(snap.bridge_bleed);
        bus.set_unison_detune_cents(snap.unison_detune_cents);
        bus.set_hammer_hardness(snap.hammer_hardness);
        bus.set_inharmonicity_b(snap.inharmonicity_b);
        bus.set_soundboard_decay(snap.soundboard_decay);
        bus.set_master_gain(snap.master_gain);

        if let Some(pb) = param_bus {
            pb.set(ParamId(base_id.0), snap.articulation as u32 as f32);
            pb.set(ParamId(base_id.0 + 1), snap.strike_velocity);
            pb.set(ParamId(base_id.0 + 2), snap.damper_lift_pos);
            pb.set(ParamId(base_id.0 + 3), snap.una_corda_shift);
            pb.set(ParamId(base_id.0 + 4), snap.sostenuto_latch_low as f32);
            pb.set(ParamId(base_id.0 + 5), snap.sostenuto_latch_mid as f32);
            pb.set(ParamId(base_id.0 + 6), snap.sostenuto_latch_high as f32);
            pb.set(ParamId(base_id.0 + 7), snap.bridge_bleed);
            pb.set(ParamId(base_id.0 + 8), snap.unison_detune_cents);
            pb.set(ParamId(base_id.0 + 9), snap.hammer_hardness);
            pb.set(ParamId(base_id.0 + 10), snap.inharmonicity_b);
            pb.set(ParamId(base_id.0 + 11), snap.soundboard_decay);
            pb.set(ParamId(base_id.0 + 12), snap.master_gain);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grand_piano_gesture_engine_chopin_evaluation() {
        let engine = GrandPianoGestureEngine::with_pattern(
            GrandPianoGesturePattern::ChopinRecitalNocturne {
                phrase_length_beats: 8.0,
                max_velocity: 0.90,
            },
        );

        let snap_start = engine.evaluate_at_beat(0.0);
        assert_eq!(snap_start.articulation, GrandPianoArticulation::ConcertRecital);
        assert!((snap_start.strike_velocity - 0.45).abs() < 1e-3);

        let snap_mid = engine.evaluate_at_beat(4.0);
        assert!(snap_mid.strike_velocity > 0.60);
        assert!(snap_mid.damper_lift_pos > 0.80);
    }

    #[test]
    fn test_grand_piano_gesture_engine_toml_roundtrip() {
        let engine = GrandPianoGestureEngine::with_pattern(
            GrandPianoGesturePattern::DebussyImpressionistUnaCorda {
                phrase_length_beats: 4.0,
            },
        );

        let toml_str = toml::to_string(&engine).expect("Failed to serialize GrandPianoGestureEngine");
        let deserialized: GrandPianoGestureEngine =
            toml::from_str(&toml_str).expect("Failed to deserialize GrandPianoGestureEngine");
        assert_eq!(engine.waypoints.len(), deserialized.waypoints.len());
    }
}
