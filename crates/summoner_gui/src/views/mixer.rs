use eframe::egui;
use summoner_project::schema::ProjectConfig;

pub fn show_mixer(ui: &mut egui::Ui, project: &mut ProjectConfig, selected_track_id: &mut Option<u64>) {
    ui.heading("Console Mixer");
    ui.separator();

    egui::ScrollArea::horizontal().show(ui, |ui| {
        ui.horizontal(|ui| {
            for track in &mut project.tracks {
                let is_selected = selected_track_id.map_or(false, |id| id == track.id);

                egui::Frame::window(ui.style())
                    .fill(if is_selected { egui::Color32::from_rgb(35, 45, 60) } else { egui::Color32::from_rgb(25, 25, 25) })
                    .show(ui, |ui| {
                        ui.set_width(120.0);
                        ui.vertical_centered(|ui| {
                            // Track header button
                            let head_btn = ui.selectable_label(is_selected, egui::RichText::new(&track.name).strong().size(14.0));
                            if head_btn.clicked() {
                                *selected_track_id = Some(track.id);
                            }

                            ui.separator();

                            // Mute & Solo
                            ui.horizontal(|ui| {
                                let mut mute = track.muted;
                                if ui.toggle_value(&mut mute, "M").changed() {
                                    track.muted = mute;
                                }
                                let _ = ui.button("S"); // Solo toggle placeholder
                            });

                            ui.add_space(8.0);

                            // Gain Fader
                            ui.label("Gain");
                            let gain_slider = egui::Slider::new(&mut track.gain, 0.0..=2.0)
                                .orientation(egui::SliderOrientation::Vertical)
                                .text("");
                            ui.add(gain_slider);

                            ui.label(format!("{:.1} dB", 20.0 * track.gain.max(0.0001).log10()));

                            ui.add_space(8.0);

                            // Pan Knob / Slider
                            ui.label("Pan");
                            ui.add(egui::Slider::new(&mut track.pan, -1.0..=1.0).text(""));

                            ui.separator();

                            // Devices count badge
                            ui.label(format!("{} Devices", track.nodes.len()));
                        });
                    });

                ui.add_space(4.0);
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_project::create_default_project;

    #[test]
    fn test_mixer_view_renders_without_panic() {
        let mut project = create_default_project("Test Session");
        let mut selected_track_id = None;

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_mixer(ui, &mut project, &mut selected_track_id);
            });
        });
    }
}
