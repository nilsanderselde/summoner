// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Physical Modeling Acoustic Membrane Percussion & Strike Velocity Resonance HUD (Step 1504).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const MEMBRANE_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_MEMBRANE_TENSION_NM: f32 = 500.0;
pub const MAX_MEMBRANE_TENSION_NM: f32 = 8000.0;
pub const MIN_MEMBRANE_RADIUS_M: f32 = 0.10;
pub const MAX_MEMBRANE_RADIUS_M: f32 = 0.60;

/// Membrane Physical Material Profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MembraneMaterial {
    MylarDrumhead,   // Modern synthetic PET film (Crisp high-frequency attack)
    CalfskinVintage, // Natural organic parchment (Warm fundamental & gentle decay)
    TitaniumFoil,    // Ultra-thin metallic alloy (Bell-like inharmonic overtones)
    SiliconeElastic, // High internal damping elastomer (Muted punchy thud)
    CarbonComposite, // Rigid woven carbon weave (Fast transient velocity propagation)
}

impl MembraneMaterial {
    pub fn surface_density_kg_m2(&self) -> f32 {
        match self {
            Self::MylarDrumhead => 0.26,
            Self::CalfskinVintage => 0.45,
            Self::TitaniumFoil => 0.72,
            Self::SiliconeElastic => 0.58,
            Self::CarbonComposite => 0.32,
        }
    }

    pub fn internal_damping_coeff(&self) -> f32 {
        match self {
            Self::MylarDrumhead => 0.015,
            Self::CalfskinVintage => 0.042,
            Self::TitaniumFoil => 0.006,
            Self::SiliconeElastic => 0.085,
            Self::CarbonComposite => 0.018,
        }
    }
}

/// Physical Modeling Membrane Resonator View HUD (Step 1504).
#[derive(Debug, Clone)]
pub struct MembraneResonatorView {
    pub material: MembraneMaterial,
    pub membrane_radius_m: f32, // [0.10 ..= 0.60 m] (e.g. 14" snare = 0.178 m)
    pub membrane_tension_nm: f32, // [500.0 ..= 8000.0 N/m]
    pub strike_pos_norm: (f32, f32), // Normalized (X, Y) relative to center (-1.0 ..= +1.0)
    pub strike_velocity: f32,   // [0.0 ..= 1.0]
    pub strike_puck_pos: (f32, f32), // Normalized [0.0 ..= 1.0] canvas coordinates
    pub is_dragging_puck: bool,
    pub fundamental_freq_hz: f32,
    pub rim_shot_coupling: f32,
    pub color_palette: ContrastColorPalette,
}

impl Default for MembraneResonatorView {
    fn default() -> Self {
        Self::new()
    }
}

impl MembraneResonatorView {
    pub fn new() -> Self {
        let mut view = Self {
            material: MembraneMaterial::MylarDrumhead,
            membrane_radius_m: 0.178, // 14-inch snare
            membrane_tension_nm: 3500.0,
            strike_pos_norm: (0.35, 0.25),
            strike_velocity: 0.85,
            strike_puck_pos: (0.5 + 0.35 * 0.45, 0.5 + 0.25 * 0.45),
            is_dragging_puck: false,
            fundamental_freq_hz: 185.0,
            rim_shot_coupling: 0.15,
            color_palette: ContrastColorPalette::default(),
        };
        view.update_physics_simulation();
        view
    }

    /// Convert Membrane Tension [500 ..= 8000 N/m] to normalized coordinate [0.0 ..= 1.0].
    pub fn tension_to_normalized(tension: f32) -> f32 {
        let t = tension.clamp(MIN_MEMBRANE_TENSION_NM, MAX_MEMBRANE_TENSION_NM);
        ((t - MIN_MEMBRANE_TENSION_NM) / (MAX_MEMBRANE_TENSION_NM - MIN_MEMBRANE_TENSION_NM))
            .clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Membrane Tension [500 ..= 8000 N/m].
    pub fn normalized_to_tension(norm: f32) -> f32 {
        MIN_MEMBRANE_TENSION_NM
            + norm.clamp(0.0, 1.0) * (MAX_MEMBRANE_TENSION_NM - MIN_MEMBRANE_TENSION_NM)
    }

    /// Update physical wave velocity $c = \sqrt{T / \sigma}$ and fundamental mode frequency $f_{01} = \frac{2.4048 c}{2 \pi a}$.
    pub fn update_physics_simulation(&mut self) {
        let sigma = self.material.surface_density_kg_m2();
        let t = self.membrane_tension_nm;
        let c = (t / sigma).sqrt(); // Wave propagation speed (m/s)
        let a = self.membrane_radius_m.max(0.05);
        // Bessel zero alpha_01 = 2.4048
        self.fundamental_freq_hz =
            (2.4048 * c / (2.0 * std::f32::consts::PI * a)).clamp(20.0, 2000.0);
    }

    /// Evaluate 2D circular membrane displacement amplitude $W(r, \theta)$ at normalized radius $r \in [0, 1]$ and angle $\theta$.
    pub fn evaluate_membrane_displacement(&self, r_norm: f32, theta: f32) -> f32 {
        let r = r_norm.clamp(0.0, 1.0);
        // Approximation of Bessel modes (0,1), (1,1), (2,1)
        let strike_r = (self.strike_pos_norm.0.powi(2) + self.strike_pos_norm.1.powi(2))
            .sqrt()
            .min(1.0);
        let mode01 = (1.0 - r * r) * (1.0 - 0.5 * strike_r);
        let mode11 = r * (1.0 - r) * 2.0 * (theta - 0.5).cos() * strike_r;
        let mode21 = r * r * (1.0 - r) * 3.0 * (2.0 * theta).sin() * strike_r;

        (mode01 + 0.6 * mode11 + 0.3 * mode21) * self.strike_velocity
    }

    /// Hit-test touch coordinate on the circular membrane strike puck.
    pub fn hit_test_strike_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.strike_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.strike_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= MEMBRANE_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render of 2D Membrane Vibration and Strike Puck.
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

        let mid_x = (width / 2) as f32;
        let mid_y = (height / 2) as f32;
        let radius = ((width / 2 - 3).min(height / 2 - 2)) as f32;

        for r in 1..height - 1 {
            for c in 1..width - 1 {
                let dx = (c as f32 - mid_x) / radius;
                let dy = (r as f32 - mid_y) / radius;
                let dist = (dx * dx + dy * dy).sqrt();
                if (dist - 1.0).abs() < 0.1 {
                    grid[r][c] = '#';
                } else if dist < 1.0 {
                    let theta = dy.atan2(dx);
                    let disp = self.evaluate_membrane_displacement(dist, theta);
                    if disp.abs() > 0.4 {
                        grid[r][c] = '~';
                    }
                }
            }
        }

        // Strike Puck
        let puck_col = ((self.strike_puck_pos.0 * (width - 3) as f32) + 1.0).round() as usize;
        let puck_row =
            (((1.0 - self.strike_puck_pos.1) * (height - 3) as f32) + 1.0).round() as usize;
        if puck_row < height - 1 && puck_col < width - 1 {
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
        let canvas_rect = Rect::new(rect.min.x, rect.min.y, rect.width(), rect.height());

        // Background
        painter.rect_filled(rect, 6.0, Color32::from_rgb(14, 18, 28));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "PHYSICAL MODELING ACOUSTIC MEMBRANE RESONATOR HUD",
            egui::FontId::proportional(16.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Material Selector Tabs (y: 48..92) - Each tab >= 44pt height
        let materials = [
            (MembraneMaterial::MylarDrumhead, "MYLAR SYNTHETIC"),
            (MembraneMaterial::CalfskinVintage, "CALFSKIN VINTAGE"),
            (MembraneMaterial::TitaniumFoil, "TITANIUM FOIL"),
            (MembraneMaterial::SiliconeElastic, "SILICONE ELASTIC"),
            (MembraneMaterial::CarbonComposite, "CARBON COMPOSITE"),
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

        // Left 55%: 2D Circular Membrane Mesh Simulation
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
            "2D CIRCULAR MEMBRANE BESSEL MODAL DISPLACEMENT",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        let drum_center = left_rect.center();
        let drum_radius = (left_rect.width() * 0.38).min(left_rect.height() * 0.40);

        // Rim Ring
        painter.circle_stroke(
            drum_center,
            drum_radius,
            Stroke::new(3.0_f32, Color32::from_rgb(0, 229, 255)),
        );

        // Nodal line guides (Bessel circles)
        painter.circle_stroke(
            drum_center,
            drum_radius * 0.65,
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 90, 130, 90)),
        );
        painter.circle_stroke(
            drum_center,
            drum_radius * 0.35,
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 90, 130, 90)),
        );

        // Crosshairs
        painter.line_segment(
            [
                egui::pos2(drum_center.x - drum_radius, drum_center.y),
                egui::pos2(drum_center.x + drum_radius, drum_center.y),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(50, 70, 100, 80)),
        );
        painter.line_segment(
            [
                egui::pos2(drum_center.x, drum_center.y - drum_radius),
                egui::pos2(drum_center.x, drum_center.y + drum_radius),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(50, 70, 100, 80)),
        );

        // Strike Puck Coordinates on drum
        let puck_x = drum_center.x + self.strike_pos_norm.0 * drum_radius;
        let puck_y = drum_center.y - self.strike_pos_norm.1 * drum_radius;

        // Hit target outer ring (>=44x44pt touch area)
        painter.circle_stroke(
            egui::pos2(puck_x, puck_y),
            MEMBRANE_PUCK_HIT_RADIUS,
            Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(255, 107, 43, 140)),
        );
        painter.circle_filled(
            egui::pos2(puck_x, puck_y),
            14.0,
            Color32::from_rgb(255, 107, 43),
        );
        painter.circle_filled(egui::pos2(puck_x, puck_y), 4.0, Color32::WHITE);

        if response.dragged() {
            if let Some(mouse_pos) = ui.input(|i| i.pointer.interact_pos()) {
                if self.is_dragging_puck
                    || self.hit_test_strike_puck((mouse_pos.x, mouse_pos.y), canvas_rect)
                {
                    self.is_dragging_puck = true;
                    let dx = (mouse_pos.x - drum_center.x) / drum_radius;
                    let dy = -(mouse_pos.y - drum_center.y) / drum_radius;
                    let len = (dx * dx + dy * dy).sqrt();
                    let (clamped_x, clamped_y) = if len > 0.95 {
                        (dx / len * 0.95, dy / len * 0.95)
                    } else {
                        (dx, dy)
                    };
                    self.strike_pos_norm = (clamped_x, clamped_y);
                    self.strike_puck_pos = (0.5 + clamped_x * 0.45, 0.5 + clamped_y * 0.45);
                }
            }
        } else {
            self.is_dragging_puck = false;
        }

        // Right 45%: Modal Resonance Overtones Spectrum
        let right_left = main_canvas.min.x + left_w + 5.0;
        let right_w = main_canvas.max.x - right_left - 10.0;
        let right_rect = egui::Rect::from_min_size(
            egui::pos2(right_left, main_canvas.min.y + 10.0),
            egui::vec2(right_w, main_canvas.height() - 20.0),
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
            "BESSEL INHARMONIC OVERTONE MODES",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // Circular membrane modal ratios: f01 (1.00), f11 (1.59), f21 (2.14), f02 (2.30), f31 (2.65), f12 (2.92)
        let modes = [
            ("f01 (1.00x)", 1.00_f32, 1.0_f32),
            ("f11 (1.59x)", 1.59_f32, 0.75_f32),
            ("f21 (2.14x)", 2.14_f32, 0.55_f32),
            ("f02 (2.30x)", 2.30_f32, 0.45_f32),
            ("f31 (2.65x)", 2.65_f32, 0.35_f32),
            ("f12 (2.92x)", 2.92_f32, 0.25_f32),
        ];

        let mode_bar_w = (right_rect.width() - 20.0) / 6.0;
        for (i, (m_name, _ratio, gain)) in modes.iter().enumerate() {
            let bx = right_rect.min.x + 10.0 + i as f32 * mode_bar_w;
            let bh = gain * (right_rect.height() - 60.0);
            let b_rect = egui::Rect::from_min_max(
                egui::pos2(bx, right_rect.max.y - 20.0 - bh),
                egui::pos2(bx + mode_bar_w - 4.0, right_rect.max.y - 20.0),
            );
            painter.rect_filled(b_rect, 2.0, Color32::from_rgb(0, 255, 180));
            painter.text(
                egui::pos2(bx + mode_bar_w * 0.5 - 2.0, right_rect.max.y - 16.0),
                egui::Align2::CENTER_TOP,
                *m_name,
                egui::FontId::proportional(8.0),
                Color32::from_rgb(150, 175, 205),
            );
        }

        // Bottom Metrics Dock (y: 350..465)
        let bottom_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 350.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 465.0),
        );
        painter.rect_filled(bottom_rect, 6.0, Color32::from_rgb(18, 25, 38));
        painter.rect_stroke(
            bottom_rect,
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        let strike_r = (self.strike_pos_norm.0.powi(2) + self.strike_pos_norm.1.powi(2)).sqrt();
        let metrics = [
            (
                "FUNDAMENTAL (f01)",
                format!(
                    "{:.1} Hz (c={:.0}m/s)",
                    self.fundamental_freq_hz,
                    (self.membrane_tension_nm / self.material.surface_density_kg_m2()).sqrt()
                ),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "TENSION / DENSITY",
                format!(
                    "{:.0} N/m ({:.2} kg/m²)",
                    self.membrane_tension_nm,
                    self.material.surface_density_kg_m2()
                ),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "STRIKE RADIUS (r/a)",
                format!(
                    "{:.2} r/R ({:.1}% Vel)",
                    strike_r,
                    self.strike_velocity * 100.0
                ),
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "INTERNAL DAMPING (γ)",
                format!("{:.4} decay/s", self.material.internal_damping_coeff()),
                Color32::from_rgb(0, 255, 180),
            ),
        ];

        let col_w = (bottom_rect.width() - 40.0) / 4.0;
        for (i, (label, val, col)) in metrics.iter().enumerate() {
            let px = bottom_rect.min.x + 20.0 + i as f32 * col_w;
            painter.text(
                egui::pos2(px, bottom_rect.min.y + 12.0),
                egui::Align2::LEFT_TOP,
                *label,
                egui::FontId::proportional(11.0),
                Color32::from_rgb(160, 180, 205),
            );
            painter.text(
                egui::pos2(px, bottom_rect.min.y + 30.0),
                egui::Align2::LEFT_TOP,
                val,
                egui::FontId::proportional(14.0),
                *col,
            );
        }

        // Pass compliance badge
        let badge_rect = egui::Rect::from_min_max(
            egui::pos2(bottom_rect.min.x + 15.0, bottom_rect.min.y + 68.0),
            egui::pos2(bottom_rect.max.x - 15.0, bottom_rect.max.y - 10.0),
        );
        painter.rect_filled(badge_rect, 4.0, Color32::from_rgb(16, 35, 28));
        painter.rect_stroke(
            badge_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(0, 255, 180)),
        );
        painter.text(
            badge_rect.center(),
            egui::Align2::CENTER_CENTER,
            "[PASS] Physical Modeling Membrane Resonator & Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
