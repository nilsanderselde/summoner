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

//! Lightweight SFZ patch parser and converter to native Summoner TOML session presets.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SfzRegionConfig {
    pub lokey: u8,
    pub hikey: u8,
    pub pitch_keycenter: u8,
    pub lovel: u8,
    pub hivel: u8,
    pub loop_mode: String,
    pub loop_start: usize,
    pub loop_end: usize,
    pub sample_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SfzPresetPatch {
    pub name: String,
    pub instrument_type: String,
    pub attack_sec: f32,
    pub release_sec: f32,
    pub regions: Vec<SfzRegionConfig>,
}

impl SfzPresetPatch {
    pub fn parse_sfz(name: &str, sfz_text: &str) -> Self {
        let mut instrument_name = name.to_string();
        let mut global_release = 0.4f32;
        let mut global_attack = 0.005f32;
        let mut group_loop_mode = "no_loop".to_string();

        let mut regions = Vec::new();
        let mut current_region: Option<SfzRegionConfig> = None;

        for line in sfz_text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("//+") {
                if trimmed.contains("Instrument:") {
                    if let Some(inst) = trimmed.split("Instrument:").nth(1) {
                        instrument_name = inst.trim().to_string();
                    }
                }
                continue;
            }
            if trimmed.starts_with("//") || trimmed.is_empty() {
                continue;
            }

            if trimmed.starts_with("<global>") || trimmed.starts_with("<group>") {
                if let Some(reg) = current_region.take() {
                    regions.push(reg);
                }
            } else if trimmed.starts_with("<region>") {
                if let Some(reg) = current_region.take() {
                    regions.push(reg);
                }
                current_region = Some(SfzRegionConfig {
                    lokey: 0,
                    hikey: 127,
                    pitch_keycenter: 60,
                    lovel: 0,
                    hivel: 127,
                    loop_mode: group_loop_mode.clone(),
                    loop_start: 0,
                    loop_end: 0,
                    sample_path: String::new(),
                });
            }

            // Parse key-value pairs separated by spaces or newlines
            for token in trimmed.split_whitespace() {
                if let Some((k, v)) = token.split_once('=') {
                    match k {
                        "ampeg_release" => {
                            global_release = v.parse().unwrap_or(0.4);
                        }
                        "ampeg_attack" => {
                            global_attack = v.parse().unwrap_or(0.005);
                        }
                        "loop_mode" => {
                            group_loop_mode = v.to_string();
                            if let Some(ref mut reg) = current_region {
                                reg.loop_mode = v.to_string();
                            }
                        }
                        "lokey" => {
                            if let Some(ref mut reg) = current_region {
                                reg.lokey = v.parse().unwrap_or(0);
                            }
                        }
                        "hikey" => {
                            if let Some(ref mut reg) = current_region {
                                reg.hikey = v.parse().unwrap_or(127);
                            }
                        }
                        "pitch_keycenter" => {
                            if let Some(ref mut reg) = current_region {
                                reg.pitch_keycenter = v.parse().unwrap_or(60);
                            }
                        }
                        "lovel" => {
                            if let Some(ref mut reg) = current_region {
                                reg.lovel = v.parse().unwrap_or(0);
                            }
                        }
                        "hivel" => {
                            if let Some(ref mut reg) = current_region {
                                reg.hivel = v.parse().unwrap_or(127);
                            }
                        }
                        "loop_start" => {
                            if let Some(ref mut reg) = current_region {
                                reg.loop_start = v.parse().unwrap_or(0);
                            }
                        }
                        "loop_end" => {
                            if let Some(ref mut reg) = current_region {
                                reg.loop_end = v.parse().unwrap_or(0);
                            }
                        }
                        "sample" => {
                            if let Some(ref mut reg) = current_region {
                                reg.sample_path = v.to_string();
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if let Some(reg) = current_region {
            regions.push(reg);
        }

        Self {
            name: instrument_name,
            instrument_type: "SamplerDevice".to_string(),
            attack_sec: global_attack,
            release_sec: global_release,
            regions,
        }
    }

    pub fn to_toml_preset(&self) -> String {
        toml::to_string_pretty(self)
            .unwrap_or_else(|_| format!("# Failed to format preset {}", self.name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sfz_parsing() {
        let sample_sfz = r#"
<global>
 //+ Instrument: Test Piano
 ampeg_release=1.2

<group>
 loop_mode=loop_continuous

<region>
 lokey=21 hikey=36
 pitch_keycenter=28
 loop_start=1000
 loop_end=5000
 sample=samples/Piano/C1.flac
"#;

        let patch = SfzPresetPatch::parse_sfz("001_Piano", sample_sfz);
        assert_eq!(patch.name, "Test Piano");
        assert_eq!(patch.release_sec, 1.2);
        assert_eq!(patch.regions.len(), 1);
        assert_eq!(patch.regions[0].pitch_keycenter, 28);
        assert_eq!(patch.regions[0].sample_path, "samples/Piano/C1.flac");
    }
}
