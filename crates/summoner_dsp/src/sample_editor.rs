// Summoner DAW - Sample Editor and Processing Tools (Steps 696-700)
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

use crate::sampler::SampleBuffer;
use crate::slicer::AutoSlicer;
use serde::{Deserialize, Serialize};

/// Sample Marker descriptor for marking region boundaries (Step 699).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SampleMarker {
    pub position: usize,
    pub label: String,
}

/// Sample Editor structure managing markers, loops, and destructive editing operations (Steps 697-699).
#[derive(Debug, Clone, Default)]
pub struct SampleEditor {
    pub markers: Vec<SampleMarker>,
}

impl SampleEditor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_marker(&mut self, position: usize, label: &str) {
        self.markers.push(SampleMarker {
            position,
            label: label.to_string(),
        });
        self.markers.sort_by_key(|m| m.position);
    }

    pub fn move_marker(&mut self, index: usize, new_position: usize) -> bool {
        if index < self.markers.len() {
            self.markers[index].position = new_position;
            self.markers.sort_by_key(|m| m.position);
            true
        } else {
            false
        }
    }

    pub fn remove_marker(&mut self, index: usize) -> bool {
        if index < self.markers.len() {
            self.markers.remove(index);
            true
        } else {
            false
        }
    }
}

/// Audition sample pitch-adjusted to target MIDI note (default C4 = 60) (Step 696).
pub fn audition_sample_at_c4(sample_buffer: &SampleBuffer, target_note: u8) -> SampleBuffer {
    let root_key = 60u8;
    let ratio = 2.0f32.powf((target_note as f32 - root_key as f32) / 12.0);

    let orig_len = sample_buffer.data.len();
    let new_len = (orig_len as f32 / ratio).round() as usize;
    if new_len == 0 {
        return sample_buffer.clone();
    }

    let mut resampled = Vec::with_capacity(new_len);
    for i in 0..new_len {
        let read_pos = i as f32 * ratio;
        let idx0 = read_pos.floor() as usize;
        let frac = read_pos - idx0 as f32;
        let s0 = if idx0 < orig_len { sample_buffer.data[idx0] } else { 0.0 };
        let s1 = if idx0 + 1 < orig_len { sample_buffer.data[idx0 + 1] } else { 0.0 };
        resampled.push(s0 + frac * (s1 - s0));
    }

    SampleBuffer {
        data: resampled,
        sample_rate: sample_buffer.sample_rate,
        channels: sample_buffer.channels,
    }
}

/// Destructive sample editing tools: trim, normalize, reverse, fade in/out, remove DC offset (Step 697).
pub fn trim_sample(buffer: &mut Vec<f32>, start_sample: usize, end_sample: usize) {
    let start = start_sample.min(buffer.len());
    let end = end_sample.clamp(start, buffer.len());
    *buffer = buffer[start..end].to_vec();
}

pub fn normalize_sample(buffer: &mut [f32], target_peak_db: f32) {
    if buffer.is_empty() { return; }
    let max_peak = buffer.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    if max_peak < 1e-6 { return; }
    let target_linear = 10.0f32.powf(target_peak_db / 20.0);
    let scale = target_linear / max_peak;
    for s in buffer.iter_mut() {
        *s *= scale;
    }
}

pub fn reverse_sample(buffer: &mut [f32]) {
    buffer.reverse();
}

pub fn fade_in_sample(buffer: &mut [f32], fade_len: usize) {
    let len = fade_len.min(buffer.len());
    if len == 0 { return; }
    for i in 0..len {
        let t = i as f32 / len as f32;
        buffer[i] *= t;
    }
}

pub fn fade_out_sample(buffer: &mut [f32], fade_len: usize) {
    let len = fade_len.min(buffer.len());
    let total = buffer.len();
    if len == 0 || total == 0 { return; }
    for i in 0..len {
        let idx = total - len + i;
        let t = 1.0 - (i as f32 / len as f32);
        buffer[idx] *= t;
    }
}

pub fn remove_dc_offset_sample(buffer: &mut [f32]) {
    if buffer.is_empty() { return; }
    let mean: f32 = buffer.iter().sum::<f32>() / buffer.len() as f32;
    for s in buffer.iter_mut() {
        *s -= mean;
    }
}

/// Sample loop editor with crossfade loop around loop boundary (Step 698).
pub fn crossfade_sample_loop(
    buffer: &mut Vec<f32>,
    loop_start: usize,
    loop_end: usize,
    crossfade_len: usize,
) {
    let total = buffer.len();
    if loop_start >= loop_end || loop_end > total || crossfade_len == 0 {
        return;
    }
    let xfade = crossfade_len.min(loop_start).min(loop_end - loop_start);
    if xfade == 0 { return; }

    for i in 0..xfade {
        let alpha = i as f32 / xfade as f32;
        let post_loop_idx = loop_end - xfade + i;
        let pre_loop_idx = loop_start - xfade + i;
        if post_loop_idx < total && pre_loop_idx < total {
            let blended = (1.0 - alpha) * buffer[post_loop_idx] + alpha * buffer[pre_loop_idx];
            buffer[post_loop_idx] = blended;
        }
    }
}

/// Chop sample to pads (up to 16 region start/end sample ranges) (Step 700).
pub fn chop_sample_to_pads(buffer: &[f32], sample_rate: u32, max_pads: usize) -> Vec<(usize, usize)> {
    if buffer.is_empty() {
        return Vec::new();
    }
    let sample_buf = SampleBuffer::new(buffer.to_vec(), sample_rate, 1);
    let slicer = AutoSlicer::new(0.12, crate::slicer::SliceAlgorithm::EnergyDerivative);
    let markers = slicer.detect_slices(&sample_buf);
    
    let pad_count = max_pads.min(16).max(1);
    let mut regions = Vec::new();
    for marker in markers.iter().take(pad_count) {
        if marker.start_sample < marker.end_sample {
            regions.push((marker.start_sample, marker.end_sample));
        }
    }
    
    if regions.is_empty() {
        let chunk_size = buffer.len() / pad_count;
        for p in 0..pad_count {
            let start = p * chunk_size;
            let end = if p == pad_count - 1 { buffer.len() } else { (p + 1) * chunk_size };
            regions.push((start, end));
        }
    }
    
    regions
}
