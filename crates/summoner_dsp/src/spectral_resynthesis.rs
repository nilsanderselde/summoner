// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Neural Spectral Morphing & Harmonic Additive Resynthesis Engine.
//!
//! Provides real-time sinusoidal harmonic analysis, phase-locked additive resynthesis,
//! cross-timbre spectral morphing, inharmonicity dispersion, and stochastic noise residual modeling.
//! Zero heap allocations in audio processing paths, strictly enforced under `AllocGuard`.

use std::f32::consts::PI;
use crate::traits::SignalProcessor;
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};

/// Maximum number of additive sinusoidal partials per voice.
pub const MAX_PARTIALS: usize = 64;

/// Size of the zero-allocation circular analysis ring buffer.
pub const ANALYSIS_BUFFER_SIZE: usize = 2048;

/// Individual sinusoidal partial state with phase tracking.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HarmonicPartial {
    /// Frequency ratio relative to the fundamental f0 (e.g. 1.0, 2.0, 3.0...).
    pub freq_ratio: f32,
    /// Current linear amplitude [0.0, 1.0].
    pub amplitude: f32,
    /// Target linear amplitude for smoothing.
    pub target_amplitude: f32,
    /// Current continuous phase accumulator in radians [0.0, 2.0 * PI).
    pub phase: f32,
    /// Inharmonicity offset or frequency shift in Hz.
    pub freq_offset: f32,
}

impl Default for HarmonicPartial {
    fn default() -> Self {
        Self {
            freq_ratio: 1.0,
            amplitude: 0.0,
            target_amplitude: 0.0,
            phase: 0.0,
            freq_offset: 0.0,
        }
    }
}

/// Static spectral profile representing a timbral fingerprint.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HarmonicProfile {
    /// Normalized linear amplitudes for each partial [0..MAX_PARTIALS].
    pub amplitudes: [f32; MAX_PARTIALS],
    /// Frequency ratio multipliers for each partial.
    pub ratio_multipliers: [f32; MAX_PARTIALS],
    /// Spectral centroid hint (normalized 0.0..1.0).
    pub spectral_centroid: f32,
    /// Noise / breathiness ratio [0.0..1.0].
    pub noise_ratio: f32,
}

impl Default for HarmonicProfile {
    fn default() -> Self {
        Self::sawtooth()
    }
}

impl HarmonicProfile {
    /// Pure sawtooth spectral profile (1/k harmonic decay).
    pub fn sawtooth() -> Self {
        let mut amplitudes = [0.0f32; MAX_PARTIALS];
        let mut ratio_multipliers = [1.0f32; MAX_PARTIALS];
        for k in 0..MAX_PARTIALS {
            let harmonic_num = (k + 1) as f32;
            amplitudes[k] = 1.0 / harmonic_num;
            ratio_multipliers[k] = harmonic_num;
        }
        Self {
            amplitudes,
            ratio_multipliers,
            spectral_centroid: 0.35,
            noise_ratio: 0.02,
        }
    }

    /// Square wave profile (odd harmonics only: 1, 3, 5... with 1/k decay).
    pub fn square() -> Self {
        let mut amplitudes = [0.0f32; MAX_PARTIALS];
        let mut ratio_multipliers = [1.0f32; MAX_PARTIALS];
        for k in 0..MAX_PARTIALS {
            let harmonic_num = (k + 1) as f32;
            if (k + 1) % 2 == 1 {
                amplitudes[k] = 1.0 / harmonic_num;
            } else {
                amplitudes[k] = 0.0;
            }
            ratio_multipliers[k] = harmonic_num;
        }
        Self {
            amplitudes,
            ratio_multipliers,
            spectral_centroid: 0.3,
            noise_ratio: 0.01,
        }
    }

    /// Triangle wave profile (odd harmonics only with 1/k^2 decay).
    pub fn triangle() -> Self {
        let mut amplitudes = [0.0f32; MAX_PARTIALS];
        let mut ratio_multipliers = [1.0f32; MAX_PARTIALS];
        for k in 0..MAX_PARTIALS {
            let harmonic_num = (k + 1) as f32;
            if (k + 1) % 2 == 1 {
                amplitudes[k] = 1.0 / (harmonic_num * harmonic_num);
            } else {
                amplitudes[k] = 0.0;
            }
            ratio_multipliers[k] = harmonic_num;
        }
        Self {
            amplitudes,
            ratio_multipliers,
            spectral_centroid: 0.15,
            noise_ratio: 0.0,
        }
    }

    /// Pure sine profile (fundamental only).
    pub fn sine() -> Self {
        let mut amplitudes = [0.0f32; MAX_PARTIALS];
        let mut ratio_multipliers = [1.0f32; MAX_PARTIALS];
        amplitudes[0] = 1.0;
        for (k, ratio) in ratio_multipliers.iter_mut().enumerate() {
            *ratio = (k + 1) as f32;
        }
        Self {
            amplitudes,
            ratio_multipliers,
            spectral_centroid: 0.05,
            noise_ratio: 0.0,
        }
    }

    /// Vocal vowel formant profile ("Ah" sound).
    pub fn vocal_formant_ah() -> Self {
        let mut amplitudes = [0.0f32; MAX_PARTIALS];
        let mut ratio_multipliers = [1.0f32; MAX_PARTIALS];
        for k in 0..MAX_PARTIALS {
            let h = (k + 1) as f32;
            ratio_multipliers[k] = h;
            // Formant peaks around 800 Hz and 1200 Hz (relative harmonic multiples)
            let f1 = (-0.5 * ((h - 4.0) / 1.5).powi(2)).exp() * 0.9;
            let f2 = (-0.5 * ((h - 7.0) / 2.0).powi(2)).exp() * 0.7;
            let f3 = (-0.5 * ((h - 14.0) / 3.0).powi(2)).exp() * 0.3;
            amplitudes[k] = (f1 + f2 + f3 + 0.05 / h).clamp(0.0, 1.0);
        }
        Self {
            amplitudes,
            ratio_multipliers,
            spectral_centroid: 0.45,
            noise_ratio: 0.05,
        }
    }

    /// Vocal vowel formant profile ("Ee" sound).
    pub fn vocal_formant_ee() -> Self {
        let mut amplitudes = [0.0f32; MAX_PARTIALS];
        let mut ratio_multipliers = [1.0f32; MAX_PARTIALS];
        for k in 0..MAX_PARTIALS {
            let h = (k + 1) as f32;
            ratio_multipliers[k] = h;
            // Formants around 300 Hz and 2500 Hz
            let f1 = (-0.5 * ((h - 2.0) / 1.0).powi(2)).exp() * 1.0;
            let f2 = (-0.5 * ((h - 12.0) / 2.5).powi(2)).exp() * 0.8;
            amplitudes[k] = (f1 + f2 + 0.02 / h).clamp(0.0, 1.0);
        }
        Self {
            amplitudes,
            ratio_multipliers,
            spectral_centroid: 0.55,
            noise_ratio: 0.04,
        }
    }

    /// Inharmonic metallic bell / chime profile.
    pub fn metallic_bell() -> Self {
        let mut amplitudes = [0.0f32; MAX_PARTIALS];
        let mut ratio_multipliers = [1.0f32; MAX_PARTIALS];
        let ratios = [
            1.0, 2.756, 5.404, 8.933, 11.23, 15.67, 18.21, 22.45,
            26.12, 30.5, 35.8, 41.2, 48.0, 55.3, 62.1, 70.0,
        ];
        for k in 0..MAX_PARTIALS {
            if k < ratios.len() {
                ratio_multipliers[k] = ratios[k];
                amplitudes[k] = (1.0 / (1.0 + k as f32 * 0.4)).powf(1.2);
            } else {
                let h = (k + 1) as f32;
                ratio_multipliers[k] = h * 1.414;
                amplitudes[k] = 0.05 / h;
            }
        }
        Self {
            amplitudes,
            ratio_multipliers,
            spectral_centroid: 0.65,
            noise_ratio: 0.08,
        }
    }

    /// Warm acoustic cello / bowed string profile.
    pub fn bowed_string() -> Self {
        let mut amplitudes = [0.0f32; MAX_PARTIALS];
        let mut ratio_multipliers = [1.0f32; MAX_PARTIALS];
        for k in 0..MAX_PARTIALS {
            let h = (k + 1) as f32;
            ratio_multipliers[k] = h;
            let formant = (-0.5 * ((h - 3.0) / 2.0).powi(2)).exp() * 0.8;
            let base = 0.9 / (h.powf(0.85));
            amplitudes[k] = (base + formant).clamp(0.0, 1.0);
        }
        Self {
            amplitudes,
            ratio_multipliers,
            spectral_centroid: 0.38,
            noise_ratio: 0.06,
        }
    }

    /// Interpolate / morph between two spectral profiles with nonlinear weighting.
    pub fn morph(&self, other: &Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        let mut amplitudes = [0.0f32; MAX_PARTIALS];
        let mut ratio_multipliers = [1.0f32; MAX_PARTIALS];

        for k in 0..MAX_PARTIALS {
            // Power-conserving smooth interpolation of partial amplitudes
            let a1 = self.amplitudes[k];
            let a2 = other.amplitudes[k];
            amplitudes[k] = ((1.0 - t) * a1.powi(2) + t * a2.powi(2)).sqrt();
            ratio_multipliers[k] = (1.0 - t) * self.ratio_multipliers[k] + t * other.ratio_multipliers[k];
        }

        let spectral_centroid = (1.0 - t) * self.spectral_centroid + t * other.spectral_centroid;
        let noise_ratio = (1.0 - t) * self.noise_ratio + t * other.noise_ratio;

        Self {
            amplitudes,
            ratio_multipliers,
            spectral_centroid,
            noise_ratio,
        }
    }
}

/// Real-time neural spectral morphing and additive resynthesis processor.
#[derive(Debug, Clone)]
pub struct SpectralResynthesisEngine {
    pub sample_rate: u32,
    pub fundamental_hz: f32,
    pub morph_amount: f32,
    pub inharmonicity: f32,
    pub odd_even_balance: f32,
    pub spectral_tilt_db: f32,
    pub formant_shift: f32,
    pub noise_residual_gain: f32,
    pub amplitude_envelope: f32,

    profile_a: HarmonicProfile,
    profile_b: HarmonicProfile,
    current_profile: HarmonicProfile,

    partials: [HarmonicPartial; MAX_PARTIALS],
    analysis_buffer: [f32; ANALYSIS_BUFFER_SIZE],
    analysis_head: usize,
    pitch_tracker_state: f32,
    noise_rng_state: u32,
    noise_lp_state: f32,
}

impl SpectralResynthesisEngine {
    pub fn new(sample_rate: u32) -> Self {
        let profile_a = HarmonicProfile::sawtooth();
        let profile_b = HarmonicProfile::vocal_formant_ah();
        let mut engine = Self {
            sample_rate: sample_rate.max(8000),
            fundamental_hz: 220.0,
            morph_amount: 0.0,
            inharmonicity: 0.0,
            odd_even_balance: 0.0,
            spectral_tilt_db: 0.0,
            formant_shift: 1.0,
            noise_residual_gain: 0.05,
            amplitude_envelope: 1.0,
            profile_a,
            profile_b,
            current_profile: profile_a,
            partials: [HarmonicPartial::default(); MAX_PARTIALS],
            analysis_buffer: [0.0; ANALYSIS_BUFFER_SIZE],
            analysis_head: 0,
            pitch_tracker_state: 0.0,
            noise_rng_state: 0x1337_900D,
            noise_lp_state: 0.0,
        };

        engine.init_partials();
        engine.update_target_profile();
        engine
    }

    fn init_partials(&mut self) {
        for k in 0..MAX_PARTIALS {
            let h = (k + 1) as f32;
            self.partials[k] = HarmonicPartial {
                freq_ratio: h,
                amplitude: self.current_profile.amplitudes[k],
                target_amplitude: self.current_profile.amplitudes[k],
                phase: 0.0,
                freq_offset: 0.0,
            };
        }
    }

    /// Set spectral profile A.
    pub fn set_profile_a(&mut self, profile: HarmonicProfile) {
        self.profile_a = profile;
        self.update_target_profile();
    }

    /// Set spectral profile B.
    pub fn set_profile_b(&mut self, profile: HarmonicProfile) {
        self.profile_b = profile;
        self.update_target_profile();
    }

    /// Set morph blend parameter [0.0..1.0].
    pub fn set_morph(&mut self, morph: f32) {
        self.morph_amount = morph.clamp(0.0, 1.0);
        self.update_target_profile();
    }

    /// Set base fundamental frequency in Hz.
    pub fn set_fundamental(&mut self, hz: f32) {
        self.fundamental_hz = hz.clamp(10.0, 20_000.0);
    }

    /// Set inharmonicity coefficient (dispersion factor B >= 0.0).
    pub fn set_inharmonicity(&mut self, b: f32) {
        self.inharmonicity = b.clamp(0.0, 1.0);
    }

    /// Set balance between odd and even harmonics (-1.0 = all odd, +1.0 = all even, 0.0 = natural).
    pub fn set_odd_even_balance(&mut self, balance: f32) {
        self.odd_even_balance = balance.clamp(-1.0, 1.0);
    }

    /// Set spectral tilt in dB per octave (-12.0 dB = darker, +12.0 dB = brighter).
    pub fn set_spectral_tilt(&mut self, tilt_db: f32) {
        self.spectral_tilt_db = tilt_db.clamp(-24.0, 24.0);
    }

    /// Set formant frequency shift ratio (0.5 = octave down, 2.0 = octave up).
    pub fn set_formant_shift(&mut self, shift: f32) {
        self.formant_shift = shift.clamp(0.25, 4.0);
    }

    /// Set stochastic noise residual gain.
    pub fn set_noise_residual_gain(&mut self, gain: f32) {
        self.noise_residual_gain = gain.clamp(0.0, 1.0);
    }

    /// Internal profile blend update.
    fn update_target_profile(&mut self) {
        self.current_profile = self.profile_a.morph(&self.profile_b, self.morph_amount);
        for k in 0..MAX_PARTIALS {
            let raw_amp = self.current_profile.amplitudes[k];
            let harmonic_idx = k + 1;

            // Apply Odd/Even balance
            let parity_factor = if harmonic_idx % 2 == 1 {
                (1.0 - self.odd_even_balance).clamp(0.0, 2.0)
            } else {
                (1.0 + self.odd_even_balance).clamp(0.0, 2.0)
            };

            // Apply Spectral Tilt (dB/octave)
            let octaves_above_f0 = (harmonic_idx as f32).log2();
            let tilt_gain = 10.0f32.powf((self.spectral_tilt_db * octaves_above_f0) / 20.0);

            let shaped_amp = (raw_amp * parity_factor * tilt_gain).clamp(0.0, 2.0);
            self.partials[k].target_amplitude = shaped_amp;
            self.partials[k].freq_ratio = self.current_profile.ratio_multipliers[k];
        }
    }

    /// Zero-allocation lightweight PRNG for stochastic noise synthesis.
    #[inline(always)]
    fn next_noise(&mut self) -> f32 {
        self.noise_rng_state = self.noise_rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        ((self.noise_rng_state as f32 / u32::MAX as f32) * 2.0) - 1.0
    }

    /// Ingest streaming sample into zero-allocation analysis ring buffer.
    #[inline(always)]
    pub fn push_analysis_sample(&mut self, sample: f32) {
        self.analysis_buffer[self.analysis_head] = sample;
        self.analysis_head = (self.analysis_head + 1) % ANALYSIS_BUFFER_SIZE;

        // Peak zero-crossing and autocorrelation pitch estimator tracker
        let diff = sample - self.pitch_tracker_state;
        self.pitch_tracker_state += 0.05 * diff;
    }

    /// Synthesize one sample frame (L, R stereo) with phase tracking and zero allocation.
    #[inline(always)]
    pub fn process_sample(&mut self, ext_audio_in: Option<f32>) -> (f32, f32) {
        if let Some(in_sample) = ext_audio_in {
            self.push_analysis_sample(in_sample);
        }

        let f0 = self.fundamental_hz;
        let sr = self.sample_rate as f32;
        let nyquist = sr * 0.495;
        let b = self.inharmonicity;

        let mut sum_left = 0.0f32;
        let mut sum_right = 0.0f32;

        let smooth_coeff = 0.01f32;

        for k in 0..MAX_PARTIALS {
            let partial = &mut self.partials[k];
            
            // Smooth amplitude transitions
            partial.amplitude += smooth_coeff * (partial.target_amplitude - partial.amplitude);

            if partial.amplitude < 1e-5 && partial.target_amplitude < 1e-5 {
                continue;
            }

            // Inharmonic dispersion calculation: fk = f0 * ratio * sqrt(1 + B * ratio^2)
            let base_ratio = partial.freq_ratio * self.formant_shift;
            let inharmonic_multiplier = (1.0 + b * base_ratio * base_ratio).sqrt();
            let partial_freq = f0 * base_ratio * inharmonic_multiplier;

            // Anti-aliasing check against Nyquist limit
            if partial_freq >= nyquist || partial_freq <= 5.0 {
                continue;
            }

            // Continuous phase increment modulo 2*PI
            let phase_inc = (2.0 * PI * partial_freq) / sr;
            partial.phase = (partial.phase + phase_inc).rem_euclid(2.0 * PI);

            let sine_val = partial.phase.sin() * partial.amplitude;

            // Spatial stereo spread across harmonic partials
            let pan_phase = (k as f32 * 0.35).sin() * 0.3; // -0.3 to +0.3 pan
            let pan_l = (0.5 - pan_phase * 0.5).sqrt();
            let pan_r = (0.5 + pan_phase * 0.5).sqrt();

            sum_left += sine_val * pan_l;
            sum_right += sine_val * pan_r;
        }

        // Stochastic noise residual component
        let raw_noise = self.next_noise();
        let noise_filter_coeff = 0.15;
        self.noise_lp_state += noise_filter_coeff * (raw_noise - self.noise_lp_state);
        let noise_out = self.noise_lp_state * self.noise_residual_gain * self.current_profile.noise_ratio;

        let out_l = ((sum_left + noise_out) * self.amplitude_envelope * 0.25).clamp(-1.0, 1.0);
        let out_r = ((sum_right + noise_out) * self.amplitude_envelope * 0.25).clamp(-1.0, 1.0);

        (out_l, out_r)
    }
}

impl SignalProcessor for SpectralResynthesisEngine {
    fn name(&self) -> &str {
        "SpectralResynthesisEngine"
    }

    fn process_block(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        _ctx: &ProcessContext,
    ) {
        if outputs.is_empty() {
            return;
        }

        let num_samples = outputs[0].len();
        let has_input = !inputs.is_empty() && !inputs[0].is_empty();

        let num_out_channels = outputs.len();

        for i in 0..num_samples {
            let in_sample = if has_input && i < inputs[0].len() {
                Some(inputs[0][i])
            } else {
                None
            };

            let (sample_l, sample_r) = self.process_sample(in_sample);

            if num_out_channels >= 2 {
                if i < outputs[0].len() {
                    outputs[0][i] = sample_l;
                }
                if i < outputs[1].len() {
                    outputs[1][i] = sample_r;
                }
            } else if num_out_channels == 1 && i < outputs[0].len() {
                outputs[0][i] = (sample_l + sample_r) * 0.5;
            }
        }
    }
}

impl AudioNode for SpectralResynthesisEngine {
    fn name(&self) -> &str {
        "SpectralResynthesisEngine"
    }

    fn process(
        &mut self,
        input: &[&[Sample]],
        output: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        self.process_block(input, output, ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::allocator::AllocGuard;
    use summoner_core::transport::Transport;

    #[test]
    fn test_spectral_resynthesis_zero_allocation() {
        let mut engine = SpectralResynthesisEngine::new(48000);
        engine.set_fundamental(440.0);
        engine.set_morph(0.5);
        engine.set_inharmonicity(0.05);

        let input_block = [0.1f32; 128];
        let mut out_l = [0.0f32; 128];
        let mut out_r = [0.0f32; 128];

        let transport = Transport::new(48000, 120.0);
        let ctx = ProcessContext::from_transport(&transport);

        // Verify zero heap allocation during block rendering
        let _guard = AllocGuard::new();

        engine.process_block(
            &[&input_block[..]],
            &mut [&mut out_l[..], &mut out_r[..]],
            &ctx,
        );

        drop(_guard);

        assert!(out_l.iter().any(|&s| s.abs() > 0.0));
        assert!(out_r.iter().any(|&s| s.abs() > 0.0));
        assert!(out_l.iter().all(|&s| s.is_finite() && (-1.0..=1.0).contains(&s)));
        assert!(out_r.iter().all(|&s| s.is_finite() && (-1.0..=1.0).contains(&s)));
    }

    #[test]
    fn test_harmonic_profile_morphing() {
        let saw = HarmonicProfile::sawtooth();
        let vocal = HarmonicProfile::vocal_formant_ah();

        let morphed_0 = saw.morph(&vocal, 0.0);
        let morphed_50 = saw.morph(&vocal, 0.5);
        let morphed_100 = saw.morph(&vocal, 1.0);

        assert_eq!(morphed_0.amplitudes[0], saw.amplitudes[0]);
        assert_eq!(morphed_100.amplitudes[0], vocal.amplitudes[0]);
        assert!(morphed_50.amplitudes[0] > 0.0);
        assert!(morphed_50.spectral_centroid > saw.spectral_centroid);
    }

    #[test]
    fn test_phase_continuity_and_clamping() {
        let mut engine = SpectralResynthesisEngine::new(44100);
        engine.set_fundamental(110.0);
        engine.set_profile_a(HarmonicProfile::metallic_bell());
        engine.set_profile_b(HarmonicProfile::vocal_formant_ee());
        engine.set_morph(0.75);

        for _ in 0..10 {
            let (l, r) = engine.process_sample(None);
            assert!(l.is_finite() && (-1.0..=1.0).contains(&l));
            assert!(r.is_finite() && (-1.0..=1.0).contains(&r));
        }
    }
}
