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

//! Step 1265: First-run onboarding wizard milestone engine and bundled
//! production-ready multi-track demonstration projects (Electronic, Acoustic, Immersive 5.1).

use crate::audio_clip::AudioClipConfig;
use crate::schema::{
    NodeConfig, ProjectConfig, SequenceConfig, TrackConfig, TrackerStepConfig, TransportConfig,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Milestones tracked during first-run user onboarding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OnboardingMilestone {
    WelcomeSeen,
    AudioDeviceConfigured,
    FirstTrackCreated,
    FirstClipRecordedOrPlaced,
    FirstEffectInserted,
    FirstProjectExported,
}

/// First-run onboarding state and progress tracker.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OnboardingManager {
    pub completed_milestones: Vec<OnboardingMilestone>,
    pub active_step_index: usize,
    pub is_dismissed: bool,
    pub preferred_genre_template: Option<String>,
}

impl OnboardingManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn complete_milestone(&mut self, milestone: OnboardingMilestone) {
        if !self.completed_milestones.contains(&milestone) {
            self.completed_milestones.push(milestone);
        }
    }

    pub fn is_milestone_completed(&self, milestone: OnboardingMilestone) -> bool {
        self.completed_milestones.contains(&milestone)
    }

    pub fn progress_percentage(&self) -> f32 {
        let total = 6.0;
        let done = self.completed_milestones.len() as f32;
        (done / total * 100.0).clamp(0.0, 100.0)
    }

    pub fn dismiss(&mut self) {
        self.is_dismissed = true;
    }
}

/// Factory demo project descriptors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoProjectInfo {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub genre: String,
    pub bpm: f64,
    pub time_signature: String,
    pub track_count: usize,
    pub description: String,
}

/// Collection of bundled factory demonstration projects.
pub struct DemoProjects;

impl DemoProjects {
    /// Returns catalog descriptors of all bundled demonstration projects.
    pub fn catalog() -> Vec<DemoProjectInfo> {
        vec![
            DemoProjectInfo {
                id: "demo_cosmic_drift".to_string(),
                title: "Cosmic Drift".to_string(),
                artist: "Summoner Audio Lab".to_string(),
                genre: "Electronic / Synthwave".to_string(),
                bpm: 124.0,
                time_signature: "4/4".to_string(),
                track_count: 5,
                description: "Full electronic multi-track with analog synth leads, 808 drums, arpeggiated bass, and sidechain compression.".to_string(),
            },
            DemoProjectInfo {
                id: "demo_acoustic_horizon".to_string(),
                title: "Acoustic Horizon".to_string(),
                artist: "Summoner Folk Session".to_string(),
                genre: "Acoustic / Folk".to_string(),
                bpm: 96.0,
                time_signature: "4/4".to_string(),
                track_count: 4,
                description: "Warm organic acoustic session featuring fingerpicked guitar, lead vocal take, subtle room reverb, and bus EQ.".to_string(),
            },
            DemoProjectInfo {
                id: "demo_deep_space_ambience".to_string(),
                title: "Deep Space Ambience (5.1 Surround)".to_string(),
                artist: "Summoner Spatial Lab".to_string(),
                genre: "Cinematic / Ambient".to_string(),
                bpm: 72.0,
                time_signature: "4/4".to_string(),
                track_count: 6,
                description: "6-channel immersive soundscape demonstrating multichannel spatial panning, LFE sub-bass rumble, and surround downmixing.".to_string(),
            },
        ]
    }

    /// Creates "Cosmic Drift" Electronic / Synthwave Demo Project.
    pub fn create_synthwave_demo() -> ProjectConfig {
        let mut kick_steps = Vec::new();
        for _ in 0..16 {
            kick_steps.push(TrackerStepConfig {
                note: 36.0,
                velocity: 0.9,
                gate: 0.5,
                ..Default::default()
            });
        }

        let mut bass_steps = Vec::new();
        for i in 0..16 {
            bass_steps.push(TrackerStepConfig {
                note: if (i / 4) % 2 == 0 { 33.0 } else { 36.0 },
                velocity: 0.85,
                gate: 0.45,
                ..Default::default()
            });
        }

        let lead_steps = vec![
            TrackerStepConfig { note: 69.0, velocity: 0.8, gate: 0.8, ..Default::default() },
            TrackerStepConfig { note: 72.0, velocity: 0.85, gate: 0.8, ..Default::default() },
            TrackerStepConfig { note: 76.0, velocity: 0.9, gate: 0.9, ..Default::default() },
            TrackerStepConfig { note: 74.0, velocity: 0.8, gate: 0.8, ..Default::default() },
        ];

        let tracks = vec![
            TrackConfig {
                id: 1,
                name: "Kick & Drums".to_string(),
                channels: 2,
                gain: 0.9,
                pan: 0.0,
                clips: vec![SequenceConfig {
                    name: "808 Four on Floor".to_string(),
                    start_beat: 0.0,
                    step_division: 1.0,
                    steps: kick_steps,
                    ..Default::default()
                }],
                nodes: vec![NodeConfig {
                    kind: "DrumSampler".to_string(),
                    params: HashMap::new(),
                    plugin_state: None,
                }],
                send_to_master: true,
                ..Default::default()
            },
            TrackConfig {
                id: 2,
                name: "Rolling Bass".to_string(),
                channels: 2,
                gain: 0.85,
                pan: 0.0,
                clips: vec![SequenceConfig {
                    name: "Offbeat Rolling Bass".to_string(),
                    start_beat: 0.0,
                    step_division: 1.0,
                    steps: bass_steps,
                    ..Default::default()
                }],
                nodes: vec![
                    NodeConfig {
                        kind: "SubtractiveSynth".to_string(),
                        params: HashMap::new(),
                        plugin_state: None,
                    },
                    NodeConfig {
                        kind: "Compressor".to_string(),
                        params: HashMap::new(),
                        plugin_state: None,
                    },
                ],
                send_to_master: true,
                ..Default::default()
            },
            TrackConfig {
                id: 3,
                name: "Neon Lead".to_string(),
                channels: 2,
                gain: 0.8,
                pan: 0.15,
                clips: vec![SequenceConfig {
                    name: "Lead Melody".to_string(),
                    start_beat: 0.0,
                    step_division: 2.0,
                    steps: lead_steps,
                    ..Default::default()
                }],
                nodes: vec![
                    NodeConfig {
                        kind: "WavetableSynth".to_string(),
                        params: HashMap::new(),
                        plugin_state: None,
                    },
                    NodeConfig {
                        kind: "StereoDelay".to_string(),
                        params: HashMap::new(),
                        plugin_state: None,
                    },
                ],
                send_to_master: true,
                ..Default::default()
            },
            TrackConfig {
                id: 4,
                name: "Sidechain Ducking Bus".to_string(),
                channels: 2,
                gain: 1.0,
                pan: 0.0,
                nodes: vec![NodeConfig {
                    kind: "SidechainMatrix".to_string(),
                    params: HashMap::new(),
                    plugin_state: None,
                }],
                send_to_master: true,
                ..Default::default()
            },
            TrackConfig {
                id: 5,
                name: "Master".to_string(),
                channels: 2,
                gain: 1.0,
                pan: 0.0,
                nodes: vec![
                    NodeConfig {
                        kind: "ParametricEQ".to_string(),
                        params: HashMap::new(),
                        plugin_state: None,
                    },
                    NodeConfig {
                        kind: "TruePeakLimiter".to_string(),
                        params: HashMap::new(),
                        plugin_state: None,
                    },
                ],
                send_to_master: false,
                ..Default::default()
            },
        ];

        ProjectConfig {
            name: "Cosmic Drift (Demo)".to_string(),
            transport: TransportConfig {
                bpm: 124.0,
                time_signature: "4/4".to_string(),
                ..Default::default()
            },
            tracks,
            ..Default::default()
        }
    }

    /// Creates "Acoustic Horizon" Folk / Acoustic Session Demo Project.
    pub fn create_acoustic_demo() -> ProjectConfig {
        let tracks = vec![
            TrackConfig {
                id: 1,
                name: "Acoustic Guitar L".to_string(),
                channels: 2,
                gain: 0.75,
                pan: -0.4,
                audio_clips: vec![AudioClipConfig::new(
                    "clip_1",
                    "Acoustic Guitar L",
                    "acoustic_guitar_take1.wav",
                    0.0,
                    32.0,
                )],
                nodes: vec![NodeConfig {
                    kind: "ParametricEQ".to_string(),
                    params: HashMap::new(),
                    plugin_state: None,
                }],
                send_to_master: true,
                ..Default::default()
            },
            TrackConfig {
                id: 2,
                name: "Acoustic Guitar R".to_string(),
                channels: 2,
                gain: 0.75,
                pan: 0.4,
                audio_clips: vec![AudioClipConfig::new(
                    "clip_2",
                    "Acoustic Guitar R",
                    "acoustic_guitar_take2.wav",
                    0.0,
                    32.0,
                )],
                nodes: vec![NodeConfig {
                    kind: "ParametricEQ".to_string(),
                    params: HashMap::new(),
                    plugin_state: None,
                }],
                send_to_master: true,
                ..Default::default()
            },
            TrackConfig {
                id: 3,
                name: "Lead Vocal".to_string(),
                channels: 2,
                gain: 0.9,
                pan: 0.0,
                audio_clips: vec![AudioClipConfig::new(
                    "clip_3",
                    "Lead Vocal Comp",
                    "lead_vocal_comp.wav",
                    0.0,
                    32.0,
                )],
                nodes: vec![
                    NodeConfig {
                        kind: "VocalDeEsser".to_string(),
                        params: HashMap::new(),
                        plugin_state: None,
                    },
                    NodeConfig {
                        kind: "Compressor".to_string(),
                        params: HashMap::new(),
                        plugin_state: None,
                    },
                ],
                send_to_master: true,
                ..Default::default()
            },
            TrackConfig {
                id: 4,
                name: "Master".to_string(),
                channels: 2,
                gain: 1.0,
                pan: 0.0,
                nodes: vec![NodeConfig {
                    kind: "TruePeakLimiter".to_string(),
                    params: HashMap::new(),
                    plugin_state: None,
                }],
                send_to_master: false,
                ..Default::default()
            },
        ];

        ProjectConfig {
            name: "Acoustic Horizon (Demo)".to_string(),
            transport: TransportConfig {
                bpm: 96.0,
                time_signature: "4/4".to_string(),
                ..Default::default()
            },
            tracks,
            ..Default::default()
        }
    }

    /// Creates "Deep Space Ambience" 5.1 Surround Immersive Demo Project.
    pub fn create_immersive_demo() -> ProjectConfig {
        let tracks = vec![
            TrackConfig {
                id: 1,
                name: "Front Left / Right Bed".to_string(),
                channels: 2,
                gain: 0.8,
                pan: 0.0,
                nodes: vec![NodeConfig {
                    kind: "AmbientDroneGenerator".to_string(),
                    params: HashMap::new(),
                    plugin_state: None,
                }],
                send_to_master: true,
                ..Default::default()
            },
            TrackConfig {
                id: 2,
                name: "Center Dialogue / Lead".to_string(),
                channels: 1,
                gain: 0.85,
                pan: 0.0,
                nodes: vec![NodeConfig {
                    kind: "ParametricEQ".to_string(),
                    params: HashMap::new(),
                    plugin_state: None,
                }],
                send_to_master: true,
                ..Default::default()
            },
            TrackConfig {
                id: 3,
                name: "LFE Sub-Bass Rumble".to_string(),
                channels: 1,
                gain: 0.9,
                pan: 0.0,
                nodes: vec![NodeConfig {
                    kind: "LowPassFilter".to_string(),
                    params: HashMap::new(),
                    plugin_state: None,
                }],
                send_to_master: true,
                ..Default::default()
            },
            TrackConfig {
                id: 4,
                name: "Surround Rear L/R Ambience".to_string(),
                channels: 2,
                gain: 0.7,
                pan: 0.0,
                nodes: vec![NodeConfig {
                    kind: "StereoReverb".to_string(),
                    params: HashMap::new(),
                    plugin_state: None,
                }],
                send_to_master: true,
                ..Default::default()
            },
            TrackConfig {
                id: 5,
                name: "Downmix Matrix Fold-Down Preview".to_string(),
                channels: 2,
                gain: 1.0,
                pan: 0.0,
                nodes: vec![NodeConfig {
                    kind: "DownmixMatrixNode".to_string(),
                    params: HashMap::new(),
                    plugin_state: None,
                }],
                send_to_master: true,
                ..Default::default()
            },
            TrackConfig {
                id: 6,
                name: "Master 5.1 Bus".to_string(),
                channels: 6,
                gain: 1.0,
                pan: 0.0,
                nodes: vec![NodeConfig {
                    kind: "MultichannelLimiter".to_string(),
                    params: HashMap::new(),
                    plugin_state: None,
                }],
                send_to_master: false,
                ..Default::default()
            },
        ];

        ProjectConfig {
            name: "Deep Space Ambience (5.1 Demo)".to_string(),
            transport: TransportConfig {
                bpm: 72.0,
                time_signature: "4/4".to_string(),
                ..Default::default()
            },
            tracks,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_onboarding_milestone_progression() {
        let mut mgr = OnboardingManager::new();
        assert_eq!(mgr.progress_percentage(), 0.0);

        mgr.complete_milestone(OnboardingMilestone::WelcomeSeen);
        mgr.complete_milestone(OnboardingMilestone::AudioDeviceConfigured);
        assert!(mgr.is_milestone_completed(OnboardingMilestone::WelcomeSeen));
        assert!(mgr.progress_percentage() > 30.0);

        // Idempotent
        mgr.complete_milestone(OnboardingMilestone::WelcomeSeen);
        assert_eq!(mgr.completed_milestones.len(), 2);
    }

    #[test]
    fn test_demo_projects_catalog_validity() {
        let catalog = DemoProjects::catalog();
        assert_eq!(catalog.len(), 3);
        assert_eq!(catalog[0].id, "demo_cosmic_drift");
        assert_eq!(catalog[1].id, "demo_acoustic_horizon");
        assert_eq!(catalog[2].id, "demo_deep_space_ambience");
    }

    #[test]
    fn test_synthwave_demo_instantiation_and_serialization() {
        let proj = DemoProjects::create_synthwave_demo();
        assert_eq!(proj.tracks.len(), 5);
        assert_eq!(proj.transport.bpm, 124.0);

        // Verify TOML roundtrip
        let toml_str = toml::to_string(&proj).unwrap();
        let loaded: ProjectConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(loaded.name, "Cosmic Drift (Demo)");
        assert_eq!(loaded.tracks.len(), 5);
        assert_eq!(loaded.tracks[0].clips.len(), 1);
    }

    #[test]
    fn test_acoustic_and_immersive_demos() {
        let acoustic = DemoProjects::create_acoustic_demo();
        assert_eq!(acoustic.tracks.len(), 4);
        assert_eq!(acoustic.tracks[0].audio_clips.len(), 1);

        let immersive = DemoProjects::create_immersive_demo();
        assert_eq!(immersive.tracks.len(), 6);
        assert_eq!(immersive.tracks[5].channels, 6);
    }
}
