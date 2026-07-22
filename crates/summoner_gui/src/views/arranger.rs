use eframe::egui;
use summoner_project::schema::ProjectConfig;

pub fn show_arranger(ui: &mut egui::Ui, project: &mut ProjectConfig) {
    ui.heading("Arranger Timeline");
    
    egui::ScrollArea::both().show(ui, |ui| {
        for track in &mut project.tracks {
            ui.horizontal(|ui| {
                ui.label(format!("Track {}: {}", track.id, track.name));
                ui.separator();
                // A very basic placeholder for clips on the timeline
                let (_id, rect) = ui.allocate_space(egui::vec2(500.0, 40.0));
                
                if ui.is_rect_visible(rect) {
                    ui.painter().rect_stroke(
                        rect, 
                        2.0, 
                        egui::Stroke::new(1.0, egui::Color32::DARK_GRAY)
                    );
                    
                    if let Some(seq) = &track.sequence {
                        // Just an indicator that it has a sequence
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            format!("Sequence ({} steps)", seq.steps.len()),
                            egui::FontId::proportional(12.0),
                            egui::Color32::WHITE,
                        );
                    } else {
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "No clips",
                            egui::FontId::proportional(12.0),
                            egui::Color32::GRAY,
                        );
                    }
                }
            });
            ui.add_space(4.0);
        }
    });
}
