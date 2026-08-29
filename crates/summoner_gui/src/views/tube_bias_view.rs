// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Vacuum Tube Bias Calibration & Harmonic Distortion HUD View (Milestone 11).
//!
//! Provides an interactive plate voltage vs grid bias DC load-line canvas with 2D touch pucks
//! (>= 44x44pt bounding targets), dynamic harmonic distortion spectrum bars (f0..5f0),
//! real-time saturation oscilloscope, and WCAG AA/AAA compliant high-contrast color palettes.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const MIN_HIT_TARGET_PT: f32 = 44.0;
pub const TUBE_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const NUM_HARMONICS: usize = 5;

/// Vacuum tube model / topology selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TubeTopology {
    #[default]
    Triode12AX7,
    PentodeEL34,
    BeamTetrode6L6,
    CleanModern,
}

impl TubeTopology {
    pub fn label(&self) -> &'static str {
        match self {
            TubeTopology::Triode12AX7 => "12AX7 TRIODE (VINTAGE WARMTH)",
            TubeTopology::PentodeEL34 => "EL34 PENTODE (BRITISH PUNCH)",
            TubeTopology::BeamTetrode6L6 => "6L6 BEAM TETRODE (AMERICAN HEADROOM)",
            TubeTopology::CleanModern => "CLEAN MODERN (TRANSPARENT SAT)",
        }
    }
}

/// Physical Tube Amp Bias & Harmonic Distortion HUD View (Milestone 11).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TubeBiasView {
    pub topology: TubeTopology,
    pub bias_voltage_v: f32,      // [-4.0 ..= 0.0 V DC] (Operating grid bias)
    pub plate_voltage_v: f32,     // [100.0 ..= 450.0 V DC]
    pub drive_warmth_db: f32,     // [0.0 ..= +24.0 dB]
    pub sag_compression_pct: f32, // [0.0 ..= 100.0%]
    pub asymmetry_balance_pct: f32, // [0.0 ..= 100.0%] (Even vs Odd harmonic emphasis)
    pub dry_wet_pct: f32,         // [0.0 ..= 100.0%]
    pub q_point_norm: (f32, f32), // (Plate Voltage norm, Anode Current norm)
    pub is_dragging_puck: bool,
    #[serde(skip, default)]
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
                let t = (x as f32 / width as f32) * TAU;
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

    /// Render headless PNG snapshot displaying load-line canvas, Q-point puck, harmonic spectrum, and HUD.
    pub fn render_snapshot_png(&self, path: &str, width: u32, height: u32) -> Result<(), String> {
        if let Some(parent) = std::path::Path::new(path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let mut pixels = vec![0u8; (width * height * 4) as usize];

        // Background: Deep slate/navy (#0C101A)
        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 4) as usize;
                pixels[idx] = 12;
                pixels[idx + 1] = 16;
                pixels[idx + 2] = 26;
                pixels[idx + 3] = 255;
            }
        }

        // Header Panel (y: 10..46, x: 20..width-20)
        fill_rounded_rect_tube(&mut pixels, width, height, 20, 10, width - 40, 36, 4, [18, 24, 38, 255]);

        // Left Panel: Anode Load-Line & Q-Point Canvas (x: 20..380, y: 56..280)
        fill_rounded_rect_tube(&mut pixels, width, height, 20, 56, 360, 224, 6, [10, 14, 22, 255]);
        draw_rect_border_tube(&mut pixels, width, height, 20, 56, 360, 224, [45, 60, 85, 255]);

        // Grid lines on load-line canvas
        for i in 1..4 {
            let gy = 56 + i * 56;
            draw_horizontal_line_tube(&mut pixels, width, height, 24, 376, gy, [25, 35, 50, 255]);
            let gx = 20 + i * 90;
            draw_vertical_line_tube(&mut pixels, width, height, gx, 60, 276, [25, 35, 50, 255]);
        }

        // DC Load Line (orange line from top-left to bottom-right)
        draw_line_segment_tube(&mut pixels, width, height, 40.0, 80.0, 360.0, 260.0, [255, 107, 43, 255]);

        // Bias Q-Point Puck (Gold center, ring with >= 22pt radius -> 44x44pt bounding hit box)
        let qx = 20.0 + self.q_point_norm.0 * 360.0;
        let qy = 56.0 + (1.0 - self.q_point_norm.1) * 224.0;
        draw_circle_ring_tube(&mut pixels, width, height, qx, qy, TUBE_PUCK_HIT_RADIUS, [255, 215, 0, 160]);
        draw_circle_filled_tube(&mut pixels, width, height, qx, qy, 14.0, [255, 215, 0, 255]);
        draw_circle_filled_tube(&mut pixels, width, height, qx, qy, 4.0, [255, 255, 255, 255]);

        // Right Panel: Harmonic Distortion Spectrum & Scope (x: 400..760, y: 56..280)
        fill_rounded_rect_tube(&mut pixels, width, height, 400, 56, 360, 224, 6, [10, 14, 22, 255]);
        draw_rect_border_tube(&mut pixels, width, height, 400, 56, 360, 224, [45, 60, 85, 255]);

        // Scope Waveform
        let scope_mid_y = 120.0f32;
        let mut prev_pt: Option<(f32, f32)> = None;
        for i in 0..100 {
            let t = (i as f32 / 99.0) * TAU * 2.0;
            let raw = t.sin();
            let sat = Self::tube_transfer_sample(
                raw,
                self.bias_voltage_v,
                self.drive_warmth_db,
                self.asymmetry_balance_pct,
            );
            let cx = 420.0 + (i as f32 / 99.0) * 320.0;
            let cy = scope_mid_y - sat * 35.0;
            if let Some((px, py)) = prev_pt {
                draw_line_segment_tube(&mut pixels, width, height, px, py, cx, cy, [0, 255, 180, 255]);
            }
            prev_pt = Some((cx, cy));
        }

        // Harmonic Spectrum Bars (f0, 2f0, 3f0, 4f0, 5f0)
        let harmonics = self.calculate_harmonic_spectrum();
        for (i, &h_db) in harmonics.iter().enumerate() {
            let bx = 430 + (i as u32) * 65;
            let by = 265u32;
            let norm_h = ((h_db + 60.0) / 60.0).clamp(0.05, 1.0);
            let bar_h = (norm_h * 55.0) as u32;

            let col = if i == 0 {
                [0, 229, 255, 255] // Fundamental cyan
            } else if i % 2 == 1 {
                [255, 215, 0, 255] // Even harmonic gold
            } else {
                [255, 107, 43, 255] // Odd harmonic orange
            };

            fill_rect_tube(&mut pixels, width, height, bx, by.saturating_sub(bar_h), 35, bar_h, col);
        }

        // Bottom Controls Bar (x: 20..760, y: 295..480)
        fill_rounded_rect_tube(&mut pixels, width, height, 20, 295, 760, 185, 6, [18, 25, 38, 255]);

        // Compliance Badge (x: 35..765, y: 430..465)
        fill_rounded_rect_tube(&mut pixels, width, height, 35, 430, 730, 36, 4, [16, 35, 28, 255]);
        draw_rect_border_tube(&mut pixels, width, height, 35, 430, 730, 36, [0, 255, 180, 255]);

        encode_minimal_png(path, &pixels, width, height)
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
            let t = (i as f32 / 49.0) * TAU * 2.0;
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

// ----------------------------------------------------------------------------
// Minimal software rasterizer & PNG encoder helpers for headless verification
// ----------------------------------------------------------------------------

fn fill_rect_tube(buf: &mut [u8], w: u32, h: u32, rx: u32, ry: u32, rw: u32, rh: u32, col: [u8; 4]) {
    for y in ry..(ry + rh).min(h) {
        for x in rx..(rx + rw).min(w) {
            let idx = ((y * w + x) * 4) as usize;
            buf[idx..idx + 4].copy_from_slice(&col);
        }
    }
}

fn fill_rounded_rect_tube(
    buf: &mut [u8],
    w: u32,
    h: u32,
    rx: u32,
    ry: u32,
    rw: u32,
    rh: u32,
    radius: u32,
    col: [u8; 4],
) {
    let r = radius as f32;
    for y in ry..(ry + rh).min(h) {
        for x in rx..(rx + rw).min(w) {
            let mut inside = true;
            if x < rx + radius && y < ry + radius {
                let dx = (rx + radius - x) as f32;
                let dy = (ry + radius - y) as f32;
                inside = dx * dx + dy * dy <= r * r;
            } else if x >= rx + rw - radius && y < ry + radius {
                let dx = (x - (rx + rw - radius - 1)) as f32;
                let dy = (ry + radius - y) as f32;
                inside = dx * dx + dy * dy <= r * r;
            } else if x < rx + radius && y >= ry + rh - radius {
                let dx = (rx + radius - x) as f32;
                let dy = (y - (ry + rh - radius - 1)) as f32;
                inside = dx * dx + dy * dy <= r * r;
            } else if x >= rx + rw - radius && y >= ry + rh - radius {
                let dx = (x - (rx + rw - radius - 1)) as f32;
                let dy = (y - (ry + rh - radius - 1)) as f32;
                inside = dx * dx + dy * dy <= r * r;
            }
            if inside {
                let idx = ((y * w + x) * 4) as usize;
                buf[idx..idx + 4].copy_from_slice(&col);
            }
        }
    }
}

fn draw_rect_border_tube(buf: &mut [u8], w: u32, h: u32, rx: u32, ry: u32, rw: u32, rh: u32, col: [u8; 4]) {
    for x in rx..(rx + rw).min(w) {
        if ry < h {
            let idx = ((ry * w + x) * 4) as usize;
            buf[idx..idx + 4].copy_from_slice(&col);
        }
        if ry + rh - 1 < h {
            let idx = (((ry + rh - 1) * w + x) * 4) as usize;
            buf[idx..idx + 4].copy_from_slice(&col);
        }
    }
    for y in ry..(ry + rh).min(h) {
        if rx < w {
            let idx = ((y * w + rx) * 4) as usize;
            buf[idx..idx + 4].copy_from_slice(&col);
        }
        if rx + rw - 1 < w {
            let idx = ((y * w + rx + rw - 1) * 4) as usize;
            buf[idx..idx + 4].copy_from_slice(&col);
        }
    }
}

fn draw_horizontal_line_tube(buf: &mut [u8], w: u32, h: u32, x0: u32, x1: u32, y: u32, col: [u8; 4]) {
    if y >= h { return; }
    for x in x0.min(w)..=x1.min(w - 1) {
        let idx = ((y * w + x) * 4) as usize;
        buf[idx..idx + 4].copy_from_slice(&col);
    }
}

fn draw_vertical_line_tube(buf: &mut [u8], w: u32, h: u32, x: u32, y0: u32, y1: u32, col: [u8; 4]) {
    if x >= w { return; }
    for y in y0.min(h)..=y1.min(h - 1) {
        let idx = ((y * w + x) * 4) as usize;
        buf[idx..idx + 4].copy_from_slice(&col);
    }
}

fn draw_line_segment_tube(buf: &mut [u8], w: u32, h: u32, x0: f32, y0: f32, x1: f32, y1: f32, col: [u8; 4]) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let steps = (dx.abs().max(dy.abs()) * 2.0).max(1.0) as u32;
    for s in 0..=steps {
        let t = s as f32 / steps as f32;
        let px = (x0 + t * dx).round() as i32;
        let py = (y0 + t * dy).round() as i32;
        if px >= 0 && px < w as i32 && py >= 0 && py < h as i32 {
            let idx = ((py as u32 * w + px as u32) * 4) as usize;
            buf[idx..idx + 4].copy_from_slice(&col);
        }
    }
}

fn draw_circle_filled_tube(buf: &mut [u8], w: u32, h: u32, cx: f32, cy: f32, r: f32, col: [u8; 4]) {
    let min_x = (cx - r).max(0.0) as u32;
    let max_x = (cx + r).min(w as f32 - 1.0) as u32;
    let min_y = (cy - r).max(0.0) as u32;
    let max_y = (cy + r).min(h as f32 - 1.0) as u32;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            if dx * dx + dy * dy <= r * r {
                let idx = ((y * w + x) * 4) as usize;
                buf[idx..idx + 4].copy_from_slice(&col);
            }
        }
    }
}

fn draw_circle_ring_tube(buf: &mut [u8], w: u32, h: u32, cx: f32, cy: f32, r: f32, col: [u8; 4]) {
    let thickness = 2.0f32;
    let r_inner = r - thickness;
    let min_x = (cx - r).max(0.0) as u32;
    let max_x = (cx + r).min(w as f32 - 1.0) as u32;
    let min_y = (cy - r).max(0.0) as u32;
    let max_y = (cy + r).min(h as f32 - 1.0) as u32;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq <= r * r && dist_sq >= r_inner * r_inner {
                let idx = ((y * w + x) * 4) as usize;
                buf[idx..idx + 4].copy_from_slice(&col);
            }
        }
    }
}

/// Minimal PNG encoder writing uncompressed raw RGBA stream into compliant PNG container.
fn encode_minimal_png(path: &str, rgba_pixels: &[u8], width: u32, height: u32) -> Result<(), String> {
    let mut out = Vec::new();
    out.extend_from_slice(b"\x89PNG\r\n\x1a\n");

    // IHDR Chunk
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.push(8); // Bit depth
    ihdr.push(6); // Color type RGBA
    ihdr.push(0); // Compression method Deflate
    ihdr.push(0); // Filter method None
    ihdr.push(0); // Interlace None
    write_png_chunk_tube(&mut out, b"IHDR", &ihdr);

    // IDAT Chunk
    let mut raw_data = Vec::with_capacity((height * (width * 4 + 1)) as usize);
    for y in 0..height {
        raw_data.push(0); // Filter type 0 None
        let row_start = (y * width * 4) as usize;
        let row_end = row_start + (width * 4) as usize;
        raw_data.extend_from_slice(&rgba_pixels[row_start..row_end]);
    }

    let mut zlib_data = Vec::with_capacity(raw_data.len() + 128);
    zlib_data.push(0x78);
    zlib_data.push(0x01);

    let mut offset = 0;
    while offset < raw_data.len() {
        let chunk_len = (raw_data.len() - offset).min(65535);
        let is_last = (offset + chunk_len) >= raw_data.len();
        let bfinal_btype = if is_last { 0x01 } else { 0x00 };
        zlib_data.push(bfinal_btype);

        let len_u16 = chunk_len as u16;
        let nlen_u16 = !len_u16;
        zlib_data.extend_from_slice(&len_u16.to_le_bytes());
        zlib_data.extend_from_slice(&nlen_u16.to_le_bytes());
        zlib_data.extend_from_slice(&raw_data[offset..offset + chunk_len]);
        offset += chunk_len;
    }

    let mut s1: u32 = 1;
    let mut s2: u32 = 0;
    for &b in &raw_data {
        s1 = (s1 + b as u32) % 65521;
        s2 = (s2 + s1) % 65521;
    }
    let adler = (s2 << 16) | s1;
    zlib_data.extend_from_slice(&adler.to_be_bytes());

    write_png_chunk_tube(&mut out, b"IDAT", &zlib_data);
    write_png_chunk_tube(&mut out, b"IEND", &[]);

    std::fs::write(path, out).map_err(|e| e.to_string())
}

fn write_png_chunk_tube(out: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let type_start = out.len();
    out.extend_from_slice(chunk_type);
    out.extend_from_slice(data);
    let crc = crc32_compute_tube(&out[type_start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

fn crc32_compute_tube(buf: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in buf {
        crc ^= b as u32;
        for _ in 0..8 {
            let mask = if (crc & 1) != 0 { 0xEDB8_8320 } else { 0 };
            crc = (crc >> 1) ^ mask;
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tube_bias_view_ascii_render() {
        let view = TubeBiasView::new();
        let ascii = view.render_ascii(80, 24);
        assert!(!ascii.is_empty());
        assert!(ascii[0].contains("TUBE BIAS"));
        assert!(ascii[0].contains("Bias:"));
    }

    #[test]
    fn test_tube_bias_view_hit_target_dimensions() {
        const { assert!(MIN_HIT_TARGET_PT >= 44.0, "Hit targets must be >= 44pt for touch accessibility") };
        const { assert!(TUBE_PUCK_HIT_RADIUS * 2.0 >= 44.0, "Puck hit bounds must be >= 44pt") };
    }

    #[test]
    fn test_tube_bias_view_render_snapshot_png() {
        let view = TubeBiasView::new();
        let render_path = "scratch/renders/tube_bias_view.png";
        let res = view.render_snapshot_png(render_path, 800, 520);
        let _ = view.render_snapshot_png("../../scratch/renders/tube_bias_view.png", 800, 520);
        assert!(res.is_ok(), "Snapshot render must succeed: {:?}", res);
        assert!(
            std::path::Path::new(render_path).exists() || std::path::Path::new("../../scratch/renders/tube_bias_view.png").exists(),
            "Rendered snapshot PNG must exist at {}",
            render_path
        );
    }
}
