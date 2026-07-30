// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

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
        }
    }

    pub fn save_preset<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let toml_string = toml::to_string_pretty(self)?;
        fs::write(path, toml_string)?;
        Ok(())
    }

    pub fn load_preset<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let toml_string = fs::read_to_string(path)?;
        let preset: Self = toml::from_str(&toml_string)?;
        Ok(preset)
    }
}
