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

//! Musical scale definitions and pitch quantization / snapping helpers.

use crate::edo::EdoTuning;

/// Scale step interval set defined relative to octave divisions.
#[derive(Debug, Clone, PartialEq)]
pub struct Scale {
    pub name: String,
    pub degrees: Vec<u16>,
}

impl Scale {
    /// 12-TET Ionian Major scale.
    pub fn major_12_tet() -> Self {
        Self {
            name: "Major".to_string(),
            degrees: vec![0, 2, 4, 5, 7, 9, 11],
        }
    }

    /// 12-TET Aeolian Minor scale.
    pub fn minor_12_tet() -> Self {
        Self {
            name: "Minor".to_string(),
            degrees: vec![0, 2, 3, 5, 7, 8, 10],
        }
    }

    /// Snap continuous note index to nearest valid scale note.
    pub fn snap_note(&self, note: f64, root: u16, tuning: &EdoTuning) -> f64 {
        if self.degrees.is_empty() {
            return note;
        }

        let divisions = tuning.divisions as f64;
        let rounded_note = note.round();
        let octave = (rounded_note / divisions).floor();
        let pitch_class = ((rounded_note % divisions) + divisions) % divisions;

        // Find closest degree in scale
        let mut min_diff = f64::MAX;
        let mut best_degree = self.degrees[0];

        for &deg in &self.degrees {
            let scale_pitch = ((deg + root) as f64) % divisions;
            let diff = (pitch_class - scale_pitch).abs();
            let cyclic_diff = diff.min(divisions - diff);
            if cyclic_diff < min_diff {
                min_diff = cyclic_diff;
                best_degree = deg;
            }
        }

        let snapped_pitch_class = ((best_degree + root) as f64) % divisions;
        octave * divisions + snapped_pitch_class
    }
}
