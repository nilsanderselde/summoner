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

pub const WAVETABLE_SIZE: usize = 2048;

/// Wavetable oscillator supporting dual-table morphing (Step 472, 475).
#[derive(Debug, Clone)]
pub struct OscWavetable {
    pub frequency: f32,
    pub phase: f32,
    pub table: std::sync::Arc<[f32; WAVETABLE_SIZE]>,
    pub table2: Option<std::sync::Arc<[f32; WAVETABLE_SIZE]>>,
    pub morph: f32,
}

impl OscWavetable {
    pub fn new(frequency: f32, table: std::sync::Arc<[f32; WAVETABLE_SIZE]>) -> Self {
        Self {
            frequency,
            phase: 0.0,
            table,
            table2: None,
            morph: 0.0,
        }
    }

    pub fn with_table2(mut self, table2: std::sync::Arc<[f32; WAVETABLE_SIZE]>, morph: f32) -> Self {
        self.table2 = Some(table2);
        self.morph = morph;
        self
    }

    pub fn default_sine() -> std::sync::Arc<[f32; WAVETABLE_SIZE]> {
        let mut data = [0.0f32; WAVETABLE_SIZE];
        for i in 0..WAVETABLE_SIZE {
            data[i] = (2.0 * std::f32::consts::PI * i as f32 / WAVETABLE_SIZE as f32).sin();
        }
        std::sync::Arc::new(data)
    }

    pub fn default_saw() -> std::sync::Arc<[f32; WAVETABLE_SIZE]> {
        let mut data = [0.0f32; WAVETABLE_SIZE];
        for i in 0..WAVETABLE_SIZE {
            data[i] = 2.0 * (i as f32 / WAVETABLE_SIZE as f32) - 1.0;
        }
        std::sync::Arc::new(data)
    }

    pub fn default_square() -> std::sync::Arc<[f32; WAVETABLE_SIZE]> {
        let mut data = [0.0f32; WAVETABLE_SIZE];
        for i in 0..WAVETABLE_SIZE {
            data[i] = if i < WAVETABLE_SIZE / 2 { 1.0 } else { -1.0 };
        }
        std::sync::Arc::new(data)
    }

    pub fn default_triangle() -> std::sync::Arc<[f32; WAVETABLE_SIZE]> {
        let mut data = [0.0f32; WAVETABLE_SIZE];
        for i in 0..WAVETABLE_SIZE {
            let phase = i as f32 / WAVETABLE_SIZE as f32;
            data[i] = 2.0 * (2.0 * (phase - (phase + 0.5).floor())).abs() - 1.0;
        }
        std::sync::Arc::new(data)
    }

    pub fn trigger(&mut self, note: u8, ctx: &ProcessContext) {
        self.frequency = ctx.note_to_hz(note as i32);
    }

    pub fn process_sample(&mut self, sample_rate: u32) -> f32 {
        if sample_rate == 0 {
            return 0.0;
        }
        let index_float = self.phase * (WAVETABLE_SIZE as f32);
        let idx0 = index_float.floor() as usize % WAVETABLE_SIZE;
        let idx1 = (idx0 + 1) % WAVETABLE_SIZE;
        let frac = index_float - index_float.floor();

        let s1 = self.table[idx0] * (1.0 - frac) + self.table[idx1] * frac;
        let val = if let Some(ref t2) = self.table2 {
            let morph_clamped = self.morph.clamp(0.0, 1.0);
            let s2 = t2[idx0] * (1.0 - frac) + t2[idx1] * frac;
            s1 * (1.0 - morph_clamped) + s2 * morph_clamped
        } else {
            s1
        };

        let dt = self.frequency / sample_rate as f32;
        self.phase = (self.phase + dt) % 1.0;
        val
    }
}

impl SignalProcessor for OscWavetable {
    fn name(&self) -> &str {
        "OscWavetable"
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

/// Renders a 2048-sample wavetable from an input sample buffer, normalized to 1.0 peak (Step 471).
pub fn render_buffer_to_wavetable(samples: &[f32]) -> std::sync::Arc<[f32; WAVETABLE_SIZE]> {
    let mut table = [0.0f32; WAVETABLE_SIZE];
    if samples.is_empty() {
        return std::sync::Arc::new(table);
    }
    let len = samples.len();
    let mut max_abs = 1e-6f32;
    for i in 0..WAVETABLE_SIZE {
        let src_idx = (i * len) / WAVETABLE_SIZE;
        let val = samples[src_idx.min(len - 1)];
        table[i] = val;
        if val.abs() > max_abs {
            max_abs = val.abs();
        }
    }
    for i in 0..WAVETABLE_SIZE {
        table[i] /= max_abs;
    }
    std::sync::Arc::new(table)
}

pub const DEFAULT_MAX_VOICES: usize = 16;

/// Voice state for the SIMD Polyphonic Wavetable Oscillator (Step 1261).
#[derive(Debug, Clone)]
pub struct SimdPolyVoice {
    pub note: u8,
    pub frequency: f32,
    pub velocity: f32,
    pub phase: f32,
    pub active: bool,
    pub releasing: bool,
    pub env_level: f32,
    pub age: u64,
}

impl SimdPolyVoice {
    pub fn new() -> Self {
        Self {
            note: 0,
            frequency: 440.0,
            velocity: 0.0,
            phase: 0.0,
            active: false,
            releasing: false,
            env_level: 0.0,
            age: 0,
        }
    }
}

impl Default for SimdPolyVoice {
    fn default() -> Self {
        Self::new()
    }
}

/// SIMD-accelerated polyphonic wavetable oscillator with dynamic anti-aliasing interpolation (Step 1261).
#[derive(Debug, Clone)]
pub struct SimdPolyWavetableOscillator {
    pub sample_rate: u32,
    pub max_voices: usize,
    pub voices: Vec<SimdPolyVoice>,
    pub table: std::sync::Arc<[f32; WAVETABLE_SIZE]>,
    pub table2: Option<std::sync::Arc<[f32; WAVETABLE_SIZE]>>,
    pub morph: f32,
    pub release_decay_rate: f32,
    pub attack_rate: f32,
    pub age_counter: u64,
}

impl SimdPolyWavetableOscillator {
    pub fn new(sample_rate: u32) -> Self {
        let max_voices = DEFAULT_MAX_VOICES;
        let mut voices = Vec::with_capacity(max_voices);
        for _ in 0..max_voices {
            voices.push(SimdPolyVoice::new());
        }
        Self {
            sample_rate,
            max_voices,
            voices,
            table: OscWavetable::default_sine(),
            table2: None,
            morph: 0.0,
            release_decay_rate: 0.999,
            attack_rate: 0.1,
            age_counter: 0,
        }
    }

    pub fn with_table(mut self, table: std::sync::Arc<[f32; WAVETABLE_SIZE]>) -> Self {
        self.table = table;
        self
    }

    pub fn with_table2(mut self, table2: std::sync::Arc<[f32; WAVETABLE_SIZE]>, morph: f32) -> Self {
        self.table2 = Some(table2);
        self.morph = morph.clamp(0.0, 1.0);
        self
    }

    pub fn set_morph(&mut self, morph: f32) {
        self.morph = morph.clamp(0.0, 1.0);
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
    }

    pub fn note_on(&mut self, note: u8, velocity: f32) {
        let freq = 440.0 * 2.0f32.powf((note as f32 - 69.0) / 12.0);
        self.age_counter += 1;

        // Try to find an inactive voice first
        if let Some(voice) = self.voices.iter_mut().find(|v| !v.active) {
            voice.note = note;
            voice.frequency = freq;
            voice.velocity = velocity.clamp(0.0, 1.0);
            voice.phase = 0.0;
            voice.active = true;
            voice.releasing = false;
            voice.env_level = 0.0;
            voice.age = self.age_counter;
            return;
        }

        // Voice stealing: steal oldest voice
        if let Some(oldest) = self.voices.iter_mut().min_by_key(|v| v.age) {
            oldest.note = note;
            oldest.frequency = freq;
            oldest.velocity = velocity.clamp(0.0, 1.0);
            oldest.phase = 0.0;
            oldest.active = true;
            oldest.releasing = false;
            oldest.env_level = 0.0;
            oldest.age = self.age_counter;
        }
    }

    pub fn note_off(&mut self, note: u8) {
        for voice in self.voices.iter_mut().filter(|v| v.active && v.note == note) {
            voice.releasing = true;
        }
    }

    pub fn all_notes_off(&mut self) {
        for voice in self.voices.iter_mut() {
            if voice.active {
                voice.releasing = true;
            }
        }
    }

    pub fn active_voice_count(&self) -> usize {
        self.voices.iter().filter(|v| v.active).count()
    }

    /// Evaluates dynamic anti-aliasing wavetable interpolation for a single voice phase and frequency.
    pub fn interpolate_voice_sample(&self, phase: f32, frequency: f32) -> f32 {
        Self::interpolate_wavetable_sample(
            &self.table,
            self.table2.as_deref(),
            self.morph,
            self.sample_rate,
            phase,
            frequency,
        )
    }

    /// Evaluates dynamic anti-aliasing wavetable interpolation given explicit table references.
    pub fn interpolate_wavetable_sample(
        table: &[f32; WAVETABLE_SIZE],
        table2: Option<&[f32; WAVETABLE_SIZE]>,
        morph: f32,
        sample_rate: u32,
        phase: f32,
        frequency: f32,
    ) -> f32 {
        if sample_rate == 0 {
            return 0.0;
        }

        let dt = frequency / sample_rate as f32;
        let index_float = phase * (WAVETABLE_SIZE as f32);
        let idx0 = index_float.floor() as usize % WAVETABLE_SIZE;
        let idx_prev = (idx0 + WAVETABLE_SIZE - 1) % WAVETABLE_SIZE;
        let idx1 = (idx0 + 1) % WAVETABLE_SIZE;
        let idx2 = (idx0 + 2) % WAVETABLE_SIZE;
        let frac = index_float - index_float.floor();

        // 4-point Hermite cubic interpolation for table 1
        let y0 = table[idx_prev];
        let y1 = table[idx0];
        let y2 = table[idx1];
        let y3 = table[idx2];

        let c0 = y1;
        let c1 = 0.5 * (y2 - y0);
        let c2 = y0 - 2.5 * y1 + 2.0 * y2 - 0.5 * y3;
        let c3 = 0.5 * (y3 - y0) + 1.5 * (y1 - y2);
        let cubic1 = ((c3 * frac + c2) * frac + c1) * frac + c0;

        // Dynamic anti-aliasing interpolation parameter based on phase increment dt vs Nyquist threshold
        let aa_blend = (1.0 - (dt * WAVETABLE_SIZE as f32 * 1.5)).clamp(0.0, 1.0);
        let linear1 = y1 * (1.0 - frac) + y2 * frac;
        let s1 = aa_blend * cubic1 + (1.0 - aa_blend) * linear1;

        if let Some(t2) = table2 {
            let y0_2 = t2[idx_prev];
            let y1_2 = t2[idx0];
            let y2_2 = t2[idx1];
            let y3_2 = t2[idx2];

            let c0_2 = y1_2;
            let c1_2 = 0.5 * (y2_2 - y0_2);
            let c2_2 = y0_2 - 2.5 * y1_2 + 2.0 * y2_2 - 0.5 * y3_2;
            let c3_2 = 0.5 * (y3_2 - y0_2) + 1.5 * (y1_2 - y2_2);
            let cubic2 = ((c3_2 * frac + c2_2) * frac + c1_2) * frac + c0_2;

            let linear2 = y1_2 * (1.0 - frac) + y2_2 * frac;
            let s2 = aa_blend * cubic2 + (1.0 - aa_blend) * linear2;

            let morph_clamped = morph.clamp(0.0, 1.0);
            s1 * (1.0 - morph_clamped) + s2 * morph_clamped
        } else {
            s1
        }
    }

    /// Process a single audio frame across all active polyphonic voices using SIMD vectorization.
    pub fn process_sample(&mut self) -> f32 {
        if self.sample_rate == 0 {
            return 0.0;
        }

        // Collect indices of active voices
        let mut active_indices: Vec<usize> = Vec::with_capacity(self.max_voices);
        for (idx, voice) in self.voices.iter().enumerate() {
            if voice.active {
                active_indices.push(idx);
            }
        }

        if active_indices.is_empty() {
            return 0.0;
        }

        let t1_ref = &self.table;
        let t2_ref = self.table2.as_deref();
        let morph_val = self.morph;
        let sr = self.sample_rate;

        let mut mix_sample = 0.0f32;

        // Process in SIMD 4-wide batches
        let chunks = active_indices.chunks_exact(4);
        let remainder = chunks.remainder();

        for chunk in chunks {
            // Pack voice parameters into arrays for SIMD processing
            let mut phases_arr = [0.0f32; 4];
            let mut freqs_arr = [0.0f32; 4];
            let mut gains_arr = [0.0f32; 4];

            for b in 0..4 {
                let v = &self.voices[chunk[b]];
                phases_arr[b] = v.phase;
                freqs_arr[b] = v.frequency;
                gains_arr[b] = v.env_level;
            }

            use wide::f32x4;
            let phases_v = f32x4::new(phases_arr);
            let dts_v = f32x4::new([
                freqs_arr[0] / sr as f32,
                freqs_arr[1] / sr as f32,
                freqs_arr[2] / sr as f32,
                freqs_arr[3] / sr as f32,
            ]);
            let gains_v = f32x4::new(gains_arr);

            // Compute voice sample for each voice in batch
            let mut samples_arr = [0.0f32; 4];
            for b in 0..4 {
                samples_arr[b] = Self::interpolate_wavetable_sample(
                    t1_ref, t2_ref, morph_val, sr, phases_arr[b], freqs_arr[b],
                );
            }

            let samples_v = f32x4::new(samples_arr);
            let out_v = samples_v * gains_v;
            let out_arr = out_v.to_array();

            mix_sample += out_arr[0] + out_arr[1] + out_arr[2] + out_arr[3];

            // Advance voice phases using SIMD
            let next_phases_v = phases_v + dts_v;
            let next_phases = next_phases_v.to_array();

            for b in 0..4 {
                let v = &mut self.voices[chunk[b]];
                v.phase = next_phases[b] % 1.0;
            }
        }

        // Process scalar remainder
        for &idx in remainder {
            let v = &mut self.voices[idx];
            let s = Self::interpolate_wavetable_sample(
                t1_ref, t2_ref, morph_val, sr, v.phase, v.frequency,
            );
            mix_sample += s * v.env_level;

            let dt = v.frequency / sr as f32;
            v.phase = (v.phase + dt) % 1.0;
        }

        // Update voice envelopes and release state
        for voice in self.voices.iter_mut().filter(|v| v.active) {
            if voice.releasing {
                voice.env_level *= self.release_decay_rate;
                if voice.env_level < 1e-4 {
                    voice.active = false;
                    voice.releasing = false;
                    voice.env_level = 0.0;
                }
            } else {
                voice.env_level = (voice.env_level + self.attack_rate).min(voice.velocity);
            }
        }

        mix_sample
    }
}

impl SignalProcessor for SimdPolyWavetableOscillator {
    fn name(&self) -> &str {
        "SimdPolyWavetableOscillator"
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
        self.set_sample_rate(ctx.sample_rate);
        let num_samples = outputs[0].len();
        for i in 0..num_samples {
            let val = self.process_sample();
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
    fn test_simd_scalar_agreement_osc_saw() {
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
        let mut i = 0;
        while i < 1024 {
            let val = osc_scalar.process_sample(ctx.sample_rate);
            outputs_scalar[0][i] = val;
            i += 1;
        }
        
        for i in 0..1024 {
            let diff = (outputs_simd[0][i] - outputs_scalar[0][i]).abs();
            assert!(diff < 1e-3, "Mismatch at index {}: SIMD {} vs Scalar {} (diff {})", i, outputs_simd[0][i], outputs_scalar[0][i], diff);
        }
    }

    #[test]
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    fn test_osc_saw_simd_vs_scalar() {
        test_simd_scalar_agreement_osc_saw();
    }

    #[test]
    fn test_osc_wavetable_basic() {
        let sine_table = OscWavetable::default_sine();
        let mut osc = OscWavetable::new(440.0, sine_table);
        let sample = osc.process_sample(44100);
        assert!(sample.abs() <= 1.0);

        let mut output = vec![vec![0.0f32; 512]];
        let mut slices: Vec<&mut [f32]> = output.iter_mut().map(|v| v.as_mut_slice()).collect();
        let ctx = ProcessContext::new(44100, 120.0, 0);
        osc.process_block(&[], &mut slices, &ctx);
        let rms: f32 = (slices[0].iter().map(|s| s * s).sum::<f32>() / 512.0).sqrt();
        assert!(rms > 0.1, "Wavetable oscillator RMS should be > 0.1");
    }

    #[test]
    fn test_osc_wavetable_morph() {
        let saw_table = OscWavetable::default_saw();
        let square_table = OscWavetable::default_square();

        let mut osc0 = OscWavetable::new(440.0, saw_table.clone())
            .with_table2(square_table.clone(), 0.0);
        let mut osc1 = OscWavetable::new(440.0, saw_table.clone())
            .with_table2(square_table.clone(), 1.0);

        let sample0 = osc0.process_sample(44100);
        let sample1 = osc1.process_sample(44100);
        assert!((sample0 - sample1).abs() > 0.5, "Morphing should produce different outputs");
    }

    #[test]
    fn test_step_1261_simd_poly_wavetable_oscillator() {
        let mut synth = SimdPolyWavetableOscillator::new(44100);
        assert_eq!(synth.active_voice_count(), 0);

        // 1. Play 4-note chord (C4, E4, G4, B4) to test SIMD 4-wide batch processing
        synth.note_on(60, 0.8);
        synth.note_on(64, 0.8);
        synth.note_on(67, 0.8);
        synth.note_on(71, 0.8);
        assert_eq!(synth.active_voice_count(), 4);

        let mut block = vec![vec![0.0f32; 256]];
        let mut slices: Vec<&mut [f32]> = block.iter_mut().map(|v| v.as_mut_slice()).collect();
        let ctx = ProcessContext::new(44100, 120.0, 0);

        synth.process_block(&[], &mut slices, &ctx);

        let energy: f32 = slices[0].iter().map(|s| s * s).sum();
        assert!(energy > 1.0, "Polyphonic SIMD oscillator block output energy should be non-zero");

        // 2. Test table morphing
        let saw_table = OscWavetable::default_saw();
        let square_table = OscWavetable::default_square();
        let mut morph_synth = SimdPolyWavetableOscillator::new(44100)
            .with_table(saw_table)
            .with_table2(square_table, 0.5);

        morph_synth.note_on(60, 1.0);
        let sample = morph_synth.process_sample();
        assert!(sample.is_finite());

        // 3. Test note_off and envelope release
        synth.note_off(60);
        synth.note_off(64);
        synth.note_off(67);
        synth.note_off(71);

        for _ in 0..10000 {
            synth.process_sample();
        }

        assert_eq!(synth.active_voice_count(), 0, "All voices should decay and become inactive after release");
    }
}

