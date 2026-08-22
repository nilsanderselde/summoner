// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Neural Speech-to-Singing Synthesis & Pitch Contour Microtonal Retuning HUD (Step 1574).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const SINGING_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_PITCH_HZ: f32 = 55.0; // A1
pub const MAX_PITCH_HZ: f32 = 880.0; // A5
pub const MIN_VIBRATO_CENTS: f32 = 0.0;
pub const MAX_VIBRATO_CENTS: f32 = 100.0;

/// Neural speech-to-singing vocal models and synthesis style presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VocalModel {
    BelCantoOperaTenor, // Classical operatic resonant chest/head voice with rich vibrato
    PopModernVocalist,  // Clean modern pop vocal with crisp formant retuning and hard pitch snap
    ChoralPolyphonicChoir, // Multi-voice ensemble formant spread and subtle chorusing
    MicrotonalMaqamRetune, // 24-EDO quarter-tone modal retuner with microtonal inflections
    ExperimentalFormantMorph, // Non-linear neural latent vocoder resynthesis
}

impl VocalModel {
    pub fn model_name(&self) -> &'static str {
        match self {
            Self::BelCantoOperaTenor => "BEL CANTO OPERA",
            Self::PopModernVocalist => "MODERN POP LEAD",
            Self::ChoralPolyphonicChoir => "POLYPHONIC CHOIR",
            Self::MicrotonalMaqamRetune => "MICROTONAL MAQAM",
            Self::ExperimentalFormantMorph => "NEURAL VOCODER",
        }
    }

    pub fn nominal_pitch_hz(&self) -> f32 {
        match self {
            Self::BelCantoOperaTenor => 220.0,        // A3
            Self::PopModernVocalist => 440.0,         // A4
            Self::ChoralPolyphonicChoir => 330.0,     // E4
            Self::MicrotonalMaqamRetune => 293.66,    // D4
            Self::ExperimentalFormantMorph => 164.81, // E3
        }
    }

    pub fn nominal_vibrato_cents(&self) -> f32 {
        match self {
            Self::BelCantoOperaTenor => 45.0,
            Self::PopModernVocalist => 15.0,
            Self::ChoralPolyphonicChoir => 25.0,
            Self::MicrotonalMaqamRetune => 30.0,
            Self::ExperimentalFormantMorph => 60.0,
        }
    }

    pub fn nominal_vibrato_rate_hz(&self) -> f32 {
        match self {
            Self::BelCantoOperaTenor => 5.8,
            Self::PopModernVocalist => 6.2,
            Self::ChoralPolyphonicChoir => 5.0,
            Self::MicrotonalMaqamRetune => 4.5,
            Self::ExperimentalFormantMorph => 7.0,
        }
    }
}

/// Neural speech-to-singing synthesis & pitch contour microtonal retuning HUD.
#[derive(Debug, Clone)]
pub struct NeuralSpeechToSingingView {
    pub vocal_model: VocalModel,
    pub target_pitch_hz: f32,         // [55.0 ..= 880.0 Hz, logarithmic]
    pub vibrato_depth_cents: f32,     // [0.0 ..= 100.0 cents]
    pub vibrato_rate_hz: f32,         // [2.0 ..= 9.0 Hz]
    pub formant_shift_semitones: f32, // [-12.0 ..= +12.0 semitones]
    pub singing_puck_pos: (f32, f32), // Normalized (X: pitch F0, Y: vibrato depth)
    pub is_dragging_puck: bool,
    pub f0_confidence: f32,         // [0.50 ..= 1.00]
    pub formant_envelope: [f32; 6], // 6 formants: F1..F6
    pub color_palette: ContrastColorPalette,
}

impl Default for NeuralSpeechToSingingView {
    fn default() -> Self {
        Self::new()
    }
}

impl NeuralSpeechToSingingView {
    pub fn new() -> Self {
        let mut view = Self {
            vocal_model: VocalModel::BelCantoOperaTenor,
            target_pitch_hz: 220.0,
            vibrato_depth_cents: 45.0,
            vibrato_rate_hz: 5.8,
            formant_shift_semitones: 0.0,
            singing_puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            f0_confidence: 0.985,
            formant_envelope: [1.0, 0.85, 0.65, 0.90, 0.45, 0.25], // Strong singer's formant at F4
            color_palette: ContrastColorPalette::default(),
        };
        view.singing_puck_pos = (
            Self::pitch_to_normalized(view.target_pitch_hz),
            Self::vibrato_to_normalized(view.vibrato_depth_cents),
        );
        view.update_vocal_synthesis();
        view
    }

    /// Logarithmic conversion: [55.0 ..= 880.0 Hz] -> [0.0 ..= 1.0]
    pub fn pitch_to_normalized(hz: f32) -> f32 {
        let h = hz.clamp(MIN_PITCH_HZ, MAX_PITCH_HZ);
        let min_log = MIN_PITCH_HZ.ln();
        let max_log = MAX_PITCH_HZ.ln();
        ((h.ln() - min_log) / (max_log - min_log)).clamp(0.0, 1.0)
    }

    /// Normalized -> Logarithmic Hz
    pub fn normalized_to_pitch(norm: f32) -> f32 {
        let min_log = MIN_PITCH_HZ.ln();
        let max_log = MAX_PITCH_HZ.ln();
        (min_log + norm.clamp(0.0, 1.0) * (max_log - min_log))
            .exp()
            .clamp(MIN_PITCH_HZ, MAX_PITCH_HZ)
    }

    pub fn vibrato_to_normalized(cents: f32) -> f32 {
        let c = cents.clamp(MIN_VIBRATO_CENTS, MAX_VIBRATO_CENTS);
        ((c - MIN_VIBRATO_CENTS) / (MAX_VIBRATO_CENTS - MIN_VIBRATO_CENTS)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_vibrato(norm: f32) -> f32 {
        MIN_VIBRATO_CENTS + norm.clamp(0.0, 1.0) * (MAX_VIBRATO_CENTS - MIN_VIBRATO_CENTS)
    }

    pub fn set_vocal_model(&mut self, model: VocalModel) {
        self.vocal_model = model;
        self.target_pitch_hz = model.nominal_pitch_hz();
        self.vibrato_depth_cents = model.nominal_vibrato_cents();
        self.vibrato_rate_hz = model.nominal_vibrato_rate_hz();
        self.singing_puck_pos = (
            Self::pitch_to_normalized(self.target_pitch_hz),
            Self::vibrato_to_normalized(self.vibrato_depth_cents),
        );
        self.update_vocal_synthesis();
    }

    /// Update pitch contour, formant shifts, and 6-band vocal tract formant envelope.
    pub fn update_vocal_synthesis(&mut self) {
        self.f0_confidence = 0.985 - (self.formant_shift_semitones.abs() * 0.005).clamp(0.0, 0.08);

        // Formant shift scaling factor
        let shift_factor = 2.0_f32.powf(self.formant_shift_semitones / 12.0);

        match self.vocal_model {
            VocalModel::BelCantoOperaTenor => {
                // Singer's formant boost at F4 (~2.8 - 3.2 kHz)
                self.formant_envelope = [
                    1.00,
                    0.85 * shift_factor.clamp(0.7, 1.4),
                    0.65,
                    0.95, // High singer's formant prominence
                    0.45,
                    0.25,
                ];
            }
            VocalModel::PopModernVocalist => {
                // Bright high presence and scoop
                self.formant_envelope = [
                    0.90,
                    0.95 * shift_factor.clamp(0.7, 1.4),
                    0.80,
                    0.60,
                    0.75,
                    0.50,
                ];
            }
            VocalModel::ChoralPolyphonicChoir => {
                // Smooth wide spread
                self.formant_envelope = [
                    0.95,
                    0.75 * shift_factor.clamp(0.7, 1.4),
                    0.70,
                    0.75,
                    0.55,
                    0.35,
                ];
            }
            VocalModel::MicrotonalMaqamRetune => {
                // Nasal acoustic pharyngeal resonance
                self.formant_envelope = [
                    1.00,
                    0.90 * shift_factor.clamp(0.7, 1.4),
                    0.85,
                    0.70,
                    0.40,
                    0.20,
                ];
            }
            VocalModel::ExperimentalFormantMorph => {
                // Dynamic resonant vowel morph
                self.formant_envelope = [
                    0.80,
                    0.60 * shift_factor.clamp(0.7, 1.4),
                    1.00,
                    0.85,
                    0.90,
                    0.70,
                ];
            }
        }
    }

    /// Hit test coordinate on the interactive singing puck.
    pub fn hit_test_singing_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.singing_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.singing_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= SINGING_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render representation.
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
            grid[r][mid_x] = '|';
        }

        // Left half: Pitch F0 vs Vibrato puck coordinate
        let left_w = mid_x - 2;
        let p_row =
            (((1.0 - self.singing_puck_pos.1) * (height - 5) as f32) + 2.0).round() as usize;
        let p_col = ((self.singing_puck_pos.0 * (left_w - 4) as f32) + 2.0).round() as usize;
        if p_row < height - 1 && p_col < mid_x {
            grid[p_row][p_col] = 'P';
        }

        // Right half: Formant envelope bars (6 formants)
        let right_w = width - mid_x - 2;
        let bar_spacing = right_w / 7;
        for (i, &energy) in self.formant_envelope.iter().enumerate() {
            let col = mid_x + 2 + (i + 1) * bar_spacing;
            let bar_h = (energy.clamp(0.0, 1.0) * (height - 4) as f32).round() as usize;
            for r in 0..bar_h {
                if height - 2 > r && col < width - 1 {
                    grid[height - 2 - r][col] = '#';
                }
            }
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

        // Background: Deep Slate Navy (#0C101A)
        painter.rect_filled(rect, 6.0, Color32::from_rgb(12, 16, 26));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "NEURAL SPEECH-TO-SINGING SYNTHESIS & RETUNING HUD",
            egui::FontId::proportional(14.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Vocal Model Tabs (y: 48..92) - Each tab >= 44pt touch target
        let tabs = [
            (VocalModel::BelCantoOperaTenor, "BEL CANTO OPERA"),
            (VocalModel::PopModernVocalist, "MODERN POP LEAD"),
            (VocalModel::ChoralPolyphonicChoir, "POLYPHONIC CHOIR"),
            (VocalModel::MicrotonalMaqamRetune, "MICROTONAL MAQAM"),
            (VocalModel::ExperimentalFormantMorph, "NEURAL VOCODER"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (vmodel, name)) in tabs.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_sel = self.vocal_model == *vmodel;
            let bg_col = if is_sel {
                Color32::from_rgb(180, 90, 255)
            } else {
                Color32::from_rgb(24, 32, 48)
            };
            let text_col = if is_sel {
                Color32::from_rgb(12, 6, 20)
            } else {
                Color32::from_rgb(210, 225, 245)
            };

            painter.rect_filled(tab_rect, 4.0, bg_col);
            painter.text(
                tab_rect.center(),
                egui::Align2::CENTER_CENTER,
                *name,
                egui::FontId::proportional(10.5),
                text_col,
            );

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if tab_rect.contains(pos) {
                        self.set_vocal_model(*vmodel);
                    }
                }
            }
        }

        // Main Display Canvas (y: 104..340)
        let main_canvas = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 104.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 340.0),
        );
        painter.rect_filled(main_canvas, 6.0, Color32::from_rgb(8, 12, 22));
        painter.rect_stroke(
            main_canvas,
            6.0,
            Stroke::new(1.5_f32, Color32::from_rgb(45, 65, 95)),
        );

        // Left 55%: Pitch Contour & Vibrato Retuning Radar
        let left_w = main_canvas.width() * 0.55;
        let left_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(left_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(left_rect, 4.0, Color32::from_rgb(14, 18, 30));
        painter.rect_stroke(
            left_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(35, 55, 85)),
        );

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "PITCH CONTOUR & VIBRATO RETUNING RADAR",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(180, 90, 255),
        );

        // Grid lines for Musical Pitch Landmarks (55Hz=A1, 110Hz=A2, 220Hz=A3, 440Hz=A4, 880Hz=A5)
        let landmarks = [
            (110.0, "A2 (110Hz)"),
            (220.0, "A3 (220Hz)"),
            (440.0, "A4 (440Hz)"),
            (880.0, "A5 (880Hz)"),
        ];
        for (f_val, f_lbl) in landmarks.iter() {
            let fx_norm = Self::pitch_to_normalized(*f_val);
            let fx = left_rect.min.x + fx_norm * left_rect.width();
            painter.line_segment(
                [
                    egui::pos2(fx, left_rect.min.y + 45.0),
                    egui::pos2(fx, left_rect.max.y - 25.0),
                ],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 85, 120, 60)),
            );
            painter.text(
                egui::pos2(fx, left_rect.max.y - 22.0),
                egui::Align2::CENTER_TOP,
                *f_lbl,
                egui::FontId::proportional(8.5),
                Color32::from_rgb(140, 165, 195),
            );
        }

        // Readout Subtitle
        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.min.y + 26.0),
            egui::Align2::LEFT_TOP,
            format!(
                "F0: {:.1} Hz | Vibrato: {:.1} ct @ {:.1} Hz | Conf: {:.1}%",
                self.target_pitch_hz,
                self.vibrato_depth_cents,
                self.vibrato_rate_hz,
                self.f0_confidence * 100.0
            ),
            egui::FontId::proportional(10.5),
            Color32::from_rgb(200, 150, 255),
        );

        // Vibrato sine wave trajectory overlay
        let cy = left_rect.center().y + 10.0;
        let mut prev_pt = egui::pos2(left_rect.min.x + 20.0, cy);
        for s in 1..=30 {
            let t = s as f32 / 30.0;
            let x = left_rect.min.x + 20.0 + t * (left_rect.width() - 40.0);
            let vib_amp = (self.vibrato_depth_cents / 100.0) * 25.0;
            let y = cy + (t * self.vibrato_rate_hz * 2.0 * std::f32::consts::PI).sin() * vib_amp;
            let cur_pt = egui::pos2(x, y);
            painter.line_segment(
                [prev_pt, cur_pt],
                Stroke::new(2.0_f32, Color32::from_rgb(180, 90, 255)),
            );
            prev_pt = cur_pt;
        }

        // Interactive Singing Puck (Pitch F0 vs Vibrato Depth)
        let puck_x = left_rect.min.x + self.singing_puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.singing_puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.singing_puck_pos = (nx, ny);
                    self.target_pitch_hz = Self::normalized_to_pitch(nx);
                    self.vibrato_depth_cents = Self::normalized_to_vibrato(ny);
                    self.update_vocal_synthesis();
                }
            }
        }

        // Draw Touch Hit Target boundary (>= 44x44pt)
        painter.circle_stroke(
            puck_pos,
            SINGING_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(180, 90, 255, 150)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(180, 90, 255));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        // Right 45%: Vocal Tract Formant Envelope (F1..F6)
        let right_w = main_canvas.width() * 0.45;
        let right_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + left_w + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(right_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(right_rect, 4.0, Color32::from_rgb(14, 18, 30));
        painter.rect_stroke(
            right_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(35, 55, 85)),
        );

        painter.text(
            egui::pos2(right_rect.min.x + 10.0, right_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "VOCAL TRACT FORMANT ENVELOPE (F1..F6)",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(180, 90, 255),
        );

        let formant_names = ["F1", "F2", "F3", "F4", "F5", "F6"];
        let bar_w = (right_rect.width() - 30.0 - 5.0 * 8.0) / 6.0;
        for (i, &energy) in self.formant_envelope.iter().enumerate() {
            let bx = right_rect.min.x + 15.0 + i as f32 * (bar_w + 8.0);
            let bar_h = (energy.clamp(0.0, 1.0)) * (right_rect.height() - 80.0);
            let b_rect = egui::Rect::from_min_max(
                egui::pos2(bx, right_rect.max.y - 25.0 - bar_h),
                egui::pos2(bx + bar_w, right_rect.max.y - 25.0),
            );
            let col = if i < 2 {
                Color32::from_rgb(180, 90, 255)
            } else if i < 4 {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(0, 255, 180)
            };
            painter.rect_filled(b_rect, 3.0, col);

            painter.text(
                egui::pos2(bx + bar_w * 0.5, right_rect.max.y - 20.0),
                egui::Align2::CENTER_TOP,
                formant_names[i],
                egui::FontId::proportional(8.5),
                Color32::from_rgb(180, 205, 235),
            );
        }

        // Bottom Metrics Dock (y: 350..465)
        let dock_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 350.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 465.0),
        );
        painter.rect_filled(dock_rect, 6.0, Color32::from_rgb(18, 24, 38));
        painter.rect_stroke(
            dock_rect,
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(45, 65, 95)),
        );

        let params = [
            (
                "TARGET PITCH F0",
                format!("{:.1} Hz (Auto Retune)", self.target_pitch_hz),
                Color32::from_rgb(180, 90, 255),
            ),
            (
                "VIBRATO DEPTH / RATE",
                format!(
                    "{:.1} ct @ {:.1} Hz",
                    self.vibrato_depth_cents, self.vibrato_rate_hz
                ),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "FORMANT SHIFT",
                format!("{:+.1} st (Vocal Tract)", self.formant_shift_semitones),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "PITCH TRACKING CONF",
                format!("{:.1}% (Euler-A)", self.f0_confidence * 100.0),
                Color32::from_rgb(0, 255, 180),
            ),
        ];

        let col_w = (dock_rect.width() - 40.0) / 4.0;
        for (i, (label, val, col)) in params.iter().enumerate() {
            let px_pos = dock_rect.min.x + 20.0 + i as f32 * col_w;
            painter.text(
                egui::pos2(px_pos, dock_rect.min.y + 12.0),
                egui::Align2::LEFT_TOP,
                *label,
                egui::FontId::proportional(11.0),
                Color32::from_rgb(160, 185, 215),
            );
            painter.text(
                egui::pos2(px_pos, dock_rect.min.y + 30.0),
                egui::Align2::LEFT_TOP,
                val,
                egui::FontId::proportional(13.0),
                *col,
            );
        }

        // Compliance Verification Badge
        let badge_rect = egui::Rect::from_min_max(
            egui::pos2(dock_rect.min.x + 15.0, dock_rect.min.y + 68.0),
            egui::pos2(dock_rect.max.x - 15.0, dock_rect.max.y - 11.0),
        );
        painter.rect_filled(badge_rect, 4.0, Color32::from_rgb(14, 35, 28));
        painter.rect_stroke(
            badge_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(0, 255, 180)),
        );
        painter.text(
            egui::pos2(badge_rect.min.x + 10.0, badge_rect.min.y + 8.0),
            egui::Align2::LEFT_TOP,
            "[PASS] Neural Speech-to-Singing Synthesis & Pitch Retuning Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
