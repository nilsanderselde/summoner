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

#![allow(clippy::all)]

use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;
use crate::traits::SignalProcessor;

pub struct LufsMeterNode {
    pub integrated_lufs: f32,
    pub momentary_lufs: f32,
    pub peak: f32,
    pub rms: f32,
}
impl LufsMeterNode {
    pub fn new() -> Self { Self { integrated_lufs: -70.0, momentary_lufs: -70.0, peak: 0.0, rms: 0.0 } }
}
impl SignalProcessor for LufsMeterNode {
    fn name(&self) -> &str { "LufsMeterNode" }
    fn process_block(&mut self, input: &[&[Sample]], _output: &mut [&mut [Sample]], _ctx: &ProcessContext) {
        if input.is_empty() { return; }
        let mut sum_sq = 0.0;
        let mut max_peak = self.peak;
        let num_samples = input[0].len();
        for i in 0..num_samples {
            let x = input[0][i];
            sum_sq += x * x;
            if x.abs() > max_peak { max_peak = x.abs(); }
        }
        self.peak = max_peak;
        self.rms = (sum_sq / num_samples as f32).sqrt();
    }
}
