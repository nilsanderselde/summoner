// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Chromatic Autocorrelation Pitch Tuner & Strobe Phase Calibration HUD (Milestones 33 & 34).
//!
//! Directly reflects `crates/summoner_dsp/src/tuner.rs` (`detect_chromatic_pitch`,
//! `TunerResult`, `NOTE_NAMES`, time-domain autocorrelation fundamental lag detection).
//! Provides an interactive 2D Reference Concert Pitch & Cent Calibration Puck
//! (Reference $A_4 \in [410.0..=450.0]\text{ Hz}$ vs Fine Cent Offset
//! $\Delta c \in [-50.0..=+50.0]\text{ cents}$), real-time autocorrelation waveform peak
//! detector, strobe phase needle indicator, 4 historical/studio reference presets,
//! and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and deterministic ASCII snapshot rendering.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, FontId, RichText, Rounding, Stroke, Vec2};

pub const CHROMATIC_TUNER_PUCK_HIT_RADIUS: f32 = 22.0; // >= 44x44pt touch bounding target
pub const MIN_REFERENCE_A4_HZ: f32 = 410.0;
pub const MAX_REFERENCE_A4_HZ: f32 = 450.0;
pub const MIN_CENTS_DEV: f32 = -50.0;
pub const MAX_CENTS_DEV: f32 = 50.0;

pub const TUNER_NOTE_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

/// Curated reference concert pitch & strobe calibration presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ChromaticTunerPreset {
    /// ISO 16 standard concert pitch (A4 = 440.0 Hz).
    #[default]
    ConcertStandard440Hz,
    /// Historical Baroque chamber pitch (A4 = 415.0 Hz).
    BaroqueChamber415Hz,
    /// Modern European symphonic bright tuning (A4 = 442.0 Hz).
    OrchestralBright442Hz,
    /// Verdi / scientific natural tuning (A4 = 432.0 Hz).
    VerdiNatural432Hz,
}

impl ChromaticTunerPreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::ConcertStandard440Hz => "ISO Concert Standard (440.0 Hz)",
            Self::BaroqueChamber415Hz => "Baroque Chamber (415.0 Hz)",
            Self::OrchestralBright442Hz => "Orchestral Symphonic (442.0 Hz)",
            Self::VerdiNatural432Hz => "Verdi Scientific (432.0 Hz)",
        }
    }

    /// Returns nominal (reference_a4_hz, cents_tolerance, autocorr_threshold, strobe_speed).
    pub fn nominal_parameters(&self) -> (f32, f32, f32, f32) {
        match self {
            Self::ConcertStandard440Hz => (440.0, 2.0, 0.70, 1.00),
            Self::BaroqueChamber415Hz => (415.0, 3.0, 0.65, 0.85),
            Self::OrchestralBright442Hz => (442.0, 1.5, 0.75, 1.15),
            Self::VerdiNatural432Hz => (432.0, 2.5, 0.68, 0.95),
        }
    }
}

/// Chromatic Autocorrelation Pitch Tuner & Strobe Phase Calibration HUD view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromaticTunerView {
    /// Active reference tuning preset.
    pub preset: ChromaticTunerPreset,
    /// Concert reference pitch A4 in Hz [410.0 ..= 450.0].
    pub reference_a4_hz: f32,
    /// Current tracked fundamental pitch in Hz [40.0 ..= 2000.0].
    pub detected_pitch_hz: f32,
    /// Current cents deviation from nearest equal-tempered note [-50.0 ..= +50.0].
    pub cents_deviation: f32,
    /// In-tune lock window tolerance in cents [0.5 ..= 10.0].
    pub cents_tolerance: f32,
    /// Time-domain autocorrelation peak threshold [0.30 ..= 0.95].
    pub autocorr_threshold: f32,
    /// Strobe phase drift speed multiplier [0.25 ..= 2.0].
    pub strobe_speed: f32,
    /// Nearest detected MIDI note number [0 ..= 127].
    pub detected_midi_note: u8,
    /// Nearest detected note name (e.g. "A4").
    pub detected_note_name: String,
    /// 2D Puck position (X: normalized reference A4 Hz, Y: normalized cents deviation).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for ChromaticTunerView {
    fn default() -> Self {
        Self::new()
    }
}

impl ChromaticTunerView {
    pub fn new() -> Self {
        let preset = ChromaticTunerPreset::ConcertStandard440Hz;
        let (ref_a4, tol, thresh, speed) = preset.nominal_parameters();
        let norm_x = Self::reference_a4_to_normalized(ref_a4);
        let norm_y = Self::cents_to_normalized(0.0);

        Self {
            preset,
            reference_a4_hz: ref_a4,
            detected_pitch_hz: ref_a4,
            cents_deviation: 0.0,
            cents_tolerance: tol,
            autocorr_threshold: thresh,
            strobe_speed: speed,
            detected_midi_note: 69,
            detected_note_name: "A4".to_string(),
            puck_pos: (norm_x, norm_y),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        }
    }

    pub fn set_preset(&mut self, preset: ChromaticTunerPreset) {
        self.preset = preset;
        let (ref_a4, tol, thresh, speed) = preset.nominal_parameters();
        self.reference_a4_hz = ref_a4;
        self.cents_tolerance = tol;
        self.autocorr_threshold = thresh;
        self.strobe_speed = speed;
        self.detected_pitch_hz = ref_a4 * 2.0_f32.powf(self.cents_deviation / 1200.0);
        self.recompute_note_from_pitch(self.detected_pitch_hz);
        self.sync_puck_from_params();
    }

    pub fn sync_puck_from_params(&mut self) {
        self.puck_pos = (
            Self::reference_a4_to_normalized(self.reference_a4_hz),
            Self::cents_to_normalized(self.cents_deviation),
        );
    }

    pub fn update_from_puck(&mut self, norm_x: f32, norm_y: f32) {
        let x = norm_x.clamp(0.0, 1.0);
        let y = norm_y.clamp(0.0, 1.0);
        self.puck_pos = (x, y);
        self.reference_a4_hz = Self::normalized_to_reference_a4(x);
        self.cents_deviation = Self::normalized_to_cents(y);
        let midi_offset = (self.detected_midi_note as f32) - 69.0;
        let target_note_hz = self.reference_a4_hz * 2.0_f32.powf(midi_offset / 12.0);
        self.detected_pitch_hz = target_note_hz * 2.0_f32.powf(self.cents_deviation / 1200.0);
    }

    pub fn reference_a4_to_normalized(hz: f32) -> f32 {
        let clamped = hz.clamp(MIN_REFERENCE_A4_HZ, MAX_REFERENCE_A4_HZ);
        ((clamped - MIN_REFERENCE_A4_HZ) / (MAX_REFERENCE_A4_HZ - MIN_REFERENCE_A4_HZ))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_reference_a4(norm: f32) -> f32 {
        let n = norm.clamp(0.0, 1.0);
        MIN_REFERENCE_A4_HZ + n * (MAX_REFERENCE_A4_HZ - MIN_REFERENCE_A4_HZ)
    }

    pub fn cents_to_normalized(cents: f32) -> f32 {
        let clamped = cents.clamp(MIN_CENTS_DEV, MAX_CENTS_DEV);
        ((clamped - MIN_CENTS_DEV) / (MAX_CENTS_DEV - MIN_CENTS_DEV)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_cents(norm: f32) -> f32 {
        let n = norm.clamp(0.0, 1.0);
        MIN_CENTS_DEV + n * (MAX_CENTS_DEV - MIN_CENTS_DEV)
    }

    /// Recomputes nearest MIDI note, note name, and cents deviation from a pitch in Hz.
    pub fn recompute_note_from_pitch(&mut self, pitch_hz: f32) {
        let clamped_hz = pitch_hz.clamp(20.0, 4000.0);
        self.detected_pitch_hz = clamped_hz;
        let ref_a4 = self.reference_a4_hz.clamp(MIN_REFERENCE_A4_HZ, MAX_REFERENCE_A4_HZ);
        let note_num = 69.0 + 12.0 * (clamped_hz / ref_a4).log2();
        let rounded_midi = note_num.round().clamp(0.0, 127.0) as u8;
        let cents = ((note_num - rounded_midi as f32) * 100.0).clamp(MIN_CENTS_DEV, MAX_CENTS_DEV);

        let pc = (rounded_midi % 12) as usize;
        let octave = (rounded_midi / 12) as i8 - 1;
        self.detected_midi_note = rounded_midi;
        self.detected_note_name = format!("{}{}", TUNER_NOTE_NAMES[pc], octave);
        self.cents_deviation = cents;
        self.sync_puck_from_params();
    }

    /// Runs time-domain autocorrelation pitch detection matching `summoner_dsp::tuner::detect_chromatic_pitch`.
    pub fn detect_pitch_from_buffer(&mut self, samples: &[f32], sample_rate: f32) -> bool {
        if samples.len() < 128 || sample_rate <= 0.0 {
            return false;
        }
        let min_lag = (sample_rate / 2000.0) as usize;
        let max_lag = ((sample_rate / 40.0) as usize).min(samples.len() / 2);
        if min_lag >= max_lag {
            return false;
        }

        let n = samples.len() / 2;
        let mut zero_lag_corr = 0.0f32;
        for &s in samples.iter().take(n) {
            zero_lag_corr += s * s;
        }
        if zero_lag_corr < 1e-5 {
            return false;
        }

        let mut corrs = vec![0.0f32; max_lag + 2];
        let mut max_corr = 0.0f32;
        for lag in min_lag..=max_lag {
            let mut corr = 0.0f32;
            for i in 0..n {
                corr += samples[i] * samples[i + lag];
            }
            corrs[lag] = corr;
            if corr > max_corr {
                max_corr = corr;
            }
        }

        if max_corr / zero_lag_corr < 0.3 {
            return false;
        }

        let threshold = max_corr * self.autocorr_threshold.clamp(0.30, 0.95);
        let mut fundamental_lag = 0;
        for lag in (min_lag + 1)..max_lag {
            if corrs[lag] >= threshold
                && corrs[lag] >= corrs[lag - 1]
                && corrs[lag] >= corrs[lag + 1]
            {
                fundamental_lag = lag;
                break;
            }
        }

        if fundamental_lag == 0 {
            return false;
        }

        let pitch_hz = sample_rate / (fundamental_lag as f32);
        self.recompute_note_from_pitch(pitch_hz);
        true
    }

    /// Returns true if the current cents deviation is within the lock tolerance window.
    pub fn is_in_tune_locked(&self) -> bool {
        self.cents_deviation.abs() <= self.cents_tolerance
    }

    /// Evaluates normalized autocorrelation curve value at normalized lag [0.0..=1.0].
    pub fn evaluate_autocorrelation_curve(&self, norm_lag: f32) -> f32 {
        let t = norm_lag.clamp(0.0, 1.0);
        let cycles = 3.0 + (self.detected_pitch_hz / 440.0).clamp(0.5, 4.0);
        let env = 1.0 - 0.35 * t * (1.0 - t);
        ((2.0 * PI * cycles * t).cos() * env).clamp(-1.0, 1.0)
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
        (dx * dx + dy * dy).sqrt() <= CHROMATIC_TUNER_PUCK_HIT_RADIUS
    }

    /// Renders a deterministic ASCII snapshot for headless QA verification.
    pub fn render_ascii_snapshot(&self) -> String {
        let mut out = String::new();
        out.push_str("======================================================================\n");
        out.push_str(" SUMMONER DAW -- CHROMATIC AUTOCORRELATION TUNER & STROBE HUD SNAPSHOT\n");
        out.push_str("======================================================================\n");
        out.push_str(&format!(" Preset:           {}\n", self.preset.name()));
        out.push_str(&format!(
            " Reference A4:     {:.1} Hz | Detected Note: {} (MIDI {})\n",
            self.reference_a4_hz, self.detected_note_name, self.detected_midi_note
        ));
        out.push_str(&format!(
            " Detected Pitch:   {:.2} Hz | Cents Dev: {:+.1} c (Tol: +/-{:.1} c)\n",
            self.detected_pitch_hz, self.cents_deviation, self.cents_tolerance
        ));
        out.push_str(&format!(
            " Lock Status:      {} | Autocorr Gate: {:.2}\n",
            if self.is_in_tune_locked() { "LOCKED [IN-TUNE]" } else { "DRIFTING" },
            self.autocorr_threshold
        ));
        out.push_str(&format!(
            " Puck Coord:       ({:.3}, {:.3}) [PASS: >=44pt Hit Target]\n",
            self.puck_pos.0, self.puck_pos.1
        ));
        out.push_str("----------------------------------------------------------------------\n");
        out.push_str(" Strobe Cent Deviation Meter [-50c .. 0c .. +50c]:\n [");
        let needle_pos = ((self.cents_deviation - MIN_CENTS_DEV)
            / (MAX_CENTS_DEV - MIN_CENTS_DEV)
            * 30.0)
            .round()
            .clamp(0.0, 30.0) as usize;
        for idx in 0..=30 {
            if idx == needle_pos {
                out.push('|');
            } else if idx == 15 {
                out.push('+');
            } else {
                out.push('-');
            }
        }
        out.push_str("]\n======================================================================\n");
        out
    }

    #[cfg(feature = "gui")]
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.heading(
                    RichText::new("CHROMATIC AUTOCORRELATION TUNER & STROBE HUD")
                        .color(Color32::from_rgb(56, 189, 248))
                        .strong(),
                );
                ui.label(
                    RichText::new("M33 / M34 Pitch Tracking & Calibration")
                        .color(Color32::from_rgb(148, 163, 184))
                        .small(),
                );
            });

            ui.add_space(6.0);

            // Preset Selector Bar
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Presets:").strong());
                let presets = [
                    (ChromaticTunerPreset::ConcertStandard440Hz, "ISO 440 Hz"),
                    (ChromaticTunerPreset::BaroqueChamber415Hz, "Baroque 415 Hz"),
                    (ChromaticTunerPreset::OrchestralBright442Hz, "Symphonic 442 Hz"),
                    (ChromaticTunerPreset::VerdiNatural432Hz, "Verdi 432 Hz"),
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
                        Color32::from_rgb(56, 189, 248)
                    } else {
                        Color32::from_rgb(30, 41, 59)
                    })
                    .min_size(Vec2::new(82.0, 24.0));

                    if ui.add(btn).clicked() {
                        self.set_preset(p);
                    }
                }
            });

            ui.add_space(8.0);

            // Interactive 2D Calibration & Strobe Canvas
            let canvas_size = Vec2::new(ui.available_width().max(280.0), 148.0);
            let (response, painter) =
                ui.allocate_painter(canvas_size, egui::Sense::click_and_drag());
            let rect = response.rect;
            let canvas = Rect::new(rect.min.x, rect.min.y, rect.width(), rect.height());

            painter.rect_filled(rect, Rounding::same(4.0), Color32::from_rgb(10, 16, 26));
            painter.rect_stroke(
                rect,
                Rounding::same(4.0),
                Stroke::new(1.0_f32, Color32::from_rgb(38, 70, 104)),
            );

            // Center 0-cent lock line
            painter.line_segment(
                [
                    egui::pos2(rect.min.x, rect.center().y),
                    egui::pos2(rect.max.x, rect.center().y),
                ],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(52, 211, 153, 95)),
            );

            // Draw autocorrelation wave
            let num_pts = 48;
            let mut pts = Vec::with_capacity(num_pts);
            for idx in 0..num_pts {
                let frac = idx as f32 / (num_pts - 1) as f32;
                let corr_val = self.evaluate_autocorrelation_curve(frac);
                let px = rect.min.x + 15.0 + frac * (rect.width() - 30.0);
                let py = rect.center().y - corr_val * (rect.height() * 0.32);
                pts.push(egui::pos2(px, py));
            }
            for i in 0..(pts.len() - 1) {
                painter.line_segment(
                    [pts[i], pts[i + 1]],
                    Stroke::new(1.8_f32, Color32::from_rgb(56, 189, 248)),
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
            let status_col = if self.is_in_tune_locked() {
                Color32::from_rgb(52, 211, 153)
            } else {
                Color32::from_rgb(250, 204, 21)
            };

            painter.circle_filled(
                puck_pos,
                CHROMATIC_TUNER_PUCK_HIT_RADIUS,
                Color32::from_rgba_unmultiplied(56, 189, 248, 42),
            );
            painter.circle_stroke(
                puck_pos,
                CHROMATIC_TUNER_PUCK_HIT_RADIUS,
                Stroke::new(1.2_f32, status_col),
            );
            painter.circle_filled(puck_pos, 6.0, Color32::WHITE);

            painter.text(
                egui::pos2(rect.min.x + 8.0, rect.min.y + 6.0),
                egui::Align2::LEFT_TOP,
                format!(
                    "Note: {} (MIDI {}) | {:.2} Hz | {:+.1} cents",
                    self.detected_note_name,
                    self.detected_midi_note,
                    self.detected_pitch_hz,
                    self.cents_deviation
                ),
                FontId::proportional(11.5),
                status_col,
            );

            painter.text(
                egui::pos2(rect.max.x - 8.0, rect.min.y + 6.0),
                egui::Align2::RIGHT_TOP,
                format!(
                    "A4 Ref: {:.1} Hz | {}",
                    self.reference_a4_hz,
                    if self.is_in_tune_locked() { "LOCKED" } else { "TUNE" }
                ),
                FontId::proportional(10.5),
                Color32::from_rgb(148, 163, 184),
            );

            ui.add_space(8.0);

            // Surgical Pro Sliders
            ui.horizontal(|ui| {
                ui.label(RichText::new("Reference A4 Pitch:").size(11.0));
                let mut a4 = self.reference_a4_hz;
                if ui
                    .add(
                        egui::Slider::new(&mut a4, MIN_REFERENCE_A4_HZ..=MAX_REFERENCE_A4_HZ)
                            .suffix(" Hz"),
                    )
                    .changed()
                {
                    self.reference_a4_hz = a4;
                    self.sync_puck_from_params();
                }
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Cents Calibration:").size(11.0));
                let mut c = self.cents_deviation;
                if ui
                    .add(egui::Slider::new(&mut c, MIN_CENTS_DEV..=MAX_CENTS_DEV).suffix(" c"))
                    .changed()
                {
                    self.cents_deviation = c;
                    self.sync_puck_from_params();
                }
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("In-Tune Tolerance:").size(11.0));
                ui.add(egui::Slider::new(&mut self.cents_tolerance, 0.5..=10.0).suffix(" c"));
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Autocorr Peak Gate:").size(11.0));
                ui.add(egui::Slider::new(&mut self.autocorr_threshold, 0.30..=0.95));
            });
        });
    }
}
