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
}
