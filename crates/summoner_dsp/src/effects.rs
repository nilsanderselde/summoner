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

//! Reverb and Stereo Ping-Pong Delay audio effect processors.

use crate::traits::SignalProcessor;
use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;

/// Stereo ping-pong delay with feedback damping.
#[derive(Debug)]
pub struct EffectDelay {
    pub delay_time_sec: f32,
    pub feedback: f32,
    pub damp: f32,
    pub mix: f32,
    buffer_l: [f32; 44100],
    buffer_r: [f32; 44100],
    write_pos: usize,
    damp_l: f32,
    damp_r: f32,
}

impl EffectDelay {
    pub fn new(delay_time_sec: f32, feedback: f32, mix: f32) -> Self {
        Self {
            delay_time_sec: delay_time_sec.clamp(0.01, 1.0),
            feedback: feedback.clamp(0.0, 0.95),
            damp: 0.2,
            mix: mix.clamp(0.0, 1.0),
            buffer_l: [0.0; 44100],
            buffer_r: [0.0; 44100],
            write_pos: 0,
            damp_l: 0.0,
            damp_r: 0.0,
        }
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

        let delay_samples = (self.delay_time_sec * ctx.sample_rate as f32).clamp(1.0, 44099.0);

        let num_samples = outputs[0].len();
        for i in 0..num_samples {
            let read_pos = (self.write_pos as f32 + 44100.0 - delay_samples) % 44100.0;
            let read_idx = read_pos.floor() as usize;
            let frac = read_pos % 1.0;
            let next_idx = (read_idx + 1) % 44100;
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

            self.write_pos = (self.write_pos + 1) % 44100;

            if !outputs.is_empty() && i < outputs[0].len() {
                outputs[0][i] = out_l;
            }
            if outputs.len() > 1 && i < outputs[1].len() {
                outputs[1][i] = out_r;
            }
        }
    }
}

/// Freeverb/Schroeder algorithmic reverb processor.
const COMB_TUNINGS: [usize; 8] = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];
const ALLPASS_TUNINGS: [usize; 4] = [556, 441, 341, 225];

#[derive(Debug)]
pub struct EffectReverb {
    pub room_size: f32,
    pub damping: f32,
    pub mix: f32,
    comb_buffers: [[f32; 1618]; 8],
    comb_pos: [usize; 8],
    comb_damp: [f32; 8],
    allpass_buffers: [[f32; 557]; 4],
    allpass_pos: [usize; 4],
}

impl EffectReverb {
    pub fn new(room_size: f32, mix: f32) -> Self {
        Self {
            room_size: room_size.clamp(0.0, 0.98),
            damping: 0.2,
            mix: mix.clamp(0.0, 1.0),
            comb_buffers: [[0.0; 1618]; 8],
            comb_pos: [0; 8],
            comb_damp: [0.0; 8],
            allpass_buffers: [[0.0; 557]; 4],
            allpass_pos: [0; 4],
        }
    }

    pub fn process_sample(&mut self, input: f32) -> f32 {
        let mut comb_sum = 0.0f32;

        for (c, &buf_len) in COMB_TUNINGS.iter().enumerate() {
            let pos = self.comb_pos[c];
            let delayed = self.comb_buffers[c][pos];

            self.comb_damp[c] += self.damping * (delayed - self.comb_damp[c]);
            self.comb_buffers[c][pos] = input + self.comb_damp[c] * (0.7 + 0.28 * self.room_size);

            self.comb_pos[c] = (pos + 1) % buf_len;
            comb_sum += delayed;
        }

        let mut ap_out = comb_sum * 0.125;
        for (a, &buf_len) in ALLPASS_TUNINGS.iter().enumerate() {
            let pos = self.allpass_pos[a];
            let delayed = self.allpass_buffers[a][pos];

            let new_val = ap_out + delayed * 0.5;
            self.allpass_buffers[a][pos] = new_val;
            ap_out = delayed - new_val * 0.5;


            self.allpass_pos[a] = (pos + 1) % buf_len;
        }

        input * (1.0 - self.mix) + ap_out * self.mix
    }
}

impl SignalProcessor for EffectReverb {
    fn name(&self) -> &str {
        "EffectReverb"
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

            let rev_out = self.process_sample(in_sample);
            for out_ch in outputs.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = rev_out;
                }
            }
        }
    }
}

/// Noise Gate effect node for suppressing background noise below threshold.
#[derive(Debug)]
pub struct NoiseGateNode {
    pub threshold_db: f32,
    pub ratio: f32,
    pub attack: f32,
    pub release: f32,
    pub env_db: f32,
}

impl NoiseGateNode {
    pub fn new() -> Self {
        Self {
            threshold_db: -40.0,
            ratio: 4.0,
            attack: 0.005,
            release: 0.1,
            env_db: -120.0,
        }
    }

    pub fn with_params(threshold_db: f32, ratio: f32, attack_ms: f32, release_ms: f32) -> Self {
        Self {
            threshold_db,
            ratio: ratio.max(1.0),
            attack: (attack_ms * 0.001).max(0.0001),
            release: (release_ms * 0.001).max(0.001),
            env_db: -120.0,
        }
    }
}

impl Default for NoiseGateNode {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalProcessor for NoiseGateNode {
    fn name(&self) -> &str {
        "NoiseGateNode"
    }

    fn process_block(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        if inputs.is_empty() || outputs.is_empty() {
            return;
        }
        let num_samples = inputs[0].len().min(outputs[0].len());
        let sr = if ctx.sample_rate > 0 { ctx.sample_rate as f32 } else { 44100.0 };
        let dt = 1.0 / sr;
        let attack_coeff = (-dt / self.attack.max(0.0001)).exp();
        let release_coeff = (-dt / self.release.max(0.001)).exp();

        for i in 0..num_samples {
            let sample = inputs[0][i];
            let abs_level = sample.abs().max(1e-6);
            let level_db = 20.0 * abs_level.log10();

            let coeff = if level_db > self.env_db { attack_coeff } else { release_coeff };
            self.env_db = self.env_db * coeff + level_db * (1.0 - coeff);

            let gain = if self.env_db < self.threshold_db {
                let depth = (self.threshold_db - self.env_db) * (self.ratio - 1.0);
                10.0f32.powf(-depth / 20.0).clamp(0.0, 1.0)
            } else {
                1.0
            };

            for (ch_idx, out_ch) in outputs.iter_mut().enumerate() {
                if i < out_ch.len() {
                    let in_val = if ch_idx < inputs.len() && i < inputs[ch_idx].len() {
                        inputs[ch_idx][i]
                    } else {
                        sample
                    };
                    out_ch[i] = in_val * gain;
                }
            }
        }
    }
}

/// De-esser node for attenuating harsh sibilance frequencies.
#[derive(Debug)]
pub struct DeesserNode {
    pub frequency: f32,
    pub threshold_db: f32,
    pub ratio: f32,
    pub attack: f32,
    pub release: f32,
    pub env_db: f32,
    filter_state: f32,
}

impl DeesserNode {
    pub fn new() -> Self {
        Self {
            frequency: 6000.0,
            threshold_db: -20.0,
            ratio: 4.0,
            attack: 0.002,
            release: 0.05,
            env_db: -120.0,
            filter_state: 0.0,
        }
    }
}

impl Default for DeesserNode {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalProcessor for DeesserNode {
    fn name(&self) -> &str {
        "DeesserNode"
    }

    fn process_block(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        if inputs.is_empty() || outputs.is_empty() {
            return;
        }
        let num_samples = inputs[0].len().min(outputs[0].len());
        let sr = if ctx.sample_rate > 0 { ctx.sample_rate as f32 } else { 44100.0 };
        let dt = 1.0 / sr;
        let attack_coeff = (-dt / self.attack.max(0.0001)).exp();
        let release_coeff = (-dt / self.release.max(0.001)).exp();
        let hp_alpha = (-2.0 * std::f32::consts::PI * self.frequency * dt).exp();

        for i in 0..num_samples {
            let in_sample = inputs[0][i];
            let high_freq = in_sample - self.filter_state;
            self.filter_state = self.filter_state * hp_alpha + in_sample * (1.0 - hp_alpha);

            let sib_db = 20.0 * high_freq.abs().max(1e-6).log10();
            let coeff = if sib_db > self.env_db { attack_coeff } else { release_coeff };
            self.env_db = self.env_db * coeff + sib_db * (1.0 - coeff);

            let gain_reduction = if self.env_db > self.threshold_db {
                let over = self.env_db - self.threshold_db;
                let red_db = over * (1.0 - 1.0 / self.ratio);
                10.0f32.powf(-red_db / 20.0).clamp(0.0, 1.0)
            } else {
                1.0
            };

            for (ch_idx, out_ch) in outputs.iter_mut().enumerate() {
                if i < out_ch.len() {
                    let s = if ch_idx < inputs.len() && i < inputs[ch_idx].len() {
                        inputs[ch_idx][i]
                    } else {
                        in_sample
                    };
                    out_ch[i] = s * gain_reduction;
                }
            }
        }
    }
}

/// Harmonic Exciter node for adding synthesized high-frequency harmonics.
#[derive(Debug)]
pub struct HarmonicExciterNode {
    pub frequency: f32,
    pub drive: f32,
    pub blend: f32,
    hp_state: f32,
}

impl HarmonicExciterNode {
    pub fn new() -> Self {
        Self {
            frequency: 3000.0,
            drive: 2.0,
            blend: 0.3,
            hp_state: 0.0,
        }
    }
}

impl Default for HarmonicExciterNode {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalProcessor for HarmonicExciterNode {
    fn name(&self) -> &str {
        "HarmonicExciterNode"
    }

    fn process_block(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        if inputs.is_empty() || outputs.is_empty() {
            return;
        }
        let num_samples = inputs[0].len().min(outputs[0].len());
        let sr = if ctx.sample_rate > 0 { ctx.sample_rate as f32 } else { 44100.0 };
        let dt = 1.0 / sr;
        let hp_alpha = (-2.0 * std::f32::consts::PI * self.frequency * dt).exp();

        for i in 0..num_samples {
            let in_sample = inputs[0][i];
            let hp_signal = in_sample - self.hp_state;
            self.hp_state = self.hp_state * hp_alpha + in_sample * (1.0 - hp_alpha);

            let driven = hp_signal * self.drive;
            let harmonic = (driven.sin() + 0.5 * driven.powi(2)).clamp(-1.0, 1.0);

            for (ch_idx, out_ch) in outputs.iter_mut().enumerate() {
                if i < out_ch.len() {
                    let s = if ch_idx < inputs.len() && i < inputs[ch_idx].len() {
                        inputs[ch_idx][i]
                    } else {
                        in_sample
                    };
                    out_ch[i] = s + harmonic * self.blend;
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
    fn test_effect_delay_and_reverb() {
        let transport = Transport::new(44100, 120.0);
        let ctx = ProcessContext::from_transport(&transport);

        let mut delay = EffectDelay::new(0.1, 0.5, 0.5);
        let mut reverb = EffectReverb::new(0.8, 0.4);

        let in_buf = vec![1.0f32; 64];
        let mut out_delay = vec![0.0f32; 64];
        let mut out_rev = vec![0.0f32; 64];

        delay.process_block(&[&in_buf[..]], &mut [&mut out_delay[..]], &ctx);
        reverb.process_block(&[&in_buf[..]], &mut [&mut out_rev[..]], &ctx);

        assert!(out_delay.iter().any(|v| *v != 0.0));
        assert!(out_rev.iter().any(|v| *v != 0.0));
    }
}
