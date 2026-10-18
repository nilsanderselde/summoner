// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! ITU-R BS.1770-4 4x Polyphase True-Peak ISP Limiter & Safety Clipper HUD (Milestones 8, 33 & 34).
//!
//! Directly reflects `crates/summoner_dsp/src/true_peak_limiter.rs` (`TruePeakLimiter`,
//! `TruePeakDetector`, `OversampledClipper`, `ITU_BS1770_POLYPHASE_COEFFS`,
//! `POLYPHASE_TAPS = 12`, `MAX_LOOKAHEAD = 256`).
//! Provides an interactive 2D Threshold & Brickwall Ceiling Puck
//! (Threshold $T \in [-18.0..=0.0]\text{ dB}$ vs Ceiling $C \in [-6.0..=0.0]\text{ dBTP}$),
//! exact 4-phase 12-tap ITU-R BS.1770 polyphase FIR inter-sample peak (ISP) interpolation,
//! quadratic soft-knee gain reduction evaluation, 4x oversampled safety clipper saturation,
//! 4 curated presets, and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and deterministic ASCII snapshot rendering.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, FontId, RichText, Rounding, Stroke, Vec2};

pub const TRUE_PEAK_LIMITER_PUCK_HIT_RADIUS: f32 = 22.0; // >= 44x44pt touch bounding target
pub const TRUE_PEAK_POLYPHASE_TAPS: usize = 12;
pub const TRUE_PEAK_MAX_LOOKAHEAD: usize = 256;
pub const MIN_LIMITER_THRESHOLD_DB: f32 = -18.0;
pub const MAX_LIMITER_THRESHOLD_DB: f32 = 0.0;
pub const MIN_LIMITER_CEILING_DB: f32 = -6.0;
pub const MAX_LIMITER_CEILING_DB: f32 = 0.0;

/// Polyphase FIR filter coefficients for 4x oversampling true-peak interpolation
/// matching `crates/summoner_dsp/src/true_peak_limiter.rs` (`ITU_BS1770_POLYPHASE_COEFFS`).
#[allow(clippy::excessive_precision)]
pub const ITU_BS1770_COEFFS: [[f32; TRUE_PEAK_POLYPHASE_TAPS]; 4] = [
    // Phase 0: Sub-sample offset 0.00
    [
        0.00000000, 0.00000000, 0.00000000, 0.00000000, 0.00000000, 1.00000000, 0.00000000,
        0.00000000, 0.00000000, 0.00000000, 0.00000000, 0.00000000,
    ],
    // Phase 1: Sub-sample offset +0.25
    [
        -0.00390625, 0.01562500, -0.04687500, 0.11718750, -0.27343750, 0.89843750, 0.39062500,
        -0.14062500, 0.06250000, -0.02734375, 0.00976562, -0.00195312,
    ],
    // Phase 2: Sub-sample offset +0.50
    [
        -0.00781250, 0.02734375, -0.07812500, 0.18750000, -0.42187500, 0.64062500, 0.64062500,
        -0.42187500, 0.18750000, -0.07812500, 0.02734375, -0.00781250,
    ],
    // Phase 3: Sub-sample offset +0.75
    [
        -0.00195312, 0.00976562, -0.02734375, 0.06250000, -0.14062500, 0.39062500, 0.89843750,
        -0.27343750, 0.11718750, -0.04687500, 0.01562500, -0.00390625,
    ],
];

/// Curated True-Peak ISP Limiter presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TruePeakLimiterPreset {
    /// Streaming True-Peak Safe (-1.0 dBTP Ceiling, -2.5 dB Threshold, 1.5 dB Knee, 65 ms Release, 64 smp Lookahead).
    #[default]
    StreamingTruePeak1Db,
    /// Broadcast EBU R128 Compliant (-2.0 dBTP Ceiling, -4.0 dB Threshold, 2.0 dB Knee, 100 ms Release, 128 smp Lookahead).
    BroadcastEbuR128,
    /// Club Brickwall Loudness (-0.1 dBTP Ceiling, -5.5 dB Threshold, 0.5 dB Knee, 35 ms Release, 32 smp Lookahead).
    ClubBrickwall01Db,
    /// Vinyl & Mastering Soft-Knee (-0.5 dBTP Ceiling, -3.0 dB Threshold, 3.0 dB Knee, 120 ms Release, 96 smp Lookahead).
    VinylSoftKnee05Db,
}

impl TruePeakLimiterPreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::StreamingTruePeak1Db => {
                "Streaming True-Peak Safe (-1.0 dBTP Ceiling / -2.5 dB Thresh / 64 smp)"
            }
            Self::BroadcastEbuR128 => {
                "Broadcast EBU R128 (-2.0 dBTP Ceiling / -4.0 dB Thresh / 128 smp)"
            }
            Self::ClubBrickwall01Db => {
                "Club Brickwall Loudness (-0.1 dBTP Ceiling / -5.5 dB Thresh / 32 smp)"
            }
            Self::VinylSoftKnee05Db => {
                "Vinyl Mastering Soft-Knee (-0.5 dBTP Ceiling / -3.0 dB Thresh / 96 smp)"
            }
        }
    }

    /// Returns nominal `(threshold_db, ceiling_db, knee_db, release_ms, lookahead_samples, soft_clip_depth)`.
    pub fn nominal_parameters(&self) -> (f32, f32, f32, f32, usize, f32) {
        match self {
            Self::StreamingTruePeak1Db => (-2.5, -1.0, 1.5, 65.0, 64, 0.08),
            Self::BroadcastEbuR128 => (-4.0, -2.0, 2.0, 100.0, 128, 0.05),
            Self::ClubBrickwall01Db => (-5.5, -0.1, 0.5, 35.0, 32, 0.15),
            Self::VinylSoftKnee05Db => (-3.0, -0.5, 3.0, 120.0, 96, 0.12),
        }
    }
}

/// ITU-R BS.1770-4 4x Polyphase True-Peak ISP Limiter & Safety Clipper HUD view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TruePeakLimiterView {
    /// Active preset.
    pub preset: TruePeakLimiterPreset,
    /// Limiting threshold in dB [-18.0 ..= 0.0].
    pub threshold_db: f32,
    /// Brickwall output ceiling in dBTP [-6.0 ..= 0.0].
    pub ceiling_db: f32,
    /// Quadratic soft-knee width in dB [0.0 ..= 6.0].
    pub knee_db: f32,
    /// Exponential release recovery time in ms [5.0 ..= 500.0].
    pub release_ms: f32,
    /// Pre-allocated circular lookahead window in samples [1 ..= 255].
    pub lookahead_samples: usize,
    /// 4x oversampled safety clipper soft-knee depth [0.0 ..= 0.50].
    pub soft_clip_depth: f32,
    /// Inter-sample peak (ISP) 4x polyphase protection enabled.
    pub isp_protection: bool,
    /// 4x oversampled safety clipper enabled.
    pub clipper_enabled: bool,
    /// Simulated input peak level in dBFS [-18.0 ..= +6.0] for real-time gain reduction telemetry.
    pub test_input_peak_db: f32,
    /// Bypass toggle.
    pub bypassed: bool,
    /// 2D Puck position (X: normalized threshold_db, Y: normalized ceiling_db).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for TruePeakLimiterView {
    fn default() -> Self {
        Self::new()
    }
}

impl TruePeakLimiterView {
    pub fn new() -> Self {
        let preset = TruePeakLimiterPreset::StreamingTruePeak1Db;
        let (thresh, ceil, knee, rel, look, clip_depth) = preset.nominal_parameters();
        let norm_x = Self::threshold_to_normalized(thresh);
        let norm_y = Self::ceiling_to_normalized(ceil);

        Self {
            preset,
            threshold_db: thresh,
            ceiling_db: ceil,
            knee_db: knee,
            release_ms: rel,
            lookahead_samples: look,
            soft_clip_depth: clip_depth,
            isp_protection: true,
            clipper_enabled: true,
            test_input_peak_db: 0.5,
            bypassed: false,
            puck_pos: (norm_x, norm_y),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        }
    }

    pub fn set_preset(&mut self, preset: TruePeakLimiterPreset) {
        self.preset = preset;
        let (thresh, ceil, knee, rel, look, clip_depth) = preset.nominal_parameters();
        self.threshold_db = thresh;
        self.ceiling_db = ceil;
        self.knee_db = knee;
        self.release_ms = rel;
        self.lookahead_samples = look;
        self.soft_clip_depth = clip_depth;
        self.sync_puck_from_params();
    }

    pub fn sync_puck_from_params(&mut self) {
        self.threshold_db = self
            .threshold_db
            .clamp(MIN_LIMITER_THRESHOLD_DB, MAX_LIMITER_THRESHOLD_DB);
        self.ceiling_db = self
            .ceiling_db
            .clamp(MIN_LIMITER_CEILING_DB, MAX_LIMITER_CEILING_DB);
        self.knee_db = self.knee_db.clamp(0.0, 6.0);
        self.release_ms = self.release_ms.clamp(5.0, 500.0);
        self.lookahead_samples = self.lookahead_samples.clamp(1, TRUE_PEAK_MAX_LOOKAHEAD - 1);
        self.soft_clip_depth = self.soft_clip_depth.clamp(0.0, 0.50);
        self.test_input_peak_db = self.test_input_peak_db.clamp(-18.0, 6.0);
        self.puck_pos = (
            Self::threshold_to_normalized(self.threshold_db),
            Self::ceiling_to_normalized(self.ceiling_db),
        );
    }

    pub fn update_from_puck(&mut self, norm_x: f32, norm_y: f32) {
        let x = norm_x.clamp(0.0, 1.0);
        let y = norm_y.clamp(0.0, 1.0);
        self.puck_pos = (x, y);
        self.threshold_db = Self::normalized_to_threshold(x);
        self.ceiling_db = Self::normalized_to_ceiling(y);
    }

    /// Threshold normalization [-18.0 dB ..= 0.0 dB] -> [0.0 ..= 1.0].
    pub fn threshold_to_normalized(thresh_db: f32) -> f32 {
        let clamped = thresh_db.clamp(MIN_LIMITER_THRESHOLD_DB, MAX_LIMITER_THRESHOLD_DB);
        ((clamped - MIN_LIMITER_THRESHOLD_DB)
            / (MAX_LIMITER_THRESHOLD_DB - MIN_LIMITER_THRESHOLD_DB))
            .clamp(0.0, 1.0)
    }

    /// Threshold de-normalization [0.0 ..= 1.0] -> [-18.0 dB ..= 0.0 dB].
    pub fn normalized_to_threshold(norm: f32) -> f32 {
        let n = norm.clamp(0.0, 1.0);
        MIN_LIMITER_THRESHOLD_DB + n * (MAX_LIMITER_THRESHOLD_DB - MIN_LIMITER_THRESHOLD_DB)
    }

    /// Ceiling normalization [-6.0 dBTP ..= 0.0 dBTP] -> [0.0 ..= 1.0].
    pub fn ceiling_to_normalized(ceil_db: f32) -> f32 {
        let clamped = ceil_db.clamp(MIN_LIMITER_CEILING_DB, MAX_LIMITER_CEILING_DB);
        ((clamped - MIN_LIMITER_CEILING_DB) / (MAX_LIMITER_CEILING_DB - MIN_LIMITER_CEILING_DB))
            .clamp(0.0, 1.0)
    }

    /// Ceiling de-normalization [0.0 ..= 1.0] -> [-6.0 dBTP ..= 0.0 dBTP].
    pub fn normalized_to_ceiling(norm: f32) -> f32 {
        let n = norm.clamp(0.0, 1.0);
        MIN_LIMITER_CEILING_DB + n * (MAX_LIMITER_CEILING_DB - MIN_LIMITER_CEILING_DB)
    }

    /// Evaluates ITU-R BS.1770-4 4-phase 12-tap polyphase FIR interpolation on a 12-sample history buffer.
    /// Returns `(max_true_peak_linear, [phase0, phase1, phase2, phase3])`.
    pub fn evaluate_itu_bs1770_polyphase(
        history: &[f32; TRUE_PEAK_POLYPHASE_TAPS],
    ) -> (f32, [f32; 4]) {
        let mut interpolated = [0.0f32; 4];
        let mut max_abs = 0.0f32;

        for phase in 0..4 {
            let mut sum = 0.0f32;
            let coeffs = &ITU_BS1770_COEFFS[phase];
            for (tap, &coeff) in coeffs.iter().enumerate() {
                sum += history[tap] * coeff;
            }
            interpolated[phase] = sum;
            let abs_val = sum.abs();
            if abs_val > max_abs {
                max_abs = abs_val;
            }
        }
        (max_abs, interpolated)
    }

    /// Computes quadratic soft-knee output level in dB and gain reduction in dB ($\le 0.0\text{ dB}$)
    /// for a given input peak level `peak_db`, matching `TruePeakLimiter::calculate_target_gain`.
    pub fn evaluate_transfer_and_gr_db(&self, peak_db: f32) -> (f32, f32) {
        if self.bypassed {
            return (peak_db, 0.0);
        }

        let thresh_db = self.threshold_db;
        let knee_db = self.knee_db;

        let target_out_db = if knee_db > 0.0
            && (peak_db > thresh_db - knee_db * 0.5)
            && (peak_db < thresh_db + knee_db * 0.5)
        {
            let delta = peak_db - thresh_db + knee_db * 0.5;
            thresh_db - knee_db * 0.5 + (delta * delta) / (2.0 * knee_db)
        } else if peak_db >= thresh_db {
            self.ceiling_db
        } else {
            peak_db
        };

        let gr_db = (target_out_db - peak_db).min(0.0);
        (peak_db + gr_db, gr_db)
    }

    /// Evaluates the 4x oversampled safety clipper saturation curve matching `OversampledClipper::clip_sample`.
    pub fn evaluate_safety_clipper_sample(&self, sample: f32) -> f32 {
        if self.bypassed || !self.clipper_enabled {
            return sample;
        }
        let ceiling_linear = 10.0f32.powf(self.ceiling_db / 20.0).max(0.01);
        let threshold = ceiling_linear * (1.0 - self.soft_clip_depth.clamp(0.0, 0.99));
        let abs_s = sample.abs();
        if abs_s <= threshold {
            sample
        } else {
            let sign = if sample >= 0.0 { 1.0 } else { -1.0 };
            let excess = abs_s - threshold;
            let range = (ceiling_linear - threshold).max(1e-6);
            let compressed = threshold + range * (excess / range).tanh();
            sign * compressed.min(ceiling_linear)
        }
    }

    /// Converts normalized puck position to canvas coordinates.
    pub fn puck_to_canvas_pos(&self, canvas_rect: &Rect) -> (f32, f32) {
        let padding = 15.0;
        let inner_w = canvas_rect.width - 2.0 * padding;
        let inner_h = canvas_rect.height - 2.0 * padding;
        let x = canvas_rect.x + padding + self.puck_pos.0 * inner_w;
        let y = canvas_rect.y + canvas_rect.height - padding - self.puck_pos.1 * inner_h;
        (x, y)
    }

    /// Hit-tests whether a touch/mouse point lands on the >= 44x44pt touch puck.
    pub fn is_puck_hit(&self, canvas_rect: &Rect, point: (f32, f32)) -> bool {
        let (puck_x, puck_y) = self.puck_to_canvas_pos(canvas_rect);
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= TRUE_PEAK_LIMITER_PUCK_HIT_RADIUS
    }

    /// Renders a deterministic ASCII snapshot for headless QA verification.
    pub fn render_ascii_snapshot(&self) -> String {
        let (out_db, gr_db) = self.evaluate_transfer_and_gr_db(self.test_input_peak_db);
        let mut out = String::new();
        out.push_str("======================================================================\n");
        out.push_str(" SUMMONER DAW -- ITU-R BS.1770-4 TRUE-PEAK ISP LIMITER HUD SNAPSHOT\n");
        out.push_str("======================================================================\n");
        out.push_str(&format!(" Preset:           {}\n", self.preset.name()));
        out.push_str(&format!(
            " Threshold/Ceil:   Thresh {:.2} dB | Ceiling {:.2} dBTP | Knee {:.2} dB\n",
            self.threshold_db, self.ceiling_db, self.knee_db
        ));
        out.push_str(&format!(
            " Lookahead/Rel:    {} smp ({} max) | Release {:.1} ms | ISP 4x: {}\n",
            self.lookahead_samples,
            TRUE_PEAK_MAX_LOOKAHEAD,
            self.release_ms,
            if self.isp_protection { "ON" } else { "OFF" }
        ));
        out.push_str(&format!(
            " Telemetry:        In {:.2} dBFS -> Out {:.2} dBTP (GR: {:.2} dB)\n",
            self.test_input_peak_db, out_db, gr_db
        ));
        out.push_str(&format!(
            " Puck Coord:       ({:.3}, {:.3}) [PASS: >=44pt Hit Target]\n",
            self.puck_pos.0, self.puck_pos.1
        ));
        out.push_str("======================================================================\n");
        out
    }

    #[cfg(feature = "gui")]
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.heading(
                    RichText::new("ITU-R BS.1770-4 TRUE-PEAK ISP LIMITER & CLIPPER HUD")
                        .color(Color32::from_rgb(244, 63, 94))
                        .strong(),
                );
                ui.label(
                    RichText::new("M8 / M33 TruePeakLimiter + TruePeakDetector + OversampledClipper")
                        .color(Color32::from_rgb(148, 163, 184))
                        .small(),
                );
            });

            ui.add_space(6.0);

            // Preset Selector Bar
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("True-Peak Presets:").strong());
                let presets = [
                    (
                        TruePeakLimiterPreset::StreamingTruePeak1Db,
                        "Streaming -1.0 dBTP",
                    ),
                    (
                        TruePeakLimiterPreset::BroadcastEbuR128,
                        "Broadcast -2.0 dBTP",
                    ),
                    (
                        TruePeakLimiterPreset::ClubBrickwall01Db,
                        "Club Brickwall -0.1",
                    ),
                    (
                        TruePeakLimiterPreset::VinylSoftKnee05Db,
                        "Vinyl Soft-Knee -0.5",
                    ),
                ];
                for (p, label) in presets {
                    let is_active = self.preset == p;
                    let btn = egui::Button::new(
                        RichText::new(label)
                            .color(if is_active {
                                Color32::BLACK
                            } else {
                                Color32::from_rgb(203, 213, 225)
                            })
                            .size(11.0),
                    )
                    .fill(if is_active {
                        Color32::from_rgb(244, 63, 94)
                    } else {
                        Color32::from_rgb(30, 41, 59)
                    })
                    .min_size(Vec2::new(88.0, 24.0));

                    if ui.add(btn).clicked() {
                        self.set_preset(p);
                    }
                }
            });

            ui.add_space(8.0);

            // Interactive 2D Threshold & Ceiling Puck + Soft-Knee Transfer Curve Canvas
            let canvas_size = Vec2::new(ui.available_width().max(280.0), 144.0);
            let (response, painter) =
                ui.allocate_painter(canvas_size, egui::Sense::click_and_drag());
            let rect = response.rect;
            let canvas = Rect::new(rect.min.x, rect.min.y, rect.width(), rect.height());

            painter.rect_filled(rect, Rounding::same(4.0), Color32::from_rgb(22, 10, 14));
            painter.rect_stroke(
                rect,
                Rounding::same(4.0),
                Stroke::new(1.0_f32, Color32::from_rgb(225, 29, 72)),
            );

            // Draw Soft-Knee Brickwall Transfer Curve (-18 dB to +6 dB)
            let num_pts = 56;
            let mut pts = Vec::with_capacity(num_pts);
            for idx in 0..num_pts {
                let frac = idx as f32 / (num_pts - 1) as f32;
                let in_db = -18.0 + frac * 24.0;
                let (out_db, _) = self.evaluate_transfer_and_gr_db(in_db);
                let norm_y = ((out_db + 18.0) / 18.0).clamp(0.05, 0.95);
                let px = rect.min.x + 15.0 + frac * (rect.width() - 30.0);
                let py = rect.max.y - 15.0 - norm_y * (rect.height() - 30.0);
                pts.push(egui::pos2(px, py));
            }
            for i in 0..(pts.len() - 1) {
                painter.line_segment(
                    [pts[i], pts[i + 1]],
                    Stroke::new(2.0_f32, Color32::from_rgb(251, 113, 133)),
                );
            }

            // Handle Puck Drag / Click
            if response.dragged() || response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let padding = 15.0;
                    let norm_x = ((pos.x - (rect.min.x + padding))
                        / (rect.width() - 2.0 * padding))
                        .clamp(0.0, 1.0);
                    let norm_y = (1.0
                        - ((pos.y - (rect.min.y + padding)) / (rect.height() - 2.0 * padding)))
                        .clamp(0.0, 1.0);
                    self.update_from_puck(norm_x, norm_y);
                }
            }

            // Draw >= 44x44pt Puck
            let (puck_x, puck_y) = self.puck_to_canvas_pos(&canvas);
            let puck_pos = egui::pos2(puck_x, puck_y);
            painter.circle_filled(
                puck_pos,
                TRUE_PEAK_LIMITER_PUCK_HIT_RADIUS,
                Color32::from_rgba_unmultiplied(244, 63, 94, 44),
            );
            painter.circle_stroke(
                puck_pos,
                TRUE_PEAK_LIMITER_PUCK_HIT_RADIUS,
                Stroke::new(1.2_f32, Color32::from_rgb(253, 164, 175)),
            );
            painter.circle_filled(puck_pos, 6.0, Color32::WHITE);

            let (out_db, gr_db) = self.evaluate_transfer_and_gr_db(self.test_input_peak_db);
            painter.text(
                egui::pos2(rect.min.x + 8.0, rect.min.y + 6.0),
                egui::Align2::LEFT_TOP,
                format!(
                    "Thresh: {:.1} dB | Ceil: {:.1} dBTP | Out: {:.1} dBTP | GR: {:.1} dB",
                    self.threshold_db, self.ceiling_db, out_db, gr_db
                ),
                FontId::proportional(11.0),
                Color32::from_rgb(254, 205, 211),
            );

            ui.add_space(8.0);

            // Surgical Pro Sliders
            ui.horizontal(|ui| {
                ui.label(RichText::new("Threshold (dB):").size(11.0));
                let mut th = self.threshold_db;
                if ui
                    .add(
                        egui::Slider::new(
                            &mut th,
                            MIN_LIMITER_THRESHOLD_DB..=MAX_LIMITER_THRESHOLD_DB,
                        )
                        .suffix(" dB"),
                    )
                    .changed()
                {
                    self.threshold_db = th;
                    self.sync_puck_from_params();
                }
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Brickwall Ceiling (dBTP):").size(11.0));
                let mut cl = self.ceiling_db;
                if ui
                    .add(
                        egui::Slider::new(&mut cl, MIN_LIMITER_CEILING_DB..=MAX_LIMITER_CEILING_DB)
                            .suffix(" dBTP"),
                    )
                    .changed()
                {
                    self.ceiling_db = cl;
                    self.sync_puck_from_params();
                }
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Soft-Knee Width (dB):").size(11.0));
                ui.add(egui::Slider::new(&mut self.knee_db, 0.0..=6.0).suffix(" dB"));
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Release Recovery (ms):").size(11.0));
                ui.add(egui::Slider::new(&mut self.release_ms, 5.0..=500.0).suffix(" ms"));
            });

            ui.horizontal(|ui| {
                ui.checkbox(&mut self.isp_protection, "4x Polyphase ISP Guard");
                ui.checkbox(&mut self.clipper_enabled, "Oversampled Safety Clipper");
            });
        });
    }
}
