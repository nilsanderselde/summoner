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

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, FontId, RichText, Stroke, Vec2};

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

    pub fn hit_test_sitar_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= SITAR_PUCK_HIT_RADIUS
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
    pub fn show(&mut self, ui: &mut egui::Ui) {
        self.ui(ui);
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let bg_color = Color32::from_rgb(14, 18, 28);
        let card_bg = Color32::from_rgb(20, 26, 40);
        let border_color = Color32::from_rgb(45, 60, 85);
        let accent_amber = Color32::from_rgb(245, 158, 11);
        let accent_cyan = Color32::from_rgb(6, 182, 212);
        let text_white = Color32::from_rgb(240, 245, 255);

        egui::Frame::none().fill(bg_color).show(ui, |ui| {
            ui.set_min_size(egui::vec2(760.0, 480.0));
            ui.add_space(6.0);

            // Title and Header
            ui.horizontal(|ui| {
                ui.add_space(12.0);
                ui.heading(
                    RichText::new("SITAR — CURVED JAWARI BRIDGE & MEEND PITCH DEFLECTION")
                        .size(16.0)
                        .color(text_white)
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(12.0);
                    ui.label(
                        RichText::new(&self.active_raga_name)
                            .size(12.0)
                            .color(accent_amber)
                            .strong(),
                    );
                });
            });

            ui.add_space(6.0);

            // Preset selector tabs (>= 44pt touch hit targets)
            ui.horizontal(|ui| {
                ui.add_space(12.0);
                let presets = [
                    (SitarHudPreset::RagaYamanAlap, "Raga Yaman"),
                    (SitarHudPreset::VilayatKhanGayaki, "Gayaki Vocal"),
                    (SitarHudPreset::RaviShankarKharaj, "Kharaj Pancham"),
                    (SitarHudPreset::SurbaharDeepBass, "Surbahar Bass"),
                    (SitarHudPreset::ElectricSitarJhajhar, "Electric Sitar"),
                ];

                for (pst, label) in presets {
                    let is_active = self.preset == pst;
                    let btn_bg = if is_active {
                        accent_amber
                    } else {
                        Color32::from_rgb(32, 44, 66)
                    };
                    let btn_fg = if is_active {
                        Color32::BLACK
                    } else {
                        text_white
                    };

                    let btn = egui::Button::new(
                        RichText::new(label).size(12.0).color(btn_fg).strong(),
                    )
                    .fill(btn_bg)
                    .min_size(egui::vec2(130.0, 44.0));

                    if ui.add(btn).clicked() {
                        self.set_preset(pst);
                    }
                    ui.add_space(4.0);
                }
            });

            ui.add_space(8.0);

            // Main interactive 2D Canvas split: Left XY Pad (Meend vs Strike), Right Jawari Waveform
            let (canvas_rect, response) = ui.allocate_exact_size(
                egui::vec2(740.0, 230.0),
                egui::Sense::click_and_drag(),
            );

            let painter = ui.painter_at(canvas_rect);
            painter.rect_filled(canvas_rect, 6.0, card_bg);
            painter.rect_stroke(canvas_rect, 6.0, Stroke::new(1.5_f32, border_color));

            let left_w = canvas_rect.width() * 0.52;
            let left_rect = egui::Rect::from_min_size(
                canvas_rect.min,
                egui::vec2(left_w, canvas_rect.height()),
            );
            let right_rect = egui::Rect::from_min_size(
                egui::pos2(canvas_rect.min.x + left_w, canvas_rect.min.y),
                egui::vec2(canvas_rect.width() - left_w, canvas_rect.height()),
            );

            // Left pad: Grid & Crosshairs
            painter.line_segment(
                [
                    egui::pos2(left_rect.min.x + 12.0, left_rect.center().y),
                    egui::pos2(left_rect.max.x - 12.0, left_rect.center().y),
                ],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(50, 75, 110, 100)),
            );
            painter.line_segment(
                [
                    egui::pos2(left_rect.center().x, left_rect.min.y + 12.0),
                    egui::pos2(left_rect.center().x, left_rect.max.y - 12.0),
                ],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(50, 75, 110, 100)),
            );

            painter.text(
                egui::pos2(left_rect.min.x + 16.0, left_rect.min.y + 14.0),
                egui::Align2::LEFT_TOP,
                "MEEND PULL (0..5 st) vs MIZRAB STRIKE (0.05..1.00)",
                FontId::proportional(11.0),
                accent_amber,
            );

            // Handle Dragging Puck
            let layout_rect = Rect::new(
                left_rect.min.x + 16.0,
                left_rect.min.y + 36.0,
                left_rect.width() - 32.0,
                left_rect.height() - 52.0,
            );

            if response.drag_started() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if self.hit_test_sitar_puck((pos.x, pos.y), layout_rect) {
                        self.is_dragging_puck = true;
                    }
                }
            }

            if response.drag_stopped() {
                self.is_dragging_puck = false;
            }

            if self.is_dragging_puck {
                if let Some(pos) = response.interact_pointer_pos() {
                    let norm_x = ((pos.x - layout_rect.x) / layout_rect.width).clamp(0.0, 1.0);
                    let norm_y = (1.0 - ((pos.y - layout_rect.y) / layout_rect.height)).clamp(0.0, 1.0);
                    self.update_physics_from_puck(norm_x, norm_y);
                }
            }

            // Draw Draggable Puck (44x44pt bounding touch hit target)
            let puck_x = layout_rect.x + self.puck_pos.0 * layout_rect.width;
            let puck_y = layout_rect.y + (1.0 - self.puck_pos.1) * layout_rect.height;
            let puck_center = egui::pos2(puck_x, puck_y);

            painter.circle_stroke(
                puck_center,
                SITAR_PUCK_HIT_RADIUS,
                Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(245, 158, 11, 140)),
            );
            painter.circle_filled(puck_center, 12.0, accent_amber);
            painter.circle_filled(puck_center, 4.0, Color32::WHITE);

            // Right side: Curved Jawari obstacle contact & string wave display
            painter.line_segment(
                [
                    egui::pos2(right_rect.min.x, right_rect.min.y + 8.0),
                    egui::pos2(right_rect.min.x, right_rect.max.y - 8.0),
                ],
                Stroke::new(1.0_f32, border_color),
            );

            painter.text(
                egui::pos2(right_rect.min.x + 16.0, right_rect.min.y + 14.0),
                egui::Align2::LEFT_TOP,
                "JAWARI BUZZ BRIDGE BOUNDARY & CONTACT CLIPPING",
                FontId::proportional(11.0),
                accent_cyan,
            );

            // Plot Curved Jawari wave
            let plot_w = right_rect.width() - 32.0;
            let plot_h = right_rect.height() - 56.0;
            let plot_x0 = right_rect.min.x + 16.0;
            let plot_y0 = right_rect.min.y + 38.0;

            // Obstacle line (parabolic profile)
            let obstacle_bound = -0.3 + (self.jawari_gap_mm - 0.18) * 0.5;
            let obst_y = plot_y0 + (1.0 - (obstacle_bound + 1.0) * 0.5) * plot_h;
            painter.line_segment(
                [egui::pos2(plot_x0, obst_y), egui::pos2(plot_x0 + plot_w, obst_y)],
                Stroke::new(1.5_f32, Color32::from_rgb(239, 68, 68)),
            );
            painter.text(
                egui::pos2(plot_x0 + plot_w - 4.0, obst_y - 2.0),
                egui::Align2::RIGHT_BOTTOM,
                format!("Jawari gap: {:.2}mm", self.jawari_gap_mm),
                FontId::proportional(9.0),
                Color32::from_rgb(239, 68, 68),
            );

            let steps = 48;
            let mut prev_pt = None;
            for s in 0..=steps {
                let col_t = s as f32 / steps as f32;
                let fundamental = (col_t * 6.0 * std::f32::consts::PI * (1.0 + self.meend_pull_semitones * 0.15)).sin();
                let harmonic2 = 0.5 * (col_t * 12.0 * std::f32::consts::PI).sin();
                let raw_wave = (fundamental * 0.7 + harmonic2 * 0.3) * self.strike_velocity;
                let clipped_wave = raw_wave.max(obstacle_bound);
                let norm_wave = (clipped_wave + 1.0) * 0.5;

                let wx = plot_x0 + col_t * plot_w;
                let wy = plot_y0 + (1.0 - norm_wave) * plot_h;
                let pt = egui::pos2(wx, wy);

                if let Some(prev) = prev_pt {
                    painter.line_segment([prev, pt], Stroke::new(2.0_f32, accent_cyan));
                }
                prev_pt = Some(pt);
            }

            ui.add_space(8.0);

            // Bottom controls: Sliders and Toggles
            ui.horizontal(|ui| {
                ui.add_space(12.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new("Acoustic Bridge Parameters:").strong());
                    ui.add(egui::Slider::new(&mut self.jawari_gap_mm, 0.01..=1.5).text("Jawari Gap (mm)"));
                    ui.add(egui::Slider::new(&mut self.jiva_thread_pos, 0.0..=1.0).text("Jiva Cotton Pos"));
                });

                ui.separator();

                ui.vertical(|ui| {
                    ui.label(RichText::new("Sympathetic & Drone:").strong());
                    ui.add(egui::Slider::new(&mut self.tarab_bleed, 0.0..=1.0).text("Tarab Bleed"));
                    ui.checkbox(&mut self.chikari_active, "Chikari Drone Active");
                });

                ui.separator();

                ui.vertical(|ui| {
                    ui.label(RichText::new("Live Readouts:").strong());
                    ui.label(format!("Meend Pull: +{:.2} semitones", self.meend_pull_semitones));
                    ui.label(format!("Strike Velocity: {:.2}", self.strike_velocity));
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
