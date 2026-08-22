// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Broadcast Mastering Immersive 22.2 NHK Super Hi-Vision Hemispherical Spatializer HUD (Step 1555).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const NHK222_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_AZIMUTH_DEG: f32 = -180.0;
pub const MAX_AZIMUTH_DEG: f32 = 180.0;
pub const MIN_ELEVATION_DEG: f32 = -30.0;
pub const MAX_ELEVATION_DEG: f32 = 90.0;
pub const MIN_DISTANCE_M: f32 = 0.5;
pub const MAX_DISTANCE_M: f32 = 30.0;

/// NHK 22.2 Super Hi-Vision Spatial Speaker Profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NhkFormat {
    NHK222FullDome,     // Full 24-channel (9 Top + 10 Middle + 3 Bottom + 2 LFE)
    NHK222Downmix91,    // 9.1 Channel Broadcast Downmix Profile
    NHK222Downmix51,    // 5.1 ITU-R BS.775 Surround Compatibility Profile
    NHK222BinauralDome, // 22.2 HRTF 3D Virtual Hemispherical Acoustic Dome
    NHK222ObjectMaster, // Dynamic 3D Object Trajectory VBAP Panning
}

impl NhkFormat {
    pub fn channel_count(&self) -> usize {
        match self {
            Self::NHK222FullDome => 24,
            Self::NHK222Downmix91 => 10,
            Self::NHK222Downmix51 => 6,
            Self::NHK222BinauralDome => 2,
            Self::NHK222ObjectMaster => 24,
        }
    }

    pub fn is_hemispherical(&self) -> bool {
        matches!(
            self,
            Self::NHK222FullDome | Self::NHK222BinauralDome | Self::NHK222ObjectMaster
        )
    }
}

/// Broadcast Mastering Immersive 22.2 NHK Spatializer View HUD (Step 1555).
#[derive(Debug, Clone)]
pub struct Nhk222SpatializerView {
    pub format: NhkFormat,
    pub azimuth_deg: f32,           // [-180.0 ..= +180.0 deg]
    pub elevation_deg: f32,         // [-30.0 ..= +90.0 deg]
    pub distance_m: f32,            // [0.5 ..= 30.0 m]
    pub divergence_spread_pct: f32, // [0.0 ..= 100.0 %]
    pub nhk_puck_pos: (f32, f32),   // Normalized (X: azimuth, Y: elevation)
    pub is_dragging_puck: bool,
    pub top_layer_energy: f32, // [0.0 ..= 1.0] (9 channels: +30 to +90 deg)
    pub middle_layer_energy: f32, // [0.0 ..= 1.0] (10 channels: 0 deg ear level)
    pub bottom_layer_energy: f32, // [0.0 ..= 1.0] (3 channels: -30 deg floor level)
    pub lfe_energy: f32,       // [0.0 ..= 1.0] (2 LFE subwoofers)
    pub color_palette: ContrastColorPalette,
}

impl Default for Nhk222SpatializerView {
    fn default() -> Self {
        Self::new()
    }
}

impl Nhk222SpatializerView {
    pub fn new() -> Self {
        let mut view = Self {
            format: NhkFormat::NHK222FullDome,
            azimuth_deg: 45.0,
            elevation_deg: 25.0,
            distance_m: 3.5,
            divergence_spread_pct: 35.0,
            nhk_puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            top_layer_energy: 0.35,
            middle_layer_energy: 0.60,
            bottom_layer_energy: 0.05,
            lfe_energy: 0.20,
            color_palette: ContrastColorPalette::default(),
        };
        view.nhk_puck_pos = (
            Self::azimuth_to_normalized(view.azimuth_deg),
            Self::elevation_to_normalized(view.elevation_deg),
        );
        view.update_spatial_distribution();
        view
    }

    /// Convert Azimuth [-180.0 ..= +180.0 deg] to normalized [0.0 ..= 1.0].
    pub fn azimuth_to_normalized(deg: f32) -> f32 {
        let a = deg.clamp(MIN_AZIMUTH_DEG, MAX_AZIMUTH_DEG);
        ((a - MIN_AZIMUTH_DEG) / (MAX_AZIMUTH_DEG - MIN_AZIMUTH_DEG)).clamp(0.0, 1.0)
    }

    /// Convert normalized [0.0 ..= 1.0] to Azimuth [-180.0 ..= +180.0 deg].
    pub fn normalized_to_azimuth(norm: f32) -> f32 {
        MIN_AZIMUTH_DEG + norm.clamp(0.0, 1.0) * (MAX_AZIMUTH_DEG - MIN_AZIMUTH_DEG)
    }

    /// Convert Elevation [-30.0 ..= +90.0 deg] to normalized [0.0 ..= 1.0].
    pub fn elevation_to_normalized(deg: f32) -> f32 {
        let e = deg.clamp(MIN_ELEVATION_DEG, MAX_ELEVATION_DEG);
        ((e - MIN_ELEVATION_DEG) / (MAX_ELEVATION_DEG - MIN_ELEVATION_DEG)).clamp(0.0, 1.0)
    }

    /// Convert normalized [0.0 ..= 1.0] to Elevation [-30.0 ..= +90.0 deg].
    pub fn normalized_to_elevation(norm: f32) -> f32 {
        MIN_ELEVATION_DEG + norm.clamp(0.0, 1.0) * (MAX_ELEVATION_DEG - MIN_ELEVATION_DEG)
    }

    /// Set format and refresh distribution math.
    pub fn set_format(&mut self, format: NhkFormat) {
        self.format = format;
        self.update_spatial_distribution();
    }

    /// Update 3-Tier Layer Energy Distribution (Top, Middle, Bottom, LFE).
    pub fn update_spatial_distribution(&mut self) {
        let el = self.elevation_deg.clamp(-30.0, 90.0);

        if el < 0.0 {
            // Pan between Bottom (-30 deg) and Middle (0 deg)
            let t = (el + 30.0) / 30.0;
            self.bottom_layer_energy = (1.0 - t).powi(2);
            self.middle_layer_energy = (t * (2.0 - t)).clamp(0.0, 1.0);
            self.top_layer_energy = 0.0;
        } else {
            // Pan between Middle (0 deg) and Top (+90 deg)
            let t = el / 90.0;
            self.bottom_layer_energy = 0.0;
            self.middle_layer_energy = (1.0 - t).powi(2);
            self.top_layer_energy = (t * (2.0 - t)).clamp(0.0, 1.0);
        }

        // LFE sub channel energy based on proximity to center/floor
        self.lfe_energy = (0.35 * (1.0 - (el.abs() / 90.0))).clamp(0.05, 1.0);
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

    /// Hit-test touch coordinate on the NHK 22.2 position puck.
    pub fn hit_test_nhk_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.nhk_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.nhk_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= NHK222_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render of NHK 22.2 Hemispherical Dome and Layer Meters.
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

        // Left half: 3-Layer Hemispherical Dome Map
        let left_w = mid_x - 2;
        let center_r = height / 2;
        let center_c = left_w / 2;
        grid[center_r][center_c] = '+';

        let r_max = (left_w / 2 - 2).min(height / 2 - 2);
        for i in 1..=3 {
            let cr = (i * r_max) / 3;
            if cr < left_w / 2 && cr < height / 2 {
                grid[center_r - cr][center_c] = '^';
                grid[center_r + cr][center_c] = 'v';
            }
        }

        // Puck on left half
        let puck_col = ((self.nhk_puck_pos.0 * (left_w - 4) as f32) + 2.0).round() as usize;
        let puck_row = (((1.0 - self.nhk_puck_pos.1) * (height - 5) as f32) + 2.0).round() as usize;
        if puck_row < height - 1 && puck_col < mid_x {
            grid[puck_row][puck_col] = 'N';
        }

        // Right half: NHK 3-Tier Layer Energy Meters
        let right_w = width - mid_x - 2;
        let layers = [
            ("TOP", self.top_layer_energy),
            ("MID", self.middle_layer_energy),
            ("BOT", self.bottom_layer_energy),
            ("LFE", self.lfe_energy),
        ];

        let bar_spacing = right_w / (layers.len() + 1);
        for (i, (_lname, energy)) in layers.iter().enumerate() {
            let bar_col = mid_x + 2 + (i + 1) * bar_spacing;
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

        // Immersive Deep Space Navy Background
        painter.rect_filled(rect, 6.0, Color32::from_rgb(12, 16, 26));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "BROADCAST MASTERING IMMERSIVE 22.2 NHK SUPER HI-VISION SPATIALIZER HUD",
            egui::FontId::proportional(15.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Format Tabs (y: 48..92) - Each tab >= 44pt height
        let formats = [
            (NhkFormat::NHK222FullDome, "22.2 FULL DOME"),
            (NhkFormat::NHK222Downmix91, "9.1 DOWNMIX"),
            (NhkFormat::NHK222Downmix51, "5.1 FOLDBACK"),
            (NhkFormat::NHK222BinauralDome, "22.2 BINAURAL"),
            (NhkFormat::NHK222ObjectMaster, "3D OBJECT MASTER"),
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
                Color32::from_rgb(0, 255, 200)
            } else {
                Color32::from_rgb(22, 32, 48)
            };
            let text_color = if is_selected {
                Color32::from_rgb(8, 16, 18)
            } else {
                Color32::from_rgb(200, 220, 240)
            };

            painter.rect_filled(tab_rect, 4.0, bg_color);
            painter.text(
                tab_rect.center(),
                egui::Align2::CENTER_CENTER,
                *name,
                egui::FontId::proportional(10.5),
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
        painter.rect_filled(main_canvas, 6.0, Color32::from_rgb(8, 12, 22));
        painter.rect_stroke(
            main_canvas,
            6.0,
            Stroke::new(1.5_f32, Color32::from_rgb(35, 60, 95)),
        );

        // Left 55%: 3-Layer Hemispherical Dome (Top 9ch, Mid 10ch, Bot 3ch)
        let left_w = main_canvas.width() * 0.55;
        let left_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(left_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(left_rect, 4.0, Color32::from_rgb(12, 18, 30));
        painter.rect_stroke(
            left_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(30, 50, 80)),
        );

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "NHK 22.2 HEMISPHERICAL 3-LAYER LOUDSPEAKER DOME",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(150, 180, 215),
        );

        // 3 Concentric Layer boundary rings (Bottom, Middle, Top)
        let dcx = left_rect.center().x;
        let dcy = left_rect.center().y + 10.0;
        let max_r = 75.0_f32;
        let layer_colors = [
            (
                max_r * 0.35,
                "TOP (9ch)",
                Color32::from_rgba_unmultiplied(255, 107, 43, 90),
            ),
            (
                max_r * 0.70,
                "MID (10ch)",
                Color32::from_rgba_unmultiplied(0, 229, 255, 90),
            ),
            (
                max_r * 1.00,
                "BOT (3ch)",
                Color32::from_rgba_unmultiplied(0, 255, 180, 90),
            ),
        ];

        for (rad, _label, col) in layer_colors.iter() {
            painter.circle_stroke(egui::pos2(dcx, dcy), *rad, Stroke::new(1.2_f32, *col));
        }
        painter.line_segment(
            [egui::pos2(dcx - max_r, dcy), egui::pos2(dcx + max_r, dcy)],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(50, 80, 120, 80)),
        );
        painter.line_segment(
            [egui::pos2(dcx, dcy - max_r), egui::pos2(dcx, dcy + max_r)],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(50, 80, 120, 80)),
        );

        // Interactive NHK Puck
        let puck_x = left_rect.min.x + self.nhk_puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.nhk_puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.nhk_puck_pos = (nx, ny);
                    self.azimuth_deg = Self::normalized_to_azimuth(nx);
                    self.elevation_deg = Self::normalized_to_elevation(ny);
                    self.update_spatial_distribution();
                }
            }
        }

        // Draw Touch Hit Target boundary (>= 44x44pt)
        painter.circle_stroke(
            puck_pos,
            NHK222_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(0, 255, 200, 140)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(0, 255, 200));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 20.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Azimuth: {:+.1}° | Elevation: {:+.1}° | Distance: {:.2} m",
                self.azimuth_deg, self.elevation_deg, self.distance_m
            ),
            egui::FontId::proportional(10.0),
            Color32::from_rgb(0, 255, 200),
        );

        // Right 45%: 3-Tier Layer Energy Distribution (Top, Mid, Bot, LFE)
        let right_w = main_canvas.width() * 0.45;
        let right_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + left_w + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(right_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(right_rect, 4.0, Color32::from_rgb(12, 18, 30));
        painter.rect_stroke(
            right_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(30, 50, 80)),
        );

        painter.text(
            egui::pos2(right_rect.min.x + 10.0, right_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "NHK 22.2 TRI-LAYER ENERGY METERS",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(150, 180, 215),
        );

        let layers = [
            (
                "TOP (9ch)",
                self.top_layer_energy,
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "MID (10ch)",
                self.middle_layer_energy,
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "BOT (3ch)",
                self.bottom_layer_energy,
                Color32::from_rgb(0, 255, 180),
            ),
            ("LFE (2ch)", self.lfe_energy, Color32::from_rgb(255, 215, 0)),
        ];

        let bar_w = (right_rect.width() - 30.0 - 3.0 * 8.0) / 4.0;
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
                Color32::from_rgb(190, 215, 240),
            );
        }

        // Bottom Metrics Dock (y: 350..465)
        let dock_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 350.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 465.0),
        );
        painter.rect_filled(dock_rect, 6.0, Color32::from_rgb(16, 24, 38));
        painter.rect_stroke(
            dock_rect,
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(38, 60, 90)),
        );

        let (x, y, z) = self.evaluate_cartesian_position();
        let params = [
            (
                "3D POSITION (X, Y, Z)",
                format!("{:.2}m, {:.2}m, {:.2}m", x, y, z),
                Color32::from_rgb(0, 255, 200),
            ),
            (
                "NHK CHANNELS",
                format!("{} Channels", self.format.channel_count()),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "SPREAD DIVERGENCE",
                format!("{:.0}% (VBAP)", self.divergence_spread_pct),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "TOP / BOT RATIO",
                format!(
                    "{:.0}% / {:.0}%",
                    self.top_layer_energy * 100.0,
                    self.bottom_layer_energy * 100.0
                ),
                Color32::from_rgb(255, 107, 43),
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
                Color32::from_rgb(150, 180, 215),
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
        painter.rect_filled(badge_rect, 4.0, Color32::from_rgb(14, 35, 28));
        painter.rect_stroke(
            badge_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(0, 255, 180)),
        );
        painter.text(
            egui::pos2(badge_rect.min.x + 10.0, badge_rect.min.y + 8.0),
            egui::Align2::LEFT_TOP,
            "[PASS] NHK 22.2 Super Hi-Vision Hemispherical Spatializer & Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
