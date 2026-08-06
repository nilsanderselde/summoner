// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! High-DPI real-time multi-track oscilloscope & phase correlation visualizer (`OscilloscopeView`).
//! Multi-channel audio buffer waveform renderer (stereo / L+R / phase XY mode), configurable zoom level,
//! grid overlays, phase correlation gauge (-1.0 to +1.0). Pure state and layout calculation methods.

use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
use eframe::egui;

/// Oscilloscope visualization rendering modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum OscilloscopeMode {
    /// Stereo view showing Left and Right channels separately.
    #[default]
    Stereo,
    /// Mono sum view showing combined (L + R) / 2 waveform.
    MonoSum,
    /// Phase XY Lissajous plot rotated by 45 degrees.
    PhaseXY,
}

/// Standalone configuration for `OscilloscopeView`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscilloscopeConfig {
    pub mode: OscilloscopeMode,
    pub zoom_level: f32,
    pub gain: f32,
    pub grid_enabled: bool,
    pub grid_divisions: usize,
    pub dpi_scale: f32,
    pub show_phase_gauge: bool,
}

impl Default for OscilloscopeConfig {
    fn default() -> Self {
        Self {
            mode: OscilloscopeMode::Stereo,
            zoom_level: 1.0,
            gain: 1.0,
            grid_enabled: true,
            grid_divisions: 8,
            dpi_scale: 1.0,
            show_phase_gauge: true,
        }
    }
}

impl OscilloscopeConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_mode(mut self, mode: OscilloscopeMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_zoom(mut self, zoom: f32) -> Self {
        self.zoom_level = zoom.clamp(0.1, 10.0);
        self
    }

    pub fn with_gain(mut self, gain: f32) -> Self {
        self.gain = gain.clamp(0.1, 10.0);
        self
    }
}

/// Calculated UI bounding regions for pure layout layout evaluation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OscilloscopeLayout {
    pub total_bounds: (f32, f32, f32, f32), // (x, y, width, height)
    pub waveform_bounds: (f32, f32, f32, f32), // Waveform rendering area
    pub gauge_bounds: (f32, f32, f32, f32), // Phase correlation meter area
}

impl OscilloscopeLayout {
    /// Computes layout component rectangles from total container bounds.
    pub fn calculate(bounds: (f32, f32, f32, f32), show_gauge: bool) -> Self {
        let (x, y, w, h) = bounds;
        let gauge_h = if show_gauge {
            (h * 0.15).clamp(24.0, 40.0)
        } else {
            0.0
        };
        let wave_h = (h - gauge_h).max(10.0);

        Self {
            total_bounds: bounds,
            waveform_bounds: (x, y, w, wave_h),
            gauge_bounds: (x, y + wave_h, w, gauge_h),
        }
    }
}

/// Multi-track audio channel buffer container for oscilloscope input.
#[derive(Debug, Clone, Default)]
pub struct TrackAudioBuffer {
    pub track_id: u64,
    pub name: String,
    pub left: Vec<f32>,
    pub right: Vec<f32>,
    pub color_rgb: (u8, u8, u8),
}

impl TrackAudioBuffer {
    pub fn new(track_id: u64, name: impl Into<String>, left: Vec<f32>, right: Vec<f32>) -> Self {
        Self {
            track_id,
            name: name.into(),
            left,
            right,
            color_rgb: (0, 229, 255),
        }
    }
}

/// Type alias for stereo waveform point pairs.
pub type WaveformPointPairs = (Vec<(f32, f32)>, Vec<(f32, f32)>);

/// Pure audio analysis and point calculation utilities.
pub struct OscilloscopeMath;

impl OscilloscopeMath {
    /// Computes phase correlation coefficient between Left and Right audio buffers in range [-1.0, +1.0].
    /// Formula: r = sum(L * R) / sqrt(sum(L^2) * sum(R^2))
    pub fn calculate_phase_correlation(left: &[f32], right: &[f32]) -> f32 {
        let len = left.len().min(right.len());
        if len == 0 {
            return 1.0;
        }

        let mut sum_lr = 0.0f64;
        let mut sum_l2 = 0.0f64;
        let mut sum_r2 = 0.0f64;

        for i in 0..len {
            let l = left[i] as f64;
            let r = right[i] as f64;
            sum_lr += l * r;
            sum_l2 += l * l;
            sum_r2 += r * r;
        }

        let denom = (sum_l2 * sum_r2).sqrt();
        if denom < 1e-9 {
            1.0
        } else {
            (sum_lr / denom).clamp(-1.0, 1.0) as f32
        }
    }

    /// Computes layout coordinates for mono-sum waveform line rendering.
    pub fn calculate_mono_waveform_points(
        left: &[f32],
        right: &[f32],
        bounds: (f32, f32, f32, f32),
        zoom: f32,
        gain: f32,
    ) -> Vec<(f32, f32)> {
        let (rx, ry, rw, rh) = bounds;
        let len = left.len().min(right.len());
        if len < 2 || rw <= 0.0 || rh <= 0.0 {
            return Vec::new();
        }

        let mid_y = ry + rh * 0.5;
        let half_h = rh * 0.45;
        let window_size = ((len as f32) / zoom.max(0.1)).max(2.0) as usize;
        let sample_count = len.min(window_size);

        let mut points = Vec::with_capacity(sample_count);
        for i in 0..sample_count {
            let t = i as f32 / (sample_count - 1) as f32;
            let x = rx + t * rw;
            let mono_sample = ((left[i] + right[i]) * 0.5 * gain).clamp(-1.0, 1.0);
            let y = mid_y - mono_sample * half_h;
            points.push((x, y));
        }
        points
    }

    /// Computes layout coordinates for stereo (separate Left and Right) waveform line rendering.
    pub fn calculate_stereo_waveform_points(
        left: &[f32],
        right: &[f32],
        bounds: (f32, f32, f32, f32),
        zoom: f32,
        gain: f32,
    ) -> WaveformPointPairs {
        let (rx, ry, rw, rh) = bounds;
        let len = left.len().min(right.len());
        if len < 2 || rw <= 0.0 || rh <= 0.0 {
            return (Vec::new(), Vec::new());
        }

        let half_h = rh * 0.5;
        let l_mid_y = ry + half_h * 0.5;
        let r_mid_y = ry + half_h + half_h * 0.5;
        let max_amp = half_h * 0.45;

        let window_size = ((len as f32) / zoom.max(0.1)).max(2.0) as usize;
        let sample_count = len.min(window_size);

        let mut left_points = Vec::with_capacity(sample_count);
        let mut right_points = Vec::with_capacity(sample_count);

        for i in 0..sample_count {
            let t = i as f32 / (sample_count - 1) as f32;
            let x = rx + t * rw;

            let l_samp = (left[i] * gain).clamp(-1.0, 1.0);
            let r_samp = (right[i] * gain).clamp(-1.0, 1.0);

            left_points.push((x, l_mid_y - l_samp * max_amp));
            right_points.push((x, r_mid_y - r_samp * max_amp));
        }

        (left_points, right_points)
    }

    /// Computes layout coordinates for Phase XY Lissajous plot (rotated 45 deg).
    pub fn calculate_phase_xy_points(
        left: &[f32],
        right: &[f32],
        bounds: (f32, f32, f32, f32),
        gain: f32,
    ) -> Vec<(f32, f32)> {
        let (rx, ry, rw, rh) = bounds;
        let len = left.len().min(right.len());
        if len == 0 || rw <= 0.0 || rh <= 0.0 {
            return Vec::new();
        }

        let center_x = rx + rw * 0.5;
        let center_y = ry + rh * 0.5;
        let radius = (rw.min(rh) * 0.45).max(10.0);
        let inv_sqrt2 = std::f32::consts::FRAC_1_SQRT_2;

        let mut points = Vec::with_capacity(len);
        for i in 0..len {
            let l = (left[i] * gain).clamp(-1.0, 1.0);
            let r = (right[i] * gain).clamp(-1.0, 1.0);

            let rot_x = (l - r) * inv_sqrt2;
            let rot_y = (l + r) * inv_sqrt2;

            let x = center_x + rot_x * radius;
            let y = center_y - rot_y * radius;
            points.push((x, y));
        }
        points
    }

    /// Calculates grid overlay line segment end-points for UI rendering.
    pub fn calculate_grid_lines(
        bounds: (f32, f32, f32, f32),
        divisions: usize,
    ) -> Vec<((f32, f32), (f32, f32))> {
        let (rx, ry, rw, rh) = bounds;
        if divisions == 0 || rw <= 0.0 || rh <= 0.0 {
            return Vec::new();
        }

        let mut lines = Vec::with_capacity(divisions * 2);

        // Vertical division grid lines
        for i in 1..divisions {
            let x = rx + (i as f32 / divisions as f32) * rw;
            lines.push(((x, ry), (x, ry + rh)));
        }

        // Horizontal division grid lines
        for i in 1..divisions {
            let y = ry + (i as f32 / divisions as f32) * rh;
            lines.push(((rx, y), (rx + rw, y)));
        }

        lines
    }
}

/// High-DPI real-time multi-track oscilloscope visualizer component.
pub struct OscilloscopeView {
    pub config: OscilloscopeConfig,
    pub main_left: Vec<f32>,
    pub main_right: Vec<f32>,
    pub tracks: Vec<TrackAudioBuffer>,
}

impl Default for OscilloscopeView {
    fn default() -> Self {
        Self::new()
    }
}

impl OscilloscopeView {
    pub fn new() -> Self {
        Self {
            config: OscilloscopeConfig::default(),
            main_left: Vec::new(),
            main_right: Vec::new(),
            tracks: Vec::new(),
        }
    }

    pub fn with_config(mut self, config: OscilloscopeConfig) -> Self {
        self.config = config;
        self
    }

    pub fn update_main_audio(&mut self, left: &[f32], right: &[f32]) {
        self.main_left = left.to_vec();
        self.main_right = right.to_vec();
    }

    pub fn add_track_buffer(&mut self, track: TrackAudioBuffer) {
        self.tracks.push(track);
    }

    pub fn phase_correlation(&self) -> f32 {
        OscilloscopeMath::calculate_phase_correlation(&self.main_left, &self.main_right)
    }

    /// GUI rendering function powered by egui.
    #[cfg(feature = "gui")]
    pub fn show(&mut self, ui: &mut egui::Ui, width: f32, height: f32) -> egui::Response {
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter_at(rect);
            let bounds = (rect.min.x, rect.min.y, rect.width(), rect.height());
            let layout = OscilloscopeLayout::calculate(bounds, self.config.show_phase_gauge);

            // Scope Background
            painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(10, 13, 20));
            painter.rect_stroke(
                rect,
                4.0,
                egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(30, 45, 70)),
            );

            let (wx, wy, ww, wh) = layout.waveform_bounds;

            // Grid Overlay Lines
            if self.config.grid_enabled {
                let grid_color = egui::Color32::from_rgb(25, 38, 55);
                let lines = OscilloscopeMath::calculate_grid_lines(
                    (wx, wy, ww, wh),
                    self.config.grid_divisions,
                );
                for ((x1, y1), (x2, y2)) in lines {
                    painter.line_segment(
                        [egui::pos2(x1, y1), egui::pos2(x2, y2)],
                        egui::Stroke::new(1.0_f32, grid_color),
                    );
                }
            }

            // Waveform / Phase Plot Rendering based on Mode
            match self.config.mode {
                OscilloscopeMode::MonoSum => {
                    let pts = OscilloscopeMath::calculate_mono_waveform_points(
                        &self.main_left,
                        &self.main_right,
                        (wx, wy, ww, wh),
                        self.config.zoom_level,
                        self.config.gain,
                    );
                    if pts.len() >= 2 {
                        let shape_pts: Vec<egui::Pos2> =
                            pts.into_iter().map(|(x, y)| egui::pos2(x, y)).collect();
                        painter.add(egui::Shape::line(
                            shape_pts,
                            egui::Stroke::new(1.8_f32, egui::Color32::from_rgb(0, 229, 255)),
                        ));
                    }
                }
                OscilloscopeMode::Stereo => {
                    let (l_pts, r_pts) = OscilloscopeMath::calculate_stereo_waveform_points(
                        &self.main_left,
                        &self.main_right,
                        (wx, wy, ww, wh),
                        self.config.zoom_level,
                        self.config.gain,
                    );
                    if l_pts.len() >= 2 {
                        let l_shape: Vec<egui::Pos2> =
                            l_pts.into_iter().map(|(x, y)| egui::pos2(x, y)).collect();
                        painter.add(egui::Shape::line(
                            l_shape,
                            egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(0, 229, 255)),
                        ));
                    }
                    if r_pts.len() >= 2 {
                        let r_shape: Vec<egui::Pos2> =
                            r_pts.into_iter().map(|(x, y)| egui::pos2(x, y)).collect();
                        painter.add(egui::Shape::line(
                            r_shape,
                            egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(255, 0, 127)),
                        ));
                    }
                }
                OscilloscopeMode::PhaseXY => {
                    let xy_pts = OscilloscopeMath::calculate_phase_xy_points(
                        &self.main_left,
                        &self.main_right,
                        (wx, wy, ww, wh),
                        self.config.gain,
                    );
                    for (x, y) in xy_pts {
                        painter.circle_filled(
                            egui::pos2(x, y),
                            1.2,
                            egui::Color32::from_rgb(0, 240, 180),
                        );
                    }
                }
            }

            // Phase Correlation Gauge Bar
            if self.config.show_phase_gauge {
                let (gx, gy, gw, gh) = layout.gauge_bounds;
                let gauge_rect =
                    egui::Rect::from_min_max(egui::pos2(gx, gy), egui::pos2(gx + gw, gy + gh));
                painter.rect_filled(gauge_rect, 2.0, egui::Color32::from_rgb(15, 18, 26));

                let correlation = self.phase_correlation();
                // Map correlation [-1.0, +1.0] -> normalized [0.0, 1.0]
                let norm_corr = (correlation + 1.0) * 0.5;
                let indicator_x = gx + norm_corr * gw;

                let gauge_color = if correlation > 0.5 {
                    egui::Color32::from_rgb(0, 230, 140)
                } else if correlation >= 0.0 {
                    egui::Color32::from_rgb(255, 200, 0)
                } else {
                    egui::Color32::from_rgb(255, 60, 60)
                };

                // Center zero line
                let center_x = gx + gw * 0.5;
                painter.line_segment(
                    [egui::pos2(center_x, gy), egui::pos2(center_x, gy + gh)],
                    egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(80, 90, 110)),
                );

                // Indicator needle
                painter.line_segment(
                    [
                        egui::pos2(indicator_x, gy),
                        egui::pos2(indicator_x, gy + gh),
                    ],
                    egui::Stroke::new(2.5_f32, gauge_color),
                );

                let label = format!("Phase Correlation: {:+.2}", correlation);
                painter.text(
                    egui::pos2(gx + 6.0, gy + gh * 0.5),
                    egui::Align2::LEFT_CENTER,
                    label,
                    egui::FontId::proportional(11.0),
                    egui::Color32::from_rgb(200, 210, 225),
                );
            }
        }

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_correlation_identical_mono() {
        let left = vec![0.5, -0.5, 0.8, -0.8];
        let right = vec![0.5, -0.5, 0.8, -0.8];
        let corr = OscilloscopeMath::calculate_phase_correlation(&left, &right);
        assert!(
            (corr - 1.0).abs() < 1e-5,
            "Identical mono signals must have +1.0 correlation, got {}",
            corr
        );
    }

    #[test]
    fn test_phase_correlation_inverted_phase() {
        let left = vec![0.5, -0.5, 0.8, -0.8];
        let right = vec![-0.5, 0.5, -0.8, 0.8];
        let corr = OscilloscopeMath::calculate_phase_correlation(&left, &right);
        assert!(
            (corr - (-1.0)).abs() < 1e-5,
            "Inverted signals must have -1.0 correlation, got {}",
            corr
        );
    }

    #[test]
    fn test_phase_correlation_uncorrelated() {
        let left = vec![1.0, -1.0, 1.0, -1.0];
        let right = vec![1.0, 1.0, -1.0, -1.0];
        let corr = OscilloscopeMath::calculate_phase_correlation(&left, &right);
        assert!(
            (corr - 0.0).abs() < 1e-5,
            "Orthogonal signals must have ~0.0 correlation, got {}",
            corr
        );
    }

    #[test]
    fn test_oscilloscope_config_zoom_and_gain_clamping() {
        let config = OscilloscopeConfig::new()
            .with_zoom(0.001) // Should clamp to 0.1
            .with_gain(50.0); // Should clamp to 10.0

        assert_eq!(config.zoom_level, 0.1);
        assert_eq!(config.gain, 10.0);
    }

    #[test]
    fn test_layout_calculation_bounds() {
        let bounds = (10.0, 20.0, 400.0, 300.0);
        let layout = OscilloscopeLayout::calculate(bounds, true);

        assert_eq!(layout.total_bounds, bounds);
        assert_eq!(layout.waveform_bounds.0, 10.0);
        assert_eq!(layout.waveform_bounds.1, 20.0);
        assert_eq!(layout.waveform_bounds.2, 400.0);
        assert!(layout.waveform_bounds.3 < 300.0);
        assert_eq!(
            layout.gauge_bounds.1,
            layout.waveform_bounds.1 + layout.waveform_bounds.3
        );
    }

    #[test]
    fn test_waveform_points_generation() {
        let left = vec![0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0, -0.5];
        let right = vec![0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0, -0.5];
        let bounds = (0.0, 0.0, 200.0, 100.0);

        let mono_pts =
            OscilloscopeMath::calculate_mono_waveform_points(&left, &right, bounds, 1.0, 1.0);
        assert_eq!(mono_pts.len(), left.len());
        assert_eq!(mono_pts[0].0, 0.0);
        assert_eq!(mono_pts[left.len() - 1].0, 200.0);

        let (l_pts, r_pts) =
            OscilloscopeMath::calculate_stereo_waveform_points(&left, &right, bounds, 1.0, 1.0);
        assert_eq!(l_pts.len(), left.len());
        assert_eq!(r_pts.len(), right.len());
    }

    #[test]
    fn test_grid_lines_generation() {
        let bounds = (0.0, 0.0, 100.0, 100.0);
        let grid = OscilloscopeMath::calculate_grid_lines(bounds, 4);
        // 3 vertical + 3 horizontal = 6 grid lines
        assert_eq!(grid.len(), 6);
    }
}
