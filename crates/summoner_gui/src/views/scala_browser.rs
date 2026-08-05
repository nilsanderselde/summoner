// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

use eframe::egui;
use summoner_harmony::bus::HarmonicContext;
use summoner_harmony::edo::EdoTuning;
use summoner_harmony::scale::Scale;
use summoner_project::schema::TrackConfig;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScalaBrowserTab {
    PresetScales,
    CustomBuilder,
    JiLattice,
}

#[derive(Debug, Clone)]
pub struct CustomScaleBuilder {
    pub name: String,
    pub cents: Vec<f64>,
    pub selected_step: Option<usize>,
}

impl Default for CustomScaleBuilder {
    fn default() -> Self {
        Self {
            name: "Custom 7-Limit Scale".to_string(),
            cents: vec![0.0, 203.91, 386.31, 498.04, 701.96, 884.36, 1088.27, 1200.0],
            selected_step: None,
        }
    }
}

impl CustomScaleBuilder {
    pub fn add_step(&mut self) {
        if self.cents.len() < 32 {
            let last = *self.cents.last().unwrap_or(&1200.0);
            self.cents.push(last + 100.0);
        }
    }

    pub fn remove_step(&mut self) {
        if self.cents.len() > 2 {
            self.cents.pop();
        }
    }

    pub fn reset_even_cents(&mut self, num_steps: usize) {
        let count = num_steps.clamp(2, 32);
        let step_cents = 1200.0 / count as f64;
        self.cents = (0..=count).map(|i| i as f64 * step_cents).collect();
    }
}

#[derive(Debug, Clone, Default)]
pub struct JiLatticeState {
    pub selected_node: Option<(i32, i32)>,
}

#[derive(Debug, Clone)]
pub struct ScalaBrowserState {
    pub current_tab: ScalaBrowserTab,
    pub custom_builder: CustomScaleBuilder,
    pub ji_lattice: JiLatticeState,
}

impl Default for ScalaBrowserState {
    fn default() -> Self {
        Self {
            current_tab: ScalaBrowserTab::PresetScales,
            custom_builder: CustomScaleBuilder::default(),
            ji_lattice: JiLatticeState::default(),
        }
    }
}

/// Historical scale definition for the Scala Scale Browser.
#[derive(Debug, Clone)]
pub struct HistoricalScale {
    pub name: String,
    pub description: String,
    pub degrees: Vec<u16>,
    pub divisions: u16,
}

impl HistoricalScale {
    pub fn get_preset_scales() -> Vec<Self> {
        vec![
            Self {
                name: "12-TET Major (Ionian)".to_string(),
                description: "Standard western major scale".to_string(),
                degrees: vec![0, 2, 4, 5, 7, 9, 11],
                divisions: 12,
            },
            Self {
                name: "12-TET Natural Minor (Aeolian)".to_string(),
                description: "Standard natural minor scale".to_string(),
                degrees: vec![0, 2, 3, 5, 7, 8, 10],
                divisions: 12,
            },
            Self {
                name: "12-TET Harmonic Minor".to_string(),
                description: "Minor scale with raised 7th degree".to_string(),
                degrees: vec![0, 2, 3, 5, 7, 8, 11],
                divisions: 12,
            },
            Self {
                name: "12-TET Dorian".to_string(),
                description: "Minor mode with raised 6th degree".to_string(),
                degrees: vec![0, 2, 3, 5, 7, 9, 10],
                divisions: 12,
            },
            Self {
                name: "12-TET Pentatonic Major".to_string(),
                description: "5-note major scale".to_string(),
                degrees: vec![0, 2, 4, 7, 9],
                divisions: 12,
            },
            Self {
                name: "12-TET Blues".to_string(),
                description: "6-note blues scale".to_string(),
                degrees: vec![0, 3, 5, 6, 7, 10],
                divisions: 12,
            },
            Self {
                name: "19-EDO Diatonic".to_string(),
                description: "Diatonic scale in 19 EDO microtonal tuning".to_string(),
                degrees: vec![0, 3, 6, 8, 11, 14, 17],
                divisions: 19,
            },
            Self {
                name: "24-EDO Quartertone Diatonic".to_string(),
                description: "Quartertone tuned 24 EDO scale with neutral intervals".to_string(),
                degrees: vec![0, 3, 7, 10, 14, 17, 21],
                divisions: 24,
            },
            Self {
                name: "31-EDO Quartertone Diatonic".to_string(),
                description: "Quartertone tuned diatonic scale in 31 EDO".to_string(),
                degrees: vec![0, 5, 10, 13, 18, 23, 28],
                divisions: 31,
            },
            Self {
                name: "53-EDO Turkish Makam Rast".to_string(),
                description: "53 EDO microtonal scale for Turkish Makam music".to_string(),
                degrees: vec![0, 9, 17, 22, 31, 40, 48],
                divisions: 53,
            },
            Self {
                name: "1/4-Comma Meantone Temperament".to_string(),
                description: "Historical Renaissance meantone tuning".to_string(),
                degrees: vec![0, 2, 4, 5, 7, 9, 11],
                divisions: 12,
            },
            Self {
                name: "Just Intonation (5-Limit Diatonic)".to_string(),
                description: "Pure harmonic ratios (1/1, 9/8, 5/4, 4/3, 3/2, 5/3, 15/8)"
                    .to_string(),
                degrees: vec![0, 2, 4, 5, 7, 9, 11],
                divisions: 12,
            },
            Self {
                name: "Bohlen-Pierce Scale".to_string(),
                description: "Non-octave tritave-based tuning scale".to_string(),
                degrees: vec![0, 2, 3, 5, 7, 9, 10],
                divisions: 13,
            },
        ]
    }
}

/// Renders the full Scala Scale Browser modal panel with state.
pub fn show_scala_browser_with_state(
    ui: &mut egui::Ui,
    state: &mut ScalaBrowserState,
    mut selected_track: Option<&mut TrackConfig>,
    harmonic_ctx: &mut HarmonicContext,
) {
    ui.group(|ui| {
        ui.heading("📜 Scala Microtonal & Scale Management");

        ui.horizontal(|ui| {
            ui.selectable_value(&mut state.current_tab, ScalaBrowserTab::PresetScales, "📜 Historical Presets");
            ui.selectable_value(&mut state.current_tab, ScalaBrowserTab::CustomBuilder, "🛠 Custom Scale Builder");
            ui.selectable_value(&mut state.current_tab, ScalaBrowserTab::JiLattice, "🕸 JI 2D Lattice View");
        });

        ui.separator();

        match state.current_tab {
            ScalaBrowserTab::PresetScales => {
                ui.label("Browse historical scales and apply microtonal tunings directly to active track.");
                let scales = HistoricalScale::get_preset_scales();

                egui::ScrollArea::vertical().max_height(250.0).show(ui, |ui| {
                    egui::Grid::new("scala_browser_grid")
                        .striped(true)
                        .min_col_width(120.0)
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("Scale Name").strong());
                            ui.label(egui::RichText::new("Divisions").strong());
                            ui.label(egui::RichText::new("Description").strong());
                            ui.label(egui::RichText::new("Action").strong());
                            ui.end_row();

                            for scale in &scales {
                                ui.label(&scale.name);
                                ui.label(format!("{} EDO", scale.divisions));
                                ui.label(&scale.description);

                                let apply_btn = ui.button("🎯 Apply to Track");
                                if apply_btn.clicked() {
                                    apply_scale_to_context(scale, selected_track.as_deref_mut(), harmonic_ctx);
                                }
                                ui.end_row();
                            }
                        });
                });
            }
            ScalaBrowserTab::CustomBuilder => {
                show_custom_scale_builder_panel(ui, &mut state.custom_builder, selected_track, harmonic_ctx);
            }
            ScalaBrowserTab::JiLattice => {
                show_ji_lattice_view(ui, &mut state.ji_lattice);
            }
        }
    });
}

/// Backwards-compatible entrypoint.
pub fn show_scala_browser(
    ui: &mut egui::Ui,
    selected_track: Option<&mut TrackConfig>,
    harmonic_ctx: &mut HarmonicContext,
) {
    let mut state = ScalaBrowserState::default();
    show_scala_browser_with_state(ui, &mut state, selected_track, harmonic_ctx);
}

/// Renders the Custom Scale Builder UI panel (Step 476, Step 477).
pub fn show_custom_scale_builder_panel(
    ui: &mut egui::Ui,
    builder: &mut CustomScaleBuilder,
    selected_track: Option<&mut TrackConfig>,
    harmonic_ctx: &mut HarmonicContext,
) {
    ui.label("Enter cent values per step manually to build microtonal scale tunings.");

    ui.horizontal(|ui| {
        ui.label("Scale Name:");
        ui.text_edit_singleline(&mut builder.name);
    });

    ui.horizontal(|ui| {
        if ui.button("➕ Add Step").clicked() {
            builder.add_step();
        }
        if ui.button("➖ Remove Step").clicked() {
            builder.remove_step();
        }
        if ui.button("⚖ Even 12-EDO").clicked() {
            builder.reset_even_cents(12);
        }
        if ui.button("⚖ Even 19-EDO").clicked() {
            builder.reset_even_cents(19);
        }
        if ui.button("⚖ Even 31-EDO").clicked() {
            builder.reset_even_cents(31);
        }
    });

    ui.separator();

    // Step 477: Virtual keyboard visual for custom scale with cents labeled
    show_custom_keyboard_visual(ui, builder);

    ui.separator();

    ui.label("Cent Values per Step:");
    let len = builder.cents.len();
    egui::ScrollArea::vertical()
        .max_height(140.0)
        .show(ui, |ui| {
            egui::Grid::new("custom_cents_grid")
                .striped(true)
                .show(ui, |ui| {
                    for i in 0..len {
                        ui.label(format!("Step {}:", i));
                        ui.add(
                            egui::DragValue::new(&mut builder.cents[i])
                                .speed(0.1)
                                .range(0.0..=2400.0)
                                .suffix("¢"),
                        );

                        if i > 0 && i < len - 1 {
                            let interval = builder.cents[i] - builder.cents[i - 1];
                            ui.label(format!("(+{:.1}¢)", interval));
                        } else if i == 0 {
                            ui.label("(Root)");
                        } else {
                            ui.label("(Octave / Period)");
                        }
                        ui.end_row();
                    }
                });
        });

    ui.separator();

    if ui.button("🎯 Apply Custom Scale").clicked() {
        let num_steps = builder.cents.len().saturating_sub(1).max(1);
        let scale_def = HistoricalScale {
            name: builder.name.clone(),
            description: format!("Custom cents scale with {} steps", num_steps),
            degrees: (0..num_steps as u16).collect(),
            divisions: num_steps as u16,
        };
        apply_scale_to_context(&scale_def, selected_track, harmonic_ctx);
    }
}

/// Renders the virtual keyboard visual for custom scale with cents labeled (Step 477).
pub fn show_custom_keyboard_visual(ui: &mut egui::Ui, builder: &mut CustomScaleBuilder) {
    ui.label(egui::RichText::new("🎹 Custom Scale Virtual Keyboard (Cents Labeled)").strong());
    let total_cents = *builder.cents.last().unwrap_or(&1200.0);
    let (response, painter) = ui.allocate_painter(egui::vec2(500.0, 50.0), egui::Sense::click());
    let rect = response.rect;

    painter.rect_filled(rect, 4.0, egui::Color32::from_gray(25));

    let len = builder.cents.len();
    for i in 0..len.saturating_sub(1) {
        let c_start = builder.cents[i];
        let c_end = builder.cents[i + 1];
        let x_start = rect.left() + (c_start / total_cents.max(1.0)) as f32 * rect.width();
        let x_end = rect.left() + (c_end / total_cents.max(1.0)) as f32 * rect.width();

        let key_rect = egui::Rect::from_min_max(
            egui::pos2(x_start, rect.top() + 3.0),
            egui::pos2((x_end - 1.0).max(x_start + 2.0), rect.bottom() - 3.0),
        );

        let is_selected = builder.selected_step == Some(i);
        let fill = if is_selected {
            egui::Color32::from_rgb(26, 140, 255)
        } else if i % 2 == 0 {
            egui::Color32::from_rgb(220, 235, 255)
        } else {
            egui::Color32::from_rgb(45, 65, 95)
        };

        let key_interact = ui.interact(
            key_rect,
            ui.id().with(("custom_key", i)),
            egui::Sense::click(),
        );
        if key_interact.clicked() {
            builder.selected_step = Some(i);
        }

        painter.rect_filled(key_rect, 2.0, fill);
        painter.rect_stroke(
            key_rect,
            2.0,
            egui::Stroke::new(1.0_f32, egui::Color32::BLACK),
        );

        let text_color = if i % 2 == 0 && !is_selected {
            egui::Color32::BLACK
        } else {
            egui::Color32::WHITE
        };

        if key_rect.width() > 20.0 {
            painter.text(
                key_rect.center() - egui::vec2(0.0, 5.0),
                egui::Align2::CENTER_CENTER,
                format!("S{}", i + 1),
                egui::FontId::proportional(9.0),
                text_color,
            );
            painter.text(
                key_rect.center() + egui::vec2(0.0, 5.0),
                egui::Align2::CENTER_CENTER,
                format!("{:.1}¢", c_start),
                egui::FontId::proportional(8.0),
                text_color,
            );
        }
    }
}

/// Renders the 2D Just Intonation Lattice View (Step 479).
pub fn show_ji_lattice_view(ui: &mut egui::Ui, state: &mut JiLatticeState) {
    ui.label(egui::RichText::new("🕸 Just Intonation 2D Harmonic Lattice").strong());
    ui.label(
        "Horizontal axis = Perfect Fifths (3:2, ~702¢) | Vertical axis = Major Thirds (5:4, ~386¢)",
    );

    let m_range = -2..=2; // Fifths
    let n_range = -2..=2; // Thirds

    let (response, painter) = ui.allocate_painter(egui::vec2(500.0, 220.0), egui::Sense::click());
    let rect = response.rect;

    let center_x = rect.center().x;
    let center_y = rect.center().y;
    let spacing_x = 80.0;
    let spacing_y = 45.0;

    for m in m_range.clone() {
        for n in n_range.clone() {
            let x = center_x + m as f32 * spacing_x;
            let y = center_y - n as f32 * spacing_y;

            if m < 2 {
                let next_x = center_x + (m + 1) as f32 * spacing_x;
                painter.line_segment(
                    [egui::pos2(x, y), egui::pos2(next_x, y)],
                    egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(60, 90, 130)),
                );
            }

            if n < 2 {
                let next_y = center_y - (n + 1) as f32 * spacing_y;
                painter.line_segment(
                    [egui::pos2(x, y), egui::pos2(x, next_y)],
                    egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(130, 90, 60)),
                );
            }

            let mut cents = (m as f64 * 701.955 + n as f64 * 386.314) % 1200.0;
            if cents < 0.0 {
                cents += 1200.0;
            }

            let ratio_str = match (m, n) {
                (0, 0) => "1/1 (C)",
                (1, 0) => "3/2 (G)",
                (-1, 0) => "4/3 (F)",
                (2, 0) => "9/8 (D)",
                (-2, 0) => "16/9 (Bb)",
                (0, 1) => "5/4 (E)",
                (1, 1) => "15/8 (B)",
                (-1, 1) => "5/3 (A)",
                (0, -1) => "8/5 (Ab)",
                (1, -1) => "6/5 (Eb)",
                (-1, -1) => "7/5 (F#)",
                _ => "Ratio",
            };

            let is_selected = state.selected_node == Some((m, n));
            let is_center = m == 0 && n == 0;

            let node_radius = if is_selected { 16.0 } else { 13.0 };
            let fill_color = if is_center {
                egui::Color32::from_rgb(255, 180, 50)
            } else if is_selected {
                egui::Color32::from_rgb(26, 140, 255)
            } else {
                egui::Color32::from_rgb(40, 50, 70)
            };

            let node_rect = egui::Rect::from_center_size(
                egui::pos2(x, y),
                egui::vec2(node_radius * 2.0, node_radius * 2.0),
            );
            if ui
                .interact(
                    node_rect,
                    ui.id().with(("ji_node", m, n)),
                    egui::Sense::click(),
                )
                .clicked()
            {
                state.selected_node = Some((m, n));
            }

            painter.circle_filled(egui::pos2(x, y), node_radius, fill_color);
            painter.circle_stroke(
                egui::pos2(x, y),
                node_radius,
                egui::Stroke::new(1.0_f32, egui::Color32::WHITE),
            );

            painter.text(
                egui::pos2(x, y - 3.0),
                egui::Align2::CENTER_CENTER,
                format!("{:.0}¢", cents),
                egui::FontId::proportional(8.5),
                egui::Color32::WHITE,
            );
            painter.text(
                egui::pos2(x, y + 5.0),
                egui::Align2::CENTER_CENTER,
                ratio_str,
                egui::FontId::proportional(7.0),
                egui::Color32::from_gray(210),
            );
        }
    }

    if let Some((m, n)) = state.selected_node {
        let mut cents = (m as f64 * 701.955 + n as f64 * 386.314) % 1200.0;
        if cents < 0.0 {
            cents += 1200.0;
        }
        ui.label(format!(
            "Selected Lattice Node ({}, {}): {:.2} cents above root",
            m, n, cents
        ));
    }
}

/// Applies a selected historical scale to the global HarmonicContext and optional track configuration.
pub fn apply_scale_to_context(
    scale_def: &HistoricalScale,
    selected_track: Option<&mut TrackConfig>,
    harmonic_ctx: &mut HarmonicContext,
) {
    harmonic_ctx.tuning = EdoTuning::new(scale_def.divisions, 440.0, 69.0);
    harmonic_ctx.scale = Scale {
        name: scale_def.name.clone(),
        degrees: scale_def.degrees.clone(),
    };

    if let Some(track) = selected_track {
        track.tuning_edo = Some(scale_def.divisions as u32);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_scale_updates_harmonic_context() {
        let scale_def = HistoricalScale {
            name: "19-EDO Test".to_string(),
            description: "Test scale".to_string(),
            degrees: vec![0, 3, 6, 8, 11, 14, 17],
            divisions: 19,
        };

        let mut track = TrackConfig {
            id: 1,
            name: "Melody Track".to_string(),
            channels: 2,
            gain: 1.0,
            pan: 0.0,
            muted: false,
            soloed: false,
            send_level: 0.0,
            nodes: Vec::new(),
            sequence: None,
            clips: Vec::new(),
            connections: Vec::new(),
            tuning_edo: None,
            tuning_root_hz: None,
            tuning_scl_path: None,
            ..Default::default()
        };

        let mut ctx = HarmonicContext::default();
        apply_scale_to_context(&scale_def, Some(&mut track), &mut ctx);

        assert_eq!(ctx.tuning.divisions, 19);
        assert_eq!(ctx.scale.name, "19-EDO Test");
        assert_eq!(track.tuning_edo, Some(19u32));
    }

    #[test]
    fn test_scala_browser_populated_preset_scales() {
        let scales = HistoricalScale::get_preset_scales();
        let names: Vec<&str> = scales.iter().map(|s| s.name.as_str()).collect();

        assert!(names.iter().any(|n| n.contains("12-TET Major")));
        assert!(names.iter().any(|n| n.contains("19-EDO")));
        assert!(names.iter().any(|n| n.contains("24-EDO")));
        assert!(names.iter().any(|n| n.contains("31-EDO")));
        assert!(names.iter().any(|n| n.contains("53-EDO")));
        assert!(names.iter().any(|n| n.contains("Meantone")));
        assert!(names.iter().any(|n| n.contains("Just Intonation")));
        assert!(names.iter().any(|n| n.contains("Bohlen-Pierce")));
    }

    #[test]
    fn test_custom_scale_builder_cents_to_scale() {
        let mut builder = CustomScaleBuilder {
            cents: vec![0.0, 200.0, 400.0, 700.0, 900.0, 1200.0],
            ..Default::default()
        };

        assert_eq!(builder.cents.len(), 6);
        builder.add_step();
        assert_eq!(builder.cents.len(), 7);
        builder.remove_step();
        assert_eq!(builder.cents.len(), 6);

        builder.reset_even_cents(19);
        assert_eq!(builder.cents.len(), 20);
        assert!((builder.cents[1] - 1200.0 / 19.0).abs() < 1e-3);
    }
}
