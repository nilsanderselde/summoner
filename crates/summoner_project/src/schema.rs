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

//! Declarative session configuration schema for `.toml` project files.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root project session definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectConfig {
    pub name: String,
    pub transport: TransportConfig,
    #[serde(default)]
    pub tracks: Vec<TrackConfig>,
}

/// Transport settings in project document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransportConfig {
    pub sample_rate: u32,
    pub bpm: f64,
    pub time_signature: String,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            bpm: 120.0,
            time_signature: "4/4".to_string(),
        }
    }
}

/// Individual track layout definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrackConfig {
    pub id: u64,
    pub name: String,
    #[serde(default = "default_channels")]
    pub channels: usize,
    #[serde(default = "default_gain")]
    pub gain: f32,
    #[serde(default)]
    pub pan: f32,
    #[serde(default)]
    pub muted: bool,
    #[serde(default)]
    pub nodes: Vec<NodeConfig>,
    #[serde(default)]
    pub sequence: Option<SequenceConfig>,
}

fn default_channels() -> usize {
    2
}

fn default_gain() -> f32 {
    1.0
}

/// Node configuration in audio pipeline.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodeConfig {
    pub kind: String,
    #[serde(default)]
    pub params: HashMap<String, f32>,
}

/// Polymetric sequence configuration for a track.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SequenceConfig {
    #[serde(default = "default_step_division")]
    pub step_division: f64,
    #[serde(default)]
    pub steps: Vec<TrackerStepConfig>,
}

fn default_step_division() -> f64 {
    0.25
}

/// Tracker step configuration in project document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrackerStepConfig {
    pub note: f64,
    pub velocity: f32,
    pub gate: f32,
    #[serde(default = "default_probability")]
    pub probability: f32,
    #[serde(default = "default_active")]
    pub active: bool,
}

fn default_probability() -> f32 {
    1.0
}

fn default_active() -> bool {
    true
}

