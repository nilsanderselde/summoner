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
use summoner_core::transport::Transport;

#[derive(Clone, Debug)]
pub struct PatternSlot {
    pub track_id: u64,
    pub pattern_name: String,
    pub color: egui::Color32,
    pub armed: bool,
}

/// High-contrast stage view for live performance.
pub struct StageView {
    pub active: bool,
    pub panic_mode: bool,
    pub pattern_slots: Vec<Option<PatternSlot>>,
    pub quantize_beats: u32,
    pub bpm_display: f64,
}

impl StageView {
    pub fn new() -> Self {
        Self {
            active: false,
            panic_mode: false,
            pattern_slots: vec![None; 16], // 4x4 grid
            quantize_beats: 4,
            bpm_display: 120.0,
        }
    }

    pub fn trigger_panic(&mut self) {
        self.panic_mode = true;
    }
    
    pub fn clear_panic(&mut self) {
        self.panic_mode = false;
    }
}

pub fn show_stage_view(ui: &mut egui::Ui, stage: &mut StageView, _transport: &mut Transport) {
    let dark_bg = egui::Color32::from_rgb(10, 10, 10);
    egui::Frame::none().fill(dark_bg).show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            
            // BPM & Quantize
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("BPM: {:.1}", stage.bpm_display)).size(48.0).color(egui::Color32::WHITE));
                ui.add_space(20.0);
                egui::ComboBox::from_id_source("quantize")
                    .selected_text(format!("Quantize: {} Beats", stage.quantize_beats))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut stage.quantize_beats, 1, "1 Beat");
                        ui.selectable_value(&mut stage.quantize_beats, 2, "2 Beats");
                        ui.selectable_value(&mut stage.quantize_beats, 4, "4 Beats");
                        ui.selectable_value(&mut stage.quantize_beats, 8, "8 Beats");
                    });
            });

            ui.add_space(40.0);

            // 4x4 Grid
            egui::Grid::new("stage_grid").spacing(egui::vec2(20.0, 20.0)).show(ui, |ui| {
                for row in 0..4 {
                    for col in 0..4 {
                        let idx = row * 4 + col;
                        let slot = &mut stage.pattern_slots[idx];
                        
                        let (text, color, is_armed) = if let Some(s) = slot {
                            (s.pattern_name.clone(), s.color, s.armed)
                        } else {
                            ("Empty".to_string(), egui::Color32::from_gray(30), false)
                        };

                        let button_rect = ui.allocate_space(egui::vec2(120.0, 120.0)).1;
                        let response = ui.interact(button_rect, ui.id().with(idx), egui::Sense::click());
                        
                        let fill_color = if is_armed {
                            // Pulsing glow animation
                            let time = ui.input(|i| i.time);
                            let pulse = (time * 5.0).sin().abs() as f32;
                            ui.ctx().request_repaint();
                            color.linear_multiply(0.5 + 0.5 * pulse)
                        } else {
                            color.linear_multiply(0.3)
                        };

                        ui.painter().rect_filled(button_rect, 10.0, fill_color);
                        ui.painter().rect_stroke(button_rect, 10.0, egui::Stroke::new(2.0f32, if response.hovered() { egui::Color32::WHITE } else { color }));
                        ui.painter().text(button_rect.center(), egui::Align2::CENTER_CENTER, text, egui::FontId::proportional(24.0), egui::Color32::WHITE);

                        if response.clicked() {
                            if let Some(s) = slot {
                                s.armed = !s.armed;
                            }
                        }
                    }
                    ui.end_row();
                }
            });

            ui.add_space(60.0);

            // Panic Button
            let panic_rect = ui.allocate_space(egui::vec2(400.0, 80.0)).1;
            let panic_response = ui.interact(panic_rect, ui.id().with("panic"), egui::Sense::click());
            let panic_color = if panic_response.hovered() { egui::Color32::from_rgb(255, 100, 100) } else { egui::Color32::from_rgb(200, 0, 0) };
            
            ui.painter().rect_filled(panic_rect, 15.0, panic_color);
            ui.painter().text(panic_rect.center(), egui::Align2::CENTER_CENTER, "PANIC (ESC)", egui::FontId::proportional(32.0), egui::Color32::WHITE);
            
            if panic_response.clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                stage.trigger_panic();
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_view_renders_without_panic() {
        let mut stage = StageView::new();
        stage.pattern_slots[0] = Some(PatternSlot {
            track_id: 1,
            pattern_name: "Kick".to_string(),
            color: egui::Color32::RED,
            armed: false,
        });
        stage.pattern_slots[1] = Some(PatternSlot {
            track_id: 1,
            pattern_name: "Snare".to_string(),
            color: egui::Color32::BLUE,
            armed: true,
        });
        
        let mut transport = Transport::new(44100, 120.0);
        
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_stage_view(ui, &mut stage, &mut transport);
            });
        });
        
        assert_eq!(stage.pattern_slots.iter().filter(|s| s.is_some()).count(), 2);
    }
}
