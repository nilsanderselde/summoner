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

pub mod automation;
pub mod generative;
pub mod pattern;
pub mod automation_timeline;
pub mod timeline;

#[cfg(test)]
mod tests {
    use super::*;
    use pattern::{PatternClip, PatternStep};
    use timeline::{Clip, ClipContent, PatternNote, TimelineArranger};
    use summoner_project::schema::{SequenceConfig, TrackConfig, TrackerStepConfig};

    #[test]
    fn test_track_multiple_clips() {
        let mut track = TrackConfig {
            id: 1,
            name: "Lead Synth".to_string(),
            channels: 2,
            gain: 1.0,
            pan: 0.0,
            muted: false,
            soloed: false,
            send_level: 0.0,
            nodes: Vec::new(),
            sequence: Some(SequenceConfig {
                start_beat: 0.0,
                step_division: 0.25,
                clip_color: None,
                clip_name: Some("Main Verse".to_string()),
                name: "Verse".to_string(),
                is_unique: false,
                steps: vec![TrackerStepConfig {
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
                }; 16],
                ..Default::default()
            }),
            clips: vec![SequenceConfig {
                start_beat: 4.0,
                step_division: 0.25,
                clip_color: None,
                clip_name: Some("Chorus".to_string()),
                name: "Chorus".to_string(),
                is_unique: true,
                steps: vec![TrackerStepConfig {
                    note: 64.0,
                    velocity: 0.9,
                    gate: 0.5,
                    probability: 1.0,
                    ratchet: 1,
                    micro_shift: 0,
                    swing: 0.0,
                    pan: 0.0,
                    pitch_offset: 0.0,
                    active: true,
                }; 16],
                ..Default::default()
            }],
            connections: Vec::new(),
            tuning_edo: None,
            tuning_root_hz: None,
            tuning_scl_path: None,
            ..Default::default()
        };

        let seqs = track.all_sequences();
        assert_eq!(seqs.len(), 2, "Track should return all 2 sequences");
        assert_eq!(seqs[0].clip_name.as_deref(), Some("Main Verse"));
        assert_eq!(seqs[1].clip_name.as_deref(), Some("Chorus"));
        assert!(seqs[1].is_unique);
    }

    #[test]
    fn test_clip_duplicate_and_make_unique() {
        let mut clip = PatternClip::new("Lead Pattern", 4.0);
        clip.add_step(PatternStep::default());
        assert!(!clip.is_unique);

        let dup = clip.duplicate();
        assert_eq!(dup.start_beat, 4.0);
        assert_eq!(dup.name, "Lead Pattern (Copy)");

        clip.make_unique();
        assert!(clip.is_unique);
    }

    #[test]
    fn test_timeline_step_swing_and_pan_pitch() {
        let mut arranger = TimelineArranger::new();
        // Offbeat note at beat 0.25 (step index 1)
        let note = PatternNote {
            start_beat: 0.25,
            length_beats: 0.25,
            note: 60.0,
            velocity: 0.8,
            probability: 1.0,
            ratchet: 1,
            micro_shift: 0,
            swing: 0.5,
            pan: -0.5,
            pitch_offset: 25.0,
        };
        arranger.add_clip(Clip {
            start_beat: 0.0,
            length_beats: 4.0,
            track_id: 1,
            content: ClipContent::Pattern(vec![note]),
        });

        // Evaluate around expected delayed beat: 0.25 + (0.5 * 0.125) = 0.3125
        let eval = arranger.evaluate(0.0, 1.0);
        assert_eq!(eval.note_events.len(), 1);
        let ev = eval.note_events[0];
        assert_eq!(ev.pan, -0.5);
        assert_eq!(ev.pitch_offset, 25.0);
        assert_eq!(ev.note, 60.25);
        assert!((ev.beat_offset - 0.3125).abs() < 1e-5, "Swing delay should shift beat to ~0.3125");
    }

    #[test]
    fn test_polyrhythmic_track_lengths() {
        // Poly-rhythm: 3 steps vs 4 steps
        let seq_3 = vec![TrackerStepConfig { note: 60.0, velocity: 0.8, gate: 0.5, probability: 1.0, ratchet: 1, micro_shift: 0, swing: 0.0, pan: 0.0, pitch_offset: 0.0, active: true }; 3];
        let seq_4 = vec![TrackerStepConfig { note: 67.0, velocity: 0.8, gate: 0.5, probability: 1.0, ratchet: 1, micro_shift: 0, swing: 0.0, pan: 0.0, pitch_offset: 0.0, active: true }; 4];

        let len_3 = seq_3.len() as f64 * 0.25;
        let len_4 = seq_4.len() as f64 * 0.25;

        assert_eq!(len_3, 0.75);
        assert_eq!(len_4, 1.0);
    }

    #[test]
    fn test_euclidean_rhythm_generation() {
        let rhythm = generative::GenerativeEngine::euclidean_rhythm(3, 8);
        assert_eq!(rhythm.len(), 8);
        let count = rhythm.iter().filter(|&&b| b).count();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_pattern_shift_rotate_reverse_mirror() {
        let mut steps = vec![
            TrackerStepConfig { note: 60.0, velocity: 0.8, gate: 0.5, probability: 1.0, ratchet: 1, micro_shift: 0, swing: 0.0, pan: 0.0, pitch_offset: 0.0, active: true },
            TrackerStepConfig { note: 62.0, velocity: 0.8, gate: 0.0, probability: 1.0, ratchet: 1, micro_shift: 0, swing: 0.0, pan: 0.0, pitch_offset: 0.0, active: false },
            TrackerStepConfig { note: 64.0, velocity: 0.8, gate: 0.5, probability: 1.0, ratchet: 1, micro_shift: 0, swing: 0.0, pan: 0.0, pitch_offset: 0.0, active: true },
        ];

        steps.rotate_left(1);
        assert_eq!(steps[0].note, 62.0);

        steps.rotate_right(1);
        assert_eq!(steps[0].note, 60.0);

        steps.reverse();
        assert_eq!(steps[0].note, 64.0);

        let was_active_0 = steps[0].active;
        for step in &mut steps {
            step.active = !step.active;
        }
        assert_eq!(steps[0].active, !was_active_0);
    }
}
