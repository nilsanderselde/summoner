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
use summoner_core::node::{AudioNode, ProcessContext};

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
}
