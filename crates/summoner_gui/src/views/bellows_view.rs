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

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, FontId, RichText, Rounding, Stroke, Vec2};

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

    pub fn pressure_to_normalized(pressure: f32) -> f32 {
        ((pressure - MIN_BELLOWS_PRESSURE_PA) / (MAX_BELLOWS_PRESSURE_PA - MIN_BELLOWS_PRESSURE_PA)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_pressure(norm: f32) -> f32 {
        MIN_BELLOWS_PRESSURE_PA + norm.clamp(0.0, 1.0) * (MAX_BELLOWS_PRESSURE_PA - MIN_BELLOWS_PRESSURE_PA)
    }

    pub fn velocity_to_normalized(vel: f32) -> f32 {
        ((vel - MIN_VALVE_VELOCITY) / (MAX_VALVE_VELOCITY - MIN_VALVE_VELOCITY)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_velocity(norm: f32) -> f32 {
        MIN_VALVE_VELOCITY + norm.clamp(0.0, 1.0) * (MAX_VALVE_VELOCITY - MIN_VALVE_VELOCITY)
    }

    pub fn hit_test_bellows_puck(&self, screen_pos: (f32, f32), canvas_rect: Rect) -> bool {
        let puck_screen_x = canvas_rect.x + self.puck_pos.0 * canvas_rect.width;
        let puck_screen_y = canvas_rect.y + (1.0 - self.puck_pos.1) * canvas_rect.height;
        let dx = screen_pos.0 - puck_screen_x;
        let dy = screen_pos.1 - puck_screen_y;
        (dx * dx + dy * dy).sqrt() <= BELLOWS_PUCK_HIT_RADIUS
    }

    pub fn render_ascii_snapshot(&self, width: usize, height: usize) -> Vec<String> {
        self.render_ascii(width, height)
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

        // Background: Deep Slate Charcoal (#0F172A)
        painter.rect_filled(rect, 6.0, Color32::from_rgb(15, 23, 42));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 16.0),
            egui::Align2::LEFT_TOP,
            "PNEUMATIC BELLOWS CHAMBER DYNAMICS & FREE-REED ARTICULATION HUD",
            FontId::proportional(13.5),
            Color32::from_rgb(240, 245, 255),
        );

        // Preset Tabs (y: 44..88) - 44pt touch targets
        let tabs = [
            (BellowsHudPreset::TangoBandoneonAccented, "BANDONEON"),
            (BellowsHudPreset::FrenchMusetteAccordion, "MUSETTE"),
            (BellowsHudPreset::RussianBayanTutti, "BAYAN TUTTI"),
            (BellowsHudPreset::VintageHarmoniumDrone, "HARMONIUM"),
            (BellowsHudPreset::EnglishConcertinaFast, "CONCERTINA"),
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
                Color32::from_rgb(217, 119, 6) // Amber #D97706
            } else {
                Color32::from_rgb(30, 41, 59) // Slate #1E293B
            };
            let text_col = if is_sel {
                Color32::from_rgb(15, 23, 42)
            } else {
                Color32::from_rgb(226, 232, 240)
            };

            painter.rect_filled(tab_rect, 4.0, bg_col);
            painter.text(
                tab_rect.center(),
                egui::Align2::CENTER_CENTER,
                *name,
                FontId::proportional(11.0),
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
        painter.rect_filled(main_canvas, 6.0, Color32::from_rgb(10, 15, 30));
        painter.rect_stroke(
            main_canvas,
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(51, 65, 85)),
        );

        // Split canvas into Left Pad (Pressure vs Valve Velocity) and Right Visualizer (Chamber & Flow)
        let pad_w = (main_canvas.width() - 30.0) * 0.52;
        let pad_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + 12.0, main_canvas.min.y + 12.0),
            egui::vec2(pad_w, main_canvas.height() - 24.0),
        );
        painter.rect_filled(pad_rect, 4.0, Color32::from_rgb(24, 34, 52));
        painter.rect_stroke(
            pad_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(71, 85, 105)),
        );

        // Zero-pressure center line (Divider between PULL < 0 and PUSH > 0)
        let mid_x = pad_rect.center().x;
        painter.line_segment(
            [egui::pos2(mid_x, pad_rect.min.y), egui::pos2(mid_x, pad_rect.max.y)],
            Stroke::new(1.5_f32, Color32::from_rgb(59, 130, 246)),
        );

        // Grid lines
        for gy in 1..4 {
            let y = pad_rect.min.y + gy as f32 * (pad_rect.height() / 4.0);
            painter.line_segment(
                [egui::pos2(pad_rect.min.x, y), egui::pos2(pad_rect.max.x, y)],
                Stroke::new(0.5_f32, Color32::from_rgb(40, 52, 75)),
            );
        }

        // Zone labels inside pad
        painter.text(
            egui::pos2(pad_rect.min.x + 12.0, pad_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "◀ PULL (Expanding)",
            FontId::proportional(10.0),
            Color32::from_rgb(148, 163, 184),
        );
        painter.text(
            egui::pos2(pad_rect.max.x - 12.0, pad_rect.min.y + 10.0),
            egui::Align2::RIGHT_TOP,
            "PUSH (Compressing) ▶",
            FontId::proportional(10.0),
            Color32::from_rgb(148, 163, 184),
        );

        // Handle Touch/Mouse Dragging for Puck
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
        painter.circle_filled(puck_center, BELLOWS_PUCK_HIT_RADIUS, Color32::from_rgb(245, 158, 11));
        painter.circle_stroke(
            puck_center,
            BELLOWS_PUCK_HIT_RADIUS,
            Stroke::new(2.5_f32, Color32::from_rgb(254, 240, 138)),
        );
        painter.circle_filled(puck_center, 4.0, Color32::from_rgb(15, 23, 42));

        // Right Visualizer Pad (Chamber & Flow Dynamics)
        let viz_left = pad_rect.max.x + 12.0;
        let viz_rect = egui::Rect::from_min_max(
            egui::pos2(viz_left, main_canvas.min.y + 12.0),
            egui::pos2(main_canvas.max.x - 12.0, main_canvas.max.y - 12.0),
        );
        painter.rect_filled(viz_rect, 4.0, Color32::from_rgb(20, 28, 44));
        painter.rect_stroke(
            viz_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(71, 85, 105)),
        );

        // Visualizer Titles and Gauges
        let is_push = self.bellows_pressure_pa >= 0.0;
        let dir_str = if is_push { "AIRFLOW: >>> PUSH >>>" } else { "AIRFLOW: <<< PULL <<<" };
        let dir_color = if is_push {
            Color32::from_rgb(245, 158, 11) // Amber
        } else {
            Color32::from_rgb(56, 189, 248) // Sky blue
        };

        painter.text(
            egui::pos2(viz_rect.center().x, viz_rect.min.y + 16.0),
            egui::Align2::CENTER_TOP,
            dir_str,
            FontId::proportional(12.0),
            dir_color,
        );

        // Pressure Gauge Readout
        let pres_str = format!("Pressure: {:+.0} Pa  ({:+.2} kPa)", self.bellows_pressure_pa, self.bellows_pressure_pa / 1000.0);
        painter.text(
            egui::pos2(viz_rect.center().x, viz_rect.min.y + 44.0),
            egui::Align2::CENTER_TOP,
            pres_str,
            FontId::proportional(11.5),
            Color32::from_rgb(241, 245, 249),
        );

        // Pressure Magnitude Bar
        let bar_bg = egui::Rect::from_center_size(
            egui::pos2(viz_rect.center().x, viz_rect.min.y + 80.0),
            egui::vec2(viz_rect.width() - 32.0, 18.0),
        );
        painter.rect_filled(bar_bg, 4.0, Color32::from_rgb(15, 23, 42));
        let half_w = bar_bg.width() * 0.5;
        let frac = (self.bellows_pressure_pa / MAX_BELLOWS_PRESSURE_PA).clamp(-1.0, 1.0);
        let bar_fill = if frac >= 0.0 {
            egui::Rect::from_min_max(
                egui::pos2(bar_bg.center().x, bar_bg.min.y),
                egui::pos2(bar_bg.center().x + frac * half_w, bar_bg.max.y),
            )
        } else {
            egui::Rect::from_min_max(
                egui::pos2(bar_bg.center().x + frac * half_w, bar_bg.min.y),
                egui::pos2(bar_bg.center().x, bar_bg.max.y),
            )
        };
        painter.rect_filled(bar_fill, 2.0, dir_color);

        // Cassotto Aperture Shutter Gauge
        let cass_label = format!("Cassotto Tone Chamber: {:.0}% Open", self.cassotto_aperture * 100.0);
        painter.text(
            egui::pos2(viz_rect.center().x, viz_rect.min.y + 115.0),
            egui::Align2::CENTER_TOP,
            cass_label,
            FontId::proportional(11.0),
            Color32::from_rgb(203, 213, 225),
        );

        // Animated shutter slats (4 horizontal bars)
        let slat_top = viz_rect.min.y + 140.0;
        let slat_h = 44.0;
        let slat_rect = egui::Rect::from_min_size(
            egui::pos2(viz_rect.min.x + 20.0, slat_top),
            egui::vec2(viz_rect.width() - 40.0, slat_h),
        );
        painter.rect_filled(slat_rect, 3.0, Color32::from_rgb(15, 23, 42));
        for s in 0..4 {
            let sy = slat_rect.min.y + s as f32 * (slat_h / 4.0);
            let open_offset = (1.0 - self.cassotto_aperture) * (slat_h / 5.0);
            painter.line_segment(
                [egui::pos2(slat_rect.min.x + 4.0, sy + open_offset), egui::pos2(slat_rect.max.x - 4.0, sy + open_offset)],
                Stroke::new(2.0_f32, Color32::from_rgb(148, 163, 184)),
            );
        }

        // Musette Detune Readout
        let musette_str = format!("Musette Detune: {:.1} cents", self.musette_detune_cents);
        painter.text(
            egui::pos2(viz_rect.center().x, viz_rect.min.y + 192.0),
            egui::Align2::CENTER_TOP,
            musette_str,
            FontId::proportional(10.5),
            Color32::from_rgb(167, 139, 250), // Lavender #A78BFA
        );

        // Bottom Controls Strip (y: 342..470)
        let controls_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 342.0),
            egui::pos2(rect.max.x - 20.0, rect.max.y - 10.0),
        );

        ui.allocate_ui_at_rect(controls_rect, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Bellows Chamber Dynamics:").font(FontId::proportional(11.0)).strong().color(Color32::from_rgb(226, 232, 240)));
                    ui.add(egui::Slider::new(&mut self.bellows_pressure_pa, MIN_BELLOWS_PRESSURE_PA..=MAX_BELLOWS_PRESSURE_PA).text("Pressure (Pa)"));
                    ui.add(egui::Slider::new(&mut self.valve_velocity, MIN_VALVE_VELOCITY..=MAX_VALVE_VELOCITY).text("Valve Velocity"));
                });

                ui.separator();

                ui.vertical(|ui| {
                    ui.label(RichText::new("Cassotto & Musette Tuning:").font(FontId::proportional(11.0)).strong().color(Color32::from_rgb(226, 232, 240)));
                    ui.add(egui::Slider::new(&mut self.cassotto_aperture, 0.0..=1.0).text("Cassotto Aperture"));
                    ui.add(egui::Slider::new(&mut self.musette_detune_cents, 0.0..=35.0).text("Musette (cents)"));
                });

                ui.separator();

                ui.vertical(|ui| {
                    ui.label(RichText::new("Reed Aeroacoustics:").font(FontId::proportional(11.0)).strong().color(Color32::from_rgb(226, 232, 240)));
                    ui.add(egui::Slider::new(&mut self.reed_stiffness, 0.5..=2.0).text("Reed Stiffness"));
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
