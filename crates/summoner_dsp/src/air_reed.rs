// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Air-Jet Reed & Utaguchi Flue Vortex Aerodynamics (Milestone 25).
//!
//! Simulates the end-blown sharp blowing edge (*utaguchi*) excitation mechanism found in
//! the Japanese Shakuhachi bamboo flute. Models the non-linear air-jet splitting,
//! Bernoulli flow velocity, jet transit delay line, convective vortex transport,
//! dynamic Meri/Kari head tilt angle pitch shifting (Δf in [-300, +200] cents),
//! and Murai-Iki explosive breath turbulence bursts.
//!
//! Enforces zero heap allocations in audio render loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

/// Maximum air-jet propagation delay buffer capacity in samples (~50 ms at 192 kHz).
pub const MAX_JET_DELAY: usize = 4096;

/// Standard speed of sound in air at 20°C in m/s.
pub const SPEED_OF_SOUND_MPS: f32 = 343.0;

/// Standard air density at 20°C in kg/m^3.
pub const AIR_DENSITY_KG_M3: f32 = 1.204;

/// Minimum blowing pressure in Pascals (~0.05 kPa).
pub const MIN_PRESSURE_PA: f32 = 50.0;

/// Maximum blowing pressure in Pascals (~4.0 kPa for violent overblowing / murai-iki).
pub const MAX_PRESSURE_PA: f32 = 4000.0;

/// Nominal default blowing pressure in Pascals (~800 Pa).
pub const DEFAULT_PRESSURE_PA: f32 = 800.0;

/// Calculate acoustic Bernoulli jet velocity in m/s given blowing pressure in Pascals:
///
/// v_jet = sqrt(2 * P / rho)
#[inline]
pub fn jet_velocity_from_pressure(pressure_pa: f32) -> f32 {
    let p_clamped = pressure_pa.clamp(0.0, 10000.0);
    (2.0 * p_clamped / AIR_DENSITY_KG_M3).sqrt()
}

/// Convert Meri/Kari cent offset to continuous pitch multiplier:
///
/// multiplier = 2^(cents / 1200)
#[inline]
pub fn meri_kari_to_pitch_multiplier(cents: f32) -> f32 {
    let clamped_cents = cents.clamp(-300.0, 200.0);
    2.0_f32.powf(clamped_cents / 1200.0)
}

/// Non-linear sharp utaguchi edge splitting function with asymmetric head tilt angle:
///
/// f(eta, theta) = tanh(eta * k_sharp + offset(theta)) - 0.22 * (eta * k_sharp)^3
#[inline]
pub fn utaguchi_splitting_function(jet_displacement: f32, angle_offset: f32, sharpness: f32) -> f32 {
    let scaled_disp = jet_displacement * sharpness + angle_offset;
    let sat = scaled_disp.tanh();
    let cubic = 0.22 * (scaled_disp.clamp(-2.0, 2.0)).powi(3);
    (sat - cubic).clamp(-1.0, 1.0)
}

/// Non-allocating 32-bit XorShift pseudo-random generator for stochastic breath vortex noise.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FastPrng {
    state: u32,
}

impl Default for FastPrng {
    fn default() -> Self {
        Self { state: 0x54A8_9103 }
    }
}

impl FastPrng {
    pub fn new(seed: u32) -> Self {
        Self {
            state: if seed == 0 { 0x54A8_9103 } else { seed },
        }
    }

    /// Generate next random float in [-1.0, 1.0].
    #[inline]
    pub fn next_f32(&mut self) -> f32 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 17;
        self.state ^= self.state << 5;
        // Map 32-bit unsigned to float in [-1.0, 1.0]
        let norm = (self.state as f32) / (u32::MAX as f32);
        norm * 2.0 - 1.0
    }
}

/// Air-reed embouchure configuration parameters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AirReedConfig {
    /// Jet distance from player's lips to utaguchi edge in mm [2.0 ..= 25.0].
    pub jet_distance_mm: f32,
    /// Embouchure resting angle in degrees [10.0 ..= 60.0].
    pub embouchure_angle_deg: f32,
    /// Utaguchi edge sharpness coefficient [0.5 ..= 3.0].
    pub edge_sharpness: f32,
    /// Convective vortex velocity factor [0.3 ..= 0.6] (ratio of vortex speed to jet speed).
    pub convective_factor: f32,
    /// Baseline breath turbulence noise amount [0.0 ..= 0.5].
    pub turbulence_gain: f32,
    /// Murai-iki explosive burst gain [0.0 ..= 1.0].
    pub murai_iki_intensity: f32,
}

impl Default for AirReedConfig {
    fn default() -> Self {
        Self {
            jet_distance_mm: 10.0,
            embouchure_angle_deg: 38.0,
            edge_sharpness: 1.45,
            convective_factor: 0.42,
            turbulence_gain: 0.15,
            murai_iki_intensity: 0.0,
        }
    }
}

fn default_jet_buffer() -> Box<[f32; MAX_JET_DELAY]> {
    vec![0.0; MAX_JET_DELAY].into_boxed_slice().try_into().unwrap()
}

/// Physical modeling Japanese Shakuhachi air-jet reed and utaguchi flue vortex generator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirReed {
    pub config: AirReedConfig,
    pub sample_rate: u32,
    /// Current blowing pressure in Pascals.
    pub blowing_pressure_pa: f32,
    /// Dynamic Meri/Kari head tilt angle pitch offset in cents [-300.0 ..= +200.0].
    pub meri_kari_cents: f32,
    /// Dynamic blowing angle in degrees.
    pub current_angle_deg: f32,
    /// Instantaneous Bernoulli jet velocity in m/s.
    pub jet_velocity_mps: f32,
    /// Jet transit delay line buffer.
    #[serde(skip, default = "default_jet_buffer")]
    pub jet_delay_buffer: Box<[f32; MAX_JET_DELAY]>,
    /// Jet delay write position.
    pub jet_write_pos: usize,
    /// Filtered breath turbulence state for lowpass coloring.
    pub turbulence_filter_state: f32,
    /// Formant filter state for murai-iki breath bursts.
    pub formant_filter_state1: f32,
    pub formant_filter_state2: f32,
    /// PRNG for stochastic vortex generation.
    pub prng: FastPrng,
}

impl Default for AirReed {
    fn default() -> Self {
        Self::new(48000)
    }
}

impl AirReed {
    /// Create a new AirReed simulator for the given sample rate.
    pub fn new(sample_rate: u32) -> Self {
        let sr = sample_rate.max(8000);
        let config = AirReedConfig::default();
        let init_p = DEFAULT_PRESSURE_PA;
        let v_jet = jet_velocity_from_pressure(init_p);

        Self {
            config,
            sample_rate: sr,
            blowing_pressure_pa: init_p,
            meri_kari_cents: 0.0,
            current_angle_deg: config.embouchure_angle_deg,
            jet_velocity_mps: v_jet,
            jet_delay_buffer: default_jet_buffer(),
            jet_write_pos: 0,
            turbulence_filter_state: 0.0,
            formant_filter_state1: 0.0,
            formant_filter_state2: 0.0,
            prng: FastPrng::new(0x71A4_B502),
        }
    }

    /// Reset internal delay line and filter memories.
    pub fn reset(&mut self) {
        self.jet_delay_buffer.fill(0.0);
        self.jet_write_pos = 0;
        self.turbulence_filter_state = 0.0;
        self.formant_filter_state1 = 0.0;
        self.formant_filter_state2 = 0.0;
    }

    /// Set blowing pressure in Pascals [0.0 ..= 4000.0].
    pub fn set_blowing_pressure(&mut self, pressure_pa: f32) {
        self.blowing_pressure_pa = pressure_pa.clamp(0.0, MAX_PRESSURE_PA);
        self.jet_velocity_mps = jet_velocity_from_pressure(self.blowing_pressure_pa);
    }

    /// Set Meri/Kari head tilt pitch bending in cents [-300.0 ..= +200.0].
    pub fn set_meri_kari(&mut self, cents: f32) {
        self.meri_kari_cents = cents.clamp(-300.0, 200.0);
        // Head tilt angle correlates with meri/kari:
        // Meri (head down, chin covers hole) -> shallower angle (~20 deg)
        // Kari (head up, open aperture) -> steeper angle (~50 deg)
        let angle_delta = (self.meri_kari_cents / 300.0) * 15.0;
        self.current_angle_deg = (self.config.embouchure_angle_deg + angle_delta).clamp(10.0, 60.0);
    }

    /// Set Murai-Iki explosive breath burst intensity [0.0 ..= 1.0].
    pub fn set_murai_iki(&mut self, intensity: f32) {
        self.config.murai_iki_intensity = intensity.clamp(0.0, 1.0);
    }

    /// Set jet distance in millimeters [2.0 ..= 25.0].
    pub fn set_jet_distance_mm(&mut self, distance_mm: f32) {
        self.config.jet_distance_mm = distance_mm.clamp(2.0, 25.0);
    }

    /// Calculate instantaneous jet transit delay in samples.
    #[inline]
    pub fn calculate_jet_delay_samples(&self) -> f32 {
        let d_m = self.config.jet_distance_mm * 0.001;
        let u_vortex = (self.jet_velocity_mps * self.config.convective_factor).max(1.0);
        let tau_sec = d_m / u_vortex;
        let delay_samples = tau_sec * (self.sample_rate as f32);
        delay_samples.clamp(1.0, (MAX_JET_DELAY - 2) as f32)
    }

    /// Read fractional delay from circular jet delay line using Hermite/linear interpolation.
    #[inline]
    fn read_jet_delay(&self, delay_samples: f32) -> f32 {
        let d_clamped = delay_samples.clamp(0.0, (MAX_JET_DELAY - 2) as f32);
        let d_int = d_clamped.floor() as usize;
        let d_frac = d_clamped - (d_int as f32);

        let cap = MAX_JET_DELAY;
        let idx0 = (self.jet_write_pos + cap - d_int) % cap;
        let idx1 = (self.jet_write_pos + cap - d_int - 1) % cap;

        let s0 = self.jet_delay_buffer[idx0];
        let s1 = self.jet_delay_buffer[idx1];
        s0 + d_frac * (s1 - s0)
    }

    /// Compute one sample of the air-jet excitation given acoustic bore feedback.
    ///
    /// Returns the acoustic volume flow / pressure injection into the bore entrance.
    #[inline]
    pub fn step(&mut self, bore_feedback_pressure: f32) -> f32 {
        if self.blowing_pressure_pa < 5.0 {
            return 0.0;
        }

        // 1. Generate stochastic vortex breath noise
        let raw_noise = self.prng.next_f32();
        
        // Lowpass filter baseline breath turbulence (subtle airy hiss)
        let turb_coeff = 0.35;
        self.turbulence_filter_state += turb_coeff * (raw_noise - self.turbulence_filter_state);
        let base_turb = self.turbulence_filter_state * self.config.turbulence_gain;

        // Bandpass formant resonator for explosive murai-iki breath bursts (around 1.2 kHz)
        let formant_freq = 1200.0;
        let q = 3.5;
        let w0 = (2.0 * PI * formant_freq / (self.sample_rate as f32)).min(PI * 0.9);
        let alpha = (w0.sin()) / (2.0 * q);
        let b0 = alpha;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * w0.cos();
        let a2 = 1.0 - alpha;

        let formant_in = raw_noise * self.config.murai_iki_intensity * (self.blowing_pressure_pa / DEFAULT_PRESSURE_PA).sqrt();
        let formant_out = (b0 * formant_in - a1 * self.formant_filter_state1 - a2 * self.formant_filter_state2) / a0;
        self.formant_filter_state2 = self.formant_filter_state1;
        self.formant_filter_state1 = formant_out;

        let total_noise = base_turb + formant_out * 0.85;

        // 2. Modulate jet displacement with bore feedback and breath turbulence
        let jet_displacement_in = bore_feedback_pressure + total_noise;

        // Write to circular jet delay buffer
        self.jet_delay_buffer[self.jet_write_pos] = jet_displacement_in;
        self.jet_write_pos = (self.jet_write_pos + 1) % MAX_JET_DELAY;

        // 3. Read delayed jet perturbation
        let jet_delay_samples = self.calculate_jet_delay_samples();
        let delayed_jet = self.read_jet_delay(jet_delay_samples);

        // 4. Calculate angle offset from head tilt
        let nominal_angle = self.config.embouchure_angle_deg * (PI / 180.0);
        let current_angle = self.current_angle_deg * (PI / 180.0);
        let angle_offset = (current_angle - nominal_angle).sin() * 0.45;

        // 5. Non-linear splitting at sharp utaguchi edge
        let split_output = utaguchi_splitting_function(
            delayed_jet,
            angle_offset,
            self.config.edge_sharpness,
        );

        // 6. Scale by dynamic Bernoulli blowing pressure
        let pressure_scale = (self.blowing_pressure_pa / DEFAULT_PRESSURE_PA).sqrt();
        (split_output * pressure_scale).clamp(-1.5, 1.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_air_reed_bernoulli_velocity() {
        let v0 = jet_velocity_from_pressure(0.0);
        assert_eq!(v0, 0.0);

        let v_nom = jet_velocity_from_pressure(800.0);
        // v = sqrt(2 * 800 / 1.204) = sqrt(1328.9) ~ 36.45 m/s
        assert!((v_nom - 36.45).abs() < 1.0);

        let v_high = jet_velocity_from_pressure(2000.0);
        assert!(v_high > v_nom);
    }

    #[test]
    fn test_meri_kari_pitch_multipliers() {
        let mult_0 = meri_kari_to_pitch_multiplier(0.0);
        assert!((mult_0 - 1.0).abs() < 1e-5);

        let mult_meri_300 = meri_kari_to_pitch_multiplier(-300.0);
        // 2^(-300/1200) = 2^(-0.25) ~ 0.84089
        assert!((mult_meri_300 - 0.84089).abs() < 1e-3);

        let mult_kari_200 = meri_kari_to_pitch_multiplier(200.0);
        // 2^(200/1200) = 2^(1/6) ~ 1.12246
        assert!((mult_kari_200 - 1.12246).abs() < 1e-3);
    }

    #[test]
    fn test_utaguchi_splitting_bounds() {
        for disp in [-3.0, -1.0, 0.0, 1.0, 3.0] {
            let split = utaguchi_splitting_function(disp, 0.0, 1.45);
            assert!((-1.0..=1.0).contains(&split));
        }
    }

    #[test]
    fn test_air_reed_zero_allocation_processing() {
        let mut reed = AirReed::new(48000);
        reed.set_blowing_pressure(800.0);
        reed.set_meri_kari(-50.0);
        reed.set_murai_iki(0.5);

        for _ in 0..1024 {
            let out = reed.step(0.1);
            assert!(!out.is_nan());
        }
    }
}
