// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Dynamic Bowing Articulation Gesture Timeline & Expression Vector Engine (Milestone 16).
//!
//! Provides multi-dimensional continuous bowing gesture trajectory generation (Bow Velocity,
//! Normal Force, Bridge Proximity $\beta$, Rosin Adhesion, String Damping, and Vibrato) supporting
//! classical and modern extended techniques with cubic Catmull-Rom and Bézier smoothing,
//! TOML serialization, and sample-accurate lock-free dispatch into `ArticulationBus` and `ParamBus`.
//!
//! Enforces zero heap allocations in real-time execution loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use summoner_core::articulation_bus::{ArticulationBus, ArticulationBusSnapshot, BowingTechnique};
use summoner_core::param_bus::{ParamBus, ParamId};

/// Interpolation mode between discrete bowing articulation keyframe waypoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum GestureInterpolationCurve {
    #[default]
    CubicCatmullRom,
    CubicBezier,
    Linear,
    Hold,
}

/// A single keyframed multi-dimensional bowing articulation waypoint.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BowingWaypoint {
    /// Timeline beat position in quarter note beats.
    pub beat: f64,
    /// Transverse bow velocity in m/s (signed for down-bow / up-bow).
    pub bow_velocity_mps: f32,
    /// Normal clamping bow force in Newtons.
    pub bow_force_n: f32,
    /// Bridge proximity ratio $\beta = x_{bow} / L$ [0.02 ..= 0.50].
    pub bridge_proximity_beta: f32,
    /// Bowing technique category.
    pub technique: BowingTechnique,
    /// Rosin adhesion strength [0.0 ..= 1.0].
    pub rosin_adhesion: f32,
    /// High-frequency string damping [0.0 ..= 1.0].
    pub string_damping: f32,
    /// Vibrato depth in semitones [0.0 ..= 2.0].
    pub vibrato_depth: f32,
    /// Vibrato rate in Hz [0.5 ..= 15.0].
    pub vibrato_rate_hz: f32,
}

impl Default for BowingWaypoint {
    fn default() -> Self {
        let (vel, force, beta) = BowingTechnique::Legato.nominal_parameters();
        Self {
            beat: 0.0,
            bow_velocity_mps: vel,
            bow_force_n: force,
            bridge_proximity_beta: beta,
            technique: BowingTechnique::Legato,
            rosin_adhesion: 0.80,
            string_damping: 0.15,
            vibrato_depth: 0.0,
            vibrato_rate_hz: 5.5,
        }
    }
}

/// Pre-configured algorithmic bowing gesture trajectory patterns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BowingGesturePattern {
    /// Smooth continuous singing legato strokes with alternating down/up bow at bar boundaries.
    LegatoCantabile {
        stroke_length_beats: f64,
        peak_velocity_mps: f32,
        bow_force_n: f32,
        bridge_proximity_beta: f32,
    },
    /// Accented hammered martelé attack per quarter note.
    MarteleAccented {
        attack_fraction: f32,
        attack_force_n: f32,
        sustain_force_n: f32,
        peak_velocity_mps: f32,
    },
    /// Bouncing spiccato off-the-string periodic strikes.
    SpiccatoBouncing {
        bounce_rate_per_beat: f32,
        contact_fraction: f32,
        strike_force_n: f32,
    },
    /// Rapid unmeasured tremolo stroke oscillation.
    TremoloRapid {
        frequency_hz: f32,
        velocity_amplitude_mps: f32,
        bow_force_n: f32,
        bridge_proximity_beta: f32,
    },
    /// Continuous spectral timbral sweep from Sul Ponticello (near bridge) to Sul Tasto (fingerboard).
    PonticelloToTastoSweep {
        sweep_length_beats: f64,
        min_beta: f32,
        max_beta: f32,
        bow_velocity_mps: f32,
        bow_force_n: f32,
    },
    /// User-defined multi-keyframe Catmull-Rom spline trajectory.
    CustomSpline,
}

impl Default for BowingGesturePattern {
    fn default() -> Self {
        Self::LegatoCantabile {
            stroke_length_beats: 2.0,
            peak_velocity_mps: 0.45,
            bow_force_n: 1.25,
            bridge_proximity_beta: 0.12,
        }
    }
}

/// Dynamic Bowing Articulation Gesture Timeline Engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BowingGestureEngine {
    /// Active procedural gesture pattern or custom spline.
    pub pattern: BowingGesturePattern,
    /// Interpolation smoothing curve type.
    pub interpolation: GestureInterpolationCurve,
    /// User waypoints (for CustomSpline mode).
    pub waypoints: Vec<BowingWaypoint>,
    /// Loop length in quarter note beats (if looping is enabled).
    pub loop_length_beats: Option<f64>,
}

impl Default for BowingGestureEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl BowingGestureEngine {
    /// Creates a new gesture engine with default Legato Cantabile pattern.
    pub fn new() -> Self {
        Self {
            pattern: BowingGesturePattern::default(),
            interpolation: GestureInterpolationCurve::CubicCatmullRom,
            waypoints: Vec::new(),
            loop_length_beats: Some(4.0),
        }
    }

    /// Adds a keyframe waypoint to the custom trajectory.
    pub fn add_waypoint(&mut self, waypoint: BowingWaypoint) {
        self.waypoints.push(waypoint);
        self.waypoints.sort_by(|a, b| a.beat.partial_cmp(&b.beat).unwrap_or(std::cmp::Ordering::Equal));
    }

    /// Evaluates continuous bowing articulation parameters at timeline beat position.
    pub fn evaluate_beat(&self, beat: f64) -> ArticulationBusSnapshot {
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
            BowingGesturePattern::LegatoCantabile {
                stroke_length_beats,
                peak_velocity_mps,
                bow_force_n,
                bridge_proximity_beta,
            } => {
                let stroke_len = stroke_length_beats.max(0.25);
                let stroke_idx = (eff_beat / stroke_len).floor() as i64;
                let stroke_phase = ((eff_beat % stroke_len) / stroke_len) as f32; // 0.0 .. 1.0

                // Smooth bell envelope for velocity; alternating direction per stroke
                let direction = if stroke_idx % 2 == 0 { 1.0f32 } else { -1.0f32 };
                let vel_env = (stroke_phase * PI).sin();
                let vel = direction * peak_velocity_mps * (0.20 + 0.80 * vel_env);

                ArticulationBusSnapshot {
                    technique: BowingTechnique::Legato,
                    bow_velocity_mps: vel,
                    bow_force_n: *bow_force_n,
                    bridge_proximity_beta: *bridge_proximity_beta,
                    rosin_adhesion: 0.82,
                    string_damping: 0.15,
                    vibrato_depth: 0.15 * vel_env,
                    vibrato_rate_hz: 5.5,
                    gesture_progress: stroke_phase,
                    is_active: true,
                }
            }

            BowingGesturePattern::MarteleAccented {
                attack_fraction,
                attack_force_n,
                sustain_force_n,
                peak_velocity_mps,
            } => {
                let beat_phase = (eff_beat % 1.0) as f32; // 1 beat period
                let att_frac = attack_fraction.clamp(0.05, 0.50);

                let (force, vel) = if beat_phase < att_frac {
                    let t = beat_phase / att_frac;
                    let f = *attack_force_n * (1.0 - t) + *sustain_force_n * t;
                    let v = *peak_velocity_mps * (0.50 + 0.50 * (1.0 - t));
                    (f, v)
                } else {
                    let t = (beat_phase - att_frac) / (1.0 - att_frac);
                    let f = *sustain_force_n * (1.0 - 0.70 * t);
                    let v = *peak_velocity_mps * 0.40 * (1.0 - 0.80 * t);
                    (f, v)
                };

                ArticulationBusSnapshot {
                    technique: BowingTechnique::Martele,
                    bow_velocity_mps: vel,
                    bow_force_n: force,
                    bridge_proximity_beta: 0.10,
                    rosin_adhesion: 0.95,
                    string_damping: 0.12,
                    vibrato_depth: 0.05,
                    vibrato_rate_hz: 6.0,
                    gesture_progress: beat_phase,
                    is_active: true,
                }
            }

            BowingGesturePattern::SpiccatoBouncing {
                bounce_rate_per_beat,
                contact_fraction,
                strike_force_n,
            } => {
                let rate = bounce_rate_per_beat.max(1.0);
                let cycle_phase = ((eff_beat * rate as f64) % 1.0) as f32;
                let contact = contact_fraction.clamp(0.10, 0.80);

                let (force, vel) = if cycle_phase < contact {
                    let t = cycle_phase / contact;
                    let f = *strike_force_n * (t * PI).sin();
                    let v = 0.55 * (t * PI).sin();
                    (f, v)
                } else {
                    (0.0f32, 0.0f32) // Airborne phase
                };

                ArticulationBusSnapshot {
                    technique: BowingTechnique::Spiccato,
                    bow_velocity_mps: vel,
                    bow_force_n: force,
                    bridge_proximity_beta: 0.14,
                    rosin_adhesion: 0.70,
                    string_damping: 0.20,
                    vibrato_depth: 0.0,
                    vibrato_rate_hz: 5.0,
                    gesture_progress: cycle_phase,
                    is_active: true,
                }
            }

            BowingGesturePattern::TremoloRapid {
                frequency_hz,
                velocity_amplitude_mps,
                bow_force_n,
                bridge_proximity_beta,
            } => {
                let t_sec = eff_beat * 0.5; // Assuming ~120 BPM
                let phase = (t_sec as f32 * 2.0 * PI * frequency_hz.max(1.0)).sin();
                let vel = phase * velocity_amplitude_mps;

                ArticulationBusSnapshot {
                    technique: BowingTechnique::Tremolo,
                    bow_velocity_mps: vel,
                    bow_force_n: *bow_force_n,
                    bridge_proximity_beta: *bridge_proximity_beta,
                    rosin_adhesion: 0.85,
                    string_damping: 0.10,
                    vibrato_depth: 0.0,
                    vibrato_rate_hz: 5.5,
                    gesture_progress: ((phase + 1.0) * 0.5).clamp(0.0, 1.0),
                    is_active: true,
                }
            }

            BowingGesturePattern::PonticelloToTastoSweep {
                sweep_length_beats,
                min_beta,
                max_beta,
                bow_velocity_mps,
                bow_force_n,
            } => {
                let sweep_len = sweep_length_beats.max(1.0);
                let progress = ((eff_beat % sweep_len) / sweep_len) as f32; // 0.0 .. 1.0
                let beta = min_beta + progress * (max_beta - min_beta);
                let tech = if beta < 0.08 {
                    BowingTechnique::SulPonticello
                } else if beta > 0.22 {
                    BowingTechnique::SulTasto
                } else {
                    BowingTechnique::Legato
                };

                ArticulationBusSnapshot {
                    technique: tech,
                    bow_velocity_mps: *bow_velocity_mps,
                    bow_force_n: *bow_force_n,
                    bridge_proximity_beta: beta,
                    rosin_adhesion: 0.80,
                    string_damping: 0.15,
                    vibrato_depth: 0.10 * (1.0 - progress),
                    vibrato_rate_hz: 5.5,
                    gesture_progress: progress,
                    is_active: true,
                }
            }

            BowingGesturePattern::CustomSpline => {
                self.evaluate_custom_spline(eff_beat)
            }
        }
    }

    /// Evaluates user-defined keyframe waypoints with cubic Catmull-Rom or Bézier interpolation.
    fn evaluate_custom_spline(&self, beat: f64) -> ArticulationBusSnapshot {
        if self.waypoints.is_empty() {
            return ArticulationBusSnapshot::default();
        }
        if self.waypoints.len() == 1 {
            let wp = &self.waypoints[0];
            return ArticulationBusSnapshot {
                technique: wp.technique,
                bow_velocity_mps: wp.bow_velocity_mps,
                bow_force_n: wp.bow_force_n,
                bridge_proximity_beta: wp.bridge_proximity_beta,
                rosin_adhesion: wp.rosin_adhesion,
                string_damping: wp.string_damping,
                vibrato_depth: wp.vibrato_depth,
                vibrato_rate_hz: wp.vibrato_rate_hz,
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
            return ArticulationBusSnapshot {
                technique: wp.technique,
                bow_velocity_mps: wp.bow_velocity_mps,
                bow_force_n: wp.bow_force_n,
                bridge_proximity_beta: wp.bridge_proximity_beta,
                rosin_adhesion: wp.rosin_adhesion,
                string_damping: wp.string_damping,
                vibrato_depth: wp.vibrato_depth,
                vibrato_rate_hz: wp.vibrato_rate_hz,
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

        let (vel, force, beta_pos, rosin, damping, vib_d, vib_r) = match self.interpolation {
            GestureInterpolationCurve::Hold => (
                wp1.bow_velocity_mps,
                wp1.bow_force_n,
                wp1.bridge_proximity_beta,
                wp1.rosin_adhesion,
                wp1.string_damping,
                wp1.vibrato_depth,
                wp1.vibrato_rate_hz,
            ),
            GestureInterpolationCurve::Linear => (
                wp1.bow_velocity_mps + t * (wp2.bow_velocity_mps - wp1.bow_velocity_mps),
                wp1.bow_force_n + t * (wp2.bow_force_n - wp1.bow_force_n),
                wp1.bridge_proximity_beta + t * (wp2.bridge_proximity_beta - wp1.bridge_proximity_beta),
                wp1.rosin_adhesion + t * (wp2.rosin_adhesion - wp1.rosin_adhesion),
                wp1.string_damping + t * (wp2.string_damping - wp1.string_damping),
                wp1.vibrato_depth + t * (wp2.vibrato_depth - wp1.vibrato_depth),
                wp1.vibrato_rate_hz + t * (wp2.vibrato_rate_hz - wp1.vibrato_rate_hz),
            ),
            GestureInterpolationCurve::CubicBezier => {
                let t_ease = t * t * (3.0 - 2.0 * t); // SmoothStep Bézier ease
                (
                    wp1.bow_velocity_mps + t_ease * (wp2.bow_velocity_mps - wp1.bow_velocity_mps),
                    wp1.bow_force_n + t_ease * (wp2.bow_force_n - wp1.bow_force_n),
                    wp1.bridge_proximity_beta + t_ease * (wp2.bridge_proximity_beta - wp1.bridge_proximity_beta),
                    wp1.rosin_adhesion + t_ease * (wp2.rosin_adhesion - wp1.rosin_adhesion),
                    wp1.string_damping + t_ease * (wp2.string_damping - wp1.string_damping),
                    wp1.vibrato_depth + t_ease * (wp2.vibrato_depth - wp1.vibrato_depth),
                    wp1.vibrato_rate_hz + t_ease * (wp2.vibrato_rate_hz - wp1.vibrato_rate_hz),
                )
            }
            GestureInterpolationCurve::CubicCatmullRom => (
                catmull_rom_1d(wp0.bow_velocity_mps, wp1.bow_velocity_mps, wp2.bow_velocity_mps, wp3.bow_velocity_mps, t),
                catmull_rom_1d(wp0.bow_force_n, wp1.bow_force_n, wp2.bow_force_n, wp3.bow_force_n, t),
                catmull_rom_1d(wp0.bridge_proximity_beta, wp1.bridge_proximity_beta, wp2.bridge_proximity_beta, wp3.bridge_proximity_beta, t),
                catmull_rom_1d(wp0.rosin_adhesion, wp1.rosin_adhesion, wp2.rosin_adhesion, wp3.rosin_adhesion, t),
                catmull_rom_1d(wp0.string_damping, wp1.string_damping, wp2.string_damping, wp3.string_damping, t),
                catmull_rom_1d(wp0.vibrato_depth, wp1.vibrato_depth, wp2.vibrato_depth, wp3.vibrato_depth, t),
                catmull_rom_1d(wp0.vibrato_rate_hz, wp1.vibrato_rate_hz, wp2.vibrato_rate_hz, wp3.vibrato_rate_hz, t),
            ),
        };

        ArticulationBusSnapshot {
            technique: if t < 0.5 { wp1.technique } else { wp2.technique },
            bow_velocity_mps: vel,
            bow_force_n: force.max(0.0),
            bridge_proximity_beta: beta_pos.clamp(0.02, 0.50),
            rosin_adhesion: rosin.clamp(0.0, 1.0),
            string_damping: damping.clamp(0.0, 1.0),
            vibrato_depth: vib_d.clamp(0.0, 2.0),
            vibrato_rate_hz: vib_r.clamp(0.5, 15.0),
            gesture_progress: t,
            is_active: true,
        }
    }

    /// Dispatches continuous evaluated parameters into `ArticulationBus`.
    pub fn dispatch_to_articulation_bus(&self, beat: f64, bus: &ArticulationBus) {
        let snap = self.evaluate_beat(beat);
        bus.set_technique(snap.technique);
        bus.set_gesture_vector(
            snap.bow_velocity_mps,
            snap.bow_force_n,
            snap.bridge_proximity_beta,
            snap.rosin_adhesion,
            snap.string_damping,
        );
        bus.set_vibrato(snap.vibrato_depth, snap.vibrato_rate_hz);
        bus.set_progress(snap.gesture_progress, snap.is_active);
    }

    /// Dispatches continuous evaluated parameters into `ParamBus`.
    pub fn dispatch_to_param_bus(&self, beat: f64, bus: &ParamBus, base_param_id: ParamId) {
        let snap = self.evaluate_beat(beat);
        bus.set(ParamId(base_param_id.0), snap.technique as u32 as f32);
        bus.set(ParamId(base_param_id.0 + 1), snap.bow_velocity_mps);
        bus.set(ParamId(base_param_id.0 + 2), snap.bow_force_n);
        bus.set(ParamId(base_param_id.0 + 3), snap.bridge_proximity_beta);
        bus.set(ParamId(base_param_id.0 + 4), snap.rosin_adhesion);
        bus.set(ParamId(base_param_id.0 + 5), snap.string_damping);
        bus.set(ParamId(base_param_id.0 + 6), snap.vibrato_depth);
        bus.set(ParamId(base_param_id.0 + 7), snap.vibrato_rate_hz);
        bus.set(ParamId(base_param_id.0 + 8), snap.gesture_progress);
    }
}

/// 1D Catmull-Rom cubic spline interpolation between 4 points.
#[inline]
fn catmull_rom_1d(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;
    0.5 * ((2.0 * p1)
        + (-p0 + p2) * t
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
        + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::allocator::AllocGuard;

    #[test]
    fn test_bowing_gesture_legato_evaluation() {
        let engine = BowingGestureEngine::new();
        let snap0 = engine.evaluate_beat(0.5);
        assert_eq!(snap0.technique, BowingTechnique::Legato);
        assert!(snap0.bow_velocity_mps > 0.0);
        assert!((snap0.bow_force_n - 1.25).abs() < 1e-4);

        // Next stroke in second half should alternate direction
        let snap1 = engine.evaluate_beat(2.5);
        assert!(snap1.bow_velocity_mps < 0.0);
    }

    #[test]
    fn test_bowing_gesture_custom_spline_catmull_rom() {
        let mut engine = BowingGestureEngine {
            pattern: BowingGesturePattern::CustomSpline,
            interpolation: GestureInterpolationCurve::CubicCatmullRom,
            waypoints: Vec::new(),
            loop_length_beats: Some(4.0),
        };

        engine.add_waypoint(BowingWaypoint {
            beat: 0.0,
            bow_velocity_mps: 0.20,
            bow_force_n: 0.80,
            bridge_proximity_beta: 0.04, // Sul Ponticello
            technique: BowingTechnique::SulPonticello,
            ..Default::default()
        });

        engine.add_waypoint(BowingWaypoint {
            beat: 2.0,
            bow_velocity_mps: 0.60,
            bow_force_n: 1.80,
            bridge_proximity_beta: 0.12, // Normale
            technique: BowingTechnique::Legato,
            ..Default::default()
        });

        engine.add_waypoint(BowingWaypoint {
            beat: 4.0,
            bow_velocity_mps: 0.30,
            bow_force_n: 0.50,
            bridge_proximity_beta: 0.28, // Sul Tasto
            technique: BowingTechnique::SulTasto,
            ..Default::default()
        });

        let mid = engine.evaluate_beat(1.0);
        assert!(mid.bow_velocity_mps > 0.20 && mid.bow_velocity_mps < 0.60);
        assert!(mid.bridge_proximity_beta > 0.04 && mid.bridge_proximity_beta < 0.12);
    }

    #[test]
    fn test_bowing_gesture_dispatch_zero_allocation() {
        let engine = BowingGestureEngine::new();
        let bus = ArticulationBus::new();
        let _guard = AllocGuard::new();

        for i in 0..100 {
            let beat = i as f64 * 0.05;
            engine.dispatch_to_articulation_bus(beat, &bus);
        }
    }

    #[test]
    fn test_bowing_gesture_toml_roundtrip() {
        let mut engine = BowingGestureEngine::new();
        engine.pattern = BowingGesturePattern::PonticelloToTastoSweep {
            sweep_length_beats: 8.0,
            min_beta: 0.03,
            max_beta: 0.35,
            bow_velocity_mps: 0.50,
            bow_force_n: 1.10,
        };
        engine.add_waypoint(BowingWaypoint::default());

        let toml_str = toml::to_string_pretty(&engine).expect("Failed to serialize gesture engine");
        let decoded: BowingGestureEngine = toml::from_str(&toml_str).expect("Failed to deserialize gesture engine");

        assert_eq!(engine.waypoints.len(), decoded.waypoints.len());
        assert_eq!(engine.interpolation, decoded.interpolation);
    }
}
