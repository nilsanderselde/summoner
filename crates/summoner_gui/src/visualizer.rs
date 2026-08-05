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

/// Render Phase Scope (Lissajous X/Y plot rotated 45 deg) inside UI (Step 671).
#[cfg(feature = "gui")]
pub fn show_phase_scope(ui: &mut egui::Ui, left_buf: &[f32], right_buf: &[f32], width: f32, height: f32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(10, 10, 16));
    painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 60, 90)));

    let center = rect.center();
    let radius = (rect.width().min(rect.height()) * 0.45).max(10.0);

    let len = left_buf.len().min(right_buf.len());
    if len > 0 {
        let inv_sqrt2 = 0.70710678f32;
        for i in 0..len {
            let l = left_buf[i].clamp(-1.0, 1.0);
            let r = right_buf[i].clamp(-1.0, 1.0);

            let x = (l - r) * inv_sqrt2;
            let y = (l + r) * inv_sqrt2;

            let pos = egui::pos2(center.x + x * radius, center.y - y * radius);
            painter.circle_filled(pos, 1.2, egui::Color32::from_rgb(0, 230, 180));
        }
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

/// Quantum state tomographic visualizer state and rendering UI (Step 1149).
#[derive(Clone, Debug)]
pub struct QuantumTomographyVisualizer {
    pub bloch_x: Arc<AtomicU32>,
    pub bloch_y: Arc<AtomicU32>,
    pub bloch_z: Arc<AtomicU32>,
    pub purity: Arc<AtomicU32>,
}

impl QuantumTomographyVisualizer {
    pub fn new() -> Self {
        Self {
            bloch_x: Arc::new(AtomicU32::new(0.0f32.to_bits())),
            bloch_y: Arc::new(AtomicU32::new(0.0f32.to_bits())),
            bloch_z: Arc::new(AtomicU32::new(1.0f32.to_bits())),
            purity: Arc::new(AtomicU32::new(1.0f32.to_bits())),
        }
    }

    pub fn update(&self, x: f32, y: f32, z: f32, purity: f32) {
        self.bloch_x.store(x.to_bits(), Ordering::Relaxed);
        self.bloch_y.store(y.to_bits(), Ordering::Relaxed);
        self.bloch_z.store(z.to_bits(), Ordering::Relaxed);
        self.purity.store(purity.to_bits(), Ordering::Relaxed);
    }

    pub fn read(&self) -> (f32, f32, f32, f32) {
        (
            f32::from_bits(self.bloch_x.load(Ordering::Relaxed)),
            f32::from_bits(self.bloch_y.load(Ordering::Relaxed)),
            f32::from_bits(self.bloch_z.load(Ordering::Relaxed)),
            f32::from_bits(self.purity.load(Ordering::Relaxed)),
        )
    }
}

impl Default for QuantumTomographyVisualizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "gui")]
pub fn show_quantum_tomography(
    ui: &mut egui::Ui,
    vis: &QuantumTomographyVisualizer,
    width: f32,
    height: f32,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(12, 16, 28));
        painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 120, 220)));

        let (x, y, z, purity) = vis.read();
        let center = rect.center();
        let radius = (rect.width().min(rect.height()) * 0.4).max(10.0);

        painter.circle_stroke(center, radius, egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 90, 150)));
        let bloch_pos = egui::pos2(center.x + x * radius, center.y - y * radius);
        painter.circle_filled(bloch_pos, 4.0, egui::Color32::from_rgb(0, 240, 255));
        painter.line_segment([center, bloch_pos], egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 240, 255)));

        let label = format!("Purity: {:.2} | Z: {:.2}", purity, z);
        painter.text(
            egui::pos2(rect.min.x + 6.0, rect.min.y + 6.0),
            egui::Align2::LEFT_TOP,
            label,
            egui::FontId::proportional(11.0),
            egui::Color32::WHITE,
        );
    }
    response
}

/// Render Peak Headroom Analyzer & EBU R128 Loudness Metering Display Component (Step 1270 & Step 1272).
#[cfg(feature = "gui")]
pub fn show_ebu_r128_loudness_meter(
    ui: &mut egui::Ui,
    meter: &summoner_dsp::EbuR128LoudnessMeter,
    analyzer: &summoner_dsp::PeakHeadroomAnalyzer,
    width: f32,
    height: f32,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(14, 16, 22));
        painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 70, 100)));

        let m_lufs = meter.momentary_lufs;
        let s_lufs = meter.short_term_lufs;
        let i_lufs = meter.integrated_lufs;
        let true_peak = analyzer.true_peak_db;

        let status_text = format!(
            "M: {:.1} LUFS | S: {:.1} LUFS | I: {:.1} LUFS | Peak: {:.1} dB",
            m_lufs, s_lufs, i_lufs, true_peak
        );

        painter.text(
            egui::pos2(rect.min.x + 8.0, rect.min.y + 6.0),
            egui::Align2::LEFT_TOP,
            status_text,
            egui::FontId::proportional(11.0),
            egui::Color32::from_rgb(0, 220, 255),
        );
    }
    response
}

/// Audio driver configuration and device selector UI panel state (Step 1269).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AudioDriverSelectorPanel {
    /// Active driver backend name (e.g. WASAPI, ASAPI, ALSA, AAudio, AudioUnit).
    pub selected_driver_name: String,
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Buffer size in frames.
    pub buffer_size: usize,
    /// Exclusive mode toggle.
    pub exclusive_mode: bool,
    /// List of available audio output devices.
    pub device_list: Vec<String>,
    /// Index of selected output device.
    pub selected_device_index: usize,
}

impl Default for AudioDriverSelectorPanel {
    fn default() -> Self {
        Self {
            selected_driver_name: "WASAPI".to_string(),
            sample_rate: 48000,
            buffer_size: 256,
            exclusive_mode: false,
            device_list: vec![
                "Default High Definition Audio Endpoint".to_string(),
                "WASAPI Low Latency Endpoint".to_string(),
                "ASAPI Direct Out Device".to_string(),
                "ALSA Hardware Device (HW:0)".to_string(),
            ],
            selected_device_index: 0,
        }
    }
}

/// Render WASAPI / ASAPI / ALSA driver device selector UI panel (Step 1269).
#[cfg(feature = "gui")]
pub fn show_audio_driver_selector_panel(
    ui: &mut egui::Ui,
    panel: &mut AudioDriverSelectorPanel,
) {
    ui.group(|ui| {
        ui.heading("Audio Driver Settings & Device Selector");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Driver API:");
            egui::ComboBox::from_id_source("driver_api_combo")
                .selected_text(&panel.selected_driver_name)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut panel.selected_driver_name, "WASAPI".to_string(), "WASAPI (Windows Native)");
                    ui.selectable_value(&mut panel.selected_driver_name, "ASAPI".to_string(), "ASAPI (Low Latency)");
                    ui.selectable_value(&mut panel.selected_driver_name, "ALSA".to_string(), "ALSA (Linux Audio)");
                    ui.selectable_value(&mut panel.selected_driver_name, "AAudio".to_string(), "AAudio (Android NDK)");
                    ui.selectable_value(&mut panel.selected_driver_name, "AudioUnit".to_string(), "AudioUnit (iOS CoreAudio)");
                });
        });

        ui.horizontal(|ui| {
            ui.label("Device:");
            if !panel.device_list.is_empty() {
                let current_device = panel.device_list[panel.selected_device_index % panel.device_list.len()].clone();
                egui::ComboBox::from_id_source("driver_device_combo")
                    .selected_text(current_device)
                    .show_ui(ui, |ui| {
                        for (idx, dev) in panel.device_list.iter().enumerate() {
                            ui.selectable_value(&mut panel.selected_device_index, idx, dev);
                        }
                    });
            }
        });

        ui.horizontal(|ui| {
            ui.label("Sample Rate:");
            egui::ComboBox::from_id_source("sample_rate_combo")
                .selected_text(format!("{} Hz", panel.sample_rate))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut panel.sample_rate, 44100, "44100 Hz");
                    ui.selectable_value(&mut panel.sample_rate, 48000, "48000 Hz");
                    ui.selectable_value(&mut panel.sample_rate, 96000, "96000 Hz");
                });

            ui.separator();
            ui.label("Buffer Size:");
            egui::ComboBox::from_id_source("buffer_size_combo")
                .selected_text(format!("{} frames", panel.buffer_size))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut panel.buffer_size, 64, "64 frames (1.3 ms)");
                    ui.selectable_value(&mut panel.buffer_size, 128, "128 frames (2.6 ms)");
                    ui.selectable_value(&mut panel.buffer_size, 256, "256 frames (5.3 ms)");
                    ui.selectable_value(&mut panel.buffer_size, 512, "512 frames (10.6 ms)");
                });
        });

        ui.checkbox(&mut panel.exclusive_mode, "Exclusive Hardware Mode");
    });
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
