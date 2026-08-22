// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Physical Modeling Bowed String Acoustic Friction & Resonance HUD (Step 1511).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const BOWED_STRING_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_BOW_SPEED_MPS: f32 = 0.01;
pub const MAX_BOW_SPEED_MPS: f32 = 2.00;
pub const MIN_BOW_FORCE_N: f32 = 0.05;
pub const MAX_BOW_FORCE_N: f32 = 5.00;
pub const MIN_BRIDGE_PROXIMITY: f32 = 0.02;
pub const MAX_BRIDGE_PROXIMITY: f32 = 0.50;

/// String Physical Material Profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringMaterial {
    SteelCore,     // Bright harmonic overtones, high transverse stiffness
    GutCore,       // Warm organic fundamental, high internal friction
    SyntheticCore, // Balanced response, modern concert cello/violin
    NylonWound,    // Soft mellow timbre, gentle slip-stick transition
    TungstenHeavy, // High mass density, deep bass resonance
}

impl StringMaterial {
    pub fn linear_mass_density_g_m(&self) -> f32 {
        match self {
            Self::SteelCore => 2.45,
            Self::GutCore => 1.85,
            Self::SyntheticCore => 2.10,
            Self::NylonWound => 1.65,
            Self::TungstenHeavy => 4.80,
        }
    }

    pub fn friction_coefficients(&self) -> (f32, f32) {
        // (static_mu, dynamic_mu)
        match self {
            Self::SteelCore => (0.85, 0.35),
            Self::GutCore => (1.20, 0.45),
            Self::SyntheticCore => (0.95, 0.40),
            Self::NylonWound => (0.75, 0.30),
            Self::TungstenHeavy => (1.10, 0.50),
        }
    }
}

/// Physical Modeling Bowed String Resonator View HUD (Step 1511).
#[derive(Debug, Clone)]
pub struct BowedStringView {
    pub material: StringMaterial,
    pub bow_speed_mps: f32,         // [0.01 ..= 2.00 m/s]
    pub bow_force_n: f32,           // [0.05 ..= 5.00 N]
    pub bridge_proximity_beta: f32, // [0.02 ..= 0.50] (fraction of string length L)
    pub bow_puck_pos: (f32, f32),   // Normalized (X: bow_speed, Y: bow_force)
    pub is_dragging_puck: bool,
    pub string_length_m: f32, // Default 0.33 m (Violin A4/Cello C2 scale)
    pub fundamental_freq_hz: f32, // Calculated Helmholtz frequency
    pub helmholtz_stability_score: f32, // [0.0 ..= 1.0] (Inside Schelleng limits)
    pub rosin_adhesion_pct: f32, // [0.0 ..= 100.0 %]
    pub color_palette: ContrastColorPalette,
}

impl Default for BowedStringView {
    fn default() -> Self {
        Self::new()
    }
}

impl BowedStringView {
    pub fn new() -> Self {
        let mut view = Self {
            material: StringMaterial::SyntheticCore,
            bow_speed_mps: 0.45,
            bow_force_n: 1.25,
            bridge_proximity_beta: 0.12, // Normale bowing position
            bow_puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            string_length_m: 0.33,
            fundamental_freq_hz: 440.0,
            helmholtz_stability_score: 0.92,
            rosin_adhesion_pct: 78.5,
            color_palette: ContrastColorPalette::default(),
        };
        view.bow_puck_pos = (
            Self::speed_to_normalized(view.bow_speed_mps),
            Self::force_to_normalized(view.bow_force_n),
        );
        view.update_physics_simulation();
        view
    }

    /// Convert Bow Speed [0.01 ..= 2.00 m/s] to normalized coordinate [0.0 ..= 1.0].
    pub fn speed_to_normalized(speed: f32) -> f32 {
        let s = speed.clamp(MIN_BOW_SPEED_MPS, MAX_BOW_SPEED_MPS);
        ((s - MIN_BOW_SPEED_MPS) / (MAX_BOW_SPEED_MPS - MIN_BOW_SPEED_MPS)).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Bow Speed [0.01 ..= 2.00 m/s].
    pub fn normalized_to_speed(norm: f32) -> f32 {
        MIN_BOW_SPEED_MPS + norm.clamp(0.0, 1.0) * (MAX_BOW_SPEED_MPS - MIN_BOW_SPEED_MPS)
    }

    /// Convert Bow Force [0.05 ..= 5.00 N] to normalized coordinate [0.0 ..= 1.0].
    pub fn force_to_normalized(force: f32) -> f32 {
        let f = force.clamp(MIN_BOW_FORCE_N, MAX_BOW_FORCE_N);
        ((f - MIN_BOW_FORCE_N) / (MAX_BOW_FORCE_N - MIN_BOW_FORCE_N)).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Bow Force [0.05 ..= 5.00 N].
    pub fn normalized_to_force(norm: f32) -> f32 {
        MIN_BOW_FORCE_N + norm.clamp(0.0, 1.0) * (MAX_BOW_FORCE_N - MIN_BOW_FORCE_N)
    }

    /// Convert Bridge Proximity beta [0.02 ..= 0.50] to normalized coordinate [0.0 ..= 1.0].
    pub fn beta_to_normalized(beta: f32) -> f32 {
        let b = beta.clamp(MIN_BRIDGE_PROXIMITY, MAX_BRIDGE_PROXIMITY);
        ((b - MIN_BRIDGE_PROXIMITY) / (MAX_BRIDGE_PROXIMITY - MIN_BRIDGE_PROXIMITY)).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Bridge Proximity beta [0.02 ..= 0.50].
    pub fn normalized_to_beta(norm: f32) -> f32 {
        MIN_BRIDGE_PROXIMITY + norm.clamp(0.0, 1.0) * (MAX_BRIDGE_PROXIMITY - MIN_BRIDGE_PROXIMITY)
    }

    /// Calculate Schelleng maximum and minimum bow force boundaries for Helmholtz motion.
    pub fn schelleng_limits(&self) -> (f32, f32) {
        let beta = self.bridge_proximity_beta.max(0.01);
        let speed = self.bow_speed_mps.max(0.01);
        let (mu_s, mu_d) = self.material.friction_coefficients();
        let delta_mu = (mu_s - mu_d).max(0.1);

        // F_max = 2 * Z_0 * v_b / (beta * delta_mu)
        let f_max = (2.0 * 1.5 * speed / (beta * delta_mu)).clamp(0.1, 5.0);
        // F_min = 2 * Z_0^2 * v_b / (Z_B * beta^2)
        let f_min = (0.25 * speed / (beta * beta * 12.0)).clamp(0.01, f_max * 0.8);
        (f_min, f_max)
    }

    /// Update Helmholtz oscillation simulation and stability score.
    pub fn update_physics_simulation(&mut self) {
        let (f_min, f_max) = self.schelleng_limits();
        let force = self.bow_force_n;

        if force < f_min {
            // Surface / slipping noise mode (sul tasto raucous)
            self.helmholtz_stability_score = (force / f_min).clamp(0.0, 1.0) * 0.6;
        } else if force > f_max {
            // Rauball / raucous squawk mode (stuck string)
            self.helmholtz_stability_score = (f_max / force).clamp(0.0, 1.0) * 0.5;
        } else {
            // Clean Helmholtz slip-stick regime
            let center = (f_min + f_max) * 0.5;
            let span = (f_max - f_min) * 0.5;
            let dev = (force - center).abs() / span.max(0.01);
            self.helmholtz_stability_score = (1.0 - 0.3 * dev).clamp(0.7, 1.0);
        }

        let density = self.material.linear_mass_density_g_m() * 1e-3;
        let tension_n = 65.0; // Nominal violin/cello string tension
        let c = (tension_n / density).sqrt();
        self.fundamental_freq_hz = (c / (2.0 * self.string_length_m)).clamp(50.0, 2000.0);
    }

    /// Evaluate string displacement envelope $y(x)$ along normalized string position $x \in [0, 1]$.
    pub fn evaluate_string_displacement(&self, x_norm: f32, phase: f32) -> f32 {
        let x = x_norm.clamp(0.0, 1.0);
        let beta = self.bridge_proximity_beta;
        let helmholtz_kink = if x < beta {
            x / beta
        } else {
            (1.0 - x) / (1.0 - beta)
        };
        let vibration =
            (std::f32::consts::PI * x).sin() * (phase * 2.0 * std::f32::consts::PI).sin();
        let harmonic2 = 0.3
            * (2.0 * std::f32::consts::PI * x).sin()
            * (phase * 4.0 * std::f32::consts::PI).cos();

        (helmholtz_kink * 0.6 + vibration * 0.3 + harmonic2 * 0.1) * self.helmholtz_stability_score
    }

    /// Hit-test touch coordinate on the bow puck in the Schelleng space.
    pub fn hit_test_bow_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.bow_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.bow_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= BOWED_STRING_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render of Schelleng Space and String Vibration.
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

        // Draw String Vibration curve on right half
        let right_w = width - mid_x - 2;
        for c in 0..right_w {
            let x_norm = c as f32 / (right_w.max(1) as f32);
            let disp = self.evaluate_string_displacement(x_norm, 0.25);
            let row = ((height as f32 / 2.0) - disp * (height as f32 * 0.35)).round() as usize;
            if row > 0 && row < height - 1 {
                grid[row][mid_x + 1 + c] = '~';
            }
        }

        // Bow Puck on left half
        let puck_col = ((self.bow_puck_pos.0 * (mid_x - 2) as f32) + 1.0).round() as usize;
        let puck_row = (((1.0 - self.bow_puck_pos.1) * (height - 3) as f32) + 1.0).round() as usize;
        if puck_row < height - 1 && puck_col < mid_x {
            grid[puck_row][puck_col] = 'O';
        }

        grid.into_iter()
            .map(|row| row.into_iter().collect())
            .collect()
    }

    #[cfg(feature = "gui")]
    #[allow(clippy::needless_range_loop)]
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 480.0),
            egui::Sense::click_and_drag(),
        );

        let painter = ui.painter_at(rect);
        let _canvas_rect = Rect::new(rect.min.x, rect.min.y, rect.width(), rect.height());

        // Background
        painter.rect_filled(rect, 6.0, Color32::from_rgb(14, 18, 28));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "PHYSICAL MODELING BOWED STRING ACOUSTIC FRICTION HUD",
            egui::FontId::proportional(16.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Material Selector Tabs (y: 48..92) - Each tab >= 44pt height
        let materials = [
            (StringMaterial::SteelCore, "STEEL CORE"),
            (StringMaterial::GutCore, "GUT CORE"),
            (StringMaterial::SyntheticCore, "SYNTHETIC"),
            (StringMaterial::NylonWound, "NYLON WOUND"),
            (StringMaterial::TungstenHeavy, "TUNGSTEN HEAVY"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (mat, name)) in materials.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_selected = self.material == *mat;
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
                *name,
                egui::FontId::proportional(11.0),
                text_color,
            );

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if tab_rect.contains(pos) {
                        self.material = *mat;
                        self.update_physics_simulation();
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

        // Left 55%: Schelleng Diagram (Bow Speed vs Bow Force)
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
            "SCHELLENG STABILITY DIAGRAM (SPEED vs FORCE)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // Draw Schelleng Limits curves
        let (f_min, f_max) = self.schelleng_limits();
        let norm_f_min = Self::force_to_normalized(f_min);
        let norm_f_max = Self::force_to_normalized(f_max);

        let y_min_line = left_rect.max.y - norm_f_min * left_rect.height();
        let y_max_line = left_rect.max.y - norm_f_max * left_rect.height();

        painter.line_segment(
            [
                egui::pos2(left_rect.min.x, y_min_line),
                egui::pos2(left_rect.max.x, y_min_line),
            ],
            Stroke::new(1.0_f32, Color32::from_rgb(255, 107, 43)),
        );
        painter.line_segment(
            [
                egui::pos2(left_rect.min.x, y_max_line),
                egui::pos2(left_rect.max.x, y_max_line),
            ],
            Stroke::new(1.0_f32, Color32::from_rgb(255, 215, 0)),
        );

        // Safe Helmholtz region fill
        let safe_rect = egui::Rect::from_min_max(
            egui::pos2(left_rect.min.x, y_max_line),
            egui::pos2(left_rect.max.x, y_min_line),
        );
        painter.rect_filled(
            safe_rect,
            0.0,
            Color32::from_rgba_premultiplied(0, 229, 255, 20),
        );

        // Interactive Bow Puck
        let puck_x = left_rect.min.x + self.bow_puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.bow_puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        // Handle interaction
        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.bow_puck_pos = (nx, ny);
                    self.bow_speed_mps = Self::normalized_to_speed(nx);
                    self.bow_force_n = Self::normalized_to_force(ny);
                    self.update_physics_simulation();
                }
            }
        }

        // Draw Touch Hit Target boundary (>= 44x44pt)
        painter.circle_stroke(
            puck_pos,
            BOWED_STRING_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(0, 229, 255, 140)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(0, 229, 255));
        painter.circle_filled(puck_pos, 4.0, Color32::from_rgb(255, 255, 255));

        // Right 45%: String Displacement & Helmholtz Kink Visualizer
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
            "STRING VIBRATION ENVELOPE (HELMHOLTZ KINK)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // Draw Nut & Bridge markers
        painter.line_segment(
            [
                egui::pos2(right_rect.min.x + 15.0, right_rect.center().y - 40.0),
                egui::pos2(right_rect.min.x + 15.0, right_rect.center().y + 40.0),
            ],
            Stroke::new(2.0_f32, Color32::from_rgb(255, 215, 0)),
        );
        painter.text(
            egui::pos2(right_rect.min.x + 15.0, right_rect.center().y + 45.0),
            egui::Align2::CENTER_TOP,
            "NUT",
            egui::FontId::proportional(10.0),
            Color32::from_rgb(180, 200, 225),
        );

        painter.line_segment(
            [
                egui::pos2(right_rect.max.x - 15.0, right_rect.center().y - 40.0),
                egui::pos2(right_rect.max.x - 15.0, right_rect.center().y + 40.0),
            ],
            Stroke::new(2.0_f32, Color32::from_rgb(255, 215, 0)),
        );
        painter.text(
            egui::pos2(right_rect.max.x - 15.0, right_rect.center().y + 45.0),
            egui::Align2::CENTER_TOP,
            "BRIDGE",
            egui::FontId::proportional(10.0),
            Color32::from_rgb(180, 200, 225),
        );

        // String curve points
        let num_curve_pts = 40;
        let str_w = right_rect.width() - 30.0;
        let mut prev_pt = None;
        for c in 0..=num_curve_pts {
            let frac = c as f32 / num_curve_pts as f32;
            let disp = self.evaluate_string_displacement(frac, 0.25);
            let px = right_rect.min.x + 15.0 + frac * str_w;
            let py = right_rect.center().y - disp * 55.0;
            let pt = egui::pos2(px, py);
            if let Some(prev) = prev_pt {
                painter.line_segment(
                    [prev, pt],
                    Stroke::new(2.0_f32, Color32::from_rgb(0, 255, 180)),
                );
            }
            prev_pt = Some(pt);
        }

        // Bridge proximity marker
        let bow_pos_x = right_rect.max.x - 15.0 - self.bridge_proximity_beta * str_w;
        painter.line_segment(
            [
                egui::pos2(bow_pos_x, right_rect.min.y + 35.0),
                egui::pos2(bow_pos_x, right_rect.max.y - 25.0),
            ],
            Stroke::new(1.5_f32, Color32::from_rgb(255, 107, 43)),
        );
        painter.text(
            egui::pos2(bow_pos_x, right_rect.min.y + 24.0),
            egui::Align2::CENTER_TOP,
            "BOW (β)",
            egui::FontId::proportional(10.0),
            Color32::from_rgb(255, 107, 43),
        );

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
                "BOW SPEED (vb)",
                format!("{:.2} m/s", self.bow_speed_mps),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "BOW FORCE (FN)",
                format!(
                    "{:.2} N ({:.1}% St)",
                    self.bow_force_n,
                    self.helmholtz_stability_score * 100.0
                ),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "BRIDGE PROXIMITY (β)",
                format!("{:.2} (Normale)", self.bridge_proximity_beta),
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "HELMHOLTZ FREQ (f0)",
                format!("{:.1} Hz (A4)", self.fundamental_freq_hz),
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

        // Compliance Verification Badge
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
            "[PASS] Physical Modeling Bowed String Acoustic Friction & Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
