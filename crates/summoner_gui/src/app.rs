use std::sync::Arc;
use summoner_core::param_bus::ParamBus;
use summoner_project::schema::ProjectConfig;
use eframe::egui;

use crate::views::arranger::show_arranger;
use crate::views::node_graph::{show_node_graph, NodeGraphState};
use crate::views::mixer::show_mixer;
use crate::views::macro_rack::show_macro_rack;
use crate::stage_view::{show_stage_view, StageView};
use crate::command_palette::CommandPalette;
use summoner_core::graph::NodeGraph;
use summoner_core::transport::Transport;
use summoner_sequencer::automation::AutomationRegistry;
use summoner_sequencer::automation_timeline::AutomationTimeline;

#[derive(PartialEq, Debug, Clone)]
pub enum ViewMode {
    Arranger,
    NodeGraph(u64),
    Mixer,
    Performance,
}

pub struct SummonerApp {
    pub project: ProjectConfig,
    pub param_bus: Arc<ParamBus>,
    pub transport_running: bool,
    pub current_view: ViewMode,
    pub node_graph_state: NodeGraphState,
    pub dummy_graph: NodeGraph,
    pub recording_all: bool,
    pub selected_track_id: Option<u64>,
    pub stage_view: StageView,
    pub command_palette: CommandPalette,
    pub automation_registry: AutomationRegistry,
    pub automation_timeline: AutomationTimeline,
    pub transport: Transport,
    pub show_rack: bool,
}

impl SummonerApp {
    pub fn new(project: ProjectConfig, param_bus: Arc<ParamBus>) -> Self {
        let sample_rate = project.transport.sample_rate;
        let bpm = project.transport.bpm;
        Self {
            project,
            param_bus,
            transport_running: false,
            current_view: ViewMode::Arranger,
            node_graph_state: NodeGraphState::default(),
            dummy_graph: NodeGraph::new("Main Track", 64, 2),
            recording_all: false,
            selected_track_id: Some(1),
            stage_view: StageView::new(),
            command_palette: CommandPalette::new(),
            automation_registry: AutomationRegistry::new(),
            automation_timeline: AutomationTimeline::new(),
            transport: Transport::new(sample_rate, bpm),
            show_rack: true,
        }
    }
}

impl eframe::App for SummonerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Ctrl+K / Cmd+K Command Palette hotkey
        if ctx.input(|i| (i.modifiers.command || i.modifiers.ctrl) && i.key_pressed(egui::Key::K)) {
            self.command_palette.open();
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
                "transport_play" => self.transport_running = !self.transport_running,
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
                    self.stage_view.trigger_panic();
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
                        nodes: Vec::new(),
                        sequence: None,
                        connections: Vec::new(),
                        tuning_edo: None,
                        tuning_root_hz: None,
                        tuning_scl_path: None,
                    });
                }
                _ => {}
            }
        }

        // Top menu bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.separator();

                ui.selectable_value(&mut self.current_view, ViewMode::Arranger, "Arranger");
                ui.selectable_value(&mut self.current_view, ViewMode::Mixer, "Console Mixer");
                ui.selectable_value(&mut self.current_view, ViewMode::Performance, "Stage Performance");

                ui.separator();

                if ui.button("🔍 Search (Ctrl+K)").clicked() {
                    self.command_palette.open();
                }

                ui.selectable_value(&mut self.show_rack, true, "Toggle Device Rack");
            });
        });

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

                if self.transport_running {
                    ctx.request_repaint();
                }
            });
        });

        // Central main view
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_view {
                ViewMode::Arranger => {
                    show_arranger(ui, &mut self.project);
                }
                ViewMode::NodeGraph(_track_id) => {
                    let mut selected_edge = None;
                    show_node_graph(ui, &mut self.dummy_graph, &mut self.node_graph_state, &mut selected_edge);
                }
                ViewMode::Mixer => {
                    show_mixer(ui, &mut self.project, &mut self.selected_track_id);
                }
                ViewMode::Performance => {
                    show_stage_view(ui, &mut self.stage_view, &mut self.transport);
                }
            }

            // Optional Macro Rack device panel at bottom of central area
            if self.show_rack && self.current_view != ViewMode::Performance {
                ui.separator();
                if let Some(tid) = self.selected_track_id {
                    if let Some(track) = self.project.tracks.iter().find(|t| t.id == tid) {
                        let track_clone = track.clone();
                        let mut open_graph = false;
                        show_macro_rack(ui, &track_clone, &self.param_bus, &mut || {
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
