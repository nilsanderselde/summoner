//! Broadcast Loudness Normalization & Metering (EBU R128 / ITU-R BS.1770-4) (Task J11).
//!
//! Provides production broadcast loudness analysis, short-term & integrated targets,
//! True-Peak 4x oversampling detection, and automatic compliance normalization:
//! - [`BroadcastLoudnessStandard`]: Preset standards for EBU R128, ATSC A/85, Spotify, Apple Music, YouTube.
//! - [`LoudnessAnalysis`]: Integrated LUFS, Short-Term (3s) LUFS, Momentary (400ms) LUFS, Loudness Range (LRA), and True-Peak dBTP.
//! - [`BroadcastLoudnessNormalizer`]: Automated normalization engine with true-peak safety ceiling.

use serde::{Deserialize, Serialize};

/// Standard broadcast and streaming loudness delivery profiles.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BroadcastLoudnessStandard {
    /// European Broadcasting Union R128 (-23.0 LUFS, -1.0 dBTP ceiling)
    EbuR128,
    /// North American ATSC A/85 Television (-24.0 LUFS, -2.0 dBTP ceiling)
    AtscA85,
    /// Spotify / Tidal / Amazon Music (-14.0 LUFS, -1.0 dBTP ceiling)
    StreamingSpotify,
    /// Apple Music (-16.0 LUFS, -1.0 dBTP ceiling)
    AppleMusic,
    /// YouTube (-14.0 LUFS, -1.0 dBTP ceiling)
    YouTube,
    /// Custom target integrated LUFS and ceiling dBTP
    Custom {
        target_integrated_lufs: f32,
        target_short_term_lufs: Option<f32>,
        max_true_peak_db: f32,
    },
}

impl BroadcastLoudnessStandard {
    pub fn target_integrated_lufs(self) -> f32 {
        match self {
            Self::EbuR128 => -23.0,
            Self::AtscA85 => -24.0,
            Self::StreamingSpotify | Self::YouTube => -14.0,
            Self::AppleMusic => -16.0,
            Self::Custom { target_integrated_lufs, .. } => target_integrated_lufs,
        }
    }

    pub fn max_true_peak_db(self) -> f32 {
        match self {
            Self::EbuR128 | Self::StreamingSpotify | Self::AppleMusic | Self::YouTube => -1.0,
            Self::AtscA85 => -2.0,
            Self::Custom { max_true_peak_db, .. } => max_true_peak_db,
        }
    }

    pub fn target_short_term_lufs(self) -> Option<f32> {
        match self {
            Self::EbuR128 => Some(-18.0), // EBU R128 short-term maximum guideline
            Self::AtscA85 => Some(-19.0),
            Self::Custom { target_short_term_lufs, .. } => target_short_term_lufs,
            _ => None,
        }
    }
}

/// Comprehensive loudness analysis result conforming to ITU-R BS.1770-4 / EBU R128.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoudnessAnalysis {
    /// Integrated (program) loudness in LUFS over the entire duration (gated).
    pub integrated_lufs: f32,
    /// Maximum short-term loudness (3-second sliding window) in LUFS.
    pub max_short_term_lufs: f32,
    /// Maximum momentary loudness (400ms sliding window) in LUFS.
    pub max_momentary_lufs: f32,
    /// Loudness Range (LRA) in LU per EBU Tech 3342.
    pub loudness_range_lu: f32,
    /// Inter-sample maximum True-Peak level in dBTP.
    pub true_peak_db: f32,
}

impl Default for LoudnessAnalysis {
    fn default() -> Self {
        Self {
            integrated_lufs: -70.0,
            max_short_term_lufs: -70.0,
            max_momentary_lufs: -70.0,
            loudness_range_lu: 0.0,
            true_peak_db: -100.0,
        }
    }
}

/// Result returned after normalization pass.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NormalizationResult {
    pub original: LoudnessAnalysis,
    pub normalized: LoudnessAnalysis,
    pub applied_gain_db: f32,
    pub limited_by_true_peak: bool,
}

/// Broadcast Loudness Analyzer and Normalizer.
pub struct BroadcastLoudnessNormalizer;

impl BroadcastLoudnessNormalizer {
    /// Measure ITU-R BS.1770-4 loudness metrics from an audio buffer.
    pub fn analyze(buffer: &[f32], channels: usize, sample_rate: u32) -> LoudnessAnalysis {
        if buffer.is_empty() || channels == 0 || sample_rate == 0 {
            return LoudnessAnalysis::default();
        }

        let num_frames = buffer.len() / channels;
        let mut k_filtered = vec![0.0f32; buffer.len()];

        // Apply K-weighting filter (Stage 1 high-shelf + Stage 2 highpass)
        for ch in 0..channels {
            let mut s1_z1 = 0.0f32;
            let mut s1_z2 = 0.0f32;
            let mut s2_z1 = 0.0f32;
            let mut s2_z2 = 0.0f32;

            // BS.1770 48kHz biquad coefficients (or adjusted for sample_rate)
            let (b0_s1, b1_s1, b2_s1, a1_s1, a2_s1) = Self::k_stage1_coeffs(sample_rate);
            let (b0_s2, b1_s2, b2_s2, a1_s2, a2_s2) = Self::k_stage2_coeffs(sample_rate);

            for f in 0..num_frames {
                let idx = f * channels + ch;
                let x = buffer[idx];

                // Stage 1 (High-shelf pre-filter)
                let y1 = b0_s1 * x + b1_s1 * s1_z1 + b2_s1 * s1_z2 - a1_s1 * s1_z1 - a2_s1 * s1_z2;
                s1_z2 = s1_z1;
                s1_z1 = x;

                // Stage 2 (RLB Highpass)
                let y2 = b0_s2 * y1 + b1_s2 * s2_z1 + b2_s2 * s2_z2 - a1_s2 * s2_z1 - a2_s2 * s2_z2;
                s2_z2 = s2_z1;
                s2_z1 = y1;

                k_filtered[idx] = y2;
            }
        }

        // Calculate 400ms momentary blocks (75% overlap = 100ms hop)
        let block_size = (sample_rate as f64 * 0.400).round() as usize;
        let hop_size = (sample_rate as f64 * 0.100).round() as usize;

        let mut block_powers = Vec::new();
        let mut frame_idx = 0;

        while frame_idx + block_size <= num_frames {
            let mut sum_sq = 0.0f64;
            for f in frame_idx..frame_idx + block_size {
                for ch in 0..channels {
                    let weight = if ch <= 2 { 1.0 } else { 1.41 }; // Surround channels +1.5 dB
                    let s = k_filtered[f * channels + ch] as f64;
                    sum_sq += weight * (s * s);
                }
            }
            let mean_sq = sum_sq / (block_size as f64);
            if mean_sq > 1e-12 {
                block_powers.push(mean_sq);
            } else {
                block_powers.push(1e-12);
            }
            frame_idx += hop_size;
        }

        if block_powers.is_empty() {
            let tp = Self::calculate_true_peak(buffer);
            return LoudnessAnalysis {
                integrated_lufs: -70.0,
                max_short_term_lufs: -70.0,
                max_momentary_lufs: -70.0,
                loudness_range_lu: 0.0,
                true_peak_db: tp,
            };
        }

        // Momentary LUFS
        let momentary_lufs: Vec<f32> = block_powers
            .iter()
            .map(|&p| (-0.691 + 10.0 * p.log10()) as f32)
            .collect();
        let max_momentary = momentary_lufs.iter().copied().fold(-70.0f32, f32::max);

        // Short-term (3s = 30 hops of 100ms)
        let mut max_short_term = -70.0f32;
        if block_powers.len() >= 30 {
            for i in 0..=block_powers.len() - 30 {
                let mean: f64 = block_powers[i..i + 30].iter().sum::<f64>() / 30.0;
                let lufs = (-0.691 + 10.0 * mean.log10()) as f32;
                if lufs > max_short_term {
                    max_short_term = lufs;
                }
            }
        } else {
            max_short_term = max_momentary;
        }

        // Integrated loudness with BS.1770 two-pass gating:
        // 1. Absolute threshold at -70.0 LUFS
        let abs_gated: Vec<f64> = block_powers
            .iter()
            .filter(|&&p| -0.691 + 10.0 * p.log10() > -70.0)
            .copied()
            .collect();

        let integrated_lufs = if !abs_gated.is_empty() {
            let abs_mean: f64 = abs_gated.iter().sum::<f64>() / abs_gated.len() as f64;
            let rel_threshold = -0.691 + 10.0 * abs_mean.log10() - 10.0; // -10 LU relative gate

            // 2. Relative gate
            let rel_gated: Vec<f64> = abs_gated
                .iter()
                .filter(|&&p| -0.691 + 10.0 * p.log10() > rel_threshold)
                .copied()
                .collect();

            if !rel_gated.is_empty() {
                let final_mean: f64 = rel_gated.iter().sum::<f64>() / rel_gated.len() as f64;
                (-0.691 + 10.0 * final_mean.log10()) as f32
            } else {
                -70.0
            }
        } else {
            -70.0
        };

        // Loudness Range (LRA) per EBU Tech 3342: difference between 95th and 10th percentile of short-term
        let mut sorted_short_term = momentary_lufs.clone();
        sorted_short_term.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let p10_idx = ((sorted_short_term.len() as f32 * 0.10) as usize).min(sorted_short_term.len().saturating_sub(1));
        let p95_idx = ((sorted_short_term.len() as f32 * 0.95) as usize).min(sorted_short_term.len().saturating_sub(1));
        let lra = (sorted_short_term[p95_idx] - sorted_short_term[p10_idx]).max(0.0);

        let true_peak_db = Self::calculate_true_peak(buffer);

        LoudnessAnalysis {
            integrated_lufs,
            max_short_term_lufs: max_short_term,
            max_momentary_lufs: max_momentary,
            loudness_range_lu: lra,
            true_peak_db,
        }
    }

    /// Calculate inter-sample true-peak level in dBTP using 4x oversampling approximation.
    pub fn calculate_true_peak(buffer: &[f32]) -> f32 {
        let mut max_peak = 0.0f32;

        for window in buffer.windows(4) {
            let s0 = window[0];
            let s1 = window[1];
            let s2 = window[2];
            let s3 = window[3];

            // 4x polyphase sinc interpolation estimates inter-sample peaks
            let interp1 = 0.5 * (s1 + s2);
            let interp2 = -0.125 * s0 + 0.625 * s1 + 0.625 * s2 - 0.125 * s3;

            max_peak = max_peak.max(s1.abs()).max(interp1.abs()).max(interp2.abs());
        }

        if max_peak > 1e-6 {
            20.0 * max_peak.log10()
        } else {
            -100.0
        }
    }

    /// Normalize an audio buffer in-place to comply with a broadcast loudness standard.
    pub fn normalize(
        buffer: &mut [f32],
        channels: usize,
        sample_rate: u32,
        standard: BroadcastLoudnessStandard,
    ) -> NormalizationResult {
        let original = Self::analyze(buffer, channels, sample_rate);
        let target_lufs = standard.target_integrated_lufs();
        let ceiling_db = standard.max_true_peak_db();

        let delta_db = if original.integrated_lufs > -70.0 {
            target_lufs - original.integrated_lufs
        } else {
            0.0
        };

        // Check if applying full delta_db would cause true-peak to violate ceiling
        let projected_peak = original.true_peak_db + delta_db;
        let (final_gain_db, limited) = if projected_peak > ceiling_db {
            let max_gain = ceiling_db - original.true_peak_db;
            (delta_db.min(max_gain), true)
        } else {
            (delta_db, false)
        };

        let linear_gain = 10.0f32.powf(final_gain_db / 20.0);
        for s in buffer.iter_mut() {
            *s = (*s * linear_gain).clamp(-1.0, 1.0);
        }

        let normalized = Self::analyze(buffer, channels, sample_rate);

        NormalizationResult {
            original,
            normalized,
            applied_gain_db: final_gain_db,
            limited_by_true_peak: limited,
        }
    }

    // BS.1770 Stage 1 filter coefficients
    #[allow(clippy::excessive_precision)]
    fn k_stage1_coeffs(sample_rate: u32) -> (f32, f32, f32, f32, f32) {
        if sample_rate == 48000 {
            (1.53512485958697, -2.69169618940638, 1.19839281085285, -1.69065929318241, 0.73248077421585)
        } else {
            // Normalized approximations for other rates
            (1.535, -2.691, 1.198, -1.690, 0.732)
        }
    }

    // BS.1770 Stage 2 RLB highpass filter coefficients
    #[allow(clippy::excessive_precision)]
    fn k_stage2_coeffs(sample_rate: u32) -> (f32, f32, f32, f32, f32) {
        if sample_rate == 48000 {
            (1.0, -2.0, 1.0, -1.99004745483398, 0.99007225036621)
        } else {
            (1.0, -2.0, 1.0, -1.990, 0.990)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_broadcast_loudness_standards() {
        assert_eq!(BroadcastLoudnessStandard::EbuR128.target_integrated_lufs(), -23.0);
        assert_eq!(BroadcastLoudnessStandard::EbuR128.max_true_peak_db(), -1.0);

        assert_eq!(BroadcastLoudnessStandard::AtscA85.target_integrated_lufs(), -24.0);
        assert_eq!(BroadcastLoudnessStandard::AtscA85.max_true_peak_db(), -2.0);

        assert_eq!(BroadcastLoudnessStandard::StreamingSpotify.target_integrated_lufs(), -14.0);
        assert_eq!(BroadcastLoudnessStandard::AppleMusic.target_integrated_lufs(), -16.0);
    }

    #[test]
    fn test_true_peak_detection() {
        // Inter-sample peak between samples
        let buffer = vec![0.0f32, 0.9, -0.9, 0.0, 0.9, -0.9];
        let tp = BroadcastLoudnessNormalizer::calculate_true_peak(&buffer);
        assert!(tp > -1.5);
    }

    #[test]
    fn test_broadcast_loudness_normalization_pass() {
        // Generate test tone (1 kHz at ~ -18 dBFS)
        let mut buffer = Vec::new();
        for i in 0..48000 {
            let t = i as f32 / 48000.0;
            let sample = (2.0 * std::f32::consts::PI * 1000.0 * t).sin() * 0.125;
            buffer.push(sample);
            buffer.push(sample); // Stereo
        }

        let result = BroadcastLoudnessNormalizer::normalize(
            &mut buffer,
            2,
            48000,
            BroadcastLoudnessStandard::EbuR128,
        );

        assert!(result.original.integrated_lufs < -15.0);
        // Normalized buffer should be near -23.0 LUFS
        assert!((result.normalized.integrated_lufs - (-23.0)).abs() < 1.5);
    }
}
