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

//! Polish for the 4 Daily Driver Stock Instruments:
//! 1. Subtractive Synth (`DailySubtractiveSynth`)
//! 2. Wavetable Synth (`DailyWavetableSynth`)
//! 3. FM Synth (`DailyFmSynth`)
//! 4. Sampler (`DailySampler`)
//!
//! Featuring 8 standardized Macro Knobs and curated factory presets.

use crate::distortion::{DistortionNode, DistortionType};
use crate::filters::{FilterLadder, FilterSVF};
use crate::modulators::{EnvADSR, LfoShape, LFO};
use crate::oscillators::{OscPulse, OscSaw, OscSine, OscWavetable, SimdPolyWavetableOscillator};
use crate::sampler::{MultiSampleBank, MultiSamplerNode};
use crate::traits::SignalProcessor;
use serde::{Deserialize, Serialize};
use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;

/// Normalized Macro Knob definition for stock instruments.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MacroKnobConfig {
    pub name: String,
    pub value: f32, // 0.0 to 1.0 normalized
    pub min: f32,
    pub max: f32,
    pub unit: String,
}

impl MacroKnobConfig {
    pub fn new(
        name: impl Into<String>,
        value: f32,
        min: f32,
        max: f32,
        unit: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            value: value.clamp(0.0, 1.0),
            min,
            max,
            unit: unit.into(),
        }
    }

    pub fn real_value(&self) -> f32 {
        self.min + self.value * (self.max - self.min)
    }

    pub fn set_normalized(&mut self, val: f32) {
        self.value = val.clamp(0.0, 1.0);
    }
}

/// 8-Macro Controller container standard across all Summoner instruments.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Macro8Controller {
    pub knobs: [MacroKnobConfig; 8],
}

impl Macro8Controller {
    pub fn new(knob_names: [&str; 8], min_max_units: [(f32, f32, &str); 8]) -> Self {
        let mut knobs = std::array::from_fn(|i| {
            let (min, max, unit) = min_max_units[i];
            MacroKnobConfig::new(knob_names[i], 0.5, min, max, unit)
        });
        // Default reasonable initial settings
        knobs[0].set_normalized(0.75); // Cutoff
        knobs[1].set_normalized(0.20); // Resonance
        Self { knobs }
    }

    pub fn set_macro(&mut self, idx: usize, val: f32) {
        if idx < 8 {
            self.knobs[idx].set_normalized(val);
        }
    }

    pub fn get_macro(&self, idx: usize) -> f32 {
        if idx < 8 {
            self.knobs[idx].value
        } else {
            0.0
        }
    }

    pub fn get_real(&self, idx: usize) -> f32 {
        if idx < 8 {
            self.knobs[idx].real_value()
        } else {
            0.0
        }
    }
}

// =========================================================================
// 1. Subtractive Synth
// =========================================================================

/// Daily Driver Subtractive Synthesizer with 8 Macro Knobs and curated presets.
pub struct DailySubtractiveSynth {
    pub osc_saw: OscSaw,
    pub osc_pulse: OscPulse,
    pub osc_sub: OscSine,
    pub filter: FilterLadder,
    pub wavefolder: DistortionNode,
    pub amp_env: EnvADSR,
    pub filter_env: EnvADSR,
    pub lfo: LFO,
    pub macros: Macro8Controller,
    pub current_preset: String,
}

impl DailySubtractiveSynth {
    pub fn new() -> Self {
        let macros = Macro8Controller::new(
            [
                "Cutoff",
                "Resonance",
                "Drive",
                "Sub Level",
                "Filter Env",
                "LFO Wobble",
                "Attack",
                "Release",
            ],
            [
                (20.0, 20000.0, "Hz"),
                (0.0, 4.0, ""),
                (0.0, 8.0, "x"),
                (0.0, 1.0, ""),
                (-1.0, 1.0, ""),
                (0.1, 25.0, "Hz"),
                (0.001, 2.0, "s"),
                (0.01, 5.0, "s"),
            ],
        );

        let mut synth = Self {
            osc_saw: OscSaw::new(110.0),
            osc_pulse: OscPulse::new(110.0, 0.5),
            osc_sub: OscSine::new(55.0),
            filter: FilterLadder::new(2000.0, 0.5),
            wavefolder: DistortionNode::new(DistortionType::Wavefolder, 1.0),
            amp_env: EnvADSR::new(0.005, 0.2, 0.8, 0.3),
            filter_env: EnvADSR::new(0.01, 0.3, 0.2, 0.4),
            lfo: LFO::new(3.0, LfoShape::Sine),
            macros,
            current_preset: "Init Patch".to_string(),
        };
        synth.load_preset("Cyberpunk Sub-Bass");
        synth
    }

    pub fn load_preset(&mut self, name: &str) {
        self.current_preset = name.to_string();
        match name {
            "Cyberpunk Sub-Bass" => {
                self.macros.set_macro(0, 0.35); // 500 Hz cutoff
                self.macros.set_macro(1, 0.30); // moderate res
                self.macros.set_macro(2, 0.60); // heavy wavefold
                self.macros.set_macro(3, 0.90); // loud sub
                self.macros.set_macro(4, 0.40); // env mod
                self.macros.set_macro(5, 0.25); // wobble
                self.macros.set_macro(6, 0.01); // fast attack
                self.macros.set_macro(7, 0.20); // punchy release
            }
            "Acid 303 Resonance" => {
                self.macros.set_macro(0, 0.25);
                self.macros.set_macro(1, 0.85); // screaming resonance
                self.macros.set_macro(2, 0.40);
                self.macros.set_macro(3, 0.10);
                self.macros.set_macro(4, 0.80); // high filter env
                self.macros.set_macro(5, 0.0);
                self.macros.set_macro(6, 0.005);
                self.macros.set_macro(7, 0.15);
            }
            "Warm Brass Lead" => {
                self.macros.set_macro(0, 0.65);
                self.macros.set_macro(1, 0.15);
                self.macros.set_macro(2, 0.10);
                self.macros.set_macro(3, 0.20);
                self.macros.set_macro(4, 0.50);
                self.macros.set_macro(5, 0.15);
                self.macros.set_macro(6, 0.08); // soft brass attack
                self.macros.set_macro(7, 0.40);
            }
            _ => {}
        }
        self.sync_dsp_parameters();
    }

    fn sync_dsp_parameters(&mut self) {
        self.filter.cutoff = self.macros.get_real(0);
        self.filter.resonance = self.macros.get_real(1);
        self.wavefolder.drive = self.macros.get_real(2);
        self.lfo.frequency = self.macros.get_real(5);
        self.amp_env.attack = self.macros.get_real(6);
        self.amp_env.release = self.macros.get_real(7);
    }
}

impl Default for DailySubtractiveSynth {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalProcessor for DailySubtractiveSynth {
    fn name(&self) -> &str {
        "DailySubtractiveSynth"
    }

    fn handle_event(&mut self, event: &summoner_core::node::NodeEvent, ctx: &ProcessContext) {
        use summoner_core::node::NodeEvent;
        match *event {
            NodeEvent::NoteOn { note, .. } => {
                let freq = ctx.note_to_hz(note as i32);
                self.osc_saw.frequency = freq;
                self.osc_pulse.frequency = freq;
                self.osc_sub.frequency = freq * 0.5;
                self.amp_env.trigger(true);
                self.filter_env.trigger(true);
            }
            NodeEvent::NoteOff { .. } => {
                self.amp_env.trigger(false);
                self.filter_env.trigger(false);
            }
            NodeEvent::SetParam { id, value } => {
                if id < 8 {
                    self.macros.set_macro(id as usize, value);
                    self.sync_dsp_parameters();
                }
            }
        }
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
        let sub_level = self.macros.get_real(3);
        let env_amount = self.macros.get_real(4);
        let base_cutoff = self.macros.get_real(0);

        for i in 0..num_samples {
            let lfo_val = self.lfo.process_sample(ctx.sample_rate);
            let a_env = self.amp_env.process_sample(ctx.sample_rate);
            let f_env = self.filter_env.process_sample(ctx.sample_rate);

            // Modulate filter cutoff with envelope and LFO
            let mod_cutoff =
                (base_cutoff * (1.0 + f_env * env_amount + lfo_val * 0.2)).clamp(20.0, 20000.0);
            self.filter.cutoff = mod_cutoff;

            let s_saw = self.osc_saw.process_sample(ctx.sample_rate);
            let s_pulse = self.osc_pulse.process_sample(ctx.sample_rate);
            let s_sub = self.osc_sub.process_sample(ctx.sample_rate, 0.0) * sub_level;

            let osc_mix = (s_saw * 0.5 + s_pulse * 0.5 + s_sub) * 0.5;

            // Wavefolder distortion
            let folded = self.wavefolder.process_sample(osc_mix);

            // Ladder filter
            let filtered = self.filter.process_sample(folded, ctx.sample_rate);

            let out_sample = filtered * a_env;

            for ch in outputs.iter_mut() {
                if i < ch.len() {
                    ch[i] = out_sample;
                }
            }
        }
    }
}

// =========================================================================
// 2. Wavetable Synth
// =========================================================================

/// Daily Driver Wavetable Synthesizer with Morphing & Macro Controls.
pub struct DailyWavetableSynth {
    pub osc: SimdPolyWavetableOscillator,
    pub filter: FilterSVF,
    pub amp_env: EnvADSR,
    pub macros: Macro8Controller,
    pub current_preset: String,
}

impl DailyWavetableSynth {
    pub fn new(sample_rate: u32) -> Self {
        let macros = Macro8Controller::new(
            [
                "Table Position",
                "Morph / Warp",
                "Cutoff",
                "Resonance",
                "Detune Spread",
                "Drive",
                "Attack",
                "Release",
            ],
            [
                (0.0, 1.0, ""),
                (0.0, 1.0, ""),
                (20.0, 20000.0, "Hz"),
                (0.0, 4.0, ""),
                (0.0, 1.0, ""),
                (1.0, 8.0, "x"),
                (0.001, 3.0, "s"),
                (0.01, 5.0, "s"),
            ],
        );

        let saw = OscWavetable::default_saw();
        let square = OscWavetable::default_square();
        let osc = SimdPolyWavetableOscillator::new(sample_rate)
            .with_table(saw)
            .with_table2(square, 0.5);

        let mut synth = Self {
            osc,
            filter: FilterSVF::new(3000.0, 1.0),
            amp_env: EnvADSR::new(0.01, 0.2, 0.8, 0.3),
            macros,
            current_preset: "Digital Hypersaw".to_string(),
        };
        synth.load_preset("Digital Hypersaw");
        synth
    }

    pub fn load_preset(&mut self, name: &str) {
        self.current_preset = name.to_string();
        match name {
            "Digital Hypersaw" => {
                self.macros.set_macro(0, 0.20);
                self.macros.set_macro(1, 0.10);
                self.macros.set_macro(2, 0.85); // bright
                self.macros.set_macro(3, 0.25);
                self.macros.set_macro(4, 0.70); // wide detune
                self.macros.set_macro(5, 0.30);
                self.macros.set_macro(6, 0.005);
                self.macros.set_macro(7, 0.30);
            }
            "Crystal Glass Bells" => {
                self.macros.set_macro(0, 0.80);
                self.macros.set_macro(1, 0.90);
                self.macros.set_macro(2, 0.60);
                self.macros.set_macro(3, 0.60);
                self.macros.set_macro(4, 0.10);
                self.macros.set_macro(5, 0.0);
                self.macros.set_macro(6, 0.001); // bell transient
                self.macros.set_macro(7, 0.80); // long bell tail
            }
            _ => {}
        }
        self.sync_dsp_parameters();
    }

    fn sync_dsp_parameters(&mut self) {
        let morph = self.macros.get_real(1);
        self.osc.set_morph(morph);
        self.filter.cutoff = self.macros.get_real(2);
        self.filter.resonance = self.macros.get_real(3);
        self.amp_env.attack = self.macros.get_real(6);
        self.amp_env.release = self.macros.get_real(7);
    }
}

impl SignalProcessor for DailyWavetableSynth {
    fn name(&self) -> &str {
        "DailyWavetableSynth"
    }

    fn handle_event(&mut self, event: &summoner_core::node::NodeEvent, _ctx: &ProcessContext) {
        use summoner_core::node::NodeEvent;
        match *event {
            NodeEvent::NoteOn { note, velocity } => {
                let vel = velocity.clamp(0.0, 1.0);
                self.osc.note_on(note, vel);
                self.amp_env.trigger(true);
            }
            NodeEvent::NoteOff { note } => {
                self.osc.note_off(note);
                self.amp_env.trigger(false);
            }
            NodeEvent::SetParam { id, value } => {
                if id < 8 {
                    self.macros.set_macro(id as usize, value);
                    self.sync_dsp_parameters();
                }
            }
        }
    }

    fn process_block(
        &mut self,
        _inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        if outputs.is_empty() {
            return;
        }

        self.osc.process_block(&[], outputs, ctx);

        let num_samples = outputs[0].len();
        let drive = self.macros.get_real(5);

        for i in 0..num_samples {
            let env = self.amp_env.process_sample(ctx.sample_rate);
            let s = outputs[0][i] * drive;
            let (filtered, _, _) = self.filter.process_sample(s, ctx.sample_rate);
            let out = filtered * env;

            for ch in outputs.iter_mut() {
                if i < ch.len() {
                    ch[i] = out;
                }
            }
        }
    }
}

// =========================================================================
// 3. FM Synthesizer
// =========================================================================

/// Daily Driver 4-Operator FM Synthesizer with 8 Macro Controls and Classic DX Presets.
pub struct DailyFmSynth {
    pub op1: OscSine, // Carrier
    pub op2: OscSine, // Modulator 1
    pub op3: OscSine, // Modulator 2
    pub op4: OscSine, // Modulator 3
    pub amp_env: EnvADSR,
    pub mod_env: EnvADSR,
    pub macros: Macro8Controller,
    pub current_preset: String,
    base_freq: f32,
}

impl DailyFmSynth {
    pub fn new() -> Self {
        let macros = Macro8Controller::new(
            [
                "FM Depth",
                "Harmonic Ratio",
                "Op 2 Level",
                "Op 3 Level",
                "Feedback",
                "Brightness",
                "Decay",
                "Release",
            ],
            [
                (0.0, 10.0, ""),
                (1.0, 16.0, "x"),
                (0.0, 1.0, ""),
                (0.0, 1.0, ""),
                (0.0, 2.0, ""),
                (20.0, 20000.0, "Hz"),
                (0.01, 5.0, "s"),
                (0.01, 5.0, "s"),
            ],
        );

        let mut synth = Self {
            op1: OscSine::new(220.0),
            op2: OscSine::new(220.0),
            op3: OscSine::new(440.0),
            op4: OscSine::new(660.0),
            amp_env: EnvADSR::new(0.005, 0.4, 0.6, 0.3),
            mod_env: EnvADSR::new(0.005, 0.2, 0.2, 0.3),
            macros,
            current_preset: "Solid DX Bass".to_string(),
            base_freq: 220.0,
        };
        synth.load_preset("Solid DX Bass");
        synth
    }

    pub fn load_preset(&mut self, name: &str) {
        self.current_preset = name.to_string();
        match name {
            "Solid DX Bass" => {
                self.macros.set_macro(0, 0.60); // punchy FM depth
                self.macros.set_macro(1, 0.0); // 1.0 ratio
                self.macros.set_macro(2, 0.80);
                self.macros.set_macro(3, 0.30);
                self.macros.set_macro(4, 0.20);
                self.macros.set_macro(5, 0.70);
                self.macros.set_macro(6, 0.15); // quick snappy decay
                self.macros.set_macro(7, 0.10);
            }
            "Electric Tine Piano" => {
                self.macros.set_macro(0, 0.45);
                self.macros.set_macro(1, 0.15); // ~3x ratio bell
                self.macros.set_macro(2, 0.60);
                self.macros.set_macro(3, 0.70);
                self.macros.set_macro(4, 0.05);
                self.macros.set_macro(5, 0.85);
                self.macros.set_macro(6, 0.50);
                self.macros.set_macro(7, 0.40);
            }
            _ => {}
        }
        self.sync_dsp_parameters();
    }

    fn sync_dsp_parameters(&mut self) {
        let ratio = self.macros.get_real(1);
        self.op2.frequency = self.base_freq * ratio;
        self.op3.frequency = self.base_freq * (ratio * 2.0);
        self.op4.frequency = self.base_freq * (ratio * 3.0);
        self.amp_env.decay = self.macros.get_real(6);
        self.amp_env.release = self.macros.get_real(7);
    }
}

impl Default for DailyFmSynth {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalProcessor for DailyFmSynth {
    fn name(&self) -> &str {
        "DailyFmSynth"
    }

    fn handle_event(&mut self, event: &summoner_core::node::NodeEvent, ctx: &ProcessContext) {
        use summoner_core::node::NodeEvent;
        match *event {
            NodeEvent::NoteOn { note, .. } => {
                self.base_freq = ctx.note_to_hz(note as i32);
                self.op1.frequency = self.base_freq;
                self.sync_dsp_parameters();
                self.amp_env.trigger(true);
                self.mod_env.trigger(true);
            }
            NodeEvent::NoteOff { .. } => {
                self.amp_env.trigger(false);
                self.mod_env.trigger(false);
            }
            NodeEvent::SetParam { id, value } => {
                if id < 8 {
                    self.macros.set_macro(id as usize, value);
                    self.sync_dsp_parameters();
                }
            }
        }
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
        let depth = self.macros.get_real(0);
        let op2_lvl = self.macros.get_real(2);
        let op3_lvl = self.macros.get_real(3);

        for i in 0..num_samples {
            let m_env = self.mod_env.process_sample(ctx.sample_rate);
            let a_env = self.amp_env.process_sample(ctx.sample_rate);

            // Modulators
            let mod3 = self.op4.process_sample(ctx.sample_rate, 0.0);
            let mod2 = self.op3.process_sample(ctx.sample_rate, mod3 * 0.5) * op3_lvl;
            let mod1 = self.op2.process_sample(ctx.sample_rate, mod2) * op2_lvl;

            // Carrier modulated by mod1
            let fm_input = mod1 * depth * m_env;
            let carrier = self.op1.process_sample(ctx.sample_rate, fm_input);

            let out_sample = carrier * a_env;

            for ch in outputs.iter_mut() {
                if i < ch.len() {
                    ch[i] = out_sample;
                }
            }
        }
    }
}

// =========================================================================
// 4. Sampler
// =========================================================================

/// Daily Driver Sampler with 8 Macro Knobs and MultiSample playback.
pub struct DailySampler {
    pub sampler: MultiSamplerNode,
    pub filter: FilterLadder,
    pub amp_env: EnvADSR,
    pub macros: Macro8Controller,
    pub current_preset: String,
}

impl DailySampler {
    pub fn new(bank: MultiSampleBank) -> Self {
        let macros = Macro8Controller::new(
            [
                "Cutoff",
                "Resonance",
                "Tune",
                "Drive",
                "Attack",
                "Decay",
                "Release",
                "Volume",
            ],
            [
                (20.0, 20000.0, "Hz"),
                (0.0, 4.0, ""),
                (-24.0, 24.0, "st"),
                (1.0, 6.0, "x"),
                (0.001, 3.0, "s"),
                (0.01, 5.0, "s"),
                (0.01, 5.0, "s"),
                (0.0, 1.5, ""),
            ],
        );

        Self {
            sampler: MultiSamplerNode::new(bank),
            filter: FilterLadder::new(18000.0, 0.5),
            amp_env: EnvADSR::new(0.005, 0.5, 0.8, 0.4),
            macros,
            current_preset: "Grand Piano".to_string(),
        }
    }

    pub fn load_preset(&mut self, name: &str) {
        self.current_preset = name.to_string();
        match name {
            "Grand Piano" => {
                self.macros.set_macro(0, 0.90);
                self.macros.set_macro(1, 0.10);
                self.macros.set_macro(2, 0.50); // 0 st tune
                self.macros.set_macro(3, 0.0); // clean
                self.macros.set_macro(4, 0.005);
                self.macros.set_macro(5, 0.50);
                self.macros.set_macro(6, 0.35);
                self.macros.set_macro(7, 0.80);
            }
            "Vintage Mellotron" => {
                self.macros.set_macro(0, 0.60); // warm low-pass
                self.macros.set_macro(1, 0.20);
                self.macros.set_macro(2, 0.50);
                self.macros.set_macro(3, 0.25); // subtle tape saturation
                self.macros.set_macro(4, 0.05); // slow tape attack
                self.macros.set_macro(5, 0.80);
                self.macros.set_macro(6, 0.50);
                self.macros.set_macro(7, 0.75);
            }
            _ => {}
        }
        self.sync_dsp_parameters();
    }

    fn sync_dsp_parameters(&mut self) {
        self.filter.cutoff = self.macros.get_real(0);
        self.filter.resonance = self.macros.get_real(1);
        self.amp_env.attack = self.macros.get_real(4);
        self.amp_env.decay = self.macros.get_real(5);
        self.amp_env.release = self.macros.get_real(6);
    }
}

impl SignalProcessor for DailySampler {
    fn name(&self) -> &str {
        "DailySampler"
    }

    fn handle_event(&mut self, event: &summoner_core::node::NodeEvent, _ctx: &ProcessContext) {
        use summoner_core::node::NodeEvent;
        match *event {
            NodeEvent::NoteOn { note, velocity } => {
                let vel_u8 = (velocity.clamp(0.0, 1.0) * 127.0).round() as u8;
                self.sampler.trigger_note(note, vel_u8);
                self.amp_env.trigger(true);
            }
            NodeEvent::NoteOff { .. } => {
                self.sampler.stop();
                self.amp_env.trigger(false);
            }
            NodeEvent::SetParam { id, value } => {
                if id < 8 {
                    self.macros.set_macro(id as usize, value);
                    self.sync_dsp_parameters();
                }
            }
        }
    }

    fn process_block(
        &mut self,
        _inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        if outputs.is_empty() {
            return;
        }

        self.sampler.process_block(&[], outputs, ctx);

        let num_samples = outputs[0].len();
        let volume = self.macros.get_real(7);
        let drive = self.macros.get_real(3);

        for i in 0..num_samples {
            let env = self.amp_env.process_sample(ctx.sample_rate);
            let raw = outputs[0][i] * drive;
            let filtered = self.filter.process_sample(raw, ctx.sample_rate);
            let out = filtered * env * volume;

            for ch in outputs.iter_mut() {
                if i < ch.len() {
                    ch[i] = out;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daily_subtractive_synth() {
        let mut synth = DailySubtractiveSynth::new();
        synth.load_preset("Acid 303 Resonance");
        assert_eq!(synth.current_preset, "Acid 303 Resonance");

        let ctx = ProcessContext::new(44100, 120.0, 0);
        synth.handle_event(
            &summoner_core::node::NodeEvent::NoteOn {
                note: 36,
                velocity: 0.9,
            },
            &ctx,
        );

        let mut out = vec![0.0f32; 128];
        synth.process_block(&[], &mut [&mut out[..]], &ctx);

        let energy: f32 = out.iter().map(|s| s * s).sum();
        assert!(energy > 0.01, "Subtractive synth must produce audio energy");
    }

    #[test]
    fn test_daily_fm_synth() {
        let mut synth = DailyFmSynth::new();
        synth.load_preset("Electric Tine Piano");
        assert_eq!(synth.current_preset, "Electric Tine Piano");

        let ctx = ProcessContext::new(44100, 120.0, 0);
        synth.handle_event(
            &summoner_core::node::NodeEvent::NoteOn {
                note: 60,
                velocity: 0.8,
            },
            &ctx,
        );

        let mut out = vec![0.0f32; 128];
        synth.process_block(&[], &mut [&mut out[..]], &ctx);

        let energy: f32 = out.iter().map(|s| s * s).sum();
        assert!(energy > 0.01, "FM synth must produce audio energy");
    }

    #[test]
    fn test_daily_wavetable_synth() {
        let mut synth = DailyWavetableSynth::new(44100);
        synth.load_preset("Crystal Glass Bells");
        assert_eq!(synth.current_preset, "Crystal Glass Bells");

        let ctx = ProcessContext::new(44100, 120.0, 0);
        synth.handle_event(
            &summoner_core::node::NodeEvent::NoteOn {
                note: 72,
                velocity: 1.0,
            },
            &ctx,
        );

        let mut out = vec![0.0f32; 128];
        synth.process_block(&[], &mut [&mut out[..]], &ctx);

        let energy: f32 = out.iter().map(|s| s * s).sum();
        assert!(energy > 0.01, "Wavetable synth must produce audio energy");
    }

    #[test]
    fn test_daily_sampler() {
        use crate::sampler::{LoopMode, SampleBuffer, SampleRegion};
        use std::sync::Arc;

        let mut bank = MultiSampleBank::new();
        let mut reg = SampleRegion::new(0, 127, 60, "piano_c4.wav");
        let dummy_data: Vec<f32> = (0..1000).map(|i| (i as f32 * 0.1).sin()).collect();
        reg.buffer = Some(Arc::new(SampleBuffer::new(dummy_data, 44100, 1)));
        reg.loop_mode = LoopMode::NoLoop;
        bank.add_region(reg);

        let mut sampler = DailySampler::new(bank);
        sampler.load_preset("Grand Piano");
        assert_eq!(sampler.current_preset, "Grand Piano");

        let ctx = ProcessContext::new(44100, 120.0, 0);
        sampler.handle_event(
            &summoner_core::node::NodeEvent::NoteOn {
                note: 60,
                velocity: 0.9,
            },
            &ctx,
        );

        let mut out = vec![0.0f32; 128];
        sampler.process_block(&[], &mut [&mut out[..]], &ctx);

        let energy: f32 = out.iter().map(|s| s * s).sum();
        assert!(energy > 0.01, "Daily sampler must produce audio energy");
    }
}
