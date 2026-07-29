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

//! Real-time Pitch Shifter processor node using overlap-add delay modulation.

use crate::traits::SignalProcessor;
use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;

const BUFFER_SIZE: usize = 8192;
const GRAIN_SIZE: f32 = 2048.0;

/// Pitch shifter DSP node operating in semitones (-24.0 to +24.0).
#[derive(Debug)]
pub struct PitchShifterNode {
    pub semitones: f32,
    pub mix: f32,
    buffer: Vec<f32>,
    write_pos: usize,
    read_pos_a: f32,
    read_pos_b: f32,
}

impl PitchShifterNode {
    pub fn new(semitones: f32) -> Self {
        Self {
            semitones: semitones.clamp(-24.0, 24.0),
            mix: 1.0,
            buffer: vec![0.0; BUFFER_SIZE],
            write_pos: 0,
            read_pos_a: 0.0,
            read_pos_b: GRAIN_SIZE * 0.5,
        }
    }

    pub fn process_sample(&mut self, input: f32) -> f32 {
        if self.semitones == 0.0 {
            return input;
        }

        self.buffer[self.write_pos] = input;
        let ratio = 2.0f32.powf(self.semitones / 12.0);
        let speed = 1.0 - ratio;

        self.read_pos_a = (self.read_pos_a + speed + BUFFER_SIZE as f32) % BUFFER_SIZE as f32;
        self.read_pos_b = (self.read_pos_b + speed + BUFFER_SIZE as f32) % BUFFER_SIZE as f32;

        let phase_a = (self.read_pos_a / GRAIN_SIZE) % 1.0;
        let phase_b = (self.read_pos_b / GRAIN_SIZE) % 1.0;

        let win_a = (phase_a * std::f32::consts::PI).sin();
        let win_b = (phase_b * std::f32::consts::PI).sin();

        let idx_a = (self.write_pos as f32 - self.read_pos_a + BUFFER_SIZE as f32) % BUFFER_SIZE as f32;
        let idx_b = (self.write_pos as f32 - self.read_pos_b + BUFFER_SIZE as f32) % BUFFER_SIZE as f32;

        let sample_a = self.buffer[idx_a.floor() as usize % BUFFER_SIZE];
        let sample_b = self.buffer[idx_b.floor() as usize % BUFFER_SIZE];

        let shifted = (sample_a * win_a + sample_b * win_b) / (win_a + win_b + 1e-6);

        self.write_pos = (self.write_pos + 1) % BUFFER_SIZE;

        input * (1.0 - self.mix) + shifted * self.mix
    }
}

impl Default for PitchShifterNode {
    fn default() -> Self {
        Self::new(0.0)
    }
}

impl SignalProcessor for PitchShifterNode {
    fn name(&self) -> &str {
        "PitchShifterNode"
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

            let out_sample = self.process_sample(in_sample);
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
    fn test_pitch_shifter() {
        let mut shifter = PitchShifterNode::new(5.0);
        let out = shifter.process_sample(0.5);
        assert!(out.is_finite());
    }
}
