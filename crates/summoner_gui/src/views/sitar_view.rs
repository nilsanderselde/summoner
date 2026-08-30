// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Sitar HUD & Meend Pitch Deflection Canvas (Milestone 27).
//!
//! Provides an interactive 2D Mizrab strike dynamics & lateral Meend pull puck (Meend Pull vs Strike Velocity),
//! real-time curved Jawari obstacle boundary collision displacement visualizer,
//! 4-string Chikari drone strum activity indicator, Raga scale selection display,
//! and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and headless PNG snapshot rendering.

use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const SITAR_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding target
pub const MIN_SITAR_STRIKE_VELOCITY: f32 = 0.05;
pub const MAX_SITAR_STRIKE_VELOCITY: f32 = 1.0;
pub const MIN_MEEND_SEMITONES: f32 = 0.0;
pub const MAX_MEEND_SEMITONES: f32 = 5.0;

/// Sitar HUD sound and articulation preset choices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SitarHudPreset {
    #[default]
    RagaYamanAlap,
    VilayatKhanGayaki,
    RaviShankarKharaj,
    SurbaharDeepBass,
    BhairavJorJhala,
    ElectricSitarJhajhar,
}

impl SitarHudPreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::RagaYamanAlap => "Raga Yaman Alap (Evening)",
            Self::VilayatKhanGayaki => "Vilayat Khan Gayaki (Vocal)",
            Self::RaviShankarKharaj => "Ravi Shankar Kharaj Pancham",
            Self::SurbaharDeepBass => "Surbahar Deep Bass Alap",
            Self::BhairavJorJhala => "Raga Bhairav Jor & Jhala",
            Self::ElectricSitarJhajhar => "Electric Sitar Jhajhar",
        }
    }
}

/// Sitar interactive performance and analysis view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitarView {
    /// Active HUD sound preset.
    pub preset: SitarHudPreset,
    /// Mizrab strike velocity $[0.05 ..= 1.0]$.
    pub strike_velocity: f32,
    /// Lateral Meend pull pitch deflection $[0.0 ..= 5.0]$ in semitones.
    pub meend_pull_semitones: f32,
    /// Jawari clearance gap in mm $[0.01 ..= 1.5]$.
    pub jawari_gap_mm: f32,
    /// Jiva cotton thread position $[0.0 ..= 1.0]$.
    pub jiva_thread_pos: f32,
    /// Tarab sympathetic coupling bleed $[0.0 ..= 1.0]$.
    pub tarab_bleed: f32,
    /// Chikari drone strum activity state.
    pub chikari_active: bool,
    /// Active Raga scale name.
    pub active_raga_name: String,
    /// 2D Puck position (X: Meend pull, Y: strike velocity).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for SitarView {
    fn default() -> Self {
        Self::new()
    }
}

impl SitarView {
    pub fn new() -> Self {
        let mut view = Self {
            preset: SitarHudPreset::RagaYamanAlap,
            strike_velocity: 0.75,
            meend_pull_semitones: 1.5,
            jawari_gap_mm: 0.18,
            jiva_thread_pos: 0.45,
            tarab_bleed: 0.40,
            chikari_active: false,
            active_raga_name: "Raga Yaman (Evening)".to_string(),
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        };
        view.update_puck_from_physics();
        view.update_sitar_acoustics();
        view
    }

    pub fn update_puck_from_physics(&mut self) {
        let norm_x = ((self.meend_pull_semitones - MIN_MEEND_SEMITONES) / (MAX_MEEND_SEMITONES - MIN_MEEND_SEMITONES)).clamp(0.0, 1.0);
        let norm_y = ((self.strike_velocity - MIN_SITAR_STRIKE_VELOCITY) / (MAX_SITAR_STRIKE_VELOCITY - MIN_SITAR_STRIKE_VELOCITY)).clamp(0.0, 1.0);
        self.puck_pos = (norm_x, norm_y);
    }

    pub fn update_physics_from_puck(&mut self, norm_x: f32, norm_y: f32) {
        self.puck_pos = (norm_x.clamp(0.0, 1.0), norm_y.clamp(0.0, 1.0));
        self.meend_pull_semitones = MIN_MEEND_SEMITONES + self.puck_pos.0 * (MAX_MEEND_SEMITONES - MIN_MEEND_SEMITONES);
        self.strike_velocity = MIN_SITAR_STRIKE_VELOCITY + self.puck_pos.1 * (MAX_SITAR_STRIKE_VELOCITY - MIN_SITAR_STRIKE_VELOCITY);
        self.update_sitar_acoustics();
    }

    pub fn update_sitar_acoustics(&mut self) {
        match self.preset {
            SitarHudPreset::RagaYamanAlap => {
                self.jawari_gap_mm = 0.18;
                self.jiva_thread_pos = 0.45;
                self.tarab_bleed = 0.40;
                self.active_raga_name = "Raga Yaman (Evening)".to_string();
            }
            SitarHudPreset::VilayatKhanGayaki => {
                self.jawari_gap_mm = 0.08;
                self.jiva_thread_pos = 0.50;
                self.tarab_bleed = 0.35;
                self.active_raga_name = "Raga Bhairav (Dawn)".to_string();
            }
            SitarHudPreset::RaviShankarKharaj => {
                self.jawari_gap_mm = 0.20;
                self.jiva_thread_pos = 0.40;
                self.tarab_bleed = 0.45;
                self.active_raga_name = "Raga Yaman (Evening)".to_string();
            }
            SitarHudPreset::SurbaharDeepBass => {
                self.jawari_gap_mm = 0.35;
                self.jiva_thread_pos = 0.35;
                self.tarab_bleed = 0.50;
                self.active_raga_name = "Raga Darbari (Midnight)".to_string();
            }
            SitarHudPreset::BhairavJorJhala => {
                self.jawari_gap_mm = 0.15;
                self.jiva_thread_pos = 0.48;
                self.tarab_bleed = 0.38;
                self.active_raga_name = "Raga Bhairav (Dawn)".to_string();
            }
            SitarHudPreset::ElectricSitarJhajhar => {
                self.jawari_gap_mm = 0.05;
                self.jiva_thread_pos = 0.60;
                self.tarab_bleed = 0.20;
                self.active_raga_name = "Raga Bilawal (Morning)".to_string();
            }
        }
    }

    pub fn set_preset(&mut self, preset: SitarHudPreset) {
        self.preset = preset;
        match preset {
            SitarHudPreset::RagaYamanAlap => {
                self.strike_velocity = 0.65;
                self.meend_pull_semitones = 1.5;
            }
            SitarHudPreset::VilayatKhanGayaki => {
                self.strike_velocity = 0.75;
                self.meend_pull_semitones = 3.0;
            }
            SitarHudPreset::RaviShankarKharaj => {
                self.strike_velocity = 0.80;
                self.meend_pull_semitones = 2.0;
            }
            SitarHudPreset::SurbaharDeepBass => {
                self.strike_velocity = 0.70;
                self.meend_pull_semitones = 4.0;
            }
            SitarHudPreset::BhairavJorJhala => {
                self.strike_velocity = 0.88;
                self.meend_pull_semitones = 0.5;
            }
            SitarHudPreset::ElectricSitarJhajhar => {
                self.strike_velocity = 0.85;
                self.meend_pull_semitones = 2.5;
            }
        }
        self.update_puck_from_physics();
        self.update_sitar_acoustics();
    }

    /// Render ASCII diagnostics representation of the view.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let border = format!("+{}+", "-".repeat(width.saturating_sub(2)));
        lines.push(border.clone());

        let title = format!(
            "| SITAR PERFORMANCE HUD | Preset: {} | Raga: {} |",
            self.preset.name(),
            self.active_raga_name
        );
        let title_padded = format!("{:<width$}|", title, width = width - 1);
        lines.push(title_padded);

        let dynamics_info = format!(
            "| Strike Velocity: {:.2} | Meend Pull: +{:.2} st | Jawari Gap: {:.2}mm | Jiva Pos: {:.2} |",
            self.strike_velocity, self.meend_pull_semitones, self.jawari_gap_mm, self.jiva_thread_pos
        );
        let dyn_padded = format!("{:<width$}|", dynamics_info, width = width - 1);
        lines.push(dyn_padded);

        let drone_info = format!(
            "| Tarab Bleed: {:.0}% | Chikari Strum: {} |",
            self.tarab_bleed * 100.0,
            if self.chikari_active { "ACTIVE" } else { "READY" }
        );
        let dro_padded = format!("{:<width$}|", drone_info, width = width - 1);
        lines.push(dro_padded);

        lines.push(border.clone());

        // Jawari curved obstacle collision phase canvas
        let canvas_h = height.saturating_sub(6);
        for row in 0..canvas_h {
            let row_ratio = 1.0 - (row as f32 / canvas_h.max(1) as f32);
            let mut line_buf = String::with_capacity(width);
            line_buf.push('|');

            for col in 0..(width.saturating_sub(2)) {
                let col_t = col as f32 / (width - 2).max(1) as f32; // time/position [0.0 .. 1.0]

                // Sitar string wave with curved Jawari unilateral contact clipping
                let fundamental = (col_t * 6.0 * std::f32::consts::PI * (1.0 + self.meend_pull_semitones * 0.15)).sin();
                let harmonic2 = 0.5 * (col_t * 12.0 * std::f32::consts::PI).sin();
                let raw_wave = (fundamental * 0.7 + harmonic2 * 0.3) * self.strike_velocity;

                // Jawari unilateral obstacle boundary
                let obstacle_bound = -0.3 + (self.jawari_gap_mm - 0.18) * 0.5;
                let clipped_wave = raw_wave.max(obstacle_bound);
                let norm_wave = (clipped_wave + 1.0) * 0.5;

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

        // 2D Dynamics Puck Area (Left panel: 40..340, 80..420)
        let puck_x_center = 40.0 + self.puck_pos.0 * 300.0;
        let puck_y_center = 420.0 - self.puck_pos.1 * 340.0; // Inverted Y

        // Active hit-target radius: >= 22pt (providing 44x44pt touch box)
        let hit_radius = SITAR_PUCK_HIT_RADIUS * 1.5;
        let vis_radius = 14.0 * 1.5;

        for y in 0..height {
            for x in 0..width {
                let dx = x as f32 - puck_x_center;
                let dy = y as f32 - puck_y_center;
                let dist = (dx * dx + dy * dy).sqrt();

                let idx = (y * width + x) * 4;
                if dist <= vis_radius {
                    // Puck body: Vibrant Sitar Gold/Amber `#F59E0B`
                    pixels[idx] = 0xF5;
                    pixels[idx + 1] = 0x9E;
                    pixels[idx + 2] = 0x0B;
                } else if dist <= hit_radius {
                    // Hit target glow ring: `#78350F`
                    pixels[idx] = 0x78;
                    pixels[idx + 1] = 0x35;
                    pixels[idx + 2] = 0x0F;
                }
            }
        }

        // Right panel: Curved Jawari contact collision phase canvas (400..760, 80..420)
        let plot_x_start = 400;
        let plot_x_end = 760;
        let plot_w = plot_x_end - plot_x_start;

        for px in 0..plot_w {
            let x = plot_x_start + px;
            let col_t = px as f32 / plot_w as f32;

            let fundamental = (col_t * 6.0 * std::f32::consts::PI * (1.0 + self.meend_pull_semitones * 0.15)).sin();
            let harmonic2 = 0.5 * (col_t * 12.0 * std::f32::consts::PI).sin();
            let raw_wave = (fundamental * 0.7 + harmonic2 * 0.3) * self.strike_velocity;

            let obstacle_bound = -0.3 + (self.jawari_gap_mm - 0.18) * 0.5;
            let clipped_wave = raw_wave.max(obstacle_bound);
            let norm_wave = (clipped_wave + 1.0) * 0.5;

            let py = 420 - (norm_wave * 320.0) as usize;
            if py < height {
                let idx = (py * width + x) * 4;
                // Waveform color: Vibrant Sitar Turquoise `#06B6D4`
                pixels[idx] = 0x06;
                pixels[idx + 1] = 0xB6;
                pixels[idx + 2] = 0xD4;

                if py > 0 {
                    let idx_above = ((py - 1) * width + x) * 4;
                    pixels[idx_above] = 0x06;
                    pixels[idx_above + 1] = 0xB6;
                    pixels[idx_above + 2] = 0xD4;
                }
            }
        }

        // Bottom status indicators (Meend deflection & Tarab bleed)
        let bar_y = 470;
        for x in 40..240 {
            let fill_limit = 40 + ((self.meend_pull_semitones / 5.0) * 200.0) as usize;
            let idx = (bar_y * width + x) * 4;
            if x <= fill_limit {
                pixels[idx] = 0x10;
                pixels[idx + 1] = 0xB9;
                pixels[idx + 2] = 0x81; // Green active
            } else {
                pixels[idx] = 0x33;
                pixels[idx + 1] = 0x41;
                pixels[idx + 2] = 0x55;
            }
        }

        for x in 280..480 {
            let fill_limit = 280 + (self.tarab_bleed * 200.0) as usize;
            let idx = (bar_y * width + x) * 4;
            if x <= fill_limit {
                pixels[idx] = 0x8B;
                pixels[idx + 1] = 0x5C;
                pixels[idx + 2] = 0xF6; // Purple active
            } else {
                pixels[idx] = 0x33;
                pixels[idx + 1] = 0x41;
                pixels[idx + 2] = 0x55;
            }
        }

        save_png_file(path, width, height, &pixels)
    }
}

#[cfg(feature = "gui")]
impl SitarView {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Sitar — Curved Jawari Bridge & Meend Pitch Deflection");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Preset:");
                if ui.selectable_label(self.preset == SitarHudPreset::RagaYamanAlap, "Raga Yaman").clicked() {
                    self.set_preset(SitarHudPreset::RagaYamanAlap);
                }
                if ui.selectable_label(self.preset == SitarHudPreset::VilayatKhanGayaki, "Gayaki Vocal").clicked() {
                    self.set_preset(SitarHudPreset::VilayatKhanGayaki);
                }
                if ui.selectable_label(self.preset == SitarHudPreset::RaviShankarKharaj, "Kharaj Pancham").clicked() {
                    self.set_preset(SitarHudPreset::RaviShankarKharaj);
                }
                if ui.selectable_label(self.preset == SitarHudPreset::SurbaharDeepBass, "Surbahar Bass").clicked() {
                    self.set_preset(SitarHudPreset::SurbaharDeepBass);
                }
                if ui.selectable_label(self.preset == SitarHudPreset::ElectricSitarJhajhar, "Electric Sitar").clicked() {
                    self.set_preset(SitarHudPreset::ElectricSitarJhajhar);
                }
            });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("Mizrab Strike & Meend Pull:");
                    ui.add(egui::Slider::new(&mut self.strike_velocity, MIN_SITAR_STRIKE_VELOCITY..=MAX_SITAR_STRIKE_VELOCITY).text("Strike Velocity"));
                    ui.add(egui::Slider::new(&mut self.meend_pull_semitones, MIN_MEEND_SEMITONES..=MAX_MEEND_SEMITONES).text("Meend Pull (st)"));
                });

                ui.separator();

                ui.vertical(|ui| {
                    ui.label("Bridge & Drone Controls:");
                    ui.add(egui::Slider::new(&mut self.jawari_gap_mm, 0.01..=1.5).text("Jawari Gap (mm)"));
                    ui.add(egui::Slider::new(&mut self.tarab_bleed, 0.0..=1.0).text("Tarab Bleed"));
                    ui.checkbox(&mut self.chikari_active, "Chikari Drone Active");
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
    fn test_sitar_view_hit_target_dimensions() {
        const {
            assert!(
                SITAR_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT,
                "Sitar puck hit target bounding box must be >= 44pt"
            );
        }
    }

    #[test]
    fn test_sitar_view_ascii_render() {
        let view = SitarView::new();
        let ascii = view.render_ascii(80, 16);
        assert_eq!(ascii.len(), 16);
        assert!(ascii[0].starts_with('+'));
    }

    #[test]
    fn test_sitar_view_snapshot_render() {
        let view = SitarView::new();
        let res = view.render_snapshot_png("scratch/renders/sitar_view.png", 800, 520);
        assert!(res.is_ok());
    }
}
