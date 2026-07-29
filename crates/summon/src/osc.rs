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

//! OSC (Open Sound Control) Remote Control Server (UDP Port 8000).

use std::net::UdpSocket;

/// Remote control commands sent over OSC / UDP.
#[derive(Debug, Clone, PartialEq)]
pub enum OscCommand {
    Play,
    Stop,
    SetBpm(f64),
    SetParam(u64, f32),
}

/// UDP listener server for OSC commands.
pub struct OscServer {
    socket: UdpSocket,
}

impl OscServer {
    /// Bind server to specified port (default 8000).
    pub fn bind(port: u16) -> std::io::Result<Self> {
        let addr = format!("0.0.0.0:{}", port);
        let socket = UdpSocket::bind(&addr)?;
        socket.set_nonblocking(true)?;
        Ok(Self { socket })
    }

    /// Non-blocking check for incoming OSC commands.
    pub fn poll_command(&self) -> Option<OscCommand> {
        let mut buf = [0u8; 512];
        if let Ok((amt, _src)) = self.socket.recv_from(&mut buf) {
            let msg = String::from_utf8_lossy(&buf[..amt]).trim().to_string();
            Self::parse_packet(&msg)
        } else {
            None
        }
    }

    /// Parse an OSC address / text command string.
    pub fn parse_packet(raw: &str) -> Option<OscCommand> {
        let parts: Vec<&str> = raw.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        match parts[0] {
            "/play" | "play" => Some(OscCommand::Play),
            "/stop" | "stop" => Some(OscCommand::Stop),
            "/bpm" | "bpm" => {
                if parts.len() >= 2 {
                    if let Ok(val) = parts[1].parse::<f64>() {
                        return Some(OscCommand::SetBpm(val));
                    }
                }
                None
            }
            "/param" | "param" => {
                if parts.len() >= 3 {
                    if let (Ok(id), Ok(val)) = (parts[1].parse::<u64>(), parts[2].parse::<f32>()) {
                        return Some(OscCommand::SetParam(id, val));
                    }
                }
                None
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_osc_command_parsing() {
        assert_eq!(OscServer::parse_packet("/play"), Some(OscCommand::Play));
        assert_eq!(OscServer::parse_packet("/stop"), Some(OscCommand::Stop));
        assert_eq!(OscServer::parse_packet("/bpm 140.5"), Some(OscCommand::SetBpm(140.5)));
        assert_eq!(OscServer::parse_packet("/param 101 0.75"), Some(OscCommand::SetParam(101, 0.75)));
    }
}
