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

//! Real-time sample-accurate metronome click generator (Epic C, Task C11).
//!
//! Provides:
//! - High-pitch accented downbeat click on Measure Beat 1 (e.g. 1600 Hz).
//! - Sub-beat clicks on subsequent beats (e.g. 800 Hz).
//! - Count-in bars preroll mode for punch recording.
//! - Exponential decay envelope avoiding clicks and DC offset.
//! - Zero heap allocation during processing (AllocGuard RT-safe).

use summoner_core::transport::Transport;

/// Real-time sample-accurate metronome generator.
#[derive(Debug, Clone)]
pub struct MetronomeGenerator {
    pub sample_rate: u32,
    pub enabled: bool,
    pub gain: f32,
    pub count_in_bars: u32,
    pub count_in_remaining_bars: u32,
    pub is_counting_in: bool,
    pub downbeat_freq_hz: f32,
    pub beat_freq_hz: f32,
    click_phase: f32,
    click_phase_inc: f32,
    click_remaining_frames: usize,
    click_decay: f32,
    click_amplitude: f32,
    last_beat_id: i64,
}

impl MetronomeGenerator {
    pub fn new(sample_rate: u32) -> Self {
        let sr = sample_rate.max(1);
        Self {
            sample_rate: sr,
            enabled: false,
            gain: 0.7,
            count_in_bars: 0,
            count_in_remaining_bars: 0,
            is_counting_in: false,
            downbeat_freq_hz: 1600.0,
            beat_freq_hz: 800.0,
            click_phase: 0.0,
            click_phase_inc: 0.0,
            click_remaining_frames: 0,
            click_decay: 0.985,
            click_amplitude: 0.0,
            last_beat_id: -1,
        }
    }

    /// Trigger a click sound of given frequency and duration.
    pub fn trigger_click(&mut self, freq_hz: f32) {
        self.click_phase = 0.0;
        self.click_phase_inc = freq_hz / self.sample_rate as f32;
        // ~12 ms click duration
        let duration_frames = (0.012 * self.sample_rate as f32) as usize;
        self.click_remaining_frames = duration_frames.max(16);
        self.click_amplitude = 1.0;
        // Compute decay per frame so amplitude drops by ~60 dB over duration
        self.click_decay = (-6.9077 / self.click_remaining_frames as f32).exp();
    }

    /// Start recording with count-in preroll bars.
    pub fn start_count_in(&mut self, bars: u32) {
        self.count_in_bars = bars;
        self.count_in_remaining_bars = bars;
        self.is_counting_in = bars > 0;
        self.last_beat_id = -1;
    }

    /// Process a block of audio, mixing metronome clicks into the left and right output buffers.
    pub fn process_block(
        &mut self,
        transport: &Transport,
        block_size: usize,
        out_l: &mut [f32],
        out_r: &mut [f32],
    ) {
        if !self.enabled && !self.is_counting_in {
            return;
        }

        let num_frames = block_size.min(out_l.len()).min(out_r.len());

        for k in 0..num_frames {
            let frame = transport.frame_position + k as u64;
            let beat = transport.tempo_map.frame_to_beat(frame);
            let (bar, beat_in_bar, _) = transport.tempo_map.bar_beat_at(beat);
            let (num, _) = transport.tempo_map.meter_at_beat(beat);

            let beat_id = ((bar as i64) * (num as i64)) + (beat_in_bar as i64);

            if beat_id != self.last_beat_id {
                self.last_beat_id = beat_id;
                let is_downbeat = beat_in_bar == 1;
                let freq = if is_downbeat {
                    self.downbeat_freq_hz
                } else {
                    self.beat_freq_hz
                };
                self.trigger_click(freq);

                // Handle count-in bar decrement on bar boundary
                if self.is_counting_in && is_downbeat && self.count_in_remaining_bars > 0 {
                    self.count_in_remaining_bars -= 1;
                    if self.count_in_remaining_bars == 0 {
                        self.is_counting_in = false;
                    }
                }
            }

            if self.click_remaining_frames > 0 {
                let sample = (self.click_phase * std::f32::consts::TAU).sin()
                    * self.click_amplitude
                    * self.gain;
                out_l[k] += sample;
                out_r[k] += sample;

                self.click_phase = (self.click_phase + self.click_phase_inc).fract();
                self.click_amplitude *= self.click_decay;
                self.click_remaining_frames -= 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metronome_generates_clicks_on_beats() {
        let mut metronome = MetronomeGenerator::new(44100);
        metronome.enabled = true;
        metronome.gain = 1.0;

        let mut transport = Transport::new(44100, 120.0);
        transport.play();

        let mut out_l = vec![0.0f32; 1024];
        let mut out_r = vec![0.0f32; 1024];

        // Process first block (starting at beat 0, downbeat)
        metronome.process_block(&transport, 1024, &mut out_l, &mut out_r);

        // Click should produce non-zero samples
        let max_val = out_l.iter().fold(0.0f32, |acc, &s| acc.max(s.abs()));
        assert!(max_val > 0.1, "Metronome should generate audible click");
        assert_eq!(out_l, out_r, "Metronome should be centered in stereo");
    }

    #[test]
    fn test_metronome_disabled_produces_silence() {
        let mut metronome = MetronomeGenerator::new(44100);
        metronome.enabled = false;

        let transport = Transport::new(44100, 120.0);
        let mut out_l = vec![0.0f32; 512];
        let mut out_r = vec![0.0f32; 512];

        metronome.process_block(&transport, 512, &mut out_l, &mut out_r);
        assert!(out_l.iter().all(|&s| s == 0.0));
    }
}
