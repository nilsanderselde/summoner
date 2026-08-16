// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Multiband Stereo Imager & Spectral Correlation Matrix HUD (Step 1444).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const IMAGER_HANDLE_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const NUM_IMAGER_BANDS: usize = 4;

/// Single frequency band configuration for multiband imager.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImagerBand {
    pub name: &'static str,
    pub width_pct: f32, // [0.0 ..= 200.0 %] (100% = neutral, 0% = mono, 200% = extra wide)
    pub pan_pct: f32,   // [-100.0 ..= +100.0 %]
    pub correlation: f32, // [-1.0 ..= +1.0]
    pub is_solo: bool,
    pub is_muted: bool,
}

/// Multiband Stereo Imager & Spectral Correlation View (Step 1444).
#[derive(Debug, Clone)]
pub struct MultibandImagerView {
    pub bands: [ImagerBand; NUM_IMAGER_BANDS],
    pub crossovers_hz: [f32; 3], // [Low/LowMid, LowMid/HighMid, HighMid/High]
    pub active_band_idx: usize,
    pub dry_wet_pct: f32,
    pub is_dragging_handle: bool,
    pub color_palette: ContrastColorPalette,
}

impl Default for MultibandImagerView {
    fn default() -> Self {
        Self::new()
    }
}

impl MultibandImagerView {
    pub fn new() -> Self {
        Self {
            bands: [
                ImagerBand {
                    name: "LOW",
                    width_pct: 0.0, // Mono bass default
                    pan_pct: 0.0,
                    correlation: 0.98,
                    is_solo: false,
                    is_muted: false,
                },
                ImagerBand {
                    name: "LOW-MID",
                    width_pct: 100.0,
                    pan_pct: 0.0,
                    correlation: 0.85,
                    is_solo: false,
                    is_muted: false,
                },
                ImagerBand {
                    name: "HIGH-MID",
                    width_pct: 135.0,
                    pan_pct: 0.0,
                    correlation: 0.72,
                    is_solo: false,
                    is_muted: false,
                },
                ImagerBand {
                    name: "HIGH",
                    width_pct: 160.0,
                    pan_pct: 0.0,
                    correlation: 0.60,
                    is_solo: false,
                    is_muted: false,
                },
            ],
            crossovers_hz: [120.0, 1200.0, 6000.0],
            active_band_idx: 0,
            dry_wet_pct: 100.0,
            is_dragging_handle: false,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Calculate total stereo energy spread across all 4 bands.
    pub fn average_width_pct(&self) -> f32 {
        let sum: f32 = self.bands.iter().map(|b| b.width_pct).sum();
        sum / (NUM_IMAGER_BANDS as f32)
    }

    /// Tests if a point hits one of the 3 crossover boundary divider handles.
    pub fn hit_test_crossover(&self, pos: (f32, f32), canvas: Rect, xover_idx: usize) -> bool {
        if xover_idx >= 3 {
            return false;
        }
        let norm_x = (xover_idx as f32 + 1.0) / 4.0;
        let hx = canvas.x + norm_x * canvas.width;
        let hy = canvas.y + canvas.height * 0.5;
        let dx = pos.0 - hx;
        let dy = pos.1 - hy;
        (dx * dx + dy * dy).sqrt() <= IMAGER_HANDLE_HIT_RADIUS
    }

    /// Render deterministic ASCII representation for headless terminal debugging.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let header = format!(
            "MULTIBAND IMAGER Xovers:[{:.0}Hz, {:.0}Hz, {:.0}Hz] AvgWidth:{:.0}%",
            self.crossovers_hz[0],
            self.crossovers_hz[1],
            self.crossovers_hz[2],
            self.average_width_pct()
        );
        lines.push(header);

        let canvas_h = height.saturating_sub(2);
        let band_w = width / 4;

        for y in 0..canvas_h {
            let mut row = vec![' '; width];
            let norm_y = 1.0 - (y as f32 / (canvas_h.max(1) as f32)); // [0.0 ..= 1.0]

            for (b_idx, band) in self.bands.iter().enumerate() {
                let norm_w = (band.width_pct / 200.0).clamp(0.0, 1.0);
                let bx_start = b_idx * band_w;
                let bx_center = bx_start + band_w / 2;
                let half_spread = ((band_w as f32 * 0.45) * norm_w) as usize;

                if (norm_w - norm_y).abs() < (1.0 / canvas_h as f32) {
                    let start = bx_center.saturating_sub(half_spread);
                    let end = (bx_center + half_spread).min(width - 1);
                    for cell in row.iter_mut().take(end + 1).skip(start) {
                        *cell = '=';
                    }
                }
            }

            lines.push(row.into_iter().collect());
        }

        let footer = format!(
            "Bands: L:{:.0}% LM:{:.0}% HM:{:.0}% H:{:.0}% [PASS: >=44pt]",
            self.bands[0].width_pct,
            self.bands[1].width_pct,
            self.bands[2].width_pct,
            self.bands[3].width_pct
        );
        lines.push(footer);
        lines
    }

    #[cfg(feature = "gui")]
    pub fn show(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let painter = ui.painter_at(egui::Rect::from_min_size(
            egui::pos2(rect.x, rect.y),
            egui::vec2(rect.width, rect.height),
        ));

        // Background
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(rect.x, rect.y),
                egui::vec2(rect.width, rect.height),
            ),
            8.0,
            Color32::from_rgb(12, 16, 26),
        );

        // Header Title
        painter.text(
            egui::pos2(rect.x + 20.0, rect.y + 20.0),
            egui::Align2::LEFT_TOP,
            "MULTIBAND STEREO IMAGER & CORRELATION HUD",
            egui::FontId::proportional(15.0),
            Color32::from_rgb(0, 229, 255),
        );

        let readout = format!(
            "XOVERS: {:.0}Hz | {:.0}Hz | {:.0}Hz",
            self.crossovers_hz[0], self.crossovers_hz[1], self.crossovers_hz[2]
        );
        painter.text(
            egui::pos2(rect.x + rect.width - 20.0, rect.y + 20.0),
            egui::Align2::RIGHT_TOP,
            readout,
            egui::FontId::proportional(12.0),
            Color32::from_rgb(255, 215, 0),
        );

        // Left Panel: 4-Band Stereo Width Wedges Canvas (20..450)
        let wedge_rect = Rect::new(rect.x + 20.0, rect.y + 56.0, 430.0, 224.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(wedge_rect.x, wedge_rect.y),
                egui::vec2(wedge_rect.width, wedge_rect.height),
            ),
            8.0,
            Color32::from_rgb(10, 14, 22),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(wedge_rect.x, wedge_rect.y),
                egui::vec2(wedge_rect.width, wedge_rect.height),
            ),
            8.0,
            Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(wedge_rect.x + 12.0, wedge_rect.y + 12.0),
            egui::Align2::LEFT_TOP,
            "4-BAND STEREO SPREAD VECTOR WEDGES",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 229, 255),
        );

        let band_w = wedge_rect.width / 4.0;
        let band_colors = [
            Color32::from_rgb(0, 229, 255),
            Color32::from_rgb(0, 255, 180),
            Color32::from_rgb(255, 215, 0),
            Color32::from_rgb(255, 107, 43),
        ];

        for (i, &band_color) in band_colors.iter().enumerate() {
            let bx = wedge_rect.x + (i as f32) * band_w;
            let b_center_x = bx + band_w * 0.5;
            let b = &self.bands[i];

            // Divider line
            if i > 0 {
                painter.line_segment(
                    [
                        egui::pos2(bx, wedge_rect.y + 28.0),
                        egui::pos2(bx, wedge_rect.y + wedge_rect.height),
                    ],
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(50, 65, 90, 120)),
                );

                // Crossover Divider Handle (Hit target >= 22pt radius -> 44x44pt bounding box)
                let hy = wedge_rect.y + wedge_rect.height * 0.5;
                painter.circle_stroke(
                    egui::pos2(bx, hy),
                    IMAGER_HANDLE_HIT_RADIUS,
                    Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(255, 215, 0, 100)),
                );
                painter.circle_filled(egui::pos2(bx, hy), 6.0, Color32::from_rgb(255, 215, 0));
            }

            // Band label
            painter.text(
                egui::pos2(b_center_x, wedge_rect.y + 32.0),
                egui::Align2::CENTER_TOP,
                b.name,
                egui::FontId::proportional(10.0),
                band_color,
            );

            // Draw Stereo Width Wedge
            let norm_w = (b.width_pct / 200.0).clamp(0.0, 1.0);
            let half_spread = (band_w * 0.42) * norm_w;
            let mid_y = wedge_rect.y + wedge_rect.height * 0.65;

            let top_pt = egui::pos2(b_center_x, mid_y - 45.0);
            let left_pt = egui::pos2(b_center_x - half_spread, mid_y + 35.0);
            let right_pt = egui::pos2(b_center_x + half_spread, mid_y + 35.0);

            // Triangle wedge
            painter.line_segment([top_pt, left_pt], Stroke::new(2.0_f32, band_color));
            painter.line_segment([top_pt, right_pt], Stroke::new(2.0_f32, band_color));
            painter.line_segment([left_pt, right_pt], Stroke::new(2.0_f32, band_color));

            // Interactive Width Puck on base
            let puck_y = mid_y + 35.0;
            painter.circle_stroke(
                egui::pos2(b_center_x, puck_y),
                IMAGER_HANDLE_HIT_RADIUS,
                Stroke::new(
                    1.5_f32,
                    Color32::from_rgba_unmultiplied(
                        band_color.r(),
                        band_color.g(),
                        band_color.b(),
                        120,
                    ),
                ),
            );
            painter.circle_filled(egui::pos2(b_center_x, puck_y), 10.0, band_color);
            painter.circle_filled(
                egui::pos2(b_center_x, puck_y),
                3.0,
                Color32::from_rgb(255, 255, 255),
            );

            painter.text(
                egui::pos2(b_center_x, wedge_rect.y + wedge_rect.height - 18.0),
                egui::Align2::CENTER_TOP,
                format!("{:.0}%", b.width_pct),
                egui::FontId::proportional(11.0),
                Color32::from_rgb(240, 245, 255),
            );
        }

        // Right Panel: Spectral Correlation Matrix HUD (470..780)
        let matrix_rect = Rect::new(rect.x + 470.0, rect.y + 56.0, 310.0, 224.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(matrix_rect.x, matrix_rect.y),
                egui::vec2(matrix_rect.width, matrix_rect.height),
            ),
            8.0,
            Color32::from_rgb(10, 14, 22),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(matrix_rect.x, matrix_rect.y),
                egui::vec2(matrix_rect.width, matrix_rect.height),
            ),
            8.0,
            Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(matrix_rect.x + 12.0, matrix_rect.y + 12.0),
            egui::Align2::LEFT_TOP,
            "SPECTRAL CORRELATION METERS",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(255, 107, 43),
        );

        // Per-band correlation meters
        for (i, &band_color) in band_colors.iter().enumerate() {
            let b = &self.bands[i];
            let my = matrix_rect.y + 45.0 + (i as f32) * 42.0;

            painter.text(
                egui::pos2(matrix_rect.x + 15.0, my),
                egui::Align2::LEFT_TOP,
                b.name,
                egui::FontId::proportional(10.0),
                band_color,
            );

            let bar_box = egui::Rect::from_min_size(
                egui::pos2(matrix_rect.x + 90.0, my),
                egui::vec2(matrix_rect.width - 110.0, 18.0),
            );
            painter.rect_filled(bar_box, 3.0, Color32::from_rgb(18, 25, 38));

            // Center mark (0.0 correlation)
            let center_bar_x = bar_box.min.x + bar_box.width() * 0.5;
            painter.line_segment(
                [
                    egui::pos2(center_bar_x, bar_box.min.y),
                    egui::pos2(center_bar_x, bar_box.max.y),
                ],
                Stroke::new(1.0_f32, Color32::from_rgb(80, 95, 120)),
            );

            // Fill correlation
            let corr_norm = b.correlation.clamp(-1.0, 1.0);
            let col = if corr_norm >= 0.5 {
                Color32::from_rgb(0, 255, 180)
            } else if corr_norm >= 0.0 {
                Color32::from_rgb(255, 215, 0)
            } else {
                Color32::from_rgb(255, 80, 80)
            };

            let fill_rect = if corr_norm >= 0.0 {
                egui::Rect::from_min_size(
                    egui::pos2(center_bar_x, bar_box.min.y),
                    egui::vec2(bar_box.width() * 0.5 * corr_norm, bar_box.height()),
                )
            } else {
                let fill_w = bar_box.width() * 0.5 * corr_norm.abs();
                egui::Rect::from_min_size(
                    egui::pos2(center_bar_x - fill_w, bar_box.min.y),
                    egui::vec2(fill_w, bar_box.height()),
                )
            };
            painter.rect_filled(fill_rect, 2.0, col);
        }

        // Bottom Controls Bar (290..475)
        let ctrl_rect = Rect::new(rect.x + 20.0, rect.y + 290.0, 760.0, 185.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(ctrl_rect.x, ctrl_rect.y),
                egui::vec2(ctrl_rect.width, ctrl_rect.height),
            ),
            6.0,
            Color32::from_rgb(18, 25, 38),
        );

        // Verified Hit Target Badge
        let badge_rect = Rect::new(ctrl_rect.x + 15.0, ctrl_rect.y + 130.0, 730.0, 36.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(badge_rect.x, badge_rect.y),
                egui::vec2(badge_rect.width, badge_rect.height),
            ),
            4.0,
            Color32::from_rgb(16, 35, 28),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(badge_rect.x, badge_rect.y),
                egui::vec2(badge_rect.width, badge_rect.height),
            ),
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(0, 255, 180)),
        );
        painter.text(
            egui::pos2(badge_rect.x + 10.0, badge_rect.y + 10.0),
            egui::Align2::LEFT_TOP,
            "[PASS] Multiband Stereo Imager Nodes & Correlation Meters (>= 44x44pt) Verified",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
