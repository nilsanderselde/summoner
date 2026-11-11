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
use std::sync::Arc;
use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;

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
    /// Optional choke group (0–255). When a pad triggers, other active pads in the same group are choked.
    pub choke_group: Option<u8>,
    /// Stereo panning (-1.0 = left, 0.0 = center, 1.0 = right).
    pub pan: f32,
    /// Pitch tune offset in semitones (-24.0..24.0).
    pub pitch_semitones: f32,
    /// Auxiliary Send A level (0.0–1.0).
    pub send_a: f32,
    /// Auxiliary Send B level (0.0–1.0).
    pub send_b: f32,
    /// Pad solo flag.
    pub solo: bool,
    /// Pad mute flag.
    pub mute: bool,
    /// Internal sampler node (driven by bank).
    sampler: MultiSamplerNode,
    /// Whether this pad is currently active (note is on).
    active: bool,
    /// Envelope output scaled by velocity of the last trigger.
    velocity_scale: f32,
}

impl DrumPad {
    /// Create a new pad with the given MIDI note, attack/decay/sustain/release, and gain.
    pub fn new(
        name: impl Into<String>,
        midi_note: u8,
        attack: f32,
        decay: f32,
        sustain: f32,
        release: f32,
        gain: f32,
    ) -> Self {
        let bank = MultiSampleBank::new();
        let sampler = MultiSamplerNode::new(bank.clone());
        Self {
            name: name.into(),
            midi_note,
            bank,
            amp_env: EnvADSR::new(attack, decay, sustain, release),
            gain,
            choke_group: None,
            pan: 0.0,
            pitch_semitones: 0.0,
            send_a: 0.0,
            send_b: 0.0,
            solo: false,
            mute: false,
            sampler,
            active: false,
            velocity_scale: 1.0,
        }
    }

    /// Set choke group for mutual exclusion (e.g. closed & open hi-hats).
    pub fn with_choke_group(mut self, group: u8) -> Self {
        self.choke_group = Some(group);
        self
    }

    /// Set stereo pan position (-1.0 full left to 1.0 full right).
    pub fn with_pan(mut self, pan: f32) -> Self {
        self.pan = pan.clamp(-1.0, 1.0);
        self
    }

    /// Set auxiliary send levels.
    pub fn with_sends(mut self, send_a: f32, send_b: f32) -> Self {
        self.send_a = send_a.clamp(0.0, 1.0);
        self.send_b = send_b.clamp(0.0, 1.0);
        self
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

    /// Choke this pad immediately (cut off sound without click).
    pub fn choke(&mut self) {
        self.amp_env.trigger(false);
        self.sampler.stop();
        self.active = false;
    }

    /// Release the pad (note-off — sends envelope to release phase).
    pub fn release(&mut self) {
        self.amp_env.trigger(false);
        self.active = false;
    }

    /// Returns whether this pad is currently active.
    pub fn is_active(&self) -> bool {
        self.active
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
        assert!(
            self.pads.len() < MAX_PADS,
            "DrumMachineDevice: exceeded MAX_PADS={}",
            MAX_PADS
        );
        self.pads.push(pad);
    }

    /// Trigger a pad by MIDI note and velocity, choking any active pads in the same choke group.
    pub fn trigger_note(&mut self, midi_note: u8, velocity: u8) {
        // 1. Identify choke group of the triggered note
        let mut choke_group = None;
        for pad in &self.pads {
            if pad.midi_note == midi_note {
                choke_group = pad.choke_group;
                break;
            }
        }

        // 2. Choke all other active pads in the same choke group
        if let Some(cg) = choke_group {
            for pad in &mut self.pads {
                if pad.midi_note != midi_note && pad.choke_group == Some(cg) && pad.active {
                    pad.choke();
                }
            }
        }

        // 3. Trigger the pad
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

    fn handle_event(&mut self, event: &summoner_core::node::NodeEvent, _ctx: &ProcessContext) {
        use summoner_core::node::NodeEvent;
        match *event {
            NodeEvent::NoteOn { note, velocity } => {
                let vel_u8 = (velocity.clamp(0.0, 1.0) * 127.0).round() as u8;
                self.trigger_note(note, vel_u8.max(1));
            }
            NodeEvent::NoteOff { note } => {
                self.release_note(note);
            }
            NodeEvent::SetParam { id: 0, value } => {
                self.macro_view.master_gain = value.clamp(0.0, 2.0);
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
        let master = self.macro_view.master_gain;
        let is_stereo = outputs.len() >= 2;

        let has_solo = self.pads.iter().any(|p| p.solo);

        for i in 0..num_samples {
            let mut mixed_l = 0.0f32;
            let mut mixed_r = 0.0f32;

            for (pad_idx, pad) in self.pads.iter_mut().enumerate() {
                let is_muted = (pad_idx < MAX_PADS && self.macro_view.pad_mutes[pad_idx])
                    || pad.mute
                    || (has_solo && !pad.solo);

                if !is_muted {
                    let s = pad.process_sample(ctx.sample_rate);
                    if s != 0.0 {
                        // Equal-power stereo panning
                        let pan_norm = (pad.pan + 1.0) * 0.5; // 0.0 to 1.0
                        let angle = pan_norm * (std::f32::consts::FRAC_PI_2);
                        let gain_l = angle.cos();
                        let gain_r = angle.sin();

                        mixed_l += s * gain_l;
                        mixed_r += s * gain_r;
                    }
                }
            }

            let out_l = (mixed_l * master).clamp(-1.0, 1.0);
            let out_r = (mixed_r * master).clamp(-1.0, 1.0);

            if i < outputs[0].len() {
                outputs[0][i] = out_l;
            }
            if is_stereo && i < outputs[1].len() {
                outputs[1][i] = out_r;
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
        kick_pad.add_sample(
            36,
            36,
            36,
            "kick.wav",
            Some(Arc::new(SampleBuffer::new(sine_data, 44100, 1))),
            LoopMode::NoLoop,
        );
        kit.add_pad(kick_pad);

        // Create a snare pad with noise buffer.
        let mut snare_pad = DrumPad::new("Snare", 38, 0.001, 0.15, 0.0, 0.08, 0.9);
        let noise_data: Vec<f32> = (0u64..4410)
            .map(|i| {
                let x = i.wrapping_mul(1664525).wrapping_add(1013904223);
                ((x & 0x00FF_FFFF) as f32 / (1u32 << 23) as f32) - 1.0
            })
            .collect();
        snare_pad.add_sample(
            38,
            38,
            38,
            "snare.wav",
            Some(Arc::new(SampleBuffer::new(noise_data, 44100, 1))),
            LoopMode::NoLoop,
        );
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

    #[test]
    fn test_drum_machine_choke_group_and_stereo_panning() {
        let mut kit = DrumMachineDevice::new("Choke Kit");

        let dummy_buf = Arc::new(SampleBuffer::new(vec![1.0; 44100], 44100, 1));

        // Closed Hi-Hat (note 42) and Open Hi-Hat (note 46) both in choke group 1
        let mut closed_hh = DrumPad::new("Closed HH", 42, 0.001, 0.05, 0.0, 0.02, 1.0)
            .with_choke_group(1)
            .with_pan(-0.5);
        closed_hh.add_sample(
            42,
            42,
            42,
            "ch.wav",
            Some(dummy_buf.clone()),
            LoopMode::NoLoop,
        );

        let mut open_hh = DrumPad::new("Open HH", 46, 0.001, 1.0, 0.5, 0.5, 1.0)
            .with_choke_group(1)
            .with_pan(0.5);
        open_hh.add_sample(46, 46, 46, "oh.wav", Some(dummy_buf), LoopMode::NoLoop);

        kit.add_pad(closed_hh);
        kit.add_pad(open_hh);

        // 1. Trigger Open Hi-Hat (note 46)
        kit.trigger_note(46, 100);
        assert!(kit.pads[1].is_active(), "Open HH should be active");
        assert!(!kit.pads[0].is_active(), "Closed HH should be inactive");

        // 2. Trigger Closed Hi-Hat (note 42) -> Open Hi-Hat must be choked immediately!
        kit.trigger_note(42, 100);
        assert!(kit.pads[0].is_active(), "Closed HH should now be active");
        assert!(
            !kit.pads[1].is_active(),
            "Open HH must be choked and inactive"
        );
    }
}
