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

//! MIDI controller mapping, event filtering, velocity curves, and monitor utilities.

use std::collections::VecDeque;

/// MIDI velocity response curve options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VelocityCurve {
    Linear,
    Logarithmic,
    Exponential,
    Fixed(u8),
}

/// Transform incoming raw MIDI velocity (0-127) according to selected curve.
pub fn transform_velocity(input_vel: u8, curve: VelocityCurve) -> u8 {
    let norm = (input_vel.min(127) as f32) / 127.0;
    let res = match curve {
        VelocityCurve::Linear => norm,
        VelocityCurve::Logarithmic => norm.sqrt(),
        VelocityCurve::Exponential => norm * norm,
        VelocityCurve::Fixed(val) => return val.min(127),
    };
    (res * 127.0).clamp(0.0, 127.0).round() as u8
}

/// Type of MIDI event mapping target.
#[derive(Debug, Clone, PartialEq)]
pub enum MidiMappingType {
    CC(u8),
    Aftertouch,
    PitchBend,
}

/// Global MIDI controller mapping entry.
#[derive(Debug, Clone, PartialEq)]
pub struct MidiControllerMapping {
    pub channel: u8, // 0 = all channels, 1-16 = specific channel
    pub mapping_type: MidiMappingType,
    pub target_param_id: String,
    pub min_val: f32,
    pub max_val: f32,
}

impl MidiControllerMapping {
    pub fn new(channel: u8, mapping_type: MidiMappingType, target_param_id: impl Into<String>, min_val: f32, max_val: f32) -> Self {
        Self {
            channel,
            mapping_type,
            target_param_id: target_param_id.into(),
            min_val,
            max_val,
        }
    }

    /// Map raw 0..127 or -8192..8191 MIDI input value to target parameter value.
    pub fn map_value(&self, raw: f32, raw_min: f32, raw_max: f32) -> f32 {
        let norm = ((raw - raw_min) / (raw_max - raw_min)).clamp(0.0, 1.0);
        self.min_val + norm * (self.max_val - self.min_val)
    }
}

/// Log entry for MIDI event monitoring.
#[derive(Debug, Clone, PartialEq)]
pub struct MidiLogEntry {
    pub timestamp_ms: u64,
    pub channel: u8,
    pub event_type: String,
    pub data1: u8,
    pub data2: u8,
}

/// MIDI event monitor ring buffer log.
#[derive(Debug, Clone, Default)]
pub struct MidiMonitorLog {
    pub entries: VecDeque<MidiLogEntry>,
    pub max_entries: usize,
}

impl MidiMonitorLog {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            max_entries,
        }
    }

    pub fn log_event(&mut self, timestamp_ms: u64, channel: u8, event_type: impl Into<String>, data1: u8, data2: u8) {
        if self.max_entries > 0 && self.entries.len() >= self.max_entries {
            self.entries.pop_front();
        }
        self.entries.push_back(MidiLogEntry {
            timestamp_ms,
            channel,
            event_type: event_type.into(),
            data1,
            data2,
        });
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

/// Generate all note off MIDI messages for panic button across all 16 channels.
pub fn generate_panic_all_note_off() -> Vec<(u8, u8, u8)> {
    let mut msgs = Vec::new();
    for ch in 0..16 {
        // CC 123 (All Notes Off) and CC 121 (Reset All Controllers)
        msgs.push((0xB0 | ch, 121, 0));
        msgs.push((0xB0 | ch, 123, 0));
        // Individual Note Offs for active notes
        for note in 0..128 {
            msgs.push((0x80 | ch, note, 0));
        }
    }
    msgs
}

/// Filter and transpose MIDI event according to track parameters.
/// Returns None if channel does not match channel_filter.
pub fn filter_and_transpose_midi_note(
    ch: u8,
    note: u8,
    channel_filter: Option<u8>,
    transpose_offset: i8,
) -> Option<u8> {
    if let Some(req_ch) = channel_filter {
        if req_ch != 0 && req_ch != ch {
            return None;
        }
    }
    let transposed = (note as i16 + transpose_offset as i16).clamp(0, 127) as u8;
    Some(transposed)
}

/// Map QWERTY keyboard key to MIDI note pitch (base_octave 0-8, default 4).
pub fn qwerty_key_to_midi_note(key: &str, base_octave: u8) -> Option<u8> {
    let root_midi = (base_octave as i16 + 1) * 12;
    let offset: i16 = match key.to_uppercase().as_str() {
        "Z" => 0,  // C
        "S" => 1,  // C#
        "X" => 2,  // D
        "D" => 3,  // D#
        "C" => 4,  // E
        "V" => 5,  // F
        "G" => 6,  // F#
        "B" => 7,  // G
        "H" => 8,  // G#
        "N" => 9,  // A
        "J" => 10, // A#
        "M" => 11, // B
        "Q" => 12, // C+1
        "2" => 13, // C#+1
        "W" => 14, // D+1
        "3" => 15, // D#+1
        "E" => 16, // E+1
        "R" => 17, // F+1
        "5" => 18, // F#+1
        "T" => 19, // G+1
        "6" => 20, // G#+1
        "Y" => 21, // A+1
        "7" => 22, // A#+1
        "U" => 23, // B+1
        "I" => 24, // C+2
        _ => return None,
    };
    let note = (root_midi + offset).clamp(0, 127);
    Some(note as u8)
}
