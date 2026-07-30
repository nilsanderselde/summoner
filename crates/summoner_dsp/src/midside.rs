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

//! Mid-Side stereo width control DSP processor node.

use crate::traits::SignalProcessor;
use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;

/// Mid-Side stereo width processor node.
/// `width = 0.0`: pure mono (mid only)
/// `width = 1.0`: un-modified stereo
/// `width > 1.0`: expanded stereo field
#[derive(Debug)]
pub struct MidSideNode {
    pub width: f32,
}

impl MidSideNode {
    pub fn new(width: f32) -> Self {
        Self {
            width: width.clamp(0.0, 4.0),
        }
    }
}

impl Default for MidSideNode {
    fn default() -> Self {
        Self::new(1.0)
    }
}

impl SignalProcessor for MidSideNode {
    fn name(&self) -> &str {
        "MidSideNode"
    }

    fn process_block(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        _ctx: &ProcessContext,
    ) {
        if inputs.is_empty() || outputs.is_empty() {
            return;
        }

        let num_samples = outputs[0].len();
        let is_stereo_in = inputs.len() > 1;
        let is_stereo_out = outputs.len() > 1;

        for i in 0..num_samples {
            let l = if !inputs[0].is_empty() && i < inputs[0].len() { inputs[0][i] } else { 0.0 };
            let r = if is_stereo_in && !inputs[1].is_empty() && i < inputs[1].len() { inputs[1][i] } else { l };

            let mid = 0.5 * (l + r);
            let side = 0.5 * (l - r) * self.width;

            let out_l = mid + side;
            let out_r = mid - side;

            if !outputs[0].is_empty() && i < outputs[0].len() {
                outputs[0][i] = out_l;
            }
            if is_stereo_out && !outputs[1].is_empty() && i < outputs[1].len() {
                outputs[1][i] = out_r;
            }
        }
    }
}

/// Stereo Imager tool supporting L/R delay, phase offset, and stereo width.
#[derive(Debug)]
pub struct StereoImager {
    pub l_delay_ms: f32,
    pub r_delay_ms: f32,
    pub phase_offset: f32,
    pub width: f32,
    l_buffer: Vec<f32>,
    r_buffer: Vec<f32>,
    l_pos: usize,
    r_pos: usize,
    sample_rate: u32,
}

impl StereoImager {
    pub fn new(sample_rate: u32) -> Self {
        let max_samples = (sample_rate as f32 * 0.1) as usize; // up to 100ms max delay
        Self {
            l_delay_ms: 0.0,
            r_delay_ms: 0.0,
            phase_offset: 0.0,
            width: 1.0,
            l_buffer: vec![0.0; max_samples.max(1)],
            r_buffer: vec![0.0; max_samples.max(1)],
            l_pos: 0,
            r_pos: 0,
            sample_rate,
        }
    }

    pub fn process_stereo(&mut self, left: f32, right: f32) -> (f32, f32) {
        let max_len = self.l_buffer.len();
        self.l_buffer[self.l_pos] = left;
        self.r_buffer[self.r_pos] = right;

        let l_delay_samples = ((self.l_delay_ms * 0.001 * self.sample_rate as f32) as usize).min(max_len - 1);
        let r_delay_samples = ((self.r_delay_ms * 0.001 * self.sample_rate as f32) as usize).min(max_len - 1);

        let l_read_idx = (self.l_pos + max_len - l_delay_samples) % max_len;
        let r_read_idx = (self.r_pos + max_len - r_delay_samples) % max_len;

        let mut l_out = self.l_buffer[l_read_idx];
        let mut r_out = self.r_buffer[r_read_idx];

        self.l_pos = (self.l_pos + 1) % max_len;
        self.r_pos = (self.r_pos + 1) % max_len;

        // Apply phase offset (invert right channel if phase_offset near PI)
        if (self.phase_offset - std::f32::consts::PI).abs() < 0.1 {
            r_out = -r_out;
        }

        // Apply mid-side width scaling
        let mid = 0.5 * (l_out + r_out);
        let side = 0.5 * (l_out - r_out) * self.width;

        (mid + side, mid - side)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::transport::Transport;

    #[test]
    fn test_midside_mono_width_zero() {
        let transport = Transport::new(44100, 120.0);
        let ctx = ProcessContext::from_transport(&transport);

        let mut ms = MidSideNode::new(0.0);
        let l_in = vec![1.0f32; 64];
        let r_in = vec![-1.0f32; 64];
        let mut l_out = vec![0.0f32; 64];
        let mut r_out = vec![0.0f32; 64];

        ms.process_block(&[&l_in[..], &r_in[..]], &mut [&mut l_out[..], &mut r_out[..]], &ctx);

        // Mono width = 0.0 results in L_out == R_out == 0.0 (since L+R = 0)
        assert_eq!(l_out[0], 0.0);
        assert_eq!(r_out[0], 0.0);
    }

    #[test]
    fn test_stereo_imager() {
        let mut imager = StereoImager::new(44100);
        imager.width = 1.5;
        imager.l_delay_ms = 5.0;
        let (out_l, out_r) = imager.process_stereo(1.0, -1.0);
        assert!(out_l.is_finite());
        assert!(out_r.is_finite());
    }
}

