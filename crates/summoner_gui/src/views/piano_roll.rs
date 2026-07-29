use eframe::egui;
use std::collections::HashSet;
use summoner_project::schema::{SequenceConfig, TrackerStepConfig};
use summoner_harmony::edo::EdoTuning;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum PianoRollMode {
    StepGrid,
    PianoRoll,
}

pub struct PianoRollState {
    pub mode: PianoRollMode,
    pub scroll_offset: egui::Vec2,
    pub snap_division: f64,
    pub selected_notes: HashSet<usize>,
    pub clipboard: Vec<(usize, TrackerStepConfig)>,
    pub loop_start: f64,
    pub loop_end: f64,
    pub euclidean_pulses: u32,
    pub euclidean_steps: u32,
    pub show_euclidean_popup: bool,
}

impl Default for PianoRollState {
    fn default() -> Self {
        Self {
            mode: PianoRollMode::PianoRoll,
            scroll_offset: egui::Vec2::ZERO,
            snap_division: 0.25,
            selected_notes: HashSet::new(),
            clipboard: Vec::new(),
            loop_start: 0.0,
            loop_end: 16.0,
            euclidean_pulses: 4,
            euclidean_steps: 16,
            show_euclidean_popup: false,
        }
    }
}

pub struct Viewport {
    pub width: f32,
    pub height: f32,
}

pub fn quantize_notes(sequence: &mut SequenceConfig, snap_div: f64, selected: &HashSet<usize>) {
    let snap = snap_div.max(0.01);
    for (i, step) in sequence.steps.iter_mut().enumerate() {
        if selected.is_empty() || selected.contains(&i) {
            let snapped_gate = ((step.gate as f64 / snap).round() * snap).max(snap);
            step.gate = snapped_gate as f32;
        }
    }
}

use summoner_harmony::bus::HarmonicContext;

pub fn show_piano_roll(
    ui: &mut egui::Ui,
    sequence: &mut SequenceConfig,
    tuning: &EdoTuning,
    state: &mut PianoRollState,
    _viewport: &Viewport,
    harmonic_ctx: Option<&HarmonicContext>,
) {
    // Keyboard shortcuts for Ctrl+C and Ctrl+V
    if ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::C)) {
        state.clipboard = state
            .selected_notes
            .iter()
            .filter_map(|&idx| sequence.steps.get(idx).cloned().map(|s| (idx, s)))
            .collect();
    }
    if ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::V)) {
        for (orig_idx, step) in &state.clipboard {
            let paste_idx = orig_idx + 4;
            if paste_idx < sequence.steps.len() {
                sequence.steps[paste_idx] = step.clone();
                state.selected_notes.insert(paste_idx);
            }
        }
    }

    // Header toolbar
    ui.horizontal(|ui| {
        ui.selectable_value(&mut state.mode, PianoRollMode::StepGrid, "Step Grid");
        ui.selectable_value(&mut state.mode, PianoRollMode::PianoRoll, "Piano Roll");
        ui.separator();
        ui.label("Snap Grid:");
        egui::ComboBox::from_id_source("piano_roll_snap_combo")
            .selected_text(format!("1/{:.0}", 1.0 / state.snap_division))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.snap_division, 1.0, "1/1 (Bar)");
                ui.selectable_value(&mut state.snap_division, 0.5, "1/2 (Half)");
                ui.selectable_value(&mut state.snap_division, 0.25, "1/4 (Beat)");
                ui.selectable_value(&mut state.snap_division, 0.125, "1/8 (Eighth)");
                ui.selectable_value(&mut state.snap_division, 0.0625, "1/16 (Sixteenth)");
            });

        if ui.button("🎯 Quantize").clicked() {
            quantize_notes(sequence, state.snap_division, &state.selected_notes);
        }
        ui.separator();
        if ui.button("🌀 Euclidean").clicked() {
            state.show_euclidean_popup = !state.show_euclidean_popup;
        }
        if ui.button("◀ Shift L").clicked() {
            sequence.steps.rotate_left(1);
        }
        if ui.button("▶ Shift R").clicked() {
            sequence.steps.rotate_right(1);
        }
        if ui.button("🔄 Reverse").clicked() {
            sequence.steps.reverse();
        }
        if ui.button("🪞 Mirror").clicked() {
            for step in &mut sequence.steps {
                step.active = !step.active;
                if !step.active {
                    step.gate = 0.0;
                } else if step.gate <= 0.0 {
                    step.gate = 0.8;
                }
            }
        }
        ui.separator();
        if let Some(hc) = harmonic_ctx {
            ui.label(egui::RichText::new(format!("Scale: {} (Root: C)", hc.scale.name)).color(egui::Color32::from_rgb(26, 140, 255)));
        }
        ui.separator();
        ui.label(format!("Selected: {}", state.selected_notes.len()));
    });

    if state.show_euclidean_popup {
        ui.horizontal(|ui| {
            ui.label("Hits:");
            ui.add(egui::Slider::new(&mut state.euclidean_pulses, 1..=32));
            ui.label("Steps:");
            ui.add(egui::Slider::new(&mut state.euclidean_steps, 1..=32));
            if ui.button("Apply Euclidean Rhythm").clicked() {
                let rhythm = summoner_sequencer::generative::GenerativeEngine::euclidean_rhythm(
                    state.euclidean_pulses,
                    state.euclidean_steps,
                );
                summoner_sequencer::generative::GenerativeEngine::apply_rhythm_to_sequence(&rhythm, &mut sequence.steps);
            }
        });
        ui.separator();
    }

    if let Some(hc) = harmonic_ctx {
        ui.collapsing("🎼 Real-Time Chord Suggestions", |ui| {
            crate::views::chord_suggestion_panel::show_chord_suggestion_panel(ui, hc, sequence, 0.0);
        });
    }

    ui.separator();

    match state.mode {
        PianoRollMode::StepGrid => {
            let step_width = 40.0;
            let step_height = 120.0;

            egui::ScrollArea::horizontal().show(ui, |ui| {
                ui.horizontal(|ui| {
                    let mut i = 0;
                    while i < sequence.steps.len() {
                        let step = &mut sequence.steps[i];

                        ui.allocate_ui(egui::vec2(step_width, step_height), |ui| {
                            ui.vertical(|ui| {
                                let mut active = step.gate > 0.0;
                                if ui.toggle_value(&mut active, format!("{}", i + 1)).changed() {
                                    step.gate = if active { 0.5 } else { 0.0 };
                                    step.active = active;
                                }

                                ui.add(
                                    egui::Slider::new(&mut step.velocity, 0.0..=1.0)
                                        .orientation(egui::SliderOrientation::Vertical),
                                );

                                let prob = step.probability;
                                ui.label(format!("{:.0}%", prob * 100.0));
                            });
                        })
                        .response
                        .context_menu(|ui| {
                            ui.heading("Step Properties");
                            ui.add(egui::Slider::new(&mut step.ratchet, 1..=16).text("Ratchet"));
                            ui.add(
                                egui::Slider::new(&mut step.micro_shift, -64..=64)
                                    .text("Micro Shift"),
                            );
                            ui.add(egui::Slider::new(&mut step.swing, 0.0..=1.0).text("Swing"));
                            ui.add(egui::Slider::new(&mut step.pan, -1.0..=1.0).text("Pan"));
                            ui.add(egui::Slider::new(&mut step.pitch_offset, -100.0..=100.0).text("Pitch Cents"));
                        });
                        i += 1;
                    }
                });
            });
        }
        PianoRollMode::PianoRoll => {
            let keys_per_octave = (tuning.divisions as usize).max(1);
            let num_octaves = 8;
            let total_keys = keys_per_octave * num_octaves;
            let key_height = 14.0;
            let beat_width = 80.0;
            let num_beats = (sequence.steps.len() as f64 * sequence.step_division).max(16.0) as f32;

            let canvas_width = 24.0 + (num_beats * beat_width);
            let canvas_height = total_keys as f32 * key_height;

            egui::ScrollArea::both().show(ui, |ui| {
                let (response, painter) = ui.allocate_painter(
                    egui::vec2(canvas_width, canvas_height),
                    egui::Sense::click_and_drag(),
                );
                let canvas_rect = response.rect;

                // Draw 24-px left piano keyboard strip
                for k in 0..total_keys {
                    let y_bottom = canvas_rect.bottom() - (k as f32) * key_height;
                    let y_top = y_bottom - key_height;
                    let key_rect = egui::Rect::from_min_max(
                        egui::pos2(canvas_rect.left(), y_top),
                        egui::pos2(canvas_rect.left() + 24.0, y_bottom),
                    );

                    let pitch_class = k % keys_per_octave;
                    let octave = k / keys_per_octave;
                    let root_pc = harmonic_ctx.map_or(0, |hc| (hc.root_note % keys_per_octave as u16) as usize);
                    let rel_pc = ((pitch_class as i32 - root_pc as i32).rem_euclid(keys_per_octave as i32)) as u16;
                    let is_in_scale = harmonic_ctx.map_or(true, |hc| hc.scale.degrees.contains(&rel_pc));
                    let scale_name = harmonic_ctx.map_or("Chromatic", |hc| hc.scale.name.as_str());

                    let is_black = if keys_per_octave == 12 {
                        matches!(pitch_class, 1 | 3 | 6 | 8 | 10)
                    } else {
                        pitch_class % 2 != 0
                    };

                    let bg_color = if is_in_scale {
                        if is_black {
                            egui::Color32::from_rgb(35, 55, 80)
                        } else {
                            egui::Color32::from_rgb(220, 235, 255)
                        }
                    } else {
                        if is_black {
                            egui::Color32::from_gray(18)
                        } else {
                            egui::Color32::from_gray(115)
                        }
                    };
                    painter.rect_filled(key_rect, 0.0, bg_color);

                    let border_stroke = if rel_pc == 0 && is_in_scale {
                        egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(255, 200, 50))
                    } else if is_in_scale {
                        egui::Stroke::new(0.8_f32, egui::Color32::from_rgb(26, 140, 255))
                    } else {
                        egui::Stroke::new(0.5_f32, egui::Color32::from_gray(60))
                    };
                    painter.rect_stroke(key_rect, 0.0, border_stroke);

                    if pitch_class == 0 {
                        let text_color = if is_black || is_in_scale {
                            egui::Color32::BLACK
                        } else {
                            egui::Color32::WHITE
                        };
                        painter.text(
                            key_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            format!("C{}", octave),
                            egui::FontId::proportional(9.0),
                            text_color,
                        );
                    }

                    // Hover tooltip with Hz + note label + scale status (Step 354)
                    let key_interact = ui.interact(key_rect, ui.id().with(("key", k)), egui::Sense::hover());
                    let freq = tuning.note_to_freq(k as f64);
                    key_interact.on_hover_text(format!(
                        "Note {} (Oct {}, Deg {}): {:.1} Hz | Scale: {} (In Scale: {})",
                        k, octave, pitch_class, freq, scale_name, if is_in_scale { "Yes" } else { "No" }
                    ));
                }

                // Draw background grid lines on piano roll canvas area (x > left + 24.0)
                let roll_left = canvas_rect.left() + 24.0;
                let roll_width = canvas_rect.width() - 24.0;

                // Horizontal key lines
                for k in 0..total_keys {
                    let y = canvas_rect.bottom() - (k as f32) * key_height;
                    painter.line_segment(
                        [egui::pos2(roll_left, y), egui::pos2(canvas_rect.right(), y)],
                        egui::Stroke::new(0.5_f32, egui::Color32::from_gray(25)),
                    );
                }

                // Vertical beat grid lines
                let total_beat_count = (roll_width / beat_width) as usize;
                for b in 0..=total_beat_count {
                    let x = roll_left + b as f32 * beat_width;
                    let is_bar = b % 4 == 0;
                    painter.line_segment(
                        [egui::pos2(x, canvas_rect.top()), egui::pos2(x, canvas_rect.bottom())],
                        egui::Stroke::new(if is_bar { 1.2_f32 } else { 0.5_f32 }, if is_bar { egui::Color32::from_gray(70) } else { egui::Color32::from_gray(35) }),
                    );
                }

                // Loop bracket range selection overlay
                let loop_start_x = roll_left + (state.loop_start as f32 * beat_width);
                let loop_end_x = roll_left + (state.loop_end as f32 * beat_width);
                let loop_rect = egui::Rect::from_min_max(
                    egui::pos2(loop_start_x.max(roll_left), canvas_rect.top()),
                    egui::pos2(loop_end_x.min(canvas_rect.right()), canvas_rect.top() + 12.0),
                );
                painter.rect_filled(loop_rect, 2.0, egui::Color32::from_rgba_unmultiplied(26, 140, 255, 60));
                painter.text(
                    loop_rect.left_center() + egui::vec2(4.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    "🔁 LOOP",
                    egui::FontId::proportional(9.0),
                    egui::Color32::WHITE,
                );

                // Render notes from sequence
                let mut note_to_delete: Option<usize> = None;
                let pointer_pos = ui.input(|i| i.pointer.hover_pos().or_else(|| i.pointer.interact_pos()));

                for (idx, step) in sequence.steps.iter_mut().enumerate() {
                    if !step.active || step.gate <= 0.0 {
                        continue;
                    }

                    let start_beat = idx as f64 * sequence.step_division;
                    let beat_duration = (step.gate as f64).max(0.1) * sequence.step_division;
                    let k = (step.note.round() as usize).min(total_keys - 1);

                    let note_left = roll_left + (start_beat as f32 * beat_width);
                    let note_right = note_left + (beat_duration as f32 * beat_width);
                    let note_bottom = canvas_rect.bottom() - (k as f32) * key_height;
                    let note_top = note_bottom - key_height;
                    let note_rect = egui::Rect::from_min_max(
                        egui::pos2(note_left, note_top + 1.0),
                        egui::pos2(note_right, note_bottom - 1.0),
                    );

                    let is_selected = state.selected_notes.contains(&idx);
                    let fill_color = if is_selected {
                        egui::Color32::from_rgb(255, 160, 40)
                    } else {
                        egui::Color32::from_rgb(26, 140, 255)
                    };

                    painter.rect_filled(note_rect, 3.0, fill_color);
                    painter.rect_stroke(
                        note_rect,
                        3.0,
                        egui::Stroke::new(1.0_f32, egui::Color32::WHITE),
                    );

                    // Note drag handle for right edge resizing
                    let handle_rect = egui::Rect::from_min_max(
                        egui::pos2(note_right - 6.0, note_top),
                        egui::pos2(note_right + 2.0, note_bottom),
                    );
                    let handle_interact = ui.interact(
                        handle_rect,
                        ui.id().with(("resize_handle", idx)),
                        egui::Sense::drag(),
                    );
                    if handle_interact.dragged() {
                        let delta_x = handle_interact.drag_delta().x;
                        let delta_beats = (delta_x / beat_width) as f64;
                        let new_gate = (step.gate as f64 + (delta_beats / sequence.step_division)).max(0.1);
                        step.gate = new_gate as f32;
                    }

                    // Context menu for note delete / selection
                    let note_interact = ui.interact(
                        note_rect,
                        ui.id().with(("note_rect", idx)),
                        egui::Sense::click(),
                    );
                    if note_interact.clicked() {
                        if !ui.input(|i| i.modifiers.shift) {
                            state.selected_notes.clear();
                        }
                        state.selected_notes.insert(idx);
                    }

                    note_interact.context_menu(|ui| {
                        if ui.button("🗑 Delete Note").clicked() {
                            note_to_delete = Some(idx);
                            ui.close_menu();
                        }
                    });
                }

                if let Some(del_idx) = note_to_delete {
                    if del_idx < sequence.steps.len() {
                        sequence.steps[del_idx].active = false;
                        sequence.steps[del_idx].gate = 0.0;
                        state.selected_notes.remove(&del_idx);
                    }
                }

                // Add note when clicking empty area on canvas
                if response.clicked() {
                    if let Some(pos) = pointer_pos {
                        if pos.x > roll_left {
                            let rel_x = pos.x - roll_left;
                            let click_beat = (rel_x / beat_width) as f64;
                            let snap = state.snap_division.max(0.01);
                            let snapped_beat = (click_beat / snap).floor() * snap;
                            let step_idx = (snapped_beat / sequence.step_division).floor() as usize;

                            let y_offset = canvas_rect.bottom() - pos.y;
                            let clicked_pitch = (y_offset / key_height).floor() as f64;

                            if step_idx < sequence.steps.len() {
                                sequence.steps[step_idx].note = clicked_pitch;
                                sequence.steps[step_idx].active = true;
                                sequence.steps[step_idx].gate = 1.0;
                                sequence.steps[step_idx].velocity = 0.8;
                                state.selected_notes.clear();
                                state.selected_notes.insert(step_idx);
                            }
                        }
                    }
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.label("📊 Velocity Bars");

            // Velocity Bars Panel below piano roll canvas
            let velocity_panel_height = 60.0;
            let (vel_response, vel_painter) = ui.allocate_painter(
                egui::vec2(canvas_width, velocity_panel_height),
                egui::Sense::click_and_drag(),
            );
            let vel_rect = vel_response.rect;
            vel_painter.rect_filled(vel_rect, 4.0, egui::Color32::from_rgb(18, 18, 24));

            let roll_left = vel_rect.left() + 24.0;
            let step_width_px = sequence.step_division as f32 * beat_width;

            for (i, step) in sequence.steps.iter_mut().enumerate() {
                let bar_x = roll_left + (i as f32 * step_width_px);
                let bar_w = (step_width_px - 2.0).max(4.0);
                let bar_h = (step.velocity * (velocity_panel_height - 10.0)).clamp(2.0, velocity_panel_height - 10.0);
                let bar_rect = egui::Rect::from_min_size(
                    egui::pos2(bar_x, vel_rect.bottom() - 5.0 - bar_h),
                    egui::vec2(bar_w, bar_h),
                );

                let bar_color = if step.active {
                    egui::Color32::from_rgb(46, 204, 113)
                } else {
                    egui::Color32::from_gray(60)
                };
                vel_painter.rect_filled(bar_rect, 2.0, bar_color);

                // Drag/Click on velocity panel to adjust velocity
                let bar_col_rect = egui::Rect::from_min_size(
                    egui::pos2(bar_x, vel_rect.top()),
                    egui::vec2(bar_w, velocity_panel_height),
                );
                let bar_interact = ui.interact(
                    bar_col_rect,
                    ui.id().with(("vel_bar", i)),
                    egui::Sense::click_and_drag(),
                );

                if bar_interact.clicked() || bar_interact.dragged() {
                    if let Some(pos) = ui.input(|inp| inp.pointer.interact_pos()) {
                        let new_vel = ((vel_rect.bottom() - 5.0 - pos.y) / (velocity_panel_height - 10.0))
                            .clamp(0.0, 1.0);
                        step.velocity = new_vel;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piano_roll_renders_without_panic() {
        let mut sequence = SequenceConfig {
            start_beat: 0.0,
            step_division: 0.25,
            clip_color: None,
            clip_name: None,
            name: "Clip".to_string(),
            is_unique: true,
            steps: vec![
                TrackerStepConfig {
                    note: 60.0,
                    velocity: 0.8,
                    gate: 0.5,
                    probability: 1.0,
                    ratchet: 1,
                    micro_shift: 0,
                    swing: 0.0,
                    pan: 0.0,
                    pitch_offset: 0.0,
                    active: true,
                };
                16
            ],
        };
        let tuning = EdoTuning::new(19, 440.0, 69.0);
        let mut state = PianoRollState::default();
        let viewport = Viewport {
            width: 800.0,
            height: 600.0,
        };

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_piano_roll(ui, &mut sequence, &tuning, &mut state, &viewport, None);
            });
        });
    }

    #[test]
    fn test_piano_roll_note_add_delete() {
        let mut sequence = SequenceConfig {
            start_beat: 0.0,
            step_division: 0.25,
            clip_color: None,
            clip_name: None,
            name: "Clip".to_string(),
            is_unique: true,
            steps: vec![
                TrackerStepConfig {
                    note: 60.0,
                    velocity: 0.0,
                    gate: 0.0,
                    probability: 1.0,
                    ratchet: 1,
                    micro_shift: 0,
                    swing: 0.0,
                    pan: 0.0,
                    pitch_offset: 0.0,
                    active: false,
                };
                16
            ],
        };
        assert!(!sequence.steps[0].active);

        // Simulate adding note to step 0
        sequence.steps[0].note = 64.0;
        sequence.steps[0].active = true;
        sequence.steps[0].gate = 1.0;
        sequence.steps[0].velocity = 0.8;

        assert!(sequence.steps[0].active);
        assert_eq!(sequence.steps[0].note, 64.0);

        // Simulate deleting note from step 0
        sequence.steps[0].active = false;
        sequence.steps[0].gate = 0.0;

        assert!(!sequence.steps[0].active);
        assert_eq!(sequence.steps[0].gate, 0.0);
    }

    #[test]
    fn test_piano_roll_velocity_bars_render() {
        let mut sequence = SequenceConfig {
            start_beat: 0.0,
            step_division: 0.25,
            clip_color: None,
            clip_name: None,
            name: "Clip".to_string(),
            is_unique: true,
            steps: vec![
                TrackerStepConfig {
                    note: 60.0,
                    velocity: 0.75,
                    gate: 0.5,
                    probability: 1.0,
                    ratchet: 1,
                    micro_shift: 0,
                    swing: 0.0,
                    pan: 0.0,
                    pitch_offset: 0.0,
                    active: true,
                };
                16
            ],
        };
        let tuning = EdoTuning::standard_12_tet();
        let mut state = PianoRollState::default();
        let viewport = Viewport {
            width: 800.0,
            height: 600.0,
        };

        // Render in egui context to verify velocity bars render loop
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_piano_roll(ui, &mut sequence, &tuning, &mut state, &viewport, None);
            });
        });

        assert_eq!(sequence.steps[0].velocity, 0.75);
    }
}

