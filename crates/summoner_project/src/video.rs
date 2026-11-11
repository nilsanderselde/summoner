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

//! Video Import, Picture Sync, and Post-Production Timeline Synchronization (Task I1).
//!
//! Provides Pro Tools / Nuendo post-production parity for video workflows:
//! - [`VideoTrackConfig`] & [`VideoClipConfig`]: Project session persistence of video tracks and clips.
//! - [`VideoThumbnail`] & [`ThumbnailFilmstripCache`]: Fast filmstrip generation and viewport caching.
//! - [`VideoSyncEngine`]: Frame-accurate timeline scrubbing, SMPTE ruler alignment, and audio/video drift detection.
//! - [`VideoAspectRatio`]: Common broadcast and cinematic display aspect ratios.

use crate::audio_clip::AudioClipConfig;
use crate::schema::TrackConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use summoner_core::timecode::{SmpteTimecode, TimecodeRate};

/// Helper to convert timeline seconds to `SmpteTimecode`.
pub fn timecode_from_seconds(seconds: f64, rate: TimecodeRate) -> SmpteTimecode {
    let total_frames = (seconds * rate.frames_per_second()).round().max(0.0) as u64;
    SmpteTimecode::from_total_frames(total_frames, rate)
}

/// Display aspect ratio for video playback preview.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum VideoAspectRatio {
    /// 16:9 Standard widescreen (1.78:1)
    Widescreen16x9,
    /// 4:3 Standard definition (1.33:1)
    Standard4x3,
    /// 2.39:1 CinemaScope / Anamorphic
    CinemaScope239,
    /// 1.85:1 US Cinema standard flat
    Flat185,
    /// 9:16 Vertical social / mobile video
    Vertical9x16,
    /// 1:1 Square
    Square1x1,
}

impl VideoAspectRatio {
    pub fn ratio_f64(self) -> f64 {
        match self {
            Self::Widescreen16x9 => 16.0 / 9.0,
            Self::Standard4x3 => 4.0 / 3.0,
            Self::CinemaScope239 => 2.39,
            Self::Flat185 => 1.85,
            Self::Vertical9x16 => 9.0 / 16.0,
            Self::Square1x1 => 1.0,
        }
    }
}

/// Video clip placed on the arrangement timeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoClipConfig {
    pub id: String,
    pub name: String,
    pub source_path: String,
    /// Start position on the DAW timeline in seconds.
    pub timeline_start_seconds: f64,
    /// Duration of the clip on the DAW timeline in seconds.
    pub duration_seconds: f64,
    /// Offset into the media file in seconds.
    pub source_offset_seconds: f64,
    /// Total number of video frames in media file.
    pub frame_count: u64,
    pub frame_width: u32,
    pub frame_height: u32,
    pub frame_rate: TimecodeRate,
    /// Number of embedded audio channels (e.g. 2 for stereo, 6 for 5.1).
    pub embedded_audio_channels: u32,
    /// Embedded audio sample rate (e.g. 48000 Hz broadcast standard).
    pub embedded_sample_rate: u32,
    /// Muted state for video display.
    pub muted: bool,
}

impl VideoClipConfig {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        source_path: impl Into<String>,
        frame_rate: TimecodeRate,
        frame_width: u32,
        frame_height: u32,
        duration_seconds: f64,
    ) -> Self {
        let fps = frame_rate.frames_per_second();
        let frame_count = (duration_seconds * fps).round() as u64;
        Self {
            id: id.into(),
            name: name.into(),
            source_path: source_path.into(),
            timeline_start_seconds: 0.0,
            duration_seconds,
            source_offset_seconds: 0.0,
            frame_count,
            frame_width,
            frame_height,
            frame_rate,
            embedded_audio_channels: 2,
            embedded_sample_rate: 48000,
            muted: false,
        }
    }

    /// End position on timeline in seconds.
    pub fn timeline_end_seconds(&self) -> f64 {
        self.timeline_start_seconds + self.duration_seconds
    }

    /// Convert timeline seconds to exact frame index within this clip's media.
    pub fn timeline_sec_to_source_frame(&self, timeline_sec: f64) -> Option<u64> {
        if timeline_sec < self.timeline_start_seconds || timeline_sec > self.timeline_end_seconds()
        {
            return None;
        }
        let clip_relative_sec =
            timeline_sec - self.timeline_start_seconds + self.source_offset_seconds;
        let fps = self.frame_rate.frames_per_second();
        let frame_idx = (clip_relative_sec * fps).floor() as u64;
        Some(frame_idx.min(self.frame_count.saturating_sub(1)))
    }
}

/// Video track holding video clips and post-production sync parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoTrackConfig {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub aspect_ratio: VideoAspectRatio,
    pub frame_rate: TimecodeRate,
    /// Timeline reference SMPTE timecode (standard 01:00:00:00 hour 1 start).
    pub start_timecode: SmpteTimecode,
    pub clips: Vec<VideoClipConfig>,
    /// Linked audio track ID containing the extracted dialogue/SFX audio.
    pub linked_audio_track_id: Option<u64>,
}

impl Default for VideoTrackConfig {
    fn default() -> Self {
        Self {
            id: "video_track_0".to_string(),
            name: "Video".to_string(),
            enabled: true,
            aspect_ratio: VideoAspectRatio::Widescreen16x9,
            frame_rate: TimecodeRate::Fps24,
            start_timecode: SmpteTimecode::new(1, 0, 0, 0, TimecodeRate::Fps24),
            clips: Vec::new(),
            linked_audio_track_id: None,
        }
    }
}

impl VideoTrackConfig {
    /// Generates a linked `TrackConfig` and `AudioClipConfig` for extracted audio.
    pub fn generate_linked_audio_track(
        &self,
        track_id: u64,
    ) -> (TrackConfig, Vec<AudioClipConfig>) {
        let mut audio_clips = Vec::new();
        for clip in &self.clips {
            let mut audio_clip = AudioClipConfig::new(
                format!("{}_audio", clip.id),
                format!("{}.audio", clip.name),
                clip.id.clone(),
                0.0,
                clip.duration_seconds * 2.0, // 2 beats/sec at 120 bpm default
            );
            audio_clip.file_path = Some(clip.source_path.clone());
            audio_clip.color = Some([33, 150, 243]);
            audio_clips.push(audio_clip);
        }

        let track = TrackConfig {
            id: track_id,
            name: format!("{} (Audio)", self.name),
            channels: 2,
            audio_clips: audio_clips.clone(),
            color: Some([63, 81, 181]),
            ..Default::default()
        };

        (track, audio_clips)
    }
}

/// Single low-resolution video thumbnail for filmstrip rendering.
#[derive(Debug, Clone, PartialEq)]
pub struct VideoThumbnail {
    pub frame_index: u64,
    pub timestamp_seconds: f64,
    pub timecode: SmpteTimecode,
    pub width: u32,
    pub height: u32,
    /// RGBA pixels (width * height * 4 bytes).
    pub rgba_data: Vec<u8>,
}

impl VideoThumbnail {
    /// Create a thumbnail frame with a given resolution.
    pub fn new(
        frame_index: u64,
        timestamp_seconds: f64,
        timecode: SmpteTimecode,
        width: u32,
        height: u32,
        rgba_data: Vec<u8>,
    ) -> Self {
        Self {
            frame_index,
            timestamp_seconds,
            timecode,
            width,
            height,
            rgba_data,
        }
    }

    /// Generate a solid color placeholder thumbnail (useful for fast testing or missing frames).
    pub fn placeholder(
        frame_index: u64,
        timestamp_seconds: f64,
        timecode: SmpteTimecode,
        width: u32,
        height: u32,
        rgb: [u8; 3],
    ) -> Self {
        let count = (width * height * 4) as usize;
        let mut rgba_data = Vec::with_capacity(count);
        for _ in 0..(width * height) {
            rgba_data.push(rgb[0]);
            rgba_data.push(rgb[1]);
            rgba_data.push(rgb[2]);
            rgba_data.push(255);
        }
        Self::new(
            frame_index,
            timestamp_seconds,
            timecode,
            width,
            height,
            rgba_data,
        )
    }
}

/// Thumbnail cache for video filmstrips on the arranger timeline.
#[derive(Debug, Clone, Default)]
pub struct ThumbnailFilmstripCache {
    /// Keyed by (clip_id, frame_index).
    thumbnails: HashMap<(String, u64), VideoThumbnail>,
    max_cached_entries: usize,
}

impl ThumbnailFilmstripCache {
    pub fn new(max_cached_entries: usize) -> Self {
        Self {
            thumbnails: HashMap::new(),
            max_cached_entries: max_cached_entries.max(32),
        }
    }

    pub fn insert(&mut self, clip_id: impl Into<String>, thumbnail: VideoThumbnail) {
        if self.thumbnails.len() >= self.max_cached_entries {
            // Drop an entry if at capacity
            if let Some(first_key) = self.thumbnails.keys().next().cloned() {
                self.thumbnails.remove(&first_key);
            }
        }
        self.thumbnails
            .insert((clip_id.into(), thumbnail.frame_index), thumbnail);
    }

    pub fn get(&self, clip_id: &str, frame_index: u64) -> Option<&VideoThumbnail> {
        self.thumbnails.get(&(clip_id.to_string(), frame_index))
    }

    pub fn clear(&mut self) {
        self.thumbnails.clear();
    }

    pub fn len(&self) -> usize {
        self.thumbnails.len()
    }

    pub fn is_empty(&self) -> bool {
        self.thumbnails.is_empty()
    }

    /// Retrieve or sample thumbnails for a timeline time range.
    pub fn get_filmstrip_for_range(
        &self,
        clip: &VideoClipConfig,
        start_sec: f64,
        end_sec: f64,
        interval_sec: f64,
    ) -> Vec<VideoThumbnail> {
        let mut strip = Vec::new();
        let interval = interval_sec.max(0.1);
        let mut t = start_sec.max(clip.timeline_start_seconds);
        let limit = end_sec.min(clip.timeline_end_seconds());

        while t <= limit {
            if let Some(frame_idx) = clip.timeline_sec_to_source_frame(t) {
                if let Some(thumb) = self.get(&clip.id, frame_idx) {
                    strip.push(thumb.clone());
                } else {
                    // Generate placeholder
                    let tc = timecode_from_seconds(t, clip.frame_rate);
                    let placeholder =
                        VideoThumbnail::placeholder(frame_idx, t, tc, 80, 45, [40, 40, 50]);
                    strip.push(placeholder);
                }
            }
            t += interval;
        }

        strip
    }
}

/// Scrubbing state returned when seeking or scrubbing across video.
#[derive(Debug, Clone, PartialEq)]
pub struct VideoScrubState {
    pub timeline_seconds: f64,
    pub timecode: SmpteTimecode,
    pub active_clip_id: Option<String>,
    pub frame_index: u64,
    /// Sub-frame interpolation factor between current and next frame [0.0..1.0].
    pub sub_frame_fraction: f64,
}

/// Sync lock status between audio playback clock and video presentation.
#[derive(Debug, Clone, PartialEq)]
pub enum VideoDriftStatus {
    /// Audio and video clocks are locked within tolerance (< 0.5 frame).
    InSync { drift_ms: f64 },
    /// Minor clock drift detected (0.5 to 2.0 frames).
    Slipping { drift_ms: f64, slip_frames: f64 },
    /// Excessive drift requiring hard resync (> 2.0 frames).
    DriftExceeded { drift_ms: f64, dropped_frames: i64 },
}

/// SMPTE Timecode ruler tick mark.
#[derive(Debug, Clone, PartialEq)]
pub struct TimecodeRulerMark {
    pub time_seconds: f64,
    pub timecode: SmpteTimecode,
    pub is_major: bool,
    pub label: String,
}

/// Video timeline synchronization and frame-accurate scrubbing engine.
#[derive(Debug, Clone)]
pub struct VideoSyncEngine {
    track: VideoTrackConfig,
    filmstrip_cache: ThumbnailFilmstripCache,
    shuttle_speed: f64,
}

impl VideoSyncEngine {
    pub fn new(track: VideoTrackConfig) -> Self {
        Self {
            track,
            filmstrip_cache: ThumbnailFilmstripCache::new(512),
            shuttle_speed: 1.0,
        }
    }

    pub fn track(&self) -> &VideoTrackConfig {
        &self.track
    }

    pub fn track_mut(&mut self) -> &mut VideoTrackConfig {
        &mut self.track
    }

    pub fn filmstrip_cache_mut(&mut self) -> &mut ThumbnailFilmstripCache {
        &mut self.filmstrip_cache
    }

    /// Set shuttle speed (-16.0x to +16.0x for jog/shuttle wheel).
    pub fn set_shuttle_speed(&mut self, speed: f64) {
        self.shuttle_speed = speed.clamp(-16.0, 16.0);
    }

    pub fn shuttle_speed(&self) -> f64 {
        self.shuttle_speed
    }

    /// Frame-accurate scrub to specific timeline timestamp.
    pub fn scrub_to(&self, timeline_seconds: f64) -> VideoScrubState {
        let fps = self.track.frame_rate.frames_per_second();
        let total_frames_float = timeline_seconds * fps;
        let frame_index = total_frames_float.floor().max(0.0) as u64;
        let sub_frame_fraction = total_frames_float - (frame_index as f64);

        let timecode = timecode_from_seconds(timeline_seconds, self.track.frame_rate);

        let mut active_clip_id = None;
        for clip in &self.track.clips {
            if timeline_seconds >= clip.timeline_start_seconds
                && timeline_seconds <= clip.timeline_end_seconds()
            {
                active_clip_id = Some(clip.id.clone());
                break;
            }
        }

        VideoScrubState {
            timeline_seconds,
            timecode,
            active_clip_id,
            frame_index,
            sub_frame_fraction,
        }
    }

    /// Check audio-to-video clock drift given audio sample position and displayed video frame timestamp.
    pub fn check_clock_drift(
        &self,
        audio_samples: u64,
        sample_rate: u32,
        video_displayed_seconds: f64,
    ) -> VideoDriftStatus {
        let audio_seconds = audio_samples as f64 / (sample_rate as f64);
        let drift_seconds = video_displayed_seconds - audio_seconds;
        let drift_ms = drift_seconds * 1000.0;

        let fps = self.track.frame_rate.frames_per_second();
        let frame_duration_ms = 1000.0 / fps;
        let slip_frames = drift_ms / frame_duration_ms;

        if slip_frames.abs() <= 0.5 {
            VideoDriftStatus::InSync { drift_ms }
        } else if slip_frames.abs() <= 2.0 {
            VideoDriftStatus::Slipping {
                drift_ms,
                slip_frames,
            }
        } else {
            VideoDriftStatus::DriftExceeded {
                drift_ms,
                dropped_frames: slip_frames.round() as i64,
            }
        }
    }

    /// Generate SMPTE timecode ruler markers for timeline display.
    pub fn generate_timecode_ruler(
        &self,
        start_seconds: f64,
        end_seconds: f64,
        step_seconds: f64,
    ) -> Vec<TimecodeRulerMark> {
        let mut marks = Vec::new();
        let step = step_seconds.max(1.0 / self.track.frame_rate.frames_per_second());
        let mut t = start_seconds;

        while t <= end_seconds {
            let tc = timecode_from_seconds(t, self.track.frame_rate);
            let is_major = (t.round() - t).abs() < 1e-4 && (t as u64).is_multiple_of(5);
            let label = tc.to_string();

            marks.push(TimecodeRulerMark {
                time_seconds: t,
                timecode: tc,
                is_major,
                label,
            });

            t += step;
        }

        marks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_aspect_ratios() {
        assert!((VideoAspectRatio::Widescreen16x9.ratio_f64() - 1.7777).abs() < 0.001);
        assert!((VideoAspectRatio::Standard4x3.ratio_f64() - 1.3333).abs() < 0.001);
        assert_eq!(VideoAspectRatio::Square1x1.ratio_f64(), 1.0);
    }

    #[test]
    fn test_video_clip_frame_mapping() {
        let clip = VideoClipConfig::new(
            "clip1",
            "Scene 1",
            "media/scene1.mp4",
            TimecodeRate::Fps24,
            1920,
            1080,
            10.0,
        );

        assert_eq!(clip.frame_count, 240);
        assert_eq!(clip.timeline_sec_to_source_frame(0.0), Some(0));
        assert_eq!(clip.timeline_sec_to_source_frame(1.0), Some(24));
        assert_eq!(clip.timeline_sec_to_source_frame(5.5), Some(132));
        assert_eq!(clip.timeline_sec_to_source_frame(10.0), Some(239));
        assert_eq!(clip.timeline_sec_to_source_frame(11.0), None);
    }

    #[test]
    fn test_frame_accurate_scrubbing() {
        let track = VideoTrackConfig::default();
        let engine = VideoSyncEngine::new(track);

        let scrub = engine.scrub_to(1.5);
        assert_eq!(scrub.frame_index, 36); // 1.5 * 24 fps = 36
        assert!((scrub.sub_frame_fraction - 0.0).abs() < 1e-5);
        assert_eq!(scrub.timecode.seconds, 1);
        assert_eq!(scrub.timecode.frames, 12);
    }

    #[test]
    fn test_filmstrip_cache_and_placeholders() {
        let mut cache = ThumbnailFilmstripCache::new(4);
        let tc = SmpteTimecode::new(1, 0, 0, 0, TimecodeRate::Fps24);
        let thumb = VideoThumbnail::placeholder(0, 0.0, tc, 80, 45, [255, 0, 0]);

        cache.insert("clip1", thumb);
        assert_eq!(cache.len(), 1);
        assert!(cache.get("clip1", 0).is_some());
        assert_eq!(cache.get("clip1", 0).unwrap().rgba_data[0], 255);
    }

    #[test]
    fn test_audio_video_drift_detection() {
        let track = VideoTrackConfig::default();
        let engine = VideoSyncEngine::new(track);

        // Audio at 48000 samples = 1.0s, video at 1.0s -> InSync
        let sync_status = engine.check_clock_drift(48000, 48000, 1.0);
        match sync_status {
            VideoDriftStatus::InSync { drift_ms } => assert!(drift_ms.abs() < 1e-4),
            _ => panic!("Expected InSync"),
        }

        // Drift of 50ms at 24fps (frame duration ~41.6ms) -> ~1.2 frames slip
        let slip_status = engine.check_clock_drift(48000, 48000, 1.050);
        match slip_status {
            VideoDriftStatus::Slipping {
                drift_ms,
                slip_frames,
            } => {
                assert!((drift_ms - 50.0).abs() < 1e-3);
                assert!((slip_frames - 1.2).abs() < 0.01);
            }
            _ => panic!("Expected Slipping"),
        }

        // Drift of 200ms -> >4 frames dropped
        let drift_status = engine.check_clock_drift(48000, 48000, 1.200);
        match drift_status {
            VideoDriftStatus::DriftExceeded { dropped_frames, .. } => {
                assert_eq!(dropped_frames, 5);
            }
            _ => panic!("Expected DriftExceeded"),
        }
    }

    #[test]
    fn test_linked_audio_track_generation() {
        let mut track = VideoTrackConfig::default();
        track.clips.push(VideoClipConfig::new(
            "clip_v1",
            "Interview",
            "video/interview.mov",
            TimecodeRate::Fps24,
            3840,
            2160,
            60.0,
        ));

        let (audio_track, audio_clips) = track.generate_linked_audio_track(42);
        assert_eq!(audio_track.id, 42);
        assert_eq!(audio_clips.len(), 1);
        assert_eq!(
            audio_clips[0].file_path,
            Some("video/interview.mov".to_string())
        );
        assert_eq!(audio_clips[0].duration_beats, 120.0);
    }
}
