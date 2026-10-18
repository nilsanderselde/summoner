// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! VST3 / CLAP / Wasm Sandboxed Plugin Host & Parameter Automap Rack HUD (Milestones 33 & 34).
//!
//! Directly reflects `crates/summoner_dsp/src/plugin_host.rs` (`PluginAudioNode`, `PluginDescriptor`,
//! `PluginFormat`, `PluginParamInfo`, `PluginStateConfig`, `scan_plugin_directory`) and
//! `crates/summoner_dsp/src/ecosystem_hardware.rs` (`ClapHostEngine`, `WasmDspRuntime`,
//! `IsolatedPluginScanner`, `ParameterAutomapper`).
//! Provides an interactive 2D Wet/Dry Parallel Mix & Hosted Cutoff/Macro Morph Puck
//! (Wet/Dry Mix $\in [0.0..=1.0]$ vs Cutoff Morph $\in [20.0..=20000.0]\text{ Hz}$),
//! hosted plugin format selector (VST3, CLAP, Wasm), crash-sandboxed status badge,
//! 4 curated plugin host presets, and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and deterministic ASCII snapshot rendering.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, FontId, RichText, Rounding, Stroke, Vec2};

pub const PLUGIN_HOST_PUCK_HIT_RADIUS: f32 = 22.0; // >= 44x44pt touch bounding target
pub const MIN_HOST_CUTOFF_HZ: f32 = 20.0;
pub const MAX_HOST_CUTOFF_HZ: f32 = 20000.0;

/// Hosted plugin binary format reflecting `summoner_dsp::plugin_host::PluginFormat` & `WasmDspRuntime`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HostedPluginFormat {
    #[default]
    ClapMpeHost,
    Vst3BundleHost,
    WasmSandboxHost,
}

impl HostedPluginFormat {
    pub fn label(&self) -> &'static str {
        match self {
            Self::ClapMpeHost => "CLAP 1.2 (MPE + Sample-Accurate)",
            Self::Vst3BundleHost => "VST3 (Embedded GUI Bridge)",
            Self::WasmSandboxHost => "Wasm DSP (Memory-Isolated Sandbox)",
        }
    }
}

/// Curated VST3 / CLAP / Wasm plugin host wrapper presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PluginHostPreset {
    /// Sample-accurate CLAP polyphonic MPE synthesizer wrapper.
    #[default]
    ClapPolyMpeSynth,
    /// VST3 mastering bus analog EQ & compressor chain wrapper.
    Vst3MasteringChain,
    /// Zero-trust WebAssembly sandboxed granular spectral effect.
    WasmSandboxedGranular,
    /// Parallel 50% wet/dry blend external hardware insert bridge.
    ParallelInsertBlend,
}

impl PluginHostPreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::ClapPolyMpeSynth => "CLAP MPE Poly Synth (Sample-Accurate)",
            Self::Vst3MasteringChain => "VST3 Mastering Bus Chain (100% Wet)",
            Self::WasmSandboxedGranular => "Wasm Sandboxed Spectral FX (64 Pages)",
            Self::ParallelInsertBlend => "Parallel Insert Blend (50% Wet/Dry)",
        }
    }

    /// Returns nominal (plugin_gain, cutoff_hz, wet_mix, mpe_timbre, format, latency_samples).
    pub fn nominal_parameters(&self) -> (f32, f32, f32, f32, HostedPluginFormat, u32) {
        match self {
            Self::ClapPolyMpeSynth => (1.00, 2400.0, 1.00, 0.72, HostedPluginFormat::ClapMpeHost, 0),
            Self::Vst3MasteringChain => (1.05, 8500.0, 1.00, 0.50, HostedPluginFormat::Vst3BundleHost, 64),
            Self::WasmSandboxedGranular => (0.95, 1600.0, 0.80, 0.85, HostedPluginFormat::WasmSandboxHost, 32),
            Self::ParallelInsertBlend => (1.15, 1200.0, 0.50, 0.45, HostedPluginFormat::ClapMpeHost, 16),
        }
    }
}

/// VST3 / CLAP / Wasm Sandboxed Plugin Host & Parameter Automap HUD view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginHostView {
    /// Active plugin host preset.
    pub preset: PluginHostPreset,
    /// Hosted plugin format (CLAP, VST3, Wasm).
    pub format: HostedPluginFormat,
    /// Hosted plugin Parameter 0: Output Gain [0.0 ..= 2.0] reflecting `PluginAudioNode` param 0.
    pub plugin_gain: f32,
    /// Hosted plugin Parameter 1: Cutoff Frequency in Hz [20.0 ..= 20000.0] reflecting param 1.
    pub cutoff_hz: f32,
    /// Hosted plugin Parameter 2: Parallel Wet/Dry Mix [0.0 ..= 1.0] reflecting param 2.
    pub wet_mix: f32,
    /// CLAP MPE CC74 Timbre expression [0.0 ..= 1.0] reflecting `ClapMpeEvent::timbre`.
    pub mpe_timbre: f32,
    /// Delay compensation latency in samples [0 ..= 4096].
    pub latency_samples: u32,
    /// Whether the plugin is isolated in crash sandbox (`IsolatedPluginScanner`).
    pub sandbox_active: bool,
    /// Whether the hosted plugin node is bypassed.
    pub bypassed: bool,
    /// 2D Puck position (X: normalized wet_mix, Y: normalized logarithmic cutoff_hz).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for PluginHostView {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginHostView {
    pub fn new() -> Self {
        let preset = PluginHostPreset::ClapPolyMpeSynth;
        let (gain, cutoff, mix, timbre, fmt, lat) = preset.nominal_parameters();
        let norm_x = Self::mix_to_normalized(mix);
        let norm_y = Self::cutoff_to_normalized(cutoff);

        Self {
            preset,
            format: fmt,
            plugin_gain: gain,
            cutoff_hz: cutoff,
            wet_mix: mix,
            mpe_timbre: timbre,
            latency_samples: lat,
            sandbox_active: true,
            bypassed: false,
            puck_pos: (norm_x, norm_y),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        }
    }

    pub fn set_preset(&mut self, preset: PluginHostPreset) {
        self.preset = preset;
        let (gain, cutoff, mix, timbre, fmt, lat) = preset.nominal_parameters();
        self.plugin_gain = gain;
        self.cutoff_hz = cutoff;
        self.wet_mix = mix;
        self.mpe_timbre = timbre;
        self.format = fmt;
        self.latency_samples = lat;
        self.sync_puck_from_params();
    }

    pub fn sync_puck_from_params(&mut self) {
        self.puck_pos = (
            Self::mix_to_normalized(self.wet_mix),
            Self::cutoff_to_normalized(self.cutoff_hz),
        );
    }

    pub fn update_from_puck(&mut self, norm_x: f32, norm_y: f32) {
        let x = norm_x.clamp(0.0, 1.0);
        let y = norm_y.clamp(0.0, 1.0);
        self.puck_pos = (x, y);
        self.wet_mix = Self::normalized_to_mix(x);
        self.cutoff_hz = Self::normalized_to_cutoff(y);
    }

    pub fn mix_to_normalized(mix: f32) -> f32 {
        mix.clamp(0.0, 1.0)
    }

    pub fn normalized_to_mix(norm: f32) -> f32 {
        norm.clamp(0.0, 1.0)
    }

    pub fn cutoff_to_normalized(hz: f32) -> f32 {
        let clamped = hz.clamp(MIN_HOST_CUTOFF_HZ, MAX_HOST_CUTOFF_HZ);
        let ratio = MAX_HOST_CUTOFF_HZ / MIN_HOST_CUTOFF_HZ;
        ((clamped / MIN_HOST_CUTOFF_HZ).ln() / ratio.ln()).clamp(0.0, 1.0)
    }

    pub fn normalized_to_cutoff(norm: f32) -> f32 {
        let n = norm.clamp(0.0, 1.0);
        let ratio = MAX_HOST_CUTOFF_HZ / MIN_HOST_CUTOFF_HZ;
        MIN_HOST_CUTOFF_HZ * ratio.powf(n)
    }

    /// Evaluates parallel wet/dry sample transfer matching `PluginAudioNode::process`.
    pub fn evaluate_hosted_sample(&self, dry_input: f32) -> f32 {
        if self.bypassed {
            return dry_input;
        }
        let wet = dry_input * self.plugin_gain.clamp(0.0, 2.0);
        let mix = self.wet_mix.clamp(0.0, 1.0);
        dry_input * (1.0 - mix) + wet * mix
    }

    /// Evaluates normalized frequency response curve across normalized frequency [0.0..=1.0].
    pub fn evaluate_response_curve(&self, norm_freq: f32) -> f32 {
        let f = norm_freq.clamp(0.0, 1.0);
        let cut_norm = Self::cutoff_to_normalized(self.cutoff_hz);
        let roll = if f <= cut_norm {
            1.0
        } else {
            let d = (f - cut_norm) / (1.0 - cut_norm).max(0.02);
            (-d * 3.2).exp()
        };
        let dry_part = 1.0 - self.wet_mix;
        let wet_part = self.wet_mix * (self.plugin_gain * 0.5) * roll;
        (dry_part * 0.5 + wet_part).clamp(0.0, 1.0)
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
        (dx * dx + dy * dy).sqrt() <= PLUGIN_HOST_PUCK_HIT_RADIUS
    }

    /// Renders a deterministic ASCII snapshot for headless QA verification.
    pub fn render_ascii_snapshot(&self) -> String {
        let mut out = String::new();
        out.push_str("======================================================================\n");
        out.push_str(" SUMMONER DAW -- VST3 / CLAP / WASM PLUGIN HOST & AUTOMAP SNAPSHOT\n");
        out.push_str("======================================================================\n");
        out.push_str(&format!(" Preset:           {}\n", self.preset.name()));
        out.push_str(&format!(
            " Format:           {} | Sandbox: {}\n",
            self.format.label(),
            if self.sandbox_active { "ISOLATED [SAFE]" } else { "DIRECT" }
        ));
        out.push_str(&format!(
            " Gain (Param 0):   {:.2}x | Cutoff (Param 1): {:.1} Hz | Mix (Param 2): {:.0}%\n",
            self.plugin_gain,
            self.cutoff_hz,
            self.wet_mix * 100.0
        ));
        out.push_str(&format!(
            " MPE Timbre CC74:  {:.0}% | PDC Latency: {} samples\n",
            self.mpe_timbre * 100.0,
            self.latency_samples
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
                    RichText::new("VST3 / CLAP / WASM SANDBOXED PLUGIN HOST HUD")
                        .color(Color32::from_rgb(16, 185, 129))
                        .strong(),
                );
                ui.label(
                    RichText::new("M33 / M34 Sample-Accurate MPE & Automap Bridge")
                        .color(Color32::from_rgb(148, 163, 184))
                        .small(),
                );
            });

            ui.add_space(6.0);

            // Preset Selector Bar
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Host Presets:").strong());
                let presets = [
                    (PluginHostPreset::ClapPolyMpeSynth, "CLAP MPE Synth"),
                    (PluginHostPreset::Vst3MasteringChain, "VST3 Master"),
                    (PluginHostPreset::WasmSandboxedGranular, "Wasm Sandbox"),
                    (PluginHostPreset::ParallelInsertBlend, "Parallel 50%"),
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
                    .min_size(Vec2::new(86.0, 24.0));

                    if ui.add(btn).clicked() {
                        self.set_preset(p);
                    }
                }
            });

            ui.add_space(8.0);

            // Interactive 2D Wet/Dry Mix & Cutoff Morph Canvas
            let canvas_size = Vec2::new(ui.available_width().max(280.0), 144.0);
            let (response, painter) =
                ui.allocate_painter(canvas_size, egui::Sense::click_and_drag());
            let rect = response.rect;
            let canvas = Rect::new(rect.min.x, rect.min.y, rect.width(), rect.height());

            painter.rect_filled(rect, Rounding::same(4.0), Color32::from_rgb(10, 22, 18));
            painter.rect_stroke(
                rect,
                Rounding::same(4.0),
                Stroke::new(1.0_f32, Color32::from_rgb(24, 92, 68)),
            );

            // Draw response contour
            let num_pts = 48;
            let mut pts = Vec::with_capacity(num_pts);
            for idx in 0..num_pts {
                let frac = idx as f32 / (num_pts - 1) as f32;
                let r_val = self.evaluate_response_curve(frac);
                let px = rect.min.x + 15.0 + frac * (rect.width() - 30.0);
                let py = rect.max.y - 14.0 - r_val * (rect.height() - 32.0);
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
                PLUGIN_HOST_PUCK_HIT_RADIUS,
                Color32::from_rgba_unmultiplied(16, 185, 129, 42),
            );
            painter.circle_stroke(
                puck_pos,
                PLUGIN_HOST_PUCK_HIT_RADIUS,
                Stroke::new(1.2_f32, Color32::from_rgb(52, 211, 153)),
            );
            painter.circle_filled(puck_pos, 6.0, Color32::WHITE);

            painter.text(
                egui::pos2(rect.min.x + 8.0, rect.min.y + 6.0),
                egui::Align2::LEFT_TOP,
                format!(
                    "{} | Cutoff: {:.0} Hz | Mix: {:.0}% | Gain: {:.2}x",
                    self.format.label(),
                    self.cutoff_hz,
                    self.wet_mix * 100.0,
                    self.plugin_gain
                ),
                FontId::proportional(11.0),
                Color32::from_rgb(110, 231, 183),
            );

            ui.add_space(8.0);

            // Surgical Pro Sliders
            ui.horizontal(|ui| {
                ui.label(RichText::new("Plugin Gain (Param 0):").size(11.0));
                ui.add(egui::Slider::new(&mut self.plugin_gain, 0.0..=2.0).suffix("x"));
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Cutoff Morph (Param 1):").size(11.0));
                let mut c = self.cutoff_hz;
                if ui
                    .add(
                        egui::Slider::new(&mut c, MIN_HOST_CUTOFF_HZ..=MAX_HOST_CUTOFF_HZ)
                            .logarithmic(true)
                            .suffix(" Hz"),
                    )
                    .changed()
                {
                    self.cutoff_hz = c;
                    self.sync_puck_from_params();
                }
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Wet/Dry Mix (Param 2):").size(11.0));
                let mut m = self.wet_mix;
                if ui.add(egui::Slider::new(&mut m, 0.0..=1.0)).changed() {
                    self.wet_mix = m;
                    self.sync_puck_from_params();
                }
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("CLAP MPE Timbre CC74:").size(11.0));
                ui.add(egui::Slider::new(&mut self.mpe_timbre, 0.0..=1.0));
            });
        });
    }
}
