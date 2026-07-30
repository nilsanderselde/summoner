// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Parameter controls, inline editing, lock/lfo modulation, hover tooltips,
//! linked parameter groups, FX send routing, and FX chain bypass (Steps 748-760).

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

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
pub fn format_hover_tooltip(name: &str, value: f64, min: f64, max: f64, unit: &ParamUnit) -> String {
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
        hash = hash.wrapping_mul(6364136223846793005).wrapping_add(b as u64);
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

    pub fn compute_linked_values(&self, source_id: &str, source_new_val: f64) -> Vec<(String, f64)> {
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

pub fn save_fx_chain_preset(name: &str, nodes: &[summoner_project::schema::NodeConfig]) -> FxChainPreset {
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
        self.assignments.insert(pad_id, (midi_note, label.to_string()));
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
        let rng_val = ((seed.wrapping_add((idx as u64).wrapping_mul(1103515245))) % 100) as f64 / 100.0;
        let jitter_shift = (rng_val * 2.0 - 1.0) * timing_jitter_ms;
        step.micro_shift = (step.micro_shift as f64 + jitter_shift).round() as i32;

        let vel_rng = ((seed.wrapping_add((idx as u64).wrapping_mul(12345))) % 100) as f64 / 100.0;
        let vel_delta = ((vel_rng * 2.0 - 1.0) * velocity_jitter as f64).round() as i32;
        step.velocity = (step.velocity as i32 + vel_delta).clamp(1, 127) as u8;
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
