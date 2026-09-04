// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Reusable Device Box Primitive with Rotary Knobs, Faders, Live CRT Oscilloscope & Filter HUD.

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, FontId, Rect, RichText, Rounding, Stroke, Vec2};
use serde::{Deserialize, Serialize};

pub const MIN_KNOB_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding target

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
}

impl Default for ModernDeviceRackState {
    fn default() -> Self {
        Self {
            device_name: "Synth 1".to_string(),
            selected_node_kind: Some("AetherSynth".to_string()),
            node_param_values: std::collections::HashMap::new(),
            is_enabled: true,
            is_minimized: false,
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

#[cfg(feature = "gui")]
pub fn show_modern_device_rack(
    ui: &mut egui::Ui,
    state: &mut ModernDeviceRackState,
    oscilloscope_data: Option<&[f32]>,
) {
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
                    ui.add_space(4.0);
                    let pwr_col = if state.is_enabled { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(100, 116, 139) };
                    if ui.button(RichText::new("⏻").font(FontId::proportional(12.0)).color(pwr_col)).clicked() {
                        state.is_enabled = !state.is_enabled;
                    }
                    ui.add_space(6.0);

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

    let rack_height = 190.0;
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
                if ui.button(RichText::new("⏻").font(FontId::proportional(12.0)).color(pwr_col)).clicked() {
                    state.is_enabled = !state.is_enabled;
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
                    let _ = ui.small_button("✕");
                    let _ = ui.small_button("⋯");
                    egui::Frame::none()
                        .fill(Color32::from_rgb(24, 34, 52))
                        .rounding(Rounding::same(3.0))
                        .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new("Zero CLI Left Behind").font(FontId::proportional(9.0)).color(Color32::from_rgb(148, 163, 184)));
                        });
                });
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            if !state.is_enabled {
                ui.centered_and_justified(|ui| {
                    ui.label(RichText::new("DEVICE BYPASSED").font(FontId::proportional(14.0)).color(Color32::from_rgb(100, 116, 139)));
                });
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

                                let p_len = knob_params.len();
                                // Top Row: first 3 parameters
                                ui.horizontal(|ui| {
                                    for i in 0..3 {
                                        if i < p_len {
                                            let p = knob_params[i];
                                            let default_norm = match &p.widget {
                                                crate::dsp_node_ui::DspWidgetKind::RotaryKnob { min, max, default, .. } => {
                                                    ((*default - *min) / (*max - *min).max(1e-5)).clamp(0.0, 1.0)
                                                }
                                                crate::dsp_node_ui::DspWidgetKind::VerticalFader { min, max, default, .. } => {
                                                    ((*default - *min) / (*max - *min).max(1e-5)).clamp(0.0, 1.0)
                                                }
                                                _ => 0.5,
                                            };
                                            let val = state.node_param_values.entry(p.id.clone()).or_insert(default_norm);
                                            let color = if i == 0 { Color32::from_rgb(56, 189, 248) } else { cat_accent };
                                            draw_rotary_dial(ui, &p.name, val, color);
                                        } else {
                                            // Blank indicator for unoccupied slot
                                            let mut dummy = 0.5;
                                            draw_rotary_dial(ui, "---", &mut dummy, Color32::from_rgb(45, 55, 75));
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
                                        if i < p_len {
                                            let p = knob_params[i];
                                            let default_norm = match &p.widget {
                                                crate::dsp_node_ui::DspWidgetKind::RotaryKnob { min, max, default, .. } => {
                                                    ((*default - *min) / (*max - *min).max(1e-5)).clamp(0.0, 1.0)
                                                }
                                                crate::dsp_node_ui::DspWidgetKind::VerticalFader { min, max, default, .. } => {
                                                    ((*default - *min) / (*max - *min).max(1e-5)).clamp(0.0, 1.0)
                                                }
                                                _ => 0.5,
                                            };
                                            let val = state.node_param_values.entry(p.id.clone()).or_insert(default_norm);
                                            let color = if i == 3 { Color32::from_rgb(56, 189, 248) } else { cat_accent };
                                            draw_rotary_dial(ui, &p.name, val, color);
                                        } else {
                                            let mut dummy = 0.5;
                                            draw_rotary_dial(ui, "---", &mut dummy, Color32::from_rgb(45, 55, 75));
                                        }
                                        if i < 5 {
                                            ui.add_space(4.0);
                                        }
                                    }
                                });
                            } else {
                                // Default / Fallback knobs
                                ui.horizontal(|ui| {
                                    draw_rotary_dial(ui, "Cutoff", &mut state.cutoff, Color32::from_rgb(56, 189, 248));
                                    ui.add_space(4.0);
                                    draw_rotary_dial(ui, "Reso", &mut state.resonance, Color32::from_rgb(245, 158, 11));
                                    ui.add_space(4.0);
                                    draw_rotary_dial(ui, "Decay", &mut state.decay, Color32::from_rgb(245, 158, 11));
                                });
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    draw_rotary_dial(ui, "Amt", &mut state.env_decay, Color32::from_rgb(56, 189, 248));
                                    ui.add_space(4.0);
                                    draw_rotary_dial(ui, "Amt", &mut state.mod_amt, Color32::from_rgb(245, 158, 11));
                                    ui.add_space(4.0);
                                    draw_rotary_dial(ui, "Drive", &mut state.drive, Color32::from_rgb(245, 158, 11));
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
                                    let val = state.node_param_values.entry(p.id.clone()).or_insert(default_norm);
                                    draw_vertical_fader(ui, &p.name, val);
                                    if i + 1 < fader_params.len() {
                                        ui.add_space(4.0);
                                    }
                                }
                                for i in fader_params.len()..3 {
                                    ui.add_space(4.0);
                                    if i == 1 {
                                        draw_vertical_fader(ui, "Shape", &mut state.shape);
                                    } else {
                                        draw_vertical_fader(ui, "Vol", &mut state.volume);
                                    }
                                }
                            });
                        } else {
                            ui.horizontal(|ui| {
                                draw_vertical_fader(ui, "Osc Mix", &mut state.osc_mix);
                                ui.add_space(4.0);
                                draw_vertical_fader(ui, "Shape", &mut state.shape);
                                ui.add_space(4.0);
                                draw_vertical_fader(ui, "Vol", &mut state.volume);
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
                                        let val = state.node_param_values.entry(p.id.clone()).or_insert(default_norm);
                                        draw_rotary_dial(ui, &p.name, val, cat_accent);
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
                                        let val = state.node_param_values.entry(p.id.clone()).or_insert(default_norm);
                                        draw_rotary_dial(ui, &p.name, val, cat_accent);
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
                                    draw_rotary_dial(ui, "Rate", &mut state.lfo_speed, Color32::from_rgb(56, 189, 248));
                                    draw_rotary_dial(ui, "Depth", &mut state.lfo_depth, Color32::from_rgb(148, 163, 184));
                                });
                                ui.add_space(2.0);
                                ui.horizontal(|ui| {
                                    draw_rotary_dial(ui, "Shape", &mut state.shape, Color32::from_rgb(56, 189, 248));
                                    draw_rotary_dial(ui, "Mod", &mut state.mod_amt, Color32::from_rgb(148, 163, 184));
                                });
                            });
                        }
                    });
            });
        });
}

#[cfg(feature = "gui")]
fn draw_rotary_dial(ui: &mut egui::Ui, label: &str, value: &mut f32, ring_color: Color32) {
    let size = Vec2::new(38.0, 52.0);
    let (resp, painter) = ui.allocate_painter(size, egui::Sense::click_and_drag());
    let rect = resp.rect;

    if resp.dragged() {
        let delta_y = ui.input(|i| i.pointer.delta().y);
        *value = (*value - delta_y * 0.01).clamp(0.0, 1.0);
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
}

#[cfg(feature = "gui")]
fn draw_vertical_fader(ui: &mut egui::Ui, label: &str, value: &mut f32) {
    let size = Vec2::new(28.0, 110.0);
    let (resp, painter) = ui.allocate_painter(size, egui::Sense::click_and_drag());
    let rect = resp.rect;

    if resp.dragged() {
        let delta_y = ui.input(|i| i.pointer.delta().y);
        *value = (*value - delta_y * 0.01).clamp(0.0, 1.0);
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
}
