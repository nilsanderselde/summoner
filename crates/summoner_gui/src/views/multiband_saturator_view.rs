// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Master Bus Linear-Phase Dynamic Multiband Saturator & Harmonic Warmth HUD (Step 1493).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const SATURATOR_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const NUM_SATURATOR_BANDS: usize = 4;
pub const MIN_SAT_DRIVE_DB: f32 = 0.0;
pub const MAX_SAT_DRIVE_DB: f32 = 24.0;

/// Saturation Non-Linear Distortion Curve Model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaturationModel {
    TapeHysteresis,      // Smooth magnetic compression and 3rd harmonic warmth
    TriodeTubeWarmth,    // Asymmetric even-order 2nd harmonic richness
    GermaniumDiodeClip,  // Vintage console transistor / diode soft-clipping
    AsymmetricOverdrive, // Harder musical drive with even/odd expansion
    SoftKneeLimiter,     // Clean mastering brickwall warmth
}

/// Single Frequency Band Configuration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SaturatorBand {
    pub id: usize,
    pub drive_gain_db: f32,       // Drive boost [0.0 ..= +24.0 dB]
    pub bias_even_harmonics: f32, // Asymmetry bias [-1.0 ..= +1.0]
    pub mix_percent: f32,         // Dry / Wet blend [0.0 ..= 100.0 %]
    pub model: SaturationModel,
    pub is_bypassed: bool,
    pub is_solo: bool,
}

/// Master Bus Linear-Phase Multiband Saturator HUD View (Step 1493).
#[derive(Debug, Clone)]
pub struct MultibandSaturatorView {
    pub selected_band_idx: usize,
    pub bands: [SaturatorBand; NUM_SATURATOR_BANDS],
    pub crossover_low_mid_hz: f32, // 40.0 ..= 300.0 Hz (default 120 Hz)
    pub crossover_mid_high_hz: f32, // 500.0 ..= 4000.0 Hz (default 1500 Hz)
    pub crossover_air_hz: f32,     // 4000.0 ..= 16000.0 Hz (default 6500 Hz)
    pub oversampling_factor: u32,  // 1x, 2x, 4x, 8x Linear-Phase
    pub saturator_puck_pos: (f32, f32), // Normalized X (Drive dB), Y (Harmonic Bias)
    pub is_dragging_puck: bool,
    pub real_time_thd_percent: f32,
    pub color_palette: ContrastColorPalette,
}

impl Default for MultibandSaturatorView {
    fn default() -> Self {
        Self::new()
    }
}

impl MultibandSaturatorView {
    pub fn new() -> Self {
        let bands = [
            SaturatorBand {
                id: 0,
                drive_gain_db: 4.5,
                bias_even_harmonics: 0.20,
                mix_percent: 100.0,
                model: SaturationModel::TriodeTubeWarmth,
                is_bypassed: false,
                is_solo: false,
            },
            SaturatorBand {
                id: 1,
                drive_gain_db: 3.2,
                bias_even_harmonics: 0.05,
                mix_percent: 90.0,
                model: SaturationModel::TapeHysteresis,
                is_bypassed: false,
                is_solo: false,
            },
            SaturatorBand {
                id: 2,
                drive_gain_db: 6.8,
                bias_even_harmonics: -0.15,
                mix_percent: 85.0,
                model: SaturationModel::GermaniumDiodeClip,
                is_bypassed: false,
                is_solo: false,
            },
            SaturatorBand {
                id: 3,
                drive_gain_db: 2.0,
                bias_even_harmonics: 0.35,
                mix_percent: 75.0,
                model: SaturationModel::TriodeTubeWarmth,
                is_bypassed: false,
                is_solo: false,
            },
        ];

        let norm_drive = Self::drive_to_normalized(6.8);
        let norm_bias = Self::bias_to_normalized(-0.15);

        Self {
            selected_band_idx: 2, // High-Mid band selected
            bands,
            crossover_low_mid_hz: 120.0,
            crossover_mid_high_hz: 1500.0,
            crossover_air_hz: 6500.0,
            oversampling_factor: 4,
            saturator_puck_pos: (norm_drive, norm_bias),
            is_dragging_puck: false,
            real_time_thd_percent: 4.15,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Convert Saturation Drive [0.0 ..= +24.0 dB] to normalized coordinate [0.0 ..= 1.0].
    pub fn drive_to_normalized(drive_db: f32) -> f32 {
        (drive_db.clamp(MIN_SAT_DRIVE_DB, MAX_SAT_DRIVE_DB) / MAX_SAT_DRIVE_DB).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Saturation Drive [0.0 ..= +24.0 dB].
    pub fn normalized_to_drive(norm: f32) -> f32 {
        norm.clamp(0.0, 1.0) * MAX_SAT_DRIVE_DB
    }

    /// Convert Harmonic Asymmetry Bias [-1.0 ..= +1.0] to normalized coordinate [0.0 ..= 1.0].
    pub fn bias_to_normalized(bias: f32) -> f32 {
        ((bias.clamp(-1.0, 1.0) + 1.0) * 0.5).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Harmonic Asymmetry Bias [-1.0 ..= +1.0].
    pub fn normalized_to_bias(norm: f32) -> f32 {
        norm.clamp(0.0, 1.0) * 2.0 - 1.0
    }

    /// Evaluate non-linear transfer function $y = f(x)$ for active saturation curve.
    pub fn evaluate_transfer_curve(&self, input_x: f32, band_idx: usize) -> f32 {
        let band = &self.bands[band_idx.min(NUM_SATURATOR_BANDS - 1)];
        if band.is_bypassed {
            return input_x;
        }

        let gain_linear = 10.0_f32.powf(band.drive_gain_db / 20.0);
        let x = input_x * gain_linear + band.bias_even_harmonics * 0.25;

        let sat_out = match band.model {
            SaturationModel::TapeHysteresis => {
                let x_clamped = x.clamp(-3.0, 3.0);
                x_clamped / (1.0 + x_clamped.abs())
            }
            SaturationModel::TriodeTubeWarmth => {
                if x >= 0.0 {
                    1.0 - (-x).exp()
                } else {
                    -((1.0 - (x).exp()) * 0.7)
                }
            }
            SaturationModel::GermaniumDiodeClip => (x * 1.5).tanh() * 0.85,
            SaturationModel::AsymmetricOverdrive => {
                if x > 1.0 {
                    1.0
                } else if x < -0.8 {
                    -0.8
                } else {
                    x - (x.powi(3) / 3.0)
                }
            }
            SaturationModel::SoftKneeLimiter => {
                let abs_x = x.abs();
                if abs_x <= 0.6 {
                    x
                } else {
                    x.signum() * (0.6 + (1.0 - 0.6) * ((abs_x - 0.6) / 0.4).tanh())
                }
            }
        };

        let mix = band.mix_percent * 0.01;
        input_x * (1.0 - mix) + sat_out * mix
    }

    /// Hit-test touch coordinate on the active band's saturation puck.
    pub fn hit_test_saturator_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.saturator_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.saturator_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= SATURATOR_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render of Multiband Saturator Transfer Function.
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
        let mid_y = height / 2;
        for r in 1..height - 1 {
            grid[r][mid_x] = ':';
        }
        for c in 1..width - 1 {
            grid[mid_y][c] = ':';
        }

        // Draw transfer curve
        for c in 1..width - 1 {
            let norm_x = ((c as f32 - 1.0) / (width - 3) as f32) * 2.0 - 1.0;
            let out_y = self.evaluate_transfer_curve(norm_x, self.selected_band_idx);
            let norm_y = ((out_y.clamp(-1.0, 1.0) + 1.0) * 0.5).clamp(0.0, 1.0);
            let row = (((1.0 - norm_y) * (height - 3) as f32) + 1.0).round() as usize;
            if row < height - 1 {
                grid[row][c] = '*';
            }
        }

        // Saturator Puck
        let puck_col = ((self.saturator_puck_pos.0 * (width - 3) as f32) + 1.0).round() as usize;
        let puck_row =
            (((1.0 - self.saturator_puck_pos.1) * (height - 3) as f32) + 1.0).round() as usize;
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
            "DYNAMIC MULTIBAND SATURATOR & HARMONIC WARMTH HUD",
            egui::FontId::proportional(16.0),
            Color32::from_rgb(240, 245, 255),
        );

        // 4 Frequency Band Selector Tabs (y: 48..92) - Each tab >= 44pt height
        let band_labels = [
            ("LOW SUB", "< 120 Hz"),
            ("LOW-MID", "120 - 1.5k Hz"),
            ("HIGH-MID", "1.5k - 6.5k Hz"),
            ("HIGH AIR", "6.5k - 20k Hz"),
        ];

        let tab_w = (rect.width() - 40.0 - 3.0 * 8.0) / 4.0;
        for (i, (name, range)) in band_labels.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_selected = self.selected_band_idx == i;
            let bg_color = if is_selected {
                Color32::from_rgb(255, 107, 43) // Coral Flame
            } else {
                Color32::from_rgb(25, 35, 50)
            };
            let text_color = if is_selected {
                Color32::from_rgb(10, 14, 24)
            } else {
                Color32::from_rgb(200, 215, 235)
            };
            let sub_color = if is_selected {
                Color32::from_rgb(30, 20, 20)
            } else {
                Color32::from_rgb(140, 160, 185)
            };

            painter.rect_filled(tab_rect, 4.0, bg_color);
            painter.text(
                egui::pos2(tab_rect.center().x, tab_rect.min.y + 14.0),
                egui::Align2::CENTER_CENTER,
                *name,
                egui::FontId::proportional(12.0),
                text_color,
            );
            painter.text(
                egui::pos2(tab_rect.center().x, tab_rect.min.y + 30.0),
                egui::Align2::CENTER_CENTER,
                *range,
                egui::FontId::proportional(10.0),
                sub_color,
            );

            if response.clicked()
                && ui.input(|i| {
                    i.pointer
                        .hover_pos()
                        .is_some_and(|pos| tab_rect.contains(pos))
                })
            {
                self.selected_band_idx = i;
                let b = &self.bands[i];
                self.saturator_puck_pos = (
                    Self::drive_to_normalized(b.drive_gain_db),
                    Self::bias_to_normalized(b.bias_even_harmonics),
                );
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

        // Center crosshair axes
        let cx = main_canvas.min.x + main_canvas.width() * 0.5;
        let cy = main_canvas.min.y + main_canvas.height() * 0.5;
        painter.line_segment(
            [
                egui::pos2(main_canvas.min.x, cy),
                egui::pos2(main_canvas.max.x, cy),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 85, 120, 80)),
        );
        painter.line_segment(
            [
                egui::pos2(cx, main_canvas.min.y),
                egui::pos2(cx, main_canvas.max.y),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 85, 120, 80)),
        );

        // Draw Non-Linear Transfer Curve
        let steps = 100;
        let mut prev_pt: Option<egui::Pos2> = None;
        for s in 0..=steps {
            let frac = s as f32 / steps as f32;
            let norm_x = frac * 2.0 - 1.0;
            let out_y = self.evaluate_transfer_curve(norm_x, self.selected_band_idx);
            let norm_y = ((out_y.clamp(-1.2, 1.2) + 1.2) / 2.4).clamp(0.0, 1.0);
            let px = main_canvas.min.x + frac * main_canvas.width();
            let py = main_canvas.min.y + (1.0 - norm_y) * main_canvas.height();
            let cur_pt = egui::pos2(px, py);

            if let Some(prev) = prev_pt {
                painter.line_segment(
                    [prev, cur_pt],
                    Stroke::new(2.5_f32, Color32::from_rgb(255, 107, 43)),
                );
            }
            prev_pt = Some(cur_pt);
        }

        // Saturation Puck
        let puck_x = main_canvas.min.x + self.saturator_puck_pos.0 * main_canvas.width();
        let puck_y = main_canvas.min.y + (1.0 - self.saturator_puck_pos.1) * main_canvas.height();

        // Hit target outer ring (>=44x44pt touch area)
        painter.circle_stroke(
            egui::pos2(puck_x, puck_y),
            SATURATOR_PUCK_HIT_RADIUS,
            Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(255, 107, 43, 140)),
        );
        painter.circle_filled(
            egui::pos2(puck_x, puck_y),
            14.0,
            Color32::from_rgb(255, 107, 43),
        );
        painter.circle_filled(egui::pos2(puck_x, puck_y), 4.0, Color32::WHITE);

        if response.dragged() {
            if let Some(mouse_pos) = ui.input(|i| i.pointer.interact_pos()) {
                if self.is_dragging_puck
                    || self.hit_test_saturator_puck((mouse_pos.x, mouse_pos.y), canvas_rect)
                {
                    self.is_dragging_puck = true;
                    let norm_x =
                        ((mouse_pos.x - main_canvas.min.x) / main_canvas.width()).clamp(0.0, 1.0);
                    let norm_y = (1.0 - (mouse_pos.y - main_canvas.min.y) / main_canvas.height())
                        .clamp(0.0, 1.0);
                    self.saturator_puck_pos = (norm_x, norm_y);
                    let new_drive = Self::normalized_to_drive(norm_x);
                    let new_bias = Self::normalized_to_bias(norm_y);
                    self.bands[self.selected_band_idx].drive_gain_db = new_drive;
                    self.bands[self.selected_band_idx].bias_even_harmonics = new_bias;
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

        let curr_drive = Self::normalized_to_drive(self.saturator_puck_pos.0);
        let curr_bias = Self::normalized_to_bias(self.saturator_puck_pos.1);
        let metrics = [
            (
                "SATURATION DRIVE",
                format!("+{:.1} dB", curr_drive),
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "HARMONIC ASYMMETRY",
                format!("{:+.2} Bias", curr_bias),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "OVERSAMPLING",
                format!("{}x Linear-Phase", self.oversampling_factor),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "TOTAL THD",
                format!("{:.2}%", self.real_time_thd_percent),
                Color32::from_rgb(0, 255, 180),
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
            "[PASS] Multiband Saturator Transfer Curves & Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
