//! State Source of Truth and Ephemeral GUI State Audit (Task K3 per AGENTS.md).
//!
//! Enforces the fundamental architectural invariant:
//! - `summoner_project::ProjectConfig` is the ABSOLUTE and ONLY source of semantic truth.
//! - The GUI (`summoner_gui`) is strictly a deterministic, ephemeral projection.
//! - The GUI is strictly forbidden from holding non-ephemeral semantic state.
//! - All user mutations (track creation, volume/pan, clip arrangement, automation, tempo)
//!   MUST be dispatched through [`ProjectMutationDispatcher`] directly into [`ProjectConfig`].
//! - All GUI transient state (zoom, scroll, hover, selection, dialog modals) is strictly
//!   ephemeral and safely discarded on session reset without loss of musical data.

use serde::{Deserialize, Serialize};
use std::fmt;
use summoner_project::audio_clip::AudioClipConfig;
use summoner_project::schema::{
    AutomationEventConfig, AutomationLaneConfig, MarkerConfig, ProjectConfig, TrackConfig,
};

/// Transient, ephemeral GUI-only state that is NEVER persisted in the project document.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct GuiEphemeralState {
    pub scroll_x: f32,
    pub scroll_y: f32,
    pub zoom_level: f32,
    pub selected_track_id: Option<u64>,
    pub selected_clip_id: Option<String>,
    pub hovered_control_id: Option<String>,
    pub is_dragging: bool,
    pub drag_origin: Option<[f32; 2]>,
    pub active_dialog: Option<String>,
    pub tooltip_text: Option<String>,
    pub playhead_interpolation: f64,
}

impl GuiEphemeralState {
    pub fn new() -> Self {
        Self {
            scroll_x: 0.0,
            scroll_y: 0.0,
            zoom_level: 1.0,
            selected_track_id: None,
            selected_clip_id: None,
            hovered_control_id: None,
            is_dragging: false,
            drag_origin: None,
            active_dialog: None,
            tooltip_text: None,
            playhead_interpolation: 0.0,
        }
    }

    /// Reset all ephemeral GUI state without touching semantic project data.
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

/// Explicit, strongly-typed semantic mutations routed directly to `ProjectConfig`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProjectMutation {
    // Track mutations
    AddTrack {
        id: u64,
        name: String,
        channels: usize,
    },
    RemoveTrack {
        track_id: u64,
    },
    SetTrackGain {
        track_id: u64,
        gain: f32,
    },
    SetTrackPan {
        track_id: u64,
        pan: f32,
    },
    SetTrackMute {
        track_id: u64,
        muted: bool,
    },
    SetTrackSolo {
        track_id: u64,
        soloed: bool,
    },
    SetTrackRecordArm {
        track_id: u64,
        armed: bool,
    },

    // Clip mutations
    AddAudioClip {
        track_id: u64,
        clip: AudioClipConfig,
    },
    MoveAudioClip {
        track_id: u64,
        clip_id: String,
        new_start_beat: f64,
    },
    RemoveAudioClip {
        track_id: u64,
        clip_id: String,
    },

    // Transport & Global mutations
    SetBpm {
        bpm: f64,
    },
    SetTimeSignature {
        signature: String,
    },
    SetLoopRegion {
        start_beat: f64,
        end_beat: f64,
        enabled: bool,
    },
    AddMarker {
        marker: MarkerConfig,
    },
    RemoveMarker {
        index: usize,
    },

    // Automation mutations
    AddAutomationLane {
        lane: AutomationLaneConfig,
    },
    AddAutomationEvent {
        track_id: u64,
        param_id: String,
        frame: u64,
        value: f32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MutationError {
    TrackNotFound(u64),
    ClipNotFound(String),
    MarkerIndexOutOfBounds(usize),
    InvalidValue(String),
}

impl fmt::Display for MutationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TrackNotFound(id) => write!(f, "Track not found: {}", id),
            Self::ClipNotFound(id) => write!(f, "Clip not found: {}", id),
            Self::MarkerIndexOutOfBounds(idx) => write!(f, "Marker index out of bounds: {}", idx),
            Self::InvalidValue(msg) => write!(f, "Invalid parameter value: {}", msg),
        }
    }
}

impl std::error::Error for MutationError {}

/// Unified mutation dispatcher and undo manager ensuring `ProjectConfig` remains source of truth.
pub struct ProjectMutationDispatcher {
    undo_stack: Vec<ProjectMutation>,
    redo_stack: Vec<ProjectMutation>,
}

impl Default for ProjectMutationDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectMutationDispatcher {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Dispatch and apply a semantic mutation to `ProjectConfig`.
    pub fn dispatch(
        &mut self,
        project: &mut ProjectConfig,
        mutation: ProjectMutation,
    ) -> Result<(), MutationError> {
        let inverse = Self::compute_inverse(project, &mutation)?;
        Self::apply_mutation_internal(project, mutation)?;
        if let Some(inv) = inverse {
            self.undo_stack.push(inv);
            self.redo_stack.clear();
        }
        Ok(())
    }

    /// Undo last dispatched mutation.
    pub fn undo(&mut self, project: &mut ProjectConfig) -> Result<bool, MutationError> {
        if let Some(inv) = self.undo_stack.pop() {
            let redo_inv = Self::compute_inverse(project, &inv)?;
            Self::apply_mutation_internal(project, inv)?;
            if let Some(r) = redo_inv {
                self.redo_stack.push(r);
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Redo last undone mutation.
    pub fn redo(&mut self, project: &mut ProjectConfig) -> Result<bool, MutationError> {
        if let Some(mutation) = self.redo_stack.pop() {
            let undo_inv = Self::compute_inverse(project, &mutation)?;
            Self::apply_mutation_internal(project, mutation)?;
            if let Some(u) = undo_inv {
                self.undo_stack.push(u);
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Direct internal mutation application to `ProjectConfig`.
    fn apply_mutation_internal(
        project: &mut ProjectConfig,
        mutation: ProjectMutation,
    ) -> Result<(), MutationError> {
        match mutation {
            ProjectMutation::AddTrack { id, name, channels } => {
                let track = TrackConfig {
                    id,
                    name,
                    channels,
                    ..TrackConfig::default()
                };
                project.tracks.push(track);
            }
            ProjectMutation::RemoveTrack { track_id } => {
                if let Some(pos) = project.tracks.iter().position(|t| t.id == track_id) {
                    project.tracks.remove(pos);
                } else {
                    return Err(MutationError::TrackNotFound(track_id));
                }
            }
            ProjectMutation::SetTrackGain { track_id, gain } => {
                let track = project
                    .tracks
                    .iter_mut()
                    .find(|t| t.id == track_id)
                    .ok_or(MutationError::TrackNotFound(track_id))?;
                track.gain = gain.clamp(0.0, 4.0);
            }
            ProjectMutation::SetTrackPan { track_id, pan } => {
                let track = project
                    .tracks
                    .iter_mut()
                    .find(|t| t.id == track_id)
                    .ok_or(MutationError::TrackNotFound(track_id))?;
                track.pan = pan.clamp(-1.0, 1.0);
            }
            ProjectMutation::SetTrackMute { track_id, muted } => {
                let track = project
                    .tracks
                    .iter_mut()
                    .find(|t| t.id == track_id)
                    .ok_or(MutationError::TrackNotFound(track_id))?;
                track.muted = muted;
            }
            ProjectMutation::SetTrackSolo { track_id, soloed } => {
                let track = project
                    .tracks
                    .iter_mut()
                    .find(|t| t.id == track_id)
                    .ok_or(MutationError::TrackNotFound(track_id))?;
                track.soloed = soloed;
            }
            ProjectMutation::SetTrackRecordArm { track_id, armed } => {
                let track = project
                    .tracks
                    .iter_mut()
                    .find(|t| t.id == track_id)
                    .ok_or(MutationError::TrackNotFound(track_id))?;
                track.record_armed = armed;
            }
            ProjectMutation::AddAudioClip { track_id, clip } => {
                let track = project
                    .tracks
                    .iter_mut()
                    .find(|t| t.id == track_id)
                    .ok_or(MutationError::TrackNotFound(track_id))?;
                track.audio_clips.push(clip);
            }
            ProjectMutation::MoveAudioClip {
                track_id,
                clip_id,
                new_start_beat,
            } => {
                let track = project
                    .tracks
                    .iter_mut()
                    .find(|t| t.id == track_id)
                    .ok_or(MutationError::TrackNotFound(track_id))?;
                let clip = track
                    .audio_clips
                    .iter_mut()
                    .find(|c| c.id == clip_id)
                    .ok_or_else(|| MutationError::ClipNotFound(clip_id.clone()))?;
                clip.start_beat = new_start_beat.max(0.0);
            }
            ProjectMutation::RemoveAudioClip { track_id, clip_id } => {
                let track = project
                    .tracks
                    .iter_mut()
                    .find(|t| t.id == track_id)
                    .ok_or(MutationError::TrackNotFound(track_id))?;
                if let Some(pos) = track.audio_clips.iter().position(|c| c.id == clip_id) {
                    track.audio_clips.remove(pos);
                } else {
                    return Err(MutationError::ClipNotFound(clip_id));
                }
            }
            ProjectMutation::SetBpm { bpm } => {
                if bpm <= 0.0 || !bpm.is_finite() {
                    return Err(MutationError::InvalidValue(
                        "BPM must be positive".to_string(),
                    ));
                }
                project.transport.bpm = bpm;
            }
            ProjectMutation::SetTimeSignature { signature } => {
                project.transport.time_signature = signature;
            }
            ProjectMutation::SetLoopRegion {
                start_beat,
                end_beat,
                enabled,
            } => {
                project.loop_start_beat = start_beat.max(0.0);
                project.loop_end_beat = end_beat.max(project.loop_start_beat);
                project.loop_enabled = enabled;
            }
            ProjectMutation::AddMarker { marker } => {
                project.markers.push(marker);
            }
            ProjectMutation::RemoveMarker { index } => {
                if index < project.markers.len() {
                    project.markers.remove(index);
                } else {
                    return Err(MutationError::MarkerIndexOutOfBounds(index));
                }
            }
            ProjectMutation::AddAutomationLane { lane } => {
                project.automation_lanes.push(lane);
            }
            ProjectMutation::AddAutomationEvent {
                track_id,
                param_id,
                frame,
                value,
            } => {
                if let Some(lane) = project
                    .automation_lanes
                    .iter_mut()
                    .find(|l| l.track_id == track_id && l.param_id == param_id)
                {
                    lane.events.push(AutomationEventConfig { frame, value });
                } else {
                    let mut lane = AutomationLaneConfig {
                        track_id,
                        param_id,
                        events: Vec::new(),
                    };
                    lane.events.push(AutomationEventConfig { frame, value });
                    project.automation_lanes.push(lane);
                }
            }
        }
        Ok(())
    }

    /// Compute reverse mutation for undo history.
    fn compute_inverse(
        project: &ProjectConfig,
        mutation: &ProjectMutation,
    ) -> Result<Option<ProjectMutation>, MutationError> {
        match mutation {
            ProjectMutation::AddTrack { id, .. } => {
                Ok(Some(ProjectMutation::RemoveTrack { track_id: *id }))
            }
            ProjectMutation::RemoveTrack { track_id } => {
                let track = project
                    .tracks
                    .iter()
                    .find(|t| t.id == *track_id)
                    .ok_or(MutationError::TrackNotFound(*track_id))?;
                Ok(Some(ProjectMutation::AddTrack {
                    id: track.id,
                    name: track.name.clone(),
                    channels: track.channels,
                }))
            }
            ProjectMutation::SetTrackGain { track_id, .. } => {
                let track = project
                    .tracks
                    .iter()
                    .find(|t| t.id == *track_id)
                    .ok_or(MutationError::TrackNotFound(*track_id))?;
                Ok(Some(ProjectMutation::SetTrackGain {
                    track_id: *track_id,
                    gain: track.gain,
                }))
            }
            ProjectMutation::SetTrackPan { track_id, .. } => {
                let track = project
                    .tracks
                    .iter()
                    .find(|t| t.id == *track_id)
                    .ok_or(MutationError::TrackNotFound(*track_id))?;
                Ok(Some(ProjectMutation::SetTrackPan {
                    track_id: *track_id,
                    pan: track.pan,
                }))
            }
            ProjectMutation::SetTrackMute { track_id, .. } => {
                let track = project
                    .tracks
                    .iter()
                    .find(|t| t.id == *track_id)
                    .ok_or(MutationError::TrackNotFound(*track_id))?;
                Ok(Some(ProjectMutation::SetTrackMute {
                    track_id: *track_id,
                    muted: track.muted,
                }))
            }
            ProjectMutation::SetTrackSolo { track_id, .. } => {
                let track = project
                    .tracks
                    .iter()
                    .find(|t| t.id == *track_id)
                    .ok_or(MutationError::TrackNotFound(*track_id))?;
                Ok(Some(ProjectMutation::SetTrackSolo {
                    track_id: *track_id,
                    soloed: track.soloed,
                }))
            }
            ProjectMutation::SetTrackRecordArm { track_id, .. } => {
                let track = project
                    .tracks
                    .iter()
                    .find(|t| t.id == *track_id)
                    .ok_or(MutationError::TrackNotFound(*track_id))?;
                Ok(Some(ProjectMutation::SetTrackRecordArm {
                    track_id: *track_id,
                    armed: track.record_armed,
                }))
            }
            ProjectMutation::AddAudioClip { track_id, clip } => {
                Ok(Some(ProjectMutation::RemoveAudioClip {
                    track_id: *track_id,
                    clip_id: clip.id.clone(),
                }))
            }
            ProjectMutation::MoveAudioClip {
                track_id, clip_id, ..
            } => {
                let track = project
                    .tracks
                    .iter()
                    .find(|t| t.id == *track_id)
                    .ok_or(MutationError::TrackNotFound(*track_id))?;
                let clip = track
                    .audio_clips
                    .iter()
                    .find(|c| c.id == *clip_id)
                    .ok_or_else(|| MutationError::ClipNotFound(clip_id.clone()))?;
                Ok(Some(ProjectMutation::MoveAudioClip {
                    track_id: *track_id,
                    clip_id: clip_id.clone(),
                    new_start_beat: clip.start_beat,
                }))
            }
            ProjectMutation::RemoveAudioClip { track_id, clip_id } => {
                let track = project
                    .tracks
                    .iter()
                    .find(|t| t.id == *track_id)
                    .ok_or(MutationError::TrackNotFound(*track_id))?;
                let clip = track
                    .audio_clips
                    .iter()
                    .find(|c| c.id == *clip_id)
                    .ok_or_else(|| MutationError::ClipNotFound(clip_id.clone()))?;
                Ok(Some(ProjectMutation::AddAudioClip {
                    track_id: *track_id,
                    clip: clip.clone(),
                }))
            }
            ProjectMutation::SetBpm { .. } => Ok(Some(ProjectMutation::SetBpm {
                bpm: project.transport.bpm,
            })),
            ProjectMutation::SetTimeSignature { .. } => {
                Ok(Some(ProjectMutation::SetTimeSignature {
                    signature: project.transport.time_signature.clone(),
                }))
            }
            ProjectMutation::SetLoopRegion { .. } => Ok(Some(ProjectMutation::SetLoopRegion {
                start_beat: project.loop_start_beat,
                end_beat: project.loop_end_beat,
                enabled: project.loop_enabled,
            })),
            ProjectMutation::AddMarker { .. } => {
                let idx = project.markers.len();
                Ok(Some(ProjectMutation::RemoveMarker { index: idx }))
            }
            ProjectMutation::RemoveMarker { index } => {
                let marker = project
                    .markers
                    .get(*index)
                    .ok_or(MutationError::MarkerIndexOutOfBounds(*index))?;
                Ok(Some(ProjectMutation::AddMarker {
                    marker: marker.clone(),
                }))
            }
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gui_ephemeral_state_isolation() {
        let mut ui_state = GuiEphemeralState::new();
        ui_state.scroll_x = 240.0;
        ui_state.zoom_level = 1.8;
        ui_state.selected_track_id = Some(42);
        ui_state.is_dragging = true;

        // Resetting GUI state clears ephemeral fields completely
        ui_state.reset();
        assert_eq!(ui_state.scroll_x, 0.0);
        assert_eq!(ui_state.zoom_level, 1.0);
        assert_eq!(ui_state.selected_track_id, None);
        assert!(!ui_state.is_dragging);
    }

    #[test]
    fn test_project_mutations_and_undo_redo_roundtrip() {
        let mut project = ProjectConfig::default();
        let mut dispatcher = ProjectMutationDispatcher::new();

        // 1. Add track
        dispatcher
            .dispatch(
                &mut project,
                ProjectMutation::AddTrack {
                    id: 101,
                    name: "Bassline".to_string(),
                    channels: 2,
                },
            )
            .unwrap();
        assert_eq!(project.tracks.len(), 1);
        assert_eq!(project.tracks[0].name, "Bassline");

        // 2. Tweak volume & pan
        dispatcher
            .dispatch(
                &mut project,
                ProjectMutation::SetTrackGain {
                    track_id: 101,
                    gain: 0.75,
                },
            )
            .unwrap();
        dispatcher
            .dispatch(
                &mut project,
                ProjectMutation::SetTrackPan {
                    track_id: 101,
                    pan: -0.25,
                },
            )
            .unwrap();
        assert_eq!(project.tracks[0].gain, 0.75);
        assert_eq!(project.tracks[0].pan, -0.25);

        // 3. Test Undo
        assert!(dispatcher.undo(&mut project).unwrap()); // Reverts pan
        assert_eq!(project.tracks[0].pan, 0.0);

        assert!(dispatcher.undo(&mut project).unwrap()); // Reverts gain
        assert_eq!(project.tracks[0].gain, 1.0);

        // 4. Test Redo
        assert!(dispatcher.redo(&mut project).unwrap()); // Restores gain
        assert_eq!(project.tracks[0].gain, 0.75);

        assert!(dispatcher.redo(&mut project).unwrap()); // Restores pan
        assert_eq!(project.tracks[0].pan, -0.25);
    }

    #[test]
    fn test_project_clip_and_transport_mutations() {
        let mut project = ProjectConfig::default();
        let mut dispatcher = ProjectMutationDispatcher::new();

        dispatcher
            .dispatch(
                &mut project,
                ProjectMutation::AddTrack {
                    id: 1,
                    name: "Drums".to_string(),
                    channels: 2,
                },
            )
            .unwrap();

        let clip = AudioClipConfig::new("clip_intro", "Intro Beat", "asset_drum", 0.0, 8.0);

        dispatcher
            .dispatch(
                &mut project,
                ProjectMutation::AddAudioClip { track_id: 1, clip },
            )
            .unwrap();
        assert_eq!(project.tracks[0].audio_clips.len(), 1);

        // Move clip
        dispatcher
            .dispatch(
                &mut project,
                ProjectMutation::MoveAudioClip {
                    track_id: 1,
                    clip_id: "clip_intro".to_string(),
                    new_start_beat: 4.0,
                },
            )
            .unwrap();
        assert_eq!(project.tracks[0].audio_clips[0].start_beat, 4.0);

        // Set BPM
        dispatcher
            .dispatch(&mut project, ProjectMutation::SetBpm { bpm: 128.0 })
            .unwrap();
        assert_eq!(project.transport.bpm, 128.0);

        // Undo move
        dispatcher.undo(&mut project).unwrap(); // reverts BPM
        dispatcher.undo(&mut project).unwrap(); // reverts move
        assert_eq!(project.tracks[0].audio_clips[0].start_beat, 0.0);
    }
}
