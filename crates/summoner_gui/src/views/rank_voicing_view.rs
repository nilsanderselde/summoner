// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Pipe Organ Rank Voicing & Acoustic Radiation HUD Canvas (Milestone 24).
//!
//! Provides an interactive 2D rank voicing canvas (mouth cutup ratio, languid height,
//! toe hole regulation, flue slit jet velocity $v = \sqrt{2P/\rho}$), real-time pipe
//! overtone harmonic structure bar display, flue air velocity vector stream visualizer,
//! and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and headless PNG snapshot rendering.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const RANK_VOICING_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding target
pub const MIN_CUTUP: f32 = 0.15;
pub const MAX_CUTUP: f32 = 0.50;
pub const MIN_TOE_HOLE: f32 = 0.20;
pub const MAX_TOE_HOLE: f32 = 1.00;

/// Voicing pipe rank selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum VoicingRankType {
    #[default]
    Principal8,
    Bourdon16,
    Octave4,
    Flute4,
    SuperOctave2,
    MixtureIV,
    Trompette8,
    VoxHumana8,
}

impl VoicingRankType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Principal8 => "Principal 8' (Diapason)",
            Self::Bourdon16 => "Bourdon 16' (Stopped Wood)",
            Self::Octave4 => "Octave 4' (Open Metal)",
            Self::Flute4 => "Flute 4' (Stopped Flute)",
            Self::SuperOctave2 => "Super Octave 2' (Shimmer)",
            Self::MixtureIV => "Mixture IV (Chorus 4-Ranks)",
            Self::Trompette8 => "Trompette 8' (Conical Reed)",
            Self::VoxHumana8 => "Vox Humana 8' (Formant Reed)",
        }
    }
}

/// Physical modeling pipe organ rank voicing and acoustic radiation view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankVoicingView {
    /// Active rank being voiced.
    pub rank_type: VoicingRankType,
    /// Cutup mouth height-to-width ratio [0.15 ..= 0.50].
    pub cutup_ratio: f32,
    /// Toe hole regulation aperture [0.20 ..= 1.00].
    pub toe_hole_aperture: f32,
    /// Languid vertical displacement in mm [-0.5 ..= 1.5 mm].
    pub languid_height_mm: f32,
    /// Windchest pressure in mmH2O [40.0 ..= 160.0].
    pub wind_pressure_mmh2o: f32,
    /// Calculated flue slit air velocity in m/s.
    pub air_velocity_mps: f32,
    /// Chiff attack transient duration in ms [5.0 ..= 80.0].
    pub chiff_duration_ms: f32,
    /// 8-harmonic overtone spectrum distribution.
    pub harmonic_weights: [f32; 8],
    /// 2D Voicing puck position (X: cutup_ratio, Y: toe_hole_aperture).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for RankVoicingView {
    fn default() -> Self {
        Self::new()
    }
}

impl RankVoicingView {
    pub fn new() -> Self {
        let mut view = Self {
            rank_type: VoicingRankType::Principal8,
            cutup_ratio: 0.25,
            toe_hole_aperture: 0.85,
            languid_height_mm: 0.20,
            wind_pressure_mmh2o: 80.0,
            air_velocity_mps: 36.1,
            chiff_duration_ms: 28.0,
            harmonic_weights: [1.0, 0.65, 0.45, 0.30, 0.18, 0.12, 0.08, 0.04],
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        };
        view.update_puck_from_physics();
        view.update_voicing_acoustics();
        view
    }

    pub fn update_puck_from_physics(&mut self) {
        let norm_x = ((self.cutup_ratio - MIN_CUTUP) / (MAX_CUTUP - MIN_CUTUP)).clamp(0.0, 1.0);
        let norm_y = ((self.toe_hole_aperture - MIN_TOE_HOLE) / (MAX_TOE_HOLE - MIN_TOE_HOLE)).clamp(0.0, 1.0);
        self.puck_pos = (norm_x, norm_y);
    }

    pub fn update_physics_from_puck(&mut self, norm_x: f32, norm_y: f32) {
        self.puck_pos = (norm_x.clamp(0.0, 1.0), norm_y.clamp(0.0, 1.0));
        self.cutup_ratio = MIN_CUTUP + self.puck_pos.0 * (MAX_CUTUP - MIN_CUTUP);
        self.toe_hole_aperture = MIN_TOE_HOLE + self.puck_pos.1 * (MAX_TOE_HOLE - MIN_TOE_HOLE);
        self.update_voicing_acoustics();
    }

    /// Calculate jet velocity, chiff transient, and overtone distribution based on voicing parameters.
    pub fn update_voicing_acoustics(&mut self) {
        let effective_p_pascals = self.wind_pressure_mmh2o * 9.80665 * self.toe_hole_aperture;
        let air_density = 1.204; // kg/m^3 at 20°C
        self.air_velocity_mps = (2.0 * effective_p_pascals / air_density).sqrt().clamp(10.0, 70.0);

        // Chiff duration: increases with larger cutup, decreases with higher pressure/velocity
        self.chiff_duration_ms = (40.0 * (self.cutup_ratio / 0.25) * (36.0 / self.air_velocity_mps.max(10.0)))
            .clamp(5.0, 80.0);

        // Harmonic overtones depending on rank type and cutup/toe regulation
        match self.rank_type {
            VoicingRankType::Principal8 => {
                for i in 0..8 {
                    let harm_idx = (i + 1) as f32;
                    let roll = 1.0 / harm_idx.powf(1.1);
                    let cutup_boost = if i == 0 { 1.0 } else { 1.0 - (self.cutup_ratio - 0.25) * 1.5 };
                    self.harmonic_weights[i] = (roll * cutup_boost * self.toe_hole_aperture).clamp(0.01, 1.0);
                }
            }
            VoicingRankType::Bourdon16 => {
                // Stopped pipe: odd harmonics dominate (1, 3, 5, 7)
                for i in 0..8 {
                    let harm_idx = i + 1;
                    if harm_idx % 2 == 1 {
                        self.harmonic_weights[i] = (1.0 / (harm_idx as f32).powf(1.3) * self.toe_hole_aperture).clamp(0.01, 1.0);
                    } else {
                        self.harmonic_weights[i] = (0.05 / (harm_idx as f32)).clamp(0.01, 0.2);
                    }
                }
            }
            VoicingRankType::Octave4 => {
                for i in 0..8 {
                    let harm_idx = (i + 1) as f32;
                    self.harmonic_weights[i] = (0.9 / harm_idx.powf(0.95) * self.toe_hole_aperture).clamp(0.01, 1.0);
                }
            }
            VoicingRankType::Flute4 => {
                for i in 0..8 {
                    let harm_idx = i + 1;
                    if harm_idx % 2 == 1 {
                        self.harmonic_weights[i] = (0.95 / (harm_idx as f32).powf(1.6)).clamp(0.01, 1.0);
                    } else {
                        self.harmonic_weights[i] = (0.04 / harm_idx as f32).clamp(0.01, 0.15);
                    }
                }
            }
            VoicingRankType::SuperOctave2 => {
                for i in 0..8 {
                    let harm_idx = (i + 1) as f32;
                    self.harmonic_weights[i] = (0.8 / harm_idx.powf(0.8) * self.toe_hole_aperture).clamp(0.01, 1.0);
                }
            }
            VoicingRankType::MixtureIV => {
                self.harmonic_weights = [
                    0.65 * self.toe_hole_aperture,
                    0.50 * self.toe_hole_aperture,
                    0.90 * self.toe_hole_aperture,
                    0.40 * self.toe_hole_aperture,
                    0.85 * self.toe_hole_aperture,
                    0.30 * self.toe_hole_aperture,
                    0.60 * self.toe_hole_aperture,
                    0.25 * self.toe_hole_aperture,
                ];
            }
            VoicingRankType::Trompette8 => {
                for i in 0..8 {
                    let harm_idx = (i + 1) as f32;
                    self.harmonic_weights[i] = (1.0 / harm_idx.powf(0.65) * self.toe_hole_aperture).clamp(0.01, 1.0);
                }
            }
            VoicingRankType::VoxHumana8 => {
                self.harmonic_weights = [
                    0.40 * self.toe_hole_aperture,
                    0.60 * self.toe_hole_aperture,
                    1.00 * self.toe_hole_aperture,
                    0.90 * self.toe_hole_aperture,
                    0.70 * self.toe_hole_aperture,
                    0.30 * self.toe_hole_aperture,
                    0.20 * self.toe_hole_aperture,
                    0.10 * self.toe_hole_aperture,
                ];
            }
        }
    }

    /// Evaluates 16 vector stream points representing flue slit acoustic airflow turbulence.
    pub fn evaluate_air_velocity_stream(&self) -> [(f32, f32); 16] {
        let mut stream = [(0.0f32, 0.0f32); 16];
        let v_scale = self.air_velocity_mps / 40.0;
        for (i, slot) in stream.iter_mut().enumerate() {
            let t = i as f32 / 15.0;
            let x = t;
            let y = (t * std::f32::consts::TAU * 1.5).sin() * 0.15 * (1.0 + self.cutup_ratio) * v_scale;
            *slot = (x, y);
        }
        stream
    }

    /// Hit test coordinate on the interactive rank voicing puck.
    pub fn hit_test_puck(&self, screen_pos: (f32, f32), canvas_rect: Rect) -> bool {
        let puck_screen_x = canvas_rect.x + self.puck_pos.0 * canvas_rect.width;
        let puck_screen_y = canvas_rect.y + (1.0 - self.puck_pos.1) * canvas_rect.height;
        let dx = screen_pos.0 - puck_screen_x;
        let dy = screen_pos.1 - puck_screen_y;
        (dx * dx + dy * dy).sqrt() <= RANK_VOICING_PUCK_HIT_RADIUS
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

        // Left half: Voicing Puck (Cutup vs Toe Hole)
        let left_w = mid_x - 2;
        let p_row = (((1.0 - self.puck_pos.1) * (height - 5) as f32) + 2.0).round() as usize;
        let p_col = ((self.puck_pos.0 * (left_w - 4) as f32) + 2.0).round() as usize;
        if p_row < height - 1 && p_col < mid_x {
            grid[p_row][p_col] = 'V';
        }

        // Right half: Harmonic overtone bars
        let right_w = width - mid_x - 2;
        let bar_spacing = right_w / 9;
        for (i, weight) in self.harmonic_weights.iter().enumerate() {
            let col = mid_x + 2 + (i + 1) * bar_spacing;
            let bar_h = (weight * (height - 4) as f32).round() as usize;
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
            "PIPE ORGAN RANK VOICING & ACOUSTIC RADIATION CANVAS",
            egui::FontId::proportional(14.5),
            Color32::from_rgb(240, 245, 255),
        );

        // Rank Selector Tabs (y: 48..92) - Each tab >= 44pt touch target
        let ranks = [
            (VoicingRankType::Principal8, "PRINCIPAL 8'"),
            (VoicingRankType::Bourdon16, "BOURDON 16'"),
            (VoicingRankType::Octave4, "OCTAVE 4'"),
            (VoicingRankType::Flute4, "FLUTE 4'"),
            (VoicingRankType::SuperOctave2, "SUPER OCT 2'"),
            (VoicingRankType::MixtureIV, "MIXTURE IV"),
            (VoicingRankType::Trompette8, "TROMPETTE 8'"),
            (VoicingRankType::VoxHumana8, "VOX HUMANA 8'"),
        ];

        let tab_w = (rect.width() - 40.0 - 7.0 * 6.0) / 8.0;
        for (i, (rtype, name)) in ranks.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 6.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_sel = self.rank_type == *rtype;
            let bg_col = if is_sel {
                Color32::from_rgb(255, 180, 50)
            } else {
                Color32::from_rgb(24, 32, 48)
            };
            let text_col = if is_sel {
                Color32::from_rgb(12, 14, 18)
            } else {
                Color32::from_rgb(210, 225, 245)
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
                        self.rank_type = *rtype;
                        self.update_voicing_acoustics();
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

        // Left 55%: Voicing 2D Puck Canvas (Cutup Ratio vs Toe Hole Aperture)
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
            "MOUTH CUTUP & TOE HOLE REGULATION PUCK",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(255, 190, 80),
        );

        // Grid lines
        for g in 1..4 {
            let gx = left_rect.min.x + left_rect.width() * (g as f32 * 0.25);
            let gy = left_rect.min.y + left_rect.height() * (g as f32 * 0.25);
            painter.line_segment(
                [
                    egui::pos2(gx, left_rect.min.y + 25.0),
                    egui::pos2(gx, left_rect.max.y - 25.0),
                ],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 85, 120, 60)),
            );
            painter.line_segment(
                [
                    egui::pos2(left_rect.min.x + 10.0, gy),
                    egui::pos2(left_rect.max.x - 10.0, gy),
                ],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 85, 120, 60)),
            );
        }

        // Air velocity stream visualizer
        let stream = self.evaluate_air_velocity_stream();
        for i in 0..15 {
            let p1 = stream[i];
            let p2 = stream[i + 1];
            let sx1 = left_rect.min.x + 20.0 + p1.0 * (left_rect.width() - 40.0);
            let sy1 = left_rect.center().y + p1.1 * 40.0;
            let sx2 = left_rect.min.x + 20.0 + p2.0 * (left_rect.width() - 40.0);
            let sy2 = left_rect.center().y + p2.1 * 40.0;
            painter.line_segment(
                [egui::pos2(sx1, sy1), egui::pos2(sx2, sy2)],
                Stroke::new(2.0_f32, Color32::from_rgb(0, 229, 255)),
            );
        }

        // Interactive Voicing Puck
        let puck_x = left_rect.min.x + self.puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.update_physics_from_puck(nx, ny);
                }
            }
        }

        painter.circle_filled(puck_pos, RANK_VOICING_PUCK_HIT_RADIUS, Color32::from_rgb(255, 180, 50));
        painter.circle_stroke(
            puck_pos,
            RANK_VOICING_PUCK_HIT_RADIUS,
            Stroke::new(2.0_f32, Color32::from_rgb(255, 240, 200)),
        );

        // Right 45%: Harmonic Overtone Radiation Bar Display
        let right_x = main_canvas.min.x + left_w + 10.0;
        let right_w = main_canvas.max.x - right_x - 10.0;
        let right_rect = egui::Rect::from_min_size(
            egui::pos2(right_x, main_canvas.min.y + 10.0),
            egui::vec2(right_w, main_canvas.height() - 20.0),
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
            "ACOUSTIC RADIATION OVERTONE SPECTRUM (1..8)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(0, 229, 255),
        );

        let bar_spacing = right_rect.width() / 9.0;
        let bar_width = bar_spacing * 0.70;
        for (i, &weight) in self.harmonic_weights.iter().enumerate() {
            let bx = right_rect.min.x + 10.0 + i as f32 * bar_spacing;
            let bar_h = weight * (right_rect.height() - 60.0);
            let bar_rect = egui::Rect::from_min_size(
                egui::pos2(bx, right_rect.max.y - 30.0 - bar_h),
                egui::vec2(bar_width, bar_h),
            );
            let col = if i == 0 {
                Color32::from_rgb(255, 180, 50)
            } else if (i + 1) % 2 == 1 {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(0, 255, 180)
            };
            painter.rect_filled(bar_rect, 2.0, col);

            painter.text(
                egui::pos2(bx + bar_width * 0.5, right_rect.max.y - 20.0),
                egui::Align2::CENTER_TOP,
                format!("H{}", i + 1),
                egui::FontId::proportional(9.0),
                Color32::from_rgb(180, 200, 230),
            );
        }

        // Bottom Telemetry Dashboard (y: 350..460)
        let dash_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 350.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 460.0),
        );
        painter.rect_filled(dash_rect, 4.0, Color32::from_rgb(16, 22, 36));

        let tele_y = dash_rect.min.y + 16.0;
        let col_w = dash_rect.width() / 4.0;

        let tele_items = [
            ("MOUTH CUTUP", format!("{:.2}", self.cutup_ratio)),
            ("TOE REGULATION", format!("{:.0}%", self.toe_hole_aperture * 100.0)),
            ("FLUE AIR VELOCITY", format!("{:.1} m/s", self.air_velocity_mps)),
            ("CHIFF DURATION", format!("{:.1} ms", self.chiff_duration_ms)),
        ];

        for (i, (label, val)) in tele_items.iter().enumerate() {
            let tx = dash_rect.min.x + 15.0 + i as f32 * col_w;
            painter.text(
                egui::pos2(tx, tele_y),
                egui::Align2::LEFT_TOP,
                *label,
                egui::FontId::proportional(10.0),
                Color32::from_rgb(160, 185, 220),
            );
            painter.text(
                egui::pos2(tx, tele_y + 20.0),
                egui::Align2::LEFT_TOP,
                val,
                egui::FontId::proportional(16.0),
                Color32::from_rgb(255, 180, 50),
            );
        }
    }

    /// Render headless PNG snapshot.
    pub fn render_snapshot_png(&self, path: &str, width: usize, height: usize) -> Result<(), String> {
        let mut pixels = vec![0u8; width * height * 4];

        // Background: Deep Slate Navy (#0C101A)
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) * 4;
                pixels[idx] = 0x0C;
                pixels[idx + 1] = 0x10;
                pixels[idx + 2] = 0x1A;
                pixels[idx + 3] = 0xFF;
            }
        }

        let margin = 20;
        let c_w = (width - 2 * margin) * 55 / 100;
        let c_h = height * 50 / 100;
        let c_x = margin;
        let c_y = 100;

        // Left canvas: Voicing box (#0E121E)
        for y in c_y..(c_y + c_h) {
            for x in c_x..(c_x + c_w) {
                if x < width && y < height {
                    let idx = (y * width + x) * 4;
                    pixels[idx] = 0x0E;
                    pixels[idx + 1] = 0x12;
                    pixels[idx + 2] = 0x1E;
                    pixels[idx + 3] = 0xFF;
                }
            }
        }

        // Draw Air Velocity Stream curve
        let stream = self.evaluate_air_velocity_stream();
        for point in stream.iter() {
            let sx = (c_x as f32 + 20.0 + point.0 * (c_w as f32 - 40.0)) as usize;
            let sy = (c_y as f32 + c_h as f32 * 0.5 + point.1 * 40.0) as usize;
            for dy in 0..3 {
                for dx in 0..3 {
                    let px = (sx + dx).saturating_sub(1);
                    let py = (sy + dy).saturating_sub(1);
                    if px < width && py < height {
                        let idx = (py * width + px) * 4;
                        // Cyan #00E5FF
                        pixels[idx] = 0x00;
                        pixels[idx + 1] = 0xE5;
                        pixels[idx + 2] = 0xFF;
                        pixels[idx + 3] = 0xFF;
                    }
                }
            }
        }

        // Draw Interactive Voicing Puck Handle (Radius = 22pt)
        let puck_cx = (c_x as f32 + self.puck_pos.0 * c_w as f32) as usize;
        let puck_cy = (c_y as f32 + (1.0 - self.puck_pos.1) * c_h as f32) as usize;
        let r = RANK_VOICING_PUCK_HIT_RADIUS as usize;

        for dy in 0..=(2 * r) {
            for dx in 0..=(2 * r) {
                let px = (puck_cx + dx).saturating_sub(r);
                let py = (puck_cy + dy).saturating_sub(r);
                let dist_sq = (dx as isize - r as isize).pow(2) + (dy as isize - r as isize).pow(2);
                if dist_sq <= (r * r) as isize && px < width && py < height {
                    let idx = (py * width + px) * 4;
                    // Golden Tangerine #FFB432
                    pixels[idx] = 0xFF;
                    pixels[idx + 1] = 0xB4;
                    pixels[idx + 2] = 0x32;
                    pixels[idx + 3] = 0xFF;
                }
            }
        }

        // Right canvas: Harmonic overtone bars
        let r_x = c_x + c_w + 10;
        let r_w = width - r_x - margin;
        for y in c_y..(c_y + c_h) {
            for x in r_x..(r_x + r_w) {
                if x < width && y < height {
                    let idx = (y * width + x) * 4;
                    pixels[idx] = 0x0E;
                    pixels[idx + 1] = 0x12;
                    pixels[idx + 2] = 0x1E;
                    pixels[idx + 3] = 0xFF;
                }
            }
        }

        let bar_spacing = r_w / 9;
        let bar_width = bar_spacing * 3 / 4;
        for (i, &weight) in self.harmonic_weights.iter().enumerate() {
            let bx = r_x + 10 + i * bar_spacing;
            let bar_h = (weight * (c_h as f32 * 0.75)) as usize;
            let by_start = (c_y + c_h).saturating_sub(bar_h + 10);
            let by_end = c_y + c_h - 10;

            for y in by_start..by_end {
                for x in bx..(bx + bar_width) {
                    if x < width && y < height {
                        let idx = (y * width + x) * 4;
                        if i == 0 {
                            // Gold #FFB432
                            pixels[idx] = 0xFF;
                            pixels[idx + 1] = 0xB4;
                            pixels[idx + 2] = 0x32;
                        } else if (i + 1) % 2 == 1 {
                            // Cyan #00E5FF
                            pixels[idx] = 0x00;
                            pixels[idx + 1] = 0xE5;
                            pixels[idx + 2] = 0xFF;
                        } else {
                            // Mint #00FFB4
                            pixels[idx] = 0x00;
                            pixels[idx + 1] = 0xFF;
                            pixels[idx + 2] = 0xB4;
                        }
                        pixels[idx + 3] = 0xFF;
                    }
                }
            }
        }

        save_png_file(path, width, height, &pixels)
    }
}

fn save_png_file(path: &str, width: usize, height: usize, rgba_pixels: &[u8]) -> Result<(), String> {
    let mut out = Vec::with_capacity(width * height * 4 + 1024);
    out.extend_from_slice(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]); // PNG Header

    // IHDR Chunk
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&(width as u32).to_be_bytes());
    ihdr.extend_from_slice(&(height as u32).to_be_bytes());
    ihdr.push(8); // Bit depth
    ihdr.push(6); // Color type: RGBA
    ihdr.push(0); // Compression method
    ihdr.push(0); // Filter method
    ihdr.push(0); // Interlace method
    write_png_chunk_to(&mut out, b"IHDR", &ihdr);

    // IDAT Chunk (Raw deflate uncompressed blocks)
    let stride = width * 4;
    let mut raw_data = Vec::with_capacity((stride + 1) * height);
    for y in 0..height {
        raw_data.push(0x00); // Filter type None
        let row_start = y * stride;
        let row_end = row_start + stride;
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

    write_png_chunk_to(&mut out, b"IDAT", &zlib_data);
    write_png_chunk_to(&mut out, b"IEND", &[]);

    if let Some(parent) = std::path::Path::new(path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(path, out).map_err(|e| e.to_string())
}

fn write_png_chunk_to(out: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let type_start = out.len();
    out.extend_from_slice(chunk_type);
    out.extend_from_slice(data);
    let crc = crc32_compute_to(&out[type_start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

fn crc32_compute_to(buf: &[u8]) -> u32 {
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
    use crate::touch_controls::MIN_HIT_TARGET_PT;

    #[test]
    fn test_rank_voicing_view_hit_target_dimensions() {
        const {
            assert!(
                RANK_VOICING_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT,
                "Rank Voicing puck hit target bounding box must be >= 44pt"
            );
        }
    }

    #[test]
    fn test_rank_voicing_view_ascii_render() {
        let view = RankVoicingView::new();
        let ascii = view.render_ascii(80, 16);
        assert_eq!(ascii.len(), 16);
        assert!(ascii[0].starts_with('+'));
    }

    #[test]
    fn test_rank_voicing_view_snapshot_render() {
        let view = RankVoicingView::new();
        let res = view.render_snapshot_png("scratch/renders/rank_voicing_view.png", 800, 520);
        assert!(res.is_ok());
    }
}
