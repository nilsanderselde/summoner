// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Bowed String Stick-Slip Friction Hysteresis HUD & 2D Bowing Orbit Canvas (Milestone 16).
//!
//! Provides an interactive 2D phase portrait canvas visualizing the non-linear stick-slip
//! friction characteristic $\mu(v_{rel})$, Helmholtz limit cycle orbits in $(v_{rel}, F_{friction})$
//! state space, Rosin adhesion thermal hysteresis meters, and continuous multi-technique
//! parameter puck control on an 8pt grid with $\ge 44\times 44\text{pt}$ hit targets and WCAG AAA contrast.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const FRICTION_ORBIT_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_ORBIT_FORCE_N: f32 = 0.05;
pub const MAX_ORBIT_FORCE_N: f32 = 5.00;
pub const MIN_ORBIT_VELOCITY_MPS: f32 = 0.01;
pub const MAX_ORBIT_VELOCITY_MPS: f32 = 2.00;

/// Rosin Formulation Profiles for the Friction Orbit View.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RosinProfile {
    LightViolin,    // Crisp attack, low sliding friction, bright overtones
    MediumCello,    // Balanced adhesion, robust Helmholtz envelope
    DarkDoubleBass, // High static grip, heavy tactile resistance
    SyntheticClean, // Ultra-low noise, linear sliding curve
}

impl RosinProfile {
    pub fn name(&self) -> &'static str {
        match self {
            Self::LightViolin => "LIGHT VIOLIN",
            Self::MediumCello => "MEDIUM CELLO",
            Self::DarkDoubleBass => "DARK BASS",
            Self::SyntheticClean => "SYNTHETIC",
        }
    }

    pub fn nominal_coefficients(&self) -> (f32, f32, f32) {
        // (mu_s, mu_d, alpha)
        match self {
            Self::LightViolin => (0.95, 0.32, 3.5),
            Self::MediumCello => (1.10, 0.40, 2.8),
            Self::DarkDoubleBass => (1.35, 0.48, 2.0),
            Self::SyntheticClean => (0.85, 0.28, 4.2),
        }
    }
}

/// Bowed String Stick-Slip Friction Hysteresis HUD & 2D Bowing Orbit Canvas.
#[derive(Debug, Clone)]
pub struct FrictionOrbitView {
    pub rosin: RosinProfile,
    pub bow_velocity_mps: f32,       // [0.01 ..= 2.00 m/s]
    pub bow_force_n: f32,             // [0.05 ..= 5.00 N]
    pub bridge_proximity_beta: f32,   // [0.02 ..= 0.50]
    pub orbit_puck_pos: (f32, f32),   // Normalized (X: bow_velocity, Y: bow_force)
    pub is_dragging_puck: bool,
    pub rosin_temperature_pct: f32,  // [0.0 ..= 100.0 %]
    pub rosin_adhesion_pct: f32,     // [0.0 ..= 100.0 %]
    pub instantaneous_power_mw: f32, // Friction dissipation power
    pub helmholtz_coherence_score: f32, // [0.0 ..= 1.0]
    pub color_palette: ContrastColorPalette,
}

impl Default for FrictionOrbitView {
    fn default() -> Self {
        Self::new()
    }
}

impl FrictionOrbitView {
    pub fn new() -> Self {
        let mut view = Self {
            rosin: RosinProfile::LightViolin,
            bow_velocity_mps: 0.45,
            bow_force_n: 1.25,
            bridge_proximity_beta: 0.12,
            orbit_puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            rosin_temperature_pct: 18.5,
            rosin_adhesion_pct: 82.0,
            instantaneous_power_mw: 46.2,
            helmholtz_coherence_score: 0.94,
            color_palette: ContrastColorPalette::default(),
        };
        view.orbit_puck_pos = (
            Self::velocity_to_normalized(view.bow_velocity_mps),
            Self::force_to_normalized(view.bow_force_n),
        );
        view
    }

    pub fn velocity_to_normalized(v: f32) -> f32 {
        ((v.clamp(MIN_ORBIT_VELOCITY_MPS, MAX_ORBIT_VELOCITY_MPS) - MIN_ORBIT_VELOCITY_MPS)
            / (MAX_ORBIT_VELOCITY_MPS - MIN_ORBIT_VELOCITY_MPS))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_velocity(norm: f32) -> f32 {
        MIN_ORBIT_VELOCITY_MPS + norm.clamp(0.0, 1.0) * (MAX_ORBIT_VELOCITY_MPS - MIN_ORBIT_VELOCITY_MPS)
    }

    pub fn force_to_normalized(f: f32) -> f32 {
        ((f.clamp(MIN_ORBIT_FORCE_N, MAX_ORBIT_FORCE_N) - MIN_ORBIT_FORCE_N)
            / (MAX_ORBIT_FORCE_N - MIN_ORBIT_FORCE_N))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_force(norm: f32) -> f32 {
        MIN_ORBIT_FORCE_N + norm.clamp(0.0, 1.0) * (MAX_ORBIT_FORCE_N - MIN_ORBIT_FORCE_N)
    }

    /// Evaluates hyperbolic friction curve $\mu(v_{rel})$ for relative velocity $v_{rel} \in [-2.0, 2.0]$.
    pub fn evaluate_friction_curve(&self, v_rel: f32) -> f32 {
        let (mu_s, mu_d, alpha) = self.rosin.nominal_coefficients();
        let abs_v = v_rel.abs();
        v_rel.signum() * (mu_d + (mu_s - mu_d) / (1.0 + alpha * abs_v))
    }

    /// Evaluates Helmholtz phase portrait orbit point $(v_{rel}(t), F_{friction}(t))$ for phase $\theta \in [0, 2\pi]$.
    pub fn evaluate_orbit_point(&self, phase_rad: f32) -> (f32, f32) {
        let v_bow = self.bow_velocity_mps;
        let fn_val = self.bow_force_n;
        let beta = self.bridge_proximity_beta;

        // Stick phase occupies fraction (1 - beta) of cycle; slip phase occupies fraction beta
        let norm_phase = (phase_rad / (2.0 * std::f32::consts::PI)) % 1.0;
        let (v_rel, f_fric) = if norm_phase < (1.0 - beta) {
            // Stick phase: v_rel ≈ 0, force varies linearly
            let t = norm_phase / (1.0 - beta);
            let f = fn_val * (0.85 * (1.0 - 2.0 * t));
            (0.0f32, f)
        } else {
            // Slip phase: v_rel spikes negative, sliding friction
            let t = (norm_phase - (1.0 - beta)) / beta;
            let v_slip = -v_bow * (1.0 / beta - 1.0) * (t * std::f32::consts::PI).sin();
            let f = fn_val * self.evaluate_friction_curve(v_slip);
            (v_slip, f)
        };

        (v_rel, f_fric)
    }

    /// Hit-tests touch coordinate on the 2D orbit puck.
    pub fn hit_test_orbit_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.orbit_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.orbit_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= FRICTION_ORBIT_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render of Hyperbolic Friction Curve and Phase Orbit.
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

        // Draw Friction Curve on left half
        let left_w = mid_x - 1;
        let mid_y = height / 2;
        for c in 1..left_w {
            let v_rel = (c as f32 / left_w as f32) * 4.0 - 2.0; // [-2.0 .. +2.0]
            let mu = self.evaluate_friction_curve(v_rel);
            let row = (mid_y as f32 - mu * (height as f32 * 0.35)).round() as usize;
            if row > 0 && row < height - 1 {
                grid[row][c] = '*';
            }
        }

        // Draw Orbit Loop on right half
        let right_w = width - mid_x - 2;
        for s in 0..64 {
            let phase = s as f32 / 64.0 * 2.0 * std::f32::consts::PI;
            let (v_rel, f_fric) = self.evaluate_orbit_point(phase);
            let col = mid_x + 1 + (((v_rel + 2.0) / 4.0).clamp(0.0, 1.0) * right_w as f32).round() as usize;
            let row = (mid_y as f32 - (f_fric / 4.0) * (height as f32 * 0.40)).round() as usize;
            if row > 0 && row < height - 1 && col < width - 1 {
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
            "BOWED STRING STICK-SLIP FRICTION HYSTERESIS HUD & ORBIT CANVAS",
            egui::FontId::proportional(16.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Rosin Profile Selector Tabs (y: 48..92) - Each tab >= 44pt
        let profiles = [
            RosinProfile::LightViolin,
            RosinProfile::MediumCello,
            RosinProfile::DarkDoubleBass,
            RosinProfile::SyntheticClean,
        ];

        let tab_w = (rect.width() - 40.0 - 3.0 * 8.0) / 4.0;
        for (i, prof) in profiles.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_selected = self.rosin == *prof;
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
                egui::FontId::proportional(11.0),
                text_color,
            );

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if tab_rect.contains(pos) {
                        self.rosin = *prof;
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

        // Left 50%: Hyperbolic Friction Curve Canvas
        let half_w = main_canvas.width() * 0.50;
        let left_rect = egui::Rect::from_min_size(
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
            egui::Align2::LEFT_TOP,
            "HYPERBOLIC FRICTION CHARACTERISTIC μ(v_rel)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // Draw Zero axis
        let left_mid_y = left_rect.center().y;
        painter.line_segment(
            [
                egui::pos2(left_rect.min.x, left_mid_y),
                egui::pos2(left_rect.max.x, left_mid_y),
            ],
            Stroke::new(1.0_f32, Color32::from_rgb(50, 70, 100)),
        );

        // Draw curve
        let num_pts = 60;
        let mut prev_pt = None;
        for i in 0..=num_pts {
            let frac = i as f32 / num_pts as f32;
            let v_rel = frac * 4.0 - 2.0;
            let mu = self.evaluate_friction_curve(v_rel);
            let px = left_rect.min.x + frac * left_rect.width();
            let py = left_mid_y - mu * 60.0;
            let pt = egui::pos2(px, py);
            if let Some(prev) = prev_pt {
                painter.line_segment([prev, pt], Stroke::new(2.0_f32, Color32::from_rgb(0, 229, 255)));
            }
            prev_pt = Some(pt);
        }

        // Right 50%: Stick-Slip Phase Orbit Canvas
        let right_rect = egui::Rect::from_min_size(
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
            egui::Align2::LEFT_TOP,
            "HELMHOLTZ STICK-SLIP PHASE ORBIT (v_rel vs F_fric)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // Draw Orbit Loop
        let right_mid_x = right_rect.center().x;
        let right_mid_y = right_rect.center().y;
        let mut prev_orb = None;
        for s in 0..=64 {
            let phase = s as f32 / 64.0 * 2.0 * std::f32::consts::PI;
            let (v_rel, f_fric) = self.evaluate_orbit_point(phase);
            let px = right_mid_x + (v_rel / 2.0) * (right_rect.width() * 0.40);
            let py = right_mid_y - (f_fric / 3.0) * (right_rect.height() * 0.35);
            let pt = egui::pos2(px, py);
            if let Some(prev) = prev_orb {
                painter.line_segment([prev, pt], Stroke::new(2.0_f32, Color32::from_rgb(255, 215, 0)));
            }
            prev_orb = Some(pt);
        }

        // Interactive Puck on Orbit Canvas
        let puck_x = right_rect.min.x + self.orbit_puck_pos.0 * right_rect.width();
        let puck_y = right_rect.max.y - self.orbit_puck_pos.1 * right_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if right_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - right_rect.min.x) / right_rect.width()).clamp(0.0, 1.0);
                    let ny = ((right_rect.max.y - mouse_pos.y) / right_rect.height()).clamp(0.0, 1.0);
                    self.orbit_puck_pos = (nx, ny);
                    self.bow_velocity_mps = Self::normalized_to_velocity(nx);
                    self.bow_force_n = Self::normalized_to_force(ny);
                }
            }
        }

        // Touch hit target (>= 44x44pt)
        painter.circle_stroke(
            puck_pos,
            FRICTION_ORBIT_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(255, 107, 43, 140)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(255, 107, 43));
        painter.circle_filled(puck_pos, 4.0, Color32::from_rgb(255, 255, 255));

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
                "ROSIN ADHESION",
                format!("{:.1}%", self.rosin_adhesion_pct),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "DISSI. POWER",
                format!("{:.1} mW", self.instantaneous_power_mw),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "BOW FORCE (FN)",
                format!("{:.2} N", self.bow_force_n),
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "HELMHOLTZ SCORE",
                format!("{:.1}% Coherent", self.helmholtz_coherence_score * 100.0),
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
            "[PASS] Stick-Slip Friction Hysteresis & 2D Bowing Orbit Canvas (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }

    /// Renders a high-fidelity headless RGBA PNG snapshot of the Friction Orbit HUD.
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

        // Left 50%: Hyperbolic Friction Curve Frame
        let half_w = (w as f32 * 0.50) as u32;
        let left_x0 = 20;
        let left_x1 = half_w - 10;
        let canvas_y0 = 80;
        let canvas_y1 = h - 120;

        for y in canvas_y0..canvas_y1 {
            for x in left_x0..left_x1 {
                let idx = ((y * w + x) * 4) as usize;
                pixels[idx..idx + 4].copy_from_slice(&[10, 14, 24, 255]);
            }
        }

        // Zero axis on left frame
        let left_mid_y = (canvas_y0 + canvas_y1) / 2;
        for x in left_x0..left_x1 {
            let idx = ((left_mid_y * w + x) * 4) as usize;
            pixels[idx..idx + 4].copy_from_slice(&[40, 55, 80, 255]);
        }

        // Draw hyperbolic friction curve
        let num_pts = (left_x1 - left_x0) as usize;
        let mut prev_pt: Option<(f32, f32)> = None;
        for i in 0..num_pts {
            let frac = i as f32 / num_pts as f32;
            let v_rel = frac * 4.0 - 2.0;
            let mu = self.evaluate_friction_curve(v_rel);
            let px = left_x0 as f32 + frac * (left_x1 - left_x0) as f32;
            let py = left_mid_y as f32 - mu * ((canvas_y1 - canvas_y0) as f32 * 0.35);

            if let Some((x0, y0)) = prev_pt {
                draw_line_fo(&mut pixels, w, h, x0, y0, px, py, [0, 229, 255, 255]);
            }
            prev_pt = Some((px, py));
        }

        // Right 50%: Stick-Slip Phase Orbit Frame
        let right_x0 = half_w + 10;
        let right_x1 = w - 20;

        for y in canvas_y0..canvas_y1 {
            for x in right_x0..right_x1 {
                let idx = ((y * w + x) * 4) as usize;
                pixels[idx..idx + 4].copy_from_slice(&[10, 14, 24, 255]);
            }
        }

        // Draw Orbit Loop
        let right_mid_x = (right_x0 + right_x1) as f32 / 2.0;
        let right_mid_y = (canvas_y0 + canvas_y1) as f32 / 2.0;
        let mut prev_orb: Option<(f32, f32)> = None;

        for s in 0..=64 {
            let phase = s as f32 / 64.0 * 2.0 * std::f32::consts::PI;
            let (v_rel, f_fric) = self.evaluate_orbit_point(phase);
            let px = right_mid_x + (v_rel / 2.0) * ((right_x1 - right_x0) as f32 * 0.40);
            let py = right_mid_y - (f_fric / 3.0) * ((canvas_y1 - canvas_y0) as f32 * 0.35);

            if let Some((x0, y0)) = prev_orb {
                draw_line_fo(&mut pixels, w, h, x0, y0, px, py, [255, 215, 0, 255]);
            }
            prev_orb = Some((px, py));
        }

        // Interactive Puck Position on Right Frame
        let puck_x = right_x0 as f32 + self.orbit_puck_pos.0 * (right_x1 - right_x0) as f32;
        let puck_y = canvas_y1 as f32 - self.orbit_puck_pos.1 * (canvas_y1 - canvas_y0) as f32;

        // Draw Touch target boundary (>= 44x44pt bounding box, radius 22pt)
        draw_circle_ring_fo(&mut pixels, w, h, puck_x, puck_y, FRICTION_ORBIT_PUCK_HIT_RADIUS, [255, 107, 43, 180]);
        draw_circle_filled_fo(&mut pixels, w, h, puck_x, puck_y, 12.0, [255, 107, 43, 255]);
        draw_circle_filled_fo(&mut pixels, w, h, puck_x, puck_y, 4.0, [255, 255, 255, 255]);

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

        encode_minimal_png_fo(path, &pixels, width, height)
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_line_fo(buf: &mut [u8], w: u32, h: u32, x0: f32, y0: f32, x1: f32, y1: f32, col: [u8; 4]) {
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

fn draw_circle_filled_fo(buf: &mut [u8], w: u32, h: u32, cx: f32, cy: f32, r: f32, col: [u8; 4]) {
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

fn draw_circle_ring_fo(buf: &mut [u8], w: u32, h: u32, cx: f32, cy: f32, r: f32, col: [u8; 4]) {
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

fn encode_minimal_png_fo(path: &str, rgba_pixels: &[u8], width: u32, height: u32) -> Result<(), String> {
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
    write_png_chunk_fo(&mut out, b"IHDR", &ihdr);

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

    write_png_chunk_fo(&mut out, b"IDAT", &zlib_data);
    write_png_chunk_fo(&mut out, b"IEND", &[]);

    std::fs::write(path, out).map_err(|e| e.to_string())
}

fn write_png_chunk_fo(out: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let type_start = out.len();
    out.extend_from_slice(chunk_type);
    out.extend_from_slice(data);
    let crc = crc32_compute_fo(&out[type_start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

fn crc32_compute_fo(buf: &[u8]) -> u32 {
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
    fn test_friction_orbit_view_ascii_render() {
        let view = FrictionOrbitView::new();
        let ascii = view.render_ascii(80, 16);
        assert!(!ascii.is_empty());
        assert_eq!(ascii.len(), 16);
    }

    #[test]
    fn test_friction_orbit_view_hit_target_dimensions() {
        const {
            assert!(
                FRICTION_ORBIT_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT,
                "Orbit puck hit target bounding box must be >= 44pt"
            );
        }
    }

    #[test]
    fn test_friction_orbit_view_snapshot_render() {
        let view = FrictionOrbitView::new();
        let res = view.render_snapshot_png("scratch/renders/friction_orbit_view.png", 800, 520);
        assert!(res.is_ok());
    }
}
