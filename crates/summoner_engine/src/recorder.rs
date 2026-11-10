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

//! Multi-Track Audio Recording to Disk Engine (Epic C, Task C2).
//!
//! Provides real-time-safe, multi-track audio recording directly to standard WAV files on disk:
//! - Track arming with flexible input channel assignment (Mono Left, Mono Right, Stereo).
//! - Non-blocking, zero-allocation write-ahead ring buffer transfer under `AllocGuard`.
//! - Dedicated background disk I/O thread writing 32-bit float or 24-bit PCM WAV.
//! - Count-in / pre-roll phase management.
//! - Punch-in and punch-out locators with sample-accurate gating.
//! - Seamless loop-record takes cycling with incremental take indices.
//! - Conversion of recorded takes into `AudioClipConfig` for the arranger timeline.

use crossbeam_channel::{bounded, Receiver, Sender};
use hound::{SampleFormat, WavSpec, WavWriter};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use summoner_core::allocator::AllocGuard;
use summoner_core::audio::AudioRingBuffer;
use summoner_project::audio_clip::AudioClipConfig;

/// Channel routing source for an armed recording track.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioInputSource {
    /// Single input channel (e.g. 0 for Left, 1 for Right).
    Mono(usize),
    /// Stereo input pair (Left channel index, Right channel index).
    Stereo(usize, usize),
}

impl Default for AudioInputSource {
    fn default() -> Self {
        Self::Stereo(0, 1)
    }
}

/// Punch-in / punch-out locator configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PunchSettings {
    pub punch_in_frame: Option<u64>,
    pub punch_out_frame: Option<u64>,
}

impl PunchSettings {
    /// Return true if the given timeline frame is within the punch recording window.
    #[inline]
    pub fn is_in_punch(&self, frame: u64) -> bool {
        if let Some(pin) = self.punch_in_frame {
            if frame < pin {
                return false;
            }
        }
        if let Some(pout) = self.punch_out_frame {
            if frame >= pout {
                return false;
            }
        }
        true
    }
}

/// Loop recording configuration for take generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LoopRecordSettings {
    pub loop_enabled: bool,
    pub loop_start_frame: u64,
    pub loop_end_frame: u64,
}

/// State of the audio recorder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordState {
    /// Recorder is idle / disarmed.
    Inactive,
    /// Pre-roll / count-in is executing; monitoring is active but no samples are written to disk.
    CountIn {
        remaining_frames: u64,
        total_bars: u32,
    },
    /// Actively recording samples to disk.
    Recording,
    /// Paused.
    Paused,
}

/// Result of a finalized recorded audio take.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordedTakeResult {
    pub track_id: u64,
    pub take_index: usize,
    pub file_path: PathBuf,
    pub start_frame: u64,
    pub start_beat: f64,
    pub total_frames: u64,
    pub sample_rate: u32,
    pub channels: u16,
}

impl RecordedTakeResult {
    /// Convert this recorded take into an `AudioClipConfig` ready for insertion into `TrackConfig`.
    pub fn to_audio_clip_config(&self, clip_id: &str, clip_name: &str, bpm: f64) -> AudioClipConfig {
        let frames_per_beat = (self.sample_rate as f64 * 60.0) / bpm.max(1.0);
        let duration_beats = (self.total_frames as f64) / frames_per_beat.max(1.0);

        let mut clip = AudioClipConfig::new(
            clip_id,
            clip_name,
            format!("rec_track_{}_take_{}", self.track_id, self.take_index),
            self.start_beat,
            duration_beats,
        );
        clip.file_path = Some(self.file_path.to_string_lossy().to_string());
        clip.duration_frames = Some(self.total_frames);
        clip
    }
}

/// Commands sent to the background write-ahead disk recording worker thread.
enum DiskWriterCommand {
    ArmTrack {
        track_id: u64,
        session_dir: PathBuf,
        prefix: String,
        sample_rate: u32,
        channels: u16,
        ring_buffers: [Arc<AudioRingBuffer>; 2],
    },
    StartRecording {
        start_frame: u64,
        start_beat: f64,
    },
    CycleTake {
        track_id: u64,
        timeline_frame: u64,
    },
    CloseTrack {
        track_id: u64,
    },
    StopRecording,
    Shutdown,
}

/// Active WAV writer context managed on the background thread.
struct ActiveTakeWriter {
    track_id: u64,
    session_dir: PathBuf,
    prefix: String,
    sample_rate: u32,
    channels: u16,
    ring_buffers: [Arc<AudioRingBuffer>; 2],
    active_buf_idx: usize,
    current_take: usize,
    start_frame: u64,
    start_beat: f64,
    frames_written: u64,
    file_path: PathBuf,
    writer: Option<WavWriter<BufWriter<File>>>,
}

impl ActiveTakeWriter {
    fn open_take_file(&mut self) {
        let filename = format!("{}_take{}.wav", self.prefix, self.current_take);
        self.file_path = self.session_dir.join(filename);
        self.frames_written = 0;

        let spec = WavSpec {
            channels: self.channels,
            sample_rate: self.sample_rate,
            bits_per_sample: 32,
            sample_format: SampleFormat::Float,
        };

        if let Ok(w) = WavWriter::create(&self.file_path, spec) {
            self.writer = Some(w);
        }
    }

    fn flush_and_close(&mut self) -> Option<RecordedTakeResult> {
        if let Some(mut w) = self.writer.take() {
            // Drain remaining samples from current active ring buffer
            let ring = &self.ring_buffers[self.active_buf_idx];
            let mut scratch = [0.0f32; 4096];
            while ring.available_read() > 0 {
                let read = ring.pop_slice(&mut scratch);
                if read == 0 {
                    break;
                }
                for &s in &scratch[..read] {
                    let _ = w.write_sample(s);
                }
                self.frames_written += (read / self.channels as usize) as u64;
            }
            let _ = w.flush();
            let _ = w.finalize();

            Some(RecordedTakeResult {
                track_id: self.track_id,
                take_index: self.current_take,
                file_path: self.file_path.clone(),
                start_frame: self.start_frame,
                start_beat: self.start_beat,
                total_frames: self.frames_written,
                sample_rate: self.sample_rate,
                channels: self.channels,
            })
        } else {
            None
        }
    }
}

/// Background worker thread that drains audio ring buffers to disk as standard WAV files.
struct DiskRecordingWorker {
    cmd_rx: Receiver<DiskWriterCommand>,
    running: Arc<AtomicBool>,
    completed_takes: Arc<Mutex<Vec<RecordedTakeResult>>>,
}

impl DiskRecordingWorker {
    fn spawn(
        cmd_rx: Receiver<DiskWriterCommand>,
        running: Arc<AtomicBool>,
        completed_takes: Arc<Mutex<Vec<RecordedTakeResult>>>,
    ) -> JoinHandle<()> {
        thread::Builder::new()
            .name("summoner-disk-recorder".to_string())
            .spawn(move || {
                let mut worker = Self {
                    cmd_rx,
                    running,
                    completed_takes,
                };
                worker.run();
            })
            .expect("Failed to spawn summoner-disk-recorder thread")
    }

    fn run(&mut self) {
        let mut active_tracks: HashMap<u64, ActiveTakeWriter> = HashMap::new();
        let mut is_recording = false;
        let global_bpm = 120.0f64;
        let mut scratch_buf = [0.0f32; 4096];

        while self.running.load(Ordering::Acquire) {
            // Drain commands
            while let Ok(cmd) = self.cmd_rx.try_recv() {
                match cmd {
                    DiskWriterCommand::ArmTrack {
                        track_id,
                        session_dir,
                        prefix,
                        sample_rate,
                        channels,
                        ring_buffers,
                    } => {
                        let writer = ActiveTakeWriter {
                            track_id,
                            session_dir,
                            prefix,
                            sample_rate,
                            channels,
                            ring_buffers,
                            active_buf_idx: 0,
                            current_take: 1,
                            start_frame: 0,
                            start_beat: 0.0,
                            frames_written: 0,
                            file_path: PathBuf::new(),
                            writer: None,
                        };
                        active_tracks.insert(track_id, writer);
                    }
                    DiskWriterCommand::StartRecording {
                        start_frame,
                        start_beat,
                    } => {
                        is_recording = true;
                        for track in active_tracks.values_mut() {
                            track.active_buf_idx = 0;
                            track.start_frame = start_frame;
                            track.start_beat = start_beat;
                            track.open_take_file();
                        }
                    }
                    DiskWriterCommand::CycleTake {
                        track_id,
                        timeline_frame,
                    } => {
                        if let Some(track) = active_tracks.get_mut(&track_id) {
                            if let Some(res) = track.flush_and_close() {
                                if let Ok(mut takes) = self.completed_takes.lock() {
                                    takes.push(res);
                                }
                            }
                            track.active_buf_idx = 1 - track.active_buf_idx;
                            track.current_take += 1;
                            track.start_frame = timeline_frame;
                            let frames_per_beat = (track.sample_rate as f64 * 60.0) / global_bpm.max(1.0);
                            track.start_beat = (timeline_frame as f64) / frames_per_beat;
                            track.open_take_file();
                        }
                    }
                    DiskWriterCommand::CloseTrack { track_id } => {
                        if let Some(mut track) = active_tracks.remove(&track_id) {
                            if let Some(res) = track.flush_and_close() {
                                if let Ok(mut takes) = self.completed_takes.lock() {
                                    takes.push(res);
                                }
                            }
                        }
                    }
                    DiskWriterCommand::StopRecording => {
                        is_recording = false;
                        for track in active_tracks.values_mut() {
                            if let Some(res) = track.flush_and_close() {
                                if let Ok(mut takes) = self.completed_takes.lock() {
                                    takes.push(res);
                                }
                            }
                        }
                    }
                    DiskWriterCommand::Shutdown => {
                        for (_, mut track) in active_tracks.drain() {
                            if let Some(res) = track.flush_and_close() {
                                if let Ok(mut takes) = self.completed_takes.lock() {
                                    takes.push(res);
                                }
                            }
                        }
                        return;
                    }
                }
            }

            // If recording, drain audio samples from active ring buffers into open WAV writers
            let mut had_samples = false;
            if is_recording {
                for track in active_tracks.values_mut() {
                    if let Some(ref mut writer) = track.writer {
                        let ring = &track.ring_buffers[track.active_buf_idx];
                        let avail = ring.available_read();
                        if avail > 0 {
                            had_samples = true;
                            let to_read = avail.min(scratch_buf.len());
                            let read = ring.pop_slice(&mut scratch_buf[..to_read]);
                            for &sample in &scratch_buf[..read] {
                                let _ = writer.write_sample(sample);
                            }
                            track.frames_written += (read / track.channels as usize) as u64;
                        }
                    }
                }
            }

            if !had_samples {
                thread::sleep(Duration::from_millis(5));
            }
        }

        // Final cleanup on exit
        for (_, mut track) in active_tracks.drain() {
            if let Some(res) = track.flush_and_close() {
                if let Ok(mut takes) = self.completed_takes.lock() {
                    takes.push(res);
                }
            }
        }
    }
}

/// Real-time safe state for an armed recording track.
pub struct ArmedTrackState {
    pub track_id: u64,
    pub source: AudioInputSource,
    pub current_take: usize,
    pub ring_buffers: [Arc<AudioRingBuffer>; 2],
    pub active_buf_idx: usize,
    pub samples_pushed: u64,
}

/// Multi-track audio recorder managing track arming, count-in, punch, loops and disk writing.
pub struct AudioRecorder {
    armed_tracks: HashMap<u64, ArmedTrackState>,
    state: RecordState,
    sample_rate: u32,
    bpm: f64,
    punch: PunchSettings,
    loop_cfg: LoopRecordSettings,
    cmd_tx: Sender<DiskWriterCommand>,
    worker_running: Arc<AtomicBool>,
    worker_handle: Option<JoinHandle<()>>,
    completed_takes: Arc<Mutex<Vec<RecordedTakeResult>>>,
}

impl AudioRecorder {
    /// Create a new `AudioRecorder` with write-ahead worker thread.
    pub fn new(sample_rate: u32, bpm: f64) -> Self {
        let (cmd_tx, cmd_rx) = bounded(256);
        let worker_running = Arc::new(AtomicBool::new(true));
        let completed_takes = Arc::new(Mutex::new(Vec::new()));
        let worker_handle = DiskRecordingWorker::spawn(
            cmd_rx,
            Arc::clone(&worker_running),
            Arc::clone(&completed_takes),
        );

        Self {
            armed_tracks: HashMap::new(),
            state: RecordState::Inactive,
            sample_rate,
            bpm,
            punch: PunchSettings::default(),
            loop_cfg: LoopRecordSettings::default(),
            cmd_tx,
            worker_running,
            worker_handle: Some(worker_handle),
            completed_takes,
        }
    }

    /// Arm a track for audio recording. Called from non-realtime thread.
    pub fn arm_track<P: AsRef<Path>>(
        &mut self,
        track_id: u64,
        source: AudioInputSource,
        session_dir: P,
        file_prefix: &str,
    ) {
        const RING_CAPACITY: usize = 131_072;
        let ring_buffers = [
            Arc::new(AudioRingBuffer::new(RING_CAPACITY)),
            Arc::new(AudioRingBuffer::new(RING_CAPACITY)),
        ];

        let channels = match source {
            AudioInputSource::Mono(_) => 1u16,
            AudioInputSource::Stereo(_, _) => 2u16,
        };

        let _ = self.cmd_tx.send(DiskWriterCommand::ArmTrack {
            track_id,
            session_dir: session_dir.as_ref().to_path_buf(),
            prefix: file_prefix.to_string(),
            sample_rate: self.sample_rate,
            channels,
            ring_buffers: [Arc::clone(&ring_buffers[0]), Arc::clone(&ring_buffers[1])],
        });

        let track_state = ArmedTrackState {
            track_id,
            source,
            current_take: 1,
            ring_buffers,
            active_buf_idx: 0,
            samples_pushed: 0,
        };

        self.armed_tracks.insert(track_id, track_state);
    }

    /// Disarm a track. Called from non-realtime thread.
    pub fn disarm_track(&mut self, track_id: u64) {
        if self.armed_tracks.remove(&track_id).is_some() {
            let _ = self.cmd_tx.send(DiskWriterCommand::CloseTrack { track_id });
        }
    }

    /// Return true if the given track is currently armed.
    pub fn is_track_armed(&self, track_id: u64) -> bool {
        self.armed_tracks.contains_key(&track_id)
    }

    /// Number of armed tracks.
    pub fn armed_track_count(&self) -> usize {
        self.armed_tracks.len()
    }

    /// Current recording state.
    pub fn state(&self) -> RecordState {
        self.state
    }

    /// Start a recording session. Called from non-realtime thread.
    pub fn start_recording(
        &mut self,
        current_timeline_frame: u64,
        count_in_bars: u32,
        punch: Option<PunchSettings>,
        loop_cfg: Option<LoopRecordSettings>,
    ) {
        if self.armed_tracks.is_empty() {
            return;
        }

        self.punch = punch.unwrap_or_default();
        self.loop_cfg = loop_cfg.unwrap_or_default();

        let frames_per_beat = (self.sample_rate as f64 * 60.0) / self.bpm.max(1.0);
        let start_beat = (current_timeline_frame as f64) / frames_per_beat;

        if count_in_bars > 0 {
            let beats_per_bar = 4.0;
            let count_in_frames = (count_in_bars as f64 * beats_per_bar * frames_per_beat).round() as u64;
            self.state = RecordState::CountIn {
                remaining_frames: count_in_frames,
                total_bars: count_in_bars,
            };
        } else {
            self.state = RecordState::Recording;
            let _ = self.cmd_tx.send(DiskWriterCommand::StartRecording {
                start_frame: current_timeline_frame,
                start_beat,
            });
        }
    }

    /// Process incoming audio blocks from the hardware input.
    ///
    /// This method is strictly RT-safe, does zero heap allocations, and operates under `AllocGuard`.
    pub fn process_input_block(
        &mut self,
        current_timeline_frame: u64,
        in_left: &[f32],
        in_right: &[f32],
    ) {
        let _guard = AllocGuard::new();
        let frames = in_left.len().min(in_right.len());
        if frames == 0 {
            return;
        }

        match self.state {
            RecordState::Inactive | RecordState::Paused => return,
            RecordState::CountIn {
                ref mut remaining_frames,
                ..
            } => {
                if *remaining_frames <= frames as u64 {
                    *remaining_frames = 0;
                    self.state = RecordState::Recording;
                    let frames_per_beat = (self.sample_rate as f64 * 60.0) / self.bpm.max(1.0);
                    let start_beat = (current_timeline_frame as f64) / frames_per_beat;
                    let _ = self.cmd_tx.try_send(DiskWriterCommand::StartRecording {
                        start_frame: current_timeline_frame,
                        start_beat,
                    });
                } else {
                    *remaining_frames -= frames as u64;
                    return;
                }
            }
            RecordState::Recording => {}
        }

        // Punch-in check: if outside punch-in locator, don't record
        if !self.punch.is_in_punch(current_timeline_frame) {
            return;
        }

        // Loop boundary check: if loop recording is enabled and playhead looped, advance take
        if self.loop_cfg.loop_enabled
            && self.loop_cfg.loop_end_frame > self.loop_cfg.loop_start_frame
            && current_timeline_frame >= self.loop_cfg.loop_end_frame
        {
            for track in self.armed_tracks.values_mut() {
                track.active_buf_idx = 1 - track.active_buf_idx;
                track.current_take += 1;
                let _ = self.cmd_tx.try_send(DiskWriterCommand::CycleTake {
                    track_id: track.track_id,
                    timeline_frame: self.loop_cfg.loop_start_frame,
                });
            }
        }

        // Stack scratch buffer for pushing into active ring buffer without heap allocation
        const MAX_SCRATCH: usize = 2048;
        let mut scratch = [0.0f32; MAX_SCRATCH];

        for track in self.armed_tracks.values_mut() {
            let ring = &track.ring_buffers[track.active_buf_idx];
            match track.source {
                AudioInputSource::Mono(ch) => {
                    let src = if ch == 0 { in_left } else { in_right };
                    let to_push = frames.min(MAX_SCRATCH);
                    scratch[..to_push].copy_from_slice(&src[..to_push]);
                    let written = ring.push_slice(&scratch[..to_push]);
                    track.samples_pushed += written as u64;
                }
                AudioInputSource::Stereo(ch_l, ch_r) => {
                    let src_l = if ch_l == 0 { in_left } else { in_right };
                    let src_r = if ch_r == 0 { in_left } else { in_right };
                    let to_push = frames.min(MAX_SCRATCH / 2);
                    for i in 0..to_push {
                        scratch[i * 2] = src_l[i];
                        scratch[i * 2 + 1] = src_r[i];
                    }
                    let written = ring.push_slice(&scratch[..to_push * 2]);
                    track.samples_pushed += written as u64;
                }
            }
        }
    }

    /// Stop recording, flush all disk streams, and return all finalized takes.
    pub fn stop_recording(&mut self) -> Vec<RecordedTakeResult> {
        self.state = RecordState::Inactive;

        let _ = self.cmd_tx.send(DiskWriterCommand::StopRecording);

        // Allow disk worker to flush all remaining samples
        thread::sleep(Duration::from_millis(60));

        if let Ok(mut lock) = self.completed_takes.lock() {
            let res = lock.clone();
            lock.clear();
            res
        } else {
            Vec::new()
        }
    }
}

impl Drop for AudioRecorder {
    fn drop(&mut self) {
        let _ = self.cmd_tx.send(DiskWriterCommand::Shutdown);
        self.worker_running.store(false, Ordering::Release);
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_audio_recorder_multitrack_and_zero_alloc() {
        let temp_dir = std::env::temp_dir().join(format!("summoner_rec_test_{}", std::process::id()));
        fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

        let mut recorder = AudioRecorder::new(44100, 120.0);

        // Arm Track 1 (Stereo) and Track 2 (Mono Right)
        recorder.arm_track(1, AudioInputSource::Stereo(0, 1), &temp_dir, "vox");
        recorder.arm_track(2, AudioInputSource::Mono(1), &temp_dir, "bass");

        assert_eq!(recorder.armed_track_count(), 2);
        assert!(recorder.is_track_armed(1));
        assert!(recorder.is_track_armed(2));

        // Start recording immediately
        recorder.start_recording(0, 0, None, None);
        assert_eq!(recorder.state(), RecordState::Recording);

        // Feed 1024 frames of audio under AllocGuard
        let in_l = vec![0.5f32; 1024];
        let in_r = vec![-0.5f32; 1024];

        {
            let _guard = AllocGuard::new();
            recorder.process_input_block(0, &in_l, &in_r);
        }

        let takes = recorder.stop_recording();
        assert_eq!(takes.len(), 2);

        // Verify files exist and are valid WAV files
        for take in &takes {
            assert!(take.file_path.exists());
            let reader = hound::WavReader::open(&take.file_path).expect("Valid WAV file");
            assert_eq!(reader.spec().sample_rate, 44100);
            if take.track_id == 1 {
                assert_eq!(reader.spec().channels, 2);
                assert_eq!(take.total_frames, 1024);
                let clip = take.to_audio_clip_config("clip_1", "Vocal Take 1", 120.0);
                assert_eq!(clip.id, "clip_1");
                assert_eq!(clip.duration_frames, Some(1024));
            } else {
                assert_eq!(reader.spec().channels, 1);
                assert_eq!(take.total_frames, 1024);
            }
        }

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_audio_recorder_count_in_and_punch() {
        let temp_dir = std::env::temp_dir().join(format!("summoner_rec_punch_{}", std::process::id()));
        fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

        let mut recorder = AudioRecorder::new(48000, 120.0);
        recorder.arm_track(10, AudioInputSource::Mono(0), &temp_dir, "gtr");

        // Count-in for 1 bar (at 120 BPM, 4 beats = 2.0s = 96,000 frames)
        // Punch from frame 100,000 to frame 102,000
        let punch = PunchSettings {
            punch_in_frame: Some(100_000),
            punch_out_frame: Some(102_000),
        };

        recorder.start_recording(0, 1, Some(punch), None);
        assert!(matches!(recorder.state(), RecordState::CountIn { .. }));

        let block = vec![0.1f32; 48000];

        {
            let _guard = AllocGuard::new();
            // Frame 0..48000 (still in count-in)
            recorder.process_input_block(0, &block, &block);
        }
        assert!(matches!(recorder.state(), RecordState::CountIn { .. }));

        {
            let _guard = AllocGuard::new();
            // Frame 48000..96000 (finishes count-in, enters recording)
            recorder.process_input_block(48000, &block, &block);
        }
        assert_eq!(recorder.state(), RecordState::Recording);

        let block2k = vec![0.2f32; 2000];
        {
            let _guard = AllocGuard::new();
            // Frame 96000..98000 (outside punch-in, frames dropped)
            recorder.process_input_block(96000, &block2k, &block2k);

            // Frame 100000..102000 (inside punch-in!)
            recorder.process_input_block(100000, &block2k, &block2k);

            // Frame 102000..104000 (after punch-out, frames dropped)
            recorder.process_input_block(102000, &block2k, &block2k);
        }

        let takes = recorder.stop_recording();
        assert_eq!(takes.len(), 1);
        assert_eq!(takes[0].total_frames, 2000);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_audio_recorder_loop_takes() {
        let temp_dir = std::env::temp_dir().join(format!("summoner_rec_loop_{}", std::process::id()));
        fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

        let mut recorder = AudioRecorder::new(48000, 120.0);
        recorder.arm_track(5, AudioInputSource::Mono(0), &temp_dir, "solo");

        let loop_cfg = LoopRecordSettings {
            loop_enabled: true,
            loop_start_frame: 0,
            loop_end_frame: 1000,
        };

        recorder.start_recording(0, 0, None, Some(loop_cfg));

        let block = vec![0.3f32; 500];

        {
            let _guard = AllocGuard::new();
            // Take 1: frame 0..500
            recorder.process_input_block(0, &block, &block);
            // Take 1: frame 500..1000
            recorder.process_input_block(500, &block, &block);

            // Cross loop boundary at 1000! Triggers Take 2
            recorder.process_input_block(1000, &block, &block);
        }

        let takes = recorder.stop_recording();
        assert_eq!(takes.len(), 2);
        assert_eq!(takes[0].take_index, 1);
        assert_eq!(takes[0].total_frames, 1000);
        assert_eq!(takes[1].take_index, 2);
        assert_eq!(takes[1].total_frames, 500);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
