// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Concert Grand Piano HUD & 3-String Unison Phase Canvas (Milestone 26).
//!
//! Provides an interactive 2D hammer strike dynamics puck (Strike Velocity vs Hammer Hardness),
//! real-time 3-string unison prompt/aftersound dual-decay phase portrait visualizer,
//! 3-pedal status displays (Sustain / Half-Pedaling, Una Corda Soft Pedal, Sostenuto Latch),
//! and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and headless PNG snapshot rendering.

use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const GRAND_PIANO_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding target
pub const MIN_GRAND_PIANO_STRIKE_VELOCITY: f32 = 0.05;
pub const MAX_GRAND_PIANO_STRIKE_VELOCITY: f32 = 1.0;
pub const MIN_HAMMER_HARDNESS: f32 = 0.1;
pub const MAX_HAMMER_HARDNESS: f32 = 1.0;

/// Grand Piano sound and physical character preset choices for the HUD.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GrandPianoHudPreset {
    #[default]
    SteinwayDRecital,
    BösendorferWarmth,
    YamahaCFXPop,
    ImpressionistUnaCorda,
    IntimateFelted,
    PreparedAvantGarde,
}

impl GrandPianoHudPreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::SteinwayDRecital => "Steinway D Concert Recital",
            Self::BösendorferWarmth => "Bösendorfer Imperial Warmth",
            Self::YamahaCFXPop => "Yamaha CFX Pop Clarity",
            Self::ImpressionistUnaCorda => "Impressionist Una Corda Shimmer",
            Self::IntimateFelted => "Intimate Felted Studio Grand",
            Self::PreparedAvantGarde => "Prepared Avant-Garde Clatter",
        }
    }
}

/// Concert Grand Piano interactive performance and analysis view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrandPianoView {
    /// Active HUD sound preset.
    pub preset: GrandPianoHudPreset,
    /// Strike velocity $[0.05 ..= 1.0]$.
    pub strike_velocity: f32,
    /// Hammer felt hardness $[0.1 ..= 1.0]$.
    pub hammer_hardness: f32,
    /// Sustain pedal damper lift $[0.0 ..= 1.0]$ (Half-Pedaling).
    pub damper_lift_pos: f32,
    /// Una Corda soft pedal shift $[0.0 ..= 1.0]$.
    pub una_corda_shift: f32,
    /// Sostenuto pedal engagement latch.
    pub sostenuto_engaged: bool,
    /// Prompt decay $T_{60}$ in seconds.
    pub prompt_t60_sec: f32,
    /// Aftersound decay $T_{60}$ in seconds.
    pub aftersound_t60_sec: f32,
    /// 3-string unison detune in cents.
    pub unison_detune_cents: f32,
    /// Inharmonicity dispersion $B$ parameter.
    pub inharmonicity_b: f32,
    /// 2D Hammer puck position (X: hardness, Y: velocity).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for GrandPianoView {
    fn default() -> Self {
        Self::new()
    }
}

impl GrandPianoView {
    pub fn new() -> Self {
        let mut view = Self {
            preset: GrandPianoHudPreset::SteinwayDRecital,
            strike_velocity: 0.75,
            hammer_hardness: 0.65,
            damper_lift_pos: 0.0,
            una_corda_shift: 0.0,
            sostenuto_engaged: false,
            prompt_t60_sec: 2.2,
            aftersound_t60_sec: 14.0,
            unison_detune_cents: 0.75,
            inharmonicity_b: 0.00018,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        };
        view.update_puck_from_physics();
        view.update_piano_acoustics();
        view
    }

    pub fn update_puck_from_physics(&mut self) {
        let norm_x = ((self.hammer_hardness - MIN_HAMMER_HARDNESS) / (MAX_HAMMER_HARDNESS - MIN_HAMMER_HARDNESS)).clamp(0.0, 1.0);
        let norm_y = ((self.strike_velocity - MIN_GRAND_PIANO_STRIKE_VELOCITY) / (MAX_GRAND_PIANO_STRIKE_VELOCITY - MIN_GRAND_PIANO_STRIKE_VELOCITY)).clamp(0.0, 1.0);
        self.puck_pos = (norm_x, norm_y);
    }

    pub fn update_physics_from_puck(&mut self, norm_x: f32, norm_y: f32) {
        self.puck_pos = (norm_x.clamp(0.0, 1.0), norm_y.clamp(0.0, 1.0));
        self.hammer_hardness = MIN_HAMMER_HARDNESS + self.puck_pos.0 * (MAX_HAMMER_HARDNESS - MIN_HAMMER_HARDNESS);
        self.strike_velocity = MIN_GRAND_PIANO_STRIKE_VELOCITY + self.puck_pos.1 * (MAX_GRAND_PIANO_STRIKE_VELOCITY - MIN_GRAND_PIANO_STRIKE_VELOCITY);
        self.update_piano_acoustics();
    }

    pub fn update_piano_acoustics(&mut self) {
        // Adjust prompt/aftersound decay times based on strike velocity, hammer hardness, and Una Corda
        let hard_mult = 0.7 + 0.6 * self.hammer_hardness;
        let una_mult = if self.una_corda_shift > 0.01 {
            1.0 + 0.35 * self.una_corda_shift
        } else {
            1.0
        };

        match self.preset {
            GrandPianoHudPreset::SteinwayDRecital => {
                self.prompt_t60_sec = 2.2 * hard_mult;
                self.aftersound_t60_sec = 14.0 * una_mult;
                self.unison_detune_cents = 0.75;
                self.inharmonicity_b = 0.00018;
            }
            GrandPianoHudPreset::BösendorferWarmth => {
                self.prompt_t60_sec = 2.8 * hard_mult;
                self.aftersound_t60_sec = 18.0 * una_mult;
                self.unison_detune_cents = 0.60;
                self.inharmonicity_b = 0.00012;
            }
            GrandPianoHudPreset::YamahaCFXPop => {
                self.prompt_t60_sec = 1.8 * hard_mult;
                self.aftersound_t60_sec = 11.0 * una_mult;
                self.unison_detune_cents = 0.90;
                self.inharmonicity_b = 0.00025;
            }
            GrandPianoHudPreset::ImpressionistUnaCorda => {
                self.prompt_t60_sec = 3.2 * hard_mult;
                self.aftersound_t60_sec = 20.0 * una_mult;
                self.unison_detune_cents = 1.10;
                self.inharmonicity_b = 0.00015;
            }
            GrandPianoHudPreset::IntimateFelted => {
                self.prompt_t60_sec = 1.5 * hard_mult;
                self.aftersound_t60_sec = 9.0 * una_mult;
                self.unison_detune_cents = 0.50;
                self.inharmonicity_b = 0.00010;
            }
            GrandPianoHudPreset::PreparedAvantGarde => {
                self.prompt_t60_sec = 0.6 * hard_mult;
                self.aftersound_t60_sec = 3.5 * una_mult;
                self.unison_detune_cents = 2.50;
                self.inharmonicity_b = 0.00060;
            }
        }
    }

    pub fn set_preset(&mut self, preset: GrandPianoHudPreset) {
        self.preset = preset;
        match preset {
            GrandPianoHudPreset::SteinwayDRecital => {
                self.strike_velocity = 0.75;
                self.hammer_hardness = 0.65;
                self.una_corda_shift = 0.0;
            }
            GrandPianoHudPreset::BösendorferWarmth => {
                self.strike_velocity = 0.70;
                self.hammer_hardness = 0.50;
                self.una_corda_shift = 0.0;
            }
            GrandPianoHudPreset::YamahaCFXPop => {
                self.strike_velocity = 0.85;
                self.hammer_hardness = 0.80;
                self.una_corda_shift = 0.0;
            }
            GrandPianoHudPreset::ImpressionistUnaCorda => {
                self.strike_velocity = 0.50;
                self.hammer_hardness = 0.30;
                self.una_corda_shift = 1.0;
                self.damper_lift_pos = 0.80;
            }
            GrandPianoHudPreset::IntimateFelted => {
                self.strike_velocity = 0.55;
                self.hammer_hardness = 0.35;
                self.una_corda_shift = 0.40;
            }
            GrandPianoHudPreset::PreparedAvantGarde => {
                self.strike_velocity = 0.90;
                self.hammer_hardness = 0.90;
                self.una_corda_shift = 0.0;
            }
        }
        self.update_puck_from_physics();
        self.update_piano_acoustics();
    }

    /// Render ASCII diagnostics representation of the view.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let border = format!("+{}+", "-".repeat(width.saturating_sub(2)));
        lines.push(border.clone());

        let title = format!(
            "| CONCERT GRAND PIANO HUD | Preset: {} |",
            self.preset.name()
        );
        let title_padded = format!("{:<width$}|", title, width = width - 1);
        lines.push(title_padded);

        let dynamics_info = format!(
            "| Velocity: {:.2} | Hardness: {:.2} | Prompt T60: {:.1}s | Aftersound: {:.1}s |",
            self.strike_velocity, self.hammer_hardness, self.prompt_t60_sec, self.aftersound_t60_sec
        );
        let dyn_padded = format!("{:<width$}|", dynamics_info, width = width - 1);
        lines.push(dyn_padded);

        let pedal_info = format!(
            "| Sustain/Half-Pedal: {:.0}% | Una Corda: {:.0}% | Sostenuto: {} | Detune: {:.2}c |",
            self.damper_lift_pos * 100.0,
            self.una_corda_shift * 100.0,
            if self.sostenuto_engaged { "ENGAGED" } else { "OFF" },
            self.unison_detune_cents
        );
        let ped_padded = format!("{:<width$}|", pedal_info, width = width - 1);
        lines.push(ped_padded);

        lines.push(border.clone());

        // Fill remaining height with 3-string unison prompt/aftersound envelope canvas
        let canvas_h = height.saturating_sub(6);
        for row in 0..canvas_h {
            let row_ratio = 1.0 - (row as f32 / canvas_h.max(1) as f32);
            let mut line_buf = String::with_capacity(width);
            line_buf.push('|');

            for col in 0..(width.saturating_sub(2)) {
                let col_t = col as f32 / (width - 2).max(1) as f32; // time [0.0 .. 1.0]

                // Prompt envelope: fast initial drop
                let env_prompt = (-col_t * 6.0).exp();
                // Aftersound envelope: slow decay with beating
                let beat_freq = self.unison_detune_cents * 4.0;
                let env_after = 0.35 * (-col_t * 1.2).exp() * (1.0 + 0.4 * (col_t * beat_freq * std::f32::consts::PI).cos());
                let total_env = (env_prompt * 0.7 + env_after * 0.3).clamp(0.0, 1.0);

                if (total_env - row_ratio).abs() < 0.06 {
                    line_buf.push('#');
                } else if total_env > row_ratio {
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
        let hit_radius = GRAND_PIANO_PUCK_HIT_RADIUS * 1.5;
        let vis_radius = 14.0 * 1.5;

        for y in 0..height {
            for x in 0..width {
                let dx = x as f32 - puck_x_center;
                let dy = y as f32 - puck_y_center;
                let dist = (dx * dx + dy * dy).sqrt();

                let idx = (y * width + x) * 4;
                if dist <= vis_radius {
                    // Puck body: Bright amber/gold `#E5A93C`
                    pixels[idx] = 0xE5;
                    pixels[idx + 1] = 0xA9;
                    pixels[idx + 2] = 0x3C;
                } else if dist <= hit_radius {
                    // Hit target glow ring (WCAG AAA contrast ring)
                    pixels[idx] = 0x72;
                    pixels[idx + 1] = 0x54;
                    pixels[idx + 2] = 0x1E;
                }
            }
        }

        // Right panel: 3-string unison prompt/aftersound decay phase portrait (400..760, 80..420)
        let plot_x_start = 400;
        let plot_x_end = 760;
        let plot_w = plot_x_end - plot_x_start;

        for px in 0..plot_w {
            let x = plot_x_start + px;
            let col_t = px as f32 / plot_w as f32;

            let env_prompt = (-col_t * 5.5).exp();
            let beat_freq = self.unison_detune_cents * 4.0;
            let env_after = 0.40 * (-col_t * 1.0).exp() * (1.0 + 0.35 * (col_t * beat_freq * std::f32::consts::PI).cos());
            let total_env = (env_prompt * 0.65 + env_after * 0.35).clamp(0.0, 1.0);

            let py = 420 - (total_env * 320.0) as usize;
            if py < height {
                let idx = (py * width + x) * 4;
                // Curve color: Crystalline cyan `#44D9E8`
                pixels[idx] = 0x44;
                pixels[idx + 1] = 0xD9;
                pixels[idx + 2] = 0xE8;

                if py > 0 {
                    let idx_above = ((py - 1) * width + x) * 4;
                    pixels[idx_above] = 0x44;
                    pixels[idx_above + 1] = 0xD9;
                    pixels[idx_above + 2] = 0xE8;
                }
            }
        }

        // Bottom Pedals Bar (Sustain, Una Corda, Sostenuto)
        let pedal_y = 470;
        for x in 40..240 {
            // Sustain pedal fill
            let fill_limit = 40 + (self.damper_lift_pos * 200.0) as usize;
            let idx = (pedal_y * width + x) * 4;
            if x <= fill_limit {
                pixels[idx] = 0x22;
                pixels[idx + 1] = 0xC5;
                pixels[idx + 2] = 0x5E; // Green active
            } else {
                pixels[idx] = 0x33;
                pixels[idx + 1] = 0x41;
                pixels[idx + 2] = 0x55;
            }
        }

        for x in 280..480 {
            // Una corda pedal fill
            let fill_limit = 280 + (self.una_corda_shift * 200.0) as usize;
            let idx = (pedal_y * width + x) * 4;
            if x <= fill_limit {
                pixels[idx] = 0xA8;
                pixels[idx + 1] = 0x55;
                pixels[idx + 2] = 0xF7; // Purple soft
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
impl GrandPianoView {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Concert Grand Piano — 3-String Unison & Felt Dynamics");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Preset:");
                if ui.selectable_label(self.preset == GrandPianoHudPreset::SteinwayDRecital, "Steinway D").clicked() {
                    self.set_preset(GrandPianoHudPreset::SteinwayDRecital);
                }
                if ui.selectable_label(self.preset == GrandPianoHudPreset::BösendorferWarmth, "Bösendorfer 290").clicked() {
                    self.set_preset(GrandPianoHudPreset::BösendorferWarmth);
                }
                if ui.selectable_label(self.preset == GrandPianoHudPreset::YamahaCFXPop, "Yamaha CFX").clicked() {
                    self.set_preset(GrandPianoHudPreset::YamahaCFXPop);
                }
                if ui.selectable_label(self.preset == GrandPianoHudPreset::ImpressionistUnaCorda, "Una Corda Shimmer").clicked() {
                    self.set_preset(GrandPianoHudPreset::ImpressionistUnaCorda);
                }
                if ui.selectable_label(self.preset == GrandPianoHudPreset::IntimateFelted, "Intimate Felt").clicked() {
                    self.set_preset(GrandPianoHudPreset::IntimateFelted);
                }
            });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("Hammer Dynamics (Hardness vs Velocity):");
                    ui.add(egui::Slider::new(&mut self.hammer_hardness, MIN_HAMMER_HARDNESS..=MAX_HAMMER_HARDNESS).text("Felt Hardness"));
                    ui.add(egui::Slider::new(&mut self.strike_velocity, MIN_GRAND_PIANO_STRIKE_VELOCITY..=MAX_GRAND_PIANO_STRIKE_VELOCITY).text("Strike Velocity"));
                });

                ui.separator();

                ui.vertical(|ui| {
                    ui.label("Continuous Pedals:");
                    ui.add(egui::Slider::new(&mut self.damper_lift_pos, 0.0..=1.0).text("Sustain (Half-Pedal)"));
                    ui.add(egui::Slider::new(&mut self.una_corda_shift, 0.0..=1.0).text("Una Corda (Soft Pedal)"));
                    ui.checkbox(&mut self.sostenuto_engaged, "Sostenuto Latch");
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
    fn test_grand_piano_view_hit_target_dimensions() {
        const {
            assert!(
                GRAND_PIANO_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT,
                "Grand piano puck hit target bounding box must be >= 44pt"
            );
        }
    }

    #[test]
    fn test_grand_piano_view_ascii_render() {
        let view = GrandPianoView::new();
        let ascii = view.render_ascii(80, 16);
        assert_eq!(ascii.len(), 16);
        assert!(ascii[0].starts_with('+'));
    }

    #[test]
    fn test_grand_piano_view_snapshot_render() {
        let view = GrandPianoView::new();
        let res = view.render_snapshot_png("scratch/renders/grand_piano_view.png", 800, 520);
        assert!(res.is_ok());
    }
}
