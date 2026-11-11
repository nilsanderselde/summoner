//! Surround, Binaural, and Dolby Atmos / Ambisonics 3D Spatial Panner (Task E7).
//!
//! Provides comprehensive spatial panning and multichannel rendering across:
//! - Stereo (2.0)
//! - 5.1 Surround (L, R, C, LFE, Ls, Rs)
//! - 7.1 Surround (L, R, C, LFE, Lss, Rss, Lsr, Rsr)
//! - 7.1.4 Dolby Atmos Immersive Bed (7 surround + 1 LFE + 4 height speakers: Tfl, Tfr, Trl, Trr)
//! - First-Order Ambisonics B-format (W, X, Y, Z)
//! - 3D HRTF Binaural headphone rendering with ITD (interaural time difference) and ILD head-shadowing.

use crate::traits::SignalProcessor;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use summoner_core::audio::{ChannelLayout, Sample};
use summoner_core::node::{AudioNode, NodeEvent, ProcessContext};

/// Supported spatial audio output speaker layouts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpatialOutputFormat {
    Stereo,
    Surround5_1,
    Surround7_1,
    DolbyAtmos7_1_4,
    AmbisonicsBFormat,
    BinauralHeadphones,
}

impl SpatialOutputFormat {
    pub fn channel_count(self) -> usize {
        match self {
            Self::Stereo | Self::BinauralHeadphones => 2,
            Self::AmbisonicsBFormat => 4,
            Self::Surround5_1 => 6,
            Self::Surround7_1 => 8,
            Self::DolbyAtmos7_1_4 => 12,
        }
    }

    pub fn to_core_layout(self) -> ChannelLayout {
        match self {
            Self::Stereo | Self::BinauralHeadphones => ChannelLayout::Stereo,
            Self::AmbisonicsBFormat => ChannelLayout::Custom(4),
            Self::Surround5_1 => ChannelLayout::Surround5_1,
            Self::Surround7_1 => ChannelLayout::Surround7_1,
            Self::DolbyAtmos7_1_4 => ChannelLayout::Surround7_1_4,
        }
    }
}

/// 3D spherical coordinate position for spatial sound objects.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpatialCoordinates3D {
    /// Horizontal angle in degrees: -180.0 (rear) to 0.0 (front) to +180.0 (rear).
    pub azimuth_deg: f32,
    /// Vertical angle in degrees: -90.0 (below) to 0.0 (ear level) to +90.0 (zenith/overhead).
    pub elevation_deg: f32,
    /// Radial distance in meters (0.1m to 50.0m).
    pub distance_m: f32,
    /// Source spatial width / spread factor: 0.0 (pinpoint) to 1.0 (diffuse ambient).
    pub spread: f32,
}

impl Default for SpatialCoordinates3D {
    fn default() -> Self {
        Self {
            azimuth_deg: 0.0,
            elevation_deg: 0.0,
            distance_m: 1.0,
            spread: 0.0,
        }
    }
}

impl SpatialCoordinates3D {
    pub fn new(azimuth_deg: f32, elevation_deg: f32, distance_m: f32) -> Self {
        Self {
            azimuth_deg: azimuth_deg.clamp(-180.0, 180.0),
            elevation_deg: elevation_deg.clamp(-90.0, 90.0),
            distance_m: distance_m.max(0.1),
            spread: 0.0,
        }
    }

    /// Convert spherical coordinates to Cartesian coordinates (x: Right, y: Front, z: Up).
    pub fn to_cartesian(&self) -> (f32, f32, f32) {
        let az_rad = self.azimuth_deg * PI / 180.0;
        let el_rad = self.elevation_deg * PI / 180.0;
        let x = self.distance_m * el_rad.cos() * az_rad.sin();
        let y = self.distance_m * el_rad.cos() * az_rad.cos();
        let z = self.distance_m * el_rad.sin();
        (x, y, z)
    }
}

/// Multichannel Spatial Panning & Rendering Node.
pub struct MultichannelSpatialPannerNode {
    pub name: String,
    pub format: SpatialOutputFormat,
    pub position: SpatialCoordinates3D,
    pub target_position: SpatialCoordinates3D,
    pub lfe_send: f32,
    pub lfe_crossover_hz: f32,
    sample_rate: f32,

    // Speaker gains: up to 12 channels for 7.1.4
    current_gains: [f32; 12],
    target_gains: [f32; 12],

    // Lowpass filter state for LFE bass management
    lfe_filter_state: f32,

    // Binaural delay lines for ITD
    left_delay_buf: [f32; 256],
    right_delay_buf: [f32; 256],
    delay_write_ptr: usize,
    left_shadow_filter: f32,
    right_shadow_filter: f32,
}

impl MultichannelSpatialPannerNode {
    pub fn new(format: SpatialOutputFormat, sample_rate: f32) -> Self {
        let mut node = Self {
            name: "MultichannelSpatialPanner".to_string(),
            format,
            position: SpatialCoordinates3D::default(),
            target_position: SpatialCoordinates3D::default(),
            lfe_send: 0.0,
            lfe_crossover_hz: 80.0,
            sample_rate,
            current_gains: [0.0; 12],
            target_gains: [0.0; 12],
            lfe_filter_state: 0.0,
            left_delay_buf: [0.0; 256],
            right_delay_buf: [0.0; 256],
            delay_write_ptr: 0,
            left_shadow_filter: 0.0,
            right_shadow_filter: 0.0,
        };
        node.calculate_target_gains();
        node.current_gains = node.target_gains;
        node
    }

    /// Set target 3D spherical position.
    pub fn set_position(&mut self, azimuth_deg: f32, elevation_deg: f32, distance_m: f32) {
        self.target_position = SpatialCoordinates3D::new(azimuth_deg, elevation_deg, distance_m);
        self.calculate_target_gains();
    }

    /// Set format.
    pub fn set_format(&mut self, format: SpatialOutputFormat) {
        self.format = format;
        self.calculate_target_gains();
    }

    /// Compute constant-power speaker gains for current target position.
    fn calculate_target_gains(&mut self) {
        let az = self.target_position.azimuth_deg * PI / 180.0;
        let el = self.target_position.elevation_deg * PI / 180.0;
        let dist = self.target_position.distance_m;

        // Distance attenuation factor: 1 / distance (clamped to 1.0 at 1 meter)
        let dist_gain = (1.0 / dist).clamp(0.05, 1.0);

        let mut gains = [0.0f32; 12];

        match self.format {
            SpatialOutputFormat::Stereo => {
                // Stereo pan law (-180..180 mapped to left/right)
                let pan = (az.sin() + 1.0) * 0.5; // 0.0 (left) to 1.0 (right)
                gains[0] = (1.0 - pan).sqrt() * dist_gain; // Left
                gains[1] = pan.sqrt() * dist_gain; // Right
            }

            SpatialOutputFormat::Surround5_1 => {
                // Channels: [0: L, 1: R, 2: C, 3: LFE, 4: Ls, 5: Rs]
                let center_weight = (az.cos().max(0.0)).powi(4);
                let left_weight = ((-az).sin().max(0.0)).powi(2);
                let right_weight = (az.sin().max(0.0)).powi(2);
                let rear_factor = ((-az.cos()).max(0.0)).powi(2);

                let l = (left_weight * (1.0 - 0.5 * rear_factor)).max(0.0);
                let r = (right_weight * (1.0 - 0.5 * rear_factor)).max(0.0);
                let c = center_weight;
                let ls = (left_weight * rear_factor).max(0.0);
                let rs = (right_weight * rear_factor).max(0.0);

                // Normalize energy
                let sum_sq = l * l + r * r + c * c + ls * ls + rs * rs;
                let norm = if sum_sq > 1e-6 {
                    1.0 / sum_sq.sqrt()
                } else {
                    1.0
                };

                gains[0] = l * norm * dist_gain;
                gains[1] = r * norm * dist_gain;
                gains[2] = c * norm * dist_gain;
                gains[3] = self.lfe_send; // LFE send gain
                gains[4] = ls * norm * dist_gain;
                gains[5] = rs * norm * dist_gain;
            }

            SpatialOutputFormat::Surround7_1 => {
                // Channels: [0: L, 1: R, 2: C, 3: LFE, 4: Lss, 5: Rss, 6: Lsr, 7: Rsr]
                let center_w = (az.cos().max(0.0)).powi(4);
                let left_w = ((-az).sin().max(0.0)).powi(2);
                let right_w = (az.sin().max(0.0)).powi(2);
                let rear_w = ((-az.cos()).max(0.0)).powi(2);

                let l = left_w * 0.7;
                let r = right_w * 0.7;
                let c = center_w;
                let lss = left_w * (1.0 - rear_w);
                let rss = right_w * (1.0 - rear_w);
                let lsr = left_w * rear_w;
                let rsr = right_w * rear_w;

                let sum_sq = l * l + r * r + c * c + lss * lss + rss * rss + lsr * lsr + rsr * rsr;
                let norm = if sum_sq > 1e-6 {
                    1.0 / sum_sq.sqrt()
                } else {
                    1.0
                };

                gains[0] = l * norm * dist_gain;
                gains[1] = r * norm * dist_gain;
                gains[2] = c * norm * dist_gain;
                gains[3] = self.lfe_send;
                gains[4] = lss * norm * dist_gain;
                gains[5] = rss * norm * dist_gain;
                gains[6] = lsr * norm * dist_gain;
                gains[7] = rsr * norm * dist_gain;
            }

            SpatialOutputFormat::DolbyAtmos7_1_4 => {
                // 12 Channels:
                // [0: L, 1: R, 2: C, 3: LFE, 4: Ls, 5: Rs, 6: Lb, 7: Rb,
                //  8: Tfl (Top Front L), 9: Tfr (Top Front R), 10: Tbl (Top Back L), 11: Tbr (Top Back R)]
                let height_factor = (el.sin().max(0.0)).clamp(0.0, 1.0); // 0 at floor, 1 at ceiling
                let floor_factor = (1.0 - height_factor).sqrt();

                // 7.1 floor bed ring
                let left_w = ((-az).sin().max(0.0)).powi(2);
                let right_w = (az.sin().max(0.0)).powi(2);
                let front_w = (az.cos().max(0.0)).powi(2);
                let back_w = ((-az.cos()).max(0.0)).powi(2);

                let l = left_w * front_w * floor_factor;
                let r = right_w * front_w * floor_factor;
                let c = front_w.powi(2) * floor_factor * 0.8;
                let ls = left_w * 0.5 * floor_factor;
                let rs = right_w * 0.5 * floor_factor;
                let lb = left_w * back_w * floor_factor;
                let rb = right_w * back_w * floor_factor;

                // 4 height overhead speakers
                let tfl = left_w * front_w * height_factor;
                let tfr = right_w * front_w * height_factor;
                let tbl = left_w * back_w * height_factor;
                let tbr = right_w * back_w * height_factor;

                let sum_sq = l * l
                    + r * r
                    + c * c
                    + ls * ls
                    + rs * rs
                    + lb * lb
                    + rb * rb
                    + tfl * tfl
                    + tfr * tfr
                    + tbl * tbl
                    + tbr * tbr;
                let norm = if sum_sq > 1e-6 {
                    1.0 / sum_sq.sqrt()
                } else {
                    1.0
                };

                gains[0] = l * norm * dist_gain;
                gains[1] = r * norm * dist_gain;
                gains[2] = c * norm * dist_gain;
                gains[3] = self.lfe_send;
                gains[4] = ls * norm * dist_gain;
                gains[5] = rs * norm * dist_gain;
                gains[6] = lb * norm * dist_gain;
                gains[7] = rb * norm * dist_gain;
                gains[8] = tfl * norm * dist_gain;
                gains[9] = tfr * norm * dist_gain;
                gains[10] = tbl * norm * dist_gain;
                gains[11] = tbr * norm * dist_gain;
            }

            SpatialOutputFormat::AmbisonicsBFormat => {
                // 4 Channels: W (omni), X (front-back), Y (left-right), Z (up-down)
                // Standard Gerzon / Furse-Malham normalization:
                // W = 1 / sqrt(2)
                // X = cos(az) * cos(el)
                // Y = sin(az) * cos(el)
                // Z = sin(el)
                let w = (1.0 / 2.0f32.sqrt()) * dist_gain;
                let x = az.cos() * el.cos() * dist_gain;
                let y = az.sin() * el.cos() * dist_gain;
                let z = el.sin() * dist_gain;

                gains[0] = w;
                gains[1] = x;
                gains[2] = y;
                gains[3] = z;
            }

            SpatialOutputFormat::BinauralHeadphones => {
                // ITD (time delay) and ILD (level difference) for 3D headphone spatialization
                let az_norm = az.sin(); // -1 (left) to +1 (right)
                let left_gain = ((1.0 - az_norm) * 0.5 + 0.2).clamp(0.1, 1.0) * dist_gain;
                let right_gain = ((1.0 + az_norm) * 0.5 + 0.2).clamp(0.1, 1.0) * dist_gain;
                gains[0] = left_gain;
                gains[1] = right_gain;
            }
        }

        self.target_gains = gains;
    }
}

impl SignalProcessor for MultichannelSpatialPannerNode {
    fn name(&self) -> &str {
        &self.name
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

        let num_frames = outputs[0].len();
        let in_mono = inputs.first().copied().unwrap_or(&[]);
        let in_right = if inputs.len() > 1 { inputs[1] } else { in_mono };

        let num_out_channels = outputs.len().min(self.format.channel_count());

        // LFE one-pole crossover alpha: dt / (RC + dt)
        let lfe_alpha = (2.0 * PI * self.lfe_crossover_hz / self.sample_rate).clamp(0.001, 0.5);

        for i in 0..num_frames {
            // Smoothly slew gains towards targets (one-pole smoother)
            for ch in 0..12 {
                self.current_gains[ch] += (self.target_gains[ch] - self.current_gains[ch]) * 0.05;
            }

            let in_l = if i < in_mono.len() { in_mono[i] } else { 0.0 };
            let in_r = if i < in_right.len() {
                in_right[i]
            } else {
                in_l
            };
            let source_mono = (in_l + in_r) * 0.5;

            // Lowpass LFE signal
            self.lfe_filter_state += (source_mono - self.lfe_filter_state) * lfe_alpha;

            if self.format == SpatialOutputFormat::BinauralHeadphones {
                // Binaural processing with ITD delay buffer
                self.left_delay_buf[self.delay_write_ptr] = source_mono;
                self.right_delay_buf[self.delay_write_ptr] = source_mono;

                let az = self.target_position.azimuth_deg * PI / 180.0;
                // Maximum head delay ~0.65ms -> ~29 samples at 44.1kHz
                let max_itd_samples = self.sample_rate * 0.00065;
                let itd_l = (az.sin().max(0.0) * max_itd_samples) as usize;
                let itd_r = ((-az.sin()).max(0.0) * max_itd_samples) as usize;

                let read_l = (self.delay_write_ptr + 256 - itd_l) % 256;
                let read_r = (self.delay_write_ptr + 256 - itd_r) % 256;

                let left_delayed = self.left_delay_buf[read_l];
                let right_delayed = self.right_delay_buf[read_r];

                self.delay_write_ptr = (self.delay_write_ptr + 1) % 256;

                // Head shadow filter (one-pole lowpass for far ear)
                self.left_shadow_filter += (left_delayed - self.left_shadow_filter) * 0.4;
                self.right_shadow_filter += (right_delayed - self.right_shadow_filter) * 0.4;

                let out_l =
                    (left_delayed * 0.6 + self.left_shadow_filter * 0.4) * self.current_gains[0];
                let out_r =
                    (right_delayed * 0.6 + self.right_shadow_filter * 0.4) * self.current_gains[1];

                outputs[0][i] = out_l;
                if outputs.len() > 1 {
                    outputs[1][i] = out_r;
                }
            } else {
                for (ch, out_ch) in outputs.iter_mut().enumerate().take(num_out_channels) {
                    if (self.format == SpatialOutputFormat::Surround5_1
                        || self.format == SpatialOutputFormat::Surround7_1
                        || self.format == SpatialOutputFormat::DolbyAtmos7_1_4)
                        && ch == 3
                    {
                        // LFE subwoofer channel
                        out_ch[i] = self.lfe_filter_state * self.current_gains[ch];
                    } else {
                        out_ch[i] = source_mono * self.current_gains[ch];
                    }
                }
            }
        }
    }
}

impl AudioNode for MultichannelSpatialPannerNode {
    fn name(&self) -> &str {
        &self.name
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
    fn test_coordinates_cartesian_conversion() {
        // Front at 2m (azimuth = 0, elevation = 0, dist = 2)
        let coord_front = SpatialCoordinates3D::new(0.0, 0.0, 2.0);
        let (x, y, z) = coord_front.to_cartesian();
        assert!((x - 0.0).abs() < 1e-4);
        assert!((y - 2.0).abs() < 1e-4);
        assert!((z - 0.0).abs() < 1e-4);

        // Zenith overhead at 1m (azimuth = 0, elevation = 90, dist = 1)
        let coord_zenith = SpatialCoordinates3D::new(0.0, 90.0, 1.0);
        let (xz, yz, zz) = coord_zenith.to_cartesian();
        assert!((xz - 0.0).abs() < 1e-4);
        assert!((yz - 0.0).abs() < 1e-4);
        assert!((zz - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_surround_5_1_panning_center_and_rears() {
        let mut panner =
            MultichannelSpatialPannerNode::new(SpatialOutputFormat::Surround5_1, 48000.0);

        // 1. Position front-center (azimuth = 0.0)
        panner.set_position(0.0, 0.0, 1.0);
        // Center channel (index 2) should receive dominant energy
        assert!(panner.target_gains[2] > panner.target_gains[0]); // Center > Left
        assert!(panner.target_gains[2] > panner.target_gains[1]); // Center > Right

        // 2. Position hard left (azimuth = -90.0)
        panner.set_position(-90.0, 0.0, 1.0);
        assert!(panner.target_gains[0] > 0.8); // Left is high
        assert!(panner.target_gains[1] < 0.1); // Right is silent

        // 3. Position rear right surround (azimuth = 135.0)
        panner.set_position(135.0, 0.0, 1.0);
        assert!(panner.target_gains[5] > 0.5); // Rs is high
    }

    #[test]
    fn test_dolby_atmos_7_1_4_height_speakers() {
        let mut panner =
            MultichannelSpatialPannerNode::new(SpatialOutputFormat::DolbyAtmos7_1_4, 48000.0);

        // Floor level (elevation = 0)
        panner.set_position(45.0, 0.0, 1.0);
        // Height channels [8..12] should have zero/negligible gain
        for h in 8..12 {
            assert!(panner.target_gains[h] < 0.05);
        }

        // Elevated to ceiling (elevation = 80 degrees)
        panner.set_position(45.0, 80.0, 1.0);
        // Top front right speaker (index 9) should receive dominant energy
        assert!(panner.target_gains[9] > 0.7);
    }

    #[test]
    fn test_ambisonics_b_format_spherical_harmonics() {
        let mut panner =
            MultichannelSpatialPannerNode::new(SpatialOutputFormat::AmbisonicsBFormat, 48000.0);

        // Front source (azimuth = 0, elevation = 0)
        panner.set_position(0.0, 0.0, 1.0);
        // W (index 0) = 0.707, X (front, index 1) = 1.0, Y (left, index 2) = 0.0, Z (up, index 3) = 0.0
        assert!((panner.target_gains[0] - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-3);
        assert!((panner.target_gains[1] - 1.0).abs() < 1e-3);
        assert!(panner.target_gains[2].abs() < 1e-3);
        assert!(panner.target_gains[3].abs() < 1e-3);
    }

    #[test]
    fn test_multichannel_panner_process_block_execution() {
        let mut panner =
            MultichannelSpatialPannerNode::new(SpatialOutputFormat::Surround5_1, 48000.0);
        panner.set_position(-45.0, 0.0, 1.0);

        let in_buf = vec![0.8f32; 64];
        let mut out0 = vec![0.0f32; 64];
        let mut out1 = vec![0.0f32; 64];
        let mut out2 = vec![0.0f32; 64];
        let mut out3 = vec![0.0f32; 64];
        let mut out4 = vec![0.0f32; 64];
        let mut out5 = vec![0.0f32; 64];

        let inputs = [&in_buf[..]];
        let mut outputs = [
            &mut out0[..],
            &mut out1[..],
            &mut out2[..],
            &mut out3[..],
            &mut out4[..],
            &mut out5[..],
        ];

        let ctx = ProcessContext {
            sample_rate: 48000,
            bpm: 120.0,
            frame_position: 0,
            param_bus: None,
            tuning_root_hz: 440.0,
            tuning_edo_divisions: 12,
            is_playing: true,
        };

        panner.process(&inputs, &mut outputs, &ctx);
        // Left channel should have rendered audio
        assert!(outputs[0][32].abs() > 0.1);
    }
}
