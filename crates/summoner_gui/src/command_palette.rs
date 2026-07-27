// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

#[cfg(feature = "gui")]
use eframe::egui;

#[derive(Debug, Clone)]
pub struct CommandAction {
    pub label: String,
    pub category: String,
    pub action_id: String,
}

/// Command Palette for rapid keyboard-first navigation (Ctrl+K / Cmd+K).
pub struct CommandPalette {
    pub is_open: bool,
    pub search_query: String,
    pub actions: Vec<CommandAction>,
    pub selected_index: usize,
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandPalette {
    pub fn new() -> Self {
        Self {
            is_open: false,
            search_query: String::new(),
            actions: vec![
                CommandAction { label: "Switch to Arranger View".into(), category: "Navigation".into(), action_id: "nav_arranger".into() },
                CommandAction { label: "Switch to Console Mixer".into(), category: "Navigation".into(), action_id: "nav_mixer".into() },
                CommandAction { label: "Switch to Live Stage Performance".into(), category: "Navigation".into(), action_id: "nav_performance".into() },
                CommandAction { label: "Open Selected Track Node Graph".into(), category: "Navigation".into(), action_id: "nav_nodegraph".into() },
                CommandAction { label: "Toggle Play / Stop Transport".into(), category: "Transport".into(), action_id: "transport_play".into() },
                CommandAction { label: "Toggle Record All Automation".into(), category: "Transport".into(), action_id: "transport_record".into() },
                CommandAction { label: "PANIC - Stop All Audio Immediately".into(), category: "Control".into(), action_id: "panic".into() },
                CommandAction { label: "Add Audio Track".into(), category: "Session".into(), action_id: "add_track".into() },
                CommandAction { label: "Add OscSine Generator Node".into(), category: "DSP".into(), action_id: "add_node_sine".into() },
                CommandAction { label: "Add FilterLadder DSP Node".into(), category: "DSP".into(), action_id: "add_node_ladder".into() },
            ],
            selected_index: 0,
        }
    }

    pub fn open(&mut self) {
        self.is_open = true;
        self.search_query.clear();
        self.selected_index = 0;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Option<String> {
        let mut executed_action = None;

        if !self.is_open {
            return None;
        }

        egui::Window::new("Command Palette (Ctrl+K)")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 100.0))
            .fixed_size(egui::vec2(500.0, 300.0))
            .show(ctx, |ui| {
                let text_edit = ui.add(
                    egui::TextEdit::singleline(&mut self.search_query)
                        .hint_text("Type a command or search node kinds...")
                        .desired_width(f32::INFINITY),
                );
                text_edit.request_focus();

                ui.separator();

                let query_lower = self.search_query.to_lowercase();
                let filtered: Vec<&CommandAction> = self
                    .actions
                    .iter()
                    .filter(|a| query_lower.is_empty() || a.label.to_lowercase().contains(&query_lower) || a.category.to_lowercase().contains(&query_lower))
                    .collect();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (idx, action) in filtered.iter().enumerate() {
                        let is_sel = idx == self.selected_index;
                        let text = format!("[{}] {}", action.category, action.label);
                        if ui.selectable_label(is_sel, text).clicked() {
                            executed_action = Some(action.action_id.clone());
                            self.is_open = false;
                        }
                    }
                });

                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.is_open = false;
                }
            });

        executed_action
    }
}
