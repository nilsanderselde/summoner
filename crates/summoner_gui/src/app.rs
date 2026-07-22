use std::sync::Arc;
use summoner_core::param_bus::ParamBus;
use summoner_project::schema::ProjectConfig;
use eframe::egui;

use crate::views::arranger::show_arranger;
use crate::views::node_graph::{show_node_graph, NodeGraphState};
use summoner_core::graph::NodeGraph;

#[derive(PartialEq)]
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
}

impl SummonerApp {
    pub fn new(project: ProjectConfig, param_bus: Arc<ParamBus>) -> Self {
        Self {
            project,
            param_bus,
            transport_running: false,
            current_view: ViewMode::Arranger,
            node_graph_state: NodeGraphState::default(),
            dummy_graph: NodeGraph::new("Dummy Track", 64, 2),
            recording_all: false,
        }
    }
}

impl eframe::App for SummonerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                
                ui.separator();
                
                ui.selectable_value(&mut self.current_view, ViewMode::Arranger, "Arranger");
                ui.selectable_value(&mut self.current_view, ViewMode::Mixer, "Mixer");
                ui.selectable_value(&mut self.current_view, ViewMode::Performance, "Performance");
            });
        });

        egui::TopBottomPanel::bottom("transport_panel").show(ctx, |ui| {
            // Handle Shift+R hotkey
            if ctx.input(|i| i.modifiers.shift && i.key_pressed(egui::Key::R)) {
                self.recording_all = !self.recording_all;
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
                }
                
                ui.label(format!("Tempo: {} BPM", self.project.transport.bpm));
            });
        });

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
                    ui.heading("Mixer View");
                }
                ViewMode::Performance => {
                    ui.heading("Performance View");
                }
            }
        });
    }
}
