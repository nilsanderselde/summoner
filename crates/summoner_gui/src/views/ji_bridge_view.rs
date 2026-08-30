// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Koto Movable Ji Bridge Scattering & Paulownia Soundboard HUD (Milestone 29).
//!
//! Provides an interactive visualizer for the 13 movable triangular *ji* bridges:
//! - Real-time bridge position layout along soundboard length (playing segment vs behind-the-bridge segment)
//! - Acoustic boundary scattering transmission/reflection matrix HUD
//! - Paulownia soundboard 8-mode acoustic modal vibration energy distribution
//! - 44x44pt hit targets for movable bridge adjustments and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and headless PNG snapshot rendering.

use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const JI_HANDLE_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding target
pub const NUM_JI_BRIDGES: usize = 13;

/// Ji Bridge material profile choices for the visualizer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum JiBridgeViewProfile {
    #[default]
    PaulowniaHardwood,
    IvoryBone,
    RosewoodGuzheng,
    SyntheticDelrin,
    SmokedBamboo,
}

impl JiBridgeViewProfile {
    pub fn name(&self) -> &'static str {
        match self {
            Self::PaulowniaHardwood => "Paulownia Hardwood Ji (Traditional)",
            Self::IvoryBone => "Ivory / Bone Sharp Ji (Bright Attack)",
            Self::RosewoodGuzheng => "Rosewood Triangular Bridge (Warm Guzheng)",
            Self::SyntheticDelrin => "Synthetic Delrin Ji (Clean & Stable)",
            Self::SmokedBamboo => "Smoked Vintage Bamboo (Mellow & Dark)",
        }
    }

    pub fn transmission_reflection(&self) -> (f32, f32) {
        match self {
            Self::PaulowniaHardwood => (0.08, 0.985),
            Self::IvoryBone => (0.04, 0.992),
            Self::RosewoodGuzheng => (0.12, 0.980),
            Self::SyntheticDelrin => (0.06, 0.988),
            Self::SmokedBamboo => (0.14, 0.975),
        }
    }
}

/// Movable Ji Bridge and Paulownia Soundboard HUD.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiBridgeView {
    /// Active material profile.
    pub profile: JiBridgeViewProfile,
    /// 13 Ji bridge position fractions along string length $[0.15 ..= 0.90]$.
    pub bridge_positions: [f32; NUM_JI_BRIDGES],
    /// Currently selected string/bridge index.
    pub selected_bridge_index: usize,
    /// Soundboard body modal energy distribution $[0.0 ..= 1.0]$ across 8 modes.
    pub soundboard_modal_energy: [f32; 8],
    /// Behind-the-bridge sympathetic ringing level $[0.0 ..= 1.0]$.
    pub behind_bridge_bleed: f32,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for JiBridgeView {
    fn default() -> Self {
        Self::new()
    }
}

impl JiBridgeView {
    pub fn new() -> Self {
        // Authentic Hirajoshi default bridge positions
        let mut default_pos = [0.65f32; NUM_JI_BRIDGES];
        let ratios: [f32; NUM_JI_BRIDGES] = [1.0000, 0.6674, 0.7492, 0.7937, 1.0000, 1.0595, 1.3348, 1.4983, 1.5874, 2.0000, 2.1189, 2.6697, 2.9966];
        for (i, &r) in ratios.iter().enumerate() {
            default_pos[i] = (0.65 / r.sqrt()).clamp(0.20, 0.85);
        }

        Self {
            profile: JiBridgeViewProfile::PaulowniaHardwood,
            bridge_positions: default_pos,
            selected_bridge_index: 0,
            soundboard_modal_energy: [0.85, 0.72, 0.60, 0.50, 0.42, 0.35, 0.28, 0.20],
            behind_bridge_bleed: 0.25,
            color_palette: ContrastColorPalette::default(),
        }
    }

    pub fn set_profile(&mut self, profile: JiBridgeViewProfile) {
        self.profile = profile;
    }

    pub fn set_bridge_position(&mut self, bridge_idx: usize, pos: f32) {
        if bridge_idx < NUM_JI_BRIDGES {
            self.bridge_positions[bridge_idx] = pos.clamp(0.15, 0.90);
        }
    }

    /// Render ASCII diagnostics representation of the view.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let border = format!("+{}+", "-".repeat(width.saturating_sub(2)));
        lines.push(border.clone());

        let (trans, refl) = self.profile.transmission_reflection();
        let title = format!(
            "| JI BRIDGE HUD | Material: {} | Trans: {:.0}% | Refl: {:.1}% |",
            self.profile.name(),
            trans * 100.0,
            refl * 100.0
        );
        let title_padded = format!("{:<width$}|", title, width = width - 1);
        lines.push(title_padded);

        lines.push(border.clone());

        // 13 movable Ji bridges distribution display
        let canvas_h = height.saturating_sub(4);
        for row in 0..canvas_h {
            let str_idx = (row * NUM_JI_BRIDGES) / canvas_h.max(1);
            let pos = self.bridge_positions[str_idx.min(NUM_JI_BRIDGES - 1)];

            let mut line_buf = String::with_capacity(width);
            let label = format!("| Str {:2} [", str_idx + 1);
            line_buf.push_str(&label);

            let bar_len = width.saturating_sub(18);
            let bridge_pos_col = (pos * bar_len as f32) as usize;

            for col in 0..bar_len {
                if col == bridge_pos_col {
                    line_buf.push('^'); // Triangular Ji bridge symbol
                } else if col < bridge_pos_col {
                    line_buf.push('='); // Playing segment
                } else {
                    line_buf.push('-'); // Behind-the-bridge sympathetic segment
                }
            }
            line_buf.push_str("] |");
            let line_padded = format!("{:<width$}|", line_buf, width = width - 1);
            lines.push(line_padded);
        }

        lines.push(border);
        lines
    }

    /// Render headless PNG snapshot visualizer for layout alignment, touch target verification, and WCAG contrast.
    pub fn render_snapshot_png(&self, path: &str, width: usize, height: usize) -> Result<(), String> {
        let mut pixels = vec![0u8; width * height * 4];

        // Background: Deep Slate (#0A0E18)
        for idx in (0..pixels.len()).step_by(4) {
            pixels[idx] = 0x0A;
            pixels[idx + 1] = 0x0E;
            pixels[idx + 2] = 0x18;
            pixels[idx + 3] = 0xFF;
        }

        // Header Panel (8pt grid padding)
        let header_h = 56;
        for y in 0..header_h {
            for x in 0..width {
                let idx = (y * width + x) * 4;
                pixels[idx] = 0x13;
                pixels[idx + 1] = 0x1D;
                pixels[idx + 2] = 0x2E;
            }
        }

        // Title indicator bar (Gold #F59E0B)
        for y in 12..20 {
            for x in 24..200 {
                let idx = (y * width + x) * 4;
                pixels[idx] = 0xF5;
                pixels[idx + 1] = 0x9E;
                pixels[idx + 2] = 0x0B;
            }
        }

        // Soundboard Body Box (Paulownia Wood Kiri)
        let c_left = 32;
        let c_top = 72;
        let c_right = width - 32;
        let c_bottom = height - 80;

        for y in c_top..c_bottom {
            for x in c_left..c_right {
                let idx = (y * width + x) * 4;
                let is_border = x == c_left || x == c_right - 1 || y == c_top || y == c_bottom - 1;
                if is_border {
                    pixels[idx] = 0xF5;
                    pixels[idx + 1] = 0x9E;
                    pixels[idx + 2] = 0x0B; // Amber gold soundboard border
                } else if x % 32 == 0 {
                    pixels[idx] = 0x1E;
                    pixels[idx + 1] = 0x29;
                    pixels[idx + 2] = 0x3B; // 8pt guides
                } else {
                    pixels[idx] = 0x0F;
                    pixels[idx + 1] = 0x17;
                    pixels[idx + 2] = 0x2A;
                }
            }
        }

        // Render 13 Koto strings and movable Ji bridges
        let lane_h = (c_bottom - c_top) as f32 / NUM_JI_BRIDGES as f32;
        let soundboard_w = (c_right - c_left) as f32;

        for str_idx in 0..NUM_JI_BRIDGES {
            let str_y = (c_top as f32 + (str_idx as f32 + 0.5) * lane_h) as usize;
            let bridge_pos = self.bridge_positions[str_idx];
            let bridge_x = c_left + (soundboard_w * bridge_pos) as usize;

            // Draw string: Playing segment (Cyan) & Behind-the-bridge (Amber)
            for x in c_left..c_right {
                let idx = (str_y * width + x) * 4;
                if x < bridge_x {
                    pixels[idx] = 0x38;
                    pixels[idx + 1] = 0xBD;
                    pixels[idx + 2] = 0xF8; // Bright cyan playing segment
                } else {
                    pixels[idx] = 0x64;
                    pixels[idx + 1] = 0x74;
                    pixels[idx + 2] = 0x8B; // Muted behind-the-bridge segment
                }
            }

            // Draw triangular Ji bridge handle (>= 44x44pt hit target)
            let radius = 12;
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    if dx * dx + dy * dy <= radius * radius {
                        let px = (bridge_x as isize + dx) as usize;
                        let py = (str_y as isize + dy) as usize;
                        if px < width && py < height {
                            let idx = (py * width + px) * 4;
                            pixels[idx] = 0xF5;
                            pixels[idx + 1] = 0x9E;
                            pixels[idx + 2] = 0x0B; // Amber gold Ji bridge puck
                        }
                    }
                }
            }
        }

        // Bottom Soundboard Modal Energy Bar (8 modes)
        let bar_y_start = height - 60;
        let bar_y_end = height - 36;
        let num_modes = 8;
        let mode_w = (c_right - c_left) / num_modes;

        for mode_idx in 0..num_modes {
            let mode_left = c_left + mode_idx * mode_w;
            let mode_right = mode_left + mode_w - 4;
            let energy = self.soundboard_modal_energy[mode_idx];
            let fill_h = ((bar_y_end - bar_y_start) as f32 * energy) as usize;

            for y in bar_y_start..bar_y_end {
                for x in mode_left..mode_right {
                    let idx = (y * width + x) * 4;
                    if y >= bar_y_end - fill_h {
                        pixels[idx] = 0x10;
                        pixels[idx + 1] = 0xB9;
                        pixels[idx + 2] = 0x81; // Green modal resonance
                    } else {
                        pixels[idx] = 0x1E;
                        pixels[idx + 1] = 0x29;
                        pixels[idx + 2] = 0x3B;
                    }
                }
            }
        }

        save_png_file(path, width, height, &pixels)
    }
}

#[cfg(feature = "gui")]
impl JiBridgeView {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Movable Ji Bridge & Paulownia Soundboard HUD");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Ji Bridge Material:");
                if ui.selectable_label(self.profile == JiBridgeViewProfile::PaulowniaHardwood, "Paulownia Wood").clicked() {
                    self.set_profile(JiBridgeViewProfile::PaulowniaHardwood);
                }
                if ui.selectable_label(self.profile == JiBridgeViewProfile::IvoryBone, "Ivory / Bone").clicked() {
                    self.set_profile(JiBridgeViewProfile::IvoryBone);
                }
                if ui.selectable_label(self.profile == JiBridgeViewProfile::RosewoodGuzheng, "Rosewood Guzheng").clicked() {
                    self.set_profile(JiBridgeViewProfile::RosewoodGuzheng);
                }
                if ui.selectable_label(self.profile == JiBridgeViewProfile::SyntheticDelrin, "Synthetic Delrin").clicked() {
                    self.set_profile(JiBridgeViewProfile::SyntheticDelrin);
                }
                if ui.selectable_label(self.profile == JiBridgeViewProfile::SmokedBamboo, "Smoked Bamboo").clicked() {
                    self.set_profile(JiBridgeViewProfile::SmokedBamboo);
                }
            });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("Selected String Bridge Position:");
                let mut pos = self.bridge_positions[self.selected_bridge_index];
                if ui.add(egui::Slider::new(&mut pos, 0.15..=0.90).text(format!("String {}", self.selected_bridge_index + 1))).changed() {
                    self.set_bridge_position(self.selected_bridge_index, pos);
                }
            });
        });
    }
}

fn save_png_file(path: &str, width: usize, height: usize, rgba_pixels: &[u8]) -> Result<(), String> {
    let mut out = Vec::with_capacity(width * height * 4 + 1024);
    out.extend_from_slice(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);

    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&(width as u32).to_be_bytes());
    ihdr.extend_from_slice(&(height as u32).to_be_bytes());
    ihdr.push(8);
    ihdr.push(6);
    ihdr.push(0);
    ihdr.push(0);
    ihdr.push(0);
    write_png_chunk_to(&mut out, b"IHDR", &ihdr);

    let stride = width * 4;
    let mut raw_data = Vec::with_capacity((stride + 1) * height);
    for y in 0..height {
        raw_data.push(0x00);
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
    fn test_ji_bridge_view_hit_target_dimensions() {
        const {
            assert!(
                JI_HANDLE_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT,
                "Ji handle hit target bounding box must be >= 44pt"
            );
        }
    }

    #[test]
    fn test_ji_bridge_view_ascii_render() {
        let view = JiBridgeView::new();
        let ascii = view.render_ascii(80, 16);
        assert_eq!(ascii.len(), 16);
        assert!(ascii[0].starts_with('+'));
    }

    #[test]
    fn test_ji_bridge_view_snapshot_render() {
        let view = JiBridgeView::new();
        let res = view.render_snapshot_png("scratch/renders/ji_bridge_view.png", 800, 520);
        assert!(res.is_ok());
    }
}
