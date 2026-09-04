// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Master Award-Winning DAW GUI View tying together the Triad Architecture.
//! Features fixed operational zones, tactile controls, and high-fidelity deterministic PNG rendering.

use crate::views::modern_asset_browser::{show_modern_asset_browser, ModernAssetBrowserState};
use crate::views::modern_device_rack::{show_modern_device_rack, ModernDeviceRackState};
use crate::views::modern_inspector::{show_modern_inspector, ModernInspectorState};
use crate::views::modern_top_bar::{show_modern_top_bar, ModernTopBarState};

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
    pub pitch_idx: usize, // 0..13 (C5 down to C4)
    pub start_beat: f32,
    pub length_beats: f32,
    pub velocity: f32,
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
    pub pending_cord_source: Option<(String, String)>,
    pub last_applied_preset: String,
    pub last_applied_macros: [f32; 4],
    pub last_synced_bpm: f64,
    pub last_synced_is_playing: bool,
    pub active_scene_idx: Option<usize>,
    pub panic_triggered: bool,
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
            pending_cord_source: None,
            last_applied_preset: "Init Synth 1".to_string(),
            last_applied_macros: [0.65, 0.40, 0.55, 0.50],
            last_synced_bpm: 120.0,
            last_synced_is_playing: false,
            active_scene_idx: None,
            panic_triggered: false,
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
        };
        view.reset_modular_nodes();
        view
    }

    #[cfg(feature = "gui")]
    pub fn show(&mut self, ui: &mut egui::Ui) {
        // 1. Synchronize Novice Presets to Active Device Rack & Inspector
        if self.top_bar_state.selected_preset != self.last_applied_preset {
            self.last_applied_preset = self.top_bar_state.selected_preset.clone();
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
                "Cathedral Pipe Organ" => "PipeOrgan",
                "Cosmic Shockwave Reverb" => "IsmShockwaveReverb",
                "Neuro HRV Bio-Sync" => "HrvTempoSyncEngine",
                "Analog Tape Stop" => "TapeStop",
                "Orchestral Timpani Drum" => "PercussionMembrane",
                "Concert Rosewood Marimba" => "StruckIdiophoneResonator",
                "Bourbonnais Vielle Gurdy" => "HurdyGurdySoundboxBody",
                "Silk String Japanese Koto" => "SitarSoundboxBody",
                _ => "AetherSynth",
            };
            if let Some(desc) = registry.get(target_node) {
                self.device_rack_state.selected_node_kind = Some(desc.kind_id.clone());
                self.device_rack_state.device_name = desc.display_name.clone();
                self.inspector_state.selected_node_kind = Some(desc.kind_id.clone());
                self.inspector_state.target_name = desc.display_name.clone();
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

        // 4. Synchronize selected DSP module across Modular Canvas, Rack & Inspector
        if self.device_rack_state.selected_node_kind != self.inspector_state.selected_node_kind {
            if let Some(ref r_kind) = self.device_rack_state.selected_node_kind {
                self.inspector_state.selected_node_kind = Some(r_kind.clone());
                self.inspector_state.target_name = self.device_rack_state.device_name.clone();
            }
        }
        for (k, v) in &self.device_rack_state.node_param_values {
            self.inspector_state.node_param_values.insert(k.clone(), *v);
        }
        for (k, v) in &self.inspector_state.node_param_values {
            self.device_rack_state.node_param_values.insert(k.clone(), *v);
        }

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
                    show_modern_device_rack(ui, &mut self.device_rack_state, None);
                });

                // Zone 4: Right Collapsible Inspector
                show_modern_inspector(ui, &mut self.inspector_state);
            });
        });
    }

    #[cfg(feature = "gui")]
    fn show_arranger_canvas(&mut self, ui: &mut egui::Ui) {
        let canvas_height = (self.tracks.len() as f32 * 36.0 + 36.0).max(280.0);
        egui::Frame::none()
            .fill(Color32::from_rgb(10, 14, 24))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(28, 40, 60)))
            .rounding(Rounding::same(6.0))
            .show(ui, |ui| {
                ui.set_height(canvas_height);
                let (resp, painter) = ui.allocate_painter(Vec2::new(ui.available_width(), canvas_height), egui::Sense::click_and_drag());
                let rect = resp.rect;

                let header_w = 140.0;
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

                // Interaction handling (Scrubbing playhead, selecting track, adjusting track gain)
                let pointer_pos = resp.interact_pointer_pos().or_else(|| ui.input(|i| i.pointer.latest_pos()));
                let is_interacting = resp.clicked() || resp.dragged() || ui.input(|i| i.pointer.primary_down() || i.pointer.primary_clicked());
                let is_click = resp.clicked() || ui.input(|i| i.pointer.primary_clicked() || (i.pointer.primary_down() && !resp.dragged()));

                if let Some(pos) = pointer_pos {
                    if is_interacting {
                        if pos.y <= rect.top() + ruler_h {
                            let beat = ((pos.x - (rect.left() + header_w)) / ppb).max(0.0);
                            self.playhead_beat = beat;
                        } else {
                            let mut selected_idx = None;
                            let row_h = 32.0;
                            for (idx, track) in self.tracks.iter_mut().enumerate() {
                                let row_top = rect.top() + ruler_h + (idx as f32 * (row_h + 2.0));
                                let head_rect = Rect::from_min_size(egui::pos2(rect.left(), row_top), Vec2::new(header_w, row_h));
                                let lane_rect = Rect::from_min_size(egui::pos2(rect.left() + header_w, row_top), Vec2::new(track_area_w, row_h));
                                let pill_rect = Rect::from_min_size(egui::pos2(head_rect.left() + 72.0, head_rect.center().y - 4.0), Vec2::new(45.0, 8.0));

                                if pill_rect.expand(4.0).contains(pos) {
                                    let new_gain = ((pos.x - pill_rect.left()) / pill_rect.width() * 1.5).clamp(0.0, 1.5);
                                    track.gain = new_gain;
                                    selected_idx = Some(idx);
                                } else if (head_rect.contains(pos) || lane_rect.contains(pos)) && is_click {
                                    selected_idx = Some(idx);
                                }
                            }
                            if let Some(s_idx) = selected_idx {
                                self.selected_track_idx = s_idx;
                                if let Some(tr) = self.tracks.get(s_idx) {
                                    self.top_bar_state.master_gain = tr.gain;
                                    self.inspector_state.target_name = tr.name.clone();
                                    self.inspector_state.gain_db = (tr.gain - 1.0) * 12.0;
                                    self.inspector_state.pan_val = tr.pan;
                                    self.inspector_state.is_muted = tr.is_muted;
                                    self.inspector_state.is_soloed = tr.is_soloed;
                                    self.inspector_state.is_armed = tr.is_armed;
                                    self.device_rack_state.device_name = tr.name.clone();
                                }
                            }
                        }
                    }
                }

                // 2. Track Lanes
                let row_h = 32.0;
                for (idx, track) in self.tracks.iter().enumerate() {
                    let row_top = rect.top() + ruler_h + (idx as f32 * (row_h + 2.0));
                    let is_sel = idx == self.selected_track_idx;

                    // Track Header
                    let head_rect = Rect::from_min_size(egui::pos2(rect.left(), row_top), Vec2::new(header_w, row_h));
                    let head_bg = if is_sel { Color32::from_rgb(20, 28, 44) } else { Color32::from_rgb(14, 18, 28) };
                    painter.rect_filled(head_rect, 2.0, head_bg);
                    painter.rect_stroke(head_rect, 2.0, Stroke::new(1.0_f32, Color32::from_rgb(28, 38, 56)));

                    // Track Index & Name
                    let col = Color32::from_rgb(track.color_rgb[0], track.color_rgb[1], track.color_rgb[2]);
                    painter.text(egui::pos2(head_rect.left() + 8.0, head_rect.center().y), egui::Align2::LEFT_CENTER, format!("{}", idx + 1), FontId::proportional(10.0), Color32::from_rgb(100, 116, 139));
                    painter.text(egui::pos2(head_rect.left() + 24.0, head_rect.center().y), egui::Align2::LEFT_CENTER, &track.name, FontId::proportional(11.0), Color32::from_rgb(241, 245, 249));

                    // Colored Volume Slider Pill
                    let pill_rect = Rect::from_min_size(egui::pos2(head_rect.left() + 72.0, head_rect.center().y - 4.0), Vec2::new(45.0, 8.0));
                    painter.rect_filled(pill_rect, 4.0, Color32::from_rgb(8, 12, 20));
                    let fill_w = pill_rect.width() * (track.gain / 1.5).clamp(0.0, 1.0);
                    painter.rect_filled(Rect::from_min_size(pill_rect.min, Vec2::new(fill_w, 8.0)), 4.0, col);

                    // Track Timeline Lane
                    let lane_rect = Rect::from_min_size(egui::pos2(rect.left() + header_w, row_top), Vec2::new(track_area_w, row_h));
                    let lane_bg = if is_sel { Color32::from_rgb(16, 24, 38) } else { Color32::from_rgb(10, 14, 22) };
                    painter.rect_filled(lane_rect, 2.0, lane_bg);

                    // Bar divider lines across lane
                    for bar in 1..=17 {
                        let bar_x = rect.left() + header_w + ((bar - 1) as f32 * 4.0 * ppb);
                        painter.line_segment([egui::pos2(bar_x, lane_rect.top()), egui::pos2(bar_x, lane_rect.bottom())], Stroke::new(0.5_f32, Color32::from_rgb(24, 32, 48)));
                    }

                    // Clip Block
                    let clip_x = rect.left() + header_w + (track.clip_start_beat * ppb);
                    let clip_w = (track.clip_length_beats * ppb).max(20.0);
                    let clip_rect = Rect::from_min_size(egui::pos2(clip_x, lane_rect.top() + 2.0), Vec2::new(clip_w, lane_rect.height() - 4.0));

                    // Clip fill with soft color glow
                    painter.rect_filled(clip_rect, 4.0, Color32::from_rgba_unmultiplied(track.color_rgb[0], track.color_rgb[1], track.color_rgb[2], 40));
                    painter.rect_stroke(clip_rect, 4.0, Stroke::new(1.2_f32, col));

                    // Clip content (waveform spikes or MIDI note blocks)
                    if track.is_audio {
                        // Waveform spikes
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
                        // MIDI note blocks
                        let num_steps = 12;
                        for step in 0..num_steps {
                            let st = step as f32 / num_steps as f32;
                            let nx = clip_rect.left() + st * (clip_rect.width() - 14.0) + 4.0;
                            let note_y = clip_rect.top() + 6.0 + (step % 4) as f32 * 4.0;
                            let note_rect = Rect::from_min_size(egui::pos2(nx, note_y), Vec2::new(10.0, 3.0));
                            painter.rect_filled(note_rect, 1.0, col);
                        }
                    }
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
        let canvas_height = (self.tracks.len() as f32 * 36.0 + 36.0).max(280.0);
        egui::Frame::none()
            .fill(Color32::from_rgb(10, 14, 24))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(28, 40, 60)))
            .rounding(Rounding::same(6.0))
            .show(ui, |ui| {
                ui.set_height(canvas_height);
                let (resp, painter) = ui.allocate_painter(Vec2::new(ui.available_width(), canvas_height), egui::Sense::click_and_drag());
                let rect = resp.rect;

                let key_w = 64.0;
                let grid_w = (rect.width() - key_w).max(200.0);
                let ppb = grid_w / 18.0;

                // 1. Timeline Header Ruler (Measures 1..17)
                let ruler_h = 24.0;
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

                // Keyboard & Pitch Row definitions (12 semitones C5 down to C4)
                let pitch_names = ["C5", "B4", "A#4", "A4", "G#4", "G4", "F#4", "F4", "E4", "D#4", "D4", "C#4", "C4"];
                let num_pitches = pitch_names.len();
                let vel_lane_h = 32.0;
                let roll_area_h = canvas_height - ruler_h - vel_lane_h;
                let pitch_row_h = roll_area_h / num_pitches as f32;

                // Delete selected note shortcut
                if is_delete_pressed {
                    if let Some(sel_id) = self.selected_note_id {
                        self.piano_roll_notes.retain(|n| n.id != sel_id);
                        self.selected_note_id = None;
                    }
                }

                // Interactive Pointer Handling (Keyboard, Velocity Lane, Grid Notes)
                if let Some(pos) = pointer_pos {
                    if pos.x <= rect.left() + key_w && pos.y > rect.top() + ruler_h && pos.y < rect.bottom() - vel_lane_h {
                        // 2A. Interactive Piano Keyboard Clicking / Auditioning
                        let p_idx = (((pos.y - (rect.top() + ruler_h)) / pitch_row_h) as usize).min(num_pitches - 1);
                        if is_down || is_click {
                            self.auditioned_pitch_idx = Some(p_idx);
                            self.inspector_state.target_name = format!("Pitch {}", pitch_names[p_idx]);
                            self.inspector_state.scale_ratio_num = (num_pitches - p_idx) as i32;
                            self.inspector_state.root_ratio = 440.0 * 2.0f32.powf(((72 - p_idx as i32) - 69) as f32 / 12.0);
                            self.inspector_state.octave_offset = if p_idx < 1 { 5 } else { 4 };
                        }
                    } else if pos.y >= rect.bottom() - vel_lane_h && pos.x > rect.left() + key_w {
                        // 2B. Interactive Velocity Lane Dragging
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
                            if min_dist < (ppb * 1.5).max(14.0) {
                                if let Some(idx) = closest_idx {
                                    let new_vel = ((rect.bottom() - 2.0 - pos.y) / (vel_lane_h * 0.85)).clamp(0.05, 1.0);
                                    self.piano_roll_notes[idx].velocity = new_vel;
                                    self.selected_note_id = Some(self.piano_roll_notes[idx].id);
                                }
                            } else if let Some(sel_id) = self.selected_note_id {
                                if let Some(note) = self.piano_roll_notes.iter_mut().find(|n| n.id == sel_id) {
                                    let new_vel = ((rect.bottom() - 2.0 - pos.y) / (vel_lane_h * 0.85)).clamp(0.05, 1.0);
                                    note.velocity = new_vel;
                                }
                            }
                        }
                    } else if pos.x > rect.left() + key_w && pos.y > rect.top() + ruler_h && pos.y < rect.bottom() - vel_lane_h {
                        // 2C. Grid Note Selection, Creation & Deletion
                        if is_click || is_secondary_click {
                            let mut hit_idx = None;
                            for (idx, note) in self.piano_roll_notes.iter().enumerate() {
                                let note_y = rect.top() + ruler_h + note.pitch_idx as f32 * pitch_row_h + 1.5;
                                let note_h = pitch_row_h - 3.0;
                                let note_x = rect.left() + key_w + note.start_beat * ppb;
                                let note_w = (note.length_beats * ppb).max(8.0);
                                let n_rect = Rect::from_min_size(egui::pos2(note_x, note_y), Vec2::new(note_w, note_h));
                                if n_rect.contains(pos) {
                                    hit_idx = Some(idx);
                                    break;
                                }
                            }
                            if let Some(idx) = hit_idx {
                                if is_secondary_click {
                                    self.piano_roll_notes.remove(idx);
                                    self.selected_note_id = None;
                                } else {
                                    let note = &self.piano_roll_notes[idx];
                                    self.selected_note_id = Some(note.id);
                                    self.inspector_state.target_name = format!("Note {} (Beat {:.1})", pitch_names[note.pitch_idx], note.start_beat);
                                    self.inspector_state.scale_ratio_num = (num_pitches - note.pitch_idx) as i32;
                                    self.inspector_state.octave_offset = if note.pitch_idx < 1 { 5 } else { 4 };
                                }
                            } else if is_click {
                                // Clicked empty cell: create new note
                                let p_idx = (((pos.y - (rect.top() + ruler_h)) / pitch_row_h) as usize).min(num_pitches - 1);
                                let raw_b = ((pos.x - (rect.left() + key_w)) / ppb).max(0.0);
                                let start_b = (raw_b * 2.0).round() / 2.0; // snapped to 0.5 beat
                                let new_id = self.next_note_id;
                                self.next_note_id += 1;
                                self.piano_roll_notes.push(PianoRollNote {
                                    id: new_id,
                                    pitch_idx: p_idx,
                                    start_beat: start_b,
                                    length_beats: 1.0,
                                    velocity: 0.85,
                                });
                                self.selected_note_id = Some(new_id);
                                self.inspector_state.target_name = format!("Note {} (Beat {:.1})", pitch_names[p_idx], start_b);
                                self.inspector_state.scale_ratio_num = (num_pitches - p_idx) as i32;
                                self.inspector_state.octave_offset = if p_idx < 1 { 5 } else { 4 };
                            }
                        }
                    }
                }
                if !is_down && !is_click {
                    self.auditioned_pitch_idx = None;
                }

                // 2. Render Pitch Rows & Piano Keys
                for (p_idx, p_name) in pitch_names.iter().enumerate() {
                    let py = rect.top() + ruler_h + p_idx as f32 * pitch_row_h;
                    let is_accidental = p_name.contains('#');
                    let is_tonic = *p_name == "A4" || *p_name == "C4" || *p_name == "C5";
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
                    painter.text(egui::pos2(key_rect.left() + 8.0, key_rect.center().y), egui::Align2::LEFT_CENTER, *p_name, FontId::proportional(9.0), key_text_col);

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

                // 3. Velocity Lane Background & Separator
                let vel_sep_y = rect.bottom() - vel_lane_h;
                painter.line_segment([egui::pos2(rect.left(), vel_sep_y), egui::pos2(rect.right(), vel_sep_y)], Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 74)));
                painter.rect_filled(Rect::from_min_size(egui::pos2(rect.left(), vel_sep_y), Vec2::new(rect.width(), vel_lane_h)), 0.0, Color32::from_rgb(12, 16, 26));
                painter.text(egui::pos2(rect.left() + 8.0, vel_sep_y + 4.0), egui::Align2::LEFT_TOP, "VELOCITY", FontId::proportional(8.0), Color32::from_rgb(100, 116, 139));

                // 4. MIDI Note Blocks & Velocity Lane Sticks
                for note in &self.piano_roll_notes {
                    let is_sel = self.selected_note_id == Some(note.id);
                    let note_y = rect.top() + ruler_h + note.pitch_idx as f32 * pitch_row_h + 1.5;
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

                    // Note Pitch Label inside block
                    if note_w >= 22.0 && note.pitch_idx < num_pitches {
                        painter.text(
                            egui::pos2(note_rect.left() + 4.0, note_rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            pitch_names[note.pitch_idx],
                            FontId::proportional(8.5),
                            if is_sel { Color32::WHITE } else { Color32::from_rgb(220, 240, 255) },
                        );
                    }

                    // Velocity lane stick
                    let v_height = vel_lane_h * note.velocity * 0.85;
                    let stick_x = note_x + 2.0;
                    let cap_y = rect.bottom() - 2.0 - v_height;
                    let stick_col = if is_sel { Color32::WHITE } else { Color32::from_rgb(56, 189, 248) };
                    painter.line_segment([egui::pos2(stick_x, rect.bottom() - 2.0), egui::pos2(stick_x, cap_y)], Stroke::new(if is_sel { 2.0_f32 } else { 1.5_f32 }, stick_col));
                    painter.circle_filled(egui::pos2(stick_x, cap_y), if is_sel { 3.5 } else { 2.5 }, stick_col);
                    if is_sel {
                        painter.text(
                            egui::pos2(stick_x + 6.0, cap_y),
                            egui::Align2::LEFT_CENTER,
                            format!("{:.0}%", note.velocity * 100.0),
                            FontId::proportional(8.0),
                            Color32::from_rgb(56, 189, 248),
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

        let instance = ModularNodeInstance {
            id: node_id.clone(),
            kind_id: desc.kind_id.clone(),
            display_name: desc.display_name.clone(),
            category: desc.category,
            pos: (x, y),
            size: (145.0, 105.0),
            ports,
        };
        self.selected_modular_node_id = Some(node_id);
        self.device_rack_state.selected_node_kind = Some(desc.kind_id.clone());
        self.device_rack_state.device_name = desc.display_name.clone();
        self.inspector_state.selected_node_kind = Some(desc.kind_id.clone());
        self.inspector_state.target_name = desc.display_name.clone();
        self.modular_nodes.push(instance);
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
                    ui.add_space(8.0);

                    // Mode Toggle: Patch Cords vs Matrix Grid
                    let cords_active = self.modular_mode == ModularCanvasMode::PatchCords;
                    if ui.selectable_label(cords_active, "∿ Patch Cords").clicked() {
                        self.modular_mode = ModularCanvasMode::PatchCords;
                    }
                    let matrix_active = self.modular_mode == ModularCanvasMode::RoutingMatrix;
                    if ui.selectable_label(matrix_active, "▦ Routing Matrix").clicked() {
                        self.modular_mode = ModularCanvasMode::RoutingMatrix;
                    }

                    ui.separator();

                    // "➕ Add Module" Dropdown
                    let registry = crate::dsp_node_ui::DspNodeRegistry::new();
                    egui::ComboBox::from_id_source("modular_canvas_add_module_combo")
                        .selected_text(RichText::new("➕ Add DSP Module...").font(FontId::proportional(10.0)).color(Color32::from_rgb(56, 189, 248)))
                        .show_ui(ui, |ui| {
                            for desc in registry.list_all() {
                                let label = format!("{} {} ({})", desc.category.icon(), desc.display_name, desc.category.name());
                                if ui.selectable_label(false, label).clicked() {
                                    self.add_modular_node_from_descriptor(desc);
                                }
                            }
                        });

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

                ui.separator();

                match self.modular_mode {
                    ModularCanvasMode::RoutingMatrix => {
                        self.patch_matrix.ui(ui);
                    }
                    ModularCanvasMode::PatchCords => {
                        self.render_patch_cords_canvas(ui, canvas_height - 38.0);
                    }
                }
            });
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

        let mut node_selected = None;
        let mut clicked_socket = None;

        for node in &self.modular_nodes {
            let n_rect = Rect::from_min_size(
                egui::pos2(rect.left() + node.pos.0, rect.top() + node.pos.1),
                Vec2::new(node.size.0, node.size.1),
            );

            // Check socket clicks first
            for port in &node.ports {
                let s_pos = egui::pos2(n_rect.left() + port.rel_pos.0, n_rect.top() + port.rel_pos.1);
                if let Some(pos) = pointer_pos {
                    if pos.distance(s_pos) <= 8.0 && clicked {
                        clicked_socket = Some((node.id.clone(), port.id.clone(), port.kind));
                    }
                }
            }

            if clicked_socket.is_none() && clicked {
                if let Some(pos) = pointer_pos {
                    if n_rect.contains(pos) {
                        node_selected = Some((node.id.clone(), node.kind_id.clone(), node.display_name.clone()));
                    }
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
                        } else {
                            self.patch_cords.push(ModularPatchCord {
                                from_node_id: src_node,
                                from_port_id: src_port,
                                to_node_id: node_id,
                                to_port_id: port_id,
                                is_audio,
                                intensity: 1.0,
                            });
                        }
                    }
                } else {
                    // Disconnect existing cable to this input socket
                    self.patch_cords.retain(|c| !(c.to_node_id == node_id && c.to_port_id == port_id));
                }
            }
        } else if clicked && pointer_pos.is_some() && node_selected.is_none() {
            // Clicked on empty space: cancel pending cord
            self.pending_cord_source = None;
        }

        if let Some((node_id, kind_id, display_name)) = node_selected {
            self.selected_modular_node_id = Some(node_id);
            self.device_rack_state.selected_node_kind = Some(kind_id.clone());
            self.device_rack_state.device_name = display_name.clone();
            self.inspector_state.selected_node_kind = Some(kind_id);
            self.inspector_state.target_name = display_name;
        }

        // 2. Render Nodes
        for node in &self.modular_nodes {
            let n_rect = Rect::from_min_size(
                egui::pos2(rect.left() + node.pos.0, rect.top() + node.pos.1),
                Vec2::new(node.size.0, node.size.1),
            );
            let is_sel = self.selected_modular_node_id.as_deref() == Some(&node.id);
            let (cr, cg, cb) = node.category.color_rgb();
            let cat_col = Color32::from_rgb(cr, cg, cb);

            // Background chassis
            painter.rect_filled(n_rect, 4.0, Color32::from_rgb(16, 24, 38));
            if is_sel {
                painter.rect_stroke(n_rect.expand(2.0), 5.0, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(cr, cg, cb, 60)));
                painter.rect_stroke(n_rect, 4.0, Stroke::new(1.8_f32, cat_col));
            } else {
                painter.rect_stroke(n_rect, 4.0, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(cr, cg, cb, 140)));
            }

            // Header
            painter.text(
                egui::pos2(n_rect.left() + 8.0, n_rect.top() + 6.0),
                egui::Align2::LEFT_TOP,
                format!("{} {}", node.category.icon(), node.display_name),
                FontId::proportional(11.0),
                Color32::from_rgb(241, 245, 249),
            );
            painter.circle_filled(egui::pos2(n_rect.right() - 12.0, n_rect.top() + 12.0), 3.5, cat_col);

            // Sockets
            for port in &node.ports {
                let s_pos = egui::pos2(n_rect.left() + port.rel_pos.0, n_rect.top() + port.rel_pos.1);
                let (pr, pg, pb) = port.kind.color_rgb();
                let port_col = Color32::from_rgb(pr, pg, pb);

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
                painter.text(text_pos, align, &port.name, FontId::proportional(9.0), Color32::from_rgb(148, 163, 184));
            }
        }

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

        // 3. Render Patch Cords (Bézier curves with gravity sag and glow)
        for cord in &self.patch_cords {
            if let (Some(src_pt), Some(dst_pt)) = (
                find_socket_pos(&cord.from_node_id, &cord.from_port_id, &self.modular_nodes),
                find_socket_pos(&cord.to_node_id, &cord.to_port_id, &self.modular_nodes),
            ) {
                let sag = ((dst_pt.x - src_pt.x).abs() * 0.2 + (dst_pt.y - src_pt.y).abs() * 0.15).clamp(20.0, 60.0);
                let c1 = egui::pos2(src_pt.x + 30.0, src_pt.y + sag);
                let c2 = egui::pos2(dst_pt.x - 30.0, dst_pt.y + sag);

                let color = if cord.is_audio {
                    Color32::from_rgb(56, 189, 248) // Cyan
                } else {
                    Color32::from_rgb(245, 158, 11) // Amber
                };

                // Glow shadow
                let shadow_col = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 65);
                let b_glow = egui::epaint::CubicBezierShape::from_points_stroke(
                    [src_pt, c1, c2, dst_pt],
                    false,
                    Color32::TRANSPARENT,
                    Stroke::new(3.5_f32, shadow_col),
                );
                painter.add(b_glow);

                // Core cable
                let b_core = egui::epaint::CubicBezierShape::from_points_stroke(
                    [src_pt, c1, c2, dst_pt],
                    false,
                    Color32::TRANSPARENT,
                    Stroke::new(1.8_f32, color),
                );
                painter.add(b_core);

                // Active animated signal pulse along the cord
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
                painter.circle_filled(pulse_pt, 2.8, Color32::WHITE);
                painter.circle_stroke(pulse_pt, 4.0, Stroke::new(1.0_f32, color));
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
        let canvas_height = (self.tracks.len() as f32 * 36.0 + 36.0).max(280.0);
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

                // Mixer interactions: Mute, Solo, Volume Fader, Track Selection
                let pointer_pos = resp.interact_pointer_pos().or_else(|| ui.input(|i| i.pointer.latest_pos()));
                let is_interacting = resp.clicked() || resp.dragged() || ui.input(|i| i.pointer.primary_down() || i.pointer.primary_clicked());
                let is_click = resp.clicked() || ui.input(|i| i.pointer.primary_clicked() || (i.pointer.primary_down() && !resp.dragged()));

                if let Some(pos) = pointer_pos {
                    if is_interacting {
                        let mut selected_idx = None;
                        for (idx, track) in self.tracks.iter_mut().enumerate() {
                            let sx = rect.left() + idx as f32 * strip_w;
                            let strip_rect = Rect::from_min_size(egui::pos2(sx, rect.top() + 4.0), Vec2::new(strip_w - 4.0, canvas_height - 8.0));
                            if strip_rect.contains(pos) {
                                let pan_y = strip_rect.top() + 32.0;
                                let m_rect = Rect::from_min_size(egui::pos2(strip_rect.left() + 4.0, pan_y + 14.0), Vec2::new((strip_rect.width() - 10.0) / 2.0, 16.0));
                                let s_rect = Rect::from_min_size(egui::pos2(m_rect.right() + 2.0, pan_y + 14.0), Vec2::new(m_rect.width(), 16.0));
                                let fader_top = s_rect.bottom() + 10.0;
                                let fader_bot = strip_rect.bottom() - 20.0;

                                if m_rect.contains(pos) && is_click {
                                    track.is_muted = !track.is_muted;
                                    selected_idx = Some(idx);
                                } else if s_rect.contains(pos) && is_click {
                                    track.is_soloed = !track.is_soloed;
                                    selected_idx = Some(idx);
                                } else if pos.y >= fader_top - 6.0 && pos.y <= fader_bot + 6.0 {
                                    let norm = ((fader_bot - pos.y) / (fader_bot - fader_top)).clamp(0.0, 1.0);
                                    track.gain = norm * 1.5;
                                    selected_idx = Some(idx);
                                } else if is_click {
                                    selected_idx = Some(idx);
                                }
                            }
                        }
                        if let Some(s_idx) = selected_idx {
                            self.selected_track_idx = s_idx;
                            if let Some(tr) = self.tracks.get(s_idx) {
                                self.top_bar_state.master_gain = tr.gain;
                                self.inspector_state.target_name = tr.name.clone();
                                self.inspector_state.gain_db = (tr.gain - 1.0) * 12.0;
                                self.inspector_state.pan_val = tr.pan;
                                self.inspector_state.is_muted = tr.is_muted;
                                self.inspector_state.is_soloed = tr.is_soloed;
                                self.inspector_state.is_armed = tr.is_armed;
                                self.device_rack_state.device_name = tr.name.clone();
                            }
                        }
                    }
                }

                // Draw Track Channel Strips 1..8
                for (idx, track) in self.tracks.iter().enumerate() {
                    let sx = rect.left() + idx as f32 * strip_w;
                    let strip_rect = Rect::from_min_size(egui::pos2(sx, rect.top() + 4.0), Vec2::new(strip_w - 4.0, canvas_height - 8.0));
                    let is_sel = idx == self.selected_track_idx;
                    let bg = if is_sel { Color32::from_rgb(18, 26, 42) } else { Color32::from_rgb(12, 16, 26) };
                    painter.rect_filled(strip_rect, 4.0, bg);
                    painter.rect_stroke(strip_rect, 4.0, Stroke::new(1.0_f32, if is_sel { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(28, 38, 56) }));

                    // Header color pill + Name
                    let col = Color32::from_rgb(track.color_rgb[0], track.color_rgb[1], track.color_rgb[2]);
                    painter.rect_filled(Rect::from_min_size(egui::pos2(strip_rect.left() + 4.0, strip_rect.top() + 4.0), Vec2::new(strip_rect.width() - 8.0, 4.0)), 2.0, col);
                    painter.text(egui::pos2(strip_rect.center().x, strip_rect.top() + 12.0), egui::Align2::CENTER_TOP, format!("{}. {}", idx + 1, track.name), FontId::proportional(10.0), Color32::from_rgb(241, 245, 249));

                    // Pan Pot
                    let pan_y = strip_rect.top() + 32.0;
                    painter.circle_filled(egui::pos2(strip_rect.center().x, pan_y), 8.0, Color32::from_rgb(18, 24, 36));
                    painter.circle_stroke(egui::pos2(strip_rect.center().x, pan_y), 8.0, Stroke::new(1.0_f32, Color32::from_rgb(45, 60, 85)));

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
                let m_strip = Rect::from_min_size(egui::pos2(m_x, rect.top() + 4.0), Vec2::new(master_w, canvas_height - 8.0));
                painter.rect_filled(m_strip, 4.0, Color32::from_rgb(16, 24, 38));
                painter.rect_stroke(m_strip, 4.0, Stroke::new(1.5_f32, Color32::from_rgb(56, 189, 248)));
                painter.text(egui::pos2(m_strip.center().x, m_strip.top() + 8.0), egui::Align2::CENTER_TOP, "MASTER", FontId::proportional(11.0), Color32::from_rgb(56, 189, 248));

                let m_fader_top = m_strip.top() + 32.0;
                let m_fader_bot = m_strip.bottom() - 24.0;
                let m_fader_x = m_strip.left() + m_strip.width() * 0.35;

                if let Some(pos) = pointer_pos {
                    if is_interacting && m_strip.contains(pos) && pos.y >= m_fader_top - 6.0 && pos.y <= m_fader_bot + 6.0 {
                        let norm = ((m_fader_bot - pos.y) / (m_fader_bot - m_fader_top)).clamp(0.0, 1.0);
                        self.top_bar_state.master_gain = norm * 2.0;
                    }
                }

                // Draw Master fader slot & thumb
                painter.line_segment([egui::pos2(m_fader_x, m_fader_top), egui::pos2(m_fader_x, m_fader_bot)], Stroke::new(2.5_f32, Color32::from_rgb(8, 12, 18)));
                let m_norm = (self.top_bar_state.master_gain / 2.0).clamp(0.0, 1.0);
                let m_thumb_y = m_fader_bot - m_norm * (m_fader_bot - m_fader_top);
                let m_thumb_r = Rect::from_center_size(egui::pos2(m_fader_x, m_thumb_y), Vec2::new(20.0, 9.0));
                painter.rect_filled(m_thumb_r, 2.0, Color32::from_rgb(56, 189, 248));
                painter.rect_stroke(m_thumb_r, 2.0, Stroke::new(1.0_f32, Color32::WHITE));

                // Dual Stereo Master VU Meter
                let m_meter_x = m_strip.right() - 18.0;
                let m_meter_rect = Rect::from_min_size(egui::pos2(m_meter_x, m_fader_top), Vec2::new(12.0, m_fader_bot - m_fader_top));
                painter.rect_filled(m_meter_rect, 1.0, Color32::from_rgb(6, 10, 16));
                let m_fill_h = (m_meter_rect.height() * m_norm).max(2.0);
                let m_fill_rect = Rect::from_min_max(egui::pos2(m_meter_rect.left(), m_meter_rect.bottom() - m_fill_h), m_meter_rect.right_bottom());
                painter.rect_filled(m_fill_rect, 1.0, Color32::from_rgb(16, 185, 129));

                let m_db = if self.top_bar_state.master_gain > 0.001 { (self.top_bar_state.master_gain - 1.0) * 12.0 } else { -96.0 };
                painter.text(egui::pos2(m_strip.center().x, m_strip.bottom() - 10.0), egui::Align2::CENTER_CENTER, format!("{:.1}dB", m_db), FontId::proportional(8.0), Color32::from_rgb(56, 189, 248));
            });
    }

    #[cfg(feature = "gui")]
    fn show_stage_canvas(&mut self, ui: &mut egui::Ui) {
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

                // 1. Top Bar inside Stage: TAP Tempo, Panic, Quantize Mode
                let bar_h = 28.0;
                painter.rect_filled(Rect::from_min_size(rect.min, Vec2::new(rect.width(), bar_h)), 0.0, Color32::from_rgb(14, 20, 32));
                painter.text(egui::pos2(rect.left() + 12.0, rect.top() + 7.0), egui::Align2::LEFT_TOP, "STAGE VIEW (LIVE PERFORMANCE MATRIX)", FontId::proportional(11.0), Color32::from_rgb(56, 189, 248));

                // Panic Button
                let panic_rect = Rect::from_min_size(egui::pos2(rect.right() - 90.0, rect.top() + 4.0), Vec2::new(80.0, 20.0));
                let panic_hovered = pointer_pos.map(|p| panic_rect.contains(p)).unwrap_or(false);
                if let Some(pos) = pointer_pos {
                    if panic_rect.contains(pos) && is_click {
                        self.top_bar_state.is_playing = false;
                        self.last_synced_is_playing = false;
                        for tr in &mut self.tracks {
                            tr.is_armed = false;
                        }
                        self.panic_triggered = true;
                    }
                }
                let panic_bg = if self.panic_triggered || panic_hovered {
                    Color32::from_rgb(220, 38, 38)
                } else {
                    Color32::from_rgb(185, 28, 28)
                };
                painter.rect_filled(panic_rect, 3.0, panic_bg);
                painter.rect_stroke(panic_rect, 3.0, Stroke::new(1.0_f32, Color32::from_rgb(252, 165, 165)));
                painter.text(panic_rect.center(), egui::Align2::CENTER_CENTER, "PANIC (ESC)", FontId::proportional(9.0), Color32::WHITE);

                // 2. Matrix Grid: Tracks x 4 Scenes
                let scene_names = ["1 Intro", "2 Verse", "3 Drop", "4 Outro"];
                let grid_top = rect.top() + bar_h + 8.0;
                let track_count = self.tracks.len();
                let col_w = (rect.width() - 80.0) / track_count.max(1) as f32;
                let row_h = (canvas_height - bar_h - 20.0) / scene_names.len() as f32;

                // Scene Launch Buttons (Leftmost column)
                for (s_idx, s_name) in scene_names.iter().enumerate() {
                    let sy = grid_top + s_idx as f32 * row_h;
                    let sc_rect = Rect::from_min_size(egui::pos2(rect.left() + 8.0, sy + 2.0), Vec2::new(64.0, row_h - 4.0));
                    let is_active_scene = self.active_scene_idx == Some(s_idx);

                    if let Some(pos) = pointer_pos {
                        if sc_rect.contains(pos) && is_click {
                            self.active_scene_idx = Some(s_idx);
                            self.top_bar_state.is_playing = true;
                            self.last_synced_is_playing = true;
                            self.panic_triggered = false;
                            self.playhead_beat = (s_idx as f32) * 16.0;
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

                // Grid Pads
                let mut pad_track_selected = None;
                for (t_idx, track) in self.tracks.iter().enumerate() {
                    let col_x = rect.left() + 80.0 + t_idx as f32 * col_w;
                    let col = Color32::from_rgb(track.color_rgb[0], track.color_rgb[1], track.color_rgb[2]);

                    for (s_idx, _) in scene_names.iter().enumerate() {
                        let row_y = grid_top + s_idx as f32 * row_h;
                        let pad_rect = Rect::from_min_size(egui::pos2(col_x + 2.0, row_y + 2.0), Vec2::new(col_w - 4.0, row_h - 4.0));
                        let is_playing = self.active_scene_idx == Some(s_idx) || (self.active_scene_idx.is_none() && (t_idx + s_idx) % 2 == 0 && self.top_bar_state.is_playing);

                        if let Some(pos) = pointer_pos {
                            if pad_rect.contains(pos) && is_click {
                                pad_track_selected = Some(t_idx);
                                self.active_scene_idx = Some(s_idx);
                                self.top_bar_state.is_playing = true;
                                self.last_synced_is_playing = true;
                                self.panic_triggered = false;
                            }
                        }

                        let pad_bg = if is_playing {
                            Color32::from_rgba_unmultiplied(track.color_rgb[0], track.color_rgb[1], track.color_rgb[2], 65)
                        } else {
                            Color32::from_rgb(14, 18, 28)
                        };
                        painter.rect_filled(pad_rect, 3.0, pad_bg);
                        painter.rect_stroke(pad_rect, 3.0, Stroke::new(1.0_f32, if is_playing { col } else { Color32::from_rgb(28, 38, 56) }));

                        let pad_label = if is_playing { "▶ Clip" } else { "⬚" };
                        let label_col = if is_playing { col } else { Color32::from_rgb(100, 116, 139) };
                        painter.text(pad_rect.center(), egui::Align2::CENTER_CENTER, pad_label, FontId::proportional(9.0), label_col);
                    }
                }

                if let Some(s_idx) = pad_track_selected {
                    self.selected_track_idx = s_idx;
                    if let Some(tr) = self.tracks.get(s_idx) {
                        self.inspector_state.target_name = tr.name.clone();
                        self.inspector_state.gain_db = (tr.gain - 1.0) * 12.0;
                        self.inspector_state.pan_val = tr.pan;
                        self.inspector_state.is_muted = tr.is_muted;
                        self.inspector_state.is_soloed = tr.is_soloed;
                        self.inspector_state.is_armed = tr.is_armed;
                        self.device_rack_state.device_name = tr.name.clone();
                    }
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
