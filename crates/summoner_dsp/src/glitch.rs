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

//! Glitch & creative audio processors (Amplitude Gating, Stutter, Shuffle, Tape Stop/Start, Reverse).

use crate::traits::SignalProcessor;
use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;

/// Square wave amplitude gating / chopper effect.
#[derive(Debug)]
pub struct GlitchGate {
    pub rate_hz: f32,
    pub pulse_width: f32, // 0.0 to 1.0
    pub phase: f32,
}

impl GlitchGate {
    pub fn new(rate_hz: f32, pulse_width: f32) -> Self {
        Self {
            rate_hz: rate_hz.max(0.1),
            pulse_width: pulse_width.clamp(0.01, 0.99),
            phase: 0.0,
        }
    }
}

impl SignalProcessor for GlitchGate {
    fn name(&self) -> &str {
        "GlitchGate"
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

        let dt = self.rate_hz / ctx.sample_rate as f32;
        let num_samples = outputs[0].len();

        for i in 0..num_samples {
            let in_sample = if !inputs.is_empty() && !inputs[0].is_empty() && i < inputs[0].len() {
                inputs[0][i]
            } else {
                0.0
            };

            let gate_mult = if self.phase < self.pulse_width { 1.0 } else { 0.0 };
            self.phase = (self.phase + dt) % 1.0;

            let out_sample = in_sample * gate_mult;
            for out_ch in outputs.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = out_sample;
                }
            }
        }
    }
}

/// Beat-repeat / stutter buffer effect.
#[derive(Debug)]
pub struct GlitchStutter {
    pub stutter_active: bool,
    pub stutter_len_frames: usize,
    buffer: [f32; 4096],
    write_pos: usize,
    stutter_pos: usize,
    start_idx: usize,
}

impl GlitchStutter {
    pub fn new(stutter_len_frames: usize) -> Self {
        Self {
            stutter_active: false,
            stutter_len_frames: stutter_len_frames.clamp(64, 4095),
            buffer: [0.0; 4096],
            write_pos: 0,
            stutter_pos: 0,
            start_idx: 0,
        }
    }

    pub fn set_active(&mut self, active: bool) {
        if active && !self.stutter_active {
            self.stutter_pos = 0;
            self.start_idx = (self.write_pos + 4096 - self.stutter_len_frames) % 4096;
        }
        self.stutter_active = active;
    }
}

impl SignalProcessor for GlitchStutter {
    fn name(&self) -> &str {
        "GlitchStutter"
    }

    fn process_block(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        _ctx: &ProcessContext,
    ) {
        if outputs.is_empty() {
            return;
        }

        let num_samples = outputs[0].len();
        for i in 0..num_samples {
            let in_sample = if !inputs.is_empty() && !inputs[0].is_empty() && i < inputs[0].len() {
                inputs[0][i]
            } else {
                0.0
            };

            self.buffer[self.write_pos] = in_sample;
            let out_sample = if self.stutter_active {
                let read_idx = (self.start_idx + self.stutter_pos) % 4096;
                self.stutter_pos = (self.stutter_pos + 1) % self.stutter_len_frames;
                self.buffer[read_idx]
            } else {
                in_sample
            };

            self.write_pos = (self.write_pos + 1) % 4096;

            for out_ch in outputs.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = out_sample;
                }
            }
        }
    }
}

/// Micro-timing swing and shuffle delay.
#[derive(Debug)]
pub struct GlitchShuffle {
    pub shuffle_amount: f32, // 0.0 to 1.0
    buffer: [f32; 1024],
    write_pos: usize,
}

impl GlitchShuffle {
    pub fn new(shuffle_amount: f32) -> Self {
        Self {
            shuffle_amount: shuffle_amount.clamp(0.0, 1.0),
            buffer: [0.0; 1024],
            write_pos: 0,
        }
    }
}

impl SignalProcessor for GlitchShuffle {
    fn name(&self) -> &str {
        "GlitchShuffle"
    }

    fn process_block(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        _ctx: &ProcessContext,
    ) {
        if outputs.is_empty() {
            return;
        }

        let delay_frames = (self.shuffle_amount * 200.0) as usize;
        let num_samples = outputs[0].len();

        for i in 0..num_samples {
            let in_sample = if !inputs.is_empty() && !inputs[0].is_empty() && i < inputs[0].len() {
                inputs[0][i]
            } else {
                0.0
            };

            self.buffer[self.write_pos] = in_sample;
            let read_idx = (self.write_pos + 1024 - delay_frames) % 1024;
            let out_sample = self.buffer[read_idx];
            self.write_pos = (self.write_pos + 1) % 1024;

            for out_ch in outputs.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = out_sample;
                }
            }
        }
    }
}

/// Tape stop & tape start pitch deceleration/acceleration envelope processor.
#[derive(Debug)]
pub struct TapeStop {
    pub is_stopping: bool,
    pub speed: f32,          // 0.0 to 1.0
    pub stop_time_sec: f32,
    pub start_time_sec: f32,
    buffer: [f32; 8192],
    write_pos: usize,
    read_pos: f32,
}

impl TapeStop {
    pub fn new(stop_time_sec: f32, start_time_sec: f32) -> Self {
        Self {
            is_stopping: false,
            speed: 1.0,
            stop_time_sec: stop_time_sec.max(0.05),
            start_time_sec: start_time_sec.max(0.05),
            buffer: [0.0; 8192],
            write_pos: 0,
            read_pos: 0.0,
        }
    }

    pub fn trigger_stop(&mut self, stopping: bool) {
        self.is_stopping = stopping;
    }
}

impl SignalProcessor for TapeStop {
    fn name(&self) -> &str {
        "TapeStop"
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

        let dt = 1.0 / ctx.sample_rate as f32;
        let num_samples = outputs[0].len();

        for i in 0..num_samples {
            let in_sample = if !inputs.is_empty() && !inputs[0].is_empty() && i < inputs[0].len() {
                inputs[0][i]
            } else {
                0.0
            };

            self.buffer[self.write_pos] = in_sample;

            if self.is_stopping {
                self.speed = (self.speed - dt / self.stop_time_sec).max(0.0);
            } else {
                self.speed = (self.speed + dt / self.start_time_sec).min(1.0);
            }

            self.read_pos = (self.read_pos + self.speed) % 8192.0;
            let read_idx = self.read_pos.floor() as usize;
            let frac = self.read_pos % 1.0;
            let next_idx = (read_idx + 1) % 8192;

            let out_sample = if self.speed <= 0.001 {
                0.0
            } else {
                self.buffer[read_idx] * (1.0 - frac) + self.buffer[next_idx] * frac
            };

            self.write_pos = (self.write_pos + 1) % 8192;

            for out_ch in outputs.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = out_sample;
                }
            }
        }
    }
}

/// Real-time audio reverser buffer playback effect.
#[derive(Debug)]
pub struct AudioReverse {
    pub block_size: usize,
    buffer_a: [f32; 2048],
    buffer_b: [f32; 2048],
    write_a: bool,
    pos: usize,
}

impl AudioReverse {
    pub fn new(block_size: usize) -> Self {
        Self {
            block_size: block_size.clamp(64, 2048),
            buffer_a: [0.0; 2048],
            buffer_b: [0.0; 2048],
            write_a: true,
            pos: 0,
        }
    }
}

impl SignalProcessor for AudioReverse {
    fn name(&self) -> &str {
        "AudioReverse"
    }

    fn process_block(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        _ctx: &ProcessContext,
    ) {
        if outputs.is_empty() {
            return;
        }

        let num_samples = outputs[0].len();
        for i in 0..num_samples {
            let in_sample = if !inputs.is_empty() && !inputs[0].is_empty() && i < inputs[0].len() {
                inputs[0][i]
            } else {
                0.0
            };

            let out_sample = if self.write_a {
                self.buffer_a[self.pos] = in_sample;
                let rev_idx = self.block_size - 1 - self.pos;
                self.buffer_b[rev_idx]
            } else {
                self.buffer_b[self.pos] = in_sample;
                let rev_idx = self.block_size - 1 - self.pos;
                self.buffer_a[rev_idx]
            };

            self.pos += 1;
            if self.pos >= self.block_size {
                self.pos = 0;
                self.write_a = !self.write_a;
            }

            for out_ch in outputs.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = out_sample;
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
    fn test_glitch_gate() {
        let transport = Transport::new(44100, 120.0);
        let ctx = ProcessContext::from_transport(&transport);

        let mut gate = GlitchGate::new(1000.0, 0.5);
        let in_buf = vec![1.0f32; 64];
        let mut out_buf = vec![0.0f32; 64];

        gate.process_block(&[&in_buf[..]], &mut [&mut out_buf[..]], &ctx);
        assert!(out_buf.contains(&1.0));
        assert!(out_buf.contains(&0.0));
    }


    #[test]
    fn test_tape_stop_and_reverse() {
        let transport = Transport::new(44100, 120.0);
        let ctx = ProcessContext::from_transport(&transport);

        let mut tape = TapeStop::new(0.5, 0.5);
        tape.trigger_stop(true);

        let mut reverse = AudioReverse::new(64);

        let in_buf = vec![0.8f32; 64];
        let mut out_tape = vec![0.0f32; 64];
        let mut out_rev = vec![0.0f32; 64];

        tape.process_block(&[&in_buf[..]], &mut [&mut out_tape[..]], &ctx);
        reverse.process_block(&[&in_buf[..]], &mut [&mut out_rev[..]], &ctx);

        assert!(tape.speed < 1.0);
    }
}
