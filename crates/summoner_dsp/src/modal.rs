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

//! Modal resonator physical modeling primitives.

use summoner_core::audio::Sample;
use std::f32::consts::TAU;

/// Second-order IIR modal bandpass resonator filter.
#[derive(Debug, Clone)]
pub struct ModalResonator {
    pub frequency: f32,
    pub damping: f32,
    a1: f32,
    a2: f32,
    b0: f32,
    y1: f32,
    y2: f32,
}

impl ModalResonator {
    pub fn new(frequency: f32, damping: f32, sample_rate: u32) -> Self {
        let mut res = Self {
            frequency,
            damping,
            a1: 0.0,
            a2: 0.0,
            b0: 0.0,
            y1: 0.0,
            y2: 0.0,
        };
        res.update_coefficients(sample_rate);
        res
    }

    pub fn update_coefficients(&mut self, sample_rate: u32) {
        if sample_rate == 0 {
            return;
        }
        let omega = (TAU * self.frequency) / sample_rate as f32;
        let radius = (-self.damping / sample_rate as f32).exp();

        self.a1 = -2.0 * radius * omega.cos();
        self.a2 = radius * radius;
        self.b0 = (1.0 - radius * radius) * 0.5;
    }

    #[inline]
    pub fn process_sample(&mut self, input: Sample) -> Sample {
        let output = self.b0 * input - self.a1 * self.y1 - self.a2 * self.y2;
        self.y2 = self.y1;
        self.y1 = output;
        output
    }

    pub fn reset(&mut self) {
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
}
