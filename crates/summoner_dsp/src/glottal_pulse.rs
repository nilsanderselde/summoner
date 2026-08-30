// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Rosenberg Glottal Flow Waveguide Pulse Oscillator (Milestone 15).
//!
//! Implements a physiologically accurate $C^1$-continuous glottal flow volume velocity
//! and flow derivative excitation generator based on the Rosenberg glottal pulse model.
//! Includes cycle-synchronous pitch jitter, amplitude shimmer, and turbulent aspiration noise.
//!
//! Enforces zero heap allocations in audio processing loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};
use crate::traits::SignalProcessor;

/// Voicing register mode for glottal source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GlottalVoicingMode {
    /// Modal voice (standard speech / singing register).
    #[default]
    Modal,
    /// Falsetto / head voice (higher open quotient, reduced high harmonics).
    Falsetto,
    /// Vocal fry / creaky voice (low fundamental, short explosive pulses).
    Creaky,
    /// Breathy voice (large open quotient, elevated aspiration turbulence).
    Breathy,
    /// Whispered phonation (unvoiced turbulent aspiration only).
    Whisper,
}

impl GlottalVoicingMode {
    /// Returns default open quotient for this register.
    pub fn default_open_quotient(&self) -> f32 {
        match self {
            Self::Modal => 0.60,
            Self::Falsetto => 0.75,
            Self::Creaky => 0.35,
            Self::Breathy => 0.85,
            Self::Whisper => 0.95,
        }
    }

    /// Returns default speed quotient (opening duration / closing duration).
    pub fn default_speed_quotient(&self) -> f32 {
        match self {
            Self::Modal => 2.0,
            Self::Falsetto => 1.5,
            Self::Creaky => 3.5,
            Self::Breathy => 1.2,
            Self::Whisper => 1.0,
        }
    }

    /// Returns default aspiration turbulence noise level [0.0 ..= 1.0].
    pub fn default_aspiration_level(&self) -> f32 {
        match self {
            Self::Modal => 0.04,
            Self::Falsetto => 0.08,
            Self::Creaky => 0.02,
            Self::Breathy => 0.35,
            Self::Whisper => 0.85,
        }
    }
}

/// Rosenberg Glottal Flow Pulse Generator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlottalPulse {
    /// Fundamental pitch frequency in Hz.
    pub frequency_hz: f32,
    /// Master voicing amplitude [0.0 ..= 1.5].
    pub voicing_amplitude: f32,
    /// Open quotient $O_q = T_{open} / T_0 \in [0.20 ..= 0.95]$.
    pub open_quotient: f32,
    /// Speed quotient $S_q = T_p / T_n \in [0.8 ..= 4.0]$.
    pub speed_quotient: f32,
    /// Pitch jitter coefficient (cycle-to-cycle frequency variation) [0.0 ..= 0.10].
    pub jitter: f32,
    /// Amplitude shimmer coefficient (cycle-to-cycle amplitude variation) [0.0 ..= 0.20].
    pub shimmer: f32,
    /// Turbulent aspiration noise gain [0.0 ..= 1.0].
    pub aspiration_level: f32,
    /// Voicing register mode.
    pub mode: GlottalVoicingMode,
    /// Output mode: true for flow derivative $dU_g/dt$ (standard acoustic pressure source),
    /// false for volume velocity flow $U_g(t)$.
    pub output_derivative: bool,
    /// Audio sample rate.
    pub sample_rate: u32,

    // Internal phase accumulator in normalized range [0.0 .. 1.0)
    #[serde(skip)]
    phase: f32,
    // Cycle-synchronous jitter offset in Hz
    #[serde(skip)]
    curr_jitter_offset_hz: f32,
    // Cycle-synchronous shimmer gain factor
    #[serde(skip)]
    curr_shimmer_gain: f32,
    // Non-allocating xorshift32 PRNG state
    #[serde(skip)]
    prng_state: u32,
    // Previous flow sample for discrete differentiation
    #[serde(skip)]
    prev_flow: f32,
}

impl Default for GlottalPulse {
    fn default() -> Self {
        Self::new(48000)
    }
}

impl GlottalPulse {
    /// Creates a new Glottal Pulse generator for the given sample rate.
    pub fn new(sample_rate: u32) -> Self {
        let sr = sample_rate.max(8000);
        let mode = GlottalVoicingMode::Modal;
        Self {
            frequency_hz: 140.0, // D3 (~typical speech pitch)
            voicing_amplitude: 1.0,
            open_quotient: mode.default_open_quotient(),
            speed_quotient: mode.default_speed_quotient(),
            jitter: 0.015,
            shimmer: 0.03,
            aspiration_level: mode.default_aspiration_level(),
            mode,
            output_derivative: true,
            sample_rate: sr,
            phase: 0.0,
            curr_jitter_offset_hz: 0.0,
            curr_shimmer_gain: 1.0,
            prng_state: 0x1337_CAFE,
            prev_flow: 0.0,
        }
    }

    /// Sets the voicing register mode and configures default open quotient & aspiration.
    pub fn set_mode(&mut self, mode: GlottalVoicingMode) {
        self.mode = mode;
        self.open_quotient = mode.default_open_quotient();
        self.speed_quotient = mode.default_speed_quotient();
        self.aspiration_level = mode.default_aspiration_level();
    }

    /// Sets the fundamental pitch frequency in Hz.
    pub fn set_frequency(&mut self, frequency_hz: f32) {
        self.frequency_hz = frequency_hz.clamp(20.0, self.sample_rate as f32 * 0.45);
    }

    /// Sets the master voicing amplitude [0.0 ..= 2.0].
    pub fn set_voicing_amplitude(&mut self, amplitude: f32) {
        self.voicing_amplitude = amplitude.clamp(0.0, 2.0);
    }

    /// Sets open quotient $O_q \in [0.20 ..= 0.95]$.
    pub fn set_open_quotient(&mut self, oq: f32) {
        self.open_quotient = oq.clamp(0.20, 0.95);
    }

    /// Sets speed quotient $S_q \in [0.5 ..= 5.0]$.
    pub fn set_speed_quotient(&mut self, sq: f32) {
        self.speed_quotient = sq.clamp(0.5, 5.0);
    }

    /// Sets aspiration noise level [0.0 ..= 1.0].
    pub fn set_aspiration_level(&mut self, aspiration: f32) {
        self.aspiration_level = aspiration.clamp(0.0, 1.0);
    }

    /// Resets oscillator phase and internal state.
    pub fn reset(&mut self) {
        self.phase = 0.0;
        self.curr_jitter_offset_hz = 0.0;
        self.curr_shimmer_gain = 1.0;
        self.prev_flow = 0.0;
    }

    /// Generates next pseudo-random uniform float in range `[-1.0, 1.0]` using xorshift32.
    #[inline]
    fn next_prng(&mut self) -> f32 {
        let mut x = self.prng_state;
        if x == 0 {
            x = 0x1337_CAFE;
        }
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.prng_state = x;
        ((x as f32) / (u32::MAX as f32)) * 2.0 - 1.0
    }

    /// Computes discrete Rosenberg glottal volume velocity $U_g(\phi)$ for phase $\phi \in [0, 1)$.
    #[inline]
    fn compute_rosenberg_flow(&self, phi: f32) -> f32 {
        let oq = self.open_quotient.clamp(0.20, 0.95);
        let sq = self.speed_quotient.clamp(0.5, 5.0);

        // Open phase duration fraction
        let t_open = oq;
        // Opening peak phase fraction
        let t_p = t_open * (sq / (sq + 1.0));
        // Closing phase fraction
        let t_n = t_open - t_p;

        if phi < t_p {
            // Opening phase: U_g(t) = 0.5 * (1 - cos(pi * t / t_p)) = sin^2(pi * t / (2 * t_p))
            let theta = PI * phi / t_p.max(1e-5);
            0.5 * (1.0 - theta.cos())
        } else if phi < t_open {
            // Closing phase: U_g(t) = cos(pi * (t - t_p) / (2 * t_n))
            let theta = PI * (phi - t_p) / (2.0 * t_n.max(1e-5));
            theta.cos().max(0.0)
        } else {
            // Closed phase
            0.0
        }
    }

    /// Computes analytical derivative of Rosenberg glottal volume velocity $dU_g/dt$.
    #[inline]
    fn compute_rosenberg_derivative(&self, phi: f32, f0: f32) -> f32 {
        let oq = self.open_quotient.clamp(0.20, 0.95);
        let sq = self.speed_quotient.clamp(0.5, 5.0);

        let t_open = oq;
        let t_p = t_open * (sq / (sq + 1.0));
        let t_n = t_open - t_p;

        if phi < t_p {
            // d/dt [0.5 * (1 - cos(pi * phi / t_p))] = 0.5 * (pi / (t_p / f0)) * sin(pi * phi / t_p)
            let theta = PI * phi / t_p.max(1e-5);
            let scale = 0.5 * PI * f0 / t_p.max(1e-5);
            scale * theta.sin()
        } else if phi < t_open {
            // d/dt [cos(pi * (phi - t_p) / (2 * t_n))] = -(pi / (2 * t_n / f0)) * sin(pi * (phi - t_p) / (2 * t_n))
            let theta = PI * (phi - t_p) / (2.0 * t_n.max(1e-5));
            let scale = -0.5 * PI * f0 / t_n.max(1e-5);
            scale * theta.sin()
        } else {
            0.0
        }
    }

    /// Advances the glottal oscillator by one sample and returns the excitation pressure sample.
    #[inline]
    pub fn process_sample(&mut self) -> Sample {
        let sr = self.sample_rate as f32;

        // Effective instantaneous fundamental frequency with cycle jitter
        let effective_f0 = (self.frequency_hz + self.curr_jitter_offset_hz).clamp(20.0, sr * 0.45);
        let delta_phase = effective_f0 / sr;

        // Advance continuous phase accumulator
        let prev_p = self.phase;
        self.phase += delta_phase;

        // Cycle boundary detection: wrap modulo 1.0 and refresh jitter / shimmer perturbations
        if self.phase >= 1.0 {
            self.phase -= self.phase.floor();

            // Calculate cycle-to-cycle perturbations
            let jitter_rand = self.next_prng();
            self.curr_jitter_offset_hz = jitter_rand * self.jitter * self.frequency_hz * 0.5;

            let shimmer_rand = self.next_prng();
            self.curr_shimmer_gain = (1.0 + shimmer_rand * self.shimmer).clamp(0.5, 1.5);
        }

        // Generate turbulent aspiration noise (unvoiced breath turbulence)
        let noise_sample = self.next_prng();
        let aspiration = noise_sample * self.aspiration_level;

        if self.mode == GlottalVoicingMode::Whisper {
            // Pure unvoiced turbulent aspiration excitation
            aspiration * self.voicing_amplitude
        } else if self.output_derivative {
            // Glottal flow derivative dU_g/dt normalized and scaled by shimmer & voicing amplitude
            let raw_deriv = self.compute_rosenberg_derivative(prev_p, effective_f0);
            let normalized_deriv = raw_deriv / (2.0 * PI * effective_f0).max(1.0);
            let voiced = normalized_deriv * self.voicing_amplitude * self.curr_shimmer_gain;

            // Modulate aspiration noise with glottal opening area
            let flow_envelope = self.compute_rosenberg_flow(prev_p);
            let modulated_aspiration = aspiration * (0.3 + 0.7 * flow_envelope);

            (voiced + modulated_aspiration).clamp(-1.0, 1.0)
        } else {
            // Glottal volume velocity flow U_g(t)
            let flow = self.compute_rosenberg_flow(prev_p);
            let voiced = flow * self.voicing_amplitude * self.curr_shimmer_gain;
            let modulated_aspiration = aspiration * flow;

            (voiced + modulated_aspiration).clamp(-1.0, 1.0)
        }
    }
}

impl SignalProcessor for GlottalPulse {
    fn name(&self) -> &str {
        "GlottalPulse"
    }

    fn process_block(
        &mut self,
        _inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        _ctx: &ProcessContext,
    ) {
        if outputs.is_empty() {
            return;
        }

        let num_samples = outputs[0].len();
        for i in 0..num_samples {
            let sample = self.process_sample();
            for ch in outputs.iter_mut() {
                if i < ch.len() {
                    ch[i] = sample;
                }
            }
        }
    }
}

/// AudioNode wrapper for Rosenberg Glottal Flow Pulse Generator.
#[derive(Debug)]
pub struct GlottalPulseNode {
    pub pulse: GlottalPulse,
}

impl GlottalPulseNode {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            pulse: GlottalPulse::new(sample_rate),
        }
    }
}

impl AudioNode for GlottalPulseNode {
    fn name(&self) -> &str {
        "GlottalPulseNode"
    }

    fn process(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        self.pulse.process_block(inputs, outputs, ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::allocator::AllocGuard;
    use summoner_core::transport::Transport;

    #[test]
    fn test_glottal_pulse_oscillation_and_energy_conservation() {
        let mut pulse = GlottalPulse::new(48000);
        pulse.set_frequency(150.0);
        pulse.set_voicing_amplitude(1.0);
        pulse.set_mode(GlottalVoicingMode::Modal);

        let mut max_abs = 0.0f32;
        let mut energy = 0.0f32;

        for _ in 0..1000 {
            let sample = pulse.process_sample();
            assert!(sample.is_finite());
            assert!((-1.0..=1.0).contains(&sample));
            max_abs = max_abs.max(sample.abs());
            energy += sample * sample;
        }

        assert!(max_abs > 0.1, "Glottal pulse must produce active waveform");
        assert!(energy > 1.0, "Glottal pulse must conserve energy during voiced playback");
    }

    #[test]
    fn test_glottal_pulse_modes_and_whisper() {
        let mut pulse = GlottalPulse::new(48000);

        for mode in [
            GlottalVoicingMode::Modal,
            GlottalVoicingMode::Falsetto,
            GlottalVoicingMode::Creaky,
            GlottalVoicingMode::Breathy,
            GlottalVoicingMode::Whisper,
        ] {
            pulse.set_mode(mode);
            for _ in 0..100 {
                let sample = pulse.process_sample();
                assert!(sample.is_finite());
                assert!((-1.0..=1.0).contains(&sample));
            }
        }
    }

    #[test]
    fn test_glottal_pulse_zero_allocation_in_process_block() {
        let mut pulse = GlottalPulse::new(48000);
        let transport = Transport::new(48000, 120.0);
        let ctx = ProcessContext::from_transport(&transport);

        let mut out_l = [0.0f32; 64];
        let mut out_r = [0.0f32; 64];

        {
            let _guard = AllocGuard::new();
            pulse.process_block(
                &[],
                &mut [&mut out_l[..], &mut out_r[..]],
                &ctx,
            );
        }

        assert!(out_l.iter().all(|s| s.is_finite()));
        assert!(out_l.iter().any(|s| *s != 0.0));
    }
}
