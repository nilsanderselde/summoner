use eframe::egui;
use std::sync::Arc;
use summoner_project::schema::TrackConfig;
use summoner_core::param_bus::ParamBus;

pub fn show_macro_rack(
    ui: &mut egui::Ui,
    track: &TrackConfig,
    param_bus: &Arc<ParamBus>,
    on_open_graph: &mut dyn FnMut(),
) {
    ui.heading(format!("Macro Rack: {}", track.name));

    ui.horizontal(|ui| {
        if ui.button("Open Node Graph").clicked() {
            on_open_graph();
        }
    });

    ui.separator();

    // Mock oscilloscope strip
    let rect = ui.allocate_space(egui::vec2(ui.available_width(), 50.0)).1;
    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, egui::Color32::from_gray(10));
    
    // Draw some mock waveform data
    let mut points = Vec::new();
    let num_samples = 512;
    for i in 0..num_samples {
        let x = rect.left() + (i as f32 / num_samples as f32) * rect.width();
        let y = rect.center().y + (i as f32 * 0.1).sin() * 20.0;
        points.push(egui::pos2(x, y));
    }
    painter.add(egui::Shape::line(points, egui::Stroke::new(1.0f32, egui::Color32::GREEN)));

    ui.separator();

    // Render device blocks based on track nodes. 
    // For this implementation we mock the UI for AetherSynth and SamplerDevice by looking at the node kind.
    egui::ScrollArea::horizontal().show(ui, |ui| {
        ui.horizontal(|ui| {
            for node in &track.nodes {
                egui::Frame::window(ui.style()).show(ui, |ui| {
                    ui.set_width(200.0);
                    ui.heading(&node.kind);
                    ui.separator();
                    
                    if node.kind == "AetherSynth" {
                        let mut mix = param_bus.get(summoner_core::param_bus::ParamId(track.id as u32 * 10 + 1)).unwrap_or(0.5);
                        if ui.add(egui::Slider::new(&mut mix, 0.0..=1.0).text("Osc Mix")).changed() {
                            param_bus.set(summoner_core::param_bus::ParamId(track.id as u32 * 10 + 1), mix);
                        }

                        let mut cutoff = param_bus.get(summoner_core::param_bus::ParamId(track.id as u32 * 10 + 2)).unwrap_or(0.6);
                        if ui.add(egui::Slider::new(&mut cutoff, 0.0..=1.0).text("Cutoff")).changed() {
                            param_bus.set(summoner_core::param_bus::ParamId(track.id as u32 * 10 + 2), cutoff);
                        }

                        let mut res = param_bus.get(summoner_core::param_bus::ParamId(track.id as u32 * 10 + 3)).unwrap_or(0.25);
                        if ui.add(egui::Slider::new(&mut res, 0.0..=1.0).text("Resonance")).changed() {
                            param_bus.set(summoner_core::param_bus::ParamId(track.id as u32 * 10 + 3), res);
                        }
                    } else if node.kind == "SamplerDevice" {
                        ui.label("Preset:");
                        let mut selected = 0;
                        egui::ComboBox::from_id_source(format!("{}_preset", track.id))
                            .selected_text("mock_piano.preset.toml")
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut selected, 0, "mock_piano.preset.toml");
                                ui.selectable_value(&mut selected, 1, "mock_drums.preset.toml");
                            });
                        
                        let mut cutoff = param_bus.get(summoner_core::param_bus::ParamId(track.id as u32 * 10 + 2)).unwrap_or(1.0);
                        if ui.add(egui::Slider::new(&mut cutoff, 0.0..=1.0).text("Cutoff")).changed() {
                            param_bus.set(summoner_core::param_bus::ParamId(track.id as u32 * 10 + 2), cutoff);
                        }
                    } else {
                        // Generic generic params fallback
                        for (i, (key, default_val)) in node.params.iter().enumerate() {
                            let mut val = param_bus.get(summoner_core::param_bus::ParamId(track.id as u32 * 100 + i as u32)).unwrap_or(*default_val);
                            if ui.add(egui::Slider::new(&mut val, 0.0..=1.0).text(key)).changed() {
                                param_bus.set(summoner_core::param_bus::ParamId(track.id as u32 * 100 + i as u32), val);
                            }
                        }
                    }
                });
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_rack_renders_without_panic() {
        let track = TrackConfig {
            id: 1,
            name: "Test Track".to_string(),
            channels: 2,
            gain: 1.0,
            pan: 0.0,
            muted: false,
            soloed: false,
            nodes: vec![
                summoner_project::schema::NodeConfig {
                    kind: "AetherSynth".to_string(),
                    params: std::collections::HashMap::new(),
                }
            ],
            sequence: None,
            connections: Vec::new(),
            tuning_edo: None,
            tuning_root_hz: None,
            tuning_scl_path: None,
        };
        let param_bus = Arc::new(ParamBus::new());
        let mut on_open = || {};

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_macro_rack(ui, &track, &param_bus, &mut on_open);
            });
        });
    }
}
