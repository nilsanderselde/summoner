// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Dynamic Woodwind Fingering Articulation Timeline & Breath Vector Engine (Milestone 17).
//!
//! Provides multi-dimensional continuous woodwind breath gesture trajectory generation (Blowing Pressure,
//! Jet Distance, Embouchure Angle, 6-Tonehole Fingering Mask, Vibrato, Flutter Tongue, and Breath Noise)
//! supporting classical and modern extended techniques with cubic Catmull-Rom and Bézier smoothing,
//! TOML serialization, and sample-accurate lock-free dispatch into `BreathBus` and `ParamBus`.
//!
//! Enforces zero heap allocations in real-time execution loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use summoner_core::breath_bus::{BreathBus, BreathBusSnapshot, WoodwindArticulation};
use summoner_core::param_bus::{ParamBus, ParamId};

/// Interpolation mode between discrete woodwind articulation keyframe waypoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WoodwindInterpolationCurve {
    #[default]
    CubicCatmullRom,
    CubicBezier,
    Linear,
    Hold,
}

/// A single keyframed multi-dimensional woodwind articulation waypoint.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WoodwindWaypoint {
    /// Timeline beat position in quarter note beats.
    pub beat: f64,
    /// Blowing mouth pressure in kPa [0.05 ..= 4.00].
    pub blowing_pressure_kpa: f32,
    /// Embouchure distance in mm [1.5 ..= 15.0].
    pub jet_distance_mm: f32,
    /// Embouchure jet angle in radians [-0.5 ..= 0.5].
    pub embouchure_angle_rad: f32,
    /// 6-tonehole fingering bitmask (bits 0..5: 1 = closed, 0 = open).
    pub fingering_mask: u8,
    /// Woodwind articulation technique.
    pub articulation: WoodwindArticulation,
    /// Breath vibrato depth in semitones [0.0 ..= 2.0].
    pub vibrato_depth: f32,
    /// Breath vibrato rate in Hz [0.5 ..= 15.0].
    pub vibrato_rate_hz: f32,
    /// Flutter tongue frequency in Hz [0.0 ..= 50.0].
    pub flutter_rate_hz: f32,
    /// Breath turbulence air noise mix fraction [0.0 ..= 0.50].
    pub breath_noise_mix: f32,
}

impl Default for WoodwindWaypoint {
    fn default() -> Self {
        let (p, d, a, n) = WoodwindArticulation::Legato.nominal_parameters();
        Self {
            beat: 0.0,
            blowing_pressure_kpa: p,
            jet_distance_mm: d,
            embouchure_angle_rad: a,
            fingering_mask: 0x3F, // All 6 holes closed
            articulation: WoodwindArticulation::Legato,
            vibrato_depth: 0.0,
            vibrato_rate_hz: 5.5,
            flutter_rate_hz: 0.0,
            breath_noise_mix: n,
        }
    }
}

/// Pre-configured algorithmic woodwind gesture trajectory patterns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WoodwindGesturePattern {
    /// Smooth continuous singing legato phrase with breathing arches and tonehole sequence progression.
    LegatoCantabile {
        phrase_length_beats: f64,
        peak_pressure_kpa: f32,
        jet_distance_mm: f32,
        fingering_sequence: Vec<u8>,
    },
    /// Crisp tongued staccato repeated rhythm.
    StaccatoTongued {
        tonguing_rate_per_beat: f32,
        pressure_kpa: f32,
        attack_duration_fraction: f32,
        fingering_mask: u8,
    },
    /// Rapid double-tonguing with optional flutter tongue modulation.
    DoubleTongueFlutter {
        alternation_rate_hz: f32,
        flutter_rate_hz: f32,
        peak_pressure_kpa: f32,
        fingering_mask: u8,
    },
    /// Dynamic overblowing jump across upper harmonic registers.
    OverblowOctaveJump {
        jump_beat: f64,
        base_pressure_kpa: f32,
        overblow_pressure_kpa: f32,
        base_fingering: u8,
    },
    /// Microtonal fingering glissando with continuous embouchure angle sweep.
    MicrotonalGlissando {
        gliss_length_beats: f64,
        start_mask: u8,
        end_mask: u8,
        angle_sweep_rad: f32,
    },
    /// Japanese Shakuhachi Meri-Kari embouchure tilt modulation (bending pitch down/up by chin movement).
    ShakuhachiMeriKari {
        cycle_length_beats: f64,
        angle_down_rad: f32,
        angle_up_rad: f32,
        pressure_kpa: f32,
        fingering_mask: u8,
    },
    /// User-defined multi-keyframe Catmull-Rom spline trajectory.
    CustomSpline,
}

impl Default for WoodwindGesturePattern {
    fn default() -> Self {
        Self::LegatoCantabile {
            phrase_length_beats: 4.0,
            peak_pressure_kpa: 1.35,
            jet_distance_mm: 7.0,
            fingering_sequence: vec![0x3F, 0x1F, 0x0F, 0x07, 0x03, 0x01, 0x00],
        }
    }
}

/// Dynamic Woodwind Fingering Articulation Timeline Engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WoodwindFingeringEngine {
    /// Active procedural gesture pattern or custom spline.
    pub pattern: WoodwindGesturePattern,
    /// Interpolation smoothing curve type.
    pub interpolation: WoodwindInterpolationCurve,
    /// User waypoints (for CustomSpline mode).
    pub waypoints: Vec<WoodwindWaypoint>,
    /// Loop length in quarter note beats (if looping is enabled).
    pub loop_length_beats: Option<f64>,
}

impl Default for WoodwindFingeringEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl WoodwindFingeringEngine {
    /// Creates a new woodwind fingering engine with default Legato Cantabile pattern.
    pub fn new() -> Self {
        Self {
            pattern: WoodwindGesturePattern::default(),
            interpolation: WoodwindInterpolationCurve::CubicCatmullRom,
            waypoints: Vec::new(),
            loop_length_beats: Some(4.0),
        }
    }

    /// Adds a keyframe waypoint to the custom trajectory.
    pub fn add_waypoint(&mut self, waypoint: WoodwindWaypoint) {
        self.waypoints.push(waypoint);
        self.waypoints.sort_by(|a, b| a.beat.partial_cmp(&b.beat).unwrap_or(std::cmp::Ordering::Equal));
    }

    /// Evaluates continuous woodwind articulation parameters at timeline beat position.
    pub fn evaluate_beat(&self, beat: f64) -> BreathBusSnapshot {
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
            WoodwindGesturePattern::LegatoCantabile {
                phrase_length_beats,
                peak_pressure_kpa,
                jet_distance_mm,
                fingering_sequence,
            } => {
                let p_len = phrase_length_beats.max(0.5);
                let progress = ((eff_beat % p_len) / p_len) as f32; // 0.0 .. 1.0

                // Breathing arch: sinusoidal pressure envelope peaking in mid-phrase
                let breath_env = (progress * PI).sin();
                let pressure = 0.40 + (*peak_pressure_kpa - 0.40) * (0.25 + 0.75 * breath_env);

                // Select tonehole fingering from sequence based on beat division
                let fingering = if !fingering_sequence.is_empty() {
                    let seq_idx = ((progress * fingering_sequence.len() as f32) as usize)
                        .min(fingering_sequence.len() - 1);
                    fingering_sequence[seq_idx]
                } else {
                    0x3F
                };

                let vib_depth = 0.20 * breath_env;

                BreathBusSnapshot {
                    articulation: WoodwindArticulation::Legato,
                    blowing_pressure_kpa: pressure,
                    jet_distance_mm: *jet_distance_mm,
                    embouchure_angle_rad: 0.0,
                    fingering_mask: fingering,
                    vibrato_depth: vib_depth,
                    vibrato_rate_hz: 5.5,
                    flutter_rate_hz: 0.0,
                    breath_noise_mix: 0.08,
                    gesture_progress: progress,
                    is_active: true,
                }
            }

            WoodwindGesturePattern::StaccatoTongued {
                tonguing_rate_per_beat,
                pressure_kpa,
                attack_duration_fraction,
                fingering_mask,
            } => {
                let rate = tonguing_rate_per_beat.max(0.5);
                let cycle_phase = ((eff_beat * rate as f64) % 1.0) as f32;
                let att_frac = attack_duration_fraction.clamp(0.10, 0.90);

                let (press, noise) = if cycle_phase < att_frac {
                    let t = cycle_phase / att_frac;
                    let p = *pressure_kpa * (t * PI).sin();
                    let n = 0.15 + 0.10 * (1.0 - t);
                    (p, n)
                } else {
                    (0.05f32, 0.02f32) // Breath stopped between notes
                };

                BreathBusSnapshot {
                    articulation: WoodwindArticulation::TonguedStaccato,
                    blowing_pressure_kpa: press,
                    jet_distance_mm: 6.0,
                    embouchure_angle_rad: 0.05,
                    fingering_mask: *fingering_mask,
                    vibrato_depth: 0.0,
                    vibrato_rate_hz: 5.5,
                    flutter_rate_hz: 0.0,
                    breath_noise_mix: noise,
                    gesture_progress: cycle_phase,
                    is_active: true,
                }
            }

            WoodwindGesturePattern::DoubleTongueFlutter {
                alternation_rate_hz,
                flutter_rate_hz,
                peak_pressure_kpa,
                fingering_mask,
            } => {
                let t_sec = eff_beat * 0.5; // ~120 BPM
                let alt_phase = (t_sec as f32 * 2.0 * PI * alternation_rate_hz.max(1.0)).sin();
                let alt_env = 0.65 + 0.35 * alt_phase.abs();
                let pressure = *peak_pressure_kpa * alt_env;

                BreathBusSnapshot {
                    articulation: WoodwindArticulation::DoubleTongue,
                    blowing_pressure_kpa: pressure,
                    jet_distance_mm: 6.5,
                    embouchure_angle_rad: 0.0,
                    fingering_mask: *fingering_mask,
                    vibrato_depth: 0.0,
                    vibrato_rate_hz: 5.5,
                    flutter_rate_hz: *flutter_rate_hz,
                    breath_noise_mix: 0.18,
                    gesture_progress: ((alt_phase + 1.0) * 0.5).clamp(0.0, 1.0),
                    is_active: true,
                }
            }

            WoodwindGesturePattern::OverblowOctaveJump {
                jump_beat,
                base_pressure_kpa,
                overblow_pressure_kpa,
                base_fingering,
            } => {
                let is_overblown = eff_beat >= *jump_beat;
                let (press, jet_dist, angle, art) = if is_overblown {
                    (*overblow_pressure_kpa, 4.5, 0.15, WoodwindArticulation::OverblowHarmonic)
                } else {
                    (*base_pressure_kpa, 7.0, 0.0, WoodwindArticulation::Legato)
                };

                BreathBusSnapshot {
                    articulation: art,
                    blowing_pressure_kpa: press,
                    jet_distance_mm: jet_dist,
                    embouchure_angle_rad: angle,
                    fingering_mask: *base_fingering,
                    vibrato_depth: 0.10,
                    vibrato_rate_hz: 6.0,
                    flutter_rate_hz: 0.0,
                    breath_noise_mix: 0.06,
                    gesture_progress: if is_overblown { 1.0 } else { 0.0 },
                    is_active: true,
                }
            }

            WoodwindGesturePattern::MicrotonalGlissando {
                gliss_length_beats,
                start_mask,
                end_mask,
                angle_sweep_rad,
            } => {
                let g_len = gliss_length_beats.max(0.5);
                let progress = ((eff_beat % g_len) / g_len) as f32; // 0.0 .. 1.0
                let angle = (progress - 0.5) * 2.0 * angle_sweep_rad;
                let fingering = if progress < 0.5 { *start_mask } else { *end_mask };

                BreathBusSnapshot {
                    articulation: WoodwindArticulation::Legato,
                    blowing_pressure_kpa: 1.30,
                    jet_distance_mm: 7.0,
                    embouchure_angle_rad: angle,
                    fingering_mask: fingering,
                    vibrato_depth: 0.05,
                    vibrato_rate_hz: 5.5,
                    flutter_rate_hz: 0.0,
                    breath_noise_mix: 0.10,
                    gesture_progress: progress,
                    is_active: true,
                }
            }

            WoodwindGesturePattern::ShakuhachiMeriKari {
                cycle_length_beats,
                angle_down_rad,
                angle_up_rad,
                pressure_kpa,
                fingering_mask,
            } => {
                let c_len = cycle_length_beats.max(0.5);
                let phase = ((eff_beat % c_len) / c_len) as f32; // 0.0 .. 1.0
                let sin_val = (phase * 2.0 * PI).sin();
                let angle = if sin_val < 0.0 {
                    sin_val * angle_down_rad.abs() // Meri (downward bend)
                } else {
                    sin_val * angle_up_rad.abs() // Kari (upward bend)
                };

                let breath_noise = 0.18 + 0.08 * (sin_val.abs());

                BreathBusSnapshot {
                    articulation: WoodwindArticulation::Legato,
                    blowing_pressure_kpa: *pressure_kpa,
                    jet_distance_mm: 10.0,
                    embouchure_angle_rad: angle,
                    fingering_mask: *fingering_mask,
                    vibrato_depth: 0.08,
                    vibrato_rate_hz: 5.0,
                    flutter_rate_hz: 0.0,
                    breath_noise_mix: breath_noise,
                    gesture_progress: phase,
                    is_active: true,
                }
            }

            WoodwindGesturePattern::CustomSpline => {
                self.evaluate_custom_spline(eff_beat)
            }
        }
    }

    /// Evaluates user-defined keyframe waypoints with cubic Catmull-Rom or Bézier interpolation.
    fn evaluate_custom_spline(&self, beat: f64) -> BreathBusSnapshot {
        if self.waypoints.is_empty() {
            return BreathBusSnapshot::default();
        }
        if self.waypoints.len() == 1 {
            let wp = &self.waypoints[0];
            return BreathBusSnapshot {
                articulation: wp.articulation,
                blowing_pressure_kpa: wp.blowing_pressure_kpa,
                jet_distance_mm: wp.jet_distance_mm,
                embouchure_angle_rad: wp.embouchure_angle_rad,
                fingering_mask: wp.fingering_mask,
                vibrato_depth: wp.vibrato_depth,
                vibrato_rate_hz: wp.vibrato_rate_hz,
                flutter_rate_hz: wp.flutter_rate_hz,
                breath_noise_mix: wp.breath_noise_mix,
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
            return BreathBusSnapshot {
                articulation: wp.articulation,
                blowing_pressure_kpa: wp.blowing_pressure_kpa,
                jet_distance_mm: wp.jet_distance_mm,
                embouchure_angle_rad: wp.embouchure_angle_rad,
                fingering_mask: wp.fingering_mask,
                vibrato_depth: wp.vibrato_depth,
                vibrato_rate_hz: wp.vibrato_rate_hz,
                flutter_rate_hz: wp.flutter_rate_hz,
                breath_noise_mix: wp.breath_noise_mix,
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

        let (press, jet_dist, angle, vib_d, vib_r, flutter, noise) = match self.interpolation {
            WoodwindInterpolationCurve::Hold => (
                wp1.blowing_pressure_kpa,
                wp1.jet_distance_mm,
                wp1.embouchure_angle_rad,
                wp1.vibrato_depth,
                wp1.vibrato_rate_hz,
                wp1.flutter_rate_hz,
                wp1.breath_noise_mix,
            ),
            WoodwindInterpolationCurve::Linear => (
                wp1.blowing_pressure_kpa + t * (wp2.blowing_pressure_kpa - wp1.blowing_pressure_kpa),
                wp1.jet_distance_mm + t * (wp2.jet_distance_mm - wp1.jet_distance_mm),
                wp1.embouchure_angle_rad + t * (wp2.embouchure_angle_rad - wp1.embouchure_angle_rad),
                wp1.vibrato_depth + t * (wp2.vibrato_depth - wp1.vibrato_depth),
                wp1.vibrato_rate_hz + t * (wp2.vibrato_rate_hz - wp1.vibrato_rate_hz),
                wp1.flutter_rate_hz + t * (wp2.flutter_rate_hz - wp1.flutter_rate_hz),
                wp1.breath_noise_mix + t * (wp2.breath_noise_mix - wp1.breath_noise_mix),
            ),
            WoodwindInterpolationCurve::CubicCatmullRom => {
                let p = Self::catmull_rom_1d(wp0.blowing_pressure_kpa, wp1.blowing_pressure_kpa, wp2.blowing_pressure_kpa, wp3.blowing_pressure_kpa, t);
                let d = Self::catmull_rom_1d(wp0.jet_distance_mm, wp1.jet_distance_mm, wp2.jet_distance_mm, wp3.jet_distance_mm, t);
                let a = Self::catmull_rom_1d(wp0.embouchure_angle_rad, wp1.embouchure_angle_rad, wp2.embouchure_angle_rad, wp3.embouchure_angle_rad, t);
                let vd = Self::catmull_rom_1d(wp0.vibrato_depth, wp1.vibrato_depth, wp2.vibrato_depth, wp3.vibrato_depth, t);
                let vr = Self::catmull_rom_1d(wp0.vibrato_rate_hz, wp1.vibrato_rate_hz, wp2.vibrato_rate_hz, wp3.vibrato_rate_hz, t);
                let fl = Self::catmull_rom_1d(wp0.flutter_rate_hz, wp1.flutter_rate_hz, wp2.flutter_rate_hz, wp3.flutter_rate_hz, t);
                let nm = Self::catmull_rom_1d(wp0.breath_noise_mix, wp1.breath_noise_mix, wp2.breath_noise_mix, wp3.breath_noise_mix, t);
                (p, d, a, vd, vr, fl, nm)
            }
            WoodwindInterpolationCurve::CubicBezier => {
                let b = t * t * (3.0 - 2.0 * t); // Smoothstep S-curve
                (
                    wp1.blowing_pressure_kpa + b * (wp2.blowing_pressure_kpa - wp1.blowing_pressure_kpa),
                    wp1.jet_distance_mm + b * (wp2.jet_distance_mm - wp1.jet_distance_mm),
                    wp1.embouchure_angle_rad + b * (wp2.embouchure_angle_rad - wp1.embouchure_angle_rad),
                    wp1.vibrato_depth + b * (wp2.vibrato_depth - wp1.vibrato_depth),
                    wp1.vibrato_rate_hz + b * (wp2.vibrato_rate_hz - wp1.vibrato_rate_hz),
                    wp1.flutter_rate_hz + b * (wp2.flutter_rate_hz - wp1.flutter_rate_hz),
                    wp1.breath_noise_mix + b * (wp2.breath_noise_mix - wp1.breath_noise_mix),
                )
            }
        };

        let fingering = if t < 0.5 { wp1.fingering_mask } else { wp2.fingering_mask };
        let art = if t < 0.5 { wp1.articulation } else { wp2.articulation };

        BreathBusSnapshot {
            articulation: art,
            blowing_pressure_kpa: press.clamp(0.05, 4.0),
            jet_distance_mm: jet_dist.clamp(1.5, 15.0),
            embouchure_angle_rad: angle.clamp(-0.5, 0.5),
            fingering_mask: fingering,
            vibrato_depth: vib_d.clamp(0.0, 2.0),
            vibrato_rate_hz: vib_r.clamp(0.5, 15.0),
            flutter_rate_hz: flutter.clamp(0.0, 50.0),
            breath_noise_mix: noise.clamp(0.0, 0.50),
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

    /// Evaluates timeline at beat and dispatches the snapshot directly into a `BreathBus`.
    pub fn dispatch_to_breath_bus(&self, beat: f64, bus: &BreathBus) {
        let snap = self.evaluate_beat(beat);
        bus.apply_snapshot(&snap);
    }

    /// Dispatches continuous evaluated parameters into `ParamBus`.
    pub fn dispatch_to_param_bus(&self, beat: f64, bus: &ParamBus, base_param_id: ParamId) {
        let snap = self.evaluate_beat(beat);
        bus.set(ParamId(base_param_id.0), snap.articulation as u32 as f32);
        bus.set(ParamId(base_param_id.0 + 1), snap.blowing_pressure_kpa);
        bus.set(ParamId(base_param_id.0 + 2), snap.jet_distance_mm);
        bus.set(ParamId(base_param_id.0 + 3), snap.embouchure_angle_rad);
        bus.set(ParamId(base_param_id.0 + 4), snap.fingering_mask as f32);
        bus.set(ParamId(base_param_id.0 + 5), snap.vibrato_depth);
        bus.set(ParamId(base_param_id.0 + 6), snap.vibrato_rate_hz);
        bus.set(ParamId(base_param_id.0 + 7), snap.flutter_rate_hz);
        bus.set(ParamId(base_param_id.0 + 8), snap.breath_noise_mix);
        bus.set(ParamId(base_param_id.0 + 9), snap.gesture_progress);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::allocator::AllocGuard;

    #[test]
    fn test_woodwind_fingering_legato_evaluation() {
        let engine = WoodwindFingeringEngine::new();
        let snap = engine.evaluate_beat(1.0);
        assert_eq!(snap.articulation, WoodwindArticulation::Legato);
        assert!(snap.blowing_pressure_kpa > 0.50);
        assert!(snap.is_active);
    }

    #[test]
    fn test_woodwind_fingering_custom_spline_catmull_rom() {
        let mut engine = WoodwindFingeringEngine {
            pattern: WoodwindGesturePattern::CustomSpline,
            interpolation: WoodwindInterpolationCurve::CubicCatmullRom,
            waypoints: Vec::new(),
            loop_length_beats: Some(4.0),
        };

        engine.add_waypoint(WoodwindWaypoint {
            beat: 0.0,
            blowing_pressure_kpa: 0.80,
            jet_distance_mm: 8.0,
            embouchure_angle_rad: -0.10,
            fingering_mask: 0x3F,
            articulation: WoodwindArticulation::Legato,
            ..Default::default()
        });

        engine.add_waypoint(WoodwindWaypoint {
            beat: 2.0,
            blowing_pressure_kpa: 2.50,
            jet_distance_mm: 5.0,
            embouchure_angle_rad: 0.15,
            fingering_mask: 0x07,
            articulation: WoodwindArticulation::OverblowHarmonic,
            ..Default::default()
        });

        engine.add_waypoint(WoodwindWaypoint {
            beat: 4.0,
            blowing_pressure_kpa: 1.20,
            jet_distance_mm: 7.0,
            embouchure_angle_rad: 0.0,
            fingering_mask: 0x00,
            articulation: WoodwindArticulation::Legato,
            ..Default::default()
        });

        let mid = engine.evaluate_beat(1.0);
        assert!(mid.blowing_pressure_kpa > 0.80 && mid.blowing_pressure_kpa < 2.50);
        assert!(mid.jet_distance_mm < 8.0 && mid.jet_distance_mm > 5.0);
    }

    #[test]
    fn test_woodwind_fingering_dispatch_zero_allocation() {
        let engine = WoodwindFingeringEngine::new();
        let bus = BreathBus::new();
        let _guard = AllocGuard::new();

        for i in 0..100 {
            let beat = i as f64 * 0.05;
            engine.dispatch_to_breath_bus(beat, &bus);
        }
    }

    #[test]
    fn test_woodwind_fingering_toml_roundtrip() {
        let mut engine = WoodwindFingeringEngine::new();
        engine.pattern = WoodwindGesturePattern::ShakuhachiMeriKari {
            cycle_length_beats: 4.0,
            angle_down_rad: -0.25,
            angle_up_rad: 0.15,
            pressure_kpa: 1.60,
            fingering_mask: 0x1F,
        };
        engine.add_waypoint(WoodwindWaypoint::default());

        let toml_str = toml::to_string_pretty(&engine).expect("Failed to serialize woodwind fingering engine");
        let decoded: WoodwindFingeringEngine = toml::from_str(&toml_str).expect("Failed to deserialize woodwind fingering engine");

        assert_eq!(engine.waypoints.len(), decoded.waypoints.len());
        assert_eq!(engine.interpolation, decoded.interpolation);
    }
}
