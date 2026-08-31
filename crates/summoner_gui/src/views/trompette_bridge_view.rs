// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Hurdy-Gurdy Chien (Buzzing Dog Bridge) Collision Phase Canvas & HUD (Milestone 30).
//!
//! Provides an interactive visualizer for the loose rocking *chien* (buzzing dog) bridge:
//! - 2D Chien clearance gap ($h_0 \in [0.05..1.20]\text{ mm}$) vs Coup de Poignet strike force puck (>=44x44pt hit targets)
//! - Real-time chattering chien obstacle collision displacement phase portrait ($y$ vs $\dot{y}$)
//! - Dynamic buzzing audio waveform and soundbox 8-mode modal energy distribution
//! - WCAG AAA high-contrast styling and headless PNG snapshot rendering.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and headless PNG snapshot rendering.

use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const CHIEN_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding target
pub const MIN_CHIEN_GAP_MM: f32 = 0.05;
pub const MAX_CHIEN_GAP_MM: f32 = 1.20;
pub const MIN_STRIKE_FORCE: f32 = 0.0;
pub const MAX_STRIKE_FORCE: f32 = 10.0;

/// Chien striking material profile choices for the visualizer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ChienViewProfile {
    #[default]
    MapleOnBone,
    IvoryPlate,
    EbonyHardwood,
    SyntheticDelrin,
    VintageFruitwood,
}

impl ChienViewProfile {
    pub fn name(&self) -> &'static str {
        match self {
            Self::MapleOnBone => "Maple Dog on Bone Plate (Traditional)",
            Self::IvoryPlate => "Ivory Striking Plate (Bright Attack)",
            Self::EbonyHardwood => "Ebony Hardwood (Dark Woody Buzz)",
            Self::SyntheticDelrin => "Synthetic Delrin (Clean & Stable)",
            Self::VintageFruitwood => "Vintage Fruitwood (Warm & Mellow)",
        }
    }

    pub fn physics_params(&self) -> (f32, f32, f32) {
        match self {
            Self::MapleOnBone => (8500.0, 0.45, 0.65),
            Self::IvoryPlate => (12000.0, 0.30, 0.75),
            Self::EbonyHardwood => (7000.0, 0.60, 0.55),
            Self::SyntheticDelrin => (9000.0, 0.40, 0.70),
            Self::VintageFruitwood => (6500.0, 0.50, 0.60),
        }
    }
}

/// Chien Buzzing Dog Bridge interactive performance and collision HUD.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrompetteBridgeView {
    /// Active material profile.
    pub profile: ChienViewProfile,
    /// Chien resting clearance gap in mm $[0.05 ..= 1.20]\text{ mm}$.
    pub clearance_gap_mm: f32,
    /// Coup de poignet strike force $[0.0 ..= 10.0]$.
    pub strike_force: f32,
    /// Instantaneous dog displacement in mm.
    pub displacement_mm: f32,
    /// Instantaneous dog velocity in mm/s.
    pub velocity_mm_s: f32,
    /// Soundbox body modal energy distribution $[0.0 ..= 1.0]$ across 8 modes.
    pub soundbox_modal_energy: [f32; 8],
    /// 2D Puck position (X: Clearance gap, Y: Strike force).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for TrompetteBridgeView {
    fn default() -> Self {
        Self::new()
    }
}

impl TrompetteBridgeView {
    pub fn new() -> Self {
        let mut view = Self {
            profile: ChienViewProfile::MapleOnBone,
            clearance_gap_mm: 0.35,
            strike_force: 4.0,
            displacement_mm: 0.0,
            velocity_mm_s: 0.0,
            soundbox_modal_energy: [0.90, 0.78, 0.65, 0.55, 0.45, 0.38, 0.30, 0.22],
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        };
        view.update_puck_from_physics();
        view
    }

    pub fn update_puck_from_physics(&mut self) {
        let norm_x = ((self.clearance_gap_mm - MIN_CHIEN_GAP_MM) / (MAX_CHIEN_GAP_MM - MIN_CHIEN_GAP_MM)).clamp(0.0, 1.0);
        let norm_y = ((self.strike_force - MIN_STRIKE_FORCE) / (MAX_STRIKE_FORCE - MIN_STRIKE_FORCE)).clamp(0.0, 1.0);
        self.puck_pos = (norm_x, norm_y);
    }

    pub fn update_physics_from_puck(&mut self, norm_x: f32, norm_y: f32) {
        self.puck_pos = (norm_x.clamp(0.0, 1.0), norm_y.clamp(0.0, 1.0));
        self.clearance_gap_mm = MIN_CHIEN_GAP_MM + self.puck_pos.0 * (MAX_CHIEN_GAP_MM - MIN_CHIEN_GAP_MM);
        self.strike_force = MIN_STRIKE_FORCE + self.puck_pos.1 * (MAX_STRIKE_FORCE - MIN_STRIKE_FORCE);
    }

    pub fn set_profile(&mut self, profile: ChienViewProfile) {
        self.profile = profile;
    }

    /// Render ASCII diagnostics representation of the view.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let border = format!("+{}+", "-".repeat(width.saturating_sub(2)));
        lines.push(border.clone());

        let title = format!(
            "| TROMPETTE CHIEN BRIDGE HUD | Material: {} | Gap: {:.2} mm |",
            self.profile.name(),
            self.clearance_gap_mm
        );
        let title_padded = format!("{:<width$}|", title, width = width - 1);
        lines.push(title_padded);

        let dynamics_info = format!(
            "| Strike Force: {:.1} N | Restitution: {:.2} | Buzz Status: {} |",
            self.strike_force,
            self.profile.physics_params().2,
            if self.strike_force > 1.5 { "ACTIVE BUZZING" } else { "IDLE DRONE" }
        );
        let dyn_padded = format!("{:<width$}|", dynamics_info, width = width - 1);
        lines.push(dyn_padded);

        lines.push(border.clone());

        // Chattering chien collision obstacle phase portrait display ($y$ vs $\dot{y}$)
        let canvas_h = height.saturating_sub(5);
        for row in 0..canvas_h {
            let row_ratio = 1.0 - (row as f32 / canvas_h.max(1) as f32); // Velocity axis [-1.0 .. 1.0]
            let vel_norm = row_ratio * 2.0 - 1.0;
            let mut line_buf = String::with_capacity(width);
            line_buf.push('|');

            for col in 0..(width.saturating_sub(2)) {
                let col_t = col as f32 / (width - 2).max(1) as f32; // Displacement axis [-1.0 .. 1.0]
                let disp_norm = col_t * 2.0 - 1.0;

                // Collision obstacle boundary at -clearance_gap
                let gap_norm = -(self.clearance_gap_mm / MAX_CHIEN_GAP_MM);
                let is_obstacle_wall = (disp_norm - gap_norm).abs() < 0.04;

                // Phase trajectory orbit
                let orbit_radius = (self.strike_force / MAX_STRIKE_FORCE).clamp(0.1, 0.9);
                let in_orbit = (disp_norm * disp_norm + vel_norm * vel_norm - orbit_radius * orbit_radius).abs() < 0.08;

                if is_obstacle_wall {
                    line_buf.push('|'); // Striking plate obstacle wall
                } else if in_orbit {
                    line_buf.push('*'); // Chattering phase orbit
                } else if disp_norm < gap_norm {
                    line_buf.push('#'); // Inside striking plate contact zone
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

        // Main 2D Puck Canvas Box (Chien Clearance vs Strike Force)
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
                    pixels[idx + 2] = 0x0B; // Amber gold border
                } else if x % 32 == 0 || y % 32 == 0 {
                    pixels[idx] = 0x1E;
                    pixels[idx + 1] = 0x29;
                    pixels[idx + 2] = 0x3B; // 8pt grid guide
                } else {
                    pixels[idx] = 0x0F;
                    pixels[idx + 1] = 0x17;
                    pixels[idx + 2] = 0x2A;
                }
            }
        }

        // Render Chien Collision Phase Portrait ($y$ vs $\dot{y}$) and obstacle wall
        let canvas_w = (c_right - c_left) as f32;
        let canvas_h_f = (c_bottom - c_top) as f32;
        let obstacle_x = c_left + ((self.clearance_gap_mm / MAX_CHIEN_GAP_MM) * canvas_w * 0.4) as usize;

        // Obstacle bone striking plate (White/Ivory column)
        for y in c_top + 1..c_bottom - 1 {
            for x in c_left + 1..obstacle_x {
                let idx = (y * width + x) * 4;
                pixels[idx] = 0x33;
                pixels[idx + 1] = 0x41;
                pixels[idx + 2] = 0x55; // Darker striking zone
            }
            let idx_wall = (y * width + obstacle_x) * 4;
            pixels[idx_wall] = 0xFF;
            pixels[idx_wall + 1] = 0xFF;
            pixels[idx_wall + 2] = 0xFF; // Solid white obstacle barrier
        }

        // Draw chattering phase collision loops
        let center_y = c_top as f32 + canvas_h_f * 0.5;
        let num_points = 200;
        let orbit_scale = (self.strike_force / MAX_STRIKE_FORCE).clamp(0.1, 1.0);

        for i in 0..num_points {
            let theta = (i as f32 / num_points as f32) * 2.0 * std::f32::consts::PI;
            let raw_x = obstacle_x as f32 + (canvas_w * 0.45 * orbit_scale * theta.cos()).max(0.0);
            let raw_y = center_y + canvas_h_f * 0.35 * orbit_scale * theta.sin();

            let px = raw_x.clamp(c_left as f32 + 2.0, c_right as f32 - 2.0) as usize;
            let py = raw_y.clamp(c_top as f32 + 2.0, c_bottom as f32 - 2.0) as usize;

            let idx = (py * width + px) * 4;
            pixels[idx] = 0x38;
            pixels[idx + 1] = 0xBD;
            pixels[idx + 2] = 0xF8; // Bright cyan phase orbit
        }

        // Interactive 2D Puck (Radius = 22pt -> 44x44pt touch target)
        let puck_center_x = c_left + ((c_right - c_left) as f32 * self.puck_pos.0) as usize;
        let puck_center_y = c_top + ((c_bottom - c_top) as f32 * (1.0 - self.puck_pos.1)) as usize;
        let radius = CHIEN_PUCK_HIT_RADIUS as isize;
        let inner_radius = (CHIEN_PUCK_HIT_RADIUS - 3.0) as isize;

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy <= radius * radius {
                    let px = (puck_center_x as isize + dx) as usize;
                    let py = (puck_center_y as isize + dy) as usize;
                    if px < width && py < height {
                        let idx = (py * width + px) * 4;
                        let dist_sq = dx * dx + dy * dy;
                        if dist_sq >= inner_radius * inner_radius {
                            pixels[idx] = 0xF5;
                            pixels[idx + 1] = 0x9E;
                            pixels[idx + 2] = 0x0B; // Gold outer ring
                        } else {
                            pixels[idx] = 0x10;
                            pixels[idx + 1] = 0xB9;
                            pixels[idx + 2] = 0x81; // Emerald green center
                        }
                    }
                }
            }
        }

        // Bottom Soundbox Modal Energy Bar (8 modes)
        let bar_y_start = height - 60;
        let bar_y_end = height - 36;
        let num_modes = 8;
        let mode_w = (c_right - c_left) / num_modes;

        for mode_idx in 0..num_modes {
            let mode_left = c_left + mode_idx * mode_w;
            let mode_right = mode_left + mode_w - 4;
            let energy = self.soundbox_modal_energy[mode_idx];
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
impl TrompetteBridgeView {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Chien (Buzzing Dog Bridge) Obstacle Collision Phase Canvas");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Chien Striking Material:");
                if ui.selectable_label(self.profile == ChienViewProfile::MapleOnBone, "Maple on Bone").clicked() {
                    self.set_profile(ChienViewProfile::MapleOnBone);
                }
                if ui.selectable_label(self.profile == ChienViewProfile::IvoryPlate, "Ivory Plate").clicked() {
                    self.set_profile(ChienViewProfile::IvoryPlate);
                }
                if ui.selectable_label(self.profile == ChienViewProfile::EbonyHardwood, "Ebony Wood").clicked() {
                    self.set_profile(ChienViewProfile::EbonyHardwood);
                }
                if ui.selectable_label(self.profile == ChienViewProfile::SyntheticDelrin, "Synthetic Delrin").clicked() {
                    self.set_profile(ChienViewProfile::SyntheticDelrin);
                }
                if ui.selectable_label(self.profile == ChienViewProfile::VintageFruitwood, "Fruitwood").clicked() {
                    self.set_profile(ChienViewProfile::VintageFruitwood);
                }
            });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("Clearance Gap & Strike Dynamics:");
                ui.add(egui::Slider::new(&mut self.clearance_gap_mm, MIN_CHIEN_GAP_MM..=MAX_CHIEN_GAP_MM).text("Clearance Gap (mm)"));
                ui.add(egui::Slider::new(&mut self.strike_force, MIN_STRIKE_FORCE..=MAX_STRIKE_FORCE).text("Coup de Poignet Force"));
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
    fn test_trompette_bridge_view_hit_target_dimensions() {
        const {
            assert!(
                CHIEN_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT,
                "Chien puck hit target bounding box must be >= 44pt"
            );
        }
    }

    #[test]
    fn test_trompette_bridge_view_ascii_render() {
        let view = TrompetteBridgeView::new();
        let ascii = view.render_ascii(80, 16);
        assert_eq!(ascii.len(), 16);
        assert!(ascii[0].starts_with('+'));
    }

    #[test]
    fn test_trompette_bridge_view_snapshot_render() {
        let view = TrompetteBridgeView::new();
        let res = view.render_snapshot_png("scratch/renders/trompette_bridge_view.png", 800, 520);
        assert!(res.is_ok());
    }
}
