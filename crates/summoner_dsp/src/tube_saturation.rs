// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Tube saturation DSP node simulating vacuum tube triode/pentode asymmetric drive.

use crate::traits::SignalProcessor;
use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;

/// Tube saturation simulation node providing warm asymmetric distortion and even-order harmonics.
#[derive(Debug)]
pub struct TubeSaturationNode {
    pub drive: f32,
    pub bias: f32,
    pub warmth: f32,
    lp_state: f32,
}

impl TubeSaturationNode {
    pub fn new(drive: f32, bias: f32) -> Self {
        Self {
            drive: drive.max(1.0),
            bias: bias.clamp(0.0, 0.5),
            warmth: 0.5,
            lp_state: 0.0,
        }
    }
}

impl Default for TubeSaturationNode {
    fn default() -> Self {
        Self::new(2.5, 0.2)
    }
}

impl SignalProcessor for TubeSaturationNode {
    fn name(&self) -> &str {
        "TubeSaturationNode"
    }

    fn process_block(&mut self, input: &[&[Sample]], output: &mut [&mut [Sample]], _ctx: &ProcessContext) {
        if input.is_empty() || output.is_empty() {
            return;
        }

        let num_samples = input[0].len().min(output[0].len());
        let dc_offset = self.bias.tanh();

        for i in 0..num_samples {
            let x = input[0][i] * self.drive;

            // Asymmetrical triode transfer curve generating even-order harmonics
            let biased_x = x + self.bias;
            let saturated = (biased_x).tanh() - dc_offset;

            // Warmth low-pass filter smoothing high harmonics
            let alpha = 0.3 + (1.0 - self.warmth.clamp(0.0, 1.0)) * 0.6;
            self.lp_state = self.lp_state * (1.0 - alpha) + saturated * alpha;

            for out_ch in output.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = self.lp_state;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tube_saturation_asymmetry() {
        let mut tube = TubeSaturationNode::new(3.0, 0.25);
        let ctx = ProcessContext::new(44100, 120.0, 0);

        let pos_in = vec![0.5f32; 64];
        let neg_in = vec![-0.5f32; 64];
        let mut pos_out = vec![0.0f32; 64];
        let mut neg_out = vec![0.0f32; 64];

        tube.process_block(&[&pos_in[..]], &mut [&mut pos_out[..]], &ctx);
        tube.process_block(&[&neg_in[..]], &mut [&mut neg_out[..]], &ctx);

        assert!(pos_out.iter().all(|s| s.is_finite()));
        assert!(neg_out.iter().all(|s| s.is_finite()));
        assert!((pos_out[63].abs() - neg_out[63].abs()).abs() > 0.001, "Tube saturation output should be asymmetric");
    }
}
