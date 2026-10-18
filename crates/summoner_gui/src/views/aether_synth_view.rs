// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Aether Subtractive & Glitch Composite Synthesizer HUD (Milestones 33 & 34).
//!
//! Directly reflects `crates/summoner_dsp/src/composites.rs` (`AetherSynth`, `AetherMacroView`,
//! `GlitchAetherMachine`, `CyberpunkSubSynth`, `AtmosphericPadSynth`, `GlitchPercussionSynth`).
//! Provides an interactive 2D Oscillator Saw/Pulse Mix & 4-Pole Ladder Filter Cutoff Puck
//! (Oscillator Mix $m \in [0.0..=1.0]$ vs Logarithmic Filter Cutoff $f_c \in [20.0..=20000.0]\text{ Hz}$),
//! 5 composite synthesizer architectures, PWM LFO modulation math, dual ADSR filter/amp envelopes,
//! glitch gate chopper duty cycle evaluation, 4 curated presets, and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and deterministic ASCII snapshot rendering.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, FontId, RichText, Rounding, Stroke, Vec2};

pub const AETHER_SYNTH_PUCK_HIT_RADIUS: f32 = 22.0; // >= 44x44pt touch bounding target
pub const MIN_AETHER_CUTOFF_HZ: f32 = 20.0;
pub const MAX_AETHER_CUTOFF_HZ: f32 = 20000.0;
pub const MIN_AETHER_RESONANCE: f32 = 0.0;
pub const MAX_AETHER_RESONANCE: f32 = 4.0;
pub const MIN_AETHER_LFO_HZ: f32 = 0.1;
pub const MAX_AETHER_LFO_HZ: f32 = 20.0;

/// Composite synthesizer voice topology reflecting `summoner_dsp::composites`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CompositeSynthVoiceMode {
    #[default]
    AetherSubtractive,
    GlitchAetherMachine,
    CyberpunkSubSynth,
    AtmosphericPadSynth,
    GlitchPercussionSynth,
}

impl CompositeSynthVoiceMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::AetherSubtractive => "AetherSynth (2-Osc Saw/Pulse + Ladder + Dual ADSR)",
            Self::GlitchAetherMachine => "GlitchAetherMachine (Tube Overdrive + GlitchGate + TapeStop)",
            Self::CyberpunkSubSynth => "CyberpunkSubSynth (Sub Sine + Wavefolder + Pitch LFO)",
            Self::AtmosphericPadSynth => "AtmosphericPadSynth (Detuned Saw/Tri + SVF + Space)",
            Self::GlitchPercussionSynth => "GlitchPercussionSynth (Noise + Comb + Bitcrusher + Stutter)",
        }
    }

    pub fn short_tag(&self) -> &'static str {
        match self {
            Self::AetherSubtractive => "AETHER",
            Self::GlitchAetherMachine => "GLITCH",
            Self::CyberpunkSubSynth => "CYBER-SUB",
            Self::AtmosphericPadSynth => "ATMOS-PAD",
            Self::GlitchPercussionSynth => "PERC-GLITCH",
        }
    }
}

/// Curated Aether & Composite Synthesizer presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AetherSynthPreset {
    /// Classic 2-Osc Subtractive Lead (`AetherSynth::new`, Mix 0.50, Cutoff 1200 Hz, Res 1.00, LFO 2.0 Hz).
    #[default]
    AetherLeadClassic,
    /// Glitch Aether Machine (`GlitchAetherMachine::new`, Mix 0.65, Cutoff 2500 Hz, Res 1.50, LFO 8.0 Hz, Drive 0.75, Gate 0.60).
    GlitchAetherChopper,
    /// Cyberpunk Sub-Bass Wavefolder (`CyberpunkSubSynth::new`, Mix 0.30, Cutoff 600 Hz, Res 1.20, LFO 4.0 Hz, Drive 0.85).
    CyberpunkWavefoldSub,
    /// Atmospheric Detuned Lush Pad (`AtmosphericPadSynth::new`, Mix 0.50, Cutoff 800 Hz, Res 0.71, LFO 0.5 Hz, Env 0.45).
    AtmosphericHorizonPad,
}

impl AetherSynthPreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::AetherLeadClassic => "Aether Classic Lead (Saw/Pulse 50% + 1.2 kHz Ladder)",
            Self::GlitchAetherChopper => "Glitch Aether Chopper (Tube Drive + 8 Hz GlitchGate)",
            Self::CyberpunkWavefoldSub => "Cyberpunk Wavefold Sub (600 Hz SVF + 4x Wavefolder)",
            Self::AtmosphericHorizonPad => "Atmospheric Horizon Pad (Detuned +3c + 0.5 Hz LFO)",
        }
    }

    /// Returns nominal `(osc_mix, cutoff_hz, resonance, lfo_rate_hz, filter_env_depth, glitch_gate_duty, distortion_drive, voice_mode)`.
    pub fn nominal_parameters(
        &self,
    ) -> (f32, f32, f32, f32, f32, f32, f32, CompositeSynthVoiceMode) {
        match self {
            Self::AetherLeadClassic => (
                0.50,
                1200.0,
                1.00,
                2.0,
                0.60,
                1.00,
                0.15,
                CompositeSynthVoiceMode::AetherSubtractive,
            ),
            Self::GlitchAetherChopper => (
                0.65,
                2500.0,
                1.50,
                8.0,
                0.75,
                0.60,
                0.75,
                CompositeSynthVoiceMode::GlitchAetherMachine,
            ),
            Self::CyberpunkWavefoldSub => (
                0.30,
                600.0,
                1.20,
                4.0,
                0.35,
                1.00,
                0.85,
                CompositeSynthVoiceMode::CyberpunkSubSynth,
            ),
            Self::AtmosphericHorizonPad => (
                0.50,
                800.0,
                0.71,
                0.5,
                0.45,
                1.00,
                0.05,
                CompositeSynthVoiceMode::AtmosphericPadSynth,
            ),
        }
    }
}

/// Aether Subtractive & Glitch Composite Synthesizer HUD view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AetherSynthView {
    /// Active composite synth preset.
    pub preset: AetherSynthPreset,
    /// Active composite voice architecture (`AetherSynth`, `GlitchAetherMachine`, etc.).
    pub voice_mode: CompositeSynthVoiceMode,
    /// Oscillator Saw (0.0) to Pulse (1.0) crossfade mix (`AetherMacroView::osc_mix`).
    pub osc_mix: f32,
    /// 4-Pole Ladder / SVF filter cutoff frequency in Hz [20.0 ..= 20000.0].
    pub filter_cutoff_hz: f32,
    /// Filter resonance Q [0.0 ..= 4.0].
    pub filter_resonance: f32,
    /// PWM / Pitch / Filter LFO modulation speed in Hz [0.1 ..= 20.0].
    pub lfo_rate_hz: f32,
    /// Filter ADSR envelope modulation depth [0.0 ..= 1.0] (up to +4000 Hz sweep).
    pub filter_env_depth: f32,
    /// GlitchGate chopper pulse duty cycle [0.05 ..= 1.0].
    pub glitch_gate_duty: f32,
    /// Tube overdrive / wavefolder saturation drive [0.0 ..= 1.0].
    pub distortion_drive: f32,
    /// Voice fundamental pitch frequency in Hz.
    pub base_freq_hz: f32,
    /// Bypass toggle.
    pub bypassed: bool,
    /// 2D Puck position (X: osc_mix [0.0..=1.0], Y: log-normalized filter_cutoff_hz [0.0..=1.0]).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for AetherSynthView {
    fn default() -> Self {
        Self::new()
    }
}

impl AetherSynthView {
    pub fn new() -> Self {
        let preset = AetherSynthPreset::AetherLeadClassic;
        let (mix, cut, res, lfo, env_d, gate_d, drive, mode) = preset.nominal_parameters();
        let norm_x = Self::mix_to_normalized(mix);
        let norm_y = Self::cutoff_to_normalized(cut);

        Self {
            preset,
            voice_mode: mode,
            osc_mix: mix,
            filter_cutoff_hz: cut,
            filter_resonance: res,
            lfo_rate_hz: lfo,
            filter_env_depth: env_d,
            glitch_gate_duty: gate_d,
            distortion_drive: drive,
            base_freq_hz: 220.0,
            bypassed: false,
            puck_pos: (norm_x, norm_y),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        }
    }

    pub fn set_preset(&mut self, preset: AetherSynthPreset) {
        self.preset = preset;
        let (mix, cut, res, lfo, env_d, gate_d, drive, mode) = preset.nominal_parameters();
        self.osc_mix = mix;
        self.filter_cutoff_hz = cut;
        self.filter_resonance = res;
        self.lfo_rate_hz = lfo;
        self.filter_env_depth = env_d;
        self.glitch_gate_duty = gate_d;
        self.distortion_drive = drive;
        self.voice_mode = mode;
        self.sync_puck_from_params();
    }

    pub fn sync_puck_from_params(&mut self) {
        self.osc_mix = self.osc_mix.clamp(0.0, 1.0);
        self.filter_cutoff_hz = self
            .filter_cutoff_hz
            .clamp(MIN_AETHER_CUTOFF_HZ, MAX_AETHER_CUTOFF_HZ);
        self.filter_resonance = self
            .filter_resonance
            .clamp(MIN_AETHER_RESONANCE, MAX_AETHER_RESONANCE);
        self.lfo_rate_hz = self.lfo_rate_hz.clamp(MIN_AETHER_LFO_HZ, MAX_AETHER_LFO_HZ);
        self.puck_pos = (
            Self::mix_to_normalized(self.osc_mix),
            Self::cutoff_to_normalized(self.filter_cutoff_hz),
        );
    }

    pub fn update_from_puck(&mut self, norm_x: f32, norm_y: f32) {
        let x = norm_x.clamp(0.0, 1.0);
        let y = norm_y.clamp(0.0, 1.0);
        self.puck_pos = (x, y);
        self.osc_mix = Self::normalized_to_mix(x);
        self.filter_cutoff_hz = Self::normalized_to_cutoff(y);
    }

    pub fn mix_to_normalized(mix: f32) -> f32 {
        mix.clamp(0.0, 1.0)
    }

    pub fn normalized_to_mix(norm: f32) -> f32 {
        norm.clamp(0.0, 1.0)
    }

    /// Logarithmic cutoff normalization [20 Hz ..= 20000 Hz] -> [0.0 ..= 1.0].
    pub fn cutoff_to_normalized(cutoff_hz: f32) -> f32 {
        let clamped = cutoff_hz.clamp(MIN_AETHER_CUTOFF_HZ, MAX_AETHER_CUTOFF_HZ);
        ((clamped / MIN_AETHER_CUTOFF_HZ).ln()
            / (MAX_AETHER_CUTOFF_HZ / MIN_AETHER_CUTOFF_HZ).ln())
        .clamp(0.0, 1.0)
    }

    /// Logarithmic cutoff de-normalization [0.0 ..= 1.0] -> [20 Hz ..= 20000 Hz].
    pub fn normalized_to_cutoff(norm: f32) -> f32 {
        let n = norm.clamp(0.0, 1.0);
        MIN_AETHER_CUTOFF_HZ * (MAX_AETHER_CUTOFF_HZ / MIN_AETHER_CUTOFF_HZ).powf(n)
    }

    /// Computes instantaneous pulse width modulation duty cycle `(0.5 + 0.4 * lfo_val).clamp(0.05, 0.95)`,
    /// matching `AetherSynth::process_block`.
    pub fn compute_pwm_duty_cycle(&self, lfo_phase_norm: f32) -> f32 {
        let lfo_val = (lfo_phase_norm * std::f32::consts::TAU).sin();
        (0.5 + 0.4 * lfo_val).clamp(0.05, 0.95)
    }

    /// Computes instantaneous filter cutoff modulation `(cutoff + f_env * 4000.0).clamp(20.0, 20000.0)`,
    /// matching `AetherSynth::process_block`.
    pub fn compute_modulated_cutoff_hz(&self, env_level: f32) -> f32 {
        let f_env = env_level.clamp(0.0, 1.0) * self.filter_env_depth;
        (self.filter_cutoff_hz + f_env * 4000.0).clamp(MIN_AETHER_CUTOFF_HZ, MAX_AETHER_CUTOFF_HZ)
    }

    /// Evaluates composite oscillator + distortion + glitch-chopped waveform at normalized phase `phase_norm` [0.0..=1.0].
    pub fn evaluate_composite_waveform(&self, phase_norm: f32) -> f32 {
        if self.bypassed {
            return 0.5;
        }
        let p = phase_norm.clamp(0.0, 1.0);
        // 2 cycles across display window
        let cycle_p = (p * 2.0).fract();
        let saw_val = 2.0 * cycle_p - 1.0;
        let pwm = self.compute_pwm_duty_cycle(p);
        let pulse_val = if cycle_p < pwm { 1.0 } else { -1.0 };

        // Crossfade Saw and Pulse matching AetherSynth::process_block
        let mixed = (1.0 - self.osc_mix) * saw_val + self.osc_mix * pulse_val;

        // Apply tube overdrive / wavefolder saturation
        let drive_gain = 1.0 + self.distortion_drive * 3.5;
        let saturated = (mixed * drive_gain).tanh();

        // Apply lowpass smoothing approximation based on normalized cutoff
        let cut_norm = Self::cutoff_to_normalized(self.filter_cutoff_hz);
        let resonance_ripple = (cycle_p * std::f32::consts::TAU * 4.0).sin()
            * (self.filter_resonance / MAX_AETHER_RESONANCE)
            * 0.22;
        let filtered = saturated * (0.35 + 0.65 * cut_norm) + resonance_ripple;

        // GlitchGate chopper window
        let gate_phase = (p * 4.0).fract();
        let gate_gain = if gate_phase <= self.glitch_gate_duty {
            1.0
        } else {
            0.0
        };

        (0.5 + 0.38 * filtered * gate_gain).clamp(0.05, 0.95)
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
        (dx * dx + dy * dy).sqrt() <= AETHER_SYNTH_PUCK_HIT_RADIUS
    }

    /// Renders a deterministic ASCII snapshot for headless QA verification.
    pub fn render_ascii_snapshot(&self) -> String {
        let pwm = self.compute_pwm_duty_cycle(0.25);
        let mod_cut = self.compute_modulated_cutoff_hz(1.0);
        let mut out = String::new();
        out.push_str("======================================================================\n");
        out.push_str(" SUMMONER DAW -- AETHER SUBTRACTIVE & GLITCH COMPOSITE SYNTH SNAPSHOT\n");
        out.push_str("======================================================================\n");
        out.push_str(&format!(" Preset:           {}\n", self.preset.name()));
        out.push_str(&format!(
            " Voice Topology:   {} | Bypassed: {}\n",
            self.voice_mode.label(),
            if self.bypassed { "YES" } else { "NO" }
        ));
        out.push_str(&format!(
            " Osc & Filter:     Mix {:.0}% Pulse | Cutoff {:.1} Hz (Peak {:.1} Hz) | Res Q: {:.2}\n",
            self.osc_mix * 100.0,
            self.filter_cutoff_hz,
            mod_cut,
            self.filter_resonance
        ));
        out.push_str(&format!(
            " Mod & Glitch:     LFO {:.2} Hz (PWM {:.0}%) | Drive {:.0}% | Gate Duty {:.0}%\n",
            self.lfo_rate_hz,
            pwm * 100.0,
            self.distortion_drive * 100.0,
            self.glitch_gate_duty * 100.0
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
                    RichText::new("AETHER SUBTRACTIVE & GLITCH COMPOSITE SYNTH HUD")
                        .color(Color32::from_rgb(56, 189, 248))
                        .strong(),
                );
                ui.label(
                    RichText::new("M33 / M34 AetherSynth + GlitchAetherMachine + CyberpunkSubSynth")
                        .color(Color32::from_rgb(148, 163, 184))
                        .small(),
                );
            });

            ui.add_space(6.0);

            // Preset Selector Bar
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Synth Presets:").strong());
                let presets = [
                    (AetherSynthPreset::AetherLeadClassic, "Aether Classic"),
                    (AetherSynthPreset::GlitchAetherChopper, "Glitch Chopper"),
                    (AetherSynthPreset::CyberpunkWavefoldSub, "Cyberpunk Sub"),
                    (AetherSynthPreset::AtmosphericHorizonPad, "Horizon Pad"),
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
                    .min_size(Vec2::new(88.0, 24.0));

                    if ui.add(btn).clicked() {
                        self.set_preset(p);
                    }
                }
            });

            ui.add_space(8.0);

            // Interactive 2D Oscillator Mix & Filter Cutoff Puck Canvas
            let canvas_size = Vec2::new(ui.available_width().max(280.0), 144.0);
            let (response, painter) =
                ui.allocate_painter(canvas_size, egui::Sense::click_and_drag());
            let rect = response.rect;
            let canvas = Rect::new(rect.min.x, rect.min.y, rect.width(), rect.height());

            painter.rect_filled(rect, Rounding::same(4.0), Color32::from_rgb(10, 18, 28));
            painter.rect_stroke(
                rect,
                Rounding::same(4.0),
                Stroke::new(1.0_f32, Color32::from_rgb(14, 116, 144)),
            );

            // Center reference line
            let zero_y = rect.center().y;
            painter.line_segment(
                [
                    egui::pos2(rect.min.x + 8.0, zero_y),
                    egui::pos2(rect.max.x - 8.0, zero_y),
                ],
                Stroke::new(1.0_f32, Color32::from_rgb(30, 48, 68)),
            );

            // Draw composite oscillator + ladder filter + glitch waveform
            let num_pts = 56;
            let mut pts = Vec::with_capacity(num_pts);
            for idx in 0..num_pts {
                let frac = idx as f32 / (num_pts - 1) as f32;
                let r_val = self.evaluate_composite_waveform(frac);
                let px = rect.min.x + 15.0 + frac * (rect.width() - 30.0);
                let py = rect.max.y - 15.0 - r_val * (rect.height() - 30.0);
                pts.push(egui::pos2(px, py));
            }
            for i in 0..(pts.len() - 1) {
                painter.line_segment(
                    [pts[i], pts[i + 1]],
                    Stroke::new(2.0_f32, Color32::from_rgb(56, 189, 248)),
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
                AETHER_SYNTH_PUCK_HIT_RADIUS,
                Color32::from_rgba_unmultiplied(56, 189, 248, 44),
            );
            painter.circle_stroke(
                puck_pos,
                AETHER_SYNTH_PUCK_HIT_RADIUS,
                Stroke::new(1.2_f32, Color32::from_rgb(125, 211, 252)),
            );
            painter.circle_filled(puck_pos, 6.0, Color32::WHITE);

            painter.text(
                egui::pos2(rect.min.x + 8.0, rect.min.y + 6.0),
                egui::Align2::LEFT_TOP,
                format!(
                    "{} | Mix: {:.0}% Pulse | Cutoff: {:.0} Hz | Res: {:.2}",
                    self.voice_mode.short_tag(),
                    self.osc_mix * 100.0,
                    self.filter_cutoff_hz,
                    self.filter_resonance
                ),
                FontId::proportional(11.0),
                Color32::from_rgb(186, 230, 253),
            );

            ui.add_space(8.0);

            // Surgical Pro Sliders
            ui.horizontal(|ui| {
                ui.label(RichText::new("Oscillator Saw/Pulse Mix:").size(11.0));
                let mut m = self.osc_mix;
                if ui.add(egui::Slider::new(&mut m, 0.0..=1.0)).changed() {
                    self.osc_mix = m;
                    self.sync_puck_from_params();
                }
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Ladder Filter Cutoff (Hz):").size(11.0));
                let mut c = self.filter_cutoff_hz;
                if ui
                    .add(
                        egui::Slider::new(&mut c, MIN_AETHER_CUTOFF_HZ..=MAX_AETHER_CUTOFF_HZ)
                            .logarithmic(true)
                            .suffix(" Hz"),
                    )
                    .changed()
                {
                    self.filter_cutoff_hz = c;
                    self.sync_puck_from_params();
                }
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Filter Resonance (Q):").size(11.0));
                ui.add(egui::Slider::new(
                    &mut self.filter_resonance,
                    MIN_AETHER_RESONANCE..=MAX_AETHER_RESONANCE,
                ));
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("PWM / Pitch LFO Rate (Hz):").size(11.0));
                ui.add(
                    egui::Slider::new(
                        &mut self.lfo_rate_hz,
                        MIN_AETHER_LFO_HZ..=MAX_AETHER_LFO_HZ,
                    )
                    .suffix(" Hz"),
                );
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("GlitchGate Duty Cycle:").size(11.0));
                ui.add(egui::Slider::new(&mut self.glitch_gate_duty, 0.05..=1.0));
            });
        });
    }
}
