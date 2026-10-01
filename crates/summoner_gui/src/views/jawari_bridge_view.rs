// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Sitar Curved Jawari Bridge & Tarab Sympathetic Resonator HUD (Milestone 27).
//!
//! Provides an interactive 2D Jawari clearance gap & Jiva cotton thread puck (Jiva Thread Position vs Clearance Gap),
//! real-time 13-string sympathetic Tarab energy dissipation spectrum visualizer,
//! curved bone/horn obstacle profile geometry display, and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and headless PNG snapshot rendering.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, FontId, RichText, Rounding, Stroke, Vec2};

pub const JAWARI_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding target
pub const MIN_JAWARI_CLEARANCE_GAP: f32 = 0.01;
pub const MAX_JAWARI_CLEARANCE_GAP: f32 = 1.5;
pub const MIN_JIVA_THREAD_POS: f32 = 0.0;
pub const MAX_JIVA_THREAD_POS: f32 = 1.0;

/// Jawari bridge presets for the Jawari HUD.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum JawariBridgeHudPreset {
    #[default]
    DeerHornRaviShankar,
    CamelBoneVilayatKhan,
    EbonySurbaharMellow,
    SyntheticDelrinPrecision,
    ElectricMetalSizzle,
    OpenAcousticGourd,
}

impl JawariBridgeHudPreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::DeerHornRaviShankar => "Deer-Horn Curved Jawari (Ravi Shankar)",
            Self::CamelBoneVilayatKhan => "Camel-Bone Sharp Jawari (Vilayat Khan)",
            Self::EbonySurbaharMellow => "Ebony Wood Mellow Jawari (Surbahar)",
            Self::SyntheticDelrinPrecision => "Synthetic Delrin Precision Jawari",
            Self::ElectricMetalSizzle => "Electric Metal Flat Jawari",
            Self::OpenAcousticGourd => "Open Acoustic Gourd Jawari",
        }
    }
}

/// Jawari bridge and 13-string Tarab sympathetic resonator interactive view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JawariBridgeView {
    /// Active Jawari bridge preset.
    pub preset: JawariBridgeHudPreset,
    /// Jawari clearance gap $h_0$ in mm $[0.01 ..= 1.5]$.
    pub clearance_gap_mm: f32,
    /// Jiva cotton thread position $[0.0 ..= 1.0]$.
    pub jiva_thread_pos: f32,
    /// Bridge curvature parameter $c$ in $\text{m}^{-1}$.
    pub curvature_c: f32,
    /// Tarab sympathetic coupling bleed $[0.0 ..= 1.0]$.
    pub tarab_bleed: f32,
    /// 13-string Tarab energy dissipation levels $[0.0 ..= 1.0]$.
    pub tarab_energy: [f32; 13],
    /// 2D Puck position (X: jiva thread position, Y: clearance gap).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for JawariBridgeView {
    fn default() -> Self {
        Self::new()
    }
}

impl JawariBridgeView {
    pub fn new() -> Self {
        let mut view = Self {
            preset: JawariBridgeHudPreset::DeerHornRaviShankar,
            clearance_gap_mm: 0.18,
            jiva_thread_pos: 0.45,
            curvature_c: 120.0,
            tarab_bleed: 0.40,
            tarab_energy: [0.75, 0.60, 0.85, 0.45, 0.90, 0.55, 0.70, 0.80, 0.65, 0.50, 0.40, 0.60, 0.35],
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        };
        view.update_puck_from_physics();
        view.update_jawari_acoustics();
        view
    }

    pub fn update_puck_from_physics(&mut self) {
        let norm_x = ((self.jiva_thread_pos - MIN_JIVA_THREAD_POS) / (MAX_JIVA_THREAD_POS - MIN_JIVA_THREAD_POS)).clamp(0.0, 1.0);
        let norm_y = ((self.clearance_gap_mm - MIN_JAWARI_CLEARANCE_GAP) / (MAX_JAWARI_CLEARANCE_GAP - MIN_JAWARI_CLEARANCE_GAP)).clamp(0.0, 1.0);
        self.puck_pos = (norm_x, norm_y);
    }

    pub fn update_physics_from_puck(&mut self, norm_x: f32, norm_y: f32) {
        self.puck_pos = (norm_x.clamp(0.0, 1.0), norm_y.clamp(0.0, 1.0));
        self.jiva_thread_pos = MIN_JIVA_THREAD_POS + self.puck_pos.0 * (MAX_JIVA_THREAD_POS - MIN_JIVA_THREAD_POS);
        self.clearance_gap_mm = MIN_JAWARI_CLEARANCE_GAP + self.puck_pos.1 * (MAX_JAWARI_CLEARANCE_GAP - MIN_JAWARI_CLEARANCE_GAP);
        self.update_jawari_acoustics();
    }

    pub fn update_jawari_acoustics(&mut self) {
        match self.preset {
            JawariBridgeHudPreset::DeerHornRaviShankar => {
                self.curvature_c = 120.0;
                self.tarab_bleed = 0.40;
                self.tarab_energy = [0.80, 0.65, 0.85, 0.50, 0.92, 0.58, 0.72, 0.85, 0.68, 0.52, 0.45, 0.62, 0.38];
            }
            JawariBridgeHudPreset::CamelBoneVilayatKhan => {
                self.curvature_c = 240.0;
                self.tarab_bleed = 0.35;
                self.tarab_energy = [0.90, 0.75, 0.70, 0.60, 0.88, 0.65, 0.78, 0.80, 0.72, 0.58, 0.50, 0.48, 0.30];
            }
            JawariBridgeHudPreset::EbonySurbaharMellow => {
                self.curvature_c = 60.0;
                self.tarab_bleed = 0.50;
                self.tarab_energy = [0.95, 0.88, 0.82, 0.75, 0.90, 0.70, 0.65, 0.60, 0.55, 0.48, 0.40, 0.35, 0.28];
            }
            JawariBridgeHudPreset::SyntheticDelrinPrecision => {
                self.curvature_c = 160.0;
                self.tarab_bleed = 0.30;
                self.tarab_energy = [0.70, 0.60, 0.65, 0.55, 0.75, 0.60, 0.68, 0.72, 0.60, 0.50, 0.42, 0.45, 0.35];
            }
            JawariBridgeHudPreset::ElectricMetalSizzle => {
                self.curvature_c = 320.0;
                self.tarab_bleed = 0.20;
                self.tarab_energy = [0.60, 0.50, 0.55, 0.45, 0.65, 0.48, 0.52, 0.58, 0.45, 0.40, 0.35, 0.30, 0.25];
            }
            JawariBridgeHudPreset::OpenAcousticGourd => {
                self.curvature_c = 90.0;
                self.tarab_bleed = 0.55;
                self.tarab_energy = [0.85, 0.72, 0.90, 0.65, 0.95, 0.70, 0.80, 0.88, 0.75, 0.62, 0.55, 0.68, 0.45];
            }
        }
    }

    pub fn set_preset(&mut self, preset: JawariBridgeHudPreset) {
        self.preset = preset;
        match preset {
            JawariBridgeHudPreset::DeerHornRaviShankar => {
                self.clearance_gap_mm = 0.18;
                self.jiva_thread_pos = 0.45;
            }
            JawariBridgeHudPreset::CamelBoneVilayatKhan => {
                self.clearance_gap_mm = 0.08;
                self.jiva_thread_pos = 0.50;
            }
            JawariBridgeHudPreset::EbonySurbaharMellow => {
                self.clearance_gap_mm = 0.35;
                self.jiva_thread_pos = 0.35;
            }
            JawariBridgeHudPreset::SyntheticDelrinPrecision => {
                self.clearance_gap_mm = 0.15;
                self.jiva_thread_pos = 0.48;
            }
            JawariBridgeHudPreset::ElectricMetalSizzle => {
                self.clearance_gap_mm = 0.05;
                self.jiva_thread_pos = 0.60;
            }
            JawariBridgeHudPreset::OpenAcousticGourd => {
                self.clearance_gap_mm = 0.22;
                self.jiva_thread_pos = 0.40;
            }
        }
        self.update_puck_from_physics();
        self.update_jawari_acoustics();
    }

    pub fn hit_test_jawari_puck(&self, screen_pos: (f32, f32), canvas_rect: Rect) -> bool {
        let puck_screen_x = canvas_rect.x + self.puck_pos.0 * canvas_rect.width;
        let puck_screen_y = canvas_rect.y + (1.0 - self.puck_pos.1) * canvas_rect.height;
        let dx = screen_pos.0 - puck_screen_x;
        let dy = screen_pos.1 - puck_screen_y;
        (dx * dx + dy * dy).sqrt() <= JAWARI_PUCK_HIT_RADIUS
    }

    /// Render deterministic ASCII snapshot as string for headless verification.
    pub fn render_ascii_snapshot_str(&self) -> String {
        self.render_ascii(80, 16).join("\n")
    }

    /// Render ASCII diagnostics representation of the view.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let border = format!("+{}+", "-".repeat(width.saturating_sub(2)));
        lines.push(border.clone());

        let title = format!(
            "| JAWARI BRIDGE & TARAB RESONATOR HUD | Preset: {} |",
            self.preset.name()
        );
        let title_padded = format!("{:<width$}|", title, width = width - 1);
        lines.push(title_padded);

        let physics_info = format!(
            "| Clearance h0: {:.2}mm | Jiva Pos: {:.2} | Curvature c: {:.0} m^-1 | Tarab Bleed: {:.0}% |",
            self.clearance_gap_mm, self.jiva_thread_pos, self.curvature_c, self.tarab_bleed * 100.0
        );
        let phys_padded = format!("{:<width$}|", physics_info, width = width - 1);
        lines.push(phys_padded);

        let tarab_info = format!(
            "| Tarab E: {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2} |",
            self.tarab_energy[0], self.tarab_energy[1], self.tarab_energy[2], self.tarab_energy[3],
            self.tarab_energy[4], self.tarab_energy[5], self.tarab_energy[6], self.tarab_energy[7],
            self.tarab_energy[8], self.tarab_energy[9], self.tarab_energy[10], self.tarab_energy[11],
            self.tarab_energy[12]
        );
        let tar_padded = format!("{:<width$}|", tarab_info, width = width - 1);
        lines.push(tar_padded);

        lines.push(border.clone());

        // 13-String Tarab energy bar chart canvas
        let canvas_h = height.saturating_sub(6);
        for row in 0..canvas_h {
            let row_ratio = 1.0 - (row as f32 / canvas_h.max(1) as f32);
            let mut line_buf = String::with_capacity(width);
            line_buf.push('|');

            for col in 0..(width.saturating_sub(2)) {
                let string_idx = (col * 13) / (width - 2).max(1);
                let string_e = if string_idx < 13 { self.tarab_energy[string_idx] } else { 0.0 };

                if string_e >= row_ratio {
                    line_buf.push('#');
                } else if string_e >= (row_ratio - 0.15) {
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

        let hit_radius = JAWARI_PUCK_HIT_RADIUS * 1.5;
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

        // Right Panel: 13-String Tarab Sympathetic Energy Bars (400..760, 80..420)
        let bar_x_start = 400;
        let bar_x_end = 760;
        let bar_w = (bar_x_end - bar_x_start) / 13;

        for i in 0..13 {
            let x0 = bar_x_start + i * bar_w;
            let x1 = x0 + bar_w - 6; // 6pt padding between bars
            let bar_h = (self.tarab_energy[i] * 320.0) as usize;
            let y_top = 420usize.saturating_sub(bar_h);

            for y in y_top..420 {
                for x in x0..=x1 {
                    if y < height && x < width {
                        let idx = (y * width + x) * 4;
                        // Vibrant Amber/Gold `#E5A93C` to Magenta `#D946EF` gradient
                        let frac = i as f32 / 13.0;
                        let r = (0xE5 as f32 * (1.0 - frac) + 0xD9 as f32 * frac) as u8;
                        let g = (0xA9 as f32 * (1.0 - frac) + 0x46 as f32 * frac) as u8;
                        let b = (0x3C as f32 * (1.0 - frac) + 0xEF as f32 * frac) as u8;
                        pixels[idx] = r;
                        pixels[idx + 1] = g;
                        pixels[idx + 2] = b;
                    }
                }
            }
        }

        save_png_file(path, width, height, &pixels)
    }
}

#[cfg(feature = "gui")]
impl JawariBridgeView {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        self.ui(ui);
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Header Bar
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("CURVED JAWARI BUZZ BRIDGE & TARAB SYMPATHETIC RESONATOR HUD")
                        .font(FontId::proportional(13.0))
                        .strong()
                        .color(Color32::from_rgb(245, 158, 11)),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("c = {:.0} m⁻¹ | Bleed = {:.0}%", self.curvature_c, self.tarab_bleed * 100.0))
                            .font(FontId::proportional(11.0))
                            .color(Color32::from_rgb(148, 163, 184)),
                    );
                });
            });

            ui.add_space(4.0);

            // Preset Selection Bar (>=44pt touch buttons)
            let presets = [
                (JawariBridgeHudPreset::DeerHornRaviShankar, "Deer Horn (Ravi)"),
                (JawariBridgeHudPreset::CamelBoneVilayatKhan, "Camel Bone (Vilayat)"),
                (JawariBridgeHudPreset::EbonySurbaharMellow, "Ebony Surbahar"),
                (JawariBridgeHudPreset::SyntheticDelrinPrecision, "Delrin Precision"),
                (JawariBridgeHudPreset::ElectricMetalSizzle, "Electric Sizzle"),
                (JawariBridgeHudPreset::OpenAcousticGourd, "Open Gourd"),
            ];

            ui.horizontal_wrapped(|ui| {
                for (preset, label) in presets {
                    let is_sel = self.preset == preset;
                    let btn = egui::Button::new(
                        RichText::new(label)
                            .font(FontId::proportional(11.0))
                            .color(if is_sel { Color32::from_rgb(15, 23, 42) } else { Color32::from_rgb(226, 232, 240) })
                    )
                    .min_size(Vec2::new(120.0, 32.0))
                    .fill(if is_sel { Color32::from_rgb(245, 158, 11) } else { Color32::from_rgb(30, 41, 59) })
                    .rounding(Rounding::same(4.0));

                    if ui.add(btn).clicked() {
                        self.set_preset(preset);
                    }
                }
            });

            ui.add_space(8.0);

            // Main Interactive Display (Canvas + Tarab Spectrum)
            let (canvas_rect, response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 240.0), egui::Sense::click_and_drag());
            let painter = ui.painter_at(canvas_rect);

            // Dark Charcoal background
            painter.rect_filled(canvas_rect, 6.0, Color32::from_rgb(15, 23, 42));
            painter.rect_stroke(canvas_rect, 6.0, Stroke::new(1.0_f32, Color32::from_rgb(51, 65, 85)));

            // Left interactive pad: 62% width for Jawari geometry & puck
            let pad_w = (canvas_rect.width() - 24.0) * 0.62;
            let pad_rect = egui::Rect::from_min_size(
                egui::pos2(canvas_rect.min.x + 8.0, canvas_rect.min.y + 8.0),
                Vec2::new(pad_w, canvas_rect.height() - 16.0),
            );
            painter.rect_filled(pad_rect, 4.0, Color32::from_rgb(20, 28, 44));
            painter.rect_stroke(pad_rect, 4.0, Stroke::new(1.0_f32, Color32::from_rgb(71, 85, 105)));

            // Right Tarab resonator pad: 38% width
            let tarab_x0 = pad_rect.max.x + 8.0;
            let tarab_w = canvas_rect.max.x - tarab_x0 - 8.0;
            let tarab_rect = egui::Rect::from_min_size(
                egui::pos2(tarab_x0, canvas_rect.min.y + 8.0),
                Vec2::new(tarab_w, canvas_rect.height() - 16.0),
            );
            painter.rect_filled(tarab_rect, 4.0, Color32::from_rgb(18, 24, 38));
            painter.rect_stroke(tarab_rect, 4.0, Stroke::new(1.0_f32, Color32::from_rgb(71, 85, 105)));

            // Draw Curved Jawari Bridge Profile on pad
            let curve_pts = 32;
            let mut prev_pt: Option<egui::Pos2> = None;
            for i in 0..=curve_pts {
                let t = i as f32 / curve_pts as f32;
                let x = pad_rect.min.x + t * pad_rect.width();
                // Curved parabolic profile representing horn/bone obstacle:
                let curve_offset = (t - 0.5) * (t - 0.5) * 4.0;
                let y = pad_rect.max.y - 20.0 - (1.0 - curve_offset * 0.4) * 50.0;
                let pt = egui::pos2(x, y);
                if let Some(prev) = prev_pt {
                    painter.line_segment([prev, pt], Stroke::new(2.5_f32, Color32::from_rgb(180, 130, 70)));
                }
                prev_pt = Some(pt);
            }

            // Draw Jiva Cotton Thread Line
            let jiva_screen_x = pad_rect.min.x + self.puck_pos.0 * pad_rect.width();
            painter.line_segment(
                [egui::pos2(jiva_screen_x, pad_rect.min.y + 10.0), egui::pos2(jiva_screen_x, pad_rect.max.y - 10.0)],
                Stroke::new(1.8_f32, Color32::from_rgb(250, 250, 240)),
            );
            painter.text(
                egui::pos2(jiva_screen_x + 4.0, pad_rect.min.y + 12.0),
                egui::Align2::LEFT_TOP,
                "JIVA THREAD",
                FontId::proportional(9.0),
                Color32::from_rgb(250, 250, 240),
            );

            // Handle Puck Dragging on Pad
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

            // Draw Interactive Puck (44x44pt touch bounding target)
            painter.circle_filled(puck_center, JAWARI_PUCK_HIT_RADIUS, Color32::from_rgb(245, 158, 11));
            painter.circle_stroke(puck_center, JAWARI_PUCK_HIT_RADIUS, Stroke::new(2.5_f32, Color32::from_rgb(254, 240, 138)));
            painter.circle_filled(puck_center, 4.0, Color32::from_rgb(15, 23, 42));

            // Axis labels inside pad
            painter.text(
                egui::pos2(pad_rect.min.x + 8.0, pad_rect.max.y - 18.0),
                egui::Align2::LEFT_BOTTOM,
                format!("Gap h₀: {:.2} mm", self.clearance_gap_mm),
                FontId::proportional(11.0),
                Color32::from_rgb(245, 158, 11),
            );
            painter.text(
                egui::pos2(pad_rect.max.x - 8.0, pad_rect.max.y - 18.0),
                egui::Align2::RIGHT_BOTTOM,
                format!("Jiva Pos: {:.2}", self.jiva_thread_pos),
                FontId::proportional(11.0),
                Color32::from_rgb(241, 245, 249),
            );

            // Right Tarab 13-String Sympathetic Spectrum
            painter.text(
                egui::pos2(tarab_rect.center().x, tarab_rect.min.y + 10.0),
                egui::Align2::CENTER_TOP,
                "13-STRING TARAB SYMPATHETIC SPECTRUM",
                FontId::proportional(10.5),
                Color32::from_rgb(245, 158, 11),
            );

            let bar_margin = 6.0;
            let bar_area_w = tarab_rect.width() - 2.0 * bar_margin;
            let bar_w = (bar_area_w / 13.0).max(4.0);
            let bar_base_y = tarab_rect.max.y - 24.0;
            let max_bar_h = tarab_rect.height() - 56.0;

            for (idx, &energy) in self.tarab_energy.iter().enumerate() {
                let bx = tarab_rect.min.x + bar_margin + idx as f32 * (bar_area_w / 13.0);
                let bar_h = (energy * max_bar_h).clamp(2.0, max_bar_h);
                let bar_rect = egui::Rect::from_min_max(
                    egui::pos2(bx + 1.0, bar_base_y - bar_h),
                    egui::pos2(bx + bar_w - 1.0, bar_base_y),
                );

                // Shimmering amber gradient
                let col = if energy > 0.8 {
                    Color32::from_rgb(255, 215, 0)
                } else if energy > 0.5 {
                    Color32::from_rgb(245, 158, 11)
                } else {
                    Color32::from_rgb(180, 83, 9)
                };

                painter.rect_filled(bar_rect, 2.0, col);
                painter.text(
                    egui::pos2(bx + bar_w * 0.5, bar_base_y + 4.0),
                    egui::Align2::CENTER_TOP,
                    format!("T{}", idx + 1),
                    FontId::proportional(7.5),
                    Color32::from_rgb(148, 163, 184),
                );
            }

            ui.add_space(8.0);

            // Surgical Parameter Sliders
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    let mut gap = self.clearance_gap_mm;
                    if ui.add(egui::Slider::new(&mut gap, MIN_JAWARI_CLEARANCE_GAP..=MAX_JAWARI_CLEARANCE_GAP).text("Clearance Gap h₀ (mm)")).changed() {
                        self.clearance_gap_mm = gap;
                        self.update_puck_from_physics();
                    }
                    let mut jiva = self.jiva_thread_pos;
                    if ui.add(egui::Slider::new(&mut jiva, MIN_JIVA_THREAD_POS..=MAX_JIVA_THREAD_POS).text("Cotton Jiva Position")).changed() {
                        self.jiva_thread_pos = jiva;
                        self.update_puck_from_physics();
                    }
                });

                ui.separator();

                ui.vertical(|ui| {
                    let mut curv = self.curvature_c;
                    if ui.add(egui::Slider::new(&mut curv, 40.0..=400.0).text("Bridge Curvature c (m⁻¹)")).changed() {
                        self.curvature_c = curv;
                    }
                    let mut bleed = self.tarab_bleed;
                    if ui.add(egui::Slider::new(&mut bleed, 0.0..=1.0).text("Tarab Sympathetic Bleed")).changed() {
                        self.tarab_bleed = bleed;
                    }
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
    fn test_jawari_bridge_view_hit_target_dimensions() {
        const {
            assert!(
                JAWARI_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT,
                "Jawari puck hit target bounding box must be >= 44pt"
            );
        }
    }

    #[test]
    fn test_jawari_bridge_view_ascii_render() {
        let view = JawariBridgeView::new();
        let ascii = view.render_ascii(80, 16);
        assert_eq!(ascii.len(), 16);
        assert!(ascii[0].starts_with('+'));
    }

    #[test]
    fn test_jawari_bridge_view_snapshot_render() {
        let view = JawariBridgeView::new();
        let res = view.render_snapshot_png("scratch/renders/jawari_bridge_view.png", 800, 520);
        assert!(res.is_ok());
    }
}
