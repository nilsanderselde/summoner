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

//! Step 1250: Auto-update checking, channel selection (stable/beta/nightly),
//! and opt-in telemetry / crash reporting dispatcher integrated with `CrashDump`.

use crate::crash_analyzer::{CrashDump, CrashSeverity};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Release channel for software updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum UpdateChannel {
    #[default]
    Stable,
    Beta,
    Nightly,
}

impl std::fmt::Display for UpdateChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateChannel::Stable => write!(f, "Stable"),
            UpdateChannel::Beta => write!(f, "Beta"),
            UpdateChannel::Nightly => write!(f, "Nightly"),
        }
    }
}

/// Simple Semantic Versioning structure for reliable version comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateSemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre_release: Option<String>,
}

impl UpdateSemVer {
    pub fn parse(s: &str) -> Result<Self, String> {
        let trimmed = s.trim().trim_start_matches('v');
        let (num_part, pre_part) = match trimmed.split_once('-') {
            Some((nums, pre)) => (nums, Some(pre.to_string())),
            None => (trimmed, None),
        };

        let parts: Vec<&str> = num_part.split('.').collect();
        if parts.len() < 3 {
            return Err(format!(
                "Invalid semver string '{}': expected major.minor.patch",
                s
            ));
        }

        let major = parts[0]
            .parse::<u32>()
            .map_err(|e| format!("Invalid major version: {}", e))?;
        let minor = parts[1]
            .parse::<u32>()
            .map_err(|e| format!("Invalid minor version: {}", e))?;
        let patch = parts[2]
            .parse::<u32>()
            .map_err(|e| format!("Invalid patch version: {}", e))?;

        Ok(Self {
            major,
            minor,
            patch,
            pre_release: pre_part,
        })
    }
}

impl PartialOrd for UpdateSemVer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UpdateSemVer {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.minor.cmp(&other.minor) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.patch.cmp(&other.patch) {
            Ordering::Equal => {}
            ord => return ord,
        }
        // Pre-release: version with pre-release is LOWER than release without pre-release
        match (&self.pre_release, &other.pre_release) {
            (None, None) => Ordering::Equal,
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (Some(a), Some(b)) => a.cmp(b),
        }
    }
}

/// Software update manifest published to update servers.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateManifest {
    pub version: String,
    pub channel: UpdateChannel,
    pub release_date: String,
    pub release_notes: String,
    pub download_url: String,
    pub blake3_checksum: String,
    pub min_os_version: HashMap<String, String>,
    pub is_security_critical: bool,
    #[serde(default)]
    pub signature_hex: String,
    #[serde(default)]
    pub signer_public_key_hex: String,
}

impl UpdateManifest {
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| e.to_string())
    }

    fn canonical_signing_bytes(&self) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(self.version.as_bytes());
        payload.push(0);
        payload.extend_from_slice(self.download_url.as_bytes());
        payload.push(0);
        payload.extend_from_slice(self.blake3_checksum.as_bytes());
        payload.push(0);
        payload.push(self.channel as u8);
        payload
    }

    /// Sign manifest metadata with a 32-byte release signing key using Winternitz/BLAKE3
    /// hash-based asymmetric signature (`M3.4` / `PLT-7`).
    pub fn sign_with_key(&mut self, signing_key: &[u8; 32]) {
        self.signer_public_key_hex = wots_public_key_hex(signing_key);
        self.signature_hex = wots_sign_hex(signing_key, &self.canonical_signing_bytes());
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

fn hex_decode(hex: &str) -> Option<Vec<u8>> {
    if !hex.len().is_multiple_of(2) {
        return None;
    }
    let bytes = hex.as_bytes();
    let mut out = Vec::with_capacity(bytes.len() / 2);
    for pair in bytes.chunks_exact(2) {
        let hi = (pair[0] as char).to_digit(16)? as u8;
        let lo = (pair[1] as char).to_digit(16)? as u8;
        out.push((hi << 4) | lo);
    }
    Some(out)
}

fn wots_nibbles(msg: &[u8]) -> [u8; 67] {
    let digest = blake3::hash(msg);
    let mut nibbles = [0u8; 67];
    let mut sum: u32 = 0;
    for (idx, &b) in digest.as_bytes().iter().enumerate() {
        let hi = (b >> 4) & 0x0f;
        let lo = b & 0x0f;
        nibbles[idx * 2] = hi;
        nibbles[idx * 2 + 1] = lo;
        sum += hi as u32 + lo as u32;
    }
    let csum = (64 * 15) - sum;
    nibbles[64] = ((csum >> 8) & 0x0f) as u8;
    nibbles[65] = ((csum >> 4) & 0x0f) as u8;
    nibbles[66] = (csum & 0x0f) as u8;
    nibbles
}

fn hash_chain(mut val: [u8; 32], steps: u8) -> [u8; 32] {
    for _ in 0..steps {
        val = *blake3::hash(&val).as_bytes();
    }
    val
}

fn wots_public_key_hex(signing_key: &[u8; 32]) -> String {
    let mut pk_hasher = blake3::Hasher::new();
    for i in 0u8..67u8 {
        let sk = *blake3::keyed_hash(signing_key, &[i]).as_bytes();
        let pk_i = hash_chain(sk, 15);
        pk_hasher.update(&pk_i);
    }
    pk_hasher.finalize().to_hex().to_string()
}

fn wots_sign_hex(signing_key: &[u8; 32], msg: &[u8]) -> String {
    let nibbles = wots_nibbles(msg);
    let mut raw = Vec::with_capacity(67 * 32);
    for (i, &n) in nibbles.iter().enumerate() {
        let sk = *blake3::keyed_hash(signing_key, &[i as u8]).as_bytes();
        let sig_i = hash_chain(sk, n);
        raw.extend_from_slice(&sig_i);
    }
    hex_encode(&raw)
}

fn wots_verify_hex(msg: &[u8], sig_hex: &str, trusted_pub_key_hex: &str) -> bool {
    let Some(sig_bytes) = hex_decode(sig_hex) else {
        return false;
    };
    if sig_bytes.len() != 67 * 32 {
        return false;
    }
    let nibbles = wots_nibbles(msg);
    let mut pk_hasher = blake3::Hasher::new();
    for (i, (&n, chunk)) in nibbles.iter().zip(sig_bytes.chunks_exact(32)).enumerate() {
        let _ = i;
        let mut arr = [0u8; 32];
        arr.copy_from_slice(chunk);
        let derived_pk_i = hash_chain(arr, 15 - n);
        pk_hasher.update(&derived_pk_i);
    }
    let derived_pub_hex = pk_hasher.finalize().to_hex().to_string();
    derived_pub_hex.eq_ignore_ascii_case(trusted_pub_key_hex)
}

/// Result of evaluating an update check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckResult {
    pub update_available: bool,
    pub current_version: String,
    pub latest_manifest: Option<UpdateManifest>,
    pub reason: String,
}

/// Software auto-update checker and channel manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoUpdateChecker {
    pub current_version: String,
    pub channel: UpdateChannel,
    pub auto_check_enabled: bool,
    pub feed_url: String,
    pub last_checked: Option<String>,
}

impl AutoUpdateChecker {
    pub fn new(current_version: &str, channel: UpdateChannel) -> Self {
        Self {
            current_version: current_version.to_string(),
            channel,
            auto_check_enabled: true,
            feed_url: "https://updates.summoner-audio.org/releases/manifest.json".to_string(),
            last_checked: None,
        }
    }

    pub fn with_feed_url(mut self, url: &str) -> Self {
        self.feed_url = url.to_string();
        self
    }

    pub fn set_channel(&mut self, channel: UpdateChannel) {
        self.channel = channel;
    }

    /// Evaluates a candidate update manifest against current version and subscribed channel.
    pub fn evaluate_manifest(&mut self, manifest: &UpdateManifest) -> UpdateCheckResult {
        self.last_checked = Some("2026-11-10T18:00:00Z".to_string());

        // Channel compatibility:
        // Stable subscriber only receives Stable updates.
        // Beta subscriber accepts Beta and Stable updates.
        // Nightly accepts all.
        let channel_match = match self.channel {
            UpdateChannel::Stable => manifest.channel == UpdateChannel::Stable,
            UpdateChannel::Beta => {
                manifest.channel == UpdateChannel::Beta || manifest.channel == UpdateChannel::Stable
            }
            UpdateChannel::Nightly => true,
        };

        if !channel_match {
            return UpdateCheckResult {
                update_available: false,
                current_version: self.current_version.clone(),
                latest_manifest: None,
                reason: format!(
                    "Update manifest channel {:?} not eligible for active channel {:?}",
                    manifest.channel, self.channel
                ),
            };
        }

        let curr_sem = match UpdateSemVer::parse(&self.current_version) {
            Ok(v) => v,
            Err(e) => {
                return UpdateCheckResult {
                    update_available: false,
                    current_version: self.current_version.clone(),
                    latest_manifest: None,
                    reason: format!("Failed to parse current version: {}", e),
                };
            }
        };

        let manifest_sem = match UpdateSemVer::parse(&manifest.version) {
            Ok(v) => v,
            Err(e) => {
                return UpdateCheckResult {
                    update_available: false,
                    current_version: self.current_version.clone(),
                    latest_manifest: None,
                    reason: format!("Failed to parse manifest version: {}", e),
                };
            }
        };

        if manifest_sem > curr_sem {
            UpdateCheckResult {
                update_available: true,
                current_version: self.current_version.clone(),
                latest_manifest: Some(manifest.clone()),
                reason: format!(
                    "New version {} available on {} channel (current: {})",
                    manifest.version, manifest.channel, self.current_version
                ),
            }
        } else {
            UpdateCheckResult {
                update_available: false,
                current_version: self.current_version.clone(),
                latest_manifest: None,
                reason: format!(
                    "Up to date. Current {} is >= manifest version {}",
                    self.current_version, manifest.version
                ),
            }
        }
    }

    /// Verifies downloaded package payload against blake3 checksum in manifest.
    pub fn verify_package_checksum(&self, package_bytes: &[u8], expected_checksum: &str) -> bool {
        let hash = blake3::hash(package_bytes).to_hex().to_string();
        hash.eq_ignore_ascii_case(expected_checksum)
    }

    /// Cryptographically verify manifest authenticity against the pinned release public key (`M3.4`).
    pub fn verify_manifest_signature(
        &self,
        manifest: &UpdateManifest,
        trusted_pub_key_hex: &str,
    ) -> bool {
        if manifest.signature_hex.is_empty()
            || !manifest
                .signer_public_key_hex
                .eq_ignore_ascii_case(trusted_pub_key_hex)
        {
            return false;
        }
        wots_verify_hex(
            &manifest.canonical_signing_bytes(),
            &manifest.signature_hex,
            trusted_pub_key_hex,
        )
    }
}

/// Explicit opt-in user consent configuration for automated crash reporting telemetry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashReportConsent {
    /// Strict opt-in requirement (default: false, zero telemetry).
    pub opted_in: bool,
    /// Whether to include application log lines.
    pub include_logs: bool,
    /// Whether to include system OS and hardware specs.
    pub include_system_specs: bool,
    /// Whether to strip private user home directory paths.
    pub anonymize_file_paths: bool,
}

impl Default for CrashReportConsent {
    fn default() -> Self {
        Self {
            opted_in: false,
            include_logs: false,
            include_system_specs: false,
            anonymize_file_paths: true,
        }
    }
}

/// Anonymized, sanitized crash report payload ready for transmission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizedCrashReport {
    pub report_id: String,
    pub timestamp: String,
    pub severity: CrashSeverity,
    pub crate_name: String,
    pub subsystem: String,
    pub sanitized_error: String,
    pub sanitized_stack_trace: Vec<String>,
    pub sanitized_system_metadata: HashMap<String, String>,
    pub sanitized_log_context: Vec<String>,
    pub blake3_digest: String,
}

impl SanitizedCrashReport {
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }
}

/// Opt-in crash reporting integration and dispatcher wired to `CrashDump`.
#[derive(Debug, Clone)]
pub struct CrashReportDispatcher {
    pub consent: CrashReportConsent,
    pub telemetry_endpoint: String,
    pub outbox: Vec<SanitizedCrashReport>,
}

impl CrashReportDispatcher {
    pub fn new(telemetry_endpoint: &str, consent: CrashReportConsent) -> Self {
        Self {
            consent,
            telemetry_endpoint: telemetry_endpoint.to_string(),
            outbox: Vec::new(),
        }
    }

    /// Strips identifiable user paths (e.g. C:\Users\<user>\ or /home/<user>/) into sanitized markers.
    fn sanitize_text(text: &str, anonymize_paths: bool) -> String {
        if !anonymize_paths {
            return text.to_string();
        }

        let mut sanitized = text.to_string();
        // Basic pattern matching for Windows user paths
        if let Some(pos) = sanitized.to_lowercase().find("c:\\users\\") {
            let after = sanitized[pos + 9..].to_string();
            if let Some(slash_idx) = after.find('\\') {
                let username = &after[..slash_idx];
                let search_target = format!("C:\\Users\\{}", username);
                sanitized = sanitized.replace(&search_target, "<USER_HOME>");
                let search_target_lower = format!("c:\\users\\{}", username);
                sanitized = sanitized.replace(&search_target_lower, "<USER_HOME>");
            }
        }
        // Basic pattern matching for Unix /home/user paths
        if let Some(pos) = sanitized.find("/home/") {
            let after = sanitized[pos + 6..].to_string();
            if let Some(slash_idx) = after.find('/') {
                let username = &after[..slash_idx];
                let search_target = format!("/home/{}", username);
                sanitized = sanitized.replace(&search_target, "<USER_HOME>");
            }
        }
        // macOS /Users/user paths
        if let Some(pos) = sanitized.find("/Users/") {
            let after = sanitized[pos + 7..].to_string();
            if let Some(slash_idx) = after.find('/') {
                let username = &after[..slash_idx];
                let search_target = format!("/Users/{}", username);
                sanitized = sanitized.replace(&search_target, "<USER_HOME>");
            }
        }

        sanitized
    }

    /// Prepares and sanitizes a `CrashDump` for telemetry transmission.
    /// Strictly rejects if user has not explicitly consented.
    pub fn prepare_report(&self, dump: &CrashDump) -> Result<SanitizedCrashReport, String> {
        if !self.consent.opted_in {
            return Err(
                "Telemetry consent refused: User has not opted in to automated crash reporting."
                    .to_string(),
            );
        }

        let anonymize = self.consent.anonymize_file_paths;
        let sanitized_error = Self::sanitize_text(&dump.error_message, anonymize);

        let sanitized_stack_trace: Vec<String> = dump
            .stack_trace
            .iter()
            .map(|frame| Self::sanitize_text(frame, anonymize))
            .collect();

        let mut sanitized_metadata = HashMap::new();
        if self.consent.include_system_specs {
            for (k, v) in &dump.system_metadata {
                let key_lower = k.to_lowercase();
                if key_lower.contains("user")
                    || key_lower.contains("path")
                    || key_lower.contains("token")
                {
                    sanitized_metadata.insert(k.clone(), "<REDACTED>".to_string());
                } else {
                    sanitized_metadata.insert(k.clone(), Self::sanitize_text(v, anonymize));
                }
            }
        }

        let sanitized_log_context: Vec<String> = if self.consent.include_logs {
            dump.log_context
                .iter()
                .map(|line| Self::sanitize_text(line, anonymize))
                .collect()
        } else {
            Vec::new()
        };

        let mut hasher = blake3::Hasher::new();
        hasher.update(dump.dump_id.as_bytes());
        hasher.update(sanitized_error.as_bytes());
        let digest = hasher.finalize().to_hex().to_string();

        Ok(SanitizedCrashReport {
            report_id: dump.dump_id.clone(),
            timestamp: dump.timestamp.clone(),
            severity: dump.severity,
            crate_name: dump.crate_name.clone(),
            subsystem: dump.subsystem.clone(),
            sanitized_error,
            sanitized_stack_trace,
            sanitized_system_metadata: sanitized_metadata,
            sanitized_log_context,
            blake3_digest: digest,
        })
    }

    /// Enqueues a crash dump report into the transmission outbox.
    pub fn enqueue_report(&mut self, dump: &CrashDump) -> Result<String, String> {
        let report = self.prepare_report(dump)?;
        let id = report.report_id.clone();
        self.outbox.push(report);
        Ok(id)
    }

    /// Spools outbox reports as JSON files into a local spool directory.
    pub fn spool_to_directory(&self, dir: &Path) -> Result<usize, String> {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        let count = self.outbox.len();
        for report in &self.outbox {
            let filename = format!("spool_report_{}.json", report.report_id);
            let path = dir.join(filename);
            let json = report.to_json()?;
            fs::write(path, json).map_err(|e| e.to_string())?;
        }
        Ok(count)
    }

    /// Clears the outbox after successful spooling or transmission.
    pub fn clear_outbox(&mut self) {
        self.outbox.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_parsing_and_ordering() {
        let v1 = UpdateSemVer::parse("0.1.0").unwrap();
        let v2 = UpdateSemVer::parse("v0.2.0").unwrap();
        let v2_beta = UpdateSemVer::parse("0.2.0-beta.1").unwrap();
        let v2_beta2 = UpdateSemVer::parse("0.2.0-beta.2").unwrap();

        assert!(v2 > v1);
        assert!(v2_beta > v1);
        assert!(v2 > v2_beta);
        assert!(v2_beta2 > v2_beta);
    }

    #[test]
    fn test_auto_update_checker_channel_filtering() {
        let mut checker = AutoUpdateChecker::new("1.0.0", UpdateChannel::Stable);

        let beta_manifest = UpdateManifest {
            version: "1.1.0-beta.1".to_string(),
            channel: UpdateChannel::Beta,
            release_date: "2026-11-10".to_string(),
            release_notes: "Beta experimental features".to_string(),
            download_url: "https://example.com/beta.tar.gz".to_string(),
            blake3_checksum: "deadbeef".to_string(),
            min_os_version: HashMap::new(),
            is_security_critical: false,
            ..Default::default()
        };

        // Stable channel ignores Beta manifest
        let result = checker.evaluate_manifest(&beta_manifest);
        assert!(!result.update_available);

        // Switch to Beta channel
        checker.set_channel(UpdateChannel::Beta);
        let result = checker.evaluate_manifest(&beta_manifest);
        assert!(result.update_available);
    }

    #[test]
    fn test_package_checksum_verification() {
        let checker = AutoUpdateChecker::new("1.0.0", UpdateChannel::Stable);
        let data = b"Summoner binary installer content";
        let expected_hash = blake3::hash(data).to_hex().to_string();

        assert!(checker.verify_package_checksum(data, &expected_hash));
        assert!(!checker.verify_package_checksum(data, "wrongchecksum"));
    }

    #[test]
    fn test_crash_report_opt_in_consent_refusal() {
        let dump = CrashDump::new(
            "dump-001",
            CrashSeverity::Panic,
            "summoner_dsp",
            "filter",
            "assertion failed in filter buffer",
            vec!["summoner_dsp::filter::process".to_string()],
        );

        // Default consent is opted-out
        let dispatcher = CrashReportDispatcher::new(
            "https://telemetry.summoner-audio.org/v1/crash",
            CrashReportConsent::default(),
        );

        let res = dispatcher.prepare_report(&dump);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("User has not opted in"));
    }

    #[test]
    fn test_crash_report_sanitization_and_spooling() {
        let dump = CrashDump::new(
            "dump-002",
            CrashSeverity::Fatal,
            "summoner_engine",
            "audio_device",
            "Error accessing C:\\Users\\alice\\AppData\\Local\\Summoner\\cache.dat",
            vec![
                "at C:\\Users\\alice\\Code\\Summoner\\engine.rs:42".to_string(),
                "at /home/bob/Code/Summoner/src/main.rs:10".to_string(),
            ],
        )
        .with_metadata("user_name", "alice")
        .with_metadata("os", "Windows 11")
        .with_log("User alice opened session /home/bob/projects/demo.smn");

        let consent = CrashReportConsent {
            opted_in: true,
            include_logs: true,
            include_system_specs: true,
            anonymize_file_paths: true,
        };

        let mut dispatcher =
            CrashReportDispatcher::new("https://telemetry.summoner-audio.org/v1/crash", consent);

        let report_id = dispatcher.enqueue_report(&dump).unwrap();
        assert_eq!(report_id, "dump-002");
        assert_eq!(dispatcher.outbox.len(), 1);

        let report = &dispatcher.outbox[0];
        // Check that paths and sensitive usernames are anonymized / redacted
        assert!(report.sanitized_error.contains("<USER_HOME>"));
        assert!(!report.sanitized_error.contains("alice"));
        assert!(report.sanitized_stack_trace[0].contains("<USER_HOME>"));
        assert!(!report.sanitized_stack_trace[0].contains("alice"));
        assert!(report.sanitized_stack_trace[1].contains("<USER_HOME>"));
        assert!(!report.sanitized_stack_trace[1].contains("bob"));
        assert_eq!(
            report.sanitized_system_metadata.get("user_name").unwrap(),
            "<REDACTED>"
        );
        assert_eq!(
            report.sanitized_system_metadata.get("os").unwrap(),
            "Windows 11"
        );
        assert!(report.sanitized_log_context[0].contains("<USER_HOME>"));

        // Spool to temporary dir
        let temp_dir = std::env::temp_dir().join("summoner_test_crash_spool");
        let count = dispatcher.spool_to_directory(&temp_dir).unwrap();
        assert_eq!(count, 1);
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_m3_signed_update_manifest_verification_and_tamper_rejection() {
        let signing_key = [0x5Au8; 32];
        let mut manifest = UpdateManifest {
            version: "1.0.1".to_string(),
            channel: UpdateChannel::Stable,
            release_date: "2026-11-16".to_string(),
            release_notes: "Security & hardware latency hotfix".to_string(),
            download_url: "https://updates.summoner-audio.org/Summoner-1.0.1.msi".to_string(),
            blake3_checksum: "abcd1234".to_string(),
            ..Default::default()
        };
        manifest.sign_with_key(&signing_key);
        let trusted_pub = manifest.signer_public_key_hex.clone();

        let checker = AutoUpdateChecker::new("1.0.0", UpdateChannel::Stable);
        assert!(checker.verify_manifest_signature(&manifest, &trusted_pub));

        // Tampering with download_url must invalidate signature even under the same public key
        let mut tampered = manifest.clone();
        tampered.download_url = "https://evil.example.com/payload.msi".to_string();
        assert!(!checker.verify_manifest_signature(&tampered, &trusted_pub));

        // Untrusted signer key must also be rejected
        assert!(!checker.verify_manifest_signature(&manifest, "0000deadbeef"));
    }
}
