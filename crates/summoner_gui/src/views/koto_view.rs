// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Asian Zither Koto HUD & Oshi-Ite String Tension Canvas (Milestone 29).
//!
//! Provides an interactive 2D Tsume pluck dynamics & Oshi-Ite left-hand press force puck
//! (Oshi-Ite Force [0..30 N] vs Tsume Velocity), real-time 13-string standing wave visualizer,
//! microtonal pitch bend deflection display, traditional scale selector, and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and headless PNG snapshot rendering.

use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const KOTO_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding target
pub const MIN_KOTO_TSUME_VELOCITY: f32 = 0.05;
pub const MAX_KOTO_TSUME_VELOCITY: f32 = 1.0;
pub const MIN_OSHI_ITE_FORCE_N: f32 = 0.0;
pub const MAX_OSHI_ITE_FORCE_N: f32 = 30.0;

/// Koto HUD sound and articulation preset choices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum KotoHudPreset {
    #[default]
    HirajoshiTraditionalSilk,
    KokinJoshiUrbanClassical,
    InSenContemporaryDramatic,
    KumoiJoshiSpringRain,
    RyukyuFestiveOkinawa,
    ConcertGuzhengVirtuoso,
}

impl KotoHudPreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::HirajoshiTraditionalSilk => "Hirajoshi Traditional Silk Koto",
            Self::KokinJoshiUrbanClassical => "Kokin-joshi Urban Classical Koto",
            Self::InSenContemporaryDramatic => "In-sen Contemporary Dramatic",
            Self::KumoiJoshiSpringRain => "Kumoi-joshi Spring Rain (Lyrical)",
            Self::RyukyuFestiveOkinawa => "Ryukyu Festive Okinawan Koto",
            Self::ConcertGuzhengVirtuoso => "Concert Guzheng Virtuoso (21-Str)",
        }
    }
}

/// Koto interactive performance and analysis view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KotoView {
    /// Active HUD sound preset.
    pub preset: KotoHudPreset,
    /// Tsume pluck strike velocity $[0.05 ..= 1.0]$.
    pub tsume_velocity: f32,
    /// Left-hand Oshi-Ite press force in Newtons $[0.0 ..= 30.0]$ (yielding up to +400 cents).
    pub oshi_ite_force_n: f32,
    /// Left-hand Hiki-Iro release fraction $[0.0 ..= 1.0]$.
    pub hiki_iro_release: f32,
    /// Movable Ji bridge position offset $[-0.20 ..= +0.20]$.
    pub ji_bridge_offset: f32,
    /// Active tuning schema name.
    pub active_tuning_schema_name: String,
    /// Behind-the-bridge sympathetic bleed fraction $[0.0 ..= 1.0]$.
    pub behind_bridge_bleed: f32,
    /// Paulownia soundboard body wood resonance $[0.0 ..= 1.0]$.
    pub body_wood_resonance: f32,
    /// 2D Puck position (X: Oshi-Ite force, Y: Tsume velocity).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for KotoView {
    fn default() -> Self {
        Self::new()
    }
}

impl KotoView {
    pub fn new() -> Self {
        let mut view = Self {
            preset: KotoHudPreset::HirajoshiTraditionalSilk,
            tsume_velocity: 0.75,
            oshi_ite_force_n: 2.0,
            hiki_iro_release: 0.10,
            ji_bridge_offset: 0.0,
            active_tuning_schema_name: "Hirajoshi (Classic Meditative)".to_string(),
            behind_bridge_bleed: 0.25,
            body_wood_resonance: 0.85,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        };
        view.update_puck_from_physics();
        view.update_koto_acoustics();
        view
    }

    pub fn update_puck_from_physics(&mut self) {
        let norm_x = ((self.oshi_ite_force_n - MIN_OSHI_ITE_FORCE_N) / (MAX_OSHI_ITE_FORCE_N - MIN_OSHI_ITE_FORCE_N)).clamp(0.0, 1.0);
        let norm_y = ((self.tsume_velocity - MIN_KOTO_TSUME_VELOCITY) / (MAX_KOTO_TSUME_VELOCITY - MIN_KOTO_TSUME_VELOCITY)).clamp(0.0, 1.0);
        self.puck_pos = (norm_x, norm_y);
    }

    pub fn update_physics_from_puck(&mut self, norm_x: f32, norm_y: f32) {
        self.puck_pos = (norm_x.clamp(0.0, 1.0), norm_y.clamp(0.0, 1.0));
        self.oshi_ite_force_n = MIN_OSHI_ITE_FORCE_N + self.puck_pos.0 * (MAX_OSHI_ITE_FORCE_N - MIN_OSHI_ITE_FORCE_N);
        self.tsume_velocity = MIN_KOTO_TSUME_VELOCITY + self.puck_pos.1 * (MAX_KOTO_TSUME_VELOCITY - MIN_KOTO_TSUME_VELOCITY);
        self.update_koto_acoustics();
    }

    pub fn update_koto_acoustics(&mut self) {
        match self.preset {
            KotoHudPreset::HirajoshiTraditionalSilk => {
                self.behind_bridge_bleed = 0.25;
                self.body_wood_resonance = 0.85;
                self.active_tuning_schema_name = "Hirajoshi (Classic Meditative)".to_string();
            }
            KotoHudPreset::KokinJoshiUrbanClassical => {
                self.behind_bridge_bleed = 0.20;
                self.body_wood_resonance = 0.80;
                self.active_tuning_schema_name = "Kokin-joshi (Miyako-bushi Urban)".to_string();
            }
            KotoHudPreset::InSenContemporaryDramatic => {
                self.behind_bridge_bleed = 0.30;
                self.body_wood_resonance = 0.90;
                self.active_tuning_schema_name = "In-sen (Traditional Dramatic)".to_string();
            }
            KotoHudPreset::KumoiJoshiSpringRain => {
                self.behind_bridge_bleed = 0.22;
                self.body_wood_resonance = 0.75;
                self.active_tuning_schema_name = "Kumoi-joshi (Lyrical Spring)".to_string();
            }
            KotoHudPreset::RyukyuFestiveOkinawa => {
                self.behind_bridge_bleed = 0.20;
                self.body_wood_resonance = 0.82;
                self.active_tuning_schema_name = "Ryukyu (Okinawan Festive)".to_string();
            }
            KotoHudPreset::ConcertGuzhengVirtuoso => {
                self.behind_bridge_bleed = 0.40;
                self.body_wood_resonance = 0.95;
                self.active_tuning_schema_name = "Guzheng (Major Pentatonic)".to_string();
            }
        }
    }

    pub fn set_preset(&mut self, preset: KotoHudPreset) {
        self.preset = preset;
        match preset {
            KotoHudPreset::HirajoshiTraditionalSilk => {
                self.tsume_velocity = 0.70;
                self.oshi_ite_force_n = 2.0;
            }
            KotoHudPreset::KokinJoshiUrbanClassical => {
                self.tsume_velocity = 0.90;
                self.oshi_ite_force_n = 4.0;
            }
            KotoHudPreset::InSenContemporaryDramatic => {
                self.tsume_velocity = 0.85;
                self.oshi_ite_force_n = 18.0;
            }
            KotoHudPreset::KumoiJoshiSpringRain => {
                self.tsume_velocity = 0.65;
                self.oshi_ite_force_n = 3.0;
            }
            KotoHudPreset::RyukyuFestiveOkinawa => {
                self.tsume_velocity = 0.80;
                self.oshi_ite_force_n = 5.0;
            }
            KotoHudPreset::ConcertGuzhengVirtuoso => {
                self.tsume_velocity = 0.92;
                self.oshi_ite_force_n = 6.0;
            }
        }
        self.update_puck_from_physics();
        self.update_koto_acoustics();
    }

    /// Calculate instantaneous cents deflection from Oshi-Ite press.
    pub fn current_pitch_cents(&self) -> f32 {
        let norm = (self.oshi_ite_force_n / 30.0).clamp(0.0, 1.0);
        400.0 * (1.0 - (-1.8 * norm).exp()) / (1.0 - (-1.8f32).exp())
    }

    /// Render ASCII diagnostics representation of the view.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let border = format!("+{}+", "-".repeat(width.saturating_sub(2)));
        lines.push(border.clone());

        let title = format!(
            "| KOTO & GUZHENG HUD | Preset: {} | Scale: {} |",
            self.preset.name(),
            self.active_tuning_schema_name
        );
        let title_padded = format!("{:<width$}|", title, width = width - 1);
        lines.push(title_padded);

        let dynamics_info = format!(
            "| Tsume Velocity: {:.2} | Oshi-Ite: {:.1} N (+{:.0} cents) | Bleed: {:.0}% | Wood Res: {:.0}% |",
            self.tsume_velocity, self.oshi_ite_force_n, self.current_pitch_cents(),
            self.behind_bridge_bleed * 100.0, self.body_wood_resonance * 100.0
        );
        let dyn_padded = format!("{:<width$}|", dynamics_info, width = width - 1);
        lines.push(dyn_padded);

        lines.push(border.clone());

        // 13-string standing wave & tension deflection canvas
        let canvas_h = height.saturating_sub(5);
        for row in 0..canvas_h {
            let row_ratio = 1.0 - (row as f32 / canvas_h.max(1) as f32);
            let mut line_buf = String::with_capacity(width);
            line_buf.push('|');

            for col in 0..(width.saturating_sub(2)) {
                let col_t = col as f32 / (width - 2).max(1) as f32; // position along soundboard [0.0 .. 1.0]

                // Silk string standing wave with oshi-ite pitch modulation
                let pitch_factor = 1.0 + self.current_pitch_cents() / 1200.0;
                let wave1 = (col_t * 8.0 * std::f32::consts::PI * pitch_factor).sin() * (col_t * std::f32::consts::PI).sin();
                let wave2 = 0.3 * (col_t * 16.0 * std::f32::consts::PI * pitch_factor).sin();
                let total_wave = (wave1 + wave2) * self.tsume_velocity;
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

        // Background: Deep Navy (#0A0E18)
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

        // Title text indicator bar (Gold #F59E0B)
        for y in 12..20 {
            for x in 24..180 {
                let idx = (y * width + x) * 4;
                pixels[idx] = 0xF5;
                pixels[idx + 1] = 0x9E;
                pixels[idx + 2] = 0x0B;
            }
        }

        // Main 2D Oshi-Ite & Tsume Puck Canvas Box
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
                    pixels[idx + 2] = 0x3B; // 8pt / 40px grid guide
                } else {
                    pixels[idx] = 0x0F;
                    pixels[idx + 1] = 0x17;
                    pixels[idx + 2] = 0x2A;
                }
            }
        }

        // Render 13 Koto strings standing wave curves across canvas
        let canvas_w = (c_right - c_left) as f32;
        let canvas_h_f = (c_bottom - c_top) as f32;
        let pitch_factor = 1.0 + self.current_pitch_cents() / 1200.0;

        for str_idx in 0..13 {
            let str_norm = str_idx as f32 / 12.0;
            let center_y = c_top as f32 + canvas_h_f * (0.15 + 0.70 * str_norm);

            for x_px in c_left..c_right {
                let x_norm = (x_px - c_left) as f32 / canvas_w;
                let wave = (x_norm * (4.0 + str_idx as f32) * std::f32::consts::PI * pitch_factor).sin()
                    * (x_norm * std::f32::consts::PI).sin()
                    * 14.0
                    * self.tsume_velocity;
                let y_px = (center_y + wave).clamp(c_top as f32 + 2.0, c_bottom as f32 - 2.0) as usize;

                let idx = (y_px * width + x_px) * 4;
                if str_idx % 2 == 0 {
                    pixels[idx] = 0x38;
                    pixels[idx + 1] = 0xBD;
                    pixels[idx + 2] = 0xF8; // Cyan silk string
                } else {
                    pixels[idx] = 0x10;
                    pixels[idx + 1] = 0xB9;
                    pixels[idx + 2] = 0x81; // Emerald string
                }
            }
        }

        // Interactive 2D Puck (Radius = 22pt -> 44x44pt touch target)
        let puck_center_x = c_left + ((c_right - c_left) as f32 * self.puck_pos.0) as usize;
        let puck_center_y = c_top + ((c_bottom - c_top) as f32 * (1.0 - self.puck_pos.1)) as usize;
        let radius = KOTO_PUCK_HIT_RADIUS as isize;
        let inner_radius = (KOTO_PUCK_HIT_RADIUS - 3.0) as isize;

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

        // Bottom Oshi-Ite Pitch Bend Gauge Bar (0 to +400 cents)
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

        // Fill pitch bend gauge
        let cents_norm = (self.current_pitch_cents() / 400.0).clamp(0.0, 1.0);
        let fill_x = bar_left + ((bar_right - bar_left) as f32 * cents_norm) as usize;

        for y in bar_y_start..bar_y_end {
            for x in bar_left..=fill_x.min(bar_right) {
                let idx = (y * width + x) * 4;
                pixels[idx] = 0x10;
                pixels[idx + 1] = 0xB9;
                pixels[idx + 2] = 0x81; // Green pitch bend indicator
            }
        }

        save_png_file(path, width, height, &pixels)
    }
}

#[cfg(feature = "gui")]
impl KotoView {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Koto & Guzheng — Asian Zither Physical Modeling HUD");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Preset:");
                if ui.selectable_label(self.preset == KotoHudPreset::HirajoshiTraditionalSilk, "Hirajoshi Silk").clicked() {
                    self.set_preset(KotoHudPreset::HirajoshiTraditionalSilk);
                }
                if ui.selectable_label(self.preset == KotoHudPreset::KokinJoshiUrbanClassical, "Kokin-joshi Urban").clicked() {
                    self.set_preset(KotoHudPreset::KokinJoshiUrbanClassical);
                }
                if ui.selectable_label(self.preset == KotoHudPreset::InSenContemporaryDramatic, "In-sen Dramatic").clicked() {
                    self.set_preset(KotoHudPreset::InSenContemporaryDramatic);
                }
                if ui.selectable_label(self.preset == KotoHudPreset::KumoiJoshiSpringRain, "Kumoi-joshi Spring").clicked() {
                    self.set_preset(KotoHudPreset::KumoiJoshiSpringRain);
                }
                if ui.selectable_label(self.preset == KotoHudPreset::RyukyuFestiveOkinawa, "Ryukyu Okinawan").clicked() {
                    self.set_preset(KotoHudPreset::RyukyuFestiveOkinawa);
                }
                if ui.selectable_label(self.preset == KotoHudPreset::ConcertGuzhengVirtuoso, "Guzheng Virtuoso").clicked() {
                    self.set_preset(KotoHudPreset::ConcertGuzhengVirtuoso);
                }
            });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("Tsume Strike & Oshi-Ite Bending:");
                    ui.add(egui::Slider::new(&mut self.tsume_velocity, MIN_KOTO_TSUME_VELOCITY..=MAX_KOTO_TSUME_VELOCITY).text("Tsume Velocity"));
                    ui.add(egui::Slider::new(&mut self.oshi_ite_force_n, MIN_OSHI_ITE_FORCE_N..=MAX_OSHI_ITE_FORCE_N).text("Oshi-Ite Force (N)"));
                    ui.add(egui::Slider::new(&mut self.hiki_iro_release, 0.0..=1.0).text("Hiki-Iro Release"));
                });

                ui.separator();

                ui.vertical(|ui| {
                    ui.label("Acoustics & Soundboard Coupling:");
                    ui.add(egui::Slider::new(&mut self.behind_bridge_bleed, 0.0..=1.0).text("Behind-Bridge Bleed"));
                    ui.add(egui::Slider::new(&mut self.body_wood_resonance, 0.0..=1.0).text("Paulownia Resonance"));
                    ui.add(egui::Slider::new(&mut self.ji_bridge_offset, -0.20..=0.20).text("Ji Bridge Offset"));
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
    fn test_koto_view_hit_target_dimensions() {
        const {
            assert!(
                KOTO_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT,
                "Koto puck hit target bounding box must be >= 44pt"
            );
        }
    }

    #[test]
    fn test_koto_view_ascii_render() {
        let view = KotoView::new();
        let ascii = view.render_ascii(80, 16);
        assert_eq!(ascii.len(), 16);
        assert!(ascii[0].starts_with('+'));
    }

    #[test]
    fn test_koto_view_snapshot_render() {
        let view = KotoView::new();
        let res = view.render_snapshot_png("scratch/renders/koto_view.png", 800, 520);
        assert!(res.is_ok());
    }
}
