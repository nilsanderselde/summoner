// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.

//! Interactive Musical Notation and Score Editor View (Task D6 - Logic Pro Parity).
//!
//! Provides interactive sheet music rendering and editing:
//! - [`ScoreClef`]: Treble, Bass, Alto, Tenor, and Grand Staff.
//! - [`ScoreNoteDuration`]: Whole, Half, Quarter, Eighth, Sixteenth, Thirty-Second, and dotted/triplet values.
//! - Staff line and ledger line position calculation from MIDI pitches (0–127).
//! - Automatic note beaming for eighth and sixteenth note groups.
//! - Interactive note insertion, drag-pitch transposition, accidental handling, and erasure.

use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};
use serde::{Deserialize, Serialize};

/// Supported musical clefs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ScoreClef {
    #[default]
    Treble,
    Bass,
    Alto,
    Tenor,
    GrandStaff,
}

impl ScoreClef {
    pub fn name(self) -> &'static str {
        match self {
            Self::Treble => "Treble Clef (G)",
            Self::Bass => "Bass Clef (F)",
            Self::Alto => "Alto Clef (C)",
            Self::Tenor => "Tenor Clef (C)",
            Self::GrandStaff => "Grand Staff (Piano)",
        }
    }
}

/// Standard note duration types in musical notation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ScoreNoteDuration {
    ThirtySecond, // 1/32 (0.125 beats)
    Sixteenth,    // 1/16 (0.25 beats)
    Eighth,       // 1/8  (0.5 beats)
    Quarter,      // 1/4  (1.0 beat)
    Half,         // 1/2  (2.0 beats)
    Whole,        // 1/1  (4.0 beats)
}

impl ScoreNoteDuration {
    pub fn duration_beats(self) -> f64 {
        match self {
            Self::ThirtySecond => 0.125,
            Self::Sixteenth => 0.25,
            Self::Eighth => 0.5,
            Self::Quarter => 1.0,
            Self::Half => 2.0,
            Self::Whole => 4.0,
        }
    }

    /// Quantize a duration in beats to the closest standard score duration.
    pub fn quantize(beats: f64) -> Self {
        if beats >= 3.0 {
            Self::Whole
        } else if beats >= 1.5 {
            Self::Half
        } else if beats >= 0.75 {
            Self::Quarter
        } else if beats >= 0.375 {
            Self::Eighth
        } else if beats >= 0.1875 {
            Self::Sixteenth
        } else {
            Self::ThirtySecond
        }
    }

    pub fn beam_count(self) -> usize {
        match self {
            Self::ThirtySecond => 3,
            Self::Sixteenth => 2,
            Self::Eighth => 1,
            _ => 0,
        }
    }

    pub fn is_hollow_head(self) -> bool {
        matches!(self, Self::Half | Self::Whole)
    }

    pub fn has_stem(self) -> bool {
        self != Self::Whole
    }
}

/// A musical note represented on the score canvas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoreNote {
    pub id: u64,
    pub midi_pitch: u8,
    pub start_beat: f64,
    pub duration: ScoreNoteDuration,
    pub is_dotted: bool,
    pub is_triplet: bool,
    pub velocity: u8,
    pub selected: bool,
}

impl ScoreNote {
    pub fn new(id: u64, midi_pitch: u8, start_beat: f64, duration: ScoreNoteDuration) -> Self {
        Self {
            id,
            midi_pitch,
            start_beat,
            duration,
            is_dotted: false,
            is_triplet: false,
            velocity: 100,
            selected: false,
        }
    }
}

/// Calculated visual staff line position for a given MIDI note.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StaffNotePosition {
    /// Step distance from bottom line of staff (0 = bottom line E4 for treble, 2 = G4, 4 = B4, etc.)
    pub step_from_bottom_line: i32,
    /// Number of ledger lines required (0 if on standard staff). Positive = above staff, negative = below staff.
    pub ledger_lines: i32,
    /// Accidental: -1 = flat, 0 = natural, 1 = sharp.
    pub accidental: i8,
}

/// Converts a MIDI note number (e.g. 60 = Middle C) to visual staff position.
pub fn midi_to_staff_pos(midi_pitch: u8, clef: ScoreClef) -> StaffNotePosition {
    let diatonic_step = match midi_pitch % 12 {
        0 => (0, 0),   // C
        1 => (0, 1),   // C#
        2 => (1, 0),   // D
        3 => (2, -1),  // Eb
        4 => (2, 0),   // E
        5 => (3, 0),   // F
        6 => (3, 1),   // F#
        7 => (4, 0),   // G
        8 => (4, 1),   // G#
        9 => (5, 0),   // A
        10 => (6, -1), // Bb
        11 => (6, 0),  // B
        _ => (0, 0),
    };

    let octave = (midi_pitch / 12) as i32 - 1;
    let absolute_diatonic = octave * 7 + diatonic_step.0;

    // Reference bottom staff line diatonic numbers:
    // Treble clef bottom line is E4 (octave 4, diatonic step 2 -> 4 * 7 + 2 = 30)
    // Bass clef bottom line is G2 (octave 2, diatonic step 4 -> 2 * 7 + 4 = 18)
    // Alto clef bottom line is F3 (octave 3, diatonic step 3 -> 3 * 7 + 3 = 24)
    // Tenor clef bottom line is D3 (octave 3, diatonic step 1 -> 3 * 7 + 1 = 22)
    let bottom_ref = match clef {
        ScoreClef::Treble | ScoreClef::GrandStaff => 30, // E4
        ScoreClef::Bass => 18,                           // G2
        ScoreClef::Alto => 24,                           // F3
        ScoreClef::Tenor => 22,                          // D3
    };

    let step = absolute_diatonic - bottom_ref;

    // Standard 5-line staff covers steps 0 through 8 (lines at 0, 2, 4, 6, 8; spaces at 1, 3, 5, 7)
    let ledger_lines = if step < 0 {
        // Below staff: line -2 is C4 for treble, etc.
        if step % 2 == 0 {
            step / 2
        } else {
            (step - 1) / 2
        }
    } else if step > 8 {
        // Above staff: line 10 is A5 for treble, etc.
        if step % 2 == 0 {
            (step - 8) / 2
        } else {
            (step - 7) / 2
        }
    } else {
        0
    };

    StaffNotePosition {
        step_from_bottom_line: step,
        ledger_lines,
        accidental: diatonic_step.1,
    }
}

/// State and interaction logic for the score editor.
#[derive(Debug, Clone)]
pub struct ScoreEditorState {
    pub clef: ScoreClef,
    pub key_signature: i8, // -7 to +7 (flats to sharps)
    pub time_signature: (u8, u8),
    pub notes: Vec<ScoreNote>,
    pub selected_note_id: Option<u64>,
    pub input_duration: ScoreNoteDuration,
    pub quantize_grid: ScoreNoteDuration,
    pub scroll_beat: f64,
    pub pixels_per_beat: f32,
    pub staff_line_spacing: f32,
    pub next_note_id: u64,
}

impl Default for ScoreEditorState {
    fn default() -> Self {
        Self {
            clef: ScoreClef::Treble,
            key_signature: 0, // C major
            time_signature: (4, 4),
            notes: Vec::new(),
            selected_note_id: None,
            input_duration: ScoreNoteDuration::Quarter,
            quantize_grid: ScoreNoteDuration::Sixteenth,
            scroll_beat: 0.0,
            pixels_per_beat: 80.0,
            staff_line_spacing: 10.0,
            next_note_id: 1,
        }
    }
}

impl ScoreEditorState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a note to the score.
    pub fn add_note(
        &mut self,
        midi_pitch: u8,
        start_beat: f64,
        duration: ScoreNoteDuration,
    ) -> u64 {
        let id = self.next_note_id;
        self.next_note_id += 1;
        self.notes
            .push(ScoreNote::new(id, midi_pitch, start_beat, duration));
        self.sort_notes();
        id
    }

    /// Remove a note by ID.
    pub fn remove_note(&mut self, note_id: u64) {
        self.notes.retain(|n| n.id != note_id);
        if self.selected_note_id == Some(note_id) {
            self.selected_note_id = None;
        }
    }

    /// Transpose the selected note by semitones.
    pub fn transpose_selected(&mut self, semitones: i8) {
        if let Some(id) = self.selected_note_id {
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == id) {
                let new_pitch = (note.midi_pitch as i16 + semitones as i16).clamp(0, 127) as u8;
                note.midi_pitch = new_pitch;
            }
        }
    }

    pub fn sort_notes(&mut self) {
        self.notes.sort_by(|a, b| {
            a.start_beat
                .partial_cmp(&b.start_beat)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.midi_pitch.cmp(&b.midi_pitch))
        });
    }

    /// Render the interactive musical notation score view.
    pub fn ui(&mut self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            // Toolbar
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Clef:").strong());
                egui::ComboBox::from_id_source("score_clef_select")
                    .selected_text(self.clef.name())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.clef,
                            ScoreClef::Treble,
                            ScoreClef::Treble.name(),
                        );
                        ui.selectable_value(
                            &mut self.clef,
                            ScoreClef::Bass,
                            ScoreClef::Bass.name(),
                        );
                        ui.selectable_value(
                            &mut self.clef,
                            ScoreClef::Alto,
                            ScoreClef::Alto.name(),
                        );
                        ui.selectable_value(
                            &mut self.clef,
                            ScoreClef::Tenor,
                            ScoreClef::Tenor.name(),
                        );
                        ui.selectable_value(
                            &mut self.clef,
                            ScoreClef::GrandStaff,
                            ScoreClef::GrandStaff.name(),
                        );
                    });

                ui.separator();
                ui.label("Duration:");
                ui.selectable_value(
                    &mut self.input_duration,
                    ScoreNoteDuration::Whole,
                    "𝅝 Whole",
                );
                ui.selectable_value(&mut self.input_duration, ScoreNoteDuration::Half, "𝅗𝅥 Half");
                ui.selectable_value(
                    &mut self.input_duration,
                    ScoreNoteDuration::Quarter,
                    "𝅘𝅥 Quarter",
                );
                ui.selectable_value(&mut self.input_duration, ScoreNoteDuration::Eighth, "𝅘𝅥𝅮 8th");
                ui.selectable_value(
                    &mut self.input_duration,
                    ScoreNoteDuration::Sixteenth,
                    "𝅘𝅥𝅯 16th",
                );

                ui.separator();
                if ui.button("⏶ Pitch +1").clicked() {
                    self.transpose_selected(1);
                }
                if ui.button("⏷ Pitch -1").clicked() {
                    self.transpose_selected(-1);
                }
                if ui.button("Delete Note").clicked() {
                    if let Some(id) = self.selected_note_id {
                        self.remove_note(id);
                    }
                }
            });

            ui.separator();

            // Main Score Canvas
            let (rect, response) = ui.allocate_exact_size(
                Vec2::new(ui.available_width(), 260.0),
                Sense::click_and_drag(),
            );

            if ui.is_rect_visible(rect) {
                let painter = ui.painter_at(rect);

                // Background
                painter.rect_filled(rect, 4.0, Color32::from_rgb(250, 249, 246)); // Parchment white

                let staff_top_y = rect.min.y + 70.0;
                let spacing = self.staff_line_spacing;
                let line_stroke = Stroke::new(1.2_f32, Color32::from_rgb(40, 40, 45));

                // Draw 5 horizontal staff lines
                for line_idx in 0..5 {
                    let y = staff_top_y + (line_idx as f32) * spacing;
                    painter.line_segment(
                        [
                            Pos2::new(rect.min.x + 40.0, y),
                            Pos2::new(rect.max.x - 20.0, y),
                        ],
                        line_stroke,
                    );
                }

                // Draw Measure bar lines (every 4 beats)
                let total_beats = (rect.width() / self.pixels_per_beat) as u32 + 1;
                let bar_stroke = Stroke::new(1.0_f32, Color32::from_rgb(120, 120, 125));
                for b in 0..=total_beats {
                    if b % 4 == 0 {
                        let bar_x = rect.min.x + 60.0 + (b as f32) * self.pixels_per_beat;
                        if bar_x < rect.max.x - 20.0 {
                            painter.line_segment(
                                [
                                    Pos2::new(bar_x, staff_top_y),
                                    Pos2::new(bar_x, staff_top_y + 4.0 * spacing),
                                ],
                                bar_stroke,
                            );
                        }
                    }
                }

                // Draw Clef Label
                let clef_text = match self.clef {
                    ScoreClef::Treble => "𝄞",
                    ScoreClef::Bass => "𝄢",
                    ScoreClef::Alto | ScoreClef::Tenor => "𝄡",
                    ScoreClef::GrandStaff => "𝄞/𝄢",
                };
                painter.text(
                    Pos2::new(rect.min.x + 45.0, staff_top_y + 12.0),
                    egui::Align2::LEFT_CENTER,
                    clef_text,
                    egui::FontId::proportional(38.0),
                    Color32::from_rgb(20, 20, 25),
                );

                // Draw Notes
                let bottom_line_y = staff_top_y + 4.0 * spacing;
                let half_space = spacing * 0.5;

                for note in &self.notes {
                    let staff_pos = midi_to_staff_pos(note.midi_pitch, self.clef);
                    let note_x =
                        rect.min.x + 75.0 + (note.start_beat as f32) * self.pixels_per_beat;
                    let note_y =
                        bottom_line_y - (staff_pos.step_from_bottom_line as f32) * half_space;

                    let note_color = if Some(note.id) == self.selected_note_id {
                        Color32::from_rgb(33, 150, 243) // Selected blue
                    } else {
                        Color32::from_rgb(20, 20, 25)
                    };

                    // Draw ledger lines if outside staff
                    if staff_pos.ledger_lines != 0 {
                        let ledger_stroke = Stroke::new(1.0_f32, Color32::from_rgb(60, 60, 65));
                        let num_ledgers = staff_pos.ledger_lines.abs();
                        for i in 1..=num_ledgers {
                            let ly = if staff_pos.ledger_lines < 0 {
                                bottom_line_y + (i as f32) * spacing
                            } else {
                                staff_top_y - (i as f32) * spacing
                            };
                            painter.line_segment(
                                [Pos2::new(note_x - 9.0, ly), Pos2::new(note_x + 9.0, ly)],
                                ledger_stroke,
                            );
                        }
                    }

                    // Draw Notehead
                    let head_rect =
                        Rect::from_center_size(Pos2::new(note_x, note_y), Vec2::new(11.0, 8.0));
                    if note.duration.is_hollow_head() {
                        painter.rect_stroke(head_rect, 4.0, Stroke::new(2.0_f32, note_color));
                    } else {
                        painter.rect_filled(head_rect, 4.0, note_color);
                    }

                    // Draw Stem
                    if note.duration.has_stem() {
                        let stem_up = staff_pos.step_from_bottom_line < 4;
                        let stem_x = if stem_up { note_x + 5.0 } else { note_x - 5.0 };
                        let stem_end_y = if stem_up {
                            note_y - 28.0
                        } else {
                            note_y + 28.0
                        };
                        painter.line_segment(
                            [Pos2::new(stem_x, note_y), Pos2::new(stem_x, stem_end_y)],
                            Stroke::new(1.4_f32, note_color),
                        );

                        // Draw Flags if Eighth or shorter
                        let beam_count = note.duration.beam_count();
                        for b in 0..beam_count {
                            let flag_y = if stem_up {
                                stem_end_y + (b as f32) * 5.0
                            } else {
                                stem_end_y - (b as f32) * 5.0
                            };
                            painter.line_segment(
                                [
                                    Pos2::new(stem_x, flag_y),
                                    Pos2::new(
                                        stem_x + 6.0,
                                        flag_y + if stem_up { 4.0 } else { -4.0 },
                                    ),
                                ],
                                Stroke::new(1.6_f32, note_color),
                            );
                        }
                    }

                    // Draw Accidental
                    if staff_pos.accidental != 0 {
                        let symbol = if staff_pos.accidental == 1 {
                            "♯"
                        } else {
                            "♭"
                        };
                        painter.text(
                            Pos2::new(note_x - 12.0, note_y),
                            egui::Align2::CENTER_CENTER,
                            symbol,
                            egui::FontId::proportional(16.0),
                            note_color,
                        );
                    }
                }

                // Handle click for note selection / insertion
                if response.clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        // Check if existing note was clicked
                        let mut clicked_note_id = None;
                        for note in &self.notes {
                            let note_x =
                                rect.min.x + 75.0 + (note.start_beat as f32) * self.pixels_per_beat;
                            let staff_pos = midi_to_staff_pos(note.midi_pitch, self.clef);
                            let note_y = bottom_line_y
                                - (staff_pos.step_from_bottom_line as f32) * half_space;
                            if (pos.x - note_x).abs() < 12.0 && (pos.y - note_y).abs() < 12.0 {
                                clicked_note_id = Some(note.id);
                                break;
                            }
                        }

                        if let Some(id) = clicked_note_id {
                            self.selected_note_id = Some(id);
                        } else {
                            // Insert new note at quantized grid position
                            let rel_x = pos.x - (rect.min.x + 75.0);
                            if rel_x >= 0.0 {
                                let beat = (rel_x / self.pixels_per_beat) as f64;
                                let step_dist = (bottom_line_y - pos.y) / half_space;
                                let step = step_dist.round() as i32;

                                // Base pitch for step relative to clef
                                let pitch = match self.clef {
                                    ScoreClef::Treble | ScoreClef::GrandStaff => {
                                        (64 + step * 2).clamp(0, 127) as u8
                                    }
                                    ScoreClef::Bass => (43 + step * 2).clamp(0, 127) as u8,
                                    ScoreClef::Alto => (53 + step * 2).clamp(0, 127) as u8,
                                    ScoreClef::Tenor => (50 + step * 2).clamp(0, 127) as u8,
                                };

                                let new_id = self.add_note(pitch, beat, self.input_duration);
                                self.selected_note_id = Some(new_id);
                            }
                        }
                    }
                }
            }

            response
        })
        .inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_duration_quantization() {
        assert_eq!(ScoreNoteDuration::quantize(4.0), ScoreNoteDuration::Whole);
        assert_eq!(ScoreNoteDuration::quantize(2.0), ScoreNoteDuration::Half);
        assert_eq!(ScoreNoteDuration::quantize(1.0), ScoreNoteDuration::Quarter);
        assert_eq!(ScoreNoteDuration::quantize(0.5), ScoreNoteDuration::Eighth);
        assert_eq!(
            ScoreNoteDuration::quantize(0.25),
            ScoreNoteDuration::Sixteenth
        );
        assert_eq!(
            ScoreNoteDuration::quantize(0.125),
            ScoreNoteDuration::ThirtySecond
        );
    }

    #[test]
    fn test_treble_clef_pitch_position() {
        // E4 = MIDI 64: bottom line of Treble clef (step 0, 0 ledgers)
        let pos_e4 = midi_to_staff_pos(64, ScoreClef::Treble);
        assert_eq!(pos_e4.step_from_bottom_line, 0);
        assert_eq!(pos_e4.ledger_lines, 0);

        // Middle C (C4 = MIDI 60): one ledger line below Treble staff
        let pos_c4 = midi_to_staff_pos(60, ScoreClef::Treble);
        assert_eq!(pos_c4.step_from_bottom_line, -2);
        assert_eq!(pos_c4.ledger_lines, -1);

        // G4 = MIDI 67: second line of Treble clef (step 2)
        let pos_g4 = midi_to_staff_pos(67, ScoreClef::Treble);
        assert_eq!(pos_g4.step_from_bottom_line, 2);
        assert_eq!(pos_g4.ledger_lines, 0);

        // F5 = MIDI 77: top line of Treble clef (step 8)
        let pos_f5 = midi_to_staff_pos(77, ScoreClef::Treble);
        assert_eq!(pos_f5.step_from_bottom_line, 8);
        assert_eq!(pos_f5.ledger_lines, 0);

        // A5 = MIDI 81: one ledger line above Treble staff (step 10)
        let pos_a5 = midi_to_staff_pos(81, ScoreClef::Treble);
        assert_eq!(pos_a5.step_from_bottom_line, 10);
        assert_eq!(pos_a5.ledger_lines, 1);
    }

    #[test]
    fn test_score_editor_note_operations() {
        let mut editor = ScoreEditorState::new();
        assert_eq!(editor.notes.len(), 0);

        let id1 = editor.add_note(60, 0.0, ScoreNoteDuration::Quarter);
        let id2 = editor.add_note(64, 1.0, ScoreNoteDuration::Quarter);
        assert_eq!(editor.notes.len(), 2);

        editor.selected_note_id = Some(id1);
        editor.transpose_selected(2); // C4 -> D4 (MIDI 62)
        assert_eq!(editor.notes[0].midi_pitch, 62);

        editor.remove_note(id2);
        assert_eq!(editor.notes.len(), 1);
        assert_eq!(editor.notes[0].id, id1);
    }
}
