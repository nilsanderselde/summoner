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

//! Dedicated MIDI Controller Mapping, MIDI Monitor, and Virtual Keyboard GUI components.

use eframe::egui;
use summoner_sequencer::midi_tools::{
    generate_panic_all_note_off, qwerty_key_to_midi_note, transform_velocity, MidiControllerMapping,
    MidiMappingType, MidiMonitorLog, VelocityCurve,
};

/// State for the dedicated virtual MIDI keyboard panel and window.
#[derive(Debug, Clone)]
pub struct VirtualKeyboardState {
    pub base_octave: u8,
    pub velocity: u8,
    pub pitch_bend: i16,
    pub mod_wheel: u8,
    pub qwerty_enabled: bool,
    pub velocity_curve: VelocityCurve,
    pub show_window: bool,
    pub active_held_notes: std::collections::HashSet<u8>,
}

impl Default for VirtualKeyboardState {
    fn default() -> Self {
        Self {
            base_octave: 4,
            velocity: 100,
            pitch_bend: 0,
            mod_wheel: 0,
            qwerty_enabled: true,
            velocity_curve: VelocityCurve::Linear,
            show_window: false,
            active_held_notes: std::collections::HashSet::new(),
        }
    }
}

/// Render Global MIDI Controller Mappings Panel.
pub fn show_midi_mapping_panel(
    ui: &mut egui::Ui,
    mappings: &mut Vec<MidiControllerMapping>,
) {
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.heading("🎛️ MIDI Controller Mappings");
            ui.separator();
            if ui.button("➕ Add Mapping").clicked() {
                mappings.push(MidiControllerMapping::new(
                    0,
                    MidiMappingType::CC(7),
                    "track.gain".to_string(),
                    0.0,
                    1.0,
                ));
            }
        });

        if mappings.is_empty() {
            ui.label("No MIDI controller mappings defined. Click 'Add Mapping' above.");
        } else {
            let mut remove_idx = None;
            egui::Grid::new("midi_mapping_grid")
                .striped(true)
                .spacing([12.0, 6.0])
                .show(ui, |ui| {
                    ui.label("Channel");
                    ui.label("Mapping Type");
                    ui.label("Target Parameter ID");
                    ui.label("Min Value");
                    ui.label("Max Value");
                    ui.label("Action");
                    ui.end_row();

                    for (idx, map) in mappings.iter_mut().enumerate() {
                        ui.add(egui::DragValue::new(&mut map.channel).range(0..=16));

                        match &mut map.mapping_type {
                            MidiMappingType::CC(cc) => {
                                ui.horizontal(|ui| {
                                    ui.label("CC");
                                    ui.add(egui::DragValue::new(cc).range(0..=127));
                                });
                            }
                            MidiMappingType::Aftertouch => {
                                ui.label("Aftertouch");
                            }
                            MidiMappingType::PitchBend => {
                                ui.label("Pitch Bend");
                            }
                        }

                        ui.text_edit_singleline(&mut map.target_param_id);
                        ui.add(egui::DragValue::new(&mut map.min_val).speed(0.01));
                        ui.add(egui::DragValue::new(&mut map.max_val).speed(0.01));

                        if ui.button("❌ Remove").clicked() {
                            remove_idx = Some(idx);
                        }
                        ui.end_row();
                    }
                });

            if let Some(idx) = remove_idx {
                mappings.remove(idx);
            }
        }
    });
}

/// Render MIDI Monitor Panel with event logging and panic button.
pub fn show_midi_monitor_panel(
    ui: &mut egui::Ui,
    monitor_log: &mut MidiMonitorLog,
    mut on_panic: impl FnMut(),
) {
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.heading("📊 MIDI Monitor & Status");
            ui.separator();
            if ui.button("🚨 PANIC (All Off)").clicked() {
                on_panic();
                monitor_log.log_event(0, 0, "PANIC", 123, 0);
            }
            if ui.button("🗑️ Clear Log").clicked() {
                monitor_log.clear();
            }
        });

        ui.separator();

        if monitor_log.entries.is_empty() {
            ui.label("No MIDI activity logged.");
        } else {
            egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                egui::Grid::new("midi_monitor_grid")
                    .striped(true)
                    .spacing([16.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("Time (ms)");
                        ui.label("Ch");
                        ui.label("Event Type");
                        ui.label("Data 1");
                        ui.label("Data 2");
                        ui.end_row();

                        for entry in &monitor_log.entries {
                            ui.label(format!("{}", entry.timestamp_ms));
                            ui.label(format!("{}", entry.channel));
                            ui.label(&entry.event_type);
                            ui.label(format!("{}", entry.data1));
                            ui.label(format!("{}", entry.data2));
                            ui.end_row();
                        }
                    });
            });
        }
    });
}

/// Render Dedicated Interactive Virtual MIDI Keyboard Panel.
pub fn show_virtual_keyboard_panel(
    ui: &mut egui::Ui,
    state: &mut VirtualKeyboardState,
    on_note_on: &mut impl FnMut(u8, u8),
    on_note_off: &mut impl FnMut(u8),
) {
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.heading("🎹 Dedicated Virtual MIDI Keyboard");
            ui.separator();
            ui.label("Octave:");
            if ui.button("➖ Oct -").clicked() {
                if state.base_octave > 0 {
                    state.base_octave -= 1;
                }
            }
            ui.label(format!("C{}", state.base_octave));
            if ui.button("➕ Oct +").clicked() {
                if state.base_octave < 8 {
                    state.base_octave += 1;
                }
            }
            ui.separator();

            ui.label("Vel:");
            ui.add(egui::Slider::new(&mut state.velocity, 1..=127));

            ui.separator();
            ui.label("Curve:");
            egui::ComboBox::from_id_source("vkey_curve_combo")
                .selected_text(format!("{:?}", state.velocity_curve))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut state.velocity_curve, VelocityCurve::Linear, "Linear");
                    ui.selectable_value(&mut state.velocity_curve, VelocityCurve::Logarithmic, "Logarithmic");
                    ui.selectable_value(&mut state.velocity_curve, VelocityCurve::Exponential, "Exponential");
                });

            ui.toggle_value(&mut state.qwerty_enabled, "⌨️ QWERTY Key Mode");
            ui.toggle_value(&mut state.show_window, "🗖 Pop Out Window");
        });

        ui.separator();

        // Wheels and piano keyboard row
        ui.horizontal(|ui| {
            // Pitch Bend & Mod Wheel sliders
            ui.vertical(|ui| {
                ui.label("PBend");
                if ui.add(egui::Slider::new(&mut state.pitch_bend, -8192..=8191).vertical()).changed() {
                    // Pitch bend changed
                }
                if ui.button("Reset").clicked() {
                    state.pitch_bend = 0;
                }
            });
            ui.vertical(|ui| {
                ui.label("Mod");
                ui.add(egui::Slider::new(&mut state.mod_wheel, 0..=127).vertical());
            });

            ui.separator();

            // Interactive Piano Keyboard Strip (2 octaves: 24 keys)
            let keyboard_width = 480.0;
            let keyboard_height = 90.0;
            let (rect, response) = ui.allocate_exact_size(egui::vec2(keyboard_width, keyboard_height), egui::Sense::click_and_drag());
            let painter = ui.painter_at(rect);

            // Draw background
            painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(15, 15, 20));

            let num_white = 14; // 2 octaves of white keys
            let key_width = keyboard_width / num_white as f32;

            // White key MIDI notes relative to base_octave
            let start_midi = (state.base_octave as i16 + 1) * 12;

            // Process QWERTY input if enabled
            if state.qwerty_enabled {
                let keys_check = [
                    ("Z", 0), ("S", 1), ("X", 2), ("D", 3), ("C", 4), ("V", 5), ("G", 6), ("B", 7), ("H", 8), ("N", 9), ("J", 10), ("M", 11),
                    ("Q", 12), ("2", 13), ("W", 14), ("3", 15), ("E", 16), ("R", 17), ("5", 18), ("T", 19), ("6", 20), ("Y", 21), ("7", 22), ("U", 23),
                ];
                for (k, _semi) in keys_check {
                    if let Some(note) = qwerty_key_to_midi_note(k, state.base_octave) {
                        let is_down = ui.input(|i| i.key_pressed(egui::Key::Name(k.to_uppercase().into())));
                        if is_down {
                            state.active_held_notes.insert(note);
                            let final_vel = transform_velocity(state.velocity, state.velocity_curve);
                            on_note_on(note, final_vel);
                        }
                    }
                }
            }

            // Draw white keys first
            let white_key_offsets = [0, 2, 4, 5, 7, 9, 11, 12, 14, 16, 17, 19, 21, 23];
            for (idx, &semi) in white_key_offsets.iter().enumerate() {
                let note = ((start_midi + semi) as u16).clamp(0, 127) as u8;
                let key_rect = egui::Rect::from_min_size(
                    egui::pos2(rect.min.x + idx as f32 * key_width, rect.min.y),
                    egui::vec2(key_width - 1.0, keyboard_height),
                );

                let is_pressed = state.active_held_notes.contains(&note)
                    || (response.hovered() && response.is_pointer_button_down_on() && key_rect.contains(response.interact_pointer_pos().unwrap_or_default()));

                if is_pressed && response.clicked_by(egui::PointerButton::Primary) {
                    let click_pos = response.interact_pointer_pos().unwrap_or(key_rect.center());
                    let norm_y = ((click_pos.y - key_rect.min.y) / key_rect.height()).clamp(0.1, 1.0);
                    let vel = (norm_y * 127.0) as u8;
                    let final_vel = transform_velocity(vel, state.velocity_curve);
                    on_note_on(note, final_vel);
                }

                let color = if is_pressed {
                    egui::Color32::from_rgb(26, 140, 255)
                } else {
                    egui::Color32::from_rgb(230, 230, 240)
                };

                painter.rect_filled(key_rect, 2.0, color);
                painter.rect_stroke(key_rect, 2.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 40, 50)), egui::StrokeKind::Outside);
            }

            // Draw black keys on top
            let black_key_offsets = [
                (0, 1), (1, 3), (3, 6), (4, 8), (5, 10),
                (7, 13), (8, 15), (10, 18), (11, 20), (12, 22),
            ];

            for (white_idx, semi) in black_key_offsets {
                let note = ((start_midi + semi) as u16).clamp(0, 127) as u8;
                let key_rect = egui::Rect::from_min_size(
                    egui::pos2(rect.min.x + (white_idx as f32 + 0.65) * key_width, rect.min.y),
                    egui::vec2(key_width * 0.7, keyboard_height * 0.6),
                );

                let is_pressed = state.active_held_notes.contains(&note)
                    || (response.hovered() && response.is_pointer_button_down_on() && key_rect.contains(response.interact_pointer_pos().unwrap_or_default()));

                if is_pressed && response.clicked_by(egui::PointerButton::Primary) {
                    let click_pos = response.interact_pointer_pos().unwrap_or(key_rect.center());
                    let norm_y = ((click_pos.y - key_rect.min.y) / key_rect.height()).clamp(0.1, 1.0);
                    let vel = (norm_y * 127.0) as u8;
                    let final_vel = transform_velocity(vel, state.velocity_curve);
                    on_note_on(note, final_vel);
                }

                let color = if is_pressed {
                    egui::Color32::from_rgb(255, 107, 43)
                } else {
                    egui::Color32::from_rgb(30, 30, 40)
                };

                painter.rect_filled(key_rect, 2.0, color);
                painter.rect_stroke(key_rect, 2.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(10, 10, 15)), egui::StrokeKind::Outside);
            }
        });
    });
}

/// Render pop-out window for Virtual MIDI Keyboard if enabled.
pub fn show_virtual_keyboard_window(
    ctx: &egui::Context,
    state: &mut VirtualKeyboardState,
    on_note_on: &mut impl FnMut(u8, u8),
    on_note_off: &mut impl FnMut(u8),
) {
    if state.show_window {
        let mut open = state.show_window;
        egui::Window::new("🎹 Virtual MIDI Keyboard")
            .open(&mut open)
            .resizable(true)
            .show(ctx, |ui| {
                show_virtual_keyboard_panel(ui, state, on_note_on, on_note_off);
            });
        state.show_window = open;
    }
}
