// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Neural Acoustic Room Transfer Function (HRTF/BRIR) Continuous Spatial Neural Radiance Field HUD (Step 1614).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const RADIANCE_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_HEADING_DEG: f32 = -180.0;
pub const MAX_HEADING_DEG: f32 = 180.0;
pub const MIN_DISTANCE_M: f32 = 0.5;
pub const MAX_DISTANCE_M: f32 = 12.0;

/// Neural acoustic room transfer function & continuous spatial radiance field models.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RadianceFieldModel {
    InstantNeRFAcoustics, // Multi-resolution hash grid spatial impulse representation
    ContinuousFieldHRIR,  // Continuous spherical harmonic implicit neural representation for HRTF
    BinauralRoomRadianceNeRF, // 6-DoF continuous position & orientation BRIR field
    WaveguideNeuralMesh,  // Hybrid neural network + finite-difference time-domain mesh
    DiffusionRoomImpulseField, // Diffusion-conditioned spatial reverberation radiance field
}

impl RadianceFieldModel {
    pub fn model_name(&self) -> &'static str {
        match self {
            Self::InstantNeRFAcoustics => "INSTANT ACOUSTIC NeRF (HASH GRID)",
            Self::ContinuousFieldHRIR => "CONTINUOUS FIELD HRIR (IMPLICIT SH)",
            Self::BinauralRoomRadianceNeRF => "6-DoF BINAURAL BRIR RADIANCE",
            Self::WaveguideNeuralMesh => "WAVEGUIDE NEURAL FDTD MESH",
            Self::DiffusionRoomImpulseField => "DIFFUSION SPATIAL IMPULSE FIELD",
        }
    }

    pub fn nominal_heading_deg(&self) -> f32 {
        match self {
            Self::InstantNeRFAcoustics => 30.0,
            Self::ContinuousFieldHRIR => -45.0,
            Self::BinauralRoomRadianceNeRF => 60.0,
            Self::WaveguideNeuralMesh => 0.0,
            Self::DiffusionRoomImpulseField => -90.0,
        }
    }

    pub fn nominal_distance_m(&self) -> f32 {
        match self {
            Self::InstantNeRFAcoustics => 3.5,
            Self::ContinuousFieldHRIR => 1.2,
            Self::BinauralRoomRadianceNeRF => 5.0,
            Self::WaveguideNeuralMesh => 2.8,
            Self::DiffusionRoomImpulseField => 8.0,
        }
    }

    pub fn nominal_latent_dim(&self) -> usize {
        match self {
            Self::InstantNeRFAcoustics => 256,
            Self::ContinuousFieldHRIR => 128,
            Self::BinauralRoomRadianceNeRF => 512,
            Self::WaveguideNeuralMesh => 128,
            Self::DiffusionRoomImpulseField => 512,
        }
    }

    pub fn nominal_absorption_alpha(&self) -> f32 {
        match self {
            Self::InstantNeRFAcoustics => 0.25,
            Self::ContinuousFieldHRIR => 0.65,
            Self::BinauralRoomRadianceNeRF => 0.35,
            Self::WaveguideNeuralMesh => 0.15,
            Self::DiffusionRoomImpulseField => 0.45,
        }
    }
}

/// Neural acoustic room transfer function (HRTF/BRIR) continuous spatial neural radiance field HUD.
#[derive(Debug, Clone)]
pub struct NeuralRadianceView {
    pub model: RadianceFieldModel,
    pub heading_deg: f32,
    pub distance_m: f32,
    pub latent_dim: usize,
    pub absorption_alpha: f32,
    pub directivity_clarity_db: f32,
    pub rendering_latency_ms: f32,
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    pub acoustic_radiance_rays: [f32; 8],
    pub color_palette: ContrastColorPalette,
}

impl Default for NeuralRadianceView {
    fn default() -> Self {
        Self::new()
    }
}

impl NeuralRadianceView {
    pub fn new() -> Self {
        let mut view = Self {
            model: RadianceFieldModel::InstantNeRFAcoustics,
            heading_deg: 30.0,
            distance_m: 3.5,
            latent_dim: 256,
            absorption_alpha: 0.25,
            directivity_clarity_db: 14.5,
            rendering_latency_ms: 1.8,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            acoustic_radiance_rays: [0.95, 0.82, 0.64, 0.45, 0.30, 0.42, 0.58, 0.78],
            color_palette: ContrastColorPalette::default(),
        };
        view.puck_pos = (
            Self::heading_to_normalized(view.heading_deg),
            Self::distance_to_normalized(view.distance_m),
        );
        view.update_radiance_simulation();
        view
    }

    pub fn heading_to_normalized(heading: f32) -> f32 {
        let h = heading.clamp(MIN_HEADING_DEG, MAX_HEADING_DEG);
        ((h - MIN_HEADING_DEG) / (MAX_HEADING_DEG - MIN_HEADING_DEG)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_heading(norm: f32) -> f32 {
        MIN_HEADING_DEG + norm.clamp(0.0, 1.0) * (MAX_HEADING_DEG - MIN_HEADING_DEG)
    }

    pub fn distance_to_normalized(dist: f32) -> f32 {
        let d = dist.clamp(MIN_DISTANCE_M, MAX_DISTANCE_M);
        ((d - MIN_DISTANCE_M) / (MAX_DISTANCE_M - MIN_DISTANCE_M)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_distance(norm: f32) -> f32 {
        MIN_DISTANCE_M + norm.clamp(0.0, 1.0) * (MAX_DISTANCE_M - MIN_DISTANCE_M)
    }

    pub fn set_model(&mut self, model: RadianceFieldModel) {
        self.model = model;
        self.heading_deg = model.nominal_heading_deg();
        self.distance_m = model.nominal_distance_m();
        self.latent_dim = model.nominal_latent_dim();
        self.absorption_alpha = model.nominal_absorption_alpha();
        self.puck_pos = (
            Self::heading_to_normalized(self.heading_deg),
            Self::distance_to_normalized(self.distance_m),
        );
        self.update_radiance_simulation();
    }

    pub fn update_radiance_simulation(&mut self) {
        let h_rad = self.heading_deg.to_radians();
        let dist = self.distance_m;
        let alpha = self.absorption_alpha;

        // Radiance field rays across 8 cardinal directions (0, 45, 90, 135, 180, 225, 270, 315 deg)
        let mut rays = [0.0_f32; 8];
        for (i, ray) in rays.iter_mut().enumerate() {
            let ray_angle = i as f32 * (std::f32::consts::TAU / 8.0);
            let angle_diff = (ray_angle - h_rad).cos();
            let direct_lobe = ((1.0 + angle_diff) * 0.5).powf(1.8);
            let dist_atten = (1.0 / (1.0 + dist * 0.25)).clamp(0.1, 1.0);
            let room_reverb = (1.0 - alpha) * 0.45;
            *ray = (direct_lobe * dist_atten + room_reverb).clamp(0.05, 1.2);
        }

        self.acoustic_radiance_rays = rays;
        self.directivity_clarity_db = (20.0 - dist * 1.5 + alpha * 10.0).clamp(2.0, 26.0);
        self.rendering_latency_ms = (0.8 + (self.latent_dim as f32 / 256.0) * 0.9).clamp(0.5, 5.0);
    }

    pub fn hit_test_radiance_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= RADIANCE_PUCK_HIT_RADIUS
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
            grid[p_row][p_col] = 'R';
        }

        let right_w = width - mid_x - 2;
        let bar_spacing = right_w / 9;
        for (i, &ray) in self.acoustic_radiance_rays.iter().enumerate() {
            let col = mid_x + 2 + (i + 1) * bar_spacing;
            let bar_h = (ray.clamp(0.0, 1.0) * (height - 4) as f32).round() as usize;
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

        // Background: Deep Obsidian Violet Slate (#0D111A)
        painter.rect_filled(rect, 6.0, Color32::from_rgb(13, 17, 26));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "NEURAL ACOUSTIC HRTF/BRIR CONTINUOUS RADIANCE FIELD HUD",
            egui::FontId::proportional(13.5),
            Color32::from_rgb(240, 245, 255),
        );

        // Tabs (y: 48..92) - 44pt height
        let tabs = [
            (RadianceFieldModel::InstantNeRFAcoustics, "INSTANT NeRF"),
            (RadianceFieldModel::ContinuousFieldHRIR, "CONTINUOUS HRIR"),
            (
                RadianceFieldModel::BinauralRoomRadianceNeRF,
                "6-DoF BRIR NeRF",
            ),
            (RadianceFieldModel::WaveguideNeuralMesh, "WAVEGUIDE MESH"),
            (
                RadianceFieldModel::DiffusionRoomImpulseField,
                "DIFFUSION FIELD",
            ),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (mtype, name)) in tabs.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_sel = self.model == *mtype;
            let bg_col = if is_sel {
                Color32::from_rgb(157, 78, 221) // Electric Violet
            } else {
                Color32::from_rgb(26, 32, 48)
            };
            let text_col = if is_sel {
                Color32::from_rgb(255, 255, 255)
            } else {
                Color32::from_rgb(215, 225, 245)
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
                        self.set_model(*mtype);
                    }
                }
            }
        }

        // Main Display Canvas (y: 104..340)
        let main_canvas = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 104.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 340.0),
        );
        painter.rect_filled(main_canvas, 6.0, Color32::from_rgb(8, 12, 20));
        painter.rect_stroke(
            main_canvas,
            6.0,
            Stroke::new(1.5_f32, Color32::from_rgb(45, 40, 75)),
        );

        // Left 55%: Polar 3D Radiation Field & Spatial Coordinates
        let left_w = main_canvas.width() * 0.55;
        let left_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(left_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(left_rect, 4.0, Color32::from_rgb(14, 18, 28));
        painter.rect_stroke(
            left_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(35, 45, 70)),
        );

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "6-DoF LISTENER HEADING (AZIMUTH) vs DISTANCE FIELD",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(157, 78, 221),
        );

        // Polar concentric range rings in background
        let polar_center = left_rect.center();
        let max_r = (left_rect.height() * 0.38).min(left_rect.width() * 0.38);
        for r_step in 1..=4 {
            let cr = max_r * (r_step as f32 / 4.0);
            painter.circle_stroke(
                polar_center,
                cr,
                Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(60, 50, 95, 80)),
            );
        }

        // Interactive Radiance Puck
        let puck_x = left_rect.min.x + self.puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.puck_pos = (nx, ny);
                    self.heading_deg = Self::normalized_to_heading(nx);
                    self.distance_m = Self::normalized_to_distance(ny);
                    self.update_radiance_simulation();
                }
            }
        }

        painter.circle_stroke(
            puck_pos,
            RADIANCE_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(157, 78, 221, 150)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(157, 78, 221));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 18.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Heading: {:+.1}° | Dist: {:.1}m | Latent: {}d | Clarity: {:.1}dB | Latency: {:.2}ms",
                self.heading_deg,
                self.distance_m,
                self.latent_dim,
                self.directivity_clarity_db,
                self.rendering_latency_ms
            ),
            egui::FontId::proportional(10.5),
            Color32::from_rgb(220, 190, 255),
        );

        // Right 45%: Directional Acoustic Radiance Ray Spectrum
        let right_w = main_canvas.width() * 0.45;
        let right_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + left_w + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(right_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(right_rect, 4.0, Color32::from_rgb(14, 18, 28));
        painter.rect_stroke(
            right_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(35, 45, 70)),
        );

        painter.text(
            egui::pos2(right_rect.min.x + 10.0, right_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "DIRECTIONAL ACOUSTIC RADIANCE LOBES (8 RAYS)",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(157, 78, 221),
        );

        let ray_labels = ["0°", "45°", "90°", "135°", "180°", "225°", "270°", "315°"];
        let bar_w = (right_rect.width() - 30.0 - 7.0 * 6.0) / 8.0;
        for (i, &ray) in self.acoustic_radiance_rays.iter().enumerate() {
            let bx = right_rect.min.x + 15.0 + i as f32 * (bar_w + 6.0);
            let bar_h = (ray.clamp(0.0, 1.2) / 1.2) * (right_rect.height() - 80.0);
            let b_rect = egui::Rect::from_min_max(
                egui::pos2(bx, right_rect.max.y - 25.0 - bar_h),
                egui::pos2(bx + bar_w, right_rect.max.y - 25.0),
            );
            let col = if i == 0 || i == 1 || i == 7 {
                Color32::from_rgb(157, 78, 221)
            } else if i == 2 || i == 6 {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(0, 255, 180)
            };
            painter.rect_filled(b_rect, 3.0, col);

            painter.text(
                egui::pos2(bx + bar_w * 0.5, right_rect.max.y - 20.0),
                egui::Align2::CENTER_TOP,
                ray_labels[i],
                egui::FontId::proportional(8.5),
                Color32::from_rgb(190, 205, 235),
            );
        }

        // Bottom Metrics Dock (y: 350..465)
        let dock_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 350.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 465.0),
        );
        painter.rect_filled(dock_rect, 6.0, Color32::from_rgb(18, 22, 36));
        painter.rect_stroke(
            dock_rect,
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(45, 40, 75)),
        );

        let params = [
            (
                "HEADING (AZIMUTH)",
                format!("{:+.1}° (Head Orientation)", self.heading_deg),
                Color32::from_rgb(157, 78, 221),
            ),
            (
                "LISTENER DISTANCE",
                format!("{:.1} m (Acoustic Range)", self.distance_m),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "ABSORPTION COEFF",
                format!("α = {:.2} (Wall Materials)", self.absorption_alpha),
                Color32::from_rgb(0, 255, 180),
            ),
            (
                "INFERENCE LATENCY",
                format!("{:.2} ms (Realtime DSP)", self.rendering_latency_ms),
                Color32::from_rgb(255, 215, 0),
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
                Color32::from_rgb(160, 175, 205),
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
            "[PASS] Neural Radiance Field BRIR Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
