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
    
    pub fn trigger(&mut self, note: u8, ctx: &ProcessContext) {
        self.frequency = ctx.note_to_hz(note as i32);
    }
    
    pub fn process_sample(&mut self, sample_rate: u32) -> f32 {
        if sample_rate == 0 {
            return 0.0;
        }
        let dt = self.frequency / sample_rate as f32;
        let naive = 2.0 * self.phase - 1.0;
        let val = naive - poly_blep(self.phase, dt);
        self.phase = (self.phase + dt) % 1.0;
        val
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

        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        {
            self.process_block_simd(outputs, ctx);
        }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            let num_samples = outputs[0].len();
            for i in 0..num_samples {
                let val = self.process_sample(ctx.sample_rate);
                for out_ch in outputs.iter_mut() {
                    if i < out_ch.len() {
                        out_ch[i] = val;
                    }
                }
            }
        }
    }
}

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
fn poly_blep_simd(t: wide::f32x4, dt: wide::f32x4) -> wide::f32x4 {
    use wide::CmpGt;
    use wide::CmpLt;
    let zero = wide::f32x4::splat(0.0);
    let one = wide::f32x4::splat(1.0);
    let two = wide::f32x4::splat(2.0);

    let mask1 = t.cmp_lt(dt);
    let t_norm1 = t / dt;
    let val1 = two * t_norm1 - (t_norm1 * t_norm1) - one;

    let mask2 = t.cmp_gt(one - dt);
    let t_norm2 = (t - one) / dt;
    let val2 = (t_norm2 * t_norm2) + (two * t_norm2) + one;

    let res1 = mask1.blend(val1, zero);
    let res2 = mask2.blend(val2, res1);

    let mask0 = dt.cmp_gt(zero);
    mask0.blend(res2, zero)
}

impl OscSaw {
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    pub fn process_block_simd(
        &mut self,
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        use wide::f32x4;
        let num_samples = outputs[0].len();
        let dt = self.frequency / ctx.sample_rate as f32;

        let mut i = 0;
        let offsets = f32x4::new([0.0, 1.0, 2.0, 3.0]);
        let dt_vec = f32x4::splat(dt);
        let phase_inc = offsets * dt_vec;

        while i + 3 < num_samples {
            // Apply mod 1.0 manually to each element (fract equivalent for positive numbers)
            let phases_raw = f32x4::splat(self.phase) + phase_inc;
            let phases_arr = phases_raw.to_array();
            let phases = f32x4::new([
                phases_arr[0] - phases_arr[0].trunc(),
                phases_arr[1] - phases_arr[1].trunc(),
                phases_arr[2] - phases_arr[2].trunc(),
                phases_arr[3] - phases_arr[3].trunc(),
            ]);
            
            // naive = 2.0 * phases - 1.0;
            let naive = f32x4::splat(2.0) * phases - f32x4::splat(1.0);
            let pb = poly_blep_simd(phases, dt_vec);
            let val = naive - pb;
            
            let arr = val.to_array();
            for out_ch in outputs.iter_mut() {
                out_ch[i] = arr[0];
                out_ch[i + 1] = arr[1];
                out_ch[i + 2] = arr[2];
                out_ch[i + 3] = arr[3];
            }
            
            self.phase = (self.phase + dt * 4.0) % 1.0;
            i += 4;
        }

        // remainder
        while i < num_samples {
            let val = self.process_sample(ctx.sample_rate);
            for out_ch in outputs.iter_mut() {
                out_ch[i] = val;
            }
            i += 1;
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

    pub fn trigger(&mut self, note: u8, ctx: &ProcessContext) {
        self.frequency = ctx.note_to_hz(note as i32);
    }

    pub fn process_sample(&mut self, sample_rate: u32) -> f32 {
        if sample_rate == 0 {
            return 0.0;
        }
        let dt = self.frequency / sample_rate as f32;
        let naive = if self.phase < self.pulse_width { 1.0 } else { -1.0 };
        let val = naive + poly_blep(self.phase, dt) - poly_blep((self.phase + 1.0 - self.pulse_width) % 1.0, dt);
        self.phase = (self.phase + dt) % 1.0;
        val
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

        let num_samples = outputs[0].len();
        for i in 0..num_samples {
            let val = self.process_sample(ctx.sample_rate);
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

    pub fn trigger(&mut self, note: u8, ctx: &ProcessContext) {
        self.frequency = ctx.note_to_hz(note as i32);
    }

    pub fn process_sample(&mut self, sample_rate: u32, pm_offset: f32) -> f32 {
        if sample_rate == 0 {
            return 0.0;
        }
        let dt = (TAU * self.frequency) / sample_rate as f32;
        let val = (self.phase + pm_offset).sin();
        self.phase = (self.phase + dt) % TAU;
        val
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

        let num_samples = outputs[0].len();
        for i in 0..num_samples {
            let pm_offset = if !inputs.is_empty() && !inputs[0].is_empty() && i < inputs[0].len() {
                inputs[0][i] * TAU
            } else {
                0.0
            };

            let val = self.process_sample(ctx.sample_rate, pm_offset);
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
    pub state: f32,
}

impl OscTriangle {
    pub fn new(frequency: f32) -> Self {
        Self { frequency, phase: 0.0, state: 0.0 }
    }

    pub fn trigger(&mut self, note: u8, ctx: &ProcessContext) {
        self.frequency = ctx.note_to_hz(note as i32);
    }

    pub fn process_sample(&mut self, sample_rate: u32) -> f32 {
        if sample_rate == 0 {
            return 0.0;
        }
        let dt = self.frequency / sample_rate as f32;
        
        let naive_sq = if self.phase < 0.5 { 1.0 } else { -1.0 };
        let sq = naive_sq + poly_blep(self.phase, dt) - poly_blep((self.phase + 0.5) % 1.0, dt);
        
        self.state += 4.0 * dt * sq;
        self.state *= 0.999; // leaky integrator
        
        let val = self.state;
        self.phase = (self.phase + dt) % 1.0;
        val
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

        let num_samples = outputs[0].len();
        for i in 0..num_samples {
            let val = self.process_sample(ctx.sample_rate);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    fn test_osc_saw_simd_vs_scalar() {
        use summoner_core::node::ProcessContext;
        
        let mut osc_simd = OscSaw::new(440.0);
        let mut osc_scalar = OscSaw::new(440.0);
        
        let ctx = ProcessContext {
            sample_rate: 44100,
            bpm: 120.0,
            frame_position: 0,
            is_playing: true,
            param_bus: None,
            tuning_root_hz: 440.0,
            tuning_edo_divisions: 12,
        };
        
        let mut outputs_simd = vec![vec![0.0; 1024]];
        let mut slices_simd: Vec<&mut [f32]> = outputs_simd.iter_mut().map(|v| v.as_mut_slice()).collect();
        osc_simd.process_block_simd(&mut slices_simd, &ctx);
        
        let mut outputs_scalar = vec![vec![0.0; 1024]];
        // Scalar process loop manually since process_block defaults to SIMD
        let mut i = 0;
        let num_samples = 1024;
        while i < num_samples {
            let val = osc_scalar.process_sample(ctx.sample_rate);
            outputs_scalar[0][i] = val;
            i += 1;
        }
        
        for i in 0..1024 {
            let diff = (outputs_simd[0][i] - outputs_scalar[0][i]).abs();
            assert!(diff < 1e-3, "Mismatch at index {}: SIMD {} vs Scalar {} (diff {})", i, outputs_simd[0][i], outputs_scalar[0][i], diff);
        }
    }
}
