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

//! Native MIDI Polyphonic Expression (MPE) per-voice event routing.

/// Unique MPE voice instance identifier.
pub type MpeVoiceId = u32;

/// MPE expressate event types (Note, Pitch Bend, Pressure, Timbre).
#[derive(Debug, Clone, PartialEq)]
pub enum MpeEvent {
    NoteOn {
        voice_id: MpeVoiceId,
        channel: u8,
        note: f32,
        velocity: f32,
    },
    NoteOff {
        voice_id: MpeVoiceId,
        channel: u8,
        release_velocity: f32,
    },
    PitchBend {
        voice_id: MpeVoiceId,
        semitones: f32,
    },
    Pressure {
        voice_id: MpeVoiceId,
        pressure: f32,
    },
    Timbre {
        voice_id: MpeVoiceId,
        timbre: f32,
    },
}

/// Active state of a single MPE voice.
#[derive(Debug, Clone, PartialEq)]
pub struct MpeVoiceState {
    pub voice_id: MpeVoiceId,
    pub channel: u8,
    pub base_note: f32,
    pub velocity: f32,
    pub pitch_bend_semitones: f32,
    pub pressure: f32,
    pub timbre: f32,
    pub is_active: bool,
}

impl MpeVoiceState {
    pub fn effective_note(&self) -> f32 {
        self.base_note + self.pitch_bend_semitones
    }
}

/// Maximum concurrent voices supported per MPE router instance.
pub const MAX_MPE_VOICES: usize = 16;

/// Fixed-capacity MPE voice allocation manager (zero heap allocation during event routing).
#[derive(Debug)]
pub struct MpeRouter {
    pub voices: [MpeVoiceState; MAX_MPE_VOICES],
}

impl MpeRouter {
    pub fn new() -> Self {
        const INACTIVE_VOICE: MpeVoiceState = MpeVoiceState {
            voice_id: 0,
            channel: 0,
            base_note: 60.0,
            velocity: 0.0,
            pitch_bend_semitones: 0.0,
            pressure: 0.0,
            timbre: 0.0,
            is_active: false,
        };

        Self {
            voices: [INACTIVE_VOICE; MAX_MPE_VOICES],
        }
    }

    /// Dispatch MPE event to active voice states.
    pub fn dispatch(&mut self, event: &MpeEvent) {
        match event {
            MpeEvent::NoteOn {
                voice_id,
                channel,
                note,
                velocity,
            } => {
                for slot in self.voices.iter_mut() {
                    if !slot.is_active || slot.voice_id == *voice_id {
                        slot.voice_id = *voice_id;
                        slot.channel = *channel;
                        slot.base_note = *note;
                        slot.velocity = *velocity;
                        slot.pitch_bend_semitones = 0.0;
                        slot.pressure = 0.0;
                        slot.timbre = 0.0;
                        slot.is_active = true;
                        break;
                    }
                }
            }
            MpeEvent::NoteOff { voice_id, .. } => {
                for slot in self.voices.iter_mut() {
                    if slot.is_active && slot.voice_id == *voice_id {
                        slot.is_active = false;
                        break;
                    }
                }
            }
            MpeEvent::PitchBend {
                voice_id,
                semitones,
            } => {
                for slot in self.voices.iter_mut() {
                    if slot.is_active && slot.voice_id == *voice_id {
                        slot.pitch_bend_semitones = *semitones;
                        break;
                    }
                }
            }
            MpeEvent::Pressure { voice_id, pressure } => {
                for slot in self.voices.iter_mut() {
                    if slot.is_active && slot.voice_id == *voice_id {
                        slot.pressure = *pressure;
                        break;
                    }
                }
            }
            MpeEvent::Timbre { voice_id, timbre } => {
                for slot in self.voices.iter_mut() {
                    if slot.is_active && slot.voice_id == *voice_id {
                        slot.timbre = *timbre;
                        break;
                    }
                }
            }
        }
    }
}

impl Default for MpeRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// MPE Expression Curve Types (Step 1265).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpressionCurveType {
    Linear,
    Logarithmic,
    Exponential,
    Sigmoid,
}

/// Real-time MIDI MPE expression curve editor and per-note pitch bend mapping (Step 1265).
#[derive(Debug, Clone)]
pub struct MpeExpressionCurveEditor {
    pub pitch_bend_range_semitones: f32,
    pub velocity_curve: ExpressionCurveType,
    pub pressure_curve: ExpressionCurveType,
    pub timbre_curve: ExpressionCurveType,
    pub curve_curvature: f32,
}

impl Default for MpeExpressionCurveEditor {
    fn default() -> Self {
        Self {
            pitch_bend_range_semitones: 48.0,
            velocity_curve: ExpressionCurveType::Linear,
            pressure_curve: ExpressionCurveType::Linear,
            timbre_curve: ExpressionCurveType::Linear,
            curve_curvature: 1.0,
        }
    }
}

impl MpeExpressionCurveEditor {
    pub fn new(pitch_bend_range_semitones: f32) -> Self {
        Self {
            pitch_bend_range_semitones: pitch_bend_range_semitones.clamp(1.0, 96.0),
            ..Default::default()
        }
    }

    pub fn map_expression_value(&self, value: f32, curve: ExpressionCurveType) -> f32 {
        let v = value.clamp(0.0, 1.0);
        match curve {
            ExpressionCurveType::Linear => v,
            ExpressionCurveType::Logarithmic => {
                (1.0 + v * (self.curve_curvature - 1.0)).ln() / self.curve_curvature.ln().max(1e-5)
            }
            ExpressionCurveType::Exponential => v.powf(self.curve_curvature),
            ExpressionCurveType::Sigmoid => {
                1.0 / (1.0 + (-((v - 0.5) * 10.0 * self.curve_curvature)).exp())
            }
        }
    }

    pub fn map_pitch_bend(&self, raw_14bit: i16) -> f32 {
        let norm = (raw_14bit as f32) / 8191.0;
        norm.clamp(-1.0, 1.0) * self.pitch_bend_range_semitones
    }
}

/// MPE Zone configuration (Task B5).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MpeZone {
    /// True for Lower Zone (master ch 0, member ch 1..=14). False for Upper Zone (master ch 15, member ch 1..=14).
    pub is_lower_zone: bool,
    /// Per-note pitch bend range in semitones (standard MPE default is 48.0).
    pub pitch_bend_range_semitones: f32,
}

impl Default for MpeZone {
    fn default() -> Self {
        Self {
            is_lower_zone: true,
            pitch_bend_range_semitones: 48.0,
        }
    }
}

impl MpeZone {
    /// Check whether a 0-indexed MIDI channel is a member voice channel in this zone.
    pub fn is_member_channel(&self, ch: u8) -> bool {
        if self.is_lower_zone {
            (1..=14).contains(&ch)
        } else {
            (0..=14).contains(&ch)
        }
    }

    /// Return master management channel for this zone (0 or 15).
    pub fn master_channel(&self) -> u8 {
        if self.is_lower_zone {
            0
        } else {
            15
        }
    }
}

/// Real-time live MIDI stream to MPE event parser (Task B5).
///
/// Converts incoming standard channelized MIDI messages into per-voice MPE events
/// (per-note pitch bend, poly pressure, CC74 slide/timbre, note on/off) with zero heap allocation.
#[derive(Debug)]
pub struct MpeLiveStreamParser {
    pub zone: MpeZone,
    pub router: MpeRouter,
    pub channel_to_voice: [Option<MpeVoiceId>; 16],
    next_voice_id: MpeVoiceId,
}

impl Default for MpeLiveStreamParser {
    fn default() -> Self {
        Self::new(MpeZone::default())
    }
}

impl MpeLiveStreamParser {
    pub fn new(zone: MpeZone) -> Self {
        Self {
            zone,
            router: MpeRouter::new(),
            channel_to_voice: [None; 16],
            next_voice_id: 1,
        }
    }

    /// Process a typed `MidiEvent` from live hardware, updating voice state and returning an `MpeEvent` if applicable.
    pub fn process_event(&mut self, event: &crate::midi::MidiEvent) -> Option<MpeEvent> {
        match *event {
            crate::midi::MidiEvent::NoteOn(ch, note, vel) => {
                if !self.zone.is_member_channel(ch) {
                    return None;
                }
                let voice_id = self.next_voice_id;
                self.next_voice_id = self.next_voice_id.wrapping_add(1).max(1);
                self.channel_to_voice[ch as usize] = Some(voice_id);

                let mpe_evt = MpeEvent::NoteOn {
                    voice_id,
                    channel: ch,
                    note: note as f32,
                    velocity: (vel as f32) / 127.0,
                };
                self.router.dispatch(&mpe_evt);
                Some(mpe_evt)
            }
            crate::midi::MidiEvent::NoteOff(ch, _note, rel_vel) => {
                if !self.zone.is_member_channel(ch) {
                    return None;
                }
                if let Some(voice_id) = self.channel_to_voice[ch as usize].take() {
                    let mpe_evt = MpeEvent::NoteOff {
                        voice_id,
                        channel: ch,
                        release_velocity: (rel_vel as f32) / 127.0,
                    };
                    self.router.dispatch(&mpe_evt);
                    Some(mpe_evt)
                } else {
                    None
                }
            }
            crate::midi::MidiEvent::PitchBend(ch, raw_14bit) => {
                if !self.zone.is_member_channel(ch) {
                    return None;
                }
                if let Some(voice_id) = self.channel_to_voice[ch as usize] {
                    // 14-bit unsigned 0..16383 centered at 8192
                    let norm = (raw_14bit as f32 - 8192.0) / 8192.0;
                    let semitones = norm.clamp(-1.0, 1.0) * self.zone.pitch_bend_range_semitones;
                    let mpe_evt = MpeEvent::PitchBend {
                        voice_id,
                        semitones,
                    };
                    self.router.dispatch(&mpe_evt);
                    Some(mpe_evt)
                } else {
                    None
                }
            }
            crate::midi::MidiEvent::Aftertouch(ch, press) => {
                if !self.zone.is_member_channel(ch) {
                    return None;
                }
                if let Some(voice_id) = self.channel_to_voice[ch as usize] {
                    let mpe_evt = MpeEvent::Pressure {
                        voice_id,
                        pressure: (press as f32) / 127.0,
                    };
                    self.router.dispatch(&mpe_evt);
                    Some(mpe_evt)
                } else {
                    None
                }
            }
            crate::midi::MidiEvent::PolyPressure(ch, _note, press) => {
                if !self.zone.is_member_channel(ch) {
                    return None;
                }
                if let Some(voice_id) = self.channel_to_voice[ch as usize] {
                    let mpe_evt = MpeEvent::Pressure {
                        voice_id,
                        pressure: (press as f32) / 127.0,
                    };
                    self.router.dispatch(&mpe_evt);
                    Some(mpe_evt)
                } else {
                    None
                }
            }
            crate::midi::MidiEvent::ControlChange(ch, 74, timbre) => {
                // CC 74 is standard MPE Timbre / Slide
                if !self.zone.is_member_channel(ch) {
                    return None;
                }
                if let Some(voice_id) = self.channel_to_voice[ch as usize] {
                    let mpe_evt = MpeEvent::Timbre {
                        voice_id,
                        timbre: (timbre as f32) / 127.0,
                    };
                    self.router.dispatch(&mpe_evt);
                    Some(mpe_evt)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::midi::MidiEvent;

    #[test]
    fn test_mpe_live_stream_parser_voice_lifecycle() {
        let mut parser = MpeLiveStreamParser::new(MpeZone {
            is_lower_zone: true,
            pitch_bend_range_semitones: 48.0,
        });

        // Channel 2 Note On (middle C = 60, vel = 100)
        let evt1 = MidiEvent::NoteOn(2, 60, 100);
        let mpe1 = parser.process_event(&evt1);
        assert!(mpe1.is_some());
        if let Some(MpeEvent::NoteOn {
            voice_id,
            channel,
            note,
            velocity,
        }) = mpe1
        {
            assert_eq!(channel, 2);
            assert_eq!(note, 60.0);
            assert!((velocity - 100.0 / 127.0).abs() < 1e-4);
            let active = parser.router.voices.iter().find(|v| v.voice_id == voice_id);
            assert!(active.is_some());
            assert!(active.unwrap().is_active);
        }

        // Per-note pitch bend on channel 2 (+1.0 full up = 16383 => +48 semitones)
        let evt_pb = MidiEvent::PitchBend(2, 16383);
        let mpe_pb = parser.process_event(&evt_pb);
        assert!(mpe_pb.is_some());
        if let Some(MpeEvent::PitchBend { semitones, .. }) = mpe_pb {
            assert!((semitones - 48.0).abs() < 0.1);
        }

        // Per-note slide / timbre (CC 74 = 80)
        let evt_slide = MidiEvent::ControlChange(2, 74, 80);
        let mpe_slide = parser.process_event(&evt_slide);
        assert!(mpe_slide.is_some());
        if let Some(MpeEvent::Timbre { timbre, .. }) = mpe_slide {
            assert!((timbre - 80.0 / 127.0).abs() < 1e-4);
        }

        // Channel 2 Note Off
        let evt_off = MidiEvent::NoteOff(2, 60, 64);
        let mpe_off = parser.process_event(&evt_off);
        assert!(mpe_off.is_some());
        assert!(!parser
            .router
            .voices
            .iter()
            .any(|v| v.channel == 2 && v.is_active));
    }
}
