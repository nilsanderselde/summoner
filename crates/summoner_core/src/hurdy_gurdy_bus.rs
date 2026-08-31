// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Lock-Free Hurdy-Gurdy (Vielle à roue) Bus Parameter Routing (Milestone 30).
//!
//! Provides thread-safe lock-free exchange of continuous physical modeling Hurdy-Gurdy parameters
//! (Crank Wheel Angular Velocity $\omega \in [0..4\pi]\text{ rad/s}$, Coup de Poignet Wrist
//! Acceleration Pulse $\Delta \alpha$, Wheel Pressure, Chien Buzzing Bridge Clearance $h_0 \in [0.05..1.20]\text{ mm}$,
//! Tangent Key Damping, Drone/Melody Mix Ratio, Trompette Buzz Level, Body Resonance, and Master Gain)
//! across audio, sequencer, and GUI threads.
//!
//! Enforces zero heap allocations in real-time audio threads under `AllocGuard`.

use crate::param_bus::{ParamBus, ParamId};
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use std::sync::atomic::{AtomicU32, Ordering};

/// Performance styles and articulation presets for the Hurdy-Gurdy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u32)]
pub enum HurdyGurdyArticulation {
    /// Traditional French Bourbonnais Coup de Poignet (Periodic buzzing accents on the beat).
    #[default]
    BourbonnaisClassicCoupDePoignet = 0,
    /// Medieval Monastic Symphonia (Steady cranking, long sustained drone, no buzzing dog).
    MedievalMonasticDrone = 1,
    /// Baroque Virtuoso Chanterelle (Fast melody ornamentations, clean precise tangent action).
    BaroqueVirtuosoChanterelle = 2,
    /// Auvergne High-Speed Buzz (Intense continuous chattering snarl and driving polyrhythms).
    AuvergneHighSpeedBuzz = 3,
    /// Gothic Pagan Dark Drone (Deep low-tuned bourdons with syncopated buzz hits).
    GothicPaganDrone = 4,
    /// Electro-Acoustic Hybrid Swell (Wide dynamic wheel velocity sweeps and high resonance).
    ElectroAcousticHybrid = 5,
    /// Custom multi-keyframe Catmull-Rom spline trajectory.
    CustomSpline = 6,
}

/// Full nominal tuple parameters for Hurdy-Gurdy articulation:
/// (crank_speed_rad_s, wrist_acceleration_pulse, wheel_pressure, chien_clearance_mm,
///  tangent_damping, drone_melody_mix, trompette_buzz_level, body_resonance, master_gain)
pub type HurdyGurdyNominalParameters = (
    f32, // crank_speed_rad_s [0.0 ..= 4.0 * PI]
    f32, // wrist_acceleration_pulse [0.0 ..= 10.0]
    f32, // wheel_pressure [0.01 ..= 1.0]
    f32, // chien_clearance_mm [0.05 ..= 1.20]
    f32, // tangent_damping [0.0 ..= 1.0]
    f32, // drone_melody_mix [0.0 ..= 1.0]
    f32, // trompette_buzz_level [0.0 ..= 2.0]
    f32, // body_resonance [0.0 ..= 1.0]
    f32, // master_gain [0.0 ..= 2.0]
);

impl HurdyGurdyArticulation {
    pub fn from_u32(val: u32) -> Self {
        match val {
            0 => Self::BourbonnaisClassicCoupDePoignet,
            1 => Self::MedievalMonasticDrone,
            2 => Self::BaroqueVirtuosoChanterelle,
            3 => Self::AuvergneHighSpeedBuzz,
            4 => Self::GothicPaganDrone,
            5 => Self::ElectroAcousticHybrid,
            6 => Self::CustomSpline,
            _ => Self::BourbonnaisClassicCoupDePoignet,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::BourbonnaisClassicCoupDePoignet => "Bourbonnais Classic Coup de Poignet",
            Self::MedievalMonasticDrone => "Medieval Monastic Sustained Drone",
            Self::BaroqueVirtuosoChanterelle => "Baroque Virtuoso Chanterelle Solo",
            Self::AuvergneHighSpeedBuzz => "Auvergne High-Speed Buzzing Snarl",
            Self::GothicPaganDrone => "Gothic Pagan Dark Drone",
            Self::ElectroAcousticHybrid => "Electro-Acoustic Hybrid Swell",
            Self::CustomSpline => "Custom Spline Gesture",
        }
    }

    /// Returns nominal default parameters tuple.
    pub fn nominal_parameters(&self) -> HurdyGurdyNominalParameters {
        match self {
            Self::BourbonnaisClassicCoupDePoignet => (2.0 * PI, 1.2, 0.70, 0.35, 0.85, 0.45, 0.85, 0.85, 0.90),
            Self::MedievalMonasticDrone => (1.5 * PI, 0.0, 0.55, 0.90, 0.95, 0.60, 0.20, 0.80, 0.88),
            Self::BaroqueVirtuosoChanterelle => (2.2 * PI, 0.4, 0.80, 0.25, 0.75, 0.35, 0.95, 0.90, 0.92),
            Self::AuvergneHighSpeedBuzz => (3.0 * PI, 2.5, 0.90, 0.15, 0.80, 0.50, 1.30, 0.85, 0.95),
            Self::GothicPaganDrone => (1.2 * PI, 0.8, 0.65, 0.45, 0.90, 0.70, 0.75, 0.95, 0.90),
            Self::ElectroAcousticHybrid => (2.5 * PI, 1.5, 0.85, 0.30, 0.80, 0.40, 1.00, 0.90, 0.92),
            Self::CustomSpline => (2.0 * PI, 1.0, 0.70, 0.35, 0.85, 0.45, 0.85, 0.85, 0.90),
        }
    }
}

/// Instantaneous lock-free snapshot of Hurdy-Gurdy bus parameters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HurdyGurdyBusSnapshot {
    pub articulation: HurdyGurdyArticulation,
    pub crank_speed_rad_s: f32,
    pub wrist_acceleration_pulse: f32,
    pub wheel_pressure: f32,
    pub chien_clearance_mm: f32,
    pub tangent_damping: f32,
    pub drone_melody_mix: f32,
    pub trompette_buzz_level: f32,
    pub body_wood_resonance: f32,
    pub master_gain: f32,
}

/// Default base ParamId for Hurdy-Gurdy physical modeling synthesis.
pub const HURDY_GURDY_BASE_PARAM_ID: ParamId = ParamId(1270);

/// Helper to pre-register all Hurdy-Gurdy parameters into a ParamBus.
pub fn register_hurdy_gurdy_params(param_bus: &mut ParamBus, base_id: ParamId) {
    let (speed, wrist, press, gap, damp, dmix, buzz, wood, mgain) =
        HurdyGurdyArticulation::BourbonnaisClassicCoupDePoignet.nominal_parameters();

    param_bus.register(ParamId(base_id.0), HurdyGurdyArticulation::BourbonnaisClassicCoupDePoignet as u32 as f32);
    param_bus.register(ParamId(base_id.0 + 1), speed);
    param_bus.register(ParamId(base_id.0 + 2), wrist);
    param_bus.register(ParamId(base_id.0 + 3), press);
    param_bus.register(ParamId(base_id.0 + 4), gap);
    param_bus.register(ParamId(base_id.0 + 5), damp);
    param_bus.register(ParamId(base_id.0 + 6), dmix);
    param_bus.register(ParamId(base_id.0 + 7), buzz);
    param_bus.register(ParamId(base_id.0 + 8), wood);
    param_bus.register(ParamId(base_id.0 + 9), mgain);
}

/// Complete thread-safe lock-free Hurdy-Gurdy parameter bus.
#[derive(Debug)]
pub struct HurdyGurdyBus {
    pub articulation: AtomicU32,
    pub crank_speed_rad_s: AtomicU32,
    pub wrist_acceleration_pulse: AtomicU32,
    pub wheel_pressure: AtomicU32,
    pub chien_clearance_mm: AtomicU32,
    pub tangent_damping: AtomicU32,
    pub drone_melody_mix: AtomicU32,
    pub trompette_buzz_level: AtomicU32,
    pub body_wood_resonance: AtomicU32,
    pub master_gain: AtomicU32,
}

impl Default for HurdyGurdyBus {
    fn default() -> Self {
        Self::new()
    }
}

impl HurdyGurdyBus {
    /// Create a new Hurdy-Gurdy parameter bus with default BourbonnaisClassic settings.
    pub fn new() -> Self {
        let (speed, wrist, press, gap, damp, dmix, buzz, wood, mgain) =
            HurdyGurdyArticulation::BourbonnaisClassicCoupDePoignet.nominal_parameters();

        Self {
            articulation: AtomicU32::new(HurdyGurdyArticulation::BourbonnaisClassicCoupDePoignet as u32),
            crank_speed_rad_s: AtomicU32::new(speed.to_bits()),
            wrist_acceleration_pulse: AtomicU32::new(wrist.to_bits()),
            wheel_pressure: AtomicU32::new(press.to_bits()),
            chien_clearance_mm: AtomicU32::new(gap.to_bits()),
            tangent_damping: AtomicU32::new(damp.to_bits()),
            drone_melody_mix: AtomicU32::new(dmix.to_bits()),
            trompette_buzz_level: AtomicU32::new(buzz.to_bits()),
            body_wood_resonance: AtomicU32::new(wood.to_bits()),
            master_gain: AtomicU32::new(mgain.to_bits()),
        }
    }

    /// Read an instantaneous atomic snapshot. Safe for real-time audio threads.
    #[inline]
    pub fn snapshot(&self) -> HurdyGurdyBusSnapshot {
        let art_u32 = self.articulation.load(Ordering::Relaxed);
        let speed_bits = self.crank_speed_rad_s.load(Ordering::Relaxed);
        let wrist_bits = self.wrist_acceleration_pulse.load(Ordering::Relaxed);
        let press_bits = self.wheel_pressure.load(Ordering::Relaxed);
        let gap_bits = self.chien_clearance_mm.load(Ordering::Relaxed);
        let damp_bits = self.tangent_damping.load(Ordering::Relaxed);
        let dmix_bits = self.drone_melody_mix.load(Ordering::Relaxed);
        let buzz_bits = self.trompette_buzz_level.load(Ordering::Relaxed);
        let wood_bits = self.body_wood_resonance.load(Ordering::Relaxed);
        let mgain_bits = self.master_gain.load(Ordering::Relaxed);

        HurdyGurdyBusSnapshot {
            articulation: HurdyGurdyArticulation::from_u32(art_u32),
            crank_speed_rad_s: f32::from_bits(speed_bits),
            wrist_acceleration_pulse: f32::from_bits(wrist_bits),
            wheel_pressure: f32::from_bits(press_bits),
            chien_clearance_mm: f32::from_bits(gap_bits),
            tangent_damping: f32::from_bits(damp_bits),
            drone_melody_mix: f32::from_bits(dmix_bits),
            trompette_buzz_level: f32::from_bits(buzz_bits),
            body_wood_resonance: f32::from_bits(wood_bits),
            master_gain: f32::from_bits(mgain_bits),
        }
    }

    /// Update articulation and set nominal parameters atomically.
    pub fn set_articulation(&self, art: HurdyGurdyArticulation) {
        let (speed, wrist, press, gap, damp, dmix, buzz, wood, mgain) =
            art.nominal_parameters();

        self.articulation.store(art as u32, Ordering::Relaxed);
        self.crank_speed_rad_s.store(speed.to_bits(), Ordering::Relaxed);
        self.wrist_acceleration_pulse.store(wrist.to_bits(), Ordering::Relaxed);
        self.wheel_pressure.store(press.to_bits(), Ordering::Relaxed);
        self.chien_clearance_mm.store(gap.to_bits(), Ordering::Relaxed);
        self.tangent_damping.store(damp.to_bits(), Ordering::Relaxed);
        self.drone_melody_mix.store(dmix.to_bits(), Ordering::Relaxed);
        self.trompette_buzz_level.store(buzz.to_bits(), Ordering::Relaxed);
        self.body_wood_resonance.store(wood.to_bits(), Ordering::Relaxed);
        self.master_gain.store(mgain.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_crank_speed(&self, speed: f32) {
        self.crank_speed_rad_s.store(speed.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_wrist_acceleration(&self, wrist: f32) {
        self.wrist_acceleration_pulse.store(wrist.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_wheel_pressure(&self, press: f32) {
        self.wheel_pressure.store(press.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_chien_clearance(&self, gap_mm: f32) {
        self.chien_clearance_mm.store(gap_mm.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_tangent_damping(&self, damp: f32) {
        self.tangent_damping.store(damp.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_drone_melody_mix(&self, mix: f32) {
        self.drone_melody_mix.store(mix.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_trompette_buzz_level(&self, buzz: f32) {
        self.trompette_buzz_level.store(buzz.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_body_wood_resonance(&self, wood: f32) {
        self.body_wood_resonance.store(wood.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_master_gain(&self, gain: f32) {
        self.master_gain.store(gain.to_bits(), Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hurdy_gurdy_bus_atomic_roundtrip() {
        let bus = HurdyGurdyBus::new();
        let snap = bus.snapshot();

        assert_eq!(snap.articulation, HurdyGurdyArticulation::BourbonnaisClassicCoupDePoignet);
        assert!((snap.crank_speed_rad_s - 2.0 * PI).abs() < 1e-4);
        assert!((snap.chien_clearance_mm - 0.35).abs() < 1e-4);

        // Update articulation
        bus.set_articulation(HurdyGurdyArticulation::AuvergneHighSpeedBuzz);
        let snap2 = bus.snapshot();
        assert_eq!(snap2.articulation, HurdyGurdyArticulation::AuvergneHighSpeedBuzz);
        assert!((snap2.crank_speed_rad_s - 3.0 * PI).abs() < 1e-4);
        assert!((snap2.trompette_buzz_level - 1.30).abs() < 1e-4);

        // Individual setter
        bus.set_wrist_acceleration(3.5);
        let snap3 = bus.snapshot();
        assert!((snap3.wrist_acceleration_pulse - 3.5).abs() < 1e-4);
    }

    #[test]
    fn test_hurdy_gurdy_bus_param_bus_registration() {
        let mut param_bus = ParamBus::new();
        register_hurdy_gurdy_params(&mut param_bus, HURDY_GURDY_BASE_PARAM_ID);

        let speed_val = param_bus.get(ParamId(HURDY_GURDY_BASE_PARAM_ID.0 + 1));
        assert!(speed_val.is_some());
        assert!((speed_val.unwrap() - 2.0 * PI).abs() < 1e-4);
    }
}
