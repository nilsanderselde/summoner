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

use serde::{Deserialize, Serialize};

/// Represents a single event step in the unified pattern sequencer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternStep {
    /// Note or event value (e.g., pitch or parameter value)
    pub note: f32,
    /// Velocity or amplitude (0.0 to 1.0)
    pub velocity: f32,
    /// Duration in beats or ticks
    pub gate: f32,
    /// Chance this step triggers (0.0 to 1.0)
    pub probability: f32,
    /// Number of times the step repeats within its duration (glitch rolls)
    pub ratchet: u32,
    /// Timing offset in ticks (swing / push / pull)
    pub micro_shift: i32,
    /// Whether this step is active
    pub active: bool,
}

impl Default for PatternStep {
    fn default() -> Self {
        Self {
            note: 0.0,
            velocity: 1.0,
            gate: 0.25,
            probability: 1.0,
            ratchet: 1,
            micro_shift: 0,
            active: true,
        }
    }
}

/// A unified pattern sequence, representing MIDI notes, automation curves, or step data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternClip {
    pub name: String,
    pub length_beats: f32,
    pub steps: Vec<PatternStep>,
}

impl PatternClip {
    pub fn new(name: &str, length_beats: f32) -> Self {
        Self {
            name: name.to_string(),
            length_beats,
            steps: Vec::new(),
        }
    }

    pub fn add_step(&mut self, step: PatternStep) {
        self.steps.push(step);
    }
}
