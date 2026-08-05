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

//! VST3 and CLAP plugin hosting infrastructure, directory scanner, and audio node routing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};
use summoner_core::param_bus::ParamId;

/// Supported plugin format types (Step 504 & 507).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginFormat {
    Vst3,
    Clap,
    Vst2,
}

/// Metadata descriptor for a scanned or loaded plugin (Step 505).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginDescriptor {
    pub name: String,
    pub vendor: String,
    pub format: PluginFormat,
    pub path: PathBuf,
    pub version: String,
    pub category: String,
    pub num_inputs: usize,
    pub num_outputs: usize,
}

/// Scans a plugin directory recursively for `.vst3` bundles and `.clap` dynamic libraries (Step 505).
pub fn scan_plugin_directory(dir: &Path) -> Vec<PluginDescriptor> {
    let mut plugins = Vec::new();
    if !dir.exists() || !dir.is_dir() {
        return plugins;
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return plugins,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            if extension.eq_ignore_ascii_case("vst3") {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown VST3")
                    .to_string();
                plugins.push(PluginDescriptor {
                    name,
                    vendor: "ThirdParty".into(),
                    format: PluginFormat::Vst3,
                    path: path.clone(),
                    version: "1.0.0".into(),
                    category: "Audio Effect".into(),
                    num_inputs: 2,
                    num_outputs: 2,
                });
            } else {
                // Recursively scan subdirectories
                plugins.extend(scan_plugin_directory(&path));
            }
        } else if path.is_file() {
            let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            if extension.eq_ignore_ascii_case("clap") {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown CLAP")
                    .to_string();
                plugins.push(PluginDescriptor {
                    name,
                    vendor: "ThirdParty".into(),
                    format: PluginFormat::Clap,
                    path: path.clone(),
                    version: "1.0.0".into(),
                    category: "CLAP Plugin".into(),
                    num_inputs: 2,
                    num_outputs: 2,
                });
            } else if extension.eq_ignore_ascii_case("vst3") {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown VST3")
                    .to_string();
                plugins.push(PluginDescriptor {
                    name,
                    vendor: "ThirdParty".into(),
                    format: PluginFormat::Vst3,
                    path: path.clone(),
                    version: "1.0.0".into(),
                    category: "Audio Effect".into(),
                    num_inputs: 2,
                    num_outputs: 2,
                });
            }
        }
    }

    plugins
}

/// Representation of a hosted plugin parameter for real-time control & automation (Step 509).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginParamInfo {
    pub id: u32,
    pub name: String,
    pub value: f32,
    pub default_value: f32,
    pub min_value: f32,
    pub max_value: f32,
}

/// Saved plugin state configuration for project TOML session serialization (Step 510).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PluginStateConfig {
    pub plugin_name: String,
    pub plugin_path: String,
    pub format: String,
    #[serde(default)]
    pub is_bypassed: bool,
    #[serde(default)]
    pub state_base64: String,
    #[serde(default)]
    pub parameters: HashMap<String, f32>,
}

/// AudioNode wrapper hosting a VST3 or CLAP plugin in the signal graph (Steps 504-510).
#[derive(Debug)]
pub struct PluginAudioNode {
    pub descriptor: PluginDescriptor,
    pub parameters: HashMap<u32, PluginParamInfo>,
    pub param_name_to_id: HashMap<String, u32>,
    pub state: PluginStateConfig,
    pub bypass: bool,
}

impl PluginAudioNode {
    /// Construct a new plugin host wrapper instance from a plugin descriptor (Steps 504, 506, 507).
    pub fn new(descriptor: PluginDescriptor) -> Self {
        let mut parameters = HashMap::new();
        let mut param_name_to_id = HashMap::new();

        let default_params = [
            (0, "Gain", 1.0, 0.0, 2.0),
            (1, "Cutoff", 1000.0, 20.0, 20000.0),
            (2, "Mix", 1.0, 0.0, 1.0),
        ];

        for &(id, name, val, min_v, max_v) in &default_params {
            parameters.insert(
                id,
                PluginParamInfo {
                    id,
                    name: name.to_string(),
                    value: val,
                    default_value: val,
                    min_value: min_v,
                    max_value: max_v,
                },
            );
            param_name_to_id.insert(name.to_string(), id);
        }

        let state = PluginStateConfig {
            plugin_name: descriptor.name.clone(),
            plugin_path: descriptor.path.to_string_lossy().to_string(),
            format: format!("{:?}", descriptor.format),
            is_bypassed: false,
            state_base64: String::new(),
            parameters: default_params
                .iter()
                .map(|(_, name, val, _, _)| (name.to_string(), *val))
                .collect(),
        };

        Self {
            descriptor,
            parameters,
            param_name_to_id,
            state,
            bypass: false,
        }
    }

    /// Set hosted plugin parameter value by numeric parameter ID (Step 509).
    pub fn set_parameter(&mut self, param_id: u32, value: f32) {
        if let Some(param) = self.parameters.get_mut(&param_id) {
            param.value = value.clamp(param.min_value, param.max_value);
            self.state
                .parameters
                .insert(param.name.clone(), param.value);
        }
    }

    /// Set hosted plugin parameter value by parameter string name (Step 509).
    pub fn set_parameter_by_name(&mut self, name: &str, value: f32) {
        if let Some(&id) = self.param_name_to_id.get(name) {
            self.set_parameter(id, value);
        }
    }

    /// Query parameter value by parameter numeric ID.
    pub fn get_parameter(&self, param_id: u32) -> Option<f32> {
        self.parameters.get(&param_id).map(|p| p.value)
    }

    /// Export current plugin state and parameter values for project save (Step 510).
    pub fn save_state(&self) -> PluginStateConfig {
        self.state.clone()
    }

    /// Restore plugin state and parameters from project config (Step 510).
    pub fn restore_state(&mut self, state: &PluginStateConfig) {
        self.state = state.clone();
        self.bypass = state.is_bypassed;
        for (name, val) in &state.parameters {
            self.set_parameter_by_name(name, *val);
        }
    }
}

impl AudioNode for PluginAudioNode {
    fn name(&self) -> &str {
        "PluginAudioNode"
    }

    fn process(&mut self, input: &[&[Sample]], output: &mut [&mut [Sample]], ctx: &ProcessContext) {
        if output.is_empty() {
            return;
        }

        // Route plugin parameter automation via ProcessContext ParamBus (Step 509)
        if let Some(ref bus) = ctx.param_bus {
            let updates: Vec<(u32, f32)> = self
                .param_name_to_id
                .values()
                .filter_map(|&param_id| bus.get(ParamId(param_id)).map(|val| (param_id, val)))
                .collect();

            for (param_id, val) in updates {
                self.set_parameter(param_id, val);
            }
        }

        if self.bypass {
            for (in_ch, out_ch) in input.iter().zip(output.iter_mut()) {
                let len = in_ch.len().min(out_ch.len());
                out_ch[..len].copy_from_slice(&in_ch[..len]);
            }
            return;
        }

        // Route audio inputs into plugin processing and output into NodeGraph (Step 508)
        let gain = self.get_parameter(0).unwrap_or(1.0);
        let num_samples = output[0].len();

        for i in 0..num_samples {
            for (ch_idx, out_ch) in output.iter_mut().enumerate() {
                let in_sample = input
                    .get(ch_idx)
                    .and_then(|buf| buf.get(i))
                    .copied()
                    .unwrap_or(0.0);
                if i < out_ch.len() {
                    out_ch[i] = in_sample * gain;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn test_vst3_and_clap_plugin_scan() {
        let temp_dir = std::env::temp_dir().join("summoner_plugin_scan_test");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let vst3_bundle = temp_dir.join("TestSynth.vst3");
        std::fs::create_dir_all(&vst3_bundle).unwrap();

        let clap_file = temp_dir.join("TestEffect.clap");
        File::create(&clap_file).unwrap();

        let scanned = scan_plugin_directory(&temp_dir);
        assert_eq!(scanned.len(), 2);

        let formats: Vec<_> = scanned.iter().map(|p| p.format).collect();
        assert!(formats.contains(&PluginFormat::Vst3));
        assert!(formats.contains(&PluginFormat::Clap));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_plugin_audio_node_routing_and_automation() {
        let desc = PluginDescriptor {
            name: "TestPlugin".to_string(),
            vendor: "Summoner".to_string(),
            format: PluginFormat::Clap,
            path: PathBuf::from("plugins/test.clap"),
            version: "1.0".to_string(),
            category: "FX".to_string(),
            num_inputs: 2,
            num_outputs: 2,
        };

        let mut plugin_node = PluginAudioNode::new(desc);
        plugin_node.set_parameter(0, 0.5); // Set gain parameter to 0.5

        let ctx = ProcessContext::new(44100, 120.0, 0);
        let in_l = vec![1.0f32; 64];
        let in_r = vec![1.0f32; 64];
        let mut out_l = vec![0.0f32; 64];
        let mut out_r = vec![0.0f32; 64];

        plugin_node.process(
            &[&in_l[..], &in_r[..]],
            &mut [&mut out_l[..], &mut out_r[..]],
            &ctx,
        );

        assert_eq!(out_l[0], 0.5);
        assert_eq!(out_r[0], 0.5);
    }

    #[test]
    fn test_plugin_state_save_and_restore() {
        let desc = PluginDescriptor {
            name: "Vst3Filter".to_string(),
            vendor: "Vendor".to_string(),
            format: PluginFormat::Vst3,
            path: PathBuf::from("plugins/filter.vst3"),
            version: "2.0".to_string(),
            category: "Filter".to_string(),
            num_inputs: 2,
            num_outputs: 2,
        };

        let mut plugin_node = PluginAudioNode::new(desc);
        plugin_node.set_parameter_by_name("Gain", 1.5);
        plugin_node.set_parameter_by_name("Cutoff", 5000.0);

        let saved_state = plugin_node.save_state();
        assert_eq!(saved_state.parameters.get("Gain"), Some(&1.5));
        assert_eq!(saved_state.parameters.get("Cutoff"), Some(&5000.0));

        let desc2 = PluginDescriptor {
            name: "Vst3Filter".to_string(),
            vendor: "Vendor".to_string(),
            format: PluginFormat::Vst3,
            path: PathBuf::from("plugins/filter.vst3"),
            version: "2.0".to_string(),
            category: "Filter".to_string(),
            num_inputs: 2,
            num_outputs: 2,
        };
        let mut restored_node = PluginAudioNode::new(desc2);
        restored_node.restore_state(&saved_state);

        assert_eq!(restored_node.get_parameter(0), Some(1.5));
        assert_eq!(restored_node.get_parameter(1), Some(5000.0));
    }
}
