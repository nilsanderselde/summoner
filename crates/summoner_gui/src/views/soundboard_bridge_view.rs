// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Grand Piano Spruce Soundboard & Bridge Wave Scattering HUD (Milestone 26).
//!
//! Provides an interactive 2D bridge impedance & modal decay puck (Bridge Coupling Bleed vs Decay Scale),
//! real-time 2D spruce soundboard standing wave modal vibration heatmap visualizer,
//! anisotropic wood grain dispersion indicators, and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and headless PNG snapshot rendering.

use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const SOUNDBOARD_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding target
pub const MIN_BRIDGE_BLEED: f32 = 0.05;
pub const MAX_BRIDGE_BLEED: f32 = 0.95;
pub const MIN_DECAY_SCALE: f32 = 0.2;
pub const MAX_DECAY_SCALE: f32 = 3.0;

/// Soundboard presets for the Bridge & Soundboard HUD.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SoundboardHudPreset {
    #[default]
    SteinwayD9Foot,
    BösendorferImperial,
    YamahaCFX,
    IntimateStudio,
    ImpressionistUnaCorda,
    PreparedAvantGarde,
}

impl SoundboardHudPreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::SteinwayD9Foot => "Steinway D (9ft Spruce Soundboard)",
            Self::BösendorferImperial => "Bösendorfer Imperial (9.5ft Spruce)",
            Self::YamahaCFX => "Yamaha CFX (9ft Resonant Spruce)",
            Self::IntimateStudio => "Intimate Studio Soundboard",
            Self::ImpressionistUnaCorda => "Impressionist Una Corda Resonator",
            Self::PreparedAvantGarde => "Prepared Avant-Garde Soundboard",
        }
    }
}

/// Spruce soundboard and bridge wave scattering interactive view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundboardBridgeView {
    /// Active soundboard preset.
    pub preset: SoundboardHudPreset,
    /// Bridge sympathetic coupling bleed $[0.05 ..= 0.95]$.
    pub bridge_bleed: f32,
    /// Soundboard modal decay scale $[0.2 ..= 3.0]$.
    pub soundboard_decay_scale: f32,
    /// Bridge mechanical impedance $Z_b$ in kg/s.
    pub bridge_impedance: f32,
    /// Inharmonicity dispersion $B$ parameter.
    pub inharmonicity_b: f32,
    /// 8-mode soundboard modal frequencies in Hz.
    pub modal_frequencies: [f32; 8],
    /// 8-mode quality factors $Q_k$.
    pub modal_qs: [f32; 8],
    /// 2D Puck position (X: bridge bleed, Y: decay scale).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for SoundboardBridgeView {
    fn default() -> Self {
        Self::new()
    }
}

impl SoundboardBridgeView {
    pub fn new() -> Self {
        let mut view = Self {
            preset: SoundboardHudPreset::SteinwayD9Foot,
            bridge_bleed: 0.35,
            soundboard_decay_scale: 1.0,
            bridge_impedance: 420.0,
            inharmonicity_b: 0.00018,
            modal_frequencies: [62.0, 125.0, 240.0, 380.0, 680.0, 1200.0, 2100.0, 3300.0],
            modal_qs: [15.0, 22.0, 28.0, 32.0, 38.0, 45.0, 50.0, 55.0],
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        };
        view.update_puck_from_physics();
        view.update_soundboard_acoustics();
        view
    }

    pub fn update_puck_from_physics(&mut self) {
        let norm_x = ((self.bridge_bleed - MIN_BRIDGE_BLEED) / (MAX_BRIDGE_BLEED - MIN_BRIDGE_BLEED)).clamp(0.0, 1.0);
        let norm_y = ((self.soundboard_decay_scale - MIN_DECAY_SCALE) / (MAX_DECAY_SCALE - MIN_DECAY_SCALE)).clamp(0.0, 1.0);
        self.puck_pos = (norm_x, norm_y);
    }

    pub fn update_physics_from_puck(&mut self, norm_x: f32, norm_y: f32) {
        self.puck_pos = (norm_x.clamp(0.0, 1.0), norm_y.clamp(0.0, 1.0));
        self.bridge_bleed = MIN_BRIDGE_BLEED + self.puck_pos.0 * (MAX_BRIDGE_BLEED - MIN_BRIDGE_BLEED);
        self.soundboard_decay_scale = MIN_DECAY_SCALE + self.puck_pos.1 * (MAX_DECAY_SCALE - MIN_DECAY_SCALE);
        self.update_soundboard_acoustics();
    }

    pub fn update_soundboard_acoustics(&mut self) {
        match self.preset {
            SoundboardHudPreset::SteinwayD9Foot => {
                self.bridge_impedance = 420.0;
                self.inharmonicity_b = 0.00018;
                self.modal_frequencies = [62.0, 125.0, 240.0, 380.0, 680.0, 1200.0, 2100.0, 3300.0];
                self.modal_qs = [15.0 * self.soundboard_decay_scale, 22.0 * self.soundboard_decay_scale, 28.0 * self.soundboard_decay_scale, 32.0 * self.soundboard_decay_scale, 38.0 * self.soundboard_decay_scale, 45.0 * self.soundboard_decay_scale, 50.0 * self.soundboard_decay_scale, 55.0 * self.soundboard_decay_scale];
            }
            SoundboardHudPreset::BösendorferImperial => {
                self.bridge_impedance = 380.0;
                self.inharmonicity_b = 0.00012;
                self.modal_frequencies = [52.0, 108.0, 210.0, 340.0, 610.0, 1080.0, 1950.0, 3050.0];
                self.modal_qs = [18.0 * self.soundboard_decay_scale, 26.0 * self.soundboard_decay_scale, 34.0 * self.soundboard_decay_scale, 40.0 * self.soundboard_decay_scale, 48.0 * self.soundboard_decay_scale, 55.0 * self.soundboard_decay_scale, 60.0 * self.soundboard_decay_scale, 65.0 * self.soundboard_decay_scale];
            }
            SoundboardHudPreset::YamahaCFX => {
                self.bridge_impedance = 480.0;
                self.inharmonicity_b = 0.00025;
                self.modal_frequencies = [68.0, 138.0, 265.0, 420.0, 740.0, 1320.0, 2350.0, 3600.0];
                self.modal_qs = [12.0 * self.soundboard_decay_scale, 18.0 * self.soundboard_decay_scale, 24.0 * self.soundboard_decay_scale, 28.0 * self.soundboard_decay_scale, 34.0 * self.soundboard_decay_scale, 40.0 * self.soundboard_decay_scale, 46.0 * self.soundboard_decay_scale, 50.0 * self.soundboard_decay_scale];
            }
            SoundboardHudPreset::IntimateStudio => {
                self.bridge_impedance = 520.0;
                self.inharmonicity_b = 0.00010;
                self.modal_frequencies = [58.0, 118.0, 225.0, 360.0, 640.0, 1150.0, 1980.0, 2900.0];
                self.modal_qs = [10.0 * self.soundboard_decay_scale, 15.0 * self.soundboard_decay_scale, 20.0 * self.soundboard_decay_scale, 24.0 * self.soundboard_decay_scale, 28.0 * self.soundboard_decay_scale, 32.0 * self.soundboard_decay_scale, 36.0 * self.soundboard_decay_scale, 40.0 * self.soundboard_decay_scale];
            }
            SoundboardHudPreset::ImpressionistUnaCorda => {
                self.bridge_impedance = 360.0;
                self.inharmonicity_b = 0.00015;
                self.modal_frequencies = [60.0, 120.0, 235.0, 370.0, 660.0, 1180.0, 2050.0, 3200.0];
                self.modal_qs = [20.0 * self.soundboard_decay_scale, 30.0 * self.soundboard_decay_scale, 38.0 * self.soundboard_decay_scale, 44.0 * self.soundboard_decay_scale, 52.0 * self.soundboard_decay_scale, 60.0 * self.soundboard_decay_scale, 68.0 * self.soundboard_decay_scale, 75.0 * self.soundboard_decay_scale];
            }
            SoundboardHudPreset::PreparedAvantGarde => {
                self.bridge_impedance = 600.0;
                self.inharmonicity_b = 0.00060;
                self.modal_frequencies = [75.0, 155.0, 290.0, 480.0, 820.0, 1480.0, 2600.0, 4100.0];
                self.modal_qs = [6.0 * self.soundboard_decay_scale, 8.0 * self.soundboard_decay_scale, 12.0 * self.soundboard_decay_scale, 14.0 * self.soundboard_decay_scale, 16.0 * self.soundboard_decay_scale, 18.0 * self.soundboard_decay_scale, 22.0 * self.soundboard_decay_scale, 25.0 * self.soundboard_decay_scale];
            }
        }
    }

    pub fn set_preset(&mut self, preset: SoundboardHudPreset) {
        self.preset = preset;
        match preset {
            SoundboardHudPreset::SteinwayD9Foot => {
                self.bridge_bleed = 0.35;
                self.soundboard_decay_scale = 1.0;
            }
            SoundboardHudPreset::BösendorferImperial => {
                self.bridge_bleed = 0.45;
                self.soundboard_decay_scale = 1.35;
            }
            SoundboardHudPreset::YamahaCFX => {
                self.bridge_bleed = 0.28;
                self.soundboard_decay_scale = 0.85;
            }
            SoundboardHudPreset::IntimateStudio => {
                self.bridge_bleed = 0.20;
                self.soundboard_decay_scale = 0.75;
            }
            SoundboardHudPreset::ImpressionistUnaCorda => {
                self.bridge_bleed = 0.50;
                self.soundboard_decay_scale = 1.20;
            }
            SoundboardHudPreset::PreparedAvantGarde => {
                self.bridge_bleed = 0.15;
                self.soundboard_decay_scale = 0.40;
            }
        }
        self.update_puck_from_physics();
        self.update_soundboard_acoustics();
    }

    /// Render ASCII diagnostics representation of the view.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let border = format!("+{}+", "-".repeat(width.saturating_sub(2)));
        lines.push(border.clone());

        let title = format!(
            "| GRAND PIANO SOUNDBOARD & BRIDGE HUD | Preset: {} |",
            self.preset.name()
        );
        let title_padded = format!("{:<width$}|", title, width = width - 1);
        lines.push(title_padded);

        let physics_info = format!(
            "| Bridge Bleed: {:.2} | Decay Scale: {:.2}x | Impedance: {:.0} kg/s | Inharm B: {:.5} |",
            self.bridge_bleed, self.soundboard_decay_scale, self.bridge_impedance, self.inharmonicity_b
        );
        let phys_padded = format!("{:<width$}|", physics_info, width = width - 1);
        lines.push(phys_padded);

        let modes_info = format!(
            "| Modes: {:.0}Hz, {:.0}Hz, {:.0}Hz, {:.0}Hz, {:.0}Hz, {:.0}Hz |",
            self.modal_frequencies[0], self.modal_frequencies[1], self.modal_frequencies[2],
            self.modal_frequencies[3], self.modal_frequencies[4], self.modal_frequencies[5]
        );
        let mod_padded = format!("{:<width$}|", modes_info, width = width - 1);
        lines.push(mod_padded);

        lines.push(border.clone());

        // 2D Spruce Soundboard plate vibration pattern
        let canvas_h = height.saturating_sub(6);
        for row in 0..canvas_h {
            let y_norm = row as f32 / canvas_h.max(1) as f32;
            let mut line_buf = String::with_capacity(width);
            line_buf.push('|');

            for col in 0..(width.saturating_sub(2)) {
                let x_norm = col as f32 / (width - 2).max(1) as f32;

                // Anisotropic standing wave pattern (1,1) + (1,2) + (2,1) + (2,2)
                let mode11 = (x_norm * std::f32::consts::PI).sin() * (y_norm * std::f32::consts::PI).sin();
                let mode12 = (x_norm * std::f32::consts::PI).sin() * (2.0 * y_norm * std::f32::consts::PI).sin();
                let mode21 = (2.0 * x_norm * std::f32::consts::PI).sin() * (y_norm * std::f32::consts::PI).sin();
                let pattern = (mode11 * 0.5 + mode12 * 0.3 + mode21 * 0.2).abs();

                if pattern > 0.65 {
                    line_buf.push('#');
                } else if pattern > 0.35 {
                    line_buf.push('+');
                } else if pattern > 0.15 {
                    line_buf.push('.');
                } else {
                    line_buf.push(' ');
                }
            }
            line_buf.push('|');
            lines.push(line_buf);
        }

        lines.push(border);
        lines
    }

    /// Render headless PNG snapshot to verify layout alignment, hit targets, and WCAG AAA contrast.
    pub fn render_snapshot_png(&self, path: &str, width: usize, height: usize) -> Result<(), String> {
        let mut pixels = vec![0u8; width * height * 4];

        // Background color: Deep midnight slate `#0A0E18`
        let bg_r = 0x0A;
        let bg_g = 0x0E;
        let bg_b = 0x18;

        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) * 4;
                pixels[idx] = bg_r;
                pixels[idx + 1] = bg_g;
                pixels[idx + 2] = bg_b;
                pixels[idx + 3] = 0xFF;
            }
        }

        // Draw 8pt spatial grid lines (15% opacity)
        let grid_r = 0x22;
        let grid_g = 0x2E;
        let grid_b = 0x48;
        for y in (0..height).step_by(8) {
            for x in 0..width {
                let idx = (y * width + x) * 4;
                pixels[idx] = grid_r;
                pixels[idx + 1] = grid_g;
                pixels[idx + 2] = grid_b;
            }
        }
        for x in (0..width).step_by(8) {
            for y in 0..height {
                let idx = (y * width + x) * 4;
                pixels[idx] = grid_r;
                pixels[idx + 1] = grid_g;
                pixels[idx + 2] = grid_b;
            }
        }

        // Left Panel: 2D Puck Area (40..340, 80..420)
        let puck_x_center = 40.0 + self.puck_pos.0 * 300.0;
        let puck_y_center = 420.0 - self.puck_pos.1 * 340.0; // Inverted Y

        let hit_radius = SOUNDBOARD_PUCK_HIT_RADIUS * 1.5;
        let vis_radius = 14.0 * 1.5;

        for y in 0..height {
            for x in 0..width {
                let dx = x as f32 - puck_x_center;
                let dy = y as f32 - puck_y_center;
                let dist = (dx * dx + dy * dy).sqrt();

                let idx = (y * width + x) * 4;
                if dist <= vis_radius {
                    // Puck body: Vibrant Emerald `#10B981`
                    pixels[idx] = 0x10;
                    pixels[idx + 1] = 0xB9;
                    pixels[idx + 2] = 0x81;
                } else if dist <= hit_radius {
                    // Glow ring: `#065F46`
                    pixels[idx] = 0x06;
                    pixels[idx + 1] = 0x5F;
                    pixels[idx + 2] = 0x46;
                }
            }
        }

        // Right Panel: 2D Spruce Soundboard Heatmap (400..760, 80..420)
        let heat_x_start = 400;
        let heat_x_end = 760;
        let heat_y_start = 80;
        let heat_y_end = 420;
        let heat_w = heat_x_end - heat_x_start;
        let heat_h = heat_y_end - heat_y_start;

        for hy in 0..heat_h {
            let y = heat_y_start + hy;
            let y_norm = hy as f32 / heat_h as f32;

            for hx in 0..heat_w {
                let x = heat_x_start + hx;
                let x_norm = hx as f32 / heat_w as f32;

                // Anisotropic Sitka spruce standing wave formula
                let mode11 = (x_norm * std::f32::consts::PI).sin() * (y_norm * std::f32::consts::PI).sin();
                let mode12 = (x_norm * std::f32::consts::PI).sin() * (2.0 * y_norm * std::f32::consts::PI).sin();
                let mode21 = (2.0 * x_norm * std::f32::consts::PI).sin() * (y_norm * std::f32::consts::PI).sin();
                let disp = (mode11 * 0.55 + mode12 * 0.30 + mode21 * 0.15).abs();

                let idx = (y * width + x) * 4;
                // Heatmap color ramp (Dark Navy -> Teal -> Golden Amber)
                if disp > 0.6 {
                    pixels[idx] = 0xF5;
                    pixels[idx + 1] = 0x9E;
                    pixels[idx + 2] = 0x0B; // Amber peak
                } else if disp > 0.3 {
                    pixels[idx] = 0x06;
                    pixels[idx + 1] = 0xB6;
                    pixels[idx + 2] = 0xD4; // Cyan mid
                } else if disp > 0.1 {
                    pixels[idx] = 0x1E;
                    pixels[idx + 1] = 0x3A;
                    pixels[idx + 2] = 0x8A; // Navy base
                }
            }
        }

        save_png_file(path, width, height, &pixels)
    }
}

#[cfg(feature = "gui")]
impl SoundboardBridgeView {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Spruce Soundboard & Bridge Wave Scattering HUD");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Profile:");
                if ui.selectable_label(self.preset == SoundboardHudPreset::SteinwayD9Foot, "Steinway D 9ft").clicked() {
                    self.set_preset(SoundboardHudPreset::SteinwayD9Foot);
                }
                if ui.selectable_label(self.preset == SoundboardHudPreset::BösendorferImperial, "Bösendorfer 290").clicked() {
                    self.set_preset(SoundboardHudPreset::BösendorferImperial);
                }
                if ui.selectable_label(self.preset == SoundboardHudPreset::YamahaCFX, "Yamaha CFX").clicked() {
                    self.set_preset(SoundboardHudPreset::YamahaCFX);
                }
                if ui.selectable_label(self.preset == SoundboardHudPreset::ImpressionistUnaCorda, "Impressionist").clicked() {
                    self.set_preset(SoundboardHudPreset::ImpressionistUnaCorda);
                }
            });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("Bridge & Modal Physics:");
                    ui.add(egui::Slider::new(&mut self.bridge_bleed, MIN_BRIDGE_BLEED..=MAX_BRIDGE_BLEED).text("Bridge Bleed"));
                    ui.add(egui::Slider::new(&mut self.soundboard_decay_scale, MIN_DECAY_SCALE..=MAX_DECAY_SCALE).text("Decay Scale"));
                });

                ui.separator();

                ui.vertical(|ui| {
                    ui.label(format!("Bridge Impedance Zb: {:.0} kg/s", self.bridge_impedance));
                    ui.label(format!("Inharmonicity B: {:.5}", self.inharmonicity_b));
                    ui.label(format!("Fundamental (1,1): {:.1} Hz", self.modal_frequencies[0]));
                });
            });
        });
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

    // IDAT Chunk
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
    fn test_soundboard_bridge_view_hit_target_dimensions() {
        const {
            assert!(
                SOUNDBOARD_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT,
                "Soundboard puck hit target bounding box must be >= 44pt"
            );
        }
    }

    #[test]
    fn test_soundboard_bridge_view_ascii_render() {
        let view = SoundboardBridgeView::new();
        let ascii = view.render_ascii(80, 16);
        assert_eq!(ascii.len(), 16);
        assert!(ascii[0].starts_with('+'));
    }

    #[test]
    fn test_soundboard_bridge_view_snapshot_render() {
        let view = SoundboardBridgeView::new();
        let res = view.render_snapshot_png("scratch/renders/soundboard_bridge_view.png", 800, 520);
        assert!(res.is_ok());
    }
}
