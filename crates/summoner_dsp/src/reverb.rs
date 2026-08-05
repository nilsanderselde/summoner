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

//! Algorithmic Schroeder-Moorer Reverb processor with comb & all-pass filter network.

use crate::traits::SignalProcessor;
use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;

const COMB_TUNINGS: [usize; 4] = [1116, 1277, 1422, 1617];
const ALLPASS_TUNINGS: [usize; 2] = [556, 341];

/// Schroeder-Moorer Reverb processor (4 parallel comb filters + 2 series all-pass filters).
#[derive(Debug)]
pub struct EffectReverb {
    pub room_size: f32,
    pub damping: f32,
    pub mix: f32,
    comb_buffers: [[f32; 1618]; 4],
    comb_pos: [usize; 4],
    comb_damp: [f32; 4],
    allpass_buffers: [[f32; 557]; 2],
    allpass_pos: [usize; 2],
}

impl EffectReverb {
    pub fn new(room_size: f32, mix: f32) -> Self {
        Self {
            room_size: room_size.clamp(0.0, 0.98),
            damping: 0.2,
            mix: mix.clamp(0.0, 1.0),
            comb_buffers: [[0.0; 1618]; 4],
            comb_pos: [0; 4],
            comb_damp: [0.0; 4],
            allpass_buffers: [[0.0; 557]; 2],
            allpass_pos: [0; 2],
        }
    }

    pub fn process_sample(&mut self, input: f32) -> f32 {
        let mut comb_sum = 0.0f32;

        for (c, &buf_len) in COMB_TUNINGS.iter().enumerate() {
            let pos = self.comb_pos[c];
            let delayed = self.comb_buffers[c][pos];

            self.comb_damp[c] += self.damping * (delayed - self.comb_damp[c]);
            self.comb_buffers[c][pos] = input + self.comb_damp[c] * (0.7 + 0.28 * self.room_size);

            self.comb_pos[c] = (pos + 1) % buf_len;
            comb_sum += delayed;
        }

        let mut ap_out = comb_sum * 0.25;
        for (a, &buf_len) in ALLPASS_TUNINGS.iter().enumerate() {
            let pos = self.allpass_pos[a];
            let delayed = self.allpass_buffers[a][pos];

            let new_val = ap_out + delayed * 0.5;
            self.allpass_buffers[a][pos] = new_val;
            ap_out = delayed - new_val * 0.5;

            self.allpass_pos[a] = (pos + 1) % buf_len;
        }

        input * (1.0 - self.mix) + ap_out * self.mix
    }
}

impl Default for EffectReverb {
    fn default() -> Self {
        Self::new(0.7, 0.3)
    }
}

impl SignalProcessor for EffectReverb {
    fn name(&self) -> &str {
        "EffectReverb"
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
        for i in 0..num_samples {
            let in_sample = if !inputs.is_empty() && !inputs[0].is_empty() && i < inputs[0].len() {
                inputs[0][i]
            } else {
                0.0
            };

            let rev_out = self.process_sample(in_sample);
            for out_ch in outputs.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = rev_out;
                }
            }
        }
    }
}

/// Convolution Reverb processor using custom impulse response samples.
#[derive(Debug)]
pub struct ConvolutionReverbNode {
    pub ir_samples: Vec<f32>,
    pub mix: f32,
    history: Vec<f32>,
    head: usize,
}

impl ConvolutionReverbNode {
    pub fn new(ir_samples: Vec<f32>, mix: f32) -> Self {
        let len = ir_samples.len().max(1);
        Self {
            ir_samples,
            mix: mix.clamp(0.0, 1.0),
            history: vec![0.0; len],
            head: 0,
        }
    }

    pub fn process_sample(&mut self, input: f32) -> f32 {
        if self.ir_samples.is_empty() {
            return input;
        }
        let len = self.ir_samples.len();
        self.history[self.head] = input;

        let mut acc = 0.0f32;
        let mut idx = self.head;
        for &ir_val in &self.ir_samples {
            acc += self.history[idx] * ir_val;
            if idx == 0 {
                idx = len - 1;
            } else {
                idx -= 1;
            }
        }

        self.head = (self.head + 1) % len;
        input * (1.0 - self.mix) + acc * self.mix
    }
}

impl SignalProcessor for ConvolutionReverbNode {
    fn name(&self) -> &str {
        "ConvolutionReverbNode"
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
        for i in 0..num_samples {
            let in_sample = if !inputs.is_empty() && !inputs[0].is_empty() && i < inputs[0].len() {
                inputs[0][i]
            } else {
                0.0
            };

            let out_sample = self.process_sample(in_sample);
            for out_ch in outputs.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = out_sample;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::transport::Transport;

    #[test]
    fn test_schroeder_reverb() {
        let transport = Transport::new(44100, 120.0);
        let ctx = ProcessContext::from_transport(&transport);

        let mut reverb = EffectReverb::new(0.8, 0.4);
        let in_buf = vec![1.0f32; 64];
        let mut out_rev = vec![0.0f32; 64];

        reverb.process_block(&[&in_buf[..]], &mut [&mut out_rev[..]], &ctx);
        assert!(out_rev.iter().any(|v| *v != 0.0));
    }

    #[test]
    fn test_convolution_reverb() {
        let ir = vec![1.0, 0.5, 0.25];
        let mut conv = ConvolutionReverbNode::new(ir, 1.0);
        let out1 = conv.process_sample(1.0);
        assert_eq!(out1, 1.0);
        let out2 = conv.process_sample(0.0);
        assert_eq!(out2, 0.5);
        let out3 = conv.process_sample(0.0);
        assert_eq!(out3, 0.25);
    }
}
