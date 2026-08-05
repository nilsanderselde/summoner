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

        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        {
            use wide::f32x4;
            let mut i = 0;
            while i + 3 < num_samples {
                let mut sum = f32x4::splat(0.0);
                for input_ch in inputs.iter() {
                    if i + 3 < input_ch.len() {
                        let val = f32x4::new([
                            input_ch[i],
                            input_ch[i + 1],
                            input_ch[i + 2],
                            input_ch[i + 3],
                        ]);
                        sum += val;
                    }
                }
                let arr = sum.to_array();
                for out_ch in outputs.iter_mut() {
                    if i + 3 < out_ch.len() {
                        out_ch[i] = arr[0];
                        out_ch[i + 1] = arr[1];
                        out_ch[i + 2] = arr[2];
                        out_ch[i + 3] = arr[3];
                    }
                }
                i += 4;
            }
            // remainder
            while i < num_samples {
                let mut sum: f32 = 0.0;
                for input_ch in inputs.iter() {
                    if i < input_ch.len() {
                        sum += input_ch[i];
                    }
                }
                for out_ch in outputs.iter_mut() {
                    if i < out_ch.len() {
                        out_ch[i] = sum;
                    }
                }
                i += 1;
            }
        }

        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    fn test_simd_scalar_agreement_math_add() {
        let mut math_add = MathAdd;
        let ctx = ProcessContext {
            sample_rate: 44100,
            bpm: 120.0,
            frame_position: 0,
            is_playing: true,
            param_bus: None,
            tuning_root_hz: 440.0,
            tuning_edo_divisions: 12,
        };

        let in1: Vec<f32> = (0..64).map(|i| i as f32 * 0.5).collect();
        let in2: Vec<f32> = (0..64).map(|i| i as f32 * 1.5).collect();
        let inputs = [in1.as_slice(), in2.as_slice()];

        let mut out = vec![0.0f32; 64];
        let mut outputs = [out.as_mut_slice()];

        math_add.process_block(&inputs, &mut outputs, &ctx);

        for i in 0..64 {
            let expected = in1[i] + in2[i];
            let diff = (outputs[0][i] - expected).abs();
            assert!(
                diff < 1e-6,
                "MathAdd mismatch at {}: output {} vs expected {}",
                i,
                outputs[0][i],
                expected
            );
        }
    }
}
