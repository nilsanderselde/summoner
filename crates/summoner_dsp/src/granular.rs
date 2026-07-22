// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde

use crate::traits::SignalProcessor;
use summoner_core::audio::Sample;

#[derive(Debug, Clone)]
pub struct GranularSynthNode {
    pub buffer: Vec<f32>,
    pub sample_rate: u32,
    pub grain_size_ms: f32,
    pub density: f32,
    pub spray: f32,
    pub pitch_jitter: f32,
    // Add internal state for active grains
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
        }
    }

    pub fn load_buffer(&mut self, data: Vec<f32>) {
        self.buffer = data;
    }
}

use summoner_core::node::ProcessContext;

impl SignalProcessor for GranularSynthNode {
    fn name(&self) -> &str {
        "GranularSynthNode"
    }

    fn process_block(
        &mut self,
        _inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        _ctx: &ProcessContext,
    ) {
        if self.buffer.is_empty() {
            return;
        }
        for out in outputs.iter_mut() {
            for sample in out.iter_mut() {
                *sample = 0.0;
            }
        }
    }
}
