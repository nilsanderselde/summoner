// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Chromatic Tuner pitch detection utility (Step 656).

#[derive(Debug, Clone, PartialEq)]
pub struct TunerResult {
    pub note_name: String,
    pub midi_note: u8,
    pub pitch_hz: f32,
    pub cents_dev: f32,
}

const NOTE_NAMES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

/// Estimate fundamental pitch (Hz), nearest MIDI note, note name, and cents deviation
/// using time-domain autocorrelation over input buffer.
pub fn detect_chromatic_pitch(samples: &[f32], sample_rate: f32) -> Option<TunerResult> {
    if samples.len() < 128 || sample_rate <= 0.0 {
        return None;
    }

    let min_lag = (sample_rate / 2000.0) as usize; // Max freq ~2000 Hz
    let max_lag = ((sample_rate / 40.0) as usize).min(samples.len() / 2); // Min freq ~40 Hz

    if min_lag >= max_lag {
        return None;
    }

    let mut best_lag = 0;
    let mut best_corr = 0.0f32;
    let mut zero_lag_corr = 0.0f32;

    for i in 0..samples.len() / 2 {
        zero_lag_corr += samples[i] * samples[i];
    }

    if zero_lag_corr < 1e-5 {
        return None; // Signal too quiet
    }

    for lag in min_lag..=max_lag {
        let mut corr = 0.0f32;
        for i in 0..samples.len() / 2 {
            corr += samples[i] * samples[i + lag];
        }
        if corr > best_corr {
            best_corr = corr;
            best_lag = lag;
        }
    }

    let norm_corr = best_corr / zero_lag_corr;
    if norm_corr < 0.3 || best_lag == 0 {
        return None; // Unpitched or weak periodicity
    }

    let pitch_hz = sample_rate / (best_lag as f32);
    let note_num = 69.0 + 12.0 * (pitch_hz / 440.0).log2();
    let rounded_midi = note_num.round().clamp(0.0, 127.0) as u8;
    let cents_dev = (note_num - rounded_midi as f32) * 100.0;

    let pc = (rounded_midi % 12) as usize;
    let octave = (rounded_midi / 12) as i8 - 1;
    let note_name = format!("{}{}", NOTE_NAMES[pc], octave);

    Some(TunerResult {
        note_name,
        midi_note: rounded_midi,
        pitch_hz,
        cents_dev,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tuner_a440_detection() {
        let sr = 44100.0;
        let mut buf = vec![0.0f32; 2048];
        for i in 0..buf.len() {
            buf[i] = (2.0 * std::f32::consts::PI * 440.0 * (i as f32) / sr).sin();
        }

        let res = detect_chromatic_pitch(&buf, sr).expect("Should detect A440");
        assert_eq!(res.midi_note, 69);
        assert_eq!(res.note_name, "A4");
        assert!((res.pitch_hz - 440.0).abs() < 5.0);
        assert!(res.cents_dev.abs() < 15.0);
    }
}
