// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Audio Round-Trip Latency (RTL) measurement probe and recording latency compensation (Task A7).

use crate::recorder::RecordedTakeResult;

/// Result of a round-trip latency calibration measurement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RoundTripLatency {
    /// Measured latency in samples / frames (with sub-sample precision).
    pub samples: f64,
    /// Measured latency in milliseconds.
    pub milliseconds: f64,
    /// Detection confidence score (0.0 to 1.0) based on peak-to-noise ratio.
    pub confidence: f32,
}

/// Latency calibration probe signal generator.
pub struct LatencyProbeGenerator;

impl LatencyProbeGenerator {
    /// Generate a bandlimited Dirac impulse surrounded by silence.
    ///
    /// `pre_roll_frames`: silence frames before the pulse.
    /// `total_frames`: total length of the buffer.
    pub fn generate_impulse(pre_roll_frames: usize, total_frames: usize) -> Vec<f32> {
        let mut buffer = vec![0.0f32; total_frames.max(pre_roll_frames + 64)];
        if pre_roll_frames < buffer.len() {
            // Bandlimited impulse (windowed sinc pulse) to prevent harsh converter clipping
            for i in 0..16 {
                let idx = pre_roll_frames + i;
                if idx < buffer.len() {
                    let x = (i as f32 - 8.0) * 0.5;
                    let sinc = if x.abs() < 1e-4 {
                        1.0
                    } else {
                        (x * std::f32::consts::PI).sin() / (x * std::f32::consts::PI)
                    };
                    // Hann window
                    let w = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / 16.0).cos());
                    buffer[idx] = sinc * w * 0.8;
                }
            }
        }
        buffer
    }

    /// Generate an exponential chirp probe signal for high-noise loopback environments.
    pub fn generate_chirp(
        pre_roll_frames: usize,
        chirp_frames: usize,
        total_frames: usize,
        sample_rate: u32,
        f_start: f32,
        f_end: f32,
    ) -> Vec<f32> {
        let mut buffer = vec![0.0f32; total_frames.max(pre_roll_frames + chirp_frames)];
        let sr = sample_rate as f32;
        let t_end = chirp_frames as f32 / sr;
        let k = (f_end / f_start).powf(1.0 / t_end);

        for i in 0..chirp_frames {
            let idx = pre_roll_frames + i;
            if idx < buffer.len() {
                let t = i as f32 / sr;
                let phase = 2.0 * std::f32::consts::PI * f_start * (k.powf(t) - 1.0) / k.ln();
                // 5% Tukey window taper
                let taper = ((i as f32) / (chirp_frames as f32 * 0.05)).min(1.0)
                    * (((chirp_frames - 1 - i) as f32) / (chirp_frames as f32 * 0.05)).min(1.0);
                buffer[idx] = phase.sin() * taper * 0.7;
            }
        }
        buffer
    }
}

/// Latency detector using cross-correlation with sub-sample peak interpolation.
pub struct LatencyDetector;

impl LatencyDetector {
    /// Detect the round-trip delay between a reference probe and the recorded input buffer.
    ///
    /// Returns `Some(RoundTripLatency)` if a reliable correlation peak is found.
    pub fn detect(
        reference: &[f32],
        recorded: &[f32],
        sample_rate: u32,
        search_window_frames: usize,
    ) -> Option<RoundTripLatency> {
        if reference.is_empty() || recorded.len() < reference.len() {
            return None;
        }

        let ref_len = reference.len();
        let max_lag = search_window_frames.min(recorded.len() - ref_len);
        if max_lag == 0 {
            return None;
        }

        // Calculate reference energy for normalization
        let ref_energy: f64 = reference.iter().map(|&x| (x as f64) * (x as f64)).sum();
        if ref_energy <= 1e-12 {
            return None;
        }

        let mut best_lag = 0;
        let mut max_corr = 0.0f64;
        let mut correlations = vec![0.0f64; max_lag];

        for lag in 0..max_lag {
            let mut sum = 0.0f64;
            let slice = &recorded[lag..lag + ref_len];
            for (&r, &s) in reference.iter().zip(slice.iter()) {
                sum += (r as f64) * (s as f64);
            }
            correlations[lag] = sum;
            if sum > max_corr {
                max_corr = sum;
                best_lag = lag;
            }
        }

        // Normalized peak correlation coefficient
        let rec_energy: f64 = recorded[best_lag..best_lag + ref_len]
            .iter()
            .map(|&x| (x as f64) * (x as f64))
            .sum();
        let norm_coeff = if rec_energy > 1e-12 {
            (max_corr / (ref_energy * rec_energy).sqrt()).clamp(0.0, 1.0) as f32
        } else {
            0.0
        };

        if norm_coeff < 0.15 {
            // Correlation peak is too weak / noisy
            return None;
        }

        // Sub-sample parabolic interpolation around peak
        let fractional_lag = if best_lag > 0 && best_lag + 1 < max_lag {
            let y_left = correlations[best_lag - 1];
            let y_mid = correlations[best_lag];
            let y_right = correlations[best_lag + 1];
            let denom = 2.0 * (2.0 * y_mid - y_left - y_right);
            if denom.abs() > 1e-12 {
                let delta = (y_left - y_right) / denom;
                best_lag as f64 + delta.clamp(-0.5, 0.5)
            } else {
                best_lag as f64
            }
        } else {
            best_lag as f64
        };

        let ms = (fractional_lag / sample_rate as f64) * 1000.0;

        Some(RoundTripLatency {
            samples: fractional_lag,
            milliseconds: ms,
            confidence: norm_coeff,
        })
    }
}

/// Recording latency compensator.
///
/// Automatically shifts newly recorded audio clips backwards in time to account for
/// DAC output buffering, ADC input buffering, hardware converter delay, and OS driver latency.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RecordingLatencyCompensation {
    /// Measured round-trip latency in samples.
    pub rtl_samples: u32,
    /// Optional manual user offset in samples (positive = shift earlier, negative = shift later).
    pub manual_offset_samples: i32,
    /// Whether recording latency compensation is actively enabled.
    pub enabled: bool,
}

impl Default for RecordingLatencyCompensation {
    fn default() -> Self {
        Self {
            rtl_samples: 0,
            manual_offset_samples: 0,
            enabled: true,
        }
    }
}

impl RecordingLatencyCompensation {
    /// Create with specific RTL in samples.
    pub fn new(rtl_samples: u32) -> Self {
        Self {
            rtl_samples,
            manual_offset_samples: 0,
            enabled: true,
        }
    }

    /// Total effective offset in samples to subtract from recording start time.
    #[inline]
    pub fn effective_offset_samples(&self) -> i64 {
        if !self.enabled {
            0
        } else {
            self.rtl_samples as i64 + self.manual_offset_samples as i64
        }
    }

    /// Compensate raw start beat by shifting backwards by the latency duration.
    pub fn compensate_start_beat(&self, raw_start_beat: f64, sample_rate: u32, bpm: f64) -> f64 {
        let offset = self.effective_offset_samples();
        if offset == 0 || sample_rate == 0 || bpm <= 0.0 {
            return raw_start_beat;
        }

        let frames_per_beat = (sample_rate as f64 * 60.0) / bpm;
        let beat_offset = (offset as f64) / frames_per_beat;
        (raw_start_beat - beat_offset).max(0.0)
    }

    /// Compensate recorded take result into an adjusted take result.
    pub fn compensate_take(&self, take: &RecordedTakeResult, bpm: f64) -> RecordedTakeResult {
        let mut compensated = take.clone();
        let offset = self.effective_offset_samples();
        if offset > 0 {
            let offset_u64 = offset as u64;
            compensated.start_frame = compensated.start_frame.saturating_sub(offset_u64);
        } else if offset < 0 {
            compensated.start_frame = compensated.start_frame.saturating_add((-offset) as u64);
        }
        compensated.start_beat = self.compensate_start_beat(take.start_beat, take.sample_rate, bpm);
        compensated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dirac_impulse_generation() {
        let impulse = LatencyProbeGenerator::generate_impulse(128, 512);
        assert_eq!(impulse.len(), 512);
        assert_eq!(impulse[0], 0.0);
        assert_eq!(impulse[127], 0.0);
        // Pulse exists around index 128..144
        let max_val = impulse.iter().cloned().fold(0.0f32, f32::max);
        assert!(max_val > 0.5, "Impulse should have significant peak");
    }

    #[test]
    fn test_detect_known_delay() {
        let sample_rate = 48000;
        let probe = LatencyProbeGenerator::generate_impulse(64, 256);
        let synthetic_delay = 320; // 320 samples latency

        let mut recorded = vec![0.0f32; 1024];
        // Inject probe delayed by synthetic_delay
        for (i, &sample) in probe.iter().enumerate() {
            if synthetic_delay + i < recorded.len() {
                recorded[synthetic_delay + i] = sample;
            }
        }

        let result = LatencyDetector::detect(&probe, &recorded, sample_rate, 600);
        assert!(result.is_some(), "Should find correlation peak");
        let rtl = result.unwrap();
        assert!(
            (rtl.samples - synthetic_delay as f64).abs() < 0.1,
            "Expected ~320 samples, got {}",
            rtl.samples
        );
        let expected_ms = (synthetic_delay as f64 / 48000.0) * 1000.0;
        assert!((rtl.milliseconds - expected_ms).abs() < 0.01);
        assert!(rtl.confidence > 0.95);
    }

    #[test]
    fn test_recording_latency_compensation_math() {
        let comp = RecordingLatencyCompensation {
            rtl_samples: 480, // at 48kHz, 480 samples = 10ms
            manual_offset_samples: 20,
            enabled: true,
        };
        assert_eq!(comp.effective_offset_samples(), 500);

        // At 120 BPM, 1 beat = 0.5s = 24000 frames at 48kHz.
        // 500 frames / 24000 frames = 0.020833 beats.
        let raw_beat = 4.0;
        let compensated_beat = comp.compensate_start_beat(raw_beat, 48000, 120.0);
        assert!((compensated_beat - (4.0 - 500.0 / 24000.0)).abs() < 1e-6);
    }
}
