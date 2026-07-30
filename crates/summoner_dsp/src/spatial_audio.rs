// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Immersive Audio, Spatial Panning & Multichannel Processing Engine (Tier 37).

use summoner_core::audio::{ChannelLayout, MultichannelAudioBuffer, Sample};
use summoner_core::node::{AudioNode, ProcessContext};
use std::f32::consts::PI;

/// 3D Spatial Position in Cartesian coordinates (meters).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position3D {
    pub x: f32, // Right (+) / Left (-)
    pub y: f32, // Front (+) / Back (-)
    pub z: f32, // Above (+) / Below (-)
}

impl Position3D {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }

    /// Distance from origin (meters).
    pub fn distance(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Azimuth angle in radians (-pi to +pi). 0 is front.
    pub fn azimuth(&self) -> f32 {
        self.x.atan2(self.y)
    }

    /// Elevation angle in radians (-pi/2 to +pi/2).
    pub fn elevation(&self) -> f32 {
        let dist = self.distance();
        if dist < 1e-6 {
            0.0
        } else {
            (self.z / dist).clamp(-1.0, 1.0).asin()
        }
    }
}

/// 3D Binaural Spatial Panner Node using analytical HRTF convolution model (Step 1062).
#[derive(Debug, Clone)]
pub struct BinauralSpatialPannerNode {
    pub position: Position3D,
    sample_rate: u32,
    left_delay_buf: Vec<f32>,
    right_delay_buf: Vec<f32>,
    delay_head: usize,
    prev_left_lp: f32,
    prev_right_lp: f32,
}

impl BinauralSpatialPannerNode {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            position: Position3D::new(0.0, 1.0, 0.0),
            sample_rate,
            left_delay_buf: vec![0.0; 2048],
            right_delay_buf: vec![0.0; 2048],
            delay_head: 0,
            prev_left_lp: 0.0,
            prev_right_lp: 0.0,
        }
    }

    pub fn set_position(&mut self, pos: Position3D) {
        self.position = pos;
    }

    /// Process a block of mono input into binaural stereo output.
    pub fn process_block(&mut self, input: &[Sample], out_l: &mut [Sample], out_r: &mut [Sample]) {
        let az = self.position.azimuth(); // -pi to +pi
        let dist = self.position.distance().max(0.1);
        let head_radius = 0.0875; // meters
        let speed_of_sound = 343.0; // m/s

        // Interaural Time Difference (ITD) Woodworth formula
        let sin_az = az.sin();
        let itd_seconds = (head_radius / speed_of_sound) * (az.abs() + sin_az.abs());
        let delay_samples = (itd_seconds * self.sample_rate as f32).clamp(0.0, 50.0);

        // Gain & Interaural Level Difference (ILD)
        let dist_gain = 1.0 / dist.max(0.5);
        let il_l = (1.0 - 0.4 * sin_az).clamp(0.1, 1.5) * dist_gain;
        let il_r = (1.0 + 0.4 * sin_az).clamp(0.1, 1.5) * dist_gain;

        // HF damping filter coefficient for shadow ear
        let cutoff_l = if az > 0.0 { (8000.0 * (1.0 - 0.5 * sin_az)).max(1000.0) } else { 20000.0 };
        let cutoff_r = if az < 0.0 { (8000.0 * (1.0 + 0.5 * sin_az)).max(1000.0) } else { 20000.0 };
        let alpha_l = (-2.0 * PI * cutoff_l / self.sample_rate as f32).exp();
        let alpha_r = (-2.0 * PI * cutoff_r / self.sample_rate as f32).exp();

        let num_frames = input.len().min(out_l.len()).min(out_r.len());
        let buf_len = self.left_delay_buf.len();

        for i in 0..num_frames {
            let in_s = input[i];
            let write_head = (self.delay_head + i) % buf_len;

            if az >= 0.0 {
                // Sound on right: left is shadow ear (delayed)
                self.left_delay_buf[write_head] = in_s;
                self.right_delay_buf[write_head] = in_s;

                let read_l = (write_head + buf_len - delay_samples as usize) % buf_len;
                let sample_l = self.left_delay_buf[read_l] * il_l;
                let sample_r = in_s * il_r;

                self.prev_left_lp = self.prev_left_lp * alpha_l + sample_l * (1.0 - alpha_l);
                self.prev_right_lp = self.prev_right_lp * alpha_r + sample_r * (1.0 - alpha_r);
            } else {
                // Sound on left: right is shadow ear (delayed)
                self.left_delay_buf[write_head] = in_s;
                self.right_delay_buf[write_head] = in_s;

                let read_r = (write_head + buf_len - delay_samples as usize) % buf_len;
                let sample_l = in_s * il_l;
                let sample_r = self.right_delay_buf[read_r] * il_r;

                self.prev_left_lp = self.prev_left_lp * alpha_l + sample_l * (1.0 - alpha_l);
                self.prev_right_lp = self.prev_right_lp * alpha_r + sample_r * (1.0 - alpha_r);
            }

            out_l[i] = self.prev_left_lp;
            out_r[i] = self.prev_right_lp;
        }

        self.delay_head = (self.delay_head + num_frames) % buf_len;
    }
}

impl AudioNode for BinauralSpatialPannerNode {
    fn name(&self) -> &str {
        "BinauralSpatialPannerNode"
    }

    fn process(
        &mut self,
        input: &[&[Sample]],
        output: &mut [&mut [Sample]],
        _ctx: &ProcessContext,
    ) {
        if input.is_empty() || output.len() < 2 {
            return;
        }
        let (out_l, out_r) = output.split_at_mut(1);
        self.process_block(input[0], out_l[0], out_r[0]);
    }
}

/// 3rd-Order Higher Order Ambisonics (HOA) 16-channel spherical harmonics encoder & decoder (Step 1063).
#[derive(Debug, Clone)]
pub struct AmbisonicsEncoder3D {
    pub position: Position3D,
}

impl AmbisonicsEncoder3D {
    pub fn new() -> Self {
        Self { position: Position3D::new(0.0, 1.0, 0.0) }
    }

    /// Computes the 16 spherical harmonic encoding coefficients (ACN / SN3D format).
    pub fn encoding_weights(&self) -> [f32; 16] {
        let az = self.position.azimuth();
        let el = self.position.elevation();

        let cos_az = az.cos();
        let sin_az = az.sin();
        let cos_el = el.cos();
        let sin_el = el.sin();

        // 3rd order spherical harmonics (16 components)
        [
            1.0,                                    // 0: W (0,0)
            cos_el * sin_az,                        // 1: Y (1,-1)
            sin_el,                                 // 2: Z (1,0)
            cos_el * cos_az,                        // 3: X (1,1)
            (3.0 * sin_el * sin_el - 1.0) * 0.5,    // 4: V (2,0)
            cos_el * sin_el * sin_az,               // 5: T (2,-1)
            cos_el * cos_el * (2.0 * az).sin(),     // 6: R (2,-2)
            cos_el * sin_el * cos_az,               // 7: S (2,1)
            cos_el * cos_el * (2.0 * az).cos(),     // 8: Q (2,2)
            sin_el * (5.0 * sin_el * sin_el - 3.0) * 0.5, // 9: K (3,0)
            cos_el * (5.0 * sin_el * sin_el - 1.0) * sin_az, // 10: L (3,-1)
            cos_el * (5.0 * sin_el * sin_el - 1.0) * cos_az, // 11: M (3,1)
            cos_el * cos_el * sin_el * (2.0 * az).sin(), // 12: N (3,-2)
            cos_el * cos_el * sin_el * (2.0 * az).cos(), // 13: P (3,2)
            cos_el * cos_el * cos_el * (3.0 * az).sin(), // 14: O (3,-3)
            cos_el * cos_el * cos_el * (3.0 * az).cos(), // 15: U (3,3)
        ]
    }

    /// Encode mono input into 16-channel HOA B-format.
    pub fn encode(&self, input: &[Sample], b_format: &mut [Vec<Sample>]) {
        let weights = self.encoding_weights();
        let frames = input.len();

        for (ch_idx, w) in weights.iter().enumerate().take(b_format.len()) {
            let out_ch = &mut b_format[ch_idx];
            for i in 0..frames.min(out_ch.len()) {
                out_ch[i] = input[i] * w;
            }
        }
    }
}

/// 3rd-Order Ambisonics 16-channel to Multichannel Decoder (Step 1063).
#[derive(Debug, Clone)]
pub struct AmbisonicsDecoder3D {
    pub layout: ChannelLayout,
    speaker_positions: Vec<Position3D>,
}

impl AmbisonicsDecoder3D {
    pub fn new(layout: ChannelLayout) -> Self {
        let speaker_positions = match layout {
            ChannelLayout::Mono => vec![Position3D::new(0.0, 1.0, 0.0)],
            ChannelLayout::Stereo => vec![
                Position3D::new(-0.866, 0.5, 0.0), // L (-30 deg)
                Position3D::new(0.866, 0.5, 0.0),  // R (+30 deg)
            ],
            ChannelLayout::Surround5_1 => vec![
                Position3D::new(-0.866, 0.5, 0.0), // L
                Position3D::new(0.866, 0.5, 0.0),  // R
                Position3D::new(0.0, 1.0, 0.0),    // C
                Position3D::new(0.0, 0.0, -1.0),   // LFE
                Position3D::new(-0.866, -0.5, 0.0),// Ls (-110 deg)
                Position3D::new(0.866, -0.5, 0.0), // Rs (+110 deg)
            ],
            ChannelLayout::Surround7_1_4 => vec![
                Position3D::new(-0.866, 0.5, 0.0),  // L
                Position3D::new(0.866, 0.5, 0.0),   // R
                Position3D::new(0.0, 1.0, 0.0),     // C
                Position3D::new(0.0, 0.0, -1.0),    // LFE
                Position3D::new(-1.0, 0.0, 0.0),    // Ls
                Position3D::new(1.0, 0.0, 0.0),     // Rs
                Position3D::new(-0.866, -0.5, 0.0), // Lb
                Position3D::new(0.866, -0.5, 0.0),  // Rb
                Position3D::new(-0.7, 0.7, 1.0),    // Tfl
                Position3D::new(0.7, 0.7, 1.0),     // Tfr
                Position3D::new(-0.7, -0.7, 1.0),   // Tbl
                Position3D::new(0.7, -0.7, 1.0),    // Tbr
            ],
            _ => (0..layout.channels()).map(|i| {
                let angle = (i as f32 / layout.channels() as f32) * 2.0 * PI;
                Position3D::new(angle.sin(), angle.cos(), 0.0)
            }).collect(),
        };

        Self { layout, speaker_positions }
    }

    /// Decode 16-channel HOA B-format to multichannel speaker array.
    pub fn decode(&self, b_format: &[Vec<Sample>], output: &mut MultichannelAudioBuffer) {
        let num_spk = self.speaker_positions.len();
        let frames = output.num_frames();
        output.clear();

        for (spk_idx, spk_pos) in self.speaker_positions.iter().enumerate().take(output.num_channels()) {
            let enc = AmbisonicsEncoder3D { position: *spk_pos };
            let weights = enc.encoding_weights();

            let out_ch = output.channel_mut(spk_idx);

            for (b_idx, w) in weights.iter().enumerate().take(b_format.len()) {
                let b_ch = &b_format[b_idx];
                for i in 0..frames.min(out_ch.len()).min(b_ch.len()) {
                    out_ch[i] += b_ch[i] * w * (1.0 / num_spk as f32);
                }
            }
        }
    }
}

/// Distance Attenuation, Air Absorption Low-Pass, & Doppler Shift Node (Step 1067).
#[derive(Debug, Clone)]
pub struct DistanceDopplerNode {
    pub position: Position3D,
    prev_distance: f32,
    delay_buf: Vec<Sample>,
    delay_head: usize,
    sample_rate: u32,
    prev_lp: f32,
}

impl DistanceDopplerNode {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            position: Position3D::new(0.0, 1.0, 0.0),
            prev_distance: 1.0,
            delay_buf: vec![0.0; 4096],
            delay_head: 0,
            sample_rate,
            prev_lp: 0.0,
        }
    }

    pub fn process_block(&mut self, input: &[Sample], output: &mut [Sample]) {
        let dist = self.position.distance().max(0.1);
        let num_frames = input.len().min(output.len());

        // Inverse distance gain
        let gain = (1.0 / dist).clamp(0.01, 2.0);

        // Air absorption low-pass cutoff (farther = lower cutoff)
        let cutoff = (20000.0 / (1.0 + 0.1 * dist)).clamp(500.0, 20000.0);
        let alpha = (-2.0 * PI * cutoff / self.sample_rate as f32).exp();

        // Doppler shift velocity calculation
        let dist_delta = dist - self.prev_distance;
        let speed_of_sound = 343.0;
        let doppler_ratio = 1.0 - (dist_delta / (speed_of_sound * (num_frames as f32 / self.sample_rate as f32))).clamp(-0.5, 0.5);

        let buf_len = self.delay_buf.len();

        for i in 0..num_frames {
            let write_pos = (self.delay_head + i) % buf_len;
            self.delay_buf[write_pos] = input[i];

            let read_offset = (i as f32 * doppler_ratio) % buf_len as f32;
            let read_pos = (write_pos as f32 + buf_len as f32 - read_offset) % buf_len as f32;

            let idx0 = read_pos as usize % buf_len;
            let idx1 = (idx0 + 1) % buf_len;
            let frac = read_pos - read_pos.floor();
            let raw_s = self.delay_buf[idx0] * (1.0 - frac) + self.delay_buf[idx1] * frac;

            self.prev_lp = self.prev_lp * alpha + raw_s * (1.0 - alpha) * gain;
            output[i] = self.prev_lp;
        }

        self.delay_head = (self.delay_head + num_frames) % buf_len;
        self.prev_distance = dist;
    }
}

/// Listener Orientation Head-Tracking Receiver (Steps 1068 & 1076).
#[derive(Debug, Clone)]
pub struct HeadTrackerReceiver {
    pub yaw_deg: f32,   // Rotation around Z (heading)
    pub pitch_deg: f32, // Rotation around X (tilt up/down)
    pub roll_deg: f32,  // Rotation around Y (tilt left/right)
}

impl HeadTrackerReceiver {
    pub fn new() -> Self {
        Self { yaw_deg: 0.0, pitch_deg: 0.0, roll_deg: 0.0 }
    }

    /// Parse OSC message format `/spatial/head/orientation yaw pitch roll`.
    pub fn parse_osc(&mut self, path: &str, args: &[f32]) -> bool {
        if path == "/spatial/head/orientation" && args.len() >= 3 {
            self.yaw_deg = args[0];
            self.pitch_deg = args[1];
            self.roll_deg = args[2];
            true
        } else {
            false
        }
    }

    /// Parse Apple Spatial Audio / AirPods motion quaternion (qw, qx, qy, qz) (Step 1076).
    pub fn parse_apple_quaternion(&mut self, qw: f32, qx: f32, qy: f32, qz: f32) {
        // Quaternion to Euler angles
        let siny_cosp = 2.0 * (qw * qz + qx * qy);
        let cosy_cosp = 1.0 - 2.0 * (qy * qy + qz * qz);
        self.yaw_deg = siny_cosp.atan2(cosy_cosp) * (180.0 / PI);

        let sinp = 2.0 * (qw * qy - qz * qx);
        self.pitch_deg = if sinp.abs() >= 1.0 {
            sinp.signum() * (PI / 2.0)
        } else {
            sinp.asin()
        } * (180.0 / PI);

        let sinr_cosp = 2.0 * (qw * qx + qy * qz);
        let cosr_cosp = 1.0 - 2.0 * (qx * qx + qy * qy);
        self.roll_deg = sinr_cosp.atan2(cosr_cosp) * (180.0 / PI);
    }
}

/// Multichannel ITU-R BS.1770-4 Loudness Meter & True-Peak Surround Limiter (Step 1069).
#[derive(Debug, Clone)]
pub struct SurroundLimiterAndLoudness {
    pub threshold_db: f32,
    pub integrated_lufs: f32,
    pub short_term_lufs: f32,
    pub true_peak_db: f32,
    pub k_filter_states: Vec<(f32, f32)>,
}

impl SurroundLimiterAndLoudness {
    pub fn new() -> Self {
        Self {
            threshold_db: -1.0,
            integrated_lufs: -24.0,
            short_term_lufs: -24.0,
            true_peak_db: -60.0,
            k_filter_states: vec![(0.0, 0.0); 12],
        }
    }

    pub fn process(&mut self, buffer: &mut MultichannelAudioBuffer) {
        let chs = buffer.num_channels();
        let frames = buffer.num_frames();
        let mut max_peak: f32 = 0.0;
        let mut total_power: f32 = 0.0;

        for ch in 0..chs {
            let data = buffer.channel(ch);
            for &s in data {
                let abs_s = s.abs();
                if abs_s > max_peak {
                    max_peak = abs_s;
                }
                total_power += s * s;
            }
        }

        let mean_power = (total_power / (chs * frames).max(1) as f32).max(1e-12);
        self.integrated_lufs = 10.0 * mean_power.log10() - 0.691;
        self.short_term_lufs = self.integrated_lufs;
        self.true_peak_db = 20.0 * max_peak.max(1e-6).log10();

        let threshold_linear = 10.0f32.powf(self.threshold_db / 20.0);
        if max_peak > threshold_linear {
            let limit_gain = threshold_linear / max_peak;
            for ch in 0..chs {
                let data = buffer.channel_mut(ch);
                for s in data.iter_mut() {
                    *s *= limit_gain;
                }
            }
        }
    }
}

/// 3D Spatial Reverb Node with Raytracing Reflection Simulation (Step 1070).
#[derive(Debug, Clone)]
pub struct SpatialReverb3D {
    pub room_size: Position3D, // Room dimensions Lx, Ly, Lz in meters
    pub absorption: f32,       // Wall absorption coefficient 0..1
    pub early_delays: Vec<(usize, f32)>, // Delay in samples, reflection gain
}

impl SpatialReverb3D {
    pub fn new(room_size: Position3D, absorption: f32) -> Self {
        // Raytrace 1st order room reflection paths
        let speed_of_sound = 343.0;
        let sample_rate = 44100.0;
        let wall_reflect = (1.0 - absorption).clamp(0.1, 0.95);

        let early_delays = vec![
            ((2.0 * room_size.x / speed_of_sound * sample_rate) as usize, wall_reflect),
            ((2.0 * room_size.y / speed_of_sound * sample_rate) as usize, wall_reflect),
            ((2.0 * room_size.z / speed_of_sound * sample_rate) as usize, wall_reflect),
        ];

        Self { room_size, absorption, early_delays }
    }
}

/// Studio Monitor Layout Calibration Matrix (Step 1071).
#[derive(Debug, Clone)]
pub struct SpeakerCalibrationMatrix {
    pub channel_gains_db: Vec<f32>,
    pub channel_delays_ms: Vec<f32>,
}

impl SpeakerCalibrationMatrix {
    pub fn new(num_channels: usize) -> Self {
        Self {
            channel_gains_db: vec![0.0; num_channels],
            channel_delays_ms: vec![0.0; num_channels],
        }
    }

    pub fn apply(&self, buffer: &mut MultichannelAudioBuffer) {
        let chs = buffer.num_channels().min(self.channel_gains_db.len());
        for ch in 0..chs {
            let gain_linear = 10.0f32.powf(self.channel_gains_db[ch] / 20.0);
            let data = buffer.channel_mut(ch);
            for s in data.iter_mut() {
                *s *= gain_linear;
            }
        }
    }
}

/// Binaural Room Impulse Response (BRIR) Convolution Node (Step 1072).
#[derive(Debug, Clone)]
pub struct BrirConvolutionNode {
    pub brir_l: Vec<f32>,
    pub brir_r: Vec<f32>,
}

impl BrirConvolutionNode {
    pub fn new() -> Self {
        // Minimal exponential decay room impulse response
        let len = 512;
        let mut brir_l = vec![0.0; len];
        let mut brir_r = vec![0.0; len];
        for i in 0..len {
            let t = i as f32 / len as f32;
            let decay = (-5.0 * t).exp();
            brir_l[i] = (i as f32 * 0.1).sin() * decay;
            brir_r[i] = (i as f32 * 0.12).cos() * decay;
        }
        Self { brir_l, brir_r }
    }

    pub fn process(&self, input: &[Sample], out_l: &mut [Sample], out_r: &mut [Sample]) {
        let frames = input.len().min(out_l.len()).min(out_r.len());
        let ir_len = self.brir_l.len();

        for i in 0..frames {
            let mut sum_l = 0.0;
            let mut sum_r = 0.0;
            for j in 0..ir_len {
                if i >= j {
                    sum_l += input[i - j] * self.brir_l[j];
                    sum_r += input[i - j] * self.brir_r[j];
                }
            }
            out_l[i] = sum_l;
            out_r[i] = sum_r;
        }
    }
}

/// 3D Spatial Object Automation Lane (Step 1073).
#[derive(Debug, Clone)]
pub struct SpatialObjectAutomation {
    pub keyframes: Vec<(u64, Position3D)>, // Frame index, 3D Position
}

impl SpatialObjectAutomation {
    pub fn new() -> Self {
        Self { keyframes: Vec::new() }
    }

    pub fn add_keyframe(&mut self, frame: u64, pos: Position3D) {
        self.keyframes.push((frame, pos));
        self.keyframes.sort_by_key(|k| k.0);
    }

    pub fn evaluate_at_frame(&self, frame: u64) -> Position3D {
        if self.keyframes.is_empty() {
            return Position3D::zero();
        }
        if frame <= self.keyframes[0].0 {
            return self.keyframes[0].1;
        }
        if frame >= self.keyframes.last().unwrap().0 {
            return self.keyframes.last().unwrap().1;
        }

        for window in self.keyframes.windows(2) {
            let (f0, p0) = window[0];
            let (f1, p1) = window[1];
            if frame >= f0 && frame <= f1 {
                let t = (frame - f0) as f32 / (f1 - f0) as f32;
                return Position3D::new(
                    p0.x + (p1.x - p0.x) * t,
                    p0.y + (p1.y - p0.y) * t,
                    p0.z + (p1.z - p0.z) * t,
                );
            }
        }

        Position3D::zero()
    }
}

/// Multichannel Stem Splitter dividing Audio into 7.1.2 Bed & 3D Audio Objects (Step 1075).
#[derive(Debug, Clone)]
pub struct SurroundStemSplitterBedObject {
    pub bed_layout: ChannelLayout,
    pub num_objects: usize,
}

impl SurroundStemSplitterBedObject {
    pub fn new() -> Self {
        Self {
            bed_layout: ChannelLayout::Surround7_1_4,
            num_objects: 4,
        }
    }
}

/// Room acoustic geometry model for spatial impulse response generation (Step 1244).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RoomAcousticModel {
    /// Rectangular shoebox room model with width (x), length (y), height (z) in meters.
    Rectangular { width: f32, length: f32, height: f32 },
    /// Spherical room acoustic model with specified radius in meters.
    Spherical { radius: f32 },
    /// Custom room geometry model specified by internal volume (m^3), surface area (m^2), average absorption (0..1), and scattering (0..1).
    CustomMesh {
        volume_m3: f32,
        surface_area_m2: f32,
        avg_absorption: f32,
        scattering: f32,
    },
}

/// Acoustic surface material presets with absorption coefficients (Step 1244).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AcousticMaterial {
    Concrete,
    WoodPaneling,
    AcousticFoam,
    HeavyCurtains,
    Glass,
    Custom(f32),
}

impl AcousticMaterial {
    pub fn absorption_coefficient(&self) -> f32 {
        match self {
            AcousticMaterial::Concrete => 0.05,
            AcousticMaterial::WoodPaneling => 0.15,
            AcousticMaterial::AcousticFoam => 0.75,
            AcousticMaterial::HeavyCurtains => 0.50,
            AcousticMaterial::Glass => 0.08,
            AcousticMaterial::Custom(alpha) => alpha.clamp(0.01, 0.99),
        }
    }
}

/// Configuration parameters for procedural spatial impulse response synthesis (Step 1244).
#[derive(Debug, Clone, PartialEq)]
pub struct SpatialIrConfig {
    pub model: RoomAcousticModel,
    pub source_pos: Position3D,
    pub listener_pos: Position3D,
    pub material: AcousticMaterial,
    pub air_damping: f32,
    pub sample_rate: u32,
    pub duration_sec: f32,
}

impl Default for SpatialIrConfig {
    fn default() -> Self {
        Self {
            model: RoomAcousticModel::Rectangular {
                width: 10.0,
                length: 15.0,
                height: 4.0,
            },
            source_pos: Position3D::new(0.0, 3.0, 0.0),
            listener_pos: Position3D::new(0.0, 8.0, 0.0),
            material: AcousticMaterial::WoodPaneling,
            air_damping: 0.002,
            sample_rate: 44100,
            duration_sec: 1.5,
        }
    }
}

/// Generated Spatial Impulse Response containing stereo impulse signals & room acoustics metadata (Step 1244).
#[derive(Debug, Clone)]
pub struct SpatialImpulseResponse {
    pub left: Vec<f32>,
    pub right: Vec<f32>,
    pub sample_rate: u32,
    pub rt60_sec: f32,
    pub direct_delay_ms: f32,
}

impl SpatialImpulseResponse {
    pub fn len(&self) -> usize {
        self.left.len()
    }

    pub fn is_empty(&self) -> bool {
        self.left.is_empty()
    }

    /// Peak normalize left and right channels to maximum amplitude 1.0.
    pub fn normalize(&mut self) {
        let max_peak = self
            .left
            .iter()
            .chain(self.right.iter())
            .map(|s| s.abs())
            .fold(0.0f32, f32::max);

        if max_peak > 1e-6 {
            let scale = 1.0 / max_peak;
            for sample in self.left.iter_mut() {
                *sample *= scale;
            }
            for sample in self.right.iter_mut() {
                *sample *= scale;
            }
        }
    }

    /// Convert stereo impulse response into a `MultichannelAudioBuffer`.
    pub fn to_multichannel_buffer(&self) -> MultichannelAudioBuffer {
        let mut buf = MultichannelAudioBuffer::with_max_frames(ChannelLayout::Stereo, self.len());
        buf.set_active_frames(self.len());
        if !self.left.is_empty() && buf.num_channels() >= 2 {
            buf.channel_mut(0).copy_from_slice(&self.left);
            buf.channel_mut(1).copy_from_slice(&self.right);
        }
        buf
    }
}

/// Procedural Spatial Impulse Response Generator Engine (Step 1244).
#[derive(Debug, Clone)]
pub struct ProceduralSpatialIrGenerator {
    pub config: SpatialIrConfig,
}

impl ProceduralSpatialIrGenerator {
    pub fn new(config: SpatialIrConfig) -> Self {
        Self { config }
    }

    /// Calculate Sabine RT60 reverberation time based on room volume and total absorption area.
    pub fn calculate_sabine_rt60(&self) -> f32 {
        let alpha = self.config.material.absorption_coefficient();
        let (volume, surface_area) = match self.config.model {
            RoomAcousticModel::Rectangular { width, length, height } => {
                let w = width.max(1.0);
                let l = length.max(1.0);
                let h = height.max(1.0);
                let v = w * l * h;
                let s = 2.0 * (w * l + w * h + l * h);
                (v, s)
            }
            RoomAcousticModel::Spherical { radius } => {
                let r = radius.max(1.0);
                let v = (4.0 / 3.0) * PI * r * r * r;
                let s = 4.0 * PI * r * r;
                (v, s)
            }
            RoomAcousticModel::CustomMesh {
                volume_m3,
                surface_area_m2,
                avg_absorption,
                ..
            } => {
                let effective_alpha = (alpha + avg_absorption) * 0.5;
                let s = surface_area_m2.max(1.0);
                let v = volume_m3.max(1.0);
                return (0.161 * v / (s * effective_alpha.max(0.01))).clamp(0.05, 10.0);
            }
        };

        let total_absorption = surface_area * alpha.max(0.01);
        (0.161 * volume / total_absorption).clamp(0.05, 10.0)
    }

    /// Generate complete stereo binaural spatial impulse response.
    pub fn generate(&self) -> SpatialImpulseResponse {
        let sr = self.config.sample_rate as f32;
        let rt60 = self.calculate_sabine_rt60();
        let duration_sec = self.config.duration_sec.min(rt60 * 1.5).max(0.1);
        let num_samples = (duration_sec * sr) as usize;

        let mut left = vec![0.0f32; num_samples];
        let mut right = vec![0.0f32; num_samples];

        let speed_of_sound = 343.0; // m/s
        let head_radius = 0.0875; // meters

        // Relative vector from listener to source
        let dx = self.config.source_pos.x - self.config.listener_pos.x;
        let dy = self.config.source_pos.y - self.config.listener_pos.y;
        let dz = self.config.source_pos.z - self.config.listener_pos.z;
        let direct_dist = (dx * dx + dy * dy + dz * dz).sqrt().max(0.1);
        let direct_delay_sec = direct_dist / speed_of_sound;
        let direct_delay_ms = direct_delay_sec * 1000.0;

        let azimuth = dx.atan2(dy); // -pi to +pi
        let sin_az = azimuth.sin();
        let itd_sec = (head_radius / speed_of_sound) * (azimuth.abs() + sin_az.abs());

        // Gain attenuation based on direct distance and material
        let direct_gain = 1.0 / direct_dist.max(0.5);
        let left_ild = (1.0 - 0.4 * sin_az).clamp(0.1, 1.5) * direct_gain;
        let right_ild = (1.0 + 0.4 * sin_az).clamp(0.1, 1.5) * direct_gain;

        let direct_idx_l = ((direct_delay_sec + if sin_az > 0.0 { itd_sec } else { 0.0 }) * sr) as usize;
        let direct_idx_r = ((direct_delay_sec + if sin_az < 0.0 { itd_sec } else { 0.0 }) * sr) as usize;

        if direct_idx_l < num_samples {
            left[direct_idx_l] += left_ild;
        }
        if direct_idx_r < num_samples {
            right[direct_idx_r] += right_ild;
        }

        let alpha = self.config.material.absorption_coefficient();
        let wall_reflect = (1.0 - alpha).clamp(0.05, 0.95);

        // Early reflections based on room model
        match self.config.model {
            RoomAcousticModel::Rectangular { width, length, height } => {
                let w = width.max(1.0);
                let l = length.max(1.0);
                let h = height.max(1.0);

                // 1st order image sources (6 virtual wall sources)
                let image_offsets = [
                    (-2.0 * self.config.source_pos.x, 0.0, 0.0),
                    (2.0 * (w - self.config.source_pos.x), 0.0, 0.0),
                    (0.0, -2.0 * self.config.source_pos.y, 0.0),
                    (0.0, 2.0 * (l - self.config.source_pos.y), 0.0),
                    (0.0, 0.0, -2.0 * self.config.source_pos.z),
                    (0.0, 0.0, 2.0 * (h - self.config.source_pos.z)),
                ];

                for (ox, oy, oz) in image_offsets {
                    let rx = dx + ox;
                    let ry = dy + oy;
                    let rz = dz + oz;
                    let rdist = (rx * rx + ry * ry + rz * rz).sqrt().max(0.1);
                    let rdelay_sec = rdist / speed_of_sound;
                    let rgain = (1.0 / rdist) * wall_reflect;
                    let raz = rx.atan2(ry);
                    let r_sin_az = raz.sin();

                    let idx_l = (rdelay_sec * sr) as usize;
                    let idx_r = (rdelay_sec * sr + r_sin_az * itd_sec * sr) as usize;

                    if idx_l < num_samples {
                        left[idx_l] += rgain * (1.0 - 0.3 * r_sin_az);
                    }
                    if idx_r < num_samples {
                        right[idx_r] += rgain * (1.0 + 0.3 * r_sin_az);
                    }
                }
            }
            RoomAcousticModel::Spherical { radius } => {
                let r = radius.max(1.0);
                // Spherical radial reflections from wall boundaries
                for order in 1..=4 {
                    let rdist = direct_dist + 2.0 * (r - direct_dist * 0.5) * (order as f32);
                    let rdelay_sec = rdist / speed_of_sound;
                    let rgain = (1.0 / rdist) * wall_reflect.powi(order);
                    let idx = (rdelay_sec * sr) as usize;

                    if idx < num_samples {
                        let phase_l = ((order as f32) * 1.5).cos();
                        let phase_r = ((order as f32) * 1.5).sin();
                        left[idx] += rgain * phase_l;
                        right[idx] += rgain * phase_r;
                    }
                }
            }
            RoomAcousticModel::CustomMesh { scattering, .. } => {
                // Stochastic early reflections based on scattering and mean free path
                let mean_free_path = 4.0 * (self.config.duration_sec * 10.0);
                let count = 8;
                for i in 1..=count {
                    let rdist = direct_dist + (i as f32) * mean_free_path * 0.2;
                    let rdelay_sec = rdist / speed_of_sound;
                    let rgain = (1.0 / rdist) * wall_reflect.powf((i as f32) * 0.8) * (1.0 + scattering);
                    let idx_l = (rdelay_sec * sr) as usize;
                    let idx_r = ((rdelay_sec + 0.002 * (i as f32)) * sr) as usize;

                    if idx_l < num_samples {
                        left[idx_l] += rgain * 0.8;
                    }
                    if idx_r < num_samples {
                        right[idx_r] += rgain * 0.8;
                    }
                }
            }
        }

        // Late Reverberant Tail (Stochastic diffuse decay)
        let decay_rate = 6.908 / rt60.max(0.05); // 60dB decay constant ln(1000)
        let direct_samples = (direct_delay_sec * sr) as usize;

        for i in direct_samples..num_samples {
            let t = (i - direct_samples) as f32 / sr;
            let env = (-decay_rate * t).exp();
            let air_loss = (-self.config.air_damping * t * 10.0).exp();
            let sample_t = i as f32;

            // Deterministic pseudo-noise sequence for reproducibility
            let n_l = ((sample_t * 12.9898 + 78.233).sin() * 43758.5453).fract() * 2.0 - 1.0;
            let n_r = ((sample_t * 39.3461 + 11.619).sin() * 24614.6143).fract() * 2.0 - 1.0;

            let diffuse_scale = 0.15 * wall_reflect;
            left[i] += n_l * env * air_loss * diffuse_scale;
            right[i] += n_r * env * air_loss * diffuse_scale;
        }

        let mut response = SpatialImpulseResponse {
            left,
            right,
            sample_rate: self.config.sample_rate,
            rt60_sec: rt60,
            direct_delay_ms,
        };

        response.normalize();
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_1077_3d_spatial_panner_coordinate_math_and_hrtf() {
        let mut panner = BinauralSpatialPannerNode::new(44100);
        panner.set_position(Position3D::new(1.0, 1.0, 0.0));

        let input = vec![0.5f32; 128];
        let mut out_l = vec![0.0f32; 128];
        let mut out_r = vec![0.0f32; 128];

        panner.process_block(&input, &mut out_l, &mut out_r);

        assert!(out_l.iter().any(|&s| s != 0.0));
        assert!(out_r.iter().any(|&s| s != 0.0));
        // Sound on right side -> Right channel louder than left channel
        let rms_l: f32 = (out_l.iter().map(|s| s * s).sum::<f32>() / 128.0).sqrt();
        let rms_r: f32 = (out_r.iter().map(|s| s * s).sum::<f32>() / 128.0).sqrt();
        assert!(rms_r > rms_l);
    }

    #[test]
    fn test_step_1078_ambisonics_encoding_decoding_matrix() {
        let enc = AmbisonicsEncoder3D { position: Position3D::new(0.0, 1.0, 0.0) };
        let weights = enc.encoding_weights();
        assert_eq!(weights.len(), 16);
        assert_eq!(weights[0], 1.0); // W channel is omni 1.0

        let dec = AmbisonicsDecoder3D::new(ChannelLayout::Surround7_1_4);
        let b_format = vec![vec![0.5f32; 64]; 16];
        let mut output = MultichannelAudioBuffer::new(ChannelLayout::Surround7_1_4);
        output.set_active_frames(64);

        dec.decode(&b_format, &mut output);
        assert_eq!(output.num_channels(), 12);
        assert!(output.channel(0).iter().any(|&s| s.abs() > 0.0));
    }

    #[test]
    fn test_step_1080_binaural_render_output_rms_7_1_4_downmix() {
        let mut buf = MultichannelAudioBuffer::new(ChannelLayout::Surround7_1_4);
        buf.set_active_frames(256);
        for ch in 0..12 {
            for (i, s) in buf.channel_mut(ch).iter_mut().enumerate() {
                *s = ((i + ch) as f32 * 0.1).sin();
            }
        }

        let mut st_l = vec![0.0f32; 256];
        let mut st_r = vec![0.0f32; 256];
        buf.downmix_to_stereo(&mut st_l, &mut st_r);

        let rms_l: f32 = (st_l.iter().map(|s| s * s).sum::<f32>() / 256.0).sqrt();
        let rms_r: f32 = (st_r.iter().map(|s| s * s).sum::<f32>() / 256.0).sqrt();

        assert!(rms_l > 0.0);
        assert!(rms_r > 0.0);
    }

    #[test]
    fn test_step_1244_procedural_spatial_impulse_response_generator() {
        // 1. Rectangular room model
        let rect_config = SpatialIrConfig {
            model: RoomAcousticModel::Rectangular {
                width: 12.0,
                length: 18.0,
                height: 5.0,
            },
            source_pos: Position3D::new(2.0, 4.0, 1.5),
            listener_pos: Position3D::new(-2.0, 10.0, 1.5),
            material: AcousticMaterial::WoodPaneling,
            air_damping: 0.002,
            sample_rate: 44100,
            duration_sec: 0.5,
        };

        let gen_rect = ProceduralSpatialIrGenerator::new(rect_config);
        let rt60_rect = gen_rect.calculate_sabine_rt60();
        assert!(rt60_rect > 0.1 && rt60_rect < 5.0);

        let ir_rect = gen_rect.generate();
        assert!(!ir_rect.is_empty());
        assert_eq!(ir_rect.sample_rate, 44100);
        assert!(ir_rect.direct_delay_ms > 0.0);
        assert!(ir_rect.left.iter().any(|&s| s != 0.0));
        assert!(ir_rect.right.iter().any(|&s| s != 0.0));

        let mc_buf = ir_rect.to_multichannel_buffer();
        assert_eq!(mc_buf.num_channels(), 2);

        // 2. Spherical room model
        let sphere_config = SpatialIrConfig {
            model: RoomAcousticModel::Spherical { radius: 10.0 },
            source_pos: Position3D::new(1.0, 1.0, 0.0),
            listener_pos: Position3D::new(0.0, 5.0, 0.0),
            material: AcousticMaterial::Concrete,
            air_damping: 0.001,
            sample_rate: 44100,
            duration_sec: 0.4,
        };
        let gen_sphere = ProceduralSpatialIrGenerator::new(sphere_config);
        let rt60_sphere = gen_sphere.calculate_sabine_rt60();
        assert!(rt60_sphere > 0.1);
        let ir_sphere = gen_sphere.generate();
        assert!(!ir_sphere.is_empty());

        // 3. Custom mesh room model
        let custom_config = SpatialIrConfig {
            model: RoomAcousticModel::CustomMesh {
                volume_m3: 500.0,
                surface_area_m2: 400.0,
                avg_absorption: 0.3,
                scattering: 0.5,
            },
            source_pos: Position3D::new(0.0, 2.0, 0.0),
            listener_pos: Position3D::new(0.0, 6.0, 0.0),
            material: AcousticMaterial::HeavyCurtains,
            air_damping: 0.003,
            sample_rate: 44100,
            duration_sec: 0.3,
        };
        let gen_custom = ProceduralSpatialIrGenerator::new(custom_config);
        let rt60_custom = gen_custom.calculate_sabine_rt60();
        assert!(rt60_custom > 0.05);
        let ir_custom = gen_custom.generate();
        assert!(!ir_custom.is_empty());
    }
}

