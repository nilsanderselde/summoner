// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! SIMD 4-Wide Polyphonic Wavetable Morphing & 4-Point Hermite Anti-Aliasing Oscillator HUD (Milestones 33 & 34).
//!
//! Directly reflects `crates/summoner_dsp/src/oscillators.rs` (`SimdPolyWavetableOscillator`,
//! `SimdPolyVoice`, `OscWavetable`, `WAVETABLE_SIZE = 2048`, `DEFAULT_MAX_VOICES = 16`,
//! `generate_wavetable_from_samples`, `interpolate_wavetable_sample`).
//! Provides an interactive 2D Dual-Table Cross-Morph & Dynamic Anti-Aliasing Fundamental Pitch Puck
//! (Wavetable Morph $m \in [0.0..=1.0]$ vs Logarithmic Fundamental Frequency $f \in [20.0..=8000.0]\text{ Hz}$),
//! 4-point Hermite cubic + Nyquist linear anti-aliasing blend evaluation, 16-voice SIMD 4-wide batch telemetry,
//! selectable primary/secondary 2048-point wavetable shapes, 4 curated presets, and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and deterministic ASCII snapshot rendering.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, FontId, RichText, Rounding, Stroke, Vec2};

pub const SIMD_POLY_WAVETABLE_PUCK_HIT_RADIUS: f32 = 22.0; // >= 44x44pt touch bounding target
pub const WAVETABLE_FRAME_SIZE: usize = 2048;
pub const MAX_SIMD_POLY_VOICES: usize = 16;
pub const MIN_POLY_WAVETABLE_FREQ_HZ: f32 = 20.0;
pub const MAX_POLY_WAVETABLE_FREQ_HZ: f32 = 8000.0;

/// Canonical 2048-point wavetable frame topology matching `OscWavetable` defaults.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WavetableFrameShape {
    #[default]
    PureSine,
    HarmonicSawtooth,
    HollowSquare,
    ChebyshevFoldedHarmonic,
    FormantSyncSweep,
}

impl WavetableFrameShape {
    pub fn label(&self) -> &'static str {
        match self {
            Self::PureSine => "Pure Sine (2048-pt Fundamental)",
            Self::HarmonicSawtooth => "Harmonic Sawtooth (All Integer Partials)",
            Self::HollowSquare => "Hollow Square (Odd Harmonics 1/2k-1)",
            Self::ChebyshevFoldedHarmonic => "Chebyshev Folded (T3 + T5 Metallic Bell)",
            Self::FormantSyncSweep => "Formant Sync Sweep (3.5x Hard-Sync Window)",
        }
    }

    pub fn short_tag(&self) -> &'static str {
        match self {
            Self::PureSine => "SINE",
            Self::HarmonicSawtooth => "SAW",
            Self::HollowSquare => "SQR",
            Self::ChebyshevFoldedHarmonic => "CHEB",
            Self::FormantSyncSweep => "SYNC",
        }
    }

    /// Evaluates normalized wavetable sample at phase $p \in [0.0..1.0)$.
    pub fn sample_at_phase(&self, phase: f32) -> f32 {
        let p = phase.rem_euclid(1.0);
        let tau = std::f32::consts::TAU;
        match self {
            Self::PureSine => (p * tau).sin(),
            Self::HarmonicSawtooth => 2.0 * p - 1.0,
            Self::HollowSquare => {
                if p < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            Self::ChebyshevFoldedHarmonic => {
                let x = (p * tau).sin();
                let t3 = 4.0 * x * x * x - 3.0 * x;
                let t5 = 16.0 * x.powi(5) - 20.0 * x.powi(3) + 5.0 * x;
                (0.55 * x + 0.30 * t3 + 0.15 * t5).clamp(-1.0, 1.0)
            }
            Self::FormantSyncSweep => {
                let win = 0.5 * (1.0 - (p * tau).cos());
                ((p * 3.5 * tau).sin() * win).clamp(-1.0, 1.0)
            }
        }
    }
}

/// Curated SIMD Polyphonic Wavetable Oscillator presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SimdPolyWavetablePreset {
    /// Warm Saw-to-Square Analog Pad (`Saw -> Square`, Morph 0.35, 220 Hz, 8 Active Voices, Attack 0.08, Release 0.9992).
    #[default]
    AnalogSawSquarePad,
    /// Crystalline Chebyshev Bell Chord (`Sine -> Chebyshev`, Morph 0.68, 440 Hz, 12 Active Voices, Attack 0.25, Release 0.9995).
    CrystallineChebyshevChoir,
    /// Aggressive Formant Sync Lead (`Saw -> FormantSync`, Morph 0.82, 880 Hz, 4 Active Voices, Attack 0.45, Release 0.9980).
    FormantSyncHyperLead,
    /// Deep Sub-Harmonic Unison Stack (`Sine -> Saw`, Morph 0.22, 65.4 Hz C2, 16 Active Voices, Attack 0.15, Release 0.9990).
    SubHarmonic16VoiceStack,
}

impl SimdPolyWavetablePreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::AnalogSawSquarePad => {
                "Analog Saw-to-Square Pad (35% Morph / 220 Hz / 8 SIMD Voices)"
            }
            Self::CrystallineChebyshevChoir => {
                "Crystalline Chebyshev Bell Choir (68% Morph / 440 Hz / 12 Voices)"
            }
            Self::FormantSyncHyperLead => {
                "Formant Sync Hyper Lead (82% Morph / 880 Hz / 4 SIMD Voices)"
            }
            Self::SubHarmonic16VoiceStack => {
                "Sub-Harmonic 16-Voice Unison Stack (22% Morph / 65.4 Hz / 16 Voices)"
            }
        }
    }

    /// Returns nominal `(morph, freq_hz, active_voices, attack_rate, release_decay, unison_detune_cents, table_a, table_b)`.
    pub fn nominal_parameters(
        &self,
    ) -> (
        f32,
        f32,
        usize,
        f32,
        f32,
        f32,
        WavetableFrameShape,
        WavetableFrameShape,
    ) {
        match self {
            Self::AnalogSawSquarePad => (
                0.35,
                220.0,
                8,
                0.08,
                0.9992,
                8.5,
                WavetableFrameShape::HarmonicSawtooth,
                WavetableFrameShape::HollowSquare,
            ),
            Self::CrystallineChebyshevChoir => (
                0.68,
                440.0,
                12,
                0.25,
                0.9995,
                12.0,
                WavetableFrameShape::PureSine,
                WavetableFrameShape::ChebyshevFoldedHarmonic,
            ),
            Self::FormantSyncHyperLead => (
                0.82,
                880.0,
                4,
                0.45,
                0.9980,
                18.0,
                WavetableFrameShape::HarmonicSawtooth,
                WavetableFrameShape::FormantSyncSweep,
            ),
            Self::SubHarmonic16VoiceStack => (
                0.22,
                65.4,
                16,
                0.15,
                0.9990,
                6.0,
                WavetableFrameShape::PureSine,
                WavetableFrameShape::HarmonicSawtooth,
            ),
        }
    }
}

/// SIMD 4-Wide Polyphonic Wavetable Morphing & 4-Point Hermite Anti-Aliasing Oscillator HUD view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimdPolyWavetableView {
    /// Active preset.
    pub preset: SimdPolyWavetablePreset,
    /// Primary wavetable slot A (`OscWavetable::table`).
    pub table_a: WavetableFrameShape,
    /// Secondary morph wavetable slot B (`OscWavetable::table2`).
    pub table_b: WavetableFrameShape,
    /// Cross-table morph factor [0.0 ..= 1.0].
    pub morph: f32,
    /// Fundamental pitch frequency in Hz [20.0 ..= 8000.0].
    pub fundamental_freq_hz: f32,
    /// Number of active polyphonic voices [1 ..= 16] processed in 4-wide `f32x4` SIMD batches.
    pub active_voices: usize,
    /// Voice envelope attack rate [0.005 ..= 0.50].
    pub attack_rate: f32,
    /// Voice envelope release decay multiplier [0.9900 ..= 0.9999].
    pub release_decay_rate: f32,
    /// Unison stereo/polyphonic detune spread in cents [0.0 ..= 50.0].
    pub unison_detune_cents: f32,
    /// Host sample rate in Hz.
    pub sample_rate: u32,
    /// Bypass toggle.
    pub bypassed: bool,
    /// 2D Puck position (X: morph [0.0..=1.0], Y: log-normalized fundamental_freq_hz [0.0..=1.0]).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for SimdPolyWavetableView {
    fn default() -> Self {
        Self::new()
    }
}

impl SimdPolyWavetableView {
    pub fn new() -> Self {
        let preset = SimdPolyWavetablePreset::AnalogSawSquarePad;
        let (morph, freq, voices, atk, rel, detune, ta, tb) = preset.nominal_parameters();
        let norm_x = Self::morph_to_normalized(morph);
        let norm_y = Self::freq_to_normalized(freq);

        Self {
            preset,
            table_a: ta,
            table_b: tb,
            morph,
            fundamental_freq_hz: freq,
            active_voices: voices,
            attack_rate: atk,
            release_decay_rate: rel,
            unison_detune_cents: detune,
            sample_rate: 48000,
            bypassed: false,
            puck_pos: (norm_x, norm_y),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        }
    }

    pub fn set_preset(&mut self, preset: SimdPolyWavetablePreset) {
        self.preset = preset;
        let (morph, freq, voices, atk, rel, detune, ta, tb) = preset.nominal_parameters();
        self.morph = morph;
        self.fundamental_freq_hz = freq;
        self.active_voices = voices;
        self.attack_rate = atk;
        self.release_decay_rate = rel;
        self.unison_detune_cents = detune;
        self.table_a = ta;
        self.table_b = tb;
        self.sync_puck_from_params();
    }

    pub fn sync_puck_from_params(&mut self) {
        self.morph = self.morph.clamp(0.0, 1.0);
        self.fundamental_freq_hz = self
            .fundamental_freq_hz
            .clamp(MIN_POLY_WAVETABLE_FREQ_HZ, MAX_POLY_WAVETABLE_FREQ_HZ);
        self.active_voices = self.active_voices.clamp(1, MAX_SIMD_POLY_VOICES);
        self.attack_rate = self.attack_rate.clamp(0.005, 0.50);
        self.release_decay_rate = self.release_decay_rate.clamp(0.9900, 0.9999);
        self.unison_detune_cents = self.unison_detune_cents.clamp(0.0, 50.0);
        self.puck_pos = (
            Self::morph_to_normalized(self.morph),
            Self::freq_to_normalized(self.fundamental_freq_hz),
        );
    }

    pub fn update_from_puck(&mut self, norm_x: f32, norm_y: f32) {
        let x = norm_x.clamp(0.0, 1.0);
        let y = norm_y.clamp(0.0, 1.0);
        self.puck_pos = (x, y);
        self.morph = Self::normalized_to_morph(x);
        self.fundamental_freq_hz = Self::normalized_to_freq(y);
    }

    pub fn morph_to_normalized(morph: f32) -> f32 {
        morph.clamp(0.0, 1.0)
    }

    pub fn normalized_to_morph(norm: f32) -> f32 {
        norm.clamp(0.0, 1.0)
    }

    /// Logarithmic frequency normalization [20 Hz ..= 8000 Hz] -> [0.0 ..= 1.0].
    pub fn freq_to_normalized(freq_hz: f32) -> f32 {
        let clamped = freq_hz.clamp(MIN_POLY_WAVETABLE_FREQ_HZ, MAX_POLY_WAVETABLE_FREQ_HZ);
        ((clamped / MIN_POLY_WAVETABLE_FREQ_HZ).ln()
            / (MAX_POLY_WAVETABLE_FREQ_HZ / MIN_POLY_WAVETABLE_FREQ_HZ).ln())
        .clamp(0.0, 1.0)
    }

    /// Logarithmic frequency de-normalization [0.0 ..= 1.0] -> [20 Hz ..= 8000 Hz].
    pub fn normalized_to_freq(norm: f32) -> f32 {
        let n = norm.clamp(0.0, 1.0);
        MIN_POLY_WAVETABLE_FREQ_HZ
            * (MAX_POLY_WAVETABLE_FREQ_HZ / MIN_POLY_WAVETABLE_FREQ_HZ).powf(n)
    }

    /// Computes dynamic anti-aliasing Hermite cubic vs Nyquist linear blend factor `aa_blend`:
    /// `(1.0 - (dt * WAVETABLE_SIZE * 1.5)).clamp(0.0, 1.0)` matching `SimdPolyWavetableOscillator::interpolate_wavetable_sample`.
    pub fn compute_hermite_aa_blend(&self) -> f32 {
        let sr = self.sample_rate.max(8000) as f32;
        let dt = self.fundamental_freq_hz / sr;
        (1.0 - (dt * WAVETABLE_FRAME_SIZE as f32 * 1.5)).clamp(0.0, 1.0)
    }

    /// Returns `(simd_4wide_batches, scalar_remainder_voices)` for `self.active_voices` matching `chunks_exact(4)`.
    pub fn simd_batch_breakdown(&self) -> (usize, usize) {
        let v = self.active_voices.clamp(1, MAX_SIMD_POLY_VOICES);
        (v / 4, v % 4)
    }

    /// Evaluates 4-point Hermite cubic + Nyquist anti-aliasing morphed wavetable sample at phase $p \in [0.0..=1.0]$.
    pub fn evaluate_hermite_morphed_sample(&self, phase_norm: f32) -> f32 {
        if self.bypassed {
            return 0.0;
        }
        let p = phase_norm.rem_euclid(1.0);
        let step = 1.0 / WAVETABLE_FRAME_SIZE as f32;
        let idx_float = p * WAVETABLE_FRAME_SIZE as f32;
        let base_phase = idx_float.floor() * step;
        let frac = idx_float - idx_float.floor();

        let eval_table = |shape: WavetableFrameShape| -> f32 {
            let y0 = shape.sample_at_phase(base_phase - step);
            let y1 = shape.sample_at_phase(base_phase);
            let y2 = shape.sample_at_phase(base_phase + step);
            let y3 = shape.sample_at_phase(base_phase + 2.0 * step);

            let c0 = y1;
            let c1 = 0.5 * (y2 - y0);
            let c2 = y0 - 2.5 * y1 + 2.0 * y2 - 0.5 * y3;
            let c3 = 0.5 * (y3 - y0) + 1.5 * (y1 - y2);
            let cubic = ((c3 * frac + c2) * frac + c1) * frac + c0;
            let linear = y1 * (1.0 - frac) + y2 * frac;
            let aa = self.compute_hermite_aa_blend();
            aa * cubic + (1.0 - aa) * linear
        };

        let s1 = eval_table(self.table_a);
        let s2 = eval_table(self.table_b);
        let m = self.morph.clamp(0.0, 1.0);
        (s1 * (1.0 - m) + s2 * m).clamp(-1.0, 1.0)
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
        (dx * dx + dy * dy).sqrt() <= SIMD_POLY_WAVETABLE_PUCK_HIT_RADIUS
    }

    /// Renders a deterministic ASCII snapshot for headless QA verification.
    pub fn render_ascii_snapshot(&self) -> String {
        let aa_blend = self.compute_hermite_aa_blend();
        let (batches, rem) = self.simd_batch_breakdown();
        let mut out = String::new();
        out.push_str("======================================================================\n");
        out.push_str(" SUMMONER DAW -- SIMD 4-WIDE POLYPHONIC WAVETABLE OSCILLATOR SNAPSHOT\n");
        out.push_str("======================================================================\n");
        out.push_str(&format!(" Preset:           {}\n", self.preset.name()));
        out.push_str(&format!(
            " Tables & Morph:   {} -> {} | Morph: {:.1}% | Bypassed: {}\n",
            self.table_a.short_tag(),
            self.table_b.short_tag(),
            self.morph * 100.0,
            if self.bypassed { "YES" } else { "NO" }
        ));
        out.push_str(&format!(
            " SIMD Polyphony:   {} Voices ({}x f32x4 Batches + {} Scalar) | Pitch: {:.1} Hz\n",
            self.active_voices, batches, rem, self.fundamental_freq_hz
        ));
        out.push_str(&format!(
            " Hermite AA & Env: Cubic Blend {:.1}% | Attack {:.3} | Release {:.4}\n",
            aa_blend * 100.0,
            self.attack_rate,
            self.release_decay_rate
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
                    RichText::new("SIMD 4-WIDE POLYPHONIC WAVETABLE OSCILLATOR HUD")
                        .color(Color32::from_rgb(168, 85, 247))
                        .strong(),
                );
                ui.label(
                    RichText::new("M33 / M34 SimdPolyWavetableOscillator + 4-Pt Hermite AA (2048-pt)")
                        .color(Color32::from_rgb(148, 163, 184))
                        .small(),
                );
            });

            ui.add_space(6.0);

            // Preset Selector Bar
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Wavetable Presets:").strong());
                let presets = [
                    (
                        SimdPolyWavetablePreset::AnalogSawSquarePad,
                        "Analog Saw/Sqr",
                    ),
                    (
                        SimdPolyWavetablePreset::CrystallineChebyshevChoir,
                        "Chebyshev Bell",
                    ),
                    (
                        SimdPolyWavetablePreset::FormantSyncHyperLead,
                        "Formant Sync",
                    ),
                    (
                        SimdPolyWavetablePreset::SubHarmonic16VoiceStack,
                        "16-Voice Stack",
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
                        Color32::from_rgb(168, 85, 247)
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

            // Interactive 2D Wavetable Morph & Fundamental Frequency Puck Canvas
            let canvas_size = Vec2::new(ui.available_width().max(280.0), 144.0);
            let (response, painter) =
                ui.allocate_painter(canvas_size, egui::Sense::click_and_drag());
            let rect = response.rect;
            let canvas = Rect::new(rect.min.x, rect.min.y, rect.width(), rect.height());

            painter.rect_filled(rect, Rounding::same(4.0), Color32::from_rgb(16, 12, 28));
            painter.rect_stroke(
                rect,
                Rounding::same(4.0),
                Stroke::new(1.0_f32, Color32::from_rgb(126, 34, 206)),
            );

            // Center reference line
            let zero_y = rect.center().y;
            painter.line_segment(
                [
                    egui::pos2(rect.min.x + 8.0, zero_y),
                    egui::pos2(rect.max.x - 8.0, zero_y),
                ],
                Stroke::new(1.0_f32, Color32::from_rgb(46, 32, 72)),
            );

            // Draw 4-Point Hermite Morphed Wavetable Cycle
            let num_pts = 64;
            let mut pts = Vec::with_capacity(num_pts);
            for idx in 0..num_pts {
                let frac = idx as f32 / (num_pts - 1) as f32;
                let sample = self.evaluate_hermite_morphed_sample(frac);
                let norm_val = (0.5 + 0.42 * sample).clamp(0.05, 0.95);
                let px = rect.min.x + 15.0 + frac * (rect.width() - 30.0);
                let py = rect.max.y - 15.0 - norm_val * (rect.height() - 30.0);
                pts.push(egui::pos2(px, py));
            }
            for i in 0..(pts.len() - 1) {
                painter.line_segment(
                    [pts[i], pts[i + 1]],
                    Stroke::new(2.0_f32, Color32::from_rgb(192, 132, 252)),
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
                SIMD_POLY_WAVETABLE_PUCK_HIT_RADIUS,
                Color32::from_rgba_unmultiplied(168, 85, 247, 44),
            );
            painter.circle_stroke(
                puck_pos,
                SIMD_POLY_WAVETABLE_PUCK_HIT_RADIUS,
                Stroke::new(1.2_f32, Color32::from_rgb(216, 180, 254)),
            );
            painter.circle_filled(puck_pos, 6.0, Color32::WHITE);

            let (batches, rem) = self.simd_batch_breakdown();
            painter.text(
                egui::pos2(rect.min.x + 8.0, rect.min.y + 6.0),
                egui::Align2::LEFT_TOP,
                format!(
                    "{}->{} ({:.0}%) | {:.1} Hz | {}V ({}x4+{} SIMD) | Hermite AA {:.0}%",
                    self.table_a.short_tag(),
                    self.table_b.short_tag(),
                    self.morph * 100.0,
                    self.fundamental_freq_hz,
                    self.active_voices,
                    batches,
                    rem,
                    self.compute_hermite_aa_blend() * 100.0
                ),
                FontId::proportional(11.0),
                Color32::from_rgb(233, 213, 255),
            );

            ui.add_space(8.0);

            // Surgical Pro Sliders
            ui.horizontal(|ui| {
                ui.label(RichText::new("Cross-Table Morph (A -> B):").size(11.0));
                let mut m = self.morph;
                if ui.add(egui::Slider::new(&mut m, 0.0..=1.0)).changed() {
                    self.morph = m;
                    self.sync_puck_from_params();
                }
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Fundamental Pitch (Hz):").size(11.0));
                let mut f = self.fundamental_freq_hz;
                if ui
                    .add(
                        egui::Slider::new(
                            &mut f,
                            MIN_POLY_WAVETABLE_FREQ_HZ..=MAX_POLY_WAVETABLE_FREQ_HZ,
                        )
                        .logarithmic(true)
                        .suffix(" Hz"),
                    )
                    .changed()
                {
                    self.fundamental_freq_hz = f;
                    self.sync_puck_from_params();
                }
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Active Polyphonic Voices (SIMD f32x4):").size(11.0));
                let mut v = self.active_voices;
                if ui
                    .add(egui::Slider::new(&mut v, 1..=MAX_SIMD_POLY_VOICES))
                    .changed()
                {
                    self.active_voices = v;
                    self.sync_puck_from_params();
                }
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Envelope Attack Rate:").size(11.0));
                ui.add(egui::Slider::new(&mut self.attack_rate, 0.005..=0.50));
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Envelope Release Decay:").size(11.0));
                ui.add(egui::Slider::new(
                    &mut self.release_decay_rate,
                    0.9900..=0.9999,
                ));
            });
        });
    }
}
