// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

use eframe::egui;
use summoner_harmony::bus::HarmonicContext;
use summoner_project::schema::SequenceConfig;

/// Real-time chord suggestion panel displaying active detected chord and candidate continuations.
pub fn show_chord_suggestion_panel(
    ui: &mut egui::Ui,
    harmonic_ctx: &HarmonicContext,
    sequence: &mut SequenceConfig,
    playhead_beat: f64,
) {
    ui.group(|ui| {
        ui.heading("🎼 Real-Time Chord Suggestions");

        let active_chord = harmonic_ctx.analyze_active_chord();
        ui.horizontal(|ui| {
            ui.label("Current Active Chord:");
            ui.colored_label(egui::Color32::from_rgb(26, 140, 255), egui::RichText::new(&active_chord).strong());
        });

        ui.separator();
        ui.label("Suggested Next Chords:");

        let suggestions = [
            ("I - C Major", vec![60, 64, 67]),
            ("IV - F Major", vec![65, 69, 72]),
            ("V - G Major", vec![67, 71, 74]),
            ("vi - A Minor", vec![69, 72, 76]),
            ("ii - D Minor", vec![62, 65, 69]),
        ];

        ui.horizontal_wrapped(|ui| {
            for (label, notes) in suggestions {
                let btn = ui.button(format!("➕ {}", label));
                if btn.clicked() {
                    insert_chord_at_playhead(sequence, &notes, playhead_beat);
                }
                btn.on_hover_text(format!("Notes: {:?}", notes));
            }
        });
    });
}

/// Inserts chord notes into sequence steps starting at the step corresponding to playhead_beat.
pub fn insert_chord_at_playhead(sequence: &mut SequenceConfig, notes: &[u8], playhead_beat: f64) {
    if sequence.steps.is_empty() {
        return;
    }
    let step_div = sequence.step_division.max(0.01);
    let start_step = (playhead_beat / step_div).floor() as usize;

    for (i, &note_val) in notes.iter().enumerate() {
        let step_idx = (start_step + i) % sequence.steps.len();
        let step = &mut sequence.steps[step_idx];
        step.note = note_val as f64;
        step.velocity = 0.8;
        step.gate = 1.0;
        step.active = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chord_suggestion_inserts_notes() {
        let mut sequence = SequenceConfig {
            start_beat: 0.0,
            step_division: 0.25,
            clip_color: None,
            clip_name: None,
            steps: vec![
                summoner_project::schema::TrackerStepConfig {
                    note: 60.0,
                    velocity: 0.0,
                    gate: 0.0,
                    probability: 1.0,
                    ratchet: 1,
                    micro_shift: 0,
                    active: false,
                };
                16
            ],
        };

        insert_chord_at_playhead(&mut sequence, &[67, 71, 74], 0.0);
        assert!(sequence.steps[0].active);
        assert_eq!(sequence.steps[0].note, 67.0);
        assert!(sequence.steps[1].active);
        assert_eq!(sequence.steps[1].note, 71.0);
        assert!(sequence.steps[2].active);
        assert_eq!(sequence.steps[2].note, 74.0);
    }
}
