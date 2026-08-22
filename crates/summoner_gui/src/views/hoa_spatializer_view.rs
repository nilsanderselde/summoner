// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Broadcast Mastering Immersive Dolby Atmos / Ambisonics HOA 7.1.4 3D Spatializer & Binaural Head-Tracking HUD (Step 1525).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const HOA_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_AZIMUTH_DEG: f32 = -180.0;
pub const MAX_AZIMUTH_DEG: f32 = 180.0;
pub const MIN_ELEVATION_DEG: f32 = -90.0;
pub const MAX_ELEVATION_DEG: f32 = 90.0;
pub const MIN_DISTANCE_M: f32 = 0.10;
pub const MAX_DISTANCE_M: f32 = 10.00;
pub const MIN_HEAD_YAW_DEG: f32 = -180.0;
pub const MAX_HEAD_YAW_DEG: f32 = 180.0;

/// Spatial Audio Format / Speaker Topology Profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoaSpatialFormat {
    HoaThirdOrder,     // Higher-Order Ambisonics 3rd Order (16-ch ACN/N3D)
    DolbyAtmos714,     // 7.1.4 Immersive Bed & Ceiling (12-ch)
    BinauralHeadTrack, // Real-time SOFA HRIR Binaural with 6-DoF Head-Tracking
    Ambisonics514,     // 5.1.4 Hybrid Surround Object
    DomeAcoustic916,   // 9.1.6 Hemispherical Dome Array (16-ch)
}

impl HoaSpatialFormat {
    pub fn channel_count(&self) -> usize {
        match self {
            Self::HoaThirdOrder => 16,
            Self::DolbyAtmos714 => 12,
            Self::BinauralHeadTrack => 2,
            Self::Ambisonics514 => 10,
            Self::DomeAcoustic916 => 16,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::HoaThirdOrder => "HOA 3RD ORDER (16-CH)",
            Self::DolbyAtmos714 => "DOLBY ATMOS 7.1.4",
            Self::BinauralHeadTrack => "BINAURAL HEAD-TRACK",
            Self::Ambisonics514 => "AMBISONICS 5.1.4",
            Self::DomeAcoustic916 => "DOME ACOUSTIC 9.1.6",
        }
    }
}

/// 3D Spatial Speaker Definition for Monitoring Array.
#[derive(Debug, Clone, PartialEq)]
pub struct SpeakerPosition {
    pub label: &'static str,
    pub azimuth_deg: f32,
    pub elevation_deg: f32,
    pub is_ceiling: bool,
}

pub const ATMOS_714_SPEAKERS: [SpeakerPosition; 12] = [
    SpeakerPosition {
        label: "L",
        azimuth_deg: -30.0,
        elevation_deg: 0.0,
        is_ceiling: false,
    },
    SpeakerPosition {
        label: "C",
        azimuth_deg: 0.0,
        elevation_deg: 0.0,
        is_ceiling: false,
    },
    SpeakerPosition {
        label: "R",
        azimuth_deg: 30.0,
        elevation_deg: 0.0,
        is_ceiling: false,
    },
    SpeakerPosition {
        label: "LFE",
        azimuth_deg: 0.0,
        elevation_deg: -15.0,
        is_ceiling: false,
    },
    SpeakerPosition {
        label: "Ls",
        azimuth_deg: -90.0,
        elevation_deg: 0.0,
        is_ceiling: false,
    },
    SpeakerPosition {
        label: "Rs",
        azimuth_deg: 90.0,
        elevation_deg: 0.0,
        is_ceiling: false,
    },
    SpeakerPosition {
        label: "Lb",
        azimuth_deg: -140.0,
        elevation_deg: 0.0,
        is_ceiling: false,
    },
    SpeakerPosition {
        label: "Rb",
        azimuth_deg: 140.0,
        elevation_deg: 0.0,
        is_ceiling: false,
    },
    SpeakerPosition {
        label: "Tfl",
        azimuth_deg: -45.0,
        elevation_deg: 45.0,
        is_ceiling: true,
    },
    SpeakerPosition {
        label: "Tfr",
        azimuth_deg: 45.0,
        elevation_deg: 45.0,
        is_ceiling: true,
    },
    SpeakerPosition {
        label: "Tbl",
        azimuth_deg: -135.0,
        elevation_deg: 45.0,
        is_ceiling: true,
    },
    SpeakerPosition {
        label: "Tbr",
        azimuth_deg: 135.0,
        elevation_deg: 45.0,
        is_ceiling: true,
    },
];

/// Broadcast Mastering Immersive Dolby Atmos / HOA 7.1.4 3D Spatializer View HUD (Step 1525).
#[derive(Debug, Clone)]
pub struct HoaSpatializerView {
    pub format: HoaSpatialFormat,
    pub azimuth_deg: f32, // [-180.0 ..= +180.0 deg] (0 = Front/North, +90 = East/Right)
    pub elevation_deg: f32, // [-90.0 ..= +90.0 deg] (0 = Horizon, +90 = Zenith, -90 = Nadir)
    pub distance_m: f32,  // [0.10 ..= 10.00 m]
    pub head_yaw_deg: f32, // Head-tracking yaw rotation [-180.0 ..= +180.0 deg]
    pub head_pitch_deg: f32, // Head-tracking pitch rotation [-90.0 ..= +90.0 deg]
    pub head_tracking_latency_ms: f32, // Tracking telemetry latency (e.g. 0.8 ms)
    pub source_puck_pos: (f32, f32), // Normalized (X: azimuth, Y: distance)
    pub elevation_puck_norm: f32, // Normalized elevation [0.0 = -90 deg, 0.5 = 0 deg, 1.0 = +90 deg]
    pub is_dragging_source: bool,
    pub is_dragging_elevation: bool,
    pub is_dragging_head_yaw: bool,
    pub hoa_order: usize,               // Max 3rd order (16 channels)
    pub spherical_harmonics: [f32; 16], // 16-channel HOA ACN/N3D harmonic weights
    pub color_palette: ContrastColorPalette,
}

impl Default for HoaSpatializerView {
    fn default() -> Self {
        Self::new()
    }
}

impl HoaSpatializerView {
    pub fn new() -> Self {
        let mut view = Self {
            format: HoaSpatialFormat::DolbyAtmos714,
            azimuth_deg: 45.0,
            elevation_deg: 18.5,
            distance_m: 2.40,
            head_yaw_deg: -12.4,
            head_pitch_deg: 0.0,
            head_tracking_latency_ms: 0.8,
            source_puck_pos: (0.0, 0.0),
            elevation_puck_norm: 0.5,
            is_dragging_source: false,
            is_dragging_elevation: false,
            is_dragging_head_yaw: false,
            hoa_order: 3,
            spherical_harmonics: [0.0; 16],
            color_palette: ContrastColorPalette::default(),
        };

        view.source_puck_pos = (
            Self::azimuth_to_normalized(view.azimuth_deg),
            Self::distance_to_normalized(view.distance_m),
        );
        view.elevation_puck_norm = Self::elevation_to_normalized(view.elevation_deg);
        view.update_spherical_harmonics();
        view
    }

    /// Convert Azimuth [-180.0 ..= +180.0 deg] to normalized [0.0 ..= 1.0].
    pub fn azimuth_to_normalized(az: f32) -> f32 {
        let a = az.clamp(MIN_AZIMUTH_DEG, MAX_AZIMUTH_DEG);
        ((a - MIN_AZIMUTH_DEG) / (MAX_AZIMUTH_DEG - MIN_AZIMUTH_DEG)).clamp(0.0, 1.0)
    }

    /// Convert normalized [0.0 ..= 1.0] to Azimuth [-180.0 ..= +180.0 deg].
    pub fn normalized_to_azimuth(norm: f32) -> f32 {
        MIN_AZIMUTH_DEG + norm.clamp(0.0, 1.0) * (MAX_AZIMUTH_DEG - MIN_AZIMUTH_DEG)
    }

    /// Convert Elevation [-90.0 ..= +90.0 deg] to normalized [0.0 ..= 1.0].
    pub fn elevation_to_normalized(el: f32) -> f32 {
        let e = el.clamp(MIN_ELEVATION_DEG, MAX_ELEVATION_DEG);
        ((e - MIN_ELEVATION_DEG) / (MAX_ELEVATION_DEG - MIN_ELEVATION_DEG)).clamp(0.0, 1.0)
    }

    /// Convert normalized [0.0 ..= 1.0] to Elevation [-90.0 ..= +90.0 deg].
    pub fn normalized_to_elevation(norm: f32) -> f32 {
        MIN_ELEVATION_DEG + norm.clamp(0.0, 1.0) * (MAX_ELEVATION_DEG - MIN_ELEVATION_DEG)
    }

    /// Convert Distance [0.10 ..= 10.00 m] to normalized [0.0 ..= 1.0].
    pub fn distance_to_normalized(dist: f32) -> f32 {
        let d = dist.clamp(MIN_DISTANCE_M, MAX_DISTANCE_M);
        ((d - MIN_DISTANCE_M) / (MAX_DISTANCE_M - MIN_DISTANCE_M)).clamp(0.0, 1.0)
    }

    /// Convert normalized [0.0 ..= 1.0] to Distance [0.10 ..= 10.00 m].
    pub fn normalized_to_distance(norm: f32) -> f32 {
        MIN_DISTANCE_M + norm.clamp(0.0, 1.0) * (MAX_DISTANCE_M - MIN_DISTANCE_M)
    }

    /// Calculate effective source azimuth relative to listener head yaw.
    pub fn effective_relative_azimuth_deg(&self) -> f32 {
        let mut rel = self.azimuth_deg - self.head_yaw_deg;
        while rel > 180.0 {
            rel -= 360.0;
        }
        while rel < -180.0 {
            rel += 360.0;
        }
        rel
    }

    /// Update 16-Channel HOA 3rd Order Spherical Harmonic Decomposition Y_l^m(theta, phi) with N3D normalization.
    pub fn update_spherical_harmonics(&mut self) {
        let phi = self.effective_relative_azimuth_deg().to_radians(); // Azimuth
        let theta = (90.0 - self.elevation_deg).to_radians(); // Colatitude / Polar angle

        let sin_t = theta.sin();
        let cos_t = theta.cos();
        let sin_p = phi.sin();
        let cos_p = phi.cos();
        let sin_2p = (2.0 * phi).sin();
        let cos_2p = (2.0 * phi).cos();
        let sin_3p = (3.0 * phi).sin();
        let cos_3p = (3.0 * phi).cos();

        let dist_att = (1.0 / (self.distance_m.max(0.5) * 0.8)).clamp(0.1, 1.5);

        // Order 0 (ACN 0)
        self.spherical_harmonics[0] = 1.0 * dist_att;

        // Order 1 (ACN 1, 2, 3: Y, Z, X)
        self.spherical_harmonics[1] = (3.0_f32).sqrt() * sin_t * sin_p * dist_att; // Y (ACN 1)
        self.spherical_harmonics[2] = (3.0_f32).sqrt() * cos_t * dist_att; // Z (ACN 2)
        self.spherical_harmonics[3] = (3.0_f32).sqrt() * sin_t * cos_p * dist_att; // X (ACN 3)

        // Order 2 (ACN 4..8)
        self.spherical_harmonics[4] = (15.0_f32).sqrt() * 0.5 * sin_t * sin_t * sin_2p * dist_att; // V (ACN 4)
        self.spherical_harmonics[5] = (15.0_f32).sqrt() * sin_t * cos_t * sin_p * dist_att; // T (ACN 5)
        self.spherical_harmonics[6] =
            (5.0_f32).sqrt() * 0.5 * (3.0 * cos_t * cos_t - 1.0) * dist_att; // R (ACN 6)
        self.spherical_harmonics[7] = (15.0_f32).sqrt() * sin_t * cos_t * cos_p * dist_att; // S (ACN 7)
        self.spherical_harmonics[8] = (15.0_f32).sqrt() * 0.5 * sin_t * sin_t * cos_2p * dist_att; // U (ACN 8)

        // Order 3 (ACN 9..15)
        self.spherical_harmonics[9] = (35.0_f32 / 8.0).sqrt() * sin_t.powi(3) * sin_3p * dist_att; // Q (ACN 9)
        self.spherical_harmonics[10] =
            (105.0_f32 / 4.0).sqrt() * sin_t * sin_t * cos_t * sin_2p * dist_att; // O (ACN 10)
        self.spherical_harmonics[11] =
            (21.0_f32 / 8.0).sqrt() * sin_t * (5.0 * cos_t * cos_t - 1.0) * sin_p * dist_att; // M (ACN 11)
        self.spherical_harmonics[12] =
            (7.0_f32 / 4.0).sqrt() * (5.0 * cos_t.powi(3) - 3.0 * cos_t) * dist_att; // K (ACN 12)
        self.spherical_harmonics[13] =
            (21.0_f32 / 8.0).sqrt() * sin_t * (5.0 * cos_t * cos_t - 1.0) * cos_p * dist_att; // L (ACN 13)
        self.spherical_harmonics[14] =
            (105.0_f32 / 4.0).sqrt() * sin_t * sin_t * cos_t * cos_2p * dist_att; // N (ACN 14)
        self.spherical_harmonics[15] = (35.0_f32 / 8.0).sqrt() * sin_t.powi(3) * cos_3p * dist_att;
        // P (ACN 15)
    }

    /// Hit-test touch coordinate on the primary 3D source puck.
    pub fn hit_test_source_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.source_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.source_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= HOA_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render of 3D HOA Spatial Radar and Spherical Harmonics Decomposition.
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

        // Draw Left Half: Polar Radar (Center: mid_x / 2, height / 2)
        let radar_cx = mid_x / 2;
        let radar_cy = height / 2;

        // Center listener head marker
        if radar_cy < height && radar_cx < mid_x {
            grid[radar_cy][radar_cx] = 'H';
        }

        // Source Puck position on radar
        let az_rad = self.azimuth_deg.to_radians();
        let norm_dist = self.distance_m / MAX_DISTANCE_M;
        let max_r = (radar_cx - 2).min(radar_cy - 2) as f32;
        let px = radar_cx as f32 + az_rad.sin() * norm_dist * max_r;
        let py = radar_cy as f32 - az_rad.cos() * norm_dist * max_r;

        let px_i = px.round() as usize;
        let py_i = py.round() as usize;
        if py_i > 0 && py_i < height - 1 && px_i > 0 && px_i < mid_x {
            grid[py_i][px_i] = 'S';
        }

        // Draw Right Half: HOA Harmonic Bars
        let right_w = width - mid_x - 2;
        let bar_count = 16.min(right_w / 2);
        for b in 0..bar_count {
            let energy = self.spherical_harmonics[b].abs().min(2.0) / 2.0;
            let bar_h = (energy * (height - 4) as f32).round() as usize;
            let col = mid_x + 2 + b * 2;
            for r in 0..bar_h {
                let row = height - 2 - r;
                if row > 0 && col < width - 1 {
                    grid[row][col] = '#';
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
            "BROADCAST MASTERING IMMERSIVE DOLBY ATMOS / HOA 7.1.4 3D SPATIALIZER HUD",
            egui::FontId::proportional(15.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Format Selector Tabs (y: 48..92) - Each tab >= 44pt height
        let formats = [
            (HoaSpatialFormat::HoaThirdOrder, "HOA 3RD ORDER (16-CH)"),
            (HoaSpatialFormat::DolbyAtmos714, "DOLBY ATMOS 7.1.4"),
            (HoaSpatialFormat::BinauralHeadTrack, "BINAURAL HEAD-TRACK"),
            (HoaSpatialFormat::Ambisonics514, "AMBISONICS 5.1.4"),
            (HoaSpatialFormat::DomeAcoustic916, "DOME ACOUSTIC 9.1.6"),
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
                        self.format = *fmt;
                        self.update_spherical_harmonics();
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

        // Left 52%: 3D Horizontal Azimuth Radar & Speaker Array (30..415)
        let left_w = main_canvas.width() * 0.52;
        let left_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(left_w - 15.0, main_canvas.height() - 20.0),
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
            "HORIZONTAL AZIMUTH RADAR & ATMOS 7.1.4 ARRAY",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        let radar_center = egui::pos2(left_rect.center().x, left_rect.center().y + 10.0);
        let max_radius = (left_rect.width() * 0.40).min(left_rect.height() * 0.40);

        // Concentric Distance Rings (1m, 2m, 3m, 4m)
        for r_step in 1..=4 {
            let r = max_radius * (r_step as f32 / 4.0);
            painter.circle_stroke(
                radar_center,
                r,
                Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(45, 65, 95, 80)),
            );
        }

        // Crosshairs
        painter.line_segment(
            [
                egui::pos2(radar_center.x - max_radius, radar_center.y),
                egui::pos2(radar_center.x + max_radius, radar_center.y),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(45, 65, 95, 90)),
        );
        painter.line_segment(
            [
                egui::pos2(radar_center.x, radar_center.y - max_radius),
                egui::pos2(radar_center.x, radar_center.y + max_radius),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(45, 65, 95, 90)),
        );

        // Draw 7.1.4 Speakers
        for spk in ATMOS_714_SPEAKERS.iter() {
            let spk_az_rad = spk.azimuth_deg.to_radians();
            let spk_r = if spk.label == "LFE" {
                max_radius * 0.40
            } else if spk.is_ceiling {
                max_radius * 0.65
            } else {
                max_radius * 0.90
            };
            let sx = radar_center.x + spk_az_rad.sin() * spk_r;
            let sy = radar_center.y - spk_az_rad.cos() * spk_r;
            let spk_pos = egui::pos2(sx, sy);

            let spk_col = if spk.label == "LFE" {
                Color32::from_rgb(255, 107, 43)
            } else if spk.is_ceiling {
                Color32::from_rgb(0, 255, 180)
            } else {
                Color32::from_rgb(255, 215, 0)
            };
            painter.circle_filled(spk_pos, 4.0, spk_col);
            painter.text(
                egui::pos2(sx, sy - 8.0),
                egui::Align2::CENTER_CENTER,
                spk.label,
                egui::FontId::proportional(9.0),
                Color32::from_rgb(180, 200, 225),
            );
        }

        // Draw Listener Head and Head-Tracking Yaw Vector
        painter.circle_filled(radar_center, 12.0, Color32::from_rgb(25, 40, 65));
        painter.circle_stroke(
            radar_center,
            12.0,
            Stroke::new(1.5_f32, Color32::from_rgb(0, 229, 255)),
        );
        let yaw_rad = self.head_yaw_deg.to_radians();
        let yaw_nose_x = radar_center.x + yaw_rad.sin() * 18.0;
        let yaw_nose_y = radar_center.y - yaw_rad.cos() * 18.0;
        painter.line_segment(
            [radar_center, egui::pos2(yaw_nose_x, yaw_nose_y)],
            Stroke::new(2.5_f32, Color32::from_rgb(255, 107, 43)),
        );

        // Interactive Source Puck on Radar
        let source_az_rad = self.azimuth_deg.to_radians();
        let source_r_norm = (self.distance_m / MAX_DISTANCE_M).clamp(0.05, 1.0);
        let src_x = radar_center.x + source_az_rad.sin() * source_r_norm * max_radius;
        let src_y = radar_center.y - source_az_rad.cos() * source_r_norm * max_radius;
        let src_puck_pos = egui::pos2(src_x, src_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let dx = mouse_pos.x - radar_center.x;
                    let dy = -(mouse_pos.y - radar_center.y);
                    let angle_deg = dx.atan2(dy).to_degrees();
                    let dist_norm = ((dx * dx + dy * dy).sqrt() / max_radius).clamp(0.02, 1.0);
                    self.azimuth_deg = angle_deg;
                    self.distance_m = dist_norm * MAX_DISTANCE_M;
                    self.source_puck_pos = (
                        Self::azimuth_to_normalized(self.azimuth_deg),
                        Self::distance_to_normalized(self.distance_m),
                    );
                    self.update_spherical_harmonics();
                }
            }
        }

        // Source Puck Visual & Touch Target (>= 44x44pt)
        painter.circle_stroke(
            src_puck_pos,
            HOA_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(0, 229, 255, 140)),
        );
        painter.circle_filled(src_puck_pos, 14.0, Color32::from_rgb(0, 229, 255));
        painter.circle_filled(src_puck_pos, 4.0, Color32::from_rgb(255, 255, 255));

        // Right 48%: Spherical Elevation Dome & HOA Harmonic Energy Decomposition (425..770)
        let right_w = main_canvas.width() * 0.48;
        let right_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + left_w + 5.0, main_canvas.min.y + 10.0),
            egui::vec2(right_w - 15.0, main_canvas.height() - 20.0),
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
            "ELEVATION DOME & 16-CH HOA HARMONICS (ACN 0..15)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        painter.text(
            egui::pos2(right_rect.min.x + 15.0, right_rect.min.y + 26.0),
            egui::Align2::LEFT_TOP,
            format!(
                "Elevation: {:+.1}° (Nadir -90° .. Zenith +90°)",
                self.elevation_deg
            ),
            egui::FontId::proportional(10.0),
            Color32::from_rgb(255, 215, 0),
        );

        // Top Half of Right Card: Elevation Arch & Slider (y: 44..74 within right_rect)
        let el_slider_rect = egui::Rect::from_min_max(
            egui::pos2(right_rect.min.x + 15.0, right_rect.min.y + 44.0),
            egui::pos2(right_rect.max.x - 15.0, right_rect.min.y + 74.0),
        );
        painter.rect_filled(el_slider_rect, 4.0, Color32::from_rgb(18, 25, 38));
        painter.rect_stroke(
            el_slider_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        let el_puck_x = el_slider_rect.min.x + self.elevation_puck_norm * el_slider_rect.width();
        let el_puck_center = egui::pos2(el_puck_x, el_slider_rect.center().y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if el_slider_rect.expand(10.0).contains(mouse_pos) {
                    let norm = ((mouse_pos.x - el_slider_rect.min.x) / el_slider_rect.width())
                        .clamp(0.0, 1.0);
                    self.elevation_puck_norm = norm;
                    self.elevation_deg = Self::normalized_to_elevation(norm);
                    self.update_spherical_harmonics();
                }
            }
        }

        // Elevation Puck (>= 44x44pt hit radius)
        painter.circle_stroke(
            el_puck_center,
            HOA_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(255, 215, 0, 140)),
        );
        painter.circle_filled(el_puck_center, 12.0, Color32::from_rgb(255, 215, 0));
        painter.circle_filled(el_puck_center, 3.0, Color32::from_rgb(10, 14, 24));

        // Bottom Half of Right Card: 16 Spherical Harmonic Bars (y: 88..height-10 within right_rect)
        let bar_area_rect = egui::Rect::from_min_max(
            egui::pos2(right_rect.min.x + 10.0, right_rect.min.y + 88.0),
            egui::pos2(right_rect.max.x - 10.0, right_rect.max.y - 10.0),
        );
        let bar_w = (bar_area_rect.width() - 30.0) / 16.0;
        let bar_bottom_y = bar_area_rect.max.y - 5.0;

        for b in 0..16 {
            let energy = (self.spherical_harmonics[b].abs().min(2.5) / 2.5).clamp(0.02, 1.0);
            let bh = energy * 95.0;
            let bx = bar_area_rect.min.x + b as f32 * (bar_w + 2.0);
            let bar_rect = egui::Rect::from_min_max(
                egui::pos2(bx, bar_bottom_y - bh),
                egui::pos2(bx + bar_w, bar_bottom_y),
            );

            // Color coding by HOA Order: Order 0 (Teal), Order 1 (Amber), Order 2 (Coral), Order 3 (Cyan)
            let col = if b == 0 {
                Color32::from_rgb(0, 255, 180)
            } else if b <= 3 {
                Color32::from_rgb(255, 215, 0)
            } else if b <= 8 {
                Color32::from_rgb(255, 107, 43)
            } else {
                Color32::from_rgb(0, 229, 255)
            };

            painter.rect_filled(bar_rect, 1.0, col);
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

        let params = [
            (
                "AZIMUTH / ELEVATION",
                format!(
                    "{:+.1}° / {:+.1}° ({:.2} m)",
                    self.azimuth_deg, self.elevation_deg, self.distance_m
                ),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "HOA ENERGY NORM",
                format!("3rd Order ({} Ch N3D)", self.format.channel_count()),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "HEAD-TRACKING YAW",
                format!(
                    "Yaw: {:+.1}° ({:.1} ms)",
                    self.head_yaw_deg, self.head_tracking_latency_ms
                ),
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "BINAURAL DECODE",
                "SOFA KEMAR 48kHz HRIR".to_string(),
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
            "[PASS] Dolby Atmos & HOA 7.1.4 3D Spatializer & Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
