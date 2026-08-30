// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Dynamic Electric Piano Articulation Timeline & Gesture Engine (Milestone 22).
//!
//! Provides multi-dimensional continuous physical modeling trajectory generation for
//! electromechanical tine and reed electric pianos (Rhodes & Wurlitzer) supporting dynamic
//! Suitcase stereo tremolo swells, Funk barking solos, Dyno-My-Rhodes bell sweeps, and
//! overdriven Soul grit with Catmull-Rom spline smoothing, TOML serialization, and
//! sample-accurate lock-free dispatch into `EpBus` and `ParamBus`.
//!
//! Enforces zero heap allocations in real-time execution loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use summoner_core::ep_bus::{EpArticulation, EpBus, EpBusSnapshot, EpModelType};
use summoner_core::param_bus::{ParamBus, ParamId};

/// Interpolation curve between discrete electric piano keyframe waypoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EpInterpolationCurve {
    #[default]
    CubicCatmullRom,
    CubicBezier,
    Linear,
    Hold,
    Exponential,
}

/// A single keyframed multi-dimensional electric piano articulation waypoint.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EpWaypoint {
    /// Timeline beat position in quarter note beats.
    pub beat: f64,
    /// Model type (Rhodes Tine / Wurlitzer Reed).
    pub model_type: EpModelType,
    /// Hammer hardness $[0.0 ..= 1.0]$.
    pub hammer_hardness: f32,
    /// Pickup air-gap distance in mm $[0.5 ..= 5.0\text{mm}]$.
    pub air_gap_mm: f32,
    /// Inductive pickup barking overdrive $[0.0 ..= 1.0]$.
    pub bark_drive: f32,
    /// Resonant tonebar coupling stiffness $[0.0 ..= 1.0]$.
    pub tonebar_coupling: f32,
    /// Damper release clunk volume $[0.0 ..= 1.0]$.
    pub damper_clunk_volume: f32,
    /// Stereo optical tremolo enabled.
    pub tremolo_enabled: bool,
    /// Tremolo rate in Hz $[0.2 ..= 20.0\text{Hz}]$.
    pub tremolo_rate_hz: f32,
    /// Tremolo depth $[0.0 ..= 1.0]$.
    pub tremolo_depth: f32,
    /// Tremolo stereo ping-pong mode.
    pub tremolo_stereo: bool,
    /// Tube preamp drive in dB $[0.0 ..= 24.0\text{dB}]$.
    pub tube_drive_db: f32,
    /// Bass gain in dB $[-12.0 ..= 12.0\text{dB}]$.
    pub bass_db: f32,
    /// Treble gain in dB $[-12.0 ..= 12.0\text{dB}]$.
    pub treble_db: f32,
    /// Mechanical articulation technique.
    pub articulation: EpArticulation,
    /// Interpolation curve leading to this waypoint.
    pub curve: EpInterpolationCurve,
}

impl Default for EpWaypoint {
    fn default() -> Self {
        let (model, hardness, air_gap, bark, tonebar, damper, trem_en, trem_rate, trem_depth, trem_stereo, drive, bass, treble) =
            EpArticulation::ClassicSuitcase.nominal_parameters();
        Self {
            beat: 0.0,
            model_type: model,
            hammer_hardness: hardness,
            air_gap_mm: air_gap,
            bark_drive: bark,
            tonebar_coupling: tonebar,
            damper_clunk_volume: damper,
            tremolo_enabled: trem_en,
            tremolo_rate_hz: trem_rate,
            tremolo_depth: trem_depth,
            tremolo_stereo: trem_stereo,
            tube_drive_db: drive,
            bass_db: bass,
            treble_db: treble,
            articulation: EpArticulation::ClassicSuitcase,
            curve: EpInterpolationCurve::CubicCatmullRom,
        }
    }
}

/// Pre-configured algorithmic electric piano gesture trajectory patterns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EpGesturePattern {
    /// Lush Rhodes Suitcase ballad with deep ping-pong stereo tremolo and warm bell tones.
    SuitcaseBallad {
        phrase_length_beats: f64,
        peak_tremolo_depth: f32,
    },
    /// Aggressive Funk barking solo with close air-gap, heavy barking distortion, and crisp attack.
    FunkBarkSolo {
        phrase_length_beats: f64,
        max_bark_drive: f32,
    },
    /// Classic Wurlitzer 200A reed groove with subtle mono optical tremolo and midrange bite.
    WurliGroove {
        groove_length_beats: f64,
        tremolo_rate_hz: f32,
    },
    /// Dyno-My-Rhodes crystalline bell swell with high treble boost and bright tonebar chime.
    BelledDynoSwell {
        swell_length_beats: f64,
        peak_treble_db: f32,
    },
    /// Saturated Soul Wurlitzer solo with overdriven tube preamp crunch.
    SoulOverdrive {
        solo_length_beats: f64,
        max_drive_db: f32,
    },
    /// User-defined multi-keyframe Catmull-Rom spline trajectory.
    CustomSpline,
}

impl Default for EpGesturePattern {
    fn default() -> Self {
        Self::SuitcaseBallad {
            phrase_length_beats: 8.0,
            peak_tremolo_depth: 0.85,
        }
    }
}

/// Dynamic Electric Piano Articulation Timeline & Gesture Engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpGestureEngine {
    /// Keyframed articulation waypoints.
    pub waypoints: Vec<EpWaypoint>,
    /// Active gesture trajectory pattern.
    pub pattern: EpGesturePattern,
    /// Default interpolation curve.
    pub default_curve: EpInterpolationCurve,
    /// Total sequence loop length in beats.
    pub loop_length_beats: f64,
    /// Whether pattern loops continuously.
    pub loop_enabled: bool,
    /// Whether gesture engine is currently active.
    pub is_active: bool,
}

impl Default for EpGestureEngine {
    fn default() -> Self {
        let mut engine = Self {
            waypoints: Vec::new(),
            pattern: EpGesturePattern::default(),
            default_curve: EpInterpolationCurve::CubicCatmullRom,
            loop_length_beats: 8.0,
            loop_enabled: true,
            is_active: true,
        };
        engine.generate_pattern_waypoints();
        engine
    }
}

impl EpGestureEngine {
    /// Creates a new EpGestureEngine initialized with the specified pattern.
    pub fn new(pattern: EpGesturePattern) -> Self {
        let mut engine = Self {
            waypoints: Vec::new(),
            pattern,
            default_curve: EpInterpolationCurve::CubicCatmullRom,
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
            EpGesturePattern::SuitcaseBallad {
                phrase_length_beats,
                peak_tremolo_depth,
            } => {
                self.loop_length_beats = *phrase_length_beats;
                self.waypoints.push(EpWaypoint {
                    beat: 0.0,
                    model_type: EpModelType::RhodesTine,
                    hammer_hardness: 0.40,
                    air_gap_mm: 2.0,
                    bark_drive: 0.40,
                    tonebar_coupling: 0.75,
                    damper_clunk_volume: 0.25,
                    tremolo_enabled: true,
                    tremolo_rate_hz: 4.8,
                    tremolo_depth: 0.40,
                    tremolo_stereo: true,
                    tube_drive_db: 3.0,
                    bass_db: 2.0,
                    treble_db: 2.0,
                    articulation: EpArticulation::BalladChime,
                    curve: EpInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(EpWaypoint {
                    beat: 0.5 * *phrase_length_beats,
                    model_type: EpModelType::RhodesTine,
                    hammer_hardness: 0.55,
                    air_gap_mm: 1.8,
                    bark_drive: 0.60,
                    tonebar_coupling: 0.70,
                    damper_clunk_volume: 0.35,
                    tremolo_enabled: true,
                    tremolo_rate_hz: 5.6,
                    tremolo_depth: *peak_tremolo_depth,
                    tremolo_stereo: true,
                    tube_drive_db: 5.5,
                    bass_db: 1.5,
                    treble_db: 3.5,
                    articulation: EpArticulation::ClassicSuitcase,
                    curve: EpInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(EpWaypoint {
                    beat: *phrase_length_beats,
                    model_type: EpModelType::RhodesTine,
                    hammer_hardness: 0.40,
                    air_gap_mm: 2.0,
                    bark_drive: 0.40,
                    tonebar_coupling: 0.75,
                    damper_clunk_volume: 0.25,
                    tremolo_enabled: true,
                    tremolo_rate_hz: 4.8,
                    tremolo_depth: 0.40,
                    tremolo_stereo: true,
                    tube_drive_db: 3.0,
                    bass_db: 2.0,
                    treble_db: 2.0,
                    articulation: EpArticulation::BalladChime,
                    curve: EpInterpolationCurve::CubicCatmullRom,
                });
            }

            EpGesturePattern::FunkBarkSolo {
                phrase_length_beats,
                max_bark_drive,
            } => {
                self.loop_length_beats = *phrase_length_beats;
                self.waypoints.push(EpWaypoint {
                    beat: 0.0,
                    model_type: EpModelType::RhodesTine,
                    hammer_hardness: 0.75,
                    air_gap_mm: 1.2,
                    bark_drive: 0.65,
                    tonebar_coupling: 0.80,
                    damper_clunk_volume: 0.40,
                    tremolo_enabled: false,
                    tremolo_rate_hz: 5.0,
                    tremolo_depth: 0.0,
                    tremolo_stereo: false,
                    tube_drive_db: 6.0,
                    bass_db: 0.0,
                    treble_db: 5.0,
                    articulation: EpArticulation::BarkingDyno,
                    curve: EpInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(EpWaypoint {
                    beat: 0.6 * *phrase_length_beats,
                    model_type: EpModelType::RhodesTine,
                    hammer_hardness: 0.90,
                    air_gap_mm: 0.9,
                    bark_drive: *max_bark_drive,
                    tonebar_coupling: 0.85,
                    damper_clunk_volume: 0.45,
                    tremolo_enabled: false,
                    tremolo_rate_hz: 5.0,
                    tremolo_depth: 0.0,
                    tremolo_stereo: false,
                    tube_drive_db: 10.0,
                    bass_db: -1.5,
                    treble_db: 7.5,
                    articulation: EpArticulation::BarkingDyno,
                    curve: EpInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(EpWaypoint {
                    beat: *phrase_length_beats,
                    model_type: EpModelType::RhodesTine,
                    hammer_hardness: 0.75,
                    air_gap_mm: 1.2,
                    bark_drive: 0.65,
                    tonebar_coupling: 0.80,
                    damper_clunk_volume: 0.40,
                    tremolo_enabled: false,
                    tremolo_rate_hz: 5.0,
                    tremolo_depth: 0.0,
                    tremolo_stereo: false,
                    tube_drive_db: 6.0,
                    bass_db: 0.0,
                    treble_db: 5.0,
                    articulation: EpArticulation::BarkingDyno,
                    curve: EpInterpolationCurve::CubicCatmullRom,
                });
            }

            EpGesturePattern::WurliGroove {
                groove_length_beats,
                tremolo_rate_hz,
            } => {
                self.loop_length_beats = *groove_length_beats;
                self.waypoints.push(EpWaypoint {
                    beat: 0.0,
                    model_type: EpModelType::WurlitzerReed,
                    hammer_hardness: 0.65,
                    air_gap_mm: 1.2,
                    bark_drive: 0.75,
                    tonebar_coupling: 0.35,
                    damper_clunk_volume: 0.45,
                    tremolo_enabled: true,
                    tremolo_rate_hz: *tremolo_rate_hz,
                    tremolo_depth: 0.50,
                    tremolo_stereo: false,
                    tube_drive_db: 6.0,
                    bass_db: 0.0,
                    treble_db: 3.0,
                    articulation: EpArticulation::ClassicWurli,
                    curve: EpInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(EpWaypoint {
                    beat: *groove_length_beats,
                    model_type: EpModelType::WurlitzerReed,
                    hammer_hardness: 0.65,
                    air_gap_mm: 1.2,
                    bark_drive: 0.75,
                    tonebar_coupling: 0.35,
                    damper_clunk_volume: 0.45,
                    tremolo_enabled: true,
                    tremolo_rate_hz: *tremolo_rate_hz,
                    tremolo_depth: 0.50,
                    tremolo_stereo: false,
                    tube_drive_db: 6.0,
                    bass_db: 0.0,
                    treble_db: 3.0,
                    articulation: EpArticulation::ClassicWurli,
                    curve: EpInterpolationCurve::CubicCatmullRom,
                });
            }

            EpGesturePattern::BelledDynoSwell {
                swell_length_beats,
                peak_treble_db,
            } => {
                self.loop_length_beats = *swell_length_beats;
                let steps = 4;
                for i in 0..=steps {
                    let frac = i as f64 / steps as f64;
                    let beat = frac * swell_length_beats;
                    let treble = *peak_treble_db * (frac as f32);

                    self.waypoints.push(EpWaypoint {
                        beat,
                        model_type: EpModelType::RhodesTine,
                        hammer_hardness: 0.50 + 0.35 * frac as f32,
                        air_gap_mm: 2.0 - 0.8 * frac as f32,
                        bark_drive: 0.45 + 0.45 * frac as f32,
                        tonebar_coupling: 0.90,
                        damper_clunk_volume: 0.25,
                        tremolo_enabled: true,
                        tremolo_rate_hz: 3.5 + 2.5 * frac as f32,
                        tremolo_depth: 0.60,
                        tremolo_stereo: true,
                        tube_drive_db: 3.0 + 3.0 * frac as f32,
                        bass_db: 2.0,
                        treble_db: treble,
                        articulation: EpArticulation::BelledAmbient,
                        curve: EpInterpolationCurve::CubicCatmullRom,
                    });
                }
            }

            EpGesturePattern::SoulOverdrive {
                solo_length_beats,
                max_drive_db,
            } => {
                self.loop_length_beats = *solo_length_beats;
                self.waypoints.push(EpWaypoint {
                    beat: 0.0,
                    model_type: EpModelType::WurlitzerReed,
                    hammer_hardness: 0.80,
                    air_gap_mm: 0.9,
                    bark_drive: 0.90,
                    tonebar_coupling: 0.40,
                    damper_clunk_volume: 0.50,
                    tremolo_enabled: true,
                    tremolo_rate_hz: 7.0,
                    tremolo_depth: 0.35,
                    tremolo_stereo: false,
                    tube_drive_db: *max_drive_db * 0.7,
                    bass_db: 2.0,
                    treble_db: 4.5,
                    articulation: EpArticulation::SoulOverdrive,
                    curve: EpInterpolationCurve::Linear,
                });

                self.waypoints.push(EpWaypoint {
                    beat: *solo_length_beats,
                    model_type: EpModelType::WurlitzerReed,
                    hammer_hardness: 0.85,
                    air_gap_mm: 0.8,
                    bark_drive: 0.95,
                    tonebar_coupling: 0.40,
                    damper_clunk_volume: 0.50,
                    tremolo_enabled: true,
                    tremolo_rate_hz: 7.5,
                    tremolo_depth: 0.40,
                    tremolo_stereo: false,
                    tube_drive_db: *max_drive_db,
                    bass_db: 2.5,
                    treble_db: 5.5,
                    articulation: EpArticulation::SoulOverdrive,
                    curve: EpInterpolationCurve::Linear,
                });
            }

            EpGesturePattern::CustomSpline => {
                if self.waypoints.is_empty() {
                    self.waypoints.push(EpWaypoint::default());
                }
            }
        }
    }

    /// Evaluates continuous parameters at the specified beat position.
    pub fn evaluate_at_beat(&self, beat: f64) -> EpBusSnapshot {
        if self.waypoints.is_empty() {
            return EpBusSnapshot::default();
        }

        if self.waypoints.len() == 1 {
            let wp = self.waypoints[0];
            return EpBusSnapshot {
                articulation: wp.articulation,
                model_type: wp.model_type,
                hammer_hardness: wp.hammer_hardness,
                air_gap_mm: wp.air_gap_mm,
                bark_drive: wp.bark_drive,
                tonebar_coupling: wp.tonebar_coupling,
                damper_clunk_volume: wp.damper_clunk_volume,
                tremolo_enabled: wp.tremolo_enabled,
                tremolo_rate_hz: wp.tremolo_rate_hz,
                tremolo_depth: wp.tremolo_depth,
                tremolo_stereo: wp.tremolo_stereo,
                tube_drive_db: wp.tube_drive_db,
                bass_db: wp.bass_db,
                treble_db: wp.treble_db,
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

        let hardness = match curve {
            EpInterpolationCurve::Hold => w1.hammer_hardness,
            EpInterpolationCurve::Linear => lerp(w1.hammer_hardness, w2.hammer_hardness, t),
            _ => catmull_rom_1d(w0.hammer_hardness, w1.hammer_hardness, w2.hammer_hardness, w3.hammer_hardness, t),
        }.clamp(0.0, 1.0);

        let air_gap = match curve {
            EpInterpolationCurve::Hold => w1.air_gap_mm,
            EpInterpolationCurve::Linear => lerp(w1.air_gap_mm, w2.air_gap_mm, t),
            _ => catmull_rom_1d(w0.air_gap_mm, w1.air_gap_mm, w2.air_gap_mm, w3.air_gap_mm, t),
        }.clamp(0.4, 6.0);

        let bark = match curve {
            EpInterpolationCurve::Hold => w1.bark_drive,
            EpInterpolationCurve::Linear => lerp(w1.bark_drive, w2.bark_drive, t),
            _ => catmull_rom_1d(w0.bark_drive, w1.bark_drive, w2.bark_drive, w3.bark_drive, t),
        }.clamp(0.0, 1.0);

        let tonebar = match curve {
            EpInterpolationCurve::Hold => w1.tonebar_coupling,
            EpInterpolationCurve::Linear => lerp(w1.tonebar_coupling, w2.tonebar_coupling, t),
            _ => catmull_rom_1d(w0.tonebar_coupling, w1.tonebar_coupling, w2.tonebar_coupling, w3.tonebar_coupling, t),
        }.clamp(0.0, 1.0);

        let damper = match curve {
            EpInterpolationCurve::Hold => w1.damper_clunk_volume,
            EpInterpolationCurve::Linear => lerp(w1.damper_clunk_volume, w2.damper_clunk_volume, t),
            _ => catmull_rom_1d(w0.damper_clunk_volume, w1.damper_clunk_volume, w2.damper_clunk_volume, w3.damper_clunk_volume, t),
        }.clamp(0.0, 1.0);

        let trem_rate = match curve {
            EpInterpolationCurve::Hold => w1.tremolo_rate_hz,
            EpInterpolationCurve::Linear => lerp(w1.tremolo_rate_hz, w2.tremolo_rate_hz, t),
            _ => catmull_rom_1d(w0.tremolo_rate_hz, w1.tremolo_rate_hz, w2.tremolo_rate_hz, w3.tremolo_rate_hz, t),
        }.clamp(0.2, 20.0);

        let trem_depth = match curve {
            EpInterpolationCurve::Hold => w1.tremolo_depth,
            EpInterpolationCurve::Linear => lerp(w1.tremolo_depth, w2.tremolo_depth, t),
            _ => catmull_rom_1d(w0.tremolo_depth, w1.tremolo_depth, w2.tremolo_depth, w3.tremolo_depth, t),
        }.clamp(0.0, 1.0);

        let drive = match curve {
            EpInterpolationCurve::Hold => w1.tube_drive_db,
            EpInterpolationCurve::Linear => lerp(w1.tube_drive_db, w2.tube_drive_db, t),
            _ => catmull_rom_1d(w0.tube_drive_db, w1.tube_drive_db, w2.tube_drive_db, w3.tube_drive_db, t),
        }.clamp(0.0, 24.0);

        let bass = match curve {
            EpInterpolationCurve::Hold => w1.bass_db,
            EpInterpolationCurve::Linear => lerp(w1.bass_db, w2.bass_db, t),
            _ => catmull_rom_1d(w0.bass_db, w1.bass_db, w2.bass_db, w3.bass_db, t),
        }.clamp(-12.0, 12.0);

        let treble = match curve {
            EpInterpolationCurve::Hold => w1.treble_db,
            EpInterpolationCurve::Linear => lerp(w1.treble_db, w2.treble_db, t),
            _ => catmull_rom_1d(w0.treble_db, w1.treble_db, w2.treble_db, w3.treble_db, t),
        }.clamp(-12.0, 12.0);

        let progress = if self.loop_length_beats > 0.0 {
            (effective_beat / self.loop_length_beats).clamp(0.0, 1.0) as f32
        } else {
            0.0
        };

        EpBusSnapshot {
            articulation: if t > 0.5 { w2.articulation } else { w1.articulation },
            model_type: if t > 0.5 { w2.model_type } else { w1.model_type },
            hammer_hardness: hardness,
            air_gap_mm: air_gap,
            bark_drive: bark,
            tonebar_coupling: tonebar,
            damper_clunk_volume: damper,
            tremolo_enabled: w1.tremolo_enabled,
            tremolo_rate_hz: trem_rate,
            tremolo_depth: trem_depth,
            tremolo_stereo: w1.tremolo_stereo,
            tube_drive_db: drive,
            bass_db: bass,
            treble_db: treble,
            gesture_progress: progress,
            is_active: self.is_active,
        }
    }

    /// Evaluates and writes parameters into an `EpBus`.
    pub fn apply_to_bus(&self, beat: f64, bus: &EpBus) {
        let snap = self.evaluate_at_beat(beat);
        bus.apply_snapshot(&snap);
    }

    /// Dispatches evaluated state to central `ParamBus`.
    pub fn dispatch_to_param_bus(&self, beat: f64, param_bus: &ParamBus, base_id: ParamId) {
        let snap = self.evaluate_at_beat(beat);
        param_bus.set(ParamId(base_id.0), snap.articulation as u32 as f32);
        param_bus.set(ParamId(base_id.0 + 1), snap.model_type as u32 as f32);
        param_bus.set(ParamId(base_id.0 + 2), snap.hammer_hardness);
        param_bus.set(ParamId(base_id.0 + 3), snap.air_gap_mm);
        param_bus.set(ParamId(base_id.0 + 4), snap.bark_drive);
        param_bus.set(ParamId(base_id.0 + 5), snap.tonebar_coupling);
        param_bus.set(ParamId(base_id.0 + 6), snap.damper_clunk_volume);
        param_bus.set(ParamId(base_id.0 + 7), if snap.tremolo_enabled { 1.0 } else { 0.0 });
        param_bus.set(ParamId(base_id.0 + 8), snap.tremolo_rate_hz);
        param_bus.set(ParamId(base_id.0 + 9), snap.tremolo_depth);
        param_bus.set(ParamId(base_id.0 + 10), if snap.tremolo_stereo { 1.0 } else { 0.0 });
        param_bus.set(ParamId(base_id.0 + 11), snap.tube_drive_db);
        param_bus.set(ParamId(base_id.0 + 12), snap.bass_db);
        param_bus.set(ParamId(base_id.0 + 13), snap.treble_db);
        param_bus.set(ParamId(base_id.0 + 14), snap.gesture_progress);
    }

    /// Serializes engine configuration to TOML string.
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Deserializes engine configuration from TOML string.
    pub fn from_toml(toml_str: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(toml_str)
    }
}

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

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
    fn test_ep_gesture_suitcase_ballad_evaluation() {
        let engine = EpGestureEngine::new(EpGesturePattern::SuitcaseBallad {
            phrase_length_beats: 8.0,
            peak_tremolo_depth: 0.85,
        });

        let snap_start = engine.evaluate_at_beat(0.0);
        assert_eq!(snap_start.model_type, EpModelType::RhodesTine);
        assert!(snap_start.tremolo_enabled);
        assert!((snap_start.tremolo_depth - 0.40).abs() < 1e-4);

        let snap_mid = engine.evaluate_at_beat(4.0);
        assert!((snap_mid.tremolo_depth - 0.85).abs() < 0.05);

        let snap_end = engine.evaluate_at_beat(8.0);
        assert!((snap_end.tremolo_depth - 0.40).abs() < 1e-4);
    }

    #[test]
    fn test_ep_gesture_dispatch_zero_allocation() {
        let engine = EpGestureEngine::default();
        let bus = EpBus::new();

        {
            let _guard = AllocGuard::new();
            for b in 0..100 {
                let beat = b as f64 * 0.1;
                engine.apply_to_bus(beat, &bus);
            }
        }
    }

    #[test]
    fn test_ep_gesture_toml_roundtrip() {
        let engine = EpGestureEngine::new(EpGesturePattern::BelledDynoSwell {
            swell_length_beats: 16.0,
            peak_treble_db: 8.0,
        });

        let toml_str = engine.to_toml().expect("Failed to serialize to TOML");
        let decoded: EpGestureEngine = EpGestureEngine::from_toml(&toml_str).expect("Failed to deserialize from TOML");

        assert_eq!(engine.waypoints.len(), decoded.waypoints.len());
        assert_eq!(engine.loop_length_beats, decoded.loop_length_beats);
    }
}
