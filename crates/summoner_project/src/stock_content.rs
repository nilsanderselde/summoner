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

//! Factory Stock Content library, curated presets, demo projects, and license audit registry.

use crate::content_browser::{ContentBrowser, ContentItem, ContentItemKind};
use crate::schema::{ProjectConfig, ProjectMetadata, TrackConfig, TransportConfig};
use serde::{Deserialize, Serialize};

/// License clearance metadata for stock assets.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StockLicenseEntry {
    pub asset_name: String,
    pub category: String,
    pub license_spdx: String,
    pub author_or_origin: String,
    pub source_url: Option<String>,
    pub commercial_use_cleared: bool,
    pub royalty_free: bool,
    pub notes: String,
}

/// Factory stock content registry.
pub struct StockContentRegistry;

impl StockContentRegistry {
    /// Return the full license audit registry for all factory assets.
    pub fn license_audit() -> Vec<StockLicenseEntry> {
        vec![
            StockLicenseEntry {
                asset_name: "TR-808 Classic Analog Drum Station".to_string(),
                category: "Drum Kits".to_string(),
                license_spdx: "CC0-1.0".to_string(),
                author_or_origin: "Summoner Procedural Synthesis & FreePats".to_string(),
                source_url: Some("https://freepats.zenvoid.org/".to_string()),
                commercial_use_cleared: true,
                royalty_free: true,
                notes: "100% royalty-free for commercial music releases and live performances."
                    .to_string(),
            },
            StockLicenseEntry {
                asset_name: "TR-909 Vintage Punch Drum Kit".to_string(),
                category: "Drum Kits".to_string(),
                license_spdx: "CC0-1.0".to_string(),
                author_or_origin: "Summoner Procedural Synthesis".to_string(),
                source_url: None,
                commercial_use_cleared: true,
                royalty_free: true,
                notes: "Mathematically synthesized analog drum circuits.".to_string(),
            },
            StockLicenseEntry {
                asset_name: "Studio Maple Acoustic Drum Kit".to_string(),
                category: "Drum Kits".to_string(),
                license_spdx: "CC0-1.0".to_string(),
                author_or_origin: "FreePats GM Project".to_string(),
                source_url: Some("https://freepats.zenvoid.org/".to_string()),
                commercial_use_cleared: true,
                royalty_free: true,
                notes: "Multi-velocity layer acoustic studio drum samples.".to_string(),
            },
            StockLicenseEntry {
                asset_name: "Concert Grand Model D Multi-Sample".to_string(),
                category: "Instruments".to_string(),
                license_spdx: "CC0-1.0".to_string(),
                author_or_origin: "FreePats Grand Piano & Physical Hammer Model".to_string(),
                source_url: Some("https://freepats.zenvoid.org/".to_string()),
                commercial_use_cleared: true,
                royalty_free: true,
                notes: "Full 88-key multisample acoustic grand piano.".to_string(),
            },
            StockLicenseEntry {
                asset_name: "Cyberpunk Sub-Bass & Heavy Wavefolders".to_string(),
                category: "Instruments".to_string(),
                license_spdx: "MIT".to_string(),
                author_or_origin: "Summoner Core Team".to_string(),
                source_url: None,
                commercial_use_cleared: true,
                royalty_free: true,
                notes: "Subtractive & wavefolding analog emulation presets.".to_string(),
            },
            StockLicenseEntry {
                asset_name: "Synthwave Sunset (120 BPM)".to_string(),
                category: "Audio Loops".to_string(),
                license_spdx: "CC0-1.0".to_string(),
                author_or_origin: "Summoner In-House Audio Lab".to_string(),
                source_url: None,
                commercial_use_cleared: true,
                royalty_free: true,
                notes: "Synth bass and drum stem loops in 44.1kHz 24-bit PCM.".to_string(),
            },
        ]
    }

    /// Populate a `ContentBrowser` with all factory stock content definitions.
    pub fn populate_factory_browser(browser: &mut ContentBrowser) {
        let items = vec![
            // Drum Kits
            ContentItem::new("factory/kits/tr808_classic.toml", ContentItemKind::Preset)
                .with_tags(&["drums", "kit", "808", "electronic", "trap", "hiphop"]),
            ContentItem::new("factory/kits/tr909_vintage.toml", ContentItemKind::Preset)
                .with_tags(&["drums", "kit", "909", "techno", "house", "punchy"]),
            ContentItem::new(
                "factory/kits/studio_maple_acoustic.sfz",
                ContentItemKind::Instrument,
            )
            .with_tags(&["drums", "acoustic", "studio", "rock", "jazz", "multisample"]),
            ContentItem::new(
                "factory/kits/cyberpunk_industrial.toml",
                ContentItemKind::Preset,
            )
            .with_tags(&["drums", "industrial", "cyberpunk", "distorted", "fx"]),
            // Multi-Sample Instruments
            ContentItem::new(
                "factory/instruments/concert_grand_piano.sfz",
                ContentItemKind::Instrument,
            )
            .with_tags(&["piano", "acoustic", "keys", "grand", "multisample"]),
            ContentItem::new(
                "factory/instruments/vintage_rhodes_mk1.sfz",
                ContentItemKind::Instrument,
            )
            .with_tags(&["electric_piano", "rhodes", "vintage", "soul", "keys"]),
            ContentItem::new(
                "factory/instruments/analog_sub_bass.toml",
                ContentItemKind::Preset,
            )
            .with_tags(&["bass", "sub", "analog", "synth", "cyberpunk"]),
            ContentItem::new(
                "factory/instruments/lush_80s_strings.toml",
                ContentItemKind::Preset,
            )
            .with_tags(&["pad", "strings", "synthwave", "ambient", "analog"]),
            // Stock Audio Loops
            ContentItem::new(
                "factory/loops/synthwave_bass_120bpm.wav",
                ContentItemKind::AudioLoop,
            )
            .with_bpm(120.0)
            .with_tags(&["loop", "bass", "synthwave", "analog", "120bpm"])
            .with_audio_metadata(4.0, 44100, 2),
            ContentItem::new(
                "factory/loops/funk_drum_groove_110bpm.wav",
                ContentItemKind::AudioLoop,
            )
            .with_bpm(110.0)
            .with_tags(&["loop", "drums", "funk", "acoustic", "groove", "110bpm"])
            .with_audio_metadata(4.36, 44100, 2),
            ContentItem::new(
                "factory/loops/lofi_guitar_chords_85bpm.wav",
                ContentItemKind::AudioLoop,
            )
            .with_bpm(85.0)
            .with_tags(&["loop", "guitar", "chords", "lofi", "chill", "85bpm"])
            .with_audio_metadata(5.64, 44100, 2),
            // Demo Projects
            ContentItem::new(
                "factory/demos/synthwave_sunset.summon",
                ContentItemKind::ProjectDemo,
            )
            .with_bpm(120.0)
            .with_tags(&["demo", "project", "synthwave", "full_mix"]),
            ContentItem::new(
                "factory/demos/ambient_dawn.summon",
                ContentItemKind::ProjectDemo,
            )
            .with_bpm(75.0)
            .with_tags(&["demo", "project", "ambient", "meditation"]),
            ContentItem::new(
                "factory/demos/cyberpunk_industrial.summon",
                ContentItemKind::ProjectDemo,
            )
            .with_bpm(138.0)
            .with_tags(&["demo", "project", "cyberpunk", "industrial", "glitch"]),
        ];

        for item in items {
            browser.add_item(item);
        }
    }

    /// Generate the complete "Synthwave Sunset" factory demo project.
    pub fn demo_synthwave_sunset() -> ProjectConfig {
        let mut proj = ProjectConfig {
            name: "Synthwave Sunset".to_string(),
            transport: TransportConfig {
                bpm: 120.0,
                time_signature: "4/4".to_string(),
                ..Default::default()
            },
            meta: Some(ProjectMetadata {
                genre: Some("Synthwave".to_string()),
                bpm: Some(120.0),
                tags: vec![
                    "synthwave".to_string(),
                    "retro".to_string(),
                    "demo".to_string(),
                ],
                notes: Some("Factory demo project for Summoner DAW.".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        // 1. Drums Track
        let drums_track = TrackConfig {
            id: 1,
            name: "Drums (TR-909)".to_string(),
            gain: 0.85,
            pan: 0.0,
            ..Default::default()
        };
        proj.tracks.push(drums_track);

        // 2. Sub-Bass Track
        let bass_track = TrackConfig {
            id: 2,
            name: "Cyberpunk Sub-Bass".to_string(),
            gain: 0.90,
            pan: 0.0,
            ..Default::default()
        };
        proj.tracks.push(bass_track);

        // 3. Synth Lead Track
        let lead_track = TrackConfig {
            id: 3,
            name: "Analog Sunset Lead".to_string(),
            gain: 0.75,
            pan: 0.15,
            ..Default::default()
        };
        proj.tracks.push(lead_track);

        proj
    }

    /// Generate the complete "Ambient Dawn" factory demo project.
    pub fn demo_ambient_dawn() -> ProjectConfig {
        let mut proj = ProjectConfig {
            name: "Ambient Dawn".to_string(),
            transport: TransportConfig {
                bpm: 75.0,
                time_signature: "4/4".to_string(),
                ..Default::default()
            },
            meta: Some(ProjectMetadata {
                genre: Some("Ambient".to_string()),
                bpm: Some(75.0),
                tags: vec![
                    "ambient".to_string(),
                    "meditation".to_string(),
                    "drone".to_string(),
                ],
                notes: Some("Warm atmospheric textures and physical model drone.".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let drone_track = TrackConfig {
            id: 1,
            name: "Atmospheric Pad Drone".to_string(),
            gain: 0.70,
            ..Default::default()
        };
        proj.tracks.push(drone_track);

        let piano_track = TrackConfig {
            id: 2,
            name: "Warm Grand Piano".to_string(),
            gain: 0.80,
            pan: -0.1,
            ..Default::default()
        };
        proj.tracks.push(piano_track);

        proj
    }

    /// Synthesize a deterministic, royalty-free stereo factory audio sample normalized to -1.0 dBFS (`CON-1`, `CON-2`).
    pub fn synthesize_factory_sample(preset_id: &str, sample_rate: u32) -> Vec<f32> {
        let sr = sample_rate.max(8000) as f32;
        let duration_secs = match preset_id {
            "hihat_closed" => 0.15,
            "snare_909" | "idm_metallic_perc" => 0.35,
            "kick_808" => 0.60,
            _ => 1.0,
        };
        let frames = (sr * duration_secs) as usize;
        let mut out = vec![0.0f32; frames * 2];
        let mut noise_state: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut phase = 0.0f32;
        let mut phase2 = 0.0f32;

        for f in 0..frames {
            let t = f as f32 / sr;
            noise_state = noise_state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let white = ((noise_state >> 40) as f32 / 16_777_215.0) * 2.0 - 1.0;

            let (l, r) = match preset_id {
                "kick_808" => {
                    let freq = 48.0 + 130.0 * (-t * 28.0).exp();
                    phase += 2.0 * std::f32::consts::PI * freq / sr;
                    let env = (-t * 5.2).exp();
                    let s = (phase.sin() * 1.4).tanh() * env;
                    (s, s)
                }
                "snare_909" => {
                    let body = (2.0 * std::f32::consts::PI * 185.0 * t).sin() * (-t * 18.0).exp();
                    let snap = white * (-t * 14.0).exp() * 0.7;
                    let s = (body + snap).clamp(-1.0, 1.0);
                    (s, s)
                }
                "hihat_closed" => {
                    let metal = (2.0 * std::f32::consts::PI * 3400.0 * t).sin()
                        * (2.0 * std::f32::consts::PI * 4980.0 * t).sin();
                    let s = (metal * 0.5 + white * 0.5) * (-t * 36.0).exp();
                    (s, s)
                }
                "reese_bass_c2" => {
                    phase += 2.0 * std::f32::consts::PI * 65.41 / sr;
                    phase2 += 2.0 * std::f32::consts::PI * 66.15 / sr;
                    let saw_l = 1.0 - 2.0 * ((phase / (2.0 * std::f32::consts::PI)) % 1.0);
                    let saw_r = 1.0 - 2.0 * ((phase2 / (2.0 * std::f32::consts::PI)) % 1.0);
                    let s_l = ((saw_l + saw_r * 0.6) * 0.8).tanh();
                    let s_r = ((saw_r + saw_l * 0.6) * 0.8).tanh();
                    (s_l, s_r)
                }
                "acid_squelch_loop" => {
                    let step_hz = if ((t * 8.0) as usize).is_multiple_of(2) {
                        55.0
                    } else {
                        110.0
                    };
                    phase += 2.0 * std::f32::consts::PI * step_hz / sr;
                    let mod_sig = (2.0
                        * std::f32::consts::PI
                        * (320.0 + 900.0 * (-((t * 8.0) % 1.0) * 6.0).exp())
                        * t)
                        .sin();
                    let s = (phase.sin() + 0.5 * mod_sig).tanh() * 0.85;
                    (s, s)
                }
                "idm_metallic_perc" => {
                    let mod_osc = (2.0 * std::f32::consts::PI * 713.0 * t).sin() * 3.5;
                    let carrier = (2.0 * std::f32::consts::PI * 440.0 * t + mod_osc).sin();
                    let env = (-t * 16.0).exp();
                    (carrier * env, (carrier * 0.85 + white * 0.1) * env)
                }
                _ => {
                    let c = (2.0 * std::f32::consts::PI * 130.81 * t).sin();
                    let g = (2.0 * std::f32::consts::PI * 196.00 * t).sin();
                    let b = (2.0 * std::f32::consts::PI * 246.94 * t).sin();
                    let swell = (t / duration_secs).min(1.0);
                    ((c + 0.6 * g) * swell * 0.5, (c + 0.6 * b) * swell * 0.5)
                }
            };

            out[f * 2] = l;
            out[f * 2 + 1] = r;
        }

        // Peak-normalize to -1.0 dBFS target ceiling
        let peak = out.iter().fold(0.0f32, |m, &s| m.max(s.abs()));
        if peak > 1e-6 {
            let target_peak = 10.0f32.powf(-1.0 / 20.0);
            let scale = target_peak / peak;
            for s in &mut out {
                *s *= scale;
            }
        }
        out
    }

    /// Generate the "IDM Crown Jewel" standalone electronic showcase project (`CON-4`).
    pub fn demo_idm_crown_jewel() -> ProjectConfig {
        use crate::schema::{
            ChordEventConfig, KeyPointConfig, NodeConfig, SequenceConfig, SidechainRouteConfig,
            TrackerStepConfig,
        };
        use std::collections::HashMap;

        let mut proj = ProjectConfig {
            name: "IDM Crown Jewel".to_string(),
            transport: TransportConfig {
                bpm: 138.0,
                time_signature: "4/4".to_string(),
                key_points: vec![KeyPointConfig {
                    beat: 0.0,
                    root_note: 5, // F Minor
                    scale_type: "Minor".to_string(),
                }],
                chord_track: vec![
                    ChordEventConfig {
                        beat: 0.0,
                        duration_beats: 4.0,
                        root_note: 5,
                        quality: "min".to_string(),
                        bass_note: None,
                    },
                    ChordEventConfig {
                        beat: 4.0,
                        duration_beats: 4.0,
                        root_note: 8,
                        quality: "Maj".to_string(),
                        bass_note: None,
                    },
                ],
                ..Default::default()
            },
            meta: Some(ProjectMetadata {
                genre: Some("IDM / Breakbeat".to_string()),
                bpm: Some(138.0),
                key: Some("Fm".to_string()),
                tags: vec![
                    "idm".to_string(),
                    "edm".to_string(),
                    "microtonal".to_string(),
                    "sidechain".to_string(),
                ],
                notes: Some(
                    "Showcase standalone project with core scale lock, chord track, and sidechain bus."
                        .to_string(),
                ),
            }),
            ..Default::default()
        };

        let drum_id = 1;
        proj.tracks.push(TrackConfig {
            id: drum_id,
            name: "Kick & Glitch Kit".to_string(),
            gain: 0.9,
            nodes: vec![NodeConfig {
                kind: "DrumMachine".to_string(),
                params: HashMap::new(),
                plugin_state: None,
            }],
            sequence: Some(SequenceConfig {
                start_beat: 0.0,
                step_division: 0.5,
                steps: vec![
                    TrackerStepConfig {
                        note: 36.0,
                        velocity: 1.0,
                        active: true,
                        ..Default::default()
                    },
                    TrackerStepConfig {
                        note: 38.0,
                        velocity: 0.85,
                        active: true,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }),
            ..Default::default()
        });

        let bass_id = 2;
        proj.tracks.push(TrackConfig {
            id: bass_id,
            name: "Neuro Reese Bass".to_string(),
            gain: 0.85,
            scale_lock: true,
            follow_chord_track: true,
            nodes: vec![NodeConfig {
                kind: "DailySubtractiveSynth".to_string(),
                params: HashMap::new(),
                plugin_state: None,
            }],
            sequence: Some(SequenceConfig {
                start_beat: 0.0,
                step_division: 1.0,
                steps: vec![TrackerStepConfig {
                    note: 41.0,
                    velocity: 0.95,
                    active: true,
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        });

        let lead_id = 3;
        proj.tracks.push(TrackConfig {
            id: lead_id,
            name: "FM Shimmer Arp".to_string(),
            gain: 0.72,
            pan: 0.2,
            scale_lock: true,
            follow_chord_track: true,
            nodes: vec![NodeConfig {
                kind: "FM2Op".to_string(),
                params: HashMap::new(),
                plugin_state: None,
            }],
            ..Default::default()
        });

        let bus_id = proj.create_bus_track("Synth Bus");
        proj.tracks
            .iter_mut()
            .filter(|t| t.id == bass_id || t.id == lead_id)
            .for_each(|t| t.bus_target = Some("Synth Bus".to_string()));
        let _ = bus_id;

        proj.upsert_sidechain_route(SidechainRouteConfig {
            source_track_id: drum_id,
            dest_track_id: bass_id,
            threshold_db: -20.0,
            ratio: 6.0,
            ..Default::default()
        });

        proj
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stock_content_registry_and_licensing() {
        let audit = StockContentRegistry::license_audit();
        assert!(!audit.is_empty());
        for entry in &audit {
            assert!(entry.commercial_use_cleared);
            assert!(entry.royalty_free);
            assert!(!entry.license_spdx.is_empty());
        }

        let mut browser = ContentBrowser::new();
        StockContentRegistry::populate_factory_browser(&mut browser);
        assert!(browser.items.len() >= 10);

        // Check finding 808
        let q = crate::content_browser::ContentSearchQuery::new("808");
        let results = browser.search(&q);
        assert!(!results.is_empty());

        // Check demo project generation
        let demo = StockContentRegistry::demo_synthwave_sunset();
        assert_eq!(demo.transport.bpm, 120.0);
        assert_eq!(demo.tracks.len(), 3);
        assert_eq!(demo.meta.unwrap().genre, Some("Synthwave".to_string()));
    }

    #[test]
    fn test_procedural_factory_samples_and_idm_demo() {
        let target_ceiling = 10.0f32.powf(-1.0 / 20.0);
        for id in [
            "kick_808",
            "snare_909",
            "hihat_closed",
            "reese_bass_c2",
            "acid_squelch_loop",
            "idm_metallic_perc",
            "ambient_shimmer_pad",
        ] {
            let pcm = StockContentRegistry::synthesize_factory_sample(id, 44100);
            assert!(!pcm.is_empty(), "empty sample for {}", id);
            assert!(pcm.iter().all(|s| s.is_finite()));
            let peak = pcm.iter().fold(0.0f32, |m, &s| m.max(s.abs()));
            assert!(
                (peak - target_ceiling).abs() < 1e-3,
                "expected -1 dBFS ceiling on {}, got peak {}",
                id,
                peak
            );
        }

        let idm = StockContentRegistry::demo_idm_crown_jewel();
        assert_eq!(idm.transport.bpm, 138.0);
        assert_eq!(idm.tracks.len(), 4);
        assert_eq!(idm.sidechain_routes.len(), 1);
        assert!(idm.tracks[1].scale_lock);
        assert!(idm.tracks[1].follow_chord_track);
    }
}
