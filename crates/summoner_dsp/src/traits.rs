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

//! Standard SignalProcessor trait for atomic DSP primitives.

use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, NodeEvent, ProcessContext};

/// Unified interface for zero-allocation atomic DSP primitives.
pub trait SignalProcessor: Send {
    /// Return human-readable identifier for this processor node.
    fn name(&self) -> &str;

    /// Process a block of audio/control input slices into output slices.
    /// MUST NOT perform heap allocations (`malloc`/`free`) or block on locks.
    fn process_block(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    );

    /// Receive a control event before the next block. Default: ignore.
    fn handle_event(&mut self, _event: &NodeEvent, _ctx: &ProcessContext) {}
}

/// Adapter converting any `SignalProcessor` into an `AudioNode`.
#[derive(Debug)]
pub struct ProcessorNodeAdapter<T: SignalProcessor> {
    pub processor: T,
}

impl<T: SignalProcessor> ProcessorNodeAdapter<T> {
    pub fn new(processor: T) -> Self {
        Self { processor }
    }
}

impl<T: SignalProcessor> AudioNode for ProcessorNodeAdapter<T> {
    fn name(&self) -> &str {
        self.processor.name()
    }

    fn process(&mut self, input: &[&[Sample]], output: &mut [&mut [Sample]], ctx: &ProcessContext) {
        self.processor.process_block(input, output, ctx);
    }

    fn handle_event(&mut self, event: &NodeEvent, ctx: &ProcessContext) {
        self.processor.handle_event(event, ctx);
    }
}

/// A processor whose pitch can be set directly in Hz.
pub trait Pitched: SignalProcessor {
    fn set_frequency(&mut self, hz: f32);
}

macro_rules! impl_pitched {
    ($($t:ty),*) => {$(
        impl Pitched for $t {
            fn set_frequency(&mut self, hz: f32) {
                self.frequency = hz;
            }
        }
    )*};
}
impl_pitched!(
    crate::oscillators::OscSaw,
    crate::oscillators::OscPulse,
    crate::oscillators::OscTriangle,
    crate::oscillators::OscWavetable
);

/// Adapter making a pitched oscillator note-driven: `NoteOn` sets pitch and
/// level, `NoteOff` closes the gate. Starts open so a bare oscillator sounds.
#[derive(Debug)]
pub struct VoiceNode<T: Pitched> {
    pub processor: T,
    pub amplitude: f32,
}

impl<T: Pitched> VoiceNode<T> {
    pub fn new(processor: T) -> Self {
        Self {
            processor,
            amplitude: 1.0,
        }
    }
}

impl<T: Pitched> AudioNode for VoiceNode<T> {
    fn name(&self) -> &str {
        self.processor.name()
    }

    fn process(&mut self, input: &[&[Sample]], output: &mut [&mut [Sample]], ctx: &ProcessContext) {
        self.processor.process_block(input, output, ctx);
    }

    fn handle_event(&mut self, event: &NodeEvent, ctx: &ProcessContext) {
        match *event {
            NodeEvent::NoteOn { note, velocity } => {
                self.processor.set_frequency(ctx.note_to_hz(note as i32));
                self.amplitude = velocity;
            }
            NodeEvent::NoteOff { .. } => self.amplitude = 0.0,
            NodeEvent::SetParam { id: 0, value } => self.processor.set_frequency(value),
            NodeEvent::SetParam { .. } => {}
        }
        self.processor.handle_event(event, ctx);
    }
}

#[cfg(test)]
mod voice_tests {
    use super::*;
    use crate::oscillators::OscSaw;

    #[test]
    fn test_voice_node_gates_and_tunes() {
        let ctx = ProcessContext::new(48000, 120.0, 0);
        let mut v = VoiceNode::new(OscSaw::new(100.0));
        v.handle_event(
            &NodeEvent::NoteOn {
                note: 69,
                velocity: 1.0,
            },
            &ctx,
        );
        assert!((v.processor.frequency - 440.0).abs() < 1e-3);
        let mut buf = vec![0.0f32; 256];
        v.process(&[], &mut [&mut buf[..]], &ctx);
        assert!(buf.iter().any(|s| s.abs() > 0.1));
        v.handle_event(&NodeEvent::NoteOff { note: 69 }, &ctx);
        v.process(&[], &mut [&mut buf[..]], &ctx);
        assert!(buf.iter().all(|s| *s == 0.0));
    }
}
