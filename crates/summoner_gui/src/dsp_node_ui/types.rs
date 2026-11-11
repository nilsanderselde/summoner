// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Fundamental schema types and descriptors for DSP nodes and parameter reflection.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
            Self::Oscillator => (56, 189, 248),            // Vibrant Cyan
            Self::AcousticPhysicalModel => (245, 158, 11), // Warm Amber
            Self::FilterEq => (168, 85, 247),              // Violet / Purple
            Self::DynamicsMaster => (239, 68, 68),         // Crimson / Coral
            Self::DistortionSaturation => (249, 115, 22),  // Bright Orange
            Self::Modulation => (34, 197, 94),             // Emerald Green
            Self::TimeSpace => (99, 102, 241),             // Indigo Blue
            Self::SpatialSurround => (14, 165, 233),       // Sky Blue
            Self::SpectralResynthesis => (217, 70, 239),   // Magenta / Pink
            Self::NeuralAi => (236, 72, 153),              // Neon Rose
            Self::SamplerSlicer => (16, 185, 129),         // Mint Green
            Self::CompositeSynth => (59, 130, 246),        // Electric Blue
            Self::Utility => (148, 163, 184),              // Slate Gray
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
                DspWidgetKind::RotaryKnob {
                    default,
                    unit,
                    min,
                    max,
                    ..
                } => {
                    let norm = ((*default - *min) / (*max - *min).max(1e-5)).clamp(0.0, 1.0);
                    (norm, unit.clone())
                }
                DspWidgetKind::VerticalFader {
                    default,
                    unit,
                    min,
                    max,
                } => {
                    let norm = ((*default - *min) / (*max - *min).max(1e-5)).clamp(0.0, 1.0);
                    (norm, unit.clone())
                }
                DspWidgetKind::Toggle { default } => {
                    (if *default { 1.0 } else { 0.0 }, "".to_string())
                }
                DspWidgetKind::EnumChoice {
                    default_idx,
                    choices,
                } => {
                    let norm = if choices.is_empty() {
                        0.0
                    } else {
                        *default_idx as f32 / choices.len() as f32
                    };
                    (norm, "".to_string())
                }
                _ => (0.5, "".to_string()),
            };
            module = module.with_param(crate::views::dsp_rack_dock::DspModuleParam::new(
                &p.name,
                default_val,
                &unit,
            ));
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
            DspParamType::FloatLinear { unit, .. }
            | DspParamType::FloatLogarithmic { unit, .. } => {
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
                } else if unit == "dB"
                    || unit == "dBFS"
                    || unit == "dBTP"
                    || unit == "LUFS"
                    || unit == "dB/oct"
                {
                    format!("{:.1} {}", self.current_value, unit)
                } else if unit == "ms" {
                    format!("{:.1} ms", self.current_value)
                } else {
                    format!("{:.2} {}", self.current_value, unit)
                }
            }
            DspParamType::Enum { variants, .. } => {
                let idx = self.current_value.round() as usize;
                variants
                    .get(idx)
                    .cloned()
                    .unwrap_or_else(|| "Unknown".to_string())
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
