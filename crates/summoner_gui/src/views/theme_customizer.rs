// Summoner DAW - Dynamic Theme Customization Panel with Live Contrast Ratio Preview Meter (Step 1303)

use serde::{Deserialize, Serialize};

/// Colorblind accessibility theme modes (Step 1305).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorblindMode {
    None,
    Protanopia,   // Red-blind / red-weak
    Deuteranopia, // Green-blind / green-weak
    Tritanopia,   // Blue-blind / blue-weak
}

impl ColorblindMode {
    pub fn display_name(&self) -> &'static str {
        match self {
            ColorblindMode::None => "Standard (Full Color)",
            ColorblindMode::Protanopia => "Protanopia (Red-Blind Safe)",
            ColorblindMode::Deuteranopia => "Deuteranopia (Green-Blind Safe)",
            ColorblindMode::Tritanopia => "Tritanopia (Blue-Blind Safe)",
        }
    }
}

/// Dynamic Theme Customizer & Accessibility View Widget (`ThemeCustomizerView`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeCustomizerView {
    pub selected_theme_idx: usize,
    pub colorblind_mode: ColorblindMode,
    pub base_font_size: f32,
    pub custom_text_rgb: (u8, u8, u8),
    pub custom_bg_rgb: (u8, u8, u8),
    pub min_hit_target: f32,
}

impl Default for ThemeCustomizerView {
    fn default() -> Self {
        Self {
            selected_theme_idx: 0,
            colorblind_mode: ColorblindMode::None,
            base_font_size: 14.0,
            custom_text_rgb: (255, 255, 255),
            custom_bg_rgb: (15, 15, 20),
            min_hit_target: 44.0,
        }
    }
}

impl ThemeCustomizerView {
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate sRGB relative luminance.
    pub fn calculate_relative_luminance(r: u8, g: u8, b: u8) -> f64 {
        fn to_lin(val: u8) -> f64 {
            let v = val as f64 / 255.0;
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * to_lin(r) + 0.7152 * to_lin(g) + 0.0722 * to_lin(b)
    }

    /// Calculate WCAG contrast ratio between text RGB and background RGB.
    pub fn calculate_contrast_ratio(&self) -> f64 {
        let (tr, tg, tb) = self.custom_text_rgb;
        let (br, bg, bb) = self.custom_bg_rgb;
        let l1 = Self::calculate_relative_luminance(tr, tg, tb);
        let l2 = Self::calculate_relative_luminance(br, bg, bb);
        let (max, min) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
        (max + 0.05) / (min + 0.05)
    }

    /// Returns true if current contrast ratio meets WCAG AA (>= 4.5:1).
    pub fn meets_wcag_aa(&self) -> bool {
        self.calculate_contrast_ratio() >= 4.5
    }

    /// Returns true if current contrast ratio meets WCAG AAA (>= 7.0:1).
    pub fn meets_wcag_aaa(&self) -> bool {
        self.calculate_contrast_ratio() >= 7.0
    }

    /// Enforce minimum hit target bounds for theme selector controls.
    pub fn control_hit_rect(
        &self,
        pos_x: f32,
        pos_y: f32,
        width: f32,
        height: f32,
    ) -> crate::layout_math::Rect {
        let rect = crate::layout_math::Rect::new(pos_x, pos_y, width, height);
        rect.enforce_min_hit_target(self.min_hit_target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wcag_contrast_ratio_calculation() {
        let customizer = ThemeCustomizerView::default();
        let ratio = customizer.calculate_contrast_ratio();
        assert!(ratio >= 15.0);
        assert!(customizer.meets_wcag_aa());
        assert!(customizer.meets_wcag_aaa());

        let low_contrast = ThemeCustomizerView {
            custom_text_rgb: (100, 100, 100),
            custom_bg_rgb: (120, 120, 120),
            ..Default::default()
        };
        assert!(!low_contrast.meets_wcag_aa());
    }

    #[test]
    fn test_colorblind_mode_names() {
        assert_eq!(ColorblindMode::None.display_name(), "Standard (Full Color)");
        assert_eq!(
            ColorblindMode::Protanopia.display_name(),
            "Protanopia (Red-Blind Safe)"
        );
        assert_eq!(
            ColorblindMode::Deuteranopia.display_name(),
            "Deuteranopia (Green-Blind Safe)"
        );
        assert_eq!(
            ColorblindMode::Tritanopia.display_name(),
            "Tritanopia (Blue-Blind Safe)"
        );
    }
}
