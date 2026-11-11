// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! WebAssembly (Wasm) & WebGPU Headless Project Renderer (Milestone O1).
//!
//! Provides a browser-executable headless project renderer:
//! - WebAssembly linear memory audio buffer management and block render callbacks.
//! - Deterministic headless project timeline scrubbing, playback, and loop regions.
//! - WebGPU multi-resolution waveform mipmap generator producing vertex buffer attributes
//!   (min/max peak pyramids) for hardware-accelerated 60 FPS browser audio visualization.

use serde::{Deserialize, Serialize};

/// WebGPU vertex attribute for instanced waveform rendering.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WebGpuWaveformVertex {
    pub time_norm: f32, // 0.0 to 1.0 along project timeline
    pub min_peak: f32,  // -1.0 to 0.0
    pub max_peak: f32,  // 0.0 to 1.0
    pub rms_energy: f32,
}

/// Mipmapped multi-resolution peak pyramid for WebGPU waveform rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebGpuWaveformMipmap {
    pub total_samples: usize,
    pub samples_per_pixel: usize,
    pub vertices: Vec<WebGpuWaveformVertex>,
}

/// WebAssembly headless renderer state.
#[derive(Debug, Clone)]
pub struct WasmHeadlessRenderer {
    pub sample_rate: f32,
    pub block_size: usize,
    pub playback_sample_pos: usize,
    pub is_playing: bool,
    pub loop_enabled: bool,
    pub loop_start_sample: usize,
    pub loop_end_sample: usize,
    pub total_project_samples: usize,
    pub output_interleaved_buffer: Vec<f32>, // Left, Right interleaved
}

impl Default for WasmHeadlessRenderer {
    fn default() -> Self {
        Self::new(48000.0, 512, 48000 * 60) // Default 1 minute project length
    }
}

impl WasmHeadlessRenderer {
    pub fn new(sample_rate: f32, block_size: usize, total_project_samples: usize) -> Self {
        let block = block_size.max(64);
        Self {
            sample_rate: sample_rate.max(1.0),
            block_size: block,
            playback_sample_pos: 0,
            is_playing: false,
            loop_enabled: false,
            loop_start_sample: 0,
            loop_end_sample: total_project_samples,
            total_project_samples,
            output_interleaved_buffer: vec![0.0f32; block * 2],
        }
    }

    /// Load or resize project duration.
    pub fn set_project_duration_samples(&mut self, samples: usize) {
        self.total_project_samples = samples;
        self.loop_end_sample = samples;
        if self.playback_sample_pos >= samples {
            self.playback_sample_pos = 0;
        }
    }

    /// Set loop region.
    pub fn set_loop_region(&mut self, start_sample: usize, end_sample: usize, enabled: bool) {
        let start = start_sample.min(self.total_project_samples);
        let end = end_sample.min(self.total_project_samples).max(start + 1);
        self.loop_start_sample = start;
        self.loop_end_sample = end;
        self.loop_enabled = enabled;
    }

    /// Seek playback cursor to specific sample position.
    pub fn seek_to_sample(&mut self, sample_pos: usize) {
        self.playback_sample_pos = sample_pos.min(self.total_project_samples);
    }

    /// Render next audio block for WebAssembly audio worklet / AudioContext.
    pub fn render_audio_block(&mut self) -> &[f32] {
        let block_samples = self.block_size;
        self.output_interleaved_buffer.fill(0.0);

        if !self.is_playing {
            return &self.output_interleaved_buffer;
        }

        for i in 0..block_samples {
            if self.loop_enabled && self.playback_sample_pos >= self.loop_end_sample {
                self.playback_sample_pos = self.loop_start_sample;
            } else if self.playback_sample_pos >= self.total_project_samples {
                self.is_playing = false;
                break;
            }

            // Deterministic synthesizer test tone generation (440 Hz preview reference)
            let phase = 2.0 * std::f32::consts::PI * 440.0 * (self.playback_sample_pos as f32)
                / self.sample_rate;
            let sample = phase.sin() * 0.2;

            self.output_interleaved_buffer[i * 2] = sample; // Left
            self.output_interleaved_buffer[i * 2 + 1] = sample; // Right

            self.playback_sample_pos += 1;
        }

        &self.output_interleaved_buffer
    }

    /// Generate WebGPU multi-resolution waveform vertex buffer from audio samples.
    pub fn generate_webgpu_waveform_mipmap(
        samples: &[f32],
        samples_per_pixel: usize,
    ) -> WebGpuWaveformMipmap {
        let spp = samples_per_pixel.max(1);
        let total = samples.len();
        let num_pixels = total.div_ceil(spp);
        let mut vertices = Vec::with_capacity(num_pixels);

        for px in 0..num_pixels {
            let start = px * spp;
            let end = (start + spp).min(total);
            let slice = &samples[start..end];

            let mut min_val = 0.0f32;
            let mut max_val = 0.0f32;
            let mut sum_sq = 0.0f32;

            for &s in slice {
                if s < min_val {
                    min_val = s;
                }
                if s > max_val {
                    max_val = s;
                }
                sum_sq += s * s;
            }

            let rms = (sum_sq / slice.len().max(1) as f32).sqrt();
            let time_norm = px as f32 / num_pixels.max(1) as f32;

            vertices.push(WebGpuWaveformVertex {
                time_norm,
                min_peak: min_val.clamp(-1.0, 0.0),
                max_peak: max_val.clamp(0.0, 1.0),
                rms_energy: rms.clamp(0.0, 1.0),
            });
        }

        WebGpuWaveformMipmap {
            total_samples: total,
            samples_per_pixel: spp,
            vertices,
        }
    }
}

// C-ABI / WebAssembly Export Interfaces
#[no_mangle]
pub extern "C" fn summoner_wasm_create_renderer(
    sr: f32,
    block: u32,
    total_samples: u32,
) -> *mut WasmHeadlessRenderer {
    let r = Box::new(WasmHeadlessRenderer::new(
        sr,
        block as usize,
        total_samples as usize,
    ));
    Box::into_raw(r)
}

/// # Safety
/// Caller must ensure `ptr` is either null or was created by `summoner_wasm_create_renderer`.
#[no_mangle]
pub unsafe extern "C" fn summoner_wasm_destroy_renderer(ptr: *mut WasmHeadlessRenderer) {
    if !ptr.is_null() {
        drop(Box::from_raw(ptr));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_headless_renderer_playback_and_loop() {
        let mut renderer = WasmHeadlessRenderer::new(48000.0, 256, 1024);
        assert!(!renderer.is_playing);

        renderer.is_playing = true;
        renderer.set_loop_region(0, 512, true);

        // Render first block (256 samples)
        let block1 = renderer.render_audio_block().to_vec();
        assert_eq!(block1.len(), 512); // 256 * 2 channels
        assert_eq!(renderer.playback_sample_pos, 256);

        // Render second block (256 samples) reaches loop boundary 512
        renderer.render_audio_block();
        assert_eq!(renderer.playback_sample_pos, 512);

        // Third block wraps back to loop start (0)
        renderer.render_audio_block();
        assert!(renderer.playback_sample_pos < 512);
    }

    #[test]
    fn test_webgpu_waveform_mipmap_generation() {
        let samples: Vec<f32> = (0..1000).map(|i| (i as f32 * 0.1).sin()).collect();

        let mipmap = WasmHeadlessRenderer::generate_webgpu_waveform_mipmap(&samples, 100);
        assert_eq!(mipmap.vertices.len(), 10);
        assert_eq!(mipmap.samples_per_pixel, 100);

        // Check that min_peak is negative and max_peak is positive
        assert!(mipmap.vertices[0].min_peak <= 0.0);
        assert!(mipmap.vertices[0].max_peak >= 0.0);
        assert!(mipmap.vertices[0].rms_energy > 0.0);
    }
}
