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

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::thread;
use summoner_core::param_bus::{ParamBus, ParamId};
use summoner_core::node::ProcessContext;
use summoner_project::schema::ProjectConfig;
use crate::graph::GraphRunner;
use std::sync::Arc;
use crossbeam_channel::{Receiver, Sender};

pub enum ParamUpdate {
    Set(ParamId, f32),
}

/// Start the real-time audio thread using cpal.
/// Blocks the current thread while the stream plays.
pub fn run_live(project: &ProjectConfig) -> ! {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("Failed to find a default output device");
    
    println!("Using audio device: {}", device.name().unwrap_or_else(|_| "Unknown".to_string()));

    let config = device
        .default_output_config()
        .expect("Failed to get default output config");
    
    let sample_rate = config.sample_rate();
    println!("Sample rate: {}", sample_rate.0);

    let stream_config: cpal::StreamConfig = config.into();
    let channels = stream_config.channels as usize;

    let mut runner = GraphRunner::new(project);
    let mut frame_position = 0u64;

    let bpm = project.transport.bpm;

    // Create a dummy param bus and update channel for future integration
    let param_bus = Arc::new(ParamBus::new());
    let param_bus_audio = param_bus.clone();
    let (_tx, rx): (Sender<ParamUpdate>, Receiver<ParamUpdate>) = crossbeam_channel::bounded(1024);

    #[cfg(feature = "gui")]
    let scope = summoner_gui::visualizer::Oscilloscope::new();
    #[cfg(feature = "gui")]
    let spectrum = summoner_gui::visualizer::SpectrumAnalyzer::new();
    #[cfg(feature = "gui")]
    let _dft_handle = summoner_gui::visualizer::SpectrumAnalyzer::spawn_dft_thread(scope.clone(), spectrum.clone());
    #[cfg(feature = "gui")]
    let scope_cb = scope.clone();

    const MAX_BLOCK_SIZE: usize = 8192;
    let mut out_l = Box::new([0.0f32; MAX_BLOCK_SIZE]);
    let mut out_r = Box::new([0.0f32; MAX_BLOCK_SIZE]);

    let stream = device
        .build_output_stream(
            &stream_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let _alloc_guard = summoner_core::allocator::AllocGuard::new();

                // Apply any parameter updates
                while let Ok(update) = rx.try_recv() {
                    match update {
                        ParamUpdate::Set(id, val) => param_bus_audio.set(id, val),
                    }
                }

                let frames = (data.len() / channels).min(MAX_BLOCK_SIZE);
                
                let mut ctx = ProcessContext::new(sample_rate.0, bpm, frame_position);
                ctx.param_bus = Some(param_bus_audio.clone());

                let buf_l = &mut out_l[..frames];
                let buf_r = &mut out_r[..frames];
                buf_l.fill(0.0);
                buf_r.fill(0.0);
                
                // Process one block
                runner.process_block(frames, &ctx, &mut [buf_l, buf_r]);

                // Interleave the output
                for (i, frame) in data.chunks_mut(channels).enumerate() {
                    if i >= frames { break; }
                    let l = out_l[i];
                    let r = if channels > 1 { out_r[i] } else { out_l[i] };
                    
                    frame[0] = l;
                    if channels > 1 {
                        frame[1] = r;
                    }
                    #[cfg(feature = "gui")]
                    scope_cb.write_sample((l + r) * 0.5);
                }
                frame_position += frames as u64;
            },
            |err| eprintln!("an error occurred on stream: {}", err),
            None, // None means no timeout for the callback
        )
        .expect("Failed to build output stream");

    stream.play().expect("Failed to start audio stream");

    // Park the thread to keep the stream alive
    loop {
        thread::park();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_project::create_default_project;
    use crate::graph::GraphRunner;
    use summoner_core::allocator::AllocGuard;

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

        assert!(result.is_ok(), "Audio callback block processing triggered heap allocation!");
    }
}
