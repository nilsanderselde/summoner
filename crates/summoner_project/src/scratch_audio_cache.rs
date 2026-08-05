// Summoner DAW - Durable Scratch Folder Audio Cache
// Step 1225: Implement durable scratch folder audio cache for time-stretching and pitch-shifting operations (.summoner/scratch/)

use std::fs;
use std::path::{Path, PathBuf};
use hound::{WavSpec, WavWriter, WavReader, SampleFormat};

/// Durable audio cache for time-stretching and pitch-shifting operations.
#[derive(Debug, Clone)]
pub struct ScratchAudioCache {
    pub cache_dir: PathBuf,
}

impl ScratchAudioCache {
    /// Initialize scratch audio cache in default directory (`.summoner/scratch/` or specified path).
    pub fn new(scratch_dir: impl AsRef<Path>) -> Self {
        let dir = scratch_dir.as_ref().to_path_buf();
        if !dir.exists() {
            let _ = fs::create_dir_all(&dir);
        }
        Self { cache_dir: dir }
    }

    /// Default scratch cache location inside user home / working directory.
    pub fn default_dir() -> PathBuf {
        PathBuf::from(".summoner").join("scratch")
    }

    /// Compute deterministic cache key hash for audio processing settings.
    pub fn compute_cache_key(
        &self,
        source_path: &Path,
        stretch_ratio: f64,
        pitch_shift_semitones: f32,
    ) -> String {
        let path_str = source_path.to_string_lossy();
        let key_raw = format!("{}:{:.4}:{:.2}", path_str, stretch_ratio, pitch_shift_semitones);
        let hash = blake3::hash(key_raw.as_bytes());
        format!("audio_cache_{}.wav", hash.to_hex())
    }

    /// Retrieve cached transformed audio sample buffer if present on disk.
    pub fn get_cached_audio(&self, cache_filename: &str) -> Option<(Vec<f32>, u32, u16)> {
        let cache_path = self.cache_dir.join(cache_filename);
        if !cache_path.exists() {
            return None;
        }

        let reader = WavReader::open(&cache_path).ok()?;
        let spec = reader.spec();
        let samples: Vec<f32> = reader
            .into_samples::<i16>()
            .filter_map(|s| s.ok())
            .map(|s| s as f32 / 32768.0)
            .collect();

        Some((samples, spec.sample_rate, spec.channels))
    }

    /// Store rendered transformed audio samples into scratch cache folder.
    pub fn store_cached_audio(
        &self,
        cache_filename: &str,
        samples: &[f32],
        sample_rate: u32,
        channels: u16,
    ) -> Result<PathBuf, String> {
        let cache_path = self.cache_dir.join(cache_filename);
        let spec = WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        let mut writer = WavWriter::create(&cache_path, spec).map_err(|e| e.to_string())?;
        for &s in samples {
            let pcm = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
            writer.write_sample(pcm).map_err(|e| e.to_string())?;
        }
        writer.finalize().map_err(|e| e.to_string())?;

        Ok(cache_path)
    }

    /// Clear all files in scratch audio cache folder.
    pub fn clear_cache(&self) -> Result<usize, String> {
        if !self.cache_dir.exists() {
            return Ok(0);
        }
        let entries = fs::read_dir(&self.cache_dir).map_err(|e| e.to_string())?;
        let mut count = 0;
        for entry in entries.flatten() {
            if entry.path().is_file()
                && fs::remove_file(entry.path()).is_ok() {
                    count += 1;
                }
        }
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_scratch_audio_cache_store_and_retrieve() {
        let temp_dir = env::temp_dir().join("summoner_scratch_test");
        let cache = ScratchAudioCache::new(&temp_dir);

        let source = Path::new("input_vocal.wav");
        let key = cache.compute_cache_key(source, 1.25, 2.0);
        assert!(key.starts_with("audio_cache_"));

        let test_samples = vec![0.0f32, 0.5f32, -0.5f32, 0.8f32, -0.8f32];
        let path = cache.store_cached_audio(&key, &test_samples, 44100, 1).expect("store cache");
        assert!(path.exists());

        let (retrieved, sample_rate, channels) = cache.get_cached_audio(&key).expect("retrieve cache");
        assert_eq!(sample_rate, 44100);
        assert_eq!(channels, 1);
        assert_eq!(retrieved.len(), test_samples.len());

        let cleared = cache.clear_cache().expect("clear cache");
        assert!(cleared >= 1);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
