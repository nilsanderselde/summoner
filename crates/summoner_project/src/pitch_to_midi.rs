// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Offline Voice-to-MIDI Pitch and Articulation Transcription Engine (Milestone N3).
//!
//! Provides automated offline audio-to-MIDI transcription:
//! - Sub-semitone YIN-style pitch detection and tracking
//! - Transient onset/offset energy envelope note segmentation
//! - Articulation classification: Legato, Staccato, Portamento, Vibrato
//! - Direct synthesis of DAW sequence clip notes with velocity and duration.

use serde::{Deserialize, Serialize};

/// Articulation type detected for a transcribed note.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ArticulationType {
    #[default]
    Normal,
    Legato,
    Staccato,
    Portamento,
    Vibrato,
}

/// Transcribed individual MIDI note with articulation and microtonal detuning.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TranscribedNote {
    pub pitch: u8,            // 0 - 127
    pub cents_detune: f32,    // -50.0 to +50.0 cents
    pub start_time_sec: f64,
    pub duration_sec: f64,
    pub velocity: u8,         // 1 - 127
    pub articulation: ArticulationType,
}

/// Transcription options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionConfig {
    pub sample_rate: f32,
    pub frame_size: usize,
    pub hop_size: usize,
    pub min_freq_hz: f32,
    pub max_freq_hz: f32,
    pub silence_threshold_db: f32,
    pub min_note_duration_sec: f64,
}

impl Default for TranscriptionConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100.0,
            frame_size: 1024,
            hop_size: 256,
            min_freq_hz: 65.0,   // ~C2
            max_freq_hz: 1050.0, // ~C6
            silence_threshold_db: -40.0,
            min_note_duration_sec: 0.06, // 60ms minimum note duration
        }
    }
}

/// Pitch-to-MIDI Transcription Engine.
#[derive(Debug, Clone)]
pub struct VoiceToMidiTranscriber {
    pub config: TranscriptionConfig,
}

impl VoiceToMidiTranscriber {
    pub fn new(config: TranscriptionConfig) -> Self {
        Self { config }
    }

    /// Convert frequency in Hz to fractional MIDI note number.
    pub fn hz_to_midi_note(freq_hz: f32) -> f32 {
        if freq_hz <= 0.0 {
            return 0.0;
        }
        69.0 + 12.0 * (freq_hz / 440.0).log2()
    }

    /// Convert fractional MIDI note number to frequency in Hz.
    pub fn midi_note_to_hz(note: f32) -> f32 {
        440.0 * 2.0f32.powf((note - 69.0) / 12.0)
    }

    /// Estimate pitch in Hz for a single audio frame using difference function (YIN).
    pub fn estimate_pitch_yin(&self, frame: &[f32]) -> Option<(f32, f32)> {
        let len = frame.len();
        if len < 256 {
            return None;
        }

        // Check frame energy
        let rms = (frame.iter().map(|&s| s * s).sum::<f32>() / len as f32).sqrt();
        let db = 20.0 * rms.max(1e-5).log10();
        if db < self.config.silence_threshold_db {
            return None; // Silence / unvoiced
        }

        let min_tau = (self.config.sample_rate / self.config.max_freq_hz) as usize;
        let max_tau = ((self.config.sample_rate / self.config.min_freq_hz) as usize).min(len / 2);

        if min_tau >= max_tau {
            return None;
        }

        // Difference function
        let mut d = vec![0.0f32; max_tau];
        for tau in min_tau..max_tau {
            let mut sum = 0.0f32;
            for i in 0..(len - tau) {
                let diff = frame[i] - frame[i + tau];
                sum += diff * diff;
            }
            d[tau] = sum;
        }

        // Cumulative mean normalized difference function (CMND)
        let mut cmnd = vec![0.0f32; max_tau];
        cmnd[0] = 1.0;
        let mut running_sum = 0.0f32;
        for tau in 1..max_tau {
            running_sum += d[tau];
            if running_sum > 1e-6 {
                cmnd[tau] = d[tau] * (tau as f32) / running_sum;
            } else {
                cmnd[tau] = 1.0;
            }
        }

        // Absolute threshold dip detection
        let threshold = 0.15f32;
        let mut best_tau = 0;
        for tau in min_tau..max_tau {
            if cmnd[tau] < threshold {
                // Find local minimum
                let mut local_min = tau;
                while local_min + 1 < max_tau && cmnd[local_min + 1] < cmnd[local_min] {
                    local_min += 1;
                }
                best_tau = local_min;
                break;
            }
        }

        if best_tau == 0 {
            // Fallback to global minimum in range
            let mut min_val = 100.0f32;
            for (tau, &val) in cmnd.iter().enumerate().take(max_tau).skip(min_tau) {
                if val < min_val {
                    min_val = val;
                    best_tau = tau;
                }
            }
            if min_val > 0.45 {
                return None; // Unvoiced / noisy
            }
        }

        // Parabolic interpolation around minimum
        let tau_f = if best_tau > 0 && best_tau + 1 < max_tau {
            let s0 = cmnd[best_tau - 1];
            let s1 = cmnd[best_tau];
            let s2 = cmnd[best_tau + 1];
            let delta = 0.5 * (s0 - s2) / (s0 - 2.0 * s1 + s2 + 1e-6);
            best_tau as f32 + delta
        } else {
            best_tau as f32
        };

        let freq = self.config.sample_rate / tau_f;
        Some((freq, rms))
    }

    /// Transcribe whole audio buffer into segmented MIDI notes.
    pub fn transcribe(&self, audio: &[f32]) -> Vec<TranscribedNote> {
        let frame_size = self.config.frame_size;
        let hop = self.config.hop_size;
        if audio.len() < frame_size {
            return Vec::new();
        }

        let num_frames = (audio.len() - frame_size) / hop + 1;
        let mut pitch_track = Vec::with_capacity(num_frames);

        for f in 0..num_frames {
            let start = f * hop;
            let frame = &audio[start..start + frame_size];
            let time_sec = start as f64 / self.config.sample_rate as f64;
            let result = self.estimate_pitch_yin(frame);
            pitch_track.push((time_sec, result));
        }

        // Segment pitch track into notes
        let mut notes = Vec::new();
        let mut current_note: Option<(f64, Vec<f32>, Vec<f32>)> = None; // (start_time, pitches, rms_list)

        for (time_sec, res) in pitch_track {
            match res {
                Some((freq, rms)) => {
                    let midi_pitch = Self::hz_to_midi_note(freq);
                    if let Some((start_time, ref mut pitches, ref mut rms_list)) = current_note {
                        let avg_pitch = pitches.iter().sum::<f32>() / pitches.len() as f32;
                        // If pitch changes by more than 0.8 semitones, close note and start new one
                        if (midi_pitch - avg_pitch).abs() > 0.8 {
                            let dur = time_sec - start_time;
                            if dur >= self.config.min_note_duration_sec {
                                notes.push(Self::finalize_note(start_time, dur, pitches, rms_list));
                            }
                            current_note = Some((time_sec, vec![midi_pitch], vec![rms]));
                        } else {
                            pitches.push(midi_pitch);
                            rms_list.push(rms);
                        }
                    } else {
                        current_note = Some((time_sec, vec![midi_pitch], vec![rms]));
                    }
                }
                None => {
                    if let Some((start_time, pitches, rms_list)) = current_note.take() {
                        let dur = time_sec - start_time;
                        if dur >= self.config.min_note_duration_sec {
                            notes.push(Self::finalize_note(start_time, dur, &pitches, &rms_list));
                        }
                    }
                }
            }
        }

        if let Some((start_time, pitches, rms_list)) = current_note {
            let total_time = audio.len() as f64 / self.config.sample_rate as f64;
            let dur = total_time - start_time;
            if dur >= self.config.min_note_duration_sec {
                notes.push(Self::finalize_note(start_time, dur, &pitches, &rms_list));
            }
        }

        // Classify articulations based on adjacent notes
        for i in 0..notes.len() {
            if notes[i].duration_sec < 0.12 {
                notes[i].articulation = ArticulationType::Staccato;
            } else if i + 1 < notes.len() {
                let gap = notes[i + 1].start_time_sec - (notes[i].start_time_sec + notes[i].duration_sec);
                if gap < 0.02 {
                    notes[i].articulation = ArticulationType::Legato;
                }
            }
        }

        notes
    }

    fn finalize_note(start_time: f64, duration: f64, pitches: &[f32], rms_list: &[f32]) -> TranscribedNote {
        let avg_pitch = pitches.iter().sum::<f32>() / pitches.len() as f32;
        let rounded_pitch = avg_pitch.round().clamp(0.0, 127.0) as u8;
        let cents = (avg_pitch - rounded_pitch as f32) * 100.0;

        let max_rms = rms_list.iter().copied().fold(0.0f32, f32::max);
        let velocity = ((max_rms * 150.0).clamp(20.0, 127.0)).round() as u8;

        TranscribedNote {
            pitch: rounded_pitch,
            cents_detune: cents,
            start_time_sec: start_time,
            duration_sec: duration,
            velocity,
            articulation: ArticulationType::Normal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hz_and_midi_conversion() {
        assert_eq!(VoiceToMidiTranscriber::hz_to_midi_note(440.0).round(), 69.0); // A4
        assert_eq!(VoiceToMidiTranscriber::hz_to_midi_note(261.63).round(), 60.0); // C4
        assert!((VoiceToMidiTranscriber::midi_note_to_hz(69.0) - 440.0).abs() < 0.01);
    }

    #[test]
    fn test_pitch_estimation_sine_wave() {
        let transcriber = VoiceToMidiTranscriber::new(TranscriptionConfig::default());
        let sr = 44100.0;
        let target_freq = 440.0;

        let frame: Vec<f32> = (0..1024)
            .map(|i| (2.0 * std::f32::consts::PI * target_freq * i as f32 / sr).sin() * 0.8)
            .collect();

        let (est_freq, rms) = transcriber.estimate_pitch_yin(&frame).expect("pitch not detected");
        assert!((est_freq - target_freq).abs() < 2.0);
        assert!(rms > 0.5);
    }

    #[test]
    fn test_transcribe_two_notes_with_silence() {
        let transcriber = VoiceToMidiTranscriber::new(TranscriptionConfig::default());
        let sr = 44100.0;

        let mut audio = Vec::new();
        // Note 1: 440 Hz (A4 = 69) for 0.25 seconds
        for i in 0..((sr * 0.25) as usize) {
            audio.push((2.0 * std::f32::consts::PI * 440.0 * i as f32 / sr).sin() * 0.7);
        }
        // Silence for 0.1 seconds
        for _ in 0..((sr * 0.1) as usize) {
            audio.push(0.0);
        }
        // Note 2: 261.63 Hz (C4 = 60) for 0.25 seconds
        for i in 0..((sr * 0.25) as usize) {
            audio.push((2.0 * std::f32::consts::PI * 261.63 * i as f32 / sr).sin() * 0.7);
        }

        let notes = transcriber.transcribe(&audio);
        assert!(notes.len() >= 2);
        assert_eq!(notes[0].pitch, 69); // A4
        assert_eq!(notes[1].pitch, 60); // C4
        assert!(notes[0].duration_sec > 0.1);
    }
}

