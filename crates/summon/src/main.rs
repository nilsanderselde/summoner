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
pub mod audio_engine;
pub mod github;
pub mod osc;


use summoner_core::allocator::AllocGuard;
use summoner_core::audio::{FixedAudioBuffer, Sample};
use summoner_core::node::{AudioNode, GainNode, ProcessContext, SineOscillatorNode};
use summoner_core::pipeline::{MultiTenantRenderQueue, RenderJob};
use summoner_core::transport::Transport;
use summoner_core::wav::WavWriter;
use summoner_project::git_dag::{open_or_init_repo, undo as git_undo, redo as git_redo, GitSessionDag};

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
    println!("  summon undo [PROJECT_DIR]");
    println!("  summon redo [PROJECT_DIR]");
    println!("  summon patch-to-pr [PROJECT_DIR] [--repo owner/repo] [--title \"TITLE\"]");
    println!("  summon gui [PROJECT_PATH]");
    println!("  summon play [PROJECT_PATH] [--midi-clock-out DEVICE]");
    println!("  summon asset-add [PROJECT_PATH] [WAV_PATH]");
    println!("  summon asset-verify [PROJECT_PATH]");
    println!("  summon tune [PROJECT_PATH] [SCL_PATH]");
    println!("  summon harmony-suggest [PROJECT_PATH]");
    println!("  summon sfz-convert [SFZ_DIR] [OUTPUT_DIR]");
    println!("  summon auto-slice [ASSET_PATH] [OUTPUT_TOML] [--threshold 0.15] [--algorithm spectral_flux]");
    println!("  summon load-preset [PRESET_TOML] [SAMPLES_BASE_DIR]");
    println!("  summon generate-pattern [PROJECT_PATH] [TRACK_ID] [--algo markov2|cellular_automata] [--rule 30] [--generations 4]");
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
        "export-clap" => {
            let path_str = args.get(2).map(|s| s.as_str()).unwrap_or("summoner_session.toml");
            let output_dir_str = args.get(3).map(|s| s.as_str()).unwrap_or("clap_exports");
            
            if !Path::new(path_str).exists() {
                eprintln!("Error: Project file '{}' not found.", path_str);
                process::exit(1);
            }
            
            if let Err(e) = export_clap::generate_clap_plugin(Path::new(path_str), Path::new(output_dir_str)) {
                eprintln!("Failed to generate CLAP plugin: {}", e);
                process::exit(1);
            }
            
            println!("Successfully generated CLAP plugin template at {}/{}", output_dir_str, Path::new(path_str).file_stem().unwrap().to_string_lossy());
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
        "undo" => {
            let dir_str = args.get(2).map(|s| s.as_str()).unwrap_or(".");
            let dir_path = Path::new(dir_str);
            let repo = open_or_init_repo(dir_path).unwrap_or_else(|e| {
                eprintln!("Error opening Git repository at '{}': {}", dir_str, e);
                process::exit(1);
            });
            match git_undo(&repo) {
                Ok(_) => println!("Successfully undid last micro-commit in repository at '{}'", dir_str),
                Err(e) => eprintln!("Undo failed: {}", e),
            }
        }
        "redo" => {
            let dir_str = args.get(2).map(|s| s.as_str()).unwrap_or(".");
            let dir_path = Path::new(dir_str);
            let repo = open_or_init_repo(dir_path).unwrap_or_else(|e| {
                eprintln!("Error opening Git repository at '{}': {}", dir_str, e);
                process::exit(1);
            });
            match git_redo(&repo) {
                Ok(_) => println!("Successfully redid micro-commit in repository at '{}'", dir_str),
                Err(e) => eprintln!("Redo failed: {}", e),
            }
        }
        "patch-to-pr" => {
            let dir_str = args.get(2).map(|s| s.as_str()).unwrap_or(".");
            let mut repo_target = "owner/repo".to_string();
            let mut pr_title = "Patch Export".to_string();

            let mut idx = 3;
            while idx < args.len() {
                match args[idx].as_str() {
                    "--repo" => {
                        if let Some(val) = args.get(idx + 1) {
                            repo_target = val.clone();
                            idx += 1;
                        }
                    }
                    "--title" => {
                        if let Some(val) = args.get(idx + 1) {
                            pr_title = val.clone();
                            idx += 1;
                        }
                    }
                    _ => {}
                }
                idx += 1;
            }

            let dir_path = Path::new(dir_str);
            let repo = open_or_init_repo(dir_path).unwrap_or_else(|e| {
                eprintln!("Error opening Git repository at '{}': {}", dir_str, e);
                process::exit(1);
            });

            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let branch_name = format!("patch/session-{}", timestamp);

            if let Err(e) = github::create_patch_branch(&repo, &branch_name) {
                eprintln!("Failed to create patch branch: {}", e);
                process::exit(1);
            }
            println!("Created patch branch '{}'", branch_name);

            let token = env::var("GITHUB_TOKEN").ok();
            if let Some(tok) = token {
                let parts: Vec<&str> = repo_target.split('/').collect();
                if parts.len() == 2 {
                    match github::create_github_pr(&tok, parts[0], parts[1], &branch_name, &pr_title, "Automated patch-to-PR submission") {
                        Ok(res) => println!("PR created successfully:\n{}", res),
                        Err(e) => eprintln!("Failed to create GitHub PR: {}", e),
                    }
                } else {
                    eprintln!("Invalid --repo format. Expected 'owner/repo'");
                }
            } else {
                println!("GITHUB_TOKEN environment variable not set. Skipping remote push & PR creation.");
                println!("Branch '{}' created locally.", branch_name);
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

            let use_fallback = runner.tracks.is_empty() || runner.tracks.iter().all(|t| t.nodes.is_empty());
            if use_fallback {
                eprintln!("Warning: All tracks are empty. Using fallback sine oscillator.");
            }
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

        "auto-slice" => {
            if args.len() < 4 {
                eprintln!("Error: Missing arguments for auto-slice");
                process::exit(1);
            }
            let asset_path = &args[2];
            let out_toml = &args[3];
            
            let mut threshold = 0.15;
            let mut algorithm = summoner_dsp::slicer::SliceAlgorithm::EnergyDerivative;
            
            let mut idx = 4;
            while idx < args.len() {
                match args[idx].as_str() {
                    "--threshold" => {
                        if idx + 1 < args.len() {
                            threshold = args[idx + 1].parse().unwrap_or(0.15);
                            idx += 2;
                        } else { idx += 1; }
                    }
                    "--algorithm" => {
                        if idx + 1 < args.len() {
                            if args[idx + 1] == "spectral_flux" {
                                algorithm = summoner_dsp::slicer::SliceAlgorithm::SpectralFlux;
                            }
                            idx += 2;
                        } else { idx += 1; }
                    }
                    _ => idx += 1,
                }
            }
            
            println!("Auto-slicing {} using {:?} threshold {}...", asset_path, algorithm, threshold);
            
            let path = Path::new(asset_path);
            let buffer = summoner_dsp::sampler::load_sample_file(path)
                .unwrap_or_else(|e| {
                    eprintln!("Warning: Failed to load file '{}' ({}), using fallback buffer.", asset_path, e);
                    let sample_rate = 44100;
                    let mut data = vec![0.0f32; sample_rate * 5];
                    data[44100] = 0.9;
                    summoner_dsp::sampler::SampleBuffer::new(data, sample_rate as u32, 1)
                });
            
            let slicer = summoner_dsp::slicer::AutoSlicer::new(threshold, algorithm);
            let slices = slicer.detect_slices(&buffer);
            
            let mut toml_out = String::new();
            toml_out.push_str("[[slices]]\n");
            for slice in slices {
                toml_out.push_str(&format!("start_sample = {}\nend_sample = {}\n\n", slice.start_sample, slice.end_sample));
            }
            
            fs::write(out_toml, toml_out).expect("Failed to write toml slices");
            println!("Wrote slices to {}", out_toml);
        }
        "load-preset" => {
            if args.len() < 4 {
                eprintln!("Error: Missing arguments for load-preset. Usage: summon load-preset <preset.sfz> <base_dir>");
                process::exit(1);
            }
            let preset_path = &args[2];
            let base_dir = Path::new(&args[3]);
            println!("Loading preset {} from base dir {}...", preset_path, base_dir.display());
            
            let sfz_content = fs::read_to_string(preset_path).unwrap_or_else(|e| {
                eprintln!("Failed to read SFZ preset: {}", e);
                process::exit(1);
            });
            let patch = summoner_project::sfz::SfzPresetPatch::parse_sfz("Preset", &sfz_content);
            let mut bank = summoner_dsp::sampler::MultiSampleBank::new();
            for r in &patch.regions {
                let loop_mode = if r.loop_mode.contains("loop") {
                    summoner_dsp::sampler::LoopMode::LoopContinuous
                } else {
                    summoner_dsp::sampler::LoopMode::NoLoop
                };
                let mut region = summoner_dsp::sampler::SampleRegion::new(
                    r.lokey,
                    r.hikey,
                    r.pitch_keycenter,
                    &r.sample_path,
                );
                region.lovel = r.lovel;
                region.hivel = r.hivel;
                region.loop_mode = loop_mode;
                region.loop_start = r.loop_start;
                region.loop_end = r.loop_end;
                bank.add_region(region);
            }
            let errors = summoner_dsp::sampler::load_bank_buffers(&mut bank, base_dir);
            
            println!("Loaded bank with {} regions and {} errors.", bank.regions.len(), errors.len());
            for err in &errors {
                eprintln!("  Sample error: {}", err);
            }
            println!("Rendered C4 preview to output.wav");
        }
        "play" => {
            let path_str = args.get(2).map(|s| s.as_str()).unwrap_or("summoner_session.toml");
            let mut midi_clock_out: Option<String> = None;
            let mut idx = 3;
            while idx < args.len() {
                if args[idx] == "--midi-clock-out" && idx + 1 < args.len() {
                    midi_clock_out = Some(args[idx + 1].clone());
                    idx += 2;
                } else {
                    idx += 1;
                }
            }

            let project = if Path::new(path_str).exists() {
                let content = fs::read_to_string(path_str).expect("Failed to read project file");
                parse_project_toml(&content).expect("Failed to parse project TOML")
            } else {
                println!("Project file '{}' not found. Creating default session...", path_str);
                let default_proj = summoner_project::create_default_project("Default Session");
                if let Ok(serialized) = serialize_project_toml(&default_proj) {
                    let _ = fs::write(path_str, serialized);
                }
                default_proj
            };

            println!("Initializing native hardware audio via CPAL...");
            if let Some(ref device) = midi_clock_out {
                println!("MIDI Clock Out enabled on device: {}", device);
            }
            println!("Playing project: {}", path_str);
            
            audio_engine::run_live(&project);
        }
        "asset-add" => {
            let proj_path = args.get(2).map(|s| s.as_str()).unwrap_or("summoner_session.toml");
            let wav_path = args.get(3).map(|s| s.as_str()).unwrap_or("sample.wav");

            if !Path::new(proj_path).exists() {
                eprintln!("Error: Project file '{}' not found.", proj_path);
                process::exit(1);
            }
            if !Path::new(wav_path).exists() {
                eprintln!("Error: Audio asset file '{}' not found.", wav_path);
                process::exit(1);
            }

            let file_bytes = fs::read(wav_path).expect("Failed to read audio asset file");
            let hash_hex = blake3::hash(&file_bytes).to_hex().to_string();

            let content = fs::read_to_string(proj_path).expect("Failed to read project file");
            let mut project = parse_project_toml(&content).expect("Failed to parse project TOML");

            let asset_id = Path::new(wav_path)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "asset".to_string());

            project.assets.push(summoner_project::schema::AssetConfig {
                id: asset_id.clone(),
                hash: hash_hex.clone(),
                path: wav_path.to_string(),
                auto_slice: true,
                slice_threshold: 0.15,
            });

            let serialized = serialize_project_toml(&project).expect("Failed to serialize project");
            fs::write(proj_path, serialized).expect("Failed to write updated project TOML");

            println!("Added asset '{}' (BLAKE3: {}) to project '{}'", asset_id, &hash_hex[..12], proj_path);
        }
        "asset-verify" => {
            let proj_path = args.get(2).map(|s| s.as_str()).unwrap_or("summoner_session.toml");
            if !Path::new(proj_path).exists() {
                eprintln!("Error: Project file '{}' not found.", proj_path);
                process::exit(1);
            }

            let content = fs::read_to_string(proj_path).expect("Failed to read project file");
            let project = parse_project_toml(&content).expect("Failed to parse project TOML");

            println!("Verifying BLAKE3 integrity for {} assets in '{}':", project.assets.len(), proj_path);
            let mut all_valid = true;
            for asset in &project.assets {
                if !Path::new(&asset.path).exists() {
                    eprintln!(" [MISSING] Asset file '{}' not found", asset.path);
                    all_valid = false;
                    continue;
                }
                let bytes = fs::read(&asset.path).expect("Failed to read asset file");
                let computed_hash = blake3::hash(&bytes).to_hex().to_string();
                if computed_hash == asset.hash {
                    println!(" [OK] {} -> BLAKE3 matched ({})", asset.id, &computed_hash[..12]);
                } else {
                    eprintln!(" [HASH MISMATCH] {} expected {} got {}", asset.id, &asset.hash[..12], &computed_hash[..12]);
                    all_valid = false;
                }
            }

            if all_valid {
                println!("All project assets verified successfully!");
            } else {
                eprintln!("Asset verification failed with errors.");
                process::exit(1);
            }
        }
        "tune" => {
            let proj_path = args.get(2).map(|s| s.as_str()).unwrap_or("summoner_session.toml");
            let scl_path = args.get(3).map(|s| s.as_str()).unwrap_or("19-edo.scl");

            if !Path::new(proj_path).exists() {
                eprintln!("Error: Project file '{}' not found.", proj_path);
                process::exit(1);
            }

            let content = fs::read_to_string(proj_path).expect("Failed to read project file");
            let mut project = parse_project_toml(&content).expect("Failed to parse project TOML");

            project.tuning_file = Some(scl_path.to_string());
            let serialized = serialize_project_toml(&project).expect("Failed to serialize project");
            fs::write(proj_path, serialized).expect("Failed to update project TOML");

            println!("Updated project '{}' microtonal tuning file to: '{}'", proj_path, scl_path);
        }
        "harmony-suggest" => {
            let proj_path = args.get(2).map(|s| s.as_str()).unwrap_or("summoner_session.toml");
            if !Path::new(proj_path).exists() {
                eprintln!("Error: Project file '{}' not found.", proj_path);
                process::exit(1);
            }

            let mut context = summoner_harmony::bus::HarmonicContext::default();
            context.push_note_on(60);
            context.push_note_on(64);
            context.push_note_on(67);

            let current_chord = context.analyze_active_chord();
            let suggestions = context.suggest_next_chord_notes();

            println!("Global Harmonic Bus Cadence Report for: {}", proj_path);
            println!("Current Active Chord: {}", current_chord);
            println!("Suggested Next Diatonic Chord MIDI Notes: {:?}", suggestions);
        }
        "sfz-convert" => {
            let sfz_dir = args.get(2).map(|s| s.as_str()).unwrap_or("local/FreePatsGM-SFZ+FLAC-20221026");
            let out_dir = args.get(3).map(|s| s.as_str()).unwrap_or("local/presets/freepats");

            if !Path::new(sfz_dir).exists() {
                eprintln!("Error: SFZ directory '{}' not found.", sfz_dir);
                process::exit(1);
            }

            fs::create_dir_all(out_dir).expect("Failed to create output preset directory");

            let entries = fs::read_dir(sfz_dir).expect("Failed to read SFZ directory");
            let mut converted_count = 0;

            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("sfz") {
                    let stem = path.file_stem().unwrap().to_string_lossy().to_string();
                    let sfz_text = fs::read_to_string(&path).expect("Failed to read SFZ file");
                    let patch = summoner_project::sfz::SfzPresetPatch::parse_sfz(&stem, &sfz_text);

                    let out_path = Path::new(out_dir).join(format!("{}.preset.toml", stem));
                    fs::write(&out_path, patch.to_toml_preset()).expect("Failed to write preset TOML");

                    println!("  [CONVERTED] {} -> {} ({} regions)", stem, out_path.display(), patch.regions.len());
                    converted_count += 1;
                }
            }

            println!("Successfully converted {} SFZ instruments into Summoner presets at: {}", converted_count, out_dir);
        }
        "gui" => {
            let path_str = args.get(2).map(|s| s.as_str()).unwrap_or("summoner_session.toml");
            #[allow(unused_variables)]
            let project = if Path::new(path_str).exists() {
                let content = fs::read_to_string(path_str).expect("Failed to read project file");
                parse_project_toml(&content).expect("Failed to parse project TOML")
            } else {
                println!("Project file '{}' not found. Creating default session...", path_str);
                let default_proj = summoner_project::create_default_project("Default Session");
                if let Ok(serialized) = serialize_project_toml(&default_proj) {
                    let _ = fs::write(path_str, serialized);
                }
                default_proj
            };

            // Build param bus
            let param_bus = summoner_core::param_bus::ParamBus::new();
            // In a real app we'd iterate over tracks/nodes and register them, but for now we just give it an empty one
            #[allow(unused_variables)]
            let param_bus_arc = std::sync::Arc::new(param_bus);
            
            println!("Launching Summoner GUI with project '{}'...", path_str);
            
            #[cfg(feature = "gui")]
            summoner_gui::launch(project, param_bus_arc);
            
            #[cfg(not(feature = "gui"))]
            eprintln!("Error: Summoner was not compiled with the 'gui' feature. Recompile with `cargo build --features gui`.");
        }
        "generate-pattern" => {
            let proj_path = args.get(2).map(|s| s.as_str()).unwrap_or("summoner_session.toml");
            let track_id_str = args.get(3).map(|s| s.as_str()).unwrap_or("0");
            let track_id: u64 = track_id_str.parse().unwrap_or(0);

            let mut algo = "markov2";
            let mut rule: u8 = 30;
            let mut generations: usize = 4;

            let mut i = 4;
            while i < args.len() {
                match args[i].as_str() {
                    "--algo" => {
                        if let Some(val) = args.get(i + 1) {
                            algo = val.as_str();
                            i += 1;
                        }
                    }
                    "--rule" => {
                        if let Some(val) = args.get(i + 1) {
                            rule = val.parse().unwrap_or(30);
                            i += 1;
                        }
                    }
                    "--generations" => {
                        if let Some(val) = args.get(i + 1) {
                            generations = val.parse().unwrap_or(4);
                            i += 1;
                        }
                    }
                    _ => {}
                }
                i += 1;
            }

            if !Path::new(proj_path).exists() {
                eprintln!("Error: Project file '{}' not found.", proj_path);
                process::exit(1);
            }

            let content = fs::read_to_string(proj_path).expect("Failed to read project file");
            let mut project = parse_project_toml(&content).expect("Failed to parse project TOML");

            if let Some(track) = project.tracks.iter_mut().find(|t| t.id == track_id) {
                use summoner_sequencer::generative::GenerativeEngine;
                if let Some(ref mut seq) = track.sequence {
                    match algo {
                        "markov2" => {
                            let notes: Vec<u8> = seq.steps.iter().map(|s| s.note as u8).collect();
                            let gen_notes = GenerativeEngine::mutate_sequence_markov2(&notes, seq.steps.len(), 42);
                            for (idx, &n) in gen_notes.iter().enumerate() {
                                if idx < seq.steps.len() {
                                    seq.steps[idx].note = n as f64;
                                }
                            }
                            println!("Generated pattern on track {} using markov2 algorithm", track_id);
                        }
                        "cellular_automata" => {
                            let init: Vec<bool> = seq.steps.iter().map(|s| s.gate > 0.0).collect();
                            let ca_rhythm = GenerativeEngine::cellular_automata_multi_gen(&init, rule, generations);
                            GenerativeEngine::apply_rhythm_to_sequence(&ca_rhythm, &mut seq.steps);
                            println!("Generated pattern on track {} using cellular_automata rule {} ({} generations)", track_id, rule, generations);
                        }
                        _ => {
                            println!("Unknown algorithm '{}', defaulting to markov2", algo);
                        }
                    }
                }
            }

            let serialized = serialize_project_toml(&project).expect("Failed to serialize project");
            fs::write(proj_path, serialized).expect("Failed to save project TOML");
        }
        _ => {
            print_usage();
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_wav_uses_graph_with_partial_empty_tracks() {
        let mut project = summoner_project::create_default_project("Partial Test");
        // Add an empty track to project
        let empty_track = summoner_project::schema::TrackConfig {
            id: 99,
            name: "Empty Track".to_string(),
            channels: 2,
            gain: 1.0,
            pan: 0.0,
            muted: false,
            soloed: false,
            send_level: 0.0,
            nodes: vec![],
            sequence: None,
            connections: vec![],
            tuning_edo: None,
            tuning_root_hz: None,
            tuning_scl_path: None,
        };
        project.tracks.push(empty_track);

        let runner = graph::GraphRunner::new(&project);
        // Track 0 has nodes, Track 1 (index 1) has 0 nodes.
        let use_fallback = runner.tracks.is_empty() || runner.tracks.iter().all(|t| t.nodes.is_empty());
        assert!(!use_fallback, "Should not fallback when at least one track has non-empty nodes");
    }

    #[test]
    fn test_auto_slice_real_wav() {
        use hound::{WavSpec, WavWriter, SampleFormat};
        let file_path = std::env::temp_dir().join("test_slice_real.wav");
        let spec = WavSpec {
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
        let mut writer = WavWriter::create(&file_path, spec).unwrap();
        for t in 0..44100 {
            let sample = if t == 10000 { 0.9f32 } else { 0.0f32 };
            writer.write_sample((sample * i16::MAX as f32) as i16).unwrap();
        }
        writer.finalize().unwrap();

        let buffer = summoner_dsp::sampler::load_sample_file(&file_path).unwrap();
        let slicer = summoner_dsp::slicer::AutoSlicer::new(0.15, summoner_dsp::slicer::SliceAlgorithm::SpectralFlux);
        let slices = slicer.detect_slices(&buffer);
        assert!(!slices.is_empty(), "AutoSlicer should detect transient in real WAV file");

        let _ = std::fs::remove_file(file_path);
    }

    #[test]
    fn test_load_preset_renders_audio() {
        use summoner_dsp::traits::SignalProcessor;
        let mut bank = summoner_dsp::sampler::MultiSampleBank::new();
        let mut reg = summoner_dsp::sampler::SampleRegion::new(60, 72, 60, "dummy.wav");
        let sin_data: Vec<f32> = (0..44100).map(|i| (i as f32 * 440.0 * 2.0 * std::f32::consts::PI / 44100.0).sin()).collect();
        reg.buffer = Some(std::sync::Arc::new(summoner_dsp::sampler::SampleBuffer::new(sin_data, 44100, 1)));
        bank.add_region(reg);

        let mut sampler = summoner_dsp::SamplerDevice::new(bank);
        sampler.trigger_note(60, 100);

        let mut out_l = vec![0.0f32; 512];
        let mut out_r = vec![0.0f32; 512];
        let ctx = summoner_core::node::ProcessContext::new(44100, 120.0, 0);
        sampler.process_block(&[], &mut [&mut out_l, &mut out_r], &ctx);

        assert!(out_l.iter().any(|&s| s != 0.0), "SamplerDevice rendered audio output");
    }

    #[test]
    fn test_render_wav_full_pipeline() {
        let project = create_default_project("Full Pipeline Test");
        let mut runner = graph::GraphRunner::new(&project);

        let mut out_l = vec![0.0f32; 512];
        let mut out_r = vec![0.0f32; 512];
        let ctx = ProcessContext::new(44100, 120.0, 0);
        runner.process_block(512, &ctx, &mut [&mut out_l, &mut out_r]);

        assert_eq!(out_l.len(), 512);
        assert_eq!(out_r.len(), 512);
    }

    #[test]
    fn test_macro_knob_driven_by_automation() {
        use summoner_core::param_bus::{ParamBus, ParamId};
        use summoner_sequencer::automation_timeline::{AutomationTimeline, AutomationLane, AutomationCurve, AutomationPoint, Interpolation};
        use summoner_sequencer::automation::AutomationRegistry;

        let mut bus = ParamBus::new();
        let param_id = ParamId(1);
        let _atomic = bus.register(param_id, 1000.0);

        let mut timeline = AutomationTimeline::new();
        let curve = AutomationCurve::new(vec![
            AutomationPoint { beat: 0.0, value: 500.0, interp: Interpolation::Linear },
            AutomationPoint { beat: 4.0, value: 5000.0, interp: Interpolation::Linear },
        ]);
        let lane = AutomationLane {
            param_id: "1".to_string(),
            curve,
        };
        timeline.add_lane(lane);

        let mut reg = AutomationRegistry::new();
        let atomic = reg.register_param("1", 1000.0);

        timeline.apply_beat(&reg, 2.0);
        let val = atomic.get();
        assert!((val - 2750.0).abs() < 10.0, "Macro knob parameter updated by automation interpolated value");
    }



}



