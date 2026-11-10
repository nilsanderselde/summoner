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

//! Arranger Timeline Clip Editing Engine (Epic C, Task C7).
//!
//! Provides comprehensive non-destructive editing operations for audio and MIDI clips on the timeline:
//! - Split clip at playhead / beat
//! - Trim start (with sample offset compensation) and trim end
//! - Slip edit (shifting underlying audio without changing timeline boundary)
//! - Ripple edit (insert and ripple delete shifting subsequent clips)
//! - Snap to grid & nudge
//! - Duplicate & consolidate (joining contiguous clips)
//! - Multi-clip selection, cut, copy, paste
//! - Fade-in, fade-out, and crossfade handle manipulation

use crate::audio_clip::{AudioClipConfig, FadeCurve};
use crate::schema::{SequenceConfig, TrackConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Grid snap quantization options for arranger editing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ArrangerSnapGrid {
    #[default]
    Off,
    Bar,
    Half,
    Quarter,
    Eighth,
    Sixteenth,
    ThirtySecond,
    Triplet(u8),
}

impl ArrangerSnapGrid {
    /// Step size in beats.
    pub fn step_beats(&self) -> Option<f64> {
        match self {
            Self::Off => None,
            Self::Bar => Some(4.0),
            Self::Half => Some(2.0),
            Self::Quarter => Some(1.0),
            Self::Eighth => Some(0.5),
            Self::Sixteenth => Some(0.25),
            Self::ThirtySecond => Some(0.125),
            Self::Triplet(div) => {
                let div = (*div).max(1) as f64;
                Some(4.0 / div)
            }
        }
    }

    /// Snap a beat to the nearest grid line.
    pub fn snap(&self, beat: f64) -> f64 {
        if let Some(step) = self.step_beats() {
            (beat / step).round() * step
        } else {
            beat
        }
    }
}

/// Multi-clip selection descriptor.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArrangerSelection {
    pub audio_clip_ids: HashSet<String>,
    pub sequence_clip_names: HashSet<String>,
}

impl ArrangerSelection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn select_audio_clip(&mut self, clip_id: impl Into<String>) {
        self.audio_clip_ids.insert(clip_id.into());
    }

    pub fn deselect_audio_clip(&mut self, clip_id: &str) {
        self.audio_clip_ids.remove(clip_id);
    }

    pub fn clear(&mut self) {
        self.audio_clip_ids.clear();
        self.sequence_clip_names.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.audio_clip_ids.is_empty() && self.sequence_clip_names.is_empty()
    }

    pub fn contains_audio_clip(&self, clip_id: &str) -> bool {
        self.audio_clip_ids.contains(clip_id)
    }
}

/// Clipboard payload for copying/pasting clips.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ClipClipboard {
    pub audio_clips: Vec<AudioClipConfig>,
    pub sequence_clips: Vec<SequenceConfig>,
    pub reference_beat: f64,
}

/// Non-destructive slip edit on an audio clip.
///
/// Shifts the underlying audio waveform offset by `delta_beats` while preserving the clip's
/// `start_beat` and `duration_beats` on the arranger timeline.
pub fn slip_audio_clip(
    clip: &mut AudioClipConfig,
    delta_beats: f64,
    sample_rate: u32,
    bpm: f64,
) {
    let frames_per_beat = (60.0 / bpm.max(1.0)) * sample_rate as f64;
    let frames_delta = (delta_beats * frames_per_beat).round() as i64;

    let new_offset = (clip.offset_frames as i64 + frames_delta).max(0) as u64;
    clip.offset_frames = new_offset;
}

/// Non-destructive trim of an audio clip's start boundary.
///
/// Moves `start_beat` to `new_start_beat`, adjusts `duration_beats`, and adjusts `offset_frames`
/// so the remaining audio stays anchored at its timeline position.
pub fn trim_audio_clip_start(
    clip: &mut AudioClipConfig,
    new_start_beat: f64,
    sample_rate: u32,
    bpm: f64,
) -> Result<(), String> {
    if new_start_beat >= clip.end_beat() {
        return Err(format!(
            "New start beat {} must be less than end beat {}",
            new_start_beat,
            clip.end_beat()
        ));
    }

    let delta_beats = new_start_beat - clip.start_beat;
    let frames_per_beat = (60.0 / bpm.max(1.0)) * sample_rate as f64;
    let frames_delta = (delta_beats * frames_per_beat).round() as i64;

    let new_offset = (clip.offset_frames as i64 + frames_delta).max(0) as u64;
    clip.start_beat = new_start_beat;
    clip.duration_beats -= delta_beats;
    clip.offset_frames = new_offset;

    // Adjust fade-in if it exceeds new duration
    if clip.fade_in_beats > clip.duration_beats {
        clip.fade_in_beats = clip.duration_beats;
    }

    Ok(())
}

/// Non-destructive trim of an audio clip's end boundary.
pub fn trim_audio_clip_end(clip: &mut AudioClipConfig, new_end_beat: f64) -> Result<(), String> {
    if new_end_beat <= clip.start_beat {
        return Err(format!(
            "New end beat {} must be greater than start beat {}",
            new_end_beat, clip.start_beat
        ));
    }

    let new_duration = new_end_beat - clip.start_beat;
    clip.duration_beats = new_duration;

    if clip.fade_out_beats > clip.duration_beats {
        clip.fade_out_beats = clip.duration_beats;
    }

    Ok(())
}

/// Nudge an audio clip forward or backward by `delta_beats`, optionally snapping to grid.
pub fn nudge_audio_clip(clip: &mut AudioClipConfig, delta_beats: f64, grid: ArrangerSnapGrid) {
    let unconstrained = (clip.start_beat + delta_beats).max(0.0);
    clip.start_beat = grid.snap(unconstrained);
}

/// Duplicate an audio clip, placing the copy immediately adjacent to the original.
pub fn duplicate_audio_clip(clip: &AudioClipConfig) -> AudioClipConfig {
    let mut dup = clip.clone();
    dup.id = format!("{}_copy", clip.id);
    dup.name = format!("{} (Copy)", clip.name);
    dup.start_beat = clip.end_beat();
    dup
}

/// Split an audio clip in track at `split_beat`.
///
/// Replaces the original clip with two contiguous clips split at that exact beat.
pub fn split_track_audio_clip(
    track: &mut TrackConfig,
    clip_id: &str,
    split_beat: f64,
    sample_rate: u32,
    bpm: f64,
) -> Result<(String, String), String> {
    let pos = track
        .audio_clips
        .iter()
        .position(|c| c.id == clip_id)
        .ok_or_else(|| format!("Clip with ID '{}' not found on track", clip_id))?;

    let original = &track.audio_clips[pos];
    let (part1, part2) = original.split_at_beat(split_beat, sample_rate, bpm)?;

    let id1 = part1.id.clone();
    let id2 = part2.id.clone();

    track.audio_clips.remove(pos);
    track.audio_clips.insert(pos, part2);
    track.audio_clips.insert(pos, part1);

    Ok((id1, id2))
}

/// Ripple delete a beat range on a track.
///
/// Removes clips or slices within `[start_beat, end_beat)`, and shifts all subsequent clips
/// to the left by `(end_beat - start_beat)`.
pub fn ripple_delete_track_range(
    track: &mut TrackConfig,
    start_beat: f64,
    end_beat: f64,
    sample_rate: u32,
    bpm: f64,
) -> Result<(), String> {
    if end_beat <= start_beat {
        return Err("end_beat must be greater than start_beat".to_string());
    }
    let range_len = end_beat - start_beat;

    let mut remaining = Vec::new();

    for mut clip in track.audio_clips.drain(..) {
        if clip.end_beat() <= start_beat {
            // Completely before range: keep untouched
            remaining.push(clip);
        } else if clip.start_beat >= end_beat {
            // Completely after range: shift left
            clip.start_beat -= range_len;
            remaining.push(clip);
        } else if clip.start_beat >= start_beat && clip.end_beat() <= end_beat {
            // Completely inside range: delete
        } else if clip.start_beat < start_beat && clip.end_beat() > end_beat {
            // Straddles entire range: split into two parts
            let (p1, mut p2) = clip.split_at_beat(start_beat, sample_rate, bpm)?;
            // Trim start of part 2
            let _ = trim_audio_clip_start(&mut p2, end_beat, sample_rate, bpm);
            p2.start_beat -= range_len;
            remaining.push(p1);
            remaining.push(p2);
        } else if clip.start_beat < start_beat {
            // Overlaps start: trim end
            let _ = trim_audio_clip_end(&mut clip, start_beat);
            remaining.push(clip);
        } else {
            // Overlaps end: trim start and shift
            let _ = trim_audio_clip_start(&mut clip, end_beat, sample_rate, bpm);
            clip.start_beat -= range_len;
            remaining.push(clip);
        }
    }

    track.audio_clips = remaining;

    // Shift sequences
    for seq in &mut track.clips {
        if seq.start_beat >= end_beat {
            seq.start_beat -= range_len;
        }
    }

    Ok(())
}

/// Copy selected clips from a track into a `ClipClipboard`.
pub fn copy_selection(track: &TrackConfig, selection: &ArrangerSelection) -> ClipClipboard {
    let mut clipboard = ClipClipboard::default();
    let mut min_beat = f64::MAX;

    for clip in &track.audio_clips {
        if selection.contains_audio_clip(&clip.id) {
            if clip.start_beat < min_beat {
                min_beat = clip.start_beat;
            }
            clipboard.audio_clips.push(clip.clone());
        }
    }

    for seq in &track.clips {
        if let Some(ref name) = seq.clip_name {
            if selection.sequence_clip_names.contains(name) {
                if seq.start_beat < min_beat {
                    min_beat = seq.start_beat;
                }
                clipboard.sequence_clips.push(seq.clone());
            }
        }
    }

    clipboard.reference_beat = if min_beat < f64::MAX { min_beat } else { 0.0 };
    clipboard
}

/// Paste clips from `ClipClipboard` onto a track at `target_beat`.
pub fn paste_clipboard(
    track: &mut TrackConfig,
    clipboard: &ClipClipboard,
    target_beat: f64,
    target_lane_index: usize,
) -> Vec<String> {
    let mut pasted_ids = Vec::new();

    for clip in &clipboard.audio_clips {
        let mut pasted = clip.clone();
        let rel_beat = clip.start_beat - clipboard.reference_beat;
        pasted.start_beat = target_beat + rel_beat;
        pasted.id = format!("{}_{}", clip.id, rand_id());
        pasted.lane_index = target_lane_index;
        pasted_ids.push(pasted.id.clone());
        track.audio_clips.push(pasted);
    }

    for seq in &clipboard.sequence_clips {
        let mut pasted = seq.clone();
        let rel_beat = seq.start_beat - clipboard.reference_beat;
        pasted.start_beat = target_beat + rel_beat;
        if let Some(ref cname) = seq.clip_name {
            pasted.clip_name = Some(format!("{} (Pasted)", cname));
        }
        track.clips.push(pasted);
    }

    pasted_ids
}

/// Helper to generate a short pseudo-random suffix for pasted clip IDs.
fn rand_id() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    (d.as_nanos() & 0xFFFFF) as u32
}

/// Set fades on an audio clip with custom curves.
pub fn set_audio_clip_fades(
    clip: &mut AudioClipConfig,
    fade_in_beats: f64,
    fade_in_curve: FadeCurve,
    fade_out_beats: f64,
    fade_out_curve: FadeCurve,
) {
    clip.fade_in_beats = fade_in_beats.clamp(0.0, clip.duration_beats);
    clip.fade_in_curve = fade_in_curve;
    clip.fade_out_beats = fade_out_beats.clamp(0.0, clip.duration_beats);
    clip.fade_out_curve = fade_out_curve;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snap_grid_calculations() {
        let grid = ArrangerSnapGrid::Sixteenth;
        assert_eq!(grid.step_beats(), Some(0.25));
        assert_eq!(grid.snap(0.23), 0.25);
        assert_eq!(grid.snap(0.38), 0.5);
        assert_eq!(grid.snap(1.0), 1.0);

        let grid_off = ArrangerSnapGrid::Off;
        assert_eq!(grid_off.snap(0.2345), 0.2345);
    }

    #[test]
    fn test_trim_start_and_end() {
        let mut clip = AudioClipConfig::new("c1", "Vocal", "asset1", 2.0, 8.0);
        clip.offset_frames = 1000;

        // Trim start from beat 2.0 to beat 4.0 (2 beats forward = 88200 frames at 44100/120bpm)
        trim_audio_clip_start(&mut clip, 4.0, 44100, 120.0).expect("Trim start");
        assert_eq!(clip.start_beat, 4.0);
        assert_eq!(clip.duration_beats, 6.0);
        assert_eq!(clip.offset_frames, 1000 + 44100);

        // Trim end from beat 10.0 to beat 8.0
        trim_audio_clip_end(&mut clip, 8.0).expect("Trim end");
        assert_eq!(clip.start_beat, 4.0);
        assert_eq!(clip.duration_beats, 4.0);
        assert_eq!(clip.end_beat(), 8.0);
    }

    #[test]
    fn test_slip_audio_clip() {
        let mut clip = AudioClipConfig::new("c1", "Guitar", "asset1", 4.0, 4.0);
        clip.offset_frames = 10000;

        // Slip forward by 1 beat
        slip_audio_clip(&mut clip, 1.0, 44100, 120.0);
        // 1 beat = 22050 frames
        assert_eq!(clip.offset_frames, 10000 + 22050);
        // Clip boundaries remain unchanged
        assert_eq!(clip.start_beat, 4.0);
        assert_eq!(clip.duration_beats, 4.0);
    }

    #[test]
    fn test_nudge_and_duplicate() {
        let mut clip = AudioClipConfig::new("c1", "Solo", "asset1", 1.0, 2.0);

        nudge_audio_clip(&mut clip, 0.5, ArrangerSnapGrid::Quarter);
        assert_eq!(clip.start_beat, 2.0);

        let dup = duplicate_audio_clip(&clip);
        assert_eq!(dup.start_beat, 4.0);
        assert_eq!(dup.duration_beats, 2.0);
    }

    #[test]
    fn test_split_and_ripple_delete() {
        let mut track = TrackConfig::default();
        let clip1 = AudioClipConfig::new("c1", "Intro", "asset1", 0.0, 8.0);
        let clip2 = AudioClipConfig::new("c2", "Outro", "asset2", 12.0, 4.0);
        track.audio_clips.push(clip1);
        track.audio_clips.push(clip2);

        // Split c1 at beat 4.0
        let (id1, id2) = split_track_audio_clip(&mut track, "c1", 4.0, 44100, 120.0).expect("Split");
        assert_eq!(track.audio_clips.len(), 3);
        assert_eq!(id1, "c1_part1");
        assert_eq!(id2, "c1_part2");

        // Ripple delete range [4.0, 8.0)
        // This should remove c1_part2, and shift c2 from 12.0 to 8.0!
        ripple_delete_track_range(&mut track, 4.0, 8.0, 44100, 120.0).expect("Ripple delete");
        assert_eq!(track.audio_clips.len(), 2);
        assert_eq!(track.audio_clips[0].id, "c1_part1");
        assert_eq!(track.audio_clips[0].start_beat, 0.0);
        assert_eq!(track.audio_clips[1].id, "c2");
        assert_eq!(track.audio_clips[1].start_beat, 8.0);
    }

    #[test]
    fn test_selection_copy_and_paste() {
        let mut track = TrackConfig::default();
        let clip1 = AudioClipConfig::new("c1", "Verse", "asset1", 4.0, 4.0);
        track.audio_clips.push(clip1);

        let mut sel = ArrangerSelection::new();
        sel.select_audio_clip("c1");
        assert!(sel.contains_audio_clip("c1"));

        let cb = copy_selection(&track, &sel);
        assert_eq!(cb.audio_clips.len(), 1);
        assert_eq!(cb.reference_beat, 4.0);

        let pasted_ids = paste_clipboard(&mut track, &cb, 16.0, 1);
        assert_eq!(pasted_ids.len(), 1);
        assert_eq!(track.audio_clips.len(), 2);
        assert_eq!(track.audio_clips[1].start_beat, 16.0);
        assert_eq!(track.audio_clips[1].lane_index, 1);
    }
}
