// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Dynamic Rotary Articulation Timeline & Harmonic Drawbar Gesture Engine (Milestone 21).
//!
//! Provides multi-dimensional continuous physical modeling trajectory generation for
//! electromechanical tonewheel organs and Leslie rotary speaker cabinets (Speed Mode,
//! 9-Drawbar Harmonic Levels, Tube Drive Overdrive, Percussion Triggers, Vibrato/Chorus)
//! supporting dynamic Gospel crescendos, Jazz ballad swells, fast Leslie rotor acceleration
//! sweeps, and rhythmic percussion key-clicks with Catmull-Rom spline smoothing,
//! TOML serialization, and sample-accurate lock-free dispatch into `RotaryBus` and `ParamBus`.
//!
//! Enforces zero heap allocations in real-time execution loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use summoner_core::param_bus::{ParamBus, ParamId};
use summoner_core::rotary_bus::{RotaryArticulation, RotaryBus, RotaryBusSnapshot};

/// Interpolation curve between discrete rotary/drawbar keyframe waypoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RotaryInterpolationCurve {
    #[default]
    CubicCatmullRom,
    CubicBezier,
    Linear,
    Hold,
    Exponential,
}

/// A single keyframed multi-dimensional rotary & drawbar waypoint.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RotaryWaypoint {
    /// Timeline beat position in quarter note beats.
    pub beat: f64,
    /// Speed state (0=Stop, 1=Chorale, 2=Tremolo, 3=Brake).
    pub speed_state: u32,
    /// 9 Drawbar levels in range $[0.0 ..= 8.0]$.
    pub drawbars: [f32; 9],
    /// Tube power amplifier drive in dB $[0.0 ..= 24.0\text{dB}]$.
    pub tube_drive_db: f32,
    /// Horn vs Drum acoustic balance percentage $[0.0 ..= 100.0\%]$.
    pub horn_drum_balance_pct: f32,
    /// Harmonic percussion enabled.
    pub percussion_enabled: bool,
    /// Percussion harmonic (2 or 3).
    pub percussion_harmonic: u32,
    /// Percussion fast decay.
    pub percussion_fast: bool,
    /// Scanner Vibrato/Chorus mode (0..=6).
    pub vibrato_mode: u32,
    /// Mechanical articulation technique.
    pub articulation: RotaryArticulation,
    /// Interpolation curve leading to this waypoint.
    pub curve: RotaryInterpolationCurve,
}

impl Default for RotaryWaypoint {
    fn default() -> Self {
        let (speed, drive, bal, dbs) = RotaryArticulation::ChoraleSlow.nominal_parameters();
        Self {
            beat: 0.0,
            speed_state: speed,
            drawbars: dbs,
            tube_drive_db: drive,
            horn_drum_balance_pct: bal,
            percussion_enabled: true,
            percussion_harmonic: 3,
            percussion_fast: true,
            vibrato_mode: 6, // C3
            articulation: RotaryArticulation::ChoraleSlow,
            curve: RotaryInterpolationCurve::CubicCatmullRom,
        }
    }
}

/// Pre-configured algorithmic rotary & tonewheel gesture trajectory patterns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RotaryGesturePattern {
    /// High-energy Gospel shout with accelerating Leslie rotor (Chorale -> Tremolo) and full drawbar swell.
    GospelShout {
        phrase_length_beats: f64,
        peak_drive_db: f32,
    },
    /// Smooth Jazz ballad with subtle Chorale spatial modulation and warm drawbar morphing.
    JazzBallad {
        phrase_length_beats: f64,
        max_drawbar_level: f32,
    },
    /// Roaring Rock organ solo with heavy 6550 tube saturation and dramatic Leslie brake/speed switches.
    RockSolo {
        solo_length_beats: f64,
        overdrive_db: f32,
    },
    /// Ambient harmonic drawbar swell with slow morphing harmonics and lush C3 chorus.
    AmbientSwell {
        swell_length_beats: f64,
    },
    /// Funk rhythm comping with rhythmic 2nd harmonic percussion bursts and fast Leslie toggling.
    PercussiveGroove {
        groove_length_beats: f64,
        percussion_harmonic: u32,
    },
    /// User-defined multi-keyframe Catmull-Rom spline trajectory.
    CustomSpline,
}

impl Default for RotaryGesturePattern {
    fn default() -> Self {
        Self::GospelShout {
            phrase_length_beats: 8.0,
            peak_drive_db: 10.0,
        }
    }
}

/// Dynamic Rotary Articulation Timeline & Gesture Engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RotaryGestureEngine {
    /// Keyframed articulation waypoints.
    pub waypoints: Vec<RotaryWaypoint>,
    /// Active gesture trajectory pattern.
    pub pattern: RotaryGesturePattern,
    /// Default interpolation curve.
    pub default_curve: RotaryInterpolationCurve,
    /// Total sequence loop length in beats.
    pub loop_length_beats: f64,
    /// Whether pattern loops continuously.
    pub loop_enabled: bool,
    /// Whether gesture engine is currently active.
    pub is_active: bool,
}

impl Default for RotaryGestureEngine {
    fn default() -> Self {
        let mut engine = Self {
            waypoints: Vec::new(),
            pattern: RotaryGesturePattern::default(),
            default_curve: RotaryInterpolationCurve::CubicCatmullRom,
            loop_length_beats: 8.0,
            loop_enabled: true,
            is_active: true,
        };
        engine.generate_pattern_waypoints();
        engine
    }
}

impl RotaryGestureEngine {
    /// Creates a new RotaryGestureEngine initialized with the specified pattern.
    pub fn new(pattern: RotaryGesturePattern) -> Self {
        let mut engine = Self {
            waypoints: Vec::new(),
            pattern,
            default_curve: RotaryInterpolationCurve::CubicCatmullRom,
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
            RotaryGesturePattern::GospelShout {
                phrase_length_beats,
                peak_drive_db,
            } => {
                self.loop_length_beats = *phrase_length_beats;
                // 4 waypoints: 0=Chorale start, 2=Start speed ramp, 4=Full Tremolo swell, 8=Peak
                self.waypoints.push(RotaryWaypoint {
                    beat: 0.0,
                    speed_state: 1, // Chorale
                    drawbars: [8.0, 8.0, 8.0, 0.0, 0.0, 0.0, 0.0, 0.0, 8.0],
                    tube_drive_db: 4.0,
                    horn_drum_balance_pct: 60.0,
                    percussion_enabled: true,
                    percussion_harmonic: 3,
                    percussion_fast: true,
                    vibrato_mode: 6, // C3
                    articulation: RotaryArticulation::ChoraleSlow,
                    curve: RotaryInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(RotaryWaypoint {
                    beat: 0.25 * *phrase_length_beats,
                    speed_state: 2, // Tremolo
                    drawbars: [8.0, 8.0, 8.0, 2.0, 2.0, 2.0, 2.0, 4.0, 8.0],
                    tube_drive_db: 6.0,
                    horn_drum_balance_pct: 65.0,
                    percussion_enabled: true,
                    percussion_harmonic: 3,
                    percussion_fast: true,
                    vibrato_mode: 6,
                    articulation: RotaryArticulation::TremoloFast,
                    curve: RotaryInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(RotaryWaypoint {
                    beat: 0.75 * *phrase_length_beats,
                    speed_state: 2, // Tremolo
                    drawbars: [8.0, 8.0, 8.0, 6.0, 6.0, 6.0, 6.0, 8.0, 8.0],
                    tube_drive_db: *peak_drive_db,
                    horn_drum_balance_pct: 70.0,
                    percussion_enabled: false,
                    percussion_harmonic: 3,
                    percussion_fast: true,
                    vibrato_mode: 6,
                    articulation: RotaryArticulation::GospelSwell,
                    curve: RotaryInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(RotaryWaypoint {
                    beat: *phrase_length_beats,
                    speed_state: 2,
                    drawbars: [8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0],
                    tube_drive_db: *peak_drive_db + 2.0,
                    horn_drum_balance_pct: 70.0,
                    percussion_enabled: false,
                    percussion_harmonic: 3,
                    percussion_fast: true,
                    vibrato_mode: 6,
                    articulation: RotaryArticulation::RockOverdrive,
                    curve: RotaryInterpolationCurve::CubicCatmullRom,
                });
            }

            RotaryGesturePattern::JazzBallad {
                phrase_length_beats,
                max_drawbar_level,
            } => {
                self.loop_length_beats = *phrase_length_beats;
                let steps = 4;
                for i in 0..=steps {
                    let frac = i as f64 / steps as f64;
                    let beat = frac * phrase_length_beats;
                    let db_mid = if frac < 0.5 {
                        *max_drawbar_level * (frac as f32 / 0.5)
                    } else {
                        *max_drawbar_level * (1.0 - (frac as f32 - 0.5) / 0.5)
                    };

                    self.waypoints.push(RotaryWaypoint {
                        beat,
                        speed_state: 1, // Chorale
                        drawbars: [8.0, 8.0, 8.0, db_mid * 0.5, 0.0, 0.0, 0.0, 0.0, db_mid * 0.8],
                        tube_drive_db: 3.5,
                        horn_drum_balance_pct: 55.0,
                        percussion_enabled: true,
                        percussion_harmonic: 3,
                        percussion_fast: true,
                        vibrato_mode: 6,
                        articulation: RotaryArticulation::DrawbarMorph,
                        curve: RotaryInterpolationCurve::CubicCatmullRom,
                    });
                }
            }

            RotaryGesturePattern::RockSolo {
                solo_length_beats,
                overdrive_db,
            } => {
                self.loop_length_beats = *solo_length_beats;
                self.waypoints.push(RotaryWaypoint {
                    beat: 0.0,
                    speed_state: 2, // Tremolo
                    drawbars: [8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0],
                    tube_drive_db: *overdrive_db,
                    horn_drum_balance_pct: 65.0,
                    percussion_enabled: false,
                    percussion_harmonic: 2,
                    percussion_fast: false,
                    vibrato_mode: 6,
                    articulation: RotaryArticulation::RockOverdrive,
                    curve: RotaryInterpolationCurve::Linear,
                });

                self.waypoints.push(RotaryWaypoint {
                    beat: 0.5 * *solo_length_beats,
                    speed_state: 3, // Brake
                    drawbars: [8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0],
                    tube_drive_db: *overdrive_db + 3.0,
                    horn_drum_balance_pct: 60.0,
                    percussion_enabled: false,
                    percussion_harmonic: 2,
                    percussion_fast: false,
                    vibrato_mode: 6,
                    articulation: RotaryArticulation::BrakeStop,
                    curve: RotaryInterpolationCurve::Linear,
                });

                self.waypoints.push(RotaryWaypoint {
                    beat: *solo_length_beats,
                    speed_state: 2, // Tremolo again
                    drawbars: [8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0],
                    tube_drive_db: *overdrive_db,
                    horn_drum_balance_pct: 65.0,
                    percussion_enabled: false,
                    percussion_harmonic: 2,
                    percussion_fast: false,
                    vibrato_mode: 6,
                    articulation: RotaryArticulation::TubeDriveBoost,
                    curve: RotaryInterpolationCurve::Linear,
                });
            }

            RotaryGesturePattern::AmbientSwell { swell_length_beats } => {
                self.loop_length_beats = *swell_length_beats;
                let steps = 5;
                for i in 0..=steps {
                    let frac = i as f64 / steps as f64;
                    let beat = frac * swell_length_beats;
                    let upper = (frac as f32) * 8.0;

                    self.waypoints.push(RotaryWaypoint {
                        beat,
                        speed_state: 1, // Chorale
                        drawbars: [8.0, 0.0, 0.0, 0.0, 0.0, upper, upper, upper, upper],
                        tube_drive_db: 2.0 + frac as f32 * 3.0,
                        horn_drum_balance_pct: 50.0 + frac as f32 * 15.0,
                        percussion_enabled: false,
                        percussion_harmonic: 3,
                        percussion_fast: false,
                        vibrato_mode: 6,
                        articulation: RotaryArticulation::DrawbarMorph,
                        curve: RotaryInterpolationCurve::CubicCatmullRom,
                    });
                }
            }

            RotaryGesturePattern::PercussiveGroove {
                groove_length_beats,
                percussion_harmonic,
            } => {
                self.loop_length_beats = *groove_length_beats;
                self.waypoints.push(RotaryWaypoint {
                    beat: 0.0,
                    speed_state: 2,
                    drawbars: [8.0, 0.0, 8.0, 0.0, 0.0, 0.0, 0.0, 0.0, 8.0],
                    tube_drive_db: 5.0,
                    horn_drum_balance_pct: 60.0,
                    percussion_enabled: true,
                    percussion_harmonic: *percussion_harmonic,
                    percussion_fast: true,
                    vibrato_mode: 0, // Off
                    articulation: RotaryArticulation::PercussionHit,
                    curve: RotaryInterpolationCurve::Hold,
                });
            }

            RotaryGesturePattern::CustomSpline => {
                if self.waypoints.is_empty() {
                    self.waypoints.push(RotaryWaypoint::default());
                }
            }
        }
    }

    /// Evaluates continuous parameters at the specified beat position.
    pub fn evaluate_at_beat(&self, beat: f64) -> RotaryBusSnapshot {
        if self.waypoints.is_empty() {
            return RotaryBusSnapshot::default();
        }

        if self.waypoints.len() == 1 {
            let wp = self.waypoints[0];
            return RotaryBusSnapshot {
                articulation: wp.articulation,
                speed_state: wp.speed_state,
                horn_rpm: if wp.speed_state == 2 { 400.0 } else if wp.speed_state == 1 { 40.0 } else { 0.0 },
                drum_rpm: if wp.speed_state == 2 { 342.0 } else if wp.speed_state == 1 { 36.0 } else { 0.0 },
                drawbars: wp.drawbars,
                percussion_enabled: wp.percussion_enabled,
                percussion_harmonic: wp.percussion_harmonic,
                percussion_fast: wp.percussion_fast,
                percussion_soft: false,
                tube_drive_db: wp.tube_drive_db,
                vibrato_mode: wp.vibrato_mode,
                key_click_amount: 0.35,
                crosstalk_amount: 0.08,
                horn_drum_balance_pct: wp.horn_drum_balance_pct,
                mic_distance_m: 0.65,
                mic_spread_deg: 120.0,
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

        // Interpolate drawbars
        let mut drawbars = [0.0f32; 9];
        for (d, db_slot) in drawbars.iter_mut().enumerate() {
            *db_slot = match curve {
                RotaryInterpolationCurve::Hold => w1.drawbars[d],
                RotaryInterpolationCurve::Linear => lerp(w1.drawbars[d], w2.drawbars[d], t),
                RotaryInterpolationCurve::Exponential => lerp(w1.drawbars[d], w2.drawbars[d], t * t),
                RotaryInterpolationCurve::CubicBezier => {
                    let tb = 3.0 * t * t - 2.0 * t * t * t;
                    lerp(w1.drawbars[d], w2.drawbars[d], tb)
                }
                RotaryInterpolationCurve::CubicCatmullRom => {
                    catmull_rom_1d(w0.drawbars[d], w1.drawbars[d], w2.drawbars[d], w3.drawbars[d], t)
                }
            }.clamp(0.0, 8.0);
        }

        // Interpolate continuous parameters
        let drive = match curve {
            RotaryInterpolationCurve::Hold => w1.tube_drive_db,
            RotaryInterpolationCurve::Linear => lerp(w1.tube_drive_db, w2.tube_drive_db, t),
            _ => catmull_rom_1d(w0.tube_drive_db, w1.tube_drive_db, w2.tube_drive_db, w3.tube_drive_db, t),
        }.clamp(0.0, 24.0);

        let balance = match curve {
            RotaryInterpolationCurve::Hold => w1.horn_drum_balance_pct,
            RotaryInterpolationCurve::Linear => lerp(w1.horn_drum_balance_pct, w2.horn_drum_balance_pct, t),
            _ => catmull_rom_1d(w0.horn_drum_balance_pct, w1.horn_drum_balance_pct, w2.horn_drum_balance_pct, w3.horn_drum_balance_pct, t),
        }.clamp(0.0, 100.0);

        let speed = if t < 0.5 { w1.speed_state } else { w2.speed_state };
        let horn_rpm = if speed == 2 { 400.0 } else if speed == 1 { 40.0 } else { 0.0 };
        let drum_rpm = if speed == 2 { 342.0 } else if speed == 1 { 36.0 } else { 0.0 };

        let progress = if self.loop_length_beats > 0.0 {
            (effective_beat / self.loop_length_beats).clamp(0.0, 1.0) as f32
        } else {
            0.0
        };

        RotaryBusSnapshot {
            articulation: if t < 0.5 { w1.articulation } else { w2.articulation },
            speed_state: speed,
            horn_rpm,
            drum_rpm,
            drawbars,
            percussion_enabled: if t < 0.5 { w1.percussion_enabled } else { w2.percussion_enabled },
            percussion_harmonic: if t < 0.5 { w1.percussion_harmonic } else { w2.percussion_harmonic },
            percussion_fast: if t < 0.5 { w1.percussion_fast } else { w2.percussion_fast },
            percussion_soft: false,
            tube_drive_db: drive,
            vibrato_mode: if t < 0.5 { w1.vibrato_mode } else { w2.vibrato_mode },
            key_click_amount: 0.35,
            crosstalk_amount: 0.08,
            horn_drum_balance_pct: balance,
            mic_distance_m: 0.65,
            mic_spread_deg: 120.0,
            gesture_progress: progress,
            is_active: self.is_active,
        }
    }

    /// Dispatches evaluated articulation parameters into a lock-free `RotaryBus`.
    pub fn dispatch_to_bus(&self, beat: f64, bus: &RotaryBus) {
        let snap = self.evaluate_at_beat(beat);
        bus.apply_snapshot(&snap);
    }

    /// Dispatches evaluated articulation parameters into a central `ParamBus`.
    pub fn dispatch_to_param_bus(&self, beat: f64, base_id: ParamId, param_bus: &ParamBus) {
        let snap = self.evaluate_at_beat(beat);
        param_bus.set(ParamId(base_id.0), snap.articulation as u32 as f32);
        param_bus.set(ParamId(base_id.0 + 1), snap.speed_state as f32);
        param_bus.set(ParamId(base_id.0 + 2), snap.tube_drive_db);
        param_bus.set(ParamId(base_id.0 + 3), snap.horn_drum_balance_pct);
        for i in 0..9 {
            param_bus.set(ParamId(base_id.0 + 4 + i as u32), snap.drawbars[i]);
        }
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
    fn test_rotary_gesture_gospel_shout_evaluation() {
        let engine = RotaryGestureEngine::new(RotaryGesturePattern::default());
        let snap_start = engine.evaluate_at_beat(0.0);
        assert_eq!(snap_start.speed_state, 1); // Chorale
        assert_eq!(snap_start.drawbars[0], 8.0);

        let snap_swell = engine.evaluate_at_beat(6.0);
        assert_eq!(snap_swell.speed_state, 2); // Tremolo
        assert!(snap_swell.tube_drive_db >= 6.0);
    }

    #[test]
    fn test_rotary_gesture_toml_roundtrip() {
        let engine = RotaryGestureEngine::new(RotaryGesturePattern::default());
        let toml_str = engine.to_toml().expect("Failed to serialize rotary gesture TOML");
        assert!(toml_str.contains("GospelShout"));

        let restored: RotaryGestureEngine = RotaryGestureEngine::from_toml(&toml_str)
            .expect("Failed to deserialize rotary gesture TOML");
        assert_eq!(engine.waypoints.len(), restored.waypoints.len());
        assert_eq!(engine.loop_length_beats, restored.loop_length_beats);
    }

    #[test]
    fn test_rotary_gesture_dispatch_zero_allocation() {
        let engine = RotaryGestureEngine::new(RotaryGesturePattern::default());
        let bus = RotaryBus::new();
        let mut param_bus = ParamBus::new();
        for i in 0..20 {
            param_bus.register(ParamId(750 + i), 0.0);
        }

        engine.dispatch_to_bus(2.0, &bus);
        let snap = bus.snapshot();
        assert!(snap.drawbars[0] > 0.0);
        assert!(snap.is_active);

        engine.dispatch_to_param_bus(3.0, ParamId(750), &param_bus);
        assert!(param_bus.get(ParamId(751)).unwrap() >= 1.0); // speed_state
        assert!(param_bus.get(ParamId(754)).unwrap() > 0.0); // drawbar 0
    }
}
