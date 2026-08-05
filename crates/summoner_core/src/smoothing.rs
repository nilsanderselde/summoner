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

/// Parameter smoother using one-pole low-pass exponential smoothing filter.
pub struct SmoothParam {
    /// Target parameter value.
    pub target: f32,
    /// Current smoothed parameter value.
    pub current: f32,
    /// Exponential smoothing coefficient factor.
    pub smoothing_factor: f32,
}

impl SmoothParam {
    /// Create a new parameter smoother initialized with starting value, sample rate, and transition time in ms.
    pub fn new(initial: f32, sample_rate: f32, time_ms: f32) -> Self {
        let factor = (-1.0 / (time_ms * 0.001 * sample_rate)).exp();
        Self {
            target: initial,
            current: initial,
            smoothing_factor: factor,
        }
    }

    /// Set new target parameter value.
    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    /// Advance smoother by one sample and return smoothed parameter value.
    pub fn next_sample(&mut self) -> f32 {
        self.current =
            self.current * self.smoothing_factor + self.target * (1.0 - self.smoothing_factor);
        self.current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smooth_param_transition() {
        let mut param = SmoothParam::new(0.0, 44100.0, 10.0);
        param.set_target(1.0);
        let sample1 = param.next_sample();
        assert!(sample1 > 0.0 && sample1 < 1.0);
        for _ in 0..3000 {
            param.next_sample();
        }
        assert!((param.current - 1.0).abs() < 0.05);
    }
}
