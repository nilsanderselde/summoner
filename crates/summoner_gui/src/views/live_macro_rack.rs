// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Touch-Optimized Live Performance Macro Rack & Multi-Touch XY Pad Controls (Step 1321).

use crate::layout_math::Rect;
use crate::touch_controls::{ContrastColorPalette, MIN_HIT_TARGET_PT};

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Pos2, Stroke, Vec2};

/// Dimensions and bounds constants for Live Macro Rack.
pub const MIN_XY_PAD_SIZE: f32 = 160.0;
pub const PUCK_RADIUS: f32 = 14.0;
pub const PUCK_HIT_RADIUS: f32 = 22.0; // >= 44x44pt hit target

/// Single Dynamic Multi-Touch XY Pad control state.
#[derive(Debug, Clone, PartialEq)]
pub struct LiveXyPadState {
    pub name: String,
    pub x_val: f32, // 0.0 ..= 1.0
    pub y_val: f32, // 0.0 ..= 1.0
    pub param_x_name: String,
    pub param_y_name: String,
    pub spring_to_center: bool,
    pub is_dragging: bool,
    pub custom_color: (u8, u8, u8),
}

impl LiveXyPadState {
    pub fn new(
        name: impl Into<String>,
        param_x: impl Into<String>,
        param_y: impl Into<String>,
        color: (u8, u8, u8),
    ) -> Self {
        Self {
            name: name.into(),
            x_val: 0.5,
            y_val: 0.5,
            param_x_name: param_x.into(),
            param_y_name: param_y.into(),
            spring_to_center: false,
            is_dragging: false,
            custom_color: color,
        }
    }

    /// Set position clamped to [0.0, 1.0]
    pub fn set_pos(&mut self, x: f32, y: f32) {
        self.x_val = x.clamp(0.0, 1.0);
        self.y_val = y.clamp(0.0, 1.0);
    }

    /// Apply spring return if active and released
    pub fn update_spring(&mut self) {
        if self.spring_to_center && !self.is_dragging {
            self.x_val += (0.5 - self.x_val) * 0.25;
            self.y_val += (0.5 - self.y_val) * 0.25;
            if (self.x_val - 0.5).abs() < 1e-3 {
                self.x_val = 0.5;
            }
            if (self.y_val - 0.5).abs() < 1e-3 {
                self.y_val = 0.5;
            }
        }
    }

    /// Coordinate transformation: normalized (0..1, 0..1) to canvas pixel rect
    pub fn normalized_to_canvas(
        norm_x: f32,
        norm_y: f32,
        pad_rect: (f32, f32, f32, f32), // (x, y, w, h)
    ) -> (f32, f32) {
        let (rx, ry, rw, rh) = pad_rect;
        let px = rx + norm_x.clamp(0.0, 1.0) * rw;
        let py = ry + (1.0 - norm_y.clamp(0.0, 1.0)) * rh; // Invert Y so 1.0 is at top
        (px, py)
    }

    /// Inverse coordinate transformation: canvas pixel to normalized (0..1, 0..1)
    pub fn canvas_to_normalized(
        canvas_x: f32,
        canvas_y: f32,
        pad_rect: (f32, f32, f32, f32),
    ) -> (f32, f32) {
        let (rx, ry, rw, rh) = pad_rect;
        let norm_x = if rw > 0.0 {
            ((canvas_x - rx) / rw).clamp(0.0, 1.0)
        } else {
            0.5
        };
        let norm_y = if rh > 0.0 {
            (1.0 - (canvas_y - ry) / rh).clamp(0.0, 1.0)
        } else {
            0.5
        };
        (norm_x, norm_y)
    }

    /// Check if click is inside puck hit target (radius >= 22.0pt)
    pub fn is_puck_hit(puck_pos: (f32, f32), click_pos: (f32, f32)) -> bool {
        let dx = puck_pos.0 - click_pos.0;
        let dy = puck_pos.1 - click_pos.1;
        (dx * dx + dy * dy).sqrt() <= PUCK_HIT_RADIUS
    }

    /// Get puck hit bounds Rect
    pub fn puck_hit_bounds(&self, pad_rect: (f32, f32, f32, f32)) -> Rect {
        let (px, py) = Self::normalized_to_canvas(self.x_val, self.y_val, pad_rect);
        Rect::new(
            px - PUCK_HIT_RADIUS,
            py - PUCK_HIT_RADIUS,
            PUCK_HIT_RADIUS * 2.0,
            PUCK_HIT_RADIUS * 2.0,
        )
    }
}

/// Macro Knob parameter definition
#[derive(Debug, Clone, PartialEq)]
pub struct QuickMacroKnob {
    pub name: String,
    pub value: f32, // 0.0 ..= 1.0
    pub display_unit: String,
    pub color: (u8, u8, u8),
}

impl QuickMacroKnob {
    pub fn new(
        name: impl Into<String>,
        value: f32,
        unit: impl Into<String>,
        color: (u8, u8, u8),
    ) -> Self {
        Self {
            name: name.into(),
            value: value.clamp(0.0, 1.0),
            display_unit: unit.into(),
            color,
        }
    }
}

/// Touch-Optimized Live Performance Macro Rack View (Step 1321).
#[derive(Debug, Clone)]
pub struct LiveMacroRackView {
    pub pad_left: LiveXyPadState,
    pub pad_right: LiveXyPadState,
    pub macro_knobs: Vec<QuickMacroKnob>,
    pub active_snapshot_index: usize,
    pub snapshot_names: Vec<String>,
    pub color_palette: ContrastColorPalette,
}

impl Default for LiveMacroRackView {
    fn default() -> Self {
        Self::new()
    }
}

impl LiveMacroRackView {
    pub fn new() -> Self {
        Self {
            pad_left: LiveXyPadState::new(
                "PAD 1: TONE / FILTER",
                "Cutoff Freq",
                "Resonance (Q)",
                (0, 229, 255),
            ),
            pad_right: LiveXyPadState::new(
                "PAD 2: SPACE / DYNAMICS",
                "Reverb Space",
                "Drive Distortion",
                (255, 107, 43),
            ),
            macro_knobs: vec![
                QuickMacroKnob::new("Macro 1 (Sub)", 0.65, "dB", (0, 229, 255)),
                QuickMacroKnob::new("Macro 2 (Air)", 0.40, "kHz", (76, 201, 240)),
                QuickMacroKnob::new("Macro 3 (Width)", 0.80, "%", (255, 215, 0)),
                QuickMacroKnob::new("Macro 4 (Punch)", 0.55, "ms", (255, 107, 43)),
            ],
            active_snapshot_index: 0,
            snapshot_names: vec![
                "Intro".into(),
                "Build".into(),
                "Drop".into(),
                "Outro".into(),
            ],
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Select performance snapshot
    pub fn select_snapshot(&mut self, index: usize) {
        if index < self.snapshot_names.len() {
            self.active_snapshot_index = index;
        }
    }

    /// Render ASCII summary of performance rack state
    pub fn render_ascii(&self) -> String {
        let mut out = String::new();
        out.push_str("[LIVE PERFORMANCE MACRO RACK]\n");
        out.push_str(&format!(
            "PAD 1: X({}): {:.2} | Y({}): {:.2} | Spring: {}\n",
            self.pad_left.param_x_name,
            self.pad_left.x_val,
            self.pad_left.param_y_name,
            self.pad_left.y_val,
            if self.pad_left.spring_to_center {
                "ON"
            } else {
                "OFF"
            }
        ));
        out.push_str(&format!(
            "PAD 2: X({}): {:.2} | Y({}): {:.2} | Spring: {}\n",
            self.pad_right.param_x_name,
            self.pad_right.x_val,
            self.pad_right.param_y_name,
            self.pad_right.y_val,
            if self.pad_right.spring_to_center {
                "ON"
            } else {
                "OFF"
            }
        ));
        out.push_str("MACROS: ");
        for knob in &self.macro_knobs {
            out.push_str(&format!(
                "{}: {:.0}{} | ",
                knob.name,
                knob.value * 100.0,
                knob.display_unit
            ));
        }
        out.push('\n');
        out.push_str(&format!(
            "SNAPSHOT: {} ({})\n",
            self.snapshot_names[self.active_snapshot_index],
            self.active_snapshot_index + 1
        ));
        out
    }
}

#[cfg(feature = "gui")]
impl LiveMacroRackView {
    /// Render egui Live Performance Macro Rack Widget
    pub fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        self.pad_left.update_spring();
        self.pad_right.update_spring();

        ui.vertical(|ui| {
            // Header Bar
            ui.horizontal(|ui| {
                ui.heading("LIVE PERFORMANCE MACRO RACK");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    for (i, name) in self.snapshot_names.iter().enumerate().rev() {
                        let is_active = self.active_snapshot_index == i;
                        let btn_color = if is_active {
                            Color32::from_rgb(0, 229, 255)
                        } else {
                            Color32::from_rgb(60, 75, 100)
                        };
                        let text_color = if is_active {
                            Color32::BLACK
                        } else {
                            Color32::WHITE
                        };
                        let btn = egui::Button::new(
                            egui::RichText::new(format!("{}: {}", i + 1, name))
                                .color(text_color)
                                .size(13.0)
                                .strong(),
                        )
                        .min_size(Vec2::new(MIN_HIT_TARGET_PT, MIN_HIT_TARGET_PT))
                        .fill(btn_color);

                        if ui.add(btn).clicked() {
                            self.active_snapshot_index = i;
                        }
                    }
                    ui.label(
                        egui::RichText::new("SNAPSHOT:")
                            .size(12.0)
                            .color(Color32::from_rgb(180, 195, 215)),
                    );
                });
            });

            ui.add_space(8.0);

            // Main Dual XY Pad Display Section
            ui.horizontal(|ui| {
                // Pad 1 (Left)
                self.render_xy_pad_widget(ui, true);

                ui.add_space(16.0);

                // Pad 2 (Right)
                self.render_xy_pad_widget(ui, false);
            });

            ui.add_space(12.0);

            // Bottom Quick Macro Knobs Bar
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("MACROS:")
                        .size(13.0)
                        .strong()
                        .color(Color32::from_rgb(0, 229, 255)),
                );
                for knob in &mut self.macro_knobs {
                    ui.group(|ui| {
                        ui.set_min_size(Vec2::new(120.0, MIN_HIT_TARGET_PT));
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new(&knob.name)
                                    .size(11.0)
                                    .color(Color32::from_rgb(200, 215, 235)),
                            );
                            let slider = egui::Slider::new(&mut knob.value, 0.0..=1.0)
                                .show_value(false)
                                .trailing_fill(true);
                            ui.add(slider);
                            ui.label(
                                egui::RichText::new(format!(
                                    "{:.0}% {}",
                                    knob.value * 100.0,
                                    knob.display_unit
                                ))
                                .size(10.0)
                                .color(Color32::from_rgb(
                                    knob.color.0,
                                    knob.color.1,
                                    knob.color.2,
                                )),
                            );
                        });
                    });
                }
            });
        })
        .response
    }

    /// Renders an individual XY Pad interactive canvas
    fn render_xy_pad_widget(&mut self, ui: &mut egui::Ui, is_left: bool) {
        let pad = if is_left {
            &mut self.pad_left
        } else {
            &mut self.pad_right
        };

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&pad.name).size(13.0).strong().color(
                    Color32::from_rgb(pad.custom_color.0, pad.custom_color.1, pad.custom_color.2),
                ));
                let spring_btn_text = if pad.spring_to_center {
                    "Spring: ON"
                } else {
                    "Spring: OFF"
                };
                let spring_color = if pad.spring_to_center {
                    Color32::from_rgb(0, 255, 180)
                } else {
                    Color32::from_rgb(140, 150, 170)
                };
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(spring_btn_text)
                                .size(11.0)
                                .color(spring_color),
                        )
                        .min_size(Vec2::new(MIN_HIT_TARGET_PT, 28.0)),
                    )
                    .clicked()
                {
                    pad.spring_to_center = !pad.spring_to_center;
                }
            });

            // Allocate XY Canvas Area (Min 180x180)
            let pad_size = Vec2::new(MIN_XY_PAD_SIZE.max(200.0), MIN_XY_PAD_SIZE.max(180.0));
            let (response, painter) = ui.allocate_painter(pad_size, egui::Sense::click_and_drag());
            let rect = response.rect;
            let pad_tuple = (rect.min.x, rect.min.y, rect.width(), rect.height());

            // Handle Touch / Drag
            if response.dragged() || response.clicked() {
                pad.is_dragging = true;
                if let Some(mouse_pos) = response.interact_pointer_pos() {
                    let (nx, ny) =
                        LiveXyPadState::canvas_to_normalized(mouse_pos.x, mouse_pos.y, pad_tuple);
                    pad.set_pos(nx, ny);
                }
            } else {
                pad.is_dragging = false;
            }

            // Canvas Background
            painter.rect_filled(rect, 8.0, Color32::from_rgb(14, 18, 28));
            painter.rect_stroke(
                rect,
                8.0,
                Stroke::new(1.5_f32, Color32::from_rgb(45, 60, 85)),
            );

            // Subdivided Grid Lines
            for i in 1..4 {
                let gx = rect.min.x + (rect.width() * (i as f32 * 0.25));
                let gy = rect.min.y + (rect.height() * (i as f32 * 0.25));
                painter.line_segment(
                    [Pos2::new(gx, rect.min.y), Pos2::new(gx, rect.max.y)],
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 110, 60)),
                );
                painter.line_segment(
                    [Pos2::new(rect.min.x, gy), Pos2::new(rect.max.x, gy)],
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 110, 60)),
                );
            }

            // Puck Pixel Position
            let (px, py) = LiveXyPadState::normalized_to_canvas(pad.x_val, pad.y_val, pad_tuple);
            let puck_pos = Pos2::new(px, py);

            // Crosshair Guides
            let guide_stroke = Stroke::new(
                1.0_f32,
                Color32::from_rgba_unmultiplied(
                    pad.custom_color.0,
                    pad.custom_color.1,
                    pad.custom_color.2,
                    100,
                ),
            );
            painter.line_segment(
                [Pos2::new(rect.min.x, py), Pos2::new(rect.max.x, py)],
                guide_stroke,
            );
            painter.line_segment(
                [Pos2::new(px, rect.min.y), Pos2::new(px, rect.max.y)],
                guide_stroke,
            );

            // Outer Active Touch Target Region (> 44x44pt)
            painter.circle_stroke(
                puck_pos,
                PUCK_HIT_RADIUS,
                Stroke::new(
                    if pad.is_dragging { 2.0_f32 } else { 1.0_f32 },
                    Color32::from_rgba_unmultiplied(
                        pad.custom_color.0,
                        pad.custom_color.1,
                        pad.custom_color.2,
                        120,
                    ),
                ),
            );

            // Glowing Puck Body
            painter.circle_filled(
                puck_pos,
                PUCK_RADIUS + 4.0,
                Color32::from_rgba_unmultiplied(
                    pad.custom_color.0,
                    pad.custom_color.1,
                    pad.custom_color.2,
                    60,
                ),
            );
            painter.circle_filled(
                puck_pos,
                PUCK_RADIUS,
                Color32::from_rgb(pad.custom_color.0, pad.custom_color.1, pad.custom_color.2),
            );
            painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

            // Readout Labels at bottom
            let readout = format!(
                "X ({}): {:.0}% | Y ({}): {:.0}%",
                pad.param_x_name,
                pad.x_val * 100.0,
                pad.param_y_name,
                pad.y_val * 100.0
            );
            ui.label(
                egui::RichText::new(readout)
                    .size(11.0)
                    .color(Color32::from_rgb(180, 200, 225)),
            );
        });
    }
}
