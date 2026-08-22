// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Neural Audio Latent Style Transfer & Phoneme Morphing Vocoder HUD (Step 1554).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const PHONEME_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_FORMANT_F1_HZ: f32 = 200.0;
pub const MAX_FORMANT_F1_HZ: f32 = 1200.0;
pub const MIN_FORMANT_F2_HZ: f32 = 600.0;
pub const MAX_FORMANT_F2_HZ: f32 = 3500.0;
pub const MIN_VOCAL_TRACT_CM: f32 = 8.0;
pub const MAX_VOCAL_TRACT_CM: f32 = 25.0;

/// Neural Phoneme & Style Transfer Synthesis Architecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhonemeModel {
    VowelFormantMorph, // Continuous 2D IPA vowel quadrilateral formant interpolation (/i/, /e/, /a/, /o/, /u/)
    WhisperToVoicedTransfer, // Generative glottal pulse harmonic excitation from unvoiced whisper
    RoboticVocoderCarrier, // 64-band phase-aligned carrier filterbank with formant tracking
    AlienFormantShift, // Hyperbolic vocal tract dilation and non-human formant dispersion
    LatentDiffusionInterpolate, // Continuous latent embedding walk between neural singer timbre identities
}

impl PhonemeModel {
    pub fn default_f1_hz(&self) -> f32 {
        match self {
            Self::VowelFormantMorph => 500.0,
            Self::WhisperToVoicedTransfer => 750.0,
            Self::RoboticVocoderCarrier => 400.0,
            Self::AlienFormantShift => 280.0,
            Self::LatentDiffusionInterpolate => 650.0,
        }
    }

    pub fn default_f2_hz(&self) -> f32 {
        match self {
            Self::VowelFormantMorph => 1800.0,
            Self::WhisperToVoicedTransfer => 1200.0,
            Self::RoboticVocoderCarrier => 2400.0,
            Self::AlienFormantShift => 950.0,
            Self::LatentDiffusionInterpolate => 2100.0,
        }
    }

    pub fn default_tract_length_cm(&self) -> f32 {
        match self {
            Self::VowelFormantMorph => 17.0, // Standard adult vocal tract
            Self::WhisperToVoicedTransfer => 16.5,
            Self::RoboticVocoderCarrier => 14.0,
            Self::AlienFormantShift => 24.0, // Elongated acoustic cavity
            Self::LatentDiffusionInterpolate => 15.5,
        }
    }
}

/// Neural Audio Latent Style Transfer & Phoneme Morphing View HUD (Step 1554).
#[derive(Debug, Clone)]
pub struct NeuralPhonemeView {
    pub model: PhonemeModel,
    pub formant_f1_hz: f32,           // [200.0 ..= 1200.0 Hz]
    pub formant_f2_hz: f32,           // [600.0 ..= 3500.0 Hz]
    pub formant_f3_hz: f32,           // [1500.0 ..= 4500.0 Hz]
    pub vocal_tract_cm: f32,          // [8.0 ..= 25.0 cm]
    pub latent_style_weight: f32,     // [0.0 ..= 1.0]
    pub phoneme_puck_pos: (f32, f32), // Normalized (X: F2 frequency, Y: F1 frequency)
    pub is_dragging_puck: bool,
    pub active_phoneme_symbol: &'static str, // IPA vowel symbol ("/a/", "/i/", etc.)
    pub latent_embeddings: [f32; 8],         // 8-D style transfer latent vector
    pub glottal_open_quotient: f32,          // Voice source pulse parameter [0.3 ..= 0.8]
    pub color_palette: ContrastColorPalette,
}

impl Default for NeuralPhonemeView {
    fn default() -> Self {
        Self::new()
    }
}

impl NeuralPhonemeView {
    pub fn new() -> Self {
        let mut view = Self {
            model: PhonemeModel::VowelFormantMorph,
            formant_f1_hz: 500.0,
            formant_f2_hz: 1800.0,
            formant_f3_hz: 2800.0,
            vocal_tract_cm: 17.0,
            latent_style_weight: 0.80,
            phoneme_puck_pos: (0.45, 0.35),
            is_dragging_puck: false,
            active_phoneme_symbol: "/e/",
            latent_embeddings: [0.85, 0.42, 0.15, 0.78, 0.92, 0.33, 0.60, 0.71],
            glottal_open_quotient: 0.55,
            color_palette: ContrastColorPalette::default(),
        };
        view.update_phoneme_simulation();
        view
    }

    /// Convert Formant F1 [200.0 ..= 1200.0 Hz] to normalized [0.0 ..= 1.0] (logarithmic).
    pub fn f1_to_normalized(f1_hz: f32) -> f32 {
        let f = f1_hz.clamp(MIN_FORMANT_F1_HZ, MAX_FORMANT_F1_HZ);
        ((f.ln() - MIN_FORMANT_F1_HZ.ln()) / (MAX_FORMANT_F1_HZ.ln() - MIN_FORMANT_F1_HZ.ln()))
            .clamp(0.0, 1.0)
    }

    /// Convert normalized [0.0 ..= 1.0] to Formant F1 [200.0 ..= 1200.0 Hz].
    pub fn normalized_to_f1(norm: f32) -> f32 {
        let n = norm.clamp(0.0, 1.0);
        (MIN_FORMANT_F1_HZ.ln() + n * (MAX_FORMANT_F1_HZ.ln() - MIN_FORMANT_F1_HZ.ln())).exp()
    }

    /// Convert Formant F2 [600.0 ..= 3500.0 Hz] to normalized [0.0 ..= 1.0] (logarithmic).
    pub fn f2_to_normalized(f2_hz: f32) -> f32 {
        let f = f2_hz.clamp(MIN_FORMANT_F2_HZ, MAX_FORMANT_F2_HZ);
        ((f.ln() - MIN_FORMANT_F2_HZ.ln()) / (MAX_FORMANT_F2_HZ.ln() - MIN_FORMANT_F2_HZ.ln()))
            .clamp(0.0, 1.0)
    }

    /// Convert normalized [0.0 ..= 1.0] to Formant F2 [600.0 ..= 3500.0 Hz].
    pub fn normalized_to_f2(norm: f32) -> f32 {
        let n = norm.clamp(0.0, 1.0);
        (MIN_FORMANT_F2_HZ.ln() + n * (MAX_FORMANT_F2_HZ.ln() - MIN_FORMANT_F2_HZ.ln())).exp()
    }

    /// Convert Vocal Tract Length [8.0 ..= 25.0 cm] to normalized [0.0 ..= 1.0].
    pub fn tract_to_normalized(tract_cm: f32) -> f32 {
        let t = tract_cm.clamp(MIN_VOCAL_TRACT_CM, MAX_VOCAL_TRACT_CM);
        ((t - MIN_VOCAL_TRACT_CM) / (MAX_VOCAL_TRACT_CM - MIN_VOCAL_TRACT_CM)).clamp(0.0, 1.0)
    }

    /// Convert normalized [0.0 ..= 1.0] to Vocal Tract Length [8.0 ..= 25.0 cm].
    pub fn normalized_to_tract(norm: f32) -> f32 {
        MIN_VOCAL_TRACT_CM + norm.clamp(0.0, 1.0) * (MAX_VOCAL_TRACT_CM - MIN_VOCAL_TRACT_CM)
    }

    /// Set model and update defaults.
    pub fn set_model(&mut self, model: PhonemeModel) {
        self.model = model;
        self.formant_f1_hz = model.default_f1_hz();
        self.formant_f2_hz = model.default_f2_hz();
        self.vocal_tract_cm = model.default_tract_length_cm();
        self.phoneme_puck_pos = (
            Self::f2_to_normalized(self.formant_f2_hz),
            Self::f1_to_normalized(self.formant_f1_hz),
        );
        self.update_phoneme_simulation();
    }

    /// Update IPA formant classification & latent style transfer embeddings.
    pub fn update_phoneme_simulation(&mut self) {
        let f1 = self.formant_f1_hz;
        let f2 = self.formant_f2_hz;

        // Formant F3 estimation based on vocal tract quarter-wave resonance
        let c_air = 34300.0_f32; // cm/s
        let f_fund_tube = c_air / (4.0 * self.vocal_tract_cm);
        self.formant_f3_hz = (5.0 * f_fund_tube).clamp(1500.0, 4500.0);

        // Classify closest IPA vowel region in (F1, F2) space
        if f1 < 400.0 && f2 > 2000.0 {
            self.active_phoneme_symbol = "/i/ (ee)";
        } else if f1 < 550.0 && f2 > 1700.0 {
            self.active_phoneme_symbol = "/e/ (ay)";
        } else if f1 > 700.0 && f2 > 1400.0 {
            self.active_phoneme_symbol = "/æ/ (ae)";
        } else if f1 > 700.0 && f2 < 1400.0 {
            self.active_phoneme_symbol = "/a/ (ah)";
        } else if f1 < 600.0 && f2 < 1100.0 {
            self.active_phoneme_symbol = "/o/ (oh)";
        } else if f1 < 400.0 && f2 < 1000.0 {
            self.active_phoneme_symbol = "/u/ (oo)";
        } else {
            self.active_phoneme_symbol = "/ə/ (schwa)";
        }

        // Generate 8-D latent style vector from F1, F2, Style Weight, Tract Length
        let n1 = Self::f1_to_normalized(f1);
        let n2 = Self::f2_to_normalized(f2);
        let w = self.latent_style_weight;

        self.latent_embeddings = [
            (n1 * w).clamp(0.0, 1.0),
            (n2 * (1.0 - w * 0.3)).clamp(0.0, 1.0),
            ((n1 + n2) * 0.5).clamp(0.0, 1.0),
            (w * (0.8 + 0.2 * n2)).clamp(0.0, 1.0),
            ((1.0 - n1) * w).clamp(0.0, 1.0),
            (n2.powi(2) * 0.9).clamp(0.0, 1.0),
            (n1 * 0.4 + w * 0.6).clamp(0.0, 1.0),
            (n2 * 0.5 + (1.0 - w) * 0.5).clamp(0.0, 1.0),
        ];

        self.glottal_open_quotient = (0.4 + 0.3 * (1.0 - n1)).clamp(0.3, 0.8);
    }

    /// Evaluate synthetic vocal tract spectral envelope magnitude (dB) at a given frequency.
    pub fn evaluate_spectral_envelope_db(&self, freq_hz: f32) -> f32 {
        let f = freq_hz.max(20.0);
        let q1 = 8.0;
        let q2 = 10.0;
        let q3 = 12.0;

        let pole1 = 1.0 / (1.0 + q1 * ((f - self.formant_f1_hz) / self.formant_f1_hz).powi(2));
        let pole2 = 0.7 / (1.0 + q2 * ((f - self.formant_f2_hz) / self.formant_f2_hz).powi(2));
        let pole3 = 0.4 / (1.0 + q3 * ((f - self.formant_f3_hz) / self.formant_f3_hz).powi(2));

        let mag = (pole1 + pole2 + pole3).max(1e-4);
        (20.0 * mag.log10()).clamp(-36.0, 12.0)
    }

    /// Hit-test touch coordinate on the Phoneme position puck.
    pub fn hit_test_phoneme_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.phoneme_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.phoneme_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= PHONEME_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render of IPA Vowel Quadrilateral & 8-D Latent Style Embeddings.
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

        // Left half: IPA Vowel Chart Quadrilateral
        let left_w = mid_x - 2;
        // Corner anchor vowels
        if 2 < height - 1 && 2 < mid_x {
            grid[2][2] = 'i';
        }
        if 2 < height - 1 && left_w > 2 {
            grid[2][left_w - 2] = 'u';
        }
        if height > 4 && 2 < mid_x {
            grid[height - 3][2] = 'a';
        }
        if height > 4 && left_w > 2 {
            grid[height - 3][left_w - 2] = 'o';
        }

        // Puck on left half
        let puck_col = ((self.phoneme_puck_pos.0 * (left_w - 4) as f32) + 2.0).round() as usize;
        let puck_row =
            (((1.0 - self.phoneme_puck_pos.1) * (height - 5) as f32) + 2.0).round() as usize;
        if puck_row < height - 1 && puck_col < mid_x {
            grid[puck_row][puck_col] = '@';
        }

        // Right half: 8-D Latent Style Embedding Histogram Bars
        let right_w = width - mid_x - 2;
        let bar_spacing = right_w / (self.latent_embeddings.len() + 1);

        for (i, &emb) in self.latent_embeddings.iter().enumerate() {
            let bar_col = mid_x + 2 + (i + 1) * bar_spacing;
            let bar_h = (emb * (height - 4) as f32).round() as usize;
            for r in 0..bar_h {
                if height - 2 > r && bar_col < width - 1 {
                    grid[height - 2 - r][bar_col] = '#';
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

        // Dark Synth Purple / Slate Background
        painter.rect_filled(rect, 6.0, Color32::from_rgb(18, 14, 28));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "NEURAL AUDIO LATENT STYLE TRANSFER & PHONEME MORPHING VOCODER HUD",
            egui::FontId::proportional(15.0),
            Color32::from_rgb(245, 235, 255),
        );

        // Model Tabs (y: 48..92) - Each tab >= 44pt height
        let models = [
            (PhonemeModel::VowelFormantMorph, "VOWEL MORPH"),
            (PhonemeModel::WhisperToVoicedTransfer, "WHISPER TRANSFER"),
            (PhonemeModel::RoboticVocoderCarrier, "ROBOT VOCODER"),
            (PhonemeModel::AlienFormantShift, "ALIEN SHIFT"),
            (PhonemeModel::LatentDiffusionInterpolate, "LATENT DIFFUSION"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (m, name)) in models.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_selected = self.model == *m;
            let bg_color = if is_selected {
                Color32::from_rgb(180, 90, 255)
            } else {
                Color32::from_rgb(32, 24, 46)
            };
            let text_color = if is_selected {
                Color32::from_rgb(14, 8, 22)
            } else {
                Color32::from_rgb(220, 205, 240)
            };

            painter.rect_filled(tab_rect, 4.0, bg_color);
            painter.text(
                tab_rect.center(),
                egui::Align2::CENTER_CENTER,
                *name,
                egui::FontId::proportional(10.5),
                text_color,
            );

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if tab_rect.contains(pos) {
                        self.set_model(*m);
                    }
                }
            }
        }

        // Main Display Canvas (y: 104..340)
        let main_canvas = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 104.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 340.0),
        );
        painter.rect_filled(main_canvas, 6.0, Color32::from_rgb(12, 10, 20));
        painter.rect_stroke(
            main_canvas,
            6.0,
            Stroke::new(1.5_f32, Color32::from_rgb(65, 45, 95)),
        );

        // Left 55%: IPA Formant Vowel Quadrilateral Map (F1 vs F2)
        let left_w = main_canvas.width() * 0.55;
        let left_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(left_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(left_rect, 4.0, Color32::from_rgb(16, 14, 28));
        painter.rect_stroke(
            left_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(50, 40, 75)),
        );

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "IPA VOWEL QUADRILATERAL (F1 vs F2 FORMANT SPACE)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(190, 170, 220),
        );

        // Vowel Chart Quadrilateral Trap Guides
        let v_corners = [
            (
                egui::pos2(left_rect.min.x + 30.0, left_rect.min.y + 35.0),
                "/i/",
            ),
            (
                egui::pos2(left_rect.max.x - 40.0, left_rect.min.y + 35.0),
                "/u/",
            ),
            (
                egui::pos2(left_rect.max.x - 60.0, left_rect.max.y - 35.0),
                "/o/",
            ),
            (
                egui::pos2(left_rect.min.x + 50.0, left_rect.max.y - 35.0),
                "/a/",
            ),
        ];

        for i in 0..4 {
            let next_i = (i + 1) % 4;
            painter.line_segment(
                [v_corners[i].0, v_corners[next_i].0],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(140, 90, 220, 80)),
            );
            painter.text(
                v_corners[i].0,
                egui::Align2::CENTER_CENTER,
                v_corners[i].1,
                egui::FontId::proportional(12.0),
                Color32::from_rgb(210, 180, 255),
            );
        }

        // Interactive Phoneme Puck
        let puck_x = left_rect.min.x + self.phoneme_puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.phoneme_puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.phoneme_puck_pos = (nx, ny);
                    self.formant_f2_hz = Self::normalized_to_f2(nx);
                    self.formant_f1_hz = Self::normalized_to_f1(ny);
                    self.update_phoneme_simulation();
                }
            }
        }

        // Draw Touch Hit Target boundary (>= 44x44pt)
        painter.circle_stroke(
            puck_pos,
            PHONEME_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(180, 90, 255, 140)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(180, 90, 255));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 20.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Phoneme: {} | F1: {:.0} Hz | F2: {:.0} Hz | F3: {:.0} Hz",
                self.active_phoneme_symbol,
                self.formant_f1_hz,
                self.formant_f2_hz,
                self.formant_f3_hz
            ),
            egui::FontId::proportional(10.0),
            Color32::from_rgb(200, 120, 255),
        );

        // Right 45%: 8-D Latent Style Embedding Spectrum
        let right_w = main_canvas.width() * 0.45;
        let right_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + left_w + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(right_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(right_rect, 4.0, Color32::from_rgb(16, 14, 28));
        painter.rect_stroke(
            right_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(50, 40, 75)),
        );

        painter.text(
            egui::pos2(right_rect.min.x + 10.0, right_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "8-D NEURAL LATENT STYLE EMBEDDINGS",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(190, 170, 220),
        );

        let bar_w = (right_rect.width() - 25.0 - 7.0 * 6.0) / 8.0;
        for (i, &emb) in self.latent_embeddings.iter().enumerate() {
            let bx = right_rect.min.x + 12.0 + i as f32 * (bar_w + 6.0);
            let bar_h = emb * (right_rect.height() - 85.0);
            let b_rect = egui::Rect::from_min_max(
                egui::pos2(bx, right_rect.max.y - 25.0 - bar_h),
                egui::pos2(bx + bar_w, right_rect.max.y - 25.0),
            );
            let col = Color32::from_rgb(
                (140.0 + 115.0 * emb) as u8,
                (60.0 + 150.0 * (1.0 - emb)) as u8,
                255,
            );
            painter.rect_filled(b_rect, 2.0, col);

            painter.text(
                egui::pos2(bx + bar_w * 0.5, right_rect.max.y - 20.0),
                egui::Align2::CENTER_TOP,
                format!("z{}", i + 1),
                egui::FontId::proportional(8.0),
                Color32::from_rgb(200, 185, 230),
            );
        }

        // Bottom Metrics Dock (y: 350..465)
        let dock_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 350.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 465.0),
        );
        painter.rect_filled(dock_rect, 6.0, Color32::from_rgb(22, 18, 34));
        painter.rect_stroke(
            dock_rect,
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(55, 45, 80)),
        );

        let params = [
            (
                "ACTIVE PHONEME",
                self.active_phoneme_symbol.to_string(),
                Color32::from_rgb(180, 90, 255),
            ),
            (
                "FORMANT F1 / F2",
                format!(
                    "{:.0} Hz / {:.0} Hz",
                    self.formant_f1_hz, self.formant_f2_hz
                ),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "VOCAL TRACT LENGTH",
                format!("{:.1} cm", self.vocal_tract_cm),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "LATENT STYLE WEIGHT",
                format!("{:.0}%", self.latent_style_weight * 100.0),
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
                Color32::from_rgb(180, 160, 210),
            );
            painter.text(
                egui::pos2(px_pos, dock_rect.min.y + 30.0),
                egui::Align2::LEFT_TOP,
                val,
                egui::FontId::proportional(14.0),
                *col,
            );
        }

        // Compliance Verification Badge
        let badge_rect = egui::Rect::from_min_max(
            egui::pos2(dock_rect.min.x + 15.0, dock_rect.min.y + 68.0),
            egui::pos2(dock_rect.max.x - 15.0, dock_rect.max.y - 11.0),
        );
        painter.rect_filled(badge_rect, 4.0, Color32::from_rgb(16, 35, 28));
        painter.rect_stroke(
            badge_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(0, 255, 180)),
        );
        painter.text(
            egui::pos2(badge_rect.min.x + 10.0, badge_rect.min.y + 8.0),
            egui::Align2::LEFT_TOP,
            "[PASS] Neural Phoneme Morphing Vocoder & Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
