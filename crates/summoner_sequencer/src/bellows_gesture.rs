// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Dynamic Bellows Expression Gesture Timeline & Free-Reed Articulation Engine (Milestone 28).
//!
//! Provides multi-dimensional continuous physical modeling trajectory generation for
//! Bellows driving pressure forces [-1200..+1200 Pa], push/pull direction switches,
//! pallet valve key velocities, cassotto tone chamber aperture shutters, multi-rank musette
//! detune spreads, and register switches with Catmull-Rom spline smoothing, TOML serialization,
//! and sample-accurate lock-free parameter dispatch into `BellowsBus` and `ParamBus`.
//!
//! Enforces zero heap allocations in real-time execution loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use summoner_core::bellows_bus::{
    BellowsArticulation, BellowsBus, BellowsBusSnapshot,
};
use summoner_core::param_bus::{ParamBus, ParamId};

/// Interpolation curve between discrete Bellows keyframe waypoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BellowsInterpolationCurve {
    #[default]
    CubicCatmullRom,
    CubicBezier,
    Linear,
    Hold,
    Exponential,
}

/// A single keyframed multi-dimensional Bellows articulation waypoint.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BellowsWaypoint {
    /// Timeline beat position in quarter note beats.
    pub beat: f64,
    /// Driving bellows pressure in Pascals $[-1200.0 ..= +1200.0]$ (positive = push, negative = pull).
    pub bellows_pressure_pa: f32,
    /// Pallet valve velocity $[0.01 ..= 1.0]$.
    pub valve_velocity: f32,
    /// Cassotto mute aperture fraction $[0.0 ..= 1.0]$.
    pub cassotto_aperture: f32,
    /// Musette detuning spread in cents $[0.0 ..= 35.0]$.
    pub musette_detune_cents: f32,
    /// Active register switch bitmask.
    pub register_mask: u32,
    /// Reed material stiffness factor $[0.5 ..= 2.0]$.
    pub reed_stiffness: f32,
    /// Master gain $[0.0 ..= 2.0]$.
    pub master_gain: f32,
    /// Articulation technique.
    pub articulation: BellowsArticulation,
    /// Interpolation curve leading to this waypoint.
    pub curve: BellowsInterpolationCurve,
}

impl Default for BellowsWaypoint {
    fn default() -> Self {
        let (p_pa, _dir, vel, cassotto, musette, mask, stiff, mgain) =
            BellowsArticulation::TangoMarcatoAccented.nominal_parameters();

        Self {
            beat: 0.0,
            bellows_pressure_pa: p_pa,
            valve_velocity: vel,
            cassotto_aperture: cassotto,
            musette_detune_cents: musette,
            register_mask: mask,
            reed_stiffness: stiff,
            master_gain: mgain,
            articulation: BellowsArticulation::TangoMarcatoAccented,
            curve: BellowsInterpolationCurve::CubicCatmullRom,
        }
    }
}

/// Pre-configured algorithmic Bellows gesture trajectory patterns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BellowsGesturePattern {
    /// Dramatic Argentine Tango Marcato with sharp push/pull accents and high pressure swings.
    TangoBandoneonAccented {
        phrase_length_beats: f64,
        max_pressure_pa: f32,
    },
    /// French Musette Waltz with smooth undulating swells and wet tremolo.
    MusetteWaltzSwell {
        phrase_length_beats: f64,
        musette_cents: f32,
    },
    /// Russian Bayan rapid bellows shake micro-tremolo across dense chords.
    BayanVirtuosoBellowsShake {
        phrase_length_beats: f64,
        shake_frequency_hz: f32,
    },
    /// Continuous Meditative Harmonium suction drone in deep wood box.
    HarmoniumMeditativeDrone {
        phrase_length_beats: f64,
    },
    /// English Concertina fast staccato with crisp valve key releases.
    ConcertinaFastStaccato {
        phrase_length_beats: f64,
    },
}

/// Dynamic Bellows articulation timeline and parameter trajectory engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BellowsGestureEngine {
    pub waypoints: Vec<BellowsWaypoint>,
    pub loop_length_beats: f64,
    pub is_looping: bool,
}

impl Default for BellowsGestureEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl BellowsGestureEngine {
    pub fn new() -> Self {
        Self {
            waypoints: Vec::new(),
            loop_length_beats: 4.0,
            is_looping: true,
        }
    }

    pub fn with_pattern(pattern: BellowsGesturePattern) -> Self {
        let mut engine = Self::new();
        engine.load_pattern(pattern);
        engine
    }

    pub fn add_waypoint(&mut self, waypoint: BellowsWaypoint) {
        self.waypoints.push(waypoint);
        self.sort_waypoints();
    }

    pub fn sort_waypoints(&mut self) {
        self.waypoints.sort_by(|a, b| a.beat.partial_cmp(&b.beat).unwrap_or(std::cmp::Ordering::Equal));
    }

    pub fn clear(&mut self) {
        self.waypoints.clear();
    }

    pub fn load_pattern(&mut self, pattern: BellowsGesturePattern) {
        self.clear();

        match &pattern {
            BellowsGesturePattern::TangoBandoneonAccented { phrase_length_beats, max_pressure_pa } => {
                self.loop_length_beats = *phrase_length_beats;
                let max_p = *max_pressure_pa;

                self.waypoints.push(BellowsWaypoint {
                    beat: 0.0,
                    bellows_pressure_pa: max_p * 0.85, // Strong push accent
                    valve_velocity: 0.92,
                    cassotto_aperture: 0.70,
                    musette_detune_cents: 2.0,
                    register_mask: 0b00011, // 16'+8' Bandoneon
                    reed_stiffness: 1.25,
                    master_gain: 0.90,
                    articulation: BellowsArticulation::TangoMarcatoAccented,
                    curve: BellowsInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(BellowsWaypoint {
                    beat: phrase_length_beats * 0.25,
                    bellows_pressure_pa: 250.0, // Release
                    valve_velocity: 0.65,
                    cassotto_aperture: 0.60,
                    musette_detune_cents: 2.0,
                    register_mask: 0b00011,
                    reed_stiffness: 1.25,
                    master_gain: 0.85,
                    articulation: BellowsArticulation::TangoMarcatoAccented,
                    curve: BellowsInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(BellowsWaypoint {
                    beat: phrase_length_beats * 0.50,
                    bellows_pressure_pa: -max_p * 0.90, // Pull accent
                    valve_velocity: 0.95,
                    cassotto_aperture: 0.75,
                    musette_detune_cents: 2.0,
                    register_mask: 0b00011,
                    reed_stiffness: 1.25,
                    master_gain: 0.92,
                    articulation: BellowsArticulation::TangoMarcatoAccented,
                    curve: BellowsInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(BellowsWaypoint {
                    beat: phrase_length_beats * 0.75,
                    bellows_pressure_pa: -300.0,
                    valve_velocity: 0.60,
                    cassotto_aperture: 0.65,
                    musette_detune_cents: 2.0,
                    register_mask: 0b00011,
                    reed_stiffness: 1.25,
                    master_gain: 0.85,
                    articulation: BellowsArticulation::TangoMarcatoAccented,
                    curve: BellowsInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(BellowsWaypoint {
                    beat: *phrase_length_beats,
                    bellows_pressure_pa: max_p * 0.85,
                    valve_velocity: 0.92,
                    cassotto_aperture: 0.70,
                    musette_detune_cents: 2.0,
                    register_mask: 0b00011,
                    reed_stiffness: 1.25,
                    master_gain: 0.90,
                    articulation: BellowsArticulation::TangoMarcatoAccented,
                    curve: BellowsInterpolationCurve::CubicCatmullRom,
                });
            }

            BellowsGesturePattern::MusetteWaltzSwell { phrase_length_beats, musette_cents } => {
                self.loop_length_beats = *phrase_length_beats;

                self.waypoints.push(BellowsWaypoint {
                    beat: 0.0,
                    bellows_pressure_pa: 300.0,
                    valve_velocity: 0.70,
                    cassotto_aperture: 0.80,
                    musette_detune_cents: *musette_cents,
                    register_mask: 0b01110, // Musette 8'+8'+8'-
                    reed_stiffness: 1.00,
                    master_gain: 0.82,
                    articulation: BellowsArticulation::MusetteWaltzSwell,
                    curve: BellowsInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(BellowsWaypoint {
                    beat: phrase_length_beats * 0.5,
                    bellows_pressure_pa: 580.0, // Crest of swell
                    valve_velocity: 0.88,
                    cassotto_aperture: 0.95,
                    musette_detune_cents: *musette_cents * 1.1,
                    register_mask: 0b01110,
                    reed_stiffness: 1.00,
                    master_gain: 0.90,
                    articulation: BellowsArticulation::MusetteWaltzSwell,
                    curve: BellowsInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(BellowsWaypoint {
                    beat: *phrase_length_beats,
                    bellows_pressure_pa: 300.0,
                    valve_velocity: 0.70,
                    cassotto_aperture: 0.80,
                    musette_detune_cents: *musette_cents,
                    register_mask: 0b01110,
                    reed_stiffness: 1.00,
                    master_gain: 0.82,
                    articulation: BellowsArticulation::MusetteWaltzSwell,
                    curve: BellowsInterpolationCurve::CubicCatmullRom,
                });
            }

            BellowsGesturePattern::BayanVirtuosoBellowsShake { phrase_length_beats, shake_frequency_hz } => {
                self.loop_length_beats = *phrase_length_beats;
                let num_shakes = ((*phrase_length_beats * (*shake_frequency_hz as f64) * 0.5) as usize).clamp(4, 32);

                for i in 0..=num_shakes {
                    let b = (*phrase_length_beats / num_shakes as f64) * (i as f64);
                    let is_push_step = (i % 2) == 0;
                    let pressure = if is_push_step { 750.0 } else { -750.0 };

                    self.waypoints.push(BellowsWaypoint {
                        beat: b,
                        bellows_pressure_pa: pressure,
                        valve_velocity: 0.92,
                        cassotto_aperture: 1.00,
                        musette_detune_cents: 4.0,
                        register_mask: 0b11111, // Full Master
                        reed_stiffness: 1.40,
                        master_gain: 0.95,
                        articulation: BellowsArticulation::BayanVirtuosoBellowsShake,
                        curve: BellowsInterpolationCurve::CubicCatmullRom,
                    });
                }
            }

            BellowsGesturePattern::HarmoniumMeditativeDrone { phrase_length_beats } => {
                self.loop_length_beats = *phrase_length_beats;

                self.waypoints.push(BellowsWaypoint {
                    beat: 0.0,
                    bellows_pressure_pa: -260.0,
                    valve_velocity: 0.60,
                    cassotto_aperture: 0.50,
                    musette_detune_cents: 8.0,
                    register_mask: 0b00111,
                    reed_stiffness: 0.85,
                    master_gain: 0.80,
                    articulation: BellowsArticulation::HarmoniumMeditativeDrone,
                    curve: BellowsInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(BellowsWaypoint {
                    beat: phrase_length_beats * 0.5,
                    bellows_pressure_pa: -320.0,
                    valve_velocity: 0.65,
                    cassotto_aperture: 0.55,
                    musette_detune_cents: 8.5,
                    register_mask: 0b00111,
                    reed_stiffness: 0.85,
                    master_gain: 0.82,
                    articulation: BellowsArticulation::HarmoniumMeditativeDrone,
                    curve: BellowsInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(BellowsWaypoint {
                    beat: *phrase_length_beats,
                    bellows_pressure_pa: -260.0,
                    valve_velocity: 0.60,
                    cassotto_aperture: 0.50,
                    musette_detune_cents: 8.0,
                    register_mask: 0b00111,
                    reed_stiffness: 0.85,
                    master_gain: 0.80,
                    articulation: BellowsArticulation::HarmoniumMeditativeDrone,
                    curve: BellowsInterpolationCurve::CubicCatmullRom,
                });
            }

            BellowsGesturePattern::ConcertinaFastStaccato { phrase_length_beats } => {
                self.loop_length_beats = *phrase_length_beats;

                self.waypoints.push(BellowsWaypoint {
                    beat: 0.0,
                    bellows_pressure_pa: 520.0,
                    valve_velocity: 0.85,
                    cassotto_aperture: 0.90,
                    musette_detune_cents: 0.0,
                    register_mask: 0b00010,
                    reed_stiffness: 1.15,
                    master_gain: 0.88,
                    articulation: BellowsArticulation::ConcertinaFastStaccato,
                    curve: BellowsInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(BellowsWaypoint {
                    beat: phrase_length_beats * 0.5,
                    bellows_pressure_pa: -480.0,
                    valve_velocity: 0.88,
                    cassotto_aperture: 0.90,
                    musette_detune_cents: 0.0,
                    register_mask: 0b00010,
                    reed_stiffness: 1.15,
                    master_gain: 0.88,
                    articulation: BellowsArticulation::ConcertinaFastStaccato,
                    curve: BellowsInterpolationCurve::CubicCatmullRom,
                });

                self.waypoints.push(BellowsWaypoint {
                    beat: *phrase_length_beats,
                    bellows_pressure_pa: 520.0,
                    valve_velocity: 0.85,
                    cassotto_aperture: 0.90,
                    musette_detune_cents: 0.0,
                    register_mask: 0b00010,
                    reed_stiffness: 1.15,
                    master_gain: 0.88,
                    articulation: BellowsArticulation::ConcertinaFastStaccato,
                    curve: BellowsInterpolationCurve::CubicCatmullRom,
                });
            }
        }
    }

    /// Evaluate timeline trajectory at arbitrary beat and return interpolated snapshot.
    pub fn evaluate_at_beat(&self, beat: f64) -> BellowsBusSnapshot {
        if self.waypoints.is_empty() {
            let (p_pa, dir, vel, cassotto, musette, mask, stiff, mgain) =
                BellowsArticulation::TangoMarcatoAccented.nominal_parameters();
            return BellowsBusSnapshot {
                articulation: BellowsArticulation::TangoMarcatoAccented,
                bellows_pressure_pa: p_pa,
                push_direction: dir,
                valve_velocity: vel,
                cassotto_aperture: cassotto,
                musette_detune_cents: musette,
                register_mask: mask,
                reed_stiffness: stiff,
                master_gain: mgain,
            };
        }

        if self.waypoints.len() == 1 {
            let wp = &self.waypoints[0];
            return BellowsBusSnapshot {
                articulation: wp.articulation,
                bellows_pressure_pa: wp.bellows_pressure_pa,
                push_direction: if wp.bellows_pressure_pa >= 0.0 { 1.0 } else { -1.0 },
                valve_velocity: wp.valve_velocity,
                cassotto_aperture: wp.cassotto_aperture,
                musette_detune_cents: wp.musette_detune_cents,
                register_mask: wp.register_mask,
                reed_stiffness: wp.reed_stiffness,
                master_gain: wp.master_gain,
            };
        }

        let effective_beat = if self.is_looping && self.loop_length_beats > 0.0 {
            beat.rem_euclid(self.loop_length_beats)
        } else {
            beat
        };

        // Find surrounding waypoints for Catmull-Rom spline evaluation
        let n = self.waypoints.len();
        let mut idx1 = 0;
        while idx1 < n && self.waypoints[idx1].beat <= effective_beat {
            idx1 += 1;
        }

        if idx1 == 0 {
            let wp = &self.waypoints[0];
            return BellowsBusSnapshot {
                articulation: wp.articulation,
                bellows_pressure_pa: wp.bellows_pressure_pa,
                push_direction: if wp.bellows_pressure_pa >= 0.0 { 1.0 } else { -1.0 },
                valve_velocity: wp.valve_velocity,
                cassotto_aperture: wp.cassotto_aperture,
                musette_detune_cents: wp.musette_detune_cents,
                register_mask: wp.register_mask,
                reed_stiffness: wp.reed_stiffness,
                master_gain: wp.master_gain,
            };
        }

        if idx1 >= n {
            let wp = &self.waypoints[n - 1];
            return BellowsBusSnapshot {
                articulation: wp.articulation,
                bellows_pressure_pa: wp.bellows_pressure_pa,
                push_direction: if wp.bellows_pressure_pa >= 0.0 { 1.0 } else { -1.0 },
                valve_velocity: wp.valve_velocity,
                cassotto_aperture: wp.cassotto_aperture,
                musette_detune_cents: wp.musette_detune_cents,
                register_mask: wp.register_mask,
                reed_stiffness: wp.reed_stiffness,
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

        let interp = |v0: f32, v1: f32, v2: f32, v3: f32, curve: BellowsInterpolationCurve| -> f32 {
            match curve {
                BellowsInterpolationCurve::Hold => v1,
                BellowsInterpolationCurve::Linear => v1 + t * (v2 - v1),
                BellowsInterpolationCurve::Exponential => {
                    let min_v = v1.min(v2).max(1e-5);
                    let ratio = (v2.max(1e-5) / min_v).max(1e-5);
                    v1 * ratio.powf(t)
                }
                BellowsInterpolationCurve::CubicBezier => {
                    let t2 = t * t;
                    let t3 = t2 * t;
                    let b = 3.0 * t2 - 2.0 * t3;
                    v1 + b * (v2 - v1)
                }
                BellowsInterpolationCurve::CubicCatmullRom => {
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
        let pressure = interp(wp0.bellows_pressure_pa, wp1.bellows_pressure_pa, wp2.bellows_pressure_pa, wp3.bellows_pressure_pa, curve).clamp(-1200.0, 1200.0);
        let vel = interp(wp0.valve_velocity, wp1.valve_velocity, wp2.valve_velocity, wp3.valve_velocity, curve).clamp(0.01, 1.0);
        let cassotto = interp(wp0.cassotto_aperture, wp1.cassotto_aperture, wp2.cassotto_aperture, wp3.cassotto_aperture, curve).clamp(0.0, 1.0);
        let musette = interp(wp0.musette_detune_cents, wp1.musette_detune_cents, wp2.musette_detune_cents, wp3.musette_detune_cents, curve).clamp(0.0, 35.0);
        let stiff = interp(wp0.reed_stiffness, wp1.reed_stiffness, wp2.reed_stiffness, wp3.reed_stiffness, curve).clamp(0.5, 2.0);
        let mgain = interp(wp0.master_gain, wp1.master_gain, wp2.master_gain, wp3.master_gain, curve).clamp(0.0, 2.0);

        BellowsBusSnapshot {
            articulation: if t > 0.5 { wp2.articulation } else { wp1.articulation },
            bellows_pressure_pa: pressure,
            push_direction: if pressure >= 0.0 { 1.0 } else { -1.0 },
            valve_velocity: vel,
            cassotto_aperture: cassotto,
            musette_detune_cents: musette,
            register_mask: if t > 0.5 { wp2.register_mask } else { wp1.register_mask },
            reed_stiffness: stiff,
            master_gain: mgain,
        }
    }

    /// Dispatch interpolated parameters into `BellowsBus` and `ParamBus`.
    pub fn dispatch_to_bus(
        &self,
        beat: f64,
        bus: &BellowsBus,
        param_bus: Option<&ParamBus>,
        base_id: ParamId,
    ) {
        let snap = self.evaluate_at_beat(beat);

        bus.set_bellows_pressure(snap.bellows_pressure_pa);
        bus.set_valve_velocity(snap.valve_velocity);
        bus.set_cassotto_aperture(snap.cassotto_aperture);
        bus.set_musette_detune(snap.musette_detune_cents);
        bus.set_register_mask(snap.register_mask);
        bus.set_reed_stiffness(snap.reed_stiffness);
        bus.set_master_gain(snap.master_gain);

        if let Some(pb) = param_bus {
            pb.set(ParamId(base_id.0), snap.articulation as u32 as f32);
            pb.set(ParamId(base_id.0 + 1), snap.bellows_pressure_pa);
            pb.set(ParamId(base_id.0 + 2), snap.push_direction);
            pb.set(ParamId(base_id.0 + 3), snap.valve_velocity);
            pb.set(ParamId(base_id.0 + 4), snap.cassotto_aperture);
            pb.set(ParamId(base_id.0 + 5), snap.musette_detune_cents);
            pb.set(ParamId(base_id.0 + 6), snap.register_mask as f32);
            pb.set(ParamId(base_id.0 + 7), snap.reed_stiffness);
            pb.set(ParamId(base_id.0 + 8), snap.master_gain);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bellows_gesture_patterns_and_catmull_rom_evaluation() {
        for pattern in [
            BellowsGesturePattern::TangoBandoneonAccented { phrase_length_beats: 4.0, max_pressure_pa: 800.0 },
            BellowsGesturePattern::MusetteWaltzSwell { phrase_length_beats: 6.0, musette_cents: 18.0 },
            BellowsGesturePattern::BayanVirtuosoBellowsShake { phrase_length_beats: 4.0, shake_frequency_hz: 6.0 },
            BellowsGesturePattern::HarmoniumMeditativeDrone { phrase_length_beats: 8.0 },
            BellowsGesturePattern::ConcertinaFastStaccato { phrase_length_beats: 4.0 },
        ] {
            let engine = BellowsGestureEngine::with_pattern(pattern);
            assert!(!engine.waypoints.is_empty());

            // Evaluate at multiple beat positions
            for beat in [0.0, 1.0, 2.0, 3.5, 4.0, 5.8] {
                let snap = engine.evaluate_at_beat(beat);
                assert!(snap.bellows_pressure_pa.is_finite());
                assert!(snap.valve_velocity > 0.0);
                assert!(snap.cassotto_aperture >= 0.0);
                assert!(snap.musette_detune_cents >= 0.0);
                assert!(snap.master_gain > 0.0);
            }
        }
    }

    #[test]
    fn test_bellows_gesture_dispatch_to_bus() {
        let engine = BellowsGestureEngine::with_pattern(BellowsGesturePattern::TangoBandoneonAccented {
            phrase_length_beats: 4.0,
            max_pressure_pa: 700.0,
        });
        let bus = BellowsBus::new();
        let mut param_bus = ParamBus::new();
        summoner_core::bellows_bus::register_bellows_params(&mut param_bus, summoner_core::bellows_bus::BELLOWS_BASE_PARAM_ID);

        engine.dispatch_to_bus(0.0, &bus, Some(&param_bus), summoner_core::bellows_bus::BELLOWS_BASE_PARAM_ID);

        let snap = bus.snapshot();
        assert!((snap.bellows_pressure_pa - 595.0).abs() < 1e-3); // 700 * 0.85
    }
}
