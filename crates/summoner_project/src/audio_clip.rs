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

//! Audio Clip and Region Arranger Data Model (Epic C, Task C1).
//!
//! Provides rich audio clip descriptions on track lanes, including:
//! - Source audio asset ID & file path
//! - Start beat and duration in beats
//! - Offset and length in sample frames
//! - Fades (linear, exponential, logarithmic, S-curve) and crossfades
//! - Non-destructive trim, slip, split, gain, pan, reversing, muting, and looping.

use serde::{Deserialize, Serialize};

fn default_gain_1() -> f32 {
    1.0
}

fn default_gain_1_f64() -> f64 {
    1.0
}

/// Curvature characteristic of a volume fade.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum FadeCurve {
    #[default]
    Linear,
    Exponential,
    Logarithmic,
    SCurve,
}

impl FadeCurve {
    /// Evaluate the fade multiplier in the range `[0.0, 1.0]` for input progress `t \in [0.0, 1.0]`.
    pub fn evaluate(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            FadeCurve::Linear => t,
            FadeCurve::Exponential => t * t,
            FadeCurve::Logarithmic => {
                if t <= 0.0 {
                    0.0
                } else {
                    (t * 9.0 + 1.0).log10()
                }
            }
            FadeCurve::SCurve => t * t * (3.0 - 2.0 * t),
        }
    }
}

/// Lane layout descriptor within an arranger track.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArrangerLaneConfig {
    pub id: String,
    pub name: String,
    pub lane_index: usize,
    #[serde(default)]
    pub muted: bool,
    #[serde(default)]
    pub soloed: bool,
    #[serde(default = "default_lane_height")]
    pub height_px: f32,
}

fn default_lane_height() -> f32 {
    64.0
}

impl Default for ArrangerLaneConfig {
    fn default() -> Self {
        Self {
            id: "lane_0".to_string(),
            name: "Lane 1".to_string(),
            lane_index: 0,
            muted: false,
            soloed: false,
            height_px: 64.0,
        }
    }
}

/// Audio clip / region definition residing on an arranger track lane.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioClipConfig {
    pub id: String,
    pub name: String,
    pub asset_id: String,
    #[serde(default)]
    pub file_path: Option<String>,
    pub start_beat: f64,
    pub duration_beats: f64,
    #[serde(default)]
    pub offset_frames: u64,
    #[serde(default)]
    pub duration_frames: Option<u64>,
    #[serde(default = "default_gain_1")]
    pub gain: f32,
    #[serde(default)]
    pub pan: f32,
    #[serde(default)]
    pub fade_in_beats: f64,
    #[serde(default)]
    pub fade_out_beats: f64,
    #[serde(default)]
    pub fade_in_curve: FadeCurve,
    #[serde(default)]
    pub fade_out_curve: FadeCurve,
    #[serde(default)]
    pub crossfade_with_clip_id: Option<String>,
    #[serde(default)]
    pub loop_enabled: bool,
    #[serde(default)]
    pub loop_length_beats: f64,
    #[serde(default)]
    pub is_reversed: bool,
    #[serde(default)]
    pub muted: bool,
    #[serde(default)]
    pub color: Option<[u8; 3]>,
    #[serde(default)]
    pub lane_index: usize,
    #[serde(default = "default_gain_1_f64")]
    pub time_stretch_ratio: f64,
    #[serde(default)]
    pub pitch_shift_semitones: f64,
}

impl AudioClipConfig {
    /// Create a new standard audio clip.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        asset_id: impl Into<String>,
        start_beat: f64,
        duration_beats: f64,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            asset_id: asset_id.into(),
            file_path: None,
            start_beat,
            duration_beats,
            offset_frames: 0,
            duration_frames: None,
            gain: 1.0,
            pan: 0.0,
            fade_in_beats: 0.0,
            fade_out_beats: 0.0,
            fade_in_curve: FadeCurve::Linear,
            fade_out_curve: FadeCurve::Linear,
            crossfade_with_clip_id: None,
            loop_enabled: false,
            loop_length_beats: duration_beats,
            is_reversed: false,
            muted: false,
            color: None,
            lane_index: 0,
            time_stretch_ratio: 1.0,
            pitch_shift_semitones: 0.0,
        }
    }

    /// End beat on the timeline.
    #[inline]
    pub fn end_beat(&self) -> f64 {
        self.start_beat + self.duration_beats
    }

    /// Test if a beat falls within this clip's timeline boundary.
    #[inline]
    pub fn contains_beat(&self, beat: f64) -> bool {
        beat >= self.start_beat && beat < self.end_beat()
    }

    /// Calculate the effective gain multiplier at a specific beat, factoring in fade-in, fade-out, and mute.
    pub fn gain_at_beat(&self, beat: f64) -> f32 {
        if self.muted || !self.contains_beat(beat) {
            return 0.0;
        }

        let local_beat = beat - self.start_beat;
        let mut multiplier = self.gain;

        // Fade in
        if self.fade_in_beats > 1e-6 && local_beat < self.fade_in_beats {
            let t = (local_beat / self.fade_in_beats) as f32;
            multiplier *= self.fade_in_curve.evaluate(t);
        }

        // Fade out
        let time_to_end = self.duration_beats - local_beat;
        if self.fade_out_beats > 1e-6 && time_to_end < self.fade_out_beats {
            let t = (time_to_end / self.fade_out_beats) as f32;
            multiplier *= self.fade_out_curve.evaluate(t);
        }

        multiplier
    }

    /// Split this clip at a specified timeline beat into two non-destructive sub-clips.
    pub fn split_at_beat(
        &self,
        split_beat: f64,
        sample_rate: u32,
        bpm: f64,
    ) -> Result<(AudioClipConfig, AudioClipConfig), String> {
        if split_beat <= self.start_beat || split_beat >= self.end_beat() {
            return Err(format!(
                "Split beat {} outside clip range [{}, {})",
                split_beat,
                self.start_beat,
                self.end_beat()
            ));
        }

        let first_duration_beats = split_beat - self.start_beat;
        let second_duration_beats = self.duration_beats - first_duration_beats;

        // Frames per beat = (60.0 / bpm) * sample_rate
        let frames_per_beat = (60.0 / bpm.max(1.0)) * sample_rate as f64;
        let frames_delta = (first_duration_beats * frames_per_beat).round() as u64;

        let mut first = self.clone();
        first.id = format!("{}_part1", self.id);
        first.duration_beats = first_duration_beats;
        first.fade_out_beats = self.fade_out_beats.min(first_duration_beats);

        let mut second = self.clone();
        second.id = format!("{}_part2", self.id);
        second.start_beat = split_beat;
        second.duration_beats = second_duration_beats;
        second.offset_frames = self.offset_frames + frames_delta;
        second.fade_in_beats = self.fade_in_beats.min(second_duration_beats);

        Ok((first, second))
    }

    /// Configure a crossfade between this clip and an adjacent clip.
    pub fn create_crossfade(&mut self, next_clip: &mut AudioClipConfig, crossfade_beats: f64) {
        self.fade_out_beats = crossfade_beats;
        self.fade_out_curve = FadeCurve::SCurve;
        self.crossfade_with_clip_id = Some(next_clip.id.clone());

        next_clip.fade_in_beats = crossfade_beats;
        next_clip.fade_in_curve = FadeCurve::SCurve;
        next_clip.crossfade_with_clip_id = Some(self.id.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_clip_creation_and_bounds() {
        let clip = AudioClipConfig::new("clip1", "Vocal 1", "asset_vocal", 4.0, 8.0);
        assert_eq!(clip.start_beat, 4.0);
        assert_eq!(clip.duration_beats, 8.0);
        assert_eq!(clip.end_beat(), 12.0);
        assert!(clip.contains_beat(4.0));
        assert!(clip.contains_beat(8.0));
        assert!(clip.contains_beat(11.99));
        assert!(!clip.contains_beat(12.0));
        assert!(!clip.contains_beat(3.99));
    }

    #[test]
    fn test_audio_clip_fade_curves() {
        let mut clip = AudioClipConfig::new("clip1", "Vocal", "asset1", 0.0, 10.0);
        clip.fade_in_beats = 2.0;
        clip.fade_out_beats = 2.0;
        clip.fade_in_curve = FadeCurve::Linear;
        clip.fade_out_curve = FadeCurve::SCurve;

        // Middle beat should be full gain
        assert!((clip.gain_at_beat(5.0) - 1.0).abs() < 1e-4);

        // Halfway through linear fade in (beat 1.0)
        assert!((clip.gain_at_beat(1.0) - 0.5).abs() < 1e-4);

        // Outside boundary should be 0.0
        assert_eq!(clip.gain_at_beat(11.0), 0.0);

        // Muted clip should be 0.0
        clip.muted = true;
        assert_eq!(clip.gain_at_beat(5.0), 0.0);
    }

    #[test]
    fn test_audio_clip_split() {
        let clip = AudioClipConfig::new("clip1", "Drums", "drum_asset", 0.0, 8.0);
        let (first, second) = clip.split_at_beat(4.0, 44100, 120.0).unwrap();

        assert_eq!(first.start_beat, 0.0);
        assert_eq!(first.duration_beats, 4.0);
        assert_eq!(second.start_beat, 4.0);
        assert_eq!(second.duration_beats, 4.0);
        assert!(second.offset_frames > 0);

        // Split outside range should err
        assert!(clip.split_at_beat(9.0, 44100, 120.0).is_err());
    }

    #[test]
    fn test_audio_clip_crossfade() {
        let mut clip_a = AudioClipConfig::new("clipA", "Guitar A", "gtr_asset", 0.0, 4.0);
        let mut clip_b = AudioClipConfig::new("clipB", "Guitar B", "gtr_asset", 3.0, 5.0);

        clip_a.create_crossfade(&mut clip_b, 1.0);
        assert_eq!(clip_a.fade_out_beats, 1.0);
        assert_eq!(clip_a.crossfade_with_clip_id.as_deref(), Some("clipB"));
        assert_eq!(clip_b.fade_in_beats, 1.0);
        assert_eq!(clip_b.crossfade_with_clip_id.as_deref(), Some("clipA"));
    }
}

