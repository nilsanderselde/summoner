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
        }
    }

    pub fn add_node(&mut self, node: Box<dyn AudioNode>) {
        self.nodes.push(node);
    }
}
