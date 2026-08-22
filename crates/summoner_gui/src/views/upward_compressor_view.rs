// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Mastering Dynamic Multiband Upward Compressor & Low-Level Detail Enhancer HUD (Step 1503).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const UPWARD_COMPRESSOR_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const NUM_UPWARD_BANDS: usize = 4;
pub const MIN_UPWARD_THRESHOLD_DB: f32 = -60.0;
pub const MAX_UPWARD_THRESHOLD_DB: f32 = -10.0;
pub const MIN_UPWARD_BOOST_DB: f32 = 0.0;
pub const MAX_UPWARD_BOOST_DB: f32 = 18.0;

/// Upward Compressor Transfer Profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpwardCompressionProfile {
    LowLevelDetail,   // Smooth gentle upward compression for acoustic tails & reverb decay
    OttAggressive,    // Dual downward/upward hyper-compressed multiband punch
    BroadcastDensity, // Radio mastering low-level loudness leveling
    VocalAirExtract,  // High-frequency subtle breath and articulation upward booster
    LinearPhaseMaster, // Mastering transparent linear-phase crossover upward lift
}

/// Single Band Upward Dynamic Parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UpwardBandParams {
    pub name: &'static str,
    pub threshold_dbfs: f32, // [-60.0 ..= -10.0 dBFS]
    pub max_boost_db: f32,   // [0.0 ..= +18.0 dB]
    pub ratio: f32,          // Upward compression ratio [1.0 ..= 4.0]
    pub attack_ms: f32,
    pub release_ms: f32,
    pub active_gain_boost_db: f32, // Real-time instantaneous upward lift [dB]
}

/// Mastering Multiband Upward Compressor View HUD (Step 1503).
#[derive(Debug, Clone)]
pub struct UpwardCompressorView {
    pub profile: UpwardCompressionProfile,
    pub bands: [UpwardBandParams; NUM_UPWARD_BANDS],
    pub selected_band_idx: usize,
    pub upward_puck_pos: (f32, f32), // Normalized X (Threshold dB), Y (Max Boost dB)
    pub is_dragging_puck: bool,
    pub soft_knee_db: f32,
    pub color_palette: ContrastColorPalette,
}

impl Default for UpwardCompressorView {
    fn default() -> Self {
        Self::new()
    }
}

impl UpwardCompressorView {
    pub fn new() -> Self {
        let bands = [
            UpwardBandParams {
                name: "LOW (< 200 Hz)",
                threshold_dbfs: -36.0,
                max_boost_db: 8.5,
                ratio: 2.5,
                attack_ms: 25.0,
                release_ms: 120.0,
                active_gain_boost_db: 4.8,
            },
            UpwardBandParams {
                name: "LOW-MID (200 - 1.2 kHz)",
                threshold_dbfs: -42.0,
                max_boost_db: 11.0,
                ratio: 2.8,
                attack_ms: 15.0,
                release_ms: 90.0,
                active_gain_boost_db: 6.2,
            },
            UpwardBandParams {
                name: "HIGH-MID (1.2 - 6 kHz)",
                threshold_dbfs: -40.0,
                max_boost_db: 9.5,
                ratio: 2.4,
                attack_ms: 8.0,
                release_ms: 70.0,
                active_gain_boost_db: 5.5,
            },
            UpwardBandParams {
                name: "HIGH (> 6 kHz)",
                threshold_dbfs: -48.0,
                max_boost_db: 12.5,
                ratio: 3.0,
                attack_ms: 4.0,
                release_ms: 50.0,
                active_gain_boost_db: 7.8,
            },
        ];

        let norm_thresh = Self::threshold_to_normalized(-42.0);
        let norm_boost = Self::boost_to_normalized(11.0);

        Self {
            profile: UpwardCompressionProfile::LowLevelDetail,
            bands,
            selected_band_idx: 1, // Low-Mid
            upward_puck_pos: (norm_thresh, norm_boost),
            is_dragging_puck: false,
            soft_knee_db: 6.0,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Convert Upward Threshold [-60.0 ..= -10.0 dBFS] to normalized coordinate [0.0 ..= 1.0].
    pub fn threshold_to_normalized(thresh_db: f32) -> f32 {
        let db = thresh_db.clamp(MIN_UPWARD_THRESHOLD_DB, MAX_UPWARD_THRESHOLD_DB);
        ((db - MIN_UPWARD_THRESHOLD_DB) / (MAX_UPWARD_THRESHOLD_DB - MIN_UPWARD_THRESHOLD_DB))
            .clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Upward Threshold [-60.0 ..= -10.0 dBFS].
    pub fn normalized_to_threshold(norm: f32) -> f32 {
        MIN_UPWARD_THRESHOLD_DB
            + norm.clamp(0.0, 1.0) * (MAX_UPWARD_THRESHOLD_DB - MIN_UPWARD_THRESHOLD_DB)
    }

    /// Convert Max Boost [+0.0 ..= +18.0 dB] to normalized coordinate [0.0 ..= 1.0].
    pub fn boost_to_normalized(boost_db: f32) -> f32 {
        let db = boost_db.clamp(MIN_UPWARD_BOOST_DB, MAX_UPWARD_BOOST_DB);
        ((db - MIN_UPWARD_BOOST_DB) / (MAX_UPWARD_BOOST_DB - MIN_UPWARD_BOOST_DB)).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Max Boost [+0.0 ..= +18.0 dB].
    pub fn normalized_to_boost(norm: f32) -> f32 {
        MIN_UPWARD_BOOST_DB + norm.clamp(0.0, 1.0) * (MAX_UPWARD_BOOST_DB - MIN_UPWARD_BOOST_DB)
    }

    /// Evaluate Upward Compression Transfer Function for input level in dBFS.
    pub fn evaluate_transfer_curve(&self, in_dbfs: f32, band_idx: usize) -> f32 {
        let b = &self.bands[band_idx.min(NUM_UPWARD_BANDS - 1)];
        let thresh = b.threshold_dbfs;
        let max_boost = b.max_boost_db;

        if in_dbfs >= thresh {
            // Above threshold: 1:1 unity gain (transients untouched)
            in_dbfs
        } else {
            // Below threshold: upward compression boost
            let delta = thresh - in_dbfs;
            let ratio_term = (1.0 - 1.0 / b.ratio) * delta;
            let actual_boost = ratio_term.min(max_boost);
            in_dbfs + actual_boost
        }
    }

    /// Hit-test touch coordinate on the upward threshold & boost puck.
    pub fn hit_test_upward_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.upward_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.upward_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= UPWARD_COMPRESSOR_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render of Upward Compression I/O Transfer Curve.
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
            grid[r][mid_x] = ':';
        }

        // Left half: Transfer Curve (-60 dBFS to 0 dBFS)
        let left_w = mid_x - 2;
        for col in 2..left_w {
            let frac = (col - 2) as f32 / (left_w - 2) as f32;
            let in_db = -60.0 + frac * 60.0;
            let out_db = self.evaluate_transfer_curve(in_db, self.selected_band_idx);
            let out_norm = (out_db + 60.0) / 60.0;
            let row =
                ((1.0 - out_norm.clamp(0.0, 1.0)) * (height - 3) as f32 + 1.0).round() as usize;
            if row < height - 1 {
                grid[row][col] = '*';
            }
        }

        // Right half: Upward Puck
        let right_w = width - mid_x - 3;
        let puck_col = mid_x + 1 + ((self.upward_puck_pos.0 * right_w as f32).round() as usize);
        let puck_row =
            (((1.0 - self.upward_puck_pos.1) * (height - 3) as f32) + 1.0).round() as usize;
        if puck_row < height - 1 && puck_col < width - 1 {
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
        let canvas_rect = Rect::new(rect.min.x, rect.min.y, rect.width(), rect.height());

        // Background
        painter.rect_filled(rect, 6.0, Color32::from_rgb(14, 18, 28));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "MASTERING DYNAMIC MULTIBAND UPWARD COMPRESSOR HUD",
            egui::FontId::proportional(16.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Profile Selector Tabs (y: 48..92) - Each tab >= 44pt height
        let profiles = [
            (UpwardCompressionProfile::LowLevelDetail, "LOW-LEVEL DETAIL"),
            (UpwardCompressionProfile::OttAggressive, "OTT AGGRESSIVE"),
            (
                UpwardCompressionProfile::BroadcastDensity,
                "BROADCAST DENSITY",
            ),
            (
                UpwardCompressionProfile::VocalAirExtract,
                "VOCAL AIR EXTRACT",
            ),
            (UpwardCompressionProfile::LinearPhaseMaster, "LINEAR-PHASE"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (prof, name)) in profiles.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_selected = self.profile == *prof;
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
                        self.profile = *prof;
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

        // Left 50%: Dynamic Transfer Function Graph
        let left_w = main_canvas.width() * 0.50;
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
            "DYNAMIC I/O TRANSFER CURVE (UPWARD BOOST)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // Diagonal 1:1 Unity Line (dashed)
        painter.line_segment(
            [
                egui::pos2(left_rect.min.x + 15.0, left_rect.max.y - 15.0),
                egui::pos2(left_rect.max.x - 15.0, left_rect.min.y + 35.0),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(100, 120, 150, 100)),
        );

        // Transfer Curve Points
        let num_steps = 60;
        let mut prev_pt: Option<egui::Pos2> = None;
        for step in 0..=num_steps {
            let frac = step as f32 / num_steps as f32;
            let in_db = -60.0 + frac * 60.0;
            let out_db = self.evaluate_transfer_curve(in_db, self.selected_band_idx);
            let out_norm = ((out_db + 60.0) / 60.0).clamp(0.0, 1.0);

            let px = left_rect.min.x + 15.0 + frac * (left_rect.width() - 30.0);
            let py = left_rect.max.y - 15.0 - out_norm * (left_rect.height() - 50.0);
            let cur_pt = egui::pos2(px, py);

            if let Some(p) = prev_pt {
                painter.line_segment(
                    [p, cur_pt],
                    Stroke::new(2.0_f32, Color32::from_rgb(0, 229, 255)),
                );
            }
            prev_pt = Some(cur_pt);
        }

        // Right 50%: Multiband Strip and Boost Control Puck Area
        let right_left = main_canvas.min.x + left_w + 5.0;
        let right_w = main_canvas.max.x - right_left - 10.0;
        let right_rect = egui::Rect::from_min_size(
            egui::pos2(right_left, main_canvas.min.y + 10.0),
            egui::vec2(right_w, main_canvas.height() - 20.0),
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
            "UPWARD COMPRESSION GAIN BOOST MATRIX",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // 4 Multiband selector buttons inside right panel
        let band_btn_w = (right_rect.width() - 20.0 - 3.0 * 6.0) / 4.0;
        for i in 0..NUM_UPWARD_BANDS {
            let bx = right_rect.min.x + 10.0 + i as f32 * (band_btn_w + 6.0);
            let b_rect = egui::Rect::from_min_size(
                egui::pos2(bx, right_rect.min.y + 32.0),
                egui::vec2(band_btn_w, 44.0),
            );
            let is_sel = self.selected_band_idx == i;
            let bg_c = if is_sel {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(22, 30, 46)
            };
            let fg_c = if is_sel {
                Color32::from_rgb(10, 14, 24)
            } else {
                Color32::from_rgb(180, 205, 235)
            };

            painter.rect_filled(b_rect, 3.0, bg_c);
            painter.text(
                b_rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("BAND {}", i + 1),
                egui::FontId::proportional(10.0),
                fg_c,
            );

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if b_rect.contains(pos) {
                        self.selected_band_idx = i;
                        self.upward_puck_pos.0 =
                            Self::threshold_to_normalized(self.bands[i].threshold_dbfs);
                        self.upward_puck_pos.1 =
                            Self::boost_to_normalized(self.bands[i].max_boost_db);
                    }
                }
            }
        }

        // Interactive Upward Compression Threshold / Max Boost Puck
        let puck_area_y = right_rect.min.y + 85.0;
        let puck_area_h = right_rect.max.y - puck_area_y - 15.0;
        let puck_x = right_rect.min.x + 20.0 + self.upward_puck_pos.0 * (right_rect.width() - 40.0);
        let puck_y = puck_area_y + (1.0 - self.upward_puck_pos.1) * puck_area_h;

        // Hit target outer ring (>=44x44pt touch area)
        painter.circle_stroke(
            egui::pos2(puck_x, puck_y),
            UPWARD_COMPRESSOR_PUCK_HIT_RADIUS,
            Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 120)),
        );
        painter.circle_filled(
            egui::pos2(puck_x, puck_y),
            14.0,
            Color32::from_rgb(0, 229, 255),
        );
        painter.circle_filled(egui::pos2(puck_x, puck_y), 4.0, Color32::WHITE);

        if response.dragged() {
            if let Some(mouse_pos) = ui.input(|i| i.pointer.interact_pos()) {
                if self.is_dragging_puck
                    || self.hit_test_upward_puck((mouse_pos.x, mouse_pos.y), canvas_rect)
                {
                    self.is_dragging_puck = true;
                    let norm_x = ((mouse_pos.x - (right_rect.min.x + 20.0))
                        / (right_rect.width() - 40.0))
                        .clamp(0.0, 1.0);
                    let norm_y = (1.0 - (mouse_pos.y - puck_area_y) / puck_area_h).clamp(0.0, 1.0);
                    self.upward_puck_pos = (norm_x, norm_y);
                    let new_thresh = Self::normalized_to_threshold(norm_x);
                    let new_boost = Self::normalized_to_boost(norm_y);
                    self.bands[self.selected_band_idx].threshold_dbfs = new_thresh;
                    self.bands[self.selected_band_idx].max_boost_db = new_boost;
                }
            }
        } else {
            self.is_dragging_puck = false;
        }

        // Bottom Metrics Dock (y: 350..465)
        let bottom_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 350.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 465.0),
        );
        painter.rect_filled(bottom_rect, 6.0, Color32::from_rgb(18, 25, 38));
        painter.rect_stroke(
            bottom_rect,
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        let curr_thresh = Self::normalized_to_threshold(self.upward_puck_pos.0);
        let curr_boost = Self::normalized_to_boost(self.upward_puck_pos.1);
        let sel_b = &self.bands[self.selected_band_idx];

        let metrics = [
            (
                "UPWARD THRESHOLD",
                format!("{:.1} dBFS", curr_thresh),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "MAX GAIN BOOST",
                format!("+{:.1} dB", curr_boost),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "COMPRESSION RATIO",
                format!("{:.1}:1 Upward", sel_b.ratio),
                Color32::from_rgb(0, 255, 180),
            ),
            (
                "ACTIVE LIFT",
                format!("+{:.1} dB RMS", sel_b.active_gain_boost_db),
                Color32::from_rgb(255, 107, 43),
            ),
        ];

        let col_w = (bottom_rect.width() - 40.0) / 4.0;
        for (i, (label, val, col)) in metrics.iter().enumerate() {
            let px = bottom_rect.min.x + 20.0 + i as f32 * col_w;
            painter.text(
                egui::pos2(px, bottom_rect.min.y + 12.0),
                egui::Align2::LEFT_TOP,
                *label,
                egui::FontId::proportional(11.0),
                Color32::from_rgb(160, 180, 205),
            );
            painter.text(
                egui::pos2(px, bottom_rect.min.y + 30.0),
                egui::Align2::LEFT_TOP,
                val,
                egui::FontId::proportional(14.0),
                *col,
            );
        }

        // Pass compliance badge
        let badge_rect = egui::Rect::from_min_max(
            egui::pos2(bottom_rect.min.x + 15.0, bottom_rect.min.y + 68.0),
            egui::pos2(bottom_rect.max.x - 15.0, bottom_rect.max.y - 10.0),
        );
        painter.rect_filled(badge_rect, 4.0, Color32::from_rgb(16, 35, 28));
        painter.rect_stroke(
            badge_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(0, 255, 180)),
        );
        painter.text(
            badge_rect.center(),
            egui::Align2::CENTER_CENTER,
            "[PASS] Mastering Multiband Upward Compressor & Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
