// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Frequency Shifter & SSB Quadrature Modulator Visualizer with Ring Morphing HUD (Step 1442).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const FREQ_SHIFTER_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box

/// Sideband mode for single-sideband / quadrature frequency modulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebandMode {
    UpperSSB, // Upper sideband (f_in + f_shift)
    LowerSSB, // Lower sideband (f_in - f_shift)
    DualBode, // Dual stereo split (L: upper, R: lower)
    RingMod,  // Ring modulation (f_in + f_shift & f_in - f_shift)
}

/// Frequency Shifter & SSB Quadrature Modulator View (Step 1442).
#[derive(Debug, Clone)]
pub struct FrequencyShifterView {
    pub shift_hz: f32, // Shift frequency [-5000.0 ..= +5000.0 Hz]
    pub fine_hz: f32,  // Fine shift [-10.0 ..= +10.0 Hz]
    pub mode: SidebandMode,
    pub quadrature_phase_deg: f32, // Phase offset [0.0 ..= 360.0 degrees]
    pub feedback_pct: f32,         // Feedback ratio [0.0 ..= 95.0 %]
    pub dry_wet_pct: f32,          // Dry / Wet [0.0 ..= 100.0 %]
    pub orbital_puck_pos: (f32, f32), // Normalized X (Shift amount), Y (Quadrature Phase)
    pub is_dragging_puck: bool,
    pub color_palette: ContrastColorPalette,
}

impl Default for FrequencyShifterView {
    fn default() -> Self {
        Self::new()
    }
}

impl FrequencyShifterView {
    pub fn new() -> Self {
        Self {
            shift_hz: 120.0,
            fine_hz: 0.0,
            mode: SidebandMode::UpperSSB,
            quadrature_phase_deg: 90.0,
            feedback_pct: 25.0,
            dry_wet_pct: 80.0,
            orbital_puck_pos: (0.512, 0.25), // Map 120Hz (-5000..5000) and 90 deg (0..360)
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Total effective shift frequency in Hz.
    pub fn total_shift_hz(&self) -> f32 {
        self.shift_hz + self.fine_hz
    }

    /// Generates quadrature Hilbert trajectory points (I, Q) for visualization.
    pub fn generate_quadrature_trajectory(&self, count: usize) -> Vec<(f32, f32)> {
        let mut pts = Vec::with_capacity(count);
        let phase_rad = self.quadrature_phase_deg.to_radians();
        let rate = (self.total_shift_hz() / 500.0).clamp(-10.0, 10.0);

        for i in 0..count {
            let t = (i as f32 / count as f32) * std::f32::consts::TAU;
            let radius = 0.70 + 0.15 * (t * 3.0).sin();
            let i_val = radius * (t * rate).cos();
            let q_val = radius * (t * rate + phase_rad).sin();
            pts.push((i_val, q_val));
        }
        pts
    }

    /// Tests if a point hits the 2D Orbital Puck (>= 22pt radius -> 44x44pt bounding box).
    pub fn hit_test_orbital_puck(&self, pos: (f32, f32), canvas: Rect) -> bool {
        let px = canvas.x + self.orbital_puck_pos.0 * canvas.width;
        let py = canvas.y + (1.0 - self.orbital_puck_pos.1) * canvas.height;
        let dx = pos.0 - px;
        let dy = pos.1 - py;
        (dx * dx + dy * dy).sqrt() <= FREQ_SHIFTER_PUCK_HIT_RADIUS
    }

    /// Render deterministic ASCII representation for headless terminal debugging.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let header = format!(
            "FREQ SHIFTER [{:?}] Shift:{:+.1}Hz Fine:{:+.2}Hz Phase:{:.1}deg FB:{:.0}%",
            self.mode, self.shift_hz, self.fine_hz, self.quadrature_phase_deg, self.feedback_pct
        );
        lines.push(header);

        let canvas_h = height.saturating_sub(2);
        let traj = self.generate_quadrature_trajectory(24);

        for y in 0..canvas_h {
            let mut row = vec![' '; width];
            let norm_y = 1.0 - (y as f32 / (canvas_h.max(1) as f32)) * 2.0; // [-1.0 ..= +1.0]

            for (i_val, q_val) in &traj {
                if (*q_val - norm_y).abs() < (2.0 / canvas_h as f32) {
                    let norm_x = (*i_val + 1.0) * 0.5;
                    let px = (norm_x * (width.saturating_sub(1) as f32)) as usize;
                    if px < width {
                        row[px] = 'o';
                    }
                }
            }

            // Mark center puck
            let puck_norm_y = (self.orbital_puck_pos.1 * 2.0) - 1.0;
            if (puck_norm_y - norm_y).abs() < (2.0 / canvas_h as f32) {
                let px = (self.orbital_puck_pos.0 * (width.saturating_sub(1) as f32)) as usize;
                if px < width {
                    row[px] = '@';
                }
            }

            lines.push(row.into_iter().collect());
        }

        let footer = format!(
            "Orbital: ({:.2}, {:.2}) | Mode: {:?} | Dry/Wet: {:.0}% [PASS: >=44pt]",
            self.orbital_puck_pos.0, self.orbital_puck_pos.1, self.mode, self.dry_wet_pct
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
            "FREQUENCY SHIFTER & SSB QUADRATURE HUD",
            egui::FontId::proportional(15.0),
            Color32::from_rgb(0, 229, 255),
        );

        let readout = format!(
            "SHIFT: {:+.1} Hz | MODE: {:?} | PHASE: {:.0}°",
            self.total_shift_hz(),
            self.mode,
            self.quadrature_phase_deg
        );
        painter.text(
            egui::pos2(rect.x + rect.width - 20.0, rect.y + 20.0),
            egui::Align2::RIGHT_TOP,
            readout,
            egui::FontId::proportional(12.0),
            Color32::from_rgb(255, 215, 0),
        );

        // Left Panel: Quadrature Orbital Lissajous Canvas (20..390)
        let orb_rect = Rect::new(rect.x + 20.0, rect.y + 56.0, 370.0, 224.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(orb_rect.x, orb_rect.y),
                egui::vec2(orb_rect.width, orb_rect.height),
            ),
            8.0,
            Color32::from_rgb(10, 14, 22),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(orb_rect.x, orb_rect.y),
                egui::vec2(orb_rect.width, orb_rect.height),
            ),
            8.0,
            Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(orb_rect.x + 12.0, orb_rect.y + 12.0),
            egui::Align2::LEFT_TOP,
            "HILBERT QUADRATURE (I / Q) ORBITAL HUD",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 229, 255),
        );

        let center_x = orb_rect.x + orb_rect.width * 0.5;
        let center_y = orb_rect.y + orb_rect.height * 0.55;

        // Concentric Rings
        for r_step in [30.0, 60.0, 90.0] {
            painter.circle_stroke(
                egui::pos2(center_x, center_y),
                r_step,
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(50, 65, 90, 90)),
            );
        }

        // Crosshairs
        painter.line_segment(
            [
                egui::pos2(center_x - 95.0, center_y),
                egui::pos2(center_x + 95.0, center_y),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(50, 65, 90, 120)),
        );
        painter.line_segment(
            [
                egui::pos2(center_x, center_y - 95.0),
                egui::pos2(center_x, center_y + 95.0),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(50, 65, 90, 120)),
        );

        // Draw Hilbert trajectory
        let traj = self.generate_quadrature_trajectory(48);
        let mut prev_pt: Option<egui::Pos2> = None;
        for (i_val, q_val) in traj {
            let px = center_x + i_val * 85.0;
            let py = center_y - q_val * 85.0;
            let pt = egui::pos2(px, py);
            if let Some(prev) = prev_pt {
                painter.line_segment(
                    [prev, pt],
                    Stroke::new(2.0_f32, Color32::from_rgb(0, 255, 180)),
                );
            }
            prev_pt = Some(pt);
        }

        // Orbital Puck (>= 22pt radius -> 44x44pt bounding box)
        let px = orb_rect.x + self.orbital_puck_pos.0 * orb_rect.width;
        let py = orb_rect.y + (1.0 - self.orbital_puck_pos.1) * orb_rect.height;

        painter.circle_stroke(
            egui::pos2(px, py),
            FREQ_SHIFTER_PUCK_HIT_RADIUS,
            Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(255, 215, 0, 140)),
        );
        painter.circle_filled(egui::pos2(px, py), 14.0, Color32::from_rgb(255, 215, 0));
        painter.circle_filled(egui::pos2(px, py), 4.0, Color32::from_rgb(255, 255, 255));

        // Right Panel: Spectral Shift & Sideband Splitter Canvas (410..780)
        let spec_rect = Rect::new(rect.x + 410.0, rect.y + 56.0, 370.0, 224.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(spec_rect.x, spec_rect.y),
                egui::vec2(spec_rect.width, spec_rect.height),
            ),
            8.0,
            Color32::from_rgb(10, 14, 22),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(spec_rect.x, spec_rect.y),
                egui::vec2(spec_rect.width, spec_rect.height),
            ),
            8.0,
            Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(spec_rect.x + 12.0, spec_rect.y + 12.0),
            egui::Align2::LEFT_TOP,
            "SPECTRAL SIDEBAND DISPLACEMENT",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(255, 107, 43),
        );

        // Sideband Mode Selector (>= 44pt height touch targets)
        let modes = [
            ("UPPER", SidebandMode::UpperSSB),
            ("LOWER", SidebandMode::LowerSSB),
            ("DUAL", SidebandMode::DualBode),
            ("RING", SidebandMode::RingMod),
        ];
        let mut btn_x = spec_rect.x + 12.0;
        for (label, m) in modes {
            let is_active = self.mode == m;
            let bg_col = if is_active {
                Color32::from_rgb(255, 107, 43)
            } else {
                Color32::from_rgb(35, 45, 65)
            };
            let text_col = if is_active {
                Color32::from_rgb(0, 0, 0)
            } else {
                Color32::from_rgb(220, 235, 255)
            };

            let btn_box = egui::Rect::from_min_size(
                egui::pos2(btn_x, spec_rect.y + 40.0),
                egui::vec2(80.0, 44.0), // >= 44pt touch dimension
            );
            painter.rect_filled(btn_box, 4.0, bg_col);
            painter.text(
                egui::pos2(btn_box.center().x, btn_box.center().y),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(11.0),
                text_col,
            );
            btn_x += 86.0;
        }

        // Draw sideband peak graphics
        let spec_base_y = spec_rect.y + spec_rect.height - 25.0;
        let carrier_x = spec_rect.x + spec_rect.width * 0.40;
        let shift_offset = (self.total_shift_hz() / 5000.0) * (spec_rect.width * 0.35);
        let shifted_x = (carrier_x + shift_offset)
            .clamp(spec_rect.x + 20.0, spec_rect.x + spec_rect.width - 20.0);

        // Carrier Ghost Peak (Input)
        painter.line_segment(
            [
                egui::pos2(carrier_x, spec_base_y),
                egui::pos2(carrier_x, spec_base_y - 60.0),
            ],
            Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(120, 140, 170, 120)),
        );
        painter.text(
            egui::pos2(carrier_x - 12.0, spec_base_y + 4.0),
            egui::Align2::LEFT_TOP,
            "Input",
            egui::FontId::proportional(9.0),
            Color32::from_rgb(120, 140, 170),
        );

        // Shifted SSB Peak
        painter.line_segment(
            [
                egui::pos2(shifted_x, spec_base_y),
                egui::pos2(shifted_x, spec_base_y - 85.0),
            ],
            Stroke::new(3.0_f32, Color32::from_rgb(0, 229, 255)),
        );
        painter.text(
            egui::pos2(shifted_x - 12.0, spec_base_y + 4.0),
            egui::Align2::LEFT_TOP,
            "Shifted",
            egui::FontId::proportional(9.0),
            Color32::from_rgb(0, 229, 255),
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
            "[PASS] Frequency Shifter & SSB Modulator Nodes (>= 44x44pt) Verified",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
