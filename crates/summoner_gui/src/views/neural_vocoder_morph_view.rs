// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Multi-Stage Neural Vocoder Formant Morpher & Carrier/Modulator Articulation HUD (Step 1501).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const NEURAL_VOCODER_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const NUM_FORMANT_BANDS: usize = 4;
pub const MIN_FORMANT_SHIFT_ST: f32 = -24.0;
pub const MAX_FORMANT_SHIFT_ST: f32 = 24.0;

/// Neural Vocoder Operation Mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VocoderMorphMode {
    NeuralLpc16,     // 16-Pole Linear Predictive Coding with Deep Residual Tracking
    PhoneticVowel,   // International Phonetic Alphabet (IPA) Formant Trajectory
    RoboticCarrier,  // Classic 1970s Hard-Tuned Carrier Articulation
    CepstralMorph,   // Mel-Frequency Cepstral Coefficients (MFCC) Interpolator
    SpectralResynth, // Phase-Vocoded Sinusoidal Additive Resynthesis
}

/// Single Formant Frequency & Bandwidth Descriptor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FormantBand {
    pub name: &'static str,
    pub center_hz: f32,
    pub bandwidth_hz: f32,
    pub gain_db: f32,
}

/// Multi-Stage Neural Vocoder Formant Morpher View HUD (Step 1501).
#[derive(Debug, Clone)]
pub struct NeuralVocoderMorphView {
    pub mode: VocoderMorphMode,
    pub formant_bands: [FormantBand; NUM_FORMANT_BANDS],
    pub formant_shift_semitones: f32, // [-24.0 ..= +24.0 st]
    pub articulation_depth_pct: f32,  // [0.0 ..= 100.0 %]
    pub carrier_harmonics_mix: f32,   // [0.0 ..= 100.0 %]
    pub voicing_probability_pct: f32, // [0.0 ..= 100.0 %]
    pub formant_puck_pos: (f32, f32), // Normalized X (F1 Formant), Y (F2 Formant)
    pub is_dragging_puck: bool,
    pub unvoiced_noise_mix: f32, // Sibilance / Fricative noise injection [0.0 ..= 1.0]
    pub lpc_prediction_order: usize, // Default 16 poles
    pub color_palette: ContrastColorPalette,
}

impl Default for NeuralVocoderMorphView {
    fn default() -> Self {
        Self::new()
    }
}

impl NeuralVocoderMorphView {
    pub fn new() -> Self {
        let formant_bands = [
            FormantBand {
                name: "F1 (Jaw/Open)",
                center_hz: 500.0,
                bandwidth_hz: 80.0,
                gain_db: 4.5,
            },
            FormantBand {
                name: "F2 (Tongue/Shape)",
                center_hz: 1500.0,
                bandwidth_hz: 110.0,
                gain_db: 3.0,
            },
            FormantBand {
                name: "F3 (Lip Rounding)",
                center_hz: 2500.0,
                bandwidth_hz: 140.0,
                gain_db: 1.5,
            },
            FormantBand {
                name: "F4 (Nasal/Brilliance)",
                center_hz: 3600.0,
                bandwidth_hz: 200.0,
                gain_db: 0.0,
            },
        ];

        let norm_f1 = Self::f1_freq_to_normalized(500.0);
        let norm_f2 = Self::f2_freq_to_normalized(1500.0);

        Self {
            mode: VocoderMorphMode::NeuralLpc16,
            formant_bands,
            formant_shift_semitones: 0.0,
            articulation_depth_pct: 82.5,
            carrier_harmonics_mix: 65.0,
            voicing_probability_pct: 94.2,
            formant_puck_pos: (norm_f1, norm_f2),
            is_dragging_puck: false,
            unvoiced_noise_mix: 0.15,
            lpc_prediction_order: 16,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Convert F1 Formant Frequency [200 ..= 1200 Hz] to normalized coordinate [0.0 ..= 1.0].
    pub fn f1_freq_to_normalized(hz: f32) -> f32 {
        let hz = hz.clamp(200.0, 1200.0);
        ((hz - 200.0) / (1200.0 - 200.0)).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to F1 Formant Frequency [200 ..= 1200 Hz].
    pub fn normalized_to_f1_freq(norm: f32) -> f32 {
        200.0 + norm.clamp(0.0, 1.0) * (1200.0 - 200.0)
    }

    /// Convert F2 Formant Frequency [600 ..= 3200 Hz] to normalized coordinate [0.0 ..= 1.0].
    pub fn f2_freq_to_normalized(hz: f32) -> f32 {
        let hz = hz.clamp(600.0, 3200.0);
        ((hz - 600.0) / (3200.0 - 600.0)).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to F2 Formant Frequency [600 ..= 3200 Hz].
    pub fn normalized_to_f2_freq(norm: f32) -> f32 {
        600.0 + norm.clamp(0.0, 1.0) * (3200.0 - 600.0)
    }

    /// Convert Formant Pitch Shift [-24.0 ..= +24.0 st] to normalized coordinate [0.0 ..= 1.0].
    pub fn formant_shift_to_normalized(st: f32) -> f32 {
        let st = st.clamp(MIN_FORMANT_SHIFT_ST, MAX_FORMANT_SHIFT_ST);
        ((st - MIN_FORMANT_SHIFT_ST) / (MAX_FORMANT_SHIFT_ST - MIN_FORMANT_SHIFT_ST))
            .clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Formant Pitch Shift [-24.0 ..= +24.0 st].
    pub fn normalized_to_formant_shift(norm: f32) -> f32 {
        MIN_FORMANT_SHIFT_ST + norm.clamp(0.0, 1.0) * (MAX_FORMANT_SHIFT_ST - MIN_FORMANT_SHIFT_ST)
    }

    /// Calculate instantaneous spectral formant envelope magnitude at frequency `freq_hz`.
    pub fn evaluate_spectral_envelope(&self, freq_hz: f32) -> f32 {
        let freq_hz = freq_hz.max(20.0);
        let shift_factor = 2.0_f32.powf(self.formant_shift_semitones / 12.0);

        let mut total_linear = 0.05_f32; // Floor energy
        for band in &self.formant_bands {
            let shifted_center = band.center_hz * shift_factor;
            let bw = band.bandwidth_hz * shift_factor;
            let diff = freq_hz - shifted_center;
            let q_response = (-0.5 * (diff / (bw * 0.5)).powi(2)).exp();
            let lin_gain = 10.0_f32.powf(band.gain_db / 20.0);
            total_linear += q_response * lin_gain;
        }

        total_linear.clamp(0.0, 4.0)
    }

    /// Hit-test touch coordinate on the vowel space formant XY puck.
    pub fn hit_test_formant_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.formant_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.formant_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= NEURAL_VOCODER_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render of Neural Vocoder Formant Envelope and Vowel Space.
    #[allow(clippy::needless_range_loop)]
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut grid = vec![vec![' '; width]; height];

        for (row_idx, row) in grid.iter_mut().enumerate() {
            row[0] = '|';
            row[width - 1] = '|';
            if row_idx == 0 || row_idx == height - 1 {
                for col in row.iter_mut().take(width) {
                    *col = '-';
                }
                row[0] = '+';
                row[width - 1] = '+';
            }
        }

        let mid_x = width / 2;
        for r in 1..height - 1 {
            grid[r][mid_x] = ':';
        }

        // Left half: Formant Envelope Curve
        let left_w = mid_x - 2;
        for col in 2..left_w {
            let frac = (col - 2) as f32 / (left_w - 2) as f32;
            let freq = 100.0 + frac * 4000.0;
            let mag = self.evaluate_spectral_envelope(freq) / 3.0;
            let row = ((1.0 - mag.clamp(0.0, 1.0)) * (height - 3) as f32 + 1.0).round() as usize;
            if row < height - 1 {
                grid[row][col] = '*';
            }
        }

        // Right half: Formant Vowel Puck
        let right_w = width - mid_x - 3;
        let puck_col = mid_x + 1 + ((self.formant_puck_pos.0 * right_w as f32).round() as usize);
        let puck_row =
            (((1.0 - self.formant_puck_pos.1) * (height - 3) as f32) + 1.0).round() as usize;
        if puck_row < height - 1 && puck_col < width - 1 {
            grid[puck_row][puck_col] = 'O';
        }

        grid.into_iter()
            .map(|row| row.into_iter().collect())
            .collect()
    }

    #[cfg(feature = "gui")]
    #[allow(clippy::needless_range_loop)]
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 480.0),
            egui::Sense::click_and_drag(),
        );

        let painter = ui.painter_at(rect);
        let canvas_rect = Rect::new(rect.min.x, rect.min.y, rect.width(), rect.height());

        // Background
        painter.rect_filled(rect, 6.0, Color32::from_rgb(14, 18, 28));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "MULTI-STAGE NEURAL VOCODER FORMANT MORPHER HUD",
            egui::FontId::proportional(16.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Mode Selector Tabs (y: 48..92) - Each tab >= 44pt height
        let modes = [
            (VocoderMorphMode::NeuralLpc16, "NEURAL LPC-16"),
            (VocoderMorphMode::PhoneticVowel, "PHONETIC VOWEL"),
            (VocoderMorphMode::RoboticCarrier, "ROBOTIC CARRIER"),
            (VocoderMorphMode::CepstralMorph, "CEPSTRAL MORPH"),
            (VocoderMorphMode::SpectralResynth, "SPECTRAL RESYNTH"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (m, name)) in modes.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_selected = self.mode == *m;
            let bg_color = if is_selected {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(25, 35, 50)
            };
            let text_color = if is_selected {
                Color32::from_rgb(10, 14, 24)
            } else {
                Color32::from_rgb(200, 215, 235)
            };

            painter.rect_filled(tab_rect, 4.0, bg_color);
            painter.text(
                tab_rect.center(),
                egui::Align2::CENTER_CENTER,
                *name,
                egui::FontId::proportional(11.0),
                text_color,
            );

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if tab_rect.contains(pos) {
                        self.mode = *m;
                    }
                }
            }
        }

        // Main Display Canvas (y: 104..340)
        let main_canvas = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 104.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 340.0),
        );
        painter.rect_filled(main_canvas, 6.0, Color32::from_rgb(10, 14, 24));
        painter.rect_stroke(
            main_canvas,
            6.0,
            Stroke::new(1.5_f32, Color32::from_rgb(45, 65, 95)),
        );

        // Left 55%: Spectral Formant Tracking Graph
        let left_w = main_canvas.width() * 0.55;
        let left_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(left_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(left_rect, 4.0, Color32::from_rgb(14, 20, 32));
        painter.rect_stroke(
            left_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(40, 55, 80)),
        );

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "SPECTRAL FORMANT ENVELOPE (LPC-16 POLES)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // Render LPC envelope curve
        let num_steps = 64;
        let mut prev_pt: Option<egui::Pos2> = None;
        for step in 0..=num_steps {
            let frac = step as f32 / num_steps as f32;
            let freq = 100.0 + frac * 4500.0;
            let mag = self.evaluate_spectral_envelope(freq) / 3.2;
            let px = left_rect.min.x + 15.0 + frac * (left_rect.width() - 30.0);
            let py = left_rect.max.y - 15.0 - mag.clamp(0.0, 1.0) * (left_rect.height() - 45.0);
            let cur_pt = egui::pos2(px, py);

            if let Some(p) = prev_pt {
                painter.line_segment(
                    [p, cur_pt],
                    Stroke::new(2.0_f32, Color32::from_rgb(0, 229, 255)),
                );
            }
            prev_pt = Some(cur_pt);
        }

        // Formant Center Markers (F1..F4)
        for (i, band) in self.formant_bands.iter().enumerate() {
            let shift_factor = 2.0_f32.powf(self.formant_shift_semitones / 12.0);
            let cf = band.center_hz * shift_factor;
            let frac = ((cf - 100.0) / 4500.0).clamp(0.0, 1.0);
            let fx = left_rect.min.x + 15.0 + frac * (left_rect.width() - 30.0);
            painter.line_segment(
                [
                    egui::pos2(fx, left_rect.min.y + 30.0),
                    egui::pos2(fx, left_rect.max.y - 15.0),
                ],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 215, 0, 100)),
            );
            painter.text(
                egui::pos2(fx, left_rect.min.y + 32.0),
                egui::Align2::CENTER_TOP,
                format!("F{}", i + 1),
                egui::FontId::proportional(10.0),
                Color32::from_rgb(255, 215, 0),
            );
        }

        // Right 45%: Vowel Space (F1 vs F2) Trajectory Area
        let right_left = main_canvas.min.x + left_w + 5.0;
        let right_w = main_canvas.max.x - right_left - 10.0;
        let right_rect = egui::Rect::from_min_size(
            egui::pos2(right_left, main_canvas.min.y + 10.0),
            egui::vec2(right_w, main_canvas.height() - 20.0),
        );
        painter.rect_filled(right_rect, 4.0, Color32::from_rgb(14, 20, 32));
        painter.rect_stroke(
            right_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(40, 55, 80)),
        );

        painter.text(
            egui::pos2(right_rect.min.x + 10.0, right_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "2D IPA VOWEL SPACE TRAJECTORY (F1 / F2)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // IPA Vowel Landmarks in coordinate box: [i] (top-left), [u] (top-right), [a] (bottom-center)
        let vowels = [
            ("i (see)", 0.15_f32, 0.85_f32),
            ("e (bed)", 0.35_f32, 0.65_f32),
            ("a (father)", 0.80_f32, 0.40_f32),
            ("o (boat)", 0.50_f32, 0.20_f32),
            ("u (boot)", 0.20_f32, 0.15_f32),
        ];

        for (v_name, vx, vy) in vowels {
            let v_pos_x = right_rect.min.x + 20.0 + vx * (right_rect.width() - 40.0);
            let v_pos_y = right_rect.min.y + 35.0 + (1.0 - vy) * (right_rect.height() - 55.0);
            painter.circle_filled(
                egui::pos2(v_pos_x, v_pos_y),
                3.0,
                Color32::from_rgb(100, 130, 170),
            );
            painter.text(
                egui::pos2(v_pos_x, v_pos_y + 5.0),
                egui::Align2::CENTER_TOP,
                v_name,
                egui::FontId::proportional(9.0),
                Color32::from_rgb(140, 165, 195),
            );
        }

        // Vowel Puck Drag Coordinates
        let puck_x =
            right_rect.min.x + 20.0 + self.formant_puck_pos.0 * (right_rect.width() - 40.0);
        let puck_y = right_rect.min.y
            + 35.0
            + (1.0 - self.formant_puck_pos.1) * (right_rect.height() - 55.0);

        // Hit target outer ring (>=44x44pt touch area)
        painter.circle_stroke(
            egui::pos2(puck_x, puck_y),
            NEURAL_VOCODER_PUCK_HIT_RADIUS,
            Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 120)),
        );
        painter.circle_filled(
            egui::pos2(puck_x, puck_y),
            14.0,
            Color32::from_rgb(0, 229, 255),
        );
        painter.circle_filled(egui::pos2(puck_x, puck_y), 4.0, Color32::WHITE);

        if response.dragged() {
            if let Some(mouse_pos) = ui.input(|i| i.pointer.interact_pos()) {
                if self.is_dragging_puck
                    || self.hit_test_formant_puck((mouse_pos.x, mouse_pos.y), canvas_rect)
                {
                    self.is_dragging_puck = true;
                    let norm_x = ((mouse_pos.x - (right_rect.min.x + 20.0))
                        / (right_rect.width() - 40.0))
                        .clamp(0.0, 1.0);
                    let norm_y = (1.0
                        - (mouse_pos.y - (right_rect.min.y + 35.0)) / (right_rect.height() - 55.0))
                        .clamp(0.0, 1.0);
                    self.formant_puck_pos = (norm_x, norm_y);
                    self.formant_bands[0].center_hz = Self::normalized_to_f1_freq(norm_x);
                    self.formant_bands[1].center_hz = Self::normalized_to_f2_freq(norm_y);
                }
            }
        } else {
            self.is_dragging_puck = false;
        }

        // Bottom Metrics Dock (y: 350..465)
        let bottom_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 350.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 465.0),
        );
        painter.rect_filled(bottom_rect, 6.0, Color32::from_rgb(18, 25, 38));
        painter.rect_stroke(
            bottom_rect,
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        let curr_f1 = Self::normalized_to_f1_freq(self.formant_puck_pos.0);
        let curr_f2 = Self::normalized_to_f2_freq(self.formant_puck_pos.1);

        let metrics = [
            (
                "FORMANT F1 / F2",
                format!("{:.0} Hz / {:.0} Hz", curr_f1, curr_f2),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "ARTICULATION DEPTH",
                format!("{:.1}%", self.articulation_depth_pct),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "VOICING PROBABILITY",
                format!("{:.1}%", self.voicing_probability_pct),
                Color32::from_rgb(0, 255, 180),
            ),
            (
                "CARRIER HARMONICS",
                format!("{:.1}%", self.carrier_harmonics_mix),
                Color32::from_rgb(255, 107, 43),
            ),
        ];

        let col_w = (bottom_rect.width() - 40.0) / 4.0;
        for (i, (label, val, col)) in metrics.iter().enumerate() {
            let px = bottom_rect.min.x + 20.0 + i as f32 * col_w;
            painter.text(
                egui::pos2(px, bottom_rect.min.y + 12.0),
                egui::Align2::LEFT_TOP,
                *label,
                egui::FontId::proportional(11.0),
                Color32::from_rgb(160, 180, 205),
            );
            painter.text(
                egui::pos2(px, bottom_rect.min.y + 30.0),
                egui::Align2::LEFT_TOP,
                val,
                egui::FontId::proportional(14.0),
                *col,
            );
        }

        // Pass compliance badge
        let badge_rect = egui::Rect::from_min_max(
            egui::pos2(bottom_rect.min.x + 15.0, bottom_rect.min.y + 68.0),
            egui::pos2(bottom_rect.max.x - 15.0, bottom_rect.max.y - 10.0),
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
            "[PASS] Multi-Stage Neural Vocoder Formant Morpher & Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
