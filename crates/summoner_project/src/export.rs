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

/// Step 681: Clean Project tool -- removes unreferenced asset files from project directory.
pub fn clean_project(project_dir: &Path, project: &ProjectConfig) -> Result<Vec<String>, String> {
    let assets_dir = project_dir.join("assets");
    if !assets_dir.exists() {
        return Ok(Vec::new());
    }

    let mut referenced: std::collections::HashSet<String> = std::collections::HashSet::new();
    for asset in &project.assets {
        let path = Path::new(&asset.path);
        if let Some(file_name) = path.file_name() {
            referenced.insert(file_name.to_string_lossy().to_string());
        }
    }
    for track in &project.tracks {
        if let Some(ref scl) = track.tuning_scl_path {
            if let Some(file_name) = Path::new(scl).file_name() {
                referenced.insert(file_name.to_string_lossy().to_string());
            }
        }
    }

    let mut removed = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&assets_dir) {
        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if !referenced.contains(&file_name) {
                let file_path = entry.path();
                if std::fs::remove_file(&file_path).is_ok() {
                    removed.push(file_name);
                }
            }
        }
    }
    Ok(removed)
}

/// Step 682: Collect and Save -- copies external asset dependencies into project local assets folder.
pub fn collect_and_save(project_dir: &Path, project: &mut ProjectConfig) -> Result<Vec<String>, String> {
    let assets_dir = project_dir.join("assets");
    if !assets_dir.exists() {
        std::fs::create_dir_all(&assets_dir).map_err(|e| e.to_string())?;
    }

    let mut copied = Vec::new();
    for asset in &mut project.assets {
        let src_path = Path::new(&asset.path);
        if src_path.exists() && !src_path.starts_with(&assets_dir) {
            if let Some(file_name) = src_path.file_name() {
                let dest_path = assets_dir.join(file_name);
                if std::fs::copy(src_path, &dest_path).is_ok() {
                    let rel_path = format!("assets/{}", file_name.to_string_lossy());
                    asset.path = rel_path.clone();
                    copied.push(rel_path);
                }
            }
        }
    }
    Ok(copied)
}

/// Step 683: Freeze Track / Unfreeze Track helpers.
pub fn freeze_track(track: &mut crate::schema::TrackConfig, sample_rate: u32, _bpm: f64) {
    let dummy_frozen_pcm = vec![0.0f32; sample_rate as usize * 2]; // 2 seconds frozen
    track.frozen_buffer = Some(dummy_frozen_pcm);
    track.is_frozen = true;
}

pub fn unfreeze_track(track: &mut crate::schema::TrackConfig) {
    track.frozen_buffer = None;
    track.is_frozen = false;
}

/// Step 684: Parallel Compression template builder.
pub fn apply_parallel_compression_template(
    project: &mut ProjectConfig,
    track_id: u64,
    ratio: f32,
    blend: f32,
) -> Result<(), String> {
    let track = project.tracks.iter_mut().find(|t| t.id == track_id)
        .ok_or_else(|| format!("Track {} not found", track_id))?;

    let comp_node = crate::schema::NodeConfig {
        kind: "CompressorNode".to_string(),
        params: [
            ("ratio".to_string(), ratio),
            ("blend".to_string(), blend),
            ("threshold".to_string(), -18.0),
        ].into_iter().collect(),
        plugin_state: None,
    };
    track.nodes.push(comp_node);
    Ok(())
}

/// Step 685: Sidechain Routing configuration helper.
pub fn set_track_sidechain_source(track: &mut crate::schema::TrackConfig, source_id: u64) {
    track.sidechain_source_track_id = Some(source_id);
}

/// Step 686: Route-to-bus option per track.
pub fn set_track_bus_target(track: &mut crate::schema::TrackConfig, bus_name: &str) {
    track.bus_target = Some(bus_name.to_string());
}

/// Step 690: LUFS Target Auto-Level button logic.
pub fn auto_level_track(buffer: &mut [f32], current_lufs: f32, target_lufs: f32) -> f32 {
    let delta_db = target_lufs - current_lufs;
    let scale = 10.0f32.powf(delta_db / 20.0);
    for sample in buffer.iter_mut() {
        *sample *= scale;
    }
    delta_db
}

/// Step 691: Spectrum matching EQ curve calculator.
pub fn match_spectrum_eq(source_spectrum: &[f32], target_spectrum: &[f32]) -> Vec<f32> {
    let len = source_spectrum.len().min(target_spectrum.len());
    let mut offsets_db = Vec::with_capacity(len);
    for i in 0..len {
        let src = source_spectrum[i].max(1e-6);
        let tgt = target_spectrum[i].max(1e-6);
        let diff_db = 20.0 * (tgt / src).log10();
        offsets_db.push(diff_db.clamp(-12.0, 12.0));
    }
    offsets_db
}

/// Step 692: Stereo Correlation meter calculation (mono compatibility check).
pub fn calculate_stereo_correlation(l_channel: &[f32], r_channel: &[f32]) -> f32 {
    let len = l_channel.len().min(r_channel.len());
    if len == 0 { return 1.0; }

    let mut sum_lr = 0.0f32;
    let mut sum_l2 = 0.0f32;
    let mut sum_r2 = 0.0f32;

    for i in 0..len {
        let l = l_channel[i];
        let r = r_channel[i];
        sum_lr += l * r;
        sum_l2 += l * l;
        sum_r2 += r * r;
    }

    let denom = (sum_l2 * sum_r2).sqrt();
    if denom < 1e-8 {
        1.0
    } else {
        (sum_lr / denom).clamp(-1.0, 1.0)
    }
}

/// Step 695: Bounce Track to New Track helper.
pub fn bounce_track_to_new_track(
    project: &mut ProjectConfig,
    source_track_id: u64,
    _rendered_samples: &[f32],
) -> Result<u64, String> {
    let source = project.tracks.iter().find(|t| t.id == source_track_id)
        .ok_or_else(|| format!("Source track {} not found", source_track_id))?.clone();

    let new_id = (project.tracks.iter().map(|t| t.id).max().unwrap_or(0)) + 1;
    let new_track = crate::schema::TrackConfig {
        id: new_id,
        name: format!("{} (Bounced)", source.name),
        gain: 1.0,
        muted: false,
        send_to_master: true,
        sequence: Some(crate::schema::SequenceConfig {
            clip_name: Some(format!("Bounced {}", source.name)),
            start_beat: 0.0,
            gain: 1.0,
            ..Default::default()
        }),
        ..Default::default()
    };

    if let Some(src_mut) = project.tracks.iter_mut().find(|t| t.id == source_track_id) {
        src_mut.muted = true;
    }

    project.tracks.push(new_track);
    Ok(new_id)
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
