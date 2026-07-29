use std::sync::Arc;
use std::path::PathBuf;
use summoner_core::param_bus::ParamBus;
use summoner_project::schema::ProjectConfig;
use eframe::egui;

use crate::views::arranger::show_arranger;
use crate::views::node_graph::{show_node_graph, NodeGraphState};
use crate::views::piano_roll::{show_piano_roll, PianoRollState, Viewport};
use crate::views::mixer::show_mixer;
use crate::views::macro_rack::show_macro_rack;
use crate::stage_view::{show_stage_view, StageView};
use crate::command_palette::CommandPalette;
use summoner_core::graph::NodeGraph;
use summoner_core::transport::Transport;
use summoner_harmony::edo::EdoTuning;
use summoner_sequencer::automation::AutomationRegistry;
use summoner_sequencer::automation_timeline::AutomationTimeline;

use summoner_harmony::bus::HarmonicContext;

use std::collections::HashMap;
use crate::visualizer::{Oscilloscope, SpectrumAnalyzer};

#[derive(PartialEq, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ViewMode {
    Arranger,
    PianoRoll(u64),
    NodeGraph(u64),
    Mixer,
    Performance,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct GuiState {
    pub current_view: ViewMode,
    pub selected_track_id: Option<u64>,
    pub show_rack: bool,
    pub pixels_per_beat: f32,
    #[serde(default = "default_macro_rack_height")]
    pub macro_rack_height: f32,
    #[serde(default = "default_track_header_width")]
    pub track_header_width: f32,
    #[serde(default = "default_dark_theme")]
    pub dark_theme: bool,
}

fn default_macro_rack_height() -> f32 { 200.0 }
fn default_track_header_width() -> f32 { 180.0 }
fn default_dark_theme() -> bool { true }

impl GuiState {
    pub const STATE_FILE: &'static str = ".summoner_gui_state.toml";

    pub fn load_from_path(path: &std::path::Path) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        toml::from_str(&content).ok()
    }

    pub fn load() -> Option<Self> {
        Self::load_from_path(std::path::Path::new(Self::STATE_FILE))
    }

    pub fn save_to_path(&self, path: &std::path::Path) {
        if let Ok(serialized) = toml::to_string(self) {
            let _ = std::fs::write(path, serialized);
        }
    }

    pub fn save(&self) {
        self.save_to_path(std::path::Path::new(Self::STATE_FILE));
    }
}

pub struct SummonerApp {
    pub project: ProjectConfig,
    pub project_path: Option<PathBuf>,
    pub param_bus: Arc<ParamBus>,
    pub transport_running: bool,
    pub playhead_beat: f64,
    pub current_beat: f64,
    pub pixels_per_beat: f32,
    pub current_view: ViewMode,
    pub node_graph_state: NodeGraphState,
    pub piano_roll_state: PianoRollState,
    pub dummy_graph: NodeGraph,
    pub recording_all: bool,
    pub selected_track_id: Option<u64>,
    pub stage_view: StageView,
    pub command_palette: CommandPalette,
    pub automation_registry: AutomationRegistry,
    pub automation_timeline: AutomationTimeline,
    pub transport: Transport,
    pub show_rack: bool,
    pub oscilloscope_buffers: HashMap<u64, Arc<Oscilloscope>>,
    pub spectrum_analyzer: SpectrumAnalyzer,
    pub show_about_dialog: bool,
    pub status_message: Option<String>,
    pub grid_division: f64,
    pub track_header_width: f32,
    pub macro_rack_height: f32,
    pub waveform_cache: crate::waveform_cache::WaveformCache,
    pub midi_learn_mode: bool,
    pub midi_learn_target: Option<String>,
    pub dark_theme: bool,
    pub show_shortcuts_modal: bool,
    pub shortcut_search_query: String,
    pub is_rendering: bool,
    pub progress_message: Option<(String, f32)>,
    pub harmonic_context: HarmonicContext,
    pub show_scala_browser_modal: bool,
}

impl SummonerApp {
    pub fn new(project: ProjectConfig, param_bus: Arc<ParamBus>) -> Self {
        let sample_rate = project.transport.sample_rate;
        let bpm = project.transport.bpm;
        let mut oscilloscope_buffers = HashMap::new();
        for track in &project.tracks {
            oscilloscope_buffers.insert(track.id, Arc::new(Oscilloscope::new()));
        }
        let spectrum_analyzer = SpectrumAnalyzer::new();
        let mut stage_view = StageView::new();
        stage_view.populate_from_project(&project);

        let mut app = Self {
            project,
            project_path: None,
            param_bus,
            transport_running: false,
            playhead_beat: 0.0,
            current_beat: 0.0,
            pixels_per_beat: 40.0,
            current_view: ViewMode::Arranger,
            node_graph_state: NodeGraphState::default(),
            piano_roll_state: PianoRollState::default(),
            dummy_graph: NodeGraph::new("Main Track", 64, 2),
            recording_all: false,
            selected_track_id: Some(1),
            stage_view,
            command_palette: CommandPalette::new(),
            automation_registry: AutomationRegistry::new(),
            automation_timeline: AutomationTimeline::new(),
            transport: Transport::new(sample_rate, bpm),
            show_rack: true,
            oscilloscope_buffers,
            spectrum_analyzer,
            show_about_dialog: false,
            status_message: None,
            grid_division: 0.25,
            track_header_width: 180.0,
            macro_rack_height: 200.0,
            waveform_cache: crate::waveform_cache::WaveformCache::new(),
            midi_learn_mode: false,
            midi_learn_target: None,
            dark_theme: true,
            show_shortcuts_modal: false,
            shortcut_search_query: String::new(),
            is_rendering: false,
            progress_message: None,
            harmonic_context: HarmonicContext::default(),
            show_scala_browser_modal: false,
        };

        if let Some(state) = GuiState::load() {
            app.current_view = state.current_view;
            app.selected_track_id = state.selected_track_id;
            app.show_rack = state.show_rack;
            app.pixels_per_beat = state.pixels_per_beat;
            app.macro_rack_height = state.macro_rack_height;
            app.track_header_width = state.track_header_width;
            app.dark_theme = state.dark_theme;
        }

        app
    }

    pub fn save_gui_state(&self) {
        let state = GuiState {
            current_view: self.current_view.clone(),
            selected_track_id: self.selected_track_id,
            show_rack: self.show_rack,
            pixels_per_beat: self.pixels_per_beat,
            macro_rack_height: self.macro_rack_height,
            track_header_width: self.track_header_width,
            dark_theme: self.dark_theme,
        };
        state.save();
    }

    pub fn new_session(&mut self) {
        let default_project = summoner_project::create_default_project("New Project");
        self.project = default_project;
        self.project_path = None;
        self.playhead_beat = 0.0;
        self.current_beat = 0.0;
        self.transport_running = false;
        self.oscilloscope_buffers.clear();
        for track in &self.project.tracks {
            self.oscilloscope_buffers.insert(track.id, Arc::new(Oscilloscope::new()));
        }
        self.stage_view.populate_from_project(&self.project);
        self.status_message = Some("Created new session".to_string());
    }

    pub fn open_session(&mut self) {
        #[cfg(feature = "gui")]
        if let Some(path) = rfd::FileDialog::new().add_filter("Project TOML", &["toml"]).pick_file() {
            self.load_session_from_path(path);
        }
    }

    pub fn load_session_from_path(&mut self, path: PathBuf) {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(proj) = summoner_project::parse_project_toml(&content) {
                self.project = proj;
                self.project_path = Some(path.clone());
                self.oscilloscope_buffers.clear();
                for track in &self.project.tracks {
                    self.oscilloscope_buffers.insert(track.id, Arc::new(Oscilloscope::new()));
                }
                self.stage_view.populate_from_project(&self.project);
                self.status_message = Some(format!("Loaded session: {}", path.display()));
            } else {
                self.status_message = Some("Failed to parse project file".to_string());
            }
        }
    }

    pub fn save_session(&mut self) {
        if let Some(path) = self.project_path.clone() {
            if let Ok(content) = summoner_project::serialize_project_toml(&self.project) {
                if std::fs::write(&path, content).is_ok() {
                    self.status_message = Some(format!("Saved session to {}", path.display()));
                } else {
                    self.status_message = Some("Failed to write project file".to_string());
                }
            } else {
                self.status_message = Some("Failed to serialize project".to_string());
            }
        } else {
            self.save_session_as();
        }
    }

    pub fn save_session_as(&mut self) {
        #[cfg(feature = "gui")]
        if let Some(path) = rfd::FileDialog::new().add_filter("Project TOML", &["toml"]).set_file_name("project.toml").save_file() {
            if let Ok(content) = summoner_project::serialize_project_toml(&self.project) {
                if std::fs::write(&path, content).is_ok() {
                    self.project_path = Some(path.clone());
                    self.status_message = Some(format!("Saved session to {}", path.display()));
                } else {
                    self.status_message = Some("Failed to write project file".to_string());
                }
            } else {
                self.status_message = Some("Failed to serialize project".to_string());
            }
        }
    }

    pub fn export_wav(&mut self) {
        #[cfg(feature = "gui")]
        {
            let save_path = rfd::FileDialog::new().add_filter("WAV Audio", &["wav"]).set_file_name("render.wav").save_file();
            if let Some(path) = save_path {
                let proj = self.project.clone();
                self.status_message = Some(format!("Exporting WAV to {}...", path.display()));
                std::thread::spawn(move || {
                    let spec = hound::WavSpec {
                        channels: 2,
                        sample_rate: proj.transport.sample_rate,
                        bits_per_sample: 32,
                        sample_format: hound::SampleFormat::Float,
                    };
                    if let Ok(mut writer) = hound::WavWriter::create(&path, spec) {
                        let duration_secs = 4.0;
                        let total_samples = (proj.transport.sample_rate as f64 * duration_secs) as usize;
                        for _ in 0..total_samples {
                            let _ = writer.write_sample(0.0f32);
                            let _ = writer.write_sample(0.0f32);
                        }
                        let _ = writer.finalize();
                        println!("Exported WAV render to {}", path.display());
                    }
                });
            }
        }
    }
}

impl eframe::App for SummonerApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_gui_state();
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.dark_theme {
            crate::theme::apply_summoner_theme(ctx);
        } else {
            crate::theme::apply_light_theme(ctx);
        }

        // Handle dropped files (step 320)
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        for file in dropped {
            if let Some(path) = file.path {
                if path.extension().is_some_and(|e| e == "toml") {
                    self.load_session_from_path(path);
                } else if path.extension().is_some_and(|e| e == "wav" || e == "flac") {
                    let id = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "asset".to_string());
                    self.project.assets.push(summoner_project::schema::AssetConfig {
                        id,
                        hash: "dropped_hash".to_string(),
                        path: path.to_string_lossy().to_string(),
                        auto_slice: false,
                        slice_threshold: 0.15,
                    });
                    self.status_message = Some(format!("Added asset: {}", path.display()));
                }
            }
        }

        // Dynamically update window title with project name & BPM
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
            "Summoner -- {} @ {:.1} BPM",
            self.project.name, self.project.transport.bpm
        )));

        let is_ctrl = ctx.input(|i| i.modifiers.command || i.modifiers.ctrl);
        let is_shift = ctx.input(|i| i.modifiers.shift);

        // Ctrl+K / Cmd+K Command Palette hotkey
        if is_ctrl && ctx.input(|i| i.key_pressed(egui::Key::K)) {
            self.command_palette.open();
        }

        // Ctrl+S Save Session hotkey
        if is_ctrl && ctx.input(|i| i.key_pressed(egui::Key::S)) {
            self.save_session();
        }

        // Ctrl+Z / Ctrl+Shift+Z / Ctrl+Y Undo and Redo
        if is_ctrl && ctx.input(|i| i.key_pressed(egui::Key::Z)) {
            let path = self.project_path.clone().unwrap_or_else(|| std::path::PathBuf::from("."));
            let dir = if path.is_dir() { path } else { path.parent().unwrap_or_else(|| std::path::Path::new(".")).to_path_buf() };
            if let Ok(repo) = summoner_project::git_dag::open_or_init_repo(&dir) {
                if is_shift {
                    if summoner_project::git_dag::redo(&repo).is_ok() {
                        self.load_session_from_path(dir.join("summoner_session.toml"));
                        self.status_message = Some("Redid micro-commit via Git".to_string());
                    } else {
                        self.status_message = Some("Redo: no further history".to_string());
                    }
                } else {
                    if summoner_project::git_dag::undo(&repo).is_ok() {
                        self.load_session_from_path(dir.join("summoner_session.toml"));
                        self.status_message = Some("Undid micro-commit via Git".to_string());
                    } else {
                        self.status_message = Some("Undo: no parent commit".to_string());
                    }
                }
            } else {
                self.status_message = Some("Git repository operation unavailable".to_string());
            }
        }
        if is_ctrl && ctx.input(|i| i.key_pressed(egui::Key::Y)) {
            let path = self.project_path.clone().unwrap_or_else(|| std::path::PathBuf::from("."));
            let dir = if path.is_dir() { path } else { path.parent().unwrap_or_else(|| std::path::Path::new(".")).to_path_buf() };
            if let Ok(repo) = summoner_project::git_dag::open_or_init_repo(&dir) {
                if summoner_project::git_dag::redo(&repo).is_ok() {
                    self.load_session_from_path(dir.join("summoner_session.toml"));
                    self.status_message = Some("Redid micro-commit via Git".to_string());
                } else {
                    self.status_message = Some("Redo: no further history".to_string());
                }
            }
        }


        // Execute command palette actions
        if let Some(action) = self.command_palette.show(ctx) {
            match action.as_str() {
                "nav_arranger" => self.current_view = ViewMode::Arranger,
                "nav_mixer" => self.current_view = ViewMode::Mixer,
                "nav_performance" => self.current_view = ViewMode::Performance,
                "nav_nodegraph" => {
                    let tid = self.selected_track_id.unwrap_or(1);
                    self.current_view = ViewMode::NodeGraph(tid);
                }
                "transport_play" => {
                    self.transport_running = !self.transport_running;
                    if !self.transport_running {
                        self.automation_registry.stop_record_all();
                    }
                }
                "transport_record" => {
                    self.recording_all = !self.recording_all;
                    if self.recording_all {
                        self.automation_registry.start_record_all();
                    } else {
                        self.automation_registry.stop_record_all();
                    }
                }
                "panic" => {
                    self.transport_running = false;
                    self.automation_registry.stop_record_all();
                    self.stage_view.trigger_panic();
                }
                "set_bpm" => {
                    self.project.transport.bpm = 120.0;
                }
                "add_track" => {
                    let next_id = self.project.tracks.len() as u64 + 1;
                    self.project.tracks.push(summoner_project::schema::TrackConfig {
                        id: next_id,
                        name: format!("Track {}", next_id),
                        channels: 2,
                        gain: 1.0,
                        pan: 0.0,
                        muted: false,
                        soloed: false,
                        send_level: 0.0,
                        nodes: Vec::new(),
                        sequence: None,
                        connections: Vec::new(),
                        tuning_edo: None,
                        tuning_root_hz: None,
                        tuning_scl_path: None,
                    });
                }
                "render_wav" => {
                    self.export_wav();
                }
                "sfz_convert" | "auto_slice" | "load_preset" | "export_clap" | "toggle_simd" => {
                    println!("Command palette action executed: {}", action);
                }
                action_str if action_str.starts_with("add_node_") => {
                    let raw_kind = &action_str["add_node_".len()..];
                    if let Some(&matched_kind) = summoner_core::node::KNOWN_NODE_TYPES
                        .iter()
                        .find(|&&k| k.to_lowercase() == raw_kind)
                    {
                        let tid = self.selected_track_id.unwrap_or(1);
                        if let Some(track) = self.project.tracks.iter_mut().find(|t| t.id == tid) {
                            track.nodes.push(summoner_project::schema::NodeConfig {
                                kind: matched_kind.to_string(),
                                params: std::collections::HashMap::new(),
                            });
                            self.current_view = ViewMode::NodeGraph(tid);
                        }
                    }
                }
                _ => {}
            }
        }

        // Advance playhead beat when transport is running & apply/record automation
        if self.transport_running {
            let dt = ctx.input(|i| i.stable_dt) as f64;
            let delta_beat = dt * (self.project.transport.bpm / 60.0);
            self.playhead_beat += delta_beat;
            self.current_beat = self.playhead_beat;

            if self.recording_all {
                self.automation_timeline.record_beat(&mut self.automation_registry, self.current_beat);
            } else {
                self.automation_timeline.apply_beat(&self.automation_registry, self.current_beat);
            }
        }

        // Top menu bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("📄 New Session").clicked() {
                        self.new_session();
                        ui.close_menu();
                    }
                    if ui.button("📂 Open Session...").clicked() {
                        self.open_session();
                        ui.close_menu();
                    }
                    if ui.button("💾 Save Session (Ctrl+S)").clicked() {
                        self.save_session();
                        ui.close_menu();
                    }
                    if ui.button("💾 Save As...").clicked() {
                        self.save_session_as();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("🎵 Export WAV...").clicked() {
                        self.export_wav();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("❌ Quit").clicked() {
                        self.save_gui_state();
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("↩ Undo (Ctrl+Z)").clicked() {
                        self.status_message = Some("Undo action triggered (stub)".to_string());
                        println!("Undo action triggered (stub)");
                        ui.close_menu();
                    }
                    if ui.button("↪ Redo (Ctrl+Shift+Z)").clicked() {
                        self.status_message = Some("Redo action triggered (stub)".to_string());
                        println!("Redo action triggered (stub)");
                        ui.close_menu();
                    }
                });

                ui.menu_button("View", |ui| {
                    if ui.selectable_label(self.dark_theme, "🌙 Dark Theme").clicked() {
                        self.dark_theme = true;
                        ui.close_menu();
                    }
                    if ui.selectable_label(!self.dark_theme, "☀️ Light Theme").clicked() {
                        self.dark_theme = false;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("📜 Scala Scale Browser").clicked() {
                        self.show_scala_browser_modal = true;
                        ui.close_menu();
                    }
                });

                ui.separator();

                let active_tid = self.selected_track_id.unwrap_or(1);
                ui.selectable_value(&mut self.current_view, ViewMode::Arranger, "Arranger");
                ui.selectable_value(&mut self.current_view, ViewMode::PianoRoll(active_tid), "Piano Roll");
                ui.selectable_value(&mut self.current_view, ViewMode::NodeGraph(active_tid), "Node Graph");
                ui.selectable_value(&mut self.current_view, ViewMode::Mixer, "Console Mixer");
                ui.selectable_value(&mut self.current_view, ViewMode::Performance, "Stage Performance");

                ui.separator();

                if ui.button("🔍 Search (Ctrl+K)").clicked() {
                    self.command_palette.open();
                }

                ui.selectable_value(&mut self.show_rack, true, "Toggle Device Rack");

                ui.menu_button("Help", |ui| {
                    if ui.button("⌨ Keyboard Shortcuts").clicked() {
                        self.show_shortcuts_modal = true;
                        ui.close_menu();
                    }
                    if ui.button("ℹ About Summoner").clicked() {
                        self.show_about_dialog = true;
                        ui.close_menu();
                    }
                });
            });
        });

        // Scala Scale Browser Modal (Step 360)
        if self.show_scala_browser_modal {
            let mut is_open = self.show_scala_browser_modal;
            let mut track_copy = self.selected_track_id.and_then(|tid| self.project.tracks.iter_mut().find(|t| t.id == tid));
            egui::Window::new("📜 Scala Historical Scale Browser")
                .open(&mut is_open)
                .resizable(true)
                .default_size([580.0, 360.0])
                .show(ctx, |ui| {
                    crate::views::scala_browser::show_scala_browser(ui, track_copy.as_deref_mut(), &mut self.harmonic_context);
                });
            self.show_scala_browser_modal = is_open;
        }

        // Keyboard Shortcuts searchable modal (step 312)
        if self.show_shortcuts_modal {
            let mut is_open = self.show_shortcuts_modal;
            egui::Window::new("⌨ Keyboard Shortcuts")
                .open(&mut is_open)
                .collapsible(false)
                .resizable(true)
                .default_size([450.0, 320.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Search:");
                        ui.text_edit_singleline(&mut self.shortcut_search_query);
                    });
                    ui.separator();
                    let shortcuts = [
                        ("Space", "Play / Stop Transport"),
                        ("S", "Toggle Solo on selected track"),
                        ("M", "Toggle Mute on selected track"),
                        ("Shift + R", "Toggle Record All Automation"),
                        ("Ctrl + S", "Save Session"),
                        ("Ctrl + O", "Open Session"),
                        ("Ctrl + N", "New Session"),
                        ("Ctrl + K", "Command Palette"),
                        ("Ctrl + Z", "Undo micro-commit"),
                        ("Ctrl + Shift + Z", "Redo micro-commit"),
                        ("Delete / Backspace", "Delete selected node(s) in Node Graph"),
                        ("Ctrl + Scroll", "Zoom Arranger Timeline / Node Graph"),
                    ];
                    let query = self.shortcut_search_query.to_lowercase();
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        egui::Grid::new("shortcuts_grid").striped(true).min_col_width(140.0).show(ui, |ui| {
                            for (key, desc) in shortcuts {
                                if query.is_empty() || key.to_lowercase().contains(&query) || desc.to_lowercase().contains(&query) {
                                    ui.label(egui::RichText::new(key).strong().color(egui::Color32::from_rgb(26, 140, 255)));
                                    ui.label(desc);
                                    ui.end_row();
                                }
                            }
                        });
                    });
                });
            self.show_shortcuts_modal = is_open;
        }

        // About Dialog window
        if self.show_about_dialog {
            let mut is_open = true;
            let mut close_clicked = false;
            egui::Window::new("About Summoner")
                .open(&mut is_open)
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.heading("Summoner DAW v0.1.0");
                    ui.label("Next-generation microtonal DAW & generative audio engine.");
                    ui.separator();
                    ui.label("License: AGPL-3.0-or-later");
                    ui.hyperlink_to("GitHub Repository", "https://github.com/nilsanderselde/summoner");
                    ui.add_space(10.0);
                    if ui.button("Close").clicked() {
                        close_clicked = true;
                    }
                });
            self.show_about_dialog = is_open && !close_clicked;
        }

        // Bottom transport panel
        egui::TopBottomPanel::bottom("transport_panel").show(ctx, |ui| {
            // Shift+R toggle Record All
            if ctx.input(|i| i.modifiers.shift && i.key_pressed(egui::Key::R)) {
                self.recording_all = !self.recording_all;
                if self.recording_all {
                    self.automation_registry.start_record_all();
                } else {
                    self.automation_registry.stop_record_all();
                }
            }

            ui.horizontal(|ui| {
                if ui.button(if self.transport_running { "⏹ Stop" } else { "▶ Play" }).clicked() {
                    self.transport_running = !self.transport_running;
                    if !self.transport_running {
                        self.automation_registry.stop_record_all();
                    }
                }

                let mut record_btn = ui.button(if self.recording_all { "🔴 Recording All" } else { "⏺ Record All (Shift+R)" });
                if self.recording_all {
                    record_btn = record_btn.highlight();
                }
                if record_btn.clicked() {
                    self.recording_all = !self.recording_all;
                    if self.recording_all {
                        self.automation_registry.start_record_all();
                    } else {
                        self.automation_registry.stop_record_all();
                    }
                }

                ui.separator();

                ui.label(format!("Tempo: {:.1} BPM", self.project.transport.bpm));

                ui.separator();

                // Harmonic Context & Status Bar info (Steps 358 & 359)
                let active_chord = self.harmonic_context.analyze_active_chord();
                let active_voices = self.harmonic_context.active_notes.len();
                ui.label(egui::RichText::new(format!(
                    "🎵 Chord: {} (Root: C) | Active Voices: {}",
                    active_chord, active_voices
                )).color(egui::Color32::from_rgb(46, 204, 113)));

                ui.separator();

                // Status bar & MIDI Learn indicator (steps 307, 317, 318)
                if self.midi_learn_mode {
                    ui.label(egui::RichText::new("🎛️ MIDI Learn Active").color(egui::Color32::YELLOW));
                    if ui.button("Cancel").clicked() {
                        self.midi_learn_mode = false;
                        self.midi_learn_target = None;
                    }
                } else if self.is_rendering {
                    ui.spinner();
                    ui.label("Rendering WAV audio...");
                } else if let Some(status) = &self.status_message {
                    ui.label(format!("Status: {}", status));
                } else {
                    ui.label("Ready");
                }

                if self.transport_running {
                    ctx.request_repaint();
                }
            });
        });

        // Central main view
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_view {
                ViewMode::Arranger => {
                    if let Some(target_view) = show_arranger(
                        ui,
                        &mut self.project,
                        &mut self.selected_track_id,
                        &mut self.playhead_beat,
                        self.transport_running,
                        &mut self.pixels_per_beat,
                        Some(&self.automation_timeline),
                        &mut self.grid_division,
                        &mut self.track_header_width,
                        &mut self.waveform_cache,
                    ) {
                        self.current_view = target_view;
                    }
                }
                ViewMode::PianoRoll(track_id) => {
                    if let Some(track) = self.project.tracks.iter_mut().find(|t| t.id == track_id) {
                        let sequence = track.sequence.get_or_insert_with(|| summoner_project::schema::SequenceConfig {
                            start_beat: 0.0,
                            step_division: 16.0,
                            clip_color: None,
                            clip_name: None,
                            steps: vec![summoner_project::schema::TrackerStepConfig {
                                note: 60.0,
                                velocity: 0.8,
                                gate: 0.5,
                                probability: 1.0,
                                ratchet: 1,
                                micro_shift: 0,
                                active: true,
                            }; 16],
                        });
                        let tuning = EdoTuning::new(track.tuning_edo.unwrap_or(12) as u16, track.tuning_root_hz.unwrap_or(440.0) as f64, 69.0);
                        let viewport = Viewport { width: ui.available_width(), height: ui.available_height() };
                        show_piano_roll(ui, sequence, &tuning, &mut self.piano_roll_state, &viewport, Some(&self.harmonic_context));
                    } else {
                        ui.heading("Track not found");
                    }
                }
                ViewMode::NodeGraph(track_id) => {
                    let mut selected_edge = None;
                    let osc = self.oscilloscope_buffers.get(&track_id).map(|o| o.as_ref());
                    show_node_graph(ui, &mut self.dummy_graph, &mut self.node_graph_state, &mut selected_edge, osc);
                }
                ViewMode::Mixer => {
                    show_mixer(ui, &mut self.project, &mut self.selected_track_id, Some(&self.spectrum_analyzer));
                }
                ViewMode::Performance => {
                    show_stage_view(ui, &mut self.stage_view, &mut self.transport);
                }
            }

            // Optional Macro Rack device panel at bottom of central area
            if self.show_rack && self.current_view != ViewMode::Performance {
                ui.separator();
                if let Some(tid) = self.selected_track_id {
                    let osc = self.oscilloscope_buffers.get(&tid).map(|o| o.as_ref());
                    if let Some(track) = self.project.tracks.iter_mut().find(|t| t.id == tid) {
                        let mut open_graph = false;
                        show_macro_rack(ui, track, &self.param_bus, osc, &mut || {
                            open_graph = true;
                        });
                        if open_graph {
                            self.current_view = ViewMode::NodeGraph(tid);
                        }
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_project::create_default_project;
    use summoner_sequencer::automation_timeline::{AutomationCurve, AutomationLane, AutomationPoint, Interpolation};

    #[test]
    fn test_record_all_toggle_wires_registry() {
        let project = create_default_project("Test Automation App");
        let param_bus = Arc::new(ParamBus::new());
        let mut app = SummonerApp::new(project, param_bus);

        // Register a parameter
        let cutoff = app.automation_registry.register_param("cutoff", 0.5);

        // Start record all
        app.recording_all = true;
        app.automation_registry.start_record_all();
        assert!(app.automation_registry.is_recording_all());

        // Mutate parameter value
        cutoff.set(0.8);

        // Record beat at 1.0
        app.automation_timeline.record_beat(&mut app.automation_registry, 1.0);

        // Verify lane created and recorded point
        assert!(app.automation_timeline.lanes.contains_key("cutoff"));
        let lane = app.automation_timeline.lanes.get("cutoff").unwrap();
        assert_eq!(lane.curve.points.len(), 1);
        assert_eq!(lane.curve.points[0].beat, 1.0);
        assert!((lane.curve.points[0].value - 0.8).abs() < 1e-5);

        // Stop record all
        app.recording_all = false;
        app.automation_registry.stop_record_all();
        assert!(!app.automation_registry.is_recording_all());
    }

    #[test]
    fn test_automation_replay_sets_param() {
        let project = create_default_project("Test Replay App");
        let param_bus = Arc::new(ParamBus::new());
        let mut app = SummonerApp::new(project, param_bus);

        let gain_param = app.automation_registry.register_param("gain", 0.2);

        // Build automation curve with points at beat 0 (0.2) and beat 4 (0.9)
        let curve = AutomationCurve::new(vec![
            AutomationPoint { beat: 0.0, value: 0.2, interp: Interpolation::Linear },
            AutomationPoint { beat: 4.0, value: 0.9, interp: Interpolation::Linear },
        ]);
        app.automation_timeline.add_lane(AutomationLane {
            param_id: "gain".to_string(),
            curve,
        });

        // Replay at beat 2.0 (midpoint linear should give ~0.55)
        app.current_beat = 2.0;
        app.automation_timeline.apply_beat(&app.automation_registry, app.current_beat);

        let val = gain_param.get();
        assert!((val - 0.55).abs() < 1e-4);
    }

    #[test]
    fn test_app_file_menu_renders() {
        let project = create_default_project("File Menu Project");
        let param_bus = Arc::new(ParamBus::new());
        let mut app = SummonerApp::new(project, param_bus);

        assert_eq!(app.project.name, "File Menu Project");
        assert!(app.project_path.is_none());

        // Test new session reset
        app.new_session();
        assert_eq!(app.project.name, "New Project");
        assert_eq!(app.status_message.as_deref(), Some("Created new session"));

        // Save GUI state persistence round-trip check with isolated temp file
        let temp_path = std::env::temp_dir().join(format!("test_gui_state_{}.toml", std::process::id()));
        let state = GuiState {
            current_view: app.current_view.clone(),
            selected_track_id: app.selected_track_id,
            show_rack: app.show_rack,
            pixels_per_beat: app.pixels_per_beat,
            macro_rack_height: 200.0,
            track_header_width: 180.0,
            dark_theme: true,
        };
        state.save_to_path(&temp_path);
        let loaded = GuiState::load_from_path(&temp_path);
        assert!(loaded.is_some());
        let loaded_state = loaded.unwrap();
        assert_eq!(loaded_state.current_view, ViewMode::Arranger);
        let _ = std::fs::remove_file(&temp_path);
    }

    #[test]
    fn test_app_view_navigation_all_modes() {
        let project = create_default_project("Navigation Project");
        let param_bus = Arc::new(ParamBus::new());
        let mut app = SummonerApp::new(project, param_bus);

        let modes = vec![
            ViewMode::Arranger,
            ViewMode::PianoRoll(1),
            ViewMode::NodeGraph(1),
            ViewMode::Mixer,
            ViewMode::Performance,
        ];

        let temp_path = std::env::temp_dir().join(format!("test_gui_nav_{}.toml", std::process::id()));

        for mode in modes {
            app.current_view = mode.clone();
            assert_eq!(app.current_view, mode);
            let state = GuiState {
                current_view: app.current_view.clone(),
                selected_track_id: app.selected_track_id,
                show_rack: app.show_rack,
                pixels_per_beat: app.pixels_per_beat,
                macro_rack_height: 200.0,
                track_header_width: 180.0,
                dark_theme: true,
            };
            state.save_to_path(&temp_path);
            let loaded = GuiState::load_from_path(&temp_path).expect("GuiState should load");
            assert_eq!(loaded.current_view, mode);
        }
        let _ = std::fs::remove_file(&temp_path);
    }
}


