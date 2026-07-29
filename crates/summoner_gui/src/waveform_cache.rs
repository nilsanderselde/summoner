// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde

use std::collections::HashMap;

/// Cache for precomputed RMS envelope representations of audio clips.
#[derive(Default)]
pub struct WaveformCache {
    pub cache: HashMap<String, Vec<f32>>,
}

impl WaveformCache {
    /// Create a new empty waveform cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Compute an RMS envelope array of fixed block length.
    pub fn compute_rms(samples: &[f32], blocks: usize) -> Vec<f32> {
        if samples.is_empty() || blocks == 0 {
            return Vec::new();
        }
        let samples_per_block = (samples.len() / blocks).max(1);
        let mut rms_env = Vec::with_capacity(blocks);
        for chunk in samples.chunks(samples_per_block) {
            let sum_sq: f32 = chunk.iter().map(|&s| s * s).sum();
            let rms = (sum_sq / chunk.len() as f32).sqrt();
            rms_env.push(rms);
        }
        rms_env
    }

    /// Retrieve precomputed RMS envelope from cache or compute it using BLAKE3 hash key.
    pub fn get_or_compute_rms(&mut self, key: &str, samples: &[f32], blocks: usize) -> &[f32] {
        self.cache
            .entry(key.to_string())
            .or_insert_with(|| Self::compute_rms(samples, blocks))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waveform_cache_rms_computation() {
        let samples = vec![0.5, -0.5, 0.5, -0.5, 1.0, -1.0, 1.0, -1.0];
        let mut cache = WaveformCache::new();
        let rms = cache.get_or_compute_rms("test_hash_key", &samples, 2);
        assert_eq!(rms.len(), 2);
        assert!((rms[0] - 0.5).abs() < 1e-4);
        assert!((rms[1] - 1.0).abs() < 1e-4);

        // Verify cache hit
        let cached = cache.get_or_compute_rms("test_hash_key", &samples, 2);
        assert_eq!(cached.len(), 2);
    }
}
