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

//! MIDI Clock Synchronization (0xF8, 24 PPQN).

use std::time::Instant;

/// Raw MIDI Clock status byte (System Real-Time).
pub const MIDI_CLOCK_BYTE: u8 = 0xF8;

/// Standard pulses per quarter note for MIDI clock.
pub const MIDI_CLOCK_PPQN: u32 = 24;

/// Generates outgoing MIDI clock ticks at 24 PPQN based on transport state.
#[derive(Debug, Clone)]
pub struct MidiClockGenerator {
    sample_rate: u32,
    bpm: f64,
    accumulated_frames: f64,
}

impl MidiClockGenerator {
    /// Create a new generator for given sample rate and BPM.
    pub fn new(sample_rate: u32, bpm: f64) -> Self {
        Self {
            sample_rate,
            bpm,
            accumulated_frames: 0.0,
        }
    }

    /// Update transport parameters.
    pub fn set_bpm(&mut self, bpm: f64) {
        self.bpm = bpm;
    }

    /// Number of frames per clock tick at current BPM and sample rate.
    pub fn frames_per_tick(&self) -> f64 {
        if self.bpm <= 0.0 || self.sample_rate == 0 {
            return f64::MAX;
        }
        let beats_per_sec = self.bpm / 60.0;
        let ticks_per_sec = beats_per_sec * MIDI_CLOCK_PPQN as f64;
        self.sample_rate as f64 / ticks_per_sec
    }

    /// Advance timeline by `frames`, returning number of 0xF8 ticks to emit.
    pub fn advance(&mut self, frames: u64) -> u32 {
        let step = self.frames_per_tick();
        if step == f64::MAX || step <= 0.0 {
            return 0;
        }
        self.accumulated_frames += frames as f64;
        let ticks = (self.accumulated_frames / step).floor() as u32;
        if ticks > 0 {
            self.accumulated_frames -= ticks as f64 * step;
        }
        ticks
    }

    /// Reset accumulator.
    pub fn reset(&mut self) {
        self.accumulated_frames = 0.0;
    }
}

/// Receives external 0xF8 MIDI clock bytes and calculates tempo / beat advancement.
#[derive(Debug, Clone)]
pub struct MidiClockReceiver {
    sample_rate: u32,
    last_tick_time: Option<Instant>,
    tick_intervals: Vec<f64>,
    tick_count: u64,
}

impl MidiClockReceiver {
    /// Create new receiver for sample rate.
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            last_tick_time: None,
            tick_intervals: Vec::with_capacity(24),
            tick_count: 0,
        }
    }

    /// Process incoming MIDI byte. Returns `Some(bpm)` when new tempo calculation is available.
    pub fn process_byte(&mut self, byte: u8, now: Instant) -> Option<f64> {
        if byte != MIDI_CLOCK_BYTE {
            return None;
        }

        self.tick_count += 1;
        let mut calculated_bpm = None;

        if let Some(prev) = self.last_tick_time {
            let dt = now.duration_since(prev).as_secs_f64();
            if dt > 0.001 && dt < 1.0 {
                self.tick_intervals.push(dt);
                if self.tick_intervals.len() > 24 {
                    self.tick_intervals.remove(0);
                }

                if self.tick_intervals.len() >= 6 {
                    let avg_interval: f64 = self.tick_intervals.iter().sum::<f64>() / self.tick_intervals.len() as f64;
                    if avg_interval > 0.0 {
                        let ticks_per_sec = 1.0 / avg_interval;
                        let beats_per_sec = ticks_per_sec / MIDI_CLOCK_PPQN as f64;
                        let bpm = (beats_per_sec * 60.0).clamp(20.0, 300.0);
                        calculated_bpm = Some((bpm * 10.0).round() / 10.0);
                    }
                }
            }
        }

        self.last_tick_time = Some(now);
        calculated_bpm
    }

    /// Total received MIDI clock tick count.
    pub fn tick_count(&self) -> u64 {
        self.tick_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_midi_clock_generator_24ppqn() {
        let mut gen = MidiClockGenerator::new(44100, 120.0);
        // At 120 BPM, 1 beat = 0.5s = 22050 frames.
        // 24 ticks per beat => 22050 / 24 = 918.75 frames per tick.
        let ticks_in_beat = gen.advance(22050);
        assert_eq!(ticks_in_beat, 24);
    }

    #[test]
    fn test_midi_clock_receiver_bpm_calc() {
        let mut recv = MidiClockReceiver::new(44100);
        let start = Instant::now();

        // 120 BPM => 24 ticks per 0.5s => tick interval = 0.5 / 24 = 0.0208333s (20.83ms)
        let interval = Duration::from_micros(20833);
        let mut last_bpm = None;

        for i in 0..30 {
            let t = start + interval * i;
            if let Some(bpm) = recv.process_byte(MIDI_CLOCK_BYTE, t) {
                last_bpm = Some(bpm);
            }
        }

        assert!(last_bpm.is_some());
        let bpm = last_bpm.unwrap();
        assert!((bpm - 120.0).abs() < 2.0, "Expected ~120 BPM, got {}", bpm);
    }
}
