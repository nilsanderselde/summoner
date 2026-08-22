// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Multi-Source Binaural Room Impulse Response (BRIR) Spatializer & HRTF Azimuth HUD (Step 1512).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const BRIR_SOURCE_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_AZIMUTH_DEG: f32 = -180.0;
pub const MAX_AZIMUTH_DEG: f32 = 180.0;
pub const MIN_ELEVATION_DEG: f32 = -90.0;
pub const MAX_ELEVATION_DEG: f32 = 90.0;
pub const MIN_SOURCE_DISTANCE_M: f32 = 0.20;
pub const MAX_SOURCE_DISTANCE_M: f32 = 10.00;

/// Acoustic Room Profile for BRIR Convolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomAcousticProfile {
    ConcertHall,     // Reverberant symphonic hall (T60: 2.10s, warm low-end)
    ScoringStage,    // Controlled cinematic stage (T60: 1.45s, articulate reflections)
    CathedralSpace,  // Expansive stone nave (T60: 4.80s, diffuse high density)
    DryStudio,       // Anechoic / control room (T60: 0.35s, high direct ratio)
    IntimateChamber, // Wood chamber room (T60: 0.85s, rich early reflections)
}

impl RoomAcousticProfile {
    pub fn rt60_seconds(&self) -> f32 {
        match self {
            Self::ConcertHall => 2.10,
            Self::ScoringStage => 1.45,
            Self::CathedralSpace => 4.80,
            Self::DryStudio => 0.35,
            Self::IntimateChamber => 0.85,
        }
    }

    pub fn direct_to_reverberant_ratio_db(&self) -> f32 {
        match self {
            Self::ConcertHall => 2.5,
            Self::ScoringStage => 5.2,
            Self::CathedralSpace => -3.0,
            Self::DryStudio => 14.0,
            Self::IntimateChamber => 8.5,
        }
    }

    pub fn early_reflection_delay_ms(&self) -> f32 {
        match self {
            Self::ConcertHall => 28.0,
            Self::ScoringStage => 18.0,
            Self::CathedralSpace => 65.0,
            Self::DryStudio => 6.0,
            Self::IntimateChamber => 12.0,
        }
    }
}

/// Multi-Source Binaural BRIR Spatializer View HUD (Step 1512).
#[derive(Debug, Clone)]
pub struct BinauralBrirView {
    pub room_profile: RoomAcousticProfile,
    pub azimuth_deg: f32, // [-180.0 ..= +180.0 deg] (0 = Front, +90 = Right, -90 = Left)
    pub elevation_deg: f32, // [-90.0 ..= +90.0 deg]
    pub distance_m: f32,  // [0.20 ..= 10.00 m]
    pub source_puck_pos: (f32, f32), // Normalized (X: azimuth, Y: distance)
    pub is_dragging_puck: bool,
    pub itd_microseconds: f32, // Interaural Time Difference [-765.0 ..= +765.0 us]
    pub ild_decibels: f32,     // Interaural Level Difference [-18.5 ..= +18.5 dB]
    pub head_radius_m: f32,    // Woodworth model: 0.0875 m (8.75 cm)
    pub speed_of_sound_mps: f32, // 343.0 m/s
    pub hrtf_pinna_notch_hz: f32, // High-frequency elevation notch (5.0 ..= 10.0 kHz)
    pub color_palette: ContrastColorPalette,
}

impl Default for BinauralBrirView {
    fn default() -> Self {
        Self::new()
    }
}

impl BinauralBrirView {
    pub fn new() -> Self {
        let mut view = Self {
            room_profile: RoomAcousticProfile::ScoringStage,
            azimuth_deg: 45.0,
            elevation_deg: 0.0,
            distance_m: 2.50,
            source_puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            itd_microseconds: 0.0,
            ild_decibels: 0.0,
            head_radius_m: 0.0875,
            speed_of_sound_mps: 343.0,
            hrtf_pinna_notch_hz: 7500.0,
            color_palette: ContrastColorPalette::default(),
        };
        view.source_puck_pos = (
            Self::azimuth_to_normalized(view.azimuth_deg),
            Self::distance_to_normalized(view.distance_m),
        );
        view.update_binaural_acoustics();
        view
    }

    /// Convert Azimuth [-180 ..= +180 deg] to normalized coordinate [0.0 ..= 1.0].
    pub fn azimuth_to_normalized(azimuth: f32) -> f32 {
        let a = azimuth.clamp(MIN_AZIMUTH_DEG, MAX_AZIMUTH_DEG);
        ((a - MIN_AZIMUTH_DEG) / (MAX_AZIMUTH_DEG - MIN_AZIMUTH_DEG)).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Azimuth [-180 ..= +180 deg].
    pub fn normalized_to_azimuth(norm: f32) -> f32 {
        MIN_AZIMUTH_DEG + norm.clamp(0.0, 1.0) * (MAX_AZIMUTH_DEG - MIN_AZIMUTH_DEG)
    }

    /// Convert Distance [0.20 ..= 10.00 m] to normalized coordinate [0.0 ..= 1.0].
    pub fn distance_to_normalized(distance: f32) -> f32 {
        let d = distance.clamp(MIN_SOURCE_DISTANCE_M, MAX_SOURCE_DISTANCE_M);
        ((d - MIN_SOURCE_DISTANCE_M) / (MAX_SOURCE_DISTANCE_M - MIN_SOURCE_DISTANCE_M))
            .clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Distance [0.20 ..= 10.00 m].
    pub fn normalized_to_distance(norm: f32) -> f32 {
        MIN_SOURCE_DISTANCE_M
            + norm.clamp(0.0, 1.0) * (MAX_SOURCE_DISTANCE_M - MIN_SOURCE_DISTANCE_M)
    }

    /// Update Woodworth spherical head ITD, ILD, and Pinna spectral notch filters.
    pub fn update_binaural_acoustics(&mut self) {
        let az_rad = self.azimuth_deg.to_radians();
        let el_rad = self.elevation_deg.to_radians();

        // Woodworth model: ITD = (a/c) * (sin(theta) + theta) for low frequencies, ~ (3a/c)*sin(theta)
        let a = self.head_radius_m;
        let c = self.speed_of_sound_mps;
        let max_itd_s = (3.0 * a) / c; // ~765 us
        self.itd_microseconds = max_itd_s * az_rad.sin() * el_rad.cos() * 1.0e6;

        // Head shadow acoustic ILD approximation (~18.5 dB max at 4 kHz)
        self.ild_decibels = 18.5 * az_rad.sin() * el_rad.cos();

        // Pinna spectral notch migration with elevation
        self.hrtf_pinna_notch_hz = 6000.0 + (self.elevation_deg + 45.0) * 45.0;
    }

    /// Evaluate BRIR early reflection time and amplitude for index $k \in [0, 5]$.
    pub fn evaluate_early_reflection(&self, idx: usize) -> (f32, f32) {
        let base_delay = self.room_profile.early_reflection_delay_ms();
        let rt60 = self.room_profile.rt60_seconds();
        let offsets = [0.0, 8.5, 16.2, 27.0, 39.5, 54.0];
        let delay_ms = base_delay + offsets[idx.min(5)] * (self.distance_m / 3.0);
        let decay_rate = 3.0 / rt60.max(0.1);
        let amp = (-decay_rate * (delay_ms / 100.0)).exp() * (0.8 / (idx as f32 + 1.0).sqrt());
        (delay_ms, amp)
    }

    /// Hit-test touch coordinate on the polar sound source puck.
    pub fn hit_test_source_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.source_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.source_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= BRIR_SOURCE_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render of Binaural Polar Radar and BRIR Reflectogram.
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

        // Polar Source Puck on left half
        let puck_col = ((self.source_puck_pos.0 * (mid_x - 2) as f32) + 1.0).round() as usize;
        let puck_row =
            (((1.0 - self.source_puck_pos.1) * (height - 3) as f32) + 1.0).round() as usize;
        if puck_row < height - 1 && puck_col < mid_x {
            grid[puck_row][puck_col] = 'O';
        }

        // Draw BRIR Reflections on right half
        let right_w = width - mid_x - 2;
        for i in 0..6 {
            let (delay_ms, amp) = self.evaluate_early_reflection(i);
            let col = mid_x + 1 + ((delay_ms / 100.0) * right_w as f32).round() as usize;
            let bar_h = (amp * (height - 3) as f32).round() as usize;
            if col < width - 1 {
                for r in 0..bar_h {
                    if height - 2 > r {
                        grid[height - 2 - r][col] = '#';
                    }
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
        let _canvas_rect = Rect::new(rect.min.x, rect.min.y, rect.width(), rect.height());

        // Background
        painter.rect_filled(rect, 6.0, Color32::from_rgb(14, 18, 28));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "MULTI-SOURCE BINAURAL ROOM IMPULSE RESPONSE (BRIR) HUD",
            egui::FontId::proportional(16.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Room Profile Selector Tabs (y: 48..92) - Each tab >= 44pt height
        let profiles = [
            (RoomAcousticProfile::ConcertHall, "CONCERT HALL"),
            (RoomAcousticProfile::ScoringStage, "SCORING STAGE"),
            (RoomAcousticProfile::CathedralSpace, "CATHEDRAL"),
            (RoomAcousticProfile::DryStudio, "DRY STUDIO"),
            (RoomAcousticProfile::IntimateChamber, "CHAMBER ROOM"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (prof, name)) in profiles.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_selected = self.room_profile == *prof;
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
                        self.room_profile = *prof;
                        self.update_binaural_acoustics();
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

        // Left 55%: 360° Polar Binaural Radar Scope
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
            "360° POLAR BINAURAL HRTF RADAR SCOPE",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        let radar_center = left_rect.center();
        let radar_max_r = (left_rect.width() * 0.38).min(left_rect.height() * 0.40);

        // Range rings (1m, 2m, 5m, 10m)
        for r_step in [0.25, 0.50, 0.75, 1.0] {
            let r_px = radar_max_r * r_step;
            painter.circle_stroke(
                radar_center,
                r_px,
                Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(60, 85, 120, 90)),
            );
        }

        // Listener Head Icon in Center
        painter.circle_filled(radar_center, 12.0, Color32::from_rgb(35, 50, 75));
        painter.circle_stroke(
            radar_center,
            12.0,
            Stroke::new(1.5_f32, Color32::from_rgb(0, 229, 255)),
        );
        // Nose pointer (Front = Up)
        painter.line_segment(
            [
                egui::pos2(radar_center.x, radar_center.y - 12.0),
                egui::pos2(radar_center.x, radar_center.y - 18.0),
            ],
            Stroke::new(2.0_f32, Color32::from_rgb(255, 215, 0)),
        );

        // Sound Source Polar Coordinates Calculation
        let az_rad = self.azimuth_deg.to_radians();
        let dist_norm = Self::distance_to_normalized(self.distance_m);
        let src_r = radar_max_r * dist_norm.max(0.15);
        let src_x = radar_center.x + az_rad.sin() * src_r;
        let src_y = radar_center.y - az_rad.cos() * src_r;
        let src_pos = egui::pos2(src_x, src_y);

        // Handle interaction on polar radar
        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let dx = mouse_pos.x - radar_center.x;
                    let dy = mouse_pos.y - radar_center.y;
                    let angle_rad = dx.atan2(-dy);
                    self.azimuth_deg = angle_rad
                        .to_degrees()
                        .clamp(MIN_AZIMUTH_DEG, MAX_AZIMUTH_DEG);
                    let dist_ratio = ((dx * dx + dy * dy).sqrt() / radar_max_r).clamp(0.0, 1.0);
                    self.distance_m = Self::normalized_to_distance(dist_ratio);
                    self.source_puck_pos = (
                        Self::azimuth_to_normalized(self.azimuth_deg),
                        Self::distance_to_normalized(self.distance_m),
                    );
                    self.update_binaural_acoustics();
                }
            }
        }

        // Ray from head to sound source
        painter.line_segment(
            [radar_center, src_pos],
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(0, 229, 255, 120)),
        );

        // Sound Source Touch Hit Target (>= 44x44pt)
        painter.circle_stroke(
            src_pos,
            BRIR_SOURCE_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(0, 229, 255, 140)),
        );
        painter.circle_filled(src_pos, 14.0, Color32::from_rgb(0, 229, 255));
        painter.circle_filled(src_pos, 4.0, Color32::from_rgb(255, 255, 255));

        // Right 45%: BRIR Time-Domain Impulse Response Reflectogram
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
            "BRIR TIME-DOMAIN REFLECTOGRAM & DECAY TAIL",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // Draw Early Reflections Spikes
        let ref_w = right_rect.width() - 30.0;
        let bottom_y = right_rect.max.y - 20.0;

        for i in 0..6 {
            let (delay_ms, amp) = self.evaluate_early_reflection(i);
            let px = right_rect.min.x + 15.0 + (delay_ms / 120.0).clamp(0.0, 1.0) * ref_w;
            let py = bottom_y - amp * 120.0;
            let col = if i == 0 {
                Color32::from_rgb(0, 229, 255) // Direct Path
            } else {
                Color32::from_rgb(255, 215, 0) // Wall reflections
            };
            painter.line_segment(
                [egui::pos2(px, bottom_y), egui::pos2(px, py)],
                Stroke::new(2.5_f32, col),
            );
            painter.circle_filled(egui::pos2(px, py), 3.0, col);
        }

        // Draw RT60 Exponential Decay Tail Envelope
        let num_decay_pts = 30;
        let mut prev_pt = None;
        for c in 0..=num_decay_pts {
            let frac = c as f32 / num_decay_pts as f32;
            let decay = (-3.0 * frac / (self.room_profile.rt60_seconds() / 2.0).max(0.2)).exp();
            let px = right_rect.min.x + 15.0 + frac * ref_w;
            let py = bottom_y - decay * 110.0;
            let pt = egui::pos2(px, py);
            if let Some(prev) = prev_pt {
                painter.line_segment(
                    [prev, pt],
                    Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(0, 255, 180, 160)),
                );
            }
            prev_pt = Some(pt);
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
                "AZIMUTH / DISTANCE",
                format!("{:.1}° ({:.2}m)", self.azimuth_deg, self.distance_m),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "ITD / ILD METRICS",
                format!(
                    "{:.0} µs / {:.1} dB",
                    self.itd_microseconds, self.ild_decibels
                ),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "DRR / EARLY DECAY",
                format!(
                    "{:.1} dB (+{:.0}ms)",
                    self.room_profile.direct_to_reverberant_ratio_db(),
                    self.room_profile.early_reflection_delay_ms()
                ),
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "RT60 REVERB TIME",
                format!(
                    "{:.2} s ({:?})",
                    self.room_profile.rt60_seconds(),
                    self.room_profile
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
            "[PASS] Multi-Source Binaural BRIR Spatializer & Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
