// Summoner DAW - Customizable Floating HUD Overlay for Real-Time Telemetry (Step 1304)

use serde::{Deserialize, Serialize};

/// Display mode layout density for HUD performance overlay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HudDensityMode {
    Compact,
    Standard,
    Detailed,
}

/// Floating HUD Overlay telemetry state and layout configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HudOverlayView {
    pub enabled: bool,
    pub density: HudDensityMode,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub cpu_load_pct: f32,
    pub dsp_buffer_latency_ms: f32,
    pub dsp_buffer_size: usize,
    pub sample_rate: u32,
    pub memory_usage_mb: f32,
    pub fps: f32,
    pub opacity: f32,
}

impl Default for HudOverlayView {
    fn default() -> Self {
        Self {
            enabled: true,
            density: HudDensityMode::Standard,
            x: 20.0,
            y: 50.0,
            width: 280.0,
            height: 160.0,
            cpu_load_pct: 18.5,
            dsp_buffer_latency_ms: 1.33,
            dsp_buffer_size: 64,
            sample_rate: 48000,
            memory_usage_mb: 412.0,
            fps: 60.0,
            opacity: 0.85,
        }
    }
}

impl HudOverlayView {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update performance and telemetry statistics.
    pub fn update_telemetry(
        &mut self,
        cpu_load_pct: f32,
        buffer_size: usize,
        sample_rate: u32,
        memory_mb: f32,
        fps: f32,
    ) {
        self.cpu_load_pct = cpu_load_pct.clamp(0.0, 100.0);
        self.dsp_buffer_size = buffer_size;
        self.sample_rate = sample_rate;
        if sample_rate > 0 {
            self.dsp_buffer_latency_ms = (buffer_size as f32 / sample_rate as f32) * 1000.0;
        }
        self.memory_usage_mb = memory_mb.max(0.0);
        self.fps = fps.clamp(0.0, 240.0);
    }

    /// Calculate bounds rect for isolated spatial math collision checking.
    pub fn bounds_rect(&self) -> crate::layout_math::Rect {
        crate::layout_math::Rect::new(self.x, self.y, self.width, self.height)
    }

    /// Clamp overlay position inside parent viewport.
    pub fn clamp_to_viewport(&mut self, viewport: crate::layout_math::Rect) {
        self.x = self.x.clamp(
            viewport.min_x(),
            (viewport.max_x() - self.width).max(viewport.min_x()),
        );
        self.y = self.y.clamp(
            viewport.min_y(),
            (viewport.max_y() - self.height).max(viewport.min_y()),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout_math::Rect;

    #[test]
    fn test_hud_overlay_telemetry_and_viewport_clamping() {
        let mut hud = HudOverlayView::new();
        assert_eq!(hud.cpu_load_pct, 18.5);

        hud.update_telemetry(45.2, 128, 48000, 512.0, 120.0);
        assert!((hud.dsp_buffer_latency_ms - 2.666).abs() < 0.01);
        assert_eq!(hud.cpu_load_pct, 45.2);

        let viewport = Rect::new(0.0, 0.0, 1920.0, 1080.0);
        hud.x = 2500.0;
        hud.y = -100.0;
        hud.clamp_to_viewport(viewport);

        assert_eq!(hud.x, 1920.0 - 280.0);
        assert_eq!(hud.y, 0.0);
    }
}
