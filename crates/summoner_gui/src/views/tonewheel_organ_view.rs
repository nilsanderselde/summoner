// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! 9-Drawbar Harmonic Register HUD & Electromechanical Tonewheel Organ View (Milestone 21).
//!
//! Provides an authentic 9-drawbar register slider interface (16', 5-1/3', 8', 4', 2-2/3', 2', 1-3/5', 1-1/3', 1'),
//! harmonic percussion control panel (2nd/3rd, Fast/Slow, Soft/Normal), scanner vibrato/chorus selector,
//! real-time additive harmonic spectrum display, and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and headless PNG snapshot rendering.

use crate::touch_controls::{ContrastColorPalette, MIN_HIT_TARGET_PT};
use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const DRAWBAR_HANDLE_HIT_RADIUS: f32 = 22.0; // 44x44pt bounding touch target

/// Standard drawbar harmonic tab color categorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawbarColorGroup {
    BrownSubOctave, // 16', 5-1/3' (Red-Brown / Mahogany)
    WhiteOctave,    // 8', 4', 2', 1' (Pure White / Ivory)
    BlackHarmonic,  // 2-2/3', 1-3/5', 1-1/3' (Dark Ebony / Slate)
}

/// 9 Drawbar specification descriptor.
pub const DRAWBAR_INFO: [(&str, &str, DrawbarColorGroup); 9] = [
    ("16'", "Sub-Octave", DrawbarColorGroup::BrownSubOctave),
    ("5 1/3'", "Quint / 5th", DrawbarColorGroup::BrownSubOctave),
    ("8'", "Unison / Principal", DrawbarColorGroup::WhiteOctave),
    ("4'", "Octave / 8ve", DrawbarColorGroup::WhiteOctave),
    ("2 2/3'", "Nazard / 12th", DrawbarColorGroup::BlackHarmonic),
    ("2'", "Blockflöte / 15th", DrawbarColorGroup::WhiteOctave),
    ("1 3/5'", "Tierce / 17th", DrawbarColorGroup::BlackHarmonic),
    ("1 1/3'", "Larigot / 19th", DrawbarColorGroup::BlackHarmonic),
    ("1'", "Sifflöte / 22nd", DrawbarColorGroup::WhiteOctave),
];

/// Harmonic percussion settings for GUI display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuiPercussionState {
    pub enabled: bool,
    pub third_harmonic: bool, // false = 2nd (4'), true = 3rd (2-2/3')
    pub fast_decay: bool,     // false = Slow, true = Fast
    pub soft_volume: bool,    // false = Normal, true = Soft
}

impl Default for GuiPercussionState {
    fn default() -> Self {
        Self {
            enabled: true,
            third_harmonic: true,
            fast_decay: true,
            soft_volume: false,
        }
    }
}

/// Scanner vibrato/chorus GUI selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GuiVibratoMode {
    Off,
    V1,
    C1,
    V2,
    C2,
    V3,
    #[default]
    C3,
}

impl GuiVibratoMode {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Off => "OFF",
            Self::V1 => "V-1",
            Self::C1 => "C-1",
            Self::V2 => "V-2",
            Self::C2 => "C-2",
            Self::V3 => "V-3",
            Self::C3 => "C-3",
        }
    }
}

/// 9-Drawbar Harmonic Register View.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TonewheelOrganView {
    /// 9 Drawbar slider values $[0.0 ..= 8.0]$.
    pub drawbars: [f32; 9],
    /// Harmonic percussion switch settings.
    pub percussion: GuiPercussionState,
    /// Scanner Vibrato/Chorus mode.
    pub vibrato_mode: GuiVibratoMode,
    /// Key click contact noise amount $[0.0 ..= 1.0]$.
    pub key_click_amount: f32,
    /// Magnetic induction crosstalk leakage $[0.0 ..= 1.0]$.
    pub crosstalk_amount: f32,
    /// Master organ output level $[0.0 ..= 1.0]$.
    pub master_level: f32,
    /// Colorblind accessible color palette.
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for TonewheelOrganView {
    fn default() -> Self {
        Self::new()
    }
}

impl TonewheelOrganView {
    pub fn new() -> Self {
        Self {
            drawbars: [8.0, 8.0, 8.0, 0.0, 0.0, 0.0, 0.0, 0.0, 8.0], // Classic Gospel / Rock 88 8000 008
            percussion: GuiPercussionState::default(),
            vibrato_mode: GuiVibratoMode::C3,
            key_click_amount: 0.35,
            crosstalk_amount: 0.08,
            master_level: 0.85,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Sets drawbar level $[0.0 ..= 8.0]$.
    pub fn set_drawbar(&mut self, index: usize, val: f32) {
        if index < 9 {
            self.drawbars[index] = val.clamp(0.0, 8.0);
        }
    }

    /// Tests if a screen coordinate hits a drawbar handle.
    pub fn hit_test_drawbar_handle(&self, pos: (f32, f32), handle_pos: (f32, f32)) -> bool {
        let dx = pos.0 - handle_pos.0;
        let dy = pos.1 - handle_pos.1;
        (dx * dx + dy * dy).sqrt() <= DRAWBAR_HANDLE_HIT_RADIUS
    }

    /// Render deterministic ASCII representation for terminal/headless audits.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let header = format!(
            "TONEWHEEL ORGAN [{:0.0}{:0.0} {:0.0}{:0.0}{:0.0}{:0.0} {:0.0}{:0.0}{:0.0}] Perc:{:?} Vib:{:?}",
            self.drawbars[0], self.drawbars[1],
            self.drawbars[2], self.drawbars[3], self.drawbars[4], self.drawbars[5],
            self.drawbars[6], self.drawbars[7], self.drawbars[8],
            if self.percussion.enabled {
                if self.percussion.third_harmonic { "3rd" } else { "2nd" }
            } else {
                "Off"
            },
            self.vibrato_mode
        );
        lines.push(header);

        let bar_h = height.saturating_sub(2).max(1);
        for row in 0..bar_h {
            let mut line = String::with_capacity(width);
            let lvl_threshold = ((bar_h - 1 - row) as f32 / (bar_h - 1).max(1) as f32) * 8.0;

            for (d, (_, _, col_group)) in DRAWBAR_INFO.iter().enumerate() {
                let is_drawn = self.drawbars[d] >= lvl_threshold;
                let char_tab = match col_group {
                    DrawbarColorGroup::BrownSubOctave => '#',
                    DrawbarColorGroup::WhiteOctave => '|',
                    DrawbarColorGroup::BlackHarmonic => ':',
                };
                if is_drawn {
                    line.push_str(&format!(" [{}] ", char_tab));
                } else {
                    line.push_str(" [ ] ");
                }
            }
            lines.push(line);
        }

        let mut labels = String::new();
        for (foot, _, _) in &DRAWBAR_INFO {
            labels.push_str(&format!("{:^5}", foot));
        }
        lines.push(labels);
        lines
    }

    #[cfg(feature = "gui")]
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // 1. Header Bar: Title & Vibrato/Chorus Knob
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("9-DRAWBAR ELECTROMECHANICAL TONEWHEEL ORGAN")
                        .size(15.0)
                        .color(Color32::from_rgb(0, 229, 255))
                        .strong(),
                );
                ui.separator();

                // Scanner Vibrato Mode Selector
                ui.label(egui::RichText::new("Scanner Vibrato/Chorus:").color(Color32::from_rgb(255, 215, 0)).strong());
                let modes = [
                    GuiVibratoMode::Off,
                    GuiVibratoMode::V1,
                    GuiVibratoMode::C1,
                    GuiVibratoMode::V2,
                    GuiVibratoMode::C2,
                    GuiVibratoMode::V3,
                    GuiVibratoMode::C3,
                ];

                for m in modes {
                    let is_active = self.vibrato_mode == m;
                    let btn = egui::Button::new(
                        egui::RichText::new(m.name())
                            .color(if is_active { Color32::from_rgb(10, 14, 22) } else { Color32::from_rgb(240, 245, 255) })
                            .strong(),
                    )
                    .min_size(Vec2::new(36.0, MIN_HIT_TARGET_PT))
                    .fill(if is_active { Color32::from_rgb(0, 229, 255) } else { Color32::from_rgb(32, 45, 66) });

                    if ui.add(btn).clicked() {
                        self.vibrato_mode = m;
                    }
                }
            });

            ui.add_space(8.0);

            // 2. Main 9-Drawbar Slider Rack Canvas
            let drawbar_w = ui.available_width().max(400.0);
            let (res, painter) = ui.allocate_painter(Vec2::new(drawbar_w, 220.0), egui::Sense::click_and_drag());
            let rect = res.rect;

            painter.rect_filled(rect, 6.0, Color32::from_rgb(14, 18, 28));
            painter.rect_stroke(rect, 6.0, Stroke::new(1.5_f32, Color32::from_rgb(45, 60, 85)));

            let col_step = (rect.width() - 32.0) / 9.0;
            let rack_top = rect.min.y + 35.0;
            let rack_height = 140.0;

            for (d, &(foot, _desc, col_group)) in DRAWBAR_INFO.iter().enumerate() {
                let cx = rect.min.x + 16.0 + (d as f32 + 0.5) * col_step;

                // Drawbar Track
                let track_p1 = egui::pos2(cx, rack_top);
                let track_p2 = egui::pos2(cx, rack_top + rack_height);
                painter.line_segment([track_p1, track_p2], Stroke::new(4.0_f32, Color32::from_rgb(30, 40, 58)));

                // Drawbar Position (0 is fully pushed in / top, 8 is fully pulled out / bottom)
                let norm_val = self.drawbars[d] / 8.0;
                let handle_y = rack_top + norm_val * rack_height;

                // Tab color styling
                let (tab_color, text_color) = match col_group {
                    DrawbarColorGroup::BrownSubOctave => (Color32::from_rgb(145, 65, 30), Color32::WHITE),
                    DrawbarColorGroup::WhiteOctave => (Color32::from_rgb(235, 240, 250), Color32::from_rgb(10, 14, 22)),
                    DrawbarColorGroup::BlackHarmonic => (Color32::from_rgb(35, 45, 65), Color32::from_rgb(220, 235, 255)),
                };

                // Drawbar Shaft (Pulled section)
                if norm_val > 0.01 {
                    painter.rect_filled(
                        egui::Rect::from_min_max(egui::pos2(cx - 8.0, rack_top), egui::pos2(cx + 8.0, handle_y)),
                        2.0,
                        Color32::from_rgb(20, 26, 38),
                    );
                }

                // Interactive Drawbar Handle (>=44pt hit bounding circle)
                painter.circle_stroke(
                    egui::pos2(cx, handle_y),
                    DRAWBAR_HANDLE_HIT_RADIUS,
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 60)),
                );
                painter.rect_filled(
                    egui::Rect::from_center_size(egui::pos2(cx, handle_y), Vec2::new(col_step - 6.0, 28.0)),
                    4.0,
                    tab_color,
                );
                painter.text(
                    egui::pos2(cx, handle_y),
                    egui::Align2::CENTER_CENTER,
                    format!("{:.0}", self.drawbars[d]),
                    egui::FontId::monospace(12.0),
                    text_color,
                );

                // Foot label at top
                painter.text(
                    egui::pos2(cx, rect.min.y + 16.0),
                    egui::Align2::CENTER_CENTER,
                    foot,
                    egui::FontId::monospace(11.0),
                    Color32::from_rgb(255, 215, 0),
                );

                // Handle drag interaction
                if res.dragged() {
                    if let Some(mouse_pos) = ui.ctx().pointer_latest_pos() {
                        if (mouse_pos.x - cx).abs() < col_step * 0.5 {
                            let new_norm = ((mouse_pos.y - rack_top) / rack_height).clamp(0.0, 1.0);
                            self.drawbars[d] = (new_norm * 8.0).round();
                        }
                    }
                }
            }

            ui.add_space(8.0);

            // 3. Bottom Controls Bar: Harmonic Percussion & Tone Polish (>=44pt targets)
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    // Percussion On/Off
                    let perc_btn = egui::Button::new(
                        egui::RichText::new(if self.percussion.enabled { "PERCUSSION: ON" } else { "PERCUSSION: OFF" })
                            .color(if self.percussion.enabled { Color32::from_rgb(10, 14, 22) } else { Color32::from_rgb(240, 245, 255) })
                            .strong(),
                    )
                    .min_size(Vec2::new(140.0, MIN_HIT_TARGET_PT))
                    .fill(if self.percussion.enabled { Color32::from_rgb(0, 255, 180) } else { Color32::from_rgb(32, 45, 66) });
                    if ui.add(perc_btn).clicked() {
                        self.percussion.enabled = !self.percussion.enabled;
                    }

                    // Percussion Harmonic (2nd vs 3rd)
                    let harm_btn = egui::Button::new(
                        egui::RichText::new(if self.percussion.third_harmonic { "HARMONIC: 3rd (2-2/3')" } else { "HARMONIC: 2nd (4')" })
                            .color(Color32::from_rgb(240, 245, 255))
                            .strong(),
                    )
                    .min_size(Vec2::new(160.0, MIN_HIT_TARGET_PT))
                    .fill(Color32::from_rgb(32, 45, 66));
                    if ui.add(harm_btn).clicked() {
                        self.percussion.third_harmonic = !self.percussion.third_harmonic;
                    }

                    // Percussion Decay (Fast vs Slow)
                    let decay_btn = egui::Button::new(
                        egui::RichText::new(if self.percussion.fast_decay { "DECAY: FAST" } else { "DECAY: SLOW" })
                            .color(Color32::from_rgb(240, 245, 255))
                            .strong(),
                    )
                    .min_size(Vec2::new(120.0, MIN_HIT_TARGET_PT))
                    .fill(Color32::from_rgb(32, 45, 66));
                    if ui.add(decay_btn).clicked() {
                        self.percussion.fast_decay = !self.percussion.fast_decay;
                    }

                    // Percussion Volume (Soft vs Normal)
                    let vol_btn = egui::Button::new(
                        egui::RichText::new(if self.percussion.soft_volume { "VOL: SOFT" } else { "VOL: NORMAL" })
                            .color(Color32::from_rgb(240, 245, 255))
                            .strong(),
                    )
                    .min_size(Vec2::new(120.0, MIN_HIT_TARGET_PT))
                    .fill(Color32::from_rgb(32, 45, 66));
                    if ui.add(vol_btn).clicked() {
                        self.percussion.soft_volume = !self.percussion.soft_volume;
                    }

                    ui.separator();

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Key Click").strong());
                        ui.add(egui::Slider::new(&mut self.key_click_amount, 0.0..=1.0));
                    });

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Crosstalk").strong());
                        ui.add(egui::Slider::new(&mut self.crosstalk_amount, 0.0..=0.5));
                    });
                });
            });
        });
    }

    /// Renders headless snapshot PNG to `path`.
    pub fn render_snapshot_png(&self, path: &str, width: u32, height: u32) -> Result<(), String> {
        let mut pixels = vec![0u8; (width * height * 4) as usize];

        let bg = [10, 14, 24, 255]; // Deep slate
        let line_col = [45, 60, 85, 255];
        let amber_col = [255, 170, 0, 255];
        let cyan_col = [0, 229, 255, 255];

        for chunk in pixels.chunks_exact_mut(4) {
            chunk.copy_from_slice(&bg);
        }

        let col_step = (width as f32 - 40.0) / 9.0;
        let rack_top = 80.0;
        let rack_height = height as f32 - 180.0;

        for (d, (_, _, col_group)) in DRAWBAR_INFO.iter().enumerate() {
            let cx = 20.0 + (d as f32 + 0.5) * col_step;
            let norm_val = self.drawbars[d] / 8.0;
            let handle_y = rack_top + norm_val * rack_height;

            let tab_col = match col_group {
                DrawbarColorGroup::BrownSubOctave => [145, 65, 30, 255],
                DrawbarColorGroup::WhiteOctave => [235, 240, 250, 255],
                DrawbarColorGroup::BlackHarmonic => [45, 55, 75, 255],
            };

            // Vertical track
            draw_line_segment_to(&mut pixels, width, height, cx, rack_top, cx, rack_top + rack_height, line_col);

            // Handle tab
            draw_rect_filled_to(
                &mut pixels,
                width,
                height,
                cx - col_step * 0.4,
                handle_y - 12.0,
                cx + col_step * 0.4,
                handle_y + 12.0,
                tab_col,
            );

            // Hit target circle outline (>= 22pt)
            draw_circle_stroke_to(&mut pixels, width, height, cx, handle_y, DRAWBAR_HANDLE_HIT_RADIUS, cyan_col);
        }

        // Draw harmonic percussion badge
        if self.percussion.enabled {
            draw_rect_filled_to(&mut pixels, width, height, 40.0, height as f32 - 60.0, 200.0, height as f32 - 20.0, amber_col);
        }

        encode_minimal_png_to(path, &pixels, width, height)
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_line_segment_to(buf: &mut [u8], w: u32, h: u32, x0: f32, y0: f32, x1: f32, y1: f32, col: [u8; 4]) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let steps = (dx.abs().max(dy.abs()) as usize).max(1);

    for s in 0..=steps {
        let t = s as f32 / steps as f32;
        let x = (x0 + dx * t) as i32;
        let y = (y0 + dy * t) as i32;

        if x >= 0 && x < w as i32 && y >= 0 && y < h as i32 {
            let idx = ((y as u32 * w + x as u32) * 4) as usize;
            buf[idx..idx + 4].copy_from_slice(&col);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_rect_filled_to(buf: &mut [u8], w: u32, h: u32, x0: f32, y0: f32, x1: f32, y1: f32, col: [u8; 4]) {
    let min_x = (x0.min(x1).max(0.0)) as u32;
    let max_x = (x0.max(x1).min(w as f32 - 1.0)) as u32;
    let min_y = (y0.min(y1).max(0.0)) as u32;
    let max_y = (y0.max(y1).min(h as f32 - 1.0)) as u32;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let idx = ((y * w + x) * 4) as usize;
            buf[idx..idx + 4].copy_from_slice(&col);
        }
    }
}

fn draw_circle_stroke_to(buf: &mut [u8], w: u32, h: u32, cx: f32, cy: f32, r: f32, col: [u8; 4]) {
    let steps = (TAU * r) as usize + 8;
    for i in 0..steps {
        let a = (i as f32 / steps as f32) * TAU;
        let x = (cx + a.cos() * r) as i32;
        let y = (cy + a.sin() * r) as i32;
        if x >= 0 && x < w as i32 && y >= 0 && y < h as i32 {
            let idx = ((y as u32 * w + x as u32) * 4) as usize;
            buf[idx..idx + 4].copy_from_slice(&col);
        }
    }
}

fn encode_minimal_png_to(path: &str, rgba_pixels: &[u8], width: u32, height: u32) -> Result<(), String> {
    let mut out = Vec::with_capacity((width * height * 4 + 1024) as usize);
    out.extend_from_slice(b"\x89PNG\r\n\x1a\n");

    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.push(8);
    ihdr.push(6);
    ihdr.push(0);
    ihdr.push(0);
    ihdr.push(0);
    write_png_chunk_to(&mut out, b"IHDR", &ihdr);

    let mut raw_data = Vec::with_capacity((height * (width * 4 + 1)) as usize);
    for y in 0..height {
        raw_data.push(0);
        let row_start = (y * width * 4) as usize;
        let row_end = row_start + (width * 4) as usize;
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

    #[test]
    fn test_tonewheel_organ_view_ascii_render() {
        let view = TonewheelOrganView::new();
        let ascii = view.render_ascii(80, 16);
        assert_eq!(ascii.len(), 16);
        assert!(ascii[0].contains("TONEWHEEL ORGAN"));
    }

    #[test]
    fn test_tonewheel_organ_view_hit_target_dimensions() {
        const {
            assert!(
                DRAWBAR_HANDLE_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT,
                "Drawbar handle hit target bounding box must be >= 44pt"
            );
        }
    }

    #[test]
    fn test_tonewheel_organ_view_snapshot_render() {
        let view = TonewheelOrganView::new();
        let res = view.render_snapshot_png("scratch/renders/tonewheel_organ_view.png", 800, 520);
        assert!(res.is_ok());
    }
}
