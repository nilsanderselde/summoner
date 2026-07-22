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

//! Polyphase FIR Oversampler for anti-aliasing in nonlinear DSP algorithms.

const FIR_TAPS: usize = 12;
// Half-band lowpass FIR filter coefficients for anti-aliasing
const FIR_COEFFS: [f32; FIR_TAPS] = [
    -0.005, 0.015, -0.035, 0.075, -0.160, 0.610,
    0.610, -0.160, 0.075, -0.035, 0.015, -0.005,
];

#[derive(Debug, Clone)]
pub struct Oversampler {
    pub factor: usize,
    up_history: [f32; FIR_TAPS],
    down_history: [f32; FIR_TAPS],
    up_pos: usize,
    down_pos: usize,
}

impl Oversampler {
    pub fn new(factor: usize) -> Self {
        let valid_factor = match factor {
            2 | 4 | 8 => factor,
            _ => 2,
        };
        Self {
            factor: valid_factor,
            up_history: [0.0; FIR_TAPS],
            down_history: [0.0; FIR_TAPS],
            up_pos: 0,
            down_pos: 0,
        }
    }

    /// Upsample a single sample into `factor` upsampled samples using zero-stuffing + FIR lowpass.
    #[allow(clippy::needless_range_loop)]
    pub fn process_up(&mut self, input: f32, output_slice: &mut [f32]) {
        let f = self.factor;
        if output_slice.len() < f {
            return;
        }

        for sub in 0..f {
            let sample = if sub == 0 { input * f as f32 } else { 0.0 };
            self.up_history[self.up_pos] = sample;
            self.up_pos = (self.up_pos + 1) % FIR_TAPS;

            let mut filtered = 0.0f32;
            for t in 0..FIR_TAPS {
                let idx = (self.up_pos + FIR_TAPS - 1 - t) % FIR_TAPS;
                filtered += self.up_history[idx] * FIR_COEFFS[t];
            }
            output_slice[sub] = filtered;
        }
    }

    /// Downsample `factor` upsampled samples back to 1 sample using FIR filtering + decimation.
    #[allow(clippy::needless_range_loop)]
    pub fn process_down(&mut self, input_slice: &[f32]) -> f32 {
        let f = self.factor;
        let mut last_filtered = 0.0f32;

        for &sample in input_slice.iter().take(f) {
            self.down_history[self.down_pos] = sample;
            self.down_pos = (self.down_pos + 1) % FIR_TAPS;

            let mut filtered = 0.0f32;
            for t in 0..FIR_TAPS {
                let idx = (self.down_pos + FIR_TAPS - 1 - t) % FIR_TAPS;
                filtered += self.down_history[idx] * FIR_COEFFS[t];
            }
            last_filtered = filtered;
        }

        last_filtered
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oversampler_up_down() {
        let mut oversampler = Oversampler::new(4);
        let mut up_buf = [0.0f32; 4];

        oversampler.process_up(0.5, &mut up_buf);
        assert_eq!(up_buf.len(), 4);

        let down_sample = oversampler.process_down(&up_buf);
        assert!(down_sample.is_finite());
    }
}

