// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Dynamic Clavinet Articulation Timeline & Gesture Engine (Milestone 23).
//!
//! Provides multi-dimensional continuous physical modeling trajectory generation for
//! electromechanical Clavinet synthesizers supporting dynamic Superstition funk grooves,
//! out-of-phase magnetic cancellation solos, and dynamic resonant auto-wah filter sweeps
//! with Catmull-Rom spline smoothing, TOML serialization, and sample-accurate lock-free
//! parameter dispatch into `ClavinetBus` and `ParamBus`.
//!
//! Enforces zero heap allocations in real-time execution loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use summoner_core::clavinet_bus::{
    ClavinetArticulation, ClavinetBus, ClavinetBusSnapshot, ClavinetPickupMode,
};
use summoner_core::param_bus::{ParamBus, ParamId};

/// Interpolation curve between discrete Clavinet keyframe waypoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ClavinetInterpolationCurve {
    #[default]
    CubicCatmullRom,
    CubicBezier,
    Linear,
    Hold,
    Exponential,
}

/// A single keyframed multi-dimensional Clavinet articulation waypoint.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ClavinetWaypoint {
    /// Timeline beat position in quarter note beats.
    pub beat: f64,
    /// Dual electromagnetic pickup mode.
    pub pickup_mode: ClavinetPickupMode,
    /// Pickup blend $[0.0 ..= 1.0]$.
    pub pickup_blend: f32,
    /// Pickup phase cancellation angle $[0.0 ..= 180.0^\circ]$.
    pub pickup_phase_deg: f32,
    /// Rubber anvil hardness $[0.0 ..= 1.0]$.
    pub anvil_hardness: f32,
    /// Yarn wool damping amount $[0.0 ..= 1.0]$.
    pub yarn_damping: f32,
    /// 4-way tone switch bitmask (Brilliant, Treble, Medium, Soft).
    pub tone_switches_mask: u32,
    /// Dynamic state-variable auto-wah enabled.
    pub auto_wah_enabled: bool,
    /// Auto-wah envelope sensitivity $[0.0 ..= 1.0]$.
    pub auto_wah_sensitivity: f32,
    /// Auto-wah base cutoff frequency in Hz $[50.0 ..= 4000.0\text{Hz}]$.
    pub auto_wah_freq_hz: f32,
    /// Auto-wah resonance $Q$ factor $[0.5 ..= 25.0]$.
    pub auto_wah_resonance_q: f32,
    /// Auto-wah dry/wet blend $[0.0 ..= 1.0]$.
    pub auto_wah_mix: f32,
    /// Mechanical articulation technique.
    pub articulation: ClavinetArticulation,
    /// Interpolation curve leading to this waypoint.
    pub curve: ClavinetInterpolationCurve,
}

impl Default for ClavinetWaypoint {
    fn default() -> Self {
        let (mode, blend, phase, hardness, damping, br, tr, md, sf, wah_en, wah_sens, wah_freq, wah_res, wah_mix) =
            ClavinetArticulation::StevieFunk.nominal_parameters();

        let mut mask = 0u32;
        if br { mask |= 1; }
        if tr { mask |= 2; }
        if md { mask |= 4; }
        if sf { mask |= 8; }

        Self {
            beat: 0.0,
            pickup_mode: mode,
            pickup_blend: blend,
            pickup_phase_deg: phase,
            anvil_hardness: hardness,
            yarn_damping: damping,
            tone_switches_mask: mask,
            auto_wah_enabled: wah_en,
            auto_wah_sensitivity: wah_sens,
            auto_wah_freq_hz: wah_freq,
            auto_wah_resonance_q: wah_res,
            auto_wah_mix: wah_mix,
            articulation: ClavinetArticulation::StevieFunk,
            curve: ClavinetInterpolationCurve::CubicCatmullRom,
        }
    }
}

/// Pre-configured algorithmic Clavinet gesture trajectory patterns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClavinetGesturePattern {
    /// Classic Stevie 1970s Funk groove with aggressive anvil strikes and punchy tone filters.
    SuperstitionFunk {
        phrase_length_beats: f64,
        max_anvil_hardness: f32,
    },
    /// Out-of-phase pickup cancellation solo with sweeping hollow "quack" filter resonance.
    QuackOutPhaseSolo {
        phrase_length_beats: f64,
        pickup_phase_deg: f32,
    },
    /// Dynamic Auto-Wah funk sweep with envelope follower tracking and resonant bandpass filter.
    DynamicWahSweep {
        sweep_length_beats: f64,
        peak_freq_hz: f32,
    },
    /// Tight staccato chamber funk chop with heavy yarn wool damping and crisp release thuds.
    ChamberStaccatoGroove {
        groove_length_beats: f64,
        yarn_damping: f32,
    },
    /// User-defined multi-keyframe Catmull-Rom spline trajectory.
    CustomSpline,
}

impl Default for ClavinetGesturePattern {
    fn default() -> Self {
        Self::SuperstitionFunk {
            phrase_length_beats: 8.0,
            max_anvil_hardness: 0.95,
        }
    }
}

/// Dynamic Clavinet Articulation Timeline & Gesture Engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClavinetGestureEngine {
    /// Keyframed articulation waypoints.
    pub waypoints: Vec<ClavinetWaypoint>,
    /// Active gesture trajectory pattern.
    pub pattern: ClavinetGesturePattern,
    /// Default interpolation curve.
    pub default_curve: ClavinetInterpolationCurve,
    /// Total sequence loop length in beats.
    pub loop_length_beats: f64,
    /// Whether pattern loops continuously.
    pub loop_enabled: bool,
    /// Whether gesture engine is currently active.
    pub is_active: bool,
}

impl Default for ClavinetGestureEngine {
    fn default() -> Self {
        let mut engine = Self {
            waypoints: Vec::new(),
            pattern: ClavinetGesturePattern::default(),
            default_curve: ClavinetInterpolationCurve::CubicCatmullRom,
            loop_length_beats: 8.0,
            loop_enabled: true,
            is_active: true,
        };
        engine.generate_pattern_waypoints();
        engine
    }
}

impl ClavinetGestureEngine {
    /// Creates a new ClavinetGestureEngine initialized with the specified pattern.
    pub fn new(pattern: ClavinetGesturePattern) -> Self {
        let mut engine = Self {
            waypoints: Vec::new(),
            pattern,
            default_curve: ClavinetInterpolationCurve::CubicCatmullRom,
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
            ClavinetGesturePattern::SuperstitionFunk {
                phrase_length_beats,
                max_anvil_hardness,
            } => {
                self.loop_length_beats = *phrase_length_beats;
                self.waypoints.push(ClavinetWaypoint {
                    beat: 0.0,
                    pickup_mode: ClavinetPickupMode::ParallelInPhase,
                    pickup_blend: 0.50,
                    pickup_phase_deg: 0.0,
                    anvil_hardness: 0.75,
                    yarn_damping: 0.70,
                    tone_switches_mask: 1 | 2 | 4, // Brilliant, Treble, Medium
                    auto_wah_enabled: false,
                    auto_wah_sensitivity: 0.70,
                    auto_wah_freq_hz: 350.0,
                    auto_wah_resonance_q: 6.0,
                    auto_wah_mix: 0.85,
                    articulation: ClavinetArticulation::StevieFunk,
                    curve: ClavinetInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(ClavinetWaypoint {
                    beat: 0.5 * *phrase_length_beats,
                    pickup_mode: ClavinetPickupMode::ParallelInPhase,
                    pickup_blend: 0.65,
                    pickup_phase_deg: 0.0,
                    anvil_hardness: *max_anvil_hardness,
                    yarn_damping: 0.85,
                    tone_switches_mask: 1 | 2 | 4,
                    auto_wah_enabled: false,
                    auto_wah_sensitivity: 0.70,
                    auto_wah_freq_hz: 350.0,
                    auto_wah_resonance_q: 6.0,
                    auto_wah_mix: 0.85,
                    articulation: ClavinetArticulation::StevieFunk,
                    curve: ClavinetInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(ClavinetWaypoint {
                    beat: *phrase_length_beats,
                    pickup_mode: ClavinetPickupMode::ParallelInPhase,
                    pickup_blend: 0.50,
                    pickup_phase_deg: 0.0,
                    anvil_hardness: 0.75,
                    yarn_damping: 0.70,
                    tone_switches_mask: 1 | 2 | 4,
                    auto_wah_enabled: false,
                    auto_wah_sensitivity: 0.70,
                    auto_wah_freq_hz: 350.0,
                    auto_wah_resonance_q: 6.0,
                    auto_wah_mix: 0.85,
                    articulation: ClavinetArticulation::StevieFunk,
                    curve: ClavinetInterpolationCurve::CubicCatmullRom,
                });
            }

            ClavinetGesturePattern::QuackOutPhaseSolo {
                phrase_length_beats,
                pickup_phase_deg,
            } => {
                self.loop_length_beats = *phrase_length_beats;
                self.waypoints.push(ClavinetWaypoint {
                    beat: 0.0,
                    pickup_mode: ClavinetPickupMode::ParallelOutOfPhase,
                    pickup_blend: 0.50,
                    pickup_phase_deg: *pickup_phase_deg,
                    anvil_hardness: 0.85,
                    yarn_damping: 0.80,
                    tone_switches_mask: 1 | 2, // Brilliant, Treble
                    auto_wah_enabled: false,
                    auto_wah_sensitivity: 0.80,
                    auto_wah_freq_hz: 400.0,
                    auto_wah_resonance_q: 7.5,
                    auto_wah_mix: 0.90,
                    articulation: ClavinetArticulation::OutPhaseQuack,
                    curve: ClavinetInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(ClavinetWaypoint {
                    beat: *phrase_length_beats,
                    pickup_mode: ClavinetPickupMode::ParallelOutOfPhase,
                    pickup_blend: 0.50,
                    pickup_phase_deg: *pickup_phase_deg,
                    anvil_hardness: 0.85,
                    yarn_damping: 0.80,
                    tone_switches_mask: 1 | 2,
                    auto_wah_enabled: false,
                    auto_wah_sensitivity: 0.80,
                    auto_wah_freq_hz: 400.0,
                    auto_wah_resonance_q: 7.5,
                    auto_wah_mix: 0.90,
                    articulation: ClavinetArticulation::OutPhaseQuack,
                    curve: ClavinetInterpolationCurve::CubicCatmullRom,
                });
            }

            ClavinetGesturePattern::DynamicWahSweep {
                sweep_length_beats,
                peak_freq_hz,
            } => {
                self.loop_length_beats = *sweep_length_beats;
                let steps = 4;
                for i in 0..=steps {
                    let frac = i as f64 / steps as f64;
                    let beat = frac * sweep_length_beats;
                    let freq = 300.0 + (*peak_freq_hz - 300.0) * frac as f32;

                    self.waypoints.push(ClavinetWaypoint {
                        beat,
                        pickup_mode: ClavinetPickupMode::ParallelOutOfPhase,
                        pickup_blend: 0.50,
                        pickup_phase_deg: 180.0,
                        anvil_hardness: 0.80 + 0.15 * frac as f32,
                        yarn_damping: 0.75,
                        tone_switches_mask: 1 | 2,
                        auto_wah_enabled: true,
                        auto_wah_sensitivity: 0.85,
                        auto_wah_freq_hz: freq,
                        auto_wah_resonance_q: 8.0 + 4.0 * frac as f32,
                        auto_wah_mix: 0.95,
                        articulation: ClavinetArticulation::ScreamingAutoWah,
                        curve: ClavinetInterpolationCurve::CubicCatmullRom,
                    });
                }
            }

            ClavinetGesturePattern::ChamberStaccatoGroove {
                groove_length_beats,
                yarn_damping,
            } => {
                self.loop_length_beats = *groove_length_beats;
                self.waypoints.push(ClavinetWaypoint {
                    beat: 0.0,
                    pickup_mode: ClavinetPickupMode::NeckOnly,
                    pickup_blend: 0.0,
                    pickup_phase_deg: 0.0,
                    anvil_hardness: 0.55,
                    yarn_damping: *yarn_damping,
                    tone_switches_mask: 8, // Soft
                    auto_wah_enabled: false,
                    auto_wah_sensitivity: 0.40,
                    auto_wah_freq_hz: 250.0,
                    auto_wah_resonance_q: 3.5,
                    auto_wah_mix: 0.60,
                    articulation: ClavinetArticulation::MellowChamber,
                    curve: ClavinetInterpolationCurve::Linear,
                });

                self.waypoints.push(ClavinetWaypoint {
                    beat: *groove_length_beats,
                    pickup_mode: ClavinetPickupMode::NeckOnly,
                    pickup_blend: 0.0,
                    pickup_phase_deg: 0.0,
                    anvil_hardness: 0.55,
                    yarn_damping: *yarn_damping,
                    tone_switches_mask: 8,
                    auto_wah_enabled: false,
                    auto_wah_sensitivity: 0.40,
                    auto_wah_freq_hz: 250.0,
                    auto_wah_resonance_q: 3.5,
                    auto_wah_mix: 0.60,
                    articulation: ClavinetArticulation::MellowChamber,
                    curve: ClavinetInterpolationCurve::Linear,
                });
            }

            ClavinetGesturePattern::CustomSpline => {
                if self.waypoints.is_empty() {
                    self.waypoints.push(ClavinetWaypoint::default());
                }
            }
        }
    }

    /// Evaluates continuous parameters at the specified beat position.
    pub fn evaluate_at_beat(&self, beat: f64) -> ClavinetBusSnapshot {
        if self.waypoints.is_empty() {
            return ClavinetBusSnapshot::default();
        }

        if self.waypoints.len() == 1 {
            let wp = self.waypoints[0];
            return ClavinetBusSnapshot {
                articulation: wp.articulation,
                pickup_mode: wp.pickup_mode,
                pickup_blend: wp.pickup_blend,
                pickup_phase_deg: wp.pickup_phase_deg,
                anvil_hardness: wp.anvil_hardness,
                yarn_damping: wp.yarn_damping,
                tone_switches_mask: wp.tone_switches_mask,
                auto_wah_enabled: wp.auto_wah_enabled,
                auto_wah_sensitivity: wp.auto_wah_sensitivity,
                auto_wah_freq_hz: wp.auto_wah_freq_hz,
                auto_wah_resonance_q: wp.auto_wah_resonance_q,
                auto_wah_mix: wp.auto_wah_mix,
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

        let blend = match curve {
            ClavinetInterpolationCurve::Hold => w1.pickup_blend,
            ClavinetInterpolationCurve::Linear => lerp(w1.pickup_blend, w2.pickup_blend, t),
            _ => catmull_rom_1d(w0.pickup_blend, w1.pickup_blend, w2.pickup_blend, w3.pickup_blend, t),
        }.clamp(0.0, 1.0);

        let phase = match curve {
            ClavinetInterpolationCurve::Hold => w1.pickup_phase_deg,
            ClavinetInterpolationCurve::Linear => lerp(w1.pickup_phase_deg, w2.pickup_phase_deg, t),
            _ => catmull_rom_1d(w0.pickup_phase_deg, w1.pickup_phase_deg, w2.pickup_phase_deg, w3.pickup_phase_deg, t),
        }.clamp(0.0, 180.0);

        let hardness = match curve {
            ClavinetInterpolationCurve::Hold => w1.anvil_hardness,
            ClavinetInterpolationCurve::Linear => lerp(w1.anvil_hardness, w2.anvil_hardness, t),
            _ => catmull_rom_1d(w0.anvil_hardness, w1.anvil_hardness, w2.anvil_hardness, w3.anvil_hardness, t),
        }.clamp(0.0, 1.0);

        let damping = match curve {
            ClavinetInterpolationCurve::Hold => w1.yarn_damping,
            ClavinetInterpolationCurve::Linear => lerp(w1.yarn_damping, w2.yarn_damping, t),
            _ => catmull_rom_1d(w0.yarn_damping, w1.yarn_damping, w2.yarn_damping, w3.yarn_damping, t),
        }.clamp(0.0, 1.0);

        let wah_sens = match curve {
            ClavinetInterpolationCurve::Hold => w1.auto_wah_sensitivity,
            ClavinetInterpolationCurve::Linear => lerp(w1.auto_wah_sensitivity, w2.auto_wah_sensitivity, t),
            _ => catmull_rom_1d(w0.auto_wah_sensitivity, w1.auto_wah_sensitivity, w2.auto_wah_sensitivity, w3.auto_wah_sensitivity, t),
        }.clamp(0.0, 1.0);

        let wah_freq = match curve {
            ClavinetInterpolationCurve::Hold => w1.auto_wah_freq_hz,
            ClavinetInterpolationCurve::Linear => lerp(w1.auto_wah_freq_hz, w2.auto_wah_freq_hz, t),
            _ => catmull_rom_1d(w0.auto_wah_freq_hz, w1.auto_wah_freq_hz, w2.auto_wah_freq_hz, w3.auto_wah_freq_hz, t),
        }.clamp(50.0, 4000.0);

        let wah_res = match curve {
            ClavinetInterpolationCurve::Hold => w1.auto_wah_resonance_q,
            ClavinetInterpolationCurve::Linear => lerp(w1.auto_wah_resonance_q, w2.auto_wah_resonance_q, t),
            _ => catmull_rom_1d(w0.auto_wah_resonance_q, w1.auto_wah_resonance_q, w2.auto_wah_resonance_q, w3.auto_wah_resonance_q, t),
        }.clamp(0.5, 25.0);

        let wah_mix = match curve {
            ClavinetInterpolationCurve::Hold => w1.auto_wah_mix,
            ClavinetInterpolationCurve::Linear => lerp(w1.auto_wah_mix, w2.auto_wah_mix, t),
            _ => catmull_rom_1d(w0.auto_wah_mix, w1.auto_wah_mix, w2.auto_wah_mix, w3.auto_wah_mix, t),
        }.clamp(0.0, 1.0);

        let progress = if self.loop_length_beats > 0.0 {
            (effective_beat / self.loop_length_beats).clamp(0.0, 1.0) as f32
        } else {
            0.0
        };

        ClavinetBusSnapshot {
            articulation: if t > 0.5 { w2.articulation } else { w1.articulation },
            pickup_mode: if t > 0.5 { w2.pickup_mode } else { w1.pickup_mode },
            pickup_blend: blend,
            pickup_phase_deg: phase,
            anvil_hardness: hardness,
            yarn_damping: damping,
            tone_switches_mask: if t > 0.5 { w2.tone_switches_mask } else { w1.tone_switches_mask },
            auto_wah_enabled: w1.auto_wah_enabled,
            auto_wah_sensitivity: wah_sens,
            auto_wah_freq_hz: wah_freq,
            auto_wah_resonance_q: wah_res,
            auto_wah_mix: wah_mix,
            gesture_progress: progress,
            is_active: self.is_active,
        }
    }

    /// Evaluates and writes parameters into a `ClavinetBus`.
    pub fn apply_to_bus(&self, beat: f64, bus: &ClavinetBus) {
        let snap = self.evaluate_at_beat(beat);
        bus.apply_snapshot(&snap);
    }

    /// Dispatches evaluated state to central `ParamBus`.
    pub fn dispatch_to_param_bus(&self, beat: f64, param_bus: &ParamBus, base_id: ParamId) {
        let snap = self.evaluate_at_beat(beat);
        param_bus.set(ParamId(base_id.0), snap.articulation as u32 as f32);
        param_bus.set(ParamId(base_id.0 + 1), snap.pickup_mode as u32 as f32);
        param_bus.set(ParamId(base_id.0 + 2), snap.pickup_blend);
        param_bus.set(ParamId(base_id.0 + 3), snap.pickup_phase_deg);
        param_bus.set(ParamId(base_id.0 + 4), snap.anvil_hardness);
        param_bus.set(ParamId(base_id.0 + 5), snap.yarn_damping);
        param_bus.set(ParamId(base_id.0 + 6), snap.tone_switches_mask as f32);
        param_bus.set(ParamId(base_id.0 + 7), if snap.auto_wah_enabled { 1.0 } else { 0.0 });
        param_bus.set(ParamId(base_id.0 + 8), snap.auto_wah_sensitivity);
        param_bus.set(ParamId(base_id.0 + 9), snap.auto_wah_freq_hz);
        param_bus.set(ParamId(base_id.0 + 10), snap.auto_wah_resonance_q);
        param_bus.set(ParamId(base_id.0 + 11), snap.auto_wah_mix);
        param_bus.set(ParamId(base_id.0 + 12), snap.gesture_progress);
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
    fn test_clavinet_gesture_superstition_funk_evaluation() {
        let engine = ClavinetGestureEngine::new(ClavinetGesturePattern::SuperstitionFunk {
            phrase_length_beats: 8.0,
            max_anvil_hardness: 0.95,
        });

        let snap_start = engine.evaluate_at_beat(0.0);
        assert_eq!(snap_start.pickup_mode, ClavinetPickupMode::ParallelInPhase);
        assert!((snap_start.anvil_hardness - 0.75).abs() < 1e-4);

        let snap_mid = engine.evaluate_at_beat(4.0);
        assert!((snap_mid.anvil_hardness - 0.95).abs() < 0.05);

        let snap_end = engine.evaluate_at_beat(8.0);
        assert!((snap_end.anvil_hardness - 0.75).abs() < 1e-4);
    }

    #[test]
    fn test_clavinet_gesture_dispatch_zero_allocation() {
        let engine = ClavinetGestureEngine::default();
        let bus = ClavinetBus::new();

        {
            let _guard = AllocGuard::new();
            for b in 0..100 {
                let beat = b as f64 * 0.1;
                engine.apply_to_bus(beat, &bus);
            }
        }
    }

    #[test]
    fn test_clavinet_gesture_toml_roundtrip() {
        let engine = ClavinetGestureEngine::new(ClavinetGesturePattern::DynamicWahSweep {
            sweep_length_beats: 16.0,
            peak_freq_hz: 1800.0,
        });

        let toml_str = engine.to_toml().expect("Failed to serialize to TOML");
        let decoded: ClavinetGestureEngine =
            ClavinetGestureEngine::from_toml(&toml_str).expect("Failed to deserialize from TOML");

        assert_eq!(engine.waypoints.len(), decoded.waypoints.len());
        assert_eq!(engine.loop_length_beats, decoded.loop_length_beats);
    }
}
