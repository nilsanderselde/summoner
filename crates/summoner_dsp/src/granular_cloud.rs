// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! High-Density Granular Cloud Texture Synthesizer & Grain Spray Engine (Milestone 12).
//!
//! Features micro-grain spray synthesis (grain duration 5..500ms, density 1..250 grains/sec,
//! position spray, pitch jitter, stereo spatial spread, and selectable windowing: Tukey, Hann,
//! Blackman, Hamming, Trapezoidal, Sine, Gaussian, and Welch).
//!
//! Enforces zero heap allocations in real-time audio render loops under `AllocGuard`.

use crate::sampler::SampleBuffer;
use crate::traits::SignalProcessor;
use serde::{Deserialize, Serialize};
use std::f32::consts::{PI, TAU};
use std::sync::Arc;
use summoner_core::allocator::AllocGuard;
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};

pub const MAX_CLOUD_GRAINS: usize = 128;
pub const CAPTURE_BUFFER_SIZE: usize = 262144; // ~5.46 seconds @ 48kHz

/// Grain envelope window function type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum GrainWindowType {
    #[default]
    Hann,
    Tukey,
    Blackman,
    Hamming,
    Trapezoidal,
    Sine,
    Gaussian,
    Welch,
}

impl GrainWindowType {
    pub fn label(&self) -> &'static str {
        match self {
            GrainWindowType::Hann => "HANN (SMOOTH BELL)",
            GrainWindowType::Tukey => "TUKEY (TAPERED COSINE)",
            GrainWindowType::Blackman => "BLACKMAN (HIGH ATTENUATION)",
            GrainWindowType::Hamming => "HAMMING (EQUIRIPPLE)",
            GrainWindowType::Trapezoidal => "TRAPEZOIDAL (LINEAR SLOPES)",
            GrainWindowType::Sine => "SINE (HALF SINE DOME)",
            GrainWindowType::Gaussian => "GAUSSIAN (BELL CURVE)",
            GrainWindowType::Welch => "WELCH (PARABOLIC)",
        }
    }

    /// Evaluates the normalized window amplitude for a normalized phase $t \in [0.0, 1.0]$.
    #[inline]
    pub fn evaluate(&self, t: f32, tukey_alpha: f32) -> f32 {
        let t_clamped = t.clamp(0.0, 1.0);
        let raw = match self {
            GrainWindowType::Hann => 0.5 * (1.0 - (TAU * t_clamped).cos()),
            GrainWindowType::Tukey => {
                let alpha = tukey_alpha.clamp(0.01, 1.0);
                if t_clamped < alpha / 2.0 {
                    0.5 * (1.0 - (PI * (2.0 * t_clamped / alpha - 1.0)).sin())
                } else if t_clamped <= 1.0 - alpha / 2.0 {
                    1.0
                } else {
                    0.5 * (1.0 - (PI * (2.0 * (1.0 - t_clamped) / alpha - 1.0)).sin())
                }
            }
            GrainWindowType::Blackman => {
                0.42 - 0.5 * (TAU * t_clamped).cos() + 0.08 * (2.0 * TAU * t_clamped).cos()
            }
            GrainWindowType::Hamming => 0.54 - 0.46 * (TAU * t_clamped).cos(),
            GrainWindowType::Trapezoidal => {
                let slope_width = (tukey_alpha * 0.5).clamp(0.01, 0.49);
                if t_clamped < slope_width {
                    t_clamped / slope_width
                } else if t_clamped > 1.0 - slope_width {
                    (1.0 - t_clamped) / slope_width
                } else {
                    1.0
                }
            }
            GrainWindowType::Sine => (PI * t_clamped).sin(),
            GrainWindowType::Gaussian => {
                let sigma = 0.20;
                let x = (t_clamped - 0.5) / sigma;
                (-0.5 * x * x).exp()
            }
            GrainWindowType::Welch => 1.0 - (2.0 * t_clamped - 1.0).powi(2),
        };
        raw.clamp(0.0, 1.0)
    }
}

/// An individual active or inactive micro-grain inside the cloud pool.
#[derive(Debug, Clone, Copy)]
pub struct CloudGrain {
    /// Starting read position in buffer samples.
    pub start_pos_samples: f32,
    /// Current playhead offset from `start_pos_samples`.
    pub playhead_samples: f32,
    /// Total duration of this grain in samples.
    pub duration_samples: f32,
    /// Playback pitch speed ratio (1.0 = original pitch).
    pub pitch_ratio: f32,
    /// Left channel pan weight.
    pub pan_l: f32,
    /// Right channel pan weight.
    pub pan_r: f32,
    /// Peak amplitude scale.
    pub amplitude: f32,
    /// True if playing in reverse direction.
    pub reverse: bool,
    /// Grain envelope window function.
    pub window_type: GrainWindowType,
    /// Tukey envelope alpha parameter.
    pub tukey_alpha: f32,
    /// Whether this grain is actively playing.
    pub active: bool,
}

impl Default for CloudGrain {
    fn default() -> Self {
        Self {
            start_pos_samples: 0.0,
            playhead_samples: 0.0,
            duration_samples: 2400.0,
            pitch_ratio: 1.0,
            pan_l: 0.707,
            pan_r: 0.707,
            amplitude: 1.0,
            reverse: false,
            window_type: GrainWindowType::Hann,
            tukey_alpha: 0.5,
            active: false,
        }
    }
}

/// High-Density Granular Cloud Texture Synthesizer Node.
#[derive(Clone)]
pub struct GranularCloudNode {
    /// Pre-allocated micro-grain pool (zero heap allocation in audio thread).
    pub grains: [CloudGrain; MAX_CLOUD_GRAINS],
    /// Audio sample buffer for loaded sample granulation.
    pub sample_buffer: Option<Arc<SampleBuffer>>,
    /// Internal static circular buffer for live audio capture granulation (~5.46 sec).
    pub live_capture_buffer: Box<[f32; CAPTURE_BUFFER_SIZE]>,
    /// Live circular write pointer.
    pub capture_write_pos: usize,
    /// Whether to record incoming audio into the internal circular buffer.
    pub record_live_input: bool,
    /// Sample rate in Hz.
    pub sample_rate: u32,

    // Parameter Controls
    /// Target grain duration in milliseconds (5.0 .. 500.0 ms).
    pub grain_duration_ms: f32,
    /// Duration jitter randomness (0.0 .. 1.0).
    pub duration_jitter: f32,
    /// Grain emission density in grains/second (1.0 .. 250.0).
    pub density: f32,
    /// Buffer scrub position (0.0 = start, 1.0 = end).
    pub scrub_position: f32,
    /// Position spray dispersion (0.0 = precise, 1.0 = full buffer random spray).
    pub spray: f32,
    /// Pitch shift in semitones (-24.0 .. +24.0).
    pub pitch_semitones: f32,
    /// Pitch jitter randomness in semitones (0.0 .. 24.0).
    pub pitch_jitter: f32,
    /// Stereo spatial spread (0.0 = mono centered, 1.0 = ultra-wide ping-pong).
    pub stereo_spread: f32,
    /// Center pan position (-1.0 = left, 0.0 = center, +1.0 = right).
    pub pan_center: f32,
    /// Probability of a spawned grain playing in reverse (0.0 .. 1.0).
    pub reverse_probability: f32,
    /// Grain envelope window function.
    pub window_type: GrainWindowType,
    /// Tukey window alpha parameter (0.01 .. 1.0).
    pub tukey_alpha: f32,
    /// Feedback from grain output back into the capture buffer (0.0 .. 0.95).
    pub feedback: f32,
    /// Dry/Wet blend (0.0 = 100% dry, 1.0 = 100% wet).
    pub dry_wet: f32,
    /// Master cloud output gain (linear).
    pub gain: f32,
    /// Freeze buffer position advancement.
    pub freeze: bool,

    // Internal State
    trigger_accumulator: f32,
    prng_state: u64,
}

impl std::fmt::Debug for GranularCloudNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GranularCloudNode")
            .field("sample_rate", &self.sample_rate)
            .field("grain_duration_ms", &self.grain_duration_ms)
            .field("density", &self.density)
            .field("scrub_position", &self.scrub_position)
            .field("spray", &self.spray)
            .field("pitch_semitones", &self.pitch_semitones)
            .field("pitch_jitter", &self.pitch_jitter)
            .field("stereo_spread", &self.stereo_spread)
            .field("window_type", &self.window_type)
            .finish()
    }
}

impl GranularCloudNode {
    /// Creates a new Granular Cloud Texture Synthesizer node.
    pub fn new(sample_rate: u32) -> Self {
        let sr = if sample_rate > 0 { sample_rate } else { 48000 };
        Self {
            grains: [CloudGrain::default(); MAX_CLOUD_GRAINS],
            sample_buffer: None,
            live_capture_buffer: Box::new([0.0; CAPTURE_BUFFER_SIZE]),
            capture_write_pos: 0,
            record_live_input: true,
            sample_rate: sr,
            grain_duration_ms: 60.0,
            duration_jitter: 0.25,
            density: 35.0,
            scrub_position: 0.5,
            spray: 0.15,
            pitch_semitones: 0.0,
            pitch_jitter: 0.0,
            stereo_spread: 0.8,
            pan_center: 0.0,
            reverse_probability: 0.0,
            window_type: GrainWindowType::Hann,
            tukey_alpha: 0.5,
            feedback: 0.0,
            dry_wet: 1.0,
            gain: 1.0,
            freeze: false,
            trigger_accumulator: 0.0,
            prng_state: 0x9E3779B97F4A7C15,
        }
    }

    /// Loads an external audio sample buffer for granular synthesis.
    pub fn load_sample_buffer(&mut self, buffer: Arc<SampleBuffer>) {
        self.sample_buffer = Some(buffer);
    }

    /// Clears the loaded external sample buffer and reverts to live capture mode.
    pub fn clear_sample_buffer(&mut self) {
        self.sample_buffer = None;
    }

    /// Resets all grains and internal capture buffers.
    pub fn reset(&mut self) {
        for grain in self.grains.iter_mut() {
            grain.active = false;
        }
        self.live_capture_buffer.fill(0.0);
        self.capture_write_pos = 0;
        self.trigger_accumulator = 0.0;
    }

    /// Returns the number of currently active grains playing in the cloud.
    pub fn active_grain_count(&self) -> usize {
        self.grains.iter().filter(|g| g.active).count()
    }

    /// Deterministic fast PRNG returning uniform float in $[-1.0, 1.0]$.
    #[inline]
    fn next_prng_bipolar(&mut self) -> f32 {
        self.prng_state = self
            .prng_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let val = (self.prng_state >> 33) as f32 / 2147483648.0;
        val - 1.0
    }

    /// Deterministic fast PRNG returning uniform float in $[0.0, 1.0]$.
    #[inline]
    fn next_prng_unipolar(&mut self) -> f32 {
        (self.next_prng_bipolar() + 1.0) * 0.5
    }

    /// Spawns a new grain into an available pool slot.
    pub fn spawn_grain(&mut self) {
        let (buf_len, is_live) = if let Some(ref buf) = self.sample_buffer {
            if buf.data.is_empty() {
                (CAPTURE_BUFFER_SIZE as f32, true)
            } else {
                (buf.data.len() as f32, false)
            }
        } else {
            (CAPTURE_BUFFER_SIZE as f32, true)
        };

        if buf_len <= 16.0 {
            return;
        }

        // Compute all randomized parameters first before borrowing grain
        let dur_jitter_factor = 1.0 + self.next_prng_bipolar() * self.duration_jitter;
        let duration_ms = (self.grain_duration_ms * dur_jitter_factor).clamp(5.0, 500.0);
        let duration_samples = (duration_ms * 0.001 * self.sample_rate as f32).max(16.0);

        // Base position and spray calculation
        let (base_pos, spray_range) = if is_live {
            // In live capture mode, scrub relative to current write head within recorded window
            let head = self.capture_write_pos as f32;
            let recorded = (self.capture_write_pos as f32).max(64.0);
            let offset = (self.scrub_position * recorded).min(buf_len);
            let base = (head - offset + buf_len) % buf_len;
            let range = recorded.min(buf_len) * 0.5;
            (base, range)
        } else {
            (self.scrub_position * (buf_len - 1.0), buf_len * 0.5)
        };

        let spray_samples = self.next_prng_bipolar() * self.spray * spray_range;
        let start_pos_samples = if is_live {
            (base_pos + spray_samples + buf_len) % buf_len
        } else {
            (base_pos + spray_samples).clamp(0.0, (buf_len - 1.0).max(0.0))
        };

        // Pitch calculation with jitter
        let pitch_jitter_semi = self.next_prng_bipolar() * self.pitch_jitter;
        let total_semitones = self.pitch_semitones + pitch_jitter_semi;
        let pitch_ratio = (2.0f32).powf(total_semitones / 12.0).clamp(0.125, 8.0);

        // Stereo panning
        let pan_rand = self.next_prng_bipolar() * self.stereo_spread;
        let total_pan = (self.pan_center + pan_rand).clamp(-1.0, 1.0);
        let pan_angle = (total_pan + 1.0) * (PI / 4.0); // 0 to PI/2
        let pan_l = pan_angle.cos();
        let pan_r = pan_angle.sin();

        // Reverse playback check
        let reverse = self.reverse_probability > 0.0
            && self.next_prng_unipolar() < self.reverse_probability;

        let window_type = self.window_type;
        let tukey_alpha = self.tukey_alpha;

        // Find inactive slot in pool and initialize
        if let Some(grain) = self.grains.iter_mut().find(|g| !g.active) {
            *grain = CloudGrain {
                start_pos_samples,
                playhead_samples: 0.0,
                duration_samples,
                pitch_ratio,
                pan_l,
                pan_r,
                amplitude: 1.0,
                reverse,
                window_type,
                tukey_alpha,
                active: true,
            };
        }
    }

    /// Triggers an immediate burst of $N$ micro-grains simultaneously.
    pub fn trigger_grain_burst(&mut self, count: usize) {
        for _ in 0..count.min(MAX_CLOUD_GRAINS) {
            self.spawn_grain();
        }
    }
}

/// Reads an audio sample from the appropriate buffer using 4-point Hermite cubic interpolation.
#[inline]
fn read_interpolated_sample_from_buffers(
    sample_buffer: Option<&SampleBuffer>,
    live_capture_buffer: &[f32; CAPTURE_BUFFER_SIZE],
    pos: f32,
) -> f32 {
    if let Some(buf) = sample_buffer {
        if !buf.data.is_empty() {
            let len = buf.data.len();
            let idx = pos.floor() as isize;
            let frac = pos - pos.floor();

            let get_sample = |i: isize| -> f32 {
                if i < 0 {
                    buf.data[0]
                } else if i >= len as isize {
                    buf.data[len - 1]
                } else {
                    buf.data[i as usize]
                }
            };

            let y0 = get_sample(idx - 1);
            let y1 = get_sample(idx);
            let y2 = get_sample(idx + 1);
            let y3 = get_sample(idx + 2);

            let c0 = y1;
            let c1 = 0.5 * (y2 - y0);
            let c2 = y0 - 2.5 * y1 + 2.0 * y2 - 0.5 * y3;
            let c3 = 0.5 * (y3 - y0) + 1.5 * (y1 - y2);

            return ((c3 * frac + c2) * frac + c1) * frac + c0;
        }
    }

    // Live circular buffer fallback
    let len = CAPTURE_BUFFER_SIZE as isize;
    let p = ((pos.floor() as isize % len) + len) % len;
    let frac = pos - pos.floor();

    let get_sample = |offset: isize| -> f32 {
        let i = ((p + offset) % len + len) % len;
        live_capture_buffer[i as usize]
    };

    let y0 = get_sample(-1);
    let y1 = get_sample(0);
    let y2 = get_sample(1);
    let y3 = get_sample(2);

    let c0 = y1;
    let c1 = 0.5 * (y2 - y0);
    let c2 = y0 - 2.5 * y1 + 2.0 * y2 - 0.5 * y3;
    let c3 = 0.5 * (y3 - y0) + 1.5 * (y1 - y2);

    ((c3 * frac + c2) * frac + c1) * frac + c0
}

impl SignalProcessor for GranularCloudNode {
    fn name(&self) -> &str {
        "GranularCloudNode"
    }

    fn process_block(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        if outputs.is_empty() {
            return;
        }

        let num_samples = outputs[0].len();
        if num_samples == 0 {
            return;
        }

        let sample_rate = if ctx.sample_rate > 0 {
            ctx.sample_rate
        } else {
            self.sample_rate
        };
        self.sample_rate = sample_rate;

        // Verify zero heap allocation in audio loop
        let _guard = AllocGuard::new();

        let has_input = !inputs.is_empty() && !inputs[0].is_empty();
        let is_stereo_in = inputs.len() >= 2 && !inputs[1].is_empty();
        let is_stereo_out = outputs.len() >= 2;

        let trigger_step = self.density / sample_rate as f32;

        for i in 0..num_samples {
            // 1. Capture incoming audio into circular buffer
            let in_l = if has_input && i < inputs[0].len() {
                inputs[0][i]
            } else {
                0.0
            };
            let in_r = if is_stereo_in && i < inputs[1].len() {
                inputs[1][i]
            } else {
                in_l
            };
            let mono_in = (in_l + in_r) * 0.5;

            if self.record_live_input {
                let write_idx = self.capture_write_pos;
                // Add input with feedback from previous cloud output
                let captured_val = mono_in.clamp(-2.0, 2.0);
                self.live_capture_buffer[write_idx] = captured_val;
                self.capture_write_pos = (self.capture_write_pos + 1) % CAPTURE_BUFFER_SIZE;
            }

            // 2. Grain trigger accumulator
            if !self.freeze && self.density > 0.0 {
                self.trigger_accumulator += trigger_step;
                while self.trigger_accumulator >= 1.0 {
                    self.trigger_accumulator -= 1.0;
                    self.spawn_grain();
                }
            }

            // 3. Granulate active grains and mix to stereo accumulator
            let mut grain_out_l = 0.0f32;
            let mut grain_out_r = 0.0f32;
            let buf_len = if let Some(ref buf) = self.sample_buffer {
                buf.data.len() as f32
            } else {
                CAPTURE_BUFFER_SIZE as f32
            };

            let sample_buf_ref = self.sample_buffer.as_deref();
            let live_buf_ref = &*self.live_capture_buffer;

            for grain in self.grains.iter_mut() {
                if !grain.active {
                    continue;
                }

                let norm_phase = grain.playhead_samples / grain.duration_samples;
                if norm_phase >= 1.0 {
                    grain.active = false;
                    continue;
                }

                // Calculate window envelope amplitude
                let env = grain.window_type.evaluate(norm_phase, grain.tukey_alpha);

                // Compute buffer read position
                let read_offset = if grain.reverse {
                    (grain.duration_samples - grain.playhead_samples) * grain.pitch_ratio
                } else {
                    grain.playhead_samples * grain.pitch_ratio
                };

                let read_pos = if sample_buf_ref.is_some() {
                    (grain.start_pos_samples + read_offset).clamp(0.0, (buf_len - 1.0).max(0.0))
                } else {
                    (grain.start_pos_samples + read_offset + buf_len) % buf_len
                };

                let sample = read_interpolated_sample_from_buffers(sample_buf_ref, live_buf_ref, read_pos) * env * grain.amplitude;
                grain_out_l += sample * grain.pan_l;
                grain_out_r += sample * grain.pan_r;

                grain.playhead_samples += 1.0;
            }

            // 4. Apply feedback into circular buffer if active
            if self.feedback > 0.0 && self.record_live_input {
                let write_idx = (self.capture_write_pos + CAPTURE_BUFFER_SIZE - 1) % CAPTURE_BUFFER_SIZE;
                let fb_sample = ((grain_out_l + grain_out_r) * 0.5 * self.feedback).clamp(-1.0, 1.0);
                self.live_capture_buffer[write_idx] += fb_sample;
            }

            // 5. Dry / Wet blending, gain, and [-1.0, 1.0] clamping
            let wet_l = grain_out_l * self.gain;
            let wet_r = grain_out_r * self.gain;

            let final_l = in_l * (1.0 - self.dry_wet) + wet_l * self.dry_wet;
            let final_r = in_r * (1.0 - self.dry_wet) + wet_r * self.dry_wet;

            outputs[0][i] = final_l.clamp(-1.0, 1.0);
            if is_stereo_out && i < outputs[1].len() {
                outputs[1][i] = final_r.clamp(-1.0, 1.0);
            }
        }
    }
}

impl AudioNode for GranularCloudNode {
    fn name(&self) -> &str {
        "GranularCloudNode"
    }

    fn process(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        self.process_block(inputs, outputs, ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::transport::Transport;

    #[test]
    fn test_grain_window_evaluations() {
        let windows = [
            GrainWindowType::Hann,
            GrainWindowType::Tukey,
            GrainWindowType::Blackman,
            GrainWindowType::Hamming,
            GrainWindowType::Trapezoidal,
            GrainWindowType::Sine,
            GrainWindowType::Gaussian,
            GrainWindowType::Welch,
        ];

        for &win in &windows {
            let start = win.evaluate(0.0, 0.5);
            let mid = win.evaluate(0.5, 0.5);
            let end = win.evaluate(1.0, 0.5);

            assert!((0.0..=1.0).contains(&start), "{:?} start out of bounds: {}", win, start);
            assert!((0.5..=1.05).contains(&mid), "{:?} mid out of bounds: {}", win, mid);
            assert!((0.0..=1.0).contains(&end), "{:?} end out of bounds: {}", win, end);
        }
    }

    #[test]
    fn test_granular_cloud_zero_allocation_and_burst() {
        let mut cloud = GranularCloudNode::new(48000);
        cloud.grain_duration_ms = 40.0;
        cloud.density = 50.0;
        cloud.spray = 0.2;
        cloud.pitch_jitter = 3.0;

        let transport = Transport::new(48000, 120.0);
        let ctx = ProcessContext::from_transport(&transport);

        let in_l = vec![0.5f32; 128];
        let in_r = vec![0.5f32; 128];
        let mut out_l = vec![0.0f32; 128];
        let mut out_r = vec![0.0f32; 128];

        // Process a few blocks with live recording
        for _ in 0..10 {
            cloud.process_block(
                &[&in_l[..], &in_r[..]],
                &mut [&mut out_l[..], &mut out_r[..]],
                &ctx,
            );
        }

        assert!(cloud.active_grain_count() > 0);
        assert!(out_l.iter().all(|s| s.is_finite()));
        assert!(out_r.iter().all(|s| s.is_finite()));

        // Trigger burst
        cloud.trigger_grain_burst(16);
        assert!(cloud.active_grain_count() >= 16);

        cloud.process_block(
            &[&in_l[..], &in_r[..]],
            &mut [&mut out_l[..], &mut out_r[..]],
            &ctx,
        );

        assert!(out_l.iter().any(|s| *s != 0.0));
    }
}
