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

//! Multi-mode distortion, overdrive, bitcrusher, and wavefolder processors with polyphase FIR oversampling support.

use crate::oversampling::Oversampler;
use crate::traits::SignalProcessor;
use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;

/// Distortion algorithm types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistortionType {
    /// Smooth analog hyperbolic tangent saturation.
    SoftClipping,
    /// Brickwall hard threshold clipping.
    HardClipping,
    /// Asymmetrical 2nd/3rd order polynomial tube saturation.
    TubeOverdrive,
    /// Sample-rate reduction and bit-depth quantization.
    Bitcrusher,
    /// High-gain sine wavefolding fuzz.
    Fuzz,
    /// Dynamic multi-stage wavefolder.
    Wavefolder,
}

/// Multi-algorithm Distortion and Bitcrusher DSP node with anti-aliasing oversampler.
#[derive(Debug)]
pub struct DistortionNode {
    pub distortion_type: DistortionType,
    pub drive: f32,
    pub tone: f32,             // Lowpass cutoff multiplier 0.1 to 1.0
    pub mix: f32,              // Dry/Wet blend 0.0 to 1.0
    pub bit_depth: u8,         // Bitcrusher depth 1 to 16
    pub sample_reduction: u32, // Downsampling factor 1 to 32
    pub oversample_factor: u8, // 1 (off), 2, 4, or 8
    lp_state: f32,
    downsample_counter: u32,
    held_sample: f32,
    oversampler: Option<Oversampler>,
}

impl DistortionNode {
    pub fn new(distortion_type: DistortionType, drive: f32) -> Self {
        Self {
            distortion_type,
            drive: drive.max(1.0),
            tone: 1.0,
            mix: 1.0,
            bit_depth: 8,
            sample_reduction: 1,
            oversample_factor: 1,
            lp_state: 0.0,
            downsample_counter: 0,
            held_sample: 0.0,
            oversampler: None,
        }
    }

    pub fn set_oversample_factor(&mut self, factor: u8) {
        self.oversample_factor = factor;
        if factor > 1 {
            self.oversampler = Some(Oversampler::new(factor as usize));
        } else {
            self.oversampler = None;
        }
    }

    fn apply_distortion(
        distortion_type: DistortionType,
        driven: f32,
        bit_depth: u8,
        sample_reduction: u32,
        downsample_counter: &mut u32,
        held_sample: &mut f32,
    ) -> f32 {
        match distortion_type {
            DistortionType::SoftClipping => (driven * 0.8).tanh(),
            DistortionType::HardClipping => driven.clamp(-1.0, 1.0),
            DistortionType::TubeOverdrive => {
                let x = driven.clamp(-3.0, 3.0);
                if x > 0.0 {
                    x - 0.15 * x * x
                } else {
                    x + 0.15 * x * x
                }
            }
            DistortionType::Bitcrusher => {
                *downsample_counter += 1;
                if *downsample_counter >= sample_reduction.max(1) {
                    *downsample_counter = 0;
                    let steps = (1 << bit_depth.clamp(1, 16)) as f32;
                    let quantized = (driven.clamp(-1.0, 1.0) * steps).round() / steps;
                    *held_sample = quantized;
                }
                *held_sample
            }
            DistortionType::Fuzz => (driven * std::f32::consts::PI).sin().clamp(-1.0, 1.0),
            DistortionType::Wavefolder => {
                let folded = (driven + 1.0).rem_euclid(4.0) - 2.0;
                (folded.abs() - 1.0).clamp(-1.0, 1.0)
            }
        }
    }

    pub fn process_sample(&mut self, input: f32) -> f32 {
        let driven = input * self.drive;

        let processed = if self.oversample_factor > 1 {
            if self.oversampler.is_none() {
                self.oversampler = Some(Oversampler::new(self.oversample_factor as usize));
            }
            let factor = self.oversample_factor as usize;
            let mut up_buf = vec![0.0f32; factor];
            let mut sub_out = vec![0.0f32; factor];

            let dist_type = self.distortion_type;
            let b_depth = self.bit_depth;
            let s_red = self.sample_reduction;

            if let Some(ref mut os) = self.oversampler {
                os.process_up(driven, &mut up_buf);
                for sub in 0..factor {
                    sub_out[sub] = Self::apply_distortion(
                        dist_type,
                        up_buf[sub],
                        b_depth,
                        s_red,
                        &mut self.downsample_counter,
                        &mut self.held_sample,
                    );
                }
                os.process_down(&sub_out)
            } else {
                Self::apply_distortion(
                    dist_type,
                    driven,
                    b_depth,
                    s_red,
                    &mut self.downsample_counter,
                    &mut self.held_sample,
                )
            }
        } else {
            Self::apply_distortion(
                self.distortion_type,
                driven,
                self.bit_depth,
                self.sample_reduction,
                &mut self.downsample_counter,
                &mut self.held_sample,
            )
        };

        // Simple 1-pole tone filter
        let alpha = self.tone.clamp(0.05, 1.0);
        self.lp_state += alpha * (processed - self.lp_state);
        let filtered = self.lp_state;

        // Dry/Wet mix
        input * (1.0 - self.mix) + filtered * self.mix
    }
}

impl SignalProcessor for DistortionNode {
    fn name(&self) -> &str {
        "DistortionNode"
    }

    fn process_block(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        _ctx: &ProcessContext,
    ) {
        if outputs.is_empty() {
            return;
        }

        let num_samples = outputs[0].len();
        for i in 0..num_samples {
            let in_sample = if !inputs.is_empty() && !inputs[0].is_empty() && i < inputs[0].len() {
                inputs[0][i]
            } else {
                0.0
            };

            let out_sample = self.process_sample(in_sample);
            for out_ch in outputs.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = out_sample;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soft_and_hard_clipping() {
        let mut soft = DistortionNode::new(DistortionType::SoftClipping, 5.0);
        let mut hard = DistortionNode::new(DistortionType::HardClipping, 5.0);

        let out_soft = soft.process_sample(0.8);
        let out_hard = hard.process_sample(0.8);

        assert!(out_soft.abs() <= 1.0);
        assert_eq!(out_hard, 1.0);
    }

    #[test]
    fn test_bitcrusher() {
        let mut crusher = DistortionNode::new(DistortionType::Bitcrusher, 1.0);
        crusher.bit_depth = 4; // 16 steps
        let sample = crusher.process_sample(0.33);
        assert!((sample - 0.3125).abs() < 0.1);
    }

    #[test]
    fn test_distortion_oversampling() {
        let mut dist = DistortionNode::new(DistortionType::SoftClipping, 4.0);
        dist.set_oversample_factor(4);

        let out = dist.process_sample(0.5);
        assert!(out.is_finite());
        assert!(out.abs() <= 1.0);
    }
}
