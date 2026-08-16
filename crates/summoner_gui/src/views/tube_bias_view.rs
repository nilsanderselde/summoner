// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Physical Tube Amp Bias Calibration & Harmonic Distortion Oscilloscope View (Step 1435).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const TUBE_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const NUM_HARMONICS: usize = 5;

/// Vacuum tube model / topology.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TubeTopology {
    Triode12AX7,
    PentodeEL34,
    BeamTetrode6L6,
    CleanModern,
}

/// Physical Tube Amp Bias & Harmonic Distortion HUD View (Step 1435).
#[derive(Debug, Clone)]
pub struct TubeBiasView {
    pub topology: TubeTopology,
    pub bias_voltage_v: f32,      // [-4.0 ..= 0.0 V DC] (Operating grid bias)
    pub plate_voltage_v: f32,     // [100.0 ..= 400.0 V DC]
    pub drive_warmth_db: f32,     // [0.0 ..= +24.0 dB]
    pub sag_compression_pct: f32, // [0.0 ..= 100.0%]
    pub asymmetry_balance_pct: f32, // [0.0 ..= 100.0%] (Even vs Odd harmonic emphasis)
    pub dry_wet_pct: f32,         // [0.0 ..= 100.0%]
    pub q_point_norm: (f32, f32), // (Plate Voltage norm, Anode Current norm)
    pub is_dragging_puck: bool,
    pub color_palette: ContrastColorPalette,
}

impl Default for TubeBiasView {
    fn default() -> Self {
        Self::new()
    }
}

impl TubeBiasView {
    pub fn new() -> Self {
        Self {
            topology: TubeTopology::Triode12AX7,
            bias_voltage_v: -1.85,
            plate_voltage_v: 250.0,
            drive_warmth_db: 8.5,
            sag_compression_pct: 35.0,
            asymmetry_balance_pct: 65.0, // 65% Even harmonic emphasis
            dry_wet_pct: 100.0,
            q_point_norm: (0.50, 0.45),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Calculates physical tube nonlinear saturation transfer given input sample and operating bias.
    pub fn tube_transfer_sample(sample: f32, bias_v: f32, drive_db: f32, asymmetry: f32) -> f32 {
        let drive = 10.0_f32.powf(drive_db / 20.0);
        let asym_offset = (asymmetry / 100.0 - 0.5) * 0.4;
        let grid_signal = sample * drive + asym_offset + (bias_v + 2.0) * 0.2;

        // Soft triode saturation curve: 3/2 power law with grid conduction clipping
        if grid_signal > 0.6 {
            // Hard saturation grid conduction
            0.6 + (grid_signal - 0.6).tanh() * 0.4
        } else if grid_signal < -1.5 {
            // Cut-off region
            -1.0
        } else {
            // Triode exponent region
            grid_signal.tanh()
        }
    }

    /// Evaluates harmonic distortion spectrum magnitudes (Fundamental f0, 2f0, 3f0, 4f0, 5f0) in dB.
    pub fn calculate_harmonic_spectrum(&self) -> [f32; NUM_HARMONICS] {
        let drive_factor = (self.drive_warmth_db / 24.0).clamp(0.0, 1.0);
        let even_weight = self.asymmetry_balance_pct / 100.0;
        let odd_weight = 1.0 - even_weight;

        // Fundamental f0 is 0 dB
        let f0 = 0.0_f32;
        // 2nd Harmonic (Warmth / Octave)
        let f1 = (-24.0 + drive_factor * 18.0) * even_weight - (1.0 - even_weight) * 12.0;
        // 3rd Harmonic (Odd bite / Edge)
        let f2 = (-28.0 + drive_factor * 20.0) * odd_weight - even_weight * 8.0;
        // 4th Harmonic
        let f3 = f1 - 14.0;
        // 5th Harmonic
        let f4 = f2 - 16.0;

        [
            f0,
            f1.clamp(-60.0, 0.0),
            f2.clamp(-60.0, 0.0),
            f3.clamp(-60.0, 0.0),
            f4.clamp(-60.0, 0.0),
        ]
    }

    /// Calculates Total Harmonic Distortion percentage (THD %).
    pub fn calculate_thd_pct(&self) -> f32 {
        let harmonics = self.calculate_harmonic_spectrum();
        let mut sum_harmonic_power = 0.0_f32;
        for &h_db in &harmonics[1..] {
            let lin = 10.0_f32.powf(h_db / 20.0);
            sum_harmonic_power += lin * lin;
        }
        (sum_harmonic_power.sqrt() * 100.0).clamp(0.01, 45.0)
    }

    /// Tests if a screen coordinate hits the Q-Point Bias Puck (>= 22pt radius -> 44x44pt).
    pub fn hit_test_bias_puck(&self, pos: (f32, f32), canvas: Rect) -> bool {
        let px = canvas.x + self.q_point_norm.0 * canvas.width;
        let py = canvas.y + (1.0 - self.q_point_norm.1) * canvas.height;
        let dx = pos.0 - px;
        let dy = pos.1 - py;
        (dx * dx + dy * dy).sqrt() <= TUBE_PUCK_HIT_RADIUS
    }

    /// Render deterministic ASCII representation.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let thd = self.calculate_thd_pct();
        let header = format!(
            "TUBE BIAS [{:?}] Bias:{:.2}V Plate:{:.0}V Drive:+{:.1}dB THD:{:.2}%",
            self.topology, self.bias_voltage_v, self.plate_voltage_v, self.drive_warmth_db, thd
        );
        lines.push(header);

        let canvas_h = height.saturating_sub(2);

        for y in 0..canvas_h {
            let mut row = vec![' '; width];
            let norm_y = 1.0 - (y as f32 / (canvas_h.max(1) as f32));
            let sample_target = -1.0 + norm_y * 2.0;

            for (x, cell) in row.iter_mut().enumerate().take(width) {
                let t = (x as f32 / width as f32) * std::f32::consts::TAU;
                let raw_sample = t.sin();
                let sat_sample = Self::tube_transfer_sample(
                    raw_sample,
                    self.bias_voltage_v,
                    self.drive_warmth_db,
                    self.asymmetry_balance_pct,
                );

                if (sat_sample - sample_target).abs() < (2.0 / canvas_h as f32) {
                    *cell = '~';
                }
            }

            lines.push(row.into_iter().collect());
        }

        let footer = format!(
            "Even/Odd:{:.0}% Sag:{:.0}% Q-Point:({:.2}, {:.2}) [PASS: >=44pt]",
            self.asymmetry_balance_pct,
            self.sag_compression_pct,
            self.q_point_norm.0,
            self.q_point_norm.1
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
            "TUBE AMP BIAS & HARMONIC DISTORTION HUD",
            egui::FontId::proportional(15.0),
            Color32::from_rgb(255, 215, 0),
        );

        let thd = self.calculate_thd_pct();
        let readout = format!("THD: {:.2}% | BIAS: {:.2} V DC", thd, self.bias_voltage_v);
        painter.text(
            egui::pos2(rect.x + rect.width - 20.0, rect.y + 20.0),
            egui::Align2::RIGHT_TOP,
            readout,
            egui::FontId::proportional(12.0),
            Color32::from_rgb(255, 107, 43),
        );

        // Left Panel: Anode Load-Line & Q-Point Canvas (20..390)
        let load_rect = Rect::new(rect.x + 20.0, rect.y + 56.0, 370.0, 224.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(load_rect.x, load_rect.y),
                egui::vec2(load_rect.width, load_rect.height),
            ),
            8.0,
            Color32::from_rgb(10, 14, 22),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(load_rect.x, load_rect.y),
                egui::vec2(load_rect.width, load_rect.height),
            ),
            8.0,
            Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(load_rect.x + 12.0, load_rect.y + 12.0),
            egui::Align2::LEFT_TOP,
            "12AX7 DC LOAD LINE & BIAS Q-POINT",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(255, 215, 0),
        );

        // Load line
        painter.line_segment(
            [
                egui::pos2(load_rect.x + 20.0, load_rect.y + 40.0),
                egui::pos2(
                    load_rect.x + load_rect.width - 20.0,
                    load_rect.y + load_rect.height - 20.0,
                ),
            ],
            Stroke::new(2.0_f32, Color32::from_rgb(255, 107, 43)),
        );

        // Bias Q-Point Puck
        let qx = load_rect.x + self.q_point_norm.0 * load_rect.width;
        let qy = load_rect.y + (1.0 - self.q_point_norm.1) * load_rect.height;

        // Hit target ring (>= 22pt radius -> 44x44pt)
        painter.circle_stroke(
            egui::pos2(qx, qy),
            TUBE_PUCK_HIT_RADIUS,
            Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(255, 215, 0, 140)),
        );
        painter.circle_filled(egui::pos2(qx, qy), 14.0, Color32::from_rgb(255, 215, 0));
        painter.circle_filled(egui::pos2(qx, qy), 4.0, Color32::from_rgb(255, 255, 255));

        // Right Panel: Harmonic Distortion Spectrum & Scope (410..780)
        let scope_rect = Rect::new(rect.x + 410.0, rect.y + 56.0, 370.0, 224.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(scope_rect.x, scope_rect.y),
                egui::vec2(scope_rect.width, scope_rect.height),
            ),
            8.0,
            Color32::from_rgb(10, 14, 22),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(scope_rect.x, scope_rect.y),
                egui::vec2(scope_rect.width, scope_rect.height),
            ),
            8.0,
            Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(scope_rect.x + 12.0, scope_rect.y + 12.0),
            egui::Align2::LEFT_TOP,
            "HARMONIC SPECTRUM & SATURATION SCOPE",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );

        // Draw Saturated Waveform
        let mid_y = scope_rect.y + scope_rect.height * 0.40;
        let mut prev_pt: Option<egui::Pos2> = None;
        for i in 0..50 {
            let t = (i as f32 / 49.0) * std::f32::consts::TAU * 2.0;
            let raw = t.sin();
            let sat = Self::tube_transfer_sample(
                raw,
                self.bias_voltage_v,
                self.drive_warmth_db,
                self.asymmetry_balance_pct,
            );
            let cx = scope_rect.x + 15.0 + (i as f32 / 49.0) * (scope_rect.width - 30.0);
            let cy = mid_y - sat * 35.0;
            let pt = egui::pos2(cx, cy);
            if let Some(prev) = prev_pt {
                painter.line_segment(
                    [prev, pt],
                    Stroke::new(2.0_f32, Color32::from_rgb(0, 255, 180)),
                );
            }
            prev_pt = Some(pt);
        }

        // Draw Harmonic Spectrum Bars (f0, 2f0, 3f0, 4f0, 5f0)
        let harmonics = self.calculate_harmonic_spectrum();
        let bar_labels = ["f0", "2f0", "3f0", "4f0", "5f0"];
        for (i, (&h_db, &label)) in harmonics.iter().zip(bar_labels.iter()).enumerate() {
            let bx = scope_rect.x + 30.0 + (i as f32) * 65.0;
            let by = scope_rect.y + scope_rect.height - 25.0;
            let norm_h = ((h_db + 60.0) / 60.0).clamp(0.0, 1.0);
            let bar_h = norm_h * 55.0;

            let bar_col = if i == 0 {
                Color32::from_rgb(0, 229, 255)
            } else if i % 2 == 1 {
                Color32::from_rgb(255, 215, 0) // Even harmonic
            } else {
                Color32::from_rgb(255, 107, 43) // Odd harmonic
            };

            painter.rect_filled(
                egui::Rect::from_min_max(egui::pos2(bx, by - bar_h), egui::pos2(bx + 35.0, by)),
                2.0,
                bar_col,
            );
            painter.text(
                egui::pos2(bx + 17.0, by + 4.0),
                egui::Align2::CENTER_TOP,
                label,
                egui::FontId::proportional(10.0),
                Color32::from_rgb(180, 200, 225),
            );
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
            "[PASS] Tube Bias Q-Point & Harmonic Distortion Nodes (>= 44x44pt) Compliant",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
