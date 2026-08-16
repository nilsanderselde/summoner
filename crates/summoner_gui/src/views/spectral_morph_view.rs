// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Real-Time Spectral Morphing Crossfader with Harmonic Centroid Tracking & Formant Curves (Step 1384).

use crate::layout_math::Rect;
use crate::touch_controls::{ContrastColorPalette, MIN_HIT_TARGET_PT};

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const MORPH_CROSSFADER_HANDLE_RADIUS: f32 = 22.0; // 44x44pt touch box
pub const NUM_SPECTRAL_BINS: usize = 64;

/// Spectral morphing interpolation mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpectralMorphMode {
    LinearInterpolation,
    EqualPowerSpectral,
    FormantPreservingWarp,
    PhaseVocoderSmear,
    ConvolutionalBlend,
}

impl SpectralMorphMode {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::LinearInterpolation => "Linear FFT Morph",
            Self::EqualPowerSpectral => "Equal Power Blend",
            Self::FormantPreservingWarp => "Formant Preservation",
            Self::PhaseVocoderSmear => "Vocoder Smear",
            Self::ConvolutionalBlend => "Spectral Convolution",
        }
    }
}

/// Vowel formant resonance preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VowelFormant {
    None,
    VowelA, // "Ah" (700Hz, 1200Hz, 2500Hz)
    VowelE, // "Eh" (500Hz, 1800Hz, 2500Hz)
    VowelI, // "Ee" (300Hz, 2300Hz, 3000Hz)
    VowelO, // "Oh" (400Hz, 800Hz, 2400Hz)
    VowelU, // "Oo" (300Hz, 700Hz, 2200Hz)
}

impl VowelFormant {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::None => "Off (Flat)",
            Self::VowelA => "Vowel /a/ (Ah)",
            Self::VowelE => "Vowel /e/ (Eh)",
            Self::VowelI => "Vowel /i/ (Ee)",
            Self::VowelO => "Vowel /o/ (Oh)",
            Self::VowelU => "Vowel /u/ (Oo)",
        }
    }

    /// Formant frequencies (F1, F2, F3) in Hz.
    pub fn formant_frequencies_hz(&self) -> Option<[f32; 3]> {
        match self {
            Self::None => None,
            Self::VowelA => Some([700.0, 1200.0, 2500.0]),
            Self::VowelE => Some([500.0, 1800.0, 2500.0]),
            Self::VowelI => Some([300.0, 2300.0, 3000.0]),
            Self::VowelO => Some([400.0, 800.0, 2400.0]),
            Self::VowelU => Some([300.0, 700.0, 2200.0]),
        }
    }
}

/// Real-Time Spectral Morphing Crossfader View (Step 1384).
#[derive(Debug, Clone)]
pub struct SpectralMorphView {
    pub source_a_bins: Vec<f32>, // 64 FFT magnitudes [0.0 ..= 1.0]
    pub source_b_bins: Vec<f32>, // 64 FFT magnitudes [0.0 ..= 1.0]
    pub morphed_bins: Vec<f32>,  // Morphed output spectrum
    pub morph_crossfade: f32,    // 0.0 (100% A) ..= 1.0 (100% B)
    pub mode: SpectralMorphMode,
    pub active_formant: VowelFormant,
    pub formant_shift_semitones: f32, // -12.0 ..= +12.0 st
    pub spectral_tilt_db_oct: f32,    // -6.0 ..= +6.0 dB/oct
    pub harmonic_centroid_hz: f32,    // Real-time tracking
    pub is_dragging_crossfader: bool,
    pub color_palette: ContrastColorPalette,
}

impl Default for SpectralMorphView {
    fn default() -> Self {
        Self::new()
    }
}

impl SpectralMorphView {
    pub const MIN_FREQ_HZ: f32 = 20.0;
    pub const MAX_FREQ_HZ: f32 = 20000.0;

    pub fn new() -> Self {
        let mut source_a = Vec::with_capacity(NUM_SPECTRAL_BINS);
        let mut source_b = Vec::with_capacity(NUM_SPECTRAL_BINS);

        for i in 0..NUM_SPECTRAL_BINS {
            let t = i as f32 / NUM_SPECTRAL_BINS as f32;
            // Source A: Sawtooth harmonic rich low-mid spectrum
            let a_val = (1.0 - t * 0.8) * (1.0 + (t * 8.0 * std::f32::consts::PI).cos() * 0.3);
            source_a.push(a_val.clamp(0.02, 1.0));

            // Source B: High-end resonant shimmer & bells
            let b_val = (t * 0.9 + 0.1) * (1.0 + (t * 12.0 * std::f32::consts::PI).sin() * 0.4);
            source_b.push(b_val.clamp(0.02, 1.0));
        }

        let mut view = Self {
            source_a_bins: source_a,
            source_b_bins: source_b,
            morphed_bins: vec![0.0; NUM_SPECTRAL_BINS],
            morph_crossfade: 0.5,
            mode: SpectralMorphMode::EqualPowerSpectral,
            active_formant: VowelFormant::VowelA,
            formant_shift_semitones: 0.0,
            spectral_tilt_db_oct: 0.0,
            harmonic_centroid_hz: 1200.0,
            is_dragging_crossfader: false,
            color_palette: ContrastColorPalette::default(),
        };

        view.recalculate_morphed_spectrum();
        view
    }

    /// Convert bin index (0..64) to center frequency in Hz (logarithmic distribution).
    pub fn bin_to_frequency_hz(bin_idx: usize) -> f32 {
        let norm = (bin_idx as f32 / (NUM_SPECTRAL_BINS - 1) as f32).clamp(0.0, 1.0);
        let log_min = Self::MIN_FREQ_HZ.log10();
        let log_max = Self::MAX_FREQ_HZ.log10();
        10.0_f32.powf(log_min + norm * (log_max - log_min))
    }

    /// Recompute morphed spectral bins and harmonic centroid.
    pub fn recalculate_morphed_spectrum(&mut self) {
        let xf = self.morph_crossfade.clamp(0.0, 1.0);
        let formants = self.active_formant.formant_frequencies_hz();

        let mut weighted_sum = 0.0;
        let mut total_magnitude = 0.0;

        for i in 0..NUM_SPECTRAL_BINS {
            let a = self.source_a_bins.get(i).copied().unwrap_or(0.0);
            let b = self.source_b_bins.get(i).copied().unwrap_or(0.0);
            let freq = Self::bin_to_frequency_hz(i);

            // Interpolation based on mode
            let mut val = match self.mode {
                SpectralMorphMode::LinearInterpolation => (1.0 - xf) * a + xf * b,
                SpectralMorphMode::EqualPowerSpectral => {
                    let angle = xf * std::f32::consts::FRAC_PI_2;
                    angle.cos() * a + angle.sin() * b
                }
                SpectralMorphMode::FormantPreservingWarp => {
                    let base = (1.0 - xf) * a + xf * b;
                    let warp = 1.0 + (freq / 1000.0).sin() * 0.2 * (xf - 0.5);
                    base * warp
                }
                SpectralMorphMode::PhaseVocoderSmear => {
                    let max_val = a.max(b);
                    let min_val = a.min(b);
                    (1.0 - xf) * a + xf * b + (max_val - min_val) * 0.25
                }
                SpectralMorphMode::ConvolutionalBlend => (a * b).sqrt(),
            };

            // Apply formant boost filter if active
            if let Some(f_freqs) = formants {
                let shift_mul = 2.0_f32.powf(self.formant_shift_semitones / 12.0);
                for &f_center in &f_freqs {
                    let shifted_f = f_center * shift_mul;
                    let q = 3.5;
                    let diff = (freq / shifted_f).ln().abs();
                    let resonance = (-diff * q).exp() * 0.6;
                    val += resonance;
                }
            }

            // Apply spectral tilt
            let octaves = (freq / 1000.0).log2();
            let tilt_gain = 10.0_f32.powf((self.spectral_tilt_db_oct * octaves) / 20.0);
            val = (val * tilt_gain).clamp(0.0, 1.2);

            if i < self.morphed_bins.len() {
                self.morphed_bins[i] = val;
            }

            weighted_sum += freq * val;
            total_magnitude += val;
        }

        if total_magnitude > 0.001 {
            self.harmonic_centroid_hz = weighted_sum / total_magnitude;
        }
    }

    /// Set morph crossfader ratio [0.0 ..= 1.0] and update calculations.
    pub fn set_crossfade(&mut self, crossfade: f32) {
        self.morph_crossfade = crossfade.clamp(0.0, 1.0);
        self.recalculate_morphed_spectrum();
    }

    /// Hit-test crossfader thumb handle (>=44x44pt touch box).
    pub fn hit_test_crossfader(&self, pos: (f32, f32), track_rect: Rect) -> bool {
        let handle_x = track_rect.x + self.morph_crossfade * track_rect.width;
        let handle_y = track_rect.y + track_rect.height * 0.5;
        let dx = pos.0 - handle_x;
        let dy = pos.1 - handle_y;
        (dx * dx + dy * dy).sqrt() <= MORPH_CROSSFADER_HANDLE_RADIUS
    }

    /// Deterministic ASCII render of the morphed output spectrum.
    pub fn render_ascii(&self, width: usize) -> String {
        let w = width.max(10);
        let mut buf = vec![' '; w];

        for (i, item) in buf.iter_mut().enumerate() {
            let bin_idx = (i * NUM_SPECTRAL_BINS) / w;
            let val = self.morphed_bins.get(bin_idx).copied().unwrap_or(0.0);
            if val > 0.8 {
                *item = '#';
            } else if val > 0.5 {
                *item = '=';
            } else if val > 0.2 {
                *item = '-';
            } else {
                *item = '.';
            }
        }

        buf.into_iter().collect()
    }
}

#[cfg(feature = "gui")]
impl SpectralMorphView {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // 1. Header Toolbar
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("REAL-TIME SPECTRAL MORPHING CROSSFADER")
                        .color(Color32::from_rgb(240, 245, 255))
                        .strong(),
                );
                ui.separator();
                ui.label(
                    egui::RichText::new(format!(
                        "Morph: {:.1}% | Mode: {}",
                        self.morph_crossfade * 100.0,
                        self.mode.display_name()
                    ))
                    .color(Color32::from_rgb(0, 229, 255))
                    .strong(),
                );
                ui.separator();
                ui.label(
                    egui::RichText::new(format!("Centroid: {:.0} Hz", self.harmonic_centroid_hz))
                        .color(Color32::from_rgb(255, 215, 0))
                        .strong(),
                );
            });

            ui.add_space(6.0);

            // 2. Dual Spectral Overlay & Morphed Spectrum Canvas
            let canvas_w = ui.available_width().max(680.0);
            let canvas_h = 180.0;
            let (response, painter) =
                ui.allocate_painter(Vec2::new(canvas_w, canvas_h), egui::Sense::hover());
            let canvas = Rect::new(
                response.rect.min.x,
                response.rect.min.y,
                response.rect.width(),
                response.rect.height(),
            );

            // Canvas Background
            painter.rect_filled(response.rect, 6.0_f32, Color32::from_rgb(10, 14, 22));
            painter.rect_stroke(
                response.rect,
                6.0_f32,
                Stroke::new(1.5_f32, Color32::from_rgb(40, 55, 80)),
            );

            // Frequency Grid Lines (100Hz, 500Hz, 1kHz, 5kHz, 10kHz)
            let freq_guides: [f32; 5] = [100.0_f32, 500.0_f32, 1000.0_f32, 5000.0_f32, 10000.0_f32];
            for f in freq_guides {
                let log_min = Self::MIN_FREQ_HZ.log10();
                let log_max = Self::MAX_FREQ_HZ.log10();
                let norm_x = (f.log10() - log_min) / (log_max - log_min);
                let gx = canvas.x + norm_x * canvas.width;
                painter.line_segment(
                    [
                        egui::pos2(gx, canvas.y),
                        egui::pos2(gx, canvas.y + canvas.height),
                    ],
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 110, 60)),
                );
                let label = if f >= 1000.0_f32 {
                    format!("{:.0}k", f / 1000.0_f32)
                } else {
                    format!("{:.0}", f)
                };
                painter.text(
                    egui::pos2(gx + 3.0_f32, canvas.y + canvas.height - 12.0_f32),
                    egui::Align2::LEFT_BOTTOM,
                    label,
                    egui::FontId::proportional(10.0_f32),
                    Color32::from_rgb(130, 155, 185),
                );
            }

            // Draw Source A Spectrum (Cyan curve)
            let mut prev_a = None;
            for (i, &val) in self.source_a_bins.iter().enumerate() {
                let x = canvas.x + (i as f32 / (NUM_SPECTRAL_BINS - 1) as f32) * canvas.width;
                let y = canvas.y + canvas.height
                    - (val.clamp(0.0_f32, 1.2_f32) / 1.2_f32) * (canvas.height - 20.0_f32);
                let pt = egui::pos2(x, y);
                if let Some(p) = prev_a {
                    painter.line_segment(
                        [p, pt],
                        Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 120)),
                    );
                }
                prev_a = Some(pt);
            }

            // Draw Source B Spectrum (Coral curve)
            let mut prev_b = None;
            for (i, &val) in self.source_b_bins.iter().enumerate() {
                let x = canvas.x + (i as f32 / (NUM_SPECTRAL_BINS - 1) as f32) * canvas.width;
                let y = canvas.y + canvas.height
                    - (val.clamp(0.0_f32, 1.2_f32) / 1.2_f32) * (canvas.height - 20.0_f32);
                let pt = egui::pos2(x, y);
                if let Some(p) = prev_b {
                    painter.line_segment(
                        [p, pt],
                        Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(255, 107, 43, 120)),
                    );
                }
                prev_b = Some(pt);
            }

            // Draw Morphed Output Spectrum (Bold Solid Gold)
            let mut prev_m = None;
            for (i, &val) in self.morphed_bins.iter().enumerate() {
                let x = canvas.x + (i as f32 / (NUM_SPECTRAL_BINS - 1) as f32) * canvas.width;
                let y = canvas.y + canvas.height
                    - (val.clamp(0.0_f32, 1.2_f32) / 1.2_f32) * (canvas.height - 20.0_f32);
                let pt = egui::pos2(x, y);
                if let Some(p) = prev_m {
                    painter.line_segment(
                        [p, pt],
                        Stroke::new(2.5_f32, Color32::from_rgb(255, 215, 0)),
                    );
                }
                prev_m = Some(pt);
            }

            // Draw Harmonic Centroid Indicator Dot
            let log_min = Self::MIN_FREQ_HZ.log10();
            let log_max = Self::MAX_FREQ_HZ.log10();
            let cent_norm_x = ((self.harmonic_centroid_hz.log10() - log_min) / (log_max - log_min))
                .clamp(0.0_f32, 1.0_f32);
            let cent_x = canvas.x + cent_norm_x * canvas.width;
            painter.line_segment(
                [
                    egui::pos2(cent_x, canvas.y),
                    egui::pos2(cent_x, canvas.y + canvas.height),
                ],
                Stroke::new(1.5_f32, Color32::from_rgb(0, 255, 180)),
            );
            painter.circle_filled(
                egui::pos2(cent_x, canvas.y + 16.0_f32),
                5.0_f32,
                Color32::from_rgb(0, 255, 180),
            );
            painter.text(
                egui::pos2(cent_x + 8.0_f32, canvas.y + 10.0_f32),
                egui::Align2::LEFT_TOP,
                format!("{:.0} Hz Centroid", self.harmonic_centroid_hz),
                egui::FontId::proportional(11.0_f32),
                Color32::from_rgb(0, 255, 180),
            );

            ui.add_space(8.0_f32);

            // 3. Tactile Morph Crossfader Track (>=44pt Touch Targets)
            let fader_h = MIN_HIT_TARGET_PT;
            let (f_resp, f_painter) =
                ui.allocate_painter(Vec2::new(canvas_w, fader_h), egui::Sense::click_and_drag());
            let f_track = Rect::new(
                f_resp.rect.min.x,
                f_resp.rect.min.y,
                f_resp.rect.width(),
                f_resp.rect.height(),
            );

            // Track Background
            f_painter.rect_filled(f_resp.rect, 8.0_f32, Color32::from_rgb(18, 24, 36));
            f_painter.rect_stroke(
                f_resp.rect,
                8.0_f32,
                Stroke::new(1.0_f32, Color32::from_rgb(45, 60, 85)),
            );

            // Left / Right Source Labels
            f_painter.text(
                egui::pos2(
                    f_resp.rect.min.x + 14.0_f32,
                    f_resp.rect.min.y + fader_h * 0.5_f32,
                ),
                egui::Align2::LEFT_CENTER,
                "SOURCE A (100% Synth)",
                egui::FontId::proportional(12.0_f32),
                Color32::from_rgb(0, 229, 255),
            );
            f_painter.text(
                egui::pos2(
                    f_resp.rect.max.x - 14.0_f32,
                    f_resp.rect.min.y + fader_h * 0.5_f32,
                ),
                egui::Align2::RIGHT_CENTER,
                "SOURCE B (100% Shimmer)",
                egui::FontId::proportional(12.0_f32),
                Color32::from_rgb(255, 107, 43),
            );

            // Crossfader Handle (>=44x44pt Touch Box)
            let h_x = f_resp.rect.min.x + self.morph_crossfade * f_resp.rect.width();
            let h_center = egui::pos2(h_x, f_resp.rect.min.y + fader_h * 0.5_f32);

            f_painter.circle_stroke(
                h_center,
                MORPH_CROSSFADER_HANDLE_RADIUS,
                Stroke::new(2.0_f32, Color32::from_rgb(255, 215, 0)),
            );
            f_painter.circle_filled(h_center, 14.0_f32, Color32::from_rgb(255, 215, 0));
            f_painter.circle_filled(h_center, 4.0_f32, Color32::from_rgb(10, 14, 22));

            // Crossfader Drag Handling
            if f_resp.dragged() || f_resp.clicked() {
                if let Some(pos) = f_resp.interact_pointer_pos() {
                    let new_xf = ((pos.x - f_track.x) / f_track.width).clamp(0.0_f32, 1.0_f32);
                    self.set_crossfade(new_xf);
                }
            }

            ui.add_space(8.0);

            // 4. Formant Preset Selectors & Morph Mode Buttons (>=44pt Touch Targets)
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Formant Vowel:").strong());
                let vowels = [
                    VowelFormant::None,
                    VowelFormant::VowelA,
                    VowelFormant::VowelE,
                    VowelFormant::VowelI,
                    VowelFormant::VowelO,
                    VowelFormant::VowelU,
                ];
                for v in vowels {
                    let is_act = self.active_formant == v;
                    let btn = egui::Button::new(
                        egui::RichText::new(v.display_name())
                            .color(if is_act {
                                Color32::from_rgb(10, 14, 22)
                            } else {
                                Color32::from_rgb(220, 235, 255)
                            })
                            .strong(),
                    )
                    .min_size(Vec2::new(70.0, MIN_HIT_TARGET_PT))
                    .fill(if is_act {
                        Color32::from_rgb(0, 229, 255)
                    } else {
                        Color32::from_rgb(30, 40, 60)
                    });

                    if ui.add(btn).clicked() {
                        self.active_formant = v;
                        self.recalculate_morphed_spectrum();
                    }
                }
            });

            ui.add_space(8.0);

            // 5. Morph Mode Buttons & Additional Parameters
            ui.horizontal(|ui| {
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Formant Shift").strong());
                        let resp = ui.add(
                            egui::Slider::new(&mut self.formant_shift_semitones, -12.0..=12.0)
                                .text("st"),
                        );
                        if resp.changed() {
                            self.recalculate_morphed_spectrum();
                        }
                    });
                });

                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Spectral Tilt").strong());
                        let resp = ui.add(
                            egui::Slider::new(&mut self.spectral_tilt_db_oct, -6.0..=6.0)
                                .text("dB/oct"),
                        );
                        if resp.changed() {
                            self.recalculate_morphed_spectrum();
                        }
                    });
                });
            });
        });
    }
}
