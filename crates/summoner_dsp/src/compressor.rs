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

//! Dynamic Range Compressor processor node with sidechain support.

use crate::traits::SignalProcessor;
use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;

/// Dynamic range compressor node.
#[derive(Debug)]
pub struct CompressorNode {
    pub threshold: f32,   // dB (-60.0 to 0.0)
    pub ratio: f32,       // Compression ratio (1.0 to 20.0)
    pub attack: f32,      // Attack time in seconds
    pub release: f32,     // Release time in seconds
    pub knee_db: f32,     // Knee width in dB
    pub makeup_gain: f32, // Makeup gain in dB
    env: f32,
}

impl CompressorNode {
    pub fn new() -> Self {
        Self {
            threshold: -20.0,
            ratio: 4.0,
            attack: 0.01,
            release: 0.1,
            knee_db: 5.0,
            makeup_gain: 0.0,
            env: 0.0,
        }
    }

    pub fn with_params(
        threshold_db: f32,
        ratio: f32,
        attack_ms: f32,
        release_ms: f32,
        makeup_gain_db: f32,
    ) -> Self {
        Self {
            threshold: threshold_db,
            ratio: ratio.max(1.0),
            attack: (attack_ms * 0.001).max(0.0001),
            release: (release_ms * 0.001).max(0.001),
            knee_db: 5.0,
            makeup_gain: makeup_gain_db,
            env: 0.0,
        }
    }
}

impl Default for CompressorNode {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalProcessor for CompressorNode {
    fn name(&self) -> &str {
        "CompressorNode"
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
            44100.0
        };

        let dt = 1.0 / sr;
        let attack_coeff = (-dt / self.attack.max(0.0001)).exp();
        let release_coeff = (-dt / self.release.max(0.001)).exp();

        let has_sidechain = input.len() > 1;

        for i in 0..num_samples {
            let detect = if has_sidechain && !input[1].is_empty() && i < input[1].len() {
                input[1][i]
            } else {
                input[0][i]
            };
            let level_db = 20.0 * detect.abs().max(1e-5).log10();

            let target_env = if level_db > self.threshold {
                level_db
            } else {
                -120.0
            };

            let coeff = if target_env > self.env {
                attack_coeff
            } else {
                release_coeff
            };
            self.env = self.env * coeff + target_env * (1.0 - coeff);

            let mut gain_db = 0.0;
            if self.env > self.threshold {
                gain_db = -(self.env - self.threshold) * (1.0 - 1.0 / self.ratio);
            }

            let gain = 10.0f32.powf((gain_db + self.makeup_gain) / 20.0);

            let x = input[0][i];
            for out_ch in output.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = x * gain;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compressor_reduces_gain() {
        let mut comp = CompressorNode::with_params(-10.0, 4.0, 10.0, 100.0, 0.0);
        let ctx = ProcessContext::new(44100, 120.0, 0);

        let loud_in = vec![0.9f32; 512];
        let mut out = vec![0.0f32; 512];

        comp.process_block(&[&loud_in[..]], &mut [&mut out[..]], &ctx);

        let final_sample = out[511];
        assert!(
            final_sample < 0.9,
            "Compressor should reduce gain on loud signals"
        );
    }
}
