// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// 3D Shoebox Room Acoustics & 6-Wall Early Reflection / Diffuse FDN Reverb HUD (Step 1561).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const SPATIAL_ROOM_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_ROOM_DIM_M: f32 = 2.0;
pub const MAX_ROOM_DIM_M: f32 = 50.0;
pub const MIN_ROOM_HEIGHT_M: f32 = 2.0;
pub const MAX_ROOM_HEIGHT_M: f32 = 20.0;

/// Wall material acoustic absorption preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallMaterialPreset {
    StudioAcoustic, // Balanced acoustic foam & diffusors (alpha ~ 0.35)
    WoodPaneling,   // Warm timber paneling (alpha ~ 0.15)
    Concrete,       // Dense reverberant masonry (alpha ~ 0.04)
    Glass,          // Reflective floor-to-ceiling windows (alpha ~ 0.08)
    HeavyCurtains,  // Absorptive velour drapery (alpha ~ 0.60)
    Plaster,        // Standard dry-wall plaster (alpha ~ 0.05)
    Custom,         // Arbitrary user coefficient override
}

impl WallMaterialPreset {
    pub fn absorption_coefficient(&self) -> f32 {
        match self {
            Self::StudioAcoustic => 0.35,
            Self::WoodPaneling => 0.15,
            Self::Concrete => 0.04,
            Self::Glass => 0.08,
            Self::HeavyCurtains => 0.60,
            Self::Plaster => 0.05,
            Self::Custom => 0.35,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::StudioAcoustic => "Studio Acoustic (LEDE)",
            Self::WoodPaneling => "Wood Paneling",
            Self::Concrete => "Concrete Masonry",
            Self::Glass => "Architectural Glass",
            Self::HeavyCurtains => "Heavy Curtains",
            Self::Plaster => "Drywall Plaster",
            Self::Custom => "Custom Absorption",
        }
    }
}

/// Evaluated early reflection ray from an image source boundary.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EarlyReflectionRay {
    pub name: &'static str,
    pub bounce_point: (f32, f32, f32),
    pub distance_m: f32,
    pub delay_ms: f32,
    pub attenuation_db: f32,
}

/// 3D Shoebox Early Reflection & Diffuse FDN Reverb Acoustic Modeler HUD (Step 1561).
#[derive(Debug, Clone)]
pub struct SpatialRoomView {
    pub room_width_m: f32,  // Room X width [2.0 ..= 50.0 m] (default: 8.0 m)
    pub room_depth_m: f32,  // Room Y depth [2.0 ..= 50.0 m] (default: 12.0 m)
    pub room_height_m: f32, // Room Z height [2.0 ..= 20.0 m] (default: 3.5 m)
    pub source_pos: (f32, f32, f32), // Source 3D position (X, Y, Z) in meters
    pub listener_pos: (f32, f32, f32), // Listener 3D position (X, Y, Z) in meters
    pub listener_yaw_deg: f32, // Listener yaw orientation angle [-180.0 ..= +180.0 deg]
    pub wall_material: WallMaterialPreset,
    pub custom_absorption: f32, // Custom absorption coefficient [0.01 ..= 0.99]
    pub rt60_seconds: f32,      // Sabine estimated reverberation time
    pub direct_gain: f32,       // Direct path gain [0.0 ..= 2.0]
    pub early_gain: f32,        // Early reflections gain [0.0 ..= 2.0]
    pub late_gain: f32,         // FDN late diffuse reverb gain [0.0 ..= 2.0]
    pub damping: f32,           // High frequency absorption damping [0.0 ..= 1.0]
    pub dry_wet: f32,           // Wet blend percentage [0.0 ..= 100.0 %]
    pub speed_of_sound_mps: f32,
    pub is_dragging_source: bool,
    pub is_dragging_listener: bool,
    pub color_palette: ContrastColorPalette,
}

impl Default for SpatialRoomView {
    fn default() -> Self {
        Self::new()
    }
}

impl SpatialRoomView {
    pub fn new() -> Self {
        let mut view = Self {
            room_width_m: 8.0,
            room_depth_m: 12.0,
            room_height_m: 3.5,
            source_pos: (3.0, 4.0, 1.5),
            listener_pos: (4.0, 8.0, 1.7),
            listener_yaw_deg: 0.0,
            wall_material: WallMaterialPreset::StudioAcoustic,
            custom_absorption: 0.35,
            rt60_seconds: 0.65,
            direct_gain: 1.0,
            early_gain: 0.8,
            late_gain: 0.6,
            damping: 0.30,
            dry_wet: 40.0,
            speed_of_sound_mps: 343.0,
            is_dragging_source: false,
            is_dragging_listener: false,
            color_palette: ContrastColorPalette::default(),
        };
        view.update_acoustics();
        view
    }

    /// Room air volume $V = W \times D \times H$ in cubic meters.
    pub fn room_volume(&self) -> f32 {
        self.room_width_m * self.room_depth_m * self.room_height_m
    }

    /// Room total boundary surface area $S = 2(WD + WH + DH)$ in square meters.
    pub fn room_surface_area(&self) -> f32 {
        2.0 * (self.room_width_m * self.room_depth_m
            + self.room_width_m * self.room_height_m
            + self.room_depth_m * self.room_height_m)
    }

    /// Active wall absorption coefficient $\alpha$.
    pub fn active_absorption(&self) -> f32 {
        if self.wall_material == WallMaterialPreset::Custom {
            self.custom_absorption.clamp(0.01, 0.99)
        } else {
            self.wall_material.absorption_coefficient()
        }
    }

    /// Calculate Sabine reverberation time $RT_{60} = \frac{0.161 \times V}{S \times \alpha}$.
    pub fn calculate_sabine_rt60(w: f32, d: f32, h: f32, alpha: f32) -> f32 {
        let v = w * d * h;
        let s = 2.0 * (w * d + w * h + d * h);
        let a = (s * alpha).max(0.1);
        (0.161 * v / a).clamp(0.05, 12.0)
    }

    /// Update calculated Sabine RT60 seconds.
    pub fn update_acoustics(&mut self) {
        let alpha = self.active_absorption();
        self.rt60_seconds = Self::calculate_sabine_rt60(
            self.room_width_m,
            self.room_depth_m,
            self.room_height_m,
            alpha,
        );
    }

    /// Distance between sound source and listener in 3D Cartesian coordinates.
    pub fn direct_distance(&self) -> f32 {
        let dx = self.listener_pos.0 - self.source_pos.0;
        let dy = self.listener_pos.1 - self.source_pos.1;
        let dz = self.listener_pos.2 - self.source_pos.2;
        (dx * dx + dy * dy + dz * dz).sqrt().max(0.1)
    }

    /// Direct sound arrival delay in milliseconds.
    pub fn direct_delay_ms(&self) -> f32 {
        (self.direct_distance() / self.speed_of_sound_mps) * 1000.0
    }

    /// Evaluate 6 first-order image reflections (Left, Right, Back, Front, Floor, Ceiling).
    pub fn evaluate_early_reflections(&self) -> [EarlyReflectionRay; 6] {
        let (sx, sy, sz) = self.source_pos;
        let (lx, ly, lz) = self.listener_pos;
        let w = self.room_width_m;
        let d = self.room_depth_m;
        let h = self.room_height_m;
        let c = self.speed_of_sound_mps;
        let alpha = self.active_absorption();
        let refl_coeff = (1.0 - alpha).clamp(0.05, 0.98);

        // 1. Left Wall (X = 0)
        let dist_left = ((lx + sx).powi(2) + (ly - sy).powi(2) + (lz - sz).powi(2)).sqrt();
        let t_left_x = sx / (sx + lx).max(1e-4);
        let bounce_left = (0.0, sy + (ly - sy) * t_left_x, sz + (lz - sz) * t_left_x);

        // 2. Right Wall (X = W)
        let dist_right = ((2.0 * w - sx - lx).powi(2) + (ly - sy).powi(2) + (lz - sz).powi(2)).sqrt();
        let t_right_x = (w - sx) / (2.0 * w - sx - lx).max(1e-4);
        let bounce_right = (w, sy + (ly - sy) * t_right_x, sz + (lz - sz) * t_right_x);

        // 3. Back Wall (Y = 0)
        let dist_back = ((lx - sx).powi(2) + (ly + sy).powi(2) + (lz - sz).powi(2)).sqrt();
        let t_back_y = sy / (sy + ly).max(1e-4);
        let bounce_back = (sx + (lx - sx) * t_back_y, 0.0, sz + (lz - sz) * t_back_y);

        // 4. Front Wall (Y = D)
        let dist_front = ((lx - sx).powi(2) + (2.0 * d - sy - ly).powi(2) + (lz - sz).powi(2)).sqrt();
        let t_front_y = (d - sy) / (2.0 * d - sy - ly).max(1e-4);
        let bounce_front = (sx + (lx - sx) * t_front_y, d, sz + (lz - sz) * t_front_y);

        // 5. Floor (Z = 0)
        let dist_floor = ((lx - sx).powi(2) + (ly - sy).powi(2) + (lz + sz).powi(2)).sqrt();
        let t_floor_z = sz / (sz + lz).max(1e-4);
        let bounce_floor = (sx + (lx - sx) * t_floor_z, sy + (ly - sy) * t_floor_z, 0.0);

        // 6. Ceiling (Z = H)
        let dist_ceil = ((lx - sx).powi(2) + (ly - sy).powi(2) + (2.0 * h - sz - lz).powi(2)).sqrt();
        let t_ceil_z = (h - sz) / (2.0 * h - sz - lz).max(1e-4);
        let bounce_ceil = (sx + (lx - sx) * t_ceil_z, sy + (ly - sy) * t_ceil_z, h);

        let make_ray = |name, bounce, dist: f32| {
            let delay_ms = (dist / c) * 1000.0;
            let atten = 20.0 * ((refl_coeff / dist.max(0.1)).clamp(0.01, 1.0)).log10();
            EarlyReflectionRay {
                name,
                bounce_point: bounce,
                distance_m: dist,
                delay_ms,
                attenuation_db: atten,
            }
        };

        [
            make_ray("Left Wall", bounce_left, dist_left),
            make_ray("Right Wall", bounce_right, dist_right),
            make_ray("Back Wall", bounce_back, dist_back),
            make_ray("Front Wall", bounce_front, dist_front),
            make_ray("Floor", bounce_floor, dist_floor),
            make_ray("Ceiling", bounce_ceil, dist_ceil),
        ]
    }

    /// Normalized room coordinate mapping (X in 0..W, Y in 0..D) to Canvas pixel position.
    pub fn room_to_canvas_pos(&self, rx: f32, ry: f32, room_rect: egui::Rect) -> egui::Pos2 {
        let norm_x = (rx / self.room_width_m).clamp(0.0, 1.0);
        let norm_y = (ry / self.room_depth_m).clamp(0.0, 1.0);
        egui::pos2(
            room_rect.min.x + norm_x * room_rect.width(),
            room_rect.max.y - norm_y * room_rect.height(), // Y inverted
        )
    }

    /// Hit test source puck (>= 44x44pt bounding target).
    pub fn hit_test_source_puck(&self, pos: (f32, f32), room_rect: egui::Rect) -> bool {
        let p = self.room_to_canvas_pos(self.source_pos.0, self.source_pos.1, room_rect);
        let dx = pos.0 - p.x;
        let dy = pos.1 - p.y;
        (dx * dx + dy * dy).sqrt() <= SPATIAL_ROOM_PUCK_HIT_RADIUS
    }

    /// Hit test listener puck (>= 44x44pt bounding target).
    pub fn hit_test_listener_puck(&self, pos: (f32, f32), room_rect: egui::Rect) -> bool {
        let p = self.room_to_canvas_pos(self.listener_pos.0, self.listener_pos.1, room_rect);
        let dx = pos.0 - p.x;
        let dy = pos.1 - p.y;
        (dx * dx + dy * dy).sqrt() <= SPATIAL_ROOM_PUCK_HIT_RADIUS
    }

    /// Render deterministic ASCII representation for headless terminal testing.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let header = format!(
            "SHOEBOX ROOM [{}] {}x{}x{}m RT60:{:.2}s Src:({:.1},{:.1})m Lis:({:.1},{:.1})m",
            self.wall_material.display_name(),
            self.room_width_m,
            self.room_depth_m,
            self.room_height_m,
            self.rt60_seconds,
            self.source_pos.0,
            self.source_pos.1,
            self.listener_pos.0,
            self.listener_pos.1
        );
        lines.push(header);

        let canvas_h = height.saturating_sub(2);
        for y in 0..canvas_h {
            let mut row = vec![' '; width];
            let is_top = y == 0;
            let is_bot = y == canvas_h - 1;

            for (x, cell) in row.iter_mut().enumerate().take(width) {
                let is_left = x == 0;
                let is_right = x == width - 1;

                if (is_top || is_bot) && (is_left || is_right) {
                    *cell = '+';
                } else if is_top || is_bot {
                    *cell = '-';
                } else if is_left || is_right {
                    *cell = '|';
                }
            }

            // Draw source (S) and listener (L) markers
            let sx_norm = (self.source_pos.0 / self.room_width_m).clamp(0.0, 1.0);
            let sy_norm = (self.source_pos.1 / self.room_depth_m).clamp(0.0, 1.0);
            let lx_norm = (self.listener_pos.0 / self.room_width_m).clamp(0.0, 1.0);
            let ly_norm = (self.listener_pos.1 / self.room_depth_m).clamp(0.0, 1.0);

            let s_col = (1.0 + sx_norm * (width as f32 - 3.0)).round() as usize;
            let s_row = ((canvas_h as f32 - 2.0) * (1.0 - sy_norm) + 1.0).round() as usize;

            let l_col = (1.0 + lx_norm * (width as f32 - 3.0)).round() as usize;
            let l_row = ((canvas_h as f32 - 2.0) * (1.0 - ly_norm) + 1.0).round() as usize;

            if y == s_row && s_col < width {
                row[s_col] = 'S';
            }
            if y == l_row && l_col < width {
                row[l_col] = 'L';
            }

            lines.push(row.into_iter().collect());
        }

        let footer = format!(
            "DirDist: {:.2}m ({:.1}ms) | Alpha: {:.2} | Vol: {:.0}m3 | Area: {:.0}m2",
            self.direct_distance(),
            self.direct_delay_ms(),
            self.active_absorption(),
            self.room_volume(),
            self.room_surface_area()
        );
        lines.push(footer);
        lines
    }

    pub fn render_ascii_snapshot_str(&self) -> String {
        self.render_ascii(40, 16).join("\n")
    }

    #[cfg(feature = "gui")]
    pub fn show(&mut self, ui: &mut egui::Ui) {
        let avail = ui.available_size();
        let rect = Rect::new(0.0, 0.0, avail.x.max(400.0), avail.y.max(400.0));
        self.show_rect(ui, rect);
    }

    #[cfg(feature = "gui")]
    pub fn show_rect(&mut self, ui: &mut egui::Ui, layout_rect: Rect) {
        self.update_acoustics();

        let total_size = egui::vec2(layout_rect.width, layout_rect.height);
        let (response, painter) = ui.allocate_painter(total_size, egui::Sense::click_and_drag());
        let rect = response.rect;

        // Background chassis
        painter.rect_filled(rect, 8.0, Color32::from_rgb(11, 15, 25));
        painter.rect_stroke(
            rect,
            8.0,
            Stroke::new(1.0_f32, Color32::from_rgb(30, 41, 59)),
        );

        // Header
        let header_rect = egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), 48.0));
        painter.rect_filled(header_rect, 8.0, Color32::from_rgb(15, 23, 42));
        painter.text(
            egui::pos2(header_rect.min.x + 16.0, header_rect.min.y + 12.0),
            egui::Align2::LEFT_TOP,
            "🏛️ 3D SHOEBOX ROOM ACOUSTICS & IMAGE-SOURCE REVERB",
            egui::FontId::proportional(14.0),
            Color32::from_rgb(255, 255, 255),
        );

        painter.text(
            egui::pos2(header_rect.max.x - 16.0, header_rect.min.y + 14.0),
            egui::Align2::RIGHT_TOP,
            format!(
                "Material: {} | RT60 = {:.2}s",
                self.wall_material.display_name(),
                self.rt60_seconds
            ),
            egui::FontId::proportional(11.0),
            Color32::from_rgb(0, 255, 180),
        );

        // Plan Canvas Area
        let max_canvas_w = (rect.width() - 32.0).max(200.0);
        let max_canvas_h = (rect.height() - 180.0).max(180.0);

        // Maintain room aspect ratio (W / D)
        let room_aspect = (self.room_width_m / self.room_depth_m).clamp(0.2, 5.0);
        let (canvas_w, canvas_h) = if max_canvas_w / max_canvas_h > room_aspect {
            (max_canvas_h * room_aspect, max_canvas_h)
        } else {
            (max_canvas_w, max_canvas_w / room_aspect)
        };

        let room_rect = egui::Rect::from_center_size(
            egui::pos2(rect.center().x, rect.min.y + 56.0 + canvas_h * 0.5),
            egui::vec2(canvas_w, canvas_h),
        );

        // Room Floor Surface
        painter.rect_filled(room_rect, 4.0, Color32::from_rgb(6, 12, 20));
        painter.rect_stroke(
            room_rect,
            4.0,
            Stroke::new(2.0_f32, Color32::from_rgb(56, 189, 248)),
        );

        // Wall boundary labels
        painter.text(
            egui::pos2(room_rect.center().x, room_rect.min.y - 12.0),
            egui::Align2::CENTER_CENTER,
            format!("Front Wall ({:.1}m)", self.room_width_m),
            egui::FontId::proportional(9.0),
            Color32::from_rgb(148, 163, 184),
        );
        painter.text(
            egui::pos2(room_rect.min.x - 6.0, room_rect.center().y),
            egui::Align2::RIGHT_CENTER,
            format!("Left ({:.1}m)", self.room_depth_m),
            egui::FontId::proportional(9.0),
            Color32::from_rgb(148, 163, 184),
        );

        // Source and Listener Positions
        let s_pos = self.room_to_canvas_pos(self.source_pos.0, self.source_pos.1, room_rect);
        let l_pos = self.room_to_canvas_pos(self.listener_pos.0, self.listener_pos.1, room_rect);

        // 1. Direct ray from Source to Listener
        painter.line_segment(
            [s_pos, l_pos],
            Stroke::new(2.0_f32, Color32::from_rgb(255, 255, 255)),
        );

        // 2. Early reflections (4 lateral wall bounces)
        let rays = self.evaluate_early_reflections();
        for ray in &rays[0..4] {
            let b_pos = self.room_to_canvas_pos(ray.bounce_point.0, ray.bounce_point.1, room_rect);
            painter.line_segment(
                [s_pos, b_pos],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(249, 115, 22, 120)),
            );
            painter.line_segment(
                [b_pos, l_pos],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(249, 115, 22, 120)),
            );
            painter.circle_filled(b_pos, 2.5, Color32::from_rgb(251, 146, 60));
        }

        // Dragging logic
        if response.drag_started() {
            if let Some(pos) = response.interact_pointer_pos() {
                if self.hit_test_source_puck((pos.x, pos.y), room_rect) {
                    self.is_dragging_source = true;
                } else if self.hit_test_listener_puck((pos.x, pos.y), room_rect) {
                    self.is_dragging_listener = true;
                }
            }
        }

        if response.drag_stopped() {
            self.is_dragging_source = false;
            self.is_dragging_listener = false;
        }

        if self.is_dragging_source {
            if let Some(pos) = response.interact_pointer_pos() {
                let norm_x = ((pos.x - room_rect.min.x) / room_rect.width()).clamp(0.02, 0.98);
                let norm_y = ((room_rect.max.y - pos.y) / room_rect.height()).clamp(0.02, 0.98);
                self.source_pos.0 = norm_x * self.room_width_m;
                self.source_pos.1 = norm_y * self.room_depth_m;
                self.update_acoustics();
            }
        } else if self.is_dragging_listener {
            if let Some(pos) = response.interact_pointer_pos() {
                let norm_x = ((pos.x - room_rect.min.x) / room_rect.width()).clamp(0.02, 0.98);
                let norm_y = ((room_rect.max.y - pos.y) / room_rect.height()).clamp(0.02, 0.98);
                self.listener_pos.0 = norm_x * self.room_width_m;
                self.listener_pos.1 = norm_y * self.room_depth_m;
                self.update_acoustics();
            }
        }

        // Draw Source Puck
        let src_color = if self.is_dragging_source {
            Color32::from_rgb(255, 230, 0)
        } else {
            Color32::from_rgb(239, 68, 68)
        };
        painter.circle_filled(s_pos, 10.0, src_color);
        painter.circle_stroke(
            s_pos,
            SPATIAL_ROOM_PUCK_HIT_RADIUS,
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(239, 68, 68, 60)),
        );
        painter.text(
            s_pos,
            egui::Align2::CENTER_CENTER,
            "S",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(255, 255, 255),
        );

        // Draw Listener Puck with orientation yaw indicator
        let lis_color = if self.is_dragging_listener {
            Color32::from_rgb(255, 230, 0)
        } else {
            Color32::from_rgb(56, 189, 248)
        };
        painter.circle_filled(l_pos, 10.0, lis_color);
        painter.circle_stroke(
            l_pos,
            SPATIAL_ROOM_PUCK_HIT_RADIUS,
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(56, 189, 248, 60)),
        );
        painter.text(
            l_pos,
            egui::Align2::CENTER_CENTER,
            "L",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(255, 255, 255),
        );

        // Metrics & Acoustic Balances Panel
        let info_y = room_rect.max.y + 14.0;
        let info_rect = egui::Rect::from_min_size(
            egui::pos2(rect.min.x + 16.0, info_y),
            egui::vec2(rect.width() - 32.0, rect.max.y - info_y - 12.0),
        );
        painter.rect_filled(info_rect, 6.0, Color32::from_rgb(15, 23, 42));
        painter.rect_stroke(
            info_rect,
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(30, 41, 59)),
        );

        let metrics_text = format!(
            "Room: {:.1}m x {:.1}m x {:.1}m | Vol: {:.0} m³ | Area: {:.0} m² | Absorption α: {:.2}\nSabine RT60: {:.2}s | Direct Path: {:.2} m ({:.1} ms) | 4-Channel FDN Diffuse Reverb",
            self.room_width_m,
            self.room_depth_m,
            self.room_height_m,
            self.room_volume(),
            self.room_surface_area(),
            self.active_absorption(),
            self.rt60_seconds,
            self.direct_distance(),
            self.direct_delay_ms()
        );
        painter.text(
            egui::pos2(info_rect.min.x + 12.0, info_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            metrics_text,
            egui::FontId::proportional(12.0),
            Color32::from_rgb(226, 232, 240),
        );
    }
}
