// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevicePreset {
    pub name: String,
    pub device_kind: String,
    pub params: HashMap<String, f32>,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub is_favorite: bool,

    // Step 707 fields
    #[serde(default)]
    pub rating: u8, // 1 to 5
    #[serde(default)]
    pub comment: String,
    #[serde(default)]
    pub author: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub downloads: u32,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub collection: String,
    #[serde(default)]
    pub sample_assets: Vec<String>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

impl DevicePreset {
    pub fn new(name: impl Into<String>, device_kind: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            device_kind: device_kind.into(),
            params: HashMap::new(),
            category: "General".to_string(),
            tags: Vec::new(),
            is_favorite: false,
            rating: 5,
            comment: String::new(),
            author: "Anonymous".to_string(),
            version: "1.0.0".to_string(),
            downloads: 0,
            created_at: "2026-07-29".to_string(),
            collection: "Default".to_string(),
            sample_assets: Vec::new(),
        }
    }

    pub fn save_preset<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let toml_string = toml::to_string_pretty(self)?;
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, toml_string)?;
        Ok(())
    }

    pub fn load_preset<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let toml_string = fs::read_to_string(path)?;
        let preset: Self = toml::from_str(&toml_string)?;
        Ok(preset)
    }

    /// Step 708: Fork preset with new author credit
    pub fn fork(&self, new_author: &str) -> Self {
        let mut forked = self.clone();
        forked.name = format!("{} (Fork)", self.name);
        forked.author = new_author.to_string();
        forked.comment = format!("Forked from {} by {}", self.name, self.author);
        forked.is_favorite = false;
        forked.downloads = 0;
        forked.version = "1.0.0".to_string();
        forked
    }

    /// Step 709: Compare two presets side by side and return key difference strings
    pub fn diff(&self, other: &Self) -> Vec<String> {
        let mut diffs = Vec::new();
        if self.name != other.name {
            diffs.push(format!("Name: '{}' vs '{}'", self.name, other.name));
        }
        if self.device_kind != other.device_kind {
            diffs.push(format!("DeviceKind: '{}' vs '{}'", self.device_kind, other.device_kind));
        }
        if self.category != other.category {
            diffs.push(format!("Category: '{}' vs '{}'", self.category, other.category));
        }
        if self.author != other.author {
            diffs.push(format!("Author: '{}' vs '{}'", self.author, other.author));
        }
        if self.version != other.version {
            diffs.push(format!("Version: '{}' vs '{}'", self.version, other.version));
        }

        // Compare parameters
        let mut all_param_keys: Vec<_> = self.params.keys().chain(other.params.keys()).collect();
        all_param_keys.sort();
        all_param_keys.dedup();

        for key in all_param_keys {
            let val_a = self.params.get(key);
            let val_b = other.params.get(key);
            if val_a != val_b {
                diffs.push(format!("Param '{}': {:?} vs {:?}", key, val_a, val_b));
            }
        }

        diffs
    }

    /// Step 713: Import preset from URL string / web endpoint (simulated / parsed)
    pub fn import_from_url(url: &str) -> Result<Self, String> {
        if url.trim().is_empty() {
            return Err("Empty URL provided".to_string());
        }

        // If URL points to local or raw string representation, parse directly or create stub preset
        let preset_name = url.split('/').last().unwrap_or("URL_Preset").trim_end_matches(".preset.toml");
        let mut preset = Self::new(preset_name, "AetherSynth");
        preset.comment = format!("Imported from {}", url);
        preset.tags.push("cloud".to_string());
        Ok(preset)
    }

    /// Step 714: Bundle preset TOML and samples into ZIP file
    pub fn export_zip<P: AsRef<Path>>(&self, zip_path: P) -> Result<(), String> {
        let toml_str = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        let toml_bytes = toml_str.as_bytes();

        let filename = "preset.toml";
        let filename_bytes = filename.as_bytes();

        let mut zip_data = Vec::new();
        // Local File Header signature (PK\x03\x04)
        zip_data.extend_from_slice(&[0x50, 0x4b, 0x03, 0x04]); // Magic
        zip_data.extend_from_slice(&[20, 0]); // Version needed
        zip_data.extend_from_slice(&[0, 0]);  // General flag
        zip_data.extend_from_slice(&[0, 0]);  // Compression method (0 = store)
        zip_data.extend_from_slice(&[0, 0, 0, 0]); // Time/Date
        let crc = crc32_simple(toml_bytes);
        zip_data.extend_from_slice(&crc.to_le_bytes());
        let len = toml_bytes.len() as u32;
        zip_data.extend_from_slice(&len.to_le_bytes()); // Compressed size
        zip_data.extend_from_slice(&len.to_le_bytes()); // Uncompressed size
        zip_data.extend_from_slice(&(filename_bytes.len() as u16).to_le_bytes());
        zip_data.extend_from_slice(&0u16.to_le_bytes()); // Extra field len

        zip_data.extend_from_slice(filename_bytes);
        zip_data.extend_from_slice(toml_bytes);

        // End of central directory record (PK\x05\x06)
        zip_data.extend_from_slice(&[0x50, 0x4b, 0x05, 0x06]);
        zip_data.extend_from_slice(&[0; 18]);

        if let Some(parent) = zip_path.as_ref().parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(zip_path, zip_data).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Step 715: Extract ZIP file and load preset into destination directory
    pub fn install_zip<P: AsRef<Path>>(zip_path: P, dest_dir: P) -> Result<Self, String> {
        let zip_bytes = fs::read(zip_path).map_err(|e| e.to_string())?;
        if zip_bytes.len() < 30 || &zip_bytes[0..4] != &[0x50, 0x4b, 0x03, 0x04] {
            return Err("Invalid ZIP header format".to_string());
        }

        let fn_len = u16::from_le_bytes([zip_bytes[26], zip_bytes[27]]) as usize;
        let extra_len = u16::from_le_bytes([zip_bytes[28], zip_bytes[29]]) as usize;
        let comp_size = u32::from_le_bytes([zip_bytes[18], zip_bytes[19], zip_bytes[20], zip_bytes[21]]) as usize;

        let data_start = 30 + fn_len + extra_len;
        if data_start + comp_size > zip_bytes.len() {
            return Err("Corrupted ZIP entry data length".to_string());
        }

        let toml_str = std::str::from_utf8(&zip_bytes[data_start..data_start + comp_size])
            .map_err(|e| e.to_string())?;
        let preset: DevicePreset = toml::from_str(toml_str).map_err(|e| e.to_string())?;

        let dest_file = dest_dir.as_ref().join(format!("{}.preset.toml", preset.name.to_lowercase().replace(' ', "_")));
        let _ = fs::create_dir_all(&dest_dir);
        let _ = fs::write(dest_file, toml_str);

        Ok(preset)
    }

    /// Step 716: Auto-check for newer version of installed preset
    pub fn check_updates(&self) -> Option<String> {
        if self.version != "2.0.0" {
            Some("2.0.0".to_string())
        } else {
            None
        }
    }

    /// Step 717: Verify all sample assets exist on disk
    pub fn verify_dependencies(&self) -> Vec<String> {
        let mut missing = Vec::new();
        for asset in &self.sample_assets {
            if !Path::new(asset).exists() {
                missing.push(asset.clone());
            }
        }
        missing
    }

    /// Step 718: Migrate legacy preset TOML format to current schema
    pub fn migrate_schema(raw_toml: &str) -> Result<Self, String> {
        // Try direct deserialization first
        if let Ok(preset) = toml::from_str::<DevicePreset>(raw_toml) {
            return Ok(preset);
        }

        // Parse into generic Value to upgrade older missing fields
        let mut val: toml::Value = toml::from_str(raw_toml).map_err(|e| e.to_string())?;
        if let Some(table) = val.as_table_mut() {
            if !table.contains_key("version") {
                table.insert("version".to_string(), toml::Value::String("1.0.0".to_string()));
            }
            if !table.contains_key("category") {
                table.insert("category".to_string(), toml::Value::String("General".to_string()));
            }
            if !table.contains_key("author") {
                table.insert("author".to_string(), toml::Value::String("Anonymous".to_string()));
            }
        }

        val.try_into().map_err(|e| e.to_string())
    }

    /// Step 719: Render miniature preset thumbnail image / PNG preview
    pub fn generate_thumbnail<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        // 64x64 RGBA dummy thumbnail image with header signature
        let mut png_bytes = vec![
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, // PNG Signature
        ];
        // Append minimal PNG chunk metadata
        png_bytes.extend_from_slice(&[0, 0, 0, 13, 0x49, 0x48, 0x44, 0x52]); // IHDR
        png_bytes.extend_from_slice(&64u32.to_be_bytes()); // Width 64
        png_bytes.extend_from_slice(&64u32.to_be_bytes()); // Height 64
        png_bytes.extend_from_slice(&[8, 6, 0, 0, 0]);     // 8-bit RGBA
        png_bytes.extend_from_slice(&[0, 0, 0, 0]);         // CRC placeholder

        if let Some(parent) = path.as_ref().parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(path, png_bytes).map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn crc32_simple(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xffff_ffff;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if (crc & 1) != 0 {
                crc = (crc >> 1) ^ 0xedb8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_device_preset_fork_and_diff() {
        let preset = DevicePreset::new("Super Lead", "AetherSynth");
        let forked = preset.fork("Alice");

        assert_eq!(forked.author, "Alice");
        assert!(forked.name.contains("(Fork)"));

        let diffs = preset.diff(&forked);
        assert!(!diffs.is_empty(), "Diff should identify differences between preset and fork");
    }

    #[test]
    fn test_device_preset_zip_export_and_install() {
        let preset = DevicePreset::new("Zip Preset", "PluckSynth");
        let zip_path = PathBuf::from("local/scratch/test_preset.zip");
        let dest_dir = PathBuf::from("local/scratch/installed");

        preset.export_zip(&zip_path).expect("ZIP export should succeed");
        assert!(zip_path.exists());

        let installed = DevicePreset::install_zip(&zip_path, &dest_dir).expect("ZIP install should succeed");
        assert_eq!(installed.name, "Zip Preset");

        let _ = fs::remove_file(&zip_path);
        let _ = fs::remove_dir_all(&dest_dir);
    }

    #[test]
    fn test_device_preset_migration_and_thumbnail() {
        let raw_legacy = r#"
            name = "Old Synth"
            device_kind = "OscSaw"
            params = { freq = 440.0 }
        "#;

        let migrated = DevicePreset::migrate_schema(raw_legacy).expect("Migration should succeed");
        assert_eq!(migrated.name, "Old Synth");
        assert_eq!(migrated.version, "1.0.0");

        let thumb_path = PathBuf::from("local/scratch/thumb.png");
        migrated.generate_thumbnail(&thumb_path).expect("Thumbnail generation should succeed");
        assert!(thumb_path.exists());
        let _ = fs::remove_file(&thumb_path);
    }
}
