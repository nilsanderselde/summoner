// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License
//
// Physical Modeling Japanese Shakuhachi Bamboo Flute / Blowing Edge Chiff & Pitch-Bend Microtone HUD (Milestone 25).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32};

pub const SHAKUHACHI_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_JET_VELOCITY_MPS: f32 = 4.0;
pub const MAX_JET_VELOCITY_MPS: f32 = 42.0;
pub const MIN_UTAGUCHI_ANGLE_DEG: f32 = 10.0;
pub const MAX_UTAGUCHI_ANGLE_DEG: f32 = 60.0;
pub const MIN_MERI_KARI_CENTS: f32 = -300.0;
pub const MAX_MERI_KARI_CENTS: f32 = 200.0;

/// Classical Japanese Shakuhachi flute lengths and pitch reference types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShakuhachiLengthType {
    IchishakuHassun,  // 1.8 Shaku (~54.5cm), standard D4 classical Honkyoku reference
    NishakuYonsun,    // 2.4 Shaku (~72.7cm), deep meditative A3 Jinashi Zen flute
    IchishakuRokusun, // 1.6 Shaku (~48.5cm), bright E4 treble flute for Min'yo folk
    NishakuIssun,     // 2.1 Shaku (~63.6cm), warm low B3 flute for ensemble Sankyoku
    SanShakuKyotaku,  // 3.0 Shaku (~90.9cm), giant sub-bass D3 Kyotaku temple flute
}

impl ShakuhachiLengthType {
    pub fn flute_name(&self) -> &'static str {
        match self {
            Self::IchishakuHassun => "1.8 SHAKU (54.5cm D4 HONKYOKU)",
            Self::NishakuYonsun => "2.4 SHAKU (72.7cm A3 JINASHI ZEN)",
            Self::IchishakuRokusun => "1.6 SHAKU (48.5cm E4 MIN'YO)",
            Self::NishakuIssun => "2.1 SHAKU (63.6cm B3 SANKYOKU)",
            Self::SanShakuKyotaku => "3.0 SHAKU (90.9cm D3 KYOTAKU)",
        }
    }

    pub fn nominal_jet_velocity_mps(&self) -> f32 {
        match self {
            Self::IchishakuHassun => 19.0,
            Self::NishakuYonsun => 14.5,
            Self::IchishakuRokusun => 24.0,
            Self::NishakuIssun => 16.5,
            Self::SanShakuKyotaku => 11.0,
        }
    }

    pub fn nominal_utaguchi_angle_deg(&self) -> f32 {
        match self {
            Self::IchishakuHassun => 38.0,
            Self::NishakuYonsun => 44.0,
            Self::IchishakuRokusun => 34.0,
            Self::NishakuIssun => 40.0,
            Self::SanShakuKyotaku => 48.0,
        }
    }

    pub fn nominal_length_cm(&self) -> f32 {
        match self {
            Self::IchishakuHassun => 54.5,
            Self::NishakuYonsun => 72.7,
            Self::IchishakuRokusun => 48.5,
            Self::NishakuIssun => 63.6,
            Self::SanShakuKyotaku => 90.9,
        }
    }

    pub fn nominal_chiff_noise(&self) -> f32 {
        match self {
            Self::IchishakuHassun => 0.35,
            Self::NishakuYonsun => 0.45,
            Self::IchishakuRokusun => 0.28,
            Self::NishakuIssun => 0.38,
            Self::SanShakuKyotaku => 0.55,
        }
    }

    pub fn nominal_bore_q(&self) -> f32 {
        match self {
            Self::IchishakuHassun => 52.0,
            Self::NishakuYonsun => 44.0,
            Self::IchishakuRokusun => 58.0,
            Self::NishakuIssun => 48.0,
            Self::SanShakuKyotaku => 38.0,
        }
    }
}

/// Traditional Shakuhachi pentatonic fingering modes (5 tone scale).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShakuhachiFingeringNote {
    Ro,  // All 5 holes closed (Root D)
    Tsu, // Hole 1 open (F)
    Re,  // Holes 1 & 2 open (G)
    Chi, // Holes 1, 2 & 3 open (A)
    Ri,  // Holes 1, 2, 3 & 4 open (C)
}

impl ShakuhachiFingeringNote {
    pub fn note_name(&self) -> &'static str {
        match self {
            Self::Ro => "RO (呂 - Root D)",
            Self::Tsu => "TSU (ツ - Minor 3rd F)",
            Self::Re => "RE (レ - 4th G)",
            Self::Chi => "CHI (チ - 5th A)",
            Self::Ri => "RI (リ - Minor 7th C)",
        }
    }

    pub fn open_hole_ratio(&self) -> f32 {
        match self {
            Self::Ro => 0.0,
            Self::Tsu => 0.25,
            Self::Re => 0.50,
            Self::Chi => 0.75,
            Self::Ri => 1.00,
        }
    }
}

/// Physical modeling Japanese Shakuhachi flute / blowing edge chiff HUD.
#[derive(Debug, Clone)]
pub struct ShakuhachiView {
    pub length_type: ShakuhachiLengthType,
    pub fingering_note: ShakuhachiFingeringNote,
    pub jet_velocity_mps: f32,
    pub utaguchi_angle_deg: f32,
    pub meri_kari_cents: f32,
    pub chiff_noise_level: f32,
    pub bamboo_node_dispersion: f32,
    pub acoustic_bore_q: f32,
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    pub modal_amplitudes: [f32; 8],
    pub color_palette: ContrastColorPalette,
}

impl Default for ShakuhachiView {
    fn default() -> Self {
        Self::new()
    }
}

impl ShakuhachiView {
    pub fn new() -> Self {
        let mut view = Self {
            length_type: ShakuhachiLengthType::IchishakuHassun,
            fingering_note: ShakuhachiFingeringNote::Ro,
            jet_velocity_mps: 19.0,
            utaguchi_angle_deg: 38.0,
            meri_kari_cents: 0.0,
            chiff_noise_level: 0.35,
            bamboo_node_dispersion: 0.15,
            acoustic_bore_q: 52.0,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            modal_amplitudes: [1.0, 0.60, 0.35, 0.20, 0.35, 0.40, 0.70, 0.50],
            color_palette: ContrastColorPalette::default(),
        };
        view.puck_pos = (
            Self::angle_to_normalized(view.utaguchi_angle_deg),
            Self::velocity_to_normalized(view.jet_velocity_mps),
        );
        view.update_shakuhachi_simulation();
        view
    }

    pub fn velocity_to_normalized(v: f32) -> f32 {
        let val = v.clamp(MIN_JET_VELOCITY_MPS, MAX_JET_VELOCITY_MPS);
        ((val - MIN_JET_VELOCITY_MPS) / (MAX_JET_VELOCITY_MPS - MIN_JET_VELOCITY_MPS))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_velocity(norm: f32) -> f32 {
        MIN_JET_VELOCITY_MPS + norm.clamp(0.0, 1.0) * (MAX_JET_VELOCITY_MPS - MIN_JET_VELOCITY_MPS)
    }

    pub fn angle_to_normalized(a: f32) -> f32 {
        let val = a.clamp(MIN_UTAGUCHI_ANGLE_DEG, MAX_UTAGUCHI_ANGLE_DEG);
        ((val - MIN_UTAGUCHI_ANGLE_DEG) / (MAX_UTAGUCHI_ANGLE_DEG - MIN_UTAGUCHI_ANGLE_DEG))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_angle(norm: f32) -> f32 {
        MIN_UTAGUCHI_ANGLE_DEG
            + norm.clamp(0.0, 1.0) * (MAX_UTAGUCHI_ANGLE_DEG - MIN_UTAGUCHI_ANGLE_DEG)
    }

    pub fn set_length_type(&mut self, len_type: ShakuhachiLengthType) {
        self.length_type = len_type;
        self.jet_velocity_mps = len_type.nominal_jet_velocity_mps();
        self.utaguchi_angle_deg = len_type.nominal_utaguchi_angle_deg();
        self.chiff_noise_level = len_type.nominal_chiff_noise();
        self.acoustic_bore_q = len_type.nominal_bore_q();
        self.puck_pos = (
            Self::angle_to_normalized(self.utaguchi_angle_deg),
            Self::velocity_to_normalized(self.jet_velocity_mps),
        );
        self.update_shakuhachi_simulation();
    }

    pub fn set_fingering(&mut self, fingering: ShakuhachiFingeringNote) {
        self.fingering_note = fingering;
        self.update_shakuhachi_simulation();
    }

    pub fn update_puck_from_normalized(&mut self, norm_x: f32, norm_y: f32) {
        self.puck_pos = (norm_x.clamp(0.0, 1.0), norm_y.clamp(0.0, 1.0));
        self.utaguchi_angle_deg = Self::normalized_to_angle(self.puck_pos.0);
        self.jet_velocity_mps = Self::normalized_to_velocity(self.puck_pos.1);
        self.update_shakuhachi_simulation();
    }

    pub fn update_shakuhachi_simulation(&mut self) {
        let vel_norm = Self::velocity_to_normalized(self.jet_velocity_mps);
        let angle_norm = Self::angle_to_normalized(self.utaguchi_angle_deg);
        let meri_shift = (self.meri_kari_cents / 300.0).clamp(-1.0, 1.0);
        let open_ratio = self.fingering_note.open_hole_ratio();

        // Fundamental
        self.modal_amplitudes[0] = (1.0 - open_ratio * 0.25 + meri_shift * 0.15).clamp(0.4, 1.0);
        // 2nd partial (octave)
        self.modal_amplitudes[1] = (vel_norm * 0.85 + angle_norm * 0.35).clamp(0.1, 0.95);
        // 3rd partial (12th)
        self.modal_amplitudes[2] = ((1.0 - vel_norm) * 0.45 + open_ratio * 0.4).clamp(0.05, 0.8);
        // 4th partial (double octave)
        self.modal_amplitudes[3] = (vel_norm.powi(2) * 0.65).clamp(0.02, 0.7);
        // 5th partial
        self.modal_amplitudes[4] = (0.2 + self.chiff_noise_level * 0.35).clamp(0.05, 0.6);
        // 6th partial
        self.modal_amplitudes[5] = (0.15 + angle_norm * 0.3).clamp(0.02, 0.5);
        // 7th partial
        self.modal_amplitudes[6] = (self.chiff_noise_level * 0.6).clamp(0.01, 0.7);
        // 8th partial / Meri-Kari inflection
        self.modal_amplitudes[7] = (0.25 + meri_shift.abs() * 0.5 + self.chiff_noise_level * 0.25).clamp(0.01, 0.95);
    }

    pub fn hit_test_shakuhachi_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let center_x = canvas.x + self.puck_pos.0 * canvas.width;
        let center_y = canvas.y + self.puck_pos.1 * canvas.height;
        let dx = point.0 - center_x;
        let dy = point.1 - center_y;
        (dx * dx + dy * dy).sqrt() <= SHAKUHACHI_PUCK_HIT_RADIUS
    }

    #[cfg(feature = "gui")]
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let accent_cyan = Color32::from_rgb(0, 229, 255);
        let accent_amber = Color32::from_rgb(255, 180, 50);
        let accent_green = Color32::from_rgb(0, 255, 180);
        let text_white = Color32::from_rgb(240, 244, 255);
        let text_dim = Color32::from_rgb(140, 150, 170);

        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("SHAKUHACHI BAMBOO FLUTE & CHIFF HUD")
                        .size(16.0)
                        .color(accent_cyan)
                        .strong(),
                );
                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new(format!(
                        "Jet: {:.1} m/s | Angle: {:.1}° | Meri/Kari: {:+.0} ct | Chiff: {:.2}",
                        self.jet_velocity_mps, self.utaguchi_angle_deg, self.meri_kari_cents, self.chiff_noise_level
                    ))
                    .size(12.0)
                    .color(text_dim),
                );
            });

            ui.add_space(8.0);

            // Flue length selector tabs
            ui.horizontal(|ui| {
                let lengths = [
                    (ShakuhachiLengthType::IchishakuHassun, "1.8 Shaku (D4)"),
                    (ShakuhachiLengthType::NishakuYonsun, "2.4 Shaku (A3)"),
                    (ShakuhachiLengthType::IchishakuRokusun, "1.6 Shaku (E4)"),
                    (ShakuhachiLengthType::NishakuIssun, "2.1 Shaku (B3)"),
                    (ShakuhachiLengthType::SanShakuKyotaku, "3.0 Shaku (D3)"),
                ];

                for (ltype, label) in lengths {
                    let is_active = self.length_type == ltype;
                    let btn_bg = if is_active { accent_green } else { Color32::from_rgb(26, 36, 52) };
                    let btn_fg = if is_active { Color32::BLACK } else { text_white };

                    let btn = egui::Button::new(
                        egui::RichText::new(label).size(12.0).color(btn_fg).strong(),
                    )
                    .fill(btn_bg)
                    .min_size(egui::vec2(84.0, 44.0));

                    if ui.add(btn).clicked() {
                        self.set_length_type(ltype);
                    }
                    ui.add_space(4.0);
                }
            });

            ui.add_space(8.0);

            // Fingering notes selector
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("FINGERING:")
                        .size(12.0)
                        .color(accent_cyan)
                        .strong(),
                );
                ui.add_space(6.0);

                let fingerings = [
                    (ShakuhachiFingeringNote::Ro, "Ro (呂)"),
                    (ShakuhachiFingeringNote::Tsu, "Tsu (ツ)"),
                    (ShakuhachiFingeringNote::Re, "Re (レ)"),
                    (ShakuhachiFingeringNote::Chi, "Chi (チ)"),
                    (ShakuhachiFingeringNote::Ri, "Ri (リ)"),
                ];

                for (fn_note, label) in fingerings {
                    let is_active = self.fingering_note == fn_note;
                    let btn_bg = if is_active {
                        accent_green
                    } else {
                        Color32::from_rgb(26, 36, 52)
                    };
                    let btn_fg = if is_active {
                        Color32::BLACK
                    } else {
                        text_white
                    };

                    let btn = egui::Button::new(
                        egui::RichText::new(label).size(12.0).color(btn_fg).strong(),
                    )
                    .fill(btn_bg)
                    .min_size(egui::vec2(72.0, 44.0));

                    if ui.add(btn).clicked() {
                        self.set_fingering(fn_note);
                    }
                    ui.add_space(4.0);
                }

                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new("MERI/KARI:")
                        .size(12.0)
                        .color(accent_amber)
                        .strong(),
                );
                let meri_slider = egui::Slider::new(&mut self.meri_kari_cents, -300.0..=200.0)
                    .suffix(" ct")
                    .show_value(true);
                if ui.add(meri_slider).changed() {
                    self.update_shakuhachi_simulation();
                }

                ui.add_space(12.0);
                ui.label(egui::RichText::new("CHIFF:").size(12.0).color(text_dim));
                let chiff_slider =
                    egui::Slider::new(&mut self.chiff_noise_level, 0.0..=1.0).show_value(true);
                if ui.add(chiff_slider).changed() {
                    self.update_shakuhachi_simulation();
                }
            });
        });
    }

    /// Render ASCII visual preview of the Shakuhachi HUD.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let w = width.max(40);
        let h = height.max(10);

        let border = format!("+{}+", "-".repeat(w - 2));
        lines.push(border.clone());

        let title = format!(" SHAKUHACHI HUD: {} | {} ", self.length_type.flute_name(), self.fingering_note.note_name());
        let title_padded = if title.len() < w - 2 {
            format!("|{}{}|", title, " ".repeat(w - 2 - title.len()))
        } else {
            format!("|{}|", &title[..w - 2])
        };
        lines.push(title_padded);

        let info = format!(
            "| Jet: {:.1} m/s | Angle: {:.1} deg | Meri/Kari: {:+.0} ct | Chiff: {:.2} |",
            self.jet_velocity_mps, self.utaguchi_angle_deg, self.meri_kari_cents, self.chiff_noise_level
        );
        let info_padded = if info.len() < w - 2 {
            format!("{}{}|", info, " ".repeat(w - 1 - info.len()))
        } else {
            format!("|{}|", &info[1..w - 1])
        };
        lines.push(info_padded);

        lines.push(format!("|{}|", "-".repeat(w - 2)));

        // Middle body with standing wave bars
        let body_height = h.saturating_sub(6);
        for row in 0..body_height {
            let threshold = 1.0 - (row as f32 / body_height.max(1) as f32);
            let mut bar_line = String::from("| ");
            for &amp in &self.modal_amplitudes {
                if amp >= threshold {
                    bar_line.push_str(" [####] ");
                } else {
                    bar_line.push_str(" [....] ");
                }
            }
            if bar_line.len() < w - 1 {
                bar_line.push_str(&" ".repeat(w - 1 - bar_line.len()));
            }
            bar_line.push('|');
            lines.push(bar_line);
        }

        let footer = "| M1(Fund)  M2(Oct)   M3(12th)  M4(2Oct)  M5(17th)  M6(19th)  M7(Chiff) M8(Noise)|";
        let footer_padded = if footer.len() < w - 2 {
            format!("{}{}|", footer, " ".repeat(w - 1 - footer.len()))
        } else {
            format!("|{}|", &footer[1..w - 1])
        };
        lines.push(footer_padded);
        lines.push(border);

        lines
    }

    /// Render headless PNG snapshot representation of the Shakuhachi HUD.
    pub fn render_snapshot_png(&self, path: &str, width: usize, height: usize) -> Result<(), String> {
        let mut pixels = vec![0u8; width * height * 4];

        // Background: #0A0E18 (slate navy)
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) * 4;
                pixels[idx] = 0x0A;
                pixels[idx + 1] = 0x0E;
                pixels[idx + 2] = 0x18;
                pixels[idx + 3] = 0xFF;
            }
        }

        let margin = 16;
        let c_y = 50;
        let c_h = height.saturating_sub(130);
        let c_w = (width - margin * 3) / 2;

        // Left canvas: Embouchure Utaguchi puck space (#121824)
        let c_x1 = margin;
        for y in c_y..(c_y + c_h) {
            for x in c_x1..(c_x1 + c_w) {
                if x < width && y < height {
                    let idx = (y * width + x) * 4;
                    pixels[idx] = 0x12;
                    pixels[idx + 1] = 0x18;
                    pixels[idx + 2] = 0x24;
                    pixels[idx + 3] = 0xFF;
                }
            }
        }

        // Draw Left Canvas Puck
        let px = c_x1 + (self.puck_pos.0 * (c_w as f32 * 0.85)) as usize + (c_w / 10);
        let py = (c_y + c_h) - (self.puck_pos.1 * (c_h as f32 * 0.85)) as usize - (c_h / 10);
        let r = 14;

        for dy in -(r as isize)..=(r as isize) {
            for dx in -(r as isize)..=(r as isize) {
                if dx * dx + dy * dy <= (r * r) as isize {
                    let gx = (px as isize + dx) as usize;
                    let gy = (py as isize + dy) as usize;
                    if gx < width && gy < height {
                        let idx = (gy * width + gx) * 4;
                        // Amber Gold #FFB432
                        pixels[idx] = 0xFF;
                        pixels[idx + 1] = 0xB4;
                        pixels[idx + 2] = 0x32;
                        pixels[idx + 3] = 0xFF;
                    }
                }
            }
        }

        // Right canvas: Bore Harmonic Spectrum (#121824)
        let c_x2 = c_x1 + c_w + margin;
        for y in c_y..(c_y + c_h) {
            for x in c_x2..(c_x2 + c_w) {
                if x < width && y < height {
                    let idx = (y * width + x) * 4;
                    pixels[idx] = 0x12;
                    pixels[idx + 1] = 0x18;
                    pixels[idx + 2] = 0x24;
                    pixels[idx + 3] = 0xFF;
                }
            }
        }

        // Draw Harmonic Bars
        let bar_spacing = c_w / 9;
        let bar_width = bar_spacing * 3 / 4;
        for (i, &amp) in self.modal_amplitudes.iter().enumerate() {
            let bx = c_x2 + 10 + i * bar_spacing;
            let bar_h = (amp * (c_h as f32 * 0.80)) as usize;
            let by_start = (c_y + c_h).saturating_sub(bar_h + 10);
            let by_end = c_y + c_h - 10;

            for y in by_start..by_end {
                for x in bx..(bx + bar_width) {
                    if x < width && y < height {
                        let idx = (y * width + x) * 4;
                        if i == 0 {
                            // Mint #00FFB4
                            pixels[idx] = 0x00;
                            pixels[idx + 1] = 0xFF;
                            pixels[idx + 2] = 0xB4;
                        } else if (i + 1) % 2 == 1 {
                            // Cyan #00E5FF
                            pixels[idx] = 0x00;
                            pixels[idx + 1] = 0xE5;
                            pixels[idx + 2] = 0xFF;
                        } else {
                            // Gold #FFB432
                            pixels[idx] = 0xFF;
                            pixels[idx + 1] = 0xB4;
                            pixels[idx + 2] = 0x32;
                        }
                        pixels[idx + 3] = 0xFF;
                    }
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
    fn test_shakuhachi_view_hit_target_dimensions() {
        const {
            assert!(
                SHAKUHACHI_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT,
                "Shakuhachi puck hit target bounding box must be >= 44pt"
            );
        }
    }

    #[test]
    fn test_shakuhachi_view_ascii_render() {
        let view = ShakuhachiView::new();
        let ascii = view.render_ascii(80, 16);
        assert_eq!(ascii.len(), 16);
        assert!(ascii[0].starts_with('+'));
    }

    #[test]
    fn test_shakuhachi_view_snapshot_render() {
        let view = ShakuhachiView::new();
        let res = view.render_snapshot_png("scratch/renders/shakuhachi_view.png", 800, 520);
        assert!(res.is_ok());
    }
}
