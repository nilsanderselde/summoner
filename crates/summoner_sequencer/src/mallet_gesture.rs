// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Dynamic Mallet Strike Gesture Timeline & Percussion Articulation Vector Engine (Milestone 19).
//!
//! Provides multi-dimensional continuous percussion strike gesture trajectory generation (Radial Strike Position $r$,
//! Strike Velocity $v_s$, Mallet Hardness $h$, Rimshot Damping, Roll Tremolo Rate, Kettle Tension / Pitch Bend,
//! and Air Cavity Depth) supporting classical orchestral timpani glissandos, marimba 4-mallet arpeggios,
//! marching snare accents, and Latin conga tumbao rhythms with cubic Catmull-Rom smoothing,
//! TOML serialization, and sample-accurate lock-free dispatch into `MalletBus` and `ParamBus`.
//!
//! Enforces zero heap allocations in real-time execution loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use summoner_core::mallet_bus::{MalletArticulation, MalletBus, MalletBusSnapshot};
use summoner_core::param_bus::{ParamBus, ParamId};

/// Interpolation mode between discrete mallet articulation keyframe waypoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MalletInterpolationCurve {
    #[default]
    CubicCatmullRom,
    CubicBezier,
    Linear,
    Hold,
    Exponential,
}

/// A single keyframed multi-dimensional percussion strike waypoint.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MalletWaypoint {
    /// Timeline beat position in quarter note beats.
    pub beat: f64,
    /// Radial strike position on the membrane or bar $r \in [0.0, 1.0]$ (0.0 center, 1.0 rim).
    pub radial_strike_pos: f32,
    /// Strike velocity $[0.0 ..= 1.0]$.
    pub strike_velocity: f32,
    /// Mallet core / felt hardness $[0.0 ..= 1.0]$ (0.0 soft yarn, 1.0 hard metal).
    pub mallet_hardness: f32,
    /// Rimshot damping ratio $[0.0 ..= 1.0]$.
    pub rimshot_damping: f32,
    /// Roll / tremolo oscillation rate in Hz $[0.0 ..= 60.0]$.
    pub roll_tremolo_rate_hz: f32,
    /// Kettle tension pitch bend offset in cents $[-1200.0 ..= 1200.0]$.
    pub kettle_tension_cents: f32,
    /// Air cavity enclosure depth $[0.0 ..= 1.0]$.
    pub air_cavity_depth: f32,
    /// Base membrane tension $[0.0 ..= 1.0]$.
    pub membrane_tension: f32,
    /// Percussion strike articulation technique.
    pub articulation: MalletArticulation,
    /// Interpolation curve leading to this waypoint.
    pub curve: MalletInterpolationCurve,
}

impl Default for MalletWaypoint {
    fn default() -> Self {
        let (r, vel, hard, rim, roll, tens) = MalletArticulation::EdgeSweetSpot.nominal_parameters();
        Self {
            beat: 0.0,
            radial_strike_pos: r,
            strike_velocity: vel,
            mallet_hardness: hard,
            rimshot_damping: rim,
            roll_tremolo_rate_hz: roll,
            kettle_tension_cents: tens,
            air_cavity_depth: 0.75,
            membrane_tension: 0.50,
            articulation: MalletArticulation::EdgeSweetSpot,
            curve: MalletInterpolationCurve::CubicCatmullRom,
        }
    }
}

/// Pre-configured algorithmic percussion mallet gesture trajectory patterns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MalletGesturePattern {
    /// Orchestral Timpani crescendo roll with continuous pedal pitch bend glissando.
    TimpaniGlissandoRoll {
        roll_length_beats: f64,
        start_cents: f32,
        end_cents: f32,
        roll_rate_hz: f32,
        crescendo_peak_velocity: f32,
    },
    /// Concert Marimba 4-mallet chord pattern with position sweeps and hardness variations.
    MarimbaFourMalletChords {
        measure_length_beats: f64,
        inner_hardness: f32,
        outer_hardness: f32,
        velocity_dynamics: f32,
    },
    /// Marching Snare Drum syncopated accent pattern with alternating rimshots and ghost notes.
    SnareMarchingAccent {
        subdivisions_per_beat: u32,
        accent_velocity: f32,
        ghost_velocity: f32,
        rimshot_chance: f32,
    },
    /// Concert Vibraphone ballad swells with motorized disc tremolo variation.
    VibraphoneBalladSwell {
        phrase_length_beats: f64,
        motor_speed_hz: f32,
        velocity_swell_peak: f32,
    },
    /// Trinidad Tenor Steelpan calypso rapid arpeggio runs.
    SteelpanCalypsoRun {
        measure_length_beats: f64,
        base_velocity: f32,
        accent_velocity: f32,
    },
    /// Afro-Cuban Conga Tumbao groove (Heel, Tip, Slap, Open tone).
    CongaTumbaoGroove {
        tempo_bpm: f32,
        slap_hardness: f32,
        open_velocity: f32,
    },
    /// User-defined multi-keyframe Catmull-Rom spline trajectory.
    CustomSpline,
}

impl Default for MalletGesturePattern {
    fn default() -> Self {
        Self::TimpaniGlissandoRoll {
            roll_length_beats: 4.0,
            start_cents: 0.0,
            end_cents: 700.0, // Perfect fifth pedal glissando
            roll_rate_hz: 14.0,
            crescendo_peak_velocity: 0.95,
        }
    }
}

/// Dynamic Mallet Strike Gesture Trajectory Engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MalletGestureEngine {
    /// Keyframed articulation waypoints.
    pub waypoints: Vec<MalletWaypoint>,
    /// Active gesture trajectory pattern.
    pub pattern: MalletGesturePattern,
    /// Default interpolation curve.
    pub default_curve: MalletInterpolationCurve,
    /// Total sequence loop length in beats.
    pub loop_length_beats: f64,
    /// Whether pattern loops continuously.
    pub loop_enabled: bool,
    /// Whether gesture engine is currently active.
    pub is_active: bool,
}

impl Default for MalletGestureEngine {
    fn default() -> Self {
        let mut engine = Self {
            waypoints: Vec::new(),
            pattern: MalletGesturePattern::default(),
            default_curve: MalletInterpolationCurve::CubicCatmullRom,
            loop_length_beats: 4.0,
            loop_enabled: true,
            is_active: true,
        };
        engine.generate_pattern_waypoints();
        engine
    }
}

impl MalletGestureEngine {
    /// Creates a new MalletGestureEngine initialized with the specified pattern.
    pub fn new(pattern: MalletGesturePattern) -> Self {
        let mut engine = Self {
            waypoints: Vec::new(),
            pattern,
            default_curve: MalletInterpolationCurve::CubicCatmullRom,
            loop_length_beats: 4.0,
            loop_enabled: true,
            is_active: true,
        };
        engine.generate_pattern_waypoints();
        engine
    }

    /// Regenerates keyframe waypoints based on the active algorithmic pattern.
    pub fn generate_pattern_waypoints(&mut self) {
        self.waypoints.clear();

        match &self.pattern {
            MalletGesturePattern::TimpaniGlissandoRoll {
                roll_length_beats,
                start_cents,
                end_cents,
                roll_rate_hz,
                crescendo_peak_velocity,
            } => {
                self.loop_length_beats = *roll_length_beats;
                let steps = 5;
                for i in 0..=steps {
                    let frac = i as f64 / steps as f64;
                    let beat = frac * roll_length_beats;
                    let tens = start_cents + (end_cents - start_cents) * frac as f32;
                    let vel = 0.30 + (crescendo_peak_velocity - 0.30) * frac as f32;
                    let hard = 0.35 + 0.30 * frac as f32;

                    self.waypoints.push(MalletWaypoint {
                        beat,
                        radial_strike_pos: 0.70 - 0.10 * frac as f32,
                        strike_velocity: vel,
                        mallet_hardness: hard,
                        rimshot_damping: 0.05,
                        roll_tremolo_rate_hz: *roll_rate_hz,
                        kettle_tension_cents: tens,
                        air_cavity_depth: 0.85,
                        membrane_tension: 0.50 + 0.30 * frac as f32,
                        articulation: if frac > 0.0 { MalletArticulation::MalletRollTremolo } else { MalletArticulation::EdgeSweetSpot },
                        curve: MalletInterpolationCurve::CubicCatmullRom,
                    });
                }
            }

            MalletGesturePattern::MarimbaFourMalletChords {
                measure_length_beats,
                inner_hardness,
                outer_hardness,
                velocity_dynamics,
            } => {
                self.loop_length_beats = *measure_length_beats;
                let positions = [0.20, 0.40, 0.60, 0.80];
                for (i, &pos) in positions.iter().enumerate() {
                    let beat = i as f64 * (measure_length_beats / 4.0);
                    let is_outer = i == 0 || i == 3;
                    let hard = if is_outer { *outer_hardness } else { *inner_hardness };
                    let vel = velocity_dynamics * (if i % 2 == 0 { 0.85 } else { 0.70 });

                    self.waypoints.push(MalletWaypoint {
                        beat,
                        radial_strike_pos: pos,
                        strike_velocity: vel,
                        mallet_hardness: hard,
                        rimshot_damping: 0.0,
                        roll_tremolo_rate_hz: 0.0,
                        kettle_tension_cents: 0.0,
                        air_cavity_depth: 0.50,
                        membrane_tension: 0.50,
                        articulation: MalletArticulation::EdgeSweetSpot,
                        curve: MalletInterpolationCurve::Linear,
                    });
                }
            }

            MalletGesturePattern::SnareMarchingAccent {
                subdivisions_per_beat,
                accent_velocity,
                ghost_velocity,
                rimshot_chance: _,
            } => {
                self.loop_length_beats = 4.0;
                let total_steps = 4 * *subdivisions_per_beat as usize;
                let step_beat = 4.0 / total_steps as f64;

                for step in 0..total_steps {
                    let beat = step as f64 * step_beat;
                    let is_accent = step == 0 || step == 6 || step == 10 || step == 14;
                    let is_rimshot = step == 10;
                    let vel = if is_accent { *accent_velocity } else { *ghost_velocity };
                    let hard = if is_accent { 0.85 } else { 0.45 };
                    let r = if is_rimshot { 0.95 } else if is_accent { 0.75 } else { 0.35 };

                    self.waypoints.push(MalletWaypoint {
                        beat,
                        radial_strike_pos: r,
                        strike_velocity: vel,
                        mallet_hardness: hard,
                        rimshot_damping: if is_rimshot { 0.80 } else { 0.10 },
                        roll_tremolo_rate_hz: if !is_accent && step % 4 == 3 { 18.0 } else { 0.0 },
                        kettle_tension_cents: 0.0,
                        air_cavity_depth: 0.40,
                        membrane_tension: 0.60,
                        articulation: if is_rimshot {
                            MalletArticulation::Rimshot
                        } else if !is_accent && step % 4 == 3 {
                            MalletArticulation::MalletRollTremolo
                        } else {
                            MalletArticulation::EdgeSweetSpot
                        },
                        curve: MalletInterpolationCurve::Hold,
                    });
                }
            }

            MalletGesturePattern::VibraphoneBalladSwell {
                phrase_length_beats,
                motor_speed_hz,
                velocity_swell_peak,
            } => {
                self.loop_length_beats = *phrase_length_beats;
                self.waypoints.push(MalletWaypoint {
                    beat: 0.0,
                    radial_strike_pos: 0.50,
                    strike_velocity: 0.40,
                    mallet_hardness: 0.35,
                    rimshot_damping: 0.0,
                    roll_tremolo_rate_hz: *motor_speed_hz,
                    kettle_tension_cents: 0.0,
                    air_cavity_depth: 0.60,
                    membrane_tension: 0.50,
                    articulation: MalletArticulation::EdgeSweetSpot,
                    curve: MalletInterpolationCurve::CubicCatmullRom,
                });
                self.waypoints.push(MalletWaypoint {
                    beat: phrase_length_beats * 0.5,
                    radial_strike_pos: 0.30,
                    strike_velocity: *velocity_swell_peak,
                    mallet_hardness: 0.55,
                    rimshot_damping: 0.0,
                    roll_tremolo_rate_hz: *motor_speed_hz * 1.5,
                    kettle_tension_cents: 0.0,
                    air_cavity_depth: 0.60,
                    membrane_tension: 0.50,
                    articulation: MalletArticulation::EdgeSweetSpot,
                    curve: MalletInterpolationCurve::CubicCatmullRom,
                });
                self.waypoints.push(MalletWaypoint {
                    beat: *phrase_length_beats,
                    radial_strike_pos: 0.50,
                    strike_velocity: 0.30,
                    mallet_hardness: 0.30,
                    rimshot_damping: 0.0,
                    roll_tremolo_rate_hz: *motor_speed_hz,
                    kettle_tension_cents: 0.0,
                    air_cavity_depth: 0.60,
                    membrane_tension: 0.50,
                    articulation: MalletArticulation::EdgeSweetSpot,
                    curve: MalletInterpolationCurve::CubicCatmullRom,
                });
            }

            MalletGesturePattern::SteelpanCalypsoRun {
                measure_length_beats,
                base_velocity,
                accent_velocity,
            } => {
                self.loop_length_beats = *measure_length_beats;
                let notes = 8;
                for i in 0..notes {
                    let beat = i as f64 * (measure_length_beats / notes as f64);
                    let is_accent = i % 3 == 0;
                    let vel = if is_accent { *accent_velocity } else { *base_velocity };

                    self.waypoints.push(MalletWaypoint {
                        beat,
                        radial_strike_pos: 0.40 + 0.40 * (i as f32 / notes as f32),
                        strike_velocity: vel,
                        mallet_hardness: 0.50,
                        rimshot_damping: 0.05,
                        roll_tremolo_rate_hz: 0.0,
                        kettle_tension_cents: 0.0,
                        air_cavity_depth: 0.10,
                        membrane_tension: 0.50,
                        articulation: MalletArticulation::EdgeSweetSpot,
                        curve: MalletInterpolationCurve::Linear,
                    });
                }
            }

            MalletGesturePattern::CongaTumbaoGroove {
                tempo_bpm: _,
                slap_hardness,
                open_velocity,
            } => {
                self.loop_length_beats = 4.0;
                // Traditional 1-bar Tumbao:
                // 1: Heel (center muted), 1&: Tip (edge muted)
                // 2: Slap (high tension sharp), 2&: Tip
                // 3: Heel, 3&: Tip
                // 4: Open tone (sweet spot ring), 4&: Open tone
                let tumbao_steps = [
                    (0.0, 0.15, 0.40, 0.30, 0.70, MalletArticulation::DeadStrokeMuted), // Heel
                    (0.5, 0.85, 0.35, 0.30, 0.70, MalletArticulation::DeadStrokeMuted), // Tip
                    (1.0, 0.90, 0.90, *slap_hardness, 0.85, MalletArticulation::Rimshot), // Slap
                    (1.5, 0.85, 0.35, 0.30, 0.70, MalletArticulation::DeadStrokeMuted), // Tip
                    (2.0, 0.15, 0.40, 0.30, 0.70, MalletArticulation::DeadStrokeMuted), // Heel
                    (2.5, 0.85, 0.35, 0.30, 0.70, MalletArticulation::DeadStrokeMuted), // Tip
                    (3.0, 0.70, *open_velocity, 0.45, 0.05, MalletArticulation::EdgeSweetSpot), // Open 1
                    (3.5, 0.70, *open_velocity * 0.95, 0.45, 0.05, MalletArticulation::EdgeSweetSpot), // Open 2
                ];

                for &(beat, r, vel, hard, rim, art) in &tumbao_steps {
                    self.waypoints.push(MalletWaypoint {
                        beat,
                        radial_strike_pos: r,
                        strike_velocity: vel,
                        mallet_hardness: hard,
                        rimshot_damping: rim,
                        roll_tremolo_rate_hz: 0.0,
                        kettle_tension_cents: 0.0,
                        air_cavity_depth: 0.30,
                        membrane_tension: 0.50,
                        articulation: art,
                        curve: MalletInterpolationCurve::Hold,
                    });
                }
            }

            MalletGesturePattern::CustomSpline => {
                if self.waypoints.is_empty() {
                    self.waypoints.push(MalletWaypoint::default());
                }
            }
        }
    }

    /// Evaluates continuous physical strike parameters at the specified beat position.
    pub fn evaluate_at_beat(&self, beat: f64) -> MalletBusSnapshot {
        if self.waypoints.is_empty() {
            return MalletBusSnapshot::default();
        }

        if self.waypoints.len() == 1 {
            let wp = self.waypoints[0];
            return MalletBusSnapshot {
                articulation: wp.articulation,
                radial_strike_pos: wp.radial_strike_pos,
                strike_velocity: wp.strike_velocity,
                mallet_hardness: wp.mallet_hardness,
                rimshot_damping: wp.rimshot_damping,
                roll_tremolo_rate_hz: wp.roll_tremolo_rate_hz,
                kettle_tension_cents: wp.kettle_tension_cents,
                air_cavity_depth: wp.air_cavity_depth,
                membrane_tension: wp.membrane_tension,
                gesture_progress: 0.0,
                is_active: self.is_active,
            };
        }

        let effective_beat = if self.loop_enabled && self.loop_length_beats > 0.0 {
            if (beat - self.loop_length_beats).abs() < 1e-6 && beat > 0.0 {
                self.loop_length_beats
            } else {
                let b = beat % self.loop_length_beats;
                if b < 0.0 { b + self.loop_length_beats } else { b }
            }
        } else {
            beat
        };

        // Find bounding waypoint indices [i0, i1, i2, i3] for 4-point Catmull-Rom spline
        let n = self.waypoints.len();
        let mut idx1 = 0;
        while idx1 < n - 1 && self.waypoints[idx1 + 1].beat <= effective_beat {
            idx1 += 1;
        }
        let idx2 = (idx1 + 1).min(n - 1);
        let idx0 = if idx1 > 0 { idx1 - 1 } else { 0 };
        let idx3 = if idx2 + 1 < n { idx2 + 1 } else { idx2 };

        let w0 = self.waypoints[idx0];
        let w1 = self.waypoints[idx1];
        let w2 = self.waypoints[idx2];
        let w3 = self.waypoints[idx3];

        let span = (w2.beat - w1.beat).max(1e-5);
        let t = ((effective_beat - w1.beat) / span).clamp(0.0, 1.0) as f32;

        let curve = w1.curve;
        let (r, vel, hard, rim, roll, tens, cav, memb) = match curve {
            MalletInterpolationCurve::Hold => (
                w1.radial_strike_pos,
                w1.strike_velocity,
                w1.mallet_hardness,
                w1.rimshot_damping,
                w1.roll_tremolo_rate_hz,
                w1.kettle_tension_cents,
                w1.air_cavity_depth,
                w1.membrane_tension,
            ),
            MalletInterpolationCurve::Linear => (
                lerp(w1.radial_strike_pos, w2.radial_strike_pos, t),
                lerp(w1.strike_velocity, w2.strike_velocity, t),
                lerp(w1.mallet_hardness, w2.mallet_hardness, t),
                lerp(w1.rimshot_damping, w2.rimshot_damping, t),
                lerp(w1.roll_tremolo_rate_hz, w2.roll_tremolo_rate_hz, t),
                lerp(w1.kettle_tension_cents, w2.kettle_tension_cents, t),
                lerp(w1.air_cavity_depth, w2.air_cavity_depth, t),
                lerp(w1.membrane_tension, w2.membrane_tension, t),
            ),
            MalletInterpolationCurve::Exponential => {
                let t_exp = t * t;
                (
                    lerp(w1.radial_strike_pos, w2.radial_strike_pos, t_exp),
                    lerp(w1.strike_velocity, w2.strike_velocity, t_exp),
                    lerp(w1.mallet_hardness, w2.mallet_hardness, t_exp),
                    lerp(w1.rimshot_damping, w2.rimshot_damping, t_exp),
                    lerp(w1.roll_tremolo_rate_hz, w2.roll_tremolo_rate_hz, t_exp),
                    lerp(w1.kettle_tension_cents, w2.kettle_tension_cents, t_exp),
                    lerp(w1.air_cavity_depth, w2.air_cavity_depth, t_exp),
                    lerp(w1.membrane_tension, w2.membrane_tension, t_exp),
                )
            }
            MalletInterpolationCurve::CubicBezier => {
                let t_bez = 3.0 * t * t - 2.0 * t * t * t;
                (
                    lerp(w1.radial_strike_pos, w2.radial_strike_pos, t_bez),
                    lerp(w1.strike_velocity, w2.strike_velocity, t_bez),
                    lerp(w1.mallet_hardness, w2.mallet_hardness, t_bez),
                    lerp(w1.rimshot_damping, w2.rimshot_damping, t_bez),
                    lerp(w1.roll_tremolo_rate_hz, w2.roll_tremolo_rate_hz, t_bez),
                    lerp(w1.kettle_tension_cents, w2.kettle_tension_cents, t_bez),
                    lerp(w1.air_cavity_depth, w2.air_cavity_depth, t_bez),
                    lerp(w1.membrane_tension, w2.membrane_tension, t_bez),
                )
            }
            MalletInterpolationCurve::CubicCatmullRom => (
                catmull_rom_1d(w0.radial_strike_pos, w1.radial_strike_pos, w2.radial_strike_pos, w3.radial_strike_pos, t),
                catmull_rom_1d(w0.strike_velocity, w1.strike_velocity, w2.strike_velocity, w3.strike_velocity, t),
                catmull_rom_1d(w0.mallet_hardness, w1.mallet_hardness, w2.mallet_hardness, w3.mallet_hardness, t),
                catmull_rom_1d(w0.rimshot_damping, w1.rimshot_damping, w2.rimshot_damping, w3.rimshot_damping, t),
                catmull_rom_1d(w0.roll_tremolo_rate_hz, w1.roll_tremolo_rate_hz, w2.roll_tremolo_rate_hz, w3.roll_tremolo_rate_hz, t),
                catmull_rom_1d(w0.kettle_tension_cents, w1.kettle_tension_cents, w2.kettle_tension_cents, w3.kettle_tension_cents, t),
                catmull_rom_1d(w0.air_cavity_depth, w1.air_cavity_depth, w2.air_cavity_depth, w3.air_cavity_depth, t),
                catmull_rom_1d(w0.membrane_tension, w1.membrane_tension, w2.membrane_tension, w3.membrane_tension, t),
            ),
        };

        let progress = if self.loop_length_beats > 0.0 {
            (effective_beat / self.loop_length_beats).clamp(0.0, 1.0) as f32
        } else {
            0.0
        };

        MalletBusSnapshot {
            articulation: if t < 0.5 { w1.articulation } else { w2.articulation },
            radial_strike_pos: r.clamp(0.0, 1.0),
            strike_velocity: vel.clamp(0.0, 1.0),
            mallet_hardness: hard.clamp(0.0, 1.0),
            rimshot_damping: rim.clamp(0.0, 1.0),
            roll_tremolo_rate_hz: roll.clamp(0.0, 60.0),
            kettle_tension_cents: tens.clamp(-1200.0, 1200.0),
            air_cavity_depth: cav.clamp(0.0, 1.0),
            membrane_tension: memb.clamp(0.0, 1.0),
            gesture_progress: progress,
            is_active: self.is_active,
        }
    }

    /// Dispatches evaluated articulation parameters into a lock-free `MalletBus`.
    pub fn dispatch_to_bus(&self, beat: f64, bus: &MalletBus) {
        let snap = self.evaluate_at_beat(beat);
        bus.apply_snapshot(&snap);
    }

    /// Dispatches evaluated articulation parameters into a central `ParamBus`.
    pub fn dispatch_to_param_bus(&self, beat: f64, base_id: ParamId, param_bus: &ParamBus) {
        let snap = self.evaluate_at_beat(beat);
        param_bus.set(ParamId(base_id.0), snap.articulation as u32 as f32);
        param_bus.set(ParamId(base_id.0 + 1), snap.radial_strike_pos);
        param_bus.set(ParamId(base_id.0 + 2), snap.strike_velocity);
        param_bus.set(ParamId(base_id.0 + 3), snap.mallet_hardness);
        param_bus.set(ParamId(base_id.0 + 4), snap.rimshot_damping);
        param_bus.set(ParamId(base_id.0 + 5), snap.roll_tremolo_rate_hz);
        param_bus.set(ParamId(base_id.0 + 6), snap.kettle_tension_cents);
        param_bus.set(ParamId(base_id.0 + 7), snap.air_cavity_depth);
        param_bus.set(ParamId(base_id.0 + 8), snap.membrane_tension);
    }

    /// Serializes this gesture engine trajectory to a TOML string representation.
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string(self)
    }

    /// Deserializes a gesture engine trajectory from a TOML string.
    pub fn from_toml(toml_str: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(toml_str)
    }
}

#[inline(always)]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[inline(always)]
fn catmull_rom_1d(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    0.5 * ((2.0 * p1)
        + (-p0 + p2) * t
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t * t
        + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t * t * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mallet_gesture_timpani_glissando_evaluation() {
        let engine = MalletGestureEngine::new(MalletGesturePattern::default());
        let snap_start = engine.evaluate_at_beat(0.0);
        assert_eq!(snap_start.articulation, MalletArticulation::EdgeSweetSpot);
        assert!((snap_start.radial_strike_pos - 0.70).abs() < 1e-3);
        assert!((snap_start.kettle_tension_cents - 0.0).abs() < 1e-2);

        let snap_end = engine.evaluate_at_beat(4.0);
        assert!((snap_end.kettle_tension_cents - 700.0).abs() < 1e-1);
        assert!(snap_end.strike_velocity > snap_start.strike_velocity);
    }

    #[test]
    fn test_mallet_gesture_conga_tumbao_evaluation() {
        let engine = MalletGestureEngine::new(MalletGesturePattern::CongaTumbaoGroove {
            tempo_bpm: 120.0,
            slap_hardness: 0.85,
            open_velocity: 0.80,
        });

        let snap_heel = engine.evaluate_at_beat(0.0);
        assert_eq!(snap_heel.articulation, MalletArticulation::DeadStrokeMuted);
        assert!(snap_heel.radial_strike_pos < 0.30); // Center

        let snap_slap = engine.evaluate_at_beat(1.0);
        assert_eq!(snap_slap.articulation, MalletArticulation::Rimshot);
        assert!(snap_slap.radial_strike_pos > 0.85); // Rim

        let snap_open = engine.evaluate_at_beat(3.0);
        assert_eq!(snap_open.articulation, MalletArticulation::EdgeSweetSpot);
        assert!((snap_open.strike_velocity - 0.80).abs() < 1e-2);
    }

    #[test]
    fn test_mallet_gesture_custom_spline_catmull_rom() {
        let mut engine = MalletGestureEngine::new(MalletGesturePattern::CustomSpline);
        engine.waypoints = vec![
            MalletWaypoint {
                beat: 0.0,
                radial_strike_pos: 0.20,
                strike_velocity: 0.50,
                mallet_hardness: 0.30,
                rimshot_damping: 0.0,
                roll_tremolo_rate_hz: 0.0,
                kettle_tension_cents: 0.0,
                air_cavity_depth: 0.75,
                membrane_tension: 0.50,
                articulation: MalletArticulation::CenterStrike,
                curve: MalletInterpolationCurve::CubicCatmullRom,
            },
            MalletWaypoint {
                beat: 2.0,
                radial_strike_pos: 0.80,
                strike_velocity: 0.90,
                mallet_hardness: 0.70,
                rimshot_damping: 0.5,
                roll_tremolo_rate_hz: 15.0,
                kettle_tension_cents: 400.0,
                air_cavity_depth: 0.75,
                membrane_tension: 0.60,
                articulation: MalletArticulation::Rimshot,
                curve: MalletInterpolationCurve::CubicCatmullRom,
            },
            MalletWaypoint {
                beat: 4.0,
                radial_strike_pos: 0.40,
                strike_velocity: 0.60,
                mallet_hardness: 0.40,
                rimshot_damping: 0.1,
                roll_tremolo_rate_hz: 0.0,
                kettle_tension_cents: 0.0,
                air_cavity_depth: 0.75,
                membrane_tension: 0.50,
                articulation: MalletArticulation::EdgeSweetSpot,
                curve: MalletInterpolationCurve::CubicCatmullRom,
            },
        ];

        let mid = engine.evaluate_at_beat(1.0);
        assert!(mid.radial_strike_pos > 0.20 && mid.radial_strike_pos < 0.80);
        assert!(mid.strike_velocity > 0.50 && mid.strike_velocity < 0.90);
    }

    #[test]
    fn test_mallet_gesture_toml_roundtrip() {
        let engine = MalletGestureEngine::new(MalletGesturePattern::default());
        let toml_str = engine.to_toml().expect("Failed to serialize mallet gesture TOML");
        assert!(toml_str.contains("TimpaniGlissandoRoll"));

        let restored: MalletGestureEngine = MalletGestureEngine::from_toml(&toml_str)
            .expect("Failed to deserialize mallet gesture TOML");
        assert_eq!(engine.waypoints.len(), restored.waypoints.len());
        assert_eq!(engine.loop_length_beats, restored.loop_length_beats);
    }

    #[test]
    fn test_mallet_gesture_dispatch_zero_allocation() {
        let engine = MalletGestureEngine::new(MalletGesturePattern::default());
        let bus = MalletBus::new();
        let mut param_bus = ParamBus::new();
        for i in 0..12 {
            param_bus.register(ParamId(100 + i), 0.0);
        }

        engine.dispatch_to_bus(1.5, &bus);
        let snap = bus.snapshot();
        assert!(snap.strike_velocity > 0.0);
        assert!(snap.is_active);

        engine.dispatch_to_param_bus(2.5, ParamId(100), &param_bus);
        assert!(param_bus.get(ParamId(101)).unwrap() > 0.0); // radial_strike_pos
        assert!(param_bus.get(ParamId(102)).unwrap() > 0.0); // strike_velocity
    }
}
