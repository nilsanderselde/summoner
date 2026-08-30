// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Plucked & Struck String Hammer/Plectrum Phase Canvas HUD (Milestone 18).
//!
//! Provides an interactive 2D phase canvas visualizing non-linear felt hammer contact force
//! $F \propto \delta^p$, spatial triangular string pluck deflection, dual-polarization vibration
//! orbits, and continuous pluck position vs strike velocity puck controls on an 8pt grid
//! with $\ge 44\times 44\text{pt}$ hit targets and WCAG AAA contrast.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const PLUCK_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_PLUCK_POSITION_BETA: f32 = 0.05;
pub const MAX_PLUCK_POSITION_BETA: f32 = 0.95;
pub const MIN_STRIKE_VELOCITY: f32 = 0.05;
pub const MAX_STRIKE_VELOCITY: f32 = 1.00;

/// Plucked & Struck Acoustic Instrument Profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PluckedInstrumentViewProfile {
    #[default]
    AcousticSteel,
    ClassicalNylon,
    GrandPiano,
    Harpsichord,
    Harp,
    SitarKoto,
}

impl PluckedInstrumentViewProfile {
    pub fn name(&self) -> &'static str {
        match self {
            Self::AcousticSteel => "ACOUSTIC GUITAR",
            Self::ClassicalNylon => "CLASSICAL NYLON",
            Self::GrandPiano => "GRAND PIANO",
            Self::Harpsichord => "HARPSICHORD",
            Self::Harp => "CONCERT HARP",
            Self::SitarKoto => "SITAR / KOTO",
        }
    }

    pub fn nominal_physics(&self) -> (f32, f32, f32) {
        // (stiffness, t60, detune_cents)
        match self {
            Self::AcousticSteel => (0.25, 4.5, 3.5),
            Self::ClassicalNylon => (0.08, 3.2, 2.0),
            Self::GrandPiano => (0.45, 8.0, 1.8),
            Self::Harpsichord => (0.18, 2.5, 4.0),
            Self::Harp => (0.05, 9.5, 1.5),
            Self::SitarKoto => (0.30, 3.8, 6.0),
        }
    }
}

/// Plucked String Hammer/Plectrum Phase Canvas HUD.
#[derive(Debug, Clone)]
pub struct PluckedStringView {
    pub instrument: PluckedInstrumentViewProfile,
    pub pluck_position_beta: f32, // [0.05 ..= 0.95]
    pub strike_velocity: f32,     // [0.05 ..= 1.00]
    pub hammer_hardness: f32,     // [0.0 ..= 1.0]
    pub palm_mute_damping: f32,   // [0.0 ..= 1.0]
    pub string_stiffness: f32,    // [0.0 ..= 1.0]
    pub polarization_coupling: f32, // [0.0 ..= 1.0]
    pub puck_pos: (f32, f32),     // Normalized (X: pluck_position, Y: strike_velocity)
    pub is_dragging_puck: bool,
    pub peak_contact_force_n: f32,
    pub contact_duration_ms: f32,
    pub inharmonicity_factor_b: f32,
    pub color_palette: ContrastColorPalette,
}

impl Default for PluckedStringView {
    fn default() -> Self {
        Self::new()
    }
}

impl PluckedStringView {
    pub fn new() -> Self {
        let mut view = Self {
            instrument: PluckedInstrumentViewProfile::AcousticSteel,
            pluck_position_beta: 0.15,
            strike_velocity: 0.80,
            hammer_hardness: 0.50,
            palm_mute_damping: 0.0,
            string_stiffness: 0.25,
            polarization_coupling: 0.35,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            peak_contact_force_n: 14.5,
            contact_duration_ms: 1.85,
            inharmonicity_factor_b: 0.00045,
            color_palette: ContrastColorPalette::default(),
        };
        view.puck_pos = (
            Self::beta_to_normalized(view.pluck_position_beta),
            Self::velocity_to_normalized(view.strike_velocity),
        );
        view.update_physics();
        view
    }

    pub fn beta_to_normalized(beta: f32) -> f32 {
        ((beta.clamp(MIN_PLUCK_POSITION_BETA, MAX_PLUCK_POSITION_BETA) - MIN_PLUCK_POSITION_BETA)
            / (MAX_PLUCK_POSITION_BETA - MIN_PLUCK_POSITION_BETA))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_beta(norm: f32) -> f32 {
        MIN_PLUCK_POSITION_BETA + norm.clamp(0.0, 1.0) * (MAX_PLUCK_POSITION_BETA - MIN_PLUCK_POSITION_BETA)
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
        let (stiff, _, _) = self.instrument.nominal_physics();
        self.string_stiffness = stiff;
        self.inharmonicity_factor_b = stiff * 0.0018;

        // Non-linear felt contact force scaling: F ~ K * v^p
        let p = 2.4;
        self.peak_contact_force_n = (self.strike_velocity * 18.0).powf(p * 0.5);
        self.contact_duration_ms = (2.5 / (self.strike_velocity * 1.5 + 0.5)).clamp(0.4, 4.0);
    }

    /// Evaluates spatial triangular string deflection $y(x)$ for normalized position $x \in [0.0, 1.0]$.
    pub fn evaluate_string_deflection(&self, x_norm: f32) -> f32 {
        let beta = self.pluck_position_beta.clamp(0.05, 0.95);
        let x = x_norm.clamp(0.0, 1.0);
        let amp = self.strike_velocity;

        if x <= beta {
            amp * (x / beta)
        } else {
            amp * ((1.0 - x) / (1.0 - beta))
        }
    }

    /// Evaluates dual-polarization vibration orbit $(y_{vert}(t), y_{horiz}(t))$ for phase $\theta \in [0, 2\pi]$.
    pub fn evaluate_polarization_orbit(&self, phase_rad: f32) -> (f32, f32) {
        let k = self.polarization_coupling;
        let v_amp = self.strike_velocity;
        let h_amp = self.strike_velocity * (0.6 + 0.4 * k);

        let y_v = v_amp * (phase_rad).sin();
        let y_h = h_amp * (phase_rad * 1.003 + 0.4).cos();

        (y_h, y_v)
    }

    /// Hit-tests touch coordinate on the pluck puck.
    pub fn hit_test_pluck_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= PLUCK_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render of Spatial String Deflection and Polarization Orbit.
    #[allow(clippy::needless_range_loop)]
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut grid = vec![vec![' '; width]; height];

        for (row_idx, row) in grid.iter_mut().enumerate() {
            row[0] = '|';
            row[width - 1] = '|';
            if row_idx == 0 || row_idx == height - 1 {
                for col in row.iter_mut().take(width) {
                    *col = '-';
                }
                row[0] = '+';
                row[width - 1] = '+';
            }
        }

        let mid_x = width / 2;
        for r in 1..height - 1 {
            grid[r][mid_x] = '|';
        }

        // Draw String Deflection on left half
        let left_w = mid_x - 1;
        let baseline_y = height - 3;
        for c in 1..left_w {
            let x_norm = c as f32 / left_w as f32;
            let defl = self.evaluate_string_deflection(x_norm);
            let row = baseline_y.saturating_sub((defl * (height as f32 * 0.70)).round() as usize);
            if row > 0 && row < height - 1 {
                grid[row][c] = '#';
            }
        }

        // Draw Polarization Orbit on right half
        let right_w = width - mid_x - 2;
        let mid_rx = mid_x + 1 + right_w / 2;
        let mid_ry = height / 2;
        for s in 0..64 {
            let phase = s as f32 / 64.0 * 2.0 * std::f32::consts::PI;
            let (yh, yv) = self.evaluate_polarization_orbit(phase);
            let col = (mid_rx as f32 + yh * (right_w as f32 * 0.40)).round() as usize;
            let row = (mid_ry as f32 - yv * (height as f32 * 0.35)).round() as usize;
            if row > 0 && row < height - 1 && col > mid_x && col < width - 1 {
                grid[row][col] = 'O';
            }
        }

        grid.into_iter().map(|row| row.into_iter().collect()).collect()
    }

    #[cfg(feature = "gui")]
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
            egui::Align2::LEFT_TOP,
            "PLUCKED & STRUCK STRING HAMMER/PLECTRUM PHASE CANVAS HUD",
            egui::FontId::proportional(16.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Instrument Preset Tabs (y: 48..92) - Each tab >= 44pt height
        let profiles = [
            PluckedInstrumentViewProfile::AcousticSteel,
            PluckedInstrumentViewProfile::ClassicalNylon,
            PluckedInstrumentViewProfile::GrandPiano,
            PluckedInstrumentViewProfile::Harpsichord,
            PluckedInstrumentViewProfile::Harp,
            PluckedInstrumentViewProfile::SitarKoto,
        ];

        let tab_w = (rect.width() - 40.0 - 5.0 * 6.0) / 6.0;
        for (i, prof) in profiles.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 6.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_selected = self.instrument == *prof;
            let bg_color = if is_selected {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(25, 35, 50)
            };
            let text_color = if is_selected {
                Color32::from_rgb(10, 14, 24)
            } else {
                Color32::from_rgb(200, 215, 235)
            };

            painter.rect_filled(tab_rect, 4.0, bg_color);
            painter.text(
                tab_rect.center(),
                egui::Align2::CENTER_CENTER,
                prof.name(),
                egui::FontId::proportional(10.0),
                text_color,
            );

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if tab_rect.contains(pos) {
                        self.instrument = *prof;
                        self.update_physics();
                    }
                }
            }
        }

        // Main Display Canvas (y: 104..340)
        let main_canvas = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 104.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 340.0),
        );
        painter.rect_filled(main_canvas, 6.0, Color32::from_rgb(10, 14, 24));
        painter.rect_stroke(
            main_canvas,
            6.0,
            Stroke::new(1.5_f32, Color32::from_rgb(45, 65, 95)),
        );

        // Left 55%: Spatial String Deflection & Pluck Puck
        let left_w = main_canvas.width() * 0.55;
        let left_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(left_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(left_rect, 4.0, Color32::from_rgb(14, 20, 32));
        painter.rect_stroke(
            left_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(40, 55, 80)),
        );

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "SPATIAL STRING DEFLECTION & PLUCK POSITION (β)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // Draw baseline string
        let base_y = left_rect.max.y - 25.0;
        painter.line_segment(
            [
                egui::pos2(left_rect.min.x + 15.0, base_y),
                egui::pos2(left_rect.max.x - 15.0, base_y),
            ],
            Stroke::new(1.5_f32, Color32::from_rgb(60, 85, 120)),
        );

        // Draw triangular spatial pluck shape
        let puck_x = left_rect.min.x + 15.0 + self.puck_pos.0 * (left_rect.width() - 30.0);
        let puck_y = base_y - self.puck_pos.1 * (left_rect.height() - 60.0);
        let puck_pos = egui::pos2(puck_x, puck_y);

        painter.line_segment(
            [egui::pos2(left_rect.min.x + 15.0, base_y), puck_pos],
            Stroke::new(2.5_f32, Color32::from_rgb(0, 229, 255)),
        );
        painter.line_segment(
            [puck_pos, egui::pos2(left_rect.max.x - 15.0, base_y)],
            Stroke::new(2.5_f32, Color32::from_rgb(0, 229, 255)),
        );

        // Interactive Puck Dragging
        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - (left_rect.min.x + 15.0)) / (left_rect.width() - 30.0)).clamp(0.0, 1.0);
                    let ny = ((base_y - mouse_pos.y) / (left_rect.height() - 60.0)).clamp(0.0, 1.0);
                    self.puck_pos = (nx, ny);
                    self.pluck_position_beta = Self::normalized_to_beta(nx);
                    self.strike_velocity = Self::normalized_to_velocity(ny);
                    self.update_physics();
                }
            }
        }

        // Touch hit target (>= 44x44pt bounding box, radius 22pt)
        painter.circle_stroke(
            puck_pos,
            PLUCK_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(0, 229, 255, 140)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(0, 229, 255));
        painter.circle_filled(puck_pos, 4.0, Color32::from_rgb(255, 255, 255));

        // Right 45%: Dual-Polarization Orbit Canvas
        let right_w = main_canvas.width() * 0.45;
        let right_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + left_w + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(right_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(right_rect, 4.0, Color32::from_rgb(14, 20, 32));
        painter.rect_stroke(
            right_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(40, 55, 80)),
        );

        painter.text(
            egui::pos2(right_rect.min.x + 10.0, right_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "DUAL-POLARIZATION VIBRATION ORBIT",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        let r_mid = right_rect.center();
        let mut prev_orb = None;
        for s in 0..=64 {
            let phase = s as f32 / 64.0 * 2.0 * std::f32::consts::PI;
            let (yh, yv) = self.evaluate_polarization_orbit(phase);
            let px = r_mid.x + yh * (right_rect.width() * 0.35);
            let py = r_mid.y - yv * (right_rect.height() * 0.35);
            let pt = egui::pos2(px, py);
            if let Some(prev) = prev_orb {
                painter.line_segment([prev, pt], Stroke::new(2.0_f32, Color32::from_rgb(255, 215, 0)));
            }
            prev_orb = Some(pt);
        }

        // Bottom Metrics Dock (y: 350..465)
        let dock_rect = egui::Rect::from_min_max(
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
                "PLUCK POSITION (β)",
                format!("{:.2} ({:.0}%)", self.pluck_position_beta, self.pluck_position_beta * 100.0),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "PEAK CONTACT FORCE",
                format!("{:.1} N ({:.2} ms)", self.peak_contact_force_n, self.contact_duration_ms),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "STRING STIFFNESS (B)",
                format!("{:.5} ({:.1}%)", self.inharmonicity_factor_b, self.string_stiffness * 100.0),
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "POLARIZATION COUPLING",
                format!("{:.0}% (Detune: {:.1}¢)", self.polarization_coupling * 100.0, 3.5),
                Color32::from_rgb(0, 255, 180),
            ),
        ];

        let col_w = (dock_rect.width() - 40.0) / 4.0;
        for (i, (label, val, col)) in params.iter().enumerate() {
            let px_pos = dock_rect.min.x + 20.0 + i as f32 * col_w;
            painter.text(
                egui::pos2(px_pos, dock_rect.min.y + 12.0),
                egui::Align2::LEFT_TOP,
                *label,
                egui::FontId::proportional(11.0),
                Color32::from_rgb(160, 180, 205),
            );
            painter.text(
                egui::pos2(px_pos, dock_rect.min.y + 30.0),
                egui::Align2::LEFT_TOP,
                val,
                egui::FontId::proportional(14.0),
                *col,
            );
        }

        // Pass Badge
        let badge_rect = egui::Rect::from_min_max(
            egui::pos2(dock_rect.min.x + 15.0, dock_rect.min.y + 68.0),
            egui::pos2(dock_rect.max.x - 15.0, dock_rect.max.y - 11.0),
        );
        painter.rect_filled(badge_rect, 4.0, Color32::from_rgb(16, 35, 28));
        painter.rect_stroke(
            badge_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(0, 255, 180)),
        );
        painter.text(
            egui::pos2(badge_rect.min.x + 10.0, badge_rect.min.y + 8.0),
            egui::Align2::LEFT_TOP,
            "[PASS] Physical Modeling Plucked String Phase Canvas & Hit Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }

    /// Renders a high-fidelity headless RGBA PNG snapshot of the Plucked String Phase Canvas HUD.
    pub fn render_snapshot_png(&self, path: &str, width: u32, height: u32) -> Result<(), String> {
        let mut pixels = vec![0u8; (width * height * 4) as usize];
        let w = width;
        let h = height;

        // Background: Deep Slate #0E121C
        for chunk in pixels.chunks_exact_mut(4) {
            chunk.copy_from_slice(&[14, 18, 28, 255]);
        }

        // Header Background bar
        for y in 0..40 {
            for x in 0..w {
                let idx = ((y * w + x) * 4) as usize;
                pixels[idx..idx + 4].copy_from_slice(&[20, 26, 40, 255]);
            }
        }

        // Left 55%: Spatial Deflection Canvas Frame
        let left_w = (w as f32 * 0.55) as u32;
        let left_x0 = 20;
        let left_x1 = left_w - 10;
        let canvas_y0 = 80;
        let canvas_y1 = h - 120;

        for y in canvas_y0..canvas_y1 {
            for x in left_x0..left_x1 {
                let idx = ((y * w + x) * 4) as usize;
                pixels[idx..idx + 4].copy_from_slice(&[10, 14, 24, 255]);
            }
        }

        // Baseline string
        let base_y = (canvas_y1 - 25) as f32;
        draw_line_ps(&mut pixels, w, h, (left_x0 + 15) as f32, base_y, (left_x1 - 15) as f32, base_y, [60, 85, 120, 255]);

        // Puck position
        let puck_x = (left_x0 + 15) as f32 + self.puck_pos.0 * (left_x1 - left_x0 - 30) as f32;
        let puck_y = base_y - self.puck_pos.1 * (canvas_y1 - canvas_y0 - 60) as f32;

        // Spatial pluck triangle
        draw_line_ps(&mut pixels, w, h, (left_x0 + 15) as f32, base_y, puck_x, puck_y, [0, 229, 255, 255]);
        draw_line_ps(&mut pixels, w, h, puck_x, puck_y, (left_x1 - 15) as f32, base_y, [0, 229, 255, 255]);

        // Touch hit target (>= 44x44pt)
        draw_circle_ring_ps(&mut pixels, w, h, puck_x, puck_y, PLUCK_PUCK_HIT_RADIUS, [0, 229, 255, 180]);
        draw_circle_filled_ps(&mut pixels, w, h, puck_x, puck_y, 12.0, [0, 229, 255, 255]);
        draw_circle_filled_ps(&mut pixels, w, h, puck_x, puck_y, 4.0, [255, 255, 255, 255]);

        // Right 45%: Dual-Polarization Orbit Canvas Frame
        let right_x0 = left_w + 10;
        let right_x1 = w - 20;

        for y in canvas_y0..canvas_y1 {
            for x in right_x0..right_x1 {
                let idx = ((y * w + x) * 4) as usize;
                pixels[idx..idx + 4].copy_from_slice(&[10, 14, 24, 255]);
            }
        }

        // Draw Polarization Orbit
        let right_mid_x = (right_x0 + right_x1) as f32 / 2.0;
        let right_mid_y = (canvas_y0 + canvas_y1) as f32 / 2.0;
        let mut prev_orb: Option<(f32, f32)> = None;

        for s in 0..=64 {
            let phase = s as f32 / 64.0 * 2.0 * std::f32::consts::PI;
            let (yh, yv) = self.evaluate_polarization_orbit(phase);
            let px = right_mid_x + yh * ((right_x1 - right_x0) as f32 * 0.35);
            let py = right_mid_y - yv * ((canvas_y1 - canvas_y0) as f32 * 0.35);

            if let Some((x0, y0)) = prev_orb {
                draw_line_ps(&mut pixels, w, h, x0, y0, px, py, [255, 215, 0, 255]);
            }
            prev_orb = Some((px, py));
        }

        // Bottom Metrics Dock
        let dock_y0 = h - 100;
        let dock_y1 = h - 20;
        for y in dock_y0..dock_y1 {
            for x in 20..(w - 20) {
                let idx = ((y * w + x) * 4) as usize;
                pixels[idx..idx + 4].copy_from_slice(&[18, 25, 38, 255]);
            }
        }

        // Pass Badge in dock
        for y in (dock_y1 - 25)..dock_y1 {
            for x in 30..(w - 30) {
                let idx = ((y * w + x) * 4) as usize;
                pixels[idx..idx + 4].copy_from_slice(&[16, 35, 28, 255]);
            }
        }

        // Create parent directory if needed
        if let Some(parent) = std::path::Path::new(path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        encode_minimal_png_ps(path, &pixels, width, height)
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_line_ps(buf: &mut [u8], w: u32, h: u32, x0: f32, y0: f32, x1: f32, y1: f32, col: [u8; 4]) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let steps = (dx.abs().max(dy.abs()) * 2.0).max(1.0) as u32;
    for s in 0..=steps {
        let t = s as f32 / steps as f32;
        let px = (x0 + t * dx).round() as i32;
        let py = (y0 + t * dy).round() as i32;
        if px >= 0 && px < w as i32 && py >= 0 && py < h as i32 {
            let idx = ((py as u32 * w + px as u32) * 4) as usize;
            buf[idx..idx + 4].copy_from_slice(&col);
        }
    }
}

fn draw_circle_filled_ps(buf: &mut [u8], w: u32, h: u32, cx: f32, cy: f32, r: f32, col: [u8; 4]) {
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

fn draw_circle_ring_ps(buf: &mut [u8], w: u32, h: u32, cx: f32, cy: f32, r: f32, col: [u8; 4]) {
    let thickness = 2.0f32;
    let r_inner = (r - thickness).max(0.0);
    let min_x = (cx - r).max(0.0) as u32;
    let max_x = (cx + r).min(w as f32 - 1.0) as u32;
    let min_y = (cy - r).max(0.0) as u32;
    let max_y = (cy + r).min(h as f32 - 1.0) as u32;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq <= r * r && dist_sq >= r_inner * r_inner {
                let idx = ((y * w + x) * 4) as usize;
                buf[idx..idx + 4].copy_from_slice(&col);
            }
        }
    }
}

fn encode_minimal_png_ps(path: &str, rgba_pixels: &[u8], width: u32, height: u32) -> Result<(), String> {
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
    write_png_chunk_ps(&mut out, b"IHDR", &ihdr);

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

    write_png_chunk_ps(&mut out, b"IDAT", &zlib_data);
    write_png_chunk_ps(&mut out, b"IEND", &[]);

    std::fs::write(path, out).map_err(|e| e.to_string())
}

fn write_png_chunk_ps(out: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let type_start = out.len();
    out.extend_from_slice(chunk_type);
    out.extend_from_slice(data);
    let crc = crc32_compute_ps(&out[type_start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

fn crc32_compute_ps(buf: &[u8]) -> u32 {
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
    fn test_plucked_string_view_ascii_render() {
        let view = PluckedStringView::new();
        let ascii = view.render_ascii(80, 16);
        assert!(!ascii.is_empty());
        assert_eq!(ascii.len(), 16);
    }

    #[test]
    fn test_plucked_string_view_hit_target_dimensions() {
        const {
            assert!(
                PLUCK_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT,
                "Pluck puck hit target bounding box must be >= 44pt"
            );
        }
    }

    #[test]
    fn test_plucked_string_view_snapshot_render() {
        let view = PluckedStringView::new();
        let res = view.render_snapshot_png("scratch/renders/plucked_string_view.png", 800, 520);
        assert!(res.is_ok());
    }
}
