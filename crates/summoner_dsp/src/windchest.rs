// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Windchest Aerodynamics & Dynamic Pneumatic Reservoir (Milestone 24).
//!
//! Simulates the dynamic air reservoir, bellows compliance, blower airflow replenishment,
//! multi-rank wind consumption sag, pneumatic tremulant oscillation, and flue slit acoustic velocity:
//!
//!   v = sqrt(2 * P / rho)
//!
//! where P is wind pressure in Pascals (from mmH2O) and rho is air density (1.204 kg/m^3 at 20°C).
//!
//! Enforces zero heap allocations in audio render loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

/// Standard air density at 20°C and 1 atm in kg/m^3.
pub const AIR_DENSITY_RHO: f32 = 1.204;

/// Conversion factor: 1 mmH2O = 9.80665 Pascals (N/m^2).
pub const MMH2O_TO_PASCALS: f32 = 9.80665;

/// Minimum windchest pressure in mmH2O.
pub const MIN_WIND_PRESSURE_MMH2O: f32 = 40.0;

/// Maximum windchest pressure in mmH2O.
pub const MAX_WIND_PRESSURE_MMH2O: f32 = 160.0;

/// Nominal default wind pressure in mmH2O (~3 inches water column).
pub const DEFAULT_WIND_PRESSURE_MMH2O: f32 = 75.0;

/// Calculate acoustic flue slit air velocity in m/s given pressure in mmH2O:
///
/// v = sqrt(2 * (P * 9.80665) / 1.204)
#[inline]
pub fn air_velocity_from_pressure(pressure_mmh2o: f32) -> f32 {
    let p_clamped = pressure_mmh2o.clamp(0.0, 500.0);
    let p_pascals = p_clamped * MMH2O_TO_PASCALS;
    (2.0 * p_pascals / AIR_DENSITY_RHO).sqrt()
}

/// Windchest configuration parameters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WindchestConfig {
    /// Nominal static wind pressure in mmH2O [40.0 ..= 160.0].
    pub nominal_pressure_mmh2o: f32,
    /// Bellows reservoir compliance / damping coefficient [0.001 ..= 0.05].
    pub sag_coefficient: f32,
    /// Reservoir recovery / blower replenishment speed in seconds [0.01 ..= 0.5].
    pub recovery_time_sec: f32,
    /// Pneumatic tremulant enabled.
    pub tremulant_enabled: bool,
    /// Tremulant pulsation frequency in Hz [2.0 ..= 10.0].
    pub tremulant_rate_hz: f32,
    /// Tremulant modulation depth [0.0 ..= 0.5].
    pub tremulant_depth: f32,
}

impl Default for WindchestConfig {
    fn default() -> Self {
        Self {
            nominal_pressure_mmh2o: DEFAULT_WIND_PRESSURE_MMH2O,
            sag_coefficient: 0.0065,
            recovery_time_sec: 0.08,
            tremulant_enabled: false,
            tremulant_rate_hz: 5.5,
            tremulant_depth: 0.12,
        }
    }
}

/// Dynamic pneumatic windchest reservoir simulator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Windchest {
    pub config: WindchestConfig,
    pub sample_rate: u32,
    /// Current instantaneous dynamic pressure in mmH2O before tremulant.
    pub current_pressure: f32,
    /// Instantaneous effective pressure after tremulant modulation in mmH2O.
    pub effective_pressure: f32,
    /// Current flue slit acoustic velocity in m/s.
    pub current_air_velocity: f32,
    /// Tremulant LFO continuous phase accumulator [0.0 .. 2*PI).
    pub tremulant_phase: f32,
    /// Filtered air demand from active sounding pipes.
    pub smoothed_demand: f32,
}

impl Default for Windchest {
    fn default() -> Self {
        Self::new(48000)
    }
}

impl Windchest {
    /// Create a new windchest simulator for the given sample rate.
    pub fn new(sample_rate: u32) -> Self {
        let config = WindchestConfig::default();
        let init_p = config.nominal_pressure_mmh2o;
        let init_v = air_velocity_from_pressure(init_p);
        Self {
            config,
            sample_rate: sample_rate.max(1),
            current_pressure: init_p,
            effective_pressure: init_p,
            current_air_velocity: init_v,
            tremulant_phase: 0.0,
            smoothed_demand: 0.0,
        }
    }

    /// Reset state to nominal defaults.
    pub fn reset(&mut self) {
        self.current_pressure = self.config.nominal_pressure_mmh2o;
        self.effective_pressure = self.config.nominal_pressure_mmh2o;
        self.current_air_velocity = air_velocity_from_pressure(self.current_pressure);
        self.tremulant_phase = 0.0;
        self.smoothed_demand = 0.0;
    }

    /// Update sample rate.
    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate.max(1);
    }

    /// Set nominal reservoir pressure in mmH2O.
    pub fn set_nominal_pressure(&mut self, pressure_mmh2o: f32) {
        let p = pressure_mmh2o.clamp(
            MIN_WIND_PRESSURE_MMH2O,
            MAX_WIND_PRESSURE_MMH2O,
        );
        self.config.nominal_pressure_mmh2o = p;
        if self.smoothed_demand < 0.001 {
            self.current_pressure = p;
            self.effective_pressure = p;
            self.current_air_velocity = air_velocity_from_pressure(p);
        }
    }

    /// Set tremulant pulsation parameters.
    pub fn set_tremulant(&mut self, enabled: bool, rate_hz: f32, depth: f32) {
        self.config.tremulant_enabled = enabled;
        self.config.tremulant_rate_hz = rate_hz.clamp(1.0, 15.0);
        self.config.tremulant_depth = depth.clamp(0.0, 0.5);
    }

    /// Process a single audio sample given total air demand from all active pipes.
    ///
    /// `total_air_demand` is a normalized sum proportional to active pipes (e.g. 0.0 for idle, ~5.0..20.0 for full tutti chord).
    /// Returns the effective dynamic pressure in mmH2O.
    #[inline]
    pub fn process_sample(&mut self, total_air_demand: f32) -> f32 {
        let dt = 1.0 / self.sample_rate as f32;

        // Smooth air demand (simulate acoustic mass of air in windchest trunk)
        let demand_tau = 0.015; // 15ms smoothing
        let demand_alpha = (-dt / demand_tau).exp();
        self.smoothed_demand = demand_alpha * self.smoothed_demand + (1.0 - demand_alpha) * total_air_demand;

        // Pressure sag: target pressure sags under load
        let target_sag = self.smoothed_demand * self.config.sag_coefficient * self.config.nominal_pressure_mmh2o;
        let target_pressure = (self.config.nominal_pressure_mmh2o - target_sag).max(20.0);

        // Blower replenishment recovery towards target pressure
        let rec_tau = self.config.recovery_time_sec.max(0.005);
        let rec_alpha = (-dt / rec_tau).exp();
        self.current_pressure = rec_alpha * self.current_pressure + (1.0 - rec_alpha) * target_pressure;

        // Tremulant modulation
        if self.config.tremulant_enabled {
            let phase_inc = (TAU * self.config.tremulant_rate_hz) / self.sample_rate as f32;
            self.tremulant_phase = (self.tremulant_phase + phase_inc) % TAU;
            // Pneumatic tremulant pulses pressure downward below nominal
            let trem_mod = 1.0 - self.config.tremulant_depth * (0.5 * (self.tremulant_phase.sin() + 1.0));
            self.effective_pressure = (self.current_pressure * trem_mod).max(10.0);
        } else {
            self.effective_pressure = self.current_pressure;
        }

        self.current_air_velocity = air_velocity_from_pressure(self.effective_pressure);
        self.effective_pressure
    }

    /// Process a block of samples with constant or average air demand.
    #[inline]
    pub fn process_block(&mut self, total_air_demand: f32, block_size: usize) -> f32 {
        let mut last_p = self.effective_pressure;
        for _ in 0..block_size {
            last_p = self.process_sample(total_air_demand);
        }
        last_p
    }

    /// Get current effective pressure in mmH2O.
    #[inline]
    pub fn effective_pressure(&self) -> f32 {
        self.effective_pressure
    }

    /// Get current acoustic flue slit air velocity in m/s.
    #[inline]
    pub fn air_velocity(&self) -> f32 {
        self.current_air_velocity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_air_velocity_equation_exact() {
        // P = 75.0 mmH2O -> P_pa = 75.0 * 9.80665 = 735.49875 Pa
        // v = sqrt(2 * 735.49875 / 1.204) = sqrt(1221.7587) = 34.953665 m/s
        let v75 = air_velocity_from_pressure(75.0);
        assert!((v75 - 34.9537).abs() < 0.01, "v75 was {}", v75);

        // P = 160.0 mmH2O -> P_pa = 1569.064 Pa -> v = sqrt(3138.128 / 1.204) = 51.0506 m/s
        let v160 = air_velocity_from_pressure(160.0);
        assert!((v160 - 51.0506).abs() < 0.01, "v160 was {}", v160);

        // P = 40.0 mmH2O -> P_pa = 392.266 Pa -> v = sqrt(784.532 / 1.204) = 25.5253 m/s
        let v40 = air_velocity_from_pressure(40.0);
        assert!((v40 - 25.5253).abs() < 0.01, "v40 was {}", v40);
    }

    #[test]
    fn test_windchest_pressure_sag_under_heavy_demand() {
        let mut chest = Windchest::new(48000);
        chest.set_nominal_pressure(80.0);
        assert_eq!(chest.effective_pressure(), 80.0);

        // Idle for 1000 samples (zero demand) -> stays at 80.0
        for _ in 0..1000 {
            chest.process_sample(0.0);
        }
        assert!((chest.effective_pressure() - 80.0).abs() < 0.01);

        // Heavy chord demand (10 active pipes sounding)
        for _ in 0..48000 {
            chest.process_sample(10.0);
        }
        // Sag expected: ~10 * 0.0065 * 80 = 5.2 mmH2O drop -> ~74.8 mmH2O
        let sagged = chest.effective_pressure();
        assert!(sagged < 78.0 && sagged > 70.0, "Expected sagged pressure, got {}", sagged);

        // Release demand (pipes stop) -> pressure recovers to 80.0
        for _ in 0..48000 {
            chest.process_sample(0.0);
        }
        assert!((chest.effective_pressure() - 80.0).abs() < 0.05, "Expected recovered pressure, got {}", chest.effective_pressure());
    }

    #[test]
    fn test_windchest_tremulant_modulation() {
        let mut chest = Windchest::new(48000);
        chest.set_nominal_pressure(75.0);
        chest.set_tremulant(true, 6.0, 0.20);

        let mut min_p = 1000.0f32;
        let mut max_p = 0.0f32;
        for _ in 0..48000 {
            let p = chest.process_sample(0.0);
            if p < min_p { min_p = p; }
            if p > max_p { max_p = p; }
        }

        assert!(min_p < 65.0, "Tremulant should drop pressure, min_p={}", min_p);
        assert!(max_p <= 76.0, "Max pressure should be close to nominal, max_p={}", max_p);
        assert!(max_p - min_p > 10.0, "Modulation depth should produce pressure swing");
    }
}
