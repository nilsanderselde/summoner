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

//! Universal N-channel track abstraction and node processing chain container.

use crate::node::AudioNode;

/// Track unique identifier type.
pub type TrackId = u64;

/// Track representation holding node graph chain, routing, and channel layout.
pub struct Track {
    pub id: TrackId,
    pub name: String,
    pub channels: usize,
    pub gain: f32,
    pub pan: f32,
    pub muted: bool,
    pub soloed: bool,
    pub nodes: Vec<Box<dyn AudioNode>>,
    temp_buf_a: Vec<Vec<crate::audio::Sample>>,
    temp_buf_b: Vec<Vec<crate::audio::Sample>>,
}

impl Track {
    pub fn new(id: TrackId, name: impl Into<String>, channels: usize) -> Self {
        Self {
            id,
            name: name.into(),
            channels,
            gain: 1.0,
            pan: 0.0,
            muted: false,
            soloed: false,
            nodes: Vec::new(),
            temp_buf_a: Vec::new(),
            temp_buf_b: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: Box<dyn AudioNode>) {
        self.nodes.push(node);
    }

    pub fn process(
        &mut self,
        block_size: usize,
        ctx: &crate::node::ProcessContext,
        out_buffers: &mut [&mut [crate::audio::Sample]],
    ) {
        if self.muted || self.nodes.is_empty() {
            for out in out_buffers.iter_mut() {
                out.fill(0.0);
            }
            return;
        }

        while self.temp_buf_a.len() < self.channels {
            self.temp_buf_a.push(vec![0.0; block_size]);
        }
        while self.temp_buf_b.len() < self.channels {
            self.temp_buf_b.push(vec![0.0; block_size]);
        }
        for ch in 0..self.channels {
            if self.temp_buf_a[ch].len() < block_size {
                self.temp_buf_a[ch].resize(block_size, 0.0);
            }
            if self.temp_buf_b[ch].len() < block_size {
                self.temp_buf_b[ch].resize(block_size, 0.0);
            }
            self.temp_buf_a[ch][..block_size].fill(0.0);
        }

        let mut a_is_input = true;

        for node in &mut self.nodes {
            let (input, output) = if a_is_input {
                (&mut self.temp_buf_a, &mut self.temp_buf_b)
            } else {
                (&mut self.temp_buf_b, &mut self.temp_buf_a)
            };

            let in_slices: Vec<&[crate::audio::Sample]> = input.iter().map(|v| &v[..block_size]).collect();
            let mut out_slices: Vec<&mut [crate::audio::Sample]> = output.iter_mut().map(|v| &mut v[..block_size]).collect();

            node.process(&in_slices, &mut out_slices, ctx);

            a_is_input = !a_is_input;
        }

        let final_out = if a_is_input { &self.temp_buf_a } else { &self.temp_buf_b };
        for (ch, out) in out_buffers.iter_mut().enumerate() {
            if ch < self.channels {
                out[..block_size].copy_from_slice(&final_out[ch][..block_size]);
            } else {
                out.fill(0.0);
            }
        }
    }
}
