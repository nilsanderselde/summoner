// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Dedicated Hardware Control Surface Profiles (Milestone M1).
//!
//! Provides bidirectional hardware control surface integration for:
//! - Novation Launchpad Pro (8x8 RGB grid, mode switching, SysEx LED palettes)
//! - Ableton Push 2 / Push 3 (8x8 MPE pad matrix, 8 rotary encoders, 960x160 display state)
//! - PreSonus FaderPort (100mm motorized fader with force-feedback, transport, pan encoder)
//!
//! Handles raw MIDI/SysEx input decoding into deterministic DAW actions and
//! serializes outgoing LED/display/motor state updates.

use serde::{Deserialize, Serialize};

/// High-level DAW actions triggered by hardware control surfaces.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SurfaceAction {
    TransportPlay,
    TransportStop,
    TransportRecord,
    TransportToggleLoop,
    TransportFastForward,
    TransportRewind,
    SetTrackVolume {
        track_index: usize,
        volume: f32,
    },
    SetTrackPan {
        track_index: usize,
        pan: f32,
    },
    SetTrackMute {
        track_index: usize,
        muted: bool,
    },
    SetTrackSolo {
        track_index: usize,
        soloed: bool,
    },
    SetTrackArm {
        track_index: usize,
        armed: bool,
    },
    SelectTrack {
        track_index: usize,
    },
    PadTrigger {
        row: u8,
        col: u8,
        velocity: u8,
        pressed: bool,
    },
    EncoderTweak {
        encoder_index: usize,
        delta: f32,
    },
    FaderTouch {
        channel: usize,
        touching: bool,
    },
    FaderMove {
        channel: usize,
        position: f32,
    },
    SceneLaunch {
        scene_index: usize,
    },
}

/// Outgoing MIDI/SysEx packet to be sent to a physical control surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceOutputPacket {
    pub raw_bytes: Vec<u8>,
}

impl SurfaceOutputPacket {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { raw_bytes: bytes }
    }
}

// ============================================================================
// Novation Launchpad Pro Profile
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LaunchpadProMode {
    #[default]
    Session,
    Note,
    Device,
    User,
    Programmer,
}

/// Profile for Novation Launchpad Pro controller.
#[derive(Debug, Clone)]
pub struct LaunchpadProProfile {
    pub mode: LaunchpadProMode,
    pub pad_rgb: [[(u8, u8, u8); 8]; 8],
    pub scene_leds: [u8; 8],
}

impl Default for LaunchpadProProfile {
    fn default() -> Self {
        Self {
            mode: LaunchpadProMode::Session,
            pad_rgb: [[(0, 0, 0); 8]; 8],
            scene_leds: [0; 8],
        }
    }
}

impl LaunchpadProProfile {
    pub fn new() -> Self {
        Self::default()
    }

    /// Generate SysEx message to set Launchpad Pro operational mode.
    pub fn set_mode(&mut self, mode: LaunchpadProMode) -> SurfaceOutputPacket {
        self.mode = mode;
        let mode_byte = match mode {
            LaunchpadProMode::Session => 0x00,
            LaunchpadProMode::Note => 0x01,
            LaunchpadProMode::Device => 0x02,
            LaunchpadProMode::User => 0x03,
            LaunchpadProMode::Programmer => 0x0F,
        };
        SurfaceOutputPacket::new(vec![
            0xF0, 0x00, 0x20, 0x29, 0x02, 0x10, 0x21, mode_byte, 0xF7,
        ])
    }

    /// Set an individual pad's RGB color via Novation SysEx palette packet.
    pub fn set_pad_rgb(
        &mut self,
        row: usize,
        col: usize,
        r: u8,
        g: u8,
        b: u8,
    ) -> Option<SurfaceOutputPacket> {
        if row >= 8 || col >= 8 {
            return None;
        }
        self.pad_rgb[row][col] = (r, g, b);
        let pad_index = ((8 - row) * 10 + (col + 1)) as u8;
        // Novation 7-bit RGB SysEx: [F0 00 20 29 02 10 0B pad r g b F7]
        let r7 = (r >> 1) & 0x7F;
        let g7 = (g >> 1) & 0x7F;
        let b7 = (b >> 1) & 0x7F;
        Some(SurfaceOutputPacket::new(vec![
            0xF0, 0x00, 0x20, 0x29, 0x02, 0x10, 0x0B, pad_index, r7, g7, b7, 0xF7,
        ]))
    }

    /// Process incoming MIDI message from Launchpad Pro.
    pub fn handle_midi_in(&self, bytes: &[u8]) -> Option<SurfaceAction> {
        if bytes.is_empty() {
            return None;
        }
        let status = bytes[0] & 0xF0;
        match status {
            0x90 => {
                // Note On
                if bytes.len() >= 3 {
                    let note = bytes[1];
                    let vel = bytes[2];
                    let row = 8_u8.checked_sub(note / 10)?;
                    let col = (note % 10).checked_sub(1)?;
                    if row < 8 && col < 8 {
                        Some(SurfaceAction::PadTrigger {
                            row,
                            col,
                            velocity: vel,
                            pressed: vel > 0,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            0x80 => {
                // Note Off
                if bytes.len() >= 2 {
                    let note = bytes[1];
                    let row = 8_u8.checked_sub(note / 10)?;
                    let col = (note % 10).checked_sub(1)?;
                    if row < 8 && col < 8 {
                        Some(SurfaceAction::PadTrigger {
                            row,
                            col,
                            velocity: 0,
                            pressed: false,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            0xB0 => {
                // CC: Top row or side scene buttons
                if bytes.len() >= 3 {
                    let cc = bytes[1];
                    let val = bytes[2];
                    if (89..=98).contains(&cc) && val > 0 {
                        // Scene launch right-hand column buttons (89..98)
                        let scene_idx = (98 - cc) as usize;
                        Some(SurfaceAction::SceneLaunch {
                            scene_index: scene_idx,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

// ============================================================================
// Ableton Push 2 / Push 3 Profile
// ============================================================================

/// Profile for Ableton Push 2 and Push 3 control surfaces.
#[derive(Debug, Clone)]
pub struct AbletonPushProfile {
    pub is_push3: bool,
    pub pad_rgb: [[(u8, u8, u8); 8]; 8],
    pub encoders: [f32; 8],
    pub master_encoder: f32,
    pub touch_strip_pos: f32,
    pub display_track_names: [String; 8],
    pub display_track_volumes: [f32; 8],
    pub display_track_pans: [f32; 8],
}

impl Default for AbletonPushProfile {
    fn default() -> Self {
        Self {
            is_push3: true,
            pad_rgb: [[(0, 0, 0); 8]; 8],
            encoders: [0.5; 8],
            master_encoder: 0.8,
            touch_strip_pos: 0.0,
            display_track_names: std::array::from_fn(|i| format!("Track {}", i + 1)),
            display_track_volumes: [0.8; 8],
            display_track_pans: [0.0; 8],
        }
    }
}

impl AbletonPushProfile {
    pub fn new(is_push3: bool) -> Self {
        Self {
            is_push3,
            ..Default::default()
        }
    }

    /// Set RGB LED on a 8x8 Push matrix pad.
    pub fn set_pad_rgb(
        &mut self,
        row: usize,
        col: usize,
        r: u8,
        g: u8,
        b: u8,
    ) -> Option<SurfaceOutputPacket> {
        if row >= 8 || col >= 8 {
            return None;
        }
        self.pad_rgb[row][col] = (r, g, b);
        let note = (36 + (7 - row) * 8 + col) as u8;
        // Ableton Push 2/3 RGB SysEx: [F0 00 21 1D 01 01 28 pad r g b F7]
        let r7 = (r >> 1) & 0x7F;
        let g7 = (g >> 1) & 0x7F;
        let b7 = (b >> 1) & 0x7F;
        Some(SurfaceOutputPacket::new(vec![
            0xF0, 0x00, 0x21, 0x1D, 0x01, 0x01, 0x28, note, r7, g7, b7, 0xF7,
        ]))
    }

    /// Update channel strip state on Push display.
    pub fn update_channel_strip(&mut self, track: usize, name: &str, volume: f32, pan: f32) {
        if track < 8 {
            self.display_track_names[track] = name.to_string();
            self.display_track_volumes[track] = volume.clamp(0.0, 1.0);
            self.display_track_pans[track] = pan.clamp(-1.0, 1.0);
        }
    }

    /// Process incoming MIDI from Push 2 / 3.
    pub fn handle_midi_in(&mut self, bytes: &[u8]) -> Option<SurfaceAction> {
        if bytes.is_empty() {
            return None;
        }
        let status = bytes[0] & 0xF0;
        match status {
            0x90 => {
                // Pad Note On
                if bytes.len() >= 3 {
                    let note = bytes[1];
                    let vel = bytes[2];
                    if (36..=99).contains(&note) {
                        let idx = note - 36;
                        let row = 7 - (idx / 8);
                        let col = idx % 8;
                        Some(SurfaceAction::PadTrigger {
                            row,
                            col,
                            velocity: vel,
                            pressed: vel > 0,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            0x80 => {
                // Pad Note Off
                if bytes.len() >= 2 {
                    let note = bytes[1];
                    if (36..=99).contains(&note) {
                        let idx = note - 36;
                        let row = 7 - (idx / 8);
                        let col = idx % 8;
                        Some(SurfaceAction::PadTrigger {
                            row,
                            col,
                            velocity: 0,
                            pressed: false,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            0xB0 => {
                // CC: Continuous rotary encoders (CC 71..78) or transport (CC 85 Play, CC 86 Stop, CC 87 Rec)
                if bytes.len() >= 3 {
                    let cc = bytes[1];
                    let val = bytes[2];
                    match cc {
                        71..=78 => {
                            let enc_idx = (cc - 71) as usize;
                            // Relative two's complement delta: 1..63 positive, 127..65 negative
                            let delta = if val < 64 {
                                val as f32 * 0.01
                            } else {
                                -((128 - val) as f32) * 0.01
                            };
                            self.encoders[enc_idx] =
                                (self.encoders[enc_idx] + delta).clamp(0.0, 1.0);
                            Some(SurfaceAction::EncoderTweak {
                                encoder_index: enc_idx,
                                delta,
                            })
                        }
                        85 if val > 0 => Some(SurfaceAction::TransportPlay),
                        86 if val > 0 => Some(SurfaceAction::TransportStop),
                        87 if val > 0 => Some(SurfaceAction::TransportRecord),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            0xE0 => {
                // Pitch Bend (Touch strip)
                if bytes.len() >= 3 {
                    let lsb = bytes[1] as u16;
                    let msb = bytes[2] as u16;
                    let bend = (msb << 7) | lsb;
                    self.touch_strip_pos = (bend as f32 / 16383.0) * 2.0 - 1.0;
                    None
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

// ============================================================================
// PreSonus FaderPort Profile
// ============================================================================

/// Profile for PreSonus FaderPort motorized fader control surface.
#[derive(Debug, Clone)]
pub struct PresonusFaderPortProfile {
    pub selected_channel: usize,
    pub fader_position: f32, // 0.0 to 1.0
    pub is_fader_touched: bool,
    pub pan_position: f32, // -1.0 to 1.0
    pub is_playing: bool,
    pub is_recording: bool,
    pub is_looping: bool,
    pub is_muted: bool,
    pub is_soloed: bool,
    pub is_armed: bool,
}

impl Default for PresonusFaderPortProfile {
    fn default() -> Self {
        Self {
            selected_channel: 0,
            fader_position: 0.75,
            is_fader_touched: false,
            pan_position: 0.0,
            is_playing: false,
            is_recording: false,
            is_looping: false,
            is_muted: false,
            is_soloed: false,
            is_armed: false,
        }
    }
}

impl PresonusFaderPortProfile {
    pub fn new() -> Self {
        Self::default()
    }

    /// Generate Pitch Bend message to physically move the motorized fader to target position.
    pub fn set_motorized_fader(&mut self, pos: f32) -> SurfaceOutputPacket {
        let clamped = pos.clamp(0.0, 1.0);
        self.fader_position = clamped;
        let int_val = (clamped * 16383.0) as u16;
        let lsb = (int_val & 0x7F) as u8;
        let msb = ((int_val >> 7) & 0x7F) as u8;
        SurfaceOutputPacket::new(vec![0xE0, lsb, msb])
    }

    /// Generate Note On LED feedback for transport buttons.
    pub fn set_button_led(&self, button_note: u8, active: bool) -> SurfaceOutputPacket {
        let vel = if active { 127 } else { 0 };
        SurfaceOutputPacket::new(vec![0x90, button_note, vel])
    }

    /// Process incoming MIDI from PreSonus FaderPort.
    pub fn handle_midi_in(&mut self, bytes: &[u8]) -> Option<SurfaceAction> {
        if bytes.is_empty() {
            return None;
        }
        let status = bytes[0] & 0xF0;
        match status {
            0xE0 => {
                // Pitch bend represents 10-bit motorized fader motion
                if bytes.len() >= 3 {
                    let lsb = bytes[1] as u16;
                    let msb = bytes[2] as u16;
                    let val = (msb << 7) | lsb;
                    let norm = (val as f32 / 16383.0).clamp(0.0, 1.0);
                    self.fader_position = norm;
                    Some(SurfaceAction::FaderMove {
                        channel: self.selected_channel,
                        position: norm,
                    })
                } else {
                    None
                }
            }
            0x90 => {
                // Button presses (Note On)
                if bytes.len() >= 3 {
                    let note = bytes[1];
                    let vel = bytes[2];
                    let pressed = vel > 0;
                    match note {
                        0x00 => {
                            // Arm button
                            if pressed {
                                self.is_armed = !self.is_armed;
                                Some(SurfaceAction::SetTrackArm {
                                    track_index: self.selected_channel,
                                    armed: self.is_armed,
                                })
                            } else {
                                None
                            }
                        }
                        0x08 => {
                            // Solo button
                            if pressed {
                                self.is_soloed = !self.is_soloed;
                                Some(SurfaceAction::SetTrackSolo {
                                    track_index: self.selected_channel,
                                    soloed: self.is_soloed,
                                })
                            } else {
                                None
                            }
                        }
                        0x10 => {
                            // Mute button
                            if pressed {
                                self.is_muted = !self.is_muted;
                                Some(SurfaceAction::SetTrackMute {
                                    track_index: self.selected_channel,
                                    muted: self.is_muted,
                                })
                            } else {
                                None
                            }
                        }
                        0x5B if pressed => Some(SurfaceAction::TransportRewind),
                        0x5C if pressed => Some(SurfaceAction::TransportFastForward),
                        0x5D if pressed => {
                            self.is_playing = false;
                            Some(SurfaceAction::TransportStop)
                        }
                        0x5E if pressed => {
                            self.is_playing = true;
                            Some(SurfaceAction::TransportPlay)
                        }
                        0x5F if pressed => {
                            self.is_recording = !self.is_recording;
                            Some(SurfaceAction::TransportRecord)
                        }
                        0x56 if pressed => {
                            self.is_looping = !self.is_looping;
                            Some(SurfaceAction::TransportToggleLoop)
                        }
                        0x68 => {
                            // Fader touch capacitive sensor
                            self.is_fader_touched = pressed;
                            Some(SurfaceAction::FaderTouch {
                                channel: self.selected_channel,
                                touching: pressed,
                            })
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }
            0xB0 => {
                // CC: Pan 360-degree endless encoder (CC 0x38 = 56)
                if bytes.len() >= 3 {
                    let cc = bytes[1];
                    let val = bytes[2];
                    if cc == 56 {
                        // Relative 2's complement: 1..63 right, 65..127 left
                        let delta = if val < 64 {
                            val as f32 * 0.02
                        } else {
                            -((128 - val) as f32) * 0.02
                        };
                        self.pan_position = (self.pan_position + delta).clamp(-1.0, 1.0);
                        Some(SurfaceAction::SetTrackPan {
                            track_index: self.selected_channel,
                            pan: self.pan_position,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

// ============================================================================
// Multi-Surface Router Manager
// ============================================================================

/// Hardware control surface kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlSurfaceKind {
    LaunchpadPro,
    AbletonPush2,
    AbletonPush3,
    PresonusFaderPort,
}

/// Central router managing connected hardware control surfaces.
#[derive(Debug, Default)]
pub struct ControlSurfaceManager {
    pub launchpad: Option<LaunchpadProProfile>,
    pub push: Option<AbletonPushProfile>,
    pub faderport: Option<PresonusFaderPortProfile>,
}

impl ControlSurfaceManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Connect a surface kind.
    pub fn connect_surface(&mut self, kind: ControlSurfaceKind) {
        match kind {
            ControlSurfaceKind::LaunchpadPro => {
                self.launchpad = Some(LaunchpadProProfile::new());
            }
            ControlSurfaceKind::AbletonPush2 => {
                self.push = Some(AbletonPushProfile::new(false));
            }
            ControlSurfaceKind::AbletonPush3 => {
                self.push = Some(AbletonPushProfile::new(true));
            }
            ControlSurfaceKind::PresonusFaderPort => {
                self.faderport = Some(PresonusFaderPortProfile::new());
            }
        }
    }

    /// Route inbound raw MIDI bytes from a specified surface kind.
    pub fn handle_midi(&mut self, kind: ControlSurfaceKind, bytes: &[u8]) -> Option<SurfaceAction> {
        match kind {
            ControlSurfaceKind::LaunchpadPro => self.launchpad.as_mut()?.handle_midi_in(bytes),
            ControlSurfaceKind::AbletonPush2 | ControlSurfaceKind::AbletonPush3 => {
                self.push.as_mut()?.handle_midi_in(bytes)
            }
            ControlSurfaceKind::PresonusFaderPort => self.faderport.as_mut()?.handle_midi_in(bytes),
        }
    }

    /// Broadcast track volume update across all connected surfaces.
    pub fn sync_track_volume(&mut self, track: usize, volume: f32) -> Vec<SurfaceOutputPacket> {
        let mut packets = Vec::new();
        if let Some(ref mut fp) = self.faderport {
            if fp.selected_channel == track {
                packets.push(fp.set_motorized_fader(volume));
            }
        }
        if let Some(ref mut push) = self.push {
            if track < 8 {
                push.display_track_volumes[track] = volume;
            }
        }
        packets
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_launchpad_pro_mode_and_pad_rgb() {
        let mut lp = LaunchpadProProfile::new();
        let mode_pkt = lp.set_mode(LaunchpadProMode::Programmer);
        assert_eq!(mode_pkt.raw_bytes[7], 0x0F);

        let rgb_pkt = lp.set_pad_rgb(0, 0, 255, 128, 64).unwrap();
        assert_eq!(rgb_pkt.raw_bytes[0], 0xF0);
        assert_eq!(rgb_pkt.raw_bytes.last(), Some(&0xF7));
        assert_eq!(lp.pad_rgb[0][0], (255, 128, 64));
    }

    #[test]
    fn test_launchpad_pro_midi_input() {
        let lp = LaunchpadProProfile::new();
        // Note On: row 0, col 0 (pad 81)
        let action = lp.handle_midi_in(&[0x90, 81, 100]);
        assert_eq!(
            action,
            Some(SurfaceAction::PadTrigger {
                row: 0,
                col: 0,
                velocity: 100,
                pressed: true,
            })
        );

        // Scene launch CC 98
        let scene_act = lp.handle_midi_in(&[0xB0, 98, 127]);
        assert_eq!(
            scene_act,
            Some(SurfaceAction::SceneLaunch { scene_index: 0 })
        );
    }

    #[test]
    fn test_ableton_push_encoders_and_pads() {
        let mut push = AbletonPushProfile::new(true);
        let action = push.handle_midi_in(&[0x90, 36, 120]); // Bottom left pad
        assert_eq!(
            action,
            Some(SurfaceAction::PadTrigger {
                row: 7,
                col: 0,
                velocity: 120,
                pressed: true,
            })
        );

        // Encoder 0 turn clockwise (val 1)
        let enc_action = push.handle_midi_in(&[0xB0, 71, 1]);
        assert!(
            matches!(enc_action, Some(SurfaceAction::EncoderTweak { encoder_index: 0, delta }) if delta > 0.0)
        );

        // Transport Play (CC 85)
        let play_act = push.handle_midi_in(&[0xB0, 85, 127]);
        assert_eq!(play_act, Some(SurfaceAction::TransportPlay));
    }

    #[test]
    fn test_presonus_faderport_motorized_fader_and_transport() {
        let mut fp = PresonusFaderPortProfile::new();
        let pkt = fp.set_motorized_fader(0.5);
        assert_eq!(pkt.raw_bytes[0], 0xE0); // Pitch Bend

        // Pitch bend input moves fader
        let act = fp.handle_midi_in(&[0xE0, 0x00, 0x40]); // approx 0.5
        assert!(
            matches!(act, Some(SurfaceAction::FaderMove { position, .. }) if (position - 0.5).abs() < 0.01)
        );

        // Transport Play button (0x90, 0x5E, 127)
        let play = fp.handle_midi_in(&[0x90, 0x5E, 127]);
        assert_eq!(play, Some(SurfaceAction::TransportPlay));
        assert!(fp.is_playing);

        // Mute button toggle (0x90, 0x10, 127)
        let mute = fp.handle_midi_in(&[0x90, 0x10, 127]);
        assert_eq!(
            mute,
            Some(SurfaceAction::SetTrackMute {
                track_index: 0,
                muted: true
            })
        );
        assert!(fp.is_muted);
    }

    #[test]
    fn test_control_surface_manager() {
        let mut manager = ControlSurfaceManager::new();
        manager.connect_surface(ControlSurfaceKind::PresonusFaderPort);
        manager.connect_surface(ControlSurfaceKind::AbletonPush3);

        let pkts = manager.sync_track_volume(0, 0.9);
        assert_eq!(pkts.len(), 1);
        assert_eq!(pkts[0].raw_bytes[0], 0xE0);
    }
}
