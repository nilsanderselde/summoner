// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Audio graph buffer processing throughput benchmark suite (Step 1250).

use serde::{Deserialize, Serialize};
use std::time::Instant;
use crate::schema::ProjectConfig;

/// Configuration for audio graph buffer processing benchmark runs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioGraphBenchmarkConfig {
    /// Total audio frames to process per benchmark run iteration.
    pub frames_to_process: usize,
    /// List of buffer block sizes to evaluate.
    pub block_sizes: Vec<usize>,
    /// Number of measured execution iterations per block size.
    pub runs_per_block_size: usize,
    /// Number of unmeasured warmup iterations before recording metrics.
    pub warmup_runs: usize,
    /// Sample rate of audio graph processing (e.g. 44100 Hz).
    pub sample_rate: u32,
    /// Number of audio channels (e.g. 2 for stereo).
    pub channels: usize,
}

impl Default for AudioGraphBenchmarkConfig {
    fn default() -> Self {
        Self {
            frames_to_process: 44100 * 5, // 5 seconds of audio @ 44.1kHz
            block_sizes: vec![32, 64, 128, 256, 512, 1024],
            runs_per_block_size: 5,
            warmup_runs: 1,
            sample_rate: 44100,
            channels: 2,
        }
    }
}

/// Performance throughput metrics for a single block size evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockSizePerformance {
    pub block_size: usize,
    pub total_frames: usize,
    pub avg_elapsed_ms: f64,
    pub min_elapsed_ms: f64,
    pub max_elapsed_ms: f64,
    pub stddev_ms: f64,
    pub realtime_factor: f64,
    pub samples_per_sec: f64,
    pub megabytes_per_sec: f64,
    pub avg_block_microseconds: f64,
    pub p95_block_microseconds: f64,
    pub checksum: f64,
}

/// Comprehensive benchmark suite execution report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioGraphBenchmarkReport {
    pub project_name: String,
    pub track_count: usize,
    pub node_count: usize,
    pub sample_rate: u32,
    pub channels: usize,
    pub total_frames_per_run: usize,
    pub audio_duration_seconds: f64,
    pub best_block_size: usize,
    pub peak_realtime_factor: f64,
    pub peak_throughput_mb_s: f64,
    pub block_results: Vec<BlockSizePerformance>,
    pub formatted_summary: String,
    pub formatted_json: String,
}

/// Audio graph buffer processing throughput benchmark suite runner.
pub struct AudioGraphBenchmarkSuite;

impl AudioGraphBenchmarkSuite {
    /// Benchmark arbitrary audio processing closure across configured block sizes.
    pub fn run_benchmark_with_runner<F>(
        project_name: &str,
        track_count: usize,
        node_count: usize,
        config: &AudioGraphBenchmarkConfig,
        mut process_block_fn: F,
    ) -> AudioGraphBenchmarkReport
    where
        F: FnMut(usize, &mut [&mut [f32]]),
    {
        let channels = config.channels.max(1);
        let audio_duration_sec = config.frames_to_process as f64 / config.sample_rate as f64;
        let mut block_results = Vec::new();

        for &block_size in &config.block_sizes {
            let bs = block_size.max(1);
            let mut scratch_buffers: Vec<Vec<f32>> = (0..channels)
                .map(|_| vec![0.0f32; bs])
                .collect();

            // 1. Warmup runs
            for _ in 0..config.warmup_runs {
                let mut processed = 0;
                while processed < config.frames_to_process {
                    let cur_block = (config.frames_to_process - processed).min(bs);
                    let mut slice_ptrs: Vec<&mut [f32]> = scratch_buffers
                        .iter_mut()
                        .map(|b| &mut b[..cur_block])
                        .collect();
                    process_block_fn(cur_block, &mut slice_ptrs);
                    processed += cur_block;
                }
            }

            // 2. Measured runs
            let mut run_durations_ms = Vec::with_capacity(config.runs_per_block_size);
            let mut block_latencies_us = Vec::new();
            let mut final_checksum = 0.0f64;

            for _ in 0..config.runs_per_block_size {
                let run_start = Instant::now();
                let mut processed = 0;
                let mut run_checksum = 0.0f64;

                while processed < config.frames_to_process {
                    let cur_block = (config.frames_to_process - processed).min(bs);
                    let block_start = Instant::now();

                    let mut slice_ptrs: Vec<&mut [f32]> = scratch_buffers
                        .iter_mut()
                        .map(|b| &mut b[..cur_block])
                        .collect();
                    process_block_fn(cur_block, &mut slice_ptrs);

                    let block_elapsed = block_start.elapsed();
                    block_latencies_us.push(block_elapsed.as_secs_f64() * 1_000_000.0);

                    for buf in &scratch_buffers {
                        for &sample in &buf[..cur_block] {
                            run_checksum += sample.abs() as f64;
                        }
                    }
                    processed += cur_block;
                }

                let run_elapsed = run_start.elapsed();
                run_durations_ms.push(run_elapsed.as_secs_f64() * 1000.0);
                final_checksum = run_checksum;
            }

            // Statistics calculation
            let n = run_durations_ms.len() as f64;
            let avg_ms = run_durations_ms.iter().sum::<f64>() / n;
            let min_ms = run_durations_ms.iter().cloned().fold(f64::INFINITY, f64::min);
            let max_ms = run_durations_ms.iter().cloned().fold(0.0f64, f64::max);

            let variance = run_durations_ms
                .iter()
                .map(|v| (v - avg_ms).powi(2))
                .sum::<f64>()
                / n;
            let stddev_ms = variance.sqrt();

            let avg_sec = (avg_ms / 1000.0).max(1e-9);
            let realtime_factor = audio_duration_sec / avg_sec;
            let samples_per_sec = (config.frames_to_process * channels) as f64 / avg_sec;
            let megabytes_per_sec = (samples_per_sec * 4.0) / (1024.0 * 1024.0);

            block_latencies_us.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let avg_block_us = if !block_latencies_us.is_empty() {
                block_latencies_us.iter().sum::<f64>() / block_latencies_us.len() as f64
            } else {
                0.0
            };
            let p95_idx = ((block_latencies_us.len() as f64 * 0.95) as usize).min(block_latencies_us.len().saturating_sub(1));
            let p95_block_us = block_latencies_us.get(p95_idx).copied().unwrap_or(0.0);

            block_results.push(BlockSizePerformance {
                block_size: bs,
                total_frames: config.frames_to_process,
                avg_elapsed_ms: avg_ms,
                min_elapsed_ms: min_ms,
                max_elapsed_ms: max_ms,
                stddev_ms,
                realtime_factor,
                samples_per_sec,
                megabytes_per_sec,
                avg_block_microseconds: avg_block_us,
                p95_block_microseconds: p95_block_us,
                checksum: final_checksum,
            });
        }

        // Determine best block size based on real-time speed factor
        let (best_block_size, peak_realtime, peak_mb_s) = block_results
            .iter()
            .max_by(|a, b| a.realtime_factor.partial_cmp(&b.realtime_factor).unwrap_or(std::cmp::Ordering::Equal))
            .map(|r| (r.block_size, r.realtime_factor, r.megabytes_per_sec))
            .unwrap_or((64, 0.0, 0.0));

        let formatted_summary = Self::format_cli_table(
            project_name,
            track_count,
            node_count,
            config,
            audio_duration_sec,
            &block_results,
            best_block_size,
            peak_realtime,
            peak_mb_s,
        );

        let report_tmp = AudioGraphBenchmarkReport {
            project_name: project_name.to_string(),
            track_count,
            node_count,
            sample_rate: config.sample_rate,
            channels: config.channels,
            total_frames_per_run: config.frames_to_process,
            audio_duration_seconds: audio_duration_sec,
            best_block_size,
            peak_realtime_factor: peak_realtime,
            peak_throughput_mb_s: peak_mb_s,
            block_results: block_results.clone(),
            formatted_summary: formatted_summary.clone(),
            formatted_json: String::new(),
        };

        let formatted_json = serde_json::to_string_pretty(&report_tmp).unwrap_or_default();

        AudioGraphBenchmarkReport {
            formatted_json,
            ..report_tmp
        }
    }

    /// Benchmark project graph directly using built-in synthetic signal generator.
    pub fn run_benchmark_on_project(
        project: &ProjectConfig,
        config: &AudioGraphBenchmarkConfig,
    ) -> AudioGraphBenchmarkReport {
        let total_nodes: usize = project.tracks.iter().map(|t| t.nodes.len()).sum();

        let mut phase = 0.0f32;
        let freq = 440.0f32;
        let sr = config.sample_rate as f32;

        Self::run_benchmark_with_runner(
            &project.name,
            project.tracks.len(),
            total_nodes,
            config,
            move |block_size, outputs| {
                let phase_step = 2.0 * std::f32::consts::PI * freq / sr;
                for i in 0..block_size {
                    let sample = (phase).sin() * 0.5;
                    phase += phase_step;
                    if phase > 2.0 * std::f32::consts::PI {
                        phase -= 2.0 * std::f32::consts::PI;
                    }
                    for ch in outputs.iter_mut() {
                        ch[i] = sample;
                    }
                }
            },
        )
    }

    fn format_cli_table(
        project_name: &str,
        track_count: usize,
        node_count: usize,
        config: &AudioGraphBenchmarkConfig,
        audio_duration_sec: f64,
        block_results: &[BlockSizePerformance],
        best_block_size: usize,
        peak_realtime: f64,
        peak_mb_s: f64,
    ) -> String {
        let mut out = String::new();
        out.push_str("================================================================================\n");
        out.push_str("SUMMONER AUDIO GRAPH BUFFER PROCESSING THROUGHPUT BENCHMARK\n");
        out.push_str("================================================================================\n");
        out.push_str(&format!(
            "Project: {} | Tracks: {} | Nodes: {} | SR: {} Hz | Channels: {}\n",
            project_name, track_count, node_count, config.sample_rate, config.channels
        ));
        out.push_str(&format!(
            "Audio Duration: {:.2}s ({} frames) | Runs per block size: {}\n",
            audio_duration_sec, config.frames_to_process, config.runs_per_block_size
        ));
        out.push_str("--------------------------------------------------------------------------------\n");
        out.push_str("Block Size | Avg Time (ms) | Speed Factor | Throughput     | MB/s    | P95 (us)\n");
        out.push_str("--------------------------------------------------------------------------------\n");

        for r in block_results {
            let throughput_str = if r.samples_per_sec >= 1_000_000.0 {
                format!("{:.1} MS/s", r.samples_per_sec / 1_000_000.0)
            } else {
                format!("{:.1} KS/s", r.samples_per_sec / 1_000.0)
            };

            out.push_str(&format!(
                "  {:^8} | {:^13.3} | {:^12.1}x | {:^14} | {:^7.1} | {:^8.2}\n",
                r.block_size,
                r.avg_elapsed_ms,
                r.realtime_factor,
                throughput_str,
                r.megabytes_per_sec,
                r.p95_block_microseconds
            ));
        }

        out.push_str("--------------------------------------------------------------------------------\n");
        out.push_str(&format!(
            "Peak Throughput: {:.1}x Real-Time @ Block Size {} ({:.1} MB/s)\n",
            peak_realtime, best_block_size, peak_mb_s
        ));
        out.push_str("================================================================================\n");
        out
    }
}
