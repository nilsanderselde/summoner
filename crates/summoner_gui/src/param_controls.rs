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
