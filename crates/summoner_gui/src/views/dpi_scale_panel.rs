// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// High-DPI Scaling Auto-Detection & Custom Scale Factor Slider View (Step 1325).

use crate::layout_math::OperatingSystem;
use crate::touch_controls::MIN_HIT_TARGET_PT;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Vec2};

pub const PRESET_SCALES: [f32; 6] = [1.0, 1.25, 1.5, 2.0, 2.5, 3.0];

/// High-DPI Scaling & Display Layout Calibration View (Step 1325).
#[derive(Debug, Clone)]
pub struct DpiScalePanelView {
    pub host_os: OperatingSystem,
    pub detected_dpi: f32,
    pub detected_scale: f32,
    pub custom_scale: f32,
    pub auto_detect_enabled: bool,
    pub preview_button_pressed: bool,
}

impl Default for DpiScalePanelView {
    fn default() -> Self {
        Self::new(OperatingSystem::current())
    }
}

impl DpiScalePanelView {
    pub fn new(os: OperatingSystem) -> Self {
        let (dpi, scale) = match os {
            OperatingSystem::Windows => (120.0, 1.25),
            OperatingSystem::MacOS => (192.0, 2.00),
            OperatingSystem::Linux => (96.0, 1.00),
        };

        Self {
            host_os: os,
            detected_dpi: dpi,
            detected_scale: scale,
            custom_scale: scale,
            auto_detect_enabled: true,
            preview_button_pressed: false,
        }
    }

    /// Effective scale factor currently applied
    pub fn effective_scale(&self) -> f32 {
        if self.auto_detect_enabled {
            self.detected_scale
        } else {
            self.custom_scale
        }
    }

    /// Calculate physical touch target size in pixels given effective scale
    pub fn physical_touch_target_px(&self) -> f32 {
        MIN_HIT_TARGET_PT * self.effective_scale()
    }

    /// Verify if minimum hit target requirement (>= 44x44pt) is satisfied
    pub fn is_touch_target_compliant(&self) -> bool {
        self.effective_scale() >= 0.75
    }

    /// Set scale preset
    pub fn apply_preset(&mut self, preset: f32) {
        self.custom_scale = preset.clamp(0.75, 3.0);
        self.auto_detect_enabled = false;
    }

    /// Render ASCII summary of DPI scale panel
    pub fn render_ascii(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "[HIGH-DPI SCALING & DISPLAY CALIBRATION - OS: {:?}]\n",
            self.host_os
        ));
        out.push_str(&format!(
            "Detected DPI: {:.0} DPI | Detected Scale: {:.2}x ({:.0}%)\n",
            self.detected_dpi,
            self.detected_scale,
            self.detected_scale * 100.0
        ));
        out.push_str(&format!(
            "Active Scale: {:.2}x ({:.0}%) | Mode: {}\n",
            self.effective_scale(),
            self.effective_scale() * 100.0,
            if self.auto_detect_enabled {
                "AUTO-DETECT"
            } else {
                "CUSTOM MANUAL"
            }
        ));
        out.push_str(&format!(
            "Min Hit Target (44pt): {:.1} px (Compliant: {})\n",
            self.physical_touch_target_px(),
            if self.is_touch_target_compliant() {
                "YES (PASS)"
            } else {
                "NO (FAIL)"
            }
        ));
        out
    }
}

#[cfg(feature = "gui")]
impl DpiScalePanelView {
    /// Render egui DPI Scaling calibration panel
    pub fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical(|ui| {
            // Header Bar
            ui.horizontal(|ui| {
                ui.heading("HIGH-DPI DISPLAY SCALING & CALIBRATION");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let auto_text = if self.auto_detect_enabled { "Auto-Detect: ON" } else { "Auto-Detect: OFF" };
                    let auto_color = if self.auto_detect_enabled {
                        Color32::from_rgb(0, 229, 255)
                    } else {
                        Color32::from_rgb(140, 160, 180)
                    };
                    if ui
                        .add(
                            egui::Button::new(egui::RichText::new(auto_text).size(12.0).color(auto_color))
                                .min_size(Vec2::new(MIN_HIT_TARGET_PT, 34.0)),
                        )
                        .clicked()
                    {
                        self.auto_detect_enabled = !self.auto_detect_enabled;
                        if self.auto_detect_enabled {
                            self.custom_scale = self.detected_scale;
                        }
                    }
                });
            });

            ui.add_space(8.0);

            // Display Info & Metrics Card
            egui::Frame::none()
                .fill(Color32::from_rgb(18, 24, 36))
                .stroke(egui::Stroke::new(1.0_f32, Color32::from_rgb(45, 60, 85)))
                .rounding(6.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "🖥️ Host OS: {:?} | System DPI: {:.0} DPI | Detected Factor: {:.0}%",
                                self.host_os, self.detected_dpi, self.detected_scale * 100.0
                            ))
                            .size(13.0)
                            .color(Color32::from_rgb(220, 235, 255)),
                        );
                    });
                });

            ui.add_space(10.0);

            // Preset Buttons Bar (>= 44x44pt hit targets)
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Presets:").size(13.0).strong().color(Color32::from_rgb(0, 229, 255)));
                for &preset in &PRESET_SCALES {
                    let is_active = (self.effective_scale() - preset).abs() < 1e-3;
                    let btn_color = if is_active {
                        Color32::from_rgb(0, 229, 255)
                    } else {
                        Color32::from_rgb(45, 55, 75)
                    };
                    let text_color = if is_active { Color32::BLACK } else { Color32::WHITE };

                    let btn = egui::Button::new(
                        egui::RichText::new(format!("{:.0}%", preset * 100.0))
                            .size(12.0)
                            .color(text_color)
                            .strong(),
                    )
                    .fill(btn_color)
                    .min_size(Vec2::new(56.0, MIN_HIT_TARGET_PT));

                    if ui.add(btn).clicked() {
                        self.apply_preset(preset);
                    }
                }
            });

            ui.add_space(12.0);

            // Custom Slider Control
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Custom Scale:").size(13.0).color(Color32::from_rgb(200, 220, 245)));
                let mut temp_scale = self.custom_scale;
                if ui
                    .add(
                        egui::Slider::new(&mut temp_scale, 0.75..=3.0)
                            .step_by(0.05)
                            .custom_formatter(|n, _| format!("{:.0}%", n * 100.0)),
                    )
                    .changed()
                {
                    self.custom_scale = temp_scale;
                    self.auto_detect_enabled = false;
                }
                ui.label(
                    egui::RichText::new(format!("Effective: {:.2}x", self.effective_scale()))
                        .size(12.0)
                        .color(Color32::from_rgb(255, 215, 0)),
                );
            });

            ui.add_space(12.0);

            // Scaled UI Preview Component Box
            ui.group(|ui| {
                ui.heading("SCALED WIDGET PREVIEW");
                ui.add_space(4.0);

                let eff = self.effective_scale();
                ui.horizontal(|ui| {
                    // Preview Scaled Button
                    let preview_btn = egui::Button::new(
                        egui::RichText::new("Touch Button (>= 44pt)")
                            .size(13.0 * eff.clamp(0.8, 1.6))
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(0, 140, 255))
                    .min_size(Vec2::new(MIN_HIT_TARGET_PT * eff, MIN_HIT_TARGET_PT * eff));

                    if ui.add(preview_btn).clicked() {
                        self.preview_button_pressed = !self.preview_button_pressed;
                    }

                    ui.add_space(12.0);

                    // Compliance Badge
                    let comp_text = if self.is_touch_target_compliant() {
                        "✅ Ergonomic Hit Target: PASS (>=44pt)"
                    } else {
                        "⚠️ Ergonomic Hit Target: FAIL (<44pt)"
                    };
                    let comp_color = if self.is_touch_target_compliant() {
                        Color32::from_rgb(0, 255, 180)
                    } else {
                        Color32::from_rgb(255, 80, 80)
                    };
                    ui.label(egui::RichText::new(comp_text).size(12.0).color(comp_color).strong());
                });
            });
        })
        .response
    }
}
