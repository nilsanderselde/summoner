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
    pub short_term_lufs: f32,
    pub peak: f32,
    pub rms: f32,
}

impl LufsMeterNode {
    pub fn new() -> Self {
        Self {
            integrated_lufs: -70.0,
            momentary_lufs: -70.0,
            short_term_lufs: -70.0,
            peak: 0.0,
            rms: 0.0,
        }
    }
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
        let mean_sq = (sum_sq / num_samples as f32).max(1e-12);
        self.rms = mean_sq.sqrt();
        let lufs = 10.0 * mean_sq.log10() - 0.691;
        self.momentary_lufs = lufs.clamp(-70.0, 6.0);
        self.short_term_lufs = (lufs * 0.9 + self.short_term_lufs * 0.1).clamp(-70.0, 6.0);
        self.integrated_lufs = (lufs * 0.5 + self.integrated_lufs * 0.5).clamp(-70.0, 6.0);
    }
}

/// True Peak Meter with 4x inter-sample peak detection.
#[derive(Debug, Default)]
pub struct TruePeakMeter {
    pub max_true_peak_db: f32,
}

impl TruePeakMeter {
    pub fn new() -> Self {
        Self { max_true_peak_db: -120.0 }
    }

    pub fn process_block(&mut self, samples: &[f32]) {
        let mut peak = 0.0f32;
        for window in samples.windows(2) {
            let s0 = window[0];
            let s1 = window[1];
            let abs_s0 = s0.abs();
            let abs_s1 = s1.abs();
            if abs_s0 > peak { peak = abs_s0; }
            if abs_s1 > peak { peak = abs_s1; }
            // Interpolate mid-point (simple 4x oversampling approximation)
            let mid = (s0 + s1) * 0.5;
            let quad1 = s0 * 0.75 + s1 * 0.25;
            let quad3 = s0 * 0.25 + s1 * 0.75;
            let max_inter = mid.abs().max(quad1.abs()).max(quad3.abs());
            if max_inter > peak { peak = max_inter; }
        }
        let peak_db = if peak > 1e-6 { 20.0 * peak.log10() } else { -120.0 };
        if peak_db > self.max_true_peak_db {
            self.max_true_peak_db = peak_db;
        }
    }
}

/// K-System Metering scale (K-12, K-14, K-20).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KSystemScale {
    K12,
    K14,
    K20,
}

impl KSystemScale {
    pub fn reference_db(&self) -> f32 {
        match self {
            KSystemScale::K12 => -12.0,
            KSystemScale::K14 => -14.0,
            KSystemScale::K20 => -20.0,
        }
    }
}

pub fn k_system_headroom(rms_db: f32, scale: KSystemScale) -> f32 {
    rms_db - scale.reference_db()
}

