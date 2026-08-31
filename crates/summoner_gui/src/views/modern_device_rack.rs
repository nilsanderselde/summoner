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
                ui.label(RichText::new(&state.device_name).font(FontId::proportional(11.0)).color(Color32::from_rgb(148, 163, 184)));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let _ = ui.small_button("✕");
                    let _ = ui.small_button("⋯");
                    egui::Frame::none()
                        .fill(Color32::from_rgb(24, 34, 52))
                        .rounding(Rounding::same(3.0))
                        .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new("Reusable").font(FontId::proportional(9.0)).color(Color32::from_rgb(148, 163, 184)));
                        });
                });
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

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
                            // Top Row Knobs
                            ui.horizontal(|ui| {
                                draw_rotary_dial(ui, "Cutoff", &mut state.cutoff, Color32::from_rgb(56, 189, 248));
                                ui.add_space(4.0);
                                draw_rotary_dial(ui, "Reso", &mut state.resonance, Color32::from_rgb(245, 158, 11));
                                ui.add_space(4.0);
                                draw_rotary_dial(ui, "Decay", &mut state.decay, Color32::from_rgb(245, 158, 11));
                            });
                            ui.add_space(4.0);
                            // Bottom Row Knobs
                            ui.horizontal(|ui| {
                                draw_rotary_dial(ui, "Amt", &mut state.env_decay, Color32::from_rgb(56, 189, 248));
                                ui.add_space(4.0);
                                draw_rotary_dial(ui, "Amt", &mut state.mod_amt, Color32::from_rgb(245, 158, 11));
                                ui.add_space(4.0);
                                draw_rotary_dial(ui, "Drive", &mut state.drive, Color32::from_rgb(245, 158, 11));
                            });
                        });
                    });

                ui.add_space(6.0);

                // Section 2: Vertical Sliders (Osc Mix, Shape, Vol)
                egui::Frame::none()
                    .fill(Color32::from_rgb(18, 24, 36))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 74)))
                    .rounding(Rounding::same(4.0))
                    .inner_margin(egui::Margin::symmetric(8.0, 6.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            draw_vertical_fader(ui, "Osc Mix", &mut state.osc_mix);
                            ui.add_space(4.0);
                            draw_vertical_fader(ui, "Shape", &mut state.shape);
                            ui.add_space(4.0);
                            draw_vertical_fader(ui, "Vol", &mut state.volume);
                        });
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

                // Section 5: LFO & Modulators
                egui::Frame::none()
                    .fill(Color32::from_rgb(18, 24, 36))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 74)))
                    .rounding(Rounding::same(4.0))
                    .inner_margin(egui::Margin::symmetric(8.0, 6.0))
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("LFO").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(200, 215, 235)));
                                ui.label(RichText::new("〰").font(FontId::proportional(10.0)).color(Color32::from_rgb(56, 189, 248)));
                            });
                            ui.add_space(2.0);
                            ui.horizontal(|ui| {
                                draw_rotary_dial(ui, "Min", &mut state.lfo_speed, Color32::from_rgb(56, 189, 248));
                                draw_rotary_dial(ui, "Wait", &mut state.lfo_depth, Color32::from_rgb(148, 163, 184));
                            });
                            ui.add_space(2.0);
                            ui.horizontal(|ui| {
                                let mut coom = 0.50;
                                let mut lfo_val = 0.60;
                                draw_rotary_dial(ui, "Coom", &mut coom, Color32::from_rgb(56, 189, 248));
                                draw_rotary_dial(ui, "LFO", &mut lfo_val, Color32::from_rgb(148, 163, 184));
                            });
                        });
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
