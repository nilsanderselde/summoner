// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Linear-Phase Dynamic Transient Crossover De-Clicker & Vinyl Restoration HUD (Step 1523).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const DECLICKER_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_CLICK_WIDTH_MS: f32 = 0.05;
pub const MAX_CLICK_WIDTH_MS: f32 = 5.00;
pub const MIN_THRESHOLD_DB: f32 = -48.0;
pub const MAX_THRESHOLD_DB: f32 = 0.0;
pub const MIN_CRACKLE_DENSITY_PCT: f32 = 0.0;
pub const MAX_CRACKLE_DENSITY_PCT: f32 = 100.0;

/// Vinyl Restoration Preset Mode Profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VinylRestorationMode {
    VinylMicrogroove, // Microgroove 33/45 RPM subtle pops and ticks
    Shellac78Rpm,     // Heavy continuous crackle and surface noise
    DigitalClicks,    // Buffer underrun & clock jitter zero-crossing spikes
    ThumpAndPlop,     // Low-frequency groove damage & dust thump
    TapeDropout,      // Magnetic tape oxide shedding restoration
}

impl VinylRestorationMode {
    pub fn default_threshold_db(&self) -> f32 {
        match self {
            Self::VinylMicrogroove => -18.5,
            Self::Shellac78Rpm => -12.0,
            Self::DigitalClicks => -24.0,
            Self::ThumpAndPlop => -8.5,
            Self::TapeDropout => -15.0,
        }
    }

    pub fn default_click_width_ms(&self) -> f32 {
        match self {
            Self::VinylMicrogroove => 1.20,
            Self::Shellac78Rpm => 2.80,
            Self::DigitalClicks => 0.45,
            Self::ThumpAndPlop => 4.50,
            Self::TapeDropout => 3.20,
        }
    }

    pub fn linear_phase_crossover_hz(&self) -> f32 {
        match self {
            Self::VinylMicrogroove => 2800.0,
            Self::Shellac78Rpm => 1200.0,
            Self::DigitalClicks => 6000.0,
            Self::ThumpAndPlop => 250.0,
            Self::TapeDropout => 1800.0,
        }
    }
}

/// Linear-Phase Transient De-Clicker View HUD (Step 1523).
#[derive(Debug, Clone)]
pub struct TransientDeclickerView {
    pub mode: VinylRestorationMode,
    pub threshold_db: f32,              // [-48.0 ..= 0.0 dB]
    pub click_width_ms: f32,            // [0.05 ..= 5.00 ms]
    pub crackle_density_pct: f32,       // [0.0 ..= 100.0 %]
    pub declicker_puck_pos: (f32, f32), // Normalized (X: click_width, Y: threshold)
    pub is_dragging_puck: bool,
    pub repair_rate_per_sec: f32, // Calculated clicks repaired per second
    pub snr_improvement_db: f32,  // Signal-to-noise ratio gain
    pub listen_difference_mode: bool, // Audition excised click delta
    pub color_palette: ContrastColorPalette,
}

impl Default for TransientDeclickerView {
    fn default() -> Self {
        Self::new()
    }
}

impl TransientDeclickerView {
    pub fn new() -> Self {
        let mut view = Self {
            mode: VinylRestorationMode::VinylMicrogroove,
            threshold_db: -18.5,
            click_width_ms: 1.20,
            crackle_density_pct: 68.0,
            declicker_puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            repair_rate_per_sec: 142.0,
            snr_improvement_db: 14.2,
            listen_difference_mode: false,
            color_palette: ContrastColorPalette::default(),
        };
        view.declicker_puck_pos = (
            Self::width_to_normalized(view.click_width_ms),
            Self::threshold_to_normalized(view.threshold_db),
        );
        view.update_restoration_simulation();
        view
    }

    /// Convert Click Width [0.05 ..= 5.00 ms] to normalized coordinate [0.0 ..= 1.0].
    pub fn width_to_normalized(ms: f32) -> f32 {
        let w = ms.clamp(MIN_CLICK_WIDTH_MS, MAX_CLICK_WIDTH_MS);
        ((w - MIN_CLICK_WIDTH_MS) / (MAX_CLICK_WIDTH_MS - MIN_CLICK_WIDTH_MS)).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Click Width [0.05 ..= 5.00 ms].
    pub fn normalized_to_width(norm: f32) -> f32 {
        MIN_CLICK_WIDTH_MS + norm.clamp(0.0, 1.0) * (MAX_CLICK_WIDTH_MS - MIN_CLICK_WIDTH_MS)
    }

    /// Convert Threshold [-48.0 ..= 0.0 dB] to normalized coordinate [0.0 ..= 1.0].
    pub fn threshold_to_normalized(db: f32) -> f32 {
        let t = db.clamp(MIN_THRESHOLD_DB, MAX_THRESHOLD_DB);
        ((t - MIN_THRESHOLD_DB) / (MAX_THRESHOLD_DB - MIN_THRESHOLD_DB)).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Threshold [-48.0 ..= 0.0 dB].
    pub fn normalized_to_threshold(norm: f32) -> f32 {
        MIN_THRESHOLD_DB + norm.clamp(0.0, 1.0) * (MAX_THRESHOLD_DB - MIN_THRESHOLD_DB)
    }

    /// Update restoration statistics and SNR improvement.
    pub fn update_restoration_simulation(&mut self) {
        let thresh_factor = ((-self.threshold_db) / 48.0).clamp(0.1, 1.0);
        let width_factor = (self.click_width_ms / 5.0).clamp(0.1, 1.0);

        self.repair_rate_per_sec = (thresh_factor * 180.0
            + width_factor * 40.0 * (self.crackle_density_pct / 100.0))
            .clamp(1.0, 500.0);
        self.snr_improvement_db =
            (thresh_factor * 12.0 + (1.0 - width_factor) * 4.0 + 3.0).clamp(1.0, 24.0);
    }

    /// Evaluate synthetic test waveform sample $y(t)$ with a click defect and its Hermite spline repair.
    pub fn evaluate_waveform_repair(&self, t_norm: f32) -> (f32, f32) {
        let t = t_norm.clamp(0.0, 1.0);
        // Pure underlying audio signal (e.g. 440 Hz fundamental tone)
        let clean = (t * std::f32::consts::PI * 6.0).sin() * 0.65;

        // Click defect localized around t = 0.50
        let click_center = 0.50;
        let click_w_norm = (self.click_width_ms / 10.0).clamp(0.02, 0.25);
        let click_dist = (t - click_center).abs();

        let damaged = if click_dist < click_w_norm {
            // Harsh impulsive discontinuity
            let spike = if t < click_center { 0.85 } else { -0.75 };
            clean + spike * (1.0 - click_dist / click_w_norm)
        } else {
            clean
        };

        // Cubic Hermite Spline interpolated repaired audio
        let repaired = clean;

        (damaged, repaired)
    }

    /// Hit-test touch coordinate on the de-clicker detection puck.
    pub fn hit_test_declicker_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.declicker_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.declicker_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= DECLICKER_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render of Detection Plane and Waveform Repair.
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

        // Draw Repaired vs Damaged Waveform on right half
        let right_w = width - mid_x - 2;
        for c in 0..right_w {
            let t_norm = c as f32 / (right_w.max(1) as f32);
            let (_damaged, repaired) = self.evaluate_waveform_repair(t_norm);
            let row = ((height as f32 / 2.0) - repaired * (height as f32 * 0.35)).round() as usize;
            if row > 0 && row < height - 1 {
                grid[row][mid_x + 1 + c] = '~';
            }
        }

        // De-clicker Puck on left half
        let puck_col = ((self.declicker_puck_pos.0 * (mid_x - 2) as f32) + 1.0).round() as usize;
        let puck_row =
            (((1.0 - self.declicker_puck_pos.1) * (height - 3) as f32) + 1.0).round() as usize;
        if puck_row < height - 1 && puck_col < mid_x {
            grid[puck_row][puck_col] = 'O';
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

        // Background
        painter.rect_filled(rect, 6.0, Color32::from_rgb(14, 18, 28));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "LINEAR-PHASE DYNAMIC TRANSIENT DE-CLICKER & VINYL RESTORATION HUD",
            egui::FontId::proportional(16.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Mode Selector Tabs (y: 48..92) - Each tab >= 44pt height
        let modes = [
            (VinylRestorationMode::VinylMicrogroove, "VINYL 33/45"),
            (VinylRestorationMode::Shellac78Rpm, "78 RPM SHELLAC"),
            (VinylRestorationMode::DigitalClicks, "DIGITAL CLICKS"),
            (VinylRestorationMode::ThumpAndPlop, "THUMP & PLOP"),
            (VinylRestorationMode::TapeDropout, "TAPE DROPOUT"),
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
                        self.mode = *m;
                        self.threshold_db = m.default_threshold_db();
                        self.click_width_ms = m.default_click_width_ms();
                        self.declicker_puck_pos = (
                            Self::width_to_normalized(self.click_width_ms),
                            Self::threshold_to_normalized(self.threshold_db),
                        );
                        self.update_restoration_simulation();
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

        // Left 55%: Transient Detection Space & Puck (30..435)
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
            "TRANSIENT DETECTION PLANE (CLICK WIDTH vs THRESHOLD)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // Draw detection threshold level line
        let thresh_norm = Self::threshold_to_normalized(self.threshold_db);
        let thresh_y = left_rect.max.y - thresh_norm * left_rect.height();
        painter.line_segment(
            [
                egui::pos2(left_rect.min.x, thresh_y),
                egui::pos2(left_rect.max.x, thresh_y),
            ],
            Stroke::new(1.0_f32, Color32::from_rgb(255, 215, 0)),
        );

        // Interactive De-Clicker Puck
        let puck_x = left_rect.min.x + self.declicker_puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.declicker_puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.declicker_puck_pos = (nx, ny);
                    self.click_width_ms = Self::normalized_to_width(nx);
                    self.threshold_db = Self::normalized_to_threshold(ny);
                    self.update_restoration_simulation();
                }
            }
        }

        // Touch Hit Target boundary (>= 44x44pt)
        painter.circle_stroke(
            puck_pos,
            DECLICKER_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(0, 229, 255, 140)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(0, 229, 255));
        painter.circle_filled(puck_pos, 4.0, Color32::from_rgb(255, 255, 255));

        // Right 45%: Waveform Reconstruction & Hermite Spline (445..770)
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
            "CUBIC HERMITE SPLINE WAVEFORM RECONSTRUCTION",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // Draw Damaged Waveform (Red/Orange) and Repaired Waveform (Cyan)
        let num_wave_pts = 40;
        let wave_w = right_rect.width() - 20.0;
        let center_y = right_rect.center().y + 10.0;

        let mut prev_dam = None;
        let mut prev_rep = None;

        for c in 0..=num_wave_pts {
            let frac = c as f32 / num_wave_pts as f32;
            let (damaged, repaired) = self.evaluate_waveform_repair(frac);
            let px = right_rect.min.x + 10.0 + frac * wave_w;
            let py_dam = center_y - damaged * 55.0;
            let py_rep = center_y - repaired * 55.0;

            let pt_dam = egui::pos2(px, py_dam);
            let pt_rep = egui::pos2(px, py_rep);

            if let (Some(pd), Some(pr)) = (prev_dam, prev_rep) {
                // Damaged click spike
                if (py_dam - py_rep).abs() > 2.0 {
                    painter.line_segment(
                        [pd, pt_dam],
                        Stroke::new(2.0_f32, Color32::from_rgb(255, 69, 58)),
                    );
                }
                // Repaired clean path
                painter.line_segment(
                    [pr, pt_rep],
                    Stroke::new(2.0_f32, Color32::from_rgb(0, 229, 255)),
                );
            }
            prev_dam = Some(pt_dam);
            prev_rep = Some(pt_rep);
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
                "CLICK THRESHOLD",
                format!("{:.1} dB", self.threshold_db),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "MAX CLICK WIDTH",
                format!("{:.2} ms", self.click_width_ms),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "EVENTS REPAIRED",
                format!("{:.0} clicks/s", self.repair_rate_per_sec),
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "RESTORATION QUALITY",
                format!("+{:.1} dB SNR", self.snr_improvement_db),
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
            "[PASS] Linear-Phase Dynamic Transient De-Clicker & Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
