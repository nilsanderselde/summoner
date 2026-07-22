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

//! Polymetric and variable time signature tracker sequence engine.

use crate::transport::Transport;

/// Time signature representation with numerator and denominator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeSignature {
    pub num: u8,
    pub den: u8,
}

impl TimeSignature {
    pub fn new(num: u8, den: u8) -> Self {
        Self { num, den }
    }

    /// Calculate number of quarter-note equivalent beats per measure.
    pub fn beats_per_measure(&self) -> f64 {
        if self.den == 0 {
            return 0.0;
        }
        self.num as f64 * (4.0 / self.den as f64)
    }
}

impl Default for TimeSignature {
    fn default() -> Self {
        Self::new(4, 4)
    }
}

/// An individual step in a tracker sequence track.
#[derive(Debug, Clone, PartialEq)]
pub struct TrackerStep {
    pub note: f64,
    pub velocity: f32,
    pub gate: f32,
    pub probability: f32,
    pub active: bool,
}

impl TrackerStep {
    pub fn new(note: f64, velocity: f32, gate: f32) -> Self {
        Self {
            note,
            velocity,
            gate,
            probability: 1.0,
            active: true,
        }
    }

    pub fn empty() -> Self {
        Self {
            note: 60.0,
            velocity: 0.0,
            gate: 0.0,
            probability: 1.0,
            active: false,
        }
    }
}

impl Default for TrackerStep {
    fn default() -> Self {
        Self::empty()
    }
}

/// Sequence track supporting polymetric step counts and independent time signatures.
#[derive(Debug, Clone, PartialEq)]
pub struct SequenceTrack {
    pub id: u64,
    pub name: String,
    pub time_signature: TimeSignature,
    pub step_division: f64, // Step duration in musical beats (e.g. 0.25 for 1/16th)
    pub steps: Vec<TrackerStep>,
}

impl SequenceTrack {
    pub fn new(id: u64, name: impl Into<String>, step_division: f64, steps: Vec<TrackerStep>) -> Self {
        Self {
            id,
            name: name.into(),
            time_signature: TimeSignature::default(),
            step_division: if step_division <= 0.0 { 0.25 } else { step_division },
            steps,
        }
    }

    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Calculate step index and step offset from total elapsed beats.
    pub fn step_at_beat(&self, beat: f64) -> (usize, f64) {
        if self.steps.is_empty() || self.step_division <= 0.0 {
            return (0, 0.0);
        }
        let total_steps = beat / self.step_division;
        let step_idx = (total_steps.floor() as usize) % self.steps.len();
        let phase = total_steps % 1.0;
        (step_idx, phase)
    }
}

/// Trigger event emitted by tracker sequencer during evaluation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SequenceEvent {
    pub track_id: u64,
    pub step_index: usize,
    pub note: f64,
    pub velocity: f32,
    pub gate: f32,
    pub frame_offset: u64,
}

/// Polymetric tracker sequence engine managing multiple sequence tracks.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct PolymetricSequencer {
    pub tracks: Vec<SequenceTrack>,
}

impl PolymetricSequencer {
    pub fn new() -> Self {
        Self { tracks: Vec::new() }
    }

    pub fn add_track(&mut self, track: SequenceTrack) {
        self.tracks.push(track);
    }

    /// Evaluate timeline slice `[start_frame, start_frame + frame_count)` and append events to `out_events`.
    /// Zero allocations when `out_events` capacity is pre-allocated.
    pub fn evaluate(
        &self,
        transport: &Transport,
        frame_count: u64,
        out_events: &mut Vec<SequenceEvent>,
    ) {
        if !transport.is_playing || transport.sample_rate == 0 || transport.bpm <= 0.0 {
            return;
        }

        let start_frame = transport.frame_position;
        let end_frame = start_frame + frame_count;
        let seconds_per_frame = 1.0 / transport.sample_rate as f64;
        let beats_per_second = transport.bpm / 60.0;
        let beats_per_frame = seconds_per_frame * beats_per_second;

        let start_beat = (start_frame as f64) * beats_per_frame;
        let end_beat = (end_frame as f64) * beats_per_frame;

        for track in &self.tracks {
            if track.steps.is_empty() || track.step_division <= 0.0 {
                continue;
            }

            let start_step_total = (start_beat / track.step_division).ceil() as u64;
            let end_step_total = (end_beat / track.step_division).ceil() as u64;

            for step_num in start_step_total..end_step_total {
                let step_beat = step_num as f64 * track.step_division;
                let frame = (step_beat / beats_per_frame).round() as u64;

                if frame >= start_frame && frame < end_frame {
                    let step_idx = (step_num as usize) % track.steps.len();
                    let step = &track.steps[step_idx];

                    if step.active && step.velocity > 0.0 {
                        let frame_offset = frame - start_frame;
                        out_events.push(SequenceEvent {
                            track_id: track.id,
                            step_index: step_idx,
                            note: step.note,
                            velocity: step.velocity,
                            gate: step.gate,
                            frame_offset,
                        });
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_signature_beats() {
        let ts_44 = TimeSignature::new(4, 4);
        assert_eq!(ts_44.beats_per_measure(), 4.0);

        let ts_78 = TimeSignature::new(7, 8);
        assert_eq!(ts_78.beats_per_measure(), 3.5);

        let ts_34 = TimeSignature::new(3, 4);
        assert_eq!(ts_34.beats_per_measure(), 3.0);
    }

    #[test]
    fn test_polymetric_step_wrap() {
        let step_div = 0.25; // 16th notes
        let steps_7 = vec![TrackerStep::new(60.0, 0.8, 0.5); 7];
        let track = SequenceTrack::new(1, "7-step Track", step_div, steps_7);

        assert_eq!(track.step_count(), 7);

        // Beat 0.0 -> Step 0
        let (idx0, _) = track.step_at_beat(0.0);
        assert_eq!(idx0, 0);

        // Beat 1.5 (6th 16th note) -> Step 6
        let (idx6, _) = track.step_at_beat(1.5);
        assert_eq!(idx6, 6);

        // Beat 1.75 (7th 16th note) -> Wraps to Step 0
        let (idx_wrap, _) = track.step_at_beat(1.75);
        assert_eq!(idx_wrap, 0);
    }

    #[test]
    fn test_sequencer_evaluation() {
        let mut transport = Transport::new(44100, 120.0);
        transport.play();

        let mut seq = PolymetricSequencer::new();
        let track_a = SequenceTrack::new(
            1,
            "Track A (4 steps)",
            0.25,
            vec![
                TrackerStep::new(60.0, 0.8, 0.5),
                TrackerStep::empty(),
                TrackerStep::new(64.0, 0.8, 0.5),
                TrackerStep::empty(),
            ],
        );
        let track_b = SequenceTrack::new(
            2,
            "Track B (3 steps)",
            0.25,
            vec![
                TrackerStep::new(72.0, 0.9, 0.5),
                TrackerStep::new(74.0, 0.9, 0.5),
                TrackerStep::empty(),
            ],
        );

        seq.add_track(track_a);
        seq.add_track(track_b);

        let mut events = Vec::new();
        // Evaluate first quarter note (starts at beat 0, includes beat 0.25 trigger)
        let frames_per_beat = (44100.0 * 60.0 / 120.0) as u64; // 22050 frames
        seq.evaluate(&transport, frames_per_beat, &mut events);

        // At frame 0: Track A step 0 (note 60), Track B step 0 (note 72)
        // At frame 5513 (beat 0.25): Track B step 1 (note 74)
        assert!(!events.is_empty());
        assert!(events.iter().any(|e| e.track_id == 1 && e.note == 60.0));
        assert!(events.iter().any(|e| e.track_id == 2 && e.note == 72.0));
        assert!(events.iter().any(|e| e.track_id == 2 && e.note == 74.0));
    }
}
