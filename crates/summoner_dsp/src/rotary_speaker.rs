// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling 2-Way Leslie Rotary Speaker Cabinet & Doppler Convolver (Milestone 21).
//!
//! Provides a physical modeling 2-way rotating speaker cabinet (independent treble horn rotor
//! and bass drum rotor), mechanical inertia acceleration physics, 800 Hz crossover network,
//! stereo Doppler frequency modulation, directional polar amplitude radiation, wooden cabinet
//! acoustic reflections, and 6550 tube preamp overdrive saturation.
//!
//! Enforces zero heap allocations in real-time audio render loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::f32::consts::{PI, TAU};
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};
use crate::traits::SignalProcessor;

/// Speed of sound in air in meters per second.
const SPEED_OF_SOUND_MPS: f32 = 343.0;

fn default_delay_buf() -> [f32; MAX_DELAY_SAMPLES] {
    [0.0; MAX_DELAY_SAMPLES]
}

fn default_refl_buf() -> [f32; 1024] {
    [0.0; 1024]
}

/// Rotary speaker motor operational speed state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RotarySpeed {
    /// Stationary stopped rotors.
    Stop,
    /// Slow rotation for lush spatial chorus (~40 RPM horn / ~36 RPM drum).
    Chorale,
    /// Fast rotation for intense Doppler vibrato & tremolo (~400 RPM horn / ~342 RPM drum).
    #[default]
    Tremolo,
    /// Active mechanical friction brake deceleration to stop.
    Brake,
}

/// Rotary speaker cabinet acoustic and hardware models.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RotaryCabinetModel {
    /// Classic Leslie 122 wooden cabinet with 40W 6550 tube power amplifier and warm wooden enclosure.
    #[default]
    Leslie122VintageTube,
    /// Leslie 147 open-back stage cabinet with punchy mid-range and bright top-end horn flare.
    Leslie147OpenBack,
    /// Leslie 760 high-power solid-state loud stage dispersion with aggressive transient bite.
    Leslie760SolidState,
    /// Custom dual counter-rotating horns with wide binaural projection and deep chamber resonance.
    CustomTwinHornSpatial,
}

impl RotaryCabinetModel {
    /// Returns default nominal $(h_{\text{accel\_sec}}, d_{\text{accel\_sec}}, \text{drive\_db}, \text{horn\_radius\_m}, \text{drum\_radius\_m})$.
    pub fn nominal_physics(&self) -> (f32, f32, f32, f32, f32) {
        match self {
            Self::Leslie122VintageTube => (1.1, 4.8, 6.0, 0.18, 0.24),
            Self::Leslie147OpenBack => (0.9, 4.2, 8.5, 0.19, 0.25),
            Self::Leslie760SolidState => (0.8, 3.8, 4.0, 0.17, 0.23),
            Self::CustomTwinHornSpatial => (1.3, 5.5, 5.0, 0.22, 0.28),
        }
    }

    /// Returns human-readable name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Leslie122VintageTube => "Leslie 122 Vintage Tube Cabinet",
            Self::Leslie147OpenBack => "Leslie 147 Open-Back Stage Cabinet",
            Self::Leslie760SolidState => "Leslie 760 Solid-State Cabinet",
            Self::CustomTwinHornSpatial => "Custom Twin-Horn Spatial Cabinet",
        }
    }
}

/// Maximum delay buffer capacity in samples (covers ~50ms delay for Doppler and reflections).
const MAX_DELAY_SAMPLES: usize = 4096;

/// Physical modeling 2-way Leslie rotary speaker cabinet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotarySpeaker {
    pub sample_rate: u32,
    pub cabinet_model: RotaryCabinetModel,
    pub speed: RotarySpeed,
    /// High-frequency treble horn rotation speed in RPM.
    pub horn_rpm: f32,
    /// Low-frequency bass drum rotation speed in RPM.
    pub drum_rpm: f32,
    /// Horn acceleration/deceleration time constant in seconds $[0.2 ..= 5.0\text{s}]$.
    pub horn_accel_time_s: f32,
    /// Drum acceleration/deceleration time constant in seconds $[1.0 ..= 10.0\text{s}]$.
    pub drum_accel_time_s: f32,
    /// Horn vs Drum acoustic balance percentage $[0.0 ..= 100.0\%]$ (60% = +2dB horn).
    pub horn_drum_balance_pct: f32,
    /// Stereo microphone distance from cabinet baffle in meters $[0.2 ..= 2.0\text{m}]$.
    pub mic_distance_m: f32,
    /// Stereo microphone spread angle in degrees $[60.0 ..= 180.0^\circ]$.
    pub mic_spread_deg: f32,
    /// Tube power amplifier drive saturation in dB $[0.0 ..= 24.0\text{dB}]$.
    pub drive_saturation_db: f32,
    /// Physical radius of the rotating treble horn flare in meters.
    pub horn_radius_m: f32,
    /// Physical radius of the rotating bass drum baffle in meters.
    pub drum_radius_m: f32,
    /// Wet/Dry mix $[0.0 ..= 1.0]$.
    pub wet_mix: f32,
    // Instantaneous mechanical rotor angles in radians
    #[serde(skip)]
    pub horn_angle_rad: f32,
    #[serde(skip)]
    pub drum_angle_rad: f32,
    // 800 Hz 2-way Linkwitz-Riley crossover filter states
    #[serde(skip)]
    lp_x1: f32,
    #[serde(skip)]
    lp_x2: f32,
    #[serde(skip)]
    lp_y1: f32,
    #[serde(skip)]
    lp_y2: f32,
    #[serde(skip)]
    hp_x1: f32,
    #[serde(skip)]
    hp_x2: f32,
    #[serde(skip)]
    hp_y1: f32,
    #[serde(skip)]
    hp_y2: f32,
    // Fractional delay lines for horn and drum Doppler modulation (Left & Right)
    #[serde(skip, default = "default_delay_buf")]
    horn_delay_buf: [f32; MAX_DELAY_SAMPLES],
    #[serde(skip, default = "default_delay_buf")]
    drum_delay_buf: [f32; MAX_DELAY_SAMPLES],
    #[serde(skip)]
    delay_write_pos: usize,
    // Cabinet cavity reflection comb filter states
    #[serde(skip, default = "default_refl_buf")]
    refl_buf_l: [f32; 1024],
    #[serde(skip, default = "default_refl_buf")]
    refl_buf_r: [f32; 1024],
    #[serde(skip)]
    refl_write_pos: usize,
    // Output DC blocking filter states
    #[serde(skip)]
    dc_l_x1: f32,
    #[serde(skip)]
    dc_l_y1: f32,
    #[serde(skip)]
    dc_r_x1: f32,
    #[serde(skip)]
    dc_r_y1: f32,
}

impl RotarySpeaker {
    /// Creates a new RotarySpeaker physical modeling cabinet.
    pub fn new(sample_rate: u32) -> Self {
        let sr = sample_rate.max(8000);
        let model = RotaryCabinetModel::Leslie122VintageTube;
        let (h_acc, d_acc, drive, h_rad, d_rad) = model.nominal_physics();

        Self {
            sample_rate: sr,
            cabinet_model: model,
            speed: RotarySpeed::Tremolo,
            horn_rpm: 395.0,
            drum_rpm: 338.0,
            horn_accel_time_s: h_acc,
            drum_accel_time_s: d_acc,
            horn_drum_balance_pct: 60.0,
            mic_distance_m: 0.65,
            mic_spread_deg: 120.0,
            drive_saturation_db: drive,
            horn_radius_m: h_rad,
            drum_radius_m: d_rad,
            wet_mix: 1.0,
            horn_angle_rad: 0.78,
            drum_angle_rad: 2.14,
            lp_x1: 0.0,
            lp_x2: 0.0,
            lp_y1: 0.0,
            lp_y2: 0.0,
            hp_x1: 0.0,
            hp_x2: 0.0,
            hp_y1: 0.0,
            hp_y2: 0.0,
            horn_delay_buf: [0.0; MAX_DELAY_SAMPLES],
            drum_delay_buf: [0.0; MAX_DELAY_SAMPLES],
            delay_write_pos: 0,
            refl_buf_l: [0.0; 1024],
            refl_buf_r: [0.0; 1024],
            refl_write_pos: 0,
            dc_l_x1: 0.0,
            dc_l_y1: 0.0,
            dc_r_x1: 0.0,
            dc_r_y1: 0.0,
        }
    }

    /// Sets cabinet model and updates nominal physics.
    pub fn set_cabinet_model(&mut self, model: RotaryCabinetModel) {
        self.cabinet_model = model;
        let (h_acc, d_acc, drive, h_rad, d_rad) = model.nominal_physics();
        self.horn_accel_time_s = h_acc;
        self.drum_accel_time_s = d_acc;
        self.drive_saturation_db = drive;
        self.horn_radius_m = h_rad;
        self.drum_radius_m = d_rad;
    }

    /// Sets motor speed state (Stop, Chorale, Tremolo, Brake).
    pub fn set_speed(&mut self, speed: RotarySpeed) {
        self.speed = speed;
    }

    /// Returns target horn RPM for the active speed state.
    pub fn target_horn_rpm(&self) -> f32 {
        match self.speed {
            RotarySpeed::Stop | RotarySpeed::Brake => 0.0,
            RotarySpeed::Chorale => 40.0,
            RotarySpeed::Tremolo => 400.0,
        }
    }

    /// Returns target drum RPM for the active speed state.
    pub fn target_drum_rpm(&self) -> f32 {
        match self.speed {
            RotarySpeed::Stop | RotarySpeed::Brake => 0.0,
            RotarySpeed::Chorale => 36.0,
            RotarySpeed::Tremolo => 342.0,
        }
    }

    /// Advances mechanical rotor inertia physics by $dt$ seconds.
    pub fn update_mechanics(&mut self, dt: f32) {
        let target_h = self.target_horn_rpm();
        let target_d = self.target_drum_rpm();

        let h_rate = 400.0 / self.horn_accel_time_s.max(0.1);
        let d_rate = 342.0 / self.drum_accel_time_s.max(0.5);

        // Horn rotor inertia
        if self.horn_rpm < target_h {
            self.horn_rpm = (self.horn_rpm + h_rate * dt).min(target_h);
        } else if self.horn_rpm > target_h {
            self.horn_rpm = (self.horn_rpm - h_rate * dt).max(target_h);
        }

        // Drum rotor inertia
        if self.drum_rpm < target_d {
            self.drum_rpm = (self.drum_rpm + d_rate * dt).min(target_d);
        } else if self.drum_rpm > target_d {
            self.drum_rpm = (self.drum_rpm - d_rate * dt).max(target_d);
        }

        // Angular velocities in radians per second: omega = RPM * 2*pi / 60
        let horn_omega = self.horn_rpm * (TAU / 60.0);
        let drum_omega = self.drum_rpm * (TAU / 60.0);

        // Treble horn rotates clockwise (+), Bass drum rotates counter-clockwise (-)
        self.horn_angle_rad = (self.horn_angle_rad + horn_omega * dt) % TAU;
        self.drum_angle_rad = (self.drum_angle_rad - drum_omega * dt + TAU) % TAU;
    }

    /// Tube preamp soft saturation transfer function (6550 power amp modeling).
    #[inline(always)]
    fn tube_saturate(input: f32, drive_db: f32) -> f32 {
        let drive_lin = 10.0_f32.powf(drive_db / 20.0);
        let x = input * drive_lin;
        // Asymmetric warm tube transfer curve with soft compression
        if x > 0.0 {
            (1.0 - (-x * 1.2).exp()) / 1.2
        } else {
            -(-x).tanh()
        }
    }

    /// Evaluates 800 Hz 2-way Linkwitz-Riley 2nd-order crossover filter.
    #[inline(always)]
    fn crossover_filter(&mut self, input: f32) -> (f32, f32) {
        // 800 Hz 2nd-order Butterworth biquad coefficients at sample rate
        let fc = 800.0;
        let q = std::f32::consts::FRAC_1_SQRT_2;
        let omega = 2.0 * PI * fc / (self.sample_rate as f32);
        let alpha = omega.sin() / (2.0 * q);
        let cos_w = omega.cos();

        // Lowpass coefficients
        let b0_lp = (1.0 - cos_w) * 0.5;
        let b1_lp = 1.0 - cos_w;
        let b2_lp = (1.0 - cos_w) * 0.5;
        let a0_lp = 1.0 + alpha;
        let a1_lp = -2.0 * cos_w;
        let a2_lp = 1.0 - alpha;

        let lp_out = (b0_lp * input + b1_lp * self.lp_x1 + b2_lp * self.lp_x2
            - a1_lp * self.lp_y1 - a2_lp * self.lp_y2) / a0_lp;
        self.lp_x2 = self.lp_x1;
        self.lp_x1 = input;
        self.lp_y2 = self.lp_y1;
        self.lp_y1 = if lp_out.is_finite() { lp_out } else { 0.0 };

        // Highpass coefficients
        let b0_hp = (1.0 + cos_w) * 0.5;
        let b1_hp = -(1.0 + cos_w);
        let b2_hp = (1.0 + cos_w) * 0.5;
        let a0_hp = 1.0 + alpha;
        let a1_hp = -2.0 * cos_w;
        let a2_hp = 1.0 - alpha;

        let hp_out = (b0_hp * input + b1_hp * self.hp_x1 + b2_hp * self.hp_x2
            - a1_hp * self.hp_y1 - a2_hp * self.hp_y2) / a0_hp;
        self.hp_x2 = self.hp_x1;
        self.hp_x1 = input;
        self.hp_y2 = self.hp_y1;
        self.hp_y1 = if hp_out.is_finite() { hp_out } else { 0.0 };

        (self.lp_y1, self.hp_y1)
    }

    /// Processes one sample through the 2-way rotating speaker cabinet.
    pub fn process_sample(&mut self, input_sample: f32) -> (f32, f32) {
        let dt = 1.0 / (self.sample_rate as f32);
        self.update_mechanics(dt);

        // 1. Preamp Tube Overdrive Stage
        let saturated_in = Self::tube_saturate(input_sample, self.drive_saturation_db);

        // 2. 800 Hz 2-Way Crossover Network
        let (low_band, high_band) = self.crossover_filter(saturated_in);

        // Write bands into ring delay buffers
        self.horn_delay_buf[self.delay_write_pos] = high_band;
        self.drum_delay_buf[self.delay_write_pos] = low_band;

        // 3. Microphone positions & Doppler geometry
        let mic_spread_rad = (self.mic_spread_deg * 0.5).to_radians();
        let mic_angle_l = -mic_spread_rad;
        let mic_angle_r = mic_spread_rad;

        let base_delay_samples = (self.mic_distance_m / SPEED_OF_SOUND_MPS) * (self.sample_rate as f32) + 128.0;

        // --- Horn Doppler & Polar Radiation ---
        let horn_rel_l = self.horn_angle_rad - mic_angle_l;
        let horn_rel_r = self.horn_angle_rad - mic_angle_r;

        // Path distance modulation: delta_d = -r * cos(angle - mic)
        let horn_delta_d_l = -self.horn_radius_m * horn_rel_l.cos();
        let horn_delta_d_r = -self.horn_radius_m * horn_rel_r.cos();

        let horn_delay_l = base_delay_samples + (horn_delta_d_l / SPEED_OF_SOUND_MPS) * (self.sample_rate as f32);
        let horn_delay_r = base_delay_samples + (horn_delta_d_r / SPEED_OF_SOUND_MPS) * (self.sample_rate as f32);

        let horn_sig_l = self.read_delay_interp(&self.horn_delay_buf, horn_delay_l);
        let horn_sig_r = self.read_delay_interp(&self.horn_delay_buf, horn_delay_r);

        // Horn Cardioid / Polar radiation pattern: (1 + cos(rel_angle)) / 2
        let horn_amp_l = 0.35 + 0.65 * (0.5 + 0.5 * horn_rel_l.cos()).powf(1.8);
        let horn_amp_r = 0.35 + 0.65 * (0.5 + 0.5 * horn_rel_r.cos()).powf(1.8);

        // --- Drum Doppler & Polar Radiation ---
        let drum_rel_l = self.drum_angle_rad - mic_angle_l;
        let drum_rel_r = self.drum_angle_rad - mic_angle_r;

        let drum_delta_d_l = -self.drum_radius_m * drum_rel_l.cos();
        let drum_delta_d_r = -self.drum_radius_m * drum_rel_r.cos();

        let drum_delay_l = base_delay_samples + (drum_delta_d_l / SPEED_OF_SOUND_MPS) * (self.sample_rate as f32);
        let drum_delay_r = base_delay_samples + (drum_delta_d_r / SPEED_OF_SOUND_MPS) * (self.sample_rate as f32);

        let drum_sig_l = self.read_delay_interp(&self.drum_delay_buf, drum_delay_l);
        let drum_sig_r = self.read_delay_interp(&self.drum_delay_buf, drum_delay_r);

        // Drum radiation pattern (gentler cardioid)
        let drum_amp_l = 0.55 + 0.45 * (0.5 + 0.5 * drum_rel_l.cos());
        let drum_amp_r = 0.55 + 0.45 * (0.5 + 0.5 * drum_rel_r.cos());

        self.delay_write_pos = (self.delay_write_pos + 1) % MAX_DELAY_SAMPLES;

        // 4. Horn/Drum Acoustic Balance Mixing
        let horn_gain = (self.horn_drum_balance_pct / 50.0).clamp(0.1, 2.0);
        let drum_gain = ((100.0 - self.horn_drum_balance_pct) / 50.0).clamp(0.1, 2.0);

        let mut out_wet_l = (horn_sig_l * horn_amp_l * horn_gain) + (drum_sig_l * drum_amp_l * drum_gain);
        let mut out_wet_r = (horn_sig_r * horn_amp_r * horn_gain) + (drum_sig_r * drum_amp_r * drum_gain);

        // 5. Wooden Cabinet Cavity Reflections (multi-tap delay)
        let refl_idx = self.refl_write_pos;
        let refl_l = self.refl_buf_l[(refl_idx + 1024 - 180) % 1024] * 0.18
            + self.refl_buf_l[(refl_idx + 1024 - 340) % 1024] * 0.12;
        let refl_r = self.refl_buf_r[(refl_idx + 1024 - 210) % 1024] * 0.18
            + self.refl_buf_r[(refl_idx + 1024 - 390) % 1024] * 0.12;

        self.refl_buf_l[refl_idx] = out_wet_l;
        self.refl_buf_r[refl_idx] = out_wet_r;
        self.refl_write_pos = (refl_idx + 1) % 1024;

        out_wet_l += refl_l;
        out_wet_r += refl_r;

        // Wet/Dry mix
        let out_l = input_sample * (1.0 - self.wet_mix) + out_wet_l * self.wet_mix;
        let out_r = input_sample * (1.0 - self.wet_mix) + out_wet_r * self.wet_mix;

        // 6. DC Block filter
        let dc_l = out_l - self.dc_l_x1 + 0.995 * self.dc_l_y1;
        self.dc_l_x1 = out_l;
        self.dc_l_y1 = if dc_l.is_finite() { dc_l } else { 0.0 };

        let dc_r = out_r - self.dc_r_x1 + 0.995 * self.dc_r_y1;
        self.dc_r_x1 = out_r;
        self.dc_r_y1 = if dc_r.is_finite() { dc_r } else { 0.0 };

        (self.dc_l_y1.clamp(-1.0, 1.0), self.dc_r_y1.clamp(-1.0, 1.0))
    }

    /// Linearly interpolated delay buffer read.
    #[inline(always)]
    fn read_delay_interp(&self, buf: &[f32; MAX_DELAY_SAMPLES], delay_samples: f32) -> f32 {
        let d_clamped = delay_samples.clamp(1.0, (MAX_DELAY_SAMPLES - 2) as f32);
        let read_pos = (self.delay_write_pos as f32 + MAX_DELAY_SAMPLES as f32 - d_clamped) % (MAX_DELAY_SAMPLES as f32);
        let idx0 = read_pos.floor() as usize % MAX_DELAY_SAMPLES;
        let idx1 = (idx0 + 1) % MAX_DELAY_SAMPLES;
        let frac = read_pos.fract();
        buf[idx0] * (1.0 - frac) + buf[idx1] * frac
    }
}

impl SignalProcessor for RotarySpeaker {
    fn name(&self) -> &str {
        "RotarySpeaker"
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

            let (out_l, out_r) = self.process_sample(in_sample);
            if !outputs.is_empty() && i < outputs[0].len() {
                outputs[0][i] = out_l;
            }
            if outputs.len() > 1 && i < outputs[1].len() {
                outputs[1][i] = out_r;
            }
        }
    }
}

/// AudioNode wrapper for Rotary Speaker Cabinet processor.
#[derive(Debug)]
pub struct RotarySpeakerNode {
    pub speaker: RotarySpeaker,
}

impl RotarySpeakerNode {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            speaker: RotarySpeaker::new(sample_rate),
        }
    }
}

impl AudioNode for RotarySpeakerNode {
    fn name(&self) -> &str {
        "RotarySpeakerNode"
    }

    fn process(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        self.speaker.process_block(inputs, outputs, ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::allocator::AllocGuard;

    #[test]
    fn test_rotary_speaker_acceleration_and_doppler_modulation() {
        let mut speaker = RotarySpeaker::new(48000);
        speaker.set_speed(RotarySpeed::Tremolo);

        let in_sine = (0..4800)
            .map(|i| (i as f32 * TAU * 440.0 / 48000.0).sin())
            .collect::<Vec<f32>>();

        let mut peak_l = 0.0f32;
        let mut peak_r = 0.0f32;

        for &sample in &in_sine {
            let (l, r) = speaker.process_sample(sample);
            assert!(l.is_finite());
            assert!(r.is_finite());
            assert!((-1.0..=1.0).contains(&l));
            assert!((-1.0..=1.0).contains(&r));
            peak_l = peak_l.max(l.abs());
            peak_r = peak_r.max(r.abs());
        }

        assert!(peak_l > 0.05, "Rotary left channel must have active signal");
        assert!(peak_r > 0.05, "Rotary right channel must have active signal");

        // Test braking deceleration
        speaker.set_speed(RotarySpeed::Brake);
        for _ in 0..(48000 * 6) {
            speaker.process_sample(0.0);
        }
        assert_eq!(speaker.horn_rpm, 0.0);
        assert_eq!(speaker.drum_rpm, 0.0);
    }

    #[test]
    fn test_rotary_speaker_zero_allocation_in_loop() {
        let mut speaker = RotarySpeaker::new(48000);
        speaker.set_speed(RotarySpeed::Tremolo);

        let in_buf = [0.5f32; 256];
        let mut out_l = [0.0f32; 256];
        let mut out_r = [0.0f32; 256];
        let ctx = ProcessContext::new(48000, 120.0, 0);

        let _guard = AllocGuard::new();
        for _ in 0..10 {
            speaker.process_block(&[&in_buf[..]], &mut [&mut out_l[..], &mut out_r[..]], &ctx);
        }
    }
}
