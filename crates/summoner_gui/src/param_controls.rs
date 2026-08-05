// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Parameter controls, inline editing, lock/lfo modulation, hover tooltips,
//! linked parameter groups, FX send routing, and FX chain bypass (Steps 748-760).

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Step 755: Parameter unit types for formatting display text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParamUnit {
    Hz,
    Db,
    Ms,
    Percent,
    Cents,
    Bpm,
    Raw,
}

impl ParamUnit {
    /// Format numerical parameter value into human-readable string with unit suffix.
    pub fn format_value(&self, val: f64) -> String {
        match self {
            ParamUnit::Hz => format!("{:.1} Hz", val),
            ParamUnit::Db => format!("{:.1} dB", val),
            ParamUnit::Ms => format!("{:.1} ms", val),
            ParamUnit::Percent => format!("{:.0}%", val * 100.0),
            ParamUnit::Cents => format!("{:.0} cents", val),
            ParamUnit::Bpm => format!("{:.1} BPM", val),
            ParamUnit::Raw => format!("{:.2}", val),
        }
    }
}

/// Step 756: Format tooltip text displaying parameter name, value, min, and max on hover.
pub fn format_hover_tooltip(
    name: &str,
    value: f64,
    min: f64,
    max: f64,
    unit: &ParamUnit,
) -> String {
    format!(
        "{}: {}\nRange: [{}, {}]",
        name,
        unit.format_value(value),
        unit.format_value(min),
        unit.format_value(max)
    )
}

/// Step 751: Parameter value clipboard for right-click copy & paste operations.
#[derive(Debug, Clone, Default)]
pub struct ParamClipboard {
    pub value: Option<f64>,
}

impl ParamClipboard {
    pub fn copy(&mut self, val: f64) {
        self.value = Some(val);
    }

    pub fn paste(&self) -> Option<f64> {
        self.value
    }
}

/// Step 752: Parameter lock manager to prevent accidental parameter mutations.
#[derive(Debug, Clone, Default)]
pub struct ParamLockManager {
    pub locked_params: HashSet<String>,
}

impl ParamLockManager {
    pub fn lock(&mut self, param_id: &str) {
        self.locked_params.insert(param_id.to_string());
    }

    pub fn unlock(&mut self, param_id: &str) {
        self.locked_params.remove(param_id);
    }

    pub fn toggle_lock(&mut self, param_id: &str) -> bool {
        if self.is_locked(param_id) {
            self.unlock(param_id);
            false
        } else {
            self.lock(param_id);
            true
        }
    }

    pub fn is_locked(&self, param_id: &str) -> bool {
        self.locked_params.contains(param_id)
    }

    pub fn apply_mutation(&self, param_id: &str, current_val: f64, new_val: f64) -> f64 {
        if self.is_locked(param_id) {
            current_val
        } else {
            new_val
        }
    }
}

/// Step 753: Per-parameter LFO modulator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamLfoModulator {
    pub param_id: String,
    pub frequency_hz: f64,
    pub depth: f64,
    pub active: bool,
}

impl ParamLfoModulator {
    pub fn new(param_id: &str, frequency_hz: f64, depth: f64) -> Self {
        Self {
            param_id: param_id.to_string(),
            frequency_hz,
            depth,
            active: true,
        }
    }

    /// Evaluate LFO offset at time `t` (seconds).
    pub fn evaluate(&self, base_value: f64, t: f64) -> f64 {
        if !self.active {
            return base_value;
        }
        let lfo_val = (2.0 * std::f64::consts::PI * self.frequency_hz * t).sin();
        base_value + (lfo_val * self.depth)
    }
}

/// Step 754: Per-parameter deterministic randomizer with range limits.
pub fn randomize_parameter(param_id: &str, min: f64, max: f64, seed: u64) -> f64 {
    let mut hash = seed ^ (param_id.len() as u64);
    for b in param_id.bytes() {
        hash = hash
            .wrapping_mul(6364136223846793005)
            .wrapping_add(b as u64);
    }
    let norm = (hash % 10_000) as f64 / 10_000.0;
    min + norm * (max - min)
}

/// Step 749: Inline parameter value editor for double-click text input.
#[derive(Debug, Clone, Default)]
pub struct InlineValueEditor {
    pub editing_param_id: Option<String>,
    pub text_buffer: String,
}

impl InlineValueEditor {
    pub fn start_edit(&mut self, param_id: &str, current_val: f64) {
        self.editing_param_id = Some(param_id.to_string());
        self.text_buffer = format!("{:.2}", current_val);
    }

    pub fn submit_edit(&mut self) -> Option<(String, f64)> {
        if let Some(id) = self.editing_param_id.take() {
            if let Ok(parsed) = self.text_buffer.trim().parse::<f64>() {
                return Some((id, parsed));
            }
        }
        None
    }

    pub fn cancel_edit(&mut self) {
        self.editing_param_id = None;
        self.text_buffer.clear();
    }
}

/// Step 750: Reset parameter to default value on Ctrl+double-click.
pub fn reset_param_to_default(default_val: f64) -> f64 {
    default_val
}

/// Step 757: Linked parameter manager updating target parameters proportional to source.
#[derive(Debug, Clone, Default)]
pub struct LinkedParamGroup {
    pub links: HashMap<String, Vec<(String, f64)>>, // source_id -> Vec<(target_id, ratio)>
}

impl LinkedParamGroup {
    pub fn link(&mut self, source_id: &str, target_id: &str, ratio: f64) {
        self.links
            .entry(source_id.to_string())
            .or_default()
            .push((target_id.to_string(), ratio));
    }

    pub fn compute_linked_values(
        &self,
        source_id: &str,
        source_new_val: f64,
    ) -> Vec<(String, f64)> {
        let mut updates = Vec::new();
        if let Some(targets) = self.links.get(source_id) {
            for (target_id, ratio) in targets {
                updates.push((target_id.clone(), source_new_val * ratio));
            }
        }
        updates
    }
}

/// Step 758: Collapsible parameter group for organizing related controls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamGroup {
    pub name: String,
    pub collapsed: bool,
    pub param_ids: Vec<String>,
}

impl ParamGroup {
    pub fn new(name: &str, param_ids: &[&str]) -> Self {
        Self {
            name: name.to_string(),
            collapsed: false,
            param_ids: param_ids.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn toggle_collapsed(&mut self) {
        self.collapsed = !self.collapsed;
    }
}

/// Step 748: Context menu actions available on parameter controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextMenuAction {
    ResetToDefault,
    CopyValue,
    PasteValue,
    ToggleLock,
    ModulateWithLfo,
    Randomize,
}

/// Step 759: Send signal amount routing to FX send bus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendFxRoute {
    pub track_id: u64,
    pub bus_name: String,
    pub send_amount: f32, // 0.0 to 1.0
}

impl SendFxRoute {
    pub fn new(track_id: u64, bus_name: &str, send_amount: f32) -> Self {
        Self {
            track_id,
            bus_name: bus_name.to_string(),
            send_amount: send_amount.clamp(0.0, 1.0),
        }
    }
}

/// Step 760: FX chain bypass manager to toggle or query master/insert FX chain bypass state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FxChainBypassManager {
    pub insert_bypassed: bool,
}

impl FxChainBypassManager {
    pub fn toggle_bypass(&mut self) -> bool {
        self.insert_bypassed = !self.insert_bypassed;
        self.insert_bypassed
    }

    pub fn process_chain<'a>(&self, input: &'a [f32], fx_processed: &'a [f32]) -> &'a [f32] {
        if self.insert_bypassed {
            input
        } else {
            fx_processed
        }
    }
}

/// Step 761: FX compare slot (Slot A vs Slot B) for A/B testing insert FX chains.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FxCompareSlot {
    SlotA,
    SlotB,
}

/// Step 761: FX chain A/B comparison state manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FxChainCompare {
    pub active_slot: FxCompareSlot,
    pub slot_a_with_fx: bool,
    pub slot_b_with_fx: bool,
}

impl Default for FxChainCompare {
    fn default() -> Self {
        Self {
            active_slot: FxCompareSlot::SlotA,
            slot_a_with_fx: true,
            slot_b_with_fx: false,
        }
    }
}

impl FxChainCompare {
    pub fn toggle_slot(&mut self) -> FxCompareSlot {
        self.active_slot = match self.active_slot {
            FxCompareSlot::SlotA => FxCompareSlot::SlotB,
            FxCompareSlot::SlotB => FxCompareSlot::SlotA,
        };
        self.active_slot
    }

    pub fn process_compare<'a>(&self, dry_signal: &'a [f32], wet_signal: &'a [f32]) -> &'a [f32] {
        let is_wet = match self.active_slot {
            FxCompareSlot::SlotA => self.slot_a_with_fx,
            FxCompareSlot::SlotB => self.slot_b_with_fx,
        };
        if is_wet {
            wet_signal
        } else {
            dry_signal
        }
    }
}

/// Step 762: FX chain preset representation for saving and loading effect configurations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FxChainPreset {
    pub name: String,
    pub nodes: Vec<summoner_project::schema::NodeConfig>,
}

pub fn save_fx_chain_preset(
    name: &str,
    nodes: &[summoner_project::schema::NodeConfig],
) -> FxChainPreset {
    FxChainPreset {
        name: name.to_string(),
        nodes: nodes.to_vec(),
    }
}

pub fn load_fx_chain_preset(preset: &FxChainPreset) -> Vec<summoner_project::schema::NodeConfig> {
    preset.nodes.clone()
}

/// Step 763: Pre/Post fader position selector for insert FX.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InsertFxPosition {
    PreFader,
    PostFader,
}

pub fn process_insert_fx_with_position(
    position: InsertFxPosition,
    fader_gain: f32,
    input: &[f32],
    fx_processed: &[f32],
) -> Vec<f32> {
    let mut out = vec![0.0f32; input.len().min(fx_processed.len())];
    match position {
        InsertFxPosition::PreFader => {
            for i in 0..out.len() {
                out[i] = fx_processed[i] * fader_gain;
            }
        }
        InsertFxPosition::PostFader => {
            for i in 0..out.len() {
                out[i] = fx_processed[i];
            }
        }
    }
    out
}

/// Step 764: Sidechain source selector for individual insert FX slots.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InsertFxSidechainConfig {
    pub effect_index: usize,
    pub sidechain_source_track_id: Option<u64>,
}

pub fn set_effect_sidechain(
    configs: &mut Vec<InsertFxSidechainConfig>,
    effect_idx: usize,
    source_track_id: Option<u64>,
) {
    if let Some(cfg) = configs.iter_mut().find(|c| c.effect_index == effect_idx) {
        cfg.sidechain_source_track_id = source_track_id;
    } else {
        configs.push(InsertFxSidechainConfig {
            effect_index: effect_idx,
            sidechain_source_track_id: source_track_id,
        });
    }
}

/// Step 765: Visual style representation for sidechain connections in node graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeGraphWireStyle {
    pub is_sidechain: bool,
    pub stroke_dashed: bool,
}

pub fn get_wire_stroke_style(is_sidechain: bool) -> NodeGraphWireStyle {
    NodeGraphWireStyle {
        is_sidechain,
        stroke_dashed: is_sidechain,
    }
}

/// Step 769: Auto-crossfade configuration for overlapping audio/sequence clips.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoCrossfadeConfig {
    pub enabled: bool,
    pub fade_duration_beats: f64,
}

impl Default for AutoCrossfadeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            fade_duration_beats: 0.25,
        }
    }
}

pub fn compute_auto_crossfade(
    clip_a_start: f64,
    clip_a_len: f64,
    clip_b_start: f64,
    fade_duration: f64,
) -> Option<(f64, f64)> {
    let clip_a_end = clip_a_start + clip_a_len;
    if clip_b_start < clip_a_end && clip_b_start >= clip_a_start {
        let overlap = clip_a_end - clip_b_start;
        let actual_fade = overlap.min(fade_duration);
        Some((actual_fade, actual_fade))
    } else {
        None
    }
}

/// Step 770: Export audio clip to SFZ format with WAV sample rendering.
pub fn export_clip_to_sfz(
    clip_name: &str,
    pcm_data: &[f32],
    sample_rate: u32,
    root_key: u8,
    output_dir: &std::path::Path,
) -> Result<(std::path::PathBuf, std::path::PathBuf), String> {
    std::fs::create_dir_all(output_dir).map_err(|e| e.to_string())?;

    let wav_path = output_dir.join(format!("{}.wav", clip_name));
    let sfz_path = output_dir.join(format!("{}.sfz", clip_name));

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(&wav_path, spec).map_err(|e| e.to_string())?;
    for &sample in pcm_data {
        let i16_val = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
        writer.write_sample(i16_val).map_err(|e| e.to_string())?;
    }
    writer.finalize().map_err(|e| e.to_string())?;

    let sfz_content = format!(
        "<group>\n<region> sample={}.wav key={}\n",
        clip_name, root_key
    );
    std::fs::write(&sfz_path, sfz_content).map_err(|e| e.to_string())?;

    Ok((wav_path, sfz_path))
}

/// Step 771: Drum Machine 4x16 pad step grid representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrumMachineGrid {
    pub grid: [[bool; 16]; 4],
}

impl Default for DrumMachineGrid {
    fn default() -> Self {
        Self {
            grid: [[false; 16]; 4],
        }
    }
}

impl DrumMachineGrid {
    pub fn toggle_step(&mut self, pad_idx: usize, step_idx: usize) -> bool {
        if pad_idx < 4 && step_idx < 16 {
            self.grid[pad_idx][step_idx] = !self.grid[pad_idx][step_idx];
            self.grid[pad_idx][step_idx]
        } else {
            false
        }
    }

    pub fn is_step_active(&self, pad_idx: usize, step_idx: usize) -> bool {
        if pad_idx < 4 && step_idx < 16 {
            self.grid[pad_idx][step_idx]
        } else {
            false
        }
    }
}

/// Step 772: Calculate velocity from drum pad click Y coordinate.
pub fn compute_pad_click_velocity(relative_y: f32, pad_height: f32) -> u8 {
    if pad_height <= 0.0 {
        return 100;
    }
    let norm = (relative_y / pad_height).clamp(0.0, 1.0);
    let vel = ((1.0 - norm) * 126.0 + 1.0) as u8;
    vel.clamp(1, 127)
}

/// Step 773: Map MIDI note/drum hit to distinct pad RGB color.
pub fn get_drum_pad_color(midi_note: u8) -> [u8; 3] {
    match midi_note {
        35 | 36 => [231, 76, 60],
        38 | 40 => [52, 152, 219],
        42 | 44 | 46 => [241, 196, 15],
        45 | 47 | 48 | 50 => [46, 204, 113],
        39 => [155, 89, 182],
        49 | 51 => [230, 126, 34],
        _ => [127, 140, 141],
    }
}

/// Step 774: Mute group manager for exclusive pad muting (e.g. Kick + HiHat).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PadMuteGroupManager {
    pub mute_groups: HashMap<u8, u8>,
}

impl PadMuteGroupManager {
    pub fn set_pad_mute_group(&mut self, pad_id: u8, group_id: u8) {
        self.mute_groups.insert(pad_id, group_id);
    }

    pub fn trigger_pad(&self, pad_id: u8, active_pads: &mut HashSet<u8>) {
        if let Some(&group) = self.mute_groups.get(&pad_id) {
            active_pads.retain(|&other| {
                if let Some(&other_group) = self.mute_groups.get(&other) {
                    other == pad_id || other_group != group
                } else {
                    true
                }
            });
        }
        active_pads.insert(pad_id);
    }
}

/// Step 775: Pad note assignment table for mapping drum pads to MIDI pitches and labels.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PadNoteAssignmentTable {
    pub assignments: HashMap<u8, (u8, String)>,
}

impl PadNoteAssignmentTable {
    pub fn assign_note(&mut self, pad_id: u8, midi_note: u8, label: &str) {
        self.assignments
            .insert(pad_id, (midi_note, label.to_string()));
    }

    pub fn get_note(&self, pad_id: u8) -> (u8, String) {
        self.assignments
            .get(&pad_id)
            .cloned()
            .unwrap_or((36 + pad_id, format!("Pad {}", pad_id + 1)))
    }
}

/// Step 776: Pad choke group manager for cutting active voice polyphony on trigger.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PadChokeGroupManager {
    pub choke_groups: HashMap<u8, u8>,
}

impl PadChokeGroupManager {
    pub fn set_choke_group(&mut self, pad_id: u8, group_id: u8) {
        self.choke_groups.insert(pad_id, group_id);
    }

    pub fn choke_active_voices(&self, trigger_pad: u8, active_voices: &mut Vec<u8>) {
        if let Some(&group) = self.choke_groups.get(&trigger_pad) {
            active_voices.retain(|&voice_pad| {
                if let Some(&voice_group) = self.choke_groups.get(&voice_pad) {
                    voice_pad == trigger_pad || voice_group != group
                } else {
                    true
                }
            });
        }
        if !active_voices.contains(&trigger_pad) {
            active_voices.push(trigger_pad);
        }
    }
}

/// Step 777: Humanize drum pattern by applying micro-shift timing jitter and velocity variation.
pub fn humanize_drum_pattern(
    steps: &mut [summoner_project::schema::TrackerStepConfig],
    timing_jitter_ms: f64,
    velocity_jitter: u8,
    seed: u64,
) {
    for (idx, step) in steps.iter_mut().enumerate() {
        let rng_val =
            ((seed.wrapping_add((idx as u64).wrapping_mul(1103515245))) % 100) as f64 / 100.0;
        let jitter_shift = (rng_val * 2.0 - 1.0) * timing_jitter_ms;
        step.micro_shift = (step.micro_shift as f64 + jitter_shift).round() as i32;

        let vel_rng = ((seed.wrapping_add((idx as u64).wrapping_mul(12345))) % 100) as f32 / 100.0;
        let vel_delta = (vel_rng * 2.0 - 1.0) * (velocity_jitter as f32 / 127.0);
        step.velocity = (step.velocity + vel_delta).clamp(0.0, 1.0);
    }
}

/// Step 778: Full-screen display mode toggle with F11 hotkey support.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FullScreenToggle {
    pub is_fullscreen: bool,
}

impl FullScreenToggle {
    pub fn handle_key_event(&mut self, is_f11: bool) -> bool {
        if is_f11 {
            self.is_fullscreen = !self.is_fullscreen;
        }
        self.is_fullscreen
    }
}

/// Step 779: Always-on-top window display mode configuration for Stage View.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StageViewWindowConfig {
    pub always_on_top: bool,
}

impl StageViewWindowConfig {
    pub fn toggle_always_on_top(&mut self) -> bool {
        self.always_on_top = !self.always_on_top;
        self.always_on_top
    }
}

/// Step 780: Panel layout configuration state for custom UI workspace save/restore.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelLayoutConfig {
    pub layout_name: String,
    pub panel_visibility: HashMap<String, bool>,
    pub panel_sizes: HashMap<String, (f32, f32)>,
}

/// Step 780: Layout manager for persisting and loading workspace UI layouts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LayoutManager {
    pub layouts: HashMap<String, PanelLayoutConfig>,
}

impl LayoutManager {
    pub fn save_layout(
        &mut self,
        name: &str,
        visibility: HashMap<String, bool>,
        sizes: HashMap<String, (f32, f32)>,
    ) {
        self.layouts.insert(
            name.to_string(),
            PanelLayoutConfig {
                layout_name: name.to_string(),
                panel_visibility: visibility,
                panel_sizes: sizes,
            },
        );
    }

    pub fn restore_layout(&self, name: &str) -> Option<&PanelLayoutConfig> {
        self.layouts.get(name)
    }
}

/// Step 781: Default workspace panel layout presets.
pub fn get_default_layout_presets() -> HashMap<String, PanelLayoutConfig> {
    let mut map = HashMap::new();

    let mut comp_vis = HashMap::new();
    comp_vis.insert("Arranger".to_string(), true);
    comp_vis.insert("PianoRoll".to_string(), true);
    comp_vis.insert("Mixer".to_string(), false);
    comp_vis.insert("NodeGraph".to_string(), false);
    map.insert(
        "Composition".to_string(),
        PanelLayoutConfig {
            layout_name: "Composition".to_string(),
            panel_visibility: comp_vis,
            panel_sizes: HashMap::new(),
        },
    );

    let mut mix_vis = HashMap::new();
    mix_vis.insert("Arranger".to_string(), true);
    mix_vis.insert("PianoRoll".to_string(), false);
    mix_vis.insert("Mixer".to_string(), true);
    mix_vis.insert("NodeGraph".to_string(), false);
    map.insert(
        "Mixing".to_string(),
        PanelLayoutConfig {
            layout_name: "Mixing".to_string(),
            panel_visibility: mix_vis,
            panel_sizes: HashMap::new(),
        },
    );

    let mut mast_vis = HashMap::new();
    mast_vis.insert("Arranger".to_string(), true);
    mast_vis.insert("PianoRoll".to_string(), false);
    mast_vis.insert("Mixer".to_string(), true);
    mast_vis.insert("NodeGraph".to_string(), true);
    map.insert(
        "Mastering".to_string(),
        PanelLayoutConfig {
            layout_name: "Mastering".to_string(),
            panel_visibility: mast_vis,
            panel_sizes: HashMap::new(),
        },
    );

    let mut live_vis = HashMap::new();
    live_vis.insert("StageView".to_string(), true);
    live_vis.insert("MacroRack".to_string(), true);
    map.insert(
        "Live".to_string(),
        PanelLayoutConfig {
            layout_name: "Live".to_string(),
            panel_visibility: live_vis,
            panel_sizes: HashMap::new(),
        },
    );

    map
}

/// Step 782: Layout hotkey manager for switching between workspace presets.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LayoutHotkeyManager {
    pub key_map: HashMap<String, String>,
}

impl LayoutHotkeyManager {
    pub fn new_default() -> Self {
        let mut key_map = HashMap::new();
        key_map.insert("F1".to_string(), "Composition".to_string());
        key_map.insert("F2".to_string(), "Mixing".to_string());
        key_map.insert("F3".to_string(), "Mastering".to_string());
        key_map.insert("F4".to_string(), "Live".to_string());
        Self { key_map }
    }

    pub fn handle_hotkey(&self, key: &str) -> Option<&str> {
        self.key_map.get(key).map(|s| s.as_str())
    }
}

/// Step 783: Timeline zoom helper for Zoom to Fit and Zoom to Selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineZoomState {
    pub pixels_per_beat: f32,
}

impl Default for TimelineZoomState {
    fn default() -> Self {
        Self {
            pixels_per_beat: 50.0,
        }
    }
}

impl TimelineZoomState {
    pub fn zoom_to_fit(&mut self, total_duration_beats: f32, viewport_width_px: f32) -> f32 {
        if total_duration_beats > 0.0 && viewport_width_px > 0.0 {
            self.pixels_per_beat = (viewport_width_px / total_duration_beats).clamp(10.0, 400.0);
        }
        self.pixels_per_beat
    }

    pub fn zoom_to_selection(
        &mut self,
        sel_start_beat: f32,
        sel_end_beat: f32,
        viewport_width_px: f32,
    ) -> f32 {
        let duration = (sel_end_beat - sel_start_beat).abs();
        if duration > 0.0 && viewport_width_px > 0.0 {
            self.pixels_per_beat = (viewport_width_px / duration).clamp(10.0, 400.0);
        }
        self.pixels_per_beat
    }
}

/// Step 784: Per-view zoom state persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewZoomPersistence {
    pub arranger_ppb: f32,
    pub piano_roll_ppb: f32,
    pub piano_roll_ppk: f32,
    pub automation_ppb: f32,
}

impl Default for ViewZoomPersistence {
    fn default() -> Self {
        Self {
            arranger_ppb: 50.0,
            piano_roll_ppb: 60.0,
            piano_roll_ppk: 20.0,
            automation_ppb: 40.0,
        }
    }
}

/// Step 785: Project search panel item result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchResultKind {
    Track,
    Clip,
    Node,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub kind: SearchResultKind,
    pub name: String,
    pub id: u64,
}

/// Step 785: Find in Project panel logic.
pub fn search_project(
    project: &summoner_project::schema::ProjectConfig,
    query: &str,
) -> Vec<SearchResultItem> {
    let mut results = Vec::new();
    if query.trim().is_empty() {
        return results;
    }
    let query_lower = query.to_lowercase();

    for track in &project.tracks {
        if track.name.to_lowercase().contains(&query_lower) {
            results.push(SearchResultItem {
                kind: SearchResultKind::Track,
                name: track.name.clone(),
                id: track.id,
            });
        }

        for clip in track.all_sequences() {
            if let Some(ref name) = clip.clip_name {
                if name.to_lowercase().contains(&query_lower) {
                    results.push(SearchResultItem {
                        kind: SearchResultKind::Clip,
                        name: name.clone(),
                        id: track.id,
                    });
                }
            }
        }

        for (idx, node) in track.nodes.iter().enumerate() {
            if node.kind.to_lowercase().contains(&query_lower) {
                results.push(SearchResultItem {
                    kind: SearchResultKind::Node,
                    name: format!("{} (#{})", node.kind, idx),
                    id: idx as u64,
                });
            }
        }
    }

    results
}

/// Step 786: Project statistics model.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectStatistics {
    pub total_tracks: usize,
    pub total_clips: usize,
    pub total_nodes: usize,
    pub total_parameters: usize,
}

impl ProjectStatistics {
    pub fn compute(project: &summoner_project::schema::ProjectConfig) -> Self {
        let mut stats = Self::default();
        stats.total_tracks = project.tracks.len();

        for track in &project.tracks {
            stats.total_clips += track.all_sequences().len();
            stats.total_nodes += track.nodes.len();
            for node in &track.nodes {
                stats.total_parameters += node.params.len();
            }
        }

        stats
    }
}

/// Step 787: Node Usage panel helper returning unique node types used in a project.
pub fn get_unique_node_types_used(
    project: &summoner_project::schema::ProjectConfig,
) -> Vec<String> {
    let mut types = HashSet::new();
    for track in &project.tracks {
        for node in &track.nodes {
            types.insert(node.kind.clone());
        }
    }
    let mut result: Vec<String> = types.into_iter().collect();
    result.sort();
    result
}

/// Step 788: Dependency graph overview adjacency computation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependencyGraphOverview {
    pub adjacency: HashMap<usize, Vec<usize>>,
}

impl DependencyGraphOverview {
    pub fn from_connections(connections: &[summoner_project::schema::ConnectionConfig]) -> Self {
        let mut adjacency: HashMap<usize, Vec<usize>> = HashMap::new();
        for conn in connections {
            if let (Ok(from_idx), Ok(to_idx)) =
                (conn.from.parse::<usize>(), conn.to.parse::<usize>())
            {
                adjacency.entry(from_idx).or_default().push(to_idx);
            }
        }
        Self { adjacency }
    }
}

/// Step 789: Signal flow animation mode for Node Graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalFlowAnimation {
    pub dot_phase: f32,
    pub dot_speed: f32,
}

impl Default for SignalFlowAnimation {
    fn default() -> Self {
        Self {
            dot_phase: 0.0,
            dot_speed: 2.0,
        }
    }
}

impl SignalFlowAnimation {
    pub fn step(&mut self, dt: f32) -> f32 {
        self.dot_phase = (self.dot_phase + self.dot_speed * dt) % 1.0;
        self.dot_phase
    }

    pub fn compute_dot_position(&self, start: (f32, f32), end: (f32, f32)) -> (f32, f32) {
        let x = start.0 + (end.0 - start.0) * self.dot_phase;
        let y = start.1 + (end.1 - start.1) * self.dot_phase;
        (x, y)
    }
}

/// Step 790: LOD switching in Node Graph depending on zoom scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeGraphLod {
    Detailed,
    Simplified,
    Compact,
}

pub fn get_node_graph_lod(zoom: f32) -> NodeGraphLod {
    if zoom >= 0.8 {
        NodeGraphLod::Detailed
    } else if zoom >= 0.4 {
        NodeGraphLod::Simplified
    } else {
        NodeGraphLod::Compact
    }
}

/// Step 791: Node grouping / subgraph box representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubGraphGroup {
    pub group_id: u64,
    pub name: String,
    pub node_ids: Vec<usize>,
    pub collapsed: bool,
}

impl SubGraphGroup {
    pub fn new(group_id: u64, name: &str, node_ids: Vec<usize>) -> Self {
        Self {
            group_id,
            name: name.to_string(),
            node_ids,
            collapsed: false,
        }
    }

    pub fn toggle_collapsed(&mut self) -> bool {
        self.collapsed = !self.collapsed;
        self.collapsed
    }
}

/// Step 792: Create device preset from subgraph.
pub fn create_preset_from_subgraph(
    group: &SubGraphGroup,
    nodes: &[summoner_project::schema::NodeConfig],
    preset_name: &str,
) -> summoner_project::preset::DevicePreset {
    let mut preset = summoner_project::preset::DevicePreset::new(preset_name, "Subgraph");
    preset.category = "Subgraph Presets".to_string();
    preset.comment = format!("Generated from subgraph {}", group.name);
    for (idx, node) in nodes.iter().enumerate() {
        if group.node_ids.contains(&idx) {
            for (p_name, p_val) in &node.params {
                preset
                    .params
                    .insert(format!("{}_{}", node.kind, p_name), *p_val);
            }
        }
    }
    preset
}

/// Step 793: GitHub Feature Request issue URL generator.
pub fn get_github_feature_request_url() -> &'static str {
    "https://github.com/nilsanderselde/summoner/issues/new?template=feature_request.md"
}

/// Step 794: GitHub Report Bug issue URL generator with diagnostic info.
pub fn generate_bug_report_url(os_info: &str, app_version: &str) -> String {
    let body = format!(
        "**OS**: {}\n**Version**: {}\n\n**Describe the bug**:\n",
        os_info, app_version
    );
    let encoded = body
        .replace(' ', "%20")
        .replace('\n', "%0A")
        .replace(':', "%3A");
    format!(
        "https://github.com/nilsanderselde/summoner/issues/new?title=Bug%20Report&body={}",
        encoded
    )
}

/// Step 795: Preferences panel categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreferencesCategory {
    Audio,
    Midi,
    Appearance,
    Shortcuts,
    Plugins,
}

impl Default for PreferencesCategory {
    fn default() -> Self {
        Self::Audio
    }
}

/// Step 796: Audio preferences config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioPreferencesConfig {
    pub device_name: String,
    pub sample_rate: u32,
    pub block_size: usize,
}

impl Default for AudioPreferencesConfig {
    fn default() -> Self {
        Self {
            device_name: "Default Output".to_string(),
            sample_rate: 44100,
            block_size: 256,
        }
    }
}

/// Step 797: MIDI preferences config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiPreferencesConfig {
    pub input_device: String,
    pub output_device: String,
    pub latency_ms: f32,
    pub routing_mode: String,
}

impl Default for MidiPreferencesConfig {
    fn default() -> Self {
        Self {
            input_device: "All Devices".to_string(),
            output_device: "None".to_string(),
            latency_ms: 5.0,
            routing_mode: "Omni".to_string(),
        }
    }
}

/// Step 798: Appearance preferences config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearancePreferencesConfig {
    pub theme: String,
    pub font_size: f32,
    pub default_zoom: f32,
}

impl Default for AppearancePreferencesConfig {
    fn default() -> Self {
        Self {
            theme: "Dark".to_string(),
            font_size: 14.0,
            default_zoom: 1.0,
        }
    }
}

/// Step 799: Shortcuts preferences config.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShortcutsPreferencesConfig {
    pub bindings: HashMap<String, String>,
}

impl ShortcutsPreferencesConfig {
    pub fn get_binding(&self, action: &str) -> Option<&str> {
        self.bindings.get(action).map(|s| s.as_str())
    }

    pub fn set_binding(&mut self, action: &str, hotkey: &str) {
        self.bindings.insert(action.to_string(), hotkey.to_string());
    }
}

/// Step 800: Plugins preferences config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginsPreferencesConfig {
    pub scan_paths: Vec<String>,
    pub safe_mode: bool,
    pub cpu_budget_percent: f32,
}

impl Default for PluginsPreferencesConfig {
    fn default() -> Self {
        Self {
            scan_paths: vec![
                "/Library/Audio/Plug-Ins/VST3".to_string(),
                "C:\\Program Files\\Common Files\\VST3".to_string(),
            ],
            safe_mode: true,
            cpu_budget_percent: 80.0,
        }
    }
}

/// Step 795: Full application Preferences State container.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PreferencesState {
    pub active_category: PreferencesCategory,
    pub audio: AudioPreferencesConfig,
    pub midi: MidiPreferencesConfig,
    pub appearance: AppearancePreferencesConfig,
    pub shortcuts: ShortcutsPreferencesConfig,
    pub plugins: PluginsPreferencesConfig,
}
