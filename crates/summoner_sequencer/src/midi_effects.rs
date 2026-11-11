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

//! Insertable MIDI Effects: Arpeggiator, Chord, Scale, Note-Length, and Random.
//!
//! Provides sample-accurate and beat-synchronized MIDI event transformation chains.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
use summoner_core::midi::MidiEvent;

/// Real-time execution context for MIDI effect processing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MidiEffectContext {
    pub sample_rate: u32,
    pub bpm: f64,
    pub block_frames: u32,
    pub current_beat: f64,
}

impl MidiEffectContext {
    pub fn new(sample_rate: u32, bpm: f64, block_frames: u32, current_beat: f64) -> Self {
        Self {
            sample_rate,
            bpm,
            block_frames,
            current_beat,
        }
    }
}

/// Common trait implemented by all insertable MIDI effect devices.
pub trait MidiEffect: Send + Sync {
    /// Return the display name of this MIDI effect.
    fn name(&self) -> &str;

    /// Process a slice of input MIDI events and return transformed output events.
    fn process(&mut self, events: &[MidiEvent], ctx: &MidiEffectContext) -> Vec<MidiEvent>;

    /// Reset internal state (flush held notes, reset arp step, clear timers).
    fn reset(&mut self);
}

// =========================================================================
// 1. Arpeggiator Device
// =========================================================================

/// Arpeggiator direction patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ArpPattern {
    #[default]
    Up,
    Down,
    UpDown,
    DownUp,
    Random,
    AsPlayed,
    Chord,
}

/// Arpeggiator musical step division rates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ArpRate {
    Whole,
    Half,
    Quarter,
    #[default]
    Eighth,
    Sixteenth,
    ThirtySecond,
}

impl ArpRate {
    /// Returns the step duration in musical beats (quarter note = 1.0).
    pub fn to_beats(&self) -> f64 {
        match self {
            ArpRate::Whole => 4.0,
            ArpRate::Half => 2.0,
            ArpRate::Quarter => 1.0,
            ArpRate::Eighth => 0.5,
            ArpRate::Sixteenth => 0.25,
            ArpRate::ThirtySecond => 0.125,
        }
    }
}

/// Insertable Arpeggiator MIDI Device.
#[derive(Debug, Clone)]
pub struct ArpeggiatorDevice {
    pub pattern: ArpPattern,
    pub rate: ArpRate,
    pub octaves: u8,      // 1 to 4
    pub gate_length: f64, // 0.1 to 2.0 (fraction of step)
    pub enabled: bool,
    held_notes: Vec<(u8, u8)>, // (note, velocity) in arrival order
    sorted_notes: BTreeSet<u8>,
    current_step: usize,
    last_beat: f64,
    active_arp_note: Option<(u8, u8)>, // (channel, note) currently sounding
}

impl ArpeggiatorDevice {
    pub fn new() -> Self {
        Self {
            pattern: ArpPattern::Up,
            rate: ArpRate::Sixteenth,
            octaves: 1,
            gate_length: 0.8,
            enabled: true,
            held_notes: Vec::new(),
            sorted_notes: BTreeSet::new(),
            current_step: 0,
            last_beat: 0.0,
            active_arp_note: None,
        }
    }

    pub fn with_pattern(mut self, pattern: ArpPattern) -> Self {
        self.pattern = pattern;
        self
    }

    pub fn with_rate(mut self, rate: ArpRate) -> Self {
        self.rate = rate;
        self
    }

    pub fn with_octaves(mut self, octaves: u8) -> Self {
        self.octaves = octaves.clamp(1, 4);
        self
    }

    /// Build notes list across configured octaves based on pattern.
    fn build_pattern_notes(&self) -> Vec<u8> {
        let base_notes: Vec<u8> = match self.pattern {
            ArpPattern::AsPlayed => self.held_notes.iter().map(|(n, _)| *n).collect(),
            _ => self.sorted_notes.iter().copied().collect(),
        };

        if base_notes.is_empty() {
            return Vec::new();
        }

        let mut all_notes = Vec::new();
        for oct in 0..self.octaves {
            for &n in &base_notes {
                let shifted = n.saturating_add(oct * 12).min(127);
                all_notes.push(shifted);
            }
        }

        match self.pattern {
            ArpPattern::Up | ArpPattern::AsPlayed => all_notes,
            ArpPattern::Down => {
                all_notes.reverse();
                all_notes
            }
            ArpPattern::UpDown => {
                let mut rev = all_notes.clone();
                rev.reverse();
                if rev.len() > 2 {
                    rev.remove(0);
                    rev.pop();
                }
                all_notes.extend(rev);
                all_notes
            }
            ArpPattern::DownUp => {
                all_notes.reverse();
                let mut up = all_notes.clone();
                up.reverse();
                if up.len() > 2 {
                    up.remove(0);
                    up.pop();
                }
                all_notes.extend(up);
                all_notes
            }
            ArpPattern::Random | ArpPattern::Chord => all_notes,
        }
    }
}

impl Default for ArpeggiatorDevice {
    fn default() -> Self {
        Self::new()
    }
}

impl MidiEffect for ArpeggiatorDevice {
    fn name(&self) -> &str {
        "Arpeggiator"
    }

    fn reset(&mut self) {
        self.held_notes.clear();
        self.sorted_notes.clear();
        self.current_step = 0;
        self.active_arp_note = None;
    }

    fn process(&mut self, events: &[MidiEvent], ctx: &MidiEffectContext) -> Vec<MidiEvent> {
        if !self.enabled {
            return events.to_vec();
        }

        let mut output = Vec::new();

        // 1. Ingest incoming MIDI notes
        for &evt in events {
            match evt {
                MidiEvent::NoteOn(_ch, note, vel) => {
                    if vel > 0 {
                        self.held_notes.retain(|(n, _)| *n != note);
                        self.held_notes.push((note, vel));
                        self.sorted_notes.insert(note);
                    } else {
                        self.held_notes.retain(|(n, _)| *n != note);
                        self.sorted_notes.remove(&note);
                    }
                }
                MidiEvent::NoteOff(_ch, note, _vel) => {
                    self.held_notes.retain(|(n, _)| *n != note);
                    self.sorted_notes.remove(&note);
                }
                other => output.push(other), // Pass through CCs, pitch bends, etc.
            }
        }

        // If no keys held, cut off any currently active arp note
        if self.held_notes.is_empty() {
            if let Some((ch, note)) = self.active_arp_note.take() {
                output.push(MidiEvent::NoteOff(ch, note, 0));
            }
            self.last_beat = ctx.current_beat;
            return output;
        }

        let step_beats = self.rate.to_beats();
        let prev_step = (self.last_beat / step_beats).floor() as u64;
        let curr_step = (ctx.current_beat / step_beats).floor() as u64;

        // Check if we advanced to a new step
        if curr_step > prev_step || self.active_arp_note.is_none() {
            // Cut previous note
            if let Some((ch, note)) = self.active_arp_note.take() {
                output.push(MidiEvent::NoteOff(ch, note, 0));
            }

            let pattern_notes = self.build_pattern_notes();
            if !pattern_notes.is_empty() {
                let note = match self.pattern {
                    ArpPattern::Random => {
                        let idx = (curr_step as usize * 31 + 7) % pattern_notes.len();
                        pattern_notes[idx]
                    }
                    _ => {
                        let idx = self.current_step % pattern_notes.len();
                        self.current_step += 1;
                        pattern_notes[idx]
                    }
                };

                let ch = 0;
                let vel = self.held_notes.last().map(|(_, v)| *v).unwrap_or(100);
                output.push(MidiEvent::NoteOn(ch, note, vel));
                self.active_arp_note = Some((ch, note));
            }
        }

        self.last_beat = ctx.current_beat;
        output
    }
}

// =========================================================================
// 2. Chord Device
// =========================================================================

/// Preset chord types for the Chord Device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ChordType {
    #[default]
    MajorTriad, // +4, +7
    MinorTriad,  // +3, +7
    Octave,      // +12
    Fifth,       // +7
    Dominant7th, // +4, +7, +10
    Major7th,    // +4, +7, +11
    Minor7th,    // +3, +7, +10
    Sus4,        // +5, +7
    Add9,        // +4, +7, +14
}

impl ChordType {
    pub fn intervals(&self) -> &'static [i8] {
        match self {
            ChordType::MajorTriad => &[4, 7],
            ChordType::MinorTriad => &[3, 7],
            ChordType::Octave => &[12],
            ChordType::Fifth => &[7],
            ChordType::Dominant7th => &[4, 7, 10],
            ChordType::Major7th => &[4, 7, 11],
            ChordType::Minor7th => &[3, 7, 10],
            ChordType::Sus4 => &[5, 7],
            ChordType::Add9 => &[4, 7, 14],
        }
    }
}

/// Insertable Chord MIDI Device: adds parallel harmony intervals to single notes.
#[derive(Debug, Clone)]
pub struct ChordDevice {
    pub chord_type: ChordType,
    pub custom_intervals: Vec<i8>,
    pub enabled: bool,
    /// Maps incoming trigger note to list of generated chord notes for NoteOff tracking
    active_chords: HashMap<u8, Vec<u8>>,
}

impl ChordDevice {
    pub fn new(chord_type: ChordType) -> Self {
        Self {
            chord_type,
            custom_intervals: Vec::new(),
            enabled: true,
            active_chords: HashMap::new(),
        }
    }

    pub fn with_custom_intervals(mut self, intervals: Vec<i8>) -> Self {
        self.custom_intervals = intervals;
        self
    }
}

impl Default for ChordDevice {
    fn default() -> Self {
        Self::new(ChordType::MajorTriad)
    }
}

impl MidiEffect for ChordDevice {
    fn name(&self) -> &str {
        "Chord"
    }

    fn reset(&mut self) {
        self.active_chords.clear();
    }

    fn process(&mut self, events: &[MidiEvent], _ctx: &MidiEffectContext) -> Vec<MidiEvent> {
        if !self.enabled {
            return events.to_vec();
        }

        let intervals = if !self.custom_intervals.is_empty() {
            &self.custom_intervals[..]
        } else {
            self.chord_type.intervals()
        };

        let mut output = Vec::with_capacity(events.len() * (intervals.len() + 1));

        for &evt in events {
            match evt {
                MidiEvent::NoteOn(ch, note, vel) => {
                    output.push(evt);
                    let mut generated = Vec::with_capacity(intervals.len());

                    for &interval in intervals {
                        let chord_note = (note as i16 + interval as i16).clamp(0, 127) as u8;
                        output.push(MidiEvent::NoteOn(ch, chord_note, vel));
                        generated.push(chord_note);
                    }
                    self.active_chords.insert(note, generated);
                }
                MidiEvent::NoteOff(ch, note, vel) => {
                    output.push(evt);
                    if let Some(generated) = self.active_chords.remove(&note) {
                        for chord_note in generated {
                            output.push(MidiEvent::NoteOff(ch, chord_note, vel));
                        }
                    }
                }
                other => output.push(other),
            }
        }

        output
    }
}

// =========================================================================
// 3. Scale Device
// =========================================================================

/// Musical scale mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ScaleMode {
    #[default]
    Major,
    Minor,
    HarmonicMinor,
    Dorian,
    Phrygian,
    Mixolydian,
    PentatonicMajor,
    PentatonicMinor,
    Blues,
}

impl ScaleMode {
    /// Bitmask of 12 semitones present in the scale relative to root (bit 0 = root).
    pub fn semitone_mask(&self) -> u16 {
        match self {
            ScaleMode::Major => 0b101010110101, // 0, 2, 4, 5, 7, 9, 11
            ScaleMode::Minor => 0b010110101101, // 0, 2, 3, 5, 7, 8, 10
            ScaleMode::HarmonicMinor => 0b100110101101, // 0, 2, 3, 5, 7, 8, 11
            ScaleMode::Dorian => 0b010101101101, // 0, 2, 3, 5, 7, 9, 10
            ScaleMode::Phrygian => 0b010110101011, // 0, 1, 3, 5, 7, 8, 10
            ScaleMode::Mixolydian => 0b011010110101, // 0, 2, 4, 5, 7, 9, 10
            ScaleMode::PentatonicMajor => 0b001001010101, // 0, 2, 4, 7, 9
            ScaleMode::PentatonicMinor => 0b010010100101, // 0, 3, 5, 7, 10
            ScaleMode::Blues => 0b010010110101, // 0, 3, 5, 6, 7, 10
        }
    }
}

/// Out-of-scale quantization policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ScaleQuantizePolicy {
    #[default]
    SnapNearest,
    SnapDown,
    SnapUp,
    Mute,
}

/// Insertable Scale MIDI Device: quantizes incoming notes to a chosen musical scale.
#[derive(Debug, Clone)]
pub struct ScaleDevice {
    pub root_note: u8, // 0 = C, 1 = C#, ... 11 = B
    pub mode: ScaleMode,
    pub policy: ScaleQuantizePolicy,
    pub enabled: bool,
    note_mapping: HashMap<u8, Option<u8>>,
}

impl ScaleDevice {
    pub fn new(root_note: u8, mode: ScaleMode) -> Self {
        Self {
            root_note: root_note % 12,
            mode,
            policy: ScaleQuantizePolicy::SnapNearest,
            enabled: true,
            note_mapping: HashMap::new(),
        }
    }

    pub fn with_policy(mut self, policy: ScaleQuantizePolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Quantize a single MIDI note number according to scale and policy.
    pub fn quantize_note(&self, note: u8) -> Option<u8> {
        let mask = self.mode.semitone_mask();
        let rel_pitch = ((note as i16 - self.root_note as i16).rem_euclid(12)) as usize;

        if (mask & (1 << rel_pitch)) != 0 {
            return Some(note); // Already in scale!
        }

        match self.policy {
            ScaleQuantizePolicy::Mute => None,
            ScaleQuantizePolicy::SnapDown => {
                for delta in 1..=12 {
                    let candidate = note.saturating_sub(delta);
                    let p = ((candidate as i16 - self.root_note as i16).rem_euclid(12)) as usize;
                    if (mask & (1 << p)) != 0 {
                        return Some(candidate);
                    }
                }
                Some(note)
            }
            ScaleQuantizePolicy::SnapUp => {
                for delta in 1..=12 {
                    let candidate = (note + delta).min(127);
                    let p = ((candidate as i16 - self.root_note as i16).rem_euclid(12)) as usize;
                    if (mask & (1 << p)) != 0 {
                        return Some(candidate);
                    }
                }
                Some(note)
            }
            ScaleQuantizePolicy::SnapNearest => {
                let down = (1..=6).find(|&d| {
                    let c = note.saturating_sub(d);
                    let p = ((c as i16 - self.root_note as i16).rem_euclid(12)) as usize;
                    (mask & (1 << p)) != 0
                });
                let up = (1..=6).find(|&d| {
                    let c = (note + d).min(127);
                    let p = ((c as i16 - self.root_note as i16).rem_euclid(12)) as usize;
                    (mask & (1 << p)) != 0
                });

                match (down, up) {
                    (Some(d), Some(u)) => {
                        if d <= u {
                            Some(note.saturating_sub(d))
                        } else {
                            Some((note + u).min(127))
                        }
                    }
                    (Some(d), None) => Some(note.saturating_sub(d)),
                    (None, Some(u)) => Some((note + u).min(127)),
                    (None, None) => Some(note),
                }
            }
        }
    }
}

impl Default for ScaleDevice {
    fn default() -> Self {
        Self::new(0, ScaleMode::Major)
    }
}

impl MidiEffect for ScaleDevice {
    fn name(&self) -> &str {
        "Scale"
    }

    fn reset(&mut self) {
        self.note_mapping.clear();
    }

    fn process(&mut self, events: &[MidiEvent], _ctx: &MidiEffectContext) -> Vec<MidiEvent> {
        if !self.enabled {
            return events.to_vec();
        }

        let mut output = Vec::with_capacity(events.len());

        for &evt in events {
            match evt {
                MidiEvent::NoteOn(ch, note, vel) => {
                    if let Some(quantized) = self.quantize_note(note) {
                        output.push(MidiEvent::NoteOn(ch, quantized, vel));
                        self.note_mapping.insert(note, Some(quantized));
                    } else {
                        self.note_mapping.insert(note, None);
                    }
                }
                MidiEvent::NoteOff(ch, note, vel) => {
                    if let Some(Some(quantized)) = self.note_mapping.remove(&note) {
                        output.push(MidiEvent::NoteOff(ch, quantized, vel));
                    }
                }
                other => output.push(other),
            }
        }

        output
    }
}

// =========================================================================
// 4. Note-Length Device
// =========================================================================

/// Mode for Note-Length processing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum NoteLengthMode {
    #[default]
    FixedBeats,
    GateScale,
    ReleaseTrigger,
}

/// Insertable Note-Length MIDI Device.
#[derive(Debug, Clone)]
pub struct NoteLengthDevice {
    pub mode: NoteLengthMode,
    pub length_beats: f64, // e.g. 0.25 = sixteenth note
    pub gate_scale: f64,   // e.g. 0.5 = 50% staccato
    pub enabled: bool,
    pending_note_offs: Vec<(u8, u8, f64)>, // (channel, note, scheduled_off_beat)
}

impl NoteLengthDevice {
    pub fn new() -> Self {
        Self {
            mode: NoteLengthMode::FixedBeats,
            length_beats: 0.25,
            gate_scale: 1.0,
            enabled: true,
            pending_note_offs: Vec::new(),
        }
    }

    pub fn with_fixed_beats(mut self, beats: f64) -> Self {
        self.mode = NoteLengthMode::FixedBeats;
        self.length_beats = beats.max(0.01);
        self
    }

    pub fn with_gate_scale(mut self, scale: f64) -> Self {
        self.mode = NoteLengthMode::GateScale;
        self.gate_scale = scale.clamp(0.05, 5.0);
        self
    }
}

impl Default for NoteLengthDevice {
    fn default() -> Self {
        Self::new()
    }
}

impl MidiEffect for NoteLengthDevice {
    fn name(&self) -> &str {
        "NoteLength"
    }

    fn reset(&mut self) {
        self.pending_note_offs.clear();
    }

    fn process(&mut self, events: &[MidiEvent], ctx: &MidiEffectContext) -> Vec<MidiEvent> {
        if !self.enabled {
            return events.to_vec();
        }

        let mut output = Vec::new();

        // 1. Drain any scheduled note offs whose beat has arrived
        self.pending_note_offs.retain(|&(ch, note, off_beat)| {
            if ctx.current_beat >= off_beat {
                output.push(MidiEvent::NoteOff(ch, note, 0));
                false
            } else {
                true
            }
        });

        // 2. Process incoming events
        for &evt in events {
            match evt {
                MidiEvent::NoteOn(ch, note, _vel) => match self.mode {
                    NoteLengthMode::FixedBeats => {
                        output.push(evt);
                        let off_beat = ctx.current_beat + self.length_beats;
                        self.pending_note_offs.push((ch, note, off_beat));
                    }
                    NoteLengthMode::GateScale => {
                        output.push(evt);
                    }
                    NoteLengthMode::ReleaseTrigger => {
                        // Suppress NoteOn until release!
                    }
                },
                MidiEvent::NoteOff(ch, note, _vel) => match self.mode {
                    NoteLengthMode::FixedBeats => {
                        // Suppress physical NoteOff since fixed length manages it
                    }
                    NoteLengthMode::GateScale => {
                        output.push(evt);
                    }
                    NoteLengthMode::ReleaseTrigger => {
                        output.push(MidiEvent::NoteOn(ch, note, 100));
                        let off_beat = ctx.current_beat + 0.1;
                        self.pending_note_offs.push((ch, note, off_beat));
                    }
                },
                other => output.push(other),
            }
        }

        output
    }
}

// =========================================================================
// 5. Random MIDI Device
// =========================================================================

/// Insertable Random MIDI Device: velocity jitter, pitch probability, and note drop.
#[derive(Debug, Clone)]
pub struct RandomMidiDevice {
    pub chance: f32,                // 0.0 to 1.0 (probability note passes)
    pub velocity_random_range: i8,  // +/- range added to velocity
    pub pitch_random_semitones: i8, // +/- random semitone range
    pub enabled: bool,
    seed: u32,
    pitch_shifts: HashMap<u8, u8>,
}

impl RandomMidiDevice {
    pub fn new() -> Self {
        Self {
            chance: 1.0,
            velocity_random_range: 0,
            pitch_random_semitones: 0,
            enabled: true,
            seed: 12345,
            pitch_shifts: HashMap::new(),
        }
    }

    pub fn with_chance(mut self, chance: f32) -> Self {
        self.chance = chance.clamp(0.0, 1.0);
        self
    }

    pub fn with_velocity_range(mut self, range: i8) -> Self {
        self.velocity_random_range = range.abs();
        self
    }

    pub fn with_pitch_range(mut self, semitones: i8) -> Self {
        self.pitch_random_semitones = semitones.abs();
        self
    }

    fn next_rand(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223);
        ((self.seed & 0x00FF_FFFF) as f32) / (16777216.0)
    }
}

impl Default for RandomMidiDevice {
    fn default() -> Self {
        Self::new()
    }
}

impl MidiEffect for RandomMidiDevice {
    fn name(&self) -> &str {
        "Random"
    }

    fn reset(&mut self) {
        self.pitch_shifts.clear();
    }

    fn process(&mut self, events: &[MidiEvent], _ctx: &MidiEffectContext) -> Vec<MidiEvent> {
        if !self.enabled {
            return events.to_vec();
        }

        let mut output = Vec::with_capacity(events.len());

        for &evt in events {
            match evt {
                MidiEvent::NoteOn(ch, note, vel) => {
                    // 1. Chance filter
                    if self.next_rand() > self.chance {
                        continue; // Dropped note
                    }

                    // 2. Pitch jitter
                    let pitch_offset = if self.pitch_random_semitones > 0 {
                        let span = (self.pitch_random_semitones * 2 + 1) as f32;
                        (self.next_rand() * span) as i8 - self.pitch_random_semitones
                    } else {
                        0
                    };
                    let shifted_note = (note as i16 + pitch_offset as i16).clamp(0, 127) as u8;
                    self.pitch_shifts.insert(note, shifted_note);

                    // 3. Velocity jitter
                    let vel_offset = if self.velocity_random_range > 0 {
                        let span = (self.velocity_random_range * 2 + 1) as f32;
                        (self.next_rand() * span) as i8 - self.velocity_random_range
                    } else {
                        0
                    };
                    let randomized_vel = (vel as i16 + vel_offset as i16).clamp(1, 127) as u8;

                    output.push(MidiEvent::NoteOn(ch, shifted_note, randomized_vel));
                }
                MidiEvent::NoteOff(ch, note, vel) => {
                    let shifted_note = self.pitch_shifts.remove(&note).unwrap_or(note);
                    output.push(MidiEvent::NoteOff(ch, shifted_note, vel));
                }
                other => output.push(other),
            }
        }

        output
    }
}

// =========================================================================
// 6. MidiEffectChain
// =========================================================================

/// Container managing a serial chain of insertable MIDI effects.
pub struct MidiEffectChain {
    pub devices: Vec<Box<dyn MidiEffect>>,
    pub bypassed: bool,
}

impl MidiEffectChain {
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
            bypassed: false,
        }
    }

    pub fn add_device<D: MidiEffect + 'static>(&mut self, device: D) {
        self.devices.push(Box::new(device));
    }

    pub fn process(
        &mut self,
        mut events: Vec<MidiEvent>,
        ctx: &MidiEffectContext,
    ) -> Vec<MidiEvent> {
        if self.bypassed {
            return events;
        }

        for dev in &mut self.devices {
            events = dev.process(&events, ctx);
        }

        events
    }

    pub fn reset(&mut self) {
        for dev in &mut self.devices {
            dev.reset();
        }
    }
}

impl Default for MidiEffectChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chord_device_generates_harmonies() {
        let mut chord = ChordDevice::new(ChordType::MajorTriad); // +4, +7
        let ctx = MidiEffectContext::new(44100, 120.0, 128, 0.0);

        // Input: NoteOn C4 (60)
        let input = vec![MidiEvent::NoteOn(0, 60, 100)];
        let output = chord.process(&input, &ctx);

        // Expected: C4 (60), E4 (64), G4 (67)
        assert_eq!(output.len(), 3);
        assert_eq!(output[0], MidiEvent::NoteOn(0, 60, 100));
        assert_eq!(output[1], MidiEvent::NoteOn(0, 64, 100));
        assert_eq!(output[2], MidiEvent::NoteOn(0, 67, 100));

        // NoteOff C4 should turn off all 3 notes
        let off_input = vec![MidiEvent::NoteOff(0, 60, 0)];
        let off_output = chord.process(&off_input, &ctx);
        assert_eq!(off_output.len(), 3);
        assert_eq!(off_output[0], MidiEvent::NoteOff(0, 60, 0));
        assert_eq!(off_output[1], MidiEvent::NoteOff(0, 64, 0));
        assert_eq!(off_output[2], MidiEvent::NoteOff(0, 67, 0));
    }

    #[test]
    fn test_scale_device_quantization() {
        // C Major scale: C D E F G A B
        let mut scale = ScaleDevice::new(0, ScaleMode::Major);
        let ctx = MidiEffectContext::new(44100, 120.0, 128, 0.0);

        // Input: C#4 (61) which is out of C major -> should quantize to D4 (62) or C4 (60)
        let input = vec![MidiEvent::NoteOn(0, 61, 90)];
        let output = scale.process(&input, &ctx);
        assert_eq!(output.len(), 1);
        if let MidiEvent::NoteOn(_, note, _) = output[0] {
            assert!(note == 60 || note == 62, "Must be quantized into C Major");
        } else {
            panic!("Expected NoteOn");
        }
    }

    #[test]
    fn test_random_device_chance_filter() {
        let mut rand_dev = RandomMidiDevice::new().with_chance(0.0); // 0% chance -> drops all notes
        let ctx = MidiEffectContext::new(44100, 120.0, 128, 0.0);

        let input = vec![MidiEvent::NoteOn(0, 60, 100)];
        let output = rand_dev.process(&input, &ctx);
        assert!(output.is_empty(), "0% chance must drop notes");
    }

    #[test]
    fn test_midi_effect_chain() {
        let mut chain = MidiEffectChain::new();
        // Scale device (C Major) -> Chord device (Major Triad)
        chain.add_device(ScaleDevice::new(0, ScaleMode::Major));
        chain.add_device(ChordDevice::new(ChordType::MajorTriad));

        let ctx = MidiEffectContext::new(44100, 120.0, 128, 0.0);
        let input = vec![MidiEvent::NoteOn(0, 60, 100)];
        let output = chain.process(input, &ctx);

        assert_eq!(output.len(), 3);
    }
}
