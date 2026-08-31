// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Universal `DspNodeUi` trait, parameter reflection schema, and comprehensive DSP module registry.
//! Provides two-tier UX: Novice Macro Strip (Tone, Space, Punch, Character) & Pro Surgical Parameter Inspector.
//! Zero CLI left behind: Every DSP module across `summoner_dsp` and `summoner_core` is registered with tactile controls.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, FontId, RichText, Rounding, Stroke, Ui};

#[cfg(feature = "gui")]
use summoner_core::param_bus::{ParamBus, ParamId};

/// Category classification for DSP Nodes with visual color-coding and icons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DspNodeCategory {
    Oscillator,
    AcousticPhysicalModel,
    FilterEq,
    DynamicsMaster,
    DistortionSaturation,
    Modulation,
    TimeSpace,
    SpatialSurround,
    SpectralResynthesis,
    NeuralAi,
    SamplerSlicer,
    CompositeSynth,
    Utility,
}

pub type DspCategory = DspNodeCategory;

impl DspNodeCategory {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Oscillator => "Oscillators & Generators",
            Self::AcousticPhysicalModel => "Acoustic Physical Models",
            Self::FilterEq => "Filters & EQ",
            Self::DynamicsMaster => "Dynamics & Mastering",
            Self::DistortionSaturation => "Distortion & Color",
            Self::Modulation => "Modulators & Envelopes",
            Self::TimeSpace => "Time, Echo & Reverb",
            Self::SpatialSurround => "Spatial & 3D Audio",
            Self::SpectralResynthesis => "Spectral Processing",
            Self::NeuralAi => "Neural & AI DSP",
            Self::SamplerSlicer => "Samplers & Slicers",
            Self::CompositeSynth => "Composite Synthesizers",
            Self::Utility => "Routing & Utilities",
        }
    }

    pub fn display_label(&self) -> &'static str {
        self.name()
    }

    pub fn color_rgb(&self) -> (u8, u8, u8) {
        match self {
            Self::Oscillator => (56, 189, 248),         // Vibrant Cyan
            Self::AcousticPhysicalModel => (245, 158, 11), // Warm Amber
            Self::FilterEq => (168, 85, 247),           // Violet / Purple
            Self::DynamicsMaster => (239, 68, 68),       // Crimson / Coral
            Self::DistortionSaturation => (249, 115, 22),// Bright Orange
            Self::Modulation => (34, 197, 94),          // Emerald Green
            Self::TimeSpace => (99, 102, 241),          // Indigo Blue
            Self::SpatialSurround => (14, 165, 233),    // Sky Blue
            Self::SpectralResynthesis => (217, 70, 239),// Magenta / Pink
            Self::NeuralAi => (236, 72, 153),           // Neon Rose
            Self::SamplerSlicer => (16, 185, 129),      // Mint Green
            Self::CompositeSynth => (59, 130, 246),     // Electric Blue
            Self::Utility => (148, 163, 184),           // Slate Gray
        }
    }

    pub fn theme_color_rgb(&self) -> (u8, u8, u8) {
        self.color_rgb()
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Oscillator => "🌊",
            Self::AcousticPhysicalModel => "🎻",
            Self::FilterEq => "🎛️",
            Self::DynamicsMaster => "📊",
            Self::DistortionSaturation => "🔥",
            Self::Modulation => "🔄",
            Self::TimeSpace => "🌌",
            Self::SpatialSurround => "🌐",
            Self::SpectralResynthesis => "✨",
            Self::NeuralAi => "🧠",
            Self::SamplerSlicer => "✂️",
            Self::CompositeSynth => "🎹",
            Self::Utility => "🛠️",
        }
    }
}

/// Tactile widget representation for a parameter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DspWidgetKind {
    RotaryKnob {
        min: f32,
        max: f32,
        default: f32,
        step: Option<f32>,
        unit: String,
        is_logarithmic: bool,
    },
    VerticalFader {
        min: f32,
        max: f32,
        default: f32,
        unit: String,
    },
    Toggle {
        default: bool,
    },
    EnumChoice {
        choices: Vec<String>,
        default_idx: usize,
    },
    AdsrEnvelope {
        attack: f32,
        decay: f32,
        sustain: f32,
        release: f32,
    },
    FreqFilterCurve {
        cutoff_hz: f32,
        resonance: f32,
    },
    WavetablePreview {
        morph: f32,
    },
}

/// Macro role mapping for Novice view (Tone, Space, Punch, Character).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MacroRole {
    Tone,      // Controls brightness / frequency / cutoff
    Space,     // Controls reverb / delay / width / diffusion
    Punch,     // Controls dynamics / transient / drive / attack
    Character, // Controls morph / modulation / resonance / warmth
    None,
}

/// Schema definition for a single parameter on a DSP node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DspParamSchema {
    pub id: String,
    pub name: String,
    pub widget: DspWidgetKind,
    pub macro_role: MacroRole,
    pub tooltip: String,
}

impl DspParamSchema {
    pub fn knob(
        id: impl Into<String>,
        name: impl Into<String>,
        min: f32,
        max: f32,
        default: f32,
        unit: impl Into<String>,
        macro_role: MacroRole,
        tooltip: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            widget: DspWidgetKind::RotaryKnob {
                min,
                max,
                default,
                step: None,
                unit: unit.into(),
                is_logarithmic: false,
            },
            macro_role,
            tooltip: tooltip.into(),
        }
    }

    pub fn log_knob(
        id: impl Into<String>,
        name: impl Into<String>,
        min: f32,
        max: f32,
        default: f32,
        unit: impl Into<String>,
        macro_role: MacroRole,
        tooltip: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            widget: DspWidgetKind::RotaryKnob {
                min,
                max,
                default,
                step: None,
                unit: unit.into(),
                is_logarithmic: true,
            },
            macro_role,
            tooltip: tooltip.into(),
        }
    }

    pub fn fader(
        id: impl Into<String>,
        name: impl Into<String>,
        min: f32,
        max: f32,
        default: f32,
        unit: impl Into<String>,
        macro_role: MacroRole,
        tooltip: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            widget: DspWidgetKind::VerticalFader {
                min,
                max,
                default,
                unit: unit.into(),
            },
            macro_role,
            tooltip: tooltip.into(),
        }
    }

    pub fn toggle(
        id: impl Into<String>,
        name: impl Into<String>,
        default: bool,
        tooltip: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            widget: DspWidgetKind::Toggle { default },
            macro_role: MacroRole::None,
            tooltip: tooltip.into(),
        }
    }

    pub fn choice(
        id: impl Into<String>,
        name: impl Into<String>,
        choices: &[&str],
        default_idx: usize,
        tooltip: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            widget: DspWidgetKind::EnumChoice {
                choices: choices.iter().map(|s| s.to_string()).collect(),
                default_idx,
            },
            macro_role: MacroRole::None,
            tooltip: tooltip.into(),
        }
    }
}

/// Metadata descriptor for a DSP Node, detailing all its parameters and controls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DspNodeDescriptor {
    pub kind_id: String,
    pub display_name: String,
    pub category: DspNodeCategory,
    pub description: String,
    pub params: Vec<DspParamSchema>,
}

impl DspNodeDescriptor {
    pub fn new(
        kind_id: impl Into<String>,
        display_name: impl Into<String>,
        category: DspNodeCategory,
        description: impl Into<String>,
    ) -> Self {
        Self {
            kind_id: kind_id.into(),
            display_name: display_name.into(),
            category,
            description: description.into(),
            params: Vec::new(),
        }
    }

    pub fn with_param(mut self, param: DspParamSchema) -> Self {
        self.params.push(param);
        self
    }

    /// Construct a default project `NodeConfig` schema initialized with the descriptor's parameters.
    pub fn default_node_config(&self) -> summoner_project::schema::NodeConfig {
        let mut params = HashMap::new();
        for p in &self.params {
            let val = match &p.widget {
                DspWidgetKind::RotaryKnob { default, .. } => *default,
                DspWidgetKind::VerticalFader { default, .. } => *default,
                DspWidgetKind::Toggle { default } => {
                    if *default {
                        1.0
                    } else {
                        0.0
                    }
                }
                DspWidgetKind::EnumChoice { default_idx, .. } => *default_idx as f32,
                _ => 0.0,
            };
            params.insert(p.id.clone(), val);
        }
        summoner_project::schema::NodeConfig {
            kind: self.kind_id.clone(),
            params,
            plugin_state: None,
        }
    }

    #[cfg(feature = "gui")]
    pub fn to_dsp_rack_module(&self) -> crate::views::dsp_rack_dock::DspRackModule {
        let (r, g, b) = self.category.color_rgb();
        let mut module = crate::views::dsp_rack_dock::DspRackModule::new(
            &self.kind_id,
            &self.display_name,
            self.category.name(),
            (r, g, b),
        );
        for p in &self.params {
            let (default_val, unit) = match &p.widget {
                DspWidgetKind::RotaryKnob { default, unit, min, max, .. } => {
                    let norm = ((*default - *min) / (*max - *min).max(1e-5)).clamp(0.0, 1.0);
                    (norm, unit.clone())
                }
                DspWidgetKind::VerticalFader { default, unit, min, max } => {
                    let norm = ((*default - *min) / (*max - *min).max(1e-5)).clamp(0.0, 1.0);
                    (norm, unit.clone())
                }
                DspWidgetKind::Toggle { default } => {
                    (if *default { 1.0 } else { 0.0 }, "".to_string())
                }
                DspWidgetKind::EnumChoice { default_idx, choices } => {
                    let norm = if choices.is_empty() { 0.0 } else { *default_idx as f32 / choices.len() as f32 };
                    (norm, "".to_string())
                }
                _ => (0.5, "".to_string()),
            };
            module = module.with_param(crate::views::dsp_rack_dock::DspModuleParam::new(&p.name, default_val, &unit));
        }
        module
    }
}

/// Parameter value distribution and scaling representation for reflection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DspParamType {
    FloatLinear {
        min: f32,
        max: f32,
        default: f32,
        step: f32,
        unit: String,
    },
    FloatLogarithmic {
        min: f32,
        max: f32,
        default: f32,
        unit: String,
    },
    Enum {
        variants: Vec<String>,
        default_index: usize,
    },
    Bool {
        default: bool,
    },
    Integer {
        min: i32,
        max: i32,
        default: i32,
        unit: String,
    },
}

/// Metadata and real-time state for a reflected DSP parameter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DspParamDescriptor {
    pub id: String,
    pub name: String,
    pub param_type: DspParamType,
    pub current_value: f32,
    pub is_locked: bool,
    pub is_modulated: bool,
    pub color_rgb: (u8, u8, u8),
    pub description: String,
}

impl DspParamDescriptor {
    pub fn new_linear(
        id: impl Into<String>,
        name: impl Into<String>,
        min: f32,
        max: f32,
        default: f32,
        unit: impl Into<String>,
        color_rgb: (u8, u8, u8),
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            param_type: DspParamType::FloatLinear {
                min,
                max,
                default,
                step: (max - min) / 100.0,
                unit: unit.into(),
            },
            current_value: default,
            is_locked: false,
            is_modulated: false,
            color_rgb,
            description: String::new(),
        }
    }

    pub fn new_log(
        id: impl Into<String>,
        name: impl Into<String>,
        min: f32,
        max: f32,
        default: f32,
        unit: impl Into<String>,
        color_rgb: (u8, u8, u8),
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            param_type: DspParamType::FloatLogarithmic {
                min,
                max,
                default,
                unit: unit.into(),
            },
            current_value: default,
            is_locked: false,
            is_modulated: false,
            color_rgb,
            description: String::new(),
        }
    }

    pub fn new_bool(
        id: impl Into<String>,
        name: impl Into<String>,
        default: bool,
        color_rgb: (u8, u8, u8),
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            param_type: DspParamType::Bool { default },
            current_value: if default { 1.0 } else { 0.0 },
            is_locked: false,
            is_modulated: false,
            color_rgb,
            description: String::new(),
        }
    }

    pub fn new_enum(
        id: impl Into<String>,
        name: impl Into<String>,
        variants: Vec<String>,
        default_index: usize,
        color_rgb: (u8, u8, u8),
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            param_type: DspParamType::Enum {
                variants,
                default_index,
            },
            current_value: default_index as f32,
            is_locked: false,
            is_modulated: false,
            color_rgb,
            description: String::new(),
        }
    }

    pub fn new_int(
        id: impl Into<String>,
        name: impl Into<String>,
        min: i32,
        max: i32,
        default: i32,
        unit: impl Into<String>,
        color_rgb: (u8, u8, u8),
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            param_type: DspParamType::Integer {
                min,
                max,
                default,
                unit: unit.into(),
            },
            current_value: default as f32,
            is_locked: false,
            is_modulated: false,
            color_rgb,
            description: String::new(),
        }
    }

    pub fn format_value(&self) -> String {
        match &self.param_type {
            DspParamType::FloatLinear { unit, .. } | DspParamType::FloatLogarithmic { unit, .. } => {
                if unit.is_empty() {
                    format!("{:.2}", self.current_value)
                } else if unit == "%" {
                    format!("{:.0}%", self.current_value * 100.0)
                } else if unit == "Hz" {
                    if self.current_value >= 1000.0 {
                        format!("{:.2} kHz", self.current_value / 1000.0)
                    } else {
                        format!("{:.1} Hz", self.current_value)
                    }
                } else if unit == "dB" || unit == "dBFS" || unit == "dBTP" || unit == "LUFS" || unit == "dB/oct" {
                    format!("{:.1} {}", self.current_value, unit)
                } else if unit == "ms" {
                    format!("{:.1} ms", self.current_value)
                } else {
                    format!("{:.2} {}", self.current_value, unit)
                }
            }
            DspParamType::Enum { variants, .. } => {
                let idx = self.current_value.round() as usize;
                variants.get(idx).cloned().unwrap_or_else(|| "Unknown".to_string())
            }
            DspParamType::Bool { .. } => {
                if self.current_value >= 0.5 {
                    "ON".to_string()
                } else {
                    "OFF".to_string()
                }
            }
            DspParamType::Integer { unit, .. } => {
                if unit.is_empty() {
                    format!("{}", self.current_value.round() as i32)
                } else {
                    format!("{} {}", self.current_value.round() as i32, unit)
                }
            }
        }
    }

    pub fn set_normalized(&mut self, norm_val: f32) {
        if self.is_locked {
            return;
        }
        let clamped_norm = norm_val.clamp(0.0, 1.0);
        match &self.param_type {
            DspParamType::FloatLinear { min, max, .. } => {
                self.current_value = min + clamped_norm * (max - min);
            }
            DspParamType::FloatLogarithmic { min, max, .. } => {
                let min_log = min.max(1e-4).ln();
                let max_log = max.max(1e-4).ln();
                self.current_value = (min_log + clamped_norm * (max_log - min_log)).exp();
            }
            DspParamType::Enum { variants, .. } => {
                if !variants.is_empty() {
                    let idx = (clamped_norm * (variants.len() - 1) as f32).round() as usize;
                    self.current_value = idx as f32;
                }
            }
            DspParamType::Bool { .. } => {
                self.current_value = if clamped_norm >= 0.5 { 1.0 } else { 0.0 };
            }
            DspParamType::Integer { min, max, .. } => {
                self.current_value = (*min as f32 + clamped_norm * (*max - *min) as f32).round();
            }
        }
    }

    pub fn normalized_value(&self) -> f32 {
        match &self.param_type {
            DspParamType::FloatLinear { min, max, .. } => {
                if max > min {
                    ((self.current_value - min) / (max - min)).clamp(0.0, 1.0)
                } else {
                    0.0
                }
            }
            DspParamType::FloatLogarithmic { min, max, .. } => {
                let min_log = min.max(1e-4).ln();
                let max_log = max.max(1e-4).ln();
                if max_log > min_log {
                    let cur_log = self.current_value.max(1e-4).ln();
                    ((cur_log - min_log) / (max_log - min_log)).clamp(0.0, 1.0)
                } else {
                    0.0
                }
            }
            DspParamType::Enum { variants, .. } => {
                if variants.len() > 1 {
                    (self.current_value / (variants.len() - 1) as f32).clamp(0.0, 1.0)
                } else {
                    0.0
                }
            }
            DspParamType::Bool { .. } => {
                if self.current_value >= 0.5 {
                    1.0
                } else {
                    0.0
                }
            }
            DspParamType::Integer { min, max, .. } => {
                if max > min {
                    ((self.current_value - *min as f32) / (*max - *min) as f32).clamp(0.0, 1.0)
                } else {
                    0.0
                }
            }
        }
    }
}

/// Universal Trait for DSP Node UI Reflection and Rendering.
pub trait DspNodeUi: Send + Sync {
    /// Returns the node type name.
    fn node_type_name(&self) -> &'static str;

    /// Human-friendly display label.
    fn display_name(&self) -> &str;

    /// Category for rack organization and palette color coding.
    fn category(&self) -> DspNodeCategory;

    /// List of parameter descriptors for reflection.
    fn parameters(&self) -> &[DspParamDescriptor];

    /// Mutable list of parameter descriptors.
    fn parameters_mut(&mut self) -> &mut [DspParamDescriptor];

    /// Query parameter value by identifier.
    fn get_param_value(&self, id: &str) -> Option<f32> {
        self.parameters()
            .iter()
            .find(|p| p.id == id)
            .map(|p| p.current_value)
    }

    /// Mutate parameter value by identifier.
    fn set_param_value(&mut self, id: &str, value: f32) -> bool {
        if let Some(param) = self.parameters_mut().iter_mut().find(|p| p.id == id) {
            if !param.is_locked {
                param.current_value = value;
                return true;
            }
        }
        false
    }

    /// Novice view: Render macro header strip.
    #[cfg(feature = "gui")]
    fn render_macro_strip(&mut self, ui: &mut Ui) {
        let (r, g, b) = self.category().theme_color_rgb();
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(self.display_name())
                    .color(Color32::from_rgb(r, g, b))
                    .strong(),
            );
            ui.separator();
            for param in self.parameters_mut().iter_mut().take(4) {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new(&param.name).size(11.0));
                    let mut norm = param.normalized_value();
                    let slider = egui::Slider::new(&mut norm, 0.0..=1.0)
                        .show_value(false)
                        .text(param.format_value());
                    if ui.add_enabled(!param.is_locked, slider).changed() {
                        param.set_normalized(norm);
                    }
                });
            }
        });
    }

    /// Pro view: Deep surgical parameter inspector.
    #[cfg(feature = "gui")]
    fn render_deep_inspector(&mut self, ui: &mut Ui) {
        let (r, g, b) = self.category().theme_color_rgb();
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new(format!("{} Inspector", self.display_name()))
                        .color(Color32::from_rgb(r, g, b)),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(self.category().display_label())
                            .size(11.0)
                            .color(Color32::from_rgb(r, g, b)),
                    );
                });
            });
            ui.add_space(4.0);

            let node_name = self.node_type_name();
            egui::Grid::new(format!("grid_{}", node_name))
                .num_columns(5)
                .spacing([12.0, 8.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Param").strong());
                    ui.label(egui::RichText::new("Value").strong());
                    ui.label(egui::RichText::new("Control").strong());
                    ui.label(egui::RichText::new("Mod").strong());
                    ui.label(egui::RichText::new("Lock").strong());
                    ui.end_row();

                    for param in self.parameters_mut() {
                        ui.label(&param.name);
                        ui.label(egui::RichText::new(param.format_value()).color(Color32::from_rgb(
                            param.color_rgb.0,
                            param.color_rgb.1,
                            param.color_rgb.2,
                        )));

                        match &param.param_type {
                            DspParamType::Bool { .. } => {
                                let mut b = param.current_value >= 0.5;
                                if ui.add_enabled(!param.is_locked, egui::Checkbox::without_text(&mut b)).changed() {
                                    param.current_value = if b { 1.0 } else { 0.0 };
                                }
                            }
                            DspParamType::Enum { variants, .. } => {
                                let idx = param.current_value.round() as usize;
                                let mut selected = idx;
                                egui::ComboBox::from_id_source(format!("combo_{}_{}", node_name, param.id))
                                    .selected_text(variants.get(idx).map(|s| s.as_str()).unwrap_or("Select"))
                                    .show_ui(ui, |ui| {
                                        for (i, var) in variants.iter().enumerate() {
                                            ui.selectable_value(&mut selected, i, var);
                                        }
                                    });
                                if selected != idx && !param.is_locked {
                                    param.current_value = selected as f32;
                                }
                            }
                            _ => {
                                let mut norm = param.normalized_value();
                                let slider = egui::Slider::new(&mut norm, 0.0..=1.0).show_value(false);
                                if ui.add_enabled(!param.is_locked, slider).changed() {
                                    param.set_normalized(norm);
                                }
                            }
                        }

                        let mod_label = if param.is_modulated { "🟢" } else { "⚪" };
                        if ui.button(mod_label).on_hover_text("Toggle Modulation Routing").clicked() {
                            param.is_modulated = !param.is_modulated;
                        }

                        let lock_text = if param.is_locked { "🔒" } else { "🔓" };
                        if ui.button(lock_text).on_hover_text("Lock Parameter").clicked() {
                            param.is_locked = !param.is_locked;
                        }
                        ui.end_row();
                    }
                });
        });
    }

    /// Compact modular rack card representation.
    #[cfg(feature = "gui")]
    fn render_rack_card(&mut self, ui: &mut Ui, is_bypassed: &mut bool) {
        let (r, g, b) = self.category().theme_color_rgb();
        let stroke_color = Color32::from_rgb(r, g, b);
        let frame_bg = if *is_bypassed {
            Color32::from_rgb(18, 22, 30)
        } else {
            Color32::from_rgb(24, 30, 46)
        };

        egui::Frame::none()
            .fill(frame_bg)
            .stroke(eframe::egui::Stroke::new(1.5_f32, stroke_color))
            .rounding(6.0)
            .inner_margin(egui::Margin::same(8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("::")
                            .size(16.0)
                            .strong()
                            .color(Color32::from_rgb(160, 180, 210)),
                    );
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(self.display_name())
                                .size(14.0)
                                .strong()
                                .color(Color32::from_rgb(240, 245, 255)),
                        );
                        ui.label(
                            egui::RichText::new(self.category().display_label())
                                .size(10.0)
                                .color(stroke_color),
                        );
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let bypass_label = if *is_bypassed { "BYPASS" } else { "ACTIVE" };
                        let bypass_bg = if *is_bypassed {
                            Color32::from_rgb(60, 40, 45)
                        } else {
                            Color32::from_rgb(0, 180, 140)
                        };
                        let bypass_btn = egui::Button::new(
                            egui::RichText::new(bypass_label)
                                .size(12.0)
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(bypass_bg);

                        if ui.add(bypass_btn).clicked() {
                            *is_bypassed = !*is_bypassed;
                        }
                    });
                });

                if !*is_bypassed {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        for param in self.parameters_mut().iter_mut().take(3) {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(format!("{}:", param.name))
                                        .size(11.0)
                                        .color(Color32::from_rgb(180, 200, 225)),
                                );
                                let mut norm = param.normalized_value();
                                let slider = egui::Slider::new(&mut norm, 0.0..=1.0).show_value(false);
                                if ui.add_enabled(!param.is_locked, slider).changed() {
                                    param.set_normalized(norm);
                                }
                                ui.label(
                                    egui::RichText::new(param.format_value())
                                        .size(11.0)
                                        .color(stroke_color),
                                );
                            });
                            ui.add_space(8.0);
                        }
                    });
                }
            });
    }
}

/// Generic configurable DSP node UI model for standard nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericDspNodeUi {
    pub type_name: &'static str,
    pub title: String,
    pub node_category: DspNodeCategory,
    pub params: Vec<DspParamDescriptor>,
}

impl GenericDspNodeUi {
    pub fn new(type_name: &'static str, title: impl Into<String>, category: DspNodeCategory) -> Self {
        Self {
            type_name,
            title: title.into(),
            node_category: category,
            params: Vec::new(),
        }
    }

    pub fn with_param(mut self, param: DspParamDescriptor) -> Self {
        self.params.push(param);
        self
    }
}

impl DspNodeUi for GenericDspNodeUi {
    fn node_type_name(&self) -> &'static str {
        self.type_name
    }

    fn display_name(&self) -> &str {
        &self.title
    }

    fn category(&self) -> DspNodeCategory {
        self.node_category
    }

    fn parameters(&self) -> &[DspParamDescriptor] {
        &self.params
    }

    fn parameters_mut(&mut self) -> &mut [DspParamDescriptor] {
        &mut self.params
    }
}

#[cfg(feature = "gui")]
pub fn draw_tactile_dial(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    min: f32,
    max: f32,
    accent: Color32,
    tooltip: &str,
) {
    let size = egui::vec2(44.0, 54.0);
    let (mut resp, painter) = ui.allocate_painter(size, egui::Sense::click_and_drag());
    let rect = resp.rect;

    if resp.hovered() {
        resp = resp.on_hover_text(format!("{}\nValue: {:.2}", tooltip, *value));
    }

    if resp.dragged() {
        let delta = -resp.drag_delta().y * 0.008;
        let range = max - min;
        *value = (*value + delta * range).clamp(min, max);
    }

    let norm = ((*value - min) / (max - min).max(1e-5)).clamp(0.0, 1.0);
    let center = egui::pos2(rect.center().x, rect.top() + 20.0);
    let radius = 16.0;

    // Track arc
    painter.circle_filled(center, radius, Color32::from_rgb(10, 14, 24));
    painter.circle_stroke(
        center,
        radius,
        Stroke::new(2.5_f32, Color32::from_rgb(36, 50, 74)),
    );

    // Active fill arc indicator
    let start_angle = std::f32::consts::PI * 0.75;
    let total_angle = std::f32::consts::PI * 1.5;
    let end_angle = start_angle + total_angle * norm;
    let ind_pos = egui::pos2(
        center.x + (radius - 4.0) * end_angle.cos(),
        center.y + (radius - 4.0) * end_angle.sin(),
    );
    painter.line_segment([center, ind_pos], Stroke::new(2.0_f32, accent));
    painter.circle_filled(ind_pos, 2.5, accent);

    // Label below
    let short_label = if label.len() > 7 {
        &label[..7]
    } else {
        label
    };
    painter.text(
        egui::pos2(rect.center().x, rect.bottom() - 4.0),
        egui::Align2::CENTER_CENTER,
        short_label,
        FontId::proportional(9.0),
        Color32::from_rgb(203, 213, 225),
    );
}

#[cfg(feature = "gui")]
impl DspNodeDescriptor {
    pub fn render_macro_strip(
        &self,
        ui: &mut egui::Ui,
        param_values: &mut HashMap<String, f32>,
        param_bus: Option<&ParamBus>,
        track_id: u64,
        node_idx: usize,
    ) {
        let (r, g, b) = self.category.color_rgb();
        let cat_color = Color32::from_rgb(r, g, b);

        ui.horizontal(|ui| {
            let roles = [
                (MacroRole::Tone, "Tone", Color32::from_rgb(56, 189, 248)),
                (MacroRole::Space, "Space", Color32::from_rgb(99, 102, 241)),
                (MacroRole::Punch, "Punch", Color32::from_rgb(239, 68, 68)),
                (MacroRole::Character, "Character", cat_color),
            ];

            for (role, label, color) in roles {
                if let Some(param) = self.params.iter().find(|p| p.macro_role == role) {
                    let default_val = match &param.widget {
                        DspWidgetKind::RotaryKnob { default, .. } => *default,
                        DspWidgetKind::VerticalFader { default, .. } => *default,
                        _ => 0.5,
                    };
                    let val = param_values.entry(param.id.clone()).or_insert(default_val);
                    draw_tactile_dial(ui, label, val, 0.0, 1.0, color, &param.tooltip);
                    if let Some(bus) = param_bus {
                        let pid = ParamId(track_id as u32 * 1000 + node_idx as u32 * 20 + 1);
                        if bus.get(pid).is_some() {
                            bus.set(pid, *val);
                        }
                    }
                    ui.add_space(6.0);
                }
            }
        });
    }

    pub fn render_pro_inspector(
        &self,
        ui: &mut egui::Ui,
        param_values: &mut HashMap<String, f32>,
        param_bus: Option<&ParamBus>,
        track_id: u64,
        node_idx: usize,
    ) {
        let (r, g, b) = self.category.color_rgb();
        let cat_color = Color32::from_rgb(r, g, b);

        ui.vertical(|ui| {
            // Node Header
            ui.horizontal(|ui| {
                ui.label(RichText::new(self.category.icon()).font(FontId::proportional(14.0)));
                ui.label(
                    RichText::new(&self.display_name)
                        .font(FontId::proportional(12.0))
                        .strong()
                        .color(Color32::from_rgb(241, 245, 249)),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::Frame::none()
                        .fill(Color32::from_rgba_unmultiplied(r, g, b, 40))
                        .rounding(Rounding::same(4.0))
                        .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(self.category.name())
                                    .font(FontId::proportional(9.0))
                                    .color(cat_color),
                            );
                        });
                });
            });

            ui.add_space(2.0);
            ui.label(
                RichText::new(&self.description)
                    .font(FontId::proportional(10.0))
                    .color(Color32::from_rgb(148, 163, 184)),
            );
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);

            // Parameter Grid
            egui::Grid::new(format!("pro_grid_{}", self.kind_id))
                .num_columns(3)
                .spacing([12.0, 8.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label(RichText::new("Parameter").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(203, 213, 225)));
                    ui.label(RichText::new("Value").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(203, 213, 225)));
                    ui.label(RichText::new("Control").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(203, 213, 225)));
                    ui.end_row();

                    for (p_i, p) in self.params.iter().enumerate() {
                        ui.label(RichText::new(&p.name).font(FontId::proportional(10.0)).color(Color32::from_rgb(226, 232, 240)));

                        let default_val = match &p.widget {
                            DspWidgetKind::RotaryKnob { default, .. } => *default,
                            DspWidgetKind::VerticalFader { default, .. } => *default,
                            DspWidgetKind::Toggle { default } => if *default { 1.0 } else { 0.0 },
                            DspWidgetKind::EnumChoice { default_idx, .. } => *default_idx as f32,
                            _ => 0.0,
                        };
                        let val = param_values.entry(p.id.clone()).or_insert(default_val);

                        match &p.widget {
                            DspWidgetKind::RotaryKnob { min, max, unit, is_logarithmic, .. } => {
                                ui.label(RichText::new(format!("{:.2} {}", *val, unit)).font(FontId::proportional(10.0)).color(Color32::from_rgb(56, 189, 248)));
                                let slider = if *is_logarithmic {
                                    egui::Slider::new(val, *min..=*max).logarithmic(true).show_value(false)
                                } else {
                                    egui::Slider::new(val, *min..=*max).show_value(false)
                                };
                                ui.add(slider);
                            }
                            DspWidgetKind::VerticalFader { min, max, unit, .. } => {
                                ui.label(RichText::new(format!("{:.1} {}", *val, unit)).font(FontId::proportional(10.0)).color(Color32::from_rgb(56, 189, 248)));
                                ui.add(egui::Slider::new(val, *min..=*max).show_value(false));
                            }
                            DspWidgetKind::Toggle { .. } => {
                                let mut b = *val >= 0.5;
                                ui.label(RichText::new(if b { "ON" } else { "OFF" }).font(FontId::proportional(10.0)).color(if b { Color32::from_rgb(34, 197, 94) } else { Color32::from_rgb(148, 163, 184) }));
                                if ui.checkbox(&mut b, "").changed() {
                                    *val = if b { 1.0 } else { 0.0 };
                                }
                            }
                            DspWidgetKind::EnumChoice { choices, .. } => {
                                let idx = (*val as usize).min(choices.len().saturating_sub(1));
                                let current_str = choices.get(idx).cloned().unwrap_or_default();
                                ui.label(RichText::new(&current_str).font(FontId::proportional(10.0)).color(Color32::from_rgb(245, 158, 11)));
                                egui::ComboBox::from_id_source(format!("combo_{}_{}", self.kind_id, p.id))
                                    .selected_text(&current_str)
                                    .show_ui(ui, |ui| {
                                        for (c_i, choice) in choices.iter().enumerate() {
                                            ui.selectable_value(val, c_i as f32, choice);
                                        }
                                    });
                            }
                            _ => {
                                ui.label("-");
                                ui.label("-");
                            }
                        }

                        if let Some(bus) = param_bus {
                            let pid = ParamId(track_id as u32 * 1000 + node_idx as u32 * 20 + p_i as u32);
                            if bus.get(pid).is_some() {
                                bus.set(pid, *val);
                            }
                        }

                        ui.end_row();
                    }
                });
        });
    }

    pub fn render_rack_unit(
        &self,
        ui: &mut egui::Ui,
        param_values: &mut HashMap<String, f32>,
        is_bypassed: &mut bool,
        is_collapsed: &mut bool,
        param_bus: Option<&ParamBus>,
        track_id: u64,
        node_idx: usize,
    ) {
        let (r, g, b) = self.category.color_rgb();
        let stroke_color = Color32::from_rgb(r, g, b);
        let frame_bg = if *is_bypassed {
            Color32::from_rgb(15, 23, 42)
        } else {
            Color32::from_rgb(18, 26, 44)
        };

        egui::Frame::none()
            .fill(frame_bg)
            .stroke(Stroke::new(1.0_f32, stroke_color))
            .rounding(Rounding::same(6.0))
            .inner_margin(egui::Margin::symmetric(10.0, 8.0))
            .show(ui, |ui| {
                // Header Bar
                ui.horizontal(|ui| {
                    let collapse_icon = if *is_collapsed { "▶" } else { "▼" };
                    if ui.button(RichText::new(collapse_icon).font(FontId::proportional(10.0))).clicked() {
                        *is_collapsed = !*is_collapsed;
                    }

                    ui.label(RichText::new(self.category.icon()).font(FontId::proportional(13.0)));
                    ui.label(
                        RichText::new(&self.display_name)
                            .font(FontId::proportional(11.0))
                            .strong()
                            .color(if *is_bypassed { Color32::from_rgb(100, 116, 139) } else { Color32::from_rgb(241, 245, 249) }),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let bypass_color = if *is_bypassed {
                            Color32::from_rgb(239, 68, 68)
                        } else {
                            Color32::from_rgb(34, 197, 94)
                        };
                        let bypass_text = if *is_bypassed { "BYP" } else { "ON" };
                        if ui.button(RichText::new(bypass_text).font(FontId::proportional(9.0)).color(bypass_color)).clicked() {
                            *is_bypassed = !*is_bypassed;
                        }
                    });
                });

                if !*is_collapsed && !*is_bypassed {
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        for (p_i, p) in self.params.iter().take(4).enumerate() {
                            let default_val = match &p.widget {
                                DspWidgetKind::RotaryKnob { default, .. } => *default,
                                DspWidgetKind::VerticalFader { default, .. } => *default,
                                _ => 0.5,
                            };
                            let val = param_values.entry(p.id.clone()).or_insert(default_val);

                            let (min, max) = match &p.widget {
                                DspWidgetKind::RotaryKnob { min, max, .. } => (*min, *max),
                                DspWidgetKind::VerticalFader { min, max, .. } => (*min, *max),
                                _ => (0.0, 1.0),
                            };

                            draw_tactile_dial(ui, &p.name, val, min, max, stroke_color, &p.tooltip);

                            if let Some(bus) = param_bus {
                                let pid = ParamId(
                                    track_id as u32 * 1000 + node_idx as u32 * 20 + p_i as u32,
                                );
                                if bus.get(pid).is_some() {
                                    bus.set(pid, *val);
                                }
                            }
                            ui.add_space(4.0);
                        }
                    });
                }
            });
    }
}


/// Central DSP Node Registry indexing every DSP module in Summoner DAW.
pub struct DspNodeRegistry {
    descriptors: HashMap<String, DspNodeDescriptor>,
}

impl Default for DspNodeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl DspNodeRegistry {
    /// Constructs and indexes all DSP nodes into the registry.
    pub fn new() -> Self {
        let mut descriptors = HashMap::new();

        descriptors.insert("OscSine".to_string(), DspNodeDescriptor::new("OscSine", "Pure Sine Wave Oscillator", DspNodeCategory::Oscillator, "Pure Sine Wave Oscillator")
            .with_param(DspParamSchema::log_knob("freq", "Frequency", 20.0, 20000.0, 440.0, "Hz", MacroRole::Tone, "Frequency logarithmic parameter"))
            .with_param(DspParamSchema::knob("phase", "Phase Offset", 0.0, 360.0, 0.0, "deg", MacroRole::Tone, "Phase Offset parameter"))
            .with_param(DspParamSchema::knob("gain", "Output Gain", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Output Gain parameter"))
        );
        descriptors.insert("OscSaw".to_string(), DspNodeDescriptor::new("OscSaw", "Band-limited Sawtooth Oscillator", DspNodeCategory::Oscillator, "Band-limited Sawtooth Oscillator")
            .with_param(DspParamSchema::log_knob("freq", "Frequency", 20.0, 20000.0, 440.0, "Hz", MacroRole::Tone, "Frequency logarithmic parameter"))
            .with_param(DspParamSchema::knob("phase", "Phase Offset", 0.0, 360.0, 0.0, "deg", MacroRole::Tone, "Phase Offset parameter"))
            .with_param(DspParamSchema::knob("gain", "Output Gain", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Output Gain parameter"))
            .with_param(DspParamSchema::knob("bandlimit", "Bandlimit Order", 1.0, 8.0, 4.0, "", MacroRole::Character, "Bandlimit Order integer control"))
        );
        descriptors.insert("OscPulse".to_string(), DspNodeDescriptor::new("OscPulse", "Pulse Width Modulation Oscillator", DspNodeCategory::Oscillator, "Pulse Width Modulation Oscillator")
            .with_param(DspParamSchema::log_knob("freq", "Frequency", 20.0, 20000.0, 440.0, "Hz", MacroRole::Tone, "Frequency logarithmic parameter"))
            .with_param(DspParamSchema::knob("pulse_width", "Pulse Width", 0.01, 0.99, 0.5, "%", MacroRole::Tone, "Pulse Width parameter"))
            .with_param(DspParamSchema::knob("pwm_rate", "PWM LFO Rate", 0.1, 20.0, 1.0, "Hz", MacroRole::Tone, "PWM LFO Rate parameter"))
            .with_param(DspParamSchema::knob("gain", "Output Gain", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Output Gain parameter"))
        );
        descriptors.insert("OscTriangle".to_string(), DspNodeDescriptor::new("OscTriangle", "Anti-aliased Triangle Generator", DspNodeCategory::Oscillator, "Anti-aliased Triangle Generator")
            .with_param(DspParamSchema::log_knob("freq", "Frequency", 20.0, 20000.0, 440.0, "Hz", MacroRole::Tone, "Frequency logarithmic parameter"))
            .with_param(DspParamSchema::knob("symmetry", "Symmetry Ramp", 0.01, 0.99, 0.5, "%", MacroRole::Tone, "Symmetry Ramp parameter"))
            .with_param(DspParamSchema::knob("gain", "Output Gain", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Output Gain parameter"))
        );
        descriptors.insert("OscWavetable".to_string(), DspNodeDescriptor::new("OscWavetable", "SIMD Wavetable Morphing Synthesizer", DspNodeCategory::Oscillator, "SIMD Wavetable Morphing Synthesizer")
            .with_param(DspParamSchema::log_knob("freq", "Frequency", 20.0, 20000.0, 440.0, "Hz", MacroRole::Tone, "Frequency logarithmic parameter"))
            .with_param(DspParamSchema::knob("morph_pos", "Morph Position", 0.0, 1.0, 0.0, "%", MacroRole::Tone, "Morph Position parameter"))
            .with_param(DspParamSchema::choice("table", "Table Bank", &["Basic Shapes", "Harmonics", "Formants", "Metallic"], 0, "Table Bank mode selector"))
            .with_param(DspParamSchema::knob("gain", "Output Gain", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Output Gain parameter"))
        );
        descriptors.insert("SimdPolyWavetableOscillator".to_string(), DspNodeDescriptor::new("SimdPolyWavetableOscillator", "AVX2/NEON Polyphonic Wavetable Voice Bank", DspNodeCategory::Oscillator, "AVX2/NEON Polyphonic Wavetable Voice Bank")
            .with_param(DspParamSchema::knob("morph_pos", "Morph Position", 0.0, 1.0, 0.0, "%", MacroRole::Tone, "Morph Position parameter"))
            .with_param(DspParamSchema::knob("unison_detune", "Unison Detune", 0.0, 50.0, 8.0, "cents", MacroRole::Tone, "Unison Detune parameter"))
            .with_param(DspParamSchema::knob("unison_voices", "Voice Count", 1.0, 16.0, 4.0, "voices", MacroRole::Character, "Voice Count integer control"))
            .with_param(DspParamSchema::knob("sub_osc_gain", "Sub Osc Level", 0.0, 1.0, 0.3, "%", MacroRole::Tone, "Sub Osc Level parameter"))
            .with_param(DspParamSchema::knob("stereo_spread", "Stereo Spread", 0.0, 1.0, 0.7, "%", MacroRole::Tone, "Stereo Spread parameter"))
        );
        descriptors.insert("GranularSynthNode".to_string(), DspNodeDescriptor::new("GranularSynthNode", "Cloud Granular Audio Streamer", DspNodeCategory::Oscillator, "Cloud Granular Audio Streamer")
            .with_param(DspParamSchema::knob("density", "Grain Density", 1.0, 100.0, 25.0, "grains/s", MacroRole::Tone, "Grain Density parameter"))
            .with_param(DspParamSchema::knob("spray", "Position Spray", 0.0, 1.0, 0.15, "%", MacroRole::Tone, "Position Spray parameter"))
            .with_param(DspParamSchema::knob("pitch_ratio", "Pitch Ratio", 0.25, 4.0, 1.0, "x", MacroRole::Tone, "Pitch Ratio parameter"))
            .with_param(DspParamSchema::knob("grain_size", "Grain Size", 0.01, 0.5, 0.06, "s", MacroRole::Tone, "Grain Size parameter"))
            .with_param(DspParamSchema::knob("reverse_prob", "Reverse Probability", 0.0, 1.0, 0.0, "%", MacroRole::Tone, "Reverse Probability parameter"))
        );
        descriptors.insert("NeuralWavetable".to_string(), DspNodeDescriptor::new("NeuralWavetable", "Latent AI Neural Wavetable Synthesizer", DspNodeCategory::NeuralAi, "Latent AI Neural Wavetable Synthesizer")
            .with_param(DspParamSchema::knob("latent_x", "Latent Timbre X", -3.0, 3.0, 0.0, "", MacroRole::Tone, "Latent Timbre X parameter"))
            .with_param(DspParamSchema::knob("latent_y", "Latent Timbre Y", -3.0, 3.0, 0.0, "", MacroRole::Tone, "Latent Timbre Y parameter"))
            .with_param(DspParamSchema::knob("spectral_tilt", "Spectral Tilt", -12.0, 12.0, 0.0, "dB", MacroRole::Tone, "Spectral Tilt parameter"))
            .with_param(DspParamSchema::knob("morph_speed", "Morph Slew Rate", 0.1, 10.0, 1.0, "Hz", MacroRole::Tone, "Morph Slew Rate parameter"))
        );
        descriptors.insert("PlasmaArcSynthesizer".to_string(), DspNodeDescriptor::new("PlasmaArcSynthesizer", "High-Voltage Plasma Arc Audio Generator", DspNodeCategory::Oscillator, "High-Voltage Plasma Arc Audio Generator")
            .with_param(DspParamSchema::log_knob("arc_voltage", "Arc Voltage", 100.0, 5000.0, 1500.0, "V", MacroRole::Tone, "Arc Voltage logarithmic parameter"))
            .with_param(DspParamSchema::knob("plasma_density", "Plasma Density", 0.1, 1.0, 0.65, "%", MacroRole::Tone, "Plasma Density parameter"))
            .with_param(DspParamSchema::knob("ionization", "Ionization Rate", 10.0, 500.0, 120.0, "kHz", MacroRole::Tone, "Ionization Rate parameter"))
            .with_param(DspParamSchema::knob("hiss_level", "Sub-thermal Hiss", 0.0, 1.0, 0.1, "%", MacroRole::Tone, "Sub-thermal Hiss parameter"))
        );
        descriptors.insert("NoiseGen".to_string(), DspNodeDescriptor::new("NoiseGen", "Multi-Color Spectral Noise Generator", DspNodeCategory::Oscillator, "Multi-Color Spectral Noise Generator")
            .with_param(DspParamSchema::choice("noise_type", "Noise Color", &["White", "Pink (1/f)", "Brownian", "Blue"], 0, "Noise Color mode selector"))
            .with_param(DspParamSchema::log_knob("cutoff", "Filter Cutoff", 20.0, 20000.0, 12000.0, "Hz", MacroRole::Tone, "Filter Cutoff logarithmic parameter"))
            .with_param(DspParamSchema::knob("gain", "Output Gain", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Output Gain parameter"))
        );
        descriptors.insert("AetherSynth".to_string(), DspNodeDescriptor::new("AetherSynth", "Dual Saw/Pulse Subtractive Synth with Moog Ladder Filter", DspNodeCategory::CompositeSynth, "Dual Saw/Pulse Subtractive Synth with Moog Ladder Filter")
            .with_param(DspParamSchema::knob("osc_mix", "Saw / Pulse Mix", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Saw / Pulse Mix parameter"))
            .with_param(DspParamSchema::log_knob("filter_cutoff", "Moog Filter Cutoff", 20.0, 20000.0, 1200.0, "Hz", MacroRole::Tone, "Moog Filter Cutoff logarithmic parameter"))
            .with_param(DspParamSchema::knob("filter_res", "Ladder Resonance", 0.1, 4.0, 1.0, "Q", MacroRole::Tone, "Ladder Resonance parameter"))
            .with_param(DspParamSchema::log_knob("lfo_speed", "PWM LFO Speed", 0.1, 20.0, 2.0, "Hz", MacroRole::Tone, "PWM LFO Speed logarithmic parameter"))
        );
        descriptors.insert("PluckSynth".to_string(), DspNodeDescriptor::new("PluckSynth", "Karplus-Strong Plucked String Synthesis Core", DspNodeCategory::Oscillator, "Karplus-Strong Plucked String Synthesis Core")
            .with_param(DspParamSchema::knob("damping", "String Damping Cutoff", 100.0, 10000.0, 3500.0, "Hz", MacroRole::Tone, "String Damping Cutoff parameter"))
            .with_param(DspParamSchema::knob("tension", "Feedback Loop Tension", 0.5, 0.999, 0.98, "%", MacroRole::Tone, "Feedback Loop Tension parameter"))
            .with_param(DspParamSchema::knob("decay", "Exciter Burst Decay", 0.001, 0.1, 0.015, "s", MacroRole::Tone, "Exciter Burst Decay parameter"))
            .with_param(DspParamSchema::knob("brightness", "Noise Exciter Brightness", 0.0, 1.0, 0.7, "%", MacroRole::Tone, "Noise Exciter Brightness parameter"))
        );
        descriptors.insert("CyberpunkSubSynth".to_string(), DspNodeDescriptor::new("CyberpunkSubSynth", "Heavy Sub-bass Synth with Asymmetric Wavefolding", DspNodeCategory::CompositeSynth, "Heavy Sub-bass Synth with Asymmetric Wavefolding")
            .with_param(DspParamSchema::log_knob("sub_freq", "Sub Oscillator Freq", 20.0, 200.0, 55.0, "Hz", MacroRole::Tone, "Sub Oscillator Freq logarithmic parameter"))
            .with_param(DspParamSchema::knob("drive_gain", "Wavefolder Drive", 1.0, 10.0, 4.0, "x", MacroRole::Tone, "Wavefolder Drive parameter"))
            .with_param(DspParamSchema::log_knob("filter_cutoff", "SVF Filter Cutoff", 40.0, 2000.0, 600.0, "Hz", MacroRole::Tone, "SVF Filter Cutoff logarithmic parameter"))
            .with_param(DspParamSchema::knob("pitch_wobble", "LFO Pitch Wobble", 0.0, 1.0, 0.3, "%", MacroRole::Tone, "LFO Pitch Wobble parameter"))
        );
        descriptors.insert("AtmosphericPadSynth".to_string(), DspNodeDescriptor::new("AtmosphericPadSynth", "Detuned Ambient Pad with SVF Filter & Space Reverb", DspNodeCategory::CompositeSynth, "Detuned Ambient Pad with SVF Filter & Space Reverb")
            .with_param(DspParamSchema::knob("saw_tri_mix", "Saw / Detuned Tri Mix", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Saw / Detuned Tri Mix parameter"))
            .with_param(DspParamSchema::log_knob("filter_cutoff", "SVF Filter Cutoff", 100.0, 5000.0, 800.0, "Hz", MacroRole::Tone, "SVF Filter Cutoff logarithmic parameter"))
            .with_param(DspParamSchema::knob("reverb_mix", "Space Reverb Level", 0.0, 1.0, 0.4, "%", MacroRole::Tone, "Space Reverb Level parameter"))
            .with_param(DspParamSchema::knob("delay_mix", "Stereo Delay Level", 0.0, 1.0, 0.3, "%", MacroRole::Tone, "Stereo Delay Level parameter"))
        );
        descriptors.insert("GlitchPercussionSynth".to_string(), DspNodeDescriptor::new("GlitchPercussionSynth", "Noise Burst & Granular Stutter Percussion Engine", DspNodeCategory::CompositeSynth, "Noise Burst & Granular Stutter Percussion Engine")
            .with_param(DspParamSchema::log_knob("pitch_freq", "Pitch Drop Center", 30.0, 800.0, 120.0, "Hz", MacroRole::Tone, "Pitch Drop Center logarithmic parameter"))
            .with_param(DspParamSchema::knob("bitcrush_depth", "Bitcrusher Depth", 1.0, 16.0, 8.0, "bits", MacroRole::Character, "Bitcrusher Depth integer control"))
            .with_param(DspParamSchema::log_knob("comb_freq", "Comb Resonator Freq", 50.0, 2000.0, 240.0, "Hz", MacroRole::Tone, "Comb Resonator Freq logarithmic parameter"))
            .with_param(DspParamSchema::knob("stutter_rate", "Buffer Stutter Speed", 1.0, 64.0, 16.0, "div", MacroRole::Tone, "Buffer Stutter Speed parameter"))
        );
        descriptors.insert("GlitchAetherMachine".to_string(), DspNodeDescriptor::new("GlitchAetherMachine", "Flagship Chopper & Tape-Stop Glitch Synthesizer", DspNodeCategory::CompositeSynth, "Flagship Chopper & Tape-Stop Glitch Synthesizer")
            .with_param(DspParamSchema::knob("drive", "Tube Drive Saturation", 1.0, 20.0, 3.0, "x", MacroRole::Tone, "Tube Drive Saturation parameter"))
            .with_param(DspParamSchema::log_knob("filter_cutoff", "Moog Ladder Cutoff", 50.0, 10000.0, 2500.0, "Hz", MacroRole::Tone, "Moog Ladder Cutoff logarithmic parameter"))
            .with_param(DspParamSchema::knob("chop_rate", "Glitch Chopper Rate", 1.0, 32.0, 8.0, "Hz", MacroRole::Tone, "Glitch Chopper Rate parameter"))
            .with_param(DspParamSchema::toggle("tape_stop", "Tape Stop Effect Active", false, "Tape Stop Effect Active toggle switch"))
        );
        descriptors.insert("FmOperatorPair".to_string(), DspNodeDescriptor::new("FmOperatorPair", "2-Operator Phase Modulation FM Synthesizer", DspNodeCategory::Oscillator, "2-Operator Phase Modulation FM Synthesizer")
            .with_param(DspParamSchema::log_knob("carrier_freq", "Carrier Frequency", 20.0, 20000.0, 440.0, "Hz", MacroRole::Tone, "Carrier Frequency logarithmic parameter"))
            .with_param(DspParamSchema::knob("mod_ratio", "Modulator Ratio", 0.25, 16.0, 2.0, "x", MacroRole::Tone, "Modulator Ratio parameter"))
            .with_param(DspParamSchema::knob("fm_depth", "FM Modulation Depth", 0.0, 10.0, 2.0, "idx", MacroRole::Tone, "FM Modulation Depth parameter"))
            .with_param(DspParamSchema::log_knob("mod_decay", "Modulator Env Decay", 10.0, 2000.0, 300.0, "ms", MacroRole::Tone, "Modulator Env Decay logarithmic parameter"))
        );
        descriptors.insert("FmMatrixSynthesizer".to_string(), DspNodeDescriptor::new("FmMatrixSynthesizer", "6-Operator DX-Style FM Matrix Synthesizer", DspNodeCategory::CompositeSynth, "6-Operator DX-Style FM Matrix Synthesizer")
            .with_param(DspParamSchema::knob("mod_index", "FM Modulation Index", 0.0, 10.0, 3.5, "idx", MacroRole::Tone, "FM Modulation Index parameter"))
            .with_param(DspParamSchema::choice("algorithm", "FM Routing Algorithm", &["Alg 1 (Serial 6-Op)", "Alg 5 (Parallel Pairs)", "Alg 16 (3-Stack Hybrid)", "Alg 32 (All Parallel)"], 0, "FM Routing Algorithm mode selector"))
            .with_param(DspParamSchema::knob("feedback", "Op 6 Self-Feedback", 0.0, 1.0, 0.45, "%", MacroRole::Tone, "Op 6 Self-Feedback parameter"))
            .with_param(DspParamSchema::knob("master_level", "Master Output Level", 0.0, 1.0, 0.85, "%", MacroRole::Tone, "Master Output Level parameter"))
        );
        descriptors.insert("AnalogDrumVoice".to_string(), DspNodeDescriptor::new("AnalogDrumVoice", "808/909 Style Analog Synthesized Drum Voice", DspNodeCategory::Oscillator, "808/909 Style Analog Synthesized Drum Voice")
            .with_param(DspParamSchema::log_knob("tune", "Fundamental Pitch", 30.0, 500.0, 60.0, "Hz", MacroRole::Tone, "Fundamental Pitch logarithmic parameter"))
            .with_param(DspParamSchema::log_knob("decay", "Amp Envelope Decay", 10.0, 1500.0, 250.0, "ms", MacroRole::Tone, "Amp Envelope Decay logarithmic parameter"))
            .with_param(DspParamSchema::knob("snap", "Click / Pitch Snap", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Click / Pitch Snap parameter"))
            .with_param(DspParamSchema::knob("punch", "Transient Punch", 0.0, 1.0, 0.7, "%", MacroRole::Tone, "Transient Punch parameter"))
        );
        descriptors.insert("DrumMachineDevice".to_string(), DspNodeDescriptor::new("DrumMachineDevice", "24-Pad Zero-Allocation Sampling Drum Machine", DspNodeCategory::SamplerSlicer, "24-Pad Zero-Allocation Sampling Drum Machine")
            .with_param(DspParamSchema::knob("master_volume", "Master Kit Volume", 0.0, 1.0, 0.85, "%", MacroRole::Tone, "Master Kit Volume parameter"))
            .with_param(DspParamSchema::knob("pitch_offset", "Kit Pitch Offset", -12.0, 12.0, 0.0, "st", MacroRole::Tone, "Kit Pitch Offset parameter"))
            .with_param(DspParamSchema::knob("pad_decay", "Global Pad Decay Scale", 0.1, 2.0, 1.0, "x", MacroRole::Tone, "Global Pad Decay Scale parameter"))
            .with_param(DspParamSchema::knob("dynamic_punch", "Master Dynamic Punch", 0.0, 1.0, 0.6, "%", MacroRole::Tone, "Master Dynamic Punch parameter"))
        );
        descriptors.insert("QuantumStateVectorOscillator".to_string(), DspNodeDescriptor::new("QuantumStateVectorOscillator", "Qubit Phase Superposition & Hadamard Gate Synth", DspNodeCategory::Oscillator, "Qubit Phase Superposition & Hadamard Gate Synth")
            .with_param(DspParamSchema::log_knob("freq", "Superposition Frequency", 20.0, 20000.0, 440.0, "Hz", MacroRole::Tone, "Superposition Frequency logarithmic parameter"))
            .with_param(DspParamSchema::knob("phase_alpha", "Qubit Alpha Phase", 0.0, 360.0, 45.0, "deg", MacroRole::Tone, "Qubit Alpha Phase parameter"))
            .with_param(DspParamSchema::knob("phase_beta", "Qubit Beta Phase", 0.0, 360.0, 45.0, "deg", MacroRole::Tone, "Qubit Beta Phase parameter"))
            .with_param(DspParamSchema::knob("hadamard_mix", "Hadamard Transformation", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Hadamard Transformation parameter"))
            .with_param(DspParamSchema::toggle("pauli_x", "Pauli-X State Flip", false, "Pauli-X State Flip toggle switch"))
        );
        descriptors.insert("QuantumHarmonicOscillatorVoice".to_string(), DspNodeDescriptor::new("QuantumHarmonicOscillatorVoice", "Quantum Harmonic Oscillator Voice Bank", DspNodeCategory::Oscillator, "Quantum Harmonic Oscillator Voice Bank")
            .with_param(DspParamSchema::knob("energy_level", "Quantum State Level (n)", 0.0, 10.0, 2.0, "n", MacroRole::Character, "Quantum State Level (n) integer control"))
            .with_param(DspParamSchema::knob("wavefunction_sigma", "Wavefunction Spatial Width", 0.1, 5.0, 1.2, "sigma", MacroRole::Tone, "Wavefunction Spatial Width parameter"))
            .with_param(DspParamSchema::knob("potential_well", "Harmonic Well Depth", 0.1, 10.0, 4.0, "eV", MacroRole::Tone, "Harmonic Well Depth parameter"))
            .with_param(DspParamSchema::knob("output_gain", "Voice Output Gain", 0.0, 1.0, 0.85, "%", MacroRole::Tone, "Voice Output Gain parameter"))
        );
        descriptors.insert("NavierStokesFluidNode".to_string(), DspNodeDescriptor::new("NavierStokesFluidNode", "Navier-Stokes 3D Acoustic Fluid Dynamics", DspNodeCategory::Oscillator, "Navier-Stokes 3D Acoustic Fluid Dynamics")
            .with_param(DspParamSchema::knob("sound_speed", "Fluid Sound Speed", 100.0, 2000.0, 343.0, "m/s", MacroRole::Tone, "Fluid Sound Speed parameter"))
            .with_param(DspParamSchema::knob("viscosity", "Kinematic Viscosity", 0.001, 1.0, 0.05, "", MacroRole::Tone, "Kinematic Viscosity parameter"))
            .with_param(DspParamSchema::knob("pressure_damping", "Poisson Pressure Damping", 0.9, 0.999, 0.995, "", MacroRole::Tone, "Poisson Pressure Damping parameter"))
            .with_param(DspParamSchema::knob("grid_res", "3D Lattice Grid Size", 2.0, 8.0, 4.0, "cube", MacroRole::Character, "3D Lattice Grid Size integer control"))
        );
        descriptors.insert("MolecularVibrationResonator".to_string(), DspNodeDescriptor::new("MolecularVibrationResonator", "Molecular Vibration Resonator Crystalline Lattice", DspNodeCategory::Oscillator, "Molecular Vibration Resonator Crystalline Lattice")
            .with_param(DspParamSchema::log_knob("fundamental_hz", "Lattice Base Frequency", 20.0, 10000.0, 520.0, "Hz", MacroRole::Tone, "Lattice Base Frequency logarithmic parameter"))
            .with_param(DspParamSchema::knob("resonance", "Modal Resonance Q", 0.1, 0.99, 0.85, "%", MacroRole::Tone, "Modal Resonance Q parameter"))
            .with_param(DspParamSchema::choice("lattice_type", "Crystalline Lattice", &["Quartz (SiO2)", "Diamond (Carbon)", "Graphene 2D", "Silicon Carbide"], 0, "Crystalline Lattice mode selector"))
            .with_param(DspParamSchema::knob("thermal_damping", "Thermal Damping Loss", 0.0, 1.0, 0.15, "%", MacroRole::Tone, "Thermal Damping Loss parameter"))
        );
        descriptors.insert("ClosedLoopNeuroFeedbackOscillator".to_string(), DspNodeDescriptor::new("ClosedLoopNeuroFeedbackOscillator", "Closed-Loop Neuro-Feedback Relaxation Oscillator", DspNodeCategory::Oscillator, "Closed-Loop Neuro-Feedback Relaxation Oscillator")
            .with_param(DspParamSchema::knob("target_alpha_hz", "Target Alpha Frequency", 7.0, 14.0, 10.5, "Hz", MacroRole::Tone, "Target Alpha Frequency parameter"))
            .with_param(DspParamSchema::knob("feedback_gain", "Neuro-Feedback Loop Gain", 0.0, 1.0, 0.7, "%", MacroRole::Tone, "Neuro-Feedback Loop Gain parameter"))
            .with_param(DspParamSchema::knob("relaxation_depth", "Entrainment Depth", 0.0, 1.0, 0.65, "%", MacroRole::Tone, "Entrainment Depth parameter"))
            .with_param(DspParamSchema::log_knob("binaural_carrier", "Binaural Carrier Center", 100.0, 500.0, 216.0, "Hz", MacroRole::Tone, "Binaural Carrier Center logarithmic parameter"))
        );
        descriptors.insert("BrainwaveEntrainmentBinauralBeat".to_string(), DspNodeDescriptor::new("BrainwaveEntrainmentBinauralBeat", "Brainwave Entrainment Binaural Beat Generator", DspNodeCategory::Oscillator, "Brainwave Entrainment Binaural Beat Generator")
            .with_param(DspParamSchema::log_knob("carrier_freq_hz", "Binaural Carrier Pitch", 50.0, 800.0, 250.0, "Hz", MacroRole::Tone, "Binaural Carrier Pitch logarithmic parameter"))
            .with_param(DspParamSchema::knob("beat_freq_hz", "Differential Beat Freq", 0.5, 30.0, 7.83, "Hz", MacroRole::Tone, "Differential Beat Freq parameter"))
            .with_param(DspParamSchema::knob("entrainment_depth", "Modulation Amplitude", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Modulation Amplitude parameter"))
            .with_param(DspParamSchema::toggle("isochronic_pulse", "Isochronic Pulse Mode", false, "Isochronic Pulse Mode toggle switch"))
        );
        descriptors.insert("FusionResonanceSynth".to_string(), DspNodeDescriptor::new("FusionResonanceSynth", "Tokamak Fusion Magnetic Plasma Resonance Synth", DspNodeCategory::CompositeSynth, "Tokamak Fusion Magnetic Plasma Resonance Synth")
            .with_param(DspParamSchema::knob("magnetic_confinement_t", "Magnetic Field Flux (B)", 1.0, 15.0, 5.5, "Tesla", MacroRole::Tone, "Magnetic Field Flux (B) parameter"))
            .with_param(DspParamSchema::knob("plasma_temp_kev", "Core Plasma Temp", 1.0, 50.0, 15.0, "keV", MacroRole::Tone, "Core Plasma Temp parameter"))
            .with_param(DspParamSchema::log_knob("cyclotron_freq_hz", "Ion Cyclotron Pitch", 100.0, 10000.0, 880.0, "Hz", MacroRole::Tone, "Ion Cyclotron Pitch logarithmic parameter"))
            .with_param(DspParamSchema::knob("fusion_gain", "Acoustic Output Level", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Acoustic Output Level parameter"))
        );
        descriptors.insert("SubharmonicSynthNode".to_string(), DspNodeDescriptor::new("SubharmonicSynthNode", "Subharmonic Multi-Octave Sub Oscillator", DspNodeCategory::Oscillator, "Subharmonic Multi-Octave Sub Oscillator")
            .with_param(DspParamSchema::knob("sub1_oct_gain", "Sub-1 Octave (-12st)", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Sub-1 Octave (-12st) parameter"))
            .with_param(DspParamSchema::knob("sub2_oct_gain", "Sub-2 Octave (-24st)", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Sub-2 Octave (-24st) parameter"))
            .with_param(DspParamSchema::log_knob("lowpass_cutoff", "Sub Lowpass Corner", 40.0, 400.0, 120.0, "Hz", MacroRole::Tone, "Sub Lowpass Corner logarithmic parameter"))
            .with_param(DspParamSchema::knob("drive", "Asymmetric Drive", 1.0, 5.0, 1.5, "x", MacroRole::Tone, "Asymmetric Drive parameter"))
        );
        descriptors.insert("SamplerDevice".to_string(), DspNodeDescriptor::new("SamplerDevice", "Multi-Sample Instrument with ADSR Envelope & Filter", DspNodeCategory::SamplerSlicer, "Multi-Sample Instrument with ADSR Envelope & Filter")
            .with_param(DspParamSchema::log_knob("filter_cutoff", "Ladder Filter Cutoff", 20.0, 20000.0, 18000.0, "Hz", MacroRole::Tone, "Ladder Filter Cutoff logarithmic parameter"))
            .with_param(DspParamSchema::knob("filter_res", "Ladder Resonance", 0.0, 3.9, 0.7, "Q", MacroRole::Tone, "Ladder Resonance parameter"))
            .with_param(DspParamSchema::knob("amp_attack", "Attack Time", 0.001, 2.0, 0.005, "s", MacroRole::Tone, "Attack Time parameter"))
            .with_param(DspParamSchema::knob("amp_release", "Release Time", 0.01, 5.0, 0.4, "s", MacroRole::Tone, "Release Time parameter"))
            .with_param(DspParamSchema::knob("gain", "Master Output Gain", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Master Output Gain parameter"))
        );
        descriptors.insert("MultiSamplerNode".to_string(), DspNodeDescriptor::new("MultiSamplerNode", "Velocity-Layered Multi-Zone Audio Sampler", DspNodeCategory::SamplerSlicer, "Velocity-Layered Multi-Zone Audio Sampler")
            .with_param(DspParamSchema::knob("root_key", "Center Root Key", 0.0, 127.0, 60.0, "midi", MacroRole::Character, "Center Root Key integer control"))
            .with_param(DspParamSchema::knob("velocity_track", "Velocity Sensitivity", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Velocity Sensitivity parameter"))
            .with_param(DspParamSchema::choice("interpolation", "Resampling Kernel", &["Linear", "Hermite Cubic", "Sinc Bandlimited"], 1, "Resampling Kernel mode selector"))
            .with_param(DspParamSchema::knob("gain", "Sample Playback Level", 0.0, 1.0, 0.9, "%", MacroRole::Tone, "Sample Playback Level parameter"))
        );
        descriptors.insert("SingingSynthesisNode".to_string(), DspNodeDescriptor::new("SingingSynthesisNode", "Differentiable Neural Phoneme Singing Synthesizer", DspNodeCategory::NeuralAi, "Differentiable Neural Phoneme Singing Synthesizer")
            .with_param(DspParamSchema::log_knob("f0_pitch", "Vocal Fundamental F0", 50.0, 1000.0, 220.0, "Hz", MacroRole::Tone, "Vocal Fundamental F0 logarithmic parameter"))
            .with_param(DspParamSchema::knob("formant_scale", "Formant Scale", 0.5, 2.0, 1.0, "x", MacroRole::Tone, "Formant Scale parameter"))
            .with_param(DspParamSchema::knob("breathiness", "Glottal Breathiness", 0.0, 1.0, 0.15, "%", MacroRole::Tone, "Glottal Breathiness parameter"))
            .with_param(DspParamSchema::knob("vibrato_depth", "Vocal Vibrato Depth", 0.0, 1.0, 0.3, "%", MacroRole::Tone, "Vocal Vibrato Depth parameter"))
            .with_param(DspParamSchema::choice("phoneme", "Current Phoneme", &["/a/", "/e/", "/i/", "/o/", "/u/", "/m/", "/n/"], 0, "Current Phoneme mode selector"))
        );
        descriptors.insert("SpectralNeuralResynthesizerNode".to_string(), DspNodeDescriptor::new("SpectralNeuralResynthesizerNode", "Latent FFT Timbre Morphing Resynthesizer", DspNodeCategory::SpectralResynthesis, "Latent FFT Timbre Morphing Resynthesizer")
            .with_param(DspParamSchema::knob("latent_timbre", "Latent Timbre Coordinate", -3.0, 3.0, 0.0, "", MacroRole::Tone, "Latent Timbre Coordinate parameter"))
            .with_param(DspParamSchema::knob("spectral_res", "FFT Bin Resolution", 64.0, 1024.0, 256.0, "bins", MacroRole::Character, "FFT Bin Resolution integer control"))
            .with_param(DspParamSchema::knob("transient_boost", "Transient Preservation", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Transient Preservation parameter"))
            .with_param(DspParamSchema::knob("smear", "Phase Spectral Smear", 0.0, 1.0, 0.2, "%", MacroRole::Tone, "Phase Spectral Smear parameter"))
        );
        descriptors.insert("NeuralAudioRepairNode".to_string(), DspNodeDescriptor::new("NeuralAudioRepairNode", "AI Spectral Audio Inpainting & De-Clicking Engine", DspNodeCategory::NeuralAi, "AI Spectral Audio Inpainting & De-Clicking Engine")
            .with_param(DspParamSchema::knob("noise_reduction", "Spectral De-Noise Depth", 0.0, 1.0, 0.6, "%", MacroRole::Tone, "Spectral De-Noise Depth parameter"))
            .with_param(DspParamSchema::knob("inpaint_strength", "Inpaint Reconstruction", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Inpaint Reconstruction parameter"))
            .with_param(DspParamSchema::toggle("transient_protect", "Protect Fast Transients", true, "Protect Fast Transients toggle switch"))
            .with_param(DspParamSchema::knob("blend", "Dry / Wet Repair Blend", 0.0, 1.0, 1.0, "%", MacroRole::Tone, "Dry / Wet Repair Blend parameter"))
        );
        descriptors.insert("FilterSVF".to_string(), DspNodeDescriptor::new("FilterSVF", "State-Variable Multi-Mode Filter (LP/HP/BP/Notch)", DspNodeCategory::FilterEq, "State-Variable Multi-Mode Filter (LP/HP/BP/Notch)")
            .with_param(DspParamSchema::choice("mode", "Filter Topology", &["Lowpass 12dB", "Highpass 12dB", "Bandpass", "Notch"], 0, "Filter Topology mode selector"))
            .with_param(DspParamSchema::log_knob("cutoff", "Cutoff Frequency", 20.0, 20000.0, 1500.0, "Hz", MacroRole::Tone, "Cutoff Frequency logarithmic parameter"))
            .with_param(DspParamSchema::knob("resonance", "Resonance Q", 0.1, 10.0, 1.2, "", MacroRole::Tone, "Resonance Q parameter"))
            .with_param(DspParamSchema::knob("drive", "Pre-filter Drive", 0.0, 18.0, 0.0, "dB", MacroRole::Tone, "Pre-filter Drive parameter"))
        );
        descriptors.insert("FilterLadder".to_string(), DspNodeDescriptor::new("FilterLadder", "24dB/Oct Transistor Moog-Style Ladder Filter", DspNodeCategory::FilterEq, "24dB/Oct Transistor Moog-Style Ladder Filter")
            .with_param(DspParamSchema::log_knob("cutoff", "Cutoff Frequency", 20.0, 20000.0, 1200.0, "Hz", MacroRole::Tone, "Cutoff Frequency logarithmic parameter"))
            .with_param(DspParamSchema::knob("resonance", "Resonance Q", 0.1, 10.0, 1.5, "", MacroRole::Tone, "Resonance Q parameter"))
            .with_param(DspParamSchema::knob("drive", "Drive Saturation", 0.0, 24.0, 3.0, "dB", MacroRole::Tone, "Drive Saturation parameter"))
            .with_param(DspParamSchema::knob("key_track", "Key Tracking", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Key Tracking parameter"))
        );
        descriptors.insert("FilterLadder8".to_string(), DspNodeDescriptor::new("FilterLadder8", "8-Pole 48dB/Oct Ultra-Steep Resonant Ladder Filter", DspNodeCategory::FilterEq, "8-Pole 48dB/Oct Ultra-Steep Resonant Ladder Filter")
            .with_param(DspParamSchema::log_knob("cutoff", "Cutoff Frequency", 20.0, 20000.0, 1000.0, "Hz", MacroRole::Tone, "Cutoff Frequency logarithmic parameter"))
            .with_param(DspParamSchema::knob("resonance", "Resonance Q", 0.1, 12.0, 2.0, "", MacroRole::Tone, "Resonance Q parameter"))
            .with_param(DspParamSchema::knob("pole_count", "Cascaded Poles", 2.0, 8.0, 8.0, "poles", MacroRole::Character, "Cascaded Poles integer control"))
            .with_param(DspParamSchema::knob("saturation", "Non-linear Warmth", 0.0, 1.0, 0.4, "%", MacroRole::Tone, "Non-linear Warmth parameter"))
        );
        descriptors.insert("FilterComb".to_string(), DspNodeDescriptor::new("FilterComb", "Feedback Comb Resonator Filter", DspNodeCategory::FilterEq, "Feedback Comb Resonator Filter")
            .with_param(DspParamSchema::knob("delay_ms", "Delay Time", 0.1, 50.0, 5.0, "ms", MacroRole::Tone, "Delay Time parameter"))
            .with_param(DspParamSchema::knob("feedback", "Feedback Loop", -0.99, 0.99, 0.7, "%", MacroRole::Tone, "Feedback Loop parameter"))
            .with_param(DspParamSchema::knob("damping", "High Damping", 0.0, 1.0, 0.2, "%", MacroRole::Tone, "High Damping parameter"))
            .with_param(DspParamSchema::knob("dry_wet", "Dry / Wet Mix", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Dry / Wet Mix parameter"))
        );
        descriptors.insert("FilterBiquad".to_string(), DspNodeDescriptor::new("FilterBiquad", "Direct Form II Transposed Biquad Multi-Mode Filter", DspNodeCategory::FilterEq, "Direct Form II Transposed Biquad Multi-Mode Filter")
            .with_param(DspParamSchema::choice("filter_type", "Biquad Mode", &["Lowpass", "Highpass", "Bandpass", "Notch", "Peaking", "LowShelf", "HighShelf"], 0, "Biquad Mode mode selector"))
            .with_param(DspParamSchema::log_knob("cutoff", "Center / Cutoff Freq", 20.0, 20000.0, 1000.0, "Hz", MacroRole::Tone, "Center / Cutoff Freq logarithmic parameter"))
            .with_param(DspParamSchema::knob("q_factor", "Filter Q Factor", 0.1, 20.0, 1.0, "Q", MacroRole::Tone, "Filter Q Factor parameter"))
            .with_param(DspParamSchema::knob("gain_db", "Band Boost / Cut", -24.0, 24.0, 0.0, "dB", MacroRole::Tone, "Band Boost / Cut parameter"))
        );
        descriptors.insert("DcBlockFilter".to_string(), DspNodeDescriptor::new("DcBlockFilter", "Zero-Phase DC Offset Removal Highpass Filter", DspNodeCategory::FilterEq, "Zero-Phase DC Offset Removal Highpass Filter")
            .with_param(DspParamSchema::knob("pole_radius", "Pole Radius (R)", 0.9, 0.9999, 0.995, "", MacroRole::Tone, "Pole Radius (R) parameter"))
            .with_param(DspParamSchema::knob("highpass_hz", "Corner Highpass Freq", 1.0, 50.0, 10.0, "Hz", MacroRole::Tone, "Corner Highpass Freq parameter"))
            .with_param(DspParamSchema::toggle("enabled", "DC Block Active", true, "DC Block Active toggle switch"))
        );
        descriptors.insert("LowCutFilter".to_string(), DspNodeDescriptor::new("LowCutFilter", "Precision Sub-Sonic Low-Cut Highpass Filter", DspNodeCategory::FilterEq, "Precision Sub-Sonic Low-Cut Highpass Filter")
            .with_param(DspParamSchema::log_knob("cutoff", "Low Cut Frequency", 10.0, 1000.0, 30.0, "Hz", MacroRole::Tone, "Low Cut Frequency logarithmic parameter"))
            .with_param(DspParamSchema::choice("slope_order", "Filter Slope", &["6 dB/oct", "12 dB/oct", "18 dB/oct", "24 dB/oct", "48 dB/oct"], 1, "Filter Slope mode selector"))
            .with_param(DspParamSchema::knob("resonance", "Resonance Q", 0.1, 2.0, 0.707, "Q", MacroRole::Tone, "Resonance Q parameter"))
        );
        descriptors.insert("HighCutFilter".to_string(), DspNodeDescriptor::new("HighCutFilter", "Anti-Aliasing High-Cut Lowpass Filter", DspNodeCategory::FilterEq, "Anti-Aliasing High-Cut Lowpass Filter")
            .with_param(DspParamSchema::log_knob("cutoff", "High Cut Frequency", 1000.0, 20000.0, 16000.0, "Hz", MacroRole::Tone, "High Cut Frequency logarithmic parameter"))
            .with_param(DspParamSchema::choice("slope_order", "Filter Slope", &["6 dB/oct", "12 dB/oct", "18 dB/oct", "24 dB/oct", "48 dB/oct"], 1, "Filter Slope mode selector"))
            .with_param(DspParamSchema::knob("resonance", "Resonance Q", 0.1, 2.0, 0.707, "Q", MacroRole::Tone, "Resonance Q parameter"))
        );
        descriptors.insert("ParametricEqNode".to_string(), DspNodeDescriptor::new("ParametricEqNode", "4-Band Precision Parametric Equalizer", DspNodeCategory::FilterEq, "4-Band Precision Parametric Equalizer")
            .with_param(DspParamSchema::knob("low_shelf", "Low Shelf (100Hz)", -18.0, 18.0, 0.0, "dB", MacroRole::Tone, "Low Shelf (100Hz) parameter"))
            .with_param(DspParamSchema::knob("low_mid", "Low Mid Gain", -18.0, 18.0, 0.0, "dB", MacroRole::Tone, "Low Mid Gain parameter"))
            .with_param(DspParamSchema::knob("high_mid", "High Mid Gain", -18.0, 18.0, 0.0, "dB", MacroRole::Tone, "High Mid Gain parameter"))
            .with_param(DspParamSchema::knob("high_shelf", "High Shelf (10kHz)", -18.0, 18.0, 0.0, "dB", MacroRole::Tone, "High Shelf (10kHz) parameter"))
            .with_param(DspParamSchema::log_knob("mid_freq", "Parametric Mid Freq", 200.0, 8000.0, 1000.0, "Hz", MacroRole::Tone, "Parametric Mid Freq logarithmic parameter"))
            .with_param(DspParamSchema::knob("mid_q", "Parametric Mid Q", 0.2, 10.0, 1.0, "Q", MacroRole::Tone, "Parametric Mid Q parameter"))
        );
        descriptors.insert("MultiChannelSpectralEqualizerNode".to_string(), DspNodeDescriptor::new("MultiChannelSpectralEqualizerNode", "64-Band Multi-Channel Spectral Sculptor", DspNodeCategory::SpectralResynthesis, "64-Band Multi-Channel Spectral Sculptor")
            .with_param(DspParamSchema::knob("tilt_slope", "Spectral Tilt Slope", -6.0, 6.0, 0.0, "dB/oct", MacroRole::Tone, "Spectral Tilt Slope parameter"))
            .with_param(DspParamSchema::knob("air_band", "High Air Presence", 0.0, 12.0, 2.0, "dB", MacroRole::Tone, "High Air Presence parameter"))
            .with_param(DspParamSchema::knob("low_sub", "Sub Bass Focus", 0.0, 12.0, 1.5, "dB", MacroRole::Tone, "Sub Bass Focus parameter"))
            .with_param(DspParamSchema::knob("dynamic_unmask", "Dynamic Masking Reduction", 0.0, 1.0, 0.4, "%", MacroRole::Tone, "Dynamic Masking Reduction parameter"))
        );
        descriptors.insert("FormantFilterNode".to_string(), DspNodeDescriptor::new("FormantFilterNode", "Vowel Formant Filter Matrix", DspNodeCategory::FilterEq, "Vowel Formant Filter Matrix")
            .with_param(DspParamSchema::choice("vowel", "Target Vowel", &["Vowel A (Ah)", "Vowel E (Eh)", "Vowel I (Ee)", "Vowel O (Oh)", "Vowel U (Oo)"], 0, "Target Vowel mode selector"))
            .with_param(DspParamSchema::knob("formant_shift", "Formant Shift", -12.0, 12.0, 0.0, "st", MacroRole::Tone, "Formant Shift parameter"))
            .with_param(DspParamSchema::knob("throat_len", "Acoustic Throat Length", 0.5, 2.0, 1.0, "x", MacroRole::Tone, "Acoustic Throat Length parameter"))
            .with_param(DspParamSchema::knob("resonance_q", "Formant Resonance Q", 1.0, 20.0, 5.0, "Q", MacroRole::Tone, "Formant Resonance Q parameter"))
        );
        descriptors.insert("ModalFilterNode".to_string(), DspNodeDescriptor::new("ModalFilterNode", "High-Order Acoustic Modal Resonator Bank", DspNodeCategory::FilterEq, "High-Order Acoustic Modal Resonator Bank")
            .with_param(DspParamSchema::log_knob("base_freq", "Fundamental Modal Freq", 50.0, 8000.0, 440.0, "Hz", MacroRole::Tone, "Fundamental Modal Freq logarithmic parameter"))
            .with_param(DspParamSchema::knob("mode_count", "Active Modal Poles", 1.0, 16.0, 8.0, "modes", MacroRole::Character, "Active Modal Poles integer control"))
            .with_param(DspParamSchema::knob("decay_rate", "Modal Decay Time", 0.05, 5.0, 1.2, "s", MacroRole::Tone, "Modal Decay Time parameter"))
            .with_param(DspParamSchema::knob("inharmonicity", "Inharmonic Mode Shift", 0.0, 2.0, 0.15, "", MacroRole::Tone, "Inharmonic Mode Shift parameter"))
        );
        descriptors.insert("HornReflectionFilter".to_string(), DspNodeDescriptor::new("HornReflectionFilter", "Acoustic Bell Flare & Throat Reflection Filter", DspNodeCategory::FilterEq, "Acoustic Bell Flare & Throat Reflection Filter")
            .with_param(DspParamSchema::knob("bell_flare", "Bell Flare Expansion", 0.1, 3.0, 1.0, "x", MacroRole::Tone, "Bell Flare Expansion parameter"))
            .with_param(DspParamSchema::knob("throat_diameter", "Throat Diameter", 5.0, 50.0, 15.0, "mm", MacroRole::Tone, "Throat Diameter parameter"))
            .with_param(DspParamSchema::knob("reflection_coeff", "End Reflection Coefficient", 0.0, 1.0, 0.65, "%", MacroRole::Tone, "End Reflection Coefficient parameter"))
        );
        descriptors.insert("AutomatedDynamicEqNode".to_string(), DspNodeDescriptor::new("AutomatedDynamicEqNode", "AI-Driven Intelligent Masking Reduction Dynamic EQ", DspNodeCategory::FilterEq, "AI-Driven Intelligent Masking Reduction Dynamic EQ")
            .with_param(DspParamSchema::knob("target_lufs", "Target Program LUFS", -24.0, -8.0, -14.0, "LUFS", MacroRole::Tone, "Target Program LUFS parameter"))
            .with_param(DspParamSchema::knob("low_dynamic_cut", "Dynamic Mud Cut (250Hz)", 0.0, 12.0, 3.0, "dB", MacroRole::Tone, "Dynamic Mud Cut (250Hz) parameter"))
            .with_param(DspParamSchema::knob("high_air_lift", "Dynamic Air Presence (12kHz)", 0.0, 12.0, 2.5, "dB", MacroRole::Tone, "Dynamic Air Presence (12kHz) parameter"))
            .with_param(DspParamSchema::knob("adaptation_speed", "Neural Adaptation Speed", 10.0, 500.0, 100.0, "ms", MacroRole::Tone, "Neural Adaptation Speed parameter"))
        );
        descriptors.insert("SubharmonicQuantumTunnelingFilter".to_string(), DspNodeDescriptor::new("SubharmonicQuantumTunnelingFilter", "Subharmonic Quantum Tunneling Resonant Filter", DspNodeCategory::FilterEq, "Subharmonic Quantum Tunneling Resonant Filter")
            .with_param(DspParamSchema::knob("barrier_width_nm", "Quantum Barrier Width", 0.5, 10.0, 2.5, "nm", MacroRole::Tone, "Quantum Barrier Width parameter"))
            .with_param(DspParamSchema::knob("tunneling_prob", "Tunneling Probability", 0.01, 1.0, 0.35, "%", MacroRole::Tone, "Tunneling Probability parameter"))
            .with_param(DspParamSchema::knob("resonance_q", "Subharmonic Q Factor", 1.0, 30.0, 8.0, "Q", MacroRole::Tone, "Subharmonic Q Factor parameter"))
            .with_param(DspParamSchema::log_knob("cutoff_hz", "Filter Cutoff Corner", 20.0, 12000.0, 800.0, "Hz", MacroRole::Tone, "Filter Cutoff Corner logarithmic parameter"))
        );
        descriptors.insert("MetamaterialRefractionFilter".to_string(), DspNodeDescriptor::new("MetamaterialRefractionFilter", "Negative Index Acoustic Metamaterial Filter", DspNodeCategory::FilterEq, "Negative Index Acoustic Metamaterial Filter")
            .with_param(DspParamSchema::knob("refractive_index", "Negative Index (n)", -5.0, -0.1, -1.4, "n", MacroRole::Tone, "Negative Index (n) parameter"))
            .with_param(DspParamSchema::knob("slab_thickness_mm", "Metamaterial Thickness", 1.0, 50.0, 12.0, "mm", MacroRole::Tone, "Metamaterial Thickness parameter"))
            .with_param(DspParamSchema::knob("evanescent_gain", "Evanescent Amplification", 0.0, 12.0, 3.5, "dB", MacroRole::Tone, "Evanescent Amplification parameter"))
            .with_param(DspParamSchema::knob("resonance_q", "Acoustic Bandgap Q", 1.0, 25.0, 6.0, "Q", MacroRole::Tone, "Acoustic Bandgap Q parameter"))
        );
        descriptors.insert("SpatialAcousticHologramFilter".to_string(), DspNodeDescriptor::new("SpatialAcousticHologramFilter", "Spatial Acoustic Hologram Reconstruction Filter", DspNodeCategory::FilterEq, "Spatial Acoustic Hologram Reconstruction Filter")
            .with_param(DspParamSchema::knob("hologram_resolution", "Holographic Phase Steps", 16.0, 256.0, 64.0, "steps", MacroRole::Character, "Holographic Phase Steps integer control"))
            .with_param(DspParamSchema::knob("phase_offset_rad", "Acoustic Phase Bias", 0.0, 6.28, 0.0, "rad", MacroRole::Tone, "Acoustic Phase Bias parameter"))
            .with_param(DspParamSchema::knob("depth_focus_m", "Focal Distance", 0.2, 10.0, 2.0, "m", MacroRole::Tone, "Focal Distance parameter"))
            .with_param(DspParamSchema::knob("spatial_dispersion", "Diffractive Smear", 0.0, 1.0, 0.2, "%", MacroRole::Tone, "Diffractive Smear parameter"))
        );
        descriptors.insert("LinearPhaseCrossoverNode".to_string(), DspNodeDescriptor::new("LinearPhaseCrossoverNode", "4-Way Linear Phase Mastering Crossover Filter", DspNodeCategory::FilterEq, "4-Way Linear Phase Mastering Crossover Filter")
            .with_param(DspParamSchema::log_knob("low_cross_hz", "Low Crossover Split", 40.0, 300.0, 120.0, "Hz", MacroRole::Tone, "Low Crossover Split logarithmic parameter"))
            .with_param(DspParamSchema::log_knob("mid_cross_hz", "Mid Crossover Split", 400.0, 3000.0, 1200.0, "Hz", MacroRole::Tone, "Mid Crossover Split logarithmic parameter"))
            .with_param(DspParamSchema::log_knob("high_cross_hz", "High Crossover Split", 3000.0, 16000.0, 6000.0, "Hz", MacroRole::Tone, "High Crossover Split logarithmic parameter"))
            .with_param(DspParamSchema::knob("fir_tap_order", "FIR Filter Tap Count", 256.0, 4096.0, 1024.0, "taps", MacroRole::Character, "FIR Filter Tap Count integer control"))
        );
        descriptors.insert("SpectralMatchingEqNode".to_string(), DspNodeDescriptor::new("SpectralMatchingEqNode", "AI 128-Band Spectral Curve Matching EQ", DspNodeCategory::SpectralResynthesis, "AI 128-Band Spectral Curve Matching EQ")
            .with_param(DspParamSchema::choice("target_curve", "Target Spectral Profile", &["Reference Track Target", "Harman Target", "Pink Noise Curve (1/f)", "Commercial Master"], 0, "Target Spectral Profile mode selector"))
            .with_param(DspParamSchema::knob("matching_intensity", "Curve Match Depth", 0.0, 100.0, 75.0, "%", MacroRole::Tone, "Curve Match Depth parameter"))
            .with_param(DspParamSchema::knob("smoothing_octaves", "Spectral Smoothing", 0.1, 2.0, 0.5, "oct", MacroRole::Tone, "Spectral Smoothing parameter"))
            .with_param(DspParamSchema::knob("gain_limit_db", "Max Boost/Cut Limit", 1.0, 18.0, 6.0, "dB", MacroRole::Tone, "Max Boost/Cut Limit parameter"))
        );
        descriptors.insert("SpectralTiltNode".to_string(), DspNodeDescriptor::new("SpectralTiltNode", "Linear Phase Psychoacoustic Spectral Tilt Filter", DspNodeCategory::SpectralResynthesis, "Linear Phase Psychoacoustic Spectral Tilt Filter")
            .with_param(DspParamSchema::knob("tilt_slope_db_oct", "Bode Spectral Tilt", -6.0, 6.0, 1.5, "dB/oct", MacroRole::Tone, "Bode Spectral Tilt parameter"))
            .with_param(DspParamSchema::log_knob("pivot_freq_hz", "Center Pivot Frequency", 200.0, 5000.0, 1000.0, "Hz", MacroRole::Tone, "Center Pivot Frequency logarithmic parameter"))
            .with_param(DspParamSchema::knob("air_lift_db", "Top End Air Sheen", 0.0, 6.0, 1.2, "dB", MacroRole::Tone, "Top End Air Sheen parameter"))
            .with_param(DspParamSchema::knob("sub_weight_db", "Sub-Bass Weight", -6.0, 6.0, 0.0, "dB", MacroRole::Tone, "Sub-Bass Weight parameter"))
        );
        descriptors.insert("CombResonatorNode".to_string(), DspNodeDescriptor::new("CombResonatorNode", "Tuned Feedback Comb Filter Resonator", DspNodeCategory::FilterEq, "Tuned Feedback Comb Filter Resonator")
            .with_param(DspParamSchema::log_knob("frequency_hz", "Comb Fundamental Pitch", 20.0, 5000.0, 440.0, "Hz", MacroRole::Tone, "Comb Fundamental Pitch logarithmic parameter"))
            .with_param(DspParamSchema::knob("feedback_gain", "Feedback Loop Resonance", -0.99, 0.99, 0.85, "%", MacroRole::Tone, "Feedback Loop Resonance parameter"))
            .with_param(DspParamSchema::knob("damping_ratio", "High Frequency Damping", 0.0, 1.0, 0.25, "%", MacroRole::Tone, "High Frequency Damping parameter"))
            .with_param(DspParamSchema::knob("dry_wet", "Dry / Wet Blend", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Dry / Wet Blend parameter"))
        );
        descriptors.insert("EnvADSR".to_string(), DspNodeDescriptor::new("EnvADSR", "Exponential Multi-Stage ADSR Envelope", DspNodeCategory::Modulation, "Exponential Multi-Stage ADSR Envelope")
            .with_param(DspParamSchema::log_knob("attack", "Attack Time", 0.5, 5000.0, 10.0, "ms", MacroRole::Tone, "Attack Time logarithmic parameter"))
            .with_param(DspParamSchema::log_knob("decay", "Decay Time", 1.0, 10000.0, 250.0, "ms", MacroRole::Tone, "Decay Time logarithmic parameter"))
            .with_param(DspParamSchema::knob("sustain", "Sustain Level", 0.0, 1.0, 0.7, "%", MacroRole::Tone, "Sustain Level parameter"))
            .with_param(DspParamSchema::log_knob("release", "Release Time", 1.0, 10000.0, 400.0, "ms", MacroRole::Tone, "Release Time logarithmic parameter"))
            .with_param(DspParamSchema::knob("curve", "Curve Curvature", 0.1, 5.0, 1.0, "exp", MacroRole::Tone, "Curve Curvature parameter"))
        );
        descriptors.insert("OscLFO".to_string(), DspNodeDescriptor::new("OscLFO", "Multi-Waveform Syncable LFO Generator", DspNodeCategory::Modulation, "Multi-Waveform Syncable LFO Generator")
            .with_param(DspParamSchema::log_knob("rate", "LFO Frequency", 0.05, 50.0, 2.0, "Hz", MacroRole::Tone, "LFO Frequency logarithmic parameter"))
            .with_param(DspParamSchema::choice("shape", "Waveform Shape", &["Sine", "Triangle", "Saw Up", "Saw Down", "Square", "Sample & Hold"], 0, "Waveform Shape mode selector"))
            .with_param(DspParamSchema::knob("depth", "Modulation Depth", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Modulation Depth parameter"))
            .with_param(DspParamSchema::knob("phase", "Phase Offset", 0.0, 360.0, 0.0, "deg", MacroRole::Tone, "Phase Offset parameter"))
            .with_param(DspParamSchema::toggle("tempo_sync", "BPM Tempo Sync", false, "BPM Tempo Sync toggle switch"))
        );
        descriptors.insert("EnvelopeFollowerNode".to_string(), DspNodeDescriptor::new("EnvelopeFollowerNode", "Real-Time Peak/RMS Sidechain Envelope Follower", DspNodeCategory::Modulation, "Real-Time Peak/RMS Sidechain Envelope Follower")
            .with_param(DspParamSchema::knob("attack_ms", "Attack Smoothing", 0.1, 200.0, 10.0, "ms", MacroRole::Tone, "Attack Smoothing parameter"))
            .with_param(DspParamSchema::knob("release_ms", "Release Smoothing", 5.0, 1000.0, 150.0, "ms", MacroRole::Tone, "Release Smoothing parameter"))
            .with_param(DspParamSchema::knob("sensitivity", "Follower Sensitivity", 0.0, 1.0, 0.7, "%", MacroRole::Tone, "Follower Sensitivity parameter"))
            .with_param(DspParamSchema::toggle("peak_mode", "Peak (True) vs RMS (False)", false, "Peak (True) vs RMS (False) toggle switch"))
        );
        descriptors.insert("ModulationMatrix".to_string(), DspNodeDescriptor::new("ModulationMatrix", "16x16 Modulation Cross-Routing Matrix", DspNodeCategory::Modulation, "16x16 Modulation Cross-Routing Matrix")
            .with_param(DspParamSchema::knob("master_depth", "Master Matrix Scale", 0.0, 2.0, 1.0, "x", MacroRole::Tone, "Master Matrix Scale parameter"))
            .with_param(DspParamSchema::knob("slew_rate", "Slew Limiter Smoothing", 0.0, 500.0, 5.0, "ms", MacroRole::Tone, "Slew Limiter Smoothing parameter"))
            .with_param(DspParamSchema::toggle("bipolar", "Bipolar Signal Range", true, "Bipolar Signal Range toggle switch"))
        );
        descriptors.insert("PitchShifterNode".to_string(), DspNodeDescriptor::new("PitchShifterNode", "Real-time Granular Pitch & Formant Shifter", DspNodeCategory::Modulation, "Real-time Granular Pitch & Formant Shifter")
            .with_param(DspParamSchema::knob("pitch_shift", "Pitch Shift", -24.0, 24.0, 0.0, "st", MacroRole::Tone, "Pitch Shift parameter"))
            .with_param(DspParamSchema::knob("fine_tune", "Fine Detune", -100.0, 100.0, 0.0, "cents", MacroRole::Tone, "Fine Detune parameter"))
            .with_param(DspParamSchema::toggle("formant_lock", "Preserve Formants", true, "Preserve Formants toggle switch"))
            .with_param(DspParamSchema::knob("window_ms", "Grain Window Size", 10.0, 200.0, 50.0, "ms", MacroRole::Tone, "Grain Window Size parameter"))
        );
        descriptors.insert("AutoWahNode".to_string(), DspNodeDescriptor::new("AutoWahNode", "Dynamic Envelope-Controlled Auto-Wah", DspNodeCategory::Modulation, "Dynamic Envelope-Controlled Auto-Wah")
            .with_param(DspParamSchema::knob("sensitivity", "Envelope Sensitivity", 0.0, 1.0, 0.65, "%", MacroRole::Tone, "Envelope Sensitivity parameter"))
            .with_param(DspParamSchema::log_knob("sweep_range", "Wah Center Freq", 200.0, 6000.0, 2400.0, "Hz", MacroRole::Tone, "Wah Center Freq logarithmic parameter"))
            .with_param(DspParamSchema::knob("attack_ms", "Attack Speed", 1.0, 200.0, 15.0, "ms", MacroRole::Tone, "Attack Speed parameter"))
            .with_param(DspParamSchema::knob("decay_ms", "Decay Speed", 10.0, 1000.0, 180.0, "ms", MacroRole::Tone, "Decay Speed parameter"))
            .with_param(DspParamSchema::knob("resonance", "Resonance Q", 1.0, 15.0, 4.5, "Q", MacroRole::Tone, "Resonance Q parameter"))
            .with_param(DspParamSchema::knob("dry_wet", "Dry / Wet Mix", 0.0, 1.0, 1.0, "%", MacroRole::Tone, "Dry / Wet Mix parameter"))
        );
        descriptors.insert("AutoTuneNode".to_string(), DspNodeDescriptor::new("AutoTuneNode", "Real-time Microtonal Scale Quantizer & Pitch Corrector", DspNodeCategory::Modulation, "Real-time Microtonal Scale Quantizer & Pitch Corrector")
            .with_param(DspParamSchema::choice("target_scale", "Target Scale", &["Chromatic", "Major", "Natural Minor", "Harmonic Minor", "Pentatonic", "Dorian"], 0, "Target Scale mode selector"))
            .with_param(DspParamSchema::knob("speed_ms", "Retune Speed", 0.0, 200.0, 25.0, "ms", MacroRole::Tone, "Retune Speed parameter"))
            .with_param(DspParamSchema::knob("humanize", "Vocal Humanize", 0.0, 1.0, 0.3, "%", MacroRole::Tone, "Vocal Humanize parameter"))
            .with_param(DspParamSchema::knob("amount", "Correction Depth", 0.0, 1.0, 0.85, "%", MacroRole::Tone, "Correction Depth parameter"))
        );
        descriptors.insert("AutotuneNode".to_string(), DspNodeDescriptor::new("AutotuneNode", "Vocal Scale Intonation & Pitch Quantizer", DspNodeCategory::Modulation, "Vocal Scale Intonation & Pitch Quantizer")
            .with_param(DspParamSchema::choice("scale", "Scale Preset", &["Chromatic", "Equal Temperament", "Just Intonation", "Pythagorean", "Quarter Tone 24-EDO"], 0, "Scale Preset mode selector"))
            .with_param(DspParamSchema::knob("snap_speed", "Snap Transition Rate", 0.0, 100.0, 15.0, "ms", MacroRole::Tone, "Snap Transition Rate parameter"))
            .with_param(DspParamSchema::knob("depth", "Quantize Amount", 0.0, 1.0, 0.9, "%", MacroRole::Tone, "Quantize Amount parameter"))
        );
        descriptors.insert("VocalPitchFormantCorrectorNode".to_string(), DspNodeDescriptor::new("VocalPitchFormantCorrectorNode", "Surgical Formant & Gender Vocal Pitch Shifter", DspNodeCategory::Modulation, "Surgical Formant & Gender Vocal Pitch Shifter")
            .with_param(DspParamSchema::knob("pitch_shift", "Chromatic Pitch Shift", -12.0, 12.0, 0.0, "st", MacroRole::Tone, "Chromatic Pitch Shift parameter"))
            .with_param(DspParamSchema::knob("formant_shift", "Formant Throat Shift", -12.0, 12.0, 0.0, "st", MacroRole::Tone, "Formant Throat Shift parameter"))
            .with_param(DspParamSchema::knob("gender_morph", "Gender Timbre Morph", -1.0, 1.0, 0.0, "", MacroRole::Tone, "Gender Timbre Morph parameter"))
            .with_param(DspParamSchema::knob("throat_model", "Throat Length Scale", 0.5, 2.0, 1.0, "x", MacroRole::Tone, "Throat Length Scale parameter"))
        );
        descriptors.insert("GlitchBufferNode".to_string(), DspNodeDescriptor::new("GlitchBufferNode", "Stutter, Shuffle, & Reverse Glitch Buffer Matrix", DspNodeCategory::Modulation, "Stutter, Shuffle, & Reverse Glitch Buffer Matrix")
            .with_param(DspParamSchema::knob("slice_size", "Glitch Slice Window", 10.0, 500.0, 80.0, "ms", MacroRole::Tone, "Glitch Slice Window parameter"))
            .with_param(DspParamSchema::knob("stutter_prob", "Stutter Probability", 0.0, 1.0, 0.3, "%", MacroRole::Tone, "Stutter Probability parameter"))
            .with_param(DspParamSchema::knob("reverse_prob", "Reverse Probability", 0.0, 1.0, 0.2, "%", MacroRole::Tone, "Reverse Probability parameter"))
            .with_param(DspParamSchema::knob("pitch_drop", "Tape Stop Pitch Drop", 0.0, 1.0, 0.15, "%", MacroRole::Tone, "Tape Stop Pitch Drop parameter"))
        );
        descriptors.insert("EffectChorus".to_string(), DspNodeDescriptor::new("EffectChorus", "Multi-Voice BBD Stereo Chorus", DspNodeCategory::Modulation, "Multi-Voice BBD Stereo Chorus")
            .with_param(DspParamSchema::log_knob("rate", "Modulation Rate", 0.1, 10.0, 1.2, "Hz", MacroRole::Tone, "Modulation Rate logarithmic parameter"))
            .with_param(DspParamSchema::knob("depth", "Modulation Depth", 0.0, 1.0, 0.6, "%", MacroRole::Tone, "Modulation Depth parameter"))
            .with_param(DspParamSchema::knob("voices", "Chorus Voice Count", 2.0, 8.0, 4.0, "voices", MacroRole::Character, "Chorus Voice Count integer control"))
            .with_param(DspParamSchema::knob("mix", "Dry / Wet Mix", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Dry / Wet Mix parameter"))
        );
        descriptors.insert("EffectFlanger".to_string(), DspNodeDescriptor::new("EffectFlanger", "Through-Zero Flanger & Comb Sweep", DspNodeCategory::Modulation, "Through-Zero Flanger & Comb Sweep")
            .with_param(DspParamSchema::log_knob("rate", "Sweep Rate", 0.05, 5.0, 0.4, "Hz", MacroRole::Tone, "Sweep Rate logarithmic parameter"))
            .with_param(DspParamSchema::knob("depth", "Flange Depth", 0.0, 1.0, 0.75, "%", MacroRole::Tone, "Flange Depth parameter"))
            .with_param(DspParamSchema::knob("feedback", "Regeneration Feedback", -0.95, 0.95, 0.65, "%", MacroRole::Tone, "Regeneration Feedback parameter"))
            .with_param(DspParamSchema::knob("mix", "Dry / Wet Mix", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Dry / Wet Mix parameter"))
        );
        descriptors.insert("EffectPhaser".to_string(), DspNodeDescriptor::new("EffectPhaser", "Multi-Stage Allpass Phase Shifter", DspNodeCategory::Modulation, "Multi-Stage Allpass Phase Shifter")
            .with_param(DspParamSchema::log_knob("rate", "Phase Sweep Rate", 0.05, 10.0, 0.8, "Hz", MacroRole::Tone, "Phase Sweep Rate logarithmic parameter"))
            .with_param(DspParamSchema::choice("stages", "Allpass Stage Count", &["2 Stages", "4 Stages", "6 Stages", "8 Stages", "12 Stages"], 1, "Allpass Stage Count mode selector"))
            .with_param(DspParamSchema::knob("feedback", "Phase Feedback", 0.0, 0.95, 0.5, "%", MacroRole::Tone, "Phase Feedback parameter"))
            .with_param(DspParamSchema::knob("mix", "Dry / Wet Mix", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Dry / Wet Mix parameter"))
        );
        descriptors.insert("RingModulator".to_string(), DspNodeDescriptor::new("RingModulator", "Carrier Multiplier Ring Modulator", DspNodeCategory::Modulation, "Carrier Multiplier Ring Modulator")
            .with_param(DspParamSchema::log_knob("freq", "Carrier Frequency", 20.0, 5000.0, 320.0, "Hz", MacroRole::Tone, "Carrier Frequency logarithmic parameter"))
            .with_param(DspParamSchema::choice("waveform", "Carrier Waveform", &["Sine", "Triangle", "Square"], 0, "Carrier Waveform mode selector"))
            .with_param(DspParamSchema::knob("mix", "Dry / Wet Mix", 0.0, 1.0, 0.7, "%", MacroRole::Tone, "Dry / Wet Mix parameter"))
        );
        descriptors.insert("FrequencyShifter".to_string(), DspNodeDescriptor::new("FrequencyShifter", "Single-Sideband Frequency Shifter", DspNodeCategory::Modulation, "Single-Sideband Frequency Shifter")
            .with_param(DspParamSchema::knob("shift_hz", "Frequency Shift", -1000.0, 1000.0, 25.0, "Hz", MacroRole::Tone, "Frequency Shift parameter"))
            .with_param(DspParamSchema::knob("feedback", "Feedback Loop", 0.0, 0.95, 0.0, "%", MacroRole::Tone, "Feedback Loop parameter"))
            .with_param(DspParamSchema::knob("mix", "Dry / Wet Mix", 0.0, 1.0, 0.6, "%", MacroRole::Tone, "Dry / Wet Mix parameter"))
        );
        descriptors.insert("ChaoticFractalAttractorModulator".to_string(), DspNodeDescriptor::new("ChaoticFractalAttractorModulator", "Lorenz / Rossler Chaotic Strange Attractor LFO", DspNodeCategory::Modulation, "Lorenz / Rossler Chaotic Strange Attractor LFO")
            .with_param(DspParamSchema::choice("attractor_type", "Attractor Topology", &["Lorenz Butterfly", "Rossler Spiral", "Chua Circuit", "Clifford Torus"], 0, "Attractor Topology mode selector"))
            .with_param(DspParamSchema::knob("sigma_rate", "Prandtl Sigma Rate", 1.0, 20.0, 10.0, "sigma", MacroRole::Tone, "Prandtl Sigma Rate parameter"))
            .with_param(DspParamSchema::knob("rho_chaos", "Rayleigh Chaos Rho", 5.0, 50.0, 28.0, "rho", MacroRole::Tone, "Rayleigh Chaos Rho parameter"))
            .with_param(DspParamSchema::knob("modulation_depth", "Modulation Amplitude", 0.0, 1.0, 0.75, "%", MacroRole::Tone, "Modulation Amplitude parameter"))
        );
        descriptors.insert("QuantumEntanglementModulationRouting".to_string(), DspNodeDescriptor::new("QuantumEntanglementModulationRouting", "Quantum Entangled Bidirectional Modulator", DspNodeCategory::Modulation, "Quantum Entangled Bidirectional Modulator")
            .with_param(DspParamSchema::choice("bell_state", "Entangled Bell State", &["Phi+ Superposition", "Phi- Phase Inverted", "Psi+ Symmetric", "Psi- Anti-Symmetric"], 0, "Entangled Bell State mode selector"))
            .with_param(DspParamSchema::knob("entanglement_fidelity", "Quantum State Fidelity", 0.5, 1.0, 0.95, "%", MacroRole::Tone, "Quantum State Fidelity parameter"))
            .with_param(DspParamSchema::knob("measurement_basis", "Measurement Angle", 0.0, 360.0, 45.0, "deg", MacroRole::Tone, "Measurement Angle parameter"))
            .with_param(DspParamSchema::knob("routing_depth", "Entangled Cross-Route", 0.0, 1.0, 0.85, "%", MacroRole::Tone, "Entangled Cross-Route parameter"))
        );
        descriptors.insert("MhdPlasmaWaveModulator".to_string(), DspNodeDescriptor::new("MhdPlasmaWaveModulator", "Magnetohydrodynamic Plasma Wave Modulator", DspNodeCategory::Modulation, "Magnetohydrodynamic Plasma Wave Modulator")
            .with_param(DspParamSchema::knob("alfven_speed_ms", "Alfvén Wave Speed", 50.0, 2000.0, 450.0, "m/s", MacroRole::Tone, "Alfvén Wave Speed parameter"))
            .with_param(DspParamSchema::knob("plasma_beta", "Plasma Beta Factor", 0.01, 2.0, 0.4, "beta", MacroRole::Tone, "Plasma Beta Factor parameter"))
            .with_param(DspParamSchema::knob("magnetic_shear", "Magnetic Shear Angle", 0.0, 90.0, 15.0, "deg", MacroRole::Tone, "Magnetic Shear Angle parameter"))
            .with_param(DspParamSchema::log_knob("wave_frequency_hz", "Modulation Carrier Freq", 0.1, 50.0, 4.5, "Hz", MacroRole::Tone, "Modulation Carrier Freq logarithmic parameter"))
        );
        descriptors.insert("BbdChorusNode".to_string(), DspNodeDescriptor::new("BbdChorusNode", "Bucket-Brigade Analog Stereo Chorus Ensemble", DspNodeCategory::Modulation, "Bucket-Brigade Analog Stereo Chorus Ensemble")
            .with_param(DspParamSchema::knob("bucket_stages", "BBD Delay Stages", 256.0, 4096.0, 1024.0, "stages", MacroRole::Character, "BBD Delay Stages integer control"))
            .with_param(DspParamSchema::knob("clock_rate_khz", "BBD Clock Rate", 10.0, 100.0, 45.0, "kHz", MacroRole::Tone, "BBD Clock Rate parameter"))
            .with_param(DspParamSchema::knob("modulation_depth", "LFO Sweep Depth", 0.0, 1.0, 0.65, "%", MacroRole::Tone, "LFO Sweep Depth parameter"))
            .with_param(DspParamSchema::knob("stereo_spread", "Stereo Ensemble Width", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Stereo Ensemble Width parameter"))
        );
        descriptors.insert("ThroughZeroFlangerNode".to_string(), DspNodeDescriptor::new("ThroughZeroFlangerNode", "Tape-Style Through-Zero Flanger & Comb Phase", DspNodeCategory::Modulation, "Tape-Style Through-Zero Flanger & Comb Phase")
            .with_param(DspParamSchema::knob("zero_crossing_ms", "Zero Cross Delay Point", 0.0, 10.0, 2.5, "ms", MacroRole::Tone, "Zero Cross Delay Point parameter"))
            .with_param(DspParamSchema::knob("flange_rate_hz", "Sweep Rate", 0.05, 5.0, 0.35, "Hz", MacroRole::Tone, "Sweep Rate parameter"))
            .with_param(DspParamSchema::knob("regen_feedback", "Regenerative Feedback", -0.98, 0.98, 0.7, "%", MacroRole::Tone, "Regenerative Feedback parameter"))
            .with_param(DspParamSchema::toggle("phase_invert", "Phase Cancellation Invert", true, "Phase Cancellation Invert toggle switch"))
        );
        descriptors.insert("TapeFlutterNode".to_string(), DspNodeDescriptor::new("TapeFlutterNode", "Capstan Wow & Tape Flutter Modulator", DspNodeCategory::Modulation, "Capstan Wow & Tape Flutter Modulator")
            .with_param(DspParamSchema::knob("wow_rate_hz", "Capstan Wow Speed", 0.2, 4.0, 1.2, "Hz", MacroRole::Tone, "Capstan Wow Speed parameter"))
            .with_param(DspParamSchema::knob("wow_depth", "Capstan Wow Depth", 0.0, 1.0, 0.25, "%", MacroRole::Tone, "Capstan Wow Depth parameter"))
            .with_param(DspParamSchema::knob("flutter_rate_hz", "Scrape Flutter Speed", 5.0, 50.0, 18.0, "Hz", MacroRole::Tone, "Scrape Flutter Speed parameter"))
            .with_param(DspParamSchema::knob("flutter_depth", "Scrape Flutter Depth", 0.0, 1.0, 0.15, "%", MacroRole::Tone, "Scrape Flutter Depth parameter"))
        );
        descriptors.insert("GranularFreezeNode".to_string(), DspNodeDescriptor::new("GranularFreezeNode", "Infinite Granular Audio Buffer Freeze Engine", DspNodeCategory::Modulation, "Infinite Granular Audio Buffer Freeze Engine")
            .with_param(DspParamSchema::toggle("freeze_active", "Buffer Freeze Locked", false, "Buffer Freeze Locked toggle switch"))
            .with_param(DspParamSchema::knob("grain_size_ms", "Grain Size Window", 10.0, 500.0, 80.0, "ms", MacroRole::Tone, "Grain Size Window parameter"))
            .with_param(DspParamSchema::knob("grain_jitter", "Playback Random Jitter", 0.0, 1.0, 0.3, "%", MacroRole::Tone, "Playback Random Jitter parameter"))
            .with_param(DspParamSchema::knob("diffusion_mix", "Stereo Diffusion Blend", 0.0, 1.0, 0.65, "%", MacroRole::Tone, "Stereo Diffusion Blend parameter"))
        );
        descriptors.insert("GranularPitchShifter".to_string(), DspNodeDescriptor::new("GranularPitchShifter", "Dual-Grain Pitch Transposition & Shifter", DspNodeCategory::Modulation, "Dual-Grain Pitch Transposition & Shifter")
            .with_param(DspParamSchema::knob("pitch_semitones", "Pitch Transposition", -24.0, 24.0, 7.0, "st", MacroRole::Tone, "Pitch Transposition parameter"))
            .with_param(DspParamSchema::knob("fine_cents", "Fine Detuning", -50.0, 50.0, 0.0, "cents", MacroRole::Tone, "Fine Detuning parameter"))
            .with_param(DspParamSchema::knob("window_size_ms", "Grain Overlap Window", 15.0, 250.0, 60.0, "ms", MacroRole::Tone, "Grain Overlap Window parameter"))
            .with_param(DspParamSchema::toggle("formant_preserve", "Lock Vocal Formants", true, "Lock Vocal Formants toggle switch"))
        );
        descriptors.insert("PitchCorrectorNode".to_string(), DspNodeDescriptor::new("PitchCorrectorNode", "Scale-Quantized Microtonal Vocal Pitch Corrector", DspNodeCategory::Modulation, "Scale-Quantized Microtonal Vocal Pitch Corrector")
            .with_param(DspParamSchema::choice("scale_mode", "Quantize Tuning Grid", &["12-EDO Equal", "19-EDO Microtonal", "31-EDO Fokker", "Bohlen-Pierce", "Arabic Maqam Bayati"], 0, "Quantize Tuning Grid mode selector"))
            .with_param(DspParamSchema::knob("snap_speed_ms", "Correction Transition Rate", 0.0, 150.0, 20.0, "ms", MacroRole::Tone, "Correction Transition Rate parameter"))
            .with_param(DspParamSchema::knob("correction_strength", "Intonation Pull Amount", 0.0, 1.0, 0.85, "%", MacroRole::Tone, "Intonation Pull Amount parameter"))
            .with_param(DspParamSchema::knob("formant_shift", "Vocal Formant Offset", -6.0, 6.0, 0.0, "st", MacroRole::Tone, "Vocal Formant Offset parameter"))
        );
        descriptors.insert("VocoderMatrixNode".to_string(), DspNodeDescriptor::new("VocoderMatrixNode", "32-Band Spectral Vocoder Analysis & Synthesis Matrix", DspNodeCategory::SpectralResynthesis, "32-Band Spectral Vocoder Analysis & Synthesis Matrix")
            .with_param(DspParamSchema::knob("band_count", "Analysis Filter Bands", 8.0, 32.0, 24.0, "bands", MacroRole::Character, "Analysis Filter Bands integer control"))
            .with_param(DspParamSchema::knob("formant_warp", "Spectral Formant Warp", 0.5, 2.0, 1.0, "x", MacroRole::Tone, "Spectral Formant Warp parameter"))
            .with_param(DspParamSchema::knob("sibilance_thru", "Unvoiced Noise Pass-thru", 0.0, 1.0, 0.35, "%", MacroRole::Tone, "Unvoiced Noise Pass-thru parameter"))
            .with_param(DspParamSchema::knob("carrier_drive", "Carrier Excitation Drive", 1.0, 10.0, 2.5, "x", MacroRole::Tone, "Carrier Excitation Drive parameter"))
        );
        descriptors.insert("GamelanGender".to_string(), DspNodeDescriptor::new("GamelanGender", "Indonesian Gamelan Gendèr Metallophone & Bamboo Tubes", DspNodeCategory::AcousticPhysicalModel, "Indonesian Gamelan Gendèr Metallophone & Bamboo Tubes")
            .with_param(DspParamSchema::knob("mallet_hardness", "Mallet Hardness", 0.05, 1.0, 0.45, "%", MacroRole::Tone, "Mallet Hardness parameter"))
            .with_param(DspParamSchema::knob("ombak_rate", "Ombak Beating Rate", 2.0, 12.0, 6.5, "Hz", MacroRole::Tone, "Ombak Beating Rate parameter"))
            .with_param(DspParamSchema::knob("bronze_thickness", "Bronze Thickness", 3.0, 20.0, 8.0, "mm", MacroRole::Tone, "Bronze Thickness parameter"))
            .with_param(DspParamSchema::knob("bamboo_q", "Bamboo Resonator Q", 5.0, 60.0, 28.0, "", MacroRole::Tone, "Bamboo Resonator Q parameter"))
            .with_param(DspParamSchema::knob("grasp_damping", "Hand Damping (Mipil)", 0.0, 1.0, 0.0, "%", MacroRole::Tone, "Hand Damping (Mipil) parameter"))
        );
        descriptors.insert("HurdyGurdy".to_string(), DspNodeDescriptor::new("HurdyGurdy", "Vielle à Roue Rosin Wheel, Melody Chanters & Chien Buzz", DspNodeCategory::AcousticPhysicalModel, "Vielle à Roue Rosin Wheel, Melody Chanters & Chien Buzz")
            .with_param(DspParamSchema::knob("wheel_speed", "Crank Wheel RPM", 0.0, 200.0, 90.0, "RPM", MacroRole::Tone, "Crank Wheel RPM parameter"))
            .with_param(DspParamSchema::knob("rosin_friction", "Rosin Stick-Slip", 0.1, 1.0, 0.65, "", MacroRole::Tone, "Rosin Stick-Slip parameter"))
            .with_param(DspParamSchema::knob("chien_buzz", "Trompette Chien Buzz", 0.0, 1.0, 0.75, "%", MacroRole::Tone, "Trompette Chien Buzz parameter"))
            .with_param(DspParamSchema::knob("drone_level", "Bourdon Drones", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Bourdon Drones parameter"))
        );
        descriptors.insert("SitarModel".to_string(), DspNodeDescriptor::new("SitarModel", "Indian Sitar, Curved Jawari Bridge & Sympathetic Tarab Bank", DspNodeCategory::AcousticPhysicalModel, "Indian Sitar, Curved Jawari Bridge & Sympathetic Tarab Bank")
            .with_param(DspParamSchema::knob("jawari_curvature", "Jawari Bridge Arc", 0.01, 1.0, 0.35, "", MacroRole::Tone, "Jawari Bridge Arc parameter"))
            .with_param(DspParamSchema::knob("tarab_coupling", "Tarab Resonance", 0.0, 1.0, 0.7, "%", MacroRole::Tone, "Tarab Resonance parameter"))
            .with_param(DspParamSchema::knob("meend_bend", "Meend Microtonal Bend", -7.0, 7.0, 0.0, "st", MacroRole::Tone, "Meend Microtonal Bend parameter"))
            .with_param(DspParamSchema::knob("mizrab_strike", "Mizrab Strike Force", 0.0, 1.0, 0.85, "%", MacroRole::Tone, "Mizrab Strike Force parameter"))
        );
        descriptors.insert("GrandPianoModel".to_string(), DspNodeDescriptor::new("GrandPianoModel", "Concert Grand Piano, Soundboard Coupling & 3 Pedals", DspNodeCategory::AcousticPhysicalModel, "Concert Grand Piano, Soundboard Coupling & 3 Pedals")
            .with_param(DspParamSchema::knob("hammer_hardness", "Felt Hammer Hardness", 0.1, 1.0, 0.6, "%", MacroRole::Tone, "Felt Hammer Hardness parameter"))
            .with_param(DspParamSchema::knob("soundboard", "Soundboard Coupling", 0.0, 1.0, 0.75, "%", MacroRole::Tone, "Soundboard Coupling parameter"))
            .with_param(DspParamSchema::knob("duplex_scale", "Duplex Resonant Scale", 0.0, 1.0, 0.4, "%", MacroRole::Tone, "Duplex Resonant Scale parameter"))
            .with_param(DspParamSchema::knob("sustain_pedal", "Damper Sustain Pedal", 0.0, 1.0, 0.0, "%", MacroRole::Tone, "Damper Sustain Pedal parameter"))
            .with_param(DspParamSchema::knob("una_corda", "Una Corda Soft Pedal", 0.0, 1.0, 0.0, "%", MacroRole::Tone, "Una Corda Soft Pedal parameter"))
        );
        descriptors.insert("ShakuhachiModel".to_string(), DspNodeDescriptor::new("ShakuhachiModel", "Japanese Bamboo Flute & Air-Reed Vortex Aerodynamics", DspNodeCategory::AcousticPhysicalModel, "Japanese Bamboo Flute & Air-Reed Vortex Aerodynamics")
            .with_param(DspParamSchema::log_knob("blowing_pressure", "Blowing Pressure", 100.0, 2500.0, 650.0, "Pa", MacroRole::Tone, "Blowing Pressure logarithmic parameter"))
            .with_param(DspParamSchema::knob("embouchure", "Embouchure Angle", -30.0, 30.0, 0.0, "deg", MacroRole::Tone, "Embouchure Angle parameter"))
            .with_param(DspParamSchema::knob("meri_kari", "Meri/Kari Micro-Pitch", -4.0, 2.0, 0.0, "st", MacroRole::Tone, "Meri/Kari Micro-Pitch parameter"))
            .with_param(DspParamSchema::knob("turbulence", "Air-Reed Turbulence", 0.0, 1.0, 0.25, "%", MacroRole::Tone, "Air-Reed Turbulence parameter"))
        );
        descriptors.insert("PipeOrganModel".to_string(), DspNodeDescriptor::new("PipeOrganModel", "Cathedral Pipe Organ Windchest & Rank Voicing", DspNodeCategory::AcousticPhysicalModel, "Cathedral Pipe Organ Windchest & Rank Voicing")
            .with_param(DspParamSchema::knob("windchest_press", "Windchest Pressure", 400.0, 2000.0, 850.0, "Pa", MacroRole::Tone, "Windchest Pressure parameter"))
            .with_param(DspParamSchema::knob("rank_mixture", "Principal / Flute Mixture", 0.0, 1.0, 0.6, "%", MacroRole::Tone, "Principal / Flute Mixture parameter"))
            .with_param(DspParamSchema::knob("chiff", "Attack Chiff Transient", 0.0, 1.0, 0.35, "%", MacroRole::Tone, "Attack Chiff Transient parameter"))
            .with_param(DspParamSchema::knob("tremulant", "Tremulant LFO Rate", 0.0, 10.0, 4.2, "Hz", MacroRole::Tone, "Tremulant LFO Rate parameter"))
        );
        descriptors.insert("ClavinetModel".to_string(), DspNodeDescriptor::new("ClavinetModel", "Electromagnetic Rock Clavinet & Rubber Pluck Hammer", DspNodeCategory::AcousticPhysicalModel, "Electromagnetic Rock Clavinet & Rubber Pluck Hammer")
            .with_param(DspParamSchema::knob("pluck_pos", "Pluck Position", 0.05, 0.95, 0.15, "%", MacroRole::Tone, "Pluck Position parameter"))
            .with_param(DspParamSchema::knob("rubber_damper", "Rubber Damper Pad", 0.0, 1.0, 0.3, "%", MacroRole::Tone, "Rubber Damper Pad parameter"))
            .with_param(DspParamSchema::choice("pickups", "Pickup Select", &["Neck (A)", "Bridge (B)", "Both (A+B)", "Phase Inv (A-B)"], 2, "Pickup Select mode selector"))
            .with_param(DspParamSchema::knob("snap", "Hammer Snap Tension", 0.0, 1.0, 0.7, "%", MacroRole::Tone, "Hammer Snap Tension parameter"))
        );
        descriptors.insert("ElectricPianoModel".to_string(), DspNodeDescriptor::new("ElectricPianoModel", "Tine Rhodes / Wurlitzer Reed Resonator", DspNodeCategory::AcousticPhysicalModel, "Tine Rhodes / Wurlitzer Reed Resonator")
            .with_param(DspParamSchema::knob("tine_coupling", "Tine-Tonebar Coupling", 0.0, 1.0, 0.65, "%", MacroRole::Tone, "Tine-Tonebar Coupling parameter"))
            .with_param(DspParamSchema::knob("pickup_dist", "Pickup Air Gap Distance", 0.5, 8.0, 2.0, "mm", MacroRole::Tone, "Pickup Air Gap Distance parameter"))
            .with_param(DspParamSchema::knob("bell_decay", "Tine Bell Decay", 0.1, 5.0, 1.8, "s", MacroRole::Tone, "Tine Bell Decay parameter"))
            .with_param(DspParamSchema::knob("drive", "Preamp Overdrive", 0.0, 12.0, 2.5, "dB", MacroRole::Tone, "Preamp Overdrive parameter"))
        );
        descriptors.insert("GlassArmonica".to_string(), DspNodeDescriptor::new("GlassArmonica", "Franklin Glass Armonica Friction Model", DspNodeCategory::AcousticPhysicalModel, "Franklin Glass Armonica Friction Model")
            .with_param(DspParamSchema::knob("finger_press", "Finger Contact Force", 0.05, 1.0, 0.4, "%", MacroRole::Tone, "Finger Contact Force parameter"))
            .with_param(DspParamSchema::knob("spindle_rpm", "Spindle Speed", 10.0, 120.0, 55.0, "RPM", MacroRole::Tone, "Spindle Speed parameter"))
            .with_param(DspParamSchema::knob("water_damp", "Water Level Damping", 0.0, 1.0, 0.2, "%", MacroRole::Tone, "Water Level Damping parameter"))
            .with_param(DspParamSchema::knob("bowl_q", "Crystal Bowl Q Factor", 10.0, 100.0, 45.0, "", MacroRole::Tone, "Crystal Bowl Q Factor parameter"))
        );
        descriptors.insert("BowedStringModel".to_string(), DspNodeDescriptor::new("BowedStringModel", "Stradivarius Violin / Cello Bow Friction", DspNodeCategory::AcousticPhysicalModel, "Stradivarius Violin / Cello Bow Friction")
            .with_param(DspParamSchema::knob("bow_force", "Bow Pressure Force", 0.05, 1.0, 0.55, "%", MacroRole::Tone, "Bow Pressure Force parameter"))
            .with_param(DspParamSchema::knob("bow_vel", "Bow Stroke Velocity", 0.01, 2.0, 0.35, "m/s", MacroRole::Tone, "Bow Stroke Velocity parameter"))
            .with_param(DspParamSchema::knob("bow_pos", "Bow Contact Point", 0.05, 0.5, 0.12, "pos", MacroRole::Tone, "Bow Contact Point parameter"))
            .with_param(DspParamSchema::knob("body_res", "Spruce Body Cavity", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Spruce Body Cavity parameter"))
        );
        descriptors.insert("SteelpanModel".to_string(), DspNodeDescriptor::new("SteelpanModel", "Trinidad Steelpan Drum Membrane Shell", DspNodeCategory::AcousticPhysicalModel, "Trinidad Steelpan Drum Membrane Shell")
            .with_param(DspParamSchema::knob("strike_vel", "Rubber Stick Strike", 0.0, 1.0, 0.7, "%", MacroRole::Tone, "Rubber Stick Strike parameter"))
            .with_param(DspParamSchema::knob("shell_depth", "Oil Drum Shell Skirt", 5.0, 40.0, 18.0, "cm", MacroRole::Tone, "Oil Drum Shell Skirt parameter"))
            .with_param(DspParamSchema::knob("ring_res", "Overtone Ring Sympathy", 0.0, 1.0, 0.6, "%", MacroRole::Tone, "Overtone Ring Sympathy parameter"))
            .with_param(DspParamSchema::knob("damping", "Perimeter Damping", 0.0, 1.0, 0.15, "%", MacroRole::Tone, "Perimeter Damping parameter"))
        );
        descriptors.insert("TurkishNeyModel".to_string(), DspNodeDescriptor::new("TurkishNeyModel", "Turkish Ney End-Blown Cane Flute", DspNodeCategory::AcousticPhysicalModel, "Turkish Ney End-Blown Cane Flute")
            .with_param(DspParamSchema::knob("angle", "Başpare Mouthpiece Angle", -25.0, 25.0, 0.0, "deg", MacroRole::Tone, "Başpare Mouthpiece Angle parameter"))
            .with_param(DspParamSchema::knob("lip_aperture", "Lip Aperture Width", 0.5, 5.0, 2.0, "mm", MacroRole::Tone, "Lip Aperture Width parameter"))
            .with_param(DspParamSchema::choice("octave", "Register Mode", &["Rast (Low)", "Neva (Mid)", "Tiz (High)"], 1, "Register Mode mode selector"))
            .with_param(DspParamSchema::knob("vortex", "Aerodynamic Vortex Blend", 0.0, 1.0, 0.3, "%", MacroRole::Tone, "Aerodynamic Vortex Blend parameter"))
        );
        descriptors.insert("WaveguideBrassModel".to_string(), DspNodeDescriptor::new("WaveguideBrassModel", "Waveguide Lip-Reed Acoustic Brass & Flare", DspNodeCategory::AcousticPhysicalModel, "Waveguide Lip-Reed Acoustic Brass & Flare")
            .with_param(DspParamSchema::knob("lip_tension", "Lip Reed Tension", 0.1, 1.0, 0.5, "%", MacroRole::Tone, "Lip Reed Tension parameter"))
            .with_param(DspParamSchema::log_knob("mouth_press", "Mouth Pressure", 200.0, 5000.0, 1200.0, "Pa", MacroRole::Tone, "Mouth Pressure logarithmic parameter"))
            .with_param(DspParamSchema::knob("bell_flare", "Bell Flare Impedance", 0.1, 2.0, 1.0, "x", MacroRole::Tone, "Bell Flare Impedance parameter"))
            .with_param(DspParamSchema::choice("mute", "Acoustic Mute", &["Open Bell", "Straight Mute", "Harmon / Wah", "Cup Mute"], 0, "Acoustic Mute mode selector"))
        );
        descriptors.insert("WoodwindJetModel".to_string(), DspNodeDescriptor::new("WoodwindJetModel", "Flute Jet Instability & Aerodynamic Vortex", DspNodeCategory::AcousticPhysicalModel, "Flute Jet Instability & Aerodynamic Vortex")
            .with_param(DspParamSchema::knob("jet_delay", "Jet Propagation Delay", 0.1, 10.0, 1.2, "ms", MacroRole::Tone, "Jet Propagation Delay parameter"))
            .with_param(DspParamSchema::knob("jet_gain", "Vortex Non-Linear Gain", 0.1, 5.0, 1.8, "", MacroRole::Tone, "Vortex Non-Linear Gain parameter"))
            .with_param(DspParamSchema::knob("breath_noise", "Turbulent Breath Noise", 0.0, 1.0, 0.18, "%", MacroRole::Tone, "Turbulent Breath Noise parameter"))
        );
        descriptors.insert("SpringLatticeNode".to_string(), DspNodeDescriptor::new("SpringLatticeNode", "Non-linear Spring Reverb Lattice & Dispersion", DspNodeCategory::TimeSpace, "Non-linear Spring Reverb Lattice & Dispersion")
            .with_param(DspParamSchema::knob("tension", "Spring Lattice Tension", 0.1, 1.0, 0.65, "%", MacroRole::Tone, "Spring Lattice Tension parameter"))
            .with_param(DspParamSchema::knob("spring_count", "Coupled Springs", 2.0, 12.0, 6.0, "springs", MacroRole::Character, "Coupled Springs integer control"))
            .with_param(DspParamSchema::knob("dispersion", "Chirp Dispersion Rate", 0.0, 1.0, 0.55, "%", MacroRole::Tone, "Chirp Dispersion Rate parameter"))
            .with_param(DspParamSchema::knob("decay_t60", "Decay Time (T60)", 0.5, 15.0, 3.5, "s", MacroRole::Tone, "Decay Time (T60) parameter"))
        );
        descriptors.insert("PlateTankNode".to_string(), DspNodeDescriptor::new("PlateTankNode", "EMT 140 Plate Reverb Tank & Transducer Dispersion", DspNodeCategory::TimeSpace, "EMT 140 Plate Reverb Tank & Transducer Dispersion")
            .with_param(DspParamSchema::knob("tension", "Steel Plate Tension", 0.1, 1.0, 0.8, "%", MacroRole::Tone, "Steel Plate Tension parameter"))
            .with_param(DspParamSchema::knob("damping_pad", "Felt Damper Distance", 0.0, 1.0, 0.35, "%", MacroRole::Tone, "Felt Damper Distance parameter"))
            .with_param(DspParamSchema::knob("pickup_spread", "Stereo Pickup Spread", 0.0, 1.0, 0.85, "%", MacroRole::Tone, "Stereo Pickup Spread parameter"))
            .with_param(DspParamSchema::knob("decay_t60", "Reverberation Time", 0.5, 20.0, 4.2, "s", MacroRole::Tone, "Reverberation Time parameter"))
        );
        descriptors.insert("TonewheelOrganModel".to_string(), DspNodeDescriptor::new("TonewheelOrganModel", "Hammond B3 Tonewheel Organ & 9 Drawbars", DspNodeCategory::AcousticPhysicalModel, "Hammond B3 Tonewheel Organ & 9 Drawbars")
            .with_param(DspParamSchema::knob("drawbar_16", "Sub Drawbar 16'", 0.0, 8.0, 8.0, "", MacroRole::Character, "Sub Drawbar 16' integer control"))
            .with_param(DspParamSchema::knob("drawbar_8", "Fund Drawbar 8'", 0.0, 8.0, 8.0, "", MacroRole::Character, "Fund Drawbar 8' integer control"))
            .with_param(DspParamSchema::knob("drawbar_4", "Harm Drawbar 4'", 0.0, 8.0, 6.0, "", MacroRole::Character, "Harm Drawbar 4' integer control"))
            .with_param(DspParamSchema::knob("drawbar_2", "Block Drawbar 2'", 0.0, 8.0, 4.0, "", MacroRole::Character, "Block Drawbar 2' integer control"))
            .with_param(DspParamSchema::knob("key_click", "Mechanical Key Click", 0.0, 1.0, 0.4, "%", MacroRole::Tone, "Mechanical Key Click parameter"))
            .with_param(DspParamSchema::choice("vibrato", "Scanner Vibrato/Chorus", &["Off", "V1", "C1", "V2", "C2", "V3", "C3"], 5, "Scanner Vibrato/Chorus mode selector"))
        );
        descriptors.insert("VocalTractNode".to_string(), DspNodeDescriptor::new("VocalTractNode", "Kelly-Lochbaum Acoustic Vocal Tract & Formants", DspNodeCategory::AcousticPhysicalModel, "Kelly-Lochbaum Acoustic Vocal Tract & Formants")
            .with_param(DspParamSchema::log_knob("pitch", "Glottal Pitch", 50.0, 1000.0, 220.0, "Hz", MacroRole::Tone, "Glottal Pitch logarithmic parameter"))
            .with_param(DspParamSchema::knob("tract_len", "Vocal Tract Length", 12.0, 22.0, 17.5, "cm", MacroRole::Tone, "Vocal Tract Length parameter"))
            .with_param(DspParamSchema::knob("lip_area", "Lip Area Opening", 0.1, 5.0, 1.5, "cm2", MacroRole::Tone, "Lip Area Opening parameter"))
            .with_param(DspParamSchema::knob("tongue_pos", "Tongue Height / Position", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Tongue Height / Position parameter"))
            .with_param(DspParamSchema::knob("nasal", "Velum Nasal Cavity", 0.0, 1.0, 0.0, "%", MacroRole::Tone, "Velum Nasal Cavity parameter"))
        );
        descriptors.insert("BambooResonatorBank".to_string(), DspNodeDescriptor::new("BambooResonatorBank", "Coupled Bamboo Air-Column Tube Resonators", DspNodeCategory::AcousticPhysicalModel, "Coupled Bamboo Air-Column Tube Resonators")
            .with_param(DspParamSchema::log_knob("tube_tuning_hz", "Tube Resonant Freq", 50.0, 2000.0, 261.63, "Hz", MacroRole::Tone, "Tube Resonant Freq logarithmic parameter"))
            .with_param(DspParamSchema::knob("cavity_q", "Air Column Q Factor", 5.0, 60.0, 30.0, "Q", MacroRole::Tone, "Air Column Q Factor parameter"))
            .with_param(DspParamSchema::knob("air_column_len", "Bamboo Tube Length", 5.0, 80.0, 25.0, "cm", MacroRole::Tone, "Bamboo Tube Length parameter"))
            .with_param(DspParamSchema::knob("coupling", "Acoustic Air Coupling", 0.0, 1.0, 0.75, "%", MacroRole::Tone, "Acoustic Air Coupling parameter"))
        );
        descriptors.insert("MizrabPluckModel".to_string(), DspNodeDescriptor::new("MizrabPluckModel", "Wire Plectrum Transient Strike & Release Model", DspNodeCategory::AcousticPhysicalModel, "Wire Plectrum Transient Strike & Release Model")
            .with_param(DspParamSchema::knob("plectrum_angle", "Pluck Angle", -45.0, 45.0, 15.0, "deg", MacroRole::Tone, "Pluck Angle parameter"))
            .with_param(DspParamSchema::knob("stroke_speed", "Stroke Velocity", 0.1, 5.0, 1.5, "m/s", MacroRole::Tone, "Stroke Velocity parameter"))
            .with_param(DspParamSchema::knob("wire_gauge", "Wire Gauge Thickness", 0.1, 1.0, 0.3, "mm", MacroRole::Tone, "Wire Gauge Thickness parameter"))
            .with_param(DspParamSchema::knob("damping", "Post-Strike Damping", 0.0, 1.0, 0.1, "%", MacroRole::Tone, "Post-Strike Damping parameter"))
        );
        descriptors.insert("JawariBridgeModel".to_string(), DspNodeDescriptor::new("JawariBridgeModel", "Curved Flat Jawari Bridge Grazing Dynamics", DspNodeCategory::AcousticPhysicalModel, "Curved Flat Jawari Bridge Grazing Dynamics")
            .with_param(DspParamSchema::knob("bridge_arc_radius", "Bridge Curvature Radius", 0.01, 1.0, 0.35, "m", MacroRole::Tone, "Bridge Curvature Radius parameter"))
            .with_param(DspParamSchema::knob("string_clearance", "String-to-Bridge Gap", 0.1, 3.0, 0.8, "mm", MacroRole::Tone, "String-to-Bridge Gap parameter"))
            .with_param(DspParamSchema::knob("buzz_decay", "Buzz Transient Decay", 0.1, 5.0, 1.5, "s", MacroRole::Tone, "Buzz Transient Decay parameter"))
            .with_param(DspParamSchema::knob("harmonic_bleed", "Harmonic Cascade Bleed", 0.0, 1.0, 0.6, "%", MacroRole::Tone, "Harmonic Cascade Bleed parameter"))
        );
        descriptors.insert("SoundboardBridgeModel".to_string(), DspNodeDescriptor::new("SoundboardBridgeModel", "Spruce Soundboard & Rib Stiffness Impedance Bridge", DspNodeCategory::AcousticPhysicalModel, "Spruce Soundboard & Rib Stiffness Impedance Bridge")
            .with_param(DspParamSchema::choice("soundboard_wood", "Soundboard Tonewood", &["Sitka Spruce", "European Spruce", "Red Cedar", "Mahogany"], 0, "Soundboard Tonewood mode selector"))
            .with_param(DspParamSchema::knob("bridge_impedance", "Bridge Driving Impedance", 0.1, 5.0, 1.2, "x", MacroRole::Tone, "Bridge Driving Impedance parameter"))
            .with_param(DspParamSchema::knob("rib_stiffness", "Spruce Rib Stiffness", 0.1, 3.0, 1.0, "x", MacroRole::Tone, "Spruce Rib Stiffness parameter"))
            .with_param(DspParamSchema::knob("decay_loss", "Soundboard Radiation Loss", 0.01, 0.5, 0.08, "", MacroRole::Tone, "Soundboard Radiation Loss parameter"))
        );
        descriptors.insert("TineResonatorModel".to_string(), DspNodeDescriptor::new("TineResonatorModel", "Tuned Spring Steel Tine & Tonebar Resonator", DspNodeCategory::AcousticPhysicalModel, "Tuned Spring Steel Tine & Tonebar Resonator")
            .with_param(DspParamSchema::knob("tine_length", "Steel Tine Length", 2.0, 20.0, 8.5, "cm", MacroRole::Tone, "Steel Tine Length parameter"))
            .with_param(DspParamSchema::knob("tonebar_mass", "Tonebar Cast Mass", 10.0, 200.0, 65.0, "g", MacroRole::Tone, "Tonebar Cast Mass parameter"))
            .with_param(DspParamSchema::knob("magnetic_gap", "Pickup Air Gap", 0.5, 8.0, 2.0, "mm", MacroRole::Tone, "Pickup Air Gap parameter"))
            .with_param(DspParamSchema::knob("reed_clack", "Hammer Reed Impact", 0.0, 1.0, 0.35, "%", MacroRole::Tone, "Hammer Reed Impact parameter"))
        );
        descriptors.insert("TrompetteChienModel".to_string(), DspNodeDescriptor::new("TrompetteChienModel", "Vielle Trompette Buzzing Dog Bridge Mechanism", DspNodeCategory::AcousticPhysicalModel, "Vielle Trompette Buzzing Dog Bridge Mechanism")
            .with_param(DspParamSchema::knob("chien_pressure", "Trompette String Tension", 0.05, 1.0, 0.6, "%", MacroRole::Tone, "Trompette String Tension parameter"))
            .with_param(DspParamSchema::knob("bridge_lift", "Loose Foot Bridge Lift", 0.1, 2.0, 0.5, "mm", MacroRole::Tone, "Loose Foot Bridge Lift parameter"))
            .with_param(DspParamSchema::knob("buzz_threshold", "Crank Acceleration Threshold", 0.01, 0.5, 0.12, "g", MacroRole::Tone, "Crank Acceleration Threshold parameter"))
            .with_param(DspParamSchema::knob("rosin_drag", "Wheel Rosin Viscosity", 0.1, 1.0, 0.45, "%", MacroRole::Tone, "Wheel Rosin Viscosity parameter"))
        );
        descriptors.insert("WindchestAerodynamicsModel".to_string(), DspNodeDescriptor::new("WindchestAerodynamicsModel", "Pipe Organ Windchest Pallet Valve & Reservoir", DspNodeCategory::AcousticPhysicalModel, "Pipe Organ Windchest Pallet Valve & Reservoir")
            .with_param(DspParamSchema::knob("reservoir_pressure", "Bellows Pressure", 400.0, 3000.0, 950.0, "Pa", MacroRole::Tone, "Bellows Pressure parameter"))
            .with_param(DspParamSchema::knob("valve_inertia", "Pallet Valve Inertia", 0.01, 0.5, 0.08, "s", MacroRole::Tone, "Pallet Valve Inertia parameter"))
            .with_param(DspParamSchema::knob("turbulence_vortex", "Channel Turbulence", 0.0, 1.0, 0.25, "%", MacroRole::Tone, "Channel Turbulence parameter"))
            .with_param(DspParamSchema::knob("pipe_drag", "Pipe Foot Flow Drag", 0.01, 0.5, 0.1, "", MacroRole::Tone, "Pipe Foot Flow Drag parameter"))
        );
        descriptors.insert("AirReedAcousticModel".to_string(), DspNodeDescriptor::new("AirReedAcousticModel", "Aeroacoustic Air-Reed Jet Stream Instability", DspNodeCategory::AcousticPhysicalModel, "Aeroacoustic Air-Reed Jet Stream Instability")
            .with_param(DspParamSchema::knob("blowing_velocity", "Jet Velocity", 5.0, 60.0, 25.0, "m/s", MacroRole::Tone, "Jet Velocity parameter"))
            .with_param(DspParamSchema::knob("jet_angle", "Flue Jet Strike Angle", -30.0, 30.0, 0.0, "deg", MacroRole::Tone, "Flue Jet Strike Angle parameter"))
            .with_param(DspParamSchema::knob("lip_distance", "Utaguchi Lip Distance", 1.0, 15.0, 4.5, "mm", MacroRole::Tone, "Utaguchi Lip Distance parameter"))
            .with_param(DspParamSchema::knob("edge_vortex_gain", "Vortex Splitting Gain", 0.1, 3.0, 1.2, "", MacroRole::Tone, "Vortex Splitting Gain parameter"))
        );
        descriptors.insert("DigitalWaveguideNode".to_string(), DspNodeDescriptor::new("DigitalWaveguideNode", "Bidirectional Delay-Line Digital Waveguide Core", DspNodeCategory::AcousticPhysicalModel, "Bidirectional Delay-Line Digital Waveguide Core")
            .with_param(DspParamSchema::knob("delay_line_len", "Waveguide Length Delay", 0.5, 50.0, 5.0, "ms", MacroRole::Tone, "Waveguide Length Delay parameter"))
            .with_param(DspParamSchema::log_knob("loss_filter_cutoff", "Loss Filter High Cut", 500.0, 20000.0, 6000.0, "Hz", MacroRole::Tone, "Loss Filter High Cut logarithmic parameter"))
            .with_param(DspParamSchema::knob("dispersion_allpass", "Dispersion Allpass", -0.9, 0.9, 0.3, "", MacroRole::Tone, "Dispersion Allpass parameter"))
            .with_param(DspParamSchema::knob("pickup_pos", "Acoustic Pickup Position", 0.01, 0.99, 0.2, "%", MacroRole::Tone, "Acoustic Pickup Position parameter"))
        );
        descriptors.insert("WaveguideMesh2D".to_string(), DspNodeDescriptor::new("WaveguideMesh2D", "2D Triangular Mesh Wave Propagation Surface", DspNodeCategory::AcousticPhysicalModel, "2D Triangular Mesh Wave Propagation Surface")
            .with_param(DspParamSchema::knob("grid_damping", "Mesh Damping Loss", 0.001, 0.1, 0.01, "", MacroRole::Tone, "Mesh Damping Loss parameter"))
            .with_param(DspParamSchema::knob("tension", "Membrane Surface Tension", 0.1, 1.0, 0.7, "%", MacroRole::Tone, "Membrane Surface Tension parameter"))
            .with_param(DspParamSchema::knob("boundary_reflection", "Boundary Reflection", 0.5, 0.999, 0.95, "%", MacroRole::Tone, "Boundary Reflection parameter"))
            .with_param(DspParamSchema::knob("strike_point", "Impulse Strike Point", 0.0, 1.0, 0.5, "pos", MacroRole::Tone, "Impulse Strike Point parameter"))
        );
        descriptors.insert("RotarySpeakerModel".to_string(), DspNodeDescriptor::new("RotarySpeakerModel", "Leslie Dual-Rotor Doppler & Cabinet Diffraction", DspNodeCategory::AcousticPhysicalModel, "Leslie Dual-Rotor Doppler & Cabinet Diffraction")
            .with_param(DspParamSchema::knob("horn_rpm", "Treble Horn RPM", 0.0, 450.0, 380.0, "RPM", MacroRole::Tone, "Treble Horn RPM parameter"))
            .with_param(DspParamSchema::knob("drum_rpm", "Bass Drum RPM", 0.0, 400.0, 340.0, "RPM", MacroRole::Tone, "Bass Drum RPM parameter"))
            .with_param(DspParamSchema::knob("doppler_depth", "Doppler Pitch Shift Depth", 0.0, 1.0, 0.7, "%", MacroRole::Tone, "Doppler Pitch Shift Depth parameter"))
            .with_param(DspParamSchema::log_knob("crossover_hz", "Cabinet Crossover", 200.0, 2000.0, 800.0, "Hz", MacroRole::Tone, "Cabinet Crossover logarithmic parameter"))
            .with_param(DspParamSchema::knob("drive", "Tube Preamp Saturation", 0.0, 12.0, 3.0, "dB", MacroRole::Tone, "Tube Preamp Saturation parameter"))
        );
        descriptors.insert("StruckIdiophoneResonatorNode".to_string(), DspNodeDescriptor::new("StruckIdiophoneResonatorNode", "Xylophone/Marimba Tuned Bar Modal Resonator", DspNodeCategory::AcousticPhysicalModel, "Xylophone/Marimba Tuned Bar Modal Resonator")
            .with_param(DspParamSchema::choice("bar_material", "Bar Material", &["Rosewood (Marimba)", "Aluminum (Vibraphone)", "Bronze (Glockenspiel)", "Synthetic Polymer"], 0, "Bar Material mode selector"))
            .with_param(DspParamSchema::knob("modal_density", "Active Modal Partials", 2.0, 16.0, 6.0, "modes", MacroRole::Character, "Active Modal Partials integer control"))
            .with_param(DspParamSchema::knob("decay_t60", "Bar Ring Decay Time", 0.2, 10.0, 2.5, "s", MacroRole::Tone, "Bar Ring Decay Time parameter"))
            .with_param(DspParamSchema::knob("strike_location", "Mallet Strike Node", 0.05, 0.95, 0.25, "pos", MacroRole::Tone, "Mallet Strike Node parameter"))
        );
        descriptors.insert("MembranePercussionModel".to_string(), DspNodeDescriptor::new("MembranePercussionModel", "2D Drumhead Membrane & Air Cavity Coupling", DspNodeCategory::AcousticPhysicalModel, "2D Drumhead Membrane & Air Cavity Coupling")
            .with_param(DspParamSchema::knob("head_tension", "Drumhead Lug Tension", 0.1, 1.0, 0.7, "%", MacroRole::Tone, "Drumhead Lug Tension parameter"))
            .with_param(DspParamSchema::knob("rim_shot_coupling", "Rim-to-Center Coupling", 0.0, 1.0, 0.4, "%", MacroRole::Tone, "Rim-to-Center Coupling parameter"))
            .with_param(DspParamSchema::knob("air_cavity_depth", "Shell Air Cavity Depth", 5.0, 50.0, 20.0, "cm", MacroRole::Tone, "Shell Air Cavity Depth parameter"))
            .with_param(DspParamSchema::knob("snare_buzz", "Bottom Snare Wire Buzz", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Bottom Snare Wire Buzz parameter"))
        );
        descriptors.insert("GlottalPulseNode".to_string(), DspNodeDescriptor::new("GlottalPulseNode", "Rosenberg Glottal Flow Waveform Model", DspNodeCategory::AcousticPhysicalModel, "Rosenberg Glottal Flow Waveform Model")
            .with_param(DspParamSchema::log_knob("f0_pitch", "Vocal Pitch F0", 40.0, 1000.0, 130.0, "Hz", MacroRole::Tone, "Vocal Pitch F0 logarithmic parameter"))
            .with_param(DspParamSchema::knob("open_quotient", "Glottal Open Quotient (OQ)", 0.3, 0.9, 0.6, "", MacroRole::Tone, "Glottal Open Quotient (OQ) parameter"))
            .with_param(DspParamSchema::knob("asymmetry_coeff", "Glottal Asymmetry", 0.5, 0.95, 0.75, "", MacroRole::Tone, "Glottal Asymmetry parameter"))
            .with_param(DspParamSchema::knob("aspiration_noise", "Turbulent Aspiration Noise", 0.0, 1.0, 0.1, "%", MacroRole::Tone, "Turbulent Aspiration Noise parameter"))
        );
        descriptors.insert("HammerStrikeModel".to_string(), DspNodeDescriptor::new("HammerStrikeModel", "Non-linear Felt Hammer Strike Dynamics", DspNodeCategory::AcousticPhysicalModel, "Non-linear Felt Hammer Strike Dynamics")
            .with_param(DspParamSchema::knob("hammer_mass", "Hammer Head Mass", 1.0, 20.0, 8.0, "g", MacroRole::Tone, "Hammer Head Mass parameter"))
            .with_param(DspParamSchema::knob("felt_elasticity", "Felt Non-linear Exponent", 0.1, 5.0, 1.8, "p", MacroRole::Tone, "Felt Non-linear Exponent parameter"))
            .with_param(DspParamSchema::knob("strike_velocity", "Key Strike Velocity", 0.1, 10.0, 3.5, "m/s", MacroRole::Tone, "Key Strike Velocity parameter"))
            .with_param(DspParamSchema::knob("contact_duration", "String Contact Duration", 0.5, 10.0, 2.5, "ms", MacroRole::Tone, "String Contact Duration parameter"))
        );
        descriptors.insert("ToneholeGridModel".to_string(), DspNodeDescriptor::new("ToneholeGridModel", "Woodwind Tonehole Lattice & Acoustic Radiation", DspNodeCategory::AcousticPhysicalModel, "Woodwind Tonehole Lattice & Acoustic Radiation")
            .with_param(DspParamSchema::knob("open_holes", "Open Finger Holes", 0.0, 12.0, 4.0, "holes", MacroRole::Character, "Open Finger Holes integer control"))
            .with_param(DspParamSchema::toggle("register_key", "Register / Speaker Key Active", false, "Register / Speaker Key Active toggle switch"))
            .with_param(DspParamSchema::knob("lattice_impedance", "Tonehole Lattice Impedance", 0.1, 3.0, 1.0, "x", MacroRole::Tone, "Tonehole Lattice Impedance parameter"))
            .with_param(DspParamSchema::knob("bell_radiation", "End Bell Radiation", 0.1, 2.0, 0.8, "x", MacroRole::Tone, "End Bell Radiation parameter"))
        );
        descriptors.insert("FrictionModel".to_string(), DspNodeDescriptor::new("FrictionModel", "Stick-Slip Rosin Tribological Friction Model", DspNodeCategory::AcousticPhysicalModel, "Stick-Slip Rosin Tribological Friction Model")
            .with_param(DspParamSchema::knob("normal_force", "Bow / Rosin Normal Force", 0.01, 5.0, 1.0, "N", MacroRole::Tone, "Bow / Rosin Normal Force parameter"))
            .with_param(DspParamSchema::knob("relative_velocity", "Relative Velocity", 0.001, 2.0, 0.2, "m/s", MacroRole::Tone, "Relative Velocity parameter"))
            .with_param(DspParamSchema::knob("static_friction", "Static Friction Coeff", 0.1, 1.5, 0.8, "", MacroRole::Tone, "Static Friction Coeff parameter"))
            .with_param(DspParamSchema::knob("dynamic_friction", "Dynamic Friction Coeff", 0.05, 1.0, 0.4, "", MacroRole::Tone, "Dynamic Friction Coeff parameter"))
        );
        descriptors.insert("ModalResonator".to_string(), DspNodeDescriptor::new("ModalResonator", "Universal Multi-Modal Resonator Filter Bank", DspNodeCategory::AcousticPhysicalModel, "Universal Multi-Modal Resonator Filter Bank")
            .with_param(DspParamSchema::log_knob("freq", "Modal Base Frequency", 50.0, 8000.0, 440.0, "Hz", MacroRole::Tone, "Modal Base Frequency logarithmic parameter"))
            .with_param(DspParamSchema::knob("q_factor", "Modal Q Factor", 1.0, 100.0, 30.0, "Q", MacroRole::Tone, "Modal Q Factor parameter"))
            .with_param(DspParamSchema::knob("decay_s", "Resonance Decay Time", 0.1, 10.0, 2.0, "s", MacroRole::Tone, "Resonance Decay Time parameter"))
            .with_param(DspParamSchema::choice("structure", "Modal Partials Spectrum", &["Harmonic String", "Inharmonic Bar", "Circular Membrane", "Stiff Beam"], 0, "Modal Partials Spectrum mode selector"))
        );
        descriptors.insert("DulcimerCimbalomModel".to_string(), DspNodeDescriptor::new("DulcimerCimbalomModel", "Hammered Dulcimer & Cimbalom Wire Resonator", DspNodeCategory::AcousticPhysicalModel, "Hammered Dulcimer & Cimbalom Wire Resonator")
            .with_param(DspParamSchema::knob("hammer_felt_hardness", "Plectrum/Felt Hardness", 0.1, 1.0, 0.65, "%", MacroRole::Tone, "Plectrum/Felt Hardness parameter"))
            .with_param(DspParamSchema::knob("course_unison_detune", "Course Unison Detune", 0.0, 25.0, 3.5, "cents", MacroRole::Tone, "Course Unison Detune parameter"))
            .with_param(DspParamSchema::knob("bridge_soundboard_gain", "Maple Bridge Coupling", 0.1, 3.0, 1.2, "x", MacroRole::Tone, "Maple Bridge Coupling parameter"))
            .with_param(DspParamSchema::knob("damping_pedal", "Damper Bar Attenuation", 0.0, 1.0, 0.0, "%", MacroRole::Tone, "Damper Bar Attenuation parameter"))
        );
        descriptors.insert("MbiraKalimbaModel".to_string(), DspNodeDescriptor::new("MbiraKalimbaModel", "African Mbira / Kalimba Plucked Lamellophone", DspNodeCategory::AcousticPhysicalModel, "African Mbira / Kalimba Plucked Lamellophone")
            .with_param(DspParamSchema::knob("tongue_length_mm", "Spring Steel Key Length", 30.0, 120.0, 65.0, "mm", MacroRole::Tone, "Spring Steel Key Length parameter"))
            .with_param(DspParamSchema::knob("gourd_resonator_q", "Calabash Gourd Resonator Q", 5.0, 50.0, 22.0, "Q", MacroRole::Tone, "Calabash Gourd Resonator Q parameter"))
            .with_param(DspParamSchema::knob("bottlecap_buzz", "Bottlecap Jingler Buzz", 0.0, 1.0, 0.45, "%", MacroRole::Tone, "Bottlecap Jingler Buzz parameter"))
            .with_param(DspParamSchema::knob("pluck_velocity", "Thumb Pluck Force", 0.1, 1.0, 0.7, "%", MacroRole::Tone, "Thumb Pluck Force parameter"))
        );
        descriptors.insert("MembranePlateModel".to_string(), DspNodeDescriptor::new("MembranePlateModel", "Coupled 2D Membrane & Thin Plate Resonator", DspNodeCategory::AcousticPhysicalModel, "Coupled 2D Membrane & Thin Plate Resonator")
            .with_param(DspParamSchema::knob("membrane_tension", "Membrane Radial Tension", 0.1, 1.0, 0.75, "%", MacroRole::Tone, "Membrane Radial Tension parameter"))
            .with_param(DspParamSchema::knob("plate_elastic_modulus", "Brass Plate Stiffness", 0.1, 5.0, 1.8, "GPa", MacroRole::Tone, "Brass Plate Stiffness parameter"))
            .with_param(DspParamSchema::knob("air_gap_coupling", "Air Chamber Coupling", 0.01, 1.0, 0.6, "%", MacroRole::Tone, "Air Chamber Coupling parameter"))
            .with_param(DspParamSchema::knob("edge_damping", "Clamped Rim Damping", 0.0, 1.0, 0.2, "%", MacroRole::Tone, "Clamped Rim Damping parameter"))
        );
        descriptors.insert("MembraneResonatorNode".to_string(), DspNodeDescriptor::new("MembraneResonatorNode", "2D Elastic Drumhead Modal Resonator", DspNodeCategory::AcousticPhysicalModel, "2D Elastic Drumhead Modal Resonator")
            .with_param(DspParamSchema::knob("radial_strike_pos", "Radial Strike Radius", 0.0, 1.0, 0.35, "r/R", MacroRole::Tone, "Radial Strike Radius parameter"))
            .with_param(DspParamSchema::knob("membrane_tension_n", "Radial Skin Tension", 50.0, 2000.0, 450.0, "N/m", MacroRole::Tone, "Radial Skin Tension parameter"))
            .with_param(DspParamSchema::knob("decay_t60_s", "Kettle Decay Time (T60)", 0.2, 8.0, 2.4, "s", MacroRole::Tone, "Kettle Decay Time (T60) parameter"))
            .with_param(DspParamSchema::knob("rim_shot_mix", "Rim Edge Excitation Mix", 0.0, 1.0, 0.15, "%", MacroRole::Tone, "Rim Edge Excitation Mix parameter"))
        );
        descriptors.insert("SympatheticCouplingModel".to_string(), DspNodeDescriptor::new("SympatheticCouplingModel", "Sympathetic String Matrix Energy Coupling", DspNodeCategory::AcousticPhysicalModel, "Sympathetic String Matrix Energy Coupling")
            .with_param(DspParamSchema::knob("string_count", "Sympathetic Drone Strings", 4.0, 16.0, 11.0, "strings", MacroRole::Character, "Sympathetic Drone Strings integer control"))
            .with_param(DspParamSchema::knob("coupling_matrix_k", "Bridge Transfer Matrix (K)", 0.01, 1.0, 0.45, "%", MacroRole::Tone, "Bridge Transfer Matrix (K) parameter"))
            .with_param(DspParamSchema::knob("tarab_resonance_gain", "Tarab Resonance Boost", 0.0, 18.0, 6.0, "dB", MacroRole::Tone, "Tarab Resonance Boost parameter"))
            .with_param(DspParamSchema::knob("decay_time_s", "Sympathetic Ring Time", 0.5, 12.0, 4.5, "s", MacroRole::Tone, "Sympathetic Ring Time parameter"))
        );
        descriptors.insert("CompressorNode".to_string(), DspNodeDescriptor::new("CompressorNode", "VCA Studio Compressor with RMS Detection", DspNodeCategory::DynamicsMaster, "VCA Studio Compressor with RMS Detection")
            .with_param(DspParamSchema::knob("threshold", "Threshold", -48.0, 0.0, -18.0, "dB", MacroRole::Tone, "Threshold parameter"))
            .with_param(DspParamSchema::knob("ratio", "Compression Ratio", 1.0, 20.0, 4.0, ":1", MacroRole::Tone, "Compression Ratio parameter"))
            .with_param(DspParamSchema::log_knob("attack", "Attack Time", 0.1, 200.0, 15.0, "ms", MacroRole::Tone, "Attack Time logarithmic parameter"))
            .with_param(DspParamSchema::log_knob("release", "Release Time", 10.0, 2000.0, 150.0, "ms", MacroRole::Tone, "Release Time logarithmic parameter"))
            .with_param(DspParamSchema::knob("makeup", "Makeup Gain", 0.0, 24.0, 3.0, "dB", MacroRole::Tone, "Makeup Gain parameter"))
            .with_param(DspParamSchema::knob("knee", "Soft Knee Width", 0.0, 18.0, 3.0, "dB", MacroRole::Tone, "Soft Knee Width parameter"))
        );
        descriptors.insert("MultibandCompressorNode".to_string(), DspNodeDescriptor::new("MultibandCompressorNode", "3-Band Linear-Phase Dynamics Processor", DspNodeCategory::DynamicsMaster, "3-Band Linear-Phase Dynamics Processor")
            .with_param(DspParamSchema::knob("low_thresh", "Low Band Threshold", -40.0, 0.0, -18.0, "dB", MacroRole::Tone, "Low Band Threshold parameter"))
            .with_param(DspParamSchema::knob("mid_thresh", "Mid Band Threshold", -40.0, 0.0, -16.0, "dB", MacroRole::Tone, "Mid Band Threshold parameter"))
            .with_param(DspParamSchema::knob("high_thresh", "High Band Threshold", -40.0, 0.0, -14.0, "dB", MacroRole::Tone, "High Band Threshold parameter"))
            .with_param(DspParamSchema::log_knob("low_cross", "Low/Mid Crossover", 60.0, 500.0, 180.0, "Hz", MacroRole::Tone, "Low/Mid Crossover logarithmic parameter"))
            .with_param(DspParamSchema::log_knob("high_cross", "Mid/High Crossover", 1000.0, 10000.0, 3500.0, "Hz", MacroRole::Tone, "Mid/High Crossover logarithmic parameter"))
            .with_param(DspParamSchema::knob("master_gain", "Master Output Trim", -12.0, 12.0, 0.0, "dB", MacroRole::Tone, "Master Output Trim parameter"))
        );
        descriptors.insert("MultibandDynamicsProcessor".to_string(), DspNodeDescriptor::new("MultibandDynamicsProcessor", "Tri-Band Expander/Compressor Dynamics Matrix", DspNodeCategory::DynamicsMaster, "Tri-Band Expander/Compressor Dynamics Matrix")
            .with_param(DspParamSchema::knob("low_comp", "Low Band Compression", 0.0, 100.0, 35.0, "%", MacroRole::Tone, "Low Band Compression parameter"))
            .with_param(DspParamSchema::knob("mid_comp", "Mid Band Compression", 0.0, 100.0, 30.0, "%", MacroRole::Tone, "Mid Band Compression parameter"))
            .with_param(DspParamSchema::knob("high_comp", "High Band Compression", 0.0, 100.0, 25.0, "%", MacroRole::Tone, "High Band Compression parameter"))
            .with_param(DspParamSchema::log_knob("crossover_1", "Low/Mid Crossover", 80.0, 600.0, 200.0, "Hz", MacroRole::Tone, "Low/Mid Crossover logarithmic parameter"))
            .with_param(DspParamSchema::log_knob("crossover_2", "Mid/High Crossover", 1500.0, 8000.0, 4000.0, "Hz", MacroRole::Tone, "Mid/High Crossover logarithmic parameter"))
            .with_param(DspParamSchema::knob("saturation", "Tape Saturation Drive", 0.0, 1.0, 0.2, "%", MacroRole::Tone, "Tape Saturation Drive parameter"))
        );
        descriptors.insert("OpticalCompressorNode".to_string(), DspNodeDescriptor::new("OpticalCompressorNode", "Vintage LA-2A Optical Photocell Dynamics", DspNodeCategory::DynamicsMaster, "Vintage LA-2A Optical Photocell Dynamics")
            .with_param(DspParamSchema::knob("peak_reduct", "Peak Reduction", 0.0, 100.0, 45.0, "%", MacroRole::Tone, "Peak Reduction parameter"))
            .with_param(DspParamSchema::knob("gain", "Output Gain", 0.0, 100.0, 50.0, "%", MacroRole::Tone, "Output Gain parameter"))
            .with_param(DspParamSchema::toggle("limit_mode", "Compress / Limit Mode", false, "Compress / Limit Mode toggle switch"))
        );
        descriptors.insert("NoiseGateNode".to_string(), DspNodeDescriptor::new("NoiseGateNode", "Fast Transient Expander & Noise Gate", DspNodeCategory::DynamicsMaster, "Fast Transient Expander & Noise Gate")
            .with_param(DspParamSchema::knob("threshold", "Gate Threshold", -80.0, 0.0, -45.0, "dB", MacroRole::Tone, "Gate Threshold parameter"))
            .with_param(DspParamSchema::log_knob("attack", "Fast Attack", 0.05, 50.0, 1.0, "ms", MacroRole::Tone, "Fast Attack logarithmic parameter"))
            .with_param(DspParamSchema::knob("hold", "Hold Duration", 0.0, 500.0, 20.0, "ms", MacroRole::Tone, "Hold Duration parameter"))
            .with_param(DspParamSchema::log_knob("release", "Smooth Release", 5.0, 1000.0, 80.0, "ms", MacroRole::Tone, "Smooth Release logarithmic parameter"))
            .with_param(DspParamSchema::knob("range", "Floor Attenuation Range", -80.0, 0.0, -60.0, "dB", MacroRole::Tone, "Floor Attenuation Range parameter"))
        );
        descriptors.insert("DeesserNode".to_string(), DspNodeDescriptor::new("DeesserNode", "Dynamic High-Frequency Sibilance Suppressor", DspNodeCategory::DynamicsMaster, "Dynamic High-Frequency Sibilance Suppressor")
            .with_param(DspParamSchema::knob("threshold", "De-Esser Threshold", -40.0, 0.0, -20.0, "dB", MacroRole::Tone, "De-Esser Threshold parameter"))
            .with_param(DspParamSchema::log_knob("freq", "Sibilance Center Freq", 3000.0, 12000.0, 6500.0, "Hz", MacroRole::Tone, "Sibilance Center Freq logarithmic parameter"))
            .with_param(DspParamSchema::toggle("listen", "Delta Audition Listen", false, "Delta Audition Listen toggle switch"))
            .with_param(DspParamSchema::knob("reduction", "Max Reduction Cap", -24.0, 0.0, -12.0, "dB", MacroRole::Tone, "Max Reduction Cap parameter"))
        );
        descriptors.insert("NeuralTransientShaperNode".to_string(), DspNodeDescriptor::new("NeuralTransientShaperNode", "Deep Neural Network Transient & Sustain Sculptor", DspNodeCategory::NeuralAi, "Deep Neural Network Transient & Sustain Sculptor")
            .with_param(DspParamSchema::knob("attack_boost", "Transient Attack Gain", -12.0, 12.0, 3.0, "dB", MacroRole::Tone, "Transient Attack Gain parameter"))
            .with_param(DspParamSchema::knob("sustain_tail", "Sustain Tail Gain", -12.0, 12.0, -1.5, "dB", MacroRole::Tone, "Sustain Tail Gain parameter"))
            .with_param(DspParamSchema::knob("neural_sensitivity", "AI Detection Sensitivity", 0.0, 1.0, 0.75, "%", MacroRole::Tone, "AI Detection Sensitivity parameter"))
            .with_param(DspParamSchema::knob("lookahead_ms", "Lookahead Buffer", 0.0, 20.0, 5.0, "ms", MacroRole::Tone, "Lookahead Buffer parameter"))
        );
        descriptors.insert("NeuralBassGeneratorNode".to_string(), DspNodeDescriptor::new("NeuralBassGeneratorNode", "Subharmonic Low-End Neural Bass Synthesizer", DspNodeCategory::NeuralAi, "Subharmonic Low-End Neural Bass Synthesizer")
            .with_param(DspParamSchema::knob("sub_octave_gain", "Subharmonic Level", 0.0, 1.0, 0.65, "%", MacroRole::Tone, "Subharmonic Level parameter"))
            .with_param(DspParamSchema::knob("harmonic_density", "Generated Harmonics", 1.0, 8.0, 3.0, "bands", MacroRole::Character, "Generated Harmonics integer control"))
            .with_param(DspParamSchema::choice("saturation_color", "Saturation Flavor", &["Warm Tube", "Solid State", "Tape Flux"], 0, "Saturation Flavor mode selector"))
            .with_param(DspParamSchema::toggle("mono_lows", "Mono Bass Summing (<120Hz)", true, "Mono Bass Summing (<120Hz) toggle switch"))
        );
        descriptors.insert("MultibandClipperNode".to_string(), DspNodeDescriptor::new("MultibandClipperNode", "Tri-Band Mastering Clipper & Saturation Stage", DspNodeCategory::DynamicsMaster, "Tri-Band Mastering Clipper & Saturation Stage")
            .with_param(DspParamSchema::knob("low_clip_ceiling_db", "Low Band Ceiling", -12.0, 0.0, -0.5, "dBFS", MacroRole::Tone, "Low Band Ceiling parameter"))
            .with_param(DspParamSchema::knob("mid_clip_ceiling_db", "Mid Band Ceiling", -12.0, 0.0, -0.3, "dBFS", MacroRole::Tone, "Mid Band Ceiling parameter"))
            .with_param(DspParamSchema::knob("high_clip_ceiling_db", "High Band Ceiling", -12.0, 0.0, -0.1, "dBFS", MacroRole::Tone, "High Band Ceiling parameter"))
            .with_param(DspParamSchema::knob("softness_knee", "Soft Clipping Knee", 0.0, 1.0, 0.35, "%", MacroRole::Tone, "Soft Clipping Knee parameter"))
        );
        descriptors.insert("MultibandDecompressorNode".to_string(), DspNodeDescriptor::new("MultibandDecompressorNode", "Multiband Upward Dynamic Range Decompressor", DspNodeCategory::DynamicsMaster, "Multiband Upward Dynamic Range Decompressor")
            .with_param(DspParamSchema::knob("expansion_ratio", "Upward Expansion Ratio", 1.0, 4.0, 1.5, ":1", MacroRole::Tone, "Upward Expansion Ratio parameter"))
            .with_param(DspParamSchema::knob("upward_threshold_db", "Decompression Threshold", -40.0, -10.0, -24.0, "dB", MacroRole::Tone, "Decompression Threshold parameter"))
            .with_param(DspParamSchema::log_knob("crossover_low_hz", "Low/Mid Split Frequency", 60.0, 500.0, 160.0, "Hz", MacroRole::Tone, "Low/Mid Split Frequency logarithmic parameter"))
            .with_param(DspParamSchema::log_knob("crossover_high_hz", "Mid/High Split Frequency", 1500.0, 10000.0, 4000.0, "Hz", MacroRole::Tone, "Mid/High Split Frequency logarithmic parameter"))
        );
        descriptors.insert("MultibandExpanderNode".to_string(), DspNodeDescriptor::new("MultibandExpanderNode", "Tri-Band Downward Transient Expander", DspNodeCategory::DynamicsMaster, "Tri-Band Downward Transient Expander")
            .with_param(DspParamSchema::knob("downward_ratio", "Expansion Ratio", 1.0, 8.0, 2.0, ":1", MacroRole::Tone, "Expansion Ratio parameter"))
            .with_param(DspParamSchema::knob("gate_threshold_db", "Expansion Floor Gate", -60.0, -10.0, -32.0, "dB", MacroRole::Tone, "Expansion Floor Gate parameter"))
            .with_param(DspParamSchema::knob("range_cap_db", "Max Dynamic Range Cap", -40.0, 0.0, -18.0, "dB", MacroRole::Tone, "Max Dynamic Range Cap parameter"))
            .with_param(DspParamSchema::knob("release_time_ms", "Expander Recovery Time", 10.0, 500.0, 85.0, "ms", MacroRole::Tone, "Expander Recovery Time parameter"))
        );
        descriptors.insert("MultibandSaturatorNode".to_string(), DspNodeDescriptor::new("MultibandSaturatorNode", "3-Band Frequency-Split Saturation Warmer", DspNodeCategory::DynamicsMaster, "3-Band Frequency-Split Saturation Warmer")
            .with_param(DspParamSchema::knob("low_drive_x", "Low Band Tape Warmth", 1.0, 10.0, 2.0, "x", MacroRole::Tone, "Low Band Tape Warmth parameter"))
            .with_param(DspParamSchema::knob("mid_drive_x", "Mid Band Tube Drive", 1.0, 10.0, 3.2, "x", MacroRole::Tone, "Mid Band Tube Drive parameter"))
            .with_param(DspParamSchema::knob("high_drive_x", "High Band Diode Sheen", 1.0, 10.0, 1.8, "x", MacroRole::Tone, "High Band Diode Sheen parameter"))
            .with_param(DspParamSchema::knob("tape_tube_blend", "Tape vs Tube Color Blend", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Tape vs Tube Color Blend parameter"))
        );
        descriptors.insert("UpwardCompressorNode".to_string(), DspNodeDescriptor::new("UpwardCompressorNode", "OTT-Style Upward Audio Compressor", DspNodeCategory::DynamicsMaster, "OTT-Style Upward Audio Compressor")
            .with_param(DspParamSchema::knob("upward_depth", "Upward Compression Depth", 0.0, 100.0, 65.0, "%", MacroRole::Tone, "Upward Compression Depth parameter"))
            .with_param(DspParamSchema::knob("downward_depth", "Downward Compression Depth", 0.0, 100.0, 40.0, "%", MacroRole::Tone, "Downward Compression Depth parameter"))
            .with_param(DspParamSchema::knob("dynamics_speed", "Dynamics Ballistics Speed", 0.1, 10.0, 2.5, "x", MacroRole::Tone, "Dynamics Ballistics Speed parameter"))
            .with_param(DspParamSchema::knob("high_lift_db", "High Air Presence Lift", 0.0, 18.0, 4.0, "dB", MacroRole::Tone, "High Air Presence Lift parameter"))
        );
        descriptors.insert("VariMuMasterNode".to_string(), DspNodeDescriptor::new("VariMuMasterNode", "Vintage Fairchild 670 Vari-Mu Tube Mastering Compressor", DspNodeCategory::DynamicsMaster, "Vintage Fairchild 670 Vari-Mu Tube Mastering Compressor")
            .with_param(DspParamSchema::choice("time_constant", "Fairchild Time Constant", &["Pos 1 (Fast 0.2s/0.3s)", "Pos 2 (0.2s/0.8s)", "Pos 3 (0.4s/2.0s)", "Pos 4 (0.8s/5.0s)", "Pos 5 (Auto Fast)", "Pos 6 (Auto Program)"], 4, "Fairchild Time Constant mode selector"))
            .with_param(DspParamSchema::knob("dc_threshold", "Vari-Mu Tube Threshold", -30.0, 0.0, -12.0, "dB", MacroRole::Tone, "Vari-Mu Tube Threshold parameter"))
            .with_param(DspParamSchema::choice("tube_bias_mode", "Tube Bias Configuration", &["Standard Push-Pull", "Aggressive Mid Push", "Clean Master Linear"], 0, "Tube Bias Configuration mode selector"))
            .with_param(DspParamSchema::toggle("link_channels", "Stereo Linked Sidechain", true, "Stereo Linked Sidechain toggle switch"))
        );
        descriptors.insert("DynamicCrestShaperNode".to_string(), DspNodeDescriptor::new("DynamicCrestShaperNode", "Intelligent Peak-to-RMS Crest Factor Shaper", DspNodeCategory::DynamicsMaster, "Intelligent Peak-to-RMS Crest Factor Shaper")
            .with_param(DspParamSchema::knob("target_crest_db", "Target Crest Factor", 6.0, 20.0, 12.0, "dB", MacroRole::Tone, "Target Crest Factor parameter"))
            .with_param(DspParamSchema::knob("transient_priority", "Transient vs Body Weight", 0.0, 1.0, 0.6, "%", MacroRole::Tone, "Transient vs Body Weight parameter"))
            .with_param(DspParamSchema::knob("smoothing_window_ms", "RMS Detection Window", 10.0, 500.0, 100.0, "ms", MacroRole::Tone, "RMS Detection Window parameter"))
            .with_param(DspParamSchema::toggle("auto_makeup", "Auto Crest Makeup Gain", true, "Auto Crest Makeup Gain toggle switch"))
        );
        descriptors.insert("ParallelTransientSaturatorNode".to_string(), DspNodeDescriptor::new("ParallelTransientSaturatorNode", "Parallel Wet/Dry Transient Driver & Saturator", DspNodeCategory::DynamicsMaster, "Parallel Wet/Dry Transient Driver & Saturator")
            .with_param(DspParamSchema::knob("transient_drive_db", "Transient Punch Drive", 0.0, 24.0, 6.0, "dB", MacroRole::Tone, "Transient Punch Drive parameter"))
            .with_param(DspParamSchema::knob("sustain_warmth_db", "Sustain Body Saturation", 0.0, 24.0, 4.5, "dB", MacroRole::Tone, "Sustain Body Saturation parameter"))
            .with_param(DspParamSchema::knob("parallel_mix", "Parallel Wet Blend", 0.0, 1.0, 0.4, "%", MacroRole::Tone, "Parallel Wet Blend parameter"))
            .with_param(DspParamSchema::choice("clipping_mode", "Clipping Topology", &["Soft Germanium Diode", "Hard Silicon Diode", "Asymmetric Tube Triode"], 0, "Clipping Topology mode selector"))
        );
        descriptors.insert("TransientClipperNode".to_string(), DspNodeDescriptor::new("TransientClipperNode", "Sub-Millisecond True Transient Soft Clipper", DspNodeCategory::DynamicsMaster, "Sub-Millisecond True Transient Soft Clipper")
            .with_param(DspParamSchema::knob("threshold_db", "Clipping Knee Threshold", -18.0, 0.0, -3.0, "dBFS", MacroRole::Tone, "Clipping Knee Threshold parameter"))
            .with_param(DspParamSchema::knob("clip_hardness", "Clipper Hardness Exponent", 0.1, 5.0, 1.2, "", MacroRole::Tone, "Clipper Hardness Exponent parameter"))
            .with_param(DspParamSchema::knob("oversampling_order", "Anti-Aliasing Oversampling", 1.0, 16.0, 4.0, "x", MacroRole::Character, "Anti-Aliasing Oversampling integer control"))
            .with_param(DspParamSchema::knob("ceiling_db", "True Peak Limit Ceiling", -6.0, 0.0, -0.1, "dBTP", MacroRole::Tone, "True Peak Limit Ceiling parameter"))
        );
        descriptors.insert("TransientDeclickerNode".to_string(), DspNodeDescriptor::new("TransientDeclickerNode", "Surgical Sample De-Clicker & Pop Filter", DspNodeCategory::DynamicsMaster, "Surgical Sample De-Clicker & Pop Filter")
            .with_param(DspParamSchema::knob("sensitivity", "Detection Threshold", 0.0, 1.0, 0.7, "%", MacroRole::Tone, "Detection Threshold parameter"))
            .with_param(DspParamSchema::knob("click_width_samples", "Max Click Width", 2.0, 128.0, 16.0, "samples", MacroRole::Character, "Max Click Width integer control"))
            .with_param(DspParamSchema::knob("spectral_inpaint_depth", "Inpainting Reconstruction", 0.0, 1.0, 0.9, "%", MacroRole::Tone, "Inpainting Reconstruction parameter"))
            .with_param(DspParamSchema::toggle("listen_delta", "Audition Isolated Clicks", false, "Audition Isolated Clicks toggle switch"))
        );
        descriptors.insert("TransientDesignerNode".to_string(), DspNodeDescriptor::new("TransientDesignerNode", "Dual-Band Attack & Sustain Transient Designer", DspNodeCategory::DynamicsMaster, "Dual-Band Attack & Sustain Transient Designer")
            .with_param(DspParamSchema::knob("attack_gain_db", "Attack Transient Gain", -15.0, 15.0, 3.5, "dB", MacroRole::Tone, "Attack Transient Gain parameter"))
            .with_param(DspParamSchema::knob("sustain_gain_db", "Sustain Tail Gain", -15.0, 15.0, -2.0, "dB", MacroRole::Tone, "Sustain Tail Gain parameter"))
            .with_param(DspParamSchema::log_knob("split_frequency_hz", "Band Split Frequency", 100.0, 5000.0, 1200.0, "Hz", MacroRole::Tone, "Band Split Frequency logarithmic parameter"))
            .with_param(DspParamSchema::knob("output_gain_db", "Master Output Level", -12.0, 12.0, 0.0, "dB", MacroRole::Tone, "Master Output Level parameter"))
        );
        descriptors.insert("TransientGateNode".to_string(), DspNodeDescriptor::new("TransientGateNode", "Envelope-Triggered Transient Noise Gate", DspNodeCategory::DynamicsMaster, "Envelope-Triggered Transient Noise Gate")
            .with_param(DspParamSchema::knob("threshold_db", "Trigger Threshold", -70.0, 0.0, -40.0, "dB", MacroRole::Tone, "Trigger Threshold parameter"))
            .with_param(DspParamSchema::knob("attack_fast_ms", "Fast Attack Speed", 0.01, 10.0, 0.5, "ms", MacroRole::Tone, "Fast Attack Speed parameter"))
            .with_param(DspParamSchema::knob("hold_time_ms", "Hold Gate Duration", 0.0, 250.0, 15.0, "ms", MacroRole::Tone, "Hold Gate Duration parameter"))
            .with_param(DspParamSchema::knob("floor_attenuation_db", "Gate Attenuation Floor", -80.0, 0.0, -50.0, "dB", MacroRole::Tone, "Gate Attenuation Floor parameter"))
        );
        descriptors.insert("TransientReconstructorNode".to_string(), DspNodeDescriptor::new("TransientReconstructorNode", "Spectral Attack Transient Reconstructor", DspNodeCategory::DynamicsMaster, "Spectral Attack Transient Reconstructor")
            .with_param(DspParamSchema::knob("attack_restoration_db", "Attack Restoration Boost", 0.0, 18.0, 5.0, "dB", MacroRole::Tone, "Attack Restoration Boost parameter"))
            .with_param(DspParamSchema::knob("harmonic_synthesis_q", "Harmonic Phase Match Q", 1.0, 20.0, 6.0, "Q", MacroRole::Tone, "Harmonic Phase Match Q parameter"))
            .with_param(DspParamSchema::knob("onset_sensitivity", "Onset Trigger Sensitivity", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Onset Trigger Sensitivity parameter"))
            .with_param(DspParamSchema::knob("wet_dry", "Reconstructed Blend", 0.0, 1.0, 0.6, "%", MacroRole::Tone, "Reconstructed Blend parameter"))
        );
        descriptors.insert("TransientShaperNode".to_string(), DspNodeDescriptor::new("TransientShaperNode", "Zero-Latency Transient Punch & Sustain Shaper", DspNodeCategory::DynamicsMaster, "Zero-Latency Transient Punch & Sustain Shaper")
            .with_param(DspParamSchema::knob("punch_level_db", "Transient Punch Level", -12.0, 12.0, 4.0, "dB", MacroRole::Tone, "Transient Punch Level parameter"))
            .with_param(DspParamSchema::knob("body_level_db", "Body / Sustain Tail Level", -12.0, 12.0, 0.0, "dB", MacroRole::Tone, "Body / Sustain Tail Level parameter"))
            .with_param(DspParamSchema::knob("recovery_speed_ms", "Transient Recovery Speed", 5.0, 200.0, 45.0, "ms", MacroRole::Tone, "Transient Recovery Speed parameter"))
            .with_param(DspParamSchema::toggle("clip_protection", "Soft Clip Brickwall Guard", true, "Soft Clip Brickwall Guard toggle switch"))
        );
        descriptors.insert("TransientUnwrapperNode".to_string(), DspNodeDescriptor::new("TransientUnwrapperNode", "De-Compression & Dynamic Unwrapper", DspNodeCategory::DynamicsMaster, "De-Compression & Dynamic Unwrapper")
            .with_param(DspParamSchema::knob("crest_expansion_ratio", "Dynamic Unwrapping Ratio", 1.0, 3.5, 1.6, "x", MacroRole::Tone, "Dynamic Unwrapping Ratio parameter"))
            .with_param(DspParamSchema::knob("unwrapper_threshold_db", "Unwrapping Floor Level", -36.0, -6.0, -18.0, "dB", MacroRole::Tone, "Unwrapping Floor Level parameter"))
            .with_param(DspParamSchema::toggle("transient_preserve", "Preserve Fast Transients", true, "Preserve Fast Transients toggle switch"))
            .with_param(DspParamSchema::knob("gain_trim_db", "Post-Unwrap Output Trim", -12.0, 12.0, -1.0, "dB", MacroRole::Tone, "Post-Unwrap Output Trim parameter"))
        );
        descriptors.insert("DistortionNode".to_string(), DspNodeDescriptor::new("DistortionNode", "Asymmetric Diode Clipping Wavefolder", DspNodeCategory::DistortionSaturation, "Asymmetric Diode Clipping Wavefolder")
            .with_param(DspParamSchema::knob("drive", "Drive Gain", 1.0, 30.0, 4.0, "x", MacroRole::Tone, "Drive Gain parameter"))
            .with_param(DspParamSchema::knob("asymmetry", "Even Harmonic Bias", 0.0, 1.0, 0.25, "%", MacroRole::Tone, "Even Harmonic Bias parameter"))
            .with_param(DspParamSchema::log_knob("tone", "High Cut Tone", 500.0, 16000.0, 4500.0, "Hz", MacroRole::Tone, "High Cut Tone logarithmic parameter"))
            .with_param(DspParamSchema::knob("output", "Master Level", -24.0, 6.0, -2.0, "dB", MacroRole::Tone, "Master Level parameter"))
        );
        descriptors.insert("TapeSaturationNode".to_string(), DspNodeDescriptor::new("TapeSaturationNode", "Magnetic Tape Hysteresis & Flux Compression", DspNodeCategory::DistortionSaturation, "Magnetic Tape Hysteresis & Flux Compression")
            .with_param(DspParamSchema::knob("drive", "Tape Input Drive", 1.0, 10.0, 3.0, "x", MacroRole::Tone, "Tape Input Drive parameter"))
            .with_param(DspParamSchema::choice("speed", "Tape Speed", &["7.5 IPS (Warm)", "15 IPS (Studio)", "30 IPS (Mastering)"], 1, "Tape Speed mode selector"))
            .with_param(DspParamSchema::knob("flutter", "Wow & Flutter", 0.0, 1.0, 0.12, "%", MacroRole::Tone, "Wow & Flutter parameter"))
            .with_param(DspParamSchema::knob("head_bump", "Head Bump Resonance", 0.0, 6.0, 2.0, "dB", MacroRole::Tone, "Head Bump Resonance parameter"))
        );
        descriptors.insert("TubeSaturationNode".to_string(), DspNodeDescriptor::new("TubeSaturationNode", "Triode/Pentode Dual-Stage Tube Saturation", DspNodeCategory::DistortionSaturation, "Triode/Pentode Dual-Stage Tube Saturation")
            .with_param(DspParamSchema::knob("drive", "Tube Saturation Drive", 1.0, 12.0, 3.5, "x", MacroRole::Tone, "Tube Saturation Drive parameter"))
            .with_param(DspParamSchema::knob("bias", "Grid Bias Asymmetry", 0.0, 0.5, 0.2, "", MacroRole::Tone, "Grid Bias Asymmetry parameter"))
            .with_param(DspParamSchema::choice("topology", "Tube Model", &["12AX7 Triode", "EL34 Pentode", "6L6 Power Tube", "KT88 Audiophile"], 0, "Tube Model mode selector"))
            .with_param(DspParamSchema::knob("second_harm", "2nd Harmonic Richness", 0.0, 1.0, 0.4, "%", MacroRole::Tone, "2nd Harmonic Richness parameter"))
        );
        descriptors.insert("TubeBiasNode".to_string(), DspNodeDescriptor::new("TubeBiasNode", "Non-Linear Vacuum Tube Grid Bias Controller", DspNodeCategory::DistortionSaturation, "Non-Linear Vacuum Tube Grid Bias Controller")
            .with_param(DspParamSchema::knob("grid_bias_volts", "Tube Grid Bias Voltage", -5.0, 0.0, -1.8, "V", MacroRole::Tone, "Tube Grid Bias Voltage parameter"))
            .with_param(DspParamSchema::knob("plate_voltage_v", "Anode Plate Voltage", 100.0, 450.0, 250.0, "V", MacroRole::Tone, "Anode Plate Voltage parameter"))
            .with_param(DspParamSchema::knob("even_harmonic_blend", "Even vs Odd Harmonic Balance", 0.0, 1.0, 0.65, "%", MacroRole::Tone, "Even vs Odd Harmonic Balance parameter"))
            .with_param(DspParamSchema::knob("drive_level", "Input Overdrive Gain", 0.5, 8.0, 2.0, "x", MacroRole::Tone, "Input Overdrive Gain parameter"))
        );
        descriptors.insert("ConsoleEmulationNode".to_string(), DspNodeDescriptor::new("ConsoleEmulationNode", "British Class-A Console Channel Coloration", DspNodeCategory::DistortionSaturation, "British Class-A Console Channel Coloration")
            .with_param(DspParamSchema::choice("console", "Console Model", &["Neve 8078 (Warm)", "SSL 4000E (Punchy)", "API 1608 (Crisp)"], 0, "Console Model mode selector"))
            .with_param(DspParamSchema::knob("drive", "Pushed Channel Drive", 0.0, 5.0, 1.2, "x", MacroRole::Tone, "Pushed Channel Drive parameter"))
            .with_param(DspParamSchema::knob("crosstalk", "Adjacent Crosstalk", -90.0, -30.0, -65.0, "dB", MacroRole::Tone, "Adjacent Crosstalk parameter"))
            .with_param(DspParamSchema::knob("transformer", "Transformer Saturation", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Transformer Saturation parameter"))
        );
        descriptors.insert("BitcrusherNode".to_string(), DspNodeDescriptor::new("BitcrusherNode", "Variable Sample-Rate & Bit-Depth Quantizer", DspNodeCategory::DistortionSaturation, "Variable Sample-Rate & Bit-Depth Quantizer")
            .with_param(DspParamSchema::knob("bits", "Quantized Bit Depth", 1.0, 16.0, 8.0, "bits", MacroRole::Character, "Quantized Bit Depth integer control"))
            .with_param(DspParamSchema::knob("downsample", "Downsample Ratio", 1.0, 32.0, 4.0, "x", MacroRole::Character, "Downsample Ratio integer control"))
            .with_param(DspParamSchema::knob("jitter", "Clock Aperture Jitter", 0.0, 1.0, 0.05, "%", MacroRole::Tone, "Clock Aperture Jitter parameter"))
            .with_param(DspParamSchema::toggle("dither", "Anti-alias Dither", false, "Anti-alias Dither toggle switch"))
        );
        descriptors.insert("WavefolderNode".to_string(), DspNodeDescriptor::new("WavefolderNode", "Non-Linear Multi-Stage Wavefolder", DspNodeCategory::DistortionSaturation, "Non-Linear Multi-Stage Wavefolder")
            .with_param(DspParamSchema::knob("threshold", "Folding Threshold", 0.05, 1.0, 0.5, "%", MacroRole::Tone, "Folding Threshold parameter"))
            .with_param(DspParamSchema::knob("folds", "Wavefold Iterations", 1.0, 8.0, 3.0, "folds", MacroRole::Character, "Wavefold Iterations integer control"))
            .with_param(DspParamSchema::knob("drive", "Pre-folder Gain", 1.0, 10.0, 2.5, "x", MacroRole::Tone, "Pre-folder Gain parameter"))
            .with_param(DspParamSchema::knob("symmetry", "DC Offset Symmetry", -1.0, 1.0, 0.0, "", MacroRole::Tone, "DC Offset Symmetry parameter"))
        );
        descriptors.insert("HarmonicExciterNode".to_string(), DspNodeDescriptor::new("HarmonicExciterNode", "Aural Exciter & High-Band Harmonics Generator", DspNodeCategory::DistortionSaturation, "Aural Exciter & High-Band Harmonics Generator")
            .with_param(DspParamSchema::knob("drive", "Exciter Harmonics Drive", 0.0, 1.0, 0.45, "%", MacroRole::Tone, "Exciter Harmonics Drive parameter"))
            .with_param(DspParamSchema::log_knob("frequency", "High Shelf Cutoff", 2000.0, 15000.0, 6000.0, "Hz", MacroRole::Tone, "High Shelf Cutoff logarithmic parameter"))
            .with_param(DspParamSchema::knob("mix", "Harmonics Wet Mix", 0.0, 1.0, 0.35, "%", MacroRole::Tone, "Harmonics Wet Mix parameter"))
        );
        descriptors.insert("NeuralHarmonicExciterNode".to_string(), DspNodeDescriptor::new("NeuralHarmonicExciterNode", "AI Spectral Sheen & High-Frequency Air Polish", DspNodeCategory::NeuralAi, "AI Spectral Sheen & High-Frequency Air Polish")
            .with_param(DspParamSchema::knob("air_sheen", "Ultra-High Air Presence", 0.0, 12.0, 4.0, "dB", MacroRole::Tone, "Ultra-High Air Presence parameter"))
            .with_param(DspParamSchema::knob("warmth_body", "Low-Mid Harmonic Warmth", 0.0, 12.0, 2.0, "dB", MacroRole::Tone, "Low-Mid Harmonic Warmth parameter"))
            .with_param(DspParamSchema::choice("ai_profile", "Exciter Neural Profile", &["Modern Polish", "Vintage Tube", "Transformer Color", "Silk Top"], 0, "Exciter Neural Profile mode selector"))
            .with_param(DspParamSchema::knob("wet_mix", "Harmonic Blend Level", 0.0, 1.0, 0.4, "%", MacroRole::Tone, "Harmonic Blend Level parameter"))
        );
        descriptors.insert("Atmos916Spatializer".to_string(), DspNodeDescriptor::new("Atmos916Spatializer", "Dolby Atmos 9.1.6 Object Panner with HRTF", DspNodeCategory::SpatialSurround, "Dolby Atmos 9.1.6 Object Panner with HRTF")
            .with_param(DspParamSchema::knob("azimuth", "Horizontal Azimuth", -180.0, 180.0, 0.0, "deg", MacroRole::Tone, "Horizontal Azimuth parameter"))
            .with_param(DspParamSchema::knob("elevation", "Vertical Elevation", -90.0, 90.0, 0.0, "deg", MacroRole::Tone, "Vertical Elevation parameter"))
            .with_param(DspParamSchema::knob("distance", "Virtual Distance", 0.5, 20.0, 2.0, "m", MacroRole::Tone, "Virtual Distance parameter"))
            .with_param(DspParamSchema::knob("spread", "Object Size Spread", 0.0, 100.0, 15.0, "%", MacroRole::Tone, "Object Size Spread parameter"))
        );
        descriptors.insert("AmbisonicRadarSpatializer".to_string(), DspNodeDescriptor::new("AmbisonicRadarSpatializer", "Higher-Order Ambisonics HOA-5 Spherical Radar", DspNodeCategory::SpatialSurround, "Higher-Order Ambisonics HOA-5 Spherical Radar")
            .with_param(DspParamSchema::knob("yaw", "Radar Yaw", -180.0, 180.0, 0.0, "deg", MacroRole::Tone, "Radar Yaw parameter"))
            .with_param(DspParamSchema::knob("pitch", "Radar Pitch", -90.0, 90.0, 0.0, "deg", MacroRole::Tone, "Radar Pitch parameter"))
            .with_param(DspParamSchema::knob("roll", "Radar Roll", -180.0, 180.0, 0.0, "deg", MacroRole::Tone, "Radar Roll parameter"))
            .with_param(DspParamSchema::knob("hoa_order", "Ambisonics Order", 1.0, 5.0, 3.0, "order", MacroRole::Character, "Ambisonics Order integer control"))
        );
        descriptors.insert("Auro3dSpatializer".to_string(), DspNodeDescriptor::new("Auro3dSpatializer", "Auro-3D 13.1 Tri-Layer Immersive Panner", DspNodeCategory::SpatialSurround, "Auro-3D 13.1 Tri-Layer Immersive Panner")
            .with_param(DspParamSchema::knob("lower_layer", "Lower Ear-Level Gain", -12.0, 6.0, 0.0, "dB", MacroRole::Tone, "Lower Ear-Level Gain parameter"))
            .with_param(DspParamSchema::knob("height_layer", "Height Layer Gain", -12.0, 6.0, 0.0, "dB", MacroRole::Tone, "Height Layer Gain parameter"))
            .with_param(DspParamSchema::knob("top_voice", "Top Voice of God", -12.0, 6.0, -3.0, "dB", MacroRole::Tone, "Top Voice of God parameter"))
            .with_param(DspParamSchema::knob("spread", "Surround Width Spread", 0.0, 1.0, 0.7, "%", MacroRole::Tone, "Surround Width Spread parameter"))
        );
        descriptors.insert("Nhk222Spatializer".to_string(), DspNodeDescriptor::new("Nhk222Spatializer", "NHK 22.2 Super Hi-Vision Multi-Channel Matrix", DspNodeCategory::SpatialSurround, "NHK 22.2 Super Hi-Vision Multi-Channel Matrix")
            .with_param(DspParamSchema::knob("top_gain", "Top Layer (9ch)", -12.0, 6.0, 0.0, "dB", MacroRole::Tone, "Top Layer (9ch) parameter"))
            .with_param(DspParamSchema::knob("mid_gain", "Middle Layer (10ch)", -12.0, 6.0, 0.0, "dB", MacroRole::Tone, "Middle Layer (10ch) parameter"))
            .with_param(DspParamSchema::knob("bottom_gain", "Bottom Layer (3ch)", -12.0, 6.0, -2.0, "dB", MacroRole::Tone, "Bottom Layer (3ch) parameter"))
            .with_param(DspParamSchema::knob("lfe_sub", "Dual LFE Subwoofers", -12.0, 6.0, 0.0, "dB", MacroRole::Tone, "Dual LFE Subwoofers parameter"))
        );
        descriptors.insert("BinauralBrirSpatializer".to_string(), DspNodeDescriptor::new("BinauralBrirSpatializer", "Binaural Room Impulse Response Convolver", DspNodeCategory::SpatialSurround, "Binaural Room Impulse Response Convolver")
            .with_param(DspParamSchema::knob("room_size", "Room Dimensions", 5.0, 50.0, 15.0, "m", MacroRole::Tone, "Room Dimensions parameter"))
            .with_param(DspParamSchema::choice("hrtf", "HRTF Profile", &["KEMAR Dummy Head", "CIPIC High-Res", "Listen Individual", "Sphaero HRIR"], 0, "HRTF Profile mode selector"))
            .with_param(DspParamSchema::knob("early_gain", "Early Reflections Gain", -24.0, 6.0, 0.0, "dB", MacroRole::Tone, "Early Reflections Gain parameter"))
            .with_param(DspParamSchema::knob("late_diffuse", "Late Diffuse Field", -24.0, 6.0, -6.0, "dB", MacroRole::Tone, "Late Diffuse Field parameter"))
        );
        descriptors.insert("BinauralSpatializerNode".to_string(), DspNodeDescriptor::new("BinauralSpatializerNode", "3D Binaural HRTF Spatializer with ITD & ILD", DspNodeCategory::SpatialSurround, "3D Binaural HRTF Spatializer with ITD & ILD")
            .with_param(DspParamSchema::knob("azim_deg", "Azimuth Angle", -180.0, 180.0, 0.0, "deg", MacroRole::Tone, "Azimuth Angle parameter"))
            .with_param(DspParamSchema::knob("elev_deg", "Elevation Angle", -90.0, 90.0, 0.0, "deg", MacroRole::Tone, "Elevation Angle parameter"))
            .with_param(DspParamSchema::knob("dist_m", "Source Distance", 0.2, 20.0, 1.5, "m", MacroRole::Tone, "Source Distance parameter"))
            .with_param(DspParamSchema::toggle("itd_ild_mode", "Interaural Delay & Level On", true, "Interaural Delay & Level On toggle switch"))
        );
        descriptors.insert("RaytracedRoomReverb".to_string(), DspNodeDescriptor::new("RaytracedRoomReverb", "Geometric Raytraced Acoustic Room Simulator", DspNodeCategory::TimeSpace, "Geometric Raytraced Acoustic Room Simulator")
            .with_param(DspParamSchema::knob("room_width", "Room Width", 2.0, 100.0, 12.0, "m", MacroRole::Tone, "Room Width parameter"))
            .with_param(DspParamSchema::knob("room_length", "Room Length", 2.0, 100.0, 18.0, "m", MacroRole::Tone, "Room Length parameter"))
            .with_param(DspParamSchema::knob("room_height", "Room Ceiling Height", 2.0, 30.0, 6.0, "m", MacroRole::Tone, "Room Ceiling Height parameter"))
            .with_param(DspParamSchema::knob("wall_absorption", "Wall Acoustic Absorption", 0.05, 0.95, 0.25, "", MacroRole::Tone, "Wall Acoustic Absorption parameter"))
            .with_param(DspParamSchema::knob("scattering", "Diffusive Surface Scattering", 0.0, 1.0, 0.4, "%", MacroRole::Tone, "Diffusive Surface Scattering parameter"))
        );
        descriptors.insert("EffectDelay".to_string(), DspNodeDescriptor::new("EffectDelay", "Stereo Ping-Pong Feedback Echo Delay", DspNodeCategory::TimeSpace, "Stereo Ping-Pong Feedback Echo Delay")
            .with_param(DspParamSchema::knob("time_s", "Delay Time", 0.01, 2.0, 0.35, "s", MacroRole::Tone, "Delay Time parameter"))
            .with_param(DspParamSchema::knob("feedback", "Delay Feedback", 0.0, 0.95, 0.45, "%", MacroRole::Tone, "Delay Feedback parameter"))
            .with_param(DspParamSchema::knob("mix", "Dry / Wet Mix", 0.0, 1.0, 0.3, "%", MacroRole::Tone, "Dry / Wet Mix parameter"))
            .with_param(DspParamSchema::toggle("ping_pong", "Stereo Ping-Pong Mode", true, "Stereo Ping-Pong Mode toggle switch"))
            .with_param(DspParamSchema::log_knob("high_cut", "Feedback High Cut Filter", 1000.0, 20000.0, 8000.0, "Hz", MacroRole::Tone, "Feedback High Cut Filter logarithmic parameter"))
        );
        descriptors.insert("EffectReverb".to_string(), DspNodeDescriptor::new("EffectReverb", "Algorithmic Feedback Delay Network (FDN) Reverb", DspNodeCategory::TimeSpace, "Algorithmic Feedback Delay Network (FDN) Reverb")
            .with_param(DspParamSchema::knob("room_size", "Virtual Room Size", 0.0, 1.0, 0.75, "%", MacroRole::Tone, "Virtual Room Size parameter"))
            .with_param(DspParamSchema::knob("damping", "High Frequency Damping", 0.0, 1.0, 0.3, "%", MacroRole::Tone, "High Frequency Damping parameter"))
            .with_param(DspParamSchema::knob("mix", "Reverb Wet Mix", 0.0, 1.0, 0.35, "%", MacroRole::Tone, "Reverb Wet Mix parameter"))
            .with_param(DspParamSchema::knob("predelay_ms", "Pre-delay Time", 0.0, 250.0, 25.0, "ms", MacroRole::Tone, "Pre-delay Time parameter"))
            .with_param(DspParamSchema::knob("decay_s", "Decay Time (T60)", 0.2, 10.0, 2.8, "s", MacroRole::Tone, "Decay Time (T60) parameter"))
        );
        descriptors.insert("FdnReverbNode".to_string(), DspNodeDescriptor::new("FdnReverbNode", "High-Density Feedback Delay Network Reverb Core", DspNodeCategory::TimeSpace, "High-Density Feedback Delay Network Reverb Core")
            .with_param(DspParamSchema::knob("matrix_size", "FDN Matrix Delay Lines", 4.0, 16.0, 8.0, "lines", MacroRole::Character, "FDN Matrix Delay Lines integer control"))
            .with_param(DspParamSchema::knob("t60_decay", "T60 Reverberation Time", 0.5, 20.0, 3.2, "s", MacroRole::Tone, "T60 Reverberation Time parameter"))
            .with_param(DspParamSchema::knob("diffusion", "Orthogonal Matrix Diffusion", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Orthogonal Matrix Diffusion parameter"))
            .with_param(DspParamSchema::knob("modal_density", "Modal Echo Density", 0.1, 2.0, 1.0, "x", MacroRole::Tone, "Modal Echo Density parameter"))
            .with_param(DspParamSchema::knob("dry_wet", "Dry / Wet Mix", 0.0, 1.0, 0.35, "%", MacroRole::Tone, "Dry / Wet Mix parameter"))
        );
        descriptors.insert("ConvolutionReverbNode".to_string(), DspNodeDescriptor::new("ConvolutionReverbNode", "Zero-Latency Partitioned Convolution Reverb", DspNodeCategory::TimeSpace, "Zero-Latency Partitioned Convolution Reverb")
            .with_param(DspParamSchema::knob("ir_decay", "Impulse Response Decay", 0.1, 10.0, 2.0, "s", MacroRole::Tone, "Impulse Response Decay parameter"))
            .with_param(DspParamSchema::knob("predelay", "Pre-delay", 0.0, 200.0, 10.0, "ms", MacroRole::Tone, "Pre-delay parameter"))
            .with_param(DspParamSchema::knob("high_damp", "High Frequency Damping", 0.0, 1.0, 0.2, "%", MacroRole::Tone, "High Frequency Damping parameter"))
            .with_param(DspParamSchema::knob("dry_wet", "Dry / Wet Mix", 0.0, 1.0, 0.4, "%", MacroRole::Tone, "Dry / Wet Mix parameter"))
        );
        descriptors.insert("HyperDimensionalTensorSpatializer".to_string(), DspNodeDescriptor::new("HyperDimensionalTensorSpatializer", "11-Dimensional Calabi-Yau Spatial Tensor Panner", DspNodeCategory::SpatialSurround, "11-Dimensional Calabi-Yau Spatial Tensor Panner")
            .with_param(DspParamSchema::knob("calabi_yau_dim", "Calabi-Yau Manifold Dims", 4.0, 11.0, 11.0, "dims", MacroRole::Character, "Calabi-Yau Manifold Dims integer control"))
            .with_param(DspParamSchema::knob("manifold_curvature", "Ricci Curvature Scaling", 0.1, 5.0, 1.8, "k", MacroRole::Tone, "Ricci Curvature Scaling parameter"))
            .with_param(DspParamSchema::knob("subspace_rotation_deg", "Subspace Rotation Angle", 0.0, 360.0, 45.0, "deg", MacroRole::Tone, "Subspace Rotation Angle parameter"))
            .with_param(DspParamSchema::knob("dimensional_spread", "High-Dim Sound Spread", 0.0, 1.0, 0.7, "%", MacroRole::Tone, "High-Dim Sound Spread parameter"))
        );
        descriptors.insert("Holographic3dSpatialSoundfield".to_string(), DspNodeDescriptor::new("Holographic3dSpatialSoundfield", "Holographic 3D Spherical Wavefield Synthesizer", DspNodeCategory::SpatialSurround, "Holographic 3D Spherical Wavefield Synthesizer")
            .with_param(DspParamSchema::knob("spherical_radius_m", "Wavefield Sphere Radius", 0.5, 25.0, 3.0, "m", MacroRole::Tone, "Wavefield Sphere Radius parameter"))
            .with_param(DspParamSchema::knob("azimuth_rad", "Source Azimuth Angle", -3.14, 3.14, 0.0, "rad", MacroRole::Tone, "Source Azimuth Angle parameter"))
            .with_param(DspParamSchema::knob("elevation_rad", "Source Elevation Angle", -1.57, 1.57, 0.0, "rad", MacroRole::Tone, "Source Elevation Angle parameter"))
            .with_param(DspParamSchema::knob("wavefront_curvature", "Virtual Wave Curvature", 0.1, 5.0, 1.2, "", MacroRole::Tone, "Virtual Wave Curvature parameter"))
        );
        descriptors.insert("NonEuclideanHyperbolicReverb".to_string(), DspNodeDescriptor::new("NonEuclideanHyperbolicReverb", "Hyperbolic Geometry Non-Euclidean Reverb Tank", DspNodeCategory::TimeSpace, "Hyperbolic Geometry Non-Euclidean Reverb Tank")
            .with_param(DspParamSchema::knob("poincare_curvature", "Poincaré Disk Curvature", -5.0, -0.1, -1.5, "K", MacroRole::Tone, "Poincaré Disk Curvature parameter"))
            .with_param(DspParamSchema::knob("geodesic_diffusion", "Hyperbolic Geodesic Diffusion", 0.0, 1.0, 0.85, "%", MacroRole::Tone, "Hyperbolic Geodesic Diffusion parameter"))
            .with_param(DspParamSchema::knob("hyperbolic_t60_s", "Hyperbolic Decay Time", 0.5, 30.0, 5.5, "s", MacroRole::Tone, "Hyperbolic Decay Time parameter"))
            .with_param(DspParamSchema::knob("reflections_mix", "Infinite Horizon Reflection Mix", 0.0, 1.0, 0.4, "%", MacroRole::Tone, "Infinite Horizon Reflection Mix parameter"))
        );
        descriptors.insert("IsmShockwaveReverb".to_string(), DspNodeDescriptor::new("IsmShockwaveReverb", "Interstellar Medium Shockwave Convolution Reverb", DspNodeCategory::TimeSpace, "Interstellar Medium Shockwave Convolution Reverb")
            .with_param(DspParamSchema::knob("shockwave_radius_au", "Shock Front Radius", 0.1, 50.0, 5.0, "AU", MacroRole::Tone, "Shock Front Radius parameter"))
            .with_param(DspParamSchema::knob("density_per_cm3", "Plasma Proton Density", 1.0, 1000.0, 150.0, "cm3", MacroRole::Tone, "Plasma Proton Density parameter"))
            .with_param(DspParamSchema::knob("plasma_cooling_t60", "Radiative Cooling Decay", 1.0, 60.0, 12.0, "s", MacroRole::Tone, "Radiative Cooling Decay parameter"))
            .with_param(DspParamSchema::knob("reverb_gain", "Interstellar Reverb Output", 0.0, 1.0, 0.75, "%", MacroRole::Tone, "Interstellar Reverb Output parameter"))
        );
        descriptors.insert("AcousticCloakingSpatializer".to_string(), DspNodeDescriptor::new("AcousticCloakingSpatializer", "Acoustic Cloaking Phase Cancellation Spatializer", DspNodeCategory::SpatialSurround, "Acoustic Cloaking Phase Cancellation Spatializer")
            .with_param(DspParamSchema::knob("cloaking_radius_m", "Cloaking Shell Radius", 0.5, 10.0, 2.0, "m", MacroRole::Tone, "Cloaking Shell Radius parameter"))
            .with_param(DspParamSchema::knob("scattering_cancellation", "Phase Scattering Cancel", 0.0, 1.0, 0.9, "%", MacroRole::Tone, "Phase Scattering Cancel parameter"))
            .with_param(DspParamSchema::knob("shadow_attenuation_db", "Acoustic Shadow Depth", -40.0, 0.0, -18.0, "dB", MacroRole::Tone, "Acoustic Shadow Depth parameter"))
            .with_param(DspParamSchema::knob("phase_invert_blend", "Metasurface Anti-Phase Blend", 0.0, 1.0, 1.0, "%", MacroRole::Tone, "Metasurface Anti-Phase Blend parameter"))
        );
        descriptors.insert("AtmosProximityNode".to_string(), DspNodeDescriptor::new("AtmosProximityNode", "Dolby Atmos Proximity Effect & Distance Attenuator", DspNodeCategory::SpatialSurround, "Dolby Atmos Proximity Effect & Distance Attenuator")
            .with_param(DspParamSchema::knob("distance_m", "Acoustic Source Distance", 0.1, 50.0, 2.5, "m", MacroRole::Tone, "Acoustic Source Distance parameter"))
            .with_param(DspParamSchema::knob("air_absorption_hf_loss_db", "Air Absorption HF Loss", 0.0, 24.0, 4.5, "dB", MacroRole::Tone, "Air Absorption HF Loss parameter"))
            .with_param(DspParamSchema::knob("proximity_bass_boost_db", "Proximity Bass Boost", 0.0, 18.0, 3.0, "dB", MacroRole::Tone, "Proximity Bass Boost parameter"))
            .with_param(DspParamSchema::knob("itd_spread", "Interaural Distance Spread", 0.0, 1.0, 0.6, "%", MacroRole::Tone, "Interaural Distance Spread parameter"))
        );
        descriptors.insert("AtmosSurroundNode".to_string(), DspNodeDescriptor::new("AtmosSurroundNode", "7.1.4 Immersive Atmos Surround Bed Renderer", DspNodeCategory::SpatialSurround, "7.1.4 Immersive Atmos Surround Bed Renderer")
            .with_param(DspParamSchema::knob("surround_azimuth", "Bed Horizontal Azimuth", -180.0, 180.0, 0.0, "deg", MacroRole::Tone, "Bed Horizontal Azimuth parameter"))
            .with_param(DspParamSchema::knob("height_elevation", "Height Overhead Angle", 0.0, 90.0, 35.0, "deg", MacroRole::Tone, "Height Overhead Angle parameter"))
            .with_param(DspParamSchema::knob("lfe_send_level", "LFE Subwoofer Send", -40.0, 6.0, 0.0, "dB", MacroRole::Tone, "LFE Subwoofer Send parameter"))
            .with_param(DspParamSchema::knob("divergence_spread", "Center-Surround Divergence", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Center-Surround Divergence parameter"))
        );
        descriptors.insert("HoaSpatializer".to_string(), DspNodeDescriptor::new("HoaSpatializer", "4th/5th-Order High Order Ambisonic Spherical Panner", DspNodeCategory::SpatialSurround, "4th/5th-Order High Order Ambisonic Spherical Panner")
            .with_param(DspParamSchema::knob("spherical_yaw", "Ambisonic Yaw Angle", -180.0, 180.0, 0.0, "deg", MacroRole::Tone, "Ambisonic Yaw Angle parameter"))
            .with_param(DspParamSchema::knob("spherical_pitch", "Ambisonic Pitch Angle", -90.0, 90.0, 0.0, "deg", MacroRole::Tone, "Ambisonic Pitch Angle parameter"))
            .with_param(DspParamSchema::knob("ambisonic_order", "HOA Spatial Order", 1.0, 5.0, 4.0, "order", MacroRole::Character, "HOA Spatial Order integer control"))
            .with_param(DspParamSchema::knob("in_phase_weighting", "In-Phase Max-rE Weighting", 0.0, 1.0, 0.75, "%", MacroRole::Tone, "In-Phase Max-rE Weighting parameter"))
        );
        descriptors.insert("Hoa5BinauralSpatializer".to_string(), DspNodeDescriptor::new("Hoa5BinauralSpatializer", "5th-Order Ambisonic Binaural Decoder with SOFA HRTF", DspNodeCategory::SpatialSurround, "5th-Order Ambisonic Binaural Decoder with SOFA HRTF")
            .with_param(DspParamSchema::choice("sofa_profile", "SOFA HRIR Profile", &["KEMAR High-Precision", "Genelec Aural ID", "SADIE II Dummy", "IRCAM Listen Subject 1002"], 0, "SOFA HRIR Profile mode selector"))
            .with_param(DspParamSchema::choice("binaural_rendering_mode", "Binaural Decoder Kernel", &["Dual-Parabolic SVD", "Spherical Harmonics Least-Squares", "Diffuse Field Equalized"], 1, "Binaural Decoder Kernel mode selector"))
            .with_param(DspParamSchema::knob("nearfield_compensation", "Nearfield Bass Compensation", 0.0, 12.0, 2.5, "dB", MacroRole::Tone, "Nearfield Bass Compensation parameter"))
            .with_param(DspParamSchema::knob("itd_emphasis", "Psychoacoustic ITD Emphasis", 0.5, 2.0, 1.0, "x", MacroRole::Tone, "Psychoacoustic ITD Emphasis parameter"))
        );
        descriptors.insert("Mpegh3DSpatializer".to_string(), DspNodeDescriptor::new("Mpegh3DSpatializer", "MPEG-H 3D Immersive Audio Channel Bed Panner", DspNodeCategory::SpatialSurround, "MPEG-H 3D Immersive Audio Channel Bed Panner")
            .with_param(DspParamSchema::choice("bed_layout", "MPEG-H Target Bed", &["5.1.4 Surround", "7.1.4 Immersive", "9.1.6 3D Master", "22.2 Hi-Vision"], 1, "MPEG-H Target Bed mode selector"))
            .with_param(DspParamSchema::knob("elevation_gain", "Overhead Layer Trim", -12.0, 6.0, 0.0, "dB", MacroRole::Tone, "Overhead Layer Trim parameter"))
            .with_param(DspParamSchema::knob("object_spread", "Object Spherical Divergence", 0.0, 100.0, 20.0, "%", MacroRole::Tone, "Object Spherical Divergence parameter"))
            .with_param(DspParamSchema::knob("lfe_gain", "LFE Subwoofer Level", -24.0, 6.0, 0.0, "dB", MacroRole::Tone, "LFE Subwoofer Level parameter"))
        );
        descriptors.insert("MpeghTrajectorySpatializer".to_string(), DspNodeDescriptor::new("MpeghTrajectorySpatializer", "3D Trajectory Bezier Spline Audio Spatializer", DspNodeCategory::SpatialSurround, "3D Trajectory Bezier Spline Audio Spatializer")
            .with_param(DspParamSchema::knob("trajectory_speed_bpm", "Spline Trajectory Speed", 1.0, 240.0, 60.0, "BPM", MacroRole::Tone, "Spline Trajectory Speed parameter"))
            .with_param(DspParamSchema::knob("spline_tension", "Bezier Spline Tension", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Bezier Spline Tension parameter"))
            .with_param(DspParamSchema::knob("path_radius_m", "Orbital Path Radius", 0.5, 20.0, 3.5, "m", MacroRole::Tone, "Orbital Path Radius parameter"))
            .with_param(DspParamSchema::knob("elevation_range_m", "Vertical Elevation Sweep", 0.0, 10.0, 2.0, "m", MacroRole::Tone, "Vertical Elevation Sweep parameter"))
        );
        descriptors.insert("WfsArraySpatializerNode".to_string(), DspNodeDescriptor::new("WfsArraySpatializerNode", "Wave Field Synthesis Multi-Transducer Array Engine", DspNodeCategory::SpatialSurround, "Wave Field Synthesis Multi-Transducer Array Engine")
            .with_param(DspParamSchema::knob("array_elements_count", "WFS Transducer Count", 16.0, 128.0, 64.0, "speakers", MacroRole::Character, "WFS Transducer Count integer control"))
            .with_param(DspParamSchema::knob("speaker_spacing_cm", "Transducer Element Pitch", 2.0, 30.0, 8.0, "cm", MacroRole::Tone, "Transducer Element Pitch parameter"))
            .with_param(DspParamSchema::knob("focused_source_depth_m", "Focused Virtual Depth", -5.0, 15.0, 2.5, "m", MacroRole::Tone, "Focused Virtual Depth parameter"))
            .with_param(DspParamSchema::knob("spatial_aliasing_cut", "Spatial Aliasing Lowpass", 1000.0, 15000.0, 4500.0, "Hz", MacroRole::Tone, "Spatial Aliasing Lowpass parameter"))
        );
        descriptors.insert("DiffractivePropagationNode".to_string(), DspNodeDescriptor::new("DiffractivePropagationNode", "Acoustic Obstacle Diffraction & Shadow Zone Simulator", DspNodeCategory::SpatialSurround, "Acoustic Obstacle Diffraction & Shadow Zone Simulator")
            .with_param(DspParamSchema::knob("obstacle_width_m", "Obstacle Barrier Width", 0.2, 20.0, 3.0, "m", MacroRole::Tone, "Obstacle Barrier Width parameter"))
            .with_param(DspParamSchema::knob("diffraction_angle_deg", "Bending Diffraction Angle", 0.0, 180.0, 45.0, "deg", MacroRole::Tone, "Bending Diffraction Angle parameter"))
            .with_param(DspParamSchema::log_knob("shadow_zone_lowpass_hz", "Shadow Zone High Cut", 200.0, 10000.0, 1200.0, "Hz", MacroRole::Tone, "Shadow Zone High Cut logarithmic parameter"))
            .with_param(DspParamSchema::knob("edge_scattering_gain", "Edge Scattering Amplitude", 0.0, 1.0, 0.35, "%", MacroRole::Tone, "Edge Scattering Amplitude parameter"))
        );
        descriptors.insert("BinauralPannerNode".to_string(), DspNodeDescriptor::new("BinauralPannerNode", "Precision Interaural Time & Level Difference 3D Panner", DspNodeCategory::SpatialSurround, "Precision Interaural Time & Level Difference 3D Panner")
            .with_param(DspParamSchema::knob("azimuth_deg", "Source Azimuth Angle", -180.0, 180.0, 0.0, "deg", MacroRole::Tone, "Source Azimuth Angle parameter"))
            .with_param(DspParamSchema::knob("elevation_deg", "Source Elevation Angle", -90.0, 90.0, 0.0, "deg", MacroRole::Tone, "Source Elevation Angle parameter"))
            .with_param(DspParamSchema::knob("itd_us", "Interaural Time Difference", -700.0, 700.0, 0.0, "us", MacroRole::Tone, "Interaural Time Difference parameter"))
            .with_param(DspParamSchema::knob("ild_db", "Interaural Level Difference", -24.0, 24.0, 0.0, "dB", MacroRole::Tone, "Interaural Level Difference parameter"))
        );
        descriptors.insert("WavefrontReflectionNode".to_string(), DspNodeDescriptor::new("WavefrontReflectionNode", "Early Specular & Diffuse Wavefront Room Reflection Engine", DspNodeCategory::SpatialSurround, "Early Specular & Diffuse Wavefront Room Reflection Engine")
            .with_param(DspParamSchema::knob("room_dimensions_xyz", "Room Scale Multiplier", 0.5, 5.0, 1.0, "x", MacroRole::Tone, "Room Scale Multiplier parameter"))
            .with_param(DspParamSchema::knob("wall_absorption_coeff", "Boundary Absorption", 0.05, 0.95, 0.2, "", MacroRole::Tone, "Boundary Absorption parameter"))
            .with_param(DspParamSchema::knob("specular_order", "Ray Image Source Order", 1.0, 8.0, 4.0, "orders", MacroRole::Character, "Ray Image Source Order integer control"))
            .with_param(DspParamSchema::knob("diffuse_scatter_mix", "Diffuse Lambertian Scatter", 0.0, 1.0, 0.4, "%", MacroRole::Tone, "Diffuse Lambertian Scatter parameter"))
        );
        descriptors.insert("ReverbSpaceNode".to_string(), DspNodeDescriptor::new("ReverbSpaceNode", "Parametric Multi-Room Acoustic Space Simulator", DspNodeCategory::TimeSpace, "Parametric Multi-Room Acoustic Space Simulator")
            .with_param(DspParamSchema::choice("space_geometry", "Room Architecture Profile", &["Concert Hall (Gold)", "Cathedral Nave", "Recording Studio Live Room", "Tiled Chamber"], 0, "Room Architecture Profile mode selector"))
            .with_param(DspParamSchema::knob("decay_time_t60", "Reverberant T60 Time", 0.3, 20.0, 2.6, "s", MacroRole::Tone, "Reverberant T60 Time parameter"))
            .with_param(DspParamSchema::knob("early_to_late_ratio", "Early to Late Diffuse Mix", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Early to Late Diffuse Mix parameter"))
            .with_param(DspParamSchema::log_knob("high_damping_hz", "High-Frequency Air Loss Cut", 1000.0, 20000.0, 6500.0, "Hz", MacroRole::Tone, "High-Frequency Air Loss Cut logarithmic parameter"))
        );
        descriptors.insert("MultibandSpatialNode".to_string(), DspNodeDescriptor::new("MultibandSpatialNode", "Tri-Band Frequency-Dependent Stereo & Spatial Panner", DspNodeCategory::SpatialSurround, "Tri-Band Frequency-Dependent Stereo & Spatial Panner")
            .with_param(DspParamSchema::knob("low_band_pan", "Low Band Panning (<200Hz)", -1.0, 1.0, 0.0, "pan", MacroRole::Tone, "Low Band Panning (<200Hz) parameter"))
            .with_param(DspParamSchema::knob("mid_band_pan", "Mid Band Panning", -1.0, 1.0, 0.25, "pan", MacroRole::Tone, "Mid Band Panning parameter"))
            .with_param(DspParamSchema::knob("high_band_pan", "High Band Panning (>4kHz)", -1.0, 1.0, -0.3, "pan", MacroRole::Tone, "High Band Panning (>4kHz) parameter"))
            .with_param(DspParamSchema::log_knob("crossover_freq_hz", "Mid-High Crossover Point", 500.0, 8000.0, 2500.0, "Hz", MacroRole::Tone, "Mid-High Crossover Point logarithmic parameter"))
        );
        descriptors.insert("MultitapDelayNode".to_string(), DspNodeDescriptor::new("MultitapDelayNode", "8-Tap Modulated Stereo Spatial Delay Line", DspNodeCategory::TimeSpace, "8-Tap Modulated Stereo Spatial Delay Line")
            .with_param(DspParamSchema::knob("tap_count", "Active Delay Taps", 2.0, 8.0, 8.0, "taps", MacroRole::Character, "Active Delay Taps integer control"))
            .with_param(DspParamSchema::knob("spatial_spread", "Stereo Tap Ping-Pong Spread", 0.0, 1.0, 0.85, "%", MacroRole::Tone, "Stereo Tap Ping-Pong Spread parameter"))
            .with_param(DspParamSchema::knob("feedback_matrix_gain", "Cross-Tap Feedback", 0.0, 0.95, 0.45, "%", MacroRole::Tone, "Cross-Tap Feedback parameter"))
            .with_param(DspParamSchema::knob("modulation_depth", "Tap Delay Time LFO Wobble", 0.0, 1.0, 0.2, "%", MacroRole::Tone, "Tap Delay Time LFO Wobble parameter"))
        );
        descriptors.insert("LimiterNode".to_string(), DspNodeDescriptor::new("LimiterNode", "True-Peak Lookahead Mastering Brickwall Limiter", DspNodeCategory::DynamicsMaster, "True-Peak Lookahead Mastering Brickwall Limiter")
            .with_param(DspParamSchema::knob("ceiling", "True Peak Ceiling", -12.0, 0.0, -0.3, "dBFS", MacroRole::Tone, "True Peak Ceiling parameter"))
            .with_param(DspParamSchema::knob("threshold", "Limiter Threshold", -24.0, 0.0, -3.0, "dB", MacroRole::Tone, "Limiter Threshold parameter"))
            .with_param(DspParamSchema::knob("release_ms", "Lookahead Release", 5.0, 500.0, 50.0, "ms", MacroRole::Tone, "Lookahead Release parameter"))
            .with_param(DspParamSchema::toggle("oversampling", "4x ISP Oversampling", true, "4x ISP Oversampling toggle switch"))
            .with_param(DspParamSchema::knob("lookahead_ms", "Lookahead Buffer", 0.5, 10.0, 3.0, "ms", MacroRole::Tone, "Lookahead Buffer parameter"))
        );
        descriptors.insert("EbuLoudnessRadar".to_string(), DspNodeDescriptor::new("EbuLoudnessRadar", "ITU-R BS.1770 / EBU R128 Loudness Radar Meter", DspNodeCategory::DynamicsMaster, "ITU-R BS.1770 / EBU R128 Loudness Radar Meter")
            .with_param(DspParamSchema::knob("target_lufs", "Target Integrated Loudness", -24.0, -9.0, -14.0, "LUFS", MacroRole::Tone, "Target Integrated Loudness parameter"))
            .with_param(DspParamSchema::knob("integrated_gate", "Relative Silence Gate", -70.0, -10.0, -23.0, "LUFS", MacroRole::Tone, "Relative Silence Gate parameter"))
            .with_param(DspParamSchema::knob("max_momentary", "Max Momentary True Peak", -18.0, 0.0, -1.0, "dBTP", MacroRole::Tone, "Max Momentary True Peak parameter"))
        );
        descriptors.insert("LufsMeterNode".to_string(), DspNodeDescriptor::new("LufsMeterNode", "Multi-Scale Momentary & Integrated LUFS Meter", DspNodeCategory::DynamicsMaster, "Multi-Scale Momentary & Integrated LUFS Meter")
            .with_param(DspParamSchema::knob("momentary_window", "Momentary Window", 100.0, 1000.0, 400.0, "ms", MacroRole::Tone, "Momentary Window parameter"))
            .with_param(DspParamSchema::knob("short_term_window", "Short-Term Window", 1000.0, 5000.0, 3000.0, "ms", MacroRole::Tone, "Short-Term Window parameter"))
            .with_param(DspParamSchema::knob("relative_threshold", "Relative Gate Threshold", -20.0, -5.0, -10.0, "LU", MacroRole::Tone, "Relative Gate Threshold parameter"))
        );
        descriptors.insert("MidSideNode".to_string(), DspNodeDescriptor::new("MidSideNode", "Mid/Side Matrix Encoder & Stereo Width Imager", DspNodeCategory::DynamicsMaster, "Mid/Side Matrix Encoder & Stereo Width Imager")
            .with_param(DspParamSchema::knob("width", "Stereo Field Width", 0.0, 3.0, 1.0, "x", MacroRole::Tone, "Stereo Field Width parameter"))
            .with_param(DspParamSchema::knob("mid_gain", "Mid Channel Gain", -12.0, 12.0, 0.0, "dB", MacroRole::Tone, "Mid Channel Gain parameter"))
            .with_param(DspParamSchema::knob("side_gain", "Side Channel Gain", -12.0, 12.0, 0.0, "dB", MacroRole::Tone, "Side Channel Gain parameter"))
            .with_param(DspParamSchema::log_knob("mono_sub", "Mono Bass Crossover", 20.0, 250.0, 90.0, "Hz", MacroRole::Tone, "Mono Bass Crossover logarithmic parameter"))
        );
        descriptors.insert("StereoImager".to_string(), DspNodeDescriptor::new("StereoImager", "Phase-Safe Psychoacoustic Stereo Imager", DspNodeCategory::DynamicsMaster, "Phase-Safe Psychoacoustic Stereo Imager")
            .with_param(DspParamSchema::knob("width_factor", "Stereo Width Expansion", 0.0, 2.5, 1.2, "x", MacroRole::Tone, "Stereo Width Expansion parameter"))
            .with_param(DspParamSchema::log_knob("bass_mono_cutoff", "Bass Mono Filter Corner", 30.0, 300.0, 120.0, "Hz", MacroRole::Tone, "Bass Mono Filter Corner logarithmic parameter"))
            .with_param(DspParamSchema::toggle("phase_correlation_protect", "Phase Correlation Protection", true, "Phase Correlation Protection toggle switch"))
        );
        descriptors.insert("TpdfDitherNode".to_string(), DspNodeDescriptor::new("TpdfDitherNode", "Triangular PDF Dither & Noise Shaping Quantizer", DspNodeCategory::DynamicsMaster, "Triangular PDF Dither & Noise Shaping Quantizer")
            .with_param(DspParamSchema::choice("bit_depth", "Target Bit Depth", &["24-bit HD", "16-bit Red Book CD", "12-bit Vintage", "8-bit Lo-Fi"], 1, "Target Bit Depth mode selector"))
            .with_param(DspParamSchema::choice("noise_shaping", "Noise Shaping Curve", &["None (Flat TPDF)", "Lipshitz Moderate", "High-Pass Ultra", "Psychoacoustic F-Weight"], 1, "Noise Shaping Curve mode selector"))
            .with_param(DspParamSchema::toggle("auto_blank", "Auto Blank On Digital Silence", true, "Auto Blank On Digital Silence toggle switch"))
        );
        descriptors.insert("AdaptivePsychoacousticLoudness".to_string(), DspNodeDescriptor::new("AdaptivePsychoacousticLoudness", "ISO 226 Equal Loudness Contour Balancing Engine", DspNodeCategory::DynamicsMaster, "ISO 226 Equal Loudness Contour Balancing Engine")
            .with_param(DspParamSchema::knob("target_phon", "Target Phon Loudness", 40.0, 100.0, 75.0, "phon", MacroRole::Tone, "Target Phon Loudness parameter"))
            .with_param(DspParamSchema::knob("phon_compensation_intensity", "Equal Loudness Compensation", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Equal Loudness Compensation parameter"))
            .with_param(DspParamSchema::knob("low_contour_boost_db", "Fletcher-Munson Sub Boost", 0.0, 12.0, 3.5, "dB", MacroRole::Tone, "Fletcher-Munson Sub Boost parameter"))
            .with_param(DspParamSchema::knob("high_contour_presence_db", "Treble Contour Presence", 0.0, 6.0, 1.8, "dB", MacroRole::Tone, "Treble Contour Presence parameter"))
        );
        descriptors.insert("DynamicStereoWidthNode".to_string(), DspNodeDescriptor::new("DynamicStereoWidthNode", "Frequency-Dependent Mid/Side Dynamic Stereo Widener", DspNodeCategory::DynamicsMaster, "Frequency-Dependent Mid/Side Dynamic Stereo Widener")
            .with_param(DspParamSchema::knob("stereo_expansion_factor", "Side Band Expansion", 0.0, 3.0, 1.35, "x", MacroRole::Tone, "Side Band Expansion parameter"))
            .with_param(DspParamSchema::log_knob("bass_mono_crossover_hz", "Mono Bass Frequency Cut", 30.0, 250.0, 100.0, "Hz", MacroRole::Tone, "Mono Bass Frequency Cut logarithmic parameter"))
            .with_param(DspParamSchema::knob("high_side_air_gain_db", "Side Channel Air Boost", 0.0, 6.0, 1.5, "dB", MacroRole::Tone, "Side Channel Air Boost parameter"))
            .with_param(DspParamSchema::toggle("phase_correlation_protect", "Auto Anti-Phase Limiter", true, "Auto Anti-Phase Limiter toggle switch"))
        );
        descriptors.insert("KSystemMeterNode".to_string(), DspNodeDescriptor::new("KSystemMeterNode", "Bob Katz K-12 / K-14 / K-20 Metering Bridge", DspNodeCategory::DynamicsMaster, "Bob Katz K-12 / K-14 / K-20 Metering Bridge")
            .with_param(DspParamSchema::choice("meter_scale", "K-Scale Target Reference", &["K-12 (Broadcast / Pop)", "K-14 (Standard Music)", "K-20 (Audiophile / Film Dynamic)"], 1, "K-Scale Target Reference mode selector"))
            .with_param(DspParamSchema::knob("headroom_target_db", "0 VU Calibration Offset", 0.0, 20.0, 14.0, "dB", MacroRole::Tone, "0 VU Calibration Offset parameter"))
            .with_param(DspParamSchema::knob("peak_hold_time_ms", "Peak Hold Persistence", 100.0, 3000.0, 1500.0, "ms", MacroRole::Tone, "Peak Hold Persistence parameter"))
            .with_param(DspParamSchema::knob("monitor_calibration_db", "SPL Reference Calibration", 60.0, 90.0, 83.0, "dBSPL", MacroRole::Tone, "SPL Reference Calibration parameter"))
        );
        descriptors.insert("AuditoryRoughnessNode".to_string(), DspNodeDescriptor::new("AuditoryRoughnessNode", "Psychoacoustic Plomp-Levelt Sensory Dissonance Analyzer", DspNodeCategory::SpectralResynthesis, "Psychoacoustic Plomp-Levelt Sensory Dissonance Analyzer")
            .with_param(DspParamSchema::knob("dissonance_threshold", "Roughness Detection Sens", 0.0, 1.0, 0.65, "%", MacroRole::Tone, "Roughness Detection Sens parameter"))
            .with_param(DspParamSchema::knob("roughness_scaling", "Plomp-Levelt Critical Scale", 0.1, 3.0, 1.0, "x", MacroRole::Tone, "Plomp-Levelt Critical Scale parameter"))
            .with_param(DspParamSchema::choice("critical_bandwidth_mode", "Filter Bank Standard", &["Bark Scale (Zwicker)", "ERB Scale (Moore-Glasberg)", "Mel Scale"], 1, "Filter Bank Standard mode selector"))
            .with_param(DspParamSchema::knob("masking_depth", "Simultaneous Masking Depth", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Simultaneous Masking Depth parameter"))
        );
        descriptors.insert("EqualLoudnessContourNode".to_string(), DspNodeDescriptor::new("EqualLoudnessContourNode", "Fletcher-Munson Psychoacoustic Equal Loudness Compensator", DspNodeCategory::DynamicsMaster, "Fletcher-Munson Psychoacoustic Equal Loudness Compensator")
            .with_param(DspParamSchema::knob("phon_level", "Listening Volume (Phon)", 30.0, 100.0, 80.0, "phon", MacroRole::Tone, "Listening Volume (Phon) parameter"))
            .with_param(DspParamSchema::choice("curve_version", "Contour Standard", &["ISO 226:2003 Standard", "Fletcher-Munson (1933)", "Robinson-Dadson (1956)"], 0, "Contour Standard mode selector"))
            .with_param(DspParamSchema::knob("bass_contour_trim", "Low Contour Trim", -6.0, 6.0, 0.0, "dB", MacroRole::Tone, "Low Contour Trim parameter"))
            .with_param(DspParamSchema::knob("treble_contour_trim", "High Contour Trim", -6.0, 6.0, 0.0, "dB", MacroRole::Tone, "High Contour Trim parameter"))
        );
        descriptors.insert("MasterLimiterRadarNode".to_string(), DspNodeDescriptor::new("MasterLimiterRadarNode", "True-Peak Limiter with Integrated Loudness Radar", DspNodeCategory::DynamicsMaster, "True-Peak Limiter with Integrated Loudness Radar")
            .with_param(DspParamSchema::knob("true_peak_ceiling_dbtp", "True-Peak Ceiling", -6.0, 0.0, -0.3, "dBTP", MacroRole::Tone, "True-Peak Ceiling parameter"))
            .with_param(DspParamSchema::knob("target_integrated_lufs", "Target Integrated LUFS", -24.0, -8.0, -14.0, "LUFS", MacroRole::Tone, "Target Integrated LUFS parameter"))
            .with_param(DspParamSchema::knob("lookahead_time_ms", "True-Peak Lookahead", 1.0, 10.0, 4.0, "ms", MacroRole::Tone, "True-Peak Lookahead parameter"))
            .with_param(DspParamSchema::toggle("release_curve_adaptive", "Adaptive Program Release", true, "Adaptive Program Release toggle switch"))
        );
        descriptors.insert("MeterBridgeNode".to_string(), DspNodeDescriptor::new("MeterBridgeNode", "32-Channel VU, RMS, & True-Peak Mastering Meter Bridge", DspNodeCategory::DynamicsMaster, "32-Channel VU, RMS, & True-Peak Mastering Meter Bridge")
            .with_param(DspParamSchema::choice("meter_type", "Ballistics Standard", &["True-Peak ISP (ITU-R BS.1770)", "VU Meter (ANSI C16.5)", "Nordic PPM (IEC 60268-10)", "BBC Type II PPM"], 0, "Ballistics Standard mode selector"))
            .with_param(DspParamSchema::knob("integration_window_ms", "RMS Integration Window", 50.0, 1000.0, 300.0, "ms", MacroRole::Tone, "RMS Integration Window parameter"))
            .with_param(DspParamSchema::knob("vu_reference_db", "VU Zero Reference", -24.0, -10.0, -18.0, "dBFS", MacroRole::Tone, "VU Zero Reference parameter"))
            .with_param(DspParamSchema::knob("peak_decay_speed", "Decay Ballistics Rate", 5.0, 60.0, 20.0, "dB/s", MacroRole::Tone, "Decay Ballistics Rate parameter"))
        );
        descriptors.insert("MidSideFocuserNode".to_string(), DspNodeDescriptor::new("MidSideFocuserNode", "Surgical Mid/Side Bass Mono & High-Side Air Focuser", DspNodeCategory::DynamicsMaster, "Surgical Mid/Side Bass Mono & High-Side Air Focuser")
            .with_param(DspParamSchema::log_knob("bass_mono_freq_hz", "Mono Bass Frequency Cut", 30.0, 300.0, 110.0, "Hz", MacroRole::Tone, "Mono Bass Frequency Cut logarithmic parameter"))
            .with_param(DspParamSchema::knob("high_side_boost_db", "Side Channel Air Sheen", 0.0, 6.0, 1.8, "dB", MacroRole::Tone, "Side Channel Air Sheen parameter"))
            .with_param(DspParamSchema::knob("side_dynamic_compression", "Side Band Dynamic Tame", 0.0, 1.0, 0.4, "%", MacroRole::Tone, "Side Band Dynamic Tame parameter"))
            .with_param(DspParamSchema::knob("mid_presence_gain_db", "Mid Channel Vocal Presence", -6.0, 6.0, 0.8, "dB", MacroRole::Tone, "Mid Channel Vocal Presence parameter"))
        );
        descriptors.insert("MultibandImagerNode".to_string(), DspNodeDescriptor::new("MultibandImagerNode", "4-Band Mastering Stereo Width & Phase Correlator", DspNodeCategory::DynamicsMaster, "4-Band Mastering Stereo Width & Phase Correlator")
            .with_param(DspParamSchema::knob("band1_sub_width", "Sub Width (<120Hz)", 0.0, 2.0, 0.0, "x", MacroRole::Tone, "Sub Width (<120Hz) parameter"))
            .with_param(DspParamSchema::knob("band2_low_width", "Low-Mid Width (120-1kHz)", 0.0, 2.0, 0.9, "x", MacroRole::Tone, "Low-Mid Width (120-1kHz) parameter"))
            .with_param(DspParamSchema::knob("band3_mid_width", "High-Mid Width (1k-6kHz)", 0.0, 2.0, 1.25, "x", MacroRole::Tone, "High-Mid Width (1k-6kHz) parameter"))
            .with_param(DspParamSchema::knob("band4_high_width", "Air Width (>6kHz)", 0.0, 2.0, 1.4, "x", MacroRole::Tone, "Air Width (>6kHz) parameter"))
        );
        descriptors.insert("OversampledLimiterNode".to_string(), DspNodeDescriptor::new("OversampledLimiterNode", "16x Polyphase Oversampled Intersample Peak Limiter", DspNodeCategory::DynamicsMaster, "16x Polyphase Oversampled Intersample Peak Limiter")
            .with_param(DspParamSchema::choice("oversampling_rate", "Polyphase Oversampling", &["2x Polyphase", "4x High Quality", "8x Mastering Grade", "16x Ultra Fidelity"], 2, "Polyphase Oversampling mode selector"))
            .with_param(DspParamSchema::knob("brickwall_ceiling_dbfs", "Ceiling Level", -6.0, 0.0, -0.2, "dBFS", MacroRole::Tone, "Ceiling Level parameter"))
            .with_param(DspParamSchema::knob("release_time_ms", "Lookahead Release Speed", 5.0, 400.0, 45.0, "ms", MacroRole::Tone, "Lookahead Release Speed parameter"))
            .with_param(DspParamSchema::toggle("isp_guard_active", "Intersample Peak Guard", true, "Intersample Peak Guard toggle switch"))
        );
        descriptors.insert("PolarPhaseCorrelatorNode".to_string(), DspNodeDescriptor::new("PolarPhaseCorrelatorNode", "Polar Goniometer Lissajous Phase Correlation Display", DspNodeCategory::DynamicsMaster, "Polar Goniometer Lissajous Phase Correlation Display")
            .with_param(DspParamSchema::knob("phosphor_persistence_ms", "Goniometer Phosphor Decay", 50.0, 1000.0, 300.0, "ms", MacroRole::Tone, "Goniometer Phosphor Decay parameter"))
            .with_param(DspParamSchema::knob("polar_scaling", "Polar Radius Scale", 0.5, 3.0, 1.0, "x", MacroRole::Tone, "Polar Radius Scale parameter"))
            .with_param(DspParamSchema::knob("lissajous_resolution", "Display Point Resolution", 128.0, 2048.0, 512.0, "pts", MacroRole::Character, "Display Point Resolution integer control"))
            .with_param(DspParamSchema::toggle("phase_cancel_alert", "Phase Inversion Warning", true, "Phase Inversion Warning toggle switch"))
        );
        descriptors.insert("ResonanceSuppressorNode".to_string(), DspNodeDescriptor::new("ResonanceSuppressorNode", "Automatic Dynamic Harmonic Notch Resonance Suppressor", DspNodeCategory::DynamicsMaster, "Automatic Dynamic Harmonic Notch Resonance Suppressor")
            .with_param(DspParamSchema::knob("suppression_depth_db", "Dynamic Notch Depth", 0.0, 18.0, 6.0, "dB", MacroRole::Tone, "Dynamic Notch Depth parameter"))
            .with_param(DspParamSchema::knob("notch_q_sharpness", "Dynamic Q Sharpness", 2.0, 50.0, 15.0, "Q", MacroRole::Tone, "Dynamic Q Sharpness parameter"))
            .with_param(DspParamSchema::knob("attack_speed_ms", "Resonance Lock Attack", 1.0, 100.0, 12.0, "ms", MacroRole::Tone, "Resonance Lock Attack parameter"))
            .with_param(DspParamSchema::knob("max_active_notches", "Concurrent Notch Filters", 1.0, 16.0, 6.0, "filters", MacroRole::Character, "Concurrent Notch Filters integer control"))
        );
        descriptors.insert("SpectralAlignerNode".to_string(), DspNodeDescriptor::new("SpectralAlignerNode", "Multi-Track Phase & Spectral Coherence Auto-Aligner", DspNodeCategory::SpectralResynthesis, "Multi-Track Phase & Spectral Coherence Auto-Aligner")
            .with_param(DspParamSchema::knob("reference_track_id", "Reference Master Track ID", 0.0, 64.0, 0.0, "trk", MacroRole::Character, "Reference Master Track ID integer control"))
            .with_param(DspParamSchema::knob("phase_correlation_target", "Target Phase Alignment", 0.5, 1.0, 0.95, "%", MacroRole::Tone, "Target Phase Alignment parameter"))
            .with_param(DspParamSchema::knob("delay_offset_samples", "Sub-Sample Delay Offset", -500.0, 500.0, 0.0, "samples", MacroRole::Tone, "Sub-Sample Delay Offset parameter"))
            .with_param(DspParamSchema::toggle("polarity_auto_invert", "Auto Polarity Inversion", true, "Auto Polarity Inversion toggle switch"))
        );
        descriptors.insert("SpectralDebleedNode".to_string(), DspNodeDescriptor::new("SpectralDebleedNode", "Microphone Acoustic Bleed Elimination Matrix", DspNodeCategory::SpectralResynthesis, "Microphone Acoustic Bleed Elimination Matrix")
            .with_param(DspParamSchema::knob("bleed_suppression_db", "Bleed Isolation Depth", 0.0, 30.0, 14.0, "dB", MacroRole::Tone, "Bleed Isolation Depth parameter"))
            .with_param(DspParamSchema::knob("spectral_gate_threshold", "Spectral Gate Floor", -60.0, -10.0, -35.0, "dB", MacroRole::Tone, "Spectral Gate Floor parameter"))
            .with_param(DspParamSchema::knob("crosstalk_estimate_ms", "Acoustic Delay Estimate", 0.1, 50.0, 8.5, "ms", MacroRole::Tone, "Acoustic Delay Estimate parameter"))
            .with_param(DspParamSchema::toggle("transient_preserve", "Preserve Direct Transients", true, "Preserve Direct Transients toggle switch"))
        );
        descriptors.insert("SpectralDeEsserNode".to_string(), DspNodeDescriptor::new("SpectralDeEsserNode", "Dynamic Spectral Sibilance & Harshness Reducer", DspNodeCategory::SpectralResynthesis, "Dynamic Spectral Sibilance & Harshness Reducer")
            .with_param(DspParamSchema::log_knob("sibilance_center_hz", "Harshness Center Freq", 3000.0, 14000.0, 7200.0, "Hz", MacroRole::Tone, "Harshness Center Freq logarithmic parameter"))
            .with_param(DspParamSchema::knob("bandwidth_oct", "Sibilance Bandwidth", 0.2, 3.0, 1.2, "oct", MacroRole::Tone, "Sibilance Bandwidth parameter"))
            .with_param(DspParamSchema::knob("max_reduction_db", "Max Dynamic Reduction", 0.0, 24.0, 9.0, "dB", MacroRole::Tone, "Max Dynamic Reduction parameter"))
            .with_param(DspParamSchema::knob("lookahead_ms", "Lookahead Buffer Time", 0.0, 10.0, 2.5, "ms", MacroRole::Tone, "Lookahead Buffer Time parameter"))
        );
        descriptors.insert("SpectralFlatnessNode".to_string(), DspNodeDescriptor::new("SpectralFlatnessNode", "Wiener Entropy Spectral Flatness & Tonal/Noise Analyzer", DspNodeCategory::SpectralResynthesis, "Wiener Entropy Spectral Flatness & Tonal/Noise Analyzer")
            .with_param(DspParamSchema::knob("entropy_threshold", "Spectral Flatness Cutoff", 0.0, 1.0, 0.45, "entropy", MacroRole::Tone, "Spectral Flatness Cutoff parameter"))
            .with_param(DspParamSchema::knob("analysis_window_size", "FFT Window Analysis Size", 256.0, 4096.0, 1024.0, "bins", MacroRole::Character, "FFT Window Analysis Size integer control"))
            .with_param(DspParamSchema::knob("tonal_split_gain", "Tonal Harmonic Level", -12.0, 12.0, 0.0, "dB", MacroRole::Tone, "Tonal Harmonic Level parameter"))
            .with_param(DspParamSchema::knob("noise_split_gain", "Noise Residual Level", -12.0, 12.0, 0.0, "dB", MacroRole::Tone, "Noise Residual Level parameter"))
        );
        descriptors.insert("SpectralGrainCloudNode".to_string(), DspNodeDescriptor::new("SpectralGrainCloudNode", "Spectral Domain Grain Scatter & Diffusion Engine", DspNodeCategory::SpectralResynthesis, "Spectral Domain Grain Scatter & Diffusion Engine")
            .with_param(DspParamSchema::knob("grain_density_spectral", "Frequency Bin Grains", 10.0, 500.0, 120.0, "grains/s", MacroRole::Tone, "Frequency Bin Grains parameter"))
            .with_param(DspParamSchema::knob("time_smear_factor", "Phase Time Smear", 0.0, 1.0, 0.35, "%", MacroRole::Tone, "Phase Time Smear parameter"))
            .with_param(DspParamSchema::knob("phase_randomization", "Bin Phase Chaos", 0.0, 1.0, 0.4, "%", MacroRole::Tone, "Bin Phase Chaos parameter"))
            .with_param(DspParamSchema::knob("diffusion_wet_mix", "Grain Cloud Blend", 0.0, 1.0, 0.3, "%", MacroRole::Tone, "Grain Cloud Blend parameter"))
        );
        descriptors.insert("SpectralMaskingNode".to_string(), DspNodeDescriptor::new("SpectralMaskingNode", "Psychoacoustic Simultaneous Spectral Masking Visualizer", DspNodeCategory::SpectralResynthesis, "Psychoacoustic Simultaneous Spectral Masking Visualizer")
            .with_param(DspParamSchema::knob("masking_threshold_offset_db", "Masking Threshold Bias", -12.0, 12.0, 0.0, "dB", MacroRole::Tone, "Masking Threshold Bias parameter"))
            .with_param(DspParamSchema::knob("critical_band_scale", "Critical Band Width Scale", 0.5, 2.0, 1.0, "x", MacroRole::Tone, "Critical Band Width Scale parameter"))
            .with_param(DspParamSchema::choice("bark_scale_mode", "Psychoacoustic Standard", &["Bark Scale Matrix", "ERB Auditory Filter", "Equivalent Rectangular Band"], 0, "Psychoacoustic Standard mode selector"))
            .with_param(DspParamSchema::knob("alert_color_intensity", "Masking Flare Brightness", 0.0, 1.0, 0.85, "%", MacroRole::Tone, "Masking Flare Brightness parameter"))
        );
        descriptors.insert("SpectralMorphNode".to_string(), DspNodeDescriptor::new("SpectralMorphNode", "Latent FFT Magnitude & Phase Interpolation Morph Engine", DspNodeCategory::SpectralResynthesis, "Latent FFT Magnitude & Phase Interpolation Morph Engine")
            .with_param(DspParamSchema::knob("morph_position", "Spectral Morph Interpolation", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Spectral Morph Interpolation parameter"))
            .with_param(DspParamSchema::knob("spectral_smear", "Cross-Bin Spectral Smear", 0.0, 1.0, 0.2, "%", MacroRole::Tone, "Cross-Bin Spectral Smear parameter"))
            .with_param(DspParamSchema::choice("phase_alignment_mode", "Phase Interpolation Mode", &["Magnitude Only (Zero Phase)", "Phase Unwrapped Morph", "Cross-Correlation Aligned"], 1, "Phase Interpolation Mode mode selector"))
            .with_param(DspParamSchema::toggle("harmonic_lock", "Lock Harmonic Partials", true, "Lock Harmonic Partials toggle switch"))
        );
        descriptors.insert("SpectralReshaperNode".to_string(), DspNodeDescriptor::new("SpectralReshaperNode", "Dynamic Spectral Envelope & Formant Reshaper", DspNodeCategory::SpectralResynthesis, "Dynamic Spectral Envelope & Formant Reshaper")
            .with_param(DspParamSchema::knob("formant_shift_semitones", "Spectral Envelope Shift", -24.0, 24.0, 0.0, "st", MacroRole::Tone, "Spectral Envelope Shift parameter"))
            .with_param(DspParamSchema::knob("spectral_envelope_gain", "Envelope Reshape Gain", -18.0, 18.0, 0.0, "dB", MacroRole::Tone, "Envelope Reshape Gain parameter"))
            .with_param(DspParamSchema::knob("smoothing_factor", "Envelope Cepstral Smoothing", 0.1, 5.0, 1.2, "oct", MacroRole::Tone, "Envelope Cepstral Smoothing parameter"))
            .with_param(DspParamSchema::toggle("dynamic_tracking", "Dynamic Pitch Tracking", true, "Dynamic Pitch Tracking toggle switch"))
        );
        descriptors.insert("SpectralResynthesisNode".to_string(), DspNodeDescriptor::new("SpectralResynthesisNode", "Harmonic Additive Spectral Resynthesis Engine", DspNodeCategory::SpectralResynthesis, "Harmonic Additive Spectral Resynthesis Engine")
            .with_param(DspParamSchema::knob("max_partials_count", "Additive Sinusoidal Partials", 8.0, 256.0, 64.0, "partials", MacroRole::Character, "Additive Sinusoidal Partials integer control"))
            .with_param(DspParamSchema::knob("spectral_threshold_db", "Noise Floor Cutoff", -80.0, -20.0, -50.0, "dB", MacroRole::Tone, "Noise Floor Cutoff parameter"))
            .with_param(DspParamSchema::knob("inharmonic_drift", "Partial Inharmonicity", 0.0, 2.0, 0.0, "", MacroRole::Tone, "Partial Inharmonicity parameter"))
            .with_param(DspParamSchema::knob("harmonic_additive_mix", "Synthesized Partials Level", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Synthesized Partials Level parameter"))
        );
        descriptors.insert("SpectralUnmaskerNode".to_string(), DspNodeDescriptor::new("SpectralUnmaskerNode", "AI-Driven Intelligent Sidechain Spectral Unmasker", DspNodeCategory::SpectralResynthesis, "AI-Driven Intelligent Sidechain Spectral Unmasker")
            .with_param(DspParamSchema::knob("sidechain_input_id", "Sidechain Trigger Track", 0.0, 64.0, 1.0, "trk", MacroRole::Character, "Sidechain Trigger Track integer control"))
            .with_param(DspParamSchema::knob("unmasking_depth_db", "Dynamic Unmask Depth", 0.0, 18.0, 5.0, "dB", MacroRole::Tone, "Dynamic Unmask Depth parameter"))
            .with_param(DspParamSchema::knob("frequency_resolution_bins", "Spectral Bin Resolution", 32.0, 512.0, 128.0, "bins", MacroRole::Character, "Spectral Bin Resolution integer control"))
            .with_param(DspParamSchema::knob("adaptation_speed_ms", "Fast Reaction Speed", 5.0, 250.0, 35.0, "ms", MacroRole::Tone, "Fast Reaction Speed parameter"))
        );
        descriptors.insert("Spectrogram3DNode".to_string(), DspNodeDescriptor::new("Spectrogram3DNode", "3D Waterfall Real-Time Spectrogram & Sonogram Analyzer", DspNodeCategory::SpectralResynthesis, "3D Waterfall Real-Time Spectrogram & Sonogram Analyzer")
            .with_param(DspParamSchema::knob("fft_size_bins", "FFT Analysis Size", 256.0, 4096.0, 1024.0, "bins", MacroRole::Character, "FFT Analysis Size integer control"))
            .with_param(DspParamSchema::knob("waterfall_depth_frames", "Waterfall History Buffer", 30.0, 300.0, 120.0, "frames", MacroRole::Character, "Waterfall History Buffer integer control"))
            .with_param(DspParamSchema::choice("color_palette", "Spectrogram Palette", &["Cyberpunk Neon", "Thermal Infrared", "Obsidian Ice", "Grayscale Contrast"], 0, "Spectrogram Palette mode selector"))
            .with_param(DspParamSchema::toggle("frequency_scale_log", "Logarithmic Frequency Grid", true, "Logarithmic Frequency Grid toggle switch"))
        );
        descriptors.insert("StereoVectorscopeNode".to_string(), DspNodeDescriptor::new("StereoVectorscopeNode", "High-Speed Phosphor Stereophonic Vectorscope Analyzer", DspNodeCategory::SpectralResynthesis, "High-Speed Phosphor Stereophonic Vectorscope Analyzer")
            .with_param(DspParamSchema::knob("phosphor_decay_rate", "Phosphor Glow Persistence", 0.05, 1.0, 0.35, "s", MacroRole::Tone, "Phosphor Glow Persistence parameter"))
            .with_param(DspParamSchema::knob("vectorscope_gain", "Lissajous Input Scale", 0.5, 4.0, 1.0, "x", MacroRole::Tone, "Lissajous Input Scale parameter"))
            .with_param(DspParamSchema::knob("lissajous_dot_size", "Vector Trace Thickness", 1.0, 5.0, 1.8, "pt", MacroRole::Tone, "Vector Trace Thickness parameter"))
            .with_param(DspParamSchema::toggle("correlation_meter_bar", "Show Phase Bar Meter", true, "Show Phase Bar Meter toggle switch"))
        );
        descriptors.insert("StereoWidenerNode".to_string(), DspNodeDescriptor::new("StereoWidenerNode", "Haas Effect & Phase-Coherent Stereo Field Widener", DspNodeCategory::DynamicsMaster, "Haas Effect & Phase-Coherent Stereo Field Widener")
            .with_param(DspParamSchema::knob("haas_delay_ms", "Haas Delay Offset", 0.1, 30.0, 12.0, "ms", MacroRole::Tone, "Haas Delay Offset parameter"))
            .with_param(DspParamSchema::knob("mid_side_ratio", "Side vs Mid Ratio", 0.0, 3.0, 1.25, "x", MacroRole::Tone, "Side vs Mid Ratio parameter"))
            .with_param(DspParamSchema::knob("center_channel_retention", "Center Lead Vocal Retain", 0.0, 1.0, 0.85, "%", MacroRole::Tone, "Center Lead Vocal Retain parameter"))
            .with_param(DspParamSchema::knob("phase_filter_q", "Allpass Phase Filter Q", 0.5, 5.0, 1.2, "Q", MacroRole::Tone, "Allpass Phase Filter Q parameter"))
        );
        descriptors.insert("TapeFluxMasterNode".to_string(), DspNodeDescriptor::new("TapeFluxMasterNode", "Half-Inch Mastering Tape Saturation & Flux Density Engine", DspNodeCategory::DynamicsMaster, "Half-Inch Mastering Tape Saturation & Flux Density Engine")
            .with_param(DspParamSchema::knob("tape_flux_nanoweber", "Reference Fluxivity", 185.0, 510.0, 355.0, "nWb/m", MacroRole::Tone, "Reference Fluxivity parameter"))
            .with_param(DspParamSchema::choice("tape_formulation", "Mastering Tape Stock", &["SM900 High Output (+9dB)", "ATR Magnetics Master (+6dB)", "GP9 Grand Master (+9dB)", "Scotch 250 Vintage (+3dB)"], 0, "Mastering Tape Stock mode selector"))
            .with_param(DspParamSchema::knob("bias_hf_saturation", "High-Frequency Bias Saturation", -3.0, 6.0, 1.5, "dB", MacroRole::Tone, "High-Frequency Bias Saturation parameter"))
            .with_param(DspParamSchema::knob("flux_compression_db", "Head Hysteresis Compression", 0.0, 6.0, 1.8, "dB", MacroRole::Tone, "Head Hysteresis Compression parameter"))
        );
        descriptors.insert("DialogGatingNode".to_string(), DspNodeDescriptor::new("DialogGatingNode", "ITU-R BS.1770 Dialogue-Aware Gating Loudness Engine", DspNodeCategory::DynamicsMaster, "ITU-R BS.1770 Dialogue-Aware Gating Loudness Engine")
            .with_param(DspParamSchema::knob("speech_presence_threshold_db", "Speech Presence Gate", -40.0, -10.0, -24.0, "dB", MacroRole::Tone, "Speech Presence Gate parameter"))
            .with_param(DspParamSchema::knob("gating_window_ms", "Voice Energy Gating Window", 100.0, 1000.0, 400.0, "ms", MacroRole::Tone, "Voice Energy Gating Window parameter"))
            .with_param(DspParamSchema::knob("ambient_noise_attenuation_db", "Background Attenuation", -30.0, 0.0, -12.0, "dB", MacroRole::Tone, "Background Attenuation parameter"))
            .with_param(DspParamSchema::knob("voice_clarity_lift_db", "Formant Intelligibility Lift", 0.0, 6.0, 2.0, "dB", MacroRole::Tone, "Formant Intelligibility Lift parameter"))
        );
        descriptors.insert("SonarHydrophoneNode".to_string(), DspNodeDescriptor::new("SonarHydrophoneNode", "Underwater Acoustic Hydrophone & Thermal Layer Simulation", DspNodeCategory::DynamicsMaster, "Underwater Acoustic Hydrophone & Thermal Layer Simulation")
            .with_param(DspParamSchema::knob("hydrophone_depth_m", "Sensor Water Depth", 1.0, 1000.0, 75.0, "m", MacroRole::Tone, "Sensor Water Depth parameter"))
            .with_param(DspParamSchema::knob("water_salinity_ppt", "Ocean Salinity", 0.0, 45.0, 35.0, "ppt", MacroRole::Tone, "Ocean Salinity parameter"))
            .with_param(DspParamSchema::knob("sound_channel_axis_depth_m", "SOFAR Channel Axis Depth", 100.0, 1500.0, 800.0, "m", MacroRole::Tone, "SOFAR Channel Axis Depth parameter"))
            .with_param(DspParamSchema::knob("cavitation_noise_level", "Propeller Cavitation Ambient", 0.0, 1.0, 0.2, "%", MacroRole::Tone, "Propeller Cavitation Ambient parameter"))
        );
        descriptors.insert("BciEegDecoderNode".to_string(), DspNodeDescriptor::new("BciEegDecoderNode", "10-20 EEG Brainwave Band Power & Alpha/Theta Decoder", DspNodeCategory::NeuralAi, "10-20 EEG Brainwave Band Power & Alpha/Theta Decoder")
            .with_param(DspParamSchema::knob("sampling_rate_hz", "EEG Electrode Sample Rate", 128.0, 1024.0, 256.0, "Hz", MacroRole::Tone, "EEG Electrode Sample Rate parameter"))
            .with_param(DspParamSchema::knob("alpha_band_gain", "Alpha Power Band (8-12Hz)", 0.0, 2.0, 1.0, "x", MacroRole::Tone, "Alpha Power Band (8-12Hz) parameter"))
            .with_param(DspParamSchema::knob("theta_band_gain", "Theta Power Band (4-8Hz)", 0.0, 2.0, 0.8, "x", MacroRole::Tone, "Theta Power Band (4-8Hz) parameter"))
            .with_param(DspParamSchema::toggle("artifact_rejection_filter", "Ocular Blink Artifact Filter", true, "Ocular Blink Artifact Filter toggle switch"))
        );
        descriptors.insert("NeuroAffectiveEmotionalStateAnalyzer".to_string(), DspNodeDescriptor::new("NeuroAffectiveEmotionalStateAnalyzer", "Russell Circumplex Valence & Arousal Emotion Analyzer", DspNodeCategory::NeuralAi, "Russell Circumplex Valence & Arousal Emotion Analyzer")
            .with_param(DspParamSchema::knob("valence_score", "Musical Valence (Pleasantness)", -1.0, 1.0, 0.55, "score", MacroRole::Tone, "Musical Valence (Pleasantness) parameter"))
            .with_param(DspParamSchema::knob("arousal_intensity", "Musical Arousal (Energy)", 0.0, 1.0, 0.65, "score", MacroRole::Tone, "Musical Arousal (Energy) parameter"))
            .with_param(DspParamSchema::knob("emotional_smoothing_window", "Affective Slew Smoothing", 0.1, 10.0, 2.0, "s", MacroRole::Tone, "Affective Slew Smoothing parameter"))
            .with_param(DspParamSchema::knob("harmony_feedback_gain", "Emotional Harmony Influence", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Emotional Harmony Influence parameter"))
        );
        descriptors.insert("NeuralImpulseResponseSynthesizer".to_string(), DspNodeDescriptor::new("NeuralImpulseResponseSynthesizer", "Latent Space Acoustic Impulse Response Generator", DspNodeCategory::NeuralAi, "Latent Space Acoustic Impulse Response Generator")
            .with_param(DspParamSchema::knob("latent_room_dimension_x", "Latent Room Geometry X", -3.0, 3.0, 0.5, "", MacroRole::Tone, "Latent Room Geometry X parameter"))
            .with_param(DspParamSchema::knob("latent_absorption_y", "Latent Wall Material Y", -3.0, 3.0, -0.2, "", MacroRole::Tone, "Latent Wall Material Y parameter"))
            .with_param(DspParamSchema::knob("early_reflection_decay", "Early Diffusion Slew", 0.1, 5.0, 1.5, "s", MacroRole::Tone, "Early Diffusion Slew parameter"))
            .with_param(DspParamSchema::knob("synthesis_quality", "Neural Inference Latent Depth", 1.0, 4.0, 3.0, "steps", MacroRole::Character, "Neural Inference Latent Depth integer control"))
        );
        descriptors.insert("SubcorticalBrainstemPitchTracker".to_string(), DspNodeDescriptor::new("SubcorticalBrainstemPitchTracker", "Auditory Nerve Frequency-Following Response Pitch Tracker", DspNodeCategory::NeuralAi, "Auditory Nerve Frequency-Following Response Pitch Tracker")
            .with_param(DspParamSchema::knob("ffr_sensitivity", "Brainstem FFR Sensitivity", 0.1, 2.0, 1.0, "x", MacroRole::Tone, "Brainstem FFR Sensitivity parameter"))
            .with_param(DspParamSchema::knob("cochlear_filterbank_channels", "Cochlear Gammatone Channels", 16.0, 128.0, 64.0, "ch", MacroRole::Character, "Cochlear Gammatone Channels integer control"))
            .with_param(DspParamSchema::log_knob("fundamental_f0_estimate_hz", "Detected Pitch F0", 30.0, 2000.0, 220.0, "Hz", MacroRole::Tone, "Detected Pitch F0 logarithmic parameter"))
            .with_param(DspParamSchema::knob("tracking_slew_ms", "Pitch Tracking Response", 1.0, 100.0, 15.0, "ms", MacroRole::Tone, "Pitch Tracking Response parameter"))
        );
        descriptors.insert("MentalImageryPatternClassifier".to_string(), DspNodeDescriptor::new("MentalImageryPatternClassifier", "Motor/Auditory Imagery Neural Intention Classifier", DspNodeCategory::NeuralAi, "Motor/Auditory Imagery Neural Intention Classifier")
            .with_param(DspParamSchema::knob("classifier_confidence_threshold", "Trigger Confidence Floor", 0.5, 0.99, 0.85, "%", MacroRole::Tone, "Trigger Confidence Floor parameter"))
            .with_param(DspParamSchema::knob("pattern_bank_index", "Intention Pattern Slot", 0.0, 15.0, 0.0, "slot", MacroRole::Character, "Intention Pattern Slot integer control"))
            .with_param(DspParamSchema::knob("trigger_event_latency_ms", "Pattern Recognition Latency", 5.0, 200.0, 35.0, "ms", MacroRole::Tone, "Pattern Recognition Latency parameter"))
            .with_param(DspParamSchema::knob("adaptation_rate", "Online Neural Plasticity Slew", 0.0, 1.0, 0.1, "%", MacroRole::Tone, "Online Neural Plasticity Slew parameter"))
        );
        descriptors.insert("BiometricHrvTempoSync".to_string(), DspNodeDescriptor::new("BiometricHrvTempoSync", "Photoplethysmography Heart Rate Variability Tempo Sync", DspNodeCategory::NeuralAi, "Photoplethysmography Heart Rate Variability Tempo Sync")
            .with_param(DspParamSchema::knob("target_heart_coherence", "Target HRV Coherence", 0.0, 1.0, 0.75, "%", MacroRole::Tone, "Target HRV Coherence parameter"))
            .with_param(DspParamSchema::knob("bpm_sync_multiplier", "BPM Metric Ratio", 0.5, 4.0, 1.0, "x", MacroRole::Tone, "BPM Metric Ratio parameter"))
            .with_param(DspParamSchema::knob("hrv_smoothing_interval_s", "Heart Rate Rolling Average", 5.0, 60.0, 20.0, "s", MacroRole::Tone, "Heart Rate Rolling Average parameter"))
            .with_param(DspParamSchema::knob("rhythm_entrainment_depth", "Tempo Entrainment Depth", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Tempo Entrainment Depth parameter"))
        );
        descriptors.insert("NeuroCognitiveFatigueDetector".to_string(), DspNodeDescriptor::new("NeuroCognitiveFatigueDetector", "Listening Fatigue & Auditory Habituation Detector", DspNodeCategory::NeuralAi, "Listening Fatigue & Auditory Habituation Detector")
            .with_param(DspParamSchema::knob("listening_duration_minutes", "Session Exposure Time", 0.0, 480.0, 65.0, "min", MacroRole::Tone, "Session Exposure Time parameter"))
            .with_param(DspParamSchema::knob("spectral_brightness_fatigue_index", "Harshness Fatigue Metric", 0.0, 1.0, 0.4, "%", MacroRole::Tone, "Harshness Fatigue Metric parameter"))
            .with_param(DspParamSchema::knob("auto_soft_filter_db", "Fatigue Relief Soft High Cut", -6.0, 0.0, 0.0, "dB", MacroRole::Tone, "Fatigue Relief Soft High Cut parameter"))
            .with_param(DspParamSchema::toggle("habituation_warning", "Break Reminder Notification", true, "Break Reminder Notification toggle switch"))
        );
        descriptors.insert("NeuroAestheticHarmonyScorer".to_string(), DspNodeDescriptor::new("NeuroAestheticHarmonyScorer", "Computational Neuro-Aesthetic Musical Harmony Scorer", DspNodeCategory::NeuralAi, "Computational Neuro-Aesthetic Musical Harmony Scorer")
            .with_param(DspParamSchema::knob("harmony_concordance_weight", "Concordance Sensation Bias", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Concordance Sensation Bias parameter"))
            .with_param(DspParamSchema::knob("voice_leading_fluency", "Voice Leading Smoothness", 0.0, 1.0, 0.7, "%", MacroRole::Tone, "Voice Leading Smoothness parameter"))
            .with_param(DspParamSchema::knob("tension_resolution_score", "Tension Cycle Resolution", 0.0, 1.0, 0.85, "%", MacroRole::Tone, "Tension Cycle Resolution parameter"))
            .with_param(DspParamSchema::knob("scale_fit_percent", "Tonal Scale Alignment", 0.0, 100.0, 94.0, "%", MacroRole::Tone, "Tonal Scale Alignment parameter"))
        );
        descriptors.insert("SubsensoryTactileHapticTransducer".to_string(), DspNodeDescriptor::new("SubsensoryTactileHapticTransducer", "Vibrotactile Sub-Bass Haptic Transducer Driver", DspNodeCategory::NeuralAi, "Vibrotactile Sub-Bass Haptic Transducer Driver")
            .with_param(DspParamSchema::log_knob("haptic_frequency_hz", "Transducer Resonance Pitch", 15.0, 120.0, 40.0, "Hz", MacroRole::Tone, "Transducer Resonance Pitch logarithmic parameter"))
            .with_param(DspParamSchema::knob("transducer_intensity", "Haptic Shaker Force", 0.0, 1.0, 0.75, "%", MacroRole::Tone, "Haptic Shaker Force parameter"))
            .with_param(DspParamSchema::choice("subharmonic_synthesis_oct", "Sub-Harmonic Generation", &["None (Direct)", "-1 Octave Sub", "-2 Octave Infrasound"], 1, "Sub-Harmonic Generation mode selector"))
            .with_param(DspParamSchema::knob("rumble_envelope_decay", "Haptic Rumble Decay", 0.05, 2.0, 0.4, "s", MacroRole::Tone, "Haptic Rumble Decay parameter"))
        );
        descriptors.insert("NeuralChoirFormantNode".to_string(), DspNodeDescriptor::new("NeuralChoirFormantNode", "Multi-Voice Neural Formant Choir & Vowel Morph Engine", DspNodeCategory::NeuralAi, "Multi-Voice Neural Formant Choir & Vowel Morph Engine")
            .with_param(DspParamSchema::knob("voice_count", "Ensemble Choir Voices", 4.0, 32.0, 16.0, "voices", MacroRole::Character, "Ensemble Choir Voices integer control"))
            .with_param(DspParamSchema::knob("vowel_morph_coordinate", "Vowel Space Morph (A-E-I-O-U)", 0.0, 4.0, 1.5, "vowel", MacroRole::Tone, "Vowel Space Morph (A-E-I-O-U) parameter"))
            .with_param(DspParamSchema::knob("formant_spread_cents", "Formant Detuning Width", 0.0, 50.0, 15.0, "cents", MacroRole::Tone, "Formant Detuning Width parameter"))
            .with_param(DspParamSchema::knob("choir_detune_cents", "Choral Micro-Pitch Detune", 0.0, 40.0, 8.0, "cents", MacroRole::Tone, "Choral Micro-Pitch Detune parameter"))
        );
        descriptors.insert("NeuralDereverbNode".to_string(), DspNodeDescriptor::new("NeuralDereverbNode", "Deep Learning Direct-to-Reverberant Ratio Blind De-Reverb", DspNodeCategory::NeuralAi, "Deep Learning Direct-to-Reverberant Ratio Blind De-Reverb")
            .with_param(DspParamSchema::knob("dereverb_intensity", "De-Reverberation Depth", 0.0, 100.0, 65.0, "%", MacroRole::Tone, "De-Reverberation Depth parameter"))
            .with_param(DspParamSchema::knob("room_size_estimate_m", "Estimated Chamber Size", 2.0, 50.0, 12.0, "m", MacroRole::Tone, "Estimated Chamber Size parameter"))
            .with_param(DspParamSchema::knob("early_reflection_suppress_db", "Early Reflection Attenuation", -24.0, 0.0, -10.0, "dB", MacroRole::Tone, "Early Reflection Attenuation parameter"))
            .with_param(DspParamSchema::knob("tail_dry_boost_db", "Direct Signal Presence Boost", 0.0, 12.0, 2.5, "dB", MacroRole::Tone, "Direct Signal Presence Boost parameter"))
        );
        descriptors.insert("NeuralInpaintNode".to_string(), DspNodeDescriptor::new("NeuralInpaintNode", "Neural Audio Inpainting & Dropout Reconstruction Engine", DspNodeCategory::NeuralAi, "Neural Audio Inpainting & Dropout Reconstruction Engine")
            .with_param(DspParamSchema::knob("dropout_threshold_ms", "Dropout Mask Detection Window", 1.0, 100.0, 20.0, "ms", MacroRole::Tone, "Dropout Mask Detection Window parameter"))
            .with_param(DspParamSchema::knob("reconstruction_depth", "AI Inpainting Strength", 0.0, 1.0, 0.85, "%", MacroRole::Tone, "AI Inpainting Strength parameter"))
            .with_param(DspParamSchema::knob("context_window_ms", "Surrounding Context Frame", 50.0, 1000.0, 250.0, "ms", MacroRole::Character, "Surrounding Context Frame integer control"))
            .with_param(DspParamSchema::toggle("spectral_continuity_protect", "Preserve Spectral Phase", true, "Preserve Spectral Phase toggle switch"))
        );
        descriptors.insert("NeuralPhonemeNode".to_string(), DspNodeDescriptor::new("NeuralPhonemeNode", "International Phonetic Alphabet Differentiable Phoneme Synth", DspNodeCategory::NeuralAi, "International Phonetic Alphabet Differentiable Phoneme Synth")
            .with_param(DspParamSchema::knob("ipa_symbol_index", "IPA Phoneme Symbol Index", 0.0, 64.0, 12.0, "ipa", MacroRole::Character, "IPA Phoneme Symbol Index integer control"))
            .with_param(DspParamSchema::knob("nasalization_index", "Velum Nasal Coupling", 0.0, 1.0, 0.2, "%", MacroRole::Tone, "Velum Nasal Coupling parameter"))
            .with_param(DspParamSchema::knob("voicing_glottal_mix", "Voiced Glottal Mix", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Voiced Glottal Mix parameter"))
            .with_param(DspParamSchema::knob("tongue_arch_position", "Tongue Constriction Place", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Tongue Constriction Place parameter"))
        );
        descriptors.insert("NeuralRadianceNode".to_string(), DspNodeDescriptor::new("NeuralRadianceNode", "Neural Acoustic Radiance Field (NeRF) 3D Room Acoustic Field", DspNodeCategory::NeuralAi, "Neural Acoustic Radiance Field (NeRF) 3D Room Acoustic Field")
            .with_param(DspParamSchema::knob("listener_pos_xyz", "Virtual Listener X Position", -10.0, 10.0, 0.0, "m", MacroRole::Tone, "Virtual Listener X Position parameter"))
            .with_param(DspParamSchema::knob("emitter_pos_xyz", "Virtual Emitter Y Position", -10.0, 10.0, 2.5, "m", MacroRole::Tone, "Virtual Emitter Y Position parameter"))
            .with_param(DspParamSchema::knob("nerf_rendering_samples", "Ray Sample Marching Steps", 32.0, 256.0, 128.0, "rays", MacroRole::Character, "Ray Sample Marching Steps integer control"))
            .with_param(DspParamSchema::knob("spatial_interpolation_slew", "Acoustic Field Slew Rate", 0.01, 1.0, 0.15, "s", MacroRole::Tone, "Acoustic Field Slew Rate parameter"))
        );
        descriptors.insert("NeuralSpeechToSingingNode".to_string(), DspNodeDescriptor::new("NeuralSpeechToSingingNode", "Speech-to-Singing AI Prosody & Intonation Transformer", DspNodeCategory::NeuralAi, "Speech-to-Singing AI Prosody & Intonation Transformer")
            .with_param(DspParamSchema::toggle("target_pitch_quantize", "Snap Speech to Scale Notes", true, "Snap Speech to Scale Notes toggle switch"))
            .with_param(DspParamSchema::knob("vibrato_infusion_depth", "Singing Vibrato Infusion", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Singing Vibrato Infusion parameter"))
            .with_param(DspParamSchema::knob("note_hold_extension_ms", "Vowel Duration Extension", 50.0, 1000.0, 350.0, "ms", MacroRole::Tone, "Vowel Duration Extension parameter"))
            .with_param(DspParamSchema::choice("singing_style_preset", "AI Singing Style", &["Bel Canto Opera", "Pop Vibrato Smooth", "R&B Micro-Riff", "Choir Polyphony"], 1, "AI Singing Style mode selector"))
        );
        descriptors.insert("NeuralTimbreMorphNode".to_string(), DspNodeDescriptor::new("NeuralTimbreMorphNode", "Non-Linear Latent Timbre Vector Interpolator", DspNodeCategory::NeuralAi, "Non-Linear Latent Timbre Vector Interpolator")
            .with_param(DspParamSchema::knob("timbre_vector_x", "Latent Timbre Axis X (Warm/Bright)", -3.0, 3.0, 0.0, "", MacroRole::Tone, "Latent Timbre Axis X (Warm/Bright) parameter"))
            .with_param(DspParamSchema::knob("timbre_vector_y", "Latent Timbre Axis Y (Wood/Metal)", -3.0, 3.0, 0.0, "", MacroRole::Tone, "Latent Timbre Axis Y (Wood/Metal) parameter"))
            .with_param(DspParamSchema::knob("timbre_vector_z", "Latent Timbre Axis Z (Acoustic/Synth)", -3.0, 3.0, 0.0, "", MacroRole::Tone, "Latent Timbre Axis Z (Acoustic/Synth) parameter"))
            .with_param(DspParamSchema::knob("morph_trajectory_speed_hz", "LFO Timbre Orbit Speed", 0.05, 10.0, 0.5, "Hz", MacroRole::Tone, "LFO Timbre Orbit Speed parameter"))
        );
        descriptors.insert("NeuralTimbreNode".to_string(), DspNodeDescriptor::new("NeuralTimbreNode", "Latent Timbre Space Feature Extractor & Profiler", DspNodeCategory::NeuralAi, "Latent Timbre Space Feature Extractor & Profiler")
            .with_param(DspParamSchema::knob("spectral_centroid_weight", "Spectral Brightness Weight", 0.0, 1.0, 0.7, "%", MacroRole::Tone, "Spectral Brightness Weight parameter"))
            .with_param(DspParamSchema::knob("flux_roughness_weight", "Spectral Roughness Weight", 0.0, 1.0, 0.4, "%", MacroRole::Tone, "Spectral Roughness Weight parameter"))
            .with_param(DspParamSchema::knob("harmonic_entropy_weight", "Harmonic Order vs Noise", 0.0, 1.0, 0.85, "%", MacroRole::Tone, "Harmonic Order vs Noise parameter"))
            .with_param(DspParamSchema::toggle("timbre_snapshot_trigger", "Capture Timbre Vector", false, "Capture Timbre Vector toggle switch"))
        );
        descriptors.insert("NeuralVocalStylizerNode".to_string(), DspNodeDescriptor::new("NeuralVocalStylizerNode", "Vocal Timbre Style Transfer & Character Transformer", DspNodeCategory::NeuralAi, "Vocal Timbre Style Transfer & Character Transformer")
            .with_param(DspParamSchema::knob("source_vocal_id", "Source Vocal Track ID", 0.0, 32.0, 1.0, "trk", MacroRole::Character, "Source Vocal Track ID integer control"))
            .with_param(DspParamSchema::choice("target_style_preset", "AI Timbre Target Profile", &["Vintage Ribbon Mic Warmth", "Modern Hyper-Pop Crisp", "Analog Tube Intimate", "Classic 80s Broadcast"], 0, "AI Timbre Target Profile mode selector"))
            .with_param(DspParamSchema::knob("formant_character_scale", "Formant Character Intensity", 0.5, 2.0, 1.0, "x", MacroRole::Tone, "Formant Character Intensity parameter"))
            .with_param(DspParamSchema::knob("style_transfer_blend", "Style Transfer Dry/Wet Blend", 0.0, 1.0, 0.75, "%", MacroRole::Tone, "Style Transfer Dry/Wet Blend parameter"))
        );
        descriptors.insert("NeuralVocoderMorphNode".to_string(), DspNodeDescriptor::new("NeuralVocoderMorphNode", "Neural HiFi-GAN / BigVGAN Vocoder Latent Morpher", DspNodeCategory::NeuralAi, "Neural HiFi-GAN / BigVGAN Vocoder Latent Morpher")
            .with_param(DspParamSchema::choice("vocoder_architecture", "Neural Vocoder Backbone", &["BigVGAN High-Fidelity", "HiFi-GAN V1", "Vocos Ultra-Fast", "WaveGlow Multi-Scale"], 0, "Neural Vocoder Backbone mode selector"))
            .with_param(DspParamSchema::knob("latent_dimension_scale", "Latent Manifold Scale", 0.1, 3.0, 1.0, "x", MacroRole::Tone, "Latent Manifold Scale parameter"))
            .with_param(DspParamSchema::knob("periodic_component_mix", "Harmonic Periodic Component", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Harmonic Periodic Component parameter"))
            .with_param(DspParamSchema::knob("aperiodic_noise_ratio", "Aperiodic Breath Noise Level", 0.0, 1.0, 0.2, "%", MacroRole::Tone, "Aperiodic Breath Noise Level parameter"))
        );
        descriptors.insert("NeuralQuantumAnnealerTopologicalSort".to_string(), DspNodeDescriptor::new("NeuralQuantumAnnealerTopologicalSort", "Simulated Quantum Annealing Audio Graph Optimizer", DspNodeCategory::NeuralAi, "Simulated Quantum Annealing Audio Graph Optimizer")
            .with_param(DspParamSchema::knob("annealing_temperature", "Annealing Temperature (T)", 0.01, 10.0, 1.0, "T", MacroRole::Tone, "Annealing Temperature (T) parameter"))
            .with_param(DspParamSchema::knob("transverse_field_gamma", "Transverse Quantum Field", 0.0, 5.0, 1.5, "gamma", MacroRole::Tone, "Transverse Quantum Field parameter"))
            .with_param(DspParamSchema::knob("graph_latency_minimize_weight", "Latency Minimization Weight", 0.0, 1.0, 0.9, "%", MacroRole::Tone, "Latency Minimization Weight parameter"))
            .with_param(DspParamSchema::toggle("optimization_step_trigger", "Trigger Topological Sort", false, "Trigger Topological Sort toggle switch"))
        );
        descriptors.insert("GainNode".to_string(), DspNodeDescriptor::new("GainNode", "Precision Unity Gain & Polarity Inverter", DspNodeCategory::Utility, "Precision Unity Gain & Polarity Inverter")
            .with_param(DspParamSchema::knob("gain_db", "Gain Level", -60.0, 18.0, 0.0, "dB", MacroRole::Tone, "Gain Level parameter"))
            .with_param(DspParamSchema::toggle("invert_phase", "Invert Phase (180 deg)", false, "Invert Phase (180 deg) toggle switch"))
            .with_param(DspParamSchema::toggle("mute", "Channel Mute", false, "Channel Mute toggle switch"))
            .with_param(DspParamSchema::knob("pan", "Stereo Balance Pan", -1.0, 1.0, 0.0, "pan", MacroRole::Tone, "Stereo Balance Pan parameter"))
        );
        descriptors.insert("ChromaticTunerNode".to_string(), DspNodeDescriptor::new("ChromaticTunerNode", "Strobe Chromatic Instrument Pitch Tuner", DspNodeCategory::Utility, "Strobe Chromatic Instrument Pitch Tuner")
            .with_param(DspParamSchema::knob("reference_pitch_hz", "Concert Pitch A4", 415.0, 466.0, 440.0, "Hz", MacroRole::Tone, "Concert Pitch A4 parameter"))
            .with_param(DspParamSchema::knob("cents_tolerance", "Strobe Sensitivity", 1.0, 20.0, 5.0, "cents", MacroRole::Tone, "Strobe Sensitivity parameter"))
            .with_param(DspParamSchema::toggle("mute_on_tune", "Mute Audio While Tuning", false, "Mute Audio While Tuning toggle switch"))
        );
        descriptors.insert("SampleSlicerNode".to_string(), DspNodeDescriptor::new("SampleSlicerNode", "Transient Beat Slicer & Loop Fragmenter", DspNodeCategory::SamplerSlicer, "Transient Beat Slicer & Loop Fragmenter")
            .with_param(DspParamSchema::knob("threshold_db", "Transient Slicer Sensitivity", -60.0, 0.0, -24.0, "dB", MacroRole::Tone, "Transient Slicer Sensitivity parameter"))
            .with_param(DspParamSchema::knob("min_slice_ms", "Minimum Slice Duration", 10.0, 500.0, 50.0, "ms", MacroRole::Tone, "Minimum Slice Duration parameter"))
            .with_param(DspParamSchema::toggle("bpm_sync", "Snap Slices to Beat Grid", true, "Snap Slices to Beat Grid toggle switch"))
            .with_param(DspParamSchema::knob("transient_sensitivity", "Onset Detection Threshold", 0.0, 1.0, 0.7, "%", MacroRole::Tone, "Onset Detection Threshold parameter"))
        );
        descriptors.insert("LoopSlicerNode".to_string(), DspNodeDescriptor::new("LoopSlicerNode", "Beat-Synchronized Grid Loop Slicer", DspNodeCategory::SamplerSlicer, "Beat-Synchronized Grid Loop Slicer")
            .with_param(DspParamSchema::choice("grid_division", "Beat Division", &["1/4 Beat", "1/8 Beat", "1/16 Beat", "1/32 Beat", "Triplet 1/8"], 2, "Beat Division mode selector"))
            .with_param(DspParamSchema::knob("crossfade_ms", "Slice Edge Crossfade", 0.0, 50.0, 5.0, "ms", MacroRole::Tone, "Slice Edge Crossfade parameter"))
            .with_param(DspParamSchema::toggle("pitch_match", "Preserve Original Pitch", true, "Preserve Original Pitch toggle switch"))
            .with_param(DspParamSchema::toggle("reverse_slices", "Reverse Alternate Slices", false, "Reverse Alternate Slices toggle switch"))
        );
        descriptors.insert("HardwareCvNode".to_string(), DspNodeDescriptor::new("HardwareCvNode", "Eurorack Modular Control Voltage & Gate Interface", DspNodeCategory::Utility, "Eurorack Modular Control Voltage & Gate Interface")
            .with_param(DspParamSchema::knob("cv_channel", "Hardware CV Channel", 1.0, 16.0, 1.0, "ch", MacroRole::Character, "Hardware CV Channel integer control"))
            .with_param(DspParamSchema::choice("voltage_range", "Output Voltage Range", &["0 to 10V (Unipolar)", "-5 to +5V (Bipolar)", "-10 to +10V (Modular Extended)"], 1, "Output Voltage Range mode selector"))
            .with_param(DspParamSchema::knob("gate_high_volts", "Gate On Threshold", 1.0, 10.0, 5.0, "V", MacroRole::Tone, "Gate On Threshold parameter"))
            .with_param(DspParamSchema::knob("slew_time_ms", "Portamento Slew Time", 0.0, 100.0, 2.0, "ms", MacroRole::Tone, "Portamento Slew Time parameter"))
        );
        descriptors.insert("ClapPluginHostNode".to_string(), DspNodeDescriptor::new("ClapPluginHostNode", "CLAP/VST3 Audio Plugin Host Adapter", DspNodeCategory::Utility, "CLAP/VST3 Audio Plugin Host Adapter")
            .with_param(DspParamSchema::knob("plugin_slot", "Hosted Plugin Slot", 0.0, 15.0, 0.0, "slot", MacroRole::Character, "Hosted Plugin Slot integer control"))
            .with_param(DspParamSchema::toggle("bypass", "Plugin Bypass State", false, "Plugin Bypass State toggle switch"))
            .with_param(DspParamSchema::knob("dry_wet", "Plugin Dry / Wet Mix", 0.0, 1.0, 1.0, "%", MacroRole::Tone, "Plugin Dry / Wet Mix parameter"))
            .with_param(DspParamSchema::knob("latency_comp_ms", "PDC Latency Offset", 0.0, 50.0, 0.0, "ms", MacroRole::Tone, "PDC Latency Offset parameter"))
        );
        descriptors.insert("Vst3HostNode".to_string(), DspNodeDescriptor::new("Vst3HostNode", "VST3 Plugin Host with Embedded GUI & Automation", DspNodeCategory::Utility, "VST3 Plugin Host with Embedded GUI & Automation")
            .with_param(DspParamSchema::knob("plugin_slot_index", "VST3 Plugin Slot", 0.0, 15.0, 0.0, "slot", MacroRole::Character, "VST3 Plugin Slot integer control"))
            .with_param(DspParamSchema::toggle("bypass_plugin", "Host Bypass Plugin", false, "Host Bypass Plugin toggle switch"))
            .with_param(DspParamSchema::knob("dry_wet_mix", "Plugin Wet / Dry Blend", 0.0, 1.0, 1.0, "%", MacroRole::Tone, "Plugin Wet / Dry Blend parameter"))
            .with_param(DspParamSchema::knob("latency_compensation_ms", "Plugin Delay Compensation", 0.0, 100.0, 0.0, "ms", MacroRole::Tone, "Plugin Delay Compensation parameter"))
        );
        descriptors.insert("PushControllerDriverNode".to_string(), DspNodeDescriptor::new("PushControllerDriverNode", "Ableton Push 2/3 High-Speed Display & RGB Pad Driver", DspNodeCategory::Utility, "Ableton Push 2/3 High-Speed Display & RGB Pad Driver")
            .with_param(DspParamSchema::knob("display_frame_rate_fps", "LCD Frame Refresh Rate", 30.0, 120.0, 60.0, "fps", MacroRole::Character, "LCD Frame Refresh Rate integer control"))
            .with_param(DspParamSchema::knob("rgb_brightness_percent", "Pad Backlight Brightness", 10.0, 100.0, 85.0, "%", MacroRole::Tone, "Pad Backlight Brightness parameter"))
            .with_param(DspParamSchema::choice("touch_strip_pitch_bend_mode", "Touch Strip Action", &["Pitch Bend Centered", "Modulation Wheel", "Expression CC11"], 0, "Touch Strip Action mode selector"))
            .with_param(DspParamSchema::knob("pad_sensitivity_curve", "Velocity Curve Exponent", 0.5, 3.0, 1.2, "exp", MacroRole::Tone, "Velocity Curve Exponent parameter"))
        );
        descriptors.insert("LaunchpadDriverNode".to_string(), DspNodeDescriptor::new("LaunchpadDriverNode", "Novation Launchpad Pro RGB Session & Step Sequencer Driver", DspNodeCategory::Utility, "Novation Launchpad Pro RGB Session & Step Sequencer Driver")
            .with_param(DspParamSchema::toggle("session_mode_active", "Session Clip Matrix Mode", true, "Session Clip Matrix Mode toggle switch"))
            .with_param(DspParamSchema::knob("velocity_response_curve", "Aftertouch Sensitivity", 0.1, 2.0, 1.0, "x", MacroRole::Tone, "Aftertouch Sensitivity parameter"))
            .with_param(DspParamSchema::choice("led_color_palette", "RGB Color Palette", &["Novation Standard Palette", "Vibrant DAW Matrix", "Monochrome Studio"], 0, "RGB Color Palette mode selector"))
            .with_param(DspParamSchema::knob("midi_channel_out", "Controller Output Channel", 1.0, 16.0, 1.0, "ch", MacroRole::Character, "Controller Output Channel integer control"))
        );
        descriptors.insert("KompleteKontrolDriverNode".to_string(), DspNodeDescriptor::new("KompleteKontrolDriverNode", "Native Instruments NKS Light Guide & Encoder Driver", DspNodeCategory::Utility, "Native Instruments NKS Light Guide & Encoder Driver")
            .with_param(DspParamSchema::toggle("light_guide_key_split", "Light Guide Key Split Colors", true, "Light Guide Key Split Colors toggle switch"))
            .with_param(DspParamSchema::toggle("scale_guide_illumination", "Light Guide Scale Illumination", true, "Light Guide Scale Illumination toggle switch"))
            .with_param(DspParamSchema::choice("touch_strip_modulation_mode", "Modulation Strip Physics", &["Standard Free", "Spring Return", "Gravity Bounce"], 1, "Modulation Strip Physics mode selector"))
            .with_param(DspParamSchema::knob("encoder_sensitivity", "Rotary Encoder Slew Rate", 0.5, 3.0, 1.0, "x", MacroRole::Tone, "Rotary Encoder Slew Rate parameter"))
        );
        descriptors.insert("McuHardwareDriverNode".to_string(), DspNodeDescriptor::new("McuHardwareDriverNode", "Mackie Control Universal 100mm Motorized Fader Surface", DspNodeCategory::Utility, "Mackie Control Universal 100mm Motorized Fader Surface")
            .with_param(DspParamSchema::toggle("motor_fader_touch_sense", "Touch-Sensitive Fader Motor", true, "Touch-Sensitive Fader Motor toggle switch"))
            .with_param(DspParamSchema::choice("v_pot_led_ring_mode", "V-Pot LED Ring Mode", &["Single Dot Center", "Boost/Cut Bar", "Wrap Around Fill"], 1, "V-Pot LED Ring Mode mode selector"))
            .with_param(DspParamSchema::knob("lcd_scribble_strip_contrast", "LCD Scribble Strip Contrast", 0.0, 100.0, 75.0, "%", MacroRole::Tone, "LCD Scribble Strip Contrast parameter"))
            .with_param(DspParamSchema::knob("fader_bank_offset", "Fader Bank Track Offset", 0.0, 64.0, 0.0, "tracks", MacroRole::Character, "Fader Bank Track Offset integer control"))
        );
        descriptors.insert("OscControlMapperNode".to_string(), DspNodeDescriptor::new("OscControlMapperNode", "Bidirectional Open Sound Control (OSC) Network Gateway", DspNodeCategory::Utility, "Bidirectional Open Sound Control (OSC) Network Gateway")
            .with_param(DspParamSchema::knob("osc_udp_port_rx", "OSC Receiver UDP Port", 1024.0, 65535.0, 9000.0, "port", MacroRole::Character, "OSC Receiver UDP Port integer control"))
            .with_param(DspParamSchema::knob("osc_udp_port_tx", "OSC Transmitter UDP Port", 1024.0, 65535.0, 9001.0, "port", MacroRole::Character, "OSC Transmitter UDP Port integer control"))
            .with_param(DspParamSchema::choice("ip_target_address", "Destination IP Host", &["127.0.0.1 (Localhost)", "192.168.1.100 (LAN)", "Broadcast (255.255.255.255)"], 0, "Destination IP Host mode selector"))
            .with_param(DspParamSchema::knob("message_rate_limit_hz", "Max OSC Message Frequency", 10.0, 500.0, 120.0, "Hz", MacroRole::Tone, "Max OSC Message Frequency parameter"))
        );
        descriptors.insert("MultiTrackAudioRouterNode".to_string(), DspNodeDescriptor::new("MultiTrackAudioRouterNode", "64x64 Low-Latency Cross-Point Matrix Audio Router", DspNodeCategory::Utility, "64x64 Low-Latency Cross-Point Matrix Audio Router")
            .with_param(DspParamSchema::knob("source_channel_index", "Input Matrix Channel", 0.0, 63.0, 0.0, "in", MacroRole::Character, "Input Matrix Channel integer control"))
            .with_param(DspParamSchema::knob("destination_bus_index", "Output Destination Bus", 0.0, 63.0, 0.0, "out", MacroRole::Character, "Output Destination Bus integer control"))
            .with_param(DspParamSchema::knob("cross_point_gain_db", "Cross-Point Gain", -60.0, 12.0, 0.0, "dB", MacroRole::Tone, "Cross-Point Gain parameter"))
            .with_param(DspParamSchema::toggle("phase_invert_route", "Invert Route Polarity", false, "Invert Route Polarity toggle switch"))
        );
        descriptors.insert("WasmDspRuntimeNode".to_string(), DspNodeDescriptor::new("WasmDspRuntimeNode", "Sandboxed WebAssembly Wasm DSP Custom Kernel Host", DspNodeCategory::Utility, "Sandboxed WebAssembly Wasm DSP Custom Kernel Host")
            .with_param(DspParamSchema::knob("wasm_kernel_memory_mb", "Sandbox Memory Allocation", 1.0, 64.0, 8.0, "MB", MacroRole::Character, "Sandbox Memory Allocation integer control"))
            .with_param(DspParamSchema::knob("instruction_budget_per_block", "Block Instruction Budget", 1000.0, 100000.0, 15000.0, "ops", MacroRole::Character, "Block Instruction Budget integer control"))
            .with_param(DspParamSchema::knob("execution_time_cap_us", "Hard Execution Watchdog", 10.0, 1000.0, 150.0, "us", MacroRole::Tone, "Hard Execution Watchdog parameter"))
            .with_param(DspParamSchema::toggle("hot_reload_kernel", "Auto Hot-Reload Wasm Binary", true, "Auto Hot-Reload Wasm Binary toggle switch"))
        );
        descriptors.insert("HardwareMidiClockJitterFilterNode".to_string(), DspNodeDescriptor::new("HardwareMidiClockJitterFilterNode", "Sub-Microsecond Phase-Locked Loop MIDI Clock Sync", DspNodeCategory::Utility, "Sub-Microsecond Phase-Locked Loop MIDI Clock Sync")
            .with_param(DspParamSchema::knob("pll_filter_damping", "PLL Phase Filter Damping", 0.1, 1.0, 0.707, "Q", MacroRole::Tone, "PLL Phase Filter Damping parameter"))
            .with_param(DspParamSchema::knob("jitter_tolerance_us", "Clock Jitter Window", 1.0, 500.0, 25.0, "us", MacroRole::Tone, "Clock Jitter Window parameter"))
            .with_param(DspParamSchema::knob("clock_ppqn_multiplier", "Clock PPQN Multiplier", 24.0, 96.0, 24.0, "ppqn", MacroRole::Character, "Clock PPQN Multiplier integer control"))
            .with_param(DspParamSchema::knob("hardware_latency_offset_ms", "Hardware Roundtrip Offset", -50.0, 50.0, 0.0, "ms", MacroRole::Tone, "Hardware Roundtrip Offset parameter"))
        );
        descriptors.insert("PluginSandboxScannerNode".to_string(), DspNodeDescriptor::new("PluginSandboxScannerNode", "Out-of-Process Crash-Proof Plugin Sandboxing & Scanner", DspNodeCategory::Utility, "Out-of-Process Crash-Proof Plugin Sandboxing & Scanner")
            .with_param(DspParamSchema::toggle("isolation_process_mode", "Out-of-Process Sandboxing", true, "Out-of-Process Sandboxing toggle switch"))
            .with_param(DspParamSchema::knob("sandbox_timeout_ms", "Plugin Init Timeout", 500.0, 10000.0, 3000.0, "ms", MacroRole::Tone, "Plugin Init Timeout parameter"))
            .with_param(DspParamSchema::knob("ipc_shared_memory_mb", "IPC Ring Buffer Size", 2.0, 32.0, 8.0, "MB", MacroRole::Character, "IPC Ring Buffer Size integer control"))
            .with_param(DspParamSchema::toggle("crash_auto_recover", "Auto-Restart Crashed Plugin", true, "Auto-Restart Crashed Plugin toggle switch"))
        );
        descriptors.insert("PluginParamAutomapEngineNode".to_string(), DspNodeDescriptor::new("PluginParamAutomapEngineNode", "AI Semantic Parameter Auto-Mapping & Grouping Engine", DspNodeCategory::Utility, "AI Semantic Parameter Auto-Mapping & Grouping Engine")
            .with_param(DspParamSchema::choice("parameter_semantic_model", "AI Grouping Semantic Model", &["Synthesizer 8-Macro Standard", "Channel Strip EQ/Dynamics", "Spatial 3D Soundfield", "Mastering Chain"], 0, "AI Grouping Semantic Model mode selector"))
            .with_param(DspParamSchema::knob("macro_cluster_group_count", "Auto-Generated Macro Groups", 2.0, 8.0, 4.0, "macros", MacroRole::Character, "Auto-Generated Macro Groups integer control"))
            .with_param(DspParamSchema::knob("ai_grouping_confidence", "Semantic Confidence Threshold", 0.5, 1.0, 0.85, "%", MacroRole::Tone, "Semantic Confidence Threshold parameter"))
            .with_param(DspParamSchema::toggle("remap_triggers", "Auto-Learn Controller Hardware", true, "Auto-Learn Controller Hardware toggle switch"))
        );
        descriptors.insert("CvGateSignalGeneratorNode".to_string(), DspNodeDescriptor::new("CvGateSignalGeneratorNode", "Modular Synth 1V/Oct CV Pitch & Gate Pulse Generator", DspNodeCategory::Utility, "Modular Synth 1V/Oct CV Pitch & Gate Pulse Generator")
            .with_param(DspParamSchema::knob("cv_voltage_output_volts", "1V/Oct Pitch Voltage", -5.0, 10.0, 0.0, "V", MacroRole::Tone, "1V/Oct Pitch Voltage parameter"))
            .with_param(DspParamSchema::knob("gate_pulse_width_ms", "Gate Pulse Duration", 1.0, 200.0, 25.0, "ms", MacroRole::Tone, "Gate Pulse Duration parameter"))
            .with_param(DspParamSchema::knob("gate_voltage_high_v", "Gate High Active Level", 1.0, 10.0, 5.0, "V", MacroRole::Tone, "Gate High Active Level parameter"))
            .with_param(DspParamSchema::knob("portamento_slew_ms", "Portamento Glide Slew", 0.0, 500.0, 0.0, "ms", MacroRole::Tone, "Portamento Glide Slew parameter"))
        );
        descriptors.insert("DinSync24PulseGeneratorNode".to_string(), DspNodeDescriptor::new("DinSync24PulseGeneratorNode", "Roland DIN Sync 24 PPQN Hardware Clock Generator", DspNodeCategory::Utility, "Roland DIN Sync 24 PPQN Hardware Clock Generator")
            .with_param(DspParamSchema::knob("din_sync_ppqn", "DIN Sync Pulse Resolution", 24.0, 48.0, 24.0, "ppqn", MacroRole::Character, "DIN Sync Pulse Resolution integer control"))
            .with_param(DspParamSchema::toggle("run_stop_gate_active", "DIN Run / Stop Voltage High", true, "DIN Run / Stop Voltage High toggle switch"))
            .with_param(DspParamSchema::knob("pulse_width_duty_cycle", "Clock Duty Cycle", 0.1, 0.9, 0.5, "%", MacroRole::Tone, "Clock Duty Cycle parameter"))
            .with_param(DspParamSchema::knob("swing_shuffle_percent", "Hardware Swing Timing", 50.0, 75.0, 50.0, "%", MacroRole::Tone, "Hardware Swing Timing parameter"))
        );
        descriptors.insert("BleMidiControllerDriverNode".to_string(), DspNodeDescriptor::new("BleMidiControllerDriverNode", "Bluetooth Low Energy (BLE-MIDI) Low-Latency Driver", DspNodeCategory::Utility, "Bluetooth Low Energy (BLE-MIDI) Low-Latency Driver")
            .with_param(DspParamSchema::knob("ble_connection_interval_ms", "BLE Connection Latency", 7.5, 30.0, 11.25, "ms", MacroRole::Tone, "BLE Connection Latency parameter"))
            .with_param(DspParamSchema::knob("packet_timestamp_align_us", "Timestamp Jitter Alignment", 10.0, 200.0, 50.0, "us", MacroRole::Tone, "Timestamp Jitter Alignment parameter"))
            .with_param(DspParamSchema::knob("signal_rssi_indicator_dbm", "Wireless Signal RSSI", -100.0, -30.0, -55.0, "dBm", MacroRole::Tone, "Wireless Signal RSSI parameter"))
            .with_param(DspParamSchema::toggle("auto_reconnect", "Auto-Reconnect Paired Device", true, "Auto-Reconnect Paired Device toggle switch"))
        );
        descriptors.insert("SupercriticalFluidNoiseNode".to_string(), DspNodeDescriptor::new("SupercriticalFluidNoiseNode", "Supercritical Phase-Transition Acoustic Noise Generator", DspNodeCategory::Utility, "Supercritical Phase-Transition Acoustic Noise Generator")
            .with_param(DspParamSchema::knob("fluid_critical_temp_k", "Critical Temperature (Tc)", 250.0, 600.0, 304.13, "K", MacroRole::Tone, "Critical Temperature (Tc) parameter"))
            .with_param(DspParamSchema::knob("fluid_critical_pressure_bar", "Critical Pressure (Pc)", 30.0, 150.0, 73.75, "bar", MacroRole::Tone, "Critical Pressure (Pc) parameter"))
            .with_param(DspParamSchema::knob("density_fluctuation_amplitude", "Phase Density Fluctuations", 0.0, 1.0, 0.7, "%", MacroRole::Tone, "Phase Density Fluctuations parameter"))
            .with_param(DspParamSchema::knob("output_noise_gain", "Acoustic Noise Output", 0.0, 1.0, 0.6, "%", MacroRole::Tone, "Acoustic Noise Output parameter"))
        );
        descriptors.insert("SonoluminescenceSonifierNode".to_string(), DspNodeDescriptor::new("SonoluminescenceSonifierNode", "Ultrasonic Acoustic Cavitation Bubble Sonifier", DspNodeCategory::Utility, "Ultrasonic Acoustic Cavitation Bubble Sonifier")
            .with_param(DspParamSchema::log_knob("ultrasound_drive_freq_khz", "Ultrasound Driver Frequency", 20.0, 100.0, 26.5, "kHz", MacroRole::Tone, "Ultrasound Driver Frequency logarithmic parameter"))
            .with_param(DspParamSchema::knob("cavitation_acoustic_pressure_atm", "Acoustic Drive Pressure", 0.5, 5.0, 1.4, "atm", MacroRole::Tone, "Acoustic Drive Pressure parameter"))
            .with_param(DspParamSchema::knob("bubble_flash_intensity", "Picosecond Light Flash Energy", 0.0, 1.0, 0.75, "%", MacroRole::Tone, "Picosecond Light Flash Energy parameter"))
            .with_param(DspParamSchema::knob("sparkle_audio_mix", "Cavitation Sparkle Level", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Cavitation Sparkle Level parameter"))
        );
        descriptors.insert("GravitationalWaveChirpNode".to_string(), DspNodeDescriptor::new("GravitationalWaveChirpNode", "Binary Black Hole Inspiral Gravitational Wave Chirp", DspNodeCategory::Utility, "Binary Black Hole Inspiral Gravitational Wave Chirp")
            .with_param(DspParamSchema::knob("component_mass_m1_solar", "Black Hole Primary Mass (M1)", 5.0, 100.0, 36.0, "Msun", MacroRole::Tone, "Black Hole Primary Mass (M1) parameter"))
            .with_param(DspParamSchema::knob("component_mass_m2_solar", "Black Hole Secondary Mass (M2)", 5.0, 100.0, 29.0, "Msun", MacroRole::Tone, "Black Hole Secondary Mass (M2) parameter"))
            .with_param(DspParamSchema::knob("coalescence_time_s", "Inspiral Coalescence Time", 0.1, 10.0, 1.5, "s", MacroRole::Tone, "Inspiral Coalescence Time parameter"))
            .with_param(DspParamSchema::knob("strain_amplitude_h", "Gravitational Wave Strain H", 0.0, 1.0, 0.85, "%", MacroRole::Tone, "Gravitational Wave Strain H parameter"))
        );
        descriptors.insert("CasimirVacuumNoiseNode".to_string(), DspNodeDescriptor::new("CasimirVacuumNoiseNode", "Quantum Vacuum Zero-Point Energy Fluctuation Noise", DspNodeCategory::Utility, "Quantum Vacuum Zero-Point Energy Fluctuation Noise")
            .with_param(DspParamSchema::knob("plate_separation_distance_nm", "Conducting Plate Distance", 1.0, 100.0, 15.0, "nm", MacroRole::Tone, "Conducting Plate Distance parameter"))
            .with_param(DspParamSchema::knob("planck_energy_scale", "Planck Energy Fluctuation Scale", 0.1, 5.0, 1.0, "x", MacroRole::Tone, "Planck Energy Fluctuation Scale parameter"))
            .with_param(DspParamSchema::knob("vacuum_polarization_factor", "Vacuum Polarization Bias", 0.0, 1.0, 0.45, "%", MacroRole::Tone, "Vacuum Polarization Bias parameter"))
            .with_param(DspParamSchema::knob("spectral_color_slope", "Quantum Noise Color Slope", -3.0, 3.0, 1.0, "dB/oct", MacroRole::Tone, "Quantum Noise Color Slope parameter"))
        );
        descriptors.insert("RelativisticDopplerShiftNode".to_string(), DspNodeDescriptor::new("RelativisticDopplerShiftNode", "Lorentz Relativistic Velocity Doppler Time Dilation", DspNodeCategory::Utility, "Lorentz Relativistic Velocity Doppler Time Dilation")
            .with_param(DspParamSchema::knob("velocity_fraction_c", "Relativistic Velocity (v/c)", 0.01, 0.99, 0.5, "c", MacroRole::Tone, "Relativistic Velocity (v/c) parameter"))
            .with_param(DspParamSchema::knob("lorentz_gamma_factor", "Lorentz Factor Gamma", 1.0, 10.0, 1.15, "gamma", MacroRole::Tone, "Lorentz Factor Gamma parameter"))
            .with_param(DspParamSchema::knob("source_trajectory_angle_deg", "Observer Angle Theta", 0.0, 180.0, 45.0, "deg", MacroRole::Tone, "Observer Angle Theta parameter"))
            .with_param(DspParamSchema::toggle("relativistic_aberration", "Relativistic Headlight Beaming", true, "Relativistic Headlight Beaming toggle switch"))
        );
        descriptors.insert("StochasticQuantumDecoherenceNoise".to_string(), DspNodeDescriptor::new("StochasticQuantumDecoherenceNoise", "Quantum Decoherence Lindblad Superoperator Noise", DspNodeCategory::Utility, "Quantum Decoherence Lindblad Superoperator Noise")
            .with_param(DspParamSchema::knob("lindblad_dissipator_rate_gamma", "Lindblad Decoherence Rate (Gamma)", 0.01, 10.0, 1.2, "gamma", MacroRole::Tone, "Lindblad Decoherence Rate (Gamma) parameter"))
            .with_param(DspParamSchema::knob("environmental_bath_temp_k", "Thermal Bath Temperature", 0.01, 300.0, 4.2, "K", MacroRole::Tone, "Thermal Bath Temperature parameter"))
            .with_param(DspParamSchema::knob("pure_dephasing_rate", "Pure Dephasing Noise Rate", 0.0, 5.0, 0.65, "", MacroRole::Tone, "Pure Dephasing Noise Rate parameter"))
            .with_param(DspParamSchema::knob("quantum_noise_mix", "Decoherence Noise Audio Blend", 0.0, 1.0, 0.4, "%", MacroRole::Tone, "Decoherence Noise Audio Blend parameter"))
        );
        descriptors.insert("QuantumTeleportationAudioBufferBus".to_string(), DspNodeDescriptor::new("QuantumTeleportationAudioBufferBus", "Zero-Latency Entangled State Audio Bus Interlink", DspNodeCategory::Utility, "Zero-Latency Entangled State Audio Bus Interlink")
            .with_param(DspParamSchema::knob("epr_pair_fidelity", "EPR Entanglement Fidelity", 0.5, 1.0, 0.98, "%", MacroRole::Tone, "EPR Entanglement Fidelity parameter"))
            .with_param(DspParamSchema::knob("bell_measurement_feedforward_gain", "Feedforward Recovery Gain", 0.0, 2.0, 1.0, "x", MacroRole::Tone, "Feedforward Recovery Gain parameter"))
            .with_param(DspParamSchema::knob("teleportation_channel_id", "Teleportation Bus Channel ID", 0.0, 15.0, 0.0, "ch", MacroRole::Character, "Teleportation Bus Channel ID integer control"))
            .with_param(DspParamSchema::toggle("state_entanglement_bus", "Quantum Entanglement Linked", true, "Quantum Entanglement Linked toggle switch"))
        );
        descriptors.insert("QuantumPhaseEstimationPitchTracker".to_string(), DspNodeDescriptor::new("QuantumPhaseEstimationPitchTracker", "Quantum Phase Estimation Sub-Cent Pitch Tracker", DspNodeCategory::Utility, "Quantum Phase Estimation Sub-Cent Pitch Tracker")
            .with_param(DspParamSchema::knob("evaluation_qubits_count", "Phase Estimation Qubits", 4.0, 16.0, 10.0, "qubits", MacroRole::Character, "Phase Estimation Qubits integer control"))
            .with_param(DspParamSchema::knob("unitary_evolution_time_step", "Unitary Time Step Delta", 0.01, 1.0, 0.1, "ms", MacroRole::Tone, "Unitary Time Step Delta parameter"))
            .with_param(DspParamSchema::knob("frequency_resolution_cents", "Sub-Cent Frequency Accuracy", 0.01, 5.0, 0.1, "cents", MacroRole::Tone, "Sub-Cent Frequency Accuracy parameter"))
            .with_param(DspParamSchema::knob("phase_lock_threshold", "Eigenphase Probability Lock", 0.5, 0.99, 0.9, "%", MacroRole::Tone, "Eigenphase Probability Lock parameter"))
        );
        descriptors.insert("AtmosphericDensityNode".to_string(), DspNodeDescriptor::new("AtmosphericDensityNode", "Atmospheric Gas Composition & Speed of Sound Simulator", DspNodeCategory::SpatialSurround, "Atmospheric Gas Composition & Speed of Sound Simulator")
            .with_param(DspParamSchema::knob("ambient_pressure_kpa", "Atmospheric Pressure", 1.0, 1000.0, 101.3, "kPa", MacroRole::Tone, "Atmospheric Pressure parameter"))
            .with_param(DspParamSchema::knob("temperature_celsius", "Ambient Temperature", -100.0, 200.0, 20.0, "degC", MacroRole::Tone, "Ambient Temperature parameter"))
            .with_param(DspParamSchema::knob("helium_fraction_ratio", "Helium Gas Ratio (Speed Boost)", 0.0, 1.0, 0.0, "%", MacroRole::Tone, "Helium Gas Ratio (Speed Boost) parameter"))
            .with_param(DspParamSchema::knob("co2_fraction_ratio", "CO2 Gas Ratio (Speed Drop)", 0.0, 1.0, 0.0, "%", MacroRole::Tone, "CO2 Gas Ratio (Speed Drop) parameter"))
        );
        descriptors.insert("QuantumDotTransducerNode".to_string(), DspNodeDescriptor::new("QuantumDotTransducerNode", "Nanoscale Quantum Dot Optical-to-Audio Transducer", DspNodeCategory::Utility, "Nanoscale Quantum Dot Optical-to-Audio Transducer")
            .with_param(DspParamSchema::knob("optical_wavelength_nm", "Optical Wavelength", 200.0, 1100.0, 532.0, "nm", MacroRole::Tone, "Optical Wavelength parameter"))
            .with_param(DspParamSchema::knob("quantum_yield_efficiency", "Quantum Yield Efficiency", 0.1, 1.0, 0.9, "%", MacroRole::Tone, "Quantum Yield Efficiency parameter"))
            .with_param(DspParamSchema::knob("fluorescence_lifetime_ns", "Carrier Lifetime Decay", 0.1, 50.0, 8.0, "ns", MacroRole::Tone, "Carrier Lifetime Decay parameter"))
            .with_param(DspParamSchema::knob("photocurrent_audio_gain", "Optoelectronic Audio Gain", 0.0, 2.0, 1.0, "x", MacroRole::Tone, "Optoelectronic Audio Gain parameter"))
        );
        descriptors.insert("AcousticLevitationTrapNode".to_string(), DspNodeDescriptor::new("AcousticLevitationTrapNode", "Gor'kov Ultrasound Acoustic Radiation Pressure Trap", DspNodeCategory::Utility, "Gor'kov Ultrasound Acoustic Radiation Pressure Trap")
            .with_param(DspParamSchema::log_knob("ultrasound_carrier_freq_khz", "Ultrasound Carrier Frequency", 20.0, 100.0, 40.0, "kHz", MacroRole::Tone, "Ultrasound Carrier Frequency logarithmic parameter"))
            .with_param(DspParamSchema::knob("standing_wave_trap_nodes", "Standing Wave Trapping Nodes", 1.0, 16.0, 4.0, "nodes", MacroRole::Character, "Standing Wave Trapping Nodes integer control"))
            .with_param(DspParamSchema::knob("levitated_particle_mass_mg", "Particle Mass Scale", 0.1, 50.0, 2.0, "mg", MacroRole::Tone, "Particle Mass Scale parameter"))
            .with_param(DspParamSchema::knob("radiation_pressure_gain", "Radiation Potential Force", 0.1, 10.0, 2.5, "x", MacroRole::Tone, "Radiation Potential Force parameter"))
        );
        descriptors.insert("MathAdd".to_string(), DspNodeDescriptor::new("MathAdd", "Signal Arithmetic Sum & Offset Scale", DspNodeCategory::Utility, "Signal Arithmetic Sum & Offset Scale")
            .with_param(DspParamSchema::knob("offset", "DC Offset Constant", -10.0, 10.0, 0.0, "", MacroRole::Tone, "DC Offset Constant parameter"))
            .with_param(DspParamSchema::knob("scale", "Scale Multiplier", 0.0, 5.0, 1.0, "x", MacroRole::Tone, "Scale Multiplier parameter"))
        );
        descriptors.insert("MathMult".to_string(), DspNodeDescriptor::new("MathMult", "Signal Arithmetic Product & Modulation Scale", DspNodeCategory::Utility, "Signal Arithmetic Product & Modulation Scale")
            .with_param(DspParamSchema::knob("factor", "Multiplication Factor", -5.0, 5.0, 1.0, "x", MacroRole::Tone, "Multiplication Factor parameter"))
            .with_param(DspParamSchema::knob("bias", "Post-Multiplier Bias", -1.0, 1.0, 0.0, "", MacroRole::Tone, "Post-Multiplier Bias parameter"))
        );
        descriptors.insert("VCA".to_string(), DspNodeDescriptor::new("VCA", "Voltage-Controlled Amplifier Gain Block", DspNodeCategory::Utility, "Voltage-Controlled Amplifier Gain Block")
            .with_param(DspParamSchema::knob("gain", "Base VCA Gain", 0.0, 2.0, 1.0, "x", MacroRole::Tone, "Base VCA Gain parameter"))
            .with_param(DspParamSchema::knob("cv_amount", "CV Modulation Depth", 0.0, 1.0, 1.0, "%", MacroRole::Tone, "CV Modulation Depth parameter"))
        );
        descriptors.insert("WsolaTimeStretcher".to_string(), DspNodeDescriptor::new("WsolaTimeStretcher", "WSOLA Pitch-Preserving Time Stretch Engine", DspNodeCategory::Utility, "WSOLA Pitch-Preserving Time Stretch Engine")
            .with_param(DspParamSchema::knob("stretch_ratio", "Time Stretch Ratio", 0.25, 4.0, 1.0, "x", MacroRole::Tone, "Time Stretch Ratio parameter"))
            .with_param(DspParamSchema::knob("window_ms", "Synthesis Window Size", 10.0, 100.0, 40.0, "ms", MacroRole::Tone, "Synthesis Window Size parameter"))
            .with_param(DspParamSchema::knob("seek_window_ms", "Cross-Correlation Seek Window", 5.0, 50.0, 15.0, "ms", MacroRole::Tone, "Cross-Correlation Seek Window parameter"))
        );
        descriptors.insert("StemSeparator".to_string(), DspNodeDescriptor::new("StemSeparator", "Neural 4-Stem Audio Source Separation Matrix", DspNodeCategory::Utility, "Neural 4-Stem Audio Source Separation Matrix")
            .with_param(DspParamSchema::knob("vocals_gain", "Vocals Stem Gain", 0.0, 2.0, 1.0, "x", MacroRole::Tone, "Vocals Stem Gain parameter"))
            .with_param(DspParamSchema::knob("drums_gain", "Drums Stem Gain", 0.0, 2.0, 1.0, "x", MacroRole::Tone, "Drums Stem Gain parameter"))
            .with_param(DspParamSchema::knob("bass_gain", "Bass Stem Gain", 0.0, 2.0, 1.0, "x", MacroRole::Tone, "Bass Stem Gain parameter"))
            .with_param(DspParamSchema::knob("other_gain", "Instruments Stem Gain", 0.0, 2.0, 1.0, "x", MacroRole::Tone, "Instruments Stem Gain parameter"))
        );
        descriptors.insert("ElasticWarpEngine".to_string(), DspNodeDescriptor::new("ElasticWarpEngine", "Real-time Elastic Audio Warp & Transient Pinning", DspNodeCategory::Utility, "Real-time Elastic Audio Warp & Transient Pinning")
            .with_param(DspParamSchema::knob("time_ratio", "Elastic Stretch Ratio", 0.2, 5.0, 1.0, "x", MacroRole::Tone, "Elastic Stretch Ratio parameter"))
            .with_param(DspParamSchema::toggle("transient_preserve", "Lock Beat Transients", true, "Lock Beat Transients toggle switch"))
            .with_param(DspParamSchema::knob("grain_overlap", "Grain Overlap Factor", 0.1, 0.9, 0.5, "%", MacroRole::Tone, "Grain Overlap Factor parameter"))
        );
        descriptors.insert("Oversampler".to_string(), DspNodeDescriptor::new("Oversampler", "Polyphase Anti-Aliasing Oversampling Filter", DspNodeCategory::Utility, "Polyphase Anti-Aliasing Oversampling Filter")
            .with_param(DspParamSchema::choice("factor", "Oversampling Factor", &["2x Oversampling", "4x Oversampling", "8x Oversampling", "16x Oversampling"], 1, "Oversampling Factor mode selector"))
            .with_param(DspParamSchema::toggle("linear_phase", "Linear Phase Reconstruction", true, "Linear Phase Reconstruction toggle switch"))
        );
        descriptors.insert("PassthroughNode".to_string(), DspNodeDescriptor::new("PassthroughNode", "Zero-Latency Transparent Buffer Relay", DspNodeCategory::Utility, "Zero-Latency Transparent Buffer Relay")
            .with_param(DspParamSchema::toggle("enabled", "Relay Active", true, "Relay Active toggle switch"))
            .with_param(DspParamSchema::knob("trim_db", "Relay Output Trim", -12.0, 12.0, 0.0, "dB", MacroRole::Tone, "Relay Output Trim parameter"))
        );

        Self { descriptors }
    }

    /// Retrieve a node descriptor by kind ID.
    pub fn get(&self, kind_id: &str) -> Option<&DspNodeDescriptor> {
        let clean = Self::normalize_type_name(kind_id).unwrap_or(kind_id);
        self.descriptors.get(clean)
    }

    /// Returns a list of all registered node descriptors.
    pub fn list_all(&self) -> Vec<&DspNodeDescriptor> {
        let mut list: Vec<&DspNodeDescriptor> = self.descriptors.values().collect();
        list.sort_by(|a, b| a.kind_id.cmp(&b.kind_id));
        list
    }

    /// Filter node descriptors by category.
    pub fn by_category(&self, category: DspNodeCategory) -> Vec<&DspNodeDescriptor> {
        self.descriptors
            .values()
            .filter(|d| d.category == category)
            .collect()
    }

    /// Alias for by_category.
    pub fn list_by_category(&self, category: DspNodeCategory) -> Vec<&DspNodeDescriptor> {
        self.by_category(category)
    }

    /// Search descriptors by name, kind, or description keywords.
    pub fn search(&self, query: &str) -> Vec<&DspNodeDescriptor> {
        let q = query.to_lowercase();
        self.descriptors
            .values()
            .filter(|d| {
                d.kind_id.to_lowercase().contains(&q)
                    || d.display_name.to_lowercase().contains(&q)
                    || d.description.to_lowercase().contains(&q)
            })
            .collect()
    }

    /// List of all registered DSP module type names with category and friendly description.
    pub fn inventory() -> Vec<(&'static str, DspNodeCategory, &'static str)> {
        vec![

            ("OscSine", DspNodeCategory::Oscillator, "Pure Sine Wave Oscillator"),
            ("OscSaw", DspNodeCategory::Oscillator, "Band-limited Sawtooth Oscillator"),
            ("OscPulse", DspNodeCategory::Oscillator, "Pulse Width Modulation Oscillator"),
            ("OscTriangle", DspNodeCategory::Oscillator, "Anti-aliased Triangle Generator"),
            ("OscWavetable", DspNodeCategory::Oscillator, "SIMD Wavetable Morphing Synthesizer"),
            ("SimdPolyWavetableOscillator", DspNodeCategory::Oscillator, "AVX2/NEON Polyphonic Wavetable Voice Bank"),
            ("GranularSynthNode", DspNodeCategory::Oscillator, "Cloud Granular Audio Streamer"),
            ("NeuralWavetable", DspNodeCategory::NeuralAi, "Latent AI Neural Wavetable Synthesizer"),
            ("PlasmaArcSynthesizer", DspNodeCategory::Oscillator, "High-Voltage Plasma Arc Audio Generator"),
            ("NoiseGen", DspNodeCategory::Oscillator, "Multi-Color Spectral Noise Generator"),
            ("AetherSynth", DspNodeCategory::CompositeSynth, "Dual Saw/Pulse Subtractive Synth with Moog Ladder Filter"),
            ("PluckSynth", DspNodeCategory::Oscillator, "Karplus-Strong Plucked String Synthesis Core"),
            ("CyberpunkSubSynth", DspNodeCategory::CompositeSynth, "Heavy Sub-bass Synth with Asymmetric Wavefolding"),
            ("AtmosphericPadSynth", DspNodeCategory::CompositeSynth, "Detuned Ambient Pad with SVF Filter & Space Reverb"),
            ("GlitchPercussionSynth", DspNodeCategory::CompositeSynth, "Noise Burst & Granular Stutter Percussion Engine"),
            ("GlitchAetherMachine", DspNodeCategory::CompositeSynth, "Flagship Chopper & Tape-Stop Glitch Synthesizer"),
            ("FmOperatorPair", DspNodeCategory::Oscillator, "2-Operator Phase Modulation FM Synthesizer"),
            ("FmMatrixSynthesizer", DspNodeCategory::CompositeSynth, "6-Operator DX-Style FM Matrix Synthesizer"),
            ("AnalogDrumVoice", DspNodeCategory::Oscillator, "808/909 Style Analog Synthesized Drum Voice"),
            ("DrumMachineDevice", DspNodeCategory::SamplerSlicer, "24-Pad Zero-Allocation Sampling Drum Machine"),
            ("QuantumStateVectorOscillator", DspNodeCategory::Oscillator, "Qubit Phase Superposition & Hadamard Gate Synth"),
            ("QuantumHarmonicOscillatorVoice", DspNodeCategory::Oscillator, "Quantum Harmonic Oscillator Voice Bank"),
            ("NavierStokesFluidNode", DspNodeCategory::Oscillator, "Navier-Stokes 3D Acoustic Fluid Dynamics"),
            ("MolecularVibrationResonator", DspNodeCategory::Oscillator, "Molecular Vibration Resonator Crystalline Lattice"),
            ("ClosedLoopNeuroFeedbackOscillator", DspNodeCategory::Oscillator, "Closed-Loop Neuro-Feedback Relaxation Oscillator"),
            ("BrainwaveEntrainmentBinauralBeat", DspNodeCategory::Oscillator, "Brainwave Entrainment Binaural Beat Generator"),
            ("FusionResonanceSynth", DspNodeCategory::CompositeSynth, "Tokamak Fusion Magnetic Plasma Resonance Synth"),
            ("SubharmonicSynthNode", DspNodeCategory::Oscillator, "Subharmonic Multi-Octave Sub Oscillator"),
            ("SamplerDevice", DspNodeCategory::SamplerSlicer, "Multi-Sample Instrument with ADSR Envelope & Filter"),
            ("MultiSamplerNode", DspNodeCategory::SamplerSlicer, "Velocity-Layered Multi-Zone Audio Sampler"),
            ("SingingSynthesisNode", DspNodeCategory::NeuralAi, "Differentiable Neural Phoneme Singing Synthesizer"),
            ("SpectralNeuralResynthesizerNode", DspNodeCategory::SpectralResynthesis, "Latent FFT Timbre Morphing Resynthesizer"),
            ("NeuralAudioRepairNode", DspNodeCategory::NeuralAi, "AI Spectral Audio Inpainting & De-Clicking Engine"),
            ("FilterSVF", DspNodeCategory::FilterEq, "State-Variable Multi-Mode Filter (LP/HP/BP/Notch)"),
            ("FilterLadder", DspNodeCategory::FilterEq, "24dB/Oct Transistor Moog-Style Ladder Filter"),
            ("FilterLadder8", DspNodeCategory::FilterEq, "8-Pole 48dB/Oct Ultra-Steep Resonant Ladder Filter"),
            ("FilterComb", DspNodeCategory::FilterEq, "Feedback Comb Resonator Filter"),
            ("FilterBiquad", DspNodeCategory::FilterEq, "Direct Form II Transposed Biquad Multi-Mode Filter"),
            ("DcBlockFilter", DspNodeCategory::FilterEq, "Zero-Phase DC Offset Removal Highpass Filter"),
            ("LowCutFilter", DspNodeCategory::FilterEq, "Precision Sub-Sonic Low-Cut Highpass Filter"),
            ("HighCutFilter", DspNodeCategory::FilterEq, "Anti-Aliasing High-Cut Lowpass Filter"),
            ("ParametricEqNode", DspNodeCategory::FilterEq, "4-Band Precision Parametric Equalizer"),
            ("MultiChannelSpectralEqualizerNode", DspNodeCategory::SpectralResynthesis, "64-Band Multi-Channel Spectral Sculptor"),
            ("FormantFilterNode", DspNodeCategory::FilterEq, "Vowel Formant Filter Matrix"),
            ("ModalFilterNode", DspNodeCategory::FilterEq, "High-Order Acoustic Modal Resonator Bank"),
            ("HornReflectionFilter", DspNodeCategory::FilterEq, "Acoustic Bell Flare & Throat Reflection Filter"),
            ("AutomatedDynamicEqNode", DspNodeCategory::FilterEq, "AI-Driven Intelligent Masking Reduction Dynamic EQ"),
            ("SubharmonicQuantumTunnelingFilter", DspNodeCategory::FilterEq, "Subharmonic Quantum Tunneling Resonant Filter"),
            ("MetamaterialRefractionFilter", DspNodeCategory::FilterEq, "Negative Index Acoustic Metamaterial Filter"),
            ("SpatialAcousticHologramFilter", DspNodeCategory::FilterEq, "Spatial Acoustic Hologram Reconstruction Filter"),
            ("LinearPhaseCrossoverNode", DspNodeCategory::FilterEq, "4-Way Linear Phase Mastering Crossover Filter"),
            ("SpectralMatchingEqNode", DspNodeCategory::SpectralResynthesis, "AI 128-Band Spectral Curve Matching EQ"),
            ("SpectralTiltNode", DspNodeCategory::SpectralResynthesis, "Linear Phase Psychoacoustic Spectral Tilt Filter"),
            ("CombResonatorNode", DspNodeCategory::FilterEq, "Tuned Feedback Comb Filter Resonator"),
            ("EnvADSR", DspNodeCategory::Modulation, "Exponential Multi-Stage ADSR Envelope"),
            ("OscLFO", DspNodeCategory::Modulation, "Multi-Waveform Syncable LFO Generator"),
            ("EnvelopeFollowerNode", DspNodeCategory::Modulation, "Real-Time Peak/RMS Sidechain Envelope Follower"),
            ("ModulationMatrix", DspNodeCategory::Modulation, "16x16 Modulation Cross-Routing Matrix"),
            ("PitchShifterNode", DspNodeCategory::Modulation, "Real-time Granular Pitch & Formant Shifter"),
            ("AutoWahNode", DspNodeCategory::Modulation, "Dynamic Envelope-Controlled Auto-Wah"),
            ("AutoTuneNode", DspNodeCategory::Modulation, "Real-time Microtonal Scale Quantizer & Pitch Corrector"),
            ("AutotuneNode", DspNodeCategory::Modulation, "Vocal Scale Intonation & Pitch Quantizer"),
            ("VocalPitchFormantCorrectorNode", DspNodeCategory::Modulation, "Surgical Formant & Gender Vocal Pitch Shifter"),
            ("GlitchBufferNode", DspNodeCategory::Modulation, "Stutter, Shuffle, & Reverse Glitch Buffer Matrix"),
            ("EffectChorus", DspNodeCategory::Modulation, "Multi-Voice BBD Stereo Chorus"),
            ("EffectFlanger", DspNodeCategory::Modulation, "Through-Zero Flanger & Comb Sweep"),
            ("EffectPhaser", DspNodeCategory::Modulation, "Multi-Stage Allpass Phase Shifter"),
            ("RingModulator", DspNodeCategory::Modulation, "Carrier Multiplier Ring Modulator"),
            ("FrequencyShifter", DspNodeCategory::Modulation, "Single-Sideband Frequency Shifter"),
            ("ChaoticFractalAttractorModulator", DspNodeCategory::Modulation, "Lorenz / Rossler Chaotic Strange Attractor LFO"),
            ("QuantumEntanglementModulationRouting", DspNodeCategory::Modulation, "Quantum Entangled Bidirectional Modulator"),
            ("MhdPlasmaWaveModulator", DspNodeCategory::Modulation, "Magnetohydrodynamic Plasma Wave Modulator"),
            ("BbdChorusNode", DspNodeCategory::Modulation, "Bucket-Brigade Analog Stereo Chorus Ensemble"),
            ("ThroughZeroFlangerNode", DspNodeCategory::Modulation, "Tape-Style Through-Zero Flanger & Comb Phase"),
            ("TapeFlutterNode", DspNodeCategory::Modulation, "Capstan Wow & Tape Flutter Modulator"),
            ("GranularFreezeNode", DspNodeCategory::Modulation, "Infinite Granular Audio Buffer Freeze Engine"),
            ("GranularPitchShifter", DspNodeCategory::Modulation, "Dual-Grain Pitch Transposition & Shifter"),
            ("PitchCorrectorNode", DspNodeCategory::Modulation, "Scale-Quantized Microtonal Vocal Pitch Corrector"),
            ("VocoderMatrixNode", DspNodeCategory::SpectralResynthesis, "32-Band Spectral Vocoder Analysis & Synthesis Matrix"),
            ("GamelanGender", DspNodeCategory::AcousticPhysicalModel, "Indonesian Gamelan Gendèr Metallophone & Bamboo Tubes"),
            ("HurdyGurdy", DspNodeCategory::AcousticPhysicalModel, "Vielle à Roue Rosin Wheel, Melody Chanters & Chien Buzz"),
            ("SitarModel", DspNodeCategory::AcousticPhysicalModel, "Indian Sitar, Curved Jawari Bridge & Sympathetic Tarab Bank"),
            ("GrandPianoModel", DspNodeCategory::AcousticPhysicalModel, "Concert Grand Piano, Soundboard Coupling & 3 Pedals"),
            ("ShakuhachiModel", DspNodeCategory::AcousticPhysicalModel, "Japanese Bamboo Flute & Air-Reed Vortex Aerodynamics"),
            ("PipeOrganModel", DspNodeCategory::AcousticPhysicalModel, "Cathedral Pipe Organ Windchest & Rank Voicing"),
            ("ClavinetModel", DspNodeCategory::AcousticPhysicalModel, "Electromagnetic Rock Clavinet & Rubber Pluck Hammer"),
            ("ElectricPianoModel", DspNodeCategory::AcousticPhysicalModel, "Tine Rhodes / Wurlitzer Reed Resonator"),
            ("GlassArmonica", DspNodeCategory::AcousticPhysicalModel, "Franklin Glass Armonica Friction Model"),
            ("BowedStringModel", DspNodeCategory::AcousticPhysicalModel, "Stradivarius Violin / Cello Bow Friction"),
            ("SteelpanModel", DspNodeCategory::AcousticPhysicalModel, "Trinidad Steelpan Drum Membrane Shell"),
            ("TurkishNeyModel", DspNodeCategory::AcousticPhysicalModel, "Turkish Ney End-Blown Cane Flute"),
            ("WaveguideBrassModel", DspNodeCategory::AcousticPhysicalModel, "Waveguide Lip-Reed Acoustic Brass & Flare"),
            ("WoodwindJetModel", DspNodeCategory::AcousticPhysicalModel, "Flute Jet Instability & Aerodynamic Vortex"),
            ("SpringLatticeNode", DspNodeCategory::TimeSpace, "Non-linear Spring Reverb Lattice & Dispersion"),
            ("PlateTankNode", DspNodeCategory::TimeSpace, "EMT 140 Plate Reverb Tank & Transducer Dispersion"),
            ("TonewheelOrganModel", DspNodeCategory::AcousticPhysicalModel, "Hammond B3 Tonewheel Organ & 9 Drawbars"),
            ("VocalTractNode", DspNodeCategory::AcousticPhysicalModel, "Kelly-Lochbaum Acoustic Vocal Tract & Formants"),
            ("BambooResonatorBank", DspNodeCategory::AcousticPhysicalModel, "Coupled Bamboo Air-Column Tube Resonators"),
            ("MizrabPluckModel", DspNodeCategory::AcousticPhysicalModel, "Wire Plectrum Transient Strike & Release Model"),
            ("JawariBridgeModel", DspNodeCategory::AcousticPhysicalModel, "Curved Flat Jawari Bridge Grazing Dynamics"),
            ("SoundboardBridgeModel", DspNodeCategory::AcousticPhysicalModel, "Spruce Soundboard & Rib Stiffness Impedance Bridge"),
            ("TineResonatorModel", DspNodeCategory::AcousticPhysicalModel, "Tuned Spring Steel Tine & Tonebar Resonator"),
            ("TrompetteChienModel", DspNodeCategory::AcousticPhysicalModel, "Vielle Trompette Buzzing Dog Bridge Mechanism"),
            ("WindchestAerodynamicsModel", DspNodeCategory::AcousticPhysicalModel, "Pipe Organ Windchest Pallet Valve & Reservoir"),
            ("AirReedAcousticModel", DspNodeCategory::AcousticPhysicalModel, "Aeroacoustic Air-Reed Jet Stream Instability"),
            ("DigitalWaveguideNode", DspNodeCategory::AcousticPhysicalModel, "Bidirectional Delay-Line Digital Waveguide Core"),
            ("WaveguideMesh2D", DspNodeCategory::AcousticPhysicalModel, "2D Triangular Mesh Wave Propagation Surface"),
            ("RotarySpeakerModel", DspNodeCategory::AcousticPhysicalModel, "Leslie Dual-Rotor Doppler & Cabinet Diffraction"),
            ("StruckIdiophoneResonatorNode", DspNodeCategory::AcousticPhysicalModel, "Xylophone/Marimba Tuned Bar Modal Resonator"),
            ("MembranePercussionModel", DspNodeCategory::AcousticPhysicalModel, "2D Drumhead Membrane & Air Cavity Coupling"),
            ("GlottalPulseNode", DspNodeCategory::AcousticPhysicalModel, "Rosenberg Glottal Flow Waveform Model"),
            ("HammerStrikeModel", DspNodeCategory::AcousticPhysicalModel, "Non-linear Felt Hammer Strike Dynamics"),
            ("ToneholeGridModel", DspNodeCategory::AcousticPhysicalModel, "Woodwind Tonehole Lattice & Acoustic Radiation"),
            ("FrictionModel", DspNodeCategory::AcousticPhysicalModel, "Stick-Slip Rosin Tribological Friction Model"),
            ("ModalResonator", DspNodeCategory::AcousticPhysicalModel, "Universal Multi-Modal Resonator Filter Bank"),
            ("DulcimerCimbalomModel", DspNodeCategory::AcousticPhysicalModel, "Hammered Dulcimer & Cimbalom Wire Resonator"),
            ("MbiraKalimbaModel", DspNodeCategory::AcousticPhysicalModel, "African Mbira / Kalimba Plucked Lamellophone"),
            ("MembranePlateModel", DspNodeCategory::AcousticPhysicalModel, "Coupled 2D Membrane & Thin Plate Resonator"),
            ("MembraneResonatorNode", DspNodeCategory::AcousticPhysicalModel, "2D Elastic Drumhead Modal Resonator"),
            ("SympatheticCouplingModel", DspNodeCategory::AcousticPhysicalModel, "Sympathetic String Matrix Energy Coupling"),
            ("CompressorNode", DspNodeCategory::DynamicsMaster, "VCA Studio Compressor with RMS Detection"),
            ("MultibandCompressorNode", DspNodeCategory::DynamicsMaster, "3-Band Linear-Phase Dynamics Processor"),
            ("MultibandDynamicsProcessor", DspNodeCategory::DynamicsMaster, "Tri-Band Expander/Compressor Dynamics Matrix"),
            ("OpticalCompressorNode", DspNodeCategory::DynamicsMaster, "Vintage LA-2A Optical Photocell Dynamics"),
            ("NoiseGateNode", DspNodeCategory::DynamicsMaster, "Fast Transient Expander & Noise Gate"),
            ("DeesserNode", DspNodeCategory::DynamicsMaster, "Dynamic High-Frequency Sibilance Suppressor"),
            ("NeuralTransientShaperNode", DspNodeCategory::NeuralAi, "Deep Neural Network Transient & Sustain Sculptor"),
            ("NeuralBassGeneratorNode", DspNodeCategory::NeuralAi, "Subharmonic Low-End Neural Bass Synthesizer"),
            ("MultibandClipperNode", DspNodeCategory::DynamicsMaster, "Tri-Band Mastering Clipper & Saturation Stage"),
            ("MultibandDecompressorNode", DspNodeCategory::DynamicsMaster, "Multiband Upward Dynamic Range Decompressor"),
            ("MultibandExpanderNode", DspNodeCategory::DynamicsMaster, "Tri-Band Downward Transient Expander"),
            ("MultibandSaturatorNode", DspNodeCategory::DynamicsMaster, "3-Band Frequency-Split Saturation Warmer"),
            ("UpwardCompressorNode", DspNodeCategory::DynamicsMaster, "OTT-Style Upward Audio Compressor"),
            ("VariMuMasterNode", DspNodeCategory::DynamicsMaster, "Vintage Fairchild 670 Vari-Mu Tube Mastering Compressor"),
            ("DynamicCrestShaperNode", DspNodeCategory::DynamicsMaster, "Intelligent Peak-to-RMS Crest Factor Shaper"),
            ("ParallelTransientSaturatorNode", DspNodeCategory::DynamicsMaster, "Parallel Wet/Dry Transient Driver & Saturator"),
            ("TransientClipperNode", DspNodeCategory::DynamicsMaster, "Sub-Millisecond True Transient Soft Clipper"),
            ("TransientDeclickerNode", DspNodeCategory::DynamicsMaster, "Surgical Sample De-Clicker & Pop Filter"),
            ("TransientDesignerNode", DspNodeCategory::DynamicsMaster, "Dual-Band Attack & Sustain Transient Designer"),
            ("TransientGateNode", DspNodeCategory::DynamicsMaster, "Envelope-Triggered Transient Noise Gate"),
            ("TransientReconstructorNode", DspNodeCategory::DynamicsMaster, "Spectral Attack Transient Reconstructor"),
            ("TransientShaperNode", DspNodeCategory::DynamicsMaster, "Zero-Latency Transient Punch & Sustain Shaper"),
            ("TransientUnwrapperNode", DspNodeCategory::DynamicsMaster, "De-Compression & Dynamic Unwrapper"),
            ("DistortionNode", DspNodeCategory::DistortionSaturation, "Asymmetric Diode Clipping Wavefolder"),
            ("TapeSaturationNode", DspNodeCategory::DistortionSaturation, "Magnetic Tape Hysteresis & Flux Compression"),
            ("TubeSaturationNode", DspNodeCategory::DistortionSaturation, "Triode/Pentode Dual-Stage Tube Saturation"),
            ("TubeBiasNode", DspNodeCategory::DistortionSaturation, "Non-Linear Vacuum Tube Grid Bias Controller"),
            ("ConsoleEmulationNode", DspNodeCategory::DistortionSaturation, "British Class-A Console Channel Coloration"),
            ("BitcrusherNode", DspNodeCategory::DistortionSaturation, "Variable Sample-Rate & Bit-Depth Quantizer"),
            ("WavefolderNode", DspNodeCategory::DistortionSaturation, "Non-Linear Multi-Stage Wavefolder"),
            ("HarmonicExciterNode", DspNodeCategory::DistortionSaturation, "Aural Exciter & High-Band Harmonics Generator"),
            ("NeuralHarmonicExciterNode", DspNodeCategory::NeuralAi, "AI Spectral Sheen & High-Frequency Air Polish"),
            ("Atmos916Spatializer", DspNodeCategory::SpatialSurround, "Dolby Atmos 9.1.6 Object Panner with HRTF"),
            ("AmbisonicRadarSpatializer", DspNodeCategory::SpatialSurround, "Higher-Order Ambisonics HOA-5 Spherical Radar"),
            ("Auro3dSpatializer", DspNodeCategory::SpatialSurround, "Auro-3D 13.1 Tri-Layer Immersive Panner"),
            ("Nhk222Spatializer", DspNodeCategory::SpatialSurround, "NHK 22.2 Super Hi-Vision Multi-Channel Matrix"),
            ("BinauralBrirSpatializer", DspNodeCategory::SpatialSurround, "Binaural Room Impulse Response Convolver"),
            ("BinauralSpatializerNode", DspNodeCategory::SpatialSurround, "3D Binaural HRTF Spatializer with ITD & ILD"),
            ("RaytracedRoomReverb", DspNodeCategory::TimeSpace, "Geometric Raytraced Acoustic Room Simulator"),
            ("EffectDelay", DspNodeCategory::TimeSpace, "Stereo Ping-Pong Feedback Echo Delay"),
            ("EffectReverb", DspNodeCategory::TimeSpace, "Algorithmic Feedback Delay Network (FDN) Reverb"),
            ("FdnReverbNode", DspNodeCategory::TimeSpace, "High-Density Feedback Delay Network Reverb Core"),
            ("ConvolutionReverbNode", DspNodeCategory::TimeSpace, "Zero-Latency Partitioned Convolution Reverb"),
            ("HyperDimensionalTensorSpatializer", DspNodeCategory::SpatialSurround, "11-Dimensional Calabi-Yau Spatial Tensor Panner"),
            ("Holographic3dSpatialSoundfield", DspNodeCategory::SpatialSurround, "Holographic 3D Spherical Wavefield Synthesizer"),
            ("NonEuclideanHyperbolicReverb", DspNodeCategory::TimeSpace, "Hyperbolic Geometry Non-Euclidean Reverb Tank"),
            ("IsmShockwaveReverb", DspNodeCategory::TimeSpace, "Interstellar Medium Shockwave Convolution Reverb"),
            ("AcousticCloakingSpatializer", DspNodeCategory::SpatialSurround, "Acoustic Cloaking Phase Cancellation Spatializer"),
            ("AtmosProximityNode", DspNodeCategory::SpatialSurround, "Dolby Atmos Proximity Effect & Distance Attenuator"),
            ("AtmosSurroundNode", DspNodeCategory::SpatialSurround, "7.1.4 Immersive Atmos Surround Bed Renderer"),
            ("HoaSpatializer", DspNodeCategory::SpatialSurround, "4th/5th-Order High Order Ambisonic Spherical Panner"),
            ("Hoa5BinauralSpatializer", DspNodeCategory::SpatialSurround, "5th-Order Ambisonic Binaural Decoder with SOFA HRTF"),
            ("Mpegh3DSpatializer", DspNodeCategory::SpatialSurround, "MPEG-H 3D Immersive Audio Channel Bed Panner"),
            ("MpeghTrajectorySpatializer", DspNodeCategory::SpatialSurround, "3D Trajectory Bezier Spline Audio Spatializer"),
            ("WfsArraySpatializerNode", DspNodeCategory::SpatialSurround, "Wave Field Synthesis Multi-Transducer Array Engine"),
            ("DiffractivePropagationNode", DspNodeCategory::SpatialSurround, "Acoustic Obstacle Diffraction & Shadow Zone Simulator"),
            ("BinauralPannerNode", DspNodeCategory::SpatialSurround, "Precision Interaural Time & Level Difference 3D Panner"),
            ("WavefrontReflectionNode", DspNodeCategory::SpatialSurround, "Early Specular & Diffuse Wavefront Room Reflection Engine"),
            ("ReverbSpaceNode", DspNodeCategory::TimeSpace, "Parametric Multi-Room Acoustic Space Simulator"),
            ("MultibandSpatialNode", DspNodeCategory::SpatialSurround, "Tri-Band Frequency-Dependent Stereo & Spatial Panner"),
            ("MultitapDelayNode", DspNodeCategory::TimeSpace, "8-Tap Modulated Stereo Spatial Delay Line"),
            ("LimiterNode", DspNodeCategory::DynamicsMaster, "True-Peak Lookahead Mastering Brickwall Limiter"),
            ("EbuLoudnessRadar", DspNodeCategory::DynamicsMaster, "ITU-R BS.1770 / EBU R128 Loudness Radar Meter"),
            ("LufsMeterNode", DspNodeCategory::DynamicsMaster, "Multi-Scale Momentary & Integrated LUFS Meter"),
            ("MidSideNode", DspNodeCategory::DynamicsMaster, "Mid/Side Matrix Encoder & Stereo Width Imager"),
            ("StereoImager", DspNodeCategory::DynamicsMaster, "Phase-Safe Psychoacoustic Stereo Imager"),
            ("TpdfDitherNode", DspNodeCategory::DynamicsMaster, "Triangular PDF Dither & Noise Shaping Quantizer"),
            ("AdaptivePsychoacousticLoudness", DspNodeCategory::DynamicsMaster, "ISO 226 Equal Loudness Contour Balancing Engine"),
            ("DynamicStereoWidthNode", DspNodeCategory::DynamicsMaster, "Frequency-Dependent Mid/Side Dynamic Stereo Widener"),
            ("KSystemMeterNode", DspNodeCategory::DynamicsMaster, "Bob Katz K-12 / K-14 / K-20 Metering Bridge"),
            ("AuditoryRoughnessNode", DspNodeCategory::SpectralResynthesis, "Psychoacoustic Plomp-Levelt Sensory Dissonance Analyzer"),
            ("EqualLoudnessContourNode", DspNodeCategory::DynamicsMaster, "Fletcher-Munson Psychoacoustic Equal Loudness Compensator"),
            ("MasterLimiterRadarNode", DspNodeCategory::DynamicsMaster, "True-Peak Limiter with Integrated Loudness Radar"),
            ("MeterBridgeNode", DspNodeCategory::DynamicsMaster, "32-Channel VU, RMS, & True-Peak Mastering Meter Bridge"),
            ("MidSideFocuserNode", DspNodeCategory::DynamicsMaster, "Surgical Mid/Side Bass Mono & High-Side Air Focuser"),
            ("MultibandImagerNode", DspNodeCategory::DynamicsMaster, "4-Band Mastering Stereo Width & Phase Correlator"),
            ("OversampledLimiterNode", DspNodeCategory::DynamicsMaster, "16x Polyphase Oversampled Intersample Peak Limiter"),
            ("PolarPhaseCorrelatorNode", DspNodeCategory::DynamicsMaster, "Polar Goniometer Lissajous Phase Correlation Display"),
            ("ResonanceSuppressorNode", DspNodeCategory::DynamicsMaster, "Automatic Dynamic Harmonic Notch Resonance Suppressor"),
            ("SpectralAlignerNode", DspNodeCategory::SpectralResynthesis, "Multi-Track Phase & Spectral Coherence Auto-Aligner"),
            ("SpectralDebleedNode", DspNodeCategory::SpectralResynthesis, "Microphone Acoustic Bleed Elimination Matrix"),
            ("SpectralDeEsserNode", DspNodeCategory::SpectralResynthesis, "Dynamic Spectral Sibilance & Harshness Reducer"),
            ("SpectralFlatnessNode", DspNodeCategory::SpectralResynthesis, "Wiener Entropy Spectral Flatness & Tonal/Noise Analyzer"),
            ("SpectralGrainCloudNode", DspNodeCategory::SpectralResynthesis, "Spectral Domain Grain Scatter & Diffusion Engine"),
            ("SpectralMaskingNode", DspNodeCategory::SpectralResynthesis, "Psychoacoustic Simultaneous Spectral Masking Visualizer"),
            ("SpectralMorphNode", DspNodeCategory::SpectralResynthesis, "Latent FFT Magnitude & Phase Interpolation Morph Engine"),
            ("SpectralReshaperNode", DspNodeCategory::SpectralResynthesis, "Dynamic Spectral Envelope & Formant Reshaper"),
            ("SpectralResynthesisNode", DspNodeCategory::SpectralResynthesis, "Harmonic Additive Spectral Resynthesis Engine"),
            ("SpectralUnmaskerNode", DspNodeCategory::SpectralResynthesis, "AI-Driven Intelligent Sidechain Spectral Unmasker"),
            ("Spectrogram3DNode", DspNodeCategory::SpectralResynthesis, "3D Waterfall Real-Time Spectrogram & Sonogram Analyzer"),
            ("StereoVectorscopeNode", DspNodeCategory::SpectralResynthesis, "High-Speed Phosphor Stereophonic Vectorscope Analyzer"),
            ("StereoWidenerNode", DspNodeCategory::DynamicsMaster, "Haas Effect & Phase-Coherent Stereo Field Widener"),
            ("TapeFluxMasterNode", DspNodeCategory::DynamicsMaster, "Half-Inch Mastering Tape Saturation & Flux Density Engine"),
            ("DialogGatingNode", DspNodeCategory::DynamicsMaster, "ITU-R BS.1770 Dialogue-Aware Gating Loudness Engine"),
            ("SonarHydrophoneNode", DspNodeCategory::DynamicsMaster, "Underwater Acoustic Hydrophone & Thermal Layer Simulation"),
            ("BciEegDecoderNode", DspNodeCategory::NeuralAi, "10-20 EEG Brainwave Band Power & Alpha/Theta Decoder"),
            ("NeuroAffectiveEmotionalStateAnalyzer", DspNodeCategory::NeuralAi, "Russell Circumplex Valence & Arousal Emotion Analyzer"),
            ("NeuralImpulseResponseSynthesizer", DspNodeCategory::NeuralAi, "Latent Space Acoustic Impulse Response Generator"),
            ("SubcorticalBrainstemPitchTracker", DspNodeCategory::NeuralAi, "Auditory Nerve Frequency-Following Response Pitch Tracker"),
            ("MentalImageryPatternClassifier", DspNodeCategory::NeuralAi, "Motor/Auditory Imagery Neural Intention Classifier"),
            ("BiometricHrvTempoSync", DspNodeCategory::NeuralAi, "Photoplethysmography Heart Rate Variability Tempo Sync"),
            ("NeuroCognitiveFatigueDetector", DspNodeCategory::NeuralAi, "Listening Fatigue & Auditory Habituation Detector"),
            ("NeuroAestheticHarmonyScorer", DspNodeCategory::NeuralAi, "Computational Neuro-Aesthetic Musical Harmony Scorer"),
            ("SubsensoryTactileHapticTransducer", DspNodeCategory::NeuralAi, "Vibrotactile Sub-Bass Haptic Transducer Driver"),
            ("NeuralChoirFormantNode", DspNodeCategory::NeuralAi, "Multi-Voice Neural Formant Choir & Vowel Morph Engine"),
            ("NeuralDereverbNode", DspNodeCategory::NeuralAi, "Deep Learning Direct-to-Reverberant Ratio Blind De-Reverb"),
            ("NeuralInpaintNode", DspNodeCategory::NeuralAi, "Neural Audio Inpainting & Dropout Reconstruction Engine"),
            ("NeuralPhonemeNode", DspNodeCategory::NeuralAi, "International Phonetic Alphabet Differentiable Phoneme Synth"),
            ("NeuralRadianceNode", DspNodeCategory::NeuralAi, "Neural Acoustic Radiance Field (NeRF) 3D Room Acoustic Field"),
            ("NeuralSpeechToSingingNode", DspNodeCategory::NeuralAi, "Speech-to-Singing AI Prosody & Intonation Transformer"),
            ("NeuralTimbreMorphNode", DspNodeCategory::NeuralAi, "Non-Linear Latent Timbre Vector Interpolator"),
            ("NeuralTimbreNode", DspNodeCategory::NeuralAi, "Latent Timbre Space Feature Extractor & Profiler"),
            ("NeuralVocalStylizerNode", DspNodeCategory::NeuralAi, "Vocal Timbre Style Transfer & Character Transformer"),
            ("NeuralVocoderMorphNode", DspNodeCategory::NeuralAi, "Neural HiFi-GAN / BigVGAN Vocoder Latent Morpher"),
            ("NeuralQuantumAnnealerTopologicalSort", DspNodeCategory::NeuralAi, "Simulated Quantum Annealing Audio Graph Optimizer"),
            ("GainNode", DspNodeCategory::Utility, "Precision Unity Gain & Polarity Inverter"),
            ("ChromaticTunerNode", DspNodeCategory::Utility, "Strobe Chromatic Instrument Pitch Tuner"),
            ("SampleSlicerNode", DspNodeCategory::SamplerSlicer, "Transient Beat Slicer & Loop Fragmenter"),
            ("LoopSlicerNode", DspNodeCategory::SamplerSlicer, "Beat-Synchronized Grid Loop Slicer"),
            ("HardwareCvNode", DspNodeCategory::Utility, "Eurorack Modular Control Voltage & Gate Interface"),
            ("ClapPluginHostNode", DspNodeCategory::Utility, "CLAP/VST3 Audio Plugin Host Adapter"),
            ("Vst3HostNode", DspNodeCategory::Utility, "VST3 Plugin Host with Embedded GUI & Automation"),
            ("PushControllerDriverNode", DspNodeCategory::Utility, "Ableton Push 2/3 High-Speed Display & RGB Pad Driver"),
            ("LaunchpadDriverNode", DspNodeCategory::Utility, "Novation Launchpad Pro RGB Session & Step Sequencer Driver"),
            ("KompleteKontrolDriverNode", DspNodeCategory::Utility, "Native Instruments NKS Light Guide & Encoder Driver"),
            ("McuHardwareDriverNode", DspNodeCategory::Utility, "Mackie Control Universal 100mm Motorized Fader Surface"),
            ("OscControlMapperNode", DspNodeCategory::Utility, "Bidirectional Open Sound Control (OSC) Network Gateway"),
            ("MultiTrackAudioRouterNode", DspNodeCategory::Utility, "64x64 Low-Latency Cross-Point Matrix Audio Router"),
            ("WasmDspRuntimeNode", DspNodeCategory::Utility, "Sandboxed WebAssembly Wasm DSP Custom Kernel Host"),
            ("HardwareMidiClockJitterFilterNode", DspNodeCategory::Utility, "Sub-Microsecond Phase-Locked Loop MIDI Clock Sync"),
            ("PluginSandboxScannerNode", DspNodeCategory::Utility, "Out-of-Process Crash-Proof Plugin Sandboxing & Scanner"),
            ("PluginParamAutomapEngineNode", DspNodeCategory::Utility, "AI Semantic Parameter Auto-Mapping & Grouping Engine"),
            ("CvGateSignalGeneratorNode", DspNodeCategory::Utility, "Modular Synth 1V/Oct CV Pitch & Gate Pulse Generator"),
            ("DinSync24PulseGeneratorNode", DspNodeCategory::Utility, "Roland DIN Sync 24 PPQN Hardware Clock Generator"),
            ("BleMidiControllerDriverNode", DspNodeCategory::Utility, "Bluetooth Low Energy (BLE-MIDI) Low-Latency Driver"),
            ("SupercriticalFluidNoiseNode", DspNodeCategory::Utility, "Supercritical Phase-Transition Acoustic Noise Generator"),
            ("SonoluminescenceSonifierNode", DspNodeCategory::Utility, "Ultrasonic Acoustic Cavitation Bubble Sonifier"),
            ("GravitationalWaveChirpNode", DspNodeCategory::Utility, "Binary Black Hole Inspiral Gravitational Wave Chirp"),
            ("CasimirVacuumNoiseNode", DspNodeCategory::Utility, "Quantum Vacuum Zero-Point Energy Fluctuation Noise"),
            ("RelativisticDopplerShiftNode", DspNodeCategory::Utility, "Lorentz Relativistic Velocity Doppler Time Dilation"),
            ("StochasticQuantumDecoherenceNoise", DspNodeCategory::Utility, "Quantum Decoherence Lindblad Superoperator Noise"),
            ("QuantumTeleportationAudioBufferBus", DspNodeCategory::Utility, "Zero-Latency Entangled State Audio Bus Interlink"),
            ("QuantumPhaseEstimationPitchTracker", DspNodeCategory::Utility, "Quantum Phase Estimation Sub-Cent Pitch Tracker"),
            ("AtmosphericDensityNode", DspNodeCategory::SpatialSurround, "Atmospheric Gas Composition & Speed of Sound Simulator"),
            ("QuantumDotTransducerNode", DspNodeCategory::Utility, "Nanoscale Quantum Dot Optical-to-Audio Transducer"),
            ("AcousticLevitationTrapNode", DspNodeCategory::Utility, "Gor'kov Ultrasound Acoustic Radiation Pressure Trap"),
            ("MathAdd", DspNodeCategory::Utility, "Signal Arithmetic Sum & Offset Scale"),
            ("MathMult", DspNodeCategory::Utility, "Signal Arithmetic Product & Modulation Scale"),
            ("VCA", DspNodeCategory::Utility, "Voltage-Controlled Amplifier Gain Block"),
            ("WsolaTimeStretcher", DspNodeCategory::Utility, "WSOLA Pitch-Preserving Time Stretch Engine"),
            ("StemSeparator", DspNodeCategory::Utility, "Neural 4-Stem Audio Source Separation Matrix"),
            ("ElasticWarpEngine", DspNodeCategory::Utility, "Real-time Elastic Audio Warp & Transient Pinning"),
            ("Oversampler", DspNodeCategory::Utility, "Polyphase Anti-Aliasing Oversampling Filter"),
            ("PassthroughNode", DspNodeCategory::Utility, "Zero-Latency Transparent Buffer Relay"),
        ]
    }

    /// Canonical normalization mapping common aliases and case variations to registered type names.
    pub fn normalize_type_name(name: &str) -> Option<&'static str> {
        let clean: String = name.chars().filter(|c| c.is_alphanumeric()).map(|c| c.to_ascii_lowercase()).collect();
        match clean.as_str() {

            "sine" => Some("OscSine"),
            "sineosc" => Some("OscSine"),
            "sineoscillator" => Some("OscSine"),
            "saw" => Some("OscSaw"),
            "sawtooth" => Some("OscSaw"),
            "sawosc" => Some("OscSaw"),
            "pulse" => Some("OscPulse"),
            "pwm" => Some("OscPulse"),
            "pulseosc" => Some("OscPulse"),
            "triangle" => Some("OscTriangle"),
            "tri" => Some("OscTriangle"),
            "wavetable" => Some("OscWavetable"),
            "wavetableoscillator" => Some("OscWavetable"),
            "polywavetable" => Some("SimdPolyWavetableOscillator"),
            "simdwavetable" => Some("SimdPolyWavetableOscillator"),
            "tubedrive" => Some("TubeSaturationNode"),
            "tubesaturation" => Some("TubeSaturationNode"),
            "tube" => Some("TubeSaturationNode"),
            "svffilter" => Some("FilterSVF"),
            "svf" => Some("FilterSVF"),
            "ladder" => Some("FilterLadder"),
            "ladderfilter" => Some("FilterLadder"),
            "moogladder" => Some("FilterLadder"),
            "tapedelay" => Some("EffectDelay"),
            "delay" => Some("EffectDelay"),
            "echodelay" => Some("EffectDelay"),
            "pingpong" => Some("EffectDelay"),
            "convreverb" => Some("ConvolutionReverbNode"),
            "convolutionreverb" => Some("ConvolutionReverbNode"),
            "reverb" => Some("EffectReverb"),
            "fdnreverb" => Some("FdnReverbNode"),
            "compressor" => Some("CompressorNode"),
            "vcaprocessor" => Some("CompressorNode"),
            "limiter" => Some("LimiterNode"),
            "brickwall" => Some("LimiterNode"),
            "brickwalllimiter" => Some("LimiterNode"),
            "aether" => Some("AetherSynth"),
            "pluck" => Some("PluckSynth"),
            "karplus" => Some("PluckSynth"),
            "subsynth" => Some("CyberpunkSubSynth"),
            "cyberpunk" => Some("CyberpunkSubSynth"),
            "ambientpad" => Some("AtmosphericPadSynth"),
            "pad" => Some("AtmosphericPadSynth"),
            "drums" => Some("DrumMachineDevice"),
            "drummachine" => Some("DrumMachineDevice"),
            "drumvoice" => Some("AnalogDrumVoice"),
            "piano" => Some("GrandPianoModel"),
            "grandpiano" => Some("GrandPianoModel"),
            "electricpiano" => Some("ElectricPianoModel"),
            "rhodes" => Some("ElectricPianoModel"),
            "wurlitzer" => Some("ElectricPianoModel"),
            "clavinet" => Some("ClavinetModel"),
            "gamelan" => Some("GamelanGender"),
            "gender" => Some("GamelanGender"),
            "hurdygurdy" => Some("HurdyGurdy"),
            "sitar" => Some("SitarModel"),
            "shakuhachi" => Some("ShakuhachiModel"),
            "pipeorgan" => Some("PipeOrganModel"),
            "organ" => Some("PipeOrganModel"),
            "cathedralorgan" => Some("PipeOrganModel"),
            "glassarmonica" => Some("GlassArmonica"),
            "violin" => Some("BowedStringModel"),
            "cello" => Some("BowedStringModel"),
            "bowedstring" => Some("BowedStringModel"),
            "steelpan" => Some("SteelpanModel"),
            "steeldrum" => Some("SteelpanModel"),
            "ney" => Some("TurkishNeyModel"),
            "turkishney" => Some("TurkishNeyModel"),
            "brass" => Some("WaveguideBrassModel"),
            "horn" => Some("WaveguideBrassModel"),
            "woodwind" => Some("WoodwindJetModel"),
            "flute" => Some("WoodwindJetModel"),
            "springreverb" => Some("SpringLatticeNode"),
            "springlattice" => Some("SpringLatticeNode"),
            "platereverb" => Some("PlateTankNode"),
            "platetank" => Some("PlateTankNode"),
            "hammond" => Some("TonewheelOrganModel"),
            "tonewheel" => Some("TonewheelOrganModel"),
            "b3" => Some("TonewheelOrganModel"),
            "vocaltract" => Some("VocalTractNode"),
            "chorus" => Some("EffectChorus"),
            "flanger" => Some("EffectFlanger"),
            "phaser" => Some("EffectPhaser"),
            "ringmod" => Some("RingModulator"),
            "eq" => Some("ParametricEqNode"),
            "parametriceq" => Some("ParametricEqNode"),
            "biquad" => Some("FilterBiquad"),
            "deesser" => Some("DeesserNode"),
            "noisegate" => Some("NoiseGateNode"),
            "gate" => Some("NoiseGateNode"),
            "distortion" => Some("DistortionNode"),
            "tapesaturation" => Some("TapeSaturationNode"),
            "console" => Some("ConsoleEmulationNode"),
            "bitcrusher" => Some("BitcrusherNode"),
            "wavefolder" => Some("WavefolderNode"),
            "exciter" => Some("HarmonicExciterNode"),
            "atmos" => Some("Atmos916Spatializer"),
            "binaural" => Some("BinauralSpatializerNode"),
            "lufs" => Some("LufsMeterNode"),
            "eburadar" => Some("EbuLoudnessRadar"),
            "midside" => Some("MidSideNode"),
            "gain" => Some("GainNode"),
            "tuner" => Some("ChromaticTunerNode"),
            "vca" => Some("VCA"),
            "stretcher" => Some("WsolaTimeStretcher"),
            "stems" => Some("StemSeparator"),
            "stemseparator" => Some("StemSeparator"),
            "warp" => Some("ElasticWarpEngine"),
            "oversampler" => Some("Oversampler"),
            "percussionmembranenode" => Some("MembranePercussionModel"),
            "percussionmembrane" => Some("MembranePercussionModel"),
            "oscsine" => Some("OscSine"),
            "oscsaw" => Some("OscSaw"),
            "oscpulse" => Some("OscPulse"),
            "osctriangle" => Some("OscTriangle"),
            "oscwavetable" => Some("OscWavetable"),
            "simdpolywavetableoscillator" => Some("SimdPolyWavetableOscillator"),
            "granularsynthnode" => Some("GranularSynthNode"),
            "neuralwavetable" => Some("NeuralWavetable"),
            "plasmaarcsynthesizer" => Some("PlasmaArcSynthesizer"),
            "noisegen" => Some("NoiseGen"),
            "aethersynth" => Some("AetherSynth"),
            "plucksynth" => Some("PluckSynth"),
            "cyberpunksubsynth" => Some("CyberpunkSubSynth"),
            "atmosphericpadsynth" => Some("AtmosphericPadSynth"),
            "glitchpercussionsynth" => Some("GlitchPercussionSynth"),
            "glitchaethermachine" => Some("GlitchAetherMachine"),
            "fmoperatorpair" => Some("FmOperatorPair"),
            "fmmatrixsynthesizer" => Some("FmMatrixSynthesizer"),
            "analogdrumvoice" => Some("AnalogDrumVoice"),
            "drummachinedevice" => Some("DrumMachineDevice"),
            "quantumstatevectoroscillator" => Some("QuantumStateVectorOscillator"),
            "quantumharmonicoscillatorvoice" => Some("QuantumHarmonicOscillatorVoice"),
            "navierstokesfluidnode" => Some("NavierStokesFluidNode"),
            "molecularvibrationresonator" => Some("MolecularVibrationResonator"),
            "closedloopneurofeedbackoscillator" => Some("ClosedLoopNeuroFeedbackOscillator"),
            "brainwaveentrainmentbinauralbeat" => Some("BrainwaveEntrainmentBinauralBeat"),
            "fusionresonancesynth" => Some("FusionResonanceSynth"),
            "subharmonicsynthnode" => Some("SubharmonicSynthNode"),
            "samplerdevice" => Some("SamplerDevice"),
            "multisamplernode" => Some("MultiSamplerNode"),
            "singingsynthesisnode" => Some("SingingSynthesisNode"),
            "spectralneuralresynthesizernode" => Some("SpectralNeuralResynthesizerNode"),
            "neuralaudiorepairnode" => Some("NeuralAudioRepairNode"),
            "filtersvf" => Some("FilterSVF"),
            "filterladder" => Some("FilterLadder"),
            "filterladder8" => Some("FilterLadder8"),
            "filtercomb" => Some("FilterComb"),
            "filterbiquad" => Some("FilterBiquad"),
            "dcblockfilter" => Some("DcBlockFilter"),
            "lowcutfilter" => Some("LowCutFilter"),
            "highcutfilter" => Some("HighCutFilter"),
            "parametriceqnode" => Some("ParametricEqNode"),
            "multichannelspectralequalizernode" => Some("MultiChannelSpectralEqualizerNode"),
            "formantfilternode" => Some("FormantFilterNode"),
            "modalfilternode" => Some("ModalFilterNode"),
            "hornreflectionfilter" => Some("HornReflectionFilter"),
            "automateddynamiceqnode" => Some("AutomatedDynamicEqNode"),
            "subharmonicquantumtunnelingfilter" => Some("SubharmonicQuantumTunnelingFilter"),
            "metamaterialrefractionfilter" => Some("MetamaterialRefractionFilter"),
            "spatialacoustichologramfilter" => Some("SpatialAcousticHologramFilter"),
            "linearphasecrossovernode" => Some("LinearPhaseCrossoverNode"),
            "spectralmatchingeqnode" => Some("SpectralMatchingEqNode"),
            "spectraltiltnode" => Some("SpectralTiltNode"),
            "combresonatornode" => Some("CombResonatorNode"),
            "envadsr" => Some("EnvADSR"),
            "osclfo" => Some("OscLFO"),
            "envelopefollowernode" => Some("EnvelopeFollowerNode"),
            "modulationmatrix" => Some("ModulationMatrix"),
            "pitchshifternode" => Some("PitchShifterNode"),
            "autowahnode" => Some("AutoWahNode"),
            "autotunenode" => Some("AutoTuneNode"),
            "vocalpitchformantcorrectornode" => Some("VocalPitchFormantCorrectorNode"),
            "glitchbuffernode" => Some("GlitchBufferNode"),
            "effectchorus" => Some("EffectChorus"),
            "effectflanger" => Some("EffectFlanger"),
            "effectphaser" => Some("EffectPhaser"),
            "ringmodulator" => Some("RingModulator"),
            "frequencyshifter" => Some("FrequencyShifter"),
            "chaoticfractalattractormodulator" => Some("ChaoticFractalAttractorModulator"),
            "quantumentanglementmodulationrouting" => Some("QuantumEntanglementModulationRouting"),
            "mhdplasmawavemodulator" => Some("MhdPlasmaWaveModulator"),
            "bbdchorusnode" => Some("BbdChorusNode"),
            "throughzeroflangernode" => Some("ThroughZeroFlangerNode"),
            "tapeflutternode" => Some("TapeFlutterNode"),
            "granularfreezenode" => Some("GranularFreezeNode"),
            "granularpitchshifter" => Some("GranularPitchShifter"),
            "pitchcorrectornode" => Some("PitchCorrectorNode"),
            "vocodermatrixnode" => Some("VocoderMatrixNode"),
            "gamelangender" => Some("GamelanGender"),
            "sitarmodel" => Some("SitarModel"),
            "grandpianomodel" => Some("GrandPianoModel"),
            "shakuhachimodel" => Some("ShakuhachiModel"),
            "pipeorganmodel" => Some("PipeOrganModel"),
            "clavinetmodel" => Some("ClavinetModel"),
            "electricpianomodel" => Some("ElectricPianoModel"),
            "bowedstringmodel" => Some("BowedStringModel"),
            "steelpanmodel" => Some("SteelpanModel"),
            "turkishneymodel" => Some("TurkishNeyModel"),
            "waveguidebrassmodel" => Some("WaveguideBrassModel"),
            "woodwindjetmodel" => Some("WoodwindJetModel"),
            "springlatticenode" => Some("SpringLatticeNode"),
            "platetanknode" => Some("PlateTankNode"),
            "tonewheelorganmodel" => Some("TonewheelOrganModel"),
            "vocaltractnode" => Some("VocalTractNode"),
            "bambooresonatorbank" => Some("BambooResonatorBank"),
            "mizrabpluckmodel" => Some("MizrabPluckModel"),
            "jawaribridgemodel" => Some("JawariBridgeModel"),
            "soundboardbridgemodel" => Some("SoundboardBridgeModel"),
            "tineresonatormodel" => Some("TineResonatorModel"),
            "trompettechienmodel" => Some("TrompetteChienModel"),
            "windchestaerodynamicsmodel" => Some("WindchestAerodynamicsModel"),
            "airreedacousticmodel" => Some("AirReedAcousticModel"),
            "digitalwaveguidenode" => Some("DigitalWaveguideNode"),
            "waveguidemesh2d" => Some("WaveguideMesh2D"),
            "rotaryspeakermodel" => Some("RotarySpeakerModel"),
            "struckidiophoneresonatornode" => Some("StruckIdiophoneResonatorNode"),
            "membranepercussionmodel" => Some("MembranePercussionModel"),
            "glottalpulsenode" => Some("GlottalPulseNode"),
            "hammerstrikemodel" => Some("HammerStrikeModel"),
            "toneholegridmodel" => Some("ToneholeGridModel"),
            "frictionmodel" => Some("FrictionModel"),
            "modalresonator" => Some("ModalResonator"),
            "dulcimercimbalommodel" => Some("DulcimerCimbalomModel"),
            "mbirakalimbamodel" => Some("MbiraKalimbaModel"),
            "membraneplatemodel" => Some("MembranePlateModel"),
            "membraneresonatornode" => Some("MembraneResonatorNode"),
            "sympatheticcouplingmodel" => Some("SympatheticCouplingModel"),
            "compressornode" => Some("CompressorNode"),
            "multibandcompressornode" => Some("MultibandCompressorNode"),
            "multibanddynamicsprocessor" => Some("MultibandDynamicsProcessor"),
            "opticalcompressornode" => Some("OpticalCompressorNode"),
            "noisegatenode" => Some("NoiseGateNode"),
            "deessernode" => Some("DeesserNode"),
            "neuraltransientshapernode" => Some("NeuralTransientShaperNode"),
            "neuralbassgeneratornode" => Some("NeuralBassGeneratorNode"),
            "multibandclippernode" => Some("MultibandClipperNode"),
            "multibanddecompressornode" => Some("MultibandDecompressorNode"),
            "multibandexpandernode" => Some("MultibandExpanderNode"),
            "multibandsaturatornode" => Some("MultibandSaturatorNode"),
            "upwardcompressornode" => Some("UpwardCompressorNode"),
            "varimumasternode" => Some("VariMuMasterNode"),
            "dynamiccrestshapernode" => Some("DynamicCrestShaperNode"),
            "paralleltransientsaturatornode" => Some("ParallelTransientSaturatorNode"),
            "transientclippernode" => Some("TransientClipperNode"),
            "transientdeclickernode" => Some("TransientDeclickerNode"),
            "transientdesignernode" => Some("TransientDesignerNode"),
            "transientgatenode" => Some("TransientGateNode"),
            "transientreconstructornode" => Some("TransientReconstructorNode"),
            "transientshapernode" => Some("TransientShaperNode"),
            "transientunwrappernode" => Some("TransientUnwrapperNode"),
            "distortionnode" => Some("DistortionNode"),
            "tapesaturationnode" => Some("TapeSaturationNode"),
            "tubesaturationnode" => Some("TubeSaturationNode"),
            "tubebiasnode" => Some("TubeBiasNode"),
            "consoleemulationnode" => Some("ConsoleEmulationNode"),
            "bitcrushernode" => Some("BitcrusherNode"),
            "wavefoldernode" => Some("WavefolderNode"),
            "harmonicexciternode" => Some("HarmonicExciterNode"),
            "neuralharmonicexciternode" => Some("NeuralHarmonicExciterNode"),
            "atmos916spatializer" => Some("Atmos916Spatializer"),
            "ambisonicradarspatializer" => Some("AmbisonicRadarSpatializer"),
            "auro3dspatializer" => Some("Auro3dSpatializer"),
            "nhk222spatializer" => Some("Nhk222Spatializer"),
            "binauralbrirspatializer" => Some("BinauralBrirSpatializer"),
            "binauralspatializernode" => Some("BinauralSpatializerNode"),
            "raytracedroomreverb" => Some("RaytracedRoomReverb"),
            "effectdelay" => Some("EffectDelay"),
            "effectreverb" => Some("EffectReverb"),
            "fdnreverbnode" => Some("FdnReverbNode"),
            "convolutionreverbnode" => Some("ConvolutionReverbNode"),
            "hyperdimensionaltensorspatializer" => Some("HyperDimensionalTensorSpatializer"),
            "holographic3dspatialsoundfield" => Some("Holographic3dSpatialSoundfield"),
            "noneuclideanhyperbolicreverb" => Some("NonEuclideanHyperbolicReverb"),
            "ismshockwavereverb" => Some("IsmShockwaveReverb"),
            "acousticcloakingspatializer" => Some("AcousticCloakingSpatializer"),
            "atmosproximitynode" => Some("AtmosProximityNode"),
            "atmossurroundnode" => Some("AtmosSurroundNode"),
            "hoaspatializer" => Some("HoaSpatializer"),
            "hoa5binauralspatializer" => Some("Hoa5BinauralSpatializer"),
            "mpegh3dspatializer" => Some("Mpegh3DSpatializer"),
            "mpeghtrajectoryspatializer" => Some("MpeghTrajectorySpatializer"),
            "wfsarrayspatializernode" => Some("WfsArraySpatializerNode"),
            "diffractivepropagationnode" => Some("DiffractivePropagationNode"),
            "binauralpannernode" => Some("BinauralPannerNode"),
            "wavefrontreflectionnode" => Some("WavefrontReflectionNode"),
            "reverbspacenode" => Some("ReverbSpaceNode"),
            "multibandspatialnode" => Some("MultibandSpatialNode"),
            "multitapdelaynode" => Some("MultitapDelayNode"),
            "limiternode" => Some("LimiterNode"),
            "ebuloudnessradar" => Some("EbuLoudnessRadar"),
            "lufsmeternode" => Some("LufsMeterNode"),
            "midsidenode" => Some("MidSideNode"),
            "stereoimager" => Some("StereoImager"),
            "tpdfdithernode" => Some("TpdfDitherNode"),
            "adaptivepsychoacousticloudness" => Some("AdaptivePsychoacousticLoudness"),
            "dynamicstereowidthnode" => Some("DynamicStereoWidthNode"),
            "ksystemmeternode" => Some("KSystemMeterNode"),
            "auditoryroughnessnode" => Some("AuditoryRoughnessNode"),
            "equalloudnesscontournode" => Some("EqualLoudnessContourNode"),
            "masterlimiterradarnode" => Some("MasterLimiterRadarNode"),
            "meterbridgenode" => Some("MeterBridgeNode"),
            "midsidefocusernode" => Some("MidSideFocuserNode"),
            "multibandimagernode" => Some("MultibandImagerNode"),
            "oversampledlimiternode" => Some("OversampledLimiterNode"),
            "polarphasecorrelatornode" => Some("PolarPhaseCorrelatorNode"),
            "resonancesuppressornode" => Some("ResonanceSuppressorNode"),
            "spectralalignernode" => Some("SpectralAlignerNode"),
            "spectraldebleednode" => Some("SpectralDebleedNode"),
            "spectraldeessernode" => Some("SpectralDeEsserNode"),
            "spectralflatnessnode" => Some("SpectralFlatnessNode"),
            "spectralgraincloudnode" => Some("SpectralGrainCloudNode"),
            "spectralmaskingnode" => Some("SpectralMaskingNode"),
            "spectralmorphnode" => Some("SpectralMorphNode"),
            "spectralreshapernode" => Some("SpectralReshaperNode"),
            "spectralresynthesisnode" => Some("SpectralResynthesisNode"),
            "spectralunmaskernode" => Some("SpectralUnmaskerNode"),
            "spectrogram3dnode" => Some("Spectrogram3DNode"),
            "stereovectorscopenode" => Some("StereoVectorscopeNode"),
            "stereowidenernode" => Some("StereoWidenerNode"),
            "tapefluxmasternode" => Some("TapeFluxMasterNode"),
            "dialoggatingnode" => Some("DialogGatingNode"),
            "sonarhydrophonenode" => Some("SonarHydrophoneNode"),
            "bcieegdecodernode" => Some("BciEegDecoderNode"),
            "neuroaffectiveemotionalstateanalyzer" => Some("NeuroAffectiveEmotionalStateAnalyzer"),
            "neuralimpulseresponsesynthesizer" => Some("NeuralImpulseResponseSynthesizer"),
            "subcorticalbrainstempitchtracker" => Some("SubcorticalBrainstemPitchTracker"),
            "mentalimagerypatternclassifier" => Some("MentalImageryPatternClassifier"),
            "biometrichrvtemposync" => Some("BiometricHrvTempoSync"),
            "neurocognitivefatiguedetector" => Some("NeuroCognitiveFatigueDetector"),
            "neuroaestheticharmonyscorer" => Some("NeuroAestheticHarmonyScorer"),
            "subsensorytactilehaptictransducer" => Some("SubsensoryTactileHapticTransducer"),
            "neuralchoirformantnode" => Some("NeuralChoirFormantNode"),
            "neuraldereverbnode" => Some("NeuralDereverbNode"),
            "neuralinpaintnode" => Some("NeuralInpaintNode"),
            "neuralphonemenode" => Some("NeuralPhonemeNode"),
            "neuralradiancenode" => Some("NeuralRadianceNode"),
            "neuralspeechtosingingnode" => Some("NeuralSpeechToSingingNode"),
            "neuraltimbremorphnode" => Some("NeuralTimbreMorphNode"),
            "neuraltimbrenode" => Some("NeuralTimbreNode"),
            "neuralvocalstylizernode" => Some("NeuralVocalStylizerNode"),
            "neuralvocodermorphnode" => Some("NeuralVocoderMorphNode"),
            "neuralquantumannealertopologicalsort" => Some("NeuralQuantumAnnealerTopologicalSort"),
            "gainnode" => Some("GainNode"),
            "chromatictunernode" => Some("ChromaticTunerNode"),
            "sampleslicernode" => Some("SampleSlicerNode"),
            "loopslicernode" => Some("LoopSlicerNode"),
            "hardwarecvnode" => Some("HardwareCvNode"),
            "clappluginhostnode" => Some("ClapPluginHostNode"),
            "vst3hostnode" => Some("Vst3HostNode"),
            "pushcontrollerdrivernode" => Some("PushControllerDriverNode"),
            "launchpaddrivernode" => Some("LaunchpadDriverNode"),
            "kompletekontroldrivernode" => Some("KompleteKontrolDriverNode"),
            "mcuhardwaredrivernode" => Some("McuHardwareDriverNode"),
            "osccontrolmappernode" => Some("OscControlMapperNode"),
            "multitrackaudiorouternode" => Some("MultiTrackAudioRouterNode"),
            "wasmdspruntimenode" => Some("WasmDspRuntimeNode"),
            "hardwaremidiclockjitterfilternode" => Some("HardwareMidiClockJitterFilterNode"),
            "pluginsandboxscannernode" => Some("PluginSandboxScannerNode"),
            "pluginparamautomapenginenode" => Some("PluginParamAutomapEngineNode"),
            "cvgatesignalgeneratornode" => Some("CvGateSignalGeneratorNode"),
            "dinsync24pulsegeneratornode" => Some("DinSync24PulseGeneratorNode"),
            "blemidicontrollerdrivernode" => Some("BleMidiControllerDriverNode"),
            "supercriticalfluidnoisenode" => Some("SupercriticalFluidNoiseNode"),
            "sonoluminescencesonifiernode" => Some("SonoluminescenceSonifierNode"),
            "gravitationalwavechirpnode" => Some("GravitationalWaveChirpNode"),
            "casimirvacuumnoisenode" => Some("CasimirVacuumNoiseNode"),
            "relativisticdopplershiftnode" => Some("RelativisticDopplerShiftNode"),
            "stochasticquantumdecoherencenoise" => Some("StochasticQuantumDecoherenceNoise"),
            "quantumteleportationaudiobufferbus" => Some("QuantumTeleportationAudioBufferBus"),
            "quantumphaseestimationpitchtracker" => Some("QuantumPhaseEstimationPitchTracker"),
            "atmosphericdensitynode" => Some("AtmosphericDensityNode"),
            "quantumdottransducernode" => Some("QuantumDotTransducerNode"),
            "acousticlevitationtrapnode" => Some("AcousticLevitationTrapNode"),
            "mathadd" => Some("MathAdd"),
            "mathmult" => Some("MathMult"),
            "wsolatimestretcher" => Some("WsolaTimeStretcher"),
            "elasticwarpengine" => Some("ElasticWarpEngine"),
            "passthroughnode" => Some("PassthroughNode"),
            _ => None,
        }
    }

    /// Query list of all registered node types in a specific category.
    pub fn nodes_by_category(category: DspNodeCategory) -> Vec<(&'static str, &'static str)> {
        Self::inventory()
            .into_iter()
            .filter(|(_, c, _)| *c == category)
            .map(|(name, _, desc)| (name, desc))
            .collect()
    }

    /// Search registered nodes by keyword in name or description.
    pub fn search_nodes(query: &str) -> Vec<(&'static str, DspNodeCategory, &'static str)> {
        let q = query.to_lowercase();
        Self::inventory()
            .into_iter()
            .filter(|(name, _, desc)| name.to_lowercase().contains(&q) || desc.to_lowercase().contains(&q))
            .collect()
    }

    /// Instantiate a `DspNodeUi` object with default parameters for any node type in the inventory.
    pub fn create_node_ui(type_name: &str) -> Option<Box<dyn DspNodeUi>> {
        let target_name = if Self::inventory().iter().any(|(n, _, _)| *n == type_name) {
            type_name
        } else {
            Self::normalize_type_name(type_name).unwrap_or(type_name)
        };
        let node: Box<dyn DspNodeUi> = match target_name {

            "OscSine" => Box::new(
                GenericDspNodeUi::new("OscSine", "Pure Sine Wave Oscillator", DspNodeCategory::Oscillator)
                    .with_param(DspParamDescriptor::new_log("freq", "Frequency", 20.0, 20000.0, 440.0, "Hz", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("phase", "Phase Offset", 0.0, 360.0, 0.0, "deg", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("gain", "Output Gain", 0.0, 1.0, 0.8, "%", (0, 229, 255)))
            ),
            "OscSaw" => Box::new(
                GenericDspNodeUi::new("OscSaw", "Band-limited Sawtooth Oscillator", DspNodeCategory::Oscillator)
                    .with_param(DspParamDescriptor::new_log("freq", "Frequency", 20.0, 20000.0, 440.0, "Hz", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("phase", "Phase Offset", 0.0, 360.0, 0.0, "deg", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("gain", "Output Gain", 0.0, 1.0, 0.8, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_int("bandlimit", "Bandlimit Order", 1, 8, 4, "", (153, 102, 255)))
            ),
            "OscPulse" => Box::new(
                GenericDspNodeUi::new("OscPulse", "Pulse Width Modulation Oscillator", DspNodeCategory::Oscillator)
                    .with_param(DspParamDescriptor::new_log("freq", "Frequency", 20.0, 20000.0, 440.0, "Hz", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("pulse_width", "Pulse Width", 0.01, 0.99, 0.5, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("pwm_rate", "PWM LFO Rate", 0.1, 20.0, 1.0, "Hz", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_linear("gain", "Output Gain", 0.0, 1.0, 0.8, "%", (0, 229, 255)))
            ),
            "OscTriangle" => Box::new(
                GenericDspNodeUi::new("OscTriangle", "Anti-aliased Triangle Generator", DspNodeCategory::Oscillator)
                    .with_param(DspParamDescriptor::new_log("freq", "Frequency", 20.0, 20000.0, 440.0, "Hz", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("symmetry", "Symmetry Ramp", 0.01, 0.99, 0.5, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("gain", "Output Gain", 0.0, 1.0, 0.8, "%", (0, 229, 255)))
            ),
            "OscWavetable" => Box::new(
                GenericDspNodeUi::new("OscWavetable", "SIMD Wavetable Morphing Synthesizer", DspNodeCategory::Oscillator)
                    .with_param(DspParamDescriptor::new_log("freq", "Frequency", 20.0, 20000.0, 440.0, "Hz", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("morph_pos", "Morph Position", 0.0, 1.0, 0.0, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_enum("table", "Table Bank", vec!["Basic Shapes".into(), "Harmonics".into(), "Formants".into(), "Metallic".into()], 0, (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("gain", "Output Gain", 0.0, 1.0, 0.8, "%", (0, 229, 255)))
            ),
            "SimdPolyWavetableOscillator" => Box::new(
                GenericDspNodeUi::new("SimdPolyWavetableOscillator", "AVX2/NEON Polyphonic Wavetable Voice Bank", DspNodeCategory::Oscillator)
                    .with_param(DspParamDescriptor::new_linear("morph_pos", "Morph Position", 0.0, 1.0, 0.0, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("unison_detune", "Unison Detune", 0.0, 50.0, 8.0, "cents", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_int("unison_voices", "Voice Count", 1, 16, 4, "voices", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("sub_osc_gain", "Sub Osc Level", 0.0, 1.0, 0.3, "%", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_linear("stereo_spread", "Stereo Spread", 0.0, 1.0, 0.7, "%", (0, 184, 217)))
            ),
            "GranularSynthNode" => Box::new(
                GenericDspNodeUi::new("GranularSynthNode", "Cloud Granular Audio Streamer", DspNodeCategory::Oscillator)
                    .with_param(DspParamDescriptor::new_linear("density", "Grain Density", 1.0, 100.0, 25.0, "grains/s", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("spray", "Position Spray", 0.0, 1.0, 0.15, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("pitch_ratio", "Pitch Ratio", 0.25, 4.0, 1.0, "x", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("grain_size", "Grain Size", 0.01, 0.5, 0.06, "s", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_linear("reverse_prob", "Reverse Probability", 0.0, 1.0, 0.0, "%", (255, 61, 113)))
            ),
            "NeuralWavetable" => Box::new(
                GenericDspNodeUi::new("NeuralWavetable", "Latent AI Neural Wavetable Synthesizer", DspNodeCategory::NeuralAi)
                    .with_param(DspParamDescriptor::new_linear("latent_x", "Latent Timbre X", -3.0, 3.0, 0.0, "", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("latent_y", "Latent Timbre Y", -3.0, 3.0, 0.0, "", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("spectral_tilt", "Spectral Tilt", -12.0, 12.0, 0.0, "dB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("morph_speed", "Morph Slew Rate", 0.1, 10.0, 1.0, "Hz", (0, 230, 118)))
            ),
            "PlasmaArcSynthesizer" => Box::new(
                GenericDspNodeUi::new("PlasmaArcSynthesizer", "High-Voltage Plasma Arc Audio Generator", DspNodeCategory::Oscillator)
                    .with_param(DspParamDescriptor::new_log("arc_voltage", "Arc Voltage", 100.0, 5000.0, 1500.0, "V", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("plasma_density", "Plasma Density", 0.1, 1.0, 0.65, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("ionization", "Ionization Rate", 10.0, 500.0, 120.0, "kHz", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_linear("hiss_level", "Sub-thermal Hiss", 0.0, 1.0, 0.1, "%", (180, 195, 215)))
            ),
            "NoiseGen" => Box::new(
                GenericDspNodeUi::new("NoiseGen", "Multi-Color Spectral Noise Generator", DspNodeCategory::Oscillator)
                    .with_param(DspParamDescriptor::new_enum("noise_type", "Noise Color", vec!["White".into(), "Pink (1/f)".into(), "Brownian".into(), "Blue".into()], 0, (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_log("cutoff", "Filter Cutoff", 20.0, 20000.0, 12000.0, "Hz", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("gain", "Output Gain", 0.0, 1.0, 0.5, "%", (0, 230, 118)))
            ),
            "AetherSynth" => Box::new(
                GenericDspNodeUi::new("AetherSynth", "Dual Saw/Pulse Subtractive Synth with Moog Ladder Filter", DspNodeCategory::CompositeSynth)
                    .with_param(DspParamDescriptor::new_linear("osc_mix", "Saw / Pulse Mix", 0.0, 1.0, 0.5, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_log("filter_cutoff", "Moog Filter Cutoff", 20.0, 20000.0, 1200.0, "Hz", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("filter_res", "Ladder Resonance", 0.1, 4.0, 1.0, "Q", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_log("lfo_speed", "PWM LFO Speed", 0.1, 20.0, 2.0, "Hz", (0, 230, 118)))
            ),
            "PluckSynth" => Box::new(
                GenericDspNodeUi::new("PluckSynth", "Karplus-Strong Plucked String Synthesis Core", DspNodeCategory::Oscillator)
                    .with_param(DspParamDescriptor::new_linear("damping", "String Damping Cutoff", 100.0, 10000.0, 3500.0, "Hz", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("tension", "Feedback Loop Tension", 0.5, 0.999, 0.98, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("decay", "Exciter Burst Decay", 0.001, 0.1, 0.015, "s", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("brightness", "Noise Exciter Brightness", 0.0, 1.0, 0.7, "%", (0, 230, 118)))
            ),
            "CyberpunkSubSynth" => Box::new(
                GenericDspNodeUi::new("CyberpunkSubSynth", "Heavy Sub-bass Synth with Asymmetric Wavefolding", DspNodeCategory::CompositeSynth)
                    .with_param(DspParamDescriptor::new_log("sub_freq", "Sub Oscillator Freq", 20.0, 200.0, 55.0, "Hz", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("drive_gain", "Wavefolder Drive", 1.0, 10.0, 4.0, "x", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_log("filter_cutoff", "SVF Filter Cutoff", 40.0, 2000.0, 600.0, "Hz", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("pitch_wobble", "LFO Pitch Wobble", 0.0, 1.0, 0.3, "%", (255, 171, 0)))
            ),
            "AtmosphericPadSynth" => Box::new(
                GenericDspNodeUi::new("AtmosphericPadSynth", "Detuned Ambient Pad with SVF Filter & Space Reverb", DspNodeCategory::CompositeSynth)
                    .with_param(DspParamDescriptor::new_linear("saw_tri_mix", "Saw / Detuned Tri Mix", 0.0, 1.0, 0.5, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_log("filter_cutoff", "SVF Filter Cutoff", 100.0, 5000.0, 800.0, "Hz", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("reverb_mix", "Space Reverb Level", 0.0, 1.0, 0.4, "%", (101, 84, 171)))
                    .with_param(DspParamDescriptor::new_linear("delay_mix", "Stereo Delay Level", 0.0, 1.0, 0.3, "%", (0, 230, 118)))
            ),
            "GlitchPercussionSynth" => Box::new(
                GenericDspNodeUi::new("GlitchPercussionSynth", "Noise Burst & Granular Stutter Percussion Engine", DspNodeCategory::CompositeSynth)
                    .with_param(DspParamDescriptor::new_log("pitch_freq", "Pitch Drop Center", 30.0, 800.0, 120.0, "Hz", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_int("bitcrush_depth", "Bitcrusher Depth", 1, 16, 8, "bits", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_log("comb_freq", "Comb Resonator Freq", 50.0, 2000.0, 240.0, "Hz", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("stutter_rate", "Buffer Stutter Speed", 1.0, 64.0, 16.0, "div", (255, 171, 0)))
            ),
            "GlitchAetherMachine" => Box::new(
                GenericDspNodeUi::new("GlitchAetherMachine", "Flagship Chopper & Tape-Stop Glitch Synthesizer", DspNodeCategory::CompositeSynth)
                    .with_param(DspParamDescriptor::new_linear("drive", "Tube Drive Saturation", 1.0, 20.0, 3.0, "x", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_log("filter_cutoff", "Moog Ladder Cutoff", 50.0, 10000.0, 2500.0, "Hz", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("chop_rate", "Glitch Chopper Rate", 1.0, 32.0, 8.0, "Hz", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_bool("tape_stop", "Tape Stop Effect Active", false, (255, 171, 0)))
            ),
            "FmOperatorPair" => Box::new(
                GenericDspNodeUi::new("FmOperatorPair", "2-Operator Phase Modulation FM Synthesizer", DspNodeCategory::Oscillator)
                    .with_param(DspParamDescriptor::new_log("carrier_freq", "Carrier Frequency", 20.0, 20000.0, 440.0, "Hz", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("mod_ratio", "Modulator Ratio", 0.25, 16.0, 2.0, "x", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("fm_depth", "FM Modulation Depth", 0.0, 10.0, 2.0, "idx", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_log("mod_decay", "Modulator Env Decay", 10.0, 2000.0, 300.0, "ms", (0, 230, 118)))
            ),
            "FmMatrixSynthesizer" => Box::new(
                GenericDspNodeUi::new("FmMatrixSynthesizer", "6-Operator DX-Style FM Matrix Synthesizer", DspNodeCategory::CompositeSynth)
                    .with_param(DspParamDescriptor::new_linear("mod_index", "FM Modulation Index", 0.0, 10.0, 3.5, "idx", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_enum("algorithm", "FM Routing Algorithm", vec!["Alg 1 (Serial 6-Op)".into(), "Alg 5 (Parallel Pairs)".into(), "Alg 16 (3-Stack Hybrid)".into(), "Alg 32 (All Parallel)".into()], 0, (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("feedback", "Op 6 Self-Feedback", 0.0, 1.0, 0.45, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("master_level", "Master Output Level", 0.0, 1.0, 0.85, "%", (0, 230, 118)))
            ),
            "AnalogDrumVoice" => Box::new(
                GenericDspNodeUi::new("AnalogDrumVoice", "808/909 Style Analog Synthesized Drum Voice", DspNodeCategory::Oscillator)
                    .with_param(DspParamDescriptor::new_log("tune", "Fundamental Pitch", 30.0, 500.0, 60.0, "Hz", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_log("decay", "Amp Envelope Decay", 10.0, 1500.0, 250.0, "ms", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("snap", "Click / Pitch Snap", 0.0, 1.0, 0.5, "%", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_linear("punch", "Transient Punch", 0.0, 1.0, 0.7, "%", (0, 230, 118)))
            ),
            "DrumMachineDevice" => Box::new(
                GenericDspNodeUi::new("DrumMachineDevice", "24-Pad Zero-Allocation Sampling Drum Machine", DspNodeCategory::SamplerSlicer)
                    .with_param(DspParamDescriptor::new_linear("master_volume", "Master Kit Volume", 0.0, 1.0, 0.85, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("pitch_offset", "Kit Pitch Offset", -12.0, 12.0, 0.0, "st", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("pad_decay", "Global Pad Decay Scale", 0.1, 2.0, 1.0, "x", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("dynamic_punch", "Master Dynamic Punch", 0.0, 1.0, 0.6, "%", (0, 230, 118)))
            ),
            "QuantumStateVectorOscillator" => Box::new(
                GenericDspNodeUi::new("QuantumStateVectorOscillator", "Qubit Phase Superposition & Hadamard Gate Synth", DspNodeCategory::Oscillator)
                    .with_param(DspParamDescriptor::new_log("freq", "Superposition Frequency", 20.0, 20000.0, 440.0, "Hz", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("phase_alpha", "Qubit Alpha Phase", 0.0, 360.0, 45.0, "deg", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("phase_beta", "Qubit Beta Phase", 0.0, 360.0, 45.0, "deg", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_linear("hadamard_mix", "Hadamard Transformation", 0.0, 1.0, 0.5, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_bool("pauli_x", "Pauli-X State Flip", false, (255, 61, 113)))
            ),
            "QuantumHarmonicOscillatorVoice" => Box::new(
                GenericDspNodeUi::new("QuantumHarmonicOscillatorVoice", "Quantum Harmonic Oscillator Voice Bank", DspNodeCategory::Oscillator)
                    .with_param(DspParamDescriptor::new_int("energy_level", "Quantum State Level (n)", 0, 10, 2, "n", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("wavefunction_sigma", "Wavefunction Spatial Width", 0.1, 5.0, 1.2, "sigma", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("potential_well", "Harmonic Well Depth", 0.1, 10.0, 4.0, "eV", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("output_gain", "Voice Output Gain", 0.0, 1.0, 0.85, "%", (0, 230, 118)))
            ),
            "NavierStokesFluidNode" => Box::new(
                GenericDspNodeUi::new("NavierStokesFluidNode", "Navier-Stokes 3D Acoustic Fluid Dynamics", DspNodeCategory::Oscillator)
                    .with_param(DspParamDescriptor::new_linear("sound_speed", "Fluid Sound Speed", 100.0, 2000.0, 343.0, "m/s", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("viscosity", "Kinematic Viscosity", 0.001, 1.0, 0.05, "", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("pressure_damping", "Poisson Pressure Damping", 0.9, 0.999, 0.995, "", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_int("grid_res", "3D Lattice Grid Size", 2, 8, 4, "cube", (0, 230, 118)))
            ),
            "MolecularVibrationResonator" => Box::new(
                GenericDspNodeUi::new("MolecularVibrationResonator", "Molecular Vibration Resonator Crystalline Lattice", DspNodeCategory::Oscillator)
                    .with_param(DspParamDescriptor::new_log("fundamental_hz", "Lattice Base Frequency", 20.0, 10000.0, 520.0, "Hz", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("resonance", "Modal Resonance Q", 0.1, 0.99, 0.85, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_enum("lattice_type", "Crystalline Lattice", vec!["Quartz (SiO2)".into(), "Diamond (Carbon)".into(), "Graphene 2D".into(), "Silicon Carbide".into()], 0, (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("thermal_damping", "Thermal Damping Loss", 0.0, 1.0, 0.15, "%", (0, 230, 118)))
            ),
            "ClosedLoopNeuroFeedbackOscillator" => Box::new(
                GenericDspNodeUi::new("ClosedLoopNeuroFeedbackOscillator", "Closed-Loop Neuro-Feedback Relaxation Oscillator", DspNodeCategory::Oscillator)
                    .with_param(DspParamDescriptor::new_linear("target_alpha_hz", "Target Alpha Frequency", 7.0, 14.0, 10.5, "Hz", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("feedback_gain", "Neuro-Feedback Loop Gain", 0.0, 1.0, 0.7, "%", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_linear("relaxation_depth", "Entrainment Depth", 0.0, 1.0, 0.65, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_log("binaural_carrier", "Binaural Carrier Center", 100.0, 500.0, 216.0, "Hz", (153, 102, 255)))
            ),
            "BrainwaveEntrainmentBinauralBeat" => Box::new(
                GenericDspNodeUi::new("BrainwaveEntrainmentBinauralBeat", "Brainwave Entrainment Binaural Beat Generator", DspNodeCategory::Oscillator)
                    .with_param(DspParamDescriptor::new_log("carrier_freq_hz", "Binaural Carrier Pitch", 50.0, 800.0, 250.0, "Hz", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("beat_freq_hz", "Differential Beat Freq", 0.5, 30.0, 7.83, "Hz", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("entrainment_depth", "Modulation Amplitude", 0.0, 1.0, 0.8, "%", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_bool("isochronic_pulse", "Isochronic Pulse Mode", false, (153, 102, 255)))
            ),
            "FusionResonanceSynth" => Box::new(
                GenericDspNodeUi::new("FusionResonanceSynth", "Tokamak Fusion Magnetic Plasma Resonance Synth", DspNodeCategory::CompositeSynth)
                    .with_param(DspParamDescriptor::new_linear("magnetic_confinement_t", "Magnetic Field Flux (B)", 1.0, 15.0, 5.5, "Tesla", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("plasma_temp_kev", "Core Plasma Temp", 1.0, 50.0, 15.0, "keV", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_log("cyclotron_freq_hz", "Ion Cyclotron Pitch", 100.0, 10000.0, 880.0, "Hz", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("fusion_gain", "Acoustic Output Level", 0.0, 1.0, 0.8, "%", (0, 230, 118)))
            ),
            "SubharmonicSynthNode" => Box::new(
                GenericDspNodeUi::new("SubharmonicSynthNode", "Subharmonic Multi-Octave Sub Oscillator", DspNodeCategory::Oscillator)
                    .with_param(DspParamDescriptor::new_linear("sub1_oct_gain", "Sub-1 Octave (-12st)", 0.0, 1.0, 0.8, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("sub2_oct_gain", "Sub-2 Octave (-24st)", 0.0, 1.0, 0.5, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_log("lowpass_cutoff", "Sub Lowpass Corner", 40.0, 400.0, 120.0, "Hz", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("drive", "Asymmetric Drive", 1.0, 5.0, 1.5, "x", (255, 61, 113)))
            ),
            "SamplerDevice" => Box::new(
                GenericDspNodeUi::new("SamplerDevice", "Multi-Sample Instrument with ADSR Envelope & Filter", DspNodeCategory::SamplerSlicer)
                    .with_param(DspParamDescriptor::new_log("filter_cutoff", "Ladder Filter Cutoff", 20.0, 20000.0, 18000.0, "Hz", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("filter_res", "Ladder Resonance", 0.0, 3.9, 0.7, "Q", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("amp_attack", "Attack Time", 0.001, 2.0, 0.005, "s", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_linear("amp_release", "Release Time", 0.01, 5.0, 0.4, "s", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("gain", "Master Output Gain", 0.0, 1.0, 0.8, "%", (255, 61, 113)))
            ),
            "MultiSamplerNode" => Box::new(
                GenericDspNodeUi::new("MultiSamplerNode", "Velocity-Layered Multi-Zone Audio Sampler", DspNodeCategory::SamplerSlicer)
                    .with_param(DspParamDescriptor::new_int("root_key", "Center Root Key", 0, 127, 60, "midi", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("velocity_track", "Velocity Sensitivity", 0.0, 1.0, 0.8, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_enum("interpolation", "Resampling Kernel", vec!["Linear".into(), "Hermite Cubic".into(), "Sinc Bandlimited".into()], 1, (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("gain", "Sample Playback Level", 0.0, 1.0, 0.9, "%", (0, 230, 118)))
            ),
            "SingingSynthesisNode" => Box::new(
                GenericDspNodeUi::new("SingingSynthesisNode", "Differentiable Neural Phoneme Singing Synthesizer", DspNodeCategory::NeuralAi)
                    .with_param(DspParamDescriptor::new_log("f0_pitch", "Vocal Fundamental F0", 50.0, 1000.0, 220.0, "Hz", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("formant_scale", "Formant Scale", 0.5, 2.0, 1.0, "x", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("breathiness", "Glottal Breathiness", 0.0, 1.0, 0.15, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("vibrato_depth", "Vocal Vibrato Depth", 0.0, 1.0, 0.3, "%", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_enum("phoneme", "Current Phoneme", vec!["/a/".into(), "/e/".into(), "/i/".into(), "/o/".into(), "/u/".into(), "/m/".into(), "/n/".into()], 0, (255, 61, 113)))
            ),
            "SpectralNeuralResynthesizerNode" => Box::new(
                GenericDspNodeUi::new("SpectralNeuralResynthesizerNode", "Latent FFT Timbre Morphing Resynthesizer", DspNodeCategory::SpectralResynthesis)
                    .with_param(DspParamDescriptor::new_linear("latent_timbre", "Latent Timbre Coordinate", -3.0, 3.0, 0.0, "", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_int("spectral_res", "FFT Bin Resolution", 64, 1024, 256, "bins", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("transient_boost", "Transient Preservation", 0.0, 1.0, 0.5, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("smear", "Phase Spectral Smear", 0.0, 1.0, 0.2, "%", (0, 230, 118)))
            ),
            "NeuralAudioRepairNode" => Box::new(
                GenericDspNodeUi::new("NeuralAudioRepairNode", "AI Spectral Audio Inpainting & De-Clicking Engine", DspNodeCategory::NeuralAi)
                    .with_param(DspParamDescriptor::new_linear("noise_reduction", "Spectral De-Noise Depth", 0.0, 1.0, 0.6, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("inpaint_strength", "Inpaint Reconstruction", 0.0, 1.0, 0.8, "%", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_bool("transient_protect", "Protect Fast Transients", true, (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_linear("blend", "Dry / Wet Repair Blend", 0.0, 1.0, 1.0, "%", (255, 171, 0)))
            ),
            "FilterSVF" => Box::new(
                GenericDspNodeUi::new("FilterSVF", "State-Variable Multi-Mode Filter (LP/HP/BP/Notch)", DspNodeCategory::FilterEq)
                    .with_param(DspParamDescriptor::new_enum("mode", "Filter Topology", vec!["Lowpass 12dB".into(), "Highpass 12dB".into(), "Bandpass".into(), "Notch".into()], 0, (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_log("cutoff", "Cutoff Frequency", 20.0, 20000.0, 1500.0, "Hz", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("resonance", "Resonance Q", 0.1, 10.0, 1.2, "", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("drive", "Pre-filter Drive", 0.0, 18.0, 0.0, "dB", (255, 61, 113)))
            ),
            "FilterLadder" => Box::new(
                GenericDspNodeUi::new("FilterLadder", "24dB/Oct Transistor Moog-Style Ladder Filter", DspNodeCategory::FilterEq)
                    .with_param(DspParamDescriptor::new_log("cutoff", "Cutoff Frequency", 20.0, 20000.0, 1200.0, "Hz", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("resonance", "Resonance Q", 0.1, 10.0, 1.5, "", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("drive", "Drive Saturation", 0.0, 24.0, 3.0, "dB", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_linear("key_track", "Key Tracking", 0.0, 1.0, 0.5, "%", (0, 229, 255)))
            ),
            "FilterLadder8" => Box::new(
                GenericDspNodeUi::new("FilterLadder8", "8-Pole 48dB/Oct Ultra-Steep Resonant Ladder Filter", DspNodeCategory::FilterEq)
                    .with_param(DspParamDescriptor::new_log("cutoff", "Cutoff Frequency", 20.0, 20000.0, 1000.0, "Hz", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("resonance", "Resonance Q", 0.1, 12.0, 2.0, "", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_int("pole_count", "Cascaded Poles", 2, 8, 8, "poles", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("saturation", "Non-linear Warmth", 0.0, 1.0, 0.4, "%", (255, 61, 113)))
            ),
            "FilterComb" => Box::new(
                GenericDspNodeUi::new("FilterComb", "Feedback Comb Resonator Filter", DspNodeCategory::FilterEq)
                    .with_param(DspParamDescriptor::new_linear("delay_ms", "Delay Time", 0.1, 50.0, 5.0, "ms", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("feedback", "Feedback Loop", -0.99, 0.99, 0.7, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("damping", "High Damping", 0.0, 1.0, 0.2, "%", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("dry_wet", "Dry / Wet Mix", 0.0, 1.0, 0.5, "%", (0, 230, 118)))
            ),
            "FilterBiquad" => Box::new(
                GenericDspNodeUi::new("FilterBiquad", "Direct Form II Transposed Biquad Multi-Mode Filter", DspNodeCategory::FilterEq)
                    .with_param(DspParamDescriptor::new_enum("filter_type", "Biquad Mode", vec!["Lowpass".into(), "Highpass".into(), "Bandpass".into(), "Notch".into(), "Peaking".into(), "LowShelf".into(), "HighShelf".into()], 0, (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_log("cutoff", "Center / Cutoff Freq", 20.0, 20000.0, 1000.0, "Hz", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("q_factor", "Filter Q Factor", 0.1, 20.0, 1.0, "Q", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("gain_db", "Band Boost / Cut", -24.0, 24.0, 0.0, "dB", (0, 230, 118)))
            ),
            "DcBlockFilter" => Box::new(
                GenericDspNodeUi::new("DcBlockFilter", "Zero-Phase DC Offset Removal Highpass Filter", DspNodeCategory::FilterEq)
                    .with_param(DspParamDescriptor::new_linear("pole_radius", "Pole Radius (R)", 0.9, 0.9999, 0.995, "", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("highpass_hz", "Corner Highpass Freq", 1.0, 50.0, 10.0, "Hz", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_bool("enabled", "DC Block Active", true, (0, 230, 118)))
            ),
            "LowCutFilter" => Box::new(
                GenericDspNodeUi::new("LowCutFilter", "Precision Sub-Sonic Low-Cut Highpass Filter", DspNodeCategory::FilterEq)
                    .with_param(DspParamDescriptor::new_log("cutoff", "Low Cut Frequency", 10.0, 1000.0, 30.0, "Hz", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_enum("slope_order", "Filter Slope", vec!["6 dB/oct".into(), "12 dB/oct".into(), "18 dB/oct".into(), "24 dB/oct".into(), "48 dB/oct".into()], 1, (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("resonance", "Resonance Q", 0.1, 2.0, 0.707, "Q", (255, 171, 0)))
            ),
            "HighCutFilter" => Box::new(
                GenericDspNodeUi::new("HighCutFilter", "Anti-Aliasing High-Cut Lowpass Filter", DspNodeCategory::FilterEq)
                    .with_param(DspParamDescriptor::new_log("cutoff", "High Cut Frequency", 1000.0, 20000.0, 16000.0, "Hz", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_enum("slope_order", "Filter Slope", vec!["6 dB/oct".into(), "12 dB/oct".into(), "18 dB/oct".into(), "24 dB/oct".into(), "48 dB/oct".into()], 1, (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("resonance", "Resonance Q", 0.1, 2.0, 0.707, "Q", (255, 171, 0)))
            ),
            "ParametricEqNode" => Box::new(
                GenericDspNodeUi::new("ParametricEqNode", "4-Band Precision Parametric Equalizer", DspNodeCategory::FilterEq)
                    .with_param(DspParamDescriptor::new_linear("low_shelf", "Low Shelf (100Hz)", -18.0, 18.0, 0.0, "dB", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("low_mid", "Low Mid Gain", -18.0, 18.0, 0.0, "dB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("high_mid", "High Mid Gain", -18.0, 18.0, 0.0, "dB", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("high_shelf", "High Shelf (10kHz)", -18.0, 18.0, 0.0, "dB", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_log("mid_freq", "Parametric Mid Freq", 200.0, 8000.0, 1000.0, "Hz", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_linear("mid_q", "Parametric Mid Q", 0.2, 10.0, 1.0, "Q", (0, 184, 217)))
            ),
            "MultiChannelSpectralEqualizerNode" => Box::new(
                GenericDspNodeUi::new("MultiChannelSpectralEqualizerNode", "64-Band Multi-Channel Spectral Sculptor", DspNodeCategory::SpectralResynthesis)
                    .with_param(DspParamDescriptor::new_linear("tilt_slope", "Spectral Tilt Slope", -6.0, 6.0, 0.0, "dB/oct", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("air_band", "High Air Presence", 0.0, 12.0, 2.0, "dB", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("low_sub", "Sub Bass Focus", 0.0, 12.0, 1.5, "dB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("dynamic_unmask", "Dynamic Masking Reduction", 0.0, 1.0, 0.4, "%", (0, 230, 118)))
            ),
            "FormantFilterNode" => Box::new(
                GenericDspNodeUi::new("FormantFilterNode", "Vowel Formant Filter Matrix", DspNodeCategory::FilterEq)
                    .with_param(DspParamDescriptor::new_enum("vowel", "Target Vowel", vec!["Vowel A (Ah)".into(), "Vowel E (Eh)".into(), "Vowel I (Ee)".into(), "Vowel O (Oh)".into(), "Vowel U (Oo)".into()], 0, (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("formant_shift", "Formant Shift", -12.0, 12.0, 0.0, "st", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("throat_len", "Acoustic Throat Length", 0.5, 2.0, 1.0, "x", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("resonance_q", "Formant Resonance Q", 1.0, 20.0, 5.0, "Q", (0, 230, 118)))
            ),
            "ModalFilterNode" => Box::new(
                GenericDspNodeUi::new("ModalFilterNode", "High-Order Acoustic Modal Resonator Bank", DspNodeCategory::FilterEq)
                    .with_param(DspParamDescriptor::new_log("base_freq", "Fundamental Modal Freq", 50.0, 8000.0, 440.0, "Hz", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_int("mode_count", "Active Modal Poles", 1, 16, 8, "modes", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("decay_rate", "Modal Decay Time", 0.05, 5.0, 1.2, "s", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("inharmonicity", "Inharmonic Mode Shift", 0.0, 2.0, 0.15, "", (0, 230, 118)))
            ),
            "HornReflectionFilter" => Box::new(
                GenericDspNodeUi::new("HornReflectionFilter", "Acoustic Bell Flare & Throat Reflection Filter", DspNodeCategory::FilterEq)
                    .with_param(DspParamDescriptor::new_linear("bell_flare", "Bell Flare Expansion", 0.1, 3.0, 1.0, "x", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("throat_diameter", "Throat Diameter", 5.0, 50.0, 15.0, "mm", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("reflection_coeff", "End Reflection Coefficient", 0.0, 1.0, 0.65, "%", (255, 171, 0)))
            ),
            "AutomatedDynamicEqNode" => Box::new(
                GenericDspNodeUi::new("AutomatedDynamicEqNode", "AI-Driven Intelligent Masking Reduction Dynamic EQ", DspNodeCategory::FilterEq)
                    .with_param(DspParamDescriptor::new_linear("target_lufs", "Target Program LUFS", -24.0, -8.0, -14.0, "LUFS", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("low_dynamic_cut", "Dynamic Mud Cut (250Hz)", 0.0, 12.0, 3.0, "dB", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("high_air_lift", "Dynamic Air Presence (12kHz)", 0.0, 12.0, 2.5, "dB", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_linear("adaptation_speed", "Neural Adaptation Speed", 10.0, 500.0, 100.0, "ms", (255, 171, 0)))
            ),
            "SubharmonicQuantumTunnelingFilter" => Box::new(
                GenericDspNodeUi::new("SubharmonicQuantumTunnelingFilter", "Subharmonic Quantum Tunneling Resonant Filter", DspNodeCategory::FilterEq)
                    .with_param(DspParamDescriptor::new_linear("barrier_width_nm", "Quantum Barrier Width", 0.5, 10.0, 2.5, "nm", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("tunneling_prob", "Tunneling Probability", 0.01, 1.0, 0.35, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("resonance_q", "Subharmonic Q Factor", 1.0, 30.0, 8.0, "Q", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_log("cutoff_hz", "Filter Cutoff Corner", 20.0, 12000.0, 800.0, "Hz", (0, 230, 118)))
            ),
            "MetamaterialRefractionFilter" => Box::new(
                GenericDspNodeUi::new("MetamaterialRefractionFilter", "Negative Index Acoustic Metamaterial Filter", DspNodeCategory::FilterEq)
                    .with_param(DspParamDescriptor::new_linear("refractive_index", "Negative Index (n)", -5.0, -0.1, -1.4, "n", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("slab_thickness_mm", "Metamaterial Thickness", 1.0, 50.0, 12.0, "mm", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("evanescent_gain", "Evanescent Amplification", 0.0, 12.0, 3.5, "dB", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_linear("resonance_q", "Acoustic Bandgap Q", 1.0, 25.0, 6.0, "Q", (153, 102, 255)))
            ),
            "SpatialAcousticHologramFilter" => Box::new(
                GenericDspNodeUi::new("SpatialAcousticHologramFilter", "Spatial Acoustic Hologram Reconstruction Filter", DspNodeCategory::FilterEq)
                    .with_param(DspParamDescriptor::new_int("hologram_resolution", "Holographic Phase Steps", 16, 256, 64, "steps", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("phase_offset_rad", "Acoustic Phase Bias", 0.0, 6.28, 0.0, "rad", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("depth_focus_m", "Focal Distance", 0.2, 10.0, 2.0, "m", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("spatial_dispersion", "Diffractive Smear", 0.0, 1.0, 0.2, "%", (0, 230, 118)))
            ),
            "LinearPhaseCrossoverNode" => Box::new(
                GenericDspNodeUi::new("LinearPhaseCrossoverNode", "4-Way Linear Phase Mastering Crossover Filter", DspNodeCategory::FilterEq)
                    .with_param(DspParamDescriptor::new_log("low_cross_hz", "Low Crossover Split", 40.0, 300.0, 120.0, "Hz", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_log("mid_cross_hz", "Mid Crossover Split", 400.0, 3000.0, 1200.0, "Hz", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_log("high_cross_hz", "High Crossover Split", 3000.0, 16000.0, 6000.0, "Hz", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_int("fir_tap_order", "FIR Filter Tap Count", 256, 4096, 1024, "taps", (0, 230, 118)))
            ),
            "SpectralMatchingEqNode" => Box::new(
                GenericDspNodeUi::new("SpectralMatchingEqNode", "AI 128-Band Spectral Curve Matching EQ", DspNodeCategory::SpectralResynthesis)
                    .with_param(DspParamDescriptor::new_enum("target_curve", "Target Spectral Profile", vec!["Reference Track Target".into(), "Harman Target".into(), "Pink Noise Curve (1/f)".into(), "Commercial Master".into()], 0, (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("matching_intensity", "Curve Match Depth", 0.0, 100.0, 75.0, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("smoothing_octaves", "Spectral Smoothing", 0.1, 2.0, 0.5, "oct", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("gain_limit_db", "Max Boost/Cut Limit", 1.0, 18.0, 6.0, "dB", (0, 230, 118)))
            ),
            "SpectralTiltNode" => Box::new(
                GenericDspNodeUi::new("SpectralTiltNode", "Linear Phase Psychoacoustic Spectral Tilt Filter", DspNodeCategory::SpectralResynthesis)
                    .with_param(DspParamDescriptor::new_linear("tilt_slope_db_oct", "Bode Spectral Tilt", -6.0, 6.0, 1.5, "dB/oct", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_log("pivot_freq_hz", "Center Pivot Frequency", 200.0, 5000.0, 1000.0, "Hz", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("air_lift_db", "Top End Air Sheen", 0.0, 6.0, 1.2, "dB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("sub_weight_db", "Sub-Bass Weight", -6.0, 6.0, 0.0, "dB", (0, 230, 118)))
            ),
            "CombResonatorNode" => Box::new(
                GenericDspNodeUi::new("CombResonatorNode", "Tuned Feedback Comb Filter Resonator", DspNodeCategory::FilterEq)
                    .with_param(DspParamDescriptor::new_log("frequency_hz", "Comb Fundamental Pitch", 20.0, 5000.0, 440.0, "Hz", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("feedback_gain", "Feedback Loop Resonance", -0.99, 0.99, 0.85, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("damping_ratio", "High Frequency Damping", 0.0, 1.0, 0.25, "%", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("dry_wet", "Dry / Wet Blend", 0.0, 1.0, 0.5, "%", (0, 230, 118)))
            ),
            "EnvADSR" => Box::new(
                GenericDspNodeUi::new("EnvADSR", "Exponential Multi-Stage ADSR Envelope", DspNodeCategory::Modulation)
                    .with_param(DspParamDescriptor::new_log("attack", "Attack Time", 0.5, 5000.0, 10.0, "ms", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_log("decay", "Decay Time", 1.0, 10000.0, 250.0, "ms", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("sustain", "Sustain Level", 0.0, 1.0, 0.7, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_log("release", "Release Time", 1.0, 10000.0, 400.0, "ms", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("curve", "Curve Curvature", 0.1, 5.0, 1.0, "exp", (255, 61, 113)))
            ),
            "OscLFO" => Box::new(
                GenericDspNodeUi::new("OscLFO", "Multi-Waveform Syncable LFO Generator", DspNodeCategory::Modulation)
                    .with_param(DspParamDescriptor::new_log("rate", "LFO Frequency", 0.05, 50.0, 2.0, "Hz", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_enum("shape", "Waveform Shape", vec!["Sine".into(), "Triangle".into(), "Saw Up".into(), "Saw Down".into(), "Square".into(), "Sample & Hold".into()], 0, (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("depth", "Modulation Depth", 0.0, 1.0, 0.8, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("phase", "Phase Offset", 0.0, 360.0, 0.0, "deg", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_bool("tempo_sync", "BPM Tempo Sync", false, (255, 61, 113)))
            ),
            "EnvelopeFollowerNode" => Box::new(
                GenericDspNodeUi::new("EnvelopeFollowerNode", "Real-Time Peak/RMS Sidechain Envelope Follower", DspNodeCategory::Modulation)
                    .with_param(DspParamDescriptor::new_linear("attack_ms", "Attack Smoothing", 0.1, 200.0, 10.0, "ms", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_linear("release_ms", "Release Smoothing", 5.0, 1000.0, 150.0, "ms", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("sensitivity", "Follower Sensitivity", 0.0, 1.0, 0.7, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_bool("peak_mode", "Peak (True) vs RMS (False)", false, (153, 102, 255)))
            ),
            "ModulationMatrix" => Box::new(
                GenericDspNodeUi::new("ModulationMatrix", "16x16 Modulation Cross-Routing Matrix", DspNodeCategory::Modulation)
                    .with_param(DspParamDescriptor::new_linear("master_depth", "Master Matrix Scale", 0.0, 2.0, 1.0, "x", (0, 184, 217)))
                    .with_param(DspParamDescriptor::new_linear("slew_rate", "Slew Limiter Smoothing", 0.0, 500.0, 5.0, "ms", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_bool("bipolar", "Bipolar Signal Range", true, (0, 230, 118)))
            ),
            "PitchShifterNode" => Box::new(
                GenericDspNodeUi::new("PitchShifterNode", "Real-time Granular Pitch & Formant Shifter", DspNodeCategory::Modulation)
                    .with_param(DspParamDescriptor::new_linear("pitch_shift", "Pitch Shift", -24.0, 24.0, 0.0, "st", (0, 184, 217)))
                    .with_param(DspParamDescriptor::new_linear("fine_tune", "Fine Detune", -100.0, 100.0, 0.0, "cents", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_bool("formant_lock", "Preserve Formants", true, (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("window_ms", "Grain Window Size", 10.0, 200.0, 50.0, "ms", (0, 230, 118)))
            ),
            "AutoWahNode" => Box::new(
                GenericDspNodeUi::new("AutoWahNode", "Dynamic Envelope-Controlled Auto-Wah", DspNodeCategory::Modulation)
                    .with_param(DspParamDescriptor::new_linear("sensitivity", "Envelope Sensitivity", 0.0, 1.0, 0.65, "%", (0, 184, 217)))
                    .with_param(DspParamDescriptor::new_log("sweep_range", "Wah Center Freq", 200.0, 6000.0, 2400.0, "Hz", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("attack_ms", "Attack Speed", 1.0, 200.0, 15.0, "ms", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_linear("decay_ms", "Decay Speed", 10.0, 1000.0, 180.0, "ms", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("resonance", "Resonance Q", 1.0, 15.0, 4.5, "Q", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_linear("dry_wet", "Dry / Wet Mix", 0.0, 1.0, 1.0, "%", (0, 229, 255)))
            ),
            "AutoTuneNode" => Box::new(
                GenericDspNodeUi::new("AutoTuneNode", "Real-time Microtonal Scale Quantizer & Pitch Corrector", DspNodeCategory::Modulation)
                    .with_param(DspParamDescriptor::new_enum("target_scale", "Target Scale", vec!["Chromatic".into(), "Major".into(), "Natural Minor".into(), "Harmonic Minor".into(), "Pentatonic".into(), "Dorian".into()], 0, (0, 184, 217)))
                    .with_param(DspParamDescriptor::new_linear("speed_ms", "Retune Speed", 0.0, 200.0, 25.0, "ms", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("humanize", "Vocal Humanize", 0.0, 1.0, 0.3, "%", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_linear("amount", "Correction Depth", 0.0, 1.0, 0.85, "%", (153, 102, 255)))
            ),
            "AutotuneNode" => Box::new(
                GenericDspNodeUi::new("AutotuneNode", "Vocal Scale Intonation & Pitch Quantizer", DspNodeCategory::Modulation)
                    .with_param(DspParamDescriptor::new_enum("scale", "Scale Preset", vec!["Chromatic".into(), "Equal Temperament".into(), "Just Intonation".into(), "Pythagorean".into(), "Quarter Tone 24-EDO".into()], 0, (0, 184, 217)))
                    .with_param(DspParamDescriptor::new_linear("snap_speed", "Snap Transition Rate", 0.0, 100.0, 15.0, "ms", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("depth", "Quantize Amount", 0.0, 1.0, 0.9, "%", (0, 230, 118)))
            ),
            "VocalPitchFormantCorrectorNode" => Box::new(
                GenericDspNodeUi::new("VocalPitchFormantCorrectorNode", "Surgical Formant & Gender Vocal Pitch Shifter", DspNodeCategory::Modulation)
                    .with_param(DspParamDescriptor::new_linear("pitch_shift", "Chromatic Pitch Shift", -12.0, 12.0, 0.0, "st", (0, 184, 217)))
                    .with_param(DspParamDescriptor::new_linear("formant_shift", "Formant Throat Shift", -12.0, 12.0, 0.0, "st", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("gender_morph", "Gender Timbre Morph", -1.0, 1.0, 0.0, "", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("throat_model", "Throat Length Scale", 0.5, 2.0, 1.0, "x", (0, 230, 118)))
            ),
            "GlitchBufferNode" => Box::new(
                GenericDspNodeUi::new("GlitchBufferNode", "Stutter, Shuffle, & Reverse Glitch Buffer Matrix", DspNodeCategory::Modulation)
                    .with_param(DspParamDescriptor::new_linear("slice_size", "Glitch Slice Window", 10.0, 500.0, 80.0, "ms", (0, 184, 217)))
                    .with_param(DspParamDescriptor::new_linear("stutter_prob", "Stutter Probability", 0.0, 1.0, 0.3, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("reverse_prob", "Reverse Probability", 0.0, 1.0, 0.2, "%", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_linear("pitch_drop", "Tape Stop Pitch Drop", 0.0, 1.0, 0.15, "%", (0, 230, 118)))
            ),
            "EffectChorus" => Box::new(
                GenericDspNodeUi::new("EffectChorus", "Multi-Voice BBD Stereo Chorus", DspNodeCategory::Modulation)
                    .with_param(DspParamDescriptor::new_log("rate", "Modulation Rate", 0.1, 10.0, 1.2, "Hz", (0, 184, 217)))
                    .with_param(DspParamDescriptor::new_linear("depth", "Modulation Depth", 0.0, 1.0, 0.6, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_int("voices", "Chorus Voice Count", 2, 8, 4, "voices", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("mix", "Dry / Wet Mix", 0.0, 1.0, 0.5, "%", (0, 230, 118)))
            ),
            "EffectFlanger" => Box::new(
                GenericDspNodeUi::new("EffectFlanger", "Through-Zero Flanger & Comb Sweep", DspNodeCategory::Modulation)
                    .with_param(DspParamDescriptor::new_log("rate", "Sweep Rate", 0.05, 5.0, 0.4, "Hz", (0, 184, 217)))
                    .with_param(DspParamDescriptor::new_linear("depth", "Flange Depth", 0.0, 1.0, 0.75, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("feedback", "Regeneration Feedback", -0.95, 0.95, 0.65, "%", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_linear("mix", "Dry / Wet Mix", 0.0, 1.0, 0.5, "%", (0, 230, 118)))
            ),
            "EffectPhaser" => Box::new(
                GenericDspNodeUi::new("EffectPhaser", "Multi-Stage Allpass Phase Shifter", DspNodeCategory::Modulation)
                    .with_param(DspParamDescriptor::new_log("rate", "Phase Sweep Rate", 0.05, 10.0, 0.8, "Hz", (0, 184, 217)))
                    .with_param(DspParamDescriptor::new_enum("stages", "Allpass Stage Count", vec!["2 Stages".into(), "4 Stages".into(), "6 Stages".into(), "8 Stages".into(), "12 Stages".into()], 1, (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("feedback", "Phase Feedback", 0.0, 0.95, 0.5, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("mix", "Dry / Wet Mix", 0.0, 1.0, 0.5, "%", (0, 230, 118)))
            ),
            "RingModulator" => Box::new(
                GenericDspNodeUi::new("RingModulator", "Carrier Multiplier Ring Modulator", DspNodeCategory::Modulation)
                    .with_param(DspParamDescriptor::new_log("freq", "Carrier Frequency", 20.0, 5000.0, 320.0, "Hz", (0, 184, 217)))
                    .with_param(DspParamDescriptor::new_enum("waveform", "Carrier Waveform", vec!["Sine".into(), "Triangle".into(), "Square".into()], 0, (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("mix", "Dry / Wet Mix", 0.0, 1.0, 0.7, "%", (0, 230, 118)))
            ),
            "FrequencyShifter" => Box::new(
                GenericDspNodeUi::new("FrequencyShifter", "Single-Sideband Frequency Shifter", DspNodeCategory::Modulation)
                    .with_param(DspParamDescriptor::new_linear("shift_hz", "Frequency Shift", -1000.0, 1000.0, 25.0, "Hz", (0, 184, 217)))
                    .with_param(DspParamDescriptor::new_linear("feedback", "Feedback Loop", 0.0, 0.95, 0.0, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("mix", "Dry / Wet Mix", 0.0, 1.0, 0.6, "%", (0, 230, 118)))
            ),
            "ChaoticFractalAttractorModulator" => Box::new(
                GenericDspNodeUi::new("ChaoticFractalAttractorModulator", "Lorenz / Rossler Chaotic Strange Attractor LFO", DspNodeCategory::Modulation)
                    .with_param(DspParamDescriptor::new_enum("attractor_type", "Attractor Topology", vec!["Lorenz Butterfly".into(), "Rossler Spiral".into(), "Chua Circuit".into(), "Clifford Torus".into()], 0, (0, 184, 217)))
                    .with_param(DspParamDescriptor::new_linear("sigma_rate", "Prandtl Sigma Rate", 1.0, 20.0, 10.0, "sigma", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("rho_chaos", "Rayleigh Chaos Rho", 5.0, 50.0, 28.0, "rho", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_linear("modulation_depth", "Modulation Amplitude", 0.0, 1.0, 0.75, "%", (0, 230, 118)))
            ),
            "QuantumEntanglementModulationRouting" => Box::new(
                GenericDspNodeUi::new("QuantumEntanglementModulationRouting", "Quantum Entangled Bidirectional Modulator", DspNodeCategory::Modulation)
                    .with_param(DspParamDescriptor::new_enum("bell_state", "Entangled Bell State", vec!["Phi+ Superposition".into(), "Phi- Phase Inverted".into(), "Psi+ Symmetric".into(), "Psi- Anti-Symmetric".into()], 0, (0, 184, 217)))
                    .with_param(DspParamDescriptor::new_linear("entanglement_fidelity", "Quantum State Fidelity", 0.5, 1.0, 0.95, "%", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_linear("measurement_basis", "Measurement Angle", 0.0, 360.0, 45.0, "deg", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("routing_depth", "Entangled Cross-Route", 0.0, 1.0, 0.85, "%", (255, 171, 0)))
            ),
            "MhdPlasmaWaveModulator" => Box::new(
                GenericDspNodeUi::new("MhdPlasmaWaveModulator", "Magnetohydrodynamic Plasma Wave Modulator", DspNodeCategory::Modulation)
                    .with_param(DspParamDescriptor::new_linear("alfven_speed_ms", "Alfvén Wave Speed", 50.0, 2000.0, 450.0, "m/s", (0, 184, 217)))
                    .with_param(DspParamDescriptor::new_linear("plasma_beta", "Plasma Beta Factor", 0.01, 2.0, 0.4, "beta", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("magnetic_shear", "Magnetic Shear Angle", 0.0, 90.0, 15.0, "deg", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_log("wave_frequency_hz", "Modulation Carrier Freq", 0.1, 50.0, 4.5, "Hz", (0, 230, 118)))
            ),
            "BbdChorusNode" => Box::new(
                GenericDspNodeUi::new("BbdChorusNode", "Bucket-Brigade Analog Stereo Chorus Ensemble", DspNodeCategory::Modulation)
                    .with_param(DspParamDescriptor::new_int("bucket_stages", "BBD Delay Stages", 256, 4096, 1024, "stages", (0, 184, 217)))
                    .with_param(DspParamDescriptor::new_linear("clock_rate_khz", "BBD Clock Rate", 10.0, 100.0, 45.0, "kHz", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("modulation_depth", "LFO Sweep Depth", 0.0, 1.0, 0.65, "%", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_linear("stereo_spread", "Stereo Ensemble Width", 0.0, 1.0, 0.8, "%", (153, 102, 255)))
            ),
            "ThroughZeroFlangerNode" => Box::new(
                GenericDspNodeUi::new("ThroughZeroFlangerNode", "Tape-Style Through-Zero Flanger & Comb Phase", DspNodeCategory::Modulation)
                    .with_param(DspParamDescriptor::new_linear("zero_crossing_ms", "Zero Cross Delay Point", 0.0, 10.0, 2.5, "ms", (0, 184, 217)))
                    .with_param(DspParamDescriptor::new_linear("flange_rate_hz", "Sweep Rate", 0.05, 5.0, 0.35, "Hz", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("regen_feedback", "Regenerative Feedback", -0.98, 0.98, 0.7, "%", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_bool("phase_invert", "Phase Cancellation Invert", true, (153, 102, 255)))
            ),
            "TapeFlutterNode" => Box::new(
                GenericDspNodeUi::new("TapeFlutterNode", "Capstan Wow & Tape Flutter Modulator", DspNodeCategory::Modulation)
                    .with_param(DspParamDescriptor::new_linear("wow_rate_hz", "Capstan Wow Speed", 0.2, 4.0, 1.2, "Hz", (0, 184, 217)))
                    .with_param(DspParamDescriptor::new_linear("wow_depth", "Capstan Wow Depth", 0.0, 1.0, 0.25, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("flutter_rate_hz", "Scrape Flutter Speed", 5.0, 50.0, 18.0, "Hz", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_linear("flutter_depth", "Scrape Flutter Depth", 0.0, 1.0, 0.15, "%", (255, 61, 113)))
            ),
            "GranularFreezeNode" => Box::new(
                GenericDspNodeUi::new("GranularFreezeNode", "Infinite Granular Audio Buffer Freeze Engine", DspNodeCategory::Modulation)
                    .with_param(DspParamDescriptor::new_bool("freeze_active", "Buffer Freeze Locked", false, (0, 184, 217)))
                    .with_param(DspParamDescriptor::new_linear("grain_size_ms", "Grain Size Window", 10.0, 500.0, 80.0, "ms", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_linear("grain_jitter", "Playback Random Jitter", 0.0, 1.0, 0.3, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("diffusion_mix", "Stereo Diffusion Blend", 0.0, 1.0, 0.65, "%", (153, 102, 255)))
            ),
            "GranularPitchShifter" => Box::new(
                GenericDspNodeUi::new("GranularPitchShifter", "Dual-Grain Pitch Transposition & Shifter", DspNodeCategory::Modulation)
                    .with_param(DspParamDescriptor::new_linear("pitch_semitones", "Pitch Transposition", -24.0, 24.0, 7.0, "st", (0, 184, 217)))
                    .with_param(DspParamDescriptor::new_linear("fine_cents", "Fine Detuning", -50.0, 50.0, 0.0, "cents", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("window_size_ms", "Grain Overlap Window", 15.0, 250.0, 60.0, "ms", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_bool("formant_preserve", "Lock Vocal Formants", true, (153, 102, 255)))
            ),
            "PitchCorrectorNode" => Box::new(
                GenericDspNodeUi::new("PitchCorrectorNode", "Scale-Quantized Microtonal Vocal Pitch Corrector", DspNodeCategory::Modulation)
                    .with_param(DspParamDescriptor::new_enum("scale_mode", "Quantize Tuning Grid", vec!["12-EDO Equal".into(), "19-EDO Microtonal".into(), "31-EDO Fokker".into(), "Bohlen-Pierce".into(), "Arabic Maqam Bayati".into()], 0, (0, 184, 217)))
                    .with_param(DspParamDescriptor::new_linear("snap_speed_ms", "Correction Transition Rate", 0.0, 150.0, 20.0, "ms", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("correction_strength", "Intonation Pull Amount", 0.0, 1.0, 0.85, "%", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_linear("formant_shift", "Vocal Formant Offset", -6.0, 6.0, 0.0, "st", (153, 102, 255)))
            ),
            "VocoderMatrixNode" => Box::new(
                GenericDspNodeUi::new("VocoderMatrixNode", "32-Band Spectral Vocoder Analysis & Synthesis Matrix", DspNodeCategory::SpectralResynthesis)
                    .with_param(DspParamDescriptor::new_int("band_count", "Analysis Filter Bands", 8, 32, 24, "bands", (0, 184, 217)))
                    .with_param(DspParamDescriptor::new_linear("formant_warp", "Spectral Formant Warp", 0.5, 2.0, 1.0, "x", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("sibilance_thru", "Unvoiced Noise Pass-thru", 0.0, 1.0, 0.35, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("carrier_drive", "Carrier Excitation Drive", 1.0, 10.0, 2.5, "x", (0, 230, 118)))
            ),
            "GamelanGender" => Box::new(
                GenericDspNodeUi::new("GamelanGender", "Indonesian Gamelan Gendèr Metallophone & Bamboo Tubes", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_linear("mallet_hardness", "Mallet Hardness", 0.05, 1.0, 0.45, "%", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_linear("ombak_rate", "Ombak Beating Rate", 2.0, 12.0, 6.5, "Hz", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("bronze_thickness", "Bronze Thickness", 3.0, 20.0, 8.0, "mm", (0, 184, 217)))
                    .with_param(DspParamDescriptor::new_linear("bamboo_q", "Bamboo Resonator Q", 5.0, 60.0, 28.0, "", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("grasp_damping", "Hand Damping (Mipil)", 0.0, 1.0, 0.0, "%", (255, 61, 113)))
            ),
            "HurdyGurdy" => Box::new(
                GenericDspNodeUi::new("HurdyGurdy", "Vielle à Roue Rosin Wheel, Melody Chanters & Chien Buzz", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_linear("wheel_speed", "Crank Wheel RPM", 0.0, 200.0, 90.0, "RPM", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_linear("rosin_friction", "Rosin Stick-Slip", 0.1, 1.0, 0.65, "", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("chien_buzz", "Trompette Chien Buzz", 0.0, 1.0, 0.75, "%", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_linear("drone_level", "Bourdon Drones", 0.0, 1.0, 0.8, "%", (0, 229, 255)))
            ),
            "SitarModel" => Box::new(
                GenericDspNodeUi::new("SitarModel", "Indian Sitar, Curved Jawari Bridge & Sympathetic Tarab Bank", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_linear("jawari_curvature", "Jawari Bridge Arc", 0.01, 1.0, 0.35, "", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_linear("tarab_coupling", "Tarab Resonance", 0.0, 1.0, 0.7, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("meend_bend", "Meend Microtonal Bend", -7.0, 7.0, 0.0, "st", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("mizrab_strike", "Mizrab Strike Force", 0.0, 1.0, 0.85, "%", (255, 171, 0)))
            ),
            "GrandPianoModel" => Box::new(
                GenericDspNodeUi::new("GrandPianoModel", "Concert Grand Piano, Soundboard Coupling & 3 Pedals", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_linear("hammer_hardness", "Felt Hammer Hardness", 0.1, 1.0, 0.6, "%", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_linear("soundboard", "Soundboard Coupling", 0.0, 1.0, 0.75, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("duplex_scale", "Duplex Resonant Scale", 0.0, 1.0, 0.4, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("sustain_pedal", "Damper Sustain Pedal", 0.0, 1.0, 0.0, "%", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_linear("una_corda", "Una Corda Soft Pedal", 0.0, 1.0, 0.0, "%", (153, 102, 255)))
            ),
            "ShakuhachiModel" => Box::new(
                GenericDspNodeUi::new("ShakuhachiModel", "Japanese Bamboo Flute & Air-Reed Vortex Aerodynamics", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_log("blowing_pressure", "Blowing Pressure", 100.0, 2500.0, 650.0, "Pa", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_linear("embouchure", "Embouchure Angle", -30.0, 30.0, 0.0, "deg", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("meri_kari", "Meri/Kari Micro-Pitch", -4.0, 2.0, 0.0, "st", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("turbulence", "Air-Reed Turbulence", 0.0, 1.0, 0.25, "%", (255, 171, 0)))
            ),
            "PipeOrganModel" => Box::new(
                GenericDspNodeUi::new("PipeOrganModel", "Cathedral Pipe Organ Windchest & Rank Voicing", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_linear("windchest_press", "Windchest Pressure", 400.0, 2000.0, 850.0, "Pa", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_linear("rank_mixture", "Principal / Flute Mixture", 0.0, 1.0, 0.6, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("chiff", "Attack Chiff Transient", 0.0, 1.0, 0.35, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("tremulant", "Tremulant LFO Rate", 0.0, 10.0, 4.2, "Hz", (153, 102, 255)))
            ),
            "ClavinetModel" => Box::new(
                GenericDspNodeUi::new("ClavinetModel", "Electromagnetic Rock Clavinet & Rubber Pluck Hammer", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_linear("pluck_pos", "Pluck Position", 0.05, 0.95, 0.15, "%", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_linear("rubber_damper", "Rubber Damper Pad", 0.0, 1.0, 0.3, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_enum("pickups", "Pickup Select", vec!["Neck (A)".into(), "Bridge (B)".into(), "Both (A+B)".into(), "Phase Inv (A-B)".into()], 2, (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("snap", "Hammer Snap Tension", 0.0, 1.0, 0.7, "%", (255, 61, 113)))
            ),
            "ElectricPianoModel" => Box::new(
                GenericDspNodeUi::new("ElectricPianoModel", "Tine Rhodes / Wurlitzer Reed Resonator", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_linear("tine_coupling", "Tine-Tonebar Coupling", 0.0, 1.0, 0.65, "%", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_linear("pickup_dist", "Pickup Air Gap Distance", 0.5, 8.0, 2.0, "mm", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("bell_decay", "Tine Bell Decay", 0.1, 5.0, 1.8, "s", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("drive", "Preamp Overdrive", 0.0, 12.0, 2.5, "dB", (255, 61, 113)))
            ),
            "GlassArmonica" => Box::new(
                GenericDspNodeUi::new("GlassArmonica", "Franklin Glass Armonica Friction Model", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_linear("finger_press", "Finger Contact Force", 0.05, 1.0, 0.4, "%", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_linear("spindle_rpm", "Spindle Speed", 10.0, 120.0, 55.0, "RPM", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("water_damp", "Water Level Damping", 0.0, 1.0, 0.2, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("bowl_q", "Crystal Bowl Q Factor", 10.0, 100.0, 45.0, "", (153, 102, 255)))
            ),
            "BowedStringModel" => Box::new(
                GenericDspNodeUi::new("BowedStringModel", "Stradivarius Violin / Cello Bow Friction", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_linear("bow_force", "Bow Pressure Force", 0.05, 1.0, 0.55, "%", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_linear("bow_vel", "Bow Stroke Velocity", 0.01, 2.0, 0.35, "m/s", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("bow_pos", "Bow Contact Point", 0.05, 0.5, 0.12, "pos", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("body_res", "Spruce Body Cavity", 0.0, 1.0, 0.8, "%", (153, 102, 255)))
            ),
            "SteelpanModel" => Box::new(
                GenericDspNodeUi::new("SteelpanModel", "Trinidad Steelpan Drum Membrane Shell", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_linear("strike_vel", "Rubber Stick Strike", 0.0, 1.0, 0.7, "%", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_linear("shell_depth", "Oil Drum Shell Skirt", 5.0, 40.0, 18.0, "cm", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("ring_res", "Overtone Ring Sympathy", 0.0, 1.0, 0.6, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("damping", "Perimeter Damping", 0.0, 1.0, 0.15, "%", (255, 61, 113)))
            ),
            "TurkishNeyModel" => Box::new(
                GenericDspNodeUi::new("TurkishNeyModel", "Turkish Ney End-Blown Cane Flute", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_linear("angle", "Başpare Mouthpiece Angle", -25.0, 25.0, 0.0, "deg", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_linear("lip_aperture", "Lip Aperture Width", 0.5, 5.0, 2.0, "mm", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_enum("octave", "Register Mode", vec!["Rast (Low)".into(), "Neva (Mid)".into(), "Tiz (High)".into()], 1, (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("vortex", "Aerodynamic Vortex Blend", 0.0, 1.0, 0.3, "%", (255, 171, 0)))
            ),
            "WaveguideBrassModel" => Box::new(
                GenericDspNodeUi::new("WaveguideBrassModel", "Waveguide Lip-Reed Acoustic Brass & Flare", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_linear("lip_tension", "Lip Reed Tension", 0.1, 1.0, 0.5, "%", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_log("mouth_press", "Mouth Pressure", 200.0, 5000.0, 1200.0, "Pa", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("bell_flare", "Bell Flare Impedance", 0.1, 2.0, 1.0, "x", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_enum("mute", "Acoustic Mute", vec!["Open Bell".into(), "Straight Mute".into(), "Harmon / Wah".into(), "Cup Mute".into()], 0, (153, 102, 255)))
            ),
            "WoodwindJetModel" => Box::new(
                GenericDspNodeUi::new("WoodwindJetModel", "Flute Jet Instability & Aerodynamic Vortex", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_linear("jet_delay", "Jet Propagation Delay", 0.1, 10.0, 1.2, "ms", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_linear("jet_gain", "Vortex Non-Linear Gain", 0.1, 5.0, 1.8, "", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("breath_noise", "Turbulent Breath Noise", 0.0, 1.0, 0.18, "%", (255, 171, 0)))
            ),
            "SpringLatticeNode" => Box::new(
                GenericDspNodeUi::new("SpringLatticeNode", "Non-linear Spring Reverb Lattice & Dispersion", DspNodeCategory::TimeSpace)
                    .with_param(DspParamDescriptor::new_linear("tension", "Spring Lattice Tension", 0.1, 1.0, 0.65, "%", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_int("spring_count", "Coupled Springs", 2, 12, 6, "springs", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("dispersion", "Chirp Dispersion Rate", 0.0, 1.0, 0.55, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("decay_t60", "Decay Time (T60)", 0.5, 15.0, 3.5, "s", (153, 102, 255)))
            ),
            "PlateTankNode" => Box::new(
                GenericDspNodeUi::new("PlateTankNode", "EMT 140 Plate Reverb Tank & Transducer Dispersion", DspNodeCategory::TimeSpace)
                    .with_param(DspParamDescriptor::new_linear("tension", "Steel Plate Tension", 0.1, 1.0, 0.8, "%", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_linear("damping_pad", "Felt Damper Distance", 0.0, 1.0, 0.35, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("pickup_spread", "Stereo Pickup Spread", 0.0, 1.0, 0.85, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("decay_t60", "Reverberation Time", 0.5, 20.0, 4.2, "s", (153, 102, 255)))
            ),
            "TonewheelOrganModel" => Box::new(
                GenericDspNodeUi::new("TonewheelOrganModel", "Hammond B3 Tonewheel Organ & 9 Drawbars", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_int("drawbar_16", "Sub Drawbar 16'", 0, 8, 8, "", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_int("drawbar_8", "Fund Drawbar 8'", 0, 8, 8, "", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_int("drawbar_4", "Harm Drawbar 4'", 0, 8, 6, "", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_int("drawbar_2", "Block Drawbar 2'", 0, 8, 4, "", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("key_click", "Mechanical Key Click", 0.0, 1.0, 0.4, "%", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_enum("vibrato", "Scanner Vibrato/Chorus", vec!["Off".into(), "V1".into(), "C1".into(), "V2".into(), "C2".into(), "V3".into(), "C3".into()], 5, (0, 230, 118)))
            ),
            "VocalTractNode" => Box::new(
                GenericDspNodeUi::new("VocalTractNode", "Kelly-Lochbaum Acoustic Vocal Tract & Formants", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_log("pitch", "Glottal Pitch", 50.0, 1000.0, 220.0, "Hz", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_linear("tract_len", "Vocal Tract Length", 12.0, 22.0, 17.5, "cm", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("lip_area", "Lip Area Opening", 0.1, 5.0, 1.5, "cm2", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("tongue_pos", "Tongue Height / Position", 0.0, 1.0, 0.5, "%", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("nasal", "Velum Nasal Cavity", 0.0, 1.0, 0.0, "%", (255, 61, 113)))
            ),
            "BambooResonatorBank" => Box::new(
                GenericDspNodeUi::new("BambooResonatorBank", "Coupled Bamboo Air-Column Tube Resonators", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_log("tube_tuning_hz", "Tube Resonant Freq", 50.0, 2000.0, 261.63, "Hz", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_linear("cavity_q", "Air Column Q Factor", 5.0, 60.0, 30.0, "Q", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("air_column_len", "Bamboo Tube Length", 5.0, 80.0, 25.0, "cm", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("coupling", "Acoustic Air Coupling", 0.0, 1.0, 0.75, "%", (0, 230, 118)))
            ),
            "MizrabPluckModel" => Box::new(
                GenericDspNodeUi::new("MizrabPluckModel", "Wire Plectrum Transient Strike & Release Model", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_linear("plectrum_angle", "Pluck Angle", -45.0, 45.0, 15.0, "deg", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_linear("stroke_speed", "Stroke Velocity", 0.1, 5.0, 1.5, "m/s", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("wire_gauge", "Wire Gauge Thickness", 0.1, 1.0, 0.3, "mm", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("damping", "Post-Strike Damping", 0.0, 1.0, 0.1, "%", (255, 61, 113)))
            ),
            "JawariBridgeModel" => Box::new(
                GenericDspNodeUi::new("JawariBridgeModel", "Curved Flat Jawari Bridge Grazing Dynamics", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_linear("bridge_arc_radius", "Bridge Curvature Radius", 0.01, 1.0, 0.35, "m", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_linear("string_clearance", "String-to-Bridge Gap", 0.1, 3.0, 0.8, "mm", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("buzz_decay", "Buzz Transient Decay", 0.1, 5.0, 1.5, "s", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("harmonic_bleed", "Harmonic Cascade Bleed", 0.0, 1.0, 0.6, "%", (153, 102, 255)))
            ),
            "SoundboardBridgeModel" => Box::new(
                GenericDspNodeUi::new("SoundboardBridgeModel", "Spruce Soundboard & Rib Stiffness Impedance Bridge", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_enum("soundboard_wood", "Soundboard Tonewood", vec!["Sitka Spruce".into(), "European Spruce".into(), "Red Cedar".into(), "Mahogany".into()], 0, (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_linear("bridge_impedance", "Bridge Driving Impedance", 0.1, 5.0, 1.2, "x", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("rib_stiffness", "Spruce Rib Stiffness", 0.1, 3.0, 1.0, "x", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("decay_loss", "Soundboard Radiation Loss", 0.01, 0.5, 0.08, "", (0, 230, 118)))
            ),
            "TineResonatorModel" => Box::new(
                GenericDspNodeUi::new("TineResonatorModel", "Tuned Spring Steel Tine & Tonebar Resonator", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_linear("tine_length", "Steel Tine Length", 2.0, 20.0, 8.5, "cm", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_linear("tonebar_mass", "Tonebar Cast Mass", 10.0, 200.0, 65.0, "g", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("magnetic_gap", "Pickup Air Gap", 0.5, 8.0, 2.0, "mm", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("reed_clack", "Hammer Reed Impact", 0.0, 1.0, 0.35, "%", (255, 61, 113)))
            ),
            "TrompetteChienModel" => Box::new(
                GenericDspNodeUi::new("TrompetteChienModel", "Vielle Trompette Buzzing Dog Bridge Mechanism", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_linear("chien_pressure", "Trompette String Tension", 0.05, 1.0, 0.6, "%", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_linear("bridge_lift", "Loose Foot Bridge Lift", 0.1, 2.0, 0.5, "mm", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("buzz_threshold", "Crank Acceleration Threshold", 0.01, 0.5, 0.12, "g", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_linear("rosin_drag", "Wheel Rosin Viscosity", 0.1, 1.0, 0.45, "%", (255, 171, 0)))
            ),
            "WindchestAerodynamicsModel" => Box::new(
                GenericDspNodeUi::new("WindchestAerodynamicsModel", "Pipe Organ Windchest Pallet Valve & Reservoir", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_linear("reservoir_pressure", "Bellows Pressure", 400.0, 3000.0, 950.0, "Pa", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_linear("valve_inertia", "Pallet Valve Inertia", 0.01, 0.5, 0.08, "s", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("turbulence_vortex", "Channel Turbulence", 0.0, 1.0, 0.25, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("pipe_drag", "Pipe Foot Flow Drag", 0.01, 0.5, 0.1, "", (0, 230, 118)))
            ),
            "AirReedAcousticModel" => Box::new(
                GenericDspNodeUi::new("AirReedAcousticModel", "Aeroacoustic Air-Reed Jet Stream Instability", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_linear("blowing_velocity", "Jet Velocity", 5.0, 60.0, 25.0, "m/s", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_linear("jet_angle", "Flue Jet Strike Angle", -30.0, 30.0, 0.0, "deg", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("lip_distance", "Utaguchi Lip Distance", 1.0, 15.0, 4.5, "mm", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("edge_vortex_gain", "Vortex Splitting Gain", 0.1, 3.0, 1.2, "", (0, 230, 118)))
            ),
            "DigitalWaveguideNode" => Box::new(
                GenericDspNodeUi::new("DigitalWaveguideNode", "Bidirectional Delay-Line Digital Waveguide Core", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_linear("delay_line_len", "Waveguide Length Delay", 0.5, 50.0, 5.0, "ms", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_log("loss_filter_cutoff", "Loss Filter High Cut", 500.0, 20000.0, 6000.0, "Hz", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("dispersion_allpass", "Dispersion Allpass", -0.9, 0.9, 0.3, "", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("pickup_pos", "Acoustic Pickup Position", 0.01, 0.99, 0.2, "%", (0, 230, 118)))
            ),
            "WaveguideMesh2D" => Box::new(
                GenericDspNodeUi::new("WaveguideMesh2D", "2D Triangular Mesh Wave Propagation Surface", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_linear("grid_damping", "Mesh Damping Loss", 0.001, 0.1, 0.01, "", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_linear("tension", "Membrane Surface Tension", 0.1, 1.0, 0.7, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("boundary_reflection", "Boundary Reflection", 0.5, 0.999, 0.95, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("strike_point", "Impulse Strike Point", 0.0, 1.0, 0.5, "pos", (0, 230, 118)))
            ),
            "RotarySpeakerModel" => Box::new(
                GenericDspNodeUi::new("RotarySpeakerModel", "Leslie Dual-Rotor Doppler & Cabinet Diffraction", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_linear("horn_rpm", "Treble Horn RPM", 0.0, 450.0, 380.0, "RPM", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_linear("drum_rpm", "Bass Drum RPM", 0.0, 400.0, 340.0, "RPM", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("doppler_depth", "Doppler Pitch Shift Depth", 0.0, 1.0, 0.7, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_log("crossover_hz", "Cabinet Crossover", 200.0, 2000.0, 800.0, "Hz", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("drive", "Tube Preamp Saturation", 0.0, 12.0, 3.0, "dB", (255, 61, 113)))
            ),
            "StruckIdiophoneResonatorNode" => Box::new(
                GenericDspNodeUi::new("StruckIdiophoneResonatorNode", "Xylophone/Marimba Tuned Bar Modal Resonator", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_enum("bar_material", "Bar Material", vec!["Rosewood (Marimba)".into(), "Aluminum (Vibraphone)".into(), "Bronze (Glockenspiel)".into(), "Synthetic Polymer".into()], 0, (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_int("modal_density", "Active Modal Partials", 2, 16, 6, "modes", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("decay_t60", "Bar Ring Decay Time", 0.2, 10.0, 2.5, "s", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("strike_location", "Mallet Strike Node", 0.05, 0.95, 0.25, "pos", (0, 230, 118)))
            ),
            "MembranePercussionModel" => Box::new(
                GenericDspNodeUi::new("MembranePercussionModel", "2D Drumhead Membrane & Air Cavity Coupling", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_linear("head_tension", "Drumhead Lug Tension", 0.1, 1.0, 0.7, "%", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_linear("rim_shot_coupling", "Rim-to-Center Coupling", 0.0, 1.0, 0.4, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("air_cavity_depth", "Shell Air Cavity Depth", 5.0, 50.0, 20.0, "cm", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("snare_buzz", "Bottom Snare Wire Buzz", 0.0, 1.0, 0.5, "%", (255, 61, 113)))
            ),
            "GlottalPulseNode" => Box::new(
                GenericDspNodeUi::new("GlottalPulseNode", "Rosenberg Glottal Flow Waveform Model", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_log("f0_pitch", "Vocal Pitch F0", 40.0, 1000.0, 130.0, "Hz", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_linear("open_quotient", "Glottal Open Quotient (OQ)", 0.3, 0.9, 0.6, "", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("asymmetry_coeff", "Glottal Asymmetry", 0.5, 0.95, 0.75, "", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("aspiration_noise", "Turbulent Aspiration Noise", 0.0, 1.0, 0.1, "%", (153, 102, 255)))
            ),
            "HammerStrikeModel" => Box::new(
                GenericDspNodeUi::new("HammerStrikeModel", "Non-linear Felt Hammer Strike Dynamics", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_linear("hammer_mass", "Hammer Head Mass", 1.0, 20.0, 8.0, "g", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_linear("felt_elasticity", "Felt Non-linear Exponent", 0.1, 5.0, 1.8, "p", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("strike_velocity", "Key Strike Velocity", 0.1, 10.0, 3.5, "m/s", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("contact_duration", "String Contact Duration", 0.5, 10.0, 2.5, "ms", (0, 230, 118)))
            ),
            "ToneholeGridModel" => Box::new(
                GenericDspNodeUi::new("ToneholeGridModel", "Woodwind Tonehole Lattice & Acoustic Radiation", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_int("open_holes", "Open Finger Holes", 0, 12, 4, "holes", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_bool("register_key", "Register / Speaker Key Active", false, (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("lattice_impedance", "Tonehole Lattice Impedance", 0.1, 3.0, 1.0, "x", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("bell_radiation", "End Bell Radiation", 0.1, 2.0, 0.8, "x", (0, 230, 118)))
            ),
            "FrictionModel" => Box::new(
                GenericDspNodeUi::new("FrictionModel", "Stick-Slip Rosin Tribological Friction Model", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_linear("normal_force", "Bow / Rosin Normal Force", 0.01, 5.0, 1.0, "N", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_linear("relative_velocity", "Relative Velocity", 0.001, 2.0, 0.2, "m/s", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("static_friction", "Static Friction Coeff", 0.1, 1.5, 0.8, "", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("dynamic_friction", "Dynamic Friction Coeff", 0.05, 1.0, 0.4, "", (255, 61, 113)))
            ),
            "ModalResonator" => Box::new(
                GenericDspNodeUi::new("ModalResonator", "Universal Multi-Modal Resonator Filter Bank", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_log("freq", "Modal Base Frequency", 50.0, 8000.0, 440.0, "Hz", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_linear("q_factor", "Modal Q Factor", 1.0, 100.0, 30.0, "Q", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("decay_s", "Resonance Decay Time", 0.1, 10.0, 2.0, "s", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_enum("structure", "Modal Partials Spectrum", vec!["Harmonic String".into(), "Inharmonic Bar".into(), "Circular Membrane".into(), "Stiff Beam".into()], 0, (153, 102, 255)))
            ),
            "DulcimerCimbalomModel" => Box::new(
                GenericDspNodeUi::new("DulcimerCimbalomModel", "Hammered Dulcimer & Cimbalom Wire Resonator", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_linear("hammer_felt_hardness", "Plectrum/Felt Hardness", 0.1, 1.0, 0.65, "%", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_linear("course_unison_detune", "Course Unison Detune", 0.0, 25.0, 3.5, "cents", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("bridge_soundboard_gain", "Maple Bridge Coupling", 0.1, 3.0, 1.2, "x", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("damping_pedal", "Damper Bar Attenuation", 0.0, 1.0, 0.0, "%", (0, 230, 118)))
            ),
            "MbiraKalimbaModel" => Box::new(
                GenericDspNodeUi::new("MbiraKalimbaModel", "African Mbira / Kalimba Plucked Lamellophone", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_linear("tongue_length_mm", "Spring Steel Key Length", 30.0, 120.0, 65.0, "mm", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_linear("gourd_resonator_q", "Calabash Gourd Resonator Q", 5.0, 50.0, 22.0, "Q", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("bottlecap_buzz", "Bottlecap Jingler Buzz", 0.0, 1.0, 0.45, "%", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_linear("pluck_velocity", "Thumb Pluck Force", 0.1, 1.0, 0.7, "%", (255, 171, 0)))
            ),
            "MembranePlateModel" => Box::new(
                GenericDspNodeUi::new("MembranePlateModel", "Coupled 2D Membrane & Thin Plate Resonator", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_linear("membrane_tension", "Membrane Radial Tension", 0.1, 1.0, 0.75, "%", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_linear("plate_elastic_modulus", "Brass Plate Stiffness", 0.1, 5.0, 1.8, "GPa", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("air_gap_coupling", "Air Chamber Coupling", 0.01, 1.0, 0.6, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("edge_damping", "Clamped Rim Damping", 0.0, 1.0, 0.2, "%", (0, 230, 118)))
            ),
            "MembraneResonatorNode" => Box::new(
                GenericDspNodeUi::new("MembraneResonatorNode", "2D Elastic Drumhead Modal Resonator", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_linear("radial_strike_pos", "Radial Strike Radius", 0.0, 1.0, 0.35, "r/R", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_linear("membrane_tension_n", "Radial Skin Tension", 50.0, 2000.0, 450.0, "N/m", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("decay_t60_s", "Kettle Decay Time (T60)", 0.2, 8.0, 2.4, "s", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("rim_shot_mix", "Rim Edge Excitation Mix", 0.0, 1.0, 0.15, "%", (255, 61, 113)))
            ),
            "SympatheticCouplingModel" => Box::new(
                GenericDspNodeUi::new("SympatheticCouplingModel", "Sympathetic String Matrix Energy Coupling", DspNodeCategory::AcousticPhysicalModel)
                    .with_param(DspParamDescriptor::new_int("string_count", "Sympathetic Drone Strings", 4, 16, 11, "strings", (255, 215, 0)))
                    .with_param(DspParamDescriptor::new_linear("coupling_matrix_k", "Bridge Transfer Matrix (K)", 0.01, 1.0, 0.45, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("tarab_resonance_gain", "Tarab Resonance Boost", 0.0, 18.0, 6.0, "dB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("decay_time_s", "Sympathetic Ring Time", 0.5, 12.0, 4.5, "s", (0, 230, 118)))
            ),
            "CompressorNode" => Box::new(
                GenericDspNodeUi::new("CompressorNode", "VCA Studio Compressor with RMS Detection", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("threshold", "Threshold", -48.0, 0.0, -18.0, "dB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("ratio", "Compression Ratio", 1.0, 20.0, 4.0, ":1", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_log("attack", "Attack Time", 0.1, 200.0, 15.0, "ms", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_log("release", "Release Time", 10.0, 2000.0, 150.0, "ms", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("makeup", "Makeup Gain", 0.0, 24.0, 3.0, "dB", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_linear("knee", "Soft Knee Width", 0.0, 18.0, 3.0, "dB", (180, 195, 215)))
            ),
            "MultibandCompressorNode" => Box::new(
                GenericDspNodeUi::new("MultibandCompressorNode", "3-Band Linear-Phase Dynamics Processor", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("low_thresh", "Low Band Threshold", -40.0, 0.0, -18.0, "dB", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("mid_thresh", "Mid Band Threshold", -40.0, 0.0, -16.0, "dB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("high_thresh", "High Band Threshold", -40.0, 0.0, -14.0, "dB", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_log("low_cross", "Low/Mid Crossover", 60.0, 500.0, 180.0, "Hz", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_log("high_cross", "Mid/High Crossover", 1000.0, 10000.0, 3500.0, "Hz", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_linear("master_gain", "Master Output Trim", -12.0, 12.0, 0.0, "dB", (180, 195, 215)))
            ),
            "MultibandDynamicsProcessor" => Box::new(
                GenericDspNodeUi::new("MultibandDynamicsProcessor", "Tri-Band Expander/Compressor Dynamics Matrix", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("low_comp", "Low Band Compression", 0.0, 100.0, 35.0, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("mid_comp", "Mid Band Compression", 0.0, 100.0, 30.0, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("high_comp", "High Band Compression", 0.0, 100.0, 25.0, "%", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_log("crossover_1", "Low/Mid Crossover", 80.0, 600.0, 200.0, "Hz", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_log("crossover_2", "Mid/High Crossover", 1500.0, 8000.0, 4000.0, "Hz", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_linear("saturation", "Tape Saturation Drive", 0.0, 1.0, 0.2, "%", (180, 195, 215)))
            ),
            "OpticalCompressorNode" => Box::new(
                GenericDspNodeUi::new("OpticalCompressorNode", "Vintage LA-2A Optical Photocell Dynamics", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("peak_reduct", "Peak Reduction", 0.0, 100.0, 45.0, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("gain", "Output Gain", 0.0, 100.0, 50.0, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_bool("limit_mode", "Compress / Limit Mode", false, (255, 61, 113)))
            ),
            "NoiseGateNode" => Box::new(
                GenericDspNodeUi::new("NoiseGateNode", "Fast Transient Expander & Noise Gate", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("threshold", "Gate Threshold", -80.0, 0.0, -45.0, "dB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_log("attack", "Fast Attack", 0.05, 50.0, 1.0, "ms", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_linear("hold", "Hold Duration", 0.0, 500.0, 20.0, "ms", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_log("release", "Smooth Release", 5.0, 1000.0, 80.0, "ms", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("range", "Floor Attenuation Range", -80.0, 0.0, -60.0, "dB", (255, 61, 113)))
            ),
            "DeesserNode" => Box::new(
                GenericDspNodeUi::new("DeesserNode", "Dynamic High-Frequency Sibilance Suppressor", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("threshold", "De-Esser Threshold", -40.0, 0.0, -20.0, "dB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_log("freq", "Sibilance Center Freq", 3000.0, 12000.0, 6500.0, "Hz", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_bool("listen", "Delta Audition Listen", false, (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_linear("reduction", "Max Reduction Cap", -24.0, 0.0, -12.0, "dB", (153, 102, 255)))
            ),
            "NeuralTransientShaperNode" => Box::new(
                GenericDspNodeUi::new("NeuralTransientShaperNode", "Deep Neural Network Transient & Sustain Sculptor", DspNodeCategory::NeuralAi)
                    .with_param(DspParamDescriptor::new_linear("attack_boost", "Transient Attack Gain", -12.0, 12.0, 3.0, "dB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("sustain_tail", "Sustain Tail Gain", -12.0, 12.0, -1.5, "dB", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("neural_sensitivity", "AI Detection Sensitivity", 0.0, 1.0, 0.75, "%", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_linear("lookahead_ms", "Lookahead Buffer", 0.0, 20.0, 5.0, "ms", (153, 102, 255)))
            ),
            "NeuralBassGeneratorNode" => Box::new(
                GenericDspNodeUi::new("NeuralBassGeneratorNode", "Subharmonic Low-End Neural Bass Synthesizer", DspNodeCategory::NeuralAi)
                    .with_param(DspParamDescriptor::new_linear("sub_octave_gain", "Subharmonic Level", 0.0, 1.0, 0.65, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_int("harmonic_density", "Generated Harmonics", 1, 8, 3, "bands", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_enum("saturation_color", "Saturation Flavor", vec!["Warm Tube".into(), "Solid State".into(), "Tape Flux".into()], 0, (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_bool("mono_lows", "Mono Bass Summing (<120Hz)", true, (0, 230, 118)))
            ),
            "MultibandClipperNode" => Box::new(
                GenericDspNodeUi::new("MultibandClipperNode", "Tri-Band Mastering Clipper & Saturation Stage", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("low_clip_ceiling_db", "Low Band Ceiling", -12.0, 0.0, -0.5, "dBFS", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("mid_clip_ceiling_db", "Mid Band Ceiling", -12.0, 0.0, -0.3, "dBFS", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("high_clip_ceiling_db", "High Band Ceiling", -12.0, 0.0, -0.1, "dBFS", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("softness_knee", "Soft Clipping Knee", 0.0, 1.0, 0.35, "%", (0, 230, 118)))
            ),
            "MultibandDecompressorNode" => Box::new(
                GenericDspNodeUi::new("MultibandDecompressorNode", "Multiband Upward Dynamic Range Decompressor", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("expansion_ratio", "Upward Expansion Ratio", 1.0, 4.0, 1.5, ":1", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("upward_threshold_db", "Decompression Threshold", -40.0, -10.0, -24.0, "dB", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_log("crossover_low_hz", "Low/Mid Split Frequency", 60.0, 500.0, 160.0, "Hz", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_log("crossover_high_hz", "Mid/High Split Frequency", 1500.0, 10000.0, 4000.0, "Hz", (153, 102, 255)))
            ),
            "MultibandExpanderNode" => Box::new(
                GenericDspNodeUi::new("MultibandExpanderNode", "Tri-Band Downward Transient Expander", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("downward_ratio", "Expansion Ratio", 1.0, 8.0, 2.0, ":1", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("gate_threshold_db", "Expansion Floor Gate", -60.0, -10.0, -32.0, "dB", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("range_cap_db", "Max Dynamic Range Cap", -40.0, 0.0, -18.0, "dB", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_linear("release_time_ms", "Expander Recovery Time", 10.0, 500.0, 85.0, "ms", (0, 230, 118)))
            ),
            "MultibandSaturatorNode" => Box::new(
                GenericDspNodeUi::new("MultibandSaturatorNode", "3-Band Frequency-Split Saturation Warmer", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("low_drive_x", "Low Band Tape Warmth", 1.0, 10.0, 2.0, "x", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("mid_drive_x", "Mid Band Tube Drive", 1.0, 10.0, 3.2, "x", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("high_drive_x", "High Band Diode Sheen", 1.0, 10.0, 1.8, "x", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("tape_tube_blend", "Tape vs Tube Color Blend", 0.0, 1.0, 0.5, "%", (255, 61, 113)))
            ),
            "UpwardCompressorNode" => Box::new(
                GenericDspNodeUi::new("UpwardCompressorNode", "OTT-Style Upward Audio Compressor", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("upward_depth", "Upward Compression Depth", 0.0, 100.0, 65.0, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("downward_depth", "Downward Compression Depth", 0.0, 100.0, 40.0, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("dynamics_speed", "Dynamics Ballistics Speed", 0.1, 10.0, 2.5, "x", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_linear("high_lift_db", "High Air Presence Lift", 0.0, 18.0, 4.0, "dB", (153, 102, 255)))
            ),
            "VariMuMasterNode" => Box::new(
                GenericDspNodeUi::new("VariMuMasterNode", "Vintage Fairchild 670 Vari-Mu Tube Mastering Compressor", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_enum("time_constant", "Fairchild Time Constant", vec!["Pos 1 (Fast 0.2s/0.3s)".into(), "Pos 2 (0.2s/0.8s)".into(), "Pos 3 (0.4s/2.0s)".into(), "Pos 4 (0.8s/5.0s)".into(), "Pos 5 (Auto Fast)".into(), "Pos 6 (Auto Program)".into()], 4, (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("dc_threshold", "Vari-Mu Tube Threshold", -30.0, 0.0, -12.0, "dB", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_enum("tube_bias_mode", "Tube Bias Configuration", vec!["Standard Push-Pull".into(), "Aggressive Mid Push".into(), "Clean Master Linear".into()], 0, (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_bool("link_channels", "Stereo Linked Sidechain", true, (0, 230, 118)))
            ),
            "DynamicCrestShaperNode" => Box::new(
                GenericDspNodeUi::new("DynamicCrestShaperNode", "Intelligent Peak-to-RMS Crest Factor Shaper", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("target_crest_db", "Target Crest Factor", 6.0, 20.0, 12.0, "dB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("transient_priority", "Transient vs Body Weight", 0.0, 1.0, 0.6, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("smoothing_window_ms", "RMS Detection Window", 10.0, 500.0, 100.0, "ms", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_bool("auto_makeup", "Auto Crest Makeup Gain", true, (153, 102, 255)))
            ),
            "ParallelTransientSaturatorNode" => Box::new(
                GenericDspNodeUi::new("ParallelTransientSaturatorNode", "Parallel Wet/Dry Transient Driver & Saturator", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("transient_drive_db", "Transient Punch Drive", 0.0, 24.0, 6.0, "dB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("sustain_warmth_db", "Sustain Body Saturation", 0.0, 24.0, 4.5, "dB", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_linear("parallel_mix", "Parallel Wet Blend", 0.0, 1.0, 0.4, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_enum("clipping_mode", "Clipping Topology", vec!["Soft Germanium Diode".into(), "Hard Silicon Diode".into(), "Asymmetric Tube Triode".into()], 0, (153, 102, 255)))
            ),
            "TransientClipperNode" => Box::new(
                GenericDspNodeUi::new("TransientClipperNode", "Sub-Millisecond True Transient Soft Clipper", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("threshold_db", "Clipping Knee Threshold", -18.0, 0.0, -3.0, "dBFS", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("clip_hardness", "Clipper Hardness Exponent", 0.1, 5.0, 1.2, "", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_int("oversampling_order", "Anti-Aliasing Oversampling", 1, 16, 4, "x", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("ceiling_db", "True Peak Limit Ceiling", -6.0, 0.0, -0.1, "dBTP", (0, 230, 118)))
            ),
            "TransientDeclickerNode" => Box::new(
                GenericDspNodeUi::new("TransientDeclickerNode", "Surgical Sample De-Clicker & Pop Filter", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("sensitivity", "Detection Threshold", 0.0, 1.0, 0.7, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_int("click_width_samples", "Max Click Width", 2, 128, 16, "samples", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("spectral_inpaint_depth", "Inpainting Reconstruction", 0.0, 1.0, 0.9, "%", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_bool("listen_delta", "Audition Isolated Clicks", false, (255, 61, 113)))
            ),
            "TransientDesignerNode" => Box::new(
                GenericDspNodeUi::new("TransientDesignerNode", "Dual-Band Attack & Sustain Transient Designer", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("attack_gain_db", "Attack Transient Gain", -15.0, 15.0, 3.5, "dB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("sustain_gain_db", "Sustain Tail Gain", -15.0, 15.0, -2.0, "dB", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_log("split_frequency_hz", "Band Split Frequency", 100.0, 5000.0, 1200.0, "Hz", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("output_gain_db", "Master Output Level", -12.0, 12.0, 0.0, "dB", (0, 230, 118)))
            ),
            "TransientGateNode" => Box::new(
                GenericDspNodeUi::new("TransientGateNode", "Envelope-Triggered Transient Noise Gate", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("threshold_db", "Trigger Threshold", -70.0, 0.0, -40.0, "dB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("attack_fast_ms", "Fast Attack Speed", 0.01, 10.0, 0.5, "ms", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_linear("hold_time_ms", "Hold Gate Duration", 0.0, 250.0, 15.0, "ms", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("floor_attenuation_db", "Gate Attenuation Floor", -80.0, 0.0, -50.0, "dB", (255, 61, 113)))
            ),
            "TransientReconstructorNode" => Box::new(
                GenericDspNodeUi::new("TransientReconstructorNode", "Spectral Attack Transient Reconstructor", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("attack_restoration_db", "Attack Restoration Boost", 0.0, 18.0, 5.0, "dB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("harmonic_synthesis_q", "Harmonic Phase Match Q", 1.0, 20.0, 6.0, "Q", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("onset_sensitivity", "Onset Trigger Sensitivity", 0.0, 1.0, 0.8, "%", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_linear("wet_dry", "Reconstructed Blend", 0.0, 1.0, 0.6, "%", (153, 102, 255)))
            ),
            "TransientShaperNode" => Box::new(
                GenericDspNodeUi::new("TransientShaperNode", "Zero-Latency Transient Punch & Sustain Shaper", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("punch_level_db", "Transient Punch Level", -12.0, 12.0, 4.0, "dB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("body_level_db", "Body / Sustain Tail Level", -12.0, 12.0, 0.0, "dB", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("recovery_speed_ms", "Transient Recovery Speed", 5.0, 200.0, 45.0, "ms", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_bool("clip_protection", "Soft Clip Brickwall Guard", true, (255, 61, 113)))
            ),
            "TransientUnwrapperNode" => Box::new(
                GenericDspNodeUi::new("TransientUnwrapperNode", "De-Compression & Dynamic Unwrapper", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("crest_expansion_ratio", "Dynamic Unwrapping Ratio", 1.0, 3.5, 1.6, "x", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("unwrapper_threshold_db", "Unwrapping Floor Level", -36.0, -6.0, -18.0, "dB", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_bool("transient_preserve", "Preserve Fast Transients", true, (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_linear("gain_trim_db", "Post-Unwrap Output Trim", -12.0, 12.0, -1.0, "dB", (153, 102, 255)))
            ),
            "DistortionNode" => Box::new(
                GenericDspNodeUi::new("DistortionNode", "Asymmetric Diode Clipping Wavefolder", DspNodeCategory::DistortionSaturation)
                    .with_param(DspParamDescriptor::new_linear("drive", "Drive Gain", 1.0, 30.0, 4.0, "x", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_linear("asymmetry", "Even Harmonic Bias", 0.0, 1.0, 0.25, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_log("tone", "High Cut Tone", 500.0, 16000.0, 4500.0, "Hz", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("output", "Master Level", -24.0, 6.0, -2.0, "dB", (153, 102, 255)))
            ),
            "TapeSaturationNode" => Box::new(
                GenericDspNodeUi::new("TapeSaturationNode", "Magnetic Tape Hysteresis & Flux Compression", DspNodeCategory::DistortionSaturation)
                    .with_param(DspParamDescriptor::new_linear("drive", "Tape Input Drive", 1.0, 10.0, 3.0, "x", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_enum("speed", "Tape Speed", vec!["7.5 IPS (Warm)".into(), "15 IPS (Studio)".into(), "30 IPS (Mastering)".into()], 1, (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("flutter", "Wow & Flutter", 0.0, 1.0, 0.12, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("head_bump", "Head Bump Resonance", 0.0, 6.0, 2.0, "dB", (0, 230, 118)))
            ),
            "TubeSaturationNode" => Box::new(
                GenericDspNodeUi::new("TubeSaturationNode", "Triode/Pentode Dual-Stage Tube Saturation", DspNodeCategory::DistortionSaturation)
                    .with_param(DspParamDescriptor::new_linear("drive", "Tube Saturation Drive", 1.0, 12.0, 3.5, "x", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_linear("bias", "Grid Bias Asymmetry", 0.0, 0.5, 0.2, "", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_enum("topology", "Tube Model", vec!["12AX7 Triode".into(), "EL34 Pentode".into(), "6L6 Power Tube".into(), "KT88 Audiophile".into()], 0, (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("second_harm", "2nd Harmonic Richness", 0.0, 1.0, 0.4, "%", (0, 229, 255)))
            ),
            "TubeBiasNode" => Box::new(
                GenericDspNodeUi::new("TubeBiasNode", "Non-Linear Vacuum Tube Grid Bias Controller", DspNodeCategory::DistortionSaturation)
                    .with_param(DspParamDescriptor::new_linear("grid_bias_volts", "Tube Grid Bias Voltage", -5.0, 0.0, -1.8, "V", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_linear("plate_voltage_v", "Anode Plate Voltage", 100.0, 450.0, 250.0, "V", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("even_harmonic_blend", "Even vs Odd Harmonic Balance", 0.0, 1.0, 0.65, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("drive_level", "Input Overdrive Gain", 0.5, 8.0, 2.0, "x", (0, 230, 118)))
            ),
            "ConsoleEmulationNode" => Box::new(
                GenericDspNodeUi::new("ConsoleEmulationNode", "British Class-A Console Channel Coloration", DspNodeCategory::DistortionSaturation)
                    .with_param(DspParamDescriptor::new_enum("console", "Console Model", vec!["Neve 8078 (Warm)".into(), "SSL 4000E (Punchy)".into(), "API 1608 (Crisp)".into()], 0, (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_linear("drive", "Pushed Channel Drive", 0.0, 5.0, 1.2, "x", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("crosstalk", "Adjacent Crosstalk", -90.0, -30.0, -65.0, "dB", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("transformer", "Transformer Saturation", 0.0, 1.0, 0.5, "%", (0, 230, 118)))
            ),
            "BitcrusherNode" => Box::new(
                GenericDspNodeUi::new("BitcrusherNode", "Variable Sample-Rate & Bit-Depth Quantizer", DspNodeCategory::DistortionSaturation)
                    .with_param(DspParamDescriptor::new_int("bits", "Quantized Bit Depth", 1, 16, 8, "bits", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_int("downsample", "Downsample Ratio", 1, 32, 4, "x", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("jitter", "Clock Aperture Jitter", 0.0, 1.0, 0.05, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_bool("dither", "Anti-alias Dither", false, (153, 102, 255)))
            ),
            "WavefolderNode" => Box::new(
                GenericDspNodeUi::new("WavefolderNode", "Non-Linear Multi-Stage Wavefolder", DspNodeCategory::DistortionSaturation)
                    .with_param(DspParamDescriptor::new_linear("threshold", "Folding Threshold", 0.05, 1.0, 0.5, "%", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_int("folds", "Wavefold Iterations", 1, 8, 3, "folds", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("drive", "Pre-folder Gain", 1.0, 10.0, 2.5, "x", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("symmetry", "DC Offset Symmetry", -1.0, 1.0, 0.0, "", (153, 102, 255)))
            ),
            "HarmonicExciterNode" => Box::new(
                GenericDspNodeUi::new("HarmonicExciterNode", "Aural Exciter & High-Band Harmonics Generator", DspNodeCategory::DistortionSaturation)
                    .with_param(DspParamDescriptor::new_linear("drive", "Exciter Harmonics Drive", 0.0, 1.0, 0.45, "%", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_log("frequency", "High Shelf Cutoff", 2000.0, 15000.0, 6000.0, "Hz", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("mix", "Harmonics Wet Mix", 0.0, 1.0, 0.35, "%", (255, 171, 0)))
            ),
            "NeuralHarmonicExciterNode" => Box::new(
                GenericDspNodeUi::new("NeuralHarmonicExciterNode", "AI Spectral Sheen & High-Frequency Air Polish", DspNodeCategory::NeuralAi)
                    .with_param(DspParamDescriptor::new_linear("air_sheen", "Ultra-High Air Presence", 0.0, 12.0, 4.0, "dB", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_linear("warmth_body", "Low-Mid Harmonic Warmth", 0.0, 12.0, 2.0, "dB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_enum("ai_profile", "Exciter Neural Profile", vec!["Modern Polish".into(), "Vintage Tube".into(), "Transformer Color".into(), "Silk Top".into()], 0, (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("wet_mix", "Harmonic Blend Level", 0.0, 1.0, 0.4, "%", (0, 230, 118)))
            ),
            "Atmos916Spatializer" => Box::new(
                GenericDspNodeUi::new("Atmos916Spatializer", "Dolby Atmos 9.1.6 Object Panner with HRTF", DspNodeCategory::SpatialSurround)
                    .with_param(DspParamDescriptor::new_linear("azimuth", "Horizontal Azimuth", -180.0, 180.0, 0.0, "deg", (101, 84, 171)))
                    .with_param(DspParamDescriptor::new_linear("elevation", "Vertical Elevation", -90.0, 90.0, 0.0, "deg", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("distance", "Virtual Distance", 0.5, 20.0, 2.0, "m", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("spread", "Object Size Spread", 0.0, 100.0, 15.0, "%", (0, 230, 118)))
            ),
            "AmbisonicRadarSpatializer" => Box::new(
                GenericDspNodeUi::new("AmbisonicRadarSpatializer", "Higher-Order Ambisonics HOA-5 Spherical Radar", DspNodeCategory::SpatialSurround)
                    .with_param(DspParamDescriptor::new_linear("yaw", "Radar Yaw", -180.0, 180.0, 0.0, "deg", (101, 84, 171)))
                    .with_param(DspParamDescriptor::new_linear("pitch", "Radar Pitch", -90.0, 90.0, 0.0, "deg", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("roll", "Radar Roll", -180.0, 180.0, 0.0, "deg", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_int("hoa_order", "Ambisonics Order", 1, 5, 3, "order", (153, 102, 255)))
            ),
            "Auro3dSpatializer" => Box::new(
                GenericDspNodeUi::new("Auro3dSpatializer", "Auro-3D 13.1 Tri-Layer Immersive Panner", DspNodeCategory::SpatialSurround)
                    .with_param(DspParamDescriptor::new_linear("lower_layer", "Lower Ear-Level Gain", -12.0, 6.0, 0.0, "dB", (101, 84, 171)))
                    .with_param(DspParamDescriptor::new_linear("height_layer", "Height Layer Gain", -12.0, 6.0, 0.0, "dB", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("top_voice", "Top Voice of God", -12.0, 6.0, -3.0, "dB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("spread", "Surround Width Spread", 0.0, 1.0, 0.7, "%", (0, 230, 118)))
            ),
            "Nhk222Spatializer" => Box::new(
                GenericDspNodeUi::new("Nhk222Spatializer", "NHK 22.2 Super Hi-Vision Multi-Channel Matrix", DspNodeCategory::SpatialSurround)
                    .with_param(DspParamDescriptor::new_linear("top_gain", "Top Layer (9ch)", -12.0, 6.0, 0.0, "dB", (101, 84, 171)))
                    .with_param(DspParamDescriptor::new_linear("mid_gain", "Middle Layer (10ch)", -12.0, 6.0, 0.0, "dB", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("bottom_gain", "Bottom Layer (3ch)", -12.0, 6.0, -2.0, "dB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("lfe_sub", "Dual LFE Subwoofers", -12.0, 6.0, 0.0, "dB", (255, 61, 113)))
            ),
            "BinauralBrirSpatializer" => Box::new(
                GenericDspNodeUi::new("BinauralBrirSpatializer", "Binaural Room Impulse Response Convolver", DspNodeCategory::SpatialSurround)
                    .with_param(DspParamDescriptor::new_linear("room_size", "Room Dimensions", 5.0, 50.0, 15.0, "m", (101, 84, 171)))
                    .with_param(DspParamDescriptor::new_enum("hrtf", "HRTF Profile", vec!["KEMAR Dummy Head".into(), "CIPIC High-Res".into(), "Listen Individual".into(), "Sphaero HRIR".into()], 0, (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("early_gain", "Early Reflections Gain", -24.0, 6.0, 0.0, "dB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("late_diffuse", "Late Diffuse Field", -24.0, 6.0, -6.0, "dB", (153, 102, 255)))
            ),
            "BinauralSpatializerNode" => Box::new(
                GenericDspNodeUi::new("BinauralSpatializerNode", "3D Binaural HRTF Spatializer with ITD & ILD", DspNodeCategory::SpatialSurround)
                    .with_param(DspParamDescriptor::new_linear("azim_deg", "Azimuth Angle", -180.0, 180.0, 0.0, "deg", (101, 84, 171)))
                    .with_param(DspParamDescriptor::new_linear("elev_deg", "Elevation Angle", -90.0, 90.0, 0.0, "deg", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("dist_m", "Source Distance", 0.2, 20.0, 1.5, "m", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_bool("itd_ild_mode", "Interaural Delay & Level On", true, (0, 230, 118)))
            ),
            "RaytracedRoomReverb" => Box::new(
                GenericDspNodeUi::new("RaytracedRoomReverb", "Geometric Raytraced Acoustic Room Simulator", DspNodeCategory::TimeSpace)
                    .with_param(DspParamDescriptor::new_linear("room_width", "Room Width", 2.0, 100.0, 12.0, "m", (101, 84, 171)))
                    .with_param(DspParamDescriptor::new_linear("room_length", "Room Length", 2.0, 100.0, 18.0, "m", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("room_height", "Room Ceiling Height", 2.0, 30.0, 6.0, "m", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("wall_absorption", "Wall Acoustic Absorption", 0.05, 0.95, 0.25, "", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("scattering", "Diffusive Surface Scattering", 0.0, 1.0, 0.4, "%", (0, 230, 118)))
            ),
            "EffectDelay" => Box::new(
                GenericDspNodeUi::new("EffectDelay", "Stereo Ping-Pong Feedback Echo Delay", DspNodeCategory::TimeSpace)
                    .with_param(DspParamDescriptor::new_linear("time_s", "Delay Time", 0.01, 2.0, 0.35, "s", (101, 84, 171)))
                    .with_param(DspParamDescriptor::new_linear("feedback", "Delay Feedback", 0.0, 0.95, 0.45, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("mix", "Dry / Wet Mix", 0.0, 1.0, 0.3, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_bool("ping_pong", "Stereo Ping-Pong Mode", true, (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_log("high_cut", "Feedback High Cut Filter", 1000.0, 20000.0, 8000.0, "Hz", (153, 102, 255)))
            ),
            "EffectReverb" => Box::new(
                GenericDspNodeUi::new("EffectReverb", "Algorithmic Feedback Delay Network (FDN) Reverb", DspNodeCategory::TimeSpace)
                    .with_param(DspParamDescriptor::new_linear("room_size", "Virtual Room Size", 0.0, 1.0, 0.75, "%", (101, 84, 171)))
                    .with_param(DspParamDescriptor::new_linear("damping", "High Frequency Damping", 0.0, 1.0, 0.3, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("mix", "Reverb Wet Mix", 0.0, 1.0, 0.35, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("predelay_ms", "Pre-delay Time", 0.0, 250.0, 25.0, "ms", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_linear("decay_s", "Decay Time (T60)", 0.2, 10.0, 2.8, "s", (153, 102, 255)))
            ),
            "FdnReverbNode" => Box::new(
                GenericDspNodeUi::new("FdnReverbNode", "High-Density Feedback Delay Network Reverb Core", DspNodeCategory::TimeSpace)
                    .with_param(DspParamDescriptor::new_int("matrix_size", "FDN Matrix Delay Lines", 4, 16, 8, "lines", (101, 84, 171)))
                    .with_param(DspParamDescriptor::new_linear("t60_decay", "T60 Reverberation Time", 0.5, 20.0, 3.2, "s", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("diffusion", "Orthogonal Matrix Diffusion", 0.0, 1.0, 0.8, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("modal_density", "Modal Echo Density", 0.1, 2.0, 1.0, "x", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("dry_wet", "Dry / Wet Mix", 0.0, 1.0, 0.35, "%", (0, 230, 118)))
            ),
            "ConvolutionReverbNode" => Box::new(
                GenericDspNodeUi::new("ConvolutionReverbNode", "Zero-Latency Partitioned Convolution Reverb", DspNodeCategory::TimeSpace)
                    .with_param(DspParamDescriptor::new_linear("ir_decay", "Impulse Response Decay", 0.1, 10.0, 2.0, "s", (101, 84, 171)))
                    .with_param(DspParamDescriptor::new_linear("predelay", "Pre-delay", 0.0, 200.0, 10.0, "ms", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("high_damp", "High Frequency Damping", 0.0, 1.0, 0.2, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("dry_wet", "Dry / Wet Mix", 0.0, 1.0, 0.4, "%", (0, 230, 118)))
            ),
            "HyperDimensionalTensorSpatializer" => Box::new(
                GenericDspNodeUi::new("HyperDimensionalTensorSpatializer", "11-Dimensional Calabi-Yau Spatial Tensor Panner", DspNodeCategory::SpatialSurround)
                    .with_param(DspParamDescriptor::new_int("calabi_yau_dim", "Calabi-Yau Manifold Dims", 4, 11, 11, "dims", (101, 84, 171)))
                    .with_param(DspParamDescriptor::new_linear("manifold_curvature", "Ricci Curvature Scaling", 0.1, 5.0, 1.8, "k", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("subspace_rotation_deg", "Subspace Rotation Angle", 0.0, 360.0, 45.0, "deg", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("dimensional_spread", "High-Dim Sound Spread", 0.0, 1.0, 0.7, "%", (0, 230, 118)))
            ),
            "Holographic3dSpatialSoundfield" => Box::new(
                GenericDspNodeUi::new("Holographic3dSpatialSoundfield", "Holographic 3D Spherical Wavefield Synthesizer", DspNodeCategory::SpatialSurround)
                    .with_param(DspParamDescriptor::new_linear("spherical_radius_m", "Wavefield Sphere Radius", 0.5, 25.0, 3.0, "m", (101, 84, 171)))
                    .with_param(DspParamDescriptor::new_linear("azimuth_rad", "Source Azimuth Angle", -3.14, 3.14, 0.0, "rad", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("elevation_rad", "Source Elevation Angle", -1.57, 1.57, 0.0, "rad", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("wavefront_curvature", "Virtual Wave Curvature", 0.1, 5.0, 1.2, "", (0, 230, 118)))
            ),
            "NonEuclideanHyperbolicReverb" => Box::new(
                GenericDspNodeUi::new("NonEuclideanHyperbolicReverb", "Hyperbolic Geometry Non-Euclidean Reverb Tank", DspNodeCategory::TimeSpace)
                    .with_param(DspParamDescriptor::new_linear("poincare_curvature", "Poincaré Disk Curvature", -5.0, -0.1, -1.5, "K", (101, 84, 171)))
                    .with_param(DspParamDescriptor::new_linear("geodesic_diffusion", "Hyperbolic Geodesic Diffusion", 0.0, 1.0, 0.85, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("hyperbolic_t60_s", "Hyperbolic Decay Time", 0.5, 30.0, 5.5, "s", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("reflections_mix", "Infinite Horizon Reflection Mix", 0.0, 1.0, 0.4, "%", (0, 230, 118)))
            ),
            "IsmShockwaveReverb" => Box::new(
                GenericDspNodeUi::new("IsmShockwaveReverb", "Interstellar Medium Shockwave Convolution Reverb", DspNodeCategory::TimeSpace)
                    .with_param(DspParamDescriptor::new_linear("shockwave_radius_au", "Shock Front Radius", 0.1, 50.0, 5.0, "AU", (101, 84, 171)))
                    .with_param(DspParamDescriptor::new_linear("density_per_cm3", "Plasma Proton Density", 1.0, 1000.0, 150.0, "cm3", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("plasma_cooling_t60", "Radiative Cooling Decay", 1.0, 60.0, 12.0, "s", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("reverb_gain", "Interstellar Reverb Output", 0.0, 1.0, 0.75, "%", (0, 230, 118)))
            ),
            "AcousticCloakingSpatializer" => Box::new(
                GenericDspNodeUi::new("AcousticCloakingSpatializer", "Acoustic Cloaking Phase Cancellation Spatializer", DspNodeCategory::SpatialSurround)
                    .with_param(DspParamDescriptor::new_linear("cloaking_radius_m", "Cloaking Shell Radius", 0.5, 10.0, 2.0, "m", (101, 84, 171)))
                    .with_param(DspParamDescriptor::new_linear("scattering_cancellation", "Phase Scattering Cancel", 0.0, 1.0, 0.9, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("shadow_attenuation_db", "Acoustic Shadow Depth", -40.0, 0.0, -18.0, "dB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("phase_invert_blend", "Metasurface Anti-Phase Blend", 0.0, 1.0, 1.0, "%", (0, 230, 118)))
            ),
            "AtmosProximityNode" => Box::new(
                GenericDspNodeUi::new("AtmosProximityNode", "Dolby Atmos Proximity Effect & Distance Attenuator", DspNodeCategory::SpatialSurround)
                    .with_param(DspParamDescriptor::new_linear("distance_m", "Acoustic Source Distance", 0.1, 50.0, 2.5, "m", (101, 84, 171)))
                    .with_param(DspParamDescriptor::new_linear("air_absorption_hf_loss_db", "Air Absorption HF Loss", 0.0, 24.0, 4.5, "dB", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("proximity_bass_boost_db", "Proximity Bass Boost", 0.0, 18.0, 3.0, "dB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("itd_spread", "Interaural Distance Spread", 0.0, 1.0, 0.6, "%", (0, 230, 118)))
            ),
            "AtmosSurroundNode" => Box::new(
                GenericDspNodeUi::new("AtmosSurroundNode", "7.1.4 Immersive Atmos Surround Bed Renderer", DspNodeCategory::SpatialSurround)
                    .with_param(DspParamDescriptor::new_linear("surround_azimuth", "Bed Horizontal Azimuth", -180.0, 180.0, 0.0, "deg", (101, 84, 171)))
                    .with_param(DspParamDescriptor::new_linear("height_elevation", "Height Overhead Angle", 0.0, 90.0, 35.0, "deg", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("lfe_send_level", "LFE Subwoofer Send", -40.0, 6.0, 0.0, "dB", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_linear("divergence_spread", "Center-Surround Divergence", 0.0, 1.0, 0.5, "%", (0, 230, 118)))
            ),
            "HoaSpatializer" => Box::new(
                GenericDspNodeUi::new("HoaSpatializer", "4th/5th-Order High Order Ambisonic Spherical Panner", DspNodeCategory::SpatialSurround)
                    .with_param(DspParamDescriptor::new_linear("spherical_yaw", "Ambisonic Yaw Angle", -180.0, 180.0, 0.0, "deg", (101, 84, 171)))
                    .with_param(DspParamDescriptor::new_linear("spherical_pitch", "Ambisonic Pitch Angle", -90.0, 90.0, 0.0, "deg", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_int("ambisonic_order", "HOA Spatial Order", 1, 5, 4, "order", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("in_phase_weighting", "In-Phase Max-rE Weighting", 0.0, 1.0, 0.75, "%", (0, 230, 118)))
            ),
            "Hoa5BinauralSpatializer" => Box::new(
                GenericDspNodeUi::new("Hoa5BinauralSpatializer", "5th-Order Ambisonic Binaural Decoder with SOFA HRTF", DspNodeCategory::SpatialSurround)
                    .with_param(DspParamDescriptor::new_enum("sofa_profile", "SOFA HRIR Profile", vec!["KEMAR High-Precision".into(), "Genelec Aural ID".into(), "SADIE II Dummy".into(), "IRCAM Listen Subject 1002".into()], 0, (101, 84, 171)))
                    .with_param(DspParamDescriptor::new_enum("binaural_rendering_mode", "Binaural Decoder Kernel", vec!["Dual-Parabolic SVD".into(), "Spherical Harmonics Least-Squares".into(), "Diffuse Field Equalized".into()], 1, (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("nearfield_compensation", "Nearfield Bass Compensation", 0.0, 12.0, 2.5, "dB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("itd_emphasis", "Psychoacoustic ITD Emphasis", 0.5, 2.0, 1.0, "x", (0, 230, 118)))
            ),
            "Mpegh3DSpatializer" => Box::new(
                GenericDspNodeUi::new("Mpegh3DSpatializer", "MPEG-H 3D Immersive Audio Channel Bed Panner", DspNodeCategory::SpatialSurround)
                    .with_param(DspParamDescriptor::new_enum("bed_layout", "MPEG-H Target Bed", vec!["5.1.4 Surround".into(), "7.1.4 Immersive".into(), "9.1.6 3D Master".into(), "22.2 Hi-Vision".into()], 1, (101, 84, 171)))
                    .with_param(DspParamDescriptor::new_linear("elevation_gain", "Overhead Layer Trim", -12.0, 6.0, 0.0, "dB", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("object_spread", "Object Spherical Divergence", 0.0, 100.0, 20.0, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("lfe_gain", "LFE Subwoofer Level", -24.0, 6.0, 0.0, "dB", (255, 61, 113)))
            ),
            "MpeghTrajectorySpatializer" => Box::new(
                GenericDspNodeUi::new("MpeghTrajectorySpatializer", "3D Trajectory Bezier Spline Audio Spatializer", DspNodeCategory::SpatialSurround)
                    .with_param(DspParamDescriptor::new_linear("trajectory_speed_bpm", "Spline Trajectory Speed", 1.0, 240.0, 60.0, "BPM", (101, 84, 171)))
                    .with_param(DspParamDescriptor::new_linear("spline_tension", "Bezier Spline Tension", 0.0, 1.0, 0.5, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("path_radius_m", "Orbital Path Radius", 0.5, 20.0, 3.5, "m", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("elevation_range_m", "Vertical Elevation Sweep", 0.0, 10.0, 2.0, "m", (0, 230, 118)))
            ),
            "WfsArraySpatializerNode" => Box::new(
                GenericDspNodeUi::new("WfsArraySpatializerNode", "Wave Field Synthesis Multi-Transducer Array Engine", DspNodeCategory::SpatialSurround)
                    .with_param(DspParamDescriptor::new_int("array_elements_count", "WFS Transducer Count", 16, 128, 64, "speakers", (101, 84, 171)))
                    .with_param(DspParamDescriptor::new_linear("speaker_spacing_cm", "Transducer Element Pitch", 2.0, 30.0, 8.0, "cm", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("focused_source_depth_m", "Focused Virtual Depth", -5.0, 15.0, 2.5, "m", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("spatial_aliasing_cut", "Spatial Aliasing Lowpass", 1000.0, 15000.0, 4500.0, "Hz", (0, 230, 118)))
            ),
            "DiffractivePropagationNode" => Box::new(
                GenericDspNodeUi::new("DiffractivePropagationNode", "Acoustic Obstacle Diffraction & Shadow Zone Simulator", DspNodeCategory::SpatialSurround)
                    .with_param(DspParamDescriptor::new_linear("obstacle_width_m", "Obstacle Barrier Width", 0.2, 20.0, 3.0, "m", (101, 84, 171)))
                    .with_param(DspParamDescriptor::new_linear("diffraction_angle_deg", "Bending Diffraction Angle", 0.0, 180.0, 45.0, "deg", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_log("shadow_zone_lowpass_hz", "Shadow Zone High Cut", 200.0, 10000.0, 1200.0, "Hz", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("edge_scattering_gain", "Edge Scattering Amplitude", 0.0, 1.0, 0.35, "%", (0, 230, 118)))
            ),
            "BinauralPannerNode" => Box::new(
                GenericDspNodeUi::new("BinauralPannerNode", "Precision Interaural Time & Level Difference 3D Panner", DspNodeCategory::SpatialSurround)
                    .with_param(DspParamDescriptor::new_linear("azimuth_deg", "Source Azimuth Angle", -180.0, 180.0, 0.0, "deg", (101, 84, 171)))
                    .with_param(DspParamDescriptor::new_linear("elevation_deg", "Source Elevation Angle", -90.0, 90.0, 0.0, "deg", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("itd_us", "Interaural Time Difference", -700.0, 700.0, 0.0, "us", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("ild_db", "Interaural Level Difference", -24.0, 24.0, 0.0, "dB", (0, 230, 118)))
            ),
            "WavefrontReflectionNode" => Box::new(
                GenericDspNodeUi::new("WavefrontReflectionNode", "Early Specular & Diffuse Wavefront Room Reflection Engine", DspNodeCategory::SpatialSurround)
                    .with_param(DspParamDescriptor::new_linear("room_dimensions_xyz", "Room Scale Multiplier", 0.5, 5.0, 1.0, "x", (101, 84, 171)))
                    .with_param(DspParamDescriptor::new_linear("wall_absorption_coeff", "Boundary Absorption", 0.05, 0.95, 0.2, "", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_int("specular_order", "Ray Image Source Order", 1, 8, 4, "orders", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("diffuse_scatter_mix", "Diffuse Lambertian Scatter", 0.0, 1.0, 0.4, "%", (0, 230, 118)))
            ),
            "ReverbSpaceNode" => Box::new(
                GenericDspNodeUi::new("ReverbSpaceNode", "Parametric Multi-Room Acoustic Space Simulator", DspNodeCategory::TimeSpace)
                    .with_param(DspParamDescriptor::new_enum("space_geometry", "Room Architecture Profile", vec!["Concert Hall (Gold)".into(), "Cathedral Nave".into(), "Recording Studio Live Room".into(), "Tiled Chamber".into()], 0, (101, 84, 171)))
                    .with_param(DspParamDescriptor::new_linear("decay_time_t60", "Reverberant T60 Time", 0.3, 20.0, 2.6, "s", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("early_to_late_ratio", "Early to Late Diffuse Mix", 0.0, 1.0, 0.5, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_log("high_damping_hz", "High-Frequency Air Loss Cut", 1000.0, 20000.0, 6500.0, "Hz", (0, 230, 118)))
            ),
            "MultibandSpatialNode" => Box::new(
                GenericDspNodeUi::new("MultibandSpatialNode", "Tri-Band Frequency-Dependent Stereo & Spatial Panner", DspNodeCategory::SpatialSurround)
                    .with_param(DspParamDescriptor::new_linear("low_band_pan", "Low Band Panning (<200Hz)", -1.0, 1.0, 0.0, "pan", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("mid_band_pan", "Mid Band Panning", -1.0, 1.0, 0.25, "pan", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("high_band_pan", "High Band Panning (>4kHz)", -1.0, 1.0, -0.3, "pan", (101, 84, 171)))
                    .with_param(DspParamDescriptor::new_log("crossover_freq_hz", "Mid-High Crossover Point", 500.0, 8000.0, 2500.0, "Hz", (0, 230, 118)))
            ),
            "MultitapDelayNode" => Box::new(
                GenericDspNodeUi::new("MultitapDelayNode", "8-Tap Modulated Stereo Spatial Delay Line", DspNodeCategory::TimeSpace)
                    .with_param(DspParamDescriptor::new_int("tap_count", "Active Delay Taps", 2, 8, 8, "taps", (101, 84, 171)))
                    .with_param(DspParamDescriptor::new_linear("spatial_spread", "Stereo Tap Ping-Pong Spread", 0.0, 1.0, 0.85, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("feedback_matrix_gain", "Cross-Tap Feedback", 0.0, 0.95, 0.45, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("modulation_depth", "Tap Delay Time LFO Wobble", 0.0, 1.0, 0.2, "%", (0, 230, 118)))
            ),
            "LimiterNode" => Box::new(
                GenericDspNodeUi::new("LimiterNode", "True-Peak Lookahead Mastering Brickwall Limiter", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("ceiling", "True Peak Ceiling", -12.0, 0.0, -0.3, "dBFS", (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_linear("threshold", "Limiter Threshold", -24.0, 0.0, -3.0, "dB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("release_ms", "Lookahead Release", 5.0, 500.0, 50.0, "ms", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_bool("oversampling", "4x ISP Oversampling", true, (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_linear("lookahead_ms", "Lookahead Buffer", 0.5, 10.0, 3.0, "ms", (153, 102, 255)))
            ),
            "EbuLoudnessRadar" => Box::new(
                GenericDspNodeUi::new("EbuLoudnessRadar", "ITU-R BS.1770 / EBU R128 Loudness Radar Meter", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("target_lufs", "Target Integrated Loudness", -24.0, -9.0, -14.0, "LUFS", (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_linear("integrated_gate", "Relative Silence Gate", -70.0, -10.0, -23.0, "LUFS", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("max_momentary", "Max Momentary True Peak", -18.0, 0.0, -1.0, "dBTP", (255, 61, 113)))
            ),
            "LufsMeterNode" => Box::new(
                GenericDspNodeUi::new("LufsMeterNode", "Multi-Scale Momentary & Integrated LUFS Meter", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("momentary_window", "Momentary Window", 100.0, 1000.0, 400.0, "ms", (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_linear("short_term_window", "Short-Term Window", 1000.0, 5000.0, 3000.0, "ms", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("relative_threshold", "Relative Gate Threshold", -20.0, -5.0, -10.0, "LU", (255, 171, 0)))
            ),
            "MidSideNode" => Box::new(
                GenericDspNodeUi::new("MidSideNode", "Mid/Side Matrix Encoder & Stereo Width Imager", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("width", "Stereo Field Width", 0.0, 3.0, 1.0, "x", (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_linear("mid_gain", "Mid Channel Gain", -12.0, 12.0, 0.0, "dB", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("side_gain", "Side Channel Gain", -12.0, 12.0, 0.0, "dB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_log("mono_sub", "Mono Bass Crossover", 20.0, 250.0, 90.0, "Hz", (0, 230, 118)))
            ),
            "StereoImager" => Box::new(
                GenericDspNodeUi::new("StereoImager", "Phase-Safe Psychoacoustic Stereo Imager", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("width_factor", "Stereo Width Expansion", 0.0, 2.5, 1.2, "x", (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_log("bass_mono_cutoff", "Bass Mono Filter Corner", 30.0, 300.0, 120.0, "Hz", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_bool("phase_correlation_protect", "Phase Correlation Protection", true, (0, 230, 118)))
            ),
            "TpdfDitherNode" => Box::new(
                GenericDspNodeUi::new("TpdfDitherNode", "Triangular PDF Dither & Noise Shaping Quantizer", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_enum("bit_depth", "Target Bit Depth", vec!["24-bit HD".into(), "16-bit Red Book CD".into(), "12-bit Vintage".into(), "8-bit Lo-Fi".into()], 1, (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_enum("noise_shaping", "Noise Shaping Curve", vec!["None (Flat TPDF)".into(), "Lipshitz Moderate".into(), "High-Pass Ultra".into(), "Psychoacoustic F-Weight".into()], 1, (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_bool("auto_blank", "Auto Blank On Digital Silence", true, (0, 230, 118)))
            ),
            "AdaptivePsychoacousticLoudness" => Box::new(
                GenericDspNodeUi::new("AdaptivePsychoacousticLoudness", "ISO 226 Equal Loudness Contour Balancing Engine", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("target_phon", "Target Phon Loudness", 40.0, 100.0, 75.0, "phon", (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_linear("phon_compensation_intensity", "Equal Loudness Compensation", 0.0, 1.0, 0.8, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("low_contour_boost_db", "Fletcher-Munson Sub Boost", 0.0, 12.0, 3.5, "dB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("high_contour_presence_db", "Treble Contour Presence", 0.0, 6.0, 1.8, "dB", (0, 230, 118)))
            ),
            "DynamicStereoWidthNode" => Box::new(
                GenericDspNodeUi::new("DynamicStereoWidthNode", "Frequency-Dependent Mid/Side Dynamic Stereo Widener", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("stereo_expansion_factor", "Side Band Expansion", 0.0, 3.0, 1.35, "x", (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_log("bass_mono_crossover_hz", "Mono Bass Frequency Cut", 30.0, 250.0, 100.0, "Hz", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("high_side_air_gain_db", "Side Channel Air Boost", 0.0, 6.0, 1.5, "dB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_bool("phase_correlation_protect", "Auto Anti-Phase Limiter", true, (0, 230, 118)))
            ),
            "KSystemMeterNode" => Box::new(
                GenericDspNodeUi::new("KSystemMeterNode", "Bob Katz K-12 / K-14 / K-20 Metering Bridge", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_enum("meter_scale", "K-Scale Target Reference", vec!["K-12 (Broadcast / Pop)".into(), "K-14 (Standard Music)".into(), "K-20 (Audiophile / Film Dynamic)".into()], 1, (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_linear("headroom_target_db", "0 VU Calibration Offset", 0.0, 20.0, 14.0, "dB", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("peak_hold_time_ms", "Peak Hold Persistence", 100.0, 3000.0, 1500.0, "ms", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("monitor_calibration_db", "SPL Reference Calibration", 60.0, 90.0, 83.0, "dBSPL", (0, 230, 118)))
            ),
            "AuditoryRoughnessNode" => Box::new(
                GenericDspNodeUi::new("AuditoryRoughnessNode", "Psychoacoustic Plomp-Levelt Sensory Dissonance Analyzer", DspNodeCategory::SpectralResynthesis)
                    .with_param(DspParamDescriptor::new_linear("dissonance_threshold", "Roughness Detection Sens", 0.0, 1.0, 0.65, "%", (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_linear("roughness_scaling", "Plomp-Levelt Critical Scale", 0.1, 3.0, 1.0, "x", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_enum("critical_bandwidth_mode", "Filter Bank Standard", vec!["Bark Scale (Zwicker)".into(), "ERB Scale (Moore-Glasberg)".into(), "Mel Scale".into()], 1, (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("masking_depth", "Simultaneous Masking Depth", 0.0, 1.0, 0.5, "%", (0, 230, 118)))
            ),
            "EqualLoudnessContourNode" => Box::new(
                GenericDspNodeUi::new("EqualLoudnessContourNode", "Fletcher-Munson Psychoacoustic Equal Loudness Compensator", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("phon_level", "Listening Volume (Phon)", 30.0, 100.0, 80.0, "phon", (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_enum("curve_version", "Contour Standard", vec!["ISO 226:2003 Standard".into(), "Fletcher-Munson (1933)".into(), "Robinson-Dadson (1956)".into()], 0, (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("bass_contour_trim", "Low Contour Trim", -6.0, 6.0, 0.0, "dB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("treble_contour_trim", "High Contour Trim", -6.0, 6.0, 0.0, "dB", (0, 230, 118)))
            ),
            "MasterLimiterRadarNode" => Box::new(
                GenericDspNodeUi::new("MasterLimiterRadarNode", "True-Peak Limiter with Integrated Loudness Radar", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("true_peak_ceiling_dbtp", "True-Peak Ceiling", -6.0, 0.0, -0.3, "dBTP", (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_linear("target_integrated_lufs", "Target Integrated LUFS", -24.0, -8.0, -14.0, "LUFS", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("lookahead_time_ms", "True-Peak Lookahead", 1.0, 10.0, 4.0, "ms", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_bool("release_curve_adaptive", "Adaptive Program Release", true, (0, 230, 118)))
            ),
            "MeterBridgeNode" => Box::new(
                GenericDspNodeUi::new("MeterBridgeNode", "32-Channel VU, RMS, & True-Peak Mastering Meter Bridge", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_enum("meter_type", "Ballistics Standard", vec!["True-Peak ISP (ITU-R BS.1770)".into(), "VU Meter (ANSI C16.5)".into(), "Nordic PPM (IEC 60268-10)".into(), "BBC Type II PPM".into()], 0, (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_linear("integration_window_ms", "RMS Integration Window", 50.0, 1000.0, 300.0, "ms", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("vu_reference_db", "VU Zero Reference", -24.0, -10.0, -18.0, "dBFS", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("peak_decay_speed", "Decay Ballistics Rate", 5.0, 60.0, 20.0, "dB/s", (0, 230, 118)))
            ),
            "MidSideFocuserNode" => Box::new(
                GenericDspNodeUi::new("MidSideFocuserNode", "Surgical Mid/Side Bass Mono & High-Side Air Focuser", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_log("bass_mono_freq_hz", "Mono Bass Frequency Cut", 30.0, 300.0, 110.0, "Hz", (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_linear("high_side_boost_db", "Side Channel Air Sheen", 0.0, 6.0, 1.8, "dB", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("side_dynamic_compression", "Side Band Dynamic Tame", 0.0, 1.0, 0.4, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("mid_presence_gain_db", "Mid Channel Vocal Presence", -6.0, 6.0, 0.8, "dB", (0, 230, 118)))
            ),
            "MultibandImagerNode" => Box::new(
                GenericDspNodeUi::new("MultibandImagerNode", "4-Band Mastering Stereo Width & Phase Correlator", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("band1_sub_width", "Sub Width (<120Hz)", 0.0, 2.0, 0.0, "x", (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_linear("band2_low_width", "Low-Mid Width (120-1kHz)", 0.0, 2.0, 0.9, "x", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("band3_mid_width", "High-Mid Width (1k-6kHz)", 0.0, 2.0, 1.25, "x", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("band4_high_width", "Air Width (>6kHz)", 0.0, 2.0, 1.4, "x", (0, 230, 118)))
            ),
            "OversampledLimiterNode" => Box::new(
                GenericDspNodeUi::new("OversampledLimiterNode", "16x Polyphase Oversampled Intersample Peak Limiter", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_enum("oversampling_rate", "Polyphase Oversampling", vec!["2x Polyphase".into(), "4x High Quality".into(), "8x Mastering Grade".into(), "16x Ultra Fidelity".into()], 2, (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_linear("brickwall_ceiling_dbfs", "Ceiling Level", -6.0, 0.0, -0.2, "dBFS", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("release_time_ms", "Lookahead Release Speed", 5.0, 400.0, 45.0, "ms", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_bool("isp_guard_active", "Intersample Peak Guard", true, (0, 230, 118)))
            ),
            "PolarPhaseCorrelatorNode" => Box::new(
                GenericDspNodeUi::new("PolarPhaseCorrelatorNode", "Polar Goniometer Lissajous Phase Correlation Display", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("phosphor_persistence_ms", "Goniometer Phosphor Decay", 50.0, 1000.0, 300.0, "ms", (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_linear("polar_scaling", "Polar Radius Scale", 0.5, 3.0, 1.0, "x", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_int("lissajous_resolution", "Display Point Resolution", 128, 2048, 512, "pts", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_bool("phase_cancel_alert", "Phase Inversion Warning", true, (255, 61, 113)))
            ),
            "ResonanceSuppressorNode" => Box::new(
                GenericDspNodeUi::new("ResonanceSuppressorNode", "Automatic Dynamic Harmonic Notch Resonance Suppressor", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("suppression_depth_db", "Dynamic Notch Depth", 0.0, 18.0, 6.0, "dB", (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_linear("notch_q_sharpness", "Dynamic Q Sharpness", 2.0, 50.0, 15.0, "Q", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("attack_speed_ms", "Resonance Lock Attack", 1.0, 100.0, 12.0, "ms", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_int("max_active_notches", "Concurrent Notch Filters", 1, 16, 6, "filters", (0, 230, 118)))
            ),
            "SpectralAlignerNode" => Box::new(
                GenericDspNodeUi::new("SpectralAlignerNode", "Multi-Track Phase & Spectral Coherence Auto-Aligner", DspNodeCategory::SpectralResynthesis)
                    .with_param(DspParamDescriptor::new_int("reference_track_id", "Reference Master Track ID", 0, 64, 0, "trk", (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_linear("phase_correlation_target", "Target Phase Alignment", 0.5, 1.0, 0.95, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("delay_offset_samples", "Sub-Sample Delay Offset", -500.0, 500.0, 0.0, "samples", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_bool("polarity_auto_invert", "Auto Polarity Inversion", true, (0, 230, 118)))
            ),
            "SpectralDebleedNode" => Box::new(
                GenericDspNodeUi::new("SpectralDebleedNode", "Microphone Acoustic Bleed Elimination Matrix", DspNodeCategory::SpectralResynthesis)
                    .with_param(DspParamDescriptor::new_linear("bleed_suppression_db", "Bleed Isolation Depth", 0.0, 30.0, 14.0, "dB", (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_linear("spectral_gate_threshold", "Spectral Gate Floor", -60.0, -10.0, -35.0, "dB", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("crosstalk_estimate_ms", "Acoustic Delay Estimate", 0.1, 50.0, 8.5, "ms", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_bool("transient_preserve", "Preserve Direct Transients", true, (0, 230, 118)))
            ),
            "SpectralDeEsserNode" => Box::new(
                GenericDspNodeUi::new("SpectralDeEsserNode", "Dynamic Spectral Sibilance & Harshness Reducer", DspNodeCategory::SpectralResynthesis)
                    .with_param(DspParamDescriptor::new_log("sibilance_center_hz", "Harshness Center Freq", 3000.0, 14000.0, 7200.0, "Hz", (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_linear("bandwidth_oct", "Sibilance Bandwidth", 0.2, 3.0, 1.2, "oct", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("max_reduction_db", "Max Dynamic Reduction", 0.0, 24.0, 9.0, "dB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("lookahead_ms", "Lookahead Buffer Time", 0.0, 10.0, 2.5, "ms", (0, 230, 118)))
            ),
            "SpectralFlatnessNode" => Box::new(
                GenericDspNodeUi::new("SpectralFlatnessNode", "Wiener Entropy Spectral Flatness & Tonal/Noise Analyzer", DspNodeCategory::SpectralResynthesis)
                    .with_param(DspParamDescriptor::new_linear("entropy_threshold", "Spectral Flatness Cutoff", 0.0, 1.0, 0.45, "entropy", (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_int("analysis_window_size", "FFT Window Analysis Size", 256, 4096, 1024, "bins", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("tonal_split_gain", "Tonal Harmonic Level", -12.0, 12.0, 0.0, "dB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("noise_split_gain", "Noise Residual Level", -12.0, 12.0, 0.0, "dB", (0, 230, 118)))
            ),
            "SpectralGrainCloudNode" => Box::new(
                GenericDspNodeUi::new("SpectralGrainCloudNode", "Spectral Domain Grain Scatter & Diffusion Engine", DspNodeCategory::SpectralResynthesis)
                    .with_param(DspParamDescriptor::new_linear("grain_density_spectral", "Frequency Bin Grains", 10.0, 500.0, 120.0, "grains/s", (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_linear("time_smear_factor", "Phase Time Smear", 0.0, 1.0, 0.35, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("phase_randomization", "Bin Phase Chaos", 0.0, 1.0, 0.4, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("diffusion_wet_mix", "Grain Cloud Blend", 0.0, 1.0, 0.3, "%", (0, 230, 118)))
            ),
            "SpectralMaskingNode" => Box::new(
                GenericDspNodeUi::new("SpectralMaskingNode", "Psychoacoustic Simultaneous Spectral Masking Visualizer", DspNodeCategory::SpectralResynthesis)
                    .with_param(DspParamDescriptor::new_linear("masking_threshold_offset_db", "Masking Threshold Bias", -12.0, 12.0, 0.0, "dB", (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_linear("critical_band_scale", "Critical Band Width Scale", 0.5, 2.0, 1.0, "x", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_enum("bark_scale_mode", "Psychoacoustic Standard", vec!["Bark Scale Matrix".into(), "ERB Auditory Filter".into(), "Equivalent Rectangular Band".into()], 0, (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("alert_color_intensity", "Masking Flare Brightness", 0.0, 1.0, 0.85, "%", (0, 230, 118)))
            ),
            "SpectralMorphNode" => Box::new(
                GenericDspNodeUi::new("SpectralMorphNode", "Latent FFT Magnitude & Phase Interpolation Morph Engine", DspNodeCategory::SpectralResynthesis)
                    .with_param(DspParamDescriptor::new_linear("morph_position", "Spectral Morph Interpolation", 0.0, 1.0, 0.5, "%", (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_linear("spectral_smear", "Cross-Bin Spectral Smear", 0.0, 1.0, 0.2, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_enum("phase_alignment_mode", "Phase Interpolation Mode", vec!["Magnitude Only (Zero Phase)".into(), "Phase Unwrapped Morph".into(), "Cross-Correlation Aligned".into()], 1, (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_bool("harmonic_lock", "Lock Harmonic Partials", true, (0, 230, 118)))
            ),
            "SpectralReshaperNode" => Box::new(
                GenericDspNodeUi::new("SpectralReshaperNode", "Dynamic Spectral Envelope & Formant Reshaper", DspNodeCategory::SpectralResynthesis)
                    .with_param(DspParamDescriptor::new_linear("formant_shift_semitones", "Spectral Envelope Shift", -24.0, 24.0, 0.0, "st", (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_linear("spectral_envelope_gain", "Envelope Reshape Gain", -18.0, 18.0, 0.0, "dB", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("smoothing_factor", "Envelope Cepstral Smoothing", 0.1, 5.0, 1.2, "oct", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_bool("dynamic_tracking", "Dynamic Pitch Tracking", true, (0, 230, 118)))
            ),
            "SpectralResynthesisNode" => Box::new(
                GenericDspNodeUi::new("SpectralResynthesisNode", "Harmonic Additive Spectral Resynthesis Engine", DspNodeCategory::SpectralResynthesis)
                    .with_param(DspParamDescriptor::new_int("max_partials_count", "Additive Sinusoidal Partials", 8, 256, 64, "partials", (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_linear("spectral_threshold_db", "Noise Floor Cutoff", -80.0, -20.0, -50.0, "dB", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("inharmonic_drift", "Partial Inharmonicity", 0.0, 2.0, 0.0, "", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("harmonic_additive_mix", "Synthesized Partials Level", 0.0, 1.0, 0.8, "%", (0, 230, 118)))
            ),
            "SpectralUnmaskerNode" => Box::new(
                GenericDspNodeUi::new("SpectralUnmaskerNode", "AI-Driven Intelligent Sidechain Spectral Unmasker", DspNodeCategory::SpectralResynthesis)
                    .with_param(DspParamDescriptor::new_int("sidechain_input_id", "Sidechain Trigger Track", 0, 64, 1, "trk", (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_linear("unmasking_depth_db", "Dynamic Unmask Depth", 0.0, 18.0, 5.0, "dB", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_int("frequency_resolution_bins", "Spectral Bin Resolution", 32, 512, 128, "bins", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("adaptation_speed_ms", "Fast Reaction Speed", 5.0, 250.0, 35.0, "ms", (0, 230, 118)))
            ),
            "Spectrogram3DNode" => Box::new(
                GenericDspNodeUi::new("Spectrogram3DNode", "3D Waterfall Real-Time Spectrogram & Sonogram Analyzer", DspNodeCategory::SpectralResynthesis)
                    .with_param(DspParamDescriptor::new_int("fft_size_bins", "FFT Analysis Size", 256, 4096, 1024, "bins", (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_int("waterfall_depth_frames", "Waterfall History Buffer", 30, 300, 120, "frames", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_enum("color_palette", "Spectrogram Palette", vec!["Cyberpunk Neon".into(), "Thermal Infrared".into(), "Obsidian Ice".into(), "Grayscale Contrast".into()], 0, (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_bool("frequency_scale_log", "Logarithmic Frequency Grid", true, (0, 230, 118)))
            ),
            "StereoVectorscopeNode" => Box::new(
                GenericDspNodeUi::new("StereoVectorscopeNode", "High-Speed Phosphor Stereophonic Vectorscope Analyzer", DspNodeCategory::SpectralResynthesis)
                    .with_param(DspParamDescriptor::new_linear("phosphor_decay_rate", "Phosphor Glow Persistence", 0.05, 1.0, 0.35, "s", (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_linear("vectorscope_gain", "Lissajous Input Scale", 0.5, 4.0, 1.0, "x", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("lissajous_dot_size", "Vector Trace Thickness", 1.0, 5.0, 1.8, "pt", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_bool("correlation_meter_bar", "Show Phase Bar Meter", true, (0, 230, 118)))
            ),
            "StereoWidenerNode" => Box::new(
                GenericDspNodeUi::new("StereoWidenerNode", "Haas Effect & Phase-Coherent Stereo Field Widener", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("haas_delay_ms", "Haas Delay Offset", 0.1, 30.0, 12.0, "ms", (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_linear("mid_side_ratio", "Side vs Mid Ratio", 0.0, 3.0, 1.25, "x", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("center_channel_retention", "Center Lead Vocal Retain", 0.0, 1.0, 0.85, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("phase_filter_q", "Allpass Phase Filter Q", 0.5, 5.0, 1.2, "Q", (0, 230, 118)))
            ),
            "TapeFluxMasterNode" => Box::new(
                GenericDspNodeUi::new("TapeFluxMasterNode", "Half-Inch Mastering Tape Saturation & Flux Density Engine", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("tape_flux_nanoweber", "Reference Fluxivity", 185.0, 510.0, 355.0, "nWb/m", (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_enum("tape_formulation", "Mastering Tape Stock", vec!["SM900 High Output (+9dB)".into(), "ATR Magnetics Master (+6dB)".into(), "GP9 Grand Master (+9dB)".into(), "Scotch 250 Vintage (+3dB)".into()], 0, (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("bias_hf_saturation", "High-Frequency Bias Saturation", -3.0, 6.0, 1.5, "dB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("flux_compression_db", "Head Hysteresis Compression", 0.0, 6.0, 1.8, "dB", (0, 230, 118)))
            ),
            "DialogGatingNode" => Box::new(
                GenericDspNodeUi::new("DialogGatingNode", "ITU-R BS.1770 Dialogue-Aware Gating Loudness Engine", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("speech_presence_threshold_db", "Speech Presence Gate", -40.0, -10.0, -24.0, "dB", (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_linear("gating_window_ms", "Voice Energy Gating Window", 100.0, 1000.0, 400.0, "ms", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("ambient_noise_attenuation_db", "Background Attenuation", -30.0, 0.0, -12.0, "dB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("voice_clarity_lift_db", "Formant Intelligibility Lift", 0.0, 6.0, 2.0, "dB", (0, 230, 118)))
            ),
            "SonarHydrophoneNode" => Box::new(
                GenericDspNodeUi::new("SonarHydrophoneNode", "Underwater Acoustic Hydrophone & Thermal Layer Simulation", DspNodeCategory::DynamicsMaster)
                    .with_param(DspParamDescriptor::new_linear("hydrophone_depth_m", "Sensor Water Depth", 1.0, 1000.0, 75.0, "m", (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_linear("water_salinity_ppt", "Ocean Salinity", 0.0, 45.0, 35.0, "ppt", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("sound_channel_axis_depth_m", "SOFAR Channel Axis Depth", 100.0, 1500.0, 800.0, "m", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("cavitation_noise_level", "Propeller Cavitation Ambient", 0.0, 1.0, 0.2, "%", (0, 230, 118)))
            ),
            "BciEegDecoderNode" => Box::new(
                GenericDspNodeUi::new("BciEegDecoderNode", "10-20 EEG Brainwave Band Power & Alpha/Theta Decoder", DspNodeCategory::NeuralAi)
                    .with_param(DspParamDescriptor::new_linear("sampling_rate_hz", "EEG Electrode Sample Rate", 128.0, 1024.0, 256.0, "Hz", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("alpha_band_gain", "Alpha Power Band (8-12Hz)", 0.0, 2.0, 1.0, "x", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_linear("theta_band_gain", "Theta Power Band (4-8Hz)", 0.0, 2.0, 0.8, "x", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_bool("artifact_rejection_filter", "Ocular Blink Artifact Filter", true, (255, 171, 0)))
            ),
            "NeuroAffectiveEmotionalStateAnalyzer" => Box::new(
                GenericDspNodeUi::new("NeuroAffectiveEmotionalStateAnalyzer", "Russell Circumplex Valence & Arousal Emotion Analyzer", DspNodeCategory::NeuralAi)
                    .with_param(DspParamDescriptor::new_linear("valence_score", "Musical Valence (Pleasantness)", -1.0, 1.0, 0.55, "score", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("arousal_intensity", "Musical Arousal (Energy)", 0.0, 1.0, 0.65, "score", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_linear("emotional_smoothing_window", "Affective Slew Smoothing", 0.1, 10.0, 2.0, "s", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("harmony_feedback_gain", "Emotional Harmony Influence", 0.0, 1.0, 0.5, "%", (0, 230, 118)))
            ),
            "NeuralImpulseResponseSynthesizer" => Box::new(
                GenericDspNodeUi::new("NeuralImpulseResponseSynthesizer", "Latent Space Acoustic Impulse Response Generator", DspNodeCategory::NeuralAi)
                    .with_param(DspParamDescriptor::new_linear("latent_room_dimension_x", "Latent Room Geometry X", -3.0, 3.0, 0.5, "", (101, 84, 171)))
                    .with_param(DspParamDescriptor::new_linear("latent_absorption_y", "Latent Wall Material Y", -3.0, 3.0, -0.2, "", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("early_reflection_decay", "Early Diffusion Slew", 0.1, 5.0, 1.5, "s", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_int("synthesis_quality", "Neural Inference Latent Depth", 1, 4, 3, "steps", (0, 230, 118)))
            ),
            "SubcorticalBrainstemPitchTracker" => Box::new(
                GenericDspNodeUi::new("SubcorticalBrainstemPitchTracker", "Auditory Nerve Frequency-Following Response Pitch Tracker", DspNodeCategory::NeuralAi)
                    .with_param(DspParamDescriptor::new_linear("ffr_sensitivity", "Brainstem FFR Sensitivity", 0.1, 2.0, 1.0, "x", (0, 184, 217)))
                    .with_param(DspParamDescriptor::new_int("cochlear_filterbank_channels", "Cochlear Gammatone Channels", 16, 128, 64, "ch", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_log("fundamental_f0_estimate_hz", "Detected Pitch F0", 30.0, 2000.0, 220.0, "Hz", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("tracking_slew_ms", "Pitch Tracking Response", 1.0, 100.0, 15.0, "ms", (0, 230, 118)))
            ),
            "MentalImageryPatternClassifier" => Box::new(
                GenericDspNodeUi::new("MentalImageryPatternClassifier", "Motor/Auditory Imagery Neural Intention Classifier", DspNodeCategory::NeuralAi)
                    .with_param(DspParamDescriptor::new_linear("classifier_confidence_threshold", "Trigger Confidence Floor", 0.5, 0.99, 0.85, "%", (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_int("pattern_bank_index", "Intention Pattern Slot", 0, 15, 0, "slot", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("trigger_event_latency_ms", "Pattern Recognition Latency", 5.0, 200.0, 35.0, "ms", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("adaptation_rate", "Online Neural Plasticity Slew", 0.0, 1.0, 0.1, "%", (0, 230, 118)))
            ),
            "BiometricHrvTempoSync" => Box::new(
                GenericDspNodeUi::new("BiometricHrvTempoSync", "Photoplethysmography Heart Rate Variability Tempo Sync", DspNodeCategory::NeuralAi)
                    .with_param(DspParamDescriptor::new_linear("target_heart_coherence", "Target HRV Coherence", 0.0, 1.0, 0.75, "%", (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_linear("bpm_sync_multiplier", "BPM Metric Ratio", 0.5, 4.0, 1.0, "x", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("hrv_smoothing_interval_s", "Heart Rate Rolling Average", 5.0, 60.0, 20.0, "s", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("rhythm_entrainment_depth", "Tempo Entrainment Depth", 0.0, 1.0, 0.5, "%", (0, 230, 118)))
            ),
            "NeuroCognitiveFatigueDetector" => Box::new(
                GenericDspNodeUi::new("NeuroCognitiveFatigueDetector", "Listening Fatigue & Auditory Habituation Detector", DspNodeCategory::NeuralAi)
                    .with_param(DspParamDescriptor::new_linear("listening_duration_minutes", "Session Exposure Time", 0.0, 480.0, 65.0, "min", (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_linear("spectral_brightness_fatigue_index", "Harshness Fatigue Metric", 0.0, 1.0, 0.4, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("auto_soft_filter_db", "Fatigue Relief Soft High Cut", -6.0, 0.0, 0.0, "dB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_bool("habituation_warning", "Break Reminder Notification", true, (0, 230, 118)))
            ),
            "NeuroAestheticHarmonyScorer" => Box::new(
                GenericDspNodeUi::new("NeuroAestheticHarmonyScorer", "Computational Neuro-Aesthetic Musical Harmony Scorer", DspNodeCategory::NeuralAi)
                    .with_param(DspParamDescriptor::new_linear("harmony_concordance_weight", "Concordance Sensation Bias", 0.0, 1.0, 0.8, "%", (255, 51, 153)))
                    .with_param(DspParamDescriptor::new_linear("voice_leading_fluency", "Voice Leading Smoothness", 0.0, 1.0, 0.7, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("tension_resolution_score", "Tension Cycle Resolution", 0.0, 1.0, 0.85, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("scale_fit_percent", "Tonal Scale Alignment", 0.0, 100.0, 94.0, "%", (0, 230, 118)))
            ),
            "SubsensoryTactileHapticTransducer" => Box::new(
                GenericDspNodeUi::new("SubsensoryTactileHapticTransducer", "Vibrotactile Sub-Bass Haptic Transducer Driver", DspNodeCategory::NeuralAi)
                    .with_param(DspParamDescriptor::new_log("haptic_frequency_hz", "Transducer Resonance Pitch", 15.0, 120.0, 40.0, "Hz", (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_linear("transducer_intensity", "Haptic Shaker Force", 0.0, 1.0, 0.75, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_enum("subharmonic_synthesis_oct", "Sub-Harmonic Generation", vec!["None (Direct)".into(), "-1 Octave Sub".into(), "-2 Octave Infrasound".into()], 1, (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("rumble_envelope_decay", "Haptic Rumble Decay", 0.05, 2.0, 0.4, "s", (0, 230, 118)))
            ),
            "NeuralChoirFormantNode" => Box::new(
                GenericDspNodeUi::new("NeuralChoirFormantNode", "Multi-Voice Neural Formant Choir & Vowel Morph Engine", DspNodeCategory::NeuralAi)
                    .with_param(DspParamDescriptor::new_int("voice_count", "Ensemble Choir Voices", 4, 32, 16, "voices", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("vowel_morph_coordinate", "Vowel Space Morph (A-E-I-O-U)", 0.0, 4.0, 1.5, "vowel", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("formant_spread_cents", "Formant Detuning Width", 0.0, 50.0, 15.0, "cents", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("choir_detune_cents", "Choral Micro-Pitch Detune", 0.0, 40.0, 8.0, "cents", (0, 230, 118)))
            ),
            "NeuralDereverbNode" => Box::new(
                GenericDspNodeUi::new("NeuralDereverbNode", "Deep Learning Direct-to-Reverberant Ratio Blind De-Reverb", DspNodeCategory::NeuralAi)
                    .with_param(DspParamDescriptor::new_linear("dereverb_intensity", "De-Reverberation Depth", 0.0, 100.0, 65.0, "%", (101, 84, 171)))
                    .with_param(DspParamDescriptor::new_linear("room_size_estimate_m", "Estimated Chamber Size", 2.0, 50.0, 12.0, "m", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("early_reflection_suppress_db", "Early Reflection Attenuation", -24.0, 0.0, -10.0, "dB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("tail_dry_boost_db", "Direct Signal Presence Boost", 0.0, 12.0, 2.5, "dB", (0, 230, 118)))
            ),
            "NeuralInpaintNode" => Box::new(
                GenericDspNodeUi::new("NeuralInpaintNode", "Neural Audio Inpainting & Dropout Reconstruction Engine", DspNodeCategory::NeuralAi)
                    .with_param(DspParamDescriptor::new_linear("dropout_threshold_ms", "Dropout Mask Detection Window", 1.0, 100.0, 20.0, "ms", (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_linear("reconstruction_depth", "AI Inpainting Strength", 0.0, 1.0, 0.85, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_int("context_window_ms", "Surrounding Context Frame", 50, 1000, 250, "ms", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_bool("spectral_continuity_protect", "Preserve Spectral Phase", true, (0, 230, 118)))
            ),
            "NeuralPhonemeNode" => Box::new(
                GenericDspNodeUi::new("NeuralPhonemeNode", "International Phonetic Alphabet Differentiable Phoneme Synth", DspNodeCategory::NeuralAi)
                    .with_param(DspParamDescriptor::new_int("ipa_symbol_index", "IPA Phoneme Symbol Index", 0, 64, 12, "ipa", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("nasalization_index", "Velum Nasal Coupling", 0.0, 1.0, 0.2, "%", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("voicing_glottal_mix", "Voiced Glottal Mix", 0.0, 1.0, 0.8, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("tongue_arch_position", "Tongue Constriction Place", 0.0, 1.0, 0.5, "%", (0, 230, 118)))
            ),
            "NeuralRadianceNode" => Box::new(
                GenericDspNodeUi::new("NeuralRadianceNode", "Neural Acoustic Radiance Field (NeRF) 3D Room Acoustic Field", DspNodeCategory::NeuralAi)
                    .with_param(DspParamDescriptor::new_linear("listener_pos_xyz", "Virtual Listener X Position", -10.0, 10.0, 0.0, "m", (101, 84, 171)))
                    .with_param(DspParamDescriptor::new_linear("emitter_pos_xyz", "Virtual Emitter Y Position", -10.0, 10.0, 2.5, "m", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_int("nerf_rendering_samples", "Ray Sample Marching Steps", 32, 256, 128, "rays", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("spatial_interpolation_slew", "Acoustic Field Slew Rate", 0.01, 1.0, 0.15, "s", (0, 230, 118)))
            ),
            "NeuralSpeechToSingingNode" => Box::new(
                GenericDspNodeUi::new("NeuralSpeechToSingingNode", "Speech-to-Singing AI Prosody & Intonation Transformer", DspNodeCategory::NeuralAi)
                    .with_param(DspParamDescriptor::new_bool("target_pitch_quantize", "Snap Speech to Scale Notes", true, (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("vibrato_infusion_depth", "Singing Vibrato Infusion", 0.0, 1.0, 0.5, "%", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("note_hold_extension_ms", "Vowel Duration Extension", 50.0, 1000.0, 350.0, "ms", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_enum("singing_style_preset", "AI Singing Style", vec!["Bel Canto Opera".into(), "Pop Vibrato Smooth".into(), "R&B Micro-Riff".into(), "Choir Polyphony".into()], 1, (0, 230, 118)))
            ),
            "NeuralTimbreMorphNode" => Box::new(
                GenericDspNodeUi::new("NeuralTimbreMorphNode", "Non-Linear Latent Timbre Vector Interpolator", DspNodeCategory::NeuralAi)
                    .with_param(DspParamDescriptor::new_linear("timbre_vector_x", "Latent Timbre Axis X (Warm/Bright)", -3.0, 3.0, 0.0, "", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("timbre_vector_y", "Latent Timbre Axis Y (Wood/Metal)", -3.0, 3.0, 0.0, "", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("timbre_vector_z", "Latent Timbre Axis Z (Acoustic/Synth)", -3.0, 3.0, 0.0, "", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("morph_trajectory_speed_hz", "LFO Timbre Orbit Speed", 0.05, 10.0, 0.5, "Hz", (0, 230, 118)))
            ),
            "NeuralTimbreNode" => Box::new(
                GenericDspNodeUi::new("NeuralTimbreNode", "Latent Timbre Space Feature Extractor & Profiler", DspNodeCategory::NeuralAi)
                    .with_param(DspParamDescriptor::new_linear("spectral_centroid_weight", "Spectral Brightness Weight", 0.0, 1.0, 0.7, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("flux_roughness_weight", "Spectral Roughness Weight", 0.0, 1.0, 0.4, "%", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("harmonic_entropy_weight", "Harmonic Order vs Noise", 0.0, 1.0, 0.85, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_bool("timbre_snapshot_trigger", "Capture Timbre Vector", false, (0, 230, 118)))
            ),
            "NeuralVocalStylizerNode" => Box::new(
                GenericDspNodeUi::new("NeuralVocalStylizerNode", "Vocal Timbre Style Transfer & Character Transformer", DspNodeCategory::NeuralAi)
                    .with_param(DspParamDescriptor::new_int("source_vocal_id", "Source Vocal Track ID", 0, 32, 1, "trk", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_enum("target_style_preset", "AI Timbre Target Profile", vec!["Vintage Ribbon Mic Warmth".into(), "Modern Hyper-Pop Crisp".into(), "Analog Tube Intimate".into(), "Classic 80s Broadcast".into()], 0, (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("formant_character_scale", "Formant Character Intensity", 0.5, 2.0, 1.0, "x", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("style_transfer_blend", "Style Transfer Dry/Wet Blend", 0.0, 1.0, 0.75, "%", (0, 230, 118)))
            ),
            "NeuralVocoderMorphNode" => Box::new(
                GenericDspNodeUi::new("NeuralVocoderMorphNode", "Neural HiFi-GAN / BigVGAN Vocoder Latent Morpher", DspNodeCategory::NeuralAi)
                    .with_param(DspParamDescriptor::new_enum("vocoder_architecture", "Neural Vocoder Backbone", vec!["BigVGAN High-Fidelity".into(), "HiFi-GAN V1".into(), "Vocos Ultra-Fast".into(), "WaveGlow Multi-Scale".into()], 0, (0, 184, 217)))
                    .with_param(DspParamDescriptor::new_linear("latent_dimension_scale", "Latent Manifold Scale", 0.1, 3.0, 1.0, "x", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("periodic_component_mix", "Harmonic Periodic Component", 0.0, 1.0, 0.8, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("aperiodic_noise_ratio", "Aperiodic Breath Noise Level", 0.0, 1.0, 0.2, "%", (0, 230, 118)))
            ),
            "NeuralQuantumAnnealerTopologicalSort" => Box::new(
                GenericDspNodeUi::new("NeuralQuantumAnnealerTopologicalSort", "Simulated Quantum Annealing Audio Graph Optimizer", DspNodeCategory::NeuralAi)
                    .with_param(DspParamDescriptor::new_linear("annealing_temperature", "Annealing Temperature (T)", 0.01, 10.0, 1.0, "T", (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_linear("transverse_field_gamma", "Transverse Quantum Field", 0.0, 5.0, 1.5, "gamma", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("graph_latency_minimize_weight", "Latency Minimization Weight", 0.0, 1.0, 0.9, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_bool("optimization_step_trigger", "Trigger Topological Sort", false, (0, 230, 118)))
            ),
            "GainNode" => Box::new(
                GenericDspNodeUi::new("GainNode", "Precision Unity Gain & Polarity Inverter", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_linear("gain_db", "Gain Level", -60.0, 18.0, 0.0, "dB", (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_bool("invert_phase", "Invert Phase (180 deg)", false, (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_bool("mute", "Channel Mute", false, (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("pan", "Stereo Balance Pan", -1.0, 1.0, 0.0, "pan", (0, 229, 255)))
            ),
            "ChromaticTunerNode" => Box::new(
                GenericDspNodeUi::new("ChromaticTunerNode", "Strobe Chromatic Instrument Pitch Tuner", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_linear("reference_pitch_hz", "Concert Pitch A4", 415.0, 466.0, 440.0, "Hz", (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_linear("cents_tolerance", "Strobe Sensitivity", 1.0, 20.0, 5.0, "cents", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_bool("mute_on_tune", "Mute Audio While Tuning", false, (255, 171, 0)))
            ),
            "SampleSlicerNode" => Box::new(
                GenericDspNodeUi::new("SampleSlicerNode", "Transient Beat Slicer & Loop Fragmenter", DspNodeCategory::SamplerSlicer)
                    .with_param(DspParamDescriptor::new_linear("threshold_db", "Transient Slicer Sensitivity", -60.0, 0.0, -24.0, "dB", (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_linear("min_slice_ms", "Minimum Slice Duration", 10.0, 500.0, 50.0, "ms", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_bool("bpm_sync", "Snap Slices to Beat Grid", true, (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_linear("transient_sensitivity", "Onset Detection Threshold", 0.0, 1.0, 0.7, "%", (255, 171, 0)))
            ),
            "LoopSlicerNode" => Box::new(
                GenericDspNodeUi::new("LoopSlicerNode", "Beat-Synchronized Grid Loop Slicer", DspNodeCategory::SamplerSlicer)
                    .with_param(DspParamDescriptor::new_enum("grid_division", "Beat Division", vec!["1/4 Beat".into(), "1/8 Beat".into(), "1/16 Beat".into(), "1/32 Beat".into(), "Triplet 1/8".into()], 2, (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_linear("crossfade_ms", "Slice Edge Crossfade", 0.0, 50.0, 5.0, "ms", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_bool("pitch_match", "Preserve Original Pitch", true, (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_bool("reverse_slices", "Reverse Alternate Slices", false, (255, 61, 113)))
            ),
            "HardwareCvNode" => Box::new(
                GenericDspNodeUi::new("HardwareCvNode", "Eurorack Modular Control Voltage & Gate Interface", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_int("cv_channel", "Hardware CV Channel", 1, 16, 1, "ch", (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_enum("voltage_range", "Output Voltage Range", vec!["0 to 10V (Unipolar)".into(), "-5 to +5V (Bipolar)".into(), "-10 to +10V (Modular Extended)".into()], 1, (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("gate_high_volts", "Gate On Threshold", 1.0, 10.0, 5.0, "V", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_linear("slew_time_ms", "Portamento Slew Time", 0.0, 100.0, 2.0, "ms", (255, 171, 0)))
            ),
            "ClapPluginHostNode" => Box::new(
                GenericDspNodeUi::new("ClapPluginHostNode", "CLAP/VST3 Audio Plugin Host Adapter", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_int("plugin_slot", "Hosted Plugin Slot", 0, 15, 0, "slot", (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_bool("bypass", "Plugin Bypass State", false, (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_linear("dry_wet", "Plugin Dry / Wet Mix", 0.0, 1.0, 1.0, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("latency_comp_ms", "PDC Latency Offset", 0.0, 50.0, 0.0, "ms", (0, 230, 118)))
            ),
            "Vst3HostNode" => Box::new(
                GenericDspNodeUi::new("Vst3HostNode", "VST3 Plugin Host with Embedded GUI & Automation", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_int("plugin_slot_index", "VST3 Plugin Slot", 0, 15, 0, "slot", (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_bool("bypass_plugin", "Host Bypass Plugin", false, (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_linear("dry_wet_mix", "Plugin Wet / Dry Blend", 0.0, 1.0, 1.0, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("latency_compensation_ms", "Plugin Delay Compensation", 0.0, 100.0, 0.0, "ms", (0, 230, 118)))
            ),
            "PushControllerDriverNode" => Box::new(
                GenericDspNodeUi::new("PushControllerDriverNode", "Ableton Push 2/3 High-Speed Display & RGB Pad Driver", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_int("display_frame_rate_fps", "LCD Frame Refresh Rate", 30, 120, 60, "fps", (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_linear("rgb_brightness_percent", "Pad Backlight Brightness", 10.0, 100.0, 85.0, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_enum("touch_strip_pitch_bend_mode", "Touch Strip Action", vec!["Pitch Bend Centered".into(), "Modulation Wheel".into(), "Expression CC11".into()], 0, (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("pad_sensitivity_curve", "Velocity Curve Exponent", 0.5, 3.0, 1.2, "exp", (0, 230, 118)))
            ),
            "LaunchpadDriverNode" => Box::new(
                GenericDspNodeUi::new("LaunchpadDriverNode", "Novation Launchpad Pro RGB Session & Step Sequencer Driver", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_bool("session_mode_active", "Session Clip Matrix Mode", true, (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_linear("velocity_response_curve", "Aftertouch Sensitivity", 0.1, 2.0, 1.0, "x", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_enum("led_color_palette", "RGB Color Palette", vec!["Novation Standard Palette".into(), "Vibrant DAW Matrix".into(), "Monochrome Studio".into()], 0, (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_int("midi_channel_out", "Controller Output Channel", 1, 16, 1, "ch", (0, 230, 118)))
            ),
            "KompleteKontrolDriverNode" => Box::new(
                GenericDspNodeUi::new("KompleteKontrolDriverNode", "Native Instruments NKS Light Guide & Encoder Driver", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_bool("light_guide_key_split", "Light Guide Key Split Colors", true, (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_bool("scale_guide_illumination", "Light Guide Scale Illumination", true, (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_enum("touch_strip_modulation_mode", "Modulation Strip Physics", vec!["Standard Free".into(), "Spring Return".into(), "Gravity Bounce".into()], 1, (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("encoder_sensitivity", "Rotary Encoder Slew Rate", 0.5, 3.0, 1.0, "x", (0, 230, 118)))
            ),
            "McuHardwareDriverNode" => Box::new(
                GenericDspNodeUi::new("McuHardwareDriverNode", "Mackie Control Universal 100mm Motorized Fader Surface", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_bool("motor_fader_touch_sense", "Touch-Sensitive Fader Motor", true, (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_enum("v_pot_led_ring_mode", "V-Pot LED Ring Mode", vec!["Single Dot Center".into(), "Boost/Cut Bar".into(), "Wrap Around Fill".into()], 1, (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("lcd_scribble_strip_contrast", "LCD Scribble Strip Contrast", 0.0, 100.0, 75.0, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_int("fader_bank_offset", "Fader Bank Track Offset", 0, 64, 0, "tracks", (0, 230, 118)))
            ),
            "OscControlMapperNode" => Box::new(
                GenericDspNodeUi::new("OscControlMapperNode", "Bidirectional Open Sound Control (OSC) Network Gateway", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_int("osc_udp_port_rx", "OSC Receiver UDP Port", 1024, 65535, 9000, "port", (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_int("osc_udp_port_tx", "OSC Transmitter UDP Port", 1024, 65535, 9001, "port", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_enum("ip_target_address", "Destination IP Host", vec!["127.0.0.1 (Localhost)".into(), "192.168.1.100 (LAN)".into(), "Broadcast (255.255.255.255)".into()], 0, (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("message_rate_limit_hz", "Max OSC Message Frequency", 10.0, 500.0, 120.0, "Hz", (0, 230, 118)))
            ),
            "MultiTrackAudioRouterNode" => Box::new(
                GenericDspNodeUi::new("MultiTrackAudioRouterNode", "64x64 Low-Latency Cross-Point Matrix Audio Router", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_int("source_channel_index", "Input Matrix Channel", 0, 63, 0, "in", (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_int("destination_bus_index", "Output Destination Bus", 0, 63, 0, "out", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("cross_point_gain_db", "Cross-Point Gain", -60.0, 12.0, 0.0, "dB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_bool("phase_invert_route", "Invert Route Polarity", false, (255, 61, 113)))
            ),
            "WasmDspRuntimeNode" => Box::new(
                GenericDspNodeUi::new("WasmDspRuntimeNode", "Sandboxed WebAssembly Wasm DSP Custom Kernel Host", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_int("wasm_kernel_memory_mb", "Sandbox Memory Allocation", 1, 64, 8, "MB", (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_int("instruction_budget_per_block", "Block Instruction Budget", 1000, 100000, 15000, "ops", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("execution_time_cap_us", "Hard Execution Watchdog", 10.0, 1000.0, 150.0, "us", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_bool("hot_reload_kernel", "Auto Hot-Reload Wasm Binary", true, (0, 230, 118)))
            ),
            "HardwareMidiClockJitterFilterNode" => Box::new(
                GenericDspNodeUi::new("HardwareMidiClockJitterFilterNode", "Sub-Microsecond Phase-Locked Loop MIDI Clock Sync", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_linear("pll_filter_damping", "PLL Phase Filter Damping", 0.1, 1.0, 0.707, "Q", (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_linear("jitter_tolerance_us", "Clock Jitter Window", 1.0, 500.0, 25.0, "us", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_int("clock_ppqn_multiplier", "Clock PPQN Multiplier", 24, 96, 24, "ppqn", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("hardware_latency_offset_ms", "Hardware Roundtrip Offset", -50.0, 50.0, 0.0, "ms", (0, 230, 118)))
            ),
            "PluginSandboxScannerNode" => Box::new(
                GenericDspNodeUi::new("PluginSandboxScannerNode", "Out-of-Process Crash-Proof Plugin Sandboxing & Scanner", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_bool("isolation_process_mode", "Out-of-Process Sandboxing", true, (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_linear("sandbox_timeout_ms", "Plugin Init Timeout", 500.0, 10000.0, 3000.0, "ms", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_int("ipc_shared_memory_mb", "IPC Ring Buffer Size", 2, 32, 8, "MB", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_bool("crash_auto_recover", "Auto-Restart Crashed Plugin", true, (0, 230, 118)))
            ),
            "PluginParamAutomapEngineNode" => Box::new(
                GenericDspNodeUi::new("PluginParamAutomapEngineNode", "AI Semantic Parameter Auto-Mapping & Grouping Engine", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_enum("parameter_semantic_model", "AI Grouping Semantic Model", vec!["Synthesizer 8-Macro Standard".into(), "Channel Strip EQ/Dynamics".into(), "Spatial 3D Soundfield".into(), "Mastering Chain".into()], 0, (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_int("macro_cluster_group_count", "Auto-Generated Macro Groups", 2, 8, 4, "macros", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("ai_grouping_confidence", "Semantic Confidence Threshold", 0.5, 1.0, 0.85, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_bool("remap_triggers", "Auto-Learn Controller Hardware", true, (0, 230, 118)))
            ),
            "CvGateSignalGeneratorNode" => Box::new(
                GenericDspNodeUi::new("CvGateSignalGeneratorNode", "Modular Synth 1V/Oct CV Pitch & Gate Pulse Generator", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_linear("cv_voltage_output_volts", "1V/Oct Pitch Voltage", -5.0, 10.0, 0.0, "V", (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_linear("gate_pulse_width_ms", "Gate Pulse Duration", 1.0, 200.0, 25.0, "ms", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("gate_voltage_high_v", "Gate High Active Level", 1.0, 10.0, 5.0, "V", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_linear("portamento_slew_ms", "Portamento Glide Slew", 0.0, 500.0, 0.0, "ms", (255, 171, 0)))
            ),
            "DinSync24PulseGeneratorNode" => Box::new(
                GenericDspNodeUi::new("DinSync24PulseGeneratorNode", "Roland DIN Sync 24 PPQN Hardware Clock Generator", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_int("din_sync_ppqn", "DIN Sync Pulse Resolution", 24, 48, 24, "ppqn", (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_bool("run_stop_gate_active", "DIN Run / Stop Voltage High", true, (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_linear("pulse_width_duty_cycle", "Clock Duty Cycle", 0.1, 0.9, 0.5, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("swing_shuffle_percent", "Hardware Swing Timing", 50.0, 75.0, 50.0, "%", (255, 171, 0)))
            ),
            "BleMidiControllerDriverNode" => Box::new(
                GenericDspNodeUi::new("BleMidiControllerDriverNode", "Bluetooth Low Energy (BLE-MIDI) Low-Latency Driver", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_linear("ble_connection_interval_ms", "BLE Connection Latency", 7.5, 30.0, 11.25, "ms", (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_linear("packet_timestamp_align_us", "Timestamp Jitter Alignment", 10.0, 200.0, 50.0, "us", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("signal_rssi_indicator_dbm", "Wireless Signal RSSI", -100.0, -30.0, -55.0, "dBm", (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_bool("auto_reconnect", "Auto-Reconnect Paired Device", true, (255, 171, 0)))
            ),
            "SupercriticalFluidNoiseNode" => Box::new(
                GenericDspNodeUi::new("SupercriticalFluidNoiseNode", "Supercritical Phase-Transition Acoustic Noise Generator", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_linear("fluid_critical_temp_k", "Critical Temperature (Tc)", 250.0, 600.0, 304.13, "K", (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_linear("fluid_critical_pressure_bar", "Critical Pressure (Pc)", 30.0, 150.0, 73.75, "bar", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("density_fluctuation_amplitude", "Phase Density Fluctuations", 0.0, 1.0, 0.7, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("output_noise_gain", "Acoustic Noise Output", 0.0, 1.0, 0.6, "%", (0, 230, 118)))
            ),
            "SonoluminescenceSonifierNode" => Box::new(
                GenericDspNodeUi::new("SonoluminescenceSonifierNode", "Ultrasonic Acoustic Cavitation Bubble Sonifier", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_log("ultrasound_drive_freq_khz", "Ultrasound Driver Frequency", 20.0, 100.0, 26.5, "kHz", (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_linear("cavitation_acoustic_pressure_atm", "Acoustic Drive Pressure", 0.5, 5.0, 1.4, "atm", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("bubble_flash_intensity", "Picosecond Light Flash Energy", 0.0, 1.0, 0.75, "%", (255, 61, 113)))
                    .with_param(DspParamDescriptor::new_linear("sparkle_audio_mix", "Cavitation Sparkle Level", 0.0, 1.0, 0.5, "%", (0, 230, 118)))
            ),
            "GravitationalWaveChirpNode" => Box::new(
                GenericDspNodeUi::new("GravitationalWaveChirpNode", "Binary Black Hole Inspiral Gravitational Wave Chirp", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_linear("component_mass_m1_solar", "Black Hole Primary Mass (M1)", 5.0, 100.0, 36.0, "Msun", (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_linear("component_mass_m2_solar", "Black Hole Secondary Mass (M2)", 5.0, 100.0, 29.0, "Msun", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("coalescence_time_s", "Inspiral Coalescence Time", 0.1, 10.0, 1.5, "s", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("strain_amplitude_h", "Gravitational Wave Strain H", 0.0, 1.0, 0.85, "%", (0, 230, 118)))
            ),
            "CasimirVacuumNoiseNode" => Box::new(
                GenericDspNodeUi::new("CasimirVacuumNoiseNode", "Quantum Vacuum Zero-Point Energy Fluctuation Noise", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_linear("plate_separation_distance_nm", "Conducting Plate Distance", 1.0, 100.0, 15.0, "nm", (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_linear("planck_energy_scale", "Planck Energy Fluctuation Scale", 0.1, 5.0, 1.0, "x", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("vacuum_polarization_factor", "Vacuum Polarization Bias", 0.0, 1.0, 0.45, "%", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("spectral_color_slope", "Quantum Noise Color Slope", -3.0, 3.0, 1.0, "dB/oct", (0, 230, 118)))
            ),
            "RelativisticDopplerShiftNode" => Box::new(
                GenericDspNodeUi::new("RelativisticDopplerShiftNode", "Lorentz Relativistic Velocity Doppler Time Dilation", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_linear("velocity_fraction_c", "Relativistic Velocity (v/c)", 0.01, 0.99, 0.5, "c", (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_linear("lorentz_gamma_factor", "Lorentz Factor Gamma", 1.0, 10.0, 1.15, "gamma", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("source_trajectory_angle_deg", "Observer Angle Theta", 0.0, 180.0, 45.0, "deg", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_bool("relativistic_aberration", "Relativistic Headlight Beaming", true, (0, 230, 118)))
            ),
            "StochasticQuantumDecoherenceNoise" => Box::new(
                GenericDspNodeUi::new("StochasticQuantumDecoherenceNoise", "Quantum Decoherence Lindblad Superoperator Noise", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_linear("lindblad_dissipator_rate_gamma", "Lindblad Decoherence Rate (Gamma)", 0.01, 10.0, 1.2, "gamma", (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_linear("environmental_bath_temp_k", "Thermal Bath Temperature", 0.01, 300.0, 4.2, "K", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("pure_dephasing_rate", "Pure Dephasing Noise Rate", 0.0, 5.0, 0.65, "", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("quantum_noise_mix", "Decoherence Noise Audio Blend", 0.0, 1.0, 0.4, "%", (0, 230, 118)))
            ),
            "QuantumTeleportationAudioBufferBus" => Box::new(
                GenericDspNodeUi::new("QuantumTeleportationAudioBufferBus", "Zero-Latency Entangled State Audio Bus Interlink", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_linear("epr_pair_fidelity", "EPR Entanglement Fidelity", 0.5, 1.0, 0.98, "%", (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_linear("bell_measurement_feedforward_gain", "Feedforward Recovery Gain", 0.0, 2.0, 1.0, "x", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_int("teleportation_channel_id", "Teleportation Bus Channel ID", 0, 15, 0, "ch", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_bool("state_entanglement_bus", "Quantum Entanglement Linked", true, (0, 230, 118)))
            ),
            "QuantumPhaseEstimationPitchTracker" => Box::new(
                GenericDspNodeUi::new("QuantumPhaseEstimationPitchTracker", "Quantum Phase Estimation Sub-Cent Pitch Tracker", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_int("evaluation_qubits_count", "Phase Estimation Qubits", 4, 16, 10, "qubits", (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_linear("unitary_evolution_time_step", "Unitary Time Step Delta", 0.01, 1.0, 0.1, "ms", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("frequency_resolution_cents", "Sub-Cent Frequency Accuracy", 0.01, 5.0, 0.1, "cents", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("phase_lock_threshold", "Eigenphase Probability Lock", 0.5, 0.99, 0.9, "%", (0, 230, 118)))
            ),
            "AtmosphericDensityNode" => Box::new(
                GenericDspNodeUi::new("AtmosphericDensityNode", "Atmospheric Gas Composition & Speed of Sound Simulator", DspNodeCategory::SpatialSurround)
                    .with_param(DspParamDescriptor::new_linear("ambient_pressure_kpa", "Atmospheric Pressure", 1.0, 1000.0, 101.3, "kPa", (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_linear("temperature_celsius", "Ambient Temperature", -100.0, 200.0, 20.0, "degC", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("helium_fraction_ratio", "Helium Gas Ratio (Speed Boost)", 0.0, 1.0, 0.0, "%", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("co2_fraction_ratio", "CO2 Gas Ratio (Speed Drop)", 0.0, 1.0, 0.0, "%", (0, 230, 118)))
            ),
            "QuantumDotTransducerNode" => Box::new(
                GenericDspNodeUi::new("QuantumDotTransducerNode", "Nanoscale Quantum Dot Optical-to-Audio Transducer", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_linear("optical_wavelength_nm", "Optical Wavelength", 200.0, 1100.0, 532.0, "nm", (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_linear("quantum_yield_efficiency", "Quantum Yield Efficiency", 0.1, 1.0, 0.9, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("fluorescence_lifetime_ns", "Carrier Lifetime Decay", 0.1, 50.0, 8.0, "ns", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("photocurrent_audio_gain", "Optoelectronic Audio Gain", 0.0, 2.0, 1.0, "x", (0, 230, 118)))
            ),
            "AcousticLevitationTrapNode" => Box::new(
                GenericDspNodeUi::new("AcousticLevitationTrapNode", "Gor'kov Ultrasound Acoustic Radiation Pressure Trap", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_log("ultrasound_carrier_freq_khz", "Ultrasound Carrier Frequency", 20.0, 100.0, 40.0, "kHz", (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_int("standing_wave_trap_nodes", "Standing Wave Trapping Nodes", 1, 16, 4, "nodes", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("levitated_particle_mass_mg", "Particle Mass Scale", 0.1, 50.0, 2.0, "mg", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("radiation_pressure_gain", "Radiation Potential Force", 0.1, 10.0, 2.5, "x", (0, 230, 118)))
            ),
            "MathAdd" => Box::new(
                GenericDspNodeUi::new("MathAdd", "Signal Arithmetic Sum & Offset Scale", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_linear("offset", "DC Offset Constant", -10.0, 10.0, 0.0, "", (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_linear("scale", "Scale Multiplier", 0.0, 5.0, 1.0, "x", (0, 229, 255)))
            ),
            "MathMult" => Box::new(
                GenericDspNodeUi::new("MathMult", "Signal Arithmetic Product & Modulation Scale", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_linear("factor", "Multiplication Factor", -5.0, 5.0, 1.0, "x", (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_linear("bias", "Post-Multiplier Bias", -1.0, 1.0, 0.0, "", (0, 229, 255)))
            ),
            "VCA" => Box::new(
                GenericDspNodeUi::new("VCA", "Voltage-Controlled Amplifier Gain Block", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_linear("gain", "Base VCA Gain", 0.0, 2.0, 1.0, "x", (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_linear("cv_amount", "CV Modulation Depth", 0.0, 1.0, 1.0, "%", (0, 229, 255)))
            ),
            "WsolaTimeStretcher" => Box::new(
                GenericDspNodeUi::new("WsolaTimeStretcher", "WSOLA Pitch-Preserving Time Stretch Engine", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_linear("stretch_ratio", "Time Stretch Ratio", 0.25, 4.0, 1.0, "x", (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_linear("window_ms", "Synthesis Window Size", 10.0, 100.0, 40.0, "ms", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("seek_window_ms", "Cross-Correlation Seek Window", 5.0, 50.0, 15.0, "ms", (255, 171, 0)))
            ),
            "StemSeparator" => Box::new(
                GenericDspNodeUi::new("StemSeparator", "Neural 4-Stem Audio Source Separation Matrix", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_linear("vocals_gain", "Vocals Stem Gain", 0.0, 2.0, 1.0, "x", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("drums_gain", "Drums Stem Gain", 0.0, 2.0, 1.0, "x", (255, 171, 0)))
                    .with_param(DspParamDescriptor::new_linear("bass_gain", "Bass Stem Gain", 0.0, 2.0, 1.0, "x", (153, 102, 255)))
                    .with_param(DspParamDescriptor::new_linear("other_gain", "Instruments Stem Gain", 0.0, 2.0, 1.0, "x", (0, 230, 118)))
            ),
            "ElasticWarpEngine" => Box::new(
                GenericDspNodeUi::new("ElasticWarpEngine", "Real-time Elastic Audio Warp & Transient Pinning", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_linear("time_ratio", "Elastic Stretch Ratio", 0.2, 5.0, 1.0, "x", (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_bool("transient_preserve", "Lock Beat Transients", true, (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("grain_overlap", "Grain Overlap Factor", 0.1, 0.9, 0.5, "%", (0, 230, 118)))
            ),
            "Oversampler" => Box::new(
                GenericDspNodeUi::new("Oversampler", "Polyphase Anti-Aliasing Oversampling Filter", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_enum("factor", "Oversampling Factor", vec!["2x Oversampling".into(), "4x Oversampling".into(), "8x Oversampling".into(), "16x Oversampling".into()], 1, (180, 195, 215)))
                    .with_param(DspParamDescriptor::new_bool("linear_phase", "Linear Phase Reconstruction", true, (0, 229, 255)))
            ),
            "PassthroughNode" => Box::new(
                GenericDspNodeUi::new("PassthroughNode", "Zero-Latency Transparent Buffer Relay", DspNodeCategory::Utility)
                    .with_param(DspParamDescriptor::new_bool("enabled", "Relay Active", true, (0, 230, 118)))
                    .with_param(DspParamDescriptor::new_linear("trim_db", "Relay Output Trim", -12.0, 12.0, 0.0, "dB", (180, 195, 215)))
            ),
            // Fallback dynamic generator for any unexpected or plugin node
            other => {
                let cat = Self::inventory()
                    .into_iter()
                    .find(|(n, _, _)| *n == other)
                    .map(|(_, c, _)| c)
                    .unwrap_or(DspNodeCategory::Utility);
                Box::new(
                    GenericDspNodeUi::new(
                        Box::leak(other.to_string().into_boxed_str()),
                        other,
                        cat,
                    )
                    .with_param(DspParamDescriptor::new_linear("macro_1", "Macro Dial 1", 0.0, 1.0, 0.5, "%", (0, 229, 255)))
                    .with_param(DspParamDescriptor::new_linear("macro_2", "Macro Dial 2", 0.0, 1.0, 0.5, "%", (255, 171, 0)))
                )
            }
        };
        Some(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dsp_node_registry_inventory_completeness() {
        let registry = DspNodeRegistry::new();
        assert!(registry.list_all().len() >= 270, "Registry should contain >= 270 descriptors");

        let inventory = DspNodeRegistry::inventory();
        assert!(inventory.len() >= 270, "Inventory should contain >= 270 entries");
    }

    #[test]
    fn test_dsp_node_descriptor_parameters_and_macro_bindings() {
        let registry = DspNodeRegistry::new();
        let aether = registry.get("AetherSynth").expect("AetherSynth must exist");
        assert_eq!(aether.category, DspNodeCategory::CompositeSynth);
        assert!(aether.params.len() >= 3);

        let shakuhachi = registry.get("ShakuhachiModel").expect("ShakuhachiModel must exist");
        assert_eq!(shakuhachi.category, DspNodeCategory::AcousticPhysicalModel);
        assert!(shakuhachi.params.len() >= 3);
    }

    #[test]
    fn test_dsp_category_colors_and_icons() {
        let cat = DspNodeCategory::Oscillator;
        assert_eq!(cat.icon(), "🌊");
        assert_eq!(cat.color_rgb(), (56, 189, 248));

        let cat_phys = DspNodeCategory::AcousticPhysicalModel;
        assert_eq!(cat_phys.icon(), "🎻");
        assert_eq!(cat_phys.color_rgb(), (245, 158, 11));
    }

    #[test]
    #[cfg(feature = "gui")]
    fn test_dsp_node_ui_rendering_headless_context() {
        let registry = DspNodeRegistry::new();
        let ctx = egui::Context::default();
        let param_bus = summoner_core::param_bus::ParamBus::new();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                for descriptor in registry.list_all() {
                    let mut params = HashMap::new();
                    let mut is_bypassed = false;
                    let mut is_collapsed = false;

                    descriptor.render_macro_strip(ui, &mut params, Some(&param_bus), 1, 0);
                    descriptor.render_pro_inspector(ui, &mut params, Some(&param_bus), 1, 0);
                    descriptor.render_rack_unit(
                        ui,
                        &mut params,
                        &mut is_bypassed,
                        &mut is_collapsed,
                        Some(&param_bus),
                        1,
                        0,
                    );
                }
            });
        });
    }
}
