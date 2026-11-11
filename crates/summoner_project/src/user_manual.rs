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

//! Step 1260: Comprehensive User Manual, In-App Searchable Help System,
//! and Interactive Step-by-Step Workflow Tutorials.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Functional taxonomy categories for user manual documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HelpCategory {
    GettingStarted,
    RecordingAndAudio,
    ArrangingAndEditing,
    MidiAndSequencing,
    MixingAndEffects,
    InstrumentsAndSampling,
    ModularAndScripting,
    MasteringAndDelivery,
    ShortcutsAndWorkflows,
}

impl HelpCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::GettingStarted => "Getting Started",
            Self::RecordingAndAudio => "Recording & Audio Tracking",
            Self::ArrangingAndEditing => "Arranging & Clip Editing",
            Self::MidiAndSequencing => "MIDI & Piano Roll",
            Self::MixingAndEffects => "Mixing & Signal Routing",
            Self::InstrumentsAndSampling => "Instruments & Sampling",
            Self::ModularAndScripting => "Modular Patching & Scripting",
            Self::MasteringAndDelivery => "Mastering & Delivery",
            Self::ShortcutsAndWorkflows => "Shortcuts & Productivity",
        }
    }
}

/// An individual help article / topic in the user manual.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelpTopic {
    pub id: String,
    pub title: String,
    pub category: HelpCategory,
    pub keywords: Vec<String>,
    pub summary: String,
    pub content_markdown: String,
    pub keyboard_shortcut: Option<String>,
    pub related_topic_ids: Vec<String>,
}

impl HelpTopic {
    pub fn new(
        id: &str,
        title: &str,
        category: HelpCategory,
        summary: &str,
        content_markdown: &str,
    ) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            category,
            keywords: Vec::new(),
            summary: summary.to_string(),
            content_markdown: content_markdown.to_string(),
            keyboard_shortcut: None,
            related_topic_ids: Vec::new(),
        }
    }

    pub fn with_keywords(mut self, keywords: &[&str]) -> Self {
        self.keywords = keywords.iter().map(|k| k.to_string()).collect();
        self
    }

    pub fn with_shortcut(mut self, shortcut: &str) -> Self {
        self.keyboard_shortcut = Some(shortcut.to_string());
        self
    }

    pub fn with_related(mut self, related_ids: &[&str]) -> Self {
        self.related_topic_ids = related_ids.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Computes keyword relevance score for a search term.
    pub fn match_score(&self, query: &str) -> usize {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return 0;
        }

        let mut score = 0;
        let title_lower = self.title.to_lowercase();
        if title_lower == q {
            score += 100;
        } else if title_lower.contains(&q) {
            score += 40;
        }

        for kw in &self.keywords {
            let kw_lower = kw.to_lowercase();
            if kw_lower == q {
                score += 50;
            } else if kw_lower.contains(&q) {
                score += 20;
            }
        }

        if self.summary.to_lowercase().contains(&q) {
            score += 15;
        }

        if self.content_markdown.to_lowercase().contains(&q) {
            score += 5;
        }

        score
    }
}

/// A discrete step in an interactive walkthrough tutorial.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorialStep {
    pub step_number: usize,
    pub title: String,
    pub instruction: String,
    pub target_ui_element: String,
    pub action_hint: String,
    pub keyboard_shortcut: Option<String>,
}

/// Interactive guided workflow tutorial.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTutorial {
    pub id: String,
    pub title: String,
    pub description: String,
    pub difficulty: String,
    pub estimated_minutes: u32,
    pub steps: Vec<TutorialStep>,
}

impl WorkflowTutorial {
    pub fn new(
        id: &str,
        title: &str,
        description: &str,
        difficulty: &str,
        estimated_minutes: u32,
    ) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            difficulty: difficulty.to_string(),
            estimated_minutes,
            steps: Vec::new(),
        }
    }

    pub fn add_step(
        mut self,
        title: &str,
        instruction: &str,
        target_ui: &str,
        hint: &str,
        shortcut: Option<&str>,
    ) -> Self {
        let step_number = self.steps.len() + 1;
        self.steps.push(TutorialStep {
            step_number,
            title: title.to_string(),
            instruction: instruction.to_string(),
            target_ui_element: target_ui.to_string(),
            action_hint: hint.to_string(),
            keyboard_shortcut: shortcut.map(|s| s.to_string()),
        });
        self
    }
}

/// Central User Manual and Documentation Engine.
#[derive(Debug, Clone)]
pub struct UserManual {
    topics: HashMap<String, HelpTopic>,
    tutorials: HashMap<String, WorkflowTutorial>,
}

impl Default for UserManual {
    fn default() -> Self {
        let mut manual = Self {
            topics: HashMap::new(),
            tutorials: HashMap::new(),
        };
        manual.load_default_manual();
        manual
    }
}

impl UserManual {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_topic(&mut self, topic: HelpTopic) {
        self.topics.insert(topic.id.clone(), topic);
    }

    pub fn insert_tutorial(&mut self, tutorial: WorkflowTutorial) {
        self.tutorials.insert(tutorial.id.clone(), tutorial);
    }

    pub fn get_topic(&self, id: &str) -> Option<&HelpTopic> {
        self.topics.get(id)
    }

    pub fn get_tutorial(&self, id: &str) -> Option<&WorkflowTutorial> {
        self.tutorials.get(id)
    }

    pub fn all_topics(&self) -> Vec<&HelpTopic> {
        self.topics.values().collect()
    }

    pub fn all_tutorials(&self) -> Vec<&WorkflowTutorial> {
        self.tutorials.values().collect()
    }

    /// Search documentation topics by natural language keywords or title fragments.
    pub fn search(&self, query: &str) -> Vec<&HelpTopic> {
        let mut scored: Vec<(&HelpTopic, usize)> = self
            .topics
            .values()
            .map(|t| (t, t.match_score(query)))
            .filter(|(_, score)| *score > 0)
            .collect();

        scored.sort_by_key(|b| std::cmp::Reverse(b.1));
        scored.into_iter().map(|(t, _)| t).collect()
    }

    /// Compiles all manual topics into a single cohesive Markdown document.
    pub fn generate_full_markdown_manual(&self) -> String {
        let mut md = String::new();
        md.push_str("# Summoner DAW -- Comprehensive User Manual\n\n");
        md.push_str(
            "Welcome to Summoner, the deterministic, headless-first digital audio workstation.\n\n",
        );
        md.push_str("## Table of Contents\n\n");

        let categories = [
            HelpCategory::GettingStarted,
            HelpCategory::RecordingAndAudio,
            HelpCategory::ArrangingAndEditing,
            HelpCategory::MidiAndSequencing,
            HelpCategory::MixingAndEffects,
            HelpCategory::InstrumentsAndSampling,
            HelpCategory::ModularAndScripting,
            HelpCategory::MasteringAndDelivery,
            HelpCategory::ShortcutsAndWorkflows,
        ];

        for cat in &categories {
            let mut cat_topics: Vec<&HelpTopic> = self
                .topics
                .values()
                .filter(|t| t.category == *cat)
                .collect();
            cat_topics.sort_by(|a, b| a.title.cmp(&b.title));

            if !cat_topics.is_empty() {
                md.push_str(&format!("### {}\n", cat.display_name()));
                for topic in &cat_topics {
                    md.push_str(&format!("- [{}]({})\n", topic.title, topic.id));
                }
                md.push('\n');
            }
        }

        md.push_str("## Articles\n\n");
        for cat in &categories {
            let mut cat_topics: Vec<&HelpTopic> = self
                .topics
                .values()
                .filter(|t| t.category == *cat)
                .collect();
            cat_topics.sort_by(|a, b| a.title.cmp(&b.title));

            for topic in cat_topics {
                md.push_str(&format!("### {}\n\n", topic.title));
                md.push_str(&format!(
                    "*Category: {}*\n\n",
                    topic.category.display_name()
                ));
                if let Some(ref sc) = topic.keyboard_shortcut {
                    md.push_str(&format!("**Keyboard Shortcut:** `{}`\n\n", sc));
                }
                md.push_str(&format!("{}\n\n", topic.content_markdown));
                md.push_str("---\n\n");
            }
        }

        md
    }

    /// Populates factory manual content covering complete real-world DAW workflows.
    fn load_default_manual(&mut self) {
        // Topic 1: Audio Recording
        let t_rec = HelpTopic::new(
            "audio_recording",
            "Multi-Track Audio Recording & Low-Latency Monitoring",
            HelpCategory::RecordingAndAudio,
            "How to arm audio tracks, select inputs, configure hardware buffer sizes, and punch in.",
            "Summoner supports zero-latency direct audio recording from ASIO, CoreAudio, ALSA, and PipeWire drivers.\n\n\
            1. Create an Audio Track (`Ctrl+T` / `Cmd+T`).\n\
            2. Choose your hardware input channel from the Track Inspector.\n\
            3. Enable the **Arm** button (`R`).\n\
            4. Enable pre-roll count-in in the Transport bar.\n\
            5. Press `Space` to start recording. To punch in on the fly, press `R` during playback.",
        )
        .with_keywords(&["record", "audio", "mic", "latency", "arm", "punch in"])
        .with_shortcut("R")
        .with_related(&["buffer_size", "comping"]);
        self.insert_topic(t_rec);

        // Topic 2: Slip and Crossfade Editing
        let t_edit = HelpTopic::new(
            "clip_editing",
            "Arranger Clip Editing: Split, Trim, Slip, and Crossfades",
            HelpCategory::ArrangingAndEditing,
            "Non-destructive clip arrangement with equal-power crossfades and transient slip editing.",
            "Clips in the arranger are non-destructive viewports into underlying audio buffers.\n\n\
            - **Split at Playhead:** Press `S` to divide the selected clip at the current playhead position.\n\
            - **Slip Editing:** Hold `Alt` and drag inside a clip to slip the audio without moving the clip boundaries.\n\
            - **Equal-Power Crossfades:** Overlap two adjacent clips to automatically create an equal-power 3dB cosine crossfade.\n\
            - **Ripple Edit:** Toggle Ripple Mode to slide all subsequent clips when deleting or inserting time.",
        )
        .with_keywords(&["split", "trim", "slip", "crossfade", "fade", "ripple", "slice"])
        .with_shortcut("S")
        .with_related(&["audio_recording", "tempo_warp"]);
        self.insert_topic(t_edit);

        // Topic 3: MIDI Piano Roll
        let t_midi = HelpTopic::new(
            "piano_roll",
            "Expressive MIDI Piano Roll & CC Automation",
            HelpCategory::MidiAndSequencing,
            "Note entry, velocity scaling, MIDI CC curves, microtonal tuning, and chord stamp tools.",
            "The Piano Roll offers per-note articulation and CC automation.\n\n\
            - Double-click in the MIDI track to insert or open a MIDI clip.\n\
            - Use the Pencil Tool (`P`) to click and drag note placements.\n\
            - Quantize selected notes to the active grid using `Q`.\n\
            - Switch between velocity lanes, pitch bend, and modulation CCs below the key bed.",
        )
        .with_keywords(&["midi", "piano roll", "velocity", "quantize", "cc", "automation"])
        .with_shortcut("Q")
        .with_related(&["clip_editing", "stock_synths"]);
        self.insert_topic(t_midi);

        // Topic 4: Mixing and Sidechain Routing
        let t_mix = HelpTopic::new(
            "mixer_sidechain",
            "Bus Groups, VCA Faders, and Sidechain Compression",
            HelpCategory::MixingAndEffects,
            "Routing signals to sub-mixes, setting up ducking compressors, and managing delay compensation.",
            "Summoner's mixer engine features automatic whole-graph Plugin Delay Compensation (PDC).\n\n\
            - **Bus Tracks:** Create a Bus Track (`Ctrl+Shift+B`) and assign individual track outputs to it.\n\
            - **Sidechain Ducking:** Add a compressor to your bass track. In the Sidechain Source selector, choose your Kick drum track.\n\
            - **VCA Groups:** Control relative fader levels across multiple tracks without altering their audio routing.",
        )
        .with_keywords(&["mixer", "sidechain", "compressor", "ducking", "vca", "bus", "sends"])
        .with_shortcut("M")
        .with_related(&["immersive_downmix", "metering"]);
        self.insert_topic(t_mix);

        // Topic 5: Immersive & ADM BWF Export
        let t_surround = HelpTopic::new(
            "immersive_mastering",
            "Immersive Audio (5.1 & Dolby Atmos) & ADM BWF Export",
            HelpCategory::MasteringAndDelivery,
            "Spatial multichannel panning, bus downmixing matrices, and ITU-R BS.2076 ADM BWF metadata export.",
            "Summoner provides full spatial surround and immersive audio capabilities.\n\n\
            - Route tracks to 5.1, 7.1, or Atmos bed format.\n\
            - The built-in Multichannel Downmixer allows real-time fold-down preview to stereo with Lo/Ro and Lt/Rt matrix profiles.\n\
            - Export your session as a Broadcast Wave Format (BWF) WAV file embedding full ITU-R BS.2076 ADM XML metadata.",
        )
        .with_keywords(&["surround", "5.1", "atmos", "adm", "bwf", "downmix", "spatial", "loudness"])
        .with_related(&["mixer_sidechain", "clip_editing"]);
        self.insert_topic(t_surround);

        // Topic 6: Modular Cable Patching
        let t_modular = HelpTopic::new(
            "modular_patching",
            "Rear-Panel CV/Gate Patching & Live DSP Scripting",
            HelpCategory::ModularAndScripting,
            "Connecting control voltages, custom Lua scripts, and JIT-compiled Faust code nodes.",
            "Press `Tab` to flip the device rack and access the modular patching canvas.\n\n\
            - Drag virtual patch cables between CV LFO outputs and filter cutoff inputs.\n\
            - Insert a **Lua Script Node** to implement custom MIDI processors or parameter modulators.\n\
            - Insert a **Code Node** to write real-time Faust DSP algorithms that compile instantly without restarting.",
        )
        .with_keywords(&["modular", "patch", "cv", "gate", "lua", "script", "gen", "faust"])
        .with_shortcut("Tab")
        .with_related(&["piano_roll", "mixer_sidechain"]);
        self.insert_topic(t_modular);

        // Walkthrough Tutorials
        let tut_first_track = WorkflowTutorial::new(
            "tut_first_track",
            "Recording Your First Multi-Track Audio Session",
            "Step-by-step walkthrough from audio interface setup to your first recorded vocal or instrument take.",
            "Beginner",
            10,
        )
        .add_step(
            "Configure Audio Hardware",
            "Open Preferences (`Ctrl+,`), select Audio Device, and verify input/output channel mapping.",
            "dialog.preferences.audio",
            "Select ASIO or CoreAudio driver with 256 frame buffer size.",
            Some("Ctrl+,"),
        )
        .add_step(
            "Add Audio Track",
            "Create a new Audio Track and name it 'Lead Vocal'.",
            "arranger.add_track",
            "Press Ctrl+T or click the '+' icon in the track header list.",
            Some("Ctrl+T"),
        )
        .add_step(
            "Arm and Set Input",
            "Select hardware Input 1 and enable the Record Arm button (red circle).",
            "track.arm_button",
            "Sing or play your instrument to confirm healthy green/yellow peak levels (-18 to -12 dBFS).",
            Some("R"),
        )
        .add_step(
            "Enable Metronome & Count-In",
            "Turn on the Metronome click and set a 1-bar pre-roll count-in.",
            "transport.metronome",
            "Click the metronome icon in the top transport bar.",
            Some("K"),
        )
        .add_step(
            "Record & Play Back",
            "Hit Space or the Record button to begin recording. Press Space when finished to review your waveform.",
            "transport.record",
            "Press Space to stop, then Zero ('0') to return playhead to start and listen back.",
            Some("Space"),
        );
        self.insert_tutorial(tut_first_track);

        let tut_sidechain = WorkflowTutorial::new(
            "tut_sidechain_ducking",
            "Sidechain Compression & Dynamic Ducking",
            "Learn how to create pumping basslines by sidechaining the bass compressor to the kick drum.",
            "Intermediate",
            8,
        )
        .add_step(
            "Create Kick and Bass Tracks",
            "Ensure you have a Kick Drum track and a Bass track in your arrangement.",
            "arranger.tracks",
            "Use the Synthwave or EDM template for instant setup.",
            None,
        )
        .add_step(
            "Insert Compressor on Bass",
            "Drag the Master Compressor DSP node onto the Bass track rack.",
            "device_rack.insert",
            "Double-click Compressor in the content browser.",
            None,
        )
        .add_step(
            "Assign Sidechain Source",
            "In the Compressor inspector, select 'Kick Drum' as the External Sidechain Source.",
            "compressor.sidechain_source",
            "Set Sidechain Mode to 'Peak' with 15ms release time.",
            None,
        )
        .add_step(
            "Dial Ratio & Threshold",
            "Lower the threshold until 4-6 dB of gain reduction occurs on each kick hit.",
            "compressor.threshold",
            "Listen to how the bass automatically ducks out of the kick's fundamental frequency.",
            None,
        );
        self.insert_tutorial(tut_sidechain);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_manual_initialization_and_topics() {
        let manual = UserManual::new();
        assert!(manual.all_topics().len() >= 6);
        assert!(manual.all_tutorials().len() >= 2);

        let rec_topic = manual.get_topic("audio_recording").unwrap();
        assert_eq!(rec_topic.category, HelpCategory::RecordingAndAudio);
        assert_eq!(rec_topic.keyboard_shortcut, Some("R".to_string()));
    }

    #[test]
    fn test_search_scoring_relevance() {
        let manual = UserManual::new();

        // Exact keyword match
        let results = manual.search("crossfade");
        assert!(!results.is_empty());
        assert_eq!(results[0].id, "clip_editing");

        // Surround search
        let surround_results = manual.search("atmos");
        assert!(!surround_results.is_empty());
        assert_eq!(surround_results[0].id, "immersive_mastering");

        // Empty search returns nothing
        assert!(manual.search("").is_empty());
    }

    #[test]
    fn test_tutorial_steps_sequence() {
        let manual = UserManual::new();
        let tut = manual.get_tutorial("tut_first_track").unwrap();
        assert_eq!(tut.steps.len(), 5);
        assert_eq!(tut.steps[0].title, "Configure Audio Hardware");
        assert_eq!(tut.steps[4].keyboard_shortcut, Some("Space".to_string()));
    }

    #[test]
    fn test_markdown_manual_generation() {
        let manual = UserManual::new();
        let md = manual.generate_full_markdown_manual();
        assert!(md.contains("# Summoner DAW -- Comprehensive User Manual"));
        assert!(md.contains("Table of Contents"));
        assert!(md.contains("Recording & Audio Tracking"));
        assert!(md.contains("Rear-Panel CV/Gate Patching"));
    }
}
