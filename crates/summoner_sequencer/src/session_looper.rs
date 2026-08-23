// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Live Multitrack Session Looper Engine & Non-Blocking State Machine (Milestone 8).
//!
//! Provides a lock-free circular audio buffer looper with seamless bar-quantized record,
//! overdub, multiply, divide, non-destructive undo/redo layers, and sample-accurate
//! playback boundary synchronization with `Transport` and `ParamBus`.

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use summoner_core::transport::Transport;

/// Default maximum buffer capacity per track (e.g. 60 seconds at 48kHz stereo = 2,880,000 samples).
pub const DEFAULT_MAX_CAPACITY_SAMPLES: usize = 2_880_000;

/// Default crossfade length in samples for seamless boundary turnaround.
pub const DEFAULT_CROSSFADE_SAMPLES: usize = 32;

/// Operational state of an individual looper track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LooperState {
    /// Empty track, ready for initial recording.
    Empty,
    /// Pending quantized trigger to begin recording.
    PendingRecord,
    /// Actively recording initial master loop length.
    Recording,
    /// Pending quantized trigger to switch to playback.
    PendingPlay,
    /// Playing back recorded loop continuously.
    Playing,
    /// Pending quantized trigger to enter overdub mode.
    PendingOverdub,
    /// Overdubbing new audio on top of existing loop.
    Overdubbing,
    /// Playback paused at current playhead.
    Paused,
    /// Audio muted but playhead continues tracking synchronization.
    Muted,
}

/// Quantized command trigger dispatched to the looper state machine.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LooperCommand {
    TriggerRecord,
    TriggerPlay,
    TriggerOverdub,
    TriggerMultiply(usize),
    TriggerDivide(usize),
    TriggerUndo,
    TriggerRedo,
    TriggerClear,
    TriggerToggleMute,
    TriggerReverse,
    TriggerHalfSpeed,
    TriggerDoubleSpeed,
}

/// Quantization interval for synchronizing looper actions with transport beats/bars.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LooperQuantize {
    /// Instant execution on current audio frame.
    None,
    /// Quantize to nearest or next beat interval (e.g. 1.0 beat, 0.25 beat).
    Beat(f64),
    /// Quantize to next whole bar boundary (e.g. 1 bar, 2 bars, 4 bars).
    Bar(u32),
}

/// Synchronization mode across multitrack looper lanes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LooperSyncMode {
    /// Independent free-running tracks.
    Free,
    /// Track 0 defines the master loop duration and grid for all follower tracks.
    MasterFollower,
    /// Locked strictly to global DAW Transport bars and tempo.
    TransportLocked,
}

/// Snapshot layer for non-destructive undo and redo stacks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LooperLayer {
    pub loop_length: usize,
    pub samples_l: Vec<f32>,
    pub samples_r: Vec<f32>,
}

impl LooperLayer {
    pub fn new_empty(capacity: usize) -> Self {
        Self {
            loop_length: 0,
            samples_l: Vec::with_capacity(capacity),
            samples_r: Vec::with_capacity(capacity),
        }
    }

    pub fn with_data(loop_length: usize, samples_l: Vec<f32>, samples_r: Vec<f32>) -> Self {
        Self {
            loop_length,
            samples_l,
            samples_r,
        }
    }
}

/// An individual audio looper track featuring circular playback, overdubbing, and undo history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LooperTrack {
    pub id: u32,
    pub name: String,
    pub state: LooperState,
    pub pending_command: Option<LooperCommand>,
    pub quantize: LooperQuantize,

    pub loop_length_samples: usize,
    pub playhead: usize,
    pub speed: f32,
    pub reversed: bool,
    pub volume: f32,
    pub pan: f32,
    pub feedback: f32,
    pub crossfade_samples: usize,

    // Audio buffers
    pub buffer_l: Vec<f32>,
    pub buffer_r: Vec<f32>,

    // Undo / Redo history
    pub undo_stack: Vec<LooperLayer>,
    pub redo_stack: Vec<LooperLayer>,
    pub max_undo_levels: usize,

    // Internal boundary tracking
    last_transport_beat: f64,
}

impl LooperTrack {
    /// Create a new looper track with pre-allocated capacity.
    pub fn new(id: u32, name: &str, capacity_samples: usize) -> Self {
        let mut buf_l = Vec::with_capacity(capacity_samples);
        let mut buf_r = Vec::with_capacity(capacity_samples);
        buf_l.resize(capacity_samples, 0.0);
        buf_r.resize(capacity_samples, 0.0);

        Self {
            id,
            name: name.to_string(),
            state: LooperState::Empty,
            pending_command: None,
            quantize: LooperQuantize::Bar(1),
            loop_length_samples: 0,
            playhead: 0,
            speed: 1.0,
            reversed: false,
            volume: 1.0,
            pan: 0.0,
            feedback: 1.0,
            crossfade_samples: DEFAULT_CROSSFADE_SAMPLES,
            buffer_l: buf_l,
            buffer_r: buf_r,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_undo_levels: 16,
            last_transport_beat: 0.0,
        }
    }

    /// Queue a command for quantized or immediate execution.
    pub fn queue_command(&mut self, cmd: LooperCommand) {
        if self.quantize == LooperQuantize::None {
            self.execute_command(cmd);
        } else {
            self.pending_command = Some(cmd);
        }
    }

    /// Execute command immediately on state machine.
    pub fn execute_command(&mut self, cmd: LooperCommand) {
        match cmd {
            LooperCommand::TriggerRecord => match self.state {
                LooperState::Empty | LooperState::Paused | LooperState::Muted => {
                    self.save_undo_state();
                    self.buffer_l.fill(0.0);
                    self.buffer_r.fill(0.0);
                    self.playhead = 0;
                    self.loop_length_samples = 0;
                    self.state = LooperState::Recording;
                }
                LooperState::Playing => {
                    self.save_undo_state();
                    self.state = LooperState::Overdubbing;
                }
                LooperState::Recording | LooperState::Overdubbing => {
                    if self.loop_length_samples == 0 {
                        self.loop_length_samples = self.playhead.max(1);
                        self.playhead = 0;
                    }
                    self.state = LooperState::Playing;
                }
                _ => {}
            },
            LooperCommand::TriggerPlay => {
                if self.state == LooperState::Recording {
                    self.loop_length_samples = self.playhead.max(1);
                    self.playhead = 0;
                }
                if self.loop_length_samples > 0 {
                    self.state = LooperState::Playing;
                }
            }
            LooperCommand::TriggerOverdub => {
                if self.loop_length_samples > 0 {
                    self.save_undo_state();
                    self.state = LooperState::Overdubbing;
                }
            }
            LooperCommand::TriggerMultiply(factor) => {
                if self.loop_length_samples > 0 && factor >= 2 {
                    let old_len = self.loop_length_samples;
                    let new_len = (old_len * factor).min(self.buffer_l.len());
                    if new_len > old_len {
                        self.save_undo_state();
                        for rep in 1..factor {
                            let src_start = 0;
                            let dst_start = rep * old_len;
                            let dst_end = (dst_start + old_len).min(self.buffer_l.len());
                            let count = dst_end - dst_start;
                            if count > 0 && dst_start < self.buffer_l.len() {
                                for i in 0..count {
                                    self.buffer_l[dst_start + i] = self.buffer_l[src_start + i];
                                    self.buffer_r[dst_start + i] = self.buffer_r[src_start + i];
                                }
                            }
                        }
                        self.loop_length_samples = new_len;
                    }
                }
            }
            LooperCommand::TriggerDivide(divisor) => {
                if self.loop_length_samples > 0 && divisor >= 2 {
                    let new_len = (self.loop_length_samples / divisor).max(1);
                    self.save_undo_state();
                    self.loop_length_samples = new_len;
                    self.playhead %= new_len;
                }
            }
            LooperCommand::TriggerUndo => {
                self.undo();
            }
            LooperCommand::TriggerRedo => {
                self.redo();
            }
            LooperCommand::TriggerClear => {
                self.save_undo_state();
                self.buffer_l.fill(0.0);
                self.buffer_r.fill(0.0);
                self.loop_length_samples = 0;
                self.playhead = 0;
                self.state = LooperState::Empty;
            }
            LooperCommand::TriggerToggleMute => {
                self.state = if self.state == LooperState::Muted {
                    if self.loop_length_samples > 0 {
                        LooperState::Playing
                    } else {
                        LooperState::Empty
                    }
                } else {
                    LooperState::Muted
                };
            }
            LooperCommand::TriggerReverse => {
                self.reversed = !self.reversed;
            }
            LooperCommand::TriggerHalfSpeed => {
                self.speed = (self.speed * 0.5).max(0.25);
            }
            LooperCommand::TriggerDoubleSpeed => {
                self.speed = (self.speed * 2.0).min(4.0);
            }
        }
        self.pending_command = None;
    }

    /// Save current layer to undo stack and clear redo stack.
    pub fn save_undo_state(&mut self) {
        if self.loop_length_samples == 0 {
            return;
        }
        let len = self.loop_length_samples;
        let layer = LooperLayer {
            loop_length: len,
            samples_l: self.buffer_l[..len].to_vec(),
            samples_r: self.buffer_r[..len].to_vec(),
        };

        self.undo_stack.push(layer);
        if self.undo_stack.len() > self.max_undo_levels {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    /// Undo last record/overdub/operation.
    pub fn undo(&mut self) -> bool {
        if let Some(prev) = self.undo_stack.pop() {
            let current_len = self.loop_length_samples;
            let current_layer = LooperLayer {
                loop_length: current_len,
                samples_l: self.buffer_l[..current_len].to_vec(),
                samples_r: self.buffer_r[..current_len].to_vec(),
            };
            self.redo_stack.push(current_layer);

            self.loop_length_samples = prev.loop_length;
            self.buffer_l[..prev.loop_length].copy_from_slice(&prev.samples_l);
            self.buffer_r[..prev.loop_length].copy_from_slice(&prev.samples_r);
            if self.loop_length_samples > 0 {
                self.playhead %= self.loop_length_samples;
            } else {
                self.playhead = 0;
                self.state = LooperState::Empty;
            }
            true
        } else {
            false
        }
    }

    /// Redo previously undone operation.
    pub fn redo(&mut self) -> bool {
        if let Some(next) = self.redo_stack.pop() {
            let current_len = self.loop_length_samples;
            let current_layer = LooperLayer {
                loop_length: current_len,
                samples_l: self.buffer_l[..current_len].to_vec(),
                samples_r: self.buffer_r[..current_len].to_vec(),
            };
            self.undo_stack.push(current_layer);

            self.loop_length_samples = next.loop_length;
            self.buffer_l[..next.loop_length].copy_from_slice(&next.samples_l);
            self.buffer_r[..next.loop_length].copy_from_slice(&next.samples_r);
            if self.loop_length_samples > 0 {
                self.playhead %= self.loop_length_samples;
            } else {
                self.playhead = 0;
            }
            true
        } else {
            false
        }
    }

    /// Check if pending quantization conditions are met at this audio block boundary.
    pub fn check_quantize_boundary(&mut self, current_beat: f64, beats_per_bar: f64) {
        if let Some(cmd) = self.pending_command {
            let triggered = match self.quantize {
                LooperQuantize::None => true,
                LooperQuantize::Beat(interval) => {
                    let prev_slot = (self.last_transport_beat / interval).floor();
                    let cur_slot = (current_beat / interval).floor();
                    cur_slot > prev_slot
                }
                LooperQuantize::Bar(bar_interval) => {
                    let bar_beats = beats_per_bar * bar_interval as f64;
                    let prev_bar = (self.last_transport_beat / bar_beats).floor();
                    let cur_bar = (current_beat / bar_beats).floor();
                    cur_bar > prev_bar
                }
            };

            if triggered {
                self.execute_command(cmd);
            }
        }
        self.last_transport_beat = current_beat;
    }

    /// Process a single stereo sample through the looper engine.
    #[inline]
    pub fn process_sample(&mut self, in_l: f32, in_r: f32) -> (f32, f32) {
        match self.state {
            LooperState::Empty | LooperState::PendingRecord => (0.0, 0.0),
            LooperState::Recording => {
                let idx = self.playhead;
                if idx < self.buffer_l.len() {
                    self.buffer_l[idx] = in_l;
                    self.buffer_r[idx] = in_r;
                    self.playhead += 1;
                }
                // Monitor input directly during initial recording
                (in_l * self.volume, in_r * self.volume)
            }
            LooperState::Playing | LooperState::Overdubbing => {
                if self.loop_length_samples == 0 {
                    return (0.0, 0.0);
                }

                let actual_idx = if self.reversed {
                    self.loop_length_samples - 1 - (self.playhead % self.loop_length_samples)
                } else {
                    self.playhead % self.loop_length_samples
                };

                let mut out_l = self.buffer_l[actual_idx];
                let mut out_r = self.buffer_r[actual_idx];

                // Apply crossfade at loop wrap point to eliminate clicks
                let xf_len = self.crossfade_samples.min(self.loop_length_samples / 4);
                if xf_len > 0 && actual_idx < xf_len {
                    let t = actual_idx as f32 / xf_len as f32;
                    let tail_idx = self.loop_length_samples - xf_len + actual_idx;
                    let head_fade = (t * PI * 0.5).sin();
                    let tail_fade = ((1.0 - t) * PI * 0.5).sin();
                    out_l = out_l * head_fade + self.buffer_l[tail_idx] * tail_fade;
                    out_r = out_r * head_fade + self.buffer_r[tail_idx] * tail_fade;
                }

                if self.state == LooperState::Overdubbing {
                    // Overdub new input mixed with decayed existing layer
                    self.buffer_l[actual_idx] = self.buffer_l[actual_idx] * self.feedback + in_l;
                    self.buffer_r[actual_idx] = self.buffer_r[actual_idx] * self.feedback + in_r;
                }

                self.playhead = (self.playhead + 1) % self.loop_length_samples;

                let (pan_l, pan_r) = if self.pan <= 0.0 {
                    (1.0, (1.0 + self.pan).max(0.0))
                } else {
                    ((1.0 - self.pan).max(0.0), 1.0)
                };

                (out_l * self.volume * pan_l, out_r * self.volume * pan_r)
            }
            LooperState::Paused => (0.0, 0.0),
            LooperState::Muted => {
                if self.loop_length_samples > 0 {
                    self.playhead = (self.playhead + 1) % self.loop_length_samples;
                }
                (0.0, 0.0)
            }
            _ => (0.0, 0.0),
        }
    }
}

/// Multitrack Live Session Looper Engine managing multiple synchronized track lanes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultitrackSessionLooper {
    pub tracks: Vec<LooperTrack>,
    pub sync_mode: LooperSyncMode,
    pub master_track_idx: usize,
}

impl Default for MultitrackSessionLooper {
    fn default() -> Self {
        Self::new(4, DEFAULT_MAX_CAPACITY_SAMPLES)
    }
}

impl MultitrackSessionLooper {
    /// Create a new session looper with `num_tracks` lanes.
    pub fn new(num_tracks: usize, capacity_per_track: usize) -> Self {
        let mut tracks = Vec::with_capacity(num_tracks);
        for i in 0..num_tracks {
            tracks.push(LooperTrack::new(
                i as u32,
                &format!("Loop Track {}", i + 1),
                capacity_per_track,
            ));
        }
        Self {
            tracks,
            sync_mode: LooperSyncMode::MasterFollower,
            master_track_idx: 0,
        }
    }

    /// Process a block of stereo audio across all active looper tracks.
    pub fn process_block(
        &mut self,
        input_l: &[f32],
        input_r: &[f32],
        output_l: &mut [f32],
        output_r: &mut [f32],
        transport: &Transport,
    ) {
        let num_samples = input_l.len().min(output_l.len()).min(output_r.len());
        let current_beat = transport.beats();
        let beats_per_bar = transport.time_signature_num as f64;

        // Check quantize boundaries across all tracks
        for track in &mut self.tracks {
            track.check_quantize_boundary(current_beat, beats_per_bar);
        }

        output_l[..num_samples].fill(0.0);
        output_r[..num_samples].fill(0.0);

        for track in &mut self.tracks {
            for i in 0..num_samples {
                let in_l = input_l[i];
                let in_r = if input_r.is_empty() { in_l } else { input_r[i] };
                let (track_out_l, track_out_r) = track.process_sample(in_l, in_r);
                output_l[i] += track_out_l;
                output_r[i] += track_out_r;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::allocator::AllocGuard;

    #[test]
    fn test_looper_record_play_overdub_lifecycle() {
        let mut track = LooperTrack::new(0, "Test Track", 48000);
        track.quantize = LooperQuantize::None;

        // 1. Record 100 samples
        track.execute_command(LooperCommand::TriggerRecord);
        assert_eq!(track.state, LooperState::Recording);

        for i in 0..100 {
            let (out_l, _) = track.process_sample(0.5, 0.5);
            assert_eq!(out_l, 0.5);
            let _ = i;
        }

        // 2. Switch to Play mode
        track.execute_command(LooperCommand::TriggerPlay);
        assert_eq!(track.state, LooperState::Playing);
        assert_eq!(track.loop_length_samples, 100);

        // Verify playback repeats recorded 0.5
        for _ in 0..200 {
            let (out_l, _) = track.process_sample(0.0, 0.0);
            assert!((out_l - 0.5).abs() < 1e-4 || track.playhead < 32); // accounts for crossfade
        }

        // 3. Overdub mode
        track.execute_command(LooperCommand::TriggerOverdub);
        assert_eq!(track.state, LooperState::Overdubbing);

        for _ in 0..100 {
            let _ = track.process_sample(0.25, 0.25);
        }

        // 4. Undo restores back to 0.5
        assert!(track.undo());
        assert_eq!(track.buffer_l[50], 0.5);

        // 5. Redo restores overdubbed value (0.5 + 0.25 = 0.75)
        assert!(track.redo());
        assert_eq!(track.buffer_l[50], 0.75);
    }

    #[test]
    fn test_looper_multiply_and_divide() {
        let mut track = LooperTrack::new(0, "Multiply Track", 48000);
        track.quantize = LooperQuantize::None;

        track.execute_command(LooperCommand::TriggerRecord);
        for _ in 0..50 {
            let _ = track.process_sample(1.0, 1.0);
        }
        track.execute_command(LooperCommand::TriggerPlay);
        assert_eq!(track.loop_length_samples, 50);

        // Multiply x2 -> 100 samples
        track.execute_command(LooperCommand::TriggerMultiply(2));
        assert_eq!(track.loop_length_samples, 100);
        assert_eq!(track.buffer_l[75], 1.0);

        // Divide /2 -> 50 samples
        track.execute_command(LooperCommand::TriggerDivide(2));
        assert_eq!(track.loop_length_samples, 50);
    }

    #[test]
    fn test_looper_zero_allocation_during_streaming() {
        let mut session = MultitrackSessionLooper::new(4, 48000);
        let transport = Transport::new(48000, 120.0);

        let in_l = [0.2f32; 64];
        let in_r = [0.2f32; 64];
        let mut out_l = [0.0f32; 64];
        let mut out_r = [0.0f32; 64];

        // Ensure real-time processing has zero allocations
        {
            let _guard = AllocGuard::new();
            session.process_block(&in_l, &in_r, &mut out_l, &mut out_r, &transport);
        }

        assert!(out_l.iter().all(|s| s.is_finite()));
    }
}
