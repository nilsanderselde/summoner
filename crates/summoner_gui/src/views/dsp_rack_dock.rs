// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Modular DSP Rack Docking System & Drag-and-Drop Reordering (Step 1341).

use crate::layout_math::Rect;
use crate::touch_controls::{ContrastColorPalette, MIN_HIT_TARGET_PT};

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const DEFAULT_MODULE_HEIGHT: f32 = 88.0;
pub const COLLAPSED_MODULE_HEIGHT: f32 = 48.0;
pub const MODULE_SPACING: f32 = 8.0;

/// Parameter representation for a DSP Module in the rack.
#[derive(Debug, Clone, PartialEq)]
pub struct DspModuleParam {
    pub name: String,
    pub value: f32, // 0.0 ..= 1.0
    pub unit: String,
    pub display_text: String,
}

impl DspModuleParam {
    pub fn new(name: impl Into<String>, value: f32, unit: impl Into<String>) -> Self {
        let val = value.clamp(0.0, 1.0);
        let u = unit.into();
        let display = format!("{:.0}{}", val * 100.0, u);
        Self {
            name: name.into(),
            value: val,
            unit: u,
            display_text: display,
        }
    }
}

/// A single DSP module slot within the rack.
#[derive(Debug, Clone, PartialEq)]
pub struct DspRackModule {
    pub id: String,
    pub name: String,
    pub module_type: String,
    pub is_bypassed: bool,
    pub is_collapsed: bool,
    pub color: (u8, u8, u8),
    pub params: Vec<DspModuleParam>,
}

impl DspRackModule {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        module_type: impl Into<String>,
        color: (u8, u8, u8),
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            module_type: module_type.into(),
            is_bypassed: false,
            is_collapsed: false,
            color,
            params: Vec::new(),
        }
    }

    pub fn with_param(mut self, param: DspModuleParam) -> Self {
        self.params.push(param);
        self
    }

    pub fn current_height(&self) -> f32 {
        if self.is_collapsed {
            COLLAPSED_MODULE_HEIGHT
        } else {
            DEFAULT_MODULE_HEIGHT
        }
    }
}

/// Modular DSP Rack Docking View (Step 1341).
#[derive(Debug, Clone)]
pub struct DspRackDockView {
    pub modules: Vec<DspRackModule>,
    pub dragging_index: Option<usize>,
    pub drag_y_offset: f32,
    pub drop_target_index: Option<usize>,
    pub master_bypass: bool,
    pub color_palette: ContrastColorPalette,
}

impl Default for DspRackDockView {
    fn default() -> Self {
        Self::new()
    }
}

impl DspRackDockView {
    pub fn new() -> Self {
        let mut view = Self {
            modules: Vec::new(),
            dragging_index: None,
            drag_y_offset: 0.0,
            drop_target_index: None,
            master_bypass: false,
            color_palette: ContrastColorPalette::default(),
        };

        // Initialize with default standard FX chain
        view.modules.push(
            DspRackModule::new("tube_drive", "Tube Overdrive", "Distortion", (255, 107, 43))
                .with_param(DspModuleParam::new("Drive", 0.65, "%"))
                .with_param(DspModuleParam::new("Bias", 0.20, "%"))
                .with_param(DspModuleParam::new("Tone", 0.50, "%")),
        );

        view.modules.push(
            DspRackModule::new(
                "svf_filter",
                "State Variable Filter",
                "Filter",
                (0, 229, 255),
            )
            .with_param(DspModuleParam::new("Cutoff", 0.72, "kHz"))
            .with_param(DspModuleParam::new("Resonance", 0.45, "Q"))
            .with_param(DspModuleParam::new("Drive", 0.15, "%")),
        );

        view.modules.push(
            DspRackModule::new(
                "tape_delay",
                "Vintage Tape Delay",
                "Time/Echo",
                (255, 215, 0),
            )
            .with_param(DspModuleParam::new("Time", 0.35, "ms"))
            .with_param(DspModuleParam::new("Feedback", 0.55, "%"))
            .with_param(DspModuleParam::new("Flutter", 0.18, "%")),
        );

        view.modules.push(
            DspRackModule::new("conv_reverb", "Algorithmic Reverb", "Space", (140, 90, 255))
                .with_param(DspModuleParam::new("Room Size", 0.82, "%"))
                .with_param(DspModuleParam::new("Damping", 0.40, "%"))
                .with_param(DspModuleParam::new("Mix Wet", 0.30, "%")),
        );

        view
    }

    pub fn add_module(&mut self, module: DspRackModule) {
        self.modules.push(module);
    }

    pub fn remove_module(&mut self, index: usize) -> Option<DspRackModule> {
        if index < self.modules.len() {
            Some(self.modules.remove(index))
        } else {
            None
        }
    }

    /// Reorders a module from `from_index` to `to_index`.
    pub fn reorder_module(&mut self, from_index: usize, to_index: usize) -> bool {
        if from_index >= self.modules.len() || to_index > self.modules.len() {
            return false;
        }
        if from_index == to_index || from_index + 1 == to_index {
            return false;
        }

        let module = self.modules.remove(from_index);
        let dest = if to_index > from_index {
            to_index - 1
        } else {
            to_index
        };
        self.modules.insert(dest.min(self.modules.len()), module);
        true
    }

    /// Calculates slot bounding boxes given container starting coordinates.
    pub fn calculate_module_bounds(&self, start_x: f32, start_y: f32, width: f32) -> Vec<Rect> {
        let mut bounds = Vec::with_capacity(self.modules.len());
        let mut cur_y = start_y;

        for module in &self.modules {
            let h = module.current_height();
            bounds.push(Rect::new(start_x, cur_y, width, h));
            cur_y += h + MODULE_SPACING;
        }

        bounds
    }

    /// Calculates the drop target index given cursor Y coordinate.
    pub fn calculate_drop_target(&self, cursor_y: f32, start_y: f32, width: f32) -> usize {
        let bounds = self.calculate_module_bounds(0.0, start_y, width);
        if bounds.is_empty() || cursor_y < start_y {
            return 0;
        }

        for (i, b) in bounds.iter().enumerate() {
            let mid_y = b.y + b.height * 0.5;
            if cursor_y < mid_y {
                return i;
            }
        }

        self.modules.len()
    }

    /// Handles drag start event
    pub fn handle_drag_start(&mut self, index: usize, cursor_y: f32, start_y: f32, width: f32) {
        if index < self.modules.len() {
            self.dragging_index = Some(index);
            let bounds = self.calculate_module_bounds(0.0, start_y, width);
            if let Some(b) = bounds.get(index) {
                self.drag_y_offset = cursor_y - b.y;
            }
        }
    }

    /// Handles drag move event and updates drop target index
    pub fn handle_drag_move(&mut self, cursor_y: f32, start_y: f32, width: f32) {
        if self.dragging_index.is_some() {
            self.drop_target_index = Some(self.calculate_drop_target(cursor_y, start_y, width));
        }
    }

    /// Handles drag end event and commits reordering
    pub fn handle_drag_end(&mut self) -> bool {
        let reordered =
            if let (Some(from_idx), Some(to_idx)) = (self.dragging_index, self.drop_target_index) {
                self.reorder_module(from_idx, to_idx)
            } else {
                false
            };

        self.dragging_index = None;
        self.drop_target_index = None;
        reordered
    }

    /// Render deterministic ASCII representation
    pub fn render_ascii(&self) -> String {
        let mut out = String::new();
        out.push_str("[DSP RACK DOCK - MODULAR AUDIO FX CHAIN]\n");
        out.push_str(&format!(
            "Master Bypass: {} | Total Modules: {}\n",
            if self.master_bypass { "ON" } else { "OFF" },
            self.modules.len()
        ));

        for (i, m) in self.modules.iter().enumerate() {
            let status = if m.is_bypassed {
                "BYPASS"
            } else if m.is_collapsed {
                "COLLAPSED"
            } else {
                "ACTIVE"
            };
            out.push_str(&format!(
                "Slot #{}: [{}] {} ({}) -- [Status: {}]\n",
                i + 1,
                m.module_type,
                m.name,
                m.id,
                status
            ));
            if !m.is_collapsed {
                out.push_str("    Params: ");
                for p in &m.params {
                    out.push_str(&format!("{}: {} | ", p.name, p.display_text));
                }
                out.push('\n');
            }
        }
        out
    }
}

#[cfg(feature = "gui")]
impl DspRackDockView {
    pub fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical(|ui| {
            // Header Bar
            ui.horizontal(|ui| {
                ui.heading("DSP RACK DOCK -- MODULAR AUDIO FX");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let bypass_btn = egui::Button::new(
                        egui::RichText::new(if self.master_bypass {
                            "Master: BYPASSED"
                        } else {
                            "Master: ACTIVE"
                        })
                        .size(13.0)
                        .strong(),
                    )
                    .min_size(Vec2::new(MIN_HIT_TARGET_PT * 2.5, MIN_HIT_TARGET_PT))
                    .fill(if self.master_bypass {
                        Color32::from_rgb(200, 50, 60)
                    } else {
                        Color32::from_rgb(35, 50, 75)
                    });

                    if ui.add(bypass_btn).clicked() {
                        self.master_bypass = !self.master_bypass;
                    }
                });
            });

            ui.add_space(8.0);

            // Rack Slots Container
            let mut module_to_remove = None;

            for (i, module) in self.modules.iter_mut().enumerate() {
                let is_dragged = self.dragging_index == Some(i);

                let frame_bg = if is_dragged {
                    Color32::from_rgb(30, 42, 65)
                } else if module.is_bypassed {
                    Color32::from_rgb(18, 22, 30)
                } else {
                    Color32::from_rgb(22, 30, 46)
                };

                let stroke_color = if is_dragged {
                    Color32::from_rgb(0, 229, 255)
                } else {
                    Color32::from_rgb(module.color.0, module.color.1, module.color.2)
                };

                egui::Frame::none()
                    .fill(frame_bg)
                    .stroke(Stroke::new(1.5_f32, stroke_color))
                    .rounding(6.0)
                    .inner_margin(egui::Margin::same(8.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Drag Handle Button (>=44x44pt)
                            let handle_btn = egui::Button::new(
                                egui::RichText::new("::")
                                    .size(16.0)
                                    .strong()
                                    .color(Color32::from_rgb(160, 180, 210)),
                            )
                            .min_size(Vec2::new(MIN_HIT_TARGET_PT, MIN_HIT_TARGET_PT))
                            .fill(Color32::from_rgb(30, 40, 60));

                            ui.add(handle_btn);

                            // Module Type & Name
                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new(&module.name)
                                        .size(14.0)
                                        .strong()
                                        .color(Color32::from_rgb(240, 245, 255)),
                                );
                                ui.label(
                                    egui::RichText::new(&module.module_type)
                                        .size(11.0)
                                        .color(Color32::from_rgb(130, 160, 200)),
                                );
                            });

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    // Delete button (>=44x44pt)
                                    let del_btn = egui::Button::new(
                                        egui::RichText::new("X")
                                            .size(13.0)
                                            .strong()
                                            .color(Color32::from_rgb(255, 120, 120)),
                                    )
                                    .min_size(Vec2::new(MIN_HIT_TARGET_PT, MIN_HIT_TARGET_PT))
                                    .fill(Color32::from_rgb(45, 25, 30));

                                    if ui.add(del_btn).clicked() {
                                        module_to_remove = Some(i);
                                    }

                                    // Collapse/Expand toggle button (>=44x44pt)
                                    let collapse_label =
                                        if module.is_collapsed { "+" } else { "-" };
                                    let collapse_btn = egui::Button::new(
                                        egui::RichText::new(collapse_label)
                                            .size(14.0)
                                            .strong()
                                            .color(Color32::from_rgb(200, 220, 250)),
                                    )
                                    .min_size(Vec2::new(MIN_HIT_TARGET_PT, MIN_HIT_TARGET_PT))
                                    .fill(Color32::from_rgb(35, 45, 65));

                                    if ui.add(collapse_btn).clicked() {
                                        module.is_collapsed = !module.is_collapsed;
                                    }

                                    // Bypass toggle button (>=44x44pt)
                                    let bypass_label =
                                        if module.is_bypassed { "OFF" } else { "ON" };
                                    let bypass_bg = if module.is_bypassed {
                                        Color32::from_rgb(60, 40, 45)
                                    } else {
                                        Color32::from_rgb(0, 180, 140)
                                    };
                                    let bypass_btn = egui::Button::new(
                                        egui::RichText::new(bypass_label)
                                            .size(13.0)
                                            .strong()
                                            .color(Color32::WHITE),
                                    )
                                    .min_size(Vec2::new(MIN_HIT_TARGET_PT, MIN_HIT_TARGET_PT))
                                    .fill(bypass_bg);

                                    if ui.add(bypass_btn).clicked() {
                                        module.is_bypassed = !module.is_bypassed;
                                    }
                                },
                            );
                        });

                        // Expanded Parameters Display
                        if !module.is_collapsed {
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                for param in &mut module.params {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new(format!("{}:", param.name))
                                                .size(12.0)
                                                .color(Color32::from_rgb(180, 200, 225)),
                                        );
                                        let slider = egui::Slider::new(&mut param.value, 0.0..=1.0)
                                            .show_value(false);
                                        ui.add(slider);
                                        param.display_text =
                                            format!("{:.0}{}", param.value * 100.0, param.unit);
                                        ui.label(
                                            egui::RichText::new(&param.display_text)
                                                .size(12.0)
                                                .color(stroke_color),
                                        );
                                    });
                                    ui.add_space(12.0);
                                }
                            });
                        }
                    });

                ui.add_space(MODULE_SPACING);
            }

            if let Some(remove_idx) = module_to_remove {
                self.remove_module(remove_idx);
            }
        })
        .response
    }
}
