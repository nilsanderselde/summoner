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

//! Surround & Immersive Bus Fold-Down Downmixing Matrix and Dolby Atmos Bed Routing (Task E5).
//!
//! Provides broadcast-standard multichannel downmixing and Dolby Atmos bed monitoring:
//! - [`DownmixProfile`]: Standard fold-down profiles (Lo/Ro stereo, Lt/Rt matrix, ITU-R BS.775, Mono).
//! - [`MultichannelDownmixer`]: Matrix calculator converting 7.1.4 Atmos bed -> 7.1 -> 5.1 -> Stereo/Mono.
//! - [`DownmixMatrixNode`]: Real-time audio node performing sample-accurate multichannel fold-down.
//! - [`DownmixConfig`]: Center, surround, LFE and height blend configurations.

use crate::spatial_multichannel::SpatialOutputFormat;
use crate::traits::SignalProcessor;
use serde::{Deserialize, Serialize};
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, NodeEvent, ProcessContext};

/// Standard fold-down profiles for surround downmixing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DownmixProfile {
    /// Left-only / Right-only standard stereo downmix (ITU-R BS.775-3).
    LoRoItuR775,
    /// Left-total / Right-total matrix downmix (Pro Logic compatible with 90° surround phase shift).
    LtRtMatrix,
    /// Equal-power custom downmix.
    EqualPower,
    /// Downmix to monophonic single-channel output.
    MonoFold,
}

/// Center speaker fold-down level attenuation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CenterDownmixGain {
    Minus3Db,   // 0.7071
    Minus4_5Db, // 0.5957
    Minus6Db,   // 0.5000
    ZeroDb,     // 1.0000
}

impl CenterDownmixGain {
    pub fn linear_gain(self) -> f32 {
        match self {
            Self::Minus3Db => 0.707_106_77,
            Self::Minus4_5Db => 0.595_662_1,
            Self::Minus6Db => 0.5,
            Self::ZeroDb => 1.0,
        }
    }
}

/// Surround speaker fold-down level attenuation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SurroundDownmixGain {
    Minus3Db, // 0.7071
    Minus6Db, // 0.5000
    ZeroDb,   // 1.0000
}

impl SurroundDownmixGain {
    pub fn linear_gain(self) -> f32 {
        match self {
            Self::Minus3Db => 0.707_106_77,
            Self::Minus6Db => 0.5,
            Self::ZeroDb => 1.0,
        }
    }
}

/// Configuration settings for the multichannel fold-down matrix.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DownmixConfig {
    pub profile: DownmixProfile,
    pub center_gain: CenterDownmixGain,
    pub surround_gain: SurroundDownmixGain,
    /// Whether to include LFE channel in stereo/mono fold-down (typically muted to prevent clipping).
    pub include_lfe: bool,
    /// Gain applied to LFE if included (e.g. -10 dB or 0.3162).
    pub lfe_gain: f32,
    /// Overhead height speaker blend factor into surround channels for 7.1.4 -> 7.1/5.1 (0.0 to 1.0).
    pub height_blend: f32,
}

impl Default for DownmixConfig {
    fn default() -> Self {
        Self {
            profile: DownmixProfile::LoRoItuR775,
            center_gain: CenterDownmixGain::Minus3Db,
            surround_gain: SurroundDownmixGain::Minus3Db,
            include_lfe: false,
            lfe_gain: 0.316_227_76, // -10 dB
            height_blend: 0.707_106_77,
        }
    }
}

/// Pure math and matrix processor for multichannel fold-down.
#[derive(Debug, Clone)]
pub struct MultichannelDownmixer {
    pub config: DownmixConfig,
}

impl MultichannelDownmixer {
    pub fn new(config: DownmixConfig) -> Self {
        Self { config }
    }

    /// Fold 7.1.4 (12 channels) down to 7.1 (8 channels).
    ///
    /// 7.1.4: L(0), R(1), C(2), LFE(3), Lss(4), Rss(5), Lsr(6), Rsr(7), Tfl(8), Tfr(9), Trl(10), Trr(11).
    pub fn fold_7_1_4_to_7_1(&self, in_12ch: &[Sample], out_8ch: &mut [Sample]) {
        assert!(in_12ch.len() >= 12 && out_8ch.len() >= 8);
        let hb = self.config.height_blend;

        // Front 3 + LFE direct
        out_8ch[0] = in_12ch[0]; // L
        out_8ch[1] = in_12ch[1]; // R
        out_8ch[2] = in_12ch[2]; // C
        out_8ch[3] = in_12ch[3]; // LFE

        // Side surrounds blend with front top
        out_8ch[4] = in_12ch[4] + in_12ch[8] * hb; // Lss + Tfl * hb
        out_8ch[5] = in_12ch[5] + in_12ch[9] * hb; // Rss + Tfr * hb

        // Rear surrounds blend with rear top
        out_8ch[6] = in_12ch[6] + in_12ch[10] * hb; // Lsr + Trl * hb
        out_8ch[7] = in_12ch[7] + in_12ch[11] * hb; // Rsr + Trr * hb
    }

    /// Fold 7.1 (8 channels) down to 5.1 (6 channels).
    ///
    /// 7.1: L(0), R(1), C(2), LFE(3), Lss(4), Rss(5), Lsr(6), Rsr(7).
    /// 5.1: L(0), R(1), C(2), LFE(3), Ls(4), Rs(5).
    pub fn fold_7_1_to_5_1(&self, in_8ch: &[Sample], out_6ch: &mut [Sample]) {
        assert!(in_8ch.len() >= 8 && out_6ch.len() >= 6);
        let sg = self.config.surround_gain.linear_gain();

        out_6ch[0] = in_8ch[0]; // L
        out_6ch[1] = in_8ch[1]; // R
        out_6ch[2] = in_8ch[2]; // C
        out_6ch[3] = in_8ch[3]; // LFE

        // Combine side and rear surrounds into single surround pair
        out_8ch_surrounds_to_5_1(in_8ch, out_6ch, sg);
    }

    /// Fold 5.1 (6 channels) down to Stereo (2 channels) according to ITU-R BS.775-3 or Lt/Rt.
    ///
    /// 5.1: L(0), R(1), C(2), LFE(3), Ls(4), Rs(5).
    pub fn fold_5_1_to_stereo(&self, in_6ch: &[Sample], out_stereo: &mut [Sample]) {
        assert!(in_6ch.len() >= 6 && out_stereo.len() >= 2);
        let cg = self.config.center_gain.linear_gain();
        let sg = self.config.surround_gain.linear_gain();
        let lfe = if self.config.include_lfe {
            in_6ch[3] * self.config.lfe_gain
        } else {
            0.0
        };

        match self.config.profile {
            DownmixProfile::LoRoItuR775 | DownmixProfile::EqualPower => {
                out_stereo[0] = in_6ch[0] + in_6ch[2] * cg + in_6ch[4] * sg + lfe;
                out_stereo[1] = in_6ch[1] + in_6ch[2] * cg + in_6ch[5] * sg + lfe;
            }
            DownmixProfile::LtRtMatrix => {
                let mono_surround = (in_6ch[4] + in_6ch[5]) * sg * 0.707_106_77;
                out_stereo[0] = in_6ch[0] + in_6ch[2] * cg - mono_surround + lfe;
                out_stereo[1] = in_6ch[1] + in_6ch[2] * cg + mono_surround + lfe;
            }
            DownmixProfile::MonoFold => {
                let mono = (in_6ch[0] + in_6ch[1] + in_6ch[2] * cg + (in_6ch[4] + in_6ch[5]) * sg)
                    * 0.5
                    + lfe;
                out_stereo[0] = mono;
                out_stereo[1] = mono;
            }
        }
    }

    /// Direct fold-down from 7.1.4 Atmos bed directly to Stereo.
    pub fn fold_7_1_4_to_stereo(&self, in_12ch: &[Sample], out_stereo: &mut [Sample]) {
        let mut intermediate_8ch = [0.0f32; 8];
        let mut intermediate_6ch = [0.0f32; 6];
        self.fold_7_1_4_to_7_1(in_12ch, &mut intermediate_8ch);
        self.fold_7_1_to_5_1(&intermediate_8ch, &mut intermediate_6ch);
        self.fold_5_1_to_stereo(&intermediate_6ch, out_stereo);
    }
}

fn out_8ch_surrounds_to_5_1(in_8ch: &[Sample], out_6ch: &mut [Sample], sg: f32) {
    out_6ch[4] = in_8ch[4] + in_8ch[6] * sg; // Ls = Lss + Lsr * sg
    out_6ch[5] = in_8ch[5] + in_8ch[7] * sg; // Rs = Rss + Rsr * sg
}

/// Real-time audio node performing multichannel fold-down.
#[derive(Debug, Clone)]
pub struct DownmixMatrixNode {
    downmixer: MultichannelDownmixer,
    input_format: SpatialOutputFormat,
    output_format: SpatialOutputFormat,
    bypass: bool,
}

impl DownmixMatrixNode {
    pub fn new(
        input_format: SpatialOutputFormat,
        output_format: SpatialOutputFormat,
        config: DownmixConfig,
    ) -> Self {
        Self {
            downmixer: MultichannelDownmixer::new(config),
            input_format,
            output_format,
            bypass: false,
        }
    }

    pub fn set_bypass(&mut self, bypass: bool) {
        self.bypass = bypass;
    }

    pub fn is_bypassed(&self) -> bool {
        self.bypass
    }

    pub fn process_frame(&self, inputs: &[Sample], outputs: &mut [Sample]) {
        if self.bypass {
            let copy_count = inputs.len().min(outputs.len());
            outputs[..copy_count].copy_from_slice(&inputs[..copy_count]);
            return;
        }

        match (self.input_format, self.output_format) {
            (SpatialOutputFormat::DolbyAtmos7_1_4, SpatialOutputFormat::Stereo) => {
                if inputs.len() >= 12 && outputs.len() >= 2 {
                    self.downmixer
                        .fold_7_1_4_to_stereo(&inputs[..12], &mut outputs[..2]);
                }
            }
            (SpatialOutputFormat::Surround7_1, SpatialOutputFormat::Surround5_1) => {
                if inputs.len() >= 8 && outputs.len() >= 6 {
                    self.downmixer
                        .fold_7_1_to_5_1(&inputs[..8], &mut outputs[..6]);
                }
            }
            (SpatialOutputFormat::Surround5_1, SpatialOutputFormat::Stereo) => {
                if inputs.len() >= 6 && outputs.len() >= 2 {
                    self.downmixer
                        .fold_5_1_to_stereo(&inputs[..6], &mut outputs[..2]);
                }
            }
            (SpatialOutputFormat::DolbyAtmos7_1_4, SpatialOutputFormat::Surround5_1) => {
                if inputs.len() >= 12 && outputs.len() >= 6 {
                    let mut tmp_8ch = [0.0f32; 8];
                    self.downmixer
                        .fold_7_1_4_to_7_1(&inputs[..12], &mut tmp_8ch);
                    self.downmixer.fold_7_1_to_5_1(&tmp_8ch, &mut outputs[..6]);
                }
            }
            _ => {
                let count = inputs.len().min(outputs.len());
                outputs[..count].copy_from_slice(&inputs[..count]);
            }
        }
    }
}

impl SignalProcessor for DownmixMatrixNode {
    fn name(&self) -> &str {
        "DownmixMatrixNode"
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
        let frames = inputs[0].len().min(outputs[0].len());
        let in_channels = inputs.len();
        let out_channels = outputs.len();

        let mut in_frame = vec![0.0f32; in_channels];
        let mut out_frame = vec![0.0f32; out_channels];

        for f in 0..frames {
            for (ch, in_ch) in inputs.iter().enumerate().take(in_channels) {
                in_frame[ch] = in_ch[f];
            }
            self.process_frame(&in_frame, &mut out_frame);
            for (ch, out_ch) in outputs.iter_mut().enumerate().take(out_channels) {
                out_ch[f] = out_frame[ch];
            }
        }
    }
}

impl AudioNode for DownmixMatrixNode {
    fn name(&self) -> &str {
        "DownmixMatrixNode"
    }

    fn process(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        self.process_block(inputs, outputs, ctx);
    }

    fn handle_event(&mut self, _event: &NodeEvent, _ctx: &ProcessContext) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_5_1_to_stereo_loro() {
        let mixer = MultichannelDownmixer::new(DownmixConfig::default());
        // L=1.0, R=0.5, C=1.0, LFE=1.0 (ignored by default), Ls=0.5, Rs=0.5
        let in_5_1 = [1.0, 0.5, 1.0, 1.0, 0.5, 0.5];
        let mut out_stereo = [0.0; 2];

        mixer.fold_5_1_to_stereo(&in_5_1, &mut out_stereo);

        // L_out = 1.0 + 1.0 * 0.7071 + 0.5 * 0.7071
        let expected_l = 1.0 + 1.0 * 0.707_106_77 + 0.5 * 0.707_106_77;
        assert!((out_stereo[0] - expected_l).abs() < 1e-4);

        // R_out = 0.5 + 1.0 * 0.7071 + 0.5 * 0.7071
        let expected_r = 0.5 + 1.0 * 0.707_106_77 + 0.5 * 0.707_106_77;
        assert!((out_stereo[1] - expected_r).abs() < 1e-4);
    }

    #[test]
    fn test_7_1_to_5_1_fold_down() {
        let mixer = MultichannelDownmixer::new(DownmixConfig::default());
        // L, R, C, LFE, Lss, Rss, Lsr, Rsr
        let in_7_1 = [1.0, 1.0, 1.0, 0.0, 0.5, 0.5, 0.5, 0.5];
        let mut out_5_1 = [0.0; 6];

        mixer.fold_7_1_to_5_1(&in_7_1, &mut out_5_1);

        assert_eq!(out_5_1[0], 1.0); // L
        assert_eq!(out_5_1[1], 1.0); // R
        assert_eq!(out_5_1[2], 1.0); // C
        assert_eq!(out_5_1[3], 0.0); // LFE
                                     // Ls = 0.5 + 0.5 * 0.7071
        let expected_ls = 0.5 + 0.5 * 0.707_106_77;
        assert!((out_5_1[4] - expected_ls).abs() < 1e-4);
    }

    #[test]
    fn test_atmos_7_1_4_to_stereo_downmix_node() {
        let node = DownmixMatrixNode::new(
            SpatialOutputFormat::DolbyAtmos7_1_4,
            SpatialOutputFormat::Stereo,
            DownmixConfig::default(),
        );

        let mut inputs = [0.0f32; 12];
        inputs[0] = 1.0; // Left channel unit impulse
        let mut outputs = [0.0f32; 2];

        node.process_frame(&inputs, &mut outputs);
        assert!((outputs[0] - 1.0).abs() < 1e-5);
        assert_eq!(outputs[1], 0.0);
    }
}
