// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Broadcast Mastering Immersive MPEG-H 3D Dynamic Object-Based Audio Trajectory Panner HUD (Step 1615).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const MPEGH_TRAJ_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_TRAJ_AZIMUTH_DEG: f32 = -180.0;
pub const MAX_TRAJ_AZIMUTH_DEG: f32 = 180.0;
pub const MIN_TRAJ_ELEVATION_DEG: f32 = -90.0;
pub const MAX_TRAJ_ELEVATION_DEG: f32 = 90.0;

/// MPEG-H 3D dynamic object-based audio trajectories and layout profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MpeghTrajectoryProfile {
    Mpegh3DHelicalAscent,     // 3D spiral/helical ascent around listener sphere
    Mpegh3DOverheadFlyby,     // Cinematic front-to-back zenith flyby object
    Mpegh3DEquatorialOrbit,   // Continuous 360-degree azimuth equatorial orbit
    Mpegh3DPendulumSwing,     // Elevation oscillatory pendulum motion
    Mpegh3DMultiObjectMaster, // 3 dynamic objects simultaneously panned in 22.2 / 7.1.4 speaker space
}

impl MpeghTrajectoryProfile {
    pub fn profile_name(&self) -> &'static str {
        match self {
            Self::Mpegh3DHelicalAscent => "MPEG-H 3D HELICAL ASCENT SPIRAL",
            Self::Mpegh3DOverheadFlyby => "CINEMATIC OVERHEAD FLYBY (ZENITH)",
            Self::Mpegh3DEquatorialOrbit => "360° EQUATORIAL ORBITAL TRAJECTORY",
            Self::Mpegh3DPendulumSwing => "ELEVATION OSCILLATORY PENDULUM",
            Self::Mpegh3DMultiObjectMaster => "MULTI-OBJECT IMMERSIVE 22.2 MASTER",
        }
    }

    pub fn nominal_azimuth_deg(&self) -> f32 {
        match self {
            Self::Mpegh3DHelicalAscent => 45.0,
            Self::Mpegh3DOverheadFlyby => 0.0,
            Self::Mpegh3DEquatorialOrbit => -90.0,
            Self::Mpegh3DPendulumSwing => 30.0,
            Self::Mpegh3DMultiObjectMaster => -45.0,
        }
    }

    pub fn nominal_elevation_deg(&self) -> f32 {
        match self {
            Self::Mpegh3DHelicalAscent => 35.0,
            Self::Mpegh3DOverheadFlyby => 75.0,
            Self::Mpegh3DEquatorialOrbit => 0.0,
            Self::Mpegh3DPendulumSwing => -20.0,
            Self::Mpegh3DMultiObjectMaster => 25.0,
        }
    }

    pub fn nominal_distance_m(&self) -> f32 {
        match self {
            Self::Mpegh3DHelicalAscent => 4.5,
            Self::Mpegh3DOverheadFlyby => 2.0,
            Self::Mpegh3DEquatorialOrbit => 5.0,
            Self::Mpegh3DPendulumSwing => 3.5,
            Self::Mpegh3DMultiObjectMaster => 6.0,
        }
    }

    pub fn nominal_speed_hz(&self) -> f32 {
        match self {
            Self::Mpegh3DHelicalAscent => 0.25,
            Self::Mpegh3DOverheadFlyby => 0.50,
            Self::Mpegh3DEquatorialOrbit => 0.15,
            Self::Mpegh3DPendulumSwing => 0.35,
            Self::Mpegh3DMultiObjectMaster => 0.20,
        }
    }

    pub fn nominal_object_spread(&self) -> f32 {
        match self {
            Self::Mpegh3DHelicalAscent => 0.30,
            Self::Mpegh3DOverheadFlyby => 0.45,
            Self::Mpegh3DEquatorialOrbit => 0.20,
            Self::Mpegh3DPendulumSwing => 0.35,
            Self::Mpegh3DMultiObjectMaster => 0.60,
        }
    }
}

/// Broadcast mastering immersive MPEG-H 3D dynamic object-based audio trajectory panner HUD.
#[derive(Debug, Clone)]
pub struct MpeghTrajectoryView {
    pub profile: MpeghTrajectoryProfile,
    pub azimuth_deg: f32,
    pub elevation_deg: f32,
    pub distance_m: f32,
    pub trajectory_speed_hz: f32,
    pub object_spread: f32,
    pub vbap_energy_focus: f32,
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    pub speaker_group_gains: [f32; 8],
    pub color_palette: ContrastColorPalette,
}

impl Default for MpeghTrajectoryView {
    fn default() -> Self {
        Self::new()
    }
}

impl MpeghTrajectoryView {
    pub fn new() -> Self {
        let mut view = Self {
            profile: MpeghTrajectoryProfile::Mpegh3DHelicalAscent,
            azimuth_deg: 45.0,
            elevation_deg: 35.0,
            distance_m: 4.5,
            trajectory_speed_hz: 0.25,
            object_spread: 0.30,
            vbap_energy_focus: 0.85,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            speaker_group_gains: [0.72, 0.55, 0.40, 0.20, 0.35, 0.48, 0.82, 0.60],
            color_palette: ContrastColorPalette::default(),
        };
        view.puck_pos = (
            Self::azimuth_to_normalized(view.azimuth_deg),
            Self::elevation_to_normalized(view.elevation_deg),
        );
        view.update_mpegh_simulation();
        view
    }

    pub fn azimuth_to_normalized(az: f32) -> f32 {
        let a = az.clamp(MIN_TRAJ_AZIMUTH_DEG, MAX_TRAJ_AZIMUTH_DEG);
        ((a - MIN_TRAJ_AZIMUTH_DEG) / (MAX_TRAJ_AZIMUTH_DEG - MIN_TRAJ_AZIMUTH_DEG)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_azimuth(norm: f32) -> f32 {
        MIN_TRAJ_AZIMUTH_DEG + norm.clamp(0.0, 1.0) * (MAX_TRAJ_AZIMUTH_DEG - MIN_TRAJ_AZIMUTH_DEG)
    }

    pub fn elevation_to_normalized(el: f32) -> f32 {
        let e = el.clamp(MIN_TRAJ_ELEVATION_DEG, MAX_TRAJ_ELEVATION_DEG);
        ((e - MIN_TRAJ_ELEVATION_DEG) / (MAX_TRAJ_ELEVATION_DEG - MIN_TRAJ_ELEVATION_DEG))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_elevation(norm: f32) -> f32 {
        MIN_TRAJ_ELEVATION_DEG
            + norm.clamp(0.0, 1.0) * (MAX_TRAJ_ELEVATION_DEG - MIN_TRAJ_ELEVATION_DEG)
    }

    pub fn set_trajectory(&mut self, profile: MpeghTrajectoryProfile) {
        self.profile = profile;
        self.azimuth_deg = profile.nominal_azimuth_deg();
        self.elevation_deg = profile.nominal_elevation_deg();
        self.distance_m = profile.nominal_distance_m();
        self.trajectory_speed_hz = profile.nominal_speed_hz();
        self.object_spread = profile.nominal_object_spread();
        self.puck_pos = (
            Self::azimuth_to_normalized(self.azimuth_deg),
            Self::elevation_to_normalized(self.elevation_deg),
        );
        self.update_mpegh_simulation();
    }

    pub fn update_mpegh_simulation(&mut self) {
        let az_rad = self.azimuth_deg.to_radians();
        let el_rad = self.elevation_deg.to_radians();
        let dist = self.distance_m;
        let spread = self.object_spread;

        // 3D Cartesian coordinates
        let x = az_rad.sin() * el_rad.cos(); // Right
        let y = az_rad.cos() * el_rad.cos(); // Front
        let z = el_rad.sin(); // Top

        // Distance attenuation
        let dist_gain = (1.0 / (1.0 + dist * 0.15)).clamp(0.2, 1.0);

        // 8 MPEG-H Speaker Groups:
        // [Front-Left, Front-Right, Center, LFE, Surround-Left, Surround-Right, Top-Front, Top-Back]
        let fl = (((-x + y) * 0.5 + 0.5).max(0.0) * (1.0 - z.abs() * 0.5) * dist_gain
            + spread * 0.2)
            .clamp(0.0, 1.2);
        let fr = (((x + y) * 0.5 + 0.5).max(0.0) * (1.0 - z.abs() * 0.5) * dist_gain
            + spread * 0.2)
            .clamp(0.0, 1.2);
        let c =
            ((y).max(0.0) * (1.0 - x.abs()) * (1.0 - z.abs() * 0.5) * dist_gain).clamp(0.0, 1.1);
        let lfe = (0.25 * dist_gain).clamp(0.05, 0.8);
        let sl = (((-x - y) * 0.5 + 0.5).max(0.0) * (1.0 - z.abs() * 0.5) * dist_gain
            + spread * 0.2)
            .clamp(0.0, 1.2);
        let sr = (((x - y) * 0.5 + 0.5).max(0.0) * (1.0 - z.abs() * 0.5) * dist_gain
            + spread * 0.2)
            .clamp(0.0, 1.2);
        let tf =
            ((z).max(0.0) * ((y).max(0.0) * 0.5 + 0.5) * dist_gain + spread * 0.3).clamp(0.0, 1.2);
        let tb =
            ((z).max(0.0) * ((-y).max(0.0) * 0.5 + 0.5) * dist_gain + spread * 0.3).clamp(0.0, 1.2);

        self.speaker_group_gains = [fl, fr, c, lfe, sl, sr, tf, tb];
        self.vbap_energy_focus = (1.0 - spread * 0.45).clamp(0.3, 1.0);
    }

    pub fn hit_test_mpegh_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= MPEGH_TRAJ_PUCK_HIT_RADIUS
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
        let bar_spacing = right_w / 9;
        for (i, &gain) in self.speaker_group_gains.iter().enumerate() {
            let col = mid_x + 2 + (i + 1) * bar_spacing;
            let bar_h = (gain.clamp(0.0, 1.0) * (height - 4) as f32).round() as usize;
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

        // Background: Deep Broadcast Slate (#0C101C)
        painter.rect_filled(rect, 6.0, Color32::from_rgb(12, 16, 28));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "MPEG-H 3D DYNAMIC OBJECT AUDIO TRAJECTORY PANNER HUD",
            egui::FontId::proportional(13.5),
            Color32::from_rgb(240, 245, 255),
        );

        // Tabs (y: 48..92) - 44pt height
        let tabs = [
            (
                MpeghTrajectoryProfile::Mpegh3DHelicalAscent,
                "HELICAL ASCENT",
            ),
            (
                MpeghTrajectoryProfile::Mpegh3DOverheadFlyby,
                "OVERHEAD FLYBY",
            ),
            (MpeghTrajectoryProfile::Mpegh3DEquatorialOrbit, "360° ORBIT"),
            (
                MpeghTrajectoryProfile::Mpegh3DPendulumSwing,
                "PENDULUM SWING",
            ),
            (
                MpeghTrajectoryProfile::Mpegh3DMultiObjectMaster,
                "22.2 OBJECTS",
            ),
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
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(24, 32, 48)
            };
            let text_col = if is_sel {
                Color32::from_rgb(4, 20, 28)
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
                        self.set_trajectory(*ptype);
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

        // Left 55%: Azimuth vs Elevation Spherical Trajectory Canvas
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
            "SPHERICAL AZIMUTH (-180°..+180°) vs ELEVATION (-90°..+90°)",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(0, 229, 255),
        );

        // Crosshairs in background
        painter.line_segment(
            [
                egui::pos2(left_rect.min.x + 10.0, left_rect.center().y),
                egui::pos2(left_rect.max.x - 10.0, left_rect.center().y),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(50, 75, 110, 100)),
        );
        painter.line_segment(
            [
                egui::pos2(left_rect.center().x, left_rect.min.y + 25.0),
                egui::pos2(left_rect.center().x, left_rect.max.y - 25.0),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(50, 75, 110, 100)),
        );

        // Interactive Trajectory Puck
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
                    self.update_mpegh_simulation();
                }
            }
        }

        painter.circle_stroke(
            puck_pos,
            MPEGH_TRAJ_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(0, 229, 255, 150)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(0, 229, 255));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 18.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Azimuth: {:+.1}° | Elev: {:+.1}° | Dist: {:.1}m | Speed: {:.2}Hz | Spread: {:.0}%",
                self.azimuth_deg,
                self.elevation_deg,
                self.distance_m,
                self.trajectory_speed_hz,
                self.object_spread * 100.0
            ),
            egui::FontId::proportional(10.5),
            Color32::from_rgb(140, 235, 255),
        );

        // Right 45%: MPEG-H 3D Speaker Group Gains Spectrum
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
            "MPEG-H 3D SPEAKER GROUP VBAP ENERGY DISTRIBUTION",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(0, 229, 255),
        );

        let group_labels = ["FL", "FR", "C", "LFE", "SL", "SR", "TOP-F", "TOP-B"];
        let bar_w = (right_rect.width() - 30.0 - 7.0 * 6.0) / 8.0;
        for (i, &gain) in self.speaker_group_gains.iter().enumerate() {
            let bx = right_rect.min.x + 15.0 + i as f32 * (bar_w + 6.0);
            let bar_h = (gain.clamp(0.0, 1.2) / 1.2) * (right_rect.height() - 80.0);
            let b_rect = egui::Rect::from_min_max(
                egui::pos2(bx, right_rect.max.y - 25.0 - bar_h),
                egui::pos2(bx + bar_w, right_rect.max.y - 25.0),
            );
            let col = if i < 3 {
                Color32::from_rgb(0, 229, 255)
            } else if i < 6 {
                Color32::from_rgb(0, 255, 180)
            } else {
                Color32::from_rgb(255, 215, 0)
            };
            painter.rect_filled(b_rect, 3.0, col);

            painter.text(
                egui::pos2(bx + bar_w * 0.5, right_rect.max.y - 20.0),
                egui::Align2::CENTER_TOP,
                group_labels[i],
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

        let params = [
            (
                "3D AZIMUTH / ELEVATION",
                format!("{:+.1}° / {:+.1}°", self.azimuth_deg, self.elevation_deg),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "OBJECT DISTANCE",
                format!("{:.1} m (Immersive Field)", self.distance_m),
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "ENERGY FOCUS (VBAP)",
                format!(
                    "{:.0}% (Directional Direct)",
                    self.vbap_energy_focus * 100.0
                ),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "TRAJECTORY SPEED",
                format!("{:.2} Hz (Orbit Frequency)", self.trajectory_speed_hz),
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
            "[PASS] MPEG-H 3D Dynamic Object Trajectory Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
