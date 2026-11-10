// Summoner DAW - Project Asset Management, Track Freezing & Routing Tools (Steps 681-686, 690, 695)
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

use crate::schema::{NodeConfig, ProjectConfig, TrackConfig};
use std::fs;
use std::path::{Path, PathBuf};

/// Step 681: Clean Project tool -- removes unused sample assets from project assets directory.
pub fn clean_project(
    project: &mut ProjectConfig,
    project_dir: &Path,
) -> Result<Vec<String>, String> {
    let assets_dir = project_dir.join("assets");
    if !assets_dir.exists() || !assets_dir.is_dir() {
        return Ok(Vec::new());
    }

    // Collect all referenced asset paths/names
    let mut referenced_names = std::collections::HashSet::new();
    for asset in &project.assets {
        let name = Path::new(&asset.path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        if !name.is_empty() {
            referenced_names.insert(name.to_string());
        }
    }
    for track in &project.tracks {
        if let Some(ref scl) = track.tuning_scl_path {
            if let Some(name) = Path::new(scl).file_name().and_then(|n| n.to_str()) {
                referenced_names.insert(name.to_string());
            }
        }
    }

    let mut removed = Vec::new();
    let entries = fs::read_dir(&assets_dir).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if !referenced_names.contains(filename) && fs::remove_file(&path).is_ok() {
                    removed.push(filename.to_string());
                }
            }
        }
    }

    // Clean unreferenced asset entries from project.assets
    project.assets.retain(|asset| {
        let name = Path::new(&asset.path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        !removed.contains(&name.to_string())
    });

    Ok(removed)
}

/// Step 682: Collect and Save -- gathers all external assets into project assets folder and updates paths.
pub fn collect_and_save(project: &mut ProjectConfig, project_dir: &Path) -> Result<usize, String> {
    let assets_dir = project_dir.join("assets");
    if !assets_dir.exists() {
        fs::create_dir_all(&assets_dir).map_err(|e| e.to_string())?;
    }

    let mut copied_count = 0;
    for asset in &mut project.assets {
        let source_path = PathBuf::from(&asset.path);
        if source_path.exists() && source_path.is_file() {
            if let Some(filename) = source_path.file_name() {
                let dest_path = assets_dir.join(filename);
                if source_path != dest_path {
                    fs::copy(&source_path, &dest_path).map_err(|e| e.to_string())?;
                    asset.path = format!("assets/{}", filename.to_string_lossy());
                    copied_count += 1;
                }
            }
        }
    }

    Ok(copied_count)
}

/// Step 683: Freeze track (renders track DSP to frozen_buffer and sets is_frozen).
pub fn freeze_track(track: &mut TrackConfig, rendered_audio: Vec<f32>) {
    track.is_frozen = true;
    track.frozen_buffer = Some(rendered_audio);
}

/// Step 683: Unfreeze track (restores live DSP processing).
pub fn unfreeze_track(track: &mut TrackConfig) {
    track.is_frozen = false;
    track.frozen_buffer = None;
}

/// Step 684: Add Parallel Compression template for a given track.
pub fn apply_parallel_compression_template(
    project: &mut ProjectConfig,
    track_id: u64,
) -> Result<u64, String> {
    let track_name = match project.tracks.iter().find(|t| t.id == track_id) {
        Some(t) => t.name.clone(),
        None => return Err(format!("Track with ID {} not found", track_id)),
    };

    let next_id = project.tracks.iter().map(|t| t.id).max().unwrap_or(0) + 1;
    let bus_name = format!("Bus - {} (Comp)", track_name);

    let mut comp_params = std::collections::HashMap::new();
    comp_params.insert("threshold_db".to_string(), -18.0);
    comp_params.insert("ratio".to_string(), 4.0);

    let comp_bus_track = TrackConfig {
        id: next_id,
        name: bus_name.clone(),
        channels: 2,
        gain: 0.8,
        nodes: vec![NodeConfig {
            kind: "CompressorNode".to_string(),
            params: comp_params,
            plugin_state: None,
        }],
        send_to_master: true,
        ..Default::default()
    };

    project.tracks.push(comp_bus_track);

    // Route target track to parallel bus via send_level or bus_target
    if let Some(target) = project.tracks.iter_mut().find(|t| t.id == track_id) {
        target.send_level = 0.5;
        target.bus_target = Some(bus_name);
    }

    Ok(next_id)
}

/// Step 685: Sidechain routing assignment.
pub fn set_sidechain_routing(track: &mut TrackConfig, source_track_id: Option<u64>) {
    track.sidechain_source_track_id = source_track_id;
}

/// Step 686: Route track to named bus.
pub fn route_track_to_bus(track: &mut TrackConfig, bus_name: Option<String>) {
    track.bus_target = bus_name;
}

/// Step 690: LUFS target input with auto-level gain adjustment.
pub fn auto_level_track_gain(track: &mut TrackConfig, current_lufs: f32, target_lufs: f32) {
    if current_lufs.is_finite() && target_lufs.is_finite() && current_lufs > -100.0 {
        let diff_db = (target_lufs - current_lufs).clamp(-36.0, 36.0);
        let linear_scale = 10.0f32.powf(diff_db / 20.0);
        track.gain = (track.gain * linear_scale).clamp(0.0, 10.0);
    }
}

/// Detect audio tail duration after playback ends (e.g. reverb/delay tails).
/// Scans backward or in blocks from the end to find where the audio envelope drops
/// below `threshold_db` (e.g. -60 dBFS) for at least `min_silence_ms`.
pub fn detect_tail_frames(
    samples_l: &[f32],
    samples_r: &[f32],
    sample_rate: u32,
    threshold_db: f32,
) -> usize {
    let num_samples = samples_l.len().min(if samples_r.is_empty() {
        samples_l.len()
    } else {
        samples_r.len()
    });
    if num_samples == 0 {
        return 0;
    }

    let threshold_amp = 10.0f32.powf(threshold_db / 20.0);
    let block_size = (sample_rate as usize / 100).max(1); // 10ms blocks
    let mut last_audible_frame = 0;

    for i in (0..num_samples).step_by(block_size) {
        let end = (i + block_size).min(num_samples);
        let mut max_val = 0.0f32;
        for j in i..end {
            let amp_l = samples_l[j].abs();
            let amp_r = if !samples_r.is_empty() {
                samples_r[j].abs()
            } else {
                amp_l
            };
            let peak = amp_l.max(amp_r);
            if peak > max_val {
                max_val = peak;
            }
        }
        if max_val >= threshold_amp {
            last_audible_frame = end;
        }
    }

    last_audible_frame
}

/// Flatten a frozen track into a permanent audio track.
/// Clears DSP plugin nodes, removes frozen buffer, and converts track to permanent audio track.
pub fn flatten_track(track: &mut TrackConfig) -> Result<(), String> {
    if !track.is_frozen || track.frozen_buffer.is_none() {
        return Err("Cannot flatten a track that is not frozen".to_string());
    }

    // Retain frozen audio as permanent audio, clear synth/plugin DSP nodes
    track.nodes.clear();
    track.is_frozen = false;
    // Set track name to indicate flattened status if not already
    if !track.name.contains("(Flattened)") {
        track.name = format!("{} (Flattened)", track.name);
    }

    Ok(())
}

/// Bounce a track in place.
/// If `replace_source` is true, replaces the DSP nodes on the source track with the bounced audio
/// and resets frozen state. If false, mutes source track and creates a new bounced track.
pub fn bounce_in_place(
    project: &mut ProjectConfig,
    track_id: u64,
    rendered_samples: Vec<f32>,
    replace_source: bool,
) -> Result<u64, String> {
    if replace_source {
        let track = match project.tracks.iter_mut().find(|t| t.id == track_id) {
            Some(t) => t,
            None => return Err(format!("Track with ID {} not found", track_id)),
        };

        track.nodes.clear();
        track.frozen_buffer = Some(rendered_samples);
        track.is_frozen = true;
        if !track.name.contains("(Bounced)") {
            track.name = format!("{} (Bounced)", track.name);
        }
        Ok(track_id)
    } else {
        bounce_track_to_new_track(project, track_id, rendered_samples)
    }
}

/// Step 695: Bounce track to new audio track (renders to audio, retains original track DSP).
pub fn bounce_track_to_new_track(
    project: &mut ProjectConfig,
    source_track_id: u64,
    rendered_samples: Vec<f32>,
) -> Result<u64, String> {
    let source_track = match project.tracks.iter_mut().find(|t| t.id == source_track_id) {
        Some(t) => t,
        None => return Err(format!("Track with ID {} not found", source_track_id)),
    };
    source_track.muted = true;
    let source_name = source_track.name.clone();

    let new_id = project.tracks.iter().map(|t| t.id).max().unwrap_or(0) + 1;
    let bounced_track = TrackConfig {
        id: new_id,
        name: format!("{} (Bounced)", source_name),
        channels: 2,
        gain: 1.0,
        is_frozen: true,
        frozen_buffer: Some(rendered_samples),
        send_to_master: true,
        ..Default::default()
    };

    project.tracks.push(bounced_track);
    Ok(new_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tail_detection() {
        let sample_rate = 44100;
        // 2 seconds total: 1 second of audio at 0.5, then 0.5s decaying to -70dB, then silence
        let mut audio = vec![0.0f32; sample_rate as usize * 2];
        for i in 0..sample_rate as usize {
            audio[i] = 0.5;
        }
        // decay from 1s to 1.5s
        let decay_samples = sample_rate as usize / 2;
        for i in 0..decay_samples {
            let t = i as f32 / decay_samples as f32;
            let amp = 0.5 * (-5.0 * t).exp(); // drops below -60dB (0.001) around t=1.0
            audio[sample_rate as usize + i] = amp;
        }

        let tail_frame = detect_tail_frames(&audio, &audio, sample_rate, -60.0);
        // Should detect tail ending somewhere between 1.0s and 1.5s (44100 to 66150)
        assert!(tail_frame >= 44100);
        assert!(tail_frame <= 66150);
    }

    #[test]
    fn test_freeze_flatten_and_bounce_in_place() {
        let mut project = ProjectConfig {
            tracks: vec![TrackConfig {
                id: 1,
                name: "Synth Bass".to_string(),
                nodes: vec![NodeConfig {
                    kind: "SynthNode".to_string(),
                    params: std::collections::HashMap::new(),
                    plugin_state: None,
                }],
                ..Default::default()
            }],
            ..Default::default()
        };

        // Freeze
        let audio = vec![0.1f32; 1000];
        freeze_track(&mut project.tracks[0], audio.clone());
        assert!(project.tracks[0].is_frozen);
        assert_eq!(project.tracks[0].frozen_buffer.as_ref().unwrap().len(), 1000);

        // Flatten
        flatten_track(&mut project.tracks[0]).unwrap();
        assert!(!project.tracks[0].is_frozen);
        assert!(project.tracks[0].nodes.is_empty());
        assert!(project.tracks[0].name.contains("(Flattened)"));

        // Bounce in place (replace source)
        let bounced_samples = vec![0.2f32; 2000];
        let bounced_id = bounce_in_place(&mut project, 1, bounced_samples, true).unwrap();
        assert_eq!(bounced_id, 1);
        assert!(project.tracks[0].is_frozen);
        assert_eq!(project.tracks[0].frozen_buffer.as_ref().unwrap().len(), 2000);
        assert!(project.tracks[0].name.contains("(Bounced)"));

        // Bounce to new track
        let new_samples = vec![0.3f32; 3000];
        let new_id = bounce_in_place(&mut project, 1, new_samples, false).unwrap();
        assert_eq!(new_id, 2);
        assert_eq!(project.tracks.len(), 2);
        assert!(project.tracks[0].muted);
        assert_eq!(project.tracks[1].name, "Synth Bass (Flattened) (Bounced) (Bounced)");
    }
}
