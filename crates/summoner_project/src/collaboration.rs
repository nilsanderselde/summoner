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

//! Real-Time Multi-Seat Collaboration and Cloud Session Sync Engine (Task I4).
//!
//! Provides real-time cooperative editing, peer presence tracking, and lock-free
//! session diff merging using CRDTs:
//! - [`CollaborationSessionManager`]: Orchestrates session sync, delta generation, and remote op application.
//! - [`PeerPresence`]: Multi-user avatar color, transport playhead position, selection, and viewport presence.
//! - [`CollaborationPacket`]: Wire protocol packet carrying peer presence and causal CRDT ops.
//! - [`ProjectDiffResult`]: Summary of remote changes applied to the active project session.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::crdt::{CrdtEngine, CrdtOp, OfflineChangeQueue};
use crate::schema::ProjectConfig;

/// Connection and synchronization state of the collaboration session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CollaborationConnectionStatus {
    Disconnected,
    Connecting,
    Connected { peer_count: usize },
    Syncing,
    Reconnecting,
}

/// Remote collaborator seat presence state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PeerPresence {
    pub peer_id: String,
    pub user_name: String,
    /// RGBA avatar / cursor indicator color.
    pub color_rgba: [u8; 4],
    /// Current playhead position in beats.
    pub playhead_beat: f64,
    /// ID of the track currently selected or being edited by this peer.
    pub selected_track_id: Option<u64>,
    /// Active view name (e.g. "Arranger", "Mixer", "PianoRoll").
    pub active_view: String,
    /// Timestamp of last heartbeat in seconds.
    pub last_heartbeat_timestamp: f64,
}

impl PeerPresence {
    pub fn new(peer_id: impl Into<String>, user_name: impl Into<String>, color_rgba: [u8; 4]) -> Self {
        Self {
            peer_id: peer_id.into(),
            user_name: user_name.into(),
            color_rgba,
            playhead_beat: 0.0,
            selected_track_id: None,
            active_view: "Arranger".to_string(),
            last_heartbeat_timestamp: 0.0,
        }
    }
}

/// Network message exchanged between collaborator seats.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CollaborationPacket {
    pub session_id: String,
    pub sender_presence: PeerPresence,
    pub delta_ops: Vec<CrdtOp>,
    pub timestamp_sec: f64,
}

/// Results of merging incoming remote operations into the active project.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ProjectDiffResult {
    pub bpm_changed: Option<f64>,
    pub updated_tracks: Vec<u64>,
    pub added_tracks: Vec<u64>,
    pub removed_tracks: Vec<u64>,
    pub total_ops_applied: usize,
}

/// Orchestrator for real-time collaboration sessions.
#[derive(Debug, Clone)]
pub struct CollaborationSessionManager {
    pub session_id: String,
    pub local_peer_id: String,
    pub local_presence: PeerPresence,
    pub status: CollaborationConnectionStatus,
    pub crdt_engine: CrdtEngine,
    pub offline_queue: OfflineChangeQueue,
    pub active_peers: HashMap<String, PeerPresence>,
    last_broadcast_op_count: usize,
    last_project_snapshot: Option<ProjectConfig>,
}

impl CollaborationSessionManager {
    pub fn new(session_id: impl Into<String>, local_peer_id: impl Into<String>, user_name: impl Into<String>) -> Self {
        let peer_id = local_peer_id.into();
        let name = user_name.into();
        let local_presence = PeerPresence::new(&peer_id, &name, [66, 133, 244, 255]); // Summoner blue
        let crdt_engine = CrdtEngine::new(&peer_id);

        Self {
            session_id: session_id.into(),
            local_peer_id: peer_id,
            local_presence,
            status: CollaborationConnectionStatus::Disconnected,
            crdt_engine,
            offline_queue: OfflineChangeQueue::default(),
            active_peers: HashMap::new(),
            last_broadcast_op_count: 0,
            last_project_snapshot: None,
        }
    }

    /// Set connection status.
    pub fn set_status(&mut self, status: CollaborationConnectionStatus) {
        self.status = status;
    }

    /// Update local user's presence (playhead, selection, view).
    pub fn update_local_presence(
        &mut self,
        playhead_beat: f64,
        selected_track_id: Option<u64>,
        active_view: &str,
        now_sec: f64,
    ) {
        self.local_presence.playhead_beat = playhead_beat;
        self.local_presence.selected_track_id = selected_track_id;
        self.local_presence.active_view = active_view.to_string();
        self.local_presence.last_heartbeat_timestamp = now_sec;
    }

    /// Detect mutations between the previous project snapshot and the current project state,
    /// recording them as atomic CRDT operations.
    pub fn compute_project_diff(&mut self, current: &ProjectConfig) -> Vec<CrdtOp> {
        let mut diff_ops = Vec::new();

        if let Some(prev) = &self.last_project_snapshot {
            // 1. Detect BPM changes
            if (prev.transport.bpm - current.transport.bpm).abs() > 1e-4 {
                diff_ops.push(CrdtOp::SetBpm {
                    bpm: current.transport.bpm as f32,
                    clock: 0,
                });
            }

            // 2. Detect added tracks
            for curr_track in &current.tracks {
                if !prev.tracks.iter().any(|t| t.id == curr_track.id) {
                    diff_ops.push(CrdtOp::AddTrack {
                        track_id: curr_track.id,
                        name: curr_track.name.clone(),
                        clock: 0,
                    });
                }
            }

            // 3. Detect removed tracks
            for prev_track in &prev.tracks {
                if !current.tracks.iter().any(|t| t.id == prev_track.id) {
                    diff_ops.push(CrdtOp::RemoveTrack {
                        track_id: prev_track.id,
                        clock: 0,
                    });
                }
            }

            // 4. Detect track parameter adjustments (gain, pan, mute)
            for curr_track in &current.tracks {
                if let Some(prev_track) = prev.tracks.iter().find(|t| t.id == curr_track.id) {
                    if (prev_track.gain - curr_track.gain).abs() > 1e-4 {
                        diff_ops.push(CrdtOp::SetTrackGain {
                            track_id: curr_track.id,
                            gain: curr_track.gain,
                            clock: 0,
                        });
                    }
                    if (prev_track.pan - curr_track.pan).abs() > 1e-4 {
                        diff_ops.push(CrdtOp::SetTrackPan {
                            track_id: curr_track.id,
                            pan: curr_track.pan,
                            clock: 0,
                        });
                    }
                    if prev_track.muted != curr_track.muted {
                        diff_ops.push(CrdtOp::SetTrackMuted {
                            track_id: curr_track.id,
                            muted: curr_track.muted,
                            clock: 0,
                        });
                    }
                }
            }
        }

        // Apply detected ops to local CRDT engine
        for op in &diff_ops {
            self.crdt_engine.apply_op(op.clone());
        }

        self.last_project_snapshot = Some(current.clone());
        diff_ops
    }

    /// Prepare an outbound network packet containing new delta operations and local presence.
    pub fn prepare_outbound_packet(&mut self, now_sec: f64) -> CollaborationPacket {
        let total_ops = self.crdt_engine.ops.len();
        let delta_ops = if total_ops > self.last_broadcast_op_count {
            self.crdt_engine.ops[self.last_broadcast_op_count..total_ops].to_vec()
        } else {
            Vec::new()
        };
        self.last_broadcast_op_count = total_ops;

        CollaborationPacket {
            session_id: self.session_id.clone(),
            sender_presence: self.local_presence.clone(),
            delta_ops,
            timestamp_sec: now_sec,
        }
    }

    /// Ingest an incoming network packet from a peer:
    /// updates their presence, merges their CRDT ops, and mutates the project.
    pub fn process_inbound_packet(
        &mut self,
        packet: CollaborationPacket,
        project: &mut ProjectConfig,
    ) -> ProjectDiffResult {
        if packet.session_id != self.session_id || packet.sender_presence.peer_id == self.local_peer_id {
            return ProjectDiffResult::default();
        }

        // 1. Update peer presence
        self.active_peers.insert(packet.sender_presence.peer_id.clone(), packet.sender_presence);

        // 2. Merge incoming CRDT delta ops
        let mut diff_result = ProjectDiffResult::default();
        for op in packet.delta_ops {
            if !self.crdt_engine.ops.contains(&op) {
                match &op {
                    CrdtOp::SetBpm { bpm, .. } => {
                        diff_result.bpm_changed = Some(*bpm as f64);
                    }
                    CrdtOp::SetTrackGain { track_id, .. }
                    | CrdtOp::SetTrackPan { track_id, .. }
                    | CrdtOp::SetTrackMuted { track_id, .. } => {
                        if !diff_result.updated_tracks.contains(track_id) {
                            diff_result.updated_tracks.push(*track_id);
                        }
                    }
                    CrdtOp::AddTrack { track_id, .. }
                        if !diff_result.added_tracks.contains(track_id) => {
                        diff_result.added_tracks.push(*track_id);
                    }
                    CrdtOp::RemoveTrack { track_id, .. }
                        if !diff_result.removed_tracks.contains(track_id) => {
                        diff_result.removed_tracks.push(*track_id);
                    }
                    _ => {}
                }
                self.crdt_engine.ops.push(op);
                diff_result.total_ops_applied += 1;
            }
        }

        // 3. Apply state onto ProjectConfig
        self.crdt_engine.apply_to_project(project);
        self.last_project_snapshot = Some(project.clone());

        diff_result
    }

    /// Purge inactive peers that have timed out (no heartbeat within timeout window).
    pub fn purge_timed_out_peers(&mut self, now_sec: f64, timeout_sec: f64) -> Vec<String> {
        let mut timed_out = Vec::new();
        self.active_peers.retain(|id, presence| {
            let keep = (now_sec - presence.last_heartbeat_timestamp) < timeout_sec;
            if !keep {
                timed_out.push(id.clone());
            }
            keep
        });
        timed_out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collaboration_session_init_and_presence() {
        let mut manager = CollaborationSessionManager::new("session_alpha", "user_1", "Alice");
        assert_eq!(manager.session_id, "session_alpha");
        assert_eq!(manager.local_peer_id, "user_1");

        manager.update_local_presence(16.0, Some(2), "Mixer", 10.0);
        assert_eq!(manager.local_presence.playhead_beat, 16.0);
        assert_eq!(manager.local_presence.selected_track_id, Some(2));
        assert_eq!(manager.local_presence.active_view, "Mixer");
        assert_eq!(manager.local_presence.last_heartbeat_timestamp, 10.0);
    }

    #[test]
    fn test_project_diff_and_packet_exchange() {
        let mut alice_mgr = CollaborationSessionManager::new("session_alpha", "alice", "Alice");
        let mut bob_mgr = CollaborationSessionManager::new("session_alpha", "bob", "Bob");

        let mut alice_proj = ProjectConfig::default();
        let mut bob_proj = ProjectConfig::default();

        // Initial snapshot
        alice_mgr.compute_project_diff(&alice_proj);
        bob_mgr.compute_project_diff(&bob_proj);

        // Alice modifies project: changes BPM to 140 and adds a track
        alice_proj.transport.bpm = 140.0;
        let diff = alice_mgr.compute_project_diff(&alice_proj);
        assert_eq!(diff.len(), 1);

        // Alice prepares packet and sends to Bob
        let packet = alice_mgr.prepare_outbound_packet(1.0);
        assert_eq!(packet.delta_ops.len(), 1);

        // Bob receives and processes Alice's packet
        let diff_result = bob_mgr.process_inbound_packet(packet, &mut bob_proj);
        assert_eq!(diff_result.bpm_changed, Some(140.0));
        assert_eq!(bob_proj.transport.bpm, 140.0);
        assert_eq!(bob_mgr.active_peers.len(), 1);
        assert!(bob_mgr.active_peers.contains_key("alice"));
    }

    #[test]
    fn test_peer_heartbeat_timeout() {
        let mut manager = CollaborationSessionManager::new("session_alpha", "alice", "Alice");
        let peer_bob = PeerPresence {
            peer_id: "bob".to_string(),
            user_name: "Bob".to_string(),
            color_rgba: [255, 0, 0, 255],
            playhead_beat: 4.0,
            selected_track_id: None,
            active_view: "Arranger".to_string(),
            last_heartbeat_timestamp: 100.0,
        };
        manager.active_peers.insert("bob".to_string(), peer_bob);

        // At time 105.0 with timeout 10.0 -> still active
        let purged = manager.purge_timed_out_peers(105.0, 10.0);
        assert!(purged.is_empty());
        assert_eq!(manager.active_peers.len(), 1);

        // At time 115.0 with timeout 10.0 -> timed out!
        let purged = manager.purge_timed_out_peers(115.0, 10.0);
        assert_eq!(purged, vec!["bob".to_string()]);
        assert!(manager.active_peers.is_empty());
    }
}
