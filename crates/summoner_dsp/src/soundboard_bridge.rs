// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Concert Grand Piano Bi-Directional Soundboard Bridge Coupling & Felt Hammer (Milestone 26).
//!
//! Provides an anisotropic spruce soundboard 2D modal matrix resonator, a multi-port
//! bi-directional wave scattering bridge impedance coupler, and a non-linear power-law
//! felt hammer contact solver ($F \propto \delta^p$) with Una Corda physical shift modeling.
//!
//! Enforces zero heap allocations in audio render loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

/// Maximum number of 2D modal plate resonance modes in the spruce soundboard.
pub const NUM_SOUNDBOARD_MODES: usize = 8;

/// Maximum number of coupled string unisons per note (1 for bass monochord, 2 for bichord, 3 for trichord).
pub const MAX_UNISONS_PER_NOTE: usize = 3;

/// Nominal spruce wood density in kg/m^3.
pub const SPRUCE_DENSITY_KG_M3: f32 = 430.0;

/// Anisotropic longitudinal-to-transverse sound velocity ratio in Sitka spruce ($c_\parallel / c_\perp \approx 4.0$).
pub const SPRUCE_ANISOTROPY_RATIO: f32 = 4.0;

/// Concert grand piano soundboard preset profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SoundboardProfile {
    /// 9-foot Steinway D Concert Grand (expansive dynamic range, balanced prompt/aftersound).
    #[default]
    SteinwayD9Foot,
    /// 9.5-foot Bösendorfer Imperial 290 (deep sub-bass resonance, warm sustain, dark modal envelope).
    BösendorferImperial,
    /// 9-foot Yamaha CFX (bright, crisp prompt attack, crystalline treble radiation).
    YamahaCFX,
    /// Intimate Warm Studio Grand (felted damping, close mic dispersion, mellow body).
    IntimateStudio,
    /// Impressionist Una Corda (delicate ethereal shimmer, soft pedal character).
    ImpressionistUnaCorda,
    /// Prepared Avant-Garde (muted string junctions, metallic boundary reflections).
    PreparedAvantGarde,
}

impl SoundboardProfile {
    pub fn name(&self) -> &'static str {
        match self {
            Self::SteinwayD9Foot => "Steinway Model D (9ft Concert Grand)",
            Self::BösendorferImperial => "Bösendorfer Imperial 290 (9.5ft Grand)",
            Self::YamahaCFX => "Yamaha CFX Concert Grand",
            Self::IntimateStudio => "Intimate Warm Studio Grand",
            Self::ImpressionistUnaCorda => "Impressionist Una Corda Grand",
            Self::PreparedAvantGarde => "Prepared Avant-Garde Grand",
        }
    }

    /// Returns nominal $(T_{60}\text{ decay scale}, \text{bridge impedance } Z_b, \text{sympathetic bleed}, \text{stiffness exponent } p)$.
    pub fn nominal_physics(&self) -> (f32, f32, f32, f32) {
        match self {
            Self::SteinwayD9Foot => (1.0, 420.0, 0.35, 2.4),
            Self::BösendorferImperial => (1.35, 380.0, 0.45, 2.3),
            Self::YamahaCFX => (0.85, 480.0, 0.28, 2.6),
            Self::IntimateStudio => (0.75, 520.0, 0.20, 2.2),
            Self::ImpressionistUnaCorda => (1.20, 360.0, 0.50, 2.2),
            Self::PreparedAvantGarde => (0.40, 600.0, 0.15, 2.8),
        }
    }

    /// Returns modal resonance frequencies in Hz $(F_0 .. F_7)$.
    pub fn modal_frequencies_hz(&self) -> [f32; NUM_SOUNDBOARD_MODES] {
        match self {
            Self::SteinwayD9Foot => [62.0, 125.0, 240.0, 380.0, 680.0, 1200.0, 2100.0, 3300.0],
            Self::BösendorferImperial => [52.0, 108.0, 210.0, 340.0, 610.0, 1080.0, 1950.0, 3050.0],
            Self::YamahaCFX => [68.0, 138.0, 265.0, 420.0, 740.0, 1320.0, 2350.0, 3600.0],
            Self::IntimateStudio => [58.0, 118.0, 225.0, 360.0, 640.0, 1150.0, 1980.0, 2900.0],
            Self::ImpressionistUnaCorda => [60.0, 120.0, 235.0, 370.0, 660.0, 1180.0, 2050.0, 3200.0],
            Self::PreparedAvantGarde => [75.0, 155.0, 290.0, 480.0, 820.0, 1480.0, 2600.0, 4100.0],
        }
    }

    /// Returns modal quality factors $Q_k$ $(Q_0 .. Q_7)$.
    pub fn modal_q_factors(&self) -> [f32; NUM_SOUNDBOARD_MODES] {
        match self {
            Self::SteinwayD9Foot => [15.0, 22.0, 28.0, 32.0, 38.0, 45.0, 50.0, 55.0],
            Self::BösendorferImperial => [18.0, 26.0, 34.0, 40.0, 48.0, 55.0, 60.0, 65.0],
            Self::YamahaCFX => [12.0, 18.0, 24.0, 28.0, 34.0, 40.0, 46.0, 50.0],
            Self::IntimateStudio => [10.0, 15.0, 20.0, 24.0, 28.0, 32.0, 36.0, 40.0],
            Self::ImpressionistUnaCorda => [20.0, 30.0, 38.0, 44.0, 52.0, 60.0, 68.0, 75.0],
            Self::PreparedAvantGarde => [6.0, 8.0, 12.0, 14.0, 16.0, 18.0, 22.0, 25.0],
        }
    }

    /// Returns modal stereo radiation gains & panning $(g_k, \text{pan}_k)$.
    pub fn modal_gains_and_pans(&self) -> [(f32, f32); NUM_SOUNDBOARD_MODES] {
        [
            (0.35, -0.10), // (1,1) Breathing mode - slight left
            (0.40, -0.25), // (1,2) Longitudinal mode - mid left
            (0.45, 0.20),  // (2,1) Transverse mode - mid right
            (0.50, -0.15), // (2,2) Torsional mode - center-left
            (0.42, 0.30),  // Inner bridge flexure - right
            (0.38, 0.10),  // Mid-treble rib - center-right
            (0.30, -0.05), // Bridge-hill 1 - center
            (0.25, 0.15),  // Bridge-hill 2 - slight right
        ]
    }
}

/// A 2nd-order Direct Form II biquad resonator for soundboard modal vibration.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpruceSoundboardMode {
    pub freq_hz: f32,
    pub q: f32,
    pub gain: f32,
    pub pan: f32,
    // Biquad filter coefficients
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    // State memory
    w1: f32,
    w2: f32,
}

impl Default for SpruceSoundboardMode {
    fn default() -> Self {
        Self {
            freq_hz: 100.0,
            q: 20.0,
            gain: 0.3,
            pan: 0.0,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            w1: 0.0,
            w2: 0.0,
        }
    }
}

impl SpruceSoundboardMode {
    pub fn new(sample_rate: u32, freq_hz: f32, q: f32, gain: f32, pan: f32) -> Self {
        let mut mode = Self {
            freq_hz,
            q: q.max(0.5),
            gain,
            pan: pan.clamp(-1.0, 1.0),
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            w1: 0.0,
            w2: 0.0,
        };
        mode.update_coefficients(sample_rate);
        mode
    }

    pub fn update_coefficients(&mut self, sample_rate: u32) {
        let sr = sample_rate.max(8000) as f32;
        let omega = (2.0 * PI * self.freq_hz.clamp(10.0, sr * 0.48) / sr).min(PI * 0.96);
        let alpha = omega.sin() / (2.0 * self.q.max(0.5));
        let cos_w = omega.cos();

        // Resonant bandpass filter with constant peak gain
        let b0 = alpha * self.gain;
        let b1 = 0.0;
        let b2 = -alpha * self.gain;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_w;
        let a2 = 1.0 - alpha;

        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }

    #[inline]
    pub fn step(&mut self, input: f32) -> (f32, f32) {
        let w0 = input - self.a1 * self.w1 - self.a2 * self.w2;
        let y = self.b0 * w0 + self.b1 * self.w1 + self.b2 * self.w2;
        self.w2 = self.w1;
        self.w1 = w0;

        // Apply constant-power stereo pan
        let pan_norm = (self.pan + 1.0) * 0.5; // [0.0 (left) .. 1.0 (right)]
        let gain_l = (1.0 - pan_norm).sqrt();
        let gain_r = pan_norm.sqrt();
        (y * gain_l, y * gain_r)
    }

    pub fn reset(&mut self) {
        self.w1 = 0.0;
        self.w2 = 0.0;
    }
}

/// Anisotropic 2D Spruce Soundboard Modal Bank.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpruceSoundboard {
    pub modes: [SpruceSoundboardMode; NUM_SOUNDBOARD_MODES],
    pub decay_scale: f32,
    pub profile: SoundboardProfile,
    pub sample_rate: u32,
}

impl SpruceSoundboard {
    pub fn new(sample_rate: u32, profile: SoundboardProfile) -> Self {
        let freqs = profile.modal_frequencies_hz();
        let qs = profile.modal_q_factors();
        let gains_pans = profile.modal_gains_and_pans();
        let (decay_scale, _, _, _) = profile.nominal_physics();

        let mut modes = [SpruceSoundboardMode::default(); NUM_SOUNDBOARD_MODES];
        for i in 0..NUM_SOUNDBOARD_MODES {
            modes[i] = SpruceSoundboardMode::new(
                sample_rate,
                freqs[i],
                qs[i] * decay_scale,
                gains_pans[i].0,
                gains_pans[i].1,
            );
        }

        Self {
            modes,
            decay_scale,
            profile,
            sample_rate,
        }
    }

    pub fn set_profile(&mut self, profile: SoundboardProfile) {
        self.profile = profile;
        let freqs = profile.modal_frequencies_hz();
        let qs = profile.modal_q_factors();
        let gains_pans = profile.modal_gains_and_pans();
        let (decay_scale, _, _, _) = profile.nominal_physics();
        self.decay_scale = decay_scale;

        for i in 0..NUM_SOUNDBOARD_MODES {
            self.modes[i].freq_hz = freqs[i];
            self.modes[i].q = qs[i] * self.decay_scale;
            self.modes[i].gain = gains_pans[i].0;
            self.modes[i].pan = gains_pans[i].1;
            self.modes[i].update_coefficients(self.sample_rate);
        }
    }

    pub fn set_decay_scale(&mut self, scale: f32) {
        self.decay_scale = scale.clamp(0.1, 5.0);
        let qs = self.profile.modal_q_factors();
        for (i, &q) in qs.iter().enumerate().take(NUM_SOUNDBOARD_MODES) {
            self.modes[i].q = q * self.decay_scale;
            self.modes[i].update_coefficients(self.sample_rate);
        }
    }

    /// Process a driving bridge force sample through the soundboard modal matrix and return (Left, Right) stereo audio.
    #[inline]
    pub fn process_bridge_force(&mut self, bridge_force: f32) -> (f32, f32) {
        let mut out_l = 0.0;
        let mut out_r = 0.0;
        for mode in &mut self.modes {
            let (l, r) = mode.step(bridge_force);
            out_l += l;
            out_r += r;
        }
        (out_l, out_r)
    }

    /// Reset all modal resonator states.
    pub fn reset(&mut self) {
        for mode in &mut self.modes {
            mode.reset();
        }
    }

    /// Analytical 2D plate displacement formula at $(x, y) \in [0, 1]^2$ normalized coordinates
    /// representing anisotropic standing wave modal patterns:
    ///
    /// $W(x, y, t) = \sum_{m,n} A_{mn} \sin(m \pi x) \sin(n \pi y) \cos(\omega_{mn} t)$
    pub fn modal_displacement_at(&self, norm_x: f32, norm_y: f32, phase: f32) -> f32 {
        let x = norm_x.clamp(0.0, 1.0);
        let y = norm_y.clamp(0.0, 1.0);
        let mut disp = 0.0;

        // Anisotropic mode indices (m: longitudinal along spruce grain, n: transverse across grain)
        let mode_indices: [(f32, f32, f32); NUM_SOUNDBOARD_MODES] = [
            (1.0, 1.0, 0.40), // (1,1) Fundamental breathing
            (1.0, 2.0, 0.35), // (1,2)
            (2.0, 1.0, 0.30), // (2,1)
            (2.0, 2.0, 0.28), // (2,2) Torsional
            (3.0, 1.0, 0.22), // (3,1)
            (1.0, 3.0, 0.18), // (1,3)
            (3.0, 2.0, 0.15), // (3,2)
            (2.0, 3.0, 0.12), // (2,3)
        ];

        for (idx, &(m, n, weight)) in mode_indices.iter().enumerate() {
            let spatial = (m * PI * x).sin() * (n * PI * y).sin();
            let mode_phase = phase * (idx as f32 + 1.0) * 1.3;
            disp += weight * spatial * mode_phase.cos();
        }
        disp.clamp(-1.0, 1.0)
    }
}

/// Bi-directional Multi-Port Bridge Wave Scattering Junction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeWaveCoupler {
    /// Nominal mechanical bridge impedance $Z_b$ in $\text{kg/s}$ or $\text{N}\cdot\text{s/m}$.
    pub bridge_impedance: f32,
    /// Sympathetic energy bleed factor $[0.0 ..= 1.0]$ between unisons and neighboring notes.
    pub sympathetic_bleed: f32,
    /// Accumulated bridge velocity state $v_b$.
    pub bridge_velocity: f32,
    /// Previous accumulated bridge force.
    pub prev_bridge_force: f32,
    /// Bridge damping filter state.
    pub bridge_filter_state: f32,
}

impl Default for BridgeWaveCoupler {
    fn default() -> Self {
        Self::new(420.0, 0.35)
    }
}

impl BridgeWaveCoupler {
    pub fn new(bridge_impedance: f32, sympathetic_bleed: f32) -> Self {
        Self {
            bridge_impedance: bridge_impedance.max(10.0),
            sympathetic_bleed: sympathetic_bleed.clamp(0.0, 1.0),
            bridge_velocity: 0.0,
            prev_bridge_force: 0.0,
            bridge_filter_state: 0.0,
        }
    }

    /// Process multi-port bridge scattering junction for incoming string forces:
    ///
    /// 1. $F_{\text{total}} = \sum_{i=0}^{N-1} F_{\text{in}, i}$
    /// 2. $v_b = \frac{1}{Z_b} F_{\text{total}}$
    /// 3. $F_{\text{refl}, i} = -F_{\text{in}, i} + 2 Z_b v_b \cdot (1 - \text{bleed}) + \text{bleed} \cdot F_{\text{total}} / N$
    ///
    /// Returns total driving force transmitted into the spruce soundboard.
    #[inline]
    pub fn process_scattering_junction(
        &mut self,
        string_forces_in: &[f32],
        reflected_forces_out: &mut [f32],
    ) -> f32 {
        let count = string_forces_in.len().min(reflected_forces_out.len());
        if count == 0 {
            return 0.0;
        }

        let mut sum_force = 0.0;
        for &f in &string_forces_in[..count] {
            sum_force += f;
        }

        // Bridge junction admittance velocity: v_b = F_sum / Z_b
        let inv_zb = 1.0 / self.bridge_impedance;
        let raw_vb = sum_force * inv_zb;

        // Bridge velocity mechanical lowpass filtering (bridge inertial mass)
        let alpha = 0.45;
        self.bridge_velocity = (1.0 - alpha) * raw_vb + alpha * self.bridge_velocity;

        let bleed = self.sympathetic_bleed;
        let avg_bleed_force = (sum_force / count as f32) * bleed;

        for i in 0..count {
            let f_in = string_forces_in[i];
            // Reflected wave with phase inversion at bridge plus bridge motion admittance
            let f_refl = -f_in + (2.0 * self.bridge_impedance * self.bridge_velocity * (1.0 - bleed)) + avg_bleed_force;
            reflected_forces_out[i] = f_refl;
        }

        self.prev_bridge_force = sum_force;
        sum_force
    }

    pub fn reset(&mut self) {
        self.bridge_velocity = 0.0;
        self.prev_bridge_force = 0.0;
        self.bridge_filter_state = 0.0;
    }
}

/// Dynamic Non-Linear Felt Hammer Contact Model ($F = K_h \cdot \delta^p \cdot (1 + \mu_h \dot{\delta})$).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrandPianoFeltHammer {
    /// Equivalent hammer mass $m_h$ in kg [0.002 ..= 0.012 kg, graded from treble to bass].
    pub mass_kg: f32,
    /// Felt stiffness constant $K_h$ in $\text{N/m}^p$.
    pub stiffness_k: f32,
    /// Non-linear power-law felt exponent $p \in [2.0 ..= 3.2]$.
    pub exponent_p: f32,
    /// Felt hysteresis damping loss $\mu_h \in [0.05 ..= 0.8]$.
    pub hysteresis_loss: f32,
    /// Relative strike position along the string $\beta \in [0.05 ..= 0.20]$ (typically $1/8$ to $1/16$).
    pub strike_position_beta: f32,
    /// Una Corda shift factor $[0.0 ..= 1.0]$ (0.0 = full 3-string strike, 1.0 = shifted 2-string soft felt strike).
    pub una_corda_shift: f32,
    /// Current hammer displacement $y_h$ in meters.
    pub hammer_disp: f32,
    /// Current hammer velocity $v_h$ in m/s.
    pub hammer_vel: f32,
    /// Previous felt compression $\delta_{prev}$.
    pub prev_compression: f32,
    /// Flag indicating if hammer is currently in contact with the string.
    pub in_contact: bool,
}

impl Default for GrandPianoFeltHammer {
    fn default() -> Self {
        Self::new(0.004, 1.5e8, 2.4, 0.35, 0.125)
    }
}

impl GrandPianoFeltHammer {
    pub fn new(
        mass_kg: f32,
        stiffness_k: f32,
        exponent_p: f32,
        hysteresis_loss: f32,
        strike_position_beta: f32,
    ) -> Self {
        Self {
            mass_kg: mass_kg.max(0.001),
            stiffness_k: stiffness_k.max(1e6),
            exponent_p: exponent_p.clamp(1.5, 3.5),
            hysteresis_loss: hysteresis_loss.clamp(0.0, 1.0),
            strike_position_beta: strike_position_beta.clamp(0.02, 0.5),
            una_corda_shift: 0.0,
            hammer_disp: 0.0,
            hammer_vel: 0.0,
            prev_compression: 0.0,
            in_contact: false,
        }
    }

    /// Trigger hammer strike with initial velocity $v_0$ in m/s $[0.1 ..= 8.0]$.
    pub fn trigger_strike(&mut self, strike_velocity_mps: f32) {
        let v = strike_velocity_mps.clamp(0.05, 10.0);
        // Soft pedal (Una Corda) reduces effective strike impulse and softens felt stiffness
        let effective_v = if self.una_corda_shift > 0.01 {
            v * (1.0 - 0.25 * self.una_corda_shift)
        } else {
            v
        };
        self.hammer_disp = 0.0;
        self.hammer_vel = effective_v;
        self.prev_compression = 0.0;
        self.in_contact = true;
    }

    /// Evaluate one discrete time step of hammer felt interaction against string displacement $y_s$.
    ///
    /// $F_h(\delta) = K_h \cdot \max(0, \delta)^p \cdot \left[1 + \mu_h \cdot \frac{\dot{\delta}}{v_0}\right]$
    ///
    /// Returns instantaneous contact force $F_h$ in Newtons.
    #[inline]
    pub fn step(&mut self, string_disp_m: f32, sample_rate: u32) -> f32 {
        if !self.in_contact && self.hammer_disp <= 0.0 && self.hammer_vel <= 0.0 {
            return 0.0;
        }

        let dt = 1.0 / sample_rate.max(8000) as f32;
        let compression = (self.hammer_disp - string_disp_m).max(0.0);

        if compression > 0.0 {
            self.in_contact = true;
            let comp_dot = (compression - self.prev_compression) / dt;

            // Effective stiffness softened by Una Corda shift
            let eff_k = if self.una_corda_shift > 0.01 {
                self.stiffness_k * (1.0 - 0.35 * self.una_corda_shift)
            } else {
                self.stiffness_k
            };

            let eff_p = if self.una_corda_shift > 0.01 {
                self.exponent_p * (1.0 - 0.08 * self.una_corda_shift)
            } else {
                self.exponent_p
            };

            let static_force = eff_k * compression.powf(eff_p);
            let dynamic_damping = (1.0 + self.hysteresis_loss * comp_dot.clamp(-50.0, 50.0)).max(0.0);
            let contact_force = (static_force * dynamic_damping).max(0.0);

            // Newton's 2nd law for hammer mass: m_h * a_h = -F_h
            let hammer_accel = -contact_force / self.mass_kg;
            self.hammer_vel += hammer_accel * dt;
            self.hammer_disp += self.hammer_vel * dt;
            self.prev_compression = compression;

            contact_force
        } else {
            // Hammer has detached / rebounded from string
            if self.in_contact {
                self.in_contact = false;
            }
            // Free flight after rebound
            self.hammer_disp += self.hammer_vel * dt;
            self.prev_compression = 0.0;
            0.0
        }
    }

    pub fn reset(&mut self) {
        self.hammer_disp = 0.0;
        self.hammer_vel = 0.0;
        self.prev_compression = 0.0;
        self.in_contact = false;
    }
}
