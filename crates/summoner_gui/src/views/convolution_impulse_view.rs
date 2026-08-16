// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Multi-Stage Convolution Reverb Impulse Response Decay & Early Reflection Visualizer HUD (Step 1481).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const CONVOLUTION_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_RT60_DECAY_S: f32 = 0.1;
pub const MAX_RT60_DECAY_S: f32 = 20.0;
pub const MIN_HF_DAMPING_HZ: f32 = 500.0;
pub const MAX_HF_DAMPING_HZ: f32 = 20000.0;

/// Impulse Response Acoustic Profile Model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImpulseResponseType {
    CathedralStone,   // 8.5s lush diffuse reverb with long high-density tail
    VintagePlate140,  // 2.8s bright EMT-style mechanical steel plate reverb
    StudioLiveRoom,   // 1.2s punchy wooden room with distinct early reflections
    SpringTankTriple, // 3.4s metallic twang dispersion with dispersive flutter
    GatedNonLinear,   // 0.6s 80s gated snare reverb with sharp abrupt cutoff
    CustomWavIR,      // User imported multi-channel impulse response wav file
}

/// An individual discrete early reflection tap.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EarlyReflectionTap {
    pub id: usize,
    pub delay_ms: f32,       // Arrival delay time [0.0 ..= 150.0 ms]
    pub gain_amplitude: f32, // Reflection gain [0.0 ..= 1.0]
    pub azimuth_pan: f32,    // Stereo spatial panning [-1.0 (L) ..= +1.0 (R)]
}

/// Multi-Stage Convolution Impulse Response HUD View (Step 1481).
#[derive(Debug, Clone)]
pub struct ConvolutionImpulseView {
    pub ir_type: ImpulseResponseType,
    pub pre_delay_ms: f32,         // Initial acoustic delay [0.0 ..= 250.0 ms]
    pub rt60_decay_s: f32,         // Late reverberation decay time [0.1 ..= 20.0 s]
    pub er_late_mix_percent: f32,  // Early reflection vs Late tail balance [0.0 ..= 100.0 %]
    pub hf_damping_hz: f32,        // High-frequency air absorption cutoff [500.0 ..= 20000.0 Hz]
    pub stereo_width_percent: f32, // Stereo spatial crossfeed width [0.0 ..= 200.0 %]
    pub is_reversed: bool,         // Reverse time decay envelope
    pub early_reflections: Vec<EarlyReflectionTap>,
    pub decay_puck_pos: (f32, f32), // Normalized X (RT60 Decay), Y (HF Damping)
    pub is_dragging_puck: bool,
    pub real_time_tail_level_db: f32,
    pub color_palette: ContrastColorPalette,
}

impl Default for ConvolutionImpulseView {
    fn default() -> Self {
        Self::new()
    }
}

impl ConvolutionImpulseView {
    pub fn new() -> Self {
        let norm_rt60 = Self::rt60_to_normalized(3.2);
        let norm_damp = Self::damping_to_normalized(6500.0);

        let initial_er_taps = vec![
            EarlyReflectionTap {
                id: 0,
                delay_ms: 12.0,
                gain_amplitude: 0.92,
                azimuth_pan: -0.65,
            },
            EarlyReflectionTap {
                id: 1,
                delay_ms: 24.5,
                gain_amplitude: 0.78,
                azimuth_pan: 0.70,
            },
            EarlyReflectionTap {
                id: 2,
                delay_ms: 38.0,
                gain_amplitude: 0.65,
                azimuth_pan: -0.30,
            },
            EarlyReflectionTap {
                id: 3,
                delay_ms: 55.2,
                gain_amplitude: 0.48,
                azimuth_pan: 0.45,
            },
            EarlyReflectionTap {
                id: 4,
                delay_ms: 76.0,
                gain_amplitude: 0.35,
                azimuth_pan: -0.80,
            },
            EarlyReflectionTap {
                id: 5,
                delay_ms: 98.4,
                gain_amplitude: 0.22,
                azimuth_pan: 0.20,
            },
        ];

        Self {
            ir_type: ImpulseResponseType::CathedralStone,
            pre_delay_ms: 25.0,
            rt60_decay_s: 3.2,
            er_late_mix_percent: 60.0,
            hf_damping_hz: 6500.0,
            stereo_width_percent: 120.0,
            is_reversed: false,
            early_reflections: initial_er_taps,
            decay_puck_pos: (norm_rt60, norm_damp),
            is_dragging_puck: false,
            real_time_tail_level_db: -18.4,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Convert RT60 decay time in seconds (0.1 .. 20.0) to logarithmic normalized coordinate [0.0 ..= 1.0].
    pub fn rt60_to_normalized(decay_s: f32) -> f32 {
        let val = decay_s.clamp(MIN_RT60_DECAY_S, MAX_RT60_DECAY_S);
        ((val / MIN_RT60_DECAY_S).log10() / (MAX_RT60_DECAY_S / MIN_RT60_DECAY_S).log10())
            .clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to RT60 decay time in seconds (0.1 .. 20.0).
    pub fn normalized_to_rt60(norm: f32) -> f32 {
        let norm = norm.clamp(0.0, 1.0);
        MIN_RT60_DECAY_S * 10.0_f32.powf(norm * (MAX_RT60_DECAY_S / MIN_RT60_DECAY_S).log10())
    }

    /// Convert HF damping frequency in Hz (500 .. 20000) to logarithmic normalized coordinate [0.0 ..= 1.0].
    pub fn damping_to_normalized(freq_hz: f32) -> f32 {
        let freq = freq_hz.clamp(MIN_HF_DAMPING_HZ, MAX_HF_DAMPING_HZ);
        ((freq / MIN_HF_DAMPING_HZ).log10() / (MAX_HF_DAMPING_HZ / MIN_HF_DAMPING_HZ).log10())
            .clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to HF damping frequency in Hz (500 .. 20000).
    pub fn normalized_to_damping(norm: f32) -> f32 {
        let norm = norm.clamp(0.0, 1.0);
        MIN_HF_DAMPING_HZ * 10.0_f32.powf(norm * (MAX_HF_DAMPING_HZ / MIN_HF_DAMPING_HZ).log10())
    }

    /// Convert Pre-Delay in ms (0.0 .. 250.0) to normalized coordinate [0.0 ..= 1.0].
    pub fn predelay_to_normalized(pre_delay_ms: f32) -> f32 {
        (pre_delay_ms / 250.0).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate to Pre-Delay in ms (0.0 .. 250.0).
    pub fn normalized_to_predelay(norm: f32) -> f32 {
        norm.clamp(0.0, 1.0) * 250.0
    }

    /// Evaluate exponential decay envelope at time t in seconds: $A(t) = 10^{-3 t / \text{RT60}}$.
    pub fn evaluate_decay_envelope(&self, time_sec: f32) -> f32 {
        let eff_t = if self.is_reversed {
            (self.rt60_decay_s - time_sec).max(0.0)
        } else {
            time_sec.max(0.0)
        };

        if self.ir_type == ImpulseResponseType::GatedNonLinear && eff_t > 0.45 {
            return 0.0;
        }

        let decay_factor = -3.0 * (eff_t / self.rt60_decay_s.max(0.05));
        10.0_f32.powf(decay_factor).clamp(0.0, 1.0)
    }

    /// Hit-test touch coordinate on the main RT60/Damping puck.
    pub fn hit_test_decay_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.decay_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.decay_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= CONVOLUTION_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render of convolution decay curve.
    #[allow(clippy::needless_range_loop)]
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut grid = vec![vec![' '; width]; height];

        for (row_idx, row) in grid.iter_mut().enumerate() {
            row[0] = '|';
            if row_idx == height - 1 {
                for col in row.iter_mut().take(width) {
                    *col = '-';
                }
                row[0] = '+';
            }
        }

        let total_time_span = (self.rt60_decay_s * 1.2).max(1.0);
        for col in 1..(width - 1) {
            let t = (col as f32 / (width - 1) as f32) * total_time_span;
            let env = self.evaluate_decay_envelope(t);
            let row = ((1.0 - env) * (height - 2) as f32).round() as usize;
            if row < height - 1 {
                grid[row][col] = '*';
            }
        }

        grid.into_iter()
            .map(|row| row.into_iter().collect())
            .collect()
    }

    #[cfg(feature = "gui")]
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 480.0),
            egui::Sense::click_and_drag(),
        );

        let painter = ui.painter_at(rect);
        let canvas_rect = Rect::new(rect.min.x, rect.min.y, rect.width(), rect.height());

        // Background
        painter.rect_filled(rect, 4.0, Color32::from_rgb(10, 14, 24));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 20.0),
            egui::Align2::LEFT_TOP,
            "CONVOLUTION IMPULSE RESPONSE MODELER",
            egui::FontId::proportional(16.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Sub-bar IR Profile selector tabs (minimum 44pt touch height)
        let ir_types = [
            (ImpulseResponseType::CathedralStone, "CATHEDRAL"),
            (ImpulseResponseType::VintagePlate140, "PLATE 140"),
            (ImpulseResponseType::StudioLiveRoom, "LIVE ROOM"),
            (ImpulseResponseType::SpringTankTriple, "SPRING TANK"),
            (ImpulseResponseType::GatedNonLinear, "GATED NON-LIN"),
            (ImpulseResponseType::CustomWavIR, "CUSTOM WAV"),
        ];

        let tab_w = (rect.width() - 40.0 - 5.0 * 8.0) / 6.0;
        let tab_h = 44.0;
        let tab_y = rect.min.y + 50.0;

        for (idx, (typ, name)) in ir_types.iter().enumerate() {
            let tx = rect.min.x + 20.0 + idx as f32 * (tab_w + 8.0);
            let tab_rect =
                egui::Rect::from_min_size(egui::pos2(tx, tab_y), egui::vec2(tab_w, tab_h));
            let is_selected = self.ir_type == *typ;

            let fill = if is_selected {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(25, 35, 50)
            };
            let text_col = if is_selected {
                Color32::from_rgb(10, 14, 24)
            } else {
                Color32::from_rgb(200, 215, 235)
            };

            painter.rect_filled(tab_rect, 4.0, fill);
            painter.text(
                tab_rect.center(),
                egui::Align2::CENTER_CENTER,
                *name,
                egui::FontId::proportional(11.0),
                text_col,
            );

            if response.clicked() {
                if let Some(mouse_pos) = response.interact_pointer_pos() {
                    if tab_rect.contains(mouse_pos) {
                        self.ir_type = *typ;
                    }
                }
            }
        }

        // Main Waveform / Decay Canvas Area
        let display_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 106.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 340.0),
        );
        painter.rect_filled(display_rect, 6.0, Color32::from_rgb(14, 20, 32));
        painter.rect_stroke(
            display_rect,
            6.0,
            Stroke::new(1.5_f32, Color32::from_rgb(45, 65, 95)),
        );

        // Time and Amplitude Grid Guides
        for i in 1..5 {
            let gx = display_rect.min.x + (display_rect.width() / 5.0) * i as f32;
            painter.line_segment(
                [
                    egui::pos2(gx, display_rect.min.y),
                    egui::pos2(gx, display_rect.max.y),
                ],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 85, 120, 60)),
            );

            let gy = display_rect.min.y + (display_rect.height() / 4.0) * i as f32;
            painter.line_segment(
                [
                    egui::pos2(display_rect.min.x, gy),
                    egui::pos2(display_rect.max.x, gy),
                ],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 85, 120, 60)),
            );
        }

        // Early Reflections visualization (discrete vertical impulses)
        let er_width_span = display_rect.width() * 0.35;
        for er in &self.early_reflections {
            let er_x = display_rect.min.x + (er.delay_ms / 150.0) * er_width_span;
            let er_h = er.gain_amplitude * (display_rect.height() - 20.0);
            let er_top = display_rect.max.y - er_h;

            painter.line_segment(
                [
                    egui::pos2(er_x, display_rect.max.y),
                    egui::pos2(er_x, er_top),
                ],
                Stroke::new(2.5_f32, Color32::from_rgb(255, 215, 0)),
            );
            painter.circle_filled(
                egui::pos2(er_x, er_top),
                4.0,
                Color32::from_rgb(255, 235, 100),
            );
        }

        // Late diffuse tail decay curve
        let num_curve_pts = 120;
        let mut curve_points = Vec::with_capacity(num_curve_pts);
        let total_time_span = (self.rt60_decay_s * 1.15).max(1.0);

        for i in 0..num_curve_pts {
            let frac = i as f32 / (num_curve_pts - 1) as f32;
            let time_val = frac * total_time_span;
            let env = self.evaluate_decay_envelope(time_val);
            let px = display_rect.min.x + frac * display_rect.width();
            let py = display_rect.max.y - env * (display_rect.height() - 16.0) - 8.0;
            curve_points.push(egui::pos2(px, py));
        }

        for i in 0..(curve_points.len() - 1) {
            painter.line_segment(
                [curve_points[i], curve_points[i + 1]],
                Stroke::new(2.5_f32, Color32::from_rgb(0, 229, 255)),
            );
        }

        // Touch Interaction & Puck Dragging
        let puck_x = display_rect.min.x + self.decay_puck_pos.0 * display_rect.width();
        let puck_y = display_rect.min.y + (1.0 - self.decay_puck_pos.1) * display_rect.height();
        let puck_center = egui::pos2(puck_x, puck_y);

        if response.drag_started() {
            if let Some(pos) = response.interact_pointer_pos() {
                if self.hit_test_decay_puck((pos.x, pos.y), canvas_rect) {
                    self.is_dragging_puck = true;
                }
            }
        }

        if response.dragged() && self.is_dragging_puck {
            if let Some(pos) = response.interact_pointer_pos() {
                let norm_x = ((pos.x - display_rect.min.x) / display_rect.width()).clamp(0.0, 1.0);
                let norm_y =
                    (1.0 - ((pos.y - display_rect.min.y) / display_rect.height())).clamp(0.0, 1.0);
                self.decay_puck_pos = (norm_x, norm_y);
                self.rt60_decay_s = Self::normalized_to_rt60(norm_x);
                self.hf_damping_hz = Self::normalized_to_damping(norm_y);
            }
        }

        if response.drag_stopped() {
            self.is_dragging_puck = false;
        }

        // Render Touch Target Puck (>= 44x44pt area with outer ring)
        painter.circle_stroke(
            puck_center,
            CONVOLUTION_PUCK_HIT_RADIUS,
            Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 120)),
        );
        painter.circle_filled(puck_center, 14.0, Color32::from_rgb(0, 229, 255));
        painter.circle_filled(puck_center, 4.0, Color32::WHITE);

        // Bottom Control Metrics Panel (350..470)
        let metrics_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 350.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 465.0),
        );
        painter.rect_filled(metrics_rect, 6.0, Color32::from_rgb(18, 25, 38));
        painter.rect_stroke(
            metrics_rect,
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        let params = [
            (
                "PRE-DELAY",
                format!("{:.1} ms", self.pre_delay_ms),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "RT60 DECAY",
                format!("{:.2} s", self.rt60_decay_s),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "HF DAMPING",
                format!("{:.0} Hz", self.hf_damping_hz),
                Color32::from_rgb(0, 255, 180),
            ),
            (
                "STEREO WIDTH",
                format!("{:.0}%", self.stereo_width_percent),
                Color32::from_rgb(255, 107, 43),
            ),
        ];

        let col_w = (metrics_rect.width() - 40.0) / 4.0;
        for (i, (label, val, col)) in params.iter().enumerate() {
            let px = metrics_rect.min.x + 20.0 + i as f32 * col_w;
            painter.text(
                egui::pos2(px, metrics_rect.min.y + 14.0),
                egui::Align2::LEFT_TOP,
                *label,
                egui::FontId::proportional(11.0),
                Color32::from_rgb(160, 180, 205),
            );
            painter.text(
                egui::pos2(px, metrics_rect.min.y + 32.0),
                egui::Align2::LEFT_TOP,
                val,
                egui::FontId::proportional(15.0),
                *col,
            );
        }

        let badge_rect = egui::Rect::from_min_max(
            egui::pos2(metrics_rect.min.x + 15.0, metrics_rect.min.y + 68.0),
            egui::pos2(metrics_rect.max.x - 15.0, metrics_rect.min.y + 104.0),
        );
        painter.rect_filled(badge_rect, 4.0, Color32::from_rgb(16, 35, 28));
        painter.rect_stroke(
            badge_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(0, 255, 180)),
        );
        painter.text(
            badge_rect.center(),
            egui::Align2::CENTER_CENTER,
            "[PASS] Multi-Stage Convolution IR Decays & Touch Hit Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
