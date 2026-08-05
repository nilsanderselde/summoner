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
    pub active_notes: Vec<u8>,
}

impl HarmonicContext {
    pub fn new(tuning: EdoTuning, root_note: u16, scale: Scale) -> Self {
        Self {
            tuning,
            root_note,
            scale,
            active_notes: Vec::new(),
        }
    }

    pub fn push_note_on(&mut self, note: u8) {
        if !self.active_notes.contains(&note) {
            self.active_notes.push(note);
        }
    }

    pub fn push_note_off(&mut self, note: u8) {
        self.active_notes.retain(|&n| n != note);
    }

    pub fn analyze_active_chord(&self) -> String {
        if self.active_notes.is_empty() {
            return "Silence".to_string();
        }
        let mut pcs: Vec<u8> = self.active_notes.iter().map(|n| n % 12).collect();
        pcs.sort();
        pcs.dedup();

        if pcs == vec![0, 4, 7] || pcs.contains(&0) && pcs.contains(&4) && pcs.contains(&7) {
            "C Major".to_string()
        } else if pcs == vec![0, 3, 7] || pcs.contains(&0) && pcs.contains(&3) && pcs.contains(&7) {
            "C Minor".to_string()
        } else {
            format!("Chord({:?})", pcs)
        }
    }

    pub fn suggest_next_chord_notes(&self) -> Vec<u8> {
        let current_label = self.analyze_active_chord();
        if current_label.contains("Major") || current_label == "Silence" {
            vec![67, 71, 74] // G Major (V)
        } else {
            vec![60, 64, 67] // C Major (I)
        }
    }

    /// Resolve note index to frequency in Hz using current tuning context.
    ///
    /// # Examples
    ///
    /// ```
    /// use summoner_harmony::bus::HarmonicContext;
    /// use summoner_harmony::edo::EdoTuning;
    /// use summoner_harmony::scale::Scale;
    ///
    /// let bus = HarmonicContext::new(EdoTuning::default(), 60, Scale::major_12_tet());
    /// let freq = bus.freq_from_note(69.0);
    /// assert!((freq - 440.0).abs() < 1e-3);
    /// ```
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
            active_notes: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_active_notes_and_suggestions() {
        let mut ctx = HarmonicContext::default();
        ctx.push_note_on(60);
        ctx.push_note_on(64);
        ctx.push_note_on(67);

        assert_eq!(ctx.analyze_active_chord(), "C Major");
        let suggestion = ctx.suggest_next_chord_notes();
        assert!(!suggestion.is_empty());
    }
}
