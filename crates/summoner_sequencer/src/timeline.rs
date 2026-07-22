// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde

use std::collections::HashMap;
use crate::automation_timeline::AutomationCurve;

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
}

#[derive(Debug, Default)]
pub struct TimelineArranger {
    pub clips: Vec<Clip>,
}

impl TimelineArranger {
    pub fn new() -> Self {
        Self { clips: Vec::new() }
    }

    pub fn add_clip(&mut self, clip: Clip) {
        self.clips.push(clip);
    }

    pub fn evaluate(&self, current_beat: f64, beats_per_block: f64) -> TimelineEvaluation {
        let mut eval = TimelineEvaluation::default();
        let end_beat = current_beat + beats_per_block;

        for clip in &self.clips {
            let clip_end = clip.start_beat + clip.length_beats;
            // Overlaps?
            if current_beat < clip_end && end_beat >= clip.start_beat {
                match &clip.content {
                    ClipContent::Pattern(notes) => {
                        for note in notes {
                            let abs_start = clip.start_beat + note.start_beat;
                            if abs_start >= current_beat && abs_start < end_beat {
                                eval.note_events.push(NoteEvent {
                                    track_id: clip.track_id,
                                    note: note.note,
                                    velocity: note.velocity,
                                    beat_offset: abs_start - current_beat,
                                });
                            }
                        }
                    },
                    ClipContent::Automation(curve) => {
                        // Sample it at start of block
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
    pub beat_offset: f64, // From current_beat
}
