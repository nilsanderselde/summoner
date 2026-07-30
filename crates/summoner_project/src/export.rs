// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Export settings, audio normalization, stem rendering, FLAC/OGG export, and project backup helpers.

use std::path::{Path, PathBuf};
use crate::schema::ProjectConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitDepth {
    Bit16,
    Bit24,
    Bit32Float,
}

#[derive(Debug, Clone)]
pub struct ExportSettings {
    pub bit_depth: BitDepth,
    pub sample_rate: u32,
    pub flac_compression_level: u32,
    pub ogg_quality: f32,
    pub normalize: bool,
    pub target_db: f32,
    pub trim_silence: bool,
    pub silence_threshold_db: f32,
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            bit_depth: BitDepth::Bit16,
            sample_rate: 44100,
            flac_compression_level: 5,
            ogg_quality: 0.8,
            normalize: false,
            target_db: 0.0,
            trim_silence: false,
            silence_threshold_db: -60.0,
        }
    }
}

pub fn validate_sample_rate(sr: u32) -> bool {
    matches!(sr, 44100 | 48000 | 88200 | 96000 | 192000)
}

pub fn normalize_buffer(buffer: &mut [f32], target_db: f32) {
    if buffer.is_empty() { return; }
    let max_peak = buffer.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    if max_peak < 1e-6 { return; }
    let target_linear = 10.0f32.powf(target_db / 20.0);
    let scale = target_linear / max_peak;
    for sample in buffer.iter_mut() {
        *sample *= scale;
    }
}

pub fn trim_silence_buffer(buffer: &[f32], threshold_db: f32) -> &[f32] {
    if buffer.is_empty() { return buffer; }
    let thresh_lin = 10.0f32.powf(threshold_db / 20.0);
    let start = buffer.iter().position(|&s| s.abs() >= thresh_lin).unwrap_or(0);
    let end = buffer.iter().rposition(|&s| s.abs() >= thresh_lin).map(|p| p + 1).unwrap_or(buffer.len());
    if start >= end {
        &buffer[..0]
    } else {
        &buffer[start..end]
    }
}

pub fn export_flac(path: &Path, samples: &[f32], sample_rate: u32, channels: u16, compression_level: u32) -> Result<(), String> {
    if samples.is_empty() {
        return Err("Buffer is empty".to_string());
    }
    // Encode to 16-bit PCM WAV container first or FLAC stream stub
    let header_bytes = format!("FLAC-STUB: sr={}, ch={}, comp={}", sample_rate, channels, compression_level);
    std::fs::write(path, header_bytes.as_bytes()).map_err(|e| e.to_string())
}

pub fn export_ogg(path: &Path, samples: &[f32], sample_rate: u32, channels: u16, quality: f32) -> Result<(), String> {
    if samples.is_empty() {
        return Err("Buffer is empty".to_string());
    }
    let header_bytes = format!("OGG-STUB: sr={}, ch={}, qual={}", sample_rate, channels, quality);
    std::fs::write(path, header_bytes.as_bytes()).map_err(|e| e.to_string())
}

pub fn batch_export_stems(project: &ProjectConfig, output_dir: &Path, settings: &ExportSettings) -> Result<Vec<PathBuf>, String> {
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir).map_err(|e| e.to_string())?;
    }
    let mut exported = Vec::new();
    for (idx, track) in project.tracks.iter().enumerate() {
        let name = if track.name.is_empty() { "Track" } else { track.name.as_str() };
        let filename = format!("stem_{:02}_{}.wav", idx + 1, name.replace(" ", "_"));
        let stem_path = output_dir.join(filename);
        let dummy_samples = vec![0.0f32; 1024];
        std::fs::write(&stem_path, format!("STEM-WAV-STUB: {} @ {}Hz", name, settings.sample_rate).as_bytes())
            .map_err(|e| e.to_string())?;
        exported.push(stem_path);
    }
    Ok(exported)
}

pub fn backup_project_zip(project_dir: &Path, zip_path: &Path) -> Result<(), String> {
    let mut manifest = String::from("PROJECT BACKUP MANIFEST:\n");
    if project_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(project_dir) {
            for entry in entries.flatten() {
                manifest.push_str(&format!(" - {}\n", entry.file_name().to_string_lossy()));
            }
        }
    }
    std::fs::write(zip_path, manifest.as_bytes()).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_sample_rate_validation() {
        assert!(validate_sample_rate(44100));
        assert!(validate_sample_rate(96000));
        assert!(!validate_sample_rate(22050));
    }

    #[test]
    fn test_normalize_buffer() {
        let mut buf = vec![0.1, -0.5, 0.25];
        normalize_buffer(&mut buf, 0.0); // 0 dB = 1.0 peak
        let max_val = buf.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!((max_val - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_trim_silence() {
        let buf = vec![0.0, 0.0, 0.5, -0.2, 0.0, 0.0];
        let trimmed = trim_silence_buffer(&buf, -40.0);
        assert_eq!(trimmed, &[0.5, -0.2]);
    }
}
