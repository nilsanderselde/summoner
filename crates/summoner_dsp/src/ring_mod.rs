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

pub struct RingModulator {
    phase: f32,
    pub freq: f32,
}
impl RingModulator {
    pub fn new() -> Self { Self { phase: 0.0, freq: 100.0 } }
}
impl SignalProcessor for RingModulator {
    fn name(&self) -> &str { "RingModulator" }
    fn process_block(&mut self, input: &[&[Sample]], output: &mut [&mut [Sample]], ctx: &ProcessContext) {
        if input.is_empty() || output.is_empty() { return; }
        let num_samples = input[0].len().min(output[0].len());
        let dt = 1.0 / ctx.sample_rate as f32;
        use std::f32::consts::PI;
        for i in 0..num_samples {
            let x = input[0][i];
            let mod_sig = (2.0 * PI * self.phase).sin();
            self.phase = (self.phase + self.freq * dt).fract();
            for out_ch in output.iter_mut() { out_ch[i] = x * mod_sig; }
        }
    }
}

pub struct FrequencyShifter { pub freq: f32 }
impl FrequencyShifter { pub fn new() -> Self { Self { freq: 100.0 } } }
impl SignalProcessor for FrequencyShifter {
    fn name(&self) -> &str { "FrequencyShifter" }
    fn process_block(&mut self, _i: &[&[Sample]], _o: &mut [&mut [Sample]], _c: &ProcessContext) {}
}
