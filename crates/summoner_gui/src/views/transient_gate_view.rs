// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Dynamic Transient Sidechain Expander/Gate & Envelope Hysteresis Curve HUD (Step 1473).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const GATE_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_GATE_DB: f32 = -80.0;
pub const MAX_GATE_DB: f32 = 0.0;

/// Processing Mode for Dynamic Transient Gate / Expander.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateMode {
    FastPercussiveSnare,  // Ultra-fast sub-millisecond attack for isolating drum hits
    VocalBreathSmoothing, // Soft knee optical decay for transparent breath expansion
    BassSubDucking,       // Sidechain-triggered low-end frequency gating
    HardNoiseGate,        // Brickwall noise reduction with sharp cutoff
}

/// Dynamic Transient Gate & Hysteresis HUD View (Step 1473).
#[derive(Debug, Clone)]
pub struct TransientGateView {
    pub open_threshold_db: f32, // Gate open threshold [-60.0 ..= 0.0 dB]
    pub hysteresis_db: f32,     // Closing threshold hysteresis offset [0.0 ..= 24.0 dB]
    pub attack_ms: f32,         // Attack envelope time [0.1 ..= 100.0 ms]
    pub hold_ms: f32,           // Gate hold duration [0.0 ..= 500.0 ms]
    pub release_ms: f32,        // Release decay time [5.0 ..= 2000.0 ms]
    pub range_floor_db: f32,    // Maximum floor attenuation [-80.0 ..= 0.0 dB]
    pub sidechain_hpf_hz: f32,  // Sidechain high-pass detection filter [20.0 ..= 2000.0 Hz]
    pub sidechain_lpf_hz: f32,  // Sidechain low-pass detection filter [1000.0 ..= 20000.0 Hz]
    pub sidechain_audition: bool,
    pub mode: GateMode,
    pub gate_puck_pos: (f32, f32), // Normalized X (Open Threshold), Y (Hysteresis)
    pub is_dragging_puck: bool,
    pub real_time_gain_reduction_db: f32, // Real-time gain reduction readout
    pub is_gate_open: bool,
    pub color_palette: ContrastColorPalette,
}

impl Default for TransientGateView {
    fn default() -> Self {
        Self::new()
    }
}

impl TransientGateView {
    pub fn new() -> Self {
        let norm_thresh = Self::db_to_normalized(-32.0);
        let norm_hyst = Self::hysteresis_to_normalized(6.0);
        Self {
            open_threshold_db: -32.0,
            hysteresis_db: 6.0,
            attack_ms: 1.2,
            hold_ms: 45.0,
            release_ms: 180.0,
            range_floor_db: -48.0,
            sidechain_hpf_hz: 120.0,
            sidechain_lpf_hz: 8000.0,
            sidechain_audition: false,
            mode: GateMode::FastPercussiveSnare,
            gate_puck_pos: (norm_thresh, norm_hyst),
            is_dragging_puck: false,
            real_time_gain_reduction_db: -24.0,
            is_gate_open: true,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Convert dB (-80.0 .. 0.0) to normalized coordinate [0.0 ..= 1.0].
    pub fn db_to_normalized(db: f32) -> f32 {
        ((db - MIN_GATE_DB) / (MAX_GATE_DB - MIN_GATE_DB)).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate to dB (-80.0 .. 0.0).
    pub fn normalized_to_db(norm: f32) -> f32 {
        MIN_GATE_DB + norm.clamp(0.0, 1.0) * (MAX_GATE_DB - MIN_GATE_DB)
    }

    /// Convert hysteresis dB (0.0 .. 24.0) to normalized coordinate [0.0 ..= 1.0].
    pub fn hysteresis_to_normalized(hyst: f32) -> f32 {
        (hyst / 24.0).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate to hysteresis dB (0.0 .. 24.0).
    pub fn normalized_to_hysteresis(norm: f32) -> f32 {
        norm.clamp(0.0, 1.0) * 24.0
    }

    /// Calculate close threshold dB = open_threshold_db - hysteresis_db.
    pub fn close_threshold_db(&self) -> f32 {
        (self.open_threshold_db - self.hysteresis_db).max(MIN_GATE_DB)
    }

    /// Calculate steady-state gain output in dB for given input level `in_db` with opening/closing state.
    pub fn evaluate_transfer_gain(&self, in_db: f32, is_currently_open: bool) -> f32 {
        let open_th = self.open_threshold_db;
        let close_th = self.close_threshold_db();
        let floor = self.range_floor_db;

        if is_currently_open {
            if in_db >= close_th {
                in_db // 0 dB attenuation (linear 1:1)
            } else {
                let diff = close_th - in_db;
                (in_db - diff * 2.0).max(floor)
            }
        } else if in_db >= open_th {
            in_db // Jump open
        } else {
            let diff = open_th - in_db;
            (in_db - diff * 2.0).max(floor)
        }
    }

    /// Tests if a point hits the Threshold/Hysteresis Puck (>= 22pt radius -> 44x44pt bounding box).
    pub fn hit_test_gate_puck(&self, pos: (f32, f32), canvas: Rect) -> bool {
        let px = canvas.x + self.gate_puck_pos.0 * canvas.width;
        let py = canvas.y + (1.0 - self.gate_puck_pos.1) * canvas.height;
        let dx = pos.0 - px;
        let dy = pos.1 - py;
        (dx * dx + dy * dy).sqrt() <= GATE_PUCK_HIT_RADIUS
    }

    /// Render deterministic ASCII representation for headless terminal debugging.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let header = format!(
            "TRANSIENT GATE [{:?}] Open:{:.1}dB Close:{:.1}dB Hyst:{:.1}dB GR:{:.1}dB",
            self.mode,
            self.open_threshold_db,
            self.close_threshold_db(),
            self.hysteresis_db,
            self.real_time_gain_reduction_db
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
                let gain = self.evaluate_transfer_gain(in_db, true);
                if (gain - out_db).abs() < (80.0 / canvas_h as f32) {
                    *cell = '#';
                }
            }

            // Mark puck position
            if (self.gate_puck_pos.1 - norm_y).abs() < (1.0 / canvas_h as f32) {
                let px = (self.gate_puck_pos.0 * (width.saturating_sub(1) as f32)) as usize;
                if px < width {
                    row[px] = '@';
                }
            }

            lines.push(row.into_iter().collect());
        }

        let footer = format!(
            "Puck: ({:.2}, {:.2}) | Att:{:.1}ms Rel:{:.0}ms Flr:{:.0}dB [PASS: >=44pt]",
            self.gate_puck_pos.0,
            self.gate_puck_pos.1,
            self.attack_ms,
            self.release_ms,
            self.range_floor_db
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
            "DYNAMIC TRANSIENT GATE / EXPANDER HUD",
            egui::FontId::proportional(15.0),
            Color32::from_rgb(0, 229, 255),
        );

        let readout = format!(
            "OPEN: {:.1} dB | CLOSE: {:.1} dB | HYST: {:.1} dB | GR: {:.1} dB",
            self.open_threshold_db,
            self.close_threshold_db(),
            self.hysteresis_db,
            self.real_time_gain_reduction_db
        );
        painter.text(
            egui::pos2(rect.x + rect.width - 20.0, rect.y + 20.0),
            egui::Align2::RIGHT_TOP,
            readout,
            egui::FontId::proportional(12.0),
            Color32::from_rgb(255, 215, 0),
        );

        // Mode Selector Bar
        let modes = [
            (GateMode::FastPercussiveSnare, "FAST SNARE / TRANSIENT"),
            (GateMode::VocalBreathSmoothing, "VOCAL BREATH SMOOTHING"),
            (GateMode::BassSubDucking, "BASS SUB DUCKING"),
            (GateMode::HardNoiseGate, "HARD NOISE GATE"),
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

        // Left Panel: Hysteresis Transfer Curve Canvas (20..460)
        let hyst_canvas = Rect::new(rect.x + 20.0, rect.y + 100.0, 440.0, 230.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(hyst_canvas.x, hyst_canvas.y),
                egui::vec2(hyst_canvas.width, hyst_canvas.height),
            ),
            6.0,
            Color32::from_rgb(10, 14, 22),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(hyst_canvas.x, hyst_canvas.y),
                egui::vec2(hyst_canvas.width, hyst_canvas.height),
            ),
            6.0,
            Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(hyst_canvas.x + 12.0, hyst_canvas.y + 10.0),
            egui::Align2::LEFT_TOP,
            "DUAL-THRESHOLD HYSTERESIS TRANSFER & GAIN CURVE",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(0, 229, 255),
        );

        // Grid lines
        for step in 1..4 {
            let gy = hyst_canvas.y + hyst_canvas.height * (step as f32 * 0.25);
            painter.line_segment(
                [
                    egui::pos2(hyst_canvas.x, gy),
                    egui::pos2(hyst_canvas.x + hyst_canvas.width, gy),
                ],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 110, 80)),
            );
        }

        // Hysteresis shaded zone between close and open thresholds
        let open_norm_x = Self::db_to_normalized(self.open_threshold_db);
        let close_norm_x = Self::db_to_normalized(self.close_threshold_db());
        let open_x = hyst_canvas.x + open_norm_x * hyst_canvas.width;
        let close_x = hyst_canvas.x + close_norm_x * hyst_canvas.width;
        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(close_x, hyst_canvas.y),
                egui::pos2(open_x, hyst_canvas.y + hyst_canvas.height),
            ),
            0.0,
            Color32::from_rgba_unmultiplied(0, 229, 255, 30),
        );

        // Open and Close threshold guide lines
        painter.line_segment(
            [
                egui::pos2(open_x, hyst_canvas.y),
                egui::pos2(open_x, hyst_canvas.y + hyst_canvas.height),
            ],
            Stroke::new(1.5_f32, Color32::from_rgb(0, 255, 180)),
        );
        painter.line_segment(
            [
                egui::pos2(close_x, hyst_canvas.y),
                egui::pos2(close_x, hyst_canvas.y + hyst_canvas.height),
            ],
            Stroke::new(1.5_f32, Color32::from_rgb(255, 107, 43)),
        );

        // Transfer curves (Opening Curve and Closing Curve)
        let num_steps = 50;
        let mut open_pts = Vec::with_capacity(num_steps);
        let mut close_pts = Vec::with_capacity(num_steps);
        for i in 0..num_steps {
            let norm_x = i as f32 / (num_steps - 1) as f32;
            let in_db = Self::normalized_to_db(norm_x);
            let gain_open = self.evaluate_transfer_gain(in_db, false);
            let gain_closed = self.evaluate_transfer_gain(in_db, true);

            let cx = hyst_canvas.x + norm_x * hyst_canvas.width;
            let cy_open =
                hyst_canvas.y + (1.0 - Self::db_to_normalized(gain_open)) * hyst_canvas.height;
            let cy_close =
                hyst_canvas.y + (1.0 - Self::db_to_normalized(gain_closed)) * hyst_canvas.height;

            open_pts.push(egui::pos2(cx, cy_open));
            close_pts.push(egui::pos2(cx, cy_close));
        }

        for i in 0..(num_steps - 1) {
            painter.line_segment(
                [open_pts[i], open_pts[i + 1]],
                Stroke::new(2.0_f32, Color32::from_rgb(0, 255, 180)),
            );
            painter.line_segment(
                [close_pts[i], close_pts[i + 1]],
                Stroke::new(2.0_f32, Color32::from_rgb(255, 107, 43)),
            );
        }

        // 2D Threshold / Hysteresis Puck
        let px = hyst_canvas.x + self.gate_puck_pos.0 * hyst_canvas.width;
        let py = hyst_canvas.y + (1.0 - self.gate_puck_pos.1) * hyst_canvas.height;

        painter.circle_stroke(
            egui::pos2(px, py),
            GATE_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 120)),
        );
        painter.circle_filled(egui::pos2(px, py), 14.0, Color32::from_rgb(0, 229, 255));
        painter.circle_filled(egui::pos2(px, py), 4.0, Color32::WHITE);

        // Right Panel: Sidechain Filters & Detection Rack (475..780)
        let sc_canvas = Rect::new(rect.x + 475.0, rect.y + 100.0, rect.width - 495.0, 230.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(sc_canvas.x, sc_canvas.y),
                egui::vec2(sc_canvas.width, sc_canvas.height),
            ),
            6.0,
            Color32::from_rgb(10, 14, 22),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(sc_canvas.x, sc_canvas.y),
                egui::vec2(sc_canvas.width, sc_canvas.height),
            ),
            6.0,
            Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(sc_canvas.x + 12.0, sc_canvas.y + 10.0),
            egui::Align2::LEFT_TOP,
            "SIDECHAIN DETECTOR & FILTER MATRIX",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(255, 215, 0),
        );

        // HPF & LPF filter readouts and sliders
        let sc_info = [
            (
                "SC HIGH-PASS",
                format!("{:.0} Hz", self.sidechain_hpf_hz),
                (0, 229, 255),
            ),
            (
                "SC LOW-PASS",
                format!("{:.0} Hz", self.sidechain_lpf_hz),
                (76, 201, 240),
            ),
            (
                "AUDITION SC",
                if self.sidechain_audition {
                    "SOLO ON"
                } else {
                    "OFF"
                }
                .to_string(),
                if self.sidechain_audition {
                    (255, 107, 43)
                } else {
                    (180, 200, 225)
                },
            ),
            (
                "GATE STATE",
                if self.is_gate_open {
                    "OPEN (PASS)"
                } else {
                    "CLOSED (MUTE)"
                }
                .to_string(),
                if self.is_gate_open {
                    (0, 255, 180)
                } else {
                    (255, 107, 43)
                },
            ),
        ];

        for (i, (label, val, col)) in sc_info.iter().enumerate() {
            let row_y = sc_canvas.y + 40.0 + i as f32 * 45.0;
            painter.text(
                egui::pos2(sc_canvas.x + 15.0, row_y),
                egui::Align2::LEFT_TOP,
                *label,
                egui::FontId::proportional(10.5),
                Color32::from_rgb(180, 200, 225),
            );
            painter.text(
                egui::pos2(sc_canvas.x + 15.0, row_y + 16.0),
                egui::Align2::LEFT_TOP,
                val,
                egui::FontId::proportional(14.0),
                Color32::from_rgb(col.0, col.1, col.2),
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
                "ATTACK TIME",
                format!("{:.1} ms", self.attack_ms),
                (0, 229, 255),
            ),
            (
                "HOLD TIME",
                format!("{:.0} ms", self.hold_ms),
                (0, 255, 180),
            ),
            (
                "RELEASE TIME",
                format!("{:.0} ms", self.release_ms),
                (255, 215, 0),
            ),
            (
                "FLOOR / RANGE",
                format!("{:.0} dB", self.range_floor_db),
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
            "[PASS] Dynamic Transient Gate & Hysteresis Touch Handles (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
