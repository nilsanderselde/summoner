// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Hardware MIDI device manager and realtime input engine powered by `midir`.

use crossbeam_channel::{Receiver, Sender};
use midir::{Ignore, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use summoner_core::midi::MidiEvent;
use summoner_core::midi_clock::{MidiClockGenerator, MidiClockReceiver};

/// Metadata information for a hardware MIDI port.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MidiPortInfo {
    pub index: usize,
    pub name: String,
}

/// Realtime timestamped MIDI message received from hardware port.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimestampedMidiEvent {
    pub timestamp_us: u64,
    pub event: MidiEvent,
}

/// Hardware MIDI port enumerator and manager.
pub struct MidiDeviceManager;

impl MidiDeviceManager {
    /// Enumerate all available hardware MIDI input ports.
    pub fn list_input_ports() -> Vec<MidiPortInfo> {
        let mut ports = Vec::new();
        if let Ok(midi_in) = MidiInput::new("summoner_midi_in_probe") {
            for (idx, port) in midi_in.ports().iter().enumerate() {
                if let Ok(name) = midi_in.port_name(port) {
                    ports.push(MidiPortInfo { index: idx, name });
                }
            }
        }
        ports
    }

    /// Enumerate all available hardware MIDI output ports.
    pub fn list_output_ports() -> Vec<MidiPortInfo> {
        let mut ports = Vec::new();
        if let Ok(midi_out) = MidiOutput::new("summoner_midi_out_probe") {
            for (idx, port) in midi_out.ports().iter().enumerate() {
                if let Ok(name) = midi_out.port_name(port) {
                    ports.push(MidiPortInfo { index: idx, name });
                }
            }
        }
        ports
    }
}

/// Active hardware MIDI input stream receiver.
pub struct MidiInputReceiver {
    pub is_connected: Arc<AtomicBool>,
    pub port_name: String,
    pub event_rx: Receiver<TimestampedMidiEvent>,
    _conn: Mutex<Option<MidiInputConnection<()>>>,
}

impl MidiInputReceiver {
    /// Connect to a MIDI input port by name or index.
    pub fn open_port(
        target_name_or_index: Option<&str>,
    ) -> Result<(Self, Sender<TimestampedMidiEvent>), String> {
        let mut midi_in = MidiInput::new("summoner_midi_receiver")
            .map_err(|e| format!("Failed to initialize MIDI input: {}", e))?;
        midi_in.ignore(Ignore::None);

        let ports = midi_in.ports();
        if ports.is_empty() {
            return Err("No MIDI input ports available".to_string());
        }

        let selected_port = if let Some(target) = target_name_or_index {
            ports
                .iter()
                .find(|p| {
                    midi_in
                        .port_name(p)
                        .map(|n| n.contains(target))
                        .unwrap_or(false)
                })
                .or_else(|| {
                    if let Ok(idx) = target.parse::<usize>() {
                        ports.get(idx)
                    } else {
                        None
                    }
                })
                .unwrap_or(&ports[0])
        } else {
            &ports[0]
        };

        let port_name = midi_in
            .port_name(selected_port)
            .unwrap_or_else(|_| "Unknown MIDI Device".to_string());

        let (tx, rx) = crossbeam_channel::bounded::<TimestampedMidiEvent>(2048);
        let tx_cb = tx.clone();
        let is_connected = Arc::new(AtomicBool::new(true));
        let is_conn_cb = Arc::clone(&is_connected);

        let conn = midi_in
            .connect(
                selected_port,
                "summoner_midi_in",
                move |timestamp_us, message, _| {
                    if let Some(event) = MidiEvent::from_bytes(message) {
                        let _ = tx_cb.try_send(TimestampedMidiEvent {
                            timestamp_us,
                            event,
                        });
                    }
                },
                (),
            )
            .map_err(|e| {
                is_conn_cb.store(false, Ordering::Relaxed);
                format!("Failed to connect to MIDI port: {}", e)
            })?;

        Ok((
            Self {
                is_connected,
                port_name,
                event_rx: rx,
                _conn: Mutex::new(Some(conn)),
            },
            tx,
        ))
    }

    /// Check if hardware MIDI connection is active.
    pub fn is_active(&self) -> bool {
        self.is_connected.load(Ordering::Relaxed)
    }

    /// Try receiving the next queued MIDI event.
    pub fn try_recv(&self) -> Option<TimestampedMidiEvent> {
        self.event_rx.try_recv().ok()
    }
}

/// Active hardware MIDI output stream sender (Task B2).
pub struct MidiOutputSender {
    pub is_connected: Arc<AtomicBool>,
    pub port_name: String,
    conn: Mutex<Option<MidiOutputConnection>>,
}

impl MidiOutputSender {
    /// Connect to a MIDI output port by name or index.
    pub fn open_port(target_name_or_index: Option<&str>) -> Result<Self, String> {
        let midi_out = MidiOutput::new("summoner_midi_sender")
            .map_err(|e| format!("Failed to initialize MIDI output: {}", e))?;
        let ports = midi_out.ports();
        if ports.is_empty() {
            return Err("No MIDI output ports available".to_string());
        }
        let selected_port = if let Some(target) = target_name_or_index {
            ports
                .iter()
                .find(|p| {
                    midi_out
                        .port_name(p)
                        .map(|n| n.contains(target))
                        .unwrap_or(false)
                })
                .or_else(|| {
                    if let Ok(idx) = target.parse::<usize>() {
                        ports.get(idx)
                    } else {
                        None
                    }
                })
                .unwrap_or(&ports[0])
        } else {
            &ports[0]
        };

        let port_name = midi_out
            .port_name(selected_port)
            .unwrap_or_else(|_| "Unknown MIDI Device".to_string());

        let conn = midi_out
            .connect(selected_port, "summoner_midi_out")
            .map_err(|e| format!("Failed to connect to MIDI output port: {}", e))?;

        Ok(Self {
            is_connected: Arc::new(AtomicBool::new(true)),
            port_name,
            conn: Mutex::new(Some(conn)),
        })
    }

    /// Send raw MIDI bytes directly to the hardware device.
    pub fn send_raw(&self, bytes: &[u8]) -> Result<(), String> {
        if let Ok(mut lock) = self.conn.lock() {
            if let Some(conn) = lock.as_mut() {
                conn.send(bytes)
                    .map_err(|e| format!("Failed to send MIDI message: {}", e))?;
                return Ok(());
            }
        }
        Err("MIDI output connection not open".to_string())
    }

    /// Send a typed MIDI event.
    pub fn send_event(&self, event: &MidiEvent) -> Result<(), String> {
        let bytes = event.to_bytes();
        self.send_raw(&bytes)
    }

    /// Send a 0xF8 Timing Clock tick.
    pub fn send_clock_tick(&self) -> Result<(), String> {
        self.send_raw(&[0xF8])
    }

    /// Send a 0xFA Transport Start message.
    pub fn send_start(&self) -> Result<(), String> {
        self.send_raw(&[0xFA])
    }

    /// Send a 0xFB Transport Continue message.
    pub fn send_continue(&self) -> Result<(), String> {
        self.send_raw(&[0xFB])
    }

    /// Send a 0xFC Transport Stop message.
    pub fn send_stop(&self) -> Result<(), String> {
        self.send_raw(&[0xFC])
    }

    /// Send a 0xF2 Song Position Pointer message (1 MIDI beat = 6 clock ticks = 1 sixteenth note).
    pub fn send_song_position_pointer(&self, midi_beats: u16) -> Result<(), String> {
        let lsb = (midi_beats & 0x7F) as u8;
        let msb = ((midi_beats >> 7) & 0x7F) as u8;
        self.send_raw(&[0xF2, lsb, msb])
    }

    /// Check if output port is active.
    pub fn is_active(&self) -> bool {
        self.is_connected.load(Ordering::Relaxed)
    }
}

/// Incoming MIDI slave sync state (Task B2).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MidiSyncState {
    pub is_running: bool,
    pub current_bpm: Option<f64>,
    pub current_beat: f64,
    pub tick_count: u64,
}

/// Receives external MIDI clock, transport commands (Start/Continue/Stop), and SPP to sync timeline (Task B2).
pub struct MidiSyncReceiver {
    clock_receiver: MidiClockReceiver,
    is_running: bool,
    current_bpm: Option<f64>,
    current_beat: f64,
}

impl MidiSyncReceiver {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            clock_receiver: MidiClockReceiver::new(sample_rate),
            is_running: false,
            current_bpm: None,
            current_beat: 0.0,
        }
    }

    pub fn process_byte(&mut self, byte: u8, now: std::time::Instant) -> MidiSyncState {
        match byte {
            0xF8 => {
                if let Some(bpm) = self.clock_receiver.process_byte(0xF8, now) {
                    self.current_bpm = Some(bpm);
                }
                if self.is_running {
                    // 24 PPQN => 1 beat = 24 ticks
                    self.current_beat += 1.0 / 24.0;
                }
            }
            0xFA => {
                self.is_running = true;
                self.current_beat = 0.0;
            }
            0xFB => {
                self.is_running = true;
            }
            0xFC => {
                self.is_running = false;
            }
            _ => {}
        }
        self.state()
    }

    pub fn process_event(&mut self, event: &MidiEvent, now: std::time::Instant) -> MidiSyncState {
        match *event {
            MidiEvent::SystemRealtime(byte) => self.process_byte(byte, now),
            MidiEvent::SongPositionPointer(spp) => {
                // 1 MIDI beat = 6 MIDI clock ticks = 0.25 quarter note
                self.current_beat = spp as f64 * 0.25;
                self.state()
            }
            _ => self.state(),
        }
    }

    pub fn state(&self) -> MidiSyncState {
        MidiSyncState {
            is_running: self.is_running,
            current_bpm: self.current_bpm,
            current_beat: self.current_beat,
            tick_count: self.clock_receiver.tick_count(),
        }
    }
}

/// Real-time MIDI clock and transport emitter for external hardware sync (Task B2).
#[derive(Debug, Clone)]
pub struct MidiClockOutRunner {
    generator: MidiClockGenerator,
    pub is_running: bool,
}

impl MidiClockOutRunner {
    pub fn new(sample_rate: u32, bpm: f64) -> Self {
        Self {
            generator: MidiClockGenerator::new(sample_rate, bpm),
            is_running: false,
        }
    }

    pub fn set_bpm(&mut self, bpm: f64) {
        self.generator.set_bpm(bpm);
    }

    pub fn start(&mut self) -> [u8; 1] {
        self.is_running = true;
        self.generator.reset();
        [0xFA]
    }

    pub fn stop(&mut self) -> [u8; 1] {
        self.is_running = false;
        [0xFC]
    }

    pub fn advance_frames(&mut self, frames: u64) -> u32 {
        if !self.is_running {
            return 0;
        }
        self.generator.advance(frames)
    }

    pub fn format_spp_from_beat(beat: f64) -> (u16, [u8; 3]) {
        let spp = (beat * 4.0).floor().max(0.0) as u16;
        let lsb = (spp & 0x7F) as u8;
        let msb = ((spp >> 7) & 0x7F) as u8;
        (spp, [0xF2, lsb, msb])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_clock_out_runner_and_spp() {
        let mut runner = MidiClockOutRunner::new(48000, 120.0);
        let start_msg = runner.start();
        assert_eq!(start_msg, [0xFA]);
        assert!(runner.is_running);

        // At 120 BPM, 1 beat = 0.5s = 24000 frames. 24 ticks per beat => 24 ticks emitted
        let ticks = runner.advance_frames(24000);
        assert_eq!(ticks, 24);

        let stop_msg = runner.stop();
        assert_eq!(stop_msg, [0xFC]);
        assert!(!runner.is_running);
        assert_eq!(runner.advance_frames(24000), 0);

        // SPP formatting for bar 4 beat 2 (e.g. beat 14.0) => 14 * 4 = 56 sixteenth notes
        let (spp, bytes) = MidiClockOutRunner::format_spp_from_beat(14.0);
        assert_eq!(spp, 56);
        assert_eq!(bytes[0], 0xF2);
        assert_eq!(bytes[1], 56);
        assert_eq!(bytes[2], 0);
    }

    #[test]
    fn test_midi_sync_receiver_transport_and_spp() {
        let mut receiver = MidiSyncReceiver::new(48000);
        let now = std::time::Instant::now();

        // SPP jump to 32 sixteenth notes (beat 8.0)
        let state = receiver.process_event(&MidiEvent::SongPositionPointer(32), now);
        assert_eq!(state.current_beat, 8.0);
        assert!(!state.is_running);

        // Transport Start
        let state = receiver.process_event(&MidiEvent::SystemRealtime(0xFA), now);
        assert!(state.is_running);
        assert_eq!(state.current_beat, 0.0);

        // 24 clock ticks advance 1 quarter note
        for _ in 0..24 {
            receiver.process_event(&MidiEvent::SystemRealtime(0xF8), now);
        }
        let state = receiver.state();
        assert!((state.current_beat - 1.0).abs() < 1e-4);

        // Transport Stop
        let state = receiver.process_event(&MidiEvent::SystemRealtime(0xFC), now);
        assert!(!state.is_running);
    }

    #[test]
    fn test_midi_device_manager_enumeration_safe() {
        let in_ports = MidiDeviceManager::list_input_ports();
        let out_ports = MidiDeviceManager::list_output_ports();
        // Should query the platform MIDI subsystem safely without panicking
        let _ = in_ports.len();
        let _ = out_ports.len();
    }

    #[test]
    fn test_timestamped_midi_event_queue() {
        let (tx, rx) = crossbeam_channel::bounded::<TimestampedMidiEvent>(16);
        let evt = TimestampedMidiEvent {
            timestamp_us: 1000,
            event: MidiEvent::NoteOn(0, 60, 100),
        };
        tx.try_send(evt).unwrap();

        let received = rx.try_recv().unwrap();
        assert_eq!(received.timestamp_us, 1000);
        assert_eq!(received.event, MidiEvent::NoteOn(0, 60, 100));
    }
}
