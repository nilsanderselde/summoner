// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Adaptive Buffer Size Auto-Scaling for Sub-Millisecond Roundtrip Latency Optimization (Step 1263).

use std::time::Duration;

/// Adaptive buffer size scaler for dynamic latency optimization.
#[derive(Debug, Clone)]
pub struct AdaptiveBufferScaler {
    pub min_buffer_size: usize,
    pub max_buffer_size: usize,
    pub current_buffer_size: usize,
    pub sample_rate: u32,
    pub target_latency_ms: f32,
    underrun_count: u64,
    success_blocks: u64,
    cpu_load_percent: f32,
}

impl AdaptiveBufferScaler {
    pub fn new(sample_rate: u32, initial_buffer_size: usize) -> Self {
        Self {
            min_buffer_size: 16,  // ~0.33 ms @ 48kHz for sub-millisecond latency
            max_buffer_size: 1024,
            current_buffer_size: initial_buffer_size.clamp(16, 1024),
            sample_rate: sample_rate.max(8000),
            target_latency_ms: 1.0,
            underrun_count: 0,
            success_blocks: 0,
            cpu_load_percent: 15.0,
        }
    }

    pub fn record_block_processing(&mut self, duration: Duration, underrun: bool) {
        if underrun {
            self.underrun_count += 1;
            self.success_blocks = 0;
            // Scale up buffer to prevent dropouts
            self.current_buffer_size = (self.current_buffer_size * 2).min(self.max_buffer_size);
        } else {
            self.success_blocks += 1;
            let block_time_sec = self.current_buffer_size as f32 / self.sample_rate as f32;
            let actual_time_sec = duration.as_secs_f32();
            if block_time_sec > 0.0 {
                self.cpu_load_percent = 0.9 * self.cpu_load_percent + 0.1 * ((actual_time_sec / block_time_sec) * 100.0);
            }

            // Auto-scale down if CPU load is low and stable for 50 consecutive blocks
            if self.success_blocks > 50 && self.cpu_load_percent < 40.0 && self.current_buffer_size > self.min_buffer_size {
                self.current_buffer_size = (self.current_buffer_size / 2).max(self.min_buffer_size);
                self.success_blocks = 0;
            }
        }
    }

    pub fn current_latency_ms(&self) -> f32 {
        (self.current_buffer_size as f32 / self.sample_rate as f32) * 1000.0
    }

    pub fn underrun_count(&self) -> u64 {
        self.underrun_count
    }

    pub fn is_sub_millisecond(&self) -> bool {
        self.current_latency_ms() < 1.0
    }
}
