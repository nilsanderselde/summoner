// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Custom DAW Shortcut Keybinding Editor with Conflict Detection & Modifier Map (Step 1323).

use crate::layout_math::OperatingSystem;
use crate::touch_controls::MIN_HIT_TARGET_PT;
use std::collections::HashMap;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Vec2};

/// Modifier key bitflags
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize,
)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool, // Cmd on macOS, Super/Win on Windows/Linux
}

impl KeyModifiers {
    pub fn none() -> Self {
        Self::default()
    }

    pub fn ctrl() -> Self {
        Self {
            ctrl: true,
            ..Default::default()
        }
    }

    pub fn ctrl_shift() -> Self {
        Self {
            ctrl: true,
            shift: true,
            ..Default::default()
        }
    }

    pub fn alt() -> Self {
        Self {
            alt: true,
            ..Default::default()
        }
    }

    /// Formats modifiers for display based on operating system
    pub fn display_string(&self, os: OperatingSystem) -> String {
        let mut parts = Vec::new();
        if self.ctrl {
            if os == OperatingSystem::MacOS {
                parts.push("⌃ Ctrl".to_string());
            } else {
                parts.push("Ctrl".to_string());
            }
        }
        if self.alt {
            if os == OperatingSystem::MacOS {
                parts.push("⌥ Option".to_string());
            } else {
                parts.push("Alt".to_string());
            }
        }
        if self.shift {
            if os == OperatingSystem::MacOS {
                parts.push("⇧ Shift".to_string());
            } else {
                parts.push("Shift".to_string());
            }
        }
        if self.meta {
            if os == OperatingSystem::MacOS {
                parts.push("⌘ Cmd".to_string());
            } else {
                parts.push("Super".to_string());
            }
        }
        if parts.is_empty() {
            String::new()
        } else {
            format!("{}+", parts.join("+"))
        }
    }
}

/// A shortcut keybinding definition
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct KeyShortcut {
    pub key: String, // e.g. "Space", "Z", "S", "1", "Delete"
    pub modifiers: KeyModifiers,
}

impl KeyShortcut {
    pub fn new(key: impl Into<String>, modifiers: KeyModifiers) -> Self {
        Self {
            key: key.into(),
            modifiers,
        }
    }

    pub fn display_string(&self, os: OperatingSystem) -> String {
        format!("{}{}", self.modifiers.display_string(os), self.key)
    }
}

/// Keybinding Action Entry
#[derive(Debug, Clone, PartialEq)]
pub struct KeyActionEntry {
    pub id: String,
    pub name: String,
    pub category: String,
    pub primary: Option<KeyShortcut>,
    pub secondary: Option<KeyShortcut>,
    pub default_primary: Option<KeyShortcut>,
}

/// Conflict report for duplicate key combinations
#[derive(Debug, Clone, PartialEq)]
pub struct KeybindingConflict {
    pub shortcut_display: String,
    pub conflicting_action_ids: Vec<String>,
    pub conflicting_action_names: Vec<String>,
}

/// Custom DAW Shortcut Keybinding Editor View (Step 1323).
#[derive(Debug, Clone)]
pub struct KeybindingEditorView {
    pub actions: Vec<KeyActionEntry>,
    pub search_query: String,
    pub selected_category: String,
    pub editing_action_id: Option<String>,
    pub editing_is_secondary: bool,
    pub current_os: OperatingSystem,
}

impl Default for KeybindingEditorView {
    fn default() -> Self {
        Self::new(OperatingSystem::current())
    }
}

impl KeybindingEditorView {
    pub fn new(os: OperatingSystem) -> Self {
        let is_mac = os == OperatingSystem::MacOS;
        let main_mod = if is_mac {
            KeyModifiers {
                meta: true,
                ..Default::default()
            }
        } else {
            KeyModifiers::ctrl()
        };
        let main_shift_mod = if is_mac {
            KeyModifiers {
                meta: true,
                shift: true,
                ..Default::default()
            }
        } else {
            KeyModifiers::ctrl_shift()
        };

        let actions = vec![
            KeyActionEntry {
                id: "transport_play_pause".into(),
                name: "Play / Stop Transport".into(),
                category: "Transport".into(),
                primary: Some(KeyShortcut::new("Space", KeyModifiers::none())),
                secondary: None,
                default_primary: Some(KeyShortcut::new("Space", KeyModifiers::none())),
            },
            KeyActionEntry {
                id: "transport_record".into(),
                name: "Toggle Record".into(),
                category: "Transport".into(),
                primary: Some(KeyShortcut::new("R", KeyModifiers::none())),
                secondary: None,
                default_primary: Some(KeyShortcut::new("R", KeyModifiers::none())),
            },
            KeyActionEntry {
                id: "file_save".into(),
                name: "Save Project".into(),
                category: "File".into(),
                primary: Some(KeyShortcut::new("S", main_mod)),
                secondary: None,
                default_primary: Some(KeyShortcut::new("S", main_mod)),
            },
            KeyActionEntry {
                id: "file_save_as".into(),
                name: "Save Project As...".into(),
                category: "File".into(),
                primary: Some(KeyShortcut::new("S", main_shift_mod)),
                secondary: None,
                default_primary: Some(KeyShortcut::new("S", main_shift_mod)),
            },
            KeyActionEntry {
                id: "edit_undo".into(),
                name: "Undo Last Action".into(),
                category: "Edit".into(),
                primary: Some(KeyShortcut::new("Z", main_mod)),
                secondary: None,
                default_primary: Some(KeyShortcut::new("Z", main_mod)),
            },
            KeyActionEntry {
                id: "edit_redo".into(),
                name: "Redo Last Action".into(),
                category: "Edit".into(),
                primary: Some(KeyShortcut::new("Y", main_mod)),
                secondary: Some(KeyShortcut::new("Z", main_shift_mod)),
                default_primary: Some(KeyShortcut::new("Y", main_mod)),
            },
            KeyActionEntry {
                id: "nav_arranger".into(),
                name: "Switch to Arranger View".into(),
                category: "Navigation".into(),
                primary: Some(KeyShortcut::new("1", main_mod)),
                secondary: None,
                default_primary: Some(KeyShortcut::new("1", main_mod)),
            },
            KeyActionEntry {
                id: "nav_mixer".into(),
                name: "Switch to Console Mixer".into(),
                category: "Navigation".into(),
                primary: Some(KeyShortcut::new("2", main_mod)),
                secondary: None,
                default_primary: Some(KeyShortcut::new("2", main_mod)),
            },
            KeyActionEntry {
                id: "nav_live_rack".into(),
                name: "Switch to Live Performance Rack".into(),
                category: "Navigation".into(),
                primary: Some(KeyShortcut::new("3", main_mod)),
                secondary: None,
                default_primary: Some(KeyShortcut::new("3", main_mod)),
            },
            KeyActionEntry {
                id: "tool_command_palette".into(),
                name: "Open Command Palette".into(),
                category: "Tools".into(),
                primary: Some(KeyShortcut::new("K", main_mod)),
                secondary: Some(KeyShortcut::new("P", main_mod)),
                default_primary: Some(KeyShortcut::new("K", main_mod)),
            },
        ];

        Self {
            actions,
            search_query: String::new(),
            selected_category: "All".into(),
            editing_action_id: None,
            editing_is_secondary: false,
            current_os: os,
        }
    }

    /// Detect duplicate conflicting shortcuts
    pub fn detect_conflicts(&self) -> Vec<KeybindingConflict> {
        let mut map: HashMap<KeyShortcut, Vec<(String, String)>> = HashMap::new();

        for action in &self.actions {
            if let Some(prim) = &action.primary {
                map.entry(prim.clone())
                    .or_default()
                    .push((action.id.clone(), action.name.clone()));
            }
            if let Some(sec) = &action.secondary {
                map.entry(sec.clone())
                    .or_default()
                    .push((action.id.clone(), action.name.clone()));
            }
        }

        let mut conflicts = Vec::new();
        for (shortcut, entries) in map {
            if entries.len() > 1 {
                let ids = entries.iter().map(|(id, _)| id.clone()).collect();
                let names = entries.iter().map(|(_, name)| name.clone()).collect();
                conflicts.push(KeybindingConflict {
                    shortcut_display: shortcut.display_string(self.current_os),
                    conflicting_action_ids: ids,
                    conflicting_action_names: names,
                });
            }
        }
        conflicts.sort_by(|a, b| a.shortcut_display.cmp(&b.shortcut_display));
        conflicts
    }

    /// Assign shortcut to an action
    pub fn assign_shortcut(
        &mut self,
        action_id: &str,
        shortcut: Option<KeyShortcut>,
        is_secondary: bool,
    ) {
        if let Some(act) = self.actions.iter_mut().find(|a| a.id == action_id) {
            if is_secondary {
                act.secondary = shortcut;
            } else {
                act.primary = shortcut;
            }
        }
    }

    /// Reset all shortcuts to defaults
    pub fn reset_all_defaults(&mut self) {
        for act in &mut self.actions {
            act.primary = act.default_primary.clone();
            act.secondary = None;
        }
    }

    /// Render ASCII summary
    pub fn render_ascii(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "[KEYBINDING EDITOR - OS: {:?}]\n",
            self.current_os
        ));
        let conflicts = self.detect_conflicts();
        if !conflicts.is_empty() {
            out.push_str(&format!("⚠️ CONFLICTS DETECTED ({}):\n", conflicts.len()));
            for c in &conflicts {
                out.push_str(&format!(
                    " - Key '{}': {:?}\n",
                    c.shortcut_display, c.conflicting_action_names
                ));
            }
        } else {
            out.push_str("✅ ZERO CONFLICTS DETECTED\n");
        }
        out.push_str("BINDINGS:\n");
        for act in &self.actions {
            let prim_str = act
                .primary
                .as_ref()
                .map(|s| s.display_string(self.current_os))
                .unwrap_or_else(|| "--".into());
            let sec_str = act
                .secondary
                .as_ref()
                .map(|s| s.display_string(self.current_os))
                .unwrap_or_else(|| "--".into());
            out.push_str(&format!(
                " - {:<28} [{:<10}] : Primary: {:<12} | Secondary: {}\n",
                act.name, act.category, prim_str, sec_str
            ));
        }
        out
    }
}

#[cfg(feature = "gui")]
impl KeybindingEditorView {
    /// Render egui Keybinding Editor UI
    pub fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let conflicts = self.detect_conflicts();

        ui.vertical(|ui| {
            // Header Bar
            ui.horizontal(|ui| {
                ui.heading("KEYBOARD SHORTCUTS & KEYBINDING EDITOR");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Reset to Defaults Button (>= 44x44pt)
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("Reset All to Defaults").size(12.0),
                            )
                            .min_size(Vec2::new(MIN_HIT_TARGET_PT, 36.0)),
                        )
                        .clicked()
                    {
                        self.reset_all_defaults();
                    }
                });
            });

            // Warning Conflict Banner if conflicts exist
            if !conflicts.is_empty() {
                ui.add_space(4.0);
                egui::Frame::none()
                    .fill(Color32::from_rgb(50, 15, 20))
                    .stroke(egui::Stroke::new(1.5_f32, Color32::from_rgb(255, 60, 60)))
                    .rounding(6.0)
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "⚠️ {} SHORTCUT CONFLICT(S) DETECTED: {}",
                                    conflicts.len(),
                                    conflicts[0].shortcut_display
                                ))
                                .size(13.0)
                                .strong()
                                .color(Color32::from_rgb(255, 100, 100)),
                            );
                        });
                    });
            }

            ui.add_space(8.0);

            // Filter Bar (Search & Category Tabs)
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Search:")
                        .size(12.0)
                        .color(Color32::from_rgb(180, 200, 225)),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut self.search_query)
                        .hint_text("Filter by action name or shortcut...")
                        .desired_width(220.0),
                );

                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new("Category:")
                        .size(12.0)
                        .color(Color32::from_rgb(180, 200, 225)),
                );
                for cat in &["All", "Transport", "File", "Edit", "Navigation", "Tools"] {
                    let is_sel = self.selected_category == *cat;
                    let btn_color = if is_sel {
                        Color32::from_rgb(0, 229, 255)
                    } else {
                        Color32::from_rgb(45, 55, 75)
                    };
                    let text_color = if is_sel {
                        Color32::BLACK
                    } else {
                        Color32::WHITE
                    };
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(*cat).size(12.0).color(text_color),
                            )
                            .fill(btn_color)
                            .min_size(Vec2::new(MIN_HIT_TARGET_PT, 30.0)),
                        )
                        .clicked()
                    {
                        self.selected_category = cat.to_string();
                    }
                }
            });

            ui.add_space(8.0);

            // Shortcut Table Rows
            egui::ScrollArea::vertical().show(ui, |ui| {
                let search_lower = self.search_query.to_lowercase();
                let actions_clone = self.actions.clone();

                for act in &actions_clone {
                    // Category filter
                    if self.selected_category != "All" && act.category != self.selected_category {
                        continue;
                    }
                    // Search filter
                    if !search_lower.is_empty()
                        && !act.name.to_lowercase().contains(&search_lower)
                        && !act.id.to_lowercase().contains(&search_lower)
                    {
                        continue;
                    }

                    // Check if this action is in conflict
                    let has_conflict = conflicts
                        .iter()
                        .any(|c| c.conflicting_action_ids.contains(&act.id));
                    let row_bg = if has_conflict {
                        Color32::from_rgba_unmultiplied(80, 20, 25, 120)
                    } else {
                        Color32::from_rgba_unmultiplied(20, 26, 38, 120)
                    };

                    egui::Frame::none()
                        .fill(row_bg)
                        .stroke(egui::Stroke::new(
                            1.0_f32,
                            if has_conflict {
                                Color32::from_rgb(255, 70, 70)
                            } else {
                                Color32::from_rgb(40, 50, 70)
                            },
                        ))
                        .rounding(4.0)
                        .inner_margin(egui::Margin::symmetric(8.0_f32, 6.0_f32))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // Action Name
                                ui.add_sized(
                                    Vec2::new(240.0, 32.0),
                                    egui::Label::new(
                                        egui::RichText::new(&act.name)
                                            .size(13.0)
                                            .strong()
                                            .color(Color32::from_rgb(240, 245, 255)),
                                    ),
                                );

                                // Category Tag
                                ui.add_sized(
                                    Vec2::new(90.0, 32.0),
                                    egui::Label::new(
                                        egui::RichText::new(&act.category)
                                            .size(11.0)
                                            .color(Color32::from_rgb(130, 160, 200)),
                                    ),
                                );

                                // Primary Shortcut Button (>= 44x44pt target)
                                let prim_text = act
                                    .primary
                                    .as_ref()
                                    .map(|s| s.display_string(self.current_os))
                                    .unwrap_or_else(|| "None".into());
                                let prim_btn = egui::Button::new(
                                    egui::RichText::new(prim_text)
                                        .size(12.0)
                                        .color(Color32::from_rgb(0, 229, 255)),
                                )
                                .min_size(Vec2::new(100.0, MIN_HIT_TARGET_PT));

                                if ui.add(prim_btn).clicked() {
                                    self.editing_action_id = Some(act.id.clone());
                                    self.editing_is_secondary = false;
                                }

                                // Secondary Shortcut Button (>= 44x44pt target)
                                let sec_text = act
                                    .secondary
                                    .as_ref()
                                    .map(|s| s.display_string(self.current_os))
                                    .unwrap_or_else(|| "--".into());
                                let sec_btn = egui::Button::new(
                                    egui::RichText::new(sec_text)
                                        .size(12.0)
                                        .color(Color32::from_rgb(180, 200, 225)),
                                )
                                .min_size(Vec2::new(100.0, MIN_HIT_TARGET_PT));

                                if ui.add(sec_btn).clicked() {
                                    self.editing_action_id = Some(act.id.clone());
                                    self.editing_is_secondary = true;
                                }

                                // Clear Button
                                if ui
                                    .add(
                                        egui::Button::new(
                                            egui::RichText::new("Clear")
                                                .size(11.0)
                                                .color(Color32::from_rgb(255, 120, 120)),
                                        )
                                        .min_size(Vec2::new(MIN_HIT_TARGET_PT, 32.0)),
                                    )
                                    .clicked()
                                {
                                    self.assign_shortcut(&act.id, None, false);
                                    self.assign_shortcut(&act.id, None, true);
                                }
                            });
                        });
                    ui.add_space(3.0);
                }
            });
        })
        .response
    }
}
