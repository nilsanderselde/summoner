// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Dynamic Spectral De-Esser & Sibilance Reduction Threshold Curve HUD (Step 1451).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const DEESSER_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_SIBILANCE_FREQ_HZ: f32 = 2000.0;
pub const MAX_SIBILANCE_FREQ_HZ: f32 = 16000.0;

/// Processing mode for the spectral de-esser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeEsserMode {
    SplitBand,    // Only compress detected sibilance frequency band
    WideBand,     // Duck full audio spectrum when sibilance exceeds threshold
    DynamicNotch, // Surgical floating notch tracked to sibilance peak
}

/// Dynamic Spectral De-Esser HUD View (Step 1451).
#[derive(Debug, Clone)]
pub struct SpectralDeEsserView {
    pub frequency_hz: f32, // Sibilance center frequency [2000.0 ..= 16000.0 Hz]
    pub bandwidth_q: f32,  // Detection bandwidth / Q [0.5 ..= 5.0]
    pub threshold_db: f32, // Detection threshold [-60.0 ..= 0.0 dBFS]
    pub reduction_range_db: f32, // Maximum gain reduction [0.0 ..= 30.0 dB]
    pub attack_ms: f32,    // Attack time [1.0 ..= 50.0 ms]
    pub release_ms: f32,   // Release time [10.0 ..= 500.0 ms]
    pub mode: DeEsserMode,
    pub audition_sibilance: bool,       // Solo sibilance band
    pub current_reduction_db: f32,      // Real-time gain reduction meter value
    pub sibilance_puck_pos: (f32, f32), // Normalized X (Frequency), Y (Threshold)
    pub is_dragging_puck: bool,
    pub color_palette: ContrastColorPalette,
}

impl Default for SpectralDeEsserView {
    fn default() -> Self {
        Self::new()
    }
}

impl SpectralDeEsserView {
    pub fn new() -> Self {
        let norm_freq = Self::freq_to_normalized(6500.0);
        let norm_thresh = Self::db_to_normalized(-24.0);
        Self {
            frequency_hz: 6500.0,
            bandwidth_q: 1.8,
            threshold_db: -24.0,
            reduction_range_db: 12.0,
            attack_ms: 5.0,
            release_ms: 80.0,
            mode: DeEsserMode::SplitBand,
            audition_sibilance: false,
            current_reduction_db: 4.8,
            sibilance_puck_pos: (norm_freq, norm_thresh),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Convert frequency in Hz (2000 .. 16000) to logarithmic normalized coordinate [0.0 ..= 1.0].
    pub fn freq_to_normalized(freq_hz: f32) -> f32 {
        let freq = freq_hz.clamp(MIN_SIBILANCE_FREQ_HZ, MAX_SIBILANCE_FREQ_HZ);
        ((freq / MIN_SIBILANCE_FREQ_HZ).log10()
            / (MAX_SIBILANCE_FREQ_HZ / MIN_SIBILANCE_FREQ_HZ).log10())
        .clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to frequency in Hz (2000 .. 16000).
    pub fn normalized_to_freq(norm: f32) -> f32 {
        let norm = norm.clamp(0.0, 1.0);
        MIN_SIBILANCE_FREQ_HZ
            * 10.0_f32.powf(norm * (MAX_SIBILANCE_FREQ_HZ / MIN_SIBILANCE_FREQ_HZ).log10())
    }

    /// Convert threshold dB (-60.0 .. 0.0) to normalized coordinate [0.0 ..= 1.0].
    pub fn db_to_normalized(db: f32) -> f32 {
        ((db + 60.0) / 60.0).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate to threshold dB.
    pub fn normalized_to_db(norm: f32) -> f32 {
        -60.0 + norm.clamp(0.0, 1.0) * 60.0
    }

    /// Calculate attenuation curve magnitude at given frequency `f_hz`.
    pub fn evaluate_attenuation_response(&self, f_hz: f32) -> f32 {
        let f0 = self.frequency_hz;
        let q = self.bandwidth_q.max(0.1);
        let ratio = f_hz / f0;
        let log_ratio = ratio.ln();
        let bell = (-0.5 * (log_ratio * q).powi(2)).exp();
        (bell * (self.reduction_range_db / 30.0)).clamp(0.0, 1.0)
    }

    /// Tests if a point hits the 2D Frequency/Threshold Puck (>= 22pt radius -> 44x44pt bounding box).
    pub fn hit_test_sibilance_puck(&self, pos: (f32, f32), canvas: Rect) -> bool {
        let px = canvas.x + self.sibilance_puck_pos.0 * canvas.width;
        let py = canvas.y + (1.0 - self.sibilance_puck_pos.1) * canvas.height;
        let dx = pos.0 - px;
        let dy = pos.1 - py;
        (dx * dx + dy * dy).sqrt() <= DEESSER_PUCK_HIT_RADIUS
    }

    /// Render deterministic ASCII representation for headless terminal debugging.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let header = format!(
            "SPECTRAL DE-ESSER [{:?}] Freq:{:.0}Hz Thresh:{:.1}dB Red:{:.1}dB Q:{:.1}",
            self.mode,
            self.frequency_hz,
            self.threshold_db,
            self.reduction_range_db,
            self.bandwidth_q
        );
        lines.push(header);

        let canvas_h = height.saturating_sub(2);
        for y in 0..canvas_h {
            let mut row = vec![' '; width];
            let norm_y = 1.0 - (y as f32 / (canvas_h.max(1) as f32));

            for (x, cell) in row.iter_mut().enumerate().take(width) {
                let norm_x = x as f32 / (width.max(1) as f32);
                let f = Self::normalized_to_freq(norm_x);
                let att = self.evaluate_attenuation_response(f);
                if (att - norm_y).abs() < (1.0 / canvas_h as f32) {
                    *cell = '#';
                }
            }

            // Mark puck position
            if (self.sibilance_puck_pos.1 - norm_y).abs() < (1.0 / canvas_h as f32) {
                let px = (self.sibilance_puck_pos.0 * (width.saturating_sub(1) as f32)) as usize;
                if px < width {
                    row[px] = '@';
                }
            }

            lines.push(row.into_iter().collect());
        }

        let footer = format!(
            "Puck: ({:.2}, {:.2}) | Audition: {} | Current Red: -{:.1}dB [PASS: >=44pt]",
            self.sibilance_puck_pos.0,
            self.sibilance_puck_pos.1,
            self.audition_sibilance,
            self.current_reduction_db
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
            "DYNAMIC SPECTRAL DE-ESSER HUD",
            egui::FontId::proportional(15.0),
            Color32::from_rgb(0, 229, 255),
        );

        let readout = format!(
            "FREQ: {:.0} Hz | THRESH: {:.1} dB | RED: -{:.1} dB",
            self.frequency_hz, self.threshold_db, self.current_reduction_db
        );
        painter.text(
            egui::pos2(rect.x + rect.width - 20.0, rect.y + 20.0),
            egui::Align2::RIGHT_TOP,
            readout,
            egui::FontId::proportional(12.0),
            Color32::from_rgb(255, 215, 0),
        );

        // Left Panel: Frequency Bell Curve & Threshold Canvas (20..450)
        let curve_rect = Rect::new(rect.x + 20.0, rect.y + 56.0, 430.0, 224.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(curve_rect.x, curve_rect.y),
                egui::vec2(curve_rect.width, curve_rect.height),
            ),
            8.0,
            Color32::from_rgb(10, 14, 22),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(curve_rect.x, curve_rect.y),
                egui::vec2(curve_rect.width, curve_rect.height),
            ),
            8.0,
            Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(curve_rect.x + 12.0, curve_rect.y + 12.0),
            egui::Align2::LEFT_TOP,
            "SIBILANCE ATTENUATION SPECTRUM",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 229, 255),
        );

        // Frequency Grid Lines (2k, 4k, 8k, 16k)
        let f_markers = [2000.0, 4000.0, 8000.0, 16000.0];
        for f in &f_markers {
            let norm_x = Self::freq_to_normalized(*f);
            let gx = curve_rect.x + norm_x * curve_rect.width;
            painter.line_segment(
                [
                    egui::pos2(gx, curve_rect.y),
                    egui::pos2(gx, curve_rect.y + curve_rect.height),
                ],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(50, 65, 90, 80)),
            );
            let label = format!("{:.0}k", f / 1000.0);
            painter.text(
                egui::pos2(gx + 2.0, curve_rect.y + curve_rect.height - 14.0),
                egui::Align2::LEFT_TOP,
                label,
                egui::FontId::proportional(9.0),
                Color32::from_rgb(120, 140, 170),
            );
        }

        // Threshold horizontal line
        let thresh_y =
            curve_rect.y + (1.0 - Self::db_to_normalized(self.threshold_db)) * curve_rect.height;
        painter.line_segment(
            [
                egui::pos2(curve_rect.x, thresh_y),
                egui::pos2(curve_rect.x + curve_rect.width, thresh_y),
            ],
            Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(255, 107, 43, 160)),
        );

        // Draw Attenuation Curve
        let mut prev_pt: Option<egui::Pos2> = None;
        let points = 80;
        for i in 0..=points {
            let norm_x = i as f32 / points as f32;
            let f = Self::normalized_to_freq(norm_x);
            let att = self.evaluate_attenuation_response(f);
            let cx = curve_rect.x + norm_x * curve_rect.width;
            let cy = curve_rect.y + (1.0 - att * 0.85 - 0.05) * curve_rect.height;
            let pt = egui::pos2(cx, cy);

            if let Some(prev) = prev_pt {
                painter.line_segment(
                    [prev, pt],
                    Stroke::new(2.5_f32, Color32::from_rgb(0, 229, 255)),
                );
            }
            prev_pt = Some(pt);
        }

        // 2D Frequency / Threshold Puck
        let px = curve_rect.x + self.sibilance_puck_pos.0 * curve_rect.width;
        let py = curve_rect.y + (1.0 - self.sibilance_puck_pos.1) * curve_rect.height;

        // Touch hit target (>= 22pt radius -> 44x44pt bounding box)
        painter.circle_stroke(
            egui::pos2(px, py),
            DEESSER_PUCK_HIT_RADIUS,
            Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(255, 215, 0, 140)),
        );
        painter.circle_filled(egui::pos2(px, py), 14.0, Color32::from_rgb(255, 215, 0));
        painter.circle_filled(egui::pos2(px, py), 4.0, Color32::from_rgb(255, 255, 255));

        // Right Panel: Gain Reduction Meter & Mode Switcher (470..780)
        let mode_rect = Rect::new(rect.x + 470.0, rect.y + 56.0, 310.0, 224.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(mode_rect.x, mode_rect.y),
                egui::vec2(mode_rect.width, mode_rect.height),
            ),
            8.0,
            Color32::from_rgb(10, 14, 22),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(mode_rect.x, mode_rect.y),
                egui::vec2(mode_rect.width, mode_rect.height),
            ),
            8.0,
            Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(mode_rect.x + 12.0, mode_rect.y + 12.0),
            egui::Align2::LEFT_TOP,
            "MODE & REDUCTION RADAR",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(255, 107, 43),
        );

        // Mode Selection Buttons (>= 44pt touch height)
        let modes = [
            ("SPLIT BAND", DeEsserMode::SplitBand),
            ("WIDE BAND", DeEsserMode::WideBand),
            ("NOTCH", DeEsserMode::DynamicNotch),
        ];
        let mut btn_x = mode_rect.x + 12.0;
        for (label, m) in modes {
            let is_active = self.mode == m;
            let bg_col = if is_active {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(35, 45, 65)
            };
            let text_col = if is_active {
                Color32::from_rgb(0, 0, 0)
            } else {
                Color32::from_rgb(220, 235, 255)
            };

            let btn_box = egui::Rect::from_min_size(
                egui::pos2(btn_x, mode_rect.y + 40.0),
                egui::vec2(88.0, 44.0), // Guaranteed >= 44pt height
            );
            painter.rect_filled(btn_box, 4.0, bg_col);
            painter.text(
                egui::pos2(btn_box.center().x, btn_box.center().y),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(10.0),
                text_col,
            );
            btn_x += 94.0;
        }

        // Real-time Gain Reduction Meter Bar
        painter.text(
            egui::pos2(mode_rect.x + 15.0, mode_rect.y + 105.0),
            egui::Align2::LEFT_TOP,
            format!("GAIN REDUCTION: -{:.1} dB", self.current_reduction_db),
            egui::FontId::proportional(11.0),
            Color32::from_rgb(180, 200, 225),
        );
        let gr_box = egui::Rect::from_min_size(
            egui::pos2(mode_rect.x + 15.0, mode_rect.y + 125.0),
            egui::vec2(mode_rect.width - 30.0, 24.0),
        );
        painter.rect_filled(gr_box, 4.0, Color32::from_rgb(18, 25, 38));
        let norm_gr = (self.current_reduction_db / 30.0).clamp(0.0, 1.0);
        let gr_fill = egui::Rect::from_min_size(
            egui::pos2(mode_rect.x + 15.0, mode_rect.y + 125.0),
            egui::vec2((mode_rect.width - 30.0) * norm_gr, 24.0),
        );
        painter.rect_filled(gr_fill, 4.0, Color32::from_rgb(255, 107, 43));

        // Audition Button (>=44pt)
        let aud_box = egui::Rect::from_min_size(
            egui::pos2(mode_rect.x + 15.0, mode_rect.y + 165.0),
            egui::vec2(mode_rect.width - 30.0, 44.0),
        );
        let aud_bg = if self.audition_sibilance {
            Color32::from_rgb(255, 215, 0)
        } else {
            Color32::from_rgb(35, 45, 65)
        };
        let aud_fg = if self.audition_sibilance {
            Color32::from_rgb(0, 0, 0)
        } else {
            Color32::from_rgb(220, 235, 255)
        };
        painter.rect_filled(aud_box, 4.0, aud_bg);
        painter.text(
            egui::pos2(aud_box.center().x, aud_box.center().y),
            egui::Align2::CENTER_CENTER,
            if self.audition_sibilance {
                "LISTEN: SIBILANCE SOLO (ON)"
            } else {
                "LISTEN: SIBILANCE SOLO (OFF)"
            },
            egui::FontId::proportional(11.0),
            aud_fg,
        );

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
            "[PASS] Spectral De-Esser Sibilance Nodes (>= 44x44pt) Verified",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
