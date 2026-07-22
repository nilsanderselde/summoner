// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde

use crate::graph::GraphRunner;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use summoner_core::node::ProcessContext;
use summoner_core::transport::Transport;

pub fn play_graph(runner: GraphRunner, sample_rate: u32) -> Result<(), Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let device = host.default_output_device().ok_or("No default output device")?;
    
    let config = device.default_output_config()?;
    
    let runner = Arc::new(Mutex::new(runner));
    let channels = config.channels() as usize;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();
    
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => {
            let runner = Arc::clone(&runner);
            device.build_output_stream(
                &config.into(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut r = runner.lock().unwrap();
                    let frames = data.len() / channels;
                    
                    let mut out_left = vec![0.0; frames];
                    let mut out_right = vec![0.0; frames];
                    let mut buffers: Vec<&mut [f32]> = vec![&mut out_left, &mut out_right];
                    
                    let ctx = ProcessContext::from_transport(&transport);
                    r.process_block(frames, &ctx, &mut buffers);
                    transport.advance_frames(frames as u64);
                    
                    for (i, frame) in data.chunks_mut(channels).enumerate() {
                        frame[0] = out_left[i];
                        if channels > 1 {
                            frame[1] = out_right[i];
                        }
                    }
                },
                err_fn,
                None,
            )?
        },
        _ => return Err("Unsupported format".into()),
    };
    
    stream.play()?;
    
    // Keep alive or something, normally we would wait here
    std::thread::sleep(std::time::Duration::from_secs(5));
    
    Ok(())
}
