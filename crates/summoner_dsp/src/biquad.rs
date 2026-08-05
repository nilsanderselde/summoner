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

use crate::traits::SignalProcessor;
use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterType {
    Lowpass,
    Highpass,
    Bandpass,
    Notch,
    Peaking,
    LowShelf,
    HighShelf,
}

#[derive(Debug, Clone)]
pub struct FilterBiquad {
    pub filter_type: FilterType,
    pub freq: f32,
    pub q: f32,
    pub gain_db: f32,
    pub sample_rate: f32,

    // coeffs
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,

    // state
    z1: f32,
    z2: f32,
}

impl FilterBiquad {
    pub fn new(filter_type: FilterType, sample_rate: f32) -> Self {
        let mut b = Self {
            filter_type,
            freq: 1000.0,
            q: 0.707,
            gain_db: 0.0,
            sample_rate,
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            z1: 0.0,
            z2: 0.0,
        };
        b.calculate_coeffs();
        b
    }

    pub fn calculate_coeffs(&mut self) {
        use std::f32::consts::PI;
        let sr = if self.sample_rate > 0.0 {
            self.sample_rate
        } else {
            44100.0
        };
        let w0 = 2.0 * PI * self.freq.clamp(10.0, sr * 0.49) / sr;
        let alpha = w0.sin() / (2.0 * self.q.max(0.01));
        let a = 10.0f32.powf(self.gain_db / 40.0);

        let (b0, b1, b2, a0, a1, a2) = match self.filter_type {
            FilterType::Lowpass => {
                let cos_w0 = w0.cos();
                (
                    (1.0 - cos_w0) / 2.0,
                    1.0 - cos_w0,
                    (1.0 - cos_w0) / 2.0,
                    1.0 + alpha,
                    -2.0 * cos_w0,
                    1.0 - alpha,
                )
            }
            FilterType::Highpass => {
                let cos_w0 = w0.cos();
                (
                    (1.0 + cos_w0) / 2.0,
                    -(1.0 + cos_w0),
                    (1.0 + cos_w0) / 2.0,
                    1.0 + alpha,
                    -2.0 * cos_w0,
                    1.0 - alpha,
                )
            }
            FilterType::Bandpass => {
                let cos_w0 = w0.cos();
                (alpha, 0.0, -alpha, 1.0 + alpha, -2.0 * cos_w0, 1.0 - alpha)
            }
            FilterType::Notch => {
                let cos_w0 = w0.cos();
                (
                    1.0,
                    -2.0 * cos_w0,
                    1.0,
                    1.0 + alpha,
                    -2.0 * cos_w0,
                    1.0 - alpha,
                )
            }
            FilterType::Peaking => {
                let cos_w0 = w0.cos();
                (
                    1.0 + alpha * a,
                    -2.0 * cos_w0,
                    1.0 - alpha * a,
                    1.0 + alpha / a,
                    -2.0 * cos_w0,
                    1.0 - alpha / a,
                )
            }
            FilterType::LowShelf => {
                let cos_w0 = w0.cos();
                let sq_a = a.sqrt();
                (
                    a * ((a + 1.0) - (a - 1.0) * cos_w0 + 2.0 * sq_a * alpha),
                    2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w0),
                    a * ((a + 1.0) - (a - 1.0) * cos_w0 - 2.0 * sq_a * alpha),
                    (a + 1.0) + (a - 1.0) * cos_w0 + 2.0 * sq_a * alpha,
                    -2.0 * ((a - 1.0) + (a + 1.0) * cos_w0),
                    (a + 1.0) + (a - 1.0) * cos_w0 - 2.0 * sq_a * alpha,
                )
            }
            FilterType::HighShelf => {
                let cos_w0 = w0.cos();
                let sq_a = a.sqrt();
                (
                    a * ((a + 1.0) + (a - 1.0) * cos_w0 + 2.0 * sq_a * alpha),
                    -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w0),
                    a * ((a + 1.0) + (a - 1.0) * cos_w0 - 2.0 * sq_a * alpha),
                    (a + 1.0) - (a - 1.0) * cos_w0 + 2.0 * sq_a * alpha,
                    2.0 * ((a - 1.0) - (a + 1.0) * cos_w0),
                    (a + 1.0) - (a - 1.0) * cos_w0 - 2.0 * sq_a * alpha,
                )
            }
        };

        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }

    pub fn process_sample(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.z1 + self.b2 * self.z2
            - self.a1 * self.z1
            - self.a2 * self.z2;
        self.z2 = self.z1;
        self.z1 = x;
        y
    }
}

impl SignalProcessor for FilterBiquad {
    fn name(&self) -> &str {
        "FilterBiquad"
    }
    fn process_block(
        &mut self,
        input: &[&[Sample]],
        output: &mut [&mut [Sample]],
        _ctx: &ProcessContext,
    ) {
        if input.is_empty() || output.is_empty() {
            return;
        }
        let num_samples = input[0].len().min(output[0].len());

        for i in 0..num_samples {
            let x = input[0][i];
            let y = self.process_sample(x);

            for out_ch in output.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = y;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::transport::Transport;

    #[test]
    fn test_biquad_filtering() {
        let transport = Transport::new(44100, 120.0);
        let ctx = ProcessContext::from_transport(&transport);
        let mut filter = FilterBiquad::new(FilterType::Lowpass, 44100.0);
        let in_buf = vec![1.0f32; 64];
        let mut out_buf = vec![0.0f32; 64];
        filter.process_block(&[&in_buf[..]], &mut [&mut out_buf[..]], &ctx);
        assert!(out_buf.iter().any(|&v| v != 0.0));
    }
}
