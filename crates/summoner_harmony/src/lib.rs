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

//! Global Harmonic Bus & N-EDO tuning systems for Summoner DAW.

pub mod bus;
pub mod cadence;
pub mod edo;
pub mod scale;

pub use bus::HarmonicContext;
pub use cadence::{CadenceEngine, CadenceType, Chord};
pub use edo::EdoTuning;
pub use scale::Scale;

pub mod scl;
pub mod kbm;
pub use scl::SclTuning;
pub use kbm::KbmMapping;


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_12_tet_pitch_frequency() {
        let tuning = EdoTuning::standard_12_tet();
        let freq_a4 = tuning.note_to_freq(69.0);
        assert!((freq_a4 - 440.0).abs() < 1e-6);

        let freq_c5 = tuning.note_to_freq(72.0);
        assert!((freq_c5 - 523.2511306011972).abs() < 1e-4);
    }

    #[test]
    fn test_microtonal_19_edo() {
        let tuning = EdoTuning::new(19, 440.0, 69.0);
        let note = tuning.freq_to_note(440.0);
        assert_eq!(note, 69.0);

        let octave_above = tuning.note_to_freq(69.0 + 19.0);
        assert!((octave_above - 880.0).abs() < 1e-6);
    }

    #[test]
    fn test_scale_snapping() {
        let ctx = HarmonicContext::default(); // 12-TET C Major
        let snapped = ctx.snap_to_scale(60.1); // C4 = 60
        assert_eq!(snapped, 60.0);
    }
}
