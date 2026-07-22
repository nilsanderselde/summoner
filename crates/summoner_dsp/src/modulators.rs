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

//! Modulator DSP primitives (EnvADSR, LFO, MacroKnob).

use crate::traits::SignalProcessor;
use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;
use std::f32::consts::TAU;

/// ADSR envelope generator state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

/// 4-stage exponential envelope generator.
#[derive(Debug, Clone)]
pub struct EnvADSR {
    pub attack: f32,  // in seconds
    pub decay: f32,   // in seconds
    pub sustain: f32, // level 0.0 to 1.0
    pub release: f32, // in seconds
    pub state: EnvState,
    pub level: f32,
    pub gate: bool,
}

impl EnvADSR {
    pub fn new(attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        Self {
            attack: attack.max(0.001),
            decay: decay.max(0.001),
            sustain: sustain.clamp(0.0, 1.0),
            release: release.max(0.001),
            state: EnvState::Idle,
            level: 0.0,
            gate: false,
        }
    }

    pub fn trigger(&mut self, gate: bool) {
        if gate && !self.gate {
            self.state = EnvState::Attack;
        } else if !gate && self.gate {
            self.state = EnvState::Release;
        }
        self.gate = gate;
    }

    pub fn process_sample(&mut self, sample_rate: u32) -> f32 {
        if sample_rate == 0 {
            return self.level;
        }

        let dt = 1.0 / sample_rate as f32;

        match self.state {
            EnvState::Idle => {
                self.level = 0.0;
            }
            EnvState::Attack => {
                self.level += dt / self.attack;
                if self.level >= 1.0 {
                    self.level = 1.0;
                    self.state = EnvState::Decay;
                }
            }
            EnvState::Decay => {
                self.level -= dt / self.decay * (1.0 - self.sustain);
                if self.level <= self.sustain {
                    self.level = self.sustain;
                    self.state = EnvState::Sustain;
                }
            }
            EnvState::Sustain => {
                self.level = self.sustain;
            }
            EnvState::Release => {
                self.level -= dt / self.release;
                if self.level <= 0.0 {
                    self.level = 0.0;
                    self.state = EnvState::Idle;
                }
            }
        }

        self.level.clamp(0.0, 1.0)
    }
}

impl SignalProcessor for EnvADSR {
    fn name(&self) -> &str {
        "EnvADSR"
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
            if !inputs.is_empty() && !inputs[0].is_empty() && i < inputs[0].len() {
                self.trigger(inputs[0][i] > 0.5);
            }

            let env_val = self.process_sample(ctx.sample_rate);
            for out_ch in outputs.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = env_val;
                }
            }
        }
    }
}

/// LFO waveform type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LfoShape {
    Sine,
    Triangle,
    Square,
    SampleAndHold,
}

/// Low-Frequency Oscillator (LFO).
#[derive(Debug)]
pub struct LFO {
    pub frequency: f32,
    pub shape: LfoShape,
    pub phase: f32,
    sh_value: f32,
    seed: u64,
}

impl LFO {
    pub fn new(frequency: f32, shape: LfoShape) -> Self {
        Self {
            frequency,
            shape,
            phase: 0.0,
            sh_value: 0.0,
            seed: 0x9E3779B97F4A7C15,
        }
    }

    pub fn process_sample(&mut self, sample_rate: u32) -> f32 {
        if sample_rate == 0 {
            return 0.0;
        }

        let dt = self.frequency / sample_rate as f32;
        let prev_phase = self.phase;
        self.phase = (self.phase + dt) % 1.0;

        match self.shape {
            LfoShape::Sine => (self.phase * TAU).sin(),
            LfoShape::Triangle => 2.0 * (2.0 * (self.phase - (self.phase + 0.5).floor())).abs() - 1.0,
            LfoShape::Square => if self.phase < 0.5 { 1.0 } else { -1.0 },
            LfoShape::SampleAndHold => {
                if self.phase < prev_phase {
                    self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                    self.sh_value = ((self.seed >> 33) as f32 / 2147483648.0) - 1.0;
                }
                self.sh_value
            }
        }
    }
}

impl SignalProcessor for LFO {
    fn name(&self) -> &str {
        "LFO"
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
            let lfo_val = self.process_sample(ctx.sample_rate);
            for out_ch in outputs.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = lfo_val;
                }
            }
        }
    }
}

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

/// MacroKnob bridging GUI/Automation parameters to DSP modulation inputs.
#[derive(Debug, Clone)]
pub struct MacroKnob {
    pub value: f32,
    pub atomic_binding: Option<Arc<AtomicU32>>,
}

impl MacroKnob {
    pub fn new(value: f32) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            atomic_binding: None,
        }
    }

    pub fn bind_atomic(&mut self, binding: Arc<AtomicU32>) {
        self.atomic_binding = Some(binding);
    }

    pub fn get_value(&self) -> f32 {
        if let Some(ref binding) = self.atomic_binding {
            f32::from_bits(binding.load(Ordering::Relaxed))
        } else {
            self.value
        }
    }
}

impl SignalProcessor for MacroKnob {
    fn name(&self) -> &str {
        "MacroKnob"
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
        let current_val = self.get_value();
        let num_samples = outputs[0].len();
        for i in 0..num_samples {
            for out_ch in outputs.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = current_val;
                }
            }
        }
    }
}

