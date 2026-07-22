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
