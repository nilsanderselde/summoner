// Summoner DAW - Project Asset Management, Track Freezing & Routing Tools (Steps 681-686, 690, 695)
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

use std::fs;
use std::path::{Path, PathBuf};
use crate::schema::{NodeConfig, ProjectConfig, TrackConfig};

/// Step 681: Clean Project tool -- removes unused sample assets from project assets directory.
pub fn clean_project(project: &mut ProjectConfig, project_dir: &Path) -> Result<Vec<String>, String> {
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
                if !referenced_names.contains(filename)
                    && fs::remove_file(&path).is_ok() {
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
pub fn apply_parallel_compression_template(project: &mut ProjectConfig, track_id: u64) -> Result<u64, String> {
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

/// Step 695: Bounce track to new audio track (renders to audio, retains original track DSP).
pub fn bounce_track_to_new_track(
    project: &mut ProjectConfig,
    source_track_id: u64,
    rendered_samples: Vec<f32>,
) -> Result<u64, String> {
    let source_name = match project.tracks.iter().find(|t| t.id == source_track_id) {
        Some(t) => t.name.clone(),
        None => return Err(format!("Track with ID {} not found", source_track_id)),
    };

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
