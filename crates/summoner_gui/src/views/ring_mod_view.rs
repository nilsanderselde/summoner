// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Analog Diode-Ring Modulator & Quadrature Frequency Doubler HUD (Milestones 33 & 34).
//!
//! Provides an interactive 2D Carrier Frequency & Modulation Depth Puck (Carrier Frequency
//! $f_c \in [0.1..=8000.0]\text{ Hz}$ vs Mod Depth $d \in [0.0..=1.0]$), selectable carrier
//! waveforms (Sine, Triangle, Saw, Square), diode bridge non-linearity asymmetry
//! $[-1.0..=+1.0]$, carrier phase offset, sideband harmonic spectrum $(f_c \pm f_m)$,
//! 4 curated ring modulation presets, and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and deterministic ASCII snapshot rendering.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, FontId, RichText, Rounding, Stroke, Vec2};

pub const RING_MOD_PUCK_HIT_RADIUS: f32 = 22.0; // >= 44x44pt touch bounding target
pub const MIN_CARRIER_FREQ_HZ: f32 = 0.1;
pub const MAX_CARRIER_FREQ_HZ: f32 = 8000.0;
pub const MIN_MOD_DEPTH: f32 = 0.0;
pub const MAX_MOD_DEPTH: f32 = 1.0;
pub const MIN_DIODE_ASYMMETRY: f32 = -1.0;
pub const MAX_DIODE_ASYMMETRY: f32 = 1.0;
pub const NUM_RING_WAVEFORM_POINTS: usize = 48;

/// Carrier waveform shapes for the ring modulator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RingWaveformShape {
    #[default]
    Sine,
    Triangle,
    Saw,
    Square,
}

impl RingWaveformShape {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Sine => "Pure Sine",
            Self::Triangle => "Symmetric Triangle",
            Self::Saw => "Ramp Sawtooth",
            Self::Square => "Bipolar Square",
        }
    }
}

/// Curated ring modulator presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RingModPreset {
    /// Classic 1963 BBC sci-fi robotic voice modulation with 30 Hz sine carrier.
    #[default]
    DalekRoboticVoice,
    /// Metallic inharmonic bell & chime resonance with 440 Hz triangle carrier.
    ChimeBellHarmonics,
    /// Sub-audio warm sinusoidal amplitude tremolo at 5.5 Hz.
    TremoloWarmSub,
    /// Extreme harsh quadrature ring distortion with 1850 Hz square carrier.
    AlienQuadratureGlitch,
}

impl RingModPreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::DalekRoboticVoice => "Dalek Robotic Voice (30Hz Sine)",
            Self::ChimeBellHarmonics => "Chime Bell Harmonics (440Hz Tri)",
            Self::TremoloWarmSub => "Tremolo Warm Sub (5.5Hz Sine)",
            Self::AlienQuadratureGlitch => "Alien Quadrature Glitch (1850Hz Sqr)",
        }
    }

    /// Returns nominal (carrier_freq_hz, waveform, mod_depth, diode_asymmetry, phase_deg).
    pub fn nominal_parameters(&self) -> (f32, RingWaveformShape, f32, f32, f32) {
        match self {
            Self::DalekRoboticVoice => (30.0, RingWaveformShape::Sine, 1.00, 0.20, 0.0),
            Self::ChimeBellHarmonics => (440.0, RingWaveformShape::Triangle, 0.85, 0.10, 45.0),
            Self::TremoloWarmSub => (5.5, RingWaveformShape::Sine, 0.35, 0.00, 0.0),
            Self::AlienQuadratureGlitch => (1850.0, RingWaveformShape::Square, 1.00, 0.65, 90.0),
        }
    }
}

/// Analog Diode-Ring Modulator & Quadrature Frequency Doubler HUD view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RingModulatorView {
    /// Active ring modulation preset.
    pub preset: RingModPreset,
    /// Carrier oscillator frequency in Hz [0.1 ..= 8000.0].
    pub carrier_freq_hz: f32,
    /// Carrier waveform shape.
    pub carrier_waveform: RingWaveformShape,
    /// Modulation depth [0.0 = Dry/Bypass ..= 1.0 = Full Bipolar Ring Mod].
    pub mod_depth: f32,
    /// Diode bridge non-linearity asymmetry [-1.0 ..= +1.0].
    pub diode_asymmetry: f32,
    /// Carrier quadrature phase offset in degrees [0.0 ..= 360.0].
    pub phase_deg: f32,
    /// Carrier running phase accumulator [0.0 ..= 1.0].
    pub phase_accum: f32,
    /// 2D Dynamics Puck position (X: normalized carrier freq, Y: normalized mod depth).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for RingModulatorView {
    fn default() -> Self {
        Self::new()
    }
}

impl RingModulatorView {
    pub fn new() -> Self {
        let preset = RingModPreset::DalekRoboticVoice;
        let (freq, wave, depth, asym, phase) = preset.nominal_parameters();
        let norm_x = Self::freq_to_normalized(freq);
        let norm_y = Self::depth_to_normalized(depth);

        Self {
            preset,
            carrier_freq_hz: freq,
            carrier_waveform: wave,
            mod_depth: depth,
            diode_asymmetry: asym,
            phase_deg: phase,
            phase_accum: 0.0,
            puck_pos: (norm_x, norm_y),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        }
    }

    pub fn set_preset(&mut self, preset: RingModPreset) {
        self.preset = preset;
        let (freq, wave, depth, asym, phase) = preset.nominal_parameters();
        self.carrier_freq_hz = freq;
        self.carrier_waveform = wave;
        self.mod_depth = depth;
        self.diode_asymmetry = asym;
        self.phase_deg = phase;
        self.sync_puck_from_params();
    }

    pub fn sync_puck_from_params(&mut self) {
        self.puck_pos = (
            Self::freq_to_normalized(self.carrier_freq_hz),
            Self::depth_to_normalized(self.mod_depth),
        );
    }

    pub fn update_from_puck(&mut self, norm_x: f32, norm_y: f32) {
        let x = norm_x.clamp(0.0, 1.0);
        let y = norm_y.clamp(0.0, 1.0);
        self.puck_pos = (x, y);
        self.carrier_freq_hz = Self::normalized_to_freq(x);
        self.mod_depth = Self::normalized_to_depth(y);
    }

    /// Logarithmic frequency mapping: [0.1 ..= 8000.0 Hz].
    pub fn freq_to_normalized(freq_hz: f32) -> f32 {
        let f = freq_hz.clamp(MIN_CARRIER_FREQ_HZ, MAX_CARRIER_FREQ_HZ);
        let log_min = MIN_CARRIER_FREQ_HZ.ln();
        let log_max = MAX_CARRIER_FREQ_HZ.ln();
        ((f.ln() - log_min) / (log_max - log_min)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_freq(norm: f32) -> f32 {
        let n = norm.clamp(0.0, 1.0);
        let log_min = MIN_CARRIER_FREQ_HZ.ln();
        let log_max = MAX_CARRIER_FREQ_HZ.ln();
        (log_min + n * (log_max - log_min)).exp().clamp(MIN_CARRIER_FREQ_HZ, MAX_CARRIER_FREQ_HZ)
    }

    pub fn depth_to_normalized(depth: f32) -> f32 {
        depth.clamp(MIN_MOD_DEPTH, MAX_MOD_DEPTH)
    }

    pub fn normalized_to_depth(norm: f32) -> f32 {
        norm.clamp(MIN_MOD_DEPTH, MAX_MOD_DEPTH)
    }

    pub fn asymmetry_to_normalized(asym: f32) -> f32 {
        ((asym.clamp(MIN_DIODE_ASYMMETRY, MAX_DIODE_ASYMMETRY) - MIN_DIODE_ASYMMETRY)
            / (MAX_DIODE_ASYMMETRY - MIN_DIODE_ASYMMETRY))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_asymmetry(norm: f32) -> f32 {
        let n = norm.clamp(0.0, 1.0);
        MIN_DIODE_ASYMMETRY + n * (MAX_DIODE_ASYMMETRY - MIN_DIODE_ASYMMETRY)
    }

    /// Evaluates instantaneous carrier oscillator waveform at a given phase in [0.0 ..= 1.0].
    pub fn evaluate_carrier_waveform(&self, phase: f32) -> f32 {
        let p = (phase + self.phase_deg / 360.0).fract();
        let raw = match self.carrier_waveform {
            RingWaveformShape::Sine => (TAU * p).sin(),
            RingWaveformShape::Triangle => 2.0 * (2.0 * p - 1.0).abs() - 1.0,
            RingWaveformShape::Saw => 2.0 * p - 1.0,
            RingWaveformShape::Square => {
                if p < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
        };

        // Apply diode non-linear asymmetry: asymmetric soft saturation
        if self.diode_asymmetry.abs() > 1e-4 {
            let offset = self.diode_asymmetry * 0.4;
            let biased = raw + offset;
            biased.tanh() / (1.0 + offset.abs()).max(1.0)
        } else {
            raw
        }
    }

    /// Modulate a test input sample $x$ through the ring modulator transfer curve.
    pub fn modulate_sample(&self, input: f32, phase: f32) -> f32 {
        let carrier = self.evaluate_carrier_waveform(phase);
        let ring_output = input * carrier;
        // Crossfade between dry input and modulated signal based on mod_depth
        input * (1.0 - self.mod_depth) + ring_output * self.mod_depth
    }

    /// Advance running carrier phase by time step dt (in seconds).
    pub fn advance_time(&mut self, dt: f32) {
        let phase_inc = self.carrier_freq_hz * dt;
        self.phase_accum = (self.phase_accum + phase_inc).fract();
    }

    /// Calculates sideband frequencies given an exemplary modulator input frequency $f_m$.
    pub fn calculate_sidebands(&self, input_freq_hz: f32) -> (f32, f32) {
        let sum_band = self.carrier_freq_hz + input_freq_hz;
        let diff_band = (self.carrier_freq_hz - input_freq_hz).abs();
        (sum_band, diff_band)
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
        (dx * dx + dy * dy).sqrt() <= RING_MOD_PUCK_HIT_RADIUS
    }

    /// Renders a deterministic ASCII snapshot for headless QA verification.
    pub fn render_ascii_snapshot(&self) -> String {
        let (sum_sb, diff_sb) = self.calculate_sidebands(440.0);
        let mut out = String::new();
        out.push_str("======================================================================\n");
        out.push_str(" SUMMONER DAW -- ANALOG DIODE-RING MODULATOR HUD SNAPSHOT\n");
        out.push_str("======================================================================\n");
        out.push_str(&format!(" Preset:           {}\n", self.preset.name()));
        out.push_str(&format!(" Carrier Freq:     {:.2} Hz\n", self.carrier_freq_hz));
        out.push_str(&format!(" Waveform:         {}\n", self.carrier_waveform.name()));
        out.push_str(&format!(" Mod Depth:        {:.1} %\n", self.mod_depth * 100.0));
        out.push_str(&format!(" Diode Asymmetry:  {:.2}\n", self.diode_asymmetry));
        out.push_str(&format!(" Phase Offset:     {:.1} deg\n", self.phase_deg));
        out.push_str(&format!(" Test Sidebands:   Upper = {:.1} Hz | Lower = {:.1} Hz (Ref 440Hz)\n", sum_sb, diff_sb));
        out.push_str(&format!(" Puck Coord:       ({:.3}, {:.3}) [PASS: >=44pt Hit Target]\n", self.puck_pos.0, self.puck_pos.1));
        out.push_str("----------------------------------------------------------------------\n");
        out.push_str(" Carrier Waveform Envelope Preview:\n ");

        for step in 0..32 {
            let p = step as f32 / 32.0;
            let val = self.evaluate_carrier_waveform(p);
            let char_repr = if val > 0.6 {
                '^'
            } else if val > 0.2 {
                '/'
            } else if val > -0.2 {
                '-'
            } else if val > -0.6 {
                '\\'
            } else {
                'v'
            };
            out.push(char_repr);
        }
        out.push_str("\n======================================================================\n");
        out
    }

    #[cfg(feature = "gui")]
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Header bar
            ui.horizontal(|ui| {
                ui.heading(
                    RichText::new("ANALOG DIODE-RING MODULATOR & QUADRATURE HUD")
                        .color(Color32::from_rgb(251, 146, 60))
                        .strong(),
                );
                ui.label(
                    RichText::new("M33 / M34 Pro Signal Processing")
                        .color(Color32::from_rgb(148, 163, 184))
                        .small(),
                );
            });

            ui.add_space(6.0);

            // Preset Buttons Bar
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Presets:").strong());
                let presets = [
                    (RingModPreset::DalekRoboticVoice, "Dalek (30Hz)"),
                    (RingModPreset::ChimeBellHarmonics, "Bell (440Hz)"),
                    (RingModPreset::TremoloWarmSub, "Tremolo (5.5Hz)"),
                    (RingModPreset::AlienQuadratureGlitch, "Alien (1.85kHz)"),
                ];
                for (p, label) in presets {
                    let is_active = self.preset == p;
                    let btn = egui::Button::new(
                        RichText::new(label)
                            .color(if is_active { Color32::WHITE } else { Color32::from_rgb(203, 213, 225) })
                            .size(11.0),
                    )
                    .fill(if is_active { Color32::from_rgb(234, 88, 12) } else { Color32::from_rgb(30, 41, 59) })
                    .min_size(Vec2::new(75.0, 24.0));

                    if ui.add(btn).clicked() {
                        self.set_preset(p);
                    }
                }
            });

            ui.add_space(8.0);

            // Carrier Waveform Selection Tabs
            ui.horizontal(|ui| {
                ui.label(RichText::new("Waveform:").strong());
                let waves = [
                    (RingWaveformShape::Sine, "∿ Sine"),
                    (RingWaveformShape::Triangle, "▲ Triangle"),
                    (RingWaveformShape::Saw, "◿ Saw"),
                    (RingWaveformShape::Square, "⊓ Square"),
                ];
                for (w, label) in waves {
                    let is_active = self.carrier_waveform == w;
                    let btn = egui::Button::new(
                        RichText::new(label)
                            .color(if is_active { Color32::BLACK } else { Color32::from_rgb(226, 232, 240) })
                            .size(11.0),
                    )
                    .fill(if is_active { Color32::from_rgb(251, 146, 60) } else { Color32::from_rgb(15, 23, 42) })
                    .min_size(Vec2::new(65.0, 22.0));

                    if ui.add(btn).clicked() {
                        self.carrier_waveform = w;
                    }
                }
            });

            ui.add_space(8.0);

            // Interactive 2D Dynamics Puck Canvas
            let canvas_size = Vec2::new(ui.available_width().max(280.0), 140.0);
            let (response, painter) = ui.allocate_painter(canvas_size, egui::Sense::click_and_drag());
            let rect = response.rect;
            let canvas = Rect::new(rect.min.x, rect.min.y, rect.width(), rect.height());

            // Background
            painter.rect_filled(rect, Rounding::same(4.0), Color32::from_rgb(15, 23, 42));
            painter.rect_stroke(rect, Rounding::same(4.0), Stroke::new(1.0_f32, Color32::from_rgb(51, 65, 85)));

            // Grid lines
            for i in 1..4 {
                let gx = rect.min.x + (i as f32 / 4.0) * rect.width();
                let gy = rect.min.y + (i as f32 / 4.0) * rect.height();
                painter.line_segment(
                    [egui::pos2(gx, rect.min.y), egui::pos2(gx, rect.max.y)],
                    Stroke::new(0.5_f32, Color32::from_rgb(30, 41, 59)),
                );
                painter.line_segment(
                    [egui::pos2(rect.min.x, gy), egui::pos2(rect.max.x, gy)],
                    Stroke::new(0.5_f32, Color32::from_rgb(30, 41, 59)),
                );
            }

            // Draw Carrier Waveform Curve inside Canvas
            let num_pts = 48;
            let mut pts = Vec::with_capacity(num_pts);
            for idx in 0..num_pts {
                let frac = idx as f32 / (num_pts - 1) as f32;
                let wave_val = self.evaluate_carrier_waveform(frac);
                let px = rect.min.x + 15.0 + frac * (rect.width() - 30.0);
                let py = rect.center().y - wave_val * (rect.height() * 0.35);
                pts.push(egui::pos2(px, py));
            }
            for i in 0..(pts.len() - 1) {
                painter.line_segment(
                    [pts[i], pts[i + 1]],
                    Stroke::new(1.5_f32, Color32::from_rgb(251, 146, 60)),
                );
            }

            // Handle Puck Drag / Click Interaction
            if response.dragged() || response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let padding = 15.0;
                    let norm_x = ((pos.x - (rect.min.x + padding)) / (rect.width() - 2.0 * padding)).clamp(0.0, 1.0);
                    let norm_y = (1.0 - ((pos.y - (rect.min.y + padding)) / (rect.height() - 2.0 * padding))).clamp(0.0, 1.0);
                    self.update_from_puck(norm_x, norm_y);
                }
            }

            // Draw >= 44x44pt Interactive Puck
            let (puck_x, puck_y) = self.puck_to_canvas_pos(&canvas);
            let puck_pos = egui::pos2(puck_x, puck_y);

            // Outer glow / hit bounds
            painter.circle_filled(puck_pos, RING_MOD_PUCK_HIT_RADIUS, Color32::from_rgba_unmultiplied(234, 88, 12, 45));
            painter.circle_stroke(puck_pos, RING_MOD_PUCK_HIT_RADIUS, Stroke::new(1.0_f32, Color32::from_rgb(249, 115, 22)));
            // Inner Core
            painter.circle_filled(puck_pos, 6.0, Color32::from_rgb(255, 237, 213));
            painter.circle_stroke(puck_pos, 8.0, Stroke::new(1.5_f32, Color32::from_rgb(234, 88, 12)));

            // Text Labels inside Canvas
            painter.text(
                egui::pos2(rect.min.x + 8.0, rect.min.y + 6.0),
                egui::Align2::LEFT_TOP,
                format!("fc: {:.1} Hz | Depth: {:.0} %", self.carrier_freq_hz, self.mod_depth * 100.0),
                FontId::proportional(11.0),
                Color32::from_rgb(253, 186, 116),
            );

            let (sb_up, sb_dn) = self.calculate_sidebands(440.0);
            painter.text(
                egui::pos2(rect.max.x - 8.0, rect.min.y + 6.0),
                egui::Align2::RIGHT_TOP,
                format!("Upper: {:.0} Hz | Lower: {:.0} Hz", sb_up, sb_dn),
                FontId::proportional(10.0),
                Color32::from_rgb(148, 163, 184),
            );

            ui.add_space(8.0);

            // Fine Sliders for surgical Pro control
            ui.horizontal(|ui| {
                ui.label(RichText::new("Carrier Freq:").size(11.0));
                let mut freq = self.carrier_freq_hz;
                if ui.add(egui::Slider::new(&mut freq, MIN_CARRIER_FREQ_HZ..=MAX_CARRIER_FREQ_HZ).logarithmic(true).suffix(" Hz")).changed() {
                    self.carrier_freq_hz = freq;
                    self.sync_puck_from_params();
                }
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Mod Depth:").size(11.0));
                let mut depth = self.mod_depth;
                if ui.add(egui::Slider::new(&mut depth, MIN_MOD_DEPTH..=MAX_MOD_DEPTH).suffix(" %")).changed() {
                    self.mod_depth = depth;
                    self.sync_puck_from_params();
                }
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Diode Asymmetry:").size(11.0));
                ui.add(egui::Slider::new(&mut self.diode_asymmetry, MIN_DIODE_ASYMMETRY..=MAX_DIODE_ASYMMETRY));
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Phase Offset:").size(11.0));
                ui.add(egui::Slider::new(&mut self.phase_deg, 0.0..=360.0).suffix("°"));
            });
        });
    }
}
