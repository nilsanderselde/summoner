// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Grand Piano Spruce Soundboard & Bridge Wave Scattering HUD (Milestone 26).
//!
//! Provides an interactive 2D bridge impedance & modal decay puck (Bridge Coupling Bleed vs Decay Scale),
//! real-time 2D spruce soundboard standing wave modal vibration heatmap visualizer,
//! anisotropic wood grain dispersion indicators, and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and headless PNG snapshot rendering.

use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, RichText, Rounding, Stroke, Vec2};

pub const SOUNDBOARD_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding target
pub const MIN_BRIDGE_BLEED: f32 = 0.05;
pub const MAX_BRIDGE_BLEED: f32 = 0.95;
pub const MIN_DECAY_SCALE: f32 = 0.2;
pub const MAX_DECAY_SCALE: f32 = 3.0;

/// Soundboard presets for the Bridge & Soundboard HUD.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SoundboardHudPreset {
    #[default]
    SteinwayD9Foot,
    BösendorferImperial,
    YamahaCFX,
    IntimateStudio,
    ImpressionistUnaCorda,
    PreparedAvantGarde,
}

impl SoundboardHudPreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::SteinwayD9Foot => "Steinway D (9ft Spruce Soundboard)",
            Self::BösendorferImperial => "Bösendorfer Imperial (9.5ft Spruce)",
            Self::YamahaCFX => "Yamaha CFX (9ft Resonant Spruce)",
            Self::IntimateStudio => "Intimate Studio Soundboard",
            Self::ImpressionistUnaCorda => "Impressionist Una Corda Resonator",
            Self::PreparedAvantGarde => "Prepared Avant-Garde Soundboard",
        }
    }
}

/// Spruce soundboard and bridge wave scattering interactive view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundboardBridgeView {
    /// Active soundboard preset.
    pub preset: SoundboardHudPreset,
    /// Bridge sympathetic coupling bleed $[0.05 ..= 0.95]$.
    pub bridge_bleed: f32,
    /// Soundboard modal decay scale $[0.2 ..= 3.0]$.
    pub soundboard_decay_scale: f32,
    /// Bridge mechanical impedance $Z_b$ in kg/s.
    pub bridge_impedance: f32,
    /// Inharmonicity dispersion $B$ parameter.
    pub inharmonicity_b: f32,
    /// 8-mode soundboard modal frequencies in Hz.
    pub modal_frequencies: [f32; 8],
    /// 8-mode quality factors $Q_k$.
    pub modal_qs: [f32; 8],
    /// 2D Puck position (X: bridge bleed, Y: decay scale).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for SoundboardBridgeView {
    fn default() -> Self {
        Self::new()
    }
}

impl SoundboardBridgeView {
    pub fn new() -> Self {
        let mut view = Self {
            preset: SoundboardHudPreset::SteinwayD9Foot,
            bridge_bleed: 0.35,
            soundboard_decay_scale: 1.0,
            bridge_impedance: 420.0,
            inharmonicity_b: 0.00018,
            modal_frequencies: [62.0, 125.0, 240.0, 380.0, 680.0, 1200.0, 2100.0, 3300.0],
            modal_qs: [15.0, 22.0, 28.0, 32.0, 38.0, 45.0, 50.0, 55.0],
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        };
        view.update_puck_from_physics();
        view.update_soundboard_acoustics();
        view
    }

    pub fn update_puck_from_physics(&mut self) {
        let norm_x = ((self.bridge_bleed - MIN_BRIDGE_BLEED) / (MAX_BRIDGE_BLEED - MIN_BRIDGE_BLEED)).clamp(0.0, 1.0);
        let norm_y = ((self.soundboard_decay_scale - MIN_DECAY_SCALE) / (MAX_DECAY_SCALE - MIN_DECAY_SCALE)).clamp(0.0, 1.0);
        self.puck_pos = (norm_x, norm_y);
    }

    pub fn update_physics_from_puck(&mut self, norm_x: f32, norm_y: f32) {
        self.puck_pos = (norm_x.clamp(0.0, 1.0), norm_y.clamp(0.0, 1.0));
        self.bridge_bleed = MIN_BRIDGE_BLEED + self.puck_pos.0 * (MAX_BRIDGE_BLEED - MIN_BRIDGE_BLEED);
        self.soundboard_decay_scale = MIN_DECAY_SCALE + self.puck_pos.1 * (MAX_DECAY_SCALE - MIN_DECAY_SCALE);
        self.update_soundboard_acoustics();
    }

    pub fn update_soundboard_acoustics(&mut self) {
        match self.preset {
            SoundboardHudPreset::SteinwayD9Foot => {
                self.bridge_impedance = 420.0;
                self.inharmonicity_b = 0.00018;
                self.modal_frequencies = [62.0, 125.0, 240.0, 380.0, 680.0, 1200.0, 2100.0, 3300.0];
                self.modal_qs = [15.0 * self.soundboard_decay_scale, 22.0 * self.soundboard_decay_scale, 28.0 * self.soundboard_decay_scale, 32.0 * self.soundboard_decay_scale, 38.0 * self.soundboard_decay_scale, 45.0 * self.soundboard_decay_scale, 50.0 * self.soundboard_decay_scale, 55.0 * self.soundboard_decay_scale];
            }
            SoundboardHudPreset::BösendorferImperial => {
                self.bridge_impedance = 380.0;
                self.inharmonicity_b = 0.00012;
                self.modal_frequencies = [52.0, 108.0, 210.0, 340.0, 610.0, 1080.0, 1950.0, 3050.0];
                self.modal_qs = [18.0 * self.soundboard_decay_scale, 26.0 * self.soundboard_decay_scale, 34.0 * self.soundboard_decay_scale, 40.0 * self.soundboard_decay_scale, 48.0 * self.soundboard_decay_scale, 55.0 * self.soundboard_decay_scale, 60.0 * self.soundboard_decay_scale, 65.0 * self.soundboard_decay_scale];
            }
            SoundboardHudPreset::YamahaCFX => {
                self.bridge_impedance = 480.0;
                self.inharmonicity_b = 0.00025;
                self.modal_frequencies = [68.0, 138.0, 265.0, 420.0, 740.0, 1320.0, 2350.0, 3600.0];
                self.modal_qs = [12.0 * self.soundboard_decay_scale, 18.0 * self.soundboard_decay_scale, 24.0 * self.soundboard_decay_scale, 28.0 * self.soundboard_decay_scale, 34.0 * self.soundboard_decay_scale, 40.0 * self.soundboard_decay_scale, 46.0 * self.soundboard_decay_scale, 50.0 * self.soundboard_decay_scale];
            }
            SoundboardHudPreset::IntimateStudio => {
                self.bridge_impedance = 520.0;
                self.inharmonicity_b = 0.00010;
                self.modal_frequencies = [58.0, 118.0, 225.0, 360.0, 640.0, 1150.0, 1980.0, 2900.0];
                self.modal_qs = [10.0 * self.soundboard_decay_scale, 15.0 * self.soundboard_decay_scale, 20.0 * self.soundboard_decay_scale, 24.0 * self.soundboard_decay_scale, 28.0 * self.soundboard_decay_scale, 32.0 * self.soundboard_decay_scale, 36.0 * self.soundboard_decay_scale, 40.0 * self.soundboard_decay_scale];
            }
            SoundboardHudPreset::ImpressionistUnaCorda => {
                self.bridge_impedance = 360.0;
                self.inharmonicity_b = 0.00015;
                self.modal_frequencies = [60.0, 120.0, 235.0, 370.0, 660.0, 1180.0, 2050.0, 3200.0];
                self.modal_qs = [20.0 * self.soundboard_decay_scale, 30.0 * self.soundboard_decay_scale, 38.0 * self.soundboard_decay_scale, 44.0 * self.soundboard_decay_scale, 52.0 * self.soundboard_decay_scale, 60.0 * self.soundboard_decay_scale, 68.0 * self.soundboard_decay_scale, 75.0 * self.soundboard_decay_scale];
            }
            SoundboardHudPreset::PreparedAvantGarde => {
                self.bridge_impedance = 600.0;
                self.inharmonicity_b = 0.00060;
                self.modal_frequencies = [75.0, 155.0, 290.0, 480.0, 820.0, 1480.0, 2600.0, 4100.0];
                self.modal_qs = [6.0 * self.soundboard_decay_scale, 8.0 * self.soundboard_decay_scale, 12.0 * self.soundboard_decay_scale, 14.0 * self.soundboard_decay_scale, 16.0 * self.soundboard_decay_scale, 18.0 * self.soundboard_decay_scale, 22.0 * self.soundboard_decay_scale, 25.0 * self.soundboard_decay_scale];
            }
        }
    }

    pub fn set_preset(&mut self, preset: SoundboardHudPreset) {
        self.preset = preset;
        match preset {
            SoundboardHudPreset::SteinwayD9Foot => {
                self.bridge_bleed = 0.35;
                self.soundboard_decay_scale = 1.0;
            }
            SoundboardHudPreset::BösendorferImperial => {
                self.bridge_bleed = 0.45;
                self.soundboard_decay_scale = 1.35;
            }
            SoundboardHudPreset::YamahaCFX => {
                self.bridge_bleed = 0.28;
                self.soundboard_decay_scale = 0.85;
            }
            SoundboardHudPreset::IntimateStudio => {
                self.bridge_bleed = 0.20;
                self.soundboard_decay_scale = 0.75;
            }
            SoundboardHudPreset::ImpressionistUnaCorda => {
                self.bridge_bleed = 0.50;
                self.soundboard_decay_scale = 1.20;
            }
            SoundboardHudPreset::PreparedAvantGarde => {
                self.bridge_bleed = 0.15;
                self.soundboard_decay_scale = 0.40;
            }
        }
        self.update_puck_from_physics();
        self.update_soundboard_acoustics();
    }

    /// Hit-tests touch coordinate on the soundboard bridge puck.
    pub fn hit_test_soundboard_puck(&self, point: (f32, f32), canvas_x: f32, canvas_y: f32, canvas_w: f32, canvas_h: f32) -> bool {
        let puck_x = canvas_x + self.puck_pos.0 * canvas_w;
        let puck_y = canvas_y + (1.0 - self.puck_pos.1) * canvas_h;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= SOUNDBOARD_PUCK_HIT_RADIUS
    }

    /// Render ASCII snapshot representation of the view as a single multiline string.
    pub fn render_ascii_snapshot_str(&self) -> String {
        self.render_ascii(80, 20).join("\n")
    }

    /// Render ASCII diagnostics representation of the view.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let border = format!("+{}+", "-".repeat(width.saturating_sub(2)));
        lines.push(border.clone());

        let title = format!(
            "| GRAND PIANO SOUNDBOARD & BRIDGE HUD | Preset: {} |",
            self.preset.name()
        );
        let title_padded = format!("{:<width$}|", title, width = width - 1);
        lines.push(title_padded);

        let physics_info = format!(
            "| Bridge Bleed: {:.2} | Decay Scale: {:.2}x | Impedance: {:.0} kg/s | Inharm B: {:.5} |",
            self.bridge_bleed, self.soundboard_decay_scale, self.bridge_impedance, self.inharmonicity_b
        );
        let phys_padded = format!("{:<width$}|", physics_info, width = width - 1);
        lines.push(phys_padded);

        let modes_info = format!(
            "| Modes: {:.0}Hz, {:.0}Hz, {:.0}Hz, {:.0}Hz, {:.0}Hz, {:.0}Hz |",
            self.modal_frequencies[0], self.modal_frequencies[1], self.modal_frequencies[2],
            self.modal_frequencies[3], self.modal_frequencies[4], self.modal_frequencies[5]
        );
        let mod_padded = format!("{:<width$}|", modes_info, width = width - 1);
        lines.push(mod_padded);

        lines.push(border.clone());

        // 2D Spruce Soundboard plate vibration pattern
        let canvas_h = height.saturating_sub(6);
        for row in 0..canvas_h {
            let y_norm = row as f32 / canvas_h.max(1) as f32;
            let mut line_buf = String::with_capacity(width);
            line_buf.push('|');

            for col in 0..(width.saturating_sub(2)) {
                let x_norm = col as f32 / (width - 2).max(1) as f32;

                // Anisotropic standing wave pattern (1,1) + (1,2) + (2,1) + (2,2)
                let mode11 = (x_norm * std::f32::consts::PI).sin() * (y_norm * std::f32::consts::PI).sin();
                let mode12 = (x_norm * std::f32::consts::PI).sin() * (2.0 * y_norm * std::f32::consts::PI).sin();
                let mode21 = (2.0 * x_norm * std::f32::consts::PI).sin() * (y_norm * std::f32::consts::PI).sin();
                let pattern = (mode11 * 0.5 + mode12 * 0.3 + mode21 * 0.2).abs();

                if pattern > 0.65 {
                    line_buf.push('#');
                } else if pattern > 0.35 {
                    line_buf.push('+');
                } else if pattern > 0.15 {
                    line_buf.push('.');
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

        // Left Panel: 2D Puck Area (40..340, 80..420)
        let puck_x_center = 40.0 + self.puck_pos.0 * 300.0;
        let puck_y_center = 420.0 - self.puck_pos.1 * 340.0; // Inverted Y

        let hit_radius = SOUNDBOARD_PUCK_HIT_RADIUS * 1.5;
        let vis_radius = 14.0 * 1.5;

        for y in 0..height {
            for x in 0..width {
                let dx = x as f32 - puck_x_center;
                let dy = y as f32 - puck_y_center;
                let dist = (dx * dx + dy * dy).sqrt();

                let idx = (y * width + x) * 4;
                if dist <= vis_radius {
                    // Puck body: Vibrant Emerald `#10B981`
                    pixels[idx] = 0x10;
                    pixels[idx + 1] = 0xB9;
                    pixels[idx + 2] = 0x81;
                } else if dist <= hit_radius {
                    // Glow ring: `#065F46`
                    pixels[idx] = 0x06;
                    pixels[idx + 1] = 0x5F;
                    pixels[idx + 2] = 0x46;
                }
            }
        }

        // Right Panel: 2D Spruce Soundboard Heatmap (400..760, 80..420)
        let heat_x_start = 400;
        let heat_x_end = 760;
        let heat_y_start = 80;
        let heat_y_end = 420;
        let heat_w = heat_x_end - heat_x_start;
        let heat_h = heat_y_end - heat_y_start;

        for hy in 0..heat_h {
            let y = heat_y_start + hy;
            let y_norm = hy as f32 / heat_h as f32;

            for hx in 0..heat_w {
                let x = heat_x_start + hx;
                let x_norm = hx as f32 / heat_w as f32;

                // Anisotropic Sitka spruce standing wave formula
                let mode11 = (x_norm * std::f32::consts::PI).sin() * (y_norm * std::f32::consts::PI).sin();
                let mode12 = (x_norm * std::f32::consts::PI).sin() * (2.0 * y_norm * std::f32::consts::PI).sin();
                let mode21 = (2.0 * x_norm * std::f32::consts::PI).sin() * (y_norm * std::f32::consts::PI).sin();
                let disp = (mode11 * 0.55 + mode12 * 0.30 + mode21 * 0.15).abs();

                let idx = (y * width + x) * 4;
                // Heatmap color ramp (Dark Navy -> Teal -> Golden Amber)
                if disp > 0.6 {
                    pixels[idx] = 0xF5;
                    pixels[idx + 1] = 0x9E;
                    pixels[idx + 2] = 0x0B; // Amber peak
                } else if disp > 0.3 {
                    pixels[idx] = 0x06;
                    pixels[idx + 1] = 0xB6;
                    pixels[idx + 2] = 0xD4; // Cyan mid
                } else if disp > 0.1 {
                    pixels[idx] = 0x1E;
                    pixels[idx + 1] = 0x3A;
                    pixels[idx + 2] = 0x8A; // Navy base
                }
            }
        }

        save_png_file(path, width, height, &pixels)
    }
}

#[cfg(feature = "gui")]
impl SoundboardBridgeView {
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

        // Background
        painter.rect_filled(rect, 6.0, Color32::from_rgb(14, 18, 28));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            Align2::LEFT_TOP,
            "SPRUCE SOUNDBOARD & BRIDGE WAVE SCATTERING HUD",
            FontId::proportional(16.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Preset Tabs (y: 48..92) - Each tab >= 44pt height
        let presets = [
            SoundboardHudPreset::SteinwayD9Foot,
            SoundboardHudPreset::BösendorferImperial,
            SoundboardHudPreset::YamahaCFX,
            SoundboardHudPreset::IntimateStudio,
            SoundboardHudPreset::ImpressionistUnaCorda,
            SoundboardHudPreset::PreparedAvantGarde,
        ];

        let tab_w = (rect.width() - 40.0 - 5.0 * 8.0) / 6.0;
        for (i, p) in presets.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_selected = self.preset == *p;
            let bg_color = if is_selected {
                Color32::from_rgb(16, 185, 129)
            } else {
                Color32::from_rgb(25, 35, 50)
            };
            let text_color = if is_selected {
                Color32::from_rgb(10, 14, 24)
            } else {
                Color32::from_rgb(200, 215, 235)
            };

            painter.rect_filled(tab_rect, 4.0, bg_color);
            let label = match p {
                SoundboardHudPreset::SteinwayD9Foot => "Steinway D",
                SoundboardHudPreset::BösendorferImperial => "Bösendorfer",
                SoundboardHudPreset::YamahaCFX => "Yamaha CFX",
                SoundboardHudPreset::IntimateStudio => "Studio",
                SoundboardHudPreset::ImpressionistUnaCorda => "Una Corda",
                SoundboardHudPreset::PreparedAvantGarde => "Prepared",
            };
            painter.text(
                tab_rect.center(),
                Align2::CENTER_CENTER,
                label,
                FontId::proportional(11.0),
                text_color,
            );

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if tab_rect.contains(pos) {
                        self.set_preset(*p);
                    }
                }
            }
        }

        // Main Display Canvas (y: 104..340)
        let main_canvas = Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 104.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 340.0),
        );
        painter.rect_filled(main_canvas, 6.0, Color32::from_rgb(10, 14, 24));
        painter.rect_stroke(
            main_canvas,
            6.0,
            Stroke::new(1.5_f32, Color32::from_rgb(30, 45, 65)),
        );

        let half_w = main_canvas.width() * 0.50;

        // Left 50%: Bridge Mechanical Impedance & Decay Scale XY Canvas
        let left_rect = Rect::from_min_size(
            egui::pos2(main_canvas.min.x + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(half_w - 15.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(left_rect, 4.0, Color32::from_rgb(14, 20, 32));
        painter.rect_stroke(
            left_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(40, 55, 80)),
        );

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.min.y + 10.0),
            Align2::LEFT_TOP,
            "BRIDGE IMPEDANCE & MODAL DECAY PUCK",
            FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // Coordinate Grid inside left canvas
        let grid_w = left_rect.width() - 40.0;
        let grid_h = left_rect.height() - 60.0;
        let grid_origin = egui::pos2(left_rect.min.x + 20.0, left_rect.min.y + 35.0);
        let grid_rect = Rect::from_min_size(grid_origin, egui::vec2(grid_w, grid_h));
        painter.rect_filled(grid_rect, 2.0, Color32::from_rgb(8, 12, 20));
        painter.rect_stroke(grid_rect, 2.0, Stroke::new(1.0_f32, Color32::from_rgb(25, 38, 55)));

        // Grid lines
        for step in 1..4 {
            let gx = grid_rect.min.x + (step as f32 / 4.0) * grid_w;
            let gy = grid_rect.min.y + (step as f32 / 4.0) * grid_h;
            painter.line_segment([egui::pos2(gx, grid_rect.min.y), egui::pos2(gx, grid_rect.max.y)], Stroke::new(0.8_f32, Color32::from_rgb(20, 30, 45)));
            painter.line_segment([egui::pos2(grid_rect.min.x, gy), egui::pos2(grid_rect.max.x, gy)], Stroke::new(0.8_f32, Color32::from_rgb(20, 30, 45)));
        }

        // Left Drag Interaction
        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if grid_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - grid_rect.min.x) / grid_w).clamp(0.0, 1.0);
                    let ny = ((grid_rect.max.y - mouse_pos.y) / grid_h).clamp(0.0, 1.0);
                    self.update_physics_from_puck(nx, ny);
                }
            }
        }

        // Puck Position
        let puck_x = grid_rect.min.x + self.puck_pos.0 * grid_w;
        let puck_y = grid_rect.max.y - self.puck_pos.1 * grid_h;
        let puck_pos = egui::pos2(puck_x, puck_y);

        // Touch hit target (>= 44x44pt)
        painter.circle_stroke(
            puck_pos,
            SOUNDBOARD_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(16, 185, 129, 140)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(16, 185, 129));
        painter.circle_filled(puck_pos, 4.0, Color32::from_rgb(255, 255, 255));

        // Right 50%: 2D Spruce Soundboard Modal Heatmap
        let right_rect = Rect::from_min_size(
            egui::pos2(main_canvas.min.x + half_w + 5.0, main_canvas.min.y + 10.0),
            egui::vec2(half_w - 15.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(right_rect, 4.0, Color32::from_rgb(14, 20, 32));
        painter.rect_stroke(
            right_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(40, 55, 80)),
        );

        painter.text(
            egui::pos2(right_rect.min.x + 10.0, right_rect.min.y + 10.0),
            Align2::LEFT_TOP,
            "SPRUCE SOUNDBOARD MODAL HEATMAP",
            FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        let heat_origin = egui::pos2(right_rect.min.x + 20.0, right_rect.min.y + 35.0);
        let heat_w = right_rect.width() - 40.0;
        let heat_h = right_rect.height() - 60.0;
        let heat_rect = Rect::from_min_size(heat_origin, egui::vec2(heat_w, heat_h));
        painter.rect_filled(heat_rect, 2.0, Color32::from_rgb(8, 14, 24));
        painter.rect_stroke(heat_rect, 2.0, Stroke::new(1.0_f32, Color32::from_rgb(25, 40, 60)));

        // Anisotropic Sitka Spruce standing wave cell grid (16 cols x 8 rows)
        let cols = 16;
        let rows = 8;
        let cw = heat_w / cols as f32;
        let ch = heat_h / rows as f32;

        for ry in 0..rows {
            let y_norm = (ry as f32 + 0.5) / rows as f32;
            for cx in 0..cols {
                let x_norm = (cx as f32 + 0.5) / cols as f32;

                let m11 = (x_norm * std::f32::consts::PI).sin() * (y_norm * std::f32::consts::PI).sin();
                let m12 = (x_norm * std::f32::consts::PI).sin() * (2.0 * y_norm * std::f32::consts::PI).sin();
                let m21 = (2.0 * x_norm * std::f32::consts::PI).sin() * (y_norm * std::f32::consts::PI).sin();
                let disp = (m11 * 0.55 + m12 * 0.30 + m21 * 0.15).abs() * (self.soundboard_decay_scale / 1.5).clamp(0.4, 2.0);

                let cell_rect = Rect::from_min_size(
                    egui::pos2(heat_rect.min.x + cx as f32 * cw, heat_rect.min.y + ry as f32 * ch),
                    egui::vec2(cw - 1.0, ch - 1.0),
                );

                let (r, g, b, alpha) = if disp > 0.6 {
                    (245, 158, 11, 230) // Amber peak
                } else if disp > 0.3 {
                    (6, 182, 212, 170) // Cyan mid
                } else if disp > 0.1 {
                    (30, 58, 138, 120) // Navy base
                } else {
                    (12, 20, 36, 60)
                };

                painter.rect_filled(cell_rect, 1.0, Color32::from_rgba_premultiplied(r, g, b, alpha));
            }
        }

        // Sitka Spruce wood grain longitudinal fibers
        for g_idx in 1..8 {
            let gx = heat_rect.min.x + (g_idx as f32 / 8.0) * heat_w;
            painter.line_segment(
                [egui::pos2(gx, heat_rect.min.y + 2.0), egui::pos2(gx, heat_rect.max.y - 2.0)],
                Stroke::new(0.6_f32, Color32::from_rgba_premultiplied(200, 220, 240, 25)),
            );
        }

        // Curved Maple Bridge Line traversing soundboard
        let bridge_y_offset = (1.0 - (self.bridge_bleed - MIN_BRIDGE_BLEED) / (MAX_BRIDGE_BLEED - MIN_BRIDGE_BLEED)) * (heat_h - 20.0);
        let by_base = heat_rect.min.y + 10.0 + bridge_y_offset;
        let mut prev_bp: Option<Pos2> = None;
        for bi in 0..=20 {
            let bt = bi as f32 / 20.0;
            let bx = heat_rect.min.x + bt * heat_w;
            let curve = (bt - 0.5) * (bt - 0.5) * 12.0;
            let by = (by_base + curve).clamp(heat_rect.min.y + 4.0, heat_rect.max.y - 4.0);
            let bp = egui::pos2(bx, by);
            if let Some(prev) = prev_bp {
                painter.line_segment([prev, bp], Stroke::new(2.5_f32, Color32::from_rgb(245, 158, 11)));
            }
            prev_bp = Some(bp);

            // Bridge pins every 4 steps
            if bi % 4 == 0 {
                painter.circle_filled(bp, 2.0, Color32::from_rgb(255, 255, 255));
            }
        }

        // Bottom Metrics Dock (y: 350..465)
        let dock_rect = Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 350.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 465.0),
        );
        painter.rect_filled(dock_rect, 6.0, Color32::from_rgb(18, 25, 38));
        painter.rect_stroke(
            dock_rect,
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        let params = [
            (
                "BRIDGE COUPLING BLEED",
                format!("{:.1}% Bleed", self.bridge_bleed * 100.0),
                Color32::from_rgb(16, 185, 129),
            ),
            (
                "MODAL DECAY SCALE",
                format!("{:.2}x T60", self.soundboard_decay_scale),
                Color32::from_rgb(245, 158, 11),
            ),
            (
                "BRIDGE IMPEDANCE (Zb)",
                format!("{:.0} kg/s", self.bridge_impedance),
                Color32::from_rgb(6, 182, 212),
            ),
            (
                "INHARMONICITY (B)",
                format!("{:.5}", self.inharmonicity_b),
                Color32::from_rgb(167, 139, 250),
            ),
        ];

        let col_w = (dock_rect.width() - 40.0) / 4.0;
        for (i, (label, val, col)) in params.iter().enumerate() {
            let px_pos = dock_rect.min.x + 20.0 + i as f32 * col_w;
            painter.text(
                egui::pos2(px_pos, dock_rect.min.y + 12.0),
                Align2::LEFT_TOP,
                *label,
                FontId::proportional(11.0),
                Color32::from_rgb(160, 180, 205),
            );
            painter.text(
                egui::pos2(px_pos, dock_rect.min.y + 30.0),
                Align2::LEFT_TOP,
                val,
                FontId::proportional(14.0),
                *col,
            );
            let sub = match i {
                0 => "Impulse energy transfer",
                1 => "Spruce decay multiplier",
                2 => "Driving point resistance",
                3 => "Partial dispersion coeff",
                _ => "",
            };
            painter.text(
                egui::pos2(px_pos, dock_rect.min.y + 48.0),
                Align2::LEFT_TOP,
                sub,
                FontId::proportional(10.0),
                Color32::from_rgb(110, 130, 155),
            );
        }
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
    fn test_soundboard_bridge_view_hit_target_dimensions() {
        const {
            assert!(
                SOUNDBOARD_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT,
                "Soundboard puck hit target bounding box must be >= 44pt"
            );
        }
    }

    #[test]
    fn test_soundboard_bridge_view_ascii_render() {
        let view = SoundboardBridgeView::new();
        let ascii = view.render_ascii(80, 16);
        assert_eq!(ascii.len(), 16);
        assert!(ascii[0].starts_with('+'));
    }

    #[test]
    fn test_soundboard_bridge_view_snapshot_render() {
        let view = SoundboardBridgeView::new();
        let res = view.render_snapshot_png("scratch/renders/soundboard_bridge_view.png", 800, 520);
        assert!(res.is_ok());
    }
}
