// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Electromechanical Clavinet HUD & String-Anvil Collision View (Milestone 23).
//!
//! Provides an interactive 2D anvil strike force vs pickup phase puck interface,
//! 4-way rocker tone switch bank, real-time rubber anvil string contact compression visualizer,
//! dual electromagnetic pickup waveform display, and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and headless PNG snapshot rendering.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const CLAV_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt bounding touch target

/// Clavinet sound profile selector for GUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GuiClavinetProfile {
    #[default]
    StevieSuperstition,
    OutPhaseFunkQuack,
    ClassicD6Clean,
    MellowChamber,
    ScreamingAutoWah,
    TwangyBridgeLead,
}

impl GuiClavinetProfile {
    pub fn name(&self) -> &'static str {
        match self {
            Self::StevieSuperstition => "STEVIE SUPERSTITION FUNK",
            Self::OutPhaseFunkQuack => "OUT-OF-PHASE FUNK QUACK",
            Self::ClassicD6Clean => "CLASSIC D6 CLEAN",
            Self::MellowChamber => "MELLOW CHAMBER CLAVINET",
            Self::ScreamingAutoWah => "SCREAMING FUNK AUTO-WAH",
            Self::TwangyBridgeLead => "TWANGY BRIDGE LEAD",
        }
    }
}

/// Dual electromagnetic pickup mode selector for GUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GuiClavinetPickupMode {
    NeckOnly,
    BridgeOnly,
    #[default]
    ParallelInPhase,
    ParallelOutOfPhase,
}

impl GuiClavinetPickupMode {
    pub fn name(&self) -> &'static str {
        match self {
            Self::NeckOnly => "NECK ONLY (SINGLE-COIL)",
            Self::BridgeOnly => "BRIDGE ONLY (TWANG)",
            Self::ParallelInPhase => "PARALLEL IN-PHASE",
            Self::ParallelOutOfPhase => "PARALLEL OUT-OF-PHASE (QUACK)",
        }
    }
}

/// Electromechanical Clavinet HUD & String-Anvil Collision View.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClavinetView {
    pub profile: GuiClavinetProfile,
    pub pickup_mode: GuiClavinetPickupMode,
    /// Rubber anvil hardness $[0.0 ..= 1.0]$.
    pub anvil_hardness: f32,
    /// Yarn wool damping amount $[0.0 ..= 1.0]$.
    pub yarn_damping: f32,
    /// Pickup mix blend $[0.0 ..= 1.0]$.
    pub pickup_blend: f32,
    /// Pickup phase angle in degrees $[0.0 ..= 180.0^\circ]$.
    pub pickup_phase_deg: f32,
    /// 4-way rocker tone switch bank.
    pub brilliant_switch: bool,
    pub treble_switch: bool,
    pub medium_switch: bool,
    pub soft_switch: bool,
    /// Dynamic Auto-Wah enabled.
    pub auto_wah_enabled: bool,
    pub auto_wah_sensitivity: f32,
    pub auto_wah_freq_hz: f32,
    pub auto_wah_resonance_q: f32,
    pub auto_wah_mix: f32,
    /// Master level $[0.0 ..= 2.0]$.
    pub master_level: f32,
    /// 2D Puck position (X: anvil_hardness, Y: pickup_phase_deg normalized).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for ClavinetView {
    fn default() -> Self {
        Self::new()
    }
}

impl ClavinetView {
    pub fn new() -> Self {
        let mut view = Self {
            profile: GuiClavinetProfile::StevieSuperstition,
            pickup_mode: GuiClavinetPickupMode::ParallelInPhase,
            anvil_hardness: 0.85,
            yarn_damping: 0.75,
            pickup_blend: 0.50,
            pickup_phase_deg: 0.0,
            brilliant_switch: true,
            treble_switch: true,
            medium_switch: true,
            soft_switch: false,
            auto_wah_enabled: false,
            auto_wah_sensitivity: 0.70,
            auto_wah_freq_hz: 350.0,
            auto_wah_resonance_q: 6.0,
            auto_wah_mix: 0.85,
            master_level: 0.90,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        };
        view.update_puck_from_physics();
        view
    }

    pub fn set_profile(&mut self, profile: GuiClavinetProfile) {
        self.profile = profile;
        match profile {
            GuiClavinetProfile::StevieSuperstition => {
                self.pickup_mode = GuiClavinetPickupMode::ParallelInPhase;
                self.anvil_hardness = 0.85;
                self.yarn_damping = 0.75;
                self.pickup_blend = 0.50;
                self.pickup_phase_deg = 0.0;
                self.brilliant_switch = true;
                self.treble_switch = true;
                self.medium_switch = true;
                self.soft_switch = false;
                self.auto_wah_enabled = false;
            }
            GuiClavinetProfile::OutPhaseFunkQuack => {
                self.pickup_mode = GuiClavinetPickupMode::ParallelOutOfPhase;
                self.anvil_hardness = 0.90;
                self.yarn_damping = 0.85;
                self.pickup_blend = 0.50;
                self.pickup_phase_deg = 180.0;
                self.brilliant_switch = true;
                self.treble_switch = true;
                self.medium_switch = false;
                self.soft_switch = false;
                self.auto_wah_enabled = false;
            }
            GuiClavinetProfile::ClassicD6Clean => {
                self.pickup_mode = GuiClavinetPickupMode::ParallelInPhase;
                self.anvil_hardness = 0.70;
                self.yarn_damping = 0.65;
                self.pickup_blend = 0.55;
                self.pickup_phase_deg = 0.0;
                self.brilliant_switch = true;
                self.treble_switch = false;
                self.medium_switch = true;
                self.soft_switch = false;
                self.auto_wah_enabled = false;
            }
            GuiClavinetProfile::MellowChamber => {
                self.pickup_mode = GuiClavinetPickupMode::NeckOnly;
                self.anvil_hardness = 0.45;
                self.yarn_damping = 0.50;
                self.pickup_blend = 0.0;
                self.pickup_phase_deg = 0.0;
                self.brilliant_switch = false;
                self.treble_switch = false;
                self.medium_switch = false;
                self.soft_switch = true;
                self.auto_wah_enabled = false;
            }
            GuiClavinetProfile::ScreamingAutoWah => {
                self.pickup_mode = GuiClavinetPickupMode::ParallelOutOfPhase;
                self.anvil_hardness = 0.90;
                self.yarn_damping = 0.80;
                self.pickup_blend = 0.50;
                self.pickup_phase_deg = 180.0;
                self.brilliant_switch = true;
                self.treble_switch = true;
                self.medium_switch = false;
                self.soft_switch = false;
                self.auto_wah_enabled = true;
                self.auto_wah_sensitivity = 0.88;
                self.auto_wah_freq_hz = 420.0;
                self.auto_wah_resonance_q = 9.5;
            }
            GuiClavinetProfile::TwangyBridgeLead => {
                self.pickup_mode = GuiClavinetPickupMode::BridgeOnly;
                self.anvil_hardness = 0.95;
                self.yarn_damping = 0.90;
                self.pickup_blend = 1.0;
                self.pickup_phase_deg = 0.0;
                self.brilliant_switch = true;
                self.treble_switch = true;
                self.medium_switch = false;
                self.soft_switch = false;
                self.auto_wah_enabled = false;
            }
        }
        self.update_puck_from_physics();
    }

    pub fn update_puck_from_physics(&mut self) {
        let norm_x = self.anvil_hardness.clamp(0.0, 1.0);
        let norm_y = (self.pickup_phase_deg / 180.0).clamp(0.0, 1.0);
        self.puck_pos = (norm_x, norm_y);
    }

    pub fn update_physics_from_puck(&mut self, norm_x: f32, norm_y: f32) {
        self.puck_pos = (norm_x.clamp(0.0, 1.0), norm_y.clamp(0.0, 1.0));
        self.anvil_hardness = self.puck_pos.0;
        self.pickup_phase_deg = self.puck_pos.1 * 180.0;
    }

    /// Evaluates rubber anvil impact force pulse curve for 32 points.
    pub fn evaluate_anvil_impact_curve(&self) -> [f32; 32] {
        let mut curve = [0.0f32; 32];
        let stiffness = 4.0 + self.anvil_hardness * 12.0;

        for (i, val) in curve.iter_mut().enumerate() {
            if i < 16 {
                let progress = i as f32 / 16.0;
                let pulse = (progress * PI).sin();
                let force = stiffness * pulse.powf(1.6) * 0.15;
                *val = force.clamp(0.0, 1.0);
            } else {
                *val = 0.0;
            }
        }
        curve
    }

    /// Evaluates dual pickup synthesized cancellation waveform display for 32 points.
    pub fn evaluate_pickup_waveform(&self) -> [f32; 32] {
        let mut wave = [0.0f32; 32];
        let phase_rad = self.pickup_phase_deg * (PI / 180.0);

        for (i, val) in wave.iter_mut().enumerate() {
            let t = (i as f32 / 32.0) * 2.0 * PI;
            let neck = (t * 2.0).sin() * 0.7 + (t * 3.0).sin() * 0.3;
            let bridge = (t * 2.0 + phase_rad).sin() * 0.6 + (t * 5.0).sin() * 0.4;

            let mixed = match self.pickup_mode {
                GuiClavinetPickupMode::NeckOnly => neck,
                GuiClavinetPickupMode::BridgeOnly => bridge,
                GuiClavinetPickupMode::ParallelInPhase => {
                    let w_neck = 1.0 - self.pickup_blend;
                    let w_bridge = self.pickup_blend;
                    neck * w_neck + bridge * w_bridge * phase_rad.cos()
                }
                GuiClavinetPickupMode::ParallelOutOfPhase => (neck - bridge) * 0.9,
            };

            *val = mixed.clamp(-1.0, 1.0);
        }
        wave
    }

    /// Hit tests the 2D anvil puck.
    pub fn hit_test_puck(&self, screen_pos: (f32, f32), canvas_rect: Rect) -> bool {
        let puck_screen_x = canvas_rect.x + self.puck_pos.0 * canvas_rect.width;
        let puck_screen_y = canvas_rect.y + (1.0 - self.puck_pos.1) * canvas_rect.height;
        let dx = screen_pos.0 - puck_screen_x;
        let dy = screen_pos.1 - puck_screen_y;
        (dx * dx + dy * dy).sqrt() <= CLAV_PUCK_HIT_RADIUS
    }

    /// Renders ASCII diagnostic representation.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let header = format!(
            "┌─ [CLAVINET HUD] {} ({}) ─┐",
            self.profile.name(),
            self.pickup_mode.name()
        );
        let pad = width.saturating_sub(header.len());
        lines.push(format!("{}{}", header, "─".repeat(pad)));

        lines.push(format!(
            "│ Anvil Hardness: {:.2} | Damping: {:.2} | Blend: {:.2} | Phase: {:.1}° │",
            self.anvil_hardness, self.yarn_damping, self.pickup_blend, self.pickup_phase_deg
        ));

        lines.push(format!(
            "│ Tone: [Brilliant: {}] [Treble: {}] [Medium: {}] [Soft: {}] │",
            if self.brilliant_switch { "ON " } else { "OFF" },
            if self.treble_switch { "ON " } else { "OFF" },
            if self.medium_switch { "ON " } else { "OFF" },
            if self.soft_switch { "ON " } else { "OFF" }
        ));

        lines.push(format!(
            "│ Auto-Wah: {} | Sens: {:.2} | Freq: {:.0}Hz | Q: {:.1} | Mix: {:.0}% │",
            if self.auto_wah_enabled { "ON " } else { "OFF" },
            self.auto_wah_sensitivity,
            self.auto_wah_freq_hz,
            self.auto_wah_resonance_q,
            self.auto_wah_mix * 100.0
        ));

        let wave = self.evaluate_pickup_waveform();
        let mut wave_str = String::from("│ Wave: ");
        for val in wave.iter().take(24) {
            let ch = if *val > 0.5 { '▲' } else if *val > 0.05 { '─' } else if *val < -0.5 { '▼' } else { ' ' };
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

        // Draw 2D Anvil Hardness vs Phase Canvas Box
        let margin = 32;
        let c_w = (width / 2) - margin * 2;
        let c_h = height - margin * 2;
        let c_x = margin;
        let c_y = margin;

        for y in c_y..(c_y + c_h) {
            for x in c_x..(c_x + c_w) {
                if x < width && y < height {
                    let idx = (y * width + x) * 4;
                    pixels[idx] = 0x18;
                    pixels[idx + 1] = 0x1C;
                    pixels[idx + 2] = 0x2A;
                    pixels[idx + 3] = 0xFF;
                }
            }
        }

        // Draw Anvil Puck Handle (Radius = 22pt)
        let puck_cx = (c_x as f32 + self.puck_pos.0 * c_w as f32) as usize;
        let puck_cy = (c_y as f32 + (1.0 - self.puck_pos.1) * c_h as f32) as usize;
        let r = CLAV_PUCK_HIT_RADIUS as usize;

        for dy in 0..=(2 * r) {
            for dx in 0..=(2 * r) {
                let px = (puck_cx + dx).saturating_sub(r);
                let py = (puck_cy + dy).saturating_sub(r);
                let dist_sq = (dx as isize - r as isize).pow(2) + (dy as isize - r as isize).pow(2);
                if dist_sq <= (r * r) as isize && px < width && py < height {
                    let idx = (py * width + px) * 4;
                    // Tangerine Orange #F77F00
                    pixels[idx] = 0xF7;
                    pixels[idx + 1] = 0x7F;
                    pixels[idx + 2] = 0x00;
                    pixels[idx + 3] = 0xFF;
                }
            }
        }

        // Draw Pickup Waveform in Right Half
        let wave = self.evaluate_pickup_waveform();
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
                    // Electric Amber #FFD166
                    pixels[idx] = 0xFF;
                    pixels[idx + 1] = 0xD1;
                    pixels[idx + 2] = 0x66;
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
    fn test_clavinet_view_hit_target_dimensions() {
        const {
            assert!(
                CLAV_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT,
                "Clavinet puck hit target bounding box must be >= 44pt"
            );
        }
    }

    #[test]
    fn test_clavinet_view_ascii_render() {
        let view = ClavinetView::new();
        let ascii = view.render_ascii(80, 16);
        assert_eq!(ascii.len(), 16);
        assert!(ascii[0].contains("CLAVINET HUD"));
    }

    #[test]
    fn test_clavinet_view_snapshot_render() {
        let view = ClavinetView::new();
        let res = view.render_snapshot_png("scratch/renders/clavinet_view.png", 800, 520);
        assert!(res.is_ok());
    }
}
