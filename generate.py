import os

# AGPLv3 header
header = """// Summoner - Deterministic, Headless-First DAW
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

"""

scl_rs = header + """pub struct SclTuning {
    pub title: String,
    pub num_notes: usize,
    pub cents_or_ratios: Vec<f64>,
}

impl SclTuning {
    pub fn parse(content: &str) -> Result<Self, &'static str> {
        let mut lines = content.lines().filter(|l| !l.trim().starts_with('!') && !l.trim().is_empty());
        let title = lines.next().ok_or("Missing title")?.trim().to_string();
        let num_notes_str = lines.next().ok_or("Missing number of notes")?;
        let num_notes = num_notes_str.trim().parse::<usize>().map_err(|_| "Invalid number of notes")?;
        
        let mut cents_or_ratios = Vec::new();
        for line in lines.take(num_notes) {
            let val_str = line.trim().split_whitespace().next().unwrap_or("");
            if val_str.contains('.') {
                // Cents
                let cents: f64 = val_str.parse().map_err(|_| "Invalid cents value")?;
                cents_or_ratios.push(cents);
            } else if val_str.contains('/') {
                // Ratio
                let mut parts = val_str.split('/');
                let num: f64 = parts.next().unwrap().parse().map_err(|_| "Invalid ratio num")?;
                let den: f64 = parts.next().unwrap_or("1").parse().map_err(|_| "Invalid ratio den")?;
                let cents = 1200.0 * (num / den).log2();
                cents_or_ratios.push(cents);
            } else {
                // Integer ratio
                let num: f64 = val_str.parse().map_err(|_| "Invalid integer ratio")?;
                let cents = 1200.0 * num.log2();
                cents_or_ratios.push(cents);
            }
        }
        
        Ok(Self {
            title,
            num_notes,
            cents_or_ratios,
        })
    }
}
"""

kbm_rs = header + """use std::collections::HashMap;

pub struct KbmMapping {
    pub map_size: usize,
    pub first_midi_note: i32,
    pub last_midi_note: i32,
    pub middle_note: i32,
    pub reference_note: i32,
    pub reference_freq: f64,
    pub scale_degree: i32,
    pub mapping: Vec<Option<i32>>,
}

impl KbmMapping {
    pub fn parse(content: &str) -> Result<Self, &'static str> {
        let mut lines = content.lines().filter(|l| !l.trim().starts_with('!') && !l.trim().is_empty());
        
        let map_size: usize = lines.next().ok_or("err")?.trim().parse().map_err(|_| "err")?;
        let first_midi_note: i32 = lines.next().ok_or("err")?.trim().parse().map_err(|_| "err")?;
        let last_midi_note: i32 = lines.next().ok_or("err")?.trim().parse().map_err(|_| "err")?;
        let middle_note: i32 = lines.next().ok_or("err")?.trim().parse().map_err(|_| "err")?;
        let reference_note: i32 = lines.next().ok_or("err")?.trim().parse().map_err(|_| "err")?;
        let reference_freq: f64 = lines.next().ok_or("err")?.trim().parse().map_err(|_| "err")?;
        let scale_degree: i32 = lines.next().ok_or("err")?.trim().parse().map_err(|_| "err")?;
        
        let mut mapping = Vec::new();
        for _ in 0..map_size {
            let val_str = lines.next().ok_or("err")?.trim();
            if val_str.to_lowercase() == "x" {
                mapping.push(None);
            } else {
                let deg: i32 = val_str.parse().map_err(|_| "err")?;
                mapping.push(Some(deg));
            }
        }
        
        Ok(Self {
            map_size,
            first_midi_note,
            last_midi_note,
            middle_note,
            reference_note,
            reference_freq,
            scale_degree,
            mapping,
        })
    }
}
"""

smoothing_rs = header + """pub struct SmoothParam {
    pub target: f32,
    pub current: f32,
    pub smoothing_factor: f32,
}

impl SmoothParam {
    pub fn new(initial: f32, sample_rate: f32, time_ms: f32) -> Self {
        let factor = (-1.0 / (time_ms * 0.001 * sample_rate)).exp();
        Self {
            target: initial,
            current: initial,
            smoothing_factor: factor,
        }
    }
    
    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }
    
    pub fn next_sample(&mut self) -> f32 {
        self.current = self.current * self.smoothing_factor + self.target * (1.0 - self.smoothing_factor);
        self.current
    }
}
"""

oversampling_rs = header + """pub struct Oversampler {
    factor: usize,
}

impl Oversampler {
    pub fn new(factor: usize) -> Self {
        Self { factor }
    }
    
    pub fn process_up(&mut self, input: f32) -> Vec<f32> {
        let mut out = vec![0.0; self.factor];
        out[0] = input * self.factor as f32; // simple zero-stuffing approximation for now
        out
    }
    
    pub fn process_down(&mut self, input: &[f32]) -> f32 {
        // simple averaging approximation for now
        input.iter().sum::<f32>() / self.factor as f32
    }
}
"""

biquad_rs = header + """use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;
use crate::traits::SignalProcessor;

#[derive(Clone, Copy)]
pub enum FilterType {
    Lowpass, Highpass, Bandpass, Notch, Peaking, LowShelf, HighShelf,
}

pub struct FilterBiquad {
    pub filter_type: FilterType,
    pub freq: f32,
    pub q: f32,
    pub gain_db: f32,
    pub sample_rate: f32,
    
    // coeffs
    b0: f32, b1: f32, b2: f32,
    a1: f32, a2: f32,
    
    // state
    z1: f32, z2: f32,
}

impl FilterBiquad {
    pub fn new(filter_type: FilterType, sample_rate: f32) -> Self {
        let mut b = Self {
            filter_type,
            freq: 1000.0,
            q: 0.707,
            gain_db: 0.0,
            sample_rate,
            b0: 1.0, b1: 0.0, b2: 0.0,
            a1: 0.0, a2: 0.0,
            z1: 0.0, z2: 0.0,
        };
        b.calculate_coeffs();
        b
    }
    
    pub fn calculate_coeffs(&mut self) {
        use std::f32::consts::PI;
        let w0 = 2.0 * PI * self.freq / self.sample_rate;
        let alpha = w0.sin() / (2.0 * self.q);
        let a = 10.0f32.powf(self.gain_db / 40.0);
        
        let (mut b0, mut b1, mut b2, mut a0, mut a1, mut a2) = (0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        
        match self.filter_type {
            FilterType::Lowpass => {
                let cos_w0 = w0.cos();
                b0 = (1.0 - cos_w0) / 2.0;
                b1 = 1.0 - cos_w0;
                b2 = (1.0 - cos_w0) / 2.0;
                a0 = 1.0 + alpha;
                a1 = -2.0 * cos_w0;
                a2 = 1.0 - alpha;
            }
            FilterType::Highpass => {
                let cos_w0 = w0.cos();
                b0 = (1.0 + cos_w0) / 2.0;
                b1 = -(1.0 + cos_w0);
                b2 = (1.0 + cos_w0) / 2.0;
                a0 = 1.0 + alpha;
                a1 = -2.0 * cos_w0;
                a2 = 1.0 - alpha;
            }
            FilterType::Bandpass => {
                let cos_w0 = w0.cos();
                b0 = alpha;
                b1 = 0.0;
                b2 = -alpha;
                a0 = 1.0 + alpha;
                a1 = -2.0 * cos_w0;
                a2 = 1.0 - alpha;
            }
            FilterType::Notch => {
                let cos_w0 = w0.cos();
                b0 = 1.0;
                b1 = -2.0 * cos_w0;
                b2 = 1.0;
                a0 = 1.0 + alpha;
                a1 = -2.0 * cos_w0;
                a2 = 1.0 - alpha;
            }
            FilterType::Peaking => {
                let cos_w0 = w0.cos();
                b0 = 1.0 + alpha * a;
                b1 = -2.0 * cos_w0;
                b2 = 1.0 - alpha * a;
                a0 = 1.0 + alpha / a;
                a1 = -2.0 * cos_w0;
                a2 = 1.0 - alpha / a;
            }
            FilterType::LowShelf => {
                let cos_w0 = w0.cos();
                let sq_a = a.sqrt();
                b0 = a * ((a + 1.0) - (a - 1.0) * cos_w0 + 2.0 * sq_a * alpha);
                b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w0);
                b2 = a * ((a + 1.0) - (a - 1.0) * cos_w0 - 2.0 * sq_a * alpha);
                a0 = (a + 1.0) + (a - 1.0) * cos_w0 + 2.0 * sq_a * alpha;
                a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos_w0);
                a2 = (a + 1.0) + (a - 1.0) * cos_w0 - 2.0 * sq_a * alpha;
            }
            FilterType::HighShelf => {
                let cos_w0 = w0.cos();
                let sq_a = a.sqrt();
                b0 = a * ((a + 1.0) + (a - 1.0) * cos_w0 + 2.0 * sq_a * alpha);
                b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w0);
                b2 = a * ((a + 1.0) + (a - 1.0) * cos_w0 - 2.0 * sq_a * alpha);
                a0 = (a + 1.0) - (a - 1.0) * cos_w0 + 2.0 * sq_a * alpha;
                a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cos_w0);
                a2 = (a + 1.0) - (a - 1.0) * cos_w0 - 2.0 * sq_a * alpha;
            }
        }
        
        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }
}

impl SignalProcessor for FilterBiquad {
    fn process_block(&mut self, input: &[&[Sample]], output: &mut [&mut [Sample]], _ctx: &ProcessContext) {
        if input.is_empty() || output.is_empty() { return; }
        let num_samples = input[0].len().min(output[0].len());
        
        for i in 0..num_samples {
            let x = input[0][i];
            let y = self.b0 * x + self.b1 * self.z1 + self.b2 * self.z2 - self.a1 * self.z1 - self.a2 * self.z2;
            self.z2 = self.z1;
            self.z1 = x;
            
            for out_ch in output.iter_mut() {
                out_ch[i] = y;
            }
        }
    }
}
"""

compressor_rs = header + """use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;
use crate::traits::SignalProcessor;

pub struct CompressorNode {
    pub threshold: f32,
    pub ratio: f32,
    pub attack: f32,
    pub release: f32,
    pub knee_db: f32,
    pub makeup_gain: f32,
    env: f32,
}

impl CompressorNode {
    pub fn new() -> Self {
        Self {
            threshold: -20.0,
            ratio: 4.0,
            attack: 0.01,
            release: 0.1,
            knee_db: 5.0,
            makeup_gain: 0.0,
            env: 0.0,
        }
    }
}

impl SignalProcessor for CompressorNode {
    fn process_block(&mut self, input: &[&[Sample]], output: &mut [&mut [Sample]], ctx: &ProcessContext) {
        if input.is_empty() || output.is_empty() { return; }
        let num_samples = input[0].len().min(output[0].len());
        
        let dt = 1.0 / ctx.sample_rate as f32;
        let attack_coeff = (-dt / self.attack).exp();
        let release_coeff = (-dt / self.release).exp();
        
        let has_sidechain = input.len() > 1;
        
        for i in 0..num_samples {
            let detect = if has_sidechain { input[1][i] } else { input[0][i] };
            let level_db = 20.0 * detect.abs().max(1e-5).log10();
            
            let target_env = if level_db > self.threshold {
                level_db
            } else {
                -120.0
            };
            
            let coeff = if target_env > self.env { attack_coeff } else { release_coeff };
            self.env = self.env * coeff + target_env * (1.0 - coeff);
            
            let mut gain_db = 0.0;
            if self.env > self.threshold {
                gain_db = -(self.env - self.threshold) * (1.0 - 1.0 / self.ratio);
            }
            
            let gain = 10.0f32.powf((gain_db + self.makeup_gain) / 20.0);
            
            let x = input[0][i];
            for out_ch in output.iter_mut() {
                out_ch[i] = x * gain;
            }
        }
    }
}
"""

limiter_rs = header + """use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;
use crate::traits::SignalProcessor;
use std::collections::VecDeque;

pub struct LimiterNode {
    pub ceiling: f32,
    pub release_time: f32,
    lookahead_buffer: VecDeque<f32>,
    env: f32,
}

impl LimiterNode {
    pub fn new(lookahead_samples: usize) -> Self {
        let mut buf = VecDeque::new();
        buf.resize(lookahead_samples, 0.0);
        Self {
            ceiling: 0.99,
            release_time: 0.1,
            lookahead_buffer: buf,
            env: 0.0,
        }
    }
}

impl SignalProcessor for LimiterNode {
    fn process_block(&mut self, input: &[&[Sample]], output: &mut [&mut [Sample]], ctx: &ProcessContext) {
        if input.is_empty() || output.is_empty() { return; }
        let num_samples = input[0].len().min(output[0].len());
        
        let dt = 1.0 / ctx.sample_rate as f32;
        let release_coeff = (-dt / self.release_time).exp();
        
        for i in 0..num_samples {
            let x = input[0][i];
            
            let target_env = x.abs();
            if target_env > self.env {
                self.env = target_env;
            } else {
                self.env = self.env * release_coeff;
            }
            
            self.lookahead_buffer.push_back(x);
            let delayed = self.lookahead_buffer.pop_front().unwrap();
            
            let gain = if self.env > self.ceiling {
                self.ceiling / self.env
            } else {
                1.0
            };
            
            for out_ch in output.iter_mut() {
                out_ch[i] = delayed * gain;
            }
        }
    }
}
"""

mod_fx_rs = header + """use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;
use crate::traits::SignalProcessor;

pub struct EffectChorus { pub depth: f32, pub feedback: f32 }
impl EffectChorus { pub fn new() -> Self { Self { depth: 0.0, feedback: 0.0 } } }
impl SignalProcessor for EffectChorus {
    fn process_block(&mut self, _i: &[&[Sample]], _o: &mut [&mut [Sample]], _c: &ProcessContext) {}
}

pub struct EffectFlanger { pub depth: f32, pub feedback: f32 }
impl EffectFlanger { pub fn new() -> Self { Self { depth: 0.0, feedback: 0.0 } } }
impl SignalProcessor for EffectFlanger {
    fn process_block(&mut self, _i: &[&[Sample]], _o: &mut [&mut [Sample]], _c: &ProcessContext) {}
}

pub struct EffectPhaser { pub depth: f32, pub feedback: f32 }
impl EffectPhaser { pub fn new() -> Self { Self { depth: 0.0, feedback: 0.0 } } }
impl SignalProcessor for EffectPhaser {
    fn process_block(&mut self, _i: &[&[Sample]], _o: &mut [&mut [Sample]], _c: &ProcessContext) {}
}
"""

ring_mod_rs = header + """use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;
use crate::traits::SignalProcessor;

pub struct RingModulator {
    phase: f32,
    pub freq: f32,
}
impl RingModulator {
    pub fn new() -> Self { Self { phase: 0.0, freq: 100.0 } }
}
impl SignalProcessor for RingModulator {
    fn process_block(&mut self, input: &[&[Sample]], output: &mut [&mut [Sample]], ctx: &ProcessContext) {
        if input.is_empty() || output.is_empty() { return; }
        let num_samples = input[0].len().min(output[0].len());
        let dt = 1.0 / ctx.sample_rate as f32;
        use std::f32::consts::PI;
        for i in 0..num_samples {
            let x = input[0][i];
            let mod_sig = (2.0 * PI * self.phase).sin();
            self.phase = (self.phase + self.freq * dt).fract();
            for out_ch in output.iter_mut() { out_ch[i] = x * mod_sig; }
        }
    }
}

pub struct FrequencyShifter { pub freq: f32 }
impl FrequencyShifter { pub fn new() -> Self { Self { freq: 100.0 } } }
impl SignalProcessor for FrequencyShifter {
    fn process_block(&mut self, _i: &[&[Sample]], _o: &mut [&mut [Sample]], _c: &ProcessContext) {}
}
"""

meter_rs = header + """use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;
use crate::traits::SignalProcessor;

pub struct LufsMeterNode {
    pub integrated_lufs: f32,
    pub momentary_lufs: f32,
    pub peak: f32,
    pub rms: f32,
}
impl LufsMeterNode {
    pub fn new() -> Self { Self { integrated_lufs: -70.0, momentary_lufs: -70.0, peak: 0.0, rms: 0.0 } }
}
impl SignalProcessor for LufsMeterNode {
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
        self.rms = (sum_sq / num_samples as f32).sqrt();
    }
}
"""

files = {
    r"c:\Users\Nils\Code\Summoner\crates\summoner_harmony\src\scl.rs": scl_rs,
    r"c:\Users\Nils\Code\Summoner\crates\summoner_harmony\src\kbm.rs": kbm_rs,
    r"c:\Users\Nils\Code\Summoner\crates\summoner_core\src\smoothing.rs": smoothing_rs,
    r"c:\Users\Nils\Code\Summoner\crates\summoner_dsp\src\oversampling.rs": oversampling_rs,
    r"c:\Users\Nils\Code\Summoner\crates\summoner_dsp\src\biquad.rs": biquad_rs,
    r"c:\Users\Nils\Code\Summoner\crates\summoner_dsp\src\compressor.rs": compressor_rs,
    r"c:\Users\Nils\Code\Summoner\crates\summoner_dsp\src\limiter.rs": limiter_rs,
    r"c:\Users\Nils\Code\Summoner\crates\summoner_dsp\src\mod_fx.rs": mod_fx_rs,
    r"c:\Users\Nils\Code\Summoner\crates\summoner_dsp\src\ring_mod.rs": ring_mod_rs,
    r"c:\Users\Nils\Code\Summoner\crates\summoner_dsp\src\meter.rs": meter_rs,
}

for path, content in files.items():
    with open(path, "w") as f:
        f.write(content)

print("Files created")
