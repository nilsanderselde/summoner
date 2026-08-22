// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.

//! Deterministic 16-bit PCM WAV audio file encoder.

use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

/// Deterministic 16-bit PCM WAV file writer.
pub struct WavWriter {
    file: File,
    sample_rate: u32,
    num_channels: u16,
    num_frames: u32,
}

impl WavWriter {
    pub fn create(path: impl AsRef<Path>, sample_rate: u32, num_channels: u16) -> io::Result<Self> {
        let mut file = File::create(path)?;
        // Write initial placeholder 44-byte WAV header
        let header = [0u8; 44];
        file.write_all(&header)?;

        Ok(Self {
            file,
            sample_rate,
            num_channels,
            num_frames: 0,
        })
    }

    /// Write interleaving 32-bit floating point audio sample frames as 16-bit PCM.
    pub fn write_interleaved_samples(&mut self, samples: &[f32]) -> io::Result<()> {
        let mut pcm_bytes = Vec::with_capacity(samples.len() * 2);
        for &s in samples {
            let pcm_val: i16 = (s.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16;
            pcm_bytes.extend_from_slice(&pcm_val.to_le_bytes());
        }
        self.file.write_all(&pcm_bytes)?;
        self.num_frames += (samples.len() / self.num_channels as usize) as u32;
        Ok(())
    }

    /// Finalize WAV file header upon completion.
    pub fn finalize(mut self) -> io::Result<()> {
        use std::io::Seek;
        let bits_per_sample: u16 = 16;
        let data_size = self.num_frames * self.num_channels as u32 * (bits_per_sample as u32 / 8);
        let file_size = 36 + data_size;

        let mut header = [0u8; 44];
        header[0..4].copy_from_slice(b"RIFF");
        header[4..8].copy_from_slice(&file_size.to_le_bytes());
        header[8..12].copy_from_slice(b"WAVE");
        header[12..16].copy_from_slice(b"fmt ");
        header[16..20].copy_from_slice(&16u32.to_le_bytes());
        header[20..22].copy_from_slice(&1u16.to_le_bytes());
        header[22..24].copy_from_slice(&self.num_channels.to_le_bytes());
        header[24..28].copy_from_slice(&self.sample_rate.to_le_bytes());

        let byte_rate = self.sample_rate * self.num_channels as u32 * (bits_per_sample as u32 / 8);
        let block_align = self.num_channels * (bits_per_sample / 8);
        header[28..32].copy_from_slice(&byte_rate.to_le_bytes());
        header[32..34].copy_from_slice(&block_align.to_le_bytes());
        header[34..36].copy_from_slice(&bits_per_sample.to_le_bytes());
        header[36..40].copy_from_slice(b"data");
        header[40..44].copy_from_slice(&data_size.to_le_bytes());

        self.file.seek(io::SeekFrom::Start(0))?;
        self.file.write_all(&header)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wav_file_export() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("summoner_test_out.wav");

        {
            let mut writer =
                WavWriter::create(&path, 44100, 2).expect("Failed to create WavWriter");
            let samples = vec![0.0f32; 128];
            writer
                .write_interleaved_samples(&samples)
                .expect("Failed to write samples");
            writer.finalize().expect("Failed to finalize WAV header");
        }

        assert!(path.exists());
        let meta = std::fs::metadata(&path).expect("Failed to get file metadata");
        assert_eq!(meta.len(), 44 + 128 * 2);

        let _ = std::fs::remove_file(&path);
    }
}
