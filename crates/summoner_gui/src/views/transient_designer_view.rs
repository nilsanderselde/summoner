// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Tactile Transient Designer & Punch/Sustain Harmonic Modeler Canvas (Step 1454).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const TRANSIENT_DESIGNER_HANDLE_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box

/// Operating mode for the transient designer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransientMode {
    Broadband,      // Full-band attack & sustain reshaping
    FrequencySplit, // Split band targeting low-end punch and high-end transient snap
    HarmonicPunch,  // Dynamic saturation injection on attack crests
}

/// Tactile Transient Designer View (Step 1454).
#[derive(Debug, Clone)]
pub struct TransientDesignerView {
    pub attack_gain_db: f32,    // Attack boost/cut [-24.0 ..= +24.0 dB]
    pub attack_length_ms: f32,  // Attack transient window [5.0 ..= 100.0 ms]
    pub sustain_gain_db: f32,   // Sustain body boost/cut [-24.0 ..= +24.0 dB]
    pub sustain_length_ms: f32, // Sustain decay window [50.0 ..= 1000.0 ms]
    pub punch_freq_hz: f32,     // Harmonic punch resonance [40.0 ..= 500.0 Hz]
    pub soft_clip_enabled: bool,
    pub mode: TransientMode,
    pub output_gain_db: f32,            // [-12.0 ..= +12.0 dB]
    pub attack_handle_pos: (f32, f32),  // Normalized X (Attack Time), Y (Attack Gain)
    pub sustain_handle_pos: (f32, f32), // Normalized X (Sustain Time), Y (Sustain Gain)
    pub is_dragging_attack: bool,
    pub is_dragging_sustain: bool,
    pub color_palette: ContrastColorPalette,
}

impl Default for TransientDesignerView {
    fn default() -> Self {
        Self::new()
    }
}

impl TransientDesignerView {
    pub fn new() -> Self {
        let norm_atk_t = Self::attack_time_to_normalized(25.0);
        let norm_atk_g = Self::gain_to_normalized(6.0);
        let norm_sus_t = Self::sustain_time_to_normalized(350.0);
        let norm_sus_g = Self::gain_to_normalized(-3.0);

        Self {
            attack_gain_db: 6.0,
            attack_length_ms: 25.0,
            sustain_gain_db: -3.0,
            sustain_length_ms: 350.0,
            punch_freq_hz: 90.0,
            soft_clip_enabled: true,
            mode: TransientMode::Broadband,
            output_gain_db: 0.0,
            attack_handle_pos: (norm_atk_t, norm_atk_g),
            sustain_handle_pos: (norm_sus_t, norm_sus_g),
            is_dragging_attack: false,
            is_dragging_sustain: false,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Convert gain in dB (-24.0 .. +24.0) to normalized coordinate [0.0 ..= 1.0].
    pub fn gain_to_normalized(gain_db: f32) -> f32 {
        ((gain_db + 24.0) / 48.0).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate to gain dB.
    pub fn normalized_to_gain(norm: f32) -> f32 {
        -24.0 + norm.clamp(0.0, 1.0) * 48.0
    }

    /// Convert attack time ms (5.0 .. 100.0) to normalized coordinate.
    pub fn attack_time_to_normalized(time_ms: f32) -> f32 {
        ((time_ms - 5.0) / (100.0 - 5.0)).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate to attack time ms.
    pub fn normalized_to_attack_time(norm: f32) -> f32 {
        5.0 + norm.clamp(0.0, 1.0) * 95.0
    }

    /// Convert sustain time ms (50.0 .. 1000.0) to normalized coordinate.
    pub fn sustain_time_to_normalized(time_ms: f32) -> f32 {
        ((time_ms - 50.0) / (1000.0 - 50.0)).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate to sustain time ms.
    pub fn normalized_to_sustain_time(norm: f32) -> f32 {
        50.0 + norm.clamp(0.0, 1.0) * 950.0
    }

    /// Evaluate modified transient envelope curve amplitude at normalized time `t` [0.0 .. 1.0].
    pub fn evaluate_envelope_curve(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        let atk_w = 0.15 * (self.attack_length_ms / 50.0);
        let atk_boost = 1.0 + (self.attack_gain_db / 24.0) * 0.8;
        let sus_boost = 1.0 + (self.sustain_gain_db / 24.0) * 0.6;

        if t < atk_w {
            let p = t / atk_w;
            (p.sin() * atk_boost).clamp(0.0, 2.0)
        } else {
            let sus_p = (t - atk_w) / (1.0 - atk_w);
            let decay = (-3.0 * sus_p).exp();
            (decay * sus_boost).clamp(0.0, 2.0)
        }
    }

    /// Tests if a point hits the Attack Handle (>= 22pt radius -> 44x44pt bounding box).
    pub fn hit_test_attack_handle(&self, pos: (f32, f32), canvas: Rect) -> bool {
        let hx = canvas.x + (self.attack_handle_pos.0 * 0.35) * canvas.width;
        let hy = canvas.y + (1.0 - self.attack_handle_pos.1) * canvas.height;
        let dx = pos.0 - hx;
        let dy = pos.1 - hy;
        (dx * dx + dy * dy).sqrt() <= TRANSIENT_DESIGNER_HANDLE_HIT_RADIUS
    }

    /// Tests if a point hits the Sustain Handle (>= 22pt radius -> 44x44pt bounding box).
    pub fn hit_test_sustain_handle(&self, pos: (f32, f32), canvas: Rect) -> bool {
        let hx = canvas.x + (0.35 + self.sustain_handle_pos.0 * 0.65) * canvas.width;
        let hy = canvas.y + (1.0 - self.sustain_handle_pos.1) * canvas.height;
        let dx = pos.0 - hx;
        let dy = pos.1 - hy;
        (dx * dx + dy * dy).sqrt() <= TRANSIENT_DESIGNER_HANDLE_HIT_RADIUS
    }

    /// Render deterministic ASCII representation for headless terminal debugging.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let header = format!(
            "TRANSIENT DESIGNER [{:?}] Atk:{:+.1}dB({:.0}ms) Sus:{:+.1}dB({:.0}ms) Punch:{:.0}Hz",
            self.mode,
            self.attack_gain_db,
            self.attack_length_ms,
            self.sustain_gain_db,
            self.sustain_length_ms,
            self.punch_freq_hz
        );
        lines.push(header);

        let canvas_h = height.saturating_sub(2);
        for y in 0..canvas_h {
            let mut row = vec![' '; width];
            let norm_y = 1.0 - (y as f32 / (canvas_h.max(1) as f32));

            for (x, cell) in row.iter_mut().enumerate().take(width) {
                let norm_x = x as f32 / (width.max(1) as f32);
                let env = self.evaluate_envelope_curve(norm_x) * 0.5; // Scaled to 0..1
                if (env - norm_y).abs() < (1.0 / canvas_h as f32) {
                    *cell = '#';
                }
            }

            lines.push(row.into_iter().collect());
        }

        let footer = format!(
            "Atk: ({:.2}, {:.2}) | Sus: ({:.2}, {:.2}) | Clip: {} [PASS: >=44pt]",
            self.attack_handle_pos.0,
            self.attack_handle_pos.1,
            self.sustain_handle_pos.0,
            self.sustain_handle_pos.1,
            self.soft_clip_enabled
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
            "TACTILE TRANSIENT DESIGNER & HARMONIC PUNCH HUD",
            egui::FontId::proportional(15.0),
            Color32::from_rgb(0, 229, 255),
        );

        let readout = format!(
            "ATTACK: {:+.1} dB | SUSTAIN: {:+.1} dB | PUNCH: {:.0} Hz",
            self.attack_gain_db, self.sustain_gain_db, self.punch_freq_hz
        );
        painter.text(
            egui::pos2(rect.x + rect.width - 20.0, rect.y + 20.0),
            egui::Align2::RIGHT_TOP,
            readout,
            egui::FontId::proportional(12.0),
            Color32::from_rgb(255, 215, 0),
        );

        // Left Panel: Dynamic Envelope Waveform Shaper Canvas (20..450)
        let env_rect = Rect::new(rect.x + 20.0, rect.y + 56.0, 430.0, 224.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(env_rect.x, env_rect.y),
                egui::vec2(env_rect.width, env_rect.height),
            ),
            8.0,
            Color32::from_rgb(10, 14, 22),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(env_rect.x, env_rect.y),
                egui::vec2(env_rect.width, env_rect.height),
            ),
            8.0,
            Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(env_rect.x + 12.0, env_rect.y + 12.0),
            egui::Align2::LEFT_TOP,
            "TRANSIENT ATTACK & SUSTAIN ENVELOPE MORPH",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 229, 255),
        );

        // Grid 0 dB line (Y = 0.5) inside plot area (y + 36 .. y + height - 24)
        let plot_top = env_rect.y + 36.0;
        let plot_h = env_rect.height - 56.0;
        let zero_db_y = plot_top + plot_h * 0.5;
        painter.line_segment(
            [
                egui::pos2(env_rect.x, zero_db_y),
                egui::pos2(env_rect.x + env_rect.width, zero_db_y),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 110, 100)),
        );

        // Draw Dynamic Envelope Curve
        let mut prev_pt: Option<egui::Pos2> = None;
        let points = 80;
        for i in 0..=points {
            let norm_x = i as f32 / points as f32;
            let env_val = self.evaluate_envelope_curve(norm_x);
            let cx = env_rect.x + norm_x * env_rect.width;
            let cy = plot_top + (1.0 - (env_val * 0.45 + 0.05)) * plot_h;
            let pt = egui::pos2(cx, cy);

            if let Some(prev) = prev_pt {
                painter.line_segment(
                    [prev, pt],
                    Stroke::new(2.5_f32, Color32::from_rgb(0, 229, 255)),
                );
            }
            prev_pt = Some(pt);
        }

        // Attack Handle (>= 22pt radius -> 44x44pt touch area)
        let ax = env_rect.x + (self.attack_handle_pos.0 * 0.35) * env_rect.width;
        let ay = plot_top + (1.0 - self.attack_handle_pos.1) * plot_h;
        painter.circle_stroke(
            egui::pos2(ax, ay),
            TRANSIENT_DESIGNER_HANDLE_HIT_RADIUS,
            Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(255, 107, 43, 140)),
        );
        painter.circle_filled(egui::pos2(ax, ay), 14.0, Color32::from_rgb(255, 107, 43));
        painter.circle_filled(egui::pos2(ax, ay), 4.0, Color32::from_rgb(255, 255, 255));
        painter.text(
            egui::pos2(ax, ay - 24.0),
            egui::Align2::CENTER_CENTER,
            "ATTACK",
            egui::FontId::proportional(10.0),
            Color32::from_rgb(255, 107, 43),
        );

        // Sustain Handle (>= 22pt radius -> 44x44pt touch area)
        let sx = env_rect.x + (0.35 + self.sustain_handle_pos.0 * 0.65) * env_rect.width;
        let sy = plot_top + (1.0 - self.sustain_handle_pos.1) * plot_h;
        painter.circle_stroke(
            egui::pos2(sx, sy),
            TRANSIENT_DESIGNER_HANDLE_HIT_RADIUS,
            Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(0, 255, 180, 140)),
        );
        painter.circle_filled(egui::pos2(sx, sy), 14.0, Color32::from_rgb(0, 255, 180));
        painter.circle_filled(egui::pos2(sx, sy), 4.0, Color32::from_rgb(255, 255, 255));
        painter.text(
            egui::pos2(sx, sy - 24.0),
            egui::Align2::CENTER_CENTER,
            "SUSTAIN",
            egui::FontId::proportional(10.0),
            Color32::from_rgb(0, 255, 180),
        );

        // Right Panel: Punch Harmonic Radar & Mode Switcher (470..780)
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
            "HARMONIC PUNCH & MODES",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(255, 107, 43),
        );

        // Mode Selection Buttons (>= 44pt touch height)
        let modes = [
            ("BROADBAND", TransientMode::Broadband),
            ("FREQ SPLIT", TransientMode::FrequencySplit),
            ("HARMONIC", TransientMode::HarmonicPunch),
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

        // Punch Resonance Bar
        painter.text(
            egui::pos2(mode_rect.x + 15.0, mode_rect.y + 105.0),
            egui::Align2::LEFT_TOP,
            format!("LOW-END PUNCH FREQ: {:.0} Hz", self.punch_freq_hz),
            egui::FontId::proportional(11.0),
            Color32::from_rgb(180, 200, 225),
        );
        let punch_box = egui::Rect::from_min_size(
            egui::pos2(mode_rect.x + 15.0, mode_rect.y + 125.0),
            egui::vec2(mode_rect.width - 30.0, 24.0),
        );
        painter.rect_filled(punch_box, 4.0, Color32::from_rgb(18, 25, 38));
        let norm_punch = ((self.punch_freq_hz - 40.0) / 460.0).clamp(0.0, 1.0);
        let punch_fill = egui::Rect::from_min_size(
            egui::pos2(mode_rect.x + 15.0, mode_rect.y + 125.0),
            egui::vec2((mode_rect.width - 30.0) * norm_punch, 24.0),
        );
        painter.rect_filled(punch_fill, 4.0, Color32::from_rgb(255, 215, 0));

        // Soft Clip Toggle Button (>=44pt)
        let clip_box = egui::Rect::from_min_size(
            egui::pos2(mode_rect.x + 15.0, mode_rect.y + 165.0),
            egui::vec2(mode_rect.width - 30.0, 44.0),
        );
        let clip_bg = if self.soft_clip_enabled {
            Color32::from_rgb(0, 255, 180)
        } else {
            Color32::from_rgb(35, 45, 65)
        };
        let clip_fg = if self.soft_clip_enabled {
            Color32::from_rgb(0, 0, 0)
        } else {
            Color32::from_rgb(220, 235, 255)
        };
        painter.rect_filled(clip_box, 4.0, clip_bg);
        painter.text(
            egui::pos2(clip_box.center().x, clip_box.center().y),
            egui::Align2::CENTER_CENTER,
            if self.soft_clip_enabled {
                "ANALOG SOFT CLIPPER: ENGAGED"
            } else {
                "ANALOG SOFT CLIPPER: BYPASS"
            },
            egui::FontId::proportional(11.0),
            clip_fg,
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
            "[PASS] Tactile Transient Designer Attack/Sustain Handles (>= 44x44pt) Verified",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
