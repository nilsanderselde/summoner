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
    pub tuning_edo: Option<u32>,
    pub tuning_root_hz: Option<f32>,
    pub nodes: Vec<Box<dyn AudioNode>>,
    temp_buf_a: Vec<Vec<crate::audio::Sample>>,
    temp_buf_b: Vec<Vec<crate::audio::Sample>>,
}

impl Track {
    pub fn new(id: TrackId, name: impl Into<String>, channels: usize) -> Self {
        const MAX_INITIAL_BLOCK_SIZE: usize = 8192;
        let mut temp_buf_a = Vec::with_capacity(channels);
        let mut temp_buf_b = Vec::with_capacity(channels);
        for _ in 0..channels {
            temp_buf_a.push(vec![0.0; MAX_INITIAL_BLOCK_SIZE]);
            temp_buf_b.push(vec![0.0; MAX_INITIAL_BLOCK_SIZE]);
        }
        Self {
            id,
            name: name.into(),
            channels,
            gain: 1.0,
            pan: 0.0,
            muted: false,
            soloed: false,
            tuning_edo: None,
            tuning_root_hz: None,
            nodes: Vec::new(),
            temp_buf_a,
            temp_buf_b,
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
                out[..block_size].fill(0.0);
            }
            return;
        }

        const MAX_TRACK_CHANNELS: usize = 16;
        let channels = self.channels.min(MAX_TRACK_CHANNELS);

        for buf in &mut self.temp_buf_a {
            buf[..block_size].fill(0.0);
        }
        for buf in &mut self.temp_buf_b {
            buf[..block_size].fill(0.0);
        }

        let mut a_is_input = true;

        for (node_idx, node) in self.nodes.iter_mut().enumerate() {
            let mut in_slices: [&[crate::audio::Sample]; MAX_TRACK_CHANNELS] =
                [&[]; MAX_TRACK_CHANNELS];
            let mut out_slices: [&mut [crate::audio::Sample]; MAX_TRACK_CHANNELS] =
                std::array::from_fn(|_| &mut [][..]);

            if a_is_input {
                for (ch, buf) in self.temp_buf_a.iter().take(channels).enumerate() {
                    in_slices[ch] = &buf[..block_size];
                }
                for (ch, buf) in self.temp_buf_b.iter_mut().take(channels).enumerate() {
                    out_slices[ch] = &mut buf[..block_size];
                }
                let in_slice_param: &[&[crate::audio::Sample]] = if node_idx == 0 {
                    &[]
                } else {
                    &in_slices[..channels]
                };
                node.process(in_slice_param, &mut out_slices[..channels], ctx);
            } else {
                for (ch, buf) in self.temp_buf_b.iter().take(channels).enumerate() {
                    in_slices[ch] = &buf[..block_size];
                }
                for (ch, buf) in self.temp_buf_a.iter_mut().take(channels).enumerate() {
                    out_slices[ch] = &mut buf[..block_size];
                }
                let in_slice_param: &[&[crate::audio::Sample]] = if node_idx == 0 {
                    &[]
                } else {
                    &in_slices[..channels]
                };
                node.process(in_slice_param, &mut out_slices[..channels], ctx);
            }

            a_is_input = !a_is_input;
        }

        let final_out = if a_is_input {
            &self.temp_buf_a
        } else {
            &self.temp_buf_b
        };
        for (ch, out) in out_buffers.iter_mut().enumerate() {
            if ch < self.channels {
                out[..block_size].copy_from_slice(&final_out[ch][..block_size]);
            } else {
                out[..block_size].fill(0.0);
            }
        }
    }
}
