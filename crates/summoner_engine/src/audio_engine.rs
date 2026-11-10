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
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use summoner_core::allocator::AllocGuard;
use summoner_core::audio::{AudioRingBuffer, FixedAudioBuffer};
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
    SetMetronomeEnabled(bool),
    SetMetronomeGain(f32),
    StartCountIn(u32),
}

#[cfg(target_os = "windows")]
pub fn set_realtime_audio_thread_priority() {
    unsafe {
        use winapi::um::processthreadsapi::{GetCurrentThread, SetThreadPriority};
        // THREAD_PRIORITY_TIME_CRITICAL = 15
        SetThreadPriority(GetCurrentThread(), 15);
    }
}

#[cfg(not(target_os = "windows"))]
pub fn set_realtime_audio_thread_priority() {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AudioDeviceInfo {
    pub name: String,
    pub is_default: bool,
    pub max_channels: u16,
    pub default_sample_rate: u32,
    pub supported_sample_rates: Vec<u32>,
}

pub struct AudioDeviceManager;

impl AudioDeviceManager {
    /// List all audio output devices available on the system.
    pub fn list_output_devices() -> Vec<AudioDeviceInfo> {
        let host = cpal::default_host();
        let default_name = host.default_output_device().and_then(|d| d.name().ok());
        let mut devices = Vec::new();
        if let Ok(dev_iter) = host.output_devices() {
            for dev in dev_iter {
                if let Ok(name) = dev.name() {
                    let is_default = default_name.as_deref() == Some(&name);
                    let mut max_channels = 2;
                    let mut default_sr = 44100;
                    let mut supported_rates = Vec::new();
                    if let Ok(configs) = dev.supported_output_configs() {
                        for c in configs {
                            max_channels = max_channels.max(c.channels());
                            let min_r = c.min_sample_rate().0;
                            let max_r = c.max_sample_rate().0;
                            for r in [44100, 48000, 88200, 96000, 192000] {
                                if r >= min_r && r <= max_r && !supported_rates.contains(&r) {
                                    supported_rates.push(r);
                                }
                            }
                        }
                    }
                    if let Ok(cfg) = dev.default_output_config() {
                        default_sr = cfg.sample_rate().0;
                    }
                    devices.push(AudioDeviceInfo {
                        name,
                        is_default,
                        max_channels,
                        default_sample_rate: default_sr,
                        supported_sample_rates: supported_rates,
                    });
                }
            }
        }
        devices
    }

    /// List all audio input devices available on the system.
    pub fn list_input_devices() -> Vec<AudioDeviceInfo> {
        let host = cpal::default_host();
        let default_name = host.default_input_device().and_then(|d| d.name().ok());
        let mut devices = Vec::new();
        if let Ok(dev_iter) = host.input_devices() {
            for dev in dev_iter {
                if let Ok(name) = dev.name() {
                    let is_default = default_name.as_deref() == Some(&name);
                    let mut max_channels = 1;
                    let mut default_sr = 44100;
                    let mut supported_rates = Vec::new();
                    if let Ok(configs) = dev.supported_input_configs() {
                        for c in configs {
                            max_channels = max_channels.max(c.channels());
                            let min_r = c.min_sample_rate().0;
                            let max_r = c.max_sample_rate().0;
                            for r in [44100, 48000, 88200, 96000] {
                                if r >= min_r && r <= max_r && !supported_rates.contains(&r) {
                                    supported_rates.push(r);
                                }
                            }
                        }
                    }
                    if let Ok(cfg) = dev.default_input_config() {
                        default_sr = cfg.sample_rate().0;
                    }
                    devices.push(AudioDeviceInfo {
                        name,
                        is_default,
                        max_channels,
                        default_sample_rate: default_sr,
                        supported_sample_rates: supported_rates,
                    });
                }
            }
        }
        devices
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AudioEngineSettings {
    pub output_device_name: Option<String>,
    pub input_device_name: Option<String>,
    pub enable_input: bool,
    pub midi_input_device_name: Option<String>,
    pub enable_midi: bool,
    pub requested_sample_rate: Option<u32>,
    pub requested_buffer_size: Option<u32>,
}

impl Default for AudioEngineSettings {
    fn default() -> Self {
        Self {
            output_device_name: None,
            input_device_name: None,
            enable_input: true,
            midi_input_device_name: None,
            enable_midi: true,
            requested_sample_rate: None,
            requested_buffer_size: None,
        }
    }
}

pub struct LiveEngineHandle {
    pub playing: Arc<AtomicBool>,
    pub frame_position: Arc<AtomicU64>,
    pub sample_rate: u32,
    pub param_bus: Arc<ParamBus>,
    pub cmd_tx: Sender<EngineCommand>,
    pub param_tx: Sender<ParamUpdate>,
    pub input_active: Arc<AtomicBool>,
    pub cpu_load_milli_pct: Arc<AtomicU32>,
    pub xrun_count: Arc<AtomicU64>,
    pub input_ring_buffer: Arc<AudioRingBuffer>,
    pub midi_receiver: Option<Arc<crate::midi_engine::MidiInputReceiver>>,
    _stream: Option<cpal::Stream>,
    _input_stream: Option<cpal::Stream>,
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

    pub fn cpu_load_pct(&self) -> f32 {
        self.cpu_load_milli_pct.load(Ordering::Relaxed) as f32 / 1000.0
    }

    pub fn xruns(&self) -> u64 {
        self.xrun_count.load(Ordering::Relaxed)
    }

    pub fn is_input_active(&self) -> bool {
        self.input_active.load(Ordering::Relaxed)
    }

    pub fn read_input_samples(&self, dest: &mut [f32]) -> usize {
        self.input_ring_buffer.pop_slice(dest)
    }

    pub fn is_midi_active(&self) -> bool {
        self.midi_receiver
            .as_ref()
            .map(|r| r.is_active())
            .unwrap_or(false)
    }

    pub fn try_recv_midi(&self) -> Option<crate::midi_engine::TimestampedMidiEvent> {
        self.midi_receiver.as_ref().and_then(|r| r.try_recv())
    }

    pub fn set_metronome_enabled(&self, enabled: bool) {
        let _ = self.cmd_tx.try_send(EngineCommand::SetMetronomeEnabled(enabled));
    }

    pub fn set_metronome_gain(&self, gain: f32) {
        let _ = self.cmd_tx.try_send(EngineCommand::SetMetronomeGain(gain));
    }

    pub fn start_count_in(&self, bars: u32) {
        let _ = self.cmd_tx.try_send(EngineCommand::StartCountIn(bars));
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

/// Render each track in `project` to an interleaved stereo buffer `[L0, R0, L1, R1, ...]` at `sample_rate`.
pub fn render_project_track_stems(
    project: &ProjectConfig,
    sample_rate: u32,
    explicit_frames: Option<usize>,
) -> Vec<Vec<f32>> {
    let mut proj_for_rate = project.clone();
    proj_for_rate.transport.sample_rate = sample_rate;
    let num_frames = explicit_frames.unwrap_or_else(|| default_render_frames(&proj_for_rate));

    const BLOCK_SIZE: usize = 64;
    let mut stems = Vec::with_capacity(proj_for_rate.tracks.len());

    for (idx, tc) in proj_for_rate.tracks.iter().enumerate() {
        let mut single_proj = proj_for_rate.clone();
        let mut solo_tc = tc.clone();
        solo_tc.soloed = false;
        single_proj.tracks = vec![solo_tc];

        let mut runner = GraphRunner::new(&single_proj);
        let mut transport = Transport::new(sample_rate, single_proj.transport.bpm);
        transport.play();

        let mut out_l = [0.0f32; BLOCK_SIZE];
        let mut out_r = [0.0f32; BLOCK_SIZE];
        let mut interleaved = Vec::with_capacity(num_frames * 2);

        if !tc.muted {
            let mut frames_processed = 0usize;
            while frames_processed < num_frames {
                let block_frames = (num_frames - frames_processed).min(BLOCK_SIZE);
                out_l[..block_frames].fill(0.0);
                out_r[..block_frames].fill(0.0);
                let ctx = ProcessContext::from_transport(&transport);
                runner.process_block(
                    block_frames,
                    &ctx,
                    &mut [&mut out_l[..block_frames], &mut out_r[..block_frames]],
                );
                for f in 0..block_frames {
                    interleaved.push(out_l[f]);
                    interleaved.push(out_r[f]);
                }
                transport.advance_frames(block_frames as u64);
                frames_processed += block_frames;
            }
        } else {
            interleaved.resize(num_frames * 2, 0.0);
        }
        let _ = idx;
        stems.push(interleaved);
    }

    stems
}

/// Start a non-blocking real-time CPAL audio stream for the given project and shared `ParamBus`.
/// If no audio output device is available (e.g. headless CI), returns a handle without an active hardware stream.
pub fn spawn_live_engine(
    project: &ProjectConfig,
    param_bus: Arc<ParamBus>,
    start_playing: bool,
) -> LiveEngineHandle {
    spawn_live_engine_with_settings(
        project,
        param_bus,
        start_playing,
        &AudioEngineSettings::default(),
    )
}

/// Start real-time CPAL audio playback and input capture using the provided `AudioEngineSettings`.
pub fn spawn_live_engine_with_settings(
    project: &ProjectConfig,
    param_bus: Arc<ParamBus>,
    start_playing: bool,
    settings: &AudioEngineSettings,
) -> LiveEngineHandle {
    let playing = Arc::new(AtomicBool::new(start_playing));
    let frame_position = Arc::new(AtomicU64::new(0));
    let input_active = Arc::new(AtomicBool::new(false));
    let cpu_load_milli_pct = Arc::new(AtomicU32::new(0));
    let xrun_count = Arc::new(AtomicU64::new(0));
    let input_ring_buffer = Arc::new(AudioRingBuffer::new(65536));

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

    // 1. Resolve output device
    let device_opt = if let Some(ref name) = settings.output_device_name {
        host.output_devices()
            .ok()
            .and_then(|mut devs| devs.find(|d| d.name().map(|n| n == *name).unwrap_or(false)))
            .or_else(|| host.default_output_device())
    } else {
        host.default_output_device()
    };

    let config_opt = device_opt
        .as_ref()
        .and_then(|d| d.default_output_config().ok());

    // 2. Resolve input device & build input stream if enabled
    let mut in_stream: Option<cpal::Stream> = None;
    let mut in_channels = 0usize;

    if settings.enable_input {
        let in_dev_opt = if let Some(ref name) = settings.input_device_name {
            host.input_devices()
                .ok()
                .and_then(|mut devs| devs.find(|d| d.name().map(|n| n == *name).unwrap_or(false)))
                .or_else(|| host.default_input_device())
        } else {
            host.default_input_device()
        };

        if let Some(in_dev) = in_dev_opt {
            if let Ok(in_cfg) = in_dev.default_input_config() {
                let stream_cfg: cpal::StreamConfig = in_cfg.into();
                in_channels = stream_cfg.channels as usize;
                let rb_producer = Arc::clone(&input_ring_buffer);
                let in_active_flag = Arc::clone(&input_active);

                let res = in_dev.build_input_stream(
                    &stream_cfg,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        let _guard = AllocGuard::new();
                        rb_producer.push_slice(data);
                        in_active_flag.store(true, Ordering::Relaxed);
                    },
                    |err| eprintln!("Live audio input stream error: {}", err),
                    None,
                );
                if let Ok(s) = res {
                    let _ = s.play();
                    in_stream = Some(s);
                }
            }
        }
    }

    // 3. Resolve MIDI input device if enabled
    let midi_receiver = if settings.enable_midi {
        crate::midi_engine::MidiInputReceiver::open_port(
            settings.midi_input_device_name.as_deref(),
        )
        .ok()
        .map(|(rx, _)| Arc::new(rx))
    } else {
        None
    };

    if let (Some(device), Some(config)) = (device_opt, config_opt) {
        let sample_rate = config.sample_rate().0;
        let stream_config: cpal::StreamConfig = config.into();
        let channels = stream_config.channels as usize;

        let mut proj_for_rate = project.clone();
        proj_for_rate.transport.sample_rate = sample_rate;
        let mut runner = GraphRunner::new(&proj_for_rate);
        let mut bpm = project.transport.bpm;
        let mut loop_enabled = project.loop_enabled;
        let mut loop_start_beat = project.loop_start_beat;
        let mut loop_end_beat = project.loop_end_beat;
        let mut current_frame = 0u64;
        let mut metronome = crate::metronome::MetronomeGenerator::new(sample_rate);

        let playing_cb = Arc::clone(&playing);
        let frame_pos_cb = Arc::clone(&frame_position);
        let param_bus_audio = Arc::clone(&param_bus);
        let rb_consumer = Arc::clone(&input_ring_buffer);
        let cpu_cb = Arc::clone(&cpu_load_milli_pct);
        let xrun_cb = Arc::clone(&xrun_count);

        const MAX_BLOCK_SIZE: usize = 8192;
        let mut out_l = Box::new([0.0f32; MAX_BLOCK_SIZE]);
        let mut out_r = Box::new([0.0f32; MAX_BLOCK_SIZE]);
        let mut in_l = Box::new([0.0f32; MAX_BLOCK_SIZE]);
        let mut in_r = Box::new([0.0f32; MAX_BLOCK_SIZE]);
        let mut in_interleaved = Box::new([0.0f32; MAX_BLOCK_SIZE * 2]);

        let stream_res = device.build_output_stream(
            &stream_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                set_realtime_audio_thread_priority();

                // Drain non-allocating commands & pre-built runners outside AllocGuard if needed
                while let Ok(cmd) = cmd_rx_worker.try_recv() {
                    match cmd {
                        EngineCommand::ReloadProject(mut p) => {
                            p.transport.sample_rate = sample_rate;
                            bpm = p.transport.bpm;
                            loop_enabled = p.loop_enabled;
                            loop_start_beat = p.loop_start_beat;
                            loop_end_beat = p.loop_end_beat;
                            let _ = reload_proj_tx.try_send(*p);
                        }
                        EngineCommand::SeekFrame(f) => {
                            current_frame = f;
                            frame_pos_cb.store(f, Ordering::Relaxed);
                        }
                        EngineCommand::SetBpm(b) => {
                            bpm = b;
                        }
                        EngineCommand::SetMetronomeEnabled(en) => {
                            metronome.enabled = en;
                        }
                        EngineCommand::SetMetronomeGain(g) => {
                            metronome.gain = g;
                        }
                        EngineCommand::StartCountIn(bars) => {
                            metronome.start_count_in(bars);
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

                // Drain input ring buffer for live monitoring into graph
                let mut has_input = false;
                if in_channels > 0 {
                    let needed = frames * in_channels;
                    let to_pop = needed.min(in_interleaved.len());
                    let popped = rb_consumer.pop_slice(&mut in_interleaved[..to_pop]);
                    let popped_frames = popped / in_channels;
                    for f in 0..popped_frames {
                        in_l[f] = in_interleaved[f * in_channels];
                        in_r[f] = if in_channels > 1 {
                            in_interleaved[f * in_channels + 1]
                        } else {
                            in_interleaved[f * in_channels]
                        };
                    }
                    if popped_frames < frames {
                        in_l[popped_frames..frames].fill(0.0);
                        in_r[popped_frames..frames].fill(0.0);
                    }
                    has_input = popped_frames > 0;
                }

                let in_slices: Option<&[&[f32]]> = if has_input {
                    Some(&[&in_l[..frames], &in_r[..frames]])
                } else {
                    None
                };

                let mut ctx = ProcessContext::new(sample_rate, bpm, current_frame);
                ctx.is_playing = true;
                ctx.param_bus = Some(Arc::clone(&param_bus_audio));

                let buf_l = &mut out_l[..frames];
                let buf_r = &mut out_r[..frames];
                buf_l.fill(0.0);
                buf_r.fill(0.0);

                let start_time = Instant::now();
                runner.process_block_with_input(frames, &ctx, in_slices, &mut [buf_l, buf_r]);

                if metronome.enabled || metronome.is_counting_in {
                    let mut transport = Transport::new(sample_rate, bpm);
                    transport.seek_frame(current_frame);
                    transport.play();
                    metronome.process_block(&transport, frames, buf_l, buf_r);
                }
                let elapsed = start_time.elapsed();
                let block_duration = frames as f64 / sample_rate as f64;
                let cpu_pct = (elapsed.as_secs_f64() / block_duration.max(0.0001) * 100.0) as f32;
                cpu_cb.store((cpu_pct.clamp(0.0, 1000.0) * 1000.0) as u32, Ordering::Relaxed);
                if elapsed.as_secs_f64() > block_duration {
                    xrun_cb.fetch_add(1, Ordering::Relaxed);
                }

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
                if loop_enabled && loop_end_beat > loop_start_beat && bpm > 0.0 {
                    let frames_per_beat = (sample_rate as f64 * 60.0) / bpm;
                    let loop_start_frame = (loop_start_beat * frames_per_beat).round() as u64;
                    let loop_end_frame = (loop_end_beat * frames_per_beat).round() as u64;
                    if current_frame >= loop_end_frame && loop_end_frame > loop_start_frame {
                        let loop_len = loop_end_frame - loop_start_frame;
                        current_frame = loop_start_frame + ((current_frame - loop_start_frame) % loop_len);
                    }
                }
                frame_pos_cb.store(current_frame, Ordering::Relaxed);
            },
            |err| eprintln!("an error occurred on output stream: {}", err),
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
                input_active,
                cpu_load_milli_pct,
                xrun_count,
                input_ring_buffer,
                midi_receiver,
                _stream: Some(stream),
                _input_stream: in_stream,
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
        input_active,
        cpu_load_milli_pct,
        xrun_count,
        input_ring_buffer,
        midi_receiver,
        _stream: None,
        _input_stream: in_stream,
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

    #[test]
    fn test_audio_device_manager_enumeration() {
        let outputs = AudioDeviceManager::list_output_devices();
        let inputs = AudioDeviceManager::list_input_devices();
        // Enumeration should complete safely without panicking on any platform
        let _ = outputs.len();
        let _ = inputs.len();
    }

    #[test]
    fn test_live_engine_input_ring_buffer_and_monitoring() {
        let project = create_default_project("Live Input Test");
        let param_bus = Arc::new(ParamBus::new());
        let handle = spawn_live_engine(&project, param_bus, false);

        assert!(!handle.is_playing());
        assert_eq!(handle.xruns(), 0);
        assert_eq!(handle.cpu_load_pct(), 0.0);

        // Verify pushing test samples into input ring buffer and reading them back
        let test_input = [0.25f32, -0.25, 0.5, -0.5];
        let pushed = handle.input_ring_buffer.push_slice(&test_input);
        assert_eq!(pushed, 4);

        let mut read_buf = [0.0f32; 4];
        let read = handle.read_input_samples(&mut read_buf);
        assert_eq!(read, 4);
        assert_eq!(read_buf, test_input);
    }

    #[test]
    fn test_graph_runner_process_block_with_input_monitoring() {
        let mut project = create_default_project("Input Monitor Graph Test");
        if let Some(track) = project.tracks.first_mut() {
            track.input_monitoring = true;
        }
        let mut runner = GraphRunner::new(&project);

        let input_l = [0.75f32; 64];
        let input_r = [0.75f32; 64];
        let mut out_l = [0.0f32; 64];
        let mut out_r = [0.0f32; 64];

        let ctx = ProcessContext::new(44100, 120.0, 0);
        runner.process_block_with_input(
            64,
            &ctx,
            Some(&[&input_l, &input_r]),
            &mut [&mut out_l, &mut out_r],
        );

        // With input monitoring enabled, the live input audio must be mixed into the track output
        assert!(out_l.iter().any(|&s| s.abs() > 0.1));
        assert!(out_r.iter().any(|&s| s.abs() > 0.1));
    }

    #[test]
    fn test_live_engine_metronome_commands() {
        let project = create_default_project("Metronome Engine Test");
        let param_bus = Arc::new(ParamBus::new());
        let handle = spawn_live_engine(&project, param_bus, false);

        // Verify sending metronome commands without panic
        handle.set_metronome_enabled(true);
        handle.set_metronome_gain(0.85);
        handle.start_count_in(2);
    }
}

