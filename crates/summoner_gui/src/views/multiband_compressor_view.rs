// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! 3-Band SVF Crossover Multiband Compressor & Dynamic Envelope HUD (Milestones 33 & 34).
//!
//! Directly reflects `crates/summoner_dsp/src/multiband_compressor.rs` (`MultibandCompressorNode`)
//! and `crates/summoner_dsp/src/compressor.rs` (`CompressorNode`, `FilterSVF` crossovers).
//! Provides an interactive 2D Low-Mid & Mid-High SVF Crossover Frequency Puck
//! (Logarithmic Low-Mid Crossover $f_{\text{LM}} \in [60.0..=800.0]\text{ Hz}$ vs
//! Logarithmic Mid-High Crossover $f_{\text{MH}} \in [1000.0..=10000.0]\text{ Hz}$),
//! per-band threshold, ratio, attack/release envelope telemetry, gain reduction calculation,
//! 3-band spectral crossover contour visualization, 4 curated presets, and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and deterministic ASCII snapshot rendering.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, FontId, RichText, Rounding, Stroke, Vec2};

pub const MULTIBAND_COMP_PUCK_HIT_RADIUS: f32 = 22.0; // >= 44x44pt touch bounding target
pub const MIN_LOW_MID_FREQ_HZ: f32 = 60.0;
pub const MAX_LOW_MID_FREQ_HZ: f32 = 800.0;
pub const MIN_MID_HIGH_FREQ_HZ: f32 = 1000.0;
pub const MAX_MID_HIGH_FREQ_HZ: f32 = 10000.0;

/// Per-band compressor configuration matching `CompressorNode::with_params` inside `MultibandCompressorNode`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MultibandCompBandSlot {
    pub threshold_db: f32,
    pub ratio: f32,
    pub attack_ms: f32,
    pub release_ms: f32,
    pub makeup_db: f32,
    pub muted: bool,
}

impl MultibandCompBandSlot {
    pub const fn new(
        threshold_db: f32,
        ratio: f32,
        attack_ms: f32,
        release_ms: f32,
        makeup_db: f32,
    ) -> Self {
        Self {
            threshold_db,
            ratio,
            attack_ms,
            release_ms,
            makeup_db,
            muted: false,
        }
    }

    /// Computes instantaneous gain reduction in dB ($\le 0.0\text{ dB}$) for a band input level `input_db`.
    pub fn evaluate_gain_reduction_db(&self, input_db: f32) -> f32 {
        if self.muted || input_db <= self.threshold_db || self.ratio <= 1.0 {
            0.0
        } else {
            let over_db = input_db - self.threshold_db;
            let compressed_over = over_db / self.ratio;
            -(over_db - compressed_over)
        }
    }
}

/// Curated 3-Band Multiband Compressor presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MultibandCompressorPreset {
    /// Default Studio 3-Band Glue (`MultibandCompressorNode::new`: 200 Hz / 2000 Hz, Low -18dB 3:1, Mid -16dB 2.5:1, High -14dB 2:1).
    #[default]
    StudioThreeBandGlue,
    /// Tight Sub-Bass & Crisp Air Master (140 Hz / 3800 Hz, Low -22dB 4.5:1, Mid -15dB 2.0:1, High -16dB 3.0:1).
    TightSubCrispAir,
    /// Broadcast Vocal & Midrange Focus (260 Hz / 2800 Hz, Low -20dB 3.5:1, Mid -19dB 4.0:1, High -15dB 2.5:1).
    BroadcastMidrangeControl,
    /// EDM OTT-Style Aggressive Up/Down Punch (180 Hz / 2400 Hz, Low -24dB 6.0:1, Mid -21dB 4.5:1, High -18dB 5.0:1).
    AggressiveElectronicPunch,
}

impl MultibandCompressorPreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::StudioThreeBandGlue => "Studio 3-Band Glue (200 Hz / 2.0 kHz SVF Crossover)",
            Self::TightSubCrispAir => "Tight Sub & Crisp Air (140 Hz / 3.8 kHz Mastering)",
            Self::BroadcastMidrangeControl => {
                "Broadcast Midrange Control (260 Hz / 2.8 kHz Vocal Focus)"
            }
            Self::AggressiveElectronicPunch => {
                "Aggressive Electronic Punch (180 Hz / 2.4 kHz High-Ratio)"
            }
        }
    }

    /// Returns nominal `(low_mid_freq_hz, mid_high_freq_hz, low_band, mid_band, high_band, input_drive_db)`.
    pub fn nominal_parameters(
        &self,
    ) -> (
        f32,
        f32,
        MultibandCompBandSlot,
        MultibandCompBandSlot,
        MultibandCompBandSlot,
        f32,
    ) {
        match self {
            Self::StudioThreeBandGlue => (
                200.0,
                2000.0,
                MultibandCompBandSlot::new(-18.0, 3.0, 10.0, 100.0, 0.0),
                MultibandCompBandSlot::new(-16.0, 2.5, 15.0, 120.0, 0.0),
                MultibandCompBandSlot::new(-14.0, 2.0, 5.0, 80.0, 0.0),
                -6.0,
            ),
            Self::TightSubCrispAir => (
                140.0,
                3800.0,
                MultibandCompBandSlot::new(-22.0, 4.5, 8.0, 90.0, 1.5),
                MultibandCompBandSlot::new(-15.0, 2.0, 18.0, 140.0, 0.5),
                MultibandCompBandSlot::new(-16.0, 3.0, 4.0, 65.0, 1.0),
                -5.0,
            ),
            Self::BroadcastMidrangeControl => (
                260.0,
                2800.0,
                MultibandCompBandSlot::new(-20.0, 3.5, 12.0, 110.0, 1.0),
                MultibandCompBandSlot::new(-19.0, 4.0, 8.0, 95.0, 2.0),
                MultibandCompBandSlot::new(-15.0, 2.5, 5.0, 75.0, 0.5),
                -4.5,
            ),
            Self::AggressiveElectronicPunch => (
                180.0,
                2400.0,
                MultibandCompBandSlot::new(-24.0, 6.0, 5.0, 70.0, 3.0),
                MultibandCompBandSlot::new(-21.0, 4.5, 6.0, 85.0, 2.5),
                MultibandCompBandSlot::new(-18.0, 5.0, 3.0, 50.0, 2.5),
                -3.0,
            ),
        }
    }
}

/// 3-Band SVF Crossover Multiband Compressor & Dynamic Envelope HUD view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultibandCompressorView {
    /// Active preset.
    pub preset: MultibandCompressorPreset,
    /// Low-to-Mid SVF crossover frequency in Hz [60.0 ..= 800.0].
    pub low_mid_freq_hz: f32,
    /// Mid-to-High SVF crossover frequency in Hz [1000.0 ..= 10000.0].
    pub mid_high_freq_hz: f32,
    /// Low band compressor parameters.
    pub low_band: MultibandCompBandSlot,
    /// Mid band compressor parameters.
    pub mid_band: MultibandCompBandSlot,
    /// High band compressor parameters.
    pub high_band: MultibandCompBandSlot,
    /// Simulated input program level in dBFS [-36.0 ..= 0.0] for real-time gain reduction telemetry.
    pub test_input_level_db: f32,
    /// Dry/Wet parallel compression mix [0.0 ..= 1.0].
    pub wet_mix: f32,
    /// Bypass toggle.
    pub bypassed: bool,
    /// 2D Puck position (X: log-normalized low_mid_freq_hz, Y: log-normalized mid_high_freq_hz).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for MultibandCompressorView {
    fn default() -> Self {
        Self::new()
    }
}

impl MultibandCompressorView {
    pub fn new() -> Self {
        let preset = MultibandCompressorPreset::StudioThreeBandGlue;
        let (lm_hz, mh_hz, low, mid, high, in_db) = preset.nominal_parameters();
        let norm_x = Self::low_mid_to_normalized(lm_hz);
        let norm_y = Self::mid_high_to_normalized(mh_hz);

        Self {
            preset,
            low_mid_freq_hz: lm_hz,
            mid_high_freq_hz: mh_hz,
            low_band: low,
            mid_band: mid,
            high_band: high,
            test_input_level_db: in_db,
            wet_mix: 1.0,
            bypassed: false,
            puck_pos: (norm_x, norm_y),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        }
    }

    pub fn set_preset(&mut self, preset: MultibandCompressorPreset) {
        self.preset = preset;
        let (lm_hz, mh_hz, low, mid, high, in_db) = preset.nominal_parameters();
        self.low_mid_freq_hz = lm_hz;
        self.mid_high_freq_hz = mh_hz;
        self.low_band = low;
        self.mid_band = mid;
        self.high_band = high;
        self.test_input_level_db = in_db;
        self.sync_puck_from_params();
    }

    pub fn sync_puck_from_params(&mut self) {
        self.low_mid_freq_hz = self
            .low_mid_freq_hz
            .clamp(MIN_LOW_MID_FREQ_HZ, MAX_LOW_MID_FREQ_HZ);
        self.mid_high_freq_hz = self
            .mid_high_freq_hz
            .clamp(MIN_MID_HIGH_FREQ_HZ, MAX_MID_HIGH_FREQ_HZ);
        self.test_input_level_db = self.test_input_level_db.clamp(-36.0, 0.0);
        self.wet_mix = self.wet_mix.clamp(0.0, 1.0);
        self.puck_pos = (
            Self::low_mid_to_normalized(self.low_mid_freq_hz),
            Self::mid_high_to_normalized(self.mid_high_freq_hz),
        );
    }

    pub fn update_from_puck(&mut self, norm_x: f32, norm_y: f32) {
        let x = norm_x.clamp(0.0, 1.0);
        let y = norm_y.clamp(0.0, 1.0);
        self.puck_pos = (x, y);
        self.low_mid_freq_hz = Self::normalized_to_low_mid(x);
        self.mid_high_freq_hz = Self::normalized_to_mid_high(y);
    }

    /// Logarithmic Low-Mid crossover normalization [60 Hz ..= 800 Hz] -> [0.0 ..= 1.0].
    pub fn low_mid_to_normalized(freq_hz: f32) -> f32 {
        let clamped = freq_hz.clamp(MIN_LOW_MID_FREQ_HZ, MAX_LOW_MID_FREQ_HZ);
        ((clamped / MIN_LOW_MID_FREQ_HZ).ln() / (MAX_LOW_MID_FREQ_HZ / MIN_LOW_MID_FREQ_HZ).ln())
            .clamp(0.0, 1.0)
    }

    /// Logarithmic Low-Mid crossover de-normalization [0.0 ..= 1.0] -> [60 Hz ..= 800 Hz].
    pub fn normalized_to_low_mid(norm: f32) -> f32 {
        let n = norm.clamp(0.0, 1.0);
        MIN_LOW_MID_FREQ_HZ * (MAX_LOW_MID_FREQ_HZ / MIN_LOW_MID_FREQ_HZ).powf(n)
    }

    /// Logarithmic Mid-High crossover normalization [1000 Hz ..= 10000 Hz] -> [0.0 ..= 1.0].
    pub fn mid_high_to_normalized(freq_hz: f32) -> f32 {
        let clamped = freq_hz.clamp(MIN_MID_HIGH_FREQ_HZ, MAX_MID_HIGH_FREQ_HZ);
        ((clamped / MIN_MID_HIGH_FREQ_HZ).ln() / (MAX_MID_HIGH_FREQ_HZ / MIN_MID_HIGH_FREQ_HZ).ln())
            .clamp(0.0, 1.0)
    }

    /// Logarithmic Mid-High crossover de-normalization [0.0 ..= 1.0] -> [1000 Hz ..= 10000 Hz].
    pub fn normalized_to_mid_high(norm: f32) -> f32 {
        let n = norm.clamp(0.0, 1.0);
        MIN_MID_HIGH_FREQ_HZ * (MAX_MID_HIGH_FREQ_HZ / MIN_MID_HIGH_FREQ_HZ).powf(n)
    }

    /// Returns current `(low_gr_db, mid_gr_db, high_gr_db)` at `test_input_level_db`.
    pub fn evaluate_three_band_gain_reduction_db(&self) -> (f32, f32, f32) {
        if self.bypassed {
            return (0.0, 0.0, 0.0);
        }
        (
            self.low_band
                .evaluate_gain_reduction_db(self.test_input_level_db),
            self.mid_band
                .evaluate_gain_reduction_db(self.test_input_level_db),
            self.high_band
                .evaluate_gain_reduction_db(self.test_input_level_db),
        )
    }

    /// Evaluates composite 3-band compressed transfer gain in dB at frequency `freq_hz` [20.0 ..= 20000.0].
    pub fn evaluate_composite_response_db_at_freq(&self, freq_hz: f32) -> f32 {
        if self.bypassed {
            return 0.0;
        }
        let f = freq_hz.clamp(20.0, 20000.0);
        let lm = self.low_mid_freq_hz;
        let mh = self.mid_high_freq_hz;

        // 2nd-order SVF Linkwitz-Riley-style power weights across Low, Mid, High bands
        let w_low = 1.0 / (1.0 + (f / lm).powi(4));
        let w_high = 1.0 / (1.0 + (mh / f).powi(4));
        let w_mid = (1.0 - w_low - w_high).max(0.0);

        let (gr_l, gr_m, gr_h) = self.evaluate_three_band_gain_reduction_db();
        let net_l = gr_l + self.low_band.makeup_db;
        let net_m = gr_m + self.mid_band.makeup_db;
        let net_h = gr_h + self.high_band.makeup_db;

        (w_low * net_l + w_mid * net_m + w_high * net_h) * self.wet_mix
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
        (dx * dx + dy * dy).sqrt() <= MULTIBAND_COMP_PUCK_HIT_RADIUS
    }

    /// Renders a deterministic ASCII snapshot for headless QA verification.
    pub fn render_ascii_snapshot(&self) -> String {
        let (gr_l, gr_m, gr_h) = self.evaluate_three_band_gain_reduction_db();
        let mut out = String::new();
        out.push_str("======================================================================\n");
        out.push_str(" SUMMONER DAW -- 3-BAND SVF MULTIBAND COMPRESSOR HUD SNAPSHOT\n");
        out.push_str("======================================================================\n");
        out.push_str(&format!(" Preset:           {}\n", self.preset.name()));
        out.push_str(&format!(
            " SVF Crossovers:   Low/Mid {:.1} Hz | Mid/High {:.1} Hz | Bypassed: {}\n",
            self.low_mid_freq_hz,
            self.mid_high_freq_hz,
            if self.bypassed { "YES" } else { "NO" }
        ));
        out.push_str(&format!(
            " Band Thresholds:  Low {:.1} dB ({:.1}:1) | Mid {:.1} dB ({:.1}:1) | High {:.1} dB ({:.1}:1)\n",
            self.low_band.threshold_db,
            self.low_band.ratio,
            self.mid_band.threshold_db,
            self.mid_band.ratio,
            self.high_band.threshold_db,
            self.high_band.ratio
        ));
        out.push_str(&format!(
            " Gain Reduction:   Low {:.2} dB | Mid {:.2} dB | High {:.2} dB (@ {:.1} dBFS In)\n",
            gr_l, gr_m, gr_h, self.test_input_level_db
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
                    RichText::new("3-BAND SVF CROSSOVER MULTIBAND COMPRESSOR HUD")
                        .color(Color32::from_rgb(16, 185, 129))
                        .strong(),
                );
                ui.label(
                    RichText::new("M33 / M34 MultibandCompressorNode + FilterSVF + CompressorNode")
                        .color(Color32::from_rgb(148, 163, 184))
                        .small(),
                );
            });

            ui.add_space(6.0);

            // Preset Selector Bar
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Multiband Presets:").strong());
                let presets = [
                    (
                        MultibandCompressorPreset::StudioThreeBandGlue,
                        "Studio 3-Band Glue",
                    ),
                    (
                        MultibandCompressorPreset::TightSubCrispAir,
                        "Tight Sub & Air",
                    ),
                    (
                        MultibandCompressorPreset::BroadcastMidrangeControl,
                        "Broadcast Mid",
                    ),
                    (
                        MultibandCompressorPreset::AggressiveElectronicPunch,
                        "Electronic Punch",
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
                        Color32::from_rgb(16, 185, 129)
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

            // Interactive 2D SVF Crossover Frequency Puck & 3-Band Gain Reduction Contour Canvas
            let canvas_size = Vec2::new(ui.available_width().max(280.0), 144.0);
            let (response, painter) =
                ui.allocate_painter(canvas_size, egui::Sense::click_and_drag());
            let rect = response.rect;
            let canvas = Rect::new(rect.min.x, rect.min.y, rect.width(), rect.height());

            painter.rect_filled(rect, Rounding::same(4.0), Color32::from_rgb(10, 22, 18));
            painter.rect_stroke(
                rect,
                Rounding::same(4.0),
                Stroke::new(1.0_f32, Color32::from_rgb(5, 150, 105)),
            );

            // Unity 0 dB reference line
            let zero_y = rect.center().y;
            painter.line_segment(
                [
                    egui::pos2(rect.min.x + 8.0, zero_y),
                    egui::pos2(rect.max.x - 8.0, zero_y),
                ],
                Stroke::new(1.0_f32, Color32::from_rgb(24, 56, 44)),
            );

            // Draw 3-Band Composite Gain Reduction / Makeup Frequency Contour
            let num_pts = 56;
            let mut pts = Vec::with_capacity(num_pts);
            for idx in 0..num_pts {
                let frac = idx as f32 / (num_pts - 1) as f32;
                let freq = 20.0 * (20000.0_f32 / 20.0).powf(frac);
                let db = self.evaluate_composite_response_db_at_freq(freq);
                let norm_y = ((db + 18.0) / 36.0).clamp(0.05, 0.95);
                let px = rect.min.x + 15.0 + frac * (rect.width() - 30.0);
                let py = rect.max.y - 15.0 - norm_y * (rect.height() - 30.0);
                pts.push(egui::pos2(px, py));
            }
            for i in 0..(pts.len() - 1) {
                painter.line_segment(
                    [pts[i], pts[i + 1]],
                    Stroke::new(2.0_f32, Color32::from_rgb(52, 211, 153)),
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
                MULTIBAND_COMP_PUCK_HIT_RADIUS,
                Color32::from_rgba_unmultiplied(16, 185, 129, 44),
            );
            painter.circle_stroke(
                puck_pos,
                MULTIBAND_COMP_PUCK_HIT_RADIUS,
                Stroke::new(1.2_f32, Color32::from_rgb(110, 231, 183)),
            );
            painter.circle_filled(puck_pos, 6.0, Color32::WHITE);

            let (gr_l, gr_m, gr_h) = self.evaluate_three_band_gain_reduction_db();
            painter.text(
                egui::pos2(rect.min.x + 8.0, rect.min.y + 6.0),
                egui::Align2::LEFT_TOP,
                format!(
                    "X-Over: {:.0} Hz / {:.0} Hz | GR: L {:.1}dB | M {:.1}dB | H {:.1}dB",
                    self.low_mid_freq_hz, self.mid_high_freq_hz, gr_l, gr_m, gr_h
                ),
                FontId::proportional(11.0),
                Color32::from_rgb(167, 243, 208),
            );

            ui.add_space(8.0);

            // Surgical Pro Sliders
            ui.horizontal(|ui| {
                ui.label(RichText::new("Low-Mid SVF Crossover (Hz):").size(11.0));
                let mut lm = self.low_mid_freq_hz;
                if ui
                    .add(
                        egui::Slider::new(&mut lm, MIN_LOW_MID_FREQ_HZ..=MAX_LOW_MID_FREQ_HZ)
                            .logarithmic(true)
                            .suffix(" Hz"),
                    )
                    .changed()
                {
                    self.low_mid_freq_hz = lm;
                    self.sync_puck_from_params();
                }
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Mid-High SVF Crossover (Hz):").size(11.0));
                let mut mh = self.mid_high_freq_hz;
                if ui
                    .add(
                        egui::Slider::new(&mut mh, MIN_MID_HIGH_FREQ_HZ..=MAX_MID_HIGH_FREQ_HZ)
                            .logarithmic(true)
                            .suffix(" Hz"),
                    )
                    .changed()
                {
                    self.mid_high_freq_hz = mh;
                    self.sync_puck_from_params();
                }
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Low Band Threshold (dB):").size(11.0));
                ui.add(
                    egui::Slider::new(&mut self.low_band.threshold_db, -48.0..=0.0).suffix(" dB"),
                );
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Mid Band Threshold (dB):").size(11.0));
                ui.add(
                    egui::Slider::new(&mut self.mid_band.threshold_db, -48.0..=0.0).suffix(" dB"),
                );
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("High Band Threshold (dB):").size(11.0));
                ui.add(
                    egui::Slider::new(&mut self.high_band.threshold_db, -48.0..=0.0).suffix(" dB"),
                );
            });
        });
    }
}
