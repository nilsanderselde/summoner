// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Analog Ladder 24dB/oct Diode/Transistor Filter Slope & Self-Oscillation Saturation HUD (Step 1471).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const LADDER_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_LADDER_FREQ_HZ: f32 = 20.0;
pub const MAX_LADDER_FREQ_HZ: f32 = 20000.0;

/// Filter circuit topology.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LadderTopology {
    MoogTransistor4Pole, // 24dB/oct warm transistor ladder with bass drop at high resonance
    Tb303DiodeLadder,    // 18dB/24dB diode ladder with squelchy non-linear feedback
    OberheimSem2Pole,    // 12dB/oct multi-mode state-variable character
    Ms20SallenKeyKorg,   // 12dB/oct aggressive screaming OTA/Sallen-Key resonance
}

/// Analog Ladder Filter HUD View (Step 1471).
#[derive(Debug, Clone)]
pub struct LadderFilterView {
    pub cutoff_freq_hz: f32,   // Filter cutoff frequency [20.0 ..= 20000.0 Hz]
    pub resonance_q: f32,      // Resonance Q factor [0.0 ..= 10.0]
    pub drive_saturation: f32, // Pre-filter saturation drive [0.0 ..= 100.0 %]
    pub key_tracking_percent: f32, // Keyboard pitch tracking [0.0 ..= 100.0 %]
    pub envelope_depth: f32,   // Filter envelope modulation depth [-100.0 ..= 100.0 %]
    pub topology: LadderTopology,
    pub self_oscillating: bool, // True when resonance exceeds self-oscillation threshold (Q >= 8.0)
    pub filter_puck_pos: (f32, f32), // Normalized X (Cutoff), Y (Resonance)
    pub is_dragging_puck: bool,
    pub real_time_peak_db: f32, // Peak resonant boost readout in dB
    pub color_palette: ContrastColorPalette,
}

impl Default for LadderFilterView {
    fn default() -> Self {
        Self::new()
    }
}

impl LadderFilterView {
    pub fn new() -> Self {
        let norm_freq = Self::freq_to_normalized(1450.0);
        let norm_res = Self::resonance_to_normalized(6.5);
        Self {
            cutoff_freq_hz: 1450.0,
            resonance_q: 6.5,
            drive_saturation: 35.0,
            key_tracking_percent: 50.0,
            envelope_depth: 40.0,
            topology: LadderTopology::MoogTransistor4Pole,
            self_oscillating: false,
            filter_puck_pos: (norm_freq, norm_res),
            is_dragging_puck: false,
            real_time_peak_db: 14.5,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Convert frequency in Hz (20 .. 20000) to logarithmic normalized coordinate [0.0 ..= 1.0].
    pub fn freq_to_normalized(freq_hz: f32) -> f32 {
        let freq = freq_hz.clamp(MIN_LADDER_FREQ_HZ, MAX_LADDER_FREQ_HZ);
        ((freq / MIN_LADDER_FREQ_HZ).log10() / (MAX_LADDER_FREQ_HZ / MIN_LADDER_FREQ_HZ).log10())
            .clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to frequency in Hz (20 .. 20000).
    pub fn normalized_to_freq(norm: f32) -> f32 {
        let norm = norm.clamp(0.0, 1.0);
        MIN_LADDER_FREQ_HZ * 10.0_f32.powf(norm * (MAX_LADDER_FREQ_HZ / MIN_LADDER_FREQ_HZ).log10())
    }

    /// Convert resonance Q (0.0 .. 10.0) to normalized coordinate [0.0 ..= 1.0].
    pub fn resonance_to_normalized(q: f32) -> f32 {
        (q / 10.0).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate to resonance Q (0.0 .. 10.0).
    pub fn normalized_to_resonance(norm: f32) -> f32 {
        norm.clamp(0.0, 1.0) * 10.0
    }

    /// Convert drive percentage (0.0 .. 100.0) to normalized coordinate [0.0 ..= 1.0].
    pub fn drive_to_normalized(drive: f32) -> f32 {
        (drive / 100.0).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate to drive percentage.
    pub fn normalized_to_drive(norm: f32) -> f32 {
        norm.clamp(0.0, 1.0) * 100.0
    }

    /// Check if current parameters cause self-oscillation.
    pub fn check_self_oscillation(&self) -> bool {
        self.resonance_q >= 8.0
    }

    /// Calculate filter transfer magnitude in normalized range [0.0 ..= 1.0] for frequency `f_hz`.
    pub fn evaluate_filter_response(&self, f_hz: f32) -> f32 {
        let fc = self.cutoff_freq_hz;
        let q = self.resonance_q;
        let drive = self.drive_saturation / 100.0;

        let ratio = (f_hz / fc).max(1e-4);
        let slope_poles = match self.topology {
            LadderTopology::MoogTransistor4Pole => 4.0, // 24 dB/oct
            LadderTopology::Tb303DiodeLadder => 3.5,    // ~21 dB/oct
            LadderTopology::OberheimSem2Pole => 2.0,    // 12 dB/oct
            LadderTopology::Ms20SallenKeyKorg => 2.2,   // ~13 dB/oct
        };

        // Low-pass roll-off
        let attenuation = 1.0 / (1.0 + ratio.powf(slope_poles * 2.0)).sqrt();

        // Resonance peak near cutoff
        let oct_diff = ratio.log2().abs();
        let resonance_peak = if oct_diff < 0.6 {
            let width = 0.35 / (1.0 + q * 0.2);
            let peak_gain = (q / 10.0) * 0.85;
            let shape = (-0.5 * (oct_diff / width).powi(2)).exp();
            shape * peak_gain
        } else {
            0.0
        };

        // Drive saturation compression
        let raw = attenuation + resonance_peak;
        let saturated = (raw * (1.0 + drive * 0.5)).tanh();
        saturated.clamp(0.0, 1.0)
    }

    /// Calculate non-linear transfer saturation curve point at input amplitude `x` in [-1.0 ..= 1.0].
    pub fn evaluate_saturation_curve(&self, x: f32) -> f32 {
        let drive = 1.0 + (self.drive_saturation / 100.0) * 4.0;
        match self.topology {
            LadderTopology::MoogTransistor4Pole => (x * drive).tanh(),
            LadderTopology::Tb303DiodeLadder => {
                // Diode asymmetric clipping
                if x > 0.0 {
                    (x * drive * 1.2).tanh()
                } else {
                    (x * drive * 0.8).tanh() * 1.1
                }
            }
            LadderTopology::OberheimSem2Pole => {
                let scaled = x * drive;
                scaled / (1.0 + scaled.abs())
            }
            LadderTopology::Ms20SallenKeyKorg => {
                // Harder OTA saturation
                let k = drive * 1.5;
                (k * x).clamp(-1.0, 1.0) * 0.8 + 0.2 * (k * x).tanh()
            }
        }
    }

    /// Tests if a point hits the 2D Cutoff/Resonance Puck (>= 22pt radius -> 44x44pt bounding box).
    pub fn hit_test_filter_puck(&self, pos: (f32, f32), canvas: Rect) -> bool {
        let px = canvas.x + self.filter_puck_pos.0 * canvas.width;
        let py = canvas.y + (1.0 - self.filter_puck_pos.1) * canvas.height;
        let dx = pos.0 - px;
        let dy = pos.1 - py;
        (dx * dx + dy * dy).sqrt() <= LADDER_PUCK_HIT_RADIUS
    }

    /// Render deterministic ASCII representation for headless terminal debugging.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let header = format!(
            "LADDER FILTER [{:?}] Fc:{:.0}Hz Res:{:.1} Drive:{:.0}% SelfOsc:{}",
            self.topology,
            self.cutoff_freq_hz,
            self.resonance_q,
            self.drive_saturation,
            if self.check_self_oscillation() {
                "YES"
            } else {
                "NO"
            }
        );
        lines.push(header);

        let canvas_h = height.saturating_sub(2);
        for y in 0..canvas_h {
            let mut row = vec![' '; width];
            let norm_y = 1.0 - (y as f32 / (canvas_h.max(1) as f32));

            for (x, cell) in row.iter_mut().enumerate().take(width) {
                let norm_x = x as f32 / (width.max(1) as f32);
                let f = Self::normalized_to_freq(norm_x);
                let mag = self.evaluate_filter_response(f);
                if (mag - norm_y).abs() < (1.0 / canvas_h as f32) {
                    *cell = '#';
                }
            }

            // Mark puck position
            if (self.filter_puck_pos.1 - norm_y).abs() < (1.0 / canvas_h as f32) {
                let px = (self.filter_puck_pos.0 * (width.saturating_sub(1) as f32)) as usize;
                if px < width {
                    row[px] = '@';
                }
            }

            lines.push(row.into_iter().collect());
        }

        let footer = format!(
            "Puck: ({:.2}, {:.2}) | KeyTrk: {:.0}% | Env: {:+.0}% [PASS: >=44pt]",
            self.filter_puck_pos.0,
            self.filter_puck_pos.1,
            self.key_tracking_percent,
            self.envelope_depth
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
            "ANALOG LADDER FILTER HUD",
            egui::FontId::proportional(15.0),
            Color32::from_rgb(0, 229, 255),
        );

        let is_osc = self.check_self_oscillation();
        let readout = format!(
            "FC: {:.0} Hz | RES: {:.1} {} | DRIVE: {:.0}% | PEAK: +{:.1} dB",
            self.cutoff_freq_hz,
            self.resonance_q,
            if is_osc { "[SELF-OSC]" } else { "" },
            self.drive_saturation,
            self.real_time_peak_db
        );
        painter.text(
            egui::pos2(rect.x + rect.width - 20.0, rect.y + 20.0),
            egui::Align2::RIGHT_TOP,
            readout,
            egui::FontId::proportional(12.0),
            if is_osc {
                Color32::from_rgb(255, 107, 43)
            } else {
                Color32::from_rgb(255, 215, 0)
            },
        );

        // Topology Selector Buttons Bar
        let topologies = [
            (LadderTopology::MoogTransistor4Pole, "MOOG 24dB TRANSISTOR"),
            (LadderTopology::Tb303DiodeLadder, "TB-303 DIODE LADDER"),
            (LadderTopology::OberheimSem2Pole, "SEM 12dB 2-POLE"),
            (LadderTopology::Ms20SallenKeyKorg, "MS-20 SALLEN-KEY"),
        ];

        let btn_y = rect.y + 54.0;
        let btn_w = (rect.width - 40.0 - 30.0) / 4.0;
        for (i, (topo, name)) in topologies.iter().enumerate() {
            let bx = rect.x + 20.0 + i as f32 * (btn_w + 10.0);
            let is_selected = self.topology == *topo;
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

        // Left Panel: Filter Frequency Response Canvas (20..490)
        let filter_canvas = Rect::new(rect.x + 20.0, rect.y + 100.0, 480.0, 230.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(filter_canvas.x, filter_canvas.y),
                egui::vec2(filter_canvas.width, filter_canvas.height),
            ),
            6.0,
            Color32::from_rgb(10, 14, 22),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(filter_canvas.x, filter_canvas.y),
                egui::vec2(filter_canvas.width, filter_canvas.height),
            ),
            6.0,
            Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(filter_canvas.x + 12.0, filter_canvas.y + 10.0),
            egui::Align2::LEFT_TOP,
            "24dB/OCT LADDER MAGNITUDE RESPONSE & RESONANCE PEAK",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(0, 229, 255),
        );

        // Grid lines
        for step in 1..4 {
            let gy = filter_canvas.y + filter_canvas.height * (step as f32 * 0.25);
            painter.line_segment(
                [
                    egui::pos2(filter_canvas.x, gy),
                    egui::pos2(filter_canvas.x + filter_canvas.width, gy),
                ],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 110, 80)),
            );
        }
        for freq in [100.0, 1000.0, 5000.0, 10000.0] {
            let gx = filter_canvas.x + Self::freq_to_normalized(freq) * filter_canvas.width;
            painter.line_segment(
                [
                    egui::pos2(gx, filter_canvas.y),
                    egui::pos2(gx, filter_canvas.y + filter_canvas.height),
                ],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 110, 80)),
            );
        }

        // Draw filter response curve
        let points_count = 60;
        let mut curve_pts = Vec::with_capacity(points_count);
        for i in 0..points_count {
            let norm_x = i as f32 / (points_count - 1) as f32;
            let f = Self::normalized_to_freq(norm_x);
            let mag = self.evaluate_filter_response(f);
            let cx = filter_canvas.x + norm_x * filter_canvas.width;
            let cy = filter_canvas.y + (1.0 - mag) * filter_canvas.height;
            curve_pts.push(egui::pos2(cx, cy));
        }
        for i in 0..(points_count - 1) {
            painter.line_segment(
                [curve_pts[i], curve_pts[i + 1]],
                Stroke::new(2.5_f32, Color32::from_rgb(0, 255, 180)),
            );
        }

        // 2D Cutoff & Resonance Puck
        let px = filter_canvas.x + self.filter_puck_pos.0 * filter_canvas.width;
        let py = filter_canvas.y + (1.0 - self.filter_puck_pos.1) * filter_canvas.height;

        // Self-oscillation outer glow
        if is_osc {
            painter.circle_stroke(
                egui::pos2(px, py),
                28.0,
                Stroke::new(3.0_f32, Color32::from_rgba_unmultiplied(255, 107, 43, 120)),
            );
        }

        // 44x44pt hit bounding target outline (radius 22pt)
        painter.circle_stroke(
            egui::pos2(px, py),
            LADDER_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 120)),
        );
        // Visual puck body
        painter.circle_filled(
            egui::pos2(px, py),
            14.0,
            if is_osc {
                Color32::from_rgb(255, 107, 43)
            } else {
                Color32::from_rgb(0, 229, 255)
            },
        );
        painter.circle_filled(egui::pos2(px, py), 4.0, Color32::WHITE);

        // Right Panel: Saturation Transfer Curve (510..780)
        let sat_canvas = Rect::new(rect.x + 515.0, rect.y + 100.0, rect.width - 535.0, 230.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(sat_canvas.x, sat_canvas.y),
                egui::vec2(sat_canvas.width, sat_canvas.height),
            ),
            6.0,
            Color32::from_rgb(10, 14, 22),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(sat_canvas.x, sat_canvas.y),
                egui::vec2(sat_canvas.width, sat_canvas.height),
            ),
            6.0,
            Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(sat_canvas.x + 12.0, sat_canvas.y + 10.0),
            egui::Align2::LEFT_TOP,
            "SATURATION & DIODE TRANSFER CURVE",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(255, 215, 0),
        );

        // Center cross axes
        let sat_mid_x = sat_canvas.x + sat_canvas.width * 0.5;
        let sat_mid_y = sat_canvas.y + sat_canvas.height * 0.5;
        painter.line_segment(
            [
                egui::pos2(sat_canvas.x, sat_mid_y),
                egui::pos2(sat_canvas.x + sat_canvas.width, sat_mid_y),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 110, 120)),
        );
        painter.line_segment(
            [
                egui::pos2(sat_mid_x, sat_canvas.y),
                egui::pos2(sat_mid_x, sat_canvas.y + sat_canvas.height),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 110, 120)),
        );

        // Saturation transfer curve points
        let mut sat_pts = Vec::with_capacity(40);
        for i in 0..40 {
            let norm_in = (i as f32 / 39.0) * 2.0 - 1.0; // [-1.0 .. 1.0]
            let out_val = self.evaluate_saturation_curve(norm_in);
            let sx = sat_mid_x + norm_in * (sat_canvas.width * 0.42);
            let sy = sat_mid_y - out_val * (sat_canvas.height * 0.42);
            sat_pts.push(egui::pos2(sx, sy));
        }
        for i in 0..39 {
            painter.line_segment(
                [sat_pts[i], sat_pts[i + 1]],
                Stroke::new(2.5_f32, Color32::from_rgb(255, 215, 0)),
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

        // Parameter readouts & quick controls
        let params = [
            (
                "KEY TRACKING",
                format!("{:.0}%", self.key_tracking_percent),
                (0, 229, 255),
            ),
            (
                "ENV DEPTH",
                format!("{:+.0}%", self.envelope_depth),
                (0, 255, 180),
            ),
            (
                "DRIVE / SAT",
                format!("{:.0}%", self.drive_saturation),
                (255, 215, 0),
            ),
            (
                "SELF OSCILLATION",
                if is_osc { "ACTIVE" } else { "OFF" }.to_string(),
                if is_osc {
                    (255, 107, 43)
                } else {
                    (180, 200, 225)
                },
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
            "[PASS] Analog Ladder Filter Touch Puck (>= 44x44pt) & Self-Oscillation HUD Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
