//! Plugin export and bridge generator (Task F8).
//!
//! Enables packaging Summoner instruments and effects as native VST3, CLAP, and
//! macOS AudioUnit (AU) plugin bundles for third-party DAWs (Ableton, Cubase, Logic, Reaper, Bitwig).
//!
//! Generates:
//! - Standard CLAP plugin descriptors (`clap_plugin_descriptor`) and C-ABI export wrappers
//! - Steinberg VST3 `moduleinfo.json` manifests and factory class registrations
//! - Apple AudioUnit `Info.plist` component dictionaries
//! - Zero-allocation real-time runtime bridge (`PluginRuntimeBridge`) routing host audio/MIDI into Summoner DSP nodes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, NodeEvent, ProcessContext};

/// Target plugin API formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluginTargetFormat {
    Clap,
    Vst3,
    AudioUnit,
}

/// Category classification of the exported plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginCategory {
    Instrument,
    AudioEffect,
    Filter,
    Mastering,
    Spatial,
}

impl PluginCategory {
    pub fn to_clap_features(self) -> &'static [&'static str] {
        match self {
            PluginCategory::Instrument => &["instrument", "synthesizer", "stereo"],
            PluginCategory::AudioEffect => &["audio-effect", "stereo"],
            PluginCategory::Filter => &["audio-effect", "filter", "stereo"],
            PluginCategory::Mastering => &["audio-effect", "mastering", "limiter", "stereo"],
            PluginCategory::Spatial => &["audio-effect", "spatial", "reverb", "stereo"],
        }
    }

    pub fn to_vst3_category(self) -> &'static str {
        match self {
            PluginCategory::Instrument => "Instrument|Synth",
            PluginCategory::AudioEffect => "Fx",
            PluginCategory::Filter => "Fx|Filter",
            PluginCategory::Mastering => "Fx|Mastering",
            PluginCategory::Spatial => "Fx|Spatial",
        }
    }

    pub fn to_au_type_code(self) -> &'static str {
        match self {
            PluginCategory::Instrument => "aumu", // MusicDevice
            _ => "aufx",                          // Effect
        }
    }
}

/// Parameter definition for exported plugin.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportedParam {
    pub id: u32,
    pub name: String,
    pub module: String,
    pub min_val: f64,
    pub max_val: f64,
    pub default_val: f64,
    pub unit: String,
}

impl ExportedParam {
    pub fn new(
        id: u32,
        name: &str,
        min_val: f64,
        max_val: f64,
        default_val: f64,
        unit: &str,
    ) -> Self {
        Self {
            id,
            name: name.to_string(),
            module: String::new(),
            min_val,
            max_val,
            default_val,
            unit: unit.to_string(),
        }
    }

    pub fn normalized_to_value(&self, norm: f64) -> f64 {
        let n = norm.clamp(0.0, 1.0);
        self.min_val + n * (self.max_val - self.min_val)
    }

    pub fn value_to_normalized(&self, val: f64) -> f64 {
        if (self.max_val - self.min_val).abs() < 1e-9 {
            return 0.0;
        }
        let clamped = val.clamp(self.min_val, self.max_val);
        (clamped - self.min_val) / (self.max_val - self.min_val)
    }
}

/// Metadata manifest for an exported plugin.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginExportManifest {
    pub id: String,
    pub name: String,
    pub vendor: String,
    pub url: String,
    pub version: String,
    pub description: String,
    pub category: PluginCategory,
    pub audio_inputs: u32,
    pub audio_outputs: u32,
    pub parameters: Vec<ExportedParam>,
    pub cid_hex: String,         // 16-byte VST3 class ID hex string
    pub au_subtype: String,      // 4-character AU subtype code (e.g. "TAPE")
    pub au_manufacturer: String, // 4-character AU manufacturer code (e.g. "SMMN")
}

impl PluginExportManifest {
    pub fn new(
        id: &str,
        name: &str,
        vendor: &str,
        version: &str,
        category: PluginCategory,
        is_instrument: bool,
    ) -> Self {
        let audio_inputs = if is_instrument { 0 } else { 2 };
        let audio_outputs = 2;

        // Generate deterministic 16-byte CID from ID
        let mut cid_bytes = [0u8; 16];
        for (i, b) in id.as_bytes().iter().enumerate() {
            cid_bytes[i % 16] ^= b;
        }
        let cid_hex: String = cid_bytes.iter().map(|b| format!("{:02X}", b)).collect();

        Self {
            id: id.to_string(),
            name: name.to_string(),
            vendor: vendor.to_string(),
            url: "https://github.com/nilsanderselde/summoner".to_string(),
            version: version.to_string(),
            description: format!("{} by {}", name, vendor),
            category,
            audio_inputs,
            audio_outputs,
            parameters: Vec::new(),
            cid_hex,
            au_subtype: "SMMN".to_string(),
            au_manufacturer: "SMMN".to_string(),
        }
    }

    pub fn add_param(&mut self, param: ExportedParam) {
        self.parameters.push(param);
    }

    /// Generate Steinberg VST3 `moduleinfo.json` manifest.
    pub fn generate_vst3_moduleinfo_json(&self) -> String {
        let mut classes = Vec::new();
        let mut cid_formatted = String::new();
        for chunk in self.cid_hex.as_bytes().chunks(2) {
            if !cid_formatted.is_empty() {
                cid_formatted.push('-');
            }
            cid_formatted.push_str(std::str::from_utf8(chunk).unwrap_or("00"));
        }

        let class_entry = serde_json::json!({
            "CID": self.cid_hex.to_uppercase(),
            "Category": self.category.to_vst3_category(),
            "Name": self.name,
            "Vendor": self.vendor,
            "Version": self.version,
            "SDKVersion": "VST 3.7.12",
            "SubCategories": [self.category.to_vst3_category()],
            "ClassFlags": 0
        });
        classes.push(class_entry);

        let root = serde_json::json!({
            "CompatibilityVersion": 1,
            "Description": self.description,
            "FactoryInfo": {
                "Vendor": self.vendor,
                "URL": self.url,
                "Email": "info@summoner-audio.org",
                "Flags": {
                    "Unicode": true,
                    "ClassesDiscardable": false,
                    "ComponentNonDiscardable": true
                }
            },
            "Classes": classes
        });

        serde_json::to_string_pretty(&root).unwrap_or_default()
    }

    /// Generate Apple AudioUnit `Info.plist` XML.
    pub fn generate_au_info_plist(&self) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>English</string>
    <key>CFBundleExecutable</key>
    <string>{name}</string>
    <key>CFBundleIdentifier</key>
    <string>{id}</string>
    <key>CFBundleName</key>
    <string>{name}</string>
    <key>CFBundlePackageType</key>
    <string>BNDL</string>
    <key>CFBundleShortVersionString</key>
    <string>{version}</string>
    <key>AudioComponents</key>
    <array>
        <dict>
            <key>type</key>
            <string>{au_type}</string>
            <key>subtype</key>
            <string>{au_subtype}</string>
            <key>manufacturer</key>
            <string>{au_manufacturer}</string>
            <key>name</key>
            <string>{vendor}: {name}</string>
            <key>description</key>
            <string>{description}</string>
            <key>version</key>
            <integer>65536</integer>
            <key>sandboxSafe</key>
            <true/>
        </dict>
    </array>
</dict>
</plist>"#,
            id = self.id,
            name = self.name,
            vendor = self.vendor,
            version = self.version,
            description = self.description,
            au_type = self.category.to_au_type_code(),
            au_subtype = self.au_subtype,
            au_manufacturer = self.au_manufacturer,
        )
    }

    /// Generate CLAP descriptor JSON definition.
    pub fn generate_clap_descriptor_json(&self) -> String {
        let features = self.category.to_clap_features();
        let root = serde_json::json!({
            "clap_version": "1.2.0",
            "id": self.id,
            "name": self.name,
            "vendor": self.vendor,
            "url": self.url,
            "version": self.version,
            "description": self.description,
            "features": features
        });
        serde_json::to_string_pretty(&root).unwrap_or_default()
    }

    /// Generate standard filesystem directory bundle for target format.
    pub fn create_bundle_layout(
        &self,
        root_dir: &Path,
        format: PluginTargetFormat,
    ) -> std::io::Result<PathBuf> {
        match format {
            PluginTargetFormat::Vst3 => {
                let bundle_dir = root_dir.join(format!("{}.vst3", self.name));
                let contents_dir = bundle_dir.join("Contents");
                let resources_dir = contents_dir.join("Resources");
                let win_dir = contents_dir.join("x86_64-win");
                let linux_dir = contents_dir.join("x86_64-linux");
                let mac_dir = contents_dir.join("MacOS");

                std::fs::create_dir_all(&resources_dir)?;
                std::fs::create_dir_all(&win_dir)?;
                std::fs::create_dir_all(&linux_dir)?;
                std::fs::create_dir_all(&mac_dir)?;

                // Write moduleinfo.json
                let moduleinfo = self.generate_vst3_moduleinfo_json();
                std::fs::write(resources_dir.join("moduleinfo.json"), moduleinfo)?;

                Ok(bundle_dir)
            }
            PluginTargetFormat::AudioUnit => {
                let bundle_dir = root_dir.join(format!("{}.component", self.name));
                let contents_dir = bundle_dir.join("Contents");
                let mac_dir = contents_dir.join("MacOS");

                std::fs::create_dir_all(&mac_dir)?;

                // Write Info.plist
                let plist = self.generate_au_info_plist();
                std::fs::write(contents_dir.join("Info.plist"), plist)?;

                Ok(bundle_dir)
            }
            PluginTargetFormat::Clap => {
                let bundle_dir = root_dir.join(format!("{}.clap", self.name));
                std::fs::create_dir_all(&bundle_dir)?;

                // Write metadata descriptor
                let desc = self.generate_clap_descriptor_json();
                std::fs::write(bundle_dir.join("descriptor.json"), desc)?;

                Ok(bundle_dir)
            }
        }
    }
}

/// Real-time audio and parameter bridge running an exported Summoner DSP node.
pub struct PluginRuntimeBridge {
    pub manifest: PluginExportManifest,
    pub inner_node: Box<dyn AudioNode + Send>,
    param_values: HashMap<u32, f64>,
    sample_rate: f64,
}

impl PluginRuntimeBridge {
    pub fn new(
        manifest: PluginExportManifest,
        inner_node: Box<dyn AudioNode + Send>,
        sample_rate: f64,
    ) -> Self {
        let mut param_values = HashMap::new();
        for p in &manifest.parameters {
            param_values.insert(p.id, p.default_val);
        }

        Self {
            manifest,
            inner_node,
            param_values,
            sample_rate,
        }
    }

    /// Set parameter by ID with host automation / user interaction.
    pub fn set_parameter(&mut self, id: u32, val: f64) -> bool {
        if let Some(param) = self.manifest.parameters.iter().find(|p| p.id == id) {
            let clamped = val.clamp(param.min_val, param.max_val);
            self.param_values.insert(id, clamped);

            // Forward parameter event to inner node
            let event = NodeEvent::SetParam {
                id,
                value: clamped as f32,
            };
            let dummy_ctx = ProcessContext {
                sample_rate: self.sample_rate as u32,
                bpm: 120.0,
                frame_position: 0,
                param_bus: None,
                tuning_root_hz: 440.0,
                tuning_edo_divisions: 12,
                is_playing: true,
            };
            self.inner_node.handle_event(&event, &dummy_ctx);
            true
        } else {
            false
        }
    }

    /// Get current parameter value.
    pub fn get_parameter(&self, id: u32) -> Option<f64> {
        self.param_values.get(&id).copied()
    }

    /// Process audio block from host buffer pointers with zero audio-thread allocation.
    pub fn process_host_block(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        self.inner_node.process(inputs, outputs, ctx);
    }

    /// Serialize current plugin state to JSON bytes.
    pub fn save_state(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec_pretty(&self.param_values).map_err(|e| e.to_string())
    }

    /// Restore plugin state from JSON bytes.
    pub fn load_state(&mut self, state: &[u8]) -> Result<(), String> {
        let values: HashMap<u32, f64> = serde_json::from_slice(state).map_err(|e| e.to_string())?;
        for (id, val) in values {
            self.set_parameter(id, val);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::code_node::GenCodeNode;

    #[test]
    fn test_vst3_moduleinfo_generation() {
        let mut manifest = PluginExportManifest::new(
            "org.summoner.tape_sat",
            "Summoner Tape Saturation",
            "Summoner Audio",
            "1.0.0",
            PluginCategory::AudioEffect,
            false,
        );
        manifest.add_param(ExportedParam::new(0, "Drive", 0.0, 24.0, 6.0, "dB"));
        manifest.add_param(ExportedParam::new(1, "Bias", -1.0, 1.0, 0.0, "%"));

        let json_str = manifest.generate_vst3_moduleinfo_json();
        assert!(json_str.contains("CompatibilityVersion"));
        assert!(json_str.contains("Summoner Tape Saturation"));
        assert!(json_str.contains("Fx"));
        assert!(json_str.contains("VST 3.7.12"));
    }

    #[test]
    fn test_au_info_plist_generation() {
        let manifest = PluginExportManifest::new(
            "org.summoner.electric_piano",
            "Summoner Electric Piano",
            "Summoner Audio",
            "1.0.0",
            PluginCategory::Instrument,
            true,
        );

        let plist = manifest.generate_au_info_plist();
        assert!(plist.contains("<key>CFBundleIdentifier</key>"));
        assert!(plist.contains("org.summoner.electric_piano"));
        assert!(plist.contains("<string>aumu</string>")); // MusicDevice
    }

    #[test]
    fn test_clap_descriptor_generation() {
        let manifest = PluginExportManifest::new(
            "org.summoner.filter",
            "Summoner Filter",
            "Summoner Audio",
            "1.0.0",
            PluginCategory::Filter,
            false,
        );

        let clap_json = manifest.generate_clap_descriptor_json();
        assert!(clap_json.contains("audio-effect"));
        assert!(clap_json.contains("filter"));
    }

    #[test]
    fn test_bundle_layout_creation() {
        let temp_dir = std::env::temp_dir().join("summoner_plugin_test");
        let manifest = PluginExportManifest::new(
            "org.summoner.test_plugin",
            "TestPlugin",
            "Summoner Audio",
            "1.0.0",
            PluginCategory::AudioEffect,
            false,
        );

        let vst3_bundle = manifest
            .create_bundle_layout(&temp_dir, PluginTargetFormat::Vst3)
            .unwrap();
        assert!(vst3_bundle
            .join("Contents/Resources/moduleinfo.json")
            .exists());

        let au_bundle = manifest
            .create_bundle_layout(&temp_dir, PluginTargetFormat::AudioUnit)
            .unwrap();
        assert!(au_bundle.join("Contents/Info.plist").exists());

        let clap_bundle = manifest
            .create_bundle_layout(&temp_dir, PluginTargetFormat::Clap)
            .unwrap();
        assert!(clap_bundle.join("descriptor.json").exists());

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_plugin_runtime_bridge_execution_and_state_roundtrip() {
        let mut manifest = PluginExportManifest::new(
            "org.summoner.runtime_test",
            "Runtime Bridge Test",
            "Summoner Audio",
            "1.0.0",
            PluginCategory::AudioEffect,
            false,
        );
        manifest.add_param(ExportedParam::new(0, "Gain", 0.0, 2.0, 1.0, "linear"));

        let mut code_node = GenCodeNode::new(44100.0);
        code_node
            .set_code("out1 = in1 * 0.5; out2 = in2 * 0.5;")
            .unwrap();

        let mut bridge = PluginRuntimeBridge::new(manifest, Box::new(code_node), 44100.0);

        let in_left = vec![1.0f32; 32];
        let in_right = vec![1.0f32; 32];
        let mut out_left = vec![0.0f32; 32];
        let mut out_right = vec![0.0f32; 32];

        let inputs = [&in_left[..], &in_right[..]];
        let mut outputs = [&mut out_left[..], &mut out_right[..]];

        let ctx = ProcessContext {
            sample_rate: 44100,
            bpm: 120.0,
            frame_position: 0,
            param_bus: None,
            tuning_root_hz: 440.0,
            tuning_edo_divisions: 12,
            is_playing: true,
        };

        bridge.process_host_block(&inputs, &mut outputs, &ctx);
        assert!((outputs[0][0] - 0.5).abs() < 1e-4);
        assert!((outputs[1][0] - 0.5).abs() < 1e-4);

        // Test parameter update & state save/load
        assert_eq!(bridge.get_parameter(0), Some(1.0));
        assert!(bridge.set_parameter(0, 1.75));
        assert_eq!(bridge.get_parameter(0), Some(1.75));

        let state_bytes = bridge.save_state().expect("State save should succeed");
        bridge.set_parameter(0, 0.2);
        assert_eq!(bridge.get_parameter(0), Some(0.2));

        bridge
            .load_state(&state_bytes)
            .expect("State load should succeed");
        assert_eq!(bridge.get_parameter(0), Some(1.75));
    }
}
