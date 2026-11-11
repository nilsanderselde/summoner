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

//! Disk-Streaming Playback Architecture (Epic C, Task C4).
//!
//! Provides background read-ahead disk streaming for audio clips and long stems,
//! eliminating the need to load multi-gigabyte sessions entirely into system RAM:
//! - Background low-priority I/O thread reads audio ahead of the playback cursor.
//! - Lock-free SPSC ring buffers feed the real-time audio thread.
//! - Zero heap allocation during RT playback (AllocGuard-safe).
//! - Non-blocking seek requests and underflow detection telemetry.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use summoner_core::audio::AudioRingBuffer;

/// Configuration options for the disk streaming engine.
#[derive(Debug, Clone)]
pub struct DiskStreamConfig {
    /// Ring buffer capacity in frames per stream (e.g. 65,536 frames \approx 1.48s at 44.1 kHz).
    pub buffer_capacity_frames: usize,
    /// Chunk size in frames to read per disk read operation.
    pub read_chunk_frames: usize,
    /// Threshold in frames: when buffer drops below this, read-ahead triggers.
    pub low_watermark_frames: usize,
}

impl Default for DiskStreamConfig {
    fn default() -> Self {
        Self {
            buffer_capacity_frames: 65536,
            read_chunk_frames: 8192,
            low_watermark_frames: 32768,
        }
    }
}

/// Internal descriptor of an active stream managed by the I/O thread.
struct ActiveStream {
    id: String,
    path: PathBuf,
    channels: u16,
    _sample_rate: u32,
    data_offset_bytes: u64,
    bits_per_sample: u16,
    total_frames: u64,
    ring_buffer: Arc<AudioRingBuffer>,
    read_cursor_frame: Arc<AtomicU64>,
    seek_target_frame: Arc<AtomicU64>,
    _underflow_count: Arc<AtomicU64>,
    active: Arc<AtomicBool>,
}

/// Client handle for reading from a disk stream on the real-time audio thread.
#[derive(Clone)]
pub struct DiskStreamHandle {
    pub id: String,
    pub channels: u16,
    pub sample_rate: u32,
    pub total_frames: u64,
    ring_buffer: Arc<AudioRingBuffer>,
    read_cursor_frame: Arc<AtomicU64>,
    seek_target_frame: Arc<AtomicU64>,
    underflow_count: Arc<AtomicU64>,
    active: Arc<AtomicBool>,
}

impl DiskStreamHandle {
    /// Number of frames currently buffered in memory and ready for zero-wait RT playback.
    pub fn available_frames(&self) -> usize {
        self.ring_buffer.available_read() / self.channels.max(1) as usize
    }

    /// Read interleaved stereo or mono frames directly from the stream buffer into destination slices.
    /// Returns the number of frames actually transferred.
    /// This method performs ZERO allocations and NEVER blocks.
    pub fn read_frames(
        &self,
        out_l: &mut [f32],
        out_r: &mut [f32],
        requested_frames: usize,
    ) -> usize {
        let ch = self.channels.max(1) as usize;
        let num_frames = requested_frames.min(out_l.len()).min(out_r.len());
        let needed_samples = num_frames * ch;

        let available_samples = self.ring_buffer.available_read();
        let samples_to_read = needed_samples.min(available_samples);
        let frames_to_read = samples_to_read / ch;

        if frames_to_read == 0 && num_frames > 0 {
            self.underflow_count.fetch_add(1, Ordering::Relaxed);
            out_l[..num_frames].fill(0.0);
            out_r[..num_frames].fill(0.0);
            return 0;
        }

        // Temporary stack buffer for popping from the SPSC ring
        let mut temp = [0.0f32; 1024];
        let mut frames_done = 0;

        while frames_done < frames_to_read {
            let chunk_frames = (frames_to_read - frames_done).min(temp.len() / ch);
            let chunk_samples = chunk_frames * ch;
            let popped = self.ring_buffer.pop_slice(&mut temp[..chunk_samples]);
            let actual_chunk_frames = popped / ch;

            for f in 0..actual_chunk_frames {
                let out_idx = frames_done + f;
                out_l[out_idx] = temp[f * ch];
                out_r[out_idx] = if ch > 1 {
                    temp[f * ch + 1]
                } else {
                    temp[f * ch]
                };
            }

            frames_done += actual_chunk_frames;
            if actual_chunk_frames == 0 {
                break;
            }
        }

        // Zero out remainder if buffer ran dry
        if frames_done < num_frames {
            self.underflow_count.fetch_add(1, Ordering::Relaxed);
            out_l[frames_done..num_frames].fill(0.0);
            out_r[frames_done..num_frames].fill(0.0);
        }

        self.read_cursor_frame
            .fetch_add(frames_done as u64, Ordering::Relaxed);
        frames_done
    }

    /// Request a non-blocking seek to a new frame position. The I/O thread will flush and refill ahead.
    pub fn seek_frame(&self, frame: u64) {
        self.seek_target_frame
            .store(frame | (1 << 63), Ordering::Release);
    }

    /// Total count of buffer underruns experienced by this stream.
    pub fn underflow_count(&self) -> u64 {
        self.underflow_count.load(Ordering::Relaxed)
    }

    /// Current playback cursor position in frames.
    pub fn current_frame(&self) -> u64 {
        self.read_cursor_frame.load(Ordering::Relaxed)
    }

    /// Whether this stream is actively registered and streaming.
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }
}

/// The central disk streaming engine managing background read-ahead threads.
pub struct DiskStreamManager {
    config: DiskStreamConfig,
    streams: Arc<Mutex<Vec<ActiveStream>>>,
    stop_signal: Arc<AtomicBool>,
    worker_handle: Option<thread::JoinHandle<()>>,
}

impl DiskStreamManager {
    pub fn new(config: DiskStreamConfig) -> Self {
        let streams = Arc::new(Mutex::new(Vec::new()));
        let stop_signal = Arc::new(AtomicBool::new(false));

        let streams_thread = Arc::clone(&streams);
        let stop_thread = Arc::clone(&stop_signal);
        let cfg = config.clone();

        let worker_handle = thread::Builder::new()
            .name("summoner-disk-stream-io".to_string())
            .spawn(move || {
                Self::io_worker_loop(streams_thread, stop_thread, cfg);
            })
            .ok();

        Self {
            config,
            streams,
            stop_signal,
            worker_handle,
        }
    }

    /// Open an audio file from disk and register it for real-time streaming playback.
    pub fn open_stream<P: AsRef<Path>>(
        &self,
        id: impl Into<String>,
        path: P,
    ) -> Result<DiskStreamHandle, String> {
        let path_ref = path.as_ref();
        if !path_ref.exists() {
            return Err(format!("File '{}' does not exist", path_ref.display()));
        }

        // Parse basic WAV/PCM header metadata to locate data chunk
        let (channels, sample_rate, bits_per_sample, data_offset, total_frames) =
            Self::probe_audio_header(path_ref)?;

        let ring_capacity_samples = self.config.buffer_capacity_frames * channels as usize;
        let ring_buffer = Arc::new(AudioRingBuffer::new(ring_capacity_samples));
        let read_cursor = Arc::new(AtomicU64::new(0));
        let seek_target = Arc::new(AtomicU64::new(0));
        let underflow_count = Arc::new(AtomicU64::new(0));
        let active = Arc::new(AtomicBool::new(true));

        let id_str = id.into();

        let active_stream = ActiveStream {
            id: id_str.clone(),
            path: path_ref.to_path_buf(),
            channels,
            _sample_rate: sample_rate,
            data_offset_bytes: data_offset,
            bits_per_sample,
            total_frames,
            ring_buffer: Arc::clone(&ring_buffer),
            read_cursor_frame: Arc::clone(&read_cursor),
            seek_target_frame: Arc::clone(&seek_target),
            _underflow_count: Arc::clone(&underflow_count),
            active: Arc::clone(&active),
        };

        if let Ok(mut lock) = self.streams.lock() {
            lock.push(active_stream);
        }

        Ok(DiskStreamHandle {
            id: id_str,
            channels,
            sample_rate,
            total_frames,
            ring_buffer,
            read_cursor_frame: read_cursor,
            seek_target_frame: seek_target,
            underflow_count,
            active,
        })
    }

    /// Close and unregister an active stream.
    pub fn close_stream(&self, id: &str) {
        if let Ok(mut lock) = self.streams.lock() {
            if let Some(pos) = lock.iter().position(|s| s.id == id) {
                lock[pos].active.store(false, Ordering::Release);
                lock.remove(pos);
            }
        }
    }

    /// Stop the disk streaming manager and wait for the I/O thread to terminate.
    pub fn stop(&mut self) {
        self.stop_signal.store(true, Ordering::Release);
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }

    fn probe_audio_header(path: &Path) -> Result<(u16, u32, u16, u64, u64), String> {
        let mut file = File::open(path).map_err(|e| e.to_string())?;
        let mut header = [0u8; 12];
        file.read_exact(&mut header).map_err(|e| e.to_string())?;

        if &header[0..4] != b"RIFF" || &header[8..12] != b"WAVE" {
            // Default fallback assumption for raw/flat PCM
            let len = file.metadata().map_err(|e| e.to_string())?.len();
            return Ok((2, 44100, 16, 0, len / 4));
        }

        let mut channels = 2u16;
        let mut sample_rate = 44100u32;
        let mut bits = 16u16;
        let mut data_offset = 44u64;
        let mut data_len = 0u64;

        // Iterate subchunks
        let mut chunk_header = [0u8; 8];
        while file.read_exact(&mut chunk_header).is_ok() {
            let chunk_id = &chunk_header[0..4];
            let chunk_size = u32::from_le_bytes([
                chunk_header[4],
                chunk_header[5],
                chunk_header[6],
                chunk_header[7],
            ]) as u64;

            if chunk_id == b"fmt " {
                let mut fmt_data = vec![0u8; chunk_size as usize];
                file.read_exact(&mut fmt_data).map_err(|e| e.to_string())?;
                if fmt_data.len() >= 16 {
                    channels = u16::from_le_bytes([fmt_data[2], fmt_data[3]]);
                    sample_rate =
                        u32::from_le_bytes([fmt_data[4], fmt_data[5], fmt_data[6], fmt_data[7]]);
                    bits = u16::from_le_bytes([fmt_data[14], fmt_data[15]]);
                }
            } else if chunk_id == b"data" {
                data_offset = file.stream_position().map_err(|e| e.to_string())?;
                data_len = chunk_size;
                break;
            } else {
                let _ = file.seek(SeekFrom::Current(chunk_size as i64));
            }
        }

        let bytes_per_frame = (channels as u64 * bits as u64) / 8;
        let total_frames = data_len.checked_div(bytes_per_frame).unwrap_or(0);

        Ok((channels, sample_rate, bits, data_offset, total_frames))
    }

    fn io_worker_loop(
        streams: Arc<Mutex<Vec<ActiveStream>>>,
        stop_signal: Arc<AtomicBool>,
        config: DiskStreamConfig,
    ) {
        let mut read_buffer = vec![0.0f32; config.read_chunk_frames * 2];
        let mut byte_buffer = vec![0u8; config.read_chunk_frames * 8];

        while !stop_signal.load(Ordering::Acquire) {
            let mut streams_snapshot = Vec::new();
            if let Ok(lock) = streams.lock() {
                for s in lock.iter() {
                    if s.active.load(Ordering::Relaxed) {
                        streams_snapshot.push((
                            s.path.clone(),
                            s.channels,
                            s.data_offset_bytes,
                            s.bits_per_sample,
                            s.total_frames,
                            Arc::clone(&s.ring_buffer),
                            Arc::clone(&s.read_cursor_frame),
                            Arc::clone(&s.seek_target_frame),
                        ));
                    }
                }
            }

            if streams_snapshot.is_empty() {
                thread::sleep(Duration::from_millis(10));
                continue;
            }

            for (path, ch, offset, bits, total_frames, ring, cursor, seek_target) in
                streams_snapshot
            {
                // Check if seek was requested
                let target_raw = seek_target.load(Ordering::Acquire);
                let is_seek = (target_raw & (1 << 63)) != 0;
                let target_frame = target_raw & !(1 << 63);

                if is_seek {
                    // Drain ring buffer
                    let mut drain = [0.0f32; 1024];
                    while ring.pop_slice(&mut drain) > 0 {}
                    cursor.store(target_frame, Ordering::Release);
                    seek_target.store(0, Ordering::Release);
                }

                let ch_usize = ch.max(1) as usize;
                let avail_frames = ring.available_read() / ch_usize;

                if avail_frames < config.low_watermark_frames {
                    let frames_to_read = config.read_chunk_frames;
                    let current_cursor = cursor.load(Ordering::Acquire);
                    let buffered_frames = ring.available_read() / ch_usize;
                    let file_frame = current_cursor + buffered_frames as u64;

                    if file_frame >= total_frames {
                        continue;
                    }

                    let bytes_per_frame = (ch as u64 * bits as u64) / 8;
                    let seek_bytes = offset + (file_frame * bytes_per_frame);

                    if let Ok(mut f) = File::open(&path) {
                        if f.seek(SeekFrom::Start(seek_bytes)).is_ok() {
                            let needed_bytes = (frames_to_read as u64 * bytes_per_frame) as usize;
                            let to_read_bytes = needed_bytes.min(byte_buffer.len());

                            if let Ok(n_read) = f.read(&mut byte_buffer[..to_read_bytes]) {
                                let frames_read =
                                    (n_read as u64).checked_div(bytes_per_frame).unwrap_or(0)
                                        as usize;

                                if frames_read > 0 {
                                    // Convert raw PCM to normalized float
                                    if bits == 16 {
                                        let sample_count = frames_read * ch_usize;
                                        for (i, slot) in
                                            read_buffer.iter_mut().enumerate().take(sample_count)
                                        {
                                            let b_idx = i * 2;
                                            if b_idx + 1 < n_read {
                                                let val = i16::from_le_bytes([
                                                    byte_buffer[b_idx],
                                                    byte_buffer[b_idx + 1],
                                                ]);
                                                *slot = val as f32 / i16::MAX as f32;
                                            }
                                        }
                                        let _ = ring.push_slice(&read_buffer[..sample_count]);
                                    } else if bits == 32 {
                                        let sample_count = frames_read * ch_usize;
                                        for (i, slot) in
                                            read_buffer.iter_mut().enumerate().take(sample_count)
                                        {
                                            let b_idx = i * 4;
                                            if b_idx + 3 < n_read {
                                                let val = f32::from_le_bytes([
                                                    byte_buffer[b_idx],
                                                    byte_buffer[b_idx + 1],
                                                    byte_buffer[b_idx + 2],
                                                    byte_buffer[b_idx + 3],
                                                ]);
                                                *slot = val;
                                            }
                                        }
                                        let _ = ring.push_slice(&read_buffer[..sample_count]);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            thread::sleep(Duration::from_millis(5));
        }
    }
}

impl Drop for DiskStreamManager {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hound::{SampleFormat, WavSpec, WavWriter};
    use std::fs;

    #[test]
    fn test_disk_stream_read_ahead_and_rt_playback() {
        let temp_dir = std::env::temp_dir().join("summoner_test_disk_stream");
        let _ = fs::create_dir_all(&temp_dir);
        let wav_path = temp_dir.join("stream_test.wav");

        // Write 44,100 frames (1 second) of 16-bit stereo WAV
        let spec = WavSpec {
            channels: 2,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
        let mut writer = WavWriter::create(&wav_path, spec).unwrap();
        for i in 0..44100 {
            let s = ((i as f32 * 440.0 * std::f32::consts::TAU / 44100.0).sin() * 20000.0) as i16;
            writer.write_sample(s).unwrap();
            writer.write_sample(s).unwrap();
        }
        writer.finalize().unwrap();

        let mut config = DiskStreamConfig::default();
        config.buffer_capacity_frames = 16384;
        config.read_chunk_frames = 4096;
        config.low_watermark_frames = 8192;

        let mut manager = DiskStreamManager::new(config);
        let handle = manager.open_stream("test_stream", &wav_path).unwrap();

        assert_eq!(handle.channels, 2);
        assert_eq!(handle.sample_rate, 44100);
        assert_eq!(handle.total_frames, 44100);

        // Allow background thread to read ahead initial chunk
        thread::sleep(Duration::from_millis(50));
        assert!(
            handle.available_frames() > 0,
            "Background I/O thread should buffer initial chunk"
        );

        // Read frames simulating real-time audio thread callback
        let mut out_l = [0.0f32; 512];
        let mut out_r = [0.0f32; 512];
        let read = handle.read_frames(&mut out_l, &mut out_r, 512);
        assert_eq!(read, 512);
        assert!(out_l.iter().any(|&s| s.abs() > 0.01));
        assert_eq!(out_l, out_r);

        // Test seeking
        handle.seek_frame(22050);
        thread::sleep(Duration::from_millis(50));
        let read_after_seek = handle.read_frames(&mut out_l, &mut out_r, 256);
        assert_eq!(read_after_seek, 256);

        manager.stop();
        let _ = fs::remove_file(wav_path);
        let _ = fs::remove_dir(temp_dir);
    }
}
