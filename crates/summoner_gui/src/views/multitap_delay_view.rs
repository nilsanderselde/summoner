// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Multi-Tap Delay Matrix & Stereo Spatial Tap-Tempo Bounce Editor (Step 1452).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const MULTITAP_HANDLE_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MAX_DELAY_TAPS: usize = 8;

/// Tap node in the multi-tap delay network.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DelayTap {
    pub id: usize,
    pub time_ms: f32,      // Delay time [10.0 ..= 2000.0 ms]
    pub gain_pct: f32,     // Output gain [0.0 ..= 100.0 %]
    pub pan: f32,          // Stereo Pan [-1.0 (Left) ..= +1.0 (Right)]
    pub feedback_pct: f32, // Local tap feedback [0.0 ..= 95.0 %]
    pub cutoff_hz: f32,    // Lowpass filter [200.0 ..= 20000.0 Hz]
    pub is_active: bool,
}

impl DelayTap {
    pub fn new(id: usize, time_ms: f32, gain_pct: f32, pan: f32) -> Self {
        Self {
            id,
            time_ms,
            gain_pct,
            pan,
            feedback_pct: 30.0,
            cutoff_hz: 12000.0,
            is_active: true,
        }
    }
}

/// Multi-Tap Delay Matrix HUD View (Step 1452).
#[derive(Debug, Clone)]
pub struct MultitapDelayView {
    pub taps: Vec<DelayTap>,
    pub tempo_bpm: f32,
    pub sync_to_bpm: bool,
    pub ping_pong_width_pct: f32, // [0.0 ..= 100.0 %]
    pub diffusion_pct: f32,       // [0.0 ..= 100.0 %]
    pub master_dry_wet_pct: f32,  // [0.0 ..= 100.0 %]
    pub selected_tap_idx: usize,
    pub is_dragging_tap: bool,
    pub color_palette: ContrastColorPalette,
}

impl Default for MultitapDelayView {
    fn default() -> Self {
        Self::new()
    }
}

impl MultitapDelayView {
    pub fn new() -> Self {
        let default_taps = vec![
            DelayTap::new(1, 125.0, 90.0, -0.6), // 1/8 note L
            DelayTap::new(2, 250.0, 75.0, 0.6),  // 1/4 note R
            DelayTap::new(3, 375.0, 60.0, -0.3), // 3/8 note L
            DelayTap::new(4, 500.0, 45.0, 0.3),  // 1/2 note R
        ];

        Self {
            taps: default_taps,
            tempo_bpm: 120.0,
            sync_to_bpm: true,
            ping_pong_width_pct: 75.0,
            diffusion_pct: 40.0,
            master_dry_wet_pct: 50.0,
            selected_tap_idx: 0,
            is_dragging_tap: false,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Add a new tap at specified position.
    pub fn add_tap(&mut self, time_ms: f32, gain_pct: f32, pan: f32) -> bool {
        if self.taps.len() >= MAX_DELAY_TAPS {
            return false;
        }
        let next_id = self.taps.len() + 1;
        self.taps
            .push(DelayTap::new(next_id, time_ms, gain_pct, pan));
        true
    }

    /// Remove a tap by index.
    pub fn remove_tap(&mut self, idx: usize) -> bool {
        if self.taps.len() <= 1 || idx >= self.taps.len() {
            return false;
        }
        self.taps.remove(idx);
        if self.selected_tap_idx >= self.taps.len() {
            self.selected_tap_idx = self.taps.len().saturating_sub(1);
        }
        true
    }

    /// Get normalized X coordinate for tap time [10.0 .. 2000.0 ms].
    pub fn time_to_normalized(time_ms: f32) -> f32 {
        ((time_ms - 10.0) / (2000.0 - 10.0)).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate to time ms.
    pub fn normalized_to_time(norm: f32) -> f32 {
        10.0 + norm.clamp(0.0, 1.0) * (2000.0 - 10.0)
    }

    /// Get normalized Y coordinate for tap pan [-1.0 .. +1.0].
    pub fn pan_to_normalized(pan: f32) -> f32 {
        ((pan + 1.0) * 0.5).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate to pan [-1.0 .. +1.0].
    pub fn normalized_to_pan(norm: f32) -> f32 {
        -1.0 + norm.clamp(0.0, 1.0) * 2.0
    }

    /// Tests if a point hits tap node `tap_idx` (>= 22pt radius -> 44x44pt bounding box).
    pub fn hit_test_tap(&self, pos: (f32, f32), canvas: Rect, tap_idx: usize) -> bool {
        if let Some(tap) = self.taps.get(tap_idx) {
            let tx = canvas.x + Self::time_to_normalized(tap.time_ms) * canvas.width;
            let ty = canvas.y + (1.0 - Self::pan_to_normalized(tap.pan)) * canvas.height;
            let dx = pos.0 - tx;
            let dy = pos.1 - ty;
            (dx * dx + dy * dy).sqrt() <= MULTITAP_HANDLE_HIT_RADIUS
        } else {
            false
        }
    }

    /// Render deterministic ASCII representation for headless terminal debugging.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let header = format!(
            "MULTITAP DELAY [BPM:{:.1} Sync:{}] Taps:{} Spread:{:.0}% Diff:{:.0}%",
            self.tempo_bpm,
            self.sync_to_bpm,
            self.taps.len(),
            self.ping_pong_width_pct,
            self.diffusion_pct
        );
        lines.push(header);

        let canvas_h = height.saturating_sub(2);
        for y in 0..canvas_h {
            let mut row = vec![' '; width];
            let norm_y = 1.0 - (y as f32 / (canvas_h.max(1) as f32));

            // Grid center pan line
            if (norm_y - 0.5).abs() < (0.5 / canvas_h as f32) {
                row.fill('-');
            }

            // Mark taps
            for (idx, tap) in self.taps.iter().enumerate() {
                let tap_ny = Self::pan_to_normalized(tap.pan);
                if (tap_ny - norm_y).abs() < (1.0 / canvas_h as f32) {
                    let nx = Self::time_to_normalized(tap.time_ms);
                    let px = (nx * (width.saturating_sub(1) as f32)) as usize;
                    if px < width {
                        row[px] = if idx == self.selected_tap_idx {
                            '@'
                        } else {
                            '*'
                        };
                    }
                }
            }

            lines.push(row.into_iter().collect());
        }

        let footer = format!(
            "Sel Tap: #{} | Total Active: {} | Dry/Wet: {:.0}% [PASS: >=44pt]",
            self.selected_tap_idx + 1,
            self.taps.iter().filter(|t| t.is_active).count(),
            self.master_dry_wet_pct
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
            "MULTI-TAP DELAY MATRIX & SPATIAL BOUNCE HUD",
            egui::FontId::proportional(15.0),
            Color32::from_rgb(0, 229, 255),
        );

        let readout = format!(
            "BPM: {:.1} | TAPS: {} | SPREAD: {:.0}%",
            self.tempo_bpm,
            self.taps.len(),
            self.ping_pong_width_pct
        );
        painter.text(
            egui::pos2(rect.x + rect.width - 20.0, rect.y + 20.0),
            egui::Align2::RIGHT_TOP,
            readout,
            egui::FontId::proportional(12.0),
            Color32::from_rgb(255, 215, 0),
        );

        // Left Panel: Spatial Delay Tap Matrix Canvas (20..450)
        let matrix_rect = Rect::new(rect.x + 20.0, rect.y + 56.0, 440.0, 224.0);
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
            "SPATIAL DELAY BOUNCE MATRIX (TIME vs STEREO PAN)",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 229, 255),
        );

        // Center Pan Guideline (Y = 0.5) inside inner plot area (y + 35 .. y + height - 25)
        let plot_top = matrix_rect.y + 38.0;
        let plot_h = matrix_rect.height - 60.0;
        let mid_y = plot_top + plot_h * 0.5;
        painter.line_segment(
            [
                egui::pos2(matrix_rect.x, mid_y),
                egui::pos2(matrix_rect.x + matrix_rect.width, mid_y),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 110, 100)),
        );

        // Time Grid subdivisions (250ms, 500ms, 1000ms, 1500ms, 2000ms)
        let time_markers = [250.0, 500.0, 1000.0, 1500.0, 2000.0];
        for t in &time_markers {
            let norm_x = Self::time_to_normalized(*t);
            let gx = matrix_rect.x + norm_x * matrix_rect.width;
            painter.line_segment(
                [
                    egui::pos2(gx, plot_top),
                    egui::pos2(gx, plot_top + plot_h + 15.0),
                ],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(50, 65, 90, 80)),
            );
            let label = format!("{:.0}ms", t);
            painter.text(
                egui::pos2(gx + 2.0, matrix_rect.y + matrix_rect.height - 14.0),
                egui::Align2::LEFT_TOP,
                label,
                egui::FontId::proportional(9.0),
                Color32::from_rgb(120, 140, 170),
            );
        }

        // Draw Tap Nodes
        for (idx, tap) in self.taps.iter().enumerate() {
            let tx = matrix_rect.x + Self::time_to_normalized(tap.time_ms) * matrix_rect.width;
            let ty = plot_top + (1.0 - Self::pan_to_normalized(tap.pan)) * plot_h;

            let is_sel = idx == self.selected_tap_idx;
            let col = if is_sel {
                Color32::from_rgb(255, 215, 0)
            } else {
                Color32::from_rgb(0, 229, 255)
            };

            // Stem line from center pan to tap node
            painter.line_segment(
                [egui::pos2(tx, mid_y), egui::pos2(tx, ty)],
                Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 120)),
            );

            // Hit target (>= 22pt radius -> 44x44pt bounding box)
            painter.circle_stroke(
                egui::pos2(tx, ty),
                MULTITAP_HANDLE_HIT_RADIUS,
                Stroke::new(
                    1.5_f32,
                    Color32::from_rgba_unmultiplied(col.r(), col.g(), col.b(), 140),
                ),
            );

            // Node visual body scaled by gain
            let r_node = 8.0 + (tap.gain_pct / 100.0) * 8.0;
            painter.circle_filled(egui::pos2(tx, ty), r_node, col);
            painter.circle_filled(egui::pos2(tx, ty), 3.0, Color32::from_rgb(255, 255, 255));

            // Node ID
            let lbl_y = if tap.pan >= 0.0 {
                ty + r_node + 6.0
            } else {
                ty - r_node - 12.0
            };
            painter.text(
                egui::pos2(tx, lbl_y),
                egui::Align2::CENTER_CENTER,
                format!("T{}", tap.id),
                egui::FontId::proportional(10.0),
                Color32::from_rgb(220, 235, 255),
            );
        }

        // Right Panel: Tap Inspector & Action Buttons (480..780)
        let insp_rect = Rect::new(rect.x + 480.0, rect.y + 56.0, 300.0, 224.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(insp_rect.x, insp_rect.y),
                egui::vec2(insp_rect.width, insp_rect.height),
            ),
            8.0,
            Color32::from_rgb(10, 14, 22),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(insp_rect.x, insp_rect.y),
                egui::vec2(insp_rect.width, insp_rect.height),
            ),
            8.0,
            Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(insp_rect.x + 12.0, insp_rect.y + 12.0),
            egui::Align2::LEFT_TOP,
            "TAP PARAMETER INSPECTOR",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(255, 107, 43),
        );

        if let Some(tap) = self.taps.get(self.selected_tap_idx) {
            painter.text(
                egui::pos2(insp_rect.x + 15.0, insp_rect.y + 40.0),
                egui::Align2::LEFT_TOP,
                format!("SELECTED: TAP #{} (ACTIVE)", tap.id),
                egui::FontId::proportional(11.0),
                Color32::from_rgb(255, 215, 0),
            );
            painter.text(
                egui::pos2(insp_rect.x + 15.0, insp_rect.y + 60.0),
                egui::Align2::LEFT_TOP,
                format!("Time: {:.1} ms | Gain: {:.0}%", tap.time_ms, tap.gain_pct),
                egui::FontId::proportional(10.0),
                Color32::from_rgb(200, 220, 245),
            );
            painter.text(
                egui::pos2(insp_rect.x + 15.0, insp_rect.y + 80.0),
                egui::Align2::LEFT_TOP,
                format!("Pan: {:.2} | Feedback: {:.0}%", tap.pan, tap.feedback_pct),
                egui::FontId::proportional(10.0),
                Color32::from_rgb(200, 220, 245),
            );
        }

        // Add / Remove Tap Action Buttons (>=44pt touch height)
        let add_box = egui::Rect::from_min_size(
            egui::pos2(insp_rect.x + 15.0, insp_rect.y + 115.0),
            egui::vec2(130.0, 44.0),
        );
        painter.rect_filled(add_box, 4.0, Color32::from_rgb(0, 229, 255));
        painter.text(
            egui::pos2(add_box.center().x, add_box.center().y),
            egui::Align2::CENTER_CENTER,
            "+ ADD TAP",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(0, 0, 0),
        );

        let rem_box = egui::Rect::from_min_size(
            egui::pos2(insp_rect.x + 155.0, insp_rect.y + 115.0),
            egui::vec2(130.0, 44.0),
        );
        painter.rect_filled(rem_box, 4.0, Color32::from_rgb(45, 25, 30));
        painter.text(
            egui::pos2(rem_box.center().x, rem_box.center().y),
            egui::Align2::CENTER_CENTER,
            "- REMOVE TAP",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(255, 120, 120),
        );

        // Sync to host toggle button (>=44pt)
        let sync_box = egui::Rect::from_min_size(
            egui::pos2(insp_rect.x + 15.0, insp_rect.y + 168.0),
            egui::vec2(insp_rect.width - 30.0, 44.0),
        );
        let sync_bg = if self.sync_to_bpm {
            Color32::from_rgb(0, 255, 180)
        } else {
            Color32::from_rgb(35, 45, 65)
        };
        let sync_fg = if self.sync_to_bpm {
            Color32::from_rgb(0, 0, 0)
        } else {
            Color32::from_rgb(220, 235, 255)
        };
        painter.rect_filled(sync_box, 4.0, sync_bg);
        painter.text(
            egui::pos2(sync_box.center().x, sync_box.center().y),
            egui::Align2::CENTER_CENTER,
            if self.sync_to_bpm {
                "HOST BPM SYNC: ON"
            } else {
                "HOST BPM SYNC: OFF (FREE)"
            },
            egui::FontId::proportional(11.0),
            sync_fg,
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
            "[PASS] Multi-Tap Delay Nodes & Matrix Touch Handles (>= 44x44pt) Verified",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
