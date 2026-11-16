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

use crate::audio_clip::{ArrangerLaneConfig, AudioClipConfig};
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
    #[serde(default)]
    pub sidechain_routes: Vec<SidechainRouteConfig>,
    #[serde(default)]
    pub reference_track_id: Option<u64>,
    #[serde(default)]
    pub reference_ab_active: bool,
    #[serde(default)]
    pub k_system_scale: Option<String>,
}

/// Solo listening mode for a mixer channel (MIX-4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SoloMode {
    #[default]
    SoloInPlace,
    Pfl,
    Afl,
}

impl SoloMode {
    /// Short badge label for channel strips.
    pub fn label(self) -> &'static str {
        match self {
            Self::SoloInPlace => "SIP",
            Self::Pfl => "PFL",
            Self::Afl => "AFL",
        }
    }

    /// Cycle through Solo-in-Place, PFL, and AFL modes.
    pub fn next(self) -> Self {
        match self {
            Self::SoloInPlace => Self::Pfl,
            Self::Pfl => Self::Afl,
            Self::Afl => Self::SoloInPlace,
        }
    }
}

fn default_sc_threshold() -> f32 {
    -18.0
}

fn default_sc_ratio() -> f32 {
    4.0
}

fn default_sc_attack() -> f32 {
    5.0
}

fn default_sc_release() -> f32 {
    120.0
}

fn default_stereo_width() -> f32 {
    1.0
}

/// Declarative sidechain route between tracks/devices (MIX-2).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SidechainRouteConfig {
    pub source_track_id: u64,
    pub dest_track_id: u64,
    #[serde(default)]
    pub dest_node_idx: Option<usize>,
    #[serde(default = "default_gain")]
    pub send_gain: f32,
    #[serde(default)]
    pub is_pre_fader: bool,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub hpf_hz: f32,
    #[serde(default)]
    pub lpf_hz: f32,
    #[serde(default = "default_sc_threshold")]
    pub threshold_db: f32,
    #[serde(default = "default_sc_ratio")]
    pub ratio: f32,
    #[serde(default = "default_sc_attack")]
    pub attack_ms: f32,
    #[serde(default = "default_sc_release")]
    pub release_ms: f32,
}

impl Default for SidechainRouteConfig {
    fn default() -> Self {
        Self {
            source_track_id: 1,
            dest_track_id: 2,
            dest_node_idx: None,
            send_gain: 1.0,
            is_pre_fader: false,
            enabled: true,
            hpf_hz: 80.0,
            lpf_hz: 16000.0,
            threshold_db: -18.0,
            ratio: 4.0,
            attack_ms: 5.0,
            release_ms: 120.0,
        }
    }
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
            sidechain_routes: Vec::new(),
            reference_track_id: None,
            reference_ab_active: false,
            k_system_scale: None,
        }
    }
}

impl ProjectConfig {
    /// Migrate or parse legacy project TOML into the current schema version (1.1.0).
    pub fn migrate_schema(raw_toml: &str) -> Result<Self, String> {
        match toml::from_str::<ProjectConfig>(raw_toml) {
            Ok(mut proj) => {
                if proj.version == "1.0" {
                    proj.version = "1.1.0".to_string();
                }
                Ok(proj)
            }
            Err(_) => {
                let mut val: toml::Value = toml::from_str(raw_toml).map_err(|e| e.to_string())?;
                if let Some(table) = val.as_table_mut() {
                    table.insert(
                        "version".to_string(),
                        toml::Value::String("1.1.0".to_string()),
                    );
                    if let Some(tracks) = table.get_mut("tracks").and_then(|t| t.as_array_mut()) {
                        for track in tracks {
                            if let Some(t_table) = track.as_table_mut() {
                                if !t_table.contains_key("audio_clips") {
                                    t_table.insert(
                                        "audio_clips".to_string(),
                                        toml::Value::Array(Vec::new()),
                                    );
                                }
                                if !t_table.contains_key("arranger_lanes") {
                                    t_table.insert(
                                        "arranger_lanes".to_string(),
                                        toml::Value::Array(Vec::new()),
                                    );
                                }
                            }
                        }
                    }
                }
                val.try_into().map_err(|e: toml::de::Error| e.to_string())
            }
        }
    }

    /// Record a parameter automation point onto `track_id` / `param_id` according to the track's `AutomationMode`.
    /// Returns `true` if a point was written, or `false` if the track is missing or in `Read`/`Off` mode.
    pub fn record_automation_point(
        &mut self,
        track_id: u64,
        param_id: &str,
        frame: u64,
        value: f32,
    ) -> bool {
        let Some(mode) = self
            .tracks
            .iter()
            .find(|t| t.id == track_id)
            .map(|t| t.automation_mode)
        else {
            return false;
        };

        match mode {
            AutomationMode::Read | AutomationMode::Off => return false,
            AutomationMode::Touch
            | AutomationMode::Latch
            | AutomationMode::Write
            | AutomationMode::Trim => {}
        }

        let lane_idx = self
            .automation_lanes
            .iter()
            .position(|l| l.track_id == track_id && l.param_id == param_id)
            .unwrap_or_else(|| {
                self.automation_lanes.push(AutomationLaneConfig {
                    param_id: param_id.to_string(),
                    track_id,
                    events: Vec::new(),
                });
                self.automation_lanes.len() - 1
            });

        let lane = &mut self.automation_lanes[lane_idx];
        lane.events.sort_by_key(|e| e.frame);

        let recorded_value = if mode == AutomationMode::Trim {
            evaluate_automation_lane_at_frame(&lane.events, frame).unwrap_or(0.0) + value
        } else {
            value
        };

        if let Some(existing) = lane.events.iter_mut().find(|e| e.frame == frame) {
            existing.value = recorded_value;
        } else {
            let pos = lane.events.partition_point(|e| e.frame < frame);
            lane.events.insert(
                pos,
                AutomationEventConfig {
                    frame,
                    value: recorded_value,
                },
            );
        }

        true
    }

    /// Simplify redundant nearly-collinear breakpoints on the specified automation lane using
    /// Ramer-Douglas-Peucker thinning with vertical tolerance `epsilon`.
    /// Returns the number of events removed.
    pub fn thin_automation_lane(&mut self, track_id: u64, param_id: &str, epsilon: f32) -> usize {
        let Some(lane) = self
            .automation_lanes
            .iter_mut()
            .find(|l| l.track_id == track_id && l.param_id == param_id)
        else {
            return 0;
        };

        if lane.events.len() <= 2 {
            return 0;
        }

        lane.events.sort_by_key(|e| e.frame);
        let original_len = lane.events.len();
        let eps = epsilon.max(0.0) as f64;
        let mut keep = vec![false; original_len];
        keep[0] = true;
        keep[original_len - 1] = true;
        rdp_mark_kept(&lane.events, 0, original_len - 1, eps, &mut keep);

        let mut idx = 0usize;
        lane.events.retain(|_| {
            let k = keep[idx];
            idx += 1;
            k
        });
        original_len - lane.events.len()
    }

    /// Return the next unused numeric track ID (`max + 1`).
    pub fn next_track_id(&self) -> u64 {
        self.tracks.iter().map(|t| t.id).max().unwrap_or(0) + 1
    }

    /// Create and append a subgroup bus track (`MIX-1`), returning its assigned track ID.
    pub fn create_bus_track(&mut self, name: &str) -> u64 {
        let id = self.next_track_id();
        self.tracks.push(TrackConfig {
            id,
            name: name.to_string(),
            is_bus_track: true,
            solo_safe: true,
            ..Default::default()
        });
        id
    }

    /// Create and append an FX return track (`MIX-1`), returning its assigned track ID.
    pub fn create_return_track(&mut self, name: &str) -> u64 {
        let id = self.next_track_id();
        self.tracks.push(TrackConfig {
            id,
            name: name.to_string(),
            is_return_track: true,
            solo_safe: true,
            ..Default::default()
        });
        id
    }

    /// Create a folder stack track (`HUM-4`) and attach the specified child tracks to it.
    pub fn create_folder_stack(&mut self, name: &str, child_track_ids: &[u64]) -> u64 {
        let folder_id = self.next_track_id();
        for tr in &mut self.tracks {
            if child_track_ids.contains(&tr.id) {
                tr.parent_folder_id = Some(folder_id);
            }
        }
        self.tracks.push(TrackConfig {
            id: folder_id,
            name: name.to_string(),
            is_folder_track: true,
            ..Default::default()
        });
        folder_id
    }

    /// Create a VCA master fader track (`MIX-4`) controlling `target_track_ids`.
    pub fn create_vca_track(&mut self, name: &str, target_track_ids: &[u64]) -> u64 {
        let id = self.next_track_id();
        self.tracks.push(TrackConfig {
            id,
            name: name.to_string(),
            is_vca: true,
            vca_target_ids: target_track_ids.to_vec(),
            ..Default::default()
        });
        id
    }

    /// Add or update an auxiliary send on `source_track_id` targeting `target_return_id` (`MIX-1`).
    pub fn add_track_send(
        &mut self,
        source_track_id: u64,
        target_return_id: u64,
        amount: f32,
        is_pre_fader: bool,
    ) -> bool {
        let Some(track) = self.tracks.iter_mut().find(|t| t.id == source_track_id) else {
            return false;
        };
        if let Some(existing) = track
            .sends
            .iter_mut()
            .find(|s| s.target_track_id == target_return_id)
        {
            existing.amount = amount;
            existing.is_pre_fader = is_pre_fader;
            existing.muted = false;
        } else {
            track.sends.push(TrackSendConfig {
                target_track_id: target_return_id,
                amount,
                is_pre_fader,
                muted: false,
            });
        }
        true
    }

    /// Insert or update a sidechain routing connection (`MIX-2`).
    pub fn upsert_sidechain_route(&mut self, route: SidechainRouteConfig) {
        if let Some(existing) = self.sidechain_routes.iter_mut().find(|r| {
            r.source_track_id == route.source_track_id
                && r.dest_track_id == route.dest_track_id
                && r.dest_node_idx == route.dest_node_idx
        }) {
            *existing = route;
        } else {
            self.sidechain_routes.push(route);
        }
    }

    /// Filter visible track IDs by folder collapse state (empty query) or search text (`HUM-4`).
    pub fn filter_tracks(&self, query: &str) -> Vec<u64> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            let collapsed_folders: Vec<u64> = self
                .tracks
                .iter()
                .filter(|t| t.is_folder_track && t.collapsed)
                .map(|t| t.id)
                .collect();
            return self
                .tracks
                .iter()
                .filter(|t| {
                    t.parent_folder_id
                        .is_none_or(|pid| !collapsed_folders.contains(&pid))
                })
                .map(|t| t.id)
                .collect();
        }
        self.tracks
            .iter()
            .filter(|t| {
                t.name.to_lowercase().contains(&q)
                    || t.comments.to_lowercase().contains(&q)
                    || t.icon
                        .as_deref()
                        .is_some_and(|i| i.to_lowercase().contains(&q))
                    || t.bus_target
                        .as_deref()
                        .is_some_and(|b| b.to_lowercase().contains(&q))
                    || t.group_bus
                        .as_deref()
                        .is_some_and(|g| g.to_lowercase().contains(&q))
                    || t.nodes.iter().any(|n| n.kind.to_lowercase().contains(&q))
            })
            .map(|t| t.id)
            .collect()
    }
}

fn evaluate_automation_lane_at_frame(events: &[AutomationEventConfig], frame: u64) -> Option<f32> {
    if events.is_empty() {
        return None;
    }
    if frame <= events[0].frame {
        return Some(events[0].value);
    }
    let last = events.last()?;
    if frame >= last.frame {
        return Some(last.value);
    }
    let idx = events.partition_point(|e| e.frame <= frame);
    let a = &events[idx - 1];
    let b = &events[idx];
    if b.frame <= a.frame {
        return Some(b.value);
    }
    let t = (frame - a.frame) as f32 / (b.frame - a.frame) as f32;
    Some(a.value + (b.value - a.value) * t)
}

fn rdp_mark_kept(
    events: &[AutomationEventConfig],
    start: usize,
    end: usize,
    epsilon: f64,
    keep: &mut [bool],
) {
    if end <= start + 1 {
        return;
    }
    let f0 = events[start].frame as f64;
    let v0 = events[start].value as f64;
    let f1 = events[end].frame as f64;
    let v1 = events[end].value as f64;
    let df = f1 - f0;

    let mut max_err = -1.0f64;
    let mut split_idx = start;
    for (i, ev) in events.iter().enumerate().take(end).skip(start + 1) {
        let interp = if df > 0.0 {
            let t = (ev.frame as f64 - f0) / df;
            v0 + (v1 - v0) * t
        } else {
            v0
        };
        let err = (ev.value as f64 - interp).abs();
        if err > max_err {
            max_err = err;
            split_idx = i;
        }
    }

    if max_err > epsilon && split_idx > start && split_idx < end {
        keep[split_idx] = true;
        rdp_mark_kept(events, start, split_idx, epsilon, keep);
        rdp_mark_kept(events, split_idx, end, epsilon, keep);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
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

/// Controller mapping operating mode (Task B3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ControllerMappingMode {
    #[default]
    Absolute,
    RelativeTwosComplement,
    RelativeSignedBit,
    RelativeBinOffset,
    Toggle,
    Momentary,
}

fn default_min_val() -> f32 {
    0.0
}

fn default_max_val() -> f32 {
    1.0
}

/// MIDI Learn mapping entry in project TOML (Task B3).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MidiMappingConfig {
    pub channel: u8,
    pub cc: u8,
    pub param_id: String,
    #[serde(default)]
    pub track_id: Option<u64>,
    #[serde(default)]
    pub mode: ControllerMappingMode,
    #[serde(default = "default_min_val")]
    pub min_val: f32,
    #[serde(default = "default_max_val")]
    pub max_val: f32,
    #[serde(default)]
    pub soft_takeover: bool,
    #[serde(default)]
    pub device_name: Option<String>,
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

/// Tempo point configuration stored in project TOML.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TempoPointConfig {
    pub beat: f64,
    pub bpm: f64,
    #[serde(default)]
    pub curve: String,
    #[serde(default)]
    pub target_bpm: Option<f64>,
    #[serde(default)]
    pub ramp_length_beats: f64,
}

/// Time signature / meter point configuration stored in project TOML.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct MeterPointConfig {
    pub beat: f64,
    pub numerator: u8,
    pub denominator: u8,
}

/// Key and scale change event on the master harmonic track.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct KeyPointConfig {
    pub beat: f64,
    /// Root pitch class `0..11` (`0 = C .. 11 = B`).
    #[serde(default)]
    pub root_note: u8,
    #[serde(default = "default_scale_type")]
    pub scale_type: String,
}

/// Chord event on the master chord track for harmonic lock and accompaniment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ChordEventConfig {
    pub beat: f64,
    pub duration_beats: f64,
    /// Chord root pitch class `0..11` (`0 = C .. 11 = B`).
    #[serde(default)]
    pub root_note: u8,
    /// Chord quality symbol (e.g. `"Maj"`, `"min"`, `"dim"`, `"aug"`, `"dom7"`, `"min7"`).
    #[serde(default = "default_chord_quality")]
    pub quality: String,
    #[serde(default)]
    pub bass_note: Option<u8>,
}

fn default_chord_quality() -> String {
    "Maj".to_string()
}

/// Track automation read/write mode (MIX-3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AutomationMode {
    #[default]
    Read,
    Touch,
    Latch,
    Write,
    Trim,
    Off,
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
    #[serde(default)]
    pub tempo_points: Vec<TempoPointConfig>,
    #[serde(default)]
    pub meter_points: Vec<MeterPointConfig>,
    #[serde(default)]
    pub key_points: Vec<KeyPointConfig>,
    #[serde(default)]
    pub chord_track: Vec<ChordEventConfig>,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            bpm: 120.0,
            time_signature: "4/4".to_string(),
            master_tune_cents: 0.0,
            master_trim_db: 0.0,
            tempo_points: Vec::new(),
            meter_points: Vec::new(),
            key_points: Vec::new(),
            chord_track: Vec::new(),
        }
    }
}

impl TransportConfig {
    /// Convert project transport configuration into live core `TempoMap`.
    pub fn to_tempo_map(&self) -> summoner_core::TempoMap {
        let mut map = summoner_core::TempoMap::new(self.bpm, 4, 4, self.sample_rate);
        if let Some((num_str, den_str)) = self.time_signature.split_once('/') {
            if let (Ok(num), Ok(den)) = (num_str.trim().parse::<u8>(), den_str.trim().parse::<u8>())
            {
                map.meter_points[0] = summoner_core::MeterPoint::new(0.0, num, den);
            }
        }
        for tp in &self.tempo_points {
            if tp.curve.eq_ignore_ascii_case("ramp") {
                map.add_tempo_point(summoner_core::TempoPoint::ramp(
                    tp.beat,
                    tp.bpm,
                    tp.target_bpm.unwrap_or(tp.bpm),
                    tp.ramp_length_beats,
                ));
            } else {
                map.add_tempo_point(summoner_core::TempoPoint::jump(tp.beat, tp.bpm));
            }
        }
        for mp in &self.meter_points {
            map.add_meter_point(summoner_core::MeterPoint::new(
                mp.beat,
                mp.numerator,
                mp.denominator,
            ));
        }
        map
    }

    /// Resolve active harmonic key `(root_note, scale_type)` at `beat`.
    /// Defaults to `(0, "Major")` (C Major) if no key point precedes `beat`.
    pub fn active_key_at_beat(&self, beat: f64) -> (u8, &str) {
        let mut active: Option<&KeyPointConfig> = None;
        for kp in &self.key_points {
            if kp.beat <= beat + 1e-9 && active.is_none_or(|cur| kp.beat >= cur.beat) {
                active = Some(kp);
            }
        }
        match active {
            Some(kp) => (kp.root_note % 12, kp.scale_type.as_str()),
            None => (0, "Major"),
        }
    }

    /// Compute the signed shortest-interval semitone offset (`-5..=6`) from the active key root
    /// to the active chord root spanning `beat`, or `0` when no chord spans `beat`.
    pub fn active_chord_offset_at_beat(&self, beat: f64) -> i32 {
        let mut active: Option<&ChordEventConfig> = None;
        for ch in &self.chord_track {
            let end_beat = ch.beat + ch.duration_beats.max(0.0);
            if ch.beat <= beat + 1e-9
                && beat < end_beat - 1e-9
                && active.is_none_or(|cur| ch.beat >= cur.beat)
            {
                active = Some(ch);
            }
        }
        let Some(chord) = active else {
            return 0;
        };
        let (key_root, _) = self.active_key_at_beat(beat);
        let raw = ((chord.root_note % 12) as i32 - (key_root % 12) as i32).rem_euclid(12);
        if raw > 6 {
            raw - 12
        } else {
            raw
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
    pub audio_clips: Vec<AudioClipConfig>,
    #[serde(default)]
    pub arranger_lanes: Vec<ArrangerLaneConfig>,
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
    #[serde(default)]
    pub input_monitoring: bool,
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
    #[serde(default)]
    pub sends: Vec<TrackSendConfig>,
    #[serde(default)]
    pub is_return_track: bool,
    #[serde(default)]
    pub is_bus_track: bool,
    #[serde(default)]
    pub is_vca: bool,
    #[serde(default)]
    pub vca_target_ids: Vec<u64>,
    #[serde(default)]
    pub vca_group: Option<u32>,
    #[serde(default)]
    pub mute_group: Option<u32>,
    #[serde(default)]
    pub solo_safe: bool,
    #[serde(default)]
    pub sidechain_audition: bool,
    #[serde(default)]
    pub hardware_input_channels: Option<(usize, usize)>,
    #[serde(default)]
    pub hardware_output_channels: Option<(usize, usize)>,
    #[serde(default)]
    pub scale_lock: bool,
    #[serde(default)]
    pub follow_chord_track: bool,
    #[serde(default)]
    pub automation_mode: AutomationMode,
    #[serde(default)]
    pub is_folder_track: bool,
    #[serde(default)]
    pub parent_folder_id: Option<u64>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub comments: String,
    #[serde(default)]
    pub pan_law: summoner_core::panner::PanLaw,
    #[serde(default = "default_stereo_width")]
    pub stereo_width: f32,
    #[serde(default)]
    pub solo_mode: SoloMode,
}

/// Auxiliary send routing configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrackSendConfig {
    pub target_track_id: u64,
    pub amount: f32,
    #[serde(default)]
    pub is_pre_fader: bool,
    #[serde(default)]
    pub muted: bool,
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
            audio_clips: Vec::new(),
            arranger_lanes: Vec::new(),
            connections: Vec::new(),
            tuning_edo: None,
            tuning_root_hz: None,
            tuning_scl_path: None,
            collapsed: false,
            color: None,
            group_bus: None,
            record_armed: false,
            input_monitoring: false,
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
            sends: Vec::new(),
            is_return_track: false,
            is_bus_track: false,
            is_vca: false,
            vca_target_ids: Vec::new(),
            vca_group: None,
            mute_group: None,
            solo_safe: false,
            sidechain_audition: false,
            hardware_input_channels: None,
            hardware_output_channels: None,
            scale_lock: false,
            follow_chord_track: false,
            automation_mode: AutomationMode::default(),
            is_folder_track: false,
            parent_folder_id: None,
            icon: None,
            comments: String::new(),
            pan_law: summoner_core::panner::PanLaw::default(),
            stereo_width: 1.0,
            solo_mode: SoloMode::default(),
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

    pub fn add_audio_clip(&mut self, clip: AudioClipConfig) {
        self.audio_clips.push(clip);
    }

    pub fn remove_audio_clip(&mut self, clip_id: &str) -> Option<AudioClipConfig> {
        if let Some(pos) = self.audio_clips.iter().position(|c| c.id == clip_id) {
            Some(self.audio_clips.remove(pos))
        } else {
            None
        }
    }

    pub fn audio_clips_at_beat(&self, beat: f64) -> Vec<&AudioClipConfig> {
        self.audio_clips
            .iter()
            .filter(|c| c.contains_beat(beat))
            .collect()
    }

    pub fn audio_clips_in_range(&self, start_beat: f64, end_beat: f64) -> Vec<&AudioClipConfig> {
        self.audio_clips
            .iter()
            .filter(|c| c.start_beat < end_beat && c.end_beat() > start_beat)
            .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_audio_clips_management() {
        let mut track = TrackConfig::default();
        assert!(track.audio_clips.is_empty());

        let clip1 = AudioClipConfig::new("c1", "Take 1", "asset1", 0.0, 4.0);
        let clip2 = AudioClipConfig::new("c2", "Take 2", "asset2", 4.0, 4.0);

        track.add_audio_clip(clip1);
        track.add_audio_clip(clip2);
        assert_eq!(track.audio_clips.len(), 2);

        let active = track.audio_clips_at_beat(2.0);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, "c1");

        let in_range = track.audio_clips_in_range(2.0, 6.0);
        assert_eq!(in_range.len(), 2);

        let removed = track.remove_audio_clip("c1");
        assert!(removed.is_some());
        assert_eq!(track.audio_clips.len(), 1);
    }

    #[test]
    fn test_project_schema_migration_v1_0_to_v1_1() {
        let legacy_toml = r#"
name = "Legacy Project"
version = "1.0"

[transport]
sample_rate = 48000
bpm = 128.0
time_signature = "4/4"

[[tracks]]
id = 1
name = "Audio 1"
"#;
        let migrated =
            ProjectConfig::migrate_schema(legacy_toml).expect("Migration should succeed");
        assert_eq!(migrated.version, "1.1.0");
        assert_eq!(migrated.tracks.len(), 1);
        assert!(migrated.tracks[0].audio_clips.is_empty());
        assert!(migrated.tracks[0].arranger_lanes.is_empty());
    }

    #[test]
    fn test_transport_config_to_tempo_map() {
        let mut transport = TransportConfig {
            sample_rate: 48000,
            bpm: 130.0,
            time_signature: "7/8".to_string(),
            ..Default::default()
        };
        transport.tempo_points.push(TempoPointConfig {
            beat: 16.0,
            bpm: 150.0,
            curve: "Jump".to_string(),
            target_bpm: None,
            ramp_length_beats: 0.0,
        });
        transport.meter_points.push(MeterPointConfig {
            beat: 16.0,
            numerator: 4,
            denominator: 4,
        });

        let map = transport.to_tempo_map();
        assert_eq!(map.sample_rate, 48000);
        assert_eq!(map.tempo_at_beat(0.0), 130.0);
        assert_eq!(map.meter_at_beat(0.0), (7, 8));
        assert_eq!(map.tempo_at_beat(16.0), 150.0);
        assert_eq!(map.meter_at_beat(16.0), (4, 4));
    }

    #[test]
    fn test_key_and_chord_track_resolution() {
        let mut transport = TransportConfig::default();
        assert_eq!(transport.active_key_at_beat(0.0), (0, "Major"));
        assert_eq!(transport.active_chord_offset_at_beat(2.0), 0);

        transport.key_points.push(KeyPointConfig {
            beat: 0.0,
            root_note: 0, // C
            scale_type: "Major".to_string(),
        });
        transport.key_points.push(KeyPointConfig {
            beat: 8.0,
            root_note: 9, // A
            scale_type: "Minor".to_string(),
        });
        transport.chord_track.push(ChordEventConfig {
            beat: 4.0,
            duration_beats: 4.0,
            root_note: 5, // F (+5 from C)
            quality: "Maj".to_string(),
            bass_note: None,
        });
        transport.chord_track.push(ChordEventConfig {
            beat: 8.0,
            duration_beats: 4.0,
            root_note: 4, // E (-5 from A)
            quality: "min".to_string(),
            bass_note: None,
        });

        assert_eq!(transport.active_key_at_beat(4.0), (0, "Major"));
        assert_eq!(transport.active_chord_offset_at_beat(5.0), 5);
        assert_eq!(transport.active_key_at_beat(9.0), (9, "Minor"));
        assert_eq!(transport.active_chord_offset_at_beat(9.0), -5);
        assert_eq!(transport.active_chord_offset_at_beat(12.0), 0);
    }

    #[test]
    fn test_automation_mode_record_and_thin() {
        let mut project = ProjectConfig::default();
        project.tracks.push(TrackConfig {
            id: 1,
            automation_mode: AutomationMode::Read,
            ..Default::default()
        });

        assert!(!project.record_automation_point(1, "track.gain", 0, 0.5));
        project.tracks[0].automation_mode = AutomationMode::Write;
        assert!(project.record_automation_point(1, "track.gain", 0, 0.0));
        assert!(project.record_automation_point(1, "track.gain", 100, 0.5));
        assert!(project.record_automation_point(1, "track.gain", 200, 1.0));
        assert!(project.record_automation_point(1, "track.gain", 300, 1.0));
        assert!(project.record_automation_point(1, "track.gain", 400, 0.2));

        project.tracks[0].automation_mode = AutomationMode::Trim;
        assert!(project.record_automation_point(1, "track.gain", 300, -0.25));
        let lane = &project.automation_lanes[0];
        let pt_300 = lane.events.iter().find(|e| e.frame == 300).unwrap();
        assert!((pt_300.value - 0.75).abs() < 1e-6);

        // Linear point at frame 100 (exact 0.5 on 0->200 segment) should be thinned cleanly
        project.tracks[0].automation_mode = AutomationMode::Write;
        assert!(project.record_automation_point(1, "track.gain", 300, 1.0));
        let removed = project.thin_automation_lane(1, "track.gain", 0.01);
        assert_eq!(removed, 1);
        assert_eq!(project.automation_lanes[0].events.len(), 4);
    }

    #[test]
    fn test_bus_return_folder_sidechain_and_filter_helpers() {
        let mut project = ProjectConfig::default();
        project.tracks.push(TrackConfig {
            id: 1,
            name: "Kick".to_string(),
            comments: "Punchy 909".to_string(),
            ..Default::default()
        });
        project.tracks.push(TrackConfig {
            id: 2,
            name: "Bass".to_string(),
            ..Default::default()
        });

        let bus_id = project.create_bus_track("Drum Bus");
        let ret_id = project.create_return_track("Reverb");
        let folder_id = project.create_folder_stack("Low End", &[1, 2]);
        let vca_id = project.create_vca_track("All Drums", &[1]);

        assert!(
            project
                .tracks
                .iter()
                .find(|t| t.id == bus_id)
                .unwrap()
                .is_bus_track
        );
        assert!(
            project
                .tracks
                .iter()
                .find(|t| t.id == ret_id)
                .unwrap()
                .is_return_track
        );
        assert!(
            project
                .tracks
                .iter()
                .find(|t| t.id == vca_id)
                .unwrap()
                .is_vca
        );
        assert_eq!(
            project
                .tracks
                .iter()
                .find(|t| t.id == 1)
                .unwrap()
                .parent_folder_id,
            Some(folder_id)
        );

        assert!(project.add_track_send(1, ret_id, 0.4, true));
        assert!(!project.add_track_send(999, ret_id, 0.4, false));

        project.upsert_sidechain_route(SidechainRouteConfig {
            source_track_id: 1,
            dest_track_id: 2,
            threshold_db: -24.0,
            ..Default::default()
        });
        assert_eq!(project.sidechain_routes.len(), 1);
        project.upsert_sidechain_route(SidechainRouteConfig {
            source_track_id: 1,
            dest_track_id: 2,
            threshold_db: -12.0,
            ..Default::default()
        });
        assert_eq!(project.sidechain_routes.len(), 1);
        assert_eq!(project.sidechain_routes[0].threshold_db, -12.0);

        // Collapse folder and verify filter_tracks hides children on empty query
        project
            .tracks
            .iter_mut()
            .find(|t| t.id == folder_id)
            .unwrap()
            .collapsed = true;
        let visible = project.filter_tracks("");
        assert!(!visible.contains(&1));
        assert!(!visible.contains(&2));
        assert!(visible.contains(&folder_id));

        // Searching by comment finds Track 1 even when collapsed
        assert_eq!(project.filter_tracks("909"), vec![1]);
    }
}
