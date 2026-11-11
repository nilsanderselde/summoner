// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Project Stability & Crash Recovery Engine (Task J10).
//!
//! Provides:
//! - Active session lock detection (`.summoner/session.lock`) to reliably identify unclean shutdowns and crashes.
//! - Automatic scan and recovery from the latest auto-save backup snapshot.
//! - Graceful session clean exit deregistration.
//! - Comprehensive project file schema migration engine across versions.
//! - Large-project stress and soak verification suite.

use crate::backup::ProjectAutoSaveManager;
use crate::parse_project_toml;
use crate::schema::{
    AssetConfig, AutomationEventConfig, AutomationLaneConfig, MarkerConfig, MidiMappingConfig,
    NodeConfig, ProjectConfig, TrackConfig, TransportConfig,
};
#[cfg(test)]
use crate::serialize_project_toml;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Session lock file metadata recorded while DAW is actively editing a project.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionLock {
    pub process_id: u32,
    pub project_name: String,
    pub opened_at_timestamp: u64,
    pub clean_shutdown: bool,
}

/// Crash recovery manager inspecting session locks and managing project restoration.
#[derive(Debug, Clone)]
pub struct CrashRecoveryManager {
    pub project_dir: PathBuf,
    pub lock_path: PathBuf,
    pub auto_save_manager: ProjectAutoSaveManager,
}

impl CrashRecoveryManager {
    pub fn new(project_dir: impl AsRef<Path>) -> Self {
        let p_dir = project_dir.as_ref().to_path_buf();
        let dot_summoner = p_dir.join(".summoner");
        let lock_path = dot_summoner.join("session.lock");
        let auto_save_manager = ProjectAutoSaveManager::new(&p_dir, 60, 10);

        Self {
            project_dir: p_dir,
            lock_path,
            auto_save_manager,
        }
    }

    /// Check if a previous session terminated abnormally (unclean shutdown or crash).
    pub fn check_unclean_shutdown(&self) -> bool {
        if !self.lock_path.exists() {
            return false;
        }

        if let Ok(content) = fs::read_to_string(&self.lock_path) {
            if let Ok(lock) = toml::from_str::<SessionLock>(&content) {
                return !lock.clean_shutdown;
            }
        }

        // If lock exists but is corrupt, consider it unclean
        true
    }

    /// Register the start of a session by writing a new lock file.
    pub fn start_session(&self, project_name: &str) -> Result<(), String> {
        let dot_summoner = self.project_dir.join(".summoner");
        if !dot_summoner.exists() {
            fs::create_dir_all(&dot_summoner).map_err(|e| e.to_string())?;
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs();

        let lock = SessionLock {
            process_id: std::process::id(),
            project_name: project_name.to_string(),
            opened_at_timestamp: timestamp,
            clean_shutdown: false,
        };

        let content = toml::to_string_pretty(&lock).map_err(|e| e.to_string())?;
        fs::write(&self.lock_path, content).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Locate the most recent auto-save snapshot from `.summoner/backups/`.
    pub fn find_latest_backup_snapshot(&self) -> Option<PathBuf> {
        let backups_dir = self.auto_save_manager.backup_dir.clone();
        if !backups_dir.exists() || !backups_dir.is_dir() {
            return None;
        }

        let entries = fs::read_dir(backups_dir).ok()?;
        let mut snapshots: Vec<(u64, PathBuf)> = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "toml") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Some(ts_str) = stem.strip_prefix("snapshot_") {
                        if let Ok(ts) = ts_str.parse::<u64>() {
                            snapshots.push((ts, path));
                        }
                    }
                }
            }
        }

        snapshots.sort_by_key(|(ts, _)| *ts);
        snapshots.pop().map(|(_, p)| p)
    }

    /// Recover project state from the latest available auto-save snapshot.
    pub fn recover_latest_snapshot(&self) -> Result<ProjectConfig, String> {
        let snapshot_path = self
            .find_latest_backup_snapshot()
            .ok_or_else(|| "No auto-save backup snapshot found for recovery".to_string())?;

        let toml_content = fs::read_to_string(&snapshot_path).map_err(|e| e.to_string())?;
        let project = parse_project_toml(&toml_content).map_err(|e| format!("{:?}", e))?;
        Ok(project)
    }

    /// Register clean exit upon normal user quit, safely removing or marking the lock.
    pub fn register_clean_exit(&self) -> Result<(), String> {
        if self.lock_path.exists() {
            let _ = fs::remove_file(&self.lock_path);
        }
        Ok(())
    }
}

/// Project file schema migration engine ensuring backward compatibility across versions.
pub struct ProjectMigrationEngine;

impl ProjectMigrationEngine {
    /// Migrate a raw project TOML string across schema versions (e.g. 1.0 -> 1.1.0 -> 1.2.0).
    pub fn migrate_project_toml(raw_toml: &str) -> Result<ProjectConfig, String> {
        ProjectConfig::migrate_schema(raw_toml)
    }

    /// Upgrade project in-memory to current version standards, applying default fields if missing.
    pub fn upgrade_project_config(project: &mut ProjectConfig) {
        if project.version == "1.0" {
            project.version = "1.2.0".to_string();
        }

        // Ensure all tracks have valid channels and send_to_master
        for track in &mut project.tracks {
            if track.channels == 0 {
                track.channels = 2;
            }
        }
    }
}

/// Large-project soak & stress test generator.
pub struct LargeProjectStressSuite;

impl LargeProjectStressSuite {
    /// Generates a massive project topology with `num_tracks`, automation, and clips for soak testing.
    pub fn generate_massive_project(
        num_tracks: usize,
        clips_per_track: usize,
        points_per_lane: usize,
    ) -> ProjectConfig {
        let mut tracks = Vec::with_capacity(num_tracks);
        let mut automation_lanes = Vec::new();

        for t in 0..num_tracks {
            let track_id = (t + 1) as u64;

            let mut audio_clips = Vec::with_capacity(clips_per_track);
            for c in 0..clips_per_track {
                let mut clip = crate::audio_clip::AudioClipConfig::new(
                    format!("clip_t{}_c{}", track_id, c),
                    format!("Audio Take {}.{}", track_id, c),
                    "asset_0",
                    (c * 4) as f64,
                    4.0,
                );
                clip.file_path = Some(format!("assets/audio/take_{}.wav", t % 8));
                audio_clips.push(clip);
            }

            tracks.push(TrackConfig {
                id: track_id,
                name: format!("Track {:03}", track_id),
                channels: 2,
                gain: 0.8,
                pan: 0.0,
                nodes: vec![NodeConfig {
                    kind: "CompressorNode".to_string(),
                    params: std::collections::HashMap::new(),
                    plugin_state: None,
                }],
                audio_clips,
                send_to_master: true,
                ..Default::default()
            });

            // Add automation lane
            let mut events = Vec::with_capacity(points_per_lane);
            for p in 0..points_per_lane {
                events.push(AutomationEventConfig {
                    frame: (p as u64) * 22050,
                    value: (p as f32 * 0.1).sin().abs(),
                });
            }

            automation_lanes.push(AutomationLaneConfig {
                track_id,
                param_id: "gain".to_string(),
                events,
            });
        }

        ProjectConfig {
            version: "1.2.0".to_string(),
            name: "Soak Test Symphony".to_string(),
            tuning_file: None,
            transport: TransportConfig {
                sample_rate: 48000,
                bpm: 128.0,
                time_signature: "4/4".to_string(),
                ..Default::default()
            },
            tracks,
            assets: vec![AssetConfig {
                id: "asset_0".to_string(),
                path: "assets/audio/take_0.wav".to_string(),
                hash: "0123456789abcdef".to_string(),
                auto_slice: false,
                slice_threshold: 0.1,
            }],
            automation_lanes,
            midi_mappings: vec![MidiMappingConfig {
                channel: 1,
                cc: 7,
                param_id: "gain".to_string(),
                track_id: Some(1),
                min_val: 0.0,
                max_val: 1.0,
                mode: crate::schema::ControllerMappingMode::Absolute,
                soft_takeover: false,
                device_name: None,
            }],
            markers: vec![MarkerConfig {
                name: "Verse 1".to_string(),
                beat: 0.0,
                ..Default::default()
            }],
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crash_recovery_session_lock_and_snapshot_restoration() {
        let temp_dir = std::env::temp_dir().join("summoner_crash_recovery_test");
        let _ = fs::remove_dir_all(&temp_dir);
        let recovery_mgr = CrashRecoveryManager::new(&temp_dir);

        // Initially no lock exists
        assert!(!recovery_mgr.check_unclean_shutdown());

        // Start session -> lock created with clean_shutdown: false
        recovery_mgr.start_session("My Masterpiece").unwrap();
        assert!(recovery_mgr.check_unclean_shutdown());

        // Create an auto-save snapshot
        let original_project = ProjectConfig {
            name: "My Masterpiece".to_string(),
            tracks: vec![TrackConfig {
                id: 1,
                name: "Lead Vocal".to_string(),
                gain: 0.75,
                ..Default::default()
            }],
            ..Default::default()
        };

        let mut auto_save = recovery_mgr.auto_save_manager.clone();
        let backup_path = auto_save.create_backup_snapshot(&original_project).unwrap();
        assert!(backup_path.exists());

        // Verify crash recovery can find latest backup and restore state
        let recovered = recovery_mgr.recover_latest_snapshot().unwrap();
        assert_eq!(recovered.name, "My Masterpiece");
        assert_eq!(recovered.tracks.len(), 1);
        assert_eq!(recovered.tracks[0].name, "Lead Vocal");
        assert!((recovered.tracks[0].gain - 0.75).abs() < 1e-4);

        // Register clean exit -> removes lock
        recovery_mgr.register_clean_exit().unwrap();
        assert!(!recovery_mgr.check_unclean_shutdown());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_large_project_soak_and_stress_serialization_roundtrip() {
        // Generate massive project with 128 tracks, 8 clips per track (1024 clips total),
        // and 20 automation events per track (2560 events total)
        let project = LargeProjectStressSuite::generate_massive_project(128, 8, 20);

        assert_eq!(project.tracks.len(), 128);
        assert_eq!(project.automation_lanes.len(), 128);

        let total_clips: usize = project.tracks.iter().map(|t| t.audio_clips.len()).sum();
        assert_eq!(total_clips, 1024);

        let total_events: usize = project
            .automation_lanes
            .iter()
            .map(|l| l.events.len())
            .sum();
        assert_eq!(total_events, 2560);

        // Measure serialization performance
        let start_ser = std::time::Instant::now();
        let serialized = serialize_project_toml(&project).unwrap();
        let ser_elapsed = start_ser.elapsed();
        assert!(
            ser_elapsed.as_millis() < 500,
            "128-track project serialization took too long: {:?}",
            ser_elapsed
        );

        // Measure deserialization roundtrip
        let start_de = std::time::Instant::now();
        let deserialized = parse_project_toml(&serialized).unwrap();
        let de_elapsed = start_de.elapsed();
        assert!(
            de_elapsed.as_millis() < 500,
            "128-track project deserialization took too long: {:?}",
            de_elapsed
        );

        // Verify state preservation
        assert_eq!(deserialized.tracks.len(), 128);
        assert_eq!(deserialized.automation_lanes.len(), 128);
        assert_eq!(deserialized.tracks[127].name, "Track 128");
    }
}
