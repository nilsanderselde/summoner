// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Customizable Multi-Touch Macro Rotary Dials & Radial Acceleration (Step 1344).

use crate::touch_controls::{ContrastColorPalette, MIN_HIT_TARGET_PT};

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const DIAL_SWEEP_ANGLE_DEG: f32 = 270.0;
pub const DIAL_START_ANGLE_DEG: f32 = -135.0;
pub const DIAL_VISUAL_RADIUS: f32 = 28.0;
pub const DIAL_HIT_RADIUS: f32 = 32.0;

/// Dial polarity mode: Unipolar [0.0 ..= 1.0] or Bipolar [-1.0 ..= 1.0].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialMode {
    Unipolar,
    Bipolar,
}

/// A single customizable macro rotary dial state.
#[derive(Debug, Clone, PartialEq)]
pub struct MacroRotaryDialState {
    pub id: String,
    pub name: String,
    pub value: f32, // Unipolar [0..1] or Bipolar [-1..1]
    pub default_value: f32,
    pub min_display: f32,
    pub max_display: f32,
    pub unit: String,
    pub mode: DialMode,
    pub color: (u8, u8, u8),
    pub mod_depth: f32, // -1.0 ..= 1.0 modulation excursion
    pub is_dragging: bool,
}

impl MacroRotaryDialState {
    pub fn new_unipolar(
        id: impl Into<String>,
        name: impl Into<String>,
        initial_value: f32,
        min_disp: f32,
        max_disp: f32,
        unit: impl Into<String>,
        color: (u8, u8, u8),
    ) -> Self {
        let val = initial_value.clamp(0.0, 1.0);
        Self {
            id: id.into(),
            name: name.into(),
            value: val,
            default_value: val,
            min_display: min_disp,
            max_display: max_disp,
            unit: unit.into(),
            mode: DialMode::Unipolar,
            color,
            mod_depth: 0.0,
            is_dragging: false,
        }
    }

    pub fn new_bipolar(
        id: impl Into<String>,
        name: impl Into<String>,
        initial_value: f32,
        min_disp: f32,
        max_disp: f32,
        unit: impl Into<String>,
        color: (u8, u8, u8),
    ) -> Self {
        let val = initial_value.clamp(-1.0, 1.0);
        Self {
            id: id.into(),
            name: name.into(),
            value: val,
            default_value: val,
            min_display: min_disp,
            max_display: max_disp,
            unit: unit.into(),
            mode: DialMode::Bipolar,
            color,
            mod_depth: 0.0,
            is_dragging: false,
        }
    }

    /// Normalized value in range [0.0 ..= 1.0] regardless of polarity mode
    pub fn normalized_value(&self) -> f32 {
        match self.mode {
            DialMode::Unipolar => self.value.clamp(0.0, 1.0),
            DialMode::Bipolar => ((self.value + 1.0) * 0.5).clamp(0.0, 1.0),
        }
    }

    /// Convert dial angle in degrees to value
    pub fn angle_to_value(angle_deg: f32, mode: DialMode) -> f32 {
        let norm = ((angle_deg - DIAL_START_ANGLE_DEG) / DIAL_SWEEP_ANGLE_DEG).clamp(0.0, 1.0);
        match mode {
            DialMode::Unipolar => norm,
            DialMode::Bipolar => norm * 2.0 - 1.0,
        }
    }

    /// Convert value to dial angle in degrees
    pub fn value_to_angle(value: f32, mode: DialMode) -> f32 {
        let norm = match mode {
            DialMode::Unipolar => value.clamp(0.0, 1.0),
            DialMode::Bipolar => ((value + 1.0) * 0.5).clamp(0.0, 1.0),
        };
        DIAL_START_ANGLE_DEG + norm * DIAL_SWEEP_ANGLE_DEG
    }

    /// Formatted display value with unit
    pub fn display_value_string(&self) -> String {
        let norm = self.normalized_value();
        let disp_val = self.min_display + norm * (self.max_display - self.min_display);
        match self.mode {
            DialMode::Unipolar => format!("{:.1}{}", disp_val, self.unit),
            DialMode::Bipolar => {
                if disp_val > 0.0 {
                    format!("+{:.1}{}", disp_val, self.unit)
                } else {
                    format!("{:.1}{}", disp_val, self.unit)
                }
            }
        }
    }

    /// Applies vertical drag delta with fine precision gear ratio
    pub fn apply_drag_delta(&mut self, delta_y: f32, fine_precision: bool) {
        let gear_ratio: f32 = if fine_precision { 0.001 } else { 0.005 };
        let delta = -delta_y * gear_ratio;

        match self.mode {
            DialMode::Unipolar => {
                self.value = (self.value + delta).clamp(0.0, 1.0);
            }
            DialMode::Bipolar => {
                self.value = (self.value + delta * 2.0).clamp(-1.0, 1.0);
            }
        }
    }

    /// Reset to default
    pub fn reset_to_default(&mut self) {
        self.value = self.default_value;
    }
}

/// Macro Rotary Dial View Bank (Step 1344).
#[derive(Debug, Clone)]
pub struct MacroRotaryDialView {
    pub dials: Vec<MacroRotaryDialState>,
    pub fine_precision_mode: bool,
    pub color_palette: ContrastColorPalette,
}

impl Default for MacroRotaryDialView {
    fn default() -> Self {
        Self::new()
    }
}

impl MacroRotaryDialView {
    pub fn new() -> Self {
        let mut view = Self {
            dials: Vec::new(),
            fine_precision_mode: false,
            color_palette: ContrastColorPalette::default(),
        };

        // Standard 4-macro rotary dial set
        view.dials.push(MacroRotaryDialState::new_unipolar(
            "macro_cutoff",
            "Filter Cutoff",
            0.72,
            20.0,
            20000.0,
            "Hz",
            (0, 229, 255),
        ));

        let mut dial_gain = MacroRotaryDialState::new_bipolar(
            "macro_drive",
            "Drive Trim",
            0.35,
            -24.0,
            24.0,
            "dB",
            (255, 107, 43),
        );
        dial_gain.mod_depth = 0.25;
        view.dials.push(dial_gain);

        view.dials.push(MacroRotaryDialState::new_unipolar(
            "macro_res",
            "Resonance Q",
            0.45,
            0.1,
            18.0,
            "Q",
            (255, 215, 0),
        ));

        view.dials.push(MacroRotaryDialState::new_bipolar(
            "macro_pan",
            "Stereo Pan",
            -0.20,
            -100.0,
            100.0,
            "%",
            (140, 90, 255),
        ));

        view
    }

    /// Render deterministic ASCII representation
    pub fn render_ascii(&self) -> String {
        let mut out = String::new();
        out.push_str("[MULTI-TOUCH MACRO ROTARY DIAL BANK]\n");
        out.push_str(&format!(
            "Fine Precision (0.1x): {} | Dial Count: {}\n",
            if self.fine_precision_mode {
                "ON"
            } else {
                "OFF"
            },
            self.dials.len()
        ));

        for (i, d) in self.dials.iter().enumerate() {
            let angle = MacroRotaryDialState::value_to_angle(d.value, d.mode);
            out.push_str(&format!(
                "Dial #{}: [{:?}] '{}' | Val: {:.2} ({}) | Angle: {:.1} deg | Mod Depth: {:.0}%\n",
                i + 1,
                d.mode,
                d.name,
                d.value,
                d.display_value_string(),
                angle,
                d.mod_depth * 100.0
            ));
        }
        out
    }
}

#[cfg(feature = "gui")]
impl MacroRotaryDialView {
    pub fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical(|ui| {
            // Header Bar
            ui.horizontal(|ui| {
                ui.heading("MULTI-TOUCH MACRO ROTARY DIALS");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let fine_label = if self.fine_precision_mode {
                        "Fine Mode (0.1x): ON"
                    } else {
                        "Fine Mode: OFF (Hold Shift)"
                    };
                    let fine_btn = egui::Button::new(
                        egui::RichText::new(fine_label).size(13.0).strong().color(
                            if self.fine_precision_mode {
                                Color32::BLACK
                            } else {
                                Color32::WHITE
                            },
                        ),
                    )
                    .min_size(Vec2::new(MIN_HIT_TARGET_PT * 3.5, MIN_HIT_TARGET_PT))
                    .fill(if self.fine_precision_mode {
                        Color32::from_rgb(0, 229, 255)
                    } else {
                        Color32::from_rgb(35, 50, 75)
                    });

                    if ui.add(fine_btn).clicked() {
                        self.fine_precision_mode = !self.fine_precision_mode;
                    }
                });
            });

            ui.add_space(10.0);

            // Rotary Dials Display Row
            ui.horizontal(|ui| {
                for dial in &mut self.dials {
                    let dial_color = Color32::from_rgb(dial.color.0, dial.color.1, dial.color.2);

                    egui::Frame::none()
                        .fill(Color32::from_rgb(20, 26, 40))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(45, 55, 75)))
                        .rounding(8.0)
                        .inner_margin(egui::Margin::same(12.0))
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    egui::RichText::new(&dial.name)
                                        .size(13.0)
                                        .strong()
                                        .color(Color32::from_rgb(240, 245, 255)),
                                );

                                ui.add_space(6.0);

                                // Rotary Knob Canvas Element
                                let (rect, response) = ui.allocate_exact_size(
                                    Vec2::new(DIAL_HIT_RADIUS * 2.0, DIAL_HIT_RADIUS * 2.0),
                                    egui::Sense::drag(),
                                );

                                if response.dragged() {
                                    dial.apply_drag_delta(
                                        response.drag_delta().y,
                                        self.fine_precision_mode,
                                    );
                                }

                                if response.double_clicked() {
                                    dial.reset_to_default();
                                }

                                if ui.is_rect_visible(rect) {
                                    let painter = ui.painter();
                                    let center = rect.center();

                                    // Outer background track circle
                                    painter.circle(
                                        center,
                                        DIAL_VISUAL_RADIUS,
                                        Color32::from_rgb(12, 16, 26),
                                        Stroke::new(2.0_f32, Color32::from_rgb(40, 55, 80)),
                                    );

                                    // Value Arc indicator
                                    let angle_rad =
                                        MacroRotaryDialState::value_to_angle(dial.value, dial.mode)
                                            .to_radians();
                                    let tip = center
                                        + Vec2::new(
                                            angle_rad.sin() * (DIAL_VISUAL_RADIUS - 4.0),
                                            -angle_rad.cos() * (DIAL_VISUAL_RADIUS - 4.0),
                                        );

                                    painter.line_segment(
                                        [center, tip],
                                        Stroke::new(3.0_f32, dial_color),
                                    );

                                    painter.circle_filled(tip, 3.5, Color32::WHITE);
                                }

                                ui.add_space(6.0);

                                ui.label(
                                    egui::RichText::new(dial.display_value_string())
                                        .size(13.0)
                                        .strong()
                                        .color(dial_color),
                                );
                            });
                        });

                    ui.add_space(12.0);
                }
            });
        })
        .response
    }
}
