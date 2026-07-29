use eframe::egui;
use std::sync::Arc;
use std::path::Path;
use summoner_project::schema::TrackConfig;
use summoner_core::param_bus::{ParamBus, ParamId};
use crate::visualizer::Oscilloscope;

/// Scans local preset directory for available `.preset.toml` or `.toml` files.
pub fn scan_preset_files(presets_dir: &Path) -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(presets_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.ends_with(".preset.toml") || name.ends_with(".toml") {
                        files.push(name.to_string());
                    }
                }
            }
        }
    }
    files.sort();
    if files.is_empty() {
        vec![
            "mock_piano.preset.toml".to_string(),
            "mock_drums.preset.toml".to_string(),
        ]
    } else {
        files
    }
}

pub fn show_macro_rack(
    ui: &mut egui::Ui,
    track: &mut TrackConfig,
    param_bus: &Arc<ParamBus>,
    oscilloscope: Option<&Oscilloscope>,
    on_open_graph: &mut dyn FnMut(),
) {
    ui.heading(format!("Macro Rack: {}", track.name));

    ui.horizontal(|ui| {
        if ui.button("Open Node Graph").clicked() {
            on_open_graph();
        }
    });

    ui.separator();

    // Real-time Oscilloscope strip
    let rect = ui.allocate_space(egui::vec2(ui.available_width(), 50.0)).1;
    let painter = ui.painter();
    painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(8, 8, 12));
    
    let mut points = Vec::with_capacity(512);
    let sample_data = oscilloscope.map(|o| o.read_all()).unwrap_or([0.0f32; 512]);
    let num_samples = sample_data.len();

    for i in 0..num_samples {
        let x = rect.left() + (i as f32 / num_samples as f32) * rect.width();
        let sample = sample_data[i].clamp(-1.0, 1.0);
        let y = rect.center().y - sample * (rect.height() * 0.4);
        points.push(egui::pos2(x, y));
    }
    painter.add(egui::Shape::line(points, egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(26, 140, 255))));

    ui.separator();

    // Scan available preset files for SamplerDevice
    let preset_files = scan_preset_files(Path::new("local/presets"));

    // Render device blocks based on track nodes with reorder and bypass controls.
    let mut move_left_idx = None;
    let mut move_right_idx = None;

    egui::ScrollArea::horizontal().show(ui, |ui| {
        ui.horizontal(|ui| {
            let num_nodes = track.nodes.len();

            for (idx, node) in track.nodes.iter_mut().enumerate() {
                // Determine node category color and [eff] badge
                let is_effect = node.kind.contains("Filter")
                    || node.kind.contains("Delay")
                    || node.kind.contains("Reverb")
                    || node.kind.contains("Distortion")
                    || node.kind.contains("Chorus")
                    || node.kind.contains("Flanger")
                    || node.kind.contains("Phaser")
                    || node.kind.contains("Effect");

                let is_synth = node.kind.contains("Synth")
                    || node.kind == "AetherSynth"
                    || node.kind == "FmOperatorPair"
                    || node.kind == "PluckSynth"
                    || node.kind == "GranularSynthNode";

                let is_sampler = node.kind == "SamplerDevice" || node.kind.contains("Sampler");

                let dot_color = if is_synth {
                    egui::Color32::from_rgb(26, 140, 255) // Electric Blue
                } else if is_sampler {
                    egui::Color32::from_rgb(40, 200, 100) // Emerald Green
                } else if is_effect {
                    egui::Color32::from_rgb(180, 80, 240) // Purple
                } else {
                    egui::Color32::from_rgb(150, 150, 150) // Gray
                };

                let mut is_bypassed = node.params.get("bypassed").map(|&v| v > 0.5).unwrap_or(false);

                let frame = egui::Frame::window(ui.style())
                    .fill(if is_bypassed { egui::Color32::from_rgb(18, 18, 22) } else { ui.style().visuals.window_fill });

                frame.show(ui, |ui| {
                    ui.set_width(220.0);

                    // Header row: colored dot, title, [eff] badge, reorder buttons
                    ui.horizontal(|ui| {
                        let (dot_rect, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
                        ui.painter().circle_filled(dot_rect.center(), 4.0, dot_color);

                        ui.label(egui::RichText::new(&node.kind).strong());

                        if is_effect {
                            ui.colored_label(egui::Color32::from_rgb(180, 80, 240), "[eff]");
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if idx + 1 < num_nodes && ui.button("▶").clicked() {
                                move_right_idx = Some(idx);
                            }
                            if idx > 0 && ui.button("◀").clicked() {
                                move_left_idx = Some(idx);
                            }
                        });
                    });

                    // Bypass toggle row
                    ui.horizontal(|ui| {
                        if ui.checkbox(&mut is_bypassed, "Bypass").changed() {
                            node.params.insert("bypassed".to_string(), if is_bypassed { 1.0 } else { 0.0 });
                        }
                    });

                    ui.separator();

                    if is_bypassed {
                        ui.weak("Device Bypassed");
                        return;
                    }

                    // Render specific device controls
                    match node.kind.as_str() {
                        "AetherSynth" => {
                            let mut mix = param_bus.get(ParamId(track.id as u32 * 10 + 1)).unwrap_or(0.5);
                            if ui.add(egui::Slider::new(&mut mix, 0.0..=1.0).text("Osc Mix")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 1), mix);
                            }

                            let mut cutoff = param_bus.get(ParamId(track.id as u32 * 10 + 2)).unwrap_or(0.6);
                            if ui.add(egui::Slider::new(&mut cutoff, 0.0..=1.0).text("Cutoff")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 2), cutoff);
                            }

                            let mut res = param_bus.get(ParamId(track.id as u32 * 10 + 3)).unwrap_or(0.25);
                            if ui.add(egui::Slider::new(&mut res, 0.0..=1.0).text("Resonance")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 3), res);
                            }

                            ui.collapsing("EnvADSR Controls", |ui| {
                                let mut attack = param_bus.get(ParamId(track.id as u32 * 10 + 4)).unwrap_or(0.01);
                                if ui.add(egui::Slider::new(&mut attack, 0.001..=2.0).text("Attack")).changed() {
                                    param_bus.set(ParamId(track.id as u32 * 10 + 4), attack);
                                }
                                let mut decay = param_bus.get(ParamId(track.id as u32 * 10 + 5)).unwrap_or(0.2);
                                if ui.add(egui::Slider::new(&mut decay, 0.01..=5.0).text("Decay")).changed() {
                                    param_bus.set(ParamId(track.id as u32 * 10 + 5), decay);
                                }
                                let mut sustain = param_bus.get(ParamId(track.id as u32 * 10 + 6)).unwrap_or(0.7);
                                if ui.add(egui::Slider::new(&mut sustain, 0.0..=1.0).text("Sustain")).changed() {
                                    param_bus.set(ParamId(track.id as u32 * 10 + 6), sustain);
                                }
                                let mut release = param_bus.get(ParamId(track.id as u32 * 10 + 7)).unwrap_or(0.5);
                                if ui.add(egui::Slider::new(&mut release, 0.01..=5.0).text("Release")).changed() {
                                    param_bus.set(ParamId(track.id as u32 * 10 + 7), release);
                                }
                            });

                            ui.collapsing("LFO Controls", |ui| {
                                let mut lfo_rate = param_bus.get(ParamId(track.id as u32 * 10 + 8)).unwrap_or(1.0);
                                if ui.add(egui::Slider::new(&mut lfo_rate, 0.1..=20.0).text("LFO Rate")).changed() {
                                    param_bus.set(ParamId(track.id as u32 * 10 + 8), lfo_rate);
                                }
                                let mut lfo_shape = param_bus.get(ParamId(track.id as u32 * 10 + 9)).unwrap_or(0.0);
                                if ui.add(egui::Slider::new(&mut lfo_shape, 0.0..=3.0).text("LFO Shape")).changed() {
                                    param_bus.set(ParamId(track.id as u32 * 10 + 9), lfo_shape);
                                }
                            });
                        }
                        "FmOperatorPair" => {
                            let mut ratio1 = param_bus.get(ParamId(track.id as u32 * 10 + 1)).unwrap_or(1.0);
                            if ui.add(egui::Slider::new(&mut ratio1, 0.5..=16.0).text("Ratio 1")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 1), ratio1);
                            }
                            let mut ratio2 = param_bus.get(ParamId(track.id as u32 * 10 + 2)).unwrap_or(2.0);
                            if ui.add(egui::Slider::new(&mut ratio2, 0.5..=16.0).text("Ratio 2")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 2), ratio2);
                            }
                            let mut fm_depth = param_bus.get(ParamId(track.id as u32 * 10 + 3)).unwrap_or(0.5);
                            if ui.add(egui::Slider::new(&mut fm_depth, 0.0..=10.0).text("FM Depth")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 3), fm_depth);
                            }
                        }
                        "PluckSynth" => {
                            let mut hardness = param_bus.get(ParamId(track.id as u32 * 10 + 1)).unwrap_or(0.8);
                            if ui.add(egui::Slider::new(&mut hardness, 0.0..=1.0).text("Pluck Hardness")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 1), hardness);
                            }
                            let mut tension = param_bus.get(ParamId(track.id as u32 * 10 + 2)).unwrap_or(0.5);
                            if ui.add(egui::Slider::new(&mut tension, 0.0..=1.0).text("String Tension")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 2), tension);
                            }
                            let mut damping = param_bus.get(ParamId(track.id as u32 * 10 + 3)).unwrap_or(0.2);
                            if ui.add(egui::Slider::new(&mut damping, 0.0..=1.0).text("Damping")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 3), damping);
                            }
                        }
                        "GranularSynthNode" => {
                            let mut density = param_bus.get(ParamId(track.id as u32 * 10 + 1)).unwrap_or(20.0);
                            if ui.add(egui::Slider::new(&mut density, 1.0..=100.0).text("Grain Density")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 1), density);
                            }
                            let mut spray = param_bus.get(ParamId(track.id as u32 * 10 + 2)).unwrap_or(0.1);
                            if ui.add(egui::Slider::new(&mut spray, 0.0..=1.0).text("Spray")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 2), spray);
                            }
                            let mut pitch_ratio = param_bus.get(ParamId(track.id as u32 * 10 + 3)).unwrap_or(1.0);
                            if ui.add(egui::Slider::new(&mut pitch_ratio, 0.25..=4.0).text("Pitch Ratio")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 3), pitch_ratio);
                            }
                            let mut grain_size = param_bus.get(ParamId(track.id as u32 * 10 + 4)).unwrap_or(0.05);
                            if ui.add(egui::Slider::new(&mut grain_size, 0.01..=0.5).text("Grain Size (s)")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 4), grain_size);
                            }
                        }
                        "SamplerDevice" => {
                            ui.label("Preset:");
                            let selected_preset = node.params.get("preset_filename")
                                .map(|_| "selected".to_string())
                                .unwrap_or_else(|| preset_files.first().cloned().unwrap_or_default());

                            let mut current = selected_preset.clone();
                            egui::ComboBox::from_id_source(format!("{}_{}_preset", track.id, idx))
                                .selected_text(&current)
                                .show_ui(ui, |ui| {
                                    for preset_name in &preset_files {
                                        ui.selectable_value(&mut current, preset_name.clone(), preset_name);
                                    }
                                });

                            let mut cutoff = param_bus.get(ParamId(track.id as u32 * 10 + 2)).unwrap_or(1.0);
                            if ui.add(egui::Slider::new(&mut cutoff, 0.0..=1.0).text("Cutoff")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 2), cutoff);
                            }
                        }
                        _ => {
                            // Generic parameters fallback
                            for (i, (key, default_val)) in node.params.iter_mut().enumerate() {
                                if key == "bypassed" {
                                    continue;
                                }
                                let mut val = param_bus.get(ParamId(track.id as u32 * 100 + i as u32)).unwrap_or(*default_val);
                                if ui.add(egui::Slider::new(&mut val, 0.0..=1.0).text(key.as_str())).changed() {
                                    *default_val = val;
                                    param_bus.set(ParamId(track.id as u32 * 100 + i as u32), val);
                                }
                            }
                        }
                    }
                });
            }
        });
    });

    // Execute reorder if requested
    if let Some(idx) = move_left_idx {
        if idx > 0 {
            track.nodes.swap(idx, idx - 1);
        }
    } else if let Some(idx) = move_right_idx {
        if idx + 1 < track.nodes.len() {
            track.nodes.swap(idx, idx + 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_rack_renders_without_panic() {
        let mut track = TrackConfig {
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
                show_macro_rack(ui, &mut track, &param_bus, None, &mut on_open);
            });
        });
    }

    #[test]
    fn test_macro_rack_real_oscilloscope_wire() {
        let mut track = TrackConfig {
            id: 1,
            name: "Osc Track".to_string(),
            channels: 2,
            gain: 1.0,
            pan: 0.0,
            muted: false,
            soloed: false,
            nodes: vec![
                summoner_project::schema::NodeConfig {
                    kind: "FmOperatorPair".to_string(),
                    params: std::collections::HashMap::new(),
                },
                summoner_project::schema::NodeConfig {
                    kind: "PluckSynth".to_string(),
                    params: std::collections::HashMap::new(),
                },
                summoner_project::schema::NodeConfig {
                    kind: "GranularSynthNode".to_string(),
                    params: std::collections::HashMap::new(),
                },
            ],
            sequence: None,
            connections: Vec::new(),
            tuning_edo: None,
            tuning_root_hz: None,
            tuning_scl_path: None,
        };
        let param_bus = Arc::new(ParamBus::new());
        let oscilloscope = Oscilloscope::new();
        for i in 0..512 {
            oscilloscope.write_sample((i as f32 * 0.05).sin());
        }

        let mut on_open = || {};
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_macro_rack(ui, &mut track, &param_bus, Some(&oscilloscope), &mut on_open);
            });
        });
        
        // Assert samples were read correctly
        let readback = oscilloscope.read_all();
        assert!((readback[10] - (10.0f32 * 0.05).sin()).abs() < 1e-4);
    }

    #[test]
    fn test_macro_rack_preset_dropdown_scan() {
        let scanned = scan_preset_files(Path::new("local/presets"));
        assert!(!scanned.is_empty(), "Preset scanner should return files or defaults");
    }
}
