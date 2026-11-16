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

//! Virtual MIDI port manager supporting macOS CoreMIDI, Linux ALSA sequencer,
//! and Windows loopMIDI/teVirtualMIDI software bridges (Task B6).

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Direction of virtual MIDI port data flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VirtualMidiDirection {
    Input,
    Output,
    Bidirectional,
}

/// Metadata description of a virtual MIDI port.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VirtualMidiPortInfo {
    pub id: usize,
    pub name: String,
    pub stable_port_id: String,
    pub direction: VirtualMidiDirection,
    pub is_connected: bool,
}

/// Representation of an active virtual MIDI port with queued message buffers.
#[derive(Debug, Clone)]
pub struct VirtualMidiPort {
    pub id: usize,
    pub name: String,
    pub stable_port_id: String,
    pub direction: VirtualMidiDirection,
    pub is_connected: bool,
    incoming_queue: VecDeque<(u64, Vec<u8>)>,
    outgoing_queue: VecDeque<(u64, Vec<u8>)>,
}

fn stable_virtual_port_id(name: &str, direction: VirtualMidiDirection) -> String {
    let dir_tag = match direction {
        VirtualMidiDirection::Input => "input",
        VirtualMidiDirection::Output => "output",
        VirtualMidiDirection::Bidirectional => "bidirectional",
    };
    let slug: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    format!("summoner://virtual/{}/{}", dir_tag, slug)
}

impl VirtualMidiPort {
    pub fn new(id: usize, name: &str, direction: VirtualMidiDirection) -> Self {
        Self {
            id,
            name: name.to_string(),
            stable_port_id: stable_virtual_port_id(name, direction),
            direction,
            is_connected: true,
            incoming_queue: VecDeque::new(),
            outgoing_queue: VecDeque::new(),
        }
    }

    /// Queue a MIDI message to be sent out from Summoner to external applications.
    pub fn send(&mut self, timestamp_samples: u64, message: &[u8]) -> Result<(), String> {
        if !self.is_connected {
            return Err("Virtual MIDI port is disconnected".into());
        }
        if self.direction == VirtualMidiDirection::Input {
            return Err("Cannot send on an input-only virtual port".into());
        }
        self.outgoing_queue
            .push_back((timestamp_samples, message.to_vec()));
        Ok(())
    }

    /// Simulate or receive incoming packet from an external DAW / software instrument.
    pub fn receive(&mut self, timestamp_samples: u64, message: &[u8]) -> Result<(), String> {
        if !self.is_connected {
            return Err("Virtual MIDI port is disconnected".into());
        }
        if self.direction == VirtualMidiDirection::Output {
            return Err("Cannot receive on an output-only virtual port".into());
        }
        self.incoming_queue
            .push_back((timestamp_samples, message.to_vec()));
        Ok(())
    }

    /// Drain all queued incoming packets for real-time dispatch into the sequencer / graph.
    pub fn drain_incoming(&mut self) -> Vec<(u64, Vec<u8>)> {
        self.incoming_queue.drain(..).collect()
    }

    /// Drain all queued outgoing packets to push to the OS MIDI subsystem.
    pub fn drain_outgoing(&mut self) -> Vec<(u64, Vec<u8>)> {
        self.outgoing_queue.drain(..).collect()
    }

    /// Frame and queue an arbitrary System Exclusive payload (`0xF0 ... 0xF7`) for output (`AIO-8`).
    /// Accepts either raw payload bytes or already-framed `0xF0..0xF7` messages.
    pub fn send_sysex(&mut self, timestamp_samples: u64, payload: &[u8]) -> Result<(), String> {
        if payload.is_empty() {
            return Err("SysEx payload cannot be empty".into());
        }
        let mut framed = Vec::with_capacity(payload.len() + 2);
        if payload.first() != Some(&0xF0) {
            framed.push(0xF0);
        }
        for &b in payload {
            if b == 0xF0 && !framed.is_empty() {
                continue;
            }
            if b == 0xF7 {
                break;
            }
            framed.push(b & 0x7F);
        }
        framed.push(0xF7);
        self.send(timestamp_samples, &framed)
    }

    /// Extract complete incoming `0xF0..0xF7` SysEx packets without discarding non-SysEx messages.
    pub fn drain_sysex(&mut self) -> Vec<(u64, Vec<u8>)> {
        let mut sysex = Vec::new();
        let mut remaining = VecDeque::with_capacity(self.incoming_queue.len());
        while let Some((ts, msg)) = self.incoming_queue.pop_front() {
            if msg.first() == Some(&0xF0) && msg.last() == Some(&0xF7) {
                sysex.push((ts, msg));
            } else {
                remaining.push_back((ts, msg));
            }
        }
        self.incoming_queue = remaining;
        sysex
    }
}

/// System-wide Virtual MIDI Manager.
#[derive(Debug, Default)]
pub struct VirtualMidiManager {
    next_id: usize,
    ports: Vec<VirtualMidiPort>,
}

impl VirtualMidiManager {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            ports: Vec::new(),
        }
    }

    /// Create and activate a new virtual MIDI port.
    pub fn create_port(
        &mut self,
        name: &str,
        direction: VirtualMidiDirection,
    ) -> Result<usize, String> {
        if name.trim().is_empty() {
            return Err("Port name cannot be empty".into());
        }
        let id = self.next_id;
        self.next_id += 1;
        let port = VirtualMidiPort::new(id, name, direction);
        self.ports.push(port);
        Ok(id)
    }

    /// Disconnect and destroy a virtual MIDI port.
    pub fn destroy_port(&mut self, port_id: usize) -> Result<(), String> {
        if let Some(pos) = self.ports.iter().position(|p| p.id == port_id) {
            self.ports.remove(pos);
            Ok(())
        } else {
            Err(format!("Virtual MIDI port with ID {} not found", port_id))
        }
    }

    /// List all registered virtual MIDI ports.
    pub fn list_ports(&self) -> Vec<VirtualMidiPortInfo> {
        self.ports
            .iter()
            .map(|p| VirtualMidiPortInfo {
                id: p.id,
                name: p.name.clone(),
                stable_port_id: p.stable_port_id.clone(),
                direction: p.direction,
                is_connected: p.is_connected,
            })
            .collect()
    }

    /// Query mutable reference to a virtual port by ID.
    pub fn get_port_mut(&mut self, port_id: usize) -> Option<&mut VirtualMidiPort> {
        self.ports.iter_mut().find(|p| p.id == port_id)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_midi_port_lifecycle() {
        let mut mgr = VirtualMidiManager::new();
        assert!(mgr.list_ports().is_empty());

        let port_id = mgr
            .create_port("Summoner Out 1", VirtualMidiDirection::Output)
            .unwrap();
        assert_eq!(mgr.list_ports().len(), 1);

        let port = mgr.get_port_mut(port_id).unwrap();
        assert_eq!(port.name, "Summoner Out 1");
        assert_eq!(port.direction, VirtualMidiDirection::Output);

        // Sending should succeed on output port
        let note_on = [0x90, 60, 100];
        assert!(port.send(128, &note_on).is_ok());
        let sent = port.drain_outgoing();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].0, 128);
        assert_eq!(sent[0].1, vec![0x90, 60, 100]);

        // Receiving on output-only port should fail
        assert!(port.receive(256, &note_on).is_err());

        // Destroy port
        assert!(mgr.destroy_port(port_id).is_ok());
        assert!(mgr.list_ports().is_empty());
        assert!(mgr.destroy_port(port_id).is_err());
    }

    #[test]
    fn test_virtual_midi_bidirectional() {
        let mut mgr = VirtualMidiManager::new();
        let port_id = mgr
            .create_port("Summoner Virtual Bus", VirtualMidiDirection::Bidirectional)
            .unwrap();

        let port = mgr.get_port_mut(port_id).unwrap();
        let cc74 = [0xB0, 74, 95];
        assert!(port.send(500, &cc74).is_ok());
        assert!(port.receive(600, &cc74).is_ok());

        let out = port.drain_outgoing();
        let inc = port.drain_incoming();
        assert_eq!(out.len(), 1);
        assert_eq!(inc.len(), 1);
        assert_eq!(out[0].0, 500);
        assert_eq!(inc[0].0, 600);
    }

    #[test]
    fn test_virtual_midi_sysex_and_stable_port_id() {
        let mut mgr = VirtualMidiManager::new();
        let pid = mgr
            .create_port("Summoner Sync Bus 1", VirtualMidiDirection::Bidirectional)
            .unwrap();
        let info = mgr.list_ports();
        assert_eq!(
            info[0].stable_port_id,
            "summoner://virtual/bidirectional/summoner_sync_bus_1"
        );

        let port = mgr.get_port_mut(pid).unwrap();
        port.send_sysex(1024, &[0x7E, 0x7F, 0x06, 0x01]).unwrap();
        let outgoing = port.drain_outgoing();
        assert_eq!(outgoing[0].1, vec![0xF0, 0x7E, 0x7F, 0x06, 0x01, 0xF7]);

        port.receive(2000, &[0x90, 60, 100]).unwrap();
        port.receive(2048, &[0xF0, 0x41, 0x10, 0xF7]).unwrap();
        let syx = port.drain_sysex();
        assert_eq!(syx, vec![(2048, vec![0xF0, 0x41, 0x10, 0xF7])]);
        let rem = port.drain_incoming();
        assert_eq!(rem, vec![(2000, vec![0x90, 60, 100])]);
    }
}
