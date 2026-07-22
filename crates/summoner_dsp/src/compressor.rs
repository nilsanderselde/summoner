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

use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;
use crate::traits::SignalProcessor;

pub struct CompressorNode {
    pub threshold: f32,
    pub ratio: f32,
    pub attack: f32,
    pub release: f32,
    pub knee_db: f32,
    pub makeup_gain: f32,
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
}

impl SignalProcessor for CompressorNode {
    fn name(&self) -> &str { "CompressorNode" }
    fn process_block(&mut self, input: &[&[Sample]], output: &mut [&mut [Sample]], ctx: &ProcessContext) {
        if input.is_empty() || output.is_empty() { return; }
        let num_samples = input[0].len().min(output[0].len());
        
        let dt = 1.0 / ctx.sample_rate as f32;
        let attack_coeff = (-dt / self.attack).exp();
        let release_coeff = (-dt / self.release).exp();
        
        let has_sidechain = input.len() > 1;
        
        for i in 0..num_samples {
            let detect = if has_sidechain { input[1][i] } else { input[0][i] };
            let level_db = 20.0 * detect.abs().max(1e-5).log10();
            
            let target_env = if level_db > self.threshold {
                level_db
            } else {
                -120.0
            };
            
            let coeff = if target_env > self.env { attack_coeff } else { release_coeff };
            self.env = self.env * coeff + target_env * (1.0 - coeff);
            
            let mut gain_db = 0.0;
            if self.env > self.threshold {
                gain_db = -(self.env - self.threshold) * (1.0 - 1.0 / self.ratio);
            }
            
            let gain = 10.0f32.powf((gain_db + self.makeup_gain) / 20.0);
            
            let x = input[0][i];
            for out_ch in output.iter_mut() {
                out_ch[i] = x * gain;
            }
        }
    }
}
