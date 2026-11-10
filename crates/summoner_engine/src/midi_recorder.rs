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

//! Real-Time MIDI Recording, Overdub, Step Input & Retrospective Capture (Epic C, Task C3).
//!
//! Provides comprehensive MIDI recording capabilities:
//! - Multi-track MIDI recording to armed tracks.
//! - Overdub mode (merging notes non-destructively) and Replace mode.
//! - Quantize-on-record with adjustable grid divisions and strength.
//! - Step input editor (entering notes/chords/rests step by step via MIDI).
//! - Retrospective MIDI Capture (circular buffer capturing played MIDI even when transport was stopped).

use std::collections::{HashMap, HashSet};
use summoner_core::midi::MidiEvent;
use summoner_project::schema::{SequenceConfig, TrackerStepConfig};

/// Quantization grid division setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QuantizeGrid {
    #[default]
    Off,
    Quarter,
    Eighth,
    Sixteenth,
    ThirtySecond,
    EighthTriplet,
    SixteenthTriplet,
}

impl QuantizeGrid {
    /// Return the grid spacing in beats, or None if Off.
    pub fn step_beats(&self) -> Option<f64> {
        match self {
            Self::Off => None,
            Self::Quarter => Some(1.0),
            Self::Eighth => Some(0.5),
            Self::Sixteenth => Some(0.25),
            Self::ThirtySecond => Some(0.125),
            Self::EighthTriplet => Some(1.0 / 3.0),
            Self::SixteenthTriplet => Some(0.5 / 3.0),
        }
    }

    /// Quantize a beat value according to this grid and strength (0.0 to 1.0).
    pub fn quantize(&self, beat: f64, strength: f32) -> f64 {
        if let Some(grid) = self.step_beats() {
            let snapped = (beat / grid).round() * grid;
            let factor = strength.clamp(0.0, 1.0) as f64;
            beat + (snapped - beat) * factor
        } else {
            beat
        }
    }
}

/// Recording mode for MIDI capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MidiRecordMode {
    #[default]
    Overdub,
    Replace,
}

/// A fully formed recorded MIDI note event.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordedMidiNote {
    pub channel: u8,
    pub pitch: u8,
    pub velocity: f32,
    pub start_beat: f64,
    pub duration_beats: f64,
}

/// Circular buffer for Retrospective MIDI Capture.
#[derive(Debug, Clone)]
pub struct RetrospectiveMidiBuffer {
    capacity: usize,
    buffer: Vec<(f64, MidiEvent)>,
    write_idx: usize,
    count: usize,
}

impl RetrospectiveMidiBuffer {
    /// Create a new buffer with the specified event capacity (e.g. 8192).
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.max(64);
        Self {
            capacity,
            buffer: vec![(0.0, MidiEvent::NoteOff(0, 0, 0)); capacity],
            write_idx: 0,
            count: 0,
        }
    }

    /// Push an incoming MIDI event with its timestamp in beats.
    pub fn push(&mut self, beat: f64, event: MidiEvent) {
        self.buffer[self.write_idx] = (beat, event);
        self.write_idx = (self.write_idx + 1) % self.capacity;
        if self.count < self.capacity {
            self.count += 1;
        }
    }

    /// Clear all captured retrospective events.
    pub fn clear(&mut self) {
        self.write_idx = 0;
        self.count = 0;
    }

    /// Extract captured events in chronological order.
    pub fn events(&self) -> Vec<(f64, MidiEvent)> {
        let mut list = Vec::with_capacity(self.count);
        let start_idx = if self.count < self.capacity {
            0
        } else {
            self.write_idx
        };
        for i in 0..self.count {
            let idx = (start_idx + i) % self.capacity;
            list.push(self.buffer[idx]);
        }
        list
    }

    /// Convert recent captured MIDI events into a `SequenceConfig`.
    pub fn capture_as_sequence(
        &self,
        quantize: QuantizeGrid,
        step_division: f64,
    ) -> Option<SequenceConfig> {
        let events = self.events();
        if events.is_empty() {
            return None;
        }

        // Pair NoteOn and NoteOff
        let mut active_notes: HashMap<(u8, u8), (f64, f32)> = HashMap::new();
        let mut notes: Vec<RecordedMidiNote> = Vec::new();
        let mut min_beat = f64::MAX;

        for (beat, event) in &events {
            match *event {
                MidiEvent::NoteOn(ch, note, vel) => {
                    active_notes.insert((ch, note), (*beat, (vel as f32) / 127.0));
                    if *beat < min_beat {
                        min_beat = *beat;
                    }
                }
                MidiEvent::NoteOff(ch, note, _) => {
                    if let Some((start, vel)) = active_notes.remove(&(ch, note)) {
                        let dur = (beat - start).max(step_division * 0.5);
                        notes.push(RecordedMidiNote {
                            channel: ch,
                            pitch: note,
                            velocity: vel,
                            start_beat: start,
                            duration_beats: dur,
                        });
                    }
                }
                _ => {}
            }
        }

        // Flush any dangling notes
        for ((ch, note), (start, vel)) in active_notes {
            notes.push(RecordedMidiNote {
                channel: ch,
                pitch: note,
                velocity: vel,
                start_beat: start,
                duration_beats: step_division,
            });
        }

        if notes.is_empty() {
            return None;
        }

        // Normalize start beat relative to earliest note
        let base_beat = min_beat;
        let mut seq = SequenceConfig {
            start_beat: (base_beat / 4.0).floor() * 4.0, // align to bar
            step_division,
            name: "Retrospective Capture".to_string(),
            ..Default::default()
        };

        for n in &notes {
            let rel_beat = (n.start_beat - seq.start_beat).max(0.0);
            let q_beat = quantize.quantize(rel_beat, 1.0);
            let step_idx = (q_beat / step_division).round() as usize;

            while seq.steps.len() <= step_idx {
                seq.steps.push(TrackerStepConfig {
                    active: false,
                    ..Default::default()
                });
            }

            seq.steps[step_idx] = TrackerStepConfig {
                note: n.pitch as f64,
                velocity: n.velocity,
                gate: (n.duration_beats / step_division) as f32,
                active: true,
                ..Default::default()
            };
        }

        Some(seq)
    }
}

/// Step input recording controller for entering notes step-by-step.
#[derive(Debug, Clone)]
pub struct StepInputController {
    pub cursor_step: usize,
    pub step_division: f64,
    pub default_gate: f32,
    pub default_velocity: f32,
}

impl Default for StepInputController {
    fn default() -> Self {
        Self {
            cursor_step: 0,
            step_division: 0.25,
            default_gate: 0.8,
            default_velocity: 0.8,
        }
    }
}

impl StepInputController {
    /// Create a new step input controller with given step division (e.g. 0.25 for 1/16).
    pub fn new(step_division: f64) -> Self {
        Self {
            step_division,
            ..Default::default()
        }
    }

    /// Input a single note at cursor and advance cursor.
    pub fn input_note(
        &mut self,
        sequence: &mut SequenceConfig,
        pitch: u8,
        velocity: Option<f32>,
    ) -> usize {
        while sequence.steps.len() <= self.cursor_step {
            sequence.steps.push(TrackerStepConfig {
                active: false,
                ..Default::default()
            });
        }

        sequence.steps[self.cursor_step] = TrackerStepConfig {
            note: pitch as f64,
            velocity: velocity.unwrap_or(self.default_velocity),
            gate: self.default_gate,
            active: true,
            ..Default::default()
        };

        let placed_step = self.cursor_step;
        self.cursor_step += 1;
        placed_step
    }

    /// Input a rest: leave step inactive and advance cursor.
    pub fn input_rest(&mut self, sequence: &mut SequenceConfig) -> usize {
        while sequence.steps.len() <= self.cursor_step {
            sequence.steps.push(TrackerStepConfig {
                active: false,
                ..Default::default()
            });
        }
        let rest_step = self.cursor_step;
        self.cursor_step += 1;
        rest_step
    }

    /// Step backwards, clearing the step if active.
    pub fn backspace(&mut self, sequence: &mut SequenceConfig) {
        if self.cursor_step > 0 {
            self.cursor_step -= 1;
            if self.cursor_step < sequence.steps.len() {
                sequence.steps[self.cursor_step].active = false;
            }
        }
    }
}

/// Comprehensive MIDI recording and overdub engine.
pub struct MidiRecorder {
    armed_tracks: HashSet<u64>,
    mode: MidiRecordMode,
    quantize: QuantizeGrid,
    quantize_strength: f32,
    is_recording: bool,
    record_start_beat: f64,
    step_division: f64,
    active_notes: HashMap<(u8, u8), (f64, f32)>, // (channel, pitch) -> (start_beat, velocity)
    recorded_notes: Vec<RecordedMidiNote>,
    pub retrospective_buffer: RetrospectiveMidiBuffer,
    pub step_input: StepInputController,
}

impl MidiRecorder {
    /// Create a new MIDI recorder.
    pub fn new() -> Self {
        Self {
            armed_tracks: HashSet::new(),
            mode: MidiRecordMode::Overdub,
            quantize: QuantizeGrid::Off,
            quantize_strength: 1.0,
            is_recording: false,
            record_start_beat: 0.0,
            step_division: 0.25,
            active_notes: HashMap::new(),
            recorded_notes: Vec::new(),
            retrospective_buffer: RetrospectiveMidiBuffer::new(4096),
            step_input: StepInputController::default(),
        }
    }

    /// Arm a track for MIDI recording.
    pub fn arm_track(&mut self, track_id: u64) {
        self.armed_tracks.insert(track_id);
    }

    /// Disarm a track.
    pub fn disarm_track(&mut self, track_id: u64) {
        self.armed_tracks.remove(&track_id);
    }

    /// Return true if track is armed for MIDI recording.
    pub fn is_track_armed(&self, track_id: u64) -> bool {
        self.armed_tracks.contains(&track_id)
    }

    /// Configure recording mode (Overdub vs Replace).
    pub fn set_mode(&mut self, mode: MidiRecordMode) {
        self.mode = mode;
    }

    /// Configure quantize-on-record grid and strength (0.0 to 1.0).
    pub fn set_quantize(&mut self, grid: QuantizeGrid, strength: f32) {
        self.quantize = grid;
        self.quantize_strength = strength.clamp(0.0, 1.0);
    }

    /// Configure step division for created sequences (default 0.25 = 1/16th).
    pub fn set_step_division(&mut self, division: f64) {
        self.step_division = division.max(0.01);
        self.step_input.step_division = self.step_division;
    }

    /// Start recording MIDI at given transport beat.
    pub fn start_recording(&mut self, current_beat: f64) {
        self.is_recording = true;
        self.record_start_beat = current_beat;
        self.active_notes.clear();
        self.recorded_notes.clear();
    }

    /// Return true if currently recording.
    pub fn is_recording(&self) -> bool {
        self.is_recording
    }

    /// Process an incoming MIDI event from hardware or virtual port.
    pub fn process_event(&mut self, event: MidiEvent, current_beat: f64) {
        // Retrospective buffer always records incoming played MIDI!
        self.retrospective_buffer.push(current_beat, event);

        if !self.is_recording || self.armed_tracks.is_empty() {
            return;
        }

        match event {
            MidiEvent::NoteOn(ch, note, vel) => {
                let norm_vel = (vel as f32) / 127.0;
                let quant_beat = self.quantize.quantize(current_beat, self.quantize_strength);
                self.active_notes.insert((ch, note), (quant_beat, norm_vel));
            }
            MidiEvent::NoteOff(ch, note, _) => {
                if let Some((start_beat, vel)) = self.active_notes.remove(&(ch, note)) {
                    let duration = (current_beat - start_beat).max(self.step_division * 0.5);
                    self.recorded_notes.push(RecordedMidiNote {
                        channel: ch,
                        pitch: note,
                        velocity: vel,
                        start_beat,
                        duration_beats: duration,
                    });
                }
            }
            _ => {}
        }
    }

    /// Stop recording, commit captured notes, and merge/replace into track sequences.
    pub fn stop_recording(
        &mut self,
        current_beat: f64,
        existing_sequences: &HashMap<u64, SequenceConfig>,
    ) -> HashMap<u64, SequenceConfig> {
        self.is_recording = false;

        // Flush any held notes
        for ((ch, note), (start_beat, vel)) in self.active_notes.drain() {
            let duration = (current_beat - start_beat).max(self.step_division);
            self.recorded_notes.push(RecordedMidiNote {
                channel: ch,
                pitch: note,
                velocity: vel,
                start_beat,
                duration_beats: duration,
            });
        }

        let mut out_sequences = HashMap::new();

        for &track_id in &self.armed_tracks {
            let mut seq = if self.mode == MidiRecordMode::Overdub {
                existing_sequences
                    .get(&track_id)
                    .cloned()
                    .unwrap_or_else(|| SequenceConfig {
                        start_beat: self.record_start_beat,
                        step_division: self.step_division,
                        name: format!("Track {} MIDI", track_id),
                        ..Default::default()
                    })
            } else {
                SequenceConfig {
                    start_beat: self.record_start_beat,
                    step_division: self.step_division,
                    name: format!("Track {} MIDI", track_id),
                    ..Default::default()
                }
            };

            for n in &self.recorded_notes {
                let rel_beat = (n.start_beat - seq.start_beat).max(0.0);
                let step_idx = (rel_beat / seq.step_division).round() as usize;

                while seq.steps.len() <= step_idx {
                    seq.steps.push(TrackerStepConfig {
                        active: false,
                        ..Default::default()
                    });
                }

                seq.steps[step_idx] = TrackerStepConfig {
                    note: n.pitch as f64,
                    velocity: n.velocity,
                    gate: (n.duration_beats / seq.step_division) as f32,
                    active: true,
                    ..Default::default()
                };
            }

            out_sequences.insert(track_id, seq);
        }

        self.recorded_notes.clear();
        out_sequences
    }

    /// Capture retrospective MIDI into a new sequence.
    pub fn capture_retrospective(&self) -> Option<SequenceConfig> {
        self.retrospective_buffer
            .capture_as_sequence(self.quantize, self.step_division)
    }
}

impl Default for MidiRecorder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_record_and_overdub() {
        let mut recorder = MidiRecorder::new();
        recorder.arm_track(1);
        assert!(recorder.is_track_armed(1));

        // Pass 1: Record 2 notes at beat 0.0 and 1.0
        recorder.start_recording(0.0);
        recorder.process_event(MidiEvent::NoteOn(0, 60, 100), 0.0);
        recorder.process_event(MidiEvent::NoteOff(0, 60, 0), 0.5);

        recorder.process_event(MidiEvent::NoteOn(0, 64, 110), 1.0);
        recorder.process_event(MidiEvent::NoteOff(0, 64, 0), 1.5);

        let pass1 = recorder.stop_recording(2.0, &HashMap::new());
        let seq1 = pass1.get(&1).expect("Track 1 sequence");
        assert!(seq1.steps.len() >= 5);
        assert_eq!(seq1.steps[0].note, 60.0);
        assert!(seq1.steps[0].active);
        // Step 4 is beat 1.0 at 0.25 division
        assert_eq!(seq1.steps[4].note, 64.0);
        assert!(seq1.steps[4].active);

        // Pass 2: Overdub a note at beat 0.5 (step 2)
        recorder.set_mode(MidiRecordMode::Overdub);
        recorder.start_recording(0.0);
        recorder.process_event(MidiEvent::NoteOn(0, 67, 90), 0.5);
        recorder.process_event(MidiEvent::NoteOff(0, 67, 0), 0.75);

        let pass2 = recorder.stop_recording(2.0, &pass1);
        let seq2 = pass2.get(&1).expect("Overdubbed sequence");

        // Both original notes AND overdubbed note exist!
        assert_eq!(seq2.steps[0].note, 60.0);
        assert_eq!(seq2.steps[2].note, 67.0);
        assert_eq!(seq2.steps[4].note, 64.0);
    }

    #[test]
    fn test_quantize_on_record() {
        let mut recorder = MidiRecorder::new();
        recorder.arm_track(2);
        recorder.set_quantize(QuantizeGrid::Sixteenth, 1.0);

        recorder.start_recording(0.0);
        // Slightly early note at beat 0.23 -> should snap to 0.25 (step 1)
        recorder.process_event(MidiEvent::NoteOn(0, 62, 100), 0.23);
        recorder.process_event(MidiEvent::NoteOff(0, 62, 0), 0.45);

        let res = recorder.stop_recording(1.0, &HashMap::new());
        let seq = res.get(&2).expect("Track 2 sequence");

        assert_eq!(seq.steps.len(), 2);
        assert!(!seq.steps[0].active);
        assert!(seq.steps[1].active);
        assert_eq!(seq.steps[1].note, 62.0);
    }

    #[test]
    fn test_step_input() {
        let mut controller = StepInputController::new(0.25);
        let mut seq = SequenceConfig::default();

        let s0 = controller.input_note(&mut seq, 60, Some(0.9));
        assert_eq!(s0, 0);

        let r1 = controller.input_rest(&mut seq);
        assert_eq!(r1, 1);

        let s2 = controller.input_note(&mut seq, 67, None);
        assert_eq!(s2, 2);

        assert_eq!(seq.steps.len(), 3);
        assert!(seq.steps[0].active);
        assert_eq!(seq.steps[0].note, 60.0);
        assert!(!seq.steps[1].active); // rest
        assert!(seq.steps[2].active);
        assert_eq!(seq.steps[2].note, 67.0);

        // Backspace
        controller.backspace(&mut seq);
        assert_eq!(controller.cursor_step, 2);
        assert!(!seq.steps[2].active);
    }

    #[test]
    fn test_retrospective_midi_capture() {
        let mut recorder = MidiRecorder::new();

        // Transport is NOT recording! But performer plays notes into the DAW
        recorder.process_event(MidiEvent::NoteOn(0, 60, 100), 4.0);
        recorder.process_event(MidiEvent::NoteOff(0, 60, 0), 4.5);

        recorder.process_event(MidiEvent::NoteOn(0, 65, 105), 5.0);
        recorder.process_event(MidiEvent::NoteOff(0, 65, 0), 5.5);

        // Now user says "Capture MIDI"!
        let captured = recorder.capture_retrospective().expect("Captured sequence");
        assert_eq!(captured.start_beat, 4.0);
        assert!(captured.steps.len() >= 5);
        assert!(captured.steps[0].active);
        assert_eq!(captured.steps[0].note, 60.0);
        assert!(captured.steps[4].active);
        assert_eq!(captured.steps[4].note, 65.0);
    }
}
