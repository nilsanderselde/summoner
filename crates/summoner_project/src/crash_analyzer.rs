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

//! Step 1247: Automated crash reporting dump analyzer (local disk offline log dump viewer).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Crash severity categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CrashSeverity {
    Fatal,
    Error,
    Panic,
    Warning,
}

impl std::fmt::Display for CrashSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CrashSeverity::Fatal => write!(f, "FATAL"),
            CrashSeverity::Error => write!(f, "ERROR"),
            CrashSeverity::Panic => write!(f, "PANIC"),
            CrashSeverity::Warning => write!(f, "WARNING"),
        }
    }
}

/// Structured offline crash dump representation stored on local disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashDump {
    pub dump_id: String,
    pub timestamp: String,
    pub severity: CrashSeverity,
    pub crate_name: String,
    pub subsystem: String,
    pub error_message: String,
    pub stack_trace: Vec<String>,
    pub system_metadata: HashMap<String, String>,
    pub log_context: Vec<String>,
}

impl CrashDump {
    pub fn new(
        dump_id: &str,
        severity: CrashSeverity,
        crate_name: &str,
        subsystem: &str,
        error_message: &str,
        stack_trace: Vec<String>,
    ) -> Self {
        Self {
            dump_id: dump_id.to_string(),
            timestamp: "2026-08-05T13:00:00Z".to_string(),
            severity,
            crate_name: crate_name.to_string(),
            subsystem: subsystem.to_string(),
            error_message: error_message.to_string(),
            stack_trace,
            system_metadata: HashMap::new(),
            log_context: Vec::new(),
        }
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.system_metadata.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_log(mut self, log_line: &str) -> Self {
        self.log_context.push(log_line.to_string());
        self
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| e.to_string())
    }

    pub fn save_to_file(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = self.to_json()?;
        fs::write(path, json).map_err(|e| e.to_string())
    }

    pub fn load_from_file(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        Self::from_json(&content)
    }
}

/// Offline crash analysis outcome for a single crash dump file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashAnalysisResult {
    pub dump_id: String,
    pub severity: CrashSeverity,
    pub crate_name: String,
    pub subsystem: String,
    pub probable_root_cause: String,
    pub top_faulting_frame: Option<String>,
    pub suggested_remedies: Vec<String>,
    pub is_offline_safe: bool,
    pub formatted_report: String,
}

/// Aggregated offline crash summary report across multiple local dump files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashDumpSummaryReport {
    pub total_dumps_analyzed: usize,
    pub counts_by_severity: HashMap<CrashSeverity, usize>,
    pub subsystem_frequencies: HashMap<String, usize>,
    pub top_root_causes: Vec<(String, usize)>,
    pub recommendations: Vec<String>,
    pub formatted_summary: String,
}

/// Offline local disk crash reporting dump analyzer engine.
pub struct CrashDumpAnalyzer;

impl CrashDumpAnalyzer {
    /// Analyze a single `CrashDump` completely offline.
    pub fn analyze_dump(dump: &CrashDump) -> CrashAnalysisResult {
        let err_lower = dump.error_message.to_lowercase();
        let top_frame = dump.stack_trace.first().cloned();

        let (probable_root_cause, remedies) = if err_lower.contains("access violation")
            || err_lower.contains("null pointer")
            || err_lower.contains("0xc0000005")
        {
            (
                "Memory Access Violation in Native Code / Plugin Host".to_string(),
                vec![
                    "Enable sub-process crash sandbox isolation for third-party plugins.".to_string(),
                    "Add offending plugin binary to host blacklist.".to_string(),
                ],
            )
        } else if err_lower.contains("underflow")
            || err_lower.contains("buffer starvation")
            || err_lower.contains("xrun")
        {
            (
                "Real-time Audio Buffer Processing Underflow (XRUN)".to_string(),
                vec![
                    "Increase native audio driver block buffer size (e.g. 256 or 512 frames).".to_string(),
                    "Enable high-priority real-time audio thread scheduling.".to_string(),
                ],
            )
        } else if err_lower.contains("out of memory")
            || err_lower.contains("allocation failed")
            || err_lower.contains("oom")
        {
            (
                "Memory Exhaustion / RAM Allocation Failure".to_string(),
                vec![
                    "Clear scratch audio cache or trim cached waveform audio buffers.".to_string(),
                    "Reduce max active sample polyphony limit.".to_string(),
                ],
            )
        } else if dump.severity == CrashSeverity::Panic
            || err_lower.contains("panicked at")
            || err_lower.contains("unwrap()")
        {
            (
                "Rust Runtime Panic / Unwrapped Option/Result".to_string(),
                vec![
                    "Inspect stack trace frame for boundary condition failure.".to_string(),
                    "Enforce strict non-panicking error handling.".to_string(),
                ],
            )
        } else {
            (
                format!("Unclassified Subsystem Error in {}", dump.subsystem),
                vec![
                    "Inspect detailed offline log dump context leading to crash.".to_string(),
                    "Run offline project validator on current project file.".to_string(),
                ],
            )
        };

        let mut report = String::new();
        report.push_str("=== OFFLINE CRASH REPORT DUMP ANALYZER ===\n");
        report.push_str(&format!("Dump ID:       {}\n", dump.dump_id));
        report.push_str(&format!("Timestamp:     {}\n", dump.timestamp));
        report.push_str(&format!("Severity:      {}\n", dump.severity));
        report.push_str(&format!("Crate / Mod:   {} / {}\n", dump.crate_name, dump.subsystem));
        report.push_str(&format!("Error Message: {}\n", dump.error_message));
        report.push_str(&format!("Root Cause:    {}\n", probable_root_cause));
        if let Some(ref frame) = top_frame {
            report.push_str(&format!("Top Frame:     {}\n", frame));
        }
        report.push_str("Remedies:\n");
        for (idx, remedy) in remedies.iter().enumerate() {
            report.push_str(&format!("  {}. {}\n", idx + 1, remedy));
        }
        report.push_str("Offline Mode:  STRICT (100% Local Disk Analysis, Zero Telemetry Network Calls)\n");

        CrashAnalysisResult {
            dump_id: dump.dump_id.clone(),
            severity: dump.severity,
            crate_name: dump.crate_name.clone(),
            subsystem: dump.subsystem.clone(),
            probable_root_cause,
            top_faulting_frame: top_frame,
            suggested_remedies: remedies,
            is_offline_safe: true,
            formatted_report: report,
        }
    }

    /// Analyze a single crash dump JSON file on local disk.
    pub fn analyze_dump_file(path: &Path) -> Result<CrashAnalysisResult, String> {
        let dump = CrashDump::load_from_file(path)?;
        Ok(Self::analyze_dump(&dump))
    }

    /// Scan a directory of crash dump JSON files and compile a summary report.
    pub fn analyze_dumps_directory(dir_path: &Path) -> Result<CrashDumpSummaryReport, String> {
        if !dir_path.exists() {
            return Err(format!("Directory path {:?} does not exist", dir_path));
        }

        let entries = fs::read_dir(dir_path).map_err(|e| e.to_string())?;
        let mut dumps = Vec::new();

        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.is_file()
                && (path.extension().and_then(|s| s.to_str()) == Some("json")
                    || path.to_string_lossy().contains("dump"))
            {
                if let Ok(dump) = CrashDump::load_from_file(&path) {
                    dumps.push(dump);
                }
            }
        }

        if dumps.is_empty() {
            return Ok(CrashDumpSummaryReport {
                total_dumps_analyzed: 0,
                counts_by_severity: HashMap::new(),
                subsystem_frequencies: HashMap::new(),
                top_root_causes: Vec::new(),
                recommendations: vec!["No crash dump log files found in directory.".to_string()],
                formatted_summary: "No crash dump files present for analysis.".to_string(),
            });
        }

        let total_dumps_analyzed = dumps.len();
        let mut counts_by_severity: HashMap<CrashSeverity, usize> = HashMap::new();
        let mut subsystem_frequencies: HashMap<String, usize> = HashMap::new();
        let mut cause_counts: HashMap<String, usize> = HashMap::new();

        for dump in &dumps {
            *counts_by_severity.entry(dump.severity).or_insert(0) += 1;
            *subsystem_frequencies.entry(dump.subsystem.clone()).or_insert(0) += 1;

            let result = Self::analyze_dump(dump);
            *cause_counts.entry(result.probable_root_cause).or_insert(0) += 1;
        }

        let mut top_root_causes: Vec<(String, usize)> = cause_counts.into_iter().collect();
        top_root_causes.sort_by(|a, b| b.1.cmp(&a.1));

        let mut recommendations = Vec::new();
        if let Some((most_frequent_cause, count)) = top_root_causes.first() {
            recommendations.push(format!(
                "Primary crash vector ({} occurrences): {}",
                count, most_frequent_cause
            ));
        }
        recommendations.push("Ensure all third-party plugins run with process isolation.".to_string());
        recommendations.push("Keep offline log dump retention set to maximum 30 days.".to_string());

        let mut formatted_summary = String::new();
        formatted_summary.push_str("=== LOCAL DISK CRASH REPORT DUMP SUMMARY ===\n");
        formatted_summary.push_str(&format!("Total Dumps Analyzed: {}\n", total_dumps_analyzed));
        formatted_summary.push_str("Severity Breakdown:\n");
        for (sev, count) in &counts_by_severity {
            formatted_summary.push_str(&format!("  - {}: {}\n", sev, count));
        }
        formatted_summary.push_str("Top Root Causes:\n");
        for (cause, count) in &top_root_causes {
            formatted_summary.push_str(&format!("  - [{}] {}\n", count, cause));
        }

        Ok(CrashDumpSummaryReport {
            total_dumps_analyzed,
            counts_by_severity,
            subsystem_frequencies,
            top_root_causes,
            recommendations,
            formatted_summary,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_1247_crash_dump_analyzer_offline() {
        let dump = CrashDump::new(
            "dump-001",
            CrashSeverity::Fatal,
            "summoner_dsp",
            "vst3_host",
            "Access violation reading address 0x00000000",
            vec!["vst3_host::process_audio (line 42)".to_string()],
        )
        .with_metadata("driver", "wasapi")
        .with_log("Audio engine initialized");

        let result = CrashDumpAnalyzer::analyze_dump(&dump);
        assert_eq!(result.dump_id, "dump-001");
        assert!(result.is_offline_safe);
        assert!(result.probable_root_cause.contains("Memory Access Violation"));
        assert!(result.suggested_remedies.iter().any(|r| r.contains("sandbox isolation")));

        let temp_dir = std::env::temp_dir().join("summoner_crash_dump_test");
        let dump_path = temp_dir.join("dump-001.json");
        dump.save_to_file(&dump_path).unwrap();

        let loaded_result = CrashDumpAnalyzer::analyze_dump_file(&dump_path).unwrap();
        assert_eq!(loaded_result.dump_id, "dump-001");

        let summary = CrashDumpAnalyzer::analyze_dumps_directory(&temp_dir).unwrap();
        assert_eq!(summary.total_dumps_analyzed, 1);
        assert!(summary.formatted_summary.contains("LOCAL DISK CRASH REPORT DUMP SUMMARY"));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
