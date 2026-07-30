// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, FontDefinitions, Rounding, Visuals};

#[cfg(feature = "gui")]
pub const COLOR_PRIMARY: Color32 = Color32::from_rgb(26, 140, 255); // #1a8cff (electric blue)
#[cfg(feature = "gui")]
pub const COLOR_ACCENT: Color32 = Color32::from_rgb(255, 107, 43); // #ff6b2b (orange)
#[cfg(feature = "gui")]
pub const COLOR_BG: Color32 = Color32::from_rgb(15, 15, 20); // #0f0f14 (near-black)

#[cfg(feature = "gui")]
pub const COLOR_HIGH_CONTRAST_BG: Color32 = Color32::from_rgb(0, 0, 0); // Pure black
#[cfg(feature = "gui")]
pub const COLOR_HIGH_CONTRAST_PANEL: Color32 = Color32::from_rgb(10, 10, 12);
#[cfg(feature = "gui")]
pub const COLOR_HIGH_CONTRAST_TEXT: Color32 = Color32::from_rgb(255, 255, 255); // Pure white
#[cfg(feature = "gui")]
pub const COLOR_HIGH_CONTRAST_FOCUS: Color32 = Color32::from_rgb(0, 255, 255); // Cyan focus ring

#[cfg(feature = "gui")]
pub fn relative_luminance(c: Color32) -> f64 {
    fn srgb_to_lin(val: u8) -> f64 {
        let v = val as f64 / 255.0;
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }
    let r = srgb_to_lin(c.r());
    let g = srgb_to_lin(c.g());
    let b = srgb_to_lin(c.b());
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

#[cfg(feature = "gui")]
pub fn contrast_ratio(c1: Color32, c2: Color32) -> f64 {
    let l1 = relative_luminance(c1);
    let l2 = relative_luminance(c2);
    let (l_max, l_min) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
    (l_max + 0.05) / (l_min + 0.05)
}

#[cfg(feature = "gui")]
pub fn meets_wcag_aa(text_color: Color32, bg_color: Color32) -> bool {
    contrast_ratio(text_color, bg_color) >= 4.5
}

#[cfg(feature = "gui")]
pub fn update_font_styles(ctx: &egui::Context, font_size: f32) {
    let size = font_size.clamp(10.0, 24.0);
    let mut style = (*ctx.style()).clone();
    style.text_styles.insert(egui::TextStyle::Small, egui::FontId::proportional(size * 0.85));
    style.text_styles.insert(egui::TextStyle::Body, egui::FontId::proportional(size));
    style.text_styles.insert(egui::TextStyle::Button, egui::FontId::proportional(size));
    style.text_styles.insert(egui::TextStyle::Heading, egui::FontId::proportional(size * 1.4));
    style.text_styles.insert(egui::TextStyle::Monospace, egui::FontId::monospace(size));
    ctx.set_style(style);
}

#[cfg(feature = "gui")]
pub fn apply_summoner_theme(ctx: &egui::Context, font_size: f32) {
    let mut visuals = Visuals::dark();

    // Custom dark theme colors
    visuals.panel_fill = COLOR_BG;
    visuals.window_fill = Color32::from_rgb(22, 22, 30);
    visuals.extreme_bg_color = Color32::from_rgb(8, 8, 12);

    visuals.widgets.hovered.bg_fill = Color32::from_rgb(50, 80, 130);
    visuals.widgets.active.bg_fill = COLOR_PRIMARY;
    visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(26, 140, 255, 100);

    // Visible focus ring on keyboard-navigated buttons (Step 522)
    visuals.selection.stroke = egui::Stroke::new(2.5_f32, Color32::from_rgb(255, 215, 0));
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.5_f32, Color32::from_rgb(255, 215, 0));
    visuals.widgets.active.bg_stroke = egui::Stroke::new(2.0_f32, Color32::WHITE);

    visuals.window_rounding = Rounding::same(8.0);

    ctx.set_visuals(visuals);

    let fonts = FontDefinitions::default();
    ctx.set_fonts(fonts);
    update_font_styles(ctx, font_size);
}

#[cfg(feature = "gui")]
pub fn apply_light_theme(ctx: &egui::Context, font_size: f32) {
    let mut visuals = Visuals::light();
    visuals.panel_fill = Color32::from_rgb(240, 242, 245);
    visuals.window_fill = Color32::from_rgb(255, 255, 255);
    visuals.extreme_bg_color = Color32::from_rgb(225, 230, 238);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(180, 210, 255);
    visuals.widgets.active.bg_fill = COLOR_PRIMARY;
    visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(26, 140, 255, 120);

    visuals.selection.stroke = egui::Stroke::new(2.5_f32, Color32::from_rgb(0, 100, 220));
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.5_f32, Color32::from_rgb(0, 100, 220));
    visuals.widgets.active.bg_stroke = egui::Stroke::new(2.0_f32, Color32::BLACK);

    visuals.window_rounding = Rounding::same(8.0);
    ctx.set_visuals(visuals);
    update_font_styles(ctx, font_size);
}

#[cfg(feature = "gui")]
pub fn apply_high_contrast_theme(ctx: &egui::Context, font_size: f32) {
    let mut visuals = Visuals::dark();
    visuals.panel_fill = COLOR_HIGH_CONTRAST_BG;
    visuals.window_fill = COLOR_HIGH_CONTRAST_PANEL;
    visuals.extreme_bg_color = Color32::from_rgb(4, 4, 6);

    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.5_f32, COLOR_HIGH_CONTRAST_TEXT);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.5_f32, Color32::from_rgb(230, 230, 230));
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(60, 60, 90);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(2.0_f32, Color32::WHITE);
    visuals.widgets.active.bg_fill = Color32::from_rgb(0, 180, 255);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0_f32, Color32::WHITE);
    visuals.selection.bg_fill = Color32::from_rgb(0, 150, 255);

    // High visibility focus ring (Step 522)
    visuals.selection.stroke = egui::Stroke::new(3.0_f32, COLOR_HIGH_CONTRAST_FOCUS);
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(2.5_f32, COLOR_HIGH_CONTRAST_FOCUS);
    visuals.widgets.active.bg_stroke = egui::Stroke::new(3.0_f32, Color32::WHITE);

    visuals.window_rounding = Rounding::same(4.0);
    ctx.set_visuals(visuals);
    update_font_styles(ctx, font_size);
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "gui")]
    use super::*;

    #[test]
    #[cfg(feature = "gui")]
    fn test_theme_applies_without_panic() {
        let ctx = egui::Context::default();
        apply_summoner_theme(&ctx, 14.0);
        assert_eq!(COLOR_PRIMARY, Color32::from_rgb(26, 140, 255));
        assert_eq!(COLOR_ACCENT, Color32::from_rgb(255, 107, 43));
        assert_eq!(COLOR_BG, Color32::from_rgb(15, 15, 20));

        apply_light_theme(&ctx, 14.0);
        apply_high_contrast_theme(&ctx, 16.0);
    }

    #[test]
    #[cfg(feature = "gui")]
    fn test_accessibility_theme_and_wcag_contrast() {
        let ctx = egui::Context::default();
        apply_summoner_theme(&ctx, 14.0);
        assert!(meets_wcag_aa(Color32::WHITE, COLOR_BG));
        assert!(meets_wcag_aa(Color32::WHITE, COLOR_HIGH_CONTRAST_BG));

        apply_high_contrast_theme(&ctx, 18.0);
        assert_eq!(COLOR_HIGH_CONTRAST_BG, Color32::from_rgb(0, 0, 0));
        assert!(contrast_ratio(COLOR_HIGH_CONTRAST_TEXT, COLOR_HIGH_CONTRAST_BG) >= 15.0);

        apply_light_theme(&ctx, 12.0);
        assert!(meets_wcag_aa(Color32::BLACK, Color32::WHITE));
    }
}

