// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Modern Right Sidebar Inspector for Track/Clip Properties & Microtonal Tuning.

#[cfg(feature = "gui")]
use crate::dsp_node_ui::DspNodeUi;
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
            scale_name: "A Minor Pentatonic".to_string(),
            scale_ratio_num: 1,
            scale_ratio_den: 1,
            root_ratio: 1.0,
            octave_offset: -1,
            is_collapsed: false,
        }
    }
}

#[cfg(feature = "gui")]
pub fn show_modern_inspector(ui: &mut egui::Ui, state: &mut ModernInspectorState) {
    if state.is_collapsed {
        if ui.button("◀").clicked() {
            state.is_collapsed = false;
        }
        return;
    }

    let panel_w = 200.0;
    egui::Frame::none()
        .fill(Color32::from_rgb(14, 20, 32))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(28, 40, 60)))
        .rounding(Rounding::same(6.0))
        .inner_margin(egui::Margin::symmetric(10.0, 10.0))
        .show(ui, |ui| {
            ui.set_width(panel_w);
            ui.set_height(ui.available_height());

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
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(format!("{:.1}dB", state.gain_db)).font(FontId::proportional(10.0)).color(Color32::from_rgb(56, 189, 248)));
                });
            });
            ui.add(egui::Slider::new(&mut state.gain_db, -36.0..=12.0).show_value(false));

            ui.add_space(6.0);

            // Pan Fader
            ui.horizontal(|ui| {
                ui.label(RichText::new("Pan").font(FontId::proportional(10.0)).color(Color32::from_rgb(148, 163, 184)));
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
            ui.add(egui::Slider::new(&mut state.pan_val, -1.0..=1.0).show_value(false));

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
                }

                let solo_btn = ui.add(
                    egui::Button::new(RichText::new("Solo").font(FontId::proportional(11.0)).color(if state.is_soloed { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(148, 163, 184) }))
                        .fill(if state.is_soloed { Color32::from_rgba_unmultiplied(56, 189, 248, 30) } else { Color32::from_rgb(24, 34, 52) })
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 74)))
                        .rounding(Rounding::same(4.0))
                );
                if solo_btn.clicked() {
                    state.is_soloed = !state.is_soloed;
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
            if let Some(ref node_kind) = state.selected_node_kind {
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(6.0);
                ui.label(RichText::new("DSP Parameter Inspector").font(FontId::proportional(11.0)).strong().color(Color32::from_rgb(200, 215, 235)));
                ui.add_space(6.0);
                let registry = crate::dsp_node_ui::DspNodeRegistry::new();
                if let Some(descriptor) = registry.get(node_kind) {
                    descriptor.render_pro_inspector(ui, &mut state.node_param_values, None, 1, 0);
                }
            }
        });
}
