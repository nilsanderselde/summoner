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

use crate::tempo_map::TempoMap;

/// Transport state representing global timeline clock and playback controls.
#[derive(Debug, Clone, PartialEq)]
pub struct Transport {
    pub sample_rate: u32,
    pub bpm: f64,
    pub frame_position: u64,
    pub is_playing: bool,
    pub time_signature_num: u8,
    pub time_signature_den: u8,
    pub tempo_map: TempoMap,
}

impl Transport {
    pub fn new(sample_rate: u32, bpm: f64) -> Self {
        let tempo_map = TempoMap::new(bpm, 4, 4, sample_rate);
        Self {
            sample_rate,
            bpm,
            frame_position: 0,
            is_playing: false,
            time_signature_num: 4,
            time_signature_den: 4,
            tempo_map,
        }
    }

    /// Create transport initialized with a custom tempo map.
    pub fn with_tempo_map(tempo_map: TempoMap) -> Self {
        let sr = tempo_map.sample_rate;
        let initial_bpm = tempo_map.tempo_at_beat(0.0);
        let (num, den) = tempo_map.meter_at_beat(0.0);
        Self {
            sample_rate: sr,
            bpm: initial_bpm,
            frame_position: 0,
            is_playing: false,
            time_signature_num: num,
            time_signature_den: den,
            tempo_map,
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
        self.update_tempo_state();
    }

    pub fn seek_frame(&mut self, frame: u64) {
        self.frame_position = frame;
        self.update_tempo_state();
    }

    /// Seek to a specific beat position using the tempo map.
    pub fn seek_beat(&mut self, beat: f64) {
        self.frame_position = self.tempo_map.beat_to_frame(beat);
        self.update_tempo_state();
    }

    /// Advance timeline by `frame_count` frames.
    pub fn advance_frames(&mut self, frame_count: u64) {
        if self.is_playing {
            self.frame_position = self.frame_position.saturating_add(frame_count);
            self.update_tempo_state();
        }
    }

    fn update_tempo_state(&mut self) {
        let b = self.beats();
        self.bpm = self.tempo_map.tempo_at_beat(b);
        let (num, den) = self.tempo_map.meter_at_beat(b);
        self.time_signature_num = num;
        self.time_signature_den = den;
    }

    /// Current timeline position in seconds.
    pub fn seconds(&self) -> f64 {
        if self.sample_rate == 0 {
            return 0.0;
        }
        self.frame_position as f64 / self.sample_rate as f64
    }

    /// Current timeline position in musical beats calculated via the tempo map.
    pub fn beats(&self) -> f64 {
        self.tempo_map.frame_to_beat(self.frame_position)
    }

    /// Current musical bar and beat position: `(bar, beat_in_bar, sub_beat_fraction)`.
    pub fn bar_beat(&self) -> (u32, u32, f64) {
        self.tempo_map.bar_beat_at(self.beats())
    }
}

impl Default for Transport {
    fn default() -> Self {
        Self::new(44100, 120.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tempo_map::TempoPoint;

    #[test]
    fn test_transport_with_tempo_changes() {
        let mut transport = Transport::new(44100, 120.0);
        // Add tempo change at beat 4 to 60 BPM
        transport.tempo_map.add_tempo_point(TempoPoint::jump(4.0, 60.0));
        transport.play();

        // Advance 2 seconds (44100 * 2 frames = 88200) -> should be at beat 4.0
        transport.advance_frames(88200);
        assert!((transport.beats() - 4.0).abs() < 1e-4);
        assert_eq!(transport.bpm, 60.0);

        // Advance 1 second at 60 BPM (44100 frames) -> should be at beat 5.0
        transport.advance_frames(44100);
        assert!((transport.beats() - 5.0).abs() < 1e-4);

        let (bar, beat, _) = transport.bar_beat();
        assert_eq!((bar, beat), (2, 1)); // Beat 5 is Bar 2, Beat 1 in 4/4
    }
}
