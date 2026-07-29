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
                name: "31-EDO Quartertone Diatonic".to_string(),
                description: "Quartertone tuned diatonic scale in 31 EDO".to_string(),
                degrees: vec![0, 5, 10, 13, 18, 23, 28],
                divisions: 31,
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

/// Renders the Scala Scale Browser modal panel.
pub fn show_scala_browser(
    ui: &mut egui::Ui,
    mut selected_track: Option<&mut TrackConfig>,
    harmonic_ctx: &mut HarmonicContext,
) {
    ui.group(|ui| {
        ui.heading("📜 Scala Historical Scale Browser");
        ui.label("Browse historical scales and apply microtonal tunings directly to active track.");

        ui.separator();

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
    });
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
}
