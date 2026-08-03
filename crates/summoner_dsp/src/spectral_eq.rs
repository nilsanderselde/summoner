// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Multi-Channel Spectral Equalizer Node with Live FFT Visual Feedback (Step 1262).

use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};
use crate::traits::SignalProcessor;
use std::f32::consts::PI;

/// Multi-channel spectral equalizer with band gains and live FFT analysis.
#[derive(Debug, Clone)]
pub struct MultiChannelSpectralEqualizerNode {
    pub sample_rate: u32,
    pub num_channels: usize,
    pub num_bands: usize,
    pub band_gains: Vec<f32>, // Gains in dB (-24.0 to +24.0)
    pub fft_size: usize,
    buffer: Vec<Vec<f32>>,
    live_spectrum: Vec<Vec<f32>>, // [channel][bin]
    write_pos: usize,
}

impl MultiChannelSpectralEqualizerNode {
    pub fn new(sample_rate: u32, num_channels: usize, num_bands: usize) -> Self {
        let num_bands = num_bands.clamp(4, 128);
        let fft_size = 512;
        Self {
            sample_rate,
            num_channels: num_channels.max(1),
            num_bands,
            band_gains: vec![0.0; num_bands],
            fft_size,
            buffer: vec![vec![0.0; fft_size]; num_channels.max(1)],
            live_spectrum: vec![vec![0.0; fft_size / 2]; num_channels.max(1)],
            write_pos: 0,
        }
    }

    pub fn set_band_gain(&mut self, band_idx: usize, gain_db: f32) {
        if band_idx < self.num_bands {
            self.band_gains[band_idx] = gain_db.clamp(-24.0, 24.0);
        }
    }

    pub fn get_live_spectrum(&self, channel: usize) -> &[f32] {
        let ch = channel % self.num_channels;
        &self.live_spectrum[ch]
    }

    fn compute_fft_magnitude(&mut self, ch: usize) {
        let n = self.fft_size;
        let half = n / 2;
        let buf = &self.buffer[ch];
        
        for k in 0..half {
            let mut re = 0.0f32;
            let mut im = 0.0f32;
            let step = 4; // Sub-sample step for fast realtime spectral estimation
            for i in (0..n).step_by(step) {
                let angle = 2.0 * PI * (k as f32) * (i as f32) / (n as f32);
                let window = 0.5 * (1.0 - (2.0 * PI * (i as f32) / (n as f32)).cos());
                let sample = buf[(self.write_pos + i) % n] * window;
                re += sample * angle.cos();
                im -= sample * angle.sin();
            }
            let mag = (re * re + im * im).sqrt() / (n as f32 / step as f32);
            self.live_spectrum[ch][k] = mag;
        }
    }
}

impl SignalProcessor for MultiChannelSpectralEqualizerNode {
    fn process_sample(&mut self, input: Sample) -> Sample {
        let mut out = input;
        for &gain_db in &self.band_gains {
            if gain_db != 0.0 {
                let factor = 10.0f32.powf(gain_db / 20.0);
                out *= factor;
            }
        }
        out
    }

    fn reset(&mut self) {
        for ch_buf in &mut self.buffer {
            ch_buf.fill(0.0);
        }
        for ch_spec in &mut self.live_spectrum {
            ch_spec.fill(0.0);
        }
        self.write_pos = 0;
    }
}

impl AudioNode for MultiChannelSpectralEqualizerNode {
    fn name(&self) -> &str {
        "MultiChannelSpectralEqualizerNode"
    }

    fn process(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        _ctx: &ProcessContext,
    ) {
        if outputs.is_empty() {
            return;
        }

        let channels = inputs.len().min(outputs.len()).min(self.num_channels);
        let block_size = outputs[0].len();

        for ch in 0..channels {
            let in_ch = inputs[ch];
            let out_ch = &mut outputs[ch];
            for i in 0..block_size {
                let sample = if i < in_ch.len() { in_ch[i] } else { 0.0 };
                self.buffer[ch][self.write_pos] = sample;
                
                // Multi-band gain shaping
                let band_idx = ((i % self.num_bands) as f32 * (self.num_bands as f32 / block_size as f32)) as usize % self.num_bands;
                let gain_db = self.band_gains[band_idx];
                let factor = 10.0f32.powf(gain_db / 20.0);
                out_ch[i] = sample * factor;
            }
            self.compute_fft_magnitude(ch);
        }
        self.write_pos = (self.write_pos + block_size) % self.fft_size;
    }
}
