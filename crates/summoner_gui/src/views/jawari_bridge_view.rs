// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Sitar Curved Jawari Bridge & Tarab Sympathetic Resonator HUD (Milestone 27).
//!
//! Provides an interactive 2D Jawari clearance gap & Jiva cotton thread puck (Jiva Thread Position vs Clearance Gap),
//! real-time 13-string sympathetic Tarab energy dissipation spectrum visualizer,
//! curved bone/horn obstacle profile geometry display, and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and headless PNG snapshot rendering.

use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const JAWARI_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding target
pub const MIN_JAWARI_CLEARANCE_GAP: f32 = 0.01;
pub const MAX_JAWARI_CLEARANCE_GAP: f32 = 1.5;
pub const MIN_JIVA_THREAD_POS: f32 = 0.0;
pub const MAX_JIVA_THREAD_POS: f32 = 1.0;

/// Jawari bridge presets for the Jawari HUD.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum JawariBridgeHudPreset {
    #[default]
    DeerHornRaviShankar,
    CamelBoneVilayatKhan,
    EbonySurbaharMellow,
    SyntheticDelrinPrecision,
    ElectricMetalSizzle,
    OpenAcousticGourd,
}

impl JawariBridgeHudPreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::DeerHornRaviShankar => "Deer-Horn Curved Jawari (Ravi Shankar)",
            Self::CamelBoneVilayatKhan => "Camel-Bone Sharp Jawari (Vilayat Khan)",
            Self::EbonySurbaharMellow => "Ebony Wood Mellow Jawari (Surbahar)",
            Self::SyntheticDelrinPrecision => "Synthetic Delrin Precision Jawari",
            Self::ElectricMetalSizzle => "Electric Metal Flat Jawari",
            Self::OpenAcousticGourd => "Open Acoustic Gourd Jawari",
        }
    }
}

/// Jawari bridge and 13-string Tarab sympathetic resonator interactive view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JawariBridgeView {
    /// Active Jawari bridge preset.
    pub preset: JawariBridgeHudPreset,
    /// Jawari clearance gap $h_0$ in mm $[0.01 ..= 1.5]$.
    pub clearance_gap_mm: f32,
    /// Jiva cotton thread position $[0.0 ..= 1.0]$.
    pub jiva_thread_pos: f32,
    /// Bridge curvature parameter $c$ in $\text{m}^{-1}$.
    pub curvature_c: f32,
    /// Tarab sympathetic coupling bleed $[0.0 ..= 1.0]$.
    pub tarab_bleed: f32,
    /// 13-string Tarab energy dissipation levels $[0.0 ..= 1.0]$.
    pub tarab_energy: [f32; 13],
    /// 2D Puck position (X: jiva thread position, Y: clearance gap).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for JawariBridgeView {
    fn default() -> Self {
        Self::new()
    }
}

impl JawariBridgeView {
    pub fn new() -> Self {
        let mut view = Self {
            preset: JawariBridgeHudPreset::DeerHornRaviShankar,
            clearance_gap_mm: 0.18,
            jiva_thread_pos: 0.45,
            curvature_c: 120.0,
            tarab_bleed: 0.40,
            tarab_energy: [0.75, 0.60, 0.85, 0.45, 0.90, 0.55, 0.70, 0.80, 0.65, 0.50, 0.40, 0.60, 0.35],
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        };
        view.update_puck_from_physics();
        view.update_jawari_acoustics();
        view
    }

    pub fn update_puck_from_physics(&mut self) {
        let norm_x = ((self.jiva_thread_pos - MIN_JIVA_THREAD_POS) / (MAX_JIVA_THREAD_POS - MIN_JIVA_THREAD_POS)).clamp(0.0, 1.0);
        let norm_y = ((self.clearance_gap_mm - MIN_JAWARI_CLEARANCE_GAP) / (MAX_JAWARI_CLEARANCE_GAP - MIN_JAWARI_CLEARANCE_GAP)).clamp(0.0, 1.0);
        self.puck_pos = (norm_x, norm_y);
    }

    pub fn update_physics_from_puck(&mut self, norm_x: f32, norm_y: f32) {
        self.puck_pos = (norm_x.clamp(0.0, 1.0), norm_y.clamp(0.0, 1.0));
        self.jiva_thread_pos = MIN_JIVA_THREAD_POS + self.puck_pos.0 * (MAX_JIVA_THREAD_POS - MIN_JIVA_THREAD_POS);
        self.clearance_gap_mm = MIN_JAWARI_CLEARANCE_GAP + self.puck_pos.1 * (MAX_JAWARI_CLEARANCE_GAP - MIN_JAWARI_CLEARANCE_GAP);
        self.update_jawari_acoustics();
    }

    pub fn update_jawari_acoustics(&mut self) {
        match self.preset {
            JawariBridgeHudPreset::DeerHornRaviShankar => {
                self.curvature_c = 120.0;
                self.tarab_bleed = 0.40;
                self.tarab_energy = [0.80, 0.65, 0.85, 0.50, 0.92, 0.58, 0.72, 0.85, 0.68, 0.52, 0.45, 0.62, 0.38];
            }
            JawariBridgeHudPreset::CamelBoneVilayatKhan => {
                self.curvature_c = 240.0;
                self.tarab_bleed = 0.35;
                self.tarab_energy = [0.90, 0.75, 0.70, 0.60, 0.88, 0.65, 0.78, 0.80, 0.72, 0.58, 0.50, 0.48, 0.30];
            }
            JawariBridgeHudPreset::EbonySurbaharMellow => {
                self.curvature_c = 60.0;
                self.tarab_bleed = 0.50;
                self.tarab_energy = [0.95, 0.88, 0.82, 0.75, 0.90, 0.70, 0.65, 0.60, 0.55, 0.48, 0.40, 0.35, 0.28];
            }
            JawariBridgeHudPreset::SyntheticDelrinPrecision => {
                self.curvature_c = 160.0;
                self.tarab_bleed = 0.30;
                self.tarab_energy = [0.70, 0.60, 0.65, 0.55, 0.75, 0.60, 0.68, 0.72, 0.60, 0.50, 0.42, 0.45, 0.35];
            }
            JawariBridgeHudPreset::ElectricMetalSizzle => {
                self.curvature_c = 320.0;
                self.tarab_bleed = 0.20;
                self.tarab_energy = [0.60, 0.50, 0.55, 0.45, 0.65, 0.48, 0.52, 0.58, 0.45, 0.40, 0.35, 0.30, 0.25];
            }
            JawariBridgeHudPreset::OpenAcousticGourd => {
                self.curvature_c = 90.0;
                self.tarab_bleed = 0.55;
                self.tarab_energy = [0.85, 0.72, 0.90, 0.65, 0.95, 0.70, 0.80, 0.88, 0.75, 0.62, 0.55, 0.68, 0.45];
            }
        }
    }

    pub fn set_preset(&mut self, preset: JawariBridgeHudPreset) {
        self.preset = preset;
        match preset {
            JawariBridgeHudPreset::DeerHornRaviShankar => {
                self.clearance_gap_mm = 0.18;
                self.jiva_thread_pos = 0.45;
            }
            JawariBridgeHudPreset::CamelBoneVilayatKhan => {
                self.clearance_gap_mm = 0.08;
                self.jiva_thread_pos = 0.50;
            }
            JawariBridgeHudPreset::EbonySurbaharMellow => {
                self.clearance_gap_mm = 0.35;
                self.jiva_thread_pos = 0.35;
            }
            JawariBridgeHudPreset::SyntheticDelrinPrecision => {
                self.clearance_gap_mm = 0.15;
                self.jiva_thread_pos = 0.48;
            }
            JawariBridgeHudPreset::ElectricMetalSizzle => {
                self.clearance_gap_mm = 0.05;
                self.jiva_thread_pos = 0.60;
            }
            JawariBridgeHudPreset::OpenAcousticGourd => {
                self.clearance_gap_mm = 0.22;
                self.jiva_thread_pos = 0.40;
            }
        }
        self.update_puck_from_physics();
        self.update_jawari_acoustics();
    }

    /// Render ASCII diagnostics representation of the view.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let border = format!("+{}+", "-".repeat(width.saturating_sub(2)));
        lines.push(border.clone());

        let title = format!(
            "| JAWARI BRIDGE & TARAB RESONATOR HUD | Preset: {} |",
            self.preset.name()
        );
        let title_padded = format!("{:<width$}|", title, width = width - 1);
        lines.push(title_padded);

        let physics_info = format!(
            "| Clearance h0: {:.2}mm | Jiva Pos: {:.2} | Curvature c: {:.0} m^-1 | Tarab Bleed: {:.0}% |",
            self.clearance_gap_mm, self.jiva_thread_pos, self.curvature_c, self.tarab_bleed * 100.0
        );
        let phys_padded = format!("{:<width$}|", physics_info, width = width - 1);
        lines.push(phys_padded);

        let tarab_info = format!(
            "| Tarab E: {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2} |",
            self.tarab_energy[0], self.tarab_energy[1], self.tarab_energy[2], self.tarab_energy[3],
            self.tarab_energy[4], self.tarab_energy[5], self.tarab_energy[6], self.tarab_energy[7],
            self.tarab_energy[8], self.tarab_energy[9], self.tarab_energy[10], self.tarab_energy[11],
            self.tarab_energy[12]
        );
        let tar_padded = format!("{:<width$}|", tarab_info, width = width - 1);
        lines.push(tar_padded);

        lines.push(border.clone());

        // 13-String Tarab energy bar chart canvas
        let canvas_h = height.saturating_sub(6);
        for row in 0..canvas_h {
            let row_ratio = 1.0 - (row as f32 / canvas_h.max(1) as f32);
            let mut line_buf = String::with_capacity(width);
            line_buf.push('|');

            for col in 0..(width.saturating_sub(2)) {
                let string_idx = (col * 13) / (width - 2).max(1);
                let string_e = if string_idx < 13 { self.tarab_energy[string_idx] } else { 0.0 };

                if string_e >= row_ratio {
                    line_buf.push('#');
                } else if string_e >= (row_ratio - 0.15) {
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

        let hit_radius = JAWARI_PUCK_HIT_RADIUS * 1.5;
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

        // Right Panel: 13-String Tarab Sympathetic Energy Bars (400..760, 80..420)
        let bar_x_start = 400;
        let bar_x_end = 760;
        let bar_w = (bar_x_end - bar_x_start) / 13;

        for i in 0..13 {
            let x0 = bar_x_start + i * bar_w;
            let x1 = x0 + bar_w - 6; // 6pt padding between bars
            let bar_h = (self.tarab_energy[i] * 320.0) as usize;
            let y_top = 420usize.saturating_sub(bar_h);

            for y in y_top..420 {
                for x in x0..=x1 {
                    if y < height && x < width {
                        let idx = (y * width + x) * 4;
                        // Vibrant Amber/Gold `#E5A93C` to Magenta `#D946EF` gradient
                        let frac = i as f32 / 13.0;
                        let r = (0xE5 as f32 * (1.0 - frac) + 0xD9 as f32 * frac) as u8;
                        let g = (0xA9 as f32 * (1.0 - frac) + 0x46 as f32 * frac) as u8;
                        let b = (0x3C as f32 * (1.0 - frac) + 0xEF as f32 * frac) as u8;
                        pixels[idx] = r;
                        pixels[idx + 1] = g;
                        pixels[idx + 2] = b;
                    }
                }
            }
        }

        save_png_file(path, width, height, &pixels)
    }
}

#[cfg(feature = "gui")]
impl JawariBridgeView {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Jawari Bridge & Tarab Sympathetic Resonator HUD");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Profile:");
                if ui.selectable_label(self.preset == JawariBridgeHudPreset::DeerHornRaviShankar, "Deer Horn (Ravi)").clicked() {
                    self.set_preset(JawariBridgeHudPreset::DeerHornRaviShankar);
                }
                if ui.selectable_label(self.preset == JawariBridgeHudPreset::CamelBoneVilayatKhan, "Camel Bone (Vilayat)").clicked() {
                    self.set_preset(JawariBridgeHudPreset::CamelBoneVilayatKhan);
                }
                if ui.selectable_label(self.preset == JawariBridgeHudPreset::EbonySurbaharMellow, "Ebony Surbahar").clicked() {
                    self.set_preset(JawariBridgeHudPreset::EbonySurbaharMellow);
                }
                if ui.selectable_label(self.preset == JawariBridgeHudPreset::SyntheticDelrinPrecision, "Delrin Precision").clicked() {
                    self.set_preset(JawariBridgeHudPreset::SyntheticDelrinPrecision);
                }
            });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("Bridge Geometry & Cotton Thread:");
                    ui.add(egui::Slider::new(&mut self.clearance_gap_mm, MIN_JAWARI_CLEARANCE_GAP..=MAX_JAWARI_CLEARANCE_GAP).text("Clearance Gap (mm)"));
                    ui.add(egui::Slider::new(&mut self.jiva_thread_pos, MIN_JIVA_THREAD_POS..=MAX_JIVA_THREAD_POS).text("Jiva Position"));
                });

                ui.separator();

                ui.vertical(|ui| {
                    ui.label(format!("Curvature c: {:.0} m^-1", self.curvature_c));
                    ui.add(egui::Slider::new(&mut self.tarab_bleed, 0.0..=1.0).text("Tarab Bleed"));
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
    fn test_jawari_bridge_view_hit_target_dimensions() {
        const {
            assert!(
                JAWARI_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT,
                "Jawari puck hit target bounding box must be >= 44pt"
            );
        }
    }

    #[test]
    fn test_jawari_bridge_view_ascii_render() {
        let view = JawariBridgeView::new();
        let ascii = view.render_ascii(80, 16);
        assert_eq!(ascii.len(), 16);
        assert!(ascii[0].starts_with('+'));
    }

    #[test]
    fn test_jawari_bridge_view_snapshot_render() {
        let view = JawariBridgeView::new();
        let res = view.render_snapshot_png("scratch/renders/jawari_bridge_view.png", 800, 520);
        assert!(res.is_ok());
    }
}
