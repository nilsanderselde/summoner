// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Ultra-Low Latency Waveform Sample Editor View (Step 1302, Step 1307).

use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
use eframe::egui;

/// Visual drag handle width for transient markers in points.
pub const MARKER_HANDLE_WIDTH: f32 = 12.0;

/// Minimum touch & pointer hit target width for transient markers in points.
pub const MIN_HIT_TARGET_WIDTH: f32 = 44.0;

/// Default snap threshold in screen points (pixels).
pub const DEFAULT_SNAP_THRESHOLD_PX: f32 = 10.0;

/// Min and max zoom level boundaries.
pub const MIN_ZOOM_LEVEL: f32 = 0.1;
pub const MAX_ZOOM_LEVEL: f32 = 1000.0;

/// Spatial layout calculation structure for `SampleEditorView`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SampleEditorLayout {
    /// Total viewport bounding box (x, y, width, height).
    pub total_bounds: (f32, f32, f32, f32),
    /// Header timeline bounding box (height: 30.0pt).
    pub header_bounds: (f32, f32, f32, f32),
    /// Waveform viewport bounding box (y: 30..height - 40pt).
    pub waveform_bounds: (f32, f32, f32, f32),
    /// Footer controls bounding box (height: 40.0pt).
    pub footer_bounds: (f32, f32, f32, f32),
}

impl SampleEditorLayout {
    pub const HEADER_HEIGHT: f32 = 30.0;
    pub const FOOTER_HEIGHT: f32 = 40.0;

    /// Computes layout component rectangles from total container bounds.
    pub fn calculate(bounds: (f32, f32, f32, f32)) -> Self {
        let (x, y, w, h) = bounds;
        let wave_y = y + Self::HEADER_HEIGHT;
        let wave_h = (h - Self::HEADER_HEIGHT - Self::FOOTER_HEIGHT).max(10.0);
        let footer_y = y + h - Self::FOOTER_HEIGHT;

        Self {
            total_bounds: bounds,
            header_bounds: (x, y, w, Self::HEADER_HEIGHT),
            waveform_bounds: (x, wave_y, w, wave_h),
            footer_bounds: (x, footer_y, w, Self::FOOTER_HEIGHT),
        }
    }
}

/// A transient marker placed within the waveform timeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransientMarker {
    pub id: usize,
    pub name: String,
    pub sample_index: usize,
    pub time_seconds: f64,
}

impl TransientMarker {
    pub fn new(id: usize, name: impl Into<String>, sample_index: usize, sample_rate: f64) -> Self {
        let name = name.into();
        let time_seconds = if sample_rate > 0.0 {
            sample_index as f64 / sample_rate
        } else {
            0.0
        };
        Self {
            id,
            name,
            sample_index,
            time_seconds,
        }
    }

    /// Updates the sample position and recomputes exact time in seconds.
    pub fn update_sample_index(&mut self, new_index: usize, sample_rate: f64) {
        self.sample_index = new_index;
        self.time_seconds = if sample_rate > 0.0 {
            new_index as f64 / sample_rate
        } else {
            0.0
        };
    }

    /// Formats tag label (e.g. "TR1: 0.12s").
    pub fn label(&self) -> String {
        format!("{}: {:.2}s", self.name, self.time_seconds)
    }

    /// Computes hit target horizontal range [min_x, max_x] centered on marker line.
    /// Clamped to >= 44.0pt hit width.
    pub fn hit_target_bounds(marker_x: f32) -> (f32, f32) {
        let half_width = MIN_HIT_TARGET_WIDTH / 2.0; // 22.0pt
        (marker_x - half_width, marker_x + half_width)
    }

    /// Checks if a pointer click horizontal position `click_x` falls within hit target bounds.
    pub fn is_hit(&self, marker_x: f32, click_x: f32) -> bool {
        let (min_x, max_x) = Self::hit_target_bounds(marker_x);
        click_x >= min_x && click_x <= max_x
    }
}

/// Ultra-low latency waveform sample editor view state and logic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleEditorView {
    pub zoom_level: f32,
    pub base_scale: f32,
    pub scroll_sample: usize,
    pub snap_enabled: bool,
    pub snap_threshold_px: f32,
    pub sample_rate: f64,
    pub grid_division_samples: Option<usize>,
    pub transient_markers: Vec<TransientMarker>,
    pub active_drag_marker: Option<usize>,
    pub waveform_samples: Vec<f32>,
    pub selected_version: String,
    pub viewport_width: f32,
    pub viewport_height: f32,
}

impl Default for SampleEditorView {
    fn default() -> Self {
        let sample_rate = 44100.0;
        let default_samples = Self::generate_demo_waveform(sample_rate, 4.25);
        let markers = vec![
            TransientMarker::new(1, "TR1", (0.12 * sample_rate) as usize, sample_rate),
            TransientMarker::new(2, "TR2", (1.05 * sample_rate) as usize, sample_rate),
            TransientMarker::new(3, "TR3", (2.50 * sample_rate) as usize, sample_rate),
        ];

        Self {
            zoom_level: 100.0,
            base_scale: 0.01,
            scroll_sample: 0,
            snap_enabled: true,
            snap_threshold_px: DEFAULT_SNAP_THRESHOLD_PX,
            sample_rate,
            grid_division_samples: Some(4410), // 0.1s grid step at 44.1kHz
            transient_markers: markers,
            active_drag_marker: None,
            waveform_samples: default_samples,
            selected_version: "V1".to_string(),
            viewport_width: 800.0,
            viewport_height: 300.0,
        }
    }
}

impl SampleEditorView {
    pub fn new() -> Self {
        Self::default()
    }

    /// Generates a synthetic audio waveform for editor preview.
    pub fn generate_demo_waveform(sample_rate: f64, duration_secs: f64) -> Vec<f32> {
        let total_samples = (sample_rate * duration_secs) as usize;
        let mut samples = Vec::with_capacity(total_samples);
        for i in 0..total_samples {
            let t = i as f32 / sample_rate as f32;
            let val = (2.0 * std::f32::consts::PI * 440.0 * t).sin() * (-1.5 * t).exp();
            samples.push(val.clamp(-1.0, 1.0));
        }
        samples
    }

    /// Calculates pixels per sample: `pixels_per_sample = base_scale * zoom_level` (clamped to [0.1, 1000.0]).
    pub fn pixels_per_sample(&self) -> f32 {
        let clamped_zoom = self.zoom_level.clamp(MIN_ZOOM_LEVEL, MAX_ZOOM_LEVEL);
        self.base_scale * clamped_zoom
    }

    /// Sets zoom level with automatic clamping to range [0.1, 1000.0].
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom_level = zoom.clamp(MIN_ZOOM_LEVEL, MAX_ZOOM_LEVEL);
    }

    /// Applies multi-touch pinch-zoom factor.
    pub fn apply_pinch_zoom(&mut self, pinch_factor: f32) {
        self.set_zoom(self.zoom_level * pinch_factor);
    }

    /// Snap target sample to nearest transient marker within sample threshold.
    pub fn snap_transient_marker(&self, target_sample: usize, threshold: usize) -> usize {
        for marker in &self.transient_markers {
            let diff = (marker.sample_index as isize - target_sample as isize).abs();
            if diff <= threshold as isize {
                return marker.sample_index;
            }
        }
        target_sample
    }

    /// Calculate transient drag handle bounding box ensuring min hit target >= 44.0pt.
    pub fn transient_handle_bounds(
        &self,
        x_px: f32,
        viewport_y: f32,
        viewport_h: f32,
    ) -> crate::layout_math::Rect {
        let rect = crate::layout_math::Rect::new(x_px - 6.0, viewport_y, 12.0, viewport_h);
        rect.enforce_min_hit_target(MIN_HIT_TARGET_WIDTH)
    }

    /// Computes layout bounding boxes for a container of width x height.
    pub fn layout(&self, width: f32, height: f32) -> SampleEditorLayout {
        SampleEditorLayout::calculate((0.0, 0.0, width, height))
    }

    /// Converts sample index to horizontal pixel offset within waveform viewport.
    pub fn sample_to_pixel(&self, sample_idx: usize) -> f32 {
        let pps = self.pixels_per_sample();
        (sample_idx as f32 - self.scroll_sample as f32) * pps
    }

    /// Converts horizontal pixel offset within waveform viewport to sample index.
    pub fn pixel_to_sample(&self, pixel_x: f32) -> usize {
        let pps = self.pixels_per_sample();
        if pps <= 0.0 {
            return self.scroll_sample;
        }
        let rel_samples = (pixel_x / pps).max(0.0);
        self.scroll_sample + rel_samples.round() as usize
    }

    /// Finds snapped sample index for a target sample position.
    /// Snaps to nearest sample zero-crossing or grid division within `snap_threshold_px` (10.0pt).
    pub fn snap_sample(&self, target_sample: usize) -> usize {
        if !self.snap_enabled || self.waveform_samples.is_empty() {
            return target_sample;
        }

        let pps = self.pixels_per_sample();
        if pps <= 0.0 {
            return target_sample;
        }

        let threshold_px = self.snap_threshold_px;
        let max_sample_dist = (threshold_px / pps).ceil() as usize;

        let search_start = target_sample.saturating_sub(max_sample_dist);
        let search_end = (target_sample + max_sample_dist).min(self.waveform_samples.len());

        let mut best_snapped = target_sample;
        let mut min_px_dist = threshold_px + 0.001; // Start strictly greater than threshold

        // 1. Check nearest sample zero-crossing
        if search_end > search_start && self.waveform_samples.len() > 1 {
            let limit = search_end.min(self.waveform_samples.len() - 1);
            for i in search_start..limit {
                let s1 = self.waveform_samples[i];
                let s2 = self.waveform_samples[i + 1];
                if s1 == 0.0 || (s1 > 0.0 && s2 <= 0.0) || (s1 < 0.0 && s2 >= 0.0) {
                    let z_idx = if s1.abs() <= s2.abs() { i } else { i + 1 };
                    let sample_diff = (z_idx as f64 - target_sample as f64).abs();
                    let px_diff = (sample_diff as f32) * pps;
                    if px_diff <= threshold_px && px_diff < min_px_dist {
                        min_px_dist = px_diff;
                        best_snapped = z_idx;
                    }
                }
            }
        }

        // 2. Check nearest grid division
        if let Some(grid_step) = self.grid_division_samples {
            if grid_step > 0 {
                let grid_idx =
                    ((target_sample as f64) / (grid_step as f64)).round() as usize * grid_step;
                let sample_diff = (grid_idx as f64 - target_sample as f64).abs();
                let px_diff = (sample_diff as f32) * pps;
                if px_diff <= threshold_px && px_diff < min_px_dist {
                    best_snapped = grid_idx;
                }
            }
        }

        best_snapped
    }

    /// Renders ASCII mental wireframe representation of sample editor view.
    pub fn render_ascii(&self) -> String {
        let mut out = String::new();
        out.push_str("+-----------------------------------------------------------+\n");
        out.push_str(&format!(
            "| ULTRA-LOW LATENCY SAMPLE EDITOR ({:.0}x{:.0}pt)              |\n",
            self.viewport_width, self.viewport_height
        ));
        out.push_str("| +-------------------------------------------------------+ |\n");

        let first_marker_str = self
            .transient_markers
            .first()
            .map(|m| m.label())
            .unwrap_or_else(|| "TR1: 0.12s".into());

        out.push_str(&format!(
            "| | [{}] |~~~~/^\\~~~~~/^\\~~~~~/^\\~~~~~~~~| [{:<10}] | |\n",
            self.selected_version, first_marker_str
        ));
        out.push_str("| |      |___/   \\___/   \\___/   \\_______|                | |\n");
        out.push_str("| |             ^ Transient handle #1                     | |\n");
        out.push_str("| +-------------------------------------------------------+ |\n");

        let end_time = if self.sample_rate > 0.0 {
            self.waveform_samples.len() as f64 / self.sample_rate
        } else {
            0.0
        };

        let start_time_str = Self::format_timecode(self.scroll_sample as f64 / self.sample_rate);
        let end_time_str = Self::format_timecode(end_time);

        out.push_str(&format!(
            "| Zoom: {:.1}x | Start: {} | End: {} | Snap: {} |\n",
            self.zoom_level,
            start_time_str,
            end_time_str,
            if self.snap_enabled { "ON" } else { "OFF" }
        ));
        out.push_str("+-----------------------------------------------------------+\n");
        out
    }

    /// Formats seconds into mm:ss.mmm timecode string (e.g. 00:04.250).
    pub fn format_timecode(seconds: f64) -> String {
        let total_ms = (seconds * 1000.0).max(0.0) as u64;
        let mins = total_ms / 60000;
        let secs = (total_ms % 60000) / 1000;
        let ms = total_ms % 1000;
        format!("{:02}:{:02}.{:03}", mins, secs, ms)
    }
}

#[cfg(feature = "gui")]
impl SampleEditorView {
    /// Renders the egui widget for `SampleEditorView`.
    pub fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(self.viewport_width, self.viewport_height),
            egui::Sense::click_and_drag(),
        );

        let layout =
            SampleEditorLayout::calculate((rect.min.x, rect.min.y, rect.width(), rect.height()));

        // Pinch zoom handling from egui input
        ui.input(|i| {
            let zoom_delta = i.zoom_delta();
            if (zoom_delta - 1.0).abs() > 0.001 {
                self.apply_pinch_zoom(zoom_delta);
            }
        });

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Background
            painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(20, 22, 28));

            // Header timeline (height 30.0pt)
            let header_rect = egui::Rect::from_min_size(
                egui::pos2(layout.header_bounds.0, layout.header_bounds.1),
                egui::vec2(layout.header_bounds.2, layout.header_bounds.3),
            );
            painter.rect_filled(header_rect, 0.0, egui::Color32::from_rgb(32, 36, 46));
            painter.text(
                header_rect.min + egui::vec2(10.0, 8.0),
                egui::Align2::LEFT_TOP,
                format!(
                    "ULTRA-LOW LATENCY SAMPLE EDITOR [{}]",
                    self.selected_version
                ),
                egui::FontId::proportional(13.0),
                egui::Color32::from_rgb(220, 225, 240),
            );

            // Waveform viewport rect (x: 0..width, y: 30..height - 40pt)
            let wave_rect = egui::Rect::from_min_size(
                egui::pos2(layout.waveform_bounds.0, layout.waveform_bounds.1),
                egui::vec2(layout.waveform_bounds.2, layout.waveform_bounds.3),
            );
            painter.rect_filled(wave_rect, 0.0, egui::Color32::from_rgb(14, 16, 20));

            // Draw center zero line
            let mid_y = wave_rect.center().y;
            painter.line_segment(
                [
                    egui::pos2(wave_rect.min.x, mid_y),
                    egui::pos2(wave_rect.max.x, mid_y),
                ],
                egui::Stroke::new(1.0f32, egui::Color32::from_rgb(50, 55, 65)),
            );

            // Draw waveform audio representation
            let wave_width = wave_rect.width();
            let half_h = wave_rect.height() / 2.0;

            if !self.waveform_samples.is_empty() {
                let step_px = 2.0;
                let num_points = (wave_width / step_px) as usize;
                for p in 0..num_points {
                    let px = wave_rect.min.x + (p as f32 * step_px);
                    let sample_idx = self.pixel_to_sample(p as f32 * step_px);
                    if sample_idx < self.waveform_samples.len() {
                        let val = self.waveform_samples[sample_idx];
                        let y_pos = mid_y - (val * half_h * 0.8);
                        painter.line_segment(
                            [egui::pos2(px, mid_y), egui::pos2(px, y_pos)],
                            egui::Stroke::new(1.5f32, egui::Color32::from_rgb(0, 200, 255)),
                        );
                    }
                }
            }

            // Draw transient markers and drag handles
            for marker in &self.transient_markers {
                let m_px = self.sample_to_pixel(marker.sample_index);
                let line_x = wave_rect.min.x + m_px;
                if line_x >= wave_rect.min.x && line_x <= wave_rect.max.x {
                    // Vertical marker line
                    painter.line_segment(
                        [
                            egui::pos2(line_x, wave_rect.min.y),
                            egui::pos2(line_x, wave_rect.max.y),
                        ],
                        egui::Stroke::new(1.5f32, egui::Color32::from_rgb(255, 180, 0)),
                    );

                    // Marker drag handle (width 12pt)
                    let handle_rect = egui::Rect::from_center_size(
                        egui::pos2(line_x, wave_rect.min.y + 12.0),
                        egui::vec2(MARKER_HANDLE_WIDTH, 16.0),
                    );
                    painter.rect_filled(handle_rect, 2.0, egui::Color32::from_rgb(255, 140, 0));

                    // Tag label
                    painter.text(
                        egui::pos2(line_x + 8.0, wave_rect.min.y + 4.0),
                        egui::Align2::LEFT_TOP,
                        marker.label(),
                        egui::FontId::proportional(11.0),
                        egui::Color32::from_rgb(255, 220, 150),
                    );
                }
            }

            // Footer bar (height 40.0pt)
            let footer_rect = egui::Rect::from_min_size(
                egui::pos2(layout.footer_bounds.0, layout.footer_bounds.1),
                egui::vec2(layout.footer_bounds.2, layout.footer_bounds.3),
            );
            painter.rect_filled(footer_rect, 0.0, egui::Color32::from_rgb(26, 28, 36));

            let total_dur = self.waveform_samples.len() as f64 / self.sample_rate;
            let footer_text = format!(
                "Zoom: {:.1}x | Start: {} | End: {} | Snap: {}",
                self.zoom_level,
                Self::format_timecode(self.scroll_sample as f64 / self.sample_rate),
                Self::format_timecode(total_dur),
                if self.snap_enabled { "ON" } else { "OFF" }
            );
            painter.text(
                footer_rect.min + egui::vec2(12.0, 12.0),
                egui::Align2::LEFT_TOP,
                footer_text,
                egui::FontId::proportional(12.0),
                egui::Color32::from_rgb(180, 190, 210),
            );
        }

        // Pointer drag interaction for transient markers
        if response.drag_started() {
            if let Some(pos) = response.interact_pointer_pos() {
                let rel_x = pos.x - rect.min.x;
                for marker in &self.transient_markers {
                    let m_px = self.sample_to_pixel(marker.sample_index);
                    if marker.is_hit(m_px, rel_x) {
                        self.active_drag_marker = Some(marker.id);
                        break;
                    }
                }
            }
        }

        if response.dragged() {
            if let Some(active_id) = self.active_drag_marker {
                if let Some(pos) = response.interact_pointer_pos() {
                    let rel_x = pos.x - rect.min.x;
                    let target_sample = self.pixel_to_sample(rel_x);
                    let snapped = self.snap_sample(target_sample);

                    if let Some(marker) = self
                        .transient_markers
                        .iter_mut()
                        .find(|m| m.id == active_id)
                    {
                        marker.update_sample_index(snapped, self.sample_rate);
                    }
                }
            }
        }

        if response.drag_stopped() {
            self.active_drag_marker = None;
        }

        response
    }
}
