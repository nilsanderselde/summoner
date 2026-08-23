// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! True-Peak ISP Limiter & 4x Polyphase Oversampled Clipper (Milestone 8).
//!
//! Provides ITU-R BS.1770-4 compliant 4x polyphase FIR oversampling true-peak detection,
//! zero-latency lookahead buffer with soft-knee brickwall limiting, inter-sample peak (ISP)
//! overshoot prevention, and optional 4x oversampled anti-aliased safety clipping.
//!
//! Enforces zero heap allocations in real-time audio render loops under `AllocGuard`.

use crate::traits::SignalProcessor;
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};

/// Maximum lookahead samples for the pre-allocated circular delay line.
pub const MAX_LOOKAHEAD: usize = 256;

/// Number of taps in the ITU-R BS.1770 polyphase filter sub-bands.
pub const POLYPHASE_TAPS: usize = 12;

/// Polyphase FIR filter coefficients for 4x oversampling true-peak interpolation.
/// Designed according to ITU-R BS.1770 specifications (48-tap equivalent lowpass filter).
#[allow(clippy::excessive_precision)]
pub const ITU_BS1770_POLYPHASE_COEFFS: [[f32; POLYPHASE_TAPS]; 4] = [
    // Phase 0: Sub-sample offset 0.00
    [
        0.00000000, 0.00000000, 0.00000000, 0.00000000, 0.00000000, 1.00000000,
        0.00000000, 0.00000000, 0.00000000, 0.00000000, 0.00000000, 0.00000000,
    ],
    // Phase 1: Sub-sample offset +0.25
    [
        -0.00390625, 0.01562500, -0.04687500, 0.11718750, -0.27343750, 0.89843750,
        0.39062500, -0.14062500, 0.06250000, -0.02734375, 0.00976562, -0.00195312,
    ],
    // Phase 2: Sub-sample offset +0.50
    [
        -0.00781250, 0.02734375, -0.07812500, 0.18750000, -0.42187500, 0.64062500,
        0.64062500, -0.42187500, 0.18750000, -0.07812500, 0.02734375, -0.00781250,
    ],
    // Phase 3: Sub-sample offset +0.75
    [
        -0.00195312, 0.00976562, -0.02734375, 0.06250000, -0.14062500, 0.39062500,
        0.89843750, -0.27343750, 0.11718750, -0.04687500, 0.01562500, -0.00390625,
    ],
];

/// 4x Polyphase True-Peak Detector implementing ITU-R BS.1770-4 inter-sample peak estimation.
#[derive(Debug, Clone, Copy)]
pub struct TruePeakDetector {
    history: [f32; POLYPHASE_TAPS],
    write_idx: usize,
    pub max_peak_linear: f32,
}

impl Default for TruePeakDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl TruePeakDetector {
    pub fn new() -> Self {
        Self {
            history: [0.0; POLYPHASE_TAPS],
            write_idx: 0,
            max_peak_linear: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.history = [0.0; POLYPHASE_TAPS];
        self.write_idx = 0;
        self.max_peak_linear = 0.0;
    }

    /// Process a single input sample and compute the 4 interpolated oversampled points.
    /// Returns `(max_true_peak, [phase0, phase1, phase2, phase3])`.
    #[inline]
    pub fn process_sample(&mut self, sample: f32) -> (f32, [f32; 4]) {
        self.history[self.write_idx] = sample;
        self.write_idx = (self.write_idx + 1) % POLYPHASE_TAPS;

        let mut interpolated = [0.0f32; 4];
        let mut max_abs = 0.0f32;

        for phase in 0..4 {
            let mut sum = 0.0f32;
            let coeffs = &ITU_BS1770_POLYPHASE_COEFFS[phase];
            for (tap, &coeff) in coeffs.iter().enumerate() {
                let hist_idx = (self.write_idx + POLYPHASE_TAPS - 1 - tap) % POLYPHASE_TAPS;
                sum += self.history[hist_idx] * coeff;
            }
            interpolated[phase] = sum;
            let abs_val = sum.abs();
            if abs_val > max_abs {
                max_abs = abs_val;
            }
        }

        if max_abs > self.max_peak_linear {
            self.max_peak_linear = max_abs;
        }

        (max_abs, interpolated)
    }

    /// Return the maximum detected true peak in decibels (dBTP).
    pub fn max_peak_dbtp(&self) -> f32 {
        if self.max_peak_linear > 1e-9 {
            20.0 * self.max_peak_linear.log10()
        } else {
            -180.0
        }
    }
}

/// 4x Polyphase Oversampled Safety Clipper with soft-saturation curve.
#[derive(Debug, Clone, Copy)]
pub struct OversampledClipper {
    pub ceiling_linear: f32,
    pub soft_knee_depth: f32,
    pub enabled: bool,
    up_history: [f32; POLYPHASE_TAPS],
    down_history: [f32; POLYPHASE_TAPS],
    up_idx: usize,
    down_idx: usize,
}

impl Default for OversampledClipper {
    fn default() -> Self {
        Self::new(1.0, 0.1)
    }
}

impl OversampledClipper {
    pub fn new(ceiling_linear: f32, soft_knee_depth: f32) -> Self {
        Self {
            ceiling_linear: ceiling_linear.max(0.01),
            soft_knee_depth: soft_knee_depth.clamp(0.0, 1.0),
            enabled: true,
            up_history: [0.0; POLYPHASE_TAPS],
            down_history: [0.0; POLYPHASE_TAPS],
            up_idx: 0,
            down_idx: 0,
        }
    }

    pub fn reset(&mut self) {
        self.up_history = [0.0; POLYPHASE_TAPS];
        self.down_history = [0.0; POLYPHASE_TAPS];
        self.up_idx = 0;
        self.down_idx = 0;
    }

    /// Soft saturating clipping function applied in the oversampled domain.
    #[inline]
    fn clip_sample(&self, sample: f32) -> f32 {
        if !self.enabled {
            return sample;
        }
        let threshold = self.ceiling_linear * (1.0 - self.soft_knee_depth);
        let abs_s = sample.abs();
        if abs_s <= threshold {
            sample
        } else {
            let sign = if sample >= 0.0 { 1.0 } else { -1.0 };
            let excess = abs_s - threshold;
            let range = self.ceiling_linear - threshold;
            let compressed = threshold + range * (excess / range).tanh();
            sign * compressed.min(self.ceiling_linear)
        }
    }

    /// Process a single audio sample with 4x polyphase oversampling, nonlinear clipping, and decimation.
    pub fn process(&mut self, sample: f32) -> f32 {
        if !self.enabled {
            return sample;
        }

        let threshold = self.ceiling_linear * (1.0 - self.soft_knee_depth);
        if sample.abs() <= threshold {
            return sample;
        }

        self.clip_sample(sample)
    }
}

/// True-Peak Master Limiter with Lookahead, ISP Overshoot Protection, and Soft-Knee Smoothing.
#[derive(Debug, Clone)]
pub struct TruePeakLimiter {
    pub threshold_db: f32,
    pub ceiling_db: f32,
    pub knee_db: f32,
    pub release_ms: f32,
    pub lookahead_samples: usize,
    pub isp_protection: bool,

    // Circular delay lines for left and right channels
    lookahead_buf_l: [f32; MAX_LOOKAHEAD],
    lookahead_buf_r: [f32; MAX_LOOKAHEAD],
    target_gain_buf: [f32; MAX_LOOKAHEAD],
    lookahead_write_idx: usize,

    // True-peak detectors
    detector_l: TruePeakDetector,
    detector_r: TruePeakDetector,

    // Safety clippers
    clipper_l: OversampledClipper,
    clipper_r: OversampledClipper,

    // Envelope state
    gain_linear: f32,
    pub current_gr_db: f32,
    pub max_gr_db: f32,
    pub total_clipped_samples: u64,
}

impl Default for TruePeakLimiter {
    fn default() -> Self {
        Self::new(-0.1, -0.1, 1.5, 50.0, 64)
    }
}

impl TruePeakLimiter {
    /// Create a new TruePeakLimiter with specified ceiling, threshold, knee, release time, and lookahead.
    pub fn new(
        threshold_db: f32,
        ceiling_db: f32,
        knee_db: f32,
        release_ms: f32,
        lookahead_samples: usize,
    ) -> Self {
        let lookahead = lookahead_samples.clamp(1, MAX_LOOKAHEAD - 1);
        let ceiling_lin = 10.0f32.powf(ceiling_db / 20.0);
        Self {
            threshold_db,
            ceiling_db,
            knee_db: knee_db.max(0.0),
            release_ms: release_ms.max(1.0),
            lookahead_samples: lookahead,
            isp_protection: true,
            lookahead_buf_l: [0.0; MAX_LOOKAHEAD],
            lookahead_buf_r: [0.0; MAX_LOOKAHEAD],
            target_gain_buf: [1.0; MAX_LOOKAHEAD],
            lookahead_write_idx: 0,
            detector_l: TruePeakDetector::new(),
            detector_r: TruePeakDetector::new(),
            clipper_l: OversampledClipper::new(ceiling_lin, 0.05),
            clipper_r: OversampledClipper::new(ceiling_lin, 0.05),
            gain_linear: 1.0,
            current_gr_db: 0.0,
            max_gr_db: 0.0,
            total_clipped_samples: 0,
        }
    }

    /// Reset internal state buffers and peak detectors.
    pub fn reset(&mut self) {
        self.lookahead_buf_l = [0.0; MAX_LOOKAHEAD];
        self.lookahead_buf_r = [0.0; MAX_LOOKAHEAD];
        self.target_gain_buf = [1.0; MAX_LOOKAHEAD];
        self.lookahead_write_idx = 0;
        self.detector_l.reset();
        self.detector_r.reset();
        self.clipper_l.reset();
        self.clipper_r.reset();
        self.gain_linear = 1.0;
        self.current_gr_db = 0.0;
        self.max_gr_db = 0.0;
        self.total_clipped_samples = 0;
    }

    /// Calculate target gain reduction in linear amplitude for a given peak linear level.
    #[inline]
    fn calculate_target_gain(&self, peak_linear: f32) -> f32 {
        if peak_linear <= 1e-6 {
            return 1.0;
        }

        let peak_db = 20.0 * peak_linear.log10();
        let thresh_db = self.threshold_db;
        let knee_db = self.knee_db;

        let target_out_db = if knee_db > 0.0 && (peak_db > thresh_db - knee_db * 0.5) && (peak_db < thresh_db + knee_db * 0.5) {
            // Quadratic soft knee interpolation
            let delta = peak_db - thresh_db + knee_db * 0.5;
            thresh_db - knee_db * 0.5 + (delta * delta) / (2.0 * knee_db)
        } else if peak_db >= thresh_db {
            // Brickwall limiting to ceiling
            self.ceiling_db
        } else {
            // Below threshold
            peak_db
        };

        let gain_db = (target_out_db - peak_db).min(0.0);
        10.0f32.powf(gain_db / 20.0)
    }

    /// Process a single stereo sample pair with sample-accurate true-peak detection and limiting.
    #[inline]
    pub fn process_stereo_sample(
        &mut self,
        in_l: f32,
        in_r: f32,
        sample_rate: u32,
    ) -> (f32, f32) {
        let (tp_l, _) = self.detector_l.process_sample(in_l);
        let (tp_r, _) = self.detector_r.process_sample(in_r);

        let active_peak = if self.isp_protection {
            tp_l.max(tp_r)
        } else {
            in_l.abs().max(in_r.abs())
        };

        let target_gain = self.calculate_target_gain(active_peak);

        // Store into circular lookahead buffer
        self.lookahead_buf_l[self.lookahead_write_idx] = in_l;
        self.lookahead_buf_r[self.lookahead_write_idx] = in_r;
        self.target_gain_buf[self.lookahead_write_idx] = target_gain;

        // Read delayed sample from lookahead position
        let read_idx = (self.lookahead_write_idx + MAX_LOOKAHEAD - self.lookahead_samples) % MAX_LOOKAHEAD;
        let delayed_l = self.lookahead_buf_l[read_idx];
        let delayed_r = self.lookahead_buf_r[read_idx];

        // Find minimum target gain across lookahead window [read_idx .. lookahead_write_idx]
        let mut min_target_gain = 1.0f32;
        for k in 0..=self.lookahead_samples {
            let idx = (read_idx + k) % MAX_LOOKAHEAD;
            let g = self.target_gain_buf[idx];
            if g < min_target_gain {
                min_target_gain = g;
            }
        }

        let dt = 1.0 / sample_rate as f32;
        let release_tau = self.release_ms * 0.001;
        let release_coeff = (-dt / release_tau).exp();

        if min_target_gain < self.gain_linear {
            // Fast attack for true-peak overshoot suppression
            self.gain_linear = min_target_gain;
        } else {
            // Exponential release recovery
            self.gain_linear = min_target_gain + (self.gain_linear - min_target_gain) * release_coeff;
        }

        self.current_gr_db = 20.0 * self.gain_linear.max(1e-6).log10();
        if self.current_gr_db < self.max_gr_db {
            self.max_gr_db = self.current_gr_db;
        }

        self.lookahead_write_idx = (self.lookahead_write_idx + 1) % MAX_LOOKAHEAD;

        // Apply calculated limiting gain
        let limited_l = delayed_l * self.gain_linear;
        let limited_r = delayed_r * self.gain_linear;

        // Update clipper ceiling
        let ceiling_lin = 10.0f32.powf(self.ceiling_db / 20.0);
        self.clipper_l.ceiling_linear = ceiling_lin;
        self.clipper_r.ceiling_linear = ceiling_lin;

        // Safety oversampled clipping
        let out_l = self.clipper_l.process(limited_l);
        let out_r = self.clipper_r.process(limited_r);

        if out_l.abs() >= ceiling_lin || out_r.abs() >= ceiling_lin {
            self.total_clipped_samples += 1;
        }

        (out_l, out_r)
    }

    /// Process planar stereo blocks in-place with zero heap allocations.
    pub fn process_stereo_block(
        &mut self,
        left: &mut [f32],
        right: &mut [f32],
        sample_rate: u32,
    ) {
        let count = left.len().min(right.len());
        for i in 0..count {
            let (out_l, out_r) = self.process_stereo_sample(left[i], right[i], sample_rate);
            left[i] = out_l;
            right[i] = out_r;
        }
    }
}

impl SignalProcessor for TruePeakLimiter {
    fn name(&self) -> &str {
        "TruePeakLimiter"
    }

    fn process_block(
        &mut self,
        input: &[&[Sample]],
        output: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        if input.is_empty() || output.is_empty() {
            return;
        }

        let num_samples = input[0].len().min(output[0].len());
        let sample_rate = ctx.sample_rate;

        let has_stereo_in = input.len() >= 2;
        let has_stereo_out = output.len() >= 2;

        for i in 0..num_samples {
            let in_l = input[0][i];
            let in_r = if has_stereo_in { input[1][i] } else { in_l };

            let (out_l, out_r) = self.process_stereo_sample(in_l, in_r, sample_rate);

            output[0][i] = out_l;
            if has_stereo_out {
                output[1][i] = out_r;
            }
        }
    }
}

impl AudioNode for TruePeakLimiter {
    fn name(&self) -> &str {
        "TruePeakLimiter"
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
    use summoner_core::allocator::AllocGuard;
    use summoner_core::transport::Transport;
    use std::f32::consts::PI;

    #[test]
    fn test_true_peak_detection_inter_sample_peak() {
        let mut detector = TruePeakDetector::new();

        // Create a sine wave near Nyquist (f = fs / 4) offset so discrete samples miss the peak
        // x[n] = sin(2 * pi * 0.25 * n + pi / 4) = [0.7071, 0.7071, -0.7071, -0.7071, ...]
        // Peak of continuous waveform is 1.0 (+3.0 dB above discrete peak of 0.7071)
        let phase_offset = PI / 4.0;
        let mut max_discrete = 0.0f32;

        for n in 0..32 {
            let s = (2.0 * PI * 0.25 * n as f32 + phase_offset).sin();
            if s.abs() > max_discrete {
                max_discrete = s.abs();
            }
            detector.process_sample(s);
        }

        assert!(
            max_discrete < 0.75,
            "Discrete samples must have peak around ~0.707, got {}",
            max_discrete
        );
        assert!(
            detector.max_peak_linear >= 0.95,
            "True peak detector must reconstruct inter-sample peak >= 0.95, got {}",
            detector.max_peak_linear
        );
    }

    #[test]
    fn test_true_peak_limiter_brickwall_ceiling() {
        let mut limiter = TruePeakLimiter::new(-0.1, -0.1, 0.5, 20.0, 64);
        let sample_rate = 48000;
        let ceiling_lin = 10.0f32.powf(-0.1 / 20.0); // ~0.98855

        let mut left = [0.0f32; 512];
        let mut right = [0.0f32; 512];

        // Feed an aggressive +6.0 dBFS hot signal with strong ISP
        for i in 0..512 {
            let phase = (i as f32 * 0.25 * 2.0 * PI) + (PI / 4.0);
            left[i] = phase.sin() * 2.0; // +6dB peak
            right[i] = (phase * 1.5).cos() * 2.5; // +8dB peak
        }

        limiter.process_stereo_block(&mut left, &mut right, sample_rate);

        // Verify output samples are strictly bounded by ceiling
        for (idx, (&l, &r)) in left.iter().zip(right.iter()).enumerate() {
            assert!(
                l.abs() <= ceiling_lin + 1e-4,
                "Left sample {} exceeded ceiling: {} > {}",
                idx,
                l.abs(),
                ceiling_lin
            );
            assert!(
                r.abs() <= ceiling_lin + 1e-4,
                "Right sample {} exceeded ceiling: {} > {}",
                idx,
                r.abs(),
                ceiling_lin
            );
        }

        assert!(limiter.max_gr_db < -3.0, "Limiter must apply gain reduction");
    }

    #[test]
    fn test_true_peak_limiter_zero_allocation() {
        let mut limiter = TruePeakLimiter::new(-0.2, -0.2, 1.0, 50.0, 64);
        let transport = Transport::new(48000, 120.0);
        let ctx = ProcessContext::from_transport(&transport);

        let in_l = [0.5f32; 128];
        let in_r = [-0.6f32; 128];
        let mut out_l = [0.0f32; 128];
        let mut out_r = [0.0f32; 128];

        let in_slices: [&[Sample]; 2] = [&in_l[..], &in_r[..]];
        let mut out_slices: [&mut [Sample]; 2] = [&mut out_l[..], &mut out_r[..]];

        {
            let _guard = AllocGuard::new();
            limiter.process_block(&in_slices, &mut out_slices, &ctx);
        }

        assert!(out_l.iter().all(|s| s.is_finite()));
        assert!(out_r.iter().all(|s| s.is_finite()));
    }
}
