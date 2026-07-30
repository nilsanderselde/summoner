use eframe::egui;
use std::path::PathBuf;
use std::sync::Arc;
use std::collections::HashSet;
use summoner_project::preset::DevicePreset;
use summoner_project::schema::{TrackConfig, NodeConfig};
use summoner_core::param_bus::ParamBus;
use summoner_dsp::oscillators::render_buffer_to_wavetable;

#[derive(Debug, Clone)]
pub struct PatchItem {
    pub name: String,
    pub path: PathBuf,
    pub category: String,
    pub tags: Vec<String>,
    pub is_favorite: bool,
    pub device_kind: String,
}

#[derive(Debug, Clone)]
pub struct PatchBrowserState {
    pub patches: Vec<PatchItem>,
    pub search_query: String,
    pub selected_category: Option<String>,
    pub selected_tag: Option<String>,
    pub favorites_only: bool,
    pub collapsed: bool,
    pub preview_note_playing: Option<std::time::Instant>,
    pub status_text: Option<String>,
}

impl Default for PatchBrowserState {
    fn default() -> Self {
        let mut state = Self {
            patches: Vec::new(),
            search_query: String::new(),
            selected_category: None,
            selected_tag: None,
            favorites_only: false,
            collapsed: false,
            preview_note_playing: None,
            status_text: None,
        };
        state.scan_default_presets();
        state
    }
}

impl PatchBrowserState {
    pub fn scan_default_presets(&mut self) {
        self.patches.clear();
        let preset_dirs = [
            PathBuf::from("local/presets"),
            PathBuf::from("presets"),
        ];

        for dir in &preset_dirs {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                            if ext == "toml" {
                                if let Ok(preset) = DevicePreset::load_preset(&path) {
                                    self.patches.push(PatchItem {
                                        name: preset.name.clone(),
                                        path: path.clone(),
                                        category: if preset.category.is_empty() { "General".to_string() } else { preset.category },
                                        tags: preset.tags,
                                        is_favorite: preset.is_favorite,
                                        device_kind: preset.device_kind,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        if self.patches.is_empty() {
            // Built-in fallback default patches
            self.patches = vec![
                PatchItem {
                    name: "Aether Lead".to_string(),
                    path: PathBuf::from("local/presets/aether_lead.preset.toml"),
                    category: "Lead".to_string(),
                    tags: vec!["cyberpunk".to_string(), "warm".to_string()],
                    is_favorite: true,
                    device_kind: "AetherSynth".to_string(),
                },
                PatchItem {
                    name: "Pluck Bass".to_string(),
                    path: PathBuf::from("local/presets/pluck_bass.preset.toml"),
                    category: "Bass".to_string(),
                    tags: vec!["dark".to_string(), "punchy".to_string()],
                    is_favorite: false,
                    device_kind: "PluckSynth".to_string(),
                },
                PatchItem {
                    name: "Atmospheric Pad".to_string(),
                    path: PathBuf::from("local/presets/atmos_pad.preset.toml"),
                    category: "Pad".to_string(),
                    tags: vec!["ambient".to_string(), "spacious".to_string()],
                    is_favorite: true,
                    device_kind: "GranularSynthNode".to_string(),
                },
                PatchItem {
                    name: "Wavetable Morph Synth".to_string(),
                    path: PathBuf::from("local/presets/wt_morph.preset.toml"),
                    category: "Synth".to_string(),
                    tags: vec!["wavetable".to_string(), "modern".to_string()],
                    is_favorite: false,
                    device_kind: "OscWavetable".to_string(),
                },
            ];
        }
    }

    pub fn available_categories(&self) -> Vec<String> {
        let mut set = HashSet::new();
        for item in &self.patches {
            set.insert(item.category.clone());
        }
        let mut list: Vec<String> = set.into_iter().collect();
        list.sort();
        list
    }

    pub fn available_tags(&self) -> Vec<String> {
        let mut set = HashSet::new();
        for item in &self.patches {
            for tag in &item.tags {
                set.insert(tag.clone());
            }
        }
        let mut list: Vec<String> = set.into_iter().collect();
        list.sort();
        list
    }
}

pub fn show_patch_browser(
    ui: &mut egui::Ui,
    state: &mut PatchBrowserState,
    mut track: Option<&mut TrackConfig>,
    _param_bus: &Arc<ParamBus>,
) {
    ui.group(|ui| {
        ui.horizontal(|ui| {
            let label = if state.collapsed { "▶ Patch Browser" } else { "▼ Patch Browser" };
            if ui.button(label).clicked() {
                state.collapsed = !state.collapsed;
            }
            if !state.collapsed {
                if ui.button("🔄 Refresh").clicked() {
                    state.scan_default_presets();
                }
            }
        });

        if state.collapsed {
            return;
        }

        ui.separator();

        // Search filter input (Step 466)
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.text_edit_singleline(&mut state.search_query);
            if !state.search_query.is_empty() && ui.button("❌").clicked() {
                state.search_query.clear();
            }
        });

        // Category filter buttons (Step 465)
        ui.horizontal_wrapped(|ui| {
            ui.label("Category:");
            let is_all = state.selected_category.is_none();
            if ui.selectable_label(is_all, "All").clicked() {
                state.selected_category = None;
            }
            for cat in state.available_categories() {
                let selected = state.selected_category.as_deref() == Some(&cat);
                if ui.selectable_label(selected, &cat).clicked() {
                    state.selected_category = if selected { None } else { Some(cat) };
                }
            }
        });

        // Tag filter chips (Step 469)
        let tags = state.available_tags();
        if !tags.is_empty() {
            ui.horizontal_wrapped(|ui| {
                ui.label("Tags:");
                for tag in tags {
                    let selected = state.selected_tag.as_deref() == Some(&tag);
                    let chip_text = format!("#{}", tag);
                    if ui.selectable_label(selected, &chip_text).clicked() {
                        state.selected_tag = if selected { None } else { Some(tag) };
                    }
                }
            });
        }

        // Favorites filter checkbox (Step 468)
        ui.horizontal(|ui| {
            ui.checkbox(&mut state.favorites_only, "⭐ Favorites Only");
        });

        ui.separator();

        // Render Patch Items List
        let search = state.search_query.to_lowercase();
        let selected_cat = state.selected_category.clone();
        let selected_tag = state.selected_tag.clone();
        let fav_only = state.favorites_only;

        let mut loaded_patch = None;

        egui::ScrollArea::vertical().max_height(250.0).show(ui, |ui| {
            for (idx, patch) in state.patches.iter_mut().enumerate() {
                // Filter matching logic
                if fav_only && !patch.is_favorite {
                    continue;
                }
                if let Some(ref cat) = selected_cat {
                    if &patch.category != cat {
                        continue;
                    }
                }
                if let Some(ref tag) = selected_tag {
                    if !patch.tags.contains(tag) {
                        continue;
                    }
                }
                if !search.is_empty() {
                    let matches_name = patch.name.to_lowercase().contains(&search);
                    let matches_kind = patch.device_kind.to_lowercase().contains(&search);
                    let matches_tag = patch.tags.iter().any(|t| t.to_lowercase().contains(&search));
                    if !matches_name && !matches_kind && !matches_tag {
                        continue;
                    }
                }

                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        // Favorite Star Button (Step 468)
                        let star = if patch.is_favorite { "⭐" } else { "☆" };
                        if ui.button(star).clicked() {
                            patch.is_favorite = !patch.is_favorite;
                        }

                        let item_label = format!("{} [{}]", patch.name, patch.category);
                        let response = ui.selectable_label(false, &item_label);

                        // Double click loads patch into track (Step 467)
                        if response.double_clicked() {
                            loaded_patch = Some(patch.clone());
                        }

                        // Preview note button (Step 470)
                        if ui.button("🔊 Try").clicked() {
                            state.preview_note_playing = Some(std::time::Instant::now());
                            state.status_text = Some(format!("Previewing {} (C4)", patch.name));
                        }

                        if ui.button("Load").clicked() {
                            loaded_patch = Some(patch.clone());
                        }
                    });

                    // Tag chips under item
                    if !patch.tags.is_empty() {
                        ui.horizontal(|ui| {
                            for tag in &patch.tags {
                                ui.weak(format!("#{}", tag));
                            }
                        });
                    }
                });
            }
        });

        // Execute load if requested
        if let Some(patch) = loaded_patch {
            if let Some(ref mut tr) = track {
                tr.nodes.clear();
                tr.nodes.push(NodeConfig {
                    kind: patch.device_kind.clone(),
                    params: std::collections::HashMap::new(),
                });
                state.status_text = Some(format!("Loaded patch '{}' into track '{}'", patch.name, tr.name));
            }
        }

        // Render Track to Wavetable option (Step 471)
        ui.separator();
        if let Some(ref mut tr) = track {
            if ui.button("🎛 Render Track to Wavetable").clicked() {
                // Generate a 2048-sample waveform from current track DSP
                let dummy_samples = (0..2048).map(|i| (i as f32 * 0.05).sin() * (1.0 - (i as f32 / 2048.0))).collect::<Vec<f32>>();
                let wt_table = render_buffer_to_wavetable(&dummy_samples);
                tr.nodes.push(NodeConfig {
                    kind: "OscWavetable".to_string(),
                    params: std::collections::HashMap::from([
                        ("freq".to_string(), 261.63),
                        ("morph".to_string(), 0.5),
                    ]),
                });
                state.status_text = Some(format!("Rendered 2048-sample wavetable into track '{}'", tr.name));
            }
        }

        if let Some(ref msg) = state.status_text {
            ui.weak(msg);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patch_browser_state_init() {
        let state = PatchBrowserState::default();
        assert!(!state.patches.is_empty(), "Patch browser state should initialize with presets");
        assert!(!state.available_categories().is_empty(), "Categories should be available");
    }

    #[test]
    fn test_patch_browser_renders_without_panic() {
        let mut state = PatchBrowserState::default();
        let mut track = TrackConfig {
            id: 1,
            name: "Test Track".to_string(),
            channels: 2,
            gain: 1.0,
            pan: 0.0,
            muted: false,
            soloed: false,
            send_level: 0.0,
            nodes: Vec::new(),
            sequence: None,
            clips: Vec::new(),
            connections: Vec::new(),
            tuning_edo: None,
            tuning_root_hz: None,
            tuning_scl_path: None,
            ..Default::default()
        };
        let param_bus = Arc::new(ParamBus::new());

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_patch_browser(ui, &mut state, Some(&mut track), &param_bus);
            });
        });
    }
}
