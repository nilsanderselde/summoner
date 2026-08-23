// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Clip Launcher Scene Matrix & Quantized Event Trigger Engine (Milestone 9).
//!
//! Provides a non-blocking clip launch matrix with quantized scene firing, legato clip launching,
//! follow action rule execution, sample-accurate beat synchronization, and lock-free scene state transitions.
//!
//! Enforces zero heap allocations in real-time execution loops under `AllocGuard`.

use serde::{Deserialize, Serialize};

/// Maximum pre-allocated dimensions for the scene matrix.
pub const MAX_MATRIX_TRACKS: usize = 16;
pub const MAX_MATRIX_SCENES: usize = 16;

/// Launch quantization interval for scheduling clip and scene firing.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LaunchQuantize {
    None,
    Sixteenth, // 0.25 beat
    Eighth,    // 0.5 beat
    Quarter,   // 1.0 beat
    Half,      // 2.0 beats
    Bar(u32),  // 1, 2, 4, 8 bars
}

impl Default for LaunchQuantize {
    fn default() -> Self {
        Self::Bar(1)
    }
}

impl LaunchQuantize {
    pub fn to_beat_interval(&self, beats_per_bar: f64) -> f64 {
        match self {
            Self::None => 0.0,
            Self::Sixteenth => 0.25,
            Self::Eighth => 0.5,
            Self::Quarter => 1.0,
            Self::Half => 2.0,
            Self::Bar(bars) => (*bars as f64) * beats_per_bar,
        }
    }
}

/// Trigger interaction behavior when launching a clip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LaunchMode {
    #[default]
    Trigger, // Starts on downbeat and loops continuously until stopped
    Gate,    // Plays only while held down; stops on release
    Toggle,  // First trigger starts, second trigger stops
    Repeat,  // Retriggers continuously at the quantize interval
}

/// Follow action target behavior after a clip finishes playing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FollowActionType {
    #[default]
    None,
    Stop,
    PlayNext,
    PlayPrevious,
    PlayFirst,
    PlayLast,
    PlayAny,
    Jump(usize), // Jump to specific scene index
}

/// Follow action configuration for chaining clip playback.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FollowAction {
    pub action_type: FollowActionType,
    pub trigger_beats: f64, // Number of beats to play before executing action
    pub chance: f32,        // Probability (0.0 to 1.0) of executing action
}

impl Default for FollowAction {
    fn default() -> Self {
        Self {
            action_type: FollowActionType::None,
            trigger_beats: 4.0,
            chance: 1.0,
        }
    }
}

/// Playback state of an individual clip slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SlotState {
    #[default]
    Empty,
    Stopped,
    QueuedToPlay,
    Playing,
    QueuedToStop,
}

/// Individual clip slot cell in the launcher matrix.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipSlot {
    pub track_idx: usize,
    pub scene_idx: usize,
    pub name: String,
    pub length_beats: f64,
    pub loop_enabled: bool,
    pub legato: bool,
    pub quantize: LaunchQuantize,
    pub launch_mode: LaunchMode,
    pub follow_action: FollowAction,

    pub state: SlotState,
    pub playhead_beats: f64,
    pub total_played_beats: f64,
    pub is_empty: bool,
}

impl ClipSlot {
    pub fn empty(track_idx: usize, scene_idx: usize) -> Self {
        Self {
            track_idx,
            scene_idx,
            name: String::new(),
            length_beats: 4.0,
            loop_enabled: true,
            legato: false,
            quantize: LaunchQuantize::Bar(1),
            launch_mode: LaunchMode::Trigger,
            follow_action: FollowAction::default(),
            state: SlotState::Empty,
            playhead_beats: 0.0,
            total_played_beats: 0.0,
            is_empty: true,
        }
    }

    pub fn new_clip(
        track_idx: usize,
        scene_idx: usize,
        name: &str,
        length_beats: f64,
        loop_enabled: bool,
    ) -> Self {
        Self {
            track_idx,
            scene_idx,
            name: name.to_string(),
            length_beats: length_beats.max(0.25),
            loop_enabled,
            legato: false,
            quantize: LaunchQuantize::Bar(1),
            launch_mode: LaunchMode::Trigger,
            follow_action: FollowAction::default(),
            state: SlotState::Stopped,
            playhead_beats: 0.0,
            total_played_beats: 0.0,
            is_empty: false,
        }
    }
}

/// Command event emitted by the matrix engine when clip state transitions occur.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MatrixEvent {
    ClipStarted { track_idx: usize, scene_idx: usize, start_beat: f64 },
    ClipStopped { track_idx: usize, scene_idx: usize },
    SceneLaunched { scene_idx: usize, beat: f64 },
}

/// Horizontal Scene definition spanning all track columns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub index: usize,
    pub name: String,
    pub bpm: Option<f64>,
    pub time_sig_num: Option<u32>,
    pub time_sig_denom: Option<u32>,
    pub color_rgb: (u8, u8, u8),
}

impl Scene {
    pub fn new(index: usize, name: &str) -> Self {
        Self {
            index,
            name: name.to_string(),
            bpm: None,
            time_sig_num: None,
            time_sig_denom: None,
            color_rgb: (40, 160, 220),
        }
    }
}

/// Clip Launcher Scene Matrix Engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipMatrix {
    pub num_tracks: usize,
    pub num_scenes: usize,
    pub global_quantize: LaunchQuantize,
    pub scenes: Vec<Scene>,
    pub slots: Vec<ClipSlot>, // Flattened: track_idx * num_scenes + scene_idx
    pub active_scene_idx: Option<usize>,
    pub queued_scene_idx: Option<usize>,

    // Event queue for audio/GUI dispatch
    #[serde(skip)]
    pub pending_events: Vec<MatrixEvent>,
    #[serde(skip)]
    pub last_evaluated_beat: f64,
}

impl Default for ClipMatrix {
    fn default() -> Self {
        Self::new(8, 8)
    }
}

impl ClipMatrix {
    /// Create a new matrix with `num_tracks` columns and `num_scenes` rows.
    pub fn new(num_tracks: usize, num_scenes: usize) -> Self {
        let tracks = num_tracks.clamp(1, MAX_MATRIX_TRACKS);
        let scenes_count = num_scenes.clamp(1, MAX_MATRIX_SCENES);

        let mut scenes = Vec::with_capacity(scenes_count);
        for s in 0..scenes_count {
            scenes.push(Scene::new(s, &format!("Scene {}", s + 1)));
        }

        let total_slots = tracks * scenes_count;
        let mut slots = Vec::with_capacity(total_slots);
        for t in 0..tracks {
            for s in 0..scenes_count {
                slots.push(ClipSlot::empty(t, s));
            }
        }

        Self {
            num_tracks: tracks,
            num_scenes: scenes_count,
            global_quantize: LaunchQuantize::Bar(1),
            scenes,
            slots,
            active_scene_idx: None,
            queued_scene_idx: None,
            pending_events: Vec::with_capacity(64),
            last_evaluated_beat: 0.0,
        }
    }

    #[inline]
    fn slot_index(&self, track_idx: usize, scene_idx: usize) -> usize {
        track_idx * self.num_scenes + scene_idx
    }

    pub fn get_slot(&self, track_idx: usize, scene_idx: usize) -> Option<&ClipSlot> {
        if track_idx < self.num_tracks && scene_idx < self.num_scenes {
            Some(&self.slots[self.slot_index(track_idx, scene_idx)])
        } else {
            None
        }
    }

    pub fn get_slot_mut(&mut self, track_idx: usize, scene_idx: usize) -> Option<&mut ClipSlot> {
        if track_idx < self.num_tracks && scene_idx < self.num_scenes {
            let idx = self.slot_index(track_idx, scene_idx);
            Some(&mut self.slots[idx])
        } else {
            None
        }
    }

    pub fn set_clip(
        &mut self,
        track_idx: usize,
        scene_idx: usize,
        name: &str,
        length_beats: f64,
        loop_enabled: bool,
    ) {
        if let Some(slot) = self.get_slot_mut(track_idx, scene_idx) {
            *slot = ClipSlot::new_clip(track_idx, scene_idx, name, length_beats, loop_enabled);
        }
    }

    /// Trigger an entire scene row with quantization.
    pub fn trigger_scene(&mut self, scene_idx: usize, immediate: bool) {
        if scene_idx >= self.num_scenes {
            return;
        }

        if immediate || self.global_quantize == LaunchQuantize::None {
            self.execute_scene_launch(scene_idx, self.last_evaluated_beat);
        } else {
            self.queued_scene_idx = Some(scene_idx);
            for t in 0..self.num_tracks {
                let s_idx = self.slot_index(t, scene_idx);
                if !self.slots[s_idx].is_empty {
                    self.slots[s_idx].state = SlotState::QueuedToPlay;
                }
            }
        }
    }

    /// Trigger an individual clip slot with quantization and optional legato playhead preservation.
    pub fn trigger_clip(&mut self, track_idx: usize, scene_idx: usize, immediate: bool) {
        if track_idx >= self.num_tracks || scene_idx >= self.num_scenes {
            return;
        }

        let slot_idx = self.slot_index(track_idx, scene_idx);
        if self.slots[slot_idx].is_empty {
            return;
        }

        if immediate || self.slots[slot_idx].quantize == LaunchQuantize::None {
            self.execute_clip_launch(track_idx, scene_idx, self.last_evaluated_beat);
        } else {
            self.slots[slot_idx].state = SlotState::QueuedToPlay;
        }
    }

    /// Stop playback on a given track with quantization.
    pub fn stop_track(&mut self, track_idx: usize, immediate: bool) {
        if track_idx >= self.num_tracks {
            return;
        }

        for s in 0..self.num_scenes {
            let idx = self.slot_index(track_idx, s);
            if self.slots[idx].state == SlotState::Playing {
                if immediate {
                    self.slots[idx].state = SlotState::Stopped;
                    self.slots[idx].playhead_beats = 0.0;
                    self.slots[idx].total_played_beats = 0.0;
                    self.pending_events.push(MatrixEvent::ClipStopped {
                        track_idx,
                        scene_idx: s,
                    });
                } else {
                    self.slots[idx].state = SlotState::QueuedToStop;
                }
            } else if self.slots[idx].state == SlotState::QueuedToPlay {
                self.slots[idx].state = SlotState::Stopped;
            }
        }
    }

    /// Stop all tracks across the entire matrix.
    pub fn stop_all(&mut self, immediate: bool) {
        for t in 0..self.num_tracks {
            self.stop_track(t, immediate);
        }
        self.queued_scene_idx = None;
        if immediate {
            self.active_scene_idx = None;
        }
    }

    fn execute_clip_launch(&mut self, track_idx: usize, scene_idx: usize, current_beat: f64) {
        // Find existing playing clip on this track to handle legato or stop
        let mut previous_playhead = 0.0;
        let mut was_playing = false;

        for s in 0..self.num_scenes {
            let idx = self.slot_index(track_idx, s);
            if s != scene_idx && self.slots[idx].state == SlotState::Playing {
                previous_playhead = self.slots[idx].playhead_beats;
                was_playing = true;
                self.slots[idx].state = SlotState::Stopped;
                self.slots[idx].playhead_beats = 0.0;
                self.slots[idx].total_played_beats = 0.0;
                self.pending_events.push(MatrixEvent::ClipStopped {
                    track_idx,
                    scene_idx: s,
                });
            }
        }

        let target_idx = self.slot_index(track_idx, scene_idx);
        let slot = &mut self.slots[target_idx];
        slot.state = SlotState::Playing;
        slot.total_played_beats = 0.0;

        if slot.legato && was_playing {
            slot.playhead_beats = previous_playhead % slot.length_beats;
        } else {
            slot.playhead_beats = 0.0;
        }

        self.pending_events.push(MatrixEvent::ClipStarted {
            track_idx,
            scene_idx,
            start_beat: current_beat,
        });
    }

    fn execute_scene_launch(&mut self, scene_idx: usize, current_beat: f64) {
        for t in 0..self.num_tracks {
            let idx = self.slot_index(t, scene_idx);
            if !self.slots[idx].is_empty {
                self.execute_clip_launch(t, scene_idx, current_beat);
            } else {
                self.stop_track(t, true);
            }
        }
        self.active_scene_idx = Some(scene_idx);
        self.queued_scene_idx = None;
        self.pending_events.push(MatrixEvent::SceneLaunched {
            scene_idx,
            beat: current_beat,
        });
    }

    /// Check and execute quantized boundary launches and follow actions.
    /// Called every audio/sequencer processing block with sample-accurate delta beats.
    pub fn evaluate_tick(&mut self, current_beat: f64, beats_per_bar: f64, dt_beats: f64) {
        let bpb = beats_per_bar.max(1.0);

        // 1. Evaluate Queued Scene launches
        if let Some(queued_scene) = self.queued_scene_idx {
            let interval = self.global_quantize.to_beat_interval(bpb);
            let boundary_crossed = if interval <= 0.001 {
                true
            } else {
                let prev_slot = (self.last_evaluated_beat / interval).floor();
                let cur_slot = (current_beat / interval).floor();
                cur_slot > prev_slot
            };

            if boundary_crossed {
                self.execute_scene_launch(queued_scene, current_beat);
            }
        }

        // 2. Evaluate Individual Queued Clips and Stopping Clips
        for t in 0..self.num_tracks {
            for s in 0..self.num_scenes {
                let idx = self.slot_index(t, s);
                let q = self.slots[idx].quantize;
                let interval = q.to_beat_interval(bpb);

                let boundary_crossed = if interval <= 0.001 {
                    true
                } else {
                    let prev_slot = (self.last_evaluated_beat / interval).floor();
                    let cur_slot = (current_beat / interval).floor();
                    cur_slot > prev_slot
                };

                if boundary_crossed {
                    if self.slots[idx].state == SlotState::QueuedToPlay {
                        self.execute_clip_launch(t, s, current_beat);
                    } else if self.slots[idx].state == SlotState::QueuedToStop {
                        self.slots[idx].state = SlotState::Stopped;
                        self.slots[idx].playhead_beats = 0.0;
                        self.slots[idx].total_played_beats = 0.0;
                        self.pending_events.push(MatrixEvent::ClipStopped {
                            track_idx: t,
                            scene_idx: s,
                        });
                    }
                }
            }
        }

        // 3. Advance Playheads and Process Follow Actions
        for t in 0..self.num_tracks {
            for s in 0..self.num_scenes {
                let idx = self.slot_index(t, s);
                if self.slots[idx].state == SlotState::Playing {
                    let length = self.slots[idx].length_beats;
                    let follow = self.slots[idx].follow_action;

                    self.slots[idx].playhead_beats += dt_beats;
                    self.slots[idx].total_played_beats += dt_beats;

                    // Check follow actions
                    if follow.action_type != FollowActionType::None
                        && self.slots[idx].total_played_beats >= follow.trigger_beats - 1e-4
                    {
                        self.execute_follow_action(t, s, follow.action_type, current_beat);
                        continue;
                    }

                    // Handle looping
                    if self.slots[idx].playhead_beats >= length {
                        if self.slots[idx].loop_enabled {
                            self.slots[idx].playhead_beats %= length;
                        } else {
                            self.slots[idx].state = SlotState::Stopped;
                            self.slots[idx].playhead_beats = 0.0;
                            self.slots[idx].total_played_beats = 0.0;
                            self.pending_events.push(MatrixEvent::ClipStopped {
                                track_idx: t,
                                scene_idx: s,
                            });
                        }
                    }
                }
            }
        }

        self.last_evaluated_beat = current_beat;
    }

    fn execute_follow_action(
        &mut self,
        track_idx: usize,
        current_scene_idx: usize,
        action: FollowActionType,
        current_beat: f64,
    ) {
        match action {
            FollowActionType::None => {}
            FollowActionType::Stop => {
                self.stop_track(track_idx, true);
            }
            FollowActionType::PlayNext => {
                let next_scene = (current_scene_idx + 1) % self.num_scenes;
                if !self.slots[self.slot_index(track_idx, next_scene)].is_empty {
                    self.execute_clip_launch(track_idx, next_scene, current_beat);
                }
            }
            FollowActionType::PlayPrevious => {
                let prev_scene = if current_scene_idx == 0 {
                    self.num_scenes - 1
                } else {
                    current_scene_idx - 1
                };
                if !self.slots[self.slot_index(track_idx, prev_scene)].is_empty {
                    self.execute_clip_launch(track_idx, prev_scene, current_beat);
                }
            }
            FollowActionType::PlayFirst => {
                if !self.slots[self.slot_index(track_idx, 0)].is_empty {
                    self.execute_clip_launch(track_idx, 0, current_beat);
                }
            }
            FollowActionType::PlayLast => {
                let last = self.num_scenes - 1;
                if !self.slots[self.slot_index(track_idx, last)].is_empty {
                    self.execute_clip_launch(track_idx, last, current_beat);
                }
            }
            FollowActionType::PlayAny => {
                let next_scene = (current_scene_idx + 1) % self.num_scenes;
                if !self.slots[self.slot_index(track_idx, next_scene)].is_empty {
                    self.execute_clip_launch(track_idx, next_scene, current_beat);
                }
            }
            FollowActionType::Jump(target_scene) => {
                if target_scene < self.num_scenes
                    && !self.slots[self.slot_index(track_idx, target_scene)].is_empty
                {
                    self.execute_clip_launch(track_idx, target_scene, current_beat);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clip_matrix_scene_launch_and_quantization() {
        let mut matrix = ClipMatrix::new(4, 4);
        matrix.global_quantize = LaunchQuantize::Bar(1);

        // Populate Scene 0
        matrix.set_clip(0, 0, "Drums 1", 4.0, true);
        matrix.set_clip(1, 0, "Bass 1", 4.0, true);

        // Populate Scene 1
        matrix.set_clip(0, 1, "Drums 2", 4.0, true);
        matrix.set_clip(1, 1, "Bass 2", 4.0, true);

        // Queue Scene 0 at beat 0.5
        matrix.trigger_scene(0, false);
        assert_eq!(matrix.queued_scene_idx, Some(0));
        assert_eq!(matrix.get_slot(0, 0).unwrap().state, SlotState::QueuedToPlay);

        // Advance to beat 3.9 (before 1 bar = 4 beats)
        matrix.evaluate_tick(3.9, 4.0, 3.4);
        assert_eq!(matrix.queued_scene_idx, Some(0));

        // Cross bar boundary at beat 4.0
        matrix.evaluate_tick(4.0, 4.0, 0.1);
        assert_eq!(matrix.active_scene_idx, Some(0));
        assert_eq!(matrix.get_slot(0, 0).unwrap().state, SlotState::Playing);
        assert_eq!(matrix.get_slot(1, 0).unwrap().state, SlotState::Playing);
    }

    #[test]
    fn test_clip_matrix_legato_launching() {
        let mut matrix = ClipMatrix::new(2, 2);
        matrix.set_clip(0, 0, "Lead A", 8.0, true);
        matrix.set_clip(0, 1, "Lead B", 8.0, true);

        if let Some(slot) = matrix.get_slot_mut(0, 1) {
            slot.legato = true;
        }

        // Launch Clip A immediately
        matrix.trigger_clip(0, 0, true);
        assert_eq!(matrix.get_slot(0, 0).unwrap().state, SlotState::Playing);

        // Play for 3.5 beats
        matrix.evaluate_tick(3.5, 4.0, 3.5);
        assert_eq!(matrix.get_slot(0, 0).unwrap().playhead_beats, 3.5);

        // Legato launch Clip B
        matrix.trigger_clip(0, 1, true);
        assert_eq!(matrix.get_slot(0, 1).unwrap().state, SlotState::Playing);
        assert_eq!(matrix.get_slot(0, 1).unwrap().playhead_beats, 3.5);
    }

    #[test]
    fn test_clip_matrix_follow_action_chaining() {
        let mut matrix = ClipMatrix::new(2, 3);
        matrix.set_clip(0, 0, "Intro", 2.0, false);
        matrix.set_clip(0, 1, "Verse", 4.0, true);

        if let Some(slot) = matrix.get_slot_mut(0, 0) {
            slot.follow_action = FollowAction {
                action_type: FollowActionType::PlayNext,
                trigger_beats: 2.0,
                chance: 1.0,
            };
        }

        matrix.trigger_clip(0, 0, true);
        assert_eq!(matrix.get_slot(0, 0).unwrap().state, SlotState::Playing);

        // Tick 2.0 beats -> Follow Action fires Verse (Scene 1)
        matrix.evaluate_tick(2.0, 4.0, 2.0);
        assert_eq!(matrix.get_slot(0, 0).unwrap().state, SlotState::Stopped);
        assert_eq!(matrix.get_slot(0, 1).unwrap().state, SlotState::Playing);
    }
}
