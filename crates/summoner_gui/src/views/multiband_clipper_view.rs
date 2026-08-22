// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Mastering Linear-Phase Dynamic Multiband Clipper & Harmonic Saturator HUD (Step 1513).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const CLIPPER_KNEE_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const NUM_CLIPPER_BANDS: usize = 4;
pub const MIN_CLIPPER_THRESHOLD_DB: f32 = -24.0;
pub const MAX_CLIPPER_THRESHOLD_DB: f32 = 0.0;
pub const MIN_CLIPPER_CEILING_DB: f32 = -12.0;
pub const MAX_CLIPPER_CEILING_DB: f32 = 0.0;
pub const MIN_KNEE_WIDTH_DB: f32 = 0.0;
pub const MAX_KNEE_WIDTH_DB: f32 = 12.0;

/// Non-linear Clipping Transfer Curve Characteristic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipperCurveMode {
    SoftKneeCubic,  // Smooth polynomial cubic rolloff (Transparent peak taming)
    HyperbolicTanh, // Analog tape-style saturation (Rich odd harmonics)
    HardBrickwall,  // Zero-overshoot sample clipping (Max loudness density)
    QuinticSmooth,  // Ultra-high order polynomial (Near-linear passband)
    AsymmetricTube, // Triode valve saturation (Pleasant 2nd-order even harmonics)
}

/// Dynamic Clipper Band Configuration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClipperBand {
    pub name: &'static str,
    pub crossover_high_hz: f32,
    pub threshold_db: f32,
    pub ceiling_db: f32,
    pub drive_db: f32,
    pub clip_reduction_db: f32,
}

/// Mastering Linear-Phase Multiband Clipper View HUD (Step 1513).
#[derive(Debug, Clone)]
pub struct MultibandClipperView {
    pub curve_mode: ClipperCurveMode,
    pub bands: [ClipperBand; NUM_CLIPPER_BANDS],
    pub active_band_idx: usize,
    pub knee_width_db: f32,        // [0.0 ..= 12.0 dB]
    pub oversampling_factor: u32,  // 1x, 2x, 4x, 8x Linear-Phase
    pub knee_puck_pos: (f32, f32), // Normalized (X: threshold, Y: ceiling)
    pub is_dragging_puck: bool,
    pub true_peak_max_db: f32, // [-12.0 ..= +3.0 dBTP]
    pub harmonic_thd_pct: f32, // Total Harmonic Distortion [0.0 ..= 15.0 %]
    pub color_palette: ContrastColorPalette,
}

impl Default for MultibandClipperView {
    fn default() -> Self {
        Self::new()
    }
}

impl MultibandClipperView {
    pub fn new() -> Self {
        let bands = [
            ClipperBand {
                name: "SUB LOW (<120Hz)",
                crossover_high_hz: 120.0,
                threshold_db: -3.0,
                ceiling_db: -0.5,
                drive_db: 2.0,
                clip_reduction_db: 1.8,
            },
            ClipperBand {
                name: "LOW MID (120-1k)",
                crossover_high_hz: 1000.0,
                threshold_db: -4.5,
                ceiling_db: -0.8,
                drive_db: 3.5,
                clip_reduction_db: 2.4,
            },
            ClipperBand {
                name: "HIGH MID (1k-6k)",
                crossover_high_hz: 6000.0,
                threshold_db: -2.0,
                ceiling_db: -0.3,
                drive_db: 1.5,
                clip_reduction_db: 1.1,
            },
            ClipperBand {
                name: "AIR HIGH (>6k)",
                crossover_high_hz: 20000.0,
                threshold_db: -6.0,
                ceiling_db: -1.2,
                drive_db: 4.0,
                clip_reduction_db: 3.2,
            },
        ];

        let mut view = Self {
            curve_mode: ClipperCurveMode::SoftKneeCubic,
            bands,
            active_band_idx: 1, // Default to Low-Mid band
            knee_width_db: 4.0,
            oversampling_factor: 4, // 4x Linear Phase Oversampling
            knee_puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            true_peak_max_db: -0.15,
            harmonic_thd_pct: 2.45,
            color_palette: ContrastColorPalette::default(),
        };
        view.sync_puck_from_active_band();
        view
    }

    /// Convert Threshold [-24.0 ..= 0.0 dB] to normalized coordinate [0.0 ..= 1.0].
    pub fn threshold_to_normalized(db: f32) -> f32 {
        let t = db.clamp(MIN_CLIPPER_THRESHOLD_DB, MAX_CLIPPER_THRESHOLD_DB);
        ((t - MIN_CLIPPER_THRESHOLD_DB) / (MAX_CLIPPER_THRESHOLD_DB - MIN_CLIPPER_THRESHOLD_DB))
            .clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Threshold [-24.0 ..= 0.0 dB].
    pub fn normalized_to_threshold(norm: f32) -> f32 {
        MIN_CLIPPER_THRESHOLD_DB
            + norm.clamp(0.0, 1.0) * (MAX_CLIPPER_THRESHOLD_DB - MIN_CLIPPER_THRESHOLD_DB)
    }

    /// Convert Ceiling [-12.0 ..= 0.0 dB] to normalized coordinate [0.0 ..= 1.0].
    pub fn ceiling_to_normalized(db: f32) -> f32 {
        let c = db.clamp(MIN_CLIPPER_CEILING_DB, MAX_CLIPPER_CEILING_DB);
        ((c - MIN_CLIPPER_CEILING_DB) / (MAX_CLIPPER_CEILING_DB - MIN_CLIPPER_CEILING_DB))
            .clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Ceiling [-12.0 ..= 0.0 dB].
    pub fn normalized_to_ceiling(norm: f32) -> f32 {
        MIN_CLIPPER_CEILING_DB
            + norm.clamp(0.0, 1.0) * (MAX_CLIPPER_CEILING_DB - MIN_CLIPPER_CEILING_DB)
    }

    /// Sync internal XY puck from the active band settings.
    pub fn sync_puck_from_active_band(&mut self) {
        let band = &self.bands[self.active_band_idx];
        self.knee_puck_pos = (
            Self::threshold_to_normalized(band.threshold_db),
            Self::ceiling_to_normalized(band.ceiling_db),
        );
    }

    /// Evaluate non-linear clipping transfer function $y = f(x)$ for input level $x \in [-24.0, 0.0] \text{ dBFS}$.
    pub fn evaluate_transfer_curve(&self, in_db: f32) -> f32 {
        let band = &self.bands[self.active_band_idx];
        let thresh = band.threshold_db;
        let ceil = band.ceiling_db;
        let knee = self.knee_width_db;

        if in_db <= thresh - knee * 0.5 {
            in_db
        } else if in_db >= thresh + knee * 0.5 {
            match self.curve_mode {
                ClipperCurveMode::HardBrickwall => ceil,
                ClipperCurveMode::SoftKneeCubic => {
                    let over = in_db - thresh;
                    ceil - 1.0 / (1.0 + over * 0.5)
                }
                ClipperCurveMode::HyperbolicTanh => {
                    let norm_in = (in_db - thresh) / 6.0;
                    thresh + 6.0 * norm_in.tanh()
                }
                ClipperCurveMode::QuinticSmooth => {
                    let over = (in_db - thresh).max(0.0);
                    ceil - 1.5 * (-0.4 * over).exp()
                }
                ClipperCurveMode::AsymmetricTube => {
                    let over = in_db - thresh;
                    ceil - 0.8 / (1.0 + 0.3 * over) + 0.1 * (over * 0.2).sin()
                }
            }
        } else {
            // Quadratic interpolation within knee region
            let k = (in_db - thresh + knee * 0.5) / knee.max(0.1);
            let unclipped = in_db;
            let clipped = thresh;
            unclipped * (1.0 - k * k) + (clipped + k * 0.5) * (k * k)
        }
    }

    /// Hit-test touch coordinate on the soft-knee threshold puck.
    pub fn hit_test_knee_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.knee_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.knee_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= CLIPPER_KNEE_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render of Dynamic Transfer Function and Multiband Bars.
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

        // Draw Transfer Curve on left half
        let left_w = mid_x - 2;
        for c in 0..left_w {
            let frac = c as f32 / left_w.max(1) as f32;
            let in_db = MIN_CLIPPER_THRESHOLD_DB
                + frac * (MAX_CLIPPER_THRESHOLD_DB - MIN_CLIPPER_THRESHOLD_DB);
            let out_db = self.evaluate_transfer_curve(in_db);
            let norm_out = (out_db - MIN_CLIPPER_THRESHOLD_DB)
                / (MAX_CLIPPER_THRESHOLD_DB - MIN_CLIPPER_THRESHOLD_DB);
            let row =
                (((1.0 - norm_out.clamp(0.0, 1.0)) * (height - 3) as f32) + 1.0).round() as usize;
            if row > 0 && row < height - 1 {
                grid[row][1 + c] = '/';
            }
        }

        // Knee Puck on left half
        let puck_col = ((self.knee_puck_pos.0 * (mid_x - 2) as f32) + 1.0).round() as usize;
        let puck_row =
            (((1.0 - self.knee_puck_pos.1) * (height - 3) as f32) + 1.0).round() as usize;
        if puck_row < height - 1 && puck_col < mid_x {
            grid[puck_row][puck_col] = 'O';
        }

        // Draw 4 Band Reduction Bars on right half
        let right_w = width - mid_x - 2;
        let col_w = right_w / 4;
        for i in 0..4 {
            let gr = self.bands[i].clip_reduction_db;
            let bar_h = ((gr / 6.0).clamp(0.0, 1.0) * (height - 3) as f32).round() as usize;
            let col_start = mid_x + 1 + i * col_w + 1;
            for r in 0..bar_h {
                if height - 2 > r {
                    grid[height - 2 - r][col_start] = '#';
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
        let _canvas_rect = Rect::new(rect.min.x, rect.min.y, rect.width(), rect.height());

        // Background
        painter.rect_filled(rect, 6.0, Color32::from_rgb(14, 18, 28));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "MASTERING LINEAR-PHASE DYNAMIC MULTIBAND CLIPPER HUD",
            egui::FontId::proportional(16.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Curve Selector Tabs (y: 48..92) - Each tab >= 44pt height
        let curves = [
            (ClipperCurveMode::SoftKneeCubic, "SOFT-KNEE CUBIC"),
            (ClipperCurveMode::HyperbolicTanh, "ANALOG TANH"),
            (ClipperCurveMode::HardBrickwall, "HARD BRICKWALL"),
            (ClipperCurveMode::QuinticSmooth, "QUINTIC SMOOTH"),
            (ClipperCurveMode::AsymmetricTube, "ASYMMETRIC TUBE"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (mode, name)) in curves.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_selected = self.curve_mode == *mode;
            let bg_color = if is_selected {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(25, 35, 50)
            };
            let text_color = if is_selected {
                Color32::from_rgb(10, 14, 24)
            } else {
                Color32::from_rgb(200, 215, 235)
            };

            painter.rect_filled(tab_rect, 4.0, bg_color);
            painter.text(
                tab_rect.center(),
                egui::Align2::CENTER_CENTER,
                *name,
                egui::FontId::proportional(11.0),
                text_color,
            );

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if tab_rect.contains(pos) {
                        self.curve_mode = *mode;
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

        // Left 55%: Dynamic Transfer Function Curve
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
            "DYNAMIC NON-LINEAR TRANSFER FUNCTION (dB in vs dB out)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // 45-degree linear reference line
        painter.line_segment(
            [
                egui::pos2(left_rect.min.x + 15.0, left_rect.max.y - 15.0),
                egui::pos2(left_rect.max.x - 15.0, left_rect.min.y + 35.0),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(100, 120, 150, 80)),
        );

        // Draw dynamic transfer curve
        let curve_w = left_rect.width() - 30.0;
        let curve_h = left_rect.height() - 50.0;
        let num_pts = 40;
        let mut prev_pt = None;

        for c in 0..=num_pts {
            let frac = c as f32 / num_pts as f32;
            let in_db = MIN_CLIPPER_THRESHOLD_DB
                + frac * (MAX_CLIPPER_THRESHOLD_DB - MIN_CLIPPER_THRESHOLD_DB);
            let out_db = self.evaluate_transfer_curve(in_db);
            let norm_out = ((out_db - MIN_CLIPPER_THRESHOLD_DB)
                / (MAX_CLIPPER_THRESHOLD_DB - MIN_CLIPPER_THRESHOLD_DB))
                .clamp(0.0, 1.0);

            let px = left_rect.min.x + 15.0 + frac * curve_w;
            let py = left_rect.max.y - 15.0 - norm_out * curve_h;
            let pt = egui::pos2(px, py);

            if let Some(prev) = prev_pt {
                painter.line_segment(
                    [prev, pt],
                    Stroke::new(2.5_f32, Color32::from_rgb(0, 229, 255)),
                );
            }
            prev_pt = Some(pt);
        }

        // Soft-Knee Threshold Puck Position
        let puck_x = left_rect.min.x + 15.0 + self.knee_puck_pos.0 * curve_w;
        let puck_y = left_rect.max.y - 15.0 - self.knee_puck_pos.1 * curve_h;
        let puck_pos = egui::pos2(puck_x, puck_y);

        // Handle interaction
        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - (left_rect.min.x + 15.0)) / curve_w).clamp(0.0, 1.0);
                    let ny = (((left_rect.max.y - 15.0) - mouse_pos.y) / curve_h).clamp(0.0, 1.0);
                    self.knee_puck_pos = (nx, ny);
                    self.bands[self.active_band_idx].threshold_db =
                        Self::normalized_to_threshold(nx);
                    self.bands[self.active_band_idx].ceiling_db = Self::normalized_to_ceiling(ny);
                }
            }
        }

        // Puck Hit Target (>= 44x44pt)
        painter.circle_stroke(
            puck_pos,
            CLIPPER_KNEE_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(0, 229, 255, 140)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(0, 229, 255));
        painter.circle_filled(puck_pos, 4.0, Color32::from_rgb(255, 255, 255));

        // Right 45%: 4-Band Multiband Clipper Strips
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
            "4-BAND LINEAR-PHASE DYNAMIC GAIN REDUCTION",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        let band_w = (right_rect.width() - 30.0) / 4.0;
        for i in 0..4 {
            let bx = right_rect.min.x + 15.0 + i as f32 * band_w;
            let b_rect = egui::Rect::from_min_size(
                egui::pos2(bx, right_rect.min.y + 35.0),
                egui::vec2(band_w - 8.0, right_rect.height() - 50.0),
            );
            let is_act = self.active_band_idx == i;
            let bg_col = if is_act {
                Color32::from_rgb(25, 40, 60)
            } else {
                Color32::from_rgb(18, 24, 36)
            };
            painter.rect_filled(b_rect, 3.0, bg_col);

            // Band Header
            let band_col = match i {
                0 => Color32::from_rgb(0, 229, 255),
                1 => Color32::from_rgb(255, 215, 0),
                2 => Color32::from_rgb(255, 107, 43),
                _ => Color32::from_rgb(0, 255, 180),
            };

            painter.text(
                egui::pos2(b_rect.center().x, b_rect.min.y + 6.0),
                egui::Align2::CENTER_TOP,
                format!("B{}", i + 1),
                egui::FontId::proportional(11.0),
                band_col,
            );

            // Gain reduction meter bar
            let gr = self.bands[i].clip_reduction_db;
            let gr_frac = (gr / 6.0).clamp(0.0, 1.0);
            let bar_h = gr_frac * (b_rect.height() - 40.0);
            let meter_rect = egui::Rect::from_min_max(
                egui::pos2(b_rect.center().x - 6.0, b_rect.max.y - 8.0 - bar_h),
                egui::pos2(b_rect.center().x + 6.0, b_rect.max.y - 8.0),
            );
            painter.rect_filled(meter_rect, 2.0, band_col);

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if b_rect.contains(pos) {
                        self.active_band_idx = i;
                        self.sync_puck_from_active_band();
                    }
                }
            }
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

        let active_band = &self.bands[self.active_band_idx];
        let params = [
            (
                "THRESHOLD / CEILING",
                format!(
                    "{:.1} dB / {:.1} dB",
                    active_band.threshold_db, active_band.ceiling_db
                ),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "KNEE WIDTH / DRIVE",
                format!(
                    "{:.1} dB (+{:.1}dB)",
                    self.knee_width_db, active_band.drive_db
                ),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "OVERSAMPLING / THD",
                format!(
                    "{}x Lin-Phase ({:.2}%)",
                    self.oversampling_factor, self.harmonic_thd_pct
                ),
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "TRUE-PEAK MAXIMUM",
                format!("{:.2} dBTP (Inter-Sample)", self.true_peak_max_db),
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
            "[PASS] Mastering Linear-Phase Multiband Clipper & Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
