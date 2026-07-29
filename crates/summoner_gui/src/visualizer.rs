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

use crate::renderer::RenderCommand;
use crate::lod::LodLevel;
#[cfg(feature = "gui")]
use eframe::egui;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Real-time lock-free oscilloscope ring buffer for audio waveform visualization.
#[derive(Clone)]
pub struct Oscilloscope {
    pub buffer: Arc<[AtomicU32; 512]>,
    pub write_pos: Arc<AtomicUsize>,
}

impl Oscilloscope {
    pub fn new() -> Self {
        let array: [AtomicU32; 512] = std::array::from_fn(|_| AtomicU32::new(0.0f32.to_bits()));
        Self {
            buffer: Arc::new(array),
            write_pos: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn write_sample(&self, sample: f32) {
        let pos = self.write_pos.fetch_add(1, Ordering::Relaxed) % 512;
        self.buffer[pos].store(sample.to_bits(), Ordering::Relaxed);
    }

    pub fn read_all(&self) -> [f32; 512] {
        let current_pos = self.write_pos.load(Ordering::Relaxed);
        let mut out = [0.0f32; 512];
        for i in 0..512 {
            let idx = (current_pos + i) % 512;
            out[i] = f32::from_bits(self.buffer[idx].load(Ordering::Relaxed));
        }
        out
    }
}

impl Default for Oscilloscope {
    fn default() -> Self {
        Self::new()
    }
}

/// Real-time spectrum analyzer containing FFT/DFT magnitude bins (256 bins).
#[derive(Clone)]
pub struct SpectrumAnalyzer {
    pub fft_output: Arc<[AtomicU32; 256]>,
}

impl SpectrumAnalyzer {
    pub fn new() -> Self {
        let array: [AtomicU32; 256] = std::array::from_fn(|_| AtomicU32::new(0.0f32.to_bits()));
        Self {
            fft_output: Arc::new(array),
        }
    }

    pub fn write_bin(&self, bin: usize, value: f32) {
        if bin < 256 {
            self.fft_output[bin].store(value.to_bits(), Ordering::Relaxed);
        }
    }

    pub fn read_all(&self) -> [f32; 256] {
        let mut out = [0.0f32; 256];
        for i in 0..256 {
            out[i] = f32::from_bits(self.fft_output[i].load(Ordering::Relaxed));
        }
        out
    }

    /// Spawns a background thread that periodically computes a 512-point DFT
    /// from the Oscilloscope waveform data and writes 256 magnitude bins to SpectrumAnalyzer.
    pub fn spawn_dft_thread(scope: Oscilloscope, spectrum: SpectrumAnalyzer) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let mut cos_table = Vec::with_capacity(256 * 512);
            let mut sin_table = Vec::with_capacity(256 * 512);
            for k in 0..256 {
                for n in 0..512 {
                    let angle = 2.0 * std::f32::consts::PI * (k as f32) * (n as f32) / 512.0;
                    cos_table.push(angle.cos());
                    sin_table.push(angle.sin());
                }
            }

            loop {
                thread::sleep(Duration::from_millis(30));
                let samples = scope.read_all();

                for k in 0..256 {
                    let mut re = 0.0f32;
                    let mut im = 0.0f32;
                    let offset = k * 512;
                    for n in 0..512 {
                        let sample = samples[n];
                        re += sample * cos_table[offset + n];
                        im -= sample * sin_table[offset + n];
                    }
                    let magnitude = ((re * re + im * im).sqrt() / 512.0).min(1.0);
                    spectrum.write_bin(k, magnitude);
                }
            }
        })
    }
}

impl Default for SpectrumAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Render real-time oscilloscope waveform inside UI.
#[cfg(feature = "gui")]
pub fn show_oscilloscope(ui: &mut egui::Ui, scope: &Oscilloscope, width: f32, height: f32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(8, 8, 12));

    let samples = scope.read_all();
    let num_samples = samples.len();
    let mut points = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let x = rect.left() + (i as f32 / num_samples as f32) * rect.width();
        let sample = samples[i].clamp(-1.0, 1.0);
        let y = rect.center().y - sample * (rect.height() * 0.45);
        points.push(egui::pos2(x, y));
    }

    if points.len() >= 2 {
        painter.add(egui::Shape::line(
            points,
            egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(26, 140, 255)),
        ));
    }

    response
}

/// Render real-time spectrum analyzer magnitude bars inside UI.
#[cfg(feature = "gui")]
pub fn show_spectrum(ui: &mut egui::Ui, spectrum: &SpectrumAnalyzer, width: f32, height: f32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(10, 12, 18));

    let bins = spectrum.read_all();
    let num_bars = 32;
    let bar_w = rect.width() / (num_bars as f32);

    for i in 0..num_bars {
        let bin_start = i * 8;
        let mut mag = 0.0f32;
        for b in 0..8 {
            mag += bins[bin_start + b];
        }
        mag = (mag / 8.0 * 8.0).clamp(0.0, 1.0);

        let bar_h = mag * rect.height();
        let x = rect.left() + (i as f32) * bar_w;
        let bar_rect = egui::Rect::from_min_max(
            egui::pos2(x, rect.bottom() - bar_h),
            egui::pos2(x + (bar_w - 1.0).max(1.0), rect.bottom()),
        );

        let ratio = i as f32 / num_bars as f32;
        let r = (ratio * 255.0) as u8;
        let g = ((1.0 - (ratio - 0.5).abs() * 2.0) * 200.0) as u8;
        let b = ((1.0 - ratio) * 255.0) as u8;

        painter.rect_filled(bar_rect, 1.0, egui::Color32::from_rgb(r, g.max(50), b));
    }

    response
}

/// Inline visualizers to be rendered inside the signal paths.
pub struct Visualizer {
    pub width: f32,
    pub height: f32,
}

impl Visualizer {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// Emits a mock render command for an oscilloscope view.
    pub fn draw_oscilloscope(&self, track_id: u64, x: f32, _y: f32) -> RenderCommand {
        RenderCommand::DrawWaveform {
            track_id,
            x,
            width: self.width,
            sample_count: 512,
            lod: LodLevel::Medium,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oscilloscope_write_read_round_trip() {
        let scope = Oscilloscope::new();
        scope.write_sample(0.5);
        scope.write_sample(-0.25);
        let samples = scope.read_all();
        assert!(samples.iter().any(|&s| (s - 0.5).abs() < 1e-5));
        assert!(samples.iter().any(|&s| (s - (-0.25)).abs() < 1e-5));
    }

    #[test]
    #[cfg(feature = "gui")]
    fn test_oscilloscope_show_does_not_panic() {
        let scope = Oscilloscope::new();
        scope.write_sample(0.8);
        let spectrum = SpectrumAnalyzer::new();
        spectrum.write_bin(10, 0.5);

        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_oscilloscope(ui, &scope, 200.0, 50.0);
                show_spectrum(ui, &spectrum, 200.0, 50.0);
            });
        });
    }
}
