// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

use crate::sequence::{SequenceTrack, TrackerStep};

/// Represent standard MIDI protocol messages for real-time engine event handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MidiEvent {
    /// Note On event with (channel 0-15, note 0-127, velocity 0-127).
    NoteOn(u8, u8, u8),
    /// Note Off event with (channel 0-15, note 0-127, release velocity 0-127).
    NoteOff(u8, u8, u8),
    /// Pitch Bend event with (channel 0-15, 14-bit unsigned pitch value 0-16383).
    PitchBend(u8, u16),
    /// Control Change event with (channel 0-15, controller number 0-127, controller value 0-127).
    ControlChange(u8, u8, u8),
    /// Program Change event with (channel 0-15, program number 0-127).
    ProgramChange(u8, u8),
    /// Channel Pressure / Aftertouch event with (channel 0-15, pressure value 0-127).
    Aftertouch(u8, u8),
    /// Polyphonic Key Pressure event with (channel 0-15, note 0-127, pressure value 0-127).
    PolyPressure(u8, u8, u8),
    /// System Realtime status byte message (e.g. 0xF8 Timing Clock, 0xFA Start, 0xFC Stop).
    SystemRealtime(u8),
}

impl MidiEvent {
    /// Parse standard raw MIDI 1.0 bytes into a typed `MidiEvent`.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() {
            return None;
        }
        let status = bytes[0];
        let message_type = status & 0xF0;
        let channel = status & 0x0F;

        match message_type {
            0x90 => {
                if bytes.len() >= 3 {
                    let note = bytes[1] & 0x7F;
                    let vel = bytes[2] & 0x7F;
                    if vel > 0 {
                        Some(MidiEvent::NoteOn(channel, note, vel))
                    } else {
                        Some(MidiEvent::NoteOff(channel, note, 0))
                    }
                } else {
                    None
                }
            }
            0x80 => {
                if bytes.len() >= 3 {
                    let note = bytes[1] & 0x7F;
                    let vel = bytes[2] & 0x7F;
                    Some(MidiEvent::NoteOff(channel, note, vel))
                } else {
                    None
                }
            }
            0xB0 => {
                if bytes.len() >= 3 {
                    let cc = bytes[1] & 0x7F;
                    let val = bytes[2] & 0x7F;
                    Some(MidiEvent::ControlChange(channel, cc, val))
                } else {
                    None
                }
            }
            0xE0 => {
                if bytes.len() >= 3 {
                    let lsb = (bytes[1] & 0x7F) as u16;
                    let msb = (bytes[2] & 0x7F) as u16;
                    let bend = (msb << 7) | lsb;
                    Some(MidiEvent::PitchBend(channel, bend))
                } else {
                    None
                }
            }
            0xC0 => {
                if bytes.len() >= 2 {
                    let prog = bytes[1] & 0x7F;
                    Some(MidiEvent::ProgramChange(channel, prog))
                } else {
                    None
                }
            }
            0xD0 => {
                if bytes.len() >= 2 {
                    let press = bytes[1] & 0x7F;
                    Some(MidiEvent::Aftertouch(channel, press))
                } else {
                    None
                }
            }
            0xA0 => {
                if bytes.len() >= 3 {
                    let note = bytes[1] & 0x7F;
                    let press = bytes[2] & 0x7F;
                    Some(MidiEvent::PolyPressure(channel, note, press))
                } else {
                    None
                }
            }
            0xF0 => {
                if status >= 0xF8 {
                    Some(MidiEvent::SystemRealtime(status))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

/// Standard Standard MIDI File (.mid) parser.
pub struct MidiFileParser {
    data: Vec<u8>,
    pos: usize,
}

impl MidiFileParser {
    pub fn new(data: &[u8]) -> Self {
        Self {
            data: data.to_vec(),
            pos: 0,
        }
    }

    fn read_u8(&mut self) -> Option<u8> {
        if self.pos < self.data.len() {
            let val = self.data[self.pos];
            self.pos += 1;
            Some(val)
        } else {
            None
        }
    }

    fn read_u16(&mut self) -> Option<u16> {
        let b1 = self.read_u8()? as u16;
        let b2 = self.read_u8()? as u16;
        Some((b1 << 8) | b2)
    }

    fn read_u32(&mut self) -> Option<u32> {
        let b1 = self.read_u8()? as u32;
        let b2 = self.read_u8()? as u32;
        let b3 = self.read_u8()? as u32;
        let b4 = self.read_u8()? as u32;
        Some((b1 << 24) | (b2 << 16) | (b3 << 8) | b4)
    }

    fn read_vlq(&mut self) -> Option<u32> {
        let mut val = 0;
        loop {
            let b = self.read_u8()?;
            val = (val << 7) | (b & 0x7F) as u32;
            if b & 0x80 == 0 {
                break;
            }
        }
        Some(val)
    }

    pub fn parse(&mut self) -> Result<Vec<SequenceTrack>, &'static str> {
        if self.read_u32() != Some(0x4D546864) {
            return Err("Invalid MThd signature");
        }
        let _header_len = self.read_u32().ok_or("EOF")?;
        let _format = self.read_u16().ok_or("EOF")?;
        let tracks = self.read_u16().ok_or("EOF")?;
        let _ticks_per_qn = self.read_u16().ok_or("EOF")?;

        let mut seq_tracks = Vec::new();

        for track_id in 1..=tracks {
            if self.read_u32() != Some(0x4D54726B) {
                // If it's not a track, skip
                break;
            }
            let track_len = self.read_u32().ok_or("EOF")? as usize;
            let end_pos = self.pos + track_len;
            let mut steps = Vec::new();
            let mut running_status = 0;

            // To properly convert to TrackerStep, we need quantize or just populate them.
            // As a simplified approach, we just append to steps.
            while self.pos < end_pos {
                let _delta = self.read_vlq().ok_or("EOF")?;
                let mut status = self.read_u8().ok_or("EOF")?;

                if status < 0x80 {
                    status = running_status;
                    self.pos -= 1;
                } else {
                    running_status = status;
                }

                if status == 0xFF {
                    let _meta_type = self.read_u8().ok_or("EOF")?;
                    let meta_len = self.read_vlq().ok_or("EOF")? as usize;
                    self.pos += meta_len;
                } else if status == 0xF0 || status == 0xF7 {
                    let sys_len = self.read_vlq().ok_or("EOF")? as usize;
                    self.pos += sys_len;
                } else {
                    let cmd = status & 0xF0;
                    let _channel = status & 0x0F;

                    if cmd == 0x80 || cmd == 0x90 {
                        let note = self.read_u8().ok_or("EOF")?;
                        let vel = self.read_u8().ok_or("EOF")?;
                        if cmd == 0x90 && vel > 0 {
                            steps.push(TrackerStep::new(note as f64, vel as f32 / 127.0, 1.0));
                        }
                    } else if cmd == 0xA0 || cmd == 0xB0 || cmd == 0xE0 {
                        self.pos += 2;
                    } else if cmd == 0xC0 || cmd == 0xD0 {
                        self.pos += 1;
                    }
                }
            }

            seq_tracks.push(SequenceTrack::new(
                track_id as u64,
                format!("Track {}", track_id),
                0.25, // 1/16th note default
                steps,
            ));
            self.pos = end_pos;
        }

        Ok(seq_tracks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_event_from_bytes_parsing() {
        assert_eq!(
            MidiEvent::from_bytes(&[0x90, 60, 100]),
            Some(MidiEvent::NoteOn(0, 60, 100))
        );
        assert_eq!(
            MidiEvent::from_bytes(&[0x90, 60, 0]),
            Some(MidiEvent::NoteOff(0, 60, 0))
        );
        assert_eq!(
            MidiEvent::from_bytes(&[0x81, 64, 64]),
            Some(MidiEvent::NoteOff(1, 64, 64))
        );
        assert_eq!(
            MidiEvent::from_bytes(&[0xB0, 7, 127]),
            Some(MidiEvent::ControlChange(0, 7, 127))
        );
        assert_eq!(
            MidiEvent::from_bytes(&[0xE2, 0x00, 0x40]),
            Some(MidiEvent::PitchBend(2, 8192))
        );
        assert_eq!(
            MidiEvent::from_bytes(&[0xF8]),
            Some(MidiEvent::SystemRealtime(0xF8))
        );
    }
}
