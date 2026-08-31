// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Universal `DspNodeUi` trait, parameter reflection schema, and comprehensive DSP module registry.
//! Provides two-tier UX: Novice Macro Strip (Tone, Space, Punch, Character) & Pro Surgical Parameter Inspector.
//! Zero CLI left behind: Every DSP module in `summoner_dsp` and `summoner_core` is registered with tactile controls.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, FontId, RichText, Rounding, Stroke};

#[cfg(feature = "gui")]
use summoner_core::param_bus::{ParamBus, ParamId};

/// Category classification for DSP Nodes with visual color-coding.
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

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Oscillator => "🌊",
            Self::AcousticPhysicalModel => "🎻",
            Self::FilterEq => "🎛️",
            Self::DynamicsMaster => "⚡",
            Self::DistortionSaturation => "🔥",
            Self::Modulation => "📈",
            Self::TimeSpace => "🌌",
            Self::SpatialSurround => "🌐",
            Self::SpectralResynthesis => "🌈",
            Self::NeuralAi => "🧠",
            Self::SamplerSlicer => "🎹",
            Self::CompositeSynth => "🔮",
            Self::Utility => "🔧",
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

    /// Convert this descriptor into a `DspRackModule` suitable for the modular rack docking system.
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

/// Universal Trait for DSP Node UI Reflection and Rendering.
pub trait DspNodeUi {
    /// Returns the node descriptor metadata.
    fn descriptor(&self) -> &DspNodeDescriptor;

    /// Renders the Novice Macro Strip (4 high-level macro knobs: Tone, Space, Punch, Character).
    #[cfg(feature = "gui")]
    fn render_macro_strip(
        &self,
        ui: &mut egui::Ui,
        param_values: &mut HashMap<String, f32>,
        param_bus: Option<&ParamBus>,
        track_id: u64,
        node_idx: usize,
    );

    /// Renders the Pro Surgical Parameter Inspector with full parameter reflection.
    #[cfg(feature = "gui")]
    fn render_pro_inspector(
        &self,
        ui: &mut egui::Ui,
        param_values: &mut HashMap<String, f32>,
        param_bus: Option<&ParamBus>,
        track_id: u64,
        node_idx: usize,
    );

    /// Renders the Modular Rack Unit Box.
    #[cfg(feature = "gui")]
    fn render_rack_unit(
        &self,
        ui: &mut egui::Ui,
        param_values: &mut HashMap<String, f32>,
        is_bypassed: &mut bool,
        is_collapsed: &mut bool,
        param_bus: Option<&ParamBus>,
        track_id: u64,
        node_idx: usize,
    );
}

/// Default implementation of `DspNodeUi` for any `DspNodeDescriptor`.
impl DspNodeUi for DspNodeDescriptor {
    fn descriptor(&self) -> &DspNodeDescriptor {
        self
    }

    #[cfg(feature = "gui")]
    fn render_macro_strip(
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

    #[cfg(feature = "gui")]
    fn render_pro_inspector(
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
                        .rounding(Rounding::same(3.0))
                        .inner_margin(egui::Margin::symmetric(4.0, 2.0))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(self.category.name())
                                    .font(FontId::proportional(9.0))
                                    .color(cat_color),
                            );
                        });
                });
            });

            ui.add_space(4.0);
            ui.label(
                RichText::new(&self.description)
                    .font(FontId::proportional(10.0))
                    .color(Color32::from_rgb(148, 163, 184)),
            );
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Render all parameters
            for (p_idx, param) in self.params.iter().enumerate() {
                let pid = ParamId(track_id as u32 * 1000 + node_idx as u32 * 20 + p_idx as u32);
                match &param.widget {
                    DspWidgetKind::RotaryKnob {
                        min,
                        max,
                        default,
                        unit,
                        ..
                    } => {
                        let val = param_values.entry(param.id.clone()).or_insert(*default);
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(&param.name)
                                    .font(FontId::proportional(10.0))
                                    .color(Color32::from_rgb(203, 213, 225)),
                            );
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(
                                    RichText::new(format!("{:.2} {}", *val, unit))
                                        .font(FontId::proportional(10.0))
                                        .color(cat_color),
                                );
                            });
                        });
                        if ui
                            .add(egui::Slider::new(val, *min..=*max).show_value(false))
                            .changed()
                        {
                            if let Some(bus) = param_bus {
                                if bus.get(pid).is_some() {
                                    bus.set(pid, *val);
                                }
                            }
                        }
                        ui.add_space(4.0);
                    }
                    DspWidgetKind::VerticalFader {
                        min,
                        max,
                        default,
                        unit,
                    } => {
                        let val = param_values.entry(param.id.clone()).or_insert(*default);
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(&param.name)
                                    .font(FontId::proportional(10.0))
                                    .color(Color32::from_rgb(203, 213, 225)),
                            );
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(
                                    RichText::new(format!("{:.2} {}", *val, unit))
                                        .font(FontId::proportional(10.0))
                                        .color(cat_color),
                                );
                            });
                        });
                        if ui
                            .add(egui::Slider::new(val, *min..=*max).show_value(false))
                            .changed()
                        {
                            if let Some(bus) = param_bus {
                                if bus.get(pid).is_some() {
                                    bus.set(pid, *val);
                                }
                            }
                        }
                        ui.add_space(4.0);
                    }
                    DspWidgetKind::Toggle { default } => {
                        let mut b = param_values
                            .get(&param.id)
                            .map(|v| *v > 0.5)
                            .unwrap_or(*default);
                        if ui.checkbox(&mut b, &param.name).changed() {
                            let f = if b { 1.0 } else { 0.0 };
                            param_values.insert(param.id.clone(), f);
                            if let Some(bus) = param_bus {
                                if bus.get(pid).is_some() {
                                    bus.set(pid, f);
                                }
                            }
                        }
                        ui.add_space(4.0);
                    }
                    DspWidgetKind::EnumChoice {
                        choices,
                        default_idx,
                    } => {
                        let mut cur_idx = param_values
                            .get(&param.id)
                            .map(|v| *v as usize)
                            .unwrap_or(*default_idx)
                            .min(choices.len().saturating_sub(1));
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(&param.name)
                                    .font(FontId::proportional(10.0))
                                    .color(Color32::from_rgb(203, 213, 225)),
                            );
                            egui::ComboBox::from_id_source(format!("combo_{}_{}_{}", track_id, node_idx, p_idx))
                                .selected_text(choices.get(cur_idx).cloned().unwrap_or_default())
                                .show_ui(ui, |ui| {
                                    for (c_i, choice) in choices.iter().enumerate() {
                                        if ui.selectable_value(&mut cur_idx, c_i, choice).clicked() {
                                            param_values.insert(param.id.clone(), cur_idx as f32);
                                            if let Some(bus) = param_bus {
                                                if bus.get(pid).is_some() {
                                                    bus.set(pid, cur_idx as f32);
                                                }
                                            }
                                        }
                                    }
                                });
                        });
                        ui.add_space(4.0);
                    }
                    _ => {}
                }
            }
        });
    }

    #[cfg(feature = "gui")]
    fn render_rack_unit(
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
        let cat_color = Color32::from_rgb(r, g, b);

        egui::Frame::none()
            .fill(if *is_bypassed {
                Color32::from_rgb(12, 16, 24)
            } else {
                Color32::from_rgb(18, 24, 36)
            })
            .stroke(Stroke::new(
                1.0_f32,
                if *is_bypassed {
                    Color32::from_rgb(28, 38, 56)
                } else {
                    Color32::from_rgb(36, 50, 74)
                },
            ))
            .rounding(Rounding::same(6.0))
            .inner_margin(egui::Margin::symmetric(10.0, 8.0))
            .show(ui, |ui| {
                ui.set_width(240.0);

                // Header
                ui.horizontal(|ui| {
                    let (dot_rect, _) =
                        ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
                    ui.painter()
                        .circle_filled(dot_rect.center(), 4.0, cat_color);

                    ui.label(
                        RichText::new(&self.display_name)
                            .font(FontId::proportional(12.0))
                            .strong()
                            .color(Color32::from_rgb(241, 245, 249)),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let collapse_txt = if *is_collapsed { "▼" } else { "▲" };
                        if ui.small_button(collapse_txt).clicked() {
                            *is_collapsed = !*is_collapsed;
                        }

                        let pwr_col = if *is_bypassed {
                            Color32::from_rgb(100, 116, 139)
                        } else {
                            cat_color
                        };
                        if ui
                            .button(
                                RichText::new("⏻")
                                    .font(FontId::proportional(11.0))
                                    .color(pwr_col),
                            )
                            .clicked()
                        {
                            *is_bypassed = !*is_bypassed;
                        }
                    });
                });

                if *is_collapsed {
                    return;
                }

                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);

                if *is_bypassed {
                    ui.weak("Device Bypassed");
                    return;
                }

                // Grid of top 4 parameters / rotary knobs
                let knob_params: Vec<&DspParamSchema> = self
                    .params
                    .iter()
                    .filter(|p| matches!(p.widget, DspWidgetKind::RotaryKnob { .. }))
                    .take(4)
                    .collect();

                if !knob_params.is_empty() {
                    ui.horizontal(|ui| {
                        for (p_i, param) in knob_params.iter().enumerate() {
                            let (min, max, default) = match &param.widget {
                                DspWidgetKind::RotaryKnob {
                                    min, max, default, ..
                                } => (*min, *max, *default),
                                _ => (0.0, 1.0, 0.5),
                            };
                            let val = param_values.entry(param.id.clone()).or_insert(default);
                            draw_tactile_dial(
                                ui,
                                &param.name,
                                val,
                                min,
                                max,
                                cat_color,
                                &param.tooltip,
                            );
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

#[cfg(feature = "gui")]
fn draw_tactile_dial(
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
    pub fn new() -> Self {
        let mut reg = Self {
            descriptors: HashMap::new(),
        };
        reg.register_all_dsp_modules();
        reg
    }

    pub fn get(&self, kind_id: &str) -> Option<&DspNodeDescriptor> {
        self.descriptors.get(kind_id)
    }

    pub fn list_all(&self) -> Vec<&DspNodeDescriptor> {
        let mut list: Vec<&DspNodeDescriptor> = self.descriptors.values().collect();
        list.sort_by(|a, b| a.display_name.cmp(&b.display_name));
        list
    }

    pub fn list_by_category(&self, category: DspNodeCategory) -> Vec<&DspNodeDescriptor> {
        let mut list: Vec<&DspNodeDescriptor> = self
            .descriptors
            .values()
            .filter(|d| d.category == category)
            .collect();
        list.sort_by(|a, b| a.display_name.cmp(&b.display_name));
        list
    }

    pub fn register(&mut self, descriptor: DspNodeDescriptor) {
        self.descriptors
            .insert(descriptor.kind_id.clone(), descriptor);
    }

    /// Exhaustive registration of ALL DSP modules in `summoner_dsp` and `summoner_core`.
    fn register_all_dsp_modules(&mut self) {
        // ==========================================
        // 1. Oscillators & Generators
        // ==========================================
        self.register(
            DspNodeDescriptor::new(
                "OscSine",
                "Sine Oscillator",
                DspNodeCategory::Oscillator,
                "Pure sinusoidal tone generator with deterministic phase tracking.",
            )
            .with_param(DspParamSchema::log_knob("freq", "Frequency", 20.0, 20000.0, 440.0, "Hz", MacroRole::Tone, "Oscillator base frequency"))
            .with_param(DspParamSchema::knob("phase", "Phase", 0.0, 360.0, 0.0, "°", MacroRole::None, "Initial phase offset"))
            .with_param(DspParamSchema::knob("level", "Level", 0.0, 1.0, 0.8, "%", MacroRole::Punch, "Signal output gain"))
        );

        self.register(
            DspNodeDescriptor::new(
                "OscSaw",
                "Band-Limited Saw Oscillator",
                DspNodeCategory::Oscillator,
                "Harmonically rich PolyBLEP anti-aliased sawtooth generator.",
            )
            .with_param(DspParamSchema::log_knob("freq", "Frequency", 20.0, 20000.0, 440.0, "Hz", MacroRole::Tone, "Oscillator base frequency"))
            .with_param(DspParamSchema::knob("sync", "Hard Sync", 0.0, 1.0, 0.0, "%", MacroRole::Character, "Master oscillator sync slave phase lock"))
            .with_param(DspParamSchema::knob("level", "Level", 0.0, 1.0, 0.8, "%", MacroRole::Punch, "Signal output gain"))
        );

        self.register(
            DspNodeDescriptor::new(
                "OscPulse",
                "Pulse / Square Oscillator",
                DspNodeCategory::Oscillator,
                "Variable duty-cycle square and pulse wave generator with PWM modulation.",
            )
            .with_param(DspParamSchema::log_knob("freq", "Frequency", 20.0, 20000.0, 440.0, "Hz", MacroRole::Tone, "Oscillator base frequency"))
            .with_param(DspParamSchema::knob("pulse_width", "Pulse Width", 0.01, 0.99, 0.5, "%", MacroRole::Character, "PWM Duty cycle balance"))
            .with_param(DspParamSchema::knob("level", "Level", 0.0, 1.0, 0.8, "%", MacroRole::Punch, "Signal output gain"))
        );

        self.register(
            DspNodeDescriptor::new(
                "OscTriangle",
                "Triangle Oscillator",
                DspNodeCategory::Oscillator,
                "Smooth triangle wave generator with odd harmonic falloff.",
            )
            .with_param(DspParamSchema::log_knob("freq", "Frequency", 20.0, 20000.0, 440.0, "Hz", MacroRole::Tone, "Oscillator base frequency"))
            .with_param(DspParamSchema::knob("level", "Level", 0.0, 1.0, 0.8, "%", MacroRole::Punch, "Signal output gain"))
        );

        self.register(
            DspNodeDescriptor::new(
                "OscWavetable",
                "Morphing Wavetable Oscillator",
                DspNodeCategory::Oscillator,
                "2048-sample morphing wavetable oscillator transitioning across custom tables.",
            )
            .with_param(DspParamSchema::log_knob("freq", "Frequency", 20.0, 20000.0, 440.0, "Hz", MacroRole::Tone, "Oscillator base frequency"))
            .with_param(DspParamSchema::knob("morph", "Morph Position", 0.0, 1.0, 0.0, "%", MacroRole::Character, "Interpolation index through wavetable slots"))
            .with_param(DspParamSchema::knob("unison", "Unison Voices", 1.0, 8.0, 1.0, "v", MacroRole::Space, "Detuned polyphonic unison voice count"))
            .with_param(DspParamSchema::knob("detune", "Detune Spread", 0.0, 100.0, 10.0, "cents", MacroRole::Space, "Stereo unison pitch detune"))
        );

        self.register(
            DspNodeDescriptor::new(
                "SimdPolyWavetableOscillator",
                "SIMD Polyphonic Wavetable Voice",
                DspNodeCategory::Oscillator,
                "AVX2/NEON vector-accelerated polyphonic wavetable synth engine.",
            )
            .with_param(DspParamSchema::knob("morph", "Wavetable Morph", 0.0, 1.0, 0.5, "%", MacroRole::Character, "Table position"))
            .with_param(DspParamSchema::knob("spread", "Stereo Spread", 0.0, 1.0, 0.5, "%", MacroRole::Space, "Voice stereo width"))
        );

        self.register(
            DspNodeDescriptor::new(
                "NoiseGen",
                "Multi-Color Noise Generator",
                DspNodeCategory::Oscillator,
                "Deterministic noise generator with White, Pink, Brown, and Velvet distribution.",
            )
            .with_param(DspParamSchema::choice("noise_type", "Noise Color", &["White", "Pink", "Brown", "Velvet"], 0, "Frequency spectrum distribution profile"))
            .with_param(DspParamSchema::knob("level", "Level", 0.0, 1.0, 0.5, "%", MacroRole::Punch, "Noise output volume"))
        );

        // ==========================================
        // 2. Physical Modeling & Acoustic Instruments
        // ==========================================
        self.register(
            DspNodeDescriptor::new(
                "PluckedStringNode",
                "Karplus-Strong Plucked String",
                DspNodeCategory::AcousticPhysicalModel,
                "Waveguide physical modeling string synthesis with tension and dampening.",
            )
            .with_param(DspParamSchema::log_knob("freq", "Frequency", 40.0, 4000.0, 440.0, "Hz", MacroRole::Tone, "String fundamental pitch"))
            .with_param(DspParamSchema::knob("hardness", "Pluck Hardness", 0.0, 1.0, 0.8, "%", MacroRole::Punch, "Impulse excitation sharpness"))
            .with_param(DspParamSchema::knob("tension", "String Tension", 0.0, 1.0, 0.5, "%", MacroRole::Character, "Non-linear pitch dispersion"))
            .with_param(DspParamSchema::knob("damping", "Damping", 0.0, 1.0, 0.2, "%", MacroRole::Space, "High-frequency string loss factor"))
        );

        self.register(
            DspNodeDescriptor::new(
                "BowedString",
                "Bowed String Resonator",
                DspNodeCategory::AcousticPhysicalModel,
                "Non-linear friction cello / violin bowed waveguide physical model.",
            )
            .with_param(DspParamSchema::knob("bow_velocity", "Bow Velocity", 0.0, 5.0, 1.2, "m/s", MacroRole::Punch, "Bowing draw speed"))
            .with_param(DspParamSchema::knob("bow_force", "Bow Force", 0.1, 10.0, 2.5, "N", MacroRole::Character, "Downward rosin contact pressure"))
            .with_param(DspParamSchema::knob("bow_pos", "Bow Position", 0.02, 0.5, 0.12, "pos", MacroRole::Tone, "Distance from bridge"))
        );

        self.register(
            DspNodeDescriptor::new(
                "Shakuhachi",
                "Shakuhachi Bamboo Flute",
                DspNodeCategory::AcousticPhysicalModel,
                "Non-linear air jet split with Utaguchi embouchure and Meri/Kari pitch bending.",
            )
            .with_param(DspParamSchema::knob("pressure", "Blowing Pressure", 10.0, 2000.0, 450.0, "Pa", MacroRole::Punch, "Air column breath pressure"))
            .with_param(DspParamSchema::knob("angle", "Embouchure Angle", -30.0, 30.0, 0.0, "°", MacroRole::Tone, "Utaguchi attack lip angle"))
            .with_param(DspParamSchema::knob("meri_kari", "Meri / Kari Bend", -3.0, 2.0, 0.0, "st", MacroRole::Character, "Microtonal head tilt bend"))
        );

        self.register(
            DspNodeDescriptor::new(
                "TurkishNey",
                "Turkish Ney End-Blown Flute",
                DspNodeCategory::AcousticPhysicalModel,
                "Reed acoustics with Baspare horn rim vortex turbulence.",
            )
            .with_param(DspParamSchema::knob("pressure", "Breath Pressure", 50.0, 1500.0, 320.0, "Pa", MacroRole::Punch, "Wind velocity"))
            .with_param(DspParamSchema::knob("baspare_lip", "Baspare Lip Fit", 0.0, 1.0, 0.6, "%", MacroRole::Tone, "Horn coupling index"))
        );

        self.register(
            DspNodeDescriptor::new(
                "GrandPiano",
                "Grand Piano Physical Model",
                DspNodeCategory::AcousticPhysicalModel,
                "Felt hammer non-linear strike, duplex scaling, and soundboard impedance coupling.",
            )
            .with_param(DspParamSchema::knob("hammer_hardness", "Hammer Hardness", 0.1, 2.0, 1.0, "x", MacroRole::Punch, "Felt stiffness dynamic response"))
            .with_param(DspParamSchema::knob("damper_decay", "Damper Release", 0.05, 2.0, 0.35, "s", MacroRole::Space, "Damper felt stop time"))
            .with_param(DspParamSchema::knob("soundboard", "Soundboard Resonance", 0.0, 1.0, 0.7, "%", MacroRole::Character, "Spruce bridge modal body resonance"))
        );

        self.register(
            DspNodeDescriptor::new(
                "ElectricPiano",
                "Tine Electric Piano (Rhodes / Wurlitzer)",
                DspNodeCategory::AcousticPhysicalModel,
                "Asymmetric magnetic pickup and vibrating cantilever tine resonator.",
            )
            .with_param(DspParamSchema::knob("tine_decay", "Tine Decay", 0.1, 8.0, 2.5, "s", MacroRole::Space, "Tine vibration length"))
            .with_param(DspParamSchema::knob("pickup_dist", "Pickup Distance", 0.5, 5.0, 1.5, "mm", MacroRole::Tone, "Magnetic bar gap and bark overdrive"))
            .with_param(DspParamSchema::knob("tremolo_rate", "Tremolo Rate", 0.5, 12.0, 4.0, "Hz", MacroRole::Character, "Stereo pan tremolo speed"))
        );

        self.register(
            DspNodeDescriptor::new(
                "Clavinet",
                "Clavinet D6 Physical Model",
                DspNodeCategory::AcousticPhysicalModel,
                "Rubber tip tangent hammer strike with dual electromagnetic pickups.",
            )
            .with_param(DspParamSchema::choice("pickup_mode", "Pickup Selector", &["Neck", "Bridge", "Both Parallel", "Both Out-of-Phase"], 2, "Pickup wiring"))
            .with_param(DspParamSchema::knob("brilliance", "Brilliance Filter", 0.0, 1.0, 0.75, "%", MacroRole::Tone, "Slap high-end brilliance"))
        );

        self.register(
            DspNodeDescriptor::new(
                "TonewheelOrgan",
                "Tonewheel B3 Organ & Leslie",
                DspNodeCategory::AcousticPhysicalModel,
                "9 Drawbars with harmonic tonewheel inductors and dual-rotor Leslie speaker.",
            )
            .with_param(DspParamSchema::knob("drawbar_16", "Sub 16'", 0.0, 8.0, 8.0, "", MacroRole::Tone, "16-foot sub drawbar"))
            .with_param(DspParamSchema::knob("drawbar_8", "Fund 8'", 0.0, 8.0, 8.0, "", MacroRole::Tone, "8-foot fundamental drawbar"))
            .with_param(DspParamSchema::knob("drawbar_4", "Oct 4'", 0.0, 8.0, 6.0, "", MacroRole::Tone, "4-foot octave drawbar"))
            .with_param(DspParamSchema::choice("leslie_speed", "Leslie Rotor Speed", &["Stop", "Chorale (Slow)", "Tremolo (Fast)"], 1, "Rotary speaker horn & drum rotation"))
        );

        self.register(
            DspNodeDescriptor::new(
                "PipeOrgan",
                "Church Pipe Organ Windchest",
                DspNodeCategory::AcousticPhysicalModel,
                "Multi-rank flue & reed pipe organ simulation with windchest pressure sag.",
            )
            .with_param(DspParamSchema::knob("wind_pressure", "Wind Pressure", 40.0, 180.0, 90.0, "mmH2O", MacroRole::Punch, "Bellows air reservoir pressure"))
            .with_param(DspParamSchema::knob("voicing", "Voicing Brightness", 0.0, 1.0, 0.6, "%", MacroRole::Tone, "Languid pipe lip brightness"))
        );

        self.register(
            DspNodeDescriptor::new(
                "VocalTract",
                "Articulatory Vocal Tract Synth",
                DspNodeCategory::AcousticPhysicalModel,
                "Kelly-Lochbaum tube section model with glottal pulse excitation and nasal tract.",
            )
            .with_param(DspParamSchema::knob("vowel_morph", "Vowel Morph (A-E-I-O-U)", 0.0, 4.0, 0.0, "", MacroRole::Tone, "Formant vocal aperture"))
            .with_param(DspParamSchema::knob("tract_length", "Tract Length (Gender)", 12.0, 20.0, 16.5, "cm", MacroRole::Character, "Formant scale"))
            .with_param(DspParamSchema::knob("glottal_open", "Glottal Open Quotient", 0.2, 0.8, 0.5, "%", MacroRole::Punch, "Vocal cord breathiness"))
        );

        self.register(
            DspNodeDescriptor::new(
                "WaveguideBrass",
                "Waveguide Brass & Lip Reed",
                DspNodeCategory::AcousticPhysicalModel,
                "Non-linear lip-reed brass instrument model with exponential bell flair radiation.",
            )
            .with_param(DspParamSchema::knob("lip_tension", "Lip Tension", 0.1, 5.0, 1.0, "x", MacroRole::Tone, "Embouchure muscular tension"))
            .with_param(DspParamSchema::knob("bore_length", "Bore Length", 0.5, 4.0, 1.4, "m", MacroRole::Character, "Acoustic tubing length"))
            .with_param(DspParamSchema::knob("bell_flare", "Bell Flare Reflection", 0.1, 1.0, 0.7, "%", MacroRole::Punch, "High frequency acoustic impedance matching"))
        );

        self.register(
            DspNodeDescriptor::new(
                "GlassArmonica",
                "Benjamin Franklin Glass Armonica",
                DspNodeCategory::AcousticPhysicalModel,
                "Rotating quartz glass bowls with stick-slip finger friction excitation.",
            )
            .with_param(DspParamSchema::knob("friction", "Friction Factor", 0.0, 1.0, 0.65, "%", MacroRole::Character, "Finger pad wetness contact"))
            .with_param(DspParamSchema::knob("rotation_speed", "Rotation Speed", 10.0, 120.0, 45.0, "RPM", MacroRole::Tone, "Bowl spindle rate"))
        );

        // ==========================================
        // 3. Filters & Equalizers
        // ==========================================
        self.register(
            DspNodeDescriptor::new(
                "FilterSVF",
                "State Variable Filter (SVF)",
                DspNodeCategory::FilterEq,
                "Zero-Delay Feedback (ZDF) 12dB/oct SVF supporting LP, HP, BP, and Notch outputs.",
            )
            .with_param(DspParamSchema::log_knob("cutoff", "Cutoff Frequency", 20.0, 20000.0, 2500.0, "Hz", MacroRole::Tone, "Filter corner frequency"))
            .with_param(DspParamSchema::knob("resonance", "Resonance / Q", 0.5, 20.0, 1.5, "Q", MacroRole::Character, "Resonance resonance peak sharpness"))
            .with_param(DspParamSchema::choice("mode", "Filter Type", &["LowPass", "HighPass", "BandPass", "Notch", "Peak"], 0, "Filter output mode"))
            .with_param(DspParamSchema::knob("drive", "Filter Drive", 0.0, 10.0, 0.0, "dB", MacroRole::Punch, "Non-linear saturation drive"))
        );

        self.register(
            DspNodeDescriptor::new(
                "FilterLadder",
                "Moog 4-Pole Transistor Ladder Filter",
                DspNodeCategory::FilterEq,
                "Classic 24dB/oct resonant transistor ladder with thermal voltage saturation.",
            )
            .with_param(DspParamSchema::log_knob("cutoff", "Cutoff Frequency", 20.0, 20000.0, 1800.0, "Hz", MacroRole::Tone, "Ladder corner frequency"))
            .with_param(DspParamSchema::knob("resonance", "Resonance (Self-Osc)", 0.0, 4.0, 1.2, "Q", MacroRole::Character, "Feedback path gain"))
            .with_param(DspParamSchema::knob("drive", "Input Overdrive", 0.0, 20.0, 2.0, "dB", MacroRole::Punch, "Differential transistor pair overdrive"))
        );

        self.register(
            DspNodeDescriptor::new(
                "ParametricEqNode",
                "8-Band Parametric Equalizer",
                DspNodeCategory::FilterEq,
                "Precision mastering grade 8-band stereo parametric EQ with bell and shelving shapes.",
            )
            .with_param(DspParamSchema::knob("low_gain", "Low Band Gain", -24.0, 24.0, 0.0, "dB", MacroRole::Punch, "Low frequency boost/cut"))
            .with_param(DspParamSchema::knob("mid_gain", "Mid Band Gain", -24.0, 24.0, 0.0, "dB", MacroRole::Tone, "Midrange boost/cut"))
            .with_param(DspParamSchema::knob("high_gain", "High Band Gain", -24.0, 24.0, 0.0, "dB", MacroRole::Character, "High frequency boost/cut"))
        );

        self.register(
            DspNodeDescriptor::new(
                "MultiChannelSpectralEqualizerNode",
                "Multi-Channel Spectral Equalizer",
                DspNodeCategory::FilterEq,
                "16-band ultra-fine linear-phase spectral equalizer with tilt and smoothing.",
            )
            .with_param(DspParamSchema::knob("tilt", "Spectral Tilt", -6.0, 6.0, 0.0, "dB/oct", MacroRole::Tone, "Linear frequency spectrum balance slope"))
            .with_param(DspParamSchema::knob("sharpness", "Resolution Sharpness", 0.1, 2.0, 1.0, "x", MacroRole::Character, "Filter Q factor per bin"))
        );

        // ==========================================
        // 4. Dynamics & Mastering
        // ==========================================
        self.register(
            DspNodeDescriptor::new(
                "CompressorNode",
                "Studio Dynamic Compressor",
                DspNodeCategory::DynamicsMaster,
                "Peak / RMS feed-forward dynamic range compressor with soft knee.",
            )
            .with_param(DspParamSchema::knob("threshold", "Threshold", -60.0, 0.0, -18.0, "dB", MacroRole::Punch, "Compression start level"))
            .with_param(DspParamSchema::knob("ratio", "Ratio", 1.0, 20.0, 4.0, ":1", MacroRole::Character, "Gain reduction ratio"))
            .with_param(DspParamSchema::knob("attack", "Attack Time", 0.1, 200.0, 15.0, "ms", MacroRole::Punch, "Time to apply full compression"))
            .with_param(DspParamSchema::knob("release", "Release Time", 10.0, 2000.0, 120.0, "ms", MacroRole::Space, "Time to restore original gain"))
            .with_param(DspParamSchema::knob("makeup", "Makeup Gain", 0.0, 24.0, 3.0, "dB", MacroRole::Punch, "Post-compression gain compensation"))
        );

        self.register(
            DspNodeDescriptor::new(
                "MultibandCompressorNode",
                "3-Band Multiband Compressor",
                DspNodeCategory::DynamicsMaster,
                "Independent dynamic compression across Low, Mid, and High frequency bands.",
            )
            .with_param(DspParamSchema::knob("low_thresh", "Low Thresh", -40.0, 0.0, -18.0, "dB", MacroRole::Punch, "Bass band compression threshold"))
            .with_param(DspParamSchema::knob("mid_thresh", "Mid Thresh", -40.0, 0.0, -16.0, "dB", MacroRole::Tone, "Mid band compression threshold"))
            .with_param(DspParamSchema::knob("high_thresh", "High Thresh", -40.0, 0.0, -14.0, "dB", MacroRole::Character, "High band compression threshold"))
        );

        self.register(
            DspNodeDescriptor::new(
                "LimiterNode",
                "Master True-Peak Brickwall Limiter",
                DspNodeCategory::DynamicsMaster,
                "Transparent lookahead brickwall limiter with inter-sample peak protection.",
            )
            .with_param(DspParamSchema::knob("ceiling", "Ceiling", -12.0, 0.0, -0.2, "dBTP", MacroRole::Punch, "Maximum allowed peak level"))
            .with_param(DspParamSchema::knob("release", "Release Time", 1.0, 500.0, 50.0, "ms", MacroRole::Space, "Recovery time"))
        );

        self.register(
            DspNodeDescriptor::new(
                "MidSideNode",
                "Mid/Side Stereo Processor",
                DspNodeCategory::DynamicsMaster,
                "Stereo width and spatial imaging controller decomposing Left/Right into Mid/Side.",
            )
            .with_param(DspParamSchema::knob("width", "Stereo Width", 0.0, 4.0, 1.0, "x", MacroRole::Space, "Side channel gain multiplier (0=Mono, >1=Expanded)"))
        );

        // ==========================================
        // 5. Distortion, Saturation & Color
        // ==========================================
        self.register(
            DspNodeDescriptor::new(
                "TapeSaturationNode",
                "Analog Tape Machine Emulation",
                DspNodeCategory::DistortionSaturation,
                "Magnetic hysteresis saturation, tape compression, head bump, and wow/flutter.",
            )
            .with_param(DspParamSchema::knob("drive", "Tape Drive", 1.0, 10.0, 3.5, "x", MacroRole::Punch, "Input recording level and tape saturation"))
            .with_param(DspParamSchema::knob("flutter", "Wow & Flutter", 0.0, 1.0, 0.15, "%", MacroRole::Character, "Mechanical transport speed irregularity"))
            .with_param(DspParamSchema::choice("tape_speed", "Tape Speed", &["7.5 ips (Warm)", "15 ips (Balanced)", "30 ips (Hi-Fi)"], 1, "Reel speed standard"))
        );

        self.register(
            DspNodeDescriptor::new(
                "TubeSaturationNode",
                "Triode / Pentode Tube Saturation",
                DspNodeCategory::DistortionSaturation,
                "Thermionic vacuum tube amplifier model with asymmetric DC bias warmth.",
            )
            .with_param(DspParamSchema::knob("drive", "Tube Drive", 1.0, 10.0, 2.5, "x", MacroRole::Punch, "Grid overdrive factor"))
            .with_param(DspParamSchema::knob("bias", "Cathode Bias", 0.0, 0.5, 0.15, "V", MacroRole::Character, "Asymmetric harmonic even/odd weighting"))
        );

        self.register(
            DspNodeDescriptor::new(
                "DistortionNode",
                "Multi-Curve Distortion & Fuzz",
                DspNodeCategory::DistortionSaturation,
                "Hard clipping, soft saturation, arctan, and diode overdrive shapes.",
            )
            .with_param(DspParamSchema::knob("drive", "Drive Amount", 1.0, 30.0, 5.0, "dB", MacroRole::Punch, "Pre-gain overdrive"))
            .with_param(DspParamSchema::choice("type", "Clipping Shape", &["Soft Tanh", "Hard Clip", "Diode Foldback", "Asymmetric"], 0, "Transfer curve algorithm"))
        );

        self.register(
            DspNodeDescriptor::new(
                "BitcrusherNode",
                "Digital Bitcrusher & Sample Reducer",
                DspNodeCategory::DistortionSaturation,
                "Bit-depth quantization and zero-order hold sample rate decimator.",
            )
            .with_param(DspParamSchema::knob("bits", "Bit Depth", 1.0, 16.0, 8.0, "bits", MacroRole::Character, "Quantization word length"))
            .with_param(DspParamSchema::knob("downsample", "Downsampling Factor", 1.0, 64.0, 4.0, "x", MacroRole::Tone, "Sample rate reduction ratio"))
        );

        self.register(
            DspNodeDescriptor::new(
                "WavefolderNode",
                "West-Coast Wavefolder",
                DspNodeCategory::DistortionSaturation,
                "Buchla-style mathematical wavefolder reflecting waveform peaks back into themselves.",
            )
            .with_param(DspParamSchema::knob("threshold", "Fold Threshold", 0.05, 1.0, 0.5, "", MacroRole::Tone, "Boundary folding level"))
            .with_param(DspParamSchema::knob("drive", "Fold Drive", 1.0, 10.0, 3.0, "x", MacroRole::Punch, "Input amplification before folds"))
        );

        self.register(
            DspNodeDescriptor::new(
                "ConsoleEmulationNode",
                "Analog Mixing Console Emulation",
                DspNodeCategory::DistortionSaturation,
                "Transformer core saturation, bus crosstalk, and discrete op-amp coloration.",
            )
            .with_param(DspParamSchema::choice("console_mode", "Console Model", &["Neve 8048 (Warm)", "SSL 4000E (Punchy)", "API 1608 (Aggressive)"], 0, "Desk transformer topology"))
            .with_param(DspParamSchema::knob("drive", "Channel Drive", 0.0, 5.0, 1.0, "x", MacroRole::Punch, "Transformer saturation level"))
        );

        // ==========================================
        // 6. Time, Space & Reverbs
        // ==========================================
        self.register(
            DspNodeDescriptor::new(
                "EffectDelay",
                "Stereo Echo / Tape Delay",
                DspNodeCategory::TimeSpace,
                "BPM-synced stereo delay with feedback filtering, ping-pong and tape modulation.",
            )
            .with_param(DspParamSchema::knob("time", "Delay Time", 0.01, 2.0, 0.35, "s", MacroRole::Space, "Echo duration"))
            .with_param(DspParamSchema::knob("feedback", "Feedback", 0.0, 0.98, 0.45, "%", MacroRole::Space, "Repeats intensity"))
            .with_param(DspParamSchema::knob("mix", "Dry / Wet Mix", 0.0, 1.0, 0.30, "%", MacroRole::Space, "Wet signal ratio"))
        );

        self.register(
            DspNodeDescriptor::new(
                "EffectReverb",
                "Algorithmic Reverb Space",
                DspNodeCategory::TimeSpace,
                "Freeverb / Dattorro matrix reverb with late diffusion and early reflections.",
            )
            .with_param(DspParamSchema::knob("room_size", "Room Size", 0.0, 1.0, 0.75, "%", MacroRole::Space, "Acoustic chamber dimensions"))
            .with_param(DspParamSchema::knob("damping", "High Damping", 0.0, 1.0, 0.35, "%", MacroRole::Tone, "Air & wall high absorption"))
            .with_param(DspParamSchema::knob("mix", "Dry / Wet Mix", 0.0, 1.0, 0.25, "%", MacroRole::Space, "Wet reverb ratio"))
        );

        self.register(
            DspNodeDescriptor::new(
                "ConvolutionReverbNode",
                "True Impulse Convolution Reverb",
                DspNodeCategory::TimeSpace,
                "FFT-partitioned zero-latency convolution reverb with real-world acoustic spaces.",
            )
            .with_param(DspParamSchema::knob("decay", "Decay Scale", 0.1, 3.0, 1.0, "x", MacroRole::Space, "Impulse decay envelope scaling"))
            .with_param(DspParamSchema::knob("mix", "Dry / Wet Mix", 0.0, 1.0, 0.35, "%", MacroRole::Space, "Wet level"))
        );

        self.register(
            DspNodeDescriptor::new(
                "BbdChorus",
                "Bucket Brigade Device (BBD) Chorus",
                DspNodeCategory::TimeSpace,
                "Analog MN3007 BBD delay line chorus with warm compander filtering.",
            )
            .with_param(DspParamSchema::knob("rate", "Modulation Rate", 0.1, 10.0, 1.2, "Hz", MacroRole::Character, "LFO sweep speed"))
            .with_param(DspParamSchema::knob("depth", "Chorus Depth", 0.0, 1.0, 0.65, "%", MacroRole::Space, "Delay sweep excursion width"))
            .with_param(DspParamSchema::knob("mix", "Wet Mix", 0.0, 1.0, 0.5, "%", MacroRole::Space, "Wet/dry blend"))
        );

        // ==========================================
        // 7. Modulators & Synthesizer Composites
        // ==========================================
        self.register(
            DspNodeDescriptor::new(
                "AetherSynth",
                "Aether Dual-Stack Subtractive Synth",
                DspNodeCategory::CompositeSynth,
                "Dual band-limited saw/pulse stack with 4-pole SVF, ADSR, and LFO modulation.",
            )
            .with_param(DspParamSchema::knob("osc_mix", "Oscillator Mix", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Balance between Saw and Pulse"))
            .with_param(DspParamSchema::log_knob("cutoff", "Filter Cutoff", 20.0, 20000.0, 2200.0, "Hz", MacroRole::Tone, "SVF Cutoff frequency"))
            .with_param(DspParamSchema::knob("resonance", "Resonance", 0.0, 1.0, 0.35, "%", MacroRole::Character, "SVF Resonance Q"))
            .with_param(DspParamSchema::knob("attack", "Env Attack", 0.001, 2.0, 0.02, "s", MacroRole::Punch, "ADSR Attack"))
            .with_param(DspParamSchema::knob("decay", "Env Decay", 0.01, 5.0, 0.3, "s", MacroRole::Punch, "ADSR Decay"))
        );

        self.register(
            DspNodeDescriptor::new(
                "FmOperatorPair",
                "2-Operator FM Synthesizer",
                DspNodeCategory::CompositeSynth,
                "Chowning-style Frequency Modulation carrier-modulator operator pair.",
            )
            .with_param(DspParamSchema::knob("ratio_1", "Modulator Ratio", 0.5, 16.0, 2.0, "x", MacroRole::Tone, "Frequency multiplier for Modulator"))
            .with_param(DspParamSchema::knob("fm_depth", "FM Modulation Depth", 0.0, 10.0, 2.5, "", MacroRole::Character, "FM modulation index / sidebands"))
        );

        self.register(
            DspNodeDescriptor::new(
                "GranularSynthNode",
                "Asynchronous Granular Synthesizer",
                DspNodeCategory::CompositeSynth,
                "Real-time sample buffer granular cloud generator with spray, density, and pitch randomization.",
            )
            .with_param(DspParamSchema::knob("density", "Grain Density", 1.0, 100.0, 25.0, "hz", MacroRole::Punch, "Grains per second"))
            .with_param(DspParamSchema::knob("spray", "Position Spray", 0.0, 1.0, 0.2, "%", MacroRole::Space, "Grain start position jitter"))
            .with_param(DspParamSchema::knob("grain_size", "Grain Size", 0.01, 0.5, 0.06, "s", MacroRole::Tone, "Duration per grain window"))
        );

        self.register(
            DspNodeDescriptor::new(
                "DrumMachineDevice",
                "8-Pad Percussion Drum Synth",
                DspNodeCategory::SamplerSlicer,
                "8-voice synthetic percussion synthesis with pitch snap, body resonance, and noise snare snap.",
            )
            .with_param(DspParamSchema::knob("kick_pitch", "Kick Base Pitch", 30.0, 120.0, 55.0, "Hz", MacroRole::Punch, "Bass drum frequency"))
            .with_param(DspParamSchema::knob("snare_snap", "Snare Noise Snap", 0.0, 1.0, 0.75, "%", MacroRole::Tone, "Snare wire noise mix"))
        );

        self.register(
            DspNodeDescriptor::new(
                "SamplerDevice",
                "Multi-Sample SFZ Player",
                DspNodeCategory::SamplerSlicer,
                "Polyphonic multi-velocity sample playback engine with root tracking and envelope.",
            )
            .with_param(DspParamSchema::log_knob("cutoff", "Sample Filter Cutoff", 20.0, 20000.0, 20000.0, "Hz", MacroRole::Tone, "Post-sample filter"))
        );

        self.register(
            DspNodeDescriptor::new(
                "AutoWah",
                "Envelope Auto-Wah / Envelope Follower",
                DspNodeCategory::DistortionSaturation,
                "Dynamic band-pass filter tracking input transient envelope for funk/disco wah.",
            )
            .with_param(DspParamSchema::knob("sensitivity", "Sensitivity", 0.1, 5.0, 1.5, "x", MacroRole::Punch, "Envelope peak tracking gain"))
            .with_param(DspParamSchema::knob("range", "Filter Sweep Range", 200.0, 5000.0, 2500.0, "Hz", MacroRole::Tone, "Wah bandpass sweep width"))
        );

        self.register(
            DspNodeDescriptor::new(
                "SpatialAudioPanner",
                "3D Binaural Spatial Audio Panner",
                DspNodeCategory::SpatialSurround,
                "Spherical HRTF spatial audio positioning with distance attenuation and Doppler shift.",
            )
            .with_param(DspParamSchema::knob("azimuth", "Azimuth Angle", -180.0, 180.0, 0.0, "°", MacroRole::Space, "Horizontal left/right orbit angle"))
            .with_param(DspParamSchema::knob("elevation", "Elevation Angle", -90.0, 90.0, 0.0, "°", MacroRole::Space, "Vertical up/down angle"))
            .with_param(DspParamSchema::knob("distance", "Source Distance", 0.1, 50.0, 2.0, "m", MacroRole::Space, "Distance from listener"))
        );

        // ==========================================
        // 8. Exotic & Quantum Generators
        // ==========================================
        self.register(
            DspNodeDescriptor::new(
                "PlasmaArcSynthesizer",
                "High-Voltage Plasma Arc Synthesizer",
                DspNodeCategory::Oscillator,
                "Electrical discharge plasma arc generator with thermal air compression and ionic breakdown.",
            )
            .with_param(DspParamSchema::log_knob("freq", "Arc Carrier Freq", 20.0, 20000.0, 440.0, "Hz", MacroRole::Tone, "Primary spark frequency"))
            .with_param(DspParamSchema::knob("spark_intensity", "Spark Current", 0.0, 1.0, 0.75, "%", MacroRole::Punch, "Current discharge intensity"))
            .with_param(DspParamSchema::knob("gap_distance", "Electrode Gap", 0.5, 20.0, 4.0, "mm", MacroRole::Character, "Electrode spark jump distance"))
        );

        self.register(
            DspNodeDescriptor::new(
                "QuantumStateVectorOscillator",
                "Quantum State Vector Oscillator",
                DspNodeCategory::Oscillator,
                "Unitary Schrödinger state vector rotation in Hilbert space with phase probability projection.",
            )
            .with_param(DspParamSchema::knob("energy_quanta", "Hamiltonian Energy", 0.1, 10.0, 1.0, "ħω", MacroRole::Tone, "Quantum eigenvalue scale"))
            .with_param(DspParamSchema::knob("superposition", "Superposition Angle", 0.0, 3.14159, 0.785, "rad", MacroRole::Character, "Qubit superposition mixing angle"))
            .with_param(DspParamSchema::knob("decoherence", "Decoherence Noise", 0.0, 1.0, 0.05, "%", MacroRole::Space, "Environmental quantum entanglement decay"))
        );

        self.register(
            DspNodeDescriptor::new(
                "SubharmonicSynth",
                "Subharmonic Generator & Octaver",
                DspNodeCategory::Oscillator,
                "Sub-octave divider synthesizing 1st and 2nd subharmonics with low-frequency wave shaping.",
            )
            .with_param(DspParamSchema::knob("sub1_level", "Sub -1 Octave", 0.0, 1.0, 0.6, "%", MacroRole::Punch, "First subharmonic level"))
            .with_param(DspParamSchema::knob("sub2_level", "Sub -2 Octaves", 0.0, 1.0, 0.4, "%", MacroRole::Punch, "Second subharmonic level"))
            .with_param(DspParamSchema::log_knob("lowpass", "Sub Lowpass", 40.0, 500.0, 120.0, "Hz", MacroRole::Tone, "Subharmonic filter cutoff"))
        );

        // ==========================================
        // 9. World Acoustic Physical Models
        // ==========================================
        self.register(
            DspNodeDescriptor::new(
                "GamelanGender",
                "Indonesian Gamelan Gendèr",
                DspNodeCategory::AcousticPhysicalModel,
                "14-key bronze metallophone with tuned bamboo resonators and paired ombak acoustic beating.",
            )
            .with_param(DspParamSchema::knob("mallet_hardness", "Panggul Hardness", 0.05, 1.0, 0.45, "%", MacroRole::Punch, "Padded mallet tip hardness"))
            .with_param(DspParamSchema::knob("ombak_rate", "Ombak Beating Rate", 2.0, 12.0, 6.5, "Hz", MacroRole::Character, "Pengisep/pengumbang beating frequency"))
            .with_param(DspParamSchema::knob("bamboo_q", "Bamboo Resonator Q", 5.0, 60.0, 28.0, "", MacroRole::Space, "Bamboo tube cavity resonance quality factor"))
            .with_param(DspParamSchema::knob("grasp_damping", "Hand Damping (Mipil)", 0.0, 1.0, 0.0, "%", MacroRole::Tone, "Thumb and index finger key damping"))
        );

        self.register(
            DspNodeDescriptor::new(
                "HurdyGurdy",
                "Vielle à Roue Hurdy Gurdy",
                DspNodeCategory::AcousticPhysicalModel,
                "Wooden cranked rosin wheel driving melody chanters, bourdon drones, and buzzing trompette chien.",
            )
            .with_param(DspParamSchema::knob("wheel_speed", "Crank Wheel RPM", 0.0, 200.0, 90.0, "RPM", MacroRole::Tone, "Rosin wheel cranking speed"))
            .with_param(DspParamSchema::knob("rosin_friction", "Rosin Stick-Slip", 0.1, 1.0, 0.65, "", MacroRole::Character, "Rosin coating contact friction"))
            .with_param(DspParamSchema::knob("chien_buzz", "Chien Buzz Sensitivity", 0.0, 1.0, 0.75, "%", MacroRole::Punch, "Trompette loose buzzing bridge sensitivity"))
            .with_param(DspParamSchema::knob("drone_level", "Bourdon Drones", 0.0, 1.0, 0.80, "%", MacroRole::Space, "Drone strings volume mix"))
        );

        self.register(
            DspNodeDescriptor::new(
                "SitarModel",
                "Indian Sitar & Jawari Bridge",
                DspNodeCategory::AcousticPhysicalModel,
                "Curved deer horn / bone Jawari bridge non-linear dynamic buzzing and 13 sympathetic Tarab strings.",
            )
            .with_param(DspParamSchema::knob("jawari_curvature", "Jawari Bridge Arc", 0.01, 1.0, 0.35, "", MacroRole::Character, "Curved bridge buzz contact profile"))
            .with_param(DspParamSchema::knob("tarab_coupling", "Tarab Sympathetic", 0.0, 1.0, 0.70, "%", MacroRole::Space, "Resonant sympathetic strings coupling"))
            .with_param(DspParamSchema::knob("meend_bend", "Meend Microtonal Bend", -7.0, 7.0, 0.0, "st", MacroRole::Tone, "Lateral string pulling bend"))
            .with_param(DspParamSchema::knob("mizrab_strike", "Mizrab Plectrum", 0.0, 1.0, 0.85, "%", MacroRole::Punch, "Wire plectrum attack sharpness"))
        );

        self.register(
            DspNodeDescriptor::new(
                "KotoModel",
                "Japanese 13-String Koto",
                DspNodeCategory::AcousticPhysicalModel,
                "Paulownia wood resonant body with movable ivory Ji bridges and Oshi-de pitch bending.",
            )
            .with_param(DspParamSchema::knob("ji_coupling", "Ji Bridge Coupling", 0.1, 1.0, 0.6, "%", MacroRole::Character, "Bridge acoustic energy transmission"))
            .with_param(DspParamSchema::knob("body_resonance", "Paulownia Body", 0.0, 1.0, 0.75, "%", MacroRole::Space, "Hollow soundboard cavity warmth"))
            .with_param(DspParamSchema::knob("tsume_hardness", "Tsume Plectrum", 0.1, 1.0, 0.7, "%", MacroRole::Punch, "Finger pick hardness"))
        );

        self.register(
            DspNodeDescriptor::new(
                "SteelpanDrum",
                "Trinidad Steelpan Drum",
                DspNodeCategory::AcousticPhysicalModel,
                "Sunken 55-gallon oil drum concave dish with tuned acoustic membrane notes and modal shell coupling.",
            )
            .with_param(DspParamSchema::knob("stick_hardness", "Rubber Stick Tip", 0.1, 1.0, 0.5, "%", MacroRole::Punch, "Rubber mallet density"))
            .with_param(DspParamSchema::knob("skirt_resonance", "Steel Skirt Resonance", 0.0, 1.0, 0.45, "%", MacroRole::Space, "Outer drum cylinder ring resonance"))
            .with_param(DspParamSchema::knob("harmonic_coupling", "Harmonic Coupling", 0.0, 1.0, 0.6, "%", MacroRole::Character, "Octave and fifth sympathetic vibration"))
        );

        self.register(
            DspNodeDescriptor::new(
                "MbiraKalimba",
                "African Mbira / Kalimba Lamellophone",
                DspNodeCategory::AcousticPhysicalModel,
                "Forged iron spring tines mounted on hardwood soundboard with gourd resonator and bottlecap buzz.",
            )
            .with_param(DspParamSchema::knob("tine_decay", "Tine Ring Decay", 0.2, 5.0, 1.8, "s", MacroRole::Space, "Metal key sustain time"))
            .with_param(DspParamSchema::knob("bottle_cap_buzz", "Machachara Buzz", 0.0, 1.0, 0.4, "%", MacroRole::Character, "Sympathetic shell / bottlecap rattle"))
            .with_param(DspParamSchema::knob("soundboard_size", "Deze Gourd Volume", 0.5, 3.0, 1.2, "x", MacroRole::Tone, "Amplifying calabash gourd resonance"))
        );

        self.register(
            DspNodeDescriptor::new(
                "DulcimerCimbalom",
                "Hammered Dulcimer & Cimbalom",
                DspNodeCategory::AcousticPhysicalModel,
                "Multi-string unison courses struck with wooden/leather mallets across split chessman bridges.",
            )
            .with_param(DspParamSchema::knob("hammer_material", "Mallet Leather/Wood", 0.0, 1.0, 0.35, "%", MacroRole::Punch, "0=Soft leather, 1=Hard bare wood"))
            .with_param(DspParamSchema::knob("pedal_damper", "Damper Pedal", 0.0, 1.0, 0.0, "%", MacroRole::Space, "Felt damper bar contact"))
            .with_param(DspParamSchema::knob("bridge_split", "Chessman Split", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Fifth-interval bridge position balance"))
        );

        self.register(
            DspNodeDescriptor::new(
                "FreeReedAccordion",
                "Cassotto Free Reed Accordion",
                DspNodeCategory::AcousticPhysicalModel,
                "Steel free reeds vibrating through brass reedplates with dual-tone cassotto wooden resonance chamber.",
            )
            .with_param(DspParamSchema::knob("bellows_pressure", "Bellows Pressure", 10.0, 500.0, 120.0, "Pa", MacroRole::Punch, "Air pumping velocity"))
            .with_param(DspParamSchema::knob("cassotto_mix", "Cassotto Chamber Mix", 0.0, 1.0, 0.7, "%", MacroRole::Tone, "Mellow wood chamber tone blend"))
            .with_param(DspParamSchema::knob("musette_detune", "Musette Beating", 0.0, 25.0, 6.0, "cents", MacroRole::Character, "French musette reed detuning"))
        );

        self.register(
            DspNodeDescriptor::new(
                "SpringLatticeReverb",
                "3D Helical Spring Lattice Reverb",
                DspNodeCategory::AcousticPhysicalModel,
                "Nonlinear multi-spring mechanical lattice network with torsional dispersion and boing transient response.",
            )
            .with_param(DspParamSchema::knob("spring_tension", "Spring Tension", 0.1, 2.0, 1.0, "x", MacroRole::Tone, "Helical coil stiffness and chirp dispersion"))
            .with_param(DspParamSchema::knob("boing_amount", "Boing Sharpness", 0.0, 1.0, 0.55, "%", MacroRole::Punch, "Transient mechanical dispersion intensity"))
            .with_param(DspParamSchema::knob("decay_time", "Reverb Decay", 0.5, 8.0, 3.2, "s", MacroRole::Space, "Lattice acoustic absorption rate"))
            .with_param(DspParamSchema::knob("mix", "Dry / Wet Mix", 0.0, 1.0, 0.35, "%", MacroRole::Space, "Wet output blend"))
        );

        self.register(
            DspNodeDescriptor::new(
                "PlateReverbTank",
                "Dispersion Plate Reverb Tank",
                DspNodeCategory::AcousticPhysicalModel,
                "Heavy cold-rolled steel plate suspension with piezoceramic driver and dual stereophonic pickups.",
            )
            .with_param(DspParamSchema::knob("damper_distance", "Felt Damper Gap", 0.0, 1.0, 0.65, "%", MacroRole::Space, "Decay time control via damper plate proximity"))
            .with_param(DspParamSchema::knob("plate_thickness", "Plate Thickness", 0.5, 3.0, 1.5, "mm", MacroRole::Tone, "High frequency modal dispersion density"))
            .with_param(DspParamSchema::knob("drive", "Transducer Overdrive", 0.0, 12.0, 0.0, "dB", MacroRole::Punch, "Input driver coil saturation"))
            .with_param(DspParamSchema::knob("mix", "Wet Mix", 0.0, 1.0, 0.30, "%", MacroRole::Space, "Reverberation wet blend"))
        );

        // ==========================================
        // 10. Advanced Filters & Resonators
        // ==========================================
        self.register(
            DspNodeDescriptor::new(
                "FormantFilterNode",
                "5-Vowel Formant Matrix Filter",
                DspNodeCategory::FilterEq,
                "Triple band-pass resonator matrix morphing smoothly between vowel formants A, E, I, O, and U.",
            )
            .with_param(DspParamSchema::knob("vowel_morph", "Vowel Position", 0.0, 4.0, 0.0, "", MacroRole::Tone, "0=A, 1=E, 2=I, 3=O, 4=U"))
            .with_param(DspParamSchema::knob("formant_shift", "Formant Shift", -12.0, 12.0, 0.0, "st", MacroRole::Character, "Overall formant scaling (gender/size)"))
            .with_param(DspParamSchema::knob("resonance_q", "Formant Q", 2.0, 30.0, 10.0, "", MacroRole::Character, "Formant peak sharpness"))
        );

        self.register(
            DspNodeDescriptor::new(
                "CombResonatorNode",
                "Tunable Feedback Comb Resonator",
                DspNodeCategory::FilterEq,
                "Feedback and feedforward delay comb filter creating pitched harmonic resonance spikes.",
            )
            .with_param(DspParamSchema::log_knob("freq", "Tuning Frequency", 20.0, 5000.0, 220.0, "Hz", MacroRole::Tone, "Comb fundamental harmonic frequency"))
            .with_param(DspParamSchema::knob("feedback", "Resonance Feedback", -0.99, 0.99, 0.85, "%", MacroRole::Character, "Feedback loop gain (positive/negative)"))
            .with_param(DspParamSchema::knob("damping", "High Damping", 0.0, 1.0, 0.2, "%", MacroRole::Space, "Internal high frequency loss"))
        );

        self.register(
            DspNodeDescriptor::new(
                "LinearPhaseCrossover",
                "Mastering Linear-Phase Crossover",
                DspNodeCategory::FilterEq,
                "Zero phase distortion FIR crossover dividing audio into Low, Mid, and High frequency bands.",
            )
            .with_param(DspParamSchema::log_knob("low_mid_freq", "Low-Mid Crossover", 40.0, 1000.0, 180.0, "Hz", MacroRole::Tone, "Bass crossover split point"))
            .with_param(DspParamSchema::log_knob("mid_high_freq", "Mid-High Crossover", 500.0, 12000.0, 3500.0, "Hz", MacroRole::Tone, "Treble crossover split point"))
        );

        // ==========================================
        // 11. Mastering & Dynamic Processors
        // ==========================================
        self.register(
            DspNodeDescriptor::new(
                "TruePeakMasterLimiter",
                "ITU-R BS.1770 True-Peak Limiter",
                DspNodeCategory::DynamicsMaster,
                "Mastering grade 8x oversampled true-peak brickwall limiter with adaptive release windowing.",
            )
            .with_param(DspParamSchema::knob("ceiling_dbfs", "Ceiling Level", -6.0, 0.0, -0.3, "dBTP", MacroRole::Punch, "Maximum permissible peak level"))
            .with_param(DspParamSchema::knob("threshold_db", "Limiter Threshold", -18.0, 0.0, -3.0, "dB", MacroRole::Punch, "Input driving gain threshold"))
            .with_param(DspParamSchema::knob("release_ms", "Adaptive Release", 5.0, 500.0, 45.0, "ms", MacroRole::Space, "Dynamic gain recovery time"))
            .with_param(DspParamSchema::toggle("isp_oversample", "8x ISP Anti-Alias", true, "Inter-sample peak detection oversampling"))
        );

        self.register(
            DspNodeDescriptor::new(
                "EbuLoudnessRadar",
                "EBU R128 / LUFS Loudness Radar",
                DspNodeCategory::DynamicsMaster,
                "Integrated, Short-Term, and Momentary LUFS loudness radar visualizer and target auto-fader.",
            )
            .with_param(DspParamSchema::knob("target_lufs", "Target LUFS", -24.0, -8.0, -14.0, "LUFS", MacroRole::Punch, "Target delivery loudness standard"))
            .with_param(DspParamSchema::knob("max_true_peak", "Max True Peak", -3.0, 0.0, -1.0, "dBTP", MacroRole::Punch, "True-peak ceiling guard threshold"))
        );

        self.register(
            DspNodeDescriptor::new(
                "OpticalCompressorNode",
                "Vintage Teletronix LA-2A Optical Leveler",
                DspNodeCategory::DynamicsMaster,
                "Electro-luminescent photocell optical attenuator with smooth two-stage release curve.",
            )
            .with_param(DspParamSchema::knob("peak_reduction", "Peak Reduction", 0.0, 100.0, 45.0, "%", MacroRole::Punch, "Optical photocell gain reduction amount"))
            .with_param(DspParamSchema::knob("gain", "Makeup Output Gain", 0.0, 40.0, 10.0, "dB", MacroRole::Punch, "Post-compression tube output gain"))
            .with_param(DspParamSchema::choice("mode", "Operating Mode", &["Compress (3:1)", "Limit (Infinity:1)"], 0, "Optical response curve"))
        );

        self.register(
            DspNodeDescriptor::new(
                "VariMuMasterCompressor",
                "Variable-Mu Tube Mastering Compressor",
                DspNodeCategory::DynamicsMaster,
                "Fairchild 670 style variable-mu triode tube compressor with program-dependent attack/release.",
            )
            .with_param(DspParamSchema::knob("threshold", "Threshold", -30.0, 0.0, -12.0, "dB", MacroRole::Punch, "Compression trigger level"))
            .with_param(DspParamSchema::knob("input_drive", "Input Tube Drive", 0.0, 12.0, 2.0, "dB", MacroRole::Character, "Triode input coloration"))
            .with_param(DspParamSchema::choice("time_constant", "Time Constant", &["1 (Fast 0.2s)", "2 (Medium 0.5s)", "3 (Auto 1.0s)", "4 (Master 2.0s)", "5 (Master Auto 5.0s)", "6 (Semi-Auto)"], 2, "Fairchild time constant profile"))
        );

        self.register(
            DspNodeDescriptor::new(
                "DynamicCrestShaper",
                "Dynamic Crest Factor Shaper",
                DspNodeCategory::DynamicsMaster,
                "Intelligent peak-to-RMS crest factor controller tightening low-end punch without crushing dynamics.",
            )
            .with_param(DspParamSchema::knob("target_crest", "Target Crest Factor", 6.0, 20.0, 12.0, "dB", MacroRole::Punch, "Desired Peak-to-RMS dynamic ratio"))
            .with_param(DspParamSchema::knob("transient_protect", "Transient Protection", 0.0, 1.0, 0.8, "%", MacroRole::Tone, "Preserve initial drum transient strikes"))
        );

        self.register(
            DspNodeDescriptor::new(
                "TransientDesignerNode",
                "Transient Attack & Sustain Shaper",
                DspNodeCategory::DynamicsMaster,
                "Level-independent differential envelope shaper boosting or cutting attack punch and sustain ring.",
            )
            .with_param(DspParamSchema::knob("attack", "Attack Punch", -24.0, 24.0, 4.0, "dB", MacroRole::Punch, "Transient onset snap boost/cut"))
            .with_param(DspParamSchema::knob("sustain", "Sustain Tail", -24.0, 24.0, -2.0, "dB", MacroRole::Space, "Body and tail sustain boost/cut"))
            .with_param(DspParamSchema::knob("gain", "Output Gain", -12.0, 12.0, 0.0, "dB", MacroRole::Punch, "Output makeup gain"))
        );

        self.register(
            DspNodeDescriptor::new(
                "MultibandClipperNode",
                "3-Band Harmonic Soft/Hard Clipper",
                DspNodeCategory::DynamicsMaster,
                "Split-band soft-knee / hard mastering clipper maximizing perceived loudness cleanly.",
            )
            .with_param(DspParamSchema::knob("low_clip", "Low Band Ceiling", -12.0, 0.0, -0.5, "dB", MacroRole::Punch, "Sub & bass clipping limit"))
            .with_param(DspParamSchema::knob("mid_clip", "Mid Band Ceiling", -12.0, 0.0, -0.2, "dB", MacroRole::Tone, "Midrange vocal clipping limit"))
            .with_param(DspParamSchema::knob("high_clip", "High Band Ceiling", -12.0, 0.0, 0.0, "dB", MacroRole::Character, "Treble & air clipping limit"))
        );

        // ==========================================
        // 12. Saturation & Coloration
        // ==========================================
        self.register(
            DspNodeDescriptor::new(
                "TapeFluxMaster",
                "Master 1/2\" 2-Track Tape Machine",
                DspNodeCategory::DistortionSaturation,
                "Ampex ATR-102 studio mastering tape machine with magnetic head bump and flux compression.",
            )
            .with_param(DspParamSchema::knob("input_flux", "Tape Flux Level", 185.0, 520.0, 355.0, "nWb/m", MacroRole::Punch, "Reference magnetic flux level"))
            .with_param(DspParamSchema::knob("bias_level", "HF Bias Current", -6.0, 6.0, 1.5, "dB", MacroRole::Tone, "High-frequency AC bias calibration"))
            .with_param(DspParamSchema::choice("tape_formula", "Tape Formulation", &["GP9 (Clean Punch)", "456 (Warm Vintage)", "499 (High Output)"], 0, "Magnetic tape formulation"))
        );

        self.register(
            DspNodeDescriptor::new(
                "HarmonicExciterNode",
                "Aural Harmonic Exciter & Air",
                DspNodeCategory::DistortionSaturation,
                "Psychoacoustic high-frequency even/odd harmonic generator adding sparkling air and presence.",
            )
            .with_param(DspParamSchema::log_knob("freq", "High-Pass Tune", 1000.0, 15000.0, 4500.0, "Hz", MacroRole::Tone, "Excitation frequency floor"))
            .with_param(DspParamSchema::knob("drive", "Harmonics Drive", 0.0, 10.0, 3.0, "x", MacroRole::Punch, "Nonlinear harmonic generation intensity"))
            .with_param(DspParamSchema::knob("mix", "Exciter Amount", 0.0, 1.0, 0.25, "%", MacroRole::Character, "Wet harmonic addition blend"))
        );

        self.register(
            DspNodeDescriptor::new(
                "ThroughZeroFlanger",
                "Through-Zero Barberpole Flanger",
                DspNodeCategory::DistortionSaturation,
                "Dual tape delay flanging passing through exact time zero for deep cancellation jet sweep.",
            )
            .with_param(DspParamSchema::knob("rate", "Flange LFO Rate", 0.02, 5.0, 0.15, "Hz", MacroRole::Tone, "Sweep cycle speed"))
            .with_param(DspParamSchema::knob("tz_offset", "Zero Crossing Offset", -2.0, 2.0, 0.0, "ms", MacroRole::Character, "Center time alignment across 0ms"))
            .with_param(DspParamSchema::knob("feedback", "Regeneration Feedback", -0.98, 0.98, 0.80, "%", MacroRole::Character, "Resonant feedback path intensity"))
        );

        self.register(
            DspNodeDescriptor::new(
                "RotarySpeakerNode",
                "Leslie Dual-Rotor Speaker Emulation",
                DspNodeCategory::DistortionSaturation,
                "Rotating high-frequency horn and low-frequency bass drum with Doppler shift and cabinet reflection.",
            )
            .with_param(DspParamSchema::choice("speed", "Rotor Speed", &["Brake (Stop)", "Chorale (Slow)", "Tremolo (Fast)"], 1, "Rotor rotation standard"))
            .with_param(DspParamSchema::knob("horn_rate", "Horn Max RPM", 100.0, 500.0, 390.0, "RPM", MacroRole::Tone, "Top treble horn rotation speed"))
            .with_param(DspParamSchema::knob("drum_rate", "Drum Max RPM", 50.0, 400.0, 340.0, "RPM", MacroRole::Punch, "Bottom bass drum rotation speed"))
            .with_param(DspParamSchema::knob("mic_distance", "Microphone Distance", 0.2, 3.0, 0.8, "m", MacroRole::Space, "Stereo boundary mic proximity"))
        );

        self.register(
            DspNodeDescriptor::new(
                "FrequencyShifterNode",
                "Bode SSB Frequency Shifter",
                DspNodeCategory::DistortionSaturation,
                "Hilbert transform single-sideband frequency shifter with independent linear pitch translation.",
            )
            .with_param(DspParamSchema::knob("shift_hz", "Frequency Shift", -2000.0, 2000.0, 5.0, "Hz", MacroRole::Tone, "Linear Hz frequency displacement"))
            .with_param(DspParamSchema::knob("feedback", "Shifter Feedback", 0.0, 0.95, 0.0, "%", MacroRole::Character, "Feedback loop for barberpole spirals"))
        );

        // ==========================================
        // 13. Spatial & 3D Audio Immersive Panners
        // ==========================================
        self.register(
            DspNodeDescriptor::new(
                "Atmos916Spatializer",
                "Dolby Atmos 9.1.6 Object Spatializer",
                DspNodeCategory::SpatialSurround,
                "16-channel immersive surround sound object panner with height layer and binaural HRIR downmixing.",
            )
            .with_param(DspParamSchema::knob("azimuth", "Azimuth Orbit", -180.0, 180.0, 0.0, "°", MacroRole::Space, "Horizontal angular position"))
            .with_param(DspParamSchema::knob("elevation", "Elevation Height", -90.0, 90.0, 0.0, "°", MacroRole::Space, "Vertical elevation layer"))
            .with_param(DspParamSchema::knob("distance", "Object Distance", 0.2, 30.0, 2.5, "m", MacroRole::Space, "Distance from central listening sweetspot"))
            .with_param(DspParamSchema::knob("size", "Object Size / Spread", 0.0, 1.0, 0.0, "%", MacroRole::Tone, "Apparent point source spatial spread"))
        );

        self.register(
            DspNodeDescriptor::new(
                "AmbisonicRadarSpatializer",
                "Higher-Order Ambisonics (HOA-5) Radar",
                DspNodeCategory::SpatialSurround,
                "36-channel 5th order spherical harmonics spatial encoder with real-time energy radar display.",
            )
            .with_param(DspParamSchema::knob("order", "Ambisonic Order", 1.0, 5.0, 3.0, "", MacroRole::Character, "Spherical harmonics order resolution"))
            .with_param(DspParamSchema::knob("focus", "Directional Focus", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Beamforming spatial sharpness"))
        );

        self.register(
            DspNodeDescriptor::new(
                "Auro3dSpatializer",
                "Auro-3D 13.1 Tri-Layer Immersive Panner",
                DspNodeCategory::SpatialSurround,
                "3-layer spatial audio panner with Ear-Level, Height, and Voice-of-God top ceiling channels.",
            )
            .with_param(DspParamSchema::knob("layer_blend", "Height Layer Blend", 0.0, 1.0, 0.5, "%", MacroRole::Space, "Vertical height energy distribution"))
            .with_param(DspParamSchema::knob("ceiling_send", "Voice of God Send", 0.0, 1.0, 0.0, "%", MacroRole::Space, "Top zenith overhead ceiling speaker send"))
        );

        self.register(
            DspNodeDescriptor::new(
                "Nhk222Spatializer",
                "NHK 22.2 Super Hi-Vision Panner",
                DspNodeCategory::SpatialSurround,
                "24-channel multi-layer broadcast matrix spatializer for 22.2 speaker dome arrays.",
            )
            .with_param(DspParamSchema::knob("dome_radius", "Dome Radius", 1.0, 20.0, 5.0, "m", MacroRole::Space, "Speaker sphere dome scale"))
        );

        self.register(
            DspNodeDescriptor::new(
                "BinauralBrirSpatializer",
                "Binaural Room Impulse Response (BRIR)",
                DspNodeCategory::SpatialSurround,
                "Head-Related Transfer Function with ear canal acoustics and real acoustic room reflections.",
            )
            .with_param(DspParamSchema::choice("head_model", "HRTF Profile", &["KEMAR Dummy Head", "Genelec Generic", "Spherical Head Model"], 0, "Ear morphology HRTF model"))
            .with_param(DspParamSchema::knob("room_damp", "Room Absorption", 0.0, 1.0, 0.35, "%", MacroRole::Space, "Early reflection room wall absorption"))
        );

        // ==========================================
        // 14. Spectral Resynthesis & Restoration
        // ==========================================
        self.register(
            DspNodeDescriptor::new(
                "SpectralMorphNode",
                "FFT Spectral Morphing Processor",
                DspNodeCategory::SpectralResynthesis,
                "Phase-vocoder spectral envelope interpolator blending magnitude and phase between carrier and modulator.",
            )
            .with_param(DspParamSchema::knob("morph", "Spectral Morph", 0.0, 1.0, 0.5, "%", MacroRole::Character, "Interpolation ratio between source A and B"))
            .with_param(DspParamSchema::knob("spectral_tilt", "Spectral Tilt", -6.0, 6.0, 0.0, "dB/oct", MacroRole::Tone, "Linear slope spectral tilt"))
            .with_param(DspParamSchema::choice("fft_size", "FFT Window Size", &["512 (Fast/Percussive)", "1024 (Balanced)", "2048 (High Res)", "4096 (Tonal)"], 2, "Spectral bin resolution"))
        );

        self.register(
            DspNodeDescriptor::new(
                "SpectralReshaperNode",
                "Spectral Envelope Reshaper",
                DspNodeCategory::SpectralResynthesis,
                "Surgical spectral formant sculpting and resonant peak boosting/attenuation in the frequency domain.",
            )
            .with_param(DspParamSchema::knob("formant_warp", "Formant Warp", -12.0, 12.0, 0.0, "st", MacroRole::Tone, "Frequency axis compression/expansion"))
            .with_param(DspParamSchema::knob("smooth_bins", "Bin Smoothing", 1.0, 32.0, 4.0, "bins", MacroRole::Character, "Spectral peak smoothing width"))
        );

        self.register(
            DspNodeDescriptor::new(
                "SpectralDebleedNode",
                "Spectral Drum & Mic Debleeder",
                DspNodeCategory::SpectralResynthesis,
                "Intelligent spectral gating removing acoustic microphone bleed from cymbals and snares.",
            )
            .with_param(DspParamSchema::knob("bleed_threshold", "Debleed Threshold", -40.0, 0.0, -18.0, "dB", MacroRole::Punch, "Bleed rejection floor"))
            .with_param(DspParamSchema::knob("suppression", "Bleed Suppression", 0.0, 40.0, 18.0, "dB", MacroRole::Character, "Amount of background bleed attenuation"))
        );

        self.register(
            DspNodeDescriptor::new(
                "SpectralDeesserNode",
                "Dynamic Spectral Vocal De-Esser",
                DspNodeCategory::SpectralResynthesis,
                "Zero-artifact spectral suppression of sibilant vocal frequencies (4kHz - 10kHz).",
            )
            .with_param(DspParamSchema::log_knob("sibilance_freq", "Sibilance Center", 3000.0, 12000.0, 6500.0, "Hz", MacroRole::Tone, "Target sibilant frequency center"))
            .with_param(DspParamSchema::knob("reduction", "Max De-Ess Reduction", 0.0, 24.0, 6.0, "dB", MacroRole::Punch, "Maximum sibilance attenuation limit"))
        );

        self.register(
            DspNodeDescriptor::new(
                "SpectrogramArtNode",
                "Spectrogram Image-to-Audio Synth",
                DspNodeCategory::SpectralResynthesis,
                "Renders visual bitmap images and drawings directly into audio frequency spectrum harmonics.",
            )
            .with_param(DspParamSchema::log_knob("min_freq", "Bottom Frequency", 40.0, 1000.0, 100.0, "Hz", MacroRole::Tone, "Bottom image pixel frequency"))
            .with_param(DspParamSchema::log_knob("max_freq", "Top Frequency", 2000.0, 20000.0, 12000.0, "Hz", MacroRole::Tone, "Top image pixel frequency"))
            .with_param(DspParamSchema::knob("contrast", "Pixel Contrast", 0.5, 4.0, 1.5, "x", MacroRole::Punch, "Luminance to volume power curve"))
        );

        // ==========================================
        // 15. Neural & AI Generative Models
        // ==========================================
        self.register(
            DspNodeDescriptor::new(
                "NeuralWavetable",
                "Latent AI Neural Wavetable Synth",
                DspNodeCategory::NeuralAi,
                "Deep generative neural network mapping 8-dimensional latent timbre coordinates to single-cycle wavetables.",
            )
            .with_param(DspParamSchema::knob("latent_z1", "Timbre Dimension 1 (Warmth)", -3.0, 3.0, 0.0, "", MacroRole::Tone, "Latent axis 1: Harmonic spectral slope"))
            .with_param(DspParamSchema::knob("latent_z2", "Timbre Dimension 2 (Bite)", -3.0, 3.0, 0.0, "", MacroRole::Character, "Latent axis 2: Odd harmonic resonance"))
            .with_param(DspParamSchema::knob("latent_z3", "Timbre Dimension 3 (Acoustic)", -3.0, 3.0, 0.0, "", MacroRole::Space, "Latent axis 3: Wood/metal physical character"))
        );

        self.register(
            DspNodeDescriptor::new(
                "NeuralVocalStylizer",
                "Neural Voice Timbre & Accent Transfer",
                DspNodeCategory::NeuralAi,
                "Neural audio autoencoder transferring singer vocal identity and formant timbre in real-time.",
            )
            .with_param(DspParamSchema::choice("target_voice", "Target Voice Model", &["Aria Soprano", "Kaelen Tenor", "Vesper Alto", "Balthazar Bass"], 0, "Neural singer embedding target"))
            .with_param(DspParamSchema::knob("transfer_strength", "Stylization Strength", 0.0, 1.0, 0.85, "%", MacroRole::Character, "Voice replacement intensity"))
        );

        self.register(
            DspNodeDescriptor::new(
                "NeuralDereverb",
                "Deep Learning Dereverberator",
                DspNodeCategory::NeuralAi,
                "U-Net neural audio model removing reverberation and room acoustics from dry vocal recordings.",
            )
            .with_param(DspParamSchema::knob("dereverb_amt", "Dereverb Intensity", 0.0, 1.0, 0.75, "%", MacroRole::Punch, "Room reflection cancellation ratio"))
            .with_param(DspParamSchema::knob("room_tail_cut", "Early Reflection Guard", 0.0, 1.0, 0.5, "%", MacroRole::Space, "Preserve natural near-field early acoustics"))
        );

        // ==========================================
        // 16. Modulators, Samplers & Utilities
        // ==========================================
        self.register(
            DspNodeDescriptor::new(
                "LoopSlicerNode",
                "Transient Beat Slicer & Groove Remixer",
                DspNodeCategory::SamplerSlicer,
                "Automatic transient-based audio slice detection with dynamic rearrangement and tempo lock.",
            )
            .with_param(DspParamSchema::knob("sensitivity", "Slice Sensitivity", 0.1, 1.0, 0.65, "%", MacroRole::Punch, "Transient onset threshold"))
            .with_param(DspParamSchema::knob("groove_swing", "Groove Swing", 0.0, 1.0, 0.0, "%", MacroRole::Character, "16th-note swing shuffle"))
        );

        self.register(
            DspNodeDescriptor::new(
                "PitchCorrectorNode",
                "Real-Time Microtonal Auto-Tuner",
                DspNodeCategory::Modulation,
                "YIN algorithm real-time pitch detector with scale snapping and formant correction.",
            )
            .with_param(DspParamSchema::knob("speed", "Correction Speed", 0.0, 100.0, 20.0, "ms", MacroRole::Tone, "0ms=Hard robotic snap, 50ms=Natural vibrato"))
            .with_param(DspParamSchema::knob("formant_lock", "Formant Lock", 0.0, 1.0, 1.0, "%", MacroRole::Character, "Prevent chipmunk vocal distortion"))
        );

        self.register(
            DspNodeDescriptor::new(
                "EnvelopeFollowerNode",
                "Fast Transient Envelope Follower",
                DspNodeCategory::Modulation,
                "Precision RMS and peak audio envelope detector outputting control signals to modulation bus.",
            )
            .with_param(DspParamSchema::knob("attack", "Attack Speed", 0.5, 100.0, 5.0, "ms", MacroRole::Punch, "Envelope rise time"))
            .with_param(DspParamSchema::knob("release", "Release Speed", 5.0, 1000.0, 80.0, "ms", MacroRole::Space, "Envelope fall time"))
            .with_param(DspParamSchema::knob("gain", "Detector Gain", 0.1, 10.0, 1.0, "x", MacroRole::Character, "Modulation output scaling"))
        );

        // ==========================================
        // 17. Zero-Gravity Fluid Dynamics & Molecular Resonators (Tier 44)
        // ==========================================
        self.register(
            DspNodeDescriptor::new(
                "NavierStokesFluidNode",
                "Navier-Stokes 3D Acoustic Fluid Dynamics",
                DspNodeCategory::AcousticPhysicalModel,
                "Real-time 3D pressure Poisson solver rendering zero-gravity acoustic fluid dispersion.",
            )
            .with_param(DspParamSchema::knob("viscosity", "Fluid Viscosity", 0.0001, 1.0, 0.05, "Pa·s", MacroRole::Character, "Fluid kinematic viscosity resistance"))
            .with_param(DspParamSchema::log_knob("sound_speed", "Speed of Sound", 100.0, 2000.0, 343.0, "m/s", MacroRole::Tone, "Acoustic wave propagation velocity in medium"))
            .with_param(DspParamSchema::knob("pressure_damping", "Cavity Damping", 0.9, 0.999, 0.995, "%", MacroRole::Space, "Boundary reflection energy absorption"))
        );

        self.register(
            DspNodeDescriptor::new(
                "MolecularVibrationResonator",
                "Molecular Bond Vibration Resonator",
                DspNodeCategory::AcousticPhysicalModel,
                "Microscopic covalent bond stretching and inter-atomic harmonic vibrational resonance.",
            )
            .with_param(DspParamSchema::knob("bond_stiffness", "Covalent Bond Stiffness", 0.1, 10.0, 2.5, "N/m", MacroRole::Tone, "Inter-atomic spring potential"))
            .with_param(DspParamSchema::knob("atomic_mass", "Atomic Mass Unit", 1.0, 200.0, 12.0, "amu", MacroRole::Punch, "Vibrating nucleus inertia"))
            .with_param(DspParamSchema::knob("thermal_damping", "Thermal Damping", 0.0, 1.0, 0.25, "%", MacroRole::Space, "Brownian phonon thermal loss"))
        );

        self.register(
            DspNodeDescriptor::new(
                "AtmosphericDensityNode",
                "Atmospheric Gas Density Acoustic Filter",
                DspNodeCategory::FilterEq,
                "Altitude, humidity, and temperature dependent acoustic air absorption modeling.",
            )
            .with_param(DspParamSchema::knob("altitude", "Altitude", 0.0, 50.0, 0.0, "km", MacroRole::Tone, "Atmospheric barometric pressure elevation"))
            .with_param(DspParamSchema::knob("humidity", "Relative Humidity", 0.0, 100.0, 50.0, "%", MacroRole::Space, "Water vapor molecular relaxation damping"))
            .with_param(DspParamSchema::knob("temperature", "Gas Temperature", -50.0, 50.0, 20.0, "°C", MacroRole::Character, "Ambient thermal molecular velocity"))
        );

        self.register(
            DspNodeDescriptor::new(
                "QuantumDotTransducerNode",
                "Quantum Dot Piezoelectric Transducer",
                DspNodeCategory::Oscillator,
                "Nanoscale semiconductor quantum dot electron confinement and discrete photon emission audio transducer.",
            )
            .with_param(DspParamSchema::knob("bandgap_ev", "Semiconductor Bandgap", 0.5, 4.0, 1.8, "eV", MacroRole::Tone, "Electronic bandgap transition energy"))
            .with_param(DspParamSchema::knob("photon_flux", "Photon Emission Flux", 0.0, 1.0, 0.70, "%", MacroRole::Punch, "Laser pump excitation intensity"))
        );

        self.register(
            DspNodeDescriptor::new(
                "AcousticLevitationTrapNode",
                "Standing-Wave Acoustic Levitation Trap",
                DspNodeCategory::Modulation,
                "High-intensity ultrasonic standing wave pressure nodes trapping particle trajectories.",
            )
            .with_param(DspParamSchema::knob("trap_freq", "Ultrasonic Carrier", 20.0, 80.0, 40.0, "kHz", MacroRole::Tone, "Transducer array carrier frequency"))
            .with_param(DspParamSchema::knob("radiation_force", "Radiation Force", 0.1, 5.0, 1.2, "N", MacroRole::Punch, "Gor'kov acoustic potential well depth"))
        );

        self.register(
            DspNodeDescriptor::new(
                "SupercriticalFluidNoise",
                "Supercritical Fluid Density Fluctuations",
                DspNodeCategory::Oscillator,
                "Near-critical-point density opalescence generating non-Gaussian stochastic sonic textures.",
            )
            .with_param(DspParamSchema::knob("critical_pressure", "Critical Pressure", 1.0, 50.0, 7.38, "MPa", MacroRole::Punch, "Thermodynamic supercritical state point"))
            .with_param(DspParamSchema::knob("opalescence", "Critical Opalescence", 0.0, 1.0, 0.65, "%", MacroRole::Character, "Clustering correlation length"))
        );

        self.register(
            DspNodeDescriptor::new(
                "MhdPlasmaWaveModulatorNode",
                "Magnetohydrodynamic (MHD) Alfvén Wave Modulator",
                DspNodeCategory::Modulation,
                "Ionized conducting fluid coupling magnetic field lines to acoustic shear wave oscillations.",
            )
            .with_param(DspParamSchema::knob("b_field", "Magnetic Field Flux", 0.1, 20.0, 3.5, "T", MacroRole::Tone, "Confinement field strength"))
            .with_param(DspParamSchema::knob("alfven_speed", "Alfvén Velocity", 100.0, 10000.0, 1500.0, "km/s", MacroRole::Character, "Magnetohydrodynamic wave velocity"))
        );

        self.register(
            DspNodeDescriptor::new(
                "SonoluminescenceSonifier",
                "Sonoluminescence Ultrasonic Cavitation",
                DspNodeCategory::Oscillator,
                "Acoustic bubble collapse generating extreme localized plasma light flashes and hypersonic shock pulses.",
            )
            .with_param(DspParamSchema::knob("bubble_radius", "Bubble Equilibrium Radius", 1.0, 50.0, 4.5, "μm", MacroRole::Tone, "Micron bubble core dimensions"))
            .with_param(DspParamSchema::knob("collapse_intensity", "Cavitation Shock Wave", 0.0, 1.0, 0.80, "%", MacroRole::Punch, "Rayleigh-Plesset collapse intensity"))
        );

        self.register(
            DspNodeDescriptor::new(
                "MetamaterialRefractionFilter",
                "Negative-Index Metamaterial Acoustic Prism",
                DspNodeCategory::FilterEq,
                "Locally resonant acoustic metamaterial exhibiting negative effective mass density and bulk modulus.",
            )
            .with_param(DspParamSchema::knob("refractive_index", "Negative Refraction Index", -5.0, -0.1, -1.4, "n", MacroRole::Tone, "Phase velocity inversion parameter"))
            .with_param(DspParamSchema::knob("subwavelength_q", "Sub-Wavelength Q", 1.0, 50.0, 15.0, "Q", MacroRole::Character, "Phononic bandgap sharpness"))
        );

        self.register(
            DspNodeDescriptor::new(
                "IsmShockwaveReverbNode",
                "Interstellar Medium Shockwave Reverb",
                DspNodeCategory::TimeSpace,
                "Supernova blast wave expanding across magnetized interstellar dust creating vast cosmic reverberation.",
            )
            .with_param(DspParamSchema::knob("mach_number", "Shockwave Mach Number", 1.0, 20.0, 3.5, "M", MacroRole::Punch, "Supersonic shock front speed"))
            .with_param(DspParamSchema::knob("interstellar_decay", "Diffuse Decay Tail", 0.5, 20.0, 8.0, "s", MacroRole::Space, "Interstellar cloud cooling rate"))
        );

        self.register(
            DspNodeDescriptor::new(
                "GravitationalWaveChirp",
                "Binary Black Hole Inspiral Gravitational Chirp",
                DspNodeCategory::Oscillator,
                "General relativity quadrupole formula generating binary merger gravitational wave frequency sweeps.",
            )
            .with_param(DspParamSchema::knob("chirp_mass", "Binary Chirp Mass", 1.0, 100.0, 30.0, "M☉", MacroRole::Tone, "Solar mass equivalent merger system"))
            .with_param(DspParamSchema::knob("coalescence_rate", "Inspiral Sweep Speed", 0.1, 10.0, 1.5, "x", MacroRole::Punch, "Frequency chirp acceleration rate"))
        );

        self.register(
            DspNodeDescriptor::new(
                "CasimirVacuumNoise",
                "Casimir Quantum Vacuum Fluctuation Generator",
                DspNodeCategory::Oscillator,
                "Sub-micron conductive boundary quantum electromagnetic zero-point energy noise transducer.",
            )
            .with_param(DspParamSchema::knob("plate_gap", "Plate Spacing", 5.0, 500.0, 50.0, "nm", MacroRole::Tone, "Nanometer Casimir gap boundary"))
            .with_param(DspParamSchema::knob("zpe_density", "Vacuum Energy Flux", 0.0, 1.0, 0.45, "%", MacroRole::Character, "Zero-point fluctuation coupling"))
        );

        self.register(
            DspNodeDescriptor::new(
                "AcousticCloakingSpatializerNode",
                "Acoustic Cloaking Invisibility Spatializer",
                DspNodeCategory::SpatialSurround,
                "Pentamode material acoustic transformation optics bending wavefronts around objects seamlessly.",
            )
            .with_param(DspParamSchema::knob("cloak_radius", "Cloaking Shell Radius", 0.1, 10.0, 1.5, "m", MacroRole::Space, "Transformation optics shell boundary"))
            .with_param(DspParamSchema::knob("scattering_cancel", "Scattering Cancellation", 0.0, 1.0, 0.90, "%", MacroRole::Tone, "Wavefront phase preservation accuracy"))
        );

        self.register(
            DspNodeDescriptor::new(
                "FusionResonanceSynthNode",
                "Tokamak Fusion Torus Plasma Resonator",
                DspNodeCategory::CompositeSynth,
                "Magnetically confined D-T fusion plasma with helical field safety factor and cyclotron harmonics.",
            )
            .with_param(DspParamSchema::knob("ion_temp", "Ion Temperature", 1.0, 50.0, 15.0, "keV", MacroRole::Tone, "Plasma kinetic energy"))
            .with_param(DspParamSchema::knob("torus_aspect", "Torus Aspect Ratio", 1.5, 6.0, 3.1, "R/a", MacroRole::Character, "Major to minor radius geometry"))
        );

        // ==========================================
        // 18. Neuro-Synthesis & BCI Brainwave Audio (Tier 43)
        // ==========================================
        self.register(
            DspNodeDescriptor::new(
                "BciEegDecoderNode",
                "Neural BCI EEG Brainwave Decoder",
                DspNodeCategory::NeuralAi,
                "Multi-channel non-invasive EEG brainwave interface separating Delta, Theta, Alpha, Beta, and Gamma bands.",
            )
            .with_param(DspParamSchema::knob("delta_band", "Delta (0.5-4Hz Sleep)", 0.0, 1.0, 0.20, "%", MacroRole::Space, "Deep sleep / sub-bass band"))
            .with_param(DspParamSchema::knob("theta_band", "Theta (4-8Hz Flow)", 0.0, 1.0, 0.40, "%", MacroRole::Character, "Meditation & trance band"))
            .with_param(DspParamSchema::knob("alpha_band", "Alpha (8-13Hz Calm)", 0.0, 1.0, 0.70, "%", MacroRole::Tone, "Relaxed alertness band"))
            .with_param(DspParamSchema::knob("beta_band", "Beta (13-30Hz Focus)", 0.0, 1.0, 0.50, "%", MacroRole::Punch, "Active focus & cognition band"))
            .with_param(DspParamSchema::knob("gamma_band", "Gamma (30-100Hz Insight)", 0.0, 1.0, 0.30, "%", MacroRole::Tone, "High-frequency cognitive binding"))
        );

        self.register(
            DspNodeDescriptor::new(
                "NeuroAffectiveAnalyzer",
                "Neuro-Affective Valence/Arousal State Synth",
                DspNodeCategory::NeuralAi,
                "Russell circumplex model mapping emotional valence and physiological arousal into synthesis parameters.",
            )
            .with_param(DspParamSchema::knob("valence", "Emotional Valence", -1.0, 1.0, 0.4, "", MacroRole::Tone, "Negative (dark) to Positive (bright) mood"))
            .with_param(DspParamSchema::knob("arousal", "Physiological Arousal", 0.0, 1.0, 0.6, "", MacroRole::Punch, "Calm to High-energy excitement"))
        );

        self.register(
            DspNodeDescriptor::new(
                "AuditoryCortexIrSynthesizer",
                "Auditory Cortex Neural Impulse Response",
                DspNodeCategory::TimeSpace,
                "Biophysical neural delay model simulating synaptic latency and reverberation in primary auditory cortex (A1).",
            )
            .with_param(DspParamSchema::knob("cortex_depth", "Cortex Layer (1-6)", 1.0, 6.0, 4.0, "L", MacroRole::Character, "Cortical laminar layer depth"))
            .with_param(DspParamSchema::knob("synaptic_delay", "Synaptic Latency", 1.0, 50.0, 12.0, "ms", MacroRole::Space, "Neural inter-spike latency dispersion"))
        );

        self.register(
            DspNodeDescriptor::new(
                "HolographicSpatializer",
                "Neuro-Acoustic Holographic 3D Soundfield",
                DspNodeCategory::SpatialSurround,
                "Diffractive acoustic hologram reconstructing virtual 3D point sources directly at the eardrum.",
            )
            .with_param(DspParamSchema::knob("hologram_order", "Hologram Order", 1.0, 8.0, 4.0, "", MacroRole::Character, "Spatial reconstruction resolution"))
            .with_param(DspParamSchema::knob("soundfield_depth", "Sweetspot Depth", 0.5, 10.0, 2.0, "m", MacroRole::Space, "Virtual source distance focal plane"))
        );

        self.register(
            DspNodeDescriptor::new(
                "BrainstemPitchTracker",
                "Subcortical Brainstem Auditory Frequency Tracker",
                DspNodeCategory::Modulation,
                "Frequency Following Response (FFR) pitch detector tracking microtonal inflections with sub-millisecond precision.",
            )
            .with_param(DspParamSchema::knob("tracking_speed", "Brainstem Speed", 1.0, 50.0, 8.0, "ms", MacroRole::Tone, "Response window latency"))
            .with_param(DspParamSchema::knob("harmonic_fidelity", "Harmonic Lock", 0.0, 1.0, 0.95, "%", MacroRole::Character, "Fundamental frequency phase lock"))
        );

        self.register(
            DspNodeDescriptor::new(
                "HrvTempoSyncEngine",
                "Heart Rate Variability (HRV) Tempo Engine",
                DspNodeCategory::Modulation,
                "Electrocardiogram R-peak detector dynamically syncing musical BPM and swing to cardiovascular biorhythms.",
            )
            .with_param(DspParamSchema::knob("target_bpm", "Cardiac Tempo", 40.0, 200.0, 120.0, "BPM", MacroRole::Punch, "Base physiological heart tempo"))
            .with_param(DspParamSchema::knob("respiratory_mod", "Respiratory Sinus Sync", 0.0, 1.0, 0.35, "%", MacroRole::Character, "Inhale/exhale tempo acceleration"))
        );

        self.register(
            DspNodeDescriptor::new(
                "NeuroFeedbackOscillator",
                "Closed-Loop Neuro-Feedback Relaxation Oscillator",
                DspNodeCategory::Oscillator,
                "Generates calming harmonic overtones reinforcing relaxed alpha brainwave states in real-time.",
            )
            .with_param(DspParamSchema::knob("feedback_gain", "Closed-Loop Gain", 0.0, 2.0, 0.85, "x", MacroRole::Character, "Biofeedback resonance strength"))
            .with_param(DspParamSchema::knob("relaxation_target", "Alpha Center Freq", 8.0, 12.0, 10.0, "Hz", MacroRole::Tone, "Target brainwave entrainment frequency"))
        );

        self.register(
            DspNodeDescriptor::new(
                "AcousticHologramFilter",
                "Spatial Acoustic Hologram Reconstruction Filter",
                DspNodeCategory::FilterEq,
                "Fresnel-Kirchhoff diffraction integral acoustic lens focusing soundbeams tightly in physical space.",
            )
            .with_param(DspParamSchema::knob("diffraction_focus", "Acoustic Focus", 0.1, 10.0, 1.8, "m", MacroRole::Space, "Diffraction lens focal distance"))
            .with_param(DspParamSchema::knob("phase_coherence", "Phase Coherence", 0.0, 1.0, 0.85, "%", MacroRole::Tone, "Wavefront phase alignment"))
        );

        self.register(
            DspNodeDescriptor::new(
                "BinauralBeatGenerator",
                "Brainwave Entrainment Binaural Beat Generator",
                DspNodeCategory::Oscillator,
                "Dual carrier frequency oscillator inducing hemispheric brainwave synchrony (Schumann 7.83Hz, Theta, Alpha).",
            )
            .with_param(DspParamSchema::log_knob("carrier_hz", "Carrier Center Pitch", 100.0, 1000.0, 216.0, "Hz", MacroRole::Tone, "Audible base tone frequency"))
            .with_param(DspParamSchema::knob("entrainment_beat", "Binaural Beat Delta", 0.5, 40.0, 7.83, "Hz", MacroRole::Character, "Interaural frequency offset (brainwave target)"))
        );

        self.register(
            DspNodeDescriptor::new(
                "HapticTransducerNode",
                "Subsensory Tactile Bone-Conduction Transducer",
                DspNodeCategory::Utility,
                "Direct somatic low-frequency tactile transducer vibrating studio chairs and haptic wearables.",
            )
            .with_param(DspParamSchema::knob("tactile_drive", "Tactile Overdrive", 0.0, 10.0, 3.0, "x", MacroRole::Punch, "Haptic motor excitation power"))
            .with_param(DspParamSchema::knob("body_resonance", "Body Resonance Peak", 20.0, 150.0, 45.0, "Hz", MacroRole::Punch, "Somatic chest cavity resonant frequency"))
        );

        // ==========================================
        // 19. Quantum Audio & Hyper-Dimensional DSP (Tier 41)
        // ==========================================
        self.register(
            DspNodeDescriptor::new(
                "HyperDimensionalTensorSpatializer",
                "Hyper-Dimensional Tensor 11D Manifold Spatializer",
                DspNodeCategory::SpatialSurround,
                "M-theory 11-dimensional Calabi-Yau manifold audio projection spatializing signals beyond 3D Euclidean space.",
            )
            .with_param(DspParamSchema::knob("manifold_dim", "Manifold Dimensions", 4.0, 11.0, 6.0, "D", MacroRole::Space, "Hyper-dimensional embedding space"))
            .with_param(DspParamSchema::knob("calabi_yau_radius", "Calabi-Yau Metric", 0.01, 2.0, 0.45, "R", MacroRole::Character, "Extra-dimensional compactification scale"))
        );

        self.register(
            DspNodeDescriptor::new(
                "QuantumEntanglementRouter",
                "Quantum Entanglement Signal Routing Matrix",
                DspNodeCategory::Utility,
                "Non-local Einstein-Podolsky-Rosen (EPR) entangled channel routing with zero phase latency teleportation.",
            )
            .with_param(DspParamSchema::knob("entanglement_fidelity", "Bell-State Fidelity", 0.0, 1.0, 0.98, "%", MacroRole::Character, "Quantum correlation purity"))
            .with_param(DspParamSchema::knob("teleport_delay", "Teleportation Latency", 0.0, 10.0, 0.0, "ms", MacroRole::Space, "Sub-luminal quantum channel delay"))
        );

        self.register(
            DspNodeDescriptor::new(
                "SubHarmonicQuantumTunnelingFilter",
                "Sub-Harmonic Quantum Tunneling Resonant Filter",
                DspNodeCategory::FilterEq,
                "Quantum wavepacket tunneling through finite potential barriers creating sub-harmonic resonance peaks.",
            )
            .with_param(DspParamSchema::knob("barrier_potential", "Barrier Potential", 0.1, 5.0, 1.5, "eV", MacroRole::Tone, "Quantum wall transmission height"))
            .with_param(DspParamSchema::knob("tunneling_q", "Tunneling Resonance Q", 1.0, 30.0, 8.0, "Q", MacroRole::Character, "Sub-harmonic peak resonance sharpness"))
        );

        self.register(
            DspNodeDescriptor::new(
                "RelativisticDopplerShiftNode",
                "Relativistic Lorentz Doppler Warp Node",
                DspNodeCategory::TimeSpace,
                "Special relativity relativistic Doppler effect and Lorentz time dilation on high-speed orbiting audio sources.",
            )
            .with_param(DspParamSchema::knob("subluminal_velocity", "Relativistic Velocity", 0.0, 0.99, 0.45, "c", MacroRole::Tone, "Speed of source as fraction of lightspeed"))
            .with_param(DspParamSchema::knob("lorentz_dilation", "Time Dilation Depth", 0.0, 1.0, 0.60, "%", MacroRole::Space, "Gravitational/kinematic pitch dilation"))
        );

        self.register(
            DspNodeDescriptor::new(
                "StochasticQuantumDecoherenceNoise",
                "Stochastic Quantum Decoherence Thermal Noise",
                DspNodeCategory::Oscillator,
                "Lindblad master equation density matrix decoherence modeling environmental quantum state collapse noise.",
            )
            .with_param(DspParamSchema::knob("vacuum_temp_k", "Thermal Bath Temp", 0.001, 300.0, 4.2, "K", MacroRole::Character, "Decoherence thermal reservoir temperature"))
            .with_param(DspParamSchema::knob("decoherence_rate", "Decoherence Rate", 0.0, 1.0, 0.20, "%", MacroRole::Space, "Quantum-to-classical transition speed"))
        );

        self.register(
            DspNodeDescriptor::new(
                "HyperbolicReverbNode",
                "Poincaré Disk Non-Euclidean Hyperbolic Reverb",
                DspNodeCategory::TimeSpace,
                "Negative curvature hyperbolic Riemannian geometry ray tracer generating exponentially expanding reflection density.",
            )
            .with_param(DspParamSchema::knob("poincare_curvature", "Hyperbolic Curvature", -5.0, -0.1, -1.2, "K", MacroRole::Space, "Gaussian negative curvature"))
            .with_param(DspParamSchema::knob("geodesic_diffusion", "Geodesic Diffusion", 0.0, 1.0, 0.80, "%", MacroRole::Space, "Non-Euclidean wave dispersion"))
        );

        self.register(
            DspNodeDescriptor::new(
                "QuantumHarmonicOscillatorVoice",
                "Hermite Polynomial Quantum Harmonic Voice",
                DspNodeCategory::Oscillator,
                "Hermite-Gaussian stationary quantum wavefunctions in parabolic harmonic potential well.",
            )
            .with_param(DspParamSchema::knob("energy_quantum_n", "Quantum Number (n)", 0.0, 16.0, 3.0, "n", MacroRole::Tone, "Discrete energy level eigenstate"))
            .with_param(DspParamSchema::knob("hermite_dispersion", "Wavepacket Width", 0.1, 4.0, 1.0, "σ", MacroRole::Character, "Spatial probability spread"))
        );

        self.register(
            DspNodeDescriptor::new(
                "ChaoticFractalAttractorModulator",
                "Lorenz / Rössler Strange Attractor Modulator",
                DspNodeCategory::Modulation,
                "Nonlinear deterministic chaos differential equations generating infinite, non-repeating modulation curves.",
            )
            .with_param(DspParamSchema::choice("attractor_type", "Attractor Topology", &["Lorenz (Butterfly)", "Rössler (Spiral)", "Clifford (Fractal)", "Chua (Double Scroll)"], 0, "Chaotic dynamical system algorithm"))
            .with_param(DspParamSchema::knob("chaos_speed", "Attractor Speed", 0.1, 20.0, 2.5, "Hz", MacroRole::Character, "Phase trajectory orbit speed"))
            .with_param(DspParamSchema::knob("lyapunov_exponent", "Chaos Exponent", 0.1, 5.0, 1.4, "λ", MacroRole::Punch, "Trajectory divergence sensitivity"))
        );

        // ==========================================
        // 20. Deep Physical Modeling & Composite Synthesizers
        // ==========================================
        self.register(
            DspNodeDescriptor::new(
                "PercussionMembraneNode",
                "2D Circular Membrane Kettle Drum & Snare",
                DspNodeCategory::AcousticPhysicalModel,
                "2D Bessel modal circular drum membrane with kettle cavity air back-pressure and snare wire coupling.",
            )
            .with_param(DspParamSchema::choice("profile", "Drum Profile", &["Timpani Kettle", "36\" Concert Bass", "Snare Drum", "Acoustic Tom-Tom", "Bongos & Congas", "African Djembe"], 0, "Membrane acoustic geometry preset"))
            .with_param(DspParamSchema::knob("tension", "Tension Drop", 0.0, 1.0, 0.30, "%", MacroRole::Punch, "Strike tension dynamic pitch bend"))
            .with_param(DspParamSchema::knob("kettle_cavity", "Kettle Air Spring", 0.0, 1.0, 0.70, "%", MacroRole::Tone, "Enclosure back-pressure loading"))
            .with_param(DspParamSchema::knob("strike_radius", "Strike Radial Pos", 0.0, 1.0, 0.40, "r", MacroRole::Tone, "0=Center (deep), 1=Rim (bright slap)"))
        );

        self.register(
            DspNodeDescriptor::new(
                "QuartzCrystalResonator",
                "Tibetan Quartz Crystal Singing Bowl",
                DspNodeCategory::AcousticPhysicalModel,
                "99.9% pure quartz crystal thin-shell modal resonance with Stribeck stick-slip wand friction and water mass tuning.",
            )
            .with_param(DspParamSchema::choice("material", "Crystal Grade", &["Pure Quartz (432Hz)", "Franklin Glass Chalice", "Wet Wine Goblet", "Pyrex Glass Bell", "Metallophone Alloy"], 0, "Resonator material composition"))
            .with_param(DspParamSchema::knob("water_level", "Water Filling Pitch", 0.0, 1.0, 0.0, "%", MacroRole::Tone, "Hydro-acoustic mass pitch lowering"))
            .with_param(DspParamSchema::knob("wand_friction", "Suede Wand Friction", 0.0, 1.0, 0.75, "%", MacroRole::Character, "Stick-slip rotational excitation"))
            .with_param(DspParamSchema::knob("mode_split", "Doublet Beating", 0.0, 5.0, 0.85, "Hz", MacroRole::Space, "Degenerate mode shimmer beating"))
        );

        self.register(
            DspNodeDescriptor::new(
                "IdiophoneResonatorNode",
                "Orchestral Marimba & Vibraphone Modal Resonator",
                DspNodeCategory::AcousticPhysicalModel,
                "Tuned rosewood/aluminum idiophone bars with Hertzian mallet contact and motorized tremolo discs.",
            )
            .with_param(DspParamSchema::choice("profile", "Bar Profile", &["Rosewood Marimba", "Rosewood Xylophone", "Vibraphone (Motor Tremolo)", "Trinidad Steelpan", "African Kalimba", "Glockenspiel Bell"], 0, "Acoustic bar & resonator material"))
            .with_param(DspParamSchema::knob("mallet_hardness", "Mallet Tip Hardness", 0.0, 1.0, 0.50, "%", MacroRole::Punch, "Felt/rubber/wood striker stiffness"))
            .with_param(DspParamSchema::knob("tremolo_rpm", "Vibraphone Rotor RPM", 0.0, 600.0, 240.0, "RPM", MacroRole::Character, "Motorized acoustic disc tremolo rate"))
        );

        self.register(
            DspNodeDescriptor::new(
                "EffectChorus",
                "Stereo LFO Delay Chorus",
                DspNodeCategory::TimeSpace,
                "Multi-voice modulated delay line producing rich stereo widening and shimmering ensemble chorus.",
            )
            .with_param(DspParamSchema::knob("rate", "Chorus LFO Rate", 0.1, 10.0, 1.5, "Hz", MacroRole::Tone, "Sweep modulation speed"))
            .with_param(DspParamSchema::knob("depth", "Delay Excursion Depth", 0.0, 1.0, 0.50, "%", MacroRole::Space, "Modulation sweep excursion width"))
            .with_param(DspParamSchema::knob("mix", "Wet Blend", 0.0, 1.0, 0.50, "%", MacroRole::Space, "Wet chorus signal level"))
        );

        self.register(
            DspNodeDescriptor::new(
                "EffectFlanger",
                "Short Delay Resonant Flanger",
                DspNodeCategory::TimeSpace,
                "Comb filtering flanger with inverted feedback phase and high-resonance jet sweep.",
            )
            .with_param(DspParamSchema::knob("rate", "Flanger Sweep Speed", 0.05, 5.0, 0.5, "Hz", MacroRole::Tone, "LFO sweep speed"))
            .with_param(DspParamSchema::knob("feedback", "Regeneration Feedback", -0.95, 0.95, 0.50, "%", MacroRole::Character, "Comb feedback intensity"))
            .with_param(DspParamSchema::knob("depth", "Excursion Depth", 0.0, 1.0, 0.70, "%", MacroRole::Space, "Modulation sweep amplitude"))
        );

        self.register(
            DspNodeDescriptor::new(
                "EffectPhaser",
                "6-Stage Cascading All-Pass Phaser",
                DspNodeCategory::TimeSpace,
                "Cascaded 1st-order all-pass filters creating swept frequency notches with deep analog feedback.",
            )
            .with_param(DspParamSchema::choice("stages", "Phaser Stages", &["2-Stage Gentle", "4-Stage Vintage", "6-Stage Deep"], 1, "Number of all-pass phase shift stages"))
            .with_param(DspParamSchema::knob("rate", "Phaser LFO Rate", 0.05, 10.0, 0.5, "Hz", MacroRole::Tone, "Phase sweep modulation frequency"))
            .with_param(DspParamSchema::knob("feedback", "Resonant Feedback", 0.0, 0.95, 0.40, "%", MacroRole::Character, "Notch resonance intensity"))
        );

        self.register(
            DspNodeDescriptor::new(
                "PluckSynth",
                "Karplus-Strong Plucked String Voice",
                DspNodeCategory::CompositeSynth,
                "Complete Karplus-Strong physical modeling synth voice with low-pass feedback loop.",
            )
            .with_param(DspParamSchema::log_knob("freq", "String Frequency", 20.0, 2000.0, 220.0, "Hz", MacroRole::Tone, "String fundamental pitch"))
            .with_param(DspParamSchema::knob("damping", "String Damping", 0.0, 1.0, 0.20, "%", MacroRole::Space, "High-frequency string loss factor"))
            .with_param(DspParamSchema::knob("hardness", "Pluck Sharpness", 0.0, 1.0, 0.80, "%", MacroRole::Punch, "Impulse excitation transient"))
        );

        self.register(
            DspNodeDescriptor::new(
                "GlitchAetherMachine",
                "Glitch Aether Industrial Subtractive Synth",
                DspNodeCategory::CompositeSynth,
                "Dual saw/pulse core driving multi-mode tube distortion, Moog ladder filter, GlitchGate chopper, and TapeStop.",
            )
            .with_param(DspParamSchema::log_knob("filter_cutoff", "Ladder Cutoff", 40.0, 15000.0, 2500.0, "Hz", MacroRole::Tone, "Ladder low-pass frequency"))
            .with_param(DspParamSchema::knob("chopper_rate", "GlitchGate Rate", 1.0, 32.0, 8.0, "Hz", MacroRole::Punch, "Rhythmic gate chopping rate"))
            .with_param(DspParamSchema::knob("distortion_drive", "Tube Fuzz Drive", 1.0, 20.0, 4.0, "x", MacroRole::Character, "Overdrive saturation amount"))
        );

        self.register(
            DspNodeDescriptor::new(
                "CyberpunkSubSynth",
                "Cyberpunk Dual Sub-Bass Wavefolder Synth",
                DspNodeCategory::CompositeSynth,
                "Sine sub-oscillator and saw driver passing through 4-pole SVF filter, Buchla wavefolder, and pitch wobble.",
            )
            .with_param(DspParamSchema::log_knob("sub_freq", "Sub-Bass Fundamental", 20.0, 200.0, 45.0, "Hz", MacroRole::Punch, "Bass fundamental pitch"))
            .with_param(DspParamSchema::knob("wavefold_drive", "Buchla Wavefolder Drive", 1.0, 10.0, 4.0, "x", MacroRole::Character, "Wavefolder folding depth"))
            .with_param(DspParamSchema::knob("wobble_rate", "Pitch LFO Wobble", 0.1, 16.0, 4.0, "Hz", MacroRole::Tone, "Bass pitch wobble speed"))
        );

        self.register(
            DspNodeDescriptor::new(
                "AtmosphericPadSynth",
                "Atmospheric Cinematic Ethereal Pad Synth",
                DspNodeCategory::CompositeSynth,
                "Detuned saw/triangle dual oscillators driving SVF filter, LFO filter modulation, tape delay, and vast reverb space.",
            )
            .with_param(DspParamSchema::knob("detune", "Stereo Detune", 0.0, 25.0, 3.0, "cents", MacroRole::Space, "Oscillator pitch detune"))
            .with_param(DspParamSchema::knob("reverb_decay", "Reverb Space Decay", 0.5, 15.0, 6.0, "s", MacroRole::Space, "Late reverb decay time"))
            .with_param(DspParamSchema::knob("filter_sweep", "LFO Filter Sweep", 0.05, 5.0, 0.5, "Hz", MacroRole::Tone, "Filter modulation cycle rate"))
        );

        self.register(
            DspNodeDescriptor::new(
                "GlitchPercussionSynth",
                "Glitch Stutter Percussion Drum Synth",
                DspNodeCategory::CompositeSynth,
                "Noise burst and sine pitch drop processed through bitcrusher, feedback comb filter, and real-time buffer stutter.",
            )
            .with_param(DspParamSchema::log_knob("pitch", "Pitch Drop Fundamental", 30.0, 300.0, 65.0, "Hz", MacroRole::Punch, "Transient bass drop tuning"))
            .with_param(DspParamSchema::knob("bitcrush_depth", "Bitcrusher Depth", 2.0, 16.0, 6.0, "bits", MacroRole::Character, "Digital bit reduction"))
            .with_param(DspParamSchema::knob("stutter_window", "Buffer Stutter Window", 16.0, 512.0, 256.0, "smpl", MacroRole::Punch, "Stutter grain repetition length"))
        );

        // ==========================================
        // 21. Specialized Physical Modeling Acoustic Components
        // ==========================================
        self.register(
            DspNodeDescriptor::new(
                "BambooResonator",
                "Bamboo Tube Resonator Cavity",
                DspNodeCategory::AcousticPhysicalModel,
                "Tuned hollow bamboo tube acoustic resonator with length, air aperture, and internal damping.",
            )
            .with_param(DspParamSchema::knob("tube_length", "Tube Length", 0.1, 2.0, 0.45, "m", MacroRole::Tone, "Acoustic column length"))
            .with_param(DspParamSchema::knob("damping", "Cavity Damping", 0.01, 0.5, 0.08, "%", MacroRole::Space, "Internal air friction and bamboo wall absorption"))
            .with_param(DspParamSchema::knob("aperture", "Aperture Radius", 0.01, 0.1, 0.035, "m", MacroRole::Character, "Open top tube radius"))
        );

        self.register(
            DspNodeDescriptor::new(
                "JawariBridge",
                "Jawari Curved Bone Bridge",
                DspNodeCategory::AcousticPhysicalModel,
                "Non-linear unilateral contact curved bridge generating buzzing sitar and tanpura overtone cascades.",
            )
            .with_param(DspParamSchema::knob("curvature", "Bridge Arc Curvature", 0.01, 1.0, 0.40, "", MacroRole::Character, "Parabolic contact slope"))
            .with_param(DspParamSchema::knob("string_tension", "String Contact Force", 0.1, 10.0, 2.0, "N", MacroRole::Punch, "Downward string pressure against bridge"))
            .with_param(DspParamSchema::knob("damping", "Contact Damping", 0.0, 1.0, 0.15, "%", MacroRole::Tone, "Material contact loss"))
        );

        self.register(
            DspNodeDescriptor::new(
                "JiBridge",
                "Koto Movable Ji Bridge",
                DspNodeCategory::AcousticPhysicalModel,
                "Movable ivory Ji bridge coupling with vertical acoustic force and Oshi-Ite lateral string pull.",
            )
            .with_param(DspParamSchema::knob("bridge_pos", "Ji Position on Soundboard", 0.1, 0.9, 0.5, "pos", MacroRole::Tone, "Location along paulownia soundboard"))
            .with_param(DspParamSchema::knob("oshi_ite", "Oshi-Ite String Pull", 0.0, 4.0, 0.0, "st", MacroRole::Character, "Left-hand microtonal tension bend"))
            .with_param(DspParamSchema::knob("vertical_load", "Vertical Force Coupling", 0.1, 1.0, 0.75, "%", MacroRole::Punch, "Acoustic bridge energy transfer"))
        );

        self.register(
            DspNodeDescriptor::new(
                "TrompetteBridge",
                "Hurdy Gurdy Trompette Chien",
                DspNodeCategory::AcousticPhysicalModel,
                "Loose buzzing dog bridge dynamic vibration, string pressure, and rhythmic crank pulsing.",
            )
            .with_param(DspParamSchema::knob("tirant_tension", "Tirant String Peg", 0.0, 1.0, 0.65, "%", MacroRole::Tone, "Tirant string peg sensitivity tuning"))
            .with_param(DspParamSchema::knob("buzz_intensity", "Chien Buzz Intensity", 0.0, 1.0, 0.80, "%", MacroRole::Punch, "Loose wooden foot slap response"))
        );

        self.register(
            DspNodeDescriptor::new(
                "HammerStrike",
                "Acoustic Hammer Strike Exciter",
                DspNodeCategory::AcousticPhysicalModel,
                "Hertzian contact stiffness and non-linear felt/leather compression excitation.",
            )
            .with_param(DspParamSchema::knob("strike_velocity", "Strike Velocity", 0.01, 10.0, 1.5, "m/s", MacroRole::Punch, "Mallet or hammer downward speed"))
            .with_param(DspParamSchema::knob("felt_stiffness", "Felt Stiffness Exponent", 1.0, 4.0, 2.5, "p", MacroRole::Character, "Non-linear felt compression curve"))
            .with_param(DspParamSchema::knob("contact_duration", "Contact Duration", 0.5, 10.0, 2.0, "ms", MacroRole::Tone, "Impulse contact window"))
        );

        self.register(
            DspNodeDescriptor::new(
                "HornReflection",
                "Webster Horn Acoustic Flare",
                DspNodeCategory::AcousticPhysicalModel,
                "Webster horn flare acoustic impedance matching and bell high-frequency radiation filter.",
            )
            .with_param(DspParamSchema::knob("flare_rate", "Flare Exponent Rate", 0.1, 5.0, 1.2, "x", MacroRole::Tone, "Exponential bell curvature expansion rate"))
            .with_param(DspParamSchema::log_knob("cutoff_freq", "Horn Cutoff Frequency", 50.0, 2000.0, 300.0, "Hz", MacroRole::Tone, "Acoustic high-pass horn radiation cutoff"))
            .with_param(DspParamSchema::knob("radiation_loss", "Radiation Loss", 0.0, 1.0, 0.35, "%", MacroRole::Space, "Open bell acoustic radiation efficiency"))
        );

        self.register(
            DspNodeDescriptor::new(
                "ToneholeGrid",
                "Woodwind Tonehole Lattice",
                DspNodeCategory::AcousticPhysicalModel,
                "Open and closed tonehole acoustic transmission network with variable finger pad coverage.",
            )
            .with_param(DspParamSchema::knob("tonehole_1", "Tonehole 1 Open", 0.0, 1.0, 1.0, "%", MacroRole::Tone, "First tonehole finger opening"))
            .with_param(DspParamSchema::knob("tonehole_2", "Tonehole 2 Open", 0.0, 1.0, 1.0, "%", MacroRole::Tone, "Second tonehole finger opening"))
            .with_param(DspParamSchema::knob("tonehole_3", "Tonehole 3 Open", 0.0, 1.0, 0.0, "%", MacroRole::Tone, "Third tonehole finger opening"))
            .with_param(DspParamSchema::knob("chimney_height", "Chimney Height", 1.0, 10.0, 3.5, "mm", MacroRole::Character, "Tonehole wall thickness chimney inductance"))
        );

        self.register(
            DspNodeDescriptor::new(
                "Windchest",
                "Pipe Organ Aerodynamic Windchest",
                DspNodeCategory::AcousticPhysicalModel,
                "Pipe organ windchest pressure reservoir, pallet valve dynamics, and air turbulence.",
            )
            .with_param(DspParamSchema::knob("bellows_pressure", "Reservoir Pressure", 200.0, 2500.0, 850.0, "Pa", MacroRole::Punch, "Static bellows windchest reservoir pressure"))
            .with_param(DspParamSchema::knob("pallet_open", "Pallet Valve Aperture", 0.0, 1.0, 1.0, "%", MacroRole::Punch, "Key-actuated pallet valve opening"))
            .with_param(DspParamSchema::knob("turbulence", "Air Jet Turbulence", 0.0, 1.0, 0.20, "%", MacroRole::Character, "Windway vortex air turbulence"))
        );

        self.register(
            DspNodeDescriptor::new(
                "WoodwindJet",
                "Air-Reed Vortex Jet Instability",
                DspNodeCategory::AcousticPhysicalModel,
                "Air-reed vortex shedding, jet displacement, and non-linear velocity amplification.",
            )
            .with_param(DspParamSchema::knob("jet_length", "Jet Distance to Edge", 1.0, 20.0, 6.0, "mm", MacroRole::Tone, "Flue exit to splitting edge distance"))
            .with_param(DspParamSchema::knob("jet_gain", "Vortex Non-Linear Gain", 0.1, 5.0, 1.8, "x", MacroRole::Character, "Instability amplification factor"))
        );

        self.register(
            DspNodeDescriptor::new(
                "GlottalPulse",
                "Liljencrants-Fant Glottal Source",
                DspNodeCategory::AcousticPhysicalModel,
                "Liljencrants-Fant (LF) model glottal flow waveform with open quotient, return phase, and pitch.",
            )
            .with_param(DspParamSchema::log_knob("pitch_hz", "Fundamental Pitch", 40.0, 1000.0, 120.0, "Hz", MacroRole::Tone, "Vocal cord vibration fundamental frequency"))
            .with_param(DspParamSchema::knob("open_quotient", "Open Quotient (OQ)", 0.2, 0.9, 0.65, "%", MacroRole::Character, "Glottal open phase duration fraction"))
            .with_param(DspParamSchema::knob("return_phase", "Return Phase (Ta)", 0.01, 0.2, 0.04, "%", MacroRole::Punch, "Glottal abrupt closure speed"))
        );

        self.register(
            DspNodeDescriptor::new(
                "AirReed",
                "Acoustic Air-Reed Flute Excitation",
                DspNodeCategory::AcousticPhysicalModel,
                "Acoustic air-reed split with blowing pressure, jet velocity, and utaguchi splitting edge.",
            )
            .with_param(DspParamSchema::knob("blowing_pressure", "Blowing Pressure", 50.0, 3000.0, 700.0, "Pa", MacroRole::Punch, "Mouth blowing pressure"))
            .with_param(DspParamSchema::knob("splitting_angle", "Splitting Edge Angle", -45.0, 45.0, 0.0, "°", MacroRole::Tone, "Utaguchi embouchure split angle"))
        );

        self.register(
            DspNodeDescriptor::new(
                "CassottoChamber",
                "Accordion Cassotto Tone Chamber",
                DspNodeCategory::AcousticPhysicalModel,
                "Solid wood internal tone chamber for free-reed instruments absorbing high harmonics warmly.",
            )
            .with_param(DspParamSchema::knob("chamber_volume", "Chamber Volume", 0.2, 2.0, 1.0, "x", MacroRole::Tone, "Cassotto wooden cavity scale"))
            .with_param(DspParamSchema::knob("wood_absorption", "Wood Absorption", 0.1, 0.9, 0.65, "%", MacroRole::Space, "Softwood high frequency damping"))
        );

        self.register(
            DspNodeDescriptor::new(
                "WaveguideMesh",
                "2D Rectilinear Waveguide Mesh",
                DspNodeCategory::AcousticPhysicalModel,
                "2D finite-difference waveguide mesh solving acoustic wave equation on plates and resonant membranes.",
            )
            .with_param(DspParamSchema::knob("mesh_tension", "Mesh Propagation Speed", 0.1, 1.0, 0.65, "c", MacroRole::Tone, "Wavefront velocity through mesh nodes"))
            .with_param(DspParamSchema::knob("boundary_loss", "Boundary Absorption", 0.001, 0.2, 0.02, "%", MacroRole::Space, "Perimeter reflection loss factor"))
            .with_param(DspParamSchema::knob("strike_x", "Excitation Pos X", 0.05, 0.95, 0.50, "", MacroRole::Tone, "Horizontal impact coordinate"))
            .with_param(DspParamSchema::knob("strike_y", "Excitation Pos Y", 0.05, 0.95, 0.50, "", MacroRole::Punch, "Vertical impact coordinate"))
        );

        self.register(
            DspNodeDescriptor::new(
                "ZeroGravityFluid",
                "Zero-Gravity Fluid Acoustic Droplet",
                DspNodeCategory::AcousticPhysicalModel,
                "Microgravity liquid droplet surface tension vibration and Rayleigh modal oscillation.",
            )
            .with_param(DspParamSchema::knob("droplet_radius", "Droplet Radius", 1.0, 50.0, 12.0, "mm", MacroRole::Tone, "Liquid sphere equilibrium size"))
            .with_param(DspParamSchema::knob("surface_tension", "Surface Tension", 10.0, 100.0, 72.8, "mN/m", MacroRole::Character, "Liquid capillary restoring force"))
            .with_param(DspParamSchema::knob("fluid_viscosity", "Kinematic Viscosity", 0.1, 10.0, 1.0, "cSt", MacroRole::Space, "Internal viscous oscillation damping"))
        );

        // ==========================================
        // 22. Filters, Modulators & Mastering Utilities
        // ==========================================
        self.register(
            DspNodeDescriptor::new(
                "FilterBiquad",
                "Direct Form II Biquad Filter",
                DspNodeCategory::FilterEq,
                "Universal biquad filter with Peaking EQ, Low Shelf, High Shelf, Notch, Bandpass, and Allpass topologies.",
            )
            .with_param(DspParamSchema::log_knob("freq", "Center Frequency", 20.0, 20000.0, 1000.0, "Hz", MacroRole::Tone, "Filter center or corner frequency"))
            .with_param(DspParamSchema::knob("gain_db", "Filter Gain", -24.0, 24.0, 0.0, "dB", MacroRole::Punch, "Boost or cut gain"))
            .with_param(DspParamSchema::knob("q", "Filter Q Factor", 0.1, 20.0, 1.0, "Q", MacroRole::Character, "Filter resonance quality factor"))
            .with_param(DspParamSchema::choice("type", "Biquad Type", &["Peaking", "LowShelf", "HighShelf", "LowPass", "HighPass", "BandPass", "Notch", "AllPass"], 0, "Filter response shape"))
        );

        self.register(
            DspNodeDescriptor::new(
                "DcBlockFilter",
                "DC Offset Removal Filter",
                DspNodeCategory::FilterEq,
                "Ultra low-frequency 5Hz high-pass filter stripping DC electrical offset cleanly.",
            )
            .with_param(DspParamSchema::log_knob("pole_radius", "Pole Radius", 0.95, 0.9999, 0.995, "", MacroRole::Tone, "DC filter pole proximity to unit circle"))
        );

        self.register(
            DspNodeDescriptor::new(
                "RingModulator",
                "Carrier Multiplier Ring Modulator",
                DspNodeCategory::DistortionSaturation,
                "Multi-waveform carrier multiplier producing metallic, bell-like, and robotic sideband harmonics.",
            )
            .with_param(DspParamSchema::log_knob("carrier_freq", "Carrier Frequency", 10.0, 8000.0, 440.0, "Hz", MacroRole::Tone, "Oscillator carrier frequency"))
            .with_param(DspParamSchema::choice("waveform", "Carrier Waveform", &["Sine", "Triangle", "Square"], 0, "Carrier oscillator shape"))
            .with_param(DspParamSchema::knob("mix", "Dry / Wet Mix", 0.0, 1.0, 0.75, "%", MacroRole::Character, "Ring modulation depth blend"))
        );

        self.register(
            DspNodeDescriptor::new(
                "MasterDitherNode",
                "TPDF / Noise-Shaped Mastering Dither",
                DspNodeCategory::DynamicsMaster,
                "Triangular PDF and Lipshitz noise-shaped dither for 16-bit and 24-bit wordlength reduction.",
            )
            .with_param(DspParamSchema::choice("dither_type", "Dither Algorithm", &["TPDF (Standard)", "Triangular", "Flat Rectangular", "Noise-Shaped High"], 0, "Dither probability distribution"))
            .with_param(DspParamSchema::choice("bit_depth", "Target Bit Depth", &["16-Bit (CD Audio)", "24-Bit (Studio)", "8-Bit (Lo-Fi)"], 0, "Quantization bit depth"))
        );

        self.register(
            DspNodeDescriptor::new(
                "PolyphaseOversampler",
                "Linear-Phase Mastering Oversampler",
                DspNodeCategory::DynamicsMaster,
                "2x, 4x, and 8x polyphase half-band linear-phase oversampler preventing aliasing in nonlinear processing.",
            )
            .with_param(DspParamSchema::choice("factor", "Oversampling Factor", &["2x Oversampling", "4x Oversampling", "8x Oversampling"], 1, "Sampling rate multiplication ratio"))
        );

        self.register(
            DspNodeDescriptor::new(
                "SpatialRoomNode",
                "Shoebox Acoustic Room Model",
                DspNodeCategory::SpatialSurround,
                "Image-source early reflection geometric room acoustics with 6-wall surface absorption and dimensions.",
            )
            .with_param(DspParamSchema::knob("room_length", "Room Length", 2.0, 50.0, 10.0, "m", MacroRole::Space, "Acoustic space length"))
            .with_param(DspParamSchema::knob("room_width", "Room Width", 2.0, 40.0, 8.0, "m", MacroRole::Space, "Acoustic space width"))
            .with_param(DspParamSchema::knob("room_height", "Room Height", 2.0, 20.0, 3.5, "m", MacroRole::Space, "Acoustic space height"))
            .with_param(DspParamSchema::knob("wall_absorption", "Wall Absorption", 0.05, 0.95, 0.25, "%", MacroRole::Tone, "Average boundary wall reflection absorption coefficient"))
        );

        self.register(
            DspNodeDescriptor::new(
                "ElasticWarpEngine",
                "WSOLA Elastic Audio Time Stretcher",
                DspNodeCategory::TimeSpace,
                "Waveform Similarity Overlap-Add pitch-preserving elastic time stretcher and tempo alignment engine.",
            )
            .with_param(DspParamSchema::knob("speed_ratio", "Playback Speed Ratio", 0.25, 4.0, 1.0, "x", MacroRole::Punch, "Time stretch tempo playback multiplier"))
            .with_param(DspParamSchema::knob("window_ms", "WSOLA Window Size", 10.0, 100.0, 35.0, "ms", MacroRole::Tone, "Correlation analysis window length"))
        );

        // ==========================================
        // 23. Neural AI, Slicers, Recorders & Utilities
        // ==========================================
        self.register(
            DspNodeDescriptor::new(
                "StemSeparatorNode",
                "4-Stem Neural Source Separator",
                DspNodeCategory::NeuralAi,
                "Deep convolutional neural network separating complex mixed audio into Vocals, Drums, Bass, and Other.",
            )
            .with_param(DspParamSchema::knob("vocal_level", "Vocals Stem Gain", 0.0, 2.0, 1.0, "x", MacroRole::Tone, "Extracted vocals volume level"))
            .with_param(DspParamSchema::knob("drums_level", "Drums Stem Gain", 0.0, 2.0, 1.0, "x", MacroRole::Punch, "Extracted drum track volume level"))
            .with_param(DspParamSchema::knob("bass_level", "Bass Stem Gain", 0.0, 2.0, 1.0, "x", MacroRole::Punch, "Extracted bassline volume level"))
            .with_param(DspParamSchema::knob("other_level", "Other/Music Stem Gain", 0.0, 2.0, 1.0, "x", MacroRole::Space, "Extracted accompaniment volume level"))
        );

        self.register(
            DspNodeDescriptor::new(
                "AudioTransientSlicer",
                "AI Transient Beat Slicer",
                DspNodeCategory::SamplerSlicer,
                "Automatic transient onset detection and dynamic slice marker placement for drum loops and breaks.",
            )
            .with_param(DspParamSchema::knob("threshold", "Transient Sensitivity", 0.05, 1.0, 0.45, "%", MacroRole::Punch, "Onset detection threshold sensitivity"))
            .with_param(DspParamSchema::knob("min_slice_ms", "Minimum Slice Gap", 10.0, 200.0, 40.0, "ms", MacroRole::Tone, "Minimum duration between consecutive slices"))
        );

        self.register(
            DspNodeDescriptor::new(
                "MultiTrackDiskRecorder",
                "Multi-Track Disk Streaming Recorder",
                DspNodeCategory::Utility,
                "Deterministic zero-allocation multi-track WAV disk recording engine with punch-in and arming.",
            )
            .with_param(DspParamSchema::toggle("is_armed", "Record Arm Active", false, "Arm recorder for live incoming audio takes"))
            .with_param(DspParamSchema::knob("pre_roll_bars", "Pre-Roll Count", 0.0, 4.0, 1.0, "bars", MacroRole::None, "Count-in metronome bars before recording starts"))
        );

        self.register(
            DspNodeDescriptor::new(
                "AudioSampleEditor",
                "Non-Destructive Audio Clip Editor",
                DspNodeCategory::Utility,
                "Audio clip waveform editor with peak normalization, gain trim, fade ramps, and reverse.",
            )
            .with_param(DspParamSchema::knob("trim_gain", "Clip Trim Gain", -24.0, 24.0, 0.0, "dB", MacroRole::Punch, "Sample clip overall gain trim"))
            .with_param(DspParamSchema::toggle("reverse", "Reverse Playback", false, "Play sample backwards"))
            .with_param(DspParamSchema::knob("fade_in_ms", "Fade In Ramp", 0.0, 500.0, 5.0, "ms", MacroRole::Tone, "Onset linear fade-in time"))
            .with_param(DspParamSchema::knob("fade_out_ms", "Fade Out Ramp", 0.0, 500.0, 10.0, "ms", MacroRole::Space, "Ending linear fade-out time"))
        );

        self.register(
            DspNodeDescriptor::new(
                "StrobeChromaticTuner",
                "Strobe Chromatic Instrument Tuner",
                DspNodeCategory::Utility,
                "High-accuracy cent deviation strobe chromatic tuner with A440 calibration and pitch detection.",
            )
            .with_param(DspParamSchema::knob("ref_pitch", "Reference Pitch A4", 415.0, 466.0, 440.0, "Hz", MacroRole::Tone, "Concert pitch reference standard"))
            .with_param(DspParamSchema::knob("tolerance_cents", "Cent Tolerance", 0.5, 10.0, 2.0, "cents", MacroRole::Character, "In-tune detection tolerance window"))
        );

        self.register(
            DspNodeDescriptor::new(
                "ExternalPluginHost",
                "CLAP / VST3 External Plugin Host",
                DspNodeCategory::Utility,
                "Universal sandboxed host wrapping external third-party CLAP and VST3 plugins with parameter automation.",
            )
            .with_param(DspParamSchema::knob("dry_wet", "Plugin Dry / Wet Mix", 0.0, 1.0, 1.0, "%", MacroRole::Space, "Overall plugin output blend"))
            .with_param(DspParamSchema::knob("output_gain", "Plugin Output Gain", -24.0, 12.0, 0.0, "dB", MacroRole::Punch, "Post-plugin makeup gain"))
        );

        // ==========================================
        // 24. AI Autonomous Mixing & Neural Stems
        // ==========================================
        self.register(
            DspNodeDescriptor::new(
                "DemucsV4Separator",
                "Demucs v4 AI 6-Stem Separator",
                DspNodeCategory::NeuralAi,
                "Hybrid Spectrogram Transformer decomposing mixed audio into Vocals, Drums, Bass, Guitar, Piano, and Other.",
            )
            .with_param(DspParamSchema::knob("vocals_gain", "Vocals Gain", 0.0, 2.0, 1.0, "x", MacroRole::Tone, "Extracted vocals volume level"))
            .with_param(DspParamSchema::knob("drums_gain", "Drums Gain", 0.0, 2.0, 1.0, "x", MacroRole::Punch, "Extracted drum track volume level"))
            .with_param(DspParamSchema::knob("bass_gain", "Bass Gain", 0.0, 2.0, 1.0, "x", MacroRole::Punch, "Extracted bassline volume level"))
            .with_param(DspParamSchema::knob("guitar_gain", "Guitar Gain", 0.0, 2.0, 1.0, "x", MacroRole::Character, "Extracted guitar volume level"))
            .with_param(DspParamSchema::knob("piano_gain", "Piano Gain", 0.0, 2.0, 1.0, "x", MacroRole::Character, "Extracted piano/keys volume level"))
            .with_param(DspParamSchema::knob("other_gain", "Other Gain", 0.0, 2.0, 1.0, "x", MacroRole::Space, "Extracted accompaniment volume level"))
        );

        self.register(
            DspNodeDescriptor::new(
                "SpectralNeuralResynthesizerNode",
                "Spectral Neural Resynthesizer",
                DspNodeCategory::NeuralAi,
                "Real-time latent auto-encoder spectral re-synthesis with continuous timbre morphing.",
            )
            .with_param(DspParamSchema::knob("morph_factor", "Timbre Morph", 0.0, 1.0, 0.5, "%", MacroRole::Character, "Latent interpolation space"))
            .with_param(DspParamSchema::knob("temperature", "Sampling Temp", 0.1, 2.0, 1.0, "", MacroRole::Tone, "Neural sampling entropy"))
            .with_param(DspParamSchema::knob("mix", "Dry / Wet Mix", 0.0, 1.0, 0.8, "%", MacroRole::Space, "Re-synthesized signal blend"))
        );

        self.register(
            DspNodeDescriptor::new(
                "AiAutonomousMasteringEngine",
                "AI Autonomous Mastering Suite",
                DspNodeCategory::DynamicsMaster,
                "Intelligent neural mastering engine targeting EBU R128 integrated loudness with true-peak safety.",
            )
            .with_param(DspParamSchema::knob("target_lufs", "Target Loudness", -24.0, -6.0, -14.0, "LUFS", MacroRole::Punch, "EBU R128 Integrated Target"))
            .with_param(DspParamSchema::knob("ceiling_dbtp", "True-Peak Ceiling", -3.0, 0.0, -1.0, "dBTP", MacroRole::Punch, "Inter-sample peak ceiling"))
            .with_param(DspParamSchema::knob("dynamic_warmth", "Analog Warmth", 0.0, 1.0, 0.35, "%", MacroRole::Tone, "Tape and tube harmonic color"))
            .with_param(DspParamSchema::knob("stereo_expansion", "Stereo Imager", 0.0, 1.0, 0.25, "%", MacroRole::Space, "Phase-coherent stereo widening"))
        );

        self.register(
            DspNodeDescriptor::new(
                "VocalPitchFormantCorrectorNode",
                "Neural Vocal Pitch & Formant Shifter",
                DspNodeCategory::NeuralAi,
                "Deterministic neural vocal processor decoupling pitch tracking from vocal tract formant preservation.",
            )
            .with_param(DspParamSchema::knob("correction_speed_ms", "Correction Speed", 0.0, 100.0, 15.0, "ms", MacroRole::Punch, "Pitch snapping response time"))
            .with_param(DspParamSchema::knob("formant_shift_semitones", "Formant Shift", -12.0, 12.0, 0.0, "st", MacroRole::Tone, "Vocal tract length shift"))
            .with_param(DspParamSchema::knob("vibrato_depth", "Vibrato Depth", 0.0, 1.0, 0.2, "%", MacroRole::Character, "Natural vibrato modulation"))
            .with_param(DspParamSchema::knob("neural_smoothing", "Smoothing", 0.0, 1.0, 0.5, "%", MacroRole::Space, "Phase vocoder artifact smoothing"))
        );

        self.register(
            DspNodeDescriptor::new(
                "NeuralAudioRepairNode",
                "Neural Audio Restoration & Inpainting",
                DspNodeCategory::NeuralAi,
                "Spectral convolutional restoration network removing background noise, clicks, hum, and clipped bursts.",
            )
            .with_param(DspParamSchema::knob("denoise_amount", "De-Noise Depth", 0.0, 1.0, 0.6, "%", MacroRole::Tone, "Stationary & dynamic noise reduction"))
            .with_param(DspParamSchema::knob("declick_sensitivity", "De-Click Sens", 0.0, 1.0, 0.5, "%", MacroRole::Punch, "Transient click detection threshold"))
            .with_param(DspParamSchema::knob("inpaint_window_ms", "Inpaint Window", 1.0, 50.0, 10.0, "ms", MacroRole::Character, "Neural waveform reconstruction span"))
        );

        self.register(
            DspNodeDescriptor::new(
                "AiMixBalanceAnalyzer",
                "AI Dynamic Mix Balance Analyzer",
                DspNodeCategory::Utility,
                "Real-time psychoacoustic spectral masking analyzer detecting frequency clashes across active stems.",
            )
            .with_param(DspParamSchema::choice("spectral_resolution", "FFT Resolution", &["Standard (512)", "Fine (1024)", "Ultra (2048)"], 1, "Spectral analysis precision"))
            .with_param(DspParamSchema::knob("masking_threshold_db", "Masking Threshold", -30.0, 0.0, -12.0, "dB", MacroRole::Tone, "Clash detection sensitivity"))
            .with_param(DspParamSchema::knob("auto_duck_depth", "Auto-Ducking", 0.0, 1.0, 0.4, "%", MacroRole::Punch, "Intelligent conflicting band ducking"))
        );

        self.register(
            DspNodeDescriptor::new(
                "AutomatedDrumReplacer",
                "AI Transient Drum Sound Replacer",
                DspNodeCategory::SamplerSlicer,
                "Dynamic transient onset tracker triggering replacement drum samples with dynamic velocity matching.",
            )
            .with_param(DspParamSchema::knob("trigger_threshold_db", "Threshold", -40.0, 0.0, -18.0, "dB", MacroRole::Punch, "Transient trigger threshold"))
            .with_param(DspParamSchema::knob("retrigger_holdoff_ms", "Holdoff Gap", 10.0, 200.0, 50.0, "ms", MacroRole::Punch, "Minimum time between hits"))
            .with_param(DspParamSchema::knob("sample_velocity_dynamic", "Velocity Scaling", 0.0, 1.0, 0.8, "%", MacroRole::Character, "Hit velocity sensitivity"))
            .with_param(DspParamSchema::knob("blend_original", "Original Bleed", 0.0, 1.0, 0.2, "%", MacroRole::Space, "Acoustic mic bleed blend"))
        );

        self.register(
            DspNodeDescriptor::new(
                "NeuralHarmonicExciterNode",
                "Neural Valve & Air Harmonic Exciter",
                DspNodeCategory::DistortionSaturation,
                "High-frequency dynamic harmonic generator adding analog air, presence, and vintage warmth.",
            )
            .with_param(DspParamSchema::log_knob("air_freq_hz", "Air Band", 5000.0, 20000.0, 12000.0, "Hz", MacroRole::Tone, "Exciter corner frequency"))
            .with_param(DspParamSchema::knob("drive_db", "Exciter Drive", 0.0, 24.0, 6.0, "dB", MacroRole::Punch, "Harmonic generation drive"))
            .with_param(DspParamSchema::choice("tube_color", "Harmonic Profile", &["Triode Warmth", "Pentode Sparkle", "Tape Flux", "Transformer"], 0, "Color profile"))
            .with_param(DspParamSchema::knob("mix", "Dry / Wet Mix", 0.0, 1.0, 0.35, "%", MacroRole::Space, "Exciter signal blend"))
        );

        self.register(
            DspNodeDescriptor::new(
                "AiSongStructureDetector",
                "AI Song Structure & Energy Tracker",
                DspNodeCategory::Utility,
                "Deep neural audio analyzer detecting Intro, Verse, Chorus, Bridge, Drop, and Outro arrangement boundaries.",
            )
            .with_param(DspParamSchema::knob("min_section_bars", "Min Section", 2.0, 32.0, 8.0, "bars", MacroRole::None, "Minimum section duration"))
            .with_param(DspParamSchema::knob("energy_sensitivity", "Energy Sensitivity", 0.0, 1.0, 0.5, "%", MacroRole::Tone, "Dynamic boundary sensitivity"))
            .with_param(DspParamSchema::toggle("auto_marker_placement", "Auto Marker Placement", true, "Place timeline marker flags automatically"))
        );

        self.register(
            DspNodeDescriptor::new(
                "AutomatedDynamicEqNode",
                "AI Anti-Masking Dynamic Equalizer",
                DspNodeCategory::FilterEq,
                "Intelligent multi-band dynamic equalizer suppressing transient frequency clashes in real time.",
            )
            .with_param(DspParamSchema::knob("num_dynamic_bands", "Band Count", 1.0, 8.0, 4.0, "bands", MacroRole::Tone, "Active dynamic filter bands"))
            .with_param(DspParamSchema::knob("max_cut_db", "Max Cut", -18.0, 0.0, -6.0, "dB", MacroRole::Punch, "Maximum dynamic reduction"))
            .with_param(DspParamSchema::knob("detection_sensitivity", "Sensitivity", 0.0, 1.0, 0.65, "%", MacroRole::Character, "Sidechain detection sensitivity"))
            .with_param(DspParamSchema::toggle("sidechain_unmask", "Sidechain Unmask", true, "Enable multi-track unmasking"))
        );

        self.register(
            DspNodeDescriptor::new(
                "NeuralTransientShaperNode",
                "Neural Multi-Band Transient Shaper",
                DspNodeCategory::DynamicsMaster,
                "Frequency-aware neural transient controller shaping punch, attack crack, and tail room decay.",
            )
            .with_param(DspParamSchema::knob("attack_gain_db", "Attack Gain", -12.0, 12.0, 3.0, "dB", MacroRole::Punch, "Initial transient punch boost/cut"))
            .with_param(DspParamSchema::knob("sustain_gain_db", "Sustain Gain", -12.0, 12.0, 0.0, "dB", MacroRole::Space, "Body and tail sustain gain"))
            .with_param(DspParamSchema::log_knob("transient_split_hz", "Crossover Freq", 100.0, 8000.0, 1200.0, "Hz", MacroRole::Tone, "Split band crossover"))
            .with_param(DspParamSchema::toggle("neural_detection", "Neural Envelope Tracking", true, "High-precision neural onset tracking"))
        );

        self.register(
            DspNodeDescriptor::new(
                "AiPolyphonicChordExtractor",
                "AI Polyphonic Chord & Key Extractor",
                DspNodeCategory::Utility,
                "Neural chromagram analyzer extracting polyphonic chord progressions, modal keys, and microtonal roots.",
            )
            .with_param(DspParamSchema::choice("chroma_resolution", "Tuning Framework", &["12-Tone Standard", "24-Tone Microtonal", "31-EDO Fokker", "53-EDO JI"], 0, "Chroma detection scale"))
            .with_param(DspParamSchema::knob("harmonic_confidence_min", "Confidence Gate", 0.0, 1.0, 0.6, "%", MacroRole::Tone, "Minimum detection certainty"))
            .with_param(DspParamSchema::toggle("auto_midi_export", "Live MIDI Chord Stream", true, "Stream detected chord notes to piano roll"))
        );

        self.register(
            DspNodeDescriptor::new(
                "NeuralBassGeneratorNode",
                "Neural Sub-Bass Synthesizer",
                DspNodeCategory::Oscillator,
                "Deep sub-harmonic bassline synthesizer generating complementary bottom-end foundation from audio inputs.",
            )
            .with_param(DspParamSchema::knob("sub_octave_mix", "Sub Octave Level", 0.0, 1.0, 0.7, "%", MacroRole::Punch, "Synthesized sub-bass volume"))
            .with_param(DspParamSchema::knob("saturation_drive", "Drive Saturation", 0.0, 1.0, 0.4, "%", MacroRole::Character, "Even/odd harmonic saturation"))
            .with_param(DspParamSchema::log_knob("lowpass_cutoff_hz", "Cutoff Freq", 40.0, 500.0, 160.0, "Hz", MacroRole::Tone, "Lowpass filter cutoff"))
            .with_param(DspParamSchema::knob("sidechain_duck", "Kick Ducking", 0.0, 1.0, 0.5, "%", MacroRole::Space, "Transient ducking depth"))
        );

        self.register(
            DspNodeDescriptor::new(
                "AudioAlignmentTool",
                "Sub-Sample Phase Aligner & Polarity",
                DspNodeCategory::Utility,
                "Sub-sample correlation alignment engine eliminating comb filtering between multi-mic recordings.",
            )
            .with_param(DspParamSchema::knob("max_delay_ms", "Delay Correction", 0.0, 50.0, 10.0, "ms", MacroRole::Space, "Time alignment delay"))
            .with_param(DspParamSchema::toggle("polarity_invert", "Invert Phase (180°)", false, "Flip acoustic polarity"))
            .with_param(DspParamSchema::toggle("sub_sample_accuracy", "Sinc Interpolation", true, "Fractional sample alignment"))
        );

        self.register(
            DspNodeDescriptor::new(
                "SingingSynthesisNode",
                "Neural Text-to-Singing Voice Synthesizer",
                DspNodeCategory::NeuralAi,
                "Deep autoregressive vocal synthesizer singing phonetic lyrics with microtonal pitch expressivity.",
            )
            .with_param(DspParamSchema::choice("voice_model", "Voice Character", &["Neural Soprano", "Neural Alto", "Neural Tenor", "Neural Baritone", "Synthesizer Voice"], 0, "Vocal identity timbre"))
            .with_param(DspParamSchema::knob("vibrato_rate_hz", "Vibrato Speed", 2.0, 10.0, 5.5, "Hz", MacroRole::Character, "LFO pitch modulation speed"))
            .with_param(DspParamSchema::knob("breathiness", "Aspiration Breath", 0.0, 1.0, 0.25, "%", MacroRole::Tone, "Vocal fold unvoiced air"))
            .with_param(DspParamSchema::knob("consonant_clarity", "Consonant Attack", 0.0, 1.0, 0.75, "%", MacroRole::Punch, "Phoneme transient articulation"))
        );

        self.register(
            DspNodeDescriptor::new(
                "NeuralRoomAcousticMatcher",
                "Neural Room Acoustic Matcher",
                DspNodeCategory::SpatialSurround,
                "Deep convolutional acoustic matcher recreating the reverberation, reflection, and absorption of reference spaces.",
            )
            .with_param(DspParamSchema::knob("rt60_target_sec", "Target RT60", 0.1, 10.0, 1.8, "s", MacroRole::Space, "Decay reverberation time"))
            .with_param(DspParamSchema::knob("spectral_tilt", "Wall Spectral Tilt", -6.0, 6.0, 0.0, "dB/oct", MacroRole::Tone, "High/low frequency absorption slope"))
            .with_param(DspParamSchema::knob("direct_to_reverb_ratio", "D/R Ratio", -24.0, 24.0, 0.0, "dB", MacroRole::Punch, "Direct versus diffuse balance"))
            .with_param(DspParamSchema::knob("ir_length_sec", "IR Tail Length", 0.5, 8.0, 3.0, "s", MacroRole::Character, "Convolution impulse tail duration"))
        );

        // ==========================================
        // 25. Neural Voice, AutoTune, Gates & De-Essers
        // ==========================================
        self.register(
            DspNodeDescriptor::new(
                "AutoTuneNode",
                "Real-Time AutoTune Pitch Snapper",
                DspNodeCategory::NeuralAi,
                "Zero-latency pitch quantization engine snapping input pitch to scale intervals with custom speed.",
            )
            .with_param(DspParamSchema::knob("correction_speed", "Retune Speed", 0.0, 1.0, 0.85, "%", MacroRole::Punch, "Pitch snapping response time"))
            .with_param(DspParamSchema::choice("scale_tuning", "Target Scale", &["Chromatic", "Major", "Minor", "Pentatonic", "Microtonal Scala"], 0, "Musical scale quantization"))
            .with_param(DspParamSchema::knob("humanize", "Humanize Pitch", 0.0, 1.0, 0.15, "%", MacroRole::Character, "Natural pitch wobble tolerance"))
            .with_param(DspParamSchema::knob("pitch_amount", "Correction Depth", 0.0, 1.0, 1.0, "%", MacroRole::Tone, "Overall tuning blend"))
        );

        self.register(
            DspNodeDescriptor::new(
                "RnnoiseNode",
                "RNNoise Recurrent Neural Denoiser",
                DspNodeCategory::NeuralAi,
                "GRU neural network suppressing background microphone noise, fan hiss, and room hum with 0ms latency.",
            )
            .with_param(DspParamSchema::choice("suppression_level", "Denoise Profile", &["Mild (-6dB)", "Medium (-12dB)", "Aggressive (-18dB)", "Maximum (-24dB)"], 1, "Noise attenuation strength"))
            .with_param(DspParamSchema::knob("voice_activity_threshold", "Voice Gate Sens", 0.0, 1.0, 0.5, "%", MacroRole::Punch, "VAD speech probability threshold"))
        );

        self.register(
            DspNodeDescriptor::new(
                "AiAutoGainNode",
                "Intelligent Auto-Gain Leveller",
                DspNodeCategory::DynamicsMaster,
                "Continuous LUFS loudness normalizer riding vocal and instrument gains smoothly without pumping.",
            )
            .with_param(DspParamSchema::knob("target_lufs", "Target Loudness", -24.0, -6.0, -14.0, "LUFS", MacroRole::Punch, "Integrated target level"))
            .with_param(DspParamSchema::knob("max_gain_boost_db", "Max Boost", 0.0, 18.0, 6.0, "dB", MacroRole::Punch, "Maximum allowable makeup gain"))
            .with_param(DspParamSchema::knob("response_time_ms", "Riding Window", 50.0, 2000.0, 300.0, "ms", MacroRole::Space, "Gain adjustment integration window"))
        );

        self.register(
            DspNodeDescriptor::new(
                "DdspTimbreTransferNode",
                "DDSP Neural Timbre Transfer",
                DspNodeCategory::NeuralAi,
                "Differentiable Digital Signal Processing neural synthesizer transforming arbitrary audio into violin, flute, or brass.",
            )
            .with_param(DspParamSchema::choice("target_timbre", "Target Instrument", &["Violin", "Flute", "Trumpet", "Cello", "Electric Guitar"], 0, "Target neural model"))
            .with_param(DspParamSchema::knob("pitch_shift_st", "Pitch Shift", -24.0, 24.0, 0.0, "st", MacroRole::Tone, "Instrument transposition"))
            .with_param(DspParamSchema::knob("loudness_scale", "Loudness Scale", 0.0, 2.0, 1.0, "x", MacroRole::Punch, "Dynamics scaling"))
            .with_param(DspParamSchema::knob("confidence_gate", "Confidence Gate", 0.0, 1.0, 0.4, "%", MacroRole::Character, "Unvoiced noise threshold"))
        );

        self.register(
            DspNodeDescriptor::new(
                "VocalHarmonyGeneratorNode",
                "Intelligent Polyphonic Vocal Harmonizer",
                DspNodeCategory::NeuralAi,
                "Multi-voice vocal harmonizer generating diatonic third, fifth, and octave backing vocal harmonies.",
            )
            .with_param(DspParamSchema::choice("harmony_preset", "Harmonic Arrangement", &["Third Above + Fifth Above", "Octave Down + Third Up", "Four-Part Choir", "Barbershop Quartet"], 0, "Vocal voicing preset"))
            .with_param(DspParamSchema::knob("humanize_pitch_cents", "Pitch Detune", 0.0, 30.0, 8.0, "cents", MacroRole::Character, "Stereo unison micro-detuning"))
            .with_param(DspParamSchema::knob("harmony_level", "Harmony Level", 0.0, 1.0, 0.6, "%", MacroRole::Space, "Backing vocals balance"))
        );

        self.register(
            DspNodeDescriptor::new(
                "NeuralSuperResolutionNode",
                "Neural Audio Super-Resolution",
                DspNodeCategory::NeuralAi,
                "Deep generative model predicting high-frequency harmonics and air beyond band-limited audio.",
            )
            .with_param(DspParamSchema::choice("upsample_factor", "Bandwidth Target", &["2x High-Frequency", "4x Ultra-Resolution (96kHz)"], 0, "Target bandwidth"))
            .with_param(DspParamSchema::knob("air_harmonic_boost_db", "Air Boost", 0.0, 12.0, 3.0, "dB", MacroRole::Tone, "Generated top-end boost"))
        );

        self.register(
            DspNodeDescriptor::new(
                "NoiseGateNode",
                "Fast Attack Studio Noise Gate",
                DspNodeCategory::DynamicsMaster,
                "Downward studio noise gate silencing mic bleed and amplifier hum below an adjustable threshold.",
            )
            .with_param(DspParamSchema::knob("threshold_db", "Gate Threshold", -60.0, 0.0, -36.0, "dB", MacroRole::Punch, "Threshold level"))
            .with_param(DspParamSchema::knob("attack_ms", "Attack Time", 0.1, 50.0, 2.0, "ms", MacroRole::Punch, "Gate opening speed"))
            .with_param(DspParamSchema::knob("hold_ms", "Hold Time", 1.0, 500.0, 50.0, "ms", MacroRole::Space, "Duration gate stays open"))
            .with_param(DspParamSchema::knob("release_ms", "Release Time", 10.0, 1000.0, 100.0, "ms", MacroRole::Space, "Gate closing speed"))
        );

        self.register(
            DspNodeDescriptor::new(
                "DeesserNode",
                "Split-Band Sibilance De-Esser",
                DspNodeCategory::DynamicsMaster,
                "High-frequency dynamic compressor taming harsh 's', 'sh', and 't' vocal sibilance.",
            )
            .with_param(DspParamSchema::log_knob("split_frequency_hz", "Split Freq", 3000.0, 12000.0, 6500.0, "Hz", MacroRole::Tone, "Sibilance band detection"))
            .with_param(DspParamSchema::knob("threshold_db", "Threshold", -36.0, 0.0, -18.0, "dB", MacroRole::Punch, "Detection threshold"))
            .with_param(DspParamSchema::knob("reduction_amount_db", "Reduction Amount", 0.0, 24.0, 6.0, "dB", MacroRole::Punch, "Maximum sibilance attenuation"))
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dsp_node_registry_comprehensive_coverage() {
        let registry = DspNodeRegistry::new();
        let all = registry.list_all();
        assert!(
            all.len() >= 160,
            "Registry should contain extensive DSP modules (found {})",
            all.len()
        );

        let oscs = registry.list_by_category(DspNodeCategory::Oscillator);
        assert!(!oscs.is_empty(), "Oscillators must be registered");

        let physical = registry.list_by_category(DspNodeCategory::AcousticPhysicalModel);
        assert!(!physical.is_empty(), "Physical models must be registered");

        let filters = registry.list_by_category(DspNodeCategory::FilterEq);
        assert!(!filters.is_empty(), "Filters must be registered");

        let dynamics = registry.list_by_category(DspNodeCategory::DynamicsMaster);
        assert!(!dynamics.is_empty(), "Dynamics must be registered");

        let dist = registry.list_by_category(DspNodeCategory::DistortionSaturation);
        assert!(!dist.is_empty(), "Distortion nodes must be registered");

        let time = registry.list_by_category(DspNodeCategory::TimeSpace);
        assert!(!time.is_empty(), "Time & space nodes must be registered");

        let spatial = registry.list_by_category(DspNodeCategory::SpatialSurround);
        assert!(!spatial.is_empty(), "Spatial nodes must be registered");

        let spectral = registry.list_by_category(DspNodeCategory::SpectralResynthesis);
        assert!(!spectral.is_empty(), "Spectral nodes must be registered");

        let neural = registry.list_by_category(DspNodeCategory::NeuralAi);
        assert!(!neural.is_empty(), "Neural AI nodes must be registered");

        // Verify AI Mixing and Neural Vocal nodes
        assert!(registry.get("DemucsV4Separator").is_some());
        assert!(registry.get("SingingSynthesisNode").is_some());
        assert!(registry.get("AiAutonomousMasteringEngine").is_some());
        assert!(registry.get("AutoTuneNode").is_some());
        assert!(registry.get("DdspTimbreTransferNode").is_some());
        assert!(registry.get("NeuralAudioRepairNode").is_some());

        // Verify default node config creation
        let gamelan_desc = registry.get("GamelanGender").expect("GamelanGender must exist");
        let node_cfg = gamelan_desc.default_node_config();
        assert_eq!(node_cfg.kind, "GamelanGender");
        assert!(node_cfg.params.contains_key("mallet_hardness"));
    }

    #[test]
    fn test_dsp_node_descriptor_parameters_and_macro_bindings() {
        let registry = DspNodeRegistry::new();
        let aether = registry.get("AetherSynth").expect("AetherSynth must exist");
        assert_eq!(aether.display_name, "Aether Dual-Stack Subtractive Synth");
        assert_eq!(aether.category, DspNodeCategory::CompositeSynth);

        assert!(aether
            .params
            .iter()
            .any(|p| p.macro_role == MacroRole::Tone));
        assert!(aether
            .params
            .iter()
            .any(|p| p.macro_role == MacroRole::Character));
        assert!(aether
            .params
            .iter()
            .any(|p| p.macro_role == MacroRole::Punch));

        let shakuhachi = registry.get("Shakuhachi").expect("Shakuhachi must exist");
        assert_eq!(shakuhachi.category, DspNodeCategory::AcousticPhysicalModel);
        assert_eq!(shakuhachi.params.len(), 3);
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
