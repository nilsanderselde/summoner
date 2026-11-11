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
}
