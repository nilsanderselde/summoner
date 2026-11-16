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

pub mod types;
pub use types::*;
mod aliases;
mod categories;
mod inventory;


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

        categories::register_all_descriptors(&mut descriptors);
        Self { descriptors }
    }

    /// Retrieve a node descriptor by kind ID.
    pub fn get(&self, kind_id: &str) -> Option<&DspNodeDescriptor> {
        if let Some(desc) = self.descriptors.get(kind_id) {
            return Some(desc);
        }
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
        inventory::inventory_list()
    }

    /// Canonical normalization mapping common aliases and case variations to registered type names.
    pub fn normalize_type_name(name: &str) -> Option<&'static str> {
        aliases::normalize_type_name(name)
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
        if let Some(node) = categories::try_create_node_ui(target_name) {
            return Some(node);
        }
        let cat = Self::inventory()
            .into_iter()
            .find(|(n, _, _)| *n == target_name)
            .map(|(_, c, _)| c)
            .unwrap_or(DspNodeCategory::Utility);
        Some(Box::new(
            GenericDspNodeUi::new(
                Box::leak(target_name.to_string().into_boxed_str()),
                target_name,
                cat,
            )
            .with_param(DspParamDescriptor::new_linear("macro_1", "Macro Dial 1", 0.0, 1.0, 0.5, "%", (0, 229, 255)))
            .with_param(DspParamDescriptor::new_linear("macro_2", "Macro Dial 2", 0.0, 1.0, 0.5, "%", (255, 171, 0)))
        ))
    }
}

#[cfg(test)]
mod tests;
