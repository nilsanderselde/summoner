// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde

use crate::traits::SignalProcessor;
use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;
use std::f32::consts::TAU;

const MAX_GRAINS: usize = 64;

#[derive(Debug, Clone, Copy, Default)]
pub struct Grain {
    pub start_pos: f32,
    pub play_head: f32,
    pub duration_samples: f32,
    pub pitch_ratio: f32,
    pub active: bool,
}

#[derive(Debug, Clone)]
pub struct GranularSynthNode {
    pub buffer: Vec<f32>,
    pub sample_rate: u32,
    pub grain_size_ms: f32,
    pub density: f32,
    pub spray: f32,
    pub pitch_jitter: f32,
    pub grains: [Grain; MAX_GRAINS],
    pub trigger_timer: f32,
    seed: u64,
}

impl GranularSynthNode {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            buffer: Vec::new(),
            sample_rate,
            grain_size_ms: 50.0,
            density: 10.0,
            spray: 0.0,
            pitch_jitter: 0.0,
            grains: [Grain::default(); MAX_GRAINS],
            trigger_timer: f32::MAX,
            seed: 0xDEADBEEF12345678,
        }
    }

    pub fn load_buffer(&mut self, data: Vec<f32>) {
        self.buffer = data;
    }

    fn next_prng(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let val = (self.seed >> 33) as f32 / 2147483648.0;
        val - 1.0 // -1.0 to 1.0
    }

    fn spawn_grain(&mut self) {
        if self.buffer.is_empty() {
            return;
        }

        let spray_offset = self.next_prng() * self.spray * self.sample_rate as f32;
        let pitch_jitter_semitones = self.next_prng() * self.pitch_jitter;

        // Find inactive slot
        if let Some(grain) = self.grains.iter_mut().find(|g| !g.active) {
            let base_start = 0.0;
            let buf_len = self.buffer.len() as f32;
            let start_pos = (base_start + spray_offset).clamp(0.0, (buf_len - 1.0).max(0.0));

            let grain_duration_sec = (self.grain_size_ms / 1000.0).max(0.005);
            let duration_samples = grain_duration_sec * self.sample_rate as f32;
            let pitch_ratio = 2.0f32.powf(pitch_jitter_semitones / 12.0);

            *grain = Grain {
                start_pos,
                play_head: 0.0,
                duration_samples,
                pitch_ratio,
                active: true,
            };
        }
    }
}

impl SignalProcessor for GranularSynthNode {
    fn name(&self) -> &str {
        "GranularSynthNode"
    }

    fn process_block(
        &mut self,
        _inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        if outputs.is_empty() || self.buffer.is_empty() {
            return;
        }

        let sample_rate = if ctx.sample_rate > 0 { ctx.sample_rate } else { self.sample_rate };
        if sample_rate == 0 {
            return;
        }

        let num_samples = outputs[0].len();
        let trigger_interval = if self.density > 0.0 {
            sample_rate as f32 / self.density
        } else {
            f32::MAX
        };

        for i in 0..num_samples {
            // Trigger check
            self.trigger_timer += 1.0;
            if self.trigger_timer >= trigger_interval {
                self.trigger_timer = 0.0;
                self.spawn_grain();
            }

            let mut mixed_sample = 0.0f32;

            for grain in self.grains.iter_mut().filter(|g| g.active) {
                let current_buf_idx = grain.start_pos + grain.play_head * grain.pitch_ratio;
                let buf_len = self.buffer.len();

                if current_buf_idx < (buf_len as f32 - 1.0) && grain.play_head < grain.duration_samples {
                    let idx_floor = current_buf_idx.floor() as usize;
                    let frac = current_buf_idx - idx_floor as f32;
                    let s0 = self.buffer[idx_floor];
                    let s1 = self.buffer[(idx_floor + 1).min(buf_len - 1)];
                    let raw_sample = s0 + frac * (s1 - s0);

                    // Hann window
                    let env_phase = grain.play_head / grain.duration_samples;
                    let env = 0.5 * (1.0 - (TAU * env_phase).cos());

                    mixed_sample += raw_sample * env;
                    grain.play_head += 1.0;
                } else {
                    grain.active = false;
                }
            }

            for out_ch in outputs.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = mixed_sample;
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
    fn test_granular_synth_node_output() {
        let mut granular = GranularSynthNode::new(44100);
        let sin_wave: Vec<f32> = (0..44100).map(|i| (i as f32 * 440.0 * TAU / 44100.0).sin()).collect();
        granular.load_buffer(sin_wave);
        granular.density = 20.0;

        let transport = Transport::new(44100, 120.0);
        let ctx = ProcessContext::from_transport(&transport);

        let mut output_buf = vec![0.0f32; 1024];
        let dummy_in: [&[Sample]; 0] = [];

        granular.process_block(&dummy_in, &mut [&mut output_buf[..]], &ctx);

        let rms: f32 = (output_buf.iter().map(|s| s * s).sum::<f32>() / output_buf.len() as f32).sqrt();
        assert!(rms > 0.0, "GranularSynthNode RMS should be greater than zero when active");
    }
}

