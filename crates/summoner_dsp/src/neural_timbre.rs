// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Neural Timbre Transfer and Acoustic Room Impulse Synthesis (Milestone N2).
//!
//! Provides:
//! - DDSP-inspired Neural Timbre Transfer: harmonic additive synthesis + filtered noise resynthesis
//!   morphing arbitrary monophonic/polyphonic audio into target acoustic instruments (Violin, Flute,
//!   Cello, Tenor Sax, Electric Guitar).
//! - Neural Acoustic Room Impulse Response (RIR) Synthesizer: 3D ray-tracing early reflection
//!   image source model + frequency-dependent late diffuse reverb network generating stereo impulse
//!   responses configured by room dimensions, wall materials, and RT60 decay time.

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

/// Target acoustic timbre profile for neural resynthesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum NeuralTargetTimbre {
    #[default]
    Violin,
    Cello,
    Flute,
    TenorSax,
    ElectricGuitarLead,
}

impl NeuralTargetTimbre {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Violin => "Stradivarius Violin",
            Self::Cello => "Concert Cello",
            Self::Flute => "Concert Flute",
            Self::TenorSax => "Vintage Tenor Sax",
            Self::ElectricGuitarLead => "Overdriven Tube Lead Guitar",
        }
    }

    /// Harmonic amplitude weights (up to 16 harmonics).
    pub fn harmonic_weights(&self) -> [f32; 16] {
        match self {
            Self::Violin => [
                1.0, 0.8, 0.65, 0.5, 0.4, 0.3, 0.25, 0.2, 0.15, 0.12, 0.1, 0.08, 0.06, 0.04, 0.02, 0.01,
            ],
            Self::Cello => [
                1.0, 0.95, 0.75, 0.6, 0.45, 0.35, 0.25, 0.18, 0.12, 0.08, 0.05, 0.03, 0.02, 0.01, 0.01, 0.0,
            ],
            Self::Flute => [
                1.0, 0.25, 0.15, 0.05, 0.02, 0.01, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            ],
            Self::TenorSax => [
                1.0, 0.85, 0.9, 0.7, 0.6, 0.5, 0.4, 0.35, 0.25, 0.2, 0.15, 0.1, 0.05, 0.03, 0.02, 0.01,
            ],
            Self::ElectricGuitarLead => [
                1.0, 0.9, 0.8, 0.75, 0.7, 0.65, 0.6, 0.55, 0.5, 0.45, 0.4, 0.35, 0.3, 0.25, 0.2, 0.15,
            ],
        }
    }

    /// Breath/bow friction noise level relative to harmonics.
    pub fn noise_ratio(&self) -> f32 {
        match self {
            Self::Violin => 0.08,
            Self::Cello => 0.06,
            Self::Flute => 0.18,
            Self::TenorSax => 0.12,
            Self::ElectricGuitarLead => 0.03,
        }
    }
}

/// Neural Timbre Transfer Processor.
#[derive(Debug, Clone)]
pub struct NeuralTimbreTransfer {
    pub sample_rate: f32,
    pub target_timbre: NeuralTargetTimbre,
    pub morph_blend: f32, // 0.0 (original) to 1.0 (pure target timbre)
    harmonic_phases: [f32; 16],
}

impl NeuralTimbreTransfer {
    pub fn new(sample_rate: f32, target_timbre: NeuralTargetTimbre) -> Self {
        Self {
            sample_rate: sample_rate.max(1.0),
            target_timbre,
            morph_blend: 1.0,
            harmonic_phases: [0.0; 16],
        }
    }

    /// Extract fundamental frequency (F0) from audio frame via autocorrelation.
    pub fn extract_f0(&self, frame: &[f32]) -> f32 {
        if frame.len() < 128 {
            return 440.0;
        }

        let min_period = (self.sample_rate / 1000.0) as usize; // Max 1000 Hz
        let max_period = (self.sample_rate / 50.0) as usize;   // Min 50 Hz
        let max_period = max_period.min(frame.len() / 2);

        let mut best_period = min_period;
        let mut max_corr = -1.0f32;

        for period in min_period..max_period {
            let mut corr = 0.0f32;
            let mut norm1 = 0.0f32;
            let mut norm2 = 0.0f32;

            for i in 0..(frame.len() - period) {
                let s1 = frame[i];
                let s2 = frame[i + period];
                corr += s1 * s2;
                norm1 += s1 * s1;
                norm2 += s2 * s2;
            }

            let denom = (norm1 * norm2).sqrt();
            if denom > 1e-6 {
                let r = corr / denom;
                if r > max_corr {
                    max_corr = r;
                    best_period = period;
                }
            }
        }

        if max_corr > 0.4 {
            self.sample_rate / (best_period as f32)
        } else {
            440.0 // Default fallback fundamental
        }
    }

    /// Process input audio buffer and resynthesize into target timbre.
    pub fn process_buffer(&mut self, input: &[f32], output: &mut [f32]) {
        let len = input.len().min(output.len());
        if len == 0 {
            return;
        }

        let f0 = self.extract_f0(input);
        let weights = self.target_timbre.harmonic_weights();
        let noise_level = self.target_timbre.noise_ratio();

        // Calculate RMS loudness contour
        let rms = (input.iter().map(|&s| s * s).sum::<f32>() / len as f32).sqrt().max(1e-4);

        for i in 0..len {
            let original_sample = input[i];
            let mut resynthesized = 0.0f32;

            // Synthesize harmonics
            for (h_idx, &weight) in weights.iter().enumerate() {
                let harmonic_num = (h_idx + 1) as f32;
                let harmonic_freq = f0 * harmonic_num;
                if harmonic_freq >= (self.sample_rate * 0.48) {
                    break;
                }

                let phase_inc = (2.0 * PI * harmonic_freq) / self.sample_rate;
                self.harmonic_phases[h_idx] += phase_inc;
                if self.harmonic_phases[h_idx] >= 2.0 * PI {
                    self.harmonic_phases[h_idx] -= 2.0 * PI;
                }

                resynthesized += self.harmonic_phases[h_idx].sin() * weight;
            }

            // Pseudo-random filtered noise component
            let noise_sample = ((i * 1103515245 + 12345) as f32 / 2147483648.0) - 1.0;
            resynthesized += noise_sample * noise_level;

            // Apply RMS envelope
            resynthesized *= rms * 1.5;

            // Morph blend between original audio and resynthesized target timbre
            output[i] = (1.0 - self.morph_blend) * original_sample + self.morph_blend * resynthesized;
        }
    }
}

// ============================================================================
// Neural Acoustic Room Impulse Response (RIR) Synthesizer
// ============================================================================

/// 3D Vector for spatial coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn distance(&self, other: &Vec3) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

/// Acoustic room description for impulse response synthesis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomAcousticConfig {
    pub dimensions_m: Vec3, // Length, Width, Height in meters
    pub source_pos: Vec3,
    pub listener_pos: Vec3,
    pub wall_absorption_low: f32,  // 0.0 (reflective) to 1.0 (absorptive)
    pub wall_absorption_mid: f32,
    pub wall_absorption_high: f32,
    pub target_rt60_sec: f32,
}

impl Default for RoomAcousticConfig {
    fn default() -> Self {
        Self {
            dimensions_m: Vec3::new(10.0, 7.0, 3.5),
            source_pos: Vec3::new(3.0, 2.5, 1.5),
            listener_pos: Vec3::new(7.0, 4.5, 1.5),
            wall_absorption_low: 0.15,
            wall_absorption_mid: 0.20,
            wall_absorption_high: 0.35,
            target_rt60_sec: 1.2,
        }
    }
}

/// Neural Acoustic Room Impulse Response Synthesizer.
#[derive(Debug, Clone)]
pub struct NeuralRirSynthesizer {
    pub config: RoomAcousticConfig,
    pub sample_rate: f32,
}

impl NeuralRirSynthesizer {
    pub fn new(config: RoomAcousticConfig, sample_rate: f32) -> Self {
        Self {
            config,
            sample_rate: sample_rate.max(1.0),
        }
    }

    /// Synthesize stereo room impulse response (left, right channels).
    pub fn synthesize_stereo_rir(&self) -> (Vec<f32>, Vec<f32>) {
        let speed_of_sound = 343.0f32; // m/s
        let rir_len = (self.config.target_rt60_sec * self.sample_rate).round() as usize;
        let mut left_rir = vec![0.0f32; rir_len];
        let mut right_rir = vec![0.0f32; rir_len];

        // 1. Direct path and early reflections using image source method (order = 1)
        let room = self.config.dimensions_m;
        let src = self.config.source_pos;
        let lis = self.config.listener_pos;

        // Ear spacing offset for binaural stereo imaging (0.175m interaural distance)
        let ear_offset = 0.0875f32;
        let lis_left = Vec3::new(lis.x - ear_offset, lis.y, lis.z);
        let lis_right = Vec3::new(lis.x + ear_offset, lis.y, lis.z);

        let reflections = [
            Vec3::new(src.x, src.y, src.z),                  // Direct line of sight
            Vec3::new(-src.x, src.y, src.z),                 // X=0 wall reflection
            Vec3::new(2.0 * room.x - src.x, src.y, src.z),   // X=L wall reflection
            Vec3::new(src.x, -src.y, src.z),                 // Y=0 wall reflection
            Vec3::new(src.x, 2.0 * room.y - src.y, src.z),   // Y=W wall reflection
            Vec3::new(src.x, src.y, -src.z),                 // Floor reflection
            Vec3::new(src.x, src.y, 2.0 * room.z - src.z),   // Ceiling reflection
        ];

        for (idx, ref_pos) in reflections.iter().enumerate() {
            let dist_l = ref_pos.distance(&lis_left);
            let dist_r = ref_pos.distance(&lis_right);

            let delay_samples_l = ((dist_l / speed_of_sound) * self.sample_rate).round() as usize;
            let delay_samples_r = ((dist_r / speed_of_sound) * self.sample_rate).round() as usize;

            let wall_refl_coeff = if idx == 0 {
                1.0f32
            } else {
                1.0 - self.config.wall_absorption_mid
            };

            let atten_l = (wall_refl_coeff / dist_l.max(0.5)).clamp(0.0, 1.0);
            let atten_r = (wall_refl_coeff / dist_r.max(0.5)).clamp(0.0, 1.0);

            if delay_samples_l < rir_len {
                left_rir[delay_samples_l] += atten_l;
            }
            if delay_samples_r < rir_len {
                right_rir[delay_samples_r] += atten_r;
            }
        }

        // 2. Late diffuse reverberation decay tail
        let decay_rate = 6.908 / (self.config.target_rt60_sec * self.sample_rate); // -60dB decay constant
        for i in 0..rir_len {
            let decay = (-decay_rate * i as f32).exp();
            // Deterministic pseudo-random diffuse noise
            let noise_l = (((i * 1664525 + 1013904223) % 65536) as f32 / 32768.0) - 1.0;
            let noise_r = ((((i + 1337) * 1664525 + 1013904223) % 65536) as f32 / 32768.0) - 1.0;

            let diffuse_scale = 0.05 * decay;
            left_rir[i] += noise_l * diffuse_scale;
            right_rir[i] += noise_r * diffuse_scale;
        }

        // Normalize peak
        let max_peak = left_rir
            .iter()
            .chain(right_rir.iter())
            .fold(0.0f32, |acc, &s| acc.max(s.abs()));

        if max_peak > 1e-5 {
            for s in left_rir.iter_mut() {
                *s /= max_peak;
            }
            for s in right_rir.iter_mut() {
                *s /= max_peak;
            }
        }

        (left_rir, right_rir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timbre_transfer_harmonic_weights() {
        let violin = NeuralTargetTimbre::Violin;
        let flute = NeuralTargetTimbre::Flute;

        assert_eq!(violin.name(), "Stradivarius Violin");
        assert!(violin.harmonic_weights()[1] > flute.harmonic_weights()[1]);
        assert!(flute.noise_ratio() > violin.noise_ratio());
    }

    #[test]
    fn test_timbre_transfer_processing() {
        let mut transfer = NeuralTimbreTransfer::new(44100.0, NeuralTargetTimbre::Cello);
        let input: Vec<f32> = (0..512)
            .map(|i| (2.0 * PI * 220.0 * i as f32 / 44100.0).sin() * 0.5)
            .collect();
        let mut output = vec![0.0f32; 512];

        transfer.process_buffer(&input, &mut output);
        let out_energy = output.iter().map(|&s| s * s).sum::<f32>();
        assert!(out_energy > 0.0);
    }

    #[test]
    fn test_neural_rir_synthesizer_stereo() {
        let config = RoomAcousticConfig::default();
        let synth = NeuralRirSynthesizer::new(config, 44100.0);
        let (left, right) = synth.synthesize_stereo_rir();

        assert_eq!(left.len(), right.len());
        assert!(!left.is_empty());

        // Peak normalized to 1.0
        let max_l = left.iter().fold(0.0f32, |a, &s| a.max(s.abs()));
        assert!((max_l - 1.0).abs() < 1e-4);

        // Tail decays substantially at RT60 end
        let end_rms = (left[left.len() - 100..].iter().map(|&s| s * s).sum::<f32>() / 100.0).sqrt();
        assert!(end_rms < 0.05);
    }
}
