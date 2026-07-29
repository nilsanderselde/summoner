// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Rounding, Visuals};

#[cfg(feature = "gui")]
pub fn apply_summoner_theme(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();

    // Custom dark theme colors
    visuals.panel_fill = Color32::from_rgb(15, 15, 20);
    visuals.window_fill = Color32::from_rgb(22, 22, 30);
    visuals.extreme_bg_color = Color32::from_rgb(8, 8, 12);

    visuals.widgets.hovered.bg_fill = Color32::from_rgb(50, 80, 130);
    visuals.widgets.active.bg_fill = Color32::from_rgb(26, 140, 255);
    visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(26, 140, 255, 100);

    visuals.window_rounding = Rounding::same(8.0);

    ctx.set_visuals(visuals);
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "gui")]
    use super::*;

    #[test]
    #[cfg(feature = "gui")]
    fn test_theme_applies_without_panic() {
        let ctx = egui::Context::default();
        apply_summoner_theme(&ctx);
    }
}
