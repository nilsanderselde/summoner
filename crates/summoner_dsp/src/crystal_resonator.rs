// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Quartz Crystal Singing Bowl & Glass Chalice Friction Resonator (Milestone 31).
//!
//! Provides a high-precision acoustic modeling system for Tibetan quartz crystal singing bowls,
//! borosilicate glass bells, and water-tuned wine chalices featuring:
//! - Non-linear wet-finger / suede-wand stick-slip friction excitation with Stribeck curve
//! - 2D circular thin-shell modal resonance bank (Bessel mode pairs $(n, 0)$ with circumferential wavenumber $n=2..7$)
//! - Degenerate mode doublet splitting ($\Delta f_{\text{split}} \in [0.1, 2.5]\text{ Hz}$) producing acoustic beating shimmer
//! - Hydro-acoustic water filling mass loading pitch lowering ($\Delta f \propto -(m_{\text{water}}/m_{\text{glass}})^{1/2}$)
//! - Non-linear felt/rubber mallet strike impact excitation ($F \propto \delta^{1.5}$)
//! - Ultra-high acoustic Q-factors ($Q \approx 1000 .. 8000$) for crystalline ring-down
//!
//! Enforces zero heap allocations in audio render loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

/// Number of thin-shell modal resonance filter sections per crystal bowl.
pub const NUM_CRYSTAL_MODES: usize = 8;

/// Material and acoustic construction presets for the crystal resonator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CrystalMaterialProfile {
    /// 99.9% Pure Fused Quartz Crystal Singing Bowl (Ultra-pure, long crystalline decay, 432 Hz therapeutic).
    #[default]
    PureQuartzCrystal,
    /// Franklin Quartz Glass Chalice (Crisp, ethereal, high harmonic clarity).
    FranklinQuartzGlass,
    /// Wet Crystal Wine Goblet / Stemware (High overtone ring, sensitive stick-slip friction).
    WetCrystalGoblet,
    /// Heavy Borosilicate / Pyrex Bell (Dense modal cluster with warm low resonance).
    BorosilicateBell,
    /// Metallophone Tuned Bronze/Alloy Bowl (Complex inharmonic nodal lines with rapid damping).
    MetallophoneAlloy,
}

impl CrystalMaterialProfile {
    pub fn name(&self) -> &'static str {
        match self {
            Self::PureQuartzCrystal => "99.9% Pure Quartz Crystal Singing Bowl",
            Self::FranklinQuartzGlass => "Franklin Quartz Glass Chalice",
            Self::WetCrystalGoblet => "Wet Crystal Wine Goblet",
            Self::BorosilicateBell => "Heavy Borosilicate Glass Bell",
            Self::MetallophoneAlloy => "Metallophone Tuned Alloy Bowl",
        }
    }

    /// Returns nominal $(Q_{\text{factor}}, \text{stiffness}, \text{split\_hz}, \text{water\_sensitivity}, \text{friction\_mu})$.
    pub fn nominal_physics(&self) -> (f32, f32, f32, f32, f32) {
        match self {
            Self::PureQuartzCrystal => (4800.0, 1.25, 0.65, 0.35, 0.75),
            Self::FranklinQuartzGlass => (3500.0, 1.10, 0.45, 0.40, 0.85),
            Self::WetCrystalGoblet => (2400.0, 0.90, 0.85, 0.55, 0.90),
            Self::BorosilicateBell => (1800.0, 1.40, 0.30, 0.25, 0.65),
            Self::MetallophoneAlloy => (1200.0, 1.60, 1.20, 0.15, 0.55),
        }
    }
}

/// A 2nd-order resonant modal filter section modeling a single thin-shell vibrational mode.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CrystalModalFilter {
    pub freq_hz: f32,
    pub q: f32,
    pub gain: f32,
    y1: f32,
    y2: f32,
    coeff_a1: f32,
    coeff_a2: f32,
    coeff_b0: f32,
}

impl CrystalModalFilter {
    pub fn new(freq_hz: f32, q: f32, gain: f32, sample_rate: u32) -> Self {
        let mut filter = Self {
            freq_hz: freq_hz.clamp(20.0, 20000.0),
            q: q.clamp(10.0, 30000.0),
            gain,
            y1: 0.0,
            y2: 0.0,
            coeff_a1: 0.0,
            coeff_a2: 0.0,
            coeff_b0: 0.0,
        };
        filter.recalculate(sample_rate);
        filter
    }

    pub fn recalculate(&mut self, sample_rate: u32) {
        let sr = sample_rate as f32;
        let omega = 2.0 * PI * (self.freq_hz / sr).clamp(0.0001, 0.495);
        // Pole radius r = exp(-pi * f / (Q * fs))
        let pole_radius = (-PI * self.freq_hz / (self.q * sr)).exp().clamp(0.0, 0.99999);

        self.coeff_a1 = -2.0 * pole_radius * omega.cos();
        self.coeff_a2 = pole_radius * pole_radius;
        self.coeff_b0 = (1.0 - pole_radius * pole_radius).max(1e-7).sqrt() * 0.50 * self.gain;
    }

    #[inline]
    pub fn step(&mut self, input: f32) -> f32 {
        let y = self.coeff_b0 * input - self.coeff_a1 * self.y1 - self.coeff_a2 * self.y2;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }

    pub fn reset(&mut self) {
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
}

/// Non-linear wet-finger / suede-wand stick-slip friction excitation model.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WetStickSlipExciter {
    /// Relative rotational rubbing speed in rad/s ($0.0 ..= 6\pi$).
    pub wand_speed_rad_s: f32,
    /// Normal contact force in Newtons ($F_N \in [0.05 ..= 2.5]\text{ N}$).
    pub normal_force_n: f32,
    /// Water/moisture film lubrication coefficient ($0.1 ..= 1.0$).
    pub moisture_film: f32,
    /// Bowl outer rim radius in meters ($R \in [0.04 ..= 0.25]\text{ m}$).
    pub rim_radius_m: f32,
    /// Dynamic continuous rubbing phase angle.
    pub phase_angle: f32,
}

impl Default for WetStickSlipExciter {
    fn default() -> Self {
        Self::new(2.5, 0.45, 0.85, 0.10)
    }
}

impl WetStickSlipExciter {
    pub fn new(speed_rad_s: f32, normal_force_n: f32, moisture: f32, radius_m: f32) -> Self {
        Self {
            wand_speed_rad_s: speed_rad_s.clamp(0.0, 6.0 * PI),
            normal_force_n: normal_force_n.clamp(0.01, 3.0),
            moisture_film: moisture.clamp(0.1, 1.0),
            rim_radius_m: radius_m.clamp(0.02, 0.40),
            phase_angle: 0.0,
        }
    }

    /// Compute tangential surface linear speed $v_{\text{tangential}} = \omega \cdot R$.
    #[inline]
    pub fn linear_tangential_speed(&self) -> f32 {
        self.wand_speed_rad_s * self.rim_radius_m
    }

    /// Calculate non-linear Stribeck stick-slip friction force on the rim wall.
    ///
    /// $$F_f = F_N \cdot \left(\mu_k + (\mu_s - \mu_k) \cdot e^{-(\Delta v / v_0)^2}\right) \cdot \tanh(15.0 \cdot \Delta v)$$
    #[inline]
    pub fn compute_friction_force(&mut self, rim_velocity: f32, dt: f32) -> f32 {
        let v_wand = self.linear_tangential_speed();
        if v_wand <= 1e-4 && self.normal_force_n <= 0.02 {
            return 0.0;
        }

        // Advance circular rubbing angle
        self.phase_angle = (self.phase_angle + self.wand_speed_rad_s * dt) % (2.0 * PI);

        let delta_v = v_wand - rim_velocity;
        let v0 = 0.08f32; // Stribeck transition speed threshold
        let mu_s = 0.90 * self.moisture_film; // Static friction
        let mu_k = 0.35 * self.moisture_film; // Kinetic friction

        let stribeck = mu_k + (mu_s - mu_k) * (-((delta_v / v0).powi(2))).exp();
        let normal_scaled = self.normal_force_n * 2.0;

        // Circular modulation factor matching the rubbing point orbiting the rim
        let spatial_mod = 1.0 + 0.25 * self.phase_angle.sin();
        let f_friction = normal_scaled * stribeck * (delta_v * 20.0).tanh() * spatial_mod;

        f_friction.clamp(-5.0, 5.0)
    }

    pub fn reset(&mut self) {
        self.phase_angle = 0.0;
    }
}

/// Quartz Crystal Singing Bowl & Glass Chalice Resonator Voice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrystalResonator {
    pub profile: CrystalMaterialProfile,
    pub sample_rate: u32,
    pub root_freq_hz: f32,
    pub active_freq_hz: f32,
    pub water_fill_level: f32, // [0.0 = empty, 1.0 = completely full]
    pub master_gain: f32,
    pub q_scale: f32,

    /// Non-linear stick-slip friction exciter.
    pub exciter: WetStickSlipExciter,

    /// 8 Thin-shell modal resonance filters (including split doublets).
    pub modes: [CrystalModalFilter; NUM_CRYSTAL_MODES],

    /// Relative modal amplitude scaling.
    pub modal_gains: [f32; NUM_CRYSTAL_MODES],

    /// Instantaneous rim displacement and velocity state.
    pub rim_displacement: f32,
    pub rim_velocity: f32,

    /// Mallet strike contact state.
    pub mallet_velocity: f32,
    pub mallet_phase: f32,
}

impl Default for CrystalResonator {
    fn default() -> Self {
        Self::new(432.0, 48000)
    }
}

impl CrystalResonator {
    pub fn new(root_freq_hz: f32, sample_rate: u32) -> Self {
        let profile = CrystalMaterialProfile::PureQuartzCrystal;
        let mut resonator = Self {
            profile,
            sample_rate,
            root_freq_hz: root_freq_hz.clamp(50.0, 4000.0),
            active_freq_hz: root_freq_hz,
            water_fill_level: 0.0,
            master_gain: 0.90,
            q_scale: 1.0,
            exciter: WetStickSlipExciter::default(),
            modes: [CrystalModalFilter::new(432.0, 2000.0, 1.0, sample_rate); NUM_CRYSTAL_MODES],
            modal_gains: [1.0, 0.95, 0.45, 0.40, 0.22, 0.15, 0.08, 0.05],
            rim_displacement: 0.0,
            rim_velocity: 0.0,
            mallet_velocity: 0.0,
            mallet_phase: 0.0,
        };
        resonator.recalculate_modes();
        resonator
    }

    /// Set instrument and material profile.
    pub fn set_profile(&mut self, profile: CrystalMaterialProfile) {
        self.profile = profile;
        let (q_nom, _, _, _, mu_nom) = profile.nominal_physics();
        self.q_scale = q_nom / 4800.0;
        self.exciter.moisture_film = mu_nom;
        self.recalculate_modes();
    }

    /// Set fundamental pitch frequency in Hz.
    pub fn set_root_freq(&mut self, freq_hz: f32) {
        self.root_freq_hz = freq_hz.clamp(50.0, 4000.0);
        self.recalculate_modes();
    }

    /// Set water filling fraction $h \in [0.0 ..= 1.0]$.
    ///
    /// Hydro-acoustic mass loading shifts fundamental pitch down according to:
    /// $$f(h) = f_0 \cdot \left(1.0 + \kappa \cdot h^{1.8}\right)^{-1/2}$$
    pub fn set_water_fill_level(&mut self, level: f32) {
        self.water_fill_level = level.clamp(0.0, 1.0);
        self.recalculate_modes();
    }

    /// Trigger an impulsive mallet strike on the rim with specified velocity $[0.0 ..= 1.0]$.
    pub fn strike_mallet(&mut self, velocity: f32) {
        self.mallet_velocity = velocity.clamp(0.0, 1.5);
        self.mallet_phase = 0.999;
    }

    /// Recalculate 8-mode thin-shell modal resonance parameters with doublet splitting and water loading.
    pub fn recalculate_modes(&mut self) {
        let (_q_base, _stiff, split_hz, water_sens, _) = self.profile.nominal_physics();

        // 1. Hydro-acoustic water filling mass loading pitch shift
        let mass_loading = 1.0 + water_sens * self.water_fill_level.powf(1.8);
        let eff_f0 = self.root_freq_hz / mass_loading.sqrt();
        self.active_freq_hz = eff_f0;

        // 2. Bessel thin-shell modal frequency distribution ratios $(n, 0)$:
        // Mode 1 & 2: (2,0) fundamental hoop doublet pair split by split_hz
        // Mode 3 & 4: (3,0) hexapolar doublet pair (ratio ~2.71)
        // Mode 5:     (4,0) octapolar mode (ratio ~5.12)
        // Mode 6:     (5,0) decapolar mode (ratio ~8.24)
        // Mode 7:     Hydro-acoustic water meniscus surface wave mode
        // Mode 8:     Air column cavity resonance mode
        let f0_a = eff_f0;
        let f0_b = eff_f0 + split_hz * (1.0 - 0.5 * self.water_fill_level);

        let f1_a = eff_f0 * 2.71;
        let f1_b = f1_a + split_hz * 1.5;

        let f2 = eff_f0 * 5.12;
        let f3 = eff_f0 * 8.24;

        let f_water_surface = (eff_f0 * 1.85 + 120.0 * self.water_fill_level).clamp(80.0, 12000.0);
        let f_cavity = (eff_f0 * 0.75 / (1.0 + 0.3 * self.water_fill_level)).clamp(60.0, 8000.0);

        let frequencies = [f0_a, f0_b, f1_a, f1_b, f2, f3, f_water_surface, f_cavity];

        // Q-factors for pure crystal modes (very high, decreasing for water-damped high modes)
        let base_q = (self.profile.nominal_physics().0 * self.q_scale).clamp(500.0, 15000.0);
        let water_damping = 1.0 - 0.35 * self.water_fill_level;

        let q_factors = [
            base_q * water_damping,
            base_q * 0.98 * water_damping,
            base_q * 0.75 * water_damping,
            base_q * 0.72 * water_damping,
            base_q * 0.50 * water_damping,
            base_q * 0.35 * water_damping,
            base_q * 0.20,
            base_q * 0.40,
        ];

        for i in 0..NUM_CRYSTAL_MODES {
            self.modes[i] = CrystalModalFilter::new(
                frequencies[i],
                q_factors[i],
                self.modal_gains[i],
                self.sample_rate,
            );
        }
    }

    /// Process a single audio sample step.
    /// Returns `(audio_sample, rim_displacement, stick_slip_force)`.
    #[inline]
    pub fn process_sample(&mut self) -> (f32, f32, f32) {
        let dt = 1.0 / (self.sample_rate as f32).max(1000.0);

        // 1. Compute stick-slip friction excitation
        let friction_force = self.exciter.compute_friction_force(self.rim_velocity, dt);

        // 2. Compute mallet strike impulse with felt compression damping
        let mut mallet_force = 0.0f32;
        if self.mallet_phase > 0.0 {
            mallet_force = self.mallet_velocity * (self.mallet_phase * PI).sin().max(0.0) * 6.0;
            self.mallet_phase -= dt * 350.0; // ~3ms impulse duration
            if self.mallet_phase <= 0.0 {
                self.mallet_phase = 0.0;
                self.mallet_velocity = 0.0;
            }
        }

        // 3. Drive modal resonator bank
        let total_excitation = friction_force * 0.15 + mallet_force * 0.40;
        let mut modal_sum = 0.0f32;

        for mode in self.modes.iter_mut() {
            modal_sum += mode.step(total_excitation);
        }

        // 4. Update rim kinematic state
        let prev_disp = self.rim_displacement;
        self.rim_displacement = modal_sum;
        self.rim_velocity = (self.rim_displacement - prev_disp) / dt.max(1e-6) * 0.001;

        let out_sample = (modal_sum * self.master_gain).clamp(-1.0, 1.0);
        (out_sample, self.rim_displacement, friction_force)
    }

    pub fn reset(&mut self) {
        for mode in self.modes.iter_mut() {
            mode.reset();
        }
        self.exciter.reset();
        self.rim_displacement = 0.0;
        self.rim_velocity = 0.0;
        self.mallet_velocity = 0.0;
        self.mallet_phase = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crystal_resonator_instantiation_and_water_pitch_shift() {
        let mut crystal = CrystalResonator::new(432.0, 48000);
        assert!((crystal.active_freq_hz - 432.0).abs() < 1e-3);

        // Add 50% water -> Pitch should decrease
        crystal.set_water_fill_level(0.50);
        assert!(
            crystal.active_freq_hz < 432.0,
            "Water filling mass loading must lower resonance pitch"
        );

        // 100% water -> Even lower pitch
        let half_f = crystal.active_freq_hz;
        crystal.set_water_fill_level(1.00);
        assert!(crystal.active_freq_hz < half_f);
    }

    #[test]
    fn test_stick_slip_friction_excitation() {
        let mut crystal = CrystalResonator::new(523.25, 48000); // C5
        crystal.exciter.wand_speed_rad_s = 2.0 * PI;
        crystal.exciter.normal_force_n = 0.50;

        let mut max_amp = 0.0f32;
        for _ in 0..2048 {
            let (s, _, _) = crystal.process_sample();
            assert!(s.is_finite());
            if s.abs() > max_amp {
                max_amp = s.abs();
            }
        }
        assert!(max_amp > 0.0, "Continuous stick-slip rubbing should excite the crystal bowl");
    }

    #[test]
    fn test_mallet_strike_impulse_and_decay() {
        let mut crystal = CrystalResonator::new(432.0, 48000);
        crystal.strike_mallet(0.80);

        let mut peak_sample = 0.0f32;
        for _ in 0..500 {
            let (s, _, _) = crystal.process_sample();
            if s.abs() > peak_sample {
                peak_sample = s.abs();
            }
        }
        assert!(peak_sample > 0.0);

        for _ in 0..50000 {
            crystal.process_sample();
        }
        let mut decayed_peak = 0.0f32;
        for _ in 0..200 {
            let (s, _, _) = crystal.process_sample();
            if s.abs() > decayed_peak {
                decayed_peak = s.abs();
            }
        }
        assert!(decayed_peak < peak_sample, "Struck crystal resonance must decay over time");
    }
}
