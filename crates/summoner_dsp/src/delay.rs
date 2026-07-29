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

//! Stereo ping-pong delay DSP processor with max 2000ms circular buffer.

use crate::traits::SignalProcessor;
use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;

const MAX_DELAY_SAMPLES: usize = 96000; // 2000 ms @ 48kHz

/// Stereo ping-pong delay processor with circular buffer max 2000ms.
#[derive(Debug)]
pub struct EffectDelay {
    pub delay_time_sec: f32,
    pub feedback: f32,
    pub damp: f32,
    pub mix: f32,
    buffer_l: Vec<f32>,
    buffer_r: Vec<f32>,
    write_pos: usize,
    damp_l: f32,
    damp_r: f32,
}

impl EffectDelay {
    pub fn new(delay_time_sec: f32, feedback: f32, mix: f32) -> Self {
        Self {
            delay_time_sec: delay_time_sec.clamp(0.001, 2.0),
            feedback: feedback.clamp(0.0, 0.98),
            damp: 0.2,
            mix: mix.clamp(0.0, 1.0),
            buffer_l: vec![0.0; MAX_DELAY_SAMPLES],
            buffer_r: vec![0.0; MAX_DELAY_SAMPLES],
            write_pos: 0,
            damp_l: 0.0,
            damp_r: 0.0,
        }
    }
}

impl Default for EffectDelay {
    fn default() -> Self {
        Self::new(0.3, 0.4, 0.3)
    }
}

impl SignalProcessor for EffectDelay {
    fn name(&self) -> &str {
        "EffectDelay"
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

        let max_samples = MAX_DELAY_SAMPLES as f32;
        let delay_samples = (self.delay_time_sec * ctx.sample_rate as f32).clamp(1.0, max_samples - 1.0);

        let num_samples = outputs[0].len();
        for i in 0..num_samples {
            let read_pos = (self.write_pos as f32 + max_samples - delay_samples) % max_samples;
            let read_idx = read_pos.floor() as usize;
            let frac = read_pos % 1.0;
            let next_idx = (read_idx + 1) % MAX_DELAY_SAMPLES;
            let in_l = if !inputs.is_empty() && !inputs[0].is_empty() && i < inputs[0].len() {
                inputs[0][i]
            } else {
                0.0
            };
            let in_r = if inputs.len() > 1 && !inputs[1].is_empty() && i < inputs[1].len() {
                inputs[1][i]
            } else {
                in_l
            };

            let delayed_l = self.buffer_l[read_idx] * (1.0 - frac) + self.buffer_l[next_idx] * frac;
            let delayed_r = self.buffer_r[read_idx] * (1.0 - frac) + self.buffer_r[next_idx] * frac;

            // Simple lowpass damping on feedback path
            self.damp_l += self.damp * (delayed_l - self.damp_l);
            self.damp_r += self.damp * (delayed_r - self.damp_r);

            // Cross feedback for ping-pong effect
            self.buffer_l[self.write_pos] = in_l + self.damp_r * self.feedback;
            self.buffer_r[self.write_pos] = in_r + self.damp_l * self.feedback;

            let out_l = in_l * (1.0 - self.mix) + delayed_l * self.mix;
            let out_r = in_r * (1.0 - self.mix) + delayed_r * self.mix;

            self.write_pos = (self.write_pos + 1) % MAX_DELAY_SAMPLES;

            if !outputs.is_empty() && i < outputs[0].len() {
                outputs[0][i] = out_l;
            }
            if outputs.len() > 1 && i < outputs[1].len() {
                outputs[1][i] = out_r;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::transport::Transport;

    #[test]
    fn test_effect_delay_2000ms_max() {
        let transport = Transport::new(44100, 120.0);
        let ctx = ProcessContext::from_transport(&transport);

        let mut delay = EffectDelay::new(1.5, 0.5, 0.5);
        let in_buf = vec![1.0f32; 64];
        let mut out_delay = vec![0.0f32; 64];

        delay.process_block(&[&in_buf[..]], &mut [&mut out_delay[..]], &ctx);
        assert!(out_delay.iter().all(|v| v.is_finite()));
    }
}
