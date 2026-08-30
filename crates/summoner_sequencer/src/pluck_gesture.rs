// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Dynamic Plucked Articulation Gesture Timeline & Plectrum/Hammer Vector Engine (Milestone 18).
//!
//! Provides multi-dimensional continuous plucking/striking gesture trajectory generation (Pluck Position $\beta$,
//! Strike Velocity, Hammer Hardness, Plectrum Angle, Palm Mute Damping, Sympathetic Bleed Ratio, and String Stiffness)
//! supporting classical, fingerstyle, flamenco, and modern rock/metal techniques with cubic Catmull-Rom and
//! Bézier smoothing, TOML serialization, and sample-accurate lock-free dispatch into `PlectrumBus` and `ParamBus`.
//!
//! Enforces zero heap allocations in real-time execution loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use summoner_core::param_bus::{ParamBus, ParamId};
use summoner_core::plectrum_bus::{PlectrumBus, PlectrumBusSnapshot, PluckArticulation};

/// Interpolation mode between discrete pluck articulation keyframe waypoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PluckInterpolationCurve {
    #[default]
    CubicCatmullRom,
    CubicBezier,
    Linear,
    Hold,
}

/// A single keyframed multi-dimensional pluck articulation waypoint.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PluckWaypoint {
    /// Timeline beat position in quarter note beats.
    pub beat: f64,
    /// Pluck / strike position along the string $\beta \in [0.05, 0.95]$.
    pub pluck_position_beta: f32,
    /// Strike / pluck velocity $[0.0 ..= 1.0]$.
    pub strike_velocity: f32,
    /// Hammer felt / plectrum hardness $[0.0 ..= 1.0]$.
    pub hammer_hardness: f32,
    /// Plectrum attack angle in radians $[-0.5 ..= 0.5]$.
    pub plectrum_angle_rad: f32,
    /// Palm mute damping coefficient $[0.0 ..= 1.0]$.
    pub palm_mute_damping: f32,
    /// Sympathetic resonance bleed ratio $[0.0 ..= 0.50]$.
    pub sympathetic_bleed: f32,
    /// String stiffness dispersion parameter $[0.0 ..= 1.0]$.
    pub string_stiffness: f32,
    /// Pluck articulation technique.
    pub articulation: PluckArticulation,
}

impl Default for PluckWaypoint {
    fn default() -> Self {
        let (beta, vel, hard, mute, bleed) = PluckArticulation::PluckNormal.nominal_parameters();
        Self {
            beat: 0.0,
            pluck_position_beta: beta,
            strike_velocity: vel,
            hammer_hardness: hard,
            plectrum_angle_rad: 0.0,
            palm_mute_damping: mute,
            sympathetic_bleed: bleed,
            string_stiffness: 0.25,
            articulation: PluckArticulation::PluckNormal,
        }
    }
}

/// Pre-configured algorithmic pluck gesture trajectory patterns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PluckGesturePattern {
    /// Smooth fingerstyle arpeggio with alternating thumb rest strokes and finger free strokes.
    FingerstyleArpeggio {
        phrase_length_beats: f64,
        base_velocity: f32,
        accent_velocity: f32,
        pluck_position_sweep: f32,
    },
    /// Palm muted syncopated rock/metal rhythm with tight dampening and heavy pick attack.
    PalmMuteRhythm {
        chug_rate_per_beat: f32,
        mute_amount: f32,
        accent_velocity: f32,
    },
    /// Flamenco multi-finger rasgueado fan strumming with wide plectrum angle sweep.
    FlamencoRasgueado {
        strum_length_beats: f64,
        spread_duration_fraction: f32,
        peak_velocity: f32,
    },
    /// Rapid mandolin / bouzouki tremolo picking with position jitter and dynamic swell.
    TremoloMandolin {
        tremolo_rate_hz: f32,
        velocity_swell_depth: f32,
        base_position: f32,
    },
    /// Piano forte dynamic strike trajectory with non-linear velocity and hammer hardness scaling.
    PianoForteDynamic {
        measure_length_beats: f64,
        min_velocity: f32,
        max_velocity: f32,
    },
    /// Funk slap bass thumb slap alternating with index snap pop.
    SlapPopAlternation {
        slap_rate_per_beat: f32,
        slap_velocity: f32,
        pop_velocity: f32,
    },
    /// User-defined multi-keyframe Catmull-Rom spline trajectory.
    CustomSpline,
}

impl Default for PluckGesturePattern {
    fn default() -> Self {
        Self::FingerstyleArpeggio {
            phrase_length_beats: 4.0,
            base_velocity: 0.65,
            accent_velocity: 0.90,
            pluck_position_sweep: 0.08,
        }
    }
}

/// Dynamic Plucked Articulation Gesture Timeline Engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluckGestureEngine {
    /// Active procedural gesture pattern or custom spline.
    pub pattern: PluckGesturePattern,
    /// Interpolation smoothing curve type.
    pub interpolation: PluckInterpolationCurve,
    /// User waypoints (for CustomSpline mode).
    pub waypoints: Vec<PluckWaypoint>,
    /// Loop length in quarter note beats (if looping is enabled).
    pub loop_length_beats: Option<f64>,
}

impl Default for PluckGestureEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PluckGestureEngine {
    /// Creates a new pluck gesture engine with default Fingerstyle Arpeggio pattern.
    pub fn new() -> Self {
        Self {
            pattern: PluckGesturePattern::default(),
            interpolation: PluckInterpolationCurve::CubicCatmullRom,
            waypoints: Vec::new(),
            loop_length_beats: Some(4.0),
        }
    }

    /// Adds a keyframe waypoint to the custom trajectory.
    pub fn add_waypoint(&mut self, waypoint: PluckWaypoint) {
        self.waypoints.push(waypoint);
        self.waypoints.sort_by(|a, b| a.beat.partial_cmp(&b.beat).unwrap_or(std::cmp::Ordering::Equal));
    }

    /// Evaluates continuous pluck articulation parameters at timeline beat position.
    pub fn evaluate_beat(&self, beat: f64) -> PlectrumBusSnapshot {
        let eff_beat = if let Some(loop_len) = self.loop_length_beats {
            if loop_len > 0.0 {
                let rem = beat % loop_len;
                if rem < 0.0 { rem + loop_len } else { rem }
            } else {
                beat
            }
        } else {
            beat
        };

        match &self.pattern {
            PluckGesturePattern::FingerstyleArpeggio {
                phrase_length_beats,
                base_velocity,
                accent_velocity,
                pluck_position_sweep,
            } => {
                let p_len = phrase_length_beats.max(0.5);
                let progress = ((eff_beat % p_len) / p_len) as f32; // 0.0 .. 1.0

                // 16th note step within the measure (4 notes per beat)
                let note_sub = ((eff_beat * 4.0) % 4.0).floor() as usize;
                let is_downbeat = note_sub == 0;

                let (art, vel, pos_offset) = if is_downbeat {
                    (PluckArticulation::FingerRestStroke, *accent_velocity, -pluck_position_sweep * 0.5)
                } else {
                    (PluckArticulation::FingerFreeStroke, *base_velocity, pluck_position_sweep * 0.5)
                };

                // Continuous breathing sweep across the phrase
                let pos = (0.16 + pos_offset + (progress * 2.0 * PI).sin() * 0.04).clamp(0.08, 0.40);
                let hardness = 0.35 + 0.20 * vel;

                PlectrumBusSnapshot {
                    articulation: art,
                    pluck_position_beta: pos,
                    strike_velocity: vel,
                    hammer_hardness: hardness,
                    plectrum_angle_rad: 0.0,
                    palm_mute_damping: 0.0,
                    sympathetic_bleed: 0.15,
                    string_stiffness: 0.20,
                    gesture_progress: progress,
                    is_active: true,
                }
            }

            PluckGesturePattern::PalmMuteRhythm {
                chug_rate_per_beat,
                mute_amount,
                accent_velocity,
            } => {
                let rate = chug_rate_per_beat.max(0.5);
                let step_idx = (eff_beat * rate as f64).floor() as usize;
                let progress = ((eff_beat * rate as f64) % 1.0) as f32;

                let is_accent = step_idx.is_multiple_of(4);
                let vel = if is_accent { *accent_velocity } else { *accent_velocity * 0.70 };
                let mute = if is_accent { *mute_amount * 0.60 } else { *mute_amount };

                PlectrumBusSnapshot {
                    articulation: PluckArticulation::PalmMute,
                    pluck_position_beta: 0.10,
                    strike_velocity: vel,
                    hammer_hardness: 0.75,
                    plectrum_angle_rad: 0.10,
                    palm_mute_damping: mute,
                    sympathetic_bleed: 0.05,
                    string_stiffness: 0.35,
                    gesture_progress: progress,
                    is_active: true,
                }
            }

            PluckGesturePattern::FlamencoRasgueado {
                strum_length_beats,
                spread_duration_fraction: _,
                peak_velocity,
            } => {
                let s_len = strum_length_beats.max(0.5);
                let progress = ((eff_beat % s_len) / s_len) as f32; // 0.0 .. 1.0

                let angle_sweep = (progress - 0.5) * 0.60;
                let vel = *peak_velocity * (progress * PI).sin().max(0.2);

                PlectrumBusSnapshot {
                    articulation: PluckArticulation::PluckNormal,
                    pluck_position_beta: 0.18 + 0.06 * progress,
                    strike_velocity: vel,
                    hammer_hardness: 0.60,
                    plectrum_angle_rad: angle_sweep,
                    palm_mute_damping: 0.0,
                    sympathetic_bleed: 0.22,
                    string_stiffness: 0.18,
                    gesture_progress: progress,
                    is_active: true,
                }
            }

            PluckGesturePattern::TremoloMandolin {
                tremolo_rate_hz,
                velocity_swell_depth,
                base_position,
            } => {
                let t_sec = eff_beat * 0.5; // ~120 BPM
                let trem_phase = (t_sec as f32 * 2.0 * PI * tremolo_rate_hz.max(1.0)).sin();
                let swell = 0.5 * (1.0 + (t_sec as f32 * PI * 0.5).sin());
                let vel = 0.50 + *velocity_swell_depth * swell + 0.15 * trem_phase.abs();

                PlectrumBusSnapshot {
                    articulation: PluckArticulation::TremoloPluck,
                    pluck_position_beta: *base_position,
                    strike_velocity: vel.clamp(0.1, 1.0),
                    hammer_hardness: 0.65,
                    plectrum_angle_rad: 0.05 * trem_phase,
                    palm_mute_damping: 0.0,
                    sympathetic_bleed: 0.18,
                    string_stiffness: 0.28,
                    gesture_progress: swell,
                    is_active: true,
                }
            }

            PluckGesturePattern::PianoForteDynamic {
                measure_length_beats,
                min_velocity,
                max_velocity,
            } => {
                let m_len = measure_length_beats.max(0.5);
                let progress = ((eff_beat % m_len) / m_len) as f32; // 0.0 .. 1.0
                let vel = *min_velocity + (*max_velocity - *min_velocity) * progress;
                let hardness = 0.30 + 0.60 * vel;

                PlectrumBusSnapshot {
                    articulation: PluckArticulation::HammerStruck,
                    pluck_position_beta: 0.12,
                    strike_velocity: vel,
                    hammer_hardness: hardness,
                    plectrum_angle_rad: 0.0,
                    palm_mute_damping: 0.0,
                    sympathetic_bleed: 0.35,
                    string_stiffness: 0.45,
                    gesture_progress: progress,
                    is_active: true,
                }
            }

            PluckGesturePattern::SlapPopAlternation {
                slap_rate_per_beat,
                slap_velocity,
                pop_velocity,
            } => {
                let rate = slap_rate_per_beat.max(0.5);
                let step_idx = (eff_beat * rate as f64).floor() as usize;
                let progress = ((eff_beat * rate as f64) % 1.0) as f32;

                let is_pop = step_idx % 2 == 1;
                let (art, vel, beta) = if is_pop {
                    (PluckArticulation::SnapPizzicato, *pop_velocity, 0.06)
                } else {
                    (PluckArticulation::PluckNormal, *slap_velocity, 0.09)
                };

                PlectrumBusSnapshot {
                    articulation: art,
                    pluck_position_beta: beta,
                    strike_velocity: vel,
                    hammer_hardness: 0.85,
                    plectrum_angle_rad: 0.0,
                    palm_mute_damping: 0.0,
                    sympathetic_bleed: 0.20,
                    string_stiffness: 0.30,
                    gesture_progress: progress,
                    is_active: true,
                }
            }

            PluckGesturePattern::CustomSpline => {
                self.evaluate_custom_spline(eff_beat)
            }
        }
    }

    /// Evaluates user-defined keyframe waypoints with cubic Catmull-Rom or Bézier interpolation.
    fn evaluate_custom_spline(&self, beat: f64) -> PlectrumBusSnapshot {
        if self.waypoints.is_empty() {
            return PlectrumBusSnapshot::default();
        }
        if self.waypoints.len() == 1 {
            let wp = &self.waypoints[0];
            return PlectrumBusSnapshot {
                articulation: wp.articulation,
                pluck_position_beta: wp.pluck_position_beta,
                strike_velocity: wp.strike_velocity,
                hammer_hardness: wp.hammer_hardness,
                plectrum_angle_rad: wp.plectrum_angle_rad,
                palm_mute_damping: wp.palm_mute_damping,
                sympathetic_bleed: wp.sympathetic_bleed,
                string_stiffness: wp.string_stiffness,
                gesture_progress: 0.0,
                is_active: true,
            };
        }

        let n = self.waypoints.len();
        let idx = match self.waypoints.binary_search_by(|wp| wp.beat.partial_cmp(&beat).unwrap()) {
            Ok(i) => i,
            Err(i) => {
                if i == 0 { 0 } else { i - 1 }
            }
        };

        if idx >= n - 1 {
            let wp = &self.waypoints[n - 1];
            return PlectrumBusSnapshot {
                articulation: wp.articulation,
                pluck_position_beta: wp.pluck_position_beta,
                strike_velocity: wp.strike_velocity,
                hammer_hardness: wp.hammer_hardness,
                plectrum_angle_rad: wp.plectrum_angle_rad,
                palm_mute_damping: wp.palm_mute_damping,
                sympathetic_bleed: wp.sympathetic_bleed,
                string_stiffness: wp.string_stiffness,
                gesture_progress: 1.0,
                is_active: true,
            };
        }

        let wp0 = if idx == 0 { &self.waypoints[0] } else { &self.waypoints[idx - 1] };
        let wp1 = &self.waypoints[idx];
        let wp2 = &self.waypoints[idx + 1];
        let wp3 = if idx + 2 < n { &self.waypoints[idx + 2] } else { &self.waypoints[idx + 1] };

        let span = (wp2.beat - wp1.beat).max(1e-6);
        let t = ((beat - wp1.beat) / span).clamp(0.0, 1.0) as f32;

        let (beta_pos, vel, hard, angle, mute, bleed, stiff) = match self.interpolation {
            PluckInterpolationCurve::Hold => (
                wp1.pluck_position_beta,
                wp1.strike_velocity,
                wp1.hammer_hardness,
                wp1.plectrum_angle_rad,
                wp1.palm_mute_damping,
                wp1.sympathetic_bleed,
                wp1.string_stiffness,
            ),
            PluckInterpolationCurve::Linear => (
                wp1.pluck_position_beta + t * (wp2.pluck_position_beta - wp1.pluck_position_beta),
                wp1.strike_velocity + t * (wp2.strike_velocity - wp1.strike_velocity),
                wp1.hammer_hardness + t * (wp2.hammer_hardness - wp1.hammer_hardness),
                wp1.plectrum_angle_rad + t * (wp2.plectrum_angle_rad - wp1.plectrum_angle_rad),
                wp1.palm_mute_damping + t * (wp2.palm_mute_damping - wp1.palm_mute_damping),
                wp1.sympathetic_bleed + t * (wp2.sympathetic_bleed - wp1.sympathetic_bleed),
                wp1.string_stiffness + t * (wp2.string_stiffness - wp1.string_stiffness),
            ),
            PluckInterpolationCurve::CubicCatmullRom => {
                let b = Self::catmull_rom_1d(wp0.pluck_position_beta, wp1.pluck_position_beta, wp2.pluck_position_beta, wp3.pluck_position_beta, t);
                let v = Self::catmull_rom_1d(wp0.strike_velocity, wp1.strike_velocity, wp2.strike_velocity, wp3.strike_velocity, t);
                let h = Self::catmull_rom_1d(wp0.hammer_hardness, wp1.hammer_hardness, wp2.hammer_hardness, wp3.hammer_hardness, t);
                let a = Self::catmull_rom_1d(wp0.plectrum_angle_rad, wp1.plectrum_angle_rad, wp2.plectrum_angle_rad, wp3.plectrum_angle_rad, t);
                let m = Self::catmull_rom_1d(wp0.palm_mute_damping, wp1.palm_mute_damping, wp2.palm_mute_damping, wp3.palm_mute_damping, t);
                let bl = Self::catmull_rom_1d(wp0.sympathetic_bleed, wp1.sympathetic_bleed, wp2.sympathetic_bleed, wp3.sympathetic_bleed, t);
                let st = Self::catmull_rom_1d(wp0.string_stiffness, wp1.string_stiffness, wp2.string_stiffness, wp3.string_stiffness, t);
                (b, v, h, a, m, bl, st)
            }
            PluckInterpolationCurve::CubicBezier => {
                let b = t * t * (3.0 - 2.0 * t); // Smoothstep S-curve
                (
                    wp1.pluck_position_beta + b * (wp2.pluck_position_beta - wp1.pluck_position_beta),
                    wp1.strike_velocity + b * (wp2.strike_velocity - wp1.strike_velocity),
                    wp1.hammer_hardness + b * (wp2.hammer_hardness - wp1.hammer_hardness),
                    wp1.plectrum_angle_rad + b * (wp2.plectrum_angle_rad - wp1.plectrum_angle_rad),
                    wp1.palm_mute_damping + b * (wp2.palm_mute_damping - wp1.palm_mute_damping),
                    wp1.sympathetic_bleed + b * (wp2.sympathetic_bleed - wp1.sympathetic_bleed),
                    wp1.string_stiffness + b * (wp2.string_stiffness - wp1.string_stiffness),
                )
            }
        };

        let art = if t < 0.5 { wp1.articulation } else { wp2.articulation };

        PlectrumBusSnapshot {
            articulation: art,
            pluck_position_beta: beta_pos.clamp(0.05, 0.95),
            strike_velocity: vel.clamp(0.0, 1.0),
            hammer_hardness: hard.clamp(0.0, 1.0),
            plectrum_angle_rad: angle.clamp(-0.5, 0.5),
            palm_mute_damping: mute.clamp(0.0, 1.0),
            sympathetic_bleed: bleed.clamp(0.0, 0.50),
            string_stiffness: stiff.clamp(0.0, 1.0),
            gesture_progress: t,
            is_active: true,
        }
    }

    #[inline]
    fn catmull_rom_1d(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
        0.5 * ((2.0 * p1)
            + (-p0 + p2) * t
            + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t * t
            + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t * t * t)
    }

    /// Evaluates timeline at beat and dispatches the snapshot directly into a `PlectrumBus`.
    pub fn dispatch_to_plectrum_bus(&self, beat: f64, bus: &PlectrumBus) {
        let snap = self.evaluate_beat(beat);
        bus.apply_snapshot(&snap);
    }

    /// Dispatches continuous evaluated parameters into `ParamBus`.
    pub fn dispatch_to_param_bus(&self, beat: f64, bus: &ParamBus, base_param_id: ParamId) {
        let snap = self.evaluate_beat(beat);
        bus.set(ParamId(base_param_id.0), snap.articulation as u32 as f32);
        bus.set(ParamId(base_param_id.0 + 1), snap.pluck_position_beta);
        bus.set(ParamId(base_param_id.0 + 2), snap.strike_velocity);
        bus.set(ParamId(base_param_id.0 + 3), snap.hammer_hardness);
        bus.set(ParamId(base_param_id.0 + 4), snap.plectrum_angle_rad);
        bus.set(ParamId(base_param_id.0 + 5), snap.palm_mute_damping);
        bus.set(ParamId(base_param_id.0 + 6), snap.sympathetic_bleed);
        bus.set(ParamId(base_param_id.0 + 7), snap.string_stiffness);
        bus.set(ParamId(base_param_id.0 + 8), snap.gesture_progress);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::allocator::AllocGuard;

    #[test]
    fn test_pluck_gesture_fingerstyle_evaluation() {
        let engine = PluckGestureEngine::new();
        let snap = engine.evaluate_beat(0.0);
        assert_eq!(snap.articulation, PluckArticulation::FingerRestStroke);
        assert!(snap.strike_velocity >= 0.85);
        assert!(snap.is_active);
    }

    #[test]
    fn test_pluck_gesture_custom_spline_catmull_rom() {
        let mut engine = PluckGestureEngine {
            pattern: PluckGesturePattern::CustomSpline,
            interpolation: PluckInterpolationCurve::CubicCatmullRom,
            waypoints: Vec::new(),
            loop_length_beats: Some(4.0),
        };

        engine.add_waypoint(PluckWaypoint {
            beat: 0.0,
            pluck_position_beta: 0.10,
            strike_velocity: 0.50,
            hammer_hardness: 0.30,
            palm_mute_damping: 0.0,
            sympathetic_bleed: 0.10,
            string_stiffness: 0.20,
            articulation: PluckArticulation::PluckNormal,
            ..Default::default()
        });

        engine.add_waypoint(PluckWaypoint {
            beat: 2.0,
            pluck_position_beta: 0.25,
            strike_velocity: 0.90,
            hammer_hardness: 0.70,
            palm_mute_damping: 0.60,
            sympathetic_bleed: 0.30,
            string_stiffness: 0.40,
            articulation: PluckArticulation::PalmMute,
            ..Default::default()
        });

        engine.add_waypoint(PluckWaypoint {
            beat: 4.0,
            pluck_position_beta: 0.15,
            strike_velocity: 0.60,
            hammer_hardness: 0.40,
            palm_mute_damping: 0.0,
            sympathetic_bleed: 0.15,
            string_stiffness: 0.25,
            articulation: PluckArticulation::PluckNormal,
            ..Default::default()
        });

        let mid = engine.evaluate_beat(1.0);
        assert!(mid.pluck_position_beta > 0.10 && mid.pluck_position_beta < 0.25);
        assert!(mid.strike_velocity > 0.50 && mid.strike_velocity < 0.90);
    }

    #[test]
    fn test_pluck_gesture_dispatch_zero_allocation() {
        let engine = PluckGestureEngine::new();
        let bus = PlectrumBus::new();
        let _guard = AllocGuard::new();

        for i in 0..100 {
            let beat = i as f64 * 0.05;
            engine.dispatch_to_plectrum_bus(beat, &bus);
        }
    }

    #[test]
    fn test_pluck_gesture_toml_roundtrip() {
        let mut engine = PluckGestureEngine::new();
        engine.pattern = PluckGesturePattern::PalmMuteRhythm {
            chug_rate_per_beat: 4.0,
            mute_amount: 0.85,
            accent_velocity: 0.95,
        };
        engine.add_waypoint(PluckWaypoint::default());

        let toml_str = toml::to_string_pretty(&engine).expect("Failed to serialize pluck gesture engine");
        let decoded: PluckGestureEngine = toml::from_str(&toml_str).expect("Failed to deserialize pluck gesture engine");

        assert_eq!(engine.waypoints.len(), decoded.waypoints.len());
        assert_eq!(engine.interpolation, decoded.interpolation);
    }
}
