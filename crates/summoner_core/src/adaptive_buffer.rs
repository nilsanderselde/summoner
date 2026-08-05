// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Adaptive Buffer Size Auto-Scaling for Sub-Millisecond Roundtrip Latency Optimization (Step 1263).

use std::time::Duration;

/// Adaptive buffer size scaler for dynamic latency optimization.
#[derive(Debug, Clone)]
pub struct AdaptiveBufferScaler {
    /// Minimum allowed buffer size in frames.
    pub min_buffer_size: usize,
    /// Maximum allowed buffer size in frames.
    pub max_buffer_size: usize,
    /// Current dynamic buffer size in frames.
    pub current_buffer_size: usize,
    /// Target sample rate in Hz.
    pub sample_rate: u32,
    /// Target roundtrip latency in milliseconds.
    pub target_latency_ms: f32,
    underrun_count: u64,
    success_blocks: u64,
    cpu_load_percent: f32,
}

impl AdaptiveBufferScaler {
    /// Create a new adaptive buffer scaler for the given sample rate and initial buffer size.
    pub fn new(sample_rate: u32, initial_buffer_size: usize) -> Self {
        Self {
            min_buffer_size: 16, // ~0.33 ms @ 48kHz for sub-millisecond latency
            max_buffer_size: 1024,
            current_buffer_size: initial_buffer_size.clamp(16, 1024),
            sample_rate: sample_rate.max(8000),
            target_latency_ms: 1.0,
            underrun_count: 0,
            success_blocks: 0,
            cpu_load_percent: 15.0,
        }
    }

    /// Record a processed audio block's duration and underrun status to adjust buffer scale.
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
                self.cpu_load_percent = 0.9 * self.cpu_load_percent
                    + 0.1 * ((actual_time_sec / block_time_sec) * 100.0);
            }

            // Auto-scale down if CPU load is low and stable for 50 consecutive blocks
            if self.success_blocks > 50
                && self.cpu_load_percent < 40.0
                && self.current_buffer_size > self.min_buffer_size
            {
                self.current_buffer_size = (self.current_buffer_size / 2).max(self.min_buffer_size);
                self.success_blocks = 0;
            }
        }
    }

    /// Get current roundtrip latency in milliseconds.
    pub fn current_latency_ms(&self) -> f32 {
        (self.current_buffer_size as f32 / self.sample_rate as f32) * 1000.0
    }

    /// Get total count of recorded buffer underruns.
    pub fn underrun_count(&self) -> u64 {
        self.underrun_count
    }

    /// Check if current roundtrip latency is sub-millisecond (< 1.0 ms).
    pub fn is_sub_millisecond(&self) -> bool {
        self.current_latency_ms() < 1.0
    }
}
