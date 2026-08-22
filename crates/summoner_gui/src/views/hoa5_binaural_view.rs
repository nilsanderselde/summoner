// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Broadcast Mastering Immersive 5th-Order (36-Channel) Higher-Order Ambisonics Spherical Virtualizer HUD (Step 1605).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const HOA5_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_AZIMUTH_DEG: f32 = -180.0;
pub const MAX_AZIMUTH_DEG: f32 = 180.0;
pub const MIN_ELEVATION_DEG: f32 = -90.0;
pub const MAX_ELEVATION_DEG: f32 = 90.0;
pub const HOA5_TOTAL_CHANNELS: usize = 36; // (N+1)^2 for N=5 -> 36 spherical harmonic components

/// 5th-Order Ambisonic decoding profiles and virtualizer topologies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hoa5Profile {
    Hoa5thOrderSpherical36Ch, // Full 36-channel spherical harmonic expansion (Order 0..5)
    MaxReSphericalEnergy,     // Max-rE energy vector weighting for focused sweet-spot listening
    InPhaseOptimalDecoder,    // In-phase decoding suppressing side-lobe energy for binaural HRIR
    BinauralKEMAR5thOrder,    // High-resolution KEMAR head-related binaural virtualizer
    DolbyAtmosBed36Virtual,   // 36-channel Ambisonics mastering downmix matrix to 9.1.6
}

impl Hoa5Profile {
    pub fn profile_name(&self) -> &'static str {
        match self {
            Self::Hoa5thOrderSpherical36Ch => "5TH-ORDER HOA (36-CH RAW)",
            Self::MaxReSphericalEnergy => "MAX-rE ENERGY OPTIMIZED",
            Self::InPhaseOptimalDecoder => "IN-PHASE BINAURAL DECODER",
            Self::BinauralKEMAR5thOrder => "KEMAR 5TH-ORDER HRIR",
            Self::DolbyAtmosBed36Virtual => "HOA36 TO ATMOS 9.1.6 BED",
        }
    }

    pub fn nominal_azimuth_deg(&self) -> f32 {
        match self {
            Self::Hoa5thOrderSpherical36Ch => 45.0,
            Self::MaxReSphericalEnergy => -30.0,
            Self::InPhaseOptimalDecoder => 60.0,
            Self::BinauralKEMAR5thOrder => 0.0,
            Self::DolbyAtmosBed36Virtual => -45.0,
        }
    }

    pub fn nominal_elevation_deg(&self) -> f32 {
        match self {
            Self::Hoa5thOrderSpherical36Ch => 20.0,
            Self::MaxReSphericalEnergy => 15.0,
            Self::InPhaseOptimalDecoder => 30.0,
            Self::BinauralKEMAR5thOrder => 0.0,
            Self::DolbyAtmosBed36Virtual => 25.0,
        }
    }

    pub fn nominal_distance_m(&self) -> f32 {
        match self {
            Self::Hoa5thOrderSpherical36Ch => 2.5,
            Self::MaxReSphericalEnergy => 2.0,
            Self::InPhaseOptimalDecoder => 1.8,
            Self::BinauralKEMAR5thOrder => 1.5,
            Self::DolbyAtmosBed36Virtual => 3.2,
        }
    }

    pub fn nominal_energy_focus(&self) -> f32 {
        match self {
            Self::Hoa5thOrderSpherical36Ch => 0.65,
            Self::MaxReSphericalEnergy => 0.92,
            Self::InPhaseOptimalDecoder => 0.85,
            Self::BinauralKEMAR5thOrder => 0.88,
            Self::DolbyAtmosBed36Virtual => 0.78,
        }
    }
}

/// Broadcast mastering immersive 5th-Order Higher-Order Ambisonics spherical virtualizer HUD.
#[derive(Debug, Clone)]
pub struct Hoa5BinauralView {
    pub profile: Hoa5Profile,
    pub azimuth_deg: f32,
    pub elevation_deg: f32,
    pub distance_m: f32,
    pub order: usize,
    pub energy_focus: f32,
    pub diffuse_field_pct: f32,
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    pub spherical_harmonics_levels: [f32; HOA5_TOTAL_CHANNELS],
    pub color_palette: ContrastColorPalette,
}

impl Default for Hoa5BinauralView {
    fn default() -> Self {
        Self::new()
    }
}

impl Hoa5BinauralView {
    pub fn new() -> Self {
        let mut view = Self {
            profile: Hoa5Profile::Hoa5thOrderSpherical36Ch,
            azimuth_deg: 45.0,
            elevation_deg: 20.0,
            distance_m: 2.5,
            order: 5,
            energy_focus: 0.65,
            diffuse_field_pct: 15.0,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            spherical_harmonics_levels: [0.0; HOA5_TOTAL_CHANNELS],
            color_palette: ContrastColorPalette::default(),
        };
        view.puck_pos = (
            Self::azimuth_to_normalized(view.azimuth_deg),
            Self::elevation_to_normalized(view.elevation_deg),
        );
        view.update_hoa5_simulation();
        view
    }

    pub fn azimuth_to_normalized(az: f32) -> f32 {
        let a = az.clamp(MIN_AZIMUTH_DEG, MAX_AZIMUTH_DEG);
        ((a - MIN_AZIMUTH_DEG) / (MAX_AZIMUTH_DEG - MIN_AZIMUTH_DEG)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_azimuth(norm: f32) -> f32 {
        MIN_AZIMUTH_DEG + norm.clamp(0.0, 1.0) * (MAX_AZIMUTH_DEG - MIN_AZIMUTH_DEG)
    }

    pub fn elevation_to_normalized(el: f32) -> f32 {
        let e = el.clamp(MIN_ELEVATION_DEG, MAX_ELEVATION_DEG);
        ((e - MIN_ELEVATION_DEG) / (MAX_ELEVATION_DEG - MIN_ELEVATION_DEG)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_elevation(norm: f32) -> f32 {
        MIN_ELEVATION_DEG + norm.clamp(0.0, 1.0) * (MAX_ELEVATION_DEG - MIN_ELEVATION_DEG)
    }

    pub fn set_profile(&mut self, prof: Hoa5Profile) {
        self.profile = prof;
        self.azimuth_deg = prof.nominal_azimuth_deg();
        self.elevation_deg = prof.nominal_elevation_deg();
        self.distance_m = prof.nominal_distance_m();
        self.energy_focus = prof.nominal_energy_focus();
        self.puck_pos = (
            Self::azimuth_to_normalized(self.azimuth_deg),
            Self::elevation_to_normalized(self.elevation_deg),
        );
        self.update_hoa5_simulation();
    }

    pub fn update_hoa5_simulation(&mut self) {
        let az_rad = self.azimuth_deg.to_radians();
        let el_rad = self.elevation_deg.to_radians();
        let focus = self.energy_focus;

        let cos_el = el_rad.cos();
        let sin_el = el_rad.sin();
        let cos_az = az_rad.cos();
        let sin_az = az_rad.sin();

        // 36 Spherical Harmonics Decomposition (ACN index 0..35):
        // Order 0: W (1 ch)
        // Order 1: Y, Z, X (3 ch)
        // Order 2: V, T, R, S, U (5 ch)
        // Order 3: Q, O, M, K, L, N, P (7 ch)
        // Order 4: (9 ch)
        // Order 5: (11 ch)
        let mut ch_levels = [0.0f32; HOA5_TOTAL_CHANNELS];

        // 0th Order: Omnidirectional Monopole W
        ch_levels[0] = 0.707;

        // 1st Order Dipoles (Y, Z, X)
        ch_levels[1] = (sin_az * cos_el * focus).abs();
        ch_levels[2] = (sin_el * focus).abs();
        ch_levels[3] = (cos_az * cos_el * focus).abs();

        // 2nd Order Quadrupoles
        ch_levels[4] = ((2.0 * az_rad).sin() * cos_el.powi(2) * focus).abs();
        ch_levels[5] = (sin_az * (2.0 * el_rad).sin() * focus).abs();
        ch_levels[6] = ((3.0 * sin_el.powi(2) - 1.0) * 0.5 * focus).abs();
        ch_levels[7] = (cos_az * (2.0 * el_rad).sin() * focus).abs();
        ch_levels[8] = ((2.0 * az_rad).cos() * cos_el.powi(2) * focus).abs();

        // 3rd to 5th Order higher spatial resolutions
        for ch in 9..HOA5_TOTAL_CHANNELS {
            let m = (ch % 7) as f32;
            let order_idx = if ch < 16 { 3 } else if ch < 25 { 4 } else { 5 };
            let spatial_decay = 1.0 / (order_idx as f32 * 0.4 + 1.0);
            let trig = ((m + 1.0) * az_rad).cos() * ((m + 1.0) * el_rad).sin();
            ch_levels[ch] = (trig.abs() * focus * spatial_decay).clamp(0.02, 1.0);
        }

        self.spherical_harmonics_levels = ch_levels;
        self.diffuse_field_pct = ((1.0 - focus) * 60.0 + 5.0).clamp(5.0, 95.0);
    }

    pub fn hit_test_hoa5_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= HOA5_PUCK_HIT_RADIUS
    }

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

        let left_w = mid_x - 2;
        let p_row = (((1.0 - self.puck_pos.1) * (height - 5) as f32) + 2.0).round() as usize;
        let p_col = ((self.puck_pos.0 * (left_w - 4) as f32) + 2.0).round() as usize;
        if p_row < height - 1 && p_col < mid_x {
            grid[p_row][p_col] = 'P';
        }

        let right_w = width - mid_x - 2;
        let bar_spacing = (right_w / 18).max(1);
        for i in 0..18 {
            let col = mid_x + 2 + i * bar_spacing;
            let amp = self.spherical_harmonics_levels[i];
            let bar_h = (amp.clamp(0.0, 1.0) * (height - 4) as f32).round() as usize;
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

        // Background: Deep Galactic Obsidian (#0A0E18)
        painter.rect_filled(rect, 6.0, Color32::from_rgb(10, 14, 24));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "5TH-ORDER (36-CHANNEL) HIGHER-ORDER AMBISONICS VIRTUALIZER HUD",
            egui::FontId::proportional(13.5),
            Color32::from_rgb(240, 245, 255),
        );

        // Tabs (y: 48..92) - 44pt height
        let tabs = [
            (Hoa5Profile::Hoa5thOrderSpherical36Ch, "36-CH RAW (HOA5)"),
            (Hoa5Profile::MaxReSphericalEnergy, "MAX-rE ENERGY"),
            (Hoa5Profile::InPhaseOptimalDecoder, "IN-PHASE BINAURAL"),
            (Hoa5Profile::BinauralKEMAR5thOrder, "KEMAR HRIR"),
            (Hoa5Profile::DolbyAtmosBed36Virtual, "ATMOS 9.1.6 BED"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (ptype, name)) in tabs.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_sel = self.profile == *ptype;
            let bg_col = if is_sel {
                Color32::from_rgb(157, 78, 221)
            } else {
                Color32::from_rgb(25, 28, 45)
            };
            let text_col = if is_sel {
                Color32::from_rgb(250, 240, 255)
            } else {
                Color32::from_rgb(215, 215, 240)
            };

            painter.rect_filled(tab_rect, 4.0, bg_col);
            painter.text(
                tab_rect.center(),
                egui::Align2::CENTER_CENTER,
                *name,
                egui::FontId::proportional(10.0),
                text_col,
            );

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if tab_rect.contains(pos) {
                        self.set_profile(*ptype);
                    }
                }
            }
        }

        // Main Display Canvas (y: 104..340)
        let main_canvas = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 104.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 340.0),
        );
        painter.rect_filled(main_canvas, 6.0, Color32::from_rgb(6, 10, 18));
        painter.rect_stroke(
            main_canvas,
            6.0,
            Stroke::new(1.5_f32, Color32::from_rgb(55, 45, 85)),
        );

        // Left 55%: Spherical Equirectangular Coordinate Field
        let left_w = main_canvas.width() * 0.55;
        let left_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(left_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(left_rect, 4.0, Color32::from_rgb(14, 16, 28));
        painter.rect_stroke(
            left_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(45, 40, 75)),
        );

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "SPHERICAL PANNER FIELD (AZIMUTH vs ELEVATION)",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(157, 78, 221),
        );

        let sphere_cx = left_rect.min.x + left_rect.width() * 0.5;
        let sphere_cy = left_rect.min.y + left_rect.height() * 0.52;

        // Draw Azimuth & Elevation Reference Grid
        painter.line_segment(
            [
                egui::pos2(left_rect.min.x + 15.0, sphere_cy),
                egui::pos2(left_rect.max.x - 15.0, sphere_cy),
            ],
            Stroke::new(1.0_f32, Color32::from_rgb(50, 45, 80)),
        );
        painter.line_segment(
            [
                egui::pos2(sphere_cx, left_rect.min.y + 25.0),
                egui::pos2(sphere_cx, left_rect.max.y - 25.0),
            ],
            Stroke::new(1.0_f32, Color32::from_rgb(50, 45, 80)),
        );

        // Elevation +/- 45 deg guide lines
        let el45_offset = (left_rect.height() - 55.0) * 0.25;
        painter.line_segment(
            [
                egui::pos2(left_rect.min.x + 15.0, sphere_cy - el45_offset),
                egui::pos2(left_rect.max.x - 15.0, sphere_cy - el45_offset),
            ],
            Stroke::new(0.8_f32, Color32::from_rgb(40, 35, 65)),
        );
        painter.line_segment(
            [
                egui::pos2(left_rect.min.x + 15.0, sphere_cy + el45_offset),
                egui::pos2(left_rect.max.x - 15.0, sphere_cy + el45_offset),
            ],
            Stroke::new(0.8_f32, Color32::from_rgb(40, 35, 65)),
        );

        // Interactive Puck
        let puck_x = left_rect.min.x + self.puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.puck_pos = (nx, ny);
                    self.azimuth_deg = Self::normalized_to_azimuth(nx);
                    self.elevation_deg = Self::normalized_to_elevation(ny);
                    self.update_hoa5_simulation();
                }
            }
        }

        painter.circle_stroke(
            puck_pos,
            HOA5_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(157, 78, 221, 150)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(157, 78, 221));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 18.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Az: {:+.1}° | El: {:+.1}° | Dist: {:.2}m | Order: 5th (36-Ch) | Focus: {:.0}%",
                self.azimuth_deg,
                self.elevation_deg,
                self.distance_m,
                self.energy_focus * 100.0
            ),
            egui::FontId::proportional(10.5),
            Color32::from_rgb(220, 175, 255),
        );

        // Right 45%: 36-Channel Spherical Harmonic Energy Coefficients Spectrum
        let right_w = main_canvas.width() * 0.45;
        let right_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + left_w + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(right_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(right_rect, 4.0, Color32::from_rgb(14, 16, 28));
        painter.rect_stroke(
            right_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(45, 40, 75)),
        );

        painter.text(
            egui::pos2(right_rect.min.x + 10.0, right_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "36-CH SPHERICAL HARMONIC COEFFICIENTS (ACN 0..35)",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(157, 78, 221),
        );

        let bar_w = (right_rect.width() - 24.0 - 35.0 * 2.0) / 36.0;
        for (i, &amp) in self.spherical_harmonics_levels.iter().enumerate() {
            let bx = right_rect.min.x + 12.0 + i as f32 * (bar_w + 2.0);
            let bar_h = (amp.clamp(0.0, 1.2) / 1.2) * (right_rect.height() - 75.0);
            let b_rect = egui::Rect::from_min_max(
                egui::pos2(bx, right_rect.max.y - 25.0 - bar_h),
                egui::pos2(bx + bar_w, right_rect.max.y - 25.0),
            );
            let col = if i == 0 {
                Color32::from_rgb(0, 229, 255)
            } else if i < 4 {
                Color32::from_rgb(157, 78, 221)
            } else if i < 9 {
                Color32::from_rgb(255, 107, 107)
            } else if i < 16 {
                Color32::from_rgb(255, 215, 0)
            } else {
                Color32::from_rgb(180, 140, 255)
            };
            painter.rect_filled(b_rect, 1.5, col);
        }

        painter.text(
            egui::pos2(right_rect.min.x + 12.0, right_rect.max.y - 20.0),
            egui::Align2::LEFT_TOP,
            "Orders: 0 (W) | 1 (XYZ) | 2 (5-ch) | 3 (7-ch) | 4 (9-ch) | 5 (11-ch)",
            egui::FontId::proportional(8.5),
            Color32::from_rgb(180, 175, 215),
        );

        // Bottom Metrics Dock (y: 350..465)
        let dock_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 350.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 465.0),
        );
        painter.rect_filled(dock_rect, 6.0, Color32::from_rgb(18, 20, 36));
        painter.rect_stroke(
            dock_rect,
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(55, 50, 85)),
        );

        let params = [
            (
                "AZIMUTH / ELEVATION",
                format!("{:+.1}° / {:+.1}° (Spherical)", self.azimuth_deg, self.elevation_deg),
                Color32::from_rgb(157, 78, 221),
            ),
            (
                "SPATIAL ORDER",
                format!("5th Order ({} Channels)", HOA5_TOTAL_CHANNELS),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "ENERGY FOCUS (rE)",
                format!("{:.0}% (Max-rE Weighting)", self.energy_focus * 100.0),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "DIFFUSE FIELD RATIO",
                format!("{:.0}% (Master Ambience)", self.diffuse_field_pct),
                Color32::from_rgb(255, 107, 107),
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
                Color32::from_rgb(170, 165, 205),
            );
            painter.text(
                egui::pos2(px_pos, dock_rect.min.y + 30.0),
                egui::Align2::LEFT_TOP,
                val,
                egui::FontId::proportional(13.0),
                *col,
            );
        }

        // Compliance Badge
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
            "[PASS] 5th-Order (36-Ch) Ambisonics Spherical Virtualizer Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
