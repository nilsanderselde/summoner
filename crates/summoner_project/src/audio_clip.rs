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

fn default_warp_grain_ms() -> f32 {
    35.0
}

/// Time-stretching and audio warping algorithm mode stored on a project audio clip (REC-2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ClipWarpMode {
    #[default]
    Beats,
    Tones,
    Texture,
    Complex,
    RePitch,
    SpectralIdm,
}

/// Persistent warp marker mapping a source audio frame to a timeline beat position (REC-2).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ClipWarpMarker {
    pub source_frame: u64,
    pub target_beat: f64,
    #[serde(default = "default_gain_1")]
    pub transient_strength: f32,
    #[serde(default)]
    pub locked: bool,
}

/// Breakpoint in a clip's per-clip gain automation envelope (REC-4).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GainEnvelopePoint {
    pub beat_offset: f64,
    pub gain: f32,
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
    #[serde(default)]
    pub warp_enabled: bool,
    #[serde(default)]
    pub warp_mode: ClipWarpMode,
    #[serde(default)]
    pub warp_markers: Vec<ClipWarpMarker>,
    #[serde(default = "default_warp_grain_ms")]
    pub warp_grain_ms: f32,
    #[serde(default)]
    pub warp_flux: f32,
    #[serde(default)]
    pub formant_shift_semitones: f32,
    #[serde(default)]
    pub harmonize_to_project_scale: bool,
    #[serde(default)]
    pub gain_envelope: Vec<GainEnvelopePoint>,
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
            warp_enabled: false,
            warp_mode: ClipWarpMode::default(),
            warp_markers: Vec::new(),
            warp_grain_ms: 35.0,
            warp_flux: 0.0,
            formant_shift_semitones: 0.0,
            harmonize_to_project_scale: false,
            gain_envelope: Vec::new(),
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

    /// Calculate the effective gain multiplier at a specific beat, factoring in fades, gain envelope, and mute.
    pub fn gain_at_beat(&self, beat: f64) -> f32 {
        if self.muted || !self.contains_beat(beat) {
            return 0.0;
        }

        let local_beat = beat - self.start_beat;
        let mut multiplier = self.gain;

        // Per-clip piecewise-linear gain automation envelope
        if !self.gain_envelope.is_empty() {
            multiplier *= Self::eval_gain_envelope(&self.gain_envelope, local_beat);
        }

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

    fn eval_gain_envelope(points: &[GainEnvelopePoint], local_beat: f64) -> f32 {
        if points.len() == 1 {
            return points[0].gain.max(0.0);
        }
        if local_beat <= points[0].beat_offset {
            return points[0].gain.max(0.0);
        }
        for w in points.windows(2) {
            if local_beat <= w[1].beat_offset {
                let span = (w[1].beat_offset - w[0].beat_offset).max(1e-9);
                let t = ((local_beat - w[0].beat_offset) / span).clamp(0.0, 1.0) as f32;
                return (w[0].gain + t * (w[1].gain - w[0].gain)).max(0.0);
            }
        }
        points.last().map(|p| p.gain.max(0.0)).unwrap_or(1.0)
    }

    /// Add a warp marker mapping `source_frame` to `target_beat` and enable warping (REC-2).
    pub fn add_warp_marker(&mut self, source_frame: u64, target_beat: f64) {
        self.warp_enabled = true;
        self.warp_markers.push(ClipWarpMarker {
            source_frame,
            target_beat: target_beat.max(0.0),
            transient_strength: 1.0,
            locked: false,
        });
        self.warp_markers.sort_by(|a, b| {
            a.target_beat
                .partial_cmp(&b.target_beat)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Move an existing warp marker to a new `target_beat` (REC-2).
    pub fn move_warp_marker(&mut self, index: usize, new_target_beat: f64) -> bool {
        if let Some(m) = self.warp_markers.get_mut(index) {
            m.target_beat = new_target_beat.max(0.0);
            self.warp_markers.sort_by(|a, b| {
                a.target_beat
                    .partial_cmp(&b.target_beat)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            true
        } else {
            false
        }
    }

    /// Remove a warp marker by index (REC-2).
    pub fn remove_warp_marker(&mut self, index: usize) -> bool {
        if index < self.warp_markers.len() {
            self.warp_markers.remove(index);
            true
        } else {
            false
        }
    }

    /// Snap unlocked warp markers toward the nearest `grid_beats` subdivision with given strength [0..1] (REC-5).
    pub fn quantize_warp_markers(&mut self, grid_beats: f64, strength: f32) {
        if grid_beats <= 1e-6 {
            return;
        }
        let s = strength.clamp(0.0, 1.0) as f64;
        for m in &mut self.warp_markers {
            if !m.locked {
                let grid_target = (m.target_beat / grid_beats).round() * grid_beats;
                m.target_beat = (m.target_beat + (grid_target - m.target_beat) * s).max(0.0);
            }
        }
        self.warp_markers.sort_by(|a, b| {
            a.target_beat
                .partial_cmp(&b.target_beat)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Slip-edit the underlying source audio offset without moving timeline clip bounds (REC-4).
    pub fn slip_edit(&mut self, delta_frames: i64) {
        if delta_frames >= 0 {
            self.offset_frames = self.offset_frames.saturating_add(delta_frames as u64);
        } else {
            self.offset_frames = self
                .offset_frames
                .saturating_sub(delta_frames.unsigned_abs());
        }
    }

    /// Slice this clip at its `warp_markers` into sequential trimmed sub-clips (REC-4, HUM-6).
    pub fn slice_at_warp_markers(&self, sample_rate: u32, bpm: f64) -> Vec<AudioClipConfig> {
        let mut beats: Vec<f64> = self
            .warp_markers
            .iter()
            .filter_map(|m| {
                if m.target_beat > 1e-4 && m.target_beat < self.duration_beats - 1e-4 {
                    Some(self.start_beat + m.target_beat)
                } else {
                    None
                }
            })
            .collect();
        beats.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        beats.dedup_by(|a, b| (*a - *b).abs() < 1e-4);

        if beats.is_empty() {
            return vec![self.clone()];
        }

        let mut slices = Vec::new();
        let mut remainder = self.clone();
        for (idx, cut_beat) in beats.into_iter().enumerate() {
            if cut_beat > remainder.start_beat && cut_beat < remainder.end_beat() {
                if let Ok((mut head, tail)) = remainder.split_at_beat(cut_beat, sample_rate, bpm) {
                    head.id = format!("{}_slice_{}", self.id, idx);
                    slices.push(head);
                    remainder = tail;
                }
            }
        }
        remainder.id = format!("{}_slice_{}", self.id, slices.len());
        slices.push(remainder);
        slices
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

    #[test]
    fn test_warp_markers_quantize_gain_envelope_slip_and_slice() {
        let mut clip = AudioClipConfig::new("clipW", "Breakbeat", "brk", 4.0, 4.0);
        clip.add_warp_marker(22050, 1.15);
        clip.add_warp_marker(44100, 2.10);
        assert!(clip.warp_enabled);
        assert_eq!(clip.warp_markers.len(), 2);

        // Quantize to 0.25-beat grid at 100% strength
        clip.quantize_warp_markers(0.25, 1.0);
        assert!((clip.warp_markers[0].target_beat - 1.25).abs() < 1e-6);
        assert!((clip.warp_markers[1].target_beat - 2.0).abs() < 1e-6);

        // Move and remove markers
        assert!(clip.move_warp_marker(0, 1.0));
        assert!((clip.warp_markers[0].target_beat - 1.0).abs() < 1e-6);

        // Gain envelope interpolation
        clip.gain_envelope = vec![
            GainEnvelopePoint {
                beat_offset: 0.0,
                gain: 0.2,
            },
            GainEnvelopePoint {
                beat_offset: 2.0,
                gain: 1.0,
            },
        ];
        assert!((clip.gain_at_beat(5.0) - 0.6).abs() < 1e-4);

        // Slip edit content offset without shifting start_beat
        clip.slip_edit(512);
        assert_eq!(clip.offset_frames, 512);
        clip.slip_edit(-128);
        assert_eq!(clip.offset_frames, 384);
        assert_eq!(clip.start_beat, 4.0);

        // Slice at warp markers (1.0 and 2.0 -> 3 slices spanning beats 4..5, 5..6, 6..8)
        let slices = clip.slice_at_warp_markers(44100, 120.0);
        assert_eq!(slices.len(), 3);
        assert!((slices[0].start_beat - 4.0).abs() < 1e-6);
        assert!((slices[1].start_beat - 5.0).abs() < 1e-6);
        assert!((slices[2].start_beat - 6.0).abs() < 1e-6);
    }
}
