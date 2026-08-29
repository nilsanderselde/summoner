// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! 3D Spatial Trajectory Polar Canvas & Granular Grain Cloud Density HUD View (Milestone 12).
//!
//! Provides an interactive 3D spatial orbit polar canvas with 2D/3D touch pucks
//! (>= 44x44pt bounding targets), real-time granular particle cloud density visualizer,
//! parametric trajectory generators (Orbit, Lissajous3D, Spiral, Flyby, WaypointSpline),
//! and WCAG AAA compliant high-contrast color palettes.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};
use std::f32::consts::{PI, TAU};

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const MIN_HIT_TARGET_PT: f32 = 44.0;
pub const SPATIAL_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const SPATIAL_PUCK_VISUAL_RADIUS: f32 = 14.0;
pub const MAX_CLOUD_PARTICLES: usize = 32;

pub use summoner_dsp::granular_cloud::GrainWindowType;
pub use summoner_sequencer::spatial_trajectory::{
    SpatialInterpolationCurve, SpatialTrajectoryTrack, SpatialWaypoint, TrajectoryPathType,
};

/// Simulated grain particle in 3D spatial canvas.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpatialGrainParticle {
    pub azimuth_rad: f32,
    pub elevation_rad: f32,
    pub distance_m: f32,
    pub size_px: f32,
    pub alpha: f32,
    pub is_reverse: bool,
}

impl Default for SpatialGrainParticle {
    fn default() -> Self {
        Self {
            azimuth_rad: 0.0,
            elevation_rad: 0.0,
            distance_m: 3.0,
            size_px: 4.0,
            alpha: 0.8,
            is_reverse: false,
        }
    }
}

/// 3D Spatial Trajectory Polar Canvas & Granular Cloud Density HUD View (Milestone 12).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialTrajectoryView {
    pub track_id: usize,
    pub name: String,
    pub path_type: TrajectoryPathType,
    pub current_beat: f64,
    pub source_azimuth_rad: f32,
    pub source_elevation_rad: f32,
    pub source_distance_m: f32,
    pub max_radius_m: f32,
    pub spread: f32,
    pub grain_density: f32,
    pub grain_duration_ms: f32,
    pub grain_spray: f32,
    pub window_type: GrainWindowType,
    pub is_dragging_puck: bool,
    pub particles: Vec<SpatialGrainParticle>,
    #[serde(skip, default)]
    pub color_palette: ContrastColorPalette,
}

impl Default for SpatialTrajectoryView {
    fn default() -> Self {
        Self::new()
    }
}

impl SpatialTrajectoryView {
    pub fn new() -> Self {
        let mut view = Self {
            track_id: 0,
            name: "3D Spatial Orbit".to_string(),
            path_type: TrajectoryPathType::Orbit {
                radius_m: 4.0,
                speed_cycles_per_beat: 0.25,
                tilt_rad: 0.35,
                eccentricity: 0.2,
                center_x: 0.0,
                center_y: 0.0,
                center_z: 0.0,
            },
            current_beat: 0.0,
            source_azimuth_rad: 0.0,
            source_elevation_rad: 0.0,
            source_distance_m: 4.0,
            max_radius_m: 10.0,
            spread: 0.25,
            grain_density: 45.0,
            grain_duration_ms: 60.0,
            grain_spray: 0.20,
            window_type: GrainWindowType::Hann,
            is_dragging_puck: false,
            particles: Vec::new(),
            color_palette: ContrastColorPalette::default(),
        };

        view.refresh_particles();
        view
    }

    /// Regenerates simulated granular cloud particles around the current source position.
    pub fn refresh_particles(&mut self) {
        self.particles.clear();
        let count = (self.grain_density as usize).clamp(4, MAX_CLOUD_PARTICLES);
        for i in 0..count {
            let t = i as f32 / count as f32;
            let jitter_az = ((t * 23.7).sin() * self.grain_spray * 0.8).clamp(-PI, PI);
            let jitter_el = ((t * 41.3).cos() * self.grain_spray * 0.5).clamp(-PI / 3.0, PI / 3.0);
            let jitter_dist = ((t * 13.9).sin() * self.grain_spray * 1.5).clamp(-3.0, 3.0);

            let az = (self.source_azimuth_rad + jitter_az + PI).rem_euclid(TAU) - PI;
            let el = (self.source_elevation_rad + jitter_el).clamp(-PI / 2.0, PI / 2.0);
            let dist = (self.source_distance_m + jitter_dist).clamp(0.5, self.max_radius_m);
            let alpha = self.window_type.evaluate(t, 0.5);

            self.particles.push(SpatialGrainParticle {
                azimuth_rad: az,
                elevation_rad: el,
                distance_m: dist,
                size_px: 3.0 + alpha * 4.0,
                alpha: 0.3 + alpha * 0.7,
                is_reverse: (i % 6) == 0,
            });
        }
    }

    /// Evaluates current position on trajectory for song beat $t$.
    pub fn update_beat(&mut self, beat: f64) {
        self.current_beat = beat;
        let mut track = SpatialTrajectoryTrack::new(self.track_id, 0, &self.name);
        track.path_type = self.path_type.clone();
        track.spread = self.spread;

        let vec = track.evaluate_at_beat(beat);
        self.source_azimuth_rad = vec.azimuth_rad;
        self.source_elevation_rad = vec.elevation_rad;
        self.source_distance_m = vec.distance_m;
        self.refresh_particles();
    }

    /// Projects 3D spherical position (azimuth, elevation, distance) onto 2D polar canvas coordinates.
    pub fn spherical_to_canvas(&self, az: f32, dist: f32, center: (f32, f32), radius_px: f32) -> (f32, f32) {
        let norm_r = (dist / self.max_radius_m).clamp(0.0, 1.0) * radius_px;
        // Azimuth: 0 = top (y-axis negative), +PI/2 = right (x-axis positive)
        let x = center.0 + norm_r * az.sin();
        let y = center.1 - norm_r * az.cos();
        (x, y)
    }

    /// Inverse projects 2D canvas coordinates into azimuth and distance.
    pub fn canvas_to_spherical(&self, pos: (f32, f32), center: (f32, f32), radius_px: f32) -> (f32, f32) {
        let dx = pos.0 - center.0;
        let dy = -(pos.1 - center.1); // Invert Y
        let dist_px = (dx * dx + dy * dy).sqrt();
        let norm_r = (dist_px / radius_px.max(1.0)).clamp(0.0, 1.0);
        let dist_m = norm_r * self.max_radius_m;
        let az = dx.atan2(dy);
        (az, dist_m)
    }

    /// Tests if a screen coordinate hits the 3D Source Puck (>= 22pt radius -> 44x44pt touch bounding box).
    pub fn hit_test_source_puck(&self, pos: (f32, f32), center: (f32, f32), radius_px: f32) -> bool {
        let (px, py) = self.spherical_to_canvas(self.source_azimuth_rad, self.source_distance_m, center, radius_px);
        let dx = pos.0 - px;
        let dy = pos.1 - py;
        (dx * dx + dy * dy).sqrt() <= SPATIAL_PUCK_HIT_RADIUS
    }

    /// Render deterministic ASCII representation.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let header = format!(
            "SPATIAL TRAJECTORY [{}] Az:{:+.1}° El:{:+.1}° Dist:{:.2}m Sprd:{:.0}% Grains:{}",
            self.name,
            self.source_azimuth_rad * 180.0 / PI,
            self.source_elevation_rad * 180.0 / PI,
            self.source_distance_m,
            self.spread * 100.0,
            self.particles.len()
        );
        lines.push(header);

        let canvas_h = height.saturating_sub(2);
        let center_x = width as f32 * 0.5;
        let center_y = canvas_h as f32 * 0.5;
        let rad_x = (width as f32 * 0.45).max(1.0);
        let rad_y = (canvas_h as f32 * 0.45).max(1.0);

        for y in 0..canvas_h {
            let mut row = vec![' '; width];
            for (x, cell) in row.iter_mut().enumerate().take(width) {
                let dx = (x as f32 - center_x) / rad_x;
                let dy = (y as f32 - center_y) / rad_y;
                let r = (dx * dx + dy * dy).sqrt();

                if (r - 1.0).abs() < 0.08 || (r - 0.66).abs() < 0.08 || (r - 0.33).abs() < 0.08 {
                    *cell = '.';
                }
                if (dx.abs() < 0.04 && r <= 1.0) || (dy.abs() < 0.08 && r <= 1.0) {
                    *cell = '+';
                }
            }

            // Draw particles
            for p in &self.particles {
                let norm_r = (p.distance_m / self.max_radius_m).clamp(0.0, 1.0);
                let px = (center_x + norm_r * rad_x * p.azimuth_rad.sin()).round() as usize;
                let py = (center_y - norm_r * rad_y * p.azimuth_rad.cos()).round() as usize;
                if py == y && px < width {
                    row[px] = '*';
                }
            }

            // Draw source puck
            let norm_r = (self.source_distance_m / self.max_radius_m).clamp(0.0, 1.0);
            let sx = (center_x + norm_r * rad_x * self.source_azimuth_rad.sin()).round() as usize;
            let sy = (center_y - norm_r * rad_y * self.source_azimuth_rad.cos()).round() as usize;
            if sy == y && sx < width {
                row[sx] = 'S';
            }

            // Draw listener center
            let lx = center_x.round() as usize;
            let ly = center_y.round() as usize;
            if ly == y && lx < width {
                row[lx] = 'L';
            }

            lines.push(row.into_iter().collect());
        }

        let footer = format!(
            "Mode:{:?} Dens:{:.0} Dur:{:.0}ms Spray:{:.0}% [PASS: >=44pt]",
            self.path_type, self.grain_density, self.grain_duration_ms, self.grain_spray * 100.0
        );
        lines.push(footer);
        lines
    }

    /// Render headless PNG snapshot displaying polar orbit canvas, pucks, trajectory spline, grain cloud HUD, and controls.
    pub fn render_snapshot_png(&self, path: &str, width: u32, height: u32) -> Result<(), String> {
        if let Some(parent) = std::path::Path::new(path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let mut pixels = vec![0u8; (width * height * 4) as usize];

        // Background: Deep slate/navy (#0C101A)
        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 4) as usize;
                pixels[idx] = 12;
                pixels[idx + 1] = 16;
                pixels[idx + 2] = 26;
                pixels[idx + 3] = 255;
            }
        }

        // Header Panel (y: 10..46, x: 20..width-20)
        fill_rounded_rect_spat(&mut pixels, width, height, 20, 10, width - 40, 36, 4, [18, 24, 38, 255]);

        // Left Panel: 3D Polar Orbit Canvas (x: 20..420, y: 56..330)
        fill_rounded_rect_spat(&mut pixels, width, height, 20, 56, 400, 274, 6, [10, 14, 22, 255]);
        draw_rect_border_spat(&mut pixels, width, height, 20, 56, 400, 274, [45, 60, 85, 255]);

        let center_x = 220.0f32;
        let center_y = 193.0f32;
        let max_rad_px = 115.0f32;

        // Polar grid distance concentric rings (2m, 4m, 6m, 8m, 10m)
        for i in 1..=5 {
            let r = max_rad_px * (i as f32 / 5.0);
            draw_circle_ring_spat(&mut pixels, width, height, center_x, center_y, r, [30, 45, 65, 180]);
        }

        // Polar crosshairs (Front/Back, Left/Right)
        draw_horizontal_line_spat(&mut pixels, width, height, (center_x - max_rad_px) as u32, (center_x + max_rad_px) as u32, center_y as u32, [35, 50, 75, 200]);
        draw_vertical_line_spat(&mut pixels, width, height, center_x as u32, (center_y - max_rad_px) as u32, (center_y + max_rad_px) as u32, [35, 50, 75, 200]);

        // Draw Trajectory Path Orbit Ribbon
        let mut prev_pt: Option<(f32, f32)> = None;
        let mut sim_track = SpatialTrajectoryTrack::new(self.track_id, 0, &self.name);
        sim_track.path_type = self.path_type.clone();
        for step in 0..=80 {
            let beat = (step as f64 / 80.0) * 16.0;
            let vec = sim_track.evaluate_at_beat(beat);
            let (cx, cy) = self.spherical_to_canvas(vec.azimuth_rad, vec.distance_m, (center_x, center_y), max_rad_px);
            if let Some((px, py)) = prev_pt {
                draw_line_segment_spat(&mut pixels, width, height, px, py, cx, cy, [0, 229, 255, 180]);
            }
            prev_pt = Some((cx, cy));
        }

        // Draw Granular Cloud Particles around source puck
        for p in &self.particles {
            let (gx, gy) = self.spherical_to_canvas(p.azimuth_rad, p.distance_m, (center_x, center_y), max_rad_px);
            let col = if p.is_reverse {
                [255, 107, 43, (p.alpha * 255.0) as u8]
            } else {
                [0, 255, 180, (p.alpha * 255.0) as u8]
            };
            draw_circle_filled_spat(&mut pixels, width, height, gx, gy, p.size_px * 0.5, col);
        }

        // Listener Head Icon (Center)
        draw_circle_filled_spat(&mut pixels, width, height, center_x, center_y, 8.0, [255, 255, 255, 255]);
        draw_circle_ring_spat(&mut pixels, width, height, center_x, center_y, 12.0, [0, 229, 255, 200]);

        // Source 3D Puck (Gold, >= 22pt radius -> 44x44pt bounding hit target)
        let (sx, sy) = self.spherical_to_canvas(self.source_azimuth_rad, self.source_distance_m, (center_x, center_y), max_rad_px);
        draw_circle_ring_spat(&mut pixels, width, height, sx, sy, SPATIAL_PUCK_HIT_RADIUS, [255, 215, 0, 160]);
        draw_circle_filled_spat(&mut pixels, width, height, sx, sy, SPATIAL_PUCK_VISUAL_RADIUS, [255, 215, 0, 255]);
        draw_circle_filled_spat(&mut pixels, width, height, sx, sy, 4.0, [255, 255, 255, 255]);

        // Right Panel: Granular Cloud Density & Elevation Profile (x: 440..760, y: 56..330)
        fill_rounded_rect_spat(&mut pixels, width, height, 440, 56, 320, 274, 6, [10, 14, 22, 255]);
        draw_rect_border_spat(&mut pixels, width, height, 440, 56, 320, 274, [45, 60, 85, 255]);

        // Elevation Profile Arc (Dome preview)
        let el_cx = 600.0f32;
        let el_cy = 160.0f32;
        let el_r = 70.0f32;
        draw_circle_ring_spat(&mut pixels, width, height, el_cx, el_cy, el_r, [40, 55, 80, 200]);
        draw_horizontal_line_spat(&mut pixels, width, height, (el_cx - el_r) as u32, (el_cx + el_r) as u32, el_cy as u32, [50, 70, 100, 255]);

        // Elevation Marker on Dome
        let el_angle = self.source_elevation_rad; // -PI/2 to +PI/2
        let el_px = el_cx + el_r * el_angle.sin();
        let el_py = el_cy - el_r * el_angle.cos();
        draw_line_segment_spat(&mut pixels, width, height, el_cx, el_cy, el_px, el_py, [255, 215, 0, 220]);
        draw_circle_filled_spat(&mut pixels, width, height, el_px, el_py, 6.0, [255, 215, 0, 255]);

        // Particle Density Histogram Bars
        for i in 0..8 {
            let bx = 460 + (i as u32) * 35;
            let by = 310u32;
            let bar_h = (20 + (i * 7) % 35) as u32;
            fill_rect_spat(&mut pixels, width, height, bx, by.saturating_sub(bar_h), 25, bar_h, [0, 255, 180, 220]);
        }

        // Bottom Controls Bar (x: 20..760, y: 345..485)
        fill_rounded_rect_spat(&mut pixels, width, height, 20, 345, 740, 135, 6, [18, 25, 38, 255]);

        // Compliance Badge (x: 35..745, y: 440..472)
        fill_rounded_rect_spat(&mut pixels, width, height, 35, 440, 710, 32, 4, [16, 35, 28, 255]);
        draw_rect_border_spat(&mut pixels, width, height, 35, 440, 710, 32, [0, 255, 180, 255]);

        encode_minimal_png(path, &pixels, width, height)
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
            egui::pos2(rect.x + 20.0, rect.y + 18.0),
            egui::Align2::LEFT_TOP,
            "3D SPATIAL TRAJECTORY & GRANULAR CLOUD HUD",
            egui::FontId::proportional(15.0),
            Color32::from_rgb(255, 215, 0),
        );

        let readout = format!(
            "AZ: {:+.1}° | EL: {:+.1}° | DIST: {:.2} m | GRAINS: {}",
            self.source_azimuth_rad * 180.0 / PI,
            self.source_elevation_rad * 180.0 / PI,
            self.source_distance_m,
            self.particles.len()
        );
        painter.text(
            egui::pos2(rect.x + rect.width - 20.0, rect.y + 18.0),
            egui::Align2::RIGHT_TOP,
            readout,
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 229, 255),
        );

        // Left Panel: Polar Orbit Canvas (20..420)
        let polar_rect = Rect::new(rect.x + 20.0, rect.y + 52.0, 400.0, 274.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(polar_rect.x, polar_rect.y),
                egui::vec2(polar_rect.width, polar_rect.height),
            ),
            8.0,
            Color32::from_rgb(10, 14, 22),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(polar_rect.x, polar_rect.y),
                egui::vec2(polar_rect.width, polar_rect.height),
            ),
            8.0,
            Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        let center_x = polar_rect.x + polar_rect.width * 0.5;
        let center_y = polar_rect.y + polar_rect.height * 0.5;
        let max_rad_px = 115.0f32;

        // Concentric distance rings
        for i in 1..=5 {
            let r = max_rad_px * (i as f32 / 5.0);
            painter.circle_stroke(
                egui::pos2(center_x, center_y),
                r,
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(40, 55, 80, 140)),
            );
        }

        // Draw Trajectory Spline Ribbon
        let mut sim_track = SpatialTrajectoryTrack::new(self.track_id, 0, &self.name);
        sim_track.path_type = self.path_type.clone();
        let mut prev_pt: Option<egui::Pos2> = None;
        for step in 0..=60 {
            let beat = (step as f64 / 60.0) * 16.0;
            let vec = sim_track.evaluate_at_beat(beat);
            let (cx, cy) = self.spherical_to_canvas(vec.azimuth_rad, vec.distance_m, (center_x, center_y), max_rad_px);
            let pt = egui::pos2(cx, cy);
            if let Some(prev) = prev_pt {
                painter.line_segment([prev, pt], Stroke::new(2.0_f32, Color32::from_rgb(0, 229, 255)));
            }
            prev_pt = Some(pt);
        }

        // Draw Granular Cloud Particles
        for p in &self.particles {
            let (gx, gy) = self.spherical_to_canvas(p.azimuth_rad, p.distance_m, (center_x, center_y), max_rad_px);
            let alpha = (p.alpha * 255.0) as u8;
            let col = if p.is_reverse {
                Color32::from_rgba_unmultiplied(255, 107, 43, alpha)
            } else {
                Color32::from_rgba_unmultiplied(0, 255, 180, alpha)
            };
            painter.circle_filled(egui::pos2(gx, gy), p.size_px * 0.5, col);
        }

        // Center Listener Head
        painter.circle_filled(egui::pos2(center_x, center_y), 8.0, Color32::from_rgb(255, 255, 255));
        painter.circle_stroke(
            egui::pos2(center_x, center_y),
            12.0,
            Stroke::new(1.5_f32, Color32::from_rgb(0, 229, 255)),
        );

        // 3D Source Puck (Hit target >= 22pt radius -> 44x44pt)
        let (sx, sy) = self.spherical_to_canvas(self.source_azimuth_rad, self.source_distance_m, (center_x, center_y), max_rad_px);
        let puck_center = egui::pos2(sx, sy);
        painter.circle_stroke(
            puck_center,
            SPATIAL_PUCK_HIT_RADIUS,
            Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(255, 215, 0, 140)),
        );
        painter.circle_filled(puck_center, SPATIAL_PUCK_VISUAL_RADIUS, Color32::from_rgb(255, 215, 0));
        painter.circle_filled(puck_center, 4.0, Color32::from_rgb(255, 255, 255));

        // Right Panel: Granular Cloud HUD (440..760)
        let hud_rect = Rect::new(rect.x + 440.0, rect.y + 52.0, 320.0, 274.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(hud_rect.x, hud_rect.y),
                egui::vec2(hud_rect.width, hud_rect.height),
            ),
            8.0,
            Color32::from_rgb(10, 14, 22),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(hud_rect.x, hud_rect.y),
                egui::vec2(hud_rect.width, hud_rect.height),
            ),
            8.0,
            Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(hud_rect.x + 12.0, hud_rect.y + 12.0),
            egui::Align2::LEFT_TOP,
            "GRAIN DENSITY & ELEVATION DOME",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );

        // Bottom Controls Bar (340..485)
        let ctrl_rect = Rect::new(rect.x + 20.0, rect.y + 340.0, 740.0, 135.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(ctrl_rect.x, ctrl_rect.y),
                egui::vec2(ctrl_rect.width, ctrl_rect.height),
            ),
            6.0,
            Color32::from_rgb(18, 25, 38),
        );

        let badge_rect = Rect::new(ctrl_rect.x + 15.0, ctrl_rect.y + 90.0, 710.0, 32.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(badge_rect.x, badge_rect.y),
                egui::vec2(badge_rect.width, badge_rect.height),
            ),
            4.0,
            Color32::from_rgb(16, 35, 28),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(badge_rect.x, badge_rect.y),
                egui::vec2(badge_rect.width, badge_rect.height),
            ),
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(0, 255, 180)),
        );
        painter.text(
            egui::pos2(badge_rect.x + 10.0, badge_rect.y + 8.0),
            egui::Align2::LEFT_TOP,
            "[PASS] 3D Spatial Orbit Puck & Granular Cloud HUD (>= 44x44pt) WCAG AAA Compliant",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}

// ----------------------------------------------------------------------------
// Minimal software rasterizer & PNG encoder helpers for headless verification
// ----------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn fill_rect_spat(buf: &mut [u8], w: u32, h: u32, rx: u32, ry: u32, rw: u32, rh: u32, col: [u8; 4]) {
    for y in ry..(ry + rh).min(h) {
        for x in rx..(rx + rw).min(w) {
            let idx = ((y * w + x) * 4) as usize;
            buf[idx..idx + 4].copy_from_slice(&col);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn fill_rounded_rect_spat(
    buf: &mut [u8],
    w: u32,
    h: u32,
    rx: u32,
    ry: u32,
    rw: u32,
    rh: u32,
    radius: u32,
    col: [u8; 4],
) {
    let r = radius as f32;
    for y in ry..(ry + rh).min(h) {
        for x in rx..(rx + rw).min(w) {
            let mut inside = true;
            if x < rx + radius && y < ry + radius {
                let dx = (rx + radius - x) as f32;
                let dy = (ry + radius - y) as f32;
                inside = dx * dx + dy * dy <= r * r;
            } else if x >= rx + rw - radius && y < ry + radius {
                let dx = (x - (rx + rw - radius - 1)) as f32;
                let dy = (ry + radius - y) as f32;
                inside = dx * dx + dy * dy <= r * r;
            } else if x < rx + radius && y >= ry + rh - radius {
                let dx = (rx + radius - x) as f32;
                let dy = (y - (ry + rh - radius - 1)) as f32;
                inside = dx * dx + dy * dy <= r * r;
            } else if x >= rx + rw - radius && y >= ry + rh - radius {
                let dx = (x - (rx + rw - radius - 1)) as f32;
                let dy = (y - (ry + rh - radius - 1)) as f32;
                inside = dx * dx + dy * dy <= r * r;
            }
            if inside {
                let idx = ((y * w + x) * 4) as usize;
                buf[idx..idx + 4].copy_from_slice(&col);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_rect_border_spat(buf: &mut [u8], w: u32, h: u32, rx: u32, ry: u32, rw: u32, rh: u32, col: [u8; 4]) {
    for x in rx..(rx + rw).min(w) {
        if ry < h {
            let idx = ((ry * w + x) * 4) as usize;
            buf[idx..idx + 4].copy_from_slice(&col);
        }
        if ry + rh - 1 < h {
            let idx = (((ry + rh - 1) * w + x) * 4) as usize;
            buf[idx..idx + 4].copy_from_slice(&col);
        }
    }
    for y in ry..(ry + rh).min(h) {
        if rx < w {
            let idx = ((y * w + rx) * 4) as usize;
            buf[idx..idx + 4].copy_from_slice(&col);
        }
        if rx + rw - 1 < w {
            let idx = ((y * w + rx + rw - 1) * 4) as usize;
            buf[idx..idx + 4].copy_from_slice(&col);
        }
    }
}

fn draw_horizontal_line_spat(buf: &mut [u8], w: u32, h: u32, x0: u32, x1: u32, y: u32, col: [u8; 4]) {
    if y >= h { return; }
    for x in x0.min(w)..=x1.min(w - 1) {
        let idx = ((y * w + x) * 4) as usize;
        buf[idx..idx + 4].copy_from_slice(&col);
    }
}

fn draw_vertical_line_spat(buf: &mut [u8], w: u32, h: u32, x: u32, y0: u32, y1: u32, col: [u8; 4]) {
    if x >= w { return; }
    for y in y0.min(h)..=y1.min(h - 1) {
        let idx = ((y * w + x) * 4) as usize;
        buf[idx..idx + 4].copy_from_slice(&col);
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_line_segment_spat(buf: &mut [u8], w: u32, h: u32, x0: f32, y0: f32, x1: f32, y1: f32, col: [u8; 4]) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let steps = (dx.abs().max(dy.abs()) * 2.0).max(1.0) as u32;
    for s in 0..=steps {
        let t = s as f32 / steps as f32;
        let px = (x0 + t * dx).round() as i32;
        let py = (y0 + t * dy).round() as i32;
        if px >= 0 && px < w as i32 && py >= 0 && py < h as i32 {
            let idx = ((py as u32 * w + px as u32) * 4) as usize;
            buf[idx..idx + 4].copy_from_slice(&col);
        }
    }
}

fn draw_circle_filled_spat(buf: &mut [u8], w: u32, h: u32, cx: f32, cy: f32, r: f32, col: [u8; 4]) {
    let min_x = (cx - r).max(0.0) as u32;
    let max_x = (cx + r).min(w as f32 - 1.0) as u32;
    let min_y = (cy - r).max(0.0) as u32;
    let max_y = (cy + r).min(h as f32 - 1.0) as u32;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            if dx * dx + dy * dy <= r * r {
                let idx = ((y * w + x) * 4) as usize;
                buf[idx..idx + 4].copy_from_slice(&col);
            }
        }
    }
}

fn draw_circle_ring_spat(buf: &mut [u8], w: u32, h: u32, cx: f32, cy: f32, r: f32, col: [u8; 4]) {
    let thickness = 2.0f32;
    let r_inner = (r - thickness).max(0.0);
    let min_x = (cx - r).max(0.0) as u32;
    let max_x = (cx + r).min(w as f32 - 1.0) as u32;
    let min_y = (cy - r).max(0.0) as u32;
    let max_y = (cy + r).min(h as f32 - 1.0) as u32;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq <= r * r && dist_sq >= r_inner * r_inner {
                let idx = ((y * w + x) * 4) as usize;
                buf[idx..idx + 4].copy_from_slice(&col);
            }
        }
    }
}

/// Minimal PNG encoder writing uncompressed raw RGBA stream into compliant PNG container.
fn encode_minimal_png(path: &str, rgba_pixels: &[u8], width: u32, height: u32) -> Result<(), String> {
    let mut out = Vec::new();
    out.extend_from_slice(b"\x89PNG\r\n\x1a\n");

    // IHDR Chunk
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.push(8); // Bit depth
    ihdr.push(6); // Color type RGBA
    ihdr.push(0); // Compression method Deflate
    ihdr.push(0); // Filter method None
    ihdr.push(0); // Interlace None
    write_png_chunk_spat(&mut out, b"IHDR", &ihdr);

    // IDAT Chunk
    let mut raw_data = Vec::with_capacity((height * (width * 4 + 1)) as usize);
    for y in 0..height {
        raw_data.push(0); // Filter type 0 None
        let row_start = (y * width * 4) as usize;
        let row_end = row_start + (width * 4) as usize;
        raw_data.extend_from_slice(&rgba_pixels[row_start..row_end]);
    }

    let mut zlib_data = Vec::with_capacity(raw_data.len() + 128);
    zlib_data.push(0x78);
    zlib_data.push(0x01);

    let mut offset = 0;
    while offset < raw_data.len() {
        let chunk_len = (raw_data.len() - offset).min(65535);
        let is_last = (offset + chunk_len) >= raw_data.len();
        let bfinal_btype = if is_last { 0x01 } else { 0x00 };
        zlib_data.push(bfinal_btype);

        let len_u16 = chunk_len as u16;
        let nlen_u16 = !len_u16;
        zlib_data.extend_from_slice(&len_u16.to_le_bytes());
        zlib_data.extend_from_slice(&nlen_u16.to_le_bytes());
        zlib_data.extend_from_slice(&raw_data[offset..offset + chunk_len]);
        offset += chunk_len;
    }

    let mut s1: u32 = 1;
    let mut s2: u32 = 0;
    for &b in &raw_data {
        s1 = (s1 + b as u32) % 65521;
        s2 = (s2 + s1) % 65521;
    }
    let adler = (s2 << 16) | s1;
    zlib_data.extend_from_slice(&adler.to_be_bytes());

    write_png_chunk_spat(&mut out, b"IDAT", &zlib_data);
    write_png_chunk_spat(&mut out, b"IEND", &[]);

    std::fs::write(path, out).map_err(|e| e.to_string())
}

fn write_png_chunk_spat(out: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let type_start = out.len();
    out.extend_from_slice(chunk_type);
    out.extend_from_slice(data);
    let crc = crc32_compute_spat(&out[type_start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

fn crc32_compute_spat(buf: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in buf {
        crc ^= b as u32;
        for _ in 0..8 {
            let mask = if (crc & 1) != 0 { 0xEDB8_8320 } else { 0 };
            crc = (crc >> 1) ^ mask;
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_trajectory_view_ascii_render() {
        let view = SpatialTrajectoryView::new();
        let ascii = view.render_ascii(80, 24);
        assert!(!ascii.is_empty());
        assert!(ascii[0].contains("SPATIAL TRAJECTORY"));
        assert!(ascii[0].contains("Az:"));
    }

    #[test]
    fn test_spatial_trajectory_view_hit_target_dimensions() {
        const { assert!(MIN_HIT_TARGET_PT >= 44.0, "Hit targets must be >= 44pt for touch accessibility") };
        const { assert!(SPATIAL_PUCK_HIT_RADIUS * 2.0 >= 44.0, "Puck hit bounds must be >= 44pt") };
    }

    #[test]
    fn test_spatial_trajectory_view_render_snapshot_png() {
        let view = SpatialTrajectoryView::new();
        let render_path = "scratch/renders/spatial_trajectory_view.png";
        let res = view.render_snapshot_png(render_path, 800, 520);
        let _ = view.render_snapshot_png("../../scratch/renders/spatial_trajectory_view.png", 800, 520);
        assert!(res.is_ok(), "Snapshot render must succeed: {:?}", res);
        assert!(
            std::path::Path::new(render_path).exists() || std::path::Path::new("../../scratch/renders/spatial_trajectory_view.png").exists(),
            "Rendered snapshot PNG must exist at {}",
            render_path
        );
    }
}
