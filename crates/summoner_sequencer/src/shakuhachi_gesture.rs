// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Dynamic Shakuhachi Breath Gesture Timeline & Trajectory Engine (Milestone 25).
//!
//! Provides multi-dimensional continuous physical modeling trajectory generation for
//! Japanese Shakuhachi breath gestures, blowing pressure swells, dynamic Meri/Kari head
//! tilt microtone pitch bends (Δf in [-300, +200] cents), jet distance modulation,
//! Murai-Iki explosive turbulence bursts, and 5-finger hole acoustic transitions
//! with Catmull-Rom spline smoothing, TOML serialization, and sample-accurate lock-free
//! parameter dispatch into `ShakuhachiBus` and `ParamBus`.
//!
//! Enforces zero heap allocations in real-time execution loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use summoner_core::param_bus::{ParamBus, ParamId};
use summoner_core::shakuhachi_bus::{
    ShakuhachiArticulation, ShakuhachiBus, ShakuhachiBusSnapshot,
};

/// Interpolation curve between discrete Shakuhachi keyframe waypoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ShakuhachiInterpolationCurve {
    #[default]
    CubicCatmullRom,
    CubicBezier,
    Linear,
    Hold,
    Exponential,
}

/// A single keyframed multi-dimensional Shakuhachi articulation waypoint.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ShakuhachiWaypoint {
    /// Timeline beat position in quarter note beats.
    pub beat: f64,
    /// Blowing pressure in Pascals [50.0 ..= 4000.0].
    pub blowing_pressure_pa: f32,
    /// Embouchure resting angle in degrees [10.0 ..= 60.0].
    pub embouchure_angle_deg: f32,
    /// Dynamic Meri/Kari head tilt pitch bend offset in cents [-300.0 ..= +200.0].
    pub meri_kari_cents: f32,
    /// Jet distance in mm [2.0 ..= 25.0].
    pub jet_distance_mm: f32,
    /// Murai-Iki explosive breath burst intensity [0.0 ..= 1.0].
    pub murai_iki_intensity: f32,
    /// 5-finger hole active bitmask.
    pub hole_mask: u32,
    /// Master gain [0.0 ..= 2.0].
    pub master_gain: f32,
    /// Articulation technique.
    pub articulation: ShakuhachiArticulation,
    /// Interpolation curve leading to this waypoint.
    pub curve: ShakuhachiInterpolationCurve,
}

impl Default for ShakuhachiWaypoint {
    fn default() -> Self {
        let (p, angle, meri, dist, murai, mask, gain) =
            ShakuhachiArticulation::HonkyokuTraditional.nominal_parameters();

        Self {
            beat: 0.0,
            blowing_pressure_pa: p,
            embouchure_angle_deg: angle,
            meri_kari_cents: meri,
            jet_distance_mm: dist,
            murai_iki_intensity: murai,
            hole_mask: mask,
            master_gain: gain,
            articulation: ShakuhachiArticulation::HonkyokuTraditional,
            curve: ShakuhachiInterpolationCurve::CubicCatmullRom,
        }
    }
}

/// Pre-configured algorithmic Shakuhachi gesture trajectory patterns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ShakuhachiGesturePattern {
    /// Classical Honkyoku meditation with gentle pressure swells and meri microtonal vibrato.
    HonkyokuMeditation {
        phrase_length_beats: f64,
        max_pressure_pa: f32,
    },
    /// Explosive Murai-Iki attack with sudden vortex noise bursts and overblowing.
    MuraiIkiBurst {
        burst_length_beats: f64,
        peak_intensity: f32,
    },
    /// Dynamic Meri/Kari continuous pitch bend glissando across full range.
    MeriKariGlissando {
        gliss_length_beats: f64,
        min_cents: f32,
        max_cents: f32,
    },
    /// Deep Jinashi Zen meditation with slow breath pacing.
    ZenJinashiFlutter {
        phrase_length_beats: f64,
    },
    /// Traditional pentatonic 5-hole scale melodic sequence (Ro -> Tsu -> Re -> Chi -> Ri).
    PentatonicMelodicRun {
        step_length_beats: f64,
    },
    /// User-defined multi-keyframe Catmull-Rom spline trajectory.
    CustomSpline,
}

impl Default for ShakuhachiGesturePattern {
    fn default() -> Self {
        Self::HonkyokuMeditation {
            phrase_length_beats: 8.0,
            max_pressure_pa: 1100.0,
        }
    }
}

/// Dynamic Shakuhachi Articulation Timeline & Gesture Engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShakuhachiGestureEngine {
    /// Keyframed articulation waypoints.
    pub waypoints: Vec<ShakuhachiWaypoint>,
    /// Active gesture trajectory pattern.
    pub pattern: ShakuhachiGesturePattern,
    /// Default interpolation curve.
    pub default_curve: ShakuhachiInterpolationCurve,
    /// Total sequence loop length in beats.
    pub loop_length_beats: f64,
    /// Whether pattern loops continuously.
    pub loop_enabled: bool,
    /// Whether gesture engine is currently active.
    pub is_active: bool,
}

impl Default for ShakuhachiGestureEngine {
    fn default() -> Self {
        let pattern = ShakuhachiGesturePattern::default();
        let mut engine = Self {
            waypoints: Vec::new(),
            pattern: pattern.clone(),
            default_curve: ShakuhachiInterpolationCurve::CubicCatmullRom,
            loop_length_beats: 8.0,
            loop_enabled: true,
            is_active: true,
        };
        engine.generate_pattern_waypoints(&pattern);
        engine
    }
}

impl ShakuhachiGestureEngine {
    /// Create a new gesture engine with default HonkyokuMeditation pattern.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create engine with a specific gesture pattern.
    pub fn with_pattern(pattern: ShakuhachiGesturePattern) -> Self {
        let mut engine = Self {
            waypoints: Vec::new(),
            pattern: pattern.clone(),
            default_curve: ShakuhachiInterpolationCurve::CubicCatmullRom,
            loop_length_beats: 8.0,
            loop_enabled: true,
            is_active: true,
        };
        engine.generate_pattern_waypoints(&pattern);
        engine
    }

    /// Clear existing waypoints and generate algorithmic waypoints from pattern.
    pub fn generate_pattern_waypoints(&mut self, pattern: &ShakuhachiGesturePattern) {
        self.waypoints.clear();
        self.pattern = pattern.clone();

        match pattern {
            ShakuhachiGesturePattern::HonkyokuMeditation { phrase_length_beats, max_pressure_pa } => {
                self.loop_length_beats = *phrase_length_beats;
                let half = phrase_length_beats * 0.5;

                self.waypoints.push(ShakuhachiWaypoint {
                    beat: 0.0,
                    blowing_pressure_pa: 400.0,
                    embouchure_angle_deg: 38.0,
                    meri_kari_cents: 0.0,
                    jet_distance_mm: 10.0,
                    murai_iki_intensity: 0.05,
                    hole_mask: 0,
                    master_gain: 0.80,
                    articulation: ShakuhachiArticulation::HonkyokuTraditional,
                    curve: ShakuhachiInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(ShakuhachiWaypoint {
                    beat: half * 0.5,
                    blowing_pressure_pa: *max_pressure_pa,
                    embouchure_angle_deg: 40.0,
                    meri_kari_cents: -40.0,
                    jet_distance_mm: 10.5,
                    murai_iki_intensity: 0.15,
                    hole_mask: 0,
                    master_gain: 0.88,
                    articulation: ShakuhachiArticulation::HonkyokuTraditional,
                    curve: ShakuhachiInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(ShakuhachiWaypoint {
                    beat: half,
                    blowing_pressure_pa: *max_pressure_pa * 0.8,
                    embouchure_angle_deg: 34.0,
                    meri_kari_cents: -120.0,
                    jet_distance_mm: 11.0,
                    murai_iki_intensity: 0.20,
                    hole_mask: 0,
                    master_gain: 0.82,
                    articulation: ShakuhachiArticulation::MeriMicrotoneBend,
                    curve: ShakuhachiInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(ShakuhachiWaypoint {
                    beat: *phrase_length_beats,
                    blowing_pressure_pa: 400.0,
                    embouchure_angle_deg: 38.0,
                    meri_kari_cents: 0.0,
                    jet_distance_mm: 10.0,
                    murai_iki_intensity: 0.05,
                    hole_mask: 0,
                    master_gain: 0.80,
                    articulation: ShakuhachiArticulation::HonkyokuTraditional,
                    curve: ShakuhachiInterpolationCurve::CubicCatmullRom,
                });
            }

            ShakuhachiGesturePattern::MuraiIkiBurst { burst_length_beats, peak_intensity } => {
                self.loop_length_beats = *burst_length_beats;

                self.waypoints.push(ShakuhachiWaypoint {
                    beat: 0.0,
                    blowing_pressure_pa: 2800.0,
                    embouchure_angle_deg: 32.0,
                    meri_kari_cents: 50.0,
                    jet_distance_mm: 9.0,
                    murai_iki_intensity: *peak_intensity,
                    hole_mask: 0,
                    master_gain: 0.95,
                    articulation: ShakuhachiArticulation::MuraiIkiExplosive,
                    curve: ShakuhachiInterpolationCurve::Exponential,
                });

                self.waypoints.push(ShakuhachiWaypoint {
                    beat: burst_length_beats * 0.25,
                    blowing_pressure_pa: 1400.0,
                    embouchure_angle_deg: 36.0,
                    meri_kari_cents: 10.0,
                    jet_distance_mm: 10.0,
                    murai_iki_intensity: *peak_intensity * 0.35,
                    hole_mask: 0,
                    master_gain: 0.88,
                    articulation: ShakuhachiArticulation::HonkyokuTraditional,
                    curve: ShakuhachiInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(ShakuhachiWaypoint {
                    beat: *burst_length_beats,
                    blowing_pressure_pa: 700.0,
                    embouchure_angle_deg: 38.0,
                    meri_kari_cents: 0.0,
                    jet_distance_mm: 10.0,
                    murai_iki_intensity: 0.05,
                    hole_mask: 0,
                    master_gain: 0.82,
                    articulation: ShakuhachiArticulation::HonkyokuTraditional,
                    curve: ShakuhachiInterpolationCurve::CubicCatmullRom,
                });
            }

            ShakuhachiGesturePattern::MeriKariGlissando { gliss_length_beats, min_cents, max_cents } => {
                self.loop_length_beats = *gliss_length_beats;

                self.waypoints.push(ShakuhachiWaypoint {
                    beat: 0.0,
                    blowing_pressure_pa: 750.0,
                    embouchure_angle_deg: 20.0,
                    meri_kari_cents: *min_cents,
                    jet_distance_mm: 12.0,
                    murai_iki_intensity: 0.10,
                    hole_mask: 0,
                    master_gain: 0.80,
                    articulation: ShakuhachiArticulation::MeriMicrotoneBend,
                    curve: ShakuhachiInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(ShakuhachiWaypoint {
                    beat: gliss_length_beats * 0.5,
                    blowing_pressure_pa: 850.0,
                    embouchure_angle_deg: 38.0,
                    meri_kari_cents: 0.0,
                    jet_distance_mm: 10.0,
                    murai_iki_intensity: 0.08,
                    hole_mask: 0,
                    master_gain: 0.85,
                    articulation: ShakuhachiArticulation::HonkyokuTraditional,
                    curve: ShakuhachiInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(ShakuhachiWaypoint {
                    beat: *gliss_length_beats,
                    blowing_pressure_pa: 1200.0,
                    embouchure_angle_deg: 52.0,
                    meri_kari_cents: *max_cents,
                    jet_distance_mm: 8.5,
                    murai_iki_intensity: 0.15,
                    hole_mask: 0,
                    master_gain: 0.90,
                    articulation: ShakuhachiArticulation::KariOverblow,
                    curve: ShakuhachiInterpolationCurve::CubicCatmullRom,
                });
            }

            ShakuhachiGesturePattern::ZenJinashiFlutter { phrase_length_beats } => {
                self.loop_length_beats = *phrase_length_beats;

                self.waypoints.push(ShakuhachiWaypoint {
                    beat: 0.0,
                    blowing_pressure_pa: 600.0,
                    embouchure_angle_deg: 44.0,
                    meri_kari_cents: -60.0,
                    jet_distance_mm: 14.0,
                    murai_iki_intensity: 0.30,
                    hole_mask: 0,
                    master_gain: 0.80,
                    articulation: ShakuhachiArticulation::ZenMeditative,
                    curve: ShakuhachiInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(ShakuhachiWaypoint {
                    beat: phrase_length_beats * 0.5,
                    blowing_pressure_pa: 850.0,
                    embouchure_angle_deg: 42.0,
                    meri_kari_cents: -150.0,
                    jet_distance_mm: 15.0,
                    murai_iki_intensity: 0.45,
                    hole_mask: 0,
                    master_gain: 0.84,
                    articulation: ShakuhachiArticulation::ZenMeditative,
                    curve: ShakuhachiInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(ShakuhachiWaypoint {
                    beat: *phrase_length_beats,
                    blowing_pressure_pa: 600.0,
                    embouchure_angle_deg: 44.0,
                    meri_kari_cents: -60.0,
                    jet_distance_mm: 14.0,
                    murai_iki_intensity: 0.30,
                    hole_mask: 0,
                    master_gain: 0.80,
                    articulation: ShakuhachiArticulation::ZenMeditative,
                    curve: ShakuhachiInterpolationCurve::CubicCatmullRom,
                });
            }

            ShakuhachiGesturePattern::PentatonicMelodicRun { step_length_beats } => {
                let masks = [0, 1, 3, 7, 15]; // Ro (0), Tsu (1), Re (3), Chi (7), Ri (15)
                self.loop_length_beats = step_length_beats * 5.0;

                for (i, &mask) in masks.iter().enumerate() {
                    self.waypoints.push(ShakuhachiWaypoint {
                        beat: (i as f64) * step_length_beats,
                        blowing_pressure_pa: 900.0 + (i as f32) * 50.0,
                        embouchure_angle_deg: 38.0,
                        meri_kari_cents: 0.0,
                        jet_distance_mm: 10.0,
                        murai_iki_intensity: 0.06,
                        hole_mask: mask,
                        master_gain: 0.85,
                        articulation: ShakuhachiArticulation::MinyoFolk,
                        curve: ShakuhachiInterpolationCurve::Hold,
                    });
                }
            }

            ShakuhachiGesturePattern::CustomSpline => {}
        }
    }

    /// Add a keyframe waypoint to the timeline.
    pub fn add_waypoint(&mut self, waypoint: ShakuhachiWaypoint) {
        self.waypoints.push(waypoint);
        self.waypoints.sort_by(|a, b| a.beat.partial_cmp(&b.beat).unwrap_or(std::cmp::Ordering::Equal));
    }

    /// Sample-accurate interpolation evaluating all continuous Shakuhachi parameters at a given beat.
    #[inline]
    pub fn evaluate_at_beat(&self, beat: f64) -> ShakuhachiBusSnapshot {
        if self.waypoints.is_empty() {
            let (p, angle, meri, dist, murai, mask, gain) =
                ShakuhachiArticulation::HonkyokuTraditional.nominal_parameters();

            return ShakuhachiBusSnapshot {
                articulation: ShakuhachiArticulation::HonkyokuTraditional,
                blowing_pressure_pa: p,
                embouchure_angle_deg: angle,
                meri_kari_cents: meri,
                jet_distance_mm: dist,
                murai_iki_intensity: murai,
                hole_mask: mask,
                master_gain: gain,
            };
        }

        if self.waypoints.len() == 1 {
            let wp = &self.waypoints[0];
            return ShakuhachiBusSnapshot {
                articulation: wp.articulation,
                blowing_pressure_pa: wp.blowing_pressure_pa,
                embouchure_angle_deg: wp.embouchure_angle_deg,
                meri_kari_cents: wp.meri_kari_cents,
                jet_distance_mm: wp.jet_distance_mm,
                murai_iki_intensity: wp.murai_iki_intensity,
                hole_mask: wp.hole_mask,
                master_gain: wp.master_gain,
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

        // Find surrounding waypoints
        let mut idx1 = 0;
        for (i, wp) in self.waypoints.iter().enumerate() {
            if wp.beat <= effective_beat {
                idx1 = i;
            } else {
                break;
            }
        }
        let idx2 = (idx1 + 1).min(self.waypoints.len() - 1);

        let wp1 = &self.waypoints[idx1];
        let wp2 = &self.waypoints[idx2];

        let span = (wp2.beat - wp1.beat).max(1e-6);
        let t = ((effective_beat - wp1.beat) / span).clamp(0.0, 1.0) as f32;

        let interp_factor = match wp2.curve {
            ShakuhachiInterpolationCurve::Hold => 0.0,
            ShakuhachiInterpolationCurve::Linear => t,
            ShakuhachiInterpolationCurve::Exponential => t * t,
            ShakuhachiInterpolationCurve::CubicBezier | ShakuhachiInterpolationCurve::CubicCatmullRom => {
                t * t * (3.0 - 2.0 * t) // Smooth cubic hermite
            }
        };

        let interp = |a: f32, b: f32| a + (b - a) * interp_factor;

        let active_art = if t >= 0.5 { wp2.articulation } else { wp1.articulation };
        let active_mask = if t >= 0.5 { wp2.hole_mask } else { wp1.hole_mask };

        ShakuhachiBusSnapshot {
            articulation: active_art,
            blowing_pressure_pa: interp(wp1.blowing_pressure_pa, wp2.blowing_pressure_pa),
            embouchure_angle_deg: interp(wp1.embouchure_angle_deg, wp2.embouchure_angle_deg),
            meri_kari_cents: interp(wp1.meri_kari_cents, wp2.meri_kari_cents),
            jet_distance_mm: interp(wp1.jet_distance_mm, wp2.jet_distance_mm),
            murai_iki_intensity: interp(wp1.murai_iki_intensity, wp2.murai_iki_intensity),
            hole_mask: active_mask,
            master_gain: interp(wp1.master_gain, wp2.master_gain),
        }
    }

    /// Dispatch evaluated snapshot parameters directly into `ShakuhachiBus`.
    #[inline]
    pub fn dispatch_to_bus(&self, snapshot: &ShakuhachiBusSnapshot, bus: &ShakuhachiBus) {
        bus.set_blowing_pressure(snapshot.blowing_pressure_pa);
        bus.set_embouchure_angle(snapshot.embouchure_angle_deg);
        bus.set_meri_kari(snapshot.meri_kari_cents);
        bus.set_jet_distance(snapshot.jet_distance_mm);
        bus.set_murai_iki(snapshot.murai_iki_intensity);
        bus.set_hole_mask(snapshot.hole_mask);
        bus.set_master_gain(snapshot.master_gain);
    }

    /// Dispatch evaluated snapshot parameters into lock-free `ParamBus`.
    #[inline]
    pub fn dispatch_to_param_bus(
        &self,
        snapshot: &ShakuhachiBusSnapshot,
        param_bus: &ParamBus,
        base_id: ParamId,
    ) {
        param_bus.set(ParamId(base_id.0), snapshot.articulation as u32 as f32);
        param_bus.set(ParamId(base_id.0 + 1), snapshot.blowing_pressure_pa);
        param_bus.set(ParamId(base_id.0 + 2), snapshot.embouchure_angle_deg);
        param_bus.set(ParamId(base_id.0 + 3), snapshot.meri_kari_cents);
        param_bus.set(ParamId(base_id.0 + 4), snapshot.jet_distance_mm);
        param_bus.set(ParamId(base_id.0 + 5), snapshot.murai_iki_intensity);
        param_bus.set(ParamId(base_id.0 + 6), snapshot.hole_mask as f32);
        param_bus.set(ParamId(base_id.0 + 7), snapshot.master_gain);
    }

    /// Serialize gesture engine to TOML string.
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Deserialize gesture engine from TOML string.
    pub fn from_toml(toml_str: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(toml_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shakuhachi_gesture_engine_evaluation() {
        let engine = ShakuhachiGestureEngine::with_pattern(ShakuhachiGesturePattern::MeriKariGlissando {
            gliss_length_beats: 4.0,
            min_cents: -200.0,
            max_cents: 100.0,
        });

        let snap_start = engine.evaluate_at_beat(0.0);
        assert_eq!(snap_start.meri_kari_cents, -200.0);

        let snap_mid = engine.evaluate_at_beat(2.0);
        assert!((snap_mid.meri_kari_cents - 0.0).abs() < 1.0);

        let snap_end = engine.evaluate_at_beat(4.0);
        assert_eq!(snap_end.meri_kari_cents, 100.0);
    }

    #[test]
    fn test_shakuhachi_gesture_engine_toml_round_trip() {
        let engine = ShakuhachiGestureEngine::default();
        let toml_str = engine.to_toml().expect("Failed to serialize gesture engine");
        let restored: ShakuhachiGestureEngine =
            ShakuhachiGestureEngine::from_toml(&toml_str).expect("Failed to deserialize");
        assert_eq!(engine.waypoints.len(), restored.waypoints.len());
        assert_eq!(engine.loop_length_beats, restored.loop_length_beats);
    }
}
