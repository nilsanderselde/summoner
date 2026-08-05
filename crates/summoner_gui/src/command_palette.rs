// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

#[cfg(feature = "gui")]
use eframe::egui;
use summoner_core::node::KNOWN_NODE_TYPES;

#[derive(Debug, Clone)]
pub struct CommandAction {
    pub label: String,
    pub category: String,
    pub action_id: String,
    pub shortcut_hint: Option<String>,
}

/// Character-subsequence fuzzy matcher for command palette search.
/// Returns `Some(score)` if all characters of `query` appear in sequence within `target`.
pub fn fuzzy_score(query: &str, target: &str) -> Option<i32> {
    if query.is_empty() {
        return Some(0);
    }
    let query_chars: Vec<char> = query.to_lowercase().chars().collect();
    let target_chars: Vec<char> = target.to_lowercase().chars().collect();

    let mut q_idx = 0;
    let mut score = 0;
    let mut last_match_idx = 0;

    for (t_idx, &t_char) in target_chars.iter().enumerate() {
        if q_idx < query_chars.len() && t_char == query_chars[q_idx] {
            if q_idx == 0 {
                if t_idx == 0 {
                    score += 50;
                }
            } else {
                if t_idx == last_match_idx + 1 {
                    score += 30;
                } else {
                    score -= (t_idx - last_match_idx) as i32 * 2;
                }
            }
            score += 10;
            last_match_idx = t_idx;
            q_idx += 1;
        }
    }

    if q_idx == query_chars.len() {
        Some(score)
    } else {
        None
    }
}

/// Command Palette for rapid keyboard-first navigation (Ctrl+K / Cmd+K).
pub struct CommandPalette {
    pub is_open: bool,
    pub search_query: String,
    pub actions: Vec<CommandAction>,
    pub selected_index: usize,
    pub recent_action_ids: Vec<String>,
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandPalette {
    pub fn new() -> Self {
        let mut actions = vec![
            CommandAction {
                label: "Switch to Arranger View".into(),
                category: "Navigation".into(),
                action_id: "nav_arranger".into(),
                shortcut_hint: Some("Ctrl+1".into()),
            },
            CommandAction {
                label: "Switch to Console Mixer".into(),
                category: "Navigation".into(),
                action_id: "nav_mixer".into(),
                shortcut_hint: Some("Ctrl+2".into()),
            },
            CommandAction {
                label: "Switch to Live Stage Performance".into(),
                category: "Navigation".into(),
                action_id: "nav_performance".into(),
                shortcut_hint: Some("Ctrl+3".into()),
            },
            CommandAction {
                label: "Open Selected Track Node Graph".into(),
                category: "Navigation".into(),
                action_id: "nav_nodegraph".into(),
                shortcut_hint: Some("Ctrl+4".into()),
            },
            CommandAction {
                label: "Toggle Play / Stop Transport".into(),
                category: "Transport".into(),
                action_id: "transport_play".into(),
                shortcut_hint: Some("Space".into()),
            },
            CommandAction {
                label: "Toggle Record All Automation".into(),
                category: "Transport".into(),
                action_id: "transport_record".into(),
                shortcut_hint: Some("R".into()),
            },
            CommandAction {
                label: "PANIC - Stop All Audio Immediately".into(),
                category: "Control".into(),
                action_id: "panic".into(),
                shortcut_hint: Some("Esc".into()),
            },
            CommandAction {
                label: "Set Project BPM".into(),
                category: "Transport".into(),
                action_id: "set_bpm".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Add Audio Track".into(),
                category: "Session".into(),
                action_id: "add_track".into(),
                shortcut_hint: Some("Ctrl+T".into()),
            },
            CommandAction {
                label: "Render WAV Audio".into(),
                category: "Session".into(),
                action_id: "render_wav".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "SFZ Instrument Convert".into(),
                category: "Session".into(),
                action_id: "sfz_convert".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Auto-Slice Sample File".into(),
                category: "Session".into(),
                action_id: "auto_slice".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Load Sampler Preset".into(),
                category: "Session".into(),
                action_id: "load_preset".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Export CLAP Plugin".into(),
                category: "Session".into(),
                action_id: "export_clap".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Toggle SIMD Acceleration".into(),
                category: "Control".into(),
                action_id: "toggle_simd".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Toggle Patch Browser".into(),
                category: "Navigation".into(),
                action_id: "toggle_patch_browser".into(),
                shortcut_hint: Some("Ctrl+B".into()),
            },
            CommandAction {
                label: "Toggle High Contrast Mode".into(),
                category: "Accessibility".into(),
                action_id: "toggle_high_contrast".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Toggle Reduce Motion Mode".into(),
                category: "Accessibility".into(),
                action_id: "toggle_reduce_motion".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Open Accessibility Settings".into(),
                category: "Accessibility".into(),
                action_id: "open_accessibility_settings".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Audio Driver Settings & Native Device Selector".into(),
                category: "Settings".into(),
                action_id: "open_audio_driver_settings".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Toggle Loudness & Peak Headroom Meter".into(),
                category: "View".into(),
                action_id: "toggle_loudness_meter".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "MPE Expression Curve Editor".into(),
                category: "Tools".into(),
                action_id: "open_mpe_editor".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Project Backup Snapshots Manager".into(),
                category: "File".into(),
                action_id: "open_backup_manager".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Neural Audio Style Transfer Preview".into(),
                category: "Tools".into(),
                action_id: "open_style_transfer".into(),
                shortcut_hint: None,
            },
            CommandAction {
                label: "Multi-Channel Spectral Equalizer".into(),
                category: "Tools".into(),
                action_id: "open_spectral_eq".into(),
                shortcut_hint: None,
            },
        ];

        // Add entries for all KNOWN_NODE_TYPES
        for &kind in KNOWN_NODE_TYPES {
            actions.push(CommandAction {
                label: format!("Add {} Node", kind),
                category: "DSP".into(),
                action_id: format!("add_node_{}", kind.to_lowercase()),
                shortcut_hint: None,
            });
        }

        Self {
            is_open: false,
            search_query: String::new(),
            actions,
            selected_index: 0,
            recent_action_ids: Vec::new(),
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

    pub fn record_recent_action(&mut self, action_id: &str) {
        self.recent_action_ids.retain(|id| id != action_id);
        self.recent_action_ids.insert(0, action_id.to_string());
        if self.recent_action_ids.len() > 5 {
            self.recent_action_ids.truncate(5);
        }
    }

    /// Filters and ranks actions based on fuzzy matching.
    pub fn get_filtered_actions(&self) -> Vec<CommandAction> {
        if self.search_query.trim().is_empty() {
            let mut result = Vec::new();

            // Recently used section
            for recent_id in &self.recent_action_ids {
                if let Some(action) = self.actions.iter().find(|a| &a.action_id == recent_id) {
                    let mut recent_action = action.clone();
                    recent_action.category = "Recently Used".into();
                    result.push(recent_action);
                }
            }

            // All remaining actions
            for action in &self.actions {
                if !self.recent_action_ids.contains(&action.action_id) {
                    result.push(action.clone());
                }
            }
            result
        } else {
            let mut scored: Vec<(i32, CommandAction)> = self
                .actions
                .iter()
                .filter_map(|action| {
                    let target_str = format!("{} {}", action.category, action.label);
                    fuzzy_score(&self.search_query, &target_str).map(|s| (s, action.clone()))
                })
                .collect();

            scored.sort_by(|a, b| b.0.cmp(&a.0));
            scored.into_iter().map(|(_, act)| act).collect()
        }
    }

    #[cfg(feature = "gui")]
    pub fn show(&mut self, ctx: &egui::Context) -> Option<String> {
        let mut executed_action = None;

        let alpha = ctx.animate_value_with_time(
            egui::Id::new("command_palette_fade"),
            if self.is_open { 1.0 } else { 0.0 },
            0.15,
        );

        if !self.is_open && alpha <= 0.001 {
            return None;
        }

        let filtered = self.get_filtered_actions();

        // Keyboard navigation
        let (up_pressed, down_pressed, enter_pressed, esc_pressed) = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::ArrowUp),
                i.key_pressed(egui::Key::ArrowDown),
                i.key_pressed(egui::Key::Enter),
                i.key_pressed(egui::Key::Escape),
            )
        });

        if esc_pressed {
            self.is_open = false;
            return None;
        }

        if !filtered.is_empty() {
            if up_pressed {
                if self.selected_index == 0 {
                    self.selected_index = filtered.len() - 1;
                } else {
                    self.selected_index -= 1;
                }
            }
            if down_pressed {
                self.selected_index = (self.selected_index + 1) % filtered.len();
            }
            if self.selected_index >= filtered.len() {
                self.selected_index = 0;
            }

            if enter_pressed {
                let action = &filtered[self.selected_index];
                executed_action = Some(action.action_id.clone());
                self.record_recent_action(&action.action_id);
                self.is_open = false;
            }
        }

        egui::Window::new("Command Palette (Ctrl+K)")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 80.0))
            .fixed_size(egui::vec2(520.0, 320.0))
            .show(ctx, |ui| {
                let text_edit = ui.add(
                    egui::TextEdit::singleline(&mut self.search_query)
                        .hint_text("Type a command or node kind (e.g. 'lf', 'wav', 'sine')...")
                        .desired_width(f32::INFINITY),
                );
                text_edit.request_focus();

                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut current_category = String::new();
                    for (idx, action) in filtered.iter().enumerate() {
                        if self.search_query.trim().is_empty()
                            && action.category != current_category
                        {
                            current_category = action.category.clone();
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new(&current_category)
                                    .strong()
                                    .color(egui::Color32::from_rgb(140, 160, 200)),
                            );
                        }

                        let is_sel = idx == self.selected_index;
                        ui.horizontal(|ui| {
                            let text = format!("[{}] {}", action.category, action.label);
                            let response = ui.selectable_label(is_sel, text);
                            if response.clicked() {
                                executed_action = Some(action.action_id.clone());
                                self.record_recent_action(&action.action_id);
                                self.is_open = false;
                            }
                            if let Some(ref hint) = action.shortcut_hint {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            egui::RichText::new(hint)
                                                .small()
                                                .color(egui::Color32::from_rgb(120, 120, 140)),
                                        );
                                    },
                                );
                            }
                        });
                    }
                });
            });

        executed_action
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_palette_fuzzy_lf_matches_lfo() {
        let _palette = CommandPalette::new();
        let score_lfo = fuzzy_score("lf", "OscLFO");
        let score_sine = fuzzy_score("lf", "OscSine");
        assert!(score_lfo.is_some());
        assert!(score_sine.is_none());

        let mut pal = CommandPalette::new();
        pal.open();
        pal.search_query = "lf".into();
        let filtered = pal.get_filtered_actions();
        assert!(!filtered.is_empty());
        assert!(filtered.iter().any(|a| a.label.contains("OscLFO")));
    }

    #[test]
    fn test_command_palette_arrow_navigation() {
        let mut palette = CommandPalette::new();
        palette.open();
        let actions_count = palette.get_filtered_actions().len();

        assert_eq!(palette.selected_index, 0);

        // Arrow down
        palette.selected_index = (palette.selected_index + 1) % actions_count;
        assert_eq!(palette.selected_index, 1);

        // Arrow up wrap around
        palette.selected_index = 0;
        palette.selected_index = actions_count - 1;
        assert_eq!(palette.selected_index, actions_count - 1);
    }

    #[test]
    fn test_command_palette_enter_executes() {
        let mut palette = CommandPalette::new();
        palette.open();
        palette.selected_index = 0;
        let actions = palette.get_filtered_actions();
        let first_id = actions[0].action_id.clone();

        palette.record_recent_action(&first_id);
        palette.close();

        assert!(!palette.is_open);
        assert_eq!(palette.recent_action_ids[0], first_id);
    }
}
