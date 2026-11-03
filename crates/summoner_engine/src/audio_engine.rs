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

use crate::graph::GraphRunner;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::{Receiver, Sender};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use summoner_core::allocator::AllocGuard;
use summoner_core::audio::FixedAudioBuffer;
use summoner_core::node::ProcessContext;
use summoner_core::param_bus::{ParamBus, ParamId};
use summoner_core::transport::Transport;
use summoner_core::wav::WavWriter;
use summoner_project::schema::ProjectConfig;

pub enum ParamUpdate {
    Set(ParamId, f32),
}

pub enum EngineCommand {
    ReloadProject(Box<ProjectConfig>),
    SeekFrame(u64),
    SetBpm(f64),
}

pub struct LiveEngineHandle {
    pub playing: Arc<AtomicBool>,
    pub frame_position: Arc<AtomicU64>,
    pub sample_rate: u32,
    pub param_bus: Arc<ParamBus>,
    pub cmd_tx: Sender<EngineCommand>,
    pub param_tx: Sender<ParamUpdate>,
    _stream: Option<cpal::Stream>,
}

impl LiveEngineHandle {
    pub fn set_playing(&self, play: bool) {
        self.playing.store(play, Ordering::Relaxed);
    }

    pub fn is_playing(&self) -> bool {
        self.playing.load(Ordering::Relaxed)
    }

    pub fn seek_frame(&self, frame: u64) {
        let _ = self.cmd_tx.try_send(EngineCommand::SeekFrame(frame));
        self.frame_position.store(frame, Ordering::Relaxed);
    }

    pub fn reload_project(&self, project: ProjectConfig) {
        let _ = self
            .cmd_tx
            .try_send(EngineCommand::ReloadProject(Box::new(project)));
    }
}

/// Compute the default number of frames to render for a project based on its clips/sequences and loop length.
pub fn default_render_frames(project: &ProjectConfig) -> usize {
    let mut max_beat = project.loop_end_beat.max(0.0);
    for tc in &project.tracks {
        for seq in tc.all_sequences() {
            let seq_end = seq.start_beat + (seq.steps.len() as f64) * seq.step_division;
            if seq_end > max_beat {
                max_beat = seq_end;
            }
        }
    }
    if max_beat <= 0.0 {
        max_beat = 4.0;
    }
    let frames_per_beat =
        project.transport.sample_rate as f64 * 60.0 / project.transport.bpm.max(1.0);
    (max_beat * frames_per_beat).round().max(1.0) as usize
}

/// Deterministically render a `ProjectConfig` to a WAV file on disk using `GraphRunner`.
pub fn render_project_to_wav<P: AsRef<Path>>(
    project: &ProjectConfig,
    out_wav_path: P,
    explicit_frames: Option<usize>,
    param_bus: Option<Arc<ParamBus>>,
) -> Result<usize, String> {
    let num_frames = explicit_frames.unwrap_or_else(|| default_render_frames(project));
    let mut transport = Transport::new(project.transport.sample_rate, project.transport.bpm);
    transport.play();

    let mut runner = GraphRunner::new(project);

    const CHANNELS: usize = 2;
    const BLOCK_SIZE: usize = 64;
    let mut out_buffer = FixedAudioBuffer::<CHANNELS, BLOCK_SIZE>::new();

    let mut wav_writer = WavWriter::create(
        out_wav_path.as_ref(),
        project.transport.sample_rate,
        CHANNELS as u16,
    )
    .map_err(|e| format!("Failed to create WAV writer: {}", e))?;

    let mut frames_processed: usize = 0;
    let mut interleaved = vec![0.0f32; BLOCK_SIZE * CHANNELS];

    while frames_processed < num_frames {
        let block_frames = (num_frames - frames_processed).min(BLOCK_SIZE);
        out_buffer.set_active_frames(block_frames);
        out_buffer.clear();

        let mut ctx = ProcessContext::from_transport(&transport);
        ctx.param_bus = param_bus.clone();

        {
            let _guard = AllocGuard::new();
            let mut out_slices = out_buffer.channels_mut_2();
            runner.process_block(block_frames, &ctx, &mut out_slices);
        }

        let ch0 = out_buffer.channel(0);
        let ch1 = out_buffer.channel(1);
        for f in 0..block_frames {
            interleaved[f * 2] = ch0[f];
            interleaved[f * 2 + 1] = ch1[f];
        }

        wav_writer
            .write_interleaved_samples(&interleaved[..block_frames * CHANNELS])
            .map_err(|e| format!("Failed to write WAV samples: {}", e))?;

        transport.advance_frames(block_frames as u64);
        frames_processed += block_frames;
    }

    wav_writer
        .finalize()
        .map_err(|e| format!("Failed to finalize WAV header: {}", e))?;

    Ok(frames_processed)
}

/// Start a non-blocking real-time CPAL audio stream for the given project and shared `ParamBus`.
/// If no audio output device is available (e.g. headless CI), returns a handle without an active hardware stream.
pub fn spawn_live_engine(
    project: &ProjectConfig,
    param_bus: Arc<ParamBus>,
    start_playing: bool,
) -> LiveEngineHandle {
    let playing = Arc::new(AtomicBool::new(start_playing));
    let frame_position = Arc::new(AtomicU64::new(0));
    let (cmd_tx, cmd_rx): (Sender<EngineCommand>, Receiver<EngineCommand>) =
        crossbeam_channel::bounded(64);
    let (param_tx, param_rx): (Sender<ParamUpdate>, Receiver<ParamUpdate>) =
        crossbeam_channel::bounded(1024);

    // Pre-build a swap slot for GraphRunner so project reloads happen off the audio thread.
    let (runner_tx, runner_rx): (Sender<GraphRunner>, Receiver<GraphRunner>) =
        crossbeam_channel::bounded(8);
    let cmd_rx_worker = cmd_rx.clone();
    let sample_rate_default = project.transport.sample_rate;

    // Background worker builds new GraphRunners when ReloadProject is sent, avoiding allocation on audio callback.
    let (reload_proj_tx, reload_proj_rx) = crossbeam_channel::bounded::<ProjectConfig>(8);
    thread::spawn(move || {
        while let Ok(proj) = reload_proj_rx.recv() {
            let new_runner = GraphRunner::new(&proj);
            let _ = runner_tx.try_send(new_runner);
        }
    });

    let host = cpal::default_host();
    let device_opt = host.default_output_device();
    let config_opt = device_opt
        .as_ref()
        .and_then(|d| d.default_output_config().ok());

    if let (Some(device), Some(config)) = (device_opt, config_opt) {
        let sample_rate = config.sample_rate().0;
        let stream_config: cpal::StreamConfig = config.into();
        let channels = stream_config.channels as usize;

        let mut proj_for_rate = project.clone();
        proj_for_rate.transport.sample_rate = sample_rate;
        let mut runner = GraphRunner::new(&proj_for_rate);
        let mut bpm = project.transport.bpm;
        let mut current_frame = 0u64;

        let playing_cb = Arc::clone(&playing);
        let frame_pos_cb = Arc::clone(&frame_position);
        let param_bus_audio = Arc::clone(&param_bus);

        const MAX_BLOCK_SIZE: usize = 8192;
        let mut out_l = Box::new([0.0f32; MAX_BLOCK_SIZE]);
        let mut out_r = Box::new([0.0f32; MAX_BLOCK_SIZE]);

        let stream_res = device.build_output_stream(
            &stream_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // Drain non-allocating commands & pre-built runners outside AllocGuard if needed
                while let Ok(cmd) = cmd_rx_worker.try_recv() {
                    match cmd {
                        EngineCommand::ReloadProject(mut p) => {
                            p.transport.sample_rate = sample_rate;
                            bpm = p.transport.bpm;
                            let _ = reload_proj_tx.try_send(*p);
                        }
                        EngineCommand::SeekFrame(f) => {
                            current_frame = f;
                            frame_pos_cb.store(f, Ordering::Relaxed);
                        }
                        EngineCommand::SetBpm(b) => {
                            bpm = b;
                        }
                    }
                }
                while let Ok(new_r) = runner_rx.try_recv() {
                    runner = new_r;
                }

                let _alloc_guard = AllocGuard::new();

                while let Ok(update) = param_rx.try_recv() {
                    match update {
                        ParamUpdate::Set(id, val) => param_bus_audio.set(id, val),
                    }
                }

                let is_playing = playing_cb.load(Ordering::Relaxed);
                if !is_playing {
                    data.fill(0.0);
                    return;
                }

                let frames = (data.len() / channels.max(1)).min(MAX_BLOCK_SIZE);

                let mut ctx = ProcessContext::new(sample_rate, bpm, current_frame);
                ctx.is_playing = true;
                ctx.param_bus = Some(Arc::clone(&param_bus_audio));

                let buf_l = &mut out_l[..frames];
                let buf_r = &mut out_r[..frames];
                buf_l.fill(0.0);
                buf_r.fill(0.0);

                runner.process_block(frames, &ctx, &mut [buf_l, buf_r]);

                for (i, frame) in data.chunks_mut(channels.max(1)).enumerate() {
                    if i >= frames {
                        break;
                    }
                    let l = out_l[i];
                    let r = if channels > 1 { out_r[i] } else { out_l[i] };

                    frame[0] = l;
                    if channels > 1 {
                        frame[1] = r;
                    }
                }
                current_frame += frames as u64;
                frame_pos_cb.store(current_frame, Ordering::Relaxed);
            },
            |err| eprintln!("an error occurred on stream: {}", err),
            None,
        );

        if let Ok(stream) = stream_res {
            let _ = stream.play();
            return LiveEngineHandle {
                playing,
                frame_position,
                sample_rate,
                param_bus,
                cmd_tx,
                param_tx,
                _stream: Some(stream),
            };
        }
    }

    LiveEngineHandle {
        playing,
        frame_position,
        sample_rate: sample_rate_default,
        param_bus,
        cmd_tx,
        param_tx,
        _stream: None,
    }
}

/// Start the real-time audio thread using cpal and block forever (for `summon play`).
pub fn run_live(project: &ProjectConfig) -> ! {
    let param_bus = Arc::new(ParamBus::new());
    let _handle = spawn_live_engine(project, param_bus, true);
    loop {
        thread::park();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::GraphRunner;
    use summoner_core::allocator::AllocGuard;
    use summoner_project::create_default_project;

    #[test]
    fn test_audio_engine_builds_without_panic() {
        let project = create_default_project("Test Session");
        let mut runner = GraphRunner::new(&project);

        let mut out_l = vec![0.0f32; 512];
        let mut out_r = vec![0.0f32; 512];
        let ctx = ProcessContext::new(44100, 120.0, 0);
        runner.process_block(512, &ctx, &mut [&mut out_l, &mut out_r]);

        assert_eq!(out_l.len(), 512);
    }

    #[test]
    fn test_audio_callback_zero_alloc() {
        let project = create_default_project("Test Session");
        let mut runner = GraphRunner::new(&project);

        const MAX_BLOCK_SIZE: usize = 4096;
        let mut out_l = Box::new([0.0f32; MAX_BLOCK_SIZE]);
        let mut out_r = Box::new([0.0f32; MAX_BLOCK_SIZE]);

        let ctx = ProcessContext::new(44100, 120.0, 0);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = AllocGuard::new();
            let buf_l = &mut out_l[..512];
            let buf_r = &mut out_r[..512];
            buf_l.fill(0.0);
            buf_r.fill(0.0);
            runner.process_block(512, &ctx, &mut [buf_l, buf_r]);
        }));

        assert!(
            result.is_ok(),
            "Audio callback block processing triggered heap allocation!"
        );
    }

    #[test]
    fn test_render_project_to_wav_produces_non_silent_audio() {
        let project = create_default_project("WAV Render Test");
        let out_path = std::env::temp_dir().join("summoner_engine_render_test.wav");
        let frames = render_project_to_wav(&project, &out_path, Some(2048), None).unwrap();
        assert_eq!(frames, 2048);

        let mut reader = hound::WavReader::open(&out_path).unwrap();
        let samples: Vec<f32> = reader
            .samples::<i16>()
            .map(|s| s.unwrap() as f32 / i16::MAX as f32)
            .collect();
        assert!(samples.iter().any(|s| s.abs() > 0.05));
        let _ = std::fs::remove_file(&out_path);
    }
}

