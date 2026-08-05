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

//! Render command pipeline and LOD-throttled UI element drawing.

use crate::lod::{LodLevel, Viewport};

/// Command emitted for rendering UI components.
#[derive(Debug, Clone, PartialEq)]
pub enum RenderCommand {
    DrawWaveform {
        track_id: u64,
        x: f32,
        width: f32,
        sample_count: usize,
        lod: LodLevel,
    },
    DrawSequenceGrid {
        track_id: u64,
        x: f32,
        width: f32,
        step_count: usize,
        lod: LodLevel,
    },
    DrawTrackHeader {
        track_id: u64,
        name: String,
        lod: LodLevel,
    },
    DrawMacroRackView {
        device_id: u64,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
    DrawMicroGraphView {
        device_id: u64,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        lod: LodLevel,
    },
}

/// GUI Renderer managing viewports, command submission, and LOD vertex throttling.
#[derive(Debug, Default)]
pub struct GuiRenderer {
    pub viewport: Viewport,
    commands: Vec<RenderCommand>,
}

impl GuiRenderer {
    pub fn new(viewport: Viewport) -> Self {
        Self {
            viewport,
            commands: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }

    /// Submit a track waveform for rendering. Automatically determines visibility and LOD level.
    pub fn submit_waveform(&mut self, track_id: u64, x: f32, width: f32, total_samples: usize) {
        if !self.viewport.is_visible(x, width) {
            return;
        }

        let lod = self.viewport.lod_for_dimension(width);
        let sample_count = match lod {
            LodLevel::Full => total_samples,
            LodLevel::Medium => total_samples / 4,
            LodLevel::Overview => total_samples / 16,
            LodLevel::Minimal => 2, // bounding line
        };

        self.commands.push(RenderCommand::DrawWaveform {
            track_id,
            x,
            width,
            sample_count: sample_count.max(2),
            lod,
        });
    }

    /// Submit a sequence grid for rendering with dynamic LOD throttling.
    pub fn submit_sequence_grid(&mut self, track_id: u64, x: f32, width: f32, total_steps: usize) {
        if !self.viewport.is_visible(x, width) {
            return;
        }

        let lod = self.viewport.lod_for_dimension(width);
        self.commands.push(RenderCommand::DrawSequenceGrid {
            track_id,
            x,
            width,
            step_count: total_steps,
            lod,
        });
    }

    /// Access active submitted render command queue.
    pub fn active_commands(&self) -> &[RenderCommand] {
        &self.commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waveform_lod_throttling() {
        let viewport = Viewport::new(1000.0, 800.0);
        let mut renderer = GuiRenderer::new(viewport);

        // Submit 1000 sample waveform spanning 400px (Full LOD)
        renderer.submit_waveform(1, 100.0, 400.0, 1000);
        assert_eq!(renderer.active_commands().len(), 1);

        if let RenderCommand::DrawWaveform {
            sample_count, lod, ..
        } = &renderer.active_commands()[0]
        {
            assert_eq!(*lod, LodLevel::Full);
            assert_eq!(*sample_count, 1000);
        } else {
            panic!("Expected DrawWaveform command");
        }
    }

    #[test]
    fn test_culled_offscreen_component() {
        let viewport = Viewport::new(1000.0, 800.0);
        let mut renderer = GuiRenderer::new(viewport);

        // Component located offscreen (x: 1200..1400)
        renderer.submit_waveform(1, 1200.0, 200.0, 1000);
        assert_eq!(renderer.active_commands().len(), 0);
    }
}
