// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Lock-Free Glass Armonica & Crystal Resonator Bus Parameter Routing (Milestone 31).
//!
//! Provides thread-safe lock-free parameter routing across audio, sequencer, and GUI threads
//! for Franklin Glass Armonica and Quartz Crystal Singing Bowl physical modeling parameters:
//! - Spindle Rotation Speed $\omega \in [0.0 ..= 6\pi]\text{ rad/s}$
//! - Wet Finger Normal Contact Force $F_N \in [0.05 ..= 2.5]\text{ N}$
//! - Water Trough / Bowl Water Fill Level $h \in [0.0 ..= 1.0]$
//! - Soft Mallet / Striker Velocity $[0.0 ..= 1.5]$
//! - Moisture Lubrication Coefficient $[0.1 ..= 1.0]$
//! - Mahogany Chassis Acoustic Resonance $[0.0 ..= 1.0]$
//! - Thin-Shell Q-Factor Scaling $[0.2 ..= 2.5]$
//! - Master Output Gain $[0.0 ..= 2.0]$
//!
//! Enforces zero heap allocations in real-time audio threads under `AllocGuard`.

use crate::param_bus::{ParamBus, ParamId};
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use std::sync::atomic::{AtomicU32, Ordering};

/// Performance styles and articulation presets for the Glass Armonica.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u32)]
pub enum GlassArticulation {
    /// Authentic 1761 Franklin Quartz Glass (Pure crystalline tone, steady rubbing).
    #[default]
    FranklinAuthenticContinuous = 0,
    /// Mesmer Magnetic Healing Chalice (Slow rotation, deep hydro-acoustic beating).
    MesmerMagneticHealing = 1,
    /// Mozart Concert Virtuoso Adagio (Crisp transient response, dynamic finger pressure).
    MozartVirtuosoAdagio = 2,
    /// Ethereal Borosilicate Swell (High-frequency overtone shimmer, expansive sustain).
    EtherealBorosilicateSwell = 3,
    /// Water-Tuned Crystal Glissando (Dynamic fluid mass-loading pitch shifts).
    WaterTunedCrystalGlissando = 4,
    /// Pure Quartz Crystal Singing Bowl Meditation (Ultra-high Q factor, sustained ring).
    CrystalSingingBowlMeditation = 5,
    /// Custom multi-keyframe Catmull-Rom spline trajectory.
    CustomSpline = 6,
}

/// Full nominal tuple parameters for Glass Armonica articulation:
/// (spindle_speed_rad_s, normal_force_n, water_fill_level, strike_velocity,
///  moisture_lubrication, chassis_resonance, q_scale, master_gain)
pub type GlassNominalParameters = (
    f32, // spindle_speed_rad_s [0.0 ..= 6.0 * PI]
    f32, // normal_force_n [0.05 ..= 2.50]
    f32, // water_fill_level [0.0 ..= 1.0]
    f32, // strike_velocity [0.0 ..= 1.5]
    f32, // moisture_lubrication [0.1 ..= 1.0]
    f32, // chassis_resonance [0.0 ..= 1.0]
    f32, // q_scale [0.2 ..= 2.5]
    f32, // master_gain [0.0 ..= 2.0]
);

impl GlassArticulation {
    pub fn from_u32(val: u32) -> Self {
        match val {
            0 => Self::FranklinAuthenticContinuous,
            1 => Self::MesmerMagneticHealing,
            2 => Self::MozartVirtuosoAdagio,
            3 => Self::EtherealBorosilicateSwell,
            4 => Self::WaterTunedCrystalGlissando,
            5 => Self::CrystalSingingBowlMeditation,
            6 => Self::CustomSpline,
            _ => Self::FranklinAuthenticContinuous,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::FranklinAuthenticContinuous => "1761 Franklin Authentic Continuous Rub",
            Self::MesmerMagneticHealing => "Mesmer Magnetic Healing Chalice",
            Self::MozartVirtuosoAdagio => "Mozart Concert Virtuoso Adagio",
            Self::EtherealBorosilicateSwell => "Ethereal Borosilicate Swell",
            Self::WaterTunedCrystalGlissando => "Water-Tuned Crystal Glissando",
            Self::CrystalSingingBowlMeditation => "Quartz Crystal Singing Bowl Meditation",
            Self::CustomSpline => "Custom Spline Gesture",
        }
    }

    /// Returns nominal default parameter values tuple.
    pub fn nominal_parameters(&self) -> GlassNominalParameters {
        match self {
            Self::FranklinAuthenticContinuous => (2.5 * PI, 0.45, 0.05, 0.0, 0.85, 0.75, 1.00, 0.90),
            Self::MesmerMagneticHealing => (1.5 * PI, 0.65, 0.35, 0.0, 0.90, 0.85, 1.25, 0.95),
            Self::MozartVirtuosoAdagio => (3.0 * PI, 0.50, 0.02, 0.2, 0.85, 0.70, 0.95, 0.92),
            Self::EtherealBorosilicateSwell => (2.2 * PI, 0.40, 0.10, 0.0, 0.80, 0.80, 1.15, 0.88),
            Self::WaterTunedCrystalGlissando => (2.0 * PI, 0.55, 0.50, 0.1, 0.85, 0.65, 1.05, 0.90),
            Self::CrystalSingingBowlMeditation => (1.2 * PI, 0.70, 0.0, 0.4, 0.95, 0.90, 1.50, 0.95),
            Self::CustomSpline => (2.5 * PI, 0.45, 0.05, 0.0, 0.85, 0.75, 1.00, 0.90),
        }
    }
}

/// Instantaneous lock-free snapshot of Glass Armonica bus parameters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GlassArmonicaBusSnapshot {
    pub articulation: GlassArticulation,
    pub spindle_speed_rad_s: f32,
    pub normal_force_n: f32,
    pub water_fill_level: f32,
    pub strike_velocity: f32,
    pub moisture_lubrication: f32,
    pub chassis_resonance: f32,
    pub q_scale: f32,
    pub master_gain: f32,
}

/// Default base ParamId for Glass Armonica physical modeling synthesis.
pub const GLASS_ARMONICA_BASE_PARAM_ID: ParamId = ParamId(1280);

/// Helper to pre-register all Glass Armonica parameters into a ParamBus.
pub fn register_glass_armonica_params(param_bus: &mut ParamBus, base_id: ParamId) {
    let (speed, force, water, strike, moist, chassis, qscale, mgain) =
        GlassArticulation::FranklinAuthenticContinuous.nominal_parameters();

    param_bus.register(ParamId(base_id.0), GlassArticulation::FranklinAuthenticContinuous as u32 as f32);
    param_bus.register(ParamId(base_id.0 + 1), speed);
    param_bus.register(ParamId(base_id.0 + 2), force);
    param_bus.register(ParamId(base_id.0 + 3), water);
    param_bus.register(ParamId(base_id.0 + 4), strike);
    param_bus.register(ParamId(base_id.0 + 5), moist);
    param_bus.register(ParamId(base_id.0 + 6), chassis);
    param_bus.register(ParamId(base_id.0 + 7), qscale);
    param_bus.register(ParamId(base_id.0 + 8), mgain);
}

/// Complete thread-safe lock-free Glass Armonica parameter bus.
#[derive(Debug)]
pub struct GlassArmonicaBus {
    pub articulation: AtomicU32,
    pub spindle_speed_rad_s: AtomicU32,
    pub normal_force_n: AtomicU32,
    pub water_fill_level: AtomicU32,
    pub strike_velocity: AtomicU32,
    pub moisture_lubrication: AtomicU32,
    pub chassis_resonance: AtomicU32,
    pub q_scale: AtomicU32,
    pub master_gain: AtomicU32,
}

impl Default for GlassArmonicaBus {
    fn default() -> Self {
        Self::new()
    }
}

impl GlassArmonicaBus {
    /// Create a new Glass Armonica parameter bus with default FranklinAuthenticContinuous settings.
    pub fn new() -> Self {
        let (speed, force, water, strike, moist, chassis, qscale, mgain) =
            GlassArticulation::FranklinAuthenticContinuous.nominal_parameters();

        Self {
            articulation: AtomicU32::new(GlassArticulation::FranklinAuthenticContinuous as u32),
            spindle_speed_rad_s: AtomicU32::new(speed.to_bits()),
            normal_force_n: AtomicU32::new(force.to_bits()),
            water_fill_level: AtomicU32::new(water.to_bits()),
            strike_velocity: AtomicU32::new(strike.to_bits()),
            moisture_lubrication: AtomicU32::new(moist.to_bits()),
            chassis_resonance: AtomicU32::new(chassis.to_bits()),
            q_scale: AtomicU32::new(qscale.to_bits()),
            master_gain: AtomicU32::new(mgain.to_bits()),
        }
    }

    /// Read an instantaneous atomic snapshot. Safe for real-time audio threads.
    #[inline]
    pub fn snapshot(&self) -> GlassArmonicaBusSnapshot {
        let art_u32 = self.articulation.load(Ordering::Relaxed);
        let speed_bits = self.spindle_speed_rad_s.load(Ordering::Relaxed);
        let force_bits = self.normal_force_n.load(Ordering::Relaxed);
        let water_bits = self.water_fill_level.load(Ordering::Relaxed);
        let strike_bits = self.strike_velocity.load(Ordering::Relaxed);
        let moist_bits = self.moisture_lubrication.load(Ordering::Relaxed);
        let chassis_bits = self.chassis_resonance.load(Ordering::Relaxed);
        let qscale_bits = self.q_scale.load(Ordering::Relaxed);
        let mgain_bits = self.master_gain.load(Ordering::Relaxed);

        GlassArmonicaBusSnapshot {
            articulation: GlassArticulation::from_u32(art_u32),
            spindle_speed_rad_s: f32::from_bits(speed_bits),
            normal_force_n: f32::from_bits(force_bits),
            water_fill_level: f32::from_bits(water_bits),
            strike_velocity: f32::from_bits(strike_bits),
            moisture_lubrication: f32::from_bits(moist_bits),
            chassis_resonance: f32::from_bits(chassis_bits),
            q_scale: f32::from_bits(qscale_bits),
            master_gain: f32::from_bits(mgain_bits),
        }
    }

    /// Update articulation and set nominal parameters atomically.
    pub fn set_articulation(&self, art: GlassArticulation) {
        let (speed, force, water, strike, moist, chassis, qscale, mgain) =
            art.nominal_parameters();

        self.articulation.store(art as u32, Ordering::Relaxed);
        self.spindle_speed_rad_s.store(speed.to_bits(), Ordering::Relaxed);
        self.normal_force_n.store(force.to_bits(), Ordering::Relaxed);
        self.water_fill_level.store(water.to_bits(), Ordering::Relaxed);
        self.strike_velocity.store(strike.to_bits(), Ordering::Relaxed);
        self.moisture_lubrication.store(moist.to_bits(), Ordering::Relaxed);
        self.chassis_resonance.store(chassis.to_bits(), Ordering::Relaxed);
        self.q_scale.store(qscale.to_bits(), Ordering::Relaxed);
        self.master_gain.store(mgain.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_spindle_speed(&self, speed: f32) {
        self.spindle_speed_rad_s.store(speed.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_normal_force(&self, force: f32) {
        self.normal_force_n.store(force.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_water_fill_level(&self, water: f32) {
        self.water_fill_level.store(water.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_strike_velocity(&self, strike: f32) {
        self.strike_velocity.store(strike.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_moisture_lubrication(&self, moist: f32) {
        self.moisture_lubrication.store(moist.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_chassis_resonance(&self, chassis: f32) {
        self.chassis_resonance.store(chassis.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_q_scale(&self, qscale: f32) {
        self.q_scale.store(qscale.to_bits(), Ordering::Relaxed);
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
    fn test_glass_armonica_bus_atomic_roundtrip() {
        let bus = GlassArmonicaBus::new();
        let snap = bus.snapshot();

        assert_eq!(snap.articulation, GlassArticulation::FranklinAuthenticContinuous);
        assert!((snap.spindle_speed_rad_s - 2.5 * PI).abs() < 1e-4);
        assert!((snap.normal_force_n - 0.45).abs() < 1e-4);

        // Switch to Mesmer healing mode
        bus.set_articulation(GlassArticulation::MesmerMagneticHealing);
        let snap2 = bus.snapshot();
        assert_eq!(snap2.articulation, GlassArticulation::MesmerMagneticHealing);
        assert!((snap2.spindle_speed_rad_s - 1.5 * PI).abs() < 1e-4);
        assert!((snap2.water_fill_level - 0.35).abs() < 1e-4);

        // Modify individual parameter
        bus.set_water_fill_level(0.75);
        let snap3 = bus.snapshot();
        assert!((snap3.water_fill_level - 0.75).abs() < 1e-4);
    }

    #[test]
    fn test_glass_armonica_bus_param_bus_registration() {
        let mut param_bus = ParamBus::new();
        register_glass_armonica_params(&mut param_bus, GLASS_ARMONICA_BASE_PARAM_ID);

        let speed_val = param_bus.get(ParamId(GLASS_ARMONICA_BASE_PARAM_ID.0 + 1));
        assert!(speed_val.is_some());
        assert!((speed_val.unwrap() - 2.5 * PI).abs() < 1e-4);
    }
}
