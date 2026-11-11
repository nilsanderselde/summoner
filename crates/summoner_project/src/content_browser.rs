// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.

//! Content Browser: indexing, categorization, tag taxonomy, favorites persistence,
//! and tempo-synchronized audio auditioning engine.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use summoner_core::audio::Sample;

/// Categorization kind for content items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContentItemKind {
    AudioSample,
    AudioLoop,
    Preset,
    Instrument,
    MidiClip,
    Plugin,
    ProjectDemo,
}

impl ContentItemKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentItemKind::AudioSample => "Sample",
            ContentItemKind::AudioLoop => "Loop",
            ContentItemKind::Preset => "Preset",
            ContentItemKind::Instrument => "Instrument",
            ContentItemKind::MidiClip => "MIDI",
            ContentItemKind::Plugin => "Plugin",
            ContentItemKind::ProjectDemo => "Demo",
        }
    }
}

/// An indexed item in the content library.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContentItem {
    pub id: String,
    pub path: PathBuf,
    pub name: String,
    pub kind: ContentItemKind,
    pub tags: Vec<String>,
    pub bpm: Option<f32>,
    pub key: Option<String>,
    pub duration_seconds: f32,
    pub sample_rate: u32,
    pub channels: usize,
    pub favorite: bool,
    pub rating: u8,
}

impl ContentItem {
    pub fn new(path: impl Into<PathBuf>, kind: ContentItemKind) -> Self {
        let path = path.into();
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unnamed")
            .to_string();

        let id = blake3::hash(path.to_string_lossy().as_bytes()).to_string();

        Self {
            id,
            path,
            name,
            kind,
            tags: Vec::new(),
            bpm: None,
            key: None,
            duration_seconds: 0.0,
            sample_rate: 44100,
            channels: 2,
            favorite: false,
            rating: 0,
        }
    }

    pub fn with_tags(mut self, tags: &[&str]) -> Self {
        self.tags = tags.iter().map(|s| s.to_lowercase()).collect();
        self
    }

    pub fn with_bpm(mut self, bpm: f32) -> Self {
        self.bpm = Some(bpm);
        self
    }

    pub fn with_audio_metadata(
        mut self,
        duration_seconds: f32,
        sample_rate: u32,
        channels: usize,
    ) -> Self {
        self.duration_seconds = duration_seconds;
        self.sample_rate = sample_rate;
        self.channels = channels;
        self
    }
}

/// Query filters for searching the content library.
#[derive(Debug, Clone, Default)]
pub struct ContentSearchQuery {
    pub text: String,
    pub tags: Vec<String>,
    pub kinds: Vec<ContentItemKind>,
    pub bpm_range: Option<(f32, f32)>,
    pub favorites_only: bool,
    pub min_rating: Option<u8>,
}

impl ContentSearchQuery {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Default::default()
        }
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into().to_lowercase());
        self
    }

    pub fn with_kind(mut self, kind: ContentItemKind) -> Self {
        self.kinds.push(kind);
        self
    }

    pub fn with_favorites_only(mut self, fav_only: bool) -> Self {
        self.favorites_only = fav_only;
        self
    }
}

/// The Content Browser database and search index.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContentBrowser {
    pub items: Vec<ContentItem>,
    pub favorite_ids: HashSet<String>,
    pub custom_tags: HashMap<String, Vec<String>>,
}

impl ContentBrowser {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an item to the library.
    pub fn add_item(&mut self, mut item: ContentItem) {
        if self.favorite_ids.contains(&item.id) {
            item.favorite = true;
        }
        self.items.push(item);
    }

    /// Toggle favorite status of an item by id.
    pub fn toggle_favorite(&mut self, id: &str) -> bool {
        if self.favorite_ids.contains(id) {
            self.favorite_ids.remove(id);
            if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
                item.favorite = false;
            }
            false
        } else {
            self.favorite_ids.insert(id.to_string());
            if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
                item.favorite = true;
            }
            true
        }
    }

    /// Set rating (0-5 stars) for an item.
    pub fn set_rating(&mut self, id: &str, rating: u8) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.rating = rating.min(5);
        }
    }

    /// Scan a directory and index all discovered sound files and presets.
    pub fn scan_directory(&mut self, root: &Path) -> usize {
        let mut count = 0;
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    count += self.scan_directory(&path);
                } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    let kind = match ext_lower.as_str() {
                        "wav" | "flac" | "aif" | "aiff" | "ogg" | "mp3" => {
                            let name_lower = path
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("")
                                .to_lowercase();
                            if name_lower.contains("loop") || name_lower.contains("bpm") {
                                ContentItemKind::AudioLoop
                            } else {
                                ContentItemKind::AudioSample
                            }
                        }
                        "toml" | "json" => ContentItemKind::Preset,
                        "sfz" | "sf2" => ContentItemKind::Instrument,
                        "mid" | "midi" => ContentItemKind::MidiClip,
                        "vst3" | "clap" => ContentItemKind::Plugin,
                        _ => continue,
                    };

                    let mut item = ContentItem::new(&path, kind);

                    // Extract auto-tags from filename and directory
                    let mut tags = Vec::new();
                    if let Some(parent) = path
                        .parent()
                        .and_then(|p| p.file_name())
                        .and_then(|n| n.to_str())
                    {
                        tags.push(parent.to_lowercase());
                    }
                    let stem = item.name.to_lowercase();
                    if stem.contains("kick") {
                        tags.push("kick".into());
                        tags.push("drums".into());
                    }
                    if stem.contains("snare") {
                        tags.push("snare".into());
                        tags.push("drums".into());
                    }
                    if stem.contains("hihat") || stem.contains("hat") {
                        tags.push("hihat".into());
                        tags.push("drums".into());
                    }
                    if stem.contains("bass") {
                        tags.push("bass".into());
                    }
                    if stem.contains("lead") {
                        tags.push("lead".into());
                        tags.push("synth".into());
                    }
                    if stem.contains("pad") {
                        tags.push("pad".into());
                        tags.push("synth".into());
                    }

                    // Detect tempo from name (e.g., "drum_loop_128bpm.wav")
                    if let Some(bpm) = parse_tempo_from_filename(&stem) {
                        item.bpm = Some(bpm);
                        tags.push(format!("{}bpm", bpm as u32));
                    }

                    item.tags = tags;
                    self.add_item(item);
                    count += 1;
                }
            }
        }
        count
    }

    /// Search the library with filters and fuzzy/substring ranking.
    pub fn search(&self, query: &ContentSearchQuery) -> Vec<&ContentItem> {
        let q_lower = query.text.trim().to_lowercase();

        self.items
            .iter()
            .filter(|item| {
                if query.favorites_only && !item.favorite {
                    return false;
                }

                if let Some(min_r) = query.min_rating {
                    if item.rating < min_r {
                        return false;
                    }
                }

                if !query.kinds.is_empty() && !query.kinds.contains(&item.kind) {
                    return false;
                }

                if let Some((min_bpm, max_bpm)) = query.bpm_range {
                    match item.bpm {
                        Some(b) => {
                            if b < min_bpm || b > max_bpm {
                                return false;
                            }
                        }
                        None => return false,
                    }
                }

                if !query.tags.is_empty() {
                    let has_all_tags = query
                        .tags
                        .iter()
                        .all(|qt| item.tags.iter().any(|t| t == qt));
                    if !has_all_tags {
                        return false;
                    }
                }

                if !q_lower.is_empty() {
                    let name_match = item.name.to_lowercase().contains(&q_lower);
                    let tag_match = item.tags.iter().any(|t| t.contains(&q_lower));
                    let path_match = item
                        .path
                        .to_string_lossy()
                        .to_lowercase()
                        .contains(&q_lower);
                    if !name_match && !tag_match && !path_match {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    /// Serialize index to JSON file.
    pub fn save_to_file(&self, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())
    }

    /// Deserialize index from JSON file.
    pub fn load_from_file(path: &Path) -> Result<Self, String> {
        let json = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&json).map_err(|e| e.to_string())
    }
}

/// Helper to parse tempo numbers like "120bpm", "128_bpm", "bpm140" from names.
fn parse_tempo_from_filename(name: &str) -> Option<f32> {
    if let Some(idx) = name.find("bpm") {
        // Look backwards for digits
        let prefix = &name[..idx];
        let digits: String = prefix
            .chars()
            .rev()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        let forward: String = digits.chars().rev().collect();
        if let Ok(val) = forward.parse::<f32>() {
            if (40.0..=300.0).contains(&val) {
                return Some(val);
            }
        }
    }
    None
}

/// Audio Audition Engine: preview selected sample or loop synchronized with project tempo.
#[derive(Debug, Clone)]
pub struct AuditionEngine {
    pub is_playing: bool,
    pub volume: f32,
    pub playback_rate: f64,
    pub playback_pos: f64,
    pub audio_buffer: Vec<Sample>,
    pub channels: usize,
    pub loop_mode: bool,
    pub project_bpm: f32,
}

impl Default for AuditionEngine {
    fn default() -> Self {
        Self {
            is_playing: false,
            volume: 0.8,
            playback_rate: 1.0,
            playback_pos: 0.0,
            audio_buffer: Vec::new(),
            channels: 2,
            loop_mode: false,
            project_bpm: 120.0,
        }
    }
}

impl AuditionEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_project_bpm(&mut self, bpm: f32) {
        self.project_bpm = bpm.clamp(20.0, 400.0);
    }

    /// Start auditioning an audio buffer. If sample_bpm is provided, stretch/pitch to project tempo.
    pub fn play_buffer(
        &mut self,
        buffer: Vec<Sample>,
        channels: usize,
        sample_bpm: Option<f32>,
        loop_playback: bool,
    ) {
        self.audio_buffer = buffer;
        self.channels = channels.max(1);
        self.playback_pos = 0.0;
        self.loop_mode = loop_playback;

        if let Some(s_bpm) = sample_bpm {
            if s_bpm > 0.0 {
                // Rate adjusts speed so audio matches project tempo!
                self.playback_rate = (self.project_bpm / s_bpm) as f64;
            } else {
                self.playback_rate = 1.0;
            }
        } else {
            self.playback_rate = 1.0;
        }

        self.is_playing = true;
    }

    /// Stop auditioning.
    pub fn stop(&mut self) {
        self.is_playing = false;
        self.playback_pos = 0.0;
    }

    /// Render audition audio into stereo output buffers.
    pub fn process_block(&mut self, out_left: &mut [Sample], out_right: &mut [Sample]) {
        if !self.is_playing || self.audio_buffer.is_empty() {
            out_left.fill(0.0);
            out_right.fill(0.0);
            return;
        }

        let total_frames = self.audio_buffer.len() / self.channels;
        let num_frames = out_left.len().min(out_right.len());

        for i in 0..num_frames {
            let current_frame = self.playback_pos.floor() as usize;
            let frac = (self.playback_pos - current_frame as f64) as f32;

            if current_frame >= total_frames {
                if self.loop_mode {
                    self.playback_pos = 0.0;
                } else {
                    self.is_playing = false;
                    out_left[i..].fill(0.0);
                    out_right[i..].fill(0.0);
                    break;
                }
            }

            let next_frame = if current_frame + 1 < total_frames {
                current_frame + 1
            } else if self.loop_mode {
                0
            } else {
                current_frame
            };

            let s0_l = self.audio_buffer[current_frame * self.channels];
            let s1_l = self.audio_buffer[next_frame * self.channels];
            let sample_l = (s0_l + frac * (s1_l - s0_l)) * self.volume;

            let sample_r = if self.channels > 1 {
                let s0_r = self.audio_buffer[current_frame * self.channels + 1];
                let s1_r = self.audio_buffer[next_frame * self.channels + 1];
                (s0_r + frac * (s1_r - s0_r)) * self.volume
            } else {
                sample_l
            };

            out_left[i] = sample_l;
            out_right[i] = sample_r;

            self.playback_pos += self.playback_rate;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_browser_search_and_favorites() {
        let mut browser = ContentBrowser::new();

        let item1 = ContentItem::new(
            "samples/drums/punchy_808_kick.wav",
            ContentItemKind::AudioSample,
        )
        .with_tags(&["drums", "kick", "808", "electronic"])
        .with_audio_metadata(0.45, 44100, 1);

        let item2 = ContentItem::new(
            "loops/synth/synthwave_lead_120bpm.wav",
            ContentItemKind::AudioLoop,
        )
        .with_tags(&["synth", "lead", "synthwave", "loop"])
        .with_bpm(120.0)
        .with_audio_metadata(4.0, 44100, 2);

        let item3 = ContentItem::new("presets/sub_bass_cyber.toml", ContentItemKind::Preset)
            .with_tags(&["bass", "preset", "cyberpunk"]);

        browser.add_item(item1.clone());
        browser.add_item(item2.clone());
        browser.add_item(item3.clone());

        // 1. Substring search for "kick"
        let q_kick = ContentSearchQuery::new("kick");
        let results = browser.search(&q_kick);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "punchy_808_kick");

        // 2. Filter by tag "synth"
        let q_tag = ContentSearchQuery::new("").with_tag("synth");
        let results = browser.search(&q_tag);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "synthwave_lead_120bpm");

        // 3. Filter by kind Preset
        let q_kind = ContentSearchQuery::new("").with_kind(ContentItemKind::Preset);
        let results = browser.search(&q_kind);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "sub_bass_cyber");

        // 4. Toggle favorite
        assert!(!browser.items[0].favorite);
        let now_fav = browser.toggle_favorite(&item1.id);
        assert!(now_fav);
        assert!(browser.items[0].favorite);

        // 5. Query favorites only
        let q_fav = ContentSearchQuery::new("").with_favorites_only(true);
        let fav_results = browser.search(&q_fav);
        assert_eq!(fav_results.len(), 1);
        assert_eq!(fav_results[0].id, item1.id);
    }

    #[test]
    fn test_audition_tempo_matching() {
        let mut audition = AuditionEngine::new();
        audition.set_project_bpm(140.0);

        // A 120 BPM 2-beat loop: at 140 BPM, it should play faster (rate = 140 / 120 = 1.1666...)
        let dummy_samples = vec![0.5f32; 88200];
        audition.play_buffer(dummy_samples, 2, Some(120.0), true);

        assert!((audition.playback_rate - (140.0 / 120.0)).abs() < 1e-4);

        let mut out_l = vec![0.0f32; 128];
        let mut out_r = vec![0.0f32; 128];
        audition.process_block(&mut out_l, &mut out_r);

        assert_eq!(out_l[0], 0.5 * 0.8);
        assert_eq!(out_r[0], 0.5 * 0.8);
        assert!(audition.playback_pos > 128.0);
    }
}
