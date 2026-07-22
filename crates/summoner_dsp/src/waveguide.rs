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

//! Digital Waveguide physical modeling primitives (Karplus-Strong string synthesis).

use summoner_core::audio::Sample;

/// Maximum delay line length for waveguide (supports down to ~20 Hz at 192 kHz).
pub const MAX_DELAY_SAMPLES: usize = 9600;

/// Karplus-Strong plucked string digital waveguide model. Zero heap allocation in processing.
#[derive(Debug)]
pub struct KarplusStrongString {
    delay_line: [Sample; MAX_DELAY_SAMPLES],
    write_pos: usize,
    delay_length: usize,
    feedback_decay: Sample,
    prev_filter_sample: Sample,
}

impl KarplusStrongString {
    pub fn new(frequency: f32, sample_rate: u32, feedback_decay: Sample) -> Self {
        let delay_length = ((sample_rate as f32) / frequency.max(10.0)).round() as usize;
        let delay_length = delay_length.clamp(2, MAX_DELAY_SAMPLES - 1);
        Self {
            delay_line: [0.0; MAX_DELAY_SAMPLES],
            write_pos: 0,
            delay_length,
            feedback_decay: feedback_decay.clamp(0.0, 0.999),
            prev_filter_sample: 0.0,
        }
    }

    /// Set frequency and recompute delay length.
    pub fn set_frequency(&mut self, frequency: f32, sample_rate: u32) {
        let length = ((sample_rate as f32) / frequency.max(10.0)).round() as usize;
        self.delay_length = length.clamp(2, MAX_DELAY_SAMPLES - 1);
    }

    /// Pluck string by filling delay buffer with noise burst.
    pub fn pluck(&mut self, amplitude: Sample) {
        // Deterministic noise fill using simple LCG to avoid rand crate dependencies
        let mut seed: u32 = 0x12345678;
        for i in 0..self.delay_length {
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let val = ((seed >> 16) & 0x7FFF) as f32 / 32768.0;
            self.delay_line[i] = (val * 2.0 - 1.0) * amplitude;
        }
        self.write_pos = 0;
        self.prev_filter_sample = 0.0;
    }

    /// Compute next output sample from string waveguide model.
    #[inline]
    pub fn process_sample(&mut self) -> Sample {
        let delayed = self.delay_line[self.write_pos];

        // Lowpass damping filter: 0.5 * (current + previous)
        let filtered = 0.5 * (delayed + self.prev_filter_sample);
        self.prev_filter_sample = delayed;

        let new_sample = filtered * self.feedback_decay;
        self.delay_line[self.write_pos] = new_sample;
        self.write_pos = (self.write_pos + 1) % self.delay_length;

        delayed
    }
}
