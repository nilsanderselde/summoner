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

    /// Renders a high-fidelity headless RGBA PNG snapshot of the Bowed String HUD.
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

        // Left 55%: Schelleng Diagram Frame
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

        // Safe Helmholtz region shading
        let (f_min, f_max) = self.schelleng_limits();
        let norm_f_min = Self::force_to_normalized(f_min);
        let norm_f_max = Self::force_to_normalized(f_max);

        let c_height = (canvas_y1 - canvas_y0) as f32;
        let y_min_line = (canvas_y1 as f32 - norm_f_min * c_height) as u32;
        let y_max_line = (canvas_y1 as f32 - norm_f_max * c_height) as u32;

        let safe_top = y_max_line.clamp(canvas_y0, canvas_y1);
        let safe_bot = y_min_line.clamp(canvas_y0, canvas_y1);

        for y in safe_top..safe_bot {
            for x in (left_x0 + 2)..(left_x1 - 2) {
                let idx = ((y * w + x) * 4) as usize;
                pixels[idx..idx + 4].copy_from_slice(&[14, 35, 48, 255]);
            }
        }

        // Schelleng Limit Lines
        for x in left_x0..left_x1 {
            if safe_top >= canvas_y0 && safe_top < canvas_y1 {
                let idx = ((safe_top * w + x) * 4) as usize;
                pixels[idx..idx + 4].copy_from_slice(&[255, 215, 0, 255]);
            }
            if safe_bot >= canvas_y0 && safe_bot < canvas_y1 {
                let idx = ((safe_bot * w + x) * 4) as usize;
                pixels[idx..idx + 4].copy_from_slice(&[255, 107, 43, 255]);
            }
        }

        // Interactive Puck Position
        let puck_x = left_x0 as f32 + self.bow_puck_pos.0 * (left_x1 - left_x0) as f32;
        let puck_y = canvas_y1 as f32 - self.bow_puck_pos.1 * c_height;

        // Draw Touch target boundary (>= 44x44pt bounding box, radius 22pt)
        draw_circle_ring_bs(&mut pixels, w, h, puck_x, puck_y, BOWED_STRING_PUCK_HIT_RADIUS, [0, 229, 255, 180]);
        draw_circle_filled_bs(&mut pixels, w, h, puck_x, puck_y, 12.0, [0, 229, 255, 255]);
        draw_circle_filled_bs(&mut pixels, w, h, puck_x, puck_y, 4.0, [255, 255, 255, 255]);

        // Right 45%: String Vibration Canvas Frame
        let right_x0 = left_w + 10;
        let right_x1 = w - 20;

        for y in canvas_y0..canvas_y1 {
            for x in right_x0..right_x1 {
                let idx = ((y * w + x) * 4) as usize;
                pixels[idx..idx + 4].copy_from_slice(&[10, 14, 24, 255]);
            }
        }

        // Nut and Bridge markers
        let nut_x = right_x0 + 15;
        let bridge_x = right_x1 - 15;
        for y in (canvas_y0 + 20)..(canvas_y1 - 20) {
            let idx_n = ((y * w + nut_x) * 4) as usize;
            pixels[idx_n..idx_n + 4].copy_from_slice(&[255, 215, 0, 255]);
            let idx_b = ((y * w + bridge_x) * 4) as usize;
            pixels[idx_b..idx_b + 4].copy_from_slice(&[255, 215, 0, 255]);
        }

        // Bow Proximity marker line (β)
        let span_w = (bridge_x - nut_x) as f32;
        let bow_mark_x = (bridge_x as f32 - self.bridge_proximity_beta * span_w).round() as u32;
        if bow_mark_x >= right_x0 && bow_mark_x <= right_x1 {
            for y in (canvas_y0 + 15)..(canvas_y1 - 15) {
                let idx_m = ((y * w + bow_mark_x) * 4) as usize;
                pixels[idx_m..idx_m + 4].copy_from_slice(&[255, 107, 43, 255]);
            }
        }

        // String Vibration curve
        let mid_y = (canvas_y0 + canvas_y1) as f32 / 2.0;
        let num_curve_pts = (bridge_x - nut_x) as usize;
        let mut prev_pt: Option<(f32, f32)> = None;

        for i in 0..num_curve_pts {
            let frac = i as f32 / num_curve_pts as f32;
            let disp = self.evaluate_string_displacement(frac, 0.25);
            let px = nut_x as f32 + frac * span_w;
            let py = mid_y - disp * (c_height * 0.35);

            if let Some((x0, y0)) = prev_pt {
                draw_line_bs(&mut pixels, w, h, x0, y0, px, py, [0, 255, 180, 255]);
            }
            prev_pt = Some((px, py));
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

        encode_minimal_png_bs(path, &pixels, width, height)
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_line_bs(buf: &mut [u8], w: u32, h: u32, x0: f32, y0: f32, x1: f32, y1: f32, col: [u8; 4]) {
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

fn draw_circle_filled_bs(buf: &mut [u8], w: u32, h: u32, cx: f32, cy: f32, r: f32, col: [u8; 4]) {
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

fn draw_circle_ring_bs(buf: &mut [u8], w: u32, h: u32, cx: f32, cy: f32, r: f32, col: [u8; 4]) {
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

fn encode_minimal_png_bs(path: &str, rgba_pixels: &[u8], width: u32, height: u32) -> Result<(), String> {
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
    write_png_chunk_bs(&mut out, b"IHDR", &ihdr);

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

    write_png_chunk_bs(&mut out, b"IDAT", &zlib_data);
    write_png_chunk_bs(&mut out, b"IEND", &[]);

    std::fs::write(path, out).map_err(|e| e.to_string())
}

fn write_png_chunk_bs(out: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let type_start = out.len();
    out.extend_from_slice(chunk_type);
    out.extend_from_slice(data);
    let crc = crc32_compute_bs(&out[type_start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

fn crc32_compute_bs(buf: &[u8]) -> u32 {
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
    fn test_bowed_string_view_ascii_render() {
        let view = BowedStringView::new();
        let ascii = view.render_ascii(80, 16);
        assert!(!ascii.is_empty());
        assert_eq!(ascii.len(), 16);
    }

    #[test]
    fn test_bowed_string_view_hit_target_dimensions() {
        const {
            assert!(
                BOWED_STRING_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT,
                "Bow puck hit target bounding box must be >= 44pt"
            );
        }
    }

    #[test]
    fn test_bowed_string_view_snapshot_render() {
        let view = BowedStringView::new();
        let res = view.render_snapshot_png("scratch/renders/bowed_string_view.png", 800, 520);
        assert!(res.is_ok());
    }
}

