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

//! Composite Sub-Graph Devices (Aether Synth, Pluck Physical Model, FM Operator Pair, GlitchAetherMachine, CyberpunkSubSynth, AtmosphericPadSynth, GlitchPercussionSynth).

use crate::distortion::{DistortionNode, DistortionType};
use crate::effects::{EffectDelay, EffectReverb};
use crate::filters::{FilterComb, FilterLadder, FilterSVF};
use crate::glitch::{GlitchGate, GlitchStutter, TapeStop};
use crate::math::{MathAdd, MathMult, VCA};
use crate::modulators::{EnvADSR, LfoShape, MacroKnob, LFO};
use crate::oscillators::{NoiseGen, NoiseType, OscPulse, OscSaw, OscSine, OscTriangle};
use crate::traits::SignalProcessor;
use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;

/// Macro View parameters for Aether Synth.
#[derive(Debug)]
pub struct AetherMacroView {
    pub osc_mix: MacroKnob,       // 0.0 (Saw) to 1.0 (Pulse)
    pub filter_cutoff: MacroKnob, // 20.0 to 20000.0 Hz
    pub filter_res: MacroKnob,    // 0.0 to 4.0
    pub lfo_speed: MacroKnob,     // Hz
}

/// Composite device: Aether Synth (2-Osc Subtractive + Dual ADSR + LFO PWM).
#[derive(Debug)]
pub struct AetherSynth {
    pub osc_saw: OscSaw,
    pub osc_pulse: OscPulse,
    pub mixer: MathAdd,
    pub filter: FilterLadder,
    pub vca: VCA,
    pub amp_env: EnvADSR,
    pub filter_env: EnvADSR,
    pub lfo: LFO,
    pub macro_view: AetherMacroView,
}

impl AetherSynth {
    pub fn new(frequency: f32) -> Self {
        Self {
            osc_saw: OscSaw::new(frequency),
            osc_pulse: OscPulse::new(frequency, 0.5),
            mixer: MathAdd,
            filter: FilterLadder::new(1200.0, 1.0),
            vca: VCA::new(1.0),
            amp_env: EnvADSR::new(0.01, 0.2, 0.8, 0.3),
            filter_env: EnvADSR::new(0.05, 0.3, 0.4, 0.4),
            lfo: LFO::new(2.0, LfoShape::Sine),
            macro_view: AetherMacroView {
                osc_mix: MacroKnob::new(0.5),
                filter_cutoff: MacroKnob::new(0.6),
                filter_res: MacroKnob::new(0.25),
                lfo_speed: MacroKnob::new(0.2),
            },
        }
    }

    pub fn trigger(&mut self, gate: bool) {
        self.amp_env.trigger(gate);
        self.filter_env.trigger(gate);
    }

    pub fn trigger_note(&mut self, note: u8, ctx: &ProcessContext) {
        self.osc_saw.trigger(note, ctx);
        self.osc_pulse.trigger(note, ctx);
        self.trigger(true);
    }
}

impl SignalProcessor for AetherSynth {
    fn name(&self) -> &str {
        "AetherSynth"
    }

    fn handle_event(&mut self, event: &summoner_core::node::NodeEvent, ctx: &ProcessContext) {
        use summoner_core::node::NodeEvent;
        match *event {
            NodeEvent::NoteOn { note, velocity } => {
                self.vca.gain = velocity;
                self.trigger_note(note, ctx);
            }
            NodeEvent::NoteOff { .. } => {
                self.trigger(false);
            }
            NodeEvent::SetParam { id: 0, value } => {
                self.macro_view.osc_mix.value = value.clamp(0.0, 1.0);
            }
            NodeEvent::SetParam { id: 1, value } => {
                self.filter.cutoff = value.clamp(20.0, 20000.0);
            }
            NodeEvent::SetParam { id: 2, value } => {
                self.filter.resonance = value.clamp(0.0, 4.0);
            }
            NodeEvent::SetParam { id: 3, value } => {
                self.lfo.frequency = value.clamp(0.01, 50.0);
            }
            NodeEvent::SetParam { .. } => {}
        }
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

            let lfo_val = self.lfo.process_sample(ctx.sample_rate);
            self.osc_pulse.pulse_width = (0.5 + 0.4 * lfo_val).clamp(0.05, 0.95);

            let saw_val = self.osc_saw.process_sample(ctx.sample_rate);
            let pulse_val = self.osc_pulse.process_sample(ctx.sample_rate);

            let mixed = (1.0 - self.macro_view.osc_mix.value) * saw_val
                + self.macro_view.osc_mix.value * pulse_val;

            let f_env = self.filter_env.process_sample(ctx.sample_rate);
            let cutoff_mod = (self.filter.cutoff + f_env * 4000.0).clamp(20.0, 20000.0);
            self.filter.cutoff = cutoff_mod;

            let filtered = self.filter.process_sample(mixed, ctx.sample_rate);
            let a_env = self.amp_env.process_sample(ctx.sample_rate);
            let out_sample = filtered * a_env * self.vca.gain;

            for out_ch in outputs.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = out_sample;
                }
            }
        }
    }
}

/// Composite device: Pluck (Karplus-Strong physical modeling synth).
#[derive(Debug)]
pub struct PluckSynth {
    pub noise: NoiseGen,
    pub exciter_env: EnvADSR,
    pub comb_filter: FilterComb,
    pub damping_filter: FilterSVF,
}

impl PluckSynth {
    pub fn new(frequency: f32) -> Self {
        let mut exciter_env = EnvADSR::new(0.001, 0.015, 0.0, 0.001);
        exciter_env.trigger(true);

        Self {
            noise: NoiseGen::new(NoiseType::White),
            exciter_env,
            comb_filter: FilterComb::new(frequency, 0.98),
            damping_filter: FilterSVF::new(3500.0, 0.707),
        }
    }

    pub fn pluck(&mut self) {
        self.exciter_env.trigger(true);
    }
}

impl SignalProcessor for PluckSynth {
    fn name(&self) -> &str {
        "PluckSynth"
    }

    fn handle_event(&mut self, event: &summoner_core::node::NodeEvent, ctx: &ProcessContext) {
        use summoner_core::node::NodeEvent;
        match *event {
            NodeEvent::NoteOn { note, .. } => {
                self.comb_filter.frequency = ctx.note_to_hz(note as i32);
                self.pluck();
            }
            NodeEvent::NoteOff { .. } => {
                self.exciter_env.trigger(false);
            }
            NodeEvent::SetParam { id: 0, value } => {
                self.comb_filter.frequency = value.max(10.0);
            }
            NodeEvent::SetParam { id: 1, value } => {
                self.comb_filter.feedback = value.clamp(0.0, 0.999);
            }
            NodeEvent::SetParam { id: 2, value } => {
                self.damping_filter.cutoff = value.clamp(20.0, 20000.0);
            }
            NodeEvent::SetParam { .. } => {}
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
        for i in 0..num_samples {
            let noise_sample = self.noise.next_sample();
            let env_val = self.exciter_env.process_sample(ctx.sample_rate);
            let exciter = noise_sample * env_val;

            let comb_out = self.comb_filter.process_sample(exciter, ctx.sample_rate);
            let (damped, _, _) = self
                .damping_filter
                .process_sample(comb_out, ctx.sample_rate);

            for out_ch in outputs.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = damped;
                }
            }
        }
    }
}

/// Composite device: Basic FM Operator Pair (2-Op Phase Modulation).
#[derive(Debug)]
pub struct FmOperatorPair {
    pub modulator: OscSine,
    pub mod_env: EnvADSR,
    pub mod_scale: MathMult,
    pub carrier: OscSine,
    pub amp_env: EnvADSR,
    pub vca: VCA,
    pub ratio: f32,
}

impl FmOperatorPair {
    pub fn new(carrier_freq: f32, ratio: f32) -> Self {
        Self {
            modulator: OscSine::new(carrier_freq * ratio),
            mod_env: EnvADSR::new(0.01, 0.3, 0.2, 0.3),
            mod_scale: MathMult,
            carrier: OscSine::new(carrier_freq),
            amp_env: EnvADSR::new(0.005, 0.4, 0.7, 0.4),
            vca: VCA::new(1.0),
            ratio,
        }
    }

    pub fn trigger(&mut self, gate: bool) {
        self.mod_env.trigger(gate);
        self.amp_env.trigger(gate);
    }
}

impl SignalProcessor for FmOperatorPair {
    fn name(&self) -> &str {
        "FmOperatorPair"
    }

    fn handle_event(&mut self, event: &summoner_core::node::NodeEvent, ctx: &ProcessContext) {
        use summoner_core::node::NodeEvent;
        match *event {
            NodeEvent::NoteOn { note, velocity } => {
                let freq = ctx.note_to_hz(note as i32);
                self.carrier.frequency = freq;
                self.modulator.frequency = freq * self.ratio;
                self.vca.gain = velocity;
                self.trigger(true);
            }
            NodeEvent::NoteOff { .. } => {
                self.trigger(false);
            }
            NodeEvent::SetParam { id: 0, value } => {
                self.carrier.frequency = value.max(1.0);
                self.modulator.frequency = self.carrier.frequency * self.ratio;
            }
            NodeEvent::SetParam { id: 1, value } => {
                self.ratio = value.max(0.01);
                self.modulator.frequency = self.carrier.frequency * self.ratio;
            }
            NodeEvent::SetParam { .. } => {}
        }
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

        let dt_carrier = (std::f32::consts::TAU * self.carrier.frequency) / ctx.sample_rate as f32;
        let dt_mod = (std::f32::consts::TAU * (self.carrier.frequency * self.ratio))
            / ctx.sample_rate as f32;

        let num_samples = outputs[0].len();
        for i in 0..num_samples {
            if !inputs.is_empty() && !inputs[0].is_empty() && i < inputs[0].len() {
                self.trigger(inputs[0][i] > 0.5);
            }

            let mod_raw = self.modulator.phase.sin();
            self.modulator.phase = (self.modulator.phase + dt_mod) % std::f32::consts::TAU;

            let m_env = self.mod_env.process_sample(ctx.sample_rate);
            let pm_offset = mod_raw * m_env * 2.0;

            let carrier_val = (self.carrier.phase + pm_offset).sin();
            self.carrier.phase = (self.carrier.phase + dt_carrier) % std::f32::consts::TAU;

            let a_env = self.amp_env.process_sample(ctx.sample_rate);
            let out_sample = carrier_val * a_env * self.vca.gain;

            for out_ch in outputs.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = out_sample;
                }
            }
        }
    }
}

/// Composite Flagship Device: GlitchAetherMachine
/// Integrates subtractive synth core, multi-mode distortion, Moog filter, GlitchGate chopper, and TapeStop.
#[derive(Debug)]
pub struct GlitchAetherMachine {
    pub osc_saw: OscSaw,
    pub osc_pulse: OscPulse,
    pub distortion: DistortionNode,
    pub filter: FilterLadder,
    pub chopper: GlitchGate,
    pub tape_stop: TapeStop,
    pub amp_env: EnvADSR,
    pub vca: VCA,
}

impl GlitchAetherMachine {
    pub fn new(frequency: f32) -> Self {
        Self {
            osc_saw: OscSaw::new(frequency),
            osc_pulse: OscPulse::new(frequency, 0.5),
            distortion: DistortionNode::new(DistortionType::TubeOverdrive, 3.0),
            filter: FilterLadder::new(2500.0, 1.5),
            chopper: GlitchGate::new(8.0, 0.6),
            tape_stop: TapeStop::new(0.3, 0.2),
            amp_env: EnvADSR::new(0.01, 0.2, 0.8, 0.3),
            vca: VCA::new(1.0),
        }
    }

    pub fn trigger(&mut self, gate: bool) {
        self.amp_env.trigger(gate);
    }
}

impl SignalProcessor for GlitchAetherMachine {
    fn name(&self) -> &str {
        "GlitchAetherMachine"
    }

    fn handle_event(&mut self, event: &summoner_core::node::NodeEvent, ctx: &ProcessContext) {
        use summoner_core::node::NodeEvent;
        match *event {
            NodeEvent::NoteOn { note, velocity } => {
                self.osc_saw.trigger(note, ctx);
                self.osc_pulse.trigger(note, ctx);
                self.vca.gain = velocity;
                self.trigger(true);
            }
            NodeEvent::NoteOff { .. } => {
                self.trigger(false);
            }
            NodeEvent::SetParam { id: 0, value } => {
                self.filter.cutoff = value.clamp(20.0, 20000.0);
            }
            NodeEvent::SetParam { id: 1, value } => {
                self.filter.resonance = value.clamp(0.0, 4.0);
            }
            NodeEvent::SetParam { id: 2, value } => {
                self.chopper.rate_hz = value.clamp(0.1, 100.0);
            }
            NodeEvent::SetParam { .. } => {}
        }
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

            let saw_val = self.osc_saw.process_sample(ctx.sample_rate);
            let pulse_val = self.osc_pulse.process_sample(ctx.sample_rate);
            let raw_synth = 0.5 * saw_val + 0.5 * pulse_val;

            let distorted = self.distortion.process_sample(raw_synth);
            let filtered = self.filter.process_sample(distorted, ctx.sample_rate);
            let gate_val = if self.chopper.phase < self.chopper.pulse_width {
                1.0
            } else {
                0.0
            };
            self.chopper.phase =
                (self.chopper.phase + self.chopper.rate_hz / ctx.sample_rate as f32) % 1.0;

            let chopped = filtered * gate_val;
            let a_env = self.amp_env.process_sample(ctx.sample_rate);
            let out_sample = chopped * a_env * self.vca.gain;

            for out_ch in outputs.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = out_sample;
                }
            }
        }
    }
}

/// Composite Device: CyberpunkSubSynth (Sub-bass + Fuzz/Wavefolder + LFO Pitch Wobble)
#[derive(Debug)]
pub struct CyberpunkSubSynth {
    pub sub_sine: OscSine,
    pub drive_saw: OscSaw,
    pub filter_svf: FilterSVF,
    pub wavefolder: DistortionNode,
    pub pitch_lfo: LFO,
    pub amp_env: EnvADSR,
}

impl CyberpunkSubSynth {
    pub fn new(sub_freq: f32) -> Self {
        Self {
            sub_sine: OscSine::new(sub_freq),
            drive_saw: OscSaw::new(sub_freq * 2.0),
            filter_svf: FilterSVF::new(600.0, 1.2),
            wavefolder: DistortionNode::new(DistortionType::Wavefolder, 4.0),
            pitch_lfo: LFO::new(4.0, LfoShape::Sine),
            amp_env: EnvADSR::new(0.01, 0.4, 0.9, 0.4),
        }
    }

    pub fn trigger(&mut self, gate: bool) {
        self.amp_env.trigger(gate);
    }
}

impl SignalProcessor for CyberpunkSubSynth {
    fn name(&self) -> &str {
        "CyberpunkSubSynth"
    }

    fn handle_event(&mut self, event: &summoner_core::node::NodeEvent, ctx: &ProcessContext) {
        use summoner_core::node::NodeEvent;
        match *event {
            NodeEvent::NoteOn { note, .. } => {
                let freq = ctx.note_to_hz(note as i32);
                self.sub_sine.frequency = freq;
                self.drive_saw.frequency = freq * 2.0;
                self.trigger(true);
            }
            NodeEvent::NoteOff { .. } => {
                self.trigger(false);
            }
            NodeEvent::SetParam { id: 0, value } => {
                self.filter_svf.cutoff = value.clamp(20.0, 20000.0);
            }
            NodeEvent::SetParam { id: 1, value } => {
                self.pitch_lfo.frequency = value.clamp(0.01, 50.0);
            }
            NodeEvent::SetParam { .. } => {}
        }
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

            let lfo_val = self.pitch_lfo.process_sample(ctx.sample_rate);
            let sub_val = self.sub_sine.process_sample(ctx.sample_rate, lfo_val * 0.1);

            let saw_val = self.drive_saw.process_sample(ctx.sample_rate);

            let mixed = sub_val * 0.7 + saw_val * 0.3;
            let (lp, _, _) = self.filter_svf.process_sample(mixed, ctx.sample_rate);
            let folded = self.wavefolder.process_sample(lp);

            let a_env = self.amp_env.process_sample(ctx.sample_rate);
            let out_sample = folded * a_env;

            for out_ch in outputs.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = out_sample;
                }
            }
        }
    }
}

/// Composite Device: AtmosphericPadSynth (Detuned Saw/Tri + SVF Filter + Reverb + Delay Space)
#[derive(Debug)]
pub struct AtmosphericPadSynth {
    pub saw1: OscSaw,
    pub tri2: OscTriangle,
    pub svf: FilterSVF,
    pub lfo_filter: LFO,
    pub delay: EffectDelay,
    pub reverb: EffectReverb,
    pub amp_env: EnvADSR,
}

impl AtmosphericPadSynth {
    pub fn new(freq: f32) -> Self {
        Self {
            saw1: OscSaw::new(freq),
            tri2: OscTriangle::new(freq * 1.003), // Detuned by 3 cents
            svf: FilterSVF::new(800.0, 0.707),
            lfo_filter: LFO::new(0.5, LfoShape::Triangle),
            delay: EffectDelay::new(0.3, 0.4, 0.3),
            reverb: EffectReverb::new(0.85, 0.4),
            amp_env: EnvADSR::new(0.5, 0.8, 0.9, 1.2),
        }
    }

    pub fn trigger(&mut self, gate: bool) {
        self.amp_env.trigger(gate);
    }
}

impl SignalProcessor for AtmosphericPadSynth {
    fn name(&self) -> &str {
        "AtmosphericPadSynth"
    }

    fn handle_event(&mut self, event: &summoner_core::node::NodeEvent, ctx: &ProcessContext) {
        use summoner_core::node::NodeEvent;
        match *event {
            NodeEvent::NoteOn { note, .. } => {
                let freq = ctx.note_to_hz(note as i32);
                self.saw1.frequency = freq;
                self.tri2.frequency = freq * 1.003;
                self.trigger(true);
            }
            NodeEvent::NoteOff { .. } => {
                self.trigger(false);
            }
            NodeEvent::SetParam { id: 0, value } => {
                self.svf.cutoff = value.clamp(20.0, 20000.0);
            }
            NodeEvent::SetParam { id: 1, value } => {
                self.lfo_filter.frequency = value.clamp(0.01, 50.0);
            }
            NodeEvent::SetParam { .. } => {}
        }
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

            let lfo_val = self.lfo_filter.process_sample(ctx.sample_rate);
            self.svf.cutoff = (800.0 + lfo_val * 400.0).clamp(100.0, 5000.0);

            let saw_val = self.saw1.process_sample(ctx.sample_rate);
            let tri_val = self.tri2.process_sample(ctx.sample_rate);

            let (filtered, _, _) = self
                .svf
                .process_sample(0.5 * (saw_val + tri_val), ctx.sample_rate);
            let a_env = self.amp_env.process_sample(ctx.sample_rate);
            let synth_out = filtered * a_env;

            let rev_out = self.reverb.process_sample(synth_out);

            for out_ch in outputs.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = rev_out;
                }
            }
        }

        let num_samples = outputs[0].len();
        let mut temp_l = [0.0f32; 4096];
        let mut temp_r = [0.0f32; 4096];
        let mut offset = 0;
        while offset < num_samples {
            let chunk_len = (num_samples - offset).min(temp_l.len());
            temp_l[..chunk_len].copy_from_slice(&outputs[0][offset..offset + chunk_len]);
            if outputs.len() > 1 {
                temp_r[..chunk_len].copy_from_slice(&outputs[1][offset..offset + chunk_len]);
            } else {
                temp_r[..chunk_len].fill(0.0);
            }
            if outputs.len() > 1 {
                let (out0, rest) = outputs.split_at_mut(1);
                let mut out_slices = [
                    &mut out0[0][offset..offset + chunk_len],
                    &mut rest[0][offset..offset + chunk_len],
                ];
                self.delay.process_block(
                    &[&temp_l[..chunk_len], &temp_r[..chunk_len]],
                    &mut out_slices,
                    ctx,
                );
            } else {
                let mut out_slices = [&mut outputs[0][offset..offset + chunk_len]];
                self.delay
                    .process_block(&[&temp_l[..chunk_len]], &mut out_slices, ctx);
            }
            offset += chunk_len;
        }
    }
}

/// Composite Device: GlitchPercussionSynth (Noise Burst + Sine Drop + Bitcrusher + Stutter)
#[derive(Debug)]
pub struct GlitchPercussionSynth {
    pub noise: NoiseGen,
    pub pitch_sine: OscSine,
    pub bitcrusher: DistortionNode,
    pub comb_filter: FilterComb,
    pub stutter: GlitchStutter,
    pub perc_env: EnvADSR,
}

impl GlitchPercussionSynth {
    pub fn new(pitch_freq: f32) -> Self {
        let mut perc_env = EnvADSR::new(0.001, 0.08, 0.0, 0.001);
        perc_env.trigger(true);

        Self {
            noise: NoiseGen::new(NoiseType::White),
            pitch_sine: OscSine::new(pitch_freq),
            bitcrusher: DistortionNode::new(DistortionType::Bitcrusher, 1.0),
            comb_filter: FilterComb::new(pitch_freq * 2.0, 0.8),
            stutter: GlitchStutter::new(256),
            perc_env,
        }
    }

    pub fn trigger(&mut self) {
        self.perc_env.trigger(true);
    }
}

impl SignalProcessor for GlitchPercussionSynth {
    fn name(&self) -> &str {
        "GlitchPercussionSynth"
    }

    fn handle_event(&mut self, event: &summoner_core::node::NodeEvent, ctx: &ProcessContext) {
        use summoner_core::node::NodeEvent;
        match *event {
            NodeEvent::NoteOn { note, .. } => {
                let freq = ctx.note_to_hz(note as i32);
                self.pitch_sine.frequency = freq;
                self.comb_filter.frequency = freq * 2.0;
                self.trigger();
            }
            NodeEvent::NoteOff { .. } => {
                self.perc_env.trigger(false);
            }
            NodeEvent::SetParam { id: 0, value } => {
                self.pitch_sine.frequency = value.max(10.0);
                self.comb_filter.frequency = self.pitch_sine.frequency * 2.0;
            }
            NodeEvent::SetParam { .. } => {}
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
        for i in 0..num_samples {
            let noise_val = self.noise.next_sample();
            let sine_val = self.pitch_sine.process_sample(ctx.sample_rate, 0.0);

            let mixed = noise_val * 0.4 + sine_val * 0.6;
            let crushed = self.bitcrusher.process_sample(mixed);
            let comb_out = self.comb_filter.process_sample(crushed, ctx.sample_rate);

            let env_val = self.perc_env.process_sample(ctx.sample_rate);
            let out_sample = comb_out * env_val;

            for out_ch in outputs.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = out_sample;
                }
            }
        }

        let num_samples = outputs[0].len();
        let mut temp_l = [0.0f32; 4096];
        let mut offset = 0;
        while offset < num_samples {
            let chunk_len = (num_samples - offset).min(temp_l.len());
            temp_l[..chunk_len].copy_from_slice(&outputs[0][offset..offset + chunk_len]);
            if outputs.len() > 1 {
                let (out0, rest) = outputs.split_at_mut(1);
                let mut out_slices = [
                    &mut out0[0][offset..offset + chunk_len],
                    &mut rest[0][offset..offset + chunk_len],
                ];
                self.stutter
                    .process_block(&[&temp_l[..chunk_len]], &mut out_slices, ctx);
            } else {
                let mut out_slices = [&mut outputs[0][offset..offset + chunk_len]];
                self.stutter
                    .process_block(&[&temp_l[..chunk_len]], &mut out_slices, ctx);
            }
            offset += chunk_len;
        }
    }
}

use crate::sampler::{MultiSampleBank, MultiSamplerNode};

/// Macro View parameters for SamplerDevice.
#[derive(Debug)]
pub struct SamplerMacroView {
    pub filter_cutoff: MacroKnob, // 20.0 to 20000.0 Hz
    pub filter_res: MacroKnob,    // 0.0 to 4.0
    pub amp_attack: MacroKnob,    // Seconds
    pub amp_release: MacroKnob,   // Seconds
    pub gain: MacroKnob,          // Volume scaling
}

/// Composite sub-graph device: SamplerDevice (MultiSamplerNode + EnvADSR + FilterLadder + VCA).
#[derive(Debug)]
pub struct SamplerDevice {
    pub sampler: MultiSamplerNode,
    pub amp_env: EnvADSR,
    pub filter: FilterLadder,
    pub vca: VCA,
    pub macro_view: SamplerMacroView,
}

impl SamplerDevice {
    pub fn new(bank: MultiSampleBank) -> Self {
        Self {
            sampler: MultiSamplerNode::new(bank),
            amp_env: EnvADSR::new(0.005, 0.1, 1.0, 0.4),
            filter: FilterLadder::new(18000.0, 0.7),
            vca: VCA::new(1.0),
            macro_view: SamplerMacroView {
                filter_cutoff: MacroKnob::new(1.0),
                filter_res: MacroKnob::new(0.2),
                amp_attack: MacroKnob::new(0.005),
                amp_release: MacroKnob::new(0.4),
                gain: MacroKnob::new(0.8),
            },
        }
    }

    /// Triggers a MIDI note on the internal sampler node and activates amplitude envelope.
    ///
    /// # Examples
    ///
    /// ```
    /// use summoner_dsp::composites::SamplerDevice;
    /// use summoner_dsp::sampler::MultiSampleBank;
    ///
    /// let bank = MultiSampleBank::new();
    /// let mut device = SamplerDevice::new(bank);
    /// device.trigger_note(60, 100);
    /// ```
    pub fn trigger_note(&mut self, note: u8, velocity: u8) {
        self.sampler.trigger_note(note, velocity);
        self.amp_env.trigger(true);
    }

    pub fn release_note(&mut self) {
        self.amp_env.trigger(false);
    }
}

impl SignalProcessor for SamplerDevice {
    fn name(&self) -> &str {
        "SamplerDevice"
    }

    fn handle_event(&mut self, event: &summoner_core::node::NodeEvent, _ctx: &ProcessContext) {
        use summoner_core::node::NodeEvent;
        match *event {
            NodeEvent::NoteOn { note, velocity } => {
                let vel_u8 = (velocity.clamp(0.0, 1.0) * 127.0).round() as u8;
                self.trigger_note(note, vel_u8.max(1));
            }
            NodeEvent::NoteOff { .. } => {
                self.release_note();
            }
            NodeEvent::SetParam { id: 0, value } => {
                self.macro_view.filter_cutoff.value = value.clamp(0.0, 1.0);
            }
            NodeEvent::SetParam { id: 1, value } => {
                self.macro_view.filter_res.value = value.clamp(0.0, 1.0);
            }
            NodeEvent::SetParam { id: 2, value } => {
                self.macro_view.gain.value = value.clamp(0.0, 1.0);
            }
            NodeEvent::SetParam { .. } => {}
        }
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
        let mut raw_buf = [0.0f32; 4096];
        let mut offset = 0;
        while offset < num_samples {
            let chunk_len = (num_samples - offset).min(raw_buf.len());
            raw_buf[..chunk_len].fill(0.0);
            {
                let mut sampler_out = [&mut raw_buf[..chunk_len]];
                self.sampler
                    .process_block(inputs, &mut sampler_out[..], ctx);
            }

            for (i, &sample_in) in raw_buf[..chunk_len].iter().enumerate() {
                self.filter.cutoff = self.macro_view.filter_cutoff.get_value() * 18000.0 + 20.0;
                self.filter.resonance = self.macro_view.filter_res.get_value() * 3.9;
                let filtered = self.filter.process_sample(sample_in, ctx.sample_rate);

                let env_level = self.amp_env.process_sample(ctx.sample_rate);
                let vca_out = filtered * env_level * self.macro_view.gain.get_value();

                let dst_idx = offset + i;
                for out_ch in outputs.iter_mut() {
                    if dst_idx < out_ch.len() {
                        out_ch[dst_idx] = vca_out;
                    }
                }
            }
            offset += chunk_len;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modulators::{LfoShape, MacroModulationMatrix, ModulationCurve};
    use summoner_core::allocator::AllocGuard;

    #[test]
    fn test_zero_heap_allocation_in_composites_and_mod_matrix() {
        let mut pad = AtmosphericPadSynth::new(220.0);
        let mut perc = GlitchPercussionSynth::new(110.0);
        let mut sampler_dev = SamplerDevice::new(MultiSampleBank::new());
        sampler_dev.trigger_note(60, 100);

        let mut matrix = MacroModulationMatrix::new("RT Matrix");
        let lfo = matrix.add_lfo(2.0, LfoShape::Sine);
        let env = matrix.add_envelope(0.01, 0.1, 0.5, 0.2);
        let mac = matrix.add_macro(0.5);
        let target = matrix.add_target("Cutoff", 1000.0, 20.0, 20000.0);
        matrix.add_assignment(lfo, target, 0.5, true, ModulationCurve::Linear);
        matrix.add_assignment(env, target, 0.3, false, ModulationCurve::Linear);
        matrix.add_assignment(mac, target, 0.2, false, ModulationCurve::Linear);
        matrix.trigger_envelope(0, true);

        let mut out_l = [0.0f32; 256];
        let mut out_r = [0.0f32; 256];
        let ctx = ProcessContext::new(48000, 120.0, 0);

        let _guard = AllocGuard::new();
        {
            let mut outs = [&mut out_l[..], &mut out_r[..]];
            pad.process_block(&[], &mut outs, &ctx);
        }
        {
            let mut outs = [&mut out_l[..], &mut out_r[..]];
            perc.process_block(&[], &mut outs, &ctx);
        }
        {
            let mut outs = [&mut out_l[..], &mut out_r[..]];
            sampler_dev.process_block(&[], &mut outs, &ctx);
        }
        for _ in 0..256 {
            matrix.process_sample(48000);
        }
    }

    #[test]
    fn test_composite_synths_handle_note_events() {
        use summoner_core::node::NodeEvent;
        let ctx = ProcessContext::new(48000, 120.0, 0);
        let mut aether = AetherSynth::new(220.0);
        let mut out_l = [0.0f32; 128];
        let mut out_r = [0.0f32; 128];

        // Initially silent (envelope not triggered)
        aether.process_block(&[], &mut [&mut out_l[..], &mut out_r[..]], &ctx);
        assert!(out_l.iter().all(|&s| s == 0.0));

        // NoteOn triggers envelope and sets note frequency
        aether.handle_event(
            &NodeEvent::NoteOn {
                note: 69,
                velocity: 0.9,
            },
            &ctx,
        );
        aether.process_block(&[], &mut [&mut out_l[..], &mut out_r[..]], &ctx);
        assert!(out_l.iter().any(|&s| s.abs() > 1e-4));
        assert!((aether.osc_saw.frequency - 440.0).abs() < 0.01);
    }
}
