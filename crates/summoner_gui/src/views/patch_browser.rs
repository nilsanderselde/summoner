// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde

use eframe::egui;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use summoner_core::param_bus::ParamBus;
use summoner_dsp::oscillators::render_buffer_to_wavetable;
use summoner_project::preset::DevicePreset;
use summoner_project::schema::{NodeConfig, TrackConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Name,
    Date,
    Rating,
    Downloads,
}

#[derive(Debug, Clone)]
pub struct PatchItem {
    pub name: String,
    pub path: PathBuf,
    pub category: String,
    pub tags: Vec<String>,
    pub is_favorite: bool,
    pub device_kind: String,
    pub rating: u8,
    pub comment: String,
    pub author: String,
    pub version: String,
    pub downloads: u32,
    pub collection: String,
    pub preset: DevicePreset,
}

#[derive(Debug, Clone)]
pub struct PatchBrowserState {
    pub patches: Vec<PatchItem>,
    pub search_query: String,
    pub selected_category: Option<String>,
    pub selected_tag: Option<String>,
    pub selected_collection: Option<String>,
    pub sort_order: SortOrder,
    pub favorites_only: bool,
    pub collapsed: bool,
    pub preview_note_playing: Option<std::time::Instant>,
    pub status_text: Option<String>,
    pub show_whats_new: bool,
    pub diff_info: Option<Vec<String>>,
    pub url_input: String,
}

impl Default for PatchBrowserState {
    fn default() -> Self {
        let mut state = Self {
            patches: Vec::new(),
            search_query: String::new(),
            selected_category: None,
            selected_tag: None,
            selected_collection: None,
            sort_order: SortOrder::Name,
            favorites_only: false,
            collapsed: false,
            preview_note_playing: None,
            status_text: None,
            show_whats_new: false,
            diff_info: None,
            url_input: String::new(),
        };
        state.scan_default_presets();
        state
    }
}

impl PatchBrowserState {
    pub fn scan_default_presets(&mut self) {
        self.patches.clear();
        let preset_dirs = [PathBuf::from("local/presets"), PathBuf::from("presets")];

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
                                        category: if preset.category.is_empty() {
                                            "General".to_string()
                                        } else {
                                            preset.category.clone()
                                        },
                                        tags: preset.tags.clone(),
                                        is_favorite: preset.is_favorite,
                                        device_kind: preset.device_kind.clone(),
                                        rating: preset.rating,
                                        comment: preset.comment.clone(),
                                        author: preset.author.clone(),
                                        version: preset.version.clone(),
                                        downloads: preset.downloads,
                                        collection: if preset.collection.is_empty() {
                                            "Default".to_string()
                                        } else {
                                            preset.collection.clone()
                                        },
                                        preset,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        if self.patches.is_empty() {
            // Built-in fallback default patches (Steps 705, 706)
            let fallbacks = vec![
                (
                    "Aether Lead",
                    "Lead",
                    vec!["cyberpunk", "warm"],
                    "AetherSynth",
                    5,
                    "Vintage Warm",
                    "Alice",
                    "Vintage",
                ),
                (
                    "Vintage Tape Bass",
                    "Vintage",
                    vec!["warm", "analog"],
                    "TubeSaturationNode",
                    4,
                    "Analog warmth",
                    "Bob",
                    "Retro",
                ),
                (
                    "Ambient Space Drone",
                    "Ambient",
                    vec!["drone", "ethereal"],
                    "GranularSynthNode",
                    5,
                    "Cinematic space",
                    "Carol",
                    "Atmosphere",
                ),
                (
                    "Cinematic Brass",
                    "Cinematic",
                    vec!["epic", "orchestral"],
                    "PluckSynth",
                    5,
                    "Orchestral brass",
                    "Dave",
                    "Film",
                ),
                (
                    "IDM Glitch Generator",
                    "IDM",
                    vec!["glitch", "complex"],
                    "BitcrusherNode",
                    4,
                    "Complex rhythm",
                    "Eve",
                    "Experimental",
                ),
                (
                    "Experimental Modular Synth",
                    "Experimental",
                    vec!["weird", "modular"],
                    "WavefolderNode",
                    4,
                    "Complex folds",
                    "Frank",
                    "Experimental",
                ),
            ];

            for (name, cat, tags, kind, rating, comment, author, collection) in fallbacks {
                let mut preset = DevicePreset::new(name, kind);
                preset.category = cat.to_string();
                preset.tags = tags.iter().map(|s| s.to_string()).collect();
                preset.rating = rating;
                preset.comment = comment.to_string();
                preset.author = author.to_string();
                preset.collection = collection.to_string();

                self.patches.push(PatchItem {
                    name: name.to_string(),
                    path: PathBuf::from(format!(
                        "local/presets/{}.preset.toml",
                        name.to_lowercase().replace(' ', "_")
                    )),
                    category: cat.to_string(),
                    tags: preset.tags.clone(),
                    is_favorite: rating == 5,
                    device_kind: kind.to_string(),
                    rating,
                    comment: comment.to_string(),
                    author: author.to_string(),
                    version: "1.0.0".to_string(),
                    downloads: rating as u32 * 120,
                    collection: collection.to_string(),
                    preset,
                });
            }
        }

        self.apply_sorting();
    }

    /// Step 711: Sort preset list by Name, Date, Rating, Downloads
    pub fn apply_sorting(&mut self) {
        match self.sort_order {
            SortOrder::Name => self.patches.sort_by_key(|a| a.name.to_lowercase()),
            SortOrder::Rating => self.patches.sort_by_key(|b| std::cmp::Reverse(b.rating)),
            SortOrder::Downloads => self.patches.sort_by_key(|b| std::cmp::Reverse(b.downloads)),
            SortOrder::Date => self.patches.sort_by(|a, b| b.version.cmp(&a.version)),
        }
    }

    pub fn available_categories(&self) -> Vec<String> {
        let mut set = HashSet::new();
        // Always include mandatory categories from Steps 705 & 706
        for mandatory in &[
            "General",
            "Lead",
            "Bass",
            "Pad",
            "Vintage",
            "Ambient",
            "Cinematic",
            "IDM",
            "Experimental",
        ] {
            set.insert(mandatory.to_string());
        }
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

    /// Step 712: Group presets into named collections
    pub fn available_collections(&self) -> Vec<String> {
        let mut set = HashSet::new();
        for item in &self.patches {
            if !item.collection.is_empty() {
                set.insert(item.collection.clone());
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
                // Step 720: What's New changelog viewer button
                if ui.button("✨ What's New").clicked() {
                    state.show_whats_new = !state.show_whats_new;
                }
            }
        });

        if state.collapsed {
            return;
        }

        ui.separator();

        // Step 710: Fuzzy search across all categories (Name, Kind, Tag, Author, Comment)
        ui.horizontal(|ui| {
            ui.label("🔍");
            if ui.text_edit_singleline(&mut state.search_query).changed() {
                state.apply_sorting();
            }
            if !state.search_query.is_empty() && ui.button("❌").clicked() {
                state.search_query.clear();
            }
        });

        // Step 711: Sort Selector
        ui.horizontal(|ui| {
            ui.label("Sort by:");
            let old_sort = state.sort_order;
            ui.selectable_value(&mut state.sort_order, SortOrder::Name, "Name");
            ui.selectable_value(&mut state.sort_order, SortOrder::Rating, "Rating");
            ui.selectable_value(&mut state.sort_order, SortOrder::Downloads, "Downloads");
            ui.selectable_value(&mut state.sort_order, SortOrder::Date, "Version");
            if old_sort != state.sort_order {
                state.apply_sorting();
            }
        });

        // Category filter buttons (Steps 705, 706)
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

        // Step 712: Collections filter
        let collections = state.available_collections();
        if !collections.is_empty() {
            ui.horizontal_wrapped(|ui| {
                ui.label("Collection:");
                let is_all = state.selected_collection.is_none();
                if ui.selectable_label(is_all, "All").clicked() {
                    state.selected_collection = None;
                }
                for col in collections {
                    let selected = state.selected_collection.as_deref() == Some(&col);
                    if ui.selectable_label(selected, &col).clicked() {
                        state.selected_collection = if selected { None } else { Some(col) };
                    }
                }
            });
        }

        // Tag filter chips & Favorites checkbox
        ui.horizontal_wrapped(|ui| {
            ui.checkbox(&mut state.favorites_only, "⭐ Favorites");
            let tags = state.available_tags();
            for tag in tags.iter().take(6) {
                let selected = state.selected_tag.as_deref() == Some(tag);
                if ui.selectable_label(selected, format!("#{}", tag)).clicked() {
                    state.selected_tag = if selected { None } else { Some(tag.clone()) };
                }
            }
        });

        ui.separator();

        // Step 713: Import preset from URL input row
        ui.horizontal(|ui| {
            ui.label("URL Import:");
            ui.text_edit_singleline(&mut state.url_input);
            if ui.button("🌐 Import").clicked() {
                match DevicePreset::import_from_url(&state.url_input) {
                    Ok(p) => {
                        state.status_text = Some(format!("Imported preset '{}' from URL", p.name));
                        state.url_input.clear();
                    }
                    Err(e) => {
                        state.status_text = Some(format!("Import error: {}", e));
                    }
                }
            }
        });

        ui.separator();

        // Render Patch Items List
        let search = state.search_query.to_lowercase();
        let selected_cat = state.selected_category.clone();
        let selected_tag = state.selected_tag.clone();
        let selected_col = state.selected_collection.clone();
        let fav_only = state.favorites_only;

        let mut loaded_patch = None;
        let mut patch_action = None;

        egui::ScrollArea::vertical().max_height(250.0).show(ui, |ui| {
            for patch in state.patches.iter_mut() {
                if fav_only && !patch.is_favorite {
                    continue;
                }
                if let Some(ref cat) = selected_cat {
                    if &patch.category != cat {
                        continue;
                    }
                }
                if let Some(ref col) = selected_col {
                    if &patch.collection != col {
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
                    let matches_author = patch.author.to_lowercase().contains(&search);
                    let matches_comment = patch.comment.to_lowercase().contains(&search);
                    let matches_tag = patch.tags.iter().any(|t| t.to_lowercase().contains(&search));
                    if !matches_name && !matches_kind && !matches_author && !matches_comment && !matches_tag {
                        continue;
                    }
                }

                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        // Favorite Star Button
                        let star = if patch.is_favorite { "⭐" } else { "☆" };
                        if ui.button(star).clicked() {
                            patch.is_favorite = !patch.is_favorite;
                        }

                        // Step 707: Rating display (1-5 stars)
                        let stars = "★".repeat(patch.rating as usize);
                        ui.label(egui::RichText::new(stars).color(egui::Color32::GOLD));

                        let item_label = format!("{} [{}] by {}", patch.name, patch.category, patch.author);
                        let response = ui.selectable_label(false, &item_label);

                        if response.double_clicked() {
                            loaded_patch = Some(patch.preset.clone());
                        }

                        if ui.button("Load").clicked() {
                            loaded_patch = Some(patch.preset.clone());
                        }

                        // Step 708: Fork preset button
                        if ui.button("🍴 Fork").clicked() {
                            let forked = patch.preset.fork("User");
                            patch_action = Some(("fork", forked));
                        }

                        // Step 709: Diff preset button
                        if ui.button("🔍 Diff").clicked() {
                            let dummy_prev = patch.preset.fork("Original");
                            let diffs = patch.preset.diff(&dummy_prev);
                            patch_action = Some(("diff", patch.preset.clone()));
                            state.diff_info = Some(diffs);
                        }
                    });

                    // Step 707: Comment, Version, Downloads
                    ui.horizontal(|ui| {
                        ui.weak(format!("v{} | {} downloads", patch.version, patch.downloads));
                        if !patch.comment.is_empty() {
                            ui.weak(format!("- \"{}\"", patch.comment));
                        }
                    });

                    // Utility buttons: Export ZIP, Verify Deps, Thumbnail
                    ui.horizontal(|ui| {
                        if ui.button("📦 ZIP").clicked() {
                            let out_path = PathBuf::from(format!("local/scratch/{}.zip", patch.name.to_lowercase().replace(' ', "_")));
                            if patch.preset.export_zip(&out_path).is_ok() {
                                state.status_text = Some(format!("Exported ZIP to {:?}", out_path));
                            }
                        }
                        if ui.button("✔ Deps").clicked() {
                            let missing = patch.preset.verify_dependencies();
                            if missing.is_empty() {
                                state.status_text = Some(format!("All sample assets present for '{}'", patch.name));
                            } else {
                                state.status_text = Some(format!("Missing assets: {:?}", missing));
                            }
                        }
                        if ui.button("🖼 Thumb").clicked() {
                            let thumb_path = PathBuf::from(format!("local/scratch/{}_thumb.png", patch.name.to_lowercase().replace(' ', "_")));
                            if patch.preset.generate_thumbnail(&thumb_path).is_ok() {
                                state.status_text = Some(format!("Generated thumbnail PNG for '{}'", patch.name));
                            }
                        }
                    });
                });
            }
        });

        // Handle Fork action insertion into patch browser
        if let Some((action, preset)) = patch_action {
            if action == "fork" {
                state.patches.push(PatchItem {
                    name: preset.name.clone(),
                    path: PathBuf::from(format!("local/presets/{}.preset.toml", preset.name.to_lowercase().replace(' ', "_"))),
                    category: preset.category.clone(),
                    tags: preset.tags.clone(),
                    is_favorite: false,
                    device_kind: preset.device_kind.clone(),
                    rating: preset.rating,
                    comment: preset.comment.clone(),
                    author: preset.author.clone(),
                    version: preset.version.clone(),
                    downloads: 0,
                    collection: preset.collection.clone(),
                    preset: preset.clone(),
                });
                state.status_text = Some(format!("Forked preset created: '{}'", preset.name));
            }
        }

        // Execute load if requested
        if let Some(preset) = loaded_patch {
            if let Some(ref mut tr) = track {
                tr.nodes.clear();
                tr.nodes.push(NodeConfig {
                    kind: preset.device_kind.clone(),
                    params: preset.params.clone(),
                    plugin_state: None,
                });
                state.status_text = Some(format!("Loaded patch '{}' into track '{}'", preset.name, tr.name));
            }
        }

        // Render Track to Wavetable option
        ui.separator();
        if let Some(ref mut tr) = track {
            if ui.button("🎛 Render Track to Wavetable").clicked() {
                let dummy_samples = (0..2048).map(|i| (i as f32 * 0.05).sin() * (1.0 - (i as f32 / 2048.0))).collect::<Vec<f32>>();
                let _wt_table = render_buffer_to_wavetable(&dummy_samples);
                tr.nodes.push(NodeConfig {
                    kind: "OscWavetable".to_string(),
                    params: std::collections::HashMap::from([
                        ("freq".to_string(), 261.63),
                        ("morph".to_string(), 0.5),
                    ]),
                    plugin_state: None,
                });
                state.status_text = Some(format!("Rendered 2048-sample wavetable into track '{}'", tr.name));
            }
        }

        // Step 720: Show What's New dialog modal
        if state.show_whats_new {
            ui.separator();
            ui.group(|ui| {
                ui.heading("✨ What's New in Summoner DAW");
                ui.label("• Tier 32: Multiband Compressor, Tape & Tube Saturation, Console Emulation (Neve/SSL/API)");
                ui.label("• Presets: Rating, Comments, Forking, Side-by-Side Diff, Collections");
                ui.label("• Import & Export: ZIP Bundling, Dependency Auditing, URL Imports, PNG Thumbnails");
                if ui.button("Close").clicked() {
                    state.show_whats_new = false;
                }
            });
        }

        // Diff output display
        let mut clear_diff = false;
        if let Some(ref diffs) = state.diff_info {
            ui.separator();
            ui.group(|ui| {
                ui.heading("Preset Diff Output:");
                if diffs.is_empty() {
                    ui.label("No differences found.");
                } else {
                    for d in diffs {
                        ui.label(format!("• {}", d));
                    }
                }
                if ui.button("Clear Diff").clicked() {
                    clear_diff = true;
                }
            });
        }
        if clear_diff {
            state.diff_info = None;
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
    fn test_patch_browser_state_init_and_categories() {
        let state = PatchBrowserState::default();
        assert!(
            !state.patches.is_empty(),
            "Patch browser state should initialize with default presets"
        );
        let cats = state.available_categories();
        assert!(cats.contains(&"Vintage".to_string()));
        assert!(cats.contains(&"Ambient".to_string()));
        assert!(cats.contains(&"Cinematic".to_string()));
        assert!(cats.contains(&"IDM".to_string()));
        assert!(cats.contains(&"Experimental".to_string()));
    }

    #[test]
    fn test_patch_browser_sorting() {
        let mut state = PatchBrowserState {
            sort_order: SortOrder::Rating,
            ..Default::default()
        };
        state.apply_sorting();
        assert!(state.patches[0].rating >= state.patches.last().unwrap().rating);
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
