use eframe::egui;
use summoner_project::schema::ProjectConfig;

pub fn show_arranger(ui: &mut egui::Ui, project: &mut ProjectConfig) {
    ui.horizontal(|ui| {
        ui.heading("Arranger Timeline");
        ui.separator();
        ui.label(format!("Session: {}", project.name));
    });

    ui.separator();

    let pixels_per_beat = 40.0;
    let total_beats = 32.0;

    egui::ScrollArea::both().show(ui, |ui| {
        // Timeline Header Ruler
        ui.horizontal(|ui| {
            ui.allocate_space(egui::vec2(180.0, 24.0)); // Space above track headers
            let (ruler_resp, ruler_painter) = ui.allocate_painter(egui::vec2(total_beats * pixels_per_beat, 24.0), egui::Sense::hover());
            let ruler_rect = ruler_resp.rect;
            
            ruler_painter.rect_filled(ruler_rect, 0.0, egui::Color32::from_rgb(30, 30, 35));
            for beat in 0..=(total_beats as usize) {
                let x = ruler_rect.left() + beat as f32 * pixels_per_beat;
                ruler_painter.line_segment(
                    [egui::pos2(x, ruler_rect.top()), egui::pos2(x, ruler_rect.bottom())],
                    egui::Stroke::new(1.0, egui::Color32::from_gray(80)),
                );
                if beat % 4 == 0 {
                    ruler_painter.text(
                        egui::pos2(x + 4.0, ruler_rect.top() + 4.0),
                        egui::Align2::LEFT_TOP,
                        format!("Bar {}", (beat / 4) + 1),
                        egui::FontId::proportional(10.0),
                        egui::Color32::WHITE,
                    );
                }
            }
        });

        ui.separator();

        // Track Lanes
        for track in &mut project.tracks {
            ui.horizontal(|ui| {
                // Track Control Header
                ui.allocate_ui(egui::vec2(180.0, 50.0), |ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(&track.name).strong());
                            let mut mute = track.muted;
                            if ui.toggle_value(&mut mute, "M").changed() {
                                track.muted = mute;
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Vol:");
                            ui.add(egui::Slider::new(&mut track.gain, 0.0..=1.5).text(""));
                        });
                    });
                });

                ui.separator();

                // Track Timeline Area
                let (lane_resp, painter) = ui.allocate_painter(egui::vec2(total_beats * pixels_per_beat, 50.0), egui::Sense::click_and_drag());
                let lane_rect = lane_resp.rect;

                // Draw beat grid lines
                painter.rect_filled(lane_rect, 2.0, egui::Color32::from_rgb(20, 20, 20));
                for beat in 0..=(total_beats as usize) {
                    let x = lane_rect.left() + beat as f32 * pixels_per_beat;
                    let stroke_color = if beat % 4 == 0 { egui::Color32::from_gray(60) } else { egui::Color32::from_gray(35) };
                    painter.line_segment(
                        [egui::pos2(x, lane_rect.top()), egui::pos2(x, lane_rect.bottom())],
                        egui::Stroke::new(1.0, stroke_color),
                    );
                }

                // Render Clips
                if let Some(seq) = &track.sequence {
                    let clip_width = (seq.steps.len() as f64 * seq.step_division * pixels_per_beat as f64) as f32;
                    let clip_rect = egui::Rect::from_min_size(
                        egui::pos2(lane_rect.left() + 4.0, lane_rect.top() + 4.0),
                        egui::vec2(clip_width.max(80.0), lane_rect.height() - 8.0),
                    );

                    painter.rect_filled(clip_rect, 4.0, egui::Color32::from_rgb(40, 80, 140));
                    painter.rect_stroke(clip_rect, 4.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 160, 240)));
                    painter.text(
                        clip_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        format!("Pattern ({} steps)", seq.steps.len()),
                        egui::FontId::proportional(12.0),
                        egui::Color32::WHITE,
                    );
                }
            });
            ui.add_space(4.0);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_project::create_default_project;

    #[test]
    fn test_arranger_renders_without_panic() {
        let mut project = create_default_project("Arranger Test");

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_arranger(ui, &mut project);
            });
        });
    }
}
