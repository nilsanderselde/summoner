// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Psychoacoustic Tonality vs Noisiness Spectral Flatness Index & Harmonic-to-Noise Ratio (HNR) HUD (Step 1632).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const SPECTRAL_FLATNESS_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_SFM_INDEX: f32 = 0.0;
pub const MAX_SFM_INDEX: f32 = 1.0;
pub const MIN_HNR_DB: f32 = -20.0;
pub const MAX_HNR_DB: f32 = 40.0;

/// Psychoacoustic spectral analysis profiles for tonality and noisiness decomposition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpectralProfileType {
    PureToneLead,        // Low SFM (<0.08), High HNR (+32 dB), pure sinusoidal resonance
    HarmonicPolyChoir,   // Moderate SFM (0.24), Balanced HNR (+18 dB), rich harmonic series
    PercussiveTransient, // High SFM (0.72), Low HNR (+4 dB), noise-burst transient energy
    AmbientAtmosphere,   // Very high SFM (0.92), Negative HNR (-6 dB), diffuse stochastic field
    MasteringFullMix,    // Mastering reference balance (SFM 0.35, HNR +15 dB)
}

impl SpectralProfileType {
    pub fn profile_name(&self) -> &'static str {
        match self {
            Self::PureToneLead => "PURE TONE / MONO LEAD",
            Self::HarmonicPolyChoir => "HARMONIC POLYPHONIC CHOIR",
            Self::PercussiveTransient => "PERCUSSIVE TRANSIENT / RIMSHOT",
            Self::AmbientAtmosphere => "AMBIENT ATMOSPHERE / NOISE",
            Self::MasteringFullMix => "MASTERING FULL MIX BALANCE",
        }
    }

    pub fn nominal_sfm(&self) -> f32 {
        match self {
            Self::PureToneLead => 0.06,
            Self::HarmonicPolyChoir => 0.24,
            Self::PercussiveTransient => 0.72,
            Self::AmbientAtmosphere => 0.92,
            Self::MasteringFullMix => 0.35,
        }
    }

    pub fn nominal_hnr_db(&self) -> f32 {
        match self {
            Self::PureToneLead => 32.0,
            Self::HarmonicPolyChoir => 18.0,
            Self::PercussiveTransient => 4.0,
            Self::AmbientAtmosphere => -6.0,
            Self::MasteringFullMix => 15.0,
        }
    }

    pub fn nominal_wiener_entropy(&self) -> f32 {
        match self {
            Self::PureToneLead => 0.04,
            Self::HarmonicPolyChoir => 0.22,
            Self::PercussiveTransient => 0.68,
            Self::AmbientAtmosphere => 0.95,
            Self::MasteringFullMix => 0.32,
        }
    }

    pub fn nominal_tonality_factor(&self) -> f32 {
        match self {
            Self::PureToneLead => 0.94,
            Self::HarmonicPolyChoir => 0.76,
            Self::PercussiveTransient => 0.28,
            Self::AmbientAtmosphere => 0.08,
            Self::MasteringFullMix => 0.65,
        }
    }
}

/// Psychoacoustic tonality vs noisiness spectral flatness index & HNR HUD.
#[derive(Debug, Clone)]
pub struct SpectralFlatnessView {
    pub profile: SpectralProfileType,
    pub spectral_flatness_measure: f32, // Wiener entropy / SFM in [0.0, 1.0]
    pub harmonic_to_noise_ratio_db: f32, // HNR in [-20.0, +40.0] dB
    pub wiener_entropy: f32,
    pub tonality_factor: f32,
    pub spectral_crest_factor_db: f32,
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    pub bark_band_flatness: [f32; 8], // Sub, Bass, LowMid, Mid, HighMid, Presence, Brilliance, Air
    pub color_palette: ContrastColorPalette,
}

impl Default for SpectralFlatnessView {
    fn default() -> Self {
        Self::new()
    }
}

impl SpectralFlatnessView {
    pub fn new() -> Self {
        let mut view = Self {
            profile: SpectralProfileType::HarmonicPolyChoir,
            spectral_flatness_measure: 0.24,
            harmonic_to_noise_ratio_db: 18.0,
            wiener_entropy: 0.22,
            tonality_factor: 0.76,
            spectral_crest_factor_db: 14.5,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            bark_band_flatness: [0.15, 0.18, 0.22, 0.28, 0.32, 0.38, 0.45, 0.52],
            color_palette: ContrastColorPalette::default(),
        };
        view.puck_pos = (
            view.spectral_flatness_measure,
            Self::hnr_to_normalized(view.harmonic_to_noise_ratio_db),
        );
        view.update_flatness_metrics();
        view
    }

    pub fn hnr_to_normalized(hnr: f32) -> f32 {
        let val = hnr.clamp(MIN_HNR_DB, MAX_HNR_DB);
        ((val - MIN_HNR_DB) / (MAX_HNR_DB - MIN_HNR_DB)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_hnr(norm: f32) -> f32 {
        MIN_HNR_DB + norm.clamp(0.0, 1.0) * (MAX_HNR_DB - MIN_HNR_DB)
    }

    pub fn set_profile(&mut self, prof: SpectralProfileType) {
        self.profile = prof;
        self.spectral_flatness_measure = prof.nominal_sfm();
        self.harmonic_to_noise_ratio_db = prof.nominal_hnr_db();
        self.wiener_entropy = prof.nominal_wiener_entropy();
        self.tonality_factor = prof.nominal_tonality_factor();
        self.puck_pos = (
            self.spectral_flatness_measure,
            Self::hnr_to_normalized(self.harmonic_to_noise_ratio_db),
        );
        self.update_flatness_metrics();
    }

    pub fn update_flatness_metrics(&mut self) {
        let sfm = self.spectral_flatness_measure.clamp(0.0, 1.0);
        let hnr = self.harmonic_to_noise_ratio_db.clamp(-20.0, 40.0);

        self.wiener_entropy =
            (sfm * 0.95 + 0.05 * (1.0 - (hnr / 40.0).clamp(0.0, 1.0))).clamp(0.0, 1.0);
        self.tonality_factor = ((1.0 - sfm) * 0.65 + ((hnr + 20.0) / 60.0) * 0.35).clamp(0.0, 1.0);
        self.spectral_crest_factor_db = (24.0 * (1.0 - sfm) + (hnr * 0.2)).clamp(0.0, 36.0);

        // Calculate 8 Bark subband flatness distribution curves
        let band_biases = [0.6, 0.7, 0.85, 1.0, 1.15, 1.3, 1.45, 1.6];
        for (i, &bias) in band_biases.iter().enumerate() {
            let val = (sfm * bias * (1.0 - (hnr / 80.0))).clamp(0.02, 0.98);
            self.bark_band_flatness[i] = val;
        }
    }

    pub fn hit_test_flatness_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= SPECTRAL_FLATNESS_PUCK_HIT_RADIUS
    }

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

        let left_w = mid_x - 2;
        let p_row = (((1.0 - self.puck_pos.1) * (height - 5) as f32) + 2.0).round() as usize;
        let p_col = ((self.puck_pos.0 * (left_w - 4) as f32) + 2.0).round() as usize;
        if p_row < height - 1 && p_col < mid_x {
            grid[p_row][p_col] = 'F';
        }

        let right_w = width - mid_x - 3;
        for (i, &flat) in self.bark_band_flatness.iter().enumerate().take(height - 4) {
            let row = 2 + i;
            let bar_len = (flat.clamp(0.0, 1.0) * right_w as f32).round() as usize;
            for c in 0..bar_len {
                if mid_x + 2 + c < width - 1 {
                    grid[row][mid_x + 2 + c] = '#';
                }
            }
        }

        grid.into_iter()
            .map(|r| r.into_iter().collect::<String>())
            .collect()
    }

    #[cfg(feature = "gui")]
    pub fn show(&mut self, ui: &mut egui::Ui) {
        let bg_color = Color32::from_rgb(14, 18, 28);
        let card_bg = Color32::from_rgb(20, 26, 40);
        let border_color = Color32::from_rgb(45, 60, 85);
        let accent_cyan = Color32::from_rgb(0, 229, 255);
        let accent_green = Color32::from_rgb(0, 255, 180);
        let accent_amber = Color32::from_rgb(255, 180, 0);
        let text_white = Color32::from_rgb(240, 245, 255);
        let text_dim = Color32::from_rgb(160, 180, 205);

        egui::Frame::none().fill(bg_color).show(ui, |ui| {
            ui.set_min_size(egui::vec2(800.0, 480.0));
            ui.add_space(8.0);

            // Title and Header
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.heading(
                    egui::RichText::new("SPECTRAL FLATNESS & TONALITY / HNR HUD")
                        .size(18.0)
                        .color(text_white)
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(16.0);
                    ui.label(
                        egui::RichText::new(self.profile.profile_name())
                            .size(13.0)
                            .color(accent_amber)
                            .strong(),
                    );
                });
            });

            ui.add_space(6.0);

            // Profile selector tabs (min 44pt touch hit targets)
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                let profiles = [
                    (SpectralProfileType::PureToneLead, "Pure Tone (Lead)"),
                    (SpectralProfileType::HarmonicPolyChoir, "Polyphonic Choir"),
                    (
                        SpectralProfileType::PercussiveTransient,
                        "Percussive Transient",
                    ),
                    (SpectralProfileType::AmbientAtmosphere, "Ambient Noise"),
                    (SpectralProfileType::MasteringFullMix, "Full Mix Balance"),
                ];

                for (prof, label) in profiles {
                    let is_active = self.profile == prof;
                    let btn_bg = if is_active {
                        accent_cyan
                    } else {
                        Color32::from_rgb(32, 44, 66)
                    };
                    let btn_fg = if is_active {
                        Color32::BLACK
                    } else {
                        text_white
                    };

                    let btn = egui::Button::new(
                        egui::RichText::new(label).size(12.0).color(btn_fg).strong(),
                    )
                    .fill(btn_bg)
                    .min_size(egui::vec2(138.0, 44.0));

                    if ui.add(btn).clicked() {
                        self.set_profile(prof);
                    }
                    ui.add_space(4.0);
                }
            });

            ui.add_space(8.0);

            // Main interactive 2D Canvas split: Left XY Pad (SFM vs HNR), Right Bark Subbands
            let (canvas_rect, response) =
                ui.allocate_exact_size(egui::vec2(768.0, 230.0), egui::Sense::click_and_drag());

            let painter = ui.painter_at(canvas_rect);
            painter.rect_filled(canvas_rect, 6.0, card_bg);
            painter.rect_stroke(canvas_rect, 6.0, Stroke::new(1.5_f32, border_color));

            let left_w = canvas_rect.width() * 0.52;
            let left_rect = egui::Rect::from_min_size(
                canvas_rect.min,
                egui::vec2(left_w, canvas_rect.height()),
            );
            let right_rect = egui::Rect::from_min_size(
                egui::pos2(canvas_rect.min.x + left_w, canvas_rect.min.y),
                egui::vec2(canvas_rect.width() - left_w, canvas_rect.height()),
            );

            // Left Section (XY Pad)
            painter.line_segment(
                [
                    egui::pos2(left_rect.max.x, left_rect.min.y + 10.0),
                    egui::pos2(left_rect.max.x, left_rect.max.y - 10.0),
                ],
                Stroke::new(1.0_f32, border_color),
            );

            for g in 1..4 {
                let frac = g as f32 * 0.25;
                let gx = left_rect.min.x + 12.0 + (left_rect.width() - 24.0) * frac;
                let gy = left_rect.min.y + 12.0 + (left_rect.height() - 24.0) * frac;

                painter.line_segment(
                    [
                        egui::pos2(gx, left_rect.min.y + 12.0),
                        egui::pos2(gx, left_rect.max.y - 12.0),
                    ],
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 110, 80)),
                );
                painter.line_segment(
                    [
                        egui::pos2(left_rect.min.x + 12.0, gy),
                        egui::pos2(left_rect.max.x - 12.0, gy),
                    ],
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 110, 80)),
                );
            }

            let pad_inner = egui::Rect::from_min_max(
                egui::pos2(left_rect.min.x + 16.0, left_rect.min.y + 16.0),
                egui::pos2(left_rect.max.x - 16.0, left_rect.max.y - 16.0),
            );

            if response.dragged() || response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let norm_x = ((pos.x - pad_inner.min.x) / pad_inner.width()).clamp(0.0, 1.0);
                    let norm_y =
                        (1.0 - ((pos.y - pad_inner.min.y) / pad_inner.height())).clamp(0.0, 1.0);
                    self.puck_pos = (norm_x, norm_y);
                    self.spectral_flatness_measure = norm_x;
                    self.harmonic_to_noise_ratio_db = Self::normalized_to_hnr(norm_y);
                    self.update_flatness_metrics();
                }
            }

            let puck_screen_x = pad_inner.min.x + self.puck_pos.0 * pad_inner.width();
            let puck_screen_y = pad_inner.min.y + (1.0 - self.puck_pos.1) * pad_inner.height();
            let puck_center = egui::pos2(puck_screen_x, puck_screen_y);

            painter.line_segment(
                [
                    egui::pos2(pad_inner.min.x, puck_center.y),
                    egui::pos2(pad_inner.max.x, puck_center.y),
                ],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 120)),
            );
            painter.line_segment(
                [
                    egui::pos2(puck_center.x, pad_inner.min.y),
                    egui::pos2(puck_center.x, pad_inner.max.y),
                ],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 120)),
            );

            painter.circle_stroke(
                puck_center,
                SPECTRAL_FLATNESS_PUCK_HIT_RADIUS,
                Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 140)),
            );
            painter.circle_filled(puck_center, 12.0, accent_cyan);
            painter.circle_filled(puck_center, 4.0, Color32::WHITE);

            painter.text(
                egui::pos2(pad_inner.min.x, pad_inner.min.y - 12.0),
                egui::Align2::LEFT_BOTTOM,
                "SPECTRAL FLATNESS (X) / HNR dB (Y)",
                egui::FontId::proportional(11.0),
                accent_cyan,
            );

            // Right Section (Bark Subband Flatness)
            painter.text(
                egui::pos2(right_rect.min.x + 16.0, right_rect.min.y + 14.0),
                egui::Align2::LEFT_TOP,
                "BARK SUBBAND FLATNESS PROFILE",
                egui::FontId::proportional(11.0),
                accent_green,
            );

            let band_names = [
                "Sub (20-100Hz)",
                "Bass (100-250Hz)",
                "Low-Mid (250-600Hz)",
                "Mid (600-1.5kHz)",
                "High-Mid (1.5-3kHz)",
                "Presence (3-6kHz)",
                "Brilliance (6-12kHz)",
                "Air (12-20kHz)",
            ];

            let bar_y_start = right_rect.min.y + 36.0;
            let bar_h = 16.0;
            let bar_spacing = 22.0;
            let bar_max_w = right_rect.width() - 170.0;

            for (i, &flat) in self.bark_band_flatness.iter().enumerate() {
                let y = bar_y_start + i as f32 * bar_spacing;
                let label = band_names[i];

                painter.text(
                    egui::pos2(right_rect.min.x + 16.0, y + 8.0),
                    egui::Align2::LEFT_CENTER,
                    label,
                    egui::FontId::proportional(10.0),
                    text_dim,
                );

                let bar_x = right_rect.min.x + 130.0;
                let bar_rect_bg =
                    egui::Rect::from_min_size(egui::pos2(bar_x, y), egui::vec2(bar_max_w, bar_h));
                painter.rect_filled(bar_rect_bg, 3.0, Color32::from_rgb(12, 16, 24));
                painter.rect_stroke(bar_rect_bg, 3.0, Stroke::new(1.0_f32, border_color));

                let fill_w = flat.clamp(0.0, 1.0) * bar_max_w;
                let fill_color = if flat > 0.65 {
                    Color32::from_rgb(255, 107, 43) // Noise dominant in orange
                } else if flat > 0.30 {
                    accent_amber // Intermediate
                } else {
                    accent_cyan // Tonal dominant in cyan
                };

                let bar_fill =
                    egui::Rect::from_min_size(egui::pos2(bar_x, y), egui::vec2(fill_w, bar_h));
                painter.rect_filled(bar_fill, 3.0, fill_color);

                let val_str = format!("{:.2}", flat);
                painter.text(
                    egui::pos2(bar_x + bar_max_w + 8.0, y + 8.0),
                    egui::Align2::LEFT_CENTER,
                    val_str,
                    egui::FontId::proportional(10.0),
                    text_white,
                );
            }

            ui.add_space(8.0);

            // Bottom Metrics Bar
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new("TONALITY:")
                        .size(12.0)
                        .color(accent_cyan)
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(format!("{:.1}%", self.tonality_factor * 100.0))
                        .size(12.0)
                        .color(text_white),
                );

                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new("WIENER ENTROPY:")
                        .size(12.0)
                        .color(accent_green)
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(format!("{:.3}", self.wiener_entropy))
                        .size(12.0)
                        .color(text_white),
                );

                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new("CREST FACTOR:")
                        .size(12.0)
                        .color(accent_amber)
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(format!("{:.1} dB", self.spectral_crest_factor_db))
                        .size(12.0)
                        .color(text_white),
                );

                ui.add_space(16.0);
                ui.label(egui::RichText::new("HNR:").size(12.0).color(text_dim));
                let hnr_slider =
                    egui::Slider::new(&mut self.harmonic_to_noise_ratio_db, -20.0..=40.0)
                        .suffix(" dB")
                        .show_value(true);
                if ui.add(hnr_slider).changed() {
                    self.puck_pos.1 = Self::hnr_to_normalized(self.harmonic_to_noise_ratio_db);
                    self.update_flatness_metrics();
                }
            });
        });
    }
}
