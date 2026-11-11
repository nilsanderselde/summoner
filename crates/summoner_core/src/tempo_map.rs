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

//! Tempo Map and Meter Architecture (Epic C, Task C5).
//!
//! Provides sample-accurate conversion between timeline Beats, Seconds, and Sample Frames,
//! supporting:
//! - Multiple tempo change events along the timeline (BPM changes).
//! - Instant jumps vs linear BPM ramps (accelerando / ritardando).
//! - Time signature / meter changes (e.g. 4/4, 3/4, 7/8, 5/4).
//! - Musical Bar:Beat:Fraction calculations for UI rulers, sequencers, and metronome accents.

use serde::{Deserialize, Serialize};

/// Curve style between tempo change points.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum TempoCurve {
    #[default]
    Jump,
    Ramp,
}

/// An individual tempo event on the timeline.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TempoPoint {
    pub beat: f64,
    pub bpm: f64,
    #[serde(default)]
    pub curve: TempoCurve,
    #[serde(default)]
    pub target_bpm: Option<f64>,
    #[serde(default)]
    pub ramp_length_beats: f64,
}

impl TempoPoint {
    /// Create a standard constant tempo jump point.
    pub fn jump(beat: f64, bpm: f64) -> Self {
        Self {
            beat,
            bpm: bpm.clamp(10.0, 999.0),
            curve: TempoCurve::Jump,
            target_bpm: None,
            ramp_length_beats: 0.0,
        }
    }

    /// Create a linear tempo ramp point.
    pub fn ramp(beat: f64, start_bpm: f64, end_bpm: f64, ramp_length_beats: f64) -> Self {
        Self {
            beat,
            bpm: start_bpm.clamp(10.0, 999.0),
            curve: TempoCurve::Ramp,
            target_bpm: Some(end_bpm.clamp(10.0, 999.0)),
            ramp_length_beats: ramp_length_beats.max(0.001),
        }
    }
}

/// Time signature / meter change event on the timeline.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MeterPoint {
    pub beat: F64Bits,
    pub numerator: u8,
    pub denominator: u8,
}

/// Helper to serialize/deserialize f64 in meter points while implementing Eq.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct F64Bits(pub f64);

impl PartialEq for F64Bits {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for F64Bits {}

impl MeterPoint {
    pub fn new(beat: f64, numerator: u8, denominator: u8) -> Self {
        Self {
            beat: F64Bits(beat),
            numerator: numerator.max(1),
            denominator: denominator.max(1),
        }
    }

    #[inline]
    pub fn beat(&self) -> f64 {
        self.beat.0
    }

    /// Duration of one complete bar in quarter beats.
    /// E.g. 4/4 = 4.0 beats; 3/4 = 3.0 beats; 6/8 = 6 * (4/8) = 3.0 beats.
    #[inline]
    pub fn bar_length_beats(&self) -> f64 {
        self.numerator as f64 * (4.0 / self.denominator as f64)
    }

    /// Duration of one metronome/sub-beat in quarter beats.
    #[inline]
    pub fn beat_length(&self) -> f64 {
        4.0 / self.denominator as f64
    }
}

/// Comprehensive DAW timeline tempo and meter map.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TempoMap {
    pub sample_rate: u32,
    pub tempo_points: Vec<TempoPoint>,
    pub meter_points: Vec<MeterPoint>,
}

impl TempoMap {
    /// Create a default tempo map with a single constant tempo and 4/4 meter.
    pub fn new(initial_bpm: f64, numerator: u8, denominator: u8, sample_rate: u32) -> Self {
        Self {
            sample_rate: sample_rate.max(1),
            tempo_points: vec![TempoPoint::jump(0.0, initial_bpm)],
            meter_points: vec![MeterPoint::new(0.0, numerator, denominator)],
        }
    }

    /// Add a tempo change point, maintaining sort order by beat.
    pub fn add_tempo_point(&mut self, point: TempoPoint) {
        if let Some(pos) = self
            .tempo_points
            .iter()
            .position(|p| (p.beat - point.beat).abs() < 1e-6)
        {
            self.tempo_points[pos] = point;
        } else {
            self.tempo_points.push(point);
            self.tempo_points.sort_by(|a, b| {
                a.beat
                    .partial_cmp(&b.beat)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
    }

    /// Add a meter change point, maintaining sort order by beat.
    pub fn add_meter_point(&mut self, point: MeterPoint) {
        if let Some(pos) = self
            .meter_points
            .iter()
            .position(|p| (p.beat() - point.beat()).abs() < 1e-6)
        {
            self.meter_points[pos] = point;
        } else {
            self.meter_points.push(point);
            self.meter_points.sort_by(|a, b| {
                a.beat()
                    .partial_cmp(&b.beat())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
    }

    /// Instantaneous BPM at a specified timeline beat.
    pub fn tempo_at_beat(&self, beat: f64) -> f64 {
        if self.tempo_points.is_empty() {
            return 120.0;
        }

        let beat = beat.max(0.0);
        let mut idx = 0;
        for (i, p) in self.tempo_points.iter().enumerate() {
            if beat >= p.beat - 1e-6 {
                idx = i;
            } else {
                break;
            }
        }

        let pt = &self.tempo_points[idx];
        match pt.curve {
            TempoCurve::Jump => pt.bpm,
            TempoCurve::Ramp => {
                if let Some(target_bpm) = pt.target_bpm {
                    let progress = (beat - pt.beat) / pt.ramp_length_beats.max(0.001);
                    if progress >= 1.0 {
                        target_bpm
                    } else {
                        pt.bpm + progress * (target_bpm - pt.bpm)
                    }
                } else {
                    pt.bpm
                }
            }
        }
    }

    /// Active meter (numerator, denominator) at a specified timeline beat.
    pub fn meter_at_beat(&self, beat: f64) -> (u8, u8) {
        if self.meter_points.is_empty() {
            return (4, 4);
        }

        let beat = beat.max(0.0);
        let mut idx = 0;
        for (i, p) in self.meter_points.iter().enumerate() {
            if beat >= p.beat() - 1e-6 {
                idx = i;
            } else {
                break;
            }
        }

        (
            self.meter_points[idx].numerator,
            self.meter_points[idx].denominator,
        )
    }

    /// Convert timeline beats into elapsed seconds from project start.
    pub fn beat_to_seconds(&self, target_beat: f64) -> f64 {
        if target_beat <= 0.0 || self.tempo_points.is_empty() {
            return 0.0;
        }

        let mut current_sec = 0.0;

        for i in 0..self.tempo_points.len() {
            let pt = &self.tempo_points[i];
            let next_beat = if i + 1 < self.tempo_points.len() {
                self.tempo_points[i + 1].beat
            } else {
                f64::INFINITY
            };

            let seg_start = pt.beat;
            let seg_end = next_beat.min(target_beat);

            if target_beat <= seg_start {
                break;
            }

            match pt.curve {
                TempoCurve::Jump => {
                    let delta_b = seg_end - seg_start;
                    current_sec += (delta_b * 60.0) / pt.bpm;
                }
                TempoCurve::Ramp => {
                    let target_bpm = pt.target_bpm.unwrap_or(pt.bpm);
                    let ramp_len = pt.ramp_length_beats.max(0.001);
                    let ramp_end_beat = seg_start + ramp_len;

                    // 1. Portion within the ramp
                    if seg_end <= ramp_end_beat {
                        current_sec += Self::integrate_ramp_seconds(
                            seg_start, seg_end, seg_start, ramp_len, pt.bpm, target_bpm,
                        );
                    } else {
                        // Traversed the entire ramp
                        current_sec += Self::integrate_ramp_seconds(
                            seg_start,
                            ramp_end_beat,
                            seg_start,
                            ramp_len,
                            pt.bpm,
                            target_bpm,
                        );
                        // Rest is constant at target_bpm
                        let flat_delta = seg_end - ramp_end_beat;
                        current_sec += (flat_delta * 60.0) / target_bpm;
                    }
                }
            }

            if target_beat <= next_beat {
                break;
            }
        }

        current_sec
    }

    fn integrate_ramp_seconds(
        b0: f64,
        b1: f64,
        ramp_start: f64,
        ramp_len: f64,
        start_bpm: f64,
        end_bpm: f64,
    ) -> f64 {
        if (end_bpm - start_bpm).abs() < 1e-6 {
            return ((b1 - b0) * 60.0) / start_bpm;
        }

        let k = (end_bpm - start_bpm) / ramp_len;
        let bpm0 = start_bpm + k * (b0 - ramp_start);
        let bpm1 = start_bpm + k * (b1 - ramp_start);

        (60.0 / k) * (bpm1 / bpm0).ln()
    }

    /// Convert elapsed seconds from project start into timeline beats.
    pub fn seconds_to_beat(&self, target_sec: f64) -> f64 {
        if target_sec <= 0.0 || self.tempo_points.is_empty() {
            return 0.0;
        }

        let mut low_beat = 0.0;
        let mut high_beat = target_sec * (self.tempo_at_beat(0.0) / 60.0) * 2.0 + 16.0;

        // Ensure upper bound bracket
        while self.beat_to_seconds(high_beat) < target_sec {
            high_beat *= 2.0;
        }

        // Bisection to converge to sub-sample accuracy
        for _ in 0..40 {
            let mid = 0.5 * (low_beat + high_beat);
            let sec = self.beat_to_seconds(mid);
            if sec < target_sec {
                low_beat = mid;
            } else {
                high_beat = mid;
            }
        }

        0.5 * (low_beat + high_beat)
    }

    /// Convert timeline beats to sample frame index.
    #[inline]
    pub fn beat_to_frame(&self, beat: f64) -> u64 {
        (self.beat_to_seconds(beat) * self.sample_rate as f64).round() as u64
    }

    /// Convert sample frame index to timeline beats.
    #[inline]
    pub fn frame_to_beat(&self, frame: u64) -> f64 {
        let sec = frame as f64 / self.sample_rate as f64;
        self.seconds_to_beat(sec)
    }

    /// Returns `(bar, beat_in_bar, sub_beat_fraction)`:
    /// - `bar`: 1-based bar number (Bar 1, Bar 2, ...)
    /// - `beat_in_bar`: 1-based beat within bar (e.g. 1, 2, 3, 4 for 4/4)
    /// - `sub_beat_fraction`: progress within current beat in range [0.0, 1.0).
    pub fn bar_beat_at(&self, beat: f64) -> (u32, u32, f64) {
        let beat = beat.max(0.0);
        let mut cur_bar: u32 = 1;
        let mut cur_beat = 0.0;

        for i in 0..self.meter_points.len() {
            let mp = &self.meter_points[i];
            let next_meter_beat = if i + 1 < self.meter_points.len() {
                self.meter_points[i + 1].beat()
            } else {
                f64::INFINITY
            };

            let bar_len = mp.bar_length_beats();
            let beat_step = mp.beat_length();

            let seg_end = next_meter_beat.min(beat);
            let seg_beats = seg_end - cur_beat;

            if seg_beats > 0.0 {
                let complete_bars = (seg_beats / bar_len).floor() as u32;
                let remainder_beats = seg_beats - (complete_bars as f64 * bar_len);

                if beat <= next_meter_beat {
                    let bar_num = cur_bar + complete_bars;
                    let beat_idx = (remainder_beats / beat_step).floor() as u32;
                    let sub_beat = (remainder_beats - (beat_idx as f64 * beat_step)) / beat_step;
                    return (bar_num, beat_idx + 1, sub_beat);
                }

                cur_bar += complete_bars;
                cur_beat = next_meter_beat;
            }
        }

        (cur_bar, 1, 0.0)
    }

    /// Test if a beat lands on a bar downbeat (beat 1 of a bar).
    pub fn is_bar_downbeat(&self, beat: f64, tolerance: f64) -> bool {
        let (_, beat_idx, frac) = self.bar_beat_at(beat);
        beat_idx == 1 && (frac < tolerance || (1.0 - frac) < tolerance)
    }

    /// Test if a beat lands on a meter beat division (for metronome clicks).
    pub fn is_meter_beat(&self, beat: f64, tolerance: f64) -> bool {
        let (_, _, frac) = self.bar_beat_at(beat);
        frac < tolerance || (1.0 - frac) < tolerance
    }
}

impl Default for TempoMap {
    fn default() -> Self {
        Self::new(120.0, 4, 4, 44100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_tempo_conversion() {
        let map = TempoMap::new(120.0, 4, 4, 44100);
        // At 120 BPM, 1 beat = 0.5s = 22050 frames
        assert!((map.beat_to_seconds(1.0) - 0.5).abs() < 1e-6);
        assert!((map.beat_to_seconds(4.0) - 2.0).abs() < 1e-6);
        assert_eq!(map.beat_to_frame(1.0), 22050);
        assert_eq!(map.beat_to_frame(4.0), 88200);

        assert!((map.seconds_to_beat(0.5) - 1.0).abs() < 1e-5);
        assert!((map.seconds_to_beat(2.0) - 4.0).abs() < 1e-5);
        assert!((map.frame_to_beat(22050) - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_tempo_jump_change() {
        let mut map = TempoMap::new(120.0, 4, 4, 44100);
        // Change to 60 BPM at beat 4.0
        map.add_tempo_point(TempoPoint::jump(4.0, 60.0));

        // First 4 beats at 120 BPM = 2.0s
        assert!((map.beat_to_seconds(4.0) - 2.0).abs() < 1e-6);
        // Next 1 beat at 60 BPM = 1.0s, total = 3.0s
        assert!((map.beat_to_seconds(5.0) - 3.0).abs() < 1e-6);

        assert!((map.seconds_to_beat(2.0) - 4.0).abs() < 1e-4);
        assert!((map.seconds_to_beat(3.0) - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_tempo_ramp() {
        let mut map = TempoMap::new(120.0, 4, 4, 44100);
        // Ramp from 120 BPM to 240 BPM across 4 beats (from beat 0 to 4)
        map.tempo_points[0] = TempoPoint::ramp(0.0, 120.0, 240.0, 4.0);

        assert!((map.tempo_at_beat(0.0) - 120.0).abs() < 1e-4);
        assert!((map.tempo_at_beat(2.0) - 180.0).abs() < 1e-4);
        assert!((map.tempo_at_beat(4.0) - 240.0).abs() < 1e-4);

        let sec = map.beat_to_seconds(4.0);
        assert!(sec > 0.0 && sec < 2.0); // Faster than 120 BPM constant (2.0s)

        let roundtrip_beat = map.seconds_to_beat(sec);
        assert!((roundtrip_beat - 4.0).abs() < 1e-3);
    }

    #[test]
    fn test_bar_beat_meter_calculation() {
        let mut map = TempoMap::new(120.0, 4, 4, 44100);
        // 4/4 meter: 4 beats per bar
        let (bar, beat, frac) = map.bar_beat_at(0.0);
        assert_eq!((bar, beat), (1, 1));
        assert!(frac.abs() < 1e-6);

        let (bar, beat, _) = map.bar_beat_at(3.0);
        assert_eq!((bar, beat), (1, 4));

        let (bar, beat, _) = map.bar_beat_at(4.0);
        assert_eq!((bar, beat), (2, 1));

        // Switch to 3/4 at beat 8 (Bar 3)
        map.add_meter_point(MeterPoint::new(8.0, 3, 4));
        let (bar, beat, _) = map.bar_beat_at(8.0);
        assert_eq!((bar, beat), (3, 1));

        let (bar, beat, _) = map.bar_beat_at(10.0);
        assert_eq!((bar, beat), (3, 3));

        let (bar, beat, _) = map.bar_beat_at(11.0);
        assert_eq!((bar, beat), (4, 1));
    }
}
