// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Modern Right Sidebar Inspector for Track/Clip Properties & Microtonal Tuning.

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, FontId, RichText, Rounding, Stroke};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModernInspectorState {
    pub target_name: String,
    #[serde(default)]
    pub selected_node_kind: Option<String>,
    #[serde(default)]
    pub node_param_values: std::collections::HashMap<String, f32>,
    pub gain_db: f32,
    pub pan_val: f32,
    pub is_muted: bool,
    pub is_soloed: bool,
    pub is_armed: bool,
    pub scale_name: String,
    pub scale_ratio_num: i32,
    pub scale_ratio_den: i32,
    pub root_ratio: f32,
    pub octave_offset: i32,
    pub is_collapsed: bool,
    #[serde(default)]
    pub requested_automation_param: Option<String>,
    #[serde(default)]
    pub chain_devices: Vec<crate::views::modern_device_rack::RackChainDeviceVisual>,
    #[serde(default)]
    pub selected_chain_idx: usize,
    #[serde(default)]
    pub phase_inverted: bool,
    #[serde(default)]
    pub input_trim_db: f32,
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
    #[serde(default)]
    pub requested_open_stems_export_modal: bool,
    #[serde(default)]
    pub requested_open_bowed_string_hud: bool,
    #[serde(default)]
    pub requested_open_shakuhachi_hud: bool,
    #[serde(default)]
    pub requested_open_sitar_hud: bool,
    #[serde(default)]
    pub requested_open_turkish_ney_hud: bool,
    #[serde(default)]
    pub requested_open_steelpan_hud: bool,
    #[serde(default)]
    pub requested_open_mbira_hud: bool,
    #[serde(default)]
    pub requested_open_koto_hud: bool,
    #[serde(default)]
    pub requested_open_gamelan_hud: bool,
    #[serde(default)]
    pub requested_open_dulcimer_hud: bool,
    #[serde(default)]
    pub requested_open_clavinet_hud: bool,
}

impl Default for ModernInspectorState {
    fn default() -> Self {
        Self {
            target_name: "Synth 1".to_string(),
            selected_node_kind: Some("AetherSynth".to_string()),
            node_param_values: std::collections::HashMap::new(),
            gain_db: 0.0,
            pan_val: 0.0,
            is_muted: false,
            is_soloed: false,
            is_armed: true,
            phase_inverted: false,
            input_trim_db: 0.0,
            scale_name: "A Minor Pentatonic".to_string(),
            scale_ratio_num: 1,
            scale_ratio_den: 1,
            root_ratio: 1.0,
            octave_offset: -1,
            is_collapsed: false,
            requested_automation_param: None,
            chain_devices: Vec::new(),
            selected_chain_idx: 0,
            requested_open_crystal_hud: false,
            requested_open_armonica_hud: false,
            requested_open_hurdy_gurdy_hud: false,
            requested_open_trompette_hud: false,
            requested_open_hoa5_radar_hud: false,
            requested_open_neural_morph_hud: false,
            requested_open_stems_export_modal: false,
            requested_open_bowed_string_hud: false,
            requested_open_shakuhachi_hud: false,
            requested_open_sitar_hud: false,
            requested_open_turkish_ney_hud: false,
            requested_open_steelpan_hud: false,
            requested_open_mbira_hud: false,
            requested_open_koto_hud: false,
            requested_open_gamelan_hud: false,
            requested_open_dulcimer_hud: false,
            requested_open_clavinet_hud: false,
        }
    }
}

impl ModernInspectorState {
    /// Ensure the chain has at least one device matching the currently selected node kind.
    pub fn ensure_chain(&mut self) {
        if self.chain_devices.is_empty() {
            let kind = self.selected_node_kind.clone().unwrap_or_else(|| "AetherSynth".to_string());
            let name = self.target_name.clone();
            self.chain_devices.push(crate::views::modern_device_rack::RackChainDeviceVisual {
                kind,
                display_name: name,
                is_bypassed: false,
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
            self.target_name = dev.display_name.clone();
            true
        } else {
            false
        }
    }

    /// Add a new DSP device module to the end of the track chain and select it.
    pub fn add_device(&mut self, kind: &str, display_name: &str) -> usize {
        self.ensure_chain();
        self.chain_devices.push(crate::views::modern_device_rack::RackChainDeviceVisual {
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
            true
        } else {
            false
        }
    }

    /// Get kind of active device.
    pub fn selected_device_kind(&self) -> Option<&str> {
        self.chain_devices.get(self.selected_chain_idx).map(|d| d.kind.as_str())
            .or(self.selected_node_kind.as_deref())
    }

    /// Get name of active device.
    pub fn selected_device_name(&self) -> Option<&str> {
        self.chain_devices.get(self.selected_chain_idx).map(|d| d.display_name.as_str())
            .or(Some(&self.target_name))
    }

    /// Check if active device is bypassed.
    pub fn is_selected_bypassed(&self) -> bool {
        self.chain_devices.get(self.selected_chain_idx).map(|d| d.is_bypassed).unwrap_or(false)
    }

    /// Request live parameter automation for a given parameter ID.
    pub fn request_automation(&mut self, param_id: &str) {
        if self.selected_chain_idx > 0 && !param_id.starts_with("node_") && !param_id.starts_with("master_node_") {
            self.requested_automation_param = Some(format!("node_{}_{}", self.selected_chain_idx, param_id));
        } else {
            self.requested_automation_param = Some(param_id.to_string());
        }
    }

    /// Reset track gain, pan, mute, solo, phase, and trim controls to unity / centered defaults.
    pub fn reset_mix_controls(&mut self) {
        self.gain_db = 0.0;
        self.pan_val = 0.0;
        self.is_muted = false;
        self.is_soloed = false;
        self.phase_inverted = false;
        self.input_trim_db = 0.0;
    }
}

#[cfg(feature = "gui")]
pub fn show_modern_inspector(ui: &mut egui::Ui, state: &mut ModernInspectorState) {
    show_modern_inspector_with_context(ui, state, None, 1);
}

#[cfg(feature = "gui")]
pub fn show_modern_inspector_with_context(
    ui: &mut egui::Ui,
    state: &mut ModernInspectorState,
    param_bus: Option<&summoner_core::param_bus::ParamBus>,
    track_id: u64,
) {
    if state.is_collapsed {
        if ui.button("◀").clicked() {
            state.is_collapsed = false;
        }
        return;
    }

    let panel_w = 240.0;
    egui::Frame::none()
        .fill(Color32::from_rgb(14, 20, 32))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(28, 40, 60)))
        .rounding(Rounding::same(6.0))
        .inner_margin(egui::Margin::symmetric(10.0, 10.0))
        .show(ui, |ui| {
            ui.set_width(panel_w);
            ui.set_height(ui.available_height());

            egui::ScrollArea::vertical()
                .id_source("modern_inspector_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    // Header
            ui.horizontal(|ui| {
                ui.label(RichText::new("Inspector").font(FontId::proportional(12.0)).strong().color(Color32::from_rgb(241, 245, 249)));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("▶").clicked() {
                        state.is_collapsed = true;
                    }
                });
            });

            ui.add_space(8.0);

            // Active Track Target Box
            egui::Frame::none()
                .fill(Color32::from_rgb(20, 28, 44))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 74)))
                .rounding(Rounding::same(4.0))
                .inner_margin(egui::Margin::symmetric(8.0, 6.0))
                .show(ui, |ui| {
                    ui.label(RichText::new(&state.target_name).font(FontId::proportional(12.0)).strong().color(Color32::from_rgb(56, 189, 248)));
                });

            ui.add_space(12.0);

            // Properties Section
            ui.label(RichText::new("Properties").font(FontId::proportional(11.0)).strong().color(Color32::from_rgb(200, 215, 235)));
            ui.add_space(4.0);

            // Gain Fader
            ui.horizontal(|ui| {
                ui.label(RichText::new("Gain Fader").font(FontId::proportional(10.0)).color(Color32::from_rgb(148, 163, 184)));
                if ui.small_button(RichText::new("📈").font(FontId::proportional(9.0))).on_hover_text("Open Live Bézier Automation Lane for Volume Gain").clicked() {
                    state.requested_automation_param = Some("gain".to_string());
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(format!("{:.1}dB", state.gain_db)).font(FontId::proportional(10.0)).color(Color32::from_rgb(56, 189, 248)));
                });
            });
            let gain_resp = ui.add(egui::Slider::new(&mut state.gain_db, -36.0..=12.0).show_value(false));
            if gain_resp.double_clicked() {
                state.gain_db = 0.0;
            }
            if gain_resp.secondary_clicked() {
                state.requested_automation_param = Some("gain".to_string());
            }
            if gain_resp.changed() || gain_resp.double_clicked() {
                if let Some(bus) = param_bus {
                    let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 200);
                    let gain_lin = ((state.gain_db / 12.0) + 1.0).clamp(0.0, 2.0);
                    if bus.get(pid).is_some() {
                        bus.set(pid, gain_lin);
                    }
                }
            }
            if gain_resp.hovered() {
                let _ = gain_resp.on_hover_text(format!(
                    "Gain Fader: {:.1} dB\n[Drag to adjust | Double-click for 0dB unity | Right-click to automate]",
                    state.gain_db
                ));
            }

            ui.add_space(6.0);

            // Pan Fader
            ui.horizontal(|ui| {
                ui.label(RichText::new("Pan").font(FontId::proportional(10.0)).color(Color32::from_rgb(148, 163, 184)));
                if ui.small_button(RichText::new("📈").font(FontId::proportional(9.0))).on_hover_text("Open Live Bézier Automation Lane for Stereo Pan").clicked() {
                    state.requested_automation_param = Some("pan".to_string());
                }
                let pan_text = if state.pan_val.abs() < 0.05 {
                    "C".to_string()
                } else if state.pan_val < 0.0 {
                    format!("L {:.0}%", state.pan_val.abs() * 100.0)
                } else {
                    format!("R {:.0}%", state.pan_val * 100.0)
                };
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(pan_text).font(FontId::proportional(10.0)).color(Color32::from_rgb(56, 189, 248)));
                });
            });
            let pan_resp = ui.add(egui::Slider::new(&mut state.pan_val, -1.0..=1.0).show_value(false));
            if pan_resp.double_clicked() {
                state.pan_val = 0.0;
            }
            if pan_resp.secondary_clicked() {
                state.requested_automation_param = Some("pan".to_string());
            }
            if pan_resp.changed() || pan_resp.double_clicked() {
                if let Some(bus) = param_bus {
                    let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 201);
                    if bus.get(pid).is_some() {
                        bus.set(pid, state.pan_val);
                    }
                }
            }
            if pan_resp.hovered() {
                let pan_display = if state.pan_val.abs() < 0.05 {
                    "C".to_string()
                } else if state.pan_val < 0.0 {
                    format!("L {:.0}%", state.pan_val.abs() * 100.0)
                } else {
                    format!("R {:.0}%", state.pan_val * 100.0)
                };
                let _ = pan_resp.on_hover_text(format!(
                    "Stereo Pan: {}\n[Drag to adjust | Double-click to center | Right-click to automate]",
                    pan_display
                ));
            }

            ui.add_space(8.0);

            // Quick Mute, Solo, Arm Buttons
            ui.horizontal(|ui| {
                let mute_btn = ui.add(
                    egui::Button::new(RichText::new("Mute").font(FontId::proportional(11.0)).color(if state.is_muted { Color32::from_rgb(234, 179, 8) } else { Color32::from_rgb(148, 163, 184) }))
                        .fill(if state.is_muted { Color32::from_rgb(45, 38, 15) } else { Color32::from_rgb(24, 34, 52) })
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 74)))
                        .rounding(Rounding::same(4.0))
                );
                if mute_btn.clicked() {
                    state.is_muted = !state.is_muted;
                    if let Some(bus) = param_bus {
                        let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 202);
                        if bus.get(pid).is_some() {
                            bus.set(pid, if state.is_muted { 1.0 } else { 0.0 });
                        }
                    }
                }
                if mute_btn.secondary_clicked() {
                    state.requested_automation_param = Some("mute".to_string());
                }

                let solo_btn = ui.add(
                    egui::Button::new(RichText::new("Solo").font(FontId::proportional(11.0)).color(if state.is_soloed { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(148, 163, 184) }))
                        .fill(if state.is_soloed { Color32::from_rgba_unmultiplied(56, 189, 248, 30) } else { Color32::from_rgb(24, 34, 52) })
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 74)))
                        .rounding(Rounding::same(4.0))
                );
                if solo_btn.clicked() {
                    state.is_soloed = !state.is_soloed;
                    if let Some(bus) = param_bus {
                        let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 203);
                        if bus.get(pid).is_some() {
                            bus.set(pid, if state.is_soloed { 1.0 } else { 0.0 });
                        }
                    }
                }
                if solo_btn.secondary_clicked() {
                    state.requested_automation_param = Some("solo".to_string());
                }

                let arm_btn = ui.add(
                    egui::Button::new(RichText::new("Arm").font(FontId::proportional(11.0)).strong().color(if state.is_armed { Color32::from_rgb(239, 68, 68) } else { Color32::from_rgb(148, 163, 184) }))
                        .fill(if state.is_armed { Color32::from_rgb(50, 20, 25) } else { Color32::from_rgb(24, 34, 52) })
                        .stroke(Stroke::new(1.0_f32, if state.is_armed { Color32::from_rgb(239, 68, 68) } else { Color32::from_rgb(36, 50, 74) }))
                        .rounding(Rounding::same(4.0))
                );
                if arm_btn.clicked() {
                    state.is_armed = !state.is_armed;
                }
            });

            ui.add_space(8.0);

            // Phase Invert ([Ø]) & Input Trim (-18.0 .. +18.0 dB)
            ui.horizontal(|ui| {
                ui.label(RichText::new("Input Trim").font(FontId::proportional(10.0)).color(Color32::from_rgb(148, 163, 184)));
                if ui.small_button(RichText::new("📈").font(FontId::proportional(9.0))).on_hover_text("Open Live Bézier Automation Lane for Input Trim").clicked() {
                    state.requested_automation_param = Some("trim".to_string());
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(format!("{:.1}dB", state.input_trim_db)).font(FontId::proportional(10.0)).color(Color32::from_rgb(56, 189, 248)));
                });
            });
            let trim_resp = ui.add(egui::Slider::new(&mut state.input_trim_db, -18.0..=18.0).show_value(false));
            if trim_resp.double_clicked() {
                state.input_trim_db = 0.0;
            }
            if trim_resp.secondary_clicked() {
                state.requested_automation_param = Some("trim".to_string());
            }
            if trim_resp.changed() || trim_resp.double_clicked() {
                if let Some(bus) = param_bus {
                    let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 205);
                    if bus.get(pid).is_some() {
                        bus.set(pid, state.input_trim_db);
                    }
                }
            }
            if trim_resp.hovered() {
                let _ = trim_resp.on_hover_text(format!(
                    "Input Trim: {:.1} dB\n[Drag to adjust | Double-click for 0.0dB | Right-click to automate]",
                    state.input_trim_db
                ));
            }

            ui.add_space(6.0);

            ui.horizontal(|ui| {
                let phase_col = if state.phase_inverted { Color32::from_rgb(245, 158, 11) } else { Color32::from_rgb(148, 163, 184) };
                let phase_bg = if state.phase_inverted { Color32::from_rgba_unmultiplied(245, 158, 11, 40) } else { Color32::from_rgb(24, 34, 52) };
                let phase_btn = ui.add(
                    egui::Button::new(RichText::new(if state.phase_inverted { "Ø Phase Inverted" } else { "Ø Phase Normal" }).font(FontId::proportional(10.0)).color(phase_col))
                        .fill(phase_bg)
                        .stroke(Stroke::new(1.0_f32, if state.phase_inverted { Color32::from_rgb(245, 158, 11) } else { Color32::from_rgb(36, 50, 74) }))
                        .rounding(Rounding::same(4.0))
                );
                if phase_btn.clicked() {
                    state.phase_inverted = !state.phase_inverted;
                    if let Some(bus) = param_bus {
                        let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 204);
                        if bus.get(pid).is_some() {
                            bus.set(pid, if state.phase_inverted { 1.0 } else { 0.0 });
                        }
                    }
                }
                if phase_btn.secondary_clicked() {
                    state.requested_automation_param = Some("phase".to_string());
                }
                let _ = phase_btn.on_hover_text("Toggle Audio Polarity Inversion (180° phase flip)\n[Click to toggle | Right-click to automate]");
            });

            ui.add_space(14.0);
            ui.separator();
            ui.add_space(6.0);

            // Microtonal Tuning Section
            ui.label(RichText::new("Microtonal Tuning").font(FontId::proportional(11.0)).strong().color(Color32::from_rgb(200, 215, 235)));
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.label(RichText::new("Scale").font(FontId::proportional(10.0)).color(Color32::from_rgb(148, 163, 184)));
                egui::ComboBox::from_id_source("inspector_scale_combo")
                    .selected_text(RichText::new(&state.scale_name).font(FontId::proportional(10.0)).color(Color32::from_rgb(56, 189, 248)))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut state.scale_name, "A Minor Pentatonic".to_string(), "A Minor Pentatonic");
                        ui.selectable_value(&mut state.scale_name, "19-EDO Equal".to_string(), "19-EDO Equal");
                        ui.selectable_value(&mut state.scale_name, "31-EDO Fokker".to_string(), "31-EDO Fokker");
                        ui.selectable_value(&mut state.scale_name, "Bohlen-Pierce".to_string(), "Bohlen-Pierce");
                        ui.selectable_value(&mut state.scale_name, "Just Intonation 5-Limit".to_string(), "Just Intonation 5-Limit");
                    });
            });

            ui.add_space(8.0);

            // Ratio Grid
            ui.horizontal(|ui| {
                egui::Frame::none()
                    .fill(Color32::from_rgb(20, 28, 44))
                    .rounding(Rounding::same(4.0))
                    .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new("Ratio").font(FontId::proportional(9.0)).color(Color32::from_rgb(100, 116, 139)));
                            ui.label(RichText::new(format!("{}:{}", state.scale_ratio_num, state.scale_ratio_den)).font(FontId::proportional(10.0)).color(Color32::from_rgb(241, 245, 249)));
                        });
                    });

                egui::Frame::none()
                    .fill(Color32::from_rgb(20, 28, 44))
                    .rounding(Rounding::same(4.0))
                    .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new("Ratio").font(FontId::proportional(9.0)).color(Color32::from_rgb(100, 116, 139)));
                            ui.label(RichText::new(format!("{:.0}", state.root_ratio)).font(FontId::proportional(10.0)).color(Color32::from_rgb(241, 245, 249)));
                        });
                    });

                egui::Frame::none()
                    .fill(Color32::from_rgb(20, 28, 44))
                    .rounding(Rounding::same(4.0))
                    .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new("Octave").font(FontId::proportional(9.0)).color(Color32::from_rgb(100, 116, 139)));
                            ui.label(RichText::new(format!("{}", state.octave_offset)).font(FontId::proportional(10.0)).color(Color32::from_rgb(241, 245, 249)));
                        });
                    });
            });

            // Surgical DSP Node Inspector Section
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(6.0);

            state.ensure_chain();
            let registry = crate::dsp_node_ui::DspNodeRegistry::new();

            ui.horizontal(|ui| {
                ui.label(RichText::new("DSP Parameter Inspector").font(FontId::proportional(11.0)).strong().color(Color32::from_rgb(200, 215, 235)));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(RichText::new("📈 Auto").font(FontId::proportional(9.0)).color(Color32::from_rgb(234, 179, 8)))
                        .on_hover_text("Open Live Bézier Automation Lane for current DSP module primary parameter")
                        .clicked()
                    {
                        if let Some(ref node_kind) = state.selected_node_kind {
                            if let Some(desc) = registry.get(node_kind) {
                                if let Some(primary) = desc.params.first() {
                                    state.request_automation(&primary.id);
                                }
                            }
                        }
                    }

                    let is_bypassed = state.is_selected_bypassed();
                    let byp_text = if is_bypassed { "BYPASS" } else { "ACTIVE" };
                    let byp_col = if is_bypassed { Color32::from_rgb(239, 68, 68) } else { Color32::from_rgb(34, 197, 94) };
                    if ui.small_button(RichText::new(byp_text).font(FontId::proportional(8.5)).color(byp_col))
                        .on_hover_text("Toggle bypass state of inspected DSP device")
                        .clicked()
                    {
                        state.toggle_device_bypass(state.selected_chain_idx);
                    }
                });
            });
            ui.add_space(6.0);

            // Specialized Physical Modeling Instrument HUD Launchers
            let active_kind = state.selected_device_kind().unwrap_or_default();
            if active_kind == "CrystalResonator" || active_kind.contains("Crystal") || active_kind.contains("Bowl") {
                if ui.add(
                    egui::Button::new(RichText::new("🔮 Open Crystal Resonator HUD").font(FontId::proportional(10.5)).strong().color(Color32::from_rgb(56, 189, 248)))
                        .fill(Color32::from_rgb(18, 30, 48))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(56, 189, 248)))
                        .rounding(Rounding::same(4.0))
                ).on_hover_text("Open interactive 2D stick-slip friction radar, hydro-acoustic water level & modal resonance doublets HUD").clicked() {
                    state.requested_open_crystal_hud = true;
                }
                ui.add_space(4.0);
            } else if active_kind == "GlassArmonica" || active_kind == "FranklinGlassArmonica" || active_kind == "ArmonicaChassisResonator" || active_kind.contains("Armonica") {
                if ui.add(
                    egui::Button::new(RichText::new("🍷 Open Glass Armonica HUD").font(FontId::proportional(10.5)).strong().color(Color32::from_rgb(245, 158, 11)))
                        .fill(Color32::from_rgb(38, 28, 16))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(245, 158, 11)))
                        .rounding(Rounding::same(4.0))
                ).on_hover_text("Open interactive 2D spindle speed, wet finger friction & circular modal ring HUD").clicked() {
                    state.requested_open_armonica_hud = true;
                }
                ui.add_space(4.0);
            } else if active_kind == "FrenchHurdyGurdy" || active_kind.contains("HurdyGurdy") || active_kind.contains("Vielle") {
                if ui.add(
                    egui::Button::new(RichText::new("🎻 Open Hurdy-Gurdy HUD").font(FontId::proportional(10.5)).strong().color(Color32::from_rgb(16, 185, 129)))
                        .fill(Color32::from_rgb(16, 36, 28))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(16, 185, 129)))
                        .rounding(Rounding::same(4.0))
                ).on_hover_text("Open interactive rosined crank wheel speed, coup de poignet pulse & bourdon drone HUD").clicked() {
                    state.requested_open_hurdy_gurdy_hud = true;
                }
                ui.add_space(4.0);
            } else if active_kind == "TrompetteChienBridge" || active_kind == "TrompetteBridge" || active_kind.contains("Trompette") || active_kind.contains("Chien") {
                if ui.add(
                    egui::Button::new(RichText::new("🐕 Open Trompette Chien Bridge HUD").font(FontId::proportional(10.5)).strong().color(Color32::from_rgb(239, 68, 68)))
                        .fill(Color32::from_rgb(42, 18, 20))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(239, 68, 68)))
                        .rounding(Rounding::same(4.0))
                ).on_hover_text("Open interactive chattering chien buzzing dog bridge obstacle collision phase canvas").clicked() {
                    state.requested_open_trompette_hud = true;
                }
                ui.add_space(4.0);
            } else if active_kind == "AmbisonicRadarSpatializer" || active_kind == "HoaSpatializer" || active_kind == "Hoa5BinauralSpatializer" || active_kind == "Hoa5RadarView" || active_kind.contains("Ambisonic") || active_kind.contains("Hoa") {
                if ui.add(
                    egui::Button::new(RichText::new("🌐 Open 5th-Order Ambisonic Radar HUD").font(FontId::proportional(10.5)).strong().color(Color32::from_rgb(56, 189, 248)))
                        .fill(Color32::from_rgb(14, 28, 48))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(56, 189, 248)))
                        .rounding(Rounding::same(4.0))
                ).on_hover_text("Open interactive 3D spherical elevation/azimuth polar projection radar canvas & wavefront ripples HUD").clicked() {
                    state.requested_open_hoa5_radar_hud = true;
                }
                ui.add_space(4.0);
            } else if active_kind.contains("Neural") || active_kind.contains("Timbre") || active_kind.contains("Latent") || active_kind.contains("Resynthesizer") || active_kind == "NeuralMorphOrbView" || active_kind == "NeuralTimbreMorph" || active_kind == "NeuralWavetable" {
                if ui.add(
                    egui::Button::new(RichText::new("🧬 Open 2D Neural Timbre Morph HUD").font(FontId::proportional(10.5)).strong().color(Color32::from_rgb(244, 63, 94)))
                        .fill(Color32::from_rgb(38, 16, 28))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(244, 63, 94)))
                        .rounding(Rounding::same(4.0))
                ).on_hover_text("Open interactive 2D latent space timbre morphing orb & spectral trajectory HUD").clicked() {
                    state.requested_open_neural_morph_hud = true;
                }
                ui.add_space(4.0);
            } else if active_kind.contains("Export") || active_kind.contains("Stem") || active_kind.contains("Batch") || active_kind == "ExportPreset" || active_kind == "StemExportFormat" {
                if ui.add(
                    egui::Button::new(RichText::new("📦 Open Multi-Track Stems Export Modal").font(FontId::proportional(10.5)).strong().color(Color32::from_rgb(14, 165, 233)))
                        .fill(Color32::from_rgb(14, 28, 48))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(14, 165, 233)))
                        .rounding(Rounding::same(4.0))
                ).on_hover_text("Open production multi-track batch stem export modal and loudness compliance inspector").clicked() {
                    state.requested_open_stems_export_modal = true;
                }
                ui.add_space(4.0);
            } else if active_kind.contains("Bowed") || active_kind.contains("Violin") || active_kind.contains("Cello") || active_kind == "BowedStringNode" || active_kind == "BowedString" {
                if ui.add(
                    egui::Button::new(RichText::new("🎻 Open Bowed String Acoustic Friction HUD").font(FontId::proportional(10.5)).strong().color(Color32::from_rgb(56, 189, 248)))
                        .fill(Color32::from_rgb(18, 30, 48))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(56, 189, 248)))
                        .rounding(Rounding::same(4.0))
                ).on_hover_text("Open interactive 2D stick-slip friction radar, Helmholtz resonance & Schelleng limits HUD").clicked() {
                    state.requested_open_bowed_string_hud = true;
                }
                ui.add_space(4.0);
            } else if active_kind.contains("Shakuhachi") || active_kind.contains("BambooFlute") || active_kind == "ShakuhachiNode" || active_kind == "Shakuhachi" {
                if ui.add(
                    egui::Button::new(RichText::new("🎍 Open Shakuhachi Bamboo Flute HUD").font(FontId::proportional(10.5)).strong().color(Color32::from_rgb(52, 211, 153)))
                        .fill(Color32::from_rgb(16, 38, 28))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(52, 211, 153)))
                        .rounding(Rounding::same(4.0))
                ).on_hover_text("Open interactive Utaguchi blowing edge chiff, jet velocity & Meri/Kari pitch-bend HUD").clicked() {
                    state.requested_open_shakuhachi_hud = true;
                }
                ui.add_space(4.0);
            } else if active_kind.contains("Sitar") || active_kind == "SitarNode" || active_kind == "SitarModel" || active_kind.contains("Jawari") {
                if ui.add(
                    egui::Button::new(RichText::new("🪕 Open Sitar Curved Jawari Bridge HUD").font(FontId::proportional(10.5)).strong().color(Color32::from_rgb(245, 158, 11)))
                        .fill(Color32::from_rgb(38, 28, 14))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(245, 158, 11)))
                        .rounding(Rounding::same(4.0))
                ).on_hover_text("Open interactive curved Jawari buzz bridge, sympathetic Tarab bleed & lateral Meend pull HUD").clicked() {
                    state.requested_open_sitar_hud = true;
                }
                ui.add_space(4.0);
            } else if active_kind.contains("TurkishNey") || active_kind.contains("Ney") || active_kind.contains("WoodwindJet") || active_kind == "TurkishNeyNode" || active_kind == "WoodwindJetNode" {
                if ui.add(
                    egui::Button::new(RichText::new("🪈 Open Turkish Ney Flute Embouchure HUD").font(FontId::proportional(10.5)).strong().color(Color32::from_rgb(45, 212, 191)))
                        .fill(Color32::from_rgb(14, 35, 35))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(45, 212, 191)))
                        .rounding(Rounding::same(4.0))
                ).on_hover_text("Open interactive Baspare horn lip jet vortex, acoustic bore circulation & octave overblowing HUD").clicked() {
                    state.requested_open_turkish_ney_hud = true;
                }
                ui.add_space(4.0);
            } else if active_kind.contains("Steelpan") || active_kind.contains("SteelDrum") || active_kind == "SteelpanNode" || active_kind == "SteelpanModel" {
                if ui.add(
                    egui::Button::new(RichText::new("🛢 Open Caribbean Steelpan Annular Resonance HUD").font(FontId::proportional(10.5)).strong().color(Color32::from_rgb(0, 229, 255)))
                        .fill(Color32::from_rgb(12, 28, 38))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(0, 229, 255)))
                        .rounding(Rounding::same(4.0))
                ).on_hover_text("Open interactive Caribbean steelpan concave oil barrel bowl, annular ring resonance & modal strike HUD").clicked() {
                    state.requested_open_steelpan_hud = true;
                }
                ui.add_space(4.0);
            } else if active_kind.contains("Mbira") || active_kind.contains("Kalimba") || active_kind == "MbiraNode" || active_kind == "KalimbaNode" || active_kind == "MbiraKalimbaModel" || active_kind == "MbiraModel" {
                if ui.add(
                    egui::Button::new(RichText::new("🎵 Open Lamellophone Mbira & Kalimba Tine HUD").font(FontId::proportional(10.5)).strong().color(Color32::from_rgb(255, 107, 43)))
                        .fill(Color32::from_rgb(38, 20, 12))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(255, 107, 43)))
                        .rounding(Rounding::same(4.0))
                ).on_hover_text("Open interactive lamellophone mbira & kalimba tine modal dispersion, acoustic buzz & pluck force HUD").clicked() {
                    state.requested_open_mbira_hud = true;
                }
                ui.add_space(4.0);
            } else if active_kind.contains("Koto") || active_kind.contains("Guzheng") || active_kind == "KotoNode" || active_kind == "KotoSynthesizer" || active_kind == "KotoStringVoice" || active_kind == "KotoSoundboard" {
                if ui.add(
                    egui::Button::new(RichText::new("箏 Open Japanese Koto 13-String Zither HUD").font(FontId::proportional(10.5)).strong().color(Color32::from_rgb(245, 158, 11)))
                        .fill(Color32::from_rgb(38, 28, 12))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(245, 158, 11)))
                        .rounding(Rounding::same(4.0))
                ).on_hover_text("Open interactive Japanese 13-string Koto Paulownia zither, movable Ji bridges & Oshi-Ite string tension HUD").clicked() {
                    state.requested_open_koto_hud = true;
                }
                ui.add_space(4.0);
            } else if active_kind.contains("Gamelan") || active_kind.contains("Gender") || active_kind.contains("Jegogan") || active_kind.contains("Metallophone") || active_kind == "GamelanGender" || active_kind == "AcousticBalineseGamelanJegoganBronzeBar" {
                if ui.add(
                    egui::Button::new(RichText::new("🔔 Open Balinese Gamelan Gender Metallophone HUD").font(FontId::proportional(10.5)).strong().color(Color32::from_rgb(52, 211, 153)))
                        .fill(Color32::from_rgb(12, 38, 28))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(52, 211, 153)))
                        .rounding(Rounding::same(4.0))
                ).on_hover_text("Open interactive Balinese Gamelan Gender bronze bar modal inharmonicity, bamboo resonator & ombak beating HUD").clicked() {
                    state.requested_open_gamelan_hud = true;
                }
                ui.add_space(4.0);
            } else if active_kind.contains("Dulcimer") || active_kind.contains("Cimbalom") || active_kind.contains("Santur") || active_kind.contains("Yangqin") || active_kind.contains("Psaltery") || active_kind == "DulcimerCimbalomModel" {
                if ui.add(
                    egui::Button::new(RichText::new("🎼 Open Hammered Dulcimer & Cimbalom String Dispersion HUD").font(FontId::proportional(10.5)).strong().color(Color32::from_rgb(255, 107, 43)))
                        .fill(Color32::from_rgb(38, 20, 12))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(255, 107, 43)))
                        .rounding(Rounding::same(4.0))
                ).on_hover_text("Open interactive Hammered Dulcimer & Cimbalom trapezoidal soundboard, dual bridges & hammer strike HUD").clicked() {
                    state.requested_open_dulcimer_hud = true;
                }
                ui.add_space(4.0);
            } else if active_kind.contains("Clavinet") || active_kind.contains("Clav") || active_kind == "ClavinetNode" || active_kind == "ClavinetD6" {
                if ui.add(
                    egui::Button::new(RichText::new("⚡ Open Electromechanical Clavinet D6 String-Anvil HUD").font(FontId::proportional(10.5)).strong().color(Color32::from_rgb(249, 115, 22)))
                        .fill(Color32::from_rgb(38, 22, 10))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(249, 115, 22)))
                        .rounding(Rounding::same(4.0))
                ).on_hover_text("Open interactive Electromechanical Clavinet D6 rubber anvil contact, dual pickups & 4-way tone switches HUD").clicked() {
                    state.requested_open_clavinet_hud = true;
                }
                ui.add_space(4.0);
            }

            // Track Multi-Node Device Chain Strip
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("Chain ({}):", state.chain_devices.len())).font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(148, 163, 184)));
                if state.chain_devices.len() > 1 {
                    if state.selected_chain_idx > 0 && ui.small_button(RichText::new("◀").font(FontId::proportional(8.5))).on_hover_text("Move device earlier in chain").clicked() {
                        let cur = state.selected_chain_idx;
                        state.move_device(cur, cur - 1);
                    }
                    if state.selected_chain_idx + 1 < state.chain_devices.len() && ui.small_button(RichText::new("▶").font(FontId::proportional(8.5))).on_hover_text("Move device later in chain").clicked() {
                        let cur = state.selected_chain_idx;
                        state.move_device(cur, cur + 1);
                    }
                    if ui.small_button(RichText::new("✕").font(FontId::proportional(8.5)).color(Color32::from_rgb(239, 68, 68))).on_hover_text("Remove selected device from chain").clicked() {
                        let cur = state.selected_chain_idx;
                        state.remove_device(cur);
                    }
                }
            });
            ui.add_space(3.0);

            // Horizontal Device Chain Pills
            let mut dev_to_select = None;
            let mut dev_to_toggle_bypass = None;
            ui.horizontal_wrapped(|ui| {
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
                        .inner_margin(egui::Margin::symmetric(4.0, 2.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let byp_col = if !dev.is_bypassed { Color32::from_rgb(34, 197, 94) } else { Color32::from_rgb(100, 116, 139) };
                                if ui.small_button(RichText::new("⏻").font(FontId::proportional(8.5)).color(byp_col)).on_hover_text(if dev.is_bypassed { "Device Bypassed" } else { "Device Active" }).clicked() {
                                    dev_to_toggle_bypass = Some(d_i);
                                }
                                let badge_text = format!("{} {}. {}", d_icon, d_i + 1, dev.display_name);
                                let badge_resp = ui.selectable_label(is_sel, RichText::new(badge_text).font(FontId::proportional(9.0)).strong().color(if is_sel { Color32::WHITE } else { Color32::from_rgb(200, 215, 235) }));
                                if badge_resp.clicked() {
                                    dev_to_select = Some(d_i);
                                }
                            });
                        });
                    ui.add_space(2.0);
                }
            });

            if let Some(idx) = dev_to_select {
                state.select_device(idx);
            }
            if let Some(idx) = dev_to_toggle_bypass {
                state.toggle_device_bypass(idx);
            }

            ui.add_space(4.0);

            // [➕ Add Device ▾] Dropdown Menu across DSP categories
            let mut module_to_add: Option<String> = None;
            ui.horizontal(|ui| {
                ui.label(RichText::new("Add:").font(FontId::proportional(10.0)).color(Color32::from_rgb(148, 163, 184)));
                egui::ComboBox::from_id_source("inspector_add_device_combo")
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
                            let pid_pro = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 500 + p_i as u32);
                            if bus.get(pid_pro).is_some() {
                                bus.set(pid_pro, def);
                            }
                            let pid_multi = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + state.selected_chain_idx as u32 * 20 + p_i as u32);
                            if bus.get(pid_multi).is_some() {
                                bus.set(pid_multi, def);
                            }
                        }
                    }
                }
            }

            ui.add_space(6.0);

            // Module Type Swap Selector Dropdown
            let cur_selection = state.selected_node_kind.clone().unwrap_or_else(|| "AetherSynth".to_string());
            let cur_display = registry.get(&cur_selection).map(|d| format!("{} {}", d.category.icon(), d.display_name)).unwrap_or_else(|| cur_selection.clone());

            ui.horizontal(|ui| {
                ui.label(RichText::new("Module:").font(FontId::proportional(10.0)).color(Color32::from_rgb(148, 163, 184)));
                egui::ComboBox::from_id_source("modern_inspector_node_selector")
                    .selected_text(RichText::new(cur_display).font(FontId::proportional(10.0)).color(Color32::from_rgb(56, 189, 248)))
                    .show_ui(ui, |ui| {
                        for desc in registry.list_all() {
                            let is_sel = state.selected_node_kind.as_deref() == Some(&desc.kind_id);
                            let label = format!("{} {}", desc.category.icon(), desc.display_name);
                            if ui.selectable_label(is_sel, label).clicked() {
                                state.selected_node_kind = Some(desc.kind_id.clone());
                                state.target_name = desc.display_name.clone();
                                if let Some(dev) = state.chain_devices.get_mut(state.selected_chain_idx) {
                                    dev.kind = desc.kind_id.clone();
                                    dev.display_name = desc.display_name.clone();
                                }
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
            });

            ui.add_space(8.0);

            if let Some(ref node_kind) = state.selected_node_kind {
                if let Some(descriptor) = registry.get(node_kind) {
                    descriptor.render_pro_inspector(ui, &mut state.node_param_values, param_bus, track_id, state.selected_chain_idx);
                }
            }
                });
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modern_inspector_defaults() {
        let state = ModernInspectorState::default();
        assert_eq!(state.target_name, "Synth 1");
        assert_eq!(state.selected_node_kind.as_deref(), Some("AetherSynth"));
        assert_eq!(state.gain_db, 0.0);
        assert!(!state.is_collapsed);
    }

    #[test]
    #[cfg(feature = "gui")]
    fn test_modern_inspector_headless_rendering_with_context() {
        let mut state = ModernInspectorState::default();
        let ctx = egui::Context::default();
        let bus = summoner_core::param_bus::ParamBus::new();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_inspector_with_context(ui, &mut state, Some(&bus), 2);
            });
        });

        assert!(!state.node_param_values.is_empty());
    }
}
