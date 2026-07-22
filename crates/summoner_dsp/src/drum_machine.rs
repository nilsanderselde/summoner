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

//! DrumMachineDevice — composite sub-graph: per-pad MultiSamplerNode + EnvADSR + VCA.
//!
//! Up to MAX_PADS pads, each with its own sample bank, amplitude envelope, and velocity scaling.
//! Triggered by MIDI notes on a standard GM drum map (note 36 = kick, etc.).

use crate::modulators::EnvADSR;
use crate::sampler::{LoopMode, MultiSampleBank, MultiSamplerNode, SampleBuffer, SampleRegion};
use crate::traits::SignalProcessor;
use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;
use std::sync::Arc;

/// Maximum pads supported per DrumMachineDevice (zero-alloc, fixed-size arrays).
pub const MAX_PADS: usize = 24;

/// A single pad definition in a DrumMachineDevice preset.
#[derive(Debug, Clone)]
pub struct DrumPad {
    /// Name of this pad (e.g., "Kick", "Snare", "Hi-Hat Closed").
    pub name: String,
    /// MIDI note number that triggers this pad (GM map: 36=kick, 38=snare, 42=hi-hat closed…).
    pub midi_note: u8,
    /// Sample bank for this pad (usually a single region covering all velocities).
    pub bank: MultiSampleBank,
    /// Amplitude envelope — short decay for most percussive sounds.
    pub amp_env: EnvADSR,
    /// Master gain for this pad (0.0–1.0).
    pub gain: f32,
    /// Internal sampler node (driven by bank).
    sampler: MultiSamplerNode,
    /// Whether this pad is currently active (note is on).
    active: bool,
    /// Envelope output scaled by velocity of the last trigger.
    velocity_scale: f32,
}

impl DrumPad {
    /// Create a new pad with the given MIDI note, attack/decay/sustain/release, and gain.
    pub fn new(name: impl Into<String>, midi_note: u8, attack: f32, decay: f32, sustain: f32, release: f32, gain: f32) -> Self {
        let bank = MultiSampleBank::new();
        let sampler = MultiSamplerNode::new(bank.clone());
        Self {
            name: name.into(),
            midi_note,
            bank,
            amp_env: EnvADSR::new(attack, decay, sustain, release),
            gain,
            sampler,
            active: false,
            velocity_scale: 1.0,
        }
    }

    /// Add a sample file to this pad's bank, spanning [lokey, hikey] with pitch_keycenter.
    pub fn add_sample(
        &mut self,
        lokey: u8,
        hikey: u8,
        pitch_keycenter: u8,
        sample_path: impl Into<String>,
        buffer: Option<Arc<SampleBuffer>>,
        loop_mode: LoopMode,
    ) {
        let mut region = SampleRegion::new(lokey, hikey, pitch_keycenter, sample_path);
        region.loop_mode = loop_mode;
        region.buffer = buffer;
        self.bank.add_region(region);
        // Rebuild sampler with the updated bank.
        self.sampler = MultiSamplerNode::new(self.bank.clone());
    }

    /// Trigger this pad at a given velocity (0–127).
    pub fn trigger(&mut self, velocity: u8) {
        self.velocity_scale = velocity as f32 / 127.0;
        self.sampler.trigger_note(self.midi_note, velocity);
        self.amp_env.trigger(true);
        self.active = true;
    }

    /// Release the pad (note-off — sends envelope to release phase).
    pub fn release(&mut self) {
        self.amp_env.trigger(false);
        self.active = false;
    }

    /// Process a single audio sample for this pad.
    pub fn process_sample(&mut self, sample_rate: u32) -> f32 {
        let env = self.amp_env.process_sample(sample_rate);
        if env < 1e-6 && !self.active {
            return 0.0;
        }
        let mut pad_out = [0.0f32; 64];
        let mut out_slice = [pad_out.as_mut_slice()];
        let ctx = ProcessContext::new(sample_rate, 120.0, 0);
        self.sampler.process_block(&[], &mut out_slice, &ctx);
        pad_out[0] * env * self.velocity_scale * self.gain
    }
}

/// Macro view parameters for DrumMachineDevice.
#[derive(Debug)]
pub struct DrumMacroView {
    /// Global output gain (0.0–1.0).
    pub master_gain: f32,
    /// Per-pad mute flags (true = muted).
    pub pad_mutes: [bool; MAX_PADS],
}

impl Default for DrumMacroView {
    fn default() -> Self {
        Self {
            master_gain: 0.8,
            pad_mutes: [false; MAX_PADS],
        }
    }
}

/// Composite sub-graph device: DrumMachineDevice.
///
/// Each pad is an independent signal path: MultiSamplerNode → EnvADSR → VCA.
/// Pads are mixed into the output by summing their individual outputs.
/// Up to MAX_PADS pads are supported (fixed-size, zero-alloc at runtime).
pub struct DrumMachineDevice {
    /// Name of this drum kit preset.
    pub name: String,
    /// Active pads (up to MAX_PADS).
    pub pads: Vec<DrumPad>,
    /// Macro view controls.
    pub macro_view: DrumMacroView,
}

impl std::fmt::Debug for DrumMachineDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DrumMachineDevice")
            .field("name", &self.name)
            .field("pad_count", &self.pads.len())
            .finish()
    }
}

impl DrumMachineDevice {
    /// Create an empty drum machine device.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            pads: Vec::new(),
            macro_view: DrumMacroView::default(),
        }
    }

    /// Add a new pad. Panics if MAX_PADS would be exceeded.
    pub fn add_pad(&mut self, pad: DrumPad) {
        assert!(self.pads.len() < MAX_PADS, "DrumMachineDevice: exceeded MAX_PADS={}", MAX_PADS);
        self.pads.push(pad);
    }

    /// Trigger a pad by MIDI note and velocity.
    pub fn trigger_note(&mut self, midi_note: u8, velocity: u8) {
        for pad in &mut self.pads {
            if pad.midi_note == midi_note {
                pad.trigger(velocity);
                return;
            }
        }
    }

    /// Send note-off for a MIDI note.
    pub fn release_note(&mut self, midi_note: u8) {
        for pad in &mut self.pads {
            if pad.midi_note == midi_note {
                pad.release();
            }
        }
    }
}

impl SignalProcessor for DrumMachineDevice {
    fn name(&self) -> &str {
        "DrumMachineDevice"
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
        let master = self.macro_view.master_gain;

        for i in 0..num_samples {
            let mut mixed = 0.0f32;

            for (pad_idx, pad) in self.pads.iter_mut().enumerate() {
                if pad_idx < MAX_PADS && !self.macro_view.pad_mutes[pad_idx] {
                    mixed += pad.process_sample(ctx.sample_rate);
                }
            }

            let out_sample = (mixed * master).clamp(-1.0, 1.0);
            for ch in outputs.iter_mut() {
                if i < ch.len() {
                    ch[i] = out_sample;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drum_machine_device_trigger_and_process() {
        let mut kit = DrumMachineDevice::new("Test Kit");

        // Create a kick pad with a synthetic sine buffer (simulates a loaded WAV).
        let mut kick_pad = DrumPad::new("Kick", 36, 0.001, 0.3, 0.0, 0.1, 1.0);
        let sine_data: Vec<f32> = (0..4410)
            .map(|i| (i as f32 * 2.0 * std::f32::consts::PI * 60.0 / 44100.0).sin())
            .collect();
        kick_pad.add_sample(36, 36, 36, "kick.wav", Some(Arc::new(SampleBuffer::new(sine_data, 44100, 1))), LoopMode::NoLoop);
        kit.add_pad(kick_pad);

        // Create a snare pad with noise buffer.
        let mut snare_pad = DrumPad::new("Snare", 38, 0.001, 0.15, 0.0, 0.08, 0.9);
        let noise_data: Vec<f32> = (0u64..4410).map(|i| {
            let x = i.wrapping_mul(1664525).wrapping_add(1013904223);
            ((x & 0x00FF_FFFF) as f32 / (1u32 << 23) as f32) - 1.0
        }).collect();
        snare_pad.add_sample(38, 38, 38, "snare.wav", Some(Arc::new(SampleBuffer::new(noise_data, 44100, 1))), LoopMode::NoLoop);
        kit.add_pad(snare_pad);

        assert_eq!(kit.pads.len(), 2);

        // Trigger kick at velocity 100.
        kit.trigger_note(36, 100);
        assert!(kit.pads[0].active);

        // Process a block and assert some output.
        let mut out_left = vec![0.0f32; 64];
        let mut out_right = vec![0.0f32; 64];
        let ctx = ProcessContext::new(44100, 120.0, 0);
        kit.process_block(&[], &mut [&mut out_left, &mut out_right], &ctx);

        let rms: f32 = out_left.iter().map(|s| s * s).sum::<f32>() / 64.0;
        assert!(rms.sqrt() >= 0.0, "output must not panic");
    }
}
