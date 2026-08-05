// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Real-time MIDI input filter and velocity curve mapping engine.

use crate::midi::MidiEvent;

/// Velocity response curves for real-time MIDI velocity mapping.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VelocityCurve {
    /// Linear identity mapping (out = in).
    Linear,
    /// Soft response (lower velocities scaled down, requires harder strikes for high velocity).
    Soft,
    /// Hard response (lower velocities boosted, easier to achieve high velocity).
    Hard,
    /// S-Curve response (compressed low/high dynamics, emphasized mid dynamics).
    SCurve,
    /// Exponential curve response with configurable power factor (> 0.0).
    Exponential(f32),
    /// Logarithmic curve response with configurable scale factor (> 0.0).
    Logarithmic(f32),
    /// Fixed velocity output value for all Note On events (1..=127).
    Fixed(u8),
    /// Range compressor: scales input velocity 1..=127 into range min..=max.
    Compress {
        /// Minimum output velocity bound (1..=127).
        min: u8,
        /// Maximum output velocity bound (1..=127).
        max: u8,
    },
    /// Custom lookup table mapping 128 input velocity values directly to output velocity values.
    CustomTable([u8; 128]),
}

impl VelocityCurve {
    /// Maps an input MIDI velocity byte (0..=127) to a curve-processed output velocity byte (0..=127).
    /// Velocity 0 (Note Off) is preserved as 0.
    pub fn map(&self, vel: u8) -> u8 {
        if vel == 0 {
            return 0;
        }

        let input_norm = (vel & 0x7F) as f32 / 127.0;
        let out_norm = match self {
            VelocityCurve::Linear => input_norm,
            VelocityCurve::Soft => input_norm.powf(1.5),
            VelocityCurve::Hard => input_norm.powf(0.667),
            VelocityCurve::SCurve => 3.0 * input_norm.powi(2) - 2.0 * input_norm.powi(3),
            VelocityCurve::Exponential(factor) => input_norm.powf((*factor).max(0.1)),
            VelocityCurve::Logarithmic(factor) => {
                let f = (*factor).max(0.01);
                (1.0 + f * input_norm).ln() / (1.0 + f).ln()
            }
            VelocityCurve::Fixed(fixed_val) => return (*fixed_val).clamp(1, 127),
            VelocityCurve::Compress { min, max } => {
                let low = (*min as f32).clamp(1.0, 127.0);
                let high = (*max as f32).clamp(low, 127.0);
                return (low + (high - low) * input_norm).round().clamp(1.0, 127.0) as u8;
            }
            VelocityCurve::CustomTable(table) => return table[(vel & 0x7F) as usize].clamp(0, 127),
        };

        (out_norm * 127.0).round().clamp(1.0, 127.0) as u8
    }

    /// Maps a normalized float velocity (0.0..=1.0) to a curve-processed float velocity (0.0..=1.0).
    pub fn map_f32(&self, vel: f32) -> f32 {
        if vel <= 0.0 {
            return 0.0;
        }
        let input_byte = (vel * 127.0).round().clamp(0.0, 127.0) as u8;
        self.map(input_byte) as f32 / 127.0
    }
}

/// Configuration settings for real-time MIDI filtering and transformation.
#[derive(Debug, Clone, PartialEq)]
pub struct MidiInputFilter {
    /// Bitmask of allowed MIDI channels (bit 0 = Ch 1, bit 15 = Ch 16). 0xFFFF = All Channels.
    pub channel_mask: u16,
    /// Minimum allowed MIDI note number (0..=127).
    pub min_note: u8,
    /// Maximum allowed MIDI note number (0..=127).
    pub max_note: u8,
    /// Note transpose offset in semitones (-128..=127).
    pub transpose: i8,
    /// Minimum velocity threshold (Note On events with velocity < min_velocity are dropped).
    pub min_velocity: u8,
    /// Maximum velocity ceiling.
    pub max_velocity: u8,
    /// Active velocity curve mapping.
    pub velocity_curve: VelocityCurve,
    /// Allow Note On / Note Off events.
    pub allow_notes: bool,
    /// Allow Control Change (CC) events.
    pub allow_cc: bool,
    /// Allowed CC numbers filter (bitset or mask array for 128 CCs).
    pub cc_mask: [bool; 128],
    /// Allow Pitch Bend events.
    pub allow_pitch_bend: bool,
    /// Allow Program Change events.
    pub allow_program_change: bool,
    /// Allow Channel / Polyphonic Aftertouch events.
    pub allow_aftertouch: bool,
    /// Allow System Realtime / Clock events.
    pub allow_system_realtime: bool,
}

impl MidiInputFilter {
    /// Create a filter configuration that passes all incoming MIDI traffic without modification.
    pub fn new_pass_all() -> Self {
        Self {
            channel_mask: 0xFFFF,
            min_note: 0,
            max_note: 127,
            transpose: 0,
            min_velocity: 1,
            max_velocity: 127,
            velocity_curve: VelocityCurve::Linear,
            allow_notes: true,
            allow_cc: true,
            cc_mask: [true; 128],
            allow_pitch_bend: true,
            allow_program_change: true,
            allow_aftertouch: true,
            allow_system_realtime: true,
        }
    }

    /// Enable or disable a specific MIDI channel (1..=16).
    pub fn set_channel_enabled(&mut self, channel_1_based: u8, enabled: bool) {
        if (1..=16).contains(&channel_1_based) {
            let bit = 1 << (channel_1_based - 1);
            if enabled {
                self.channel_mask |= bit;
            } else {
                self.channel_mask &= !bit;
            }
        }
    }

    /// Check if a 0-indexed MIDI channel (0..=15) is enabled in the channel mask.
    pub fn is_channel_enabled(&self, channel_0_based: u8) -> bool {
        if channel_0_based < 16 {
            (self.channel_mask & (1 << channel_0_based)) != 0
        } else {
            false
        }
    }

    /// Enable or disable a specific Control Change (CC) controller number (0..=127).
    pub fn set_cc_enabled(&mut self, cc_number: u8, enabled: bool) {
        if cc_number < 128 {
            self.cc_mask[cc_number as usize] = enabled;
        }
    }

    /// Set active key split / note range bounds (0..=127).
    pub fn set_key_range(&mut self, min_note: u8, max_note: u8) {
        self.min_note = min_note.min(127);
        self.max_note = max_note.clamp(self.min_note, 127);
    }

    /// Set real-time note transpose in semitones (-128..=127).
    pub fn set_transpose(&mut self, semitones: i8) {
        self.transpose = semitones;
    }

    /// Set active velocity mapping curve.
    pub fn set_velocity_curve(&mut self, curve: VelocityCurve) {
        self.velocity_curve = curve;
    }
}

impl Default for MidiInputFilter {
    fn default() -> Self {
        Self::new_pass_all()
    }
}

/// Real-time zero-allocation MIDI filter processing engine.
#[derive(Debug, Clone)]
pub struct MidiFilterEngine {
    /// Active filter parameters and velocity curve settings.
    pub filter: MidiInputFilter,
}

impl MidiFilterEngine {
    /// Create a new real-time MIDI filter engine with specified filter configuration.
    pub fn new(filter: MidiInputFilter) -> Self {
        Self { filter }
    }

    /// Process a single [`MidiEvent`], applying channel filtering, key range splits,
    /// velocity curves, transpose, and message type gating.
    pub fn process_event(&self, event: &MidiEvent) -> Option<MidiEvent> {
        match *event {
            MidiEvent::NoteOn(channel, note, velocity) => {
                if !self.filter.allow_notes || !self.filter.is_channel_enabled(channel) {
                    return None;
                }
                if velocity == 0 {
                    // Velocity 0 Note On handled as Note Off
                    let transposed =
                        (note as i16 + self.filter.transpose as i16).clamp(0, 127) as u8;
                    return Some(MidiEvent::NoteOn(channel, transposed, 0));
                }
                if note < self.filter.min_note || note > self.filter.max_note {
                    return None;
                }
                if velocity < self.filter.min_velocity || velocity > self.filter.max_velocity {
                    return None;
                }
                let mapped_vel = self.filter.velocity_curve.map(velocity);
                let transposed = (note as i16 + self.filter.transpose as i16).clamp(0, 127) as u8;
                Some(MidiEvent::NoteOn(channel, transposed, mapped_vel))
            }
            MidiEvent::NoteOff(channel, note, velocity) => {
                if !self.filter.allow_notes || !self.filter.is_channel_enabled(channel) {
                    return None;
                }
                if note < self.filter.min_note || note > self.filter.max_note {
                    return None;
                }
                let mapped_vel = self.filter.velocity_curve.map(velocity);
                let transposed = (note as i16 + self.filter.transpose as i16).clamp(0, 127) as u8;
                Some(MidiEvent::NoteOff(channel, transposed, mapped_vel))
            }
            MidiEvent::ControlChange(channel, cc, val) => {
                if !self.filter.allow_cc || !self.filter.is_channel_enabled(channel) {
                    return None;
                }
                if !self.filter.cc_mask[cc as usize & 0x7F] {
                    return None;
                }
                Some(MidiEvent::ControlChange(channel, cc, val))
            }
            MidiEvent::PitchBend(channel, val) => {
                if !self.filter.allow_pitch_bend || !self.filter.is_channel_enabled(channel) {
                    return None;
                }
                Some(MidiEvent::PitchBend(channel, val))
            }
            MidiEvent::ProgramChange(channel, program) => {
                if !self.filter.allow_program_change || !self.filter.is_channel_enabled(channel) {
                    return None;
                }
                Some(MidiEvent::ProgramChange(channel, program))
            }
            MidiEvent::Aftertouch(channel, pressure) => {
                if !self.filter.allow_aftertouch || !self.filter.is_channel_enabled(channel) {
                    return None;
                }
                Some(MidiEvent::Aftertouch(channel, pressure))
            }
            MidiEvent::PolyPressure(channel, note, pressure) => {
                if !self.filter.allow_aftertouch || !self.filter.is_channel_enabled(channel) {
                    return None;
                }
                if note < self.filter.min_note || note > self.filter.max_note {
                    return None;
                }
                let transposed = (note as i16 + self.filter.transpose as i16).clamp(0, 127) as u8;
                Some(MidiEvent::PolyPressure(channel, transposed, pressure))
            }
            MidiEvent::SystemRealtime(msg) => {
                if !self.filter.allow_system_realtime {
                    return None;
                }
                Some(MidiEvent::SystemRealtime(msg))
            }
        }
    }

    /// Process a raw 3-byte MIDI packet, applying active filter rules and velocity curves.
    pub fn process_raw(&self, raw: [u8; 3]) -> Option<[u8; 3]> {
        let status = raw[0];
        if status >= 0xF8 {
            let event = MidiEvent::SystemRealtime(status);
            return self.process_event(&event).map(|_| raw);
        }

        let channel = status & 0x0F;
        let cmd = status & 0xF0;

        let event = match cmd {
            0x90 => MidiEvent::NoteOn(channel, raw[1], raw[2]),
            0x80 => MidiEvent::NoteOff(channel, raw[1], raw[2]),
            0xB0 => MidiEvent::ControlChange(channel, raw[1], raw[2]),
            0xE0 => {
                let val = ((raw[2] as u16) << 7) | (raw[1] as u16);
                MidiEvent::PitchBend(channel, val)
            }
            0xC0 => MidiEvent::ProgramChange(channel, raw[1]),
            0xD0 => MidiEvent::Aftertouch(channel, raw[1]),
            0xA0 => MidiEvent::PolyPressure(channel, raw[1], raw[2]),
            _ => return Some(raw),
        };

        self.process_event(&event).map(|proc| match proc {
            MidiEvent::NoteOn(ch, n, v) => [0x90 | (ch & 0x0F), n & 0x7F, v & 0x7F],
            MidiEvent::NoteOff(ch, n, v) => [0x80 | (ch & 0x0F), n & 0x7F, v & 0x7F],
            MidiEvent::ControlChange(ch, cc, v) => [0xB0 | (ch & 0x0F), cc & 0x7F, v & 0x7F],
            MidiEvent::PitchBend(ch, val) => [
                0xE0 | (ch & 0x0F),
                (val & 0x7F) as u8,
                ((val >> 7) & 0x7F) as u8,
            ],
            MidiEvent::ProgramChange(ch, p) => [0xC0 | (ch & 0x0F), p & 0x7F, 0],
            MidiEvent::Aftertouch(ch, p) => [0xD0 | (ch & 0x0F), p & 0x7F, 0],
            MidiEvent::PolyPressure(ch, n, p) => [0xA0 | (ch & 0x0F), n & 0x7F, p & 0x7F],
            MidiEvent::SystemRealtime(msg) => [msg, 0, 0],
        })
    }
}

impl Default for MidiFilterEngine {
    fn default() -> Self {
        Self::new(MidiInputFilter::new_pass_all())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_velocity_curve_mapping() {
        let linear = VelocityCurve::Linear;
        assert_eq!(linear.map(64), 64);
        assert_eq!(linear.map(0), 0);

        let soft = VelocityCurve::Soft;
        assert!(soft.map(64) < 64);

        let hard = VelocityCurve::Hard;
        assert!(hard.map(64) > 64);

        let fixed = VelocityCurve::Fixed(100);
        assert_eq!(fixed.map(10), 100);
        assert_eq!(fixed.map(0), 0);

        let compress = VelocityCurve::Compress { min: 40, max: 100 };
        assert_eq!(compress.map(127), 100);
        assert_eq!(compress.map(1), 40);
    }

    #[test]
    fn test_midi_filter_engine_channel_and_transpose() {
        let mut filter = MidiInputFilter::new_pass_all();
        filter.set_channel_enabled(1, true);
        filter.set_channel_enabled(2, false);
        filter.set_transpose(12);

        let engine = MidiFilterEngine::new(filter);

        let ev_ch1 = MidiEvent::NoteOn(0, 60, 100);
        let processed1 = engine.process_event(&ev_ch1);
        assert_eq!(processed1, Some(MidiEvent::NoteOn(0, 72, 100)));

        let ev_ch2 = MidiEvent::NoteOn(1, 60, 100);
        let processed2 = engine.process_event(&ev_ch2);
        assert_eq!(processed2, None);
    }

    #[test]
    fn test_midi_filter_engine_key_range_and_cc() {
        let mut filter = MidiInputFilter::new_pass_all();
        filter.set_key_range(60, 72);
        filter.set_cc_enabled(7, true);
        filter.set_cc_enabled(1, false);

        let engine = MidiFilterEngine::new(filter);

        assert!(engine
            .process_event(&MidiEvent::NoteOn(0, 59, 100))
            .is_none());
        assert!(engine
            .process_event(&MidiEvent::NoteOn(0, 64, 100))
            .is_some());
        assert!(engine
            .process_event(&MidiEvent::ControlChange(0, 1, 127))
            .is_none());
        assert!(engine
            .process_event(&MidiEvent::ControlChange(0, 7, 127))
            .is_some());
    }

    #[test]
    fn test_raw_midi_packet_filtering() {
        let mut filter = MidiInputFilter::new_pass_all();
        filter.set_transpose(-2);
        filter.set_velocity_curve(VelocityCurve::Fixed(120));

        let engine = MidiFilterEngine::new(filter);
        let raw_in = [0x90, 62, 80]; // Note On Ch 1, Note 62 (D4), Vel 80
        let raw_out = engine.process_raw(raw_in);

        assert_eq!(raw_out, Some([0x90, 60, 120]));
    }
}
