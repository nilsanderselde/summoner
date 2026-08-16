// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Interactive Audio Waveform Transient Marker Editor with Touch-Draggable Warp Anchors (Step 1361).

use crate::layout_math::Rect;
use crate::touch_controls::{ContrastColorPalette, MIN_HIT_TARGET_PT};

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const WARP_MARKER_VISUAL_RADIUS: f32 = 14.0;
pub const WARP_MARKER_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const TRANSIENT_FLAG_HEIGHT: f32 = 44.0;

/// A single warp anchor point or transient detection marker.
#[derive(Debug, Clone, PartialEq)]
pub struct WarpMarker {
    pub id: String,
    pub original_sample_idx: usize,
    pub warped_sample_idx: usize,
    pub is_pinned: bool,
    pub transient_strength: f32, // 0.0 ..= 1.0
    pub is_selected: bool,
    pub is_dragging: bool,
}

impl WarpMarker {
    pub fn new(
        id: impl Into<String>,
        sample_idx: usize,
        transient_strength: f32,
        is_pinned: bool,
    ) -> Self {
        Self {
            id: id.into(),
            original_sample_idx: sample_idx,
            warped_sample_idx: sample_idx,
            is_pinned,
            transient_strength: transient_strength.clamp(0.0, 1.0),
            is_selected: false,
            is_dragging: false,
        }
    }

    /// Time stretch ratio at this marker relative to original sample position.
    pub fn stretch_ratio(&self) -> f32 {
        if self.original_sample_idx == 0 {
            1.0_f32
        } else {
            self.warped_sample_idx as f32 / self.original_sample_idx as f32
        }
    }

    /// Stretch percentage (+/-%) relative to original timing.
    pub fn stretch_percentage(&self) -> f32 {
        (self.stretch_ratio() - 1.0_f32) * 100.0_f32
    }
}

/// Interactive Audio Waveform Transient Marker Editor View (Step 1361).
#[derive(Debug, Clone)]
pub struct TransientWarpEditorView {
    pub total_samples: usize,
    pub sample_rate: u32,
    pub bpm: f64,
    pub markers: Vec<WarpMarker>,
    pub zoom_level: f32,          // 1.0x to 32.0x
    pub scroll_offset_ratio: f32, // 0.0 ..= 1.0
    pub selected_marker_idx: Option<usize>,
    pub dragging_marker_idx: Option<usize>,
    pub playback_cursor_sample: usize,
    pub grid_snap_subdivision: u32, // e.g. 4 = 1/16th notes (4 per quarter beat)
    pub color_palette: ContrastColorPalette,
}

impl Default for TransientWarpEditorView {
    fn default() -> Self {
        Self::new(44100 * 4, 44100, 120.0)
    }
}

impl TransientWarpEditorView {
    pub fn new(total_samples: usize, sample_rate: u32, bpm: f64) -> Self {
        let mut view = Self {
            total_samples: total_samples.max(1000),
            sample_rate,
            bpm,
            markers: Vec::new(),
            zoom_level: 1.0_f32,
            scroll_offset_ratio: 0.0_f32,
            selected_marker_idx: None,
            dragging_marker_idx: None,
            playback_cursor_sample: 0,
            grid_snap_subdivision: 4,
            color_palette: ContrastColorPalette::default(),
        };

        // Populate sample transients across 4 bars
        let samples_per_beat = (sample_rate as f64 * 60.0 / bpm) as usize;
        for beat in 0..16 {
            let s_idx = (beat * samples_per_beat).min(view.total_samples - 1);
            let is_downbeat = beat % 4 == 0;
            let strength = if is_downbeat { 0.95_f32 } else { 0.65_f32 };
            let is_pinned = is_downbeat;
            view.markers.push(WarpMarker::new(
                format!("marker_{beat}"),
                s_idx,
                strength,
                is_pinned,
            ));
        }

        view
    }

    /// Calculate visible samples range [start_sample, end_sample] based on scroll and zoom.
    pub fn visible_sample_range(&self) -> (usize, usize) {
        let visible_samples = ((self.total_samples as f32) / self.zoom_level.max(1.0_f32)) as usize;
        let visible_samples = visible_samples.clamp(100, self.total_samples);
        let max_start = self.total_samples.saturating_sub(visible_samples);
        let start_sample =
            ((max_start as f32) * self.scroll_offset_ratio.clamp(0.0_f32, 1.0_f32)) as usize;
        let end_sample = (start_sample + visible_samples).min(self.total_samples);
        (start_sample, end_sample)
    }

    /// Convert sample index to canvas X coordinate in points.
    pub fn sample_to_screen_x(&self, sample_idx: usize, canvas_rect: Rect) -> f32 {
        let (start, end) = self.visible_sample_range();
        let range = (end - start).max(1) as f32;
        let norm = (sample_idx.saturating_sub(start) as f32 / range).clamp(0.0_f32, 1.0_f32);
        canvas_rect.x + norm * canvas_rect.width
    }

    /// Convert canvas screen X coordinate to sample index.
    pub fn screen_x_to_sample(&self, screen_x: f32, canvas_rect: Rect) -> usize {
        let (start, end) = self.visible_sample_range();
        let range = (end - start).max(1) as f32;
        let norm =
            ((screen_x - canvas_rect.x) / canvas_rect.width.max(1.0_f32)).clamp(0.0_f32, 1.0_f32);
        start + (norm * range) as usize
    }

    /// Snap sample to nearest grid subdivision (e.g. 1/16th note).
    pub fn snap_sample_to_grid(&self, sample_idx: usize) -> usize {
        let samples_per_beat = (self.sample_rate as f64 * 60.0 / self.bpm) as usize;
        let samples_per_subdiv =
            (samples_per_beat / self.grid_snap_subdivision.max(1) as usize).max(1);
        let half = samples_per_subdiv / 2;
        let snapped = ((sample_idx + half) / samples_per_subdiv) * samples_per_subdiv;
        snapped.min(self.total_samples)
    }

    /// Hit-test warp markers with minimum 44x44pt bounding touch target.
    pub fn hit_test_marker(&self, pos: (f32, f32), canvas_rect: Rect) -> Option<usize> {
        let flag_top = canvas_rect.y;
        let flag_bottom = canvas_rect.y + TRANSIENT_FLAG_HEIGHT.max(MIN_HIT_TARGET_PT);

        for (idx, marker) in self.markers.iter().enumerate() {
            let marker_x = self.sample_to_screen_x(marker.warped_sample_idx, canvas_rect);
            let dx = (pos.0 - marker_x).abs();
            // Check flag handle touch zone (X within hit radius, Y in top flag band)
            if dx <= WARP_MARKER_HIT_RADIUS
                && pos.1 >= flag_top - 10.0_f32
                && pos.1 <= flag_bottom + 10.0_f32
            {
                return Some(idx);
            }
            // Also allow hit on baseline anchor
            let baseline_y = canvas_rect.y + canvas_rect.height;
            let dy_bottom = (pos.1 - baseline_y).abs();
            if dx <= WARP_MARKER_HIT_RADIUS && dy_bottom <= WARP_MARKER_HIT_RADIUS {
                return Some(idx);
            }
        }
        None
    }

    /// Add a new warp marker at sample position.
    pub fn add_marker(&mut self, sample_idx: usize, is_pinned: bool) -> usize {
        let sample = sample_idx.min(self.total_samples);
        let id = format!("warp_{}_{}", sample, self.markers.len());
        let marker = WarpMarker::new(id, sample, 0.8_f32, is_pinned);
        self.markers.push(marker);
        self.markers.sort_by_key(|m| m.warped_sample_idx);
        self.markers
            .iter()
            .position(|m| m.warped_sample_idx == sample)
            .unwrap_or(self.markers.len() - 1)
    }

    /// Move/warp marker to target sample position.
    pub fn move_marker(&mut self, marker_idx: usize, target_sample: usize) {
        if marker_idx < self.markers.len() {
            let clamped = target_sample.min(self.total_samples);
            self.markers[marker_idx].warped_sample_idx = clamped;
            self.markers[marker_idx].is_pinned = true;
        }
    }

    /// Delete marker at index.
    pub fn delete_marker(&mut self, marker_idx: usize) -> bool {
        if marker_idx < self.markers.len() {
            self.markers.remove(marker_idx);
            if self.selected_marker_idx == Some(marker_idx) {
                self.selected_marker_idx = None;
            }
            true
        } else {
            false
        }
    }

    /// Reset all warped markers to their original unwarped timing.
    pub fn reset_all_warp(&mut self) {
        for marker in &mut self.markers {
            marker.warped_sample_idx = marker.original_sample_idx;
        }
    }

    /// Set zoom level clamped to [1.0 ..= 32.0].
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom_level = zoom.clamp(1.0_f32, 32.0_f32);
    }

    /// Generate deterministic ASCII visualization for testing and verification.
    pub fn render_ascii(&self, width: usize) -> String {
        let mut buf = vec![' '; width];
        let (start, end) = self.visible_sample_range();
        let range = (end - start).max(1) as f32;

        for marker in &self.markers {
            if marker.warped_sample_idx >= start && marker.warped_sample_idx <= end {
                let norm = (marker.warped_sample_idx - start) as f32 / range;
                let col = ((norm * (width - 1) as f32).round() as usize).min(width - 1);
                buf[col] = if marker.is_pinned { 'P' } else { '|' };
            }
        }
        buf.into_iter().collect()
    }
}

#[cfg(feature = "gui")]
impl TransientWarpEditorView {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        let total_w = ui.available_width().max(600.0_f32);
        let canvas_h = 240.0_f32;

        // 1. Header Toolbar
        ui.horizontal(|ui| {
            ui.heading(
                egui::RichText::new("TRANSIENT & AUDIO WARP EDITOR")
                    .color(Color32::from_rgb(240, 245, 255))
                    .strong(),
            );
            ui.separator();
            ui.label(
                egui::RichText::new(format!("Zoom: {:.1}x", self.zoom_level))
                    .color(Color32::from_rgb(0, 229, 255)),
            );
            if ui.button("Reset Warp").clicked() {
                self.reset_all_warp();
            }
        });

        // 2. Waveform & Warp Marker Canvas
        let (response, painter) =
            ui.allocate_painter(Vec2::new(total_w, canvas_h), egui::Sense::click_and_drag());
        let canvas_rect = Rect::new(
            response.rect.min.x,
            response.rect.min.y,
            response.rect.width(),
            response.rect.height(),
        );

        // Draw Canvas Background
        painter.rect_filled(response.rect, 6.0_f32, Color32::from_rgb(11, 15, 25));
        painter.rect_stroke(
            response.rect,
            6.0_f32,
            Stroke::new(1.5_f32, Color32::from_rgb(45, 60, 85)),
        );

        // Center baseline
        let center_y = response.rect.min.y + canvas_h * 0.5_f32;
        painter.line_segment(
            [
                egui::pos2(response.rect.min.x, center_y),
                egui::pos2(response.rect.max.x, center_y),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 110, 100)),
        );

        // Top Transient Flag Track
        let flag_y = response.rect.min.y + 24.0_f32;
        painter.line_segment(
            [
                egui::pos2(response.rect.min.x, flag_y),
                egui::pos2(response.rect.max.x, flag_y),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(70, 95, 130, 80)),
        );

        // Handle Touch and Drag Interactions
        if let Some(pos) = response.hover_pos() {
            let pt = (pos.x, pos.y);
            if response.drag_started() {
                if let Some(idx) = self.hit_test_marker(pt, canvas_rect) {
                    self.dragging_marker_idx = Some(idx);
                    self.selected_marker_idx = Some(idx);
                }
            }
        }

        if response.dragged() {
            if let (Some(drag_idx), Some(pos)) = (self.dragging_marker_idx, response.hover_pos()) {
                let target_sample = self.screen_x_to_sample(pos.x, canvas_rect);
                let snapped = self.snap_sample_to_grid(target_sample);
                self.move_marker(drag_idx, snapped);
            }
        }

        if response.drag_stopped() {
            self.dragging_marker_idx = None;
        }

        // Draw Warp Markers
        for (idx, marker) in self.markers.iter().enumerate() {
            let mx = self.sample_to_screen_x(marker.warped_sample_idx, canvas_rect);
            let is_selected = self.selected_marker_idx == Some(idx);

            let marker_col = if marker.is_pinned {
                Color32::from_rgb(0, 255, 180) // Mint green pinned
            } else {
                Color32::from_rgb(255, 180, 0) // Amber transient
            };

            // Vertical marker pin line
            let stroke_width = if is_selected { 2.5_f32 } else { 1.5_f32 };
            painter.line_segment(
                [egui::pos2(mx, flag_y), egui::pos2(mx, response.rect.max.y)],
                Stroke::new(stroke_width, marker_col),
            );

            // Flag Head Touch Handle (>=44pt hit bounding box)
            let head_rect = egui::Rect::from_center_size(
                egui::pos2(mx, flag_y),
                Vec2::new(MIN_HIT_TARGET_PT, MIN_HIT_TARGET_PT),
            );

            // Visual flag diamond
            painter.circle_filled(
                egui::pos2(mx, flag_y),
                WARP_MARKER_VISUAL_RADIUS,
                marker_col,
            );
            painter.circle_stroke(
                egui::pos2(mx, flag_y),
                WARP_MARKER_VISUAL_RADIUS,
                Stroke::new(2.0_f32, Color32::from_rgb(255, 255, 255)),
            );

            if is_selected {
                // Focus highlight ring
                painter.rect_stroke(
                    head_rect,
                    4.0_f32,
                    Stroke::new(2.0_f32, Color32::from_rgb(0, 229, 255)),
                );
            }
        }

        // 3. Selected Marker Readout Bar
        let mut delete_idx = None;
        if let Some(idx) = self.selected_marker_idx {
            if let Some(marker) = self.markers.get(idx) {
                let orig = marker.original_sample_idx;
                let warped = marker.warped_sample_idx;
                let stretch = marker.stretch_percentage();
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "Marker #{}: Orig={} -> Warped={} ({:+.1}% stretch)",
                            idx, orig, warped, stretch
                        ))
                        .color(Color32::from_rgb(255, 215, 0))
                        .strong(),
                    );
                    if ui.button("Unpin / Delete").clicked() {
                        delete_idx = Some(idx);
                    }
                });
            }
        }
        if let Some(idx) = delete_idx {
            self.delete_marker(idx);
        }
    }
}
