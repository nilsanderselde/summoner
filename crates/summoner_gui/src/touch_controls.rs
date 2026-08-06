// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Modular ergonomic touch-friendly control widgets (`TouchKnob`, `TouchSlider`, `TouchToggle`).
//! Minimum hit target dimensions enforced at 44x44pt.
//! High visual contrast colors (WCAG AA/AAA compliant >4.5:1 ratio), active glow/hover highlight rings,
//! tactile drag sensitivity scaling, cross-platform DPI scale adaptation.

use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
use eframe::egui;

/// Minimum touch hit target size in logical points (WCAG / Apple / Android touch standard).
pub const MIN_HIT_TARGET_PT: f32 = 44.0;

/// Default WCAG AA/AAA compliant color palette for high visual contrast touch widgets.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContrastColorPalette {
    /// Background color (R, G, B)
    pub bg_rgb: (u8, u8, u8),
    /// Active fill color (R, G, B)
    pub active_fill_rgb: (u8, u8, u8),
    /// Foreground / Label color (R, G, B)
    pub text_rgb: (u8, u8, u8),
    /// Glow / Highlight ring color (R, G, B, A)
    pub glow_rgba: (u8, u8, u8, u8),
    /// Track / border color (R, G, B)
    pub border_rgb: (u8, u8, u8),
}

impl Default for ContrastColorPalette {
    fn default() -> Self {
        Self {
            bg_rgb: (24, 28, 40),           // Dark charcoal background
            active_fill_rgb: (0, 229, 255), // Vibrant Cyan (>8:1 contrast against bg)
            text_rgb: (245, 247, 250),      // Near-white text (>13:1 contrast against bg)
            glow_rgba: (0, 229, 255, 120),  // High-visibility active glow
            border_rgb: (60, 75, 100),      // High contrast border
        }
    }
}

impl ContrastColorPalette {
    /// Calculates relative luminance for WCAG contrast ratio evaluation.
    pub fn luminance(rgb: (u8, u8, u8)) -> f32 {
        let convert = |c: u8| {
            let s = c as f32 / 255.0;
            if s <= 0.03928 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * convert(rgb.0) + 0.7152 * convert(rgb.1) + 0.0722 * convert(rgb.2)
    }

    /// Calculates WCAG contrast ratio between two colors (returns value >= 1.0).
    pub fn contrast_ratio(c1: (u8, u8, u8), c2: (u8, u8, u8)) -> f32 {
        let l1 = Self::luminance(c1);
        let l2 = Self::luminance(c2);
        let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
        (lighter + 0.05) / (darker + 0.05)
    }

    /// Verifies if text-to-background contrast ratio meets WCAG AA standard (>4.5:1).
    pub fn is_wcag_aa_compliant(&self) -> bool {
        Self::contrast_ratio(self.text_rgb, self.bg_rgb) >= 4.5
    }

    /// Verifies if text-to-background contrast ratio meets WCAG AAA standard (>7.0:1).
    pub fn is_wcag_aaa_compliant(&self) -> bool {
        Self::contrast_ratio(self.text_rgb, self.bg_rgb) >= 7.0
    }
}

/// Metric calculation helper for cross-platform DPI scaling and hit target size adaptation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TouchWidgetMetrics {
    pub dpi_scale: f32,
    pub base_width: f32,
    pub base_height: f32,
    pub drag_sensitivity: f32,
}

impl Default for TouchWidgetMetrics {
    fn default() -> Self {
        Self {
            dpi_scale: 1.0,
            base_width: MIN_HIT_TARGET_PT,
            base_height: MIN_HIT_TARGET_PT,
            drag_sensitivity: 1.0,
        }
    }
}

impl TouchWidgetMetrics {
    pub fn new(width: f32, height: f32, dpi_scale: f32) -> Self {
        let mut metrics = Self {
            dpi_scale: dpi_scale.max(0.1),
            base_width: width,
            base_height: height,
            drag_sensitivity: 1.0,
        };
        metrics.enforce_min_size();
        metrics
    }

    /// Enforces minimum 44.0pt hit target dimensions in logical points.
    pub fn enforce_min_size(&mut self) {
        self.base_width = self.base_width.max(MIN_HIT_TARGET_PT);
        self.base_height = self.base_height.max(MIN_HIT_TARGET_PT);
    }

    /// Calculates physical pixel dimensions adapted for high-DPI scaling.
    pub fn scaled_dimensions(&self) -> (f32, f32) {
        (
            (self.base_width * self.dpi_scale).max(MIN_HIT_TARGET_PT),
            (self.base_height * self.dpi_scale).max(MIN_HIT_TARGET_PT),
        )
    }

    /// Calculates scaled drag delta given raw input delta and fine control modifier.
    pub fn scale_drag(&self, raw_delta: f32, fine_control: bool) -> f32 {
        let modifier = if fine_control { 0.1 } else { 1.0 };
        raw_delta * self.drag_sensitivity * modifier / self.dpi_scale.max(0.5)
    }
}

/// Standalone state for rotary knob controls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnobState {
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub default_value: f32,
    pub step: Option<f32>,
    pub sensitivity: f32,
    pub label: String,
    pub unit_suffix: String,
}

impl KnobState {
    pub fn new(value: f32, min: f32, max: f32) -> Self {
        let mut state = Self {
            value,
            min,
            max,
            default_value: value,
            step: None,
            sensitivity: 0.005,
            label: String::new(),
            unit_suffix: String::new(),
        };
        state.clamp_value();
        state
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn with_unit(mut self, unit: impl Into<String>) -> Self {
        self.unit_suffix = unit.into();
        self
    }

    pub fn with_step(mut self, step: f32) -> Self {
        self.step = Some(step.abs());
        self.clamp_value();
        self
    }

    pub fn with_default(mut self, default_val: f32) -> Self {
        self.default_value = default_val.clamp(self.min.min(self.max), self.min.max(self.max));
        self
    }

    pub fn reset_to_default(&mut self) {
        self.value = self.default_value;
        self.clamp_value();
    }

    pub fn clamp_value(&mut self) {
        let min = self.min.min(self.max);
        let max = self.min.max(self.max);
        let mut val = self.value.clamp(min, max);
        if let Some(step) = self.step {
            if step > 0.0 {
                val = min + ((val - min) / step).round() * step;
                val = val.clamp(min, max);
            }
        }
        self.value = val;
    }

    /// Converts current value to normalized ratio in range [0.0, 1.0].
    pub fn normalized(&self) -> f32 {
        let range = self.max - self.min;
        if range.abs() < 1e-7 {
            0.0
        } else {
            ((self.value - self.min) / range).clamp(0.0, 1.0)
        }
    }

    /// Sets value from normalized ratio in range [0.0, 1.0].
    pub fn set_normalized(&mut self, norm: f32) {
        let norm = norm.clamp(0.0, 1.0);
        self.value = self.min + norm * (self.max - self.min);
        self.clamp_value();
    }

    /// Updates value by vertical drag delta (up increases, down decreases).
    pub fn drag_update(&mut self, delta_y: f32, fine_control: bool) {
        let sensitivity = if fine_control {
            self.sensitivity * 0.1
        } else {
            self.sensitivity
        };
        let range = self.max - self.min;
        let delta_val = -delta_y * sensitivity * range;
        self.value += delta_val;
        self.clamp_value();
    }
}

/// Orientation mode for slider controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SliderOrientation {
    Vertical,
    Horizontal,
}

/// Standalone state for slider controls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SliderState {
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub default_value: f32,
    pub orientation: SliderOrientation,
    pub step: Option<f32>,
    pub sensitivity: f32,
    pub label: String,
    pub unit_suffix: String,
}

impl SliderState {
    pub fn new(value: f32, min: f32, max: f32, orientation: SliderOrientation) -> Self {
        let mut state = Self {
            value,
            min,
            max,
            default_value: value,
            orientation,
            step: None,
            sensitivity: 0.005,
            label: String::new(),
            unit_suffix: String::new(),
        };
        state.clamp_value();
        state
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn with_unit(mut self, unit: impl Into<String>) -> Self {
        self.unit_suffix = unit.into();
        self
    }

    pub fn with_step(mut self, step: f32) -> Self {
        self.step = Some(step.abs());
        self.clamp_value();
        self
    }

    pub fn with_default(mut self, default_val: f32) -> Self {
        self.default_value = default_val.clamp(self.min.min(self.max), self.min.max(self.max));
        self
    }

    pub fn reset_to_default(&mut self) {
        self.value = self.default_value;
        self.clamp_value();
    }

    pub fn clamp_value(&mut self) {
        let min = self.min.min(self.max);
        let max = self.min.max(self.max);
        let mut val = self.value.clamp(min, max);
        if let Some(step) = self.step {
            if step > 0.0 {
                val = min + ((val - min) / step).round() * step;
                val = val.clamp(min, max);
            }
        }
        self.value = val;
    }

    pub fn normalized(&self) -> f32 {
        let range = self.max - self.min;
        if range.abs() < 1e-7 {
            0.0
        } else {
            ((self.value - self.min) / range).clamp(0.0, 1.0)
        }
    }

    pub fn set_normalized(&mut self, norm: f32) {
        let norm = norm.clamp(0.0, 1.0);
        self.value = self.min + norm * (self.max - self.min);
        self.clamp_value();
    }

    pub fn drag_update(&mut self, delta: f32, fine_control: bool) {
        let sensitivity = if fine_control {
            self.sensitivity * 0.1
        } else {
            self.sensitivity
        };
        let range = self.max - self.min;
        let factor = match self.orientation {
            SliderOrientation::Vertical => -1.0,
            SliderOrientation::Horizontal => 1.0,
        };
        let delta_val = delta * factor * sensitivity * range;
        self.value += delta_val;
        self.clamp_value();
    }
}

/// Standalone state for touch toggle buttons/switches.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToggleState {
    pub active: bool,
    pub label: String,
}

impl ToggleState {
    pub fn new(active: bool) -> Self {
        Self {
            active,
            label: String::new(),
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn toggle(&mut self) -> bool {
        self.active = !self.active;
        self.active
    }

    pub fn set(&mut self, active: bool) {
        self.active = active;
    }
}

/// Touch-friendly ergonomic knob GUI component.
#[cfg(feature = "gui")]
pub struct TouchKnob;

#[cfg(feature = "gui")]
impl TouchKnob {
    pub fn show(ui: &mut egui::Ui, state: &mut KnobState) -> egui::Response {
        let dpi_scale = ui.ctx().pixels_per_point();
        let metrics = TouchWidgetMetrics::new(MIN_HIT_TARGET_PT, MIN_HIT_TARGET_PT, dpi_scale);
        let (width, height) = (metrics.base_width, metrics.base_height);

        let (rect, mut response) =
            ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::click_and_drag());

        if response.double_clicked() {
            state.reset_to_default();
            response.mark_changed();
        } else if response.dragged() {
            let delta = response.drag_delta();
            let fine = ui.input(|i| i.modifiers.shift);
            state.drag_update(delta.y, fine);
            response.mark_changed();
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter_at(rect);
            let palette = ContrastColorPalette::default();

            let center = rect.center();
            let radius = (width.min(height) * 0.4).max(18.0);

            // Active glow highlight ring on hover or drag
            if response.hovered() || response.dragged() {
                let glow_col = egui::Color32::from_rgba_unmultiplied(
                    palette.glow_rgba.0,
                    palette.glow_rgba.1,
                    palette.glow_rgba.2,
                    palette.glow_rgba.3,
                );
                painter.circle_stroke(center, radius + 3.0, egui::Stroke::new(2.5_f32, glow_col));
            }

            // Knob background circle
            let bg_col =
                egui::Color32::from_rgb(palette.bg_rgb.0, palette.bg_rgb.1, palette.bg_rgb.2);
            let border_col = egui::Color32::from_rgb(
                palette.border_rgb.0,
                palette.border_rgb.1,
                palette.border_rgb.2,
            );
            painter.circle_filled(center, radius, bg_col);
            painter.circle_stroke(center, radius, egui::Stroke::new(1.5_f32, border_col));

            // Indicator line angle (240 deg arc from 135 deg to 405 deg)
            let norm = state.normalized();
            let start_angle = std::f32::consts::PI * 0.75;
            let current_angle = start_angle + std::f32::consts::PI * 1.5 * norm;

            let fill_col = egui::Color32::from_rgb(
                palette.active_fill_rgb.0,
                palette.active_fill_rgb.1,
                palette.active_fill_rgb.2,
            );

            // Pointer line
            let pointer_x = center.x + radius * 0.85 * current_angle.cos();
            let pointer_y = center.y + radius * 0.85 * current_angle.sin();
            painter.line_segment(
                [center, egui::pos2(pointer_x, pointer_y)],
                egui::Stroke::new(2.5_f32, fill_col),
            );

            // Label text
            if !state.label.is_empty() {
                let text_col = egui::Color32::from_rgb(
                    palette.text_rgb.0,
                    palette.text_rgb.1,
                    palette.text_rgb.2,
                );
                painter.text(
                    egui::pos2(center.x, rect.bottom() - 2.0),
                    egui::Align2::CENTER_BOTTOM,
                    &state.label,
                    egui::FontId::proportional(10.0),
                    text_col,
                );
            }
        }

        response
    }
}

/// Touch-friendly ergonomic slider GUI component.
#[cfg(feature = "gui")]
pub struct TouchSlider;

#[cfg(feature = "gui")]
impl TouchSlider {
    pub fn show(ui: &mut egui::Ui, state: &mut SliderState) -> egui::Response {
        let dpi_scale = ui.ctx().pixels_per_point();
        let (req_w, req_h) = match state.orientation {
            SliderOrientation::Vertical => (MIN_HIT_TARGET_PT, 120.0),
            SliderOrientation::Horizontal => (120.0, MIN_HIT_TARGET_PT),
        };
        let metrics = TouchWidgetMetrics::new(req_w, req_h, dpi_scale);
        let (width, height) = (metrics.base_width, metrics.base_height);

        let (rect, mut response) =
            ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::click_and_drag());

        if response.double_clicked() {
            state.reset_to_default();
            response.mark_changed();
        } else if response.dragged() {
            let delta = response.drag_delta();
            let raw_delta = match state.orientation {
                SliderOrientation::Vertical => delta.y,
                SliderOrientation::Horizontal => delta.x,
            };
            let fine = ui.input(|i| i.modifiers.shift);
            state.drag_update(raw_delta, fine);
            response.mark_changed();
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter_at(rect);
            let palette = ContrastColorPalette::default();

            let bg_col =
                egui::Color32::from_rgb(palette.bg_rgb.0, palette.bg_rgb.1, palette.bg_rgb.2);
            let border_col = egui::Color32::from_rgb(
                palette.border_rgb.0,
                palette.border_rgb.1,
                palette.border_rgb.2,
            );
            let fill_col = egui::Color32::from_rgb(
                palette.active_fill_rgb.0,
                palette.active_fill_rgb.1,
                palette.active_fill_rgb.2,
            );

            // Active glow highlight ring on hover or drag
            if response.hovered() || response.dragged() {
                let glow_col = egui::Color32::from_rgba_unmultiplied(
                    palette.glow_rgba.0,
                    palette.glow_rgba.1,
                    palette.glow_rgba.2,
                    palette.glow_rgba.3,
                );
                painter.rect_stroke(rect.expand(2.0), 6.0, egui::Stroke::new(2.5_f32, glow_col));
            }

            // Track background
            painter.rect_filled(rect, 4.0, bg_col);
            painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0_f32, border_col));

            let norm = state.normalized();
            match state.orientation {
                SliderOrientation::Vertical => {
                    let fill_h = rect.height() * norm;
                    let fill_rect = egui::Rect::from_min_max(
                        egui::pos2(rect.min.x + 4.0, rect.max.y - fill_h),
                        egui::pos2(rect.max.x - 4.0, rect.max.y - 2.0),
                    );
                    painter.rect_filled(fill_rect, 2.0, fill_col);
                }
                SliderOrientation::Horizontal => {
                    let fill_w = rect.width() * norm;
                    let fill_rect = egui::Rect::from_min_max(
                        egui::pos2(rect.min.x + 2.0, rect.min.y + 4.0),
                        egui::pos2(rect.min.x + fill_w, rect.max.y - 4.0),
                    );
                    painter.rect_filled(fill_rect, 2.0, fill_col);
                }
            }

            if !state.label.is_empty() {
                let text_col = egui::Color32::from_rgb(
                    palette.text_rgb.0,
                    palette.text_rgb.1,
                    palette.text_rgb.2,
                );
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    &state.label,
                    egui::FontId::proportional(11.0),
                    text_col,
                );
            }
        }

        response
    }
}

/// Touch-friendly toggle button/switch GUI component.
#[cfg(feature = "gui")]
pub struct TouchToggle;

#[cfg(feature = "gui")]
impl TouchToggle {
    pub fn show(ui: &mut egui::Ui, state: &mut ToggleState) -> egui::Response {
        let dpi_scale = ui.ctx().pixels_per_point();
        let metrics =
            TouchWidgetMetrics::new(MIN_HIT_TARGET_PT * 1.5, MIN_HIT_TARGET_PT, dpi_scale);
        let (width, height) = (metrics.base_width, metrics.base_height);

        let (rect, mut response) =
            ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::click());

        if response.clicked() {
            state.toggle();
            response.mark_changed();
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter_at(rect);
            let palette = ContrastColorPalette::default();

            let active = state.active;
            let bg_col = if active {
                egui::Color32::from_rgb(
                    palette.active_fill_rgb.0,
                    palette.active_fill_rgb.1,
                    palette.active_fill_rgb.2,
                )
            } else {
                egui::Color32::from_rgb(palette.bg_rgb.0, palette.bg_rgb.1, palette.bg_rgb.2)
            };

            let border_col = egui::Color32::from_rgb(
                palette.border_rgb.0,
                palette.border_rgb.1,
                palette.border_rgb.2,
            );

            // Glow on hover
            if response.hovered() {
                let glow_col = egui::Color32::from_rgba_unmultiplied(
                    palette.glow_rgba.0,
                    palette.glow_rgba.1,
                    palette.glow_rgba.2,
                    palette.glow_rgba.3,
                );
                painter.rect_stroke(
                    rect.expand(2.0),
                    height * 0.5,
                    egui::Stroke::new(2.5_f32, glow_col),
                );
            }

            painter.rect_filled(rect, height * 0.5, bg_col);
            painter.rect_stroke(rect, height * 0.5, egui::Stroke::new(1.5_f32, border_col));

            // Sliding thumb
            let thumb_radius = height * 0.4;
            let thumb_x = if active {
                rect.max.x - thumb_radius - 4.0
            } else {
                rect.min.x + thumb_radius + 4.0
            };
            let thumb_center = egui::pos2(thumb_x, rect.center().y);
            let thumb_col =
                egui::Color32::from_rgb(palette.text_rgb.0, palette.text_rgb.1, palette.text_rgb.2);
            painter.circle_filled(thumb_center, thumb_radius, thumb_col);
        }

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wcag_contrast_compliance() {
        let palette = ContrastColorPalette::default();
        assert!(
            palette.is_wcag_aa_compliant(),
            "Palette must meet WCAG AA ratio > 4.5:1"
        );
        assert!(
            palette.is_wcag_aaa_compliant(),
            "Palette must meet WCAG AAA ratio > 7.0:1"
        );
        let contrast = ContrastColorPalette::contrast_ratio(palette.text_rgb, palette.bg_rgb);
        assert!(
            contrast > 4.5,
            "Text contrast ratio should be > 4.5, got {}",
            contrast
        );
    }

    #[test]
    fn test_touch_widget_metrics_min_hit_target() {
        let metrics = TouchWidgetMetrics::new(20.0, 30.0, 1.0);
        assert_eq!(metrics.base_width, MIN_HIT_TARGET_PT);
        assert_eq!(metrics.base_height, MIN_HIT_TARGET_PT);

        let (scaled_w, scaled_h) = metrics.scaled_dimensions();
        assert!(scaled_w >= MIN_HIT_TARGET_PT);
        assert!(scaled_h >= MIN_HIT_TARGET_PT);
    }

    #[test]
    fn test_knob_state_bounds_clamping_and_normalization() {
        let mut knob = KnobState::new(50.0, 0.0, 100.0).with_default(50.0);
        assert_eq!(knob.value, 50.0);
        assert_eq!(knob.normalized(), 0.5);

        knob.set_normalized(0.8);
        assert_eq!(knob.value, 80.0);

        knob.drag_update(10.0, false); // Drag down decreases value
        assert!(knob.value < 80.0);

        knob.drag_update(-500.0, false); // Drag up increases value
        assert_eq!(knob.value, 100.0); // Clamped at max

        knob.reset_to_default();
        assert_eq!(knob.value, 50.0);
    }

    #[test]
    fn test_knob_state_step_discretization() {
        let mut knob = KnobState::new(0.0, 0.0, 10.0).with_step(2.5);
        knob.value = 3.1;
        knob.clamp_value();
        assert_eq!(knob.value, 2.5);

        knob.value = 4.0;
        knob.clamp_value();
        assert_eq!(knob.value, 5.0);
    }

    #[test]
    fn test_slider_state_operations() {
        let mut slider =
            SliderState::new(0.0, -12.0, 12.0, SliderOrientation::Vertical).with_default(0.0);
        assert_eq!(slider.normalized(), 0.5);

        slider.drag_update(-10.0, false); // Drag up increases vertical slider
        assert!(slider.value > 0.0);

        slider.set_normalized(1.0);
        assert_eq!(slider.value, 12.0);

        slider.set_normalized(-0.5); // Clamped to 0.0 -> min (-12.0)
        assert_eq!(slider.value, -12.0);
    }

    #[test]
    fn test_toggle_state() {
        let mut toggle = ToggleState::new(false).with_label("Bypass");
        assert!(!toggle.active);
        assert_eq!(toggle.label, "Bypass");

        let new_state = toggle.toggle();
        assert!(new_state);
        assert!(toggle.active);

        toggle.set(false);
        assert!(!toggle.active);
    }
}
