// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Tape saturation DSP node simulating analog tape drive, hysteresis, and flutter.

use crate::traits::SignalProcessor;
use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;

/// Tape saturation simulation node.
#[derive(Debug)]
pub struct TapeSaturationNode {
    pub drive: f32,
    pub saturation: f32,
    pub tone: f32,
    pub wow_flutter: f32,
    lfo_phase: f32,
    lp_state: f32,
}

impl TapeSaturationNode {
    pub fn new(drive: f32, saturation: f32) -> Self {
        Self {
            drive: drive.max(1.0),
            saturation: saturation.clamp(0.0, 1.0),
            tone: 0.8,
            wow_flutter: 0.1,
            lfo_phase: 0.0,
            lp_state: 0.0,
        }
    }
}

impl Default for TapeSaturationNode {
    fn default() -> Self {
        Self::new(2.0, 0.5)
    }
}

impl SignalProcessor for TapeSaturationNode {
    fn name(&self) -> &str {
        "TapeSaturationNode"
    }

    fn process_block(&mut self, input: &[&[Sample]], output: &mut [&mut [Sample]], ctx: &ProcessContext) {
        if input.is_empty() || output.is_empty() {
            return;
        }

        let num_samples = input[0].len().min(output[0].len());
        let sr = if ctx.sample_rate > 0 { ctx.sample_rate as f32 } else { 44100.0 };

        for i in 0..num_samples {
            // LFO for subtle wow & flutter
            self.lfo_phase += 5.0 / sr;
            if self.lfo_phase > 1.0 {
                self.lfo_phase -= 1.0;
            }
            let flutter_mod = 1.0 + (self.lfo_phase * std::f32::consts::TAU).sin() * 0.002 * self.wow_flutter;

            let in_sample = input[0][i] * self.drive * flutter_mod;

            // Soft-clipping hyperbolic tangent tape curve
            let saturated = (in_sample * (1.0 + self.saturation)).tanh() / (1.0 + self.saturation * 0.5);

            // Tone high-frequency attenuation damping
            let alpha = 1.0 - (self.tone * 0.5);
            self.lp_state = self.lp_state * (1.0 - alpha) + saturated * alpha;

            for out_ch in output.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = self.lp_state;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tape_saturation_saturates_signal() {
        let mut tape = TapeSaturationNode::new(4.0, 0.8);
        let ctx = ProcessContext::new(44100, 120.0, 0);

        let input_sig = vec![0.9f32; 128];
        let mut out_sig = vec![0.0f32; 128];

        tape.process_block(&[&input_sig[..]], &mut [&mut out_sig[..]], &ctx);

        assert!(out_sig.iter().all(|s| s.is_finite()));
        assert!(out_sig[127].abs() < 1.0, "Tape saturation should soft clip loud inputs below 1.0");
    }
}
