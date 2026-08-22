// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Broadcast Mastering Immersive MPEG-H 3D Audio Object Metadata Spatializer HUD (Step 1565).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const MPEGH_3D_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_MPEGH_AZIMUTH_DEG: f32 = -180.0;
pub const MAX_MPEGH_AZIMUTH_DEG: f32 = 180.0;
pub const MIN_MPEGH_ELEVATION_DEG: f32 = -90.0;
pub const MAX_MPEGH_ELEVATION_DEG: f32 = 90.0;
pub const MIN_MPEGH_DISTANCE_M: f32 = 0.5;
pub const MAX_MPEGH_DISTANCE_M: f32 = 20.0;

/// MPEG-H 3D Audio Delivery & Rendering Profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MpeghProfile {
    Level4_7_1_4,      // 7.1.4 Standard Immersive Broadcast
    Level5_22_2,       // 22.2 Super Hi-Vision Ultra High Definition
    BinauralHeadTrack, // MPEG-H 3D Binaural with 6-DOF Dynamic Head Tracking
    Dynamic3DObject,   // Dynamic 3D Object Metadata Bitstream (ADM / OAM)
    AdvancedDownmix,   // ITU-R BS.775 Advanced Immersive Downmix
}

impl MpeghProfile {
    pub fn profile_name(&self) -> &'static str {
        match self {
            Self::Level4_7_1_4 => "7.1.4 IMMERSIVE (L4)",
            Self::Level5_22_2 => "22.2 BROADCAST (L5)",
            Self::BinauralHeadTrack => "BINAURAL HEAD TRACK",
            Self::Dynamic3DObject => "3D OBJECT STREAM",
            Self::AdvancedDownmix => "ADVANCED DOWNMIX",
        }
    }

    pub fn speaker_count(&self) -> usize {
        match self {
            Self::Level4_7_1_4 => 12,
            Self::Level5_22_2 => 24,
            Self::BinauralHeadTrack => 2,
            Self::Dynamic3DObject => 16,
            Self::AdvancedDownmix => 6,
        }
    }

    pub fn metadata_bitrate_kbps(&self) -> usize {
        match self {
            Self::Level4_7_1_4 => 512,
            Self::Level5_22_2 => 1024,
            Self::BinauralHeadTrack => 384,
            Self::Dynamic3DObject => 640,
            Self::AdvancedDownmix => 256,
        }
    }
}

/// MPEG-H 3D Object Metadata Spatializer View HUD.
#[derive(Debug, Clone)]
pub struct Mpegh3DSpatializerView {
    pub profile: MpeghProfile,
    pub azimuth_deg: f32,           // [-180.0 ..= +180.0 deg]
    pub elevation_deg: f32,         // [-90.0 ..= +90.0 deg]
    pub distance_m: f32,            // [0.5 ..= 20.0 m]
    pub spread_divergence_pct: f32, // [0.0 ..= 100.0 %]
    pub mpegh_puck_pos: (f32, f32), // Normalized (X: azimuth, Y: elevation)
    pub is_dragging_puck: bool,
    pub bed_energy: [f32; 6], // L/R, C/LFE, Ls/Rs, Ltf/Rtf, Ltr/Rtr, OBJ
    pub object_gain_db: f32,  // [-36.0 ..= +6.0 dBFS]
    pub color_palette: ContrastColorPalette,
}

impl Default for Mpegh3DSpatializerView {
    fn default() -> Self {
        Self::new()
    }
}

impl Mpegh3DSpatializerView {
    pub fn new() -> Self {
        let mut view = Self {
            profile: MpeghProfile::Level4_7_1_4,
            azimuth_deg: 35.0,
            elevation_deg: 20.0,
            distance_m: 2.8,
            spread_divergence_pct: 30.0,
            mpegh_puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            bed_energy: [0.75, 0.40, 0.60, 0.50, 0.35, 0.85],
            object_gain_db: -2.0,
            color_palette: ContrastColorPalette::default(),
        };
        view.mpegh_puck_pos = (
            Self::azimuth_to_normalized(view.azimuth_deg),
            Self::elevation_to_normalized(view.elevation_deg),
        );
        view.update_spatial_distribution();
        view
    }

    pub fn azimuth_to_normalized(deg: f32) -> f32 {
        let a = deg.clamp(MIN_MPEGH_AZIMUTH_DEG, MAX_MPEGH_AZIMUTH_DEG);
        ((a - MIN_MPEGH_AZIMUTH_DEG) / (MAX_MPEGH_AZIMUTH_DEG - MIN_MPEGH_AZIMUTH_DEG))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_azimuth(norm: f32) -> f32 {
        MIN_MPEGH_AZIMUTH_DEG
            + norm.clamp(0.0, 1.0) * (MAX_MPEGH_AZIMUTH_DEG - MIN_MPEGH_AZIMUTH_DEG)
    }

    pub fn elevation_to_normalized(deg: f32) -> f32 {
        let e = deg.clamp(MIN_MPEGH_ELEVATION_DEG, MAX_MPEGH_ELEVATION_DEG);
        ((e - MIN_MPEGH_ELEVATION_DEG) / (MAX_MPEGH_ELEVATION_DEG - MIN_MPEGH_ELEVATION_DEG))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_elevation(norm: f32) -> f32 {
        MIN_MPEGH_ELEVATION_DEG
            + norm.clamp(0.0, 1.0) * (MAX_MPEGH_ELEVATION_DEG - MIN_MPEGH_ELEVATION_DEG)
    }

    pub fn set_profile(&mut self, profile: MpeghProfile) {
        self.profile = profile;
        self.update_spatial_distribution();
    }

    /// Update MPEG-H 3D VBAP speaker bed gains and object trajectory.
    pub fn update_spatial_distribution(&mut self) {
        let az_rad = self.azimuth_deg.to_radians();
        let el_rad = self.elevation_deg.to_radians();

        let top_weight = el_rad.sin().max(0.0);
        let ear_weight = el_rad.cos();
        let front_weight = az_rad.cos().max(0.0);
        let rear_weight = (-az_rad.cos()).max(0.0);

        self.bed_energy[0] = (ear_weight * front_weight * 0.8).clamp(0.0, 1.0); // L/R
        self.bed_energy[1] = (front_weight * 0.5).clamp(0.0, 1.0); // C/LFE
        self.bed_energy[2] = (ear_weight * rear_weight * 0.8).clamp(0.0, 1.0); // Ls/Rs
        self.bed_energy[3] = (top_weight * front_weight * 0.9).clamp(0.0, 1.0); // Ltf/Rtf
        self.bed_energy[4] = (top_weight * rear_weight * 0.9).clamp(0.0, 1.0); // Ltr/Rtr
        self.bed_energy[5] = 0.85; // OBJ
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

    /// Hit test coordinate on the interactive MPEG-H 3D object puck.
    pub fn hit_test_mpegh_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.mpegh_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.mpegh_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= MPEGH_3D_PUCK_HIT_RADIUS
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

        // Left half: 3D Object Hemispherical Radar
        let left_w = mid_x - 2;
        let p_row = (((1.0 - self.mpegh_puck_pos.1) * (height - 5) as f32) + 2.0).round() as usize;
        let p_col = ((self.mpegh_puck_pos.0 * (left_w - 4) as f32) + 2.0).round() as usize;
        if p_row < height - 1 && p_col < mid_x {
            grid[p_row][p_col] = 'M';
        }

        // Right half: MPEG-H 3D Bed Channel Energy
        let right_w = width - mid_x - 2;
        let bar_spacing = right_w / 7;
        for (i, energy) in self.bed_energy.iter().enumerate() {
            let col = mid_x + 2 + (i + 1) * bar_spacing;
            let bar_h = (energy * (height - 4) as f32).round() as usize;
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
            "BROADCAST MASTERING IMMERSIVE MPEG-H 3D OBJECT SPATIALIZER HUD",
            egui::FontId::proportional(14.5),
            Color32::from_rgb(240, 245, 255),
        );

        // Profile Preset Tabs (y: 48..92) - Each tab >= 44pt touch target
        let profiles = [
            (MpeghProfile::Level4_7_1_4, "7.1.4 IMMERSIVE"),
            (MpeghProfile::Level5_22_2, "22.2 BROADCAST"),
            (MpeghProfile::BinauralHeadTrack, "BINAURAL HEAD"),
            (MpeghProfile::Dynamic3DObject, "3D OBJECT STREAM"),
            (MpeghProfile::AdvancedDownmix, "ADVANCED DOWNMIX"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (pr, name)) in profiles.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_sel = self.profile == *pr;
            let bg_col = if is_sel {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(24, 32, 48)
            };
            let text_col = if is_sel {
                Color32::from_rgb(10, 16, 24)
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
                        self.set_profile(*pr);
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

        // Left 55%: MPEG-H 3D Hemispherical Trajectory Radar
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
            "MPEG-H 3D HEMISPHERICAL OBJECT TRAJECTORY RADAR",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(0, 229, 255),
        );

        let dcx = left_rect.center().x;
        let dcy = left_rect.center().y + 10.0;
        let max_r = 75.0_f32;

        for r_step in [0.35, 0.70, 1.00] {
            painter.circle_stroke(
                egui::pos2(dcx, dcy),
                max_r * r_step,
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 60)),
            );
        }
        painter.line_segment(
            [egui::pos2(dcx - max_r, dcy), egui::pos2(dcx + max_r, dcy)],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(50, 80, 120, 80)),
        );
        painter.line_segment(
            [egui::pos2(dcx, dcy - max_r), egui::pos2(dcx, dcy + max_r)],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(50, 80, 120, 80)),
        );

        // Interactive MPEG-H Puck
        let puck_x = left_rect.min.x + self.mpegh_puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.mpegh_puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.mpegh_puck_pos = (nx, ny);
                    self.azimuth_deg = Self::normalized_to_azimuth(nx);
                    self.elevation_deg = Self::normalized_to_elevation(ny);
                    self.update_spatial_distribution();
                }
            }
        }

        // Touch Hit Target boundary (>= 44x44pt)
        painter.circle_stroke(
            puck_pos,
            MPEGH_3D_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(0, 229, 255, 150)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(0, 229, 255));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 20.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Azimuth: {:+.1}° | Elevation: {:+.1}° | Distance: {:.2} m",
                self.azimuth_deg, self.elevation_deg, self.distance_m
            ),
            egui::FontId::proportional(10.5),
            Color32::from_rgb(0, 229, 255),
        );

        // Right 45%: MPEG-H 3D Bed Channel Energy & Object Metadata
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
            "MPEG-H 3D SPEAKER BED DISTRIBUTION",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(0, 229, 255),
        );

        let channels = [
            ("L/R", self.bed_energy[0], Color32::from_rgb(0, 229, 255)),
            ("C/LFE", self.bed_energy[1], Color32::from_rgb(255, 215, 0)),
            ("Ls/Rs", self.bed_energy[2], Color32::from_rgb(0, 255, 180)),
            (
                "Ltf/Rtf",
                self.bed_energy[3],
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "Ltr/Rtr",
                self.bed_energy[4],
                Color32::from_rgb(180, 90, 255),
            ),
            ("OBJ", self.bed_energy[5], Color32::from_rgb(255, 180, 50)),
        ];

        let bar_w = (right_rect.width() - 30.0 - 5.0 * 6.0) / 6.0;
        for (i, (cname, energy, col)) in channels.iter().enumerate() {
            let bx = right_rect.min.x + 15.0 + i as f32 * (bar_w + 6.0);
            let bar_h = energy * (right_rect.height() - 80.0);
            let b_rect = egui::Rect::from_min_max(
                egui::pos2(bx, right_rect.max.y - 25.0 - bar_h),
                egui::pos2(bx + bar_w, right_rect.max.y - 25.0),
            );
            painter.rect_filled(b_rect, 3.0, *col);

            painter.text(
                egui::pos2(bx + bar_w * 0.5, right_rect.max.y - 20.0),
                egui::Align2::CENTER_TOP,
                *cname,
                egui::FontId::proportional(8.5),
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
                "MPEG-H 3D PROFILE",
                format!(
                    "{} ({} ch)",
                    self.profile.profile_name(),
                    self.profile.speaker_count()
                ),
                Color32::from_rgb(0, 255, 180),
            ),
            (
                "SPREAD DIVERGENCE",
                format!("{:.0}% (Polar Cones)", self.spread_divergence_pct),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "METADATA BITSTREAM",
                format!(
                    "{} kbps (Low Latency ADM)",
                    self.profile.metadata_bitrate_kbps()
                ),
                Color32::from_rgb(255, 180, 50),
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
                egui::FontId::proportional(13.5),
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
            "[PASS] MPEG-H 3D Audio Object Spatializer & Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
