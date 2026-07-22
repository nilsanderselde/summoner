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

//! Arbitrary N-EDO (Equal Division of the Octave) microtonal tuning systems.

/// N-EDO tuning system configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct EdoTuning {
    /// Number of equal divisions of the octave (e.g., 12 for 12-TET, 19, 31, 53).
    pub divisions: u16,
    /// Reference frequency in Hz (e.g. 440.0 for A4).
    pub reference_freq: f64,
    /// Reference note index corresponding to reference frequency (e.g. 69.0 for A4).
    pub reference_note: f64,
}

impl EdoTuning {
    /// Standard 12-TET (12 Equal Division of Octave) tuning.
    pub fn standard_12_tet() -> Self {
        Self {
            divisions: 12,
            reference_freq: 440.0,
            reference_note: 69.0,
        }
    }

    /// Create custom N-EDO tuning system.
    pub fn new(divisions: u16, reference_freq: f64, reference_note: f64) -> Self {
        assert!(divisions > 0, "divisions must be greater than 0");
        Self {
            divisions,
            reference_freq,
            reference_note,
        }
    }

    /// Calculate frequency in Hz for a given (possibly fractional) note index.
    pub fn note_to_freq(&self, note: f64) -> f64 {
        let octave_exponent = (note - self.reference_note) / (self.divisions as f64);
        self.reference_freq * 2.0_f64.powf(octave_exponent)
    }

    /// Calculate (possibly fractional) note index for a given frequency in Hz.
    pub fn freq_to_note(&self, freq: f64) -> f64 {
        if freq <= 0.0 {
            return 0.0;
        }
        let octave_exponent = (freq / self.reference_freq).log2();
        self.reference_note + octave_exponent * (self.divisions as f64)
    }
}

impl Default for EdoTuning {
    fn default() -> Self {
        Self::standard_12_tet()
    }
}
