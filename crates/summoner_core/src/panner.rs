// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

/// Stereo panner using equal power panning law.
pub struct StereoPanner {
    pan: f32, // -1.0 to 1.0
}

impl StereoPanner {
    /// Create a new stereo panner with initial pan position (-1.0 to 1.0).
    pub fn new(pan: f32) -> Self {
        Self {
            pan: pan.clamp(-1.0, 1.0),
        }
    }

    /// Set pan position (-1.0 left to 1.0 right).
    pub fn set_pan(&mut self, pan: f32) {
        self.pan = pan.clamp(-1.0, 1.0);
    }

    /// Process input left/right sample pair returning panned stereo output.
    pub fn process(&self, left_in: f32, right_in: f32) -> (f32, f32) {
        // -1 = hard left, 1 = hard right
        let p = (self.pan + 1.0) * 0.5;
        let left_gain = (p * std::f32::consts::FRAC_PI_2).cos();
        let right_gain = (p * std::f32::consts::FRAC_PI_2).sin();

        (left_in * left_gain, right_in * right_gain)
    }
}

/// Stereo width adjustment processor (Mid/Side width matrix).
pub struct StereoWidth {
    width: f32, // 0.0 to 2.0 (0% to 200%)
}

impl StereoWidth {
    /// Create a new stereo width processor with initial width (0.0 mono to 2.0 ultra-wide).
    pub fn new(width: f32) -> Self {
        Self {
            width: width.clamp(0.0, 2.0),
        }
    }

    /// Set stereo width (0.0 to 2.0).
    pub fn set_width(&mut self, width: f32) {
        self.width = width.clamp(0.0, 2.0);
    }

    /// Process stereo sample pair with Mid/Side width adjustment.
    pub fn process(&self, left_in: f32, right_in: f32) -> (f32, f32) {
        let mid = (left_in + right_in) * 0.5;
        let side = (left_in - right_in) * 0.5;

        let new_side = side * self.width;

        (mid + new_side, mid - new_side)
    }
}
