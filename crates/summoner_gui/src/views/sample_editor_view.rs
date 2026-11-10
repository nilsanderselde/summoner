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

    // Task D7: Selection, loop points, spectral view, and undo/redo history
    pub selection_start: Option<usize>,
    pub selection_end: Option<usize>,
    pub loop_enabled: bool,
    pub loop_start_sample: usize,
    pub loop_end_sample: usize,
    pub loop_crossfade_samples: usize,
    pub spectral_view_enabled: bool,
    #[serde(skip)]
    pub undo_stack: Vec<Vec<f32>>,
    #[serde(skip)]
    pub redo_stack: Vec<Vec<f32>>,
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

        let num_samples = default_samples.len();
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
            selection_start: None,
            selection_end: None,
            loop_enabled: false,
            loop_start_sample: 0,
            loop_end_sample: num_samples,
            loop_crossfade_samples: 128,
            spectral_view_enabled: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
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

    // =========================================================================
    // Task D7: Destructive & Non-Destructive Editing, Loop Points & Spectral View
    // =========================================================================

    /// Get current selection range clamped to waveform boundaries.
    pub fn get_selection_range(&self) -> (usize, usize) {
        if self.waveform_samples.is_empty() {
            return (0, 0);
        }
        let max_idx = self.waveform_samples.len();
        match (self.selection_start, self.selection_end) {
            (Some(s), Some(e)) => {
                let start = s.min(e).min(max_idx);
                let end = s.max(e).min(max_idx);
                (start, end)
            }
            _ => (0, max_idx),
        }
    }

    /// Save current state to undo history before making modifications.
    pub fn push_undo_state(&mut self) {
        self.undo_stack.push(self.waveform_samples.clone());
        self.redo_stack.clear();
        // Limit undo stack depth
        if self.undo_stack.len() > 64 {
            self.undo_stack.remove(0);
        }
    }

    /// Undo last edit.
    pub fn undo(&mut self) -> bool {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(self.waveform_samples.clone());
            self.waveform_samples = prev;
            true
        } else {
            false
        }
    }

    /// Redo last undone edit.
    pub fn redo(&mut self) -> bool {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.waveform_samples.clone());
            self.waveform_samples = next;
            true
        } else {
            false
        }
    }

    /// Trim waveform to selection (destructive).
    pub fn trim_selection(&mut self) -> bool {
        let (start, end) = self.get_selection_range();
        if end <= start || self.waveform_samples.is_empty() {
            return false;
        }

        self.push_undo_state();
        self.waveform_samples = self.waveform_samples[start..end].to_vec();
        self.selection_start = None;
        self.selection_end = None;
        self.loop_start_sample = 0;
        self.loop_end_sample = self.waveform_samples.len();
        true
    }

    /// Reverse samples within selection.
    pub fn reverse_selection(&mut self) -> bool {
        let (start, end) = self.get_selection_range();
        if end <= start {
            return false;
        }

        self.push_undo_state();
        self.waveform_samples[start..end].reverse();
        true
    }

    /// Normalize selection to target dBFS (e.g. 0.0 dBFS).
    pub fn normalize_selection(&mut self, target_db: f32) -> bool {
        let (start, end) = self.get_selection_range();
        if end <= start {
            return false;
        }

        let mut peak = 0.0f32;
        for &s in &self.waveform_samples[start..end] {
            let abs_s = s.abs();
            if abs_s > peak {
                peak = abs_s;
            }
        }

        if peak <= 1e-6 {
            return false;
        }

        self.push_undo_state();
        let target_linear = 10.0f32.powf(target_db / 20.0);
        let factor = target_linear / peak;

        for s in &mut self.waveform_samples[start..end] {
            *s = (*s * factor).clamp(-1.0, 1.0);
        }
        true
    }

    /// Invert phase (polarity) of selection.
    pub fn invert_phase_selection(&mut self) -> bool {
        let (start, end) = self.get_selection_range();
        if end <= start {
            return false;
        }

        self.push_undo_state();
        for s in &mut self.waveform_samples[start..end] {
            *s = -*s;
        }
        true
    }

    /// Apply fade-in over selection (cosine equal-power or linear).
    pub fn fade_in_selection(&mut self) -> bool {
        let (start, end) = self.get_selection_range();
        let len = end.saturating_sub(start);
        if len == 0 {
            return false;
        }

        self.push_undo_state();
        for i in 0..len {
            let t = i as f32 / len as f32;
            let gain = (t * std::f32::consts::PI * 0.5).sin();
            self.waveform_samples[start + i] *= gain;
        }
        true
    }

    /// Apply fade-out over selection.
    pub fn fade_out_selection(&mut self) -> bool {
        let (start, end) = self.get_selection_range();
        let len = end.saturating_sub(start);
        if len == 0 {
            return false;
        }

        self.push_undo_state();
        for i in 0..len {
            let t = (len - 1 - i) as f32 / len as f32;
            let gain = (t * std::f32::consts::PI * 0.5).sin();
            self.waveform_samples[start + i] *= gain;
        }
        true
    }

    /// Adjust gain of selection by dB amount.
    pub fn adjust_gain_selection(&mut self, gain_db: f32) -> bool {
        let (start, end) = self.get_selection_range();
        if end <= start {
            return false;
        }

        self.push_undo_state();
        let factor = 10.0f32.powf(gain_db / 20.0);
        for s in &mut self.waveform_samples[start..end] {
            *s = (*s * factor).clamp(-1.0, 1.0);
        }
        true
    }

    /// Silence selection (zero out samples).
    pub fn silence_selection(&mut self) -> bool {
        let (start, end) = self.get_selection_range();
        if end <= start {
            return false;
        }

        self.push_undo_state();
        for s in &mut self.waveform_samples[start..end] {
            *s = 0.0;
        }
        true
    }

    /// Set loop points for seamless sampler / playback looping.
    pub fn set_loop_points(&mut self, start_sample: usize, end_sample: usize) {
        let max_idx = self.waveform_samples.len();
        self.loop_start_sample = start_sample.min(max_idx);
        self.loop_end_sample = end_sample.min(max_idx).max(self.loop_start_sample);
        self.loop_enabled = true;
    }

    /// Auto-snap loop start and end to nearest zero crossings to avoid clicks.
    pub fn snap_loop_to_zero_crossing(&mut self) {
        self.loop_start_sample = self.find_nearest_zero_crossing(self.loop_start_sample);
        self.loop_end_sample = self.find_nearest_zero_crossing(self.loop_end_sample);
    }

    fn find_nearest_zero_crossing(&self, sample: usize) -> usize {
        if self.waveform_samples.len() < 2 {
            return sample;
        }
        let window = 256;
        let start = sample.saturating_sub(window);
        let end = (sample + window).min(self.waveform_samples.len() - 1);

        let mut best_idx = sample;
        let mut min_dist = usize::MAX;

        for i in start..end {
            let s1 = self.waveform_samples[i];
            let s2 = self.waveform_samples[i + 1];
            if (s1 >= 0.0 && s2 <= 0.0) || (s1 <= 0.0 && s2 >= 0.0) {
                let dist = (i as isize - sample as isize).unsigned_abs();
                if dist < min_dist {
                    min_dist = dist;
                    best_idx = if s1.abs() <= s2.abs() { i } else { i + 1 };
                }
            }
        }

        best_idx
    }

    /// Compute a spectral slice (magnitude spectrum in dB) centered around a sample index.
    pub fn compute_spectral_slice(&self, center_sample: usize, window_size: usize) -> Vec<f32> {
        let half_win = window_size / 2;
        let start = center_sample.saturating_sub(half_win);
        let num_bins = window_size / 2;
        let mut magnitudes = vec![0.0f32; num_bins];

        if self.waveform_samples.is_empty() {
            return magnitudes;
        }

        // Hann-windowed DFT analysis
        let mut windowed = vec![0.0f32; window_size];
        for (i, win_val) in windowed.iter_mut().enumerate().take(window_size) {
            let idx = start + i;
            let sample_val = if idx < self.waveform_samples.len() {
                self.waveform_samples[idx]
            } else {
                0.0
            };
            // Hann window
            let w = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / window_size as f32).cos());
            *win_val = sample_val * w;
        }

        for (k, mag_entry) in magnitudes.iter_mut().enumerate().take(num_bins) {
            let mut re = 0.0f32;
            let mut im = 0.0f32;
            for (n, &win_samp) in windowed.iter().enumerate().take(window_size) {
                let angle = 2.0 * std::f32::consts::PI * k as f32 * n as f32 / window_size as f32;
                re += win_samp * angle.cos();
                im -= win_samp * angle.sin();
            }
            let mag = (re * re + im * im).sqrt() / (window_size as f32 * 0.5);
            let mag_db = (20.0 * (mag + 1e-6).log10()).clamp(-120.0, 0.0);
            *mag_entry = mag_db;
        }

        magnitudes
    }

    /// Compute full spectrogram matrix (time slices x freq bins in dB) for current viewport.
    pub fn compute_spectrogram(
        &self,
        num_time_slices: usize,
        num_freq_bins: usize,
    ) -> Vec<Vec<f32>> {
        let total_samples = self.waveform_samples.len();
        if total_samples == 0 || num_time_slices == 0 {
            return Vec::new();
        }

        let window_size = num_freq_bins * 2;
        let step = total_samples as f32 / num_time_slices as f32;
        let mut spectrogram = Vec::with_capacity(num_time_slices);

        for s in 0..num_time_slices {
            let center = (s as f32 * step) as usize;
            spectrogram.push(self.compute_spectral_slice(center, window_size));
        }

        spectrogram
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_editor_editing_and_undo_redo() {
        let mut editor = SampleEditorView::new();
        editor.waveform_samples = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];
        editor.selection_start = Some(2);
        editor.selection_end = Some(6);

        // Reverse selection: [0.3, 0.4, 0.5, 0.6] -> [0.6, 0.5, 0.4, 0.3]
        assert!(editor.reverse_selection());
        assert_eq!(
            editor.waveform_samples,
            vec![0.1, 0.2, 0.6, 0.5, 0.4, 0.3, 0.7, 0.8]
        );

        // Undo reverse
        assert!(editor.undo());
        assert_eq!(
            editor.waveform_samples,
            vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8]
        );

        // Redo reverse
        assert!(editor.redo());
        assert_eq!(
            editor.waveform_samples,
            vec![0.1, 0.2, 0.6, 0.5, 0.4, 0.3, 0.7, 0.8]
        );

        // Invert phase of selection
        assert!(editor.invert_phase_selection());
        assert_eq!(
            editor.waveform_samples,
            vec![0.1, 0.2, -0.6, -0.5, -0.4, -0.3, 0.7, 0.8]
        );

        // Silence selection
        assert!(editor.silence_selection());
        assert_eq!(
            editor.waveform_samples,
            vec![0.1, 0.2, 0.0, 0.0, 0.0, 0.0, 0.7, 0.8]
        );

        // Trim selection
        assert!(editor.trim_selection());
        assert_eq!(editor.waveform_samples, vec![0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_sample_editor_normalization_and_fades() {
        let mut editor = SampleEditorView::new();
        editor.waveform_samples = vec![0.1, 0.2, 0.5, 0.2, 0.1];
        editor.selection_start = None;
        editor.selection_end = None; // Full buffer

        // Normalize to 0 dBFS (peak = 1.0)
        assert!(editor.normalize_selection(0.0));
        assert!((editor.waveform_samples[2] - 1.0).abs() < 1e-4);
        assert!((editor.waveform_samples[1] - 0.4).abs() < 1e-4);

        // Fade in
        assert!(editor.fade_in_selection());
        assert_eq!(editor.waveform_samples[0], 0.0);
        assert!(editor.waveform_samples[4] > 0.1);
    }

    #[test]
    fn test_sample_editor_loop_points_and_spectral_analysis() {
        let mut editor = SampleEditorView::new();
        // 440Hz sine wave
        let sample_rate = 44100.0;
        let mut samples = Vec::new();
        for i in 0..4410 {
            let t = i as f32 / sample_rate as f32;
            samples.push((2.0 * std::f32::consts::PI * 440.0 * t).sin());
        }
        editor.waveform_samples = samples;
        editor.sample_rate = sample_rate;

        // Set loop points
        editor.set_loop_points(100, 2000);
        assert!(editor.loop_enabled);
        assert_eq!(editor.loop_start_sample, 100);
        assert_eq!(editor.loop_end_sample, 2000);

        // Snap loop points to zero crossing
        editor.snap_loop_to_zero_crossing();
        let s_val = editor.waveform_samples[editor.loop_start_sample];
        let e_val = editor.waveform_samples[editor.loop_end_sample];
        assert!(s_val.abs() < 0.1);
        assert!(e_val.abs() < 0.1);

        // Compute spectral slice (FFT magnitude bins)
        let slice = editor.compute_spectral_slice(1000, 256);
        assert_eq!(slice.len(), 128); // 128 bins for 256-point window
        // 440 Hz bin at 44.1kHz with 256 window is bin round(440 * 256 / 44100) = ~3
        assert!(slice[3] > -20.0); // Strong peak near 440Hz

        // Spectrogram matrix
        let spec = editor.compute_spectrogram(8, 64);
        assert_eq!(spec.len(), 8);
        assert_eq!(spec[0].len(), 64);
    }
}
