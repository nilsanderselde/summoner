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

//! Math, logic, and VCA DSP primitives.

use crate::traits::SignalProcessor;
use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;

/// Summing mixer adding input signals together.
#[derive(Debug, Default)]
pub struct MathAdd;

impl SignalProcessor for MathAdd {
    fn name(&self) -> &str {
        "MathAdd"
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
            let mut sum: f32 = 0.0;
            for input_ch in inputs {
                if i < input_ch.len() {
                    sum += input_ch[i];
                }
            }

            for out_ch in outputs.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = sum;
                }
            }
        }
    }
}

/// Signal multiplier (Ring Modulator / Amplitude Scaler).
#[derive(Debug, Default)]
pub struct MathMult;

impl SignalProcessor for MathMult {
    fn name(&self) -> &str {
        "MathMult"
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
            let mut prod: f32 = 1.0;
            if inputs.is_empty() {
                prod = 0.0;
            } else {
                for input_ch in inputs {
                    if i < input_ch.len() {
                        prod *= input_ch[i];
                    }
                }
            }

            for out_ch in outputs.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = prod;
                }
            }
        }
    }
}

/// Voltage Controlled Amplifier (VCA).
#[derive(Debug)]
pub struct VCA {
    pub gain: f32,
}

impl VCA {
    pub fn new(gain: f32) -> Self {
        Self { gain }
    }
}

impl Default for VCA {
    fn default() -> Self {
        Self::new(1.0)
    }
}

impl SignalProcessor for VCA {
    fn name(&self) -> &str {
        "VCA"
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
        let sig_ch = inputs.first();
        let ctrl_ch = inputs.get(1);

        for i in 0..num_samples {
            let audio_in = sig_ch.and_then(|ch| ch.get(i)).copied().unwrap_or(0.0);
            let ctrl_in = ctrl_ch.and_then(|ch| ch.get(i)).copied().unwrap_or(1.0);

            let out_sample = audio_in * ctrl_in * self.gain;
            for out_ch in outputs.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = out_sample;
                }
            }
        }
    }
}
