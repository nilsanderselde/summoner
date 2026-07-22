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

//! Sample-accurate deterministic transport engine.

/// Transport state representing global timeline clock and playback controls.
#[derive(Debug, Clone, PartialEq)]
pub struct Transport {
    pub sample_rate: u32,
    pub bpm: f64,
    pub frame_position: u64,
    pub is_playing: bool,
    pub time_signature_num: u8,
    pub time_signature_den: u8,
}

impl Transport {
    pub fn new(sample_rate: u32, bpm: f64) -> Self {
        Self {
            sample_rate,
            bpm,
            frame_position: 0,
            is_playing: false,
            time_signature_num: 4,
            time_signature_den: 4,
        }
    }

    pub fn play(&mut self) {
        self.is_playing = true;
    }

    pub fn pause(&mut self) {
        self.is_playing = false;
    }

    pub fn stop(&mut self) {
        self.is_playing = false;
        self.frame_position = 0;
    }

    pub fn seek_frame(&mut self, frame: u64) {
        self.frame_position = frame;
    }

    /// Advance timeline by `frame_count` frames.
    pub fn advance_frames(&mut self, frame_count: u64) {
        if self.is_playing {
            self.frame_position = self.frame_position.saturating_add(frame_count);
        }
    }

    /// Current timeline position in seconds.
    pub fn seconds(&self) -> f64 {
        if self.sample_rate == 0 {
            return 0.0;
        }
        self.frame_position as f64 / self.sample_rate as f64
    }

    /// Current timeline position in musical beats.
    pub fn beats(&self) -> f64 {
        (self.seconds() * self.bpm) / 60.0
    }
}

impl Default for Transport {
    fn default() -> Self {
        Self::new(44100, 120.0)
    }
}
