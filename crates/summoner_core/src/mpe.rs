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
