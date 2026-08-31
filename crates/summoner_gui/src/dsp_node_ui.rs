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
            all.len() >= 25,
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
