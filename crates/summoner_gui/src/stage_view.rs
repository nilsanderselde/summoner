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

use eframe::egui;
use std::time::Instant;
use summoner_core::transport::Transport;
use summoner_project::schema::ProjectConfig;

#[derive(Clone, Debug)]
pub struct PatternSlot {
    pub track_id: u64,
    pub pattern_name: String,
    pub color: egui::Color32,
    pub armed: bool,
    pub pending_fire: bool,
    pub target_fire_beat: Option<f64>,
    pub last_fired_beat: Option<f64>,
    pub last_fired_time: Option<Instant>,
}

/// High-contrast stage view for live performance.
pub struct StageView {
    pub active: bool,
    pub panic_mode: bool,
    pub pattern_slots: Vec<Option<PatternSlot>>,
    pub quantize_beats: u32,
    pub bpm_display: f64,
    pub bpm_step: f64,
    pub tap_history: Vec<Instant>,
    pub pending_launch: Option<(usize, u64)>, // (slot_index, fire_at_frame)
    pub steam_deck: crate::platform::SteamDeckControllerState,
}

impl StageView {
    pub fn new() -> Self {
        Self {
            active: false,
            panic_mode: false,
            pattern_slots: vec![None; 16], // 4x4 grid
            quantize_beats: 4,
            bpm_display: 120.0,
            bpm_step: 1.0,
            tap_history: Vec::new(),
            pending_launch: None,
            steam_deck: crate::platform::SteamDeckControllerState::detect(),
        }
    }

    /// Step 322: Calculate next launch frame position rounded to quantize boundary.
    pub fn calculate_next_fire_frame(
        current_frame: u64,
        sample_rate: u32,
        bpm: f64,
        quantize_beats: u32,
    ) -> u64 {
        if sample_rate == 0 || bpm <= 0.0 || quantize_beats == 0 {
            return current_frame;
        }
        let beats_per_sec = bpm / 60.0;
        let frames_per_beat = (sample_rate as f64 / beats_per_sec).max(1.0);
        let quantize_frames = (frames_per_beat * quantize_beats as f64) as u64;
        if quantize_frames == 0 {
            return current_frame;
        }
        let frames_f = current_frame as f64;
        let q_f = quantize_frames as f64;
        (frames_f / q_f).ceil() as u64 * quantize_frames
    }

    pub fn populate_from_project(&mut self, project: &ProjectConfig) {
        let colors = [
            egui::Color32::from_rgb(220, 80, 80),   // Red
            egui::Color32::from_rgb(80, 180, 220),  // Cyan
            egui::Color32::from_rgb(120, 220, 100), // Green
            egui::Color32::from_rgb(220, 180, 60),  // Gold
            egui::Color32::from_rgb(180, 100, 220), // Purple
            egui::Color32::from_rgb(220, 120, 180), // Pink
            egui::Color32::from_rgb(100, 220, 200), // Teal
            egui::Color32::from_rgb(240, 140, 60),  // Orange
        ];
        for (i, track) in project.tracks.iter().enumerate().take(16) {
            let color = colors[i % colors.len()];
            self.pattern_slots[i] = Some(PatternSlot {
                track_id: track.id,
                pattern_name: track.name.clone(),
                color,
                armed: false,
                pending_fire: false,
                target_fire_beat: None,
                last_fired_beat: None,
                last_fired_time: None,
            });
        }
    }

    pub fn register_tap(&mut self, now: Instant) -> Option<f64> {
        self.tap_history
            .retain(|t| now.duration_since(*t).as_secs_f32() < 3.0);
        self.tap_history.push(now);
        if self.tap_history.len() > 4 {
            self.tap_history.remove(0);
        }
        if self.tap_history.len() >= 2 {
            let intervals: Vec<f64> = self
                .tap_history
                .windows(2)
                .map(|w| w[1].duration_since(w[0]).as_secs_f64())
                .collect();
            let avg_interval: f64 = intervals.iter().sum::<f64>() / intervals.len() as f64;
            if avg_interval > 0.0 {
                let bpm = (60.0 / avg_interval).clamp(20.0, 300.0);
                self.bpm_display = (bpm * 10.0).round() / 10.0;
                return Some(self.bpm_display);
            }
        }
        None
    }

    pub fn all_stop(&mut self, transport: &mut Transport) {
        transport.stop();
        self.pending_launch = None;
        for slot in self.pattern_slots.iter_mut().flatten() {
            slot.armed = false;
            slot.pending_fire = false;
            slot.target_fire_beat = None;
        }
    }

    pub fn trigger_panic(&mut self) {
        self.panic_mode = true;
    }

    pub fn clear_panic(&mut self) {
        self.panic_mode = false;
    }
}

fn key_to_slot_index(key: egui::Key) -> Option<usize> {
    match key {
        egui::Key::Num1 => Some(0),
        egui::Key::Num2 => Some(1),
        egui::Key::Num3 => Some(2),
        egui::Key::Num4 => Some(3),
        egui::Key::Num5 => Some(4),
        egui::Key::Num6 => Some(5),
        egui::Key::Num7 => Some(6),
        egui::Key::Num8 => Some(7),
        egui::Key::Num9 => Some(8),
        egui::Key::Num0 => Some(9),
        egui::Key::A => Some(10),
        egui::Key::B => Some(11),
        egui::Key::C => Some(12),
        egui::Key::D => Some(13),
        egui::Key::E => Some(14),
        egui::Key::F => Some(15),
        _ => None,
    }
}

fn slot_hotkey_label(idx: usize) -> &'static str {
    match idx {
        0 => "1",
        1 => "2",
        2 => "3",
        3 => "4",
        4 => "5",
        5 => "6",
        6 => "7",
        7 => "8",
        8 => "9",
        9 => "0",
        10 => "A",
        11 => "B",
        12 => "C",
        13 => "D",
        14 => "E",
        15 => "F",
        _ => "",
    }
}

fn trigger_slot_launch(idx: usize, stage: &mut StageView, transport: &mut Transport) {
    let q = stage.quantize_beats as f64;
    let current_beat = transport.beats();
    if let Some(slot) = &mut stage.pattern_slots[idx] {
        if slot.armed || slot.pending_fire {
            // Disarm / cancel pending launch
            slot.armed = false;
            slot.pending_fire = false;
            slot.target_fire_beat = None;
            if stage.pending_launch.map(|p| p.0) == Some(idx) {
                stage.pending_launch = None;
            }
        } else {
            if transport.is_playing {
                let target = ((current_beat / q).floor() + 1.0) * q;
                let target_frame = StageView::calculate_next_fire_frame(
                    transport.frame_position,
                    transport.sample_rate,
                    transport.bpm,
                    stage.quantize_beats,
                );
                slot.pending_fire = true;
                slot.target_fire_beat = Some(target);
                stage.pending_launch = Some((idx, target_frame));
            } else {
                slot.armed = true;
                slot.pending_fire = false;
                slot.last_fired_beat = Some(current_beat);
                slot.last_fired_time = Some(Instant::now());
                stage.pending_launch = None;
                transport.play();
            }
        }
    }
}

pub fn show_stage_view(ui: &mut egui::Ui, stage: &mut StageView, transport: &mut Transport) {
    // Step 137: Keep stage.bpm_display synced with transport.bpm if stage view hasn't updated it
    if (stage.bpm_display - transport.bpm).abs() > 0.001 && stage.tap_history.is_empty() {
        stage.bpm_display = transport.bpm;
    }

    // Step 139: Process pending pattern launches when transport reaches quantize beat boundary
    let current_beat = transport.beats();
    for slot in stage.pattern_slots.iter_mut().flatten() {
        if slot.pending_fire {
            if let Some(target) = slot.target_fire_beat {
                if current_beat >= target {
                    slot.pending_fire = false;
                    slot.armed = true;
                    slot.target_fire_beat = None;
                    slot.last_fired_beat = Some(current_beat);
                    slot.last_fired_time = Some(Instant::now());
                    transport.is_playing = true;
                } else {
                    ui.ctx().request_repaint();
                }
            }
        }
    }

    // Step 143: Keyboard shortcuts for pattern slots (1-9, 0, A-F)
    let hotkeys = [
        egui::Key::Num1,
        egui::Key::Num2,
        egui::Key::Num3,
        egui::Key::Num4,
        egui::Key::Num5,
        egui::Key::Num6,
        egui::Key::Num7,
        egui::Key::Num8,
        egui::Key::Num9,
        egui::Key::Num0,
        egui::Key::A,
        egui::Key::B,
        egui::Key::C,
        egui::Key::D,
        egui::Key::E,
        egui::Key::F,
    ];
    for key in hotkeys {
        if ui.input(|i| i.key_pressed(key)) {
            if let Some(idx) = key_to_slot_index(key) {
                trigger_slot_launch(idx, stage, transport);
            }
        }
    }

    // Step 536: Steam Deck controller D-pad / Arrow key grid navigation
    if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
        stage.steam_deck.navigate_grid(-1, 0);
    }
    if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
        stage.steam_deck.navigate_grid(1, 0);
    }
    if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
        stage.steam_deck.navigate_grid(0, -1);
    }
    if ui.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
        stage.steam_deck.navigate_grid(0, 1);
    }
    if ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space)) {
        let focused = stage.steam_deck.focused_slot;
        trigger_slot_launch(focused, stage, transport);
    }

    let dark_bg = egui::Color32::from_rgb(10, 10, 10);
    egui::Frame::none().fill(dark_bg).show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);

            // BPM, Tap Tempo, BPM Step & Quantize Header
            ui.horizontal(|ui| {
                ui.add_space(20.0);

                // Step 138: Tap tempo button with BPM display
                let bpm_str = format!("BPM: {:.1}", stage.bpm_display);
                let bpm_btn = ui.add(
                    egui::Button::new(
                        egui::RichText::new(&bpm_str)
                            .size(40.0)
                            .strong()
                            .color(egui::Color32::WHITE),
                    )
                    .fill(egui::Color32::from_gray(25)),
                );

                if bpm_btn.clicked() {
                    if let Some(new_bpm) = stage.register_tap(Instant::now()) {
                        transport.bpm = new_bpm;
                    }
                }
                if bpm_btn.hovered() {
                    ui.painter().text(
                        bpm_btn.rect.center_bottom() + egui::vec2(0.0, 12.0),
                        egui::Align2::CENTER_TOP,
                        "Click to Tap Tempo",
                        egui::FontId::proportional(12.0),
                        egui::Color32::LIGHT_BLUE,
                    );
                }

                ui.add_space(10.0);

                // Step 144: BPM nudge +/- buttons and step selector
                if ui
                    .button(egui::RichText::new("-").size(24.0).strong())
                    .clicked()
                {
                    stage.bpm_display = (stage.bpm_display - stage.bpm_step).max(20.0);
                    transport.bpm = stage.bpm_display;
                }
                if ui
                    .button(egui::RichText::new("+").size(24.0).strong())
                    .clicked()
                {
                    stage.bpm_display = (stage.bpm_display + stage.bpm_step).min(300.0);
                    transport.bpm = stage.bpm_display;
                }

                egui::ComboBox::from_id_source("bpm_step_combo")
                    .selected_text(format!("±{:.1}", stage.bpm_step))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut stage.bpm_step, 0.1, "Step 0.1");
                        ui.selectable_value(&mut stage.bpm_step, 1.0, "Step 1.0");
                    });

                ui.add_space(30.0);

                // Step 139: Quantize dropdown selector
                egui::ComboBox::from_id_source("quantize")
                    .selected_text(format!("Quantize: {} Beats", stage.quantize_beats))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut stage.quantize_beats, 1, "1 Beat");
                        ui.selectable_value(&mut stage.quantize_beats, 2, "2 Beats");
                        ui.selectable_value(&mut stage.quantize_beats, 4, "4 Beats");
                        ui.selectable_value(&mut stage.quantize_beats, 8, "8 Beats");
                    });
            });

            ui.add_space(30.0);

            // 4x4 Grid
            egui::Grid::new("stage_grid")
                .spacing(egui::vec2(20.0, 20.0))
                .show(ui, |ui| {
                    for row in 0..4 {
                        for col in 0..4 {
                            let idx = row * 4 + col;
                            let hk = slot_hotkey_label(idx);

                            let (text, color, is_armed, is_pending) =
                                if let Some(s) = &stage.pattern_slots[idx] {
                                    (
                                        format!("[{}] {}", hk, s.pattern_name),
                                        s.color,
                                        s.armed,
                                        s.pending_fire,
                                    )
                                } else {
                                    (
                                        format!("[{}] Empty", hk),
                                        egui::Color32::from_gray(30),
                                        false,
                                        false,
                                    )
                                };

                            let button_rect = ui.allocate_space(egui::vec2(120.0, 120.0)).1;
                            let response =
                                ui.interact(button_rect, ui.id().with(idx), egui::Sense::click());

                            let fill_color = if is_pending {
                                // Blinking amber for pending launch
                                let time = ui.input(|i| i.time);
                                let blink = (time * 10.0).sin().abs() as f32;
                                ui.ctx().request_repaint();
                                egui::Color32::from_rgb(220, 160, 40)
                                    .linear_multiply(0.4 + 0.6 * blink)
                            } else if is_armed {
                                // Pulsing glow animation
                                let time = ui.input(|i| i.time);
                                let pulse = (time * 5.0).sin().abs() as f32;
                                ui.ctx().request_repaint();
                                color.linear_multiply(0.5 + 0.5 * pulse)
                            } else {
                                color.linear_multiply(0.3)
                            };

                            ui.painter().rect_filled(button_rect, 10.0, fill_color);

                            let stroke_color = if response.hovered() {
                                egui::Color32::WHITE
                            } else if is_pending {
                                egui::Color32::YELLOW
                            } else {
                                color
                            };
                            ui.painter().rect_stroke(
                                button_rect,
                                10.0,
                                egui::Stroke::new(2.0f32, stroke_color),
                            );

                            let display_text = if is_pending {
                                format!("{}\n(WAIT)", text)
                            } else {
                                text
                            };
                            ui.painter().text(
                                button_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                display_text,
                                egui::FontId::proportional(18.0),
                                egui::Color32::WHITE,
                            );

                            // Step 145: Loop progress bar indicator inside each lit/armed slot
                            if is_armed {
                                let q = stage.quantize_beats as f64;
                                let progress = ((current_beat % q) / q) as f32;
                                let progress_width = (button_rect.width() - 16.0) * progress;
                                let bar_rect = egui::Rect::from_min_max(
                                    egui::pos2(
                                        button_rect.left() + 8.0,
                                        button_rect.bottom() - 12.0,
                                    ),
                                    egui::pos2(
                                        button_rect.left() + 8.0 + progress_width,
                                        button_rect.bottom() - 6.0,
                                    ),
                                );
                                ui.painter()
                                    .rect_filled(bar_rect, 3.0, egui::Color32::WHITE);
                                ui.ctx().request_repaint();
                            }

                            // Step 146: Flashing border transition animation when pattern fires
                            if let Some(slot) = &stage.pattern_slots[idx] {
                                if let Some(fired_t) = slot.last_fired_time {
                                    let elapsed = fired_t.elapsed().as_secs_f32();
                                    if elapsed < 0.3 {
                                        let alpha = 1.0 - (elapsed / 0.3);
                                        let flash_stroke = egui::Stroke::new(
                                            4.0f32,
                                            egui::Color32::WHITE.linear_multiply(alpha),
                                        );
                                        ui.painter().rect_stroke(button_rect, 10.0, flash_stroke);
                                        ui.ctx().request_repaint();
                                    }
                                }
                            }

                            if response.clicked() {
                                trigger_slot_launch(idx, stage, transport);
                            }
                        }
                        ui.end_row();
                    }
                });

            ui.add_space(40.0);

            // Toolbar with ALL STOP (Step 142) and Panic (ESC) Buttons
            ui.horizontal(|ui| {
                ui.add_space(20.0);

                // Step 142: ALL STOP Button
                let stop_rect = ui.allocate_space(egui::vec2(220.0, 60.0)).1;
                let stop_response =
                    ui.interact(stop_rect, ui.id().with("all_stop"), egui::Sense::click());
                let stop_color = if stop_response.hovered() {
                    egui::Color32::from_rgb(240, 120, 40)
                } else {
                    egui::Color32::from_rgb(180, 70, 20)
                };
                ui.painter().rect_filled(stop_rect, 12.0, stop_color);
                ui.painter().text(
                    stop_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "ALL STOP",
                    egui::FontId::proportional(24.0),
                    egui::Color32::WHITE,
                );
                if stop_response.clicked() {
                    stage.all_stop(transport);
                }

                ui.add_space(40.0);

                // Panic Button
                let panic_rect = ui.allocate_space(egui::vec2(220.0, 60.0)).1;
                let panic_response =
                    ui.interact(panic_rect, ui.id().with("panic"), egui::Sense::click());
                let panic_color = if panic_response.hovered() {
                    egui::Color32::from_rgb(255, 100, 100)
                } else {
                    egui::Color32::from_rgb(200, 0, 0)
                };

                ui.painter().rect_filled(panic_rect, 12.0, panic_color);
                ui.painter().text(
                    panic_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "PANIC (ESC)",
                    egui::FontId::proportional(24.0),
                    egui::Color32::WHITE,
                );

                if panic_response.clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    stage.trigger_panic();
                }
            });
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_stage_view_renders_without_panic() {
        let mut stage = StageView::new();
        stage.pattern_slots[0] = Some(PatternSlot {
            track_id: 1,
            pattern_name: "Kick".to_string(),
            color: egui::Color32::RED,
            armed: false,
            pending_fire: false,
            target_fire_beat: None,
            last_fired_beat: None,
            last_fired_time: None,
        });
        stage.pattern_slots[1] = Some(PatternSlot {
            track_id: 1,
            pattern_name: "Snare".to_string(),
            color: egui::Color32::BLUE,
            armed: true,
            pending_fire: false,
            target_fire_beat: None,
            last_fired_beat: None,
            last_fired_time: None,
        });

        let mut transport = Transport::new(44100, 120.0);

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_stage_view(ui, &mut stage, &mut transport);
            });
        });

        assert_eq!(
            stage.pattern_slots.iter().filter(|s| s.is_some()).count(),
            2
        );
    }

    #[test]
    fn test_stage_view_tap_tempo() {
        let mut stage = StageView::new();
        let now = Instant::now();

        // Simulate 4 taps spaced exactly 0.5s apart (120 BPM)
        stage.register_tap(now);
        stage.register_tap(now + Duration::from_millis(500));
        stage.register_tap(now + Duration::from_millis(1000));
        let bpm = stage.register_tap(now + Duration::from_millis(1500));

        assert!(bpm.is_some());
        let val = bpm.unwrap();
        assert!((val - 120.0).abs() < 1.0, "Expected ~120 BPM, got {}", val);
    }

    #[test]
    fn test_stage_view_populate_from_project() {
        let mut stage = StageView::new();
        let project = ProjectConfig {
            name: "Test Project".to_string(),
            tuning_file: None,
            transport: summoner_project::schema::TransportConfig::default(),
            tracks: vec![
                summoner_project::schema::TrackConfig {
                    id: 10,
                    name: "Bass Synthesizer".to_string(),
                    channels: 2,
                    gain: 1.0,
                    pan: 0.0,
                    muted: false,
                    soloed: false,
                    send_level: 0.0,
                    nodes: vec![],
                    sequence: None,
                    clips: vec![],
                    connections: vec![],
                    tuning_edo: Some(12),
                    tuning_root_hz: Some(440.0),
                    tuning_scl_path: None,
                    ..Default::default()
                },
                summoner_project::schema::TrackConfig {
                    id: 20,
                    name: "Lead Synth".to_string(),
                    channels: 2,
                    gain: 1.0,
                    pan: 0.0,
                    muted: false,
                    soloed: false,
                    send_level: 0.0,
                    nodes: vec![],
                    sequence: None,
                    clips: vec![],
                    connections: vec![],
                    tuning_edo: Some(12),
                    tuning_root_hz: Some(440.0),
                    tuning_scl_path: None,
                    ..Default::default()
                },
            ],
            assets: vec![],
            automation_lanes: vec![],
            midi_mappings: vec![],
            ..Default::default()
        };

        stage.populate_from_project(&project);

        assert!(stage.pattern_slots[0].is_some());
        assert_eq!(
            stage.pattern_slots[0].as_ref().unwrap().pattern_name,
            "Bass Synthesizer"
        );
        assert!(stage.pattern_slots[1].is_some());
        assert_eq!(
            stage.pattern_slots[1].as_ref().unwrap().pattern_name,
            "Lead Synth"
        );
    }

    #[test]
    fn test_stage_view_all_stop() {
        let mut stage = StageView::new();
        stage.pattern_slots[0] = Some(PatternSlot {
            track_id: 1,
            pattern_name: "Synth".to_string(),
            color: egui::Color32::GREEN,
            armed: true,
            pending_fire: true,
            target_fire_beat: Some(4.0),
            last_fired_beat: None,
            last_fired_time: None,
        });

        let mut transport = Transport::new(44100, 120.0);
        transport.play();
        assert!(transport.is_playing);

        stage.all_stop(&mut transport);

        assert!(!transport.is_playing);
        let slot = stage.pattern_slots[0].as_ref().unwrap();
        assert!(!slot.armed);
        assert!(!slot.pending_fire);
        assert!(slot.target_fire_beat.is_none());
        assert!(stage.pending_launch.is_none());
    }

    #[test]
    fn test_stage_view_pending_launch_frame_quantization() {
        // At 44100 Hz, 120 BPM, 1 beat = 22050 frames. 4 beats quantize = 88200 frames.
        let target_frame = StageView::calculate_next_fire_frame(1000, 44100, 120.0, 4);
        assert_eq!(target_frame, 88200);

        let target_frame_exact = StageView::calculate_next_fire_frame(88200, 44100, 120.0, 4);
        assert_eq!(target_frame_exact, 88200);
    }
}
