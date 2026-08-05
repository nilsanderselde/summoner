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
    #[serde(default = "default_schema_version")]
    pub version: String,
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
    #[serde(default)]
    pub midi_mappings: Vec<MidiMappingConfig>,
    #[serde(default)]
    pub markers: Vec<MarkerConfig>,
    #[serde(default)]
    pub loop_start_beat: f64,
    #[serde(default = "default_loop_end")]
    pub loop_end_beat: f64,
    #[serde(default)]
    pub loop_enabled: bool,
    #[serde(default)]
    pub punch_in_beat: Option<f64>,
    #[serde(default)]
    pub punch_out_beat: Option<f64>,
    #[serde(default)]
    pub locator_a_beat: Option<f64>,
    #[serde(default)]
    pub locator_b_beat: Option<f64>,
    #[serde(default)]
    pub meta: Option<ProjectMetadata>,
    #[serde(default)]
    pub scripts: Vec<LuaScriptConfig>,
    #[serde(default)]
    pub lua_state: Option<String>,
}

/// Step 869: Persistent Lua script configuration stored in project TOML.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct LuaScriptConfig {
    pub name: String,
    pub script: String,
    #[serde(default)]
    pub bound_cc: Option<u8>,
    #[serde(default)]
    pub bound_lane: Option<String>,
    #[serde(default)]
    pub sandbox_fs: bool,
}

/// Step 838: Project tags/genre/BPM/key stored in project TOML [meta] section.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ProjectMetadata {
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub genre: Option<String>,
    #[serde(default)]
    pub bpm: Option<f64>,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

fn default_schema_version() -> String {
    "1.0".to_string()
}

fn default_loop_end() -> f64 {
    16.0
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            name: "New Project".to_string(),
            tuning_file: None,
            transport: TransportConfig::default(),
            tracks: Vec::new(),
            assets: Vec::new(),
            automation_lanes: Vec::new(),
            midi_mappings: Vec::new(),
            markers: Vec::new(),
            loop_start_beat: 0.0,
            loop_end_beat: 16.0,
            loop_enabled: false,
            punch_in_beat: None,
            punch_out_beat: None,
            locator_a_beat: None,
            locator_b_beat: None,
            meta: None,
            scripts: Vec::new(),
            lua_state: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarkerConfig {
    pub name: String,
    pub beat: f64,
    #[serde(default)]
    pub color: Option<[u8; 3]>,
    #[serde(default)]
    pub end_beat: Option<f64>,
    #[serde(default)]
    pub chapter_type: Option<crate::session_markers::ChapterType>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub hotkey_binding: Option<String>,
}

/// MIDI Learn mapping entry in project TOML.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MidiMappingConfig {
    pub channel: u8,
    pub cc: u8,
    pub param_id: String,
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
    #[serde(default)]
    pub master_tune_cents: f32,
    #[serde(default)]
    pub master_trim_db: f32,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            bpm: 120.0,
            time_signature: "4/4".to_string(),
            master_tune_cents: 0.0,
            master_trim_db: 0.0,
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
    pub soloed: bool,
    #[serde(default)]
    pub send_level: f32,
    #[serde(default)]
    pub nodes: Vec<NodeConfig>,
    #[serde(default)]
    pub sequence: Option<SequenceConfig>,
    #[serde(default)]
    pub clips: Vec<SequenceConfig>,
    #[serde(default)]
    pub connections: Vec<ConnectionConfig>,
    #[serde(default)]
    pub tuning_edo: Option<u32>,
    #[serde(default)]
    pub tuning_root_hz: Option<f32>,
    #[serde(default)]
    pub tuning_scl_path: Option<String>,
    #[serde(default)]
    pub collapsed: bool,
    #[serde(default)]
    pub color: Option<[u8; 3]>,
    #[serde(default)]
    pub group_bus: Option<String>,
    #[serde(default)]
    pub record_armed: bool,
    #[serde(default = "default_true")]
    pub send_to_master: bool,
    #[serde(default)]
    pub is_frozen: bool,
    #[serde(default)]
    pub frozen_buffer: Option<Vec<f32>>,
    #[serde(default)]
    pub root_note: u8,
    #[serde(default = "default_scale_type")]
    pub scale_type: String,
    #[serde(default)]
    pub midi_transpose: i8,
    #[serde(default)]
    pub midi_channel_filter: Option<u8>,
    #[serde(default)]
    pub input_echo: bool,
    #[serde(default)]
    pub fine_tune_cents: f32,
    #[serde(default)]
    pub split_key: Option<u8>,
    #[serde(default)]
    pub layer_target_ids: Vec<u64>,
    #[serde(default)]
    pub sidechain_source_track_id: Option<u64>,
    #[serde(default)]
    pub bus_target: Option<String>,
    #[serde(default)]
    pub phase_flip: bool,
    #[serde(default)]
    pub dc_block: bool,
    #[serde(default)]
    pub low_cut_hz: Option<f32>,
    #[serde(default)]
    pub high_cut_hz: Option<f32>,
    #[serde(default)]
    pub input_gain_db: f32,
    #[serde(default)]
    pub output_gain_db: f32,
}

fn default_scale_type() -> String {
    "Major".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for TrackConfig {
    fn default() -> Self {
        Self {
            id: 1,
            name: "Track 1".to_string(),
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
            collapsed: false,
            color: None,
            group_bus: None,
            record_armed: false,
            send_to_master: true,
            is_frozen: false,
            frozen_buffer: None,
            root_note: 0,
            scale_type: "Major".to_string(),
            midi_transpose: 0,
            midi_channel_filter: None,
            input_echo: false,
            fine_tune_cents: 0.0,
            split_key: None,
            layer_target_ids: Vec::new(),
            sidechain_source_track_id: None,
            bus_target: None,
            phase_flip: false,
            dc_block: false,
            low_cut_hz: None,
            high_cut_hz: None,
            input_gain_db: 0.0,
            output_gain_db: 0.0,
        }
    }
}

impl TrackConfig {
    pub fn all_sequences(&self) -> Vec<&SequenceConfig> {
        let mut list = Vec::new();
        if let Some(ref seq) = self.sequence {
            list.push(seq);
        }
        for clip in &self.clips {
            list.push(clip);
        }
        list
    }

    pub fn all_sequences_mut(&mut self) -> Vec<&mut SequenceConfig> {
        let mut list = Vec::new();
        if let Some(ref mut seq) = self.sequence {
            list.push(seq);
        }
        for clip in &mut self.clips {
            list.push(clip);
        }
        list
    }
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

/// Plugin state representation for project TOML session serialization (Step 510).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PluginStateConfig {
    pub plugin_name: String,
    pub plugin_path: String,
    pub format: String,
    #[serde(default)]
    pub is_bypassed: bool,
    #[serde(default)]
    pub state_base64: String,
    #[serde(default)]
    pub parameters: HashMap<String, f32>,
}

/// Node configuration in audio pipeline.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodeConfig {
    pub kind: String,
    #[serde(default)]
    pub params: HashMap<String, f32>,
    #[serde(default)]
    pub plugin_state: Option<PluginStateConfig>,
}

/// Polymetric sequence configuration for a track.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SequenceConfig {
    #[serde(default)]
    pub start_beat: f64,
    #[serde(default = "default_step_division")]
    pub step_division: f64,
    #[serde(default)]
    pub clip_color: Option<[u8; 3]>,
    #[serde(default)]
    pub clip_name: Option<String>,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub is_unique: bool,
    #[serde(default)]
    pub steps: Vec<TrackerStepConfig>,
    #[serde(default)]
    pub fade_in: f64,
    #[serde(default)]
    pub fade_out: f64,
    #[serde(default)]
    pub is_reversed: bool,
    #[serde(default = "default_one_f64")]
    pub time_stretch: f64,
    #[serde(default = "default_one_f64")]
    pub gain: f64,
    #[serde(default)]
    pub pitch_offset: f64,
    #[serde(default)]
    pub trim_start: f64,
    #[serde(default)]
    pub trim_end: f64,
}

fn default_one_f64() -> f64 {
    1.0
}

impl Default for SequenceConfig {
    fn default() -> Self {
        Self {
            start_beat: 0.0,
            step_division: 0.25,
            clip_color: None,
            clip_name: None,
            name: "Sequence".to_string(),
            is_unique: true,
            steps: Vec::new(),
            fade_in: 0.0,
            fade_out: 0.0,
            is_reversed: false,
            time_stretch: 1.0,
            gain: 1.0,
            pitch_offset: 0.0,
            trim_start: 0.0,
            trim_end: 0.0,
        }
    }
}

impl SequenceConfig {
    pub fn restore(&mut self) {
        self.trim_start = 0.0;
        self.trim_end = 0.0;
        self.fade_in = 0.0;
        self.fade_out = 0.0;
        self.gain = 1.0;
        self.pitch_offset = 0.0;
    }
}

impl SequenceConfig {
    pub fn duplicate(&self) -> Self {
        let mut dup = self.clone();
        let clip_len = self.steps.len() as f64 * self.step_division;
        dup.start_beat += if clip_len > 0.0 { clip_len } else { 4.0 };
        if let Some(ref cname) = self.clip_name {
            dup.clip_name = Some(format!("{} (Copy)", cname));
        }
        dup
    }

    pub fn make_unique(&mut self) {
        self.is_unique = true;
    }
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
    #[serde(default)]
    pub swing: f32,
    #[serde(default)]
    pub pan: f32,
    #[serde(default)]
    pub pitch_offset: f32,
    #[serde(default = "default_active")]
    pub active: bool,
    #[serde(default)]
    pub muted: bool,
}

impl Default for TrackerStepConfig {
    fn default() -> Self {
        Self {
            note: 60.0,
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
        }
    }
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
