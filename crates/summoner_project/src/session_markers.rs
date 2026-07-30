//! Session Marker & Project Chapter Navigation Manager with Hotkey Bindings (Step 1246).
//!
//! Provides comprehensive session marker management, project chapter navigation,
//! hotkey binding handling, CUE sheet export, and YouTube timestamp formatting.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::schema::{MarkerConfig, ProjectConfig};

/// Chapter type categories for song / session structure.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ChapterType {
    Intro,
    Verse,
    Chorus,
    Bridge,
    Outro,
    Breakdown,
    Drop,
    Marker,
    Custom(String),
}

impl ChapterType {
    /// Returns default RGB color for each chapter type.
    pub fn default_color(&self) -> [u8; 3] {
        match self {
            ChapterType::Intro => [64, 158, 255],     // Blue
            ChapterType::Verse => [103, 194, 58],    // Green
            ChapterType::Chorus => [230, 162, 60],   // Gold / Orange
            ChapterType::Bridge => [157, 92, 255],   // Purple
            ChapterType::Outro => [245, 108, 108],   // Red
            ChapterType::Breakdown => [230, 126, 34], // Dark Orange
            ChapterType::Drop => [231, 76, 60],      // Crimson
            ChapterType::Marker => [144, 147, 153],  // Gray
            ChapterType::Custom(_) => [180, 180, 180],
        }
    }

    /// Display string label.
    pub fn label(&self) -> &str {
        match self {
            ChapterType::Intro => "Intro",
            ChapterType::Verse => "Verse",
            ChapterType::Chorus => "Chorus",
            ChapterType::Bridge => "Bridge",
            ChapterType::Outro => "Outro",
            ChapterType::Breakdown => "Breakdown",
            ChapterType::Drop => "Drop",
            ChapterType::Marker => "Marker",
            ChapterType::Custom(s) => s.as_str(),
        }
    }
}

/// Rich Session Marker & Project Chapter descriptor.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionMarker {
    pub id: String,
    pub name: String,
    pub beat: f64,
    pub end_beat: Option<f64>,
    pub chapter_type: ChapterType,
    pub color: [u8; 3],
    pub notes: String,
    pub hotkey_binding: Option<String>,
}

impl SessionMarker {
    /// Create a standard point marker.
    pub fn new(name: impl Into<String>, beat: f64) -> Self {
        let name_str = name.into();
        Self {
            id: format!("marker_{}_{}", beat as u64, name_str.replace(' ', "_")),
            name: name_str,
            beat,
            end_beat: None,
            chapter_type: ChapterType::Marker,
            color: ChapterType::Marker.default_color(),
            notes: String::new(),
            hotkey_binding: None,
        }
    }

    /// Create a chapter range marker (e.g. Intro, Verse, Chorus).
    pub fn chapter(name: impl Into<String>, start_beat: f64, end_beat: f64, chapter_type: ChapterType) -> Self {
        let name_str = name.into();
        let color = chapter_type.default_color();
        Self {
            id: format!("chapter_{}_{}", start_beat as u64, name_str.replace(' ', "_")),
            name: name_str,
            beat: start_beat,
            end_beat: Some(end_beat),
            chapter_type,
            color,
            notes: String::new(),
            hotkey_binding: None,
        }
    }

    /// Set hotkey binding for this marker.
    pub fn with_hotkey(mut self, hotkey: impl Into<String>) -> Self {
        self.hotkey_binding = Some(hotkey.into());
        self
    }

    /// Set notes for this marker.
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = notes.into();
        self
    }

    /// Duration in beats if this marker is a chapter.
    pub fn duration_beats(&self) -> Option<f64> {
        self.end_beat.map(|end| (end - self.beat).max(0.0))
    }
}

impl From<MarkerConfig> for SessionMarker {
    fn from(cfg: MarkerConfig) -> Self {
        let chapter_type = cfg.chapter_type.unwrap_or(ChapterType::Marker);
        let color = cfg.color.unwrap_or_else(|| chapter_type.default_color());
        let id = format!("marker_{}_{}", cfg.beat as u64, cfg.name.replace(' ', "_"));
        SessionMarker {
            id,
            name: cfg.name,
            beat: cfg.beat,
            end_beat: cfg.end_beat,
            chapter_type,
            color,
            notes: cfg.notes.unwrap_or_default(),
            hotkey_binding: cfg.hotkey_binding,
        }
    }
}

impl From<&SessionMarker> for MarkerConfig {
    fn from(m: &SessionMarker) -> Self {
        MarkerConfig {
            name: m.name.clone(),
            beat: m.beat,
            color: Some(m.color),
            end_beat: m.end_beat,
            chapter_type: Some(m.chapter_type.clone()),
            notes: if m.notes.is_empty() { None } else { Some(m.notes.clone()) },
            hotkey_binding: m.hotkey_binding.clone(),
        }
    }
}

/// Resulting command from a hotkey / navigation trigger.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NavigationCommand {
    JumpToBeat(f64),
    MarkerCreated(SessionMarker),
    MarkerRemoved(String),
    LoopChapter { start_beat: f64, end_beat: f64 },
    ActiveChapterChanged(usize),
}

/// Navigation Manager controlling session markers, chapter navigation, hotkeys, and exports.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SessionMarkerNavigationManager {
    markers: Vec<SessionMarker>,
    active_marker_index: Option<usize>,
    hotkey_bindings: HashMap<String, String>, // hotkey -> marker id or action keyword
}

impl SessionMarkerNavigationManager {
    /// Create new empty navigation manager.
    pub fn new() -> Self {
        let mut mgr = Self {
            markers: Vec::new(),
            active_marker_index: None,
            hotkey_bindings: HashMap::new(),
        };
        mgr.register_default_hotkeys();
        mgr
    }

    /// Initialize navigation manager from project config.
    pub fn from_project(project: &ProjectConfig) -> Self {
        let mut mgr = Self::new();
        for m in &project.markers {
            mgr.add_marker(SessionMarker::from(m.clone()));
        }
        mgr
    }

    /// Sync active markers back into project config.
    pub fn sync_to_project(&self, project: &mut ProjectConfig) {
        project.markers = self.markers.iter().map(MarkerConfig::from).collect();
    }

    /// Register standard default hotkeys.
    pub fn register_default_hotkeys(&mut self) {
        self.hotkey_bindings.insert("Ctrl+Left".to_string(), "PREV_MARKER".to_string());
        self.hotkey_bindings.insert("P".to_string(), "PREV_MARKER".to_string());
        self.hotkey_bindings.insert("Ctrl+Right".to_string(), "NEXT_MARKER".to_string());
        self.hotkey_bindings.insert("N".to_string(), "NEXT_MARKER".to_string());
        self.hotkey_bindings.insert("M".to_string(), "CREATE_MARKER".to_string());
        self.hotkey_bindings.insert("Shift+M".to_string(), "CREATE_CHAPTER".to_string());
        self.hotkey_bindings.insert("L".to_string(), "LOOP_ACTIVE_CHAPTER".to_string());
        for i in 1..=9 {
            self.hotkey_bindings.insert(format!("{}", i), format!("JUMP_INDEX_{}", i - 1));
            self.hotkey_bindings.insert(format!("Ctrl+{}", i), format!("JUMP_INDEX_{}", i - 1));
        }
    }

    /// Bind custom hotkey to a marker ID or action keyword.
    pub fn bind_hotkey(&mut self, hotkey: impl Into<String>, target: impl Into<String>) {
        self.hotkey_bindings.insert(hotkey.into(), target.into());
    }

    /// Unbind hotkey.
    pub fn unbind_hotkey(&mut self, hotkey: &str) {
        self.hotkey_bindings.remove(hotkey);
    }

    /// Get current hotkey bindings map.
    pub fn hotkey_bindings(&self) -> &HashMap<String, String> {
        &self.hotkey_bindings
    }

    /// Add a session marker or chapter (automatically keeps markers sorted by beat).
    pub fn add_marker(&mut self, marker: SessionMarker) -> usize {
        if let Some(ref hk) = marker.hotkey_binding {
            self.hotkey_bindings.insert(hk.clone(), marker.id.clone());
        }

        let insert_idx = self.markers.binary_search_by(|m| {
            m.beat.partial_cmp(&marker.beat).unwrap_or(std::cmp::Ordering::Equal)
        }).unwrap_or_else(|e| e);

        self.markers.insert(insert_idx, marker);
        self.active_marker_index = Some(insert_idx);
        insert_idx
    }

    /// Add a quick chapter marker.
    pub fn add_chapter(&mut self, name: &str, start_beat: f64, end_beat: f64, chapter_type: ChapterType) -> usize {
        let marker = SessionMarker::chapter(name, start_beat, end_beat, chapter_type);
        self.add_marker(marker)
    }

    /// Remove marker by index.
    pub fn remove_marker(&mut self, index: usize) -> Option<SessionMarker> {
        if index < self.markers.len() {
            let removed = self.markers.remove(index);
            if let Some(ref hk) = removed.hotkey_binding {
                self.hotkey_bindings.remove(hk);
            }
            if self.active_marker_index == Some(index) {
                self.active_marker_index = if self.markers.is_empty() {
                    None
                } else {
                    Some(index.min(self.markers.len() - 1))
                };
            } else if let Some(ref mut active) = self.active_marker_index {
                if *active > index {
                    *active -= 1;
                }
            }
            Some(removed)
        } else {
            None
        }
    }

    /// Remove marker by ID.
    pub fn remove_marker_by_id(&mut self, id: &str) -> Option<SessionMarker> {
        if let Some(idx) = self.markers.iter().position(|m| m.id == id) {
            self.remove_marker(idx)
        } else {
            None
        }
    }

    /// Update marker at index.
    pub fn update_marker(&mut self, index: usize, updated: SessionMarker) -> bool {
        if index < self.markers.len() {
            self.remove_marker(index);
            self.add_marker(updated);
            true
        } else {
            false
        }
    }

    /// Get marker reference by index.
    pub fn get_marker(&self, index: usize) -> Option<&SessionMarker> {
        self.markers.get(index)
    }

    /// Get marker reference by ID.
    pub fn get_marker_by_id(&self, id: &str) -> Option<&SessionMarker> {
        self.markers.iter().find(|m| m.id == id)
    }

    /// Slice of all markers.
    pub fn markers(&self) -> &[SessionMarker] {
        &self.markers
    }

    /// Total count of markers.
    pub fn len(&self) -> usize {
        self.markers.len()
    }

    /// Is marker list empty.
    pub fn is_empty(&self) -> bool {
        self.markers.is_empty()
    }

    /// Active selected marker index.
    pub fn active_index(&self) -> Option<usize> {
        self.active_marker_index
    }

    /// Set active marker index manually.
    pub fn set_active_index(&mut self, index: Option<usize>) {
        if let Some(idx) = index {
            if idx < self.markers.len() {
                self.active_marker_index = Some(idx);
            }
        } else {
            self.active_marker_index = None;
        }
    }

    /// Jump to next marker after `current_beat`.
    pub fn jump_next(&mut self, current_beat: f64) -> Option<&SessionMarker> {
        let next_idx = self.markers.iter().position(|m| m.beat > current_beat + 0.0001);
        if let Some(idx) = next_idx {
            self.active_marker_index = Some(idx);
            Some(&self.markers[idx])
        } else {
            None
        }
    }

    /// Jump to previous marker before `current_beat`.
    pub fn jump_prev(&mut self, current_beat: f64) -> Option<&SessionMarker> {
        let prev_idx = self.markers.iter().rposition(|m| m.beat < current_beat - 0.0001);
        if let Some(idx) = prev_idx {
            self.active_marker_index = Some(idx);
            Some(&self.markers[idx])
        } else {
            None
        }
    }

    /// Find chapter or marker active at `beat`.
    pub fn find_chapter_at(&self, beat: f64) -> Option<&SessionMarker> {
        for (i, m) in self.markers.iter().enumerate() {
            if beat >= m.beat {
                if let Some(end) = m.end_beat {
                    if beat < end {
                        return Some(m);
                    }
                } else if i + 1 < self.markers.len() {
                    if beat < self.markers[i + 1].beat {
                        return Some(m);
                    }
                } else {
                    return Some(m);
                }
            }
        }
        None
    }

    /// Jump to marker by index.
    pub fn jump_to_index(&mut self, index: usize) -> Option<&SessionMarker> {
        if index < self.markers.len() {
            self.active_marker_index = Some(index);
            Some(&self.markers[index])
        } else {
            None
        }
    }

    /// Jump to marker by name.
    pub fn jump_to_name(&mut self, name: &str) -> Option<&SessionMarker> {
        if let Some(idx) = self.markers.iter().position(|m| m.name.eq_ignore_ascii_case(name)) {
            self.active_marker_index = Some(idx);
            Some(&self.markers[idx])
        } else {
            None
        }
    }

    /// Handle keyboard input / hotkey string, returning NavigationCommand if triggered.
    pub fn handle_key_input(&mut self, hotkey: &str, current_beat: f64) -> Option<NavigationCommand> {
        if let Some(target) = self.hotkey_bindings.get(hotkey).cloned() {
            match target.as_str() {
                "PREV_MARKER" => {
                    if let Some(m) = self.jump_prev(current_beat) {
                        return Some(NavigationCommand::JumpToBeat(m.beat));
                    }
                }
                "NEXT_MARKER" => {
                    if let Some(m) = self.jump_next(current_beat) {
                        return Some(NavigationCommand::JumpToBeat(m.beat));
                    }
                }
                "CREATE_MARKER" => {
                    let next_num = self.markers.len() + 1;
                    let marker = SessionMarker::new(format!("Marker {}", next_num), current_beat);
                    self.add_marker(marker.clone());
                    return Some(NavigationCommand::MarkerCreated(marker));
                }
                "CREATE_CHAPTER" => {
                    let next_num = self.markers.len() + 1;
                    let marker = SessionMarker::chapter(
                        format!("Chapter {}", next_num),
                        current_beat,
                        current_beat + 16.0,
                        ChapterType::Verse,
                    );
                    self.add_marker(marker.clone());
                    return Some(NavigationCommand::MarkerCreated(marker));
                }
                "LOOP_ACTIVE_CHAPTER" => {
                    if let Some(idx) = self.active_marker_index {
                        let m = &self.markers[idx];
                        let start = m.beat;
                        let end = m.end_beat.unwrap_or(start + 16.0);
                        return Some(NavigationCommand::LoopChapter { start_beat: start, end_beat: end });
                    } else if let Some(ch) = self.find_chapter_at(current_beat) {
                        let start = ch.beat;
                        let end = ch.end_beat.unwrap_or(start + 16.0);
                        return Some(NavigationCommand::LoopChapter { start_beat: start, end_beat: end });
                    }
                }
                other if other.starts_with("JUMP_INDEX_") => {
                    if let Ok(idx) = other.trim_start_matches("JUMP_INDEX_").parse::<usize>() {
                        if let Some(m) = self.jump_to_index(idx) {
                            return Some(NavigationCommand::JumpToBeat(m.beat));
                        }
                    }
                }
                marker_id => {
                    if let Some(m) = self.get_marker_by_id(marker_id) {
                        let beat = m.beat;
                        if let Some(idx) = self.markers.iter().position(|x| x.id == marker_id) {
                            self.active_marker_index = Some(idx);
                        }
                        return Some(NavigationCommand::JumpToBeat(beat));
                    }
                }
            }
        }
        None
    }

    /// Export session markers as YouTube/Podcast formatted timestamps (e.g. `00:00 Intro\n01:30 Verse 1`).
    pub fn export_chapter_timestamps_text(&self, tempo_bpm: f64) -> String {
        let mut lines = Vec::new();
        let beats_per_sec = tempo_bpm / 60.0;
        for m in &self.markers {
            let total_secs = (m.beat / beats_per_sec).max(0.0) as u64;
            let mins = total_secs / 60;
            let secs = total_secs % 60;
            lines.push(format!("{:02}:{:02} {}", mins, secs, m.name));
        }
        lines.join("\n")
    }

    /// Export session markers as standard CUE sheet content.
    pub fn export_cue_sheet(
        &self,
        track_title: &str,
        performer: &str,
        audio_file: &str,
        tempo_bpm: f64,
    ) -> String {
        let mut out = String::new();
        out.push_str(&format!("TITLE \"{}\"\n", track_title));
        out.push_str(&format!("PERFORMER \"{}\"\n", performer));
        out.push_str(&format!("FILE \"{}\" WAVE\n", audio_file));

        let beats_per_sec = tempo_bpm / 60.0;
        for (i, m) in self.markers.iter().enumerate() {
            let total_secs = (m.beat / beats_per_sec).max(0.0);
            let mins = (total_secs / 60.0) as u32;
            let secs = (total_secs % 60.0) as u32;
            let frames = (((total_secs % 1.0) * 75.0).round() as u32).min(74);

            out.push_str(&format!("  TRACK {:02} AUDIO\n", i + 1));
            out.push_str(&format!("    TITLE \"{}\"\n", m.name));
            out.push_str(&format!("    INDEX 01 {:02}:{:02}:{:02}\n", mins, secs, frames));
        }
        out
    }

    /// Export navigation manager state to JSON.
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Import navigation manager state from JSON.
    pub fn import_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_default_project;

    #[test]
    fn test_step_1246_session_marker_and_chapter_navigation() {
        let mut mgr = SessionMarkerNavigationManager::new();
        assert!(mgr.is_empty());

        let idx_intro = mgr.add_chapter("Intro", 0.0, 16.0, ChapterType::Intro);
        let idx_verse = mgr.add_chapter("Verse 1", 16.0, 48.0, ChapterType::Verse);
        let idx_chorus = mgr.add_chapter("Chorus", 48.0, 80.0, ChapterType::Chorus);

        assert_eq!(mgr.len(), 3);
        assert_eq!(idx_intro, 0);
        assert_eq!(idx_verse, 1);
        assert_eq!(idx_chorus, 2);

        // Find active chapter at beat 32
        let current_ch = mgr.find_chapter_at(32.0).expect("chapter at beat 32");
        assert_eq!(current_ch.name, "Verse 1");
        assert_eq!(current_ch.chapter_type, ChapterType::Verse);

        // Test navigation next/prev
        let next = mgr.jump_next(0.0).expect("next marker");
        assert_eq!(next.name, "Verse 1");

        let prev = mgr.jump_prev(48.0).expect("prev marker");
        assert_eq!(prev.name, "Verse 1");

        // Test hotkey navigation
        let cmd = mgr.handle_key_input("Ctrl+Right", 0.0);
        assert_eq!(cmd, Some(NavigationCommand::JumpToBeat(16.0)));

        let cmd_create = mgr.handle_key_input("M", 100.0);
        assert!(matches!(cmd_create, Some(NavigationCommand::MarkerCreated(_))));
        assert_eq!(mgr.len(), 4);

        // Test project sync
        let mut proj = create_default_project("Marker Test Project");
        mgr.sync_to_project(&mut proj);
        assert_eq!(proj.markers.len(), 4);

        let restored_mgr = SessionMarkerNavigationManager::from_project(&proj);
        assert_eq!(restored_mgr.len(), 4);

        // Test YouTube timestamp export
        let yt_text = mgr.export_chapter_timestamps_text(120.0);
        assert!(yt_text.contains("00:00 Intro"));
        assert!(yt_text.contains("00:08 Verse 1"));

        // Test CUE sheet export
        let cue = mgr.export_cue_sheet("Epic Track", "Artist", "track.wav", 120.0);
        assert!(cue.contains("TITLE \"Epic Track\""));
        assert!(cue.contains("TRACK 01 AUDIO"));
        assert!(cue.contains("TITLE \"Intro\""));

        // Test JSON round-trip
        let json = mgr.export_json().unwrap();
        let imported = SessionMarkerNavigationManager::import_json(&json).unwrap();
        assert_eq!(imported.len(), mgr.len());
    }
}
