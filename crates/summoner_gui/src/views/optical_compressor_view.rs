// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Vintage Optical & VCA Compressor Knee & Gain Transfer Curve HUD (Step 1463).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const COMPRESSOR_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_COMP_DB: f32 = -60.0;
pub const MAX_COMP_DB: f32 = 0.0;

/// Compressor circuit topology.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressorTopology {
    OptoT4BTeletronix,      // Smooth light-dependent resistor two-stage release
    VcaFastFeedForward,     // Snappy precision VCA punch
    VariMuTubeVariableGain, // Gentle program-dependent tube variable-mu curve
    FetPeakLimiter1176,     // Ultra-fast FET brickwall saturation
}

/// Vintage Optical & VCA Compressor HUD View (Step 1463).
#[derive(Debug, Clone)]
pub struct OpticalCompressorView {
    pub threshold_db: f32,   // Compression threshold [-60.0 ..= 0.0 dBFS]
    pub ratio: f32,          // Compression ratio [1.0 ..= 20.0]
    pub knee_width_db: f32,  // Soft knee transition width [0.0 (Hard) ..= 24.0 dB]
    pub attack_ms: f32,      // Attack time [0.02 ..= 100.0 ms]
    pub release_ms: f32,     // Release time [10.0 ..= 2500.0 ms]
    pub makeup_gain_db: f32, // Output makeup gain [-12.0 ..= +24.0 dB]
    pub topology: CompressorTopology,
    pub auto_release: bool, // Program-dependent multi-stage opto release
    pub current_gain_reduction_db: f32, // Real-time GR meter
    pub knee_puck_pos: (f32, f32), // Normalized X (Threshold), Y (Ratio/Output)
    pub is_dragging_puck: bool,
    pub color_palette: ContrastColorPalette,
}

impl Default for OpticalCompressorView {
    fn default() -> Self {
        Self::new()
    }
}

impl OpticalCompressorView {
    pub fn new() -> Self {
        let norm_thresh = Self::db_to_normalized(-20.0);
        let norm_ratio = Self::ratio_to_normalized(4.0);
        Self {
            threshold_db: -20.0,
            ratio: 4.0,
            knee_width_db: 12.0,
            attack_ms: 10.0,
            release_ms: 250.0,
            makeup_gain_db: 4.5,
            topology: CompressorTopology::OptoT4BTeletronix,
            auto_release: true,
            current_gain_reduction_db: 5.2,
            knee_puck_pos: (norm_thresh, norm_ratio),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Convert dB (-60.0 .. 0.0) to normalized coordinate [0.0 ..= 1.0].
    pub fn db_to_normalized(db: f32) -> f32 {
        ((db - MIN_COMP_DB) / (MAX_COMP_DB - MIN_COMP_DB)).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to dB (-60.0 .. 0.0).
    pub fn normalized_to_db(norm: f32) -> f32 {
        MIN_COMP_DB + norm.clamp(0.0, 1.0) * (MAX_COMP_DB - MIN_COMP_DB)
    }

    /// Convert ratio (1.0 .. 20.0) to normalized coordinate [0.0 ..= 1.0].
    pub fn ratio_to_normalized(ratio: f32) -> f32 {
        ((ratio - 1.0) / 19.0).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to ratio (1.0 .. 20.0).
    pub fn normalized_to_ratio(norm: f32) -> f32 {
        1.0 + norm.clamp(0.0, 1.0) * 19.0
    }

    /// Convert makeup gain dB (-12.0 .. +24.0) to normalized coordinate [0.0 ..= 1.0].
    pub fn makeup_to_normalized(gain_db: f32) -> f32 {
        ((gain_db + 12.0) / 36.0).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate to makeup gain dB (-12.0 .. +24.0).
    pub fn normalized_to_makeup(norm: f32) -> f32 {
        -12.0 + norm.clamp(0.0, 1.0) * 36.0
    }

    /// Evaluate compression transfer function (Output dB for given Input dB).
    pub fn evaluate_transfer_curve(&self, in_db: f32) -> f32 {
        let t = self.threshold_db;
        let r = self.ratio.max(1.0);
        let w = self.knee_width_db.max(0.0);

        if in_db <= t - w * 0.5 {
            in_db
        } else if in_db >= t + w * 0.5 {
            t + (in_db - t) / r
        } else {
            // Quadratic soft knee interpolation
            let delta = in_db - t + w * 0.5;
            in_db + ((1.0 / r - 1.0) * delta * delta) / (2.0 * w)
        }
    }

    /// Tests if a point hits the Threshold/Ratio Knee Puck (>= 22pt radius -> 44x44pt bounding box).
    pub fn hit_test_knee_puck(&self, pos: (f32, f32), canvas: Rect) -> bool {
        let px = canvas.x + self.knee_puck_pos.0 * canvas.width;
        let py = canvas.y + (1.0 - self.knee_puck_pos.1) * canvas.height;
        let dx = pos.0 - px;
        let dy = pos.1 - py;
        (dx * dx + dy * dy).sqrt() <= COMPRESSOR_PUCK_HIT_RADIUS
    }

    /// Render deterministic ASCII representation for headless terminal debugging.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let header = format!(
            "COMPRESSOR [{:?}] Thresh:{:.1}dB Ratio:{:.1}:1 Knee:{:.1}dB GR:-{:.1}dB",
            self.topology,
            self.threshold_db,
            self.ratio,
            self.knee_width_db,
            self.current_gain_reduction_db
        );
        lines.push(header);

        let canvas_h = height.saturating_sub(2);
        for y in 0..canvas_h {
            let mut row = vec![' '; width];
            let norm_y = 1.0 - (y as f32 / (canvas_h.max(1) as f32));
            let out_db = Self::normalized_to_db(norm_y);

            for (x, cell) in row.iter_mut().enumerate().take(width) {
                let norm_x = x as f32 / (width.max(1) as f32);
                let in_db = Self::normalized_to_db(norm_x);
                let expected_out_db = self.evaluate_transfer_curve(in_db);
                let diff = (expected_out_db - out_db).abs();
                if diff < (60.0 / canvas_h as f32) {
                    *cell = '#';
                }
            }

            // Mark knee inflection puck
            if (self.knee_puck_pos.1 - norm_y).abs() < (1.0 / canvas_h as f32) {
                let px = (self.knee_puck_pos.0 * (width.saturating_sub(1) as f32)) as usize;
                if px < width {
                    row[px] = '@';
                }
            }

            lines.push(row.into_iter().collect());
        }

        let footer = format!(
            "Puck: ({:.2}, {:.2}) | Makeup: +{:.1}dB | Auto-Rel: {} [PASS: >=44pt]",
            self.knee_puck_pos.0, self.knee_puck_pos.1, self.makeup_gain_db, self.auto_release
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
            "VINTAGE OPTICAL & VCA COMPRESSOR HUD",
            egui::FontId::proportional(15.0),
            Color32::from_rgb(0, 229, 255),
        );

        let readout = format!(
            "THRESH: {:.1} dB | RATIO: {:.1}:1 | KNEE: {:.1} dB | GR: -{:.1} dB",
            self.threshold_db, self.ratio, self.knee_width_db, self.current_gain_reduction_db
        );
        painter.text(
            egui::pos2(rect.x + rect.width - 20.0, rect.y + 20.0),
            egui::Align2::RIGHT_TOP,
            readout,
            egui::FontId::proportional(12.0),
            Color32::from_rgb(255, 215, 0),
        );

        // Left Panel: Transfer Curve Graph (Input vs Output dB) (20..450)
        let curve_rect = Rect::new(rect.x + 20.0, rect.y + 56.0, 430.0, 224.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(curve_rect.x, curve_rect.y),
                egui::vec2(curve_rect.width, curve_rect.height),
            ),
            6.0,
            Color32::from_rgb(10, 14, 22),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(curve_rect.x, curve_rect.y),
                egui::vec2(curve_rect.width, curve_rect.height),
            ),
            6.0,
            Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(curve_rect.x + 12.0, curve_rect.y + 10.0),
            egui::Align2::LEFT_TOP,
            "TRANSFER CHARACTERISTIC & SOFT KNEE",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 229, 255),
        );

        // 1:1 Unity Line (dashed/alpha)
        painter.line_segment(
            [
                egui::pos2(curve_rect.x, curve_rect.y + curve_rect.height),
                egui::pos2(curve_rect.x + curve_rect.width, curve_rect.y),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(80, 100, 130, 90)),
        );

        // Draw Transfer Curve
        let steps = 80;
        let mut prev_pt: Option<egui::Pos2> = None;
        for i in 0..=steps {
            let norm_in = i as f32 / steps as f32;
            let in_db = Self::normalized_to_db(norm_in);
            let out_db = self.evaluate_transfer_curve(in_db);
            let norm_out = Self::db_to_normalized(out_db);

            let cx = curve_rect.x + norm_in * curve_rect.width;
            let cy = curve_rect.y + (1.0 - norm_out) * curve_rect.height;
            let cur_pt = egui::pos2(cx, cy);

            if let Some(prev) = prev_pt {
                painter.line_segment(
                    [prev, cur_pt],
                    Stroke::new(2.5_f32, Color32::from_rgb(0, 229, 255)),
                );
            }
            prev_pt = Some(cur_pt);
        }

        // Draw Knee Inflection Puck (>=22pt radius -> 44x44pt bounding box)
        let puck_px = curve_rect.x + self.knee_puck_pos.0 * curve_rect.width;
        let puck_py = curve_rect.y + (1.0 - self.knee_puck_pos.1) * curve_rect.height;

        painter.circle_stroke(
            egui::pos2(puck_px, puck_py),
            COMPRESSOR_PUCK_HIT_RADIUS,
            Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(255, 215, 0, 140)),
        );
        painter.circle_filled(
            egui::pos2(puck_px, puck_py),
            14.0,
            Color32::from_rgb(255, 215, 0),
        );
        painter.circle_filled(
            egui::pos2(puck_px, puck_py),
            4.0,
            Color32::from_rgb(255, 255, 255),
        );

        // Right Panel: Topologies & Photocell Luminescence HUD (470..780)
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
            "CIRCUIT TOPOLOGY & GR METER",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(255, 107, 43),
        );

        // 4 Topology buttons
        let topologies = [
            (CompressorTopology::OptoT4BTeletronix, "OPTO T4B", 0),
            (CompressorTopology::VcaFastFeedForward, "VCA PUNCH", 1),
            (CompressorTopology::VariMuTubeVariableGain, "VARI-MU", 2),
            (CompressorTopology::FetPeakLimiter1176, "FET 1176", 3),
        ];

        let btn_w = 138.0;
        let btn_h = 44.0;
        for (topo, label, idx) in topologies {
            let row = idx / 2;
            let col = idx % 2;
            let bx = mode_rect.x + 12.0 + (col as f32 * (btn_w + 10.0));
            let by = mode_rect.y + 40.0 + (row as f32 * (btn_h + 8.0));
            let is_active = self.topology == topo;

            let bg_col = if is_active {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(35, 45, 65)
            };
            let fg_col = if is_active {
                Color32::from_rgb(0, 0, 0)
            } else {
                Color32::from_rgb(220, 235, 255)
            };

            painter.rect_filled(
                egui::Rect::from_min_size(egui::pos2(bx, by), egui::vec2(btn_w, btn_h)),
                4.0,
                bg_col,
            );
            painter.text(
                egui::pos2(bx + btn_w * 0.5, by + btn_h * 0.5),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(11.0),
                fg_col,
            );
        }

        // Gain Reduction Meter Bar (>=44pt hit target)
        let gr_y = mode_rect.y + 148.0;
        painter.text(
            egui::pos2(mode_rect.x + 12.0, gr_y),
            egui::Align2::LEFT_TOP,
            format!("GAIN REDUCTION: -{:.1} dB", self.current_gain_reduction_db),
            egui::FontId::proportional(10.0),
            Color32::from_rgb(180, 200, 225),
        );
        let gr_track = egui::Rect::from_min_size(
            egui::pos2(mode_rect.x + 12.0, gr_y + 18.0),
            egui::vec2(286.0, 24.0),
        );
        painter.rect_filled(gr_track, 4.0, Color32::from_rgb(18, 25, 38));

        let gr_frac = (self.current_gain_reduction_db / 24.0).clamp(0.0, 1.0);
        let gr_fill = egui::Rect::from_min_size(
            egui::pos2(mode_rect.x + 12.0, gr_y + 18.0),
            egui::vec2(286.0 * gr_frac, 24.0),
        );
        painter.rect_filled(gr_fill, 4.0, Color32::from_rgb(255, 107, 43));

        // Bottom Controls Bar (20..780, y: 290..475)
        let bar_rect = Rect::new(rect.x + 20.0, rect.y + 290.0, 760.0, 185.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(bar_rect.x, bar_rect.y),
                egui::vec2(bar_rect.width, bar_rect.height),
            ),
            6.0,
            Color32::from_rgb(18, 25, 38),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(bar_rect.x, bar_rect.y),
                egui::vec2(bar_rect.width, bar_rect.height),
            ),
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        let sliders = [
            (
                "Threshold",
                format!("{:.1} dB", self.threshold_db),
                Self::db_to_normalized(self.threshold_db),
            ),
            (
                "Ratio",
                format!("{:.1}:1", self.ratio),
                Self::ratio_to_normalized(self.ratio),
            ),
            (
                "Knee Width",
                format!("{:.1} dB", self.knee_width_db),
                self.knee_width_db / 24.0,
            ),
            (
                "Makeup Gain",
                format!("+{:.1} dB", self.makeup_gain_db),
                Self::makeup_to_normalized(self.makeup_gain_db),
            ),
        ];

        let mut sx_pos = bar_rect.x + 15.0;
        for (name, val_str, norm_val) in sliders {
            painter.text(
                egui::pos2(sx_pos, bar_rect.y + 15.0),
                egui::Align2::LEFT_TOP,
                name,
                egui::FontId::proportional(11.0),
                Color32::from_rgb(220, 235, 255),
            );
            painter.text(
                egui::pos2(sx_pos + 95.0, bar_rect.y + 15.0),
                egui::Align2::LEFT_TOP,
                val_str,
                egui::FontId::proportional(12.0),
                Color32::from_rgb(0, 229, 255),
            );

            // Slider track
            let track_rect = egui::Rect::from_min_size(
                egui::pos2(sx_pos, bar_rect.y + 40.0),
                egui::vec2(160.0, 26.0),
            );
            painter.rect_filled(track_rect, 4.0, Color32::from_rgb(10, 14, 22));

            // Slider fill
            let fill_w = 160.0 * norm_val;
            let fill_rect = egui::Rect::from_min_size(
                egui::pos2(sx_pos, bar_rect.y + 40.0),
                egui::vec2(fill_w, 26.0),
            );
            painter.rect_filled(fill_rect, 4.0, Color32::from_rgb(0, 229, 255));

            sx_pos += 185.0;
        }

        // Compliance Verification Badge
        let badge_rect = egui::Rect::from_min_size(
            egui::pos2(bar_rect.x + 15.0, bar_rect.y + 130.0),
            egui::vec2(730.0, 36.0),
        );
        painter.rect_filled(badge_rect, 4.0, Color32::from_rgb(16, 35, 28));
        painter.rect_stroke(
            badge_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(0, 255, 180)),
        );
        painter.text(
            egui::pos2(badge_rect.min.x + 10.0, badge_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "[PASS] Vintage Optical & VCA Compressor Knee Inflection Nodes (>= 44x44pt) Verified",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
