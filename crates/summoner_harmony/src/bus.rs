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

//! Global Harmonic Bus reactive context for pitch, tuning, and scale awareness.

use crate::edo::EdoTuning;
use crate::scale::Scale;

/// Global Harmonic Context providing microtonal tuning and scale quantization.
#[derive(Debug, Clone, PartialEq)]
pub struct HarmonicContext {
    pub tuning: EdoTuning,
    pub root_note: u16,
    pub scale: Scale,
}

impl HarmonicContext {
    pub fn new(tuning: EdoTuning, root_note: u16, scale: Scale) -> Self {
        Self {
            tuning,
            root_note,
            scale,
        }
    }

    /// Resolve note index to frequency in Hz using current tuning context.
    pub fn freq_from_note(&self, note: f64) -> f64 {
        self.tuning.note_to_freq(note)
    }

    /// Snap continuous note index to closest scale step in context.
    pub fn snap_to_scale(&self, note: f64) -> f64 {
        self.scale.snap_note(note, self.root_note, &self.tuning)
    }

    /// Calculate frequency for a scale degree with scale quantization.
    pub fn scale_degree_freq(&self, degree: f64) -> f64 {
        let snapped_note = self.snap_to_scale(degree);
        self.freq_from_note(snapped_note)
    }
}

impl Default for HarmonicContext {
    fn default() -> Self {
        Self {
            tuning: EdoTuning::standard_12_tet(),
            root_note: 0, // C
            scale: Scale::major_12_tet(),
        }
    }
}
