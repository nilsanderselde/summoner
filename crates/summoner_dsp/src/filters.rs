// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.

//! Atomic filter DSP primitives (Moog Ladder, State Variable Filter, Comb Filter).

use crate::traits::SignalProcessor;
use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;
use std::f32::consts::PI;

/// 4-pole (24dB/octave) Moog-style nonlinear ladder filter.
#[derive(Debug)]
pub struct FilterLadder {
    pub cutoff: f32,
    pub resonance: f32,
    stage: [f32; 4],
    stage_tanh: [f32; 3],
}

impl FilterLadder {
    pub fn new(cutoff: f32, resonance: f32) -> Self {
        Self {
            cutoff,
            resonance: resonance.clamp(0.0, 4.0),
            stage: [0.0; 4],
            stage_tanh: [0.0; 3],
        }
    }

    pub fn process_sample(&mut self, input: f32, sample_rate: u32) -> f32 {
        if sample_rate == 0 {
            return 0.0;
        }

        let input = if input.is_finite() { input.clamp(-100.0, 100.0) } else { 0.0 };
        if !self.stage[0].is_finite() {
            self.stage = [0.0; 4];
            self.stage_tanh = [0.0; 3];
        }

        let cutoff_norm = (self.cutoff / sample_rate as f32).clamp(0.0001, 0.49);
        let f = (PI * cutoff_norm).sin();
        let k = self.resonance;

        // Feedback calculation
        let res_input = input - 4.0 * k * self.stage[3];
        let input_tanh = res_input.tanh();

        self.stage[0] += f * (input_tanh - self.stage_tanh[0]);
        self.stage_tanh[0] = self.stage[0].tanh();

        self.stage[1] += f * (self.stage_tanh[0] - self.stage_tanh[1]);
        self.stage_tanh[1] = self.stage[1].tanh();

        self.stage[2] += f * (self.stage_tanh[1] - self.stage_tanh[2]);
        self.stage_tanh[2] = self.stage[2].tanh();

        self.stage[3] += f * (self.stage_tanh[2] - self.stage[3].tanh());

        if !self.stage[3].is_finite() {
            self.stage = [0.0; 4];
            self.stage_tanh = [0.0; 3];
            0.0
        } else {
            self.stage[3]
        }
    }
}

impl SignalProcessor for FilterLadder {
    fn name(&self) -> &str {
        "FilterLadder"
    }

    fn process_block(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        if ctx.sample_rate == 0 || outputs.is_empty() {
            return;
        }

        let num_samples = outputs[0].len();
        for i in 0..num_samples {
            let in_sample = if !inputs.is_empty() && !inputs[0].is_empty() && i < inputs[0].len() {
                inputs[0][i]
            } else {
                0.0
            };

            let out_sample = self.process_sample(in_sample, ctx.sample_rate);
            for out_ch in outputs.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = out_sample;
                }
            }
        }
    }
}

/// State Variable Filter (SVF) outputting Lowpass, Highpass, and Bandpass.
#[derive(Debug)]
pub struct FilterSVF {
    pub cutoff: f32,
    pub resonance: f32,
    ic1eq: f32,
    ic2eq: f32,
}

impl FilterSVF {
    pub fn new(cutoff: f32, resonance: f32) -> Self {
        Self {
            cutoff,
            resonance: resonance.max(0.1),
            ic1eq: 0.0,
            ic2eq: 0.0,
        }
    }

    pub fn process_sample(&mut self, input: f32, sample_rate: u32) -> (f32, f32, f32) {
        if sample_rate == 0 {
            return (0.0, 0.0, 0.0);
        }

        let input = if input.is_finite() { input.clamp(-100.0, 100.0) } else { 0.0 };
        if !self.ic1eq.is_finite() { self.ic1eq = 0.0; }
        if !self.ic2eq.is_finite() { self.ic2eq = 0.0; }

        let g = (PI * (self.cutoff / sample_rate as f32).clamp(0.0001, 0.49)).tan();
        let k = 1.0 / self.resonance;

        let a1 = 1.0 / (1.0 + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;

        let v3 = input - self.ic2eq;
        let v1 = a1 * self.ic1eq + a2 * v3;
        let v2 = self.ic2eq + a2 * self.ic1eq + a3 * v3;

        self.ic1eq = 2.0 * v1 - self.ic1eq;
        self.ic2eq = 2.0 * v2 - self.ic2eq;

        let lowpass = if v2.is_finite() { v2 } else { 0.0 };
        let bandpass = if v1.is_finite() { v1 } else { 0.0 };
        let highpass = if (input - k * v1 - v2).is_finite() { input - k * v1 - v2 } else { 0.0 };

        (lowpass, highpass, bandpass)
    }
}

impl SignalProcessor for FilterSVF {
    fn name(&self) -> &str {
        "FilterSVF"
    }

    fn process_block(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        if ctx.sample_rate == 0 || outputs.is_empty() {
            return;
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        {
            self.process_block_simd(inputs, outputs, ctx);
        }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            let num_samples = outputs[0].len();
            for i in 0..num_samples {
                let in_sample = if !inputs.is_empty() && !inputs[0].is_empty() && i < inputs[0].len() {
                    inputs[0][i]
                } else {
                    0.0
                };

                let (lp, hp, bp) = self.process_sample(in_sample, ctx.sample_rate);
                if !outputs.is_empty() && i < outputs[0].len() {
                    outputs[0][i] = lp;
                }
                if outputs.len() > 1 && i < outputs[1].len() {
                    outputs[1][i] = hp;
                }
                if outputs.len() > 2 && i < outputs[2].len() {
                    outputs[2][i] = bp;
                }
            }
        }
    }
}

impl FilterSVF {
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    pub fn process_block_simd(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        use wide::f32x4;
        let num_samples = outputs[0].len();
        let in_buf = if !inputs.is_empty() && !inputs[0].is_empty() {
            inputs[0]
        } else {
            &[]
        };

        let mut i = 0;
        
        while i + 3 < num_samples {
            let x0 = if i < in_buf.len() { in_buf[i] } else { 0.0 };
            let x1 = if i + 1 < in_buf.len() { in_buf[i+1] } else { 0.0 };
            let x2 = if i + 2 < in_buf.len() { in_buf[i+2] } else { 0.0 };
            let x3 = if i + 3 < in_buf.len() { in_buf[i+3] } else { 0.0 };
            
            let (lp0, hp0, bp0) = self.process_sample(x0, ctx.sample_rate);
            let (lp1, hp1, bp1) = self.process_sample(x1, ctx.sample_rate);
            let (lp2, hp2, bp2) = self.process_sample(x2, ctx.sample_rate);
            let (lp3, hp3, bp3) = self.process_sample(x3, ctx.sample_rate);
            
            let lp = f32x4::new([lp0, lp1, lp2, lp3]);
            let hp = f32x4::new([hp0, hp1, hp2, hp3]);
            let bp = f32x4::new([bp0, bp1, bp2, bp3]);
            
            let lp_arr = lp.to_array();
            let hp_arr = hp.to_array();
            let bp_arr = bp.to_array();
            
            if !outputs.is_empty() && i + 3 < outputs[0].len() {
                outputs[0][i] = lp_arr[0];
                outputs[0][i+1] = lp_arr[1];
                outputs[0][i+2] = lp_arr[2];
                outputs[0][i+3] = lp_arr[3];
            }
            if outputs.len() > 1 && i + 3 < outputs[1].len() {
                outputs[1][i] = hp_arr[0];
                outputs[1][i+1] = hp_arr[1];
                outputs[1][i+2] = hp_arr[2];
                outputs[1][i+3] = hp_arr[3];
            }
            if outputs.len() > 2 && i + 3 < outputs[2].len() {
                outputs[2][i] = bp_arr[0];
                outputs[2][i+1] = bp_arr[1];
                outputs[2][i+2] = bp_arr[2];
                outputs[2][i+3] = bp_arr[3];
            }
            
            i += 4;
        }
        
        while i < num_samples {
            let in_sample = if i < in_buf.len() { in_buf[i] } else { 0.0 };
            let (lp, hp, bp) = self.process_sample(in_sample, ctx.sample_rate);
            if !outputs.is_empty() && i < outputs[0].len() { outputs[0][i] = lp; }
            if outputs.len() > 1 && i < outputs[1].len() { outputs[1][i] = hp; }
            if outputs.len() > 2 && i < outputs[2].len() { outputs[2][i] = bp; }
            i += 1;
        }
    }
}

/// Tuned delay line with feedback (Comb Filter) for physical modeling synthesis.
#[derive(Debug)]
pub struct FilterComb {
    pub frequency: f32,
    pub feedback: f32,
    buffer: [f32; 2048],
    write_pos: usize,
}

impl FilterComb {
    pub fn new(frequency: f32, feedback: f32) -> Self {
        Self {
            frequency: frequency.max(20.0),
            feedback: feedback.clamp(-0.999, 0.999),
            buffer: [0.0; 2048],
            write_pos: 0,
        }
    }

    pub fn process_sample(&mut self, input: f32, sample_rate: u32) -> f32 {
        if sample_rate == 0 {
            return input;
        }

        let delay_samples = (sample_rate as f32 / self.frequency).clamp(1.0, 2047.0);
        let read_pos = (self.write_pos as f32 + 2048.0 - delay_samples) % 2048.0;

        let read_idx = read_pos.floor() as usize;
        let frac = read_pos % 1.0;
        let next_idx = (read_idx + 1) % 2048;

        let delayed = self.buffer[read_idx] * (1.0 - frac) + self.buffer[next_idx] * frac;
        let output = input + delayed * self.feedback;

        self.buffer[self.write_pos] = output;
        self.write_pos = (self.write_pos + 1) % 2048;

        output
    }
}

impl SignalProcessor for FilterComb {
    fn name(&self) -> &str {
        "FilterComb"
    }

    fn process_block(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        if ctx.sample_rate == 0 || outputs.is_empty() {
            return;
        }

        let num_samples = outputs[0].len();
        for i in 0..num_samples {
            let in_sample = if !inputs.is_empty() && !inputs[0].is_empty() && i < inputs[0].len() {
                inputs[0][i]
            } else {
                0.0
            };

            let out_sample = self.process_sample(in_sample, ctx.sample_rate);
            for out_ch in outputs.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = out_sample;
                }
            }
        }
    }
}

/// DC Blocking filter (y[n] = x[n] - x[n-1] + R * y[n-1], with R = 0.995).
#[derive(Debug, Clone)]
pub struct DcBlockFilter {
    x1: f32,
    y1: f32,
    pub r: f32,
}

impl Default for DcBlockFilter {
    fn default() -> Self {
        Self { x1: 0.0, y1: 0.0, r: 0.995 }
    }
}

impl DcBlockFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn process_sample(&mut self, input: f32) -> f32 {
        let y = input - self.x1 + self.r * self.y1;
        self.x1 = input;
        self.y1 = if y.is_finite() { y } else { 0.0 };
        self.y1
    }

    pub fn process_buffer(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.process_sample(*sample);
        }
    }
}

/// 1st order High-Pass (Low-Cut) filter.
#[derive(Debug, Clone)]
pub struct LowCutFilter {
    pub cutoff_hz: f32,
    x1: f32,
    y1: f32,
}

impl LowCutFilter {
    pub fn new(cutoff_hz: f32) -> Self {
        Self { cutoff_hz, x1: 0.0, y1: 0.0 }
    }

    pub fn process_sample(&mut self, input: f32, sample_rate: u32) -> f32 {
        if sample_rate == 0 { return input; }
        let dt = 1.0 / sample_rate as f32;
        let rc = 1.0 / (2.0 * std::f32::consts::PI * self.cutoff_hz.max(1.0));
        let alpha = rc / (rc + dt);
        let y = alpha * (self.y1 + input - self.x1);
        self.x1 = input;
        self.y1 = if y.is_finite() { y } else { 0.0 };
        self.y1
    }
}

/// 1st order Low-Pass (High-Cut) filter.
#[derive(Debug, Clone)]
pub struct HighCutFilter {
    pub cutoff_hz: f32,
    y1: f32,
}

impl HighCutFilter {
    pub fn new(cutoff_hz: f32) -> Self {
        Self { cutoff_hz, y1: 0.0 }
    }

    pub fn process_sample(&mut self, input: f32, sample_rate: u32) -> f32 {
        if sample_rate == 0 { return input; }
        let dt = 1.0 / sample_rate as f32;
        let rc = 1.0 / (2.0 * std::f32::consts::PI * self.cutoff_hz.max(1.0));
        let alpha = dt / (rc + dt);
        let y = self.y1 + alpha * (input - self.y1);
        self.y1 = if y.is_finite() { y } else { 0.0 };
        self.y1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    fn test_simd_scalar_agreement_filter_svf() {
        let mut filter_simd = FilterSVF::new(1000.0, 0.707);
        let mut filter_scalar = FilterSVF::new(1000.0, 0.707);

        let ctx = ProcessContext {
            sample_rate: 44100,
            bpm: 120.0,
            frame_position: 0,
            is_playing: true,
            param_bus: None,
            tuning_root_hz: 440.0,
            tuning_edo_divisions: 12,
        };

        let input_signal: Vec<f32> = (0..256).map(|i| (i as f32 * 0.1).sin()).collect();
        let input_slice = input_signal.as_slice();

        let mut simd_out_lp = vec![0.0f32; 256];
        let mut simd_out_hp = vec![0.0f32; 256];
        let mut simd_out_bp = vec![0.0f32; 256];
        let mut simd_outputs: Vec<&mut [f32]> = vec![
            simd_out_lp.as_mut_slice(),
            simd_out_hp.as_mut_slice(),
            simd_out_bp.as_mut_slice(),
        ];

        filter_simd.process_block_simd(&[input_slice], &mut simd_outputs, &ctx);

        let mut scalar_out_lp = vec![0.0f32; 256];
        let mut scalar_out_hp = vec![0.0f32; 256];
        let mut scalar_out_bp = vec![0.0f32; 256];

        for i in 0..256 {
            let (lp, hp, bp) = filter_scalar.process_sample(input_signal[i], ctx.sample_rate);
            scalar_out_lp[i] = lp;
            scalar_out_hp[i] = hp;
            scalar_out_bp[i] = bp;
        }

        for i in 0..256 {
            let diff_lp = (simd_out_lp[i] - scalar_out_lp[i]).abs();
            assert!(diff_lp < 1e-4, "FilterSVF LP mismatch at {}: SIMD {} vs Scalar {}", i, simd_out_lp[i], scalar_out_lp[i]);
        }
    }
}
