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

use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;
use crate::traits::SignalProcessor;
use std::f32::consts::TAU;

const CHORUS_BUF_SIZE: usize = 4800; // ~100ms at 48kHz
const FLANGER_BUF_SIZE: usize = 1024; // ~20ms at 48kHz
const MAX_PHASER_STAGES: usize = 6;

/// Chorus effect using LFO-modulated delay line.
#[derive(Debug)]
pub struct EffectChorus {
    pub depth: f32, // 0.0 to 1.0
    pub rate_hz: f32,
    pub mix: f32,
    buffer: [f32; CHORUS_BUF_SIZE],
    write_pos: usize,
    lfo_phase: f32,
}

impl EffectChorus {
    pub fn new() -> Self {
        Self {
            depth: 0.5,
            rate_hz: 1.5,
            mix: 0.5,
            buffer: [0.0; CHORUS_BUF_SIZE],
            write_pos: 0,
            lfo_phase: 0.0,
        }
    }
}

impl Default for EffectChorus {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalProcessor for EffectChorus {
    fn name(&self) -> &str {
        "EffectChorus"
    }

    fn process_block(&mut self, inputs: &[&[Sample]], outputs: &mut [&mut [Sample]], ctx: &ProcessContext) {
        if outputs.is_empty() {
            return;
        }
        let sample_rate = if ctx.sample_rate > 0 { ctx.sample_rate as f32 } else { 44100.0 };
        let num_samples = outputs[0].len();
        let dt = self.rate_hz / sample_rate;

        for i in 0..num_samples {
            let in_val = if !inputs.is_empty() && !inputs[0].is_empty() && i < inputs[0].len() {
                inputs[0][i]
            } else {
                0.0
            };

            self.buffer[self.write_pos] = in_val;

            let lfo_val = (self.lfo_phase * TAU).sin() * 0.5 + 0.5;
            self.lfo_phase = (self.lfo_phase + dt) % 1.0;

            let delay_samples = (0.010 + lfo_val * 0.015 * self.depth) * sample_rate; // 10ms..25ms delay
            let read_pos = (self.write_pos as f32 + CHORUS_BUF_SIZE as f32 - delay_samples) % CHORUS_BUF_SIZE as f32;

            let r_floor = read_pos.floor() as usize;
            let frac = read_pos - r_floor as f32;
            let s0 = self.buffer[r_floor % CHORUS_BUF_SIZE];
            let s1 = self.buffer[(r_floor + 1) % CHORUS_BUF_SIZE];
            let wet = s0 + frac * (s1 - s0);

            let out_val = in_val * (1.0 - self.mix) + wet * self.mix;

            self.write_pos = (self.write_pos + 1) % CHORUS_BUF_SIZE;

            for ch in outputs.iter_mut() {
                if i < ch.len() {
                    ch[i] = out_val;
                }
            }
        }
    }
}

/// Flanger effect with feedback loop and short modulation delay.
#[derive(Debug)]
pub struct EffectFlanger {
    pub depth: f32,
    pub feedback: f32,
    pub rate_hz: f32,
    pub mix: f32,
    buffer: [f32; FLANGER_BUF_SIZE],
    write_pos: usize,
    lfo_phase: f32,
}

impl EffectFlanger {
    pub fn new() -> Self {
        Self {
            depth: 0.7,
            feedback: 0.5,
            rate_hz: 0.5,
            mix: 0.5,
            buffer: [0.0; FLANGER_BUF_SIZE],
            write_pos: 0,
            lfo_phase: 0.0,
        }
    }
}

impl Default for EffectFlanger {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalProcessor for EffectFlanger {
    fn name(&self) -> &str {
        "EffectFlanger"
    }

    fn process_block(&mut self, inputs: &[&[Sample]], outputs: &mut [&mut [Sample]], ctx: &ProcessContext) {
        if outputs.is_empty() {
            return;
        }
        let sample_rate = if ctx.sample_rate > 0 { ctx.sample_rate as f32 } else { 44100.0 };
        let num_samples = outputs[0].len();
        let dt = self.rate_hz / sample_rate;

        for i in 0..num_samples {
            let in_val = if !inputs.is_empty() && !inputs[0].is_empty() && i < inputs[0].len() {
                inputs[0][i]
            } else {
                0.0
            };

            let lfo_val = (self.lfo_phase * TAU).sin() * 0.5 + 0.5;
            self.lfo_phase = (self.lfo_phase + dt) % 1.0;

            let delay_samples = (0.001 + lfo_val * 0.005 * self.depth) * sample_rate; // 1ms..6ms delay
            let read_pos = (self.write_pos as f32 + FLANGER_BUF_SIZE as f32 - delay_samples) % FLANGER_BUF_SIZE as f32;

            let r_floor = read_pos.floor() as usize;
            let frac = read_pos - r_floor as f32;
            let s0 = self.buffer[r_floor % FLANGER_BUF_SIZE];
            let s1 = self.buffer[(r_floor + 1) % FLANGER_BUF_SIZE];
            let wet = s0 + frac * (s1 - s0);

            self.buffer[self.write_pos] = in_val + wet * self.feedback;
            self.write_pos = (self.write_pos + 1) % FLANGER_BUF_SIZE;

            let out_val = in_val * (1.0 - self.mix) + wet * self.mix;

            for ch in outputs.iter_mut() {
                if i < ch.len() {
                    ch[i] = out_val;
                }
            }
        }
    }
}

/// All-pass stage for Phaser.
#[derive(Debug, Default, Clone, Copy)]
struct AllPassStage {
    x1: f32,
    y1: f32,
}

impl AllPassStage {
    fn process(&mut self, input: f32, a: f32) -> f32 {
        let output = -a * input + self.x1 + a * self.y1;
        self.x1 = input;
        self.y1 = output;
        output
    }
}

/// Multi-stage Phaser effect using cascading all-pass filters.
#[derive(Debug)]
pub struct EffectPhaser {
    pub depth: f32,
    pub feedback: f32,
    pub rate_hz: f32,
    pub stages: u8,
    stages_buf: [AllPassStage; MAX_PHASER_STAGES],
    lfo_phase: f32,
    last_feedback: f32,
}

impl EffectPhaser {
    pub fn new() -> Self {
        Self {
            depth: 0.8,
            feedback: 0.4,
            rate_hz: 0.5,
            stages: 4,
            stages_buf: [AllPassStage::default(); MAX_PHASER_STAGES],
            lfo_phase: 0.0,
            last_feedback: 0.0,
        }
    }
}

impl Default for EffectPhaser {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalProcessor for EffectPhaser {
    fn name(&self) -> &str {
        "EffectPhaser"
    }

    fn process_block(&mut self, inputs: &[&[Sample]], outputs: &mut [&mut [Sample]], ctx: &ProcessContext) {
        if outputs.is_empty() {
            return;
        }
        let sample_rate = if ctx.sample_rate > 0 { ctx.sample_rate as f32 } else { 44100.0 };
        let num_samples = outputs[0].len();
        let dt = self.rate_hz / sample_rate;
        let num_stages = (self.stages as usize).clamp(1, MAX_PHASER_STAGES);

        for i in 0..num_samples {
            let in_val = if !inputs.is_empty() && !inputs[0].is_empty() && i < inputs[0].len() {
                inputs[0][i]
            } else {
                0.0
            };

            let lfo_val = (self.lfo_phase * TAU).sin() * 0.5 + 0.5;
            self.lfo_phase = (self.lfo_phase + dt) % 1.0;

            // Map LFO to allpass coefficient 'a'
            let min_freq = 200.0;
            let max_freq = 4000.0;
            let center_freq = min_freq + lfo_val * (max_freq - min_freq) * self.depth;
            let w0 = TAU * center_freq / sample_rate;
            let a = (1.0 - w0.tan()) / (1.0 + w0.tan());

            let mut stage_input = in_val + self.last_feedback * self.feedback;
            for st in 0..num_stages {
                stage_input = self.stages_buf[st].process(stage_input, a);
            }
            self.last_feedback = stage_input;

            let out_val = (in_val + stage_input) * 0.5;

            for ch in outputs.iter_mut() {
                if i < ch.len() {
                    ch[i] = out_val;
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
    fn test_mod_fx_processing() {
        let transport = Transport::new(44100, 120.0);
        let ctx = ProcessContext::from_transport(&transport);

        let mut chorus = EffectChorus::new();
        let mut flanger = EffectFlanger::new();
        let mut phaser = EffectPhaser::new();

        let input_buf: Vec<f32> = (0..512).map(|i| (i as f32 * 0.05).sin()).collect();
        let mut out_buf = vec![0.0f32; 512];

        chorus.process_block(&[&input_buf[..]], &mut [&mut out_buf[..]], &ctx);
        assert!(out_buf.iter().any(|&v| v != 0.0), "Chorus output should not be zero");

        flanger.process_block(&[&input_buf[..]], &mut [&mut out_buf[..]], &ctx);
        assert!(out_buf.iter().any(|&v| v != 0.0), "Flanger output should not be zero");

        phaser.process_block(&[&input_buf[..]], &mut [&mut out_buf[..]], &ctx);
        assert!(out_buf.iter().any(|&v| v != 0.0), "Phaser output should not be zero");
    }
}

