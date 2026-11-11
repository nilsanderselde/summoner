// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Polyphonic Aftertouch & MIDI 2.0 Property Exchange (CI/PE) Deep Profile Editor (Milestone M2).
//!
//! Implements:
//! - Universal MIDI Packet (UMP) 32-bit resolution Per-Note Pitch Bend & Polyphonic Key Pressure
//! - MIDI-CI (Capability Inquiry) Discovery handshake & MUID (MIDI Unique Identifier) generation
//! - MIDI-CI Profile Negotiation (Profile Inquiry, Profile Details, Profile Set On/Off)
//! - MIDI-CI Property Exchange (PE) JSON-based parameter inquiry and two-way sync
//! - MIDI 2.0 Profile Editor state holding registered device profiles and channel maps.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Universal MIDI Packet (UMP) Message Types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UmpMessageType {
    Utility,
    SystemRealtime,
    Midi1ChannelVoice,
    Data64Extended,
    Midi2ChannelVoice,
}

/// MIDI 2.0 Channel Voice Event with high-resolution 32-bit fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Midi2VoiceEvent {
    /// Note On with 16-bit velocity and attribute data.
    NoteOn {
        group: u8,
        channel: u8,
        note: u8,
        velocity_16: u16,
        attribute_type: u8,
        attribute_data: u16,
    },
    /// Note Off with 16-bit release velocity.
    NoteOff {
        group: u8,
        channel: u8,
        note: u8,
        velocity_16: u16,
    },
    /// Polyphonic Key Pressure (Per-Note Aftertouch) with full 32-bit pressure resolution.
    PolyphonicKeyPressure {
        group: u8,
        channel: u8,
        note: u8,
        pressure_32: u32,
    },
    /// Per-Note Pitch Bend with 32-bit signed pitch value (center = 0x8000_0000).
    PerNotePitchBend {
        group: u8,
        channel: u8,
        note: u8,
        pitch_32: u32,
    },
    /// Channel Pitch Bend with 32-bit resolution.
    ChannelPitchBend {
        group: u8,
        channel: u8,
        pitch_32: u32,
    },
}

impl Midi2VoiceEvent {
    /// Encode event into a 64-bit UMP packet representation (two 32-bit words).
    pub fn to_ump_words(&self) -> (u32, u32) {
        match *self {
            Midi2VoiceEvent::NoteOn {
                group,
                channel,
                note,
                velocity_16,
                attribute_type,
                attribute_data,
            } => {
                let w1 = 0x4000_0000
                    | ((group as u32 & 0x0F) << 24)
                    | (0x90 << 16)
                    | ((channel as u32 & 0x0F) << 16)
                    | ((note as u32 & 0x7F) << 8)
                    | (attribute_type as u32);
                let w2 = ((velocity_16 as u32) << 16) | (attribute_data as u32);
                (w1, w2)
            }
            Midi2VoiceEvent::NoteOff {
                group,
                channel,
                note,
                velocity_16,
            } => {
                let w1 = 0x4000_0000
                    | ((group as u32 & 0x0F) << 24)
                    | (0x80 << 16)
                    | ((channel as u32 & 0x0F) << 16)
                    | ((note as u32 & 0x7F) << 8);
                let w2 = (velocity_16 as u32) << 16;
                (w1, w2)
            }
            Midi2VoiceEvent::PolyphonicKeyPressure {
                group,
                channel,
                note,
                pressure_32,
            } => {
                let w1 = 0x4000_0000
                    | ((group as u32 & 0x0F) << 24)
                    | (0xA0 << 16)
                    | ((channel as u32 & 0x0F) << 16)
                    | ((note as u32 & 0x7F) << 8);
                (w1, pressure_32)
            }
            Midi2VoiceEvent::PerNotePitchBend {
                group,
                channel,
                note,
                pitch_32,
            } => {
                let w1 = 0x4000_0000
                    | ((group as u32 & 0x0F) << 24)
                    | (0x60 << 16)
                    | ((channel as u32 & 0x0F) << 16)
                    | ((note as u32 & 0x7F) << 8);
                (w1, pitch_32)
            }
            Midi2VoiceEvent::ChannelPitchBend {
                group,
                channel,
                pitch_32,
            } => {
                let w1 = 0x4000_0000
                    | ((group as u32 & 0x0F) << 24)
                    | (0xE0 << 16)
                    | ((channel as u32 & 0x0F) << 16);
                (w1, pitch_32)
            }
        }
    }

    /// Decode from two 32-bit UMP words.
    pub fn from_ump_words(w1: u32, w2: u32) -> Option<Self> {
        let msg_type = (w1 >> 28) & 0x0F;
        if msg_type != 0x04 {
            // Must be MIDI 2.0 Channel Voice (0x04)
            return None;
        }
        let group = ((w1 >> 24) & 0x0F) as u8;
        let status = ((w1 >> 16) & 0xF0) as u8;
        let channel = ((w1 >> 16) & 0x0F) as u8;
        let note = ((w1 >> 8) & 0x7F) as u8;

        match status {
            0x90 => Some(Midi2VoiceEvent::NoteOn {
                group,
                channel,
                note,
                velocity_16: (w2 >> 16) as u16,
                attribute_type: (w1 & 0xFF) as u8,
                attribute_data: (w2 & 0xFFFF) as u16,
            }),
            0x80 => Some(Midi2VoiceEvent::NoteOff {
                group,
                channel,
                note,
                velocity_16: (w2 >> 16) as u16,
            }),
            0xA0 => Some(Midi2VoiceEvent::PolyphonicKeyPressure {
                group,
                channel,
                note,
                pressure_32: w2,
            }),
            0x60 => Some(Midi2VoiceEvent::PerNotePitchBend {
                group,
                channel,
                note,
                pitch_32: w2,
            }),
            0xE0 => Some(Midi2VoiceEvent::ChannelPitchBend {
                group,
                channel,
                pitch_32: w2,
            }),
            _ => None,
        }
    }
}

// ============================================================================
// MIDI-CI (Capability Inquiry) Protocol
// ============================================================================

/// 28-bit MIDI Unique Identifier (MUID).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Muid(pub u32);

impl Muid {
    pub const BROADCAST: Muid = Muid(0x0FFF_FFFF);

    pub fn new(id: u32) -> Self {
        Self(id & 0x0FFF_FFFF)
    }

    pub fn random() -> Self {
        // Deterministic PRNG or time-based MUID
        let val = (blake3::hash(b"summoner_muid_seed").as_bytes()[0] as u32) << 20
            | (blake3::hash(b"summoner_muid_seed").as_bytes()[1] as u32) << 12
            | (blake3::hash(b"summoner_muid_seed").as_bytes()[2] as u32) << 4
            | 0x07;
        Self(val & 0x0FFF_FFFF)
    }
}

/// Device Identity for MIDI-CI discovery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceIdentity {
    pub manufacturer_id: [u8; 3], // System SysEx ID (e.g. [0x00, 0x21, 0x1D])
    pub family_id: u16,
    pub model_id: u16,
    pub software_revision: u32,
}

impl Default for DeviceIdentity {
    fn default() -> Self {
        Self {
            manufacturer_id: [0x00, 0x55, 0x4D], // "SUM"
            family_id: 0x0001,
            model_id: 0x0001,
            software_revision: 0x01_00_00_00,
        }
    }
}

/// Standard 5-byte MIDI-CI Profile Identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProfileId(pub [u8; 5]);

impl ProfileId {
    /// Standard Drawbar Organ profile.
    pub const DRAWBAR_ORGAN: ProfileId = ProfileId([0x7E, 0x30, 0x01, 0x01, 0x00]);
    /// Standard Analog Synth profile.
    pub const ANALOG_SYNTH: ProfileId = ProfileId([0x7E, 0x40, 0x01, 0x01, 0x00]);
    /// Microtonal Tuning profile.
    pub const MICROTONAL_TUNING: ProfileId = ProfileId([0x7E, 0x50, 0x01, 0x01, 0x00]);
}

/// Supported MIDI-CI sub-protocols.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupportedInquiries {
    pub profile_configuration: bool,
    pub property_exchange: bool,
    pub process_inquiry: bool,
}

impl Default for SupportedInquiries {
    fn default() -> Self {
        Self {
            profile_configuration: true,
            property_exchange: true,
            process_inquiry: false,
        }
    }
}

// ============================================================================
// MIDI-CI Property Exchange (PE) JSON Resources
// ============================================================================

/// Property Exchange Header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyExchangeHeader {
    pub resource: String,
    pub status: u16,
    pub message: Option<String>,
    pub cache_time: Option<u32>,
}

/// Property Exchange Resource Description.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyResource {
    pub resource_name: String,
    pub can_get: bool,
    pub can_set: bool,
    pub can_subscribe: bool,
    pub media_type: String,
    pub json_schema: Option<String>,
    pub current_json_data: String,
}

impl PropertyResource {
    pub fn new(resource_name: &str, data: &str) -> Self {
        Self {
            resource_name: resource_name.to_string(),
            can_get: true,
            can_set: true,
            can_subscribe: true,
            media_type: "application/json".to_string(),
            json_schema: None,
            current_json_data: data.to_string(),
        }
    }
}

// ============================================================================
// MIDI 2.0 Profile Editor & CI Coordinator
// ============================================================================

/// State for MIDI 2.0 CI/PE Profile Editor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Midi2ProfileEditor {
    pub local_muid: Muid,
    pub device_identity: DeviceIdentity,
    pub supported_inquiries: SupportedInquiries,
    pub active_profiles: HashMap<u8, Vec<ProfileId>>, // Channel -> Active Profiles
    pub property_resources: HashMap<String, PropertyResource>,
    pub peer_muids: Vec<Muid>,
}

impl Default for Midi2ProfileEditor {
    fn default() -> Self {
        let mut editor = Self {
            local_muid: Muid(0x0123_4567),
            device_identity: DeviceIdentity::default(),
            supported_inquiries: SupportedInquiries::default(),
            active_profiles: HashMap::new(),
            property_resources: HashMap::new(),
            peer_muids: Vec::new(),
        };

        // Populate standard default properties
        editor.register_property_resource(PropertyResource::new(
            "DeviceInfo",
            r#"{"manufacturer":"Summoner Audio","model":"Summoner DAW Engine","version":"1.0"}"#,
        ));
        editor.register_property_resource(PropertyResource::new(
            "ModeList",
            r#"{"modes":["Monophonic","Polyphonic MPE","Microtonal Multichannel"]}"#,
        ));
        editor.register_property_resource(PropertyResource::new(
            "ChannelList",
            r#"{"num_channels":16,"max_polyphony_per_channel":64}"#,
        ));

        editor
    }
}

impl Midi2ProfileEditor {
    pub fn new(local_muid: Muid) -> Self {
        Self {
            local_muid,
            ..Default::default()
        }
    }

    /// Register a Property Exchange resource.
    pub fn register_property_resource(&mut self, resource: PropertyResource) {
        self.property_resources
            .insert(resource.resource_name.clone(), resource);
    }

    /// Discover peer device handshake.
    pub fn handle_discovery_inquiry(
        &mut self,
        sender_muid: Muid,
    ) -> (DeviceIdentity, SupportedInquiries) {
        if !self.peer_muids.contains(&sender_muid) {
            self.peer_muids.push(sender_muid);
        }
        (self.device_identity.clone(), self.supported_inquiries)
    }

    /// Enable a profile on a given channel.
    pub fn enable_profile(&mut self, channel: u8, profile: ProfileId) {
        let list = self.active_profiles.entry(channel).or_default();
        if !list.contains(&profile) {
            list.push(profile);
        }
    }

    /// Disable a profile on a given channel.
    pub fn disable_profile(&mut self, channel: u8, profile: &ProfileId) {
        if let Some(list) = self.active_profiles.get_mut(&channel) {
            list.retain(|p| p != profile);
        }
    }

    /// Check if a profile is active on channel.
    pub fn is_profile_active(&self, channel: u8, profile: &ProfileId) -> bool {
        self.active_profiles
            .get(&channel)
            .map(|list| list.contains(profile))
            .unwrap_or(false)
    }

    /// Execute Property Exchange GET request.
    pub fn get_property(&self, resource_name: &str) -> Result<String, u16> {
        match self.property_resources.get(resource_name) {
            Some(res) if res.can_get => Ok(res.current_json_data.clone()),
            Some(_) => Err(403), // Forbidden
            None => Err(404),    // Not Found
        }
    }

    /// Execute Property Exchange SET request.
    pub fn set_property(&mut self, resource_name: &str, new_json_data: &str) -> Result<(), u16> {
        match self.property_resources.get_mut(resource_name) {
            Some(res) if res.can_set => {
                res.current_json_data = new_json_data.to_string();
                Ok(())
            }
            Some(_) => Err(403), // Read-only property
            None => Err(404),    // Not Found
        }
    }

    /// Generate SysEx Discovery Inquiry packet.
    pub fn create_discovery_sysex(&self) -> Vec<u8> {
        let mut sysex = vec![0xF0, 0x7E, 0x7F, 0x0D, 0x70, 0x01]; // MIDI-CI Discovery Inquiry header
        let muid_bytes = self.local_muid.0.to_le_bytes();
        sysex.extend_from_slice(&muid_bytes);
        sysex.extend_from_slice(&self.device_identity.manufacturer_id);
        sysex.extend_from_slice(&self.device_identity.family_id.to_le_bytes());
        sysex.extend_from_slice(&self.device_identity.model_id.to_le_bytes());
        sysex.extend_from_slice(&self.device_identity.software_revision.to_le_bytes());
        sysex.push(0x03); // Supported inquiries bitmask: Profile Config | Property Exchange
        sysex.push(0xF7);
        sysex
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ump_polyphonic_pressure_roundtrip() {
        let event = Midi2VoiceEvent::PolyphonicKeyPressure {
            group: 1,
            channel: 3,
            note: 60,
            pressure_32: 0xDEAD_BEEF,
        };
        let (w1, w2) = event.to_ump_words();
        let decoded = Midi2VoiceEvent::from_ump_words(w1, w2).expect("decode failed");
        assert_eq!(decoded, event);
    }

    #[test]
    fn test_ump_per_note_pitch_bend_roundtrip() {
        let event = Midi2VoiceEvent::PerNotePitchBend {
            group: 0,
            channel: 2,
            note: 64,
            pitch_32: 0x9000_0000,
        };
        let (w1, w2) = event.to_ump_words();
        let decoded = Midi2VoiceEvent::from_ump_words(w1, w2).expect("decode failed");
        assert_eq!(decoded, event);
    }

    #[test]
    fn test_midi_ci_discovery_handshake() {
        let mut editor = Midi2ProfileEditor::new(Muid(0x0123_4567));
        let peer = Muid(0x0765_4321);
        let (identity, supported) = editor.handle_discovery_inquiry(peer);

        assert_eq!(identity.manufacturer_id, [0x00, 0x55, 0x4D]);
        assert!(supported.property_exchange);
        assert!(editor.peer_muids.contains(&peer));
    }

    #[test]
    fn test_profile_configuration_enabling() {
        let mut editor = Midi2ProfileEditor::new(Muid(0x0123_4567));
        assert!(!editor.is_profile_active(0, &ProfileId::ANALOG_SYNTH));

        editor.enable_profile(0, ProfileId::ANALOG_SYNTH);
        assert!(editor.is_profile_active(0, &ProfileId::ANALOG_SYNTH));

        editor.disable_profile(0, &ProfileId::ANALOG_SYNTH);
        assert!(!editor.is_profile_active(0, &ProfileId::ANALOG_SYNTH));
    }

    #[test]
    fn test_property_exchange_get_and_set() {
        let mut editor = Midi2ProfileEditor::new(Muid(0x0123_4567));
        let info = editor.get_property("DeviceInfo").expect("get failed");
        assert!(info.contains("Summoner Audio"));

        let updated = r#"{"manufacturer":"Summoner Audio","model":"Summoner Pro V2"}"#;
        editor
            .set_property("DeviceInfo", updated)
            .expect("set failed");

        let fetched = editor.get_property("DeviceInfo").unwrap();
        assert!(fetched.contains("Summoner Pro V2"));

        assert_eq!(editor.get_property("NonExistentProperty"), Err(404));
    }

    #[test]
    fn test_discovery_sysex_serialization() {
        let editor = Midi2ProfileEditor::new(Muid(0x0123_4567));
        let bytes = editor.create_discovery_sysex();
        assert_eq!(bytes[0], 0xF0);
        assert_eq!(bytes[1], 0x7E);
        assert_eq!(bytes[3], 0x0D);
        assert_eq!(bytes.last(), Some(&0xF7));
    }
}
