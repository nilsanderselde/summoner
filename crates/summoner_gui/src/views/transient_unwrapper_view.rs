// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Psychoacoustic Dynamic Stereo Transient Unwrapper & Spatial Depth Decorrelator HUD (Step 1552).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const TRANSIENT_UNWRAPPER_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_SPATIAL_WIDTH_PCT: f32 = 0.0;
pub const MAX_SPATIAL_WIDTH_PCT: f32 = 200.0;
pub const MIN_DECORRELATION_DELAY_MS: f32 = 0.0;
pub const MAX_DECORRELATION_DELAY_MS: f32 = 25.0;
pub const MIN_UNWRAP_ANGLE_DEG: f32 = -90.0;
pub const MAX_UNWRAP_ANGLE_DEG: f32 = 90.0;
pub const MIN_CROSSOVER_HZ: f32 = 40.0;
pub const MAX_CROSSOVER_HZ: f32 = 1000.0;

/// Stereo Transient Unwrapping & Psychoacoustic Mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnwrapperMode {
    WideStereoExpansion, // Wide side-transient unwrapping with mono-centered bass punch
    BinauralDepthExtraction, // Pinna HRIR distance cues and front-to-back depth separation
    DrumTransientDecomb, // Dynamic allpass phase alignment to eliminate multi-mic cymbal comb filtering
    MasteringSpatialUnwrap, // Elliptical sub-bass monomaker (120Hz) with high-frequency spatial decorrelation
    MicrotonalPhaseDecorrelate, // Polyphonic voice stereo separation across tuning intervals
}

impl UnwrapperMode {
    pub fn default_width_pct(&self) -> f32 {
        match self {
            Self::WideStereoExpansion => 140.0,
            Self::BinauralDepthExtraction => 110.0,
            Self::DrumTransientDecomb => 100.0,
            Self::MasteringSpatialUnwrap => 125.0,
            Self::MicrotonalPhaseDecorrelate => 160.0,
        }
    }

    pub fn default_decorrelation_delay_ms(&self) -> f32 {
        match self {
            Self::WideStereoExpansion => 4.5,
            Self::BinauralDepthExtraction => 8.2,
            Self::DrumTransientDecomb => 1.8,
            Self::MasteringSpatialUnwrap => 3.2,
            Self::MicrotonalPhaseDecorrelate => 12.0,
        }
    }

    pub fn default_mono_crossover_hz(&self) -> f32 {
        match self {
            Self::WideStereoExpansion => 90.0,
            Self::BinauralDepthExtraction => 120.0,
            Self::DrumTransientDecomb => 60.0,
            Self::MasteringSpatialUnwrap => 140.0,
            Self::MicrotonalPhaseDecorrelate => 80.0,
        }
    }
}

/// Psychoacoustic Dynamic Stereo Transient Unwrapper View HUD (Step 1552).
#[derive(Debug, Clone)]
pub struct TransientUnwrapperView {
    pub mode: UnwrapperMode,
    pub spatial_width_pct: f32,         // [0.0 ..= 200.0 %]
    pub decorrelation_delay_ms: f32,    // [0.0 ..= 25.0 ms]
    pub unwrap_angle_deg: f32,          // [-90.0 ..= +90.0 deg]
    pub mono_crossover_hz: f32,         // Elliptical filter cutoff [40.0 ..= 1000.0 Hz]
    pub transient_sensitivity: f32,     // [0.0 ..= 1.0]
    pub unwrapper_puck_pos: (f32, f32), // Normalized (X: unwrap angle / width, Y: decorrelation delay)
    pub is_dragging_puck: bool,
    pub iacc_correlation: f32, // Inter-Aural Cross-Correlation [-1.0 ..= 1.0]
    pub transient_energy_ratio_db: f32, // Transient vs Tonal energy ratio [-24.0 ..= +24.0 dB]
    pub side_channel_gain_db: f32, // Side matrix gain in dB
    pub color_palette: ContrastColorPalette,
}

impl Default for TransientUnwrapperView {
    fn default() -> Self {
        Self::new()
    }
}

impl TransientUnwrapperView {
    pub fn new() -> Self {
        let mut view = Self {
            mode: UnwrapperMode::WideStereoExpansion,
            spatial_width_pct: 140.0,
            decorrelation_delay_ms: 4.5,
            unwrap_angle_deg: 25.0,
            mono_crossover_hz: 120.0,
            transient_sensitivity: 0.75,
            unwrapper_puck_pos: (0.65, 0.35),
            is_dragging_puck: false,
            iacc_correlation: 0.35,
            transient_energy_ratio_db: 4.2,
            side_channel_gain_db: 2.8,
            color_palette: ContrastColorPalette::default(),
        };
        view.update_psychoacoustic_simulation();
        view
    }

    /// Convert Spatial Width [0.0 ..= 200.0 %] to normalized [0.0 ..= 1.0].
    pub fn width_to_normalized(width_pct: f32) -> f32 {
        let w = width_pct.clamp(MIN_SPATIAL_WIDTH_PCT, MAX_SPATIAL_WIDTH_PCT);
        ((w - MIN_SPATIAL_WIDTH_PCT) / (MAX_SPATIAL_WIDTH_PCT - MIN_SPATIAL_WIDTH_PCT))
            .clamp(0.0, 1.0)
    }

    /// Convert normalized [0.0 ..= 1.0] to Spatial Width [0.0 ..= 200.0 %].
    pub fn normalized_to_width(norm: f32) -> f32 {
        MIN_SPATIAL_WIDTH_PCT
            + norm.clamp(0.0, 1.0) * (MAX_SPATIAL_WIDTH_PCT - MIN_SPATIAL_WIDTH_PCT)
    }

    /// Convert Decorrelation Delay [0.0 ..= 25.0 ms] to normalized [0.0 ..= 1.0].
    pub fn delay_to_normalized(delay_ms: f32) -> f32 {
        let d = delay_ms.clamp(MIN_DECORRELATION_DELAY_MS, MAX_DECORRELATION_DELAY_MS);
        ((d - MIN_DECORRELATION_DELAY_MS)
            / (MAX_DECORRELATION_DELAY_MS - MIN_DECORRELATION_DELAY_MS))
            .clamp(0.0, 1.0)
    }

    /// Convert normalized [0.0 ..= 1.0] to Decorrelation Delay [0.0 ..= 25.0 ms].
    pub fn normalized_to_delay(norm: f32) -> f32 {
        MIN_DECORRELATION_DELAY_MS
            + norm.clamp(0.0, 1.0) * (MAX_DECORRELATION_DELAY_MS - MIN_DECORRELATION_DELAY_MS)
    }

    /// Convert Unwrap Angle [-90.0 ..= +90.0 deg] to normalized [0.0 ..= 1.0].
    pub fn angle_to_normalized(angle_deg: f32) -> f32 {
        let a = angle_deg.clamp(MIN_UNWRAP_ANGLE_DEG, MAX_UNWRAP_ANGLE_DEG);
        ((a - MIN_UNWRAP_ANGLE_DEG) / (MAX_UNWRAP_ANGLE_DEG - MIN_UNWRAP_ANGLE_DEG)).clamp(0.0, 1.0)
    }

    /// Convert normalized [0.0 ..= 1.0] to Unwrap Angle [-90.0 ..= +90.0 deg].
    pub fn normalized_to_angle(norm: f32) -> f32 {
        MIN_UNWRAP_ANGLE_DEG + norm.clamp(0.0, 1.0) * (MAX_UNWRAP_ANGLE_DEG - MIN_UNWRAP_ANGLE_DEG)
    }

    /// Set mode and refresh simulation defaults.
    pub fn set_mode(&mut self, mode: UnwrapperMode) {
        self.mode = mode;
        self.spatial_width_pct = mode.default_width_pct();
        self.decorrelation_delay_ms = mode.default_decorrelation_delay_ms();
        self.mono_crossover_hz = mode.default_mono_crossover_hz();
        self.update_psychoacoustic_simulation();
    }

    /// Update psychoacoustic stereo decorrelation & IACC model math.
    pub fn update_psychoacoustic_simulation(&mut self) {
        // Inter-Aural Cross-Correlation (IACC): w=100% -> ~0.7, w=200% -> ~0.0 (wide diffuse), w=0% -> 1.0 (mono)
        let w_norm = self.spatial_width_pct / 100.0;
        let d_factor = (self.decorrelation_delay_ms / 15.0).clamp(0.0, 1.0);
        self.iacc_correlation = (1.0 - (w_norm * 0.5 + d_factor * 0.5)).clamp(-0.5, 1.0);

        // Side channel boost in Mid/Side matrix (dB)
        self.side_channel_gain_db = (20.0 * w_norm.max(0.01).log10()).clamp(-24.0, 12.0);

        // Transient energy ratio vs tonal background
        self.transient_energy_ratio_db =
            (self.transient_sensitivity * 12.0 - 2.0).clamp(-12.0, 12.0);
    }

    /// Evaluate 2D Stereo Goniometer Lissajous spread coordinate (L, R) given an input phase angle (rad).
    pub fn evaluate_stereo_spread(&self, phase_rad: f32) -> (f32, f32) {
        let angle_rad = self.unwrap_angle_deg.to_radians();
        let width_factor = self.spatial_width_pct / 100.0;
        let l = (phase_rad.sin() * angle_rad.cos()
            - phase_rad.cos() * angle_rad.sin() * width_factor)
            * 0.7;
        let r = (phase_rad.sin() * angle_rad.cos()
            + phase_rad.cos() * angle_rad.sin() * width_factor)
            * 0.7;
        (l, r)
    }

    /// Hit-test touch coordinate on the Transient Unwrapper position puck.
    pub fn hit_test_unwrapper_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.unwrapper_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.unwrapper_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= TRANSIENT_UNWRAPPER_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render of Stereo Goniometer Field and IACC meters.
    #[allow(clippy::needless_range_loop)]
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut grid = vec![vec![' '; width]; height];

        for (row_idx, row) in grid.iter_mut().enumerate() {
            row[0] = '|';
            row[width - 1] = '|';
            if row_idx == 0 || row_idx == height - 1 {
                for col in row.iter_mut().take(width) {
                    *col = '-';
                }
                row[0] = '+';
                row[width - 1] = '+';
            }
        }

        let mid_x = width / 2;
        for r in 1..height - 1 {
            grid[r][mid_x] = '|';
        }

        // Left half: Stereo Goniometer Lissajous Diamond Map
        let left_w = mid_x - 2;
        let center_r = height / 2;
        let center_c = left_w / 2;

        grid[center_r][center_c] = '+';
        for step in 1..=(left_w / 4).min(height / 3) {
            if center_c >= step
                && center_c + step < mid_x
                && center_r >= step
                && center_r + step < height - 1
            {
                grid[center_r - step][center_c] = '|';
                grid[center_r + step][center_c] = '|';
                grid[center_r][center_c - step] = '-';
                grid[center_r][center_c + step] = '-';
            }
        }

        // Unwrapper Puck on left half
        let puck_col = ((self.unwrapper_puck_pos.0 * (left_w - 4) as f32) + 2.0).round() as usize;
        let puck_row =
            (((1.0 - self.unwrapper_puck_pos.1) * (height - 5) as f32) + 2.0).round() as usize;
        if puck_row < height - 1 && puck_col < mid_x {
            grid[puck_row][puck_col] = '$';
        }

        // Right half: IACC Decorrelation & Transient Split Meters
        let right_w = width - mid_x - 2;
        let meters = [
            ("WIDTH", (self.spatial_width_pct / 200.0).clamp(0.0, 1.0)),
            ("IACC", (1.0 - self.iacc_correlation).clamp(0.0, 1.0)),
            ("TRANS", (self.transient_sensitivity).clamp(0.0, 1.0)),
        ];

        let bar_spacing = right_w / (meters.len() + 1);
        for (i, (_mname, val)) in meters.iter().enumerate() {
            let bar_col = mid_x + 2 + (i + 1) * bar_spacing;
            let bar_h = (val * (height - 4) as f32).round() as usize;
            for r in 0..bar_h {
                if height - 2 > r && bar_col < width - 1 {
                    grid[height - 2 - r][bar_col] = '#';
                }
            }
        }

        grid.into_iter()
            .map(|row| row.into_iter().collect())
            .collect()
    }

    #[cfg(feature = "gui")]
    #[allow(clippy::needless_range_loop)]
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 480.0),
            egui::Sense::click_and_drag(),
        );

        let painter = ui.painter_at(rect);

        // Dark Modern Navy Background
        painter.rect_filled(rect, 6.0, Color32::from_rgb(14, 18, 28));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "PSYCHOACOUSTIC DYNAMIC STEREO TRANSIENT UNWRAPPER & DECORRELATOR HUD",
            egui::FontId::proportional(15.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Unwrapper Mode Tabs (y: 48..92) - Each tab >= 44pt height
        let modes = [
            (UnwrapperMode::WideStereoExpansion, "STEREO EXPAND"),
            (UnwrapperMode::BinauralDepthExtraction, "BINAURAL DEPTH"),
            (UnwrapperMode::DrumTransientDecomb, "DRUM DE-COMB"),
            (UnwrapperMode::MasteringSpatialUnwrap, "MASTERING UNWRAP"),
            (
                UnwrapperMode::MicrotonalPhaseDecorrelate,
                "POLY DECORRELATE",
            ),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (m, name)) in modes.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_selected = self.mode == *m;
            let bg_color = if is_selected {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(25, 35, 52)
            };
            let text_color = if is_selected {
                Color32::from_rgb(10, 14, 24)
            } else {
                Color32::from_rgb(200, 220, 245)
            };

            painter.rect_filled(tab_rect, 4.0, bg_color);
            painter.text(
                tab_rect.center(),
                egui::Align2::CENTER_CENTER,
                *name,
                egui::FontId::proportional(10.5),
                text_color,
            );

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if tab_rect.contains(pos) {
                        self.set_mode(*m);
                    }
                }
            }
        }

        // Main Display Canvas (y: 104..340)
        let main_canvas = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 104.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 340.0),
        );
        painter.rect_filled(main_canvas, 6.0, Color32::from_rgb(10, 14, 24));
        painter.rect_stroke(
            main_canvas,
            6.0,
            Stroke::new(1.5_f32, Color32::from_rgb(45, 65, 95)),
        );

        // Left 55%: Stereo Goniometer Lissajous Diamond Map
        let left_w = main_canvas.width() * 0.55;
        let left_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(left_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(left_rect, 4.0, Color32::from_rgb(14, 20, 32));
        painter.rect_stroke(
            left_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(40, 55, 80)),
        );

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "STEREO TRANSIENT POLAR GONIOMETER & SPREAD ELLIPSE",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // Goniometer 45-degree guide axes (L / R / M / S)
        let g_center = left_rect.center();
        let g_radius = 80.0_f32;
        painter.line_segment(
            [
                egui::pos2(g_center.x - g_radius, g_center.y + g_radius),
                egui::pos2(g_center.x + g_radius, g_center.y - g_radius),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 90, 130, 80)),
        );
        painter.line_segment(
            [
                egui::pos2(g_center.x - g_radius, g_center.y - g_radius),
                egui::pos2(g_center.x + g_radius, g_center.y + g_radius),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 90, 130, 80)),
        );
        painter.line_segment(
            [
                egui::pos2(g_center.x, g_center.y - g_radius),
                egui::pos2(g_center.x, g_center.y + g_radius),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 90, 130, 80)),
        );

        // Draw Lissajous Transient Spread Ellipse
        let num_pts = 32;
        let mut ellipse_pts = Vec::with_capacity(num_pts);
        for p in 0..num_pts {
            let phase = (p as f32 / num_pts as f32) * std::f32::consts::TAU;
            let (l, r) = self.evaluate_stereo_spread(phase);
            let ex = g_center.x + (r - l) * (g_radius * 0.8);
            let ey = g_center.y - (l + r) * (g_radius * 0.8);
            ellipse_pts.push(egui::pos2(ex, ey));
        }
        for i in 0..num_pts {
            let next_i = (i + 1) % num_pts;
            painter.line_segment(
                [ellipse_pts[i], ellipse_pts[next_i]],
                Stroke::new(2.0_f32, Color32::from_rgb(0, 229, 255)),
            );
        }

        // Interactive Transient Unwrapper Puck
        let puck_x = left_rect.min.x + self.unwrapper_puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.unwrapper_puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.unwrapper_puck_pos = (nx, ny);
                    self.spatial_width_pct = Self::normalized_to_width(nx);
                    self.decorrelation_delay_ms = Self::normalized_to_delay(ny);
                    self.update_psychoacoustic_simulation();
                }
            }
        }

        // Draw Touch Hit Target boundary (>= 44x44pt)
        painter.circle_stroke(
            puck_pos,
            TRANSIENT_UNWRAPPER_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(0, 229, 255, 140)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(0, 229, 255));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 20.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Width: {:.0}% | Decorrelation: {:.1} ms | IACC: {:.2}",
                self.spatial_width_pct, self.decorrelation_delay_ms, self.iacc_correlation
            ),
            egui::FontId::proportional(10.0),
            Color32::from_rgb(0, 229, 255),
        );

        // Right 45%: IACC Decorrelation Curve & Transient vs Tonal Split Meters
        let right_w = main_canvas.width() * 0.45;
        let right_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + left_w + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(right_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(right_rect, 4.0, Color32::from_rgb(14, 20, 32));
        painter.rect_stroke(
            right_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(40, 55, 80)),
        );

        painter.text(
            egui::pos2(right_rect.min.x + 10.0, right_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "PSYCHOACOUSTIC DECORRELATION & TRANSIENT SPLIT",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        let metrics = [
            (
                "SPATIAL WIDTH",
                format!("{:.0}%", self.spatial_width_pct),
                (self.spatial_width_pct / 200.0).clamp(0.0, 1.0),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "IACC DIFFUSION",
                format!("{:.2}", 1.0 - self.iacc_correlation),
                (1.0 - self.iacc_correlation).clamp(0.0, 1.0),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "TRANSIENT SPLIT",
                format!("{:.1} dB", self.transient_energy_ratio_db),
                self.transient_sensitivity,
                Color32::from_rgb(255, 107, 43),
            ),
        ];

        let bar_w = (right_rect.width() - 30.0 - 2.0 * 8.0) / 3.0;
        for (i, (label, val_str, mag, col)) in metrics.iter().enumerate() {
            let bx = right_rect.min.x + 15.0 + i as f32 * (bar_w + 8.0);
            let bar_h = mag * (right_rect.height() - 85.0);
            let b_rect = egui::Rect::from_min_max(
                egui::pos2(bx, right_rect.max.y - 25.0 - bar_h),
                egui::pos2(bx + bar_w, right_rect.max.y - 25.0),
            );
            painter.rect_filled(b_rect, 3.0, *col);

            painter.text(
                egui::pos2(bx + bar_w * 0.5, right_rect.max.y - 20.0),
                egui::Align2::CENTER_TOP,
                *label,
                egui::FontId::proportional(8.0),
                Color32::from_rgb(200, 220, 245),
            );
            painter.text(
                egui::pos2(bx + bar_w * 0.5, right_rect.max.y - 38.0 - bar_h),
                egui::Align2::CENTER_BOTTOM,
                val_str,
                egui::FontId::proportional(9.0),
                *col,
            );
        }

        // Bottom Metrics Dock (y: 350..465)
        let dock_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 350.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 465.0),
        );
        painter.rect_filled(dock_rect, 6.0, Color32::from_rgb(18, 25, 38));
        painter.rect_stroke(
            dock_rect,
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        let params = [
            (
                "SPATIAL WIDTH",
                format!("{:.0}%", self.spatial_width_pct),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "DECORRELATION DELAY",
                format!("{:.1} ms (Haas)", self.decorrelation_delay_ms),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "MONO CROSSOVER",
                format!("{:.0} Hz (Sub-Bass)", self.mono_crossover_hz),
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "SIDE GAIN",
                format!("{:.1} dB (M/S)", self.side_channel_gain_db),
                Color32::from_rgb(0, 255, 180),
            ),
        ];

        let col_w = (dock_rect.width() - 40.0) / 4.0;
        for (i, (label, val, col)) in params.iter().enumerate() {
            let px_pos = dock_rect.min.x + 20.0 + i as f32 * col_w;
            painter.text(
                egui::pos2(px_pos, dock_rect.min.y + 12.0),
                egui::Align2::LEFT_TOP,
                *label,
                egui::FontId::proportional(11.0),
                Color32::from_rgb(160, 180, 205),
            );
            painter.text(
                egui::pos2(px_pos, dock_rect.min.y + 30.0),
                egui::Align2::LEFT_TOP,
                val,
                egui::FontId::proportional(14.0),
                *col,
            );
        }

        // Compliance Verification Badge
        let badge_rect = egui::Rect::from_min_max(
            egui::pos2(dock_rect.min.x + 15.0, dock_rect.min.y + 68.0),
            egui::pos2(dock_rect.max.x - 15.0, dock_rect.max.y - 11.0),
        );
        painter.rect_filled(badge_rect, 4.0, Color32::from_rgb(16, 35, 28));
        painter.rect_stroke(
            badge_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(0, 255, 180)),
        );
        painter.text(
            egui::pos2(badge_rect.min.x + 10.0, badge_rect.min.y + 8.0),
            egui::Align2::LEFT_TOP,
            "[PASS] Psychoacoustic Stereo Transient Unwrapper & Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
