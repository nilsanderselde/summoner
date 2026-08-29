// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Analog Console Desk Emulation & Transformer Character Modeler (Milestone 11).
//!
//! Provides accurate physical simulation of classic analog mixing console desks:
//! - Neve 8048 / 1073: Marinair/Carnhill input/output transformer saturation, low-end resonant weight (~50 Hz bump), silky HF smoothing.
//! - SSL 4000 E/G: VCA channel drive, punchy bass, crisp high-frequency sheen (~10 kHz presence), fast dynamic transients.
//! - API 1608 / 2520: Discrete op-amp and AP2503 steel transformer bite, forward midrange punch (~2.5 kHz), aggressive asymmetric saturation.
//!
//! Enforces zero heap allocations in real-time audio render loops under `AllocGuard`.

use crate::traits::SignalProcessor;
use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};

/// Console desk emulation mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ConsoleMode {
    /// Neve 8048 / 1073: British Class A console with thick transformer weight, 2nd harmonics, LF bump.
    #[default]
    Neve = 0,
    /// SSL 4000 E/G: British VCA console with punchy transient clarity and top-end sheen.
    SSL = 1,
    /// API 1608 / 2520: American discrete console with forward midrange bite and transformer punch.
    API = 2,
}

impl ConsoleMode {
    pub fn from_f32(val: f32) -> Self {
        match val.round() as i32 {
            0 => ConsoleMode::Neve,
            1 => ConsoleMode::SSL,
            _ => ConsoleMode::API,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ConsoleMode::Neve => "NEVE 8048 / 1073 (VINTAGE CLASS-A)",
            ConsoleMode::SSL => "SSL 4000 E/G (VCA PUNCH & SHEEN)",
            ConsoleMode::API => "API 1608 / 2520 (DISCRETE MID-PUNCH)",
        }
    }

    /// Center frequency of transformer resonance in Hz.
    pub fn transformer_center_hz(&self) -> f32 {
        match self {
            ConsoleMode::Neve => 52.0,
            ConsoleMode::SSL => 90.0,
            ConsoleMode::API => 2500.0,
        }
    }
}

/// 2nd-order biquad peaking filter for desk transformer response.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ConsoleBiquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    z1: f32,
    z2: f32,
}

impl ConsoleBiquad {
    pub fn reset(&mut self) {
        self.z1 = 0.0;
        self.z2 = 0.0;
    }

    pub fn update_peaking(&mut self, freq_hz: f32, gain_db: f32, q: f32, sr: f32) {
        let sr = sr.max(1000.0);
        let fc = freq_hz.clamp(10.0, sr * 0.45);
        let w0 = TAU * fc / sr;
        let cos_w = w0.cos();
        let sin_w = w0.sin();
        let a = 10.0f32.powf(gain_db / 40.0);
        let alpha = sin_w / (2.0 * q.max(0.1));

        let a0 = 1.0 + alpha / a;
        self.b0 = (1.0 + alpha * a) / a0;
        self.b1 = (-2.0 * cos_w) / a0;
        self.b2 = (1.0 - alpha * a) / a0;
        self.a1 = (-2.0 * cos_w) / a0;
        self.a2 = (1.0 - alpha / a) / a0;
    }

    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.z1;
        self.z1 = self.b1 * x - self.a1 * y + self.z2;
        self.z2 = self.b2 * x - self.a2 * y;
        y
    }
}

/// Internal DSP state for a single audio channel of console emulation.
#[derive(Debug, Clone, Copy, Default)]
pub struct ConsoleChannelState {
    pub transformer_filter: ConsoleBiquad,
    pub iron_flux: f32,
    pub dc_x1: f32,
    pub dc_y1: f32,
    pub hf_state: f32,
}

impl ConsoleChannelState {
    pub fn reset(&mut self) {
        self.transformer_filter.reset();
        self.iron_flux = 0.0;
        self.dc_x1 = 0.0;
        self.dc_y1 = 0.0;
        self.hf_state = 0.0;
    }
}

/// Comprehensive Analog Console Desk Emulation DSP Node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleEmulationNode {
    /// Desk emulation model mode.
    pub mode: ConsoleMode,
    /// Channel drive gain multiplier (0.0 to 10.0, default 1.5).
    pub drive: f32,
    /// Warmth / transformer saturation emphasis (0.0 to 1.0, default 0.6).
    pub warmth: f32,
    /// Transformer iron core hysteresis non-linearity (0.0 to 1.0, default 0.5).
    pub transformer_iron: f32,
    /// Channel crosstalk bleed factor (0.0 to 1.0, default 0.05).
    pub crosstalk: f32,
    /// Output makeup gain (0.1 to 4.0, default 1.0).
    pub output_gain: f32,
    /// Dry/Wet blend ratio (0.0 to 1.0, default 1.0).
    pub dry_wet: f32,

    #[serde(skip)]
    channels: [ConsoleChannelState; 2],
    #[serde(skip)]
    sample_rate: f32,
}

impl ConsoleEmulationNode {
    /// Create a new `ConsoleEmulationNode` with specified mode and drive.
    pub fn new(mode: ConsoleMode, drive: f32) -> Self {
        let mut node = Self {
            mode,
            drive: drive.max(0.0),
            warmth: 0.6,
            transformer_iron: 0.5,
            crosstalk: 0.05,
            output_gain: 1.0,
            dry_wet: 1.0,
            channels: [ConsoleChannelState::default(), ConsoleChannelState::default()],
            sample_rate: 44100.0,
        };
        node.update_filters();
        node
    }

    /// Builder pattern configuring console mode.
    pub fn with_mode(mut self, mode: ConsoleMode) -> Self {
        self.mode = mode;
        self.update_filters();
        self
    }

    /// Reset internal filter and flux states across all channels.
    pub fn reset(&mut self) {
        for ch in &mut self.channels {
            ch.reset();
        }
        self.update_filters();
    }

    /// Set operating sample rate and refresh filters.
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1000.0);
        self.update_filters();
    }

    /// Recompute peaking biquad filters matching the active console mode.
    pub fn update_filters(&mut self) {
        let sr = self.sample_rate;
        match self.mode {
            ConsoleMode::Neve => {
                // Neve: +2.5 dB LF transformer bump at 52 Hz (Q = 1.1)
                let gain_db = 2.5 * self.warmth;
                for ch in &mut self.channels {
                    ch.transformer_filter.update_peaking(52.0, gain_db, 1.1, sr);
                }
            }
            ConsoleMode::SSL => {
                // SSL: +1.8 dB high sheen presence boost at 10 kHz (Q = 0.8)
                let gain_db = 1.8 * self.warmth;
                for ch in &mut self.channels {
                    ch.transformer_filter.update_peaking(10000.0, gain_db, 0.8, sr);
                }
            }
            ConsoleMode::API => {
                // API: +2.2 dB midrange punch at 2.5 kHz (Q = 1.4)
                let gain_db = 2.2 * self.warmth;
                for ch in &mut self.channels {
                    ch.transformer_filter.update_peaking(2500.0, gain_db, 1.4, sr);
                }
            }
        }
    }

    /// Evaluates non-linear desk transformer saturation and harmonic distortion.
    #[inline]
    pub fn eval_console_sample(
        mode: ConsoleMode,
        drive: f32,
        warmth: f32,
        iron: f32,
        x: f32,
        state: &mut ConsoleChannelState,
        sr: f32,
    ) -> f32 {
        let effective_drive = 1.0 + drive * 1.6;
        let shaped = state.transformer_filter.process(x);
        let driven = shaped * effective_drive;

        // Non-linear transformer core saturation
        let sat_out = match mode {
            ConsoleMode::Neve => {
                // Marinair transformer saturation: Soft 2nd-order asymmetric saturation with iron flux memory
                let iron_diff = driven - state.iron_flux * 0.25 * iron;
                state.iron_flux += 0.3 * (iron_diff.tanh() - state.iron_flux);
                let asym = driven + 0.15 * warmth * driven.abs();
                (asym * 1.15).tanh() + state.iron_flux * 0.1 * iron
            }
            ConsoleMode::SSL => {
                // VCA Drive: Crisp, fast arctangent saturation with high presence
                let vca_sat = (driven * 1.1).atan() * 1.15;
                // High frequency sheen filter
                let hf_alpha = (TAU * 12000.0 / sr).clamp(0.1, 0.9);
                state.hf_state += hf_alpha * (vca_sat - state.hf_state);
                vca_sat + (vca_sat - state.hf_state) * 0.15 * warmth
            }
            ConsoleMode::API => {
                // Discrete 2520 Op-Amp + AP2503 steel transformer bite: Asymmetric cubic saturation
                let asym = driven + 0.18 * driven * driven.abs();
                let iron_core = asym - 0.08 * asym.powi(3) / (1.0 + asym.powi(2));
                (iron_core * 1.25).tanh()
            }
        };

        // DC Blocker to prevent drift
        let r = 0.996f32;
        let dc_out = sat_out - state.dc_x1 + r * state.dc_y1;
        state.dc_x1 = sat_out;
        state.dc_y1 = dc_out;

        dc_out / (1.0 + drive * 0.4)
    }
}

impl Default for ConsoleEmulationNode {
    fn default() -> Self {
        Self::new(ConsoleMode::Neve, 1.5)
    }
}

impl SignalProcessor for ConsoleEmulationNode {
    fn name(&self) -> &str {
        "ConsoleEmulationNode"
    }

    fn process_block(
        &mut self,
        input: &[&[Sample]],
        output: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        if input.is_empty() || output.is_empty() {
            return;
        }

        let num_samples = input[0].len().min(output[0].len());
        let sr = if ctx.sample_rate > 0 {
            ctx.sample_rate as f32
        } else {
            self.sample_rate
        };

        if (self.sample_rate - sr).abs() > 1.0 {
            self.sample_rate = sr;
            self.update_filters();
        }

        let num_in_ch = input.len();
        let num_out_ch = output.len();

        let mode = self.mode;
        let drive = self.drive;
        let warmth = self.warmth;
        let iron = self.transformer_iron;
        let crosstalk = self.crosstalk * 0.02; // -54 dB bleed
        let output_gain = self.output_gain;
        let dry_wet = self.dry_wet;

        for i in 0..num_samples {
            // Read inputs with optional stereo crosstalk
            let in_l = if !input[0].is_empty() && i < input[0].len() {
                input[0][i]
            } else {
                0.0
            };
            let in_r = if num_in_ch > 1 && i < input[1].len() {
                input[1][i]
            } else {
                in_l
            };

            let mixed_l = in_l + in_r * crosstalk;
            let mixed_r = in_r + in_l * crosstalk;

            let out_l_proc = Self::eval_console_sample(
                mode,
                drive,
                warmth,
                iron,
                mixed_l,
                &mut self.channels[0],
                sr,
            );
            let out_r_proc = Self::eval_console_sample(
                mode,
                drive,
                warmth,
                iron,
                mixed_r,
                &mut self.channels[1],
                sr,
            );

            let final_l = in_l * (1.0 - dry_wet) + (out_l_proc * output_gain) * dry_wet;
            let final_r = in_r * (1.0 - dry_wet) + (out_r_proc * output_gain) * dry_wet;

            if !output[0].is_empty() && i < output[0].len() {
                output[0][i] = final_l;
            }
            if num_out_ch > 1 && !output[1].is_empty() && i < output[1].len() {
                output[1][i] = final_r;
            }
        }
    }
}

impl AudioNode for ConsoleEmulationNode {
    fn name(&self) -> &str {
        "ConsoleEmulationNode"
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

    #[test]
    fn test_console_emulation_zero_allocation_in_audio_loop() {
        let mut console = ConsoleEmulationNode::new(ConsoleMode::Neve, 2.0);
        let ctx = ProcessContext::new(48000, 120.0, 0);

        let in_l = [0.5f32; 128];
        let in_r = [-0.5f32; 128];
        let mut out_l = [0.0f32; 128];
        let mut out_r = [0.0f32; 128];

        {
            let _guard = AllocGuard::new();
            console.process_block(
                &[&in_l[..], &in_r[..]],
                &mut [&mut out_l[..], &mut out_r[..]],
                &ctx,
            );
        }

        assert!(out_l.iter().all(|s| s.is_finite()));
        assert!(out_r.iter().all(|s| s.is_finite()));
    }

    #[test]
    fn test_console_emulation_modes_distinct_responses() {
        let ctx = ProcessContext::new(48000, 120.0, 0);
        let input_sig = [0.4f32; 128];

        let mut neve = ConsoleEmulationNode::new(ConsoleMode::Neve, 2.5);
        let mut ssl = ConsoleEmulationNode::new(ConsoleMode::SSL, 2.5);
        let mut api = ConsoleEmulationNode::new(ConsoleMode::API, 2.5);

        let mut out_neve = [0.0f32; 128];
        let mut out_ssl = [0.0f32; 128];
        let mut out_api = [0.0f32; 128];

        neve.process_block(&[&input_sig[..]], &mut [&mut out_neve[..]], &ctx);
        ssl.process_block(&[&input_sig[..]], &mut [&mut out_ssl[..]], &ctx);
        api.process_block(&[&input_sig[..]], &mut [&mut out_api[..]], &ctx);

        assert!(out_neve[127].is_finite());
        assert!(out_ssl[127].is_finite());
        assert!(out_api[127].is_finite());

        assert!(
            (out_neve[127] - out_ssl[127]).abs() > 0.001,
            "Neve and SSL modes should have distinct transfer characteristics"
        );
        assert!(
            (out_neve[127] - out_api[127]).abs() > 0.001,
            "Neve and API modes should have distinct transfer characteristics"
        );
    }
}
