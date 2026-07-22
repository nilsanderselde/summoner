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

//! CLI entry point for Summoner DAW (`summon`).

pub mod export_clap;
pub mod graph;


use summoner_core::allocator::AllocGuard;
use summoner_core::audio::{FixedAudioBuffer, Sample};
use summoner_core::node::{AudioNode, GainNode, ProcessContext, SineOscillatorNode};
use summoner_core::pipeline::{MultiTenantRenderQueue, RenderJob};
use summoner_core::transport::Transport;
use summoner_core::wav::WavWriter;
use summoner_project::git_dag::GitSessionDag;
use summoner_project::{create_default_project, parse_project_toml, serialize_project_toml};
use std::env;
use std::fs;
use std::path::Path;
use std::process;

fn print_usage() {
    println!("Summoner DAW CLI (`summon`)");
    println!("Usage:");
    println!("  summon init [PATH]");
    println!("  summon render-stub [PROJECT_PATH] [--frames N]");
    println!("  summon render-wav [PROJECT_PATH] [OUTPUT_WAV_PATH] [--frames N]");
    println!("  summon render-batch [MANIFEST_PATH]");
    println!("  summon patch-export [PROJECT_PATH]");
    println!("  summon patch-export-clap [PROJECT_PATH] [OUTPUT_DIR]");
    println!("  summon commit-history [PROJECT_PATH]");
    println!("  summon play [PROJECT_PATH]");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    match args[1].as_str() {
        "init" => {
            let path_str = args.get(2).map(|s| s.as_str()).unwrap_or("summoner_session.toml");
            let project = create_default_project("New Session");
            let serialized = serialize_project_toml(&project).expect("Failed to serialize default project");
            fs::write(path_str, serialized).expect("Failed to write project file");
            println!("Initialized new Summoner session at: {}", path_str);
        }
        "patch-export" => {
            let path_str = args.get(2).map(|s| s.as_str()).unwrap_or("summoner_session.toml");
            if !Path::new(path_str).exists() {
                eprintln!("Error: Project file '{}' not found.", path_str);
                process::exit(1);
            }
            let content = fs::read_to_string(path_str).expect("Failed to read project file");
            let project = parse_project_toml(&content).expect("Failed to parse project TOML");
            let dag = GitSessionDag::new(project, "summon-cli");
            println!("{}", dag.export_patch());
        }
        "commit-history" => {
            let path_str = args.get(2).map(|s| s.as_str()).unwrap_or("summoner_session.toml");
            if !Path::new(path_str).exists() {
                eprintln!("Error: Project file '{}' not found.", path_str);
                process::exit(1);
            }
            let content = fs::read_to_string(path_str).expect("Failed to read project file");
            let project = parse_project_toml(&content).expect("Failed to parse project TOML");
            let dag = GitSessionDag::new(project, "summon-cli");

            println!("Git Micro-Commit History for: {}", path_str);
            for commit in dag.history() {
                println!("Commit: {} | Author: {} | Msg: {}", commit.id, commit.author, commit.message);
            }
        }
        "render-batch" => {
            println!("Starting Multi-Tenant Cloud Render Batch...");
            let mut queue = MultiTenantRenderQueue::new();
            queue.enqueue(RenderJob::new("job-alpha-001", "tenant-1", 2048, 44100, 120.0, 440.0));
            queue.enqueue(RenderJob::new("job-beta-002", "tenant-2", 4096, 48000, 128.0, 880.0));

            let results = queue.process_all();
            for res in results {
                println!(
                    "Job ID: {} | Tenant: {} | Frames: {} | Digest: {} | Status: Success",
                    res.job_id, res.tenant_id, res.frames_processed, &res.hash_digest[..12]
                );
            }
            println!("Batch rendering completed successfully.");
        }
        "render-wav" => {
            let proj_path = args.get(2).map(|s| s.as_str()).unwrap_or("summoner_session.toml");
            let out_wav_path = args.get(3).map(|s| s.as_str()).unwrap_or("output.wav");
            let mut num_frames: usize = 44100; // 1 second default

            let mut i = 4;
            while i < args.len() {
                if args[i] == "--frames" && i + 1 < args.len() {
                    num_frames = args[i + 1].parse().unwrap_or(44100);
                    i += 2;
                } else {
                    i += 1;
                }
            }

            if !Path::new(proj_path).exists() {
                eprintln!("Error: Project file '{}' not found. Run `summon init` first.", proj_path);
                process::exit(1);
            }

            let content = fs::read_to_string(proj_path).expect("Failed to read project file");
            let project = parse_project_toml(&content).expect("Failed to parse project TOML");

            println!("Rendering WAV file: '{}' -> '{}'", proj_path, out_wav_path);

            let mut transport = Transport::new(project.transport.sample_rate, project.transport.bpm);
            transport.play();

            let mut runner = graph::GraphRunner::new(&project);

            // Fallback for empty tracks
            let use_fallback = runner.tracks.is_empty() || runner.tracks.iter().all(|t| t.nodes.is_empty());
            let mut fallback_sine = SineOscillatorNode::new(440.0);
            let mut fallback_gain = GainNode::new(0.5);

            const CHANNELS: usize = 2;
            const BLOCK_SIZE: usize = 64;
            let mut mid_buffer = FixedAudioBuffer::<CHANNELS, BLOCK_SIZE>::new();
            let mut out_buffer = FixedAudioBuffer::<CHANNELS, BLOCK_SIZE>::new();

            let mut wav_writer = WavWriter::create(out_wav_path, project.transport.sample_rate, CHANNELS as u16)
                .expect("Failed to create WAV writer");

            let mut frames_processed: usize = 0;
            let mut interleaved = vec![0.0f32; BLOCK_SIZE * CHANNELS];

            while frames_processed < num_frames {
                let block_frames = (num_frames - frames_processed).min(BLOCK_SIZE);
                mid_buffer.set_active_frames(block_frames);
                out_buffer.set_active_frames(block_frames);

                mid_buffer.clear();
                out_buffer.clear();

                let ctx = ProcessContext::from_transport(&transport);

                {
                    let _guard = AllocGuard::new();
                    let mut out_slices = out_buffer.channels_mut_2();

                    if use_fallback {
                        let dummy_in: [&[Sample]; 0] = [];
                        let mut mid_slices = mid_buffer.channels_mut_2();
                        fallback_sine.process(&dummy_in, &mut mid_slices, &ctx);
                        let mid_ref = mid_buffer.channels_ref_2();
                        fallback_gain.process(&mid_ref, &mut out_slices, &ctx);
                    } else {
                        runner.process_block(block_frames, &ctx, &mut out_slices);
                    }
                }

                // Interleave samples and write file I/O outside real-time audio thread scope
                let ch0 = out_buffer.channel(0);
                let ch1 = out_buffer.channel(1);
                for f in 0..block_frames {
                    interleaved[f * 2] = ch0[f];
                    interleaved[f * 2 + 1] = ch1[f];
                }

                wav_writer
                    .write_interleaved_samples(&interleaved[..block_frames * CHANNELS])
                    .expect("Failed to write WAV samples");

                transport.advance_frames(block_frames as u64);
                frames_processed += block_frames;
            }


            wav_writer.finalize().expect("Failed to finalize WAV header");
            println!("Successfully rendered {} frames to '{}'", frames_processed, out_wav_path);
        }
        "render-stub" => {
            let path_str = args.get(2).map(|s| s.as_str()).unwrap_or("summoner_session.toml");
            let mut num_frames: usize = 1024;
            
            let mut i = 3;
            while i < args.len() {
                if args[i] == "--frames" && i + 1 < args.len() {
                    num_frames = args[i + 1].parse().unwrap_or(1024);
                    i += 2;
                } else {
                    i += 1;
                }
            }

            if !Path::new(path_str).exists() {
                eprintln!("Error: Project file '{}' not found. Run `summon init` first.", path_str);
                process::exit(1);
            }

            let content = fs::read_to_string(path_str).expect("Failed to read project file");
            let project = parse_project_toml(&content).expect("Failed to parse project TOML");

            println!("Loaded project: '{}' (BPM: {}, Sample Rate: {})",
                project.name, project.transport.bpm, project.transport.sample_rate);

            let mut transport = Transport::new(project.transport.sample_rate, project.transport.bpm);
            transport.play();

            // Set up test DSP chain (Sine -> Gain)
            let mut sine_node = SineOscillatorNode::new(440.0);
            let mut gain_node = GainNode::new(0.5);

            // Pre-allocate fixed frame buffer outside render loop (Zero Heap Allocation in processing loop)
            const CHANNELS: usize = 2;
            const BLOCK_SIZE: usize = 64;
            let mut in_buffer = FixedAudioBuffer::<CHANNELS, BLOCK_SIZE>::new();
            let mut mid_buffer = FixedAudioBuffer::<CHANNELS, BLOCK_SIZE>::new();
            let mut out_buffer = FixedAudioBuffer::<CHANNELS, BLOCK_SIZE>::new();

            let mut frames_processed: usize = 0;
            let mut sample_sum: f64 = 0.0;

            println!("Starting deterministic rendering run of {} frames...", num_frames);

            {
                // Enter zero-allocation guard scope to enforce real-time safety
                let _guard = AllocGuard::new();

                while frames_processed < num_frames {
                    let block_frames = (num_frames - frames_processed).min(BLOCK_SIZE);
                    in_buffer.set_active_frames(block_frames);
                    mid_buffer.set_active_frames(block_frames);
                    out_buffer.set_active_frames(block_frames);

                    in_buffer.clear();
                    mid_buffer.clear();
                    out_buffer.clear();

                    let ctx = ProcessContext::from_transport(&transport);

                    // Slice buffers for slice-based AudioNode API
                    let dummy_in: [&[Sample]; 0] = [];
                    let mut mid_slices = mid_buffer.channels_mut_2();

                    // Process sine oscillator node
                    sine_node.process(&dummy_in, &mut mid_slices, &ctx);

                    let mid_ref = mid_buffer.channels_ref_2();
                    let mut out_slices = out_buffer.channels_mut_2();

                    // Process gain node
                    gain_node.process(&mid_ref, &mut out_slices, &ctx);

                    // Accumulate checksum/sum for determinism verification
                    for ch in 0..CHANNELS {
                        for s in out_buffer.channel(ch) {
                            sample_sum += s.abs() as f64;
                        }
                    }

                    transport.advance_frames(block_frames as u64);
                    frames_processed += block_frames;
                }
            }

            println!("Rendering completed successfully!");
            println!("Processed Frames: {}", frames_processed);
            println!("Final Transport Position: {} frames ({:.3}s)", transport.frame_position, transport.seconds());
            println!("Output Energy Checksum: {:.6}", sample_sum);
        }
        "patch-export-clap" => {
            let path_str = args.get(2).map(|s| s.as_str()).unwrap_or("summoner_session.toml");
            let out_dir = args.get(3).map(|s| s.as_str()).unwrap_or(".");
            match export_clap::generate_clap_plugin(Path::new(path_str), Path::new(out_dir)) {
                Ok(msg) => println!("{}", msg),
                Err(e) => eprintln!("Failed to export CLAP plugin: {}", e),
            }
        }
        "play" => {
            let path_str = args.get(2).map(|s| s.as_str()).unwrap_or("summoner_session.toml");
            if !Path::new(path_str).exists() {
                eprintln!("Error: Project file '{}' not found.", path_str);
                process::exit(1);
            }
            // Scaffold: cpal initialization and real-time thread spawning
            println!("Initializing native hardware audio via CPAL...");
            println!("Playing project: {}", path_str);
            println!("(Mock: Running real-time audio thread)");
        }
        _ => {
            print_usage();
            process::exit(1);
        }
    }
}
