// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Hardware MIDI device manager and realtime input engine powered by `midir`.

use crossbeam_channel::{Receiver, Sender};
use midir::{Ignore, MidiInput, MidiInputConnection, MidiOutput};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use summoner_core::midi::MidiEvent;

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

#[cfg(test)]
mod tests {
    use super::*;

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

