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
    let mut synth_params = HashMap::new();
    synth_params.insert("frequency".to_string(), 440.0);

    let mut gain_params = HashMap::new();
    gain_params.insert("gain".to_string(), 0.8);

    ProjectConfig {
        name: name.to_string(),
        tuning_file: None,
        transport: TransportConfig::default(),
        tracks: vec![
            TrackConfig {
                id: 1,
                name: "Master Track".to_string(),
                channels: 2,
                gain: 1.0,
                pan: 0.0,
                muted: false,
                nodes: vec![NodeConfig {
                    kind: "GainNode".to_string(),
                    params: gain_params,
                }],
                sequence: None,
                connections: Vec::new(),
            },
            TrackConfig {
                id: 2,
                name: "Synth Lead".to_string(),
                channels: 2,
                gain: 0.75,
                pan: 0.0,
                muted: false,
                nodes: vec![
                    NodeConfig {
                        kind: "SineOscillatorNode".to_string(),
                        params: synth_params,
                    },
                    NodeConfig {
                        kind: "GainNode".to_string(),
                        params: {
                            let mut p = HashMap::new();
                            p.insert("gain".to_string(), 0.5);
                            p
                        },
                    },
                ],
                sequence: None,
                connections: Vec::new(),
            },

        ],
        assets: Vec::new(),
        automation_lanes: Vec::new(),
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
            step_division: 0.25,
            steps: vec![
                TrackerStepConfig {
                    note: 60.0,
                    velocity: 0.9,
                    gate: 0.5,
                    probability: 1.0,
                    ratchet: 1,
                    micro_shift: 0,
                    active: true,
                },
                TrackerStepConfig {
                    note: 64.0,
                    velocity: 0.8,
                    gate: 0.5,
                    probability: 1.0,
                    ratchet: 1,
                    micro_shift: 0,
                    active: true,
                },
                TrackerStepConfig {
                    note: 67.0,
                    velocity: 0.85,
                    gate: 0.5,
                    probability: 1.0,
                    ratchet: 1,
                    micro_shift: 0,
                    active: true,
                },
            ],
        });

        let serialized = serialize_project_toml(&proj).expect("Serialization failed");
        let parsed = parse_project_toml(&serialized).expect("Deserialization failed");
        assert_eq!(proj, parsed);
    }
}

