// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Master Award-Winning DAW GUI View tying together the Triad Architecture.
//! Features fixed operational zones, tactile controls, and high-fidelity deterministic PNG rendering.

use crate::views::modern_asset_browser::{show_modern_asset_browser, ModernAssetBrowserState};
use crate::views::modern_device_rack::{show_modern_device_rack, ModernDeviceRackState};
use crate::views::modern_inspector::{show_modern_inspector_with_context, ModernInspectorState};
use crate::views::modern_top_bar::{show_modern_top_bar, ModernTopBarState};
use summoner_core::node::AudioNode;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, FontId, Rect, RichText, Rounding, Stroke, Vec2};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackVisualData {
    pub id: u64,
    pub name: String,
    pub color_rgb: [u8; 3],
    pub is_audio: bool,
    pub gain: f32,
    pub pan: f32,
    pub is_muted: bool,
    pub is_soloed: bool,
    pub is_armed: bool,
    pub clip_start_beat: f32,
    pub clip_length_beats: f32,
    #[serde(default)]
    pub clips: Vec<ArrangerClipVisual>,
    #[serde(default)]
    pub active_clip_idx: Option<usize>,
}

/// Interactive Arranger Clip visual data representation supporting slicing, trimming, and crossfading.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ArrangerClipVisual {
    pub id: usize,
    pub name: String,
    pub start_beat: f32,
    pub length_beats: f32,
    pub fade_in: f32,
    pub fade_out: f32,
    pub gain: f32,
    pub is_selected: bool,
}

impl Default for ArrangerClipVisual {
    fn default() -> Self {
        Self {
            id: 0,
            name: "Clip".to_string(),
            start_beat: 0.0,
            length_beats: 4.0,
            fade_in: 0.0,
            fade_out: 0.0,
            gain: 1.0,
            is_selected: false,
        }
    }
}

/// Active tool mode for the Arranger timeline canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum ArrangerToolMode {
    #[default]
    Pointer,
    Slice,
    Fade,
}

/// Snap grid resolution for the Arranger timeline canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum ArrangerSnapResolution {
    Bar1,            // 4.0 beats (1 measure)
    #[default]
    BeatQuarter,     // 1.0 beat (1/4 note)
    BeatEighth,      // 0.5 beat (1/8 note)
    BeatSixteenth,   // 0.25 beat (1/16 note)
    Free,            // 0.0 (off)
}

impl ArrangerSnapResolution {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Bar1 => "1 Bar (4b)",
            Self::BeatQuarter => "1/4 Beat (1b)",
            Self::BeatEighth => "1/8 Beat (0.5b)",
            Self::BeatSixteenth => "1/16 Beat (0.25b)",
            Self::Free => "Off (Free)",
        }
    }

    pub fn step_beats(&self) -> f32 {
        match self {
            Self::Bar1 => 4.0,
            Self::BeatQuarter => 1.0,
            Self::BeatEighth => 0.5,
            Self::BeatSixteenth => 0.25,
            Self::Free => 0.0,
        }
    }

    pub fn snap(&self, beat: f32) -> f32 {
        let step = self.step_beats();
        if step <= 0.0 {
            beat
        } else {
            (beat / step).round() * step
        }
    }
}

/// Quantization resolution for live clip/scene launching in the Stage view matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum StageLaunchQuantize {
    #[default]
    Bar1,        // 4.0 beats (1 measure)
    BarHalf,     // 2.0 beats (1/2 measure)
    Beat1,       // 1.0 beat (1/4 note)
    Instant,     // 0.0 (immediate launch)
}

impl StageLaunchQuantize {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Bar1 => "1 Bar (4b)",
            Self::BarHalf => "1/2 Bar (2b)",
            Self::Beat1 => "1 Beat (1b)",
            Self::Instant => "Instant (0b)",
        }
    }

    pub fn step_beats(&self) -> f32 {
        match self {
            Self::Bar1 => 4.0,
            Self::BarHalf => 2.0,
            Self::Beat1 => 1.0,
            Self::Instant => 0.0,
        }
    }

    pub fn next_boundary(&self, current_beat: f32) -> f32 {
        let step = self.step_beats();
        if step <= 0.0 {
            current_beat
        } else {
            (current_beat / step).ceil() * step
        }
    }
}

impl TrackVisualData {
    /// Ensure the track has at least one active Arranger clip based on defaults.
    pub fn ensure_clips(&mut self) {
        if self.clips.is_empty() && self.clip_length_beats > 0.0 {
            self.clips.push(ArrangerClipVisual {
                id: 1,
                name: self.name.clone(),
                start_beat: self.clip_start_beat,
                length_beats: self.clip_length_beats,
                fade_in: 0.0,
                fade_out: 0.0,
                gain: 1.0,
                is_selected: false,
            });
        }
    }

    /// Split a clip into two distinct clips at a given beat offset along the timeline.
    pub fn split_clip_at_beat(&mut self, clip_id: usize, beat: f32) -> bool {
        self.ensure_clips();
        if let Some(pos) = self.clips.iter().position(|c| c.id == clip_id) {
            let clip = &self.clips[pos];
            if beat > clip.start_beat && beat < clip.start_beat + clip.length_beats {
                let first_len = beat - clip.start_beat;
                let second_len = clip.length_beats - first_len;
                let second_start = beat;
                let orig_name = clip.name.clone();
                let orig_gain = clip.gain;
                let orig_fade_out = clip.fade_out;

                // Truncate first clip
                self.clips[pos].length_beats = first_len;
                self.clips[pos].fade_out = 0.05_f32.min(first_len * 0.25);

                // Add second clip
                let next_id = self.clips.iter().map(|c| c.id).max().unwrap_or(0) + 1;
                let second_clip = ArrangerClipVisual {
                    id: next_id,
                    name: format!("{} (2)", orig_name),
                    start_beat: second_start,
                    length_beats: second_len,
                    fade_in: 0.05_f32.min(second_len * 0.25),
                    fade_out: orig_fade_out,
                    gain: orig_gain,
                    is_selected: false,
                };
                self.clips.insert(pos + 1, second_clip);
                return true;
            }
        }
        false
    }

    /// Duplicate a clip, placing the copy immediately adjacent along the timeline.
    pub fn duplicate_clip(&mut self, clip_id: usize) -> Option<usize> {
        self.ensure_clips();
        if let Some(pos) = self.clips.iter().position(|c| c.id == clip_id) {
            let orig = &self.clips[pos];
            let next_id = self.clips.iter().map(|c| c.id).max().unwrap_or(0) + 1;
            let dup = ArrangerClipVisual {
                id: next_id,
                name: format!("{} (Copy)", orig.name),
                start_beat: orig.start_beat + orig.length_beats,
                length_beats: orig.length_beats,
                fade_in: orig.fade_in,
                fade_out: orig.fade_out,
                gain: orig.gain,
                is_selected: false,
            };
            self.clips.insert(pos + 1, dup);
            Some(next_id)
        } else {
            None
        }
    }

    /// Delete a clip with the specified id.
    pub fn delete_clip(&mut self, clip_id: usize) -> bool {
        let initial_len = self.clips.len();
        self.clips.retain(|c| c.id != clip_id);
        let removed = self.clips.len() < initial_len;
        if self.clips.is_empty() {
            self.clip_length_beats = 0.0;
        }
        removed
    }

    /// Quantize clip start_beat to the specified snap grid step.
    pub fn quantize_clip(&mut self, clip_id: usize, snap_step: f32) -> bool {
        if snap_step <= 0.0 {
            return false;
        }
        if let Some(clip) = self.clips.iter_mut().find(|c| c.id == clip_id) {
            clip.start_beat = ((clip.start_beat / snap_step).round() * snap_step).max(0.0);
            true
        } else {
            false
        }
    }

    /// Move a clip by delta_beats along the timeline.
    pub fn move_clip(&mut self, clip_id: usize, delta_beats: f32) -> bool {
        if let Some(clip) = self.clips.iter_mut().find(|c| c.id == clip_id) {
            clip.start_beat = (clip.start_beat + delta_beats).max(0.0);
            true
        } else {
            false
        }
    }

    /// Detect overlapping regions between clips on this track for crossfading.
    /// Returns vector of (overlap_start_beat, overlap_end_beat, clip1_id, clip2_id).
    pub fn detect_crossfades(&self) -> Vec<(f32, f32, usize, usize)> {
        let mut xfades = Vec::new();
        for i in 0..self.clips.len() {
            for j in (i + 1)..self.clips.len() {
                let c1 = &self.clips[i];
                let c2 = &self.clips[j];
                let s1 = c1.start_beat;
                let e1 = c1.start_beat + c1.length_beats;
                let s2 = c2.start_beat;
                let e2 = c2.start_beat + c2.length_beats;
                let o_start = s1.max(s2);
                let o_end = e1.min(e2);
                if o_end > o_start {
                    xfades.push((o_start, o_end, c1.id, c2.id));
                }
            }
        }
        xfades
    }
}

/// Canvas display mode for Pro Modular view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ModularCanvasMode {
    #[default]
    PatchCords,
    RoutingMatrix,
}

impl ModularCanvasMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::PatchCords => "∿ Patch Cords",
            Self::RoutingMatrix => "▦ Routing Matrix",
        }
    }
}

/// Port socket signal type for modular nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModularPortKind {
    AudioIn,
    AudioOut,
    ModulationIn,
    ModulationOut,
    GateIn,
    GateOut,
}

impl ModularPortKind {
    pub fn color_rgb(&self) -> (u8, u8, u8) {
        match self {
            Self::AudioIn | Self::AudioOut => (56, 189, 248),        // Cyan
            Self::ModulationIn | Self::ModulationOut => (245, 158, 11), // Amber
            Self::GateIn | Self::GateOut => (236, 72, 153),           // Neon Pink
        }
    }

    pub fn is_output(&self) -> bool {
        matches!(self, Self::AudioOut | Self::ModulationOut | Self::GateOut)
    }

    pub fn is_modulation(&self) -> bool {
        matches!(self, Self::ModulationIn | Self::ModulationOut | Self::GateIn | Self::GateOut)
    }
}

/// A socket port on a modular node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModularPort {
    pub id: String,
    pub name: String,
    pub kind: ModularPortKind,
    pub rel_pos: (f32, f32),
}

/// A tactile parameter on a modular DSP node faceplate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModularNodeParam {
    pub id: String,
    pub name: String,
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub default_value: f32,
    pub step: Option<f32>,
    pub unit: String,
    pub rel_pos: (f32, f32),
}

/// An instantiated modular DSP node in the modular canvas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModularNodeInstance {
    pub id: String,
    pub kind_id: String,
    pub display_name: String,
    pub category: crate::dsp_node_ui::DspNodeCategory,
    pub pos: (f32, f32),
    pub size: (f32, f32),
    pub ports: Vec<ModularPort>,
    #[serde(default)]
    pub params: Vec<ModularNodeParam>,
    #[serde(default)]
    pub graph_node_idx: Option<usize>,
    #[serde(default)]
    pub bypassed: bool,
}

/// A routed patch cord between two modular node ports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModularPatchCord {
    pub from_node_id: String,
    pub from_port_id: String,
    pub to_node_id: String,
    pub to_port_id: String,
    pub is_audio: bool,
    pub intensity: f32,
}

/// Interactive MIDI note block inside Piano Roll Canvas.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PianoRollNote {
    pub id: usize,
    pub pitch_idx: usize, // 0..num_pitches - 1
    pub start_beat: f32,
    pub length_beats: f32,
    pub velocity: f32,
}

/// Microtonal and standard tuning systems supported by the Piano Roll canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PianoRollTuningSystem {
    Edo12,
    Edo19,
    Edo31,
    BohlenPierce,
}

impl PianoRollTuningSystem {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Edo12 => "12-EDO (Standard)",
            Self::Edo19 => "19-EDO (Microtonal)",
            Self::Edo31 => "31-EDO (Fokker Meantone)",
            Self::BohlenPierce => "Bohlen-Pierce (Tritave 3:1)",
        }
    }

    pub fn pitch_names(&self) -> Vec<String> {
        match self {
            Self::Edo12 => {
                vec![
                    "C5".into(), "B4".into(), "A#4".into(), "A4".into(), "G#4".into(), "G4".into(),
                    "F#4".into(), "F4".into(), "E4".into(), "D#4".into(), "D4".into(), "C#4".into(), "C4".into(),
                ]
            }
            Self::Edo19 => {
                vec![
                    "C5".into(), "B#4".into(), "B4".into(), "A#4".into(), "Bb4".into(), "A4".into(),
                    "G#4".into(), "Ab4".into(), "G4".into(), "F#4".into(), "Gb4".into(), "F4".into(),
                    "E#4".into(), "E4".into(), "D#4".into(), "Eb4".into(), "D4".into(), "C#4".into(),
                    "Db4".into(), "C4".into(),
                ]
            }
            Self::Edo31 => {
                (0..=31).rev().map(|step| {
                    if step == 31 {
                        "C5 (1200¢)".into()
                    } else if step == 0 {
                        "C4 (0¢)".into()
                    } else {
                        format!("S{:02} ({:.0}¢)", step, (step as f32 * 1200.0 / 31.0))
                    }
                }).collect()
            }
            Self::BohlenPierce => {
                let bp = ["C", "C#", "D", "E", "F", "F#", "G", "H", "H#", "J", "A", "A#", "B", "C'"];
                bp.iter().rev().map(|&s| s.to_string()).collect()
            }
        }
    }

    pub fn pitch_to_hz(&self, p_idx: usize, total_pitches: usize) -> f32 {
        let base_c4 = 261.6256_f32;
        let step_from_bottom = (total_pitches.saturating_sub(1)).saturating_sub(p_idx) as f32;
        match self {
            Self::Edo12 => base_c4 * 2.0_f32.powf(step_from_bottom / 12.0),
            Self::Edo19 => base_c4 * 2.0_f32.powf(step_from_bottom / 19.0),
            Self::Edo31 => base_c4 * 2.0_f32.powf(step_from_bottom / 31.0),
            Self::BohlenPierce => base_c4 * 3.0_f32.powf(step_from_bottom / 13.0),
        }
    }
}

/// Snap grid quantization resolutions for note editing in Piano Roll canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PianoRollSnapResolution {
    BeatQuarter,     // 1.0 beat
    BeatEighth,      // 0.5 beat
    BeatSixteenth,   // 0.25 beat
    BeatThirtySecond,// 0.125 beat
    Free,            // 0.0 (off)
}

impl PianoRollSnapResolution {
    pub fn name(&self) -> &'static str {
        match self {
            Self::BeatQuarter => "1/4 Beat",
            Self::BeatEighth => "1/8 Beat",
            Self::BeatSixteenth => "1/16 Beat",
            Self::BeatThirtySecond => "1/32 Beat",
            Self::Free => "Off (Free)",
        }
    }

    pub fn step_beats(&self) -> f32 {
        match self {
            Self::BeatQuarter => 1.0,
            Self::BeatEighth => 0.5,
            Self::BeatSixteenth => 0.25,
            Self::BeatThirtySecond => 0.125,
            Self::Free => 0.0,
        }
    }

    pub fn snap(&self, beat: f32) -> f32 {
        let step = self.step_beats();
        if step <= 0.0 {
            beat
        } else {
            (beat / step).round() * step
        }
    }
}

/// Editing mode for the lower lane in Piano Roll canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum PianoRollLaneMode {
    #[default]
    Velocity,
    Gate,
    PitchBend,
}

impl PianoRollLaneMode {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Velocity => "Velocity",
            Self::Gate => "Gate Length",
            Self::PitchBend => "Pitch Bend",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AwardWinningGuiView {
    pub top_bar_state: ModernTopBarState,
    pub asset_browser_state: ModernAssetBrowserState,
    pub inspector_state: ModernInspectorState,
    pub device_rack_state: ModernDeviceRackState,
    pub playhead_beat: f32,
    pub loop_start_beat: f32,
    pub loop_end_beat: f32,
    pub selected_track_idx: usize,
    pub tracks: Vec<TrackVisualData>,
    pub patch_matrix: crate::patch_matrix::PatchMatrixView,
    pub modular_mode: ModularCanvasMode,
    pub modular_nodes: Vec<ModularNodeInstance>,
    pub patch_cords: Vec<ModularPatchCord>,
    pub selected_modular_node_id: Option<String>,
    pub selected_patch_cord_idx: Option<usize>,
    pub pending_cord_source: Option<(String, String)>,
    pub arranger_tool_mode: ArrangerToolMode,
    pub arranger_snap_grid: ArrangerSnapResolution,
    pub selected_clip: Option<(usize, usize)>,
    pub audio_graph: std::sync::Arc<std::sync::Mutex<summoner_core::graph::NodeGraph>>,
    pub last_applied_preset: String,
    pub last_applied_macros: [f32; 4],
    pub last_synced_bpm: f64,
    pub last_synced_is_playing: bool,
    pub active_scene_idx: Option<usize>,
    pub panic_triggered: bool,
    pub stage_quantize: StageLaunchQuantize,
    pub last_tap_time: Option<std::time::Instant>,
    pub last_inspector_gain_db: f32,
    pub last_inspector_pan: f32,
    pub last_inspector_muted: bool,
    pub last_inspector_soloed: bool,
    pub last_inspector_armed: bool,
    pub last_selected_track_idx: usize,
    pub last_applied_master_gain: f32,
    pub piano_roll_notes: Vec<PianoRollNote>,
    pub selected_note_id: Option<usize>,
    pub auditioned_pitch_idx: Option<usize>,
    pub next_note_id: usize,
    pub last_synced_track_notes_idx: usize,
    pub current_oscilloscope_samples: Option<Vec<f32>>,
    pub last_device_rack_node_kind: Option<String>,
    pub last_inspector_node_kind: Option<String>,
    pub last_device_rack_node_param_values: std::collections::HashMap<String, f32>,
    pub last_inspector_node_param_values: std::collections::HashMap<String, f32>,
    pub modular_add_modal_open: bool,
    pub modular_search_query: String,
    pub modular_selected_category: Option<crate::dsp_node_ui::DspCategory>,
    pub piano_roll_tuning: PianoRollTuningSystem,
    pub piano_roll_snap: PianoRollSnapResolution,
    pub piano_roll_resizing_note_id: Option<usize>,
    pub last_auditioned_pitch_hz: f32,
    pub last_auditioned_gate: f32,
    pub requested_modular_automation_param: Option<String>,
    pub show_automation_editor_window: bool,
    pub automation_editor: Option<crate::views::bezier_automation_editor::BezierAutomationEditorView>,
    pub is_master_muted: bool,
    pub unmuted_master_gain: f32,
    pub piano_roll_lane_mode: PianoRollLaneMode,
    pub is_piano_roll_audition_enabled: bool,
    pub is_master_mono: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutomationQuickShape {
    Flat,
    RampUp,
    RampDown,
    SineLfo,
    ExpDrop,
    SCurve,
    Invert,
    Smooth,
}

impl Default for AwardWinningGuiView {
    fn default() -> Self {
        Self::new()
    }
}

impl AwardWinningGuiView {
    pub fn new() -> Self {
        let tracks = vec![
            TrackVisualData {
                id: 1,
                name: "Kick".to_string(),
                color_rgb: [56, 189, 248], // Cyan
                is_audio: true,
                gain: 1.0,
                pan: 0.0,
                is_muted: false,
                is_soloed: false,
                is_armed: false,
                clip_start_beat: 0.0,
                clip_length_beats: 16.0,
                clips: Vec::new(),
                active_clip_idx: None,
            },
            TrackVisualData {
                id: 2,
                name: "Snare".to_string(),
                color_rgb: [245, 158, 11], // Amber / Orange
                is_audio: true,
                gain: 0.9,
                pan: 0.0,
                is_muted: false,
                is_soloed: false,
                is_armed: false,
                clip_start_beat: 2.0,
                clip_length_beats: 14.0,
                clips: Vec::new(),
                active_clip_idx: None,
            },
            TrackVisualData {
                id: 3,
                name: "HiHats".to_string(),
                color_rgb: [132, 204, 22], // Lime
                is_audio: true,
                gain: 0.75,
                pan: 0.1,
                is_muted: false,
                is_soloed: false,
                is_armed: false,
                clip_start_beat: 2.0,
                clip_length_beats: 14.0,
                clips: Vec::new(),
                active_clip_idx: None,
            },
            TrackVisualData {
                id: 4,
                name: "Bassline".to_string(),
                color_rgb: [16, 185, 129], // Emerald / Mint
                is_audio: false,
                gain: 1.1,
                pan: 0.0,
                is_muted: false,
                is_soloed: false,
                is_armed: false,
                clip_start_beat: 1.0,
                clip_length_beats: 15.0,
                clips: Vec::new(),
                active_clip_idx: None,
            },
            TrackVisualData {
                id: 5,
                name: "Synth 1".to_string(),
                color_rgb: [56, 189, 248], // Electric Blue / Cyan (Selected)
                is_audio: false,
                gain: 1.0,
                pan: 0.0,
                is_muted: false,
                is_soloed: false,
                is_armed: true,
                clip_start_beat: 4.0,
                clip_length_beats: 12.0,
                clips: Vec::new(),
                active_clip_idx: None,
            },
            TrackVisualData {
                id: 6,
                name: "Vocals".to_string(),
                color_rgb: [236, 72, 153], // Magenta / Pink
                is_audio: false,
                gain: 0.95,
                pan: -0.1,
                is_muted: false,
                is_soloed: false,
                is_armed: false,
                clip_start_beat: 2.0,
                clip_length_beats: 10.0,
                clips: Vec::new(),
                active_clip_idx: None,
            },
            TrackVisualData {
                id: 7,
                name: "Arp".to_string(),
                color_rgb: [163, 230, 53], // Yellow-Green / Lime
                is_audio: false,
                gain: 0.85,
                pan: 0.2,
                is_muted: false,
                is_soloed: false,
                is_armed: false,
                clip_start_beat: 6.0,
                clip_length_beats: 10.0,
                clips: Vec::new(),
                active_clip_idx: None,
            },
            TrackVisualData {
                id: 8,
                name: "Pads".to_string(),
                color_rgb: [244, 63, 94], // Rose / Magenta
                is_audio: true,
                gain: 0.80,
                pan: -0.2,
                is_muted: false,
                is_soloed: false,
                is_armed: false,
                clip_start_beat: 7.0,
                clip_length_beats: 9.0,
                clips: Vec::new(),
                active_clip_idx: None,
            },
        ];

        let mut view = Self {
            top_bar_state: ModernTopBarState::default(),
            asset_browser_state: ModernAssetBrowserState::default(),
            inspector_state: ModernInspectorState::default(),
            device_rack_state: ModernDeviceRackState::default(),
            playhead_beat: 5.5,
            loop_start_beat: 0.0,
            loop_end_beat: 8.0,
            selected_track_idx: 4, // Synth 1
            tracks,
            patch_matrix: crate::patch_matrix::PatchMatrixView::default(),
            modular_mode: ModularCanvasMode::PatchCords,
            modular_nodes: Vec::new(),
            patch_cords: Vec::new(),
            selected_modular_node_id: Some("osc_1".to_string()),
            selected_patch_cord_idx: None,
            pending_cord_source: None,
            arranger_tool_mode: ArrangerToolMode::Pointer,
            arranger_snap_grid: ArrangerSnapResolution::BeatQuarter,
            selected_clip: None,
            audio_graph: std::sync::Arc::new(std::sync::Mutex::new(
                summoner_core::graph::NodeGraph::new("ModularCanvasGraph", 512, 2)
            )),
            last_applied_preset: "Init Synth 1".to_string(),
            last_applied_macros: [0.65, 0.40, 0.55, 0.50],
            last_synced_bpm: 120.0,
            last_synced_is_playing: false,
            active_scene_idx: None,
            panic_triggered: false,
            stage_quantize: StageLaunchQuantize::default(),
            last_tap_time: None,
            last_inspector_gain_db: 0.0,
            last_inspector_pan: 0.0,
            last_inspector_muted: false,
            last_inspector_soloed: false,
            last_inspector_armed: false,
            last_selected_track_idx: 4,
            last_applied_master_gain: 1.0,
            piano_roll_notes: vec![
                PianoRollNote { id: 1, pitch_idx: 3, start_beat: 0.0, length_beats: 1.5, velocity: 0.90 },
                PianoRollNote { id: 2, pitch_idx: 5, start_beat: 2.0, length_beats: 1.5, velocity: 0.80 },
                PianoRollNote { id: 3, pitch_idx: 8, start_beat: 4.0, length_beats: 2.0, velocity: 0.85 },
                PianoRollNote { id: 4, pitch_idx: 10, start_beat: 6.0, length_beats: 1.5, velocity: 0.75 },
                PianoRollNote { id: 5, pitch_idx: 12, start_beat: 8.0, length_beats: 3.0, velocity: 0.95 },
                PianoRollNote { id: 6, pitch_idx: 10, start_beat: 11.5, length_beats: 1.0, velocity: 0.70 },
                PianoRollNote { id: 7, pitch_idx: 8, start_beat: 13.0, length_beats: 2.0, velocity: 0.85 },
                PianoRollNote { id: 8, pitch_idx: 3, start_beat: 15.0, length_beats: 2.0, velocity: 1.00 },
            ],
            selected_note_id: Some(1),
            auditioned_pitch_idx: None,
            next_note_id: 9,
            last_synced_track_notes_idx: 4,
            current_oscilloscope_samples: None,
            last_device_rack_node_kind: Some("AetherSynth".to_string()),
            last_inspector_node_kind: Some("AetherSynth".to_string()),
            last_device_rack_node_param_values: std::collections::HashMap::new(),
            last_inspector_node_param_values: std::collections::HashMap::new(),
            modular_add_modal_open: false,
            modular_search_query: String::new(),
            modular_selected_category: None,
            piano_roll_tuning: PianoRollTuningSystem::Edo12,
            piano_roll_snap: PianoRollSnapResolution::BeatEighth,
            piano_roll_resizing_note_id: None,
            last_auditioned_pitch_hz: 440.0,
            last_auditioned_gate: 0.0,
            requested_modular_automation_param: None,
            show_automation_editor_window: false,
            automation_editor: None,
            is_master_muted: false,
            unmuted_master_gain: 1.0,
            piano_roll_lane_mode: PianoRollLaneMode::Velocity,
            is_piano_roll_audition_enabled: true,
            is_master_mono: false,
        };
        view.reset_modular_nodes();
        view.device_rack_state.device_name = "Synth 1".to_string();
        view.device_rack_state.selected_node_kind = Some("AetherSynth".to_string());
        view.inspector_state.target_name = "Synth 1".to_string();
        view.inspector_state.selected_node_kind = Some("AetherSynth".to_string());
        view.last_device_rack_node_kind = Some("AetherSynth".to_string());
        view.last_inspector_node_kind = Some("AetherSynth".to_string());
        view
    }

    #[cfg(feature = "gui")]
    pub fn show(&mut self, ui: &mut egui::Ui) {
        // 0. Handle Pending Factory Demo Template load request
        if let Some(ref tmpl_id) = self.top_bar_state.pending_demo_template.take() {
            self.load_factory_demo_template(tmpl_id);
        }

        // 1. Synchronize Novice Presets to Active Device Rack & Inspector
        if self.top_bar_state.selected_preset != self.last_applied_preset {
            self.last_applied_preset = self.top_bar_state.selected_preset.clone();
            if let Some(preset) = crate::factory_presets::find_preset(&self.last_applied_preset) {
                self.top_bar_state.macro_tone = preset.macro_tone;
                self.top_bar_state.macro_space = preset.macro_space;
                self.top_bar_state.macro_punch = preset.macro_punch;
                self.top_bar_state.macro_character = preset.macro_character;
                self.last_applied_macros = [
                    preset.macro_tone,
                    preset.macro_space,
                    preset.macro_punch,
                    preset.macro_character,
                ];

                self.device_rack_state.selected_node_kind = Some(preset.target_node_kind.to_string());
                self.device_rack_state.device_name = preset.name.to_string();
                self.device_rack_state.cutoff = preset.macro_tone;
                self.device_rack_state.decay = preset.macro_space;
                self.device_rack_state.drive = preset.macro_punch;
                self.device_rack_state.mod_amt = preset.macro_character;
                self.device_rack_state.node_param_values.clear();
                for &(p_name, p_val) in preset.params {
                    self.device_rack_state.node_param_values.insert(p_name.to_string(), p_val);
                }

                self.inspector_state.selected_node_kind = Some(preset.target_node_kind.to_string());
                self.inspector_state.target_name = preset.name.to_string();
                self.inspector_state.node_param_values.clear();
                for &(p_name, p_val) in preset.params {
                    self.inspector_state.node_param_values.insert(p_name.to_string(), p_val);
                }

                if let Some(edo) = preset.tuning_edo {
                    self.inspector_state.scale_name = format!("{}-EDO Microtonal", edo);
                    self.top_bar_state.key_signature = format!("{}-EDO", edo);
                }

                if let Some(track) = self.tracks.get_mut(self.selected_track_idx) {
                    track.name = preset.name.to_string();
                }
            } else {
                let registry = crate::dsp_node_ui::DspNodeRegistry::new();
                let target_node = match self.last_applied_preset.as_str() {
                "Init Synth 1" => "AetherSynth",
                "Aether Warm Pad" => "AtmosphericPadSynth",
                "808 Sub Kick" => "CyberpunkSubSynth",
                "Vintage Tape Lead" => "TapeSaturation",
                "Karplus Acoustic Pluck" => "PluckSynth",
                "Neural Vocal Demucs" => "DemucsV4Separator",
                "Ambient Crystal Bells" => "CrystalResonator",
                "Lo-Fi Breakbeat" => "LoopSlicerNode",
                "Cathedral Pipe Organ" => "PipeOrganModel",
                "Cosmic Shockwave Reverb" => "IsmShockwaveReverb",
                "Neuro HRV Bio-Sync" => "HrvTempoSyncEngine",
                "Analog Tape Stop" => "TapeStop",
                "Orchestral Timpani Drum" => "PercussionMembrane",
                "Concert Rosewood Marimba" => "StruckIdiophoneResonator",
                "Bourbonnais Vielle Gurdy" => "HurdyGurdySoundboxBody",
                "Silk String Japanese Koto" => "SitarSoundboxBody",
                "Hindustani Ravi Sitar" => "JawariBridge",
                "Rhodes Classic Mark I" => "TineResonator",
                "Hurdy-Gurdy Crank Wheel" => "FrictionWheelExciter",
                "Concert Snare Drum Rattle" => "SnareRattleModel",
                "Indian Raga Chikari Drone" => "ChikariDroneBank",
                "Franklin Water Armonica" => "ArmonicaChassisResonator",
                "Concert Grand Felt Hammer" => "GrandPianoFeltHammer",
                "Sitka Spruce Resonance" => "SpruceSoundboardMode",
                "Hohner D6 Funk Clavinet" => "ClavinetVoice",
                "Indian Tarab Sympathetic" => "TarabStringResonator",
                "Neural AI Room Impulse" => "NeuralIrSynthesizer",
                "AI Voice-Leading Master" => "AiChordGenerator",
                "Quantum Hyperbolic Reverb" => "NonEuclideanHyperbolicReverb",
                "Neuro Relaxation Feedback" => "NeuroFeedbackOscillator",
                "Acoustic Cloaking Soundfield" => "AcousticCloakingSpatializer",
                "Sub-Harmonic Quantum Tunnel" => "SubharmonicQuantumTunnelingFilter",
                "EBU R128 Master Broadcast" => "EbuR128LoudnessMeter",
                "Dolby Atmos Bed Splitter" => "SurroundStemSplitterBedObject",
                "Procedural Raytraced Hall" => "ProceduralSpatialIrGenerator",
                "Polymetric Euclidean Groove" => "PolymetricSequencer",
                "Markov Generative Matrix" => "MarkovSequenceMutator",
                "3D Atmos Trajectory Orbit" => "SpatialAutomationEngine",
                "Concert Grand Piano Escapement" => "GrandPianoGestureEngine",
                "Bowed Violin Kinematics" => "BowingGestureEngine",
                "Concert Acoustic Harp Arpeggio" => "PlectrumBus",
                "Hurdy Gurdy Chien Drone" => "HurdyGurdyBus",
                "Vocal Tract Formant Shaper" => "FormantBus",
                "Electric Piano Tine Saturation" => "EpBus",
                "Karplus Plucked Nylon Resonator" => "KarplusStrongString",
                "Analog Reel-to-Reel Tape Saturation" => "TapeChannelState",
                "Shoebox Early Acoustic Reflections" => "ImageReflection",
                "MPE Polyphonic Gesture Router" => "MpeRouter",
                "Multitrack Ambient Soundscape Loop" => "MultitrackSessionLooper",
                "Harmonium Free-Reed Expression Swell" => "FreeReedVoice",
                "Glass Armonica Celestial Shimmer" => "ArmonicaChassisMode",
                "High-Gain WaveNet Tube Overdrive" => "NamWaveNetEngine",
                "Concert Hall Bowed Double Bass" => "BowedStringNode",
                "Tuned Orchestral Timpani Membrane" => "PercussionMembraneNode",
                "Vintage 91-Wheel Jazz Tonewheel Organ" => "TonewheelOrganNode",
                "Master Brickwall True-Peak Limiter" => "MasterLimiter",
                "Concert Grand Imperial Steinway" => "ConcertGrandPiano",
                "Stevie 70s Clavinet Wah Funk" => "Clavinet",
                "Plate Tank Mechanical Reverb" => "PlateTank",
                "Zen Bamboo Flute Breath" => "Shakuhachi",
                "Neo-Riemannian Cadence Graph" => "CadenceGraph",
                "Bjorklund Euclidean Rhythm Engine" => "GenerativeEngine",
                "Microtonal 31-EDO Harmonic Scale" => "EdoTuning",
                "Acoustic Flamenco Guitar Strummer" => "Strummer",
                "Just Intonation Concordance Lattice" => "ConcordanceLattice",
                "Bowed Cello Friction Orbit" => "FrictionOrbit",
                "Moog 4-Pole Ladder Self-Oscillation" => "LadderFilterBode",
                "Leslie 122 Dual Rotor Doppler" => "RotaryDoppler",
                "Neural High-Gain Amp Stack" => "NamModelConfig",
                "Spectrogram Visual Audio Canvas" => "SpectrogramSoundGenerator",
                "Cathedral Pipe Organ Windchest" => "WindchestReservoir",
                "Tactile 16-Pad Groove Machine" => "DrumMacroStrip",
                "Benjamin Franklin Glass Armonica" => "GlassArmonicaBus",
                "Multi-Zone SFZ Orchestra Sample Patch" => "SfzPresetPatch",
                "Lua Real-Time DSP Script Controller" => "LuaScriptEngine",
                "Zero-Latency Plugin Sandbox Guard" => "PluginSandbox",
                "Ultra-Low Latency Opus Audio Relay" => "OpusAudioRelay",
                "Real-Time Session Telemetry & Headroom HUD" => "SessionAnalyticsDashboard",
                "Interactive Lua DSP Profiler & Flamegraph" => "LuaProfiler",
                "Git DAG Branching & Undo Timeline HUD" => "GitSessionDag",
                "Live Session Matrix Clip Launcher" => "LiveClipMatrix",
                "AI Spectral Unmasking & Mix Balance HUD" => "AiMixBalanceInspector",
                "Hexagonal Wicki-Hayden Isomorphic Keyboard" => "IsomorphicKeyboardRouter",
                "Polymetric Euclidean Tracker Groove Matrix" => "PolymetricTrackerSequencer",
                "WASM & AudioWorklet Standalone Packager" => "WasmPackageExporter",
                "ITU-R BS.2076 ADM Dolby Atmos Spatial Exporter" => "AdmSpatialBwfExporter",
                "AI Harmonic Cadence & Leading Tone Suggester" => "AiHarmonySuggester",
                "Adaptive Automation Bézier Spline Thinning HUD" => "CurveThinningOptimizer",
                "Wolfram Rule 30 Cellular Rhythm Matrix" => "CellularAutomataRhythmGenerator",
                "ONNX Neural Diatonic Melody Composer" => "NeuralOnnxMelodyGenerator",
                "Demucs 4-Stem Deep Learning Splitter" => "DemucsNeuralStemSplitter",
                "Standalone CLAP Audio Plugin Bundler" => "ClapStandaloneExporter",
                "Privacy-Preserving Federated Mix Engine" => "FederatedMixLearner",
                "Sample-Accurate Multi-Take Studio Vocal Comp" => "MultiTakeCompManager",
                "Dynamic L1/L2 Cadence Tree Path Resolver" => "CadencePathResolver",
                "44-Cylinder Acoustic Vocal Tract Synthesizer" => "VowelCylinderAreaSynthesizer",
                "WebAssembly AudioWorklet Real-Time Bundle" => "AudioWorkletExporter",
                "Psychoacoustic Lipshitz Mastering Dither" => "DitherNoiseShaper",
                "Multi-Band Attack & Sustain Transient Sculptor" => "TransientShaperMatrix",
                "Spherical Harmonic 3D Binaural HRTF Soundfield" => "BinauralHrtfInterpolator",
                "Real-Time CRDT Multi-User Session" => "CrdtEngine",
                "Ultrasonic Phased-Array Levitation Soundfield" => "AcousticLevitationTrap",
                "Non-Linear Follow Action Matrix Sequencer" => "FollowAction",
                "Concert Grand Piano Escapement Trajectory" => "GrandPianoWaypoint",
                "Neural Amp Modeler WaveNet Studio Rig" => "NamModel",
                "Visual Spectrogram Image Sonification Canvas" => "SpectrogramArtConfig",
                "Multi-Track DAW Transport & Graph Engine" => "Transport",
                "Polyphonic Voice Pool & Micro-Timing Sequencer" => "VoicePool",
                "Live Session Clip Matrix & Scene Launcher" => "ClipSlot",
                "Vocal Multi-Take Comping & Crossfade Editor" => "CompLane",
                "Generative Markov Algorithmic Melodic Mutator" => "MarkovMutatorConfig",
                "Broadcast Multi-Stem Master Export Station" => "ExportPreset",
                "Concert Grand Piano Acoustic Resonance Model" => "GrandPianoBusSnapshot",
                "Cathedral Pipe Organ Tracker Wind Chest" => "PipeOrganBusSnapshot",
                "Hurdy-Gurdy Rosin Friction & Drone Resonator" => "HurdyGurdyBusSnapshot",
                "Continuous Human Breath & Wind Controller" => "BreathBusSnapshot",
                "Hurdy-Gurdy Continuous Crank & Buzzing Chien Gesture" => "HurdyGurdyWaypoint",
                "6-DOF Acoustic Articulation Bowing Dynamics Bus" => "ArticulationBusSnapshot",
                "Phonetic IPA Vowel Formant Trajectory Matrix" => "VowelNode",
                "Embedded Hardware Supervisor & Thermal Throttling" => "EmbeddedHardwareConfig",
                "Live Session Lossless WAV Disk Recorder Station" => "RecordingStats",
                "AI Multi-Track Frequency Collision & Masking Inspector" => "MaskingReport",
                "Acoustic Tonehole 3-Port Scattering Wave Matrix" => "ToneholeScatteringResult",
                "Quantum Bloch Sphere Density Matrix Tomography" => "QuantumTomographyData",
                "Neo-Riemannian Tonnetz Harmonic Matrix Synthesizer" => "HarmonicNode",
                "CLAP Sub-Sample Precision Automation & MPE Rig" => "ClapSampleAccurateAutomation",
                "Multi-Velocity Layered Acoustic Studio Drum Station" => "DrumPad",
                "3D Binaural Spatial Soundfield Vector Tracker" => "AtomicSpatialSlot",
                "Analog Subtractive Twin-Oscillator Performance Lead" => "AetherMacroView",
                "Quantum Wavepacket Coherence Spatial Interferometer" => "Complex32",
                "Dynamic Sidechain Matrix Ducking & Punch Controller" => "ControlEvent",
                "Neo-Riemannian Microtonal Modal Harmony Matrix" => "Scale",
                "Expressive Cello Continuous Bowing Physics" => "BowingGesturePattern",
                "Electric Piano Tine Hammer Dynamics" => "EpGesturePattern",
                "Japanese Bamboo Shakuhachi Breath & Meri" => "ShakuhachiGesturePattern",
                "Mechanical Spring Reverb Tank Perturbation" => "SpringGesturePattern",
                "Bellows Accordion Dual Reed Chamber Dynamics" => "BellowsGesturePattern",
                "Clavinet Funk Dual Pickup Tangent Strike" => "ClavinetGesturePattern",
                "Franklin Lead Crystal Armonica Rotational Friction" => "GlassGesturePattern",
                "Concert Grand Escapement Felt Hammer Action" => "GrandPianoGesturePattern",
                "Baroque Pipe Organ Tutti Plenum & Mixture Ranks" => "PipeOrganProfile",
                "Vintage EMT-140 Cold-Rolled Steel Suspension Reverb" => "PlateReverbProfile",
                "Concert Steinway D 9-Foot Resonant Soundboard" => "SoundboardProfile",
                "Classic B3 Tonewheel Organ Full Gospel Drawbars" => "TonewheelOrganProfile",
                "Scriptable Real-Time Lua Device & Macro Control Surface" => "MacroRackLuaDevice",
                "Stochastic Chaos Modulation & Algorithmic Random Generator" => "LuaRandomEngine",
                "Decentralized WebRTC Low-Latency Audio Peer Streamer" => "PeerNode",
                "Zero-Knowledge Cryptographic DSP Patch & State Verifier" => "ZkPatchProof",
                "Concert Flute 6-Tonehole Acoustic Resonant Waveguide" => "ToneholeGrid",
                "African Rosewood Concert Marimba Modal Resonator Bank" => "IdiophoneResonator",
                "Air-Gapped Zero-Cloud Sovereign Project Workspace" => "OfflineLocalProjectManager",
                "Crash-Proof Third-Party Audio Plugin Sandbox Shield" => "PluginBlacklist",
                "Imperial 97-Key Concert Grand Escapement & Duplex Resonance" => "GrandPianoArticulation",
                "Cathedral 64-Foot Mechanical Tracker Organ Pallet Action" => "PipeOrganArticulation",
                "Ancient Asian Paulownia Zither Movable Bone Ji Bridge" => "JiBridgeProfile",
                "Zero-Latency WebRTC High-Resolution MIDI 2.0 Network Jam" => "WebRtcMidiPacket",
                "Cremona 1715 Stradivarius Solo Violin & Golden Amber Rosin" => "BowedInstrumentModel",
                "Miles Dark Harmon Stem-Out Trumpet & Tractrix Brass Flare" => "HornAcousticMute",
                "Kyoto Imperial 13-String Paulownia Koto Hira-Joshi Suite" => "KotoTuningSchema",
                "Abbey Road Class-A 12AX7 Dual Triode Tube Console" => "VacuumTubeTopology",
                "Neural Lo-Fi Vinyl Audio Style Transfer & Saturation" => "AudioStylePreset",
                "Bourbonnais Vielle à Roue Wrist Accent Coup de Poignet" => "HurdyGurdyArticulation",
                "Hammond B3 3rd Fast Percussion & C3 Scanner Chorus Suite" => "PercussionHarmonic",
                "Kelly-Lochbaum Formant Vowel Kelly Acoustic Waveguide" => "VowelPreset",
                "Dolby Atmos 7.1.4 Multi-Channel Surround Immersion" => "ChannelLayout",
                "Hohner D6 Dual-Coil Parallel Out-of-Phase Funk Quack" => "ClavinetPickupSelection",
                "Multi-Topology Dynamic Wavefolder & Tanh Tube Saturation" => "DistortionType",
                "Mastering Psychoacoustic Noise-Shaped 24-Bit Dither" => "DitherType",
                "Modal Vibraphone & Marimba Wood Acoustic Resonator" => "ModalMaterialPreset",
                "Non-Linear SmoothStep Modulation Curve & Matrix Router" => "ModulationCurve",
                "AI Autonomous Mastering Target Pop & Club Spectral Curve" => "TargetCurve",
                "Deep Neural Audio Voice Morphing & Resynthesis Engine" => "TargetTimbre",
                "Dynamic Euclidean Ratchet Generative Pattern Engine" => "MutationStrategy",
                "Live Multitrack Performance Looper & Quantized Overdub" => "LooperCommand",
                "Polyphonic Chord Strumming Humanizer & Arpeggiator" => "StrumDirection",
                "Studio Mastering 32-Bit Float Multi-Stem Batch Exporter" => "StemExportFormat",
                "Aeroelastic Free-Reed Cassotto Chamber & Bellows Flow" => "FreeReedExciter",
                "Rosenberg Glottal Pulse Vocal Flow & Aspiration" => "GlottalFlowModel",
                "Asian Zither Paulownia Soundboard & Movable Ji Bridge" => "JiBridgeCoupling",
                "EMT 140 Cold-Rolled Steel Continuous Dispersion Plate" => "PlateReverbTank",
                "Parametric 3D Spatial Trajectory & Catmull-Rom Orbit" => "TrajectoryPathType",
                "Orchestral String Bowing Phrase Gesture & Friction Slew" => "GestureInterpolationCurve",
                "FastTracker Chromatic Note Pattern & Microtonal Detune" => "TrackerStepConfig",
                "Deterministic Master Audio Transport & Cycle Looper Clock" => "TransportConfig",
                "Master DAW Session Manifest & Audio Routing Topology" => "ProjectConfig",
                "Polymetric Clip Sequence Pattern & Tracker Step Array" => "SequenceConfig",
                "Enterprise Diagnostic Crash Dump & Subsystem Telemetry" => "CrashDump",
                "Distributed CRDT Collaborative Session & Multi-User Cursor" => "RemoteUserCursor",
                "Concert Grand Piano Steinway D-274 Voicing & Duplex Bleed" => "GrandPianoProfile",
                "Indian Classical Sitar Jawari Obstacle & Jiva Silk Sizzle" => "JawariProfile",
                "Procedural Lua Project Builder & Multi-Track Graph" => "LuaProjectBuilder",
                "Enterprise Audio Engine Health Telemetry & Latency Diagnostics" => "UserSessionMetric",
                "Physical Bowed Cello & Resonant Rosin Friction Slew" => "BowedInstrument",
                "North Indian Classical Raga Shruti Microtonal Drone" => "RagaScale",
                "Vintage 12AX7 Valve Triode Saturation & Sag Bloom" => "TubeTopology",
                "Opt-In Real-Time Audio Engine Health Telemetry & Diagnostics" => "TelemetryManager",
                "Headless Batch Project Render Farm & Offline Export Pipeline" => "CliBatchRenderEngine",
                "Demucs v4 Neural Stem Separation & Vocal Isolation Matrix" => "CliStemSeparatorEngine",
                "Real-Time OSC UDP Command Dispatcher & Control Protocol Bridge" => "OscServer",
                "ITU-R BS.2076 ADM 3D Spatial Audio & Dolby Atmos Broadcast Master" => "CliAdmSpatialExporter",
                "Deterministic Session Git DAG Commit & Visual Tree Diff" => "CliCommitHistoryViewer",
                "Automated C-ABI CLAP Standalone Audio Plugin Packager" => "CliClapPluginExporter",
                "Real-Time DSP Hardware Load Profiler & Thread Latency Monitor" => "CliDspProfiler",
                "Microtiming Groove Jitter Humanizer & Velocity Dynamism" => "CliGrooveHumanizer",
                "Universal Multi-Format Batch Audio Transcoding & Dither Studio" => "CliBatchAudioConverter",
                "Universal Open Standard DAWproject Cross-DAW Container Packager" => "CliDawprojectExporter",
                "Real-Time Audio Graph Throughput Benchmark & Latency Profiler" => "CliAudioBenchmarkRunner",
                "Static Lua DSP AST Security Guard & Sandbox Privilege Auditor" => "CliScriptSecurityAuditor",
                "General MIDI Acoustic Grand Piano" => "GrandPianoModel",
                "General MIDI Electric Piano 1 (Rhodes)" => "ElectricPianoModel",
                "General MIDI Church Pipe Organ" => "PipeOrganModel",
                "General MIDI Nylon String Guitar" => "PluckSynth",
                "General MIDI Overdriven Rock Guitar" => "DistortionNode",
                "General MIDI Acoustic Bass" => "BowedStringModel",
                "General MIDI Synth Brass 1" => "WaveguideBrassModel",
                "General MIDI Shakuhachi Flute" => "ShakuhachiModel",
                "General MIDI Standard Drum Kit 1" => "DrumMachineDevice",
                "Chiptune 8-Bit NES Pulse Lead" => "OscPulse",
                "Chiptune 8-Bit GameBoy Triangle Bass" => "OscTriangle",
                "Chiptune FastTracker II Arp Arpeggio" => "TrackerStepConfig",
                "Chiptune Noise Snare & Hi-Hat" => "NoiseGen",
                _ => "AetherSynth",
            };
            if let Some(desc) = registry.get(target_node) {
                self.device_rack_state.selected_node_kind = Some(desc.kind_id.clone());
                self.device_rack_state.device_name = desc.display_name.clone();
                self.inspector_state.selected_node_kind = Some(desc.kind_id.clone());
                self.inspector_state.target_name = desc.display_name.clone();
            }
        }
    }

        // 2. Synchronize Novice Macro Strip Knobs to Active Device Rack
        let cur_macros = [
            self.top_bar_state.macro_tone,
            self.top_bar_state.macro_space,
            self.top_bar_state.macro_punch,
            self.top_bar_state.macro_character,
        ];
        if cur_macros != self.last_applied_macros {
            self.last_applied_macros = cur_macros;
            self.device_rack_state.cutoff = cur_macros[0];
            self.device_rack_state.decay = cur_macros[1];
            self.device_rack_state.drive = cur_macros[2];
            self.device_rack_state.mod_amt = cur_macros[3];

            self.device_rack_state.node_param_values.insert("cutoff".to_string(), cur_macros[0]);
            self.device_rack_state.node_param_values.insert("decay".to_string(), cur_macros[1]);
            self.device_rack_state.node_param_values.insert("drive".to_string(), cur_macros[2]);
            self.device_rack_state.node_param_values.insert("character".to_string(), cur_macros[3]);
        }

        // Synchronize Two-Tier Mode between Top Bar and Device Rack / Inspector
        if self.top_bar_state.is_pro_mode {
            self.top_bar_state.is_novice_macro_visible = false;
            self.device_rack_state.is_minimized = false;
            self.device_rack_state.is_expanded_params = true;
            self.inspector_state.is_collapsed = false;
        } else {
            self.top_bar_state.is_novice_macro_visible = true;
            self.device_rack_state.is_expanded_params = false;
        }

        // Synchronize Master Gain from Novice / Top Bar if adjusted
        if (self.top_bar_state.master_gain - self.last_applied_master_gain).abs() > 0.001 {
            self.last_applied_master_gain = self.top_bar_state.master_gain;
            if let Some(track) = self.tracks.get_mut(self.selected_track_idx) {
                track.gain = self.top_bar_state.master_gain;
                self.inspector_state.gain_db = (track.gain - 1.0) * 12.0;
                self.last_inspector_gain_db = self.inspector_state.gain_db;
            }
        }

        // 3. Bidirectional Track <--> Inspector Synchronization
        if self.selected_track_idx != self.last_selected_track_idx {
            self.last_selected_track_idx = self.selected_track_idx;
            if let Some(track) = self.tracks.get(self.selected_track_idx) {
                self.inspector_state.target_name = track.name.clone();
                self.inspector_state.gain_db = (track.gain - 1.0) * 12.0;
                self.inspector_state.pan_val = track.pan;
                self.inspector_state.is_muted = track.is_muted;
                self.inspector_state.is_soloed = track.is_soloed;
                self.inspector_state.is_armed = track.is_armed;
                self.last_inspector_gain_db = self.inspector_state.gain_db;
                self.last_inspector_pan = self.inspector_state.pan_val;
                self.last_inspector_muted = self.inspector_state.is_muted;
                self.last_inspector_soloed = self.inspector_state.is_soloed;
                self.last_inspector_armed = self.inspector_state.is_armed;
            }
        } else if let Some(track) = self.tracks.get_mut(self.selected_track_idx) {
            if (self.inspector_state.gain_db - self.last_inspector_gain_db).abs() > 0.01 {
                track.gain = ((self.inspector_state.gain_db / 12.0) + 1.0).clamp(0.0, 2.0);
                self.last_inspector_gain_db = self.inspector_state.gain_db;
            } else if (track.gain - ((self.last_inspector_gain_db / 12.0) + 1.0)).abs() > 0.01 {
                self.inspector_state.gain_db = (track.gain - 1.0) * 12.0;
                self.last_inspector_gain_db = self.inspector_state.gain_db;
            }

            if (self.inspector_state.pan_val - self.last_inspector_pan).abs() > 0.01 {
                track.pan = self.inspector_state.pan_val;
                self.last_inspector_pan = self.inspector_state.pan_val;
            } else if (track.pan - self.last_inspector_pan).abs() > 0.01 {
                self.inspector_state.pan_val = track.pan;
                self.last_inspector_pan = track.pan;
            }

            if self.inspector_state.is_muted != self.last_inspector_muted {
                track.is_muted = self.inspector_state.is_muted;
                self.last_inspector_muted = self.inspector_state.is_muted;
            } else if track.is_muted != self.last_inspector_muted {
                self.inspector_state.is_muted = track.is_muted;
                self.last_inspector_muted = track.is_muted;
            }

            if self.inspector_state.is_soloed != self.last_inspector_soloed {
                track.is_soloed = self.inspector_state.is_soloed;
                self.last_inspector_soloed = self.inspector_state.is_soloed;
            } else if track.is_soloed != self.last_inspector_soloed {
                self.inspector_state.is_soloed = track.is_soloed;
                self.last_inspector_soloed = track.is_soloed;
            }

            if self.inspector_state.is_armed != self.last_inspector_armed {
                track.is_armed = self.inspector_state.is_armed;
                self.last_inspector_armed = self.inspector_state.is_armed;
            } else if track.is_armed != self.last_inspector_armed {
                self.inspector_state.is_armed = track.is_armed;
                self.last_inspector_armed = track.is_armed;
            }
        }

        // 4. Robust Bidirectional Synchronization across Modular Canvas, Rack & Inspector
        let rack_kind_changed = self.device_rack_state.selected_node_kind != self.last_device_rack_node_kind;
        let inspector_kind_changed = self.inspector_state.selected_node_kind != self.last_inspector_node_kind;

        if inspector_kind_changed && !rack_kind_changed {
            // User changed module in Inspector dropdown
            self.device_rack_state.selected_node_kind = self.inspector_state.selected_node_kind.clone();
            self.device_rack_state.device_name = self.inspector_state.target_name.clone();
        } else if rack_kind_changed {
            // User changed module in Device Rack dropdown
            self.inspector_state.selected_node_kind = self.device_rack_state.selected_node_kind.clone();
            self.inspector_state.target_name = self.device_rack_state.device_name.clone();
        }
        self.last_device_rack_node_kind = self.device_rack_state.selected_node_kind.clone();
        self.last_inspector_node_kind = self.inspector_state.selected_node_kind.clone();

        // Detect parameter changes made in Inspector and propagate to Device Rack
        for (k, v) in &self.inspector_state.node_param_values {
            if let Some(last_val) = self.last_inspector_node_param_values.get(k) {
                if (v - last_val).abs() > 1e-5 {
                    self.device_rack_state.node_param_values.insert(k.clone(), *v);
                }
            } else {
                self.device_rack_state.node_param_values.insert(k.clone(), *v);
            }
        }
        // Detect parameter changes made in Device Rack and propagate to Inspector
        for (k, v) in &self.device_rack_state.node_param_values {
            if let Some(last_val) = self.last_device_rack_node_param_values.get(k) {
                if (v - last_val).abs() > 1e-5 {
                    self.inspector_state.node_param_values.insert(k.clone(), *v);
                }
            } else {
                self.inspector_state.node_param_values.insert(k.clone(), *v);
            }
        }
        // Mirror any missing keys between both maps
        for (k, v) in &self.device_rack_state.node_param_values {
            self.inspector_state.node_param_values.entry(k.clone()).or_insert(*v);
        }
        for (k, v) in &self.inspector_state.node_param_values {
            self.device_rack_state.node_param_values.entry(k.clone()).or_insert(*v);
        }
        self.last_inspector_node_param_values = self.inspector_state.node_param_values.clone();
        self.last_device_rack_node_param_values = self.device_rack_state.node_param_values.clone();
        self.sync_selected_modular_node_params();

        ui.vertical(|ui| {
            // Zone 1: Top Bar
            show_modern_top_bar(ui, &mut self.top_bar_state, || {});

            // Main Workspace Frame (Zone 2: Left, Zone 3: Center Canvas + Bottom Rack, Zone 4: Right Inspector)
            ui.horizontal(|ui| {
                // Zone 2: Left Collapsible Asset Browser
                let mut selected_asset = None;
                show_modern_asset_browser(ui, &mut self.asset_browser_state, |drag_preset| {
                    selected_asset = Some(drag_preset.to_string());
                });
                if let Some(asset) = selected_asset {
                    if let Some(tmpl) = crate::factory_presets::find_demo_template(&asset) {
                        self.load_factory_demo_template(tmpl.id);
                    } else if !self.apply_factory_preset(&asset, None) {
                        let registry = crate::dsp_node_ui::DspNodeRegistry::new();
                        if let Some(desc) = registry.get(&asset) {
                            self.device_rack_state.selected_node_kind = Some(desc.kind_id.clone());
                            self.device_rack_state.device_name = desc.display_name.clone();
                            self.inspector_state.selected_node_kind = Some(desc.kind_id.clone());
                            self.inspector_state.target_name = desc.display_name.clone();
                        } else {
                            self.inspector_state.target_name = asset;
                        }
                    }
                }

                // Center Column (Dynamic Canvas + Bottom Dock)
                ui.vertical(|ui| {
                    match self.top_bar_state.active_tab {
                        crate::views::modern_top_bar::ModernViewTab::Arranger => {
                            self.show_arranger_canvas(ui);
                        }
                        crate::views::modern_top_bar::ModernViewTab::PianoRoll => {
                            self.show_piano_roll_canvas(ui);
                        }
                        crate::views::modern_top_bar::ModernViewTab::Modular => {
                            self.show_modular_canvas(ui);
                        }
                        crate::views::modern_top_bar::ModernViewTab::Mixer => {
                            self.show_mixer_canvas(ui);
                        }
                        crate::views::modern_top_bar::ModernViewTab::Performance => {
                            self.show_stage_canvas(ui);
                        }
                    }

                    // Zone 5: Bottom Dock (Reusable Device Rack)
                    if self.top_bar_state.active_tab != crate::views::modern_top_bar::ModernViewTab::Performance {
                        show_modern_device_rack(ui, &mut self.device_rack_state, self.current_oscilloscope_samples.as_deref());
                    }
                });

                // Zone 4: Right Collapsible Inspector
                if self.top_bar_state.active_tab != crate::views::modern_top_bar::ModernViewTab::Performance {
                    let cur_track_id = self.tracks.get(self.selected_track_idx).map(|t| t.id).unwrap_or(1);
                    show_modern_inspector_with_context(ui, &mut self.inspector_state, None, cur_track_id);
                }
            });
        });

        // Handle 1-click live automation requests from Inspector & Device Rack
        if let Some(param) = self.inspector_state.requested_automation_param.take() {
            let cur_track_id = self.tracks.get(self.selected_track_idx).map(|t| t.id).unwrap_or(1);
            self.open_track_automation_editor(cur_track_id, &param);
        }
        if let Some(param) = self.device_rack_state.requested_automation_param.take() {
            let cur_track_id = self.tracks.get(self.selected_track_idx).map(|t| t.id).unwrap_or(1);
            self.open_track_automation_editor(cur_track_id, &param);
        }

        // Global Modal Windows: Live Parameter Automation Editor & Modular DSP Catalog
        self.show_modular_automation_editor_window(ui);
        self.show_modular_dsp_catalog_window(ui);
    }

    #[cfg(feature = "gui")]
    fn show_arranger_canvas(&mut self, ui: &mut egui::Ui) {
        let cur_track_id = self.tracks.get(self.selected_track_idx).map(|t| t.id).unwrap_or(1);
        let cur_track_name = self.tracks.get(self.selected_track_idx).map(|t| t.name.clone()).unwrap_or_else(|| "Track 1".to_string());

        // 0. Arranger Pro Top Toolbar (Two-Tier: Novice vs Pro)
        ui.horizontal(|ui| {
            ui.label(RichText::new("📋 Arranger").font(FontId::proportional(12.0)).strong().color(Color32::from_rgb(241, 245, 249)));
            ui.add_space(6.0);

            // Track Selector ComboBox
            let mut track_to_select = None;
            egui::ComboBox::from_id_source("arranger_track_selector")
                .selected_text(RichText::new(format!("🎚 {}", cur_track_name)).font(FontId::proportional(10.5)).color(Color32::from_rgb(56, 189, 248)))
                .show_ui(ui, |ui| {
                    for (t_idx, tr) in self.tracks.iter().enumerate() {
                        let is_sel = t_idx == self.selected_track_idx;
                        if ui.selectable_label(is_sel, format!("{}: {}", tr.id, tr.name)).clicked() {
                            track_to_select = Some(t_idx);
                        }
                    }
                });
            if let Some(t_idx) = track_to_select {
                self.select_track_for_arranger(t_idx);
            }

            ui.add_space(4.0);

            // 1-Click Pro View Switchers
            if ui.button(RichText::new("🎹 Piano Roll").font(FontId::proportional(10.0)).color(Color32::from_rgb(200, 215, 235))).clicked() {
                self.top_bar_state.active_tab = crate::views::modern_top_bar::ModernViewTab::PianoRoll;
            }
            if ui.button(RichText::new("∿ Modular").font(FontId::proportional(10.0)).color(Color32::from_rgb(200, 215, 235))).clicked() {
                self.top_bar_state.active_tab = crate::views::modern_top_bar::ModernViewTab::Modular;
            }
            if ui.button(RichText::new("🎚 Mixer").font(FontId::proportional(10.0)).color(Color32::from_rgb(200, 215, 235))).clicked() {
                self.top_bar_state.active_tab = crate::views::modern_top_bar::ModernViewTab::Mixer;
            }
            if ui.button(RichText::new("🎭 Stage").font(FontId::proportional(10.0)).color(Color32::from_rgb(200, 215, 235))).clicked() {
                self.top_bar_state.active_tab = crate::views::modern_top_bar::ModernViewTab::Performance;
            }

            ui.add_space(4.0);

            // 1-Click Live Parameter Automation Launcher
            egui::ComboBox::from_id_source("arranger_auto_selector")
                .selected_text(RichText::new("📈 Auto ▾").font(FontId::proportional(10.0)).color(Color32::from_rgb(234, 179, 8)))
                .show_ui(ui, |ui| {
                    if ui.selectable_label(false, "📈 Gain (Volume)").clicked() {
                        self.open_track_automation_editor(cur_track_id, "gain");
                    }
                    if ui.selectable_label(false, "📈 Stereo Pan").clicked() {
                        self.open_track_automation_editor(cur_track_id, "pan");
                    }
                    if ui.selectable_label(false, "📈 Mute Gate").clicked() {
                        self.open_track_automation_editor(cur_track_id, "mute");
                    }
                    if ui.selectable_label(false, "📈 Solo Audition").clicked() {
                        self.open_track_automation_editor(cur_track_id, "solo");
                    }
                    if ui.selectable_label(false, "📈 Filter Cutoff").clicked() {
                        self.open_track_automation_editor(cur_track_id, "cutoff");
                    }
                    if ui.selectable_label(false, "📈 Resonance").clicked() {
                        self.open_track_automation_editor(cur_track_id, "resonance");
                    }
                    if ui.selectable_label(false, "📈 Decay Time").clicked() {
                        self.open_track_automation_editor(cur_track_id, "decay");
                    }
                    if ui.selectable_label(false, "📈 Drive / Saturation").clicked() {
                        self.open_track_automation_editor(cur_track_id, "drive");
                    }
                });

            ui.add_space(4.0);

            // Snap Grid Selector
            egui::ComboBox::from_id_source("arranger_snap_grid_selector")
                .selected_text(RichText::new(format!("🧲 {}", self.arranger_snap_grid.name())).font(FontId::proportional(10.0)).color(Color32::from_rgb(168, 85, 247)))
                .show_ui(ui, |ui| {
                    for res in [
                        ArrangerSnapResolution::Bar1,
                        ArrangerSnapResolution::BeatQuarter,
                        ArrangerSnapResolution::BeatEighth,
                        ArrangerSnapResolution::BeatSixteenth,
                        ArrangerSnapResolution::Free,
                    ] {
                        if ui.selectable_label(self.arranger_snap_grid == res, res.name()).clicked() {
                            self.arranger_snap_grid = res;
                        }
                    }
                });

            ui.add_space(4.0);

            // Clip Operations Toolbar
            let has_clip = self.selected_clip.is_some();
            let clip_op_col = if has_clip { Color32::from_rgb(241, 245, 249) } else { Color32::from_rgb(100, 116, 139) };

            if ui.add_enabled(has_clip, egui::Button::new(RichText::new("⇥ Quantize").font(FontId::proportional(9.5)).color(clip_op_col)))
                .on_hover_text("Quantize selected clip to active snap grid (Q)")
                .clicked()
            {
                self.quantize_selected_clip();
            }

            if ui.add_enabled(has_clip, egui::Button::new(RichText::new("⎘ Duplicate").font(FontId::proportional(9.5)).color(clip_op_col)))
                .on_hover_text("Duplicate selected clip (Ctrl+D)")
                .clicked()
            {
                self.duplicate_selected_clip();
            }

            if ui.add_enabled(has_clip, egui::Button::new(RichText::new("✂ Split").font(FontId::proportional(9.5)).color(clip_op_col)))
                .on_hover_text("Split selected clip at playhead (S)")
                .clicked()
            {
                self.split_selected_clip_at_playhead();
            }

            if ui.add_enabled(has_clip, egui::Button::new(RichText::new("🔁 Loop").font(FontId::proportional(9.5)).color(clip_op_col)))
                .on_hover_text("Set loop bracket to selected clip bounds (L)")
                .clicked()
            {
                self.set_loop_to_selected_clip();
            }

            if ui.add_enabled(has_clip, egui::Button::new(RichText::new("🗑").font(FontId::proportional(9.5)).color(if has_clip { Color32::from_rgb(239, 68, 68) } else { Color32::from_rgb(100, 116, 139) })))
                .on_hover_text("Delete selected clip (Del)")
                .clicked()
            {
                self.delete_selected_clip();
            }
        });
        ui.add_space(4.0);

        let canvas_height = (self.tracks.len() as f32 * 36.0 + 36.0).max(280.0);
        egui::Frame::none()
            .fill(Color32::from_rgb(10, 14, 24))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(28, 40, 60)))
            .rounding(Rounding::same(6.0))
            .show(ui, |ui| {
                ui.set_height(canvas_height);
                let (resp, painter) = ui.allocate_painter(Vec2::new(ui.available_width(), canvas_height), egui::Sense::click_and_drag());
                let rect = resp.rect;

                let is_pro = self.top_bar_state.is_pro_mode;
                let header_w = if is_pro { 202.0 } else { 140.0 };
                let track_area_w = (rect.width() - header_w).max(200.0);
                let ppb = track_area_w / 18.0; // 18 measures visible

                // 1. Timeline Header Ruler (Measures 1..17)
                let ruler_h = 28.0;
                let ruler_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), ruler_h));
                painter.rect_filled(ruler_rect, 0.0, Color32::from_rgb(14, 20, 32));
                painter.line_segment([ruler_rect.left_bottom(), ruler_rect.right_bottom()], Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 74)));

                // Loop Bracket
                let loop_x1 = rect.left() + header_w + self.loop_start_beat * ppb;
                let loop_x2 = rect.left() + header_w + self.loop_end_beat * ppb;
                let loop_rect = Rect::from_min_max(egui::pos2(loop_x1, ruler_rect.top() + 2.0), egui::pos2(loop_x2, ruler_rect.bottom() - 2.0));
                painter.rect_filled(loop_rect, 2.0, Color32::from_rgba_unmultiplied(56, 189, 248, 40));
                painter.line_segment([loop_rect.left_top(), loop_rect.left_bottom()], Stroke::new(2.0_f32, Color32::from_rgb(56, 189, 248)));
                painter.line_segment([loop_rect.right_top(), loop_rect.right_bottom()], Stroke::new(2.0_f32, Color32::from_rgb(56, 189, 248)));

                for bar in 1..=17 {
                    let bar_x = rect.left() + header_w + ((bar - 1) as f32 * 4.0 * ppb);
                    painter.line_segment([egui::pos2(bar_x, ruler_rect.top()), egui::pos2(bar_x, ruler_rect.bottom())], Stroke::new(1.0_f32, Color32::from_rgb(50, 65, 90)));
                    painter.text(egui::pos2(bar_x + 4.0, ruler_rect.top() + 6.0), egui::Align2::LEFT_TOP, format!("{}", bar), FontId::proportional(10.0), Color32::from_rgb(148, 163, 184));
                }

                // Arranger Tools Selector (Pointer / Slice / Fade)
                let tool_btn_w = 40.0;
                let tool_btn_h = 20.0;
                let t_pointer = Rect::from_min_size(egui::pos2(rect.left() + 4.0, ruler_rect.top() + 4.0), Vec2::new(tool_btn_w, tool_btn_h));
                let t_slice = Rect::from_min_size(egui::pos2(rect.left() + 48.0, ruler_rect.top() + 4.0), Vec2::new(tool_btn_w, tool_btn_h));
                let t_fade = Rect::from_min_size(egui::pos2(rect.left() + 92.0, ruler_rect.top() + 4.0), Vec2::new(tool_btn_w + 2.0, tool_btn_h));

                let p_bg = if self.arranger_tool_mode == ArrangerToolMode::Pointer { Color32::from_rgb(26, 140, 255) } else { Color32::from_rgb(20, 28, 44) };
                let s_bg = if self.arranger_tool_mode == ArrangerToolMode::Slice { Color32::from_rgb(236, 72, 153) } else { Color32::from_rgb(20, 28, 44) };
                let f_bg = if self.arranger_tool_mode == ArrangerToolMode::Fade { Color32::from_rgb(16, 185, 129) } else { Color32::from_rgb(20, 28, 44) };

                painter.rect_filled(t_pointer, 3.0, p_bg);
                painter.text(t_pointer.center(), egui::Align2::CENTER_CENTER, "↖ Sel", FontId::proportional(9.0), Color32::WHITE);

                painter.rect_filled(t_slice, 3.0, s_bg);
                painter.text(t_slice.center(), egui::Align2::CENTER_CENTER, "✂ Cut", FontId::proportional(9.0), Color32::WHITE);

                painter.rect_filled(t_fade, 3.0, f_bg);
                painter.text(t_fade.center(), egui::Align2::CENTER_CENTER, "◿ Fade", FontId::proportional(9.0), Color32::WHITE);

                let s_key_pressed = ui.input(|i| i.key_pressed(egui::Key::S) && !i.modifiers.ctrl);

                // Interaction handling (Scrubbing playhead, selecting track, adjusting track gain)
                let pointer_pos = resp.interact_pointer_pos().or_else(|| ui.input(|i| i.pointer.latest_pos()));
                let is_interacting = resp.clicked() || resp.dragged() || ui.input(|i| i.pointer.primary_down() || i.pointer.primary_clicked());
                let is_click = resp.clicked() || ui.input(|i| i.pointer.primary_clicked() || (i.pointer.primary_down() && !resp.dragged()));
                let is_double = resp.double_clicked() || ui.input(|i| i.pointer.button_double_clicked(egui::PointerButton::Primary));

                if let Some(pos) = pointer_pos {
                    if is_click {
                        if t_pointer.contains(pos) { self.arranger_tool_mode = ArrangerToolMode::Pointer; }
                        else if t_slice.contains(pos) { self.arranger_tool_mode = ArrangerToolMode::Slice; }
                        else if t_fade.contains(pos) { self.arranger_tool_mode = ArrangerToolMode::Fade; }
                    }

                    if is_interacting {
                        if pos.y <= rect.top() + ruler_h && pos.x >= rect.left() + header_w {
                            let beat = ((pos.x - (rect.left() + header_w)) / ppb).max(0.0);
                            self.playhead_beat = beat;
                        } else if pos.y > rect.top() + ruler_h {
                            let mut selected_idx = None;
                            let mut track_to_open_auto = None;
                            let row_h = 32.0;
                            for (idx, track) in self.tracks.iter_mut().enumerate() {
                                let row_top = rect.top() + ruler_h + (idx as f32 * (row_h + 2.0));
                                let head_rect = Rect::from_min_size(egui::pos2(rect.left(), row_top), Vec2::new(header_w, row_h));
                                let lane_rect = Rect::from_min_size(egui::pos2(rect.left() + header_w, row_top), Vec2::new(track_area_w, row_h));
                                let pill_rect = if is_pro {
                                    Rect::from_min_size(egui::pos2(head_rect.left() + 124.0, head_rect.center().y - 4.0), Vec2::new(24.0, 8.0))
                                } else {
                                    Rect::from_min_size(egui::pos2(head_rect.left() + 72.0, head_rect.center().y - 4.0), Vec2::new(45.0, 8.0))
                                };

                                let m_rect = Rect::from_min_size(egui::pos2(head_rect.left() + 70.0, head_rect.center().y - 8.0), Vec2::new(16.0, 16.0));
                                let s_rect = Rect::from_min_size(egui::pos2(head_rect.left() + 88.0, head_rect.center().y - 8.0), Vec2::new(16.0, 16.0));
                                let a_rect = Rect::from_min_size(egui::pos2(head_rect.left() + 106.0, head_rect.center().y - 8.0), Vec2::new(16.0, 16.0));
                                let p_rect = Rect::from_min_size(egui::pos2(head_rect.left() + 152.0, head_rect.center().y - 7.0), Vec2::new(10.0, 14.0));
                                let mod_rect = Rect::from_min_size(egui::pos2(head_rect.left() + 164.0, head_rect.center().y - 7.0), Vec2::new(10.0, 14.0));
                                let mix_rect = Rect::from_min_size(egui::pos2(head_rect.left() + 176.0, head_rect.center().y - 7.0), Vec2::new(10.0, 14.0));
                                let auto_rect = Rect::from_min_size(egui::pos2(head_rect.left() + 188.0, head_rect.center().y - 7.0), Vec2::new(10.0, 14.0));

                                if is_pro && m_rect.contains(pos) && is_click {
                                    track.is_muted = !track.is_muted;
                                    selected_idx = Some(idx);
                                } else if is_pro && s_rect.contains(pos) && is_click {
                                    track.is_soloed = !track.is_soloed;
                                    selected_idx = Some(idx);
                                } else if is_pro && a_rect.contains(pos) && is_click {
                                    track.is_armed = !track.is_armed;
                                    selected_idx = Some(idx);
                                } else if is_pro && p_rect.contains(pos) && is_click {
                                    selected_idx = Some(idx);
                                    self.selected_track_idx = idx;
                                    self.top_bar_state.active_tab = crate::views::modern_top_bar::ModernViewTab::PianoRoll;
                                } else if is_pro && mod_rect.contains(pos) && is_click {
                                    selected_idx = Some(idx);
                                    self.selected_track_idx = idx;
                                    self.top_bar_state.active_tab = crate::views::modern_top_bar::ModernViewTab::Modular;
                                } else if is_pro && mix_rect.contains(pos) && is_click {
                                    selected_idx = Some(idx);
                                    self.selected_track_idx = idx;
                                    self.top_bar_state.active_tab = crate::views::modern_top_bar::ModernViewTab::Mixer;
                                } else if is_pro && auto_rect.contains(pos) && is_click {
                                    selected_idx = Some(idx);
                                    self.selected_track_idx = idx;
                                    track_to_open_auto = Some(track.id);
                                } else if pill_rect.expand(4.0).contains(pos) {
                                    if is_double {
                                        track.gain = 1.0;
                                    } else {
                                        let new_gain = ((pos.x - pill_rect.left()) / pill_rect.width() * 1.5).clamp(0.0, 1.5);
                                        track.gain = new_gain;
                                    }
                                    selected_idx = Some(idx);
                                } else if (head_rect.contains(pos) || lane_rect.contains(pos)) && is_click {
                                    selected_idx = Some(idx);
                                }
                            }
                            if let Some(s_idx) = selected_idx {
                                self.select_track_for_arranger(s_idx);
                            }
                            if let Some(tid) = track_to_open_auto {
                                self.open_track_automation_editor(tid, "gain");
                            }
                        }
                    }
                }

                // 2. Track Lanes
                let row_h = 32.0;
                let mut split_action: Option<(usize, usize, f32)> = None;
                let mut clip_to_select: Option<(usize, usize)> = None;
                let mut clip_double_clicked: Option<(usize, usize, bool)> = None;
                let mut clip_drag_action: Option<(usize, usize, f32)> = None;

                for (idx, track) in self.tracks.iter_mut().enumerate() {
                    track.ensure_clips();
                    let row_top = rect.top() + ruler_h + (idx as f32 * (row_h + 2.0));
                    let is_sel = idx == self.selected_track_idx;

                    // Track Header
                    let head_rect = Rect::from_min_size(egui::pos2(rect.left(), row_top), Vec2::new(header_w, row_h));
                    let head_bg = if is_sel { Color32::from_rgb(20, 28, 44) } else { Color32::from_rgb(14, 18, 28) };
                    painter.rect_filled(head_rect, 2.0, head_bg);
                    painter.rect_stroke(head_rect, 2.0, Stroke::new(1.0_f32, Color32::from_rgb(28, 38, 56)));

                    // Track Index & Name
                    let col = Color32::from_rgb(track.color_rgb[0], track.color_rgb[1], track.color_rgb[2]);
                    painter.text(egui::pos2(head_rect.left() + 4.0, head_rect.center().y), egui::Align2::LEFT_CENTER, format!("{}", idx + 1), FontId::proportional(10.0), Color32::from_rgb(100, 116, 139));
                    let name_str = if is_pro && track.name.len() > 6 {
                        format!("{}..", &track.name[..5])
                    } else {
                        track.name.clone()
                    };
                    painter.text(egui::pos2(head_rect.left() + 18.0, head_rect.center().y), egui::Align2::LEFT_CENTER, &name_str, FontId::proportional(10.5), Color32::from_rgb(241, 245, 249));

                    if is_pro {
                        let m_rect = Rect::from_min_size(egui::pos2(head_rect.left() + 70.0, head_rect.center().y - 8.0), Vec2::new(16.0, 16.0));
                        let s_rect = Rect::from_min_size(egui::pos2(head_rect.left() + 88.0, head_rect.center().y - 8.0), Vec2::new(16.0, 16.0));
                        let a_rect = Rect::from_min_size(egui::pos2(head_rect.left() + 106.0, head_rect.center().y - 8.0), Vec2::new(16.0, 16.0));
                        let p_rect = Rect::from_min_size(egui::pos2(head_rect.left() + 152.0, head_rect.center().y - 7.0), Vec2::new(10.0, 14.0));
                        let mod_rect = Rect::from_min_size(egui::pos2(head_rect.left() + 164.0, head_rect.center().y - 7.0), Vec2::new(10.0, 14.0));
                        let mix_rect = Rect::from_min_size(egui::pos2(head_rect.left() + 176.0, head_rect.center().y - 7.0), Vec2::new(10.0, 14.0));
                        let auto_rect = Rect::from_min_size(egui::pos2(head_rect.left() + 188.0, head_rect.center().y - 7.0), Vec2::new(10.0, 14.0));

                        // Mute button
                        let m_bg = if track.is_muted { Color32::from_rgb(239, 68, 68) } else { Color32::from_rgb(24, 34, 52) };
                        painter.rect_filled(m_rect, 2.0, m_bg);
                        painter.text(m_rect.center(), egui::Align2::CENTER_CENTER, "M", FontId::proportional(8.5), Color32::WHITE);

                        // Solo button
                        let s_bg = if track.is_soloed { Color32::from_rgb(234, 179, 8) } else { Color32::from_rgb(24, 34, 52) };
                        painter.rect_filled(s_rect, 2.0, s_bg);
                        painter.text(s_rect.center(), egui::Align2::CENTER_CENTER, "S", FontId::proportional(8.5), Color32::WHITE);

                        // Arm button
                        let a_bg = if track.is_armed { Color32::from_rgb(220, 38, 38) } else { Color32::from_rgb(24, 34, 52) };
                        painter.rect_filled(a_rect, 2.0, a_bg);
                        painter.text(a_rect.center(), egui::Align2::CENTER_CENTER, "●", FontId::proportional(8.5), if track.is_armed { Color32::WHITE } else { Color32::from_rgb(148, 163, 184) });

                        // 1-Click Pro View Launchers [🎹] [∿] [🎚] [📈]
                        painter.rect_filled(p_rect, 2.0, Color32::from_rgb(18, 26, 42));
                        painter.text(p_rect.center(), egui::Align2::CENTER_CENTER, "🎹", FontId::proportional(7.5), Color32::from_rgb(168, 85, 247));

                        painter.rect_filled(mod_rect, 2.0, Color32::from_rgb(18, 26, 42));
                        painter.text(mod_rect.center(), egui::Align2::CENTER_CENTER, "∿", FontId::proportional(8.0), Color32::from_rgb(56, 189, 248));

                        painter.rect_filled(mix_rect, 2.0, Color32::from_rgb(18, 26, 42));
                        painter.text(mix_rect.center(), egui::Align2::CENTER_CENTER, "🎚", FontId::proportional(7.5), Color32::from_rgb(34, 197, 94));

                        painter.rect_filled(auto_rect, 2.0, Color32::from_rgb(18, 26, 42));
                        painter.text(auto_rect.center(), egui::Align2::CENTER_CENTER, "📈", FontId::proportional(7.0), Color32::from_rgb(245, 158, 11));
                    }

                    // Colored Volume Slider Pill
                    let pill_rect = if is_pro {
                        Rect::from_min_size(egui::pos2(head_rect.left() + 124.0, head_rect.center().y - 4.0), Vec2::new(24.0, 8.0))
                    } else {
                        Rect::from_min_size(egui::pos2(head_rect.left() + 72.0, head_rect.center().y - 4.0), Vec2::new(45.0, 8.0))
                    };
                    painter.rect_filled(pill_rect, 4.0, Color32::from_rgb(8, 12, 20));
                    let fill_w = pill_rect.width() * (track.gain / 1.5).clamp(0.0, 1.0);
                    painter.rect_filled(Rect::from_min_size(pill_rect.min, Vec2::new(fill_w, pill_rect.height())), 4.0, col);

                    // Track Timeline Lane
                    let lane_rect = Rect::from_min_size(egui::pos2(rect.left() + header_w, row_top), Vec2::new(track_area_w, row_h));
                    let lane_bg = if is_sel { Color32::from_rgb(16, 24, 38) } else { Color32::from_rgb(10, 14, 22) };
                    painter.rect_filled(lane_rect, 2.0, lane_bg);

                    // Bar divider lines across lane
                    for bar in 1..=17 {
                        let bar_x = rect.left() + header_w + ((bar - 1) as f32 * 4.0 * ppb);
                        painter.line_segment([egui::pos2(bar_x, lane_rect.top()), egui::pos2(bar_x, lane_rect.bottom())], Stroke::new(0.5_f32, Color32::from_rgb(24, 32, 48)));
                    }

                    // Render Clips on this track
                    for clip in &mut track.clips {
                        let clip_x = rect.left() + header_w + (clip.start_beat * ppb);
                        let clip_w = (clip.length_beats * ppb).max(18.0);
                        let clip_rect = Rect::from_min_size(egui::pos2(clip_x, lane_rect.top() + 2.0), Vec2::new(clip_w, lane_rect.height() - 4.0));

                        let is_this_clip_sel = self.selected_clip == Some((idx, clip.id)) || clip.is_selected;

                        // Slicing, selection, double-click, and dragging interactions
                        if let Some(pos) = pointer_pos {
                            if clip_rect.contains(pos) {
                                if (self.arranger_tool_mode == ArrangerToolMode::Slice || ui.input(|i| i.modifiers.ctrl)) && is_click {
                                    let click_beat = ((pos.x - (rect.left() + header_w)) / ppb).max(0.0);
                                    split_action = Some((idx, clip.id, click_beat));
                                } else if s_key_pressed {
                                    split_action = Some((idx, clip.id, self.playhead_beat));
                                } else if is_double {
                                    clip_double_clicked = Some((idx, clip.id, track.is_audio));
                                } else if is_click {
                                    clip_to_select = Some((idx, clip.id));
                                }

                                // Draggable Fade handles interaction
                                let fade_in_h = egui::pos2(clip_rect.left() + (clip.fade_in * ppb).min(clip_rect.width() * 0.5), clip_rect.top() + 4.0);
                                let fade_out_h = egui::pos2(clip_rect.right() - (clip.fade_out * ppb).min(clip_rect.width() * 0.5), clip_rect.top() + 4.0);

                                if pos.distance(fade_in_h) < 10.0 && resp.dragged() {
                                    let delta_b = resp.drag_delta().x / ppb;
                                    clip.fade_in = (clip.fade_in + delta_b).clamp(0.0, clip.length_beats * 0.5);
                                } else if pos.distance(fade_out_h) < 10.0 && resp.dragged() {
                                    let delta_b = -resp.drag_delta().x / ppb;
                                    clip.fade_out = (clip.fade_out + delta_b).clamp(0.0, clip.length_beats * 0.5);
                                } else if self.arranger_tool_mode == ArrangerToolMode::Pointer && resp.dragged() {
                                    let delta_b = resp.drag_delta().x / ppb;
                                    clip_drag_action = Some((idx, clip.id, delta_b));
                                }
                            }
                        }

                        // Clip fill with soft color glow & selected highlight
                        let fill_alpha = if is_this_clip_sel { 70 } else { 40 };
                        painter.rect_filled(clip_rect, 4.0, Color32::from_rgba_unmultiplied(track.color_rgb[0], track.color_rgb[1], track.color_rgb[2], fill_alpha));
                        let stroke_col = if is_this_clip_sel { Color32::from_rgb(250, 204, 21) } else { col };
                        let stroke_w = if is_this_clip_sel { 2.0_f32 } else { 1.2_f32 };
                        painter.rect_stroke(clip_rect, 4.0, Stroke::new(stroke_w, stroke_col));

                        if is_this_clip_sel {
                            let sel_badge = Rect::from_min_size(egui::pos2(clip_rect.right() - 28.0, clip_rect.top() + 3.0), Vec2::new(24.0, 11.0));
                            painter.rect_filled(sel_badge, 2.0, Color32::from_rgb(250, 204, 21));
                            painter.text(sel_badge.center(), egui::Align2::CENTER_CENTER, "SEL", FontId::proportional(8.0), Color32::BLACK);
                        }

                        // Fade In Polygon & Curve
                        if clip.fade_in > 0.0 {
                            let fade_w = (clip.fade_in * ppb).min(clip_rect.width() * 0.5);
                            let pts = vec![
                                clip_rect.left_bottom(),
                                clip_rect.left_top(),
                                egui::pos2(clip_rect.left() + fade_w, clip_rect.top()),
                            ];
                            painter.add(egui::Shape::convex_polygon(
                                pts,
                                Color32::from_rgba_unmultiplied(255, 255, 255, 30),
                                Stroke::new(1.0_f32, Color32::WHITE),
                            ));
                        }

                        // Fade Out Polygon & Curve
                        if clip.fade_out > 0.0 {
                            let fade_w = (clip.fade_out * ppb).min(clip_rect.width() * 0.5);
                            let pts = vec![
                                egui::pos2(clip_rect.right() - fade_w, clip_rect.top()),
                                clip_rect.right_top(),
                                clip_rect.right_bottom(),
                            ];
                            painter.add(egui::Shape::convex_polygon(
                                pts,
                                Color32::from_rgba_unmultiplied(255, 255, 255, 30),
                                Stroke::new(1.0_f32, Color32::WHITE),
                            ));
                        }

                        // Fade Handle points
                        let fade_in_pos = egui::pos2(clip_rect.left() + (clip.fade_in * ppb).min(clip_rect.width() * 0.5), clip_rect.top() + 4.0);
                        let fade_out_pos = egui::pos2(clip_rect.right() - (clip.fade_out * ppb).min(clip_rect.width() * 0.5), clip_rect.top() + 4.0);
                        painter.circle_filled(fade_in_pos, 2.5, Color32::WHITE);
                        painter.circle_filled(fade_out_pos, 2.5, Color32::WHITE);

                        // Clip content (waveform spikes or MIDI note blocks)
                        if track.is_audio {
                            let num_spikes = (clip_rect.width() / 4.0) as usize;
                            for s in 0..num_spikes {
                                let sx = clip_rect.left() + s as f32 * 4.0 + 2.0;
                                let norm = s as f32 / num_spikes.max(1) as f32;
                                let amp = (norm * 12.0 * std::f32::consts::PI).sin().abs() * 0.70 + 0.15;
                                let h = (clip_rect.height() - 6.0) * amp;
                                let sy1 = clip_rect.center().y - h * 0.5;
                                let sy2 = clip_rect.center().y + h * 0.5;
                                painter.line_segment([egui::pos2(sx, sy1), egui::pos2(sx, sy2)], Stroke::new(1.8_f32, col));
                            }
                        } else {
                            let num_steps = 12;
                            for step in 0..num_steps {
                                let st = step as f32 / num_steps as f32;
                                let nx = clip_rect.left() + st * (clip_rect.width() - 14.0) + 4.0;
                                let note_y = clip_rect.top() + 6.0 + (step % 4) as f32 * 4.0;
                                let note_rect = Rect::from_min_size(egui::pos2(nx, note_y), Vec2::new(10.0, 3.0));
                                painter.rect_filled(note_rect, 1.0, col);
                            }
                        }

                        // Clip Name Label
                        painter.text(
                            egui::pos2(clip_rect.left() + 6.0, clip_rect.top() + 3.0),
                            egui::Align2::LEFT_TOP,
                            &clip.name,
                            FontId::proportional(10.0),
                            Color32::from_rgb(241, 245, 249),
                        );
                    }

                    // Crossfades between overlapping clips on this track
                    for (o_start, o_end, _c1, _c2) in track.detect_crossfades() {
                        let xf_x1 = rect.left() + header_w + (o_start * ppb);
                        let xf_x2 = rect.left() + header_w + (o_end * ppb);
                        let xf_rect = Rect::from_min_max(egui::pos2(xf_x1, lane_rect.top() + 2.0), egui::pos2(xf_x2, lane_rect.bottom() - 2.0));
                        painter.rect_filled(xf_rect, 2.0, Color32::from_rgba_unmultiplied(16, 185, 129, 60));
                        painter.line_segment([xf_rect.left_top(), xf_rect.right_bottom()], Stroke::new(1.2_f32, Color32::from_rgb(16, 185, 129)));
                        painter.line_segment([xf_rect.left_bottom(), xf_rect.right_top()], Stroke::new(1.2_f32, Color32::from_rgb(16, 185, 129)));
                        if xf_rect.width() > 24.0 {
                            painter.text(xf_rect.center(), egui::Align2::CENTER_CENTER, format!("XFade {:.1}b", o_end - o_start), FontId::proportional(8.5), Color32::from_rgb(200, 255, 235));
                        }
                    }
                }

                // Apply split action if triggered
                if let Some((t_idx, c_id, split_beat)) = split_action {
                    if let Some(track) = self.tracks.get_mut(t_idx) {
                        track.split_clip_at_beat(c_id, split_beat);
                    }
                }

                // Apply clip selection, double-click, and drag movement
                if let Some((t_idx, c_id)) = clip_to_select {
                    self.select_clip(t_idx, c_id);
                }
                if let Some((t_idx, c_id, is_audio)) = clip_double_clicked {
                    self.select_clip(t_idx, c_id);
                    if !is_audio {
                        self.top_bar_state.active_tab = crate::views::modern_top_bar::ModernViewTab::PianoRoll;
                    }
                }
                if let Some((t_idx, c_id, delta_b)) = clip_drag_action {
                    if let Some(track) = self.tracks.get_mut(t_idx) {
                        track.move_clip(c_id, delta_b);
                    }
                }
                if resp.drag_stopped() {
                    if let Some((t_idx, c_id)) = self.selected_clip {
                        let snap = self.arranger_snap_grid;
                        if let Some(track) = self.tracks.get_mut(t_idx) {
                            if let Some(c) = track.clips.iter_mut().find(|c| c.id == c_id) {
                                c.start_beat = snap.snap(c.start_beat);
                            }
                        }
                    }
                }

                // Keyboard shortcuts for Arranger clip operations
                if ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::D)) {
                    self.duplicate_selected_clip();
                }
                if ui.input(|i| (i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)) && !i.modifiers.ctrl) {
                    self.delete_selected_clip();
                }
                if ui.input(|i| i.key_pressed(egui::Key::Q) && !i.modifiers.ctrl) {
                    self.quantize_selected_clip();
                }
                if ui.input(|i| i.key_pressed(egui::Key::L) && !i.modifiers.ctrl) {
                    self.set_loop_to_selected_clip();
                }

                // 3. Bright Glowing Cyan Playhead Line (Spans entire height)
                let playhead_x = rect.left() + header_w + (self.playhead_beat * ppb);
                painter.line_segment(
                    [egui::pos2(playhead_x, rect.top()), egui::pos2(playhead_x, rect.bottom())],
                    Stroke::new(3.0_f32, Color32::from_rgba_unmultiplied(56, 189, 248, 70)),
                );
                painter.line_segment(
                    [egui::pos2(playhead_x, rect.top()), egui::pos2(playhead_x, rect.bottom())],
                    Stroke::new(1.5_f32, Color32::from_rgb(56, 189, 248)),
                );
                // Top Playhead Diamond/Pill Marker
                let marker_rect = Rect::from_center_size(egui::pos2(playhead_x, rect.top() + 6.0), Vec2::new(8.0, 12.0));
                painter.rect_filled(marker_rect, 2.0, Color32::from_rgb(56, 189, 248));
            });
    }

    #[cfg(feature = "gui")]
    fn show_piano_roll_canvas(&mut self, ui: &mut egui::Ui) {
        let is_pro = self.top_bar_state.is_pro_mode;
        let pitch_names = self.piano_roll_tuning.pitch_names();
        let num_pitches = pitch_names.len();
        let cur_track_id = self.tracks.get(self.selected_track_idx).map(|t| t.id).unwrap_or(1);
        let cur_track_name = self.tracks.get(self.selected_track_idx).map(|t| t.name.clone()).unwrap_or_else(|| "Track 1".to_string());

        // 0. Piano Roll Top Toolbar (Two-Tier: Novice vs Pro)
        ui.horizontal(|ui| {
            ui.label(RichText::new("🎹 Piano Roll").font(FontId::proportional(12.0)).strong().color(Color32::from_rgb(241, 245, 249)));
            ui.add_space(6.0);

            // Track Selector ComboBox
            egui::ComboBox::from_id_source("piano_roll_track_selector")
                .selected_text(RichText::new(format!("🎚 {}", cur_track_name)).font(FontId::proportional(10.5)).color(Color32::from_rgb(56, 189, 248)))
                .show_ui(ui, |ui| {
                    for (t_idx, tr) in self.tracks.iter().enumerate() {
                        let is_sel = t_idx == self.selected_track_idx;
                        if ui.selectable_label(is_sel, format!("{}: {}", tr.id, tr.name)).clicked() {
                            self.selected_track_idx = t_idx;
                            self.device_rack_state.device_name = tr.name.clone();
                        }
                    }
                });

            ui.add_space(4.0);

            // 1-Click Pro View Launchers
            if ui.button(RichText::new("📋 Arranger").font(FontId::proportional(10.0)).color(Color32::from_rgb(200, 215, 235))).clicked() {
                self.top_bar_state.active_tab = crate::views::modern_top_bar::ModernViewTab::Arranger;
            }
            if ui.button(RichText::new("∿ Modular").font(FontId::proportional(10.0)).color(Color32::from_rgb(200, 215, 235))).clicked() {
                self.top_bar_state.active_tab = crate::views::modern_top_bar::ModernViewTab::Modular;
            }
            if ui.button(RichText::new("🎚 Mixer").font(FontId::proportional(10.0)).color(Color32::from_rgb(200, 215, 235))).clicked() {
                self.top_bar_state.active_tab = crate::views::modern_top_bar::ModernViewTab::Mixer;
            }

            ui.add_space(4.0);

            // 1-Click Live Parameter Automation Launcher
            egui::ComboBox::from_id_source("piano_roll_auto_selector")
                .selected_text(RichText::new("📈 Auto ▾").font(FontId::proportional(10.0)).color(Color32::from_rgb(234, 179, 8)))
                .show_ui(ui, |ui| {
                    if ui.selectable_label(false, "📈 Cutoff (Filter)").clicked() {
                        self.open_track_automation_editor(cur_track_id, "cutoff");
                    }
                    if ui.selectable_label(false, "📈 Pitch Bend").clicked() {
                        self.open_track_automation_editor(cur_track_id, "pitch");
                    }
                    if ui.selectable_label(false, "📈 Modulation Wheel").clicked() {
                        self.open_track_automation_editor(cur_track_id, "mod");
                    }
                    if ui.selectable_label(false, "📈 Note Velocity").clicked() {
                        self.open_track_automation_editor(cur_track_id, "velocity");
                    }
                    if ui.selectable_label(false, "📈 Volume Gain").clicked() {
                        self.open_track_automation_editor(cur_track_id, "gain");
                    }
                    if ui.selectable_label(false, "📈 Stereo Pan").clicked() {
                        self.open_track_automation_editor(cur_track_id, "pan");
                    }
                });

            ui.add_space(4.0);

            // Audition sound toggle
            let aud_col = if self.is_piano_roll_audition_enabled { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(100, 116, 139) };
            let aud_txt = if self.is_piano_roll_audition_enabled { "🔊 Audition" } else { "🔇 Muted" };
            if ui.button(RichText::new(aud_txt).font(FontId::proportional(10.0)).color(aud_col)).on_hover_text("Toggle live keyboard note auditioning audio").clicked() {
                self.toggle_piano_roll_audition();
            }

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            // Lower lane mode selector
            let v_col = if self.piano_roll_lane_mode == PianoRollLaneMode::Velocity { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(148, 163, 184) };
            let g_col = if self.piano_roll_lane_mode == PianoRollLaneMode::Gate { Color32::from_rgb(34, 197, 94) } else { Color32::from_rgb(148, 163, 184) };
            let p_col = if self.piano_roll_lane_mode == PianoRollLaneMode::PitchBend { Color32::from_rgb(245, 158, 11) } else { Color32::from_rgb(148, 163, 184) };
            if ui.selectable_label(self.piano_roll_lane_mode == PianoRollLaneMode::Velocity, RichText::new("Vel").font(FontId::proportional(10.0)).color(v_col)).clicked() {
                self.piano_roll_lane_mode = PianoRollLaneMode::Velocity;
            }
            if ui.selectable_label(self.piano_roll_lane_mode == PianoRollLaneMode::Gate, RichText::new("Gate").font(FontId::proportional(10.0)).color(g_col)).clicked() {
                self.piano_roll_lane_mode = PianoRollLaneMode::Gate;
            }
            if ui.selectable_label(self.piano_roll_lane_mode == PianoRollLaneMode::PitchBend, RichText::new("Pitch").font(FontId::proportional(10.0)).color(p_col)).clicked() {
                self.piano_roll_lane_mode = PianoRollLaneMode::PitchBend;
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let note_count = self.piano_roll_notes.len();
                ui.label(RichText::new(format!("{} Notes", note_count)).font(FontId::proportional(9.5)).color(Color32::from_rgb(100, 116, 139)));
            });
        });
        ui.add_space(3.0);

        if is_pro {
            ui.horizontal(|ui| {
                // Tuning System Selector
                egui::ComboBox::from_id_source("piano_roll_tuning_selector")
                    .selected_text(RichText::new(format!("∿ {}", self.piano_roll_tuning.name())).font(FontId::proportional(10.5)).color(Color32::from_rgb(56, 189, 248)))
                    .show_ui(ui, |ui| {
                        for &t in &[PianoRollTuningSystem::Edo12, PianoRollTuningSystem::Edo19, PianoRollTuningSystem::Edo31, PianoRollTuningSystem::BohlenPierce] {
                            let is_sel = self.piano_roll_tuning == t;
                            if ui.selectable_label(is_sel, t.name()).clicked() {
                                self.piano_roll_tuning = t;
                            }
                        }
                    });

                ui.add_space(4.0);

                // Snap Grid Selector
                egui::ComboBox::from_id_source("piano_roll_snap_selector")
                    .selected_text(RichText::new(format!("⊞ Snap: {}", self.piano_roll_snap.name())).font(FontId::proportional(10.5)).color(Color32::from_rgb(200, 215, 235)))
                    .show_ui(ui, |ui| {
                        for &s in &[PianoRollSnapResolution::BeatQuarter, PianoRollSnapResolution::BeatEighth, PianoRollSnapResolution::BeatSixteenth, PianoRollSnapResolution::BeatThirtySecond, PianoRollSnapResolution::Free] {
                            let is_sel = self.piano_roll_snap == s;
                            if ui.selectable_label(is_sel, s.name()).clicked() {
                                self.piano_roll_snap = s;
                            }
                        }
                    });

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                // Surgical Note Operations
                if ui.button(RichText::new("⇥ Quantize").font(FontId::proportional(10.0)).color(Color32::from_rgb(200, 215, 235))).on_hover_text("Quantize notes to active snap grid").clicked() {
                    self.quantize_piano_roll_notes();
                }

                if ui.button(RichText::new("▲ +1").font(FontId::proportional(10.0)).color(Color32::from_rgb(200, 215, 235))).on_hover_text("Transpose +1 pitch step").clicked() {
                    self.transpose_selected_piano_roll_note(1);
                }

                if ui.button(RichText::new("▼ -1").font(FontId::proportional(10.0)).color(Color32::from_rgb(200, 215, 235))).on_hover_text("Transpose -1 pitch step").clicked() {
                    self.transpose_selected_piano_roll_note(-1);
                }

                let oct_steps = match self.piano_roll_tuning {
                    PianoRollTuningSystem::Edo12 => 12,
                    PianoRollTuningSystem::Edo19 => 19,
                    PianoRollTuningSystem::Edo31 => 31,
                    PianoRollTuningSystem::BohlenPierce => 13,
                };

                if ui.button(RichText::new("▲▲ +Oct").font(FontId::proportional(10.0)).color(Color32::from_rgb(200, 215, 235))).on_hover_text("Transpose +1 Octave / Tritave").clicked() {
                    self.transpose_selected_piano_roll_note(oct_steps);
                }

                if ui.button(RichText::new("▼▼ -Oct").font(FontId::proportional(10.0)).color(Color32::from_rgb(200, 215, 235))).on_hover_text("Transpose -1 Octave / Tritave").clicked() {
                    self.transpose_selected_piano_roll_note(-oct_steps);
                }

                if ui.button(RichText::new("🎲 Humanize").font(FontId::proportional(10.0)).color(Color32::from_rgb(200, 215, 235))).on_hover_text("Apply subtle micro-timing and velocity humanization").clicked() {
                    for (idx, n) in self.piano_roll_notes.iter_mut().enumerate() {
                        let jitter_t = ((idx * 7 + 3) % 5) as f32 * 0.01 - 0.02;
                        n.start_beat = (n.start_beat + jitter_t).max(0.0);
                        let jitter_v = ((idx * 11 + 5) % 9) as f32 * 0.02 - 0.08;
                        n.velocity = (n.velocity + jitter_v).clamp(0.2, 1.0);
                    }
                }

                if ui.button(RichText::new("⧉ Duplicate").font(FontId::proportional(10.0)).color(Color32::from_rgb(200, 215, 235))).on_hover_text("Duplicate selected note").clicked() {
                    self.duplicate_selected_piano_roll_note();
                }

                if ui.button(RichText::new("✕ Delete").font(FontId::proportional(10.0)).color(Color32::from_rgb(244, 63, 94))).clicked() {
                    if let Some(sel_id) = self.selected_note_id {
                        self.piano_roll_notes.retain(|n| n.id != sel_id);
                        self.selected_note_id = None;
                    }
                }

                if ui.button(RichText::new("⊘ Clear").font(FontId::proportional(10.0)).color(Color32::from_rgb(244, 63, 94))).on_hover_text("Clear all notes from piano roll").clicked() {
                    self.clear_piano_roll_notes();
                }
            });
        } else {
            ui.horizontal(|ui| {
                egui::Frame::none()
                    .fill(Color32::from_rgb(20, 28, 44))
                    .rounding(Rounding::same(3.0))
                    .inner_margin(egui::Margin::symmetric(6.0, 3.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new("12-TET Standard • 1/4 Beat Snap").font(FontId::proportional(9.5)).color(Color32::from_rgb(148, 163, 184)));
                    });
                ui.add_space(4.0);
                if ui.button(RichText::new("⇥ Quantize").font(FontId::proportional(10.0)).color(Color32::from_rgb(200, 215, 235))).clicked() {
                    self.quantize_piano_roll_notes();
                }
            });
        }
        ui.add_space(4.0);

        let min_row_h = if num_pitches > 24 { 14.0 } else if num_pitches > 16 { 18.0 } else { 22.0 };
        let roll_area_h = (num_pitches as f32 * min_row_h).max(220.0);
        let pitch_row_h = roll_area_h / num_pitches as f32;
        let ruler_h = 24.0;
        let vel_lane_h = 32.0;
        let canvas_height = ruler_h + roll_area_h + vel_lane_h;

        egui::Frame::none()
            .fill(Color32::from_rgb(10, 14, 24))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(28, 40, 60)))
            .rounding(Rounding::same(6.0))
            .show(ui, |ui| {
                ui.set_height(canvas_height);
                let (resp, painter) = ui.allocate_painter(Vec2::new(ui.available_width(), canvas_height), egui::Sense::click_and_drag());
                let rect = resp.rect;

                let key_w = if is_pro { 78.0 } else { 64.0 };
                let grid_w = (rect.width() - key_w).max(200.0);
                let ppb = grid_w / 18.0;

                // 1. Timeline Header Ruler (Measures 1..17)
                let ruler_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), ruler_h));
                painter.rect_filled(ruler_rect, 0.0, Color32::from_rgb(14, 20, 32));
                painter.line_segment([ruler_rect.left_bottom(), ruler_rect.right_bottom()], Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 74)));

                for bar in 1..=17 {
                    let bar_x = rect.left() + key_w + ((bar - 1) as f32 * 4.0 * ppb);
                    painter.line_segment([egui::pos2(bar_x, ruler_rect.top()), egui::pos2(bar_x, ruler_rect.bottom())], Stroke::new(1.0_f32, Color32::from_rgb(50, 65, 90)));
                    painter.text(egui::pos2(bar_x + 4.0, ruler_rect.top() + 4.0), egui::Align2::LEFT_TOP, format!("{}", bar), FontId::proportional(9.0), Color32::from_rgb(148, 163, 184));
                }

                let pointer_pos = resp.interact_pointer_pos().or_else(|| ui.input(|i| i.pointer.latest_pos()));
                let is_click = resp.clicked() || ui.input(|i| i.pointer.primary_clicked());
                let is_down = ui.input(|i| i.pointer.primary_down());
                let is_secondary_click = ui.input(|i| i.pointer.secondary_clicked());
                let is_delete_pressed = ui.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace));

                // Ruler playhead scrubbing
                if let Some(pos) = pointer_pos {
                    if (is_click || is_down || resp.dragged()) && pos.y <= rect.top() + ruler_h {
                        let beat = ((pos.x - (rect.left() + key_w)) / ppb).max(0.0);
                        self.playhead_beat = beat;
                    }
                }

                // Delete selected note shortcut
                if is_delete_pressed {
                    if let Some(sel_id) = self.selected_note_id {
                        self.piano_roll_notes.retain(|n| n.id != sel_id);
                        self.selected_note_id = None;
                    }
                }

                // Interactive Pointer Handling (Keyboard, Velocity Lane, Grid Notes, Resizing)
                if let Some(pos) = pointer_pos {
                    if pos.x <= rect.left() + key_w && pos.y > rect.top() + ruler_h && pos.y < rect.bottom() - vel_lane_h {
                        // 2A. Interactive Piano Keyboard Clicking / Auditioning
                        let p_idx = (((pos.y - (rect.top() + ruler_h)) / pitch_row_h) as usize).min(num_pitches - 1);
                        if is_down || is_click {
                            let freq_hz = self.piano_roll_tuning.pitch_to_hz(p_idx, num_pitches);
                            self.auditioned_pitch_idx = Some(p_idx);
                            self.last_auditioned_pitch_hz = freq_hz;
                            self.last_auditioned_gate = if self.is_piano_roll_audition_enabled { 1.0 } else { 0.0 };
                            self.inspector_state.target_name = format!("Pitch {} ({:.1} Hz)", pitch_names[p_idx], freq_hz);
                            self.inspector_state.scale_ratio_num = (num_pitches - p_idx) as i32;
                            self.inspector_state.root_ratio = freq_hz;
                            self.inspector_state.octave_offset = if p_idx < 1 { 5 } else { 4 };
                        }
                    } else if pos.x <= rect.left() + key_w && pos.y >= rect.bottom() - vel_lane_h {
                        // 2D. Interactive Lane Mode Badge Cycling
                        if is_click {
                            self.piano_roll_lane_mode = match self.piano_roll_lane_mode {
                                PianoRollLaneMode::Velocity => PianoRollLaneMode::Gate,
                                PianoRollLaneMode::Gate => PianoRollLaneMode::PitchBend,
                                PianoRollLaneMode::PitchBend => PianoRollLaneMode::Velocity,
                            };
                        }
                    } else if pos.y >= rect.bottom() - vel_lane_h && pos.x > rect.left() + key_w {
                        // 2B. Interactive Bottom Lane Dragging (Velocity / Gate Length / Pitch Bend)
                        if is_down || is_click || resp.dragged() {
                            let mut closest_idx = None;
                            let mut min_dist = f32::MAX;
                            for (idx, note) in self.piano_roll_notes.iter().enumerate() {
                                let stick_x = rect.left() + key_w + note.start_beat * ppb + 2.0;
                                let dist = (pos.x - stick_x).abs();
                                if dist < min_dist {
                                    min_dist = dist;
                                    closest_idx = Some(idx);
                                }
                            }
                            let norm_y = ((rect.bottom() - 2.0 - pos.y) / (vel_lane_h * 0.85)).clamp(0.05, 1.0);
                            if min_dist < (ppb * 1.5).max(14.0) {
                                if let Some(idx) = closest_idx {
                                    match self.piano_roll_lane_mode {
                                        PianoRollLaneMode::Velocity => {
                                            self.piano_roll_notes[idx].velocity = norm_y;
                                        }
                                        PianoRollLaneMode::Gate => {
                                            let step = self.piano_roll_snap.step_beats().max(0.125);
                                            let raw_len = (norm_y * 4.0).max(0.125);
                                            self.piano_roll_notes[idx].length_beats = if self.piano_roll_snap != PianoRollSnapResolution::Free {
                                                (raw_len / step).round() * step
                                            } else {
                                                raw_len
                                            };
                                        }
                                        PianoRollLaneMode::PitchBend => {
                                            let track_id = self.tracks.get(self.selected_track_idx).map(|t| t.id).unwrap_or(1);
                                            self.open_track_automation_editor(track_id, "pitch");
                                        }
                                    }
                                    self.selected_note_id = Some(self.piano_roll_notes[idx].id);
                                }
                            } else if let Some(sel_id) = self.selected_note_id {
                                if let Some(note) = self.piano_roll_notes.iter_mut().find(|n| n.id == sel_id) {
                                    match self.piano_roll_lane_mode {
                                        PianoRollLaneMode::Velocity => {
                                            note.velocity = norm_y;
                                        }
                                        PianoRollLaneMode::Gate => {
                                            let step = self.piano_roll_snap.step_beats().max(0.125);
                                            let raw_len = (norm_y * 4.0).max(0.125);
                                            note.length_beats = if self.piano_roll_snap != PianoRollSnapResolution::Free {
                                                (raw_len / step).round() * step
                                            } else {
                                                raw_len
                                            };
                                        }
                                        PianoRollLaneMode::PitchBend => {
                                            let track_id = self.tracks.get(self.selected_track_idx).map(|t| t.id).unwrap_or(1);
                                            self.open_track_automation_editor(track_id, "pitch");
                                        }
                                    }
                                }
                            }
                        }
                    } else if pos.x > rect.left() + key_w && pos.y > rect.top() + ruler_h && pos.y < rect.bottom() - vel_lane_h {
                        // 2C. Grid Note Interaction: Resizing, Selection, Creation, Deletion
                        // If actively resizing a note, update its length
                        if let Some(resizing_id) = self.piano_roll_resizing_note_id {
                            if is_down || resp.dragged() {
                                if let Some(note) = self.piano_roll_notes.iter_mut().find(|n| n.id == resizing_id) {
                                    let note_x = rect.left() + key_w + note.start_beat * ppb;
                                    let raw_len = ((pos.x - note_x) / ppb).max(0.125);
                                    let snapped_len = if self.piano_roll_snap != PianoRollSnapResolution::Free {
                                        self.piano_roll_snap.snap(raw_len).max(self.piano_roll_snap.step_beats().max(0.125))
                                    } else {
                                        raw_len
                                    };
                                    note.length_beats = snapped_len;
                                }
                            }
                        } else if is_click || is_secondary_click {
                            let mut resize_hit_id = None;
                            let mut body_hit_idx = None;

                            for (idx, note) in self.piano_roll_notes.iter().enumerate() {
                                let clamped_p = note.pitch_idx.min(num_pitches - 1);
                                let note_y = rect.top() + ruler_h + clamped_p as f32 * pitch_row_h + 1.5;
                                let note_h = pitch_row_h - 3.0;
                                let note_x = rect.left() + key_w + note.start_beat * ppb;
                                let note_w = (note.length_beats * ppb).max(8.0);
                                let n_rect = Rect::from_min_size(egui::pos2(note_x, note_y), Vec2::new(note_w, note_h));
                                if n_rect.contains(pos) {
                                    if pos.x >= n_rect.right() - 6.0 && !is_secondary_click {
                                        resize_hit_id = Some(note.id);
                                    } else {
                                        body_hit_idx = Some(idx);
                                    }
                                    break;
                                }
                            }

                            if let Some(r_id) = resize_hit_id {
                                self.piano_roll_resizing_note_id = Some(r_id);
                                self.selected_note_id = Some(r_id);
                            } else if let Some(idx) = body_hit_idx {
                                if is_secondary_click {
                                    self.piano_roll_notes.remove(idx);
                                    self.selected_note_id = None;
                                } else {
                                    let note = &self.piano_roll_notes[idx];
                                    self.selected_note_id = Some(note.id);
                                    let clamped_p = note.pitch_idx.min(num_pitches - 1);
                                    let freq_hz = self.piano_roll_tuning.pitch_to_hz(clamped_p, num_pitches);
                                    self.last_auditioned_pitch_hz = freq_hz;
                                    self.last_auditioned_gate = if self.is_piano_roll_audition_enabled { 1.0 } else { 0.0 };
                                    self.inspector_state.target_name = format!("Note {} ({:.1} Hz)", pitch_names[clamped_p], freq_hz);
                                    self.inspector_state.scale_ratio_num = (num_pitches - clamped_p) as i32;
                                    self.inspector_state.root_ratio = freq_hz;
                                    self.inspector_state.octave_offset = if clamped_p < 1 { 5 } else { 4 };
                                }
                            } else if is_click {
                                // Clicked empty cell: create new note
                                let p_idx = (((pos.y - (rect.top() + ruler_h)) / pitch_row_h) as usize).min(num_pitches - 1);
                                let raw_b = ((pos.x - (rect.left() + key_w)) / ppb).max(0.0);
                                let start_b = self.piano_roll_snap.snap(raw_b);
                                let snap_len = self.piano_roll_snap.step_beats().max(0.5);
                                let new_id = self.next_note_id;
                                self.next_note_id += 1;
                                self.piano_roll_notes.push(PianoRollNote {
                                    id: new_id,
                                    pitch_idx: p_idx,
                                    start_beat: start_b,
                                    length_beats: snap_len,
                                    velocity: 0.85,
                                });
                                self.selected_note_id = Some(new_id);
                                let freq_hz = self.piano_roll_tuning.pitch_to_hz(p_idx, num_pitches);
                                self.last_auditioned_pitch_hz = freq_hz;
                                self.last_auditioned_gate = if self.is_piano_roll_audition_enabled { 1.0 } else { 0.0 };
                                self.inspector_state.target_name = format!("Note {} ({:.1} Hz)", pitch_names[p_idx], freq_hz);
                                self.inspector_state.scale_ratio_num = (num_pitches - p_idx) as i32;
                                self.inspector_state.root_ratio = freq_hz;
                                self.inspector_state.octave_offset = if p_idx < 1 { 5 } else { 4 };
                            }
                        }
                    }
                }
                if !is_down && !is_click {
                    self.auditioned_pitch_idx = None;
                    self.last_auditioned_gate = 0.0;
                    self.piano_roll_resizing_note_id = None;
                }

                // 2. Render Pitch Rows & Piano Keys
                for (p_idx, p_name) in pitch_names.iter().enumerate() {
                    let py = rect.top() + ruler_h + p_idx as f32 * pitch_row_h;
                    let is_accidental = p_name.contains('#') || p_name.contains('b');
                    let is_tonic = p_name.starts_with("A4") || p_name.starts_with("C4") || p_name.starts_with("C5");
                    let is_auditioned = self.auditioned_pitch_idx == Some(p_idx);

                    // Key Label Header
                    let key_rect = Rect::from_min_size(egui::pos2(rect.left(), py), Vec2::new(key_w, pitch_row_h));
                    let key_bg = if is_auditioned {
                        Color32::from_rgb(14, 116, 144) // Active pressed ocean cyan
                    } else if is_tonic {
                        Color32::from_rgb(24, 38, 56)
                    } else if is_accidental {
                        Color32::from_rgb(12, 16, 24)
                    } else {
                        Color32::from_rgb(18, 24, 34)
                    };
                    painter.rect_filled(key_rect, 1.0, key_bg);
                    let border_col = if is_auditioned {
                        Color32::from_rgb(56, 189, 248)
                    } else {
                        Color32::from_rgb(28, 40, 60)
                    };
                    painter.rect_stroke(key_rect, 1.0, Stroke::new(if is_auditioned { 1.5_f32 } else { 0.5_f32 }, border_col));
                    let key_text_col = if is_auditioned {
                        Color32::WHITE
                    } else if is_tonic {
                        Color32::from_rgb(56, 189, 248)
                    } else {
                        Color32::from_rgb(148, 163, 184)
                    };
                    let key_display = if is_pro && key_w >= 75.0 && num_pitches <= 24 {
                        let hz = self.piano_roll_tuning.pitch_to_hz(p_idx, num_pitches);
                        format!("{} {:.0}Hz", p_name, hz)
                    } else {
                        p_name.clone()
                    };
                    painter.text(egui::pos2(key_rect.left() + 6.0, key_rect.center().y), egui::Align2::LEFT_CENTER, key_display, FontId::proportional(8.5), key_text_col);

                    // Grid Lane
                    let lane_rect = Rect::from_min_size(egui::pos2(rect.left() + key_w, py), Vec2::new(grid_w, pitch_row_h));
                    let lane_bg = if is_accidental { Color32::from_rgb(8, 12, 18) } else { Color32::from_rgb(12, 16, 26) };
                    painter.rect_filled(lane_rect, 0.0, lane_bg);
                    painter.line_segment([lane_rect.left_bottom(), lane_rect.right_bottom()], Stroke::new(0.5_f32, Color32::from_rgb(20, 28, 42)));

                    // Beat grid vertical lines
                    for bar in 1..=17 {
                        let bar_x = rect.left() + key_w + ((bar - 1) as f32 * 4.0 * ppb);
                        painter.line_segment([egui::pos2(bar_x, lane_rect.top()), egui::pos2(bar_x, lane_rect.bottom())], Stroke::new(0.5_f32, Color32::from_rgb(24, 34, 50)));
                    }
                }

                // 3. Velocity / Gate / Pitch Lane Background & Separator
                let vel_sep_y = rect.bottom() - vel_lane_h;
                painter.line_segment([egui::pos2(rect.left(), vel_sep_y), egui::pos2(rect.right(), vel_sep_y)], Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 74)));
                painter.rect_filled(Rect::from_min_size(egui::pos2(rect.left(), vel_sep_y), Vec2::new(rect.width(), vel_lane_h)), 0.0, Color32::from_rgb(12, 16, 26));

                // 3B. Bottom-Left Mode Badge
                let lane_badge_rect = Rect::from_min_size(egui::pos2(rect.left() + 2.0, vel_sep_y + 4.0), Vec2::new(key_w - 4.0, vel_lane_h - 8.0));
                painter.rect_filled(lane_badge_rect, 3.0, Color32::from_rgb(18, 24, 36));
                painter.rect_stroke(lane_badge_rect, 3.0, Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 74)));
                let (badge_col, badge_txt) = match self.piano_roll_lane_mode {
                    PianoRollLaneMode::Velocity => (Color32::from_rgb(56, 189, 248), "VELOCITY"),
                    PianoRollLaneMode::Gate => (Color32::from_rgb(34, 197, 94), "GATE LEN"),
                    PianoRollLaneMode::PitchBend => (Color32::from_rgb(245, 158, 11), "PITCH BEND"),
                };
                painter.text(lane_badge_rect.center(), egui::Align2::CENTER_CENTER, badge_txt, FontId::proportional(8.5), badge_col);

                // 4. MIDI Note Blocks & Multi-Lane Sticks
                for note in &self.piano_roll_notes {
                    let is_sel = self.selected_note_id == Some(note.id);
                    let clamped_p = note.pitch_idx.min(num_pitches - 1);
                    let note_y = rect.top() + ruler_h + clamped_p as f32 * pitch_row_h + 1.5;
                    let note_h = pitch_row_h - 3.0;
                    let note_x = rect.left() + key_w + note.start_beat * ppb;
                    let note_w = (note.length_beats * ppb).max(8.0);
                    let note_rect = Rect::from_min_size(egui::pos2(note_x, note_y), Vec2::new(note_w, note_h));

                    // Glowing note fill
                    let fill_alpha = if is_sel {
                        (note.velocity * 220.0 + 35.0).min(255.0) as u8
                    } else {
                        (note.velocity * 180.0 + 20.0).min(240.0) as u8
                    };
                    let fill_col = Color32::from_rgba_unmultiplied(56, 189, 248, fill_alpha);
                    painter.rect_filled(note_rect, 2.0, fill_col);

                    if is_sel {
                        // High-contrast selection aura
                        painter.rect_stroke(note_rect.expand(1.5), 3.0, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(56, 189, 248, 120)));
                        painter.rect_stroke(note_rect, 2.0, Stroke::new(1.8_f32, Color32::WHITE));
                    } else {
                        painter.rect_stroke(note_rect, 2.0, Stroke::new(1.0_f32, Color32::from_rgb(56, 189, 248)));
                    }

                    // Interactive note resize handle indicator on right edge
                    if is_sel || note_w >= 14.0 {
                        let handle_w = 4.0;
                        let handle_rect = Rect::from_min_size(egui::pos2(note_rect.right() - handle_w, note_rect.top() + 1.0), Vec2::new(handle_w, note_h - 2.0));
                        painter.rect_filled(handle_rect, 1.0, if is_sel { Color32::WHITE } else { Color32::from_rgba_unmultiplied(255, 255, 255, 120) });
                    }

                    // Note Pitch Label inside block
                    if note_w >= 22.0 && clamped_p < num_pitches {
                        painter.text(
                            egui::pos2(note_rect.left() + 4.0, note_rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            &pitch_names[clamped_p],
                            FontId::proportional(8.5),
                            if is_sel { Color32::WHITE } else { Color32::from_rgb(220, 240, 255) },
                        );
                    }

                    // Bottom lane stick based on piano_roll_lane_mode
                    let (lane_val_norm, readout_text, stick_col) = match self.piano_roll_lane_mode {
                        PianoRollLaneMode::Velocity => (
                            note.velocity,
                            format!("{:.0}%", note.velocity * 100.0),
                            if is_sel { Color32::WHITE } else { Color32::from_rgb(56, 189, 248) },
                        ),
                        PianoRollLaneMode::Gate => (
                            (note.length_beats / 4.0).clamp(0.05, 1.0),
                            format!("{:.2}b", note.length_beats),
                            if is_sel { Color32::WHITE } else { Color32::from_rgb(34, 197, 94) },
                        ),
                        PianoRollLaneMode::PitchBend => (
                            0.5_f32,
                            "0st".to_string(),
                            if is_sel { Color32::WHITE } else { Color32::from_rgb(245, 158, 11) },
                        ),
                    };
                    let v_height = vel_lane_h * lane_val_norm * 0.85;
                    let stick_x = note_x + 2.0;
                    let cap_y = rect.bottom() - 2.0 - v_height;
                    painter.line_segment([egui::pos2(stick_x, rect.bottom() - 2.0), egui::pos2(stick_x, cap_y)], Stroke::new(if is_sel { 2.0_f32 } else { 1.5_f32 }, stick_col));
                    painter.circle_filled(egui::pos2(stick_x, cap_y), if is_sel { 3.5 } else { 2.5 }, stick_col);
                    if is_sel {
                        painter.text(
                            egui::pos2(stick_x + 6.0, cap_y),
                            egui::Align2::LEFT_CENTER,
                            readout_text,
                            FontId::proportional(8.0),
                            stick_col,
                        );
                    }
                }

                // 5. Playhead Line (Spans entire height with diamond marker)
                let playhead_x = rect.left() + key_w + (self.playhead_beat * ppb);
                painter.line_segment(
                    [egui::pos2(playhead_x, rect.top()), egui::pos2(playhead_x, rect.bottom())],
                    Stroke::new(3.0_f32, Color32::from_rgba_unmultiplied(56, 189, 248, 70)),
                );
                painter.line_segment(
                    [egui::pos2(playhead_x, rect.top()), egui::pos2(playhead_x, rect.bottom())],
                    Stroke::new(1.5_f32, Color32::from_rgb(56, 189, 248)),
                );
                let marker_rect = Rect::from_center_size(egui::pos2(playhead_x, rect.top() + 6.0), Vec2::new(8.0, 12.0));
                painter.rect_filled(marker_rect, 2.0, Color32::from_rgb(56, 189, 248));
            });
    }

    pub fn reset_modular_nodes(&mut self) {
        let mut graph = summoner_core::graph::NodeGraph::new("ModularCanvasGraph", 512, 2);
        let osc_idx = graph.add_node(Box::new(summoner_core::node::SineOscillatorNode::new(440.0)));
        let filter_idx = graph.add_node(Box::new(summoner_core::node::GainNode::new(1.0)));
        let env_idx = graph.add_node(Box::new(summoner_core::node::GainNode::new(1.0)));
        let vca_idx = graph.add_node(Box::new(summoner_core::node::GainNode::new(1.0)));

        graph.connect(osc_idx, 0, filter_idx, 0);
        graph.connect(filter_idx, 0, vca_idx, 0);
        graph.connect(env_idx, 0, vca_idx, 1);

        self.audio_graph = std::sync::Arc::new(std::sync::Mutex::new(graph));

        self.modular_nodes = vec![
            ModularNodeInstance {
                id: "osc_1".into(),
                kind_id: "OscSaw".into(),
                display_name: "Osc Saw".into(),
                category: crate::dsp_node_ui::DspNodeCategory::Oscillator,
                pos: (24.0, 20.0),
                size: (140.0, 105.0),
                ports: vec![
                    ModularPort { id: "voct".into(), name: "V/Oct".into(), kind: ModularPortKind::ModulationIn, rel_pos: (14.0, 88.0) },
                    ModularPort { id: "sync".into(), name: "Sync".into(), kind: ModularPortKind::GateIn, rel_pos: (45.0, 88.0) },
                    ModularPort { id: "out".into(), name: "Out".into(), kind: ModularPortKind::AudioOut, rel_pos: (126.0, 88.0) },
                ],
                params: vec![
                    ModularNodeParam {
                        id: "freq".into(),
                        name: "Freq".into(),
                        value: 440.0,
                        min: 20.0,
                        max: 2000.0,
                        default_value: 440.0,
                        step: Some(1.0),
                        unit: "Hz".into(),
                        rel_pos: (46.0, 48.0),
                    },
                    ModularNodeParam {
                        id: "shape".into(),
                        name: "Shape".into(),
                        value: 0.5,
                        min: 0.0,
                        max: 1.0,
                        default_value: 0.5,
                        step: Some(0.01),
                        unit: "".into(),
                        rel_pos: (96.0, 48.0),
                    },
                ],
                graph_node_idx: Some(osc_idx),
                bypassed: false,
            },
            ModularNodeInstance {
                id: "filter_1".into(),
                kind_id: "FilterSvf".into(),
                display_name: "Filter SVF".into(),
                category: crate::dsp_node_ui::DspNodeCategory::FilterEq,
                pos: (190.0, 35.0),
                size: (150.0, 115.0),
                ports: vec![
                    ModularPort { id: "in".into(), name: "In 1".into(), kind: ModularPortKind::AudioIn, rel_pos: (14.0, 45.0) },
                    ModularPort { id: "cv_cut".into(), name: "CV Cut".into(), kind: ModularPortKind::ModulationIn, rel_pos: (14.0, 75.0) },
                    ModularPort { id: "lp".into(), name: "LP".into(), kind: ModularPortKind::AudioOut, rel_pos: (136.0, 45.0) },
                ],
                params: vec![
                    ModularNodeParam {
                        id: "cutoff".into(),
                        name: "Cutoff".into(),
                        value: 1200.0,
                        min: 20.0,
                        max: 20000.0,
                        default_value: 1200.0,
                        step: Some(10.0),
                        unit: "Hz".into(),
                        rel_pos: (48.0, 50.0),
                    },
                    ModularNodeParam {
                        id: "resonance".into(),
                        name: "Res".into(),
                        value: 1.0,
                        min: 0.1,
                        max: 10.0,
                        default_value: 1.0,
                        step: Some(0.05),
                        unit: "Q".into(),
                        rel_pos: (100.0, 50.0),
                    },
                ],
                graph_node_idx: Some(filter_idx),
                bypassed: false,
            },
            ModularNodeInstance {
                id: "env_1".into(),
                kind_id: "EnvAdsr".into(),
                display_name: "Env ADSR".into(),
                category: crate::dsp_node_ui::DspNodeCategory::Modulation,
                pos: (370.0, 20.0),
                size: (140.0, 105.0),
                ports: vec![
                    ModularPort { id: "gate".into(), name: "Gate".into(), kind: ModularPortKind::GateIn, rel_pos: (14.0, 88.0) },
                    ModularPort { id: "cv".into(), name: "CV".into(), kind: ModularPortKind::ModulationOut, rel_pos: (126.0, 88.0) },
                ],
                params: vec![
                    ModularNodeParam {
                        id: "attack".into(),
                        name: "Atk".into(),
                        value: 0.01,
                        min: 0.001,
                        max: 2.0,
                        default_value: 0.01,
                        step: Some(0.005),
                        unit: "s".into(),
                        rel_pos: (46.0, 48.0),
                    },
                    ModularNodeParam {
                        id: "decay".into(),
                        name: "Dec".into(),
                        value: 0.3,
                        min: 0.01,
                        max: 5.0,
                        default_value: 0.3,
                        step: Some(0.01),
                        unit: "s".into(),
                        rel_pos: (96.0, 48.0),
                    },
                ],
                graph_node_idx: Some(env_idx),
                bypassed: false,
            },
            ModularNodeInstance {
                id: "vca_1".into(),
                kind_id: "VcaNode".into(),
                display_name: "VCA Master".into(),
                category: crate::dsp_node_ui::DspNodeCategory::DynamicsMaster,
                pos: (540.0, 40.0),
                size: (140.0, 110.0),
                ports: vec![
                    ModularPort { id: "audio".into(), name: "Audio".into(), kind: ModularPortKind::AudioIn, rel_pos: (14.0, 45.0) },
                    ModularPort { id: "cv".into(), name: "CV".into(), kind: ModularPortKind::ModulationIn, rel_pos: (14.0, 75.0) },
                    ModularPort { id: "main".into(), name: "Main".into(), kind: ModularPortKind::AudioOut, rel_pos: (126.0, 60.0) },
                ],
                params: vec![
                    ModularNodeParam {
                        id: "gain".into(),
                        name: "Gain".into(),
                        value: 1.0,
                        min: 0.0,
                        max: 2.0,
                        default_value: 1.0,
                        step: Some(0.01),
                        unit: "".into(),
                        rel_pos: (46.0, 50.0),
                    },
                    ModularNodeParam {
                        id: "drive".into(),
                        name: "Drive".into(),
                        value: 0.0,
                        min: 0.0,
                        max: 2.0,
                        default_value: 0.0,
                        step: Some(0.01),
                        unit: "".into(),
                        rel_pos: (96.0, 50.0),
                    },
                ],
                graph_node_idx: Some(vca_idx),
                bypassed: false,
            },
        ];

        self.patch_cords = vec![
            ModularPatchCord {
                from_node_id: "osc_1".into(),
                from_port_id: "out".into(),
                to_node_id: "filter_1".into(),
                to_port_id: "in".into(),
                is_audio: true,
                intensity: 1.0,
            },
            ModularPatchCord {
                from_node_id: "filter_1".into(),
                from_port_id: "lp".into(),
                to_node_id: "vca_1".into(),
                to_port_id: "audio".into(),
                is_audio: true,
                intensity: 1.0,
            },
            ModularPatchCord {
                from_node_id: "env_1".into(),
                from_port_id: "cv".into(),
                to_node_id: "vca_1".into(),
                to_port_id: "cv".into(),
                is_audio: false,
                intensity: 1.0,
            },
        ];
        self.selected_patch_cord_idx = None;
        self.select_modular_node("osc_1");
    }

    pub fn add_modular_node_from_descriptor(&mut self, desc: &crate::dsp_node_ui::DspNodeDescriptor) {
        let count = self.modular_nodes.len();
        let col = (count % 4) as f32;
        let row = (count / 4) as f32;
        let x = 24.0 + col * 170.0;
        let y = 20.0 + row * 125.0;
        let node_id = format!("node_{}_{}", desc.kind_id.to_lowercase(), count + 1);

        let is_synth_source = matches!(
            desc.category,
            crate::dsp_node_ui::DspNodeCategory::Oscillator
                | crate::dsp_node_ui::DspNodeCategory::CompositeSynth
                | crate::dsp_node_ui::DspNodeCategory::AcousticPhysicalModel
                | crate::dsp_node_ui::DspNodeCategory::SamplerSlicer
        );
        let is_mod = desc.category == crate::dsp_node_ui::DspNodeCategory::Modulation;
        let is_spatial = desc.category == crate::dsp_node_ui::DspNodeCategory::SpatialSurround;
        let is_master_dyn = desc.category == crate::dsp_node_ui::DspNodeCategory::DynamicsMaster;

        let mut ports = Vec::new();
        if is_synth_source {
            ports.push(ModularPort { id: "voct".into(), name: "V/Oct".into(), kind: ModularPortKind::ModulationIn, rel_pos: (14.0, 85.0) });
            ports.push(ModularPort { id: "gate".into(), name: "Gate".into(), kind: ModularPortKind::GateIn, rel_pos: (45.0, 85.0) });
            ports.push(ModularPort { id: "mod".into(), name: "Mod".into(), kind: ModularPortKind::ModulationIn, rel_pos: (76.0, 85.0) });
            ports.push(ModularPort { id: "out".into(), name: "Out".into(), kind: ModularPortKind::AudioOut, rel_pos: (126.0, 85.0) });
        } else if is_mod {
            ports.push(ModularPort { id: "gate".into(), name: "Gate".into(), kind: ModularPortKind::GateIn, rel_pos: (14.0, 85.0) });
            ports.push(ModularPort { id: "sync".into(), name: "Sync".into(), kind: ModularPortKind::ModulationIn, rel_pos: (45.0, 85.0) });
            ports.push(ModularPort { id: "cv_out".into(), name: "CV".into(), kind: ModularPortKind::ModulationOut, rel_pos: (126.0, 85.0) });
        } else if is_spatial {
            ports.push(ModularPort { id: "in_l".into(), name: "In L".into(), kind: ModularPortKind::AudioIn, rel_pos: (14.0, 45.0) });
            ports.push(ModularPort { id: "in_r".into(), name: "In R".into(), kind: ModularPortKind::AudioIn, rel_pos: (14.0, 75.0) });
            ports.push(ModularPort { id: "pos_cv".into(), name: "Pan CV".into(), kind: ModularPortKind::ModulationIn, rel_pos: (60.0, 85.0) });
            ports.push(ModularPort { id: "out_l".into(), name: "Out L".into(), kind: ModularPortKind::AudioOut, rel_pos: (126.0, 45.0) });
            ports.push(ModularPort { id: "out_r".into(), name: "Out R".into(), kind: ModularPortKind::AudioOut, rel_pos: (126.0, 75.0) });
        } else if is_master_dyn {
            ports.push(ModularPort { id: "in_l".into(), name: "In L".into(), kind: ModularPortKind::AudioIn, rel_pos: (14.0, 45.0) });
            ports.push(ModularPort { id: "in_r".into(), name: "In R".into(), kind: ModularPortKind::AudioIn, rel_pos: (14.0, 75.0) });
            ports.push(ModularPort { id: "sidechain".into(), name: "SC CV".into(), kind: ModularPortKind::ModulationIn, rel_pos: (60.0, 85.0) });
            ports.push(ModularPort { id: "out_l".into(), name: "Out L".into(), kind: ModularPortKind::AudioOut, rel_pos: (126.0, 45.0) });
            ports.push(ModularPort { id: "out_r".into(), name: "Out R".into(), kind: ModularPortKind::AudioOut, rel_pos: (126.0, 75.0) });
        } else {
            ports.push(ModularPort { id: "in".into(), name: "In".into(), kind: ModularPortKind::AudioIn, rel_pos: (14.0, 45.0) });
            ports.push(ModularPort { id: "cv".into(), name: "CV".into(), kind: ModularPortKind::ModulationIn, rel_pos: (14.0, 75.0) });
            ports.push(ModularPort { id: "out".into(), name: "Out".into(), kind: ModularPortKind::AudioOut, rel_pos: (126.0, 60.0) });
        }

        let mut graph_node_idx = None;
        if let Ok(mut g) = self.audio_graph.lock() {
            let node_box: Box<dyn summoner_core::node::AudioNode> = match desc.category {
                crate::dsp_node_ui::DspNodeCategory::Oscillator
                | crate::dsp_node_ui::DspNodeCategory::CompositeSynth
                | crate::dsp_node_ui::DspNodeCategory::AcousticPhysicalModel => {
                    Box::new(summoner_core::node::SineOscillatorNode::new(440.0))
                }
                _ => Box::new(summoner_core::node::GainNode::new(1.0)),
            };
            graph_node_idx = Some(g.add_node(node_box));
        }

        let mut params = Vec::new();
        for (p_i, schema) in desc.params.iter().take(2).enumerate() {
            let (min, max, def, unit) = match &schema.widget {
                crate::dsp_node_ui::DspWidgetKind::RotaryKnob { min, max, default, unit, .. } => (*min, *max, *default, unit.clone()),
                crate::dsp_node_ui::DspWidgetKind::VerticalFader { min, max, default, unit, .. } => (*min, *max, *default, unit.clone()),
                crate::dsp_node_ui::DspWidgetKind::Toggle { default } => (0.0, 1.0, if *default { 1.0 } else { 0.0 }, "".to_string()),
                _ => (0.0, 1.0, 0.5, "".to_string()),
            };
            let x_pos = if desc.params.len() == 1 {
                70.0
            } else if p_i == 0 {
                46.0
            } else {
                98.0
            };
            params.push(ModularNodeParam {
                id: schema.id.clone(),
                name: schema.name.clone(),
                value: def,
                min,
                max,
                default_value: def,
                step: None,
                unit,
                rel_pos: (x_pos, 48.0),
            });
        }

        let instance = ModularNodeInstance {
            id: node_id.clone(),
            kind_id: desc.kind_id.clone(),
            display_name: desc.display_name.clone(),
            category: desc.category,
            pos: (x, y),
            size: (145.0, 105.0),
            ports,
            params,
            graph_node_idx,
            bypassed: false,
        };
        let n_id = node_id.clone();
        self.modular_nodes.push(instance);
        self.select_modular_node(&n_id);
    }

    /// Remove a modular node from the canvas, disconnect all its cables, and update the graph.
    pub fn remove_modular_node(&mut self, node_id: &str) -> bool {
        if let Some(pos) = self.modular_nodes.iter().position(|n| n.id == node_id) {
            let node = self.modular_nodes.remove(pos);

            // Disconnect and remove all cords connected to this node
            let cords_to_remove: Vec<ModularPatchCord> = self.patch_cords.iter()
                .filter(|c| c.from_node_id == node_id || c.to_node_id == node_id)
                .cloned()
                .collect();
            for c in cords_to_remove {
                self.disconnect_patch_cord_in_graph(&c.from_node_id, &c.from_port_id, &c.to_node_id, &c.to_port_id);
            }
            self.patch_cords.retain(|c| c.from_node_id != node_id && c.to_node_id != node_id);
            if let Some(sel) = self.selected_patch_cord_idx {
                if sel >= self.patch_cords.len() {
                    self.selected_patch_cord_idx = None;
                }
            }

            // Clean up graph node edges if present
            if let Some(g_idx) = node.graph_node_idx {
                if let Ok(mut g) = self.audio_graph.lock() {
                    let edges_to_remove: Vec<summoner_core::graph::Edge> = g.edges
                        .iter()
                        .filter(|e| e.from_node == g_idx || e.to_node == g_idx)
                        .cloned()
                        .collect();
                    for e in edges_to_remove {
                        g.remove_edge(e);
                    }
                }
            }

            // Update selection if the removed node was selected
            if self.selected_modular_node_id.as_deref() == Some(node_id) {
                self.selected_modular_node_id = self.modular_nodes.first().map(|n| n.id.clone());
                if let Some(ref first_id) = self.selected_modular_node_id {
                    if let Some(first_node) = self.modular_nodes.iter().find(|n| &n.id == first_id) {
                        self.device_rack_state.selected_node_kind = Some(first_node.kind_id.clone());
                        self.device_rack_state.device_name = first_node.display_name.clone();
                        self.inspector_state.selected_node_kind = Some(first_node.kind_id.clone());
                        self.inspector_state.target_name = first_node.display_name.clone();
                    }
                } else {
                    self.device_rack_state.selected_node_kind = None;
                    self.inspector_state.selected_node_kind = None;
                }
            }
            true
        } else {
            false
        }
    }

    /// Duplicate an existing modular node with an offset position.
    pub fn duplicate_modular_node(&mut self, node_id: &str) -> Option<String> {
        let source_node = self.modular_nodes.iter().find(|n| n.id == node_id)?.clone();
        let new_id = format!("{}_copy_{}", node_id, self.modular_nodes.len() + 1);
        let new_pos = (
            (source_node.pos.0 + 25.0).clamp(0.0, 1200.0),
            (source_node.pos.1 + 25.0).clamp(0.0, 800.0),
        );

        let mut graph_node_idx = None;
        if let Ok(mut g) = self.audio_graph.lock() {
            let node_box: Box<dyn summoner_core::node::AudioNode> = match source_node.category {
                crate::dsp_node_ui::DspNodeCategory::Oscillator
                | crate::dsp_node_ui::DspNodeCategory::CompositeSynth
                | crate::dsp_node_ui::DspNodeCategory::AcousticPhysicalModel => {
                    Box::new(summoner_core::node::SineOscillatorNode::new(440.0))
                }
                _ => Box::new(summoner_core::node::GainNode::new(1.0)),
            };
            graph_node_idx = Some(g.add_node(node_box));
        }

        let new_instance = ModularNodeInstance {
            id: new_id.clone(),
            kind_id: source_node.kind_id.clone(),
            display_name: format!("{} (Copy)", source_node.display_name),
            category: source_node.category,
            pos: new_pos,
            size: source_node.size,
            ports: source_node.ports.clone(),
            params: source_node.params.clone(),
            graph_node_idx,
            bypassed: source_node.bypassed,
        };

        self.selected_modular_node_id = Some(new_id.clone());
        self.device_rack_state.selected_node_kind = Some(new_instance.kind_id.clone());
        self.device_rack_state.device_name = new_instance.display_name.clone();
        self.inspector_state.selected_node_kind = Some(new_instance.kind_id.clone());
        self.inspector_state.target_name = new_instance.display_name.clone();
        self.modular_nodes.push(new_instance);
        Some(new_id)
    }

    /// Toggle the bypass state of a modular node.
    pub fn toggle_bypass_modular_node(&mut self, node_id: &str) -> Option<bool> {
        let node = self.modular_nodes.iter_mut().find(|n| n.id == node_id)?;
        node.bypassed = !node.bypassed;
        Some(node.bypassed)
    }

    /// Dynamically connect two modular nodes with an audio or CV cord in the underlying NodeGraph.
    pub fn connect_patch_cord_in_graph(&mut self, src_node_id: &str, src_port_id: &str, dst_node_id: &str, dst_port_id: &str) {
        let from_node = self.modular_nodes.iter().find(|n| n.id == src_node_id);
        let to_node = self.modular_nodes.iter().find(|n| n.id == dst_node_id);

        if let (Some(from), Some(to)) = (from_node, to_node) {
            if let (Some(f_idx), Some(t_idx)) = (from.graph_node_idx, to.graph_node_idx) {
                let from_port_idx = from.ports.iter().position(|p| p.id == src_port_id).unwrap_or(0);
                let to_port_idx = to.ports.iter().position(|p| p.id == dst_port_id).unwrap_or(0);

                if let Ok(mut g) = self.audio_graph.lock() {
                    g.connect(f_idx, from_port_idx, t_idx, to_port_idx);
                }
            }
        }
    }

    /// Disconnect an edge from the underlying NodeGraph.
    pub fn disconnect_patch_cord_in_graph(&mut self, src_node_id: &str, src_port_id: &str, dst_node_id: &str, dst_port_id: &str) {
        let from_node = self.modular_nodes.iter().find(|n| n.id == src_node_id);
        let to_node = self.modular_nodes.iter().find(|n| n.id == dst_node_id);

        if let (Some(from), Some(to)) = (from_node, to_node) {
            if let (Some(f_idx), Some(t_idx)) = (from.graph_node_idx, to.graph_node_idx) {
                let from_port_idx = from.ports.iter().position(|p| p.id == src_port_id).unwrap_or(0);
                let to_port_idx = to.ports.iter().position(|p| p.id == dst_port_id).unwrap_or(0);

                if let Ok(mut g) = self.audio_graph.lock() {
                    g.remove_edge(summoner_core::graph::Edge {
                        from_node: f_idx,
                        from_port: from_port_idx,
                        to_node: t_idx,
                        to_port: to_port_idx,
                    });
                }
            }
        }
    }

    /// Set a patch cord's intensity / attenuation factor (-1.0..=1.0).
    pub fn set_patch_cord_intensity(&mut self, cord_idx: usize, intensity: f32) -> bool {
        if let Some(cord) = self.patch_cords.get_mut(cord_idx) {
            cord.intensity = intensity.clamp(-1.0, 1.0);
            true
        } else {
            false
        }
    }

    /// Toggle patch cord polarity (+ / -) or invert its intensity.
    pub fn toggle_patch_cord_polarity(&mut self, cord_idx: usize) -> Option<f32> {
        if let Some(cord) = self.patch_cords.get_mut(cord_idx) {
            cord.intensity = -cord.intensity;
            Some(cord.intensity)
        } else {
            None
        }
    }

    /// Remove a patch cord by index, disconnecting it from the underlying graph and updating selection.
    pub fn remove_patch_cord_by_index(&mut self, cord_idx: usize) -> bool {
        if cord_idx < self.patch_cords.len() {
            let cord = self.patch_cords.remove(cord_idx);
            self.disconnect_patch_cord_in_graph(&cord.from_node_id, &cord.from_port_id, &cord.to_node_id, &cord.to_port_id);
            if self.selected_patch_cord_idx == Some(cord_idx) {
                self.selected_patch_cord_idx = None;
            } else if let Some(sel) = self.selected_patch_cord_idx {
                if sel > cord_idx {
                    self.selected_patch_cord_idx = Some(sel - 1);
                }
            }
            true
        } else {
            false
        }
    }

    /// Get reference to currently selected patch cord, if any.
    pub fn selected_patch_cord(&self) -> Option<&ModularPatchCord> {
        self.selected_patch_cord_idx.and_then(|idx| self.patch_cords.get(idx))
    }

    /// Get mutable reference to currently selected patch cord, if any.
    pub fn selected_patch_cord_mut(&mut self) -> Option<&mut ModularPatchCord> {
        self.selected_patch_cord_idx.and_then(|idx| self.patch_cords.get_mut(idx))
    }

    /// Set a modular node's parameter value by node_id and param_id.
    pub fn set_modular_node_param(&mut self, node_id: &str, param_id: &str, value: f32) -> bool {
        if let Some(node) = self.modular_nodes.iter_mut().find(|n| n.id == node_id) {
            if let Some(param) = node.params.iter_mut().find(|p| p.id == param_id) {
                param.value = value.clamp(param.min, param.max);
                if self.selected_modular_node_id.as_deref() == Some(node_id) {
                    self.inspector_state.node_param_values.insert(param_id.to_string(), param.value);
                    self.device_rack_state.node_param_values.insert(param_id.to_string(), param.value);
                }
                return true;
            }
        }
        false
    }

    /// Get a modular node's parameter value by node_id and param_id.
    pub fn get_modular_node_param(&self, node_id: &str, param_id: &str) -> Option<f32> {
        let node = self.modular_nodes.iter().find(|n| n.id == node_id)?;
        let param = node.params.iter().find(|p| p.id == param_id)?;
        Some(param.value)
    }

    /// Reset a modular node's parameter to its default value.
    pub fn reset_modular_node_param(&mut self, node_id: &str, param_id: &str) -> Option<f32> {
        let node = self.modular_nodes.iter_mut().find(|n| n.id == node_id)?;
        let param = node.params.iter_mut().find(|p| p.id == param_id)?;
        param.value = param.default_value;
        let def = param.default_value;
        if self.selected_modular_node_id.as_deref() == Some(node_id) {
            self.inspector_state.node_param_values.insert(param_id.to_string(), def);
            self.device_rack_state.node_param_values.insert(param_id.to_string(), def);
        }
        Some(def)
    }

    /// Programmatically select a modular node by id, synchronizing its descriptor and parameters
    /// into both the Pro Inspector and Device Rack drawers.
    pub fn select_modular_node(&mut self, node_id: &str) -> bool {
        if let Some(node) = self.modular_nodes.iter().find(|n| n.id == node_id).cloned() {
            self.selected_patch_cord_idx = None;
            self.selected_modular_node_id = Some(node_id.to_string());
            self.device_rack_state.selected_node_kind = Some(node.kind_id.clone());
            self.device_rack_state.device_name = node.display_name.clone();
            self.inspector_state.selected_node_kind = Some(node.kind_id);
            self.inspector_state.target_name = node.display_name;
            self.inspector_state.node_param_values.clear();
            self.device_rack_state.node_param_values.clear();
            self.last_inspector_node_param_values.clear();
            self.last_device_rack_node_param_values.clear();
            for p in &node.params {
                self.inspector_state.node_param_values.insert(p.id.clone(), p.value);
                self.last_inspector_node_param_values.insert(p.id.clone(), p.value);
                self.device_rack_state.node_param_values.insert(p.id.clone(), p.value);
                self.last_device_rack_node_param_values.insert(p.id.clone(), p.value);
            }
            return true;
        }
        false
    }

    /// Synchronize parameter values bidirectionally between selected modular node faceplate
    /// and the active Inspector / Device Rack parameter maps.
    pub fn sync_selected_modular_node_params(&mut self) {
        if let Some(ref sel_node_id) = self.selected_modular_node_id {
            if let Some(node) = self.modular_nodes.iter_mut().find(|n| &n.id == sel_node_id) {
                for param in &mut node.params {
                    let insp_val = self.inspector_state.node_param_values.get(&param.id).copied();
                    let last_insp = self.last_inspector_node_param_values.get(&param.id).copied();
                    let rack_val = self.device_rack_state.node_param_values.get(&param.id).copied();
                    let last_rack = self.last_device_rack_node_param_values.get(&param.id).copied();

                    // If inspector changed, it takes precedence
                    if let (Some(iv), Some(liv)) = (insp_val, last_insp) {
                        if (iv - liv).abs() > 1e-4 {
                            param.value = iv.clamp(param.min, param.max);
                        }
                    } else if let Some(iv) = insp_val {
                        if (iv - param.value).abs() > 1e-4 && last_insp.is_none() {
                            param.value = iv.clamp(param.min, param.max);
                        }
                    }

                    // If rack changed, it takes precedence
                    if let (Some(rv), Some(lrv)) = (rack_val, last_rack) {
                        if (rv - lrv).abs() > 1e-4 {
                            param.value = rv.clamp(param.min, param.max);
                        }
                    } else if let Some(rv) = rack_val {
                        if (rv - param.value).abs() > 1e-4 && last_rack.is_none() {
                            param.value = rv.clamp(param.min, param.max);
                        }
                    }

                    // Keep both maps and shadows updated to param.value
                    self.inspector_state.node_param_values.insert(param.id.clone(), param.value);
                    self.last_inspector_node_param_values.insert(param.id.clone(), param.value);
                    self.device_rack_state.node_param_values.insert(param.id.clone(), param.value);
                    self.last_device_rack_node_param_values.insert(param.id.clone(), param.value);
                }
            }
        }
    }

    /// Take any pending requested modular parameter automation lane.
    pub fn take_requested_modular_automation_param(&mut self) -> Option<String> {
        self.requested_modular_automation_param.take()
    }

    /// Open the live Bézier parameter automation editor for a specific modular node parameter.
    pub fn open_modular_automation_editor(&mut self, node_id: &str, param_id: &str) -> bool {
        if let Some(node) = self.modular_nodes.iter().find(|n| n.id == node_id) {
            if let Some(param) = node.params.iter().find(|p| p.id == param_id) {
                let lane_key = format!("modular_{}_{}", node.id, param.id);
                self.requested_modular_automation_param = Some(lane_key.clone());
                self.show_automation_editor_window = true;
                let title = format!("{} — {}", node.display_name, param.name);
                let mut editor = crate::views::bezier_automation_editor::BezierAutomationEditorView::new(
                    title,
                    &param.unit,
                    param.min,
                    param.max,
                    16.0,
                );
                let norm = if (param.max - param.min).abs() > 1e-4 {
                    ((param.value - param.min) / (param.max - param.min)).clamp(0.0, 1.0)
                } else {
                    0.5
                };
                editor.nodes.clear();
                editor.nodes.push(crate::views::bezier_automation_editor::AutomationNode::new(
                    "start",
                    0.0,
                    norm,
                    crate::views::bezier_automation_editor::AutomationCurveType::Bezier {
                        handle_out_y: norm,
                        handle_in_y: norm,
                    },
                ));
                editor.nodes.push(crate::views::bezier_automation_editor::AutomationNode::new(
                    "end",
                    16.0,
                    norm,
                    crate::views::bezier_automation_editor::AutomationCurveType::Linear,
                ));
                self.automation_editor = Some(editor);
                return true;
            }
        }
        false
    }

    /// Close the modular parameter automation editor window.
    pub fn close_modular_automation_editor(&mut self) {
        self.show_automation_editor_window = false;
    }

    /// Open the live Bézier automation editor for a modular patch cord's intensity attenuator.
    pub fn open_patch_cord_automation_editor(&mut self, cord_idx: usize) -> bool {
        if cord_idx >= self.patch_cords.len() {
            return false;
        }
        let cord = &self.patch_cords[cord_idx];
        let lane_key = format!(
            "modular_cord_{}_{}_{}_{}_intensity",
            cord.from_node_id, cord.from_port_id, cord.to_node_id, cord.to_port_id
        );
        self.requested_modular_automation_param = Some(lane_key);
        self.show_automation_editor_window = true;
        let title = format!(
            "Cable: {}:{} ➔ {}:{} Attenuation",
            cord.from_node_id, cord.from_port_id, cord.to_node_id, cord.to_port_id
        );
        let mut editor = crate::views::bezier_automation_editor::BezierAutomationEditorView::new(
            title,
            "%",
            -1.0,
            1.0,
            16.0,
        );
        let norm = ((cord.intensity + 1.0) / 2.0).clamp(0.0, 1.0);
        editor.nodes.clear();
        editor.nodes.push(crate::views::bezier_automation_editor::AutomationNode::new(
            "start",
            0.0,
            norm,
            crate::views::bezier_automation_editor::AutomationCurveType::Bezier {
                handle_out_y: norm,
                handle_in_y: norm,
            },
        ));
        editor.nodes.push(crate::views::bezier_automation_editor::AutomationNode::new(
            "end",
            16.0,
            norm,
            crate::views::bezier_automation_editor::AutomationCurveType::Linear,
        ));
        self.automation_editor = Some(editor);
        true
    }

    /// Apply an instant quick shape to the active automation curve.
    pub fn apply_automation_quick_shape(&mut self, shape: AutomationQuickShape) {
        if let Some(ref mut editor) = self.automation_editor {
            match shape {
                AutomationQuickShape::Flat => {
                    let cur_norm = editor.nodes.first().map(|n| n.value).unwrap_or(0.5);
                    editor.nodes = vec![
                        crate::views::bezier_automation_editor::AutomationNode::new("n0", 0.0, cur_norm, crate::views::bezier_automation_editor::AutomationCurveType::Linear),
                        crate::views::bezier_automation_editor::AutomationNode::new("n1", editor.total_beats, cur_norm, crate::views::bezier_automation_editor::AutomationCurveType::Linear),
                    ];
                }
                AutomationQuickShape::RampUp => {
                    editor.nodes = vec![
                        crate::views::bezier_automation_editor::AutomationNode::new("n0", 0.0, 0.0, crate::views::bezier_automation_editor::AutomationCurveType::Linear),
                        crate::views::bezier_automation_editor::AutomationNode::new("n1", editor.total_beats, 1.0, crate::views::bezier_automation_editor::AutomationCurveType::Linear),
                    ];
                }
                AutomationQuickShape::RampDown => {
                    editor.nodes = vec![
                        crate::views::bezier_automation_editor::AutomationNode::new("n0", 0.0, 1.0, crate::views::bezier_automation_editor::AutomationCurveType::Linear),
                        crate::views::bezier_automation_editor::AutomationNode::new("n1", editor.total_beats, 0.0, crate::views::bezier_automation_editor::AutomationCurveType::Linear),
                    ];
                }
                AutomationQuickShape::SineLfo => {
                    editor.nodes.clear();
                    let steps = 8;
                    for s in 0..=steps {
                        let beat = (s as f64 / steps as f64) * editor.total_beats;
                        let phase = (s as f32 / steps as f32) * std::f32::consts::TAU * 2.0;
                        let val = 0.5 + 0.5 * phase.sin();
                        editor.nodes.push(crate::views::bezier_automation_editor::AutomationNode::new(
                            format!("n{}", s),
                            beat,
                            val,
                            crate::views::bezier_automation_editor::AutomationCurveType::Bezier {
                                handle_out_y: val,
                                handle_in_y: val,
                            },
                        ));
                    }
                }
                AutomationQuickShape::ExpDrop => {
                    editor.nodes = vec![
                        crate::views::bezier_automation_editor::AutomationNode::new("n0", 0.0, 1.0, crate::views::bezier_automation_editor::AutomationCurveType::Exponential { tension: 0.7 }),
                        crate::views::bezier_automation_editor::AutomationNode::new("n1", editor.total_beats, 0.05, crate::views::bezier_automation_editor::AutomationCurveType::Linear),
                    ];
                }
                AutomationQuickShape::SCurve => {
                    editor.nodes = vec![
                        crate::views::bezier_automation_editor::AutomationNode::new("n0", 0.0, 0.0, crate::views::bezier_automation_editor::AutomationCurveType::Bezier { handle_out_y: 0.1, handle_in_y: 0.9 }),
                        crate::views::bezier_automation_editor::AutomationNode::new("n1", editor.total_beats, 1.0, crate::views::bezier_automation_editor::AutomationCurveType::Linear),
                    ];
                }
                AutomationQuickShape::Invert => {
                    for node in &mut editor.nodes {
                        node.value = (1.0 - node.value).clamp(0.0, 1.0);
                    }
                }
                AutomationQuickShape::Smooth => {
                    for node in &mut editor.nodes {
                        if let crate::views::bezier_automation_editor::AutomationCurveType::Linear = node.curve {
                            node.curve = crate::views::bezier_automation_editor::AutomationCurveType::Bezier {
                                handle_out_y: node.value,
                                handle_in_y: node.value,
                            };
                        }
                    }
                }
            }
        }
    }

    /// Open the live Bézier parameter automation editor for a specific track parameter (e.g. "gain", "pan", "mute", "solo", "cutoff").
    pub fn open_track_automation_editor(&mut self, track_id: u64, param_name: &str) -> bool {
        if let Some(track) = self.tracks.iter().find(|t| t.id == track_id) {
            let (lane_key, title, min, max, unit, norm) = match param_name {
                "gain" => (
                    format!("track_{}_gain", track_id),
                    format!("{} — Volume Gain", track.name),
                    0.0_f32,
                    1.5_f32,
                    "x".to_string(),
                    (track.gain / 1.5).clamp(0.0, 1.0),
                ),
                "pan" => (
                    format!("track_{}_pan", track_id),
                    format!("{} — Stereo Pan", track.name),
                    -1.0_f32,
                    1.0_f32,
                    "".to_string(),
                    ((track.pan + 1.0) * 0.5).clamp(0.0, 1.0),
                ),
                "mute" => (
                    format!("track_{}_mute", track_id),
                    format!("{} — Mute", track.name),
                    0.0_f32,
                    1.0_f32,
                    "".to_string(),
                    if track.is_muted { 1.0 } else { 0.0 },
                ),
                "solo" => (
                    format!("track_{}_solo", track_id),
                    format!("{} — Solo", track.name),
                    0.0_f32,
                    1.0_f32,
                    "".to_string(),
                    if track.is_soloed { 1.0 } else { 0.0 },
                ),
                "cutoff" => (
                    format!("track_{}_cutoff", track_id),
                    format!("{} — Filter Cutoff", track.name),
                    20.0_f32,
                    20000.0_f32,
                    "Hz".to_string(),
                    self.device_rack_state.cutoff.clamp(0.0, 1.0),
                ),
                "resonance" => (
                    format!("track_{}_resonance", track_id),
                    format!("{} — Filter Resonance", track.name),
                    0.1_f32,
                    10.0_f32,
                    "Q".to_string(),
                    self.device_rack_state.resonance.clamp(0.0, 1.0),
                ),
                "decay" => (
                    format!("track_{}_decay", track_id),
                    format!("{} — Envelope Decay", track.name),
                    0.01_f32,
                    5.0_f32,
                    "s".to_string(),
                    self.device_rack_state.decay.clamp(0.0, 1.0),
                ),
                "drive" => (
                    format!("track_{}_drive", track_id),
                    format!("{} — Saturation Drive", track.name),
                    0.0_f32,
                    2.0_f32,
                    "x".to_string(),
                    self.device_rack_state.drive.clamp(0.0, 1.0),
                ),
                "mod" | "mod_amt" => (
                    format!("track_{}_mod_amt", track_id),
                    format!("{} — Modulation Depth", track.name),
                    0.0_f32,
                    1.0_f32,
                    "%".to_string(),
                    self.device_rack_state.mod_amt.clamp(0.0, 1.0),
                ),
                "volume" => (
                    format!("track_{}_volume", track_id),
                    format!("{} — Track Volume", track.name),
                    0.0_f32,
                    1.5_f32,
                    "x".to_string(),
                    self.device_rack_state.volume.clamp(0.0, 1.0),
                ),
                "osc_mix" => (
                    format!("track_{}_osc_mix", track_id),
                    format!("{} — Oscillator Mix", track.name),
                    0.0_f32,
                    1.0_f32,
                    "%".to_string(),
                    self.device_rack_state.osc_mix.clamp(0.0, 1.0),
                ),
                "shape" => (
                    format!("track_{}_shape", track_id),
                    format!("{} — Oscillator Shape", track.name),
                    0.0_f32,
                    1.0_f32,
                    "%".to_string(),
                    self.device_rack_state.shape.clamp(0.0, 1.0),
                ),
                "pitch" | "pitch_bend" => (
                    format!("track_{}_pitch", track_id),
                    format!("{} — Pitch Bend", track.name),
                    -12.0_f32,
                    12.0_f32,
                    "st".to_string(),
                    0.5_f32,
                ),
                "velocity" => (
                    format!("track_{}_velocity", track_id),
                    format!("{} — Note Velocity", track.name),
                    0.0_f32,
                    1.0_f32,
                    "%".to_string(),
                    self.selected_note_id
                        .and_then(|sid| self.piano_roll_notes.iter().find(|n| n.id == sid))
                        .map(|n| n.velocity)
                        .unwrap_or(0.85),
                ),
                "expression" => (
                    format!("track_{}_expression", track_id),
                    format!("{} — Expression", track.name),
                    0.0_f32,
                    1.0_f32,
                    "%".to_string(),
                    1.0_f32,
                ),
                other_name => {
                    let registry = crate::dsp_node_ui::DspNodeRegistry::new();
                    let cur_kind = self.device_rack_state.selected_node_kind.as_deref().unwrap_or("AetherSynth");
                    if let Some(desc) = registry.get(cur_kind) {
                        if let Some(param) = desc.params.iter().find(|p| p.id == other_name) {
                            let (min_v, max_v, unit_s) = match &param.widget {
                                crate::dsp_node_ui::DspWidgetKind::RotaryKnob { min, max, unit, .. } => (*min, *max, unit.clone()),
                                crate::dsp_node_ui::DspWidgetKind::VerticalFader { min, max, unit, .. } => (*min, *max, unit.clone()),
                                crate::dsp_node_ui::DspWidgetKind::Toggle { .. } => (0.0, 1.0, "state".to_string()),
                                crate::dsp_node_ui::DspWidgetKind::EnumChoice { .. } => (0.0, 1.0, "enum".to_string()),
                                _ => (0.0, 1.0, String::new()),
                            };
                            let norm_v = self.device_rack_state.node_param_values.get(other_name).copied()
                                .map(|v| ((v - min_v) / (max_v - min_v).max(1e-5)).clamp(0.0, 1.0))
                                .unwrap_or(0.5);
                            (
                                format!("track_{}_{}", track_id, param.id),
                                format!("{} — {}", track.name, param.name),
                                min_v,
                                max_v,
                                unit_s,
                                norm_v,
                            )
                        } else {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
            };
            self.requested_modular_automation_param = Some(lane_key);
            self.show_automation_editor_window = true;
            let mut editor = crate::views::bezier_automation_editor::BezierAutomationEditorView::new(
                title,
                unit,
                min,
                max,
                16.0,
            );
            editor.nodes.clear();
            editor.nodes.push(crate::views::bezier_automation_editor::AutomationNode::new(
                "start",
                0.0,
                norm,
                crate::views::bezier_automation_editor::AutomationCurveType::Linear,
            ));
            editor.nodes.push(crate::views::bezier_automation_editor::AutomationNode::new(
                "end",
                16.0,
                norm,
                crate::views::bezier_automation_editor::AutomationCurveType::Linear,
            ));
            self.automation_editor = Some(editor);
            return true;
        }
        false
    }

    /// Open the live Bézier parameter automation editor for master bus volume gain.
    pub fn open_master_automation_editor(&mut self) -> bool {
        let lane_key = "master_gain".to_string();
        let title = "Master Bus — Gain".to_string();
        let norm = (self.top_bar_state.master_gain / 2.0).clamp(0.0, 1.0);
        self.requested_modular_automation_param = Some(lane_key);
        self.show_automation_editor_window = true;
        let mut editor = crate::views::bezier_automation_editor::BezierAutomationEditorView::new(
            title,
            "x",
            0.0,
            2.0,
            16.0,
        );
        editor.nodes.clear();
        editor.nodes.push(crate::views::bezier_automation_editor::AutomationNode::new(
            "start",
            0.0,
            norm,
            crate::views::bezier_automation_editor::AutomationCurveType::Linear,
        ));
        editor.nodes.push(crate::views::bezier_automation_editor::AutomationNode::new(
            "end",
            16.0,
            norm,
            crate::views::bezier_automation_editor::AutomationCurveType::Linear,
        ));
        self.automation_editor = Some(editor);
        true
    }

    /// Launch a live scene matrix row (0..4) in the Stage view with beat positioning and playback trigger across all tracks.
    pub fn launch_scene(&mut self, scene_idx: usize) -> bool {
        if scene_idx < 4 {
            self.active_scene_idx = Some(scene_idx);
            for tr in &mut self.tracks {
                tr.active_clip_idx = Some(scene_idx);
            }
            self.top_bar_state.is_playing = true;
            self.last_synced_is_playing = true;
            self.panic_triggered = false;
            self.playhead_beat = (scene_idx as f32) * 16.0;
            true
        } else {
            false
        }
    }

    /// Stop the active scene, reset all track playing clips, and pause transport in the Stage view.
    pub fn stop_scene(&mut self) {
        self.active_scene_idx = None;
        for tr in &mut self.tracks {
            tr.active_clip_idx = None;
        }
        self.top_bar_state.is_playing = false;
        self.last_synced_is_playing = false;
    }

    /// Launch or toggle an individual track clip in the Stage matrix.
    pub fn launch_track_clip(&mut self, track_idx: usize, clip_idx: usize) -> bool {
        if track_idx < self.tracks.len() && clip_idx < 4 {
            self.select_track_for_stage(track_idx);
            let is_already_active = self.tracks[track_idx].active_clip_idx == Some(clip_idx);
            if is_already_active {
                self.tracks[track_idx].active_clip_idx = None;
            } else {
                self.tracks[track_idx].active_clip_idx = Some(clip_idx);
            }
            self.top_bar_state.is_playing = true;
            self.last_synced_is_playing = true;
            self.panic_triggered = false;

            // If all tracks are playing the same clip, reflect as active scene
            if self.tracks.iter().all(|t| t.active_clip_idx == Some(clip_idx)) {
                self.active_scene_idx = Some(clip_idx);
            } else {
                self.active_scene_idx = None;
            }
            true
        } else {
            false
        }
    }

    /// Stop a specific track's playing clip in the Stage view.
    pub fn stop_track_clip(&mut self, track_idx: usize) -> bool {
        if track_idx < self.tracks.len() {
            self.tracks[track_idx].active_clip_idx = None;
            self.active_scene_idx = None;
            true
        } else {
            false
        }
    }

    /// Stop all active clips across all tracks and reset active scene in the Stage view.
    pub fn stop_all_clips(&mut self) {
        self.active_scene_idx = None;
        for tr in &mut self.tracks {
            tr.active_clip_idx = None;
        }
        self.top_bar_state.is_playing = false;
        self.last_synced_is_playing = false;
    }

    /// Tap tempo engine calculating BPM from consecutive taps with bounds clamping [40.0, 280.0].
    pub fn tap_tempo(&mut self) {
        let now = std::time::Instant::now();
        if let Some(prev) = self.last_tap_time {
            let elapsed_sec = now.duration_since(prev).as_secs_f64();
            if elapsed_sec > 0.15 && elapsed_sec < 2.5 {
                let computed_bpm = (60.0 / elapsed_sec).clamp(40.0, 280.0);
                self.top_bar_state.bpm = (computed_bpm * 10.0).round() / 10.0;
                self.last_synced_bpm = self.top_bar_state.bpm;
            }
        }
        self.last_tap_time = Some(now);
    }

    /// Trigger global panic killswitch: pause playback, disarm all tracks, stop clips, and flag panic.
    pub fn trigger_panic(&mut self) {
        self.top_bar_state.is_playing = false;
        self.last_synced_is_playing = false;
        self.active_scene_idx = None;
        for tr in &mut self.tracks {
            tr.is_armed = false;
            tr.active_clip_idx = None;
        }
        self.panic_triggered = true;
    }

    /// Set a track's stereo pan value with bounds clamping [-1.0, 1.0].
    pub fn set_track_pan(&mut self, track_idx: usize, pan: f32) -> bool {
        if let Some(track) = self.tracks.get_mut(track_idx) {
            track.pan = pan.clamp(-1.0, 1.0);
            if self.selected_track_idx == track_idx {
                self.inspector_state.pan_val = track.pan;
            }
            return true;
        }
        false
    }

    /// Reset a track's stereo pan value to center (0.0).
    pub fn reset_track_pan(&mut self, track_idx: usize) -> bool {
        self.set_track_pan(track_idx, 0.0)
    }

    /// Reset a track's volume gain to unity (1.0 = 0.0 dB).
    pub fn reset_track_gain(&mut self, track_idx: usize) -> bool {
        if let Some(track) = self.tracks.get_mut(track_idx) {
            track.gain = 1.0;
            if self.selected_track_idx == track_idx {
                self.inspector_state.gain_db = 0.0;
            }
            return true;
        }
        false
    }

    /// Reset master volume gain to unity (1.0 = 0.0 dB).
    pub fn reset_master_gain(&mut self) {
        self.top_bar_state.master_gain = 1.0;
        self.is_master_muted = false;
    }

    /// Toggle master output mute state, muting to 0.0 and restoring previous gain on unmute.
    pub fn toggle_master_mute(&mut self) -> bool {
        if self.is_master_muted {
            self.top_bar_state.master_gain = self.unmuted_master_gain;
            self.is_master_muted = false;
        } else {
            self.unmuted_master_gain = self.top_bar_state.master_gain.max(0.1);
            self.top_bar_state.master_gain = 0.0;
            self.is_master_muted = true;
        }
        self.is_master_muted
    }

    /// Toggle master output mono summing audition mode.
    pub fn toggle_master_mono(&mut self) -> bool {
        self.is_master_mono = !self.is_master_mono;
        self.is_master_mono
    }

    /// Reset all track channel faders to unity gain (1.0 = 0.0 dB) and center stereo pans (0.0).
    pub fn reset_all_channel_faders(&mut self) {
        for tr in &mut self.tracks {
            tr.gain = 1.0;
            tr.pan = 0.0;
        }
        if self.selected_track_idx < self.tracks.len() {
            self.inspector_state.gain_db = 0.0;
            self.inspector_state.pan_val = 0.0;
        }
    }

    /// Toggle mute across all tracks. If any track is unmuted, mute all; otherwise unmute all.
    pub fn toggle_all_tracks_mute(&mut self) -> bool {
        let any_unmuted = self.tracks.iter().any(|t| !t.is_muted);
        for tr in &mut self.tracks {
            tr.is_muted = any_unmuted;
        }
        if self.selected_track_idx < self.tracks.len() {
            self.inspector_state.is_muted = any_unmuted;
        }
        any_unmuted
    }

    /// Synchronize all modular node sockets and active patch cords into the PatchMatrixView.
    pub fn sync_modular_to_routing_matrix(&mut self) {
        let mut sources = Vec::new();
        let mut destinations = Vec::new();

        for node in &self.modular_nodes {
            let (r, g, b) = node.category.color_rgb();
            for port in &node.ports {
                let id = format!("{}:{}", node.id, port.id);
                let name = format!("{}: {}", node.display_name, port.name);
                if port.kind.is_output() {
                    let kind = if port.kind.is_modulation() {
                        crate::patch_matrix::SourceKind::Envelope
                    } else {
                        crate::patch_matrix::SourceKind::Custom("Audio".into())
                    };
                    sources.push(crate::patch_matrix::SourceNode::new(id, name, kind).with_rgb(r, g, b));
                } else {
                    destinations.push(crate::patch_matrix::DestNode::new(id, name, node.display_name.clone()));
                }
            }
        }

        self.patch_matrix.sources = sources;
        self.patch_matrix.destinations = destinations;
        self.patch_matrix.connections.clear();

        for cord in &self.patch_cords {
            let key = (
                format!("{}:{}", cord.from_node_id, cord.from_port_id),
                format!("{}:{}", cord.to_node_id, cord.to_port_id),
            );
            self.patch_matrix.connections.insert(key, crate::patch_matrix::PatchConnection {
                active: true,
                intensity: cord.intensity.abs(),
                muted: cord.intensity.abs() < 0.001,
                inverted: cord.intensity < -0.001,
            });
        }
    }

    /// Synchronize active connections from PatchMatrixView back into modular patch cords and audio graph.
    pub fn sync_routing_matrix_to_modular(&mut self) {
        let conns: Vec<((String, String), crate::patch_matrix::PatchConnection)> = self.patch_matrix.connections.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        for ((src, dst), conn) in conns {
            let src_parts: Vec<&str> = src.split(':').collect();
            let dst_parts: Vec<&str> = dst.split(':').collect();
            if src_parts.len() == 2 && dst_parts.len() == 2 {
                let (s_node, s_port) = (src_parts[0], src_parts[1]);
                let (d_node, d_port) = (dst_parts[0], dst_parts[1]);
                let intensity = if conn.inverted { -conn.intensity } else { conn.intensity };

                if conn.active {
                    if let Some(cord) = self.patch_cords.iter_mut().find(|c| {
                        c.from_node_id == s_node && c.from_port_id == s_port && c.to_node_id == d_node && c.to_port_id == d_port
                    }) {
                        cord.intensity = intensity;
                    } else {
                        let is_audio = self.modular_nodes.iter()
                            .find(|n| n.id == s_node)
                            .and_then(|n| n.ports.iter().find(|p| p.id == s_port))
                            .map(|p| !p.kind.is_modulation())
                            .unwrap_or(true);
                        self.patch_cords.push(ModularPatchCord {
                            from_node_id: s_node.to_string(),
                            from_port_id: s_port.to_string(),
                            to_node_id: d_node.to_string(),
                            to_port_id: d_port.to_string(),
                            is_audio,
                            intensity,
                        });
                        self.connect_patch_cord_in_graph(s_node, s_port, d_node, d_port);
                    }
                } else if let Some(pos) = self.patch_cords.iter().position(|c| {
                    c.from_node_id == s_node && c.from_port_id == s_port && c.to_node_id == d_node && c.to_port_id == d_port
                }) {
                    self.patch_cords.remove(pos);
                    self.disconnect_patch_cord_in_graph(s_node, s_port, d_node, d_port);
                }
            }
        }
    }

    /// Run real-time block processing through the live modular NodeGraph.
    pub fn process_modular_graph(
        &self,
        input: &[&[summoner_core::audio::Sample]],
        output: &mut [&mut [summoner_core::audio::Sample]],
        ctx: &summoner_core::node::ProcessContext,
    ) {
        if let Ok(mut g) = self.audio_graph.lock() {
            g.process(input, output, ctx);
        }
    }

    /// Synchronize live session configuration, transport position, and active track between Summoner Core and the Studio GUI.
    pub fn sync_with_project(
        &mut self,
        project: &mut summoner_project::schema::ProjectConfig,
        playhead_beat: &mut f64,
        transport_running: &mut bool,
        selected_track_id: &mut Option<u64>,
    ) {
        // 1. Reconcile tracks if project has tracks
        if !project.tracks.is_empty() {
            let proj_ids: Vec<u64> = project.tracks.iter().map(|t| t.id).collect();
            let view_ids: Vec<u64> = self.tracks.iter().map(|t| t.id).collect();
            if proj_ids != view_ids {
                let default_colors = [
                    [56, 189, 248],   // Cyan
                    [168, 85, 247],   // Purple
                    [236, 72, 153],   // Pink
                    [245, 158, 11],   // Amber
                    [16, 185, 129],   // Emerald
                    [59, 130, 246],   // Blue
                    [239, 68, 68],    // Red
                    [14, 165, 233],   // Sky
                ];
                self.tracks = project.tracks.iter().enumerate().map(|(i, t)| {
                    let color = t.color.unwrap_or_else(|| default_colors[i % default_colors.len()]);
                    let (clip_start, clip_len) = if let Some(ref seq) = t.sequence {
                        (seq.start_beat as f32, seq.step_division as f32)
                    } else if let Some(clip) = t.clips.first() {
                        (clip.start_beat as f32, clip.step_division as f32)
                    } else {
                        (0.0_f32, 16.0_f32)
                    };
                    let is_audio = t.nodes.iter().any(|n| {
                        let k = n.kind.to_lowercase();
                        k.contains("audio") || k.contains("sample") || k.contains("wav")
                    });
                    let mut clips = Vec::new();
                    for (c_i, c) in t.clips.iter().enumerate() {
                        clips.push(ArrangerClipVisual {
                            id: c_i + 1,
                            name: c.clip_name.clone().unwrap_or_else(|| format!("{} {}", t.name, c_i + 1)),
                            start_beat: c.start_beat as f32,
                            length_beats: (c.steps.len() as f32 * c.step_division as f32).max(1.0),
                            fade_in: c.fade_in as f32,
                            fade_out: c.fade_out as f32,
                            gain: c.gain as f32,
                            is_selected: false,
                        });
                    }
                    if clips.is_empty() && clip_len > 0.0 {
                        clips.push(ArrangerClipVisual {
                            id: 1,
                            name: t.name.clone(),
                            start_beat: clip_start,
                            length_beats: clip_len,
                            fade_in: 0.0,
                            fade_out: 0.0,
                            gain: 1.0,
                            is_selected: false,
                        });
                    }
                    TrackVisualData {
                        id: t.id,
                        name: t.name.clone(),
                        color_rgb: color,
                        is_audio,
                        gain: t.gain,
                        pan: t.pan,
                        is_muted: t.muted,
                        is_soloed: t.soloed,
                        is_armed: t.record_armed,
                        clip_start_beat: clip_start,
                        clip_length_beats: clip_len,
                        clips,
                        active_clip_idx: None,
                    }
                }).collect();
            } else {
                // Bi-directional parameter sync: push view track mutations to project tracks
                for t in &mut project.tracks {
                    if let Some(vt) = self.tracks.iter().find(|v| v.id == t.id) {
                        t.gain = vt.gain;
                        t.pan = vt.pan;
                        t.muted = vt.is_muted;
                        t.soloed = vt.is_soloed;
                        t.record_armed = vt.is_armed;
                    }
                }
            }
        }

        // 2. Track selection sync
        if let Some(sel_id) = *selected_track_id {
            if let Some(pos) = self.tracks.iter().position(|t| t.id == sel_id) {
                self.selected_track_idx = pos;
            }
        } else if self.selected_track_idx < self.tracks.len() {
            *selected_track_id = Some(self.tracks[self.selected_track_idx].id);
        }

        // 3. Transport & Playhead sync
        if self.top_bar_state.is_playing != self.last_synced_is_playing {
            *transport_running = self.top_bar_state.is_playing;
            self.last_synced_is_playing = self.top_bar_state.is_playing;
        } else {
            self.top_bar_state.is_playing = *transport_running;
            self.last_synced_is_playing = *transport_running;
        }
        if *transport_running {
            self.playhead_beat = *playhead_beat as f32;
        } else {
            *playhead_beat = self.playhead_beat as f64;
        }

        // 4. Loop bounds & Tempo sync
        self.loop_start_beat = project.loop_start_beat as f32;
        self.loop_end_beat = project.loop_end_beat as f32;
        if (self.top_bar_state.bpm - self.last_synced_bpm).abs() > 0.01 {
            // User adjusted BPM in GUI top bar
            project.transport.bpm = self.top_bar_state.bpm;
            self.last_synced_bpm = self.top_bar_state.bpm;
        } else {
            // Project transport drives GUI
            self.top_bar_state.bpm = project.transport.bpm;
            self.last_synced_bpm = project.transport.bpm;
        }

        // 5. Bidirectional Piano Roll <-> Track Sequence synchronization
        if self.selected_track_idx < project.tracks.len() {
            if self.selected_track_idx != self.last_synced_track_notes_idx {
                self.last_synced_track_notes_idx = self.selected_track_idx;
                let track = &project.tracks[self.selected_track_idx];
                self.load_piano_roll_from_track(track);
            } else {
                let track = &mut project.tracks[self.selected_track_idx];
                self.sync_piano_roll_to_track(track);
            }
        }

        // 6. Bidirectional Track DSP Nodes & Parameters synchronization
        if self.selected_track_idx < project.tracks.len() {
            let track_changed = self.selected_track_idx != self.last_selected_track_idx;
            self.last_selected_track_idx = self.selected_track_idx;

            let track = &mut project.tracks[self.selected_track_idx];
            if track.nodes.is_empty() {
                let default_kind = self.device_rack_state.selected_node_kind.clone()
                    .unwrap_or_else(|| "AetherSynth".to_string());
                track.nodes.push(summoner_project::schema::NodeConfig {
                    kind: default_kind,
                    params: std::collections::HashMap::new(),
                    plugin_state: None,
                });
            }

            let first_node = &mut track.nodes[0];

            if track_changed {
                // Newly selected track: load node kind and parameters into GUI
                self.device_rack_state.selected_node_kind = Some(first_node.kind.clone());
                if first_node.params.is_empty() {
                    first_node.params.insert("cutoff".to_string(), self.device_rack_state.cutoff);
                    first_node.params.insert("resonance".to_string(), self.device_rack_state.resonance);
                    first_node.params.insert("decay".to_string(), self.device_rack_state.decay);
                    first_node.params.insert("drive".to_string(), self.device_rack_state.drive);
                    first_node.params.insert("volume".to_string(), self.device_rack_state.volume);
                }
                for (k, v) in &first_node.params {
                    self.device_rack_state.node_param_values.insert(k.clone(), *v);
                }
                if let Some(&c) = first_node.params.get("cutoff") {
                    self.device_rack_state.cutoff = c;
                }
                if let Some(&r) = first_node.params.get("resonance") {
                    self.device_rack_state.resonance = r;
                }
                if let Some(&d) = first_node.params.get("decay") {
                    self.device_rack_state.decay = d;
                }
                if let Some(&ed) = first_node.params.get("env_decay") {
                    self.device_rack_state.env_decay = ed;
                }
                if let Some(&ma) = first_node.params.get("mod_amt") {
                    self.device_rack_state.mod_amt = ma;
                }
                if let Some(&dr) = first_node.params.get("drive") {
                    self.device_rack_state.drive = dr;
                }
                if let Some(&om) = first_node.params.get("osc_mix") {
                    self.device_rack_state.osc_mix = om;
                }
                if let Some(&sh) = first_node.params.get("shape") {
                    self.device_rack_state.shape = sh;
                }
                if let Some(&v) = first_node.params.get("volume") {
                    self.device_rack_state.volume = v;
                }
                if let Some(&ls) = first_node.params.get("lfo_speed") {
                    self.device_rack_state.lfo_speed = ls;
                }
                if let Some(&ld) = first_node.params.get("lfo_depth") {
                    self.device_rack_state.lfo_depth = ld;
                }
            } else {
                // Same track: GUI edits propagate to project
                if let Some(ref gui_kind) = self.device_rack_state.selected_node_kind {
                    if &first_node.kind != gui_kind {
                        first_node.kind = gui_kind.clone();
                    }
                } else {
                    self.device_rack_state.selected_node_kind = Some(first_node.kind.clone());
                }

                // If node_param_values has newly updated values for standard dials, update them
                if let Some(&c) = self.device_rack_state.node_param_values.get("cutoff") {
                    if first_node.params.get("cutoff") != Some(&c) {
                        self.device_rack_state.cutoff = c;
                    }
                }
                if let Some(&r) = self.device_rack_state.node_param_values.get("resonance") {
                    if first_node.params.get("resonance") != Some(&r) {
                        self.device_rack_state.resonance = r;
                    }
                }
                if let Some(&dr) = self.device_rack_state.node_param_values.get("drive") {
                    if first_node.params.get("drive") != Some(&dr) {
                        self.device_rack_state.drive = dr;
                    }
                }

                // Sync GUI node_param_values to project
                for (k, v) in &self.device_rack_state.node_param_values {
                    first_node.params.insert(k.clone(), *v);
                }

                // Standard dials reflect into project
                first_node.params.insert("cutoff".to_string(), self.device_rack_state.cutoff);
                first_node.params.insert("resonance".to_string(), self.device_rack_state.resonance);
                first_node.params.insert("decay".to_string(), self.device_rack_state.decay);
                first_node.params.insert("env_decay".to_string(), self.device_rack_state.env_decay);
                first_node.params.insert("mod_amt".to_string(), self.device_rack_state.mod_amt);
                first_node.params.insert("drive".to_string(), self.device_rack_state.drive);
                first_node.params.insert("osc_mix".to_string(), self.device_rack_state.osc_mix);
                first_node.params.insert("shape".to_string(), self.device_rack_state.shape);
                first_node.params.insert("volume".to_string(), self.device_rack_state.volume);
                first_node.params.insert("lfo_speed".to_string(), self.device_rack_state.lfo_speed);
                first_node.params.insert("lfo_depth".to_string(), self.device_rack_state.lfo_depth);

                // Ensure node_param_values has dials updated
                self.device_rack_state.node_param_values.insert("cutoff".to_string(), self.device_rack_state.cutoff);
                self.device_rack_state.node_param_values.insert("resonance".to_string(), self.device_rack_state.resonance);
                self.device_rack_state.node_param_values.insert("decay".to_string(), self.device_rack_state.decay);
                self.device_rack_state.node_param_values.insert("env_decay".to_string(), self.device_rack_state.env_decay);
                self.device_rack_state.node_param_values.insert("mod_amt".to_string(), self.device_rack_state.mod_amt);
                self.device_rack_state.node_param_values.insert("drive".to_string(), self.device_rack_state.drive);
                self.device_rack_state.node_param_values.insert("osc_mix".to_string(), self.device_rack_state.osc_mix);
                self.device_rack_state.node_param_values.insert("shape".to_string(), self.device_rack_state.shape);
                self.device_rack_state.node_param_values.insert("volume".to_string(), self.device_rack_state.volume);
                self.device_rack_state.node_param_values.insert("lfo_speed".to_string(), self.device_rack_state.lfo_speed);
                self.device_rack_state.node_param_values.insert("lfo_depth".to_string(), self.device_rack_state.lfo_depth);
            }
        }
    }

    /// Loads a factory preset into the device rack, inspector, macros, and active track.
    /// If param_bus is provided, parameters are dispatched to the real-time audio thread without allocations.
    pub fn apply_factory_preset(
        &mut self,
        preset_name_or_id: &str,
        param_bus: Option<&summoner_core::param_bus::ParamBus>,
    ) -> bool {
        if let Some(preset) = crate::factory_presets::find_preset(preset_name_or_id) {
            self.top_bar_state.selected_preset = preset.name.to_string();
            self.last_applied_preset = preset.name.to_string();

            self.top_bar_state.macro_tone = preset.macro_tone;
            self.top_bar_state.macro_space = preset.macro_space;
            self.top_bar_state.macro_punch = preset.macro_punch;
            self.top_bar_state.macro_character = preset.macro_character;
            self.last_applied_macros = [
                preset.macro_tone,
                preset.macro_space,
                preset.macro_punch,
                preset.macro_character,
            ];

            self.device_rack_state.selected_node_kind = Some(preset.target_node_kind.to_string());
            self.device_rack_state.device_name = preset.name.to_string();
            self.device_rack_state.cutoff = preset.macro_tone;
            self.device_rack_state.decay = preset.macro_space;
            self.device_rack_state.drive = preset.macro_punch;
            self.device_rack_state.mod_amt = preset.macro_character;
            self.device_rack_state.node_param_values.clear();
            for &(p_name, p_val) in preset.params {
                self.device_rack_state.node_param_values.insert(p_name.to_string(), p_val);
            }

            self.inspector_state.selected_node_kind = Some(preset.target_node_kind.to_string());
            self.inspector_state.target_name = preset.name.to_string();
            self.inspector_state.node_param_values.clear();
            for &(p_name, p_val) in preset.params {
                self.inspector_state.node_param_values.insert(p_name.to_string(), p_val);
            }

            if let Some(edo) = preset.tuning_edo {
                self.inspector_state.scale_name = format!("{}-EDO Microtonal", edo);
                self.top_bar_state.key_signature = format!("{}-EDO", edo);
            }

            if let Some(track) = self.tracks.get_mut(self.selected_track_idx) {
                track.name = preset.name.to_string();
            }

            if let Some(bus) = param_bus {
                let track_id = (self.selected_track_idx + 1) as u32;
                Self::dispatch_preset_to_param_bus(bus, track_id, preset);
                bus.set(summoner_core::param_bus::ParamId(track_id * 1000 + 106), self.device_rack_state.volume);
            }
            true
        } else {
            self.top_bar_state.selected_preset = preset_name_or_id.to_string();
            self.last_applied_preset = preset_name_or_id.to_string();

            let registry = crate::dsp_node_ui::DspNodeRegistry::new();
            if let Some(desc) = registry.get(preset_name_or_id) {
                self.device_rack_state.selected_node_kind = Some(desc.kind_id.clone());
                self.device_rack_state.device_name = desc.display_name.clone();
                self.inspector_state.selected_node_kind = Some(desc.kind_id.clone());
                self.inspector_state.target_name = desc.display_name.clone();
                true
            } else {
                false
            }
        }
    }

    /// Dispatches a factory preset's macro parameters and node parameters to ParamBus
    /// without any dynamic heap allocations, safe for real-time threads.
    #[inline]
    pub fn dispatch_preset_to_param_bus(
        bus: &summoner_core::param_bus::ParamBus,
        track_id: u32,
        preset: &crate::factory_presets::FactoryPreset,
    ) {
        bus.set(summoner_core::param_bus::ParamId(track_id * 1000 + 100), preset.macro_tone);
        bus.set(summoner_core::param_bus::ParamId(track_id * 1000 + 102), preset.macro_space);
        bus.set(summoner_core::param_bus::ParamId(track_id * 1000 + 105), preset.macro_punch);

        for (p_idx, &(_p_name, p_val)) in preset.params.iter().enumerate() {
            bus.set(summoner_core::param_bus::ParamId(track_id * 1000 + p_idx as u32), p_val);
        }
    }

    /// Loads a full factory multi-track demo project template into the arranger, mixer, and device racks.
    pub fn load_factory_demo_template(&mut self, template_id: &str) -> bool {
        if let Some(template) = crate::factory_presets::find_demo_template(template_id) {
            self.top_bar_state.bpm = template.bpm;
            self.top_bar_state.key_signature = template.key_signature.to_string();
            self.top_bar_state.time_signature = template.time_signature.to_string();
            if let Some(edo) = template.tuning_edo {
                self.inspector_state.scale_name = format!("{}-EDO Microtonal", edo);
            } else {
                self.inspector_state.scale_name = template.tuning_name.to_string();
            }

            self.tracks.clear();
            self.piano_roll_notes.clear();

            for (idx, dt) in template.tracks.iter().enumerate() {
                self.tracks.push(TrackVisualData {
                    id: dt.id as u64,
                    name: dt.name.to_string(),
                    color_rgb: dt.color_rgb,
                    is_audio: dt.is_audio,
                    gain: dt.gain,
                    pan: dt.pan,
                    is_muted: false,
                    is_soloed: false,
                    is_armed: idx == 0,
                    clip_start_beat: dt.clip_start_beat as f32,
                    clip_length_beats: dt.clip_length_beats as f32,
                    clips: Vec::new(),
                    active_clip_idx: None,
                });

                if idx == 0 {
                    for (n_i, &(pitch, start, len, vel)) in dt.notes.iter().enumerate() {
                        self.piano_roll_notes.push(PianoRollNote {
                            id: n_i + 1,
                            pitch_idx: pitch as usize,
                            start_beat: start as f32,
                            length_beats: len as f32,
                            velocity: vel,
                        });
                    }
                    self.next_note_id = dt.notes.len() + 1;
                    if !self.piano_roll_notes.is_empty() {
                        self.selected_note_id = Some(1);
                    }
                }
            }

            self.selected_track_idx = 0;
            self.last_selected_track_idx = 0;
            self.playhead_beat = 0.0;

            if let Some(first_track) = template.tracks.first() {
                let saved_track_name = self.tracks[0].name.clone();
                self.apply_factory_preset(first_track.preset_id, None);
                self.tracks[0].name = saved_track_name;
                self.top_bar_state.key_signature = template.key_signature.to_string();
            }
            true
        } else {
            false
        }
    }

    /// Step 33.1: Real-time lock-free parameter automation bridge connecting GUI controls,
    /// device rack dials, and automation timelines directly to ParamBus for live streaming.
    pub fn sync_with_param_bus(
        &mut self,
        project: &summoner_project::schema::ProjectConfig,
        param_bus: &summoner_core::param_bus::ParamBus,
        automation_registry: &mut summoner_sequencer::automation::AutomationRegistry,
        automation_timeline: &mut summoner_sequencer::automation_timeline::AutomationTimeline,
        playhead_beat: f64,
        is_recording_automation: bool,
    ) {
        let track_idx = if !project.tracks.is_empty() {
            if self.selected_track_idx < project.tracks.len() {
                self.selected_track_idx
            } else {
                0
            }
        } else {
            self.selected_track_idx
        };
        let track_id = if !project.tracks.is_empty() {
            project.tracks[track_idx].id
        } else {
            self.tracks.get(track_idx).map(|t| t.id).unwrap_or(1)
        };

        // Synchronize Novice Macro knobs with Device Rack dials before dispatch/recording if altered
        if !self.top_bar_state.is_pro_mode && (
            (self.top_bar_state.macro_tone - self.last_applied_macros[0]).abs() > 1e-5
            || (self.top_bar_state.macro_space - self.last_applied_macros[1]).abs() > 1e-5
            || (self.top_bar_state.macro_punch - self.last_applied_macros[2]).abs() > 1e-5
        ) {
            self.last_applied_macros[0] = self.top_bar_state.macro_tone;
            self.last_applied_macros[1] = self.top_bar_state.macro_space;
            self.last_applied_macros[2] = self.top_bar_state.macro_punch;
            self.device_rack_state.cutoff = self.top_bar_state.macro_tone;
            self.device_rack_state.decay = self.top_bar_state.macro_space;
            self.device_rack_state.drive = self.top_bar_state.macro_punch;
        }

        // 1. If playing and NOT recording, evaluate automated curves to drive GUI knobs
        if self.top_bar_state.is_playing && !is_recording_automation {
            let standard_keys = [
                ("cutoff", &mut self.device_rack_state.cutoff),
                ("resonance", &mut self.device_rack_state.resonance),
                ("decay", &mut self.device_rack_state.decay),
                ("env_decay", &mut self.device_rack_state.env_decay),
                ("mod_amt", &mut self.device_rack_state.mod_amt),
                ("drive", &mut self.device_rack_state.drive),
                ("osc_mix", &mut self.device_rack_state.osc_mix),
                ("shape", &mut self.device_rack_state.shape),
                ("volume", &mut self.device_rack_state.volume),
            ];
            for (key, val_ref) in standard_keys {
                let lane_key = format!("track_{}_{}", track_id, key);
                if let Some(val) = automation_timeline.evaluate(&lane_key, playhead_beat) {
                    *val_ref = val;
                    self.device_rack_state.node_param_values.insert(key.to_string(), val);
                }
            }

            for (k, v) in &mut self.device_rack_state.node_param_values {
                let lane_key = format!("track_{}_{}", track_id, k);
                if let Some(val) = automation_timeline.evaluate(&lane_key, playhead_beat) {
                    *v = val;
                }
            }

            // Evaluate Top Bar Macros & Master Volume from automation timeline
            let macro_keys = [
                ("macro_tone", &mut self.top_bar_state.macro_tone),
                ("macro_space", &mut self.top_bar_state.macro_space),
                ("macro_punch", &mut self.top_bar_state.macro_punch),
                ("macro_character", &mut self.top_bar_state.macro_character),
            ];
            let mut any_macro_automated = false;
            for (m_key, m_val_ref) in macro_keys {
                let lane_key = format!("track_{}_{}", track_id, m_key);
                if let Some(val) = automation_timeline.evaluate(&lane_key, playhead_beat) {
                    *m_val_ref = val;
                    any_macro_automated = true;
                }
            }
            if any_macro_automated {
                self.device_rack_state.cutoff = self.top_bar_state.macro_tone;
                self.device_rack_state.decay = self.top_bar_state.macro_space;
                self.device_rack_state.drive = self.top_bar_state.macro_punch;
                self.last_applied_macros = [
                    self.top_bar_state.macro_tone,
                    self.top_bar_state.macro_space,
                    self.top_bar_state.macro_punch,
                    self.top_bar_state.macro_character,
                ];
            }

            if let Some(val) = automation_timeline.evaluate("master_gain", playhead_beat) {
                self.top_bar_state.master_gain = val;
            }

            for (t_i, vt) in self.tracks.iter_mut().enumerate() {
                let t_id = vt.id;
                let gain_lane = format!("track_{}_gain", t_id);
                if let Some(val) = automation_timeline.evaluate(&gain_lane, playhead_beat) {
                    vt.gain = val;
                    if t_i == track_idx {
                        self.inspector_state.gain_db = (val - 1.0) * 12.0;
                        self.last_inspector_gain_db = self.inspector_state.gain_db;
                    }
                }
                let pan_lane = format!("track_{}_pan", t_id);
                if let Some(val) = automation_timeline.evaluate(&pan_lane, playhead_beat) {
                    vt.pan = val;
                    if t_i == track_idx {
                        self.inspector_state.pan_val = val;
                        self.last_inspector_pan = val;
                    }
                }
                let mute_lane = format!("track_{}_mute", t_id);
                if let Some(val) = automation_timeline.evaluate(&mute_lane, playhead_beat) {
                    let is_m = val >= 0.5;
                    vt.is_muted = is_m;
                    if t_i == track_idx {
                        self.inspector_state.is_muted = is_m;
                        self.last_inspector_muted = is_m;
                    }
                }
                let solo_lane = format!("track_{}_solo", t_id);
                if let Some(val) = automation_timeline.evaluate(&solo_lane, playhead_beat) {
                    let is_s = val >= 0.5;
                    vt.is_soloed = is_s;
                    if t_i == track_idx {
                        self.inspector_state.is_soloed = is_s;
                        self.last_inspector_soloed = is_s;
                    }
                }
                let clip_lane = format!("track_{}_clip", t_id);
                if let Some(val) = automation_timeline.evaluate(&clip_lane, playhead_beat) {
                    if val >= 0.0 {
                        vt.active_clip_idx = Some(val.round() as usize);
                    } else {
                        vt.active_clip_idx = None;
                    }
                }
            }
            for mod_node in &mut self.modular_nodes {
                for param in &mut mod_node.params {
                    let auto_key = format!("modular_{}_{}", mod_node.id, param.id);
                    if let Some(val) = automation_timeline.evaluate(&auto_key, playhead_beat) {
                        param.value = val.clamp(param.min, param.max);
                        if self.selected_modular_node_id.as_deref() == Some(&mod_node.id) {
                            self.inspector_state.node_param_values.insert(param.id.clone(), param.value);
                            self.device_rack_state.node_param_values.insert(param.id.clone(), param.value);
                        }
                    }
                }
            }
        }

        // 2. Dispatch all parameters to ParamBus and AutomationRegistry
        let registry = crate::dsp_node_ui::DspNodeRegistry::new();
        let cur_kind = self.device_rack_state.selected_node_kind.as_deref().unwrap_or("AetherSynth");
        let opt_desc = registry.get(cur_kind);

        if let Some(desc) = opt_desc {
            for (p_i, schema) in desc.params.iter().enumerate() {
                let val = self.device_rack_state.node_param_values.get(&schema.id).copied()
                    .unwrap_or(match &schema.widget {
                        crate::dsp_node_ui::DspWidgetKind::RotaryKnob { default, .. }
                        | crate::dsp_node_ui::DspWidgetKind::VerticalFader { default, .. } => *default,
                        _ => 0.5,
                    });

                let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + p_i as u32);
                if param_bus.get(pid).is_some() {
                    param_bus.set(pid, val);
                }

                let auto_key = format!("track_{}_{}", track_id, schema.id);
                if automation_registry.get_param(&auto_key).is_none() {
                    automation_registry.register_param(&auto_key, val);
                }
                automation_registry.set(&auto_key, val);

                if is_recording_automation {
                    let point = summoner_sequencer::automation_timeline::AutomationPoint {
                        beat: playhead_beat,
                        value: val,
                        interp: summoner_sequencer::automation_timeline::Interpolation::Linear,
                    };
                    let lane = automation_timeline.lanes.entry(auto_key.clone()).or_insert_with(|| {
                        summoner_sequencer::automation_timeline::AutomationLane {
                            param_id: auto_key.clone(),
                            curve: summoner_sequencer::automation_timeline::AutomationCurve { points: Vec::new() },
                        }
                    });
                    match lane.curve.points.binary_search_by(|p| p.beat.partial_cmp(&playhead_beat).unwrap()) {
                        Ok(idx) => lane.curve.points[idx] = point,
                        Err(idx) => lane.curve.points.insert(idx, point),
                    }
                }
            }
        }

        // Standard dials dispatch
        let standard_dials = [
            ("cutoff", self.device_rack_state.cutoff, 0),
            ("resonance", self.device_rack_state.resonance, 1),
            ("decay", self.device_rack_state.decay, 2),
            ("env_decay", self.device_rack_state.env_decay, 3),
            ("mod_amt", self.device_rack_state.mod_amt, 4),
            ("drive", self.device_rack_state.drive, 5),
            ("volume", self.device_rack_state.volume, 6),
        ];
        for (name, val, offset) in standard_dials {
            let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 100 + offset);
            if param_bus.get(pid).is_some() {
                param_bus.set(pid, val);
            }

            let auto_key = format!("track_{}_{}", track_id, name);
            if automation_registry.get_param(&auto_key).is_none() {
                automation_registry.register_param(&auto_key, val);
            }
            automation_registry.set(&auto_key, val);

            if is_recording_automation {
                let point = summoner_sequencer::automation_timeline::AutomationPoint {
                    beat: playhead_beat,
                    value: val,
                    interp: summoner_sequencer::automation_timeline::Interpolation::Linear,
                };
                let lane = automation_timeline.lanes.entry(auto_key.clone()).or_insert_with(|| {
                    summoner_sequencer::automation_timeline::AutomationLane {
                        param_id: auto_key.clone(),
                        curve: summoner_sequencer::automation_timeline::AutomationCurve { points: Vec::new() },
                    }
                });
                match lane.curve.points.binary_search_by(|p| p.beat.partial_cmp(&playhead_beat).unwrap()) {
                    Ok(idx) => lane.curve.points[idx] = point,
                    Err(idx) => lane.curve.points.insert(idx, point),
                }
            }
        }

        // 3. Novice Top Bar Macros Dispatch & Live Recording
        self.top_bar_state.macro_tone = self.device_rack_state.cutoff;
        self.top_bar_state.macro_space = self.device_rack_state.decay;
        self.top_bar_state.macro_punch = self.device_rack_state.drive;
        self.top_bar_state.macro_character = self.device_rack_state.mod_amt;
        self.last_applied_macros = [
            self.top_bar_state.macro_tone,
            self.top_bar_state.macro_space,
            self.top_bar_state.macro_punch,
            self.top_bar_state.macro_character,
        ];
        let top_macros = [
            ("macro_tone", self.device_rack_state.cutoff, 100),
            ("macro_space", self.device_rack_state.decay, 102),
            ("macro_punch", self.device_rack_state.drive, 105),
            ("macro_character", self.device_rack_state.mod_amt, 104),
        ];
        for (m_name, m_val, offset) in top_macros {
            let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + offset);
            if param_bus.get(pid).is_some() {
                param_bus.set(pid, m_val);
            }
            let auto_key = format!("track_{}_{}", track_id, m_name);
            if automation_registry.get_param(&auto_key).is_none() {
                automation_registry.register_param(&auto_key, m_val);
            }
            automation_registry.set(&auto_key, m_val);

            if is_recording_automation {
                let point = summoner_sequencer::automation_timeline::AutomationPoint {
                    beat: playhead_beat,
                    value: m_val,
                    interp: summoner_sequencer::automation_timeline::Interpolation::Linear,
                };
                let lane = automation_timeline.lanes.entry(auto_key.clone()).or_insert_with(|| {
                    summoner_sequencer::automation_timeline::AutomationLane {
                        param_id: auto_key.clone(),
                        curve: summoner_sequencer::automation_timeline::AutomationCurve { points: Vec::new() },
                    }
                });
                match lane.curve.points.binary_search_by(|p| p.beat.partial_cmp(&playhead_beat).unwrap()) {
                    Ok(idx) => lane.curve.points[idx] = point,
                    Err(idx) => lane.curve.points.insert(idx, point),
                }
            }
        }

        // 4. Master Gain & Mono Dispatch & Live Recording
        let m_pid = summoner_core::param_bus::ParamId(9999);
        if param_bus.get(m_pid).is_some() {
            param_bus.set(m_pid, self.top_bar_state.master_gain);
        }
        let mono_pid = summoner_core::param_bus::ParamId(9998);
        if param_bus.get(mono_pid).is_some() {
            param_bus.set(mono_pid, if self.is_master_mono { 1.0 } else { 0.0 });
        }
        let master_auto_key = "master_gain".to_string();
        if automation_registry.get_param(&master_auto_key).is_none() {
            automation_registry.register_param(&master_auto_key, self.top_bar_state.master_gain);
        }
        automation_registry.set(&master_auto_key, self.top_bar_state.master_gain);

        if is_recording_automation {
            let point = summoner_sequencer::automation_timeline::AutomationPoint {
                beat: playhead_beat,
                value: self.top_bar_state.master_gain,
                interp: summoner_sequencer::automation_timeline::Interpolation::Linear,
            };
            let lane = automation_timeline.lanes.entry(master_auto_key.clone()).or_insert_with(|| {
                summoner_sequencer::automation_timeline::AutomationLane {
                    param_id: master_auto_key.clone(),
                    curve: summoner_sequencer::automation_timeline::AutomationCurve { points: Vec::new() },
                }
            });
            match lane.curve.points.binary_search_by(|p| p.beat.partial_cmp(&playhead_beat).unwrap()) {
                Ok(idx) => lane.curve.points[idx] = point,
                Err(idx) => lane.curve.points.insert(idx, point),
            }
        }

        // 5. Track Gain, Pan, Mute, Solo Dispatch & Live Recording across All Tracks
        for (t_i, tr) in self.tracks.iter().enumerate() {
            let t_id = tr.id;
            let t_gain = tr.gain;
            let t_pan = tr.pan;
            let t_muted = tr.is_muted;
            let t_soloed = tr.is_soloed;
            let m_val = if t_muted { 1.0 } else { 0.0 };
            let s_val = if t_soloed { 1.0 } else { 0.0 };

            let g_pid = summoner_core::param_bus::ParamId(t_id as u32 * 1000 + 200);
            if param_bus.get(g_pid).is_some() {
                param_bus.set(g_pid, t_gain);
            }
            let p_pid = summoner_core::param_bus::ParamId(t_id as u32 * 1000 + 201);
            if param_bus.get(p_pid).is_some() {
                param_bus.set(p_pid, t_pan);
            }
            let mute_pid = summoner_core::param_bus::ParamId(t_id as u32 * 1000 + 202);
            if param_bus.get(mute_pid).is_some() {
                param_bus.set(mute_pid, m_val);
            }
            let solo_pid = summoner_core::param_bus::ParamId(t_id as u32 * 1000 + 203);
            if param_bus.get(solo_pid).is_some() {
                param_bus.set(solo_pid, s_val);
            }

            let gain_auto_key = format!("track_{}_gain", t_id);
            if automation_registry.get_param(&gain_auto_key).is_none() {
                automation_registry.register_param(&gain_auto_key, t_gain);
            }
            automation_registry.set(&gain_auto_key, t_gain);

            let pan_auto_key = format!("track_{}_pan", t_id);
            if automation_registry.get_param(&pan_auto_key).is_none() {
                automation_registry.register_param(&pan_auto_key, t_pan);
            }
            automation_registry.set(&pan_auto_key, t_pan);

            let mute_auto_key = format!("track_{}_mute", t_id);
            if automation_registry.get_param(&mute_auto_key).is_none() {
                automation_registry.register_param(&mute_auto_key, m_val);
            }
            automation_registry.set(&mute_auto_key, m_val);

            let solo_auto_key = format!("track_{}_solo", t_id);
            if automation_registry.get_param(&solo_auto_key).is_none() {
                automation_registry.register_param(&solo_auto_key, s_val);
            }
            automation_registry.set(&solo_auto_key, s_val);

            if is_recording_automation && t_i == track_idx {
                for (key, val, interp) in [
                    (&gain_auto_key, t_gain, summoner_sequencer::automation_timeline::Interpolation::Linear),
                    (&pan_auto_key, t_pan, summoner_sequencer::automation_timeline::Interpolation::Linear),
                    (&mute_auto_key, m_val, summoner_sequencer::automation_timeline::Interpolation::Step),
                    (&solo_auto_key, s_val, summoner_sequencer::automation_timeline::Interpolation::Step),
                ] {
                    let point = summoner_sequencer::automation_timeline::AutomationPoint {
                        beat: playhead_beat,
                        value: val,
                        interp,
                    };
                    let lane = automation_timeline.lanes.entry(key.clone()).or_insert_with(|| {
                        summoner_sequencer::automation_timeline::AutomationLane {
                            param_id: key.clone(),
                            curve: summoner_sequencer::automation_timeline::AutomationCurve { points: Vec::new() },
                        }
                    });
                    match lane.curve.points.binary_search_by(|p| p.beat.partial_cmp(&playhead_beat).unwrap()) {
                        Ok(idx) => lane.curve.points[idx] = point,
                        Err(idx) => lane.curve.points.insert(idx, point),
                    }
                }
            }
        }

        // 6. Stage Live Scene & Track Clip Cue Dispatch to ParamBus (Milestone 33)
        let panic_pid = summoner_core::param_bus::ParamId(9996);
        let panic_val = if self.panic_triggered { 1.0 } else { 0.0 };
        if param_bus.get(panic_pid).is_some() {
            param_bus.set(panic_pid, panic_val);
        }

        let scene_pid = summoner_core::param_bus::ParamId(9997);
        let scene_val = self.active_scene_idx.map(|s| s as f32).unwrap_or(-1.0);
        if param_bus.get(scene_pid).is_some() {
            param_bus.set(scene_pid, scene_val);
        }

        for tr in &self.tracks {
            let clip_pid = summoner_core::param_bus::ParamId(tr.id as u32 * 1000 + 204);
            let clip_val = tr.active_clip_idx.map(|c| c as f32).unwrap_or(-1.0);
            if param_bus.get(clip_pid).is_some() {
                param_bus.set(clip_pid, clip_val);
            }
        }

        // 7. Modular Node Bypass Dispatch & Live Recording
        for (m_idx, mod_node) in self.modular_nodes.iter().enumerate() {
            let byp_pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 600 + m_idx as u32);
            let byp_val = if mod_node.bypassed { 1.0 } else { 0.0 };
            if param_bus.get(byp_pid).is_some() {
                param_bus.set(byp_pid, byp_val);
            }
            let byp_auto_key = format!("modular_{}_bypassed", mod_node.id);
            if automation_registry.get_param(&byp_auto_key).is_none() {
                automation_registry.register_param(&byp_auto_key, byp_val);
            }
            automation_registry.set(&byp_auto_key, byp_val);

            if is_recording_automation {
                let point = summoner_sequencer::automation_timeline::AutomationPoint {
                    beat: playhead_beat,
                    value: byp_val,
                    interp: summoner_sequencer::automation_timeline::Interpolation::Step,
                };
                let lane = automation_timeline.lanes.entry(byp_auto_key.clone()).or_insert_with(|| {
                    summoner_sequencer::automation_timeline::AutomationLane {
                        param_id: byp_auto_key.clone(),
                        curve: summoner_sequencer::automation_timeline::AutomationCurve { points: Vec::new() },
                    }
                });
                match lane.curve.points.binary_search_by(|p| p.beat.partial_cmp(&playhead_beat).unwrap()) {
                    Ok(idx) => lane.curve.points[idx] = point,
                    Err(idx) => lane.curve.points.insert(idx, point),
                }
            }
        }

        // 7a-2. Sync active Bézier automation editor curve into automation timeline (M33)
        if let Some(ref lane_key) = self.requested_modular_automation_param {
            if let Some(ref editor) = self.automation_editor {
                let lane = automation_timeline.lanes.entry(lane_key.clone()).or_insert_with(|| {
                    summoner_sequencer::automation_timeline::AutomationLane {
                        param_id: lane_key.clone(),
                        curve: summoner_sequencer::automation_timeline::AutomationCurve { points: Vec::new() },
                    }
                });
                lane.curve.points.clear();
                for node in &editor.nodes {
                    let val = editor.min_display + node.value * (editor.max_display - editor.min_display);
                    let interp = match node.curve {
                        crate::views::bezier_automation_editor::AutomationCurveType::Hold => summoner_sequencer::automation_timeline::Interpolation::Step,
                        _ => summoner_sequencer::automation_timeline::Interpolation::Linear,
                    };
                    lane.curve.points.push(summoner_sequencer::automation_timeline::AutomationPoint {
                        beat: node.time_beats,
                        value: val,
                        interp,
                    });
                }
            }
        }

        // Keep selected modular node parameters synchronized with Inspector & Device Rack
        self.sync_selected_modular_node_params();

        // 7b. All Modular Nodes Faceplate Parameters Dispatch & Live Recording (M33)
        for (n_idx, mod_node) in self.modular_nodes.iter_mut().enumerate() {
            for (p_idx, param) in mod_node.params.iter_mut().enumerate() {
                let auto_key = format!("modular_{}_{}", mod_node.id, param.id);
                if !is_recording_automation {
                    if let Some(val) = automation_timeline.evaluate(&auto_key, playhead_beat) {
                        param.value = val.clamp(param.min, param.max);
                    }
                }
                let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 500 + (n_idx as u32 * 16) + p_idx as u32);
                if param_bus.get(pid).is_some() {
                    param_bus.set(pid, param.value);
                }
                if automation_registry.get_param(&auto_key).is_none() {
                    automation_registry.register_param(&auto_key, param.value);
                }
                automation_registry.set(&auto_key, param.value);

                if is_recording_automation {
                    let point = summoner_sequencer::automation_timeline::AutomationPoint {
                        beat: playhead_beat,
                        value: param.value,
                        interp: summoner_sequencer::automation_timeline::Interpolation::Linear,
                    };
                    let lane = automation_timeline.lanes.entry(auto_key.clone()).or_insert_with(|| {
                        summoner_sequencer::automation_timeline::AutomationLane {
                            param_id: auto_key.clone(),
                            curve: summoner_sequencer::automation_timeline::AutomationCurve { points: Vec::new() },
                        }
                    });
                    match lane.curve.points.binary_search_by(|p| p.beat.partial_cmp(&playhead_beat).unwrap()) {
                        Ok(idx) => lane.curve.points[idx] = point,
                        Err(idx) => lane.curve.points.insert(idx, point),
                    }
                }
            }
        }

        if let Some(ref mod_node_id) = self.selected_modular_node_id {
            let is_external = self.modular_nodes.iter().all(|n| &n.id != mod_node_id);
            let mut sorted_params: Vec<_> = self.inspector_state.node_param_values.iter().collect();
            sorted_params.sort_by_key(|(k, _)| (*k).clone());
            for (p_idx, (k, &v)) in sorted_params.into_iter().enumerate() {
                let auto_key = format!("modular_{}_{}", mod_node_id, k);
                if automation_registry.get_param(&auto_key).is_none() {
                    automation_registry.register_param(&auto_key, v);
                }
                automation_registry.set(&auto_key, v);
                if is_external {
                    let pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 500 + p_idx as u32);
                    if param_bus.get(pid).is_some() {
                        param_bus.set(pid, v);
                    }
                }
            }
        }

        // 7c. Modular Patch Cord Intensity Dispatch & Live Recording (M33)
        for (c_idx, cord) in self.patch_cords.iter_mut().enumerate() {
            let cord_auto_key = format!("modular_cord_{}_{}_{}_{}_intensity", cord.from_node_id, cord.from_port_id, cord.to_node_id, cord.to_port_id);
            if !is_recording_automation {
                if let Some(val) = automation_timeline.evaluate(&cord_auto_key, playhead_beat) {
                    cord.intensity = val.clamp(-1.0, 1.0);
                }
            }
            let cord_pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 700 + c_idx as u32);
            if param_bus.get(cord_pid).is_some() {
                param_bus.set(cord_pid, cord.intensity);
            }
            if automation_registry.get_param(&cord_auto_key).is_none() {
                automation_registry.register_param(&cord_auto_key, cord.intensity);
            }
            automation_registry.set(&cord_auto_key, cord.intensity);

            if is_recording_automation {
                let point = summoner_sequencer::automation_timeline::AutomationPoint {
                    beat: playhead_beat,
                    value: cord.intensity,
                    interp: summoner_sequencer::automation_timeline::Interpolation::Linear,
                };
                let lane = automation_timeline.lanes.entry(cord_auto_key.clone()).or_insert_with(|| {
                    summoner_sequencer::automation_timeline::AutomationLane {
                        param_id: cord_auto_key.clone(),
                        curve: summoner_sequencer::automation_timeline::AutomationCurve { points: Vec::new() },
                    }
                });
                match lane.curve.points.binary_search_by(|p| p.beat.partial_cmp(&playhead_beat).unwrap()) {
                    Ok(idx) => lane.curve.points[idx] = point,
                    Err(idx) => lane.curve.points.insert(idx, point),
                }
            }
        }

        // 8. Piano Roll Note Audition Live Dispatch (M33)
        let pitch_pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 300);
        if param_bus.get(pitch_pid).is_some() {
            let pitch = if self.is_piano_roll_audition_enabled { self.last_auditioned_pitch_hz } else { 0.0 };
            param_bus.set(pitch_pid, pitch);
        }
        let gate_pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 301);
        if param_bus.get(gate_pid).is_some() {
            let gate = if self.is_piano_roll_audition_enabled { self.last_auditioned_gate } else { 0.0 };
            param_bus.set(gate_pid, gate);
        }
        let vel_pid = summoner_core::param_bus::ParamId(track_id as u32 * 1000 + 302);
        if param_bus.get(vel_pid).is_some() {
            let vel = self.selected_note_id
                .and_then(|sid| self.piano_roll_notes.iter().find(|n| n.id == sid))
                .map(|n| n.velocity)
                .unwrap_or(0.85);
            param_bus.set(vel_pid, vel);
        }
    }

    /// Load sequence steps from a ProjectConfig track into the interactive piano roll.
    pub fn load_piano_roll_from_track(&mut self, track: &summoner_project::schema::TrackConfig) {
        let seq_opt = track.sequence.as_ref().or_else(|| track.clips.first());
        if let Some(seq) = seq_opt {
            if !seq.steps.is_empty() {
                let step_div = if seq.step_division > 0.0 { seq.step_division as f32 } else { 0.25 };
                self.piano_roll_notes = seq.steps.iter().enumerate().filter(|(_, s)| s.active && !s.muted).map(|(i, s)| {
                    let pitch_idx = (72.0 - s.note).clamp(0.0, 12.0).round() as usize;
                    let start_beat = seq.start_beat as f32 + (i as f32) * step_div;
                    let length_beats = (s.gate * step_div).max(0.25);
                    PianoRollNote {
                        id: i + 1,
                        pitch_idx,
                        start_beat,
                        length_beats,
                        velocity: s.velocity,
                    }
                }).collect();
                self.selected_note_id = self.piano_roll_notes.first().map(|n| n.id);
                self.next_note_id = self.piano_roll_notes.iter().map(|n| n.id).max().unwrap_or(0) + 1;
                return;
            }
        }
        self.piano_roll_notes.clear();
        self.selected_note_id = None;
    }

    /// Write active piano roll notes into a ProjectConfig track's sequence.
    pub fn sync_piano_roll_to_track(&self, track: &mut summoner_project::schema::TrackConfig) {
        if self.piano_roll_notes.is_empty() {
            return;
        }
        let step_div = 0.25_f32;
        let max_beat = self.piano_roll_notes.iter().map(|n| n.start_beat + n.length_beats).fold(16.0_f32, f32::max);
        let total_steps = ((max_beat / step_div).ceil() as usize).max(16);

        let seq = track.sequence.get_or_insert_with(|| summoner_project::schema::SequenceConfig {
            start_beat: 0.0,
            step_division: step_div as f64,
            clip_color: track.color,
            clip_name: Some(format!("{} Sequence", track.name)),
            name: format!("{} Sequence", track.name),
            is_unique: true,
            steps: vec![summoner_project::schema::TrackerStepConfig::default(); total_steps],
            ..Default::default()
        });

        if seq.steps.len() < total_steps {
            seq.steps.resize_with(total_steps, summoner_project::schema::TrackerStepConfig::default);
        }
        for s in &mut seq.steps {
            s.active = false;
        }
        for note in &self.piano_roll_notes {
            let step_idx = ((note.start_beat / step_div).round() as usize).min(seq.steps.len().saturating_sub(1));
            let step = &mut seq.steps[step_idx];
            step.note = (72 - (note.pitch_idx.min(12) as i32)) as f64;
            step.velocity = note.velocity;
            step.gate = (note.length_beats / step_div).max(0.1);
            step.active = true;
            step.muted = false;
        }
    }

    /// Select active track for Piano Roll editing and reflect its device name.
    pub fn select_track_for_piano_roll(&mut self, track_idx: usize) -> bool {
        if track_idx < self.tracks.len() {
            self.selected_track_idx = track_idx;
            let tr_name = self.tracks[track_idx].name.clone();
            self.device_rack_state.device_name = tr_name.clone();
            self.inspector_state.target_name = tr_name;
            if let Some(tr) = self.tracks.get(track_idx) {
                self.inspector_state.gain_db = (tr.gain - 1.0) * 12.0;
                self.inspector_state.pan_val = tr.pan;
                self.inspector_state.is_muted = tr.is_muted;
                self.inspector_state.is_soloed = tr.is_soloed;
                self.inspector_state.is_armed = tr.is_armed;
            }
            true
        } else {
            false
        }
    }

    /// Select active track for Modular canvas routing and reflect its state across Inspector and Device Rack.
    pub fn select_track_for_modular(&mut self, track_idx: usize) -> bool {
        if track_idx < self.tracks.len() {
            self.selected_track_idx = track_idx;
            let tr_name = self.tracks[track_idx].name.clone();
            self.device_rack_state.device_name = tr_name.clone();
            self.inspector_state.target_name = tr_name;
            if let Some(tr) = self.tracks.get(track_idx) {
                self.inspector_state.gain_db = (tr.gain - 1.0) * 12.0;
                self.inspector_state.pan_val = tr.pan;
                self.inspector_state.is_muted = tr.is_muted;
                self.inspector_state.is_soloed = tr.is_soloed;
                self.inspector_state.is_armed = tr.is_armed;
            }
            true
        } else {
            false
        }
    }

    /// Select active track for Stage / Live Performance Matrix and reflect its state across Inspector and Device Rack.
    pub fn select_track_for_stage(&mut self, track_idx: usize) -> bool {
        if track_idx < self.tracks.len() {
            self.selected_track_idx = track_idx;
            let tr_name = self.tracks[track_idx].name.clone();
            self.device_rack_state.device_name = tr_name.clone();
            self.inspector_state.target_name = tr_name;
            if let Some(tr) = self.tracks.get(track_idx) {
                self.inspector_state.gain_db = (tr.gain - 1.0) * 12.0;
                self.inspector_state.pan_val = tr.pan;
                self.inspector_state.is_muted = tr.is_muted;
                self.inspector_state.is_soloed = tr.is_soloed;
                self.inspector_state.is_armed = tr.is_armed;
            }
            true
        } else {
            false
        }
    }

    /// Select active track for Console Mixer and reflect its state across Inspector and Device Rack.
    pub fn select_track_for_mixer(&mut self, track_idx: usize) -> bool {
        if track_idx < self.tracks.len() {
            self.selected_track_idx = track_idx;
            let tr_name = self.tracks[track_idx].name.clone();
            self.device_rack_state.device_name = tr_name.clone();
            self.inspector_state.target_name = tr_name;
            if let Some(tr) = self.tracks.get(track_idx) {
                self.inspector_state.gain_db = (tr.gain - 1.0) * 12.0;
                self.inspector_state.pan_val = tr.pan;
                self.inspector_state.is_muted = tr.is_muted;
                self.inspector_state.is_soloed = tr.is_soloed;
                self.inspector_state.is_armed = tr.is_armed;
            }
            true
        } else {
            false
        }
    }

    /// Select active track for Arranger timeline editing and reflect its state across Inspector and Device Rack.
    pub fn select_track_for_arranger(&mut self, track_idx: usize) -> bool {
        if track_idx < self.tracks.len() {
            self.selected_track_idx = track_idx;
            let tr_name = self.tracks[track_idx].name.clone();
            self.device_rack_state.device_name = tr_name.clone();
            self.inspector_state.target_name = tr_name;
            if let Some(tr) = self.tracks.get(track_idx) {
                self.inspector_state.gain_db = (tr.gain - 1.0) * 12.0;
                self.inspector_state.pan_val = tr.pan;
                self.inspector_state.is_muted = tr.is_muted;
                self.inspector_state.is_soloed = tr.is_soloed;
                self.inspector_state.is_armed = tr.is_armed;
            }
            true
        } else {
            false
        }
    }

    /// Select a specific clip in the Arranger and update track selection and clip highlight state.
    pub fn select_clip(&mut self, track_idx: usize, clip_id: usize) -> bool {
        if track_idx < self.tracks.len() {
            self.select_track_for_arranger(track_idx);
            let mut found = false;
            for (t_idx, tr) in self.tracks.iter_mut().enumerate() {
                for c in &mut tr.clips {
                    if t_idx == track_idx && c.id == clip_id {
                        c.is_selected = true;
                        found = true;
                    } else {
                        c.is_selected = false;
                    }
                }
            }
            if found {
                self.selected_clip = Some((track_idx, clip_id));
                return true;
            }
        }
        false
    }

    /// Deselect any currently selected clip in the Arranger.
    pub fn deselect_clip(&mut self) {
        self.selected_clip = None;
        for tr in &mut self.tracks {
            for c in &mut tr.clips {
                c.is_selected = false;
            }
        }
    }

    /// Duplicate the currently selected clip, or the first clip of the selected track if none selected.
    pub fn duplicate_selected_clip(&mut self) -> Option<(usize, usize)> {
        if let Some((t_idx, c_id)) = self.selected_clip {
            let new_id_opt = self.tracks.get_mut(t_idx).and_then(|tr| tr.duplicate_clip(c_id));
            if let Some(new_id) = new_id_opt {
                self.select_clip(t_idx, new_id);
                return Some((t_idx, new_id));
            }
        } else {
            let t_idx = self.selected_track_idx;
            let new_id_opt = self.tracks.get_mut(t_idx).and_then(|tr| {
                tr.ensure_clips();
                let first_id = tr.clips.first()?.id;
                tr.duplicate_clip(first_id)
            });
            if let Some(new_id) = new_id_opt {
                self.select_clip(t_idx, new_id);
                return Some((t_idx, new_id));
            }
        }
        None
    }

    /// Delete the currently selected clip from its track.
    pub fn delete_selected_clip(&mut self) -> bool {
        if let Some((t_idx, c_id)) = self.selected_clip {
            if let Some(tr) = self.tracks.get_mut(t_idx) {
                let res = tr.delete_clip(c_id);
                self.selected_clip = None;
                return res;
            }
        }
        false
    }

    /// Quantize the selected clip (or all clips on active track) to the active Arranger snap grid.
    pub fn quantize_selected_clip(&mut self) -> bool {
        let step = self.arranger_snap_grid.step_beats();
        if step <= 0.0 {
            return false;
        }
        if let Some((t_idx, c_id)) = self.selected_clip {
            if let Some(tr) = self.tracks.get_mut(t_idx) {
                return tr.quantize_clip(c_id, step);
            }
        } else if let Some(tr) = self.tracks.get_mut(self.selected_track_idx) {
            let mut any = false;
            for c in &mut tr.clips {
                c.start_beat = ((c.start_beat / step).round() * step).max(0.0);
                any = true;
            }
            return any;
        }
        false
    }

    /// Split the selected clip (or the active clip under the playhead) at the current playhead position.
    pub fn split_selected_clip_at_playhead(&mut self) -> bool {
        let beat = self.playhead_beat;
        if let Some((t_idx, c_id)) = self.selected_clip {
            if let Some(tr) = self.tracks.get_mut(t_idx) {
                return tr.split_clip_at_beat(c_id, beat);
            }
        } else if let Some(tr) = self.tracks.get_mut(self.selected_track_idx) {
            tr.ensure_clips();
            if let Some(c_id) = tr.clips.iter().find(|c| beat > c.start_beat && beat < c.start_beat + c.length_beats).map(|c| c.id) {
                return tr.split_clip_at_beat(c_id, beat);
            }
        }
        false
    }

    /// Set timeline loop start and end bounds to match the selected clip's boundary.
    pub fn set_loop_to_selected_clip(&mut self) -> bool {
        if let Some((t_idx, c_id)) = self.selected_clip {
            if let Some(tr) = self.tracks.get(t_idx) {
                if let Some(c) = tr.clips.iter().find(|clip| clip.id == c_id) {
                    self.loop_start_beat = c.start_beat;
                    self.loop_end_beat = c.start_beat + c.length_beats;
                    return true;
                }
            }
        } else if let Some(tr) = self.tracks.get(self.selected_track_idx) {
            if let Some(c) = tr.clips.first() {
                self.loop_start_beat = c.start_beat;
                self.loop_end_beat = c.start_beat + c.length_beats;
                return true;
            }
        }
        false
    }

    /// Set whether note auditioning sound is enabled during Piano Roll editing.
    pub fn set_piano_roll_audition(&mut self, enabled: bool) {
        self.is_piano_roll_audition_enabled = enabled;
        if !enabled {
            self.last_auditioned_gate = 0.0;
        }
    }

    /// Toggle Piano Roll note auditioning sound state.
    pub fn toggle_piano_roll_audition(&mut self) -> bool {
        self.is_piano_roll_audition_enabled = !self.is_piano_roll_audition_enabled;
        if !self.is_piano_roll_audition_enabled {
            self.last_auditioned_gate = 0.0;
        }
        self.is_piano_roll_audition_enabled
    }

    /// Set the lower lane mode (Velocity, Gate Length, Pitch Bend).
    pub fn set_piano_roll_lane_mode(&mut self, mode: PianoRollLaneMode) {
        self.piano_roll_lane_mode = mode;
    }

    /// Clear all notes from the interactive Piano Roll.
    pub fn clear_piano_roll_notes(&mut self) {
        self.piano_roll_notes.clear();
        self.selected_note_id = None;
    }

    /// Duplicate currently selected note in the Piano Roll, advancing by active snap grid.
    pub fn duplicate_selected_piano_roll_note(&mut self) -> Option<usize> {
        if let Some(sel_id) = self.selected_note_id {
            if let Some(n) = self.piano_roll_notes.iter().find(|n| n.id == sel_id).cloned() {
                let step = self.piano_roll_snap.step_beats().max(0.5);
                let new_id = self.next_note_id;
                self.next_note_id += 1;
                self.piano_roll_notes.push(PianoRollNote {
                    id: new_id,
                    pitch_idx: n.pitch_idx,
                    start_beat: n.start_beat + step,
                    length_beats: n.length_beats,
                    velocity: n.velocity,
                });
                self.selected_note_id = Some(new_id);
                return Some(new_id);
            }
        }
        None
    }

    /// Quantize all notes or selected note to the active snap grid resolution.
    pub fn quantize_piano_roll_notes(&mut self) {
        let snap = self.piano_roll_snap;
        if let Some(sel_id) = self.selected_note_id {
            if let Some(n) = self.piano_roll_notes.iter_mut().find(|n| n.id == sel_id) {
                n.start_beat = snap.snap(n.start_beat);
            }
        } else {
            for n in &mut self.piano_roll_notes {
                n.start_beat = snap.snap(n.start_beat);
            }
        }
    }

    /// Transpose selected note by a number of semitones / steps.
    pub fn transpose_selected_piano_roll_note(&mut self, steps: i32) -> bool {
        let num_pitches = self.piano_roll_tuning.pitch_names().len();
        let max_p = num_pitches.saturating_sub(1);
        if let Some(sel_id) = self.selected_note_id {
            if let Some(n) = self.piano_roll_notes.iter_mut().find(|n| n.id == sel_id) {
                let new_p = (n.pitch_idx as i32 - steps).clamp(0, max_p as i32) as usize;
                n.pitch_idx = new_p;
                return true;
            }
        }
        false
    }

    #[cfg(feature = "gui")]
    fn show_modular_canvas(&mut self, ui: &mut egui::Ui) {
        let canvas_height = (self.tracks.len() as f32 * 36.0 + 36.0).max(280.0);
        egui::Frame::none()
            .fill(Color32::from_rgb(8, 12, 20))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(28, 40, 60)))
            .rounding(Rounding::same(6.0))
            .show(ui, |ui| {
                ui.set_height(canvas_height);

                // Modular Pro Toolbar: Switch between Patch Cords & Routing Matrix, Add Modules from DspNodeRegistry
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Modular Routing").font(FontId::proportional(11.0)).strong().color(Color32::from_rgb(241, 245, 249)));
                    ui.add_space(4.0);

                    // 1-Click Pro View Switchers
                    if ui.button(RichText::new("📋 Arranger").font(FontId::proportional(10.0)).color(Color32::from_rgb(200, 215, 235))).clicked() {
                        self.top_bar_state.active_tab = crate::views::modern_top_bar::ModernViewTab::Arranger;
                    }
                    if ui.button(RichText::new("🎹 Piano Roll").font(FontId::proportional(10.0)).color(Color32::from_rgb(200, 215, 235))).clicked() {
                        self.top_bar_state.active_tab = crate::views::modern_top_bar::ModernViewTab::PianoRoll;
                    }
                    if ui.button(RichText::new("🎚 Mixer").font(FontId::proportional(10.0)).color(Color32::from_rgb(200, 215, 235))).clicked() {
                        self.top_bar_state.active_tab = crate::views::modern_top_bar::ModernViewTab::Mixer;
                    }
                    if ui.button(RichText::new("🎭 Stage").font(FontId::proportional(10.0)).color(Color32::from_rgb(200, 215, 235))).clicked() {
                        self.top_bar_state.active_tab = crate::views::modern_top_bar::ModernViewTab::Performance;
                    }

                    ui.add_space(4.0);

                    // Track Selector ComboBox
                    let cur_tname = self.tracks.get(self.selected_track_idx).map(|t| t.name.as_str()).unwrap_or("Master");
                    let mut switch_track = None;
                    egui::ComboBox::from_id_source("modular_track_selector")
                        .selected_text(RichText::new(format!("🎚 Track: {}", cur_tname)).font(FontId::proportional(10.0)).color(Color32::from_rgb(56, 189, 248)))
                        .show_ui(ui, |ui| {
                            for (t_idx, t) in self.tracks.iter().enumerate() {
                                let is_sel = t_idx == self.selected_track_idx;
                                if ui.selectable_label(is_sel, &t.name).clicked() {
                                    switch_track = Some(t_idx);
                                }
                            }
                        });
                    if let Some(idx) = switch_track {
                        self.select_track_for_modular(idx);
                    }

                    ui.add_space(4.0);

                    // 1-Click Live Parameter Automation Launcher
                    let mut auto_req = None;
                    let mut mod_auto_req = None;
                    egui::ComboBox::from_id_source("modular_auto_selector")
                        .selected_text(RichText::new("📈 Auto ▾").font(FontId::proportional(10.0)).color(Color32::from_rgb(234, 179, 8)))
                        .show_ui(ui, |ui| {
                            let cur_tid = self.tracks.get(self.selected_track_idx).map(|t| t.id).unwrap_or(1);
                            if ui.button("Gain").clicked() {
                                auto_req = Some((cur_tid, "gain".to_string()));
                            }
                            if ui.button("Pan").clicked() {
                                auto_req = Some((cur_tid, "pan".to_string()));
                            }
                            if ui.button("Cutoff").clicked() {
                                auto_req = Some((cur_tid, "cutoff".to_string()));
                            }
                            if ui.button("Resonance").clicked() {
                                auto_req = Some((cur_tid, "resonance".to_string()));
                            }
                            if ui.button("Drive").clicked() {
                                auto_req = Some((cur_tid, "drive".to_string()));
                            }
                            if let Some(ref sel_n_id) = self.selected_modular_node_id {
                                if let Some(node) = self.modular_nodes.iter().find(|n| &n.id == sel_n_id) {
                                    ui.separator();
                                    ui.label(RichText::new(&node.display_name).font(FontId::proportional(9.0)).color(Color32::from_rgb(148, 163, 184)));
                                    for p in &node.params {
                                        if ui.button(&p.name).clicked() {
                                            mod_auto_req = Some((node.id.clone(), p.id.clone()));
                                        }
                                    }
                                }
                            }
                        });
                    if let Some((tid, param)) = auto_req {
                        self.open_track_automation_editor(tid, &param);
                    }
                    if let Some((node_id, param_id)) = mod_auto_req {
                        self.open_modular_automation_editor(&node_id, &param_id);
                    }

                    ui.separator();

                    // Mode Toggle: Patch Cords vs Matrix Grid
                    let cords_active = self.modular_mode == ModularCanvasMode::PatchCords;
                    if ui.selectable_label(cords_active, "∿ Patch Cords").clicked() {
                        if self.modular_mode == ModularCanvasMode::RoutingMatrix {
                            self.sync_routing_matrix_to_modular();
                        }
                        self.modular_mode = ModularCanvasMode::PatchCords;
                    }
                    let matrix_active = self.modular_mode == ModularCanvasMode::RoutingMatrix;
                    if ui.selectable_label(matrix_active, "▦ Routing Matrix").clicked() {
                        if self.modular_mode == ModularCanvasMode::PatchCords {
                            self.sync_modular_to_routing_matrix();
                        }
                        self.modular_mode = ModularCanvasMode::RoutingMatrix;
                    }

                    ui.separator();

                    // "➕ Add DSP Module..." Universal Catalog Launcher
                    let add_btn = ui.add(
                        egui::Button::new(RichText::new("➕ Add DSP Module...").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(56, 189, 248)))
                            .fill(Color32::from_rgb(20, 28, 44))
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(56, 189, 248)))
                            .rounding(Rounding::same(4.0))
                    );
                    if add_btn.clicked() {
                        self.modular_add_modal_open = !self.modular_add_modal_open;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("Clear Cords").clicked() {
                            self.patch_cords.clear();
                        }
                        if ui.small_button("Reset Layout").clicked() {
                            self.reset_modular_nodes();
                        }
                        ui.label(RichText::new(format!("{} Nodes | {} Cables", self.modular_nodes.len(), self.patch_cords.len())).font(FontId::proportional(9.0)).color(Color32::from_rgb(148, 163, 184)));
                    });
                });

                let mut cord_to_remove = None;
                let mut cord_to_auto = None;
                if let Some(c_idx) = self.selected_patch_cord_idx {
                    if c_idx < self.patch_cords.len() {
                        let cord = &mut self.patch_cords[c_idx];
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Cable:").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(56, 189, 248)));
                            ui.label(RichText::new(format!("{}:{} ➔ {}:{}", cord.from_node_id, cord.from_port_id, cord.to_node_id, cord.to_port_id)).font(FontId::proportional(10.0)).color(Color32::from_rgb(241, 245, 249)));
                            let sig_text = if cord.is_audio { "[Audio]" } else { "[Modulation]" };
                            let sig_col = if cord.is_audio { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(245, 158, 11) };
                            ui.label(RichText::new(sig_text).font(FontId::proportional(9.0)).color(sig_col));

                            ui.label(RichText::new("Attenuator:").font(FontId::proportional(10.0)).color(Color32::from_rgb(148, 163, 184)));
                            ui.add(egui::Slider::new(&mut cord.intensity, -1.0..=1.0).show_value(true).text(""));

                            if ui.small_button("± Invert").clicked() {
                                cord.intensity = -cord.intensity;
                            }
                            if ui.small_button("🔇 Mute").clicked() {
                                cord.intensity = 0.0;
                            }
                            if ui.small_button("100%").clicked() {
                                cord.intensity = 1.0;
                            }
                            if ui.small_button(RichText::new("📈 Auto").color(Color32::from_rgb(234, 179, 8))).clicked() {
                                cord_to_auto = Some(c_idx);
                            }
                            if ui.small_button(RichText::new("✕ Disconnect").color(Color32::from_rgb(239, 68, 68))).clicked() {
                                cord_to_remove = Some(c_idx);
                            }
                        });
                    }
                }
                if let Some(c_idx) = cord_to_auto {
                    self.open_patch_cord_automation_editor(c_idx);
                }
                if let Some(c_idx) = cord_to_remove {
                    self.remove_patch_cord_by_index(c_idx);
                }

                ui.separator();

                match self.modular_mode {
                    ModularCanvasMode::RoutingMatrix => {
                        self.patch_matrix.ui(ui);
                        self.sync_routing_matrix_to_modular();
                    }
                    ModularCanvasMode::PatchCords => {
                        self.render_patch_cords_canvas(ui, canvas_height - 38.0);
                    }
                }
            });
    }

    #[cfg(feature = "gui")]
    fn show_modular_dsp_catalog_window(&mut self, ui: &mut egui::Ui) {
        if !self.modular_add_modal_open {
            return;
        }

        let mut is_open = self.modular_add_modal_open;
        let mut node_to_add_kind: Option<String> = None;
        let mut close_modal = false;

        egui::Window::new("Modular DSP Module Catalog (2,012+ Nodes Registered)")
            .id(egui::Id::new("modular_dsp_catalog_modal"))
            .open(&mut is_open)
            .default_size([540.0, 420.0])
            .collapsible(false)
            .resizable(true)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("🔍").font(FontId::proportional(12.0)));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.modular_search_query)
                            .hint_text("Search 2,012+ DSP modules by name, kind, or category...")
                            .desired_width(ui.available_width() - 60.0),
                    );
                    if ui.button("Clear").clicked() {
                        self.modular_search_query.clear();
                        self.modular_selected_category = None;
                    }
                });

                ui.add_space(6.0);

                // Category Filter Bar (Horizontal scroll with category pills)
                egui::ScrollArea::horizontal()
                    .id_source("modular_dsp_cat_scroll")
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let is_all = self.modular_selected_category.is_none();
                            if ui.selectable_label(is_all, "All Categories").clicked() {
                                self.modular_selected_category = None;
                            }

                            let categories = [
                                crate::dsp_node_ui::DspCategory::Oscillator,
                                crate::dsp_node_ui::DspCategory::CompositeSynth,
                                crate::dsp_node_ui::DspCategory::AcousticPhysicalModel,
                                crate::dsp_node_ui::DspCategory::SamplerSlicer,
                                crate::dsp_node_ui::DspCategory::FilterEq,
                                crate::dsp_node_ui::DspCategory::DynamicsMaster,
                                crate::dsp_node_ui::DspCategory::DistortionSaturation,
                                crate::dsp_node_ui::DspCategory::Modulation,
                                crate::dsp_node_ui::DspCategory::TimeSpace,
                                crate::dsp_node_ui::DspCategory::SpatialSurround,
                                crate::dsp_node_ui::DspCategory::SpectralResynthesis,
                                crate::dsp_node_ui::DspCategory::NeuralAi,
                                crate::dsp_node_ui::DspCategory::Utility,
                            ];

                            for cat in categories {
                                let is_sel = self.modular_selected_category == Some(cat);
                                let label = format!("{} {}", cat.icon(), cat.display_label());
                                if ui.selectable_label(is_sel, label).clicked() {
                                    self.modular_selected_category = if is_sel { None } else { Some(cat) };
                                }
                            }
                        });
                    });

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(4.0);

                let registry = crate::dsp_node_ui::DspNodeRegistry::new();
                let all_nodes = registry.list_all();
                let q = self.modular_search_query.trim().to_lowercase();
                let sel_cat = self.modular_selected_category;

                let filtered_nodes: Vec<_> = all_nodes
                    .into_iter()
                    .filter(|desc| {
                        if let Some(cat) = sel_cat {
                            if desc.category != cat {
                                return false;
                            }
                        }
                        if !q.is_empty() {
                            let name_match = desc.display_name.to_lowercase().contains(&q);
                            let kind_match = desc.kind_id.to_lowercase().contains(&q);
                            let desc_match = desc.description.to_lowercase().contains(&q);
                            if !name_match && !kind_match && !desc_match {
                                return false;
                            }
                        }
                        true
                    })
                    .collect();

                ui.label(
                    RichText::new(format!(
                        "Showing {} of 2,012+ registered DSP modules:",
                        filtered_nodes.len()
                    ))
                    .font(FontId::proportional(10.0))
                    .color(Color32::from_rgb(148, 163, 184)),
                );

                ui.add_space(4.0);

                egui::ScrollArea::vertical()
                    .id_source("modular_dsp_node_list_scroll")
                    .max_height(280.0)
                    .show(ui, |ui| {
                        for desc in &filtered_nodes {
                            let (r, g, b) = desc.category.theme_color_rgb();
                            let cat_col = Color32::from_rgb(r, g, b);

                            egui::Frame::none()
                                .fill(Color32::from_rgb(16, 22, 34))
                                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(28, 40, 60)))
                                .rounding(Rounding::same(4.0))
                                .inner_margin(egui::Margin::symmetric(8.0, 6.0))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(desc.category.icon())
                                                .font(FontId::proportional(14.0)),
                                        );
                                        ui.vertical(|ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    RichText::new(&desc.display_name)
                                                        .font(FontId::proportional(11.0))
                                                        .strong()
                                                        .color(cat_col),
                                                );
                                                ui.label(
                                                    RichText::new(format!(
                                                        "({})",
                                                        desc.category.display_label()
                                                    ))
                                                    .font(FontId::proportional(9.0))
                                                    .color(Color32::from_rgb(100, 116, 139)),
                                                );
                                            });
                                            if !desc.description.is_empty() {
                                                ui.label(
                                                    RichText::new(&desc.description)
                                                        .font(FontId::proportional(9.0))
                                                        .color(Color32::from_rgb(148, 163, 184)),
                                                );
                                            }
                                        });

                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                let btn = ui.add(
                                                    egui::Button::new(
                                                        RichText::new("➕ Insert")
                                                            .font(FontId::proportional(10.0))
                                                            .strong()
                                                            .color(cat_col),
                                                    )
                                                    .fill(Color32::from_rgba_unmultiplied(
                                                        r, g, b, 30,
                                                    ))
                                                    .stroke(Stroke::new(1.0_f32, cat_col))
                                                    .rounding(Rounding::same(3.0)),
                                                );
                                                if btn.clicked() {
                                                    node_to_add_kind = Some(desc.kind_id.clone());
                                                }
                                                ui.label(
                                                    RichText::new(format!(
                                                        "{} params",
                                                        desc.params.len()
                                                    ))
                                                    .font(FontId::proportional(9.0))
                                                    .color(Color32::from_rgb(100, 116, 139)),
                                                );
                                            },
                                        );
                                    });
                                });
                            ui.add_space(2.0);
                        }
                    });

                if let Some(ref kind) = node_to_add_kind {
                    if let Some(desc) = registry.get(kind) {
                        self.add_modular_node_from_descriptor(desc);
                        close_modal = true;
                    }
                }

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Close").clicked() {
                        close_modal = true;
                    }
                });
            });

        if close_modal {
            is_open = false;
        }
        self.modular_add_modal_open = is_open;
    }

    #[cfg(feature = "gui")]
    fn show_modular_automation_editor_window(&mut self, ui: &mut egui::Ui) {
        if !self.show_automation_editor_window {
            return;
        }

        let mut is_open = self.show_automation_editor_window;
        let mut close_window = false;
        let param_title = self.requested_modular_automation_param.clone().unwrap_or_else(|| "Parameter".into());
        let window_title = format!("🎛️ Live Parameter Automation — {} (M33)", param_title);

        egui::Window::new(window_title)
            .id(egui::Id::new("modular_automation_editor_modal"))
            .open(&mut is_open)
            .default_size([720.0, 420.0])
            .min_size([580.0, 340.0])
            .collapsible(true)
            .resizable(true)
            .show(ui.ctx(), |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new("Quick Shapes:").font(FontId::proportional(11.0)).strong().color(Color32::from_rgb(148, 163, 184)));
                    if ui.button("Flat").clicked() {
                        self.apply_automation_quick_shape(AutomationQuickShape::Flat);
                    }
                    if ui.button("📈 Ramp Up").clicked() {
                        self.apply_automation_quick_shape(AutomationQuickShape::RampUp);
                    }
                    if ui.button("📉 Ramp Down").clicked() {
                        self.apply_automation_quick_shape(AutomationQuickShape::RampDown);
                    }
                    if ui.button("∿ Sine LFO").clicked() {
                        self.apply_automation_quick_shape(AutomationQuickShape::SineLfo);
                    }
                    if ui.button("⚡ Exp Drop").clicked() {
                        self.apply_automation_quick_shape(AutomationQuickShape::ExpDrop);
                    }
                    if ui.button("〰 S-Curve").clicked() {
                        self.apply_automation_quick_shape(AutomationQuickShape::SCurve);
                    }
                    if ui.button("⇅ Invert").clicked() {
                        self.apply_automation_quick_shape(AutomationQuickShape::Invert);
                    }
                    if ui.button("✨ Smooth").clicked() {
                        self.apply_automation_quick_shape(AutomationQuickShape::Smooth);
                    }
                    ui.separator();
                    if ui.button("✕ Close").clicked() {
                        close_window = true;
                    }
                });
                ui.separator();

                if let Some(ref mut editor) = self.automation_editor {
                    editor.show(ui);
                }
            });

        if close_window {
            is_open = false;
        }
        self.show_automation_editor_window = is_open;
    }

    #[cfg(feature = "gui")]
    fn render_patch_cords_canvas(&mut self, ui: &mut egui::Ui, available_h: f32) {
        let (resp, painter) = ui.allocate_painter(Vec2::new(ui.available_width(), available_h), egui::Sense::click_and_drag());
        let rect = resp.rect;

        // 1. Cybernetic Modular Background Grid
        let grid_spacing = 24.0;
        let mut gx = rect.left();
        while gx < rect.right() {
            let mut gy = rect.top();
            while gy < rect.bottom() {
                painter.circle_filled(egui::pos2(gx, gy), 1.0, Color32::from_rgb(24, 34, 52));
                gy += grid_spacing;
            }
            gx += grid_spacing;
        }

        let pointer_pos = resp.interact_pointer_pos();
        let clicked = resp.clicked();

        // Helper to locate socket position by (node_id, port_id)
        let find_socket_pos = |node_id: &str, port_id: &str, nodes: &[ModularNodeInstance]| -> Option<egui::Pos2> {
            for n in nodes {
                if n.id == node_id {
                    for p in &n.ports {
                        if p.id == port_id {
                            return Some(egui::pos2(
                                rect.left() + n.pos.0 + p.rel_pos.0,
                                rect.top() + n.pos.1 + p.rel_pos.1,
                            ));
                        }
                    }
                }
            }
            None
        };

        // Check cord attenuverter puck interactions first
        let mut clicked_cord_puck = None;
        let mut dragged_cord_puck = None;
        let mut clicked_knob = None;
        let mut dragged_knob = None;

        for (c_idx, cord) in self.patch_cords.iter().enumerate() {
            if let (Some(src_pt), Some(dst_pt)) = (
                find_socket_pos(&cord.from_node_id, &cord.from_port_id, &self.modular_nodes),
                find_socket_pos(&cord.to_node_id, &cord.to_port_id, &self.modular_nodes),
            ) {
                let sag = ((dst_pt.x - src_pt.x).abs() * 0.2 + (dst_pt.y - src_pt.y).abs() * 0.15).clamp(20.0, 60.0);
                let c1 = egui::pos2(src_pt.x + 30.0, src_pt.y + sag);
                let c2 = egui::pos2(dst_pt.x - 30.0, dst_pt.y + sag);
                let mid_pt = egui::pos2(
                    0.125 * src_pt.x + 0.375 * c1.x + 0.375 * c2.x + 0.125 * dst_pt.x,
                    0.125 * src_pt.y + 0.375 * c1.y + 0.375 * c2.y + 0.125 * dst_pt.y,
                );

                if let Some(pos) = pointer_pos {
                    if pos.distance(mid_pt) <= 14.0 {
                        if resp.double_clicked() {
                            clicked_cord_puck = Some((c_idx, false, true));
                        } else if clicked || resp.secondary_clicked() {
                            clicked_cord_puck = Some((c_idx, resp.secondary_clicked(), false));
                        } else if resp.dragged() {
                            dragged_cord_puck = Some(c_idx);
                        }
                    }
                }
            }
        }

        let drag_delta = resp.drag_delta();
        if let Some((c_idx, is_sec, is_double)) = clicked_cord_puck {
            if is_double {
                self.patch_cords[c_idx].intensity = 1.0;
            } else if is_sec {
                self.patch_cords[c_idx].intensity = -self.patch_cords[c_idx].intensity;
            }
            self.selected_patch_cord_idx = Some(c_idx);
            self.selected_modular_node_id = None;
            self.inspector_state.target_name = format!(
                "Cable: {}:{} ➔ {}:{}",
                self.patch_cords[c_idx].from_node_id,
                self.patch_cords[c_idx].from_port_id,
                self.patch_cords[c_idx].to_node_id,
                self.patch_cords[c_idx].to_port_id
            );
        } else if let Some(c_idx) = dragged_cord_puck {
            let delta_y = drag_delta.y;
            let speed = if ui.input(|i| i.modifiers.shift) { 0.003 } else { 0.015 };
            self.patch_cords[c_idx].intensity = (self.patch_cords[c_idx].intensity - delta_y * speed).clamp(-1.0, 1.0);
            self.selected_patch_cord_idx = Some(c_idx);
            self.selected_modular_node_id = None;
        }

        let mut node_selected = None;
        let mut clicked_socket = None;
        let mut clicked_knob_auto = None;
        let mut node_to_delete = None;
        let mut node_to_bypass = None;
        let mut node_to_duplicate = None;
        let mut node_to_inspect = None;
        let mut node_to_auto = None;

        if clicked_cord_puck.is_none() && dragged_cord_puck.is_none() {
            for node in &self.modular_nodes {
                let n_rect = Rect::from_min_size(
                    egui::pos2(rect.left() + node.pos.0, rect.top() + node.pos.1),
                    Vec2::new(node.size.0, node.size.1),
                );

                // Check parameter knob clicks and drags
                for param in &node.params {
                    let k_pos = egui::pos2(n_rect.left() + param.rel_pos.0, n_rect.top() + param.rel_pos.1);
                    if let Some(pos) = pointer_pos {
                        if pos.distance(k_pos) <= 13.0 {
                            if resp.double_clicked() {
                                clicked_knob = Some((node.id.clone(), param.id.clone(), true));
                            } else if resp.secondary_clicked() {
                                clicked_knob_auto = Some((node.id.clone(), param.id.clone()));
                            } else if clicked {
                                clicked_knob = Some((node.id.clone(), param.id.clone(), false));
                            } else if resp.dragged() {
                                dragged_knob = Some((node.id.clone(), param.id.clone()));
                            }
                        }
                    }
                }

                // Check socket clicks
                if clicked_knob.is_none() && dragged_knob.is_none() {
                    for port in &node.ports {
                        let s_pos = egui::pos2(n_rect.left() + port.rel_pos.0, n_rect.top() + port.rel_pos.1);
                        if let Some(pos) = pointer_pos {
                            if pos.distance(s_pos) <= 8.0 && clicked {
                                clicked_socket = Some((node.id.clone(), port.id.clone(), port.kind));
                            }
                        }
                    }
                }

                // Check header action button clicks
                if clicked_knob.is_none() && dragged_knob.is_none() && clicked_socket.is_none() && clicked {
                    if let Some(pos) = pointer_pos {
                        let del_rect = Rect::from_min_size(egui::pos2(n_rect.right() - 16.0, n_rect.top() + 4.0), Vec2::new(13.0, 14.0));
                        let byp_rect = Rect::from_min_size(egui::pos2(del_rect.left() - 20.0, n_rect.top() + 4.0), Vec2::new(18.0, 14.0));
                        let dup_rect = Rect::from_min_size(egui::pos2(byp_rect.left() - 16.0, n_rect.top() + 4.0), Vec2::new(14.0, 14.0));
                        let insp_rect = Rect::from_min_size(egui::pos2(dup_rect.left() - 16.0, n_rect.top() + 4.0), Vec2::new(14.0, 14.0));
                        let auto_rect = Rect::from_min_size(egui::pos2(insp_rect.left() - 16.0, n_rect.top() + 4.0), Vec2::new(14.0, 14.0));

                        if del_rect.contains(pos) {
                            node_to_delete = Some(node.id.clone());
                        } else if byp_rect.contains(pos) {
                            node_to_bypass = Some(node.id.clone());
                        } else if dup_rect.contains(pos) {
                            node_to_duplicate = Some(node.id.clone());
                        } else if insp_rect.contains(pos) {
                            node_to_inspect = Some(node.id.clone());
                        } else if auto_rect.contains(pos) {
                            node_to_auto = Some(node.id.clone());
                        } else if n_rect.contains(pos) {
                            node_selected = Some((node.id.clone(), node.kind_id.clone(), node.display_name.clone()));
                        }
                    }
                } else if clicked_knob.is_none() && dragged_knob.is_none() && clicked_socket.is_none() && resp.dragged() && self.selected_modular_node_id.is_none() {
                    if let Some(pos) = pointer_pos {
                        if n_rect.contains(pos) {
                            node_selected = Some((node.id.clone(), node.kind_id.clone(), node.display_name.clone()));
                        }
                    }
                }
            }
        }

        let is_dragging_knob = dragged_knob.is_some();
        if let Some((n_id, p_id)) = dragged_knob {
            let delta_y = drag_delta.y;
            if let Some(node) = self.modular_nodes.iter_mut().find(|n| n.id == n_id) {
                if let Some(param) = node.params.iter_mut().find(|p| p.id == p_id) {
                    let range = param.max - param.min;
                    let speed = if ui.input(|i| i.modifiers.shift) { 0.002 } else { 0.01 };
                    param.value = (param.value - delta_y * range * speed).clamp(param.min, param.max);
                    if self.selected_modular_node_id.as_deref() == Some(&n_id) {
                        self.inspector_state.node_param_values.insert(param.id.clone(), param.value);
                        self.device_rack_state.node_param_values.insert(param.id.clone(), param.value);
                    }
                }
            }
            if let Some(node) = self.modular_nodes.iter().find(|n| n.id == n_id) {
                node_selected = Some((node.id.clone(), node.kind_id.clone(), node.display_name.clone()));
            }
        } else if let Some((n_id, p_id, is_double)) = clicked_knob {
            if is_double {
                if let Some(def_val) = self.reset_modular_node_param(&n_id, &p_id) {
                    if self.selected_modular_node_id.as_deref() == Some(&n_id) {
                        self.inspector_state.node_param_values.insert(p_id.clone(), def_val);
                        self.device_rack_state.node_param_values.insert(p_id.clone(), def_val);
                    }
                }
            }
            if let Some(node) = self.modular_nodes.iter().find(|n| n.id == n_id) {
                node_selected = Some((node.id.clone(), node.kind_id.clone(), node.display_name.clone()));
            }
        }

        // Tactile node drag-to-reposition
        if resp.dragged() && self.pending_cord_source.is_none() && dragged_cord_puck.is_none() && !is_dragging_knob && (drag_delta.x.abs() > 0.0 || drag_delta.y.abs() > 0.0) {
            if let Some(ref sel_id) = self.selected_modular_node_id {
                if let Some(node) = self.modular_nodes.iter_mut().find(|n| &n.id == sel_id) {
                    node.pos.0 = (node.pos.0 + drag_delta.x).clamp(0.0, (rect.width() - node.size.0).max(0.0));
                    node.pos.1 = (node.pos.1 + drag_delta.y).clamp(0.0, (available_h - node.size.1).max(0.0));
                }
            }
        }

        // Execute header button actions
        if let Some(id) = node_to_delete {
            self.remove_modular_node(&id);
        } else if let Some(id) = node_to_bypass {
            self.toggle_bypass_modular_node(&id);
        } else if let Some(id) = node_to_duplicate {
            self.duplicate_modular_node(&id);
        } else if let Some(id) = node_to_inspect {
            self.select_modular_node(&id);
            self.inspector_state.is_collapsed = false;
        } else if let Some(id) = node_to_auto {
            self.select_modular_node(&id);
            let first_p_id = self.modular_nodes.iter().find(|n| n.id == id)
                .and_then(|node| node.params.first().map(|p| p.id.clone()));
            if let Some(p_id) = first_p_id {
                self.open_modular_automation_editor(&id, &p_id);
            }
        }

        if let Some((n_id, p_id)) = clicked_knob_auto {
            self.select_modular_node(&n_id);
            self.open_modular_automation_editor(&n_id, &p_id);
        }

        // Right-click on empty canvas opens the Modular DSP Catalog
        if resp.secondary_clicked() {
            if let Some(pos) = pointer_pos {
                let hit_node = self.modular_nodes.iter().any(|n| {
                    let nr = Rect::from_min_size(egui::pos2(rect.left() + n.pos.0, rect.top() + n.pos.1), Vec2::new(n.size.0, n.size.1));
                    nr.contains(pos)
                });
                if !hit_node {
                    self.modular_add_modal_open = true;
                }
            }
        }

        // Handle socket connection logic
        if let Some((node_id, port_id, port_kind)) = clicked_socket {
            if port_kind.is_output() {
                if self.pending_cord_source == Some((node_id.clone(), port_id.clone())) {
                    self.pending_cord_source = None;
                } else {
                    self.pending_cord_source = Some((node_id, port_id));
                }
            } else {
                // It is an input
                if let Some((src_node, src_port)) = self.pending_cord_source.take() {
                    if src_node != node_id {
                        let is_audio = !port_kind.is_modulation();
                        if let Some(pos) = self.patch_cords.iter().position(|c| c.from_node_id == src_node && c.from_port_id == src_port && c.to_node_id == node_id && c.to_port_id == port_id) {
                            self.patch_cords.remove(pos);
                            self.disconnect_patch_cord_in_graph(&src_node, &src_port, &node_id, &port_id);
                        } else {
                            self.patch_cords.push(ModularPatchCord {
                                from_node_id: src_node.clone(),
                                from_port_id: src_port.clone(),
                                to_node_id: node_id.clone(),
                                to_port_id: port_id.clone(),
                                is_audio,
                                intensity: 1.0,
                            });
                            self.connect_patch_cord_in_graph(&src_node, &src_port, &node_id, &port_id);
                            self.selected_patch_cord_idx = Some(self.patch_cords.len() - 1);
                        }
                    }
                } else {
                    // Disconnect existing cable to this input socket
                    let cords_to_remove: Vec<ModularPatchCord> = self.patch_cords.iter()
                        .filter(|c| c.to_node_id == node_id && c.to_port_id == port_id)
                        .cloned()
                        .collect();
                    for c in cords_to_remove {
                        self.disconnect_patch_cord_in_graph(&c.from_node_id, &c.from_port_id, &c.to_node_id, &c.to_port_id);
                    }
                    self.patch_cords.retain(|c| !(c.to_node_id == node_id && c.to_port_id == port_id));
                }
            }
        } else if clicked && pointer_pos.is_some() && node_selected.is_none() && clicked_cord_puck.is_none() {
            // Clicked on empty space: cancel pending cord and deselect cord
            self.pending_cord_source = None;
            self.selected_patch_cord_idx = None;
        }

        if let Some((node_id, _kind_id, _display_name)) = node_selected {
            self.select_modular_node(&node_id);
        }

        // Keyboard shortcuts for modular canvas: Delete/Backspace removes selected cord
        if ui.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)) {
            if let Some(sel_c) = self.selected_patch_cord_idx {
                self.remove_patch_cord_by_index(sel_c);
            }
        }

        // 2. Render Nodes
        for node in &self.modular_nodes {
            let n_rect = Rect::from_min_size(
                egui::pos2(rect.left() + node.pos.0, rect.top() + node.pos.1),
                Vec2::new(node.size.0, node.size.1),
            );
            let is_sel = self.selected_modular_node_id.as_deref() == Some(&node.id);
            let (cr, cg, cb) = node.category.color_rgb();
            let cat_col = if node.bypassed {
                Color32::from_rgb(100, 116, 139)
            } else {
                Color32::from_rgb(cr, cg, cb)
            };

            // Background chassis
            let bg_color = if node.bypassed {
                Color32::from_rgb(10, 14, 22)
            } else if is_sel {
                Color32::from_rgb(18, 26, 42)
            } else {
                Color32::from_rgb(16, 24, 38)
            };
            painter.rect_filled(n_rect, 4.0, bg_color);
            if is_sel {
                painter.rect_stroke(n_rect.expand(2.0), 5.0, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(cr, cg, cb, 60)));
                painter.rect_stroke(n_rect, 4.0, Stroke::new(1.8_f32, cat_col));
            } else {
                painter.rect_stroke(n_rect, 4.0, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(cr, cg, cb, if node.bypassed { 60 } else { 140 })));
            }

            // Header text
            let header_title = if node.bypassed {
                format!("{} {} [BYP]", node.category.icon(), node.display_name)
            } else {
                format!("{} {}", node.category.icon(), node.display_name)
            };
            painter.text(
                egui::pos2(n_rect.left() + 8.0, n_rect.top() + 6.0),
                egui::Align2::LEFT_TOP,
                header_title,
                FontId::proportional(11.0),
                if node.bypassed { Color32::from_rgb(148, 163, 184) } else { Color32::from_rgb(241, 245, 249) },
            );

            // Action buttons on header
            let del_rect = Rect::from_min_size(egui::pos2(n_rect.right() - 16.0, n_rect.top() + 4.0), Vec2::new(13.0, 14.0));
            let byp_rect = Rect::from_min_size(egui::pos2(del_rect.left() - 20.0, n_rect.top() + 4.0), Vec2::new(18.0, 14.0));
            let dup_rect = Rect::from_min_size(egui::pos2(byp_rect.left() - 16.0, n_rect.top() + 4.0), Vec2::new(14.0, 14.0));
            let insp_rect = Rect::from_min_size(egui::pos2(dup_rect.left() - 16.0, n_rect.top() + 4.0), Vec2::new(14.0, 14.0));
            let auto_rect = Rect::from_min_size(egui::pos2(insp_rect.left() - 16.0, n_rect.top() + 4.0), Vec2::new(14.0, 14.0));

            let del_hovered = pointer_pos.map(|p| del_rect.contains(p)).unwrap_or(false);
            let byp_hovered = pointer_pos.map(|p| byp_rect.contains(p)).unwrap_or(false);
            let dup_hovered = pointer_pos.map(|p| dup_rect.contains(p)).unwrap_or(false);
            let insp_hovered = pointer_pos.map(|p| insp_rect.contains(p)).unwrap_or(false);
            let auto_hovered = pointer_pos.map(|p| auto_rect.contains(p)).unwrap_or(false);

            // Auto button (∿)
            painter.rect_filled(auto_rect, 2.0, if auto_hovered { Color32::from_rgb(168, 85, 247) } else { Color32::from_rgb(24, 34, 52) });
            painter.text(auto_rect.center(), egui::Align2::CENTER_CENTER, "∿", FontId::proportional(9.0), Color32::from_rgb(241, 245, 249));

            // Inspector button (🔎)
            painter.rect_filled(insp_rect, 2.0, if insp_hovered { Color32::from_rgb(14, 165, 233) } else { Color32::from_rgb(24, 34, 52) });
            painter.text(insp_rect.center(), egui::Align2::CENTER_CENTER, "🔎", FontId::proportional(8.0), Color32::from_rgb(241, 245, 249));

            // Duplicate button (⧉)
            painter.rect_filled(dup_rect, 2.0, if dup_hovered { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(24, 34, 52) });
            painter.text(dup_rect.center(), egui::Align2::CENTER_CENTER, "⧉", FontId::proportional(9.0), Color32::from_rgb(241, 245, 249));

            // Bypass button (ON/OFF)
            painter.rect_filled(byp_rect, 2.0, if node.bypassed { Color32::from_rgb(245, 158, 11) } else if byp_hovered { Color32::from_rgb(38, 54, 82) } else { Color32::from_rgb(24, 34, 52) });
            let byp_text = if node.bypassed { "OFF" } else { "ON" };
            painter.text(byp_rect.center(), egui::Align2::CENTER_CENTER, byp_text, FontId::proportional(8.0), if node.bypassed { Color32::BLACK } else { Color32::from_rgb(148, 163, 184) });

            // Delete button (✕)
            painter.rect_filled(del_rect, 2.0, if del_hovered { Color32::from_rgb(220, 38, 38) } else { Color32::from_rgb(24, 34, 52) });
            painter.text(del_rect.center(), egui::Align2::CENTER_CENTER, "✕", FontId::proportional(9.0), Color32::from_rgb(241, 245, 249));

            // Tactile parameter knobs on node faceplate
            for param in &node.params {
                let k_pos = egui::pos2(n_rect.left() + param.rel_pos.0, n_rect.top() + param.rel_pos.1);
                let is_hovered = pointer_pos.map(|p| p.distance(k_pos) <= 12.0).unwrap_or(false);
                let norm = if (param.max - param.min).abs() > 0.0001 {
                    ((param.value - param.min) / (param.max - param.min)).clamp(0.0, 1.0)
                } else {
                    0.5
                };

                // Outer metallic bezel ring
                let bezel_stroke = if is_hovered {
                    Stroke::new(1.4_f32, Color32::from_rgb(148, 163, 184))
                } else {
                    Stroke::new(1.0_f32, Color32::from_rgb(51, 65, 85))
                };
                painter.circle_stroke(k_pos, 10.5, bezel_stroke);

                // Inner chassis cavity
                painter.circle_filled(k_pos, 9.5, Color32::from_rgb(15, 23, 42));

                // Colored value arc indicator
                let start_ang = -std::f32::consts::PI * 0.75;
                let arc_sweep = std::f32::consts::PI * 1.5 * norm;
                let steps = (norm * 14.0).ceil() as usize;
                if steps > 0 {
                    let mut prev_pt = egui::pos2(k_pos.x + start_ang.sin() * 8.5, k_pos.y - start_ang.cos() * 8.5);
                    for s in 1..=steps {
                        let t = s as f32 / 14.0;
                        let ang = start_ang + std::f32::consts::PI * 1.5 * t.min(norm);
                        let cur_pt = egui::pos2(k_pos.x + ang.sin() * 8.5, k_pos.y - ang.cos() * 8.5);
                        painter.line_segment([prev_pt, cur_pt], Stroke::new(1.6_f32, cat_col));
                        prev_pt = cur_pt;
                    }
                }

                // Needle pointer line
                let needle_ang = start_ang + arc_sweep;
                let pointer_end = egui::pos2(k_pos.x + needle_ang.sin() * 6.8, k_pos.y - needle_ang.cos() * 6.8);
                let pointer_start = egui::pos2(k_pos.x + needle_ang.sin() * 2.0, k_pos.y - needle_ang.cos() * 2.0);
                painter.line_segment([pointer_start, pointer_end], Stroke::new(1.6_f32, Color32::WHITE));

                // Parameter Name Label (above knob)
                painter.text(
                    egui::pos2(k_pos.x, k_pos.y - 14.0),
                    egui::Align2::CENTER_CENTER,
                    &param.name,
                    FontId::proportional(8.0),
                    if node.bypassed { Color32::from_rgb(100, 116, 139) } else { Color32::from_rgb(148, 163, 184) },
                );

                // Formatted Value Readout (below knob)
                let val_str = if param.value.abs() >= 1000.0 {
                    format!("{:.1}k{}", param.value / 1000.0, param.unit)
                } else if param.value.abs() < 1.0 && param.value.abs() > 0.0 {
                    format!("{:.2}{}", param.value, param.unit)
                } else {
                    format!("{:.1}{}", param.value, param.unit)
                };
                painter.text(
                    egui::pos2(k_pos.x, k_pos.y + 13.0),
                    egui::Align2::CENTER_CENTER,
                    val_str,
                    FontId::proportional(7.5),
                    if node.bypassed { Color32::from_rgb(71, 85, 105) } else { Color32::from_rgb(203, 213, 225) },
                );
            }

            // Sockets
            for port in &node.ports {
                let s_pos = egui::pos2(n_rect.left() + port.rel_pos.0, n_rect.top() + port.rel_pos.1);
                let (pr, pg, pb) = port.kind.color_rgb();
                let port_col = if node.bypassed {
                    Color32::from_rgb(71, 85, 105)
                } else {
                    Color32::from_rgb(pr, pg, pb)
                };

                // Outer metallic ring
                painter.circle_stroke(s_pos, 6.0, Stroke::new(1.5_f32, Color32::from_rgb(45, 60, 85)));
                // Inner socket hole
                painter.circle_filled(s_pos, 4.0, Color32::from_rgb(6, 10, 18));
                // LED pin
                painter.circle_filled(s_pos, 2.2, port_col);

                // Port label
                let text_pos = if port.kind.is_output() {
                    egui::pos2(s_pos.x - 10.0, s_pos.y)
                } else {
                    egui::pos2(s_pos.x + 10.0, s_pos.y)
                };
                let align = if port.kind.is_output() { egui::Align2::RIGHT_CENTER } else { egui::Align2::LEFT_CENTER };
                painter.text(text_pos, align, &port.name, FontId::proportional(9.0), if node.bypassed { Color32::from_rgb(100, 116, 139) } else { Color32::from_rgb(148, 163, 184) });
            }
        }

        // 3. Render Patch Cords (Bézier curves with gravity sag, glow, and attenuverter pucks)
        for (c_idx, cord) in self.patch_cords.iter().enumerate() {
            if let (Some(src_pt), Some(dst_pt)) = (
                find_socket_pos(&cord.from_node_id, &cord.from_port_id, &self.modular_nodes),
                find_socket_pos(&cord.to_node_id, &cord.to_port_id, &self.modular_nodes),
            ) {
                let sag = ((dst_pt.x - src_pt.x).abs() * 0.2 + (dst_pt.y - src_pt.y).abs() * 0.15).clamp(20.0, 60.0);
                let c1 = egui::pos2(src_pt.x + 30.0, src_pt.y + sag);
                let c2 = egui::pos2(dst_pt.x - 30.0, dst_pt.y + sag);
                let mid_pt = egui::pos2(
                    0.125 * src_pt.x + 0.375 * c1.x + 0.375 * c2.x + 0.125 * dst_pt.x,
                    0.125 * src_pt.y + 0.375 * c1.y + 0.375 * c2.y + 0.125 * dst_pt.y,
                );

                let is_cord_sel = self.selected_patch_cord_idx == Some(c_idx);

                let color = if cord.is_audio {
                    Color32::from_rgb(56, 189, 248) // Cyan for audio
                } else if cord.intensity < -0.01 {
                    Color32::from_rgb(236, 72, 153) // Magenta for inverted modulation
                } else if cord.intensity.abs() < 0.01 {
                    Color32::from_rgb(100, 116, 139) // Slate for muted
                } else {
                    Color32::from_rgb(245, 158, 11) // Amber for positive modulation
                };

                // Glow shadow
                let shadow_col = Color32::from_rgba_unmultiplied(
                    color.r(),
                    color.g(),
                    color.b(),
                    if is_cord_sel { 120 } else { ((cord.intensity.abs() * 65.0).clamp(15.0, 70.0)) as u8 },
                );
                let b_glow = egui::epaint::CubicBezierShape::from_points_stroke(
                    [src_pt, c1, c2, dst_pt],
                    false,
                    Color32::TRANSPARENT,
                    Stroke::new(if is_cord_sel { 4.5_f32 } else { 3.2_f32 }, shadow_col),
                );
                painter.add(b_glow);

                // Core cable
                let core_w = if is_cord_sel { 2.4_f32 } else { (1.2 + 1.2 * cord.intensity.abs()).clamp(1.0, 2.6) };
                let b_core = egui::epaint::CubicBezierShape::from_points_stroke(
                    [src_pt, c1, c2, dst_pt],
                    false,
                    Color32::TRANSPARENT,
                    Stroke::new(core_w, if is_cord_sel { Color32::WHITE } else { color }),
                );
                painter.add(b_core);

                // Active animated signal pulse along the cord
                if cord.intensity.abs() > 0.02 {
                    let pulse_t = (self.playhead_beat * 0.25).fract();
                    let pulse_pt = egui::pos2(
                        (1.0 - pulse_t) * (1.0 - pulse_t) * (1.0 - pulse_t) * src_pt.x
                            + 3.0 * (1.0 - pulse_t) * (1.0 - pulse_t) * pulse_t * c1.x
                            + 3.0 * (1.0 - pulse_t) * pulse_t * pulse_t * c2.x
                            + pulse_t * pulse_t * pulse_t * dst_pt.x,
                        (1.0 - pulse_t) * (1.0 - pulse_t) * (1.0 - pulse_t) * src_pt.y
                            + 3.0 * (1.0 - pulse_t) * (1.0 - pulse_t) * pulse_t * c1.y
                            + 3.0 * (1.0 - pulse_t) * pulse_t * pulse_t * c2.y
                            + pulse_t * pulse_t * pulse_t * dst_pt.y,
                    );
                    let p_scale = cord.intensity.abs().clamp(0.4, 1.0);
                    painter.circle_filled(pulse_pt, 2.8 * p_scale, Color32::WHITE);
                    painter.circle_stroke(pulse_pt, 4.0 * p_scale, Stroke::new(1.0_f32, color));
                }

                // Attenuverter / Attenuator Puck at mid_pt
                let puck_radius = if is_cord_sel { 9.0 } else { 7.5 };
                let is_puck_hovered = pointer_pos.map(|p| p.distance(mid_pt) <= 14.0).unwrap_or(false);

                if is_cord_sel {
                    painter.circle_stroke(mid_pt, puck_radius + 4.0, Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(56, 189, 248, 140)));
                    painter.circle_stroke(mid_pt, puck_radius + 1.5, Stroke::new(1.8_f32, Color32::WHITE));
                } else if is_puck_hovered {
                    painter.circle_stroke(mid_pt, puck_radius + 2.0, Stroke::new(1.0_f32, Color32::from_rgb(148, 163, 184)));
                }

                let puck_bg = if is_cord_sel {
                    Color32::from_rgb(28, 42, 68)
                } else if is_puck_hovered {
                    Color32::from_rgb(24, 34, 52)
                } else {
                    Color32::from_rgb(14, 20, 32)
                };
                painter.circle_filled(mid_pt, puck_radius, puck_bg);
                painter.circle_stroke(mid_pt, puck_radius, Stroke::new(1.2_f32, color));

                let text = if cord.intensity < -0.05 {
                    format!("-{:.0}%", cord.intensity.abs() * 100.0)
                } else if cord.intensity > 0.05 {
                    format!("{:.0}%", cord.intensity * 100.0)
                } else {
                    "0%".to_string()
                };
                let text_col = if cord.intensity < -0.05 {
                    Color32::from_rgb(244, 114, 182)
                } else if cord.intensity > 0.05 {
                    Color32::from_rgb(241, 245, 249)
                } else {
                    Color32::from_rgb(100, 116, 139)
                };
                painter.text(mid_pt, egui::Align2::CENTER_CENTER, text, FontId::proportional(7.5), text_col);
            }
        }

        // 4. Pending cord preview while dragging
        if let Some((src_n, src_p)) = &self.pending_cord_source {
            if let Some(src_pt) = find_socket_pos(src_n, src_p, &self.modular_nodes) {
                if let Some(cur_pos) = pointer_pos {
                    let sag = ((cur_pos.x - src_pt.x).abs() * 0.2 + (cur_pos.y - src_pt.y).abs() * 0.15).clamp(20.0, 50.0);
                    let c1 = egui::pos2(src_pt.x + 25.0, src_pt.y + sag);
                    let c2 = egui::pos2(cur_pos.x - 25.0, cur_pos.y + sag);

                    let b_pending = egui::epaint::CubicBezierShape::from_points_stroke(
                        [src_pt, c1, c2, cur_pos],
                        false,
                        Color32::TRANSPARENT,
                        Stroke::new(2.0_f32, Color32::from_rgb(56, 189, 248)),
                    );
                    painter.add(b_pending);
                    painter.circle_filled(cur_pos, 3.5, Color32::from_rgb(56, 189, 248));
                }
            }
        }
    }

    #[cfg(feature = "gui")]
    fn show_mixer_canvas(&mut self, ui: &mut egui::Ui) {
        let bar_h = 28.0;
        let canvas_height = (self.tracks.len() as f32 * 36.0 + 36.0 + bar_h).max(300.0);
        egui::Frame::none()
            .fill(Color32::from_rgb(10, 14, 24))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(28, 40, 60)))
            .rounding(Rounding::same(6.0))
            .show(ui, |ui| {
                ui.set_height(canvas_height);
                let (resp, painter) = ui.allocate_painter(Vec2::new(ui.available_width(), canvas_height), egui::Sense::click_and_drag());
                let rect = resp.rect;

                let num_ch = self.tracks.len();
                let master_w = 68.0;
                let ch_area_w = rect.width() - master_w - 12.0;
                let strip_w = (ch_area_w / num_ch as f32).max(44.0);
                let is_pro = self.top_bar_state.is_pro_mode;

                // Mixer interactions: Mute, Solo, Volume Fader, Pan Pot, 1-Click Pro View Launchers, Track Selection
                let pointer_pos = resp.interact_pointer_pos().or_else(|| ui.input(|i| i.pointer.latest_pos()));
                let is_interacting = resp.clicked() || resp.dragged() || ui.input(|i| i.pointer.primary_down() || i.pointer.primary_clicked());
                let is_click = resp.clicked() || ui.input(|i| i.pointer.primary_clicked() || (i.pointer.primary_down() && !resp.dragged()));
                let is_double = resp.double_clicked();
                let drag_delta = resp.drag_delta();

                let mut track_to_open_auto: Option<u64> = None;
                let mut master_to_open_auto = false;

                // 1. Top Bar inside Mixer: Title, 1-Click Pro View Switchers, Quick Utilities
                painter.rect_filled(Rect::from_min_size(rect.min, Vec2::new(rect.width(), bar_h)), 0.0, Color32::from_rgb(14, 20, 32));
                painter.text(egui::pos2(rect.left() + 12.0, rect.top() + 7.0), egui::Align2::LEFT_TOP, "CONSOLE MIXER (PRO CHANNEL STRIPS & MASTER BUS)", FontId::proportional(11.0), Color32::from_rgb(56, 189, 248));

                let arr_header_rect = Rect::from_min_size(egui::pos2(rect.left() + 320.0, rect.top() + 4.0), Vec2::new(75.0, 20.0));
                let pno_header_rect = Rect::from_min_size(egui::pos2(arr_header_rect.right() + 4.0, rect.top() + 4.0), Vec2::new(85.0, 20.0));
                let mod_header_rect = Rect::from_min_size(egui::pos2(pno_header_rect.right() + 4.0, rect.top() + 4.0), Vec2::new(75.0, 20.0));
                let stg_header_rect = Rect::from_min_size(egui::pos2(mod_header_rect.right() + 4.0, rect.top() + 4.0), Vec2::new(65.0, 20.0));

                let unity_all_rect = Rect::from_min_size(egui::pos2(stg_header_rect.right() + 8.0, rect.top() + 4.0), Vec2::new(70.0, 20.0));
                let mute_all_rect = Rect::from_min_size(egui::pos2(unity_all_rect.right() + 4.0, rect.top() + 4.0), Vec2::new(65.0, 20.0));
                let auto_header_rect = Rect::from_min_size(egui::pos2(mute_all_rect.right() + 4.0, rect.top() + 4.0), Vec2::new(60.0, 20.0));

                let arr_hov = pointer_pos.map(|p| arr_header_rect.contains(p)).unwrap_or(false);
                let pno_hov = pointer_pos.map(|p| pno_header_rect.contains(p)).unwrap_or(false);
                let mod_hov = pointer_pos.map(|p| mod_header_rect.contains(p)).unwrap_or(false);
                let stg_hov = pointer_pos.map(|p| stg_header_rect.contains(p)).unwrap_or(false);
                let unity_hov = pointer_pos.map(|p| unity_all_rect.contains(p)).unwrap_or(false);
                let mute_hov = pointer_pos.map(|p| mute_all_rect.contains(p)).unwrap_or(false);
                let auto_hov = pointer_pos.map(|p| auto_header_rect.contains(p)).unwrap_or(false);

                let draw_nav_btn = |p: &egui::Painter, r: Rect, label: &str, is_hov: bool, text_col: Color32| {
                    let bg = if is_hov { Color32::from_rgb(32, 44, 68) } else { Color32::from_rgb(20, 28, 44) };
                    p.rect_filled(r, 3.0, bg);
                    p.rect_stroke(r, 3.0, Stroke::new(1.0_f32, Color32::from_rgb(56, 189, 248)));
                    p.text(r.center(), egui::Align2::CENTER_CENTER, label, FontId::proportional(9.0), text_col);
                };

                draw_nav_btn(&painter, arr_header_rect, "📋 Arranger", arr_hov, Color32::from_rgb(200, 215, 235));
                draw_nav_btn(&painter, pno_header_rect, "🎹 Piano Roll", pno_hov, Color32::from_rgb(200, 215, 235));
                draw_nav_btn(&painter, mod_header_rect, "∿ Modular", mod_hov, Color32::from_rgb(200, 215, 235));
                draw_nav_btn(&painter, stg_header_rect, "🎭 Stage", stg_hov, Color32::from_rgb(200, 215, 235));
                draw_nav_btn(&painter, unity_all_rect, "↺ Unity All", unity_hov, Color32::from_rgb(168, 85, 247));
                draw_nav_btn(&painter, mute_all_rect, "🔇 Mute All", mute_hov, Color32::from_rgb(239, 68, 68));
                draw_nav_btn(&painter, auto_header_rect, "📈 Auto", auto_hov, Color32::from_rgb(245, 158, 11));

                if let Some(pos) = pointer_pos {
                    if is_click {
                        if arr_header_rect.contains(pos) {
                            self.top_bar_state.active_tab = crate::views::modern_top_bar::ModernViewTab::Arranger;
                        } else if pno_header_rect.contains(pos) {
                            self.top_bar_state.active_tab = crate::views::modern_top_bar::ModernViewTab::PianoRoll;
                        } else if mod_header_rect.contains(pos) {
                            self.top_bar_state.active_tab = crate::views::modern_top_bar::ModernViewTab::Modular;
                        } else if stg_header_rect.contains(pos) {
                            self.top_bar_state.active_tab = crate::views::modern_top_bar::ModernViewTab::Performance;
                        } else if unity_all_rect.contains(pos) {
                            self.reset_all_channel_faders();
                        } else if mute_all_rect.contains(pos) {
                            self.toggle_all_tracks_mute();
                        } else if auto_header_rect.contains(pos) {
                            if let Some(tr) = self.tracks.get(self.selected_track_idx) {
                                track_to_open_auto = Some(tr.id);
                            }
                        }
                    }
                }

                let top_offset = rect.top() + bar_h + 4.0;
                let strip_h = (canvas_height - bar_h - 8.0).max(180.0);

                if let Some(pos) = pointer_pos {
                    if is_interacting {
                        let mut selected_idx = None;
                        let mut nav_to_tab: Option<crate::views::modern_top_bar::ModernViewTab> = None;
                        for (idx, track) in self.tracks.iter_mut().enumerate() {
                            let sx = rect.left() + idx as f32 * strip_w;
                            let strip_rect = Rect::from_min_size(egui::pos2(sx, top_offset), Vec2::new(strip_w - 4.0, strip_h));
                            if strip_rect.contains(pos) {
                                let pan_y = strip_rect.top() + 32.0;
                                let pan_center = egui::pos2(strip_rect.center().x, pan_y);
                                let m_rect = Rect::from_min_size(egui::pos2(strip_rect.left() + 4.0, pan_y + 14.0), Vec2::new((strip_rect.width() - 10.0) / 2.0, 16.0));
                                let s_rect = Rect::from_min_size(egui::pos2(m_rect.right() + 2.0, pan_y + 14.0), Vec2::new(m_rect.width(), 16.0));
                                let fader_top = s_rect.bottom() + 10.0;
                                let fader_bot = strip_rect.bottom() - 20.0;

                                // 1-Click Pro View Launchers hit testing
                                let pro_nav_y = strip_rect.top() + 20.0;
                                let arr_btn = Rect::from_min_size(egui::pos2(strip_rect.left() + 3.0, pro_nav_y), Vec2::new(10.0, 10.0));
                                let p_btn = Rect::from_min_size(egui::pos2(arr_btn.right() + 2.0, pro_nav_y), Vec2::new(10.0, 10.0));
                                let mod_btn = Rect::from_min_size(egui::pos2(p_btn.right() + 2.0, pro_nav_y), Vec2::new(10.0, 10.0));
                                let auto_btn = Rect::from_min_size(egui::pos2(mod_btn.right() + 2.0, pro_nav_y), Vec2::new(10.0, 10.0));

                                if is_pro && arr_btn.contains(pos) && is_click {
                                    selected_idx = Some(idx);
                                    nav_to_tab = Some(crate::views::modern_top_bar::ModernViewTab::Arranger);
                                } else if is_pro && p_btn.contains(pos) && is_click {
                                    selected_idx = Some(idx);
                                    nav_to_tab = Some(crate::views::modern_top_bar::ModernViewTab::PianoRoll);
                                } else if is_pro && mod_btn.contains(pos) && is_click {
                                    selected_idx = Some(idx);
                                    nav_to_tab = Some(crate::views::modern_top_bar::ModernViewTab::Modular);
                                } else if is_pro && auto_btn.contains(pos) && is_click {
                                    selected_idx = Some(idx);
                                    track_to_open_auto = Some(track.id);
                                } else if pos.distance(pan_center) <= 10.0 {
                                    // Interactive Pan Pot Dragging, Clicking & Double-Click Reset
                                    if is_double {
                                        track.pan = 0.0;
                                    } else if resp.dragged() {
                                        let speed = if ui.input(|i| i.modifiers.shift) { 0.003 } else { 0.015 };
                                        track.pan = (track.pan + (drag_delta.x - drag_delta.y) * speed).clamp(-1.0, 1.0);
                                    } else if is_click {
                                        let offset = ((pos.x - pan_center.x) / 8.0).clamp(-1.0, 1.0);
                                        track.pan = offset;
                                    }
                                    selected_idx = Some(idx);
                                } else if m_rect.contains(pos) && is_click {
                                    track.is_muted = !track.is_muted;
                                    selected_idx = Some(idx);
                                } else if s_rect.contains(pos) && is_click {
                                    track.is_soloed = !track.is_soloed;
                                    selected_idx = Some(idx);
                                } else if pos.y >= fader_top - 6.0 && pos.y <= fader_bot + 6.0 {
                                    if is_double {
                                        track.gain = 1.0; // Double click resets to 0.0 dB unity
                                    } else {
                                        let norm = ((fader_bot - pos.y) / (fader_bot - fader_top)).clamp(0.0, 1.0);
                                        track.gain = norm * 1.5;
                                    }
                                    selected_idx = Some(idx);
                                } else if is_click {
                                    selected_idx = Some(idx);
                                }
                            }
                        }
                        if let Some(tab) = nav_to_tab {
                            self.top_bar_state.active_tab = tab;
                        }
                        if let Some(s_idx) = selected_idx {
                            self.select_track_for_mixer(s_idx);
                        }
                    }
                }

                // Draw Track Channel Strips 1..8
                for (idx, track) in self.tracks.iter().enumerate() {
                    let sx = rect.left() + idx as f32 * strip_w;
                    let strip_rect = Rect::from_min_size(egui::pos2(sx, top_offset), Vec2::new(strip_w - 4.0, strip_h));
                    let is_sel = idx == self.selected_track_idx;
                    let bg = if is_sel { Color32::from_rgb(18, 26, 42) } else { Color32::from_rgb(12, 16, 26) };
                    painter.rect_filled(strip_rect, 4.0, bg);
                    painter.rect_stroke(strip_rect, 4.0, Stroke::new(1.0_f32, if is_sel { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(28, 38, 56) }));

                    // Header color pill + Name
                    let col = Color32::from_rgb(track.color_rgb[0], track.color_rgb[1], track.color_rgb[2]);
                    painter.rect_filled(Rect::from_min_size(egui::pos2(strip_rect.left() + 4.0, strip_rect.top() + 4.0), Vec2::new(strip_rect.width() - 8.0, 4.0)), 2.0, col);
                    painter.text(egui::pos2(strip_rect.center().x, strip_rect.top() + 11.0), egui::Align2::CENTER_TOP, format!("{}. {}", idx + 1, track.name), FontId::proportional(9.5), Color32::from_rgb(241, 245, 249));

                    // 1-Click Pro View Launchers [📋] [🎹] [∿] [📈] in Pro Mode
                    if is_pro {
                        let pro_nav_y = strip_rect.top() + 20.0;
                        let arr_btn = Rect::from_min_size(egui::pos2(strip_rect.left() + 3.0, pro_nav_y), Vec2::new(10.0, 10.0));
                        let p_btn = Rect::from_min_size(egui::pos2(arr_btn.right() + 2.0, pro_nav_y), Vec2::new(10.0, 10.0));
                        let mod_btn = Rect::from_min_size(egui::pos2(p_btn.right() + 2.0, pro_nav_y), Vec2::new(10.0, 10.0));
                        let auto_btn = Rect::from_min_size(egui::pos2(mod_btn.right() + 2.0, pro_nav_y), Vec2::new(10.0, 10.0));

                        painter.rect_filled(arr_btn, 2.0, Color32::from_rgb(22, 30, 46));
                        painter.text(arr_btn.center(), egui::Align2::CENTER_CENTER, "📋", FontId::proportional(7.0), Color32::from_rgb(200, 215, 235));

                        painter.rect_filled(p_btn, 2.0, Color32::from_rgb(22, 30, 46));
                        painter.text(p_btn.center(), egui::Align2::CENTER_CENTER, "🎹", FontId::proportional(7.0), Color32::from_rgb(168, 85, 247));

                        painter.rect_filled(mod_btn, 2.0, Color32::from_rgb(22, 30, 46));
                        painter.text(mod_btn.center(), egui::Align2::CENTER_CENTER, "∿", FontId::proportional(7.5), Color32::from_rgb(56, 189, 248));

                        painter.rect_filled(auto_btn, 2.0, Color32::from_rgb(22, 30, 46));
                        painter.text(auto_btn.center(), egui::Align2::CENTER_CENTER, "📈", FontId::proportional(7.0), Color32::from_rgb(245, 158, 11));
                    }

                    // Tactile Rotary Pan Pot with 12 o'clock center tick and angle needle
                    let pan_y = strip_rect.top() + 32.0;
                    let pan_center = egui::pos2(strip_rect.center().x, pan_y);
                    painter.circle_filled(pan_center, 8.0, Color32::from_rgb(18, 24, 36));
                    painter.circle_stroke(pan_center, 8.0, Stroke::new(1.0_f32, Color32::from_rgb(45, 60, 85)));

                    // 12 o'clock center tick
                    painter.line_segment([egui::pos2(pan_center.x, pan_center.y - 8.0), egui::pos2(pan_center.x, pan_center.y - 5.0)], Stroke::new(1.0_f32, Color32::from_rgb(148, 163, 184)));

                    // Dynamic angle needle
                    let pan_angle = track.pan * 135.0_f32.to_radians() - std::f32::consts::FRAC_PI_2;
                    let needle_col = if track.pan.abs() < 0.05 {
                        Color32::from_rgb(200, 215, 235)
                    } else if track.pan < 0.0 {
                        Color32::from_rgb(245, 158, 11) // Warm amber for Left
                    } else {
                        Color32::from_rgb(56, 189, 248)  // Radiant cyan for Right
                    };
                    let nx = pan_center.x + pan_angle.cos() * 6.5;
                    let ny = pan_center.y + pan_angle.sin() * 6.5;
                    painter.line_segment([pan_center, egui::pos2(nx, ny)], Stroke::new(1.5_f32, needle_col));
                    painter.circle_filled(pan_center, 2.0, needle_col);

                    // Mute / Solo Buttons
                    let m_rect = Rect::from_min_size(egui::pos2(strip_rect.left() + 4.0, pan_y + 14.0), Vec2::new((strip_rect.width() - 10.0) / 2.0, 16.0));
                    let s_rect = Rect::from_min_size(egui::pos2(m_rect.right() + 2.0, pan_y + 14.0), Vec2::new(m_rect.width(), 16.0));
                    painter.rect_filled(m_rect, 2.0, if track.is_muted { Color32::from_rgb(239, 68, 68) } else { Color32::from_rgb(24, 34, 50) });
                    painter.text(m_rect.center(), egui::Align2::CENTER_CENTER, "M", FontId::proportional(8.0), Color32::WHITE);
                    painter.rect_filled(s_rect, 2.0, if track.is_soloed { Color32::from_rgb(234, 179, 8) } else { Color32::from_rgb(24, 34, 50) });
                    painter.text(s_rect.center(), egui::Align2::CENTER_CENTER, "S", FontId::proportional(8.0), Color32::WHITE);

                    // Fader Slot & Thumb
                    let fader_top = s_rect.bottom() + 10.0;
                    let fader_bot = strip_rect.bottom() - 20.0;
                    let fader_x = strip_rect.left() + strip_rect.width() * 0.35;
                    painter.line_segment([egui::pos2(fader_x, fader_top), egui::pos2(fader_x, fader_bot)], Stroke::new(2.0_f32, Color32::from_rgb(8, 12, 18)));
                    let thumb_y = fader_bot - (track.gain / 1.5).clamp(0.0, 1.0) * (fader_bot - fader_top);
                    let thumb_r = Rect::from_center_size(egui::pos2(fader_x, thumb_y), Vec2::new(18.0, 8.0));
                    painter.rect_filled(thumb_r, 2.0, col);
                    painter.rect_stroke(thumb_r, 2.0, Stroke::new(1.0_f32, Color32::WHITE));

                    // Stereo Peak Meter Ladder
                    let meter_x = strip_rect.right() - 14.0;
                    let meter_w = 8.0;
                    let meter_rect = Rect::from_min_size(egui::pos2(meter_x, fader_top), Vec2::new(meter_w, fader_bot - fader_top));
                    painter.rect_filled(meter_rect, 1.0, Color32::from_rgb(6, 10, 16));
                    let fill_m_h = (meter_rect.height() * (track.gain / 1.5).clamp(0.0, 1.0)).max(2.0);
                    let fill_m_rect = Rect::from_min_max(egui::pos2(meter_rect.left(), meter_rect.bottom() - fill_m_h), meter_rect.right_bottom());
                    painter.rect_filled(fill_m_rect, 1.0, col);

                    // dB readout
                    painter.text(egui::pos2(strip_rect.center().x, strip_rect.bottom() - 10.0), egui::Align2::CENTER_CENTER, format!("{:.1}dB", (track.gain - 1.0) * 12.0), FontId::proportional(8.0), Color32::from_rgb(148, 163, 184));
                }

                // Master Bus Strip on the Right
                let m_x = rect.right() - master_w - 4.0;
                let m_strip = Rect::from_min_size(egui::pos2(m_x, top_offset), Vec2::new(master_w, strip_h));
                painter.rect_filled(m_strip, 4.0, Color32::from_rgb(16, 24, 38));
                painter.rect_stroke(m_strip, 4.0, Stroke::new(1.5_f32, Color32::from_rgb(56, 189, 248)));
                painter.text(egui::pos2(m_strip.center().x, m_strip.top() + 6.0), egui::Align2::CENTER_TOP, "MASTER", FontId::proportional(11.0), Color32::from_rgb(56, 189, 248));

                // Master Mute & Mono Audition Buttons Side-by-Side
                let btn_w = (m_strip.width() - 15.0) / 2.0;
                let m_mute_rect = Rect::from_min_size(egui::pos2(m_strip.left() + 5.0, m_strip.top() + 18.0), Vec2::new(btn_w, 11.0));
                let m_mono_rect = Rect::from_min_size(egui::pos2(m_mute_rect.right() + 4.0, m_strip.top() + 18.0), Vec2::new(btn_w, 11.0));

                let m_mute_bg = if self.is_master_muted { Color32::from_rgb(239, 68, 68) } else { Color32::from_rgb(24, 34, 52) };
                painter.rect_filled(m_mute_rect, 2.0, m_mute_bg);
                painter.text(m_mute_rect.center(), egui::Align2::CENTER_CENTER, if self.is_master_muted { "MUTED" } else { "MUTE" }, FontId::proportional(7.0), Color32::WHITE);

                let m_mono_bg = if self.is_master_mono { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(24, 34, 52) };
                let m_mono_fg = if self.is_master_mono { Color32::from_rgb(10, 14, 24) } else { Color32::from_rgb(148, 163, 184) };
                painter.rect_filled(m_mono_rect, 2.0, m_mono_bg);
                painter.text(m_mono_rect.center(), egui::Align2::CENTER_CENTER, if self.is_master_mono { "MONO" } else { "ST" }, FontId::proportional(7.0), m_mono_fg);

                let m_fader_top = m_strip.top() + 32.0;
                let m_fader_bot = m_strip.bottom() - 24.0;
                let m_fader_x = m_strip.left() + m_strip.width() * 0.35;

                let m_unity_rect = Rect::from_min_size(egui::pos2(m_strip.left() + 5.0, m_fader_bot + 1.0), Vec2::new(btn_w, 10.0));
                let m_auto_rect = Rect::from_min_size(egui::pos2(m_unity_rect.right() + 4.0, m_fader_bot + 1.0), Vec2::new(btn_w, 10.0));

                if let Some(pos) = pointer_pos {
                    if is_interacting && m_strip.contains(pos) {
                        if m_mute_rect.contains(pos) && is_click {
                            self.toggle_master_mute();
                        } else if m_mono_rect.contains(pos) && is_click {
                            self.toggle_master_mono();
                        } else if m_unity_rect.contains(pos) && is_click {
                            self.reset_master_gain();
                        } else if m_auto_rect.contains(pos) && is_click {
                            master_to_open_auto = true;
                        } else if pos.y >= m_fader_top - 6.0 && pos.y <= m_fader_bot + 6.0 {
                            if is_double {
                                self.reset_master_gain();
                            } else {
                                let norm = ((m_fader_bot - pos.y) / (m_fader_bot - m_fader_top)).clamp(0.0, 1.0);
                                self.top_bar_state.master_gain = norm * 2.0;
                                self.is_master_muted = false;
                            }
                        }
                    }
                }

                // Draw Master fader slot & thumb
                painter.line_segment([egui::pos2(m_fader_x, m_fader_top), egui::pos2(m_fader_x, m_fader_bot)], Stroke::new(2.5_f32, Color32::from_rgb(8, 12, 18)));
                let m_norm = (self.top_bar_state.master_gain / 2.0).clamp(0.0, 1.0);
                let m_thumb_y = m_fader_bot - m_norm * (m_fader_bot - m_fader_top);
                let m_thumb_r = Rect::from_center_size(egui::pos2(m_fader_x, m_thumb_y), Vec2::new(20.0, 9.0));
                painter.rect_filled(m_thumb_r, 2.0, Color32::from_rgb(56, 189, 248));
                painter.rect_stroke(m_thumb_r, 2.0, Stroke::new(1.0_f32, Color32::WHITE));

                // Master Unity Reset & Auto Buttons [0dB] [📈 Auto]
                painter.rect_filled(m_unity_rect, 2.0, Color32::from_rgb(22, 30, 46));
                painter.text(m_unity_rect.center(), egui::Align2::CENTER_CENTER, "0dB", FontId::proportional(7.0), Color32::from_rgb(168, 85, 247));

                painter.rect_filled(m_auto_rect, 2.0, Color32::from_rgb(22, 30, 46));
                painter.text(m_auto_rect.center(), egui::Align2::CENTER_CENTER, "📈 Auto", FontId::proportional(7.0), Color32::from_rgb(245, 158, 11));

                // Dual Stereo Master VU Meter
                let m_meter_x = m_strip.right() - 18.0;
                let m_meter_rect = Rect::from_min_size(egui::pos2(m_meter_x, m_fader_top), Vec2::new(12.0, m_fader_bot - m_fader_top));
                painter.rect_filled(m_meter_rect, 1.0, Color32::from_rgb(6, 10, 16));
                let m_fill_h = (m_meter_rect.height() * m_norm).max(2.0);
                let m_fill_rect = Rect::from_min_max(egui::pos2(m_meter_rect.left(), m_meter_rect.bottom() - m_fill_h), m_meter_rect.right_bottom());
                painter.rect_filled(m_fill_rect, 1.0, if self.is_master_muted { Color32::from_rgb(100, 116, 139) } else { Color32::from_rgb(16, 185, 129) });

                let m_db = if self.top_bar_state.master_gain > 0.001 { (self.top_bar_state.master_gain - 1.0) * 12.0 } else { -96.0 };
                let m_txt = if self.is_master_muted { "-∞ dB".to_string() } else { format!("{:.1}dB", m_db) };
                painter.text(egui::pos2(m_strip.center().x, m_strip.bottom() - 10.0), egui::Align2::CENTER_CENTER, m_txt, FontId::proportional(8.0), Color32::from_rgb(56, 189, 248));

                // Handle pending automation window requests from mixer strip
                if let Some(tid) = track_to_open_auto {
                    self.open_track_automation_editor(tid, "gain");
                }
                if master_to_open_auto {
                    self.open_master_automation_editor();
                }
            });
    }

    #[cfg(feature = "gui")]
    fn show_stage_canvas(&mut self, ui: &mut egui::Ui) {
        let cur_track_id = self.tracks.get(self.selected_track_idx).map(|t| t.id).unwrap_or(1);
        let cur_track_name = self.tracks.get(self.selected_track_idx).map(|t| t.name.clone()).unwrap_or_else(|| "Track 1".to_string());

        // 0. Stage Pro Top Toolbar (Two-Tier UX: Novice vs Pro)
        ui.horizontal(|ui| {
            ui.label(RichText::new("🎭 Stage").font(FontId::proportional(12.0)).strong().color(Color32::from_rgb(56, 189, 248)));
            ui.add_space(6.0);

            // Track Selector ComboBox
            let mut track_to_select = None;
            egui::ComboBox::from_id_source("stage_track_selector")
                .selected_text(RichText::new(format!("🎚 {}", cur_track_name)).font(FontId::proportional(10.5)).color(Color32::from_rgb(56, 189, 248)))
                .show_ui(ui, |ui| {
                    for (t_idx, tr) in self.tracks.iter().enumerate() {
                        let is_sel = t_idx == self.selected_track_idx;
                        if ui.selectable_label(is_sel, format!("{}: {}", tr.id, tr.name)).clicked() {
                            track_to_select = Some(t_idx);
                        }
                    }
                });
            if let Some(t_idx) = track_to_select {
                self.select_track_for_stage(t_idx);
            }

            ui.add_space(4.0);

            // 1-Click Pro View Switchers
            if ui.button(RichText::new("📋 Arranger").font(FontId::proportional(10.0)).color(Color32::from_rgb(200, 215, 235))).clicked() {
                self.top_bar_state.active_tab = crate::views::modern_top_bar::ModernViewTab::Arranger;
            }
            if ui.button(RichText::new("🎹 Piano Roll").font(FontId::proportional(10.0)).color(Color32::from_rgb(200, 215, 235))).clicked() {
                self.top_bar_state.active_tab = crate::views::modern_top_bar::ModernViewTab::PianoRoll;
            }
            if ui.button(RichText::new("∿ Modular").font(FontId::proportional(10.0)).color(Color32::from_rgb(200, 215, 235))).clicked() {
                self.top_bar_state.active_tab = crate::views::modern_top_bar::ModernViewTab::Modular;
            }
            if ui.button(RichText::new("🎚 Mixer").font(FontId::proportional(10.0)).color(Color32::from_rgb(200, 215, 235))).clicked() {
                self.top_bar_state.active_tab = crate::views::modern_top_bar::ModernViewTab::Mixer;
            }

            ui.add_space(4.0);

            // 1-Click Live Parameter Automation Launcher
            egui::ComboBox::from_id_source("stage_auto_selector")
                .selected_text(RichText::new("📈 Auto ▾").font(FontId::proportional(10.0)).color(Color32::from_rgb(234, 179, 8)))
                .show_ui(ui, |ui| {
                    if ui.selectable_label(false, "📈 Gain (Volume)").clicked() {
                        self.open_track_automation_editor(cur_track_id, "gain");
                    }
                    if ui.selectable_label(false, "📈 Stereo Pan").clicked() {
                        self.open_track_automation_editor(cur_track_id, "pan");
                    }
                    if ui.selectable_label(false, "📈 Mute Gate").clicked() {
                        self.open_track_automation_editor(cur_track_id, "mute");
                    }
                    if ui.selectable_label(false, "📈 Solo Audition").clicked() {
                        self.open_track_automation_editor(cur_track_id, "solo");
                    }
                    if ui.selectable_label(false, "📈 Filter Cutoff").clicked() {
                        self.open_track_automation_editor(cur_track_id, "cutoff");
                    }
                    if ui.selectable_label(false, "📈 Resonance").clicked() {
                        self.open_track_automation_editor(cur_track_id, "resonance");
                    }
                    if ui.selectable_label(false, "📈 Decay Time").clicked() {
                        self.open_track_automation_editor(cur_track_id, "decay");
                    }
                    if ui.selectable_label(false, "📈 Drive / Saturator").clicked() {
                        self.open_track_automation_editor(cur_track_id, "drive");
                    }
                });

            ui.add_space(4.0);

            // Launch Quantization Selector
            egui::ComboBox::from_id_source("stage_quantize_selector")
                .selected_text(RichText::new(format!("🧲 {}", self.stage_quantize.name())).font(FontId::proportional(10.0)).color(Color32::from_rgb(168, 85, 247)))
                .show_ui(ui, |ui| {
                    for q in [
                        StageLaunchQuantize::Bar1,
                        StageLaunchQuantize::BarHalf,
                        StageLaunchQuantize::Beat1,
                        StageLaunchQuantize::Instant,
                    ] {
                        if ui.selectable_label(self.stage_quantize == q, q.name()).clicked() {
                            self.stage_quantize = q;
                        }
                    }
                });

            ui.add_space(4.0);

            // Tap Tempo Button & BPM Display
            let tap_btn = egui::Button::new(RichText::new(format!("⏱ TAP ({:.1} BPM)", self.top_bar_state.bpm)).font(FontId::proportional(10.0)).color(Color32::from_rgb(16, 185, 129)))
                .fill(Color32::from_rgb(16, 40, 32));
            if ui.add(tap_btn).clicked() {
                self.tap_tempo();
            }

            ui.add_space(4.0);

            // Stop All Clips Killswitch
            let stop_btn = egui::Button::new(RichText::new("⏹ Stop All").font(FontId::proportional(10.0)).color(Color32::from_rgb(251, 146, 60)))
                .fill(Color32::from_rgb(45, 25, 20));
            if ui.add(stop_btn).clicked() {
                self.stop_all_clips();
            }

            ui.add_space(4.0);

            // Panic Button
            let panic_bg = if self.panic_triggered {
                Color32::from_rgb(220, 38, 38)
            } else {
                Color32::from_rgb(140, 24, 24)
            };
            let panic_btn = egui::Button::new(RichText::new("🚨 PANIC (ESC)").font(FontId::proportional(10.0)).color(Color32::WHITE))
                .fill(panic_bg);
            if ui.add(panic_btn).clicked() {
                self.trigger_panic();
            }
        });

        // ESC hotkey for Panic
        let esc_pressed = ui.input(|i| i.key_pressed(egui::Key::Escape));
        if esc_pressed {
            self.trigger_panic();
        }

        ui.add_space(4.0);

        let canvas_height = (self.tracks.len() as f32 * 36.0 + 36.0).max(280.0);
        egui::Frame::none()
            .fill(Color32::from_rgb(10, 14, 24))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(28, 40, 60)))
            .rounding(Rounding::same(6.0))
            .show(ui, |ui| {
                ui.set_height(canvas_height);
                let (resp, painter) = ui.allocate_painter(Vec2::new(ui.available_width(), canvas_height), egui::Sense::click_and_drag());
                let rect = resp.rect;

                let pointer_pos = resp.interact_pointer_pos().or_else(|| ui.input(|i| i.pointer.latest_pos()));
                let is_click = resp.clicked() || ui.input(|i| i.pointer.primary_clicked() || (i.pointer.primary_down() && !resp.dragged()));

                // Matrix Grid: Tracks x 4 Scenes + Stop Row
                let scene_names = ["1 Intro", "2 Verse", "3 Drop", "4 Outro"];
                let hdr_h = 20.0;
                let stop_h = 18.0;
                let grid_top = rect.top() + hdr_h + 6.0;
                let track_count = self.tracks.len();
                let col_w = (rect.width() - 80.0) / track_count.max(1) as f32;
                let row_h = (canvas_height - hdr_h - stop_h - 16.0) / scene_names.len() as f32;

                // Scene Launch Buttons (Leftmost column)
                let mut scene_action = None;
                for (s_idx, s_name) in scene_names.iter().enumerate() {
                    let sy = grid_top + s_idx as f32 * row_h;
                    let sc_rect = Rect::from_min_size(egui::pos2(rect.left() + 8.0, sy + 2.0), Vec2::new(64.0, row_h - 4.0));
                    let is_active_scene = self.active_scene_idx == Some(s_idx);

                    if let Some(pos) = pointer_pos {
                        if sc_rect.contains(pos) && is_click {
                            if is_active_scene {
                                scene_action = Some(None);
                            } else {
                                scene_action = Some(Some(s_idx));
                            }
                        }
                    }

                    let sc_bg = if is_active_scene {
                        Color32::from_rgb(16, 50, 40)
                    } else {
                        Color32::from_rgb(20, 28, 44)
                    };
                    painter.rect_filled(sc_rect, 3.0, sc_bg);
                    let sc_border = if is_active_scene {
                        Color32::from_rgb(16, 185, 129)
                    } else {
                        Color32::from_rgb(56, 189, 248)
                    };
                    painter.rect_stroke(sc_rect, 3.0, Stroke::new(1.2_f32, sc_border));
                    painter.text(egui::pos2(sc_rect.left() + 6.0, sc_rect.center().y), egui::Align2::LEFT_CENTER, *s_name, FontId::proportional(9.0), sc_border);
                    let launch_icon = if is_active_scene { "■" } else { "▶" };
                    painter.text(egui::pos2(sc_rect.right() - 8.0, sc_rect.center().y), egui::Align2::RIGHT_CENTER, launch_icon, FontId::proportional(8.0), if is_active_scene { Color32::from_rgb(16, 185, 129) } else { Color32::from_rgb(100, 116, 139) });
                }

                // Global Stop Row button in left column
                let left_stop_rect = Rect::from_min_size(egui::pos2(rect.left() + 8.0, grid_top + scene_names.len() as f32 * row_h + 3.0), Vec2::new(64.0, stop_h - 2.0));
                let mut do_stop_all = false;
                if let Some(pos) = pointer_pos {
                    if left_stop_rect.contains(pos) && is_click {
                        do_stop_all = true;
                    }
                }
                painter.rect_filled(left_stop_rect, 2.0, Color32::from_rgb(32, 20, 20));
                painter.rect_stroke(left_stop_rect, 2.0, Stroke::new(1.0_f32, Color32::from_rgb(239, 68, 68)));
                painter.text(left_stop_rect.center(), egui::Align2::CENTER_CENTER, "⏹ Stop All", FontId::proportional(8.5), Color32::from_rgb(252, 165, 165));

                // Grid Column Track Headers, Pads, and per-track Stop buttons
                let mut pad_track_selected = None;
                let mut clip_to_toggle = None;
                let mut track_to_stop = None;

                for (t_idx, track) in self.tracks.iter().enumerate() {
                    let col_x = rect.left() + 80.0 + t_idx as f32 * col_w;
                    let col = Color32::from_rgb(track.color_rgb[0], track.color_rgb[1], track.color_rgb[2]);

                    // Track Header
                    let hdr_rect = Rect::from_min_size(egui::pos2(col_x + 2.0, grid_top - hdr_h - 2.0), Vec2::new(col_w - 4.0, hdr_h));
                    let is_sel = t_idx == self.selected_track_idx;
                    let hdr_bg = if is_sel { Color32::from_rgb(28, 42, 65) } else { Color32::from_rgb(16, 22, 34) };
                    let border_col = if is_sel { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(28, 38, 56) };
                    painter.rect_filled(hdr_rect, 2.0, hdr_bg);
                    painter.rect_stroke(hdr_rect, 2.0, Stroke::new(1.0_f32, border_col));
                    painter.text(hdr_rect.center(), egui::Align2::CENTER_CENTER, &track.name, FontId::proportional(9.0), col);

                    if let Some(pos) = pointer_pos {
                        if hdr_rect.contains(pos) && is_click {
                            pad_track_selected = Some(t_idx);
                        }
                    }

                    // Scene clip pads
                    for (s_idx, _) in scene_names.iter().enumerate() {
                        let row_y = grid_top + s_idx as f32 * row_h;
                        let pad_rect = Rect::from_min_size(egui::pos2(col_x + 2.0, row_y + 2.0), Vec2::new(col_w - 4.0, row_h - 4.0));
                        let is_playing = track.active_clip_idx == Some(s_idx) || (self.active_scene_idx == Some(s_idx) && self.top_bar_state.is_playing);

                        if let Some(pos) = pointer_pos {
                            if pad_rect.contains(pos) && is_click {
                                pad_track_selected = Some(t_idx);
                                clip_to_toggle = Some((t_idx, s_idx));
                            }
                        }

                        let pad_bg = if is_playing {
                            Color32::from_rgba_unmultiplied(track.color_rgb[0], track.color_rgb[1], track.color_rgb[2], 75)
                        } else {
                            Color32::from_rgb(14, 18, 28)
                        };
                        painter.rect_filled(pad_rect, 3.0, pad_bg);
                        painter.rect_stroke(pad_rect, 3.0, Stroke::new(if is_playing { 1.5_f32 } else { 1.0_f32 }, if is_playing { col } else { Color32::from_rgb(28, 38, 56) }));

                        let pad_label = if is_playing { "▶ Clip" } else { "⬚" };
                        let label_col = if is_playing { col } else { Color32::from_rgb(100, 116, 139) };
                        painter.text(pad_rect.center(), egui::Align2::CENTER_CENTER, pad_label, FontId::proportional(9.0), label_col);
                    }

                    // Per-Track Stop Button
                    let tr_stop_rect = Rect::from_min_size(egui::pos2(col_x + 2.0, grid_top + scene_names.len() as f32 * row_h + 3.0), Vec2::new(col_w - 4.0, stop_h - 2.0));
                    let has_active_clip = track.active_clip_idx.is_some();
                    if let Some(pos) = pointer_pos {
                        if tr_stop_rect.contains(pos) && is_click {
                            track_to_stop = Some(t_idx);
                        }
                    }
                    let tr_stop_bg = if has_active_clip { Color32::from_rgb(45, 25, 20) } else { Color32::from_rgb(16, 20, 30) };
                    let tr_stop_border = if has_active_clip { Color32::from_rgb(251, 146, 60) } else { Color32::from_rgb(28, 38, 56) };
                    let tr_stop_label_col = if has_active_clip { Color32::from_rgb(251, 146, 60) } else { Color32::from_rgb(100, 116, 139) };
                    painter.rect_filled(tr_stop_rect, 2.0, tr_stop_bg);
                    painter.rect_stroke(tr_stop_rect, 2.0, Stroke::new(1.0_f32, tr_stop_border));
                    painter.text(tr_stop_rect.center(), egui::Align2::CENTER_CENTER, "■ Stop", FontId::proportional(8.5), tr_stop_label_col);
                }

                if do_stop_all {
                    self.stop_all_clips();
                } else if let Some(action) = scene_action {
                    match action {
                        Some(s_idx) => { self.launch_scene(s_idx); }
                        None => { self.stop_scene(); }
                    }
                } else if let Some((t_idx, s_idx)) = clip_to_toggle {
                    self.launch_track_clip(t_idx, s_idx);
                }

                if let Some(t_idx) = track_to_stop {
                    self.stop_track_clip(t_idx);
                }

                if let Some(s_idx) = pad_track_selected {
                    self.select_track_for_stage(s_idx);
                }
            });
    }

    /// Render deterministic high-fidelity PNG snapshot visualizer of the complete award-winning GUI.
    pub fn render_snapshot_png(&self, path: &str, width: usize, height: usize) -> Result<(), String> {
        let mut canvas = SnapshotCanvas::new(width, height);

        // 1. Overall Dark Workspace Background (#0B0F17)
        canvas.fill_rect(0, 0, width, height, [0x0B, 0x0F, 0x17]);

        // Proportions
        let top_h = (height as f32 * 0.065).clamp(44.0, 52.0) as usize;
        let dock_w = (width as f32 * 0.032).clamp(34.0, 42.0) as usize;
        let browser_w = (width as f32 * 0.165).clamp(160.0, 220.0) as usize;
        let insp_w = (width as f32 * 0.165).clamp(165.0, 225.0) as usize;
        let bottom_dock_h = (height as f32 * 0.325).clamp(180.0, 255.0) as usize;
        let bottom_dock_top = height.saturating_sub(bottom_dock_h);

        // ==========================================
        // 2. Top Global Navigation / Transport Bar
        // ==========================================
        canvas.fill_rect(0, 0, width, top_h, [0x0F, 0x14, 0x22]);
        canvas.draw_line(0, top_h - 1, width, top_h - 1, [0x1E, 0x28, 0x3E], 1);

        // Left Brand Stylized "K" Icon Badge + Menu Button
        let logo_box_x = 10;
        let logo_box_y = top_h / 2 - 14;
        canvas.fill_rounded_rect(logo_box_x, logo_box_y, 28, 28, 6, [0x14, 0x1C, 0x2C]);
        canvas.stroke_rounded_rect(logo_box_x, logo_box_y, 28, 28, 6, [0x26, 0x36, 0x50], 1);
        canvas.draw_stylized_k_logo(logo_box_x + 6, logo_box_y + 6, [0x00, 0xE5, 0xFF]);

        let menu_x = 44;
        let menu_w = 48;
        canvas.fill_rounded_rect(menu_x, top_h / 2 - 12, menu_w, 24, 4, [0x15, 0x1D, 0x2C]);
        canvas.stroke_rounded_rect(menu_x, top_h / 2 - 12, menu_w, 24, 4, [0x26, 0x36, 0x50], 1);
        // Menu burger lines icon
        canvas.draw_line(menu_x + 8, top_h / 2 - 4, menu_x + 16, top_h / 2 - 4, [0x94, 0xA3, 0xB8], 1);
        canvas.draw_line(menu_x + 8, top_h / 2, menu_x + 16, top_h / 2, [0x94, 0xA3, 0xB8], 1);
        canvas.draw_line(menu_x + 8, top_h / 2 + 4, menu_x + 16, top_h / 2 + 4, [0x94, 0xA3, 0xB8], 1);
        canvas.draw_text_smooth(menu_x + 20, top_h / 2 - 4, "Menu", [0xD0, 0xE0, 0xF0]);

        // Transport Stats Pill Group: BPM, KEY, SIG, CPU
        let stats_x = menu_x + menu_w + 12;
        let stats_w = 172;
        canvas.fill_rounded_rect(stats_x, top_h / 2 - 15, stats_w, 30, 4, [0x12, 0x18, 0x26]);
        canvas.stroke_rounded_rect(stats_x, top_h / 2 - 15, stats_w, 30, 4, [0x22, 0x30, 0x48], 1);

        // BPM
        canvas.draw_text_smooth(stats_x + 6, top_h / 2 - 12, "BPM", [0x64, 0x74, 0x8B]);
        canvas.draw_text_smooth(stats_x + 6, top_h / 2, &format!("{:.2}", self.top_bar_state.bpm), [0x00, 0xE5, 0xFF]);
        canvas.draw_line(stats_x + 48, top_h / 2 - 11, stats_x + 48, top_h / 2 + 11, [0x1C, 0x28, 0x3E], 1);

        // KEY
        canvas.draw_text_smooth(stats_x + 54, top_h / 2 - 12, "KEY", [0x64, 0x74, 0x8B]);
        canvas.draw_text_smooth(stats_x + 54, top_h / 2, &self.top_bar_state.key_signature, [0x00, 0xE5, 0xFF]);
        canvas.draw_line(stats_x + 84, top_h / 2 - 11, stats_x + 84, top_h / 2 + 11, [0x1C, 0x28, 0x3E], 1);

        // SIG
        canvas.draw_text_smooth(stats_x + 90, top_h / 2 - 12, "SIG", [0x64, 0x74, 0x8B]);
        canvas.draw_text_smooth(stats_x + 90, top_h / 2, &self.top_bar_state.time_signature, [0x00, 0xE5, 0xFF]);
        canvas.draw_line(stats_x + 120, top_h / 2 - 11, stats_x + 120, top_h / 2 + 11, [0x1C, 0x28, 0x3E], 1);

        // CPU
        canvas.draw_text_smooth(stats_x + 126, top_h / 2 - 12, "CPU", [0x64, 0x74, 0x8B]);
        canvas.draw_text_smooth(stats_x + 126, top_h / 2, &format!("{:.0}%", self.top_bar_state.cpu_percent), [0x00, 0xE5, 0xFF]);
        // CPU Mini meter bar
        canvas.fill_rect(stats_x + 152, top_h / 2 + 3, 14, 4, [0x08, 0x0E, 0x18]);
        canvas.fill_rect(stats_x + 152, top_h / 2 + 3, 4, 4, [0x00, 0xE5, 0xFF]);

        // Transport Controls Container
        let t_btn_x = stats_x + stats_w + 12;
        let btn_size = 28;
        let btn_y = top_h / 2 - btn_size / 2;

        // Group container
        canvas.fill_rounded_rect(t_btn_x, btn_y, 148, btn_size, 4, [0x12, 0x18, 0x26]);
        canvas.stroke_rounded_rect(t_btn_x, btn_y, 148, btn_size, 4, [0x22, 0x30, 0x48], 1);

        // Play (Solid Cyan Button)
        canvas.fill_rounded_rect(t_btn_x + 2, btn_y + 2, 26, 24, 3, [0x00, 0xE5, 0xFF]);
        canvas.draw_play_icon(t_btn_x + 11, btn_y + 8, 10, [0x00, 0x00, 0x00]);

        // Pause
        canvas.draw_pause_icon(t_btn_x + 38, btn_y + 8, 10, [0x94, 0xA3, 0xB8]);

        // Record (Red Dot)
        canvas.draw_circle_filled(t_btn_x + 68, btn_y + 14, 5, [0xEF, 0x44, 0x44]);

        // Stop
        canvas.fill_rect(t_btn_x + 94, btn_y + 9, 9, 9, [0x94, 0xA3, 0xB8]);

        // Loop Toggle
        canvas.draw_loop_icon(t_btn_x + 120, btn_y + 8, [0x00, 0xE5, 0xFF]);

        // View Mode Switcher Tabs (Right side of Top Bar)
        let tab_w = (width as f32 * 0.058).clamp(58.0, 76.0) as usize;
        let tabs = [
            ("MIXER", false),
            ("MODULAR", false),
            ("PIANO ROLL", false),
            ("ARRANGER", true),
        ];
        let mut min_tab_x = width - 16;
        for (i, (t_name, is_active)) in tabs.iter().enumerate() {
            let tx = width - 16 - (i + 1) * (tab_w + 6);
            if tx < min_tab_x {
                min_tab_x = tx;
            }
            if *is_active {
                canvas.fill_rounded_rect(tx, top_h / 2 - 13, tab_w, 26, 4, [0x10, 0x2A, 0x42]);
                canvas.stroke_rounded_rect(tx, top_h / 2 - 13, tab_w, 26, 4, [0x00, 0xE5, 0xFF], 1);
                let text_offset = (tab_w.saturating_sub(t_name.len() * 6)) / 2;
                canvas.draw_text_smooth(tx + text_offset, top_h / 2 - 4, t_name, [0x00, 0xE5, 0xFF]);
            } else {
                canvas.fill_rounded_rect(tx, top_h / 2 - 13, tab_w, 26, 4, [0x14, 0x1C, 0x2C]);
                canvas.stroke_rounded_rect(tx, top_h / 2 - 13, tab_w, 26, 4, [0x26, 0x36, 0x4C], 1);
                let text_offset = (tab_w.saturating_sub(t_name.len() * 6)) / 2;
                canvas.draw_text_smooth(tx + text_offset, top_h / 2 - 4, t_name, [0x94, 0xA3, 0xB8]);
            }
        }

        // Master Volume VU Meter (Adaptive placement between transport buttons and mode tabs)
        let vu_x = t_btn_x + 160;
        if min_tab_x > vu_x + 70 {
            let avail_vu_w = (min_tab_x - vu_x - 18).min(160);
            canvas.draw_text_smooth(vu_x, top_h / 2 - 12, "Master Vol VU Meters", [0x94, 0xA3, 0xB8]);
            if vu_x + 120 < min_tab_x {
                canvas.draw_text_smooth(vu_x + 116, top_h / 2 - 12, "+1.2dB", [0x00, 0xE5, 0xFF]);
            }

            // Dual stereo gradient meter
            let g1 = (avail_vu_w as f32 * 0.55) as usize;
            let g2 = (avail_vu_w as f32 * 0.20) as usize;
            let g3 = (avail_vu_w as f32 * 0.15) as usize;
            let g4 = avail_vu_w.saturating_sub(g1 + g2 + g3);

            for m_y in [top_h / 2, top_h / 2 + 6] {
                canvas.fill_rounded_rect(vu_x, m_y, avail_vu_w, 4, 2, [0x08, 0x0E, 0x18]);
                // Green (0..55%)
                canvas.fill_rect(vu_x, m_y, g1, 4, [0x10, 0xB9, 0x81]);
                // Yellow (55..75%)
                canvas.fill_rect(vu_x + g1, m_y, g2, 4, [0xEA, 0xB3, 0x08]);
                // Orange (75..90%)
                canvas.fill_rect(vu_x + g1 + g2, m_y, g3, 4, [0xF9, 0x73, 0x16]);
                // Pink / Red (90..100%)
                canvas.fill_rect(vu_x + g1 + g2 + g3, m_y, g4, 4, [0xEC, 0x48, 0x99]);
            }
        }

        // ==========================================
        // 3. Far Left Activity Bar (Icon Strip)
        // ==========================================
        canvas.fill_rect(0, top_h, dock_w, height - top_h, [0x0A, 0x0E, 0x17]);
        canvas.draw_line(dock_w - 1, top_h, dock_w - 1, height, [0x1C, 0x26, 0x3A], 1);

        // Top activity icons: 0: Browser, 1: Faders/Mixer, 2: Modular, 3: Wave, 4: Play clip
        let top_icons_y = [top_h + 10, top_h + 44, top_h + 78, top_h + 112, top_h + 146];
        for (i, &iy) in top_icons_y.iter().enumerate() {
            if iy + 26 < bottom_dock_top {
                let ix = dock_w / 2;
                if i == 0 {
                    // Active Browser Icon with glowing cyan frame
                    canvas.fill_rounded_rect(dock_w / 2 - 13, iy, 26, 26, 4, [0x10, 0x2A, 0x40]);
                    canvas.stroke_rounded_rect(dock_w / 2 - 13, iy, 26, 26, 4, [0x00, 0xE5, 0xFF], 1);
                    canvas.draw_activity_icon(0, ix, iy + 13, [0x00, 0xE5, 0xFF]);
                } else {
                    canvas.draw_activity_icon(i, ix, iy + 13, [0x64, 0x74, 0x8B]);
                }
            }
        }
        // Bottom activity icons: Routing, Lightbulb, Settings, Trash
        let bot_icons_y = [height.saturating_sub(110), height.saturating_sub(82), height.saturating_sub(54), height.saturating_sub(26)];
        for (i, &iy) in bot_icons_y.iter().enumerate() {
            if iy < height - 6 {
                let ix = dock_w / 2;
                canvas.draw_activity_icon(5 + i, ix, iy + 10, [0x64, 0x74, 0x8B]);
            }
        }

        // ==========================================
        // 4. Left Collapsible Asset Browser
        // ==========================================
        let browser_x = dock_w;
        canvas.fill_rect(browser_x, top_h, browser_w, height - top_h, [0x0E, 0x13, 0x20]);
        canvas.draw_line(browser_x + browser_w - 1, top_h, browser_x + browser_w - 1, height, [0x1C, 0x26, 0x3A], 1);

        // Header
        canvas.draw_text_smooth(browser_x + 12, top_h + 12, "Asset Browser", [0xF1, 0xF5, 0xF9]);
        canvas.draw_text_smooth(browser_x + browser_w - 18, top_h + 12, "v", [0x94, 0xA3, 0xB8]);

        // Categories
        let categories = [
            ("Instruments", false, 0),
            ("Audio FX", false, 1),
            ("MIDI FX", false, 2),
            ("Samples", true, 3),
        ];
        let mut cat_y = top_h + 36;
        for (name, is_sel, icon_type) in categories {
            if is_sel {
                // Highlighted Samples category with rich warm amber container
                canvas.fill_rounded_rect(browser_x + 6, cat_y, browser_w - 12, 24, 4, [0x3D, 0x26, 0x08]);
                canvas.stroke_rounded_rect(browser_x + 6, cat_y, browser_w - 12, 24, 4, [0xF5, 0x9E, 0x0B], 1);
                canvas.draw_category_icon(icon_type, browser_x + 14, cat_y + 12, [0xF5, 0x9E, 0x0B]);
                canvas.draw_text_smooth(browser_x + 28, cat_y + 5, name, [0xF5, 0x9E, 0x0B]);
            } else {
                canvas.draw_category_icon(icon_type, browser_x + 14, cat_y + 12, [0x94, 0xA3, 0xB8]);
                canvas.draw_text_smooth(browser_x + 28, cat_y + 5, name, [0x94, 0xA3, 0xB8]);
            }
            cat_y += 28;
        }

        // Subtree Folders & Files
        cat_y += 4;
        let tree_items = [
            (">  Drums", true, [0xF5, 0x9E, 0x0B]),
            (">  Synths", true, [0xF5, 0x9E, 0x0B]),
            (">  Bass", true, [0xF5, 0x9E, 0x0B]),
            ("    Drums/Synths...", false, [0xC8, 0xD7, 0xEB]),
            ("    Audit.midi", false, [0xC8, 0xD7, 0xEB]),
            ("    Sample..midi", false, [0xC8, 0xD7, 0xEB]),
            ("    Banan-feorturo...", false, [0x64, 0x74, 0x8B]),
            ("    Pads kiinods.o...", false, [0x64, 0x74, 0x8B]),
            ("    Synths Holsch...", false, [0x64, 0x74, 0x8B]),
            ("    Sample.Phops...", false, [0x64, 0x74, 0x8B]),
        ];
        for (item_name, is_folder, item_col) in tree_items {
            if cat_y + 14 < height - 10 {
                if is_folder {
                    canvas.draw_folder_icon(browser_x + 12, cat_y + 6, [0xF5, 0x9E, 0x0B]);
                    canvas.draw_text_smooth(browser_x + 24, cat_y, item_name, item_col);
                } else {
                    canvas.draw_doc_icon(browser_x + 14, cat_y + 6, [0x64, 0x74, 0x8B]);
                    canvas.draw_text_smooth(browser_x + 24, cat_y, item_name, item_col);
                }
                cat_y += 18;
            }
        }

        // ==========================================
        // 5. Right Collapsible Inspector Sidebar
        // ==========================================
        let insp_x = width - insp_w;
        canvas.fill_rect(insp_x, top_h, insp_w, height - top_h, [0x0E, 0x13, 0x20]);
        canvas.draw_line(insp_x, top_h, insp_x, height, [0x1C, 0x26, 0x3A], 1);

        // Header
        canvas.draw_text_smooth(insp_x + 12, top_h + 12, "Inspector", [0xF1, 0xF5, 0xF9]);

        // Target Box
        canvas.fill_rounded_rect(insp_x + 10, top_h + 34, insp_w - 20, 24, 4, [0x14, 0x1C, 0x2C]);
        canvas.stroke_rounded_rect(insp_x + 10, top_h + 34, insp_w - 20, 24, 4, [0x26, 0x36, 0x50], 1);
        canvas.draw_text_smooth(insp_x + 18, top_h + 40, &self.inspector_state.target_name, [0x00, 0xE5, 0xFF]);

        // Properties Section
        let prop_y = top_h + 68;
        canvas.draw_text_smooth(insp_x + 12, prop_y, "Properties", [0xC8, 0xD7, 0xEB]);
        canvas.draw_text_smooth(insp_x + insp_w - 20, prop_y, "^", [0x64, 0x74, 0x8B]);

        // Gain Fader
        let gain_y = prop_y + 18;
        canvas.draw_text_smooth(insp_x + 12, gain_y, "Gain Fader", [0x94, 0xA3, 0xB8]);
        canvas.draw_text_smooth(insp_x + insp_w - 48, gain_y, &format!("{:.1}dB", self.inspector_state.gain_db), [0x00, 0xE5, 0xFF]);

        // Gain Slider track & thumb
        canvas.fill_rounded_rect(insp_x + 12, gain_y + 14, insp_w - 24, 4, 2, [0x08, 0x0E, 0x18]);
        let gain_norm = ((self.inspector_state.gain_db + 36.0) / 48.0).clamp(0.0, 1.0);
        let g_thumb_x = insp_x + 12 + (gain_norm * (insp_w - 32) as f32) as usize;
        canvas.draw_circle_filled(g_thumb_x + 4, gain_y + 16, 5, [0x00, 0xE5, 0xFF]);

        // Pan Fader
        let pan_y = gain_y + 28;
        canvas.draw_text_smooth(insp_x + 12, pan_y, "Pan", [0x94, 0xA3, 0xB8]);
        canvas.draw_text_smooth(insp_x + insp_w - 24, pan_y, "C", [0x00, 0xE5, 0xFF]);

        canvas.fill_rounded_rect(insp_x + 12, pan_y + 14, insp_w - 24, 4, 2, [0x08, 0x0E, 0x18]);
        let p_thumb_x = insp_x + 12 + ((self.inspector_state.pan_val + 1.0) * 0.5 * (insp_w - 32) as f32) as usize;
        canvas.draw_circle_filled(p_thumb_x + 4, pan_y + 16, 5, [0xC8, 0xD7, 0xEB]);

        // Mute / Solo / Arm Quick Buttons
        let btn_row_y = pan_y + 28;
        let b_w = ((insp_w - 36) / 3).max(36);
        // Mute
        canvas.fill_rounded_rect(insp_x + 12, btn_row_y, b_w, 22, 4, [0x16, 0x20, 0x32]);
        canvas.stroke_rounded_rect(insp_x + 12, btn_row_y, b_w, 22, 4, [0x26, 0x36, 0x50], 1);
        canvas.draw_text_smooth(insp_x + 12 + b_w / 2 - 10, btn_row_y + 5, "Mute", [0x94, 0xA3, 0xB8]);

        // Solo
        canvas.fill_rounded_rect(insp_x + 12 + b_w + 6, btn_row_y, b_w, 22, 4, [0x16, 0x20, 0x32]);
        canvas.stroke_rounded_rect(insp_x + 12 + b_w + 6, btn_row_y, b_w, 22, 4, [0x26, 0x36, 0x50], 1);
        canvas.draw_text_smooth(insp_x + 12 + b_w + 6 + b_w / 2 - 10, btn_row_y + 5, "Solo", [0x94, 0xA3, 0xB8]);

        // Arm (Active Red)
        canvas.fill_rounded_rect(insp_x + 12 + (b_w + 6) * 2, btn_row_y, b_w, 22, 4, [0x4A, 0x16, 0x1E]);
        canvas.stroke_rounded_rect(insp_x + 12 + (b_w + 6) * 2, btn_row_y, b_w, 22, 4, [0xEF, 0x44, 0x44], 1);
        canvas.draw_text_smooth(insp_x + 12 + (b_w + 6) * 2 + b_w / 2 - 8, btn_row_y + 5, "Arm", [0xEF, 0x44, 0x44]);

        // Microtonal Tuning Section
        let micro_y = btn_row_y + 32;
        canvas.draw_line(insp_x + 8, micro_y, insp_x + insp_w - 8, micro_y, [0x1C, 0x26, 0x3A], 1);
        canvas.draw_text_smooth(insp_x + 12, micro_y + 8, "Microtonal Tuning", [0xC8, 0xD7, 0xEB]);
        canvas.draw_text_smooth(insp_x + insp_w - 20, micro_y + 8, "v", [0x64, 0x74, 0x8B]);

        // Scale dropdown pill
        let scale_y = micro_y + 24;
        canvas.draw_text_smooth(insp_x + 12, scale_y, "Scale", [0x94, 0xA3, 0xB8]);
        canvas.fill_rounded_rect(insp_x + 46, scale_y - 3, insp_w - 58, 20, 3, [0x14, 0x1C, 0x2C]);
        canvas.stroke_rounded_rect(insp_x + 46, scale_y - 3, insp_w - 58, 20, 3, [0x26, 0x36, 0x50], 1);
        canvas.draw_text_smooth(insp_x + 50, scale_y + 2, "A Minor Pentatonic", [0x00, 0xE5, 0xFF]);

        // Ratio Grid Boxes
        let ratio_y = scale_y + 24;
        let r_box_w = ((insp_w - 36) / 3).max(36);
        let r_boxes = [
            ("Ratio", "1:1"),
            ("Ratio", "1"),
            ("Octave", "-1"),
        ];
        for (i, (title, val)) in r_boxes.iter().enumerate() {
            let rx = insp_x + 12 + i * (r_box_w + 6);
            canvas.draw_text_smooth(rx + 4, ratio_y, title, [0x64, 0x74, 0x8B]);
            canvas.fill_rounded_rect(rx, ratio_y + 12, r_box_w, 20, 3, [0x14, 0x1C, 0x2C]);
            canvas.draw_text_smooth(rx + r_box_w / 2 - 8, ratio_y + 16, val, [0xF1, 0xF5, 0xF9]);
        }

        // ==========================================
        // 6. Main Center Arranger Canvas
        // ==========================================
        let canvas_left = browser_x + browser_w;
        let canvas_right = insp_x;
        let canvas_w = canvas_right - canvas_left;
        let ruler_h = 32;

        // Toolbar Above Ruler: '+' Track button, Snap, Tools
        canvas.fill_rect(canvas_left, top_h, canvas_w, ruler_h, [0x10, 0x16, 0x24]);
        canvas.draw_line(canvas_left, top_h + ruler_h - 1, canvas_right, top_h + ruler_h - 1, [0x1E, 0x28, 0x3E], 1);

        // '+' Button
        canvas.fill_rounded_rect(canvas_left + 8, top_h + 5, 22, 22, 3, [0x18, 0x22, 0x36]);
        canvas.stroke_rounded_rect(canvas_left + 8, top_h + 5, 22, 22, 3, [0x28, 0x38, 0x54], 1);
        canvas.draw_text_smooth(canvas_left + 15, top_h + 9, "+", [0x00, 0xE5, 0xFF]);

        // Snap and Split tool icons
        canvas.draw_text_smooth(canvas_left + 38, top_h + 9, "#", [0x94, 0xA3, 0xB8]);
        canvas.draw_text_smooth(canvas_left + 54, top_h + 9, "[ ]", [0x94, 0xA3, 0xB8]);

        // Ruler Numbers & Timeline
        let track_head_w = (canvas_w as f32 * 0.22).clamp(95.0, 135.0) as usize;
        let timeline_x = canvas_left + track_head_w;
        let timeline_w = (canvas_right - timeline_x) as f32;
        let ppb = timeline_w / 18.0;

        // Loop Bracket on Ruler (Bars 1..9)
        let l_x1 = timeline_x + (self.loop_start_beat * ppb) as usize;
        let l_x2 = timeline_x + (self.loop_end_beat * ppb) as usize;
        let l_x2_clamped = l_x2.min(canvas_right - 4);
        if l_x2_clamped > l_x1 {
            canvas.fill_rounded_rect(l_x1, top_h + 4, l_x2_clamped - l_x1, ruler_h - 8, 2, [0x0E, 0x30, 0x4C]);
            canvas.draw_line(l_x1, top_h + 4, l_x1, top_h + ruler_h - 4, [0x00, 0xE5, 0xFF], 2);
            canvas.draw_line(l_x2_clamped, top_h + 4, l_x2_clamped, top_h + ruler_h - 4, [0x00, 0xE5, 0xFF], 2);
        }

        // Major Ruler Markers (1, 5, 7, 9, 11, 13, 15, 17) & Sub-ticks
        for bar in 1..=17 {
            let bx = timeline_x + ((bar - 1) as f32 * ppb) as usize;
            if bx < canvas_right - 10 {
                let is_major = bar == 1 || bar == 5 || bar == 7 || bar == 9 || bar == 11 || bar == 13 || bar == 15 || bar == 17;
                canvas.draw_line(bx, top_h + 16, bx, top_h + ruler_h - 1, [0x26, 0x36, 0x50], 1);
                if is_major {
                    canvas.draw_text_smooth(bx + 4, top_h + 4, &format!("{}", bar), [0x00, 0xE5, 0xFF]);
                }
                canvas.draw_text_smooth(bx + 4, top_h + 16, &format!("{}", bar), [0x64, 0x74, 0x8B]);
            }
        }

        // Arranger Track Lanes
        let num_tracks = self.tracks.len();
        let avail_track_h = bottom_dock_top - (top_h + ruler_h);
        let track_row_h = avail_track_h / num_tracks.max(1);

        for (t_idx, track) in self.tracks.iter().enumerate() {
            let ty = top_h + ruler_h + t_idx * track_row_h;
            let ty_end = ty + track_row_h;
            let is_sel = t_idx == self.selected_track_idx;

            // Track Header Box
            let h_bg = if is_sel { [0x16, 0x22, 0x38] } else { [0x0E, 0x14, 0x22] };
            canvas.fill_rect(canvas_left, ty, track_head_w, track_row_h, h_bg);
            canvas.draw_line(canvas_left, ty_end - 1, canvas_left + track_head_w, ty_end - 1, [0x1C, 0x26, 0x3A], 1);
            canvas.draw_line(canvas_left + track_head_w - 1, ty, canvas_left + track_head_w - 1, ty_end, [0x1C, 0x26, 0x3A], 1);

            // Left track color stripe
            canvas.fill_rect(canvas_left, ty, 3, track_row_h, track.color_rgb);

            // Track Index & Name
            canvas.draw_text_smooth(canvas_left + 6, ty + 7, &format!("{}", t_idx + 1), [0x64, 0x74, 0x8B]);
            canvas.draw_text_smooth(canvas_left + 18, ty + 7, &track.name, [0xF1, 0xF5, 0xF9]);

            // Mini Colored Gain Meter Gradient Pill & Status Icons
            if track_head_w > 85 {
                let pill_w = 28;
                let px = canvas_left + track_head_w - pill_w - 22;
                canvas.fill_rounded_rect(px, ty + 8, pill_w, 6, 2, [0x08, 0x0E, 0x16]);
                let fill_len = ((track.gain / 1.5).clamp(0.0, 1.0) * pill_w as f32) as usize;
                canvas.fill_rounded_rect(px, ty + 8, fill_len, 6, 2, track.color_rgb);

                // Mini channel routing/meter glyph icons
                canvas.draw_line(canvas_left + track_head_w - 16, ty + 7, canvas_left + track_head_w - 16, ty + 13, [0x64, 0x74, 0x8B], 1);
                canvas.draw_line(canvas_left + track_head_w - 14, ty + 8, canvas_left + track_head_w - 10, ty + 8, [0x64, 0x74, 0x8B], 1);
                canvas.draw_line(canvas_left + track_head_w - 14, ty + 10, canvas_left + track_head_w - 10, ty + 10, [0x64, 0x74, 0x8B], 1);
                canvas.draw_line(canvas_left + track_head_w - 14, ty + 12, canvas_left + track_head_w - 10, ty + 12, [0x64, 0x74, 0x8B], 1);
            }

            // Track Timeline Lane Background
            let lane_bg = if is_sel { [0x12, 0x1C, 0x2E] } else { [0x0A, 0x0E, 0x17] };
            canvas.fill_rect(timeline_x, ty, canvas_right - timeline_x, track_row_h, lane_bg);
            canvas.draw_line(timeline_x, ty_end - 1, canvas_right, ty_end - 1, [0x16, 0x20, 0x32], 1);

            if is_sel {
                // Highlight border around selected track lane (Synth 1)
                canvas.stroke_rounded_rect(timeline_x, ty, canvas_right - timeline_x, track_row_h, 2, [0x00, 0xE5, 0xFF], 1);
            }

            // Bar grid lines across lane
            for bar in 1..=17 {
                let bx = timeline_x + ((bar - 1) as f32 * ppb) as usize;
                if bx < canvas_right {
                    canvas.draw_line(bx, ty, bx, ty_end - 1, [0x14, 0x1C, 0x2E], 1);
                }
            }

            // Clip Block
            let cx_start = timeline_x + (track.clip_start_beat * ppb) as usize;
            let cx_len = (track.clip_length_beats * ppb) as usize;
            let cx_end = (cx_start + cx_len).min(canvas_right - 4);

            if cx_end > cx_start + 8 {
                let clip_h = track_row_h.saturating_sub(4);
                let clip_y = ty + 2;

                // Soft glowing fill
                let mut soft_fill = track.color_rgb;
                soft_fill[0] = (soft_fill[0] as f32 * 0.18) as u8;
                soft_fill[1] = (soft_fill[1] as f32 * 0.18) as u8;
                soft_fill[2] = (soft_fill[2] as f32 * 0.18) as u8;
                canvas.fill_rounded_rect(cx_start, clip_y, cx_end - cx_start, clip_h, 4, soft_fill);
                canvas.stroke_rounded_rect(cx_start, clip_y, cx_end - cx_start, clip_h, 4, track.color_rgb, 1);

                // High-Fidelity DAW Clip Waveform / MIDI Rendering
                if track.is_audio {
                    let clip_w_px = cx_end - cx_start;
                    let mid_y = clip_y + clip_h / 2;
                    let max_amp = (clip_h / 2).saturating_sub(2);

                    if track.name == "Kick" {
                        // Dense audio waveform with sharp transients and decaying sub-bass body
                        for px in 0..clip_w_px {
                            let norm_beat = (px as f32 / (ppb.max(1.0) * 0.5)) % 1.0;
                            let transient = (-norm_beat * 6.0).exp() * 0.90;
                            let sub_tail = (norm_beat * 12.0 * std::f32::consts::PI).sin().abs() * 0.35 * (1.0 - norm_beat);
                            let h = (max_amp as f32 * (transient + sub_tail).min(0.95)) as usize;
                            let cur_x = cx_start + px;
                            if cur_x < cx_end && h > 0 {
                                canvas.draw_line(cur_x, mid_y.saturating_sub(h), cur_x, mid_y + h, track.color_rgb, 1);
                            }
                        }
                    } else if track.name == "Snare" {
                        // Continuous stereo snare audio recording with sharp transients on 2 and 4
                        for px in 0..clip_w_px {
                            let norm_beat = (px as f32 / (ppb.max(1.0) * 0.5)) % 1.0;
                            let hit_idx = (px as f32 / (ppb.max(1.0) * 0.5)) as usize;
                            let is_snare_hit = hit_idx % 2 == 1; // 2nd & 4th beats
                            let env = if is_snare_hit {
                                (-norm_beat * 3.8).exp() * 0.90 + (((px * 13) % 7) as f32 * 0.02)
                            } else {
                                0.12 * (norm_beat * 16.0 * std::f32::consts::PI).sin().abs() + (((px * 7) % 5) as f32 * 0.02)
                            };
                            let h = (max_amp as f32 * env.min(0.95)) as usize;
                            let cur_x = cx_start + px;
                            if cur_x < cx_end && h > 0 {
                                canvas.draw_line(cur_x, mid_y.saturating_sub(h), cur_x, mid_y + h, track.color_rgb, 1);
                            }
                        }
                    } else if track.name == "HiHats" {
                        // 16th-note hat bursts with crisp transients
                        for px in 0..clip_w_px {
                            let norm_16th = (px as f32 / (ppb.max(1.0) * 0.25)) % 1.0;
                            let tick_idx = (px as f32 / (ppb.max(1.0) * 0.25)) as usize;
                            let vel = if tick_idx.is_multiple_of(4) { 0.85 } else if tick_idx.is_multiple_of(2) { 0.60 } else { 0.40 };
                            let env = (-norm_16th * 4.2).exp() * vel + (((px * 11) % 5) as f32 * 0.015);
                            let h = (max_amp as f32 * env.min(0.95)) as usize;
                            let cur_x = cx_start + px;
                            if cur_x < cx_end && h > 0 {
                                canvas.draw_line(cur_x, mid_y.saturating_sub(h), cur_x, mid_y + h, track.color_rgb, 1);
                            }
                        }
                    } else {
                        // Pads: Lush continuous swelling audio waveform with rich modulation
                        for px in 0..clip_w_px {
                            let norm = px as f32 / clip_w_px.max(1) as f32;
                            let swell = (norm * 3.0 * std::f32::consts::PI).sin().abs() * 0.55 + 0.30;
                            let texture = (norm * 36.0 * std::f32::consts::PI).sin().abs() * 0.22;
                            let h = (max_amp as f32 * (swell + texture).min(0.95)) as usize;
                            let cur_x = cx_start + px;
                            if cur_x < cx_end && h > 0 {
                                canvas.draw_line(cur_x, mid_y.saturating_sub(h), cur_x, mid_y + h, track.color_rgb, 1);
                            }
                        }
                    }
                } else {
                    // MIDI note blocks with rounded pill styling
                    let clip_w_px = cx_end - cx_start;
                    if track.name == "Bassline" {
                        // Bassline groove note blocks
                        let bass_notes = [
                            (0.0, 1.2, 2),
                            (1.5, 0.8, 4),
                            (2.5, 1.0, 1),
                            (4.0, 1.2, 2),
                            (5.5, 0.8, 5),
                            (6.5, 1.0, 1),
                            (8.0, 1.2, 2),
                            (9.5, 0.8, 4),
                            (10.5, 1.0, 1),
                            (12.0, 1.2, 2),
                            (13.5, 0.8, 5),
                        ];
                        for (st_b, len_b, pitch) in bass_notes {
                            let nx = cx_start + (st_b * clip_w_px as f32 / 16.0) as usize;
                            let nw = ((len_b * clip_w_px as f32 / 16.0) as usize).max(8);
                            let ny = clip_y + 4 + pitch * 3;
                            if nx + nw <= cx_end && ny + 4 <= clip_y + clip_h {
                                canvas.fill_rounded_rect(nx, ny, nw, 4, 2, track.color_rgb);
                            }
                        }
                    } else if track.name == "Synth 1" {
                        // Lead melody note blocks
                        let lead_notes = [
                            (0.0, 2.0, 2),
                            (2.5, 1.5, 4),
                            (4.5, 2.5, 1),
                            (7.5, 1.5, 3),
                            (9.5, 2.0, 5),
                            (12.0, 3.0, 2),
                        ];
                        for (st_b, len_b, pitch) in lead_notes {
                            let nx = cx_start + (st_b * clip_w_px as f32 / 16.0) as usize;
                            let nw = ((len_b * clip_w_px as f32 / 16.0) as usize).max(10);
                            let ny = clip_y + 4 + pitch * 3;
                            if nx + nw <= cx_end && ny + 4 <= clip_y + clip_h {
                                canvas.fill_rounded_rect(nx, ny, nw, 4, 2, [0x00, 0xE5, 0xFF]);
                            }
                        }
                    } else if track.name == "Arp" {
                        // Ascending 16th-note arpeggiator staircases
                        let num_steps = 24;
                        for s in 0..num_steps {
                            let nx = cx_start + (s as f32 * clip_w_px as f32 / num_steps as f32) as usize;
                            let nw = (clip_w_px / num_steps).max(4);
                            let pitch = 6 - (s % 6);
                            let ny = clip_y + 3 + pitch * 3;
                            if nx + nw <= cx_end && ny + 3 <= clip_y + clip_h {
                                canvas.fill_rounded_rect(nx, ny, nw.saturating_sub(1), 3, 1, track.color_rgb);
                            }
                        }
                    } else {
                        // Vocals / Other MIDI
                        let vocal_notes = [
                            (0.0, 1.0, 3),
                            (1.2, 1.0, 2),
                            (2.4, 1.5, 4),
                            (4.2, 0.8, 3),
                            (5.2, 1.8, 1),
                            (7.5, 1.2, 3),
                        ];
                        for (st_b, len_b, pitch) in vocal_notes {
                            let nx = cx_start + (st_b * clip_w_px as f32 / 10.0) as usize;
                            let nw = ((len_b * clip_w_px as f32 / 10.0) as usize).max(8);
                            let ny = clip_y + 4 + pitch * 3;
                            if nx + nw <= cx_end && ny + 3 <= clip_y + clip_h {
                                canvas.fill_rounded_rect(nx, ny, nw, 3, 1, track.color_rgb);
                            }
                        }
                    }
                }
            }
        }

        // Glowing Cyan Playhead Line (Spans entire height of Arranger)
        let playhead_x = timeline_x + (self.playhead_beat * ppb) as usize;
        if playhead_x < canvas_right {
            // Glow halo
            canvas.draw_line(playhead_x - 1, top_h, playhead_x - 1, bottom_dock_top, [0x0C, 0x48, 0x64], 1);
            canvas.draw_line(playhead_x + 1, top_h, playhead_x + 1, bottom_dock_top, [0x0C, 0x48, 0x64], 1);
            // Core bright line
            canvas.draw_line(playhead_x, top_h, playhead_x, bottom_dock_top, [0x00, 0xE5, 0xFF], 2);
            // Top Diamond handle
            canvas.fill_rounded_rect(playhead_x - 4, top_h + 2, 9, 14, 2, [0x00, 0xE5, 0xFF]);
        }

        // ==========================================
        // 7. Bottom Dock: Reusable Device Rack (5 Complete Modules)
        // ==========================================
        canvas.fill_rect(canvas_left, bottom_dock_top, canvas_w, height - bottom_dock_top, [0x0E, 0x14, 0x22]);
        canvas.draw_line(canvas_left, bottom_dock_top, canvas_right, bottom_dock_top, [0x1E, 0x28, 0x3E], 1);

        // Device Rack Header Bar
        let r_head_h = 28;
        canvas.draw_text_smooth(canvas_left + 10, bottom_dock_top + 8, "Device Rack", [0xF1, 0xF5, 0xF9]);

        // Power toggle & Device name
        canvas.draw_circle_filled(canvas_left + 88, bottom_dock_top + 13, 4, [0x00, 0xE5, 0xFF]);
        canvas.draw_text_smooth(canvas_left + 98, bottom_dock_top + 8, &self.device_rack_state.device_name, [0x00, 0xE5, 0xFF]);

        // Reusable Pill badge
        canvas.fill_rounded_rect(canvas_right - 80, bottom_dock_top + 5, 54, 18, 3, [0x16, 0x20, 0x32]);
        canvas.draw_text_smooth(canvas_right - 74, bottom_dock_top + 8, "Reusable", [0x94, 0xA3, 0xB8]);
        canvas.draw_text_smooth(canvas_right - 18, bottom_dock_top + 8, "v", [0x94, 0xA3, 0xB8]);

        // Device Body Modules Layout (5 Modules Proportional Distribution)
        let body_top = bottom_dock_top + r_head_h + 4;
        let body_h = height.saturating_sub(body_top + 8);
        let avail_body_w = canvas_w.saturating_sub(40);

        let m1_w = ((avail_body_w as f32 * 0.25).clamp(110.0, 200.0)) as usize;
        let m2_w = ((avail_body_w as f32 * 0.15).clamp(70.0, 120.0)) as usize;
        let m3_w = ((avail_body_w as f32 * 0.24).clamp(110.0, 180.0)) as usize;
        let m4_w = ((avail_body_w as f32 * 0.22).clamp(100.0, 170.0)) as usize;
        let m5_w = avail_body_w.saturating_sub(m1_w + m2_w + m3_w + m4_w + 32).max(60);

        let mut mod_cx = canvas_left + 8;

        // Module 1: Rotary Knobs Box (2 rows x 3 cols)
        let mod1_x = mod_cx;
        canvas.fill_rounded_rect(mod1_x, body_top, m1_w, body_h, 4, [0x12, 0x18, 0x28]);
        canvas.stroke_rounded_rect(mod1_x, body_top, m1_w, body_h, 4, [0x22, 0x30, 0x48], 1);

        let knob_col_w = m1_w / 3;
        let knob_row_h = body_h / 2;
        let knobs = [
            ("Cutoff", "Cutoff", self.device_rack_state.cutoff, [0x00, 0xE5, 0xFF], 0, 0),
            ("Reso", "Reso", self.device_rack_state.resonance, [0xF5, 0x9E, 0x0B], 1, 0),
            ("Decay", "Decay", self.device_rack_state.decay, [0xF5, 0x9E, 0x0B], 2, 0),
            ("Decay", "Amt", self.device_rack_state.env_decay, [0x00, 0xE5, 0xFF], 0, 1),
            ("Amt", "Amt", self.device_rack_state.mod_amt, [0xF5, 0x9E, 0x0B], 1, 1),
            ("Drive", "Drive", self.device_rack_state.drive, [0xF5, 0x9E, 0x0B], 2, 1),
        ];
        let knob_rad = ((knob_col_w.min(knob_row_h) as f32 * 0.28).clamp(8.0, 13.0)) as usize;
        for (top_label, bot_label, val, ring_col, cx_idx, cy_idx) in knobs {
            let k_cx = mod1_x + cx_idx * knob_col_w + knob_col_w / 2;
            let k_cy = body_top + cy_idx * knob_row_h + knob_row_h / 2;
            canvas.draw_rotary_knob_dual(k_cx, k_cy, knob_rad, val, top_label, bot_label, ring_col);
        }
        mod_cx += m1_w + 8;

        // Module 2: Vertical Sliders (Osc Mix, Shape, Vol)
        let mod2_x = mod_cx;
        canvas.fill_rounded_rect(mod2_x, body_top, m2_w, body_h, 4, [0x12, 0x18, 0x28]);
        canvas.stroke_rounded_rect(mod2_x, body_top, m2_w, body_h, 4, [0x22, 0x30, 0x48], 1);

        let faders = [
            ("Osc Mix", self.device_rack_state.osc_mix, 0, "5.0ix"),
            ("Shape", self.device_rack_state.shape, 1, "Sol"),
            ("Vol", self.device_rack_state.volume, 2, "Vol"),
        ];
        let f_col_w = m2_w / 3;
        for (label, val, f_idx, bottom_readout) in faders {
            let fx = mod2_x + f_idx * f_col_w + f_col_w / 2;
            canvas.draw_vertical_fader(fx, body_top + 16, body_top + body_h - 22, val, label, bottom_readout);
        }
        mod_cx += m2_w + 8;

        // Module 3: Real-Time CRT Oscilloscope Display with Multi-Harmonic Synth Glow
        let mod3_x = mod_cx;
        canvas.fill_rounded_rect(mod3_x, body_top, m3_w, body_h, 4, [0x08, 0x0E, 0x18]);
        canvas.stroke_rounded_rect(mod3_x, body_top, m3_w, body_h, 4, [0x22, 0x30, 0x48], 1);

        canvas.draw_text_smooth(mod3_x + 8, body_top + 8, "Oscilloscope", [0x94, 0xA3, 0xB8]);
        canvas.draw_text_smooth(mod3_x + m3_w - 18, body_top + 8, "~", [0x10, 0xB9, 0x81]);

        // CRT Screen Area
        let osc_y = body_top + 22;
        let osc_h = body_h.saturating_sub(28);
        canvas.fill_rect(mod3_x + 6, osc_y, m3_w.saturating_sub(12), osc_h, [0x04, 0x07, 0x0E]);

        // CRT Grid lines
        for gy in 1..4 {
            let y_line = osc_y + (gy * osc_h) / 4;
            canvas.draw_line(mod3_x + 6, y_line, mod3_x + m3_w.saturating_sub(6), y_line, [0x0E, 0x18, 0x26], 1);
        }

        // Phosphor Neon Multi-Harmonic Waveform with Bloom Glow
        let wave_pts = m3_w.saturating_sub(16);
        let mut osc_pts = Vec::with_capacity(wave_pts);
        for i in 0..wave_pts {
            let t = i as f32 / wave_pts.max(1) as f32;
            let sample = (t * 6.0 * std::f32::consts::PI).sin() * 0.45
                + (t * 18.0 * std::f32::consts::PI).sin() * 0.28
                + (t * 36.0 * std::f32::consts::PI).sin() * 0.16
                + (((i * 13) % 7) as f32 * 0.02 - 0.01);
            let px = mod3_x + 8 + i;
            let py = osc_y + osc_h / 2 - (sample * (osc_h as f32 * 0.42)) as usize;
            osc_pts.push((px, py));
        }

        // Layer 1: Ambient Green Glow Bloom
        for w in osc_pts.windows(2) {
            let (p1, p2) = (w[0], w[1]);
            canvas.draw_line(p1.0, p1.1.saturating_sub(1), p2.0, p2.1.saturating_sub(1), [0x06, 0x4E, 0x3B], 3);
            canvas.draw_line(p1.0, p1.1 + 1, p2.0, p2.1 + 1, [0x06, 0x4E, 0x3B], 3);
        }
        // Layer 2: Mid Neon Line
        for w in osc_pts.windows(2) {
            let (p1, p2) = (w[0], w[1]);
            canvas.draw_line(p1.0, p1.1, p2.0, p2.1, [0x10, 0xB9, 0x81], 2);
        }
        // Layer 3: High-Intensity Core
        for w in osc_pts.windows(2) {
            let (p1, p2) = (w[0], w[1]);
            canvas.draw_line(p1.0, p1.1, p2.0, p2.1, [0x86, 0xEF, 0xAC], 1);
        }

        mod_cx += m3_w + 8;

        // Module 4: Filter Frequency Response Curve with Cyan Area Fill
        let mod4_x = mod_cx;
        canvas.fill_rounded_rect(mod4_x, body_top, m4_w, body_h, 4, [0x12, 0x18, 0x28]);
        canvas.stroke_rounded_rect(mod4_x, body_top, m4_w, body_h, 4, [0x22, 0x30, 0x48], 1);
        canvas.draw_text_smooth(mod4_x + 8, body_top + 8, "Filter Frequency", [0x94, 0xA3, 0xB8]);

        let filt_y = body_top + 22;
        let filt_h = body_h.saturating_sub(28);
        canvas.fill_rect(mod4_x + 6, filt_y, m4_w.saturating_sub(12), filt_h, [0x06, 0x0C, 0x18]);

        // Filter response area
        let f_pts = m4_w.saturating_sub(16);
        let mut curve_pts = Vec::with_capacity(f_pts);
        for i in 0..f_pts {
            let t = i as f32 / f_pts.max(1) as f32;
            let gain = if t < 0.52 { 0.72 } else { 0.72 * (-(t - 0.52) * 5.2).exp() };
            let px = mod4_x + 8 + i;
            let top_curve_y = filt_y + filt_h - (gain * filt_h as f32 * 0.85) as usize;
            curve_pts.push((px, top_curve_y));

            // Vertical gradient fill under curve
            let start_y = top_curve_y;
            let end_y = filt_y + filt_h - 2;
            let span = (end_y.saturating_sub(start_y)).max(1);
            for y_step in start_y..=end_y {
                let norm_depth = (y_step - start_y) as f32 / span as f32;
                let c_r = ((1.0 - norm_depth) * 0.0 + norm_depth * 6.0) as u8;
                let c_g = ((1.0 - norm_depth) * 110.0 + norm_depth * 18.0) as u8;
                let c_b = ((1.0 - norm_depth) * 160.0 + norm_depth * 32.0) as u8;
                canvas.set_pixel(px, y_step, [c_r, c_g, c_b]);
            }
        }

        // Draw filter response bright cyan top line
        for w in curve_pts.windows(2) {
            canvas.draw_line(w[0].0, w[0].1, w[1].0, w[1].1, [0x00, 0xE5, 0xFF], 2);
        }

        // Draggable Control Puck Nodes on Filter Curve
        let filter_pucks = [
            (0.18, 0.72),
            (0.38, 0.72),
            (0.52, 0.70),
            (0.68, 0.42),
            (0.88, 0.12),
        ];
        for &(nx, ny) in &filter_pucks {
            let px = mod4_x + 8 + (nx * (f_pts as f32)) as usize;
            let py = filt_y + filt_h - (ny * (filt_h as f32 * 0.85)) as usize;
            canvas.draw_circle_filled(px, py, 4, [0x00, 0xE5, 0xFF]);
            canvas.draw_circle_stroke(px, py, 4, [0xFF, 0xFF, 0xFF], 1);
        }
        mod_cx += m4_w + 8;

        // Module 5: LFO & Modulators
        let mod5_x = mod_cx;
        let final_m5_w = (canvas_right - mod5_x).min(m5_w).max(50);
        if mod5_x + final_m5_w <= canvas_right {
            canvas.fill_rounded_rect(mod5_x, body_top, final_m5_w, body_h, 4, [0x12, 0x18, 0x28]);
            canvas.stroke_rounded_rect(mod5_x, body_top, final_m5_w, body_h, 4, [0x22, 0x30, 0x48], 1);

            canvas.draw_text_smooth(mod5_x + 8, body_top + 8, "LFO", [0x94, 0xA3, 0xB8]);
            canvas.draw_text_smooth(mod5_x + final_m5_w - 18, body_top + 8, "~", [0x00, 0xE5, 0xFF]);

            let lfo_knob_rad = ((final_m5_w as f32 * 0.16).clamp(7.0, 11.0)) as usize;
            let col1_x = mod5_x + final_m5_w / 4 + 2;
            let col2_x = mod5_x + (final_m5_w * 3) / 4 - 2;
            let row1_y = body_top + body_h / 4 + 4;
            let row2_y = body_top + (body_h * 3) / 4 - 2;

            canvas.draw_rotary_knob(col1_x, row1_y, lfo_knob_rad, self.device_rack_state.lfo_speed, "Min", [0x00, 0xE5, 0xFF]);
            canvas.draw_rotary_knob(col2_x, row1_y, lfo_knob_rad, 0.35, "Wait", [0x94, 0xA3, 0xB8]);
            canvas.draw_rotary_knob(col1_x, row2_y, lfo_knob_rad, self.device_rack_state.lfo_depth, "Coom", [0x00, 0xE5, 0xFF]);
            canvas.draw_rotary_knob(col2_x, row2_y, lfo_knob_rad, 0.50, "LFO", [0x94, 0xA3, 0xB8]);
        }

        save_png_file(path, width, height, &canvas.pixels)
    }
}

// ==========================================================
// High-Fidelity 2D Snapshot Canvas Rasterizer
// ==========================================================
struct SnapshotCanvas {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
}

impl SnapshotCanvas {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![0u8; width * height * 4],
        }
    }

    #[inline]
    pub fn set_pixel(&mut self, x: usize, y: usize, color: [u8; 3]) {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) * 4;
            self.pixels[idx] = color[0];
            self.pixels[idx + 1] = color[1];
            self.pixels[idx + 2] = color[2];
            self.pixels[idx + 3] = 0xFF;
        }
    }

    pub fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: [u8; 3]) {
        let x_end = (x + w).min(self.width);
        let y_end = (y + h).min(self.height);
        for py in y..y_end {
            let row_start = (py * self.width + x) * 4;
            let row_end = (py * self.width + x_end) * 4;
            for idx in (row_start..row_end).step_by(4) {
                self.pixels[idx] = color[0];
                self.pixels[idx + 1] = color[1];
                self.pixels[idx + 2] = color[2];
                self.pixels[idx + 3] = 0xFF;
            }
        }
    }

    pub fn draw_line(&mut self, x1: usize, y1: usize, x2: usize, y2: usize, color: [u8; 3], thickness: usize) {
        let dx = (x2 as isize - x1 as isize).abs();
        let dy = (y2 as isize - y1 as isize).abs();
        let sx = if x1 < x2 { 1_isize } else { -1_isize };
        let sy = if y1 < y2 { 1_isize } else { -1_isize };
        let mut err = dx - dy;

        let mut cx = x1 as isize;
        let mut cy = y1 as isize;

        loop {
            for tx in 0..thickness {
                for ty in 0..thickness {
                    let px = (cx + tx as isize) as usize;
                    let py = (cy + ty as isize) as usize;
                    self.set_pixel(px, py, color);
                }
            }

            if cx == x2 as isize && cy == y2 as isize {
                break;
            }
            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                cx += sx;
            }
            if e2 < dx {
                err += dx;
                cy += sy;
            }
        }
    }

    pub fn fill_rounded_rect(&mut self, x: usize, y: usize, w: usize, h: usize, r: usize, color: [u8; 3]) {
        let x_end = (x + w).min(self.width);
        let y_end = (y + h).min(self.height);
        let r_f = r as f32;

        for py in y..y_end {
            for px in x..x_end {
                let mut inside = true;
                // Top-Left corner
                if px < x + r && py < y + r {
                    let dx = (x + r - px) as f32;
                    let dy = (y + r - py) as f32;
                    if dx * dx + dy * dy > r_f * r_f { inside = false; }
                }
                // Top-Right corner
                if px >= x + w - r && py < y + r {
                    let dx = (px - (x + w - r - 1)) as f32;
                    let dy = (y + r - py) as f32;
                    if dx * dx + dy * dy > r_f * r_f { inside = false; }
                }
                // Bottom-Left corner
                if px < x + r && py >= y + h - r {
                    let dx = (x + r - px) as f32;
                    let dy = (py - (y + h - r - 1)) as f32;
                    if dx * dx + dy * dy > r_f * r_f { inside = false; }
                }
                // Bottom-Right corner
                if px >= x + w - r && py >= y + h - r {
                    let dx = (px - (x + w - r - 1)) as f32;
                    let dy = (py - (y + h - r - 1)) as f32;
                    if dx * dx + dy * dy > r_f * r_f { inside = false; }
                }

                if inside {
                    self.set_pixel(px, py, color);
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn stroke_rounded_rect(&mut self, x: usize, y: usize, w: usize, h: usize, r: usize, color: [u8; 3], stroke_w: usize) {
        if w < stroke_w * 2 || h < stroke_w * 2 {
            return;
        }
        // Top edge
        self.fill_rect(x + r, y, w.saturating_sub(r * 2), stroke_w, color);
        // Bottom edge
        self.fill_rect(x + r, y + h - stroke_w, w.saturating_sub(r * 2), stroke_w, color);
        // Left edge
        self.fill_rect(x, y + r, stroke_w, h.saturating_sub(r * 2), color);
        // Right edge
        self.fill_rect(x + w - stroke_w, y + r, stroke_w, h.saturating_sub(r * 2), color);
    }

    pub fn draw_circle_filled(&mut self, cx: usize, cy: usize, radius: usize, color: [u8; 3]) {
        let r_f = radius as f32;
        let x_start = cx.saturating_sub(radius);
        let x_end = (cx + radius).min(self.width);
        let y_start = cy.saturating_sub(radius);
        let y_end = (cy + radius).min(self.height);

        for py in y_start..=y_end {
            for px in x_start..=x_end {
                let dx = (px as isize - cx as isize) as f32;
                let dy = (py as isize - cy as isize) as f32;
                if dx * dx + dy * dy <= r_f * r_f {
                    self.set_pixel(px, py, color);
                }
            }
        }
    }

    pub fn draw_circle_stroke(&mut self, cx: usize, cy: usize, radius: usize, color: [u8; 3], stroke_w: usize) {
        let r_outer = radius as f32;
        let r_inner = radius.saturating_sub(stroke_w) as f32;
        let x_start = cx.saturating_sub(radius);
        let x_end = (cx + radius).min(self.width);
        let y_start = cy.saturating_sub(radius);
        let y_end = (cy + radius).min(self.height);

        for py in y_start..=y_end {
            for px in x_start..=x_end {
                let dx = (px as isize - cx as isize) as f32;
                let dy = (py as isize - cy as isize) as f32;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= r_outer * r_outer && dist_sq >= r_inner * r_inner {
                    self.set_pixel(px, py, color);
                }
            }
        }
    }

    pub fn draw_rotary_knob(&mut self, cx: usize, cy: usize, radius: usize, value: f32, label: &str, ring_col: [u8; 3]) {
        // Outer dark chassis
        self.draw_circle_filled(cx, cy, radius, [0x0A, 0x10, 0x1E]);
        self.draw_circle_stroke(cx, cy, radius, [0x1E, 0x2C, 0x44], 1);

        // Active Arc (270 degrees total span)
        let start_ang = -std::f32::consts::PI * 0.75;
        let end_ang = start_ang + (value.clamp(0.0, 1.0) * std::f32::consts::PI * 1.5);
        let steps = 18;
        for i in 0..steps {
            let a = start_ang + (i as f32 / steps as f32) * (end_ang - start_ang);
            let px = (cx as f32 + a.cos() * (radius as f32 - 2.0)) as usize;
            let py = (cy as f32 + a.sin() * (radius as f32 - 2.0)) as usize;
            self.draw_circle_filled(px, py, 1, ring_col);
        }

        // Pointer notch
        let notch_x = (cx as f32 + end_ang.cos() * (radius as f32 - 4.0)) as usize;
        let notch_y = (cy as f32 + end_ang.sin() * (radius as f32 - 4.0)) as usize;
        self.draw_line(cx, cy, notch_x, notch_y, [0xFF, 0xFF, 0xFF], 1);

        // Label below knob
        let text_w = label.len() * 6;
        let tx = cx.saturating_sub(text_w / 2);
        self.draw_text_smooth(tx, cy + radius + 3, label, [0x94, 0xA3, 0xB8]);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_rotary_knob_dual(&mut self, cx: usize, cy: usize, radius: usize, value: f32, top_label: &str, bot_label: &str, ring_col: [u8; 3]) {
        // Upper label above knob
        let top_w = top_label.len() * 6;
        let top_tx = cx.saturating_sub(top_w / 2);
        self.draw_text_smooth(top_tx, cy.saturating_sub(radius + 10), top_label, [0x94, 0xA3, 0xB8]);

        // Outer dark chassis
        self.draw_circle_filled(cx, cy, radius, [0x0A, 0x10, 0x1E]);
        self.draw_circle_stroke(cx, cy, radius, [0x1E, 0x2C, 0x44], 1);

        // Active Arc (270 degrees total span)
        let start_ang = -std::f32::consts::PI * 0.75;
        let end_ang = start_ang + (value.clamp(0.0, 1.0) * std::f32::consts::PI * 1.5);
        let steps = 20;
        for i in 0..steps {
            let a = start_ang + (i as f32 / steps as f32) * (end_ang - start_ang);
            let px = (cx as f32 + a.cos() * (radius as f32 - 2.0)) as usize;
            let py = (cy as f32 + a.sin() * (radius as f32 - 2.0)) as usize;
            self.draw_circle_filled(px, py, 1, ring_col);
        }

        // Pointer notch
        let notch_x = (cx as f32 + end_ang.cos() * (radius as f32 - 3.5)) as usize;
        let notch_y = (cy as f32 + end_ang.sin() * (radius as f32 - 3.5)) as usize;
        self.draw_line(cx, cy, notch_x, notch_y, [0xFF, 0xFF, 0xFF], 1);

        // Label below knob
        let bot_w = bot_label.len() * 6;
        let bot_tx = cx.saturating_sub(bot_w / 2);
        self.draw_text_smooth(bot_tx, cy + radius + 3, bot_label, [0x94, 0xA3, 0xB8]);
    }

    pub fn draw_vertical_fader(&mut self, cx: usize, top_y: usize, bottom_y: usize, value: f32, label: &str, readout: &str) {
        // Fader track slot
        self.fill_rounded_rect(cx - 2, top_y, 4, bottom_y - top_y, 2, [0x06, 0x0A, 0x14]);

        // Thumb handle (Amber)
        let fader_travel = (bottom_y - top_y) as f32;
        let thumb_y = bottom_y - (value.clamp(0.0, 1.0) * fader_travel) as usize;
        self.fill_rounded_rect(cx - 8, thumb_y.saturating_sub(4), 16, 8, 2, [0xF5, 0x9E, 0x0B]);
        self.stroke_rounded_rect(cx - 8, thumb_y.saturating_sub(4), 16, 8, 2, [0xFF, 0xFF, 0xFF], 1);

        // Label above fader
        let text_w = label.len() * 6;
        let tx = cx.saturating_sub(text_w / 2);
        self.draw_text_smooth(tx, top_y.saturating_sub(12), label, [0x94, 0xA3, 0xB8]);

        // Readout below fader
        let r_w = readout.len() * 6;
        let rx = cx.saturating_sub(r_w / 2);
        self.draw_text_smooth(rx, bottom_y + 4, readout, [0x94, 0xA3, 0xB8]);
    }

    pub fn draw_stylized_k_logo(&mut self, x: usize, y: usize, col: [u8; 3]) {
        // Geometric stylized K logo
        self.fill_rect(x, y, 4, 16, col);
        // Diagonal top
        for i in 0..8 {
            self.fill_rect(x + 4 + i, y + 8 - i, 2, 2, col);
        }
        // Diagonal bottom
        for i in 0..8 {
            self.fill_rect(x + 4 + i, y + 8 + i, 2, 2, col);
        }
    }

    pub fn draw_play_icon(&mut self, x: usize, y: usize, size: usize, col: [u8; 3]) {
        for row in 0..size {
            let max_col = (row * size / (size / 2 + 1)).min(size - row);
            for c in 0..max_col {
                self.set_pixel(x + c, y + row, col);
            }
        }
    }

    pub fn draw_pause_icon(&mut self, x: usize, y: usize, size: usize, col: [u8; 3]) {
        self.fill_rect(x, y, 3, size, col);
        self.fill_rect(x + 6, y, 3, size, col);
    }

    pub fn draw_loop_icon(&mut self, x: usize, y: usize, col: [u8; 3]) {
        // Circular arrows
        self.draw_circle_stroke(x + 6, y + 6, 6, col, 1);
        self.fill_rect(x + 6, y - 1, 4, 3, [0x12, 0x18, 0x26]); // Break gap
        self.fill_rect(x + 7, y - 2, 3, 3, col); // Arrowhead
    }

    pub fn draw_activity_icon(&mut self, icon_type: usize, cx: usize, cy: usize, col: [u8; 3]) {
        match icon_type {
            0 => {
                // 4-box Grid (Browser)
                self.fill_rect(cx - 5, cy - 5, 4, 4, col);
                self.fill_rect(cx + 1, cy - 5, 4, 4, col);
                self.fill_rect(cx - 5, cy + 1, 4, 4, col);
                self.fill_rect(cx + 1, cy + 1, 4, 4, col);
            }
            1 => {
                // Mixer Faders
                self.draw_line(cx - 4, cy - 6, cx - 4, cy + 6, col, 1);
                self.draw_line(cx, cy - 6, cx, cy + 6, col, 1);
                self.draw_line(cx + 4, cy - 6, cx + 4, cy + 6, col, 1);
                self.fill_rect(cx - 6, cy - 2, 4, 2, col);
                self.fill_rect(cx - 2, cy + 2, 4, 2, col);
                self.fill_rect(cx + 2, cy - 4, 4, 2, col);
            }
            2 => {
                // Modular / Knob
                self.draw_circle_stroke(cx, cy, 5, col, 1);
                self.draw_line(cx, cy, cx + 3, cy - 3, col, 1);
            }
            3 => {
                // Audio Waveform
                self.draw_line(cx - 5, cy - 2, cx - 5, cy + 2, col, 1);
                self.draw_line(cx - 2, cy - 5, cx - 2, cy + 5, col, 1);
                self.draw_line(cx + 1, cy - 3, cx + 1, cy + 3, col, 1);
                self.draw_line(cx + 4, cy - 6, cx + 4, cy + 6, col, 1);
            }
            4 => {
                // Play Clip
                self.draw_play_icon(cx - 3, cy - 4, 8, col);
            }
            5 => {
                // Cable routing
                self.draw_circle_stroke(cx - 3, cy - 3, 2, col, 1);
                self.draw_circle_stroke(cx + 3, cy + 3, 2, col, 1);
                self.draw_line(cx - 1, cy - 2, cx + 2, cy + 2, col, 1);
            }
            6 => {
                // Lightbulb
                self.draw_circle_stroke(cx, cy - 2, 4, col, 1);
                self.fill_rect(cx - 2, cy + 3, 4, 2, col);
            }
            7 => {
                // Gear settings
                self.draw_circle_stroke(cx, cy, 4, col, 1);
                self.fill_rect(cx - 5, cy - 1, 10, 2, col);
                self.fill_rect(cx - 1, cy - 5, 2, 10, col);
            }
            _ => {
                // Trash / List
                self.fill_rect(cx - 4, cy - 4, 8, 8, col);
            }
        }
    }

    pub fn draw_category_icon(&mut self, icon_type: usize, cx: usize, cy: usize, col: [u8; 3]) {
        match icon_type {
            0 => {
                // Keyboard / Instruments
                self.fill_rect(cx - 5, cy - 4, 10, 8, col);
                self.fill_rect(cx - 3, cy - 2, 2, 4, [0x0E, 0x13, 0x20]);
                self.fill_rect(cx + 1, cy - 2, 2, 4, [0x0E, 0x13, 0x20]);
            }
            1 => {
                // Audio FX
                self.draw_circle_stroke(cx, cy, 4, col, 1);
                self.draw_line(cx - 4, cy, cx + 4, cy, col, 1);
            }
            2 => {
                // MIDI FX Globe
                self.draw_circle_stroke(cx, cy, 4, col, 1);
                self.draw_line(cx - 4, cy, cx + 4, cy, col, 1);
                self.draw_line(cx, cy - 4, cx, cy + 4, col, 1);
            }
            _ => {
                // Samples Folder
                self.fill_rect(cx - 5, cy - 4, 4, 2, col);
                self.fill_rect(cx - 5, cy - 2, 10, 6, col);
            }
        }
    }

    pub fn draw_folder_icon(&mut self, x: usize, y: usize, col: [u8; 3]) {
        self.fill_rect(x, y, 4, 2, col);
        self.fill_rect(x, y + 2, 9, 6, col);
    }

    pub fn draw_doc_icon(&mut self, x: usize, y: usize, col: [u8; 3]) {
        self.stroke_rounded_rect(x, y, 7, 9, 1, col, 1);
        self.draw_line(x + 2, y + 3, x + 5, y + 3, col, 1);
        self.draw_line(x + 2, y + 5, x + 5, y + 5, col, 1);
    }

    pub fn draw_text_smooth(&mut self, x: usize, y: usize, text: &str, color: [u8; 3]) {
        let mut cur_x = x;
        for c in text.chars() {
            let (glyph, w) = get_proportional_glyph(c);
            for (col_idx, &col_byte) in glyph[..w].iter().enumerate() {
                for row_idx in 0..8 {
                    if (col_byte & (1 << row_idx)) != 0 {
                        self.set_pixel(cur_x + col_idx, y + row_idx, color);
                    }
                }
            }
            cur_x += w + 1;
        }
    }

    #[allow(dead_code)]
    pub fn draw_bezier_cable(&mut self, p0: (usize, usize), p3: (usize, usize), sag: isize, col: [u8; 3], thickness: usize) {
        let (x0, y0) = (p0.0 as f32, p0.1 as f32);
        let (x3, y3) = (p3.0 as f32, p3.1 as f32);
        let x1 = x0 + (x3 - x0) * 0.33;
        let y1 = y0 + sag as f32;
        let x2 = x0 + (x3 - x0) * 0.67;
        let y2 = y3 + sag as f32;

        let steps = 40;
        let mut prev = None;
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let u = 1.0 - t;
            let px = (u * u * u * x0 + 3.0 * u * u * t * x1 + 3.0 * u * t * t * x2 + t * t * t * x3) as usize;
            let py = (u * u * u * y0 + 3.0 * u * u * t * y1 + 3.0 * u * t * t * y2 + t * t * t * y3) as usize;
            if let Some((lx, ly)) = prev {
                self.draw_line(lx, ly, px, py, col, thickness);
            }
            prev = Some((px, py));
        }
    }

    #[allow(dead_code)]
    pub fn draw_jack_socket(&mut self, cx: usize, cy: usize, label: &str, col: [u8; 3]) {
        self.draw_circle_filled(cx, cy, 6, [0x08, 0x0E, 0x18]);
        self.draw_circle_stroke(cx, cy, 6, [0x36, 0x48, 0x64], 1);
        self.draw_circle_stroke(cx, cy, 4, col, 1);
        self.draw_circle_filled(cx, cy, 2, [0x00, 0x00, 0x00]);
        let text_w = label.len() * 6;
        let tx = cx.saturating_sub(text_w / 2);
        self.draw_text_smooth(tx, cy + 8, label, [0x94, 0xA3, 0xB8]);
    }

    #[allow(dead_code)]
    pub fn draw_meter_ladder(&mut self, x: usize, y: usize, w: usize, h: usize, level: f32) {
        let num_segs = 16;
        let seg_h = (h / num_segs).max(2);
        let filled_segs = ((level.clamp(0.0, 1.2) / 1.2) * num_segs as f32).round() as usize;

        for s in 0..num_segs {
            let seg_y = y + h - (s + 1) * seg_h;
            let is_lit = s < filled_segs;
            let color = if s >= 14 {
                if is_lit { [0xEF, 0x44, 0x44] } else { [0x3A, 0x10, 0x14] } // Red clip
            } else if s >= 11 {
                if is_lit { [0xF5, 0x9E, 0x0B] } else { [0x3A, 0x26, 0x08] } // Amber warning
            } else if s >= 7 {
                if is_lit { [0xEA, 0xB3, 0x08] } else { [0x32, 0x28, 0x08] } // Yellow mid
            } else {
                if is_lit { [0x10, 0xB9, 0x81] } else { [0x08, 0x2A, 0x1E] } // Green safe
            };
            self.fill_rect(x, seg_y, w, seg_h.saturating_sub(1), color);
        }
    }
}

/// Proportional bitmap glyph generator supporting uppercase, lowercase, numbers, and symbols.
fn get_proportional_glyph(c: char) -> ([u8; 6], usize) {
    match c {
        'A' => ([0x7E, 0x09, 0x09, 0x09, 0x7E, 0], 5),
        'B' => ([0x7F, 0x49, 0x49, 0x49, 0x36, 0], 5),
        'C' => ([0x3E, 0x41, 0x41, 0x41, 0x22, 0], 5),
        'D' => ([0x7F, 0x41, 0x41, 0x22, 0x1C, 0], 5),
        'E' => ([0x7F, 0x49, 0x49, 0x49, 0x41, 0], 5),
        'F' => ([0x7F, 0x09, 0x09, 0x09, 0x01, 0], 5),
        'G' => ([0x3E, 0x41, 0x49, 0x49, 0x7A, 0], 5),
        'H' => ([0x7F, 0x08, 0x08, 0x08, 0x7F, 0], 5),
        'I' => ([0x00, 0x41, 0x7F, 0x41, 0x00, 0], 3),
        'J' => ([0x20, 0x40, 0x41, 0x3F, 0x01, 0], 5),
        'K' => ([0x7F, 0x08, 0x14, 0x22, 0x41, 0], 5),
        'L' => ([0x7F, 0x40, 0x40, 0x40, 0x40, 0], 5),
        'M' => ([0x7F, 0x02, 0x0C, 0x02, 0x7F, 0], 5),
        'N' => ([0x7F, 0x04, 0x08, 0x10, 0x7F, 0], 5),
        'O' => ([0x3E, 0x41, 0x41, 0x41, 0x3E, 0], 5),
        'P' => ([0x7F, 0x09, 0x09, 0x09, 0x06, 0], 5),
        'Q' => ([0x3E, 0x41, 0x51, 0x21, 0x5E, 0], 5),
        'R' => ([0x7F, 0x09, 0x19, 0x29, 0x46, 0], 5),
        'S' => ([0x46, 0x49, 0x49, 0x49, 0x31, 0], 5),
        'T' => ([0x01, 0x01, 0x7F, 0x01, 0x01, 0], 5),
        'U' => ([0x3F, 0x40, 0x40, 0x40, 0x3F, 0], 5),
        'V' => ([0x1F, 0x20, 0x40, 0x20, 0x1F, 0], 5),
        'W' => ([0x7F, 0x20, 0x18, 0x20, 0x7F, 0], 5),
        'X' => ([0x63, 0x14, 0x08, 0x14, 0x63, 0], 5),
        'Y' => ([0x07, 0x08, 0x70, 0x08, 0x07, 0], 5),
        'Z' => ([0x61, 0x51, 0x49, 0x45, 0x43, 0], 5),

        'a' => ([0x20, 0x54, 0x54, 0x78, 0x40, 0], 5),
        'b' => ([0x7F, 0x48, 0x44, 0x44, 0x38, 0], 5),
        'c' => ([0x38, 0x44, 0x44, 0x44, 0x20, 0], 5),
        'd' => ([0x38, 0x44, 0x44, 0x48, 0x7F, 0], 5),
        'e' => ([0x38, 0x54, 0x54, 0x54, 0x18, 0], 5),
        'f' => ([0x08, 0x7E, 0x09, 0x01, 0x02, 0], 4),
        'g' => ([0x08, 0x14, 0x54, 0x54, 0x3C, 0], 5),
        'h' => ([0x7F, 0x08, 0x04, 0x04, 0x78, 0], 5),
        'i' => ([0x00, 0x44, 0x7D, 0x40, 0x00, 0], 3),
        'j' => ([0x20, 0x40, 0x44, 0x3D, 0x00, 0], 4),
        'k' => ([0x7F, 0x10, 0x28, 0x44, 0x00, 0], 4),
        'l' => ([0x00, 0x41, 0x7F, 0x40, 0x00, 0], 3),
        'm' => ([0x7C, 0x04, 0x18, 0x04, 0x78, 0], 5),
        'n' => ([0x7C, 0x08, 0x04, 0x04, 0x78, 0], 5),
        'o' => ([0x38, 0x44, 0x44, 0x44, 0x38, 0], 5),
        'p' => ([0x7C, 0x14, 0x14, 0x14, 0x08, 0], 5),
        'q' => ([0x08, 0x14, 0x14, 0x18, 0x7C, 0], 5),
        'r' => ([0x7C, 0x08, 0x04, 0x04, 0x08, 0], 5),
        's' => ([0x48, 0x54, 0x54, 0x54, 0x20, 0], 5),
        't' => ([0x04, 0x3F, 0x44, 0x40, 0x20, 0], 4),
        'u' => ([0x3C, 0x40, 0x40, 0x20, 0x7C, 0], 5),
        'v' => ([0x1C, 0x20, 0x40, 0x20, 0x1C, 0], 5),
        'w' => ([0x3C, 0x40, 0x30, 0x40, 0x3C, 0], 5),
        'x' => ([0x44, 0x28, 0x10, 0x28, 0x44, 0], 5),
        'y' => ([0x0C, 0x50, 0x50, 0x50, 0x3C, 0], 5),
        'z' => ([0x44, 0x64, 0x54, 0x4C, 0x44, 0], 5),

        '0' => ([0x3E, 0x51, 0x49, 0x45, 0x3E, 0], 5),
        '1' => ([0x00, 0x42, 0x7F, 0x40, 0x00, 0], 3),
        '2' => ([0x42, 0x61, 0x51, 0x49, 0x46, 0], 5),
        '3' => ([0x21, 0x41, 0x45, 0x4B, 0x31, 0], 5),
        '4' => ([0x18, 0x14, 0x12, 0x7F, 0x10, 0], 5),
        '5' => ([0x27, 0x45, 0x45, 0x45, 0x39, 0], 5),
        '6' => ([0x3C, 0x4A, 0x49, 0x49, 0x30, 0], 5),
        '7' => ([0x01, 0x71, 0x09, 0x05, 0x03, 0], 5),
        '8' => ([0x36, 0x49, 0x49, 0x49, 0x36, 0], 5),
        '9' => ([0x06, 0x49, 0x49, 0x29, 0x1E, 0], 5),

        '.' => ([0x60, 0x60, 0, 0, 0, 0], 2),
        ':' => ([0x36, 0x36, 0, 0, 0, 0], 2),
        '+' => ([0x08, 0x08, 0x3E, 0x08, 0x08, 0], 5),
        '-' => ([0x08, 0x08, 0x08, 0x08, 0, 0], 4),
        '%' => ([0x23, 0x13, 0x08, 0x64, 0x62, 0], 5),
        '/' => ([0x20, 0x10, 0x08, 0x04, 0x02, 0], 5),
        '(' => ([0x1C, 0x22, 0x41, 0, 0, 0], 3),
        ')' => ([0x41, 0x22, 0x1C, 0, 0, 0], 3),
        '<' => ([0x08, 0x14, 0x22, 0x41, 0, 0], 4),
        '>' => ([0x41, 0x22, 0x14, 0x08, 0, 0], 4),
        '[' => ([0x7F, 0x41, 0x41, 0, 0, 0], 3),
        ']' => ([0x41, 0x41, 0x7F, 0, 0, 0], 3),
        '|' => ([0x7F, 0, 0, 0, 0, 0], 1),
        '~' => ([0x08, 0x04, 0x08, 0x10, 0x08, 0], 5),
        '^' => ([0x04, 0x02, 0x01, 0x02, 0x04, 0], 5),
        '=' => ([0x14, 0x14, 0x14, 0x14, 0, 0], 4),
        '*' => ([0x14, 0x08, 0x3E, 0x08, 0x14, 0], 5),
        '#' => ([0x14, 0x7F, 0x14, 0x7F, 0x14, 0], 5),
        '!' => ([0x5F, 0, 0, 0, 0, 0], 1),
        '?' => ([0x02, 0x01, 0x51, 0x09, 0x06, 0], 5),
        ' ' => ([0, 0, 0, 0, 0, 0], 3),
        _ => ([0x3E, 0x22, 0x22, 0x3E, 0, 0], 4),
    }
}

fn save_png_file(path: &str, width: usize, height: usize, rgba_pixels: &[u8]) -> Result<(), String> {
    let mut out = Vec::with_capacity(width * height * 4 + 1024);
    out.extend_from_slice(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);

    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&(width as u32).to_be_bytes());
    ihdr.extend_from_slice(&(height as u32).to_be_bytes());
    ihdr.push(8);
    ihdr.push(6);
    ihdr.push(0);
    ihdr.push(0);
    ihdr.push(0);
    write_png_chunk_to(&mut out, b"IHDR", &ihdr);

    let stride = width * 4;
    let mut raw_data = Vec::with_capacity((stride + 1) * height);
    for y in 0..height {
        raw_data.push(0x00);
        let row_start = y * stride;
        let row_end = row_start + stride;
        raw_data.extend_from_slice(&rgba_pixels[row_start..row_end]);
    }

    let mut zlib_data = Vec::with_capacity(raw_data.len() + 128);
    zlib_data.push(0x78);
    zlib_data.push(0x01);

    let mut offset = 0;
    while offset < raw_data.len() {
        let chunk_len = (raw_data.len() - offset).min(65535);
        let is_last = (offset + chunk_len) >= raw_data.len();
        let bfinal_btype = if is_last { 0x01 } else { 0x00 };
        zlib_data.push(bfinal_btype);

        let len_u16 = chunk_len as u16;
        let nlen_u16 = !len_u16;
        zlib_data.extend_from_slice(&len_u16.to_le_bytes());
        zlib_data.extend_from_slice(&nlen_u16.to_le_bytes());
        zlib_data.extend_from_slice(&raw_data[offset..offset + chunk_len]);
        offset += chunk_len;
    }

    let mut s1: u32 = 1;
    let mut s2: u32 = 0;
    for &b in &raw_data {
        s1 = (s1 + b as u32) % 65521;
        s2 = (s2 + s1) % 65521;
    }
    let adler = (s2 << 16) | s1;
    zlib_data.extend_from_slice(&adler.to_be_bytes());

    write_png_chunk_to(&mut out, b"IDAT", &zlib_data);
    write_png_chunk_to(&mut out, b"IEND", &[]);

    if let Some(parent) = std::path::Path::new(path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let res = std::fs::write(path, &out).map_err(|e| e.to_string());
    
    // Also mirror to workspace root scratch/renders if run from crate dir
    let ws_path = format!("../../{}", path);
    if let Some(parent) = std::path::Path::new(&ws_path).parent() {
        if parent.exists() || std::path::Path::new("../../crates").exists() {
            let _ = std::fs::create_dir_all(parent);
            let _ = std::fs::write(&ws_path, &out);
        }
    }
    // And mirror to crate scratch/renders if run from workspace root
    let crate_path = format!("crates/summoner_gui/{}", path);
    if let Some(parent) = std::path::Path::new(&crate_path).parent() {
        if parent.exists() || std::path::Path::new("crates/summoner_gui").exists() {
            let _ = std::fs::create_dir_all(parent);
            let _ = std::fs::write(&crate_path, &out);
        }
    }

    res
}

fn write_png_chunk_to(out: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let type_start = out.len();
    out.extend_from_slice(chunk_type);
    out.extend_from_slice(data);
    let crc = crc32_compute_to(&out[type_start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

fn crc32_compute_to(buf: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in buf {
        crc ^= b as u32;
        for _ in 0..8 {
            let mask = if (crc & 1) != 0 { 0xEDB8_8320 } else { 0 };
            crc = (crc >> 1) ^ mask;
        }
    }
    !crc
}
