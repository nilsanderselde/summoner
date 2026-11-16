// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

/// Selectable stereo panning law for channel strips (MIX-7).
/// `Linear0dB` matches the legacy linear center law as default so existing renders stay deterministic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum PanLaw {
    #[default]
    Linear0dB,
    EqualPower3dB,
    Compensated3dB,
    EqualPower4_5dB,
    Linear6dB,
}

impl PanLaw {
    /// Human-readable short badge label for mixer UIs.
    pub fn label(self) -> &'static str {
        match self {
            Self::Linear0dB => "0dB",
            Self::EqualPower3dB => "-3dB",
            Self::Compensated3dB => "C-3",
            Self::EqualPower4_5dB => "-4.5",
            Self::Linear6dB => "-6dB",
        }
    }

    /// Cycle through available pan laws for compact UI controls.
    pub fn next(self) -> Self {
        match self {
            Self::Linear0dB => Self::EqualPower3dB,
            Self::EqualPower3dB => Self::Compensated3dB,
            Self::Compensated3dB => Self::EqualPower4_5dB,
            Self::EqualPower4_5dB => Self::Linear6dB,
            Self::Linear6dB => Self::Linear0dB,
        }
    }

    /// Compute left/right channel gain multipliers for normalized pan in `[-1.0, 1.0]`.
    #[inline]
    pub fn gains(self, pan: f32) -> (f32, f32) {
        let p = pan.clamp(-1.0, 1.0);
        match self {
            Self::Linear0dB => ((1.0 - p).min(1.0), (1.0 + p).min(1.0)),
            Self::EqualPower3dB => {
                let theta = (p + 1.0) * 0.5 * std::f32::consts::FRAC_PI_2;
                (theta.cos(), theta.sin())
            }
            Self::Compensated3dB => {
                let theta = (p + 1.0) * 0.5 * std::f32::consts::FRAC_PI_2;
                (
                    std::f32::consts::SQRT_2 * theta.cos(),
                    std::f32::consts::SQRT_2 * theta.sin(),
                )
            }
            Self::EqualPower4_5dB => {
                let theta = (p + 1.0) * 0.5 * std::f32::consts::FRAC_PI_2;
                (theta.cos().powf(1.5), theta.sin().powf(1.5))
            }
            Self::Linear6dB => (0.5 * (1.0 - p), 0.5 * (1.0 + p)),
        }
    }
}

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
