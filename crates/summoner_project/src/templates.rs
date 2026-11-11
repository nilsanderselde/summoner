// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Project and Track Templates Management with Factory Presets and "Save As Default".

use crate::schema::{NodeConfig, ProjectConfig, TrackConfig, TransportConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// A reusable track archetype template.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrackTemplate {
    pub name: String,
    pub category: String,
    pub color_rgba: [u8; 4],
    pub gain: f32,
    pub pan: f32,
    pub channels: usize,
    pub nodes: Vec<NodeConfig>,
    pub send_level: f32,
    pub bus_target: Option<String>,
}

impl Default for TrackTemplate {
    fn default() -> Self {
        Self {
            name: "Audio Track".to_string(),
            category: "Audio".to_string(),
            color_rgba: [80, 140, 220, 255],
            gain: 0.8,
            pan: 0.0,
            channels: 2,
            nodes: Vec::new(),
            send_level: 0.0,
            bus_target: None,
        }
    }
}

impl TrackTemplate {
    pub fn new(name: &str, category: &str) -> Self {
        Self {
            name: name.to_string(),
            category: category.to_string(),
            ..Default::default()
        }
    }

    /// Instantiate this template into a concrete `TrackConfig`.
    pub fn instantiate(&self, track_id: u64) -> TrackConfig {
        TrackConfig {
            id: track_id,
            name: self.name.clone(),
            channels: self.channels,
            gain: self.gain,
            pan: self.pan,
            nodes: self.nodes.clone(),
            send_level: self.send_level,
            bus_target: self.bus_target.clone(),
            send_to_master: true,
            ..Default::default()
        }
    }
}

/// A complete project template configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub bpm: f64,
    pub time_signature: String,
    pub tracks: Vec<TrackTemplate>,
    pub master_nodes: Vec<NodeConfig>,
}

impl Default for ProjectTemplate {
    fn default() -> Self {
        Self {
            id: "default_blank".to_string(),
            name: "Default Blank".to_string(),
            description: "Default starting template with basic audio and MIDI tracks".to_string(),
            category: "General".to_string(),
            bpm: 120.0,
            time_signature: "4/4".to_string(),
            tracks: vec![
                TrackTemplate::new("Audio 1", "Audio"),
                TrackTemplate::new("Audio 2", "Audio"),
                TrackTemplate::new("MIDI 1", "MIDI"),
            ],
            master_nodes: Vec::new(),
        }
    }
}

impl ProjectTemplate {
    /// Instantiate this template into a complete `ProjectConfig`.
    pub fn instantiate(&self, project_name: &str) -> ProjectConfig {
        let mut tracks = Vec::new();
        for (i, t_tpl) in self.tracks.iter().enumerate() {
            tracks.push(t_tpl.instantiate((i + 1) as u64));
        }

        if !self.master_nodes.is_empty() {
            let master_id = (tracks.len() + 1) as u64;
            tracks.push(TrackConfig {
                id: master_id,
                name: "Master".to_string(),
                channels: 2,
                gain: 1.0,
                pan: 0.0,
                nodes: self.master_nodes.clone(),
                send_to_master: false,
                ..Default::default()
            });
        }

        ProjectConfig {
            name: project_name.to_string(),
            transport: TransportConfig {
                bpm: self.bpm,
                time_signature: self.time_signature.clone(),
                ..Default::default()
            },
            tracks,
            ..Default::default()
        }
    }
}

/// Template manager handling discovery, serialization, and default configuration.
pub struct TemplateManager {
    pub user_templates_dir: PathBuf,
}

impl TemplateManager {
    pub fn new(templates_dir: impl Into<PathBuf>) -> Self {
        Self {
            user_templates_dir: templates_dir.into(),
        }
    }

    /// Factory-curated templates for various musical genres and workflows.
    pub fn factory_templates() -> Vec<ProjectTemplate> {
        vec![
            ProjectTemplate::default(),
            Self::synthwave_template(),
            Self::acoustic_template(),
            Self::mastering_template(),
        ]
    }

    fn synthwave_template() -> ProjectTemplate {
        let make_synth_node = |kind: &str| NodeConfig {
            kind: kind.to_string(),
            params: HashMap::new(),
            plugin_state: None,
        };

        ProjectTemplate {
            id: "synthwave".to_string(),
            name: "Synthwave / Outrun".to_string(),
            description: "80s retro synthwave setup with drum machine, analog bass, and lush leads"
                .to_string(),
            category: "Electronic".to_string(),
            bpm: 124.0,
            time_signature: "4/4".to_string(),
            tracks: vec![
                TrackTemplate {
                    name: "Kick & Snare".to_string(),
                    category: "Drums".to_string(),
                    color_rgba: [220, 60, 60, 255],
                    gain: 0.9,
                    pan: 0.0,
                    channels: 2,
                    nodes: vec![make_synth_node("DrumMachine")],
                    send_level: 0.1,
                    bus_target: None,
                },
                TrackTemplate {
                    name: "Analog Bass".to_string(),
                    category: "Bass".to_string(),
                    color_rgba: [60, 160, 240, 255],
                    gain: 0.85,
                    pan: 0.0,
                    channels: 2,
                    nodes: vec![make_synth_node("DailySubtractiveSynth")],
                    send_level: 0.05,
                    bus_target: None,
                },
                TrackTemplate {
                    name: "Poly Lead".to_string(),
                    category: "Synth".to_string(),
                    color_rgba: [240, 180, 50, 255],
                    gain: 0.8,
                    pan: 0.1,
                    channels: 2,
                    nodes: vec![make_synth_node("DailyWavetableSynth")],
                    send_level: 0.3,
                    bus_target: None,
                },
                TrackTemplate {
                    name: "Dream Arp".to_string(),
                    category: "Synth".to_string(),
                    color_rgba: [180, 80, 240, 255],
                    gain: 0.75,
                    pan: -0.15,
                    channels: 2,
                    nodes: vec![make_synth_node("DailyFmSynth")],
                    send_level: 0.25,
                    bus_target: None,
                },
            ],
            master_nodes: vec![make_synth_node("LimiterNode")],
        }
    }

    fn acoustic_template() -> ProjectTemplate {
        let make_node = |kind: &str| NodeConfig {
            kind: kind.to_string(),
            params: HashMap::new(),
            plugin_state: None,
        };

        ProjectTemplate {
            id: "acoustic_songwriter".to_string(),
            name: "Acoustic / Songwriter".to_string(),
            description: "Clean tracking template for lead vocals and stereo acoustic guitars"
                .to_string(),
            category: "Acoustic".to_string(),
            bpm: 104.0,
            time_signature: "4/4".to_string(),
            tracks: vec![
                TrackTemplate {
                    name: "Lead Vocal Comp".to_string(),
                    category: "Vocals".to_string(),
                    color_rgba: [240, 120, 100, 255],
                    gain: 0.85,
                    pan: 0.0,
                    channels: 2,
                    nodes: vec![make_node("CompressorNode"), make_node("EqNode")],
                    send_level: 0.2,
                    bus_target: None,
                },
                TrackTemplate {
                    name: "Acoustic Guitar L".to_string(),
                    category: "Acoustic".to_string(),
                    color_rgba: [100, 200, 120, 255],
                    gain: 0.75,
                    pan: -0.4,
                    channels: 2,
                    nodes: vec![make_node("EqNode")],
                    send_level: 0.15,
                    bus_target: None,
                },
                TrackTemplate {
                    name: "Acoustic Guitar R".to_string(),
                    category: "Acoustic".to_string(),
                    color_rgba: [100, 200, 120, 255],
                    gain: 0.75,
                    pan: 0.4,
                    channels: 2,
                    nodes: vec![make_node("EqNode")],
                    send_level: 0.15,
                    bus_target: None,
                },
            ],
            master_nodes: vec![make_node("CompressorNode"), make_node("LimiterNode")],
        }
    }

    fn mastering_template() -> ProjectTemplate {
        let make_node = |kind: &str| NodeConfig {
            kind: kind.to_string(),
            params: HashMap::new(),
            plugin_state: None,
        };

        ProjectTemplate {
            id: "mastering_suite".to_string(),
            name: "Mastering Suite".to_string(),
            description: "Precision mastering session with reference track and loudness analysis"
                .to_string(),
            category: "Mastering".to_string(),
            bpm: 120.0,
            time_signature: "4/4".to_string(),
            tracks: vec![
                TrackTemplate {
                    name: "Target Mix".to_string(),
                    category: "Mastering".to_string(),
                    color_rgba: [80, 180, 240, 255],
                    gain: 1.0,
                    pan: 0.0,
                    channels: 2,
                    nodes: vec![],
                    send_level: 0.0,
                    bus_target: None,
                },
                TrackTemplate {
                    name: "Commercial Reference".to_string(),
                    category: "Reference".to_string(),
                    color_rgba: [160, 160, 160, 255],
                    gain: 1.0,
                    pan: 0.0,
                    channels: 2,
                    nodes: vec![],
                    send_level: 0.0,
                    bus_target: None,
                },
            ],
            master_nodes: vec![
                make_node("EqNode"),
                make_node("CompressorNode"),
                make_node("LimiterNode"),
            ],
        }
    }

    /// Save a project template to TOML file.
    pub fn save_project_template(
        &self,
        template: &ProjectTemplate,
        dest_path: &Path,
    ) -> Result<(), String> {
        let toml_str = toml::to_string_pretty(template).map_err(|e| e.to_string())?;
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(dest_path, toml_str).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Load a project template from a TOML file.
    pub fn load_project_template(path: &Path) -> Result<ProjectTemplate, String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        toml::from_str(&content).map_err(|e| e.to_string())
    }

    /// Save a track template to TOML file.
    pub fn save_track_template(
        &self,
        template: &TrackTemplate,
        dest_path: &Path,
    ) -> Result<(), String> {
        let toml_str = toml::to_string_pretty(template).map_err(|e| e.to_string())?;
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(dest_path, toml_str).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Load a track template from a TOML file.
    pub fn load_track_template(path: &Path) -> Result<TrackTemplate, String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        toml::from_str(&content).map_err(|e| e.to_string())
    }

    /// Save a template as the user's default template.
    pub fn save_as_default_template(&self, template: &ProjectTemplate) -> Result<(), String> {
        let default_path = self.user_templates_dir.join("default_template.toml");
        self.save_project_template(template, &default_path)
    }

    /// Load user's default template, falling back to factory default blank.
    pub fn load_default_template(&self) -> ProjectTemplate {
        let default_path = self.user_templates_dir.join("default_template.toml");
        if default_path.exists() {
            if let Ok(tpl) = Self::load_project_template(&default_path) {
                return tpl;
            }
        }
        ProjectTemplate::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_instantiation() {
        let synthwave = TemplateManager::synthwave_template();
        let project = synthwave.instantiate("My Cyberpunk Song");

        assert_eq!(project.name, "My Cyberpunk Song");
        assert_eq!(project.transport.bpm, 124.0);
        assert_eq!(project.transport.time_signature, "4/4");
        // 4 tracks + 1 Master track
        assert_eq!(project.tracks.len(), 5);
        assert_eq!(project.tracks[0].name, "Kick & Snare");
        assert_eq!(project.tracks[1].name, "Analog Bass");
        assert_eq!(project.tracks[4].name, "Master");
        assert_eq!(project.tracks[4].nodes.len(), 1);
        assert_eq!(project.tracks[4].nodes[0].kind, "LimiterNode");
    }

    #[test]
    fn test_track_template_instantiation() {
        let tpl = TrackTemplate {
            name: "Vocal Double".to_string(),
            category: "Vocals".to_string(),
            gain: 0.7,
            pan: 0.3,
            ..Default::default()
        };

        let track = tpl.instantiate(42);
        assert_eq!(track.id, 42);
        assert_eq!(track.name, "Vocal Double");
        assert_eq!(track.gain, 0.7);
        assert_eq!(track.pan, 0.3);
    }

    #[test]
    fn test_template_file_serialization_and_save_as_default() {
        let temp_dir = std::env::temp_dir().join("summoner_test_templates");
        let _ = fs::remove_dir_all(&temp_dir);
        let mgr = TemplateManager::new(&temp_dir);

        let custom_tpl = ProjectTemplate {
            name: "My Custom Default".to_string(),
            bpm: 140.0,
            ..Default::default()
        };

        // Save as default
        mgr.save_as_default_template(&custom_tpl).unwrap();

        // Load default
        let loaded = mgr.load_default_template();
        assert_eq!(loaded.name, "My Custom Default");
        assert_eq!(loaded.bpm, 140.0);

        // Track template serialization
        let track_tpl = TrackTemplate::new("Bass Guit", "Bass");
        let track_path = temp_dir.join("bass_track.toml");
        mgr.save_track_template(&track_tpl, &track_path).unwrap();
        let loaded_track = TemplateManager::load_track_template(&track_path).unwrap();
        assert_eq!(loaded_track.name, "Bass Guit");

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
