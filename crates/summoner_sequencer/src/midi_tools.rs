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
    pub fn new(
        channel: u8,
        mapping_type: MidiMappingType,
        target_param_id: impl Into<String>,
        min_val: f32,
        max_val: f32,
    ) -> Self {
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

/// Soft takeover / pickup state tracking to prevent sudden parameter jumps on hardware faders (Task B3).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SoftTakeoverState {
    pub is_picked_up: bool,
    pub last_hardware_val: Option<f32>,
}

impl SoftTakeoverState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset pickup state (e.g. when switching presets or tracks).
    pub fn reset(&mut self) {
        self.is_picked_up = false;
        self.last_hardware_val = None;
    }

    /// Update with incoming hardware value.
    ///
    /// If not yet picked up, checks if hardware value is within threshold or crossed over
    /// `current_project_val`. Once picked up, updates pass through freely.
    pub fn update(
        &mut self,
        incoming_val: f32,
        current_project_val: f32,
        threshold: f32,
    ) -> Option<f32> {
        if self.is_picked_up {
            self.last_hardware_val = Some(incoming_val);
            return Some(incoming_val);
        }

        let is_close = (incoming_val - current_project_val).abs() <= threshold;
        let crossed_over = if let Some(last) = self.last_hardware_val {
            (last < current_project_val && incoming_val >= current_project_val)
                || (last > current_project_val && incoming_val <= current_project_val)
        } else {
            false
        };

        self.last_hardware_val = Some(incoming_val);

        if is_close || crossed_over {
            self.is_picked_up = true;
            Some(incoming_val)
        } else {
            None
        }
    }
}

/// Decode relative 2's complement encoder values (1..64 = +1..+64, 127..65 = -1..-63).
pub fn decode_relative_twos_complement(raw: u8) -> i8 {
    if raw <= 64 {
        raw as i8
    } else {
        (raw as i16 - 128) as i8
    }
}

/// Decode relative signed bit encoder values (bit 6 = sign: 0x40 | delta).
pub fn decode_relative_signed_bit(raw: u8) -> i8 {
    let sign = (raw & 0x40) != 0;
    let magnitude = (raw & 0x3F) as i8;
    if sign {
        -magnitude
    } else {
        magnitude
    }
}

/// Decode relative binary offset encoder values (64 = zero, >64 = positive, <64 = negative).
pub fn decode_relative_bin_offset(raw: u8) -> i8 {
    raw as i8 - 64
}

/// Interactive MIDI Learn session manager (Task B3).
#[derive(Debug, Clone, Default)]
pub struct MidiLearnSession {
    pub is_learning: bool,
    pub target_param_id: Option<String>,
    pub target_track_id: Option<u64>,
}

impl MidiLearnSession {
    pub fn new() -> Self {
        Self::default()
    }

    /// Begin listening for incoming MIDI messages to bind to a parameter.
    pub fn start_learning(&mut self, param_id: impl Into<String>, track_id: Option<u64>) {
        self.is_learning = true;
        self.target_param_id = Some(param_id.into());
        self.target_track_id = track_id;
    }

    /// Cancel active learn session.
    pub fn cancel(&mut self) {
        self.is_learning = false;
        self.target_param_id = None;
        self.target_track_id = None;
    }

    /// Process incoming MIDI CC event. If in learn mode, produces a new `MidiMappingConfig` and ends learn mode.
    pub fn process_cc_event(
        &mut self,
        channel: u8,
        cc: u8,
        device_name: Option<String>,
    ) -> Option<summoner_project::schema::MidiMappingConfig> {
        if !self.is_learning {
            return None;
        }

        let param_id = self.target_param_id.take()?;
        let track_id = self.target_track_id;
        self.is_learning = false;

        Some(summoner_project::schema::MidiMappingConfig {
            channel,
            cc,
            param_id,
            track_id,
            mode: summoner_project::schema::ControllerMappingMode::Absolute,
            min_val: 0.0,
            max_val: 1.0,
            soft_takeover: true,
            device_name,
        })
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

    pub fn log_event(
        &mut self,
        timestamp_ms: u64,
        channel: u8,
        event_type: impl Into<String>,
        data1: u8,
        data2: u8,
    ) {
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

/// Handle Input Echo toggle (Step 646).
/// Returns true if incoming MIDI should echo through instrument output.
pub fn should_echo_midi_input(echo_enabled: bool, is_instrument_selected: bool) -> bool {
    echo_enabled && is_instrument_selected
}

/// Arpeggiator direction modes (Step 647).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArpDirection {
    Up,
    Down,
    UpDown,
    Random,
    AsPlayed,
}

/// Arpeggiator configuration and pattern generator (Steps 647, 648, 649).
#[derive(Debug, Clone)]
pub struct Arpeggiator {
    pub direction: ArpDirection,
    pub octave_range: u8, // 1..=4
    pub gate_length: f32, // e.g. 0.8 = 80% step duration
    pub latch_enabled: bool,
    pub step_index: usize,
    pub latched_notes: Vec<u8>,
}

impl Default for Arpeggiator {
    fn default() -> Self {
        Self {
            direction: ArpDirection::Up,
            octave_range: 1,
            gate_length: 0.8,
            latch_enabled: false,
            step_index: 0,
            latched_notes: Vec::new(),
        }
    }
}

impl Arpeggiator {
    pub fn new(
        direction: ArpDirection,
        octave_range: u8,
        gate_length: f32,
        latch_enabled: bool,
    ) -> Self {
        Self {
            direction,
            octave_range: octave_range.clamp(1, 4),
            gate_length: gate_length.clamp(0.05, 2.0),
            latch_enabled,
            step_index: 0,
            latched_notes: Vec::new(),
        }
    }

    /// Generate the full expanded sequence of MIDI notes across octaves based on direction.
    pub fn generate_expanded_sequence(&self, base_notes: &[u8]) -> Vec<u8> {
        let active_base = if self.latch_enabled && base_notes.is_empty() {
            &self.latched_notes[..]
        } else {
            base_notes
        };

        if active_base.is_empty() {
            return Vec::new();
        }

        let mut expanded = Vec::new();

        match self.direction {
            ArpDirection::AsPlayed => {
                for oct in 0..self.octave_range {
                    for &n in active_base {
                        let shifted = (n as u16 + oct as u16 * 12).min(127) as u8;
                        expanded.push(shifted);
                    }
                }
            }
            ArpDirection::Up => {
                let mut sorted = active_base.to_vec();
                sorted.sort_unstable();
                for oct in 0..self.octave_range {
                    for &n in &sorted {
                        let shifted = (n as u16 + oct as u16 * 12).min(127) as u8;
                        expanded.push(shifted);
                    }
                }
            }
            ArpDirection::Down => {
                let mut sorted = active_base.to_vec();
                sorted.sort_unstable();
                sorted.reverse();
                for oct in (0..self.octave_range).rev() {
                    for &n in &sorted {
                        let shifted = (n as u16 + oct as u16 * 12).min(127) as u8;
                        expanded.push(shifted);
                    }
                }
            }
            ArpDirection::UpDown => {
                let mut sorted = active_base.to_vec();
                sorted.sort_unstable();
                let mut up_seq = Vec::new();
                for oct in 0..self.octave_range {
                    for &n in &sorted {
                        let shifted = (n as u16 + oct as u16 * 12).min(127) as u8;
                        up_seq.push(shifted);
                    }
                }
                expanded.extend_from_slice(&up_seq);
                if up_seq.len() > 2 {
                    let mut down_seq = up_seq[1..up_seq.len() - 1].to_vec();
                    down_seq.reverse();
                    expanded.extend(down_seq);
                }
            }
            ArpDirection::Random => {
                let mut sorted = active_base.to_vec();
                sorted.sort_unstable();
                for oct in 0..self.octave_range {
                    for &n in &sorted {
                        let shifted = (n as u16 + oct as u16 * 12).min(127) as u8;
                        expanded.push(shifted);
                    }
                }
                // Pseudo-random deterministic permutation based on note values
                expanded.sort_by_key(|&n| (n as u32 * 2654435761) % 1000);
            }
        }
        expanded
    }

    /// Step the arpeggiator to retrieve next note and active gate duration (in beats/fraction).
    pub fn next_step(&mut self, base_notes: &[u8]) -> Option<(u8, f32)> {
        if !base_notes.is_empty() && self.latch_enabled {
            self.latched_notes = base_notes.to_vec();
        }

        let sequence = self.generate_expanded_sequence(base_notes);
        if sequence.is_empty() {
            return None;
        }

        let idx = self.step_index % sequence.len();
        let note = sequence[idx];
        self.step_index += 1;
        Some((note, self.gate_length))
    }
}

/// Strum direction option (Step 650).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrumDirection {
    LowToHigh,
    HighToLow,
}

/// Strummer tool: spreads chord notes over configurable millisecond delay (Step 650).
#[derive(Debug, Clone)]
pub struct Strummer {
    pub strum_time_ms: f32, // total time spread across chord notes
    pub direction: StrumDirection,
}

impl Default for Strummer {
    fn default() -> Self {
        Self {
            strum_time_ms: 30.0,
            direction: StrumDirection::LowToHigh,
        }
    }
}

impl Strummer {
    pub fn new(strum_time_ms: f32, direction: StrumDirection) -> Self {
        Self {
            strum_time_ms: strum_time_ms.max(0.0),
            direction,
        }
    }

    /// Takes a list of chord notes and returns pairs of (note, delay_ms).
    pub fn strum(&self, chord_notes: &[u8]) -> Vec<(u8, f32)> {
        if chord_notes.is_empty() {
            return Vec::new();
        }
        let mut sorted = chord_notes.to_vec();
        sorted.sort_unstable();
        if self.direction == StrumDirection::HighToLow {
            sorted.reverse();
        }

        let step_delay = if sorted.len() > 1 {
            self.strum_time_ms / (sorted.len() - 1) as f32
        } else {
            0.0
        };

        sorted
            .into_iter()
            .enumerate()
            .map(|(idx, note)| (note, idx as f32 * step_delay))
            .collect()
    }
}

/// Chord Memory manager: save up to 8 chords, trigger by slot index (0..7) or MIDI note 1..8 (Step 651).
#[derive(Debug, Clone, Default)]
pub struct ChordMemory {
    pub slots: [Vec<u8>; 8],
}

impl ChordMemory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn save_chord(&mut self, slot: usize, notes: Vec<u8>) -> bool {
        if slot < 8 {
            self.slots[slot] = notes;
            true
        } else {
            false
        }
    }

    pub fn trigger(&self, slot: usize) -> Option<&[u8]> {
        if slot < 8 && !self.slots[slot].is_empty() {
            Some(&self.slots[slot])
        } else {
            None
        }
    }

    /// Trigger stored chord by MIDI note index 1..=8 (or MIDI pitch 36..=43 / C2..G2).
    pub fn trigger_by_note(&self, note: u8) -> Option<&[u8]> {
        let slot = match note {
            1..=8 => (note - 1) as usize,
            36..=43 => (note - 36) as usize,
            _ => return None,
        };
        self.trigger(slot)
    }
}

/// Keyboard Split router: low range plays one instrument, high range plays another (Step 652).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyboardSplit {
    pub split_key: u8,
    pub low_track_id: u64,
    pub high_track_id: u64,
}

impl KeyboardSplit {
    pub fn new(split_key: u8, low_track_id: u64, high_track_id: u64) -> Self {
        Self {
            split_key,
            low_track_id,
            high_track_id,
        }
    }

    pub fn route(&self, note: u8) -> u64 {
        if note < self.split_key {
            self.low_track_id
        } else {
            self.high_track_id
        }
    }
}

/// Keyboard Layering router: multiple instruments play simultaneously (Step 653).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyboardLayering {
    pub target_track_ids: Vec<u64>,
}

impl KeyboardLayering {
    pub fn new(target_track_ids: Vec<u64>) -> Self {
        Self { target_track_ids }
    }

    pub fn route(&self) -> &[u64] {
        &self.target_track_ids
    }
}

/// Calculate frequency ratio multiplier for +/-50 cents fine tuning (Step 654).
pub fn cents_to_freq_ratio(cents: f32) -> f32 {
    2.0f32.powf(cents.clamp(-50.0, 50.0) / 1200.0)
}

/// Calculate tuned frequency (Hz) for a MIDI note with master tune offset (-100..+100 cents)
/// and fine tune offset (-50..+50 cents) (Step 655).
pub fn midi_note_to_hz_tuned(note: u8, master_cents: f32, fine_cents: f32) -> f32 {
    let total_cents = master_cents.clamp(-100.0, 100.0) + fine_cents.clamp(-50.0, 50.0);
    440.0 * 2.0f32.powf(((note as f32) - 69.0 + total_cents / 100.0) / 12.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soft_takeover_pickup_logic() {
        let mut st = SoftTakeoverState::new();
        let target_project_val = 0.75;
        let threshold = 0.05;

        // Fader starting far away at 0.1: should not update (prevent jumps)
        assert_eq!(st.update(0.1, target_project_val, threshold), None);
        assert!(!st.is_picked_up);

        // Fader moving to 0.4: still too far
        assert_eq!(st.update(0.4, target_project_val, threshold), None);
        assert!(!st.is_picked_up);

        // Fader reaches 0.73 (within 0.05 threshold of 0.75): should pick up!
        assert_eq!(st.update(0.73, target_project_val, threshold), Some(0.73));
        assert!(st.is_picked_up);

        // Once picked up, arbitrary movements pass through
        assert_eq!(st.update(0.85, target_project_val, threshold), Some(0.85));
        assert_eq!(st.update(0.20, target_project_val, threshold), Some(0.20));
    }

    #[test]
    fn test_relative_encoder_decoding() {
        // Two's complement
        assert_eq!(decode_relative_twos_complement(1), 1);
        assert_eq!(decode_relative_twos_complement(5), 5);
        assert_eq!(decode_relative_twos_complement(127), -1);
        assert_eq!(decode_relative_twos_complement(123), -5);

        // Signed bit
        assert_eq!(decode_relative_signed_bit(3), 3);
        assert_eq!(decode_relative_signed_bit(0x40 | 3), -3);

        // Binary offset
        assert_eq!(decode_relative_bin_offset(64), 0);
        assert_eq!(decode_relative_bin_offset(67), 3);
        assert_eq!(decode_relative_bin_offset(60), -4);
    }

    #[test]
    fn test_midi_learn_session() {
        let mut session = MidiLearnSession::new();
        assert!(!session.is_learning);

        // Event before learning starts is ignored
        assert!(session.process_cc_event(1, 7, None).is_none());

        // Start learning parameter
        session.start_learning("track_1_filter_cutoff", Some(1));
        assert!(session.is_learning);

        // Move hardware knob CC 14 on Channel 2
        let mapping = session.process_cc_event(2, 14, Some("APC40 MkII".to_string()));
        assert!(mapping.is_some());
        let map = mapping.unwrap();
        assert_eq!(map.channel, 2);
        assert_eq!(map.cc, 14);
        assert_eq!(map.param_id, "track_1_filter_cutoff");
        assert_eq!(map.track_id, Some(1));
        assert_eq!(map.device_name.as_deref(), Some("APC40 MkII"));
        assert!(map.soft_takeover);

        // Session ended
        assert!(!session.is_learning);
    }
}
