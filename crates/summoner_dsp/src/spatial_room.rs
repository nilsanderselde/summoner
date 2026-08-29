// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! 3D Shoebox Early Reflection & Diffuse Reverberation Acoustic Room Modeler (Milestone 12).
//!
//! Provides geometric 3D shoebox acoustics using the Image Source Method (ISM) for first- and
//! second-order early reflections coupled with a multi-channel Feedback Delay Network (FDN)
//! for diffuse late reverberation, air absorption, and frequency-dependent boundary damping.
//!
//! Enforces zero heap allocations in real-time audio render loops under `AllocGuard`.

use crate::spatial_audio::Position3D;
use crate::traits::SignalProcessor;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use summoner_core::allocator::AllocGuard;
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};

pub const SPEED_OF_SOUND_M_S: f32 = 343.0;
pub const MAX_EARLY_REFLECTIONS: usize = 18; // 6 1st-order + 12 2nd-order
pub const EARLY_DELAY_BUFFER_SIZE: usize = 16384; // ~341ms @ 48kHz (max path ~117 meters)
pub const FDN_NUM_LINES: usize = 4;
pub const FDN_MAX_DELAY: usize = 4096;

/// Wall material presets specifying typical acoustic absorption coefficients ($\alpha$).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WallMaterial {
    Concrete,
    Wood,
    #[default]
    StudioAcoustic,
    Carpet,
    Glass,
    CathedralStone,
}

impl WallMaterial {
    pub fn label(&self) -> &'static str {
        match self {
            WallMaterial::Concrete => "CONCRETE (HARD REFLECTIVE)",
            WallMaterial::Wood => "WOOD (WARM ABSORPTION)",
            WallMaterial::StudioAcoustic => "STUDIO ACOUSTIC (BALANCED)",
            WallMaterial::Carpet => "CARPET (HIGH HF ABSORPTION)",
            WallMaterial::Glass => "GLASS (BRIGHT REFLECTIONS)",
            WallMaterial::CathedralStone => "CATHEDRAL STONE (DIFFUSE CAVERN)",
        }
    }

    /// Average mid-frequency absorption coefficient $\alpha \in [0.0, 1.0]$.
    pub fn absorption_coefficient(&self) -> f32 {
        match self {
            WallMaterial::Concrete => 0.05,
            WallMaterial::Wood => 0.20,
            WallMaterial::StudioAcoustic => 0.45,
            WallMaterial::Carpet => 0.60,
            WallMaterial::Glass => 0.08,
            WallMaterial::CathedralStone => 0.03,
        }
    }

    /// High-frequency damping factor $\beta \in [0.0, 1.0]$.
    pub fn hf_damping(&self) -> f32 {
        match self {
            WallMaterial::Concrete => 0.15,
            WallMaterial::Wood => 0.40,
            WallMaterial::StudioAcoustic => 0.55,
            WallMaterial::Carpet => 0.85,
            WallMaterial::Glass => 0.10,
            WallMaterial::CathedralStone => 0.20,
        }
    }
}

/// Metadata and real-time parameters for an individual virtual image source reflection path.
#[derive(Debug, Clone, Copy)]
pub struct ImageReflection {
    pub position: Position3D,
    pub delay_samples: f32,
    pub gain_l: f32,
    pub gain_r: f32,
    pub lp_coeff: f32,
    pub lp_state_l: f32,
    pub lp_state_r: f32,
    pub active: bool,
}

impl Default for ImageReflection {
    fn default() -> Self {
        Self {
            position: Position3D::zero(),
            delay_samples: 0.0,
            gain_l: 0.0,
            gain_r: 0.0,
            lp_coeff: 0.8,
            lp_state_l: 0.0,
            lp_state_r: 0.0,
            active: false,
        }
    }
}

/// 3D Shoebox Early Reflection & Diffuse Reverberation Acoustic Modeler Node.
#[derive(Clone)]
pub struct SpatialRoomNode {
    /// Room dimensions in meters (X: Width, Y: Depth, Z: Height).
    pub room_width: f32,
    pub room_depth: f32,
    pub room_height: f32,
    /// 3D Source position in room coordinates (meters).
    pub source_pos: Position3D,
    /// 3D Listener position in room coordinates (meters).
    pub listener_pos: Position3D,
    /// Listener yaw orientation angle in radians (0 = facing +Y front).
    pub listener_yaw: f32,
    /// Selected wall material preset.
    pub material: WallMaterial,
    /// Custom wall absorption coefficient override (0.01 .. 0.99).
    pub custom_absorption: f32,

    // Balances & Reverb parameters
    pub direct_gain: f32,
    pub early_gain: f32,
    pub late_gain: f32,
    pub rt60_seconds: f32,
    pub damping: f32,
    pub dry_wet: f32,
    pub sample_rate: u32,

    // Pre-allocated Early Reflection Delay Line
    early_delay_buf: Box<[f32; EARLY_DELAY_BUFFER_SIZE]>,
    early_write_idx: usize,
    reflections: [ImageReflection; MAX_EARLY_REFLECTIONS],

    // Pre-allocated FDN Late Reverb Engine
    fdn_buffers: [Box<[f32; FDN_MAX_DELAY]>; FDN_NUM_LINES],
    fdn_indices: [usize; FDN_NUM_LINES],
    fdn_lengths: [usize; FDN_NUM_LINES],
    fdn_feedback_gains: [f32; FDN_NUM_LINES],
    fdn_lp_states: [f32; FDN_NUM_LINES],
}

impl std::fmt::Debug for SpatialRoomNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpatialRoomNode")
            .field("room_width", &self.room_width)
            .field("room_depth", &self.room_depth)
            .field("room_height", &self.room_height)
            .field("source_pos", &self.source_pos)
            .field("listener_pos", &self.listener_pos)
            .field("material", &self.material)
            .field("rt60_seconds", &self.rt60_seconds)
            .field("dry_wet", &self.dry_wet)
            .finish()
    }
}

impl SpatialRoomNode {
    /// Creates a new 3D Shoebox Room Modeler node.
    pub fn new(sample_rate: u32) -> Self {
        let sr = if sample_rate > 0 { sample_rate } else { 48000 };

        // Mutually prime delay line lengths for 4-channel FDN
        let fdn_prime_lengths = [1087, 1283, 1511, 1789];
        let fdn_buffers = [
            Box::new([0.0; FDN_MAX_DELAY]),
            Box::new([0.0; FDN_MAX_DELAY]),
            Box::new([0.0; FDN_MAX_DELAY]),
            Box::new([0.0; FDN_MAX_DELAY]),
        ];

        let mut node = Self {
            room_width: 8.0,
            room_depth: 12.0,
            room_height: 3.5,
            source_pos: Position3D::new(3.0, 4.0, 1.5),
            listener_pos: Position3D::new(4.0, 8.0, 1.7),
            listener_yaw: 0.0,
            material: WallMaterial::StudioAcoustic,
            custom_absorption: 0.35,
            direct_gain: 1.0,
            early_gain: 0.8,
            late_gain: 0.5,
            rt60_seconds: 1.8,
            damping: 0.45,
            dry_wet: 0.5,
            sample_rate: sr,
            early_delay_buf: Box::new([0.0; EARLY_DELAY_BUFFER_SIZE]),
            early_write_idx: 0,
            reflections: [ImageReflection::default(); MAX_EARLY_REFLECTIONS],
            fdn_buffers,
            fdn_indices: [0; FDN_NUM_LINES],
            fdn_lengths: fdn_prime_lengths,
            fdn_feedback_gains: [0.7; FDN_NUM_LINES],
            fdn_lp_states: [0.0; FDN_NUM_LINES],
        };

        node.recalculate_early_reflections();
        node.recalculate_fdn_gains();
        node
    }

    /// Sets the 3D room dimensions (meters) and updates reflection geometries.
    pub fn set_room_dimensions(&mut self, width: f32, depth: f32, height: f32) {
        self.room_width = width.max(2.0);
        self.room_depth = depth.max(2.0);
        self.room_height = height.max(2.0);
        self.recalculate_early_reflections();
    }

    /// Sets source and listener 3D positions in room coordinates (meters).
    pub fn set_positions(&mut self, source: Position3D, listener: Position3D) {
        self.source_pos = source;
        self.listener_pos = listener;
        self.recalculate_early_reflections();
    }

    /// Sets RT60 reverberation decay time in seconds.
    pub fn set_rt60(&mut self, rt60: f32) {
        self.rt60_seconds = rt60.clamp(0.1, 20.0);
        self.recalculate_fdn_gains();
    }

    /// Recalculates feedback gains for the FDN lines based on desired RT60 decay.
    pub fn recalculate_fdn_gains(&mut self) {
        let sr = self.sample_rate as f32;
        let rt60_samples = (self.rt60_seconds * sr).max(100.0);
        for i in 0..FDN_NUM_LINES {
            let delay = self.fdn_lengths[i] as f32;
            // -60 dB attenuation over RT60 duration: g = 10^(-3 * delay / (RT60 * sr))
            let gain = 10.0f32.powf(-3.0 * delay / rt60_samples);
            self.fdn_feedback_gains[i] = gain.clamp(0.0, 0.99);
        }
    }

    /// Recalculates the image source coordinates, delays, attenuation, and binaural panning
    /// for direct path and up to 18 first- and second-order early reflections.
    pub fn recalculate_early_reflections(&mut self) {
        let w = self.room_width;
        let d = self.room_depth;
        let h = self.room_height;
        let s = self.source_pos;
        let l = self.listener_pos;
        let alpha = self.material.absorption_coefficient().max(self.custom_absorption * 0.5);
        let refl_coeff = (1.0 - alpha).clamp(0.05, 0.98).sqrt();

        // 1. Direct path (Index 0)
        let direct_dist = ((s.x - l.x).powi(2) + (s.y - l.y).powi(2) + (s.z - l.z).powi(2)).sqrt().max(0.1);
        let direct_delay = (direct_dist / SPEED_OF_SOUND_M_S * self.sample_rate as f32).clamp(0.0, (EARLY_DELAY_BUFFER_SIZE - 1) as f32);
        let direct_azimuth = (s.x - l.x).atan2(s.y - l.y) - self.listener_yaw;
        let direct_pan = (direct_azimuth.sin() + 1.0) * (PI / 4.0);
        let direct_att = (1.0 / direct_dist).min(2.0);

        self.reflections[0] = ImageReflection {
            position: s,
            delay_samples: direct_delay,
            gain_l: direct_pan.cos() * direct_att * self.direct_gain,
            gain_r: direct_pan.sin() * direct_att * self.direct_gain,
            lp_coeff: 1.0,
            lp_state_l: 0.0,
            lp_state_r: 0.0,
            active: true,
        };

        // 2. Six 1st-order image sources (Indices 1..6)
        let first_order_sources = [
            Position3D::new(-s.x, s.y, s.z),          // Left wall
            Position3D::new(2.0 * w - s.x, s.y, s.z),  // Right wall
            Position3D::new(s.x, 2.0 * d - s.y, s.z),  // Front wall
            Position3D::new(s.x, -s.y, s.z),          // Back wall
            Position3D::new(s.x, s.y, -s.z),          // Floor
            Position3D::new(s.x, s.y, 2.0 * h - s.z),  // Ceiling
        ];

        for (i, &img_pos) in first_order_sources.iter().enumerate() {
            let dist = ((img_pos.x - l.x).powi(2) + (img_pos.y - l.y).powi(2) + (img_pos.z - l.z).powi(2)).sqrt().max(0.2);
            let delay = (dist / SPEED_OF_SOUND_M_S * self.sample_rate as f32).clamp(0.0, (EARLY_DELAY_BUFFER_SIZE - 1) as f32);
            let azimuth = (img_pos.x - l.x).atan2(img_pos.y - l.y) - self.listener_yaw;
            let pan = (azimuth.sin() + 1.0) * (PI / 4.0);
            let att = (1.0 / dist) * refl_coeff * self.early_gain;
            let lp_coeff = (1.0 - self.damping * 0.3 * (dist / 10.0)).clamp(0.1, 0.95);

            self.reflections[1 + i] = ImageReflection {
                position: img_pos,
                delay_samples: delay,
                gain_l: pan.cos() * att,
                gain_r: pan.sin() * att,
                lp_coeff,
                lp_state_l: 0.0,
                lp_state_r: 0.0,
                active: true,
            };
        }

        // 3. Eleven 2nd-order corner image sources (Indices 7..17)
        let second_order_sources = [
            Position3D::new(-s.x, 2.0 * d - s.y, s.z),
            Position3D::new(-s.x, -s.y, s.z),
            Position3D::new(2.0 * w - s.x, 2.0 * d - s.y, s.z),
            Position3D::new(2.0 * w - s.x, -s.y, s.z),
            Position3D::new(-s.x, s.y, 2.0 * h - s.z),
            Position3D::new(-s.x, s.y, -s.z),
            Position3D::new(2.0 * w - s.x, s.y, 2.0 * h - s.z),
            Position3D::new(2.0 * w - s.x, s.y, -s.z),
            Position3D::new(s.x, 2.0 * d - s.y, 2.0 * h - s.z),
            Position3D::new(s.x, -s.y, 2.0 * h - s.z),
            Position3D::new(s.x, -s.y, -s.z),
        ];

        let refl_coeff_2 = refl_coeff * refl_coeff;
        for (i, &img_pos) in second_order_sources.iter().enumerate() {
            if 7 + i >= MAX_EARLY_REFLECTIONS {
                break;
            }
            let dist = ((img_pos.x - l.x).powi(2) + (img_pos.y - l.y).powi(2) + (img_pos.z - l.z).powi(2)).sqrt().max(0.3);
            let delay = (dist / SPEED_OF_SOUND_M_S * self.sample_rate as f32).clamp(0.0, (EARLY_DELAY_BUFFER_SIZE - 1) as f32);
            let azimuth = (img_pos.x - l.x).atan2(img_pos.y - l.y) - self.listener_yaw;
            let pan = (azimuth.sin() + 1.0) * (PI / 4.0);
            let att = (1.0 / dist) * refl_coeff_2 * self.early_gain * 0.75;
            let lp_coeff = (1.0 - self.damping * 0.5 * (dist / 10.0)).clamp(0.05, 0.9);

            self.reflections[7 + i] = ImageReflection {
                position: img_pos,
                delay_samples: delay,
                gain_l: pan.cos() * att,
                gain_r: pan.sin() * att,
                lp_coeff,
                lp_state_l: 0.0,
                lp_state_r: 0.0,
                active: true,
            };
        }
    }

    /// Resets all delay line buffers and filter memories.
    pub fn reset(&mut self) {
        self.early_delay_buf.fill(0.0);
        self.early_write_idx = 0;
        for buf in self.fdn_buffers.iter_mut() {
            buf.fill(0.0);
        }
        self.fdn_indices.fill(0);
        self.fdn_lp_states.fill(0.0);
        for refl in self.reflections.iter_mut() {
            refl.lp_state_l = 0.0;
            refl.lp_state_r = 0.0;
        }
    }
}

impl SignalProcessor for SpatialRoomNode {
    fn name(&self) -> &str {
        "SpatialRoomNode"
    }

    fn process_block(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        if outputs.is_empty() {
            return;
        }

        let num_samples = outputs[0].len();
        if num_samples == 0 {
            return;
        }

        let sample_rate = if ctx.sample_rate > 0 {
            ctx.sample_rate
        } else {
            self.sample_rate
        };
        if sample_rate != self.sample_rate {
            self.sample_rate = sample_rate;
            self.recalculate_early_reflections();
            self.recalculate_fdn_gains();
        }

        // Enforce zero heap allocation in audio callback
        let _guard = AllocGuard::new();

        let has_input = !inputs.is_empty() && !inputs[0].is_empty();
        let is_stereo_in = inputs.len() >= 2 && !inputs[1].is_empty();
        let is_stereo_out = outputs.len() >= 2;

        let early_len = EARLY_DELAY_BUFFER_SIZE;

        for i in 0..num_samples {
            let in_l = if has_input && i < inputs[0].len() {
                inputs[0][i]
            } else {
                0.0
            };
            let in_r = if is_stereo_in && i < inputs[1].len() {
                inputs[1][i]
            } else {
                in_l
            };
            let mono_in = (in_l + in_r) * 0.5;

            // 1. Write mono input to early reflection circular buffer
            let w_idx = self.early_write_idx;
            self.early_delay_buf[w_idx] = mono_in;
            self.early_write_idx = (self.early_write_idx + 1) % early_len;

            // 2. Accumulate Direct & Early Reflection Image Sources
            let mut early_accum_l = 0.0f32;
            let mut early_accum_r = 0.0f32;

            for refl in self.reflections.iter_mut() {
                if !refl.active {
                    continue;
                }

                let d = refl.delay_samples;
                let d_floor = d.floor() as usize;
                let frac = d - d.floor();

                let r_idx0 = (w_idx + early_len - (d_floor % early_len)) % early_len;
                let r_idx1 = (r_idx0 + early_len - 1) % early_len;

                let s0 = self.early_delay_buf[r_idx0];
                let s1 = self.early_delay_buf[r_idx1];
                let raw_sample = s0 + frac * (s1 - s0);

                // 1-pole HF damping filter per reflection path
                let lp = refl.lp_coeff;
                refl.lp_state_l = refl.lp_state_l * (1.0 - lp) + raw_sample * lp;
                refl.lp_state_r = refl.lp_state_r * (1.0 - lp) + raw_sample * lp;

                early_accum_l += refl.lp_state_l * refl.gain_l;
                early_accum_r += refl.lp_state_r * refl.gain_r;
            }

            // 3. Feed Early Accumulation into FDN Late Diffuse Reverb Tank
            let fdn_in = (early_accum_l + early_accum_r) * 0.5;
            let mut fdn_outs = [0.0f32; FDN_NUM_LINES];

            for (ch, out) in fdn_outs.iter_mut().enumerate() {
                let idx = self.fdn_indices[ch];
                *out = self.fdn_buffers[ch][idx];
            }

            // 4x4 Hadamard / Householder lossless orthogonal scattering matrix:
            // H = 0.5 * [ [1, 1, 1, 1], [1, -1, 1, -1], [1, 1, -1, -1], [1, -1, -1, 1] ]
            let [u0, u1, u2, u3] = fdn_outs;
            let s0 = 0.5 * (u0 + u1 + u2 + u3);
            let s1 = 0.5 * (u0 - u1 + u2 - u3);
            let s2 = 0.5 * (u0 + u1 - u2 - u3);
            let s3 = 0.5 * (u0 - u1 - u2 + u3);

            let scattered = [s0, s1, s2, s3];
            let damp_coeff = (1.0 - self.damping * 0.6).clamp(0.1, 0.99);

            for (ch, &scat) in scattered.iter().enumerate() {
                let len = self.fdn_lengths[ch];
                let idx = self.fdn_indices[ch];

                // Apply damping lowpass and feedback gain
                let fed = scat * self.fdn_feedback_gains[ch] + fdn_in * 0.25;
                self.fdn_lp_states[ch] = self.fdn_lp_states[ch] * (1.0 - damp_coeff) + fed * damp_coeff;

                self.fdn_buffers[ch][idx] = self.fdn_lp_states[ch];
                self.fdn_indices[ch] = (idx + 1) % len;
            }

            let late_l = (fdn_outs[0] + fdn_outs[2]) * self.late_gain;
            let late_r = (fdn_outs[1] + fdn_outs[3]) * self.late_gain;

            // 4. Mix Direct + Early + Late with Dry/Wet blend
            let wet_l = early_accum_l + late_l;
            let wet_r = early_accum_r + late_r;

            let out_l_final = in_l * (1.0 - self.dry_wet) + wet_l * self.dry_wet;
            let out_r_final = in_r * (1.0 - self.dry_wet) + wet_r * self.dry_wet;

            outputs[0][i] = out_l_final.clamp(-1.0, 1.0);
            if is_stereo_out && i < outputs[1].len() {
                outputs[1][i] = out_r_final.clamp(-1.0, 1.0);
            }
        }
    }
}

impl AudioNode for SpatialRoomNode {
    fn name(&self) -> &str {
        "SpatialRoomNode"
    }

    fn process(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        self.process_block(inputs, outputs, ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::transport::Transport;

    #[test]
    fn test_spatial_room_image_sources_recalculation() {
        let mut room = SpatialRoomNode::new(48000);
        room.set_room_dimensions(10.0, 15.0, 4.0);
        room.set_positions(Position3D::new(2.0, 3.0, 1.5), Position3D::new(5.0, 10.0, 1.7));
        room.set_rt60(2.5);

        assert!(room.reflections[0].active);
        assert!(room.reflections[0].delay_samples > 0.0);
        assert!(room.reflections[1].active);
        assert!(room.reflections[1].delay_samples > room.reflections[0].delay_samples);
    }

    #[test]
    fn test_spatial_room_zero_allocation_and_reverberation() {
        let mut room = SpatialRoomNode::new(48000);
        let transport = Transport::new(48000, 120.0);
        let ctx = ProcessContext::from_transport(&transport);

        let mut in_l = vec![0.0f32; 128];
        let mut in_r = vec![0.0f32; 128];
        in_l[0] = 1.0; // Impulse
        in_r[0] = 1.0;

        let mut out_l = vec![0.0f32; 128];
        let mut out_r = vec![0.0f32; 128];

        room.process_block(
            &[&in_l[..], &in_r[..]],
            &mut [&mut out_l[..], &mut out_r[..]],
            &ctx,
        );

        assert!(out_l.iter().all(|s| s.is_finite()));
        assert!(out_r.iter().all(|s| s.is_finite()));

        // Process decay blocks
        in_l.fill(0.0);
        in_r.fill(0.0);
        for _ in 0..20 {
            room.process_block(
                &[&in_l[..], &in_r[..]],
                &mut [&mut out_l[..], &mut out_r[..]],
                &ctx,
            );
        }

        assert!(out_l.iter().any(|s| *s != 0.0));
    }
}
