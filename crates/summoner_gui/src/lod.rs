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

//! Level-of-Detail (LOD) UI throttling and viewport scale calculation.

/// Level-of-Detail rendering tiers for UI throttling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LodLevel {
    /// Full rendering detail (individual waveform samples, micro-timing, full control labels).
    Full,
    /// Medium detail (decimated waveform peaks, simplified step indicators).
    Medium,
    /// Low detail overview blocks.
    Overview,
    /// Culled / minimal representation (bounding box outline only).
    Minimal,
}

/// Dynamic viewport tracking screen boundaries and zoom scale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    /// Zoom scale factor (higher means zoomed in).
    pub zoom_factor: f32,
}

impl Viewport {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width,
            height,
            zoom_factor: 1.0,
        }
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom_factor = zoom.max(0.01);
    }

    /// Determine LOD tier for a component based on its screen-space pixel dimension and viewport zoom.
    pub fn lod_for_dimension(&self, pixel_size: f32) -> LodLevel {
        let scaled_size = pixel_size * self.zoom_factor;
        if scaled_size >= 200.0 {
            LodLevel::Full
        } else if scaled_size >= 80.0 {
            LodLevel::Medium
        } else if scaled_size >= 20.0 {
            LodLevel::Overview
        } else {
            LodLevel::Minimal
        }
    }

    /// Check if component screen bounds overlap viewport rect.
    pub fn is_visible(&self, component_x: f32, component_width: f32) -> bool {
        let right = component_x + component_width;
        let viewport_right = self.x + self.width;
        right >= self.x && component_x <= viewport_right
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new(1920.0, 1080.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lod_level_thresholds() {
        let mut viewport = Viewport::new(1920.0, 1080.0);
        viewport.set_zoom(1.0);

        assert_eq!(viewport.lod_for_dimension(250.0), LodLevel::Full);
        assert_eq!(viewport.lod_for_dimension(100.0), LodLevel::Medium);
        assert_eq!(viewport.lod_for_dimension(30.0), LodLevel::Overview);
        assert_eq!(viewport.lod_for_dimension(10.0), LodLevel::Minimal);
    }

    #[test]
    fn test_zoom_scaling_lod() {
        let mut viewport = Viewport::new(1920.0, 1080.0);
        // Zoom out (scale 0.1x)
        viewport.set_zoom(0.1);
        assert_eq!(viewport.lod_for_dimension(250.0), LodLevel::Overview); // 25px -> Overview

        // Zoom in (scale 4.0x)
        viewport.set_zoom(4.0);
        assert_eq!(viewport.lod_for_dimension(60.0), LodLevel::Full); // 240px -> Full
    }

    #[test]
    fn test_viewport_visibility() {
        let viewport = Viewport::new(1000.0, 800.0);

        assert!(viewport.is_visible(100.0, 200.0));
        assert!(viewport.is_visible(-50.0, 100.0)); // partially left
        assert!(!viewport.is_visible(1100.0, 50.0)); // completely right of screen
    }
}
