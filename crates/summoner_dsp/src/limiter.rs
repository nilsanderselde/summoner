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
use std::collections::VecDeque;

pub struct LimiterNode {
    pub ceiling: f32,
    pub release_time: f32,
    lookahead_buffer: VecDeque<f32>,
    env: f32,
}

impl LimiterNode {
    pub fn new(lookahead_samples: usize) -> Self {
        let mut buf = VecDeque::new();
        buf.resize(lookahead_samples, 0.0);
        Self {
            ceiling: 0.99,
            release_time: 0.1,
            lookahead_buffer: buf,
            env: 0.0,
        }
    }
}

impl SignalProcessor for LimiterNode {
    fn name(&self) -> &str { "LimiterNode" }
    fn process_block(&mut self, input: &[&[Sample]], output: &mut [&mut [Sample]], ctx: &ProcessContext) {
        if input.is_empty() || output.is_empty() { return; }
        let num_samples = input[0].len().min(output[0].len());
        
        let dt = 1.0 / ctx.sample_rate as f32;
        let release_coeff = (-dt / self.release_time).exp();
        
        for i in 0..num_samples {
            let x = input[0][i];
            
            let target_env = x.abs();
            if target_env > self.env {
                self.env = target_env;
            } else {
                self.env *= release_coeff;
            }
            
            self.lookahead_buffer.push_back(x);
            let delayed = self.lookahead_buffer.pop_front().unwrap();
            
            let gain = if self.env > self.ceiling {
                self.ceiling / self.env
            } else {
                1.0
            };
            
            for out_ch in output.iter_mut() {
                out_ch[i] = delayed * gain;
            }
        }
    }
}
