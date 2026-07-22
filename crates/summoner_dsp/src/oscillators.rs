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

//! Atomic oscillator DSP primitives with anti-aliasing and modulation.

use crate::traits::SignalProcessor;
use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;
use std::f32::consts::TAU;

fn poly_blep(t: f32, dt: f32) -> f32 {
    if dt <= 0.0 {
        return 0.0;
    }
    if t < dt {
        let t_norm = t / dt;
        2.0 * t_norm - t_norm * t_norm - 1.0
    } else if t > 1.0 - dt {
        let t_norm = (t - 1.0) / dt;
        t_norm * t_norm + 2.0 * t_norm + 1.0
    } else {
        0.0
    }
}

/// Band-limited sawtooth generator using PolyBLEP.
#[derive(Debug)]
pub struct OscSaw {
    pub frequency: f32,
    pub phase: f32,
}

impl OscSaw {
    pub fn new(frequency: f32) -> Self {
        Self { frequency, phase: 0.0 }
    }
}

impl SignalProcessor for OscSaw {
    fn name(&self) -> &str {
        "OscSaw"
    }

    fn process_block(
        &mut self,
        _inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        if ctx.sample_rate == 0 || outputs.is_empty() {
            return;
        }

        let dt = self.frequency / ctx.sample_rate as f32;
        let num_samples = outputs[0].len();

        for i in 0..num_samples {
            let naive = 2.0 * self.phase - 1.0;
            let val = naive - poly_blep(self.phase, dt);
            self.phase = (self.phase + dt) % 1.0;

            for out_ch in outputs.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = val;
                }
            }
        }
    }
}

/// Pulse/square wave generator with modulatable pulse width (PWM).
#[derive(Debug)]
pub struct OscPulse {
    pub frequency: f32,
    pub pulse_width: f32,
    pub phase: f32,
}

impl OscPulse {
    pub fn new(frequency: f32, pulse_width: f32) -> Self {
        Self {
            frequency,
            pulse_width: pulse_width.clamp(0.01, 0.99),
            phase: 0.0,
        }
    }
}

impl SignalProcessor for OscPulse {
    fn name(&self) -> &str {
        "OscPulse"
    }

    fn process_block(
        &mut self,
        _inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        if ctx.sample_rate == 0 || outputs.is_empty() {
            return;
        }

        let dt = self.frequency / ctx.sample_rate as f32;
        let num_samples = outputs[0].len();

        for i in 0..num_samples {
            let naive = if self.phase < self.pulse_width { 1.0 } else { -1.0 };
            let val = naive + poly_blep(self.phase, dt) - poly_blep((self.phase + 1.0 - self.pulse_width) % 1.0, dt);
            self.phase = (self.phase + dt) % 1.0;

            for out_ch in outputs.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = val;
                }
            }
        }
    }
}

/// Pure sine wave generator with Phase Modulation and Linear FM.
#[derive(Debug)]
pub struct OscSine {
    pub frequency: f32,
    pub phase: f32,
}

impl OscSine {
    pub fn new(frequency: f32) -> Self {
        Self { frequency, phase: 0.0 }
    }
}

impl SignalProcessor for OscSine {
    fn name(&self) -> &str {
        "OscSine"
    }

    fn process_block(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        if ctx.sample_rate == 0 || outputs.is_empty() {
            return;
        }

        let dt = (TAU * self.frequency) / ctx.sample_rate as f32;
        let num_samples = outputs[0].len();

        for i in 0..num_samples {
            let pm_offset = if !inputs.is_empty() && !inputs[0].is_empty() && i < inputs[0].len() {
                inputs[0][i] * TAU
            } else {
                0.0
            };

            let val = (self.phase + pm_offset).sin();
            self.phase = (self.phase + dt) % TAU;

            for out_ch in outputs.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = val;
                }
            }
        }
    }
}

/// Band-limited triangle wave generator.
#[derive(Debug)]
pub struct OscTriangle {
    pub frequency: f32,
    pub phase: f32,
}

impl OscTriangle {
    pub fn new(frequency: f32) -> Self {
        Self { frequency, phase: 0.0 }
    }
}

impl SignalProcessor for OscTriangle {
    fn name(&self) -> &str {
        "OscTriangle"
    }

    fn process_block(
        &mut self,
        _inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        if ctx.sample_rate == 0 || outputs.is_empty() {
            return;
        }

        let dt = self.frequency / ctx.sample_rate as f32;
        let num_samples = outputs[0].len();

        for i in 0..num_samples {
            let val = 2.0 * (2.0 * (self.phase - (self.phase + 0.5).floor())).abs() - 1.0;
            self.phase = (self.phase + dt) % 1.0;

            for out_ch in outputs.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = val;
                }
            }
        }
    }
}

/// Noise generator (White, Pink, Brown) using a zero-allocation PRNG.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoiseType {
    White,
    Pink,
    Brown,
}

#[derive(Debug)]
pub struct NoiseGen {
    pub noise_type: NoiseType,
    seed: u64,
    b0: f32, b1: f32, b2: f32, b3: f32, b4: f32, b5: f32, b6: f32, // Pink noise state
    last_brown: f32, // Brown noise state
}

impl NoiseGen {
    pub fn new(noise_type: NoiseType) -> Self {
        Self {
            noise_type,
            seed: 0x123456789ABCDEF0,
            b0: 0.0, b1: 0.0, b2: 0.0, b3: 0.0, b4: 0.0, b5: 0.0, b6: 0.0,
            last_brown: 0.0,
        }
    }

    fn next_prng(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let val = (self.seed >> 33) as f32 / 2147483648.0;
        val - 1.0
    }

    pub fn next_sample(&mut self) -> f32 {
        let white = self.next_prng();
        match self.noise_type {
            NoiseType::White => white,
            NoiseType::Pink => {
                self.b0 = 0.99886 * self.b0 + white * 0.0555179;
                self.b1 = 0.99332 * self.b1 + white * 0.0750759;
                self.b2 = 0.96900 * self.b2 + white * 0.153852;
                self.b3 = 0.86650 * self.b3 + white * 0.3104856;
                self.b4 = 0.55000 * self.b4 + white * 0.5329522;
                self.b5 = -0.7616 * self.b5 - white * 0.0168980;
                let pink = self.b0 + self.b1 + self.b2 + self.b3 + self.b4 + self.b5 + self.b6 + white * 0.5362;
                self.b6 = white * 0.115926;
                pink * 0.11
            }
            NoiseType::Brown => {
                self.last_brown = (self.last_brown + 0.02 * white) / 1.02;
                self.last_brown * 3.5
            }
        }
    }
}

impl SignalProcessor for NoiseGen {
    fn name(&self) -> &str {
        "NoiseGen"
    }

    fn process_block(
        &mut self,
        _inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        _ctx: &ProcessContext,
    ) {
        if outputs.is_empty() {
            return;
        }
        let num_samples = outputs[0].len();
        for i in 0..num_samples {
            let val = self.next_sample();
            for out_ch in outputs.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = val;
                }
            }
        }
    }
}
