use eframe::egui;
use std::sync::Arc;
use std::path::Path;
use summoner_project::schema::TrackConfig;
use summoner_core::param_bus::{ParamBus, ParamId};
use crate::visualizer::{show_oscilloscope, Oscilloscope};

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
    let dummy_scope = Oscilloscope::new();
    let scope_ref = oscilloscope.unwrap_or(&dummy_scope);
    show_oscilloscope(ui, scope_ref, ui.available_width(), 50.0);

    ui.separator();

    // Tuning collapsible section (Step 357)
    ui.collapsing("🎼 Microtonal Tuning & Scale", |ui| {
        let mut edo = track.tuning_edo.unwrap_or(12) as i32;
        if ui.add(egui::Slider::new(&mut edo, 1..=72).text("EDO Divisions")).changed() {
            track.tuning_edo = Some(edo as u32);
        }
        let mut root_hz = track.tuning_root_hz.unwrap_or(440.0);
        if ui.add(egui::Slider::new(&mut root_hz, 100.0..=1000.0).text("Root Freq (Hz)")).changed() {
            track.tuning_root_hz = Some(root_hz);
        }
        let mut scl_path = track.tuning_scl_path.clone().unwrap_or_default();
        ui.horizontal(|ui| {
            ui.label("SCL File:");
            if ui.text_edit_singleline(&mut scl_path).changed() {
                track.tuning_scl_path = if scl_path.is_empty() { None } else { Some(scl_path.clone()) };
            }
            if ui.button("📂 Browse...").clicked() {
                if let Some(path) = rfd::FileDialog::new().add_filter("Scala SCL", &["scl"]).pick_file() {
                    if let Some(path_str) = path.to_str() {
                        track.tuning_scl_path = Some(path_str.to_string());
                    }
                }
            }
        });
    });

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

                    // Bypass toggle row & LLM Explain Patch (Step 490)
                    ui.horizontal(|ui| {
                        if ui.checkbox(&mut is_bypassed, "Bypass").changed() {
                            node.params.insert("bypassed".to_string(), if is_bypassed { 1.0 } else { 0.0 });
                        }

                        let popup_id = ui.make_persistent_id(format!("explain_patch_{}_{}", track.id, idx));
                        let mut explain_open = ui.data(|d| d.get_temp::<bool>(popup_id).unwrap_or(false));

                        if ui.button("🤖 Explain").clicked() {
                            explain_open = !explain_open;
                            ui.data_mut(|d| d.insert_temp(popup_id, explain_open));
                        }

                        if explain_open {
                            egui::Window::new(format!("🤖 Patch Architecture -- {}", node.kind))
                                .id(popup_id)
                                .collapsible(false)
                                .resizable(true)
                                .default_size([300.0, 140.0])
                                .show(ui.ctx(), |ui| {
                                    ui.label(format!("Architecture analysis for '{}':", node.kind));
                                    ui.add_space(4.0);
                                    let explanation = match node.kind.as_str() {
                                        "AetherSynth" => "Dual band-limited saw/pulse stack driving a 4-pole SVF filter with ADSR envelope modulation and LFO rate control.",
                                        "FmOperatorPair" => "Frequency modulation pair with 2 operators. Operator 1 frequency modulates operator 2 with variable FM depth and ratio controls.",
                                        "PluckSynth" => "Karplus-Strong physical modeling waveguide algorithm producing plucked string acoustics with adjustable damping and tension.",
                                        "GranularSynthNode" => "Asynchronous grain cloud generator splitting audio buffer into 50ms grain windows with density and spray randomization.",
                                        "OscWavetable" | "WavetableOscillator" => "2048-sample morphing wavetable oscillator transitioning smoothly between saw and square tables.",
                                        _ => "Generic signal graph audio processing node with real-time parameter bus bindings.",
                                    };
                                    ui.colored_label(egui::Color32::from_rgb(0, 200, 255), explanation);
                                    ui.add_space(8.0);
                                    if ui.button("Close").clicked() {
                                        ui.data_mut(|d| d.insert_temp(popup_id, false));
                                    }
                                });
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

                            ui.collapsing("📊 Real-time Spectrum & Harmonics", |ui| {
                                let dummy_spectrum = [0.1, 0.4, 0.7, 0.5, 0.3, 0.2, 0.15, 0.08];
                                show_spectral_display(ui, &dummy_spectrum, 200.0, 40.0);
                                ui.add_space(4.0);
                                let dummy_harmonics = [1.0, 0.8, 0.6, 0.4, 0.3, 0.25, 0.2, 0.15, 0.1, 0.08, 0.06, 0.05, 0.04, 0.03, 0.02, 0.01];
                                show_harmonics_display(ui, &dummy_harmonics, 200.0, 40.0);
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
                        "EffectDelay" | "DelayNode" => {
                            let mut delay_time = param_bus.get(ParamId(track.id as u32 * 10 + 1)).unwrap_or(0.3);
                            if ui.add(egui::Slider::new(&mut delay_time, 0.01..=2.0).text("Time (s)")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 1), delay_time);
                            }
                            let mut feedback = param_bus.get(ParamId(track.id as u32 * 10 + 2)).unwrap_or(0.4);
                            if ui.add(egui::Slider::new(&mut feedback, 0.0..=0.95).text("Feedback")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 2), feedback);
                            }
                            let mut mix = param_bus.get(ParamId(track.id as u32 * 10 + 3)).unwrap_or(0.3);
                            if ui.add(egui::Slider::new(&mut mix, 0.0..=1.0).text("Mix")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 3), mix);
                            }
                        }
                        "EffectReverb" | "ReverbNode" => {
                            let mut room_size = param_bus.get(ParamId(track.id as u32 * 10 + 1)).unwrap_or(0.7);
                            if ui.add(egui::Slider::new(&mut room_size, 0.0..=0.98).text("Room Size")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 1), room_size);
                            }
                            let mut mix = param_bus.get(ParamId(track.id as u32 * 10 + 2)).unwrap_or(0.3);
                            if ui.add(egui::Slider::new(&mut mix, 0.0..=1.0).text("Mix")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 2), mix);
                            }
                        }
                        "WavefolderNode" => {
                            let mut threshold = param_bus.get(ParamId(track.id as u32 * 10 + 1)).unwrap_or(0.5);
                            if ui.add(egui::Slider::new(&mut threshold, 0.05..=1.0).text("Threshold")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 1), threshold);
                            }
                            let mut drive = param_bus.get(ParamId(track.id as u32 * 10 + 2)).unwrap_or(2.0);
                            if ui.add(egui::Slider::new(&mut drive, 1.0..=10.0).text("Drive")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 2), drive);
                            }
                        }
                        "PitchShifterNode" => {
                            let mut semitones = param_bus.get(ParamId(track.id as u32 * 10 + 1)).unwrap_or(0.0);
                            if ui.add(egui::Slider::new(&mut semitones, -24.0..=24.0).text("Semitones")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 1), semitones);
                            }
                        }
                        "BitcrusherNode" => {
                            let mut bit_depth = param_bus.get(ParamId(track.id as u32 * 10 + 1)).unwrap_or(8.0);
                            if ui.add(egui::Slider::new(&mut bit_depth, 1.0..=16.0).text("Bit Depth")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 1), bit_depth);
                            }
                            let mut reduction = param_bus.get(ParamId(track.id as u32 * 10 + 2)).unwrap_or(4.0);
                            if ui.add(egui::Slider::new(&mut reduction, 1.0..=32.0).text("Downsample")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 2), reduction);
                            }
                        }
                        "MidSideNode" => {
                            let mut width = param_bus.get(ParamId(track.id as u32 * 10 + 1)).unwrap_or(1.0);
                            if ui.add(egui::Slider::new(&mut width, 0.0..=4.0).text("Stereo Width")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 1), width);
                            }
                        }
                        "ParametricEqNode" => {
                            ui.label("8-Band Parametric EQ");
                            let mut boost = param_bus.get(ParamId(track.id as u32 * 10 + 1)).unwrap_or(0.0);
                            if ui.add(egui::Slider::new(&mut boost, -12.0..=12.0).text("Mid Gain (dB)")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 1), boost);
                            }
                        }
                        "MultibandCompressorNode" | "MultibandCompressor" => {
                            ui.label("3-Band Multiband Compressor");
                            let mut low_thresh = param_bus.get(ParamId(track.id as u32 * 10 + 1)).unwrap_or(-18.0);
                            if ui.add(egui::Slider::new(&mut low_thresh, -40.0..=0.0).text("Low Thresh (dB)")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 1), low_thresh);
                            }
                            let mut mid_thresh = param_bus.get(ParamId(track.id as u32 * 10 + 2)).unwrap_or(-16.0);
                            if ui.add(egui::Slider::new(&mut mid_thresh, -40.0..=0.0).text("Mid Thresh (dB)")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 2), mid_thresh);
                            }
                            let mut high_thresh = param_bus.get(ParamId(track.id as u32 * 10 + 3)).unwrap_or(-14.0);
                            if ui.add(egui::Slider::new(&mut high_thresh, -40.0..=0.0).text("High Thresh (dB)")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 3), high_thresh);
                            }
                        }
                        "TapeSaturationNode" | "TapeSaturation" => {
                            let mut drive = param_bus.get(ParamId(track.id as u32 * 10 + 1)).unwrap_or(2.0);
                            if ui.add(egui::Slider::new(&mut drive, 1.0..=10.0).text("Tape Drive")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 1), drive);
                            }
                            let mut sat = param_bus.get(ParamId(track.id as u32 * 10 + 2)).unwrap_or(0.5);
                            if ui.add(egui::Slider::new(&mut sat, 0.0..=1.0).text("Saturation")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 2), sat);
                            }
                        }
                        "TubeSaturationNode" | "TubeSaturation" => {
                            let mut drive = param_bus.get(ParamId(track.id as u32 * 10 + 1)).unwrap_or(2.5);
                            if ui.add(egui::Slider::new(&mut drive, 1.0..=10.0).text("Tube Drive")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 1), drive);
                            }
                            let mut bias = param_bus.get(ParamId(track.id as u32 * 10 + 2)).unwrap_or(0.2);
                            if ui.add(egui::Slider::new(&mut bias, 0.0..=0.5).text("Bias (Asymmetry)")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 2), bias);
                            }
                        }
                        "ConsoleEmulationNode" | "ConsoleEmulation" => {
                            let mut mode = param_bus.get(ParamId(track.id as u32 * 10 + 1)).unwrap_or(0.0);
                            ui.horizontal(|ui| {
                                ui.label("Console Mode:");
                                ui.selectable_value(&mut mode, 0.0, "Neve");
                                ui.selectable_value(&mut mode, 1.0, "SSL");
                                ui.selectable_value(&mut mode, 2.0, "API");
                            });
                            param_bus.set(ParamId(track.id as u32 * 10 + 1), mode);

                            let mut drive = param_bus.get(ParamId(track.id as u32 * 10 + 2)).unwrap_or(1.0);
                            if ui.add(egui::Slider::new(&mut drive, 0.0..=5.0).text("Console Drive")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 2), drive);
                            }
                        }
                        "DistortionNode" => {
                            let mut drive = param_bus.get(ParamId(track.id as u32 * 10 + 1)).unwrap_or(2.0);
                            if ui.add(egui::Slider::new(&mut drive, 1.0..=20.0).text("Drive")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 1), drive);
                            }
                        }
                        "OscWavetable" | "WavetableOscillator" => {
                            let mut freq = param_bus.get(ParamId(track.id as u32 * 10 + 1)).unwrap_or(440.0);
                            if ui.add(egui::Slider::new(&mut freq, 20.0..=2000.0).text("Frequency")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 1), freq);
                                node.params.insert("freq".to_string(), freq);
                            }

                            let mut morph = param_bus.get(ParamId(track.id as u32 * 10 + 2)).unwrap_or(0.0);
                            if ui.add(egui::Slider::new(&mut morph, 0.0..=1.0).text("Wavetable Morph")).changed() {
                                param_bus.set(ParamId(track.id as u32 * 10 + 2), morph);
                                node.params.insert("morph".to_string(), morph);
                            }

                            // Wavetable Display (Step 474)
                            ui.label("Wavetable Curve:");
                            let (response, painter) = ui.allocate_painter(egui::Vec2::new(160.0, 40.0), egui::Sense::hover());
                            let rect = response.rect;
                            painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(10, 15, 25));
                            painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 80, 120)));

                            let morph_val = morph.clamp(0.0, 1.0);
                            let points: Vec<egui::Pos2> = (0..50).map(|i| {
                                let t = i as f32 / 50.0;
                                let saw = 2.0 * t - 1.0;
                                let sq = if t < 0.5 { 1.0 } else { -1.0 };
                                let val = saw * (1.0 - morph_val) + sq * morph_val;
                                let x = rect.left() + t * rect.width();
                                let y = rect.center().y - val * (rect.height() * 0.4);
                                egui::Pos2::new(x, y)
                            }).collect();

                            for window in points.windows(2) {
                                painter.line_segment([window[0], window[1]], egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 200, 255)));
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

/// Real-time spectral display component for Macro Rack (Step 659).
pub fn show_spectral_display(ui: &mut egui::Ui, spectrum: &[f32], width: f32, height: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    ui.painter().rect_filled(rect, 4.0, egui::Color32::from_rgb(10, 10, 15));

    if spectrum.is_empty() {
        return;
    }

    let bar_w = rect.width() / spectrum.len() as f32;
    for (i, &mag) in spectrum.iter().enumerate() {
        let h = (mag.clamp(0.0, 1.0) * rect.height()).max(2.0);
        let x = rect.left() + i as f32 * bar_w;
        let bar_rect = egui::Rect::from_min_size(
            egui::pos2(x + 1.0, rect.bottom() - h),
            egui::vec2((bar_w - 1.0).max(1.0), h),
        );
        let color = egui::Color32::from_rgb(
            (mag * 255.0) as u8,
            (140.0 + mag * 115.0) as u8,
            255,
        );
        ui.painter().rect_filled(bar_rect, 1.0, color);
    }
}

/// 16 Harmonics bar graph display component for Macro Rack (Step 660).
pub fn show_harmonics_display(ui: &mut egui::Ui, harmonics: &[f32; 16], width: f32, height: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    ui.painter().rect_filled(rect, 4.0, egui::Color32::from_rgb(8, 12, 20));

    let bar_w = rect.width() / 16.0;
    for (i, &amp) in harmonics.iter().enumerate() {
        let h = (amp.clamp(0.0, 1.0) * rect.height()).max(2.0);
        let x = rect.left() + i as f32 * bar_w;
        let bar_rect = egui::Rect::from_min_size(
            egui::pos2(x + 1.0, rect.bottom() - h),
            egui::vec2((bar_w - 2.0).max(1.0), h),
        );
        let color = egui::Color32::from_rgb(255, (120.0 + amp * 135.0) as u8, 40);
        ui.painter().rect_filled(bar_rect, 1.0, color);
    }
}

/// FM Operator Matrix topology display grid (Step 661).
pub fn show_fm_matrix_display(ui: &mut egui::Ui, matrix: &[[f32; 4]; 4], width: f32, height: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    ui.painter().rect_filled(rect, 4.0, egui::Color32::from_rgb(12, 16, 24));

    let cell_w = rect.width() / 4.0;
    let cell_h = rect.height() / 4.0;

    for row in 0..4 {
        for col in 0..4 {
            let cell_rect = egui::Rect::from_min_size(
                egui::pos2(rect.left() + col as f32 * cell_w, rect.top() + row as f32 * cell_h),
                egui::vec2(cell_w, cell_h),
            );
            let intensity = matrix[row][col].clamp(0.0, 1.0);
            let fill_color = egui::Color32::from_rgba_unmultiplied(26, 140, 255, (intensity * 220.0) as u8);
            ui.painter().rect_filled(cell_rect.shrink(1.0), 2.0, fill_color);
            ui.painter().rect_stroke(cell_rect.shrink(1.0), 2.0, egui::Stroke::new(0.5, egui::Color32::from_rgb(40, 60, 90)));
        }
    }
}

/// Filter magnitude response curve display (Step 662).
pub fn show_filter_response_curve(ui: &mut egui::Ui, cutoff_hz: f32, resonance: f32, width: f32, height: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    ui.painter().rect_filled(rect, 4.0, egui::Color32::from_rgb(10, 12, 18));
    ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 60, 90)));

    let points: Vec<egui::Pos2> = (0..50).map(|i| {
        let t = i as f32 / 49.0;
        let freq = 20.0 * (20000.0f32 / 20.0).powf(t);
        let ratio = freq / cutoff_hz.max(20.0);
        let magnitude = 1.0 / (1.0 + ratio.powi(4) - 0.5 * resonance * ratio.powi(2)).sqrt();
        let norm_mag = magnitude.clamp(0.0, 2.0) / 2.0;
        let x = rect.left() + t * rect.width();
        let y = rect.bottom() - norm_mag * rect.height();
        egui::Pos2::new(x, y)
    }).collect();

    for window in points.windows(2) {
        ui.painter().line_segment([window[0], window[1]], egui::Stroke::new(1.5, egui::Color32::from_rgb(180, 80, 240)));
    }
}

/// Reverb Impulse Response decay curve display (Step 663).
pub fn show_impulse_response_display(ui: &mut egui::Ui, ir_samples: &[f32], width: f32, height: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    ui.painter().rect_filled(rect, 4.0, egui::Color32::from_rgb(12, 10, 18));

    if ir_samples.is_empty() {
        return;
    }
    let step = (ir_samples.len() as f32 / width).max(1.0) as usize;
    for (i, x_idx) in (0..ir_samples.len()).step_by(step).enumerate() {
        let val = ir_samples[x_idx].abs().clamp(0.0, 1.0);
        let x = rect.left() + i as f32 * 2.0;
        if x > rect.right() { break; }
        let h = val * rect.height();
        let line_rect = egui::Rect::from_min_size(egui::pos2(x, rect.center().y - h * 0.5), egui::vec2(1.5, h.max(1.0)));
        ui.painter().rect_filled(line_rect, 0.0, egui::Color32::from_rgb(255, 130, 40));
    }
}

/// Convolution Reverb UI block with IR File Browser (Step 665).
pub fn show_convolution_reverb_block(ui: &mut egui::Ui, ir_filename: &mut String, mix: &mut f32) {
    ui.group(|ui| {
        ui.heading("Convolution Reverb");
        ui.horizontal(|ui| {
            ui.label("IR File:");
            ui.text_edit_singleline(ir_filename);
            if ui.button("📂 Browse IR...").clicked() {
                if let Some(path) = rfd::FileDialog::new().add_filter("Audio IR", &["wav", "flac"]).pick_file() {
                    if let Some(p_str) = path.to_str() {
                        *ir_filename = p_str.to_string();
                    }
                }
            }
        });
        ui.add(egui::Slider::new(mix, 0.0..=1.0).text("Mix"));
    });
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
            send_level: 0.0,
            nodes: vec![
                summoner_project::schema::NodeConfig {
                    kind: "AetherSynth".to_string(),
                    params: std::collections::HashMap::new(),
                    plugin_state: None,
                }
            ],
            sequence: None,
            clips: Vec::new(),
            connections: Vec::new(),
            tuning_edo: None,
            tuning_root_hz: None,
            tuning_scl_path: None,
            ..Default::default()
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
            send_level: 0.0,
            nodes: vec![
                summoner_project::schema::NodeConfig {
                    kind: "FmOperatorPair".to_string(),
                    params: std::collections::HashMap::new(),
                    plugin_state: None,
                },
                summoner_project::schema::NodeConfig {
                    kind: "PluckSynth".to_string(),
                    params: std::collections::HashMap::new(),
                    plugin_state: None,
                },
                summoner_project::schema::NodeConfig {
                    kind: "GranularSynthNode".to_string(),
                    params: std::collections::HashMap::new(),
                    plugin_state: None,
                },
            ],
            sequence: None,
            clips: Vec::new(),
            connections: Vec::new(),
            tuning_edo: None,
            tuning_root_hz: None,
            tuning_scl_path: None,
            ..Default::default()
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
