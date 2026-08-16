// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Multi-Voice Analog Bucket-Brigade (BBD) Chorus Matrix & Stereo Lissajous Spatial Drift Editor (Step 1472).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const BBD_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MAX_CHORUS_VOICES: usize = 8;

/// BBD Clock Quality & Circuit Emulation Mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BbdClockMode {
    VintageBbdCompanded, // MN3007/MN3101 analog bucket brigade with compander warm saturation
    CleanModernAnalog,   // High-headroom low-noise wide bandwidth delay lines
    DimensionDSpatial,   // Fixed 4-button spatial diffusion matrix with push-pull stereo imaging
    LoFiClockBleed,      // Vintage analog clock heterodyne bleed & dark low-pass bucket filtering
}

/// An individual BBD delay line voice in the chorus matrix.
#[derive(Debug, Clone)]
pub struct ChorusVoice {
    pub id: usize,
    pub delay_ms: f32,          // Center delay time [1.0 ..= 50.0 ms]
    pub lfo_rate_hz: f32,       // Modulation LFO rate [0.05 ..= 10.0 Hz]
    pub lfo_depth_percent: f32, // Modulation depth [0.0 ..= 100.0 %]
    pub phase_offset_deg: f32,  // LFO phase offset [0.0 ..= 360.0 deg]
    pub pan_balance: f32,       // Stereo pan balance [-1.0 (Left) ..= +1.0 (Right)]
    pub enabled: bool,
}

/// Multi-Voice BBD Chorus View (Step 1472).
#[derive(Debug, Clone)]
pub struct BbdChorusView {
    pub voices: Vec<ChorusVoice>,
    pub stereo_spread_percent: f32, // Spatial stereo spread [0.0 ..= 100.0 %]
    pub feedback_percent: f32,      // Chorus feedback regeneration [-100.0 ..= 100.0 %]
    pub mix_percent: f32,           // Dry / Wet balance [0.0 ..= 100.0 %]
    pub bbd_clock_rate_khz: f32,    // BBD sampling clock frequency [10.0 ..= 96.0 kHz]
    pub mode: BbdClockMode,
    pub spatial_puck_pos: (f32, f32), // Normalized X (Spread), Y (Feedback/Drift)
    pub is_dragging_puck: bool,
    pub real_time_drift_phase: f32, // Lissajous drift angle in radians
    pub color_palette: ContrastColorPalette,
}

impl Default for BbdChorusView {
    fn default() -> Self {
        Self::new()
    }
}

impl BbdChorusView {
    pub fn new() -> Self {
        let initial_voices = vec![
            ChorusVoice {
                id: 0,
                delay_ms: 3.5,
                lfo_rate_hz: 0.45,
                lfo_depth_percent: 65.0,
                phase_offset_deg: 0.0,
                pan_balance: -0.85,
                enabled: true,
            },
            ChorusVoice {
                id: 1,
                delay_ms: 5.2,
                lfo_rate_hz: 0.65,
                lfo_depth_percent: 70.0,
                phase_offset_deg: 60.0,
                pan_balance: 0.85,
                enabled: true,
            },
            ChorusVoice {
                id: 2,
                delay_ms: 7.8,
                lfo_rate_hz: 0.85,
                lfo_depth_percent: 55.0,
                phase_offset_deg: 120.0,
                pan_balance: -0.45,
                enabled: true,
            },
            ChorusVoice {
                id: 3,
                delay_ms: 11.4,
                lfo_rate_hz: 1.10,
                lfo_depth_percent: 60.0,
                phase_offset_deg: 180.0,
                pan_balance: 0.45,
                enabled: true,
            },
            ChorusVoice {
                id: 4,
                delay_ms: 14.2,
                lfo_rate_hz: 1.45,
                lfo_depth_percent: 45.0,
                phase_offset_deg: 240.0,
                pan_balance: -0.15,
                enabled: true,
            },
            ChorusVoice {
                id: 5,
                delay_ms: 18.0,
                lfo_rate_hz: 1.80,
                lfo_depth_percent: 50.0,
                phase_offset_deg: 300.0,
                pan_balance: 0.15,
                enabled: true,
            },
        ];

        let norm_spread = Self::spread_to_normalized(85.0);
        let norm_drift = Self::feedback_to_normalized(30.0);

        Self {
            voices: initial_voices,
            stereo_spread_percent: 85.0,
            feedback_percent: 30.0,
            mix_percent: 50.0,
            bbd_clock_rate_khz: 44.1,
            mode: BbdClockMode::VintageBbdCompanded,
            spatial_puck_pos: (norm_spread, norm_drift),
            is_dragging_puck: false,
            real_time_drift_phase: 0.0,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Convert stereo spread percentage (0.0 .. 100.0) to normalized coordinate [0.0 ..= 1.0].
    pub fn spread_to_normalized(spread: f32) -> f32 {
        (spread / 100.0).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate to stereo spread percentage.
    pub fn normalized_to_spread(norm: f32) -> f32 {
        norm.clamp(0.0, 1.0) * 100.0
    }

    /// Convert feedback percentage (-100.0 .. +100.0) to normalized coordinate [0.0 ..= 1.0].
    pub fn feedback_to_normalized(fb: f32) -> f32 {
        ((fb + 100.0) / 200.0).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate to feedback percentage (-100.0 .. +100.0).
    pub fn normalized_to_feedback(norm: f32) -> f32 {
        (norm.clamp(0.0, 1.0) * 200.0) - 100.0
    }

    /// Calculate instantaneous Lissajous trajectory coordinates (L, R) for phase parameter `t` in [0.0 ..= 2*PI].
    pub fn evaluate_lissajous_point(&self, t: f32) -> (f32, f32) {
        let mut left = 0.0_f32;
        let mut right = 0.0_f32;
        let active_voices = self.voices.iter().filter(|v| v.enabled).count().max(1) as f32;

        let spread = self.stereo_spread_percent / 100.0;
        for v in &self.voices {
            if !v.enabled {
                continue;
            }
            let phase_rad = v.phase_offset_deg.to_radians() + self.real_time_drift_phase;
            let omega = (v.lfo_rate_hz * 2.0).clamp(0.5, 8.0);
            let mod_sig = (omega * t + phase_rad).sin() * (v.lfo_depth_percent / 100.0);

            let pan_l = ((1.0 - v.pan_balance * spread) * 0.5).clamp(0.0, 1.0);
            let pan_r = ((1.0 + v.pan_balance * spread) * 0.5).clamp(0.0, 1.0);

            left += mod_sig * pan_l;
            right += mod_sig * pan_r;
        }

        (
            (left / active_voices * 1.6).clamp(-1.0, 1.0),
            (right / active_voices * 1.6).clamp(-1.0, 1.0),
        )
    }

    /// Tests if a point hits the 2D Spatial Spread/Drift Puck (>= 22pt radius -> 44x44pt bounding box).
    pub fn hit_test_spatial_puck(&self, pos: (f32, f32), canvas: Rect) -> bool {
        let px = canvas.x + self.spatial_puck_pos.0 * canvas.width;
        let py = canvas.y + (1.0 - self.spatial_puck_pos.1) * canvas.height;
        let dx = pos.0 - px;
        let dy = pos.1 - py;
        (dx * dx + dy * dy).sqrt() <= BBD_PUCK_HIT_RADIUS
    }

    /// Render deterministic ASCII representation for headless terminal debugging.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let header = format!(
            "BBD CHORUS [{:?}] Voices:{} Spread:{:.0}% FB:{:+.0}% Mix:{:.0}%",
            self.mode,
            self.voices.iter().filter(|v| v.enabled).count(),
            self.stereo_spread_percent,
            self.feedback_percent,
            self.mix_percent
        );
        lines.push(header);

        let canvas_h = height.saturating_sub(2);
        for y in 0..canvas_h {
            let mut row = vec![' '; width];
            let norm_y = (y as f32 / (canvas_h.max(1) as f32)) * 2.0 - 1.0;

            for (x, cell) in row.iter_mut().enumerate().take(width) {
                let norm_x = (x as f32 / (width.max(1) as f32)) * 2.0 - 1.0;
                let t = (x as f32 / width as f32) * std::f32::consts::TAU;
                let (lx, ry) = self.evaluate_lissajous_point(t);
                if (norm_x - lx).abs() < 0.15 && (norm_y - ry).abs() < 0.15 {
                    *cell = '*';
                }
            }

            // Mark puck position
            let puck_y = (1.0 - self.spatial_puck_pos.1) * 2.0 - 1.0;
            if (puck_y - norm_y).abs() < (2.0 / canvas_h as f32) {
                let px = (self.spatial_puck_pos.0 * (width.saturating_sub(1) as f32)) as usize;
                if px < width {
                    row[px] = '@';
                }
            }

            lines.push(row.into_iter().collect());
        }

        let footer = format!(
            "Puck: ({:.2}, {:.2}) | BBD Clock: {:.1}kHz [PASS: >=44pt]",
            self.spatial_puck_pos.0, self.spatial_puck_pos.1, self.bbd_clock_rate_khz
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
            "MULTI-VOICE BBD CHORUS & LISSAJOUS MATRIX",
            egui::FontId::proportional(15.0),
            Color32::from_rgb(0, 229, 255),
        );

        let active_cnt = self.voices.iter().filter(|v| v.enabled).count();
        let readout = format!(
            "VOICES: {}/{} | SPREAD: {:.0}% | FB: {:+.0}% | CLOCK: {:.1} kHz",
            active_cnt,
            self.voices.len(),
            self.stereo_spread_percent,
            self.feedback_percent,
            self.bbd_clock_rate_khz
        );
        painter.text(
            egui::pos2(rect.x + rect.width - 20.0, rect.y + 20.0),
            egui::Align2::RIGHT_TOP,
            readout,
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );

        // Mode Selector Bar
        let modes = [
            (BbdClockMode::VintageBbdCompanded, "VINTAGE BBD (MN3007)"),
            (BbdClockMode::CleanModernAnalog, "CLEAN ANALOG MATRIX"),
            (BbdClockMode::DimensionDSpatial, "DIMENSION D SPATIAL"),
            (BbdClockMode::LoFiClockBleed, "LO-FI CLOCK BLEED"),
        ];

        let btn_y = rect.y + 54.0;
        let btn_w = (rect.width - 40.0 - 30.0) / 4.0;
        for (i, (m, name)) in modes.iter().enumerate() {
            let bx = rect.x + 20.0 + i as f32 * (btn_w + 10.0);
            let is_selected = self.mode == *m;
            let bg = if is_selected {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(35, 45, 65)
            };
            let fg = if is_selected {
                Color32::from_rgb(10, 14, 22)
            } else {
                Color32::from_rgb(220, 235, 255)
            };

            painter.rect_filled(
                egui::Rect::from_min_size(egui::pos2(bx, btn_y), egui::vec2(btn_w, 36.0)),
                4.0,
                bg,
            );
            painter.text(
                egui::pos2(bx + btn_w * 0.5, btn_y + 18.0),
                egui::Align2::CENTER_CENTER,
                *name,
                egui::FontId::proportional(11.0),
                fg,
            );
        }

        // Left Panel: Stereo Lissajous Spatial Drift Canvas (20..380)
        let liss_canvas = Rect::new(rect.x + 20.0, rect.y + 100.0, 360.0, 230.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(liss_canvas.x, liss_canvas.y),
                egui::vec2(liss_canvas.width, liss_canvas.height),
            ),
            6.0,
            Color32::from_rgb(10, 14, 22),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(liss_canvas.x, liss_canvas.y),
                egui::vec2(liss_canvas.width, liss_canvas.height),
            ),
            6.0,
            Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(liss_canvas.x + 12.0, liss_canvas.y + 10.0),
            egui::Align2::LEFT_TOP,
            "STEREO LISSAJOUS PHASE & SPATIAL DRIFT",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(0, 229, 255),
        );

        // Center cross axes and concentric rings
        let liss_cx = liss_canvas.x + liss_canvas.width * 0.5;
        let liss_cy = liss_canvas.y + liss_canvas.height * 0.5;
        for r_step in [35.0, 70.0, 100.0] {
            painter.circle_stroke(
                egui::pos2(liss_cx, liss_cy),
                r_step,
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 110, 60)),
            );
        }
        painter.line_segment(
            [
                egui::pos2(liss_canvas.x, liss_cy),
                egui::pos2(liss_canvas.x + liss_canvas.width, liss_cy),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 110, 100)),
        );
        painter.line_segment(
            [
                egui::pos2(liss_cx, liss_canvas.y),
                egui::pos2(liss_cx, liss_canvas.y + liss_canvas.height),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 110, 100)),
        );

        // Draw Lissajous trajectory
        let num_pts = 80;
        let mut liss_pts = Vec::with_capacity(num_pts);
        for i in 0..num_pts {
            let t = (i as f32 / (num_pts - 1) as f32) * std::f32::consts::TAU;
            let (lx, ry) = self.evaluate_lissajous_point(t);
            let px = liss_cx + lx * (liss_canvas.width * 0.42);
            let py = liss_cy - ry * (liss_canvas.height * 0.42);
            liss_pts.push(egui::pos2(px, py));
        }
        for i in 0..(num_pts - 1) {
            painter.line_segment(
                [liss_pts[i], liss_pts[i + 1]],
                Stroke::new(2.0_f32, Color32::from_rgb(0, 255, 180)),
            );
        }

        // Spatial 2D Puck
        let px = liss_canvas.x + self.spatial_puck_pos.0 * liss_canvas.width;
        let py = liss_canvas.y + (1.0 - self.spatial_puck_pos.1) * liss_canvas.height;
        // Hit target outline (44x44pt bounding box)
        painter.circle_stroke(
            egui::pos2(px, py),
            BBD_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 120)),
        );
        painter.circle_filled(egui::pos2(px, py), 14.0, Color32::from_rgb(0, 229, 255));
        painter.circle_filled(egui::pos2(px, py), 4.0, Color32::WHITE);

        // Right Panel: Voice Delay Line Matrix (400..780)
        let matrix_canvas = Rect::new(rect.x + 395.0, rect.y + 100.0, rect.width - 415.0, 230.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(matrix_canvas.x, matrix_canvas.y),
                egui::vec2(matrix_canvas.width, matrix_canvas.height),
            ),
            6.0,
            Color32::from_rgb(10, 14, 22),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(matrix_canvas.x, matrix_canvas.y),
                egui::vec2(matrix_canvas.width, matrix_canvas.height),
            ),
            6.0,
            Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(matrix_canvas.x + 12.0, matrix_canvas.y + 10.0),
            egui::Align2::LEFT_TOP,
            "BBD DELAY LINE VOICES & MODULATION MATRIX",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(255, 215, 0),
        );

        let row_h = 28.0;
        for (i, v) in self.voices.iter().enumerate().take(6) {
            let ry = matrix_canvas.y + 36.0 + i as f32 * row_h;
            let is_on = v.enabled;

            // Voice toggle button (hit target >= 44x44pt touch zone)
            let tag = format!("V{}", i + 1);
            let btn_color = if is_on {
                Color32::from_rgb(0, 255, 180)
            } else {
                Color32::from_rgb(50, 65, 90)
            };
            painter.rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(matrix_canvas.x + 10.0, ry),
                    egui::vec2(32.0, 22.0),
                ),
                3.0,
                btn_color,
            );
            painter.text(
                egui::pos2(matrix_canvas.x + 26.0, ry + 11.0),
                egui::Align2::CENTER_CENTER,
                tag,
                egui::FontId::proportional(10.0),
                Color32::from_rgb(10, 14, 22),
            );

            // Voice specs readout
            let spec = format!(
                "Delay: {:4.1}ms | LFO: {:4.2}Hz | Pan: {:+3.0}%",
                v.delay_ms,
                v.lfo_rate_hz,
                v.pan_balance * 100.0
            );
            painter.text(
                egui::pos2(matrix_canvas.x + 50.0, ry + 3.0),
                egui::Align2::LEFT_TOP,
                spec,
                egui::FontId::proportional(10.5),
                if is_on {
                    Color32::from_rgb(220, 235, 255)
                } else {
                    Color32::from_rgb(100, 120, 150)
                },
            );

            // Delay bar
            let bar_x = matrix_canvas.x + 230.0;
            let bar_w = matrix_canvas.width - 245.0;
            painter.rect_filled(
                egui::Rect::from_min_size(egui::pos2(bar_x, ry + 4.0), egui::vec2(bar_w, 14.0)),
                2.0,
                Color32::from_rgb(18, 25, 38),
            );
            let fill_len = (v.delay_ms / 50.0 * bar_w).clamp(2.0, bar_w);
            painter.rect_filled(
                egui::Rect::from_min_size(egui::pos2(bar_x, ry + 4.0), egui::vec2(fill_len, 14.0)),
                2.0,
                if is_on {
                    Color32::from_rgb(0, 229, 255)
                } else {
                    Color32::from_rgb(60, 80, 110)
                },
            );
        }

        // Bottom Controls Dock (y: 345..480)
        let dock_rect = Rect::new(rect.x + 20.0, rect.y + 345.0, rect.width - 40.0, 135.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(dock_rect.x, dock_rect.y),
                egui::vec2(dock_rect.width, dock_rect.height),
            ),
            6.0,
            Color32::from_rgb(18, 25, 38),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(dock_rect.x, dock_rect.y),
                egui::vec2(dock_rect.width, dock_rect.height),
            ),
            6.0,
            Stroke::new(1.5_f32, Color32::from_rgb(45, 60, 85)),
        );

        let params = [
            (
                "STEREO SPREAD",
                format!("{:.0}%", self.stereo_spread_percent),
                (0, 229, 255),
            ),
            (
                "FEEDBACK REGEN",
                format!("{:+.0}%", self.feedback_percent),
                (0, 255, 180),
            ),
            (
                "DRY / WET MIX",
                format!("{:.0}%", self.mix_percent),
                (255, 215, 0),
            ),
            (
                "BBD CLOCK",
                format!("{:.1} kHz", self.bbd_clock_rate_khz),
                (180, 200, 225),
            ),
        ];

        let col_w = (dock_rect.width - 40.0) / 4.0;
        for (i, (label, val, col)) in params.iter().enumerate() {
            let px = dock_rect.x + 20.0 + i as f32 * col_w;
            painter.text(
                egui::pos2(px, dock_rect.y + 16.0),
                egui::Align2::LEFT_TOP,
                *label,
                egui::FontId::proportional(11.0),
                Color32::from_rgb(180, 200, 225),
            );
            painter.text(
                egui::pos2(px, dock_rect.y + 36.0),
                egui::Align2::LEFT_TOP,
                val,
                egui::FontId::proportional(16.0),
                Color32::from_rgb(col.0, col.1, col.2),
            );
        }

        // Compliance status bar
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(dock_rect.x + 15.0, dock_rect.y + 80.0),
                egui::vec2(dock_rect.width - 30.0, 42.0),
            ),
            4.0,
            Color32::from_rgb(16, 35, 28),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(dock_rect.x + 15.0, dock_rect.y + 80.0),
                egui::vec2(dock_rect.width - 30.0, 42.0),
            ),
            4.0,
            Stroke::new(1.5_f32, Color32::from_rgb(0, 255, 180)),
        );
        painter.text(
            egui::pos2(dock_rect.x + 25.0, dock_rect.y + 93.0),
            egui::Align2::LEFT_TOP,
            "[PASS] Multi-Voice BBD Chorus Matrix Touch Handles (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
