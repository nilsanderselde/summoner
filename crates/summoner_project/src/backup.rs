// Summoner DAW - Automated Project Backup Snapshot Manager (Step 1266 & Step 1273)
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use crate::schema::ProjectConfig;
use crate::{parse_project_toml, serialize_project_toml};

/// Automated project backup snapshot creator and manager.
#[derive(Debug, Clone)]
pub struct ProjectAutoSaveManager {
    pub auto_save_interval: Duration,
    pub max_backups: usize,
    pub last_save_time: Option<Instant>,
    pub backup_dir: PathBuf,
}

impl ProjectAutoSaveManager {
    pub fn new(project_dir: impl AsRef<Path>, interval_secs: u64, max_backups: usize) -> Self {
        let backup_dir = project_dir.as_ref().join(".summoner").join("backups");
        Self {
            auto_save_interval: Duration::from_secs(interval_secs.max(1)),
            max_backups: max_backups.max(1),
            last_save_time: None,
            backup_dir,
        }
    }

    /// Check if auto-save is due based on elapsed time.
    pub fn should_auto_save(&self) -> bool {
        match self.last_save_time {
            Some(last) => last.elapsed() >= self.auto_save_interval,
            None => true,
        }
    }

    /// Create a project backup snapshot in `.summoner/backups/`.
    pub fn create_backup_snapshot(&mut self, project: &ProjectConfig) -> Result<PathBuf, String> {
        if !self.backup_dir.exists() {
            fs::create_dir_all(&self.backup_dir).map_err(|e| e.to_string())?;
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs();

        let filename = format!("snapshot_{}.toml", timestamp);
        let backup_path = self.backup_dir.join(filename);

        let content = serialize_project_toml(project).map_err(|e| e.to_string())?;
        fs::write(&backup_path, content).map_err(|e| e.to_string())?;

        self.last_save_time = Some(Instant::now());
        self.prune_old_backups()?;

        Ok(backup_path)
    }

    /// Prune old backup files exceeding `max_backups`.
    pub fn prune_old_backups(&self) -> Result<usize, String> {
        let mut backups = self.list_backups()?;
        if backups.len() <= self.max_backups {
            return Ok(0);
        }

        backups.sort_by_key(|p| fs::metadata(p).and_then(|m| m.modified()).ok());
        let to_remove = backups.len() - self.max_backups;
        let mut removed_count = 0;

        for path in backups.iter().take(to_remove) {
            if fs::remove_file(path).is_ok() {
                removed_count += 1;
            }
        }

        Ok(removed_count)
    }

    /// List all backup snapshot paths ordered by modification time.
    pub fn list_backups(&self) -> Result<Vec<PathBuf>, String> {
        if !self.backup_dir.exists() {
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(&self.backup_dir).map_err(|e| e.to_string())?;
        let mut backups = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("toml") {
                backups.push(path);
            }
        }

        Ok(backups)
    }

    /// Restore project configuration from a specified backup path.
    pub fn restore_snapshot(&self, snapshot_path: &Path) -> Result<ProjectConfig, String> {
        let content = fs::read_to_string(snapshot_path).map_err(|e| e.to_string())?;
        parse_project_toml(&content).map_err(|e| e.to_string())
    }

    /// Restore the latest available backup snapshot.
    pub fn restore_latest_snapshot(&self) -> Result<ProjectConfig, String> {
        let mut backups = self.list_backups()?;
        if backups.is_empty() {
            return Err("No backup snapshots found in `.summoner/backups/`".to_string());
        }

        backups.sort_by_key(|p| fs::metadata(p).and_then(|m| m.modified()).ok());
        let latest = backups.last().unwrap();
        self.restore_snapshot(latest)
    }
}
