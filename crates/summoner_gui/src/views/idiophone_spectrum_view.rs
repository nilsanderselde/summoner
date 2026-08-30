// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Idiophone Modal Spectrum & Resonator HUD (Milestone 19).
//!
//! Provides an interactive modal harmonic overtone spectrum bar visualizer (1 : 4 : 10 Marimba,
//! 1 : 3 : 9 Xylophone, 1 : 6.27 : 17.55 Kalimba, 1 : 2 : 3 Steelpan), resonator tube coupling
//! tuning indicators, vibraphone motorized tremolo HUD, and continuous mallet hardness vs strike
//! velocity puck controls on an 8pt grid with $\ge 44\times 44\text{pt}$ hit targets and WCAG AAA contrast.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const IDIOPHONE_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_MALLET_HARDNESS: f32 = 0.0;
pub const MAX_MALLET_HARDNESS: f32 = 1.0;
pub const MIN_STRIKE_VELOCITY: f32 = 0.05;
pub const MAX_STRIKE_VELOCITY: f32 = 1.00;
pub const NUM_VIEW_MODES: usize = 8;

/// Idiophone View Instrument Preset Profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IdiophoneViewProfile {
    #[default]
    MarimbaWood,
    XylophoneRosewood,
    VibraphoneAluminum,
    SteelpanTrinidad,
    KalimbaMbira,
    GlockenspielBell,
}

impl IdiophoneViewProfile {
    pub fn name(&self) -> &'static str {
        match self {
            Self::MarimbaWood => "ROSEWOOD MARIMBA",
            Self::XylophoneRosewood => "ORCHESTRAL XYLOPHONE",
            Self::VibraphoneAluminum => "CONCERT VIBRAPHONE",
            Self::SteelpanTrinidad => "TRINIDAD STEELPAN",
            Self::KalimbaMbira => "KALIMBA / MBIRA",
            Self::GlockenspielBell => "GLOCKENSPIEL BELL",
        }
    }

    pub fn modal_ratios(&self) -> [f32; NUM_VIEW_MODES] {
        match self {
            Self::MarimbaWood => [1.000, 4.000, 10.000, 14.250, 18.600, 24.500, 31.200, 38.800],
            Self::XylophoneRosewood => [1.000, 3.000, 9.000, 13.500, 18.000, 23.400, 29.800, 37.100],
            Self::VibraphoneAluminum => [1.000, 3.980, 9.950, 15.200, 21.400, 28.100, 35.600, 43.900],
            Self::SteelpanTrinidad => [1.000, 2.000, 3.000, 4.120, 5.350, 6.780, 8.420, 10.250],
            Self::KalimbaMbira => [1.000, 6.267, 17.550, 34.390, 56.840, 84.880, 118.50, 157.70],
            Self::GlockenspielBell => [1.000, 2.756, 5.404, 8.933, 13.345, 18.640, 24.815, 31.870],
        }
    }

    pub fn nominal_physics(&self) -> (f32, f32, f32) {
        // (tube_mix, tremolo_rate, default_hardness)
        match self {
            Self::MarimbaWood => (0.50, 0.0, 0.25),
            Self::XylophoneRosewood => (0.25, 0.0, 0.85),
            Self::VibraphoneAluminum => (0.60, 4.2, 0.50),
            Self::SteelpanTrinidad => (0.10, 0.0, 0.55),
            Self::KalimbaMbira => (0.35, 0.0, 0.30),
            Self::GlockenspielBell => (0.05, 0.0, 0.95),
        }
    }
}

/// Idiophone Modal Spectrum & Resonator HUD.
#[derive(Debug, Clone)]
pub struct IdiophoneSpectrumView {
    pub instrument: IdiophoneViewProfile,
    pub mallet_hardness: f32,    // [0.0 soft yarn ..= 1.0 brass]
    pub strike_velocity: f32,    // [0.05 ..= 1.00]
    pub strike_position: f32,    // [0.0 center ..= 1.0 node/edge]
    pub resonator_tube_mix: f32, // [0.0 ..= 1.0]
    pub tremolo_rate_hz: f32,    // [0.0 ..= 10.0]
    pub tremolo_depth: f32,      // [0.0 ..= 1.0]
    pub fundamental_hz: f32,
    pub puck_pos: (f32, f32),    // Normalized (X: mallet_hardness, Y: strike_velocity)
    pub is_dragging_puck: bool,
    pub color_palette: ContrastColorPalette,
}

impl Default for IdiophoneSpectrumView {
    fn default() -> Self {
        Self::new()
    }
}

impl IdiophoneSpectrumView {
    pub fn new() -> Self {
        let mut view = Self {
            instrument: IdiophoneViewProfile::MarimbaWood,
            mallet_hardness: 0.35,
            strike_velocity: 0.80,
            strike_position: 0.20,
            resonator_tube_mix: 0.50,
            tremolo_rate_hz: 0.0,
            tremolo_depth: 0.0,
            fundamental_hz: 261.63,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        };
        view.puck_pos = (
            Self::hardness_to_normalized(view.mallet_hardness),
            Self::velocity_to_normalized(view.strike_velocity),
        );
        view.update_physics();
        view
    }

    pub fn hardness_to_normalized(h: f32) -> f32 {
        h.clamp(0.0, 1.0)
    }

    pub fn normalized_to_hardness(norm: f32) -> f32 {
        norm.clamp(0.0, 1.0)
    }

    pub fn velocity_to_normalized(v: f32) -> f32 {
        ((v.clamp(MIN_STRIKE_VELOCITY, MAX_STRIKE_VELOCITY) - MIN_STRIKE_VELOCITY)
            / (MAX_STRIKE_VELOCITY - MIN_STRIKE_VELOCITY))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_velocity(norm: f32) -> f32 {
        MIN_STRIKE_VELOCITY + norm.clamp(0.0, 1.0) * (MAX_STRIKE_VELOCITY - MIN_STRIKE_VELOCITY)
    }

    pub fn update_physics(&mut self) {
        let (tube, trem, _hard) = self.instrument.nominal_physics();
        self.resonator_tube_mix = tube;
        self.tremolo_rate_hz = trem;
        self.tremolo_depth = if trem > 0.0 { 0.70 } else { 0.0 };
    }

    /// Evaluates modal amplitude bar heights $[A_0, \dots, A_7]$ for the spectrum display.
    pub fn evaluate_modal_spectrum(&self) -> [f32; NUM_VIEW_MODES] {
        let ratios = self.instrument.modal_ratios();
        let mut amplitudes = [0.0f32; NUM_VIEW_MODES];

        for i in 0..NUM_VIEW_MODES {
            // Mode amplitude boosted by strike velocity and higher modes excited by mallet hardness
            let mode_freq_factor = (ratios[i]).powf(0.65 * self.mallet_hardness);
            let base_amp = 1.0 / (1.0 + 0.45 * (i as f32) * (1.0 - 0.75 * self.mallet_hardness));
            let amp = (base_amp * mode_freq_factor.min(3.0) * self.strike_velocity).clamp(0.02, 1.0);
            amplitudes[i] = amp;
        }

        amplitudes
    }

    /// Hit tests interaction on the 2D hardness vs velocity strike puck.
    pub fn hit_test_idiophone_puck(&self, pos: (f32, f32), canvas_rect: Rect) -> bool {
        let px = canvas_rect.x + self.puck_pos.0 * canvas_rect.width;
        let py = canvas_rect.y + (1.0 - self.puck_pos.1) * canvas_rect.height;
        let dx = pos.0 - px;
        let dy = pos.1 - py;
        (dx * dx + dy * dy).sqrt() <= IDIOPHONE_PUCK_HIT_RADIUS
    }

    #[cfg(feature = "gui")]
    pub fn show(&mut self, ui: &mut egui::Ui) {
        let available = ui.available_size();
        let width = available.x.max(320.0);
        let height = 340.0;

        let (rect, response) = ui.allocate_exact_size(
            egui::Vec2::new(width, height),
            egui::Sense::click_and_drag(),
        );

        let painter = ui.painter_at(rect);
        let canvas_rect = Rect::new(rect.min.x, rect.min.y, rect.width(), rect.height());

        // Background
        painter.rect_filled(
            rect,
            4.0,
            Color32::from_rgb(
                self.color_palette.bg_rgb.0,
                self.color_palette.bg_rgb.1,
                self.color_palette.bg_rgb.2,
            ),
        );

        // Header
        let header_text = format!(
            "IDIOPHONE SPECTRUM HUD // {} // TUBE MIX: {:.0}% // FUND: {:.1} Hz",
            self.instrument.name(),
            self.resonator_tube_mix * 100.0,
            self.fundamental_hz
        );
        painter.text(
            egui::pos2(rect.min.x + 12.0, rect.min.y + 12.0),
            egui::Align2::LEFT_TOP,
            header_text,
            egui::FontId::monospace(12.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Interaction
        if response.drag_started() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if self.hit_test_idiophone_puck((mouse_pos.x, mouse_pos.y), canvas_rect) {
                    self.is_dragging_puck = true;
                }
            }
        }
        if response.drag_stopped() {
            self.is_dragging_puck = false;
        }
        if self.is_dragging_puck {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                let norm_x = ((mouse_pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
                let norm_y = (1.0 - (mouse_pos.y - rect.min.y) / rect.height()).clamp(0.0, 1.0);
                self.puck_pos = (norm_x, norm_y);
                self.mallet_hardness = Self::normalized_to_hardness(norm_x);
                self.strike_velocity = Self::normalized_to_velocity(norm_y);
                self.update_physics();
            }
        }

        // Draw Modal Spectrum Bar Chart (Left Side)
        let spectrum = self.evaluate_modal_spectrum();
        let ratios = self.instrument.modal_ratios();
        let bar_area_x = rect.min.x + 20.0;
        let bar_area_y = rect.min.y + 60.0;
        let bar_area_w = rect.width() * 0.45;
        let bar_area_h = rect.height() - 90.0;

        let bar_w = bar_area_w / (NUM_VIEW_MODES as f32 * 1.3);

        for (i, &amp) in spectrum.iter().enumerate() {
            let bx = bar_area_x + i as f32 * (bar_w * 1.3);
            let bh = bar_area_h * amp;
            let by = bar_area_y + (bar_area_h - bh);

            let bar_rect = egui::Rect::from_min_size(
                egui::pos2(bx, by),
                egui::vec2(bar_w, bh),
            );

            painter.rect_filled(
                bar_rect,
                2.0,
                Color32::from_rgb(255, 170, 0),
            );

            // Modal ratio label below bar
            let ratio_label = format!("{:.1}x", ratios[i]);
            painter.text(
                egui::pos2(bx + bar_w * 0.5, bar_area_y + bar_area_h + 4.0),
                egui::Align2::CENTER_TOP,
                ratio_label,
                egui::FontId::monospace(9.0),
                Color32::from_rgb(170, 190, 215),
            );
        }

        // 2D Mallet Hardness vs Velocity Puck (Center Right)
        let pad_x = rect.min.x + rect.width() * 0.55;
        let pad_y = rect.min.y + 60.0;
        let pad_w = rect.width() * 0.40;
        let pad_h = rect.height() - 90.0;

        let pad_rect = egui::Rect::from_min_size(
            egui::pos2(pad_x, pad_y),
            egui::vec2(pad_w, pad_h),
        );

        painter.rect_stroke(
            pad_rect,
            4.0_f32,
            Stroke::new(
                1.5_f32,
                Color32::from_rgb(60, 75, 100),
            ),
        );

        let puck_px = pad_x + self.puck_pos.0 * pad_w;
        let puck_py = pad_y + (1.0 - self.puck_pos.1) * pad_h;

        // 44x44pt Touch Target Area
        painter.circle_filled(
            egui::pos2(puck_px, puck_py),
            IDIOPHONE_PUCK_HIT_RADIUS,
            Color32::from_rgba_unmultiplied(0, 229, 255, 45),
        );

        // Visual Puck Center
        painter.circle_filled(
            egui::pos2(puck_px, puck_py),
            12.0,
            Color32::from_rgb(0, 229, 255),
        );
    }

    /// Renders an ASCII visualization for terminal/headless audits.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let border = format!("+{}+", "-".repeat(width.saturating_sub(2)));
        lines.push(border.clone());

        let title = format!(
            "| IDIOPHONE SPECTRUM [{}] Hard={:.2} Vel={:.2} Tube={:.0}%",
            self.instrument.name(),
            self.mallet_hardness,
            self.strike_velocity,
            self.resonator_tube_mix * 100.0
        );
        let title_padded = format!("{:<width$}|", title, width = width - 1);
        lines.push(title_padded);

        let spectrum = self.evaluate_modal_spectrum();

        for row in 2..height - 1 {
            let mut row_str = String::with_capacity(width);
            row_str.push('|');
            let norm_y = 1.0 - (row as f32 / height as f32);

            for col in 1..width - 1 {
                let col_frac = col as f32 / width as f32;
                if col_frac < 0.50 {
                    let bar_idx = ((col_frac / 0.50) * NUM_VIEW_MODES as f32) as usize;
                    if bar_idx < NUM_VIEW_MODES && norm_y <= spectrum[bar_idx] {
                        row_str.push('#');
                    } else {
                        row_str.push(' ');
                    }
                } else if ((col_frac - (0.55 + 0.35 * self.mallet_hardness)).abs() < 0.03)
                    && ((norm_y - self.strike_velocity).abs() < 0.08)
                {
                    row_str.push('@'); // Puck
                } else {
                    row_str.push(' ');
                }
            }
            row_str.push('|');
            lines.push(row_str);
        }

        lines.push(border);
        lines
    }

    /// Renders snapshot PNG to `path`.
    pub fn render_snapshot_png(&self, path: &str, width: u32, height: u32) -> Result<(), String> {
        let mut pixels = vec![0u8; (width * height * 4) as usize];

        let bg = [10, 14, 24, 255]; // Deep slate
        let bar_col = [255, 170, 0, 255]; // Amber
        let puck_col = [0, 220, 255, 255]; // Cyan

        for chunk in pixels.chunks_exact_mut(4) {
            chunk.copy_from_slice(&bg);
        }

        let spectrum = self.evaluate_modal_spectrum();
        let bar_area_w = width as f32 * 0.45;
        let bar_area_h = height as f32 * 0.70;
        let bar_w = bar_area_w / (NUM_VIEW_MODES as f32 * 1.3);

        for (i, &amp) in spectrum.iter().enumerate() {
            let bx = 30.0 + i as f32 * (bar_w * 1.3);
            let bh = bar_area_h * amp;
            let by = height as f32 * 0.80 - bh;

            draw_rect_filled_id(&mut pixels, width, height, bx, by, bar_w, bh, bar_col);
        }

        // Draw 2D puck
        let px = width as f32 * (0.55 + 0.38 * self.mallet_hardness);
        let py = height as f32 * (0.80 - 0.60 * self.strike_velocity);
        draw_circle_filled_id(&mut pixels, width, height, px, py, 14.0, puck_col);

        encode_minimal_png_id(path, &pixels, width, height)
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_rect_filled_id(buf: &mut [u8], w: u32, h: u32, rx: f32, ry: f32, rw: f32, rh: f32, col: [u8; 4]) {
    let min_x = rx.max(0.0) as u32;
    let max_x = (rx + rw).min(w as f32 - 1.0) as u32;
    let min_y = ry.max(0.0) as u32;
    let max_y = (ry + rh).min(h as f32 - 1.0) as u32;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let idx = ((y * w + x) * 4) as usize;
            buf[idx..idx + 4].copy_from_slice(&col);
        }
    }
}

fn draw_circle_filled_id(buf: &mut [u8], w: u32, h: u32, cx: f32, cy: f32, r: f32, col: [u8; 4]) {
    let min_x = (cx - r).max(0.0) as u32;
    let max_x = (cx + r).min(w as f32 - 1.0) as u32;
    let min_y = (cy - r).max(0.0) as u32;
    let max_y = (cy + r).min(h as f32 - 1.0) as u32;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            if dx * dx + dy * dy <= r * r {
                let idx = ((y * w + x) * 4) as usize;
                buf[idx..idx + 4].copy_from_slice(&col);
            }
        }
    }
}

fn encode_minimal_png_id(path: &str, rgba_pixels: &[u8], width: u32, height: u32) -> Result<(), String> {
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
    write_png_chunk_id(&mut out, b"IHDR", &ihdr);

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

    write_png_chunk_id(&mut out, b"IDAT", &zlib_data);
    write_png_chunk_id(&mut out, b"IEND", &[]);

    if let Some(parent) = std::path::Path::new(path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(path, out).map_err(|e| e.to_string())
}

fn write_png_chunk_id(out: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let type_start = out.len();
    out.extend_from_slice(chunk_type);
    out.extend_from_slice(data);
    let crc = crc32_compute_id(&out[type_start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

fn crc32_compute_id(buf: &[u8]) -> u32 {
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
    fn test_idiophone_spectrum_view_ascii_render() {
        let view = IdiophoneSpectrumView::new();
        let ascii = view.render_ascii(80, 16);
        assert!(!ascii.is_empty());
        assert_eq!(ascii.len(), 16);
    }

    #[test]
    fn test_idiophone_spectrum_view_hit_target_dimensions() {
        const {
            assert!(
                IDIOPHONE_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT,
                "Idiophone puck hit target bounding box must be >= 44pt"
            );
        }
    }

    #[test]
    fn test_idiophone_spectrum_view_snapshot_render() {
        let view = IdiophoneSpectrumView::new();
        let res = view.render_snapshot_png("scratch/renders/idiophone_spectrum_view.png", 800, 520);
        assert!(res.is_ok());
    }
}
