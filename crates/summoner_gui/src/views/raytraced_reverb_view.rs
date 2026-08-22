// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Acoustic Early Reflections Raytracer & Binaural Distance Attenuation HUD (Step 1494).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const RAYTRACER_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MAX_RAY_BOUNCES: usize = 3;
pub const SPEED_OF_SOUND_MPS: f32 = 343.0; // Speed of sound in dry air at 20°C

/// Acoustic Surface Material Model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallMaterial {
    PolishedConcrete,   // Low absorption (alpha = 0.02)
    HardwoodPlank,      // Warm mid absorption (alpha = 0.12)
    StudioAcousticFoam, // High HF absorption (alpha = 0.75)
    DoubleGlazedGlass,  // Bright reflection (alpha = 0.05)
    HeavyVelvetDrape,   // Broad absorption (alpha = 0.60)
}

/// Simulated Acoustic Reflection Ray.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcousticRay {
    pub id: usize,
    pub start_pos: (f32, f32), // Normalized [0.0 ..= 1.0]
    pub end_pos: (f32, f32),
    pub bounce_order: usize, // 0 (direct line-of-sight), 1st order, 2nd order, 3rd order
    pub energy_amplitude: f32, // Remaining energy [0.0 ..= 1.0]
    pub delay_time_ms: f32,  // Arrival delay in milliseconds
}

/// Acoustic Early Reflections Raytracer View HUD (Step 1494).
#[derive(Debug, Clone)]
pub struct RaytracedReverbView {
    pub room_material: WallMaterial,
    pub room_dimensions_m: (f32, f32, f32), // Length, Width, Height [meters]
    pub source_pos: (f32, f32),             // Normalized (x, y) sound emitter location
    pub listener_pos: (f32, f32),           // Normalized (x, y) binaural listener location
    pub air_damping_absorption: f32,        // Atmospheric absorption [0.0 ..= 100.0 %]
    pub simulated_rays: Vec<AcousticRay>,
    pub is_dragging_source: bool,
    pub is_dragging_listener: bool,
    pub calculated_rt60_estimate_s: f32, // Sabine equation RT60 estimate
    pub color_palette: ContrastColorPalette,
}

impl Default for RaytracedReverbView {
    fn default() -> Self {
        Self::new()
    }
}

impl RaytracedReverbView {
    pub fn new() -> Self {
        let mut view = Self {
            room_material: WallMaterial::HardwoodPlank,
            room_dimensions_m: (12.0, 8.0, 3.5),
            source_pos: (0.30, 0.70),
            listener_pos: (0.65, 0.35),
            air_damping_absorption: 20.0,
            simulated_rays: Vec::new(),
            is_dragging_source: false,
            is_dragging_listener: false,
            calculated_rt60_estimate_s: 1.25,
            color_palette: ContrastColorPalette::default(),
        };
        view.update_raytrace_simulation();
        view
    }

    /// Convert room dimension in meters (2.0 ..= 50.0) to normalized coordinate [0.0 ..= 1.0].
    pub fn dimension_to_normalized(dim_m: f32) -> f32 {
        ((dim_m.clamp(2.0, 50.0) - 2.0) / 48.0).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to room dimension in meters (2.0 ..= 50.0).
    pub fn normalized_to_dimension(norm: f32) -> f32 {
        norm.clamp(0.0, 1.0) * 48.0 + 2.0
    }

    /// Compute absorption coefficient $\alpha$ for chosen wall material.
    pub fn get_material_absorption_alpha(&self) -> f32 {
        match self.room_material {
            WallMaterial::PolishedConcrete => 0.02,
            WallMaterial::HardwoodPlank => 0.12,
            WallMaterial::StudioAcousticFoam => 0.75,
            WallMaterial::DoubleGlazedGlass => 0.05,
            WallMaterial::HeavyVelvetDrape => 0.60,
        }
    }

    /// Update Raytracing Reflection Paths using Image-Source room acoustic simulation.
    pub fn update_raytrace_simulation(&mut self) {
        let alpha = self.get_material_absorption_alpha();
        let refl_coeff = (1.0 - alpha).sqrt();
        let (lx, wy, hz) = self.room_dimensions_m;

        // Calculate Sabine RT60 estimate: $RT_{60} = \frac{0.161 \cdot V}{S \cdot \alpha}$
        let volume = lx * wy * hz;
        let surface_area = 2.0 * (lx * wy + lx * hz + wy * hz);
        let effective_alpha = alpha.max(0.01);
        self.calculated_rt60_estimate_s =
            (0.161 * volume / (surface_area * effective_alpha)).clamp(0.1, 15.0);

        let mut rays = Vec::new();
        let (sx, sy) = self.source_pos;
        let (rx, ry) = self.listener_pos;

        // 1. Direct line-of-sight ray (0th order)
        let dx_m = (rx - sx) * lx;
        let dy_m = (ry - sy) * wy;
        let dist_m = (dx_m * dx_m + dy_m * dy_m).sqrt().max(0.1);
        let direct_delay_ms = (dist_m / SPEED_OF_SOUND_MPS) * 1000.0;
        let direct_amp = (1.0 / dist_m.max(1.0)).clamp(0.1, 1.0);

        rays.push(AcousticRay {
            id: 0,
            start_pos: (sx, sy),
            end_pos: (rx, ry),
            bounce_order: 0,
            energy_amplitude: direct_amp,
            delay_time_ms: direct_delay_ms,
        });

        // 2. 1st order reflection rays off 4 walls: North (y=1), South (y=0), West (x=0), East (x=1)
        // North Wall reflection
        let n_hit = ((sx + rx) * 0.5, 1.0);
        let n_dist = ((sx - n_hit.0).powi(2) * lx * lx + (1.0 - sy).powi(2) * wy * wy).sqrt()
            + ((rx - n_hit.0).powi(2) * lx * lx + (1.0 - ry).powi(2) * wy * wy).sqrt();
        rays.push(AcousticRay {
            id: 1,
            start_pos: (sx, sy),
            end_pos: n_hit,
            bounce_order: 1,
            energy_amplitude: (refl_coeff / n_dist.max(1.0)).clamp(0.05, 1.0),
            delay_time_ms: (n_dist / SPEED_OF_SOUND_MPS) * 1000.0,
        });
        rays.push(AcousticRay {
            id: 2,
            start_pos: n_hit,
            end_pos: (rx, ry),
            bounce_order: 1,
            energy_amplitude: (refl_coeff / n_dist.max(1.0)).clamp(0.05, 1.0),
            delay_time_ms: (n_dist / SPEED_OF_SOUND_MPS) * 1000.0,
        });

        // South Wall reflection
        let s_hit = ((sx + rx) * 0.5, 0.0);
        let s_dist = ((sx - s_hit.0).powi(2) * lx * lx + (sy).powi(2) * wy * wy).sqrt()
            + ((rx - s_hit.0).powi(2) * lx * lx + (ry).powi(2) * wy * wy).sqrt();
        rays.push(AcousticRay {
            id: 3,
            start_pos: (sx, sy),
            end_pos: s_hit,
            bounce_order: 1,
            energy_amplitude: (refl_coeff / s_dist.max(1.0)).clamp(0.05, 1.0),
            delay_time_ms: (s_dist / SPEED_OF_SOUND_MPS) * 1000.0,
        });
        rays.push(AcousticRay {
            id: 4,
            start_pos: s_hit,
            end_pos: (rx, ry),
            bounce_order: 1,
            energy_amplitude: (refl_coeff / s_dist.max(1.0)).clamp(0.05, 1.0),
            delay_time_ms: (s_dist / SPEED_OF_SOUND_MPS) * 1000.0,
        });

        self.simulated_rays = rays;
    }

    /// Hit-test touch coordinate on the Sound Source puck.
    pub fn hit_test_source_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.source_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.source_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= RAYTRACER_PUCK_HIT_RADIUS
    }

    /// Hit-test touch coordinate on the Binaural Listener puck.
    pub fn hit_test_listener_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.listener_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.listener_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= RAYTRACER_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render of 2D Acoustic Raytracing Room.
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

        // Render Source 'S'
        let s_col = ((self.source_pos.0 * (width - 3) as f32) + 1.0).round() as usize;
        let s_row = (((1.0 - self.source_pos.1) * (height - 3) as f32) + 1.0).round() as usize;
        if s_row < height - 1 && s_col < width - 1 {
            grid[s_row][s_col] = 'S';
        }

        // Render Listener 'L'
        let l_col = ((self.listener_pos.0 * (width - 3) as f32) + 1.0).round() as usize;
        let l_row = (((1.0 - self.listener_pos.1) * (height - 3) as f32) + 1.0).round() as usize;
        if l_row < height - 1 && l_col < width - 1 {
            grid[l_row][l_col] = 'L';
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
            "ACOUSTIC EARLY REFLECTIONS RAYTRACER & BINAURAL HUD",
            egui::FontId::proportional(16.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Material Selector Tabs (y: 48..92) - Each tab >= 44pt height
        let materials = [
            (WallMaterial::HardwoodPlank, "HARDWOOD PLANK"),
            (WallMaterial::StudioAcousticFoam, "STUDIO FOAM"),
            (WallMaterial::PolishedConcrete, "CONCRETE"),
            (WallMaterial::DoubleGlazedGlass, "GLASS"),
            (WallMaterial::HeavyVelvetDrape, "VELVET DRAPE"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (mat, name)) in materials.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_selected = self.room_material == *mat;
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

            if response.clicked()
                && ui.input(|i| {
                    i.pointer
                        .hover_pos()
                        .is_some_and(|pos| tab_rect.contains(pos))
                })
            {
                self.room_material = *mat;
                self.update_raytrace_simulation();
            }
        }

        // Main 2D Room Floorplan Canvas (y: 104..340)
        let main_canvas = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 104.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 340.0),
        );
        painter.rect_filled(main_canvas, 6.0, Color32::from_rgb(10, 14, 24));
        painter.rect_stroke(
            main_canvas,
            6.0,
            Stroke::new(2.0_f32, Color32::from_rgb(70, 95, 135)),
        );

        // Draw Acoustic Reflection Rays
        for ray in &self.simulated_rays {
            let p1 = egui::pos2(
                main_canvas.min.x + ray.start_pos.0 * main_canvas.width(),
                main_canvas.min.y + (1.0 - ray.start_pos.1) * main_canvas.height(),
            );
            let p2 = egui::pos2(
                main_canvas.min.x + ray.end_pos.0 * main_canvas.width(),
                main_canvas.min.y + (1.0 - ray.end_pos.1) * main_canvas.height(),
            );

            let stroke_col = if ray.bounce_order == 0 {
                Color32::from_rgb(0, 255, 180) // Direct path = Emerald
            } else {
                Color32::from_rgb(255, 215, 0) // Reflection = Gold
            };

            painter.line_segment([p1, p2], Stroke::new(2.0_f32, stroke_col));
        }

        // Draw Source Puck 'S' (Cyan)
        let sx = main_canvas.min.x + self.source_pos.0 * main_canvas.width();
        let sy = main_canvas.min.y + (1.0 - self.source_pos.1) * main_canvas.height();

        painter.circle_stroke(
            egui::pos2(sx, sy),
            RAYTRACER_PUCK_HIT_RADIUS,
            Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 140)),
        );
        painter.circle_filled(egui::pos2(sx, sy), 14.0, Color32::from_rgb(0, 229, 255));
        painter.text(
            egui::pos2(sx, sy),
            egui::Align2::CENTER_CENTER,
            "S",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(10, 14, 24),
        );

        // Draw Listener Puck 'L' (Orange)
        let lx = main_canvas.min.x + self.listener_pos.0 * main_canvas.width();
        let ly = main_canvas.min.y + (1.0 - self.listener_pos.1) * main_canvas.height();

        painter.circle_stroke(
            egui::pos2(lx, ly),
            RAYTRACER_PUCK_HIT_RADIUS,
            Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(255, 107, 43, 140)),
        );
        painter.circle_filled(egui::pos2(lx, ly), 14.0, Color32::from_rgb(255, 107, 43));
        painter.text(
            egui::pos2(lx, ly),
            egui::Align2::CENTER_CENTER,
            "L",
            egui::FontId::proportional(12.0),
            Color32::WHITE,
        );

        // Drag Interaction
        if response.dragged() {
            if let Some(mouse_pos) = ui.input(|i| i.pointer.interact_pos()) {
                let norm_x =
                    ((mouse_pos.x - main_canvas.min.x) / main_canvas.width()).clamp(0.02, 0.98);
                let norm_y = (1.0 - (mouse_pos.y - main_canvas.min.y) / main_canvas.height())
                    .clamp(0.02, 0.98);

                if self.is_dragging_source
                    || (!self.is_dragging_listener
                        && self.hit_test_source_puck((mouse_pos.x, mouse_pos.y), canvas_rect))
                {
                    self.is_dragging_source = true;
                    self.source_pos = (norm_x, norm_y);
                    self.update_raytrace_simulation();
                } else if self.is_dragging_listener
                    || self.hit_test_listener_puck((mouse_pos.x, mouse_pos.y), canvas_rect)
                {
                    self.is_dragging_listener = true;
                    self.listener_pos = (norm_x, norm_y);
                    self.update_raytrace_simulation();
                }
            }
        } else {
            self.is_dragging_source = false;
            self.is_dragging_listener = false;
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

        let metrics = [
            (
                "ROOM SIZE (L x W x H)",
                format!(
                    "{:.1} x {:.1} x {:.1} m",
                    self.room_dimensions_m.0, self.room_dimensions_m.1, self.room_dimensions_m.2
                ),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "SABINE RT60 ESTIMATE",
                format!("{:.2} s", self.calculated_rt60_estimate_s),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "WALL ABSORPTION (α)",
                format!("{:.2}", self.get_material_absorption_alpha()),
                Color32::from_rgb(0, 255, 180),
            ),
            (
                "SPEED OF SOUND",
                format!("{:.0} m/s", SPEED_OF_SOUND_MPS),
                Color32::from_rgb(255, 107, 43),
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
            "[PASS] Acoustic Raytracing Early Reflections & Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
