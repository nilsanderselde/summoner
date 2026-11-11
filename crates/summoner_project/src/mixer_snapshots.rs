// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Mixer Snapshots, Scene Morphing, and A/B Mix Comparison Engine.

use crate::schema::ProjectConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Captured state of a single track in the mixer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MixerTrackState {
    pub track_id: u64,
    pub name: String,
    pub gain: f32,
    pub pan: f32,
    pub muted: bool,
    pub soloed: bool,
    pub send_level: f32,
    pub bus_target: Option<String>,
    pub node_params: HashMap<usize, HashMap<String, f32>>,
}

/// A frozen mixer snapshot / scene.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MixerSnapshot {
    pub id: u32,
    pub name: String,
    pub track_states: HashMap<u64, MixerTrackState>,
}

impl MixerSnapshot {
    pub fn capture(id: u32, name: &str, project: &ProjectConfig) -> Self {
        let mut track_states = HashMap::new();

        for track in &project.tracks {
            let mut node_params = HashMap::new();
            for (node_idx, node) in track.nodes.iter().enumerate() {
                let mut params = HashMap::new();
                for (k, v) in &node.params {
                    params.insert(k.clone(), *v);
                }
                node_params.insert(node_idx, params);
            }

            let state = MixerTrackState {
                track_id: track.id,
                name: track.name.clone(),
                gain: track.gain,
                pan: track.pan,
                muted: track.muted,
                soloed: track.soloed,
                send_level: track.send_level,
                bus_target: track.bus_target.clone(),
                node_params,
            };

            track_states.insert(track.id, state);
        }

        Self {
            id,
            name: name.to_string(),
            track_states,
        }
    }

    pub fn apply(&self, project: &mut ProjectConfig) {
        for track in &mut project.tracks {
            if let Some(state) = self.track_states.get(&track.id) {
                track.gain = state.gain;
                track.pan = state.pan;
                track.muted = state.muted;
                track.soloed = state.soloed;
                track.send_level = state.send_level;
                track.bus_target = state.bus_target.clone();

                for (node_idx, params) in &state.node_params {
                    if let Some(node) = track.nodes.get_mut(*node_idx) {
                        for (k, v) in params {
                            node.params.insert(k.clone(), *v);
                        }
                    }
                }
            }
        }
    }
}

/// Manages multiple mixer snapshots, scene morphing, and A/B comparison.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MixerSnapshotManager {
    pub snapshots: Vec<MixerSnapshot>,
    pub next_id: u32,
    pub ab_snapshot_a: Option<u32>,
    pub ab_snapshot_b: Option<u32>,
    pub active_ab: Option<char>,
}

impl MixerSnapshotManager {
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
            next_id: 1,
            ab_snapshot_a: None,
            ab_snapshot_b: None,
            active_ab: None,
        }
    }

    /// Capture a new mixer snapshot.
    pub fn take_snapshot(&mut self, project: &ProjectConfig, name: &str) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let snapshot = MixerSnapshot::capture(id, name, project);
        self.snapshots.push(snapshot);
        id
    }

    /// Recall a snapshot by ID.
    pub fn recall_snapshot(
        &self,
        project: &mut ProjectConfig,
        snapshot_id: u32,
    ) -> Result<(), String> {
        let snapshot = self
            .snapshots
            .iter()
            .find(|s| s.id == snapshot_id)
            .ok_or_else(|| format!("Snapshot with id {} not found", snapshot_id))?;

        snapshot.apply(project);
        Ok(())
    }

    /// Delete a snapshot by ID.
    pub fn delete_snapshot(&mut self, snapshot_id: u32) -> bool {
        let before = self.snapshots.len();
        self.snapshots.retain(|s| s.id != snapshot_id);
        if self.ab_snapshot_a == Some(snapshot_id) {
            self.ab_snapshot_a = None;
        }
        if self.ab_snapshot_b == Some(snapshot_id) {
            self.ab_snapshot_b = None;
        }
        self.snapshots.len() < before
    }

    /// Assign snapshots A and B for instant comparison.
    pub fn set_ab_compare(&mut self, snapshot_a: u32, snapshot_b: u32) -> Result<(), String> {
        if !self.snapshots.iter().any(|s| s.id == snapshot_a) {
            return Err(format!("Snapshot A ({}) not found", snapshot_a));
        }
        if !self.snapshots.iter().any(|s| s.id == snapshot_b) {
            return Err(format!("Snapshot B ({}) not found", snapshot_b));
        }

        self.ab_snapshot_a = Some(snapshot_a);
        self.ab_snapshot_b = Some(snapshot_b);
        self.active_ab = Some('A');
        Ok(())
    }

    /// Toggle between snapshot A and B and apply to the project.
    pub fn toggle_ab(&mut self, project: &mut ProjectConfig) -> Result<char, String> {
        let (id_a, id_b) = match (self.ab_snapshot_a, self.ab_snapshot_b) {
            (Some(a), Some(b)) => (a, b),
            _ => return Err("A/B snapshots are not fully configured".to_string()),
        };

        let next_state = match self.active_ab {
            Some('A') => 'B',
            _ => 'A',
        };

        let target_id = if next_state == 'A' { id_a } else { id_b };
        self.recall_snapshot(project, target_id)?;
        self.active_ab = Some(next_state);
        Ok(next_state)
    }

    /// Smoothly morph mixer parameters between snapshot A and snapshot B.
    /// `factor` ranges from 0.0 (100% snapshot A) to 1.0 (100% snapshot B).
    pub fn morph_snapshots(
        &self,
        project: &mut ProjectConfig,
        id_a: u32,
        id_b: u32,
        factor: f32,
    ) -> Result<(), String> {
        let snap_a = self
            .snapshots
            .iter()
            .find(|s| s.id == id_a)
            .ok_or_else(|| format!("Snapshot {} not found", id_a))?;
        let snap_b = self
            .snapshots
            .iter()
            .find(|s| s.id == id_b)
            .ok_or_else(|| format!("Snapshot {} not found", id_b))?;

        let factor = factor.clamp(0.0, 1.0);

        for track in &mut project.tracks {
            let state_a = snap_a.track_states.get(&track.id);
            let state_b = snap_b.track_states.get(&track.id);

            match (state_a, state_b) {
                (Some(a), Some(b)) => {
                    // Linear interpolation of continuous mixer values
                    track.gain = a.gain + (b.gain - a.gain) * factor;
                    track.pan = a.pan + (b.pan - a.pan) * factor;
                    track.send_level = a.send_level + (b.send_level - a.send_level) * factor;

                    // Boolean discrete parameters switch at midpoint
                    track.muted = if factor < 0.5 { a.muted } else { b.muted };
                    track.soloed = if factor < 0.5 { a.soloed } else { b.soloed };

                    // Morph plugin numeric parameters
                    for (node_idx, params_a) in &a.node_params {
                        if let Some(params_b) = b.node_params.get(node_idx) {
                            if let Some(node) = track.nodes.get_mut(*node_idx) {
                                for (key, val_a) in params_a {
                                    if let Some(val_b) = params_b.get(key) {
                                        let morphed = val_a + (val_b - val_a) * factor;
                                        node.params.insert(key.clone(), morphed);
                                    }
                                }
                            }
                        }
                    }
                }
                (Some(a), None) => {
                    if factor < 0.5 {
                        track.gain = a.gain;
                        track.pan = a.pan;
                    }
                }
                (None, Some(b)) => {
                    if factor >= 0.5 {
                        track.gain = b.gain;
                        track.pan = b.pan;
                    }
                }
                (None, None) => {}
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{NodeConfig, TrackConfig};

    #[test]
    fn test_mixer_snapshot_capture_recall_and_ab_toggle() {
        let mut track_nodes = Vec::new();
        let mut node_params = HashMap::new();
        node_params.insert("cutoff".to_string(), 1000.0);
        node_params.insert("resonance".to_string(), 0.5);
        track_nodes.push(NodeConfig {
            kind: "FilterNode".to_string(),
            params: node_params,
            plugin_state: None,
        });

        let mut project = ProjectConfig {
            tracks: vec![TrackConfig {
                id: 1,
                name: "Lead Synth".to_string(),
                gain: 0.8,
                pan: -0.2,
                nodes: track_nodes,
                ..Default::default()
            }],
            ..Default::default()
        };

        let mut mgr = MixerSnapshotManager::new();

        // Snapshot A: original state
        let snap_a = mgr.take_snapshot(&project, "Clean Mix");

        // Modify mix for Snapshot B
        project.tracks[0].gain = 0.5;
        project.tracks[0].pan = 0.4;
        project.tracks[0].nodes[0]
            .params
            .insert("cutoff".to_string(), 3500.0);
        let snap_b = mgr.take_snapshot(&project, "Brighter Mix");

        assert_eq!(mgr.snapshots.len(), 2);

        // Recall Snapshot A
        mgr.recall_snapshot(&mut project, snap_a).unwrap();
        assert!((project.tracks[0].gain - 0.8).abs() < 1e-4);
        assert!((project.tracks[0].pan - (-0.2)).abs() < 1e-4);
        assert!((project.tracks[0].nodes[0].params["cutoff"] - 1000.0).abs() < 1e-4);

        // Configure A/B compare
        mgr.set_ab_compare(snap_a, snap_b).unwrap();
        assert_eq!(mgr.active_ab, Some('A'));

        // Toggle to B
        let toggled = mgr.toggle_ab(&mut project).unwrap();
        assert_eq!(toggled, 'B');
        assert!((project.tracks[0].gain - 0.5).abs() < 1e-4);
        assert!((project.tracks[0].pan - 0.4).abs() < 1e-4);
        assert!((project.tracks[0].nodes[0].params["cutoff"] - 3500.0).abs() < 1e-4);

        // Toggle back to A
        let toggled_back = mgr.toggle_ab(&mut project).unwrap();
        assert_eq!(toggled_back, 'A');
        assert!((project.tracks[0].gain - 0.8).abs() < 1e-4);
        assert!((project.tracks[0].pan - (-0.2)).abs() < 1e-4);
    }

    #[test]
    fn test_mixer_snapshot_morphing() {
        let mut node_params = HashMap::new();
        node_params.insert("cutoff".to_string(), 1000.0);

        let mut project = ProjectConfig {
            tracks: vec![TrackConfig {
                id: 1,
                name: "Bass".to_string(),
                gain: 0.2,
                pan: -0.6,
                nodes: vec![NodeConfig {
                    kind: "Filter".to_string(),
                    params: node_params,
                    plugin_state: None,
                }],
                ..Default::default()
            }],
            ..Default::default()
        };

        let mut mgr = MixerSnapshotManager::new();
        let snap_a = mgr.take_snapshot(&project, "A");

        project.tracks[0].gain = 0.8;
        project.tracks[0].pan = 0.4;
        project.tracks[0].nodes[0]
            .params
            .insert("cutoff".to_string(), 5000.0);
        let snap_b = mgr.take_snapshot(&project, "B");

        // Morph 50% between A and B
        mgr.morph_snapshots(&mut project, snap_a, snap_b, 0.5)
            .unwrap();

        // Gain midpoint: 0.2 + (0.8 - 0.2) * 0.5 = 0.5
        assert!((project.tracks[0].gain - 0.5).abs() < 1e-4);
        // Pan midpoint: -0.6 + (0.4 - (-0.6)) * 0.5 = -0.1
        assert!((project.tracks[0].pan - (-0.1)).abs() < 1e-4);
        // Cutoff midpoint: 1000 + 4000 * 0.5 = 3000
        assert!((project.tracks[0].nodes[0].params["cutoff"] - 3000.0).abs() < 1e-4);
    }
}
