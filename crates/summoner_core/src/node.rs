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

//! Signal graph AudioNode traits and reference DSP nodes.

use crate::audio::Sample;
use crate::transport::Transport;
use std::f32::consts::TAU;

use std::sync::Arc;
use crate::param_bus::ParamBus;

/// Contextual metadata passed to `AudioNode::process` during DSP render evaluation.
#[derive(Debug, Clone)]
pub struct ProcessContext {
    pub frame_position: u64,
    pub sample_rate: u32,
    pub bpm: f64,
    pub is_playing: bool,
    pub param_bus: Option<Arc<ParamBus>>,
    pub tuning_root_hz: f32,
    pub tuning_edo_divisions: u32,
}

impl ProcessContext {
    /// Construct a `ProcessContext` directly from raw values, for tests and internal DSP use.
    pub fn new(sample_rate: u32, bpm: f64, frame_position: u64) -> Self {
        Self {
            frame_position,
            sample_rate,
            bpm,
            is_playing: true,
            param_bus: None,
            tuning_root_hz: 440.0,
            tuning_edo_divisions: 12,
        }
    }

    pub fn from_transport(transport: &Transport) -> Self {
        Self {
            frame_position: transport.frame_position,
            sample_rate: transport.sample_rate,
            bpm: transport.bpm,
            is_playing: transport.is_playing,
            param_bus: None,
            tuning_root_hz: 440.0,
            tuning_edo_divisions: 12,
        }
    }

    pub fn with_tuning(transport: &Transport, root_hz: f32, edo_divisions: u32) -> Self {
        Self {
            frame_position: transport.frame_position,
            sample_rate: transport.sample_rate,
            bpm: transport.bpm,
            is_playing: transport.is_playing,
            param_bus: None,
            tuning_root_hz: root_hz,
            tuning_edo_divisions: edo_divisions,
        }
    }

    pub fn note_to_hz(&self, midi_note: i32) -> f32 {
        let semitone_from_root = midi_note as f32 - 69.0; // relative to A4
        self.tuning_root_hz * 2.0f32.powf(semitone_from_root / self.tuning_edo_divisions as f32)
    }
}

/// Fundamental signal processing node interface for audio rendering graph.
pub trait AudioNode: Send {
    /// Return human-readable identifier for this node type.
    fn name(&self) -> &str;

    /// Process an input sample buffer slice into output sample buffer slice.
    /// MUST NOT perform heap allocations (`malloc`/`free`) or block on locks.
    fn process(
        &mut self,
        input: &[&[Sample]],
        output: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    );
}

/// A transparent audio node that copies inputs directly to outputs.
#[derive(Debug, Default)]
pub struct PassthroughNode;

impl AudioNode for PassthroughNode {
    fn name(&self) -> &str {
        "PassthroughNode"
    }

    fn process(
        &mut self,
        input: &[&[Sample]],
        output: &mut [&mut [Sample]],
        _ctx: &ProcessContext,
    ) {
        let channels = input.len().min(output.len());
        for ch in 0..channels {
            let samples = input[ch].len().min(output[ch].len());
            output[ch][..samples].copy_from_slice(&input[ch][..samples]);
        }
    }
}

/// Gain node that scales input signals by a linear gain factor.
#[derive(Debug)]
pub struct GainNode {
    pub gain: Sample,
}

impl GainNode {
    pub fn new(gain: Sample) -> Self {
        Self { gain }
    }
}

impl AudioNode for GainNode {
    fn name(&self) -> &str {
        "GainNode"
    }

    fn process(
        &mut self,
        input: &[&[Sample]],
        output: &mut [&mut [Sample]],
        _ctx: &ProcessContext,
    ) {
        let channels = input.len().min(output.len());
        for ch in 0..channels {
            let samples = input[ch].len().min(output[ch].len());
            for i in 0..samples {
                output[ch][i] = input[ch][i] * self.gain;
            }
        }
    }
}

/// Deterministic sine wave oscillator generator node.
#[derive(Debug)]
pub struct SineOscillatorNode {
    pub frequency: f32,
    pub phase: f32,
}

impl SineOscillatorNode {
    pub fn new(frequency: f32) -> Self {
        Self {
            frequency,
            phase: 0.0,
        }
    }

    pub fn trigger(&mut self, note: u8, ctx: &ProcessContext) {
        self.frequency = ctx.note_to_hz(note as i32);
    }
}

impl AudioNode for SineOscillatorNode {
    fn name(&self) -> &str {
        "SineOscillatorNode"
    }

    fn process(
        &mut self,
        _input: &[&[Sample]],
        output: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        if ctx.sample_rate == 0 {
            return;
        }
        let phase_step = (TAU * self.frequency) / ctx.sample_rate as f32;
        let output_channels = output.len();
        if output_channels == 0 {
            return;
        }

        let num_samples = output[0].len();
        for i in 0..num_samples {
            let val = self.phase.sin();
            self.phase = (self.phase + phase_step) % TAU;
            for ch_slice in output.iter_mut().take(output_channels) {
                if i < ch_slice.len() {
                    ch_slice[i] = val;
                }
            }
        }
    }
}

/// Standard list of known DSP and utility node type names for graph editor and CLI.
pub const KNOWN_NODE_TYPES: &[&str] = &[
    "OscSine",
    "OscSaw",
    "OscPulse",
    "OscTriangle",
    "OscWavetable",
    "OscLFO",
    "FilterSVF",
    "FilterLadder",
    "EnvADSR",
    "MathAdd",
    "MathMult",
    "GainNode",
    "DistortionNode",
    "TapeSaturationNode",
    "TubeSaturationNode",
    "ConsoleEmulationNode",
    "WavefolderNode",
    "PitchShifterNode",
    "BitcrusherNode",
    "CompressorNode",
    "MultibandCompressorNode",
    "LimiterNode",
    "MidSideNode",
    "ParametricEqNode",
    "GranularSynthNode",
    "EffectChorus",
    "EffectFlanger",
    "EffectPhaser",
    "EffectDelay",
    "EffectReverb",
    "NoiseGateNode",
    "DeesserNode",
    "HarmonicExciterNode",
    "PassthroughNode",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_context_tuning() {
        // Default 12-EDO, A4 = 440
        let ctx_12 = ProcessContext::new(44100, 120.0, 0);
        assert!((ctx_12.note_to_hz(69) - 440.0).abs() < 1e-4);
        assert!((ctx_12.note_to_hz(70) - 466.1637).abs() < 1e-3); // A#4

        // Custom 19-EDO, A4 = 440
        let mut ctx_19 = ProcessContext::new(44100, 120.0, 0);
        ctx_19.tuning_edo_divisions = 19;
        assert!((ctx_19.note_to_hz(69) - 440.0).abs() < 1e-4);
        // Note 70 in 19-EDO: 440 * 2^(1/19) = 456.3482
        assert!((ctx_19.note_to_hz(70) - 456.3482).abs() < 1e-3);
    }

    #[test]
    fn test_process_context_param_bus_send_sync() {
        let bus = Arc::new(ParamBus::new());
        let mut ctx = ProcessContext::new(44100, 120.0, 0);
        ctx.param_bus = Some(Arc::clone(&bus));

        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ProcessContext>();
        assert!(ctx.param_bus.is_some());
    }
}
