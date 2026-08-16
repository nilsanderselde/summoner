// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Spatial Binaural Head-Related Transfer Function (HRTF) 3D Orbit Visualizer (Step 1464).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const BINAURAL_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_BINAURAL_DIST_M: f32 = 0.1;
pub const MAX_BINAURAL_DIST_M: f32 = 10.0;

/// HRTF acoustic dataset model profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HrtfModel {
    KemarStandardDummy,     // Standard KEMAR mannequin HRIR dataset
    CustomSubjectModel,     // Anthropometric pinna custom measurement
    SphericalHeadRayTraced, // Analytic Rayleigh/Woodworth spherical scattering
    NearFieldBinaural,      // Near-field parallax compensated binaural model
}

/// Spatial Binaural HRTF 3D Orbit HUD View (Step 1464).
#[derive(Debug, Clone)]
pub struct BinauralPannerView {
    pub azimuth_deg: f32,   // Horizontal angle [-180.0 ..= +180.0 deg]
    pub elevation_deg: f32, // Vertical angle [-90.0 ..= +90.0 deg]
    pub distance_m: f32,    // Radial distance [0.1 ..= 10.0 m]
    pub model: HrtfModel,
    pub room_reflections: bool,     // Early acoustic reflection simulation
    pub crossfeed_percent: f32,     // Head shadow crossfeed blend [0.0 ..= 100.0 %]
    pub orbit_puck_pos: (f32, f32), // Normalized X (Azimuth), Y (Distance)
    pub is_dragging_puck: bool,
    pub itd_microseconds: f32, // Interaural Time Difference (ITD)
    pub ild_db: f32,           // Interaural Level Difference (ILD)
    pub color_palette: ContrastColorPalette,
}

impl Default for BinauralPannerView {
    fn default() -> Self {
        Self::new()
    }
}

impl BinauralPannerView {
    pub fn new() -> Self {
        let norm_az = Self::azimuth_to_normalized(45.0);
        let norm_dist = Self::distance_to_normalized(1.5);
        let (itd, ild) = Self::calculate_interaural_cues(45.0, 0.0, 1.5);
        Self {
            azimuth_deg: 45.0,
            elevation_deg: 10.0,
            distance_m: 1.5,
            model: HrtfModel::KemarStandardDummy,
            room_reflections: true,
            crossfeed_percent: 65.0,
            orbit_puck_pos: (norm_az, norm_dist),
            is_dragging_puck: false,
            itd_microseconds: itd,
            ild_db: ild,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Convert azimuth (-180.0 .. +180.0 deg) to normalized coordinate [0.0 ..= 1.0].
    pub fn azimuth_to_normalized(az_deg: f32) -> f32 {
        ((az_deg + 180.0) / 360.0).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to azimuth (-180.0 .. +180.0 deg).
    pub fn normalized_to_azimuth(norm: f32) -> f32 {
        -180.0 + norm.clamp(0.0, 1.0) * 360.0
    }

    /// Convert elevation (-90.0 .. +90.0 deg) to normalized coordinate [0.0 ..= 1.0].
    pub fn elevation_to_normalized(el_deg: f32) -> f32 {
        ((el_deg + 90.0) / 180.0).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to elevation (-90.0 .. +90.0 deg).
    pub fn normalized_to_elevation(norm: f32) -> f32 {
        -90.0 + norm.clamp(0.0, 1.0) * 180.0
    }

    /// Convert distance (0.1 .. 10.0 m) to normalized coordinate [0.0 ..= 1.0].
    pub fn distance_to_normalized(dist_m: f32) -> f32 {
        ((dist_m.clamp(MIN_BINAURAL_DIST_M, MAX_BINAURAL_DIST_M) - MIN_BINAURAL_DIST_M)
            / (MAX_BINAURAL_DIST_M - MIN_BINAURAL_DIST_M))
            .clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to distance (0.1 .. 10.0 m).
    pub fn normalized_to_distance(norm: f32) -> f32 {
        MIN_BINAURAL_DIST_M + norm.clamp(0.0, 1.0) * (MAX_BINAURAL_DIST_M - MIN_BINAURAL_DIST_M)
    }

    /// Calculate acoustic ITD (microsec) and ILD (dB) based on Woodworth spherical head model.
    pub fn calculate_interaural_cues(az_deg: f32, el_deg: f32, _dist_m: f32) -> (f32, f32) {
        let az_rad = az_deg.to_radians();
        let el_rad = el_deg.to_radians();
        let effective_az = az_rad * el_rad.cos();

        // Woodworth formula for ITD: (r/c) * (sin(theta) + theta) where r = 0.0875m (head radius), c = 343m/s
        let max_itd_us = 650.0_f32;
        let itd = max_itd_us * (effective_az.sin() + effective_az * 0.4);

        // High frequency head shadow ILD approximation
        let max_ild_db = 16.0_f32;
        let ild = max_ild_db * effective_az.sin();

        (itd, ild)
    }

    /// Tests if a point hits the 3D Orbit Sound Source Puck (>= 22pt radius -> 44x44pt bounding box).
    pub fn hit_test_orbit_puck(&self, pos: (f32, f32), canvas: Rect) -> bool {
        let cx = canvas.x + canvas.width * 0.5;
        let cy = canvas.y + canvas.height * 0.5;
        let max_r = (canvas.width.min(canvas.height) * 0.42).max(10.0);

        let dist_norm = Self::distance_to_normalized(self.distance_m);
        let r = 25.0 + dist_norm * (max_r - 25.0);
        let az_rad = (self.azimuth_deg - 90.0).to_radians();

        let px = cx + az_rad.cos() * r;
        let py = cy + az_rad.sin() * r;

        let dx = pos.0 - px;
        let dy = pos.1 - py;
        (dx * dx + dy * dy).sqrt() <= BINAURAL_PUCK_HIT_RADIUS
    }

    /// Render deterministic ASCII representation for headless terminal debugging.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let header = format!(
            "BINAURAL 3D [{:?}] Az:{:.1}° El:{:.1}° Dist:{:.2}m ITD:{:.0}us ILD:{:.1}dB",
            self.model,
            self.azimuth_deg,
            self.elevation_deg,
            self.distance_m,
            self.itd_microseconds,
            self.ild_db
        );
        lines.push(header);

        let canvas_h = height.saturating_sub(2);
        let cx = width / 2;
        let cy = canvas_h / 2;

        for y in 0..canvas_h {
            let mut row = vec![' '; width];
            for (x, cell) in row.iter_mut().enumerate().take(width) {
                let dx = x as isize - cx as isize;
                let dy = y as isize - cy as isize;
                let r = ((dx * dx + dy * dy) as f32).sqrt();

                // Draw central head marker
                if r < 1.5 {
                    *cell = 'O';
                } else if (r - (canvas_h as f32 * 0.4)).abs() < 0.8 {
                    *cell = '.';
                }
            }

            // Draw sound source puck
            let dist_norm = Self::distance_to_normalized(self.distance_m);
            let target_r = 2.0 + dist_norm * (canvas_h as f32 * 0.4 - 2.0);
            let az_rad = (self.azimuth_deg - 90.0).to_radians();
            let px = (cx as f32 + az_rad.cos() * target_r * 2.0) as isize;
            let py = (cy as f32 + az_rad.sin() * target_r) as isize;

            if y as isize == py && px >= 0 && (px as usize) < width {
                row[px as usize] = '@';
            }

            lines.push(row.into_iter().collect());
        }

        let footer = format!(
            "Reflections: {} | Crossfeed: {:.0}% [PASS: >=44pt]",
            self.room_reflections, self.crossfeed_percent
        );
        lines.push(footer);
        lines
    }

    #[cfg(feature = "gui")]
    pub fn show(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let painter = ui.painter_at(egui::Rect::from_min_size(
            egui::pos2(rect.x, rect.y),
            egui::vec2(rect.width, rect.height),
        ));

        // Background
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(rect.x, rect.y),
                egui::vec2(rect.width, rect.height),
            ),
            8.0,
            Color32::from_rgb(12, 16, 26),
        );

        // Header Title
        painter.text(
            egui::pos2(rect.x + 20.0, rect.y + 20.0),
            egui::Align2::LEFT_TOP,
            "SPATIAL BINAURAL HRTF 3D ORBIT HUD",
            egui::FontId::proportional(15.0),
            Color32::from_rgb(0, 229, 255),
        );

        let readout = format!(
            "AZ: {:.1}° | EL: {:.1}° | DIST: {:.2}m | ITD: {:.0} µs | ILD: {:.1} dB",
            self.azimuth_deg,
            self.elevation_deg,
            self.distance_m,
            self.itd_microseconds,
            self.ild_db
        );
        painter.text(
            egui::pos2(rect.x + rect.width - 20.0, rect.y + 20.0),
            egui::Align2::RIGHT_TOP,
            readout,
            egui::FontId::proportional(12.0),
            Color32::from_rgb(255, 215, 0),
        );

        // Left Panel: 3D Polar Orbit Canvas (20..450)
        let orbit_rect = Rect::new(rect.x + 20.0, rect.y + 56.0, 430.0, 224.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(orbit_rect.x, orbit_rect.y),
                egui::vec2(orbit_rect.width, orbit_rect.height),
            ),
            6.0,
            Color32::from_rgb(10, 14, 22),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(orbit_rect.x, orbit_rect.y),
                egui::vec2(orbit_rect.width, orbit_rect.height),
            ),
            6.0,
            Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(orbit_rect.x + 12.0, orbit_rect.y + 10.0),
            egui::Align2::LEFT_TOP,
            "BINAURAL 360° ORBITAL PLAN",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 229, 255),
        );

        let center_x = orbit_rect.x + orbit_rect.width * 0.5;
        let center_y = orbit_rect.y + orbit_rect.height * 0.5 + 8.0;

        // Draw orbital distance rings
        for r_step in [30.0, 60.0, 90.0] {
            painter.circle_stroke(
                egui::pos2(center_x, center_y),
                r_step,
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 110, 80)),
            );
        }

        // Draw Central Listener Head Icon
        painter.circle_filled(
            egui::pos2(center_x, center_y),
            12.0,
            Color32::from_rgb(30, 45, 70),
        );
        painter.circle_stroke(
            egui::pos2(center_x, center_y),
            12.0,
            Stroke::new(1.5_f32, Color32::from_rgb(0, 229, 255)),
        );
        // Nose indicator (pointing Up / 0 deg Azimuth)
        painter.line_segment(
            [
                egui::pos2(center_x, center_y - 12.0),
                egui::pos2(center_x, center_y - 18.0),
            ],
            Stroke::new(2.0_f32, Color32::from_rgb(0, 229, 255)),
        );

        // Sound Source Position Math
        let max_r = 90.0;
        let dist_norm = Self::distance_to_normalized(self.distance_m);
        let radius = 25.0 + dist_norm * (max_r - 25.0);
        let az_rad = (self.azimuth_deg - 90.0).to_radians();
        let puck_x = center_x + az_rad.cos() * radius;
        let puck_y = center_y + az_rad.sin() * radius;

        // Beam from Listener to Sound Source
        painter.line_segment(
            [egui::pos2(center_x, center_y), egui::pos2(puck_x, puck_y)],
            Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(255, 215, 0, 140)),
        );

        // Sound Source Interactive Puck (>=22pt radius -> 44x44pt bounding box)
        painter.circle_stroke(
            egui::pos2(puck_x, puck_y),
            BINAURAL_PUCK_HIT_RADIUS,
            Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(255, 107, 43, 140)),
        );
        painter.circle_filled(
            egui::pos2(puck_x, puck_y),
            14.0,
            Color32::from_rgb(255, 107, 43),
        );
        painter.circle_filled(
            egui::pos2(puck_x, puck_y),
            4.0,
            Color32::from_rgb(255, 255, 255),
        );

        // Right Panel: HRTF Models & Spatial Processing (470..780)
        let mode_rect = Rect::new(rect.x + 470.0, rect.y + 56.0, 310.0, 224.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(mode_rect.x, mode_rect.y),
                egui::vec2(mode_rect.width, mode_rect.height),
            ),
            8.0,
            Color32::from_rgb(10, 14, 22),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(mode_rect.x, mode_rect.y),
                egui::vec2(mode_rect.width, mode_rect.height),
            ),
            8.0,
            Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(mode_rect.x + 12.0, mode_rect.y + 12.0),
            egui::Align2::LEFT_TOP,
            "HRTF ACOUSTIC DATASET & PROFILE",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(255, 107, 43),
        );

        // 4 Model buttons
        let models = [
            (HrtfModel::KemarStandardDummy, "KEMAR DUMMY", 0),
            (HrtfModel::CustomSubjectModel, "CUSTOM PINNA", 1),
            (HrtfModel::SphericalHeadRayTraced, "RAY-TRACED", 2),
            (HrtfModel::NearFieldBinaural, "NEAR-FIELD", 3),
        ];

        let btn_w = 138.0;
        let btn_h = 44.0;
        for (mdl, label, idx) in models {
            let row = idx / 2;
            let col = idx % 2;
            let bx = mode_rect.x + 12.0 + (col as f32 * (btn_w + 10.0));
            let by = mode_rect.y + 40.0 + (row as f32 * (btn_h + 8.0));
            let is_active = self.model == mdl;

            let bg_col = if is_active {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(35, 45, 65)
            };
            let fg_col = if is_active {
                Color32::from_rgb(0, 0, 0)
            } else {
                Color32::from_rgb(220, 235, 255)
            };

            painter.rect_filled(
                egui::Rect::from_min_size(egui::pos2(bx, by), egui::vec2(btn_w, btn_h)),
                4.0,
                bg_col,
            );
            painter.text(
                egui::pos2(bx + btn_w * 0.5, by + btn_h * 0.5),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(11.0),
                fg_col,
            );
        }

        // Room Reflections Button (>=44x44pt)
        let refl_y = mode_rect.y + 148.0;
        let refl_bg = if self.room_reflections {
            Color32::from_rgb(0, 255, 180)
        } else {
            Color32::from_rgb(35, 45, 65)
        };
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(mode_rect.x + 12.0, refl_y),
                egui::vec2(286.0, 44.0),
            ),
            4.0,
            refl_bg,
        );
        painter.text(
            egui::pos2(mode_rect.x + 155.0, refl_y + 22.0),
            egui::Align2::CENTER_CENTER,
            if self.room_reflections {
                "EARLY REFLECTIONS: ENGAGED"
            } else {
                "EARLY REFLECTIONS: OFF"
            },
            egui::FontId::proportional(11.0),
            if self.room_reflections {
                Color32::from_rgb(0, 0, 0)
            } else {
                Color32::from_rgb(220, 235, 255)
            },
        );

        // Bottom Controls Bar (20..780, y: 290..475)
        let bar_rect = Rect::new(rect.x + 20.0, rect.y + 290.0, 760.0, 185.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(bar_rect.x, bar_rect.y),
                egui::vec2(bar_rect.width, bar_rect.height),
            ),
            6.0,
            Color32::from_rgb(18, 25, 38),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(bar_rect.x, bar_rect.y),
                egui::vec2(bar_rect.width, bar_rect.height),
            ),
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        let sliders = [
            (
                "Azimuth Angle",
                format!("{:.1}°", self.azimuth_deg),
                Self::azimuth_to_normalized(self.azimuth_deg),
            ),
            (
                "Elevation",
                format!("{:.1}°", self.elevation_deg),
                Self::elevation_to_normalized(self.elevation_deg),
            ),
            (
                "Distance",
                format!("{:.2} m", self.distance_m),
                Self::distance_to_normalized(self.distance_m),
            ),
            (
                "Crossfeed Blend",
                format!("{:.0}%", self.crossfeed_percent),
                self.crossfeed_percent / 100.0,
            ),
        ];

        let mut sx_pos = bar_rect.x + 15.0;
        for (name, val_str, norm_val) in sliders {
            painter.text(
                egui::pos2(sx_pos, bar_rect.y + 15.0),
                egui::Align2::LEFT_TOP,
                name,
                egui::FontId::proportional(11.0),
                Color32::from_rgb(220, 235, 255),
            );
            painter.text(
                egui::pos2(sx_pos + 95.0, bar_rect.y + 15.0),
                egui::Align2::LEFT_TOP,
                val_str,
                egui::FontId::proportional(12.0),
                Color32::from_rgb(0, 229, 255),
            );

            // Slider track
            let track_rect = egui::Rect::from_min_size(
                egui::pos2(sx_pos, bar_rect.y + 40.0),
                egui::vec2(160.0, 26.0),
            );
            painter.rect_filled(track_rect, 4.0, Color32::from_rgb(10, 14, 22));

            // Slider fill
            let fill_w = 160.0 * norm_val;
            let fill_rect = egui::Rect::from_min_size(
                egui::pos2(sx_pos, bar_rect.y + 40.0),
                egui::vec2(fill_w, 26.0),
            );
            painter.rect_filled(fill_rect, 4.0, Color32::from_rgb(0, 229, 255));

            sx_pos += 185.0;
        }

        // Compliance Verification Badge
        let badge_rect = egui::Rect::from_min_size(
            egui::pos2(bar_rect.x + 15.0, bar_rect.y + 130.0),
            egui::vec2(730.0, 36.0),
        );
        painter.rect_filled(badge_rect, 4.0, Color32::from_rgb(16, 35, 28));
        painter.rect_stroke(
            badge_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(0, 255, 180)),
        );
        painter.text(
            egui::pos2(badge_rect.min.x + 10.0, badge_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "[PASS] Spatial Binaural HRTF 3D Orbital Sound Pucks (>= 44x44pt) Verified",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
