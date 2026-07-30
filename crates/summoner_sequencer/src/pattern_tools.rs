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

//! Pattern editing and MIDI export/import tools for sequence manipulation (Steps 616-625).

use summoner_project::schema::{SequenceConfig, TrackerStepConfig};
use std::fs;
use std::path::Path;

/// Randomizes sequence steps using a deterministic pseudo-random seed.
pub fn randomize_pattern(sequence: &mut SequenceConfig, seed: u64, density: f32, note_range: (u8, u8)) {
    let mut state = seed.wrapping_add(12345);
    let min_note = note_range.0.min(note_range.1) as f64;
    let max_note = note_range.0.max(note_range.1) as f64;
    let range = (max_note - min_note).max(1.0);

    for step in sequence.steps.iter_mut() {
        // LCG PRNG step
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let rand_val = (state >> 33) as f32 / (u32::MAX as f32);

        let active = rand_val < density;
        step.active = active;
        step.muted = false;

        if active {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let note_frac = (state >> 33) as f64 / (u32::MAX as f64);
            step.note = (min_note + note_frac * range).round();

            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let vel_frac = (state >> 33) as f32 / (u32::MAX as f32);
            step.velocity = (0.3 + vel_frac * 0.7).clamp(0.1, 1.0);

            step.gate = 0.5;
            step.probability = 1.0;
        } else {
            step.gate = 0.0;
        }
    }
}

/// Filter/proportionally reduces active steps based on density threshold (0.0 to 1.0).
pub fn apply_pattern_density(sequence: &mut SequenceConfig, density: f32) {
    let active_indices: Vec<usize> = sequence.steps.iter().enumerate().filter(|(_, s)| s.active && !s.muted).map(|(i, _)| i).collect();
    if active_indices.is_empty() {
        return;
    }

    let target_count = ((active_indices.len() as f32) * density.clamp(0.0, 1.0)).round() as usize;
    for (rank, &idx) in active_indices.iter().enumerate() {
        if rank >= target_count {
            sequence.steps[idx].active = false;
        }
    }
}

/// Quantizes velocities of all active steps to the nearest value in `levels`.
pub fn quantize_velocities(sequence: &mut SequenceConfig, levels: &[f32]) {
    if levels.is_empty() {
        return;
    }
    for step in &mut sequence.steps {
        if step.active && !step.muted {
            let mut best_level = levels[0];
            let mut min_diff = (step.velocity - levels[0]).abs();
            for &level in levels {
                let diff = (step.velocity - level).abs();
                if diff < min_diff {
                    min_diff = diff;
                    best_level = level;
                }
            }
            step.velocity = best_level;
        }
    }
}

/// Sets the step resolution division (e.g. 0.25 for 1/4, 0.125 for 1/8) and optional triplet mode.
pub fn set_pattern_resolution(sequence: &mut SequenceConfig, division: f64, is_triplet: bool) {
    let base_div = division.max(0.001);
    sequence.step_division = if is_triplet { base_div * (2.0 / 3.0) } else { base_div };
}

/// Sets the length (number of steps) of a pattern, truncating or extending with default inactive steps.
pub fn set_pattern_length(sequence: &mut SequenceConfig, new_len: usize) {
    let target_len = new_len.max(1);
    if sequence.steps.len() > target_len {
        sequence.steps.truncate(target_len);
    } else {
        while sequence.steps.len() < target_len {
            sequence.steps.push(TrackerStepConfig {
                active: false,
                gate: 0.0,
                ..Default::default()
            });
        }
    }
}

/// Encodes a SequenceConfig as standard MIDI byte stream (Type 0 SMF file).
pub fn export_pattern_to_midi_bytes(sequence: &SequenceConfig, bpm: f64) -> Vec<u8> {
    let ticks_per_quarter = 480u16;
    let mut track_data = Vec::new();

    // Set Tempo event (Meta event 0x51, 3 bytes)
    let us_per_quarter = (60_000_000.0 / bpm.max(1.0)).round() as u32;
    track_data.extend_from_slice(&[0x00, 0xFF, 0x51, 0x03]);
    track_data.push((us_per_quarter >> 16) as u8);
    track_data.push((us_per_quarter >> 8) as u8);
    track_data.push(us_per_quarter as u8);

    let mut current_tick: u64 = 0;

    for (idx, step) in sequence.steps.iter().enumerate() {
        if !step.active || step.muted || step.gate <= 0.0 {
            continue;
        }

        let start_beat = idx as f64 * sequence.step_division;
        let duration_beats = step.gate as f64 * sequence.step_division;

        let start_tick = (start_beat * ticks_per_quarter as f64).round() as u64;
        let note_len_ticks = (duration_beats * ticks_per_quarter as f64).round() as u64;
        let end_tick = start_tick + note_len_ticks.max(1);

        let note = (step.note.round() as u8).clamp(0, 127);
        let vel = ((step.velocity * 127.0).round() as u8).clamp(1, 127);

        // Note On
        let delta_on = start_tick.saturating_sub(current_tick);
        write_varlen(&mut track_data, delta_on);
        track_data.push(0x90); // Note On channel 0
        track_data.push(note);
        track_data.push(vel);
        current_tick = start_tick;

        // Note Off
        let delta_off = end_tick.saturating_sub(current_tick);
        write_varlen(&mut track_data, delta_off);
        track_data.push(0x80); // Note Off channel 0
        track_data.push(note);
        track_data.push(0);
        current_tick = end_tick;
    }

    // End of Track meta event
    track_data.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]);

    let mut midi = Vec::new();
    // Header Chunk: MThd
    midi.extend_from_slice(b"MThd");
    midi.extend_from_slice(&6u32.to_be_bytes()); // Chunk size = 6
    midi.extend_from_slice(&0u16.to_be_bytes()); // Format 0
    midi.extend_from_slice(&1u16.to_be_bytes()); // 1 track
    midi.extend_from_slice(&ticks_per_quarter.to_be_bytes());

    // Track Chunk: MTrk
    midi.extend_from_slice(b"MTrk");
    midi.extend_from_slice(&(track_data.len() as u32).to_be_bytes());
    midi.extend_from_slice(&track_data);

    midi
}

/// Writes pattern as standard MIDI file to disk.
pub fn export_pattern_to_midi_file(sequence: &SequenceConfig, bpm: f64, path: &Path) -> Result<(), String> {
    let bytes = export_pattern_to_midi_bytes(sequence, bpm);
    fs::write(path, bytes).map_err(|e| e.to_string())
}

/// Imports standard MIDI byte stream into a SequenceConfig pattern.
pub fn import_pattern_from_midi_bytes(bytes: &[u8]) -> Result<SequenceConfig, String> {
    if bytes.len() < 14 || &bytes[0..4] != b"MThd" {
        return Err("Invalid MIDI file header".to_string());
    }

    let ticks_per_quarter = u16::from_be_bytes([bytes[12], bytes[13]]) as f64;
    let ticks_per_quarter = if ticks_per_quarter == 0.0 { 480.0 } else { ticks_per_quarter };

    let mut pos = 14;
    while pos + 8 <= bytes.len() {
        if &bytes[pos..pos + 4] == b"MTrk" {
            let track_len = u32::from_be_bytes([bytes[pos+4], bytes[pos+5], bytes[pos+6], bytes[pos+7]]) as usize;
            let track_data = &bytes[pos + 8..(pos + 8 + track_len).min(bytes.len())];

            let mut steps = vec![TrackerStepConfig { active: false, gate: 0.0, ..Default::default() }; 16];
            let mut curr_tick = 0u64;
            let mut cursor = 0;

            while cursor < track_data.len() {
                let (delta, len) = read_varlen(&track_data[cursor..]);
                cursor += len;
                curr_tick += delta as u64;

                if cursor >= track_data.len() { break; }
                let status = track_data[cursor];
                cursor += 1;

                if status == 0xFF {
                    // Meta event
                    if cursor < track_data.len() {
                        let _meta_type = track_data[cursor];
                        cursor += 1;
                        let (meta_len, vlen) = read_varlen(&track_data[cursor..]);
                        cursor += vlen + meta_len;
                    }
                } else if status >= 0x80 && status <= 0x9F {
                    // Note On / Note Off
                    if cursor + 2 <= track_data.len() {
                        let note = track_data[cursor];
                        let vel = track_data[cursor + 1];
                        cursor += 2;

                        let is_on = (status & 0xF0) == 0x90 && vel > 0;
                        if is_on {
                            let beat = curr_tick as f64 / ticks_per_quarter;
                            let step_idx = (beat / 0.25).round() as usize;
                            if step_idx >= steps.len() {
                                steps.resize(step_idx + 1, TrackerStepConfig { active: false, gate: 0.0, ..Default::default() });
                            }
                            steps[step_idx] = TrackerStepConfig {
                                note: note as f64,
                                velocity: vel as f32 / 127.0,
                                gate: 0.5,
                                probability: 1.0,
                                ratchet: 1,
                                micro_shift: 0,
                                swing: 0.0,
                                pan: 0.0,
                                pitch_offset: 0.0,
                                active: true,
                                muted: false,
                            };
                        }
                    }
                } else if status >= 0xA0 && status <= 0xDF {
                    cursor += 2;
                } else if status >= 0xE0 && status <= 0xEF {
                    cursor += 2;
                }
            }

            return Ok(SequenceConfig {
                start_beat: 0.0,
                step_division: 0.25,
                clip_color: None,
                clip_name: Some("Imported MIDI".to_string()),
                name: "Imported MIDI".to_string(),
                is_unique: true,
                steps,
                ..Default::default()
            });
        }
        pos += 1;
    }

    Err("No track chunk found in MIDI file".to_string())
}

/// Reads standard MIDI file from disk into a SequenceConfig pattern.
pub fn import_pattern_from_midi_file(path: &Path) -> Result<SequenceConfig, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    import_pattern_from_midi_bytes(&bytes)
}

fn write_varlen(buf: &mut Vec<u8>, mut val: u64) {
    let mut buffer = [0u8; 9];
    let mut count = 0;
    loop {
        buffer[count] = (val & 0x7F) as u8;
        val >>= 7;
        count += 1;
        if val == 0 { break; }
    }
    for i in (0..count).rev() {
        let b = if i > 0 { buffer[i] | 0x80 } else { buffer[i] };
        buf.push(b);
    }
}

fn read_varlen(buf: &[u8]) -> (usize, usize) {
    let mut result = 0usize;
    let mut count = 0usize;
    for &byte in buf {
        count += 1;
        result = (result << 7) | (byte & 0x7F) as usize;
        if (byte & 0x80) == 0 || count >= 4 { break; }
    }
    (result, count)
}

/// Transpose all active steps in a sequence to match the specified scale and root note.
pub fn transpose_sequence_to_scale(sequence: &mut SequenceConfig, root_note: u8, scale_type: &str) {
    use summoner_harmony::edo::EdoTuning;
    use summoner_harmony::scale::Scale;

    let scale = Scale::get_scale_by_name(scale_type);
    let tuning = EdoTuning::default();

    for step in &mut sequence.steps {
        if step.active {
            step.note = scale.snap_note(step.note, root_note as u16, &tuning);
        }
    }
}

/// Detect chord name from a slice of MIDI note pitches.
pub fn detect_chord_name_from_notes(notes: &[f64]) -> String {
    if notes.is_empty() {
        return "Silence".to_string();
    }
    if notes.len() == 1 {
        let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
        let pc = (notes[0].round() as i32).rem_euclid(12) as usize;
        let octave = ((notes[0].round() as i32) / 12) - 1;
        return format!("{}{}", note_names[pc], octave);
    }

    let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let mut pcs: Vec<i32> = notes.iter().map(|&n| (n.round() as i32).rem_euclid(12)).collect();
    pcs.sort_unstable();
    pcs.dedup();

    if pcs.len() == 2 {
        let diff = (pcs[1] - pcs[0]).rem_euclid(12);
        let interval_name = match diff {
            1 => "m2", 2 => "M2", 3 => "m3", 4 => "M3", 5 => "P4",
            6 => "TT", 7 => "P5", 8 => "m6", 9 => "M6", 10 => "m7", 11 => "M7",
            _ => "Unison",
        };
        return format!("{}-{} ({})", note_names[pcs[0] as usize], note_names[pcs[1] as usize], interval_name);
    }

    // Try each pitch as candidate root note
    for &root in &pcs {
        let rel_pcs: Vec<i32> = pcs.iter().map(|&pc| (pc - root).rem_euclid(12)).collect();
        let mut rel_set: std::collections::HashSet<i32> = rel_pcs.into_iter().collect();
        rel_set.insert(0); // Root is 0

        let root_name = note_names[root as usize];

        if rel_set.contains(&4) && rel_set.contains(&7) && rel_set.contains(&11) {
            return format!("{}maj7", root_name);
        }
        if rel_set.contains(&3) && rel_set.contains(&7) && rel_set.contains(&10) {
            return format!("{}m7", root_name);
        }
        if rel_set.contains(&4) && rel_set.contains(&7) && rel_set.contains(&10) {
            return format!("{}7", root_name);
        }
        if rel_set.contains(&4) && rel_set.contains(&7) {
            return format!("{}maj", root_name);
        }
        if rel_set.contains(&3) && rel_set.contains(&7) {
            return format!("{}m", root_name);
        }
        if rel_set.contains(&3) && rel_set.contains(&6) {
            return format!("{}dim", root_name);
        }
        if rel_set.contains(&4) && rel_set.contains(&8) {
            return format!("{}aug", root_name);
        }
        if rel_set.contains(&5) && rel_set.contains(&7) {
            return format!("{}sus4", root_name);
        }
        if rel_set.contains(&2) && rel_set.contains(&7) {
            return format!("{}sus2", root_name);
        }
    }

    format!("Chord({:?})", pcs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_randomize_pattern_reproducible() {
        let mut seq1 = SequenceConfig {
            steps: vec![TrackerStepConfig::default(); 16],
            ..Default::default()
        };
        let mut seq2 = seq1.clone();

        randomize_pattern(&mut seq1, 42, 0.5, (48, 72));
        randomize_pattern(&mut seq2, 42, 0.5, (48, 72));

        assert_eq!(seq1.steps, seq2.steps, "Same seed should generate identical randomized pattern");
        assert!(seq1.steps.iter().any(|s| s.active));
    }

    #[test]
    fn test_apply_pattern_density() {
        let mut seq = SequenceConfig {
            steps: vec![
                TrackerStepConfig { active: true, ..Default::default() },
                TrackerStepConfig { active: true, ..Default::default() },
                TrackerStepConfig { active: true, ..Default::default() },
                TrackerStepConfig { active: true, ..Default::default() },
            ],
            ..Default::default()
        };

        apply_pattern_density(&mut seq, 0.5);
        let active_count = seq.steps.iter().filter(|s| s.active).count();
        assert_eq!(active_count, 2, "Density 0.5 should reduce 4 active steps to 2");
    }

    #[test]
    fn test_quantize_velocities() {
        let mut seq = SequenceConfig {
            steps: vec![
                TrackerStepConfig { active: true, velocity: 0.23, ..Default::default() },
                TrackerStepConfig { active: true, velocity: 0.88, ..Default::default() },
            ],
            ..Default::default()
        };

        let levels = [0.2, 0.5, 0.8, 1.0];
        quantize_velocities(&mut seq, &levels);
        assert_eq!(seq.steps[0].velocity, 0.2);
        assert_eq!(seq.steps[1].velocity, 0.8);
    }

    #[test]
    fn test_set_pattern_length_and_resolution() {
        let mut seq = SequenceConfig {
            steps: vec![TrackerStepConfig::default(); 4],
            ..Default::default()
        };

        set_pattern_length(&mut seq, 8);
        assert_eq!(seq.steps.len(), 8);

        set_pattern_resolution(&mut seq, 0.25, true);
        assert!((seq.step_division - 0.16666666666666666).abs() < 1e-5);
    }

    #[test]
    fn test_export_import_midi_round_trip() {
        let seq_orig = SequenceConfig {
            start_beat: 0.0,
            step_division: 0.25,
            clip_color: None,
            clip_name: Some("Test Midi".to_string()),
            name: "Test Midi".to_string(),
            is_unique: true,
            steps: vec![
                TrackerStepConfig { note: 60.0, velocity: 0.8, gate: 0.5, active: true, ..Default::default() },
                TrackerStepConfig { note: 64.0, velocity: 0.9, gate: 0.5, active: true, ..Default::default() },
            ],
            ..Default::default()
        };

        let midi_bytes = export_pattern_to_midi_bytes(&seq_orig, 120.0);
        assert!(!midi_bytes.is_empty());

        let seq_imported = import_pattern_from_midi_bytes(&midi_bytes).expect("MIDI import should succeed");
        assert!(seq_imported.steps.len() >= 2);
        assert_eq!(seq_imported.steps[0].note, 60.0);
        assert_eq!(seq_imported.steps[1].note, 64.0);
    }
}
