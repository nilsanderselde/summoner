// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Electromechanical Tine & Reed Electric Piano HUD & Pickup Alignment View (Milestone 22).
//!
//! Provides an interactive 2D tine tip alignment vs pickup air-gap puck interface,
//! real-time non-linear inductive pickup clipping wave display, stereo optical tremolo controls,
//! tube preamp saturation, tone controls, and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and headless PNG snapshot rendering.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const EP_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt bounding touch target

/// Electric piano sound profile selector for GUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GuiEpProfile {
    #[default]
    ClassicRhodesSuitcase,
    BarkingDynoRhodes,
    MellowRhodesStage,
    ClassicWurlitzer200A,
    SoulOverdrivenWurli,
    BelledAmbientRhodes,
}

impl GuiEpProfile {
    pub fn name(&self) -> &'static str {
        match self {
            Self::ClassicRhodesSuitcase => "CLASSIC RHODES SUITCASE (PING-PONG)",
            Self::BarkingDynoRhodes => "DYNO-MY-RHODES BARKING LEAD",
            Self::MellowRhodesStage => "MELLOW RHODES STAGE 73",
            Self::ClassicWurlitzer200A => "CLASSIC WURLITZER 200A REED",
            Self::SoulOverdrivenWurli => "SOUL OVERDRIVE WURLITZER",
            Self::BelledAmbientRhodes => "BELLED AMBIENT SPATIAL SWELL",
        }
    }
}

/// Electromechanical model selector for GUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GuiEpModel {
    #[default]
    RhodesTine,
    WurlitzerReed,
}

/// Electromechanical Tine & Reed Electric Piano View.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectricPianoView {
    pub profile: GuiEpProfile,
    pub model: GuiEpModel,
    /// Pickup air-gap distance in mm $[0.5 ..= 5.0\text{mm}]$.
    pub air_gap_mm: f32,
    /// Pickup vertical alignment offset in mm $[-2.0 ..= 2.0\text{mm}]$.
    pub alignment_offset_mm: f32,
    /// Hammer hardness $[0.0 ..= 1.0]$.
    pub hammer_hardness: f32,
    /// Inductive pickup barking overdrive $[0.0 ..= 1.0]$.
    pub bark_drive: f32,
    /// Resonant tonebar coupling stiffness $[0.0 ..= 1.0]$.
    pub tonebar_coupling: f32,
    /// Damper release clunk volume $[0.0 ..= 1.0]$.
    pub damper_clunk_volume: f32,
    /// Stereo optical tremolo enabled.
    pub tremolo_enabled: bool,
    /// Tremolo rate in Hz $[0.2 ..= 20.0\text{Hz}]$.
    pub tremolo_rate_hz: f32,
    /// Tremolo depth $[0.0 ..= 1.0]$.
    pub tremolo_depth: f32,
    /// Tremolo stereo ping-pong mode.
    pub tremolo_stereo: bool,
    /// Tube preamp drive in dB $[0.0 ..= 24.0\text{dB}]$.
    pub tube_drive_db: f32,
    /// Bass gain in dB $[-12.0 ..= 12.0\text{dB}]$.
    pub bass_db: f32,
    /// Treble gain in dB $[-12.0 ..= 12.0\text{dB}]$.
    pub treble_db: f32,
    /// Master level $[0.0 ..= 2.0]$.
    pub master_level: f32,
    /// 2D Puck position (X: air_gap normalized, Y: alignment_offset normalized).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for ElectricPianoView {
    fn default() -> Self {
        Self::new()
    }
}

impl ElectricPianoView {
    pub fn new() -> Self {
        let mut view = Self {
            profile: GuiEpProfile::ClassicRhodesSuitcase,
            model: GuiEpModel::RhodesTine,
            air_gap_mm: 1.8,
            alignment_offset_mm: 0.45,
            hammer_hardness: 0.50,
            bark_drive: 0.55,
            tonebar_coupling: 0.70,
            damper_clunk_volume: 0.35,
            tremolo_enabled: true,
            tremolo_rate_hz: 5.2,
            tremolo_depth: 0.65,
            tremolo_stereo: true,
            tube_drive_db: 4.0,
            bass_db: 1.5,
            treble_db: 2.0,
            master_level: 0.90,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        };
        view.update_puck_from_physics();
        view
    }

    pub fn set_profile(&mut self, profile: GuiEpProfile) {
        self.profile = profile;
        match profile {
            GuiEpProfile::ClassicRhodesSuitcase => {
                self.model = GuiEpModel::RhodesTine;
                self.air_gap_mm = 1.8;
                self.alignment_offset_mm = 0.45;
                self.hammer_hardness = 0.50;
                self.bark_drive = 0.55;
                self.tonebar_coupling = 0.70;
                self.damper_clunk_volume = 0.35;
                self.tremolo_enabled = true;
                self.tremolo_rate_hz = 5.2;
                self.tremolo_depth = 0.65;
                self.tremolo_stereo = true;
                self.tube_drive_db = 4.0;
                self.bass_db = 1.5;
                self.treble_db = 2.0;
            }
            GuiEpProfile::BarkingDynoRhodes => {
                self.model = GuiEpModel::RhodesTine;
                self.air_gap_mm = 1.0;
                self.alignment_offset_mm = 0.60;
                self.hammer_hardness = 0.85;
                self.bark_drive = 0.90;
                self.tonebar_coupling = 0.85;
                self.damper_clunk_volume = 0.40;
                self.tremolo_enabled = false;
                self.tube_drive_db = 8.0;
                self.bass_db = -1.0;
                self.treble_db = 6.5;
            }
            GuiEpProfile::MellowRhodesStage => {
                self.model = GuiEpModel::RhodesTine;
                self.air_gap_mm = 2.4;
                self.alignment_offset_mm = 0.20;
                self.hammer_hardness = 0.30;
                self.bark_drive = 0.30;
                self.tonebar_coupling = 0.60;
                self.damper_clunk_volume = 0.25;
                self.tremolo_enabled = false;
                self.tube_drive_db = 2.0;
                self.bass_db = 3.0;
                self.treble_db = -1.5;
            }
            GuiEpProfile::ClassicWurlitzer200A => {
                self.model = GuiEpModel::WurlitzerReed;
                self.air_gap_mm = 1.2;
                self.alignment_offset_mm = 0.15;
                self.hammer_hardness = 0.60;
                self.bark_drive = 0.75;
                self.tonebar_coupling = 0.35;
                self.damper_clunk_volume = 0.45;
                self.tremolo_enabled = true;
                self.tremolo_rate_hz = 6.0;
                self.tremolo_depth = 0.50;
                self.tremolo_stereo = false;
                self.tube_drive_db = 6.0;
                self.bass_db = 0.0;
                self.treble_db = 3.0;
            }
            GuiEpProfile::SoulOverdrivenWurli => {
                self.model = GuiEpModel::WurlitzerReed;
                self.air_gap_mm = 0.9;
                self.alignment_offset_mm = 0.25;
                self.hammer_hardness = 0.80;
                self.bark_drive = 0.95;
                self.tonebar_coupling = 0.40;
                self.damper_clunk_volume = 0.50;
                self.tremolo_enabled = true;
                self.tremolo_rate_hz = 7.2;
                self.tremolo_depth = 0.40;
                self.tremolo_stereo = false;
                self.tube_drive_db = 14.0;
                self.bass_db = 2.0;
                self.treble_db = 5.0;
            }
            GuiEpProfile::BelledAmbientRhodes => {
                self.model = GuiEpModel::RhodesTine;
                self.air_gap_mm = 2.0;
                self.alignment_offset_mm = 0.40;
                self.hammer_hardness = 0.40;
                self.bark_drive = 0.40;
                self.tonebar_coupling = 0.95;
                self.damper_clunk_volume = 0.20;
                self.tremolo_enabled = true;
                self.tremolo_rate_hz = 2.8;
                self.tremolo_depth = 0.80;
                self.tremolo_stereo = true;
                self.tube_drive_db = 2.0;
                self.bass_db = 2.5;
                self.treble_db = 3.5;
            }
        }
        self.update_puck_from_physics();
    }

    pub fn update_puck_from_physics(&mut self) {
        // air_gap: 0.5..5.0 mm -> 0.0..1.0
        let norm_x = ((self.air_gap_mm - 0.5) / 4.5).clamp(0.0, 1.0);
        // alignment_offset: -2.0..2.0 mm -> 0.0..1.0
        let norm_y = ((self.alignment_offset_mm + 2.0) / 4.0).clamp(0.0, 1.0);
        self.puck_pos = (norm_x, norm_y);
    }

    pub fn update_physics_from_puck(&mut self, norm_x: f32, norm_y: f32) {
        self.puck_pos = (norm_x.clamp(0.0, 1.0), norm_y.clamp(0.0, 1.0));
        self.air_gap_mm = 0.5 + self.puck_pos.0 * 4.5;
        self.alignment_offset_mm = -2.0 + self.puck_pos.1 * 4.0;
    }

    /// Evaluates non-linear inductive pickup transfer function wave display for 32 points.
    pub fn evaluate_pickup_wave(&self) -> [f32; 32] {
        let mut wave = [0.0f32; 32];
        for (i, val) in wave.iter_mut().enumerate() {
            let phase = (i as f32 / 32.0) * 2.0 * PI;
            let displacement = phase.sin() * 0.8;
            let velocity = phase.cos() * 0.8;

            let d = self.air_gap_mm;
            let y_rel = displacement * 1.5 - self.alignment_offset_mm;
            let r_sq = (d * d + y_rel * y_rel).max(0.05);
            let flux_grad = y_rel / (r_sq * r_sq.sqrt());
            let raw_v = flux_grad * velocity * 12.0;

            let drive = 1.0 + self.bark_drive * 3.5;
            let scaled = raw_v * drive;

            let sat = if scaled > 0.0 {
                scaled / (1.0 + scaled.abs())
            } else {
                let neg = scaled * 1.15;
                neg / (1.0 + neg.abs())
            };

            *val = sat.clamp(-1.0, 1.0);
        }
        wave
    }

    /// Hit tests the 2D alignment puck.
    pub fn hit_test_puck(&self, screen_pos: (f32, f32), canvas_rect: Rect) -> bool {
        let puck_screen_x = canvas_rect.x + self.puck_pos.0 * canvas_rect.width;
        let puck_screen_y = canvas_rect.y + (1.0 - self.puck_pos.1) * canvas_rect.height;
        let dx = screen_pos.0 - puck_screen_x;
        let dy = screen_pos.1 - puck_screen_y;
        (dx * dx + dy * dy).sqrt() <= EP_PUCK_HIT_RADIUS
    }

    /// Renders ASCII diagnostic representation.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let header = format!(
            "┌─ [EP HUD] {} ({}) ─┐",
            self.profile.name(),
            match self.model {
                GuiEpModel::RhodesTine => "Rhodes Tine",
                GuiEpModel::WurlitzerReed => "Wurlitzer Reed",
            }
        );
        let pad = width.saturating_sub(header.len());
        lines.push(format!("{}{}", header, "─".repeat(pad)));

        lines.push(format!(
            "│ Air-Gap: {:.2}mm | Offset: {:+.2}mm | Hardness: {:.2} | Bark: {:.2} │",
            self.air_gap_mm, self.alignment_offset_mm, self.hammer_hardness, self.bark_drive
        ));

        lines.push(format!(
            "│ Tonebar: {:.2} | Clunk: {:.2} | Drive: {:+.1}dB | EQ: B:{:+.1} T:{:+.1}dB │",
            self.tonebar_coupling, self.damper_clunk_volume, self.tube_drive_db, self.bass_db, self.treble_db
        ));

        lines.push(format!(
            "│ Tremolo: {} | Rate: {:.1}Hz | Depth: {:.0}% | Mode: {} │",
            if self.tremolo_enabled { "ON " } else { "OFF" },
            self.tremolo_rate_hz,
            self.tremolo_depth * 100.0,
            if self.tremolo_stereo { "Stereo Ping-Pong" } else { "Mono Optical" }
        ));

        let wave = self.evaluate_pickup_wave();
        let mut wave_str = String::from("│ Wave: ");
        for val in wave.iter().take(24) {
            let ch = if *val > 0.6 { '▲' } else if *val > 0.1 { '─' } else if *val < -0.6 { '▼' } else { ' ' };
            wave_str.push(ch);
        }
        wave_str.push_str(" │");
        lines.push(wave_str);

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

        // Draw 2D Air-Gap vs Alignment Canvas Box
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
                    pixels[idx + 1] = 0x1E;
                    pixels[idx + 2] = 0x30;
                    pixels[idx + 3] = 0xFF;
                }
            }
        }

        // Draw 2D Puck Handle (Radius = 22pt)
        let puck_cx = (c_x as f32 + self.puck_pos.0 * c_w as f32) as usize;
        let puck_cy = (c_y as f32 + (1.0 - self.puck_pos.1) * c_h as f32) as usize;
        let r = EP_PUCK_HIT_RADIUS as usize;

        for dy in 0..=(2 * r) {
            for dx in 0..=(2 * r) {
                let px = (puck_cx + dx).saturating_sub(r);
                let py = (puck_cy + dy).saturating_sub(r);
                let dist_sq = (dx as isize - r as isize).pow(2) + (dy as isize - r as isize).pow(2);
                if dist_sq <= (r * r) as isize && px < width && py < height {
                    let idx = (py * width + px) * 4;
                    // Amber/Gold highlight #FFB703
                    pixels[idx] = 0xFF;
                    pixels[idx + 1] = 0xB7;
                    pixels[idx + 2] = 0x03;
                    pixels[idx + 3] = 0xFF;
                }
            }
        }

        // Draw Pickup Bark Waveform in Right Half
        let wave = self.evaluate_pickup_wave();
        let w_x0 = (width / 2) + margin;
        let w_w = width - w_x0 - margin;
        let w_cy = height / 2;

        for (i, &val) in wave.iter().enumerate() {
            let x_coord = w_x0 + (i * w_w) / 32;
            let y_offset = (val * (c_h as f32 * 0.35)) as isize;
            let y_coord = (w_cy as isize - y_offset).clamp(margin as isize, (height - margin) as isize) as usize;

            for dy in 0..3 {
                for dx in 0..3 {
                    let px = (x_coord + dx).min(width - 1);
                    let py = (y_coord + dy).min(height - 1);
                    let idx = (py * width + px) * 4;
                    // Cyan #00F5D4
                    pixels[idx] = 0x00;
                    pixels[idx + 1] = 0xF5;
                    pixels[idx + 2] = 0xD4;
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
    fn test_electric_piano_view_hit_target_dimensions() {
        const {
            assert!(
                EP_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT,
                "EP puck hit target bounding box must be >= 44pt"
            );
        }
    }

    #[test]
    fn test_electric_piano_view_ascii_render() {
        let view = ElectricPianoView::new();
        let ascii = view.render_ascii(80, 16);
        assert_eq!(ascii.len(), 16);
        assert!(ascii[0].contains("EP HUD"));
    }

    #[test]
    fn test_electric_piano_view_snapshot_render() {
        let view = ElectricPianoView::new();
        let res = view.render_snapshot_png("scratch/renders/electric_piano_view.png", 800, 520);
        assert!(res.is_ok());
    }
}
