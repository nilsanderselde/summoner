// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Dynamic Pipe Organ Stop Registration Timeline & Gesture Engine (Milestone 24).
//!
//! Provides multi-dimensional continuous physical modeling trajectory generation for
//! pipe organ registration transitions, windchest pressure swells, cutup mouth variations,
//! chiff attack transients, tracker action key velocity dynamics, and pneumatic tremulant modulation
//! with Catmull-Rom spline smoothing, TOML serialization, and sample-accurate lock-free
//! parameter dispatch into `PipeOrganBus` and `ParamBus`.
//!
//! Enforces zero heap allocations in real-time execution loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use summoner_core::param_bus::{ParamBus, ParamId};
use summoner_core::pipe_organ_bus::{
    PipeOrganArticulation, PipeOrganBus, PipeOrganBusSnapshot, PIPE_ORGAN_BASE_PARAM_ID,
};

/// Interpolation curve between discrete Pipe Organ keyframe waypoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PipeOrganInterpolationCurve {
    #[default]
    CubicCatmullRom,
    CubicBezier,
    Linear,
    Hold,
    Exponential,
}

/// A single keyframed multi-dimensional Pipe Organ articulation waypoint.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PipeOrganWaypoint {
    /// Timeline beat position in quarter note beats.
    pub beat: f64,
    /// 8-stop mechanical registration bitmask.
    pub stops_mask: u32,
    /// Windchest pressure in mmH2O [40.0 ..= 160.0].
    pub wind_pressure_mmh2o: f32,
    /// Cutup mouth height-to-width ratio [0.15 ..= 0.50].
    pub cutup_ratio: f32,
    /// Attack chiff transient duration in ms [5.0 ..= 80.0].
    pub chiff_duration_ms: f32,
    /// Tracker action key velocity / opening speed [0.0 ..= 1.0].
    pub tracker_velocity: f32,
    /// Pneumatic tremulant enabled.
    pub tremulant_enabled: bool,
    /// Tremulant pulsation frequency in Hz [2.0 ..= 10.0].
    pub tremulant_rate_hz: f32,
    /// Tremulant modulation depth [0.0 ..= 0.5].
    pub tremulant_depth: f32,
    /// Master gain [0.0 ..= 2.0].
    pub master_gain: f32,
    /// Articulation technique.
    pub articulation: PipeOrganArticulation,
    /// Interpolation curve leading to this waypoint.
    pub curve: PipeOrganInterpolationCurve,
}

impl Default for PipeOrganWaypoint {
    fn default() -> Self {
        let (mask, p, cutup, chiff, tracker, trem_en, trem_rate, trem_depth, gain) =
            PipeOrganArticulation::ToccataPlenum.nominal_parameters();

        Self {
            beat: 0.0,
            stops_mask: mask,
            wind_pressure_mmh2o: p,
            cutup_ratio: cutup,
            chiff_duration_ms: chiff,
            tracker_velocity: tracker,
            tremulant_enabled: trem_en,
            tremulant_rate_hz: trem_rate,
            tremulant_depth: trem_depth,
            master_gain: gain,
            articulation: PipeOrganArticulation::ToccataPlenum,
            curve: PipeOrganInterpolationCurve::CubicCatmullRom,
        }
    }
}

/// Pre-configured algorithmic Pipe Organ gesture trajectory patterns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PipeOrganGesturePattern {
    /// Grand Baroque Tutti plenum with opening stops and wind reservoir swells.
    ToccataPlenum {
        phrase_length_beats: f64,
        max_pressure_mmh2o: f32,
    },
    /// Gentle devotional chorale with subtle soft swell.
    BaroqueChorale {
        phrase_length_beats: f64,
        target_pressure_mmh2o: f32,
    },
    /// Massive Gothic crescendo with gradual reed and mixture additions.
    GothicCathedral {
        crescendo_length_beats: f64,
        peak_pressure_mmh2o: f32,
    },
    /// French Romantic symphonic crescendo (Flutes -> Principals -> Reeds Tutti).
    RomanticSymphonicCrescendo {
        total_length_beats: f64,
    },
    /// Pneumatic tremulant pulsing swell on Vox Humana and Bourdon.
    TremulantSwell {
        phrase_length_beats: f64,
        max_depth: f32,
    },
    /// User-defined multi-keyframe Catmull-Rom spline trajectory.
    CustomSpline,
}

impl Default for PipeOrganGesturePattern {
    fn default() -> Self {
        Self::ToccataPlenum {
            phrase_length_beats: 8.0,
            max_pressure_mmh2o: 90.0,
        }
    }
}

/// Dynamic Pipe Organ Articulation Timeline & Gesture Engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PipeOrganGestureEngine {
    /// Keyframed articulation waypoints.
    pub waypoints: Vec<PipeOrganWaypoint>,
    /// Active gesture trajectory pattern.
    pub pattern: PipeOrganGesturePattern,
    /// Default interpolation curve.
    pub default_curve: PipeOrganInterpolationCurve,
    /// Total sequence loop length in beats.
    pub loop_length_beats: f64,
    /// Whether pattern loops continuously.
    pub loop_enabled: bool,
    /// Whether gesture engine is currently active.
    pub is_active: bool,
}

impl Default for PipeOrganGestureEngine {
    fn default() -> Self {
        let mut engine = Self {
            waypoints: Vec::new(),
            pattern: PipeOrganGesturePattern::default(),
            default_curve: PipeOrganInterpolationCurve::CubicCatmullRom,
            loop_length_beats: 8.0,
            loop_enabled: true,
            is_active: true,
        };
        engine.generate_pattern_waypoints();
        engine
    }
}

impl PipeOrganGestureEngine {
    /// Creates a new PipeOrganGestureEngine initialized with the specified pattern.
    pub fn new(pattern: PipeOrganGesturePattern) -> Self {
        let mut engine = Self {
            waypoints: Vec::new(),
            pattern,
            default_curve: PipeOrganInterpolationCurve::CubicCatmullRom,
            loop_length_beats: 8.0,
            loop_enabled: true,
            is_active: true,
        };
        engine.generate_pattern_waypoints();
        engine
    }

    /// Regenerates keyframe waypoints based on the active pattern.
    pub fn generate_pattern_waypoints(&mut self) {
        self.waypoints.clear();

        match &self.pattern {
            PipeOrganGesturePattern::ToccataPlenum {
                phrase_length_beats,
                max_pressure_mmh2o,
            } => {
                self.loop_length_beats = *phrase_length_beats;
                self.waypoints.push(PipeOrganWaypoint {
                    beat: 0.0,
                    stops_mask: 119, // Principal 8', Bourdon 16', Octave 4', Super Octave 2', Mixture IV, Trompette 8'
                    wind_pressure_mmh2o: 80.0,
                    cutup_ratio: 0.25,
                    chiff_duration_ms: 28.0,
                    tracker_velocity: 0.85,
                    tremulant_enabled: false,
                    tremulant_rate_hz: 5.5,
                    tremulant_depth: 0.10,
                    master_gain: 0.85,
                    articulation: PipeOrganArticulation::ToccataPlenum,
                    curve: PipeOrganInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(PipeOrganWaypoint {
                    beat: phrase_length_beats * 0.5,
                    stops_mask: 119,
                    wind_pressure_mmh2o: *max_pressure_mmh2o,
                    cutup_ratio: 0.28,
                    chiff_duration_ms: 32.0,
                    tracker_velocity: 0.95,
                    tremulant_enabled: false,
                    tremulant_rate_hz: 5.5,
                    tremulant_depth: 0.10,
                    master_gain: 0.90,
                    articulation: PipeOrganArticulation::ToccataPlenum,
                    curve: PipeOrganInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(PipeOrganWaypoint {
                    beat: *phrase_length_beats,
                    stops_mask: 119,
                    wind_pressure_mmh2o: 80.0,
                    cutup_ratio: 0.25,
                    chiff_duration_ms: 28.0,
                    tracker_velocity: 0.85,
                    tremulant_enabled: false,
                    tremulant_rate_hz: 5.5,
                    tremulant_depth: 0.10,
                    master_gain: 0.85,
                    articulation: PipeOrganArticulation::ToccataPlenum,
                    curve: PipeOrganInterpolationCurve::CubicCatmullRom,
                });
            }

            PipeOrganGesturePattern::BaroqueChorale {
                phrase_length_beats,
                target_pressure_mmh2o,
            } => {
                self.loop_length_beats = *phrase_length_beats;
                self.waypoints.push(PipeOrganWaypoint {
                    beat: 0.0,
                    stops_mask: 11, // Principal 8', Bourdon 16', Flute 4'
                    wind_pressure_mmh2o: 60.0,
                    cutup_ratio: 0.30,
                    chiff_duration_ms: 35.0,
                    tracker_velocity: 0.60,
                    tremulant_enabled: false,
                    tremulant_rate_hz: 5.0,
                    tremulant_depth: 0.08,
                    master_gain: 0.78,
                    articulation: PipeOrganArticulation::BaroqueChorale,
                    curve: PipeOrganInterpolationCurve::Linear,
                });

                self.waypoints.push(PipeOrganWaypoint {
                    beat: phrase_length_beats * 0.5,
                    stops_mask: 11,
                    wind_pressure_mmh2o: *target_pressure_mmh2o,
                    cutup_ratio: 0.32,
                    chiff_duration_ms: 38.0,
                    tracker_velocity: 0.70,
                    tremulant_enabled: false,
                    tremulant_rate_hz: 5.0,
                    tremulant_depth: 0.08,
                    master_gain: 0.82,
                    articulation: PipeOrganArticulation::BaroqueChorale,
                    curve: PipeOrganInterpolationCurve::Linear,
                });

                self.waypoints.push(PipeOrganWaypoint {
                    beat: *phrase_length_beats,
                    stops_mask: 11,
                    wind_pressure_mmh2o: 60.0,
                    cutup_ratio: 0.30,
                    chiff_duration_ms: 35.0,
                    tracker_velocity: 0.60,
                    tremulant_enabled: false,
                    tremulant_rate_hz: 5.0,
                    tremulant_depth: 0.08,
                    master_gain: 0.78,
                    articulation: PipeOrganArticulation::BaroqueChorale,
                    curve: PipeOrganInterpolationCurve::Linear,
                });
            }

            PipeOrganGesturePattern::GothicCathedral {
                crescendo_length_beats,
                peak_pressure_mmh2o,
            } => {
                self.loop_length_beats = *crescendo_length_beats;
                self.waypoints.push(PipeOrganWaypoint {
                    beat: 0.0,
                    stops_mask: 3, // Bourdon 16' + Principal 8'
                    wind_pressure_mmh2o: 75.0,
                    cutup_ratio: 0.22,
                    chiff_duration_ms: 20.0,
                    tracker_velocity: 0.70,
                    tremulant_enabled: false,
                    tremulant_rate_hz: 6.0,
                    tremulant_depth: 0.12,
                    master_gain: 0.80,
                    articulation: PipeOrganArticulation::GothicCathedral,
                    curve: PipeOrganInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(PipeOrganWaypoint {
                    beat: crescendo_length_beats * 0.4,
                    stops_mask: 31, // + Octave 4', Flute 4', Super Octave 2'
                    wind_pressure_mmh2o: 90.0,
                    cutup_ratio: 0.24,
                    chiff_duration_ms: 24.0,
                    tracker_velocity: 0.85,
                    tremulant_enabled: false,
                    tremulant_rate_hz: 6.0,
                    tremulant_depth: 0.12,
                    master_gain: 0.88,
                    articulation: PipeOrganArticulation::GothicCathedral,
                    curve: PipeOrganInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(PipeOrganWaypoint {
                    beat: crescendo_length_beats * 0.8,
                    stops_mask: 255, // Full Tutti Grand Orgue
                    wind_pressure_mmh2o: *peak_pressure_mmh2o,
                    cutup_ratio: 0.26,
                    chiff_duration_ms: 28.0,
                    tracker_velocity: 1.00,
                    tremulant_enabled: false,
                    tremulant_rate_hz: 6.0,
                    tremulant_depth: 0.12,
                    master_gain: 0.95,
                    articulation: PipeOrganArticulation::GothicCathedral,
                    curve: PipeOrganInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(PipeOrganWaypoint {
                    beat: *crescendo_length_beats,
                    stops_mask: 255,
                    wind_pressure_mmh2o: *peak_pressure_mmh2o,
                    cutup_ratio: 0.26,
                    chiff_duration_ms: 28.0,
                    tracker_velocity: 1.00,
                    tremulant_enabled: false,
                    tremulant_rate_hz: 6.0,
                    tremulant_depth: 0.12,
                    master_gain: 0.95,
                    articulation: PipeOrganArticulation::GothicCathedral,
                    curve: PipeOrganInterpolationCurve::CubicCatmullRom,
                });
            }

            PipeOrganGesturePattern::RomanticSymphonicCrescendo {
                total_length_beats,
            } => {
                self.loop_length_beats = *total_length_beats;
                let steps = 4;
                for i in 0..=steps {
                    let frac = i as f64 / steps as f64;
                    let beat = frac * total_length_beats;
                    let mask = match i {
                        0 => 8,       // Flute 4'
                        1 => 11,      // Flute 4' + Bourdon 16' + Principal 8'
                        2 => 27,      // + Super Octave 2'
                        3 => 91,      // + Trompette 8'
                        _ => 255,     // Full Tutti
                    };
                    self.waypoints.push(PipeOrganWaypoint {
                        beat,
                        stops_mask: mask,
                        wind_pressure_mmh2o: 65.0 + (frac as f32 * 45.0),
                        cutup_ratio: 0.20 + (frac as f32 * 0.08),
                        chiff_duration_ms: 20.0 + (frac as f32 * 15.0),
                        tracker_velocity: 0.60 + (frac as f32 * 0.35),
                        tremulant_enabled: false,
                        tremulant_rate_hz: 5.5,
                        tremulant_depth: 0.10,
                        master_gain: 0.75 + (frac as f32 * 0.20),
                        articulation: PipeOrganArticulation::FrenchRomanticReed,
                        curve: PipeOrganInterpolationCurve::Linear,
                    });
                }
            }

            PipeOrganGesturePattern::TremulantSwell {
                phrase_length_beats,
                max_depth,
            } => {
                self.loop_length_beats = *phrase_length_beats;
                self.waypoints.push(PipeOrganWaypoint {
                    beat: 0.0,
                    stops_mask: 138, // Bourdon 16', Flute 4', Vox Humana 8'
                    wind_pressure_mmh2o: 70.0,
                    cutup_ratio: 0.25,
                    chiff_duration_ms: 35.0,
                    tracker_velocity: 0.70,
                    tremulant_enabled: true,
                    tremulant_rate_hz: 5.2,
                    tremulant_depth: 0.05,
                    master_gain: 0.80,
                    articulation: PipeOrganArticulation::VocalVoxHumana,
                    curve: PipeOrganInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(PipeOrganWaypoint {
                    beat: phrase_length_beats * 0.5,
                    stops_mask: 138,
                    wind_pressure_mmh2o: 74.0,
                    cutup_ratio: 0.28,
                    chiff_duration_ms: 40.0,
                    tracker_velocity: 0.80,
                    tremulant_enabled: true,
                    tremulant_rate_hz: 6.0,
                    tremulant_depth: *max_depth,
                    master_gain: 0.85,
                    articulation: PipeOrganArticulation::TremulantSwell,
                    curve: PipeOrganInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(PipeOrganWaypoint {
                    beat: *phrase_length_beats,
                    stops_mask: 138,
                    wind_pressure_mmh2o: 70.0,
                    cutup_ratio: 0.25,
                    chiff_duration_ms: 35.0,
                    tracker_velocity: 0.70,
                    tremulant_enabled: true,
                    tremulant_rate_hz: 5.2,
                    tremulant_depth: 0.05,
                    master_gain: 0.80,
                    articulation: PipeOrganArticulation::VocalVoxHumana,
                    curve: PipeOrganInterpolationCurve::CubicCatmullRom,
                });
            }

            PipeOrganGesturePattern::CustomSpline => {
                if self.waypoints.is_empty() {
                    self.waypoints.push(PipeOrganWaypoint::default());
                }
            }
        }
    }

    /// Evaluates continuous parameters at the specified beat position.
    pub fn evaluate_at_beat(&self, beat: f64) -> PipeOrganBusSnapshot {
        if self.waypoints.is_empty() {
            return PipeOrganBusSnapshot {
                stops_mask: 119,
                wind_pressure_mmh2o: 85.0,
                cutup_ratio: 0.25,
                chiff_duration_ms: 28.0,
                tracker_velocity: 0.90,
                tremulant_enabled: false,
                tremulant_rate_hz: 5.5,
                tremulant_depth: 0.10,
                master_gain: 0.85,
                articulation: PipeOrganArticulation::ToccataPlenum,
            };
        }

        if self.waypoints.len() == 1 {
            let wp = self.waypoints[0];
            return PipeOrganBusSnapshot {
                stops_mask: wp.stops_mask,
                wind_pressure_mmh2o: wp.wind_pressure_mmh2o,
                cutup_ratio: wp.cutup_ratio,
                chiff_duration_ms: wp.chiff_duration_ms,
                tracker_velocity: wp.tracker_velocity,
                tremulant_enabled: wp.tremulant_enabled,
                tremulant_rate_hz: wp.tremulant_rate_hz,
                tremulant_depth: wp.tremulant_depth,
                master_gain: wp.master_gain,
                articulation: wp.articulation,
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

        // Find bounding indices for 4-point Catmull-Rom spline
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

        let pressure = match curve {
            PipeOrganInterpolationCurve::Hold => w1.wind_pressure_mmh2o,
            PipeOrganInterpolationCurve::Linear => lerp(w1.wind_pressure_mmh2o, w2.wind_pressure_mmh2o, t),
            _ => catmull_rom_1d(w0.wind_pressure_mmh2o, w1.wind_pressure_mmh2o, w2.wind_pressure_mmh2o, w3.wind_pressure_mmh2o, t),
        }.clamp(40.0, 160.0);

        let cutup = match curve {
            PipeOrganInterpolationCurve::Hold => w1.cutup_ratio,
            PipeOrganInterpolationCurve::Linear => lerp(w1.cutup_ratio, w2.cutup_ratio, t),
            _ => catmull_rom_1d(w0.cutup_ratio, w1.cutup_ratio, w2.cutup_ratio, w3.cutup_ratio, t),
        }.clamp(0.15, 0.50);

        let chiff = match curve {
            PipeOrganInterpolationCurve::Hold => w1.chiff_duration_ms,
            PipeOrganInterpolationCurve::Linear => lerp(w1.chiff_duration_ms, w2.chiff_duration_ms, t),
            _ => catmull_rom_1d(w0.chiff_duration_ms, w1.chiff_duration_ms, w2.chiff_duration_ms, w3.chiff_duration_ms, t),
        }.clamp(5.0, 80.0);

        let tracker = match curve {
            PipeOrganInterpolationCurve::Hold => w1.tracker_velocity,
            PipeOrganInterpolationCurve::Linear => lerp(w1.tracker_velocity, w2.tracker_velocity, t),
            _ => catmull_rom_1d(w0.tracker_velocity, w1.tracker_velocity, w2.tracker_velocity, w3.tracker_velocity, t),
        }.clamp(0.0, 1.0);

        let trem_rate = match curve {
            PipeOrganInterpolationCurve::Hold => w1.tremulant_rate_hz,
            PipeOrganInterpolationCurve::Linear => lerp(w1.tremulant_rate_hz, w2.tremulant_rate_hz, t),
            _ => catmull_rom_1d(w0.tremulant_rate_hz, w1.tremulant_rate_hz, w2.tremulant_rate_hz, w3.tremulant_rate_hz, t),
        }.clamp(1.0, 15.0);

        let trem_depth = match curve {
            PipeOrganInterpolationCurve::Hold => w1.tremulant_depth,
            PipeOrganInterpolationCurve::Linear => lerp(w1.tremulant_depth, w2.tremulant_depth, t),
            _ => catmull_rom_1d(w0.tremulant_depth, w1.tremulant_depth, w2.tremulant_depth, w3.tremulant_depth, t),
        }.clamp(0.0, 0.5);

        let gain = match curve {
            PipeOrganInterpolationCurve::Hold => w1.master_gain,
            PipeOrganInterpolationCurve::Linear => lerp(w1.master_gain, w2.master_gain, t),
            _ => catmull_rom_1d(w0.master_gain, w1.master_gain, w2.master_gain, w3.master_gain, t),
        }.clamp(0.0, 2.0);

        // Discrete stop switches switch at midpoint or hold from w1
        let stops = if t < 0.5 { w1.stops_mask } else { w2.stops_mask };
        let trem_en = if t < 0.5 { w1.tremulant_enabled } else { w2.tremulant_enabled };
        let art = if t < 0.5 { w1.articulation } else { w2.articulation };

        PipeOrganBusSnapshot {
            stops_mask: stops,
            wind_pressure_mmh2o: pressure,
            cutup_ratio: cutup,
            chiff_duration_ms: chiff,
            tracker_velocity: tracker,
            tremulant_enabled: trem_en,
            tremulant_rate_hz: trem_rate,
            tremulant_depth: trem_depth,
            master_gain: gain,
            articulation: art,
        }
    }

    /// Dispatches evaluated state at `beat` into `PipeOrganBus` and optional `ParamBus`.
    pub fn dispatch_to_bus(
        &self,
        beat: f64,
        bus: &PipeOrganBus,
        param_bus: Option<&ParamBus>,
        base_id: Option<ParamId>,
    ) {
        let snap = self.evaluate_at_beat(beat);
        bus.write_snapshot(&snap);

        if let Some(pb) = param_bus {
            let id = base_id.unwrap_or(PIPE_ORGAN_BASE_PARAM_ID);
            bus.dispatch_to_param_bus(pb, id);
        }
    }

    /// Evaluates and writes snapshot to `PipeOrganBus`.
    pub fn apply_to_bus(&self, beat: f64, bus: &PipeOrganBus) {
        let snap = self.evaluate_at_beat(beat);
        bus.write_snapshot(&snap);
    }

    /// Dispatches snapshot to `ParamBus`.
    pub fn dispatch_to_param_bus(&self, beat: f64, param_bus: &ParamBus, base_id: ParamId) {
        let snap = self.evaluate_at_beat(beat);
        let bus = PipeOrganBus::new();
        bus.write_snapshot(&snap);
        bus.dispatch_to_param_bus(param_bus, base_id);
    }
}

/// 1D Linear interpolation helper.
#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

/// 1D Catmull-Rom cubic spline interpolation helper.
#[inline]
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
    fn test_pipe_organ_gesture_engine_toccata_pattern() {
        let engine = PipeOrganGestureEngine::new(PipeOrganGesturePattern::ToccataPlenum {
            phrase_length_beats: 8.0,
            max_pressure_mmh2o: 95.0,
        });

        let start = engine.evaluate_at_beat(0.0);
        assert_eq!(start.wind_pressure_mmh2o, 80.0);
        assert_eq!(start.stops_mask, 119);

        let mid = engine.evaluate_at_beat(4.0);
        assert!((mid.wind_pressure_mmh2o - 95.0).abs() < 0.1);

        let end = engine.evaluate_at_beat(8.0);
        assert_eq!(end.wind_pressure_mmh2o, 80.0);
    }

    #[test]
    fn test_pipe_organ_gesture_toml_round_trip() {
        let engine = PipeOrganGestureEngine::new(PipeOrganGesturePattern::RomanticSymphonicCrescendo {
            total_length_beats: 16.0,
        });

        let toml_str = toml::to_string_pretty(&engine).expect("Failed to serialize PipeOrganGestureEngine to TOML");
        let deserialized: PipeOrganGestureEngine =
            toml::from_str(&toml_str).expect("Failed to deserialize PipeOrganGestureEngine from TOML");

        assert_eq!(engine.waypoints.len(), deserialized.waypoints.len());
        assert_eq!(engine.loop_length_beats, deserialized.loop_length_beats);
    }
}
