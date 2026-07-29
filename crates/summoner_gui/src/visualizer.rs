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

use crate::renderer::RenderCommand;
use crate::lod::LodLevel;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::Arc;

/// Real-time lock-free oscilloscope ring buffer for audio waveform visualization.
#[derive(Clone)]
pub struct Oscilloscope {
    pub buffer: Arc<[AtomicU32; 512]>,
    pub write_pos: Arc<AtomicUsize>,
}

impl Oscilloscope {
    pub fn new() -> Self {
        let array: [AtomicU32; 512] = std::array::from_fn(|_| AtomicU32::new(0.0f32.to_bits()));
        Self {
            buffer: Arc::new(array),
            write_pos: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn write_sample(&self, sample: f32) {
        let pos = self.write_pos.fetch_add(1, Ordering::Relaxed) % 512;
        self.buffer[pos].store(sample.to_bits(), Ordering::Relaxed);
    }

    pub fn read_all(&self) -> [f32; 512] {
        let current_pos = self.write_pos.load(Ordering::Relaxed);
        let mut out = [0.0f32; 512];
        for i in 0..512 {
            let idx = (current_pos + i) % 512;
            out[i] = f32::from_bits(self.buffer[idx].load(Ordering::Relaxed));
        }
        out
    }
}

impl Default for Oscilloscope {
    fn default() -> Self {
        Self::new()
    }
}

/// Inline visualizers to be rendered inside the signal paths.
pub struct Visualizer {
    pub width: f32,
    pub height: f32,
}

impl Visualizer {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// Emits a mock render command for an oscilloscope view.
    pub fn draw_oscilloscope(&self, track_id: u64, x: f32, _y: f32) -> RenderCommand {
        RenderCommand::DrawWaveform {
            track_id,
            x,
            width: self.width,
            sample_count: 512,
            lod: LodLevel::Medium,
        }
    }
}
