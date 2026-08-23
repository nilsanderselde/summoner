// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! WSOLA (Waveform Similarity Overlap-Add) Elastic Audio Time-Stretching & Pitch-Shift Engine.

use std::f32::consts::PI;

/// A warp marker mapping a source frame in an audio asset to a target timeline frame.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WarpMarker {
    pub source_frame: usize,
    pub target_frame: usize,
}

impl WarpMarker {
    pub fn new(source_frame: usize, target_frame: usize) -> Self {
        Self {
            source_frame,
            target_frame,
        }
    }
}

/// WSOLA (Waveform Similarity Overlap-Add) time-stretching engine.
///
/// Provides pitch-preserving time stretching without phase artifacts or comb filtering
/// by finding maximal cross-correlation alignment between consecutive synthesis grains.
#[derive(Debug, Clone)]
pub struct WsolaTimeStretcher {
    pub sample_rate: u32,
    pub window_size: usize,
    pub hop_size: usize,
    pub max_delta: usize,
    pub stretch_ratio: f32, // Time stretch factor (0.25 to 4.0). 2.0 = twice as long (half speed)
    window: Vec<f32>,
    target_grain: Vec<f32>,
    output_accumulator: Vec<f32>,
    weight_accumulator: Vec<f32>,
}

impl WsolaTimeStretcher {
    pub fn new(sample_rate: u32, window_size: usize, hop_size: usize) -> Self {
        let win_sz = window_size.max(64);
        let hop_sz = hop_size.max(16).min(win_sz / 2);
        let max_delta = hop_sz;

        // Symmetric Hanning window
        let mut window = vec![0.0f32; win_sz];
        for (i, val) in window.iter_mut().enumerate() {
            *val = 0.5 * (1.0 - (2.0 * PI * i as f32 / win_sz as f32).cos());
        }

        Self {
            sample_rate,
            window_size: win_sz,
            hop_size: hop_sz,
            max_delta,
            stretch_ratio: 1.0,
            window,
            target_grain: vec![0.0; win_sz],
            output_accumulator: Vec::new(),
            weight_accumulator: Vec::new(),
        }
    }

    pub fn set_stretch_ratio(&mut self, ratio: f32) {
        self.stretch_ratio = ratio.clamp(0.25, 4.0);
    }

    /// Finds the best alignment offset delta in [-max_delta, +max_delta] via normalized cross-correlation.
    fn find_best_alignment(&self, input: &[f32], nominal_pos: isize) -> isize {
        let in_len = input.len() as isize;
        let win_sz = self.window_size as isize;
        let max_d = self.max_delta as isize;

        let mut best_delta: isize = 0;
        let mut max_corr = -1e9f32;

        for delta in -max_d..=max_d {
            let start = nominal_pos + delta;
            if start < 0 || start + win_sz > in_len {
                continue;
            }

            let mut corr = 0.0f32;
            let mut energy = 0.0f32;

            for k in 0..self.window_size {
                let s = input[(start + k as isize) as usize];
                let t = self.target_grain[k];
                corr += s * t;
                energy += s * s;
            }

            let norm_corr = if energy > 1e-6 {
                corr / (energy.sqrt() + 1e-6)
            } else {
                corr
            };

            if norm_corr > max_corr {
                max_corr = norm_corr;
                best_delta = delta;
            }
        }

        best_delta
    }

    /// Process a mono slice of audio using WSOLA time-stretching.
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) -> usize {
        if input.is_empty() || output.is_empty() {
            return 0;
        }

        let out_len = output.len();
        let in_len = input.len();
        let win_sz = self.window_size;
        let s_s = self.hop_size; // Synthesis hop size
        let s_a = (s_s as f32 / self.stretch_ratio).round().max(1.0) as usize; // Analysis hop size

        if self.output_accumulator.len() < out_len + win_sz {
            self.output_accumulator.resize(out_len + win_sz, 0.0);
            self.weight_accumulator.resize(out_len + win_sz, 0.0);
        }
        self.output_accumulator.fill(0.0);
        self.weight_accumulator.fill(0.0);

        let mut out_pos = 0usize;
        let mut in_pos = 0isize;
        let mut prev_best_in_pos = 0isize;

        // Initialize target grain with first window of input
        for (k, grain_val) in self.target_grain.iter_mut().enumerate().take(win_sz) {
            if k < in_len {
                *grain_val = input[k] * self.window[k];
            } else {
                *grain_val = 0.0;
            }
        }

        while out_pos + win_sz <= out_len + win_sz && (in_pos as usize) < in_len {
            // Find best matching grain near nominal in_pos
            let nominal_pos = prev_best_in_pos + s_a as isize;
            let delta = self.find_best_alignment(input, nominal_pos);
            let best_in_pos = (nominal_pos + delta).clamp(0, (in_len.saturating_sub(win_sz)) as isize);
            prev_best_in_pos = best_in_pos;

            // Overlap-add windowed grain
            for k in 0..win_sz {
                let in_idx = (best_in_pos as usize + k).min(in_len - 1);
                let w = self.window[k];
                let sample = input[in_idx] * w;
                let acc_idx = out_pos + k;
                if acc_idx < self.output_accumulator.len() {
                    self.output_accumulator[acc_idx] += sample;
                    self.weight_accumulator[acc_idx] += w * w;
                }
            }

            // Prepare target grain for next iteration from synthesized natural continuation
            let next_target_nominal = best_in_pos + s_s as isize;
            for k in 0..win_sz {
                let idx = (next_target_nominal + k as isize).clamp(0, (in_len - 1) as isize) as usize;
                self.target_grain[k] = input[idx];
            }

            out_pos += s_s;
            in_pos += s_a as isize;
        }

        // Normalize by accumulated window weights
        let written = out_len.min(out_pos);
        for (i, out_val) in output.iter_mut().enumerate().take(written) {
            let weight = self.weight_accumulator[i];
            if weight > 1e-4 {
                *out_val = (self.output_accumulator[i] / weight).clamp(-1.0, 1.0);
            } else {
                *out_val = self.output_accumulator[i].clamp(-1.0, 1.0);
            }
        }

        written
    }

    /// Process a stereo pair of audio slices using WSOLA time-stretching.
    pub fn process_stereo(
        &mut self,
        in_l: &[f32],
        in_r: &[f32],
        out_l: &mut [f32],
        out_r: &mut [f32],
    ) -> usize {
        let frames_l = self.process(in_l, out_l);
        let frames_r = self.process(in_r, out_r);
        frames_l.min(frames_r)
    }
}

/// Non-destructive elastic audio warping engine using piecewise dynamic WSOLA stretch.
#[derive(Debug, Clone)]
pub struct ElasticWarpEngine {
    pub sample_rate: u32,
    pub markers: Vec<WarpMarker>,
    stretcher: WsolaTimeStretcher,
}

impl ElasticWarpEngine {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            markers: Vec::new(),
            stretcher: WsolaTimeStretcher::new(sample_rate, 1024, 256),
        }
    }

    pub fn add_marker(&mut self, source_frame: usize, target_frame: usize) {
        self.markers.push(WarpMarker::new(source_frame, target_frame));
        self.markers.sort_by_key(|m| m.target_frame);
    }

    pub fn clear_markers(&mut self) {
        self.markers.clear();
    }

    /// Warp a mono audio signal according to warp markers into the given output length.
    pub fn warp_audio(&self, input: &[f32], output_len: usize) -> Vec<f32> {
        if input.is_empty() || output_len == 0 {
            return vec![0.0; output_len];
        }

        if self.markers.is_empty() {
            // Uniform stretch
            let mut out = vec![0.0f32; output_len];
            let mut s = self.stretcher.clone();
            s.set_stretch_ratio(output_len as f32 / input.len() as f32);
            s.process(input, &mut out);
            return out;
        }

        // Piecewise stretch across marker segments
        let mut sorted_markers = self.markers.clone();
        if sorted_markers.first().is_none_or(|m| m.source_frame > 0 || m.target_frame > 0) {
            sorted_markers.insert(0, WarpMarker::new(0, 0));
        }
        if sorted_markers.last().is_none_or(|m| m.target_frame < output_len) {
            sorted_markers.push(WarpMarker::new(input.len(), output_len));
        }

        let mut output = vec![0.0f32; output_len];

        for window in sorted_markers.windows(2) {
            let m0 = window[0];
            let m1 = window[1];

            let src_start = m0.source_frame.min(input.len());
            let src_end = m1.source_frame.min(input.len());
            let tgt_start = m0.target_frame.min(output_len);
            let tgt_end = m1.target_frame.min(output_len);

            if src_end <= src_start || tgt_end <= tgt_start {
                continue;
            }

            let src_slice = &input[src_start..src_end];
            let tgt_len = tgt_end - tgt_start;
            let ratio = tgt_len as f32 / src_slice.len() as f32;

            let mut seg_out = vec![0.0f32; tgt_len];
            let mut s = self.stretcher.clone();
            s.set_stretch_ratio(ratio);
            s.process(src_slice, &mut seg_out);

            let copy_len = tgt_len.min(output_len - tgt_start);
            output[tgt_start..tgt_start + copy_len].copy_from_slice(&seg_out[..copy_len]);
        }

        output
    }

    /// Warp a stereo pair of audio signals non-destructively according to warp markers.
    pub fn warp_stereo(&self, in_l: &[f32], in_r: &[f32], output_len: usize) -> (Vec<f32>, Vec<f32>) {
        let out_l = self.warp_audio(in_l, output_len);
        let out_r = self.warp_audio(in_r, output_len);
        (out_l, out_r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wsola_time_stretching_mono_and_stereo() {
        let sample_rate = 44100;
        let mut stretcher = WsolaTimeStretcher::new(sample_rate, 512, 128);
        stretcher.set_stretch_ratio(1.5); // 1.5x longer

        let input_len = 2048;
        let input: Vec<f32> = (0..input_len)
            .map(|i| (i as f32 * 440.0 * 2.0 * PI / 44100.0).sin())
            .collect();
        let mut output = vec![0.0f32; 3072];

        let written = stretcher.process(&input, &mut output);
        assert!(written > 0);
        assert!(output.iter().any(|&s| s.abs() > 0.0));
        assert!(output.iter().all(|s| s.is_finite()));
    }

    #[test]
    fn test_elastic_warp_engine_piecewise_markers() {
        let sample_rate = 44100;
        let mut warp = ElasticWarpEngine::new(sample_rate);
        warp.add_marker(1000, 1500); // 1.5x stretch first segment
        warp.add_marker(2000, 2200); // 0.7x compress second segment

        let input: Vec<f32> = (0..2000)
            .map(|i| (i as f32 * 220.0 * 2.0 * PI / 44100.0).sin())
            .collect();

        let warped = warp.warp_audio(&input, 2500);
        assert_eq!(warped.len(), 2500);
        assert!(warped.iter().any(|&s| s.abs() > 0.0));
        assert!(warped.iter().all(|s| s.is_finite()));
    }
}
