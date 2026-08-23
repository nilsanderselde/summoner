// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Analog Tape Saturation & Magnetic Hysteresis Modeler (Milestone 10).
//!
//! Provides realistic non-linear magnetic tape hysteresis simulation with AC bias,
//! multiple tape speeds (7.5 / 15 / 30 IPS), speed-dependent head bump resonance,
//! high-frequency gap/spacing loss damping, and wow/flutter mechanical modulation.
//!
//! Enforces zero heap allocations in real-time audio render loops under `AllocGuard`.

use crate::traits::SignalProcessor;
use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};

/// Tape operational speed in Inches Per Second (IPS).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TapeSpeed {
    /// 7.5 IPS: Vintage warmth, higher harmonic saturation, low-frequency head bump ~50 Hz, HF rolloff ~14 kHz.
    Ips7_5,
    /// 15.0 IPS: Studio standard punch, balanced harmonic depth, head bump ~80 Hz, HF rolloff ~19 kHz.
    #[default]
    Ips15,
    /// 30.0 IPS: Mastering grade high-fidelity, subtle saturation, head bump ~120 Hz, extended top-end.
    Ips30,
}

impl TapeSpeed {
    /// Nominal speed in inches per second.
    pub fn ips_value(&self) -> f32 {
        match self {
            TapeSpeed::Ips7_5 => 7.5,
            TapeSpeed::Ips15 => 15.0,
            TapeSpeed::Ips30 => 30.0,
        }
    }

    /// Center frequency (Hz) for the reproductive head bump resonance.
    pub fn head_bump_freq(&self) -> f32 {
        match self {
            TapeSpeed::Ips7_5 => 50.0,
            TapeSpeed::Ips15 => 80.0,
            TapeSpeed::Ips30 => 120.0,
        }
    }

    /// Head bump peaking Q factor.
    pub fn head_bump_q(&self) -> f32 {
        match self {
            TapeSpeed::Ips7_5 => 1.4,
            TapeSpeed::Ips15 => 1.2,
            TapeSpeed::Ips30 => 1.0,
        }
    }

    /// Peak head bump boost in decibels.
    pub fn head_bump_gain_db(&self) -> f32 {
        match self {
            TapeSpeed::Ips7_5 => 3.2,
            TapeSpeed::Ips15 => 2.5,
            TapeSpeed::Ips30 => 1.6,
        }
    }

    /// High frequency gap-loss rolloff frequency (Hz).
    pub fn hf_loss_cutoff(&self) -> f32 {
        match self {
            TapeSpeed::Ips7_5 => 14000.0,
            TapeSpeed::Ips15 => 19000.0,
            TapeSpeed::Ips30 => 24000.0,
        }
    }

    /// Saturation multiplier character.
    pub fn saturation_factor(&self) -> f32 {
        match self {
            TapeSpeed::Ips7_5 => 1.35,
            TapeSpeed::Ips15 => 1.00,
            TapeSpeed::Ips30 => 0.75,
        }
    }
}

/// 2nd-order peaking biquad filter for low-frequency head bump simulation.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct HeadBumpFilter {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    z1: f32,
    z2: f32,
}

impl HeadBumpFilter {
    pub fn reset(&mut self) {
        self.z1 = 0.0;
        self.z2 = 0.0;
    }

    /// Recompute peaking filter coefficients based on center freq, boost gain, and Q.
    pub fn update(&mut self, freq_hz: f32, gain_db: f32, q: f32, sample_rate: f32) {
        let sr = sample_rate.max(1000.0);
        let fc = freq_hz.clamp(10.0, sr * 0.45);
        let w0 = TAU * fc / sr;
        let cos_w = w0.cos();
        let sin_w = w0.sin();
        let a = 10.0f32.powf(gain_db / 40.0);
        let alpha = sin_w / (2.0 * q.max(0.1));

        let a0 = 1.0 + alpha / a;
        self.b0 = (1.0 + alpha * a) / a0;
        self.b1 = (-2.0 * cos_w) / a0;
        self.b2 = (1.0 - alpha * a) / a0;
        self.a1 = (-2.0 * cos_w) / a0;
        self.a2 = (1.0 - alpha / a) / a0;
    }

    #[inline]
    pub fn process_sample(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.z1;
        self.z1 = self.b1 * x - self.a1 * y + self.z2;
        self.z2 = self.b2 * x - self.a2 * y;
        y
    }
}

/// Buffer size for short wow/flutter mechanical transport delay line.
pub const WOW_FLUTTER_BUFFER_SIZE: usize = 1024;

/// Internal channel DSP state for tape saturation and hysteresis.
#[derive(Debug, Clone)]
pub struct TapeChannelState {
    /// Current magnetic domain state (magnetization $M$).
    pub magnetization: f32,
    /// Head bump resonator filter.
    pub head_bump_filter: HeadBumpFilter,
    /// High-frequency gap loss 1-pole filter state.
    pub gap_loss_state: f32,
    /// High-frequency tone 1-pole filter state.
    pub tone_state: f32,
    /// Wow/Flutter circular delay buffer.
    pub delay_buf: [f32; WOW_FLUTTER_BUFFER_SIZE],
    /// Delay write index.
    pub write_idx: usize,
}

impl Default for TapeChannelState {
    fn default() -> Self {
        Self {
            magnetization: 0.0,
            head_bump_filter: HeadBumpFilter::default(),
            gap_loss_state: 0.0,
            tone_state: 0.0,
            delay_buf: [0.0; WOW_FLUTTER_BUFFER_SIZE],
            write_idx: 0,
        }
    }
}

impl TapeChannelState {
    pub fn reset(&mut self) {
        self.magnetization = 0.0;
        self.head_bump_filter.reset();
        self.gap_loss_state = 0.0;
        self.tone_state = 0.0;
        self.delay_buf.fill(0.0);
        self.write_idx = 0;
    }
}

/// Comprehensive Analog Tape Saturation & Magnetic Hysteresis DSP Node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TapeSaturationNode {
    /// Input drive gain multiplier (0.1 to 10.0, default 1.5).
    pub drive: f32,
    /// Non-linear magnetic core saturation level (0.0 to 1.0, default 0.6).
    pub saturation: f32,
    /// AC bias calibration offset (-1.0 to 1.0, optimal = 0.0).
    pub bias: f32,
    /// Tape transport speed configuration (7.5, 15, or 30 IPS).
    pub tape_speed: TapeSpeed,
    /// Low-frequency reproductive head bump resonance intensity (0.0 to 1.0, default 0.5).
    pub head_bump: f32,
    /// Mechanical wow and flutter modulation depth (0.0 to 1.0, default 0.15).
    pub wow_flutter: f32,
    /// Magnetic hysteresis memory loop strength (0.0 to 1.0, default 0.5).
    pub hysteresis: f32,
    /// Output makeup gain multiplier (0.1 to 4.0, default 1.0).
    pub output_gain: f32,
    /// High-frequency playback tone damping (0.0 to 1.0, default 0.8).
    pub tone: f32,
    /// Dry/Wet blend ratio (0.0 = full dry, 1.0 = full wet, default 1.0).
    pub dry_wet: f32,

    // Internal modulation & filter states (skipped from serde if needed, but simple arrays serializable)
    #[serde(skip)]
    channels: [TapeChannelState; 2],
    #[serde(skip)]
    wow_phase: f32,
    #[serde(skip)]
    flutter_phase: f32,
    #[serde(skip)]
    ac_bias_phase: f32,
    #[serde(skip)]
    sample_rate: f32,
}

impl TapeSaturationNode {
    /// Create a new `TapeSaturationNode` with specified drive and saturation parameters.
    pub fn new(drive: f32, saturation: f32) -> Self {
        let mut node = Self {
            drive: drive.max(0.1),
            saturation: saturation.clamp(0.0, 1.0),
            bias: 0.0,
            tape_speed: TapeSpeed::Ips15,
            head_bump: 0.5,
            wow_flutter: 0.15,
            hysteresis: 0.5,
            output_gain: 1.0,
            tone: 0.85,
            dry_wet: 1.0,
            channels: [TapeChannelState::default(), TapeChannelState::default()],
            wow_phase: 0.0,
            flutter_phase: 0.0,
            ac_bias_phase: 0.0,
            sample_rate: 44100.0,
        };
        node.update_filters();
        node
    }

    /// Builder pattern configuring tape operational speed.
    pub fn with_speed(mut self, speed: TapeSpeed) -> Self {
        self.tape_speed = speed;
        self.update_filters();
        self
    }

    /// Set tape operational speed and refresh filter responses.
    pub fn set_speed(&mut self, speed: TapeSpeed) {
        self.tape_speed = speed;
        self.update_filters();
    }

    /// Set sample rate and recompute filter coefficients.
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1000.0);
        self.update_filters();
    }

    /// Recompute head bump and high-frequency damping filters.
    pub fn update_filters(&mut self) {
        let fc = self.tape_speed.head_bump_freq();
        let q = self.tape_speed.head_bump_q();
        let gain_db = self.tape_speed.head_bump_gain_db() * self.head_bump;
        let sr = self.sample_rate;

        for ch in &mut self.channels {
            ch.head_bump_filter.update(fc, gain_db, q, sr);
        }
    }

    /// Reset all filter states, magnetization states, and phase accumulators.
    pub fn reset(&mut self) {
        for ch in &mut self.channels {
            ch.reset();
        }
        self.wow_phase = 0.0;
        self.flutter_phase = 0.0;
        self.ac_bias_phase = 0.0;
        self.update_filters();
    }

    /// Evaluate magnetic hysteresis transfer function for a single input value $x$.
    /// Uses continuous differential hysteresis approximation.
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn eval_hysteresis(
        drive: f32,
        saturation: f32,
        bias: f32,
        hysteresis: f32,
        speed_factor: f32,
        x: f32,
        mag_state: &mut f32,
        bias_osc: f32,
    ) -> f32 {
        let effective_drive = drive * speed_factor;
        let sat = saturation * speed_factor;

        // Total recording magnetic field H with AC bias injection and DC bias trim
        let h = x * effective_drive + (bias * 0.25) + (bias_osc * 0.05);

        // Hysteresis feedback parameter alpha and coercivity beta
        let alpha = 0.35 * hysteresis;
        let beta = 1.0 + sat * 1.5;

        // Differential rate towards target magnetization
        let diff = (h - alpha * *mag_state) / beta;
        let target_m = diff.tanh();

        // Magnetic domain realignment rate
        let rate = 0.45 + (1.0 - hysteresis) * 0.45;
        *mag_state += rate * (target_m - *mag_state);

        // Core non-linear tape flux transfer with soft-clipping saturation
        let m = *mag_state;
        let flux = m * (1.0 + sat * 0.3) - 0.12 * sat * m * m * m;

        // Normalize output level relative to drive to maintain consistent nominal gain
        flux / (1.0 + effective_drive * 0.35)
    }

    /// Evaluate magnetic hysteresis transfer function on self.
    #[inline]
    pub fn process_hysteresis_sample(
        &self,
        x: f32,
        mag_state: &mut f32,
        bias_osc: f32,
    ) -> f32 {
        Self::eval_hysteresis(
            self.drive,
            self.saturation,
            self.bias,
            self.hysteresis,
            self.tape_speed.saturation_factor(),
            x,
            mag_state,
            bias_osc,
        )
    }

    /// Generate static points for B-H magnetic hysteresis loop curve visualization.
    pub fn compute_hysteresis_curve(&self, num_points: usize) -> Vec<(f32, f32)> {
        let mut points = Vec::with_capacity(num_points);
        let mut sim_m = 0.0f32;
        let cycles = 2;
        let total_steps = num_points.max(32) * cycles;

        // Drive a sinusoidal test signal to trace full open hysteresis loop
        for step in 0..total_steps {
            let phase = (step as f32 / (num_points as f32)) * TAU;
            let h = phase.sin() * 1.2;
            let b = self.process_hysteresis_sample(h, &mut sim_m, 0.0);
            if step >= (cycles - 1) * num_points {
                points.push((h, b));
            }
        }
        points
    }

    /// Get current head bump resonant frequency in Hz.
    pub fn get_head_bump_freq(&self) -> f32 {
        self.tape_speed.head_bump_freq()
    }

    /// Estimate total harmonic distortion (THD) percentage based on drive and tape speed.
    pub fn get_thd_estimate(&self) -> f32 {
        let base_thd = (self.drive - 1.0).max(0.0) * 0.8 + self.saturation * 1.5;
        let speed_factor = match self.tape_speed {
            TapeSpeed::Ips7_5 => 1.5,
            TapeSpeed::Ips15 => 1.0,
            TapeSpeed::Ips30 => 0.6,
        };
        (base_thd * speed_factor).clamp(0.05, 12.0)
    }
}

impl Default for TapeSaturationNode {
    fn default() -> Self {
        Self::new(1.8, 0.5)
    }
}

impl SignalProcessor for TapeSaturationNode {
    fn name(&self) -> &str {
        "TapeSaturationNode"
    }

    fn process_block(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        if inputs.is_empty() || outputs.is_empty() {
            return;
        }

        let num_samples = inputs[0].len().min(outputs[0].len());
        let sr = if ctx.sample_rate > 0 {
            ctx.sample_rate as f32
        } else {
            self.sample_rate
        };

        if (self.sample_rate - sr).abs() > 1.0 {
            self.sample_rate = sr;
            self.update_filters();
        }

        let num_in_ch = inputs.len();
        let num_out_ch = outputs.len();

        // Speed-dependent high-frequency loss alpha
        let hf_cutoff = self.tape_speed.hf_loss_cutoff().min(sr * 0.48);
        let hf_alpha = (TAU * hf_cutoff / sr).clamp(0.05, 0.95);

        // Tone damping alpha
        let tone_cutoff = (2000.0 + self.tone.clamp(0.0, 1.0) * 18000.0).min(sr * 0.48);
        let tone_alpha = (TAU * tone_cutoff / sr).clamp(0.05, 0.95);

        // Phase step increments
        let wow_inc = TAU * 0.75 / sr; // 0.75 Hz wow drift
        let flutter_inc = TAU * 8.2 / sr; // 8.2 Hz flutter flutter
        let ac_bias_inc = TAU * 19200.0 / sr; // Ultrasonic AC bias simulation

        let drive = self.drive;
        let saturation = self.saturation;
        let bias = self.bias;
        let hysteresis = self.hysteresis;
        let speed_factor = self.tape_speed.saturation_factor();
        let output_gain = self.output_gain;
        let dry_wet = self.dry_wet;

        for i in 0..num_samples {
            // Update continuous phase state accumulators modulo TAU (2 * PI)
            self.wow_phase = (self.wow_phase + wow_inc) % TAU;
            self.flutter_phase = (self.flutter_phase + flutter_inc) % TAU;
            self.ac_bias_phase = (self.ac_bias_phase + ac_bias_inc) % TAU;

            // Wow & Flutter modulation time offset in samples
            let wow_mod = self.wow_phase.sin() * 1.8;
            let flutter_mod = self.flutter_phase.sin() * 0.8;
            let total_mod_samples = (wow_mod + flutter_mod) * self.wow_flutter;
            let base_delay = 16.0f32;
            let read_offset = (base_delay + total_mod_samples).clamp(1.0, (WOW_FLUTTER_BUFFER_SIZE - 4) as f32);

            let bias_osc = self.ac_bias_phase.sin();

            for ch_idx in 0..num_out_ch {
                let in_sample = if ch_idx < num_in_ch && i < inputs[ch_idx].len() {
                    inputs[ch_idx][i]
                } else if !inputs[0].is_empty() && i < inputs[0].len() {
                    inputs[0][i]
                } else {
                    0.0
                };

                let ch_state_idx = ch_idx.min(1);
                let ch_state = &mut self.channels[ch_state_idx];

                // 1. Write incoming sample to mechanical delay buffer
                let w_idx = ch_state.write_idx;
                ch_state.delay_buf[w_idx] = in_sample;
                ch_state.write_idx = (w_idx + 1) % WOW_FLUTTER_BUFFER_SIZE;

                // 2. Read interpolated delayed sample for wow/flutter pitch modulation
                let read_pos = (w_idx as f32 + WOW_FLUTTER_BUFFER_SIZE as f32 - read_offset) % (WOW_FLUTTER_BUFFER_SIZE as f32);
                let idx0 = read_pos.floor() as usize % WOW_FLUTTER_BUFFER_SIZE;
                let idx1 = (idx0 + 1) % WOW_FLUTTER_BUFFER_SIZE;
                let frac = read_pos.fract();
                let delayed_in = ch_state.delay_buf[idx0] * (1.0 - frac) + ch_state.delay_buf[idx1] * frac;

                // 3. Apply Low-Frequency Head Bump Resonance
                let bumped = ch_state.head_bump_filter.process_sample(delayed_in);

                // 4. Magnetic Tape Hysteresis & Non-Linear Saturation
                let sat_out = Self::eval_hysteresis(
                    drive,
                    saturation,
                    bias,
                    hysteresis,
                    speed_factor,
                    bumped,
                    &mut ch_state.magnetization,
                    bias_osc,
                );

                // 5. Tape Gap & Spacing High-Frequency Loss Filter
                ch_state.gap_loss_state += hf_alpha * (sat_out - ch_state.gap_loss_state);
                let gap_filtered = ch_state.gap_loss_state;

                // 6. User Tone Filter & Makeup Gain
                ch_state.tone_state += tone_alpha * (gap_filtered - ch_state.tone_state);
                let processed = ch_state.tone_state * output_gain;

                // 7. Dry/Wet Crossfade
                let final_sample = in_sample * (1.0 - dry_wet) + processed * dry_wet;

                if i < outputs[ch_idx].len() {
                    outputs[ch_idx][i] = final_sample;
                }
            }
        }
    }
}

impl AudioNode for TapeSaturationNode {
    fn name(&self) -> &str {
        "TapeSaturationNode"
    }

    fn process(
        &mut self,
        input: &[&[Sample]],
        output: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        self.process_block(input, output, ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;
    use summoner_core::allocator::AllocGuard;

    #[test]
    fn test_tape_saturation_zero_allocation_in_audio_loop() {
        let mut tape = TapeSaturationNode::new(2.5, 0.75);
        let ctx = ProcessContext::new(48000, 120.0, 0);

        let in_l = [0.45f32; 128];
        let in_r = [-0.45f32; 128];
        let mut out_l = [0.0f32; 128];
        let mut out_r = [0.0f32; 128];

        {
            let _guard = AllocGuard::new();
            tape.process_block(
                &[&in_l[..], &in_r[..]],
                &mut [&mut out_l[..], &mut out_r[..]],
                &ctx,
            );
        }

        assert!(out_l.iter().all(|s| s.is_finite()));
        assert!(out_r.iter().all(|s| s.is_finite()));
    }

    #[test]
    fn test_tape_saturation_hysteresis_loop_directionality() {
        let tape = TapeSaturationNode::new(3.0, 0.8);
        let points = tape.compute_hysteresis_curve(64);
        assert_eq!(points.len(), 64);

        // Find samples where input H is approximately 0 going up vs going down
        let mut m_rising = None;
        let mut m_falling = None;

        for i in 1..points.len() {
            let (h_prev, _) = points[i - 1];
            let (h_curr, m_curr) = points[i];

            if h_prev < 0.0 && h_curr >= 0.0 {
                m_rising = Some(m_curr);
            }
            if h_prev > 0.0 && h_curr <= 0.0 {
                m_falling = Some(m_curr);
            }
        }

        if let (Some(m_r), Some(m_f)) = (m_rising, m_falling) {
            assert!(
                (m_r - m_f).abs() > 0.01,
                "Hysteresis should cause distinct remanence magnetization depending on sweep direction: rising={}, falling={}",
                m_r,
                m_f
            );
        }
    }

    #[test]
    fn test_tape_speed_frequency_characteristics() {
        let mut tape_7_5 = TapeSaturationNode::new(1.0, 0.0).with_speed(TapeSpeed::Ips7_5);
        let mut tape_30 = TapeSaturationNode::new(1.0, 0.0).with_speed(TapeSpeed::Ips30);
        let ctx = ProcessContext::new(48000, 120.0, 0);

        assert_eq!(tape_7_5.get_head_bump_freq(), 50.0);
        assert_eq!(tape_30.get_head_bump_freq(), 120.0);

        // Process a 50Hz tone and verify 7.5 IPS exhibits higher head bump response
        let sample_rate = 48000.0;
        let mut tone_50hz = [0.0f32; 256];
        for (i, sample) in tone_50hz.iter_mut().enumerate() {
            *sample = (2.0 * PI * 50.0 * i as f32 / sample_rate).sin() * 0.2;
        }

        let mut out_7_5 = [0.0f32; 256];
        let mut out_30 = [0.0f32; 256];

        tape_7_5.process_block(&[&tone_50hz[..]], &mut [&mut out_7_5[..]], &ctx);
        tape_30.process_block(&[&tone_50hz[..]], &mut [&mut out_30[..]], &ctx);

        let rms_7_5: f32 = (out_7_5.iter().map(|s| s * s).sum::<f32>() / 256.0).sqrt();
        let rms_30: f32 = (out_30.iter().map(|s| s * s).sum::<f32>() / 256.0).sqrt();

        assert!(
            rms_7_5 > rms_30,
            "7.5 IPS should have stronger 50Hz head bump response than 30 IPS ({} vs {})",
            rms_7_5,
            rms_30
        );
    }

    #[test]
    fn test_tape_saturation_soft_limiting() {
        let mut tape = TapeSaturationNode::new(5.0, 0.9);
        let ctx = ProcessContext::new(48000, 120.0, 0);

        let loud_in = [1.5f32; 128];
        let mut out = [0.0f32; 128];

        tape.process_block(&[&loud_in[..]], &mut [&mut out[..]], &ctx);

        assert!(
            out[127].abs() <= 1.05,
            "Tape saturation should soft limit extreme input amplitudes: got {}",
            out[127]
        );
    }
}
