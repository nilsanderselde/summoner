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

//! Streaming Sampler (Direct-From-Disk / DFD) engine for gigabyte-scale sample libraries.
//!
//! Maintains a lightweight in-RAM pre-buffer (e.g. 2048-4096 frames) for instant zero-latency
//! note attack, while streaming remaining sample data asynchronously from disk in the background.

use crate::sampler::LoopMode;
use crate::traits::SignalProcessor;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;

/// Default pre-buffer frame size kept in RAM (e.g., ~90ms at 44.1 kHz).
pub const DEFAULT_PREBUFFER_FRAMES: usize = 4096;

/// Maximum simultaneous streaming voices.
pub const MAX_STREAMING_VOICES: usize = 32;

/// A mapped region in a streaming sample bank.
#[derive(Debug, Clone)]
pub struct StreamingSampleRegion {
    pub sample_path: PathBuf,
    pub lokey: u8,
    pub hikey: u8,
    pub pitch_keycenter: u8,
    pub lovel: u8,
    pub hivel: u8,
    pub loop_mode: LoopMode,
    pub loop_start: usize,
    pub loop_end: usize,
    /// Fast in-memory pre-buffer for zero-latency note-on response.
    pub prebuffer: Vec<Sample>,
    pub prebuffer_frames: usize,
    pub total_frames: usize,
    pub sample_rate: u32,
    pub channels: usize,
}

impl StreamingSampleRegion {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sample_path: impl Into<PathBuf>,
        lokey: u8,
        hikey: u8,
        pitch_keycenter: u8,
        lovel: u8,
        hivel: u8,
        total_frames: usize,
        sample_rate: u32,
        channels: usize,
        prebuffer: Vec<Sample>,
    ) -> Self {
        let prebuffer_frames = prebuffer.len().checked_div(channels).unwrap_or(0);
        Self {
            sample_path: sample_path.into(),
            lokey,
            hikey,
            pitch_keycenter,
            lovel,
            hivel,
            loop_mode: LoopMode::NoLoop,
            loop_start: 0,
            loop_end: 0,
            prebuffer,
            prebuffer_frames,
            total_frames,
            sample_rate,
            channels,
        }
    }

    pub fn matches(&self, note: u8, velocity: u8) -> bool {
        note >= self.lokey && note <= self.hikey && velocity >= self.lovel && velocity <= self.hivel
    }
}

/// Request sent to the background disk-streaming worker.
#[derive(Debug)]
pub struct DiskStreamRequest {
    pub voice_id: usize,
    pub path: PathBuf,
    pub start_frame: usize,
    pub num_frames: usize,
    pub channels: usize,
}

/// Chunk response received from the background disk-streaming worker.
#[derive(Debug)]
pub struct DiskStreamResponse {
    pub voice_id: usize,
    pub start_frame: usize,
    pub samples: Vec<Sample>,
    pub is_eof: bool,
}

/// Internal voice state for streaming playback.
#[derive(Debug, Clone)]
pub struct StreamingVoice {
    pub voice_id: usize,
    pub active: bool,
    pub note: u8,
    pub velocity: f32,
    pub region_idx: usize,
    pub playback_pos: f64,
    pub playback_rate: f64,
    pub requested_disk_pos: usize,
    pub stream_buffer: Vec<Sample>,
    pub stream_start_frame: usize,
    pub is_eof: bool,
}

impl StreamingVoice {
    pub fn new(voice_id: usize) -> Self {
        Self {
            voice_id,
            active: false,
            note: 0,
            velocity: 1.0,
            region_idx: 0,
            playback_pos: 0.0,
            playback_rate: 1.0,
            requested_disk_pos: 0,
            stream_buffer: Vec::with_capacity(32768),
            stream_start_frame: 0,
            is_eof: false,
        }
    }

    pub fn reset(&mut self) {
        self.active = false;
        self.playback_pos = 0.0;
        self.stream_buffer.clear();
        self.stream_start_frame = 0;
        self.requested_disk_pos = 0;
        self.is_eof = false;
    }
}

/// Bank of streaming sample regions.
#[derive(Debug, Clone, Default)]
pub struct StreamingSampleBank {
    pub regions: Vec<StreamingSampleRegion>,
}

impl StreamingSampleBank {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_region(&mut self, region: StreamingSampleRegion) {
        self.regions.push(region);
    }

    pub fn find_region(&self, note: u8, velocity: u8) -> Option<(usize, &StreamingSampleRegion)> {
        self.regions
            .iter()
            .enumerate()
            .find(|(_, r)| r.matches(note, velocity))
    }
}

/// A Direct-From-Disk Streaming Sampler DSP Node.
pub struct StreamingSamplerNode {
    pub bank: Arc<StreamingSampleBank>,
    voices: Vec<StreamingVoice>,
    req_tx: Sender<DiskStreamRequest>,
    resp_rx: Receiver<DiskStreamResponse>,
    _worker_thread: Option<std::thread::JoinHandle<()>>,
}

impl StreamingSamplerNode {
    /// Creates a new `StreamingSamplerNode` with a dedicated background disk stream worker.
    pub fn new(bank: Arc<StreamingSampleBank>) -> Self {
        let (req_tx, req_rx) = channel::<DiskStreamRequest>();
        let (resp_tx, resp_rx) = channel::<DiskStreamResponse>();

        // Spawn background disk I/O worker
        let worker_thread = std::thread::Builder::new()
            .name("summoner-stream-sampler-io".to_string())
            .spawn(move || {
                while let Ok(req) = req_rx.recv() {
                    let (samples, is_eof) =
                        read_disk_frames(&req.path, req.start_frame, req.num_frames, req.channels);
                    let _ = resp_tx.send(DiskStreamResponse {
                        voice_id: req.voice_id,
                        start_frame: req.start_frame,
                        samples,
                        is_eof,
                    });
                }
            })
            .ok();

        let voices = (0..MAX_STREAMING_VOICES).map(StreamingVoice::new).collect();

        Self {
            bank,
            voices,
            req_tx,
            resp_rx,
            _worker_thread: worker_thread,
        }
    }

    /// Triggers a note-on event.
    pub fn trigger_note(&mut self, note: u8, velocity: u8) {
        if let Some((region_idx, region)) = self.bank.find_region(note, velocity) {
            // Allocate a voice (idle first, else oldest)
            let voice_id = self.voices.iter().position(|v| !v.active).unwrap_or(0);

            let semitone_diff = note as f64 - region.pitch_keycenter as f64;
            let playback_rate = 2.0f64.powf(semitone_diff / 12.0);

            let v = &mut self.voices[voice_id];
            v.active = true;
            v.note = note;
            v.velocity = velocity as f32 / 127.0;
            v.region_idx = region_idx;
            v.playback_pos = 0.0;
            v.playback_rate = playback_rate;
            v.stream_buffer.clear();
            v.stream_start_frame = region.prebuffer_frames;
            v.requested_disk_pos = region.prebuffer_frames;
            v.is_eof = false;

            // Immediately request initial streaming chunk from disk
            if region.total_frames > region.prebuffer_frames {
                let to_read = (region.total_frames - region.prebuffer_frames).min(8192);
                let _ = self.req_tx.send(DiskStreamRequest {
                    voice_id,
                    path: region.sample_path.clone(),
                    start_frame: region.prebuffer_frames,
                    num_frames: to_read,
                    channels: region.channels,
                });
                v.requested_disk_pos += to_read;
            }
        }
    }

    /// Stops any voice playing the given note.
    pub fn stop_note(&mut self, note: u8) {
        for v in &mut self.voices {
            if v.active && v.note == note {
                v.reset();
            }
        }
    }

    /// Process responses from disk streaming thread non-blockingly.
    fn poll_stream_responses(&mut self) {
        while let Ok(resp) = self.resp_rx.try_recv() {
            if resp.voice_id < self.voices.len() {
                let voice = &mut self.voices[resp.voice_id];
                if voice.active {
                    voice.stream_buffer.extend_from_slice(&resp.samples);
                    voice.is_eof = resp.is_eof;
                }
            }
        }
    }
}

impl SignalProcessor for StreamingSamplerNode {
    fn name(&self) -> &str {
        "StreamingSamplerNode"
    }

    fn handle_event(&mut self, event: &summoner_core::node::NodeEvent, _ctx: &ProcessContext) {
        use summoner_core::node::NodeEvent;
        match *event {
            NodeEvent::NoteOn { note, velocity } => {
                let vel_u8 = (velocity.clamp(0.0, 1.0) * 127.0).round() as u8;
                self.trigger_note(note, vel_u8.max(1));
            }
            NodeEvent::NoteOff { note, .. } => {
                self.stop_note(note);
            }
            _ => {}
        }
    }

    #[allow(clippy::needless_range_loop)]
    fn process_block(
        &mut self,
        _inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        _ctx: &ProcessContext,
    ) {
        self.poll_stream_responses();

        if outputs.is_empty() {
            return;
        }

        for out in outputs.iter_mut() {
            out.fill(0.0);
        }

        let num_frames = outputs[0].len();
        let num_channels = outputs.len();

        for voice in &mut self.voices {
            if !voice.active {
                continue;
            }

            let region = match self.bank.regions.get(voice.region_idx) {
                Some(r) => r,
                None => {
                    voice.active = false;
                    continue;
                }
            };

            let channels = region.channels.min(num_channels);
            if channels == 0 {
                continue;
            }

            for i in 0..num_frames {
                let current_frame = voice.playback_pos.floor() as usize;
                let frac = (voice.playback_pos - current_frame as f64) as f32;

                if current_frame >= region.total_frames {
                    voice.reset();
                    break;
                }

                // Sample reading: from prebuffer or streamed buffer
                let (s0_l, s1_l, s0_r, s1_r) = if current_frame < region.prebuffer_frames {
                    // Prebuffer playback (in RAM)
                    let p_pos = current_frame * region.channels;
                    let p_next =
                        ((current_frame + 1).min(region.prebuffer_frames - 1)) * region.channels;

                    let s0_l = region.prebuffer.get(p_pos).copied().unwrap_or(0.0);
                    let s1_l = region.prebuffer.get(p_next).copied().unwrap_or(s0_l);

                    let (s0_r, s1_r) = if region.channels > 1 {
                        (
                            region.prebuffer.get(p_pos + 1).copied().unwrap_or(s0_l),
                            region.prebuffer.get(p_next + 1).copied().unwrap_or(s1_l),
                        )
                    } else {
                        (s0_l, s1_l)
                    };
                    (s0_l, s1_l, s0_r, s1_r)
                } else {
                    // Stream buffer playback (streamed from disk)
                    let rel_frame = current_frame.saturating_sub(voice.stream_start_frame);
                    let rel_next = rel_frame + 1;

                    let b_pos = rel_frame * region.channels;
                    let b_next = rel_next * region.channels;

                    let s0_l = voice.stream_buffer.get(b_pos).copied().unwrap_or(0.0);
                    let s1_l = voice.stream_buffer.get(b_next).copied().unwrap_or(s0_l);

                    let (s0_r, s1_r) = if region.channels > 1 {
                        (
                            voice.stream_buffer.get(b_pos + 1).copied().unwrap_or(s0_l),
                            voice.stream_buffer.get(b_next + 1).copied().unwrap_or(s1_l),
                        )
                    } else {
                        (s0_l, s1_l)
                    };
                    (s0_l, s1_l, s0_r, s1_r)
                };

                let sample_l = (s0_l + frac * (s1_l - s0_l)) * voice.velocity;
                let sample_r = (s0_r + frac * (s1_r - s0_r)) * voice.velocity;

                outputs[0][i] += sample_l;
                if num_channels > 1 {
                    outputs[1][i] += sample_r;
                }

                voice.playback_pos += voice.playback_rate;

                // Check if we need to request more streaming data ahead of playback
                let ahead_frames = voice.playback_pos.floor() as usize + 4096;
                if ahead_frames > voice.requested_disk_pos
                    && voice.requested_disk_pos < region.total_frames
                    && !voice.is_eof
                {
                    let chunk_size = (region.total_frames - voice.requested_disk_pos).min(8192);
                    let _ = self.req_tx.send(DiskStreamRequest {
                        voice_id: voice.voice_id,
                        path: region.sample_path.clone(),
                        start_frame: voice.requested_disk_pos,
                        num_frames: chunk_size,
                        channels: region.channels,
                    });
                    voice.requested_disk_pos += chunk_size;
                }
            }
        }
    }
}

/// Helper function to read PCM frames from a disk file.
fn read_disk_frames(
    path: &Path,
    start_frame: usize,
    num_frames: usize,
    channels: usize,
) -> (Vec<Sample>, bool) {
    if !path.exists() {
        return (vec![0.0; num_frames * channels], true);
    }

    if let Ok(mut reader) = hound::WavReader::open(path) {
        let spec = reader.spec();
        let ch = spec.channels as usize;
        let mut samples = Vec::with_capacity(num_frames * ch);

        let start_sample = start_frame * ch;
        let samples_to_read = num_frames * ch;

        let scale = 1.0 / (1i64 << (spec.bits_per_sample - 1)) as f32;
        let mut idx = 0;
        let mut is_eof = true;

        if spec.sample_format == hound::SampleFormat::Float {
            for s in reader.samples::<f32>() {
                if idx >= start_sample && idx < start_sample + samples_to_read {
                    if let Ok(val) = s {
                        samples.push(val);
                    }
                }
                idx += 1;
                if idx >= start_sample + samples_to_read {
                    is_eof = false;
                    break;
                }
            }
        } else {
            for s in reader.samples::<i32>() {
                if idx >= start_sample && idx < start_sample + samples_to_read {
                    if let Ok(val) = s {
                        samples.push(val as f32 * scale);
                    }
                }
                idx += 1;
                if idx >= start_sample + samples_to_read {
                    is_eof = false;
                    break;
                }
            }
        }

        (samples, is_eof)
    } else {
        (vec![0.0; num_frames * channels], true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_sampler_prebuffer_playback() {
        // Create an in-memory sample region with prebuffer
        let total_frames = 10000;
        let prebuffer_frames = 2048;
        let mut prebuffer = Vec::with_capacity(prebuffer_frames);
        for i in 0..prebuffer_frames {
            prebuffer.push((i as f32 * 0.05).sin());
        }

        let region = StreamingSampleRegion::new(
            "nonexistent_virtual_piano.wav",
            0,
            127,
            60,
            0,
            127,
            total_frames,
            44100,
            1,
            prebuffer,
        );

        let mut bank = StreamingSampleBank::new();
        bank.add_region(region);

        let mut node = StreamingSamplerNode::new(Arc::new(bank));
        node.trigger_note(60, 100);

        let mut left = vec![0.0f32; 512];
        let mut right = vec![0.0f32; 512];
        let mut outputs = [&mut left[..], &mut right[..]];
        let inputs: [&[Sample]; 0] = [];
        let ctx = ProcessContext::new(44100, 120.0, 0);

        node.process_block(&inputs, &mut outputs, &ctx);

        // Verify audio was generated immediately from the prebuffer with zero latency
        let energy: f32 = left.iter().map(|s| s * s).sum();
        assert!(
            energy > 0.1,
            "Expected immediate audio energy from prebuffer"
        );
    }
}
