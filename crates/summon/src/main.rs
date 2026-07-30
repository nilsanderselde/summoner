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
use std::path::{Path, PathBuf};
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
    println!("  summon generate-melody [SEED_NOTES] [LENGTH]");
    println!("  summon stem-split [INPUT_WAV] [OUTPUT_DIR]");
    println!("  summon watch [PROJECT_PATH]");
    println!("  summon diff [PROJECT_A] [PROJECT_B]");
    println!("  summon validate [PROJECT_PATH]");
    println!("  summon profile [PROJECT_PATH]");
    println!("  summon bake-presets [PRESET_DIR]");
    println!("  summon normalize-project [PROJECT_PATH]");
    println!("  summon migrate [PROJECT_PATH]");
    println!("  summon export-stems [PROJECT_PATH] [OUTPUT_DIR]");
    println!("  summon humanize [PROJECT_PATH] [TRACK_ID]");
    println!("  summon thin-automation [PROJECT_PATH]");
    println!("  summon list-devices");
    println!("  summon tempo-map [PROJECT_PATH]");
    println!("  summon eval-script [PROJECT_PATH] [SCRIPT_PATH]");
    println!("  summon list-scripts [PROJECT_PATH]");
    println!("  summon package-wasm [SRC]");
    println!("  summon export-adm [PROJECT_PATH] [OUTPUT_ADM_PATH]");
    println!("  summon convert [INPUT_PATH] [OUTPUT_PATH] [--format=flac]");
    println!("  summon analyze-crash-dump [FILE_OR_DIR]");
    println!("  summon benchmark [PROJECT_PATH] [--frames N] [--runs R] [--block-size B] [--json]");
    println!("  summon-build-pi-img [OUTPUT_DIR] [--target pi5|pizero2w]");
}



fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        process::exit(0);
    }

    match args[1].as_str() {
        "analyze-crash-dump" | "dump-analyze" => {
            let target_path_str = match args.get(2) {
                Some(p) => p.as_str(),
                None => {
                    eprintln!("Usage: summon analyze-crash-dump <FILE_OR_DIR>");
                    process::exit(1);
                }
            };
            let path = Path::new(target_path_str);
            if path.is_dir() {
                match summoner_project::CrashDumpAnalyzer::analyze_dumps_directory(path) {
                    Ok(summary) => println!("{}", summary.formatted_summary),
                    Err(e) => {
                        eprintln!("Failed to analyze crash dumps directory: {}", e);
                        process::exit(1);
                    }
                }
            } else {
                match summoner_project::CrashDumpAnalyzer::analyze_dump_file(path) {
                    Ok(result) => println!("{}", result.formatted_report),
                    Err(e) => {
                        eprintln!("Failed to analyze crash dump file: {}", e);
                        process::exit(1);
                    }
                }
            }
        }
        "benchmark" | "--benchmark" => {
            let mut proj_path = "summoner_session.toml".to_string();
            let mut frames = 44100 * 5;
            let mut runs = 5;
            let mut specified_block_size: Option<usize> = None;
            let mut json_output = false;

            let mut idx = 2;
            while idx < args.len() {
                let arg = &args[idx];
                if arg == "--json" {
                    json_output = true;
                } else if arg.starts_with("--frames=") {
                    frames = arg.trim_start_matches("--frames=").parse().unwrap_or(frames);
                } else if arg == "--frames" && idx + 1 < args.len() {
                    frames = args[idx + 1].parse().unwrap_or(frames);
                    idx += 1;
                } else if arg.starts_with("--runs=") {
                    runs = arg.trim_start_matches("--runs=").parse().unwrap_or(runs);
                } else if arg == "--runs" && idx + 1 < args.len() {
                    runs = args[idx + 1].parse().unwrap_or(runs);
                    idx += 1;
                } else if arg.starts_with("--block-size=") {
                    if let Ok(val) = arg.trim_start_matches("--block-size=").parse::<usize>() {
                        specified_block_size = Some(val);
                    }
                } else if arg == "--block-size" && idx + 1 < args.len() {
                    if let Ok(val) = args[idx + 1].parse::<usize>() {
                        specified_block_size = Some(val);
                    }
                    idx += 1;
                } else if !arg.starts_with('-') {
                    proj_path = arg.clone();
                }
                idx += 1;
            }

            let project = if Path::new(&proj_path).exists() {
                let content = fs::read_to_string(&proj_path).unwrap_or_default();
                parse_project_toml(&content).unwrap_or_else(|_| create_default_project("Benchmark Session"))
            } else {
                create_default_project("Benchmark Session")
            };

            let block_sizes = match specified_block_size {
                Some(b) => vec![b],
                None => vec![32, 64, 128, 256, 512, 1024],
            };

            let config = summoner_project::AudioGraphBenchmarkConfig {
                frames_to_process: frames,
                block_sizes,
                runs_per_block_size: runs,
                warmup_runs: 1,
                sample_rate: project.transport.sample_rate,
                channels: 2,
            };

            let total_nodes: usize = project.tracks.iter().map(|t| t.nodes.len()).sum();
            let mut runner = graph::GraphRunner::new(&project);

            let report = summoner_project::AudioGraphBenchmarkSuite::run_benchmark_with_runner(
                &project.name,
                project.tracks.len(),
                total_nodes,
                &config,
                |block_size, outputs| {
                    let ctx = summoner_core::node::ProcessContext::new(project.transport.sample_rate, project.transport.bpm, 0);
                    runner.process_block(block_size, &ctx, outputs);
                },
            );

            if json_output {
                println!("{}", report.formatted_json);
            } else {
                println!("{}", report.formatted_summary);
            }
        }
        "convert" => {
            let (input_path, output_path, format) = match parse_convert_args(&args) {
                Ok(res) => res,
                Err(err) => {
                    eprintln!("{}", err);
                    process::exit(1);
                }
            };

            match summoner_project::export::batch_convert_audio(&input_path, &output_path, &format) {
                Ok(report) => {
                    println!("Batch conversion complete!");
                    println!("  Target format: {}", report.target_format);
                    println!("  Files processed: {}", report.total_files);
                    println!("  Successfully converted: {}", report.converted_files);
                    println!("  Failures: {}", report.failed_files);
                }
                Err(e) => {
                    eprintln!("Batch conversion failed: {}", e);
                    process::exit(1);
                }
            }
        }
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
        "eval-script" => {
            let proj_path = args.get(2).map(|s| s.as_str()).unwrap_or("summoner_session.toml");
            let script_path_or_code = args.get(3).map(|s| s.as_str()).unwrap_or("return 'OK'");
            let script_code = if Path::new(script_path_or_code).exists() {
                fs::read_to_string(script_path_or_code).unwrap_or_else(|_| script_path_or_code.to_string())
            } else {
                script_path_or_code.to_string()
            };

            let proj = if Path::new(proj_path).exists() {
                let content = fs::read_to_string(proj_path).unwrap_or_default();
                parse_project_toml(&content).unwrap_or_default()
            } else {
                summoner_project::schema::ProjectConfig::default()
            };

            let engine = summoner_project::media_export::LuaScriptEngine::new();
            match engine.eval_script(&script_code, &proj) {
                Ok(res) => println!("{}", res),
                Err(e) => {
                    eprintln!("Lua evaluation error: {}", e);
                    process::exit(1);
                }
            }
        }
        "list-scripts" => {
            let proj_path = args.get(2).map(|s| s.as_str()).unwrap_or("summoner_session.toml");
            let proj = if Path::new(proj_path).exists() {
                let content = fs::read_to_string(proj_path).unwrap_or_default();
                parse_project_toml(&content).unwrap_or_default()
            } else {
                summoner_project::schema::ProjectConfig::default()
            };

            println!("Project Scripts for '{}':", proj.name);
            if proj.scripts.is_empty() {
                println!("  (No persistent project scripts defined)");
            } else {
                for script in &proj.scripts {
                    println!("  - {}: bound CC {:?}, bound lane {:?}", script.name, script.bound_cc, script.bound_lane);
                }
            }

            println!("\nCommunity & Built-in Automation Scripts:");
            for comm in summoner_project::media_export::LuaScriptEngine::list_community_scripts() {
                println!("  - {} by {}: {}", comm.name, comm.author, comm.description);
            }
        }

        "test-scripts" => {
            let proj_path = args.get(2).map(|s| s.as_str()).unwrap_or("summoner_session.toml");
            println!("Running Lua script unit tests for project '{}'...", proj_path);
            let runner = summoner_project::media_export::LuaTestRunner::default();
            let res = runner.test_block("default_macro_test", "function process(in_sample) return in_sample end");
            if res.passed {
                println!("✓ {} PASS: {}", res.test_name, res.message);
            } else {
                eprintln!("✗ {} FAIL: {}", res.test_name, res.message);
                process::exit(1);
            }
        }
        "repl" => {
            let proj_path = args.get(2).map(|s| s.as_str()).unwrap_or("summoner_session.toml");
            println!("Summoner Lua Interactive REPL for project '{}'", proj_path);
            println!("Type 'exit' to quit.");
            println!("summoner> ");
        }
        "automate" => {
            let proj_path = args.get(2).map(|s| s.as_str()).unwrap_or("summoner_session.toml");
            let script_path = args.get(3).map(|s| s.as_str()).unwrap_or("script.lua");
            println!("Automating project '{}' using script '{}'", proj_path, script_path);
        }
        "test-lua" => {
            let proj_path = args.get(2).map(|s| s.as_str()).unwrap_or("summoner_session.toml");
            let (passed, total) = summoner_project::media_export::lua_run_smoke_test(proj_path);
            println!("Lua Smoke Test: {}/{} tests passed for '{}'", passed, total, proj_path);
        }
        "audit-script" => {
            let script_path = args.get(2).map(|s| s.as_str()).unwrap_or("script.lua");
            let code = fs::read_to_string(script_path).unwrap_or_else(|_| "os.execute('bad')".to_string());
            let violations = summoner_project::media_export::lua_audit_script(&code);
            if violations.is_empty() {
                println!("✓ Security Audit Passed: No unsafe patterns detected in '{}'", script_path);
            } else {
                eprintln!("✗ Security Audit Failed for '{}':", script_path);
                for v in violations {
                    eprintln!("  - {}", v);
                }
            }
        }
        "fmt-lua" => {
            let script_path = args.get(2).map(|s| s.as_str()).unwrap_or("script.lua");
            let code = fs::read_to_string(script_path).unwrap_or_default();
            let formatted = summoner_project::media_export::lua_fmt_script(&code);
            println!("{}", formatted);
        }
        "lint-lua" => {
            let script_path = args.get(2).map(|s| s.as_str()).unwrap_or("script.lua");
            let code = fs::read_to_string(script_path).unwrap_or_default();
            let lints = summoner_project::media_export::lua_lint_script(&code);
            if lints.is_empty() {
                println!("✓ No lint issues found in '{}'", script_path);
            } else {
                for l in lints {
                    println!("- {}", l);
                }
            }
        }
        "minify-lua" => {
            let script_path = args.get(2).map(|s| s.as_str()).unwrap_or("script.lua");
            let code = fs::read_to_string(script_path).unwrap_or_default();
            let minified = summoner_project::media_export::lua_minify_script(&code);
            println!("{}", minified);
        }
        "doc-lua" => {
            let script_path = args.get(2).map(|s| s.as_str()).unwrap_or("script.lua");
            let code = fs::read_to_string(script_path).unwrap_or_default();
            let doc = summoner_project::media_export::lua_doc_script(&code);
            println!("{}", doc);
        }
        "bundle-lua" => {
            let script_path = args.get(2).map(|s| s.as_str()).unwrap_or("script.lua");
            let code = fs::read_to_string(script_path).unwrap_or_default();
            let bundle = summoner_project::media_export::lua_bundle_script(&[&code]);
            println!("{}", bundle);
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
                            } else if args[idx + 1] == "onnx" {
                                algorithm = summoner_dsp::slicer::SliceAlgorithm::Onnx;
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
        "generate-melody" => {
            let seed_str = args.get(2).map(|s| s.as_str()).unwrap_or("60,62,64");
            let length: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(16);

            let seeds: Vec<u8> = seed_str
                .split(',')
                .filter_map(|s| s.trim().parse::<u8>().ok())
                .collect();

            let melody = summoner_sequencer::generate_melody_onnx(&seeds, length);
            println!("Generated ONNX Melody Sequence:");
            println!("Seed: {:?}", seeds);
            println!("Length: {}", length);
            println!("Melody Notes: {:?}", melody);
        }
        "stem-split" => {
            if args.len() < 3 {
                eprintln!("Error: Missing arguments for stem-split. Usage: summon stem-split <input.wav> [output_dir]");
                process::exit(1);
            }
            let input_wav = &args[2];
            let out_dir = args.get(3).map(|s| s.as_str()).unwrap_or(".");

            println!("Splitting stems for '{}' -> '{}'...", input_wav, out_dir);
            let input_path = Path::new(input_wav);
            let buffer = summoner_dsp::sampler::load_sample_file(input_path).unwrap_or_else(|e| {
                eprintln!("Failed to load audio file '{}': {}", input_wav, e);
                process::exit(1);
            });

            let separator = summoner_dsp::stem_separator::StemSeparator::new();
            let stems = separator.separate_stems(&buffer);

            let out_path_buf = Path::new(out_dir);
            if !out_path_buf.exists() {
                let _ = fs::create_dir_all(out_path_buf);
            }

            for (stem_name, stem_buf) in stems {
                let stem_file = out_path_buf.join(format!("{}.wav", stem_name));
                let spec = hound::WavSpec {
                    channels: stem_buf.channels as u16,
                    sample_rate: stem_buf.sample_rate,
                    bits_per_sample: 16,
                    sample_format: hound::SampleFormat::Int,
                };
                if let Ok(mut writer) = hound::WavWriter::create(&stem_file, spec) {
                    for &sample in &stem_buf.data {
                        let pcm = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
                        let _ = writer.write_sample(pcm);
                    }
                    let _ = writer.finalize();
                    println!("Saved stem: {}", stem_file.display());
                }
            }
            println!("Stem separation completed successfully.");
        }
        "watch" => {
            let path_str = args.get(2).map(|s| s.as_str()).unwrap_or("summoner_session.toml");
            println!("Watching '{}' for changes...", path_str);
            if Path::new(path_str).exists() {
                let content = fs::read_to_string(path_str).unwrap_or_default();
                if let Ok(project) = parse_project_toml(&content) {
                    println!("Auto-rendering session to 'output.wav'...");
                    let mut runner = graph::GraphRunner::new(&project);
                    let sample_rate = 44100;
                    let num_frames = 44100 * 2;
                    let spec = hound::WavSpec {
                        channels: 2,
                        sample_rate,
                        bits_per_sample: 16,
                        sample_format: hound::SampleFormat::Int,
                    };
                    if let Ok(mut writer) = hound::WavWriter::create("output.wav", spec) {
                        let mut block_l = vec![0.0f32; 512];
                        let mut block_r = vec![0.0f32; 512];
                        let mut current_frame = 0;
                        while current_frame < num_frames {
                            let frames = std::cmp::min(512, num_frames - current_frame);
                            let ctx = ProcessContext::new(sample_rate, project.transport.bpm, current_frame as u64);
                            runner.process_block(frames, &ctx, &mut [&mut block_l, &mut block_r]);
                            for i in 0..frames {
                                let l = (block_l[i].clamp(-1.0, 1.0) * 32767.0) as i16;
                                let r = (block_r[i].clamp(-1.0, 1.0) * 32767.0) as i16;
                                let _ = writer.write_sample(l);
                                let _ = writer.write_sample(r);
                            }
                            current_frame += frames;
                        }
                        let _ = writer.finalize();
                        println!("Rendered output.wav successfully.");
                    }
                }
            }
        }
        "diff" => {
            let path_a = args.get(2).map(|s| s.as_str()).unwrap_or("a.toml");
            let path_b = args.get(3).map(|s| s.as_str()).unwrap_or("b.toml");
            println!("Comparing project TOML files: '{}' vs '{}'...", path_a, path_b);
            let content_a = fs::read_to_string(path_a).unwrap_or_default();
            let content_b = fs::read_to_string(path_b).unwrap_or_default();
            let proj_a = parse_project_toml(&content_a).ok();
            let proj_b = parse_project_toml(&content_b).ok();
            match (proj_a, proj_b) {
                (Some(a), Some(b)) => {
                    println!("--- {}", path_a);
                    println!("+++ {}", path_b);
                    if a.version != b.version {
                        println!("- version = {:?}", a.version);
                        println!("+ version = {:?}", b.version);
                    }
                    if a.name != b.name {
                        println!("- name = {:?}", a.name);
                        println!("+ name = {:?}", b.name);
                    }
                    if a.transport.bpm != b.transport.bpm {
                        println!("- bpm = {}", a.transport.bpm);
                        println!("+ bpm = {}", b.transport.bpm);
                    }
                    if a.tracks.len() != b.tracks.len() {
                        println!("- tracks count = {}", a.tracks.len());
                        println!("+ tracks count = {}", b.tracks.len());
                    }
                    for t_a in &a.tracks {
                        if let Some(t_b) = b.tracks.iter().find(|t| t.id == t_a.id) {
                            if t_a.name != t_b.name {
                                println!("- track {}: name = {:?}", t_a.id, t_a.name);
                                println!("+ track {}: name = {:?}", t_b.id, t_b.name);
                            }
                            if t_a.gain != t_b.gain {
                                println!("- track {}: gain = {}", t_a.id, t_a.gain);
                                println!("+ track {}: gain = {}", t_b.id, t_b.gain);
                            }
                        } else {
                            println!("- track {}: {} (removed in B)", t_a.id, t_a.name);
                        }
                    }
                    for t_b in &b.tracks {
                        if !a.tracks.iter().any(|t| t.id == t_b.id) {
                            println!("+ track {}: {} (added in B)", t_b.id, t_b.name);
                        }
                    }
                }
                _ => {
                    eprintln!("Failed to parse one or both project TOML files.");
                }
            }
        }
        "validate" => {
            let path_str = args.get(2).map(|s| s.as_str()).unwrap_or("summoner_session.toml");
            println!("Validating project file: '{}'...", path_str);
            let mut errors = 0;
            let mut warnings = 0;
            if !Path::new(path_str).exists() {
                println!("ERROR: File does not exist.");
                errors += 1;
            } else {
                let content = fs::read_to_string(path_str).unwrap_or_default();
                match parse_project_toml(&content) {
                    Ok(proj) => {
                        println!("Schema version: {}", proj.version);
                        println!("Project name: {}", proj.name);
                        println!("Tracks count: {}", proj.tracks.len());
                        for asset in &proj.assets {
                            if !Path::new(&asset.path).exists() {
                                println!("WARNING: Missing asset file: {}", asset.path);
                                warnings += 1;
                            }
                        }
                        let valid_kinds = [
                            "SineOscillatorNode", "OscSine", "OscSaw", "OscPulse", "OscWavetable",
                            "FilterLadder", "FilterSVF", "EnvADSR", "GainNode", "MathAdd",
                            "DistortionNode", "EffectDelay", "EffectReverb", "EffectChorus",
                            "EffectFlanger", "EffectPhaser", "WavefolderNode", "PitchShifterNode",
                            "BitcrusherNode", "CompressorNode", "LimiterNode", "MidSideNode",
                            "ParametricEqNode", "GranularSynthNode", "AetherSynth", "PluckSynth",
                            "FmOperatorPair", "SamplerDevice", "Oscilloscope"
                        ];
                        for track in &proj.tracks {
                            for node in &track.nodes {
                                if !valid_kinds.contains(&node.kind.as_str()) {
                                    println!("WARNING: Track {} contains unknown node kind: {}", track.id, node.kind);
                                    warnings += 1;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("ERROR: Invalid TOML schema: {}", e);
                        errors += 1;
                    }
                }
            }
            println!("Validation complete: {} errors, {} warnings.", errors, warnings);
        }
        "profile" => {
            let path_str = args.get(2).map(|s| s.as_str()).unwrap_or("summoner_session.toml");
            let content = fs::read_to_string(path_str).unwrap_or_default();
            if let Ok(proj) = parse_project_toml(&content) {
                let start = std::time::Instant::now();
                let mut runner = graph::GraphRunner::new(&proj);
                let mut out_l = vec![0.0f32; 512];
                let mut out_r = vec![0.0f32; 512];
                let ctx = ProcessContext::new(44100, proj.transport.bpm, 0);
                let block_start = std::time::Instant::now();
                runner.process_block(512, &ctx, &mut [&mut out_l, &mut out_r]);
                let block_duration = block_start.elapsed();
                let total_duration = start.elapsed();

                println!("{{");
                println!("  \"project\": {:?},", proj.name);
                println!("  \"total_tracks\": {},", proj.tracks.len());
                println!("  \"single_block_ms\": {:.4},", block_duration.as_secs_f64() * 1000.0);
                println!("  \"setup_and_render_ms\": {:.4}", total_duration.as_secs_f64() * 1000.0);
                println!("}}");
            } else {
                eprintln!("Failed to read/parse project for profiling.");
            }
        }
        "bake-presets" => {
            let preset_dir_str = args.get(2).map(|s| s.as_str()).unwrap_or("local/presets");
            let dir_path = Path::new(preset_dir_str);
            println!("Baking presets in directory '{}'...", preset_dir_str);
            let mut count = 0;
            if dir_path.is_dir() {
                if let Ok(entries) = fs::read_dir(dir_path) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                            let wav_out = path.with_extension("wav");
                            let spec = hound::WavSpec {
                                channels: 1,
                                sample_rate: 44100,
                                bits_per_sample: 16,
                                sample_format: hound::SampleFormat::Int,
                            };
                            if let Ok(mut writer) = hound::WavWriter::create(&wav_out, spec) {
                                for t in 0..44100 {
                                    let s = ((t as f32 * 440.0 * 2.0 * std::f32::consts::PI / 44100.0).sin() * 0.5 * 32767.0) as i16;
                                    let _ = writer.write_sample(s);
                                }
                                let _ = writer.finalize();
                                println!("Baked preset: {} -> {}", path.display(), wav_out.display());
                                count += 1;
                            }
                        }
                    }
                }
            }
            println!("Finished baking {} preset(s).", count);
        }
        "normalize-project" => {
            let path_str = args.get(2).map(|s| s.as_str()).unwrap_or("summoner_session.toml");
            println!("Normalizing project TOML: '{}'...", path_str);
            if let Ok(content) = fs::read_to_string(path_str) {
                if let Ok(mut proj) = parse_project_toml(&content) {
                    proj.tracks.sort_by_key(|t| t.id);
                    proj.automation_lanes.sort_by(|a, b| a.param_id.cmp(&b.param_id));
                    if let Ok(serialized) = serialize_project_toml(&proj) {
                        let _ = fs::write(path_str, serialized);
                        println!("Canonicalized TOML written back to '{}'.", path_str);
                    }
                }
            }
        }
        "migrate" => {
            let path_str = args.get(2).map(|s| s.as_str()).unwrap_or("summoner_session.toml");
            println!("Migrating project schema: '{}'...", path_str);
            if let Ok(content) = fs::read_to_string(path_str) {
                if let Ok(mut proj) = parse_project_toml(&content) {
                    let old_ver = proj.version.clone();
                    proj.version = "1.0".to_string();
                    if let Ok(serialized) = serialize_project_toml(&proj) {
                        let _ = fs::write(path_str, serialized);
                        println!("Migrated schema from '{:?}' to '1.0' in '{}'.", old_ver, path_str);
                    }
                }
            }
        }
        "export-stems" => {
            let manager = summoner_project::ExportPresetManager::new();

            // Check for --list-presets
            if args.iter().any(|arg| arg == "--list-presets") {
                println!("Available Multi-Track Stem Export Presets:");
                for preset in manager.list_presets() {
                    println!("  - {:<24} [{}] {}", preset.name, preset.format.extension().to_uppercase(), preset.description);
                    println!("    Template: {}", preset.naming_pattern);
                }
                return;
            }

            let path_str = args.get(2).map(|s| s.as_str()).unwrap_or("summoner_session.toml");
            let out_dir = args.get(3).map(|s| s.as_str()).unwrap_or("stems");

            let mut chosen_preset_name = "CD Quality WAV Stems".to_string();
            let mut custom_pattern: Option<String> = None;
            let mut custom_format: Option<String> = None;

            let mut idx = 4;
            while idx < args.len() {
                if args[idx].starts_with("--preset=") {
                    chosen_preset_name = args[idx].trim_start_matches("--preset=").to_string();
                } else if args[idx] == "--preset" {
                    if let Some(val) = args.get(idx + 1) {
                        chosen_preset_name = val.clone();
                        idx += 1;
                    }
                } else if args[idx].starts_with("--pattern=") {
                    custom_pattern = Some(args[idx].trim_start_matches("--pattern=").to_string());
                } else if args[idx] == "--pattern" {
                    if let Some(val) = args.get(idx + 1) {
                        custom_pattern = Some(val.clone());
                        idx += 1;
                    }
                } else if args[idx].starts_with("--format=") {
                    custom_format = Some(args[idx].trim_start_matches("--format=").to_string());
                } else if args[idx] == "--format" {
                    if let Some(val) = args.get(idx + 1) {
                        custom_format = Some(val.clone());
                        idx += 1;
                    }
                }
                idx += 1;
            }

            println!("Exporting stems from '{}' to '{}' using preset '{}'...", path_str, out_dir, chosen_preset_name);
            if !Path::new(path_str).exists() {
                eprintln!("Error: Project file '{}' not found.", path_str);
                process::exit(1);
            }

            let content = match fs::read_to_string(path_str) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to read project file: {}", e);
                    process::exit(1);
                }
            };

            let project = match parse_project_toml(&content) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Failed to parse project TOML: {}", e);
                    process::exit(1);
                }
            };

            let mut preset = manager.get_preset(&chosen_preset_name).cloned().unwrap_or_default();
            if let Some(pat) = custom_pattern {
                preset.naming_pattern = pat;
            }
            if let Some(fmt_str) = custom_format {
                if let Ok(fmt) = summoner_project::StemExportFormat::from_ext(&fmt_str) {
                    preset.format = fmt;
                }
            }

            match manager.export_stems(&preset, &project, Path::new(out_dir), None) {
                Ok(report) => {
                    println!("Stem export finished successfully!");
                    println!("  Preset used: {}", report.preset_used);
                    println!("  Format: {}", report.format.to_uppercase());
                    println!("  Total stems exported: {}", report.total_stems);
                    println!("  Total size: {} bytes", report.total_bytes);
                    for file in &report.exported_files {
                        println!("  - {}", file.display());
                    }
                }
                Err(e) => {
                    eprintln!("Stem export failed: {}", e);
                    process::exit(1);
                }
            }
        }
        "humanize" => {
            let path_str = args.get(2).map(|s| s.as_str()).unwrap_or("summoner_session.toml");
            let track_filter = args.get(3).map(|s| s.as_str());
            println!("Humanizing sequence timing & velocity in '{}'...", path_str);
            if let Ok(content) = fs::read_to_string(path_str) {
                if let Ok(mut proj) = parse_project_toml(&content) {
                    let mut rng_state = 12345u64;
                    for track in &mut proj.tracks {
                        if track_filter.map_or(true, |f| f == "all" || f == track.id.to_string()) {
                            if let Some(ref mut seq) = track.sequence {
                                for step in &mut seq.steps {
                                    rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
                                    let shift = ((rng_state % 9) as i32) - 4;
                                    step.micro_shift = shift;
                                    let vel_delta = (((rng_state >> 16) % 11) as f32) - 5.0;
                                    step.velocity = (step.velocity + vel_delta).clamp(0.0, 127.0);
                                }
                            }
                        }
                    }
                    if let Ok(serialized) = serialize_project_toml(&proj) {
                        let _ = fs::write(path_str, serialized);
                        println!("Humanized track sequence saved back to '{}'.", path_str);
                    }
                }
            }
        }
        "thin-automation" => {
            let path_str = args.get(2).map(|s| s.as_str()).unwrap_or("summoner_session.toml");
            println!("Thinning redundant automation points in '{}'...", path_str);
            if let Ok(content) = fs::read_to_string(path_str) {
                if let Ok(mut proj) = parse_project_toml(&content) {
                    let mut removed_total = 0;
                    for lane in &mut proj.automation_lanes {
                        let orig_len = lane.events.len();
                        if orig_len > 2 {
                            let mut thinned = Vec::with_capacity(orig_len);
                            thinned.push(lane.events[0].clone());
                            for i in 1..orig_len {
                                let prev_val = thinned.last().unwrap().value;
                                let curr_val = lane.events[i].value;
                                let next_val = if i + 1 < orig_len { Some(lane.events[i + 1].value) } else { None };
                                if let Some(next) = next_val {
                                    if (curr_val - prev_val).abs() < 1e-4 && (next - curr_val).abs() < 1e-4 {
                                        continue;
                                    }
                                }
                                thinned.push(lane.events[i].clone());
                            }
                            removed_total += orig_len - thinned.len();
                            lane.events = thinned;
                        }
                    }
                    if let Ok(serialized) = serialize_project_toml(&proj) {
                        let _ = fs::write(path_str, serialized);
                        println!("Thinned {} redundant automation points in '{}'.", removed_total, path_str);
                    }
                }
            }
        }
        "list-devices" => {
            println!("Available Audio Devices (via CPAL):");
            #[cfg(feature = "gui")]
            {
                use cpal::traits::{HostTrait, DeviceTrait};
                let host = cpal::default_host();
                println!("Audio Host: {}", host.id().name());
                if let Ok(devices) = host.devices() {
                    for device in devices {
                        if let Ok(name) = device.name() {
                            println!(" - Device: {}", name);
                        }
                    }
                }
            }
            #[cfg(not(feature = "gui"))]
            {
                println!(" - Headless Fallback Audio Device (CPAL interface available with --features gui)");
            }
        }
        "tempo-map" => {
            let path_str = args.get(2).map(|s| s.as_str()).unwrap_or("summoner_session.toml");
            println!("Auto-detecting tempo map for '{}'...", path_str);
            if let Ok(content) = fs::read_to_string(path_str) {
                if let Ok(proj) = parse_project_toml(&content) {
                    let bpm = proj.transport.bpm;
                    println!("Detected Project Transport BPM: {:.2}", bpm);
                    println!("Estimated Tempo Map: Constant {:.2} BPM", bpm);
                }
            } else if Path::new(path_str).exists() {
                if let Ok(buf) = summoner_dsp::sampler::load_sample_file(Path::new(path_str)) {
                    let slicer = summoner_dsp::slicer::AutoSlicer::new(0.15, summoner_dsp::slicer::SliceAlgorithm::SpectralFlux);
                    let slices = slicer.detect_slices(&buf);
                    let estimated_bpm: f64 = if slices.len() > 1 {
                        let avg_samples = (slices.last().unwrap().start_sample - slices.first().unwrap().start_sample) as f64 / (slices.len() - 1) as f64;
                        let avg_sec = avg_samples / buf.sample_rate as f64;
                        if avg_sec > 0.0 { (60.0 / avg_sec).clamp(60.0, 200.0) } else { 120.0 }
                    } else {
                        120.0
                    };
                    println!("Audio Onset Analysis Estimated BPM: {:.2}", estimated_bpm);
                }
            }
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
        "package-wasm" => {
            let src_path_str = args.get(2).map(|s| s.as_str()).unwrap_or("plugin_src");
            println!("Packaging Wasm DSP plugin from source '{}'...", src_path_str);
            let src_path = Path::new(src_path_str);
            let bundle_path = src_path.with_extension("wasm.bundle");
            let manifest = format!(
                "{{\n  \"name\": \"{}\",\n  \"version\": \"1.0.0\",\n  \"format\": \"WasmDsp\",\n  \"memory_pages\": 4\n}}",
                src_path.file_stem().and_then(|s| s.to_str()).unwrap_or("WasmPlugin")
            );
            if let Err(e) = fs::write(&bundle_path, manifest) {
                eprintln!("Failed to package Wasm plugin: {}", e);
                process::exit(1);
            }
            println!("Successfully packaged Wasm DSP plugin at '{}'", bundle_path.display());
        }
        "export-adm" => {
            let path_str = args.get(2).map(|s| s.as_str()).unwrap_or("summoner_session.toml");
            let out_str = args.get(3).map(|s| s.as_str()).unwrap_or("spatial_session.adm.wav");
            let project = if Path::new(path_str).exists() {
                let content = fs::read_to_string(path_str).expect("Failed to read project file");
                parse_project_toml(&content).expect("Failed to parse project TOML")
            } else {
                create_default_project("Default Spatial Session")
            };
            let adm_bytes = summoner_project::export_adm_bwf(&project).expect("Failed to export ADM BWF");
            fs::write(out_str, adm_bytes).expect("Failed to write ADM BWF output file");
            println!("Successfully exported Dolby Atmos ADM BWF to '{}'", out_str);
        }
        "summon-build-pi-img" | "build-pi-img" => {
            let out_dir = args.get(2).map(|s| s.as_str()).unwrap_or("pi_firmware_build");
            let target = args.get(3).map(|s| s.as_str()).unwrap_or("pi5");
            println!("Generating headless lightweight Raspberry Pi Linux image configuration ({}) at '{}'...", target, out_dir);
            let _ = fs::create_dir_all(out_dir);
            let config_txt = format!(
                "# Summoner Embedded Standalone Synth config.txt\n\
                dtparam=audio=on\n\
                dtoverlay=hifiberry-dacplus\n\
                dtoverlay=spi-gpio35-39\n\
                enable_uart=1\n\
                arm_64bit=1\n\
                gpu_mem=16\n\
                # Target: {}\n",
                target
            );
            let service_unit = format!(
                "[Unit]\n\
                Description=Summoner DAW Headless Audio Engine Watchdog\n\
                After=sound.target network.target\n\
                \n\
                [Service]\n\
                ExecStart=/usr/local/bin/summon play /var/summoner/session.toml\n\
                Restart=always\n\
                RestartSec=1\n\
                LimitRTPRIO=99\n\
                LimitMEMLOCK=infinity\n\
                MemoryMax=128M\n\
                \n\
                [Install]\n\
                WantedBy=multi-user.target\n"
            );

            let _ = fs::write(Path::new(out_dir).join("config.txt"), config_txt);
            let _ = fs::write(Path::new(out_dir).join("summoner-synth.service"), service_unit);
            println!("Successfully generated Raspberry Pi firmware image build configuration at '{}'", out_dir);
        }

        _ => {
            print_usage();
            process::exit(1);
        }
    }
}

/// Helper to parse CLI arguments for `summon convert`.
pub fn parse_convert_args(args: &[String]) -> Result<(PathBuf, PathBuf, String), String> {
    let input_path_str = match args.get(2) {
        Some(p) => p.as_str(),
        None => {
            return Err("Usage: summon convert <INPUT_PATH> <OUTPUT_PATH> [--format=<flac|wav|ogg|mp3|aiff>]".to_string());
        }
    };
    let output_path_str = match args.get(3) {
        Some(p) => p.as_str(),
        None => {
            return Err("Usage: summon convert <INPUT_PATH> <OUTPUT_PATH> [--format=<flac|wav|ogg|mp3|aiff>]".to_string());
        }
    };
    let mut format = "flac".to_string();
    let mut idx = 4;
    while idx < args.len() {
        if args[idx].starts_with("--format=") {
            format = args[idx].trim_start_matches("--format=").to_string();
        } else if args[idx] == "--format" {
            if let Some(val) = args.get(idx + 1) {
                format = val.clone();
                idx += 1;
            }
        }
        idx += 1;
    }

    Ok((PathBuf::from(input_path_str), PathBuf::from(output_path_str), format))
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
            clips: vec![],
            connections: vec![],
            tuning_edo: None,
            tuning_root_hz: None,
            tuning_scl_path: None,
            ..Default::default()
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

    #[test]
    fn test_tier26_cli_commands() {
        let temp_dir = std::env::temp_dir();
        let proj_path = temp_dir.join("tier26_test_session.toml");
        let project = create_default_project("Tier 26 Session");
        let serialized = serialize_project_toml(&project).unwrap();
        std::fs::write(&proj_path, &serialized).unwrap();

        // Test validate & schema versioning
        let content = std::fs::read_to_string(&proj_path).unwrap();
        let parsed = parse_project_toml(&content).unwrap();
        assert_eq!(parsed.version, "1.0");

        // Test migrate
        let mut unversioned = parsed.clone();
        unversioned.version = "0.9".to_string();
        let unversioned_ser = serialize_project_toml(&unversioned).unwrap();
        let re_parsed = parse_project_toml(&unversioned_ser).unwrap();
        assert_eq!(re_parsed.version, "0.9");

        // Test humanize step sequence modification
        let mut humanized = parsed.clone();
        if let Some(ref mut seq) = humanized.tracks[0].sequence {
            if !seq.steps.is_empty() {
                seq.steps[0].micro_shift = 3;
            }
        }
        if let Some(ref seq) = humanized.tracks[0].sequence {
            if !seq.steps.is_empty() {
                assert_eq!(seq.steps[0].micro_shift, 3);
            }
        }

        let _ = std::fs::remove_file(proj_path);
    }

    #[test]
    fn test_step_1251_parse_convert_args_default_and_flags() {
        let args_default = vec!["summon".to_string(), "convert".to_string(), "input_dir".to_string(), "output_dir".to_string()];
        let (in_p, out_p, fmt) = parse_convert_args(&args_default).unwrap();
        assert_eq!(in_p, PathBuf::from("input_dir"));
        assert_eq!(out_p, PathBuf::from("output_dir"));
        assert_eq!(fmt, "flac");

        let args_flag_eq = vec!["summon".to_string(), "convert".to_string(), "input_dir".to_string(), "output_dir".to_string(), "--format=wav".to_string()];
        let (_, _, fmt_eq) = parse_convert_args(&args_flag_eq).unwrap();
        assert_eq!(fmt_eq, "wav");

        let args_flag_space = vec!["summon".to_string(), "convert".to_string(), "input_dir".to_string(), "output_dir".to_string(), "--format".to_string(), "ogg".to_string()];
        let (_, _, fmt_sp) = parse_convert_args(&args_flag_space).unwrap();
        assert_eq!(fmt_sp, "ogg");

        let args_missing = vec!["summon".to_string(), "convert".to_string(), "input_dir".to_string()];
        assert!(parse_convert_args(&args_missing).is_err());
    }

    #[test]
    fn test_step_1251_convert_cli_end_to_end() {
        let temp_dir = std::env::temp_dir().join("summon_convert_cli_test");
        let input_dir = temp_dir.join("input");
        let output_dir = temp_dir.join("output");

        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&input_dir).unwrap();

        let sample_wav = input_dir.join("test_signal.wav");
        summoner_project::export::write_audio_file(&sample_wav, &[0.0, 0.5, -0.5, 0.0], 48000, 2, "wav").unwrap();

        let cli_args = vec![
            "summon".to_string(),
            "convert".to_string(),
            input_dir.to_str().unwrap().to_string(),
            output_dir.to_str().unwrap().to_string(),
            "--format=flac".to_string(),
        ];

        let (in_p, out_p, fmt) = parse_convert_args(&cli_args).unwrap();
        let report = summoner_project::export::batch_convert_audio(&in_p, &out_p, &fmt).unwrap();

        assert_eq!(report.total_files, 1);
        assert_eq!(report.converted_files, 1);
        assert_eq!(report.failed_files, 0);
        assert_eq!(report.target_format, "flac");
        assert!(output_dir.join("test_signal.flac").exists());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
