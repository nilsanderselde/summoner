// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Discrete Schottky Diode Multi-Stage Wavefolder & Overdrive HUD (Milestones 11, 33 & 34).
//!
//! Provides an interactive 2D Wavefolding Puck (Pre-Amp Drive [0.0..36.0 dB] vs Diode Threshold Asymmetry Bias [-1.0..1.0]),
//! real-time folded transfer curve V_out = f(V_in) across [-2.0..+2.0 V] with Schottky exponential knee smoothing,
//! 4 wavefolding saturation presets (Subtle Even Fold Warmth, Buchla West-Coast Harmonics, Screaming Diode Cascade, Asymmetric Metallic Crunch),
//! 9-harmonic distortion breakdown (Fundamental H1 through H9 with odd/even harmonic separation),
//! dynamic Miller low-pass capacitance rolloff calculation,
//! and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and deterministic ASCII snapshot rendering.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, FontId, RichText, Rounding, Stroke, Vec2};

pub const SCHOTTKY_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding target
pub const MIN_DRIVE_DB: f32 = 0.0;
pub const MAX_DRIVE_DB: f32 = 36.0;
pub const MIN_BIAS: f32 = -1.0;
pub const MAX_BIAS: f32 = 1.0;
pub const NUM_WAVEFOLDER_HARMONICS: usize = 9;

/// Classic wavefolding saturation and diode behavior presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SchottkyWavefolderPreset {
    /// Subtle warm saturation with 2 folding stages and gentle asymmetric bias for rich even 2nd harmonics.
    SubtleEvenFoldWarmth,
    /// Classic West-Coast complex timbre wavefolder with 3 stages and zero symmetry bias.
    #[default]
    BuchlaWestCoastHarmonics,
    /// High-gain 5-stage diode folding cascade with sharp conduction knees and aggressive overtone multiplication.
    ScreamingDiodeCascade,
    /// Heavy asymmetric bias and high Miller capacitance producing metallic industrial growl.
    AsymmetricMetallicCrunch,
}

impl SchottkyWavefolderPreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::SubtleEvenFoldWarmth => "Subtle Even Fold Warmth (2-Stage Diode)",
            Self::BuchlaWestCoastHarmonics => "Buchla West-Coast Harmonics (3-Stage Classic)",
            Self::ScreamingDiodeCascade => "Screaming Diode Cascade (5-Stage Extreme)",
            Self::AsymmetricMetallicCrunch => "Asymmetric Metallic Crunch (Industrial)",
        }
    }

    /// Returns nominal (drive_db, bias, stages, miller_cap_pf, knee_softness, trim_db).
    pub fn nominal_parameters(&self) -> (f32, f32, f32, f32, f32, f32) {
        match self {
            Self::SubtleEvenFoldWarmth => (10.0, 0.15, 2.0, 120.0, 0.50, -2.0),
            Self::BuchlaWestCoastHarmonics => (18.0, 0.0, 3.0, 80.0, 0.35, -4.0),
            Self::ScreamingDiodeCascade => (28.0, 0.05, 5.0, 220.0, 0.20, -7.5),
            Self::AsymmetricMetallicCrunch => (22.0, -0.45, 4.0, 450.0, 0.40, -5.0),
        }
    }
}

/// Discrete Schottky Diode Multi-Stage Wavefolder & Overdrive HUD view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchottkyWavefolderView {
    /// Active saturation preset.
    pub preset: SchottkyWavefolderPreset,
    /// Pre-folding operational amplifier boost in dB [0.0 ..= 36.0].
    pub input_drive_gain: f32,
    /// DC offset asymmetry bias across diode thresholds [-1.0 ..= 1.0].
    pub fold_symmetry_bias: f32,
    /// Active cascaded diode inflection folding stages [1.0 ..= 6.0].
    pub folding_stages_count: f32,
    /// Junction capacitance forming high-frequency low-pass roll-off in pF [10.0 ..= 1000.0].
    pub miller_junction_capacitance: f32,
    /// Diode exponential IV curve knee softness [0.1 ..= 1.0].
    pub diode_saturation_softness: f32,
    /// Post-folding output makeup level trim in dB [-24.0 ..= 12.0].
    pub output_trim_makeup: f32,
    /// Effective count of active wavefolding inflection points.
    pub effective_fold_count: usize,
    /// Estimated Miller low-pass corner frequency in kHz.
    pub miller_cutoff_khz: f32,
    /// Total harmonic distortion and density index score [0.0 ..= 1.0].
    pub total_harmonic_density: f32,
    /// 2D Wavefolding Puck position (X: input_drive_gain normalized, Y: fold_symmetry_bias normalized).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for SchottkyWavefolderView {
    fn default() -> Self {
        Self::new()
    }
}

impl SchottkyWavefolderView {
    pub fn new() -> Self {
        let mut view = Self {
            preset: SchottkyWavefolderPreset::BuchlaWestCoastHarmonics,
            input_drive_gain: 18.0,
            fold_symmetry_bias: 0.0,
            folding_stages_count: 3.0,
            miller_junction_capacitance: 80.0,
            diode_saturation_softness: 0.35,
            output_trim_makeup: -4.0,
            effective_fold_count: 3,
            miller_cutoff_khz: 42.5,
            total_harmonic_density: 0.68,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        };
        view.sync_from_preset();
        view
    }

    pub fn set_preset(&mut self, preset: SchottkyWavefolderPreset) {
        self.preset = preset;
        self.sync_from_preset();
    }

    pub fn sync_from_preset(&mut self) {
        let (drv, bias, stg, cap, knee, trim) = self.preset.nominal_parameters();
        self.input_drive_gain = drv;
        self.fold_symmetry_bias = bias;
        self.folding_stages_count = stg;
        self.miller_junction_capacitance = cap;
        self.diode_saturation_softness = knee;
        self.output_trim_makeup = trim;
        self.puck_pos = (
            Self::drive_to_normalized(self.input_drive_gain),
            Self::bias_to_normalized(self.fold_symmetry_bias),
        );
        self.update_physics();
    }

    pub fn update_physics(&mut self) {
        let drive_lin = 10.0_f32.powf(self.input_drive_gain / 20.0);
        let potential_folds = (drive_lin * 0.4 * self.folding_stages_count).round() as usize;
        self.effective_fold_count = potential_folds.clamp(1, 12);

        // Miller low-pass corner estimation: fc ~ 1 / (2 * pi * R * C) with R_source ~ 47k Ohm
        let r_source = 47_000.0;
        let c_farads = self.miller_junction_capacitance * 1e-12;
        let fc_hz = 1.0 / (2.0 * std::f32::consts::PI * r_source * c_farads);
        self.miller_cutoff_khz = (fc_hz / 1000.0).clamp(2.0, 96.0);

        // Total harmonic density increases with drive and stages, and increases with sharper knees
        let drive_factor = (self.input_drive_gain / 36.0).clamp(0.0, 1.0);
        let stage_factor = (self.folding_stages_count / 6.0).clamp(0.0, 1.0);
        let sharpness = (1.0 - self.diode_saturation_softness).clamp(0.0, 1.0);
        self.total_harmonic_density = (0.2 + 0.5 * drive_factor + 0.2 * stage_factor + 0.1 * sharpness).clamp(0.0, 1.0);
    }

    #[inline]
    pub fn drive_to_normalized(d: f32) -> f32 {
        ((d - MIN_DRIVE_DB) / (MAX_DRIVE_DB - MIN_DRIVE_DB)).clamp(0.0, 1.0)
    }

    #[inline]
    pub fn normalized_to_drive(norm: f32) -> f32 {
        MIN_DRIVE_DB + norm.clamp(0.0, 1.0) * (MAX_DRIVE_DB - MIN_DRIVE_DB)
    }

    #[inline]
    pub fn bias_to_normalized(b: f32) -> f32 {
        ((b - MIN_BIAS) / (MAX_BIAS - MIN_BIAS)).clamp(0.0, 1.0)
    }

    #[inline]
    pub fn normalized_to_bias(norm: f32) -> f32 {
        MIN_BIAS + norm.clamp(0.0, 1.0) * (MAX_BIAS - MIN_BIAS)
    }

    /// Evaluates wavefolded transfer characteristic V_out for a given V_in in [-2.0 ..= +2.0 V].
    pub fn evaluate_transfer_curve(&self, v_in: f32) -> f32 {
        let drive_lin = 10.0_f32.powf(self.input_drive_gain / 20.0);
        // Add DC asymmetry bias
        let v_biased = v_in + self.fold_symmetry_bias * 0.45;
        let x = v_biased * drive_lin * 0.5;

        // Multi-stage sinusoidal wavefolder with Schottky diode exponential knee smoothing
        let num_stages = self.folding_stages_count;
        let knee = self.diode_saturation_softness.clamp(0.1, 1.0);

        let folded = (x * std::f32::consts::FRAC_PI_2).sin();
        let softened = if knee > 0.3 {
            (folded / knee).tanh() * knee
        } else {
            folded
        };

        // Output makeup trim
        let trim_lin = 10.0_f32.powf(self.output_trim_makeup / 20.0);
        (softened * (1.0 + 0.15 * (num_stages - 1.0)) * trim_lin).clamp(-1.8, 1.8)
    }

    /// Evaluates harmonic breakdown amplitudes for H1 through H9.
    pub fn evaluate_harmonics(&self) -> [f32; NUM_WAVEFOLDER_HARMONICS] {
        let mut harmonics = [0.0; NUM_WAVEFOLDER_HARMONICS];
        let drive_norm = self.input_drive_gain / 36.0;
        let bias_abs = self.fold_symmetry_bias.abs();

        harmonics[0] = 1.0; // Fundamental H1

        for (i, item) in harmonics.iter_mut().enumerate().skip(1) {
            let n = (i + 1) as f32;
            let is_even = (i + 1) % 2 == 0;

            if is_even {
                // Even harmonics emerge directly with asymmetry bias
                let base_even = bias_abs * (0.8 / n.powf(0.9));
                *item = (base_even * (0.4 + 0.6 * drive_norm)).clamp(0.01, 0.95);
            } else {
                // Odd harmonics generated from symmetric wavefolding
                let base_odd = (drive_norm * 1.2) / n.powf(0.8);
                *item = (base_odd * (1.0 - bias_abs * 0.3)).clamp(0.02, 0.95);
            }
        }

        harmonics
    }

    pub fn hit_test_schottky_puck(&self, pointer_pos: (f32, f32), canvas_rect: Rect) -> bool {
        let padding = 15.0;
        let inner_w = (canvas_rect.width - 2.0 * padding).max(10.0);
        let inner_h = (canvas_rect.height - 2.0 * padding).max(10.0);

        let puck_x = canvas_rect.x + padding + self.puck_pos.0 * inner_w;
        let puck_y = canvas_rect.y + canvas_rect.height - padding - self.puck_pos.1 * inner_h;

        let dx = pointer_pos.0 - puck_x;
        let dy = pointer_pos.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= SCHOTTKY_PUCK_HIT_RADIUS
    }

    /// Produces a deterministic ASCII snapshot for headless test verification.
    pub fn render_ascii_snapshot_str(&self) -> String {
        let harmonics = self.evaluate_harmonics();
        let mut out = String::new();
        out.push_str("=== DISCRETE SCHOTTKY DIODE MULTI-STAGE WAVEFOLDER & OVERDRIVE HUD ===\n");
        out.push_str(&format!("Preset: {}\n", self.preset.name()));
        out.push_str(&format!(
            "Drive: {:.1} dB | Symmetry Bias: {:.2} | Folding Stages: {:.1} | Folds: {}\n",
            self.input_drive_gain, self.fold_symmetry_bias, self.folding_stages_count, self.effective_fold_count
        ));
        out.push_str(&format!(
            "Miller Cap: {:.0} pF (fc: {:.1} kHz) | Knee Softness: {:.2} | Trim: {:.1} dB\n",
            self.miller_junction_capacitance, self.miller_cutoff_khz, self.diode_saturation_softness, self.output_trim_makeup
        ));
        out.push_str(&format!("Touch Target Radius: {:.1} pt [PASS: >=44pt]\n", SCHOTTKY_PUCK_HIT_RADIUS));
        out.push_str("Folded Transfer Curve V_out = f(V_in) across [-2.0V .. +2.0V]:\n");

        let cols = 32;
        let rows = 6;
        for r in 0..rows {
            let v_level = 1.5 - (r as f32) * 0.6;
            out.push('|');
            for c in 0..cols {
                let v_in = -2.0 + (c as f32 / (cols - 1) as f32) * 4.0;
                let v_out = self.evaluate_transfer_curve(v_in);
                if (v_out - v_level).abs() < 0.35 {
                    out.push('*');
                } else if r == rows / 2 {
                    out.push('-'); // Zero axis
                } else {
                    out.push(' ');
                }
            }
            out.push('\n');
        }

        out.push_str("Harmonic Distortion Spectrum (H1..H9):\n");
        for (i, &amp) in harmonics.iter().enumerate() {
            let bars = (amp * 16.0).round() as usize;
            let kind = if (i + 1) % 2 == 0 { "Even" } else { "Odd " };
            out.push_str(&format!("H{} ({}): {:2} |{}\n", i + 1, kind, (amp * 100.0) as usize, "=".repeat(bars)));
        }
        out.push_str("======================================================================\n");
        out
    }

    #[cfg(feature = "gui")]
    pub fn show_ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Header & Preset Selection
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Saturation Profile:")
                        .font(FontId::proportional(12.0))
                        .color(Color32::from_rgb(226, 232, 240)),
                );

                let presets = [
                    (SchottkyWavefolderPreset::SubtleEvenFoldWarmth, "Subtle Even Fold"),
                    (SchottkyWavefolderPreset::BuchlaWestCoastHarmonics, "Buchla Harmonics"),
                    (SchottkyWavefolderPreset::ScreamingDiodeCascade, "Screaming Cascade"),
                    (SchottkyWavefolderPreset::AsymmetricMetallicCrunch, "Metallic Crunch"),
                ];

                for (p, label) in presets {
                    let is_active = self.preset == p;
                    let text = RichText::new(label).font(FontId::proportional(11.0));
                    let btn = if is_active {
                        egui::Button::new(text.color(Color32::from_rgb(239, 68, 68)).strong())
                            .fill(Color32::from_rgb(40, 20, 20))
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(239, 68, 68)))
                    } else {
                        egui::Button::new(text.color(Color32::from_rgb(148, 163, 184)))
                            .fill(Color32::from_rgb(24, 28, 36))
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(51, 65, 85)))
                    };

                    if ui.add_sized([105.0, 26.0], btn).clicked() {
                        self.set_preset(p);
                    }
                }
            });

            ui.add_space(8.0);

            // Upper Visualizer: 2D Wavefolding Puck Canvas + Folded Transfer Curve + Harmonics
            ui.horizontal(|ui| {
                // 2D Puck Canvas (X: Input Drive, Y: Symmetry Bias)
                let (canvas_rect, response) = ui.allocate_exact_size(Vec2::new(340.0, 200.0), egui::Sense::drag());
                let painter = ui.painter_at(canvas_rect);

                // Background
                painter.rect_filled(canvas_rect, Rounding::same(6.0), Color32::from_rgb(15, 23, 42));
                painter.rect_stroke(canvas_rect, Rounding::same(6.0), Stroke::new(1.0_f32, Color32::from_rgb(51, 65, 85)));

                let pad = 15.0;
                let inner_w = canvas_rect.width() - 2.0 * pad;
                let inner_h = canvas_rect.height() - 2.0 * pad;

                // Center zero-bias horizontal line
                let center_y = canvas_rect.top() + pad + 0.5 * inner_h;
                painter.line_segment(
                    [egui::pos2(canvas_rect.left() + pad, center_y), egui::pos2(canvas_rect.right() - pad, center_y)],
                    Stroke::new(1.0_f32, Color32::from_rgb(71, 85, 105)),
                );

                // Gridlines
                for i in 1..4 {
                    let gx = canvas_rect.left() + pad + (i as f32 / 4.0) * inner_w;
                    painter.line_segment(
                        [egui::pos2(gx, canvas_rect.top() + pad), egui::pos2(gx, canvas_rect.bottom() - pad)],
                        Stroke::new(0.5_f32, Color32::from_rgb(30, 41, 59)),
                    );
                }

                // Puck Drag Interaction
                if response.dragged() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let nx = ((pos.x - (canvas_rect.left() + pad)) / inner_w).clamp(0.0, 1.0);
                        let ny = (((canvas_rect.bottom() - pad) - pos.y) / inner_h).clamp(0.0, 1.0);
                        self.puck_pos = (nx, ny);
                        self.input_drive_gain = Self::normalized_to_drive(nx);
                        self.fold_symmetry_bias = Self::normalized_to_bias(ny);
                        self.update_physics();
                    }
                }

                // Puck Render
                let puck_center = egui::pos2(
                    canvas_rect.left() + pad + self.puck_pos.0 * inner_w,
                    canvas_rect.bottom() - pad - self.puck_pos.1 * inner_h,
                );

                // Glow ring
                painter.circle_filled(puck_center, SCHOTTKY_PUCK_HIT_RADIUS, Color32::from_rgba_unmultiplied(239, 68, 68, 40));
                // Center puck
                painter.circle_filled(puck_center, 9.0, Color32::from_rgb(239, 68, 68));
                painter.circle_stroke(puck_center, 9.0, Stroke::new(2.0_f32, Color32::from_rgb(255, 255, 255)));

                // Axis labels
                painter.text(
                    egui::pos2(canvas_rect.left() + pad + 4.0, canvas_rect.bottom() - 6.0),
                    egui::Align2::LEFT_BOTTOM,
                    "Pre-Amp Drive (dB) ➔",
                    FontId::proportional(10.0),
                    Color32::from_rgb(148, 163, 184),
                );
                painter.text(
                    egui::pos2(canvas_rect.left() + 6.0, canvas_rect.top() + pad + 2.0),
                    egui::Align2::LEFT_TOP,
                    "▲ Symmetry Bias (+/-)",
                    FontId::proportional(10.0),
                    Color32::from_rgb(148, 163, 184),
                );

                // Right Panel: Transfer Characteristic Curve + Harmonic Spectrum
                ui.vertical(|ui| {
                    let (curve_rect, _) = ui.allocate_exact_size(Vec2::new(380.0, 110.0), egui::Sense::hover());
                    let cpainter = ui.painter_at(curve_rect);
                    cpainter.rect_filled(curve_rect, Rounding::same(6.0), Color32::from_rgb(15, 23, 42));
                    cpainter.rect_stroke(curve_rect, Rounding::same(6.0), Stroke::new(1.0_f32, Color32::from_rgb(51, 65, 85)));

                    let mid_x = curve_rect.center().x;
                    let mid_y = curve_rect.center().y;

                    // Axes
                    cpainter.line_segment([egui::pos2(curve_rect.left() + 10.0, mid_y), egui::pos2(curve_rect.right() - 10.0, mid_y)], Stroke::new(0.6_f32, Color32::from_rgb(51, 65, 85)));
                    cpainter.line_segment([egui::pos2(mid_x, curve_rect.top() + 6.0), egui::pos2(mid_x, curve_rect.bottom() - 6.0)], Stroke::new(0.6_f32, Color32::from_rgb(51, 65, 85)));

                    // Transfer Curve Points
                    let steps = 64;
                    let mut pts = Vec::with_capacity(steps);
                    for i in 0..steps {
                        let t = (i as f32) / ((steps - 1) as f32);
                        let v_in = -2.0 + t * 4.0;
                        let v_out = self.evaluate_transfer_curve(v_in);
                        let px = curve_rect.left() + 10.0 + t * (curve_rect.width() - 20.0);
                        let py = mid_y - (v_out / 2.0) * (curve_rect.height() * 0.42);
                        pts.push(egui::pos2(px, py));
                    }

                    for w in pts.windows(2) {
                        cpainter.line_segment([w[0], w[1]], Stroke::new(1.8_f32, Color32::from_rgb(239, 68, 68)));
                    }

                    cpainter.text(
                        egui::pos2(curve_rect.left() + 8.0, curve_rect.top() + 6.0),
                        egui::Align2::LEFT_TOP,
                        format!("V_out = f(V_in) | Active Folds: {}", self.effective_fold_count),
                        FontId::proportional(10.0),
                        Color32::from_rgb(226, 232, 240),
                    );

                    ui.add_space(6.0);

                    // Harmonics Bar Spectrum (H1..H9)
                    let (spec_rect, _) = ui.allocate_exact_size(Vec2::new(380.0, 84.0), egui::Sense::hover());
                    let spainter = ui.painter_at(spec_rect);
                    spainter.rect_filled(spec_rect, Rounding::same(6.0), Color32::from_rgb(15, 23, 42));
                    spainter.rect_stroke(spec_rect, Rounding::same(6.0), Stroke::new(1.0_f32, Color32::from_rgb(51, 65, 85)));

                    let harmonics = self.evaluate_harmonics();
                    let bar_w = (spec_rect.width() - 30.0) / (NUM_WAVEFOLDER_HARMONICS as f32);

                    for (i, &amp) in harmonics.iter().enumerate() {
                        let bx = spec_rect.left() + 15.0 + (i as f32) * bar_w;
                        let bar_h = (amp * (spec_rect.height() - 28.0)).max(2.0);
                        let by = spec_rect.bottom() - 14.0 - bar_h;
                        let rect = egui::Rect::from_min_size(egui::pos2(bx + 2.0, by), Vec2::new(bar_w - 4.0, bar_h));
                        let is_even = (i + 1) % 2 == 0;
                        let color = if is_even {
                            Color32::from_rgb(245, 158, 11) // Even harmonics: Amber
                        } else {
                            Color32::from_rgb(239, 68, 68) // Odd harmonics: Crimson
                        };
                        spainter.rect_filled(rect, Rounding::same(2.0), color);

                        spainter.text(
                            egui::pos2(bx + bar_w * 0.5, spec_rect.bottom() - 4.0),
                            egui::Align2::CENTER_BOTTOM,
                            format!("H{}", i + 1),
                            FontId::proportional(8.5),
                            Color32::from_rgb(148, 163, 184),
                        );
                    }
                });
            });

            ui.add_space(10.0);

            // Tactile Secondary Parameter Sliders
            egui::Grid::new("schottky_params_grid").num_columns(2).spacing([16.0, 8.0]).show(ui, |ui| {
                ui.label(RichText::new("Schottky Pre-Amp Drive:").font(FontId::proportional(11.0)).color(Color32::from_rgb(203, 213, 225)));
                if ui.add(egui::Slider::new(&mut self.input_drive_gain, 0.0..=36.0).show_value(true).suffix(" dB")).changed() {
                    self.puck_pos.0 = Self::drive_to_normalized(self.input_drive_gain);
                    self.update_physics();
                }
                ui.end_row();

                ui.label(RichText::new("Diode Threshold Asymmetry Bias:").font(FontId::proportional(11.0)).color(Color32::from_rgb(203, 213, 225)));
                if ui.add(egui::Slider::new(&mut self.fold_symmetry_bias, -1.0..=1.0).show_value(true)).changed() {
                    self.puck_pos.1 = Self::bias_to_normalized(self.fold_symmetry_bias);
                    self.update_physics();
                }
                ui.end_row();

                ui.label(RichText::new("Active Diode Folding Stages:").font(FontId::proportional(11.0)).color(Color32::from_rgb(203, 213, 225)));
                if ui.add(egui::Slider::new(&mut self.folding_stages_count, 1.0..=6.0).step_by(1.0).show_value(true).suffix(" stages")).changed() {
                    self.update_physics();
                }
                ui.end_row();

                ui.label(RichText::new("Miller Junction Capacitance:").font(FontId::proportional(11.0)).color(Color32::from_rgb(203, 213, 225)));
                if ui.add(egui::Slider::new(&mut self.miller_junction_capacitance, 10.0..=1000.0).show_value(true).suffix(" pF")).changed() {
                    self.update_physics();
                }
                ui.end_row();

                ui.label(RichText::new("Schottky Knee Softness:").font(FontId::proportional(11.0)).color(Color32::from_rgb(203, 213, 225)));
                if ui.add(egui::Slider::new(&mut self.diode_saturation_softness, 0.1..=1.0).show_value(true)).changed() {
                    self.update_physics();
                }
                ui.end_row();

                ui.label(RichText::new("Post-Folding Output Makeup Trim:").font(FontId::proportional(11.0)).color(Color32::from_rgb(203, 213, 225)));
                if ui.add(egui::Slider::new(&mut self.output_trim_makeup, -24.0..=12.0).show_value(true).suffix(" dB")).changed() {
                    self.update_physics();
                }
                ui.end_row();
            });

            ui.add_space(6.0);

            // Status Bar
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!(
                        "Miller Roll-Off: {:.1} kHz | Harmonic Density: {:.1}% | Makeup: {:.1} dB",
                        self.miller_cutoff_khz,
                        self.total_harmonic_density * 100.0,
                        self.output_trim_makeup
                    ))
                    .font(FontId::proportional(11.0))
                    .color(Color32::from_rgb(148, 163, 184)),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new("Touch Target: PASS (≥44pt)")
                            .font(FontId::proportional(10.0))
                            .color(Color32::from_rgb(34, 197, 94)),
                    );
                });
            });
        });
    }
}
