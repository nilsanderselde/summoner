// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Non-Destructive Multi-Take Audio Comping Engine with Sample-Accurate Splice Boundaries.

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

/// A recorded audio take with multichannel sample buffers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTake {
    pub id: u32,
    pub name: String,
    pub sample_rate: u32,
    pub start_frame: usize,
    pub channels: usize,
    pub samples_left: Vec<f32>,
    pub samples_right: Vec<f32>,
}

impl AudioTake {
    pub fn new_stereo(
        id: u32,
        name: &str,
        sample_rate: u32,
        samples_l: Vec<f32>,
        samples_r: Vec<f32>,
    ) -> Self {
        let channels = if samples_r.is_empty() { 1 } else { 2 };
        Self {
            id,
            name: name.to_string(),
            sample_rate,
            start_frame: 0,
            channels,
            samples_left: samples_l,
            samples_right: samples_r,
        }
    }

    pub fn num_frames(&self) -> usize {
        self.samples_left.len()
    }
}

/// A selected comp region spliced from a specific take into the master comp lane.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompRegion {
    pub id: u32,
    pub take_id: u32,
    pub source_start_frame: usize,
    pub source_end_frame: usize,
    pub timeline_start_frame: usize,
    pub crossfade_in_samples: usize,
    pub crossfade_out_samples: usize,
    pub gain: f32,
    pub muted: bool,
}

impl CompRegion {
    pub fn new(
        id: u32,
        take_id: u32,
        source_start: usize,
        source_end: usize,
        timeline_start: usize,
    ) -> Self {
        Self {
            id,
            take_id,
            source_start_frame: source_start,
            source_end_frame: source_end.max(source_start),
            timeline_start_frame: timeline_start,
            crossfade_in_samples: 128,
            crossfade_out_samples: 128,
            gain: 1.0,
            muted: false,
        }
    }

    pub fn duration_frames(&self) -> usize {
        self.source_end_frame
            .saturating_sub(self.source_start_frame)
    }

    pub fn timeline_end_frame(&self) -> usize {
        self.timeline_start_frame + self.duration_frames()
    }
}

/// A single comping lane holding an underlying take and UI state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompLane {
    pub id: u32,
    pub name: String,
    pub take: AudioTake,
    pub muted: bool,
    pub soloed: bool,
    pub color_rgba: [u8; 4],
}

impl CompLane {
    pub fn new(id: u32, name: &str, take: AudioTake) -> Self {
        Self {
            id,
            name: name.to_string(),
            take,
            muted: false,
            soloed: false,
            color_rgba: [80, 140, 220, 255],
        }
    }
}

/// Multi-Take Comping Manager for an audio track.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompTrack {
    pub id: u32,
    pub name: String,
    pub sample_rate: u32,
    pub lanes: Vec<CompLane>,
    pub comp_regions: Vec<CompRegion>,
    pub default_crossfade_samples: usize,
    next_take_id: u32,
    next_region_id: u32,
}

impl CompTrack {
    pub fn new(id: u32, name: &str, sample_rate: u32) -> Self {
        Self {
            id,
            name: name.to_string(),
            sample_rate,
            lanes: Vec::new(),
            comp_regions: Vec::new(),
            default_crossfade_samples: 128,
            next_take_id: 1,
            next_region_id: 1,
        }
    }

    /// Add a new audio take into a new lane.
    pub fn add_take(&mut self, name: &str, samples_l: Vec<f32>, samples_r: Vec<f32>) -> u32 {
        let take_id = self.next_take_id;
        self.next_take_id += 1;

        let take = AudioTake::new_stereo(take_id, name, self.sample_rate, samples_l, samples_r);
        let lane = CompLane::new(take_id, name, take);
        self.lanes.push(lane);
        take_id
    }

    /// Promote a slice from a take into the active comp track.
    pub fn promote_region(
        &mut self,
        take_id: u32,
        source_start: usize,
        source_end: usize,
        timeline_start: usize,
    ) -> u32 {
        let region_id = self.next_region_id;
        self.next_region_id += 1;

        let mut region =
            CompRegion::new(region_id, take_id, source_start, source_end, timeline_start);
        region.crossfade_in_samples = self.default_crossfade_samples;
        region.crossfade_out_samples = self.default_crossfade_samples;

        // Auto-splice / resolve overlaps with existing regions
        let new_end = region.timeline_end_frame();
        self.comp_regions.retain(|r| {
            let r_end = r.timeline_end_frame();
            // Remove regions completely covered by new region
            !(r.timeline_start_frame >= timeline_start && r_end <= new_end)
        });

        self.comp_regions.push(region);
        self.comp_regions.sort_by_key(|r| r.timeline_start_frame);
        region_id
    }

    /// Remove a comp region by ID.
    pub fn remove_region(&mut self, region_id: u32) -> bool {
        let before_len = self.comp_regions.len();
        self.comp_regions.retain(|r| r.id != region_id);
        self.comp_regions.len() < before_len
    }

    /// Split a comp region at a specified timeline frame.
    pub fn split_region(
        &mut self,
        region_id: u32,
        split_frame: usize,
    ) -> (Option<u32>, Option<u32>) {
        if let Some(pos) = self.comp_regions.iter().position(|r| r.id == region_id) {
            let reg = &self.comp_regions[pos];
            if split_frame <= reg.timeline_start_frame || split_frame >= reg.timeline_end_frame() {
                return (Some(region_id), None);
            }

            let offset = split_frame - reg.timeline_start_frame;
            let first_half = CompRegion {
                id: reg.id,
                take_id: reg.take_id,
                source_start_frame: reg.source_start_frame,
                source_end_frame: reg.source_start_frame + offset,
                timeline_start_frame: reg.timeline_start_frame,
                crossfade_in_samples: reg.crossfade_in_samples,
                crossfade_out_samples: self.default_crossfade_samples,
                gain: reg.gain,
                muted: reg.muted,
            };

            let second_id = self.next_region_id;
            self.next_region_id += 1;

            let second_half = CompRegion {
                id: second_id,
                take_id: reg.take_id,
                source_start_frame: reg.source_start_frame + offset,
                source_end_frame: reg.source_end_frame,
                timeline_start_frame: split_frame,
                crossfade_in_samples: self.default_crossfade_samples,
                crossfade_out_samples: reg.crossfade_out_samples,
                gain: reg.gain,
                muted: reg.muted,
            };

            self.comp_regions[pos] = first_half;
            self.comp_regions.insert(pos + 1, second_half);
            return (Some(region_id), Some(second_id));
        }

        (None, None)
    }

    /// Perform swipe-comping across a take lane for a given time range.
    /// Overlapping master comp regions are automatically trimmed, split, or replaced,
    /// and the new take slice is cleanly spliced in with crossfades.
    pub fn swipe_comp(
        &mut self,
        take_id: u32,
        timeline_start: usize,
        timeline_end: usize,
    ) -> Option<u32> {
        if timeline_end <= timeline_start {
            return None;
        }

        let take = self.get_take(take_id)?.clone();

        let source_start = timeline_start.saturating_sub(take.start_frame);
        let duration = timeline_end - timeline_start;
        let source_end = source_start + duration;

        // Process existing regions: trim, split, or discard
        let mut updated_regions = Vec::new();
        for r in &self.comp_regions {
            let r_start = r.timeline_start_frame;
            let r_end = r.timeline_end_frame();

            if r_end <= timeline_start || r_start >= timeline_end {
                // Completely outside swipe range
                updated_regions.push(r.clone());
            } else if r_start >= timeline_start && r_end <= timeline_end {
                // Completely covered by new swipe selection -> discard
                continue;
            } else if r_start < timeline_start && r_end > timeline_end {
                // Region spans across both start and end -> split into two
                let left_len = timeline_start - r_start;
                let mut left_piece = r.clone();
                left_piece.source_end_frame = r.source_start_frame + left_len;

                let right_offset = timeline_end - r_start;
                let mut right_piece = r.clone();
                right_piece.id = self.next_region_id;
                self.next_region_id += 1;
                right_piece.timeline_start_frame = timeline_end;
                right_piece.source_start_frame = r.source_start_frame + right_offset;

                updated_regions.push(left_piece);
                updated_regions.push(right_piece);
            } else if r_start < timeline_start && r_end > timeline_start {
                // Overlaps the beginning of the swipe range -> trim end
                let left_len = timeline_start - r_start;
                let mut left_piece = r.clone();
                left_piece.source_end_frame = r.source_start_frame + left_len;
                updated_regions.push(left_piece);
            } else if r_start < timeline_end && r_end > timeline_end {
                // Overlaps the end of the swipe range -> trim start
                let trim_offset = timeline_end - r_start;
                let mut right_piece = r.clone();
                right_piece.timeline_start_frame = timeline_end;
                right_piece.source_start_frame = r.source_start_frame + trim_offset;
                updated_regions.push(right_piece);
            }
        }

        let new_region_id = self.next_region_id;
        self.next_region_id += 1;

        let mut new_region = CompRegion::new(
            new_region_id,
            take_id,
            source_start,
            source_end,
            timeline_start,
        );
        new_region.crossfade_in_samples = self.default_crossfade_samples;
        new_region.crossfade_out_samples = self.default_crossfade_samples;

        updated_regions.push(new_region);
        updated_regions.sort_by_key(|r| r.timeline_start_frame);
        self.comp_regions = updated_regions;

        Some(new_region_id)
    }

    /// Flatten the current composite regions into a brand new take.
    pub fn flatten_to_take(&mut self, take_name: &str) -> Option<u32> {
        if self.comp_regions.is_empty() {
            return None;
        }

        let max_end = self
            .comp_regions
            .iter()
            .map(|r| r.timeline_end_frame())
            .max()
            .unwrap_or(0);
        if max_end == 0 {
            return None;
        }

        let (samples_l, samples_r) = self.render_comp_stereo(max_end);
        let take_id = self.add_take(take_name, samples_l, samples_r);
        Some(take_id)
    }

    /// Find an audio take by ID.
    pub fn get_take(&self, take_id: u32) -> Option<&AudioTake> {
        self.lanes
            .iter()
            .find(|l| l.take.id == take_id)
            .map(|l| &l.take)
    }

    /// Render the composite audio track to stereo buffers with sample-accurate equal-power crossfading.
    pub fn render_comp_stereo(&self, total_frames: usize) -> (Vec<f32>, Vec<f32>) {
        let mut out_l = vec![0.0f32; total_frames];
        let mut out_r = vec![0.0f32; total_frames];

        for region in &self.comp_regions {
            if region.muted {
                continue;
            }

            let take = match self.get_take(region.take_id) {
                Some(t) => t,
                None => continue,
            };

            let dur = region.duration_frames();
            let fade_in = region.crossfade_in_samples.min(dur / 2);
            let fade_out = region.crossfade_out_samples.min(dur / 2);

            for i in 0..dur {
                let timeline_idx = region.timeline_start_frame + i;
                if timeline_idx >= total_frames {
                    break;
                }

                let src_idx = region.source_start_frame + i;
                let sample_l = if src_idx < take.samples_left.len() {
                    take.samples_left[src_idx]
                } else {
                    0.0
                };
                let sample_r =
                    if !take.samples_right.is_empty() && src_idx < take.samples_right.len() {
                        take.samples_right[src_idx]
                    } else {
                        sample_l
                    };

                // Equal-power crossfade curves (quarter-sine / cosine)
                let mut env = 1.0f32;
                if fade_in > 0 && i < fade_in {
                    let t = i as f32 / fade_in as f32;
                    env *= (t * (PI * 0.5)).sin();
                }
                if fade_out > 0 && i >= dur.saturating_sub(fade_out) {
                    let t = (dur - 1 - i) as f32 / fade_out as f32;
                    env *= (t * (PI * 0.5)).sin();
                }

                out_l[timeline_idx] += sample_l * env * region.gain;
                out_r[timeline_idx] += sample_r * env * region.gain;
            }
        }

        // Clamp to [-1.0, 1.0]
        for s in out_l.iter_mut() {
            *s = s.clamp(-1.0, 1.0);
        }
        for s in out_r.iter_mut() {
            *s = s.clamp(-1.0, 1.0);
        }

        (out_l, out_r)
    }

    /// Render the composite audio track to a mono buffer.
    pub fn render_comp_mono(&self, total_frames: usize) -> Vec<f32> {
        let (l, r) = self.render_comp_stereo(total_frames);
        let mut mono = vec![0.0f32; total_frames];
        for i in 0..total_frames {
            mono[i] = (l[i] + r[i]) * 0.5;
        }
        mono
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comp_track_multi_take_promotion_and_crossfade_render() {
        let mut comp = CompTrack::new(1, "Vocal Track", 44100);
        let take1_l: Vec<f32> = vec![0.5; 1000];
        let take1_r: Vec<f32> = vec![0.5; 1000];
        let t1 = comp.add_take("Take 1", take1_l, take1_r);

        let take2_l: Vec<f32> = vec![0.8; 1000];
        let take2_r: Vec<f32> = vec![0.8; 1000];
        let t2 = comp.add_take("Take 2", take2_l, take2_r);

        assert_eq!(comp.lanes.len(), 2);

        // Promote slice 0..400 from Take 1 to timeline 0..400
        let r1 = comp.promote_region(t1, 0, 400, 0);
        // Promote slice 400..1000 from Take 2 to timeline 400..1000
        let r2 = comp.promote_region(t2, 400, 1000, 400);

        assert_eq!(comp.comp_regions.len(), 2);
        assert_eq!(r1, 1);
        assert_eq!(r2, 2);

        let (out_l, out_r) = comp.render_comp_stereo(1000);
        assert_eq!(out_l.len(), 1000);
        assert_eq!(out_r.len(), 1000);

        // Verify start has take 1 signal and end has take 2 signal
        assert!(out_l[200] > 0.4 && out_l[200] < 0.6);
        assert!(out_l[800] > 0.7 && out_l[800] < 0.9);
    }

    #[test]
    fn test_comp_track_region_split_and_remove() {
        let mut comp = CompTrack::new(1, "Guitar", 44100);
        let take_l = vec![0.3f32; 1000];
        let t = comp.add_take("Take 1", take_l.clone(), take_l);

        let r = comp.promote_region(t, 0, 800, 100);
        let (first, second) = comp.split_region(r, 500);

        assert_eq!(first, Some(r));
        assert!(second.is_some());
        assert_eq!(comp.comp_regions.len(), 2);

        assert_eq!(comp.comp_regions[0].timeline_start_frame, 100);
        assert_eq!(comp.comp_regions[0].timeline_end_frame(), 500);
        assert_eq!(comp.comp_regions[1].timeline_start_frame, 500);
        assert_eq!(comp.comp_regions[1].timeline_end_frame(), 900);

        let removed = comp.remove_region(second.unwrap());
        assert!(removed);
        assert_eq!(comp.comp_regions.len(), 1);
    }

    #[test]
    fn test_swipe_comping_and_flatten_to_take() {
        let mut comp = CompTrack::new(1, "Lead Vocal", 44100);
        let take1_l = vec![0.2f32; 10_000];
        let take2_l = vec![0.8f32; 10_000];
        let t1 = comp.add_take("Take 1", take1_l.clone(), take1_l);
        let t2 = comp.add_take("Take 2", take2_l.clone(), take2_l);

        // Initial swipe: Take 1 over whole 0..8000
        let r1 = comp.swipe_comp(t1, 0, 8000).unwrap();
        assert_eq!(comp.comp_regions.len(), 1);
        assert_eq!(comp.comp_regions[0].id, r1);

        // Swipe Take 2 in the middle: 3000..5000 (should split Take 1 into [0..3000] and [5000..8000])
        let r2 = comp.swipe_comp(t2, 3000, 5000).unwrap();
        assert_eq!(comp.comp_regions.len(), 3);

        assert_eq!(comp.comp_regions[0].take_id, t1);
        assert_eq!(comp.comp_regions[0].timeline_start_frame, 0);
        assert_eq!(comp.comp_regions[0].timeline_end_frame(), 3000);

        assert_eq!(comp.comp_regions[1].take_id, t2);
        assert_eq!(comp.comp_regions[1].id, r2);
        assert_eq!(comp.comp_regions[1].timeline_start_frame, 3000);
        assert_eq!(comp.comp_regions[1].timeline_end_frame(), 5000);

        assert_eq!(comp.comp_regions[2].take_id, t1);
        assert_eq!(comp.comp_regions[2].timeline_start_frame, 5000);
        assert_eq!(comp.comp_regions[2].timeline_end_frame(), 8000);

        // Flatten to new take
        let final_take_id = comp.flatten_to_take("Comped Lead Vocal Master").unwrap();
        assert_eq!(comp.lanes.len(), 3);
        let master_take = comp.get_take(final_take_id).unwrap();
        assert_eq!(master_take.name, "Comped Lead Vocal Master");
        assert_eq!(master_take.num_frames(), 8000);

        // Check values in middle of take 1 and take 2 sections
        assert!((master_take.samples_left[1500] - 0.2).abs() < 1e-4);
        assert!((master_take.samples_left[4000] - 0.8).abs() < 1e-4);
        assert!((master_take.samples_left[6500] - 0.2).abs() < 1e-4);
    }
}
