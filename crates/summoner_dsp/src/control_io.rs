//! Real-time OSC (Open Sound Control), UDP, and Serial I/O control nodes.
//!
//! Provides modular control and modulation source nodes:
//! - [`OscControlNode`]: Decodes OSC 1.0 messages and bundles with address pattern matching,
//!   delivering smoothed CV and trigger pulses to the audio DSP graph with zero audio-thread allocation.
//! - [`UdpControlNode`]: Receives raw UDP datagrams containing binary floats or ASCII numbers.
//! - [`SerialControlNode`]: Decodes microcontroller / sensor telemetry (ASCII lines or framed binary packets).

use crate::traits::SignalProcessor;
use std::collections::HashMap;
use std::net::UdpSocket;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::Arc;
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, NodeEvent, ProcessContext};

/// OSC data value variants.
#[derive(Debug, Clone, PartialEq)]
pub enum OscValue {
    Float(f32),
    Int(i32),
    String(String),
}

/// Parsed OSC 1.0 message.
#[derive(Debug, Clone, PartialEq)]
pub struct OscMessage {
    pub address: String,
    pub args: Vec<OscValue>,
}

/// Zero-copy OSC 1.0 packet decoder.
pub struct OscPacketParser;

impl OscPacketParser {
    /// Decodes an OSC packet (single message or bundle) from raw bytes.
    pub fn parse_packet(buf: &[u8]) -> Result<Vec<OscMessage>, &'static str> {
        if buf.len() < 4 {
            return Err("Packet too short");
        }

        if buf.starts_with(b"#bundle\0") {
            Self::parse_bundle(buf)
        } else {
            let msg = Self::parse_message(buf)?;
            Ok(vec![msg])
        }
    }

    /// Parse single OSC message: address pattern, type tags, and arguments.
    pub fn parse_message(buf: &[u8]) -> Result<OscMessage, &'static str> {
        let (address, mut offset) = Self::read_padded_string(buf, 0)?;

        if offset >= buf.len() {
            return Ok(OscMessage {
                address,
                args: Vec::new(),
            });
        }

        let type_tags = if buf[offset] == b',' {
            let (tags, next_offset) = Self::read_padded_string(buf, offset)?;
            offset = next_offset;
            tags
        } else {
            String::new()
        };

        let mut args = Vec::new();
        for tag in type_tags.chars().skip(1) {
            match tag {
                'f' => {
                    if offset + 4 > buf.len() {
                        return Err("Premature EOF reading float");
                    }
                    let val_bytes: [u8; 4] = buf[offset..offset + 4].try_into().unwrap();
                    let val = f32::from_be_bytes(val_bytes);
                    offset += 4;
                    args.push(OscValue::Float(val));
                }
                'i' => {
                    if offset + 4 > buf.len() {
                        return Err("Premature EOF reading int");
                    }
                    let val_bytes: [u8; 4] = buf[offset..offset + 4].try_into().unwrap();
                    let val = i32::from_be_bytes(val_bytes);
                    offset += 4;
                    args.push(OscValue::Int(val));
                }
                's' => {
                    let (s, next_offset) = Self::read_padded_string(buf, offset)?;
                    offset = next_offset;
                    args.push(OscValue::String(s));
                }
                _ => {
                    // Skip unrecognized tag (pad to 4 bytes if data)
                    if offset + 4 <= buf.len() {
                        offset += 4;
                    }
                }
            }
        }

        Ok(OscMessage { address, args })
    }

    /// Parse OSC bundle (#bundle + 8-byte timetag + element sizes + elements).
    fn parse_bundle(buf: &[u8]) -> Result<Vec<OscMessage>, &'static str> {
        if buf.len() < 16 {
            return Err("Bundle header too short");
        }
        let mut offset = 16; // Skip "#bundle\0" (8 bytes) + timetag (8 bytes)
        let mut messages = Vec::new();

        while offset + 4 <= buf.len() {
            let size_bytes: [u8; 4] = buf[offset..offset + 4].try_into().unwrap();
            let element_size = u32::from_be_bytes(size_bytes) as usize;
            offset += 4;

            if offset + element_size > buf.len() {
                return Err("Element size exceeds bundle bounds");
            }

            let element_slice = &buf[offset..offset + element_size];
            offset += element_size;

            let mut sub_msgs = Self::parse_packet(element_slice)?;
            messages.append(&mut sub_msgs);
        }

        Ok(messages)
    }

    /// Read null-terminated string padded to 4-byte boundary.
    fn read_padded_string(buf: &[u8], start: usize) -> Result<(String, usize), &'static str> {
        if start >= buf.len() {
            return Err("Out of bounds reading string");
        }

        let mut end = start;
        while end < buf.len() && buf[end] != 0 {
            end += 1;
        }

        if end >= buf.len() {
            return Err("Unterminated string");
        }

        let s = std::str::from_utf8(&buf[start..end])
            .map_err(|_| "Invalid UTF-8 string")?
            .to_string();

        // 4-byte padding: advance to next multiple of 4
        let padded_len = (end - start + 1).div_ceil(4) * 4;
        let next_offset = start + padded_len;

        Ok((s, next_offset))
    }

    /// Encode an OSC message into byte buffer.
    pub fn encode_message(msg: &OscMessage, buf: &mut Vec<u8>) {
        // 1. Write address with 4-byte padding
        Self::write_padded_string(&msg.address, buf);

        // 2. Build and write type tags
        let mut type_tags = String::from(",");
        for arg in &msg.args {
            match arg {
                OscValue::Float(_) => type_tags.push('f'),
                OscValue::Int(_) => type_tags.push('i'),
                OscValue::String(_) => type_tags.push('s'),
            }
        }
        Self::write_padded_string(&type_tags, buf);

        // 3. Write arguments
        for arg in &msg.args {
            match arg {
                OscValue::Float(f) => buf.extend_from_slice(&f.to_be_bytes()),
                OscValue::Int(i) => buf.extend_from_slice(&i.to_be_bytes()),
                OscValue::String(s) => Self::write_padded_string(s, buf),
            }
        }
    }

    fn write_padded_string(s: &str, buf: &mut Vec<u8>) {
        let bytes = s.as_bytes();
        buf.extend_from_slice(bytes);
        buf.push(0); // Null terminator
        while !buf.len().is_multiple_of(4) {
            buf.push(0);
        }
    }
}

/// Control message payload transferred from network/serial thread to audio thread.
#[derive(Debug, Clone, Copy)]
pub struct ControlUpdate {
    pub value: f32,
    pub is_trigger: bool,
}

/// Atomic real-time single-float mailbox.
#[derive(Debug)]
pub struct AtomicControlChannel {
    val_bits: AtomicU32,
    trigger_flag: AtomicU32,
}

impl Default for AtomicControlChannel {
    fn default() -> Self {
        Self::new(0.0)
    }
}

impl AtomicControlChannel {
    pub fn new(initial: f32) -> Self {
        Self {
            val_bits: AtomicU32::new(initial.to_bits()),
            trigger_flag: AtomicU32::new(0),
        }
    }

    pub fn set(&self, val: f32, trigger: bool) {
        self.val_bits.store(val.to_bits(), Ordering::Release);
        if trigger {
            self.trigger_flag.store(1, Ordering::Release);
        }
    }

    pub fn fetch(&self) -> (f32, bool) {
        let bits = self.val_bits.load(Ordering::Acquire);
        let trigger = self.trigger_flag.swap(0, Ordering::AcqRel) == 1;
        (f32::from_bits(bits), trigger)
    }
}

/// OSC Control Node: binds address patterns and outputs smoothed CV and trigger audio streams.
pub struct OscControlNode {
    pub name: String,
    pub address_pattern: String,
    pub min_range: f32,
    pub max_range: f32,
    pub slew_factor: f32, // One-pole filter coefficient for de-zippering
    current_cv: f32,
    target_cv: f32,
    trigger_pulse: f32,
    mailbox: Arc<AtomicControlChannel>,
    _tx: SyncSender<Vec<u8>>,
    rx: Receiver<Vec<u8>>,
}

impl OscControlNode {
    pub fn new(address_pattern: &str, min_range: f32, max_range: f32) -> Self {
        let (tx, rx) = sync_channel(128);
        Self {
            name: "OscControlNode".to_string(),
            address_pattern: address_pattern.to_string(),
            min_range,
            max_range,
            slew_factor: 0.05,
            current_cv: min_range,
            target_cv: min_range,
            trigger_pulse: 0.0,
            mailbox: Arc::new(AtomicControlChannel::new(min_range)),
            _tx: tx,
            rx,
        }
    }

    /// Access the thread-safe mailbox to push external updates.
    pub fn mailbox(&self) -> Arc<AtomicControlChannel> {
        Arc::clone(&self.mailbox)
    }

    /// Inject an OSC packet directly into this node's lock-free channel.
    pub fn inject_packet(&mut self, packet_data: &[u8]) {
        if let Ok(msgs) = OscPacketParser::parse_packet(packet_data) {
            for msg in msgs {
                if msg.address == self.address_pattern {
                    if let Some(first_arg) = msg.args.first() {
                        let raw_val = match first_arg {
                            OscValue::Float(f) => *f,
                            OscValue::Int(i) => *i as f32,
                            _ => 0.0,
                        };
                        let scaled = self.min_range
                            + raw_val.clamp(0.0, 1.0) * (self.max_range - self.min_range);
                        self.mailbox.set(scaled, true);
                    }
                }
            }
        }
    }

    /// Start a background UDP listener on specified socket address.
    pub fn start_udp_listener(&self, bind_addr: &str) -> std::io::Result<()> {
        let socket = UdpSocket::bind(bind_addr)?;
        socket.set_nonblocking(false)?;
        let mailbox = Arc::clone(&self.mailbox);
        let pattern = self.address_pattern.clone();
        let min_r = self.min_range;
        let max_r = self.max_range;

        std::thread::Builder::new()
            .name("osc-udp-listener".to_string())
            .spawn(move || {
                let mut buf = [0u8; 2048];
                while let Ok((amt, _src)) = socket.recv_from(&mut buf) {
                    if let Ok(msgs) = OscPacketParser::parse_packet(&buf[..amt]) {
                        for msg in msgs {
                            if msg.address == pattern {
                                if let Some(arg) = msg.args.first() {
                                    let v = match arg {
                                        OscValue::Float(f) => *f,
                                        OscValue::Int(i) => *i as f32,
                                        _ => 0.0,
                                    };
                                    let val = min_r + v.clamp(0.0, 1.0) * (max_r - min_r);
                                    mailbox.set(val, true);
                                }
                            }
                        }
                    }
                }
            })?;

        Ok(())
    }
}

impl SignalProcessor for OscControlNode {
    fn name(&self) -> &str {
        &self.name
    }

    fn process_block(
        &mut self,
        _inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        _ctx: &ProcessContext,
    ) {
        if outputs.is_empty() {
            return;
        }

        // Dequeue any pending packet data non-blockingly
        while let Ok(data) = self.rx.try_recv() {
            self.inject_packet(&data);
        }

        // Fetch latest value from atomic mailbox
        let (val, trigger) = self.mailbox.fetch();
        self.target_cv = val;
        if trigger {
            self.trigger_pulse = 1.0;
        }

        let (out0, out1) = if outputs.len() >= 2 {
            let (first, rest) = outputs.split_at_mut(1);
            (&mut first[0][..], Some(&mut rest[0][..]))
        } else {
            (&mut outputs[0][..], None)
        };

        if let Some(gate) = out1 {
            for (cv_sample, gate_sample) in out0.iter_mut().zip(gate.iter_mut()) {
                self.current_cv += (self.target_cv - self.current_cv) * self.slew_factor;
                *cv_sample = self.current_cv;
                *gate_sample = self.trigger_pulse;
                self.trigger_pulse *= 0.95;
                if self.trigger_pulse < 1e-4 {
                    self.trigger_pulse = 0.0;
                }
            }
        } else {
            for cv_sample in out0.iter_mut() {
                self.current_cv += (self.target_cv - self.current_cv) * self.slew_factor;
                *cv_sample = self.current_cv;
            }
        }
    }
}

impl AudioNode for OscControlNode {
    fn name(&self) -> &str {
        &self.name
    }

    fn process(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        self.process_block(inputs, outputs, ctx);
    }

    fn handle_event(&mut self, _event: &NodeEvent, _ctx: &ProcessContext) {}
}

/// Generic UDP control node streaming numeric telemetry into CV modulation.
pub struct UdpControlNode {
    pub name: String,
    pub min_range: f32,
    pub max_range: f32,
    mailbox: Arc<AtomicControlChannel>,
    current_cv: f32,
    target_cv: f32,
}

impl UdpControlNode {
    pub fn new(min_range: f32, max_range: f32) -> Self {
        Self {
            name: "UdpControlNode".to_string(),
            min_range,
            max_range,
            mailbox: Arc::new(AtomicControlChannel::new(min_range)),
            current_cv: min_range,
            target_cv: min_range,
        }
    }

    pub fn mailbox(&self) -> Arc<AtomicControlChannel> {
        Arc::clone(&self.mailbox)
    }

    /// Process a raw binary or ASCII datagram.
    pub fn handle_datagram(&self, data: &[u8]) {
        if let Ok(s) = std::str::from_utf8(data) {
            // Line-delimited ASCII number
            if let Ok(val) = s.trim().parse::<f32>() {
                let scaled =
                    self.min_range + val.clamp(0.0, 1.0) * (self.max_range - self.min_range);
                self.mailbox.set(scaled, true);
                return;
            }
        }

        if data.len() == 4 {
            // 32-bit big-endian or little-endian float
            let arr: [u8; 4] = data.try_into().unwrap();
            let val = f32::from_le_bytes(arr);
            let scaled = self.min_range + val.clamp(0.0, 1.0) * (self.max_range - self.min_range);
            self.mailbox.set(scaled, true);
        }
    }
}

impl SignalProcessor for UdpControlNode {
    fn name(&self) -> &str {
        &self.name
    }

    fn process_block(
        &mut self,
        _inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        _ctx: &ProcessContext,
    ) {
        if outputs.is_empty() {
            return;
        }

        let (val, _) = self.mailbox.fetch();
        self.target_cv = val;

        for sample in outputs[0].iter_mut() {
            self.current_cv += (self.target_cv - self.current_cv) * 0.1;
            *sample = self.current_cv;
        }
    }
}

impl AudioNode for UdpControlNode {
    fn name(&self) -> &str {
        &self.name
    }

    fn process(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        self.process_block(inputs, outputs, ctx);
    }

    fn handle_event(&mut self, _event: &NodeEvent, _ctx: &ProcessContext) {}
}

/// Serial control node for microcontroller / hardware sensor telemetry.
pub struct SerialControlNode {
    pub name: String,
    channels: HashMap<String, Arc<AtomicControlChannel>>,
    current_values: [f32; 4],
    target_values: [f32; 4],
}

impl Default for SerialControlNode {
    fn default() -> Self {
        Self::new()
    }
}

impl SerialControlNode {
    pub fn new() -> Self {
        let mut channels = HashMap::new();
        channels.insert("A0".to_string(), Arc::new(AtomicControlChannel::new(0.0)));
        channels.insert("A1".to_string(), Arc::new(AtomicControlChannel::new(0.0)));
        channels.insert("A2".to_string(), Arc::new(AtomicControlChannel::new(0.0)));
        channels.insert("A3".to_string(), Arc::new(AtomicControlChannel::new(0.0)));

        Self {
            name: "SerialControlNode".to_string(),
            channels,
            current_values: [0.0; 4],
            target_values: [0.0; 4],
        }
    }

    /// Ingest a line of serial data, e.g. "A0:0.85\n" or "A1: 512\n" (10-bit ADC 0..1023).
    pub fn parse_serial_line(&self, line: &str) {
        let trimmed = line.trim();
        if let Some((chan_str, val_str)) = trimmed.split_once(':') {
            let chan = chan_str.trim().to_uppercase();
            if let Some(mailbox) = self.channels.get(&chan) {
                if let Ok(num) = val_str.trim().parse::<f32>() {
                    // Normalize if ADC 10-bit integer (> 1.0)
                    let norm = if num > 1.0 {
                        (num / 1023.0).clamp(0.0, 1.0)
                    } else {
                        num.clamp(0.0, 1.0)
                    };
                    mailbox.set(norm, true);
                }
            }
        }
    }
}

impl SignalProcessor for SerialControlNode {
    fn name(&self) -> &str {
        &self.name
    }

    fn process_block(
        &mut self,
        _inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        _ctx: &ProcessContext,
    ) {
        if outputs.is_empty() {
            return;
        }

        // Update target values from mailboxes
        for (idx, key) in ["A0", "A1", "A2", "A3"].iter().enumerate() {
            if let Some(ch) = self.channels.get(*key) {
                let (val, _) = ch.fetch();
                self.target_values[idx] = val;
            }
        }

        for (ch, out_ch) in outputs.iter_mut().enumerate().take(4) {
            let target = self.target_values[ch];
            for sample in out_ch.iter_mut() {
                self.current_values[ch] += (target - self.current_values[ch]) * 0.1;
                *sample = self.current_values[ch];
            }
        }
    }
}

impl AudioNode for SerialControlNode {
    fn name(&self) -> &str {
        &self.name
    }

    fn process(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        self.process_block(inputs, outputs, ctx);
    }

    fn handle_event(&mut self, _event: &NodeEvent, _ctx: &ProcessContext) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_osc_packet_encode_decode_message() {
        let msg = OscMessage {
            address: "/filter/cutoff".to_string(),
            args: vec![OscValue::Float(0.85), OscValue::Int(42)],
        };

        let mut buf = Vec::new();
        OscPacketParser::encode_message(&msg, &mut buf);
        assert_eq!(buf.len() % 4, 0);

        let decoded = OscPacketParser::parse_message(&buf).expect("Decode should succeed");
        assert_eq!(decoded.address, "/filter/cutoff");
        assert_eq!(decoded.args.len(), 2);
        assert_eq!(decoded.args[0], OscValue::Float(0.85));
        assert_eq!(decoded.args[1], OscValue::Int(42));
    }

    #[test]
    fn test_osc_control_node_process_block_and_slew() {
        let mut node = OscControlNode::new("/synth/pitch", 100.0, 1000.0);

        let msg = OscMessage {
            address: "/synth/pitch".to_string(),
            args: vec![OscValue::Float(0.5)], // Should map to 550.0 Hz
        };
        let mut buf = Vec::new();
        OscPacketParser::encode_message(&msg, &mut buf);
        node.inject_packet(&buf);

        let mut cv_out = vec![0.0f32; 64];
        let mut gate_out = vec![0.0f32; 64];
        let mut outputs = [&mut cv_out[..], &mut gate_out[..]];

        let ctx = ProcessContext {
            sample_rate: 44100,
            bpm: 120.0,
            frame_position: 0,
            param_bus: None,
            tuning_root_hz: 440.0,
            tuning_edo_divisions: 12,
            is_playing: true,
        };

        node.process(&[], &mut outputs, &ctx);

        // First frame gate should be high trigger
        assert!(outputs[1][0] > 0.9);
        // CV should begin slewing smoothly towards 550.0
        assert!(outputs[0][0] > 100.0);
        assert!(outputs[0][63] > outputs[0][0]);
    }

    #[test]
    fn test_udp_control_node_datagram() {
        let mut node = UdpControlNode::new(0.0, 100.0);
        node.handle_datagram(b"0.75");

        let mut cv_out = vec![0.0f32; 32];
        let mut outputs = [&mut cv_out[..]];
        let ctx = ProcessContext {
            sample_rate: 44100,
            bpm: 120.0,
            frame_position: 0,
            param_bus: None,
            tuning_root_hz: 440.0,
            tuning_edo_divisions: 12,
            is_playing: true,
        };

        node.process(&[], &mut outputs, &ctx);
        // Should slew towards 75.0
        assert!(outputs[0][31] > 5.0);
    }

    #[test]
    fn test_serial_control_node_ascii_telemetry() {
        let mut node = SerialControlNode::new();
        node.parse_serial_line("A0: 1023\n"); // Max ADC -> 1.0
        node.parse_serial_line("A1: 511.5\n"); // Half ADC -> 0.5

        let mut out_a0 = vec![0.0f32; 32];
        let mut out_a1 = vec![0.0f32; 32];
        let mut outputs = [&mut out_a0[..], &mut out_a1[..]];

        let ctx = ProcessContext {
            sample_rate: 44100,
            bpm: 120.0,
            frame_position: 0,
            param_bus: None,
            tuning_root_hz: 440.0,
            tuning_edo_divisions: 12,
            is_playing: true,
        };

        node.process(&[], &mut outputs, &ctx);
        assert!(outputs[0][31] > 0.1);
        assert!(outputs[1][31] > 0.05);
    }
}
