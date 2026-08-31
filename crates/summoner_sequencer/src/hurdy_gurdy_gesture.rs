// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Dynamic Hurdy-Gurdy (Vielle à roue) Gesture Timeline & Articulation Engine (Milestone 30).
//!
//! Provides multi-dimensional continuous physical modeling trajectory generation for
//! Crank Wheel Angular Velocity $\omega \in [0..4\pi]\text{ rad/s}$, Coup de Poignet Wrist
//! Acceleration Pulses $\Delta \alpha$, Chien Buzzing Dog Clearance $h_0 \in [0.05..1.20]\text{ mm}$,
//! Tangent Key Damping, Drone/Melody Mix Ratios, Trompette Buzz Levels, and Soundbox Resonances
//! with Catmull-Rom spline smoothing, TOML serialization, and sample-accurate lock-free parameter
//! dispatch into `HurdyGurdyBus` and `ParamBus`.
//!
//! Enforces zero heap allocations in real-time execution loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use summoner_core::hurdy_gurdy_bus::{
    HurdyGurdyArticulation, HurdyGurdyBus, HurdyGurdyBusSnapshot,
};
use summoner_core::param_bus::{ParamBus, ParamId};

/// Interpolation curve between discrete Hurdy-Gurdy keyframe waypoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum HurdyGurdyInterpolationCurve {
    #[default]
    CubicCatmullRom,
    CubicBezier,
    Linear,
    Hold,
    Exponential,
}

/// A single keyframed multi-dimensional Hurdy-Gurdy articulation waypoint.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HurdyGurdyWaypoint {
    /// Timeline beat position in quarter note beats.
    pub beat: f64,
    /// Crank wheel angular velocity $[0.0 ..= 4.0\pi]\text{ rad/s}$.
    pub crank_speed_rad_s: f32,
    /// Coup de poignet wrist acceleration impulse $[0.0 ..= 10.0]$.
    pub wrist_acceleration_pulse: f32,
    /// Wheel normal pressure force $[0.01 ..= 1.0]$.
    pub wheel_pressure: f32,
    /// Chien buzzing dog resting clearance gap in mm $[0.05 ..= 1.20]\text{ mm}$.
    pub chien_clearance_mm: f32,
    /// Tangent key action damping $[0.0 ..= 1.0]$.
    pub tangent_damping: f32,
    /// Drone vs Chanterelle mix ratio $[0.0 ..= 1.0]$.
    pub drone_melody_mix: f32,
    /// Trompette buzzing bridge volume level $[0.0 ..= 2.0]$.
    pub trompette_buzz_level: f32,
    /// Soundbox body wood resonance gain $[0.0 ..= 1.0]$.
    pub body_wood_resonance: f32,
    /// Master output gain $[0.0 ..= 2.0]$.
    pub master_gain: f32,
    /// Articulation technique.
    pub articulation: HurdyGurdyArticulation,
    /// Interpolation curve leading to this waypoint.
    pub curve: HurdyGurdyInterpolationCurve,
}

impl Default for HurdyGurdyWaypoint {
    fn default() -> Self {
        let (speed, wrist, press, gap, damp, dmix, buzz, wood, mgain) =
            HurdyGurdyArticulation::BourbonnaisClassicCoupDePoignet.nominal_parameters();

        Self {
            beat: 0.0,
            crank_speed_rad_s: speed,
            wrist_acceleration_pulse: wrist,
            wheel_pressure: press,
            chien_clearance_mm: gap,
            tangent_damping: damp,
            drone_melody_mix: dmix,
            trompette_buzz_level: buzz,
            body_wood_resonance: wood,
            master_gain: mgain,
            articulation: HurdyGurdyArticulation::BourbonnaisClassicCoupDePoignet,
            curve: HurdyGurdyInterpolationCurve::CubicCatmullRom,
        }
    }
}

/// Pre-configured algorithmic Hurdy-Gurdy gesture trajectory patterns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HurdyGurdyGesturePattern {
    /// Bourbonnais Coup de Poignet with rhythmic buzzing wrist snaps on the beats.
    BourbonnaisClassicCoupDePoignet {
        phrase_length_beats: f64,
        pulse_intensity: f32,
    },
    /// Medieval Monastic Drone with smooth sustained wheel motion and zero buzz.
    MedievalMonasticDrone {
        phrase_length_beats: f64,
    },
    /// Baroque Virtuoso Chanterelle with fast melody transitions and subtle buzzing accents.
    BaroqueVirtuosoChanterelle {
        phrase_length_beats: f64,
    },
    /// Auvergne High-Speed Buzz with driving continuous dog rattling.
    AuvergneHighSpeedBuzz {
        phrase_length_beats: f64,
        crank_speed_multiplier: f32,
    },
    /// Gothic Pagan Dark Drone with heavy low drones and syncopated buzz strikes.
    GothicPaganDrone {
        phrase_length_beats: f64,
    },
    /// Electro-Acoustic Hybrid Swell with wide wheel acceleration ramps.
    ElectroAcousticHybrid {
        phrase_length_beats: f64,
    },
}

/// Dynamic Hurdy-Gurdy articulation timeline and parameter trajectory engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HurdyGurdyGestureEngine {
    pub waypoints: Vec<HurdyGurdyWaypoint>,
    pub loop_length_beats: f64,
    pub is_looping: bool,
}

impl Default for HurdyGurdyGestureEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl HurdyGurdyGestureEngine {
    pub fn new() -> Self {
        Self {
            waypoints: Vec::new(),
            loop_length_beats: 4.0,
            is_looping: true,
        }
    }

    pub fn with_pattern(pattern: HurdyGurdyGesturePattern) -> Self {
        let mut engine = Self::new();
        engine.load_pattern(pattern);
        engine
    }

    pub fn add_waypoint(&mut self, waypoint: HurdyGurdyWaypoint) {
        self.waypoints.push(waypoint);
        self.sort_waypoints();
    }

    pub fn sort_waypoints(&mut self) {
        self.waypoints.sort_by(|a, b| a.beat.partial_cmp(&b.beat).unwrap_or(std::cmp::Ordering::Equal));
    }

    pub fn clear(&mut self) {
        self.waypoints.clear();
    }

    pub fn load_pattern(&mut self, pattern: HurdyGurdyGesturePattern) {
        self.clear();

        match &pattern {
            HurdyGurdyGesturePattern::BourbonnaisClassicCoupDePoignet { phrase_length_beats, pulse_intensity } => {
                self.loop_length_beats = *phrase_length_beats;
                let intensity = *pulse_intensity;

                self.waypoints.push(HurdyGurdyWaypoint {
                    beat: 0.0,
                    crank_speed_rad_s: 2.0 * PI,
                    wrist_acceleration_pulse: intensity * 1.5, // Initial Coup de poignet strike
                    wheel_pressure: 0.70,
                    chien_clearance_mm: 0.35,
                    tangent_damping: 0.85,
                    drone_melody_mix: 0.45,
                    trompette_buzz_level: 0.90,
                    body_wood_resonance: 0.85,
                    master_gain: 0.90,
                    articulation: HurdyGurdyArticulation::BourbonnaisClassicCoupDePoignet,
                    curve: HurdyGurdyInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(HurdyGurdyWaypoint {
                    beat: phrase_length_beats * 0.25,
                    crank_speed_rad_s: 2.0 * PI,
                    wrist_acceleration_pulse: 0.0, // Sustained glide
                    wheel_pressure: 0.65,
                    chien_clearance_mm: 0.35,
                    tangent_damping: 0.85,
                    drone_melody_mix: 0.45,
                    trompette_buzz_level: 0.80,
                    body_wood_resonance: 0.85,
                    master_gain: 0.90,
                    articulation: HurdyGurdyArticulation::BourbonnaisClassicCoupDePoignet,
                    curve: HurdyGurdyInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(HurdyGurdyWaypoint {
                    beat: phrase_length_beats * 0.50,
                    crank_speed_rad_s: 2.2 * PI,
                    wrist_acceleration_pulse: intensity * 1.8, // Second strong Coup de poignet
                    wheel_pressure: 0.75,
                    chien_clearance_mm: 0.30,
                    tangent_damping: 0.85,
                    drone_melody_mix: 0.48,
                    trompette_buzz_level: 0.95,
                    body_wood_resonance: 0.88,
                    master_gain: 0.92,
                    articulation: HurdyGurdyArticulation::BourbonnaisClassicCoupDePoignet,
                    curve: HurdyGurdyInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(HurdyGurdyWaypoint {
                    beat: phrase_length_beats * 0.75,
                    crank_speed_rad_s: 2.0 * PI,
                    wrist_acceleration_pulse: 0.0,
                    wheel_pressure: 0.68,
                    chien_clearance_mm: 0.35,
                    tangent_damping: 0.85,
                    drone_melody_mix: 0.45,
                    trompette_buzz_level: 0.85,
                    body_wood_resonance: 0.85,
                    master_gain: 0.90,
                    articulation: HurdyGurdyArticulation::BourbonnaisClassicCoupDePoignet,
                    curve: HurdyGurdyInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(HurdyGurdyWaypoint {
                    beat: *phrase_length_beats,
                    crank_speed_rad_s: 2.0 * PI,
                    wrist_acceleration_pulse: intensity * 1.5,
                    wheel_pressure: 0.70,
                    chien_clearance_mm: 0.35,
                    tangent_damping: 0.85,
                    drone_melody_mix: 0.45,
                    trompette_buzz_level: 0.90,
                    body_wood_resonance: 0.85,
                    master_gain: 0.90,
                    articulation: HurdyGurdyArticulation::BourbonnaisClassicCoupDePoignet,
                    curve: HurdyGurdyInterpolationCurve::CubicCatmullRom,
                });
            }

            HurdyGurdyGesturePattern::MedievalMonasticDrone { phrase_length_beats } => {
                self.loop_length_beats = *phrase_length_beats;

                self.waypoints.push(HurdyGurdyWaypoint {
                    beat: 0.0,
                    crank_speed_rad_s: 1.5 * PI,
                    wrist_acceleration_pulse: 0.0,
                    wheel_pressure: 0.55,
                    chien_clearance_mm: 0.90,
                    tangent_damping: 0.95,
                    drone_melody_mix: 0.60,
                    trompette_buzz_level: 0.20,
                    body_wood_resonance: 0.80,
                    master_gain: 0.88,
                    articulation: HurdyGurdyArticulation::MedievalMonasticDrone,
                    curve: HurdyGurdyInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(HurdyGurdyWaypoint {
                    beat: phrase_length_beats * 0.50,
                    crank_speed_rad_s: 1.6 * PI,
                    wrist_acceleration_pulse: 0.0,
                    wheel_pressure: 0.58,
                    chien_clearance_mm: 0.90,
                    tangent_damping: 0.95,
                    drone_melody_mix: 0.62,
                    trompette_buzz_level: 0.20,
                    body_wood_resonance: 0.82,
                    master_gain: 0.88,
                    articulation: HurdyGurdyArticulation::MedievalMonasticDrone,
                    curve: HurdyGurdyInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(HurdyGurdyWaypoint {
                    beat: *phrase_length_beats,
                    crank_speed_rad_s: 1.5 * PI,
                    wrist_acceleration_pulse: 0.0,
                    wheel_pressure: 0.55,
                    chien_clearance_mm: 0.90,
                    tangent_damping: 0.95,
                    drone_melody_mix: 0.60,
                    trompette_buzz_level: 0.20,
                    body_wood_resonance: 0.80,
                    master_gain: 0.88,
                    articulation: HurdyGurdyArticulation::MedievalMonasticDrone,
                    curve: HurdyGurdyInterpolationCurve::CubicCatmullRom,
                });
            }

            HurdyGurdyGesturePattern::BaroqueVirtuosoChanterelle { phrase_length_beats } => {
                self.loop_length_beats = *phrase_length_beats;

                self.waypoints.push(HurdyGurdyWaypoint {
                    beat: 0.0,
                    crank_speed_rad_s: 2.2 * PI,
                    wrist_acceleration_pulse: 0.4,
                    wheel_pressure: 0.80,
                    chien_clearance_mm: 0.25,
                    tangent_damping: 0.75,
                    drone_melody_mix: 0.35,
                    trompette_buzz_level: 0.95,
                    body_wood_resonance: 0.90,
                    master_gain: 0.92,
                    articulation: HurdyGurdyArticulation::BaroqueVirtuosoChanterelle,
                    curve: HurdyGurdyInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(HurdyGurdyWaypoint {
                    beat: phrase_length_beats * 0.50,
                    crank_speed_rad_s: 2.5 * PI,
                    wrist_acceleration_pulse: 0.8,
                    wheel_pressure: 0.85,
                    chien_clearance_mm: 0.22,
                    tangent_damping: 0.70,
                    drone_melody_mix: 0.30,
                    trompette_buzz_level: 1.05,
                    body_wood_resonance: 0.92,
                    master_gain: 0.94,
                    articulation: HurdyGurdyArticulation::BaroqueVirtuosoChanterelle,
                    curve: HurdyGurdyInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(HurdyGurdyWaypoint {
                    beat: *phrase_length_beats,
                    crank_speed_rad_s: 2.2 * PI,
                    wrist_acceleration_pulse: 0.4,
                    wheel_pressure: 0.80,
                    chien_clearance_mm: 0.25,
                    tangent_damping: 0.75,
                    drone_melody_mix: 0.35,
                    trompette_buzz_level: 0.95,
                    body_wood_resonance: 0.90,
                    master_gain: 0.92,
                    articulation: HurdyGurdyArticulation::BaroqueVirtuosoChanterelle,
                    curve: HurdyGurdyInterpolationCurve::CubicCatmullRom,
                });
            }

            HurdyGurdyGesturePattern::AuvergneHighSpeedBuzz { phrase_length_beats, crank_speed_multiplier } => {
                self.loop_length_beats = *phrase_length_beats;
                let mult = *crank_speed_multiplier;

                self.waypoints.push(HurdyGurdyWaypoint {
                    beat: 0.0,
                    crank_speed_rad_s: 3.0 * PI * mult,
                    wrist_acceleration_pulse: 2.5,
                    wheel_pressure: 0.90,
                    chien_clearance_mm: 0.15,
                    tangent_damping: 0.80,
                    drone_melody_mix: 0.50,
                    trompette_buzz_level: 1.30,
                    body_wood_resonance: 0.85,
                    master_gain: 0.95,
                    articulation: HurdyGurdyArticulation::AuvergneHighSpeedBuzz,
                    curve: HurdyGurdyInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(HurdyGurdyWaypoint {
                    beat: phrase_length_beats * 0.50,
                    crank_speed_rad_s: 3.5 * PI * mult,
                    wrist_acceleration_pulse: 3.2,
                    wheel_pressure: 0.95,
                    chien_clearance_mm: 0.12,
                    tangent_damping: 0.78,
                    drone_melody_mix: 0.55,
                    trompette_buzz_level: 1.45,
                    body_wood_resonance: 0.90,
                    master_gain: 0.96,
                    articulation: HurdyGurdyArticulation::AuvergneHighSpeedBuzz,
                    curve: HurdyGurdyInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(HurdyGurdyWaypoint {
                    beat: *phrase_length_beats,
                    crank_speed_rad_s: 3.0 * PI * mult,
                    wrist_acceleration_pulse: 2.5,
                    wheel_pressure: 0.90,
                    chien_clearance_mm: 0.15,
                    tangent_damping: 0.80,
                    drone_melody_mix: 0.50,
                    trompette_buzz_level: 1.30,
                    body_wood_resonance: 0.85,
                    master_gain: 0.95,
                    articulation: HurdyGurdyArticulation::AuvergneHighSpeedBuzz,
                    curve: HurdyGurdyInterpolationCurve::CubicCatmullRom,
                });
            }

            HurdyGurdyGesturePattern::GothicPaganDrone { phrase_length_beats } => {
                self.loop_length_beats = *phrase_length_beats;

                self.waypoints.push(HurdyGurdyWaypoint {
                    beat: 0.0,
                    crank_speed_rad_s: 1.2 * PI,
                    wrist_acceleration_pulse: 0.8,
                    wheel_pressure: 0.65,
                    chien_clearance_mm: 0.45,
                    tangent_damping: 0.90,
                    drone_melody_mix: 0.70,
                    trompette_buzz_level: 0.75,
                    body_wood_resonance: 0.95,
                    master_gain: 0.90,
                    articulation: HurdyGurdyArticulation::GothicPaganDrone,
                    curve: HurdyGurdyInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(HurdyGurdyWaypoint {
                    beat: phrase_length_beats * 0.50,
                    crank_speed_rad_s: 1.4 * PI,
                    wrist_acceleration_pulse: 1.2,
                    wheel_pressure: 0.70,
                    chien_clearance_mm: 0.40,
                    tangent_damping: 0.88,
                    drone_melody_mix: 0.72,
                    trompette_buzz_level: 0.85,
                    body_wood_resonance: 0.98,
                    master_gain: 0.92,
                    articulation: HurdyGurdyArticulation::GothicPaganDrone,
                    curve: HurdyGurdyInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(HurdyGurdyWaypoint {
                    beat: *phrase_length_beats,
                    crank_speed_rad_s: 1.2 * PI,
                    wrist_acceleration_pulse: 0.8,
                    wheel_pressure: 0.65,
                    chien_clearance_mm: 0.45,
                    tangent_damping: 0.90,
                    drone_melody_mix: 0.70,
                    trompette_buzz_level: 0.75,
                    body_wood_resonance: 0.95,
                    master_gain: 0.90,
                    articulation: HurdyGurdyArticulation::GothicPaganDrone,
                    curve: HurdyGurdyInterpolationCurve::CubicCatmullRom,
                });
            }

            HurdyGurdyGesturePattern::ElectroAcousticHybrid { phrase_length_beats } => {
                self.loop_length_beats = *phrase_length_beats;

                self.waypoints.push(HurdyGurdyWaypoint {
                    beat: 0.0,
                    crank_speed_rad_s: 2.5 * PI,
                    wrist_acceleration_pulse: 1.5,
                    wheel_pressure: 0.85,
                    chien_clearance_mm: 0.30,
                    tangent_damping: 0.80,
                    drone_melody_mix: 0.40,
                    trompette_buzz_level: 1.00,
                    body_wood_resonance: 0.90,
                    master_gain: 0.92,
                    articulation: HurdyGurdyArticulation::ElectroAcousticHybrid,
                    curve: HurdyGurdyInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(HurdyGurdyWaypoint {
                    beat: phrase_length_beats * 0.50,
                    crank_speed_rad_s: 3.2 * PI,
                    wrist_acceleration_pulse: 2.0,
                    wheel_pressure: 0.90,
                    chien_clearance_mm: 0.25,
                    tangent_damping: 0.75,
                    drone_melody_mix: 0.42,
                    trompette_buzz_level: 1.15,
                    body_wood_resonance: 0.92,
                    master_gain: 0.94,
                    articulation: HurdyGurdyArticulation::ElectroAcousticHybrid,
                    curve: HurdyGurdyInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(HurdyGurdyWaypoint {
                    beat: *phrase_length_beats,
                    crank_speed_rad_s: 2.5 * PI,
                    wrist_acceleration_pulse: 1.5,
                    wheel_pressure: 0.85,
                    chien_clearance_mm: 0.30,
                    tangent_damping: 0.80,
                    drone_melody_mix: 0.40,
                    trompette_buzz_level: 1.00,
                    body_wood_resonance: 0.90,
                    master_gain: 0.92,
                    articulation: HurdyGurdyArticulation::ElectroAcousticHybrid,
                    curve: HurdyGurdyInterpolationCurve::CubicCatmullRom,
                });
            }
        }
    }

    /// Evaluate timeline trajectory at arbitrary beat and return interpolated snapshot.
    pub fn evaluate_at_beat(&self, beat: f64) -> HurdyGurdyBusSnapshot {
        if self.waypoints.is_empty() {
            let (speed, wrist, press, gap, damp, dmix, buzz, wood, mgain) =
                HurdyGurdyArticulation::BourbonnaisClassicCoupDePoignet.nominal_parameters();
            return HurdyGurdyBusSnapshot {
                articulation: HurdyGurdyArticulation::BourbonnaisClassicCoupDePoignet,
                crank_speed_rad_s: speed,
                wrist_acceleration_pulse: wrist,
                wheel_pressure: press,
                chien_clearance_mm: gap,
                tangent_damping: damp,
                drone_melody_mix: dmix,
                trompette_buzz_level: buzz,
                body_wood_resonance: wood,
                master_gain: mgain,
            };
        }

        if self.waypoints.len() == 1 {
            let wp = &self.waypoints[0];
            return HurdyGurdyBusSnapshot {
                articulation: wp.articulation,
                crank_speed_rad_s: wp.crank_speed_rad_s,
                wrist_acceleration_pulse: wp.wrist_acceleration_pulse,
                wheel_pressure: wp.wheel_pressure,
                chien_clearance_mm: wp.chien_clearance_mm,
                tangent_damping: wp.tangent_damping,
                drone_melody_mix: wp.drone_melody_mix,
                trompette_buzz_level: wp.trompette_buzz_level,
                body_wood_resonance: wp.body_wood_resonance,
                master_gain: wp.master_gain,
            };
        }

        let effective_beat = if self.is_looping && self.loop_length_beats > 0.0 {
            beat.rem_euclid(self.loop_length_beats)
        } else {
            beat
        };

        let n = self.waypoints.len();
        let mut idx1 = 0;
        while idx1 < n && self.waypoints[idx1].beat <= effective_beat {
            idx1 += 1;
        }

        if idx1 == 0 {
            let wp = &self.waypoints[0];
            return HurdyGurdyBusSnapshot {
                articulation: wp.articulation,
                crank_speed_rad_s: wp.crank_speed_rad_s,
                wrist_acceleration_pulse: wp.wrist_acceleration_pulse,
                wheel_pressure: wp.wheel_pressure,
                chien_clearance_mm: wp.chien_clearance_mm,
                tangent_damping: wp.tangent_damping,
                drone_melody_mix: wp.drone_melody_mix,
                trompette_buzz_level: wp.trompette_buzz_level,
                body_wood_resonance: wp.body_wood_resonance,
                master_gain: wp.master_gain,
            };
        }

        if idx1 >= n {
            let wp = &self.waypoints[n - 1];
            return HurdyGurdyBusSnapshot {
                articulation: wp.articulation,
                crank_speed_rad_s: wp.crank_speed_rad_s,
                wrist_acceleration_pulse: wp.wrist_acceleration_pulse,
                wheel_pressure: wp.wheel_pressure,
                chien_clearance_mm: wp.chien_clearance_mm,
                tangent_damping: wp.tangent_damping,
                drone_melody_mix: wp.drone_melody_mix,
                trompette_buzz_level: wp.trompette_buzz_level,
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

        let interp = |v0: f32, v1: f32, v2: f32, v3: f32, curve: HurdyGurdyInterpolationCurve| -> f32 {
            match curve {
                HurdyGurdyInterpolationCurve::Hold => v1,
                HurdyGurdyInterpolationCurve::Linear => v1 + t * (v2 - v1),
                HurdyGurdyInterpolationCurve::Exponential => {
                    let min_v = v1.min(v2).max(1e-5);
                    let ratio = (v2.max(1e-5) / min_v).max(1e-5);
                    v1 * ratio.powf(t)
                }
                HurdyGurdyInterpolationCurve::CubicBezier => {
                    let t2 = t * t;
                    let t3 = t2 * t;
                    let b = 3.0 * t2 - 2.0 * t3;
                    v1 + b * (v2 - v1)
                }
                HurdyGurdyInterpolationCurve::CubicCatmullRom => {
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
        let speed = interp(wp0.crank_speed_rad_s, wp1.crank_speed_rad_s, wp2.crank_speed_rad_s, wp3.crank_speed_rad_s, curve).clamp(0.0, 4.0 * PI);
        let wrist = interp(wp0.wrist_acceleration_pulse, wp1.wrist_acceleration_pulse, wp2.wrist_acceleration_pulse, wp3.wrist_acceleration_pulse, curve).clamp(0.0, 10.0);
        let press = interp(wp0.wheel_pressure, wp1.wheel_pressure, wp2.wheel_pressure, wp3.wheel_pressure, curve).clamp(0.01, 1.0);
        let gap = interp(wp0.chien_clearance_mm, wp1.chien_clearance_mm, wp2.chien_clearance_mm, wp3.chien_clearance_mm, curve).clamp(0.05, 1.20);
        let damp = interp(wp0.tangent_damping, wp1.tangent_damping, wp2.tangent_damping, wp3.tangent_damping, curve).clamp(0.0, 1.0);
        let dmix = interp(wp0.drone_melody_mix, wp1.drone_melody_mix, wp2.drone_melody_mix, wp3.drone_melody_mix, curve).clamp(0.0, 1.0);
        let buzz = interp(wp0.trompette_buzz_level, wp1.trompette_buzz_level, wp2.trompette_buzz_level, wp3.trompette_buzz_level, curve).clamp(0.0, 2.0);
        let wood = interp(wp0.body_wood_resonance, wp1.body_wood_resonance, wp2.body_wood_resonance, wp3.body_wood_resonance, curve).clamp(0.0, 1.0);
        let mgain = interp(wp0.master_gain, wp1.master_gain, wp2.master_gain, wp3.master_gain, curve).clamp(0.0, 2.0);

        HurdyGurdyBusSnapshot {
            articulation: if t > 0.5 { wp2.articulation } else { wp1.articulation },
            crank_speed_rad_s: speed,
            wrist_acceleration_pulse: wrist,
            wheel_pressure: press,
            chien_clearance_mm: gap,
            tangent_damping: damp,
            drone_melody_mix: dmix,
            trompette_buzz_level: buzz,
            body_wood_resonance: wood,
            master_gain: mgain,
        }
    }

    /// Dispatch interpolated parameters into `HurdyGurdyBus` and `ParamBus`.
    pub fn dispatch_to_bus(
        &self,
        beat: f64,
        bus: &HurdyGurdyBus,
        param_bus: Option<&ParamBus>,
        base_id: ParamId,
    ) {
        let snap = self.evaluate_at_beat(beat);

        bus.set_crank_speed(snap.crank_speed_rad_s);
        bus.set_wrist_acceleration(snap.wrist_acceleration_pulse);
        bus.set_wheel_pressure(snap.wheel_pressure);
        bus.set_chien_clearance(snap.chien_clearance_mm);
        bus.set_tangent_damping(snap.tangent_damping);
        bus.set_drone_melody_mix(snap.drone_melody_mix);
        bus.set_trompette_buzz_level(snap.trompette_buzz_level);
        bus.set_body_wood_resonance(snap.body_wood_resonance);
        bus.set_master_gain(snap.master_gain);

        if let Some(pb) = param_bus {
            pb.set(ParamId(base_id.0), snap.articulation as u32 as f32);
            pb.set(ParamId(base_id.0 + 1), snap.crank_speed_rad_s);
            pb.set(ParamId(base_id.0 + 2), snap.wrist_acceleration_pulse);
            pb.set(ParamId(base_id.0 + 3), snap.wheel_pressure);
            pb.set(ParamId(base_id.0 + 4), snap.chien_clearance_mm);
            pb.set(ParamId(base_id.0 + 5), snap.tangent_damping);
            pb.set(ParamId(base_id.0 + 6), snap.drone_melody_mix);
            pb.set(ParamId(base_id.0 + 7), snap.trompette_buzz_level);
            pb.set(ParamId(base_id.0 + 8), snap.body_wood_resonance);
            pb.set(ParamId(base_id.0 + 9), snap.master_gain);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hurdy_gurdy_gesture_patterns_and_catmull_rom_evaluation() {
        for pattern in [
            HurdyGurdyGesturePattern::BourbonnaisClassicCoupDePoignet { phrase_length_beats: 4.0, pulse_intensity: 1.5 },
            HurdyGurdyGesturePattern::MedievalMonasticDrone { phrase_length_beats: 4.0 },
            HurdyGurdyGesturePattern::BaroqueVirtuosoChanterelle { phrase_length_beats: 4.0 },
            HurdyGurdyGesturePattern::AuvergneHighSpeedBuzz { phrase_length_beats: 6.0, crank_speed_multiplier: 1.2 },
            HurdyGurdyGesturePattern::GothicPaganDrone { phrase_length_beats: 4.0 },
            HurdyGurdyGesturePattern::ElectroAcousticHybrid { phrase_length_beats: 4.0 },
        ] {
            let engine = HurdyGurdyGestureEngine::with_pattern(pattern);
            assert!(!engine.waypoints.is_empty());

            // Evaluate at multiple beat positions
            for beat in [0.0, 0.5, 1.2, 2.0, 3.5, 4.0, 5.8] {
                let snap = engine.evaluate_at_beat(beat);
                assert!(snap.crank_speed_rad_s > 0.0);
                assert!(snap.wheel_pressure > 0.0);
                assert!(snap.chien_clearance_mm > 0.0);
                assert!(snap.drone_melody_mix >= 0.0);
                assert!(snap.body_wood_resonance > 0.0);
                assert!(snap.master_gain > 0.0);
            }
        }
    }

    #[test]
    fn test_hurdy_gurdy_gesture_dispatch_to_bus() {
        let engine = HurdyGurdyGestureEngine::with_pattern(HurdyGurdyGesturePattern::BourbonnaisClassicCoupDePoignet {
            phrase_length_beats: 4.0,
            pulse_intensity: 2.0,
        });
        let bus = HurdyGurdyBus::new();
        let mut param_bus = ParamBus::new();
        summoner_core::hurdy_gurdy_bus::register_hurdy_gurdy_params(&mut param_bus, summoner_core::hurdy_gurdy_bus::HURDY_GURDY_BASE_PARAM_ID);

        engine.dispatch_to_bus(0.0, &bus, Some(&param_bus), summoner_core::hurdy_gurdy_bus::HURDY_GURDY_BASE_PARAM_ID);

        let snap = bus.snapshot();
        assert!((snap.crank_speed_rad_s - 2.0 * PI).abs() < 1e-3);
    }
}
