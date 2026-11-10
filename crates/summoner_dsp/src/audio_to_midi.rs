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

//! Audio-to-MIDI conversion, Slice-to-Drum-Pad, Slice-to-MIDI, and Audio Quantization.

use crate::drum_machine::{DrumMachineDevice, DrumPad};
use crate::pitch_shift::{detect_transients, ElasticWarpEngine, WarpMode};
use crate::sampler::{LoopMode, SampleBuffer};
use std::sync::Arc;
use summoner_core::audio::Sample;
use summoner_core::midi::MidiEvent;

/// Audio-to-MIDI transcription engine for monophonic and percussive audio.
pub struct AudioToMidiConverter {
    pub sample_rate: u32,
    pub min_freq_hz: f32, // e.g. 50.0 Hz
    pub max_freq_hz: f32, // e.g. 1200.0 Hz
    pub silence_threshold_db: f32, // e.g. -40.0 dB
}

impl AudioToMidiConverter {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            min_freq_hz: 50.0,
            max_freq_hz: 1200.0,
            silence_threshold_db: -40.0,
        }
    }

    /// Transcribe monophonic pitch into standard `MidiEvent` sequence.
    pub fn transcribe_mono(&self, audio: &[Sample]) -> Vec<MidiEvent> {
        if audio.is_empty() {
            return Vec::new();
        }

        let frame_size = 2048;
        let hop_size = 512;
        let threshold_amp = 10.0f32.powf(self.silence_threshold_db / 20.0);

        let mut events = Vec::new();
        let mut active_note: Option<(u8, u8)> = None; // (note, velocity)
        let mut pos = 0;

        while pos + frame_size <= audio.len() {
            let window = &audio[pos..pos + frame_size];
            let rms: f32 = (window.iter().map(|&s| s * s).sum::<f32>() / frame_size as f32).sqrt();

            if rms > threshold_amp {
                if let Some(freq) = detect_pitch_autocorr(window, self.sample_rate, self.min_freq_hz, self.max_freq_hz) {
                    let midi_pitch = (69.0 + 12.0 * (freq / 440.0).log2()).round().clamp(12.0, 120.0) as u8;
                    let vel = (rms * 2.0).clamp(0.1, 1.0) * 127.0;
                    let vel_u8 = vel.round() as u8;

                    match active_note {
                        Some((cur_note, _)) if cur_note == midi_pitch => {
                            // Note continues sounding
                        }
                        Some((cur_note, _)) => {
                            // Pitch changed: end current note, start new note
                            events.push(MidiEvent::NoteOff(0, cur_note, 0));
                            events.push(MidiEvent::NoteOn(0, midi_pitch, vel_u8));
                            active_note = Some((midi_pitch, vel_u8));
                        }
                        None => {
                            // Note start
                            events.push(MidiEvent::NoteOn(0, midi_pitch, vel_u8));
                            active_note = Some((midi_pitch, vel_u8));
                        }
                    }
                } else if let Some((cur_note, _)) = active_note.take() {
                    events.push(MidiEvent::NoteOff(0, cur_note, 0));
                }
            } else if let Some((cur_note, _)) = active_note.take() {
                events.push(MidiEvent::NoteOff(0, cur_note, 0));
            }

            pos += hop_size;
        }

        if let Some((cur_note, _)) = active_note {
            events.push(MidiEvent::NoteOff(0, cur_note, 0));
        }

        events
    }
}

/// Squared-difference pitch estimator.
fn detect_pitch_autocorr(window: &[f32], sample_rate: u32, min_hz: f32, max_hz: f32) -> Option<f32> {
    let min_period = (sample_rate as f32 / max_hz).floor() as usize;
    let max_period = (sample_rate as f32 / min_hz).ceil() as usize;

    if max_period >= window.len() {
        return None;
    }

    let mut diffs = Vec::with_capacity(max_period - min_period);
    let n = window.len() - max_period;

    for lag in min_period..max_period {
        let mut diff = 0.0f32;
        for i in 0..n {
            let d = window[i] - window[i + lag];
            diff += d * d;
        }
        diffs.push((lag, diff));
    }

    // Find first local minimum to avoid octave errors
    let mut best_period = 0;
    for i in 1..diffs.len().saturating_sub(1) {
        let (lag, d) = diffs[i];
        if d <= diffs[i - 1].1 && d <= diffs[i + 1].1 && d < 10.0 {
            best_period = lag;
            break;
        }
    }

    if best_period == 0 {
        if let Some(&(lag, _)) = diffs.iter().min_by(|a, b| a.1.partial_cmp(&b.1).unwrap()) {
            best_period = lag;
        }
    }

    if best_period > 0 {
        Some(sample_rate as f32 / best_period as f32)
    } else {
        None
    }
}

// =========================================================================
// Slice to Drum Pad & Slice to MIDI
// =========================================================================

/// Result of slicing an audio file into drum pads and matching MIDI trigger sequence.
pub struct SlicedDrumKit {
    pub drum_machine: DrumMachineDevice,
    pub midi_trigger_events: Vec<MidiEvent>,
    pub slice_boundaries: Vec<(usize, usize)>,
}

/// Slices an audio loop at transient boundaries into a `DrumMachineDevice` with trigger MIDI.
pub fn slice_to_drum_pad(
    audio: &[Sample],
    sample_rate: u32,
    sensitivity: f32,
) -> SlicedDrumKit {
    let transients = detect_transients(audio, sample_rate, sensitivity);
    let mut boundaries = Vec::new();

    let mut start = 0;
    for &t in &transients {
        if t > start && t < audio.len() {
            boundaries.push((start, t));
            start = t;
        }
    }
    if start < audio.len() {
        boundaries.push((start, audio.len()));
    }

    let mut kit = DrumMachineDevice::new("Sliced Loop Kit");
    let mut midi_events = Vec::new();

    // GM Standard starts at note 36 (C1)
    let base_note = 36u8;

    for (i, &(src_start, src_end)) in boundaries.iter().take(crate::drum_machine::MAX_PADS).enumerate() {
        let note = base_note + i as u8;
        let slice_samples = audio[src_start..src_end].to_vec();
        let buf = Arc::new(SampleBuffer::new(slice_samples, sample_rate, 1));

        let mut pad = DrumPad::new(
            format!("Slice {:02}", i + 1),
            note,
            0.001,
            0.4,
            0.0,
            0.05,
            1.0,
        );
        pad.add_sample(note, note, note, format!("slice_{}.wav", i), Some(buf), LoopMode::NoLoop);
        kit.add_pad(pad);

        // Add a trigger event
        midi_events.push(MidiEvent::NoteOn(0, note, 100));
        midi_events.push(MidiEvent::NoteOff(0, note, 0));
    }

    SlicedDrumKit {
        drum_machine: kit,
        midi_trigger_events: midi_events,
        slice_boundaries: boundaries,
    }
}

// =========================================================================
// Audio Quantization
// =========================================================================

/// Quantize audio transient hits to a musical beat grid.
pub fn quantize_audio(
    audio: &[Sample],
    sample_rate: u32,
    bpm: f64,
    grid_beats: f64, // e.g. 0.25 for 1/16th notes
    strength: f32,   // 0.0 (no change) to 1.0 (100% hard snap)
) -> Vec<Sample> {
    if audio.is_empty() || bpm <= 0.0 || grid_beats <= 0.0 {
        return audio.to_vec();
    }

    let transients = detect_transients(audio, sample_rate, 1.0);
    if transients.is_empty() {
        return audio.to_vec();
    }

    let beats_per_second = bpm / 60.0;
    let samples_per_beat = (sample_rate as f64) / beats_per_second;
    let grid_samples = samples_per_beat * grid_beats;

    let mut warp = ElasticWarpEngine::new(sample_rate).with_mode(WarpMode::Beats);

    for &src_frame in &transients {
        // Find nearest grid line
        let grid_index = (src_frame as f64 / grid_samples).round();
        let target_grid_frame = grid_index * grid_samples;

        // Apply quantization strength interpolation
        let final_target_frame = (src_frame as f64 + (target_grid_frame - src_frame as f64) * (strength as f64))
            .round() as usize;

        warp.add_marker(src_frame, final_target_frame);
    }

    warp.warp_audio(audio, audio.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_to_midi_transcription() {
        let sample_rate = 44100;
        let converter = AudioToMidiConverter::new(sample_rate);

        // Generate 0.5s of 440 Hz (A4 = MIDI note 69)
        let num_samples = 22050;
        let mut audio = Vec::with_capacity(num_samples);
        for i in 0..num_samples {
            audio.push((i as f32 * 440.0 * 2.0 * std::f32::consts::PI / 44100.0).sin() * 0.8);
        }

        let events = converter.transcribe_mono(&audio);
        assert!(!events.is_empty(), "Transcriber must detect pitch");

        let has_a4 = events.iter().any(|&e| match e {
            MidiEvent::NoteOn(_, note, _) => note == 69,
            _ => false,
        });
        assert!(has_a4, "Detected MIDI note should be A4 (69)");
    }

    #[test]
    fn test_slice_to_drum_pad_and_midi() {
        let sample_rate = 44100;
        let mut audio = vec![0.0f32; 44100]; // 1s loop with 3 hits
        audio[5000] = 1.0;
        audio[15000] = 1.0;
        audio[25000] = 1.0;

        let sliced = slice_to_drum_pad(&audio, sample_rate, 0.2);
        assert!(sliced.drum_machine.pads.len() >= 2);
        assert_eq!(sliced.drum_machine.pads[0].midi_note, 36);
        assert!(!sliced.midi_trigger_events.is_empty());
    }

    #[test]
    fn test_audio_quantize() {
        let sample_rate = 44100;
        let mut audio = vec![0.0f32; 44100];
        // Trigger a transient slightly off the 1/4 note beat
        audio[10000] = 1.0;

        let quantized = quantize_audio(&audio, sample_rate, 120.0, 0.25, 1.0);
        assert_eq!(quantized.len(), audio.len());
    }
}
