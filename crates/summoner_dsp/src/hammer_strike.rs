// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Non-Linear Plectrum/Hammer Dynamics & Sympathetic Resonance Matrix (Milestone 18).
//!
//! Provides non-linear felt hammer contact dynamics ($F \propto \delta^p$), plectrum snap/release
//! profiles, and an $N$-string sympathetic resonance energy coupling matrix for acoustic plucked
//! and struck strings.
//!
//! Enforces zero heap allocations in audio processing loops under `AllocGuard`.

use serde::{Deserialize, Serialize};

/// Number of strings coupled in the sympathetic resonance matrix.
pub const NUM_SYMPATHETIC_STRINGS: usize = 6;

/// String excitation mechanism types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ExcitationType {
    /// Non-linear felt hammer strike (Piano, Hammered Dulcimer, Rhodes).
    #[default]
    HammerStrike,
    /// Plectrum / pick snap and release (Acoustic Guitar, Harpsichord, Mandolin).
    PlectrumPluck,
    /// Soft fingertip flesh pluck (Classical Nylon Guitar, Harp).
    FingerNylon,
    /// Metallic slap / pop with hard fretboard impact boundary.
    SlapBass,
}

impl ExcitationType {
    /// Returns nominal $(m_h, K_h, p, \beta)$ parameters for this excitation.
    pub fn nominal_parameters(&self) -> (f32, f32, f32, f32) {
        match self {
            Self::HammerStrike => (0.003, 1.2e8, 2.4, 0.12), // Piano hammer: m_h=3g, p=2.4
            Self::PlectrumPluck => (0.0005, 5.0e7, 1.8, 0.15), // Plectrum pick: fast snap
            Self::FingerNylon => (0.0015, 2.0e7, 1.5, 0.20), // Finger flesh: soft contact
            Self::SlapBass => (0.008, 3.0e8, 3.2, 0.08),    // Hard thumb slap: high stiffness
        }
    }
}

/// Physical parameters for the non-linear felt hammer contact model.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HammerParameters {
    /// Equivalent hammer mass in kilograms (typical 0.001 .. 0.015 kg).
    pub mass_kg: f32,
    /// Non-linear felt stiffness coefficient $K_h$ in $\text{N/m}^p$.
    pub stiffness_k: f32,
    /// Felt compression exponent $p$ (typical 1.5 .. 3.5).
    pub exponent_p: f32,
    /// Felt hysteresis damping loss coefficient $\mu_h$ (typical 0.1 .. 0.8).
    pub hysteresis_loss: f32,
    /// Pluck / strike position along the string $\beta = x_{strike} / L \in [0.05, 0.95]$.
    pub strike_position: f32,
}

impl Default for HammerParameters {
    fn default() -> Self {
        let (m, k, p, beta) = ExcitationType::HammerStrike.nominal_parameters();
        Self {
            mass_kg: m,
            stiffness_k: k,
            exponent_p: p,
            hysteresis_loss: 0.35,
            strike_position: beta,
        }
    }
}

/// Instantaneous result of a hammer/plectrum contact evaluation step.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StrikeContactResult {
    /// Instantaneous contact force $F_{contact}$ exerted on the string (Newtons).
    pub contact_force_n: f32,
    /// Instantaneous hammer displacement $y_h$ (meters).
    pub hammer_disp_m: f32,
    /// Instantaneous hammer velocity $v_h$ (m/s).
    pub hammer_vel_mps: f32,
    /// Felt compression $\delta = \max(0, y_h - y_s)$ (meters).
    pub compression_m: f32,
    /// True if hammer/plectrum is currently in physical contact with the string.
    pub is_in_contact: bool,
}

/// Non-linear felt hammer & plectrum dynamic excitation solver.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HammerStrike {
    /// Active excitation type.
    pub excitation_type: ExcitationType,
    /// Physical hammer / plectrum parameters.
    pub params: HammerParameters,
    /// Initial strike velocity in m/s [0.1 ..= 10.0].
    pub strike_velocity: f32,
    /// Plectrum attack angle in radians [-0.5 ..= 0.5].
    pub plectrum_angle_rad: f32,
    /// Current hammer displacement $y_h$ (m).
    pub hammer_disp: f32,
    /// Current hammer velocity $v_h$ (m/s).
    pub hammer_vel: f32,
    /// Previous compression for rate-of-compression $\dot{\delta}$.
    prev_compression: f32,
    /// Contact duration in sample frames.
    pub contact_frames: u32,
    /// Whether hammer is currently active.
    pub is_active: bool,
    /// Audio sample rate.
    pub sample_rate: u32,
}

impl Default for HammerStrike {
    fn default() -> Self {
        Self::new(48000)
    }
}

impl HammerStrike {
    /// Creates a new hammer strike solver initialized for the given sample rate.
    pub fn new(sample_rate: u32) -> Self {
        let sr = sample_rate.max(8000);
        Self {
            excitation_type: ExcitationType::HammerStrike,
            params: HammerParameters::default(),
            strike_velocity: 2.5,
            plectrum_angle_rad: 0.0,
            hammer_disp: 0.0,
            hammer_vel: 0.0,
            prev_compression: 0.0,
            contact_frames: 0,
            is_active: false,
            sample_rate: sr,
        }
    }

    /// Triggers a new strike or pluck with given velocity (0.0 .. 1.0) and optional hardness.
    pub fn trigger(&mut self, velocity: f32, hardness: f32) {
        let vel = velocity.clamp(0.01, 1.0);
        let hard = hardness.clamp(0.0, 1.0);

        // Adjust stiffness and exponent based on hardness
        let (_, base_k, base_p, _) = self.excitation_type.nominal_parameters();
        self.params.stiffness_k = base_k * (0.5 + 1.5 * hard);
        self.params.exponent_p = base_p * (0.85 + 0.30 * hard);

        self.strike_velocity = vel * 5.0; // Map normalized velocity to 0.05 .. 5.0 m/s
        self.hammer_disp = 0.0001; // Initial small displacement
        self.hammer_vel = self.strike_velocity;
        self.prev_compression = 0.0;
        self.contact_frames = 0;
        self.is_active = true;
    }

    /// Sets the excitation type and updates baseline parameters.
    pub fn set_excitation_type(&mut self, excitation: ExcitationType) {
        self.excitation_type = excitation;
        let (m, k, p, beta) = excitation.nominal_parameters();
        self.params.mass_kg = m;
        self.params.stiffness_k = k;
        self.params.exponent_p = p;
        self.params.strike_position = beta;
    }

    /// Evaluates one discrete time step of the non-linear contact interaction.
    ///
    /// # Arguments
    /// - `string_disp_m`: Current displacement of the string at the strike point (meters).
    /// - `string_vel_mps`: Current velocity of the string at the strike point (m/s).
    #[inline]
    pub fn step(&mut self, string_disp_m: f32, _string_vel_mps: f32) -> StrikeContactResult {
        if !self.is_active {
            return StrikeContactResult {
                contact_force_n: 0.0,
                hammer_disp_m: self.hammer_disp,
                hammer_vel_mps: self.hammer_vel,
                compression_m: 0.0,
                is_in_contact: false,
            };
        }

        let dt = 1.0 / (self.sample_rate as f32);

        // Felt compression $\delta = \max(0, y_h - y_s)$
        let raw_compression = self.hammer_disp - string_disp_m;
        let compression = raw_compression.max(0.0);
        let is_contact = compression > 1e-9;

        let contact_force = if is_contact {
            self.contact_frames += 1;
            let delta_dot = (compression - self.prev_compression) / dt;
            self.prev_compression = compression;

            // Non-linear felt contact force: F = K_h * delta^p * (1 + mu_h * delta_dot)
            let base_force = self.params.stiffness_k * compression.powf(self.params.exponent_p);
            let hysteretic_force = base_force * (1.0 + self.params.hysteresis_loss * delta_dot.clamp(-10.0, 10.0));
            hysteretic_force.max(0.0).clamp(0.0, 500.0)
        } else {
            self.prev_compression = 0.0;
            0.0
        };

        // Update hammer kinematics: m_h * a_h = -F_contact (deceleration)
        let accel = -contact_force / self.params.mass_kg.max(1e-5);
        self.hammer_vel += accel * dt;
        self.hammer_disp += self.hammer_vel * dt;

        // Hammer rebounds and separates: contact finished when moving away and delta <= 0
        if self.contact_frames > 2 && self.hammer_disp <= 0.0 && self.hammer_vel <= 0.0 {
            self.is_active = false;
        }

        StrikeContactResult {
            contact_force_n: contact_force,
            hammer_disp_m: self.hammer_disp,
            hammer_vel_mps: self.hammer_vel,
            compression_m: compression,
            is_in_contact: is_contact,
        }
    }

    /// Computes spatial comb filter gain coefficients for pluck position $\beta$.
    #[inline]
    pub fn spatial_comb_coefficients(&self) -> (f32, f32) {
        let beta = self.params.strike_position.clamp(0.05, 0.95);
        let angle_factor = (self.plectrum_angle_rad).cos();
        let amp = (1.0 - (beta - 0.5).abs() * 0.5) * angle_factor;
        (beta, amp)
    }

    /// Resets hammer state.
    pub fn reset(&mut self) {
        self.hammer_disp = 0.0;
        self.hammer_vel = 0.0;
        self.prev_compression = 0.0;
        self.contact_frames = 0;
        self.is_active = false;
    }
}

/// Dynamic $N$-string Sympathetic Resonance Coupling Matrix.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SympatheticResonatorMatrix {
    /// Coupling strength / bleed ratio between strings $[0.0 ..= 0.50]$.
    pub coupling_strength: f32,
    /// Bridge junction mechanical impedance $Z_{bridge}$ (kg/s).
    pub bridge_impedance: f32,
    /// Energy dissipation loss at the bridge $[0.90 ..= 0.999]$.
    pub bridge_loss: f32,
    /// Sympathetic energy accumulator buffer across all $N$ strings.
    pub sympathetic_energy: [f32; NUM_SYMPATHETIC_STRINGS],
    /// Filter states for sympathetic cross-talk lowpass smoothing.
    prev_bridge_velocity: f32,
}

impl Default for SympatheticResonatorMatrix {
    fn default() -> Self {
        Self::new()
    }
}

impl SympatheticResonatorMatrix {
    /// Creates a new sympathetic resonator coupling matrix.
    pub fn new() -> Self {
        Self {
            coupling_strength: 0.15,
            bridge_impedance: 2.5,
            bridge_loss: 0.995,
            sympathetic_energy: [0.0; NUM_SYMPATHETIC_STRINGS],
            prev_bridge_velocity: 0.0,
        }
    }

    /// Sets sympathetic resonance coupling strength $[0.0 ..= 0.50]$.
    pub fn set_coupling_strength(&mut self, strength: f32) {
        self.coupling_strength = strength.clamp(0.0, 0.50);
    }

    /// Solves the multi-string bridge scattering junction for all $N$ strings.
    ///
    /// Energy conservation invariant: $\sum_{i} |v_{out, i}|^2 \le \sum_{i} |v_{in, i}|^2$.
    #[inline]
    pub fn process_junction(
        &mut self,
        string_inputs: &[f32; NUM_SYMPATHETIC_STRINGS],
        string_outputs: &mut [f32; NUM_SYMPATHETIC_STRINGS],
    ) {
        // 1. Calculate common bridge velocity: v_bridge = (2 / Z_total) * sum(Z_i * v_in_i)
        let total_force: f32 = string_inputs.iter().sum();
        let raw_bridge_v = (2.0 * total_force) / (self.bridge_impedance * NUM_SYMPATHETIC_STRINGS as f32 + 1.0);

        // Lowpass bridge smoothing
        let bridge_v = 0.80 * raw_bridge_v + 0.20 * self.prev_bridge_velocity;
        self.prev_bridge_velocity = bridge_v;

        let alpha = self.coupling_strength;
        let loss = self.bridge_loss;

        // 2. Distribute sympathetic cross-coupling to each string
        for i in 0..NUM_SYMPATHETIC_STRINGS {
            let in_val = string_inputs[i];
            // Reflected wave = incoming wave + bridge motion cross-bleed
            let cross_bleed = (bridge_v - in_val) * alpha;
            let out_val = (in_val + cross_bleed) * loss;
            string_outputs[i] = out_val.clamp(-2.0, 2.0);

            // Update energy tracker
            self.sympathetic_energy[i] = 0.99 * self.sympathetic_energy[i] + 0.01 * out_val.abs();
        }
    }

    /// Returns the active sympathetic energy distribution across all $N$ strings.
    pub fn energy_profile(&self) -> [f32; NUM_SYMPATHETIC_STRINGS] {
        self.sympathetic_energy
    }

    /// Resets sympathetic energy buffers.
    pub fn reset(&mut self) {
        self.sympathetic_energy = [0.0; NUM_SYMPATHETIC_STRINGS];
        self.prev_bridge_velocity = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::allocator::AllocGuard;

    #[test]
    fn test_hammer_strike_trigger_and_compression() {
        let mut hammer = HammerStrike::new(48000);
        hammer.trigger(0.8, 0.5);
        assert!(hammer.is_active);

        let mut contact_happened = false;
        let mut max_force = 0.0f32;

        for _ in 0..100 {
            let res = hammer.step(0.0, 0.0);
            if res.is_in_contact {
                contact_happened = true;
                max_force = max_force.max(res.contact_force_n);
            }
        }

        assert!(contact_happened, "Hammer should make contact with string");
        assert!(max_force > 0.1, "Non-zero contact force expected");
    }

    #[test]
    fn test_sympathetic_matrix_energy_conservation() {
        let mut matrix = SympatheticResonatorMatrix::new();
        matrix.set_coupling_strength(0.25);

        let inputs = [1.0f32, 0.0, 0.0, 0.0, 0.0, 0.0];
        let mut outputs = [0.0f32; NUM_SYMPATHETIC_STRINGS];

        let _guard = AllocGuard::new();
        matrix.process_junction(&inputs, &mut outputs);

        let in_energy: f32 = inputs.iter().map(|&x| x * x).sum();
        let out_energy: f32 = outputs.iter().map(|&x| x * x).sum();

        assert!(out_energy <= in_energy * 1.01, "Sympathetic matrix must conserve energy");
        // Open strings should receive sympathetic bleed energy
        assert!(outputs[1].abs() > 0.0, "Neighboring string should receive energy");
    }

    #[test]
    fn test_hammer_and_sympathetic_zero_allocation() {
        let mut hammer = HammerStrike::new(48000);
        hammer.trigger(0.7, 0.5);
        let mut matrix = SympatheticResonatorMatrix::new();

        let inputs = [0.1f32; NUM_SYMPATHETIC_STRINGS];
        let mut outputs = [0.0f32; NUM_SYMPATHETIC_STRINGS];

        let _guard = AllocGuard::new();
        for _ in 0..500 {
            hammer.step(0.001, 0.0);
            matrix.process_junction(&inputs, &mut outputs);
        }
    }
}
