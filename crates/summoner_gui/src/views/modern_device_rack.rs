// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Reusable Device Box Primitive with Rotary Knobs, Faders, Live CRT Oscilloscope & Filter HUD.

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, FontId, Rect, RichText, Rounding, Stroke, Vec2};
use serde::{Deserialize, Serialize};

pub const MIN_KNOB_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding target

/// A visual representation of a single DSP device slot in a track's processing chain.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RackChainDeviceVisual {
    pub kind: String,
    pub display_name: String,
    pub is_bypassed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModernDeviceRackState {
    pub device_name: String,
    #[serde(default)]
    pub selected_node_kind: Option<String>,
    #[serde(default)]
    pub node_param_values: std::collections::HashMap<String, f32>,
    pub is_enabled: bool,
    pub is_minimized: bool,
    // Knobs
    pub cutoff: f32,
    pub resonance: f32,
    pub decay: f32,
    pub env_decay: f32,
    pub mod_amt: f32,
    pub drive: f32,
    // Faders
    pub osc_mix: f32,
    pub shape: f32,
    pub volume: f32,
    // Filter Curve Nodes (4 node frequencies 0.0..1.0 and gains)
    pub filter_nodes: [(f32, f32); 4],
    // LFO params
    pub lfo_speed: f32,
    pub lfo_depth: f32,
    #[serde(default)]
    pub is_expanded_params: bool,
    #[serde(default)]
    pub requested_automation_param: Option<String>,
    #[serde(default)]
    pub chain_devices: Vec<RackChainDeviceVisual>,
    #[serde(default)]
    pub selected_chain_idx: usize,
    #[serde(default)]
    pub requested_open_crystal_hud: bool,
    #[serde(default)]
    pub requested_open_armonica_hud: bool,
    #[serde(default)]
    pub requested_open_hurdy_gurdy_hud: bool,
    #[serde(default)]
    pub requested_open_trompette_hud: bool,
    #[serde(default)]
    pub requested_open_hoa5_radar_hud: bool,
    #[serde(default)]
    pub requested_open_neural_morph_hud: bool,
}

impl Default for ModernDeviceRackState {
    fn default() -> Self {
        Self {
            device_name: "Synth 1".to_string(),
            selected_node_kind: Some("AetherSynth".to_string()),
            node_param_values: std::collections::HashMap::new(),
            is_enabled: true,
            is_minimized: false,
            is_expanded_params: false,
            requested_automation_param: None,
            chain_devices: Vec::new(),
            selected_chain_idx: 0,
            requested_open_crystal_hud: false,
            requested_open_armonica_hud: false,
            requested_open_hurdy_gurdy_hud: false,
            requested_open_trompette_hud: false,
            requested_open_hoa5_radar_hud: false,
            requested_open_neural_morph_hud: false,
            cutoff: 0.65,
            resonance: 0.45,
            decay: 0.50,
            env_decay: 0.35,
            mod_amt: 0.60,
            drive: 0.40,
            osc_mix: 0.75,
            shape: 0.50,
            volume: 0.85,
            filter_nodes: [
                (0.15, 0.70),
                (0.40, 0.65),
                (0.65, 0.55),
                (0.90, 0.20),
            ],
            lfo_speed: 0.40,
            lfo_depth: 0.60,
        }
    }
}

impl ModernDeviceRackState {
    /// Ensure the chain has at least one device matching the currently selected node kind.
    pub fn ensure_chain(&mut self) {
        if self.chain_devices.is_empty() {
            let kind = self.selected_node_kind.clone().unwrap_or_else(|| "AetherSynth".to_string());
            let name = self.device_name.clone();
            self.chain_devices.push(RackChainDeviceVisual {
                kind,
                display_name: name,
                is_bypassed: !self.is_enabled,
            });
            self.selected_chain_idx = 0;
        }
    }

    /// Select a device slot in the track chain.
    pub fn select_device(&mut self, idx: usize) -> bool {
        self.ensure_chain();
        if idx < self.chain_devices.len() {
            self.selected_chain_idx = idx;
            let dev = &self.chain_devices[idx];
            self.selected_node_kind = Some(dev.kind.clone());
            self.device_name = dev.display_name.clone();
            self.is_enabled = !dev.is_bypassed;
            true
        } else {
            false
        }
    }

    /// Add a new DSP device module to the end of the track chain and select it.
    pub fn add_device(&mut self, kind: &str, display_name: &str) -> usize {
        self.ensure_chain();
        self.chain_devices.push(RackChainDeviceVisual {
            kind: kind.to_string(),
            display_name: display_name.to_string(),
            is_bypassed: false,
        });
        let new_idx = self.chain_devices.len() - 1;
        self.select_device(new_idx);
        new_idx
    }

    /// Remove a DSP device from the chain (keeping at least 1 device).
    pub fn remove_device(&mut self, idx: usize) -> bool {
        self.ensure_chain();
        if self.chain_devices.len() > 1 && idx < self.chain_devices.len() {
            self.chain_devices.remove(idx);
            if self.selected_chain_idx >= self.chain_devices.len() {
                self.selected_chain_idx = self.chain_devices.len() - 1;
            }
            let sel = self.selected_chain_idx;
            self.select_device(sel);
            true
        } else {
            false
        }
    }

    /// Move a device within the chain from `from` to `to`.
    pub fn move_device(&mut self, from: usize, to: usize) -> bool {
        self.ensure_chain();
        if from < self.chain_devices.len() && to < self.chain_devices.len() && from != to {
            let dev = self.chain_devices.remove(from);
            self.chain_devices.insert(to, dev);
            self.selected_chain_idx = to;
            true
        } else {
            false
        }
    }

    /// Toggle bypass state for a device in the chain.
    pub fn toggle_device_bypass(&mut self, idx: usize) -> bool {
        self.ensure_chain();
        if let Some(dev) = self.chain_devices.get_mut(idx) {
            dev.is_bypassed = !dev.is_bypassed;
            if idx == self.selected_chain_idx {
                self.is_enabled = !dev.is_bypassed;
            }
            true
        } else {
            false
        }
    }

    /// Request live parameter automation for a given parameter ID (e.g. from right-click or auto button).
    pub fn request_automation(&mut self, param_id: &str) {
        if self.selected_chain_idx > 0 && !param_id.starts_with("node_") && !param_id.starts_with("master_node_") {
            self.requested_automation_param = Some(format!("node_{}_{}", self.selected_chain_idx, param_id));
        } else {
            self.requested_automation_param = Some(param_id.to_string());
        }
    }

    /// Reset standard device rack knobs to factory default values.
    pub fn reset_knob_defaults(&mut self) {
        self.cutoff = 0.65;
        self.resonance = 0.45;
        self.decay = 0.50;
        self.env_decay = 0.35;
        self.mod_amt = 0.60;
        self.drive = 0.40;
        self.osc_mix = 0.75;
        self.shape = 0.50;
        self.volume = 0.85;
        self.lfo_speed = 0.40;
        self.lfo_depth = 0.60;
    }
}

#[cfg(feature = "gui")]
pub fn show_modern_device_rack(
    ui: &mut egui::Ui,
    state: &mut ModernDeviceRackState,
    oscilloscope_data: Option<&[f32]>,
) {
    show_modern_device_rack_with_context(ui, state, oscilloscope_data, None, 1);
}

#[cfg(feature = "gui")]
pub fn show_modern_device_rack_with_context(
    ui: &mut egui::Ui,
    state: &mut ModernDeviceRackState,
    oscilloscope_data: Option<&[f32]>,
    param_bus: Option<&summoner_core::param_bus::ParamBus>,
    track_id: u64,
) {
    state.ensure_chain();
    state.node_param_values.entry("cutoff".to_string()).or_insert(state.cutoff);
    state.node_param_values.entry("resonance".to_string()).or_insert(state.resonance);
    state.node_param_values.entry("decay".to_string()).or_insert(state.decay);
    state.node_param_values.entry("drive".to_string()).or_insert(state.drive);

    let registry = crate::dsp_node_ui::DspNodeRegistry::new();
    let cur_selection = state.selected_node_kind.clone().unwrap_or_else(|| "AetherSynth".to_string());
    let opt_desc = registry.get(&cur_selection);
    let (r, g, b) = opt_desc.map(|d| d.category.color_rgb()).unwrap_or((56, 189, 248));
    let cat_accent = Color32::from_rgb(r, g, b);

    // Collapsed Drawer Mode (Click-to-Expand Pro Rack Drawer)
    if state.is_minimized {
        let collapsed_h = 36.0;
        egui::Frame::none()
            .fill(Color32::from_rgb(14, 20, 32))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(28, 40, 60)))
            .rounding(Rounding::same(6.0))
            .inner_margin(egui::Margin::symmetric(10.0, 6.0))
            .show(ui, |ui| {
                ui.set_height(collapsed_h);
                ui.horizontal(|ui| {
                    if ui.button(RichText::new("▲ EXPAND RACK").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(56, 189, 248))).clicked() {
                        state.is_minimized = false;
                    }
                    if ui.button(RichText::new("⊞ PRO").font(FontId::proportional(10.0)).color(Color32::from_rgb(200, 215, 235))).clicked() {
                        state.is_minimized = false;
                        state.is_expanded_params = true;
                    }
                    if ui.button(RichText::new("📈 AUTO").font(FontId::proportional(10.0)).color(Color32::from_rgb(234, 179, 8))).on_hover_text("Open Live Bézier Automation Lane for primary parameter").clicked() {
                        let primary = opt_desc.and_then(|d| d.params.first()).map(|p| p.id.clone()).unwrap_or_else(|| "cutoff".to_string());
                        state.requested_automation_param = Some(primary);
                    }
                    let is_crystal = cur_selection == "CrystalResonator" || cur_selection.contains("Crystal") || cur_selection.contains("Bowl");
                    let is_armonica = cur_selection == "GlassArmonica" || cur_selection == "FranklinGlassArmonica" || cur_selection == "ArmonicaChassisResonator" || cur_selection.contains("Armonica");
                    let is_hurdy = cur_selection == "FrenchHurdyGurdy" || cur_selection.contains("HurdyGurdy") || cur_selection.contains("Vielle");
                    let is_trompette = cur_selection == "TrompetteChienBridge" || cur_selection == "TrompetteBridge" || cur_selection.contains("Trompette") || cur_selection.contains("Chien");
                    let is_hoa = cur_selection == "AmbisonicRadarSpatializer" || cur_selection == "HoaSpatializer" || cur_selection == "Hoa5BinauralSpatializer" || cur_selection == "Hoa5RadarView" || cur_selection.contains("Ambisonic") || cur_selection.contains("Hoa");
                    if is_crystal {
                        if ui.button(RichText::new("🔮 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(56, 189, 248))).on_hover_text("Open Physical Modeling Crystal Resonator HUD").clicked() {
                            state.requested_open_crystal_hud = true;
                        }
                    } else if is_armonica {
                        if ui.button(RichText::new("🍷 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(245, 158, 11))).on_hover_text("Open Physical Modeling Glass Armonica HUD").clicked() {
                            state.requested_open_armonica_hud = true;
                        }
                    } else if is_hurdy {
                        if ui.button(RichText::new("🎻 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(16, 185, 129))).on_hover_text("Open Physical Modeling Hurdy-Gurdy HUD").clicked() {
                            state.requested_open_hurdy_gurdy_hud = true;
                        }
                    } else if is_trompette {
                        if ui.button(RichText::new("🐕 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(239, 68, 68))).on_hover_text("Open Trompette Chien Buzzing Bridge HUD").clicked() {
                            state.requested_open_trompette_hud = true;
                        }
                    } else if is_hoa
                        && ui.button(RichText::new("🌐 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(56, 189, 248))).on_hover_text("Open 5th-Order Ambisonics (HOA5) 3D Radar HUD").clicked() {
                        state.requested_open_hoa5_radar_hud = true;
                    }
                    ui.add_space(4.0);
                    let pwr_col = if state.is_enabled { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(100, 116, 139) };
                    if ui.button(RichText::new("⏻").font(FontId::proportional(12.0)).color(pwr_col)).clicked() {
                        state.toggle_device_bypass(state.selected_chain_idx);
                    }
                    ui.add_space(4.0);

                    // Compact Device Chain Pills in Collapsed Header
                    let mut pill_to_select = None;
                    for (d_i, dev) in state.chain_devices.iter().enumerate() {
                        let is_d_sel = d_i == state.selected_chain_idx;
                        let card_col = if is_d_sel { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(148, 163, 184) };
                        let bg_col = if is_d_sel { Color32::from_rgb(24, 38, 58) } else { Color32::from_rgb(18, 24, 36) };
                        egui::Frame::none()
                            .fill(bg_col)
                            .stroke(Stroke::new(1.0_f32, if is_d_sel { card_col } else { Color32::from_rgb(36, 50, 74) }))
                            .rounding(Rounding::same(3.0))
                            .inner_margin(egui::Margin::symmetric(4.0, 2.0))
                            .show(ui, |ui| {
                                if ui.selectable_label(is_d_sel, RichText::new(format!("{}. {}", d_i + 1, dev.display_name)).font(FontId::proportional(9.0)).color(card_col)).clicked() {
                                    pill_to_select = Some(d_i);
                                }
                            });
                        ui.add_space(2.0);
                    }
                    if let Some(idx) = pill_to_select {
                        state.select_device(idx);
                    }

                    ui.add_space(4.0);

                    let cur_display = opt_desc.map(|d| format!("{} {}", d.category.icon(), d.display_name)).unwrap_or_else(|| state.device_name.clone());
                    egui::ComboBox::from_id_source("minimized_device_rack_module_selector")
                        .selected_text(RichText::new(cur_display).font(FontId::proportional(11.0)).color(cat_accent))
                        .show_ui(ui, |ui| {
                            for desc in registry.list_all() {
                                let is_sel = state.selected_node_kind.as_deref() == Some(&desc.kind_id);
                                let label = format!("{} {} ({})", desc.category.icon(), desc.display_name, desc.category.name());
                                if ui.selectable_label(is_sel, label).clicked() {
                                    state.selected_node_kind = Some(desc.kind_id.clone());
                                    state.device_name = desc.display_name.clone();
                                    if let Some(dev) = state.chain_devices.get_mut(state.selected_chain_idx) {
                                        dev.kind = desc.kind_id.clone();
                                        dev.display_name = desc.display_name.clone();
                                    }
                                    state.node_param_values.clear();
                                    for (p_i, schema) in desc.params.iter().enumerate() {
                                        let def = match &schema.widget {
                                            crate::dsp_node_ui::DspWidgetKind::RotaryKnob { default, .. }
                                            | crate::dsp_node_ui::DspWidgetKind::VerticalFader { default, .. } => *default,
                                            _ => 0.5,
                                        };
                                        state.node_param_values.insert(schema.id.clone(), def);
                                        if let Some(bus) = param_bus {
                                            let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 500 + p_i as u32);
                                            if bus.get(pid).is_some() {
                                                bus.set(pid, def);
                                            }
                                        }
                                    }
                                }
                            }
                        });

                    if let Some(desc) = opt_desc {
                        egui::Frame::none()
                            .fill(Color32::from_rgba_unmultiplied(r, g, b, 25))
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(r, g, b, 80)))
                            .rounding(Rounding::same(3.0))
                            .inner_margin(egui::Margin::symmetric(5.0, 2.0))
                            .show(ui, |ui| {
                                ui.label(RichText::new(desc.category.name()).font(FontId::proportional(9.0)).color(cat_accent));
                            });
                    }

                    ui.add_space(8.0);
                    let pills = [
                        ("Tone", state.cutoff),
                        ("Space", state.decay),
                        ("Punch", state.drive),
                        ("Vol", state.volume),
                    ];
                    for (name, val) in pills {
                        egui::Frame::none()
                            .fill(Color32::from_rgb(20, 28, 44))
                            .rounding(Rounding::same(3.0))
                            .inner_margin(egui::Margin::symmetric(5.0, 2.0))
                            .show(ui, |ui| {
                                ui.label(RichText::new(format!("{}: {:.0}%", name, val * 100.0)).font(FontId::proportional(9.0)).color(Color32::from_rgb(200, 215, 235)));
                            });
                        ui.add_space(2.0);
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new("Modular Rack Drawer (Collapsed)").font(FontId::proportional(9.0)).color(Color32::from_rgb(100, 116, 139)));
                    });
                });
            });
        return;
    }

    let rack_height = if state.is_expanded_params { 260.0 } else { 210.0 };
    egui::Frame::none()
        .fill(Color32::from_rgb(14, 20, 32))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(28, 40, 60)))
        .rounding(Rounding::same(6.0))
        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            ui.set_height(rack_height);

            // 1. Device Box Header (Rule 2 from GUI_RULES.md)
            ui.horizontal(|ui| {
                ui.label(RichText::new("Device Rack").font(FontId::proportional(12.0)).strong().color(Color32::from_rgb(241, 245, 249)));
                ui.add_space(8.0);
                let pwr_col = if state.is_enabled { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(100, 116, 139) };
                if ui.button(RichText::new("⏻").font(FontId::proportional(12.0)).color(pwr_col)).on_hover_text("Bypass / Enable Active Device").clicked() {
                    state.toggle_device_bypass(state.selected_chain_idx);
                }

                // DSP Module Selection Dropdown
                let cur_display = opt_desc.map(|d| format!("{} {}", d.category.icon(), d.display_name)).unwrap_or_else(|| state.device_name.clone());

                egui::ComboBox::from_id_source("device_rack_module_selector")
                    .selected_text(RichText::new(cur_display).font(FontId::proportional(11.0)).color(cat_accent))
                    .show_ui(ui, |ui| {
                        for desc in registry.list_all() {
                            let is_sel = state.selected_node_kind.as_deref() == Some(&desc.kind_id);
                            let label = format!("{} {} ({})", desc.category.icon(), desc.display_name, desc.category.name());
                            if ui.selectable_label(is_sel, label).clicked() {
                                state.selected_node_kind = Some(desc.kind_id.clone());
                                state.device_name = desc.display_name.clone();
                                if let Some(dev) = state.chain_devices.get_mut(state.selected_chain_idx) {
                                    dev.kind = desc.kind_id.clone();
                                    dev.display_name = desc.display_name.clone();
                                }
                                state.node_param_values.clear();
                                for (p_i, schema) in desc.params.iter().enumerate() {
                                    let def = match &schema.widget {
                                        crate::dsp_node_ui::DspWidgetKind::RotaryKnob { default, .. }
                                        | crate::dsp_node_ui::DspWidgetKind::VerticalFader { default, .. } => *default,
                                        _ => 0.5,
                                    };
                                    state.node_param_values.insert(schema.id.clone(), def);
                                    if let Some(bus) = param_bus {
                                        let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 500 + p_i as u32);
                                        if bus.get(pid).is_some() {
                                            bus.set(pid, def);
                                        }
                                    }
                                }
                            }
                        }
                    });

                if let Some(desc) = opt_desc {
                    egui::Frame::none()
                        .fill(Color32::from_rgba_unmultiplied(r, g, b, 25))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(r, g, b, 80)))
                        .rounding(Rounding::same(3.0))
                        .inner_margin(egui::Margin::symmetric(5.0, 2.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new(desc.category.name()).font(FontId::proportional(9.0)).color(cat_accent));
                        });
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(RichText::new("▼ COLLAPSE").font(FontId::proportional(10.0)).color(Color32::from_rgb(148, 163, 184))).clicked() {
                        state.is_minimized = true;
                    }
                    let pro_text = if state.is_expanded_params { "⊟ OVERVIEW" } else { "⊞ PRO PARAMS" };
                    let pro_col = if state.is_expanded_params { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(200, 215, 235) };
                    if ui.button(RichText::new(pro_text).font(FontId::proportional(10.0)).strong().color(pro_col)).clicked() {
                        state.is_expanded_params = !state.is_expanded_params;
                    }
                    if ui.button(RichText::new("📈 Auto").font(FontId::proportional(10.0)).color(Color32::from_rgb(234, 179, 8))).on_hover_text("Open Live Bézier Automation Lane for primary parameter").clicked() {
                        let primary = opt_desc.and_then(|d| d.params.first()).map(|p| p.id.clone()).unwrap_or_else(|| "cutoff".to_string());
                        state.requested_automation_param = Some(primary);
                    }
                    let is_crystal = cur_selection == "CrystalResonator" || cur_selection.contains("Crystal") || cur_selection.contains("Bowl");
                    let is_armonica = cur_selection == "GlassArmonica" || cur_selection == "FranklinGlassArmonica" || cur_selection == "ArmonicaChassisResonator" || cur_selection.contains("Armonica");
                    let is_hurdy = cur_selection == "FrenchHurdyGurdy" || cur_selection.contains("HurdyGurdy") || cur_selection.contains("Vielle");
                    let is_trompette = cur_selection == "TrompetteChienBridge" || cur_selection == "TrompetteBridge" || cur_selection.contains("Trompette") || cur_selection.contains("Chien");
                    let is_hoa = cur_selection == "AmbisonicRadarSpatializer" || cur_selection == "HoaSpatializer" || cur_selection == "Hoa5BinauralSpatializer" || cur_selection == "Hoa5RadarView" || cur_selection.contains("Ambisonic") || cur_selection.contains("Hoa");
                    let is_neural = cur_selection.contains("Neural") || cur_selection.contains("Timbre") || cur_selection.contains("Latent") || cur_selection.contains("Resynthesizer") || cur_selection == "NeuralMorphOrbView" || cur_selection == "NeuralTimbreMorph" || cur_selection == "NeuralWavetable";
                    if is_crystal {
                        if ui.button(RichText::new("🔮 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(56, 189, 248))).on_hover_text("Open Physical Modeling Crystal Resonator HUD").clicked() {
                            state.requested_open_crystal_hud = true;
                        }
                    } else if is_armonica {
                        if ui.button(RichText::new("🍷 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(245, 158, 11))).on_hover_text("Open Physical Modeling Glass Armonica HUD").clicked() {
                            state.requested_open_armonica_hud = true;
                        }
                    } else if is_hurdy {
                        if ui.button(RichText::new("🎻 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(16, 185, 129))).on_hover_text("Open Physical Modeling Hurdy-Gurdy HUD").clicked() {
                            state.requested_open_hurdy_gurdy_hud = true;
                        }
                    } else if is_trompette {
                        if ui.button(RichText::new("🐕 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(239, 68, 68))).on_hover_text("Open Trompette Chien Buzzing Bridge HUD").clicked() {
                            state.requested_open_trompette_hud = true;
                        }
                    } else if is_hoa {
                        if ui.button(RichText::new("🌐 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(56, 189, 248))).on_hover_text("Open 5th-Order Ambisonics (HOA5) 3D Radar HUD").clicked() {
                            state.requested_open_hoa5_radar_hud = true;
                        }
                    } else if is_neural
                        && ui.button(RichText::new("🧬 HUD").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(244, 63, 94))).on_hover_text("Open 2D Neural Timbre Morphing Orb HUD").clicked() {
                        state.requested_open_neural_morph_hud = true;
                    }
                    let _ = ui.small_button("✕");
                    egui::Frame::none()
                        .fill(Color32::from_rgb(24, 34, 52))
                        .rounding(Rounding::same(3.0))
                        .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new("Zero CLI Left Behind").font(FontId::proportional(9.0)).color(Color32::from_rgb(148, 163, 184)));
                        });
                });
            });

            ui.add_space(4.0);

            // 1b. Track DSP Device Chain Bar (Multi-node processing chain)
            ui.horizontal(|ui| {
                ui.label(RichText::new("CHAIN:").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(148, 163, 184)));
                ui.add_space(2.0);

                let mut dev_to_select = None;
                let mut dev_to_toggle_bypass = None;
                let mut dev_to_remove = None;
                let mut dev_move_left = None;
                let mut dev_move_right = None;

                for (d_i, dev) in state.chain_devices.iter().enumerate() {
                    let is_sel = d_i == state.selected_chain_idx;
                    let d_desc = registry.get(&dev.kind);
                    let (dr, dg, db) = d_desc.map(|d| d.category.color_rgb()).unwrap_or((56, 189, 248));
                    let d_accent = Color32::from_rgb(dr, dg, db);
                    let d_icon = d_desc.map(|d| d.category.icon()).unwrap_or("🎛️");

                    let border_col = if is_sel { d_accent } else { Color32::from_rgb(36, 50, 74) };
                    let bg_col = if is_sel { Color32::from_rgba_unmultiplied(dr, dg, db, 35) } else { Color32::from_rgb(18, 24, 36) };

                    egui::Frame::none()
                        .fill(bg_col)
                        .stroke(Stroke::new(if is_sel { 1.5_f32 } else { 1.0_f32 }, border_col))
                        .rounding(Rounding::same(4.0))
                        .inner_margin(egui::Margin::symmetric(5.0, 2.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let byp_col = if !dev.is_bypassed { Color32::from_rgb(34, 197, 94) } else { Color32::from_rgb(100, 116, 139) };
                                if ui.button(RichText::new("⏻").font(FontId::proportional(9.5)).color(byp_col)).on_hover_text(if dev.is_bypassed { "Device Bypassed (Click to Enable)" } else { "Device Active (Click to Bypass)" }).clicked() {
                                    dev_to_toggle_bypass = Some(d_i);
                                }
                                let badge_text = format!("{} {}. {}", d_icon, d_i + 1, dev.display_name);
                                let badge_resp = ui.selectable_label(is_sel, RichText::new(badge_text).font(FontId::proportional(9.5)).strong().color(if is_sel { Color32::WHITE } else { Color32::from_rgb(200, 215, 235) }));
                                if badge_resp.clicked() {
                                    dev_to_select = Some(d_i);
                                }
                            });
                        });
                    ui.add_space(2.0);
                }

                if let Some(idx) = dev_to_select {
                    state.select_device(idx);
                }
                if let Some(idx) = dev_to_toggle_bypass {
                    state.toggle_device_bypass(idx);
                }

                // Move / Remove controls for active device
                if state.chain_devices.len() > 1 {
                    if state.selected_chain_idx > 0
                        && ui.small_button(RichText::new("◀").font(FontId::proportional(9.0)).color(Color32::from_rgb(148, 163, 184))).on_hover_text("Move device earlier in processing chain").clicked()
                    {
                        dev_move_left = Some(state.selected_chain_idx);
                    }
                    if state.selected_chain_idx + 1 < state.chain_devices.len()
                        && ui.small_button(RichText::new("▶").font(FontId::proportional(9.0)).color(Color32::from_rgb(148, 163, 184))).on_hover_text("Move device later in processing chain").clicked()
                    {
                        dev_move_right = Some(state.selected_chain_idx);
                    }
                    if ui.small_button(RichText::new("✕").font(FontId::proportional(9.0)).color(Color32::from_rgb(239, 68, 68))).on_hover_text("Remove selected device from chain").clicked() {
                        dev_to_remove = Some(state.selected_chain_idx);
                    }
                }

                if let Some(idx) = dev_move_left {
                    state.move_device(idx, idx - 1);
                }
                if let Some(idx) = dev_move_right {
                    state.move_device(idx, idx + 1);
                }
                if let Some(idx) = dev_to_remove {
                    state.remove_device(idx);
                }

                ui.add_space(4.0);

                // [➕ Add Device] dropdown menu across all DSP categories
                let mut module_to_add: Option<String> = None;
                egui::ComboBox::from_id_source("add_device_to_chain_combo")
                    .selected_text(RichText::new("➕ Add Device ▾").font(FontId::proportional(9.5)).color(Color32::from_rgb(56, 189, 248)))
                    .show_ui(ui, |ui| {
                        for cat in [
                            crate::dsp_node_ui::DspCategory::Oscillator,
                            crate::dsp_node_ui::DspCategory::FilterEq,
                            crate::dsp_node_ui::DspCategory::Modulation,
                            crate::dsp_node_ui::DspCategory::DynamicsMaster,
                            crate::dsp_node_ui::DspCategory::DistortionSaturation,
                            crate::dsp_node_ui::DspCategory::TimeSpace,
                            crate::dsp_node_ui::DspCategory::SpatialSurround,
                            crate::dsp_node_ui::DspCategory::AcousticPhysicalModel,
                            crate::dsp_node_ui::DspCategory::SpectralResynthesis,
                            crate::dsp_node_ui::DspCategory::NeuralAi,
                            crate::dsp_node_ui::DspCategory::SamplerSlicer,
                        ] {
                            let cat_nodes = registry.list_by_category(cat);
                            if !cat_nodes.is_empty() {
                                ui.label(RichText::new(format!("{} {}", cat.icon(), cat.name())).font(FontId::proportional(9.0)).strong().color(Color32::from_rgb(148, 163, 184)));
                                for desc in cat_nodes.iter().take(8) {
                                    if ui.selectable_label(false, format!("  {} {}", desc.category.icon(), desc.display_name)).clicked() {
                                        module_to_add = Some(desc.kind_id.clone());
                                    }
                                }
                                ui.separator();
                            }
                        }
                    });

                if let Some(kind_id) = module_to_add {
                    if let Some(desc) = registry.get(&kind_id) {
                        state.add_device(&desc.kind_id, &desc.display_name);
                        state.node_param_values.clear();
                        for (p_i, schema) in desc.params.iter().enumerate() {
                            let def = match &schema.widget {
                                crate::dsp_node_ui::DspWidgetKind::RotaryKnob { default, .. }
                                | crate::dsp_node_ui::DspWidgetKind::VerticalFader { default, .. } => *default,
                                _ => 0.5,
                            };
                            state.node_param_values.insert(schema.id.clone(), def);
                            if let Some(bus) = param_bus {
                                let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 500 + p_i as u32);
                                if bus.get(pid).is_some() {
                                    bus.set(pid, def);
                                }
                            }
                        }
                    }
                }
            });

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            if !state.is_enabled {
                ui.centered_and_justified(|ui| {
                    ui.label(RichText::new("DEVICE BYPASSED — Click ⏻ to Enable").font(FontId::proportional(13.0)).color(Color32::from_rgb(148, 163, 184)));
                });
                return;
            }

            // Check if expanded Pro Parameters drawer is active
            if state.is_expanded_params {
                show_pro_parameter_drawer(ui, state, opt_desc, oscilloscope_data, cat_accent, param_bus, track_id);
                return;
            }

            // 2. Chassis Main Body (5 distinct modular sections)
            ui.horizontal(|ui| {
                // Section 1: Rotary Knobs Grid (2 rows x 3 cols)
                egui::Frame::none()
                    .fill(Color32::from_rgb(18, 24, 36))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 74)))
                    .rounding(Rounding::same(4.0))
                    .inner_margin(egui::Margin::symmetric(8.0, 6.0))
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            if let Some(desc) = opt_desc {
                                let knob_params: Vec<&crate::dsp_node_ui::DspParamSchema> = desc
                                    .params
                                    .iter()
                                    .filter(|p| matches!(p.widget, crate::dsp_node_ui::DspWidgetKind::RotaryKnob { .. } | crate::dsp_node_ui::DspWidgetKind::VerticalFader { .. }))
                                    .collect();

                                // Top Row: first 3 parameters
                                ui.horizontal(|ui| {
                                    for i in 0..3 {
                                        if let Some(&p) = knob_params.get(i) {
                                            let default_norm = match &p.widget {
                                                crate::dsp_node_ui::DspWidgetKind::RotaryKnob { min, max, default, .. } => {
                                                    ((*default - *min) / (*max - *min).max(1e-5)).clamp(0.0, 1.0)
                                                }
                                                crate::dsp_node_ui::DspWidgetKind::VerticalFader { min, max, default, .. } => {
                                                    ((*default - *min) / (*max - *min).max(1e-5)).clamp(0.0, 1.0)
                                                }
                                                _ => 0.5,
                                            };
                                            let mut val = *state.node_param_values.entry(p.id.clone()).or_insert(default_norm);
                                            let color = if i == 0 { Color32::from_rgb(56, 189, 248) } else { cat_accent };
                                            let resp = draw_rotary_dial(ui, &p.name, &mut val, color, default_norm);
                                            if resp.changed() || resp.dragged() || resp.double_clicked() {
                                                state.node_param_values.insert(p.id.clone(), val);
                                                if let Some(bus) = param_bus {
                                                    let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + i as u32);
                                                    if bus.get(pid).is_some() {
                                                        bus.set(pid, val);
                                                    }
                                                    let slot_pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + (state.selected_chain_idx as u32 * 20) + i as u32);
                                                    if bus.get(slot_pid).is_some() {
                                                        bus.set(slot_pid, val);
                                                    }
                                                }
                                            }
                                            if resp.secondary_clicked() {
                                                state.request_automation(&p.id);
                                            }
                                        } else {
                                            // Blank indicator for unoccupied slot
                                            let mut dummy = 0.5;
                                            draw_rotary_dial(ui, "---", &mut dummy, Color32::from_rgb(45, 55, 75), 0.5);
                                        }
                                        if i < 2 {
                                            ui.add_space(4.0);
                                        }
                                    }
                                });
                                ui.add_space(4.0);
                                // Bottom Row: next 3 parameters (indices 3..6)
                                ui.horizontal(|ui| {
                                    for i in 3..6 {
                                        if let Some(&p) = knob_params.get(i) {
                                            let default_norm = match &p.widget {
                                                crate::dsp_node_ui::DspWidgetKind::RotaryKnob { min, max, default, .. } => {
                                                    ((*default - *min) / (*max - *min).max(1e-5)).clamp(0.0, 1.0)
                                                }
                                                crate::dsp_node_ui::DspWidgetKind::VerticalFader { min, max, default, .. } => {
                                                    ((*default - *min) / (*max - *min).max(1e-5)).clamp(0.0, 1.0)
                                                }
                                                _ => 0.5,
                                            };
                                            let mut val = *state.node_param_values.entry(p.id.clone()).or_insert(default_norm);
                                            let color = if i == 3 { Color32::from_rgb(56, 189, 248) } else { cat_accent };
                                            let resp = draw_rotary_dial(ui, &p.name, &mut val, color, default_norm);
                                            if resp.changed() || resp.dragged() || resp.double_clicked() {
                                                state.node_param_values.insert(p.id.clone(), val);
                                                if let Some(bus) = param_bus {
                                                    let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + i as u32);
                                                    if bus.get(pid).is_some() {
                                                        bus.set(pid, val);
                                                    }
                                                    let slot_pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + (state.selected_chain_idx as u32 * 20) + i as u32);
                                                    if bus.get(slot_pid).is_some() {
                                                        bus.set(slot_pid, val);
                                                    }
                                                }
                                            }
                                            if resp.secondary_clicked() {
                                                state.request_automation(&p.id);
                                            }
                                        } else {
                                            let mut dummy = 0.5;
                                            draw_rotary_dial(ui, "---", &mut dummy, Color32::from_rgb(45, 55, 75), 0.5);
                                        }
                                        if i < 5 {
                                            ui.add_space(4.0);
                                        }
                                    }
                                });
                            } else {
                                // Default / Fallback knobs
                                ui.horizontal(|ui| {
                                    let r1 = draw_rotary_dial(ui, "Cutoff", &mut state.cutoff, Color32::from_rgb(56, 189, 248), 0.65);
                                    if r1.secondary_clicked() { state.requested_automation_param = Some("cutoff".to_string()); }
                                    if let Some(bus) = param_bus {
                                        if r1.changed() || r1.dragged() || r1.double_clicked() {
                                            let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000);
                                            if bus.get(pid).is_some() { bus.set(pid, state.cutoff); }
                                        }
                                    }
                                    ui.add_space(4.0);
                                    let r2 = draw_rotary_dial(ui, "Reso", &mut state.resonance, Color32::from_rgb(245, 158, 11), 0.45);
                                    if r2.secondary_clicked() { state.requested_automation_param = Some("resonance".to_string()); }
                                    if let Some(bus) = param_bus {
                                        if r2.changed() || r2.dragged() || r2.double_clicked() {
                                            let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 1);
                                            if bus.get(pid).is_some() { bus.set(pid, state.resonance); }
                                        }
                                    }
                                    ui.add_space(4.0);
                                    let r3 = draw_rotary_dial(ui, "Decay", &mut state.decay, Color32::from_rgb(245, 158, 11), 0.50);
                                    if r3.secondary_clicked() { state.requested_automation_param = Some("decay".to_string()); }
                                    if let Some(bus) = param_bus {
                                        if r3.changed() || r3.dragged() || r3.double_clicked() {
                                            let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 2);
                                            if bus.get(pid).is_some() { bus.set(pid, state.decay); }
                                        }
                                    }
                                });
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    let r4 = draw_rotary_dial(ui, "Amt", &mut state.env_decay, Color32::from_rgb(56, 189, 248), 0.35);
                                    if r4.secondary_clicked() { state.requested_automation_param = Some("env_decay".to_string()); }
                                    if let Some(bus) = param_bus {
                                        if r4.changed() || r4.dragged() || r4.double_clicked() {
                                            let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 3);
                                            if bus.get(pid).is_some() { bus.set(pid, state.env_decay); }
                                        }
                                    }
                                    ui.add_space(4.0);
                                    let r5 = draw_rotary_dial(ui, "Mod", &mut state.mod_amt, Color32::from_rgb(245, 158, 11), 0.60);
                                    if r5.secondary_clicked() { state.requested_automation_param = Some("mod_amt".to_string()); }
                                    if let Some(bus) = param_bus {
                                        if r5.changed() || r5.dragged() || r5.double_clicked() {
                                            let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 4);
                                            if bus.get(pid).is_some() { bus.set(pid, state.mod_amt); }
                                        }
                                    }
                                    ui.add_space(4.0);
                                    let r6 = draw_rotary_dial(ui, "Drive", &mut state.drive, Color32::from_rgb(245, 158, 11), 0.40);
                                    if r6.secondary_clicked() { state.requested_automation_param = Some("drive".to_string()); }
                                    if let Some(bus) = param_bus {
                                        if r6.changed() || r6.dragged() || r6.double_clicked() {
                                            let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 5);
                                            if bus.get(pid).is_some() { bus.set(pid, state.drive); }
                                        }
                                    }
                                });
                            }
                        });
                    });

                ui.add_space(6.0);

                // Section 2: Vertical Sliders (Osc Mix, Shape, Vol or dynamic reflection params 6..9)
                egui::Frame::none()
                    .fill(Color32::from_rgb(18, 24, 36))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 74)))
                    .rounding(Rounding::same(4.0))
                    .inner_margin(egui::Margin::symmetric(8.0, 6.0))
                    .show(ui, |ui| {
                        let fader_params: Vec<&crate::dsp_node_ui::DspParamSchema> = if let Some(desc) = opt_desc {
                            if desc.params.len() > 6 {
                                desc.params[6..].iter().take(3).collect()
                            } else {
                                Vec::new()
                            }
                        } else {
                            Vec::new()
                        };

                        if !fader_params.is_empty() {
                            ui.horizontal(|ui| {
                                for (i, p) in fader_params.iter().enumerate() {
                                    let default_norm = match &p.widget {
                                        crate::dsp_node_ui::DspWidgetKind::RotaryKnob { min, max, default, .. }
                                        | crate::dsp_node_ui::DspWidgetKind::VerticalFader { min, max, default, .. } => {
                                            ((*default - *min) / (*max - *min).max(1e-5)).clamp(0.0, 1.0)
                                        }
                                        _ => 0.5,
                                    };
                                    let mut val = *state.node_param_values.entry(p.id.clone()).or_insert(default_norm);
                                    let resp = draw_vertical_fader(ui, &p.name, &mut val, default_norm);
                                    if resp.changed() || resp.dragged() || resp.double_clicked() {
                                        state.node_param_values.insert(p.id.clone(), val);
                                        if let Some(bus) = param_bus {
                                            let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + (6 + i) as u32);
                                            if bus.get(pid).is_some() {
                                                bus.set(pid, val);
                                            }
                                            let slot_pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + (state.selected_chain_idx as u32 * 20) + (6 + i) as u32);
                                            if bus.get(slot_pid).is_some() {
                                                bus.set(slot_pid, val);
                                            }
                                        }
                                    }
                                    if resp.secondary_clicked() {
                                        state.request_automation(&p.id);
                                    }
                                    if i + 1 < fader_params.len() {
                                        ui.add_space(4.0);
                                    }
                                }
                                for i in fader_params.len()..3 {
                                    ui.add_space(4.0);
                                    if i == 1 {
                                        let resp = draw_vertical_fader(ui, "Shape", &mut state.shape, 0.50);
                                        if resp.secondary_clicked() { state.requested_automation_param = Some("shape".to_string()); }
                                        if let Some(bus) = param_bus {
                                            if resp.changed() || resp.dragged() || resp.double_clicked() {
                                                let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 7);
                                                if bus.get(pid).is_some() { bus.set(pid, state.shape); }
                                            }
                                        }
                                    } else {
                                        let resp = draw_vertical_fader(ui, "Vol", &mut state.volume, 0.85);
                                        if resp.secondary_clicked() { state.requested_automation_param = Some("volume".to_string()); }
                                        if let Some(bus) = param_bus {
                                            if resp.changed() || resp.dragged() || resp.double_clicked() {
                                                let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 200);
                                                if bus.get(pid).is_some() { bus.set(pid, state.volume); }
                                            }
                                        }
                                    }
                                }
                            });
                        } else {
                            ui.horizontal(|ui| {
                                let r1 = draw_vertical_fader(ui, "Osc Mix", &mut state.osc_mix, 0.75);
                                if r1.secondary_clicked() { state.requested_automation_param = Some("osc_mix".to_string()); }
                                if let Some(bus) = param_bus {
                                    if r1.changed() || r1.dragged() || r1.double_clicked() {
                                        let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 6);
                                        if bus.get(pid).is_some() { bus.set(pid, state.osc_mix); }
                                    }
                                }
                                ui.add_space(4.0);
                                let r2 = draw_vertical_fader(ui, "Shape", &mut state.shape, 0.50);
                                if r2.secondary_clicked() { state.requested_automation_param = Some("shape".to_string()); }
                                if let Some(bus) = param_bus {
                                    if r2.changed() || r2.dragged() || r2.double_clicked() {
                                        let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 7);
                                        if bus.get(pid).is_some() { bus.set(pid, state.shape); }
                                    }
                                }
                                ui.add_space(4.0);
                                let r3 = draw_vertical_fader(ui, "Vol", &mut state.volume, 0.85);
                                if r3.secondary_clicked() { state.requested_automation_param = Some("volume".to_string()); }
                                if let Some(bus) = param_bus {
                                    if r3.changed() || r3.dragged() || r3.double_clicked() {
                                        let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 200);
                                        if bus.get(pid).is_some() { bus.set(pid, state.volume); }
                                    }
                                }
                            });
                        }
                    });

                ui.add_space(6.0);

                // Section 3: Real-Time CRT Oscilloscope Screen
                egui::Frame::none()
                    .fill(Color32::from_rgb(8, 12, 20))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 74)))
                    .rounding(Rounding::same(4.0))
                    .inner_margin(egui::Margin::symmetric(6.0, 6.0))
                    .show(ui, |ui| {
                        ui.set_width(170.0);
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Oscilloscope").font(FontId::proportional(10.0)).color(Color32::from_rgb(148, 163, 184)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new("〰").font(FontId::proportional(10.0)).color(Color32::from_rgb(16, 185, 129)));
                                });
                            });
                            ui.add_space(2.0);

                            let (osc_resp, osc_painter) = ui.allocate_painter(Vec2::new(158.0, 96.0), egui::Sense::hover());
                            let osc_rect = osc_resp.rect;
                            osc_painter.rect_filled(osc_rect, 2.0, Color32::from_rgb(4, 7, 12));

                            // Draw subtle CRT grid
                            for g_y in 1..4 {
                                let gy = osc_rect.top() + (osc_rect.height() * (g_y as f32 / 4.0));
                                osc_painter.line_segment([egui::pos2(osc_rect.left(), gy), egui::pos2(osc_rect.right(), gy)], Stroke::new(0.5_f32, Color32::from_rgb(15, 25, 35)));
                            }

                            // Draw Neon Phosphor Green Wave
                            let points_count = 60;
                            let mut prev_pt = None;
                            for i in 0..points_count {
                                let norm_x = i as f32 / (points_count - 1) as f32;
                                let px = osc_rect.left() + norm_x * osc_rect.width();

                                let sample = if let Some(buf) = oscilloscope_data {
                                    let idx = (norm_x * (buf.len() as f32 - 1.0)) as usize;
                                    buf.get(idx).copied().unwrap_or(0.0)
                                } else {
                                    // Synthesize nice aesthetic wave
                                    (norm_x * 8.0 * std::f32::consts::PI).sin() * 0.45
                                        + (norm_x * 16.0 * std::f32::consts::PI).sin() * 0.25
                                };

                                let py = osc_rect.center().y - (sample * (osc_rect.height() * 0.42));
                                let current_pt = egui::pos2(px, py);

                                if let Some(last) = prev_pt {
                                    // Glow shadow
                                    osc_painter.line_segment([last, current_pt], Stroke::new(2.5_f32, Color32::from_rgba_unmultiplied(16, 185, 129, 60)));
                                    // Crisp center
                                    osc_painter.line_segment([last, current_pt], Stroke::new(1.2_f32, Color32::from_rgb(16, 185, 129)));
                                }
                                prev_pt = Some(current_pt);
                            }
                        });
                    });

                ui.add_space(6.0);

                // Section 4: Filter Frequency Curve Visualizer
                egui::Frame::none()
                    .fill(Color32::from_rgb(18, 24, 36))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 74)))
                    .rounding(Rounding::same(4.0))
                    .inner_margin(egui::Margin::symmetric(6.0, 6.0))
                    .show(ui, |ui| {
                        ui.set_width(170.0);
                        ui.vertical(|ui| {
                            ui.label(RichText::new("Filter Frequency").font(FontId::proportional(10.0)).color(Color32::from_rgb(148, 163, 184)));
                            ui.add_space(2.0);

                            let (filt_resp, filt_painter) = ui.allocate_painter(Vec2::new(158.0, 96.0), egui::Sense::click_and_drag());
                            let filt_rect = filt_resp.rect;
                            filt_painter.rect_filled(filt_rect, 2.0, Color32::from_rgb(8, 12, 20));

                            // Interactive drag handling for filter node 1 (cutoff)
                            if filt_resp.dragged() {
                                if let Some(pos) = filt_resp.interact_pointer_pos() {
                                    let nx = ((pos.x - filt_rect.left()) / filt_rect.width()).clamp(0.05, 0.95);
                                    let ny = (1.0 - ((pos.y - filt_rect.top()) / filt_rect.height())).clamp(0.05, 0.95);
                                    state.filter_nodes[1] = (nx, ny);
                                    state.cutoff = nx;
                                    state.resonance = ny;
                                    state.node_param_values.insert("cutoff".to_string(), nx);
                                    state.node_param_values.insert("resonance".to_string(), ny);
                                    if let Some(bus) = param_bus {
                                        let pid_cut = summoner_core::param_bus::ParamId(track_id as u32 * 1000);
                                        if bus.get(pid_cut).is_some() { bus.set(pid_cut, nx); }
                                        let pid_res = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 1);
                                        if bus.get(pid_res).is_some() { bus.set(pid_res, ny); }
                                    }
                                }
                            }

                            // Draw shaded filter transfer curve
                            let steps = 40;
                            let mut curve_pts = Vec::with_capacity(steps);
                            for i in 0..steps {
                                let t = i as f32 / (steps - 1) as f32;
                                let px = filt_rect.left() + t * filt_rect.width();

                                // 4-pole lowpass roll-off curve shape
                                let cutoff_norm = state.filter_nodes[1].0;
                                let q = state.filter_nodes[1].1;
                                let gain = if t <= cutoff_norm {
                                    1.0 + (t / cutoff_norm) * (q - 0.5) * 0.6
                                } else {
                                    let roll = (t - cutoff_norm) / (1.0 - cutoff_norm).max(0.01);
                                    (1.0 + (q - 0.5) * 0.6) * (-roll * 3.5).exp()
                                };

                                let py = filt_rect.bottom() - (gain * filt_rect.height() * 0.70).clamp(2.0, filt_rect.height() - 2.0);
                                curve_pts.push(egui::pos2(px, py));
                            }

                            for w in curve_pts.windows(2) {
                                filt_painter.line_segment([w[0], w[1]], Stroke::new(2.0_f32, Color32::from_rgb(56, 189, 248)));
                            }

                            // Draw draggable filter node pucks (44x44pt hit bounds)
                            for &(nx, ny) in &state.filter_nodes {
                                let px = filt_rect.left() + nx * filt_rect.width();
                                let py = filt_rect.top() + (1.0 - ny) * filt_rect.height();
                                filt_painter.circle_filled(egui::pos2(px, py), 4.5, Color32::from_rgb(56, 189, 248));
                                filt_painter.circle_stroke(egui::pos2(px, py), 5.5, Stroke::new(1.0_f32, Color32::WHITE));
                            }
                        });
                    });

                ui.add_space(6.0);

                // Section 5: LFO & Modulators / Schema Params 9..13
                egui::Frame::none()
                    .fill(Color32::from_rgb(18, 24, 36))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 74)))
                    .rounding(Rounding::same(4.0))
                    .inner_margin(egui::Margin::symmetric(8.0, 6.0))
                    .show(ui, |ui| {
                        let mod_params: Vec<&crate::dsp_node_ui::DspParamSchema> = if let Some(desc) = opt_desc {
                            if desc.params.len() > 9 {
                                desc.params[9..].iter().take(4).collect()
                            } else {
                                Vec::new()
                            }
                        } else {
                            Vec::new()
                        };

                        if !mod_params.is_empty() {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Aux Params").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(200, 215, 235)));
                                    ui.label(RichText::new("⚡").font(FontId::proportional(10.0)).color(cat_accent));
                                });
                                ui.add_space(2.0);
                                ui.horizontal(|ui| {
                                    for (i, p) in mod_params.iter().take(2).enumerate() {
                                        let default_norm = match &p.widget {
                                            crate::dsp_node_ui::DspWidgetKind::RotaryKnob { min, max, default, .. }
                                            | crate::dsp_node_ui::DspWidgetKind::VerticalFader { min, max, default, .. } => {
                                                ((*default - *min) / (*max - *min).max(1e-5)).clamp(0.0, 1.0)
                                            }
                                            _ => 0.5,
                                        };
                                        let mut val = *state.node_param_values.entry(p.id.clone()).or_insert(default_norm);
                                        let resp = draw_rotary_dial(ui, &p.name, &mut val, cat_accent, default_norm);
                                        if resp.changed() || resp.dragged() || resp.double_clicked() {
                                            state.node_param_values.insert(p.id.clone(), val);
                                            if let Some(bus) = param_bus {
                                                let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + (9 + i) as u32);
                                                if bus.get(pid).is_some() {
                                                    bus.set(pid, val);
                                                }
                                                let slot_pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + (state.selected_chain_idx as u32 * 20) + (9 + i) as u32);
                                                if bus.get(slot_pid).is_some() {
                                                    bus.set(slot_pid, val);
                                                }
                                            }
                                        }
                                        if resp.secondary_clicked() {
                                            state.request_automation(&p.id);
                                        }
                                        if i == 0 && mod_params.len() > 1 {
                                            ui.add_space(2.0);
                                        }
                                    }
                                });
                                ui.add_space(2.0);
                                ui.horizontal(|ui| {
                                    for (i, p) in mod_params.iter().skip(2).take(2).enumerate() {
                                        let default_norm = match &p.widget {
                                            crate::dsp_node_ui::DspWidgetKind::RotaryKnob { min, max, default, .. }
                                            | crate::dsp_node_ui::DspWidgetKind::VerticalFader { min, max, default, .. } => {
                                                ((*default - *min) / (*max - *min).max(1e-5)).clamp(0.0, 1.0)
                                            }
                                            _ => 0.5,
                                        };
                                        let mut val = *state.node_param_values.entry(p.id.clone()).or_insert(default_norm);
                                        let resp = draw_rotary_dial(ui, &p.name, &mut val, cat_accent, default_norm);
                                        if resp.changed() || resp.dragged() || resp.double_clicked() {
                                            state.node_param_values.insert(p.id.clone(), val);
                                            if let Some(bus) = param_bus {
                                                let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + (11 + i) as u32);
                                                if bus.get(pid).is_some() {
                                                    bus.set(pid, val);
                                                }
                                                let slot_pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + (state.selected_chain_idx as u32 * 20) + (11 + i) as u32);
                                                if bus.get(slot_pid).is_some() {
                                                    bus.set(slot_pid, val);
                                                }
                                            }
                                        }
                                        if resp.secondary_clicked() {
                                            state.request_automation(&p.id);
                                        }
                                        if i == 0 {
                                            ui.add_space(2.0);
                                        }
                                    }
                                });
                            });
                        } else {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("LFO").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(200, 215, 235)));
                                    ui.label(RichText::new("〰").font(FontId::proportional(10.0)).color(Color32::from_rgb(56, 189, 248)));
                                });
                                ui.add_space(2.0);
                                ui.horizontal(|ui| {
                                    let r1 = draw_rotary_dial(ui, "Rate", &mut state.lfo_speed, Color32::from_rgb(56, 189, 248), 0.40);
                                    if r1.secondary_clicked() { state.requested_automation_param = Some("lfo_speed".to_string()); }
                                    if let Some(bus) = param_bus {
                                        if r1.changed() || r1.dragged() || r1.double_clicked() {
                                            let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 8);
                                            if bus.get(pid).is_some() { bus.set(pid, state.lfo_speed); }
                                        }
                                    }
                                    let r2 = draw_rotary_dial(ui, "Depth", &mut state.lfo_depth, Color32::from_rgb(148, 163, 184), 0.60);
                                    if r2.secondary_clicked() { state.requested_automation_param = Some("lfo_depth".to_string()); }
                                    if let Some(bus) = param_bus {
                                        if r2.changed() || r2.dragged() || r2.double_clicked() {
                                            let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 9);
                                            if bus.get(pid).is_some() { bus.set(pid, state.lfo_depth); }
                                        }
                                    }
                                });
                                ui.add_space(2.0);
                                ui.horizontal(|ui| {
                                    let r3 = draw_rotary_dial(ui, "Shape", &mut state.shape, Color32::from_rgb(56, 189, 248), 0.50);
                                    if r3.secondary_clicked() { state.requested_automation_param = Some("shape".to_string()); }
                                    if let Some(bus) = param_bus {
                                        if r3.changed() || r3.dragged() || r3.double_clicked() {
                                            let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 7);
                                            if bus.get(pid).is_some() { bus.set(pid, state.shape); }
                                        }
                                    }
                                    let r4 = draw_rotary_dial(ui, "Mod", &mut state.mod_amt, Color32::from_rgb(148, 163, 184), 0.60);
                                    if r4.secondary_clicked() { state.requested_automation_param = Some("mod_amt".to_string()); }
                                    if let Some(bus) = param_bus {
                                        if r4.changed() || r4.dragged() || r4.double_clicked() {
                                            let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 4);
                                            if bus.get(pid).is_some() { bus.set(pid, state.mod_amt); }
                                        }
                                    }
                                });
                            });
                        }
                    });
            });
        });
}

#[cfg(feature = "gui")]
fn show_pro_parameter_drawer(
    ui: &mut egui::Ui,
    state: &mut ModernDeviceRackState,
    opt_desc: Option<&crate::dsp_node_ui::DspNodeDescriptor>,
    oscilloscope_data: Option<&[f32]>,
    cat_accent: Color32,
    param_bus: Option<&summoner_core::param_bus::ParamBus>,
    track_id: u64,
) {
    ui.horizontal(|ui| {
        // Left Visualizer Column: CRT Mini Oscilloscope + Filter Curve
        egui::Frame::none()
            .fill(Color32::from_rgb(10, 14, 24))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 74)))
            .rounding(Rounding::same(4.0))
            .inner_margin(egui::Margin::symmetric(6.0, 6.0))
            .show(ui, |ui| {
                ui.set_width(170.0);
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Scope & Curve").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(148, 163, 184)));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new("〰").font(FontId::proportional(10.0)).color(Color32::from_rgb(16, 185, 129)));
                        });
                    });
                    ui.add_space(2.0);

                    // CRT Mini Scope
                    let (osc_resp, osc_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::hover());
                    let osc_rect = osc_resp.rect;
                    osc_painter.rect_filled(osc_rect, 2.0, Color32::from_rgb(4, 7, 12));
                    for g_y in 1..3 {
                        let gy = osc_rect.top() + (osc_rect.height() * (g_y as f32 / 3.0));
                        osc_painter.line_segment([egui::pos2(osc_rect.left(), gy), egui::pos2(osc_rect.right(), gy)], Stroke::new(0.5_f32, Color32::from_rgb(15, 25, 35)));
                    }
                    let points_count = 50;
                    let mut prev_pt = None;
                    for i in 0..points_count {
                        let norm_x = i as f32 / (points_count - 1) as f32;
                        let px = osc_rect.left() + norm_x * osc_rect.width();
                        let sample = if let Some(buf) = oscilloscope_data {
                            let idx = (norm_x * (buf.len() as f32 - 1.0)) as usize;
                            buf.get(idx).copied().unwrap_or(0.0)
                        } else {
                            (norm_x * 8.0 * std::f32::consts::PI).sin() * 0.45
                        };
                        let py = osc_rect.center().y - (sample * (osc_rect.height() * 0.42));
                        let current_pt = egui::pos2(px, py);
                        if let Some(last) = prev_pt {
                            osc_painter.line_segment([last, current_pt], Stroke::new(1.5_f32, Color32::from_rgb(16, 185, 129)));
                        }
                        prev_pt = Some(current_pt);
                    }

                    ui.add_space(4.0);

                    let dev_kind = state.chain_devices.get(state.selected_chain_idx).map(|d| d.kind.as_str())
                        .or(state.selected_node_kind.as_deref())
                        .unwrap_or("AetherSynth");
                    let is_crystal = dev_kind == "CrystalResonator" || dev_kind.contains("Crystal") || dev_kind.contains("Bowl");
                    let is_armonica = dev_kind == "GlassArmonica" || dev_kind == "FranklinGlassArmonica" || dev_kind == "ArmonicaChassisResonator" || dev_kind.contains("Armonica");
                    let is_hurdy = dev_kind == "FrenchHurdyGurdy" || dev_kind.contains("HurdyGurdy") || dev_kind.contains("Vielle");
                    let is_trompette = dev_kind == "TrompetteChienBridge" || dev_kind == "TrompetteBridge" || dev_kind.contains("Trompette") || dev_kind.contains("Chien");
                    let is_hoa = dev_kind == "AmbisonicRadarSpatializer" || dev_kind == "HoaSpatializer" || dev_kind == "Hoa5BinauralSpatializer" || dev_kind == "Hoa5RadarView" || dev_kind.contains("Ambisonic") || dev_kind.contains("Hoa");
                    let is_neural = dev_kind.contains("Neural") || dev_kind.contains("Timbre") || dev_kind.contains("Latent") || dev_kind.contains("Resynthesizer") || dev_kind == "NeuralMorphOrbView" || dev_kind == "NeuralTimbreMorph" || dev_kind == "NeuralWavetable";

                    if is_crystal {
                        // Crystal Resonator 2D Acoustic Rim & Hydro-Acoustic Water Level Visualizer
                        let (c_resp, c_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let c_rect = c_resp.rect;
                        c_painter.rect_filled(c_rect, 2.0, Color32::from_rgb(6, 12, 22));

                        let mut water_fill = state.node_param_values.get("water_fill_level").copied().unwrap_or(0.25);
                        let mut friction = state.node_param_values.get("friction_velocity").copied().unwrap_or(0.50);

                        if c_resp.dragged() {
                            if let Some(pos) = c_resp.interact_pointer_pos() {
                                friction = ((pos.x - c_rect.left()) / c_rect.width()).clamp(0.0, 1.0);
                                water_fill = (1.0 - ((pos.y - c_rect.top()) / c_rect.height())).clamp(0.0, 1.0);
                                state.node_param_values.insert("water_fill_level".to_string(), water_fill);
                                state.node_param_values.insert("friction_velocity".to_string(), friction);
                                if let Some(bus) = param_bus {
                                    let pid_water = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_water).is_some() { bus.set(pid_water, water_fill); }
                                    let pid_fric = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 3);
                                    if bus.get(pid_fric).is_some() { bus.set(pid_fric, friction); }
                                }
                            }
                        }

                        // Draw singing bowl outer rim & inner cavity
                        let center = c_rect.center();
                        let radius = 26.0_f32;
                        c_painter.circle_stroke(center, radius, Stroke::new(1.5_f32, Color32::from_rgb(56, 189, 248)));
                        c_painter.circle_filled(center, radius * 0.88, Color32::from_rgb(10, 20, 36));

                        // Hydro-acoustic water fill meniscus line
                        let water_y = center.y + (1.0 - 2.0 * water_fill) * radius * 0.75;
                        let half_w = (radius * radius - (water_y - center.y).powi(2)).max(0.0).sqrt() * 0.85;
                        c_painter.line_segment(
                            [egui::pos2(center.x - half_w, water_y), egui::pos2(center.x + half_w, water_y)],
                            Stroke::new(1.5_f32, Color32::from_rgb(34, 211, 238)),
                        );

                        // Rotating wand friction puck on rim
                        let angle = friction * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
                        let puck_pos = egui::pos2(center.x + angle.cos() * radius, center.y + angle.sin() * radius);
                        c_painter.circle_filled(puck_pos, 3.5, Color32::from_rgb(245, 158, 11));

                        // Text labels: Water fill & Friction
                        c_painter.text(
                            egui::pos2(c_rect.left() + 4.0, c_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("💧 {:.0}%", water_fill * 100.0),
                            FontId::proportional(8.5),
                            Color32::from_rgb(34, 211, 238),
                        );
                        c_painter.text(
                            egui::pos2(c_rect.right() - 4.0, c_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("⚡ {:.0}%", friction * 100.0),
                            FontId::proportional(8.5),
                            Color32::from_rgb(245, 158, 11),
                        );

                        if c_resp.hovered() {
                            let _ = c_resp.on_hover_text("Crystal Singing Bowl Rim & Hydro-Acoustic Loading\n[Drag horizontally: Stick-slip friction | Drag vertically: Water fill level]");
                        }
                    } else if is_armonica {
                        // Glass Armonica Spindle Axis & Concentric Bowl Visualizer
                        let (a_resp, a_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let a_rect = a_resp.rect;
                        a_painter.rect_filled(a_rect, 2.0, Color32::from_rgb(18, 12, 8));

                        let mut speed = state.node_param_values.get("rotation_speed_rad_s").copied().unwrap_or(2.5);
                        let mut force = state.node_param_values.get("normal_force_n").copied().unwrap_or(0.45);

                        if a_resp.dragged() {
                            if let Some(pos) = a_resp.interact_pointer_pos() {
                                speed = 0.1 + ((pos.x - a_rect.left()) / a_rect.width()).clamp(0.0, 1.0) * 9.9;
                                force = 0.05 + (1.0 - ((pos.y - a_rect.top()) / a_rect.height())).clamp(0.0, 1.0) * 0.95;
                                state.node_param_values.insert("rotation_speed_rad_s".to_string(), speed);
                                state.node_param_values.insert("normal_force_n".to_string(), force);
                                if let Some(bus) = param_bus {
                                    let pid_spd = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_spd).is_some() { bus.set(pid_spd, speed); }
                                    let pid_frc = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_frc).is_some() { bus.set(pid_frc, force); }
                                }
                            }
                        }

                        // Central horizontal spindle axis
                        let cy = a_rect.center().y;
                        a_painter.line_segment(
                            [egui::pos2(a_rect.left() + 10.0, cy), egui::pos2(a_rect.right() - 10.0, cy)],
                            Stroke::new(2.0_f32, Color32::from_rgb(148, 163, 184)),
                        );

                        // 5 Nested concentric glass cup profiles
                        for (i, cup_r) in [12.0_f32, 16.0, 20.0, 24.0, 28.0].iter().enumerate() {
                            let cx = a_rect.left() + 25.0 + i as f32 * 26.0;
                            a_painter.line_segment(
                                [egui::pos2(cx, cy - cup_r), egui::pos2(cx, cy + cup_r)],
                                Stroke::new(1.5_f32, Color32::from_rgb(245, 158, 11)),
                            );
                        }

                        // Wet finger contact indicator
                        let contact_x = a_rect.left() + 25.0 + 3.0 * 26.0;
                        a_painter.circle_filled(egui::pos2(contact_x, cy - 24.0), 3.5, Color32::from_rgb(56, 189, 248));

                        // Readout
                        a_painter.text(
                            egui::pos2(a_rect.left() + 4.0, a_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("ω: {:.1} rad/s", speed),
                            FontId::proportional(8.5),
                            Color32::from_rgb(245, 158, 11),
                        );
                        a_painter.text(
                            egui::pos2(a_rect.right() - 4.0, a_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("F: {:.2} N", force),
                            FontId::proportional(8.5),
                            Color32::from_rgb(56, 189, 248),
                        );

                        if a_resp.hovered() {
                            let _ = a_resp.on_hover_text("Glass Armonica Spindle & Wet Friction Contact\n[Drag horizontally: Spindle angular velocity | Drag vertically: Normal force]");
                        }
                    } else if is_hurdy {
                        // Hurdy-Gurdy 2D Rosined Crank Wheel & Coup de Poignet Acceleration Visualizer
                        let (h_resp, h_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let h_rect = h_resp.rect;
                        h_painter.rect_filled(h_rect, 2.0, Color32::from_rgb(14, 18, 26));

                        let mut crank_speed = state.node_param_values.get("crank_speed_rad_s").copied().unwrap_or(2.0 * std::f32::consts::PI);
                        let mut wrist_accel = state.node_param_values.get("wrist_acceleration_pulse").copied().unwrap_or(1.2);

                        if h_resp.dragged() {
                            if let Some(pos) = h_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - h_rect.left()) / h_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - h_rect.top()) / h_rect.height())).clamp(0.0, 1.0);
                                wrist_accel = norm_x * 5.0;
                                crank_speed = norm_y * 4.0 * std::f32::consts::PI;
                                state.node_param_values.insert("crank_speed_rad_s".to_string(), crank_speed);
                                state.node_param_values.insert("wrist_acceleration_pulse".to_string(), wrist_accel);
                                if let Some(bus) = param_bus {
                                    let pid_speed = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_speed).is_some() { bus.set(pid_speed, crank_speed); }
                                    let pid_accel = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_accel).is_some() { bus.set(pid_accel, wrist_accel); }
                                }
                            }
                        }

                        // Background grid
                        for gx in 1..4 {
                            let px = h_rect.left() + h_rect.width() * (gx as f32 / 4.0);
                            h_painter.line_segment([egui::pos2(px, h_rect.top()), egui::pos2(px, h_rect.bottom())], Stroke::new(0.5_f32, Color32::from_rgb(26, 36, 48)));
                        }

                        // Wooden Rosined Wheel Circle Arc
                        let wheel_center = egui::pos2(h_rect.left() + 45.0, h_rect.center().y);
                        let wheel_r = 24.0;
                        h_painter.circle_stroke(wheel_center, wheel_r, Stroke::new(2.5_f32, Color32::from_rgb(180, 83, 9)));
                        h_painter.circle_stroke(wheel_center, wheel_r - 3.0, Stroke::new(1.0_f32, Color32::from_rgb(245, 158, 11)));

                        // Rotating crank indicator angle based on crank_speed
                        let crank_angle = (crank_speed * 1.5) % (2.0 * std::f32::consts::PI);
                        let crank_handle = egui::pos2(wheel_center.x + crank_angle.cos() * (wheel_r - 6.0), wheel_center.y + crank_angle.sin() * (wheel_r - 6.0));
                        h_painter.line_segment([wheel_center, crank_handle], Stroke::new(2.0_f32, Color32::from_rgb(251, 191, 36)));
                        h_painter.circle_filled(crank_handle, 3.5, Color32::from_rgb(234, 88, 12));

                        // Strings across wheel: Chanterelles & Bourdon Drone
                        let str_y1 = h_rect.center().y - 10.0;
                        let str_y2 = h_rect.center().y;
                        let str_y3 = h_rect.center().y + 10.0;
                        h_painter.line_segment([egui::pos2(h_rect.left() + 10.0, str_y1), egui::pos2(h_rect.right() - 10.0, str_y1)], Stroke::new(1.2_f32, Color32::from_rgb(56, 189, 248)));
                        h_painter.line_segment([egui::pos2(h_rect.left() + 10.0, str_y2), egui::pos2(h_rect.right() - 10.0, str_y2)], Stroke::new(1.2_f32, Color32::from_rgb(56, 189, 248)));
                        h_painter.line_segment([egui::pos2(h_rect.left() + 10.0, str_y3), egui::pos2(h_rect.right() - 10.0, str_y3)], Stroke::new(1.5_f32, Color32::from_rgb(16, 185, 129)));

                        // Coup de Poignet Wrist Impulse Puck
                        let puck_x = h_rect.left() + (wrist_accel / 5.0).clamp(0.0, 1.0) * h_rect.width();
                        let puck_y = h_rect.bottom() - (crank_speed / (4.0 * std::f32::consts::PI)).clamp(0.0, 1.0) * h_rect.height();
                        h_painter.circle_filled(egui::pos2(puck_x, puck_y), 5.0, Color32::from_rgb(239, 68, 68));
                        h_painter.circle_stroke(egui::pos2(puck_x, puck_y), 6.5, Stroke::new(1.0_f32, Color32::from_rgb(254, 202, 202)));

                        // Readout
                        h_painter.text(
                            egui::pos2(h_rect.left() + 4.0, h_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("ω: {:.1} rad/s", crank_speed),
                            FontId::proportional(8.5),
                            Color32::from_rgb(251, 191, 36),
                        );
                        h_painter.text(
                            egui::pos2(h_rect.right() - 4.0, h_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("Coup: {:.1}", wrist_accel),
                            FontId::proportional(8.5),
                            Color32::from_rgb(239, 68, 68),
                        );

                        if h_resp.hovered() {
                            let _ = h_resp.on_hover_text("Hurdy-Gurdy Rosined Crank Wheel & Coup de Poignet\n[Drag horizontally: Wrist impulse | Drag vertically: Crank speed]");
                        }
                    } else if is_trompette {
                        // Trompette Chien Buzzing Bridge 2D Collision Gap Visualizer
                        let (t_resp, t_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let t_rect = t_resp.rect;
                        t_painter.rect_filled(t_rect, 2.0, Color32::from_rgb(24, 12, 14));

                        let mut clearance_gap = state.node_param_values.get("chien_gap_clearance_mm").copied().unwrap_or(0.35);
                        let mut strike_force = state.node_param_values.get("strike_force").copied().unwrap_or(4.0);

                        if t_resp.dragged() {
                            if let Some(pos) = t_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - t_rect.left()) / t_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - t_rect.top()) / t_rect.height())).clamp(0.0, 1.0);
                                clearance_gap = 0.05 + norm_x * (1.20 - 0.05);
                                strike_force = norm_y * 10.0;
                                state.node_param_values.insert("chien_gap_clearance_mm".to_string(), clearance_gap);
                                state.node_param_values.insert("strike_force".to_string(), strike_force);
                                if let Some(bus) = param_bus {
                                    let pid_gap = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_gap).is_some() { bus.set(pid_gap, clearance_gap); }
                                    let pid_force = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_force).is_some() { bus.set(pid_force, strike_force); }
                                }
                            }
                        }

                        // Soundboard Bone Striking Plate (rigid obstacle barrier)
                        let plate_y = t_rect.bottom() - 14.0;
                        t_painter.line_segment([egui::pos2(t_rect.left() + 8.0, plate_y), egui::pos2(t_rect.right() - 8.0, plate_y)], Stroke::new(3.0_f32, Color32::from_rgb(226, 232, 240)));

                        // Resting Dog Foot Footprint with clearance gap
                        let gap_px = (clearance_gap / 1.20) * 16.0;
                        let dog_foot_y = plate_y - gap_px - 2.0;

                        // Rocking Chien (dog bridge) contour (asymmetric triangular bridge)
                        let pivot_pt = egui::pos2(t_rect.left() + 30.0, plate_y - 2.0);
                        let dog_foot_pt = egui::pos2(t_rect.right() - 40.0, dog_foot_y);
                        let string_notch_pt = egui::pos2(t_rect.right() - 48.0, dog_foot_y - 22.0);

                        t_painter.line_segment([pivot_pt, dog_foot_pt], Stroke::new(2.5_f32, Color32::from_rgb(239, 68, 68)));
                        t_painter.line_segment([dog_foot_pt, string_notch_pt], Stroke::new(2.5_f32, Color32::from_rgb(245, 158, 11)));
                        t_painter.line_segment([string_notch_pt, pivot_pt], Stroke::new(2.0_f32, Color32::from_rgb(239, 68, 68)));

                        // Trompette vibrating buzzing string passing over notch
                        t_painter.line_segment([egui::pos2(t_rect.left() + 5.0, string_notch_pt.y - 2.0), egui::pos2(t_rect.right() - 5.0, string_notch_pt.y + 4.0)], Stroke::new(1.8_f32, Color32::from_rgb(251, 191, 36)));

                        // Collision Impact Sparkles if striking force is high
                        if strike_force > 2.0 {
                            t_painter.circle_filled(egui::pos2(dog_foot_pt.x, plate_y), 2.5, Color32::from_rgb(254, 240, 138));
                        }

                        // Readout
                        t_painter.text(
                            egui::pos2(t_rect.left() + 4.0, t_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("Gap: {:.2} mm", clearance_gap),
                            FontId::proportional(8.5),
                            Color32::from_rgb(239, 68, 68),
                        );
                        t_painter.text(
                            egui::pos2(t_rect.right() - 4.0, t_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("Force: {:.1} N", strike_force),
                            FontId::proportional(8.5),
                            Color32::from_rgb(245, 158, 11),
                        );

                        if t_resp.hovered() {
                            let _ = t_resp.on_hover_text("Trompette Chien Buzzing Bridge Collision Gap\n[Drag horizontally: Clearance gap | Drag vertically: Coup strike force]");
                        }
                    } else if is_hoa {
                        // 5th-Order Ambisonics 2D Polar Radar Disk Visualizer
                        let (h_resp, h_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let h_rect = h_resp.rect;
                        h_painter.rect_filled(h_rect, 2.0, Color32::from_rgb(6, 10, 18));

                        let mut azimuth = state.node_param_values.get("azimuth_deg").or_else(|| state.node_param_values.get("yaw")).or_else(|| state.node_param_values.get("spherical_yaw")).copied().unwrap_or(45.0);
                        let mut elevation = state.node_param_values.get("elevation_deg").or_else(|| state.node_param_values.get("pitch")).or_else(|| state.node_param_values.get("spherical_pitch")).copied().unwrap_or(20.0);

                        if h_resp.dragged() {
                            if let Some(pos) = h_resp.interact_pointer_pos() {
                                let norm_x = ((pos.x - h_rect.left()) / h_rect.width()).clamp(0.0, 1.0);
                                let norm_y = (1.0 - ((pos.y - h_rect.top()) / h_rect.height())).clamp(0.0, 1.0);
                                azimuth = -180.0 + norm_x * 360.0;
                                elevation = -90.0 + norm_y * 180.0;
                                state.node_param_values.insert("azimuth_deg".to_string(), azimuth);
                                state.node_param_values.insert("elevation_deg".to_string(), elevation);
                                state.node_param_values.insert("yaw".to_string(), azimuth);
                                state.node_param_values.insert("pitch".to_string(), elevation);
                                state.node_param_values.insert("spherical_yaw".to_string(), azimuth);
                                state.node_param_values.insert("spherical_pitch".to_string(), elevation);
                                if let Some(bus) = param_bus {
                                    let pid_az = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_az).is_some() { bus.set(pid_az, azimuth); }
                                    let pid_el = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_el).is_some() { bus.set(pid_el, elevation); }
                                }
                            }
                        }

                        // Draw Radar Circle & Concentric Rings
                        let center = h_rect.center();
                        let radius = 28.0_f32;
                        h_painter.circle_filled(center, radius, Color32::from_rgb(8, 14, 26));
                        h_painter.circle_stroke(center, radius, Stroke::new(1.0_f32, Color32::from_rgb(14, 165, 233)));
                        h_painter.circle_stroke(center, radius * 0.6, Stroke::new(0.5_f32, Color32::from_rgb(30, 48, 75)));

                        // Compass crosshairs
                        h_painter.line_segment([egui::pos2(center.x - radius, center.y), egui::pos2(center.x + radius, center.y)], Stroke::new(0.5_f32, Color32::from_rgb(30, 48, 75)));
                        h_painter.line_segment([egui::pos2(center.x, center.y - radius), egui::pos2(center.x, center.y + radius)], Stroke::new(0.5_f32, Color32::from_rgb(30, 48, 75)));

                        // Puck position on radar disk
                        let az_rad = azimuth.to_radians();
                        let el_rad = elevation.to_radians();
                        let r_norm = (el_rad.cos() * 0.85).clamp(0.1, 1.0);
                        let puck_pos = egui::pos2(center.x + az_rad.sin() * radius * r_norm, center.y - az_rad.cos() * radius * r_norm);

                        // Wavefront ripple ring
                        h_painter.circle_stroke(puck_pos, 7.0, Stroke::new(1.0_f32, Color32::from_rgb(34, 211, 238)));
                        h_painter.circle_filled(puck_pos, 3.5, Color32::from_rgb(244, 63, 94));

                        // Readout
                        h_painter.text(
                            egui::pos2(h_rect.left() + 4.0, h_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("Az: {:+.0}°", azimuth),
                            FontId::proportional(8.5),
                            Color32::from_rgb(56, 189, 248),
                        );
                        h_painter.text(
                            egui::pos2(h_rect.right() - 4.0, h_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("El: {:+.0}°", elevation),
                            FontId::proportional(8.5),
                            Color32::from_rgb(34, 211, 238),
                        );

                        if h_resp.hovered() {
                            let _ = h_resp.on_hover_text("5th-Order Ambisonics (HOA5) 3D Radar Disk\n[Drag horizontally: Azimuth [-180°..+180°] | Drag vertically: Elevation [-90°..+90°]]");
                        }
                    } else if is_neural {
                        // 2D Neural Timbre Morphing Orb Mini Visualizer
                        let (n_resp, n_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let n_rect = n_resp.rect;
                        n_painter.rect_filled(n_rect, 2.0, Color32::from_rgb(8, 12, 22));

                        let mut morph_x = state.node_param_values.get("morph_x").or_else(|| state.node_param_values.get("timbre_x")).copied().unwrap_or(-0.35);
                        let mut morph_y = state.node_param_values.get("morph_y").or_else(|| state.node_param_values.get("timbre_y")).copied().unwrap_or(0.70);

                        if n_resp.dragged() {
                            if let Some(pos) = n_resp.interact_pointer_pos() {
                                let nx = ((pos.x - n_rect.left()) / n_rect.width()).clamp(0.0, 1.0);
                                let ny = (1.0 - ((pos.y - n_rect.top()) / n_rect.height())).clamp(0.0, 1.0);
                                morph_x = -1.0 + nx * 2.0;
                                morph_y = -1.0 + ny * 2.0;
                                state.node_param_values.insert("morph_x".to_string(), morph_x);
                                state.node_param_values.insert("morph_y".to_string(), morph_y);
                                state.node_param_values.insert("timbre_x".to_string(), morph_x);
                                state.node_param_values.insert("timbre_y".to_string(), morph_y);
                                if let Some(bus) = param_bus {
                                    let pid_x = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_x).is_some() { bus.set(pid_x, morph_x); }
                                    let pid_y = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_y).is_some() { bus.set(pid_y, morph_y); }
                                }
                            }
                        }

                        // Background Crosshairs
                        let center = n_rect.center();
                        n_painter.line_segment([egui::pos2(n_rect.left() + 10.0, center.y), egui::pos2(n_rect.right() - 10.0, center.y)], Stroke::new(0.5_f32, Color32::from_rgb(30, 48, 75)));
                        n_painter.line_segment([egui::pos2(center.x, n_rect.top() + 6.0), egui::pos2(center.x, n_rect.bottom() - 6.0)], Stroke::new(0.5_f32, Color32::from_rgb(30, 48, 75)));

                        // Corner Anchors
                        let anchors = [
                            (n_rect.left() + 18.0, n_rect.top() + 14.0, Color32::from_rgb(245, 158, 11)), // Acoustic (Amber)
                            (n_rect.right() - 18.0, n_rect.top() + 14.0, Color32::from_rgb(34, 211, 238)), // Metallic (Cyan)
                            (n_rect.right() - 18.0, n_rect.bottom() - 14.0, Color32::from_rgb(244, 63, 94)), // Cyber (Rose)
                            (n_rect.left() + 18.0, n_rect.bottom() - 14.0, Color32::from_rgb(168, 85, 247)), // Sub (Purple)
                        ];
                        for (ax, ay, col) in anchors {
                            n_painter.circle_filled(egui::pos2(ax, ay), 3.0, col);
                        }

                        // Puck Position
                        let half_w = n_rect.width() * 0.42;
                        let half_h = n_rect.height() * 0.38;
                        let puck_pos = egui::pos2(center.x + morph_x * half_w, center.y - morph_y * half_h);

                        // Lines from center to puck
                        n_painter.line_segment([center, puck_pos], Stroke::new(1.0_f32, Color32::from_rgb(244, 63, 94).gamma_multiply(0.5)));
                        n_painter.circle_stroke(puck_pos, 6.0, Stroke::new(1.0_f32, Color32::from_rgb(244, 63, 94).gamma_multiply(0.6)));
                        n_painter.circle_filled(puck_pos, 3.5, Color32::from_rgb(244, 63, 94));
                        n_painter.circle_filled(puck_pos, 1.5, Color32::WHITE);

                        // Readout
                        n_painter.text(
                            egui::pos2(n_rect.left() + 4.0, n_rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!("X: {:+.2}", morph_x),
                            FontId::proportional(8.5),
                            Color32::from_rgb(244, 63, 94),
                        );
                        n_painter.text(
                            egui::pos2(n_rect.right() - 4.0, n_rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            format!("Y: {:+.2}", morph_y),
                            FontId::proportional(8.5),
                            Color32::from_rgb(168, 85, 247),
                        );

                        if n_resp.hovered() {
                            let _ = n_resp.on_hover_text("2D Neural Timbre Morphing Orb\n[Drag horizontally: Latent X [-1.0..+1.0] | Drag vertically: Latent Y [-1.0..+1.0]]");
                        }
                    } else {
                        // Filter Curve Mini View (slot-addressed)
                        let (filt_resp, filt_painter) = ui.allocate_painter(Vec2::new(158.0, 68.0), egui::Sense::click_and_drag());
                        let filt_rect = filt_resp.rect;
                        filt_painter.rect_filled(filt_rect, 2.0, Color32::from_rgb(8, 12, 20));
                        if filt_resp.dragged() {
                            if let Some(pos) = filt_resp.interact_pointer_pos() {
                                let nx = ((pos.x - filt_rect.left()) / filt_rect.width()).clamp(0.05, 0.95);
                                let ny = (1.0 - ((pos.y - filt_rect.top()) / filt_rect.height())).clamp(0.05, 0.95);
                                state.filter_nodes[1] = (nx, ny);
                                state.cutoff = nx;
                                state.resonance = ny;
                                state.node_param_values.insert("cutoff".to_string(), nx);
                                state.node_param_values.insert("resonance".to_string(), ny);
                                if let Some(bus) = param_bus {
                                    let pid_cut = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20);
                                    if bus.get(pid_cut).is_some() { bus.set(pid_cut, nx); }
                                    let pid_res = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + 1);
                                    if bus.get(pid_res).is_some() { bus.set(pid_res, ny); }
                                }
                            }
                        }
                        let steps = 30;
                        let mut curve_pts = Vec::with_capacity(steps);
                        for i in 0..steps {
                            let t = i as f32 / (steps - 1) as f32;
                            let px = filt_rect.left() + t * filt_rect.width();
                            let cutoff_norm = state.filter_nodes[1].0;
                            let q = state.filter_nodes[1].1;
                            let gain = if t <= cutoff_norm {
                                1.0 + (t / cutoff_norm) * (q - 0.5) * 0.6
                            } else {
                                let roll = (t - cutoff_norm) / (1.0 - cutoff_norm).max(0.01);
                                (1.0 + (q - 0.5) * 0.6) * (-roll * 3.5).exp()
                            };
                            let py = filt_rect.bottom() - (gain * filt_rect.height() * 0.70).clamp(2.0, filt_rect.height() - 2.0);
                            curve_pts.push(egui::pos2(px, py));
                        }
                        for w in curve_pts.windows(2) {
                            filt_painter.line_segment([w[0], w[1]], Stroke::new(1.5_f32, Color32::from_rgb(56, 189, 248)));
                        }
                        for &(nx, ny) in &state.filter_nodes {
                            let px = filt_rect.left() + nx * filt_rect.width();
                            let py = filt_rect.top() + (1.0 - ny) * filt_rect.height();
                            filt_painter.circle_filled(egui::pos2(px, py), 3.5, Color32::from_rgb(56, 189, 248));
                        }
                    }
                });
            });

        ui.add_space(8.0);

        // Main Parameters Drawer (Scrollable Grid of Cards)
        egui::ScrollArea::horizontal()
            .id_source("pro_device_rack_scroll_area")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if let Some(desc) = opt_desc {
                    let chunks: Vec<&[crate::dsp_node_ui::DspParamSchema]> = desc.params.chunks(2).collect();
                    for (chunk_idx, chunk) in chunks.iter().enumerate() {
                        ui.vertical(|ui| {
                            ui.set_width(128.0);
                            for (item_idx, p) in chunk.iter().enumerate() {
                                let p_i = chunk_idx * 2 + item_idx;
                                egui::Frame::none()
                                    .fill(Color32::from_rgb(18, 24, 36))
                                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 74)))
                                    .rounding(Rounding::same(4.0))
                                    .inner_margin(egui::Margin::symmetric(6.0, 5.0))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            let name_lbl = ui.label(RichText::new(&p.name).font(FontId::proportional(9.5)).strong().color(Color32::from_rgb(226, 232, 240)));
                                            if name_lbl.secondary_clicked() {
                                                state.requested_automation_param = Some(p.id.clone());
                                            }
                                            if ui.button(RichText::new("📈").font(FontId::proportional(8.5)).color(Color32::from_rgb(234, 179, 8)))
                                                .on_hover_text(format!("Open Live Bézier Automation Lane for {}", p.name))
                                                .clicked()
                                            {
                                                state.requested_automation_param = Some(p.id.clone());
                                            }
                                            if p.macro_role != crate::dsp_node_ui::MacroRole::None {
                                                let (role_label, role_col) = match p.macro_role {
                                                    crate::dsp_node_ui::MacroRole::Tone => ("T", Color32::from_rgb(56, 189, 248)),
                                                    crate::dsp_node_ui::MacroRole::Space => ("S", Color32::from_rgb(99, 102, 241)),
                                                    crate::dsp_node_ui::MacroRole::Punch => ("P", Color32::from_rgb(239, 68, 68)),
                                                    crate::dsp_node_ui::MacroRole::Character => ("C", Color32::from_rgb(245, 158, 11)),
                                                    _ => ("", Color32::WHITE),
                                                };
                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    egui::Frame::none()
                                                        .fill(Color32::from_rgba_unmultiplied(role_col.r(), role_col.g(), role_col.b(), 40))
                                                        .rounding(Rounding::same(2.0))
                                                        .inner_margin(egui::Margin::symmetric(3.0, 1.0))
                                                        .show(ui, |ui| {
                                                            ui.label(RichText::new(role_label).font(FontId::proportional(8.0)).strong().color(role_col));
                                                        });
                                                });
                                            }
                                        });

                                        let default_norm = match &p.widget {
                                            crate::dsp_node_ui::DspWidgetKind::RotaryKnob { min, max, default, .. }
                                            | crate::dsp_node_ui::DspWidgetKind::VerticalFader { min, max, default, .. } => {
                                                ((*default - *min) / (*max - *min).max(1e-5)).clamp(0.0, 1.0)
                                            }
                                            crate::dsp_node_ui::DspWidgetKind::Toggle { default } => {
                                                if *default { 1.0 } else { 0.0 }
                                            }
                                            crate::dsp_node_ui::DspWidgetKind::EnumChoice { default_idx, .. } => {
                                                *default_idx as f32
                                            }
                                            _ => 0.5,
                                        };
                                        let mut val = *state.node_param_values.entry(p.id.clone()).or_insert(default_norm);
                                        let mut req_automation = false;

                                        match &p.widget {
                                            crate::dsp_node_ui::DspWidgetKind::RotaryKnob { min, max, unit, is_logarithmic, .. } => {
                                                let actual_val = *min + val * (*max - *min);
                                                ui.horizontal(|ui| {
                                                    let dial_resp = draw_rotary_dial(ui, "", &mut val, cat_accent, default_norm);
                                                    if dial_resp.secondary_clicked() {
                                                        req_automation = true;
                                                    }
                                                    ui.vertical(|ui| {
                                                        ui.label(RichText::new(format!("{:.1} {}", actual_val, unit)).font(FontId::proportional(8.5)).color(Color32::from_rgb(56, 189, 248)));
                                                        let slider = if *is_logarithmic {
                                                            egui::Slider::new(&mut val, 0.0..=1.0).logarithmic(true).show_value(false)
                                                        } else {
                                                            egui::Slider::new(&mut val, 0.0..=1.0).show_value(false)
                                                        };
                                                        let slider_resp = ui.add(slider);
                                                        if slider_resp.secondary_clicked() {
                                                            req_automation = true;
                                                        }
                                                        if slider_resp.double_clicked() {
                                                            val = default_norm;
                                                        }
                                                    });
                                                });
                                            }
                                            crate::dsp_node_ui::DspWidgetKind::VerticalFader { min, max, unit, .. } => {
                                                let actual_val = *min + val * (*max - *min);
                                                ui.horizontal(|ui| {
                                                    ui.label(RichText::new(format!("{:.1} {}", actual_val, unit)).font(FontId::proportional(8.5)).color(Color32::from_rgb(56, 189, 248)));
                                                    let slider_resp = ui.add(egui::Slider::new(&mut val, 0.0..=1.0).show_value(false));
                                                    if slider_resp.secondary_clicked() {
                                                        req_automation = true;
                                                    }
                                                    if slider_resp.double_clicked() {
                                                        val = default_norm;
                                                    }
                                                });
                                            }
                                            crate::dsp_node_ui::DspWidgetKind::Toggle { .. } => {
                                                let mut b = val >= 0.5;
                                                let tog_text = if b { "ON" } else { "OFF" };
                                                let tog_col = if b { Color32::from_rgb(34, 197, 94) } else { Color32::from_rgb(148, 163, 184) };
                                                let tog_btn = ui.button(RichText::new(tog_text).font(FontId::proportional(9.0)).color(tog_col));
                                                if tog_btn.clicked() {
                                                    b = !b;
                                                    val = if b { 1.0 } else { 0.0 };
                                                }
                                                if tog_btn.secondary_clicked() {
                                                    req_automation = true;
                                                }
                                            }
                                            crate::dsp_node_ui::DspWidgetKind::EnumChoice { choices, .. } => {
                                                let idx = (val as usize).min(choices.len().saturating_sub(1));
                                                let cur_choice = choices.get(idx).cloned().unwrap_or_default();
                                                egui::ComboBox::from_id_source(format!("pro_rack_combo_{}_{}", desc.kind_id, p.id))
                                                    .selected_text(RichText::new(&cur_choice).font(FontId::proportional(9.0)).color(cat_accent))
                                                    .show_ui(ui, |ui| {
                                                        for (c_i, choice) in choices.iter().enumerate() {
                                                            ui.selectable_value(&mut val, c_i as f32, choice);
                                                        }
                                                    });
                                            }
                                            _ => {
                                                let slider_resp = ui.add(egui::Slider::new(&mut val, 0.0..=1.0).show_value(false));
                                                if slider_resp.secondary_clicked() {
                                                    req_automation = true;
                                                }
                                                if slider_resp.double_clicked() {
                                                    val = default_norm;
                                                }
                                            }
                                        }

                                        state.node_param_values.insert(p.id.clone(), val);
                                        if req_automation {
                                            state.request_automation(&p.id);
                                        }

                                        if let Some(bus) = param_bus {
                                            let pid_pro = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 500 + p_i as u32);
                                            if bus.get(pid_pro).is_some() {
                                                bus.set(pid_pro, val);
                                            }
                                            let pid_multi = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + (state.selected_chain_idx as u32 * 20) + p_i as u32);
                                            if bus.get(pid_multi).is_some() {
                                                bus.set(pid_multi, val);
                                            }
                                            let pid_compat = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + p_i as u32);
                                            if bus.get(pid_compat).is_some() {
                                                bus.set(pid_compat, val);
                                            }
                                        }

                                        // Synchronize standard dial values if modified
                                        match p.id.as_str() {
                                            "cutoff" => {
                                                state.cutoff = val;
                                                if let Some(bus) = param_bus {
                                                    let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000);
                                                    if bus.get(pid).is_some() { bus.set(pid, val); }
                                                }
                                            }
                                            "resonance" => {
                                                state.resonance = val;
                                                if let Some(bus) = param_bus {
                                                    let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 1);
                                                    if bus.get(pid).is_some() { bus.set(pid, val); }
                                                }
                                            }
                                            "decay" => {
                                                state.decay = val;
                                                if let Some(bus) = param_bus {
                                                    let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 2);
                                                    if bus.get(pid).is_some() { bus.set(pid, val); }
                                                }
                                            }
                                            "env_decay" => {
                                                state.env_decay = val;
                                                if let Some(bus) = param_bus {
                                                    let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 3);
                                                    if bus.get(pid).is_some() { bus.set(pid, val); }
                                                }
                                            }
                                            "mod_amt" => {
                                                state.mod_amt = val;
                                                if let Some(bus) = param_bus {
                                                    let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 4);
                                                    if bus.get(pid).is_some() { bus.set(pid, val); }
                                                }
                                            }
                                            "drive" => {
                                                state.drive = val;
                                                if let Some(bus) = param_bus {
                                                    let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 5);
                                                    if bus.get(pid).is_some() { bus.set(pid, val); }
                                                }
                                            }
                                            "osc_mix" => {
                                                state.osc_mix = val;
                                                if let Some(bus) = param_bus {
                                                    let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 6);
                                                    if bus.get(pid).is_some() { bus.set(pid, val); }
                                                }
                                            }
                                            "shape" => {
                                                state.shape = val;
                                                if let Some(bus) = param_bus {
                                                    let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 7);
                                                    if bus.get(pid).is_some() { bus.set(pid, val); }
                                                }
                                            }
                                            "volume" => {
                                                state.volume = val;
                                                if let Some(bus) = param_bus {
                                                    let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 200);
                                                    if bus.get(pid).is_some() { bus.set(pid, val); }
                                                }
                                            }
                                            "lfo_speed" => {
                                                state.lfo_speed = val;
                                                if let Some(bus) = param_bus {
                                                    let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 8);
                                                    if bus.get(pid).is_some() { bus.set(pid, val); }
                                                }
                                            }
                                            "lfo_depth" => {
                                                state.lfo_depth = val;
                                                if let Some(bus) = param_bus {
                                                    let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 9);
                                                    if bus.get(pid).is_some() { bus.set(pid, val); }
                                                }
                                            }
                                            _ => {}
                                        }
                                    });
                                ui.add_space(4.0);
                            }
                        });
                        ui.add_space(4.0);
                    }
                } else {
                    ui.label(RichText::new("No descriptor for selected module").font(FontId::proportional(11.0)).color(Color32::from_rgb(148, 163, 184)));
                }
            });
    });
}

#[cfg(feature = "gui")]
fn draw_rotary_dial(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    ring_color: Color32,
    default_val: f32,
) -> egui::Response {
    let size = Vec2::new(38.0, 52.0);
    let (mut resp, painter) = ui.allocate_painter(size, egui::Sense::click_and_drag());
    let rect = resp.rect;

    if resp.dragged() {
        let delta_y = ui.input(|i| i.pointer.delta().y);
        *value = (*value - delta_y * 0.01).clamp(0.0, 1.0);
    }
    if resp.double_clicked() {
        *value = default_val.clamp(0.0, 1.0);
    }
    if resp.hovered() && !label.is_empty() && label != "---" {
        resp = resp.on_hover_text(format!(
            "{}: {:.0}%\n[Drag to adjust | Double-click to reset | Right-click to automate]",
            label,
            *value * 100.0
        ));
    }

    let center = egui::pos2(rect.center().x, rect.top() + 18.0);
    let radius = 14.0;

    // Background circle
    painter.circle_filled(center, radius, Color32::from_rgb(12, 16, 26));
    painter.circle_stroke(center, radius, Stroke::new(1.0_f32, Color32::from_rgb(28, 40, 60)));

    // Active arc ring (from -135 deg to +135 deg)
    let start_angle = -std::f32::consts::PI * 0.75;
    let end_angle = start_angle + (*value * std::f32::consts::PI * 1.5);

    let arc_steps = 16;
    let mut prev_arc = None;
    for i in 0..=arc_steps {
        let a = start_angle + (i as f32 / arc_steps as f32) * (end_angle - start_angle);
        let pt = egui::pos2(center.x + a.cos() * (radius - 1.5), center.y + a.sin() * (radius - 1.5));
        if let Some(last) = prev_arc {
            painter.line_segment([last, pt], Stroke::new(2.5_f32, ring_color));
        }
        prev_arc = Some(pt);
    }

    // Pointer notch indicator
    let notch_pt = egui::pos2(center.x + end_angle.cos() * (radius - 3.0), center.y + end_angle.sin() * (radius - 3.0));
    painter.line_segment([center, notch_pt], Stroke::new(1.5_f32, Color32::WHITE));

    // Label
    painter.text(
        egui::pos2(rect.center().x, rect.top() + 38.0),
        egui::Align2::CENTER_CENTER,
        label,
        FontId::proportional(9.0),
        Color32::from_rgb(148, 163, 184),
    );

    resp
}

#[cfg(feature = "gui")]
fn draw_vertical_fader(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    default_val: f32,
) -> egui::Response {
    let size = Vec2::new(28.0, 110.0);
    let (mut resp, painter) = ui.allocate_painter(size, egui::Sense::click_and_drag());
    let rect = resp.rect;

    if resp.dragged() {
        let delta_y = ui.input(|i| i.pointer.delta().y);
        *value = (*value - delta_y * 0.01).clamp(0.0, 1.0);
    }
    if resp.double_clicked() {
        *value = default_val.clamp(0.0, 1.0);
    }
    if resp.hovered() && !label.is_empty() {
        resp = resp.on_hover_text(format!(
            "{}: {:.1}\n[Drag to adjust | Double-click to reset | Right-click to automate]",
            label,
            *value * 10.0
        ));
    }

    let track_x = rect.center().x;
    let track_top = rect.top() + 18.0;
    let track_bottom = rect.bottom() - 16.0;

    // Fader slot track
    painter.line_segment(
        [egui::pos2(track_x, track_top), egui::pos2(track_x, track_bottom)],
        Stroke::new(2.0_f32, Color32::from_rgb(10, 14, 22)),
    );

    // Fader thumb handle (amber cap)
    let thumb_y = track_bottom - (*value * (track_bottom - track_top));
    let thumb_rect = Rect::from_center_size(egui::pos2(track_x, thumb_y), Vec2::new(18.0, 8.0));
    painter.rect_filled(thumb_rect, 2.0, Color32::from_rgb(245, 158, 11));
    painter.rect_stroke(thumb_rect, 2.0, Stroke::new(1.0_f32, Color32::WHITE));

    // Top Label
    painter.text(
        egui::pos2(rect.center().x, rect.top() + 6.0),
        egui::Align2::CENTER_CENTER,
        label,
        FontId::proportional(9.0),
        Color32::from_rgb(148, 163, 184),
    );

    // Bottom Value
    painter.text(
        egui::pos2(rect.center().x, rect.bottom() - 6.0),
        egui::Align2::CENTER_CENTER,
        format!("{:.1}", *value * 10.0),
        FontId::proportional(8.0),
        Color32::from_rgb(200, 215, 235),
    );

    resp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modern_device_rack_defaults() {
        let state = ModernDeviceRackState::default();
        assert!(state.is_enabled);
        assert_eq!(state.selected_node_kind.as_deref(), Some("AetherSynth"));
        assert_eq!(state.device_name, "Synth 1");
    }

    #[test]
    #[cfg(feature = "gui")]
    fn test_modern_device_rack_headless_rendering_dynamic_modules() {
        let mut state = ModernDeviceRackState::default();
        let ctx = egui::Context::default();

        let modules_to_test = [
            "AetherSynth",
            "DemucsV4Separator",
            "SingingSynthesisNode",
            "AiAutonomousMasteringEngine",
            "PluckedStringNode",
            "BowedString",
            "FilterSVF",
            "NavierStokesFluidNode",
        ];

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                for mod_kind in modules_to_test {
                    state.selected_node_kind = Some(mod_kind.to_string());
                    show_modern_device_rack(ui, &mut state, None);
                }
            });
        });

        assert!(!state.node_param_values.is_empty());
    }

    #[test]
    #[cfg(feature = "gui")]
    fn test_modern_device_rack_drawer_minimize_expand_toggle() {
        let mut state = ModernDeviceRackState::default();
        let ctx = egui::Context::default();

        // 1. Initial expanded state
        assert!(!state.is_minimized);
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_device_rack(ui, &mut state, None);
            });
        });

        // 2. Collapse to drawer mode
        state.is_minimized = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_device_rack(ui, &mut state, None);
            });
        });
        assert!(state.is_minimized);

        // 3. Expand back to full modular rack
        state.is_minimized = false;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_device_rack(ui, &mut state, None);
            });
        });
        assert!(!state.is_minimized);
    }

    #[test]
    #[cfg(feature = "gui")]
    fn test_modern_device_rack_dynamic_faders_and_aux_reflection() {
        let mut state = ModernDeviceRackState::default();
        let ctx = egui::Context::default();

        // Test with AI Mastering Engine (contains multiple parameters spanning faders & aux)
        state.selected_node_kind = Some("AiAutonomousMasteringEngine".to_string());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_device_rack(ui, &mut state, None);
            });
        });

        assert!(state.node_param_values.contains_key("target_lufs"));
        assert!(state.node_param_values.contains_key("ceiling_db"));
        assert!(state.node_param_values.contains_key("punch"));
    }

    #[test]
    #[cfg(feature = "gui")]
    fn test_modern_device_rack_expanded_pro_params_drawer() {
        let mut state = ModernDeviceRackState::default();
        let ctx = egui::Context::default();

        state.selected_node_kind = Some("AetherSynth".to_string());
        state.is_expanded_params = true;

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_device_rack(ui, &mut state, None);
            });
        });

        assert!(state.is_expanded_params);
        assert!(!state.node_param_values.is_empty());
        assert!(state.node_param_values.contains_key("cutoff"));
    }

    #[test]
    #[cfg(feature = "gui")]
    fn test_modern_device_rack_with_context_and_live_parambus() {
        let mut state = ModernDeviceRackState::default();
        let ctx = egui::Context::default();
        let mut bus = summoner_core::param_bus::ParamBus::new();
        bus.register(summoner_core::param_bus::ParamId(1000), 0.5);
        bus.register(summoner_core::param_bus::ParamId(1001), 0.5);

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_device_rack_with_context(ui, &mut state, None, Some(&bus), 1);
            });
        });

        assert_eq!(state.device_name, "Synth 1");
    }
}
