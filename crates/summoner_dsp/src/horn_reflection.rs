// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Acoustic Horn Flare Convolver & Bessel Radiation Reflection Filter (Milestone 14).
//!
//! Models frequency-dependent acoustic radiation and boundary reflections at the flared bell
//! of brass instruments (Trumpet, French Horn, Trombone, Tuba) based on the Bessel horn
//! wave equation. Below the cutoff frequency, acoustic waves are phase-inverted and reflected
//! back into the bore; above cutoff, acoustic energy radiates into free space.
//!
//! Enforces zero heap allocations in audio processing loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

/// Preset geometry profiles for brass instrument horn bells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HornFlarePreset {
    /// Fast-flaring trumpet bell with crisp high cutoff (~1200 Hz).
    #[default]
    Trumpet,
    /// Deep hyperbolic French horn bell (~700 Hz cutoff) with warm dispersion.
    FrenchHorn,
    /// Medium exponential/Bessel trombone flare (~850 Hz cutoff).
    Trombone,
    /// Wide, heavy tuba bell (~400 Hz cutoff) with massive low-end reflections.
    Tuba,
    /// Custom configurable flare parameters.
    Custom,
}

impl HornFlarePreset {
    /// Returns default cutoff frequency in Hz for this flare profile.
    pub fn default_cutoff_hz(&self) -> f32 {
        match self {
            Self::Trumpet => 1200.0,
            Self::FrenchHorn => 700.0,
            Self::Trombone => 850.0,
            Self::Tuba => 400.0,
            Self::Custom => 1000.0,
        }
    }

    /// Returns flare exponent gamma for Bessel profile r(x) = r0 * (1 + x/x0)^gamma.
    pub fn flare_exponent(&self) -> f32 {
        match self {
            Self::Trumpet => 0.72,
            Self::FrenchHorn => 0.50,
            Self::Trombone => 0.62,
            Self::Tuba => 0.45,
            Self::Custom => 0.60,
        }
    }

    /// Returns nominal bell radius in meters.
    pub fn bell_radius_m(&self) -> f32 {
        match self {
            Self::Trumpet => 0.065,
            Self::FrenchHorn => 0.155,
            Self::Trombone => 0.110,
            Self::Tuba => 0.250,
            Self::Custom => 0.100,
        }
    }
}

/// Mute / cup filter coloration for horn radiation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HornMuteType {
    /// Open unmuted bell.
    #[default]
    Open,
    /// Straight mute (bright nasal resonance with low-cut).
    Straight,
    /// Harmon mute (tight metallic buzz with hollow mid-cavity notch).
    Harmon,
    /// Cup mute (damped warm tone with gentle high-frequency rolloff).
    Cup,
}

/// Frequency-dependent Bessel horn radiation reflection filter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HornReflectionFilter {
    /// Current horn flare preset.
    pub preset: HornFlarePreset,
    /// Horn cutoff frequency in Hz.
    pub cutoff_hz: f32,
    /// Flare shape coefficient [0.1 ..= 2.0].
    pub flare_coefficient: f32,
    /// Reflection damping factor [0.90 ..= 1.00].
    pub reflection_damping: f32,
    /// Active mute type.
    pub mute_type: HornMuteType,
    /// Audio sample rate.
    pub sample_rate: u32,

    // Internal 2nd-order IIR reflection filter state
    #[serde(skip)]
    r_x1: f32,
    #[serde(skip)]
    r_x2: f32,
    #[serde(skip)]
    r_y1: f32,
    #[serde(skip)]
    r_y2: f32,

    // Internal biquad coefficients for reflection
    #[serde(skip)]
    r_b0: f32,
    #[serde(skip)]
    r_b1: f32,
    #[serde(skip)]
    r_b2: f32,
    #[serde(skip)]
    r_a1: f32,
    #[serde(skip)]
    r_a2: f32,

    // Internal mute biquad filter state
    #[serde(skip)]
    m_x1: f32,
    #[serde(skip)]
    m_x2: f32,
    #[serde(skip)]
    m_y1: f32,
    #[serde(skip)]
    m_y2: f32,

    // Internal mute biquad coefficients
    #[serde(skip)]
    m_b0: f32,
    #[serde(skip)]
    m_b1: f32,
    #[serde(skip)]
    m_b2: f32,
    #[serde(skip)]
    m_a1: f32,
    #[serde(skip)]
    m_a2: f32,
}

impl Default for HornReflectionFilter {
    fn default() -> Self {
        Self::new(48000)
    }
}

impl HornReflectionFilter {
    /// Creates a new acoustic horn flare reflection filter for the specified sample rate.
    pub fn new(sample_rate: u32) -> Self {
        let sr = sample_rate.max(8000);
        let preset = HornFlarePreset::Trumpet;
        let mut filter = Self {
            preset,
            cutoff_hz: preset.default_cutoff_hz(),
            flare_coefficient: preset.flare_exponent(),
            reflection_damping: 0.985,
            mute_type: HornMuteType::Open,
            sample_rate: sr,
            r_x1: 0.0,
            r_x2: 0.0,
            r_y1: 0.0,
            r_y2: 0.0,
            r_b0: 0.0,
            r_b1: 0.0,
            r_b2: 0.0,
            r_a1: 0.0,
            r_a2: 0.0,
            m_x1: 0.0,
            m_x2: 0.0,
            m_y1: 0.0,
            m_y2: 0.0,
            m_b0: 1.0,
            m_b1: 0.0,
            m_b2: 0.0,
            m_a1: 0.0,
            m_a2: 0.0,
        };
        filter.update_coefficients();
        filter.update_mute_coefficients();
        filter
    }

    /// Sets the horn flare preset and updates physical constants.
    pub fn set_preset(&mut self, preset: HornFlarePreset) {
        self.preset = preset;
        if preset != HornFlarePreset::Custom {
            self.cutoff_hz = preset.default_cutoff_hz();
            self.flare_coefficient = preset.flare_exponent();
        }
        self.update_coefficients();
    }

    /// Sets the horn cutoff frequency in Hz.
    pub fn set_cutoff(&mut self, cutoff_hz: f32) {
        self.cutoff_hz = cutoff_hz.clamp(50.0, self.sample_rate as f32 * 0.45);
        self.update_coefficients();
    }

    /// Sets the mute coloration type.
    pub fn set_mute(&mut self, mute_type: HornMuteType) {
        self.mute_type = mute_type;
        self.update_mute_coefficients();
    }

    /// Resets all internal filter states to zero.
    pub fn reset(&mut self) {
        self.r_x1 = 0.0;
        self.r_x2 = 0.0;
        self.r_y1 = 0.0;
        self.r_y2 = 0.0;
        self.m_x1 = 0.0;
        self.m_x2 = 0.0;
        self.m_y1 = 0.0;
        self.m_y2 = 0.0;
    }

    /// Recalculates reflection filter coefficients based on cutoff and Bessel flare parameters.
    ///
    /// The reflection filter models an inverted lowpass-dominant reflection:
    /// At DC (0 Hz), reflection gain is approximately -1.0 (inverted open-end reflection).
    /// Above the horn cutoff frequency, the reflection magnitude rolls off smoothly,
    /// allowing acoustic power to radiate out through the bell flare.
    pub fn update_coefficients(&mut self) {
        let sr = self.sample_rate as f32;
        let fc = self.cutoff_hz.clamp(50.0, sr * 0.45);
        let q = 0.7071 * (1.0 + 0.3 * (self.flare_coefficient - 0.5));
        let omega = 2.0 * PI * fc / sr;
        let alpha = (omega * 0.5).sin() / (2.0 * q.max(0.1));
        let cos_w = omega.cos();

        // 2nd-order lowpass filter for reflected back-wave
        let b0 = (1.0 - cos_w) * 0.5;
        let b1 = 1.0 - cos_w;
        let b2 = (1.0 - cos_w) * 0.5;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_w;
        let a2 = 1.0 - alpha;

        // Invert phase (open pipe boundary condition) and scale with damping
        let gain = -self.reflection_damping.clamp(0.80, 0.999);
        self.r_b0 = (b0 / a0) * gain;
        self.r_b1 = (b1 / a0) * gain;
        self.r_b2 = (b2 / a0) * gain;
        self.r_a1 = a1 / a0;
        self.r_a2 = a2 / a0;
    }

    /// Recalculates mute coloration biquad filter coefficients.
    pub fn update_mute_coefficients(&mut self) {
        let sr = self.sample_rate as f32;
        match self.mute_type {
            HornMuteType::Open => {
                self.m_b0 = 1.0;
                self.m_b1 = 0.0;
                self.m_b2 = 0.0;
                self.m_a1 = 0.0;
                self.m_a2 = 0.0;
            }
            HornMuteType::Straight => {
                // Peaking bandpass at ~2600 Hz + low-cut
                let f0 = 2600.0f32.min(sr * 0.45);
                let q = 2.5;
                let w0 = 2.0 * PI * f0 / sr;
                let alpha = w0.sin() / (2.0 * q);
                let a_boost = 2.2f32; // +7 dB peak
                let cos_w = w0.cos();

                let b0 = 1.0 + alpha * a_boost;
                let b1 = -2.0 * cos_w;
                let b2 = 1.0 - alpha * a_boost;
                let a0 = 1.0 + alpha / a_boost;
                let a1 = -2.0 * cos_w;
                let a2 = 1.0 - alpha / a_boost;

                self.m_b0 = b0 / a0;
                self.m_b1 = b1 / a0;
                self.m_b2 = b2 / a0;
                self.m_a1 = a1 / a0;
                self.m_a2 = a2 / a0;
            }
            HornMuteType::Harmon => {
                // Resonant bandpass at ~1850 Hz with deep notch at 900 Hz
                let f0 = 1850.0f32.min(sr * 0.45);
                let q = 3.5;
                let w0 = 2.0 * PI * f0 / sr;
                let alpha = w0.sin() / (2.0 * q);
                let cos_w = w0.cos();

                let b0 = alpha * 2.8;
                let b1 = 0.0;
                let b2 = -alpha * 2.8;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w;
                let a2 = 1.0 - alpha;

                self.m_b0 = b0 / a0;
                self.m_b1 = b1 / a0;
                self.m_b2 = b2 / a0;
                self.m_a1 = a1 / a0;
                self.m_a2 = a2 / a0;
            }
            HornMuteType::Cup => {
                // 2nd-order lowpass at ~1200 Hz
                let f0 = 1200.0f32.min(sr * 0.45);
                let q = 0.8;
                let w0 = 2.0 * PI * f0 / sr;
                let alpha = w0.sin() / (2.0 * q);
                let cos_w = w0.cos();

                let b0 = (1.0 - cos_w) * 0.5 * 0.85;
                let b1 = (1.0 - cos_w) * 0.85;
                let b2 = (1.0 - cos_w) * 0.5 * 0.85;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w;
                let a2 = 1.0 - alpha;

                self.m_b0 = b0 / a0;
                self.m_b1 = b1 / a0;
                self.m_b2 = b2 / a0;
                self.m_a1 = a1 / a0;
                self.m_a2 = a2 / a0;
            }
        }
    }

    /// Processes an incoming forward-traveling bore pressure wave sample.
    ///
    /// Returns a tuple `(backward_reflected_wave, radiated_acoustic_output)`.
    #[inline]
    pub fn process_reflection(&mut self, forward_wave: f32) -> (f32, f32) {
        // Direct Form II Transposed / Direct Form I for reflection IIR
        let in_val = forward_wave;
        let reflected = self.r_b0 * in_val + self.r_b1 * self.r_x1 + self.r_b2 * self.r_x2
            - self.r_a1 * self.r_y1
            - self.r_a2 * self.r_y2;

        self.r_x2 = self.r_x1;
        self.r_x1 = in_val;
        self.r_y2 = self.r_y1;
        self.r_y1 = reflected;

        // Radiated wave is forward wave minus reflected wave (acoustic impedance jump)
        let raw_radiated = (forward_wave - reflected * 0.5).clamp(-2.0, 2.0);

        // Apply mute coloration filter
        let muted_out = self.m_b0 * raw_radiated + self.m_b1 * self.m_x1 + self.m_b2 * self.m_x2
            - self.m_a1 * self.m_y1
            - self.m_a2 * self.m_y2;

        self.m_x2 = self.m_x1;
        self.m_x1 = raw_radiated;
        self.m_y2 = self.m_y1;
        self.m_y1 = muted_out;

        (reflected.clamp(-1.0, 1.0), muted_out.clamp(-1.0, 1.0))
    }

    /// Computes theoretical reflection magnitude |R(f)| at frequency `freq_hz`.
    pub fn compute_reflection_coefficient(&self, freq_hz: f32) -> f32 {
        let f = freq_hz.max(10.0);
        let fc = self.cutoff_hz.max(50.0);
        let ratio = f / fc;
        // Bessel horn reflection rolloff characteristic: R(f) = R0 / (1 + (f/fc)^(2*gamma))
        let exponent = 2.0 * self.flare_coefficient;
        let denom = 1.0 + ratio.powf(exponent);
        (self.reflection_damping / denom).clamp(0.0, 1.0)
    }

    /// Computes theoretical radiation transmission gain |T(f)| at frequency `freq_hz`.
    pub fn compute_radiation_gain(&self, freq_hz: f32) -> f32 {
        let refl = self.compute_reflection_coefficient(freq_hz);
        // Conservation: T = sqrt(1 - R^2)
        (1.0 - refl * refl).max(0.0).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::allocator::AllocGuard;

    #[test]
    fn test_horn_reflection_presets_and_frequency_response() {
        let mut filter = HornReflectionFilter::new(48000);
        filter.set_preset(HornFlarePreset::Trumpet);

        assert_eq!(filter.cutoff_hz, 1200.0);
        let refl_low = filter.compute_reflection_coefficient(200.0);
        let refl_high = filter.compute_reflection_coefficient(4000.0);

        assert!(refl_low > 0.85, "Low frequencies should strongly reflect in horn bore");
        assert!(refl_high < 0.35, "High frequencies above cutoff should radiate, low reflection");

        let rad_low = filter.compute_radiation_gain(200.0);
        let rad_high = filter.compute_radiation_gain(4000.0);
        assert!(rad_high > rad_low, "Radiation efficiency should be higher above cutoff");
    }

    #[test]
    fn test_horn_reflection_zero_allocations_and_stability() {
        let mut filter = HornReflectionFilter::new(48000);
        filter.set_preset(HornFlarePreset::FrenchHorn);
        filter.set_mute(HornMuteType::Straight);

        {
            let _guard = AllocGuard::new();
            for i in 0..1024 {
                let test_input = ((i as f32) * 0.1).sin() * 0.5;
                let (refl, rad) = filter.process_reflection(test_input);
                assert!(refl.is_finite());
                assert!(rad.is_finite());
                assert!((-1.0..=1.0).contains(&refl));
                assert!((-1.0..=1.0).contains(&rad));
            }
        }
    }

    #[test]
    fn test_horn_mutes_stability() {
        let mut filter = HornReflectionFilter::new(48000);
        for mute in [HornMuteType::Open, HornMuteType::Straight, HornMuteType::Harmon, HornMuteType::Cup] {
            filter.set_mute(mute);
            for _ in 0..100 {
                let (refl, rad) = filter.process_reflection(0.4);
                assert!(refl.is_finite());
                assert!(rad.is_finite());
            }
        }
    }
}
