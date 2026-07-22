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

#![allow(clippy::all)]

use crate::sampler::SampleBuffer;

/// A slice marker produced by the auto-slicer.
#[derive(Debug, Clone)]
pub struct SliceMarker {
    pub start_sample: usize,
    pub end_sample: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SliceAlgorithm {
    EnergyDerivative,
    SpectralFlux,
}

/// Offline transient detection module (Mimic-style).
pub struct AutoSlicer {
    pub algorithm: SliceAlgorithm,
    pub threshold: f32,
}

impl AutoSlicer {
    pub fn new(threshold: f32, algorithm: SliceAlgorithm) -> Self {
        Self { threshold, algorithm }
    }

    pub fn detect_slices(&self, buffer: &SampleBuffer) -> Vec<SliceMarker> {
        self.detect_slices_algorithm(buffer, self.threshold, self.algorithm)
    }

    pub fn detect_slices_algorithm(&self, buffer: &SampleBuffer, threshold: f32, algorithm: SliceAlgorithm) -> Vec<SliceMarker> {
        match algorithm {
            SliceAlgorithm::EnergyDerivative => self.detect_energy_derivative(buffer, threshold),
            SliceAlgorithm::SpectralFlux => self.detect_spectral_flux(buffer, threshold),
        }
    }

    fn detect_energy_derivative(&self, buffer: &SampleBuffer, threshold: f32) -> Vec<SliceMarker> {
        let mut slices = Vec::new();
        if buffer.data.is_empty() {
            return slices;
        }
        
        let window_size = 512;
        let mut prev_energy = 0.0;
        let mut current_start = 0;
        let mut is_in_slice = false;
        
        let data = &buffer.data;
        
        for i in (0..data.len()).step_by(window_size) {
            let end = (i + window_size).min(data.len());
            let mut energy = 0.0;
            for j in i..end {
                energy += data[j] * data[j];
            }
            
            let derivative = energy - prev_energy;
            
            if derivative > threshold && !is_in_slice {
                if !slices.is_empty() {
                    if let Some(last) = slices.last_mut() {
                        if last.end_sample == 0 {
                            last.end_sample = i;
                        }
                    }
                }
                
                current_start = i;
                is_in_slice = true;
            } else if energy < threshold * 0.1 && is_in_slice {
                slices.push(SliceMarker {
                    start_sample: current_start,
                    end_sample: i,
                });
                is_in_slice = false;
            }
            
            prev_energy = energy;
        }
        
        if is_in_slice {
            slices.push(SliceMarker {
                start_sample: current_start,
                end_sample: data.len(),
            });
        } else if !slices.is_empty() && slices.last().unwrap().end_sample == 0 {
             let last = slices.last_mut().unwrap();
             last.end_sample = data.len();
        }
        
        slices
    }

    fn detect_spectral_flux(&self, buffer: &SampleBuffer, threshold: f32) -> Vec<SliceMarker> {
        let mut slices = Vec::new();
        if buffer.data.is_empty() {
            return slices;
        }
        
        let window_size = 2048;
        let hop_size = 512;
        let data = &buffer.data;
        
        if data.len() < window_size {
            return slices;
        }

        let mut prev_mags = vec![0.0; window_size / 2];
        let mut flux_curve = Vec::new();

        // 1. Compute spectral flux for all frames
        for start in (0..data.len().saturating_sub(window_size)).step_by(hop_size) {
            let mut real = vec![0.0; window_size];
            let mut imag = vec![0.0; window_size];
            
            // Hann window and copy
            for i in 0..window_size {
                let window = 0.5 * (1.0 - (2.0 * core::f32::consts::PI * i as f32 / (window_size - 1) as f32).cos());
                real[i] = data[start + i] * window;
            }

            Self::fft(&mut real, &mut imag);

            let mut flux = 0.0;
            for i in 0..window_size / 2 {
                let mag = (real[i] * real[i] + imag[i] * imag[i]).sqrt();
                let diff = mag - prev_mags[i];
                if diff > 0.0 {
                    flux += diff;
                }
                prev_mags[i] = mag;
            }
            
            flux_curve.push((start, flux));
        }

        // 2. Peak picking logic
        let mut i = 1;
        while i < flux_curve.len() - 1 {
            let (start, flux) = flux_curve[i];
            let prev_flux = flux_curve[i - 1].1;
            let next_flux = flux_curve[i + 1].1;
            
            if flux > threshold && flux > prev_flux && flux > next_flux {
                // It's a peak above threshold
                if let Some(last) = slices.last_mut() {
                    if last.end_sample == 0 {
                        last.end_sample = start; // Close previous slice
                    }
                }
                slices.push(SliceMarker {
                    start_sample: start,
                    end_sample: 0,
                });
                
                // Skip the next few frames to avoid double-triggering
                i += 4;
            } else {
                i += 1;
            }
        }
        
        if let Some(last) = slices.last_mut() {
            if last.end_sample == 0 {
                last.end_sample = data.len();
            }
        }
        
        slices
    }

    fn fft(real: &mut [f32], imag: &mut [f32]) {
        let n = real.len();
        let bits = n.trailing_zeros() as usize;

        for i in 0..n {
            let mut rev = 0;
            let mut temp = i;
            for _ in 0..bits {
                rev = (rev << 1) | (temp & 1);
                temp >>= 1;
            }
            if i < rev {
                real.swap(i, rev);
                imag.swap(i, rev);
            }
        }

        let mut len = 2;
        while len <= n {
            let half = len / 2;
            let angle = -2.0 * core::f32::consts::PI / (len as f32);
            let w_r = angle.cos();
            let w_i = angle.sin();

            for i in (0..n).step_by(len) {
                let mut curr_r = 1.0;
                let mut curr_i = 0.0;
                for j in 0..half {
                    let u_r = real[i + j];
                    let u_i = imag[i + j];
                    let v_r = real[i + j + half] * curr_r - imag[i + j + half] * curr_i;
                    let v_i = real[i + j + half] * curr_i + imag[i + j + half] * curr_r;

                    real[i + j] = u_r + v_r;
                    imag[i + j] = u_i + v_i;
                    real[i + j + half] = u_r - v_r;
                    imag[i + j + half] = u_i - v_i;

                    let next_r = curr_r * w_r - curr_i * w_i;
                    let next_i = curr_r * w_i + curr_i * w_r;
                    curr_r = next_r;
                    curr_i = next_i;
                }
            }
            len *= 2;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spectral_flux_transient_detection() {
        // Generate a 44100 Hz buffer with transients every 0.25s (11025 samples)
        let sample_rate = 44100;
        let num_samples = sample_rate * 2; // 2 seconds
        let mut data = vec![0.0; num_samples as usize];
        
        let transient_interval = 11025;
        for i in 1..8 {
            let idx = i * transient_interval;
            if idx < data.len() {
                // Insert a sharp transient (impulse + exponential decay noise)
                for j in 0..100 {
                    if idx + j < data.len() {
                        let decay = (-(j as f32) / 10.0).exp();
                        data[idx + j] = decay * 0.8;
                    }
                }
            }
        }
        
        let buffer = SampleBuffer {
            data,
            channels: 1,
            sample_rate,
        };
        
        let slicer = AutoSlicer::new(2.0, SliceAlgorithm::SpectralFlux);
        let slices = slicer.detect_slices(&buffer);
        
        // We expect 7 transients + the final slice
        assert!(slices.len() >= 7, "Expected at least 7 slices, found {}", slices.len());
        
        // Check if the slices roughly align with the transients
        for i in 1..=7 {
            let expected_start = i * transient_interval;
            let slice = &slices[i - 1];
            let diff = (slice.start_sample as i32 - expected_start as i32).abs();
            assert!(diff < 2048, "Slice {} start {} is too far from expected {}", i, slice.start_sample, expected_start);
        }
    }
}
