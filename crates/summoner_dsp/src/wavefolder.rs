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

//! Multi-stage wavefolder processor node.

use crate::traits::SignalProcessor;
use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;

/// Wavefolder DSP node with customizable threshold, fold iteration count, drive, and mix.
#[derive(Debug)]
pub struct WavefolderNode {
    pub threshold: f32,
    pub folds: u8,
    pub drive: f32,
    pub mix: f32,
}

impl WavefolderNode {
    pub fn new(threshold: f32, folds: u8, drive: f32) -> Self {
        Self {
            threshold: threshold.clamp(0.05, 1.0),
            folds: folds.clamp(1, 16),
            drive: drive.max(1.0),
            mix: 1.0,
        }
    }

    pub fn process_sample(&self, input: f32) -> f32 {
        let mut sample = input * self.drive;
        let thresh = self.threshold;

        for _ in 0..self.folds {
            if sample > thresh {
                sample = 2.0 * thresh - sample;
            } else if sample < -thresh {
                sample = -2.0 * thresh - sample;
            }
        }

        let folded = sample.clamp(-1.0, 1.0);
        input * (1.0 - self.mix) + folded * self.mix
    }
}

impl Default for WavefolderNode {
    fn default() -> Self {
        Self::new(0.5, 4, 2.0)
    }
}

impl SignalProcessor for WavefolderNode {
    fn name(&self) -> &str {
        "WavefolderNode"
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
    fn test_wavefolder_folds() {
        let node = WavefolderNode::new(0.5, 2, 2.0);
        let out = node.process_sample(0.8);
        assert!(out.is_finite());
        assert!(out.abs() <= 1.0);
    }
}
