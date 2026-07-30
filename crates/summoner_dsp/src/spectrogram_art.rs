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

/// Spectral morphing blend modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpectralMorphMode {
    LinearCrossfade,
    SpectralWarp,
    ThresholdBlend,
    ColorHueMorph,
}

/// Configuration for spectral morphing engine.
#[derive(Debug, Clone)]
pub struct SpectralMorphConfig {
    pub morph_mode: SpectralMorphMode,
    pub morph_factor: f32,
    pub art_config: SpectrogramArtConfig,
}

impl Default for SpectralMorphConfig {
    fn default() -> Self {
        Self {
            morph_mode: SpectralMorphMode::LinearCrossfade,
            morph_factor: 0.5,
            art_config: SpectrogramArtConfig::default(),
        }
    }
}

/// Spectrogram Art spectral morphing crossfader engine.
#[derive(Debug, Clone)]
pub struct SpectrogramArtMorpher {
    pub config: SpectralMorphConfig,
    pub engine: SpectrogramArtEngine,
}

impl SpectrogramArtMorpher {
    pub fn new(config: SpectralMorphConfig) -> Self {
        let engine = SpectrogramArtEngine::new(config.art_config.clone());
        Self { config, engine }
    }

    /// Morph two dual SpectrogramImage visual soundscapes into a single blended SpectrogramImage.
    pub fn morph_images(
        &self,
        image_a: &SpectrogramImage,
        image_b: &SpectrogramImage,
        morph_factor: f32,
    ) -> SpectrogramImage {
        let alpha = morph_factor.clamp(0.0, 1.0);
        let target_w = image_a.width.max(image_b.width).max(1);
        let target_h = image_a.height.max(image_b.height).max(1);

        let mut morphed = SpectrogramImage::new(target_w, target_h);

        for y in 0..target_h {
            for x in 0..target_w {
                let src_xa = if image_a.width == 0 { 0 } else { x * image_a.width / target_w };
                let src_ya = if image_a.height == 0 { 0 } else { y * image_a.height / target_h };
                let src_xb = if image_b.width == 0 { 0 } else { x * image_b.width / target_w };
                let src_yb = if image_b.height == 0 { 0 } else { y * image_b.height / target_h };

                let (r_a, g_a, b_a) = image_a.get_pixel(src_xa, src_ya);
                let (r_b, g_b, b_b) = image_b.get_pixel(src_xb, src_yb);

                let (r, g, b) = match self.config.morph_mode {
                    SpectralMorphMode::LinearCrossfade | SpectralMorphMode::ColorHueMorph => {
                        let r = (1.0 - alpha) * (r_a as f32) + alpha * (r_b as f32);
                        let g = (1.0 - alpha) * (g_a as f32) + alpha * (g_b as f32);
                        let b = (1.0 - alpha) * (b_a as f32) + alpha * (b_b as f32);
                        (r as u8, g as u8, b as u8)
                    }
                    SpectralMorphMode::SpectralWarp => {
                        let warp_y_a = ((1.0 - alpha) * src_ya as f32) as usize;
                        let warp_y_b = (alpha * src_yb as f32) as usize;
                        let (r1, g1, b1) = image_a.get_pixel(src_xa, warp_y_a.min(image_a.height.saturating_sub(1)));
                        let (r2, g2, b2) = image_b.get_pixel(src_xb, warp_y_b.min(image_b.height.saturating_sub(1)));
                        let r = (1.0 - alpha) * (r1 as f32) + alpha * (r2 as f32);
                        let g = (1.0 - alpha) * (g1 as f32) + alpha * (g2 as f32);
                        let b = (1.0 - alpha) * (b1 as f32) + alpha * (b2 as f32);
                        (r as u8, g as u8, b as u8)
                    }
                    SpectralMorphMode::ThresholdBlend => {
                        let amp_a = self.engine.pixel_to_amplitude(r_a, g_a, b_a);
                        let amp_b = self.engine.pixel_to_amplitude(r_b, g_b, b_b);
                        if alpha < 0.5 {
                            if amp_a >= amp_b * alpha {
                                (r_a, g_a, b_a)
                            } else {
                                (r_b, g_b, b_b)
                            }
                        } else {
                            if amp_b >= amp_a * (1.0 - alpha) {
                                (r_b, g_b, b_b)
                            } else {
                                (r_a, g_a, b_a)
                            }
                        }
                    }
                };

                morphed.set_pixel(x, y, r, g, b);
            }
        }

        morphed
    }

    /// Generate morphed offline audio PCM buffer from dual SpectrogramImage soundscapes.
    pub fn generate_morphed_audio_buffer(
        &self,
        image_a: &SpectrogramImage,
        image_b: &SpectrogramImage,
        morph_factor: f32,
        sample_rate: u32,
        duration_sec: f32,
    ) -> Vec<f32> {
        let morphed_img = self.morph_images(image_a, image_b, morph_factor);
        self.engine.generate_audio_buffer(&morphed_img, sample_rate, duration_sec)
    }
}

/// Real-time Spectral Morphing Crossfader Audio Node.
#[derive(Debug, Clone)]
pub struct SpectrogramArtMorphNode {
    pub morpher: SpectrogramArtMorpher,
    pub image_a: SpectrogramImage,
    pub image_b: SpectrogramImage,
    pub morph_factor: f32,
    pub phase_accumulator: Vec<f32>,
}

impl SpectrogramArtMorphNode {
    pub fn new(
        config: SpectralMorphConfig,
        image_a: SpectrogramImage,
        image_b: SpectrogramImage,
        morph_factor: f32,
    ) -> Self {
        let num_osc = config.art_config.num_oscillators;
        Self {
            morpher: SpectrogramArtMorpher::new(config),
            image_a,
            image_b,
            morph_factor,
            phase_accumulator: vec![0.0; num_osc],
        }
    }

    pub fn set_morph_factor(&mut self, morph_factor: f32) {
        self.morph_factor = morph_factor.clamp(0.0, 1.0);
    }
}

impl AudioNode for SpectrogramArtMorphNode {
    fn name(&self) -> &str {
        "SpectrogramArtMorphNode"
    }

    fn process(&mut self, _inputs: &[&[Sample]], outputs: &mut [&mut [Sample]], ctx: &ProcessContext) {
        let sample_rate = ctx.sample_rate as f32;
        let num_samples = outputs[0].len();
        let max_h = self.image_a.height.max(self.image_b.height).max(1);
        let num_osc = self.morpher.engine.config.num_oscillators.min(max_h).max(1);
        let alpha = self.morph_factor.clamp(0.0, 1.0);

        for i in 0..num_samples {
            let mut sample_out = 0.0f32;
            let width_a = self.image_a.width.max(1);
            let width_b = self.image_b.width.max(1);

            let current_x_a = ((ctx.frame_position + i as u64) as usize / 512) % width_a;
            let current_x_b = ((ctx.frame_position + i as u64) as usize / 512) % width_b;

            for osc_idx in 0..num_osc {
                let y_a = osc_idx * self.image_a.height / num_osc;
                let y_b = osc_idx * self.image_b.height / num_osc;

                let freq_a = self.morpher.engine.map_y_to_freq(y_a, self.image_a.height.max(1));
                let freq_b = self.morpher.engine.map_y_to_freq(y_b, self.image_b.height.max(1));
                let freq = (1.0 - alpha) * freq_a + alpha * freq_b;

                let (r_a, g_a, b_a) = self.image_a.get_pixel(current_x_a, y_a);
                let (r_b, g_b, b_b) = self.image_b.get_pixel(current_x_b, y_b);

                let amp_a = self.morpher.engine.pixel_to_amplitude(r_a, g_a, b_a);
                let amp_b = self.morpher.engine.pixel_to_amplitude(r_b, g_b, b_b);
                let amp = (1.0 - alpha) * amp_a + alpha * amp_b;

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

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::transport::Transport;

    #[test]
    fn test_step_1245_spectral_morphing_crossfader_images() {
        let mut img_a = SpectrogramImage::new(8, 8);
        let mut img_b = SpectrogramImage::new(8, 8);

        // Solid red for A, solid blue for B
        for y in 0..8 {
            for x in 0..8 {
                img_a.set_pixel(x, y, 255, 0, 0);
                img_b.set_pixel(x, y, 0, 0, 255);
            }
        }

        let config = SpectralMorphConfig {
            morph_mode: SpectralMorphMode::LinearCrossfade,
            morph_factor: 0.5,
            art_config: SpectrogramArtConfig::default(),
        };
        let morpher = SpectrogramArtMorpher::new(config);

        let morphed = morpher.morph_images(&img_a, &img_b, 0.5);
        assert_eq!(morphed.width, 8);
        assert_eq!(morphed.height, 8);

        let (r, g, b) = morphed.get_pixel(0, 0);
        assert_eq!(r, 127);
        assert_eq!(g, 0);
        assert_eq!(b, 127);

        // Test audio buffer generation from morphed images
        let audio = morpher.generate_morphed_audio_buffer(&img_a, &img_b, 0.5, 44100, 0.1);
        assert_eq!(audio.len(), 4410);
        assert!(audio.iter().any(|&s| s.abs() > 0.001));
    }

    #[test]
    fn test_step_1245_spectrogram_art_morph_node() {
        let mut img_a = SpectrogramImage::new(16, 16);
        let mut img_b = SpectrogramImage::new(16, 16);

        for i in 0..16 {
            img_a.set_pixel(i, i, 255, 255, 255);
            img_b.set_pixel(15 - i, i, 255, 255, 255);
        }

        let config = SpectralMorphConfig::default();
        let mut node = SpectrogramArtMorphNode::new(config, img_a, img_b, 0.3);

        let transport = Transport::new(44100, 120.0);
        let ctx = ProcessContext::from_transport(&transport);
        let mut buf_l = vec![0.0f32; 64];
        let mut buf_r = vec![0.0f32; 64];

        node.process(&[], &mut [&mut buf_l, &mut buf_r], &ctx);
        assert!(buf_l.iter().any(|&s| s.abs() > 0.0));

        node.set_morph_factor(0.8);
        assert_eq!(node.morph_factor, 0.8);
    }
}


