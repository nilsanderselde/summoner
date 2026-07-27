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

//! Polyphonic voice manager with oldest-note voice stealing and MPE event routing.

use crate::audio::Sample;
use crate::mpe::MpeEvent;
use crate::node::{AudioNode, ProcessContext};

pub trait PolyphonicVoice: AudioNode + Send {
    fn note_on(&mut self, note: u8, velocity: f32);
    fn note_off(&mut self, velocity: f32);
    fn set_pitch_bend(&mut self, semitones: f32);
    fn set_pressure(&mut self, pressure: f32);
    fn is_active(&self) -> bool;
}

#[derive(Debug, Clone, Copy, Default)]
struct VoiceSlotMetadata {
    channel: u8,
    age: u64,
    active: bool,
}

pub struct VoicePool<V: PolyphonicVoice, const MAX_VOICES: usize> {
    pub name: String,
    voices: Vec<V>,
    metadata: [VoiceSlotMetadata; MAX_VOICES],
    global_age: u64,
    // Buffer for mixing individual voice outputs
    voice_buffer: Vec<Vec<Sample>>,
    max_block_size: usize,
}

impl<V: PolyphonicVoice, const MAX_VOICES: usize> VoicePool<V, MAX_VOICES> {
    pub fn new(name: impl Into<String>, voice_factory: impl Fn() -> V, max_block_size: usize) -> Self {
        let mut voices = Vec::with_capacity(MAX_VOICES);
        for _ in 0..MAX_VOICES {
            voices.push(voice_factory());
        }

        Self {
            name: name.into(),
            voices,
            metadata: [VoiceSlotMetadata::default(); MAX_VOICES],
            global_age: 0,
            voice_buffer: vec![vec![0.0; max_block_size]; 2], // stereo mixing buffer
            max_block_size,
        }
    }

    pub fn dispatch_mpe(&mut self, event: MpeEvent) {
        self.global_age += 1;
        match event {
            MpeEvent::NoteOn { channel, note, velocity, .. } => {
                let target_slot = self
                    .find_inactive_slot()
                    .unwrap_or_else(|| self.find_oldest_slot());

                self.metadata[target_slot] = VoiceSlotMetadata {
                    channel,
                    age: self.global_age,
                    active: true,
                };
                self.voices[target_slot].note_on(note as u8, velocity);
            }
            MpeEvent::NoteOff { channel, release_velocity, .. } => {
                if let Some(slot) = self.find_slot_by_channel(channel) {
                    self.metadata[slot].active = false;
                    self.voices[slot].note_off(release_velocity);
                }
            }
            MpeEvent::PitchBend { voice_id, semitones, .. } => {
                let slot = (voice_id as usize) % MAX_VOICES;
                self.voices[slot].set_pitch_bend(semitones);
            }
            MpeEvent::Pressure { voice_id, pressure, .. } => {
                let slot = (voice_id as usize) % MAX_VOICES;
                self.voices[slot].set_pressure(pressure);
            }
            _ => {}
        }
    }

    fn find_inactive_slot(&self) -> Option<usize> {
        self.metadata
            .iter()
            .enumerate()
            .position(|(idx, meta)| !meta.active && !self.voices[idx].is_active())
    }

    fn find_oldest_slot(&self) -> usize {
        self.metadata
            .iter()
            .enumerate()
            .min_by_key(|(_, meta)| meta.age)
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    }

    fn find_slot_by_channel(&self, channel: u8) -> Option<usize> {
        self.metadata.iter().position(|meta| meta.active && meta.channel == channel)
    }

    pub fn active_voice_count(&self) -> usize {
        self.metadata.iter().filter(|m| m.active).count()
    }
}

impl<V: PolyphonicVoice, const MAX_VOICES: usize> AudioNode for VoicePool<V, MAX_VOICES> {
    fn name(&self) -> &str {
        &self.name
    }

    fn process(
        &mut self,
        inputs: &[&[Sample]],
        output: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        if output.is_empty() {
            return;
        }

        let num_samples = output[0].len().min(self.max_block_size);

        // Clear output buffer
        for ch in output.iter_mut() {
            for sample in ch.iter_mut().take(num_samples) {
                *sample = 0.0;
            }
        }

        // Process all active voices and accumulate
        for (idx, voice) in self.voices.iter_mut().enumerate() {
            if self.metadata[idx].active || voice.is_active() {
                // Clear scratch buffer
                for ch in self.voice_buffer.iter_mut() {
                    for s in ch.iter_mut().take(num_samples) {
                        *s = 0.0;
                    }
                }

                let mut voice_out: Vec<&mut [Sample]> = self
                    .voice_buffer
                    .iter_mut()
                    .map(|buf| &mut buf[..num_samples])
                    .collect();

                voice.process(inputs, &mut voice_out[..], ctx);

                // Accumulate voice output into main output
                #[allow(clippy::needless_range_loop)]
                for ch in 0..output.len().min(self.voice_buffer.len()) {
                    for i in 0..num_samples {
                        output[ch][i] += self.voice_buffer[ch][i];
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::Transport;

    struct TestVoice {
        active: bool,
        frequency: f32,
    }

    impl TestVoice {
        fn new() -> Self {
            Self { active: false, frequency: 440.0 }
        }
    }

    impl AudioNode for TestVoice {
        fn name(&self) -> &str { "TestVoice" }
        fn process(&mut self, _in: &[&[Sample]], out: &mut [&mut [Sample]], _ctx: &ProcessContext) {
            if self.active {
                for ch in out.iter_mut() {
                    for sample in ch.iter_mut() {
                        *sample = 0.5;
                    }
                }
            }
        }
    }

    impl PolyphonicVoice for TestVoice {
        fn note_on(&mut self, note: u8, _vel: f32) { self.active = true; self.frequency = note as f32 * 10.0; }
        fn note_off(&mut self, _vel: f32) { self.active = false; }
        fn set_pitch_bend(&mut self, _sb: f32) {}
        fn set_pressure(&mut self, _p: f32) {}
        fn is_active(&self) -> bool { self.active }
    }

    #[test]
    fn test_voice_pool_allocation_and_stealing() {
        let mut pool: VoicePool<TestVoice, 4> = VoicePool::new("TestPool", TestVoice::new, 256);

        assert_eq!(pool.active_voice_count(), 0);

        for note in 60..64 {
            pool.dispatch_mpe(MpeEvent::NoteOn {
                voice_id: note as u32,
                channel: (note - 60) as u8,
                note: note as f32,
                velocity: 0.8,
            });
        }

        assert_eq!(pool.active_voice_count(), 4);

        // Dispatched 5th note should steal oldest voice
        pool.dispatch_mpe(MpeEvent::NoteOn {
            voice_id: 65,
            channel: 4,
            note: 65.0,
            velocity: 0.8,
        });

        assert_eq!(pool.active_voice_count(), 4);

        let transport = Transport::new(44100, 120.0);
        let ctx = ProcessContext::from_transport(&transport);

        let mut out_l = vec![0.0f32; 256];
        let mut out_r = vec![0.0f32; 256];
        pool.process(&[], &mut [&mut out_l[..], &mut out_r[..]], &ctx);

        assert_eq!(out_l[0], 2.0); // 4 voices * 0.5 = 2.0
    }
}
