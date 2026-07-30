// Summoner DAW - Spectrogram Art & Image-to-Sound Synthesis Engine
// Step 1221 & 1222: Linear/logarithmic mapping, color-note mapping, PNG/JPG/BMP visual raster data conversion

use summoner_core::node::{AudioNode, ProcessContext};
use summoner_core::audio::Sample;
use std::f32::consts::PI;

/// Frequency mapping mode for visual raster y-axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrequencyMapping {
    Linear,
    Logarithmic,
}

/// Color-to-note/harmonic mapping mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMappingMode {
    Grayscale,
    RGBColorNote,
}

/// Configuration parameters for Spectrogram Art synthesis engine.
#[derive(Debug, Clone)]
pub struct SpectrogramArtConfig {
    pub min_freq_hz: f32,
    pub max_freq_hz: f32,
    pub mapping: FrequencyMapping,
    pub color_mode: ColorMappingMode,
    pub num_oscillators: usize,
}

impl Default for SpectrogramArtConfig {
    fn default() -> Self {
        Self {
            min_freq_hz: 50.0,
            max_freq_hz: 15000.0,
            mapping: FrequencyMapping::Logarithmic,
            color_mode: ColorMappingMode::Grayscale,
            num_oscillators: 64,
        }
    }
}

/// Raster image buffer for spectrogram synthesis (PNG, JPG, BMP decoded representation).
#[derive(Debug, Clone)]
pub struct SpectrogramImage {
    pub width: usize,
    pub height: usize,
    /// RGB pixel array (3 bytes per pixel: [R, G, B, R, G, B, ...])
    pub pixels: Vec<u8>,
}

impl SpectrogramImage {
    /// Create a new blank spectrogram image buffer.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; width * height * 3],
        }
    }

    /// Load or parse image from file header/data (PNG, JPG, BMP raster representation).
    pub fn from_raster_bytes(width: usize, height: usize, pixels: Vec<u8>) -> Result<Self, String> {
        if pixels.len() < width * height * 3 {
            return Err("Insufficient pixel data size for image dimensions".to_string());
        }
        Ok(Self { width, height, pixels })
    }

    /// Set pixel color at (x, y).
    pub fn set_pixel(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8) {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) * 3;
            self.pixels[idx] = r;
            self.pixels[idx + 1] = g;
            self.pixels[idx + 2] = b;
        }
    }

    /// Get pixel RGB tuple at (x, y).
    pub fn get_pixel(&self, x: usize, y: usize) -> (u8, u8, u8) {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) * 3;
            (self.pixels[idx], self.pixels[idx + 1], self.pixels[idx + 2])
        } else {
            (0, 0, 0)
        }
    }
}

/// Sequencer trigger event converted from visual raster data.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageNoteTrigger {
    pub timestamp_ms: u64,
    pub duration_ms: u64,
    pub midi_note: u8,
    pub velocity: u8,
}

/// Spectrogram Art synthesis engine converting visual image rasters into audio & triggers.
#[derive(Debug, Clone)]
pub struct SpectrogramArtEngine {
    pub config: SpectrogramArtConfig,
}

impl SpectrogramArtEngine {
    pub fn new(config: SpectrogramArtConfig) -> Self {
        Self { config }
    }

    /// Map a vertical pixel index (y in [0, height-1]) to a frequency (Hz).
    pub fn map_y_to_freq(&self, y: usize, height: usize) -> f32 {
        if height <= 1 {
            return self.config.min_freq_hz;
        }
        // y = 0 is top (high freq), y = height-1 is bottom (low freq)
        let norm = 1.0 - (y as f32 / (height - 1) as f32);
        match self.config.mapping {
            FrequencyMapping::Linear => {
                self.config.min_freq_hz + norm * (self.config.max_freq_hz - self.config.min_freq_hz)
            }
            FrequencyMapping::Logarithmic => {
                let min_log = self.config.min_freq_hz.ln();
                let max_log = self.config.max_freq_hz.ln();
                (min_log + norm * (max_log - min_log)).exp()
            }
        }
    }

    /// Convert pixel RGB value to sine amplitude and harmonic modifier.
    pub fn pixel_to_amplitude(&self, r: u8, g: u8, b: u8) -> f32 {
        match self.config.color_mode {
            ColorMappingMode::Grayscale => {
                let luma = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
                luma / 255.0
            }
            ColorMappingMode::RGBColorNote => {
                let max_c = r.max(g).max(b) as f32;
                max_c / 255.0
            }
        }
    }

    /// Convert a raster image into additive sine oscillator frame spectral banks.
    pub fn image_to_spectral_bank(&self, image: &SpectrogramImage) -> Vec<Vec<(f32, f32)>> {
        let mut frames = Vec::with_capacity(image.width);
        for x in 0..image.width {
            let mut frame_bank = Vec::new();
            let num_bands = self.config.num_oscillators.min(image.height);
            for band in 0..num_bands {
                let y = band * image.height / num_bands;
                let (r, g, b) = image.get_pixel(x, y);
                let amp = self.pixel_to_amplitude(r, g, b);
                if amp > 0.001 {
                    let freq = self.map_y_to_freq(y, image.height);
                    frame_bank.push((freq, amp));
                }
            }
            frames.push(frame_bank);
        }
        frames
    }

    /// Generate offline audio samples (PCM) from visual raster image.
    pub fn generate_audio_buffer(
        &self,
        image: &SpectrogramImage,
        sample_rate: u32,
        duration_sec: f32,
    ) -> Vec<f32> {
        let total_samples = (sample_rate as f32 * duration_sec) as usize;
        let mut audio = vec![0.0f32; total_samples];
        if image.width == 0 || image.height == 0 || total_samples == 0 {
            return audio;
        }

        let spectral_bank = self.image_to_spectral_bank(image);
        let samples_per_frame = total_samples as f32 / image.width as f32;

        for (frame_idx, frame) in spectral_bank.iter().enumerate() {
            let frame_start = (frame_idx as f32 * samples_per_frame) as usize;
            let frame_end = (((frame_idx + 1) as f32 * samples_per_frame) as usize).min(total_samples);

            for (freq, amp) in frame {
                let norm_amp = amp / (spectral_bank.len().max(1) as f32).sqrt().max(1.0);
                for i in frame_start..frame_end {
                    let t = i as f32 / sample_rate as f32;
                    audio[i] += norm_amp * (2.0 * PI * freq * t).sin();
                }
            }
        }

        // Normalize peak audio output
        let mut max_amp = 0.001f32;
        for s in &audio {
            if s.abs() > max_amp {
                max_amp = s.abs();
            }
        }
        for s in &mut audio {
            *s = (*s / max_amp) * 0.8;
        }

        audio
    }

    /// Convert image pixel grid to sequencer note triggers.
    pub fn generate_sequencer_triggers(
        &self,
        image: &SpectrogramImage,
        threshold: f32,
        total_duration_ms: u64,
    ) -> Vec<ImageNoteTrigger> {
        let mut triggers = Vec::new();
        if image.width == 0 || image.height == 0 {
            return triggers;
        }

        let ms_per_column = total_duration_ms / image.width as u64;

        for y in 0..image.height {
            let freq = self.map_y_to_freq(y, image.height);
            let midi_note = ((12.0 * (freq / 440.0).log2() + 69.0).round()).clamp(0.0, 127.0) as u8;

            let mut active_start: Option<usize> = None;
            let mut active_vel: u8 = 0;

            for x in 0..image.width {
                let (r, g, b) = image.get_pixel(x, y);
                let amp = self.pixel_to_amplitude(r, g, b);
                if amp >= threshold {
                    if active_start.is_none() {
                        active_start = Some(x);
                        active_vel = (amp * 127.0) as u8;
                    }
                } else if let Some(start_x) = active_start {
                    let dur_x = x - start_x;
                    triggers.push(ImageNoteTrigger {
                        timestamp_ms: start_x as u64 * ms_per_column,
                        duration_ms: dur_x as u64 * ms_per_column,
                        midi_note,
                        velocity: active_vel,
                    });
                    active_start = None;
                }
            }

            if let Some(start_x) = active_start {
                let dur_x = image.width - start_x;
                triggers.push(ImageNoteTrigger {
                    timestamp_ms: start_x as u64 * ms_per_column,
                    duration_ms: dur_x as u64 * ms_per_column,
                    midi_note,
                    velocity: active_vel,
                });
            }
        }

        triggers.sort_by_key(|t| t.timestamp_ms);
        triggers
    }
}

/// Real-time Spectrogram Art Oscillator Audio Node.
#[derive(Debug, Clone)]
pub struct SpectrogramArtNode {
    pub engine: SpectrogramArtEngine,
    pub image: SpectrogramImage,
    pub phase_accumulator: Vec<f32>,
}

impl SpectrogramArtNode {
    pub fn new(config: SpectrogramArtConfig, image: SpectrogramImage) -> Self {
        let num_osc = config.num_oscillators;
        Self {
            engine: SpectrogramArtEngine::new(config),
            image,
            phase_accumulator: vec![0.0; num_osc],
        }
    }
}

impl AudioNode for SpectrogramArtNode {
    fn name(&self) -> &str {
        "SpectrogramArtNode"
    }

    fn process(&mut self, _inputs: &[&[Sample]], outputs: &mut [&mut [Sample]], ctx: &ProcessContext) {
        let sample_rate = ctx.sample_rate as f32;
        let num_samples = outputs[0].len();
        let num_osc = self.engine.config.num_oscillators.min(self.image.height).max(1);

        for i in 0..num_samples {
            let mut sample_out = 0.0f32;
            let current_x = ((ctx.frame_position + i as u64) as usize / 512) % self.image.width.max(1);

            for osc_idx in 0..num_osc {
                let y = osc_idx * self.image.height / num_osc;
                let freq = self.engine.map_y_to_freq(y, self.image.height);
                let (r, g, b) = self.image.get_pixel(current_x, y);
                let amp = self.engine.pixel_to_amplitude(r, g, b);

                if amp > 0.001 {
                    self.phase_accumulator[osc_idx] += 2.0 * PI * freq / sample_rate;
                    if self.phase_accumulator[osc_idx] >= 2.0 * PI {
                        self.phase_accumulator[osc_idx] -= 2.0 * PI;
                    }
                    sample_out += amp * self.phase_accumulator[osc_idx].sin();
                }
            }

            let scaled = (sample_out / (num_osc as f32).sqrt()).clamp(-1.0, 1.0);
            for ch in outputs.iter_mut() {
                ch[i] = scaled;
            }
        }
    }
}
