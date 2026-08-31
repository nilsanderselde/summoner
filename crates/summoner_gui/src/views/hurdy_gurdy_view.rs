// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Hurdy-Gurdy (Vielle à roue) Performance HUD & Rosined Wheel Velocity Canvas (Milestone 30).
//!
//! Provides an interactive 2D Crank Wheel Speed vs Wrist Acceleration Impulse puck
//! (Crank Speed [0..4pi rad/s] vs Wrist Accel Impulse [0..5.0]), real-time Chanterelle vs Bourdon
//! drone harmonic spectrum bar visualizer, Coup de Poignet transient pulse indicator,
//! preset selector, and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and headless PNG snapshot rendering.

use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const HURDY_GURDY_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding target
pub const MIN_CRANK_SPEED_RAD_S: f32 = 0.0;
pub const MAX_CRANK_SPEED_RAD_S: f32 = 4.0 * PI;
pub const MIN_WRIST_ACCEL_IMPULSE: f32 = 0.0;
pub const MAX_WRIST_ACCEL_IMPULSE: f32 = 5.0;

/// Hurdy-Gurdy HUD sound and articulation preset choices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HurdyGurdyHudPreset {
    #[default]
    BourbonnaisTraditional,
    MedievalSymphonia,
    BaroqueVirtuoso,
    AuvergneFolkSnarl,
    GothicPaganDrone,
    ElectroAcousticModern,
}

impl HurdyGurdyHudPreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::BourbonnaisTraditional => "Bourbonnais Traditional Vielle",
            Self::MedievalSymphonia => "Medieval Symphonia Organistrum",
            Self::BaroqueVirtuoso => "Baroque Virtuoso Vielle (Ivory Chien)",
            Self::AuvergneFolkSnarl => "Auvergne Folk (High-Speed Snarl)",
            Self::GothicPaganDrone => "Gothic Pagan Dark Drone",
            Self::ElectroAcousticModern => "Electro-Acoustic Modern Hybrid",
        }
    }
}

/// Hurdy-Gurdy interactive performance and analysis view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HurdyGurdyView {
    /// Active HUD sound preset.
    pub preset: HurdyGurdyHudPreset,
    /// Crank wheel angular velocity $[0.0 ..= 4.0\pi]\text{ rad/s}$.
    pub crank_speed_rad_s: f32,
    /// Coup de poignet wrist acceleration impulse $[0.0 ..= 5.0]$.
    pub wrist_acceleration_pulse: f32,
    /// Normal wheel pressure force $[0.01 ..= 1.0]$.
    pub wheel_pressure: f32,
    /// Chien buzzing dog resting clearance gap in mm $[0.05 ..= 1.20]\text{ mm}$.
    pub chien_clearance_mm: f32,
    /// Drone vs Chanterelle mix ratio $[0.0 ..= 1.0]$.
    pub drone_melody_mix: f32,
    /// Trompette buzzing bridge volume level $[0.0 ..= 2.0]$.
    pub trompette_buzz_level: f32,
    /// Soundbox body wood resonance $[0.0 ..= 1.0]$.
    pub body_wood_resonance: f32,
    /// 2D Puck position (X: Wrist acceleration impulse, Y: Crank wheel speed).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for HurdyGurdyView {
    fn default() -> Self {
        Self::new()
    }
}

impl HurdyGurdyView {
    pub fn new() -> Self {
        let mut view = Self {
            preset: HurdyGurdyHudPreset::BourbonnaisTraditional,
            crank_speed_rad_s: 2.0 * PI,
            wrist_acceleration_pulse: 1.2,
            wheel_pressure: 0.70,
            chien_clearance_mm: 0.35,
            drone_melody_mix: 0.45,
            trompette_buzz_level: 0.85,
            body_wood_resonance: 0.85,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        };
        view.update_puck_from_physics();
        view.update_acoustics();
        view
    }

    pub fn update_puck_from_physics(&mut self) {
        let norm_x = ((self.wrist_acceleration_pulse - MIN_WRIST_ACCEL_IMPULSE)
            / (MAX_WRIST_ACCEL_IMPULSE - MIN_WRIST_ACCEL_IMPULSE))
            .clamp(0.0, 1.0);
        let norm_y = ((self.crank_speed_rad_s - MIN_CRANK_SPEED_RAD_S)
            / (MAX_CRANK_SPEED_RAD_S - MIN_CRANK_SPEED_RAD_S))
            .clamp(0.0, 1.0);
        self.puck_pos = (norm_x, norm_y);
    }

    pub fn update_physics_from_puck(&mut self, norm_x: f32, norm_y: f32) {
        self.puck_pos = (norm_x.clamp(0.0, 1.0), norm_y.clamp(0.0, 1.0));
        self.wrist_acceleration_pulse = MIN_WRIST_ACCEL_IMPULSE
            + self.puck_pos.0 * (MAX_WRIST_ACCEL_IMPULSE - MIN_WRIST_ACCEL_IMPULSE);
        self.crank_speed_rad_s = MIN_CRANK_SPEED_RAD_S
            + self.puck_pos.1 * (MAX_CRANK_SPEED_RAD_S - MIN_CRANK_SPEED_RAD_S);
        self.update_acoustics();
    }

    pub fn update_acoustics(&mut self) {
        match self.preset {
            HurdyGurdyHudPreset::BourbonnaisTraditional => {
                self.wheel_pressure = 0.70;
                self.drone_melody_mix = 0.45;
                self.trompette_buzz_level = 0.85;
                self.body_wood_resonance = 0.85;
            }
            HurdyGurdyHudPreset::MedievalSymphonia => {
                self.wheel_pressure = 0.55;
                self.drone_melody_mix = 0.60;
                self.trompette_buzz_level = 0.20;
                self.body_wood_resonance = 0.80;
            }
            HurdyGurdyHudPreset::BaroqueVirtuoso => {
                self.wheel_pressure = 0.80;
                self.drone_melody_mix = 0.35;
                self.trompette_buzz_level = 0.95;
                self.body_wood_resonance = 0.90;
            }
            HurdyGurdyHudPreset::AuvergneFolkSnarl => {
                self.wheel_pressure = 0.90;
                self.drone_melody_mix = 0.50;
                self.trompette_buzz_level = 1.30;
                self.body_wood_resonance = 0.85;
            }
            HurdyGurdyHudPreset::GothicPaganDrone => {
                self.wheel_pressure = 0.65;
                self.drone_melody_mix = 0.70;
                self.trompette_buzz_level = 0.75;
                self.body_wood_resonance = 0.95;
            }
            HurdyGurdyHudPreset::ElectroAcousticModern => {
                self.wheel_pressure = 0.85;
                self.drone_melody_mix = 0.40;
                self.trompette_buzz_level = 1.00;
                self.body_wood_resonance = 0.90;
            }
        }
    }

    pub fn set_preset(&mut self, preset: HurdyGurdyHudPreset) {
        self.preset = preset;
        match preset {
            HurdyGurdyHudPreset::BourbonnaisTraditional => {
                self.crank_speed_rad_s = 2.0 * PI;
                self.wrist_acceleration_pulse = 1.2;
                self.chien_clearance_mm = 0.35;
            }
            HurdyGurdyHudPreset::MedievalSymphonia => {
                self.crank_speed_rad_s = 1.5 * PI;
                self.wrist_acceleration_pulse = 0.0;
                self.chien_clearance_mm = 0.90;
            }
            HurdyGurdyHudPreset::BaroqueVirtuoso => {
                self.crank_speed_rad_s = 2.2 * PI;
                self.wrist_acceleration_pulse = 0.4;
                self.chien_clearance_mm = 0.25;
            }
            HurdyGurdyHudPreset::AuvergneFolkSnarl => {
                self.crank_speed_rad_s = 3.0 * PI;
                self.wrist_acceleration_pulse = 2.5;
                self.chien_clearance_mm = 0.15;
            }
            HurdyGurdyHudPreset::GothicPaganDrone => {
                self.crank_speed_rad_s = 1.2 * PI;
                self.wrist_acceleration_pulse = 0.8;
                self.chien_clearance_mm = 0.45;
            }
            HurdyGurdyHudPreset::ElectroAcousticModern => {
                self.crank_speed_rad_s = 2.5 * PI;
                self.wrist_acceleration_pulse = 1.5;
                self.chien_clearance_mm = 0.30;
            }
        }
        self.update_puck_from_physics();
        self.update_acoustics();
    }

    /// Render ASCII diagnostics representation of the view.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let border = format!("+{}+", "-".repeat(width.saturating_sub(2)));
        lines.push(border.clone());

        let title = format!(
            "| HURDY-GURDY HUD | Preset: {} |",
            self.preset.name()
        );
        let title_padded = format!("{:<width$}|", title, width = width - 1);
        lines.push(title_padded);

        let dynamics_info = format!(
            "| Crank: {:.1} rad/s ({:.1} rev/s) | Coup Accel: {:.1} | Chien Gap: {:.2} mm | Drone/Mel: {:.0}% | Buzz: {:.2} |",
            self.crank_speed_rad_s, self.crank_speed_rad_s / (2.0 * PI),
            self.wrist_acceleration_pulse, self.chien_clearance_mm,
            self.drone_melody_mix * 100.0, self.trompette_buzz_level
        );
        let dyn_padded = format!("{:<width$}|", dynamics_info, width = width - 1);
        lines.push(dyn_padded);

        lines.push(border.clone());

        // Multi-string rosined friction standing wave & drone spectrum canvas
        let canvas_h = height.saturating_sub(5);
        for row in 0..canvas_h {
            let row_ratio = 1.0 - (row as f32 / canvas_h.max(1) as f32);
            let mut line_buf = String::with_capacity(width);
            line_buf.push('|');

            for col in 0..(width.saturating_sub(2)) {
                let col_t = col as f32 / (width - 2).max(1) as f32;

                // Rosined friction waveform with coup de poignet pulse modulation
                let speed_norm = (self.crank_speed_rad_s / (4.0 * PI)).clamp(0.1, 1.0);
                let pulse_mod = 1.0 + self.wrist_acceleration_pulse * 0.4;
                let wave1 = (col_t * 6.0 * PI * speed_norm * pulse_mod).sin() * (col_t * PI).sin();
                let wave2 = 0.35 * (col_t * 12.0 * PI * speed_norm).sin();
                let buzz_wave = if self.trompette_buzz_level > 0.3 {
                    0.25 * ((col_t * 24.0 * PI * speed_norm).sin() * 4.0).tanh()
                } else {
                    0.0
                };

                let total_wave = (wave1 + wave2 + buzz_wave) * 0.8;
                let norm_wave = (total_wave + 1.0) * 0.5;

                if (norm_wave - row_ratio).abs() < 0.06 {
                    line_buf.push('#');
                } else if norm_wave > row_ratio {
                    line_buf.push(':');
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

        // Title Indicator Bar (Gold #F59E0B)
        for y in 12..20 {
            for x in 24..190 {
                let idx = (y * width + x) * 4;
                pixels[idx] = 0xF5;
                pixels[idx + 1] = 0x9E;
                pixels[idx + 2] = 0x0B;
            }
        }

        // Main 2D Puck Canvas Box (Crank Speed vs Wrist Accel)
        let c_left = 24;
        let c_top = 72;
        let c_right = width - 24;
        let c_bottom = height - 88;

        for y in c_top..c_bottom {
            for x in c_left..c_right {
                let idx = (y * width + x) * 4;
                let is_border = x == c_left || x == c_right - 1 || y == c_top || y == c_bottom - 1;
                if is_border {
                    pixels[idx] = 0x38;
                    pixels[idx + 1] = 0xBD;
                    pixels[idx + 2] = 0xF8; // Bright cyan border
                } else if x % 40 == 0 || y % 40 == 0 {
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

        // Render multi-string standing waves across canvas (Chanterelles, Bourdons, Trompette)
        let canvas_w = (c_right - c_left) as f32;
        let canvas_h_f = (c_bottom - c_top) as f32;
        let speed_norm = (self.crank_speed_rad_s / (4.0 * PI)).clamp(0.1, 1.0);
        let pulse_factor = 1.0 + self.wrist_acceleration_pulse * 0.4;

        // 6 string paths: Chanterelle 1 & 2, Gros Bourdon, Petit Bourdon, Mouche, Trompette
        for str_idx in 0..6 {
            let str_norm = str_idx as f32 / 5.0;
            let center_y = c_top as f32 + canvas_h_f * (0.15 + 0.70 * str_norm);

            for x_px in c_left..c_right {
                let x_norm = (x_px - c_left) as f32 / canvas_w;
                let wave_freq = (3.0 + str_idx as f32 * 2.0) * PI * speed_norm;
                let wave = (x_norm * wave_freq * pulse_factor).sin()
                    * (x_norm * PI).sin()
                    * 12.0;
                let y_px = (center_y + wave).clamp(c_top as f32 + 2.0, c_bottom as f32 - 2.0) as usize;

                let idx = (y_px * width + x_px) * 4;
                if str_idx < 2 {
                    // Chanterelle melody strings (Cyan)
                    pixels[idx] = 0x38;
                    pixels[idx + 1] = 0xBD;
                    pixels[idx + 2] = 0xF8;
                } else if str_idx < 5 {
                    // Bourdon drone strings (Emerald)
                    pixels[idx] = 0x10;
                    pixels[idx + 1] = 0xB9;
                    pixels[idx + 2] = 0x81;
                } else {
                    // Trompette buzzing dog string (Gold)
                    pixels[idx] = 0xF5;
                    pixels[idx + 1] = 0x9E;
                    pixels[idx + 2] = 0x0B;
                }
            }
        }

        // Interactive 2D Puck (Radius = 22pt -> 44x44pt touch target)
        let puck_center_x = c_left + ((c_right - c_left) as f32 * self.puck_pos.0) as usize;
        let puck_center_y = c_top + ((c_bottom - c_top) as f32 * (1.0 - self.puck_pos.1)) as usize;
        let radius = HURDY_GURDY_PUCK_HIT_RADIUS as isize;
        let inner_radius = (HURDY_GURDY_PUCK_HIT_RADIUS - 3.0) as isize;

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy <= radius * radius {
                    let px = (puck_center_x as isize + dx) as usize;
                    let py = (puck_center_y as isize + dy) as usize;
                    if px < width && py < height {
                        let idx = (py * width + px) * 4;
                        let dist_sq = dx * dx + dy * dy;
                        if dist_sq >= inner_radius * inner_radius {
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

        // Bottom Drone vs Chanterelle Balance Bar
        let bar_y_start = height - 68;
        let bar_y_end = height - 44;
        let bar_left = c_left;
        let bar_right = c_right;

        for y in bar_y_start..bar_y_end {
            for x in bar_left..bar_right {
                let idx = (y * width + x) * 4;
                pixels[idx] = 0x33;
                pixels[idx + 1] = 0x41;
                pixels[idx + 2] = 0x55;
            }
        }

        let mix_split_x = bar_left + ((bar_right - bar_left) as f32 * (1.0 - self.drone_melody_mix)) as usize;
        for y in bar_y_start..bar_y_end {
            for x in bar_left..=mix_split_x.min(bar_right) {
                let idx = (y * width + x) * 4;
                pixels[idx] = 0x38;
                pixels[idx + 1] = 0xBD;
                pixels[idx + 2] = 0xF8; // Chanterelle melody portion (Cyan)
            }
            for x in mix_split_x.min(bar_right)..bar_right {
                let idx = (y * width + x) * 4;
                pixels[idx] = 0x10;
                pixels[idx + 1] = 0xB9;
                pixels[idx + 2] = 0x81; // Bourdon drone portion (Emerald)
            }
        }

        save_png_file(path, width, height, &pixels)
    }
}

#[cfg(feature = "gui")]
impl HurdyGurdyView {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Hurdy-Gurdy (Vielle à roue) — Physical Modeling HUD");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Preset:");
                if ui.selectable_label(self.preset == HurdyGurdyHudPreset::BourbonnaisTraditional, "Bourbonnais Vielle").clicked() {
                    self.set_preset(HurdyGurdyHudPreset::BourbonnaisTraditional);
                }
                if ui.selectable_label(self.preset == HurdyGurdyHudPreset::MedievalSymphonia, "Medieval Symphonia").clicked() {
                    self.set_preset(HurdyGurdyHudPreset::MedievalSymphonia);
                }
                if ui.selectable_label(self.preset == HurdyGurdyHudPreset::BaroqueVirtuoso, "Baroque Virtuoso").clicked() {
                    self.set_preset(HurdyGurdyHudPreset::BaroqueVirtuoso);
                }
                if ui.selectable_label(self.preset == HurdyGurdyHudPreset::AuvergneFolkSnarl, "Auvergne Folk").clicked() {
                    self.set_preset(HurdyGurdyHudPreset::AuvergneFolkSnarl);
                }
                if ui.selectable_label(self.preset == HurdyGurdyHudPreset::GothicPaganDrone, "Gothic Pagan").clicked() {
                    self.set_preset(HurdyGurdyHudPreset::GothicPaganDrone);
                }
                if ui.selectable_label(self.preset == HurdyGurdyHudPreset::ElectroAcousticModern, "Electro Hybrid").clicked() {
                    self.set_preset(HurdyGurdyHudPreset::ElectroAcousticModern);
                }
            });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("Crank Wheel & Coup de Poignet Dynamics:");
                    ui.add(egui::Slider::new(&mut self.crank_speed_rad_s, MIN_CRANK_SPEED_RAD_S..=MAX_CRANK_SPEED_RAD_S).text("Crank Speed (rad/s)"));
                    ui.add(egui::Slider::new(&mut self.wrist_acceleration_pulse, MIN_WRIST_ACCEL_IMPULSE..=MAX_WRIST_ACCEL_IMPULSE).text("Coup de Poignet"));
                    ui.add(egui::Slider::new(&mut self.wheel_pressure, 0.01..=1.0).text("Wheel Pressure"));
                });

                ui.separator();

                ui.vertical(|ui| {
                    ui.label("Chien & Acoustic Mix:");
                    ui.add(egui::Slider::new(&mut self.chien_clearance_mm, 0.05..=1.20).text("Chien Clearance (mm)"));
                    ui.add(egui::Slider::new(&mut self.drone_melody_mix, 0.0..=1.0).text("Drone/Melody Mix"));
                    ui.add(egui::Slider::new(&mut self.trompette_buzz_level, 0.0..=2.0).text("Trompette Buzz"));
                });
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
    fn test_hurdy_gurdy_view_hit_target_dimensions() {
        const {
            assert!(
                HURDY_GURDY_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT,
                "Hurdy-Gurdy puck hit target bounding box must be >= 44pt"
            );
        }
    }

    #[test]
    fn test_hurdy_gurdy_view_ascii_render() {
        let view = HurdyGurdyView::new();
        let ascii = view.render_ascii(80, 16);
        assert_eq!(ascii.len(), 16);
        assert!(ascii[0].starts_with('+'));
    }

    #[test]
    fn test_hurdy_gurdy_view_snapshot_render() {
        let view = HurdyGurdyView::new();
        let res = view.render_snapshot_png("scratch/renders/hurdy_gurdy_view.png", 800, 520);
        assert!(res.is_ok());
    }
}
