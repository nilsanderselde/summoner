//! User-device packaging and sharing format (Max for Live / Rack Extension analog).
//!
//! Provides a secure, self-contained container format (`.summondevice`) bundling:
//! - Device manifest with semantic versioning and metadata
//! - Real-time DSP source / node graph definitions
//! - Macro parameter bindings and normalization curves
//! - Embedded asset files (wavetables, impulse responses, sound fonts)
//! - Preset bank
//! - Cryptographic BLAKE3 integrity verification and capability-based sandbox validation.

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::io::{Cursor, Read, Write};
use zip::write::FileOptions;
use zip::{ZipArchive, ZipWriter};

/// Device category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceType {
    AudioEffect,
    Instrument,
    MidiProcessor,
    ControlModulator,
}

impl fmt::Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceType::AudioEffect => write!(f, "Audio Effect"),
            DeviceType::Instrument => write!(f, "Instrument"),
            DeviceType::MidiProcessor => write!(f, "MIDI Processor"),
            DeviceType::ControlModulator => write!(f, "Control Modulator"),
        }
    }
}

/// Sandbox capabilities granting permissions to user devices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeviceCapability {
    /// Pure mathematical DSP (zero side-effects).
    PureDsp,
    /// Permission to receive and output MIDI messages.
    MidiAccess,
    /// Permission to load and embed audio sample buffers.
    SampleLoading,
    /// Permission to bind external Open Sound Control (OSC) ports.
    NetworkOsc,
}

/// Semantic version (Major.Minor.Patch).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SemVer {
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn parse(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.trim().split('.').collect();
        if parts.len() != 3 {
            return Err(format!("Expected major.minor.patch, got '{}'", s));
        }
        let major = parts[0]
            .parse()
            .map_err(|e| format!("Invalid major: {}", e))?;
        let minor = parts[1]
            .parse()
            .map_err(|e| format!("Invalid minor: {}", e))?;
        let patch = parts[2]
            .parse()
            .map_err(|e| format!("Invalid patch: {}", e))?;
        Ok(Self::new(major, minor, patch))
    }

    /// Check if this version is backward-compatible with a requested minimum version.
    pub fn is_compatible_with(&self, target: &SemVer) -> bool {
        if self.major != target.major {
            return false;
        }
        if self.minor < target.minor {
            return false;
        }
        if self.minor == target.minor && self.patch < target.patch {
            return false;
        }
        true
    }
}

impl fmt::Display for SemVer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Exposed macro control parameter on the user device front panel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviceMacroParam {
    pub id: String,
    pub name: String,
    pub min_val: f32,
    pub max_val: f32,
    pub default_val: f32,
    pub unit: String,
}

impl DeviceMacroParam {
    pub fn new(
        id: &str,
        name: &str,
        min_val: f32,
        max_val: f32,
        default_val: f32,
        unit: &str,
    ) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            min_val,
            max_val,
            default_val: default_val.clamp(min_val, max_val),
            unit: unit.to_string(),
        }
    }

    pub fn normalized_to_value(&self, norm: f32) -> f32 {
        let n = norm.clamp(0.0, 1.0);
        self.min_val + n * (self.max_val - self.min_val)
    }

    pub fn value_to_normalized(&self, val: f32) -> f32 {
        if (self.max_val - self.min_val).abs() < 1e-6 {
            return 0.0;
        }
        let clamped = val.clamp(self.min_val, self.max_val);
        (clamped - self.min_val) / (self.max_val - self.min_val)
    }
}

/// Metadata manifest for a Summoner device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceManifest {
    pub id: String,
    pub name: String,
    pub author: String,
    pub version: SemVer,
    pub api_version: SemVer,
    pub device_type: DeviceType,
    pub capabilities: Vec<DeviceCapability>,
    pub checksum: String,
}

/// Errors occurring during packaging, unpackaging, or sandbox validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DevicePackageError {
    Serialization(String),
    Archive(String),
    ChecksumMismatch { expected: String, actual: String },
    IncompatibleApiVersion { required: SemVer, supported: SemVer },
    MissingManifest,
    SandboxViolation(String),
}

impl fmt::Display for DevicePackageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serialization(e) => write!(f, "Serialization error: {}", e),
            Self::Archive(e) => write!(f, "Archive error: {}", e),
            Self::ChecksumMismatch { expected, actual } => {
                write!(
                    f,
                    "Checksum mismatch: expected {}, got {}",
                    expected, actual
                )
            }
            Self::IncompatibleApiVersion {
                required,
                supported,
            } => {
                write!(
                    f,
                    "Incompatible API version: device requires {}, host supports {}",
                    required, supported
                )
            }
            Self::MissingManifest => write!(f, "Missing device.json manifest"),
            Self::SandboxViolation(e) => write!(f, "Sandbox violation: {}", e),
        }
    }
}

impl std::error::Error for DevicePackageError {}

/// Complete user device bundle with manifest, DSP program, parameters, assets, and presets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DevicePackage {
    pub manifest: DeviceManifest,
    pub dsp_code: String,
    pub parameters: Vec<DeviceMacroParam>,
    pub sample_assets: HashMap<String, Vec<u8>>,
    pub presets: HashMap<String, HashMap<String, f32>>,
}

impl DevicePackage {
    pub const CURRENT_API_VERSION: SemVer = SemVer::new(1, 0, 0);

    pub fn new(
        id: &str,
        name: &str,
        author: &str,
        version: SemVer,
        device_type: DeviceType,
        capabilities: Vec<DeviceCapability>,
        dsp_code: &str,
    ) -> Self {
        let manifest = DeviceManifest {
            id: id.to_string(),
            name: name.to_string(),
            author: author.to_string(),
            version,
            api_version: Self::CURRENT_API_VERSION,
            device_type,
            capabilities,
            checksum: String::new(),
        };

        let mut pkg = Self {
            manifest,
            dsp_code: dsp_code.to_string(),
            parameters: Vec::new(),
            sample_assets: HashMap::new(),
            presets: HashMap::new(),
        };
        pkg.update_checksum();
        pkg
    }

    pub fn add_parameter(&mut self, param: DeviceMacroParam) {
        self.parameters.push(param);
        self.update_checksum();
    }

    pub fn add_sample_asset(&mut self, name: &str, data: Vec<u8>) {
        self.sample_assets.insert(name.to_string(), data);
        self.update_checksum();
    }

    pub fn add_preset(&mut self, name: &str, values: HashMap<String, f32>) {
        self.presets.insert(name.to_string(), values);
        self.update_checksum();
    }

    /// Compute BLAKE3 digest of device contents (excluding existing manifest checksum).
    pub fn compute_checksum(&self) -> String {
        let mut hasher = Hasher::new();
        hasher.update(self.manifest.id.as_bytes());
        hasher.update(self.manifest.version.to_string().as_bytes());
        hasher.update(self.dsp_code.as_bytes());

        // Sort parameters by ID for deterministic hash
        let mut sorted_params = self.parameters.clone();
        sorted_params.sort_by(|a, b| a.id.cmp(&b.id));
        for p in &sorted_params {
            hasher.update(p.id.as_bytes());
            hasher.update(&p.min_val.to_le_bytes());
            hasher.update(&p.max_val.to_le_bytes());
            hasher.update(&p.default_val.to_le_bytes());
        }

        // Sort assets by name
        let mut asset_keys: Vec<&String> = self.sample_assets.keys().collect();
        asset_keys.sort();
        for k in asset_keys {
            hasher.update(k.as_bytes());
            hasher.update(&self.sample_assets[k]);
        }

        hasher.finalize().to_hex().to_string()
    }

    pub fn update_checksum(&mut self) {
        self.manifest.checksum = self.compute_checksum();
    }

    /// Pack device into a portable `.summondevice` ZIP archive buffer.
    pub fn export_to_bytes(&self) -> Result<Vec<u8>, DevicePackageError> {
        let mut buffer = Vec::new();
        {
            let mut zip = ZipWriter::new(Cursor::new(&mut buffer));
            let options =
                FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

            // 1. Write manifest
            zip.start_file("manifest.json", options)
                .map_err(|e| DevicePackageError::Archive(e.to_string()))?;
            let manifest_json = serde_json::to_string_pretty(&self.manifest)
                .map_err(|e| DevicePackageError::Serialization(e.to_string()))?;
            zip.write_all(manifest_json.as_bytes())
                .map_err(|e| DevicePackageError::Archive(e.to_string()))?;

            // 2. Write DSP code
            zip.start_file("dsp/program.gen", options)
                .map_err(|e| DevicePackageError::Archive(e.to_string()))?;
            zip.write_all(self.dsp_code.as_bytes())
                .map_err(|e| DevicePackageError::Archive(e.to_string()))?;

            // 3. Write parameters
            zip.start_file("params.json", options)
                .map_err(|e| DevicePackageError::Archive(e.to_string()))?;
            let params_json = serde_json::to_string_pretty(&self.parameters)
                .map_err(|e| DevicePackageError::Serialization(e.to_string()))?;
            zip.write_all(params_json.as_bytes())
                .map_err(|e| DevicePackageError::Archive(e.to_string()))?;

            // 4. Write presets
            zip.start_file("presets.json", options)
                .map_err(|e| DevicePackageError::Archive(e.to_string()))?;
            let presets_json = serde_json::to_string_pretty(&self.presets)
                .map_err(|e| DevicePackageError::Serialization(e.to_string()))?;
            zip.write_all(presets_json.as_bytes())
                .map_err(|e| DevicePackageError::Archive(e.to_string()))?;

            // 5. Write assets
            for (asset_name, asset_data) in &self.sample_assets {
                let path = format!("assets/{}", asset_name);
                zip.start_file(&path, options)
                    .map_err(|e| DevicePackageError::Archive(e.to_string()))?;
                zip.write_all(asset_data)
                    .map_err(|e| DevicePackageError::Archive(e.to_string()))?;
            }

            zip.finish()
                .map_err(|e| DevicePackageError::Archive(e.to_string()))?;
        }

        Ok(buffer)
    }

    /// Unpack and validate `.summondevice` ZIP archive buffer.
    pub fn import_from_bytes(
        bytes: &[u8],
        sandbox: Option<&DeviceSandbox>,
    ) -> Result<Self, DevicePackageError> {
        let cursor = Cursor::new(bytes);
        let mut zip =
            ZipArchive::new(cursor).map_err(|e| DevicePackageError::Archive(e.to_string()))?;

        // 1. Read manifest
        let manifest: DeviceManifest = {
            let mut file = zip
                .by_name("manifest.json")
                .map_err(|_| DevicePackageError::MissingManifest)?;
            let mut content = String::new();
            file.read_to_string(&mut content)
                .map_err(|e| DevicePackageError::Archive(e.to_string()))?;
            serde_json::from_str(&content)
                .map_err(|e| DevicePackageError::Serialization(e.to_string()))?
        };

        // Validate API compatibility
        if !Self::CURRENT_API_VERSION.is_compatible_with(&manifest.api_version) {
            return Err(DevicePackageError::IncompatibleApiVersion {
                required: manifest.api_version,
                supported: Self::CURRENT_API_VERSION,
            });
        }

        // 2. Read DSP code
        let dsp_code = {
            let mut file = zip
                .by_name("dsp/program.gen")
                .map_err(|e| DevicePackageError::Archive(e.to_string()))?;
            let mut content = String::new();
            file.read_to_string(&mut content)
                .map_err(|e| DevicePackageError::Archive(e.to_string()))?;
            content
        };

        // 3. Read parameters
        let parameters: Vec<DeviceMacroParam> = {
            if let Ok(mut file) = zip.by_name("params.json") {
                let mut content = String::new();
                file.read_to_string(&mut content)
                    .map_err(|e| DevicePackageError::Archive(e.to_string()))?;
                serde_json::from_str(&content)
                    .map_err(|e| DevicePackageError::Serialization(e.to_string()))?
            } else {
                Vec::new()
            }
        };

        // 4. Read presets
        let presets: HashMap<String, HashMap<String, f32>> = {
            if let Ok(mut file) = zip.by_name("presets.json") {
                let mut content = String::new();
                file.read_to_string(&mut content)
                    .map_err(|e| DevicePackageError::Archive(e.to_string()))?;
                serde_json::from_str(&content)
                    .map_err(|e| DevicePackageError::Serialization(e.to_string()))?
            } else {
                HashMap::new()
            }
        };

        // 5. Read assets
        let mut sample_assets = HashMap::new();
        for i in 0..zip.len() {
            let mut file = zip
                .by_index(i)
                .map_err(|e| DevicePackageError::Archive(e.to_string()))?;
            let name = file.name().to_string();
            if name.starts_with("assets/") && !name.ends_with('/') {
                let asset_name = name.trim_start_matches("assets/").to_string();
                let mut data = Vec::new();
                file.read_to_end(&mut data)
                    .map_err(|e| DevicePackageError::Archive(e.to_string()))?;
                sample_assets.insert(asset_name, data);
            }
        }

        let pkg = Self {
            manifest: manifest.clone(),
            dsp_code,
            parameters,
            sample_assets,
            presets,
        };

        // Verify cryptographic BLAKE3 checksum
        let computed = pkg.compute_checksum();
        if computed != manifest.checksum {
            return Err(DevicePackageError::ChecksumMismatch {
                expected: manifest.checksum,
                actual: computed,
            });
        }

        // Enforce sandbox restrictions if provided
        if let Some(sb) = sandbox {
            sb.validate_package(&pkg)?;
        }

        Ok(pkg)
    }
}

/// Device sandbox enforcing security boundaries and quota limits.
#[derive(Debug, Clone)]
pub struct DeviceSandbox {
    pub max_memory_bytes: usize,
    pub max_asset_count: usize,
    pub allowed_capabilities: Vec<DeviceCapability>,
}

impl Default for DeviceSandbox {
    fn default() -> Self {
        Self {
            max_memory_bytes: 64 * 1024 * 1024, // 64 MB maximum asset quota
            max_asset_count: 32,
            allowed_capabilities: vec![
                DeviceCapability::PureDsp,
                DeviceCapability::MidiAccess,
                DeviceCapability::SampleLoading,
            ],
        }
    }
}

impl DeviceSandbox {
    pub fn new(max_memory_bytes: usize, max_asset_count: usize) -> Self {
        Self {
            max_memory_bytes,
            max_asset_count,
            allowed_capabilities: vec![
                DeviceCapability::PureDsp,
                DeviceCapability::MidiAccess,
                DeviceCapability::SampleLoading,
            ],
        }
    }

    pub fn allow_capability(&mut self, cap: DeviceCapability) {
        if !self.allowed_capabilities.contains(&cap) {
            self.allowed_capabilities.push(cap);
        }
    }

    pub fn validate_package(&self, pkg: &DevicePackage) -> Result<(), DevicePackageError> {
        // 1. Verify all requested capabilities are permitted
        for cap in &pkg.manifest.capabilities {
            if !self.allowed_capabilities.contains(cap) {
                return Err(DevicePackageError::SandboxViolation(format!(
                    "Capability {:?} is prohibited by current host sandbox policy",
                    cap
                )));
            }
        }

        // 2. Verify asset count
        if pkg.sample_assets.len() > self.max_asset_count {
            return Err(DevicePackageError::SandboxViolation(format!(
                "Asset count ({}) exceeds sandbox maximum limit ({})",
                pkg.sample_assets.len(),
                self.max_asset_count
            )));
        }

        // 3. Verify total asset memory
        let total_bytes: usize = pkg.sample_assets.values().map(|v| v.len()).sum();
        if total_bytes > self.max_memory_bytes {
            return Err(DevicePackageError::SandboxViolation(format!(
                "Asset payload size ({} bytes) exceeds sandbox quota ({} bytes)",
                total_bytes, self.max_memory_bytes
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_parsing_and_compatibility() {
        let v1_0_0 = SemVer::parse("1.0.0").unwrap();
        let v1_2_1 = SemVer::parse("1.2.1").unwrap();
        let v2_0_0 = SemVer::parse("2.0.0").unwrap();

        assert_eq!(v1_0_0.to_string(), "1.0.0");
        assert!(v1_2_1.is_compatible_with(&v1_0_0));
        assert!(!v1_0_0.is_compatible_with(&v1_2_1));
        assert!(!v2_0_0.is_compatible_with(&v1_2_1));
    }

    #[test]
    fn test_device_macro_param_normalization() {
        let param = DeviceMacroParam::new("cutoff", "Cutoff Freq", 20.0, 20000.0, 1000.0, "Hz");
        assert_eq!(param.normalized_to_value(0.0), 20.0);
        assert_eq!(param.normalized_to_value(1.0), 20000.0);
        let mid = param.normalized_to_value(0.5);
        assert!((mid - 10010.0).abs() < 1e-4);
        assert!((param.value_to_normalized(mid) - 0.5).abs() < 1e-4);
    }

    #[test]
    fn test_device_package_export_and_import() {
        let dsp_code = "out1 = in1 * 0.7; out2 = in2 * 0.7;";
        let mut pkg = DevicePackage::new(
            "org.summoner.stereo_dim",
            "Stereo Dimmer",
            "Summoner Audio",
            SemVer::new(1, 0, 0),
            DeviceType::AudioEffect,
            vec![DeviceCapability::PureDsp],
            dsp_code,
        );

        pkg.add_parameter(DeviceMacroParam::new(
            "dim_amount",
            "Dim Amount",
            0.0,
            1.0,
            0.7,
            "%",
        ));
        pkg.add_sample_asset("impulse.raw", vec![1, 2, 3, 4, 5, 6, 7, 8]);

        let mut preset_vals = HashMap::new();
        preset_vals.insert("dim_amount".to_string(), 0.5);
        pkg.add_preset("Gentle Dim", preset_vals);

        let bytes = pkg.export_to_bytes().expect("Export should succeed");
        assert!(!bytes.is_empty());

        let imported =
            DevicePackage::import_from_bytes(&bytes, None).expect("Import should succeed");
        assert_eq!(imported.manifest.id, "org.summoner.stereo_dim");
        assert_eq!(imported.manifest.name, "Stereo Dimmer");
        assert_eq!(imported.dsp_code, dsp_code);
        assert_eq!(imported.parameters.len(), 1);
        assert_eq!(imported.sample_assets.len(), 1);
        assert_eq!(
            imported.sample_assets["impulse.raw"],
            vec![1, 2, 3, 4, 5, 6, 7, 8]
        );
        assert_eq!(imported.presets.len(), 1);
    }

    #[test]
    fn test_device_tampering_fails_checksum() {
        let mut pkg = DevicePackage::new(
            "org.summoner.tamper_test",
            "Tamper Test",
            "Auditor",
            SemVer::new(1, 0, 0),
            DeviceType::AudioEffect,
            vec![DeviceCapability::PureDsp],
            "out1 = in1;",
        );
        let _bytes = pkg.export_to_bytes().unwrap();

        // Deliberately corrupt package by re-packaging with modified manifest checksum
        pkg.manifest.checksum =
            "0000000000000000000000000000000000000000000000000000000000000000".to_string();
        let bad_bytes = pkg.export_to_bytes().unwrap();

        let res = DevicePackage::import_from_bytes(&bad_bytes, None);
        assert!(matches!(
            res,
            Err(DevicePackageError::ChecksumMismatch { .. })
        ));
    }

    #[test]
    fn test_device_sandbox_capability_and_quota_enforcement() {
        let mut pkg = DevicePackage::new(
            "org.summoner.networked_device",
            "Networked Synth",
            "Auditor",
            SemVer::new(1, 0, 0),
            DeviceType::Instrument,
            vec![DeviceCapability::NetworkOsc], // NetworkOsc not allowed in default sandbox
            "out1 = in1;",
        );

        let bytes = pkg.export_to_bytes().unwrap();
        let default_sandbox = DeviceSandbox::default();

        let res = DevicePackage::import_from_bytes(&bytes, Some(&default_sandbox));
        assert!(matches!(res, Err(DevicePackageError::SandboxViolation(_))));

        // When permitted:
        let mut open_sandbox = DeviceSandbox::default();
        open_sandbox.allow_capability(DeviceCapability::NetworkOsc);
        let ok_res = DevicePackage::import_from_bytes(&bytes, Some(&open_sandbox));
        assert!(ok_res.is_ok());

        // Test quota overflow
        let tiny_sandbox = DeviceSandbox::new(10, 5); // 10 bytes max
        pkg.manifest.capabilities = vec![DeviceCapability::SampleLoading];
        pkg.add_sample_asset("large.bin", vec![0u8; 100]); // 100 bytes > 10 bytes
        let bytes_large = pkg.export_to_bytes().unwrap();
        let quota_res = DevicePackage::import_from_bytes(&bytes_large, Some(&tiny_sandbox));
        assert!(matches!(
            quota_res,
            Err(DevicePackageError::SandboxViolation(_))
        ));
    }
}
