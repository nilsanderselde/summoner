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

use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;
use crate::traits::SignalProcessor;
use std::sync::Arc;

/// A shared buffer holding audio samples (e.g., loaded from a WAV file).
pub struct SampleBuffer {
    pub data: Vec<Sample>,
    pub sample_rate: u32,
    pub channels: usize,
}

impl SampleBuffer {
    pub fn new(data: Vec<Sample>, sample_rate: u32, channels: usize) -> Self {
        Self {
            data,
            sample_rate,
            channels,
        }
    }

    /// Get the sample rate of this buffer (for resampling calculations).
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

/// A Sampler node that plays back a loaded `SampleBuffer`.
pub struct SamplerNode {
    buffer: Option<Arc<SampleBuffer>>,
    playback_position: f64, // Floating point for interpolation
    playback_rate: f64,
    playing: bool,
}

impl SamplerNode {
    pub fn new() -> Self {
        Self {
            buffer: None,
            playback_position: 0.0,
            playback_rate: 1.0,
            playing: false,
        }
    }

    pub fn set_buffer(&mut self, buffer: Arc<SampleBuffer>) {
        self.buffer = Some(buffer);
        self.playback_position = 0.0;
    }

    pub fn trigger(&mut self, rate: f64) {
        self.playback_rate = rate;
        self.playback_position = 0.0;
        self.playing = true;
    }
    
    pub fn stop(&mut self) {
        self.playing = false;
    }
}

impl SignalProcessor for SamplerNode {
    fn name(&self) -> &str {
        "SamplerNode"
    }

    fn process_block(
        &mut self,
        _inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        _ctx: &ProcessContext,
    ) {
        let block_size = outputs[0].len();

        if let Some(buf) = &self.buffer {
            if self.playing {
                let channels = outputs.len().min(buf.channels);
                
                for i in 0..block_size {
                    let pos_int = self.playback_position as usize;
                    
                    if pos_int >= buf.data.len() / buf.channels {
                        self.playing = false;
                        for ch in 0..outputs.len() {
                            outputs[ch][i..block_size].fill(0.0);
                        }
                        break;
                    }
                    
                    // Simple nearest-neighbor interpolation for scaffold
                    for ch in 0..channels {
                        outputs[ch][i] = buf.data[pos_int * buf.channels + ch];
                    }
                    
                    self.playback_position += self.playback_rate;
                }
            } else {
                for out in outputs.iter_mut() {
                    out.fill(0.0);
                }
            }
        } else {
            for out in outputs.iter_mut() {
                out.fill(0.0);
            }
        }
    }
}
