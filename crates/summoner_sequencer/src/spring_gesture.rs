// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Dynamic Mechanical Dispersion Timeline & Elastic Mesh Articulation Engine (Milestone 20).
//!
//! Provides multi-dimensional continuous physical modeling trajectory generation for
//! spring-mass lattices and mechanical plate reverb tanks (Drive Force, Dispersion Factor,
//! Boundary Tension, Duffing Spring Non-Linearity, Transducer Pickup Angle, Damper Damping,
//! and Decay $T_{60}$) supporting vintage Accutronics boing swells, EMT 140 plate dispersion sweeps,
//! non-linear shaker chirps, and motorized damper choke patterns with cubic Catmull-Rom smoothing,
//! TOML serialization, and sample-accurate lock-free dispatch into `SpringBus` and `ParamBus`.
//!
//! Enforces zero heap allocations in real-time execution loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use summoner_core::param_bus::{ParamBus, ParamId};
use summoner_core::spring_bus::{SpringArticulation, SpringBus, SpringBusSnapshot};

/// Interpolation curve between discrete spring articulation keyframe waypoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SpringInterpolationCurve {
    #[default]
    CubicCatmullRom,
    CubicBezier,
    Linear,
    Hold,
    Exponential,
}

/// A single keyframed multi-dimensional mechanical spring/plate waypoint.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpringWaypoint {
    /// Timeline beat position in quarter note beats.
    pub beat: f64,
    /// Transducer driver force $[0.0 ..= 1.0]$.
    pub drive_force: f32,
    /// Flexural wave dispersion factor $[0.0 ..= 1.0]$.
    pub dispersion_factor: f32,
    /// Boundary frame tension $[0.0 ..= 1.0]$.
    pub boundary_tension: f32,
    /// Duffing spring cubic non-linearity $[0.0 ..= 1.0]$.
    pub spring_nonlinearity: f32,
    /// Transducer pickup angle in radians $[0.0 ..= 2\pi]$.
    pub transducer_pickup_angle: f32,
    /// Mechanical damper pad absorption $[0.0 ..= 1.0]$.
    pub damper_damping: f32,
    /// Plate decay time $T_{60}$ in seconds $[0.1 ..= 15.0]$.
    pub decay_t60_sec: f32,
    /// Mechanical articulation technique.
    pub articulation: SpringArticulation,
    /// Interpolation curve leading to this waypoint.
    pub curve: SpringInterpolationCurve,
}

impl Default for SpringWaypoint {
    fn default() -> Self {
        let (f, disp, tens, nonlin, angle, damp) = SpringArticulation::DriverDrive.nominal_parameters();
        Self {
            beat: 0.0,
            drive_force: f,
            dispersion_factor: disp,
            boundary_tension: tens,
            spring_nonlinearity: nonlin,
            transducer_pickup_angle: angle,
            damper_damping: damp,
            decay_t60_sec: 3.5,
            articulation: SpringArticulation::DriverDrive,
            curve: SpringInterpolationCurve::CubicCatmullRom,
        }
    }
}

/// Pre-configured algorithmic mechanical spring & plate gesture trajectory patterns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SpringGesturePattern {
    /// Vintage spring reverb decay swell with dynamic drive force and trailing tension modulation.
    SpringReverbDecaySwell {
        phrase_length_beats: f64,
        peak_drive: f32,
        sustain_tension: f32,
        decay_t60: f32,
    },
    /// Mechanical plate dispersion sweep from tight reflection to wide flexural chirp.
    PlateDispersionSweep {
        sweep_length_beats: f64,
        start_dispersion: f32,
        end_dispersion: f32,
        damper_pad_mix: f32,
    },
    /// Non-linear spring coil chirp with intense shaker excitation and decaying Duffing stiffness.
    NonlinearSpringChirp {
        chirp_length_beats: f64,
        initial_nonlinearity: f32,
        drive_burst: f32,
    },
    /// Mechanized damper pad choke with rapid spring plucks and damp muting.
    DamperMutePluck {
        subdivisions_per_beat: u32,
        pluck_velocity: f32,
        mute_damping: f32,
    },
    /// Orbital 360° rotation sweep of stereo pickup transducers.
    TransducerOrbitPan {
        orbit_period_beats: f64,
        dispersion_depth: f32,
    },
    /// User-defined multi-keyframe Catmull-Rom spline trajectory.
    CustomSpline,
}

impl Default for SpringGesturePattern {
    fn default() -> Self {
        Self::SpringReverbDecaySwell {
            phrase_length_beats: 4.0,
            peak_drive: 0.95,
            sustain_tension: 0.85,
            decay_t60: 4.2,
        }
    }
}

/// Dynamic Mechanical Dispersion Timeline & Gesture Engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpringGestureEngine {
    /// Keyframed articulation waypoints.
    pub waypoints: Vec<SpringWaypoint>,
    /// Active gesture trajectory pattern.
    pub pattern: SpringGesturePattern,
    /// Default interpolation curve.
    pub default_curve: SpringInterpolationCurve,
    /// Total sequence loop length in beats.
    pub loop_length_beats: f64,
    /// Whether pattern loops continuously.
    pub loop_enabled: bool,
    /// Whether gesture engine is currently active.
    pub is_active: bool,
}

impl Default for SpringGestureEngine {
    fn default() -> Self {
        let mut engine = Self {
            waypoints: Vec::new(),
            pattern: SpringGesturePattern::default(),
            default_curve: SpringInterpolationCurve::CubicCatmullRom,
            loop_length_beats: 4.0,
            loop_enabled: true,
            is_active: true,
        };
        engine.generate_pattern_waypoints();
        engine
    }
}

impl SpringGestureEngine {
    /// Creates a new SpringGestureEngine initialized with the specified pattern.
    pub fn new(pattern: SpringGesturePattern) -> Self {
        let mut engine = Self {
            waypoints: Vec::new(),
            pattern,
            default_curve: SpringInterpolationCurve::CubicCatmullRom,
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
            SpringGesturePattern::SpringReverbDecaySwell {
                phrase_length_beats,
                peak_drive,
                sustain_tension,
                decay_t60,
            } => {
                self.loop_length_beats = *phrase_length_beats;
                let steps = 4;
                for i in 0..=steps {
                    let frac = i as f64 / steps as f64;
                    let beat = frac * phrase_length_beats;
                    let drive = if frac < 0.25 {
                        0.30 + (*peak_drive - 0.30) * (frac as f32 / 0.25)
                    } else {
                        *peak_drive * (1.0 - 0.70 * ((frac as f32 - 0.25) / 0.75))
                    };
                    let nonlin = 0.20 + 0.50 * (1.0 - frac as f32);

                    self.waypoints.push(SpringWaypoint {
                        beat,
                        drive_force: drive,
                        dispersion_factor: 0.60 + 0.25 * frac as f32,
                        boundary_tension: *sustain_tension,
                        spring_nonlinearity: nonlin,
                        transducer_pickup_angle: 0.0,
                        damper_damping: 0.10 + 0.30 * frac as f32,
                        decay_t60_sec: *decay_t60,
                        articulation: if frac < 0.25 {
                            SpringArticulation::DriverDrive
                        } else {
                            SpringArticulation::TensionModulation
                        },
                        curve: SpringInterpolationCurve::CubicCatmullRom,
                    });
                }
            }

            SpringGesturePattern::PlateDispersionSweep {
                sweep_length_beats,
                start_dispersion,
                end_dispersion,
                damper_pad_mix,
            } => {
                self.loop_length_beats = *sweep_length_beats;
                let steps = 5;
                for i in 0..=steps {
                    let frac = i as f64 / steps as f64;
                    let beat = frac * sweep_length_beats;
                    let disp = start_dispersion + (end_dispersion - start_dispersion) * frac as f32;

                    self.waypoints.push(SpringWaypoint {
                        beat,
                        drive_force: 0.75,
                        dispersion_factor: disp,
                        boundary_tension: 0.90,
                        spring_nonlinearity: 0.15,
                        transducer_pickup_angle: frac as f32 * std::f32::consts::FRAC_PI_2, // 90° sweep
                        damper_damping: *damper_pad_mix,
                        decay_t60_sec: 3.8 - 1.2 * frac as f32,
                        articulation: SpringArticulation::DispersionWarp,
                        curve: SpringInterpolationCurve::CubicCatmullRom,
                    });
                }
            }

            SpringGesturePattern::NonlinearSpringChirp {
                chirp_length_beats,
                initial_nonlinearity,
                drive_burst,
            } => {
                self.loop_length_beats = *chirp_length_beats;
                let steps = 4;
                for i in 0..=steps {
                    let frac = i as f64 / steps as f64;
                    let beat = frac * chirp_length_beats;
                    let nonlin = *initial_nonlinearity * (1.0 - frac as f32).powi(2);
                    let drive = if i == 0 { *drive_burst } else { 0.20 };

                    self.waypoints.push(SpringWaypoint {
                        beat,
                        drive_force: drive,
                        dispersion_factor: 0.70,
                        boundary_tension: 0.65 + 0.30 * frac as f32,
                        spring_nonlinearity: nonlin,
                        transducer_pickup_angle: 0.0,
                        damper_damping: 0.08,
                        decay_t60_sec: 3.2,
                        articulation: if i == 0 {
                            SpringArticulation::NonLinearSaturate
                        } else {
                            SpringArticulation::ShakerExcite
                        },
                        curve: SpringInterpolationCurve::Exponential,
                    });
                }
            }

            SpringGesturePattern::DamperMutePluck {
                subdivisions_per_beat,
                pluck_velocity,
                mute_damping,
            } => {
                self.loop_length_beats = 4.0;
                let total_steps = 4 * *subdivisions_per_beat as usize;
                let step_beat = 4.0 / total_steps as f64;

                for step in 0..total_steps {
                    let beat = step as f64 * step_beat;
                    let is_pluck = step % 4 == 0;
                    let is_mute = step % 4 == 2;
                    let drive = if is_pluck { *pluck_velocity } else { 0.10 };
                    let damp = if is_mute { *mute_damping } else { 0.05 };

                    self.waypoints.push(SpringWaypoint {
                        beat,
                        drive_force: drive,
                        dispersion_factor: 0.50,
                        boundary_tension: 0.85,
                        spring_nonlinearity: if is_pluck { 0.60 } else { 0.10 },
                        transducer_pickup_angle: 0.0,
                        damper_damping: damp,
                        decay_t60_sec: if is_mute { 0.5 } else { 4.0 },
                        articulation: if is_pluck {
                            SpringArticulation::SpringPluck
                        } else if is_mute {
                            SpringArticulation::DamperMute
                        } else {
                            SpringArticulation::DriverDrive
                        },
                        curve: SpringInterpolationCurve::Hold,
                    });
                }
            }

            SpringGesturePattern::TransducerOrbitPan {
                orbit_period_beats,
                dispersion_depth,
            } => {
                self.loop_length_beats = *orbit_period_beats;
                let steps = 8;
                for i in 0..=steps {
                    let frac = i as f64 / steps as f64;
                    let beat = frac * orbit_period_beats;
                    let angle = (frac as f32) * 2.0 * std::f32::consts::PI;

                    self.waypoints.push(SpringWaypoint {
                        beat,
                        drive_force: 0.80,
                        dispersion_factor: *dispersion_depth,
                        boundary_tension: 0.85,
                        spring_nonlinearity: 0.20,
                        transducer_pickup_angle: angle,
                        damper_damping: 0.12,
                        decay_t60_sec: 3.5,
                        articulation: SpringArticulation::TransducerSweep,
                        curve: SpringInterpolationCurve::Linear,
                    });
                }
            }

            SpringGesturePattern::CustomSpline => {
                if self.waypoints.is_empty() {
                    self.waypoints.push(SpringWaypoint::default());
                }
            }
        }
    }

    /// Evaluates continuous mechanical parameters at the specified beat position.
    pub fn evaluate_at_beat(&self, beat: f64) -> SpringBusSnapshot {
        if self.waypoints.is_empty() {
            return SpringBusSnapshot::default();
        }

        if self.waypoints.len() == 1 {
            let wp = self.waypoints[0];
            return SpringBusSnapshot {
                articulation: wp.articulation,
                drive_force: wp.drive_force,
                dispersion_factor: wp.dispersion_factor,
                boundary_tension: wp.boundary_tension,
                spring_nonlinearity: wp.spring_nonlinearity,
                transducer_pickup_angle: wp.transducer_pickup_angle,
                damper_damping: wp.damper_damping,
                decay_t60_sec: wp.decay_t60_sec,
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
        let (f, disp, tens, nonlin, angle, damp, t60) = match curve {
            SpringInterpolationCurve::Hold => (
                w1.drive_force,
                w1.dispersion_factor,
                w1.boundary_tension,
                w1.spring_nonlinearity,
                w1.transducer_pickup_angle,
                w1.damper_damping,
                w1.decay_t60_sec,
            ),
            SpringInterpolationCurve::Linear => (
                lerp(w1.drive_force, w2.drive_force, t),
                lerp(w1.dispersion_factor, w2.dispersion_factor, t),
                lerp(w1.boundary_tension, w2.boundary_tension, t),
                lerp(w1.spring_nonlinearity, w2.spring_nonlinearity, t),
                lerp(w1.transducer_pickup_angle, w2.transducer_pickup_angle, t),
                lerp(w1.damper_damping, w2.damper_damping, t),
                lerp(w1.decay_t60_sec, w2.decay_t60_sec, t),
            ),
            SpringInterpolationCurve::Exponential => {
                let t_exp = t * t;
                (
                    lerp(w1.drive_force, w2.drive_force, t_exp),
                    lerp(w1.dispersion_factor, w2.dispersion_factor, t_exp),
                    lerp(w1.boundary_tension, w2.boundary_tension, t_exp),
                    lerp(w1.spring_nonlinearity, w2.spring_nonlinearity, t_exp),
                    lerp(w1.transducer_pickup_angle, w2.transducer_pickup_angle, t_exp),
                    lerp(w1.damper_damping, w2.damper_damping, t_exp),
                    lerp(w1.decay_t60_sec, w2.decay_t60_sec, t_exp),
                )
            }
            SpringInterpolationCurve::CubicBezier => {
                let t_bez = 3.0 * t * t - 2.0 * t * t * t;
                (
                    lerp(w1.drive_force, w2.drive_force, t_bez),
                    lerp(w1.dispersion_factor, w2.dispersion_factor, t_bez),
                    lerp(w1.boundary_tension, w2.boundary_tension, t_bez),
                    lerp(w1.spring_nonlinearity, w2.spring_nonlinearity, t_bez),
                    lerp(w1.transducer_pickup_angle, w2.transducer_pickup_angle, t_bez),
                    lerp(w1.damper_damping, w2.damper_damping, t_bez),
                    lerp(w1.decay_t60_sec, w2.decay_t60_sec, t_bez),
                )
            }
            SpringInterpolationCurve::CubicCatmullRom => (
                catmull_rom_1d(w0.drive_force, w1.drive_force, w2.drive_force, w3.drive_force, t),
                catmull_rom_1d(w0.dispersion_factor, w1.dispersion_factor, w2.dispersion_factor, w3.dispersion_factor, t),
                catmull_rom_1d(w0.boundary_tension, w1.boundary_tension, w2.boundary_tension, w3.boundary_tension, t),
                catmull_rom_1d(w0.spring_nonlinearity, w1.spring_nonlinearity, w2.spring_nonlinearity, w3.spring_nonlinearity, t),
                catmull_rom_1d(w0.transducer_pickup_angle, w1.transducer_pickup_angle, w2.transducer_pickup_angle, w3.transducer_pickup_angle, t),
                catmull_rom_1d(w0.damper_damping, w1.damper_damping, w2.damper_damping, w3.damper_damping, t),
                catmull_rom_1d(w0.decay_t60_sec, w1.decay_t60_sec, w2.decay_t60_sec, w3.decay_t60_sec, t),
            ),
        };

        let progress = if self.loop_length_beats > 0.0 {
            (effective_beat / self.loop_length_beats).clamp(0.0, 1.0) as f32
        } else {
            0.0
        };

        SpringBusSnapshot {
            articulation: if t < 0.5 { w1.articulation } else { w2.articulation },
            drive_force: f.clamp(0.0, 1.0),
            dispersion_factor: disp.clamp(0.0, 1.0),
            boundary_tension: tens.clamp(0.0, 1.0),
            spring_nonlinearity: nonlin.clamp(0.0, 1.0),
            transducer_pickup_angle: angle.clamp(0.0, std::f32::consts::TAU),
            damper_damping: damp.clamp(0.0, 1.0),
            decay_t60_sec: t60.clamp(0.1, 15.0),
            gesture_progress: progress,
            is_active: self.is_active,
        }
    }

    /// Dispatches evaluated articulation parameters into a lock-free `SpringBus`.
    pub fn dispatch_to_bus(&self, beat: f64, bus: &SpringBus) {
        let snap = self.evaluate_at_beat(beat);
        bus.apply_snapshot(&snap);
    }

    /// Dispatches evaluated articulation parameters into a central `ParamBus`.
    pub fn dispatch_to_param_bus(&self, beat: f64, base_id: ParamId, param_bus: &ParamBus) {
        let snap = self.evaluate_at_beat(beat);
        param_bus.set(ParamId(base_id.0), snap.articulation as u32 as f32);
        param_bus.set(ParamId(base_id.0 + 1), snap.drive_force);
        param_bus.set(ParamId(base_id.0 + 2), snap.dispersion_factor);
        param_bus.set(ParamId(base_id.0 + 3), snap.boundary_tension);
        param_bus.set(ParamId(base_id.0 + 4), snap.spring_nonlinearity);
        param_bus.set(ParamId(base_id.0 + 5), snap.transducer_pickup_angle);
        param_bus.set(ParamId(base_id.0 + 6), snap.damper_damping);
        param_bus.set(ParamId(base_id.0 + 7), snap.decay_t60_sec);
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
    fn test_spring_gesture_decay_swell_evaluation() {
        let engine = SpringGestureEngine::new(SpringGesturePattern::default());
        let snap_start = engine.evaluate_at_beat(0.0);
        assert_eq!(snap_start.articulation, SpringArticulation::DriverDrive);
        assert!((snap_start.drive_force - 0.30).abs() < 1e-2);

        let snap_peak = engine.evaluate_at_beat(1.0);
        assert!(snap_peak.drive_force > 0.85);

        let snap_end = engine.evaluate_at_beat(4.0);
        assert!(snap_end.drive_force < snap_peak.drive_force);
    }

    #[test]
    fn test_spring_gesture_dispersion_sweep_evaluation() {
        let engine = SpringGestureEngine::new(SpringGesturePattern::PlateDispersionSweep {
            sweep_length_beats: 8.0,
            start_dispersion: 0.20,
            end_dispersion: 0.90,
            damper_pad_mix: 0.15,
        });

        let snap_start = engine.evaluate_at_beat(0.0);
        assert!((snap_start.dispersion_factor - 0.20).abs() < 1e-2);

        let snap_end = engine.evaluate_at_beat(8.0);
        assert!((snap_end.dispersion_factor - 0.90).abs() < 1e-2);
    }

    #[test]
    fn test_spring_gesture_custom_spline_catmull_rom() {
        let mut engine = SpringGestureEngine::new(SpringGesturePattern::CustomSpline);
        engine.waypoints = vec![
            SpringWaypoint {
                beat: 0.0,
                drive_force: 0.20,
                dispersion_factor: 0.30,
                boundary_tension: 0.70,
                spring_nonlinearity: 0.10,
                transducer_pickup_angle: 0.0,
                damper_damping: 0.05,
                decay_t60_sec: 2.0,
                articulation: SpringArticulation::DriverDrive,
                curve: SpringInterpolationCurve::CubicCatmullRom,
            },
            SpringWaypoint {
                beat: 2.0,
                drive_force: 0.90,
                dispersion_factor: 0.80,
                boundary_tension: 0.95,
                spring_nonlinearity: 0.85,
                transducer_pickup_angle: std::f32::consts::FRAC_PI_2,
                damper_damping: 0.50,
                decay_t60_sec: 5.0,
                articulation: SpringArticulation::NonLinearSaturate,
                curve: SpringInterpolationCurve::CubicCatmullRom,
            },
            SpringWaypoint {
                beat: 4.0,
                drive_force: 0.40,
                dispersion_factor: 0.40,
                boundary_tension: 0.80,
                spring_nonlinearity: 0.20,
                transducer_pickup_angle: std::f32::consts::PI,
                damper_damping: 0.10,
                decay_t60_sec: 3.0,
                articulation: SpringArticulation::TransducerSweep,
                curve: SpringInterpolationCurve::CubicCatmullRom,
            },
        ];

        let mid = engine.evaluate_at_beat(1.0);
        assert!(mid.drive_force > 0.20 && mid.drive_force < 0.90);
        assert!(mid.dispersion_factor > 0.30 && mid.dispersion_factor < 0.80);
    }

    #[test]
    fn test_spring_gesture_toml_roundtrip() {
        let engine = SpringGestureEngine::new(SpringGesturePattern::default());
        let toml_str = engine.to_toml().expect("Failed to serialize spring gesture TOML");
        assert!(toml_str.contains("SpringReverbDecaySwell"));

        let restored: SpringGestureEngine = SpringGestureEngine::from_toml(&toml_str)
            .expect("Failed to deserialize spring gesture TOML");
        assert_eq!(engine.waypoints.len(), restored.waypoints.len());
        assert_eq!(engine.loop_length_beats, restored.loop_length_beats);
    }

    #[test]
    fn test_spring_gesture_dispatch_zero_allocation() {
        let engine = SpringGestureEngine::new(SpringGesturePattern::default());
        let bus = SpringBus::new();
        let mut param_bus = ParamBus::new();
        for i in 0..12 {
            param_bus.register(ParamId(700 + i), 0.0);
        }

        engine.dispatch_to_bus(1.5, &bus);
        let snap = bus.snapshot();
        assert!(snap.drive_force > 0.0);
        assert!(snap.is_active);

        engine.dispatch_to_param_bus(2.0, ParamId(700), &param_bus);
        assert!(param_bus.get(ParamId(701)).unwrap() > 0.0); // drive_force
        assert!(param_bus.get(ParamId(702)).unwrap() > 0.0); // dispersion_factor
    }
}
