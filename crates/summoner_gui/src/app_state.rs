// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! GUI persistent state schema and lifecycle management.

use crate::app::ViewMode;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
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
    #[serde(default = "default_true")]
    pub first_run: bool,
    #[serde(default)]
    pub beginner_mode: bool,
    #[serde(default)]
    pub recent_projects: Vec<PathBuf>,
    #[serde(default = "default_auto_save_interval")]
    pub auto_save_interval_secs: u64,
    #[serde(default = "default_true")]
    pub show_tutorial_tooltips: bool,
    #[serde(default)]
    pub high_contrast_mode: bool,
    #[serde(default)]
    pub colorblind_mode: crate::views::theme_customizer::ColorblindMode,
    #[serde(default = "default_ui_scale")]
    pub ui_scale: f32,
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    #[serde(default)]
    pub reduce_motion: bool,
    #[serde(default = "default_true")]
    pub keyboard_navigation: bool,
    #[serde(default)]
    pub window_pos: Option<[f32; 2]>,
    #[serde(default)]
    pub window_size: Option<[f32; 2]>,
    #[serde(default)]
    pub multi_monitor_enabled: bool,
    #[serde(default)]
    pub detached_panels: Vec<String>,
    #[serde(default = "default_panel_dock_edge")]
    pub panel_dock_edge: String,
    #[serde(default = "default_true")]
    pub snap_to_edges: bool,
    #[serde(default)]
    pub show_project_notes_panel: bool,
    #[serde(default)]
    pub audio_driver_panel: Option<crate::visualizer::AudioDriverSelectorPanel>,
    #[serde(default)]
    pub preferences_state: Option<crate::param_controls::PreferencesState>,
}

pub fn default_panel_dock_edge() -> String {
    "Left".to_string()
}

pub fn default_macro_rack_height() -> f32 {
    200.0
}
pub fn default_track_header_width() -> f32 {
    180.0
}
pub fn default_dark_theme() -> bool {
    true
}
pub fn default_ui_scale() -> f32 {
    1.0
}
pub fn default_true() -> bool {
    true
}
pub fn default_font_size() -> f32 {
    14.0
}
pub fn default_auto_save_interval() -> u64 {
    300
}

impl GuiState {
    pub const STATE_FILE: &'static str = ".summoner_gui_state.toml";

    pub fn load_from_path(path: &Path) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        toml::from_str(&content).ok()
    }

    pub fn load() -> Option<Self> {
        Self::load_from_path(Path::new(Self::STATE_FILE))
    }

    pub fn save_to_path(&self, path: &Path) {
        if let Ok(serialized) = toml::to_string(self) {
            let _ = std::fs::write(path, serialized);
        }
    }

    pub fn save(&self) {
        self.save_to_path(Path::new(Self::STATE_FILE));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuiDisplayMode {
    SimpleMode,
    AdvancedMode,
}

