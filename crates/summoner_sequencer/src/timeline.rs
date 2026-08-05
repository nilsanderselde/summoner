// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde

use crate::automation_timeline::AutomationCurve;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Clip {
    pub start_beat: f64,
    pub length_beats: f64,
    pub track_id: u64,
    pub content: ClipContent,
}

#[derive(Debug, Clone)]
pub enum ClipContent {
    Pattern(Vec<PatternNote>),
    Automation(AutomationCurve),
}

#[derive(Debug, Clone, Copy)]
pub struct PatternNote {
    pub start_beat: f64, // relative to clip
    pub length_beats: f64,
    pub note: f64,
    pub velocity: f32,
    pub probability: f32,
    pub ratchet: u32,
    pub micro_shift: i32,
    pub swing: f32,
    pub pan: f32,
    pub pitch_offset: f32,
}

impl PatternNote {
    pub fn simple(start_beat: f64, length_beats: f64, note: f64, velocity: f32) -> Self {
        Self {
            start_beat,
            length_beats,
            note,
            velocity,
            probability: 1.0,
            ratchet: 1,
            micro_shift: 0,
            swing: 0.0,
            pan: 0.0,
            pitch_offset: 0.0,
        }
    }
}

#[derive(Debug)]
pub struct TimelineArranger {
    pub clips: Vec<Clip>,
    seed: u64,
}

impl Default for TimelineArranger {
    fn default() -> Self {
        Self::new()
    }
}

impl TimelineArranger {
    pub fn new() -> Self {
        Self {
            clips: Vec::new(),
            seed: 0x123456789ABCDEF0,
        }
    }

    pub fn add_clip(&mut self, clip: Clip) {
        self.clips.push(clip);
    }

    fn next_prng_seed(seed: &mut u64) -> f32 {
        *seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (*seed >> 33) as f32 / 2147483648.0
    }

    pub fn evaluate(&mut self, current_beat: f64, beats_per_block: f64) -> TimelineEvaluation {
        let mut eval = TimelineEvaluation::default();
        let end_beat = current_beat + beats_per_block;

        for clip in &self.clips {
            let clip_end = clip.start_beat + clip.length_beats;
            if current_beat < clip_end && end_beat >= clip.start_beat {
                match &clip.content {
                    ClipContent::Pattern(notes) => {
                        for note in notes {
                            // Check probability
                            if note.probability < 1.0 {
                                let roll = Self::next_prng_seed(&mut self.seed);
                                if roll > note.probability {
                                    continue;
                                }
                            }

                            // Calculate swing timing shift (Step 365)
                            // If note falls on off-beat sub-division (e.g. step index odd), apply swing delay
                            let is_offbeat = ((note.start_beat * 4.0).round() as i64 % 2) != 0;
                            let swing_offset_beats = if is_offbeat {
                                note.swing as f64 * 0.125
                            } else {
                                0.0
                            };

                            let micro_offset_beats =
                                note.micro_shift as f64 * 0.001 + swing_offset_beats;
                            let base_start = clip.start_beat + note.start_beat + micro_offset_beats;

                            let ratchet_count = note.ratchet.max(1);
                            let sub_step_dur = note.length_beats / ratchet_count as f64;

                            for r in 0..ratchet_count {
                                let abs_start = base_start + r as f64 * sub_step_dur;
                                if abs_start >= current_beat && abs_start < end_beat {
                                    let effective_note =
                                        note.note + (note.pitch_offset as f64 / 100.0);
                                    eval.note_events.push(NoteEvent {
                                        track_id: clip.track_id,
                                        note: effective_note,
                                        velocity: note.velocity,
                                        beat_offset: abs_start - current_beat,
                                        pan: note.pan,
                                        pitch_offset: note.pitch_offset,
                                    });
                                }
                            }
                        }
                    }
                    ClipContent::Automation(curve) => {
                        let val = curve.evaluate_at_beat(current_beat - clip.start_beat);
                        eval.automation_values.insert(clip.track_id, val);
                    }
                }
            }
        }
        eval
    }
}

#[derive(Debug, Default)]
pub struct TimelineEvaluation {
    pub note_events: Vec<NoteEvent>,
    pub automation_values: HashMap<u64, f32>,
}

#[derive(Debug, Clone, Copy)]
pub struct NoteEvent {
    pub track_id: u64,
    pub note: f64,
    pub velocity: f32,
    pub beat_offset: f64,
    pub pan: f32,
    pub pitch_offset: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ratchet_and_probability_timeline() {
        let mut arranger = TimelineArranger::new();
        let note = PatternNote {
            start_beat: 0.0,
            length_beats: 1.0,
            note: 60.0,
            velocity: 0.8,
            probability: 1.0,
            ratchet: 4,
            micro_shift: 0,
            swing: 0.0,
            pan: 0.0,
            pitch_offset: 0.0,
        };
        arranger.add_clip(Clip {
            start_beat: 0.0,
            length_beats: 4.0,
            track_id: 1,
            content: ClipContent::Pattern(vec![note]),
        });

        let eval = arranger.evaluate(0.0, 1.0);
        assert_eq!(
            eval.note_events.len(),
            4,
            "Ratchet 4 should create 4 sub-events"
        );
    }
}
