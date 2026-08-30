// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Bowed String Non-Linear Stick-Slip Friction Modeler (Milestone 16).
//!
//! Provides the non-linear hyperbolic friction interaction model between bow hair (rosin)
//! and vibrating digital waveguide strings, based on the McIntyre-Schumacher-Woodhouse (1983)
//! and Smith (1986) physical formulation.
//!
//! Features:
//! - Static ($\mu_s$) and dynamic ($\mu_d$) friction coefficients with rosin adhesion.
//! - Exact closed-form quadratic scattering junction solver for slip transitions.
//! - Rosin temperature & thermal hysteresis tracking.
//! - Enforces zero heap allocations in audio processing loops under `AllocGuard`.

use serde::{Deserialize, Serialize};

/// Standard Rosin material types with distinct stick-slip adhesive profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RosinType {
    /// Light violin rosin: crisp attack, rapid slip transition, bright overtone excitation.
    #[default]
    LightViolin,
    /// Medium cello/viola rosin: balanced adhesive grip and smooth Helmholtz resonance.
    MediumCello,
    /// Dark heavy bass rosin: sticky high static friction, deep fundamental bite.
    DarkDoubleBass,
    /// Synthetic low-dust rosin: clean linear sliding response.
    Synthetic,
}

impl RosinType {
    /// Returns nominal $(\mu_s, \mu_d, \alpha)$ friction parameters.
    pub fn nominal_parameters(&self) -> (f32, f32, f32) {
        match self {
            Self::LightViolin => (0.95, 0.32, 3.5),
            Self::MediumCello => (1.10, 0.40, 2.8),
            Self::DarkDoubleBass => (1.35, 0.48, 2.0),
            Self::Synthetic => (0.85, 0.28, 4.2),
        }
    }
}

/// Physical parameters defining the non-linear bow-string friction contact.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FrictionParameters {
    /// Static friction coefficient $\mu_s$ (typical 0.7 .. 1.5).
    pub mu_static: f32,
    /// Dynamic sliding friction coefficient $\mu_d$ (typical 0.2 .. 0.5).
    pub mu_dynamic: f32,
    /// Hyperbolic velocity slope parameter $\alpha$ (typical 1.0 .. 6.0).
    pub velocity_slope: f32,
    /// Characteristic string mechanical wave impedance $Z_0$ (typical 0.2 .. 1.5 kg/s).
    pub string_impedance: f32,
    /// Thermal softening sensitivity coefficient.
    pub thermal_sensitivity: f32,
}

impl Default for FrictionParameters {
    fn default() -> Self {
        let (mu_s, mu_d, alpha) = RosinType::LightViolin.nominal_parameters();
        Self {
            mu_static: mu_s,
            mu_dynamic: mu_d,
            velocity_slope: alpha,
            string_impedance: 0.50,
            thermal_sensitivity: 0.12,
        }
    }
}

/// Result of evaluating the bow-string friction junction for one sample.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrictionResult {
    /// Instantaneous string velocity at the bowing point $v_{string}$.
    pub string_velocity: f32,
    /// Reflected outgoing wave toward nut $v_{nut}^-$.
    pub out_nut: f32,
    /// Reflected outgoing wave toward bridge $v_{bridge}^-$.
    pub out_bridge: f32,
    /// Friction force exerted by bow on string $F_{friction}$.
    pub friction_force: f32,
    /// Relative sliding velocity $v_{rel} = v_{string} - v_{bow}$.
    pub rel_velocity: f32,
    /// True if string is currently adhering to the bow hair (sticking phase).
    pub is_sticking: bool,
}

/// Non-linear stick-slip friction solver state with thermal hysteresis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrictionModel {
    /// Rosin configuration preset.
    pub rosin_type: RosinType,
    /// Physical friction parameters.
    pub params: FrictionParameters,
    /// Dynamic rosin temperature state $[0.0 ..= 1.0]$.
    pub rosin_temperature: f32,
    /// Dynamic hysteresis adhesion memory $[0.0 ..= 1.0]$.
    pub hysteresis_adhesion: f32,
    /// Audio sample rate.
    pub sample_rate: u32,
}

impl Default for FrictionModel {
    fn default() -> Self {
        Self::new(48000)
    }
}

impl FrictionModel {
    /// Creates a new stick-slip friction model with default violin rosin.
    pub fn new(sample_rate: u32) -> Self {
        let mut model = Self {
            rosin_type: RosinType::LightViolin,
            params: FrictionParameters::default(),
            rosin_temperature: 0.0,
            hysteresis_adhesion: 1.0,
            sample_rate: sample_rate.max(8000),
        };
        model.set_rosin(RosinType::LightViolin);
        model
    }

    /// Configures rosin type and updates baseline static/dynamic coefficients.
    pub fn set_rosin(&mut self, rosin: RosinType) {
        self.rosin_type = rosin;
        let (mu_s, mu_d, alpha) = rosin.nominal_parameters();
        self.params.mu_static = mu_s;
        self.params.mu_dynamic = mu_d;
        self.params.velocity_slope = alpha;
    }

    /// Evaluates the friction curve coefficient $\mu(v_{rel})$ for a given relative velocity.
    #[inline]
    pub fn friction_coefficient(&self, v_rel: f32) -> f32 {
        let abs_v = v_rel.abs();
        let mu_s_eff = self.effective_mu_static();
        let mu_d = self.params.mu_dynamic;
        let alpha = self.params.velocity_slope;

        v_rel.signum() * (mu_d + (mu_s_eff - mu_d) / (1.0 + alpha * abs_v))
    }

    /// Returns the effective static friction coefficient taking thermal softening into account.
    #[inline]
    pub fn effective_mu_static(&self) -> f32 {
        let thermal_drop = self.rosin_temperature * self.params.thermal_sensitivity;
        (self.params.mu_static * (1.0 - thermal_drop)).max(self.params.mu_dynamic + 0.05)
    }

    /// Solves the non-linear digital waveguide bow-string scattering junction.
    ///
    /// # Arguments
    /// - `in_nut`: Incoming wave velocity from nut section ($v_{nut}^+$)
    /// - `in_bridge`: Incoming wave velocity from bridge section ($v_{bridge}^+$)
    /// - `v_bow`: Transverse bow velocity (m/s)
    /// - `f_normal`: Normal clamping bow force ($F_N$ in Newtons, $\ge 0$)
    #[inline]
    pub fn solve(
        &mut self,
        in_nut: f32,
        in_bridge: f32,
        v_bow: f32,
        f_normal: f32,
    ) -> FrictionResult {
        let z2 = 2.0 * self.params.string_impedance; // 2 * Z_0
        let v_h = in_nut + in_bridge; // Unperturbed wave velocity
        let v_diff = v_h - v_bow;
        let mu_s = self.effective_mu_static();
        let mu_d = self.params.mu_dynamic;
        let alpha = self.params.velocity_slope;

        let fn_clamped = f_normal.max(0.0);
        let f_stick_max = fn_clamped * mu_s;
        let f_req = z2 * v_diff;

        let (v_string, v_rel, friction_force, is_sticking) = if f_req.abs() <= f_stick_max || fn_clamped < 1e-6 {
            // --- STICK STATE ---
            // The string is locked to the bow hair velocity.
            let v_str = if fn_clamped < 1e-6 { v_h } else { v_bow };
            let f_fric = z2 * (v_h - v_str);
            (v_str, 0.0f32, f_fric, true)
        } else {
            // --- SLIP STATE ---
            // Solve 2*Z_0*(v_rel - v_diff) + F_N * sign(v_rel) * (mu_d + (mu_s - mu_d)/(1 + alpha*|v_rel|)) = 0
            let sign_d = if v_diff >= 0.0 { 1.0f32 } else { -1.0f32 };
            let d_mag = v_diff.abs();

            let a_coeff = z2 * alpha;
            let b_coeff = z2 - z2 * alpha * d_mag + fn_clamped * mu_d * alpha;
            let c_coeff = fn_clamped * mu_s - z2 * d_mag; // Strictly negative because |f_req| > f_stick_max

            let disc = (b_coeff * b_coeff - 4.0 * a_coeff * c_coeff).max(0.0);
            let u = (-b_coeff + disc.sqrt()) / (2.0 * a_coeff);
            let u_clamped = u.max(0.0);

            let rel_v = sign_d * u_clamped;
            let v_str = v_bow + rel_v;
            let f_fric = z2 * (v_h - v_str);
            (v_str, rel_v, f_fric, false)
        };

        // Update thermal state: friction power dissipation heats rosin; ambient cools it.
        let power = (friction_force * v_rel).abs();
        let dt = 1.0 / (self.sample_rate as f32);
        let heat_rate = 0.08 * power;
        let cool_rate = 1.5 * self.rosin_temperature;
        self.rosin_temperature = (self.rosin_temperature + (heat_rate - cool_rate) * dt).clamp(0.0, 1.0);

        // Update hysteresis adhesion memory
        let target_adhesion = if is_sticking { 1.0 } else { 0.35 };
        self.hysteresis_adhesion += (target_adhesion - self.hysteresis_adhesion) * (dt * 15.0);

        // Scattering wave reflections
        let out_nut = v_string - in_bridge;
        let out_bridge = v_string - in_nut;

        FrictionResult {
            string_velocity: v_string,
            out_nut,
            out_bridge,
            friction_force,
            rel_velocity: v_rel,
            is_sticking,
        }
    }

    /// Resets thermal and hysteresis states.
    pub fn reset(&mut self) {
        self.rosin_temperature = 0.0;
        self.hysteresis_adhesion = 1.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::allocator::AllocGuard;

    #[test]
    fn test_friction_model_stick_state() {
        let mut model = FrictionModel::new(48000);
        // Small difference within sticking threshold: f_req = 2 * 0.5 * 0.05 = 0.05 <= 2.0 * 0.95 = 1.90
        let res = model.solve(0.10, 0.15, 0.20, 2.0);
        assert!(res.is_sticking);
        assert!((res.string_velocity - 0.20).abs() < 1e-4);
        assert_eq!(res.rel_velocity, 0.0);
    }

    #[test]
    fn test_friction_model_slip_state() {
        let mut model = FrictionModel::new(48000);
        // Large difference causing slip: v_diff = 2.0, f_req = 2.0 > f_stick_max = 0.5 * 0.95 = 0.475
        let res = model.solve(1.0, 1.5, 0.5, 0.5);
        assert!(!res.is_sticking);
        assert!(res.rel_velocity.abs() > 0.0);
        assert!((res.string_velocity - (0.5 + res.rel_velocity)).abs() < 1e-4);
    }

    #[test]
    fn test_friction_model_zero_allocation_in_loop() {
        let mut model = FrictionModel::new(48000);
        let _guard = AllocGuard::new();

        for i in 0..1000 {
            let t = i as f32 / 48000.0;
            let v_bow = 0.3 + 0.1 * (2.0 * std::f32::consts::PI * 5.0 * t).sin();
            let res = model.solve(0.05, -0.04, v_bow, 1.2);
            assert!(res.string_velocity.is_finite());
            assert!(res.friction_force.is_finite());
        }
    }
}
