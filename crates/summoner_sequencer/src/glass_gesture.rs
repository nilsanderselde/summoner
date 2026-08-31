// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Dynamic Glass Armonica & Crystal Resonator Gesture Timeline & Articulation Engine (Milestone 31).
//!
//! Provides multi-dimensional continuous physical modeling trajectory generation for
//! Spindle Rotation Speed $\omega \in [0..6\pi]\text{ rad/s}$, Wet Finger Normal Contact Force $F_N \in [0.05..2.50]\text{ N}$,
//! Hydro-Acoustic Water Fill Level $h \in [0.0..1.0]$, Striker Mallet Velocity, Moisture Lubrication,
//! Mahogany Chassis Resonance, and Thin-Shell Q-Factor Scaling with Catmull-Rom spline smoothing,
//! TOML serialization, and sample-accurate lock-free parameter dispatch into `GlassArmonicaBus` and `ParamBus`.
//!
//! Enforces zero heap allocations in real-time execution loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use summoner_core::glass_bus::{
    GlassArticulation, GlassArmonicaBus, GlassArmonicaBusSnapshot,
};
use summoner_core::param_bus::{ParamBus, ParamId};

/// Interpolation curve between discrete Glass Armonica keyframe waypoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum GlassInterpolationCurve {
    #[default]
    CubicCatmullRom,
    CubicBezier,
    Linear,
    Hold,
    Exponential,
}

/// A single keyframed multi-dimensional Glass Armonica articulation waypoint.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GlassWaypoint {
    /// Timeline beat position in quarter note beats.
    pub beat: f64,
    /// Spindle rotation speed $[0.0 ..= 6.0\pi]\text{ rad/s}$.
    pub spindle_speed_rad_s: f32,
    /// Wet finger normal contact force in Newtons $[0.05 ..= 2.50]\text{ N}$.
    pub normal_force_n: f32,
    /// Hydro-acoustic water fill level $[0.0 ..= 1.0]$.
    pub water_fill_level: f32,
    /// Soft mallet / striker impact velocity $[0.0 ..= 1.5]$.
    pub strike_velocity: f32,
    /// Moisture lubrication coefficient $[0.1 ..= 1.0]$.
    pub moisture_lubrication: f32,
    /// Mahogany chassis acoustic resonance $[0.0 ..= 1.0]$.
    pub chassis_resonance: f32,
    /// Thin-shell Q-factor scaling $[0.2 ..= 2.5]$.
    pub q_scale: f32,
    /// Master output gain $[0.0 ..= 2.0]$.
    pub master_gain: f32,
    /// Articulation technique.
    pub articulation: GlassArticulation,
    /// Interpolation curve leading to this waypoint.
    pub curve: GlassInterpolationCurve,
}

impl Default for GlassWaypoint {
    fn default() -> Self {
        let (speed, force, water, strike, moist, chassis, qscale, mgain) =
            GlassArticulation::FranklinAuthenticContinuous.nominal_parameters();

        Self {
            beat: 0.0,
            spindle_speed_rad_s: speed,
            normal_force_n: force,
            water_fill_level: water,
            strike_velocity: strike,
            moisture_lubrication: moist,
            chassis_resonance: chassis,
            q_scale: qscale,
            master_gain: mgain,
            articulation: GlassArticulation::FranklinAuthenticContinuous,
            curve: GlassInterpolationCurve::CubicCatmullRom,
        }
    }
}

/// Pre-configured algorithmic Glass Armonica gesture trajectory patterns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GlassGesturePattern {
    /// 1761 Franklin Authentic continuous gentle spindle rubbing.
    FranklinAuthenticContinuous {
        phrase_length_beats: f64,
    },
    /// Mesmer Magnetic Healing Chalice with slow rotation and heavy water modulation.
    MesmerMagneticHealing {
        phrase_length_beats: f64,
        water_swell_depth: f32,
    },
    /// Mozart Virtuoso Adagio with dynamic finger pressure accents.
    MozartVirtuosoAdagio {
        phrase_length_beats: f64,
    },
    /// Ethereal Borosilicate Swell with expansive high-frequency sustain.
    EtherealBorosilicateSwell {
        phrase_length_beats: f64,
    },
    /// Water-Tuned Crystal Glissando with dynamic fluid mass-loading pitch shifts.
    WaterTunedCrystalGlissando {
        phrase_length_beats: f64,
        max_water_level: f32,
    },
    /// Quartz Crystal Singing Bowl Meditation with gentle mallet taps and long resonance.
    CrystalSingingBowlMeditation {
        phrase_length_beats: f64,
        mallet_interval_beats: f64,
    },
}

/// Dynamic Glass Armonica articulation timeline and parameter trajectory engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlassGestureEngine {
    pub waypoints: Vec<GlassWaypoint>,
    pub loop_length_beats: f64,
    pub is_looping: bool,
}

impl Default for GlassGestureEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl GlassGestureEngine {
    pub fn new() -> Self {
        Self {
            waypoints: Vec::new(),
            loop_length_beats: 4.0,
            is_looping: true,
        }
    }

    pub fn with_pattern(pattern: GlassGesturePattern) -> Self {
        let mut engine = Self::new();
        engine.load_pattern(pattern);
        engine
    }

    pub fn add_waypoint(&mut self, waypoint: GlassWaypoint) {
        self.waypoints.push(waypoint);
        self.sort_waypoints();
    }

    pub fn sort_waypoints(&mut self) {
        self.waypoints.sort_by(|a, b| a.beat.partial_cmp(&b.beat).unwrap_or(std::cmp::Ordering::Equal));
    }

    pub fn clear(&mut self) {
        self.waypoints.clear();
    }

    pub fn load_pattern(&mut self, pattern: GlassGesturePattern) {
        self.clear();

        match &pattern {
            GlassGesturePattern::FranklinAuthenticContinuous { phrase_length_beats } => {
                self.loop_length_beats = *phrase_length_beats;

                self.waypoints.push(GlassWaypoint {
                    beat: 0.0,
                    spindle_speed_rad_s: 2.5 * PI,
                    normal_force_n: 0.45,
                    water_fill_level: 0.05,
                    strike_velocity: 0.0,
                    moisture_lubrication: 0.85,
                    chassis_resonance: 0.75,
                    q_scale: 1.00,
                    master_gain: 0.90,
                    articulation: GlassArticulation::FranklinAuthenticContinuous,
                    curve: GlassInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(GlassWaypoint {
                    beat: phrase_length_beats * 0.50,
                    spindle_speed_rad_s: 2.8 * PI,
                    normal_force_n: 0.55,
                    water_fill_level: 0.05,
                    strike_velocity: 0.0,
                    moisture_lubrication: 0.88,
                    chassis_resonance: 0.78,
                    q_scale: 1.05,
                    master_gain: 0.92,
                    articulation: GlassArticulation::FranklinAuthenticContinuous,
                    curve: GlassInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(GlassWaypoint {
                    beat: *phrase_length_beats,
                    spindle_speed_rad_s: 2.5 * PI,
                    normal_force_n: 0.45,
                    water_fill_level: 0.05,
                    strike_velocity: 0.0,
                    moisture_lubrication: 0.85,
                    chassis_resonance: 0.75,
                    q_scale: 1.00,
                    master_gain: 0.90,
                    articulation: GlassArticulation::FranklinAuthenticContinuous,
                    curve: GlassInterpolationCurve::CubicCatmullRom,
                });
            }

            GlassGesturePattern::MesmerMagneticHealing { phrase_length_beats, water_swell_depth } => {
                self.loop_length_beats = *phrase_length_beats;
                let depth = *water_swell_depth;

                self.waypoints.push(GlassWaypoint {
                    beat: 0.0,
                    spindle_speed_rad_s: 1.5 * PI,
                    normal_force_n: 0.60,
                    water_fill_level: 0.20,
                    strike_velocity: 0.0,
                    moisture_lubrication: 0.90,
                    chassis_resonance: 0.85,
                    q_scale: 1.25,
                    master_gain: 0.95,
                    articulation: GlassArticulation::MesmerMagneticHealing,
                    curve: GlassInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(GlassWaypoint {
                    beat: phrase_length_beats * 0.50,
                    spindle_speed_rad_s: 1.8 * PI,
                    normal_force_n: 0.75,
                    water_fill_level: (0.20 + depth).clamp(0.0, 1.0),
                    strike_velocity: 0.0,
                    moisture_lubrication: 0.95,
                    chassis_resonance: 0.90,
                    q_scale: 1.35,
                    master_gain: 0.96,
                    articulation: GlassArticulation::MesmerMagneticHealing,
                    curve: GlassInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(GlassWaypoint {
                    beat: *phrase_length_beats,
                    spindle_speed_rad_s: 1.5 * PI,
                    normal_force_n: 0.60,
                    water_fill_level: 0.20,
                    strike_velocity: 0.0,
                    moisture_lubrication: 0.90,
                    chassis_resonance: 0.85,
                    q_scale: 1.25,
                    master_gain: 0.95,
                    articulation: GlassArticulation::MesmerMagneticHealing,
                    curve: GlassInterpolationCurve::CubicCatmullRom,
                });
            }

            GlassGesturePattern::MozartVirtuosoAdagio { phrase_length_beats } => {
                self.loop_length_beats = *phrase_length_beats;

                self.waypoints.push(GlassWaypoint {
                    beat: 0.0,
                    spindle_speed_rad_s: 3.0 * PI,
                    normal_force_n: 0.50,
                    water_fill_level: 0.02,
                    strike_velocity: 0.3,
                    moisture_lubrication: 0.85,
                    chassis_resonance: 0.70,
                    q_scale: 0.95,
                    master_gain: 0.92,
                    articulation: GlassArticulation::MozartVirtuosoAdagio,
                    curve: GlassInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(GlassWaypoint {
                    beat: phrase_length_beats * 0.33,
                    spindle_speed_rad_s: 3.4 * PI,
                    normal_force_n: 0.70,
                    water_fill_level: 0.02,
                    strike_velocity: 0.6,
                    moisture_lubrication: 0.88,
                    chassis_resonance: 0.75,
                    q_scale: 1.00,
                    master_gain: 0.94,
                    articulation: GlassArticulation::MozartVirtuosoAdagio,
                    curve: GlassInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(GlassWaypoint {
                    beat: phrase_length_beats * 0.66,
                    spindle_speed_rad_s: 2.8 * PI,
                    normal_force_n: 0.35,
                    water_fill_level: 0.02,
                    strike_velocity: 0.1,
                    moisture_lubrication: 0.82,
                    chassis_resonance: 0.68,
                    q_scale: 0.92,
                    master_gain: 0.90,
                    articulation: GlassArticulation::MozartVirtuosoAdagio,
                    curve: GlassInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(GlassWaypoint {
                    beat: *phrase_length_beats,
                    spindle_speed_rad_s: 3.0 * PI,
                    normal_force_n: 0.50,
                    water_fill_level: 0.02,
                    strike_velocity: 0.3,
                    moisture_lubrication: 0.85,
                    chassis_resonance: 0.70,
                    q_scale: 0.95,
                    master_gain: 0.92,
                    articulation: GlassArticulation::MozartVirtuosoAdagio,
                    curve: GlassInterpolationCurve::CubicCatmullRom,
                });
            }

            GlassGesturePattern::EtherealBorosilicateSwell { phrase_length_beats } => {
                self.loop_length_beats = *phrase_length_beats;

                self.waypoints.push(GlassWaypoint {
                    beat: 0.0,
                    spindle_speed_rad_s: 2.2 * PI,
                    normal_force_n: 0.40,
                    water_fill_level: 0.10,
                    strike_velocity: 0.0,
                    moisture_lubrication: 0.80,
                    chassis_resonance: 0.80,
                    q_scale: 1.15,
                    master_gain: 0.88,
                    articulation: GlassArticulation::EtherealBorosilicateSwell,
                    curve: GlassInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(GlassWaypoint {
                    beat: phrase_length_beats * 0.50,
                    spindle_speed_rad_s: 2.6 * PI,
                    normal_force_n: 0.60,
                    water_fill_level: 0.12,
                    strike_velocity: 0.0,
                    moisture_lubrication: 0.85,
                    chassis_resonance: 0.85,
                    q_scale: 1.25,
                    master_gain: 0.92,
                    articulation: GlassArticulation::EtherealBorosilicateSwell,
                    curve: GlassInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(GlassWaypoint {
                    beat: *phrase_length_beats,
                    spindle_speed_rad_s: 2.2 * PI,
                    normal_force_n: 0.40,
                    water_fill_level: 0.10,
                    strike_velocity: 0.0,
                    moisture_lubrication: 0.80,
                    chassis_resonance: 0.80,
                    q_scale: 1.15,
                    master_gain: 0.88,
                    articulation: GlassArticulation::EtherealBorosilicateSwell,
                    curve: GlassInterpolationCurve::CubicCatmullRom,
                });
            }

            GlassGesturePattern::WaterTunedCrystalGlissando { phrase_length_beats, max_water_level } => {
                self.loop_length_beats = *phrase_length_beats;
                let max_w = *max_water_level;

                self.waypoints.push(GlassWaypoint {
                    beat: 0.0,
                    spindle_speed_rad_s: 2.0 * PI,
                    normal_force_n: 0.55,
                    water_fill_level: 0.0,
                    strike_velocity: 0.0,
                    moisture_lubrication: 0.85,
                    chassis_resonance: 0.65,
                    q_scale: 1.05,
                    master_gain: 0.90,
                    articulation: GlassArticulation::WaterTunedCrystalGlissando,
                    curve: GlassInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(GlassWaypoint {
                    beat: phrase_length_beats * 0.50,
                    spindle_speed_rad_s: 2.2 * PI,
                    normal_force_n: 0.60,
                    water_fill_level: max_w.clamp(0.0, 1.0),
                    strike_velocity: 0.1,
                    moisture_lubrication: 0.90,
                    chassis_resonance: 0.70,
                    q_scale: 1.10,
                    master_gain: 0.92,
                    articulation: GlassArticulation::WaterTunedCrystalGlissando,
                    curve: GlassInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(GlassWaypoint {
                    beat: *phrase_length_beats,
                    spindle_speed_rad_s: 2.0 * PI,
                    normal_force_n: 0.55,
                    water_fill_level: 0.0,
                    strike_velocity: 0.0,
                    moisture_lubrication: 0.85,
                    chassis_resonance: 0.65,
                    q_scale: 1.05,
                    master_gain: 0.90,
                    articulation: GlassArticulation::WaterTunedCrystalGlissando,
                    curve: GlassInterpolationCurve::CubicCatmullRom,
                });
            }

            GlassGesturePattern::CrystalSingingBowlMeditation { phrase_length_beats, mallet_interval_beats } => {
                self.loop_length_beats = *phrase_length_beats;
                let interval = mallet_interval_beats.max(0.5);

                let mut current_beat = 0.0;
                while current_beat <= *phrase_length_beats {
                    self.waypoints.push(GlassWaypoint {
                        beat: current_beat,
                        spindle_speed_rad_s: 1.2 * PI,
                        normal_force_n: 0.70,
                        water_fill_level: 0.0,
                        strike_velocity: 0.75, // Mallet strike
                        moisture_lubrication: 0.95,
                        chassis_resonance: 0.90,
                        q_scale: 1.50,
                        master_gain: 0.95,
                        articulation: GlassArticulation::CrystalSingingBowlMeditation,
                        curve: GlassInterpolationCurve::CubicCatmullRom,
                    });

                    if current_beat + interval * 0.5 < *phrase_length_beats {
                        self.waypoints.push(GlassWaypoint {
                            beat: current_beat + interval * 0.5,
                            spindle_speed_rad_s: 1.2 * PI,
                            normal_force_n: 0.65,
                            water_fill_level: 0.0,
                            strike_velocity: 0.0, // Sustained crystalline ringing
                            moisture_lubrication: 0.95,
                            chassis_resonance: 0.90,
                            q_scale: 1.50,
                            master_gain: 0.95,
                            articulation: GlassArticulation::CrystalSingingBowlMeditation,
                            curve: GlassInterpolationCurve::CubicCatmullRom,
                        });
                    }
                    current_beat += interval;
                }
            }
        }
    }

    /// Evaluate timeline trajectory at arbitrary beat and return interpolated snapshot.
    pub fn evaluate_at_beat(&self, beat: f64) -> GlassArmonicaBusSnapshot {
        if self.waypoints.is_empty() {
            let (speed, force, water, strike, moist, chassis, qscale, mgain) =
                GlassArticulation::FranklinAuthenticContinuous.nominal_parameters();
            return GlassArmonicaBusSnapshot {
                articulation: GlassArticulation::FranklinAuthenticContinuous,
                spindle_speed_rad_s: speed,
                normal_force_n: force,
                water_fill_level: water,
                strike_velocity: strike,
                moisture_lubrication: moist,
                chassis_resonance: chassis,
                q_scale: qscale,
                master_gain: mgain,
            };
        }

        if self.waypoints.len() == 1 {
            let wp = &self.waypoints[0];
            return GlassArmonicaBusSnapshot {
                articulation: wp.articulation,
                spindle_speed_rad_s: wp.spindle_speed_rad_s,
                normal_force_n: wp.normal_force_n,
                water_fill_level: wp.water_fill_level,
                strike_velocity: wp.strike_velocity,
                moisture_lubrication: wp.moisture_lubrication,
                chassis_resonance: wp.chassis_resonance,
                q_scale: wp.q_scale,
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
            return GlassArmonicaBusSnapshot {
                articulation: wp.articulation,
                spindle_speed_rad_s: wp.spindle_speed_rad_s,
                normal_force_n: wp.normal_force_n,
                water_fill_level: wp.water_fill_level,
                strike_velocity: wp.strike_velocity,
                moisture_lubrication: wp.moisture_lubrication,
                chassis_resonance: wp.chassis_resonance,
                q_scale: wp.q_scale,
                master_gain: wp.master_gain,
            };
        }

        if idx1 >= n {
            let wp = &self.waypoints[n - 1];
            return GlassArmonicaBusSnapshot {
                articulation: wp.articulation,
                spindle_speed_rad_s: wp.spindle_speed_rad_s,
                normal_force_n: wp.normal_force_n,
                water_fill_level: wp.water_fill_level,
                strike_velocity: wp.strike_velocity,
                moisture_lubrication: wp.moisture_lubrication,
                chassis_resonance: wp.chassis_resonance,
                q_scale: wp.q_scale,
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

        let interp = |v0: f32, v1: f32, v2: f32, v3: f32, curve: GlassInterpolationCurve| -> f32 {
            match curve {
                GlassInterpolationCurve::Hold => v1,
                GlassInterpolationCurve::Linear => v1 + t * (v2 - v1),
                GlassInterpolationCurve::Exponential => {
                    let min_v = v1.min(v2).max(1e-5);
                    let ratio = (v2.max(1e-5) / min_v).max(1e-5);
                    v1 * ratio.powf(t)
                }
                GlassInterpolationCurve::CubicBezier => {
                    let t2 = t * t;
                    let t3 = t2 * t;
                    let b = 3.0 * t2 - 2.0 * t3;
                    v1 + b * (v2 - v1)
                }
                GlassInterpolationCurve::CubicCatmullRom => {
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
        let speed = interp(wp0.spindle_speed_rad_s, wp1.spindle_speed_rad_s, wp2.spindle_speed_rad_s, wp3.spindle_speed_rad_s, curve).clamp(0.0, 6.0 * PI);
        let force = interp(wp0.normal_force_n, wp1.normal_force_n, wp2.normal_force_n, wp3.normal_force_n, curve).clamp(0.05, 2.50);
        let water = interp(wp0.water_fill_level, wp1.water_fill_level, wp2.water_fill_level, wp3.water_fill_level, curve).clamp(0.0, 1.0);
        let strike = interp(wp0.strike_velocity, wp1.strike_velocity, wp2.strike_velocity, wp3.strike_velocity, curve).clamp(0.0, 1.5);
        let moist = interp(wp0.moisture_lubrication, wp1.moisture_lubrication, wp2.moisture_lubrication, wp3.moisture_lubrication, curve).clamp(0.1, 1.0);
        let chassis = interp(wp0.chassis_resonance, wp1.chassis_resonance, wp2.chassis_resonance, wp3.chassis_resonance, curve).clamp(0.0, 1.0);
        let qscale = interp(wp0.q_scale, wp1.q_scale, wp2.q_scale, wp3.q_scale, curve).clamp(0.2, 2.5);
        let mgain = interp(wp0.master_gain, wp1.master_gain, wp2.master_gain, wp3.master_gain, curve).clamp(0.0, 2.0);

        GlassArmonicaBusSnapshot {
            articulation: if t > 0.5 { wp2.articulation } else { wp1.articulation },
            spindle_speed_rad_s: speed,
            normal_force_n: force,
            water_fill_level: water,
            strike_velocity: strike,
            moisture_lubrication: moist,
            chassis_resonance: chassis,
            q_scale: qscale,
            master_gain: mgain,
        }
    }

    /// Dispatch interpolated parameters into `GlassArmonicaBus` and `ParamBus`.
    pub fn dispatch_to_bus(
        &self,
        beat: f64,
        bus: &GlassArmonicaBus,
        param_bus: Option<&ParamBus>,
        base_id: ParamId,
    ) {
        let snap = self.evaluate_at_beat(beat);

        bus.set_spindle_speed(snap.spindle_speed_rad_s);
        bus.set_normal_force(snap.normal_force_n);
        bus.set_water_fill_level(snap.water_fill_level);
        bus.set_strike_velocity(snap.strike_velocity);
        bus.set_moisture_lubrication(snap.moisture_lubrication);
        bus.set_chassis_resonance(snap.chassis_resonance);
        bus.set_q_scale(snap.q_scale);
        bus.set_master_gain(snap.master_gain);

        if let Some(pb) = param_bus {
            pb.set(ParamId(base_id.0), snap.articulation as u32 as f32);
            pb.set(ParamId(base_id.0 + 1), snap.spindle_speed_rad_s);
            pb.set(ParamId(base_id.0 + 2), snap.normal_force_n);
            pb.set(ParamId(base_id.0 + 3), snap.water_fill_level);
            pb.set(ParamId(base_id.0 + 4), snap.strike_velocity);
            pb.set(ParamId(base_id.0 + 5), snap.moisture_lubrication);
            pb.set(ParamId(base_id.0 + 6), snap.chassis_resonance);
            pb.set(ParamId(base_id.0 + 7), snap.q_scale);
            pb.set(ParamId(base_id.0 + 8), snap.master_gain);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glass_gesture_patterns_and_catmull_rom_evaluation() {
        for pattern in [
            GlassGesturePattern::FranklinAuthenticContinuous { phrase_length_beats: 4.0 },
            GlassGesturePattern::MesmerMagneticHealing { phrase_length_beats: 4.0, water_swell_depth: 0.3 },
            GlassGesturePattern::MozartVirtuosoAdagio { phrase_length_beats: 6.0 },
            GlassGesturePattern::EtherealBorosilicateSwell { phrase_length_beats: 4.0 },
            GlassGesturePattern::WaterTunedCrystalGlissando { phrase_length_beats: 4.0, max_water_level: 0.6 },
            GlassGesturePattern::CrystalSingingBowlMeditation { phrase_length_beats: 8.0, mallet_interval_beats: 2.0 },
        ] {
            let engine = GlassGestureEngine::with_pattern(pattern);
            assert!(!engine.waypoints.is_empty());

            for beat in [0.0, 0.5, 1.2, 2.0, 3.5, 4.0, 5.8] {
                let snap = engine.evaluate_at_beat(beat);
                assert!(snap.spindle_speed_rad_s > 0.0);
                assert!(snap.normal_force_n > 0.0);
                assert!(snap.water_fill_level >= 0.0);
                assert!(snap.moisture_lubrication > 0.0);
                assert!(snap.chassis_resonance > 0.0);
                assert!(snap.q_scale > 0.0);
                assert!(snap.master_gain > 0.0);
            }
        }
    }

    #[test]
    fn test_glass_gesture_dispatch_to_bus() {
        let engine = GlassGestureEngine::with_pattern(GlassGesturePattern::FranklinAuthenticContinuous {
            phrase_length_beats: 4.0,
        });
        let bus = GlassArmonicaBus::new();
        let mut param_bus = ParamBus::new();
        summoner_core::glass_bus::register_glass_armonica_params(
            &mut param_bus,
            summoner_core::glass_bus::GLASS_ARMONICA_BASE_PARAM_ID,
        );

        engine.dispatch_to_bus(
            0.0,
            &bus,
            Some(&param_bus),
            summoner_core::glass_bus::GLASS_ARMONICA_BASE_PARAM_ID,
        );

        let snap = bus.snapshot();
        assert!((snap.spindle_speed_rad_s - 2.5 * PI).abs() < 1e-3);
    }
}
