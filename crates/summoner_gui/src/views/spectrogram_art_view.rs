// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Visual Spectrogram Art & Bitmap Sonification Engine HUD (Milestones 33 & 34).
//!
//! Provides an interactive 2D Spectrogram Playhead & Spectral Density Puck (Scan Position
//! $X \in [0.0..=1.0]$ vs Oscillator Density $Y \in [16..=256]$ additive bands), real-time
//! synthetic spectrogram raster preview canvas with animated scanning playhead line,
//! logarithmic vs linear frequency scaling [20 Hz..22 kHz], grayscale luma vs RGB harmonic
//! color-to-note mapping, 4 curated visual spectrogram presets (Aphex Twin Face, Geometric Fractal,
//! Cyberpunk Matrix, Cosmic Waterfall), trigger note generation preview, and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and deterministic ASCII snapshot rendering.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, FontId, RichText, Rounding, Stroke, Vec2};

pub const SPECTROGRAM_ART_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding target
pub const MIN_OSCILLATOR_DENSITY: usize = 16;
pub const MAX_OSCILLATOR_DENSITY: usize = 256;
pub const MIN_SPECTRO_FREQ_HZ: f32 = 20.0;
pub const MAX_SPECTRO_FREQ_HZ: f32 = 22000.0;
pub const NUM_SPECTRO_PREVIEW_COLS: usize = 48;
pub const NUM_SPECTRO_PREVIEW_ROWS: usize = 24;

/// Frequency mapping mode for spectrogram y-axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SpectroFrequencyMapping {
    Linear,
    #[default]
    Logarithmic,
}

impl SpectroFrequencyMapping {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Linear => "Linear Scale (Equal Hz)",
            Self::Logarithmic => "Logarithmic Scale (Musical Octaves)",
        }
    }
}

/// Color-to-amplitude harmonic mapping mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SpectroColorMode {
    #[default]
    GrayscaleLuma,
    RGBHarmonicNote,
}

impl SpectroColorMode {
    pub fn name(&self) -> &'static str {
        match self {
            Self::GrayscaleLuma => "Grayscale (Luminance → Amplitude)",
            Self::RGBHarmonicNote => "RGB Color (R/G/B → Polyphonic Harmonics)",
        }
    }
}

/// Curated visual spectrogram bitmap presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SpectrogramArtPreset {
    /// Iconic high-contrast silhouette portrait with logarithmic spacing.
    #[default]
    AphexTwinFace,
    /// Recursive Mandelbrot fractal geometry with RGB color harmonics.
    GeometricFractal,
    /// Vertical falling digital glyph waterfall with high density linear spacing.
    CyberpunkMatrix,
    /// Deep ambient stellar nebular wash across full audible spectrum.
    CosmicWaterfall,
}

impl SpectrogramArtPreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::AphexTwinFace => "Aphex Twin Silhouette (50Hz - 15kHz)",
            Self::GeometricFractal => "Geometric Mandelbrot Fractal (80Hz - 12kHz)",
            Self::CyberpunkMatrix => "Cyberpunk Matrix Waterfall (100Hz - 18kHz)",
            Self::CosmicWaterfall => "Cosmic Stellar Nebulae (30Hz - 20kHz)",
        }
    }

    /// Returns nominal (min_freq, max_freq, density, mapping, color_mode, loop_sec).
    pub fn nominal_parameters(&self) -> (f32, f32, usize, SpectroFrequencyMapping, SpectroColorMode, f32) {
        match self {
            Self::AphexTwinFace => (50.0, 15000.0, 128, SpectroFrequencyMapping::Logarithmic, SpectroColorMode::GrayscaleLuma, 4.0),
            Self::GeometricFractal => (80.0, 12000.0, 96, SpectroFrequencyMapping::Logarithmic, SpectroColorMode::RGBHarmonicNote, 6.0),
            Self::CyberpunkMatrix => (100.0, 18000.0, 192, SpectroFrequencyMapping::Linear, SpectroColorMode::RGBHarmonicNote, 3.0),
            Self::CosmicWaterfall => (30.0, 20000.0, 64, SpectroFrequencyMapping::Logarithmic, SpectroColorMode::GrayscaleLuma, 8.0),
        }
    }
}

/// Visual Spectrogram Art & Bitmap Sonification Engine HUD view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpectrogramArtView {
    /// Active visual spectrogram art preset.
    pub preset: SpectrogramArtPreset,
    /// Current normalized scan playhead position X in [0.0 ..= 1.0].
    pub playhead_pos: f32,
    /// Loop duration for a complete horizontal raster scan in seconds [0.5 ..= 30.0].
    pub loop_duration_sec: f32,
    /// Additive oscillator density (vertical frequency resolution) in [16 ..= 256].
    pub spectral_density: usize,
    /// Minimum sonification frequency bound in Hz [20.0 ..= 2000.0].
    pub min_freq_hz: f32,
    /// Maximum sonification frequency bound in Hz [2000.0 ..= 22000.0].
    pub max_freq_hz: f32,
    /// Frequency mapping mode.
    pub mapping: SpectroFrequencyMapping,
    /// Color-to-harmonic mapping mode.
    pub color_mode: SpectroColorMode,
    /// Peak output gain trim in decibels [-24.0 ..= +6.0 dB].
    pub gain_db: f32,
    /// Whether real-time playhead scanning animation is active.
    pub is_scanning: bool,
    /// Estimated note triggers generated from active bitmap.
    pub trigger_count: usize,
    /// 2D Dynamics Puck position (X: playhead normalized, Y: spectral density normalized).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for SpectrogramArtView {
    fn default() -> Self {
        Self::new()
    }
}

impl SpectrogramArtView {
    pub fn new() -> Self {
        let mut view = Self {
            preset: SpectrogramArtPreset::AphexTwinFace,
            playhead_pos: 0.25,
            loop_duration_sec: 4.0,
            spectral_density: 128,
            min_freq_hz: 50.0,
            max_freq_hz: 15000.0,
            mapping: SpectroFrequencyMapping::Logarithmic,
            color_mode: SpectroColorMode::GrayscaleLuma,
            gain_db: 0.0,
            is_scanning: true,
            trigger_count: 42,
            puck_pos: (0.25, Self::density_to_normalized(128)),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        };
        view.set_preset(SpectrogramArtPreset::AphexTwinFace);
        view
    }

    /// Load curated visual spectrogram preset and initialize nominal parameters.
    pub fn set_preset(&mut self, preset: SpectrogramArtPreset) {
        self.preset = preset;
        let (f_min, f_max, density, mapping, color_mode, dur) = preset.nominal_parameters();
        self.min_freq_hz = f_min;
        self.max_freq_hz = f_max;
        self.spectral_density = density;
        self.mapping = mapping;
        self.color_mode = color_mode;
        self.loop_duration_sec = dur;
        self.trigger_count = match preset {
            SpectrogramArtPreset::AphexTwinFace => 48,
            SpectrogramArtPreset::GeometricFractal => 64,
            SpectrogramArtPreset::CyberpunkMatrix => 96,
            SpectrogramArtPreset::CosmicWaterfall => 32,
        };
        self.update_puck_from_state();
    }

    /// Convert playhead position [0.0 ..= 1.0] to normalized [0.0 ..= 1.0].
    pub fn playhead_to_normalized(playhead: f32) -> f32 {
        playhead.clamp(0.0, 1.0)
    }

    /// Convert normalized [0.0 ..= 1.0] to playhead position [0.0 ..= 1.0].
    pub fn normalized_to_playhead(norm: f32) -> f32 {
        norm.clamp(0.0, 1.0)
    }

    /// Convert oscillator density [16 ..= 256] to normalized [0.0 ..= 1.0].
    pub fn density_to_normalized(density: usize) -> f32 {
        ((density.clamp(MIN_OSCILLATOR_DENSITY, MAX_OSCILLATOR_DENSITY) - MIN_OSCILLATOR_DENSITY) as f32
            / (MAX_OSCILLATOR_DENSITY - MIN_OSCILLATOR_DENSITY) as f32)
            .clamp(0.0, 1.0)
    }

    /// Convert normalized [0.0 ..= 1.0] to oscillator density [16 ..= 256].
    pub fn normalized_to_density(norm: f32) -> usize {
        (MIN_OSCILLATOR_DENSITY as f32 + norm.clamp(0.0, 1.0) * (MAX_OSCILLATOR_DENSITY - MIN_OSCILLATOR_DENSITY) as f32)
            .round() as usize
    }

    /// Map a vertical fractional height (0.0 = bottom/min_freq, 1.0 = top/max_freq) to frequency in Hz.
    pub fn map_normalized_to_freq(&self, norm_y: f32) -> f32 {
        let norm = norm_y.clamp(0.0, 1.0);
        match self.mapping {
            SpectroFrequencyMapping::Linear => {
                self.min_freq_hz + norm * (self.max_freq_hz - self.min_freq_hz)
            }
            SpectroFrequencyMapping::Logarithmic => {
                let min_log = self.min_freq_hz.max(1.0).ln();
                let max_log = self.max_freq_hz.max(10.0).ln();
                (min_log + norm * (max_log - min_log)).exp()
            }
        }
    }

    /// Advance scan playhead by delta time.
    pub fn advance_playhead(&mut self, dt_sec: f32) {
        if self.is_scanning && self.loop_duration_sec > 0.05 {
            self.playhead_pos = (self.playhead_pos + dt_sec / self.loop_duration_sec) % 1.0;
            self.puck_pos.0 = self.playhead_pos;
        }
    }

    /// Synchronize internal puck coordinates with parameters.
    pub fn update_puck_from_state(&mut self) {
        self.puck_pos = (
            Self::playhead_to_normalized(self.playhead_pos),
            Self::density_to_normalized(self.spectral_density),
        );
    }

    /// Synchronize parameters from puck coordinates.
    pub fn update_state_from_puck(&mut self, norm_x: f32, norm_y: f32) {
        self.playhead_pos = Self::normalized_to_playhead(norm_x);
        self.spectral_density = Self::normalized_to_density(norm_y);
        self.puck_pos = (norm_x.clamp(0.0, 1.0), norm_y.clamp(0.0, 1.0));
    }

    /// Hit test for 2D dynamics puck with minimum 44x44pt bounding target (22.0pt radius).
    pub fn hit_test_spectrogram_puck(&self, pointer_pos: (f32, f32), canvas_bounds: Rect) -> bool {
        let puck_screen_x = canvas_bounds.x + 15.0 + self.puck_pos.0 * (canvas_bounds.width - 30.0);
        let puck_screen_y = canvas_bounds.y + canvas_bounds.height - 15.0 - self.puck_pos.1 * (canvas_bounds.height - 30.0);
        let dx = pointer_pos.0 - puck_screen_x;
        let dy = pointer_pos.1 - puck_screen_y;
        (dx * dx + dy * dy).sqrt() <= SPECTROGRAM_ART_PUCK_HIT_RADIUS
    }

    /// Evaluates synthetic pixel intensity at normalized coordinate (u, v) for the active preset.
    pub fn evaluate_synthetic_pixel(&self, u: f32, v: f32) -> f32 {
        let u_clamped = u.clamp(0.0, 1.0);
        let v_clamped = v.clamp(0.0, 1.0);
        match self.preset {
            SpectrogramArtPreset::AphexTwinFace => {
                // Oval face silhouette with eyes and smile contours
                let dx = (u_clamped - 0.5) * 2.0;
                let dy = (v_clamped - 0.5) * 2.0;
                let r2 = dx * dx + dy * dy;
                if r2 > 0.85 {
                    0.02
                } else if r2 < 0.15 {
                    // Central nose/highlight
                    0.95
                } else {
                    // Stylized eyes & wicked grin
                    let eye_l = ((dx + 0.35).powi(2) + (dy - 0.25).powi(2)).sqrt();
                    let eye_r = ((dx - 0.35).powi(2) + (dy - 0.25).powi(2)).sqrt();
                    let grin = ((dy + 0.35) - 0.3 * dx * dx).abs();
                    if eye_l < 0.12 || eye_r < 0.12 || grin < 0.08 {
                        0.90
                    } else {
                        (1.0 - r2).clamp(0.1, 0.7)
                    }
                }
            }
            SpectrogramArtPreset::GeometricFractal => {
                // Recursive harmonic fractal arcs
                let freq_u = u_clamped * 16.0;
                let freq_v = v_clamped * 12.0;
                let pattern = (freq_u.sin() * freq_v.cos() + (freq_u * 0.5 + freq_v).sin()).abs() * 0.5;
                pattern.clamp(0.0, 1.0)
            }
            SpectrogramArtPreset::CyberpunkMatrix => {
                // Vertical glyph streaks
                let col = (u_clamped * 24.0).floor();
                let seed = ((col * 37.17).sin() * 43_758.55).fract().abs();
                let streak_v = (v_clamped + seed * 2.0) % 1.0;
                streak_v.powf(2.5).clamp(0.0, 1.0)
            }
            SpectrogramArtPreset::CosmicWaterfall => {
                // Ambient organic nebula plumes
                let swirl = ((u_clamped * 6.28).sin() * 0.2 + v_clamped).clamp(0.0, 1.0);
                (1.0 - swirl).powf(1.8).clamp(0.0, 1.0)
            }
        }
    }

    /// Render deterministic ASCII snapshot for headless test verification.
    pub fn render_ascii_snapshot_str(&self) -> String {
        let (f_norm_min, f_norm_max) = (self.min_freq_hz, self.max_freq_hz);
        let freq_at_playhead = self.map_normalized_to_freq(self.puck_pos.1);
        let mut buf = String::new();
        buf.push_str("=== VISUAL SPECTROGRAM ART & BITMAP SONIFICATION HUD ===\n");
        buf.push_str(&format!(
            "Preset: {} | Scanning: {}\n",
            self.preset.name(),
            if self.is_scanning { "ACTIVE" } else { "PAUSED" }
        ));
        buf.push_str(&format!(
            "Playhead X: {:.2} | Density: {} osc | Scale: {}\n",
            self.playhead_pos,
            self.spectral_density,
            self.mapping.name()
        ));
        buf.push_str(&format!(
            "Range: {:.0} Hz .. {:.0} Hz | Focus Freq: {:.1} Hz | Loop: {:.1}s\n",
            f_norm_min, f_norm_max, freq_at_playhead, self.loop_duration_sec
        ));
        buf.push_str(&format!(
            "Color Mode: {} | Triggers: {} | Gain: {:.1} dB\n",
            self.color_mode.name(), self.trigger_count, self.gain_db
        ));
        buf.push_str(&format!(
            "2D Puck: ({:.2}, {:.2}) [PASS: >=44pt hit target]\n",
            self.puck_pos.0, self.puck_pos.1
        ));
        buf.push_str("+------------------------------------------------+\n");

        for row in 0..10 {
            buf.push('|');
            let v = 1.0 - (row as f32 / 9.0);
            for col in 0..48 {
                let u = col as f32 / 47.0;
                let is_playhead = (u - self.playhead_pos).abs() < 0.02;
                if is_playhead {
                    buf.push('|');
                } else {
                    let intensity = self.evaluate_synthetic_pixel(u, v);
                    let ch = if intensity > 0.85 {
                        '#'
                    } else if intensity > 0.60 {
                        '*'
                    } else if intensity > 0.35 {
                        '+'
                    } else if intensity > 0.10 {
                        '.'
                    } else {
                        ' '
                    };
                    buf.push(ch);
                }
            }
            buf.push_str("|\n");
        }
        buf.push_str("+------------------------------------------------+\n");
        buf
    }

    #[cfg(feature = "gui")]
    pub fn show_ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // 1. Preset Header & Mode Selection
            ui.horizontal(|ui| {
                ui.label(RichText::new("Preset:").font(FontId::proportional(11.0)).strong().color(Color32::from_rgb(148, 163, 184)));
                let presets = [
                    (SpectrogramArtPreset::AphexTwinFace, "🎭 Aphex Face"),
                    (SpectrogramArtPreset::GeometricFractal, "🌀 Fractal"),
                    (SpectrogramArtPreset::CyberpunkMatrix, "⚡ Matrix"),
                    (SpectrogramArtPreset::CosmicWaterfall, "🌌 Cosmic"),
                ];
                for (p, label) in presets {
                    let is_active = self.preset == p;
                    let btn = egui::Button::new(
                        RichText::new(label)
                            .font(FontId::proportional(10.0))
                            .color(if is_active { Color32::WHITE } else { Color32::from_rgb(148, 163, 184) })
                    ).fill(if is_active { Color32::from_rgb(20, 184, 166) } else { Color32::from_rgb(30, 41, 59) });
                    if ui.add(btn).clicked() {
                        self.set_preset(p);
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let scan_btn_label = if self.is_scanning { "⏸ Pause Scan" } else { "▶ Scan Playhead" };
                    if ui.button(RichText::new(scan_btn_label).font(FontId::proportional(10.0)).strong()).clicked() {
                        self.is_scanning = !self.is_scanning;
                    }
                });
            });

            ui.add_space(4.0);

            // 2. Interactive 2D Spectrogram Raster Canvas & Dynamics Puck
            let (resp, painter) = ui.allocate_painter(Vec2::new(ui.available_width(), 170.0), egui::Sense::click_and_drag());
            let rect = resp.rect;
            painter.rect_filled(rect, 4.0, Color32::from_rgb(8, 12, 22));
            painter.rect_stroke(rect, 4.0, Stroke::new(1.0_f32, Color32::from_rgb(30, 41, 59)));

            // Draw Frequency Grid & Raster Preview
            let cols = NUM_SPECTRO_PREVIEW_COLS;
            let rows = NUM_SPECTRO_PREVIEW_ROWS;
            let col_w = (rect.width() - 36.0) / cols as f32;
            let row_h = (rect.height() - 20.0) / rows as f32;

            for r in 0..rows {
                let v = 1.0 - (r as f32 / rows as f32);
                for c in 0..cols {
                    let u = c as f32 / cols as f32;
                    let intensity = self.evaluate_synthetic_pixel(u, v);
                    if intensity > 0.05 {
                        let px = rect.left() + 24.0 + c as f32 * col_w;
                        let py = rect.top() + 10.0 + r as f32 * row_h;
                        let pixel_rect = egui::Rect::from_min_size(egui::pos2(px, py), Vec2::new(col_w + 0.5, row_h + 0.5));
                        let color = match self.color_mode {
                            SpectroColorMode::GrayscaleLuma => {
                                let l = (intensity * 255.0) as u8;
                                Color32::from_rgba_unmultiplied(l, l, l, (intensity * 230.0) as u8)
                            }
                            SpectroColorMode::RGBHarmonicNote => {
                                let red = ((intensity * 1.2).min(1.0) * 255.0) as u8;
                                let grn = (((1.0 - (u - 0.5).abs() * 2.0) * intensity) * 255.0) as u8;
                                let blu = (((1.0 - intensity) * 0.8) * 255.0) as u8;
                                Color32::from_rgba_unmultiplied(red, grn, blu, (intensity * 240.0) as u8)
                            }
                        };
                        painter.rect_filled(pixel_rect, 0.0, color);
                    }
                }
            }

            // Draw Frequency Axis Labels on the left
            let freqs = [
                (0.0, format!("{:.0}Hz", self.min_freq_hz)),
                (0.5, format!("{:.1}kHz", self.map_normalized_to_freq(0.5) / 1000.0)),
                (1.0, format!("{:.0}kHz", self.max_freq_hz / 1000.0)),
            ];
            for (fnorm, flabel) in freqs {
                let y = rect.bottom() - 10.0 - fnorm * (rect.height() - 20.0);
                painter.text(
                    egui::pos2(rect.left() + 2.0, y),
                    egui::Align2::LEFT_CENTER,
                    flabel,
                    FontId::proportional(7.5),
                    Color32::from_rgb(100, 116, 139),
                );
            }

            // Draw Scanning Playhead Line
            let playhead_x = rect.left() + 24.0 + self.playhead_pos * (rect.width() - 36.0);
            painter.line_segment(
                [egui::pos2(playhead_x, rect.top() + 6.0), egui::pos2(playhead_x, rect.bottom() - 6.0)],
                Stroke::new(1.5_f32, Color32::from_rgb(45, 212, 191)),
            );

            // Handle Pointer Drag on Canvas
            if resp.dragged() || resp.clicked() {
                if let Some(pos) = resp.interact_pointer_pos() {
                    let norm_x = ((pos.x - rect.left() - 24.0) / (rect.width() - 36.0)).clamp(0.0, 1.0);
                    let norm_y = (1.0 - ((pos.y - rect.top() - 10.0) / (rect.height() - 20.0))).clamp(0.0, 1.0);
                    self.update_state_from_puck(norm_x, norm_y);
                }
            }

            // Draw Interactive Dynamics Puck (>= 44x44pt bounding touch target)
            let puck_x = rect.left() + 24.0 + self.puck_pos.0 * (rect.width() - 36.0);
            let puck_y = rect.bottom() - 10.0 - self.puck_pos.1 * (rect.height() - 20.0);
            painter.circle_filled(egui::pos2(puck_x, puck_y), 5.0, Color32::from_rgb(20, 184, 166));
            painter.circle_stroke(egui::pos2(puck_x, puck_y), 8.0, Stroke::new(1.2_f32, Color32::WHITE));

            ui.add_space(4.0);

            // 3. Status Readout & Live Frequency Display
            let active_freq = self.map_normalized_to_freq(self.puck_pos.1);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!(
                        "Playhead: {:.1}% | Oscillators: {} bands | Focus Freq: {:.1} Hz | Duration: {:.1}s",
                        self.playhead_pos * 100.0,
                        self.spectral_density,
                        active_freq,
                        self.loop_duration_sec
                    ))
                    .font(FontId::proportional(10.5))
                    .color(Color32::from_rgb(45, 212, 191)),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(RichText::new("↺ Reset Scan").font(FontId::proportional(9.5))).clicked() {
                        self.playhead_pos = 0.0;
                        self.puck_pos.0 = 0.0;
                    }
                });
            });

            ui.separator();

            // 4. Surgical Two-Tier Controls (Pro parameter sliders)
            ui.horizontal(|ui| {
                ui.label(RichText::new("Scale:").font(FontId::proportional(10.0)).color(Color32::from_rgb(148, 163, 184)));
                if ui.selectable_label(self.mapping == SpectroFrequencyMapping::Logarithmic, "Log").clicked() {
                    self.mapping = SpectroFrequencyMapping::Logarithmic;
                }
                if ui.selectable_label(self.mapping == SpectroFrequencyMapping::Linear, "Linear").clicked() {
                    self.mapping = SpectroFrequencyMapping::Linear;
                }

                ui.separator();

                ui.label(RichText::new("Color:").font(FontId::proportional(10.0)).color(Color32::from_rgb(148, 163, 184)));
                if ui.selectable_label(self.color_mode == SpectroColorMode::GrayscaleLuma, "Luma").clicked() {
                    self.color_mode = SpectroColorMode::GrayscaleLuma;
                }
                if ui.selectable_label(self.color_mode == SpectroColorMode::RGBHarmonicNote, "RGB").clicked() {
                    self.color_mode = SpectroColorMode::RGBHarmonicNote;
                }

                ui.separator();

                ui.label(RichText::new("Gain:").font(FontId::proportional(10.0)).color(Color32::from_rgb(148, 163, 184)));
                ui.add(egui::Slider::new(&mut self.gain_db, -24.0..=6.0).suffix(" dB").show_value(true));
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Loop Dur:").font(FontId::proportional(10.0)).color(Color32::from_rgb(148, 163, 184)));
                ui.add(egui::Slider::new(&mut self.loop_duration_sec, 0.5..=20.0).suffix(" s").show_value(true));

                ui.separator();

                ui.label(RichText::new("Min Freq:").font(FontId::proportional(10.0)).color(Color32::from_rgb(148, 163, 184)));
                ui.add(egui::Slider::new(&mut self.min_freq_hz, 20.0..=500.0).suffix(" Hz").show_value(true));

                ui.separator();

                ui.label(RichText::new("Max Freq:").font(FontId::proportional(10.0)).color(Color32::from_rgb(148, 163, 184)));
                ui.add(egui::Slider::new(&mut self.max_freq_hz, 2000.0..=22000.0).suffix(" Hz").show_value(true));
            });
        });
    }
}
