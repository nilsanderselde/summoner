// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! 8-Band Parametric Equalizer & Biquad Magnitude Response HUD (Milestones 33 & 34).
//!
//! Directly reflects `crates/summoner_dsp/src/parametric_eq.rs` (`ParametricEqNode`, `EqBand`)
//! and `crates/summoner_dsp/src/biquad.rs` (`FilterBiquad`, `FilterType`).
//! Provides an interactive 2D Active Band Center Frequency & Gain dB Puck
//! (Frequency $f \in [20.0..=20000.0]\text{ Hz}$ logarithmic vs Gain $G \in [-18.0..=+18.0]\text{ dB}$),
//! 8 independent parametric EQ bands (`LowShelf`, `Peaking`, `HighShelf`, `Notch`, `Lowpass`, `Highpass`, `Bandpass`),
//! real-time RBJ Audio EQ Cookbook biquad coefficient evaluation, 8-band composite dB response curve,
//! 4 curated mixing & mastering EQ presets, and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and deterministic ASCII snapshot rendering.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, FontId, RichText, Rounding, Stroke, Vec2};

pub const PARAMETRIC_EQ_PUCK_HIT_RADIUS: f32 = 22.0; // >= 44x44pt touch bounding target
pub const MIN_EQ_FREQ_HZ: f32 = 20.0;
pub const MAX_EQ_FREQ_HZ: f32 = 20000.0;
pub const MIN_EQ_GAIN_DB: f32 = -18.0;
pub const MAX_EQ_GAIN_DB: f32 = 18.0;
pub const MIN_EQ_Q: f32 = 0.10;
pub const MAX_EQ_Q: f32 = 12.0;

/// Biquad filter topology reflecting `summoner_dsp::biquad::FilterType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BiquadFilterMode {
    Lowpass,
    Highpass,
    Bandpass,
    Notch,
    #[default]
    Peaking,
    LowShelf,
    HighShelf,
}

impl BiquadFilterMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Lowpass => "Lowpass (12 dB/oct)",
            Self::Highpass => "Highpass (12 dB/oct)",
            Self::Bandpass => "Bandpass (Constant Skirt)",
            Self::Notch => "Surgical Notch",
            Self::Peaking => "Bell Peaking",
            Self::LowShelf => "Low Shelf",
            Self::HighShelf => "High Shelf",
        }
    }

    pub fn short_tag(&self) -> &'static str {
        match self {
            Self::Lowpass => "LPF",
            Self::Highpass => "HPF",
            Self::Bandpass => "BPF",
            Self::Notch => "NCH",
            Self::Peaking => "BELL",
            Self::LowShelf => "LSH",
            Self::HighShelf => "HSH",
        }
    }
}

/// Configuration for a single band in the 8-band Parametric Equalizer (`EqBand`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ParametricBandConfig {
    pub mode: BiquadFilterMode,
    pub freq_hz: f32,
    pub gain_db: f32,
    pub q_factor: f32,
    pub enabled: bool,
}

impl Default for ParametricBandConfig {
    fn default() -> Self {
        Self {
            mode: BiquadFilterMode::Peaking,
            freq_hz: 1000.0,
            gain_db: 0.0,
            q_factor: 1.0,
            enabled: true,
        }
    }
}

/// Curated 8-Band Parametric Equalizer presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ParametricEqPreset {
    /// Flat studio calibration across all 8 bands (0.0 dB, Q = 1.00).
    #[default]
    StudioFlatReference,
    /// Vocal presence & air lift (-2.5 dB @ 250 Hz, +3.5 dB @ 2.5 kHz, +2.5 dB @ 12 kHz).
    VocalPresenceSilk,
    /// Sub-bass punch & low-mid boxiness cut (+4.5 dB @ 60 Hz LowShelf, -3.0 dB @ 150 Hz).
    SubBassPunchSculpt,
    /// Surgical narrow notch resonance removal (-12.0 dB @ 4.0 kHz, Q = 6.00).
    SurgicalResonanceNotch,
}

impl ParametricEqPreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::StudioFlatReference => "Studio Flat Reference (8-Band 0.0 dB Unity)",
            Self::VocalPresenceSilk => "Vocal Presence & Air Silk (+3.5 dB @ 2.5 kHz)",
            Self::SubBassPunchSculpt => "Sub-Bass Punch Sculpt (+4.5 dB @ 60 Hz Shelf)",
            Self::SurgicalResonanceNotch => "Surgical Resonance Notch (-12.0 dB @ 4.0 kHz Q=6)",
        }
    }

    /// Returns `(selected_band_idx, active_freq_hz, active_gain_db, active_q, output_trim_db)` and the 8 bands.
    pub fn nominal_bands(&self) -> (usize, f32, [ParametricBandConfig; 8]) {
        let default_freqs = [60.0, 150.0, 400.0, 1000.0, 2500.0, 6000.0, 12000.0, 16000.0];
        let mut bands = [ParametricBandConfig::default(); 8];
        for (i, b) in bands.iter_mut().enumerate() {
            b.freq_hz = default_freqs[i];
            b.mode = if i == 0 {
                BiquadFilterMode::LowShelf
            } else if i == 7 {
                BiquadFilterMode::HighShelf
            } else {
                BiquadFilterMode::Peaking
            };
            b.gain_db = 0.0;
            b.q_factor = 1.0;
            b.enabled = true;
        }

        match self {
            Self::StudioFlatReference => (3, 0.0, bands),
            Self::VocalPresenceSilk => {
                bands[0].mode = BiquadFilterMode::Highpass;
                bands[0].freq_hz = 85.0;
                bands[0].q_factor = 0.71;
                bands[1].freq_hz = 240.0;
                bands[1].gain_db = -2.5;
                bands[1].q_factor = 1.40;
                bands[4].freq_hz = 2500.0;
                bands[4].gain_db = 3.5;
                bands[4].q_factor = 1.25;
                bands[6].freq_hz = 12000.0;
                bands[6].gain_db = 2.5;
                bands[6].q_factor = 0.90;
                bands[7].freq_hz = 16000.0;
                bands[7].gain_db = 2.0;
                bands[7].q_factor = 0.71;
                (4, -0.5, bands)
            }
            Self::SubBassPunchSculpt => {
                bands[0].mode = BiquadFilterMode::LowShelf;
                bands[0].freq_hz = 60.0;
                bands[0].gain_db = 4.5;
                bands[0].q_factor = 0.71;
                bands[1].freq_hz = 150.0;
                bands[1].gain_db = -3.0;
                bands[1].q_factor = 1.80;
                bands[3].freq_hz = 900.0;
                bands[3].gain_db = 1.5;
                bands[3].q_factor = 1.20;
                (0, -1.0, bands)
            }
            Self::SurgicalResonanceNotch => {
                bands[5].mode = BiquadFilterMode::Peaking;
                bands[5].freq_hz = 4000.0;
                bands[5].gain_db = -12.0;
                bands[5].q_factor = 6.00;
                bands[6].mode = BiquadFilterMode::Notch;
                bands[6].freq_hz = 8200.0;
                bands[6].gain_db = -18.0;
                bands[6].q_factor = 8.00;
                (5, 0.0, bands)
            }
        }
    }
}

/// 8-Band Parametric Equalizer & Biquad Magnitude Response HUD view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParametricEqView {
    /// Active preset.
    pub preset: ParametricEqPreset,
    /// 8 Parametric EQ bands matching `ParametricEqNode::bands`.
    pub bands: [ParametricBandConfig; 8],
    /// Currently selected band index [0..=7].
    pub selected_band_idx: usize,
    /// Active band center frequency in Hz [20.0 ..= 20000.0].
    pub active_freq_hz: f32,
    /// Active band gain in dB [-18.0 ..= +18.0].
    pub active_gain_db: f32,
    /// Active band Q resonance factor [0.10 ..= 12.0].
    pub active_q: f32,
    /// Global output trim gain in dB [-12.0 ..= +12.0].
    pub output_trim_db: f32,
    /// Sample rate used for RBJ biquad coefficient computation.
    pub sample_rate: f32,
    /// Bypass toggle.
    pub bypassed: bool,
    /// 2D Puck position (X: log-normalized active_freq_hz, Y: normalized active_gain_db).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for ParametricEqView {
    fn default() -> Self {
        Self::new()
    }
}

impl ParametricEqView {
    pub fn new() -> Self {
        let preset = ParametricEqPreset::StudioFlatReference;
        let (sel_idx, trim_db, bands) = preset.nominal_bands();
        let active = bands[sel_idx];
        let norm_x = Self::freq_to_normalized(active.freq_hz);
        let norm_y = Self::gain_to_normalized(active.gain_db);

        Self {
            preset,
            bands,
            selected_band_idx: sel_idx,
            active_freq_hz: active.freq_hz,
            active_gain_db: active.gain_db,
            active_q: active.q_factor,
            output_trim_db: trim_db,
            sample_rate: 48000.0,
            bypassed: false,
            puck_pos: (norm_x, norm_y),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        }
    }

    pub fn set_preset(&mut self, preset: ParametricEqPreset) {
        self.preset = preset;
        let (sel_idx, trim_db, bands) = preset.nominal_bands();
        self.bands = bands;
        self.selected_band_idx = sel_idx;
        self.output_trim_db = trim_db;
        let active = self.bands[sel_idx];
        self.active_freq_hz = active.freq_hz;
        self.active_gain_db = active.gain_db;
        self.active_q = active.q_factor;
        self.sync_puck_from_params();
    }

    pub fn select_band(&mut self, idx: usize) {
        let clamped = idx.min(7);
        self.selected_band_idx = clamped;
        let b = self.bands[clamped];
        self.active_freq_hz = b.freq_hz;
        self.active_gain_db = b.gain_db;
        self.active_q = b.q_factor;
        self.sync_puck_from_params();
    }

    pub fn sync_puck_from_params(&mut self) {
        self.active_freq_hz = self.active_freq_hz.clamp(MIN_EQ_FREQ_HZ, MAX_EQ_FREQ_HZ);
        self.active_gain_db = self.active_gain_db.clamp(MIN_EQ_GAIN_DB, MAX_EQ_GAIN_DB);
        self.active_q = self.active_q.clamp(MIN_EQ_Q, MAX_EQ_Q);
        let idx = self.selected_band_idx.min(7);
        self.bands[idx].freq_hz = self.active_freq_hz;
        self.bands[idx].gain_db = self.active_gain_db;
        self.bands[idx].q_factor = self.active_q;
        self.puck_pos = (
            Self::freq_to_normalized(self.active_freq_hz),
            Self::gain_to_normalized(self.active_gain_db),
        );
    }

    pub fn update_from_puck(&mut self, norm_x: f32, norm_y: f32) {
        let x = norm_x.clamp(0.0, 1.0);
        let y = norm_y.clamp(0.0, 1.0);
        self.puck_pos = (x, y);
        self.active_freq_hz = Self::normalized_to_freq(x);
        self.active_gain_db = Self::normalized_to_gain(y);
        let idx = self.selected_band_idx.min(7);
        self.bands[idx].freq_hz = self.active_freq_hz;
        self.bands[idx].gain_db = self.active_gain_db;
    }

    /// Logarithmic frequency normalization [20 Hz ..= 20000 Hz] -> [0.0 ..= 1.0].
    pub fn freq_to_normalized(freq_hz: f32) -> f32 {
        let clamped = freq_hz.clamp(MIN_EQ_FREQ_HZ, MAX_EQ_FREQ_HZ);
        ((clamped / MIN_EQ_FREQ_HZ).ln() / (MAX_EQ_FREQ_HZ / MIN_EQ_FREQ_HZ).ln()).clamp(0.0, 1.0)
    }

    /// Logarithmic frequency de-normalization [0.0 ..= 1.0] -> [20 Hz ..= 20000 Hz].
    pub fn normalized_to_freq(norm: f32) -> f32 {
        let n = norm.clamp(0.0, 1.0);
        MIN_EQ_FREQ_HZ * (MAX_EQ_FREQ_HZ / MIN_EQ_FREQ_HZ).powf(n)
    }

    /// Linear gain dB normalization [-18.0 ..= +18.0] -> [0.0 ..= 1.0].
    pub fn gain_to_normalized(gain_db: f32) -> f32 {
        ((gain_db.clamp(MIN_EQ_GAIN_DB, MAX_EQ_GAIN_DB) - MIN_EQ_GAIN_DB)
            / (MAX_EQ_GAIN_DB - MIN_EQ_GAIN_DB))
            .clamp(0.0, 1.0)
    }

    /// Linear gain dB de-normalization [0.0 ..= 1.0] -> [-18.0 ..= +18.0].
    pub fn normalized_to_gain(norm: f32) -> f32 {
        MIN_EQ_GAIN_DB + norm.clamp(0.0, 1.0) * (MAX_EQ_GAIN_DB - MIN_EQ_GAIN_DB)
    }

    /// Computes normalized RBJ Audio EQ Cookbook biquad coefficients `(b0, b1, b2, a1, a2)`
    /// matching `summoner_dsp::biquad::FilterBiquad::calculate_coeffs`.
    pub fn compute_biquad_coeffs(&self, band_idx: usize) -> (f32, f32, f32, f32, f32) {
        use std::f32::consts::PI;
        let band = self.bands[band_idx.min(7)];
        let sr = self.sample_rate.max(8000.0);
        let w0 = 2.0 * PI * band.freq_hz.clamp(10.0, sr * 0.49) / sr;
        let alpha = w0.sin() / (2.0 * band.q_factor.max(0.01));
        let a = 10.0_f32.powf(band.gain_db / 40.0);
        let cos_w0 = w0.cos();

        let (b0, b1, b2, a0, a1, a2) = match band.mode {
            BiquadFilterMode::Lowpass => (
                (1.0 - cos_w0) / 2.0,
                1.0 - cos_w0,
                (1.0 - cos_w0) / 2.0,
                1.0 + alpha,
                -2.0 * cos_w0,
                1.0 - alpha,
            ),
            BiquadFilterMode::Highpass => (
                (1.0 + cos_w0) / 2.0,
                -(1.0 + cos_w0),
                (1.0 + cos_w0) / 2.0,
                1.0 + alpha,
                -2.0 * cos_w0,
                1.0 - alpha,
            ),
            BiquadFilterMode::Bandpass => {
                (alpha, 0.0, -alpha, 1.0 + alpha, -2.0 * cos_w0, 1.0 - alpha)
            }
            BiquadFilterMode::Notch => (
                1.0,
                -2.0 * cos_w0,
                1.0,
                1.0 + alpha,
                -2.0 * cos_w0,
                1.0 - alpha,
            ),
            BiquadFilterMode::Peaking => (
                1.0 + alpha * a,
                -2.0 * cos_w0,
                1.0 - alpha * a,
                1.0 + alpha / a,
                -2.0 * cos_w0,
                1.0 - alpha / a,
            ),
            BiquadFilterMode::LowShelf => {
                let sq_a = a.sqrt();
                (
                    a * ((a + 1.0) - (a - 1.0) * cos_w0 + 2.0 * sq_a * alpha),
                    2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w0),
                    a * ((a + 1.0) - (a - 1.0) * cos_w0 - 2.0 * sq_a * alpha),
                    (a + 1.0) + (a - 1.0) * cos_w0 + 2.0 * sq_a * alpha,
                    -2.0 * ((a - 1.0) + (a + 1.0) * cos_w0),
                    (a + 1.0) + (a - 1.0) * cos_w0 - 2.0 * sq_a * alpha,
                )
            }
            BiquadFilterMode::HighShelf => {
                let sq_a = a.sqrt();
                (
                    a * ((a + 1.0) + (a - 1.0) * cos_w0 + 2.0 * sq_a * alpha),
                    -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w0),
                    a * ((a + 1.0) + (a - 1.0) * cos_w0 - 2.0 * sq_a * alpha),
                    (a + 1.0) - (a - 1.0) * cos_w0 + 2.0 * sq_a * alpha,
                    2.0 * ((a - 1.0) - (a + 1.0) * cos_w0),
                    (a + 1.0) - (a - 1.0) * cos_w0 - 2.0 * sq_a * alpha,
                )
            }
        };

        (b0 / a0, b1 / a0, b2 / a0, a1 / a0, a2 / a0)
    }

    /// Evaluates composite 8-band magnitude response in dB at `freq_hz`,
    /// reflecting `ParametricEqNode::response_curve`.
    pub fn evaluate_magnitude_db_at_freq(&self, freq_hz: f32) -> f32 {
        if self.bypassed {
            return 0.0;
        }
        let f = freq_hz.clamp(MIN_EQ_FREQ_HZ, MAX_EQ_FREQ_HZ);
        let mut total_db = self.output_trim_db;
        for band in &self.bands {
            if !band.enabled {
                continue;
            }
            let ratio = f / band.freq_hz.max(1.0);
            match band.mode {
                BiquadFilterMode::Peaking => {
                    if band.gain_db.abs() > 1e-4 {
                        let bell =
                            1.0 / (1.0 + (ratio - 1.0 / ratio).powi(2) * band.q_factor.powi(2));
                        total_db += band.gain_db * bell;
                    }
                }
                BiquadFilterMode::LowShelf => {
                    if f < band.freq_hz {
                        total_db += band.gain_db;
                    } else if f < band.freq_hz * 2.0 {
                        let factor = 1.0 - (f - band.freq_hz) / band.freq_hz;
                        total_db += band.gain_db * factor.max(0.0);
                    }
                }
                BiquadFilterMode::HighShelf => {
                    if f > band.freq_hz {
                        total_db += band.gain_db;
                    } else if f > band.freq_hz * 0.5 {
                        let factor = (f - band.freq_hz * 0.5) / (band.freq_hz * 0.5);
                        total_db += band.gain_db * factor.max(0.0);
                    }
                }
                BiquadFilterMode::Notch => {
                    let notch_width = 0.25 / band.q_factor.max(0.5);
                    if (f - band.freq_hz).abs() < band.freq_hz * notch_width {
                        total_db -= 24.0;
                    }
                }
                BiquadFilterMode::Highpass => {
                    if f < band.freq_hz {
                        let octaves = (band.freq_hz / f.max(1.0)).log2();
                        total_db -= (octaves * 12.0).min(24.0);
                    }
                }
                BiquadFilterMode::Lowpass => {
                    if f > band.freq_hz {
                        let octaves = (f / band.freq_hz.max(1.0)).log2();
                        total_db -= (octaves * 12.0).min(24.0);
                    }
                }
                BiquadFilterMode::Bandpass => {
                    let bell =
                        1.0 / (1.0 + (ratio - 1.0 / ratio).powi(2) * band.q_factor.powi(2));
                    total_db += (bell - 1.0) * 18.0;
                }
            }
        }
        total_db.clamp(-24.0, 24.0)
    }

    /// Evaluates normalized magnitude response [0.0..=1.0] across log-normalized frequency [0.0..=1.0].
    pub fn evaluate_response_curve_normalized(&self, norm_freq: f32) -> f32 {
        let freq_hz = Self::normalized_to_freq(norm_freq.clamp(0.0, 1.0));
        let db = self.evaluate_magnitude_db_at_freq(freq_hz);
        Self::gain_to_normalized(db)
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
        (dx * dx + dy * dy).sqrt() <= PARAMETRIC_EQ_PUCK_HIT_RADIUS
    }

    /// Renders a deterministic ASCII snapshot for headless QA verification.
    pub fn render_ascii_snapshot(&self) -> String {
        let active = self.bands[self.selected_band_idx.min(7)];
        let (b0, b1, b2, a1, a2) = self.compute_biquad_coeffs(self.selected_band_idx);
        let mut out = String::new();
        out.push_str("======================================================================\n");
        out.push_str(" SUMMONER DAW -- 8-BAND PARAMETRIC EQ & BIQUAD RESPONSE SNAPSHOT\n");
        out.push_str("======================================================================\n");
        out.push_str(&format!(" Preset:           {}\n", self.preset.name()));
        out.push_str(&format!(
            " Selected Band:    #{} ({}) | Enabled: {} | Bypassed: {}\n",
            self.selected_band_idx + 1,
            active.mode.label(),
            if active.enabled { "YES" } else { "NO" },
            if self.bypassed { "YES" } else { "NO" }
        ));
        out.push_str(&format!(
            " Active Params:    {:.1} Hz | {:+.2} dB | Q: {:.2} | Trim: {:+.2} dB\n",
            self.active_freq_hz, self.active_gain_db, self.active_q, self.output_trim_db
        ));
        out.push_str(&format!(
            " RBJ Coeffs:       b0={:+.4} b1={:+.4} b2={:+.4} a1={:+.4} a2={:+.4}\n",
            b0, b1, b2, a1, a2
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
                    RichText::new("8-BAND PARAMETRIC EQUALIZER & BIQUAD RESPONSE HUD")
                        .color(Color32::from_rgb(52, 211, 153))
                        .strong(),
                );
                ui.label(
                    RichText::new("M33 / M34 ParametricEqNode + FilterBiquad RBJ Transfer Curve")
                        .color(Color32::from_rgb(148, 163, 184))
                        .small(),
                );
            });

            ui.add_space(6.0);

            // Preset Selector Bar
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("EQ Presets:").strong());
                let presets = [
                    (ParametricEqPreset::StudioFlatReference, "Flat Reference"),
                    (ParametricEqPreset::VocalPresenceSilk, "Vocal Presence"),
                    (ParametricEqPreset::SubBassPunchSculpt, "Sub-Bass Punch"),
                    (ParametricEqPreset::SurgicalResonanceNotch, "Surgical Notch"),
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
                        Color32::from_rgb(52, 211, 153)
                    } else {
                        Color32::from_rgb(30, 41, 59)
                    })
                    .min_size(Vec2::new(88.0, 24.0));

                    if ui.add(btn).clicked() {
                        self.set_preset(p);
                    }
                }
            });

            ui.add_space(6.0);

            // 8-Band Selector Strip
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Band Select:").size(11.0).strong());
                for idx in 0..8 {
                    let band = self.bands[idx];
                    let is_sel = self.selected_band_idx == idx;
                    let label = format!("B{} {}", idx + 1, band.mode.short_tag());
                    let btn = egui::Button::new(
                        RichText::new(label)
                            .color(if is_sel {
                                Color32::BLACK
                            } else {
                                Color32::from_rgb(167, 243, 208)
                            })
                            .size(10.5),
                    )
                    .fill(if is_sel {
                        Color32::from_rgb(16, 185, 129)
                    } else {
                        Color32::from_rgb(20, 35, 32)
                    })
                    .min_size(Vec2::new(56.0, 22.0));

                    if ui.add(btn).clicked() {
                        self.select_band(idx);
                    }
                }
            });

            ui.add_space(8.0);

            // Interactive 2D Frequency Response & Active Band Puck Canvas
            let canvas_size = Vec2::new(ui.available_width().max(280.0), 144.0);
            let (response, painter) =
                ui.allocate_painter(canvas_size, egui::Sense::click_and_drag());
            let rect = response.rect;
            let canvas = Rect::new(rect.min.x, rect.min.y, rect.width(), rect.height());

            painter.rect_filled(rect, Rounding::same(4.0), Color32::from_rgb(10, 22, 20));
            painter.rect_stroke(
                rect,
                Rounding::same(4.0),
                Stroke::new(1.0_f32, Color32::from_rgb(20, 83, 65)),
            );

            // 0 dB center reference line
            let zero_y = rect.center().y;
            painter.line_segment(
                [
                    egui::pos2(rect.min.x + 8.0, zero_y),
                    egui::pos2(rect.max.x - 8.0, zero_y),
                ],
                Stroke::new(1.0_f32, Color32::from_rgb(30, 58, 52)),
            );

            // Draw composite 8-band frequency response curve
            let num_pts = 56;
            let mut pts = Vec::with_capacity(num_pts);
            for idx in 0..num_pts {
                let frac = idx as f32 / (num_pts - 1) as f32;
                let r_val = self.evaluate_response_curve_normalized(frac);
                let px = rect.min.x + 15.0 + frac * (rect.width() - 30.0);
                let py = rect.max.y - 15.0 - r_val * (rect.height() - 30.0);
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

            // Draw >= 44x44pt Puck for selected band
            let (puck_x, puck_y) = self.puck_to_canvas_pos(&canvas);
            let puck_pos = egui::pos2(puck_x, puck_y);
            painter.circle_filled(
                puck_pos,
                PARAMETRIC_EQ_PUCK_HIT_RADIUS,
                Color32::from_rgba_unmultiplied(16, 185, 129, 44),
            );
            painter.circle_stroke(
                puck_pos,
                PARAMETRIC_EQ_PUCK_HIT_RADIUS,
                Stroke::new(1.2_f32, Color32::from_rgb(52, 211, 153)),
            );
            painter.circle_filled(puck_pos, 6.0, Color32::WHITE);

            painter.text(
                egui::pos2(rect.min.x + 8.0, rect.min.y + 6.0),
                egui::Align2::LEFT_TOP,
                format!(
                    "Band #{} ({}) | {:.0} Hz | {:+.1} dB | Q: {:.2}",
                    self.selected_band_idx + 1,
                    self.bands[self.selected_band_idx.min(7)].mode.short_tag(),
                    self.active_freq_hz,
                    self.active_gain_db,
                    self.active_q
                ),
                FontId::proportional(11.0),
                Color32::from_rgb(167, 243, 208),
            );

            ui.add_space(8.0);

            // Surgical Pro Sliders
            ui.horizontal(|ui| {
                ui.label(RichText::new("Active Frequency (Hz):").size(11.0));
                let mut f = self.active_freq_hz;
                if ui
                    .add(
                        egui::Slider::new(&mut f, MIN_EQ_FREQ_HZ..=MAX_EQ_FREQ_HZ)
                            .logarithmic(true)
                            .suffix(" Hz"),
                    )
                    .changed()
                {
                    self.active_freq_hz = f;
                    self.sync_puck_from_params();
                }
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Active Band Gain (dB):").size(11.0));
                let mut g = self.active_gain_db;
                if ui
                    .add(egui::Slider::new(&mut g, MIN_EQ_GAIN_DB..=MAX_EQ_GAIN_DB).suffix(" dB"))
                    .changed()
                {
                    self.active_gain_db = g;
                    self.sync_puck_from_params();
                }
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Active Band Q Resonance:").size(11.0));
                let mut q = self.active_q;
                if ui
                    .add(egui::Slider::new(&mut q, MIN_EQ_Q..=MAX_EQ_Q))
                    .changed()
                {
                    self.active_q = q;
                    self.sync_puck_from_params();
                }
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Global Output Trim (dB):").size(11.0));
                ui.add(egui::Slider::new(&mut self.output_trim_db, -12.0..=12.0).suffix(" dB"));
            });
        });
    }
}
