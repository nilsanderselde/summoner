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

//! Modulator DSP primitives (EnvADSR, LFO, MacroKnob, and MacroModulationMatrix).

use crate::traits::SignalProcessor;
use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;

/// ADSR envelope generator state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnvState {
    /// Envelope is idle at level 0.
    Idle,
    /// Envelope is rising to maximum attack level.
    Attack,
    /// Envelope is falling to sustain level.
    Decay,
    /// Envelope is holding at sustain level.
    Sustain,
    /// Envelope is decaying to zero release level.
    Release,
}

/// 4-stage exponential envelope generator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvADSR {
    /// Attack duration in seconds.
    pub attack: f32,
    /// Decay duration in seconds.
    pub decay: f32,
    /// Sustain level (0.0 to 1.0).
    pub sustain: f32,
    /// Release duration in seconds.
    pub release: f32,
    /// Current state of envelope generator.
    pub state: EnvState,
    /// Current envelope level (0.0 to 1.0).
    pub level: f32,
    /// Gate trigger signal.
    pub gate: bool,
}

impl EnvADSR {
    /// Create a new ADSR envelope generator.
    pub fn new(attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        Self {
            attack: attack.max(0.001),
            decay: decay.max(0.001),
            sustain: sustain.clamp(0.0, 1.0),
            release: release.max(0.001),
            state: EnvState::Idle,
            level: 0.0,
            gate: false,
        }
    }

    /// Trigger envelope with gate signal state (on/off).
    pub fn trigger(&mut self, gate: bool) {
        if gate && !self.gate {
            self.state = EnvState::Attack;
        } else if !gate && self.gate {
            self.state = EnvState::Release;
        }
        self.gate = gate;
    }

    /// Process a single audio sample step and return current envelope level.
    pub fn process_sample(&mut self, sample_rate: u32) -> f32 {
        if sample_rate == 0 {
            return self.level;
        }

        let dt = 1.0 / sample_rate as f32;

        match self.state {
            EnvState::Idle => {
                self.level = 0.0;
            }
            EnvState::Attack => {
                self.level += dt / self.attack;
                if self.level >= 1.0 {
                    self.level = 1.0;
                    self.state = EnvState::Decay;
                }
            }
            EnvState::Decay => {
                self.level -= dt / self.decay * (1.0 - self.sustain);
                if self.level <= self.sustain {
                    self.level = self.sustain;
                    self.state = EnvState::Sustain;
                }
            }
            EnvState::Sustain => {
                self.level = self.sustain;
            }
            EnvState::Release => {
                self.level -= dt / self.release;
                if self.level <= 0.0 {
                    self.level = 0.0;
                    self.state = EnvState::Idle;
                }
            }
        }

        self.level.clamp(0.0, 1.0)
    }
}

impl SignalProcessor for EnvADSR {
    fn name(&self) -> &str {
        "EnvADSR"
    }

    fn process_block(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        if ctx.sample_rate == 0 || outputs.is_empty() {
            return;
        }

        let num_samples = outputs[0].len();
        for i in 0..num_samples {
            if !inputs.is_empty() && !inputs[0].is_empty() && i < inputs[0].len() {
                self.trigger(inputs[0][i] > 0.5);
            }

            let env_val = self.process_sample(ctx.sample_rate);
            for out_ch in outputs.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = env_val;
                }
            }
        }
    }
}

/// LFO waveform type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LfoShape {
    /// Sine wave LFO.
    Sine,
    /// Triangle wave LFO.
    Triangle,
    /// Square wave LFO.
    Square,
    /// Sample and hold random stepped LFO.
    SampleAndHold,
}

fn default_lfo_seed() -> u64 {
    0x9E3779B97F4A7C15
}

/// Low-Frequency Oscillator (LFO).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LFO {
    /// LFO frequency in Hz.
    pub frequency: f32,
    /// LFO shape.
    pub shape: LfoShape,
    /// Phase in range 0.0 to 1.0.
    pub phase: f32,
    #[serde(skip)]
    sh_value: f32,
    #[serde(skip, default = "default_lfo_seed")]
    seed: u64,
}

impl LFO {
    /// Create a new LFO with target frequency and waveform shape.
    pub fn new(frequency: f32, shape: LfoShape) -> Self {
        Self {
            frequency,
            shape,
            phase: 0.0,
            sh_value: 0.0,
            seed: 0x9E3779B97F4A7C15,
        }
    }

    /// Process a single sample step and return normalized LFO output value (-1.0 to 1.0).
    pub fn process_sample(&mut self, sample_rate: u32) -> f32 {
        if sample_rate == 0 {
            return 0.0;
        }

        let dt = self.frequency / sample_rate as f32;
        let prev_phase = self.phase;
        self.phase = (self.phase + dt) % 1.0;

        match self.shape {
            LfoShape::Sine => (self.phase * TAU).sin(),
            LfoShape::Triangle => {
                2.0 * (2.0 * (self.phase - (self.phase + 0.5).floor())).abs() - 1.0
            }
            LfoShape::Square => {
                if self.phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            LfoShape::SampleAndHold => {
                if self.phase < prev_phase {
                    self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                    self.sh_value = ((self.seed >> 33) as f32 / 2147483648.0) - 1.0;
                }
                self.sh_value
            }
        }
    }
}

impl SignalProcessor for LFO {
    fn name(&self) -> &str {
        "LFO"
    }

    fn process_block(
        &mut self,
        _inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        if ctx.sample_rate == 0 || outputs.is_empty() {
            return;
        }

        let num_samples = outputs[0].len();
        for i in 0..num_samples {
            let lfo_val = self.process_sample(ctx.sample_rate);
            for out_ch in outputs.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = lfo_val;
                }
            }
        }
    }
}

/// MacroKnob bridging GUI/Automation parameters to DSP modulation inputs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroKnob {
    /// Static macro parameter value (0.0 to 1.0).
    pub value: f32,
    /// Atomic binding to real-time parameter bus.
    #[serde(skip)]
    pub atomic_binding: Option<Arc<AtomicU32>>,
}

impl MacroKnob {
    /// Create a new MacroKnob with initial normalized value.
    pub fn new(value: f32) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            atomic_binding: None,
        }
    }

    /// Bind an atomic float container for real-time automation updates.
    pub fn bind_atomic(&mut self, binding: Arc<AtomicU32>) {
        self.atomic_binding = Some(binding);
    }

    /// Get current macro value (from atomic binding if attached, else static value).
    pub fn get_value(&self) -> f32 {
        if let Some(ref binding) = self.atomic_binding {
            f32::from_bits(binding.load(Ordering::Relaxed))
        } else {
            self.value
        }
    }
}

impl SignalProcessor for MacroKnob {
    fn name(&self) -> &str {
        "MacroKnob"
    }

    fn process_block(
        &mut self,
        _inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        _ctx: &ProcessContext,
    ) {
        if outputs.is_empty() {
            return;
        }
        let current_val = self.get_value();
        let num_samples = outputs[0].len();
        for i in 0..num_samples {
            for out_ch in outputs.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = current_val;
                }
            }
        }
    }
}

/// Modulation source identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModulationSourceId {
    /// Macro knob index.
    Macro(usize),
    /// LFO index.
    Lfo(usize),
    /// Envelope index.
    Envelope(usize),
    /// MIDI velocity (0.0 to 1.0).
    Velocity,
    /// MIDI Mod Wheel (0.0 to 1.0).
    ModWheel,
    /// MIDI Pitch Bend (-1.0 to 1.0).
    PitchBend,
}

/// Modulation target parameter identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModulationTargetId(pub usize);

/// Target parameter definition in the modulation matrix.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulationTarget {
    /// Target parameter ID.
    pub id: ModulationTargetId,
    /// Human readable parameter name.
    pub name: String,
    /// Unmodulated base parameter value.
    pub base_value: f32,
    /// Minimum allowed value.
    pub min_value: f32,
    /// Maximum allowed value.
    pub max_value: f32,
    /// Evaluated modulated value after matrix routing calculation.
    pub current_modulated_value: f32,
}

/// Curve transformation applied to modulation source before routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModulationCurve {
    /// Linear scaling.
    Linear,
    /// Inverted scaling.
    Inverted,
    /// Exponential response curve (squared).
    Exponential,
    /// Logarithmic response curve (square root).
    Logarithmic,
    /// Cubic smoothstep response curve.
    SmoothStep,
}

impl ModulationCurve {
    /// Apply the transformation curve to input value `x`.
    pub fn apply(&self, x: f32) -> f32 {
        let clamped = x.clamp(-1.0, 1.0);
        match self {
            ModulationCurve::Linear => clamped,
            ModulationCurve::Inverted => -clamped,
            ModulationCurve::Exponential => clamped.signum() * (clamped.abs().powf(2.0)),
            ModulationCurve::Logarithmic => clamped.signum() * (clamped.abs().sqrt()),
            ModulationCurve::SmoothStep => {
                let s = clamped.signum();
                let a = clamped.abs();
                s * (a * a * (3.0 - 2.0 * a))
            }
        }
    }
}

/// A single modulation matrix connection / routing assignment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulationAssignment {
    /// Source of modulation.
    pub source: ModulationSourceId,
    /// Target parameter to modulate.
    pub target: ModulationTargetId,
    /// Modulation depth/amount (-1.0 to 1.0).
    pub amount: f32,
    /// Bipolar routing flag (-1..1 vs 0..1).
    pub bipolar: bool,
    /// Response curve transformation.
    pub curve: ModulationCurve,
    /// Enabled flag.
    pub enabled: bool,
}

/// Macro parameter mapping matrix with LFO/Envelope modulation assignments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroModulationMatrix {
    /// Name of the modulation matrix setup or preset.
    pub name: String,
    /// Registered macro knobs.
    pub macros: Vec<MacroKnob>,
    /// Registered LFO modulators.
    pub lfos: Vec<LFO>,
    /// Registered Envelope generators.
    pub envelopes: Vec<EnvADSR>,
    /// Target parameters registered in matrix.
    pub targets: Vec<ModulationTarget>,
    /// Modulation routing assignments.
    pub assignments: Vec<ModulationAssignment>,
    /// Real-time MIDI velocity control input (0.0 to 1.0).
    pub velocity: f32,
    /// Real-time MIDI mod wheel control input (0.0 to 1.0).
    pub mod_wheel: f32,
    /// Real-time MIDI pitch bend control input (-1.0 to 1.0).
    pub pitch_bend: f32,
}

impl MacroModulationMatrix {
    /// Create a new macro modulation matrix with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            macros: Vec::new(),
            lfos: Vec::new(),
            envelopes: Vec::new(),
            targets: Vec::new(),
            assignments: Vec::new(),
            velocity: 1.0,
            mod_wheel: 0.0,
            pitch_bend: 0.0,
        }
    }

    /// Add a macro knob source and return its ModulationSourceId.
    pub fn add_macro(&mut self, initial_value: f32) -> ModulationSourceId {
        let idx = self.macros.len();
        self.macros.push(MacroKnob::new(initial_value));
        ModulationSourceId::Macro(idx)
    }

    /// Add an LFO modulator source and return its ModulationSourceId.
    pub fn add_lfo(&mut self, frequency: f32, shape: LfoShape) -> ModulationSourceId {
        let idx = self.lfos.len();
        self.lfos.push(LFO::new(frequency, shape));
        ModulationSourceId::Lfo(idx)
    }

    /// Add an Envelope generator source and return its ModulationSourceId.
    pub fn add_envelope(
        &mut self,
        attack: f32,
        decay: f32,
        sustain: f32,
        release: f32,
    ) -> ModulationSourceId {
        let idx = self.envelopes.len();
        self.envelopes
            .push(EnvADSR::new(attack, decay, sustain, release));
        ModulationSourceId::Envelope(idx)
    }

    /// Add a target parameter to the matrix and return its ModulationTargetId.
    pub fn add_target(
        &mut self,
        name: impl Into<String>,
        base_value: f32,
        min_val: f32,
        max_val: f32,
    ) -> ModulationTargetId {
        let id = ModulationTargetId(self.targets.len());
        self.targets.push(ModulationTarget {
            id,
            name: name.into(),
            base_value: base_value.clamp(min_val, max_val),
            min_value: min_val,
            max_value: max_val,
            current_modulated_value: base_value.clamp(min_val, max_val),
        });
        id
    }

    /// Add a modulation assignment route to the matrix. Returns the assignment index.
    pub fn add_assignment(
        &mut self,
        source: ModulationSourceId,
        target: ModulationTargetId,
        amount: f32,
        bipolar: bool,
        curve: ModulationCurve,
    ) -> usize {
        let idx = self.assignments.len();
        self.assignments.push(ModulationAssignment {
            source,
            target,
            amount: amount.clamp(-1.0, 1.0),
            bipolar,
            curve,
            enabled: true,
        });
        idx
    }

    /// Remove a modulation assignment by index. Returns true if removed.
    pub fn remove_assignment(&mut self, index: usize) -> bool {
        if index < self.assignments.len() {
            self.assignments.remove(index);
            true
        } else {
            false
        }
    }

    /// Update modulation assignment amount depth.
    pub fn set_assignment_amount(&mut self, index: usize, amount: f32) {
        if let Some(assignment) = self.assignments.get_mut(index) {
            assignment.amount = amount.clamp(-1.0, 1.0);
        }
    }

    /// Enable or disable a modulation assignment.
    pub fn set_assignment_enabled(&mut self, index: usize, enabled: bool) {
        if let Some(assignment) = self.assignments.get_mut(index) {
            assignment.enabled = enabled;
        }
    }

    /// Update static macro knob value.
    pub fn set_macro_value(&mut self, macro_idx: usize, value: f32) {
        if let Some(knob) = self.macros.get_mut(macro_idx) {
            knob.value = value.clamp(0.0, 1.0);
        }
    }

    /// Trigger ADSR envelope generator by index.
    pub fn trigger_envelope(&mut self, env_idx: usize, gate: bool) {
        if let Some(env) = self.envelopes.get_mut(env_idx) {
            env.trigger(gate);
        }
    }

    /// Update MIDI velocity control input (0.0 to 1.0).
    pub fn set_velocity(&mut self, velocity: f32) {
        self.velocity = velocity.clamp(0.0, 1.0);
    }

    /// Update MIDI mod wheel control input (0.0 to 1.0).
    pub fn set_mod_wheel(&mut self, mod_wheel: f32) {
        self.mod_wheel = mod_wheel.clamp(0.0, 1.0);
    }

    /// Update MIDI pitch bend control input (-1.0 to 1.0).
    pub fn set_pitch_bend(&mut self, pitch_bend: f32) {
        self.pitch_bend = pitch_bend.clamp(-1.0, 1.0);
    }

    /// Retrieve the current modulated value for a target parameter ID.
    pub fn get_modulated_value(&self, target_id: ModulationTargetId) -> Option<f32> {
        self.targets
            .iter()
            .find(|t| t.id == target_id)
            .map(|t| t.current_modulated_value)
    }

    /// Process sample step: advances all internal LFOs and Envelopes,
    /// evaluates all modulation assignments, and updates target parameter values.
    pub fn process_sample(&mut self, sample_rate: u32) {
        let lfo_vals: Vec<f32> = self
            .lfos
            .iter_mut()
            .map(|lfo| lfo.process_sample(sample_rate))
            .collect();
        let env_vals: Vec<f32> = self
            .envelopes
            .iter_mut()
            .map(|env| env.process_sample(sample_rate))
            .collect();
        let macro_vals: Vec<f32> = self.macros.iter().map(|m| m.get_value()).collect();

        for target in self.targets.iter_mut() {
            let range = target.max_value - target.min_value;
            let mut total_offset = 0.0f32;

            for assignment in &self.assignments {
                if !assignment.enabled || assignment.target != target.id {
                    continue;
                }

                let raw_val = match assignment.source {
                    ModulationSourceId::Macro(idx) => macro_vals.get(idx).copied().unwrap_or(0.0),
                    ModulationSourceId::Lfo(idx) => lfo_vals.get(idx).copied().unwrap_or(0.0),
                    ModulationSourceId::Envelope(idx) => env_vals.get(idx).copied().unwrap_or(0.0),
                    ModulationSourceId::Velocity => self.velocity,
                    ModulationSourceId::ModWheel => self.mod_wheel,
                    ModulationSourceId::PitchBend => self.pitch_bend,
                };

                let curved_val = assignment.curve.apply(raw_val);
                let normalized_contrib = if assignment.bipolar {
                    curved_val * assignment.amount
                } else {
                    ((curved_val + 1.0) * 0.5) * assignment.amount
                };

                total_offset += normalized_contrib * range;
            }

            target.current_modulated_value =
                (target.base_value + total_offset).clamp(target.min_value, target.max_value);
        }
    }
}

impl Default for MacroModulationMatrix {
    fn default() -> Self {
        Self::new("Default Matrix")
    }
}

impl SignalProcessor for MacroModulationMatrix {
    fn name(&self) -> &str {
        "MacroModulationMatrix"
    }

    fn process_block(
        &mut self,
        _inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        if ctx.sample_rate == 0 || outputs.is_empty() {
            return;
        }

        let num_samples = outputs[0].len();
        for i in 0..num_samples {
            self.process_sample(ctx.sample_rate);
            for (ch_idx, target) in self.targets.iter().enumerate() {
                if ch_idx < outputs.len() && i < outputs[ch_idx].len() {
                    outputs[ch_idx][i] = target.current_modulated_value;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_modulation_matrix_lfo_env_routing() {
        let mut matrix = MacroModulationMatrix::new("Test Matrix");

        let macro_src = matrix.add_macro(0.5);
        let lfo_src = matrix.add_lfo(5.0, LfoShape::Sine);
        let env_src = matrix.add_envelope(0.01, 0.1, 0.5, 0.2);

        let filter_cutoff = matrix.add_target("Filter Cutoff", 1000.0, 20.0, 20000.0);
        let amp_gain = matrix.add_target("Amplifier Gain", 0.5, 0.0, 1.0);

        matrix.add_assignment(
            macro_src,
            filter_cutoff,
            0.5,
            true,
            ModulationCurve::Linear,
        );
        matrix.add_assignment(
            lfo_src,
            filter_cutoff,
            0.2,
            true,
            ModulationCurve::Exponential,
        );
        matrix.add_assignment(env_src, amp_gain, 0.4, false, ModulationCurve::SmoothStep);

        matrix.trigger_envelope(0, true);

        for _ in 0..100 {
            matrix.process_sample(44100);
        }

        let modulated_cutoff = matrix.get_modulated_value(filter_cutoff).unwrap();
        let modulated_gain = matrix.get_modulated_value(amp_gain).unwrap();

        assert!(
            (20.0..=20000.0).contains(&modulated_cutoff),
            "Modulated cutoff out of bounds: {}",
            modulated_cutoff
        );
        assert!(
            (0.0..=1.0).contains(&modulated_gain),
            "Modulated gain out of bounds: {}",
            modulated_gain
        );
    }

    #[test]
    fn test_macro_modulation_matrix_serde() {
        let mut matrix = MacroModulationMatrix::new("Serde Matrix");
        let macro_src = matrix.add_macro(0.8);
        let target_id = matrix.add_target("Resonance", 0.2, 0.0, 1.0);
        matrix.add_assignment(macro_src, target_id, 0.3, true, ModulationCurve::Linear);

        let json = serde_json::to_string(&matrix).expect("Serialization failed");
        let deserialized: MacroModulationMatrix =
            serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(deserialized.name, "Serde Matrix");
        assert_eq!(deserialized.targets.len(), 1);
        assert_eq!(deserialized.assignments.len(), 1);
    }
}
