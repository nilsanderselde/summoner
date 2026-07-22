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

pub struct Oversampler {
    factor: usize,
}

impl Oversampler {
    pub fn new(factor: usize) -> Self {
        Self { factor }
    }
    
    pub fn process_up(&mut self, input: f32) -> Vec<f32> {
        let mut out = vec![0.0; self.factor];
        out[0] = input * self.factor as f32; // simple zero-stuffing approximation for now
        out
    }
    
    pub fn process_down(&mut self, input: &[f32]) -> f32 {
        // simple averaging approximation for now
        input.iter().sum::<f32>() / self.factor as f32
    }
}
