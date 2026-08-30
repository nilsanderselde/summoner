// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Dynamic Resonant Auto-Wah Filter HUD & Envelope Tracking Canvas (Milestone 23).
//!
//! Provides an interactive 2D sensitivity vs resonance puck interface, real-time dynamic
//! SVF bandpass transfer curve display, envelope follower ballistics visualizer,
//! filter mode selectors, and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and headless PNG snapshot rendering.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const WAH_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt bounding touch target

/// Auto-wah filter mode selector for GUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GuiWahFilterMode {
    #[default]
    Bandpass,
    Lowpass,
    PeakingWah,
}

impl GuiWahFilterMode {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Bandpass => "BANDPASS (FUNK WAH)",
            Self::Lowpass => "LOWPASS (SYNTH SWEEP)",
            Self::PeakingWah => "PEAKING (VOCAL VOWEL)",
        }
    }
}

/// Dynamic Resonant Auto-Wah Filter View.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoWahView {
    /// Effect enabled.
    pub enabled: bool,
    /// Envelope sensitivity $[0.0 ..= 1.0]$.
    pub sensitivity: f32,
    /// Base resting cutoff frequency in Hz $[50.0 ..= 4000.0\text{Hz}]$.
    pub base_freq_hz: f32,
    /// Maximum frequency sweep range in octaves $[0.5 ..= 6.0]$.
    pub sweep_range_octaves: f32,
    /// Filter resonance $Q$ factor $[0.5 ..= 25.0]$.
    pub resonance_q: f32,
    /// Filter topology mode.
    pub filter_mode: GuiWahFilterMode,
    /// Attack time in ms $[0.5 ..= 100.0\text{ms}]$.
    pub attack_ms: f32,
    /// Release time in ms $[5.0 ..= 1000.0\text{ms}]$.
    pub release_ms: f32,
    /// Dry / Wet mix balance $[0.0 ..= 1.0]$.
    pub mix: f32,
    /// Real-time modulated cutoff frequency in Hz for display.
    pub dynamic_cutoff_hz: f32,
    /// Current envelope follower value $[0.0 ..= 1.0]$.
    pub envelope_level: f32,
    /// 2D Puck position (X: sensitivity, Y: resonance_q normalized).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for AutoWahView {
    fn default() -> Self {
        Self::new()
    }
}

impl AutoWahView {
    pub fn new() -> Self {
        let mut view = Self {
            enabled: true,
            sensitivity: 0.75,
            base_freq_hz: 350.0,
            sweep_range_octaves: 3.5,
            resonance_q: 6.5,
            filter_mode: GuiWahFilterMode::Bandpass,
            attack_ms: 8.0,
            release_ms: 120.0,
            mix: 0.85,
            dynamic_cutoff_hz: 350.0,
            envelope_level: 0.0,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        };
        view.update_puck_from_physics();
        view
    }

    pub fn update_puck_from_physics(&mut self) {
        let norm_x = self.sensitivity.clamp(0.0, 1.0);
        // resonance_q: 0.5 .. 25.0 -> 0.0 .. 1.0
        let norm_y = ((self.resonance_q - 0.5) / 24.5).clamp(0.0, 1.0);
        self.puck_pos = (norm_x, norm_y);
    }

    pub fn update_physics_from_puck(&mut self, norm_x: f32, norm_y: f32) {
        self.puck_pos = (norm_x.clamp(0.0, 1.0), norm_y.clamp(0.0, 1.0));
        self.sensitivity = self.puck_pos.0;
        self.resonance_q = 0.5 + self.puck_pos.1 * 24.5;
    }

    /// Evaluates dynamic SVF filter transfer magnitude response curve across 32 logarithmic bins.
    pub fn evaluate_filter_magnitude_response(&self) -> [f32; 32] {
        let mut response = [0.0f32; 32];
        let fc = self.dynamic_cutoff_hz.clamp(50.0, 15000.0);
        let q = self.resonance_q.clamp(0.5, 25.0);

        for (i, val) in response.iter_mut().enumerate() {
            // Logarithmic frequency grid: 40 Hz .. 16 kHz
            let freq = 40.0 * 400.0f32.powf(i as f32 / 31.0);
            let w_ratio = freq / fc;

            let mag = match self.filter_mode {
                GuiWahFilterMode::Bandpass => {
                    // Bandpass 2nd-order transfer function |H(jw)|
                    let num = w_ratio / q;
                    let denom = ((1.0 - w_ratio * w_ratio).powi(2) + (w_ratio / q).powi(2)).sqrt();
                    (num / denom.max(1e-5)) * 1.5
                }
                GuiWahFilterMode::Lowpass => {
                    // Lowpass 2nd-order transfer function
                    let denom = ((1.0 - w_ratio * w_ratio).powi(2) + (w_ratio / q).powi(2)).sqrt();
                    1.0 / denom.max(1e-5)
                }
                GuiWahFilterMode::PeakingWah => {
                    // Peaking bell transfer function
                    let bp = (w_ratio / q) / (((1.0 - w_ratio * w_ratio).powi(2) + (w_ratio / q).powi(2)).sqrt()).max(1e-5);
                    1.0 + bp * 2.0
                }
            };

            // Convert to normalized UI coordinate [0.0 ..= 1.0]
            let db = 20.0 * (mag.max(1e-4)).log10();
            let norm = ((db + 24.0) / 48.0).clamp(0.0, 1.0);
            *val = norm;
        }
        response
    }

    /// Hit tests the 2D sensitivity/resonance puck.
    pub fn hit_test_puck(&self, screen_pos: (f32, f32), canvas_rect: Rect) -> bool {
        let puck_screen_x = canvas_rect.x + self.puck_pos.0 * canvas_rect.width;
        let puck_screen_y = canvas_rect.y + (1.0 - self.puck_pos.1) * canvas_rect.height;
        let dx = screen_pos.0 - puck_screen_x;
        let dy = screen_pos.1 - puck_screen_y;
        (dx * dx + dy * dy).sqrt() <= WAH_PUCK_HIT_RADIUS
    }

    /// Renders ASCII diagnostic representation.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let header = format!(
            "┌─ [AUTO-WAH HUD] {} ({}) ─┐",
            if self.enabled { "ACTIVE" } else { "BYPASS" },
            self.filter_mode.name()
        );
        let pad = width.saturating_sub(header.len());
        lines.push(format!("{}{}", header, "─".repeat(pad)));

        lines.push(format!(
            "│ Sens: {:.2} | Base Freq: {:.0}Hz | Range: {:.1} Oct | Q: {:.1} │",
            self.sensitivity, self.base_freq_hz, self.sweep_range_octaves, self.resonance_q
        ));

        lines.push(format!(
            "│ Ballistics: Att: {:.1}ms Rel: {:.0}ms | Mix: {:.0}% | Env: {:.2} │",
            self.attack_ms, self.release_ms, self.mix * 100.0, self.envelope_level
        ));

        let curve = self.evaluate_filter_magnitude_response();
        let mut curve_str = String::from("│ Response: ");
        for val in curve.iter().take(24) {
            let ch = if *val > 0.75 { '█' } else if *val > 0.5 { '▀' } else if *val > 0.25 { '─' } else { ' ' };
            curve_str.push(ch);
        }
        curve_str.push_str(" │");
        lines.push(curve_str);

        while lines.len() < height {
            lines.push(format!("│{:width$}│", "", width = width.saturating_sub(2)));
        }

        lines
    }

    /// Renders a headless PNG snapshot to the specified path.
    pub fn render_snapshot_png(&self, path: &str, width: usize, height: usize) -> Result<(), String> {
        let mut pixels = vec![0u8; width * height * 4];

        // Background: Deep slate #0A0E18
        for i in 0..(width * height) {
            pixels[i * 4] = 0x0A;
            pixels[i * 4 + 1] = 0x0E;
            pixels[i * 4 + 2] = 0x18;
            pixels[i * 4 + 3] = 0xFF;
        }

        // Draw 2D Sensitivity vs Q Canvas Box (Left Half)
        let margin = 32;
        let c_w = (width / 2) - margin * 2;
        let c_h = height - margin * 2;
        let c_x = margin;
        let c_y = margin;

        for y in c_y..(c_y + c_h) {
            for x in c_x..(c_x + c_w) {
                if x < width && y < height {
                    let idx = (y * width + x) * 4;
                    pixels[idx] = 0x14;
                    pixels[idx + 1] = 0x18;
                    pixels[idx + 2] = 0x24;
                    pixels[idx + 3] = 0xFF;
                }
            }
        }

        // Draw Sensitivity/Q Puck Handle (Radius = 22pt)
        let puck_cx = (c_x as f32 + self.puck_pos.0 * c_w as f32) as usize;
        let puck_cy = (c_y as f32 + (1.0 - self.puck_pos.1) * c_h as f32) as usize;
        let r = WAH_PUCK_HIT_RADIUS as usize;

        for dy in 0..=(2 * r) {
            for dx in 0..=(2 * r) {
                let px = (puck_cx + dx).saturating_sub(r);
                let py = (puck_cy + dy).saturating_sub(r);
                let dist_sq = (dx as isize - r as isize).pow(2) + (dy as isize - r as isize).pow(2);
                if dist_sq <= (r * r) as isize && px < width && py < height {
                    let idx = (py * width + px) * 4;
                    // Neon Mint #06D6A0
                    pixels[idx] = 0x06;
                    pixels[idx + 1] = 0xD6;
                    pixels[idx + 2] = 0xA0;
                    pixels[idx + 3] = 0xFF;
                }
            }
        }

        // Draw Filter Magnitude Response in Right Half
        let curve = self.evaluate_filter_magnitude_response();
        let w_x0 = (width / 2) + margin;
        let w_w = width - w_x0 - margin;
        let w_cy = height - margin;

        for (i, &val) in curve.iter().enumerate() {
            let x_coord = w_x0 + (i * w_w) / 32;
            let y_offset = (val * (c_h as f32 * 0.80)) as usize;
            let y_coord = (w_cy.saturating_sub(y_offset)).clamp(margin, height - margin);

            for dy in 0..3 {
                for dx in 0..3 {
                    let px = (x_coord + dx).min(width - 1);
                    let py = (y_coord + dy).min(height - 1);
                    let idx = (py * width + px) * 4;
                    // Electric Cyan #118AB2
                    pixels[idx] = 0x11;
                    pixels[idx + 1] = 0x8A;
                    pixels[idx + 2] = 0xB2;
                    pixels[idx + 3] = 0xFF;
                }
            }
        }

        save_png_file(path, width, height, &pixels)
    }
}

fn save_png_file(path: &str, width: usize, height: usize, rgba_pixels: &[u8]) -> Result<(), String> {
    let mut out = Vec::with_capacity(width * height * 4 + 1024);
    out.extend_from_slice(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]); // PNG Header

    // IHDR Chunk
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&(width as u32).to_be_bytes());
    ihdr.extend_from_slice(&(height as u32).to_be_bytes());
    ihdr.push(8); // Bit depth
    ihdr.push(6); // Color type: RGBA
    ihdr.push(0); // Compression method
    ihdr.push(0); // Filter method
    ihdr.push(0); // Interlace method
    write_png_chunk_to(&mut out, b"IHDR", &ihdr);

    // IDAT Chunk (Raw deflate uncompressed blocks)
    let stride = width * 4;
    let mut raw_data = Vec::with_capacity((stride + 1) * height);
    for y in 0..height {
        raw_data.push(0x00); // Filter type None
        let row_start = y * stride;
        let row_end = row_start + stride;
        raw_data.extend_from_slice(&rgba_pixels[row_start..row_end]);
    }

    let mut zlib_data = Vec::with_capacity(raw_data.len() + 128);
    zlib_data.push(0x78);
    zlib_data.push(0x01);

    let mut offset = 0;
    while offset < raw_data.len() {
        let chunk_len = (raw_data.len() - offset).min(65535);
        let is_last = (offset + chunk_len) >= raw_data.len();
        let bfinal_btype = if is_last { 0x01 } else { 0x00 };
        zlib_data.push(bfinal_btype);

        let len_u16 = chunk_len as u16;
        let nlen_u16 = !len_u16;
        zlib_data.extend_from_slice(&len_u16.to_le_bytes());
        zlib_data.extend_from_slice(&nlen_u16.to_le_bytes());
        zlib_data.extend_from_slice(&raw_data[offset..offset + chunk_len]);
        offset += chunk_len;
    }

    let mut s1: u32 = 1;
    let mut s2: u32 = 0;
    for &b in &raw_data {
        s1 = (s1 + b as u32) % 65521;
        s2 = (s2 + s1) % 65521;
    }
    let adler = (s2 << 16) | s1;
    zlib_data.extend_from_slice(&adler.to_be_bytes());

    write_png_chunk_to(&mut out, b"IDAT", &zlib_data);
    write_png_chunk_to(&mut out, b"IEND", &[]);

    if let Some(parent) = std::path::Path::new(path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(path, out).map_err(|e| e.to_string())
}

fn write_png_chunk_to(out: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let type_start = out.len();
    out.extend_from_slice(chunk_type);
    out.extend_from_slice(data);
    let crc = crc32_compute_to(&out[type_start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

fn crc32_compute_to(buf: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in buf {
        crc ^= b as u32;
        for _ in 0..8 {
            let mask = if (crc & 1) != 0 { 0xEDB8_8320 } else { 0 };
            crc = (crc >> 1) ^ mask;
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::touch_controls::MIN_HIT_TARGET_PT;

    #[test]
    fn test_auto_wah_view_hit_target_dimensions() {
        const {
            assert!(
                WAH_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT,
                "AutoWah puck hit target bounding box must be >= 44pt"
            );
        }
    }

    #[test]
    fn test_auto_wah_view_ascii_render() {
        let view = AutoWahView::new();
        let ascii = view.render_ascii(80, 16);
        assert_eq!(ascii.len(), 16);
        assert!(ascii[0].contains("AUTO-WAH HUD"));
    }

    #[test]
    fn test_auto_wah_view_snapshot_render() {
        let view = AutoWahView::new();
        let res = view.render_snapshot_png("scratch/renders/auto_wah_view.png", 800, 520);
        assert!(res.is_ok());
    }
}
