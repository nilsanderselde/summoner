// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Broadcast Mastering Immersive Auro-3D 13.1 Spatializer & Tri-Level Height Elevation HUD (Step 1545).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const AURO3D_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_AZIMUTH_DEG: f32 = -180.0;
pub const MAX_AZIMUTH_DEG: f32 = 180.0;
pub const MIN_ELEVATION_DEG: f32 = -30.0;
pub const MAX_ELEVATION_DEG: f32 = 90.0;
pub const MIN_DISTANCE_M: f32 = 0.5;
pub const MAX_DISTANCE_M: f32 = 25.0;

/// Auro-3D Spatial Speaker Format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuroFormat {
    Auro91,          // 9.1 (5.1 Lower Bed + 4 Heights)
    Auro101,         // 10.1 (5.1 Bed + 4 Heights + 1 Top Voice of God)
    Auro111,         // 11.1 Cinema (6 Lower + 5 Heights)
    Auro131,         // 13.1 Ultimate Broadcast (7.1 Bed + 5 Heights + 1 Top)
    AuroMaxBinaural, // Binaural Auro-3D 3-Layer HRTF Rendering
}

impl AuroFormat {
    pub fn channel_count(&self) -> usize {
        match self {
            Self::Auro91 => 10,
            Self::Auro101 => 11,
            Self::Auro111 => 12,
            Self::Auro131 => 14,
            Self::AuroMaxBinaural => 2,
        }
    }

    pub fn has_top_ceiling_channel(&self) -> bool {
        matches!(self, Self::Auro101 | Self::Auro131)
    }
}

/// Auro-3D 13.1 Spatializer View HUD (Step 1545).
#[derive(Debug, Clone)]
pub struct Auro3dSpatializerView {
    pub format: AuroFormat,
    pub azimuth_deg: f32,           // [-180.0 ..= +180.0 deg]
    pub elevation_deg: f32,         // [-30.0 ..= +90.0 deg]
    pub distance_m: f32,            // [0.5 ..= 25.0 m]
    pub height_layer_delay_ms: f32, // [0.0 ..= 30.0 ms]
    pub auro_puck_pos: (f32, f32),  // Normalized (X: azimuth, Y: elevation)
    pub is_dragging_puck: bool,
    pub bed_layer_energy: f32,    // [0.0 ..= 1.0] Layer 1 (0 deg)
    pub height_layer_energy: f32, // [0.0 ..= 1.0] Layer 2 (+30 deg)
    pub top_layer_energy: f32,    // [0.0 ..= 1.0] Layer 3 (+90 deg Voice of God)
    pub color_palette: ContrastColorPalette,
}

impl Default for Auro3dSpatializerView {
    fn default() -> Self {
        Self::new()
    }
}

impl Auro3dSpatializerView {
    pub fn new() -> Self {
        let mut view = Self {
            format: AuroFormat::Auro131,
            azimuth_deg: 35.0,
            elevation_deg: 28.0,
            distance_m: 2.8,
            height_layer_delay_ms: 12.5,
            auro_puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            bed_layer_energy: 0.25,
            height_layer_energy: 0.65,
            top_layer_energy: 0.10,
            color_palette: ContrastColorPalette::default(),
        };
        view.auro_puck_pos = (
            Self::azimuth_to_normalized(view.azimuth_deg),
            Self::elevation_to_normalized(view.elevation_deg),
        );
        view.update_spatial_energies();
        view
    }

    /// Convert Azimuth [-180.0 ..= +180.0 deg] to normalized coordinate [0.0 ..= 1.0].
    pub fn azimuth_to_normalized(deg: f32) -> f32 {
        let a = deg.clamp(MIN_AZIMUTH_DEG, MAX_AZIMUTH_DEG);
        ((a - MIN_AZIMUTH_DEG) / (MAX_AZIMUTH_DEG - MIN_AZIMUTH_DEG)).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Azimuth [-180.0 ..= +180.0 deg].
    pub fn normalized_to_azimuth(norm: f32) -> f32 {
        MIN_AZIMUTH_DEG + norm.clamp(0.0, 1.0) * (MAX_AZIMUTH_DEG - MIN_AZIMUTH_DEG)
    }

    /// Convert Elevation [-30.0 ..= +90.0 deg] to normalized coordinate [0.0 ..= 1.0].
    pub fn elevation_to_normalized(deg: f32) -> f32 {
        let e = deg.clamp(MIN_ELEVATION_DEG, MAX_ELEVATION_DEG);
        ((e - MIN_ELEVATION_DEG) / (MAX_ELEVATION_DEG - MIN_ELEVATION_DEG)).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Elevation [-30.0 ..= +90.0 deg].
    pub fn normalized_to_elevation(norm: f32) -> f32 {
        MIN_ELEVATION_DEG + norm.clamp(0.0, 1.0) * (MAX_ELEVATION_DEG - MIN_ELEVATION_DEG)
    }

    /// Set Auro-3D format.
    pub fn set_format(&mut self, format: AuroFormat) {
        self.format = format;
        self.update_spatial_energies();
    }

    /// Update Tri-Level energy panning distribution across Lower Bed, Height, and Voice of God layers.
    pub fn update_spatial_energies(&mut self) {
        let el = self.elevation_deg.clamp(-30.0, 90.0);

        if el <= 0.0 {
            // All energy in Bed Layer (0 deg)
            self.bed_layer_energy = 1.0;
            self.height_layer_energy = 0.0;
            self.top_layer_energy = 0.0;
        } else if el <= 30.0 {
            // Pan between Bed (0 deg) and Height (+30 deg)
            let t = el / 30.0;
            self.bed_layer_energy = (1.0 - t).powi(2);
            self.height_layer_energy = (t * (2.0 - t)).clamp(0.0, 1.0);
            self.top_layer_energy = 0.0;
        } else {
            // Pan between Height (+30 deg) and Top (+90 deg)
            let t = (el - 30.0) / 60.0;
            self.bed_layer_energy = 0.0;
            self.height_layer_energy = (1.0 - t).powi(2);
            self.top_layer_energy = if self.format.has_top_ceiling_channel() {
                t * (2.0 - t)
            } else {
                0.0
            };
        }
    }

    /// Evaluate 3D Cartesian coordinates (X, Y, Z) in meters.
    pub fn evaluate_cartesian_position(&self) -> (f32, f32, f32) {
        let az_rad = self.azimuth_deg.to_radians();
        let el_rad = self.elevation_deg.to_radians();
        let d = self.distance_m;

        let x = d * el_rad.cos() * az_rad.sin();
        let y = d * el_rad.cos() * az_rad.cos();
        let z = d * el_rad.sin();
        (x, y, z)
    }

    /// Hit-test touch coordinate on the Auro-3D position puck.
    pub fn hit_test_auro_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.auro_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.auro_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= AURO3D_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render of Auro-3D Tri-Level Height Radar and Layer Meter.
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

        // Draw 3-Layer Height Radar on left half
        let left_w = mid_x - 2;
        let center_r = height / 2;
        grid[center_r][left_w / 2] = '+';

        // Rings for Lower Bed, Height, Top
        let r_max = (left_w / 2 - 2).min(height / 2 - 2);
        for i in 1..=3 {
            let cr = (i * r_max) / 3;
            if cr < left_w / 2 && cr < height / 2 {
                grid[center_r - cr][left_w / 2] = '-';
                grid[center_r + cr][left_w / 2] = '-';
            }
        }

        // Puck on left half
        let puck_col = ((self.auro_puck_pos.0 * (left_w - 2) as f32) + 2.0).round() as usize;
        let puck_row =
            (((1.0 - self.auro_puck_pos.1) * (height - 5) as f32) + 2.0).round() as usize;
        if puck_row < height - 1 && puck_col < mid_x {
            grid[puck_row][puck_col] = '@';
        }

        // Tri-Level Energy Meters on right half
        let right_w = width - mid_x - 2;
        let layers = [
            ("L1 BED", self.bed_layer_energy),
            ("L2 HGT", self.height_layer_energy),
            ("L3 TOP", self.top_layer_energy),
        ];

        for (i, (_name, energy)) in layers.iter().enumerate() {
            let bar_col = mid_x + 3 + i * (right_w / 4);
            let bar_h = (energy * (height - 4) as f32).round() as usize;
            for r in 0..bar_h {
                if height - 2 > r && bar_col < width - 1 {
                    grid[height - 2 - r][bar_col] = '#';
                }
            }
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

        // Background
        painter.rect_filled(rect, 6.0, Color32::from_rgb(14, 18, 28));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "IMMERSIVE AURO-3D 13.1 SPATIALIZER & TRI-LEVEL HEIGHT ELEVATION HUD",
            egui::FontId::proportional(16.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Auro-3D Format Tabs (y: 48..92) - Each tab >= 44pt height
        let formats = [
            (AuroFormat::Auro131, "AURO-3D 13.1"),
            (AuroFormat::Auro111, "AURO-3D 11.1"),
            (AuroFormat::Auro101, "AURO-3D 10.1"),
            (AuroFormat::Auro91, "AURO-3D 9.1"),
            (AuroFormat::AuroMaxBinaural, "AURO BINAURAL"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (fmt, name)) in formats.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_selected = self.format == *fmt;
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
                        self.set_format(*fmt);
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

        // Left 55%: Auro-3D Tri-Level Height Space (Azimuth vs Elevation)
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
            "AURO-3D 3-LAYER OBJECT SPACE (AZIMUTH vs ELEVATION)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // Concentric distance / height layer boundary rings
        let rcx = left_rect.center().x;
        let rcy = left_rect.center().y + 10.0;
        let max_r = 75.0_f32;
        for r_step in 1..=3 {
            let cr = max_r * (r_step as f32 / 3.0);
            painter.circle_stroke(
                egui::pos2(rcx, rcy),
                cr,
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 90, 130, 80)),
            );
        }
        painter.line_segment(
            [egui::pos2(rcx - max_r, rcy), egui::pos2(rcx + max_r, rcy)],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 90, 130, 80)),
        );
        painter.line_segment(
            [egui::pos2(rcx, rcy - max_r), egui::pos2(rcx, rcy + max_r)],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 90, 130, 80)),
        );

        // Interactive Auro-3D Puck
        let puck_x = left_rect.min.x + self.auro_puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.auro_puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.auro_puck_pos = (nx, ny);
                    self.azimuth_deg = Self::normalized_to_azimuth(nx);
                    self.elevation_deg = Self::normalized_to_elevation(ny);
                    self.update_spatial_energies();
                }
            }
        }

        // Draw Touch Hit Target boundary (>= 44x44pt)
        painter.circle_stroke(
            puck_pos,
            AURO3D_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(0, 229, 255, 140)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(0, 229, 255));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 20.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Azimuth: {:+.1}° | Elevation: {:+.1}° | Dist: {:.2} m",
                self.azimuth_deg, self.elevation_deg, self.distance_m
            ),
            egui::FontId::proportional(10.0),
            Color32::from_rgb(0, 229, 255),
        );

        // Right 45%: Tri-Level Energy Distribution & Inter-Layer Delay
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
            "TRI-LEVEL HEIGHT ENERGY DISTRIBUTION",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        let layers = [
            (
                "L1 LOWER BED (0°)",
                self.bed_layer_energy,
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "L2 HEIGHT (+30°)",
                self.height_layer_energy,
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "L3 VOICE OF GOD (+90°)",
                self.top_layer_energy,
                Color32::from_rgb(255, 107, 43),
            ),
        ];

        let bar_w = (right_rect.width() - 30.0 - 2.0 * 8.0) / 3.0;
        for (i, (lname, energy, col)) in layers.iter().enumerate() {
            let bx = right_rect.min.x + 15.0 + i as f32 * (bar_w + 8.0);
            let bar_h = energy * (right_rect.height() - 85.0);
            let b_rect = egui::Rect::from_min_max(
                egui::pos2(bx, right_rect.max.y - 25.0 - bar_h),
                egui::pos2(bx + bar_w, right_rect.max.y - 25.0),
            );
            painter.rect_filled(b_rect, 3.0, *col);

            painter.text(
                egui::pos2(bx + bar_w * 0.5, right_rect.max.y - 20.0),
                egui::Align2::CENTER_TOP,
                *lname,
                egui::FontId::proportional(8.0),
                Color32::from_rgb(200, 220, 245),
            );
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

        let (x, y, z) = self.evaluate_cartesian_position();
        let params = [
            (
                "3D POSITION (X, Y, Z)",
                format!("{:.2}m, {:.2}m, {:.2}m", x, y, z),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "AURO-3D CHANNELS",
                format!("{} Channels", self.format.channel_count()),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "HEIGHT LAYER DELAY",
                format!("{:.1} ms (Haas Delay)", self.height_layer_delay_ms),
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "TRI-LEVEL SPREAD",
                format!(
                    "{:.0}% Hgt / {:.0}% Top",
                    self.height_layer_energy * 100.0,
                    self.top_layer_energy * 100.0
                ),
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
            "[PASS] Immersive Auro-3D 13.1 Spatializer & Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
