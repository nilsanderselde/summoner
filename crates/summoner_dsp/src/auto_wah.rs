// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Dynamic State-Variable Auto-Wah Envelope Follower Filter (Milestone 23).
//!
//! Provides a real-time dynamic auto-wah filter with envelope follower ballistics,
//! non-linear state-variable bandpass/lowpass/peaking filter modes, adjustable resonance $Q$,
//! dynamic frequency sweep range, saturating feedback, and dry/wet blend.
//!
//! Enforces zero heap allocations in audio processing loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};
use crate::traits::SignalProcessor;

/// Auto-wah filter response type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AutoWahFilterMode {
    /// Resonant Bandpass (Classic funk wah pedal sound).
    #[default]
    Bandpass,
    /// Resonant Lowpass (Fat synth-like envelope sweep).
    Lowpass,
    /// Peaking Bell (Subtle vocal vowel wah).
    PeakingWah,
}

/// Auto-wah sweep direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AutoWahDirection {
    /// Positive sweep (louder strike sweeps cutoff higher).
    #[default]
    Up,
    /// Inverted sweep (louder strike sweeps cutoff downwards).
    Down,
}

/// Dynamic State-Variable Auto-Wah Filter.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AutoWah {
    /// Effect enabled.
    pub enabled: bool,
    /// Envelope sensitivity $[0.0 ..= 1.0]$.
    pub sensitivity: f32,
    /// Base resting cutoff frequency in Hz $[50.0 ..= 4000.0\text{Hz}]$.
    pub base_freq_hz: f32,
    /// Maximum frequency sweep range in octaves $[0.5 ..= 6.0]$.
    pub sweep_range_octaves: f32,
    /// Filter resonance $Q$ factor $[0.5 ..= 25.0]$.
    pub resonance_q: f32,
    /// Filter topology mode.
    pub filter_mode: AutoWahFilterMode,
    /// Sweep direction (Up / Down).
    pub direction: AutoWahDirection,
    /// Envelope attack time in milliseconds $[0.5 ..= 100.0\text{ms}]$.
    pub attack_ms: f32,
    /// Envelope release time in milliseconds $[5.0 ..= 1000.0\text{ms}]$.
    pub release_ms: f32,
    /// Non-linear saturation drive in dB $[0.0 ..= 18.0\text{dB}]$.
    pub drive_db: f32,
    /// Dry / Wet mix balance $[0.0 ..= 1.0]$.
    pub mix: f32,
    /// Envelope follower state (Left channel).
    env_l: f32,
    /// Envelope follower state (Right channel).
    env_r: f32,
    /// SVF internal state variables (Left channel).
    s1_l: f32,
    s2_l: f32,
    /// SVF internal state variables (Right channel).
    s1_r: f32,
    s2_r: f32,
    /// Instantaneous smoothed cutoff frequency for GUI visualizers.
    pub current_cutoff_hz: f32,
}

impl Default for AutoWah {
    fn default() -> Self {
        Self {
            enabled: true,
            sensitivity: 0.70,
            base_freq_hz: 350.0,
            sweep_range_octaves: 3.5,
            resonance_q: 6.5,
            filter_mode: AutoWahFilterMode::Bandpass,
            direction: AutoWahDirection::Up,
            attack_ms: 8.0,
            release_ms: 120.0,
            drive_db: 4.0,
            mix: 0.85,
            env_l: 0.0,
            env_r: 0.0,
            s1_l: 0.0,
            s2_l: 0.0,
            s1_r: 0.0,
            s2_r: 0.0,
            current_cutoff_hz: 350.0,
        }
    }
}

impl AutoWah {
    /// Creates a new AutoWah filter instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets envelope follower ballistics.
    pub fn set_ballistics(&mut self, attack_ms: f32, release_ms: f32) {
        self.attack_ms = attack_ms.clamp(0.5, 100.0);
        self.release_ms = release_ms.clamp(5.0, 1000.0);
    }

    /// Sets tuning and resonance parameters.
    pub fn set_params(
        &mut self,
        sensitivity: f32,
        base_freq_hz: f32,
        sweep_range_octaves: f32,
        resonance_q: f32,
        mix: f32,
    ) {
        self.sensitivity = sensitivity.clamp(0.0, 1.0);
        self.base_freq_hz = base_freq_hz.clamp(50.0, 4000.0);
        self.sweep_range_octaves = sweep_range_octaves.clamp(0.5, 6.0);
        self.resonance_q = resonance_q.clamp(0.5, 25.0);
        self.mix = mix.clamp(0.0, 1.0);
    }

    /// Processes a stereo sample pair through the dynamic auto-wah.
    #[inline]
    pub fn process_sample(&mut self, in_l: f32, in_r: f32, sample_rate: u32) -> (f32, f32) {
        if !self.enabled || self.mix < 0.001 {
            return (in_l, in_r);
        }

        let sr = sample_rate.max(8000) as f32;

        // Ballistics coefficients
        let att_coef = (-1.0 / (self.attack_ms * 0.001 * sr).max(1.0)).exp();
        let rel_coef = (-1.0 / (self.release_ms * 0.001 * sr).max(1.0)).exp();

        // Envelope detection (peak rectifier with decoupled attack/release)
        let abs_l = in_l.abs();
        let abs_r = in_r.abs();

        if abs_l > self.env_l {
            self.env_l = abs_l + att_coef * (self.env_l - abs_l);
        } else {
            self.env_l = abs_l + rel_coef * (self.env_l - abs_l);
        }

        if abs_r > self.env_r {
            self.env_r = abs_r + att_coef * (self.env_r - abs_r);
        } else {
            self.env_r = abs_r + rel_coef * (self.env_r - abs_r);
        }

        let combined_env = (self.env_l + self.env_r) * 0.5;
        let mod_env = (combined_env * self.sensitivity * 3.0).clamp(0.0, 1.0);

        // Calculate dynamic cutoff frequency
        let octave_shift = match self.direction {
            AutoWahDirection::Up => mod_env * self.sweep_range_octaves,
            AutoWahDirection::Down => -mod_env * self.sweep_range_octaves,
        };

        let target_cutoff = (self.base_freq_hz * 2.0f32.powf(octave_shift)).clamp(60.0, sr * 0.45);
        self.current_cutoff_hz = target_cutoff;

        // Cytomic State Variable Filter (SVF) discretization
        // g = tan(pi * fc / fs)
        let g = (PI * target_cutoff / sr).tan().clamp(0.0001, 15.0);
        let k = 1.0 / self.resonance_q.clamp(0.5, 25.0);
        let a1 = 1.0 / (1.0 + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;

        let drive_gain = 10.0f32.powf(self.drive_db / 20.0);

        // Process Left Channel
        let x_l = in_l * drive_gain;
        let v0_l = x_l;
        let v1_l = a1 * self.s1_l + a2 * (v0_l - self.s2_l);
        let v2_l = self.s2_l + a2 * self.s1_l + a3 * (v0_l - self.s2_l);

        // Non-linear soft saturation in the integrator loop
        let sat_v1_l = self.saturate(v1_l);
        self.s1_l = 2.0 * sat_v1_l - self.s1_l;
        self.s2_l = 2.0 * v2_l - self.s2_l;

        let lp_l = v2_l;
        let bp_l = sat_v1_l;
        let hp_l = v0_l - k * bp_l - lp_l;

        let filtered_l = match self.filter_mode {
            AutoWahFilterMode::Bandpass => bp_l * (1.0 + (self.resonance_q * 0.2).sqrt()),
            AutoWahFilterMode::Lowpass => lp_l,
            AutoWahFilterMode::PeakingWah => in_l + bp_l * 1.5,
        };

        // Process Right Channel
        let x_r = in_r * drive_gain;
        let v0_r = x_r;
        let v1_r = a1 * self.s1_r + a2 * (v0_r - self.s2_r);
        let v2_r = self.s2_r + a2 * self.s1_r + a3 * (v0_r - self.s2_r);

        let sat_v1_r = self.saturate(v1_r);
        self.s1_r = 2.0 * sat_v1_r - self.s1_r;
        self.s2_r = 2.0 * v2_r - self.s2_r;

        let lp_r = v2_r;
        let bp_r = sat_v1_r;
        let hp_r = v0_r - k * bp_r - lp_r;
        let _ = hp_l; // Ensure variables are used
        let _ = hp_r;

        let filtered_r = match self.filter_mode {
            AutoWahFilterMode::Bandpass => bp_r * (1.0 + (self.resonance_q * 0.2).sqrt()),
            AutoWahFilterMode::Lowpass => lp_r,
            AutoWahFilterMode::PeakingWah => in_r + bp_r * 1.5,
        };

        let out_l = in_l * (1.0 - self.mix) + (filtered_l / drive_gain) * self.mix;
        let out_r = in_r * (1.0 - self.mix) + (filtered_r / drive_gain) * self.mix;

        (out_l.clamp(-2.0, 2.0), out_r.clamp(-2.0, 2.0))
    }

    #[inline]
    fn saturate(&self, x: f32) -> f32 {
        // Symmetrical rational soft clipper
        if x > 0.0 {
            x / (1.0 + x * 0.4)
        } else {
            let nx = -x;
            -(nx / (1.0 + nx * 0.4))
        }
    }

    /// Returns the active envelope follower amplitude $[0.0 ..= 1.0]$.
    pub fn get_envelope(&self) -> f32 {
        ((self.env_l + self.env_r) * 0.5).min(1.0)
    }

    /// Resets all internal filters and envelope memory.
    pub fn reset(&mut self) {
        self.env_l = 0.0;
        self.env_r = 0.0;
        self.s1_l = 0.0;
        self.s2_l = 0.0;
        self.s1_r = 0.0;
        self.s2_r = 0.0;
        self.current_cutoff_hz = self.base_freq_hz;
    }
}

impl SignalProcessor for AutoWah {
    fn name(&self) -> &str {
        "AutoWah"
    }

    fn process_block(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        _context: &ProcessContext,
    ) {
        if outputs.is_empty() {
            return;
        }

        let num_samples = outputs[0].len();
        let is_stereo = outputs.len() >= 2;

        let in_l = if !inputs.is_empty() { inputs[0] } else { &[] };
        let in_r = if inputs.len() >= 2 { inputs[1] } else { in_l };

        for i in 0..num_samples {
            let l_in = if i < in_l.len() { in_l[i] } else { 0.0 };
            let r_in = if i < in_r.len() { in_r[i] } else { l_in };

            let (l_out, r_out) = self.process_sample(l_in, r_in, 48000);
            outputs[0][i] = l_out;
            if is_stereo {
                outputs[1][i] = r_out;
            }
        }
    }
}

impl AudioNode for AutoWah {
    fn name(&self) -> &str {
        "AutoWahNode"
    }

    fn process(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        context: &ProcessContext,
    ) {
        self.process_block(inputs, outputs, context);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::allocator::AllocGuard;
    use summoner_core::transport::Transport;

    #[test]
    fn test_auto_wah_zero_allocation() {
        let mut wah = AutoWah::new();
        wah.set_params(0.85, 400.0, 3.5, 8.0, 1.0);

        let transport = Transport::new(48000, 120.0);
        let ctx = ProcessContext::from_transport(&transport);

        let in_l = [0.5f32; 64];
        let in_r = [0.5f32; 64];
        let mut out_l = [0.0f32; 64];
        let mut out_r = [0.0f32; 64];

        {
            let _guard = AllocGuard::new();
            for _ in 0..32 {
                wah.process_block(
                    &[&in_l[..], &in_r[..]],
                    &mut [&mut out_l[..], &mut out_r[..]],
                    &ctx,
                );
                for i in 0..64 {
                    assert!(out_l[i].is_finite());
                    assert!(out_r[i].is_finite());
                }
            }
        }
    }

    #[test]
    fn test_auto_wah_dynamic_sweep() {
        let mut wah = AutoWah::new();
        wah.set_params(0.9, 300.0, 4.0, 6.0, 1.0);

        // Low level input -> low cutoff
        for _ in 0..100 {
            wah.process_sample(0.01, 0.01, 48000);
        }
        let low_cutoff = wah.current_cutoff_hz;

        // High burst input -> high cutoff
        for _ in 0..100 {
            wah.process_sample(0.9, 0.9, 48000);
        }
        let high_cutoff = wah.current_cutoff_hz;

        assert!(
            high_cutoff > low_cutoff,
            "AutoWah should sweep cutoff upwards on high amplitude input: high={} > low={}",
            high_cutoff,
            low_cutoff
        );
    }
}
