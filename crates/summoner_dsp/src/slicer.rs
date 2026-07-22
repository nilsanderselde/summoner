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

use crate::sampler::SampleBuffer;

/// A slice marker produced by the auto-slicer.
#[derive(Debug, Clone)]
pub struct SliceMarker {
    pub start_sample: usize,
    pub end_sample: usize,
}

/// Offline transient detection module (Mimic-style).
/// In a full build, this would use a small ONNX model to detect transients.
pub struct AutoSlicer {
    pub threshold: f32,
}

impl AutoSlicer {
    pub fn new(threshold: f32) -> Self {
        Self { threshold }
    }

    /// Processes a sample buffer offline and returns a list of slice markers.
    pub fn detect_slices(&self, buffer: &SampleBuffer, threshold: f32) -> Vec<SliceMarker> {
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
                if is_in_slice {
                    // Close previous slice
                    slices.push(SliceMarker {
                        start_sample: current_start,
                        end_sample: i,
                    });
                } else if !slices.is_empty() {
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
        }
        
        slices
    }
}
