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

//! Multi-format audio decoding and import engine (Epic G, Task G8).
//!
//! Provides comprehensive decoding for:
//! - MP3 (MPEG Audio Layer III)
//! - AAC / M4A (Advanced Audio Coding in ISO/MP4 container)
//! - OGG Vorbis
//! - AIFF / AIFC
//! - WAV (PCM 16-bit, 24-bit, 32-bit int, 32-bit and 64-bit float)
//! - FLAC (Free Lossless Audio Codec)
//!
//! Backed by pure-Rust `symphonia` with zero external C dependencies, ensuring identical
//! cross-platform decoding across Windows, macOS, and Linux.

use std::fs;
use std::path::Path;
use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// Decoded audio container holding 32-bit floating point audio samples normalized to [-1.0, 1.0].
#[derive(Debug, Clone, PartialEq)]
pub struct DecodedAudio {
    /// Interleaved audio samples in the range `[-1.0, 1.0]`.
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
    pub format_name: String,
    pub total_frames: u64,
}

impl DecodedAudio {
    /// Total duration of the audio in seconds.
    pub fn duration_seconds(&self) -> f64 {
        if self.sample_rate == 0 {
            0.0
        } else {
            self.total_frames as f64 / self.sample_rate as f64
        }
    }

    /// Extract de-interleaved samples for a specific channel (0-indexed).
    pub fn channel_samples(&self, channel_index: u16) -> Vec<f32> {
        let ch = self.channels.max(1) as usize;
        let c = channel_index as usize;
        if c >= ch {
            return Vec::new();
        }

        let mut out = Vec::with_capacity(self.total_frames as usize);
        for frame in 0..(self.samples.len() / ch) {
            out.push(self.samples[frame * ch + c]);
        }
        out
    }

    /// Convert stereo or multi-channel audio to mono by averaging all channels.
    pub fn to_mono(&self) -> Vec<f32> {
        let ch = self.channels.max(1) as usize;
        if ch == 1 {
            return self.samples.clone();
        }

        let frame_count = self.samples.len() / ch;
        let mut mono = Vec::with_capacity(frame_count);
        let scale = 1.0 / ch as f32;

        for f in 0..frame_count {
            let mut sum = 0.0f32;
            for c in 0..ch {
                sum += self.samples[f * ch + c];
            }
            mono.push(sum * scale);
        }
        mono
    }
}

/// Decode any supported audio file from disk into a `DecodedAudio` structure.
pub fn decode_audio_file<P: AsRef<Path>>(path: P) -> Result<DecodedAudio, String> {
    let path_ref = path.as_ref();
    let file = fs::File::open(path_ref)
        .map_err(|e| format!("Failed to open audio file '{}': {}", path_ref.display(), e))?;

    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path_ref.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &fmt_opts, &meta_opts)
        .map_err(|e| {
            format!(
                "Failed to probe audio format of '{}': {}",
                path_ref.display(),
                e
            )
        })?;

    let mut format = probed.format;

    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| format!("No supported audio track found in '{}'", path_ref.display()))?;

    let track_id = track.id;
    let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
    let channels = track
        .codec_params
        .channels
        .map(|c| c.count() as u16)
        .unwrap_or(2);
    let codec_name = format!("{:?}", track.codec_params.codec);

    let dec_opts: DecoderOptions = Default::default();
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &dec_opts)
        .map_err(|e| format!("Cannot create decoder for '{}': {}", path_ref.display(), e))?;

    let mut samples: Vec<f32> = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(Error::ResetRequired) => continue,
            Err(Error::IoError(ref err)) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
                break
            }
            Err(Error::IoError(_)) => break,
            Err(err) => return Err(format!("Error reading packet: {}", err)),
        };

        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(decoded) => match decoded {
                AudioBufferRef::F32(buf) => {
                    for frame in 0..buf.frames() {
                        for ch in 0..buf.spec().channels.count() {
                            samples.push(buf.chan(ch)[frame]);
                        }
                    }
                }
                AudioBufferRef::U8(buf) => {
                    for frame in 0..buf.frames() {
                        for ch in 0..buf.spec().channels.count() {
                            samples.push((buf.chan(ch)[frame] as f32 - 128.0) / 128.0);
                        }
                    }
                }
                AudioBufferRef::S16(buf) => {
                    for frame in 0..buf.frames() {
                        for ch in 0..buf.spec().channels.count() {
                            samples.push(buf.chan(ch)[frame] as f32 / i16::MAX as f32);
                        }
                    }
                }
                AudioBufferRef::S24(buf) => {
                    for frame in 0..buf.frames() {
                        for ch in 0..buf.spec().channels.count() {
                            samples.push(buf.chan(ch)[frame].0 as f32 / 8_388_607.0);
                        }
                    }
                }
                AudioBufferRef::S32(buf) => {
                    for frame in 0..buf.frames() {
                        for ch in 0..buf.spec().channels.count() {
                            samples.push(buf.chan(ch)[frame] as f32 / i32::MAX as f32);
                        }
                    }
                }
                AudioBufferRef::F64(buf) => {
                    for frame in 0..buf.frames() {
                        for ch in 0..buf.spec().channels.count() {
                            samples.push(buf.chan(ch)[frame] as f32);
                        }
                    }
                }
                _ => {}
            },
            Err(Error::IoError(ref err)) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
                break
            }
            Err(Error::DecodeError(_)) => continue,
            Err(err) => return Err(format!("Decode error: {}", err)),
        }
    }

    let ch_count = channels.max(1) as usize;
    let total_frames = (samples.len() / ch_count) as u64;

    Ok(DecodedAudio {
        samples,
        sample_rate,
        channels,
        format_name: codec_name,
        total_frames,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use hound::{SampleFormat, WavSpec, WavWriter};

    #[test]
    fn test_decode_wav_24bit_pcm() {
        let temp_dir = std::env::temp_dir().join("summoner_test_wav24");
        let _ = fs::create_dir_all(&temp_dir);
        let wav_path = temp_dir.join("test24.wav");

        let spec = WavSpec {
            channels: 2,
            sample_rate: 48000,
            bits_per_sample: 24,
            sample_format: SampleFormat::Int,
        };
        let mut writer = WavWriter::create(&wav_path, spec).unwrap();
        // Write 480 frames of 1 kHz sine wave
        for i in 0..480 {
            let s =
                ((i as f32 * 1000.0 * std::f32::consts::TAU / 48000.0).sin() * 8_000_000.0) as i32;
            writer.write_sample(s).unwrap();
            writer.write_sample(s).unwrap();
        }
        writer.finalize().unwrap();

        let decoded = decode_audio_file(&wav_path).expect("Should decode 24-bit WAV");
        assert_eq!(decoded.sample_rate, 48000);
        assert_eq!(decoded.channels, 2);
        assert_eq!(decoded.total_frames, 480);
        assert_eq!(decoded.samples.len(), 960);
        assert!((decoded.duration_seconds() - 0.01).abs() < 1e-4);

        let left = decoded.channel_samples(0);
        assert_eq!(left.len(), 480);
        let mono = decoded.to_mono();
        assert_eq!(mono.len(), 480);

        let _ = fs::remove_file(wav_path);
        let _ = fs::remove_dir(temp_dir);
    }

    #[test]
    fn test_decode_wav_32bit_float() {
        let temp_dir = std::env::temp_dir().join("summoner_test_wav32f");
        let _ = fs::create_dir_all(&temp_dir);
        let wav_path = temp_dir.join("test32f.wav");

        let spec = WavSpec {
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 32,
            sample_format: SampleFormat::Float,
        };
        let mut writer = WavWriter::create(&wav_path, spec).unwrap();
        for i in 0..441 {
            let s = (i as f32 * 440.0 * std::f32::consts::TAU / 44100.0).sin() * 0.75;
            writer.write_sample(s).unwrap();
        }
        writer.finalize().unwrap();

        let decoded = decode_audio_file(&wav_path).expect("Should decode 32-bit float WAV");
        assert_eq!(decoded.sample_rate, 44100);
        assert_eq!(decoded.channels, 1);
        assert_eq!(decoded.total_frames, 441);
        assert_eq!(decoded.samples.len(), 441);

        let _ = fs::remove_file(wav_path);
        let _ = fs::remove_dir(temp_dir);
    }
}
