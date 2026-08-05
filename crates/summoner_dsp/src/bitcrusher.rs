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

//! Dedicated Bitcrusher processor node with bit quantization and downsampling.

use crate::traits::SignalProcessor;
use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;

/// Bitcrusher DSP node for sample-rate reduction and bit-depth quantization.
#[derive(Debug)]
pub struct BitcrusherNode {
    pub bit_depth: u8,              // Bit depth 1 to 16
    pub sample_rate_reduction: u32, // Downsampling factor 1 to 64
    pub mix: f32,                   // Dry/Wet mix 0.0 to 1.0
    counter: u32,
    held_sample: f32,
}

impl BitcrusherNode {
    pub fn new(bit_depth: u8, sample_rate_reduction: u32) -> Self {
        Self {
            bit_depth: bit_depth.clamp(1, 16),
            sample_rate_reduction: sample_rate_reduction.clamp(1, 64),
            mix: 1.0,
            counter: 0,
            held_sample: 0.0,
        }
    }

    pub fn process_sample(&mut self, input: f32) -> f32 {
        self.counter += 1;
        if self.counter >= self.sample_rate_reduction.max(1) {
            self.counter = 0;
            let steps = (1 << self.bit_depth.clamp(1, 16)) as f32;
            let quantized = (input.clamp(-1.0, 1.0) * steps).round() / steps;
            self.held_sample = quantized;
        }

        input * (1.0 - self.mix) + self.held_sample * self.mix
    }
}

impl Default for BitcrusherNode {
    fn default() -> Self {
        Self::new(8, 4)
    }
}

impl SignalProcessor for BitcrusherNode {
    fn name(&self) -> &str {
        "BitcrusherNode"
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

    #[test]
    fn test_bitcrusher_quantization() {
        let mut crusher = BitcrusherNode::new(4, 1);
        let out = crusher.process_sample(0.5);
        assert!((out - 0.5).abs() < 0.1);
    }
}
