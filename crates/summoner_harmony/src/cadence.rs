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

//! Cadence-aware chord progression generation and dynamic harmonic tracking.

use crate::bus::HarmonicContext;

/// Cadence classification for harmonic progressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CadenceType {
    /// Authentic cadence (V -> I).
    Authentic,
    /// Plagal cadence (IV -> I).
    Plagal,
    /// Deceptive cadence (V -> vi).
    Deceptive,
    /// Half cadence (I -> V).
    Half,
}

/// Chord definition holding root degree, scale intervals, and descriptive name.
#[derive(Debug, Clone, PartialEq)]
pub struct Chord {
    pub name: String,
    pub root_step: i32,
    pub intervals: Vec<i32>,
}

impl Chord {
    pub fn new(name: impl Into<String>, root_step: i32, intervals: Vec<i32>) -> Self {
        Self {
            name: name.into(),
            root_step,
            intervals,
        }
    }

    /// Calculate frequency for each chord note within current harmonic context.
    pub fn frequencies(&self, context: &HarmonicContext) -> Vec<f64> {
        self.intervals
            .iter()
            .map(|interval| {
                let note_step = self.root_step + interval;
                context.scale_degree_freq(note_step as f64)
            })
            .collect()
    }
}

/// Cadence-aware progression engine querying `HarmonicContext`.
#[derive(Debug, Default)]
pub struct CadenceEngine;

impl CadenceEngine {
    pub fn new() -> Self {
        Self
    }

    /// Generate a full chord progression matching specified cadence type.
    pub fn generate_progression(context: &HarmonicContext, cadence: CadenceType) -> Vec<Chord> {
        let root = context.root_note as i32;

        match cadence {
            CadenceType::Authentic => vec![
                Chord::new("I", root, vec![0, 4, 7]),
                Chord::new("IV", root + 5, vec![0, 4, 7]),
                Chord::new("V", root + 7, vec![0, 4, 7]),
                Chord::new("I", root, vec![0, 4, 7]),
            ],
            CadenceType::Plagal => vec![
                Chord::new("I", root, vec![0, 4, 7]),
                Chord::new("IV", root + 5, vec![0, 4, 7]),
                Chord::new("I", root, vec![0, 4, 7]),
            ],
            CadenceType::Deceptive => vec![
                Chord::new("I", root, vec![0, 4, 7]),
                Chord::new("V", root + 7, vec![0, 4, 7]),
                Chord::new("vi", root + 9, vec![0, 3, 7]),
            ],
            CadenceType::Half => vec![
                Chord::new("I", root, vec![0, 4, 7]),
                Chord::new("ii", root + 2, vec![0, 3, 7]),
                Chord::new("V", root + 7, vec![0, 4, 7]),
            ],
        }
    }

    /// Suggest harmonically compatible next chords based on current chord.
    pub fn suggest_next_chords(current: &Chord, context: &HarmonicContext) -> Vec<Chord> {
        let root = context.root_note as i32;
        if current.name.contains("I") {
            vec![
                Chord::new("IV", root + 5, vec![0, 4, 7]),
                Chord::new("V", root + 7, vec![0, 4, 7]),
                Chord::new("vi", root + 9, vec![0, 3, 7]),
            ]
        } else if current.name.contains("V") {
            vec![
                Chord::new("I", root, vec![0, 4, 7]),
                Chord::new("vi", root + 9, vec![0, 3, 7]),
            ]
        } else {
            vec![
                Chord::new("V", root + 7, vec![0, 4, 7]),
                Chord::new("I", root, vec![0, 4, 7]),
            ]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::HarmonicContext;

    #[test]
    fn test_cadence_generation() {
        let ctx = HarmonicContext::default();
        let auth_prog = CadenceEngine::generate_progression(&ctx, CadenceType::Authentic);

        assert_eq!(auth_prog.len(), 4);
        assert_eq!(auth_prog[0].name, "I");
        assert_eq!(auth_prog[3].name, "I");

        let freqs = auth_prog[0].frequencies(&ctx);
        assert_eq!(freqs.len(), 3);
        assert!(freqs[0] > 0.0);
    }

    #[test]
    fn test_chord_suggestions() {
        let ctx = HarmonicContext::default();
        let tonic = Chord::new("I", 0, vec![0, 4, 7]);
        let suggestions = CadenceEngine::suggest_next_chords(&tonic, &ctx);

        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|c| c.name == "V"));
    }
}
