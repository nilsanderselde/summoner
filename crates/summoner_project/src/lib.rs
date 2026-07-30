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

//! Project document TOML schema parsing and serialization for Summoner DAW.

pub mod git_dag;
pub mod schema;
pub mod preset;
pub mod sfz;
pub mod export;
pub mod media_export;
pub mod system_tools;
pub mod project_tools;
pub mod crdt;
pub mod enterprise_qa;
pub mod cloud_federated;
pub mod scratch_audio_cache;
pub mod session_markers;

pub use media_export::*;
pub use crdt::*;
pub use export::*;
pub use enterprise_qa::*;
pub use cloud_federated::*;
pub use scratch_audio_cache::*;
pub use session_markers::*;

#[cfg(target_os = "windows")]
#[link(name = "advapi32")]
extern "C" {}



use schema::{NodeConfig, ProjectConfig, TrackConfig, TransportConfig};
use std::collections::HashMap;

/// Error type for project parsing and serialization.
#[derive(Debug)]
pub enum ProjectError {
    TomlParse(toml::de::Error),
    TomlSerialize(toml::ser::Error),
}

impl std::fmt::Display for ProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectError::TomlParse(e) => write!(f, "TOML parse error: {}", e),
            ProjectError::TomlSerialize(e) => write!(f, "TOML serialize error: {}", e),
        }
    }
}

impl std::error::Error for ProjectError {}

/// Parse TOML string into `ProjectConfig`.
pub fn parse_project_toml(content: &str) -> Result<ProjectConfig, ProjectError> {
    toml::from_str(content).map_err(ProjectError::TomlParse)
}

/// Serialize `ProjectConfig` into TOML string format.
pub fn serialize_project_toml(config: &ProjectConfig) -> Result<String, ProjectError> {
    toml::to_string_pretty(config).map_err(ProjectError::TomlSerialize)
}

/// Generate default project configuration.
pub fn create_default_project(name: &str) -> ProjectConfig {
    create_project_from_template(name, "Default")
}

/// Generate a project from a preset template (Step 416).
pub fn create_project_from_template(name: &str, template_kind: &str) -> ProjectConfig {
    let mut synth_params = HashMap::new();
    synth_params.insert("frequency".to_string(), 440.0);

    let mut gain_params = HashMap::new();
    gain_params.insert("gain".to_string(), 0.8);

    let base_tracks = match template_kind {
        "Synth + Drums" | "Drum Beat" => vec![
            TrackConfig {
                id: 1,
                name: "Master Track".to_string(),
                channels: 2,
                gain: 1.0,
                pan: 0.0,
                muted: false,
                soloed: false,
                send_level: 0.0,
                nodes: Vec::new(),
                sequence: None,
                clips: Vec::new(),
                connections: Vec::new(),
                tuning_edo: None,
                tuning_root_hz: None,
                tuning_scl_path: None,
                ..Default::default()
            },
            TrackConfig {
                id: 2,
                name: "Synth Lead".to_string(),
                channels: 2,
                gain: 0.75,
                pan: 0.0,
                muted: false,
                soloed: false,
                send_level: 0.0,
                nodes: vec![
                    NodeConfig {
                        kind: "SineOscillatorNode".to_string(),
                        params: synth_params,
                        plugin_state: None,
                    },
                    NodeConfig {
                        kind: "GainNode".to_string(),
                        params: gain_params.clone(),
                        plugin_state: None,
                    },
                ],
                sequence: None,
                clips: Vec::new(),
                connections: Vec::new(),
                tuning_edo: None,
                tuning_root_hz: None,
                tuning_scl_path: None,
                ..Default::default()
            },
            TrackConfig {
                id: 3,
                name: "Drum Track".to_string(),
                channels: 2,
                gain: 0.85,
                pan: 0.0,
                muted: false,
                soloed: false,
                send_level: 0.0,
                nodes: Vec::new(),
                sequence: None,
                clips: Vec::new(),
                connections: Vec::new(),
                tuning_edo: None,
                tuning_root_hz: None,
                tuning_scl_path: None,
                ..Default::default()
            },
        ],
        "Microtonal Exploration" => vec![
            TrackConfig {
                id: 1,
                name: "Master Track".to_string(),
                channels: 2,
                gain: 1.0,
                pan: 0.0,
                muted: false,
                soloed: false,
                send_level: 0.0,
                nodes: Vec::new(),
                sequence: None,
                clips: Vec::new(),
                connections: Vec::new(),
                tuning_edo: None,
                tuning_root_hz: None,
                tuning_scl_path: None,
                ..Default::default()
            },
            TrackConfig {
                id: 2,
                name: "19-EDO Micro Synth".to_string(),
                channels: 2,
                gain: 0.8,
                pan: 0.0,
                muted: false,
                soloed: false,
                send_level: 0.0,
                nodes: Vec::new(),
                sequence: None,
                clips: Vec::new(),
                connections: Vec::new(),
                tuning_edo: Some(19),
                tuning_root_hz: Some(440.0),
                tuning_scl_path: None,
                ..Default::default()
            },
        ],
        _ => vec![
            TrackConfig {
                id: 1,
                name: "Master Track".to_string(),
                channels: 2,
                gain: 1.0,
                pan: 0.0,
                muted: false,
                soloed: false,
                send_level: 0.0,
                nodes: Vec::new(),
                sequence: None,
                clips: Vec::new(),
                connections: Vec::new(),
                tuning_edo: None,
                tuning_root_hz: None,
                tuning_scl_path: None,
                ..Default::default()
            },
            TrackConfig {
                id: 2,
                name: "Synth Lead".to_string(),
                channels: 2,
                gain: 0.75,
                pan: 0.0,
                muted: false,
                soloed: false,
                send_level: 0.0,
                nodes: vec![
                    NodeConfig {
                        kind: "SineOscillatorNode".to_string(),
                        params: synth_params,
                        plugin_state: None,
                    },
                    NodeConfig {
                        kind: "GainNode".to_string(),
                        params: gain_params,
                        plugin_state: None,
                    },
                ],
                sequence: None,
                clips: Vec::new(),
                connections: Vec::new(),
                tuning_edo: None,
                tuning_root_hz: None,
                tuning_scl_path: None,
                ..Default::default()
            },
        ],
    };

    ProjectConfig {
        name: name.to_string(),
        tuning_file: None,
        transport: TransportConfig::default(),
        tracks: base_tracks,
        assets: Vec::new(),
        automation_lanes: Vec::new(),
        midi_mappings: Vec::new(),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use schema::{SequenceConfig, TrackerStepConfig};

    #[test]
    fn test_project_roundtrip() {
        let proj = create_default_project("Test Session");
        let serialized = serialize_project_toml(&proj).expect("Serialization failed");
        let parsed = parse_project_toml(&serialized).expect("Deserialization failed");
        assert_eq!(proj, parsed);
    }

    #[test]
    fn test_polymetric_sequence_roundtrip() {
        let mut proj = create_default_project("Polymetric Session");
        proj.tracks[1].sequence = Some(SequenceConfig {
            start_beat: 0.0,
            step_division: 0.25,
            clip_color: None,
            clip_name: None,
            name: "Default Clip".to_string(),
            is_unique: false,
            steps: vec![
                TrackerStepConfig {
                    note: 60.0,
                    velocity: 0.9,
                    gate: 0.5,
                    probability: 1.0,
                    ratchet: 1,
                    micro_shift: 0,
                    swing: 0.0,
                    pan: 0.0,
                    pitch_offset: 0.0,
                    active: true,
                    muted: false,
                },
                TrackerStepConfig {
                    note: 64.0,
                    velocity: 0.8,
                    gate: 0.5,
                    probability: 1.0,
                    ratchet: 1,
                    micro_shift: 0,
                    swing: 0.0,
                    pan: 0.0,
                    pitch_offset: 0.0,
                    active: true,
                    muted: false,
                },
                TrackerStepConfig {
                    note: 67.0,
                    velocity: 0.85,
                    gate: 0.5,
                    probability: 1.0,
                    ratchet: 1,
                    micro_shift: 0,
                    swing: 0.0,
                    pan: 0.0,
                    pitch_offset: 0.0,
                    active: true,
                    muted: false,
                },
            ],
            ..Default::default()
        });

        let serialized = serialize_project_toml(&proj).expect("Serialization failed");
        let parsed = parse_project_toml(&serialized).expect("Deserialization failed");
        assert_eq!(proj, parsed);
    }

    #[test]
    fn test_preset_schema_validates_freepats_presets() {
        let presets_dir = std::path::Path::new("local/presets/freepats");
        if presets_dir.exists() {
            let entries = std::fs::read_dir(presets_dir).unwrap();
            let mut validated_count = 0;
            for entry in entries {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("toml") || path.to_string_lossy().ends_with(".preset.toml") {
                    let content = std::fs::read_to_string(&path).unwrap();
                    let val: toml::Value = toml::from_str(&content).unwrap();
                    assert!(val.get("name").is_some(), "Preset {:?} missing 'name'", path);
                    assert!(val.get("regions").is_some(), "Preset {:?} missing 'regions'", path);
                    validated_count += 1;
                }
            }
            assert!(validated_count > 0, "Expected to validate at least one freepats preset");
        }
    }
}

