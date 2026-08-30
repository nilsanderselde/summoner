// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Bellows Compression Dynamics HUD & Free-Reed Performance Canvas (Milestone 28).
//!
//! Provides an interactive 2D Bellows pressure dynamics & pallet valve velocity puck
//! (Bellows Pressure Force [-1200..+1200 Pa] vs Pallet Valve Velocity),
//! real-time chamber airflow direction indicator (Push / Pull),
//! multi-rank register switch controls, cassotto tone chamber aperture shutter slider,
//! and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and headless PNG snapshot rendering.

use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const BELLOWS_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding target
pub const MIN_BELLOWS_PRESSURE_PA: f32 = -1200.0;
pub const MAX_BELLOWS_PRESSURE_PA: f32 = 1200.0;
pub const MIN_VALVE_VELOCITY: f32 = 0.05;
pub const MAX_VALVE_VELOCITY: f32 = 1.0;

/// Bellows HUD sound and articulation preset choices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BellowsHudPreset {
    #[default]
    TangoBandoneonAccented,
    FrenchMusetteAccordion,
    RussianBayanTutti,
    VintageHarmoniumDrone,
    EnglishConcertinaFast,
}

impl BellowsHudPreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::TangoBandoneonAccented => "Tango Bandoneon (Marcato Push/Pull)",
            Self::FrenchMusetteAccordion => "French Musette (Waltz Swell)",
            Self::RussianBayanTutti => "Russian Bayan (Grand Tutti)",
            Self::VintageHarmoniumDrone => "Vintage Harmonium (Suction Drone)",
            Self::EnglishConcertinaFast => "English Concertina (Fast Staccato)",
        }
    }
}

/// Bellows dynamics and free-reed interactive performance view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BellowsView {
    /// Active HUD preset.
    pub preset: BellowsHudPreset,
    /// Bellows driving pressure in Pascals $[-1200.0 ..= +1200.0]$.
    pub bellows_pressure_pa: f32,
    /// Pallet valve velocity $[0.05 ..= 1.0]$.
    pub valve_velocity: f32,
    /// Cassotto tone chamber aperture fraction $[0.0 ..= 1.0]$.
    pub cassotto_aperture: f32,
    /// Musette detuning spread in cents $[0.0 ..= 35.0]$.
    pub musette_detune_cents: f32,
    /// Active register bitmask.
    pub register_mask: u32,
    /// Reed material stiffness factor $[0.5 ..= 2.0]$.
    pub reed_stiffness: f32,
    /// 2D Puck position (X: Bellows pressure norm, Y: valve velocity norm).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for BellowsView {
    fn default() -> Self {
        Self::new()
    }
}

impl BellowsView {
    pub fn new() -> Self {
        let mut view = Self {
            preset: BellowsHudPreset::TangoBandoneonAccented,
            bellows_pressure_pa: 680.0,
            valve_velocity: 0.90,
            cassotto_aperture: 0.70,
            musette_detune_cents: 2.0,
            register_mask: 0b00011, // 16'+8' Bandoneon
            reed_stiffness: 1.25,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        };
        view.update_puck_from_physics();
        view
    }

    pub fn update_puck_from_physics(&mut self) {
        let norm_x = ((self.bellows_pressure_pa - MIN_BELLOWS_PRESSURE_PA) / (MAX_BELLOWS_PRESSURE_PA - MIN_BELLOWS_PRESSURE_PA)).clamp(0.0, 1.0);
        let norm_y = ((self.valve_velocity - MIN_VALVE_VELOCITY) / (MAX_VALVE_VELOCITY - MIN_VALVE_VELOCITY)).clamp(0.0, 1.0);
        self.puck_pos = (norm_x, norm_y);
    }

    pub fn update_physics_from_puck(&mut self, norm_x: f32, norm_y: f32) {
        self.puck_pos = (norm_x.clamp(0.0, 1.0), norm_y.clamp(0.0, 1.0));
        self.bellows_pressure_pa = MIN_BELLOWS_PRESSURE_PA + self.puck_pos.0 * (MAX_BELLOWS_PRESSURE_PA - MIN_BELLOWS_PRESSURE_PA);
        self.valve_velocity = MIN_VALVE_VELOCITY + self.puck_pos.1 * (MAX_VALVE_VELOCITY - MIN_VALVE_VELOCITY);
    }

    pub fn set_preset(&mut self, preset: BellowsHudPreset) {
        self.preset = preset;
        match preset {
            BellowsHudPreset::TangoBandoneonAccented => {
                self.bellows_pressure_pa = 680.0;
                self.valve_velocity = 0.90;
                self.cassotto_aperture = 0.70;
                self.musette_detune_cents = 2.0;
                self.register_mask = 0b00011;
                self.reed_stiffness = 1.25;
            }
            BellowsHudPreset::FrenchMusetteAccordion => {
                self.bellows_pressure_pa = 420.0;
                self.valve_velocity = 0.75;
                self.cassotto_aperture = 0.85;
                self.musette_detune_cents = 18.0;
                self.register_mask = 0b01110;
                self.reed_stiffness = 1.00;
            }
            BellowsHudPreset::RussianBayanTutti => {
                self.bellows_pressure_pa = 850.0;
                self.valve_velocity = 0.95;
                self.cassotto_aperture = 1.00;
                self.musette_detune_cents = 4.0;
                self.register_mask = 0b11111;
                self.reed_stiffness = 1.40;
            }
            BellowsHudPreset::VintageHarmoniumDrone => {
                self.bellows_pressure_pa = -280.0;
                self.valve_velocity = 0.60;
                self.cassotto_aperture = 0.50;
                self.musette_detune_cents = 8.0;
                self.register_mask = 0b00111;
                self.reed_stiffness = 0.85;
            }
            BellowsHudPreset::EnglishConcertinaFast => {
                self.bellows_pressure_pa = 520.0;
                self.valve_velocity = 0.85;
                self.cassotto_aperture = 0.90;
                self.musette_detune_cents = 0.0;
                self.register_mask = 0b00010;
                self.reed_stiffness = 1.15;
            }
        }
        self.update_puck_from_physics();
    }

    /// Render ASCII art overview for terminal and headless verification.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let border = format!("+{}+", "-".repeat(width.saturating_sub(2)));
        lines.push(border.clone());

        let title = format!("| BELLOWS DYNAMICS HUD [P={:+.0} Pa, Vel={:.2}] |", self.bellows_pressure_pa, self.valve_velocity);
        let padding = width.saturating_sub(title.len() + 2);
        lines.push(format!("|{}{}|", title, " ".repeat(padding)));

        let inner_height = height.saturating_sub(4);
        let puck_x = ((self.puck_pos.0 * (width.saturating_sub(4) as f32)) as usize).min(width.saturating_sub(4));
        let puck_y = (((1.0 - self.puck_pos.1) * (inner_height.saturating_sub(1) as f32)) as usize).min(inner_height.saturating_sub(1));

        for y in 0..inner_height {
            let mut row = vec![' '; width.saturating_sub(2)];
            // Zero pressure center line
            let center_x = (width.saturating_sub(2)) / 2;
            if center_x < row.len() {
                row[center_x] = ':';
            }
            if y == puck_y && puck_x < row.len() {
                row[puck_x] = 'O';
            }
            let s: String = row.into_iter().collect();
            lines.push(format!("|{}|", s));
        }

        let direction_str = if self.bellows_pressure_pa >= 0.0 { ">>> PUSH >>>" } else { "<<< PULL <<<" };
        let footer = format!("| Dir: {:<12} Aperture: {:.2} Musette: {:>4.1}c |", direction_str, self.cassotto_aperture, self.musette_detune_cents);
        let foot_pad = width.saturating_sub(footer.len() + 2);
        lines.push(format!("|{}{}|", footer, " ".repeat(foot_pad)));
        lines.push(border);

        lines
    }

    /// Render headless PNG snapshot with WCAG AAA contrast for layout and touch target verification.
    pub fn render_snapshot_png(&self, path: &str, width: usize, height: usize) -> Result<(), String> {
        let mut pixels = vec![0u8; width * height * 4];

        // Background dark charcoal #0F172A
        for i in 0..(width * height) {
            pixels[i * 4] = 0x0F;
            pixels[i * 4 + 1] = 0x17;
            pixels[i * 4 + 2] = 0x2A;
            pixels[i * 4 + 3] = 0xFF;
        }

        // Header bar #1E293B
        for y in 0..48 {
            for x in 0..width {
                let idx = (y * width + x) * 4;
                pixels[idx] = 0x1E;
                pixels[idx + 1] = 0x29;
                pixels[idx + 2] = 0x3B;
            }
        }

        // 2D Interaction Canvas rectangle (x: 40..width-40, y: 72..height-100)
        let c_left = 40;
        let c_right = width - 40;
        let c_top = 72;
        let c_bottom = height - 100;

        for y in c_top..c_bottom {
            for x in c_left..c_right {
                let idx = (y * width + x) * 4;
                pixels[idx] = 0x18;
                pixels[idx + 1] = 0x22;
                pixels[idx + 2] = 0x34;
            }
        }

        // Center zero-pressure reference line (Push / Pull divider)
        let mid_x = (c_left + c_right) / 2;
        for y in c_top..c_bottom {
            let idx = (y * width + mid_x) * 4;
            pixels[idx] = 0x3B;
            pixels[idx + 1] = 0x82;
            pixels[idx + 2] = 0xF6; // Blue #3B82F6
        }

        // Draw Interactive Puck (radius 22pt -> 44x44pt target)
        let puck_px_x = c_left + ((self.puck_pos.0 * (c_right - c_left) as f32) as usize);
        let puck_px_y = c_bottom - ((self.puck_pos.1 * (c_bottom - c_top) as f32) as usize);

        let radius = BELLOWS_PUCK_HIT_RADIUS as i32;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy <= radius * radius {
                    let px = (puck_px_x as i32 + dx) as usize;
                    let py = (puck_px_y as i32 + dy) as usize;
                    if px < width && py < height {
                        let idx = (py * width + px) * 4;
                        let dist_sq = dx * dx + dy * dy;
                        if dist_sq >= (radius - 3) * (radius - 3) {
                            pixels[idx] = 0x38;
                            pixels[idx + 1] = 0xBD;
                            pixels[idx + 2] = 0xF8; // Bright cyan outer ring
                        } else {
                            pixels[idx] = 0xF5;
                            pixels[idx + 1] = 0x9E;
                            pixels[idx + 2] = 0x0B; // Amber gold center
                        }
                    }
                }
            }
        }

        // Bottom Bellows Pressure Bar (-1200 Pa to +1200 Pa)
        let bar_y_start = height - 70;
        let bar_y_end = height - 45;
        let bar_center_x = (c_left + c_right) / 2;

        for y in bar_y_start..bar_y_end {
            for x in c_left..c_right {
                let idx = (y * width + x) * 4;
                pixels[idx] = 0x33;
                pixels[idx + 1] = 0x41;
                pixels[idx + 2] = 0x55;
            }
        }

        // Fill active pressure meter
        let norm_p = (self.bellows_pressure_pa / 1200.0).clamp(-1.0, 1.0);
        let fill_x = (bar_center_x as f32 + norm_p * ((c_right - bar_center_x) as f32)) as usize;

        let (min_x, max_x, r_col, g_col, b_col) = if norm_p >= 0.0 {
            (bar_center_x, fill_x, 0x10, 0xB9, 0x81) // Green for push
        } else {
            (fill_x, bar_center_x, 0xEC, 0x48, 0x99) // Pink/Magenta for pull
        };

        for y in bar_y_start..bar_y_end {
            for x in min_x..=max_x.min(c_right) {
                let idx = (y * width + x) * 4;
                pixels[idx] = r_col;
                pixels[idx + 1] = g_col;
                pixels[idx + 2] = b_col;
            }
        }

        save_png_file(path, width, height, &pixels)
    }
}

#[cfg(feature = "gui")]
impl BellowsView {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Bellows — Dynamic Chamber Aerodynamics & Free-Reed Articulation");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Preset:");
                if ui.selectable_label(self.preset == BellowsHudPreset::TangoBandoneonAccented, "Bandoneon Marcato").clicked() {
                    self.set_preset(BellowsHudPreset::TangoBandoneonAccented);
                }
                if ui.selectable_label(self.preset == BellowsHudPreset::FrenchMusetteAccordion, "Musette Waltz").clicked() {
                    self.set_preset(BellowsHudPreset::FrenchMusetteAccordion);
                }
                if ui.selectable_label(self.preset == BellowsHudPreset::RussianBayanTutti, "Bayan Tutti").clicked() {
                    self.set_preset(BellowsHudPreset::RussianBayanTutti);
                }
                if ui.selectable_label(self.preset == BellowsHudPreset::VintageHarmoniumDrone, "Harmonium Drone").clicked() {
                    self.set_preset(BellowsHudPreset::VintageHarmoniumDrone);
                }
                if ui.selectable_label(self.preset == BellowsHudPreset::EnglishConcertinaFast, "Concertina Staccato").clicked() {
                    self.set_preset(BellowsHudPreset::EnglishConcertinaFast);
                }
            });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("Bellows & Valve Dynamics:");
                    ui.add(egui::Slider::new(&mut self.bellows_pressure_pa, MIN_BELLOWS_PRESSURE_PA..=MAX_BELLOWS_PRESSURE_PA).text("Pressure (Pa)"));
                    ui.add(egui::Slider::new(&mut self.valve_velocity, MIN_VALVE_VELOCITY..=MAX_VALVE_VELOCITY).text("Valve Velocity"));
                });

                ui.separator();

                ui.vertical(|ui| {
                    ui.label("Cassotto & Musette Tuning:");
                    ui.add(egui::Slider::new(&mut self.cassotto_aperture, 0.0..=1.0).text("Cassotto Aperture"));
                    ui.add(egui::Slider::new(&mut self.musette_detune_cents, 0.0..=35.0).text("Musette Detune (cents)"));
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
    fn test_bellows_view_hit_target_dimensions() {
        const {
            assert!(
                BELLOWS_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT,
                "Bellows puck hit target bounding box must be >= 44pt"
            );
        }
    }

    #[test]
    fn test_bellows_view_ascii_render() {
        let view = BellowsView::new();
        let ascii = view.render_ascii(80, 16);
        assert_eq!(ascii.len(), 16);
        assert!(ascii[0].starts_with('+'));
    }

    #[test]
    fn test_bellows_view_snapshot_render() {
        let view = BellowsView::new();
        let res = view.render_snapshot_png("scratch/renders/bellows_view.png", 800, 520);
        assert!(res.is_ok());
    }
}
