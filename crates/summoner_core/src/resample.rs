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

//! High-quality Blackman-Harris windowed-sinc polyphase sample-rate converter (AIO-6).

use std::f64::consts::PI;

const TAPS: usize = 32;
const PHASES: usize = 256;

/// Evaluates the normalized `sin(pi * x) / (pi * x)` sinc kernel.
#[inline]
fn sinc(x: f64) -> f64 {
    if x.abs() < 1e-12 {
        1.0
    } else {
        let pix = PI * x;
        pix.sin() / pix
    }
}

/// Evaluates a 4-term Blackman-Harris window at normalized position `u in [0.0, 1.0]`.
#[inline]
fn blackman_harris(u: f64) -> f64 {
    let a0 = 0.35875;
    let a1 = 0.48829;
    let a2 = 0.14128;
    let a3 = 0.01168;
    let phi = 2.0 * PI * u;
    a0 - a1 * phi.cos() + a2 * (2.0 * phi).cos() - a3 * (3.0 * phi).cos()
}

/// Real-time safe multi-channel windowed-sinc polyphase resampler.
///
/// Pre-allocates the polyphase coefficient table and per-channel delay rings at
/// construction time; [`Self::process_channel`] performs zero heap allocations.
#[derive(Debug, Clone)]
pub struct SincResampler {
    src_rate: u32,
    dst_rate: u32,
    step: f64,
    filter_bank: Vec<f32>,
    rings: Vec<[f32; TAPS]>,
    ring_pos: Vec<usize>,
    frac_pos: Vec<f64>,
}

impl SincResampler {
    /// Construct a windowed-sinc resampler for `channels` channels from `src_rate` to `dst_rate`.
    pub fn new(src_rate: u32, dst_rate: u32, channels: usize) -> Self {
        let src = src_rate.max(1);
        let dst = dst_rate.max(1);
        let chs = channels.max(1);
        let step = src as f64 / dst as f64;
        // Scale cutoff below Nyquist when downsampling to prevent aliasing.
        let cutoff = (dst as f64 / src as f64).min(1.0) * 0.95;

        let mut filter_bank = vec![0.0f32; PHASES * TAPS];
        let half = (TAPS as f64) * 0.5 - 1.0;

        for p in 0..PHASES {
            let frac = p as f64 / PHASES as f64;
            let mut sum = 0.0f64;
            let mut row = [0.0f64; TAPS];

            for (t, slot) in row.iter_mut().enumerate() {
                let m = t as f64 - half;
                let x = frac - m;
                // Window spans [-TAPS/2, +TAPS/2] centered at the evaluation point
                let win_pos = ((TAPS as f64 * 0.5 - x) / TAPS as f64).clamp(0.0, 1.0);
                let w = blackman_harris(win_pos);
                let coeff = cutoff * sinc(cutoff * x) * w;
                *slot = coeff;
                sum += coeff;
            }

            let inv = if sum.abs() > 1e-12 { 1.0 / sum } else { 1.0 };
            for (t, &c) in row.iter().enumerate() {
                filter_bank[p * TAPS + t] = (c * inv) as f32;
            }
        }

        Self {
            src_rate: src,
            dst_rate: dst,
            step,
            filter_bank,
            rings: vec![[0.0f32; TAPS]; chs],
            ring_pos: vec![0usize; chs],
            frac_pos: vec![0.0f64; chs],
        }
    }

    pub fn src_rate(&self) -> u32 {
        self.src_rate
    }

    pub fn dst_rate(&self) -> u32 {
        self.dst_rate
    }

    pub fn reset(&mut self) {
        for r in &mut self.rings {
            r.fill(0.0);
        }
        self.ring_pos.fill(0);
        self.frac_pos.fill(0.0);
    }

    /// Resample a single channel block without heap allocation (`AllocGuard`-safe).
    ///
    /// Returns `(input_frames_consumed, output_frames_written)`.
    pub fn process_channel(
        &mut self,
        ch: usize,
        input: &[f32],
        output: &mut [f32],
    ) -> (usize, usize) {
        if ch >= self.rings.len() || output.is_empty() {
            return (0, 0);
        }
        if self.src_rate == self.dst_rate {
            let n = input.len().min(output.len());
            output[..n].copy_from_slice(&input[..n]);
            return (n, n);
        }

        let mut in_consumed = 0usize;
        let mut out_written = 0usize;

        while out_written < output.len() {
            while self.frac_pos[ch] >= 1.0 {
                if in_consumed >= input.len() {
                    return (in_consumed, out_written);
                }
                let rpos = self.ring_pos[ch];
                self.rings[ch][rpos] = input[in_consumed];
                self.ring_pos[ch] = (rpos + 1) % TAPS;
                in_consumed += 1;
                self.frac_pos[ch] -= 1.0;
            }

            let phase_idx = ((self.frac_pos[ch] * PHASES as f64) as usize).min(PHASES - 1);
            let base = phase_idx * TAPS;
            let rpos = self.ring_pos[ch];
            let mut acc = 0.0f32;
            for t in 0..TAPS {
                let idx = (rpos + t) % TAPS;
                acc += self.rings[ch][idx] * self.filter_bank[base + t];
            }

            output[out_written] = acc;
            out_written += 1;
            self.frac_pos[ch] += self.step;
        }

        (in_consumed, out_written)
    }
}

/// Offline high-quality windowed-sinc mono sample-rate conversion with zero group-delay shift.
pub fn resample_mono(input: &[f32], src_rate: u32, dst_rate: u32) -> Vec<f32> {
    if input.is_empty() || src_rate == 0 || dst_rate == 0 || src_rate == dst_rate {
        return input.to_vec();
    }

    let out_len = ((input.len() as f64) * (dst_rate as f64) / (src_rate as f64))
        .round()
        .max(1.0) as usize;
    let step = src_rate as f64 / dst_rate as f64;
    let cutoff = (dst_rate as f64 / src_rate as f64).min(1.0) * 0.95;
    let half_taps = (TAPS / 2) as isize;
    let in_len = input.len() as isize;

    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src_pos = (i as f64) * step;
        let center = src_pos.floor() as isize;
        let frac = src_pos - center as f64;

        let mut acc = 0.0f64;
        let mut norm = 0.0f64;

        for k in (-half_taps + 1)..=half_taps {
            let idx = center + k;
            let x = frac - k as f64;
            let win_pos = ((TAPS as f64 * 0.5 - x) / TAPS as f64).clamp(0.0, 1.0);
            let w = blackman_harris(win_pos) * cutoff * sinc(cutoff * x);
            let sample = if idx >= 0 && idx < in_len {
                input[idx as usize] as f64
            } else {
                0.0
            };
            acc += sample * w;
            norm += w;
        }

        out.push(if norm.abs() > 1e-12 {
            (acc / norm) as f32
        } else {
            0.0
        });
    }
    out
}

/// Offline high-quality windowed-sinc stereo sample-rate conversion.
pub fn resample_stereo(
    left: &[f32],
    right: &[f32],
    src_rate: u32,
    dst_rate: u32,
) -> (Vec<f32>, Vec<f32>) {
    (
        resample_mono(left, src_rate, dst_rate),
        resample_mono(right, src_rate, dst_rate),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::allocator::AllocGuard;
    use std::f32::consts::TAU;

    #[test]
    fn test_resample_identity_same_rate() {
        let input: Vec<f32> = (0..512).map(|i| (i as f32 * 0.05).sin()).collect();
        let output = resample_mono(&input, 48000, 48000);
        assert_eq!(input, output);
    }

    #[test]
    fn test_resample_length_ratios_44100_and_48000() {
        let src: Vec<f32> = vec![0.25; 4410];
        let up = resample_mono(&src, 44100, 48000);
        assert_eq!(up.len(), 4800);

        let down = resample_mono(&up, 48000, 44100);
        assert_eq!(down.len(), 4410);
    }

    #[test]
    fn test_resample_preserves_sine_frequency_and_amplitude() {
        let freq = 440.0f32;
        let src_rate = 44100u32;
        let dst_rate = 48000u32;
        let input: Vec<f32> = (0..4410)
            .map(|i| (TAU * freq * i as f32 / src_rate as f32).sin())
            .collect();

        let (out_l, out_r) = resample_stereo(&input, &input, src_rate, dst_rate);
        assert_eq!(out_l.len(), 4800);
        assert_eq!(out_l, out_r);

        // Compare interior away from boundary taper against analytical 440 Hz @ 48 kHz sine
        let max_err = (200..4600)
            .map(|i| {
                let expected = (TAU * freq * i as f32 / dst_rate as f32).sin();
                (out_l[i] - expected).abs()
            })
            .fold(0.0f32, f32::max);
        assert!(
            max_err < 0.01,
            "Windowed-sinc resampling error too high: {max_err}"
        );
    }

    #[test]
    fn test_streaming_sinc_resampler_zero_alloc_under_guard() {
        let mut resampler = SincResampler::new(44100, 48000, 2);
        let input = [0.5f32; 256];
        let mut output = [0.0f32; 512];

        // Warm up
        let _ = resampler.process_channel(0, &input, &mut output);

        let (consumed, written) = {
            let _guard = AllocGuard::new();
            resampler.process_channel(1, &input, &mut output)
        };
        assert!(consumed > 0);
        assert!(written > consumed);
        assert!(output[..written].iter().all(|s| s.is_finite()));
    }
}
