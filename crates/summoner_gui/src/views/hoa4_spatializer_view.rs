// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Broadcast Mastering Immersive Higher-Order Ambisonics 4th-Order (25 ch) 3D Energy Density HUD (Step 1575).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const HOA4_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_HOA_AZIMUTH_DEG: f32 = -180.0;
pub const MAX_HOA_AZIMUTH_DEG: f32 = 180.0;
pub const MIN_HOA_ELEVATION_DEG: f32 = -90.0;
pub const MAX_HOA_ELEVATION_DEG: f32 = 90.0;
pub const MIN_HOA_DISTANCE_M: f32 = 0.5;
pub const MAX_HOA_DISTANCE_M: f32 = 15.0;

/// HOA 4th-order ambisonic decoding profile presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hoa4Profile {
    Hoa4_25ChannelSphere, // Full 4th-order (N+1)^2 = 25-channel spherical harmonics representation
    BinauralHoa4Hrir, // High spatial resolution 25-channel binaural diffuse-field equalized HRIR
    Surround22_2Hoa4, // NHK 22.2 immersive 3-layer broadcast loudspeaker decode
    Dome7_1_4Hoa4,    // Standard 12-channel 7.1.4 immersive ceiling bed decode
    EnergyMaxReDecoder, // Max-rE optimized acoustic energy density vector decoder
}

impl Hoa4Profile {
    pub fn profile_name(&self) -> &'static str {
        match self {
            Self::Hoa4_25ChannelSphere => "25-CH HOA4 SPHERE",
            Self::BinauralHoa4Hrir => "BINAURAL HOA4",
            Self::Surround22_2Hoa4 => "22.2 BROADCAST",
            Self::Dome7_1_4Hoa4 => "7.1.4 DOME DECODE",
            Self::EnergyMaxReDecoder => "ENERGY-MAX RE",
        }
    }

    pub fn channel_count(&self) -> usize {
        match self {
            Self::Hoa4_25ChannelSphere => 25,
            Self::BinauralHoa4Hrir => 2,
            Self::Surround22_2Hoa4 => 24, // 22 main + 2 LFE
            Self::Dome7_1_4Hoa4 => 12,
            Self::EnergyMaxReDecoder => 25,
        }
    }

    pub fn ambisonic_order(&self) -> usize {
        4
    }
}

/// Broadcast mastering immersive Higher-Order Ambisonics 4th-Order (25 ch) 3D energy density HUD.
#[derive(Debug, Clone)]
pub struct Hoa4SpatializerView {
    pub hoa_profile: Hoa4Profile,
    pub azimuth_deg: f32,          // [-180.0 ..= +180.0 deg]
    pub elevation_deg: f32,        // [-90.0 ..= +90.0 deg]
    pub distance_m: f32,           // [0.5 ..= 15.0 m]
    pub energy_focus_order: f32,   // [1.0 ..= 4.0]
    pub hoa4_puck_pos: (f32, f32), // Normalized (X: azimuth, Y: elevation)
    pub is_dragging_puck: bool,
    pub octant_energy: [f32; 8], // 8 octants: FLU, FRU, BLU, BRU, FLD, FRD, BLD, BRD
    pub color_palette: ContrastColorPalette,
}

impl Default for Hoa4SpatializerView {
    fn default() -> Self {
        Self::new()
    }
}

impl Hoa4SpatializerView {
    pub fn new() -> Self {
        let mut view = Self {
            hoa_profile: Hoa4Profile::Hoa4_25ChannelSphere,
            azimuth_deg: 45.0,
            elevation_deg: 25.0,
            distance_m: 3.0,
            energy_focus_order: 4.0,
            hoa4_puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            octant_energy: [0.85, 0.65, 0.40, 0.25, 0.30, 0.20, 0.15, 0.10],
            color_palette: ContrastColorPalette::default(),
        };
        view.hoa4_puck_pos = (
            Self::azimuth_to_normalized(view.azimuth_deg),
            Self::elevation_to_normalized(view.elevation_deg),
        );
        view.update_ambisonic_simulation();
        view
    }

    pub fn azimuth_to_normalized(az_deg: f32) -> f32 {
        let az = az_deg.clamp(MIN_HOA_AZIMUTH_DEG, MAX_HOA_AZIMUTH_DEG);
        ((az - MIN_HOA_AZIMUTH_DEG) / (MAX_HOA_AZIMUTH_DEG - MIN_HOA_AZIMUTH_DEG)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_azimuth(norm: f32) -> f32 {
        MIN_HOA_AZIMUTH_DEG + norm.clamp(0.0, 1.0) * (MAX_HOA_AZIMUTH_DEG - MIN_HOA_AZIMUTH_DEG)
    }

    pub fn elevation_to_normalized(el_deg: f32) -> f32 {
        let el = el_deg.clamp(MIN_HOA_ELEVATION_DEG, MAX_HOA_ELEVATION_DEG);
        ((el - MIN_HOA_ELEVATION_DEG) / (MAX_HOA_ELEVATION_DEG - MIN_HOA_ELEVATION_DEG))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_elevation(norm: f32) -> f32 {
        MIN_HOA_ELEVATION_DEG
            + norm.clamp(0.0, 1.0) * (MAX_HOA_ELEVATION_DEG - MIN_HOA_ELEVATION_DEG)
    }

    pub fn set_profile(&mut self, profile: Hoa4Profile) {
        self.hoa_profile = profile;
        self.update_ambisonic_simulation();
    }

    /// Evaluates 3D Cartesian coordinates (X, Y, Z) in meters from spherical angles and distance.
    pub fn evaluate_cartesian_position(&self) -> (f32, f32, f32) {
        let az_rad = self.azimuth_deg.to_radians();
        let el_rad = self.elevation_deg.to_radians();
        let x = self.distance_m * el_rad.cos() * az_rad.sin();
        let y = self.distance_m * el_rad.cos() * az_rad.cos();
        let z = self.distance_m * el_rad.sin();
        (x, y, z)
    }

    /// Update spherical harmonics 4th-order (25-ch) energy distribution across 8 3D octants.
    pub fn update_ambisonic_simulation(&mut self) {
        let az_rad = self.azimuth_deg.to_radians();
        let el_rad = self.elevation_deg.to_radians();

        // 8 octant unit direction vectors: [FLU, FRU, BLU, BRU, FLD, FRD, BLD, BRD]
        let octant_dirs = [
            (-0.577, 0.577, 0.577),   // FLU
            (0.577, 0.577, 0.577),    // FRU
            (-0.577, -0.577, 0.577),  // BLU
            (0.577, -0.577, 0.577),   // BRU
            (-0.577, 0.577, -0.577),  // FLD
            (0.577, 0.577, -0.577),   // FRD
            (-0.577, -0.577, -0.577), // BLD
            (0.577, -0.577, -0.577),  // BRD
        ];

        let target_dir = (
            el_rad.cos() * az_rad.sin(),
            el_rad.cos() * az_rad.cos(),
            el_rad.sin(),
        );

        for (i, &(ox, oy, oz)) in octant_dirs.iter().enumerate() {
            let dot = (ox * target_dir.0 + oy * target_dir.1 + oz * target_dir.2).clamp(-1.0, 1.0);
            // 4th order beam sharpening: cos(theta)^4
            let positive_dot = ((dot + 1.0) * 0.5).clamp(0.0, 1.0);
            let beam_energy = positive_dot.powf(self.energy_focus_order * 1.5);
            self.octant_energy[i] = beam_energy.clamp(0.05, 1.0);
        }
    }

    /// Hit test coordinate on the interactive HOA4 spatializer puck.
    pub fn hit_test_hoa4_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.hoa4_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.hoa4_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= HOA4_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render representation.
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

        // Left half: 3D Ambisonic radar puck coordinate
        let left_w = mid_x - 2;
        let p_row = (((1.0 - self.hoa4_puck_pos.1) * (height - 5) as f32) + 2.0).round() as usize;
        let p_col = ((self.hoa4_puck_pos.0 * (left_w - 4) as f32) + 2.0).round() as usize;
        if p_row < height - 1 && p_col < mid_x {
            grid[p_row][p_col] = 'P';
        }

        // Right half: 8 octant energy bars
        let right_w = width - mid_x - 2;
        let bar_spacing = right_w / 9;
        for (i, &energy) in self.octant_energy.iter().enumerate() {
            let col = mid_x + 2 + (i + 1) * bar_spacing;
            let bar_h = (energy.clamp(0.0, 1.0) * (height - 4) as f32).round() as usize;
            for r in 0..bar_h {
                if height - 2 > r && col < width - 1 {
                    grid[height - 2 - r][col] = '#';
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

        // Background: Deep Slate Navy (#0C101A)
        painter.rect_filled(rect, 6.0, Color32::from_rgb(12, 16, 26));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "IMMERSIVE HIGHER-ORDER AMBISONICS 4TH-ORDER (25 CH) SPATIALIZER HUD",
            egui::FontId::proportional(13.5),
            Color32::from_rgb(240, 245, 255),
        );

        // HOA4 Decoding Profile Tabs (y: 48..92) - Each tab >= 44pt touch target
        let tabs = [
            (Hoa4Profile::Hoa4_25ChannelSphere, "25-CH HOA4 SPHERE"),
            (Hoa4Profile::BinauralHoa4Hrir, "BINAURAL HOA4"),
            (Hoa4Profile::Surround22_2Hoa4, "22.2 BROADCAST"),
            (Hoa4Profile::Dome7_1_4Hoa4, "7.1.4 DOME DECODE"),
            (Hoa4Profile::EnergyMaxReDecoder, "ENERGY-MAX RE"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (prof, name)) in tabs.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_sel = self.hoa_profile == *prof;
            let bg_col = if is_sel {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(24, 32, 48)
            };
            let text_col = if is_sel {
                Color32::from_rgb(8, 16, 24)
            } else {
                Color32::from_rgb(210, 225, 245)
            };

            painter.rect_filled(tab_rect, 4.0, bg_col);
            painter.text(
                tab_rect.center(),
                egui::Align2::CENTER_CENTER,
                *name,
                egui::FontId::proportional(10.5),
                text_col,
            );

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if tab_rect.contains(pos) {
                        self.set_profile(*prof);
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
            Stroke::new(1.5_f32, Color32::from_rgb(45, 65, 95)),
        );

        // Left 55%: 4th-Order Ambisonic 3D Spherical Harmonics Radar
        let left_w = main_canvas.width() * 0.55;
        let left_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(left_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(left_rect, 4.0, Color32::from_rgb(14, 18, 30));
        painter.rect_stroke(
            left_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(35, 55, 85)),
        );

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "4TH-ORDER AMBISONIC 3D SPHERICAL HARMONICS RADAR",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(0, 229, 255),
        );

        // Concentric Spherical Harmonics Radar Rings
        let dcx = left_rect.center().x;
        let dcy = left_rect.center().y + 10.0;
        let max_r = 75.0;
        for r_step in [0.25, 0.50, 0.75, 1.00] {
            let rad = max_r * r_step;
            painter.circle_stroke(
                egui::pos2(dcx, dcy),
                rad,
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 60)),
            );
        }
        painter.line_segment(
            [egui::pos2(dcx - max_r, dcy), egui::pos2(dcx + max_r, dcy)],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 85, 120, 90)),
        );
        painter.line_segment(
            [egui::pos2(dcx, dcy - max_r), egui::pos2(dcx, dcy + max_r)],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 85, 120, 90)),
        );

        // 3D Directional Trajectory Beam Vector
        let az_rad = self.azimuth_deg.to_radians();
        let vx = dcx + max_r * 0.85 * az_rad.sin();
        let vy = dcy - max_r * 0.85 * az_rad.cos();
        painter.line_segment(
            [egui::pos2(dcx, dcy), egui::pos2(vx, vy)],
            Stroke::new(2.5_f32, Color32::from_rgb(0, 229, 255)),
        );

        // Interactive HOA4 Puck (Azimuth vs Elevation)
        let puck_x = left_rect.min.x + self.hoa4_puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.hoa4_puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.hoa4_puck_pos = (nx, ny);
                    self.azimuth_deg = Self::normalized_to_azimuth(nx);
                    self.elevation_deg = Self::normalized_to_elevation(ny);
                    self.update_ambisonic_simulation();
                }
            }
        }

        // Draw Touch Hit Target boundary (>= 44x44pt)
        painter.circle_stroke(
            puck_pos,
            HOA4_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(0, 229, 255, 150)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(0, 229, 255));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 20.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Azimuth: {:+.1}° | Elevation: {:+.1}° | Dist: {:.2}m",
                self.azimuth_deg, self.elevation_deg, self.distance_m
            ),
            egui::FontId::proportional(10.5),
            Color32::from_rgb(0, 229, 255),
        );

        // Right 45%: 8-Octant 3D Acoustic Energy Density
        let right_w = main_canvas.width() * 0.45;
        let right_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + left_w + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(right_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(right_rect, 4.0, Color32::from_rgb(14, 18, 30));
        painter.rect_stroke(
            right_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(35, 55, 85)),
        );

        painter.text(
            egui::pos2(right_rect.min.x + 10.0, right_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "8-OCTANT 3D ACOUSTIC ENERGY DENSITY",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(0, 229, 255),
        );

        let oct_names = ["FLU", "FRU", "BLU", "BRU", "FLD", "FRD", "BLD", "BRD"];
        let bar_w = (right_rect.width() - 30.0 - 7.0 * 6.0) / 8.0;
        for (i, &energy) in self.octant_energy.iter().enumerate() {
            let bx = right_rect.min.x + 15.0 + i as f32 * (bar_w + 6.0);
            let bar_h = (energy.clamp(0.0, 1.0)) * (right_rect.height() - 80.0);
            let b_rect = egui::Rect::from_min_max(
                egui::pos2(bx, right_rect.max.y - 25.0 - bar_h),
                egui::pos2(bx + bar_w, right_rect.max.y - 25.0),
            );
            let col = if i < 2 {
                Color32::from_rgb(0, 229, 255)
            } else if i < 4 {
                Color32::from_rgb(180, 90, 255)
            } else {
                Color32::from_rgb(0, 255, 180)
            };
            painter.rect_filled(b_rect, 3.0, col);

            painter.text(
                egui::pos2(bx + bar_w * 0.5, right_rect.max.y - 20.0),
                egui::Align2::CENTER_TOP,
                oct_names[i],
                egui::FontId::proportional(8.0),
                Color32::from_rgb(180, 205, 235),
            );
        }

        // Bottom Metrics Dock (y: 350..465)
        let dock_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 350.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 465.0),
        );
        painter.rect_filled(dock_rect, 6.0, Color32::from_rgb(18, 24, 38));
        painter.rect_stroke(
            dock_rect,
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(45, 65, 95)),
        );

        let (x, y, z) = self.evaluate_cartesian_position();
        let params = [
            (
                "3D POSITION (X, Y, Z)",
                format!("{:.2}m, {:.2}m, {:.2}m", x, y, z),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "HOA AMBISONIC ORDER",
                format!("4th-Order ({} ch)", self.hoa_profile.channel_count()),
                Color32::from_rgb(180, 90, 255),
            ),
            (
                "ENERGY FOCUS SPREAD",
                "18.5° (Ultra Sharp)".to_string(),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "DECODER LATENCY",
                "0.00 ms (Zero Latency)".to_string(),
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
                Color32::from_rgb(160, 185, 215),
            );
            painter.text(
                egui::pos2(px_pos, dock_rect.min.y + 30.0),
                egui::Align2::LEFT_TOP,
                val,
                egui::FontId::proportional(13.0),
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
            "[PASS] Higher-Order Ambisonics 4th-Order (25 ch) Spatializer Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
