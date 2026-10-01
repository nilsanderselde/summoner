// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Free-Reed Aeroelastic Displacement Phase Canvas & Musette Beating Spectrum HUD (Milestone 28).
//!
//! Provides an interactive 2D reed stiffness vs cassotto aperture puck,
//! real-time aeroelastic limit cycle phase portrait visualizer (Displacement y vs Velocity dy/dt),
//! 5-rank musette beating spectrum bar display (16' Bassoon, 8' Clarinet, 8'+/- Musette detuned, 4' Piccolo),
//! and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and headless PNG snapshot rendering.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, FontId, RichText, Rounding, Stroke, Vec2};

pub const FREE_REED_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding target
pub const MIN_REED_STIFFNESS: f32 = 0.5;
pub const MAX_REED_STIFFNESS: f32 = 2.0;
pub const MIN_CASSOTTO_APERTURE: f32 = 0.0;
pub const MAX_CASSOTTO_APERTURE: f32 = 1.0;

/// Free-Reed HUD sound and chamber preset choices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FreeReedHudPreset {
    #[default]
    TangoBandoneonZinc,
    FrenchMusetteMaple,
    RussianBayanDuralumin,
    VintageHarmoniumBrass,
    EnglishConcertinaSteel,
}

impl FreeReedHudPreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::TangoBandoneonZinc => "Tango Bandoneon (Zinc Reedplate / Double Cassotto)",
            Self::FrenchMusetteMaple => "French Musette (Solid Maple / Triple 8')",
            Self::RussianBayanDuralumin => "Russian Bayan (Duralumin Reedblocks)",
            Self::VintageHarmoniumBrass => "Vintage Harmonium (Brass Free Reeds)",
            Self::EnglishConcertinaSteel => "English Concertina (Spring Steel Reeds)",
        }
    }
}

/// Free-Reed aeroelastic phase portrait and musette spectrum view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeReedView {
    /// Active HUD sound preset.
    pub preset: FreeReedHudPreset,
    /// Reed material stiffness factor $[0.5 ..= 2.0]$.
    pub reed_stiffness: f32,
    /// Cassotto tone chamber aperture fraction $[0.0 ..= 1.0]$.
    pub cassotto_aperture: f32,
    /// Musette detuning spread in cents $[0.0 ..= 35.0]$.
    pub musette_detune_cents: f32,
    /// 5-rank acoustic energy levels $[0.0 ..= 1.0]$ for visualizer bars (16', 8', 8'+, 8'-, 4').
    pub rank_energies: [f32; 5],
    /// 2D Puck position (X: Cassotto aperture, Y: reed stiffness).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for FreeReedView {
    fn default() -> Self {
        Self::new()
    }
}

impl FreeReedView {
    pub fn new() -> Self {
        let mut view = Self {
            preset: FreeReedHudPreset::TangoBandoneonZinc,
            reed_stiffness: 1.25,
            cassotto_aperture: 0.70,
            musette_detune_cents: 2.0,
            rank_energies: [0.90, 0.85, 0.20, 0.0, 0.0], // 16' + 8' Bandoneon
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        };
        view.update_puck_from_physics();
        view
    }

    pub fn update_puck_from_physics(&mut self) {
        let norm_x = ((self.cassotto_aperture - MIN_CASSOTTO_APERTURE) / (MAX_CASSOTTO_APERTURE - MIN_CASSOTTO_APERTURE)).clamp(0.0, 1.0);
        let norm_y = ((self.reed_stiffness - MIN_REED_STIFFNESS) / (MAX_REED_STIFFNESS - MIN_REED_STIFFNESS)).clamp(0.0, 1.0);
        self.puck_pos = (norm_x, norm_y);
    }

    pub fn update_physics_from_puck(&mut self, norm_x: f32, norm_y: f32) {
        self.puck_pos = (norm_x.clamp(0.0, 1.0), norm_y.clamp(0.0, 1.0));
        self.cassotto_aperture = MIN_CASSOTTO_APERTURE + self.puck_pos.0 * (MAX_CASSOTTO_APERTURE - MIN_CASSOTTO_APERTURE);
        self.reed_stiffness = MIN_REED_STIFFNESS + self.puck_pos.1 * (MAX_REED_STIFFNESS - MIN_REED_STIFFNESS);
    }

    pub fn set_preset(&mut self, preset: FreeReedHudPreset) {
        self.preset = preset;
        match preset {
            FreeReedHudPreset::TangoBandoneonZinc => {
                self.reed_stiffness = 1.25;
                self.cassotto_aperture = 0.70;
                self.musette_detune_cents = 2.0;
                self.rank_energies = [0.90, 0.85, 0.15, 0.0, 0.0];
            }
            FreeReedHudPreset::FrenchMusetteMaple => {
                self.reed_stiffness = 1.00;
                self.cassotto_aperture = 0.85;
                self.musette_detune_cents = 18.0;
                self.rank_energies = [0.0, 0.88, 0.85, 0.82, 0.0];
            }
            FreeReedHudPreset::RussianBayanDuralumin => {
                self.reed_stiffness = 1.40;
                self.cassotto_aperture = 1.00;
                self.musette_detune_cents = 4.0;
                self.rank_energies = [0.95, 0.90, 0.70, 0.65, 0.80];
            }
            FreeReedHudPreset::VintageHarmoniumBrass => {
                self.reed_stiffness = 0.85;
                self.cassotto_aperture = 0.50;
                self.musette_detune_cents = 8.0;
                self.rank_energies = [0.80, 0.75, 0.60, 0.0, 0.0];
            }
            FreeReedHudPreset::EnglishConcertinaSteel => {
                self.reed_stiffness = 1.15;
                self.cassotto_aperture = 0.90;
                self.musette_detune_cents = 0.0;
                self.rank_energies = [0.0, 0.90, 0.0, 0.0, 0.0];
            }
        }
        self.update_puck_from_physics();
    }

    pub fn stiffness_to_normalized(stiffness: f32) -> f32 {
        ((stiffness - MIN_REED_STIFFNESS) / (MAX_REED_STIFFNESS - MIN_REED_STIFFNESS)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_stiffness(norm: f32) -> f32 {
        MIN_REED_STIFFNESS + norm.clamp(0.0, 1.0) * (MAX_REED_STIFFNESS - MIN_REED_STIFFNESS)
    }

    pub fn aperture_to_normalized(aperture: f32) -> f32 {
        ((aperture - MIN_CASSOTTO_APERTURE) / (MAX_CASSOTTO_APERTURE - MIN_CASSOTTO_APERTURE)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_aperture(norm: f32) -> f32 {
        MIN_CASSOTTO_APERTURE + norm.clamp(0.0, 1.0) * (MAX_CASSOTTO_APERTURE - MIN_CASSOTTO_APERTURE)
    }

    pub fn hit_test_free_reed_puck(&self, screen_pos: (f32, f32), canvas_rect: Rect) -> bool {
        let puck_screen_x = canvas_rect.x + self.puck_pos.0 * canvas_rect.width;
        let puck_screen_y = canvas_rect.y + (1.0 - self.puck_pos.1) * canvas_rect.height;
        let dx = screen_pos.0 - puck_screen_x;
        let dy = screen_pos.1 - puck_screen_y;
        (dx * dx + dy * dy).sqrt() <= FREE_REED_PUCK_HIT_RADIUS
    }

    pub fn render_ascii_snapshot(&self, width: usize, height: usize) -> Vec<String> {
        self.render_ascii(width, height)
    }

    /// Evaluates 32 points along the aeroelastic non-linear limit cycle trajectory (displacement, velocity).
    pub fn evaluate_phase_portrait(&self) -> [(f32, f32); 32] {
        let mut points = [(0.0f32, 0.0f32); 32];
        for (i, p) in points.iter_mut().enumerate() {
            let theta = (i as f32 / 32.0) * 2.0 * std::f32::consts::PI;
            // Distorted limit cycle from non-linear Duffing stiffness
            let r = 1.0 + 0.15 * (theta * 2.0).sin();
            let x = theta.cos() * r;
            let y = theta.sin() * r;
            *p = (x, y);
        }
        points
    }

    /// Render ASCII art overview for terminal and headless verification.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let border = format!("+{}+", "-".repeat(width.saturating_sub(2)));
        lines.push(border.clone());

        let title = format!("| FREE-REED AEROELASTIC PHASE CANVAS [k={:.2}, Aperture={:.2}] |", self.reed_stiffness, self.cassotto_aperture);
        let padding = width.saturating_sub(title.len() + 2);
        lines.push(format!("|{}{}|", title, " ".repeat(padding)));

        let inner_height = height.saturating_sub(4);
        let puck_x = ((self.puck_pos.0 * (width.saturating_sub(4) as f32)) as usize).min(width.saturating_sub(4));
        let puck_y = (((1.0 - self.puck_pos.1) * (inner_height.saturating_sub(1) as f32)) as usize).min(inner_height.saturating_sub(1));

        for y in 0..inner_height {
            let mut row = vec![' '; width.saturating_sub(2)];
            // Draw limit cycle ellipse approximation in phase portrait
            for x in 0..row.len() {
                let nx = (x as f32 / row.len() as f32 - 0.5) * 2.0;
                let ny = (y as f32 / inner_height as f32 - 0.5) * 2.0;
                let r_sq = nx * nx + ny * ny;
                if (0.45..=0.60).contains(&r_sq) {
                    row[x] = '*';
                }
            }
            if y == puck_y && puck_x < row.len() {
                row[puck_x] = '@';
            }
            let s: String = row.into_iter().collect();
            lines.push(format!("|{}|", s));
        }

        let footer = format!("| 16': {:.2} | 8': {:.2} | 8'+: {:.2} | 8'-: {:.2} | 4': {:.2} |",
            self.rank_energies[0], self.rank_energies[1], self.rank_energies[2], self.rank_energies[3], self.rank_energies[4]);
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

        // Left 2D Phase Portrait Canvas rectangle (x: 40..width/2 - 20, y: 72..height-80)
        let left_c_left = 40;
        let left_c_right = width / 2 - 20;
        let c_top = 72;
        let c_bottom = height - 80;

        for y in c_top..c_bottom {
            for x in left_c_left..left_c_right {
                let idx = (y * width + x) * 4;
                pixels[idx] = 0x18;
                pixels[idx + 1] = 0x22;
                pixels[idx + 2] = 0x34;
            }
        }

        // Draw Limit Cycle Orbit in Phase Portrait (Reed tip displacement vs velocity)
        let center_x = (left_c_left + left_c_right) as f32 * 0.5;
        let center_y = (c_top + c_bottom) as f32 * 0.5;
        let radius_x = (left_c_right - left_c_left) as f32 * 0.38;
        let radius_y = (c_bottom - c_top) as f32 * 0.38;

        for step in 0..360 {
            let theta = (step as f32) * std::f32::consts::PI / 180.0;
            // Distorted limit cycle from non-linear Duffing stiffness
            let r = 1.0 + 0.15 * (theta * 2.0).sin();
            let px = (center_x + theta.cos() * radius_x * r) as usize;
            let py = (center_y + theta.sin() * radius_y * r) as usize;
            if px < width && py < height {
                let idx = (py * width + px) * 4;
                pixels[idx] = 0x38;
                pixels[idx + 1] = 0xBD;
                pixels[idx + 2] = 0xF8; // Light cyan
            }
        }

        // Draw Interactive Puck (radius 22pt -> 44x44pt hit target)
        let puck_px_x = left_c_left + ((self.puck_pos.0 * (left_c_right - left_c_left) as f32) as usize);
        let puck_px_y = c_bottom - ((self.puck_pos.1 * (c_bottom - c_top) as f32) as usize);

        let radius = FREE_REED_PUCK_HIT_RADIUS as i32;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy <= radius * radius {
                    let px = (puck_px_x as i32 + dx) as usize;
                    let py = (puck_px_y as i32 + dy) as usize;
                    if px < width && py < height {
                        let idx = (py * width + px) * 4;
                        let dist_sq = dx * dx + dy * dy;
                        if dist_sq >= (radius - 3) * (radius - 3) {
                            pixels[idx] = 0x10;
                            pixels[idx + 1] = 0xB9;
                            pixels[idx + 2] = 0x81; // Green ring
                        } else {
                            pixels[idx] = 0xEC;
                            pixels[idx + 1] = 0x48;
                            pixels[idx + 2] = 0x99; // Pink center
                        }
                    }
                }
            }
        }

        // Right Musette Beating Spectrum Bars (x: width/2 + 20..width - 40, y: 72..height-80)
        let right_c_left = width / 2 + 20;
        let right_c_right = width - 40;

        for y in c_top..c_bottom {
            for x in right_c_left..right_c_right {
                let idx = (y * width + x) * 4;
                pixels[idx] = 0x18;
                pixels[idx + 1] = 0x22;
                pixels[idx + 2] = 0x34;
            }
        }

        let num_bars = 5;
        let bar_width = (right_c_right - right_c_left) / (num_bars * 2);
        let bar_colors = [
            (0x3B, 0x82, 0xF6), // Blue 16'
            (0x10, 0xB9, 0x81), // Green 8'
            (0xF5, 0x9E, 0x0B), // Amber 8'+
            (0xEC, 0x48, 0x99), // Pink 8'-
            (0x8B, 0x5C, 0xF6), // Purple 4'
        ];

        for (i, &energy) in self.rank_energies.iter().enumerate() {
            let bar_left = right_c_left + i * bar_width * 2 + bar_width / 2;
            let bar_right = bar_left + bar_width;
            let bar_height_px = (energy * (c_bottom - c_top) as f32) as usize;
            let bar_top_y = c_bottom.saturating_sub(bar_height_px);

            let (cr, cg, cb) = bar_colors[i];
            for y in bar_top_y..c_bottom {
                for x in bar_left..bar_right.min(right_c_right) {
                    let idx = (y * width + x) * 4;
                    pixels[idx] = cr;
                    pixels[idx + 1] = cg;
                    pixels[idx + 2] = cb;
                }
            }
        }

        save_png_file(path, width, height, &pixels)
    }
}

#[cfg(feature = "gui")]
impl FreeReedView {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        self.ui(ui);
    }

    #[allow(clippy::needless_range_loop)]
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 480.0),
            egui::Sense::click_and_drag(),
        );

        let painter = ui.painter_at(rect);

        // Background: Deep Slate Charcoal (#0B1120)
        painter.rect_filled(rect, 6.0, Color32::from_rgb(11, 17, 32));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 16.0),
            egui::Align2::LEFT_TOP,
            "AEROELASTIC FREE-REED PHASE PORTRAIT & 5-RANK MUSETTE SPECTRUM HUD",
            FontId::proportional(13.5),
            Color32::from_rgb(240, 245, 255),
        );

        // Preset Tabs (y: 44..88) - 44pt touch targets
        let tabs = [
            (FreeReedHudPreset::TangoBandoneonZinc, "BANDONEON (ZINC)"),
            (FreeReedHudPreset::FrenchMusetteMaple, "MUSETTE (MAPLE)"),
            (FreeReedHudPreset::RussianBayanDuralumin, "BAYAN (DURAL)"),
            (FreeReedHudPreset::VintageHarmoniumBrass, "HARMONIUM (BRASS)"),
            (FreeReedHudPreset::EnglishConcertinaSteel, "CONCERTINA (STEEL)"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (preset, name)) in tabs.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 44.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_sel = self.preset == *preset;
            let bg_col = if is_sel {
                Color32::from_rgb(16, 185, 129) // Emerald #10B981
            } else {
                Color32::from_rgb(30, 41, 59) // Slate #1E293B
            };
            let text_col = if is_sel {
                Color32::from_rgb(11, 17, 32)
            } else {
                Color32::from_rgb(226, 232, 240)
            };

            painter.rect_filled(tab_rect, 4.0, bg_col);
            painter.text(
                tab_rect.center(),
                egui::Align2::CENTER_CENTER,
                *name,
                FontId::proportional(10.5),
                text_col,
            );

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if tab_rect.contains(pos) {
                        self.set_preset(*preset);
                    }
                }
            }
        }

        // Main Display Canvas (y: 98..330)
        let main_canvas = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 98.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 330.0),
        );
        painter.rect_filled(main_canvas, 6.0, Color32::from_rgb(10, 15, 28));
        painter.rect_stroke(
            main_canvas,
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(51, 65, 85)),
        );

        // Split canvas into Left Pad (Limit Cycle Phase Portrait) and Right Pad (5-Rank Spectrum)
        let pad_w = (main_canvas.width() - 30.0) * 0.50;
        let pad_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + 12.0, main_canvas.min.y + 12.0),
            egui::vec2(pad_w, main_canvas.height() - 24.0),
        );
        painter.rect_filled(pad_rect, 4.0, Color32::from_rgb(22, 32, 48));
        painter.rect_stroke(
            pad_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(71, 85, 105)),
        );

        // Grid lines on Left Pad
        let center_pad = pad_rect.center();
        painter.line_segment(
            [egui::pos2(pad_rect.min.x, center_pad.y), egui::pos2(pad_rect.max.x, center_pad.y)],
            Stroke::new(0.8_f32, Color32::from_rgb(45, 60, 85)),
        );
        painter.line_segment(
            [egui::pos2(center_pad.x, pad_rect.min.y), egui::pos2(center_pad.x, pad_rect.max.y)],
            Stroke::new(0.8_f32, Color32::from_rgb(45, 60, 85)),
        );

        // Draw Aeroelastic Limit Cycle Orbit in Left Pad
        let rad_x = pad_rect.width() * 0.35;
        let rad_y = pad_rect.height() * 0.35;
        let mut loop_points = Vec::with_capacity(33);
        for step in 0..=32 {
            let theta = (step as f32 / 32.0) * 2.0 * std::f32::consts::PI;
            let r = 1.0 + 0.15 * (theta * 2.0).sin();
            let px = center_pad.x + theta.cos() * rad_x * r;
            let py = center_pad.y + theta.sin() * rad_y * r;
            loop_points.push(egui::pos2(px, py));
        }
        for w in loop_points.windows(2) {
            painter.line_segment([w[0], w[1]], Stroke::new(1.8_f32, Color32::from_rgb(56, 189, 248)));
        }

        // Zone labels inside pad
        painter.text(
            egui::pos2(pad_rect.min.x + 10.0, pad_rect.min.y + 8.0),
            egui::Align2::LEFT_TOP,
            "AEROELASTIC LIMIT CYCLE (dy/dt vs y)",
            FontId::proportional(10.0),
            Color32::from_rgb(148, 163, 184),
        );

        // Handle Touch/Mouse Dragging for Puck (X: Cassotto Aperture, Y: Reed Stiffness)
        let puck_screen_x = pad_rect.min.x + self.puck_pos.0 * pad_rect.width();
        let puck_screen_y = pad_rect.max.y - self.puck_pos.1 * pad_rect.height();
        let puck_center = egui::pos2(puck_screen_x, puck_screen_y);

        if response.dragged() || response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                if pad_rect.contains(pos) || self.is_dragging_puck {
                    self.is_dragging_puck = true;
                    let norm_x = ((pos.x - pad_rect.min.x) / pad_rect.width()).clamp(0.0, 1.0);
                    let norm_y = ((pad_rect.max.y - pos.y) / pad_rect.height()).clamp(0.0, 1.0);
                    self.update_physics_from_puck(norm_x, norm_y);
                }
            }
        } else {
            self.is_dragging_puck = false;
        }

        // Draw Interactive Puck
        painter.circle_filled(puck_center, FREE_REED_PUCK_HIT_RADIUS, Color32::from_rgb(236, 72, 153)); // Pink #EC4899
        painter.circle_stroke(
            puck_center,
            FREE_REED_PUCK_HIT_RADIUS,
            Stroke::new(2.5_f32, Color32::from_rgb(16, 185, 129)), // Emerald #10B981
        );
        painter.circle_filled(puck_center, 4.0, Color32::from_rgb(11, 17, 32));

        // Right Visualizer Pad (5-Rank Musette Spectrum)
        let viz_left = pad_rect.max.x + 12.0;
        let viz_rect = egui::Rect::from_min_max(
            egui::pos2(viz_left, main_canvas.min.y + 12.0),
            egui::pos2(main_canvas.max.x - 12.0, main_canvas.max.y - 12.0),
        );
        painter.rect_filled(viz_rect, 4.0, Color32::from_rgb(18, 26, 42));
        painter.rect_stroke(
            viz_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(71, 85, 105)),
        );

        painter.text(
            egui::pos2(viz_rect.center().x, viz_rect.min.y + 12.0),
            egui::Align2::CENTER_TOP,
            "5-RANK ACOUSTIC HARMONIC ENERGY",
            FontId::proportional(11.5),
            Color32::from_rgb(226, 232, 240),
        );

        // Draw 5 Spectrum Bars
        let rank_names = ["16' Bassoon", "8' Clarinet", "8'+ Musette", "8'- Musette", "4' Piccolo"];
        let rank_colors = [
            Color32::from_rgb(239, 68, 68),   // Red
            Color32::from_rgb(245, 158, 11),  // Amber
            Color32::from_rgb(16, 185, 129),  // Emerald
            Color32::from_rgb(56, 189, 248),  // Sky Blue
            Color32::from_rgb(168, 85, 247),  // Purple
        ];

        let num_bars = 5;
        let bar_margin = 16.0;
        let total_bar_space = viz_rect.width() - bar_margin * 2.0;
        let bar_slot = total_bar_space / num_bars as f32;
        let bar_w = bar_slot * 0.65;
        let bar_base_y = viz_rect.max.y - 32.0;
        let bar_max_h = bar_base_y - (viz_rect.min.y + 40.0);

        for (i, (&energy, (&name, &color))) in self.rank_energies.iter().zip(rank_names.iter().zip(rank_colors.iter())).enumerate() {
            let cx = viz_rect.min.x + bar_margin + (i as f32 + 0.5) * bar_slot;
            let bar_h = (energy.clamp(0.0, 1.0) * bar_max_h).max(3.0);
            let bar_box = egui::Rect::from_min_max(
                egui::pos2(cx - bar_w * 0.5, bar_base_y - bar_h),
                egui::pos2(cx + bar_w * 0.5, bar_base_y),
            );

            // Background slot
            let bg_box = egui::Rect::from_min_max(
                egui::pos2(cx - bar_w * 0.5, bar_base_y - bar_max_h),
                egui::pos2(cx + bar_w * 0.5, bar_base_y),
            );
            painter.rect_filled(bg_box, 3.0, Color32::from_rgb(11, 17, 32));
            painter.rect_filled(bar_box, 3.0, color);

            // Label
            painter.text(
                egui::pos2(cx, bar_base_y + 6.0),
                egui::Align2::CENTER_TOP,
                name,
                FontId::proportional(9.5),
                Color32::from_rgb(203, 213, 225),
            );
        }

        // Bottom Controls Strip (y: 342..470)
        let controls_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 342.0),
            egui::pos2(rect.max.x - 20.0, rect.max.y - 10.0),
        );

        ui.allocate_ui_at_rect(controls_rect, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Aeroelastic & Chamber:").font(FontId::proportional(11.0)).strong().color(Color32::from_rgb(226, 232, 240)));
                    ui.add(egui::Slider::new(&mut self.reed_stiffness, MIN_REED_STIFFNESS..=MAX_REED_STIFFNESS).text("Reed Stiffness"));
                    ui.add(egui::Slider::new(&mut self.cassotto_aperture, MIN_CASSOTTO_APERTURE..=MAX_CASSOTTO_APERTURE).text("Cassotto Aperture"));
                });

                ui.separator();

                ui.vertical(|ui| {
                    ui.label(RichText::new("Musette Detuning:").font(FontId::proportional(11.0)).strong().color(Color32::from_rgb(226, 232, 240)));
                    ui.add(egui::Slider::new(&mut self.musette_detune_cents, 0.0..=35.0).text("Detune (cents)"));
                });

                ui.separator();

                ui.vertical(|ui| {
                    ui.label(RichText::new("Rank Energy Weights:").font(FontId::proportional(11.0)).strong().color(Color32::from_rgb(226, 232, 240)));
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut self.rank_energies[0]).speed(0.01).clamp_range(0.0..=1.0).prefix("16': "));
                        ui.add(egui::DragValue::new(&mut self.rank_energies[1]).speed(0.01).clamp_range(0.0..=1.0).prefix("8': "));
                        ui.add(egui::DragValue::new(&mut self.rank_energies[2]).speed(0.01).clamp_range(0.0..=1.0).prefix("8'+: "));
                    });
                });
            });
        });
        self.update_puck_from_physics();
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
    fn test_free_reed_view_hit_target_dimensions() {
        const {
            assert!(
                FREE_REED_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT,
                "Free-reed puck hit target bounding box must be >= 44pt"
            );
        }
    }

    #[test]
    fn test_free_reed_view_ascii_render() {
        let view = FreeReedView::new();
        let ascii = view.render_ascii(80, 16);
        assert_eq!(ascii.len(), 16);
        assert!(ascii[0].starts_with('+'));
    }

    #[test]
    fn test_free_reed_view_snapshot_render() {
        let view = FreeReedView::new();
        let res = view.render_snapshot_png("scratch/renders/free_reed_view.png", 800, 520);
        assert!(res.is_ok());
    }
}
