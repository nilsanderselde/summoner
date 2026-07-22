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
    #[serde(default)]
    pub tuning_file: Option<String>,
    pub transport: TransportConfig,
    #[serde(default)]
    pub tracks: Vec<TrackConfig>,
    #[serde(default)]
    pub assets: Vec<AssetConfig>,
    #[serde(default)]
    pub automation_lanes: Vec<AutomationLaneConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutomationEventConfig {
    pub frame: u64,
    pub value: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutomationLaneConfig {
    pub param_id: String,
    pub track_id: u64,
    pub events: Vec<AutomationEventConfig>,
}

/// Asset descriptor in project document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssetConfig {
    pub id: String,
    pub hash: String,
    pub path: String,
    #[serde(default)]
    pub auto_slice: bool,
    #[serde(default = "default_slice_threshold")]
    pub slice_threshold: f32,
}

fn default_slice_threshold() -> f32 {
    0.15
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
    #[serde(default)]
    pub connections: Vec<ConnectionConfig>,
}

/// Routing connection configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConnectionConfig {
    pub from: String,
    pub to: String,
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
    #[serde(default = "default_ratchet")]
    pub ratchet: u32,
    #[serde(default = "default_micro_shift")]
    pub micro_shift: i32,
    #[serde(default = "default_active")]
    pub active: bool,
}

fn default_probability() -> f32 {
    1.0
}

fn default_active() -> bool {
    true
}

fn default_ratchet() -> u32 {
    1
}

fn default_micro_shift() -> i32 {
    0
}

