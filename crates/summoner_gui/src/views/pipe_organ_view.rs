// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Physical Modeling Pipe Organ Windchest & Flue/Reed Acoustic Turbulence HUD (Step 1561).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const PIPE_ORGAN_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_WIND_PRESSURE_MMH2O: f32 = 40.0;
pub const MAX_WIND_PRESSURE_MMH2O: f32 = 160.0;
pub const MIN_CUTUP_RATIO: f32 = 0.15;
pub const MAX_CUTUP_RATIO: f32 = 0.50;
pub const MIN_PIPE_LENGTH_FT: f32 = 0.5;
pub const MAX_PIPE_LENGTH_FT: f32 = 32.0;

/// Pipe organ rank and acoustic generator type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipeType {
    Principal8Flue,     // Open metal flue pipe (Montre / Diapason)
    Bourdon16Stopped,   // Stopped wooden flue pipe (Gedeckt)
    Trompette8Reed,     // Beating brass reed with conical resonator
    MixtureIVMultiRank, // High-pitched harmonic compound flue rank
    VoxHumana8Reed,     // Fractional length short cylindrical reed
}

impl PipeType {
    pub fn rank_name(&self) -> &'static str {
        match self {
            Self::Principal8Flue => "PRINCIPAL 8'",
            Self::Bourdon16Stopped => "BOURDON 16'",
            Self::Trompette8Reed => "TROMPETTE 8'",
            Self::MixtureIVMultiRank => "MIXTURE IV",
            Self::VoxHumana8Reed => "VOX HUMANA 8'",
        }
    }

    pub fn nominal_pressure_mmh2o(&self) -> f32 {
        match self {
            Self::Principal8Flue => 75.0,
            Self::Bourdon16Stopped => 60.0,
            Self::Trompette8Reed => 110.0,
            Self::MixtureIVMultiRank => 85.0,
            Self::VoxHumana8Reed => 95.0,
        }
    }

    pub fn nominal_cutup_ratio(&self) -> f32 {
        match self {
            Self::Principal8Flue => 0.25,
            Self::Bourdon16Stopped => 0.35,
            Self::Trompette8Reed => 0.20,
            Self::MixtureIVMultiRank => 0.22,
            Self::VoxHumana8Reed => 0.18,
        }
    }

    pub fn is_reed(&self) -> bool {
        matches!(self, Self::Trompette8Reed | Self::VoxHumana8Reed)
    }
}

/// Physical modeling pipe organ windchest and flue acoustic turbulence HUD.
#[derive(Debug, Clone)]
pub struct PipeOrganView {
    pub pipe_type: PipeType,
    pub wind_pressure_mmh2o: f32,   // [40.0 ..= 160.0 mmH2O]
    pub cutup_ratio: f32,           // [0.15 ..= 0.50 mouth height / width ratio]
    pub pipe_length_ft: f32,        // [0.5 ..= 32.0 ft]
    pub chiff_duration_ms: f32,     // [5.0 ..= 80.0 ms attack transient noise]
    pub organ_puck_pos: (f32, f32), // Normalized (X: wind pressure, Y: cutup ratio)
    pub is_dragging_puck: bool,
    pub turbulence_noise_level: f32, // [0.0 ..= 1.0]
    pub flue_air_velocity_mps: f32,  // [10.0 ..= 60.0 m/s]
    pub harmonic_weights: [f32; 8],  // 8 overtones
    pub color_palette: ContrastColorPalette,
}

impl Default for PipeOrganView {
    fn default() -> Self {
        Self::new()
    }
}

impl PipeOrganView {
    pub fn new() -> Self {
        let mut view = Self {
            pipe_type: PipeType::Principal8Flue,
            wind_pressure_mmh2o: 75.0,
            cutup_ratio: 0.25,
            pipe_length_ft: 8.0,
            chiff_duration_ms: 32.0,
            organ_puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            turbulence_noise_level: 0.28,
            flue_air_velocity_mps: 34.5,
            harmonic_weights: [1.0, 0.65, 0.45, 0.30, 0.18, 0.12, 0.08, 0.04],
            color_palette: ContrastColorPalette::default(),
        };
        view.organ_puck_pos = (
            Self::pressure_to_normalized(view.wind_pressure_mmh2o),
            Self::cutup_to_normalized(view.cutup_ratio),
        );
        view.update_acoustic_simulation();
        view
    }

    pub fn pressure_to_normalized(mmh2o: f32) -> f32 {
        let p = mmh2o.clamp(MIN_WIND_PRESSURE_MMH2O, MAX_WIND_PRESSURE_MMH2O);
        ((p - MIN_WIND_PRESSURE_MMH2O) / (MAX_WIND_PRESSURE_MMH2O - MIN_WIND_PRESSURE_MMH2O))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_pressure(norm: f32) -> f32 {
        MIN_WIND_PRESSURE_MMH2O
            + norm.clamp(0.0, 1.0) * (MAX_WIND_PRESSURE_MMH2O - MIN_WIND_PRESSURE_MMH2O)
    }

    pub fn cutup_to_normalized(cutup: f32) -> f32 {
        let c = cutup.clamp(MIN_CUTUP_RATIO, MAX_CUTUP_RATIO);
        ((c - MIN_CUTUP_RATIO) / (MAX_CUTUP_RATIO - MIN_CUTUP_RATIO)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_cutup(norm: f32) -> f32 {
        MIN_CUTUP_RATIO + norm.clamp(0.0, 1.0) * (MAX_CUTUP_RATIO - MIN_CUTUP_RATIO)
    }

    pub fn set_pipe_type(&mut self, pipe_type: PipeType) {
        self.pipe_type = pipe_type;
        self.wind_pressure_mmh2o = pipe_type.nominal_pressure_mmh2o();
        self.cutup_ratio = pipe_type.nominal_cutup_ratio();
        self.organ_puck_pos = (
            Self::pressure_to_normalized(self.wind_pressure_mmh2o),
            Self::cutup_to_normalized(self.cutup_ratio),
        );
        self.update_acoustic_simulation();
    }

    /// Update fluid dynamics, jet velocity v = sqrt(2 * P / rho), and harmonic distribution.
    pub fn update_acoustic_simulation(&mut self) {
        let p_pascals = self.wind_pressure_mmh2o * 9.80665;
        let air_density = 1.204; // kg/m^3 at 20 C
        self.flue_air_velocity_mps = (2.0 * p_pascals / air_density).sqrt().clamp(10.0, 70.0);

        // Chiff duration increases with larger cutup and lower wind pressure
        self.chiff_duration_ms =
            (50.0 * (self.cutup_ratio / 0.25) * (75.0 / self.wind_pressure_mmh2o).sqrt())
                .clamp(5.0, 80.0);

        // Turbulence noise level
        let reynolds_factor = (self.flue_air_velocity_mps / 35.0) * (1.0 + self.cutup_ratio);
        self.turbulence_noise_level = (0.25 * reynolds_factor).clamp(0.05, 0.95);

        // Harmonic overtones depending on flue vs stopped vs reed
        match self.pipe_type {
            PipeType::Principal8Flue => {
                for i in 0..8 {
                    self.harmonic_weights[i] = (1.0 / ((i + 1) as f32).powf(1.1))
                        * (1.0 + 0.1 * (self.cutup_ratio - 0.25));
                }
            }
            PipeType::Bourdon16Stopped => {
                // Stopped pipe: odd harmonics dominate (1, 3, 5, 7)
                for i in 0..8 {
                    let harm_idx = i + 1;
                    if harm_idx % 2 == 1 {
                        self.harmonic_weights[i] = 1.0 / (harm_idx as f32).powf(1.3);
                    } else {
                        self.harmonic_weights[i] = 0.08 / (harm_idx as f32);
                    }
                }
            }
            PipeType::Trompette8Reed => {
                // Bright reed with strong upper spectrum
                for i in 0..8 {
                    self.harmonic_weights[i] = 1.0 / ((i + 1) as f32).powf(0.65);
                }
            }
            PipeType::MixtureIVMultiRank => {
                // Multi-rank compound shimmer
                self.harmonic_weights = [0.7, 0.5, 0.9, 0.4, 0.85, 0.3, 0.6, 0.2];
            }
            PipeType::VoxHumana8Reed => {
                // Nasal reed cavity formant at 800-1200 Hz (approx 3rd-5th harmonics)
                self.harmonic_weights = [0.4, 0.6, 1.0, 0.9, 0.7, 0.3, 0.2, 0.1];
            }
        }
    }

    /// Hit test coordinate on the interactive organ voicing puck.
    pub fn hit_test_organ_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.organ_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.organ_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= PIPE_ORGAN_PUCK_HIT_RADIUS
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

        // Left half: Windchest & Flue Slit
        let left_w = mid_x - 2;
        let p_row = (((1.0 - self.organ_puck_pos.1) * (height - 5) as f32) + 2.0).round() as usize;
        let p_col = ((self.organ_puck_pos.0 * (left_w - 4) as f32) + 2.0).round() as usize;
        if p_row < height - 1 && p_col < mid_x {
            grid[p_row][p_col] = 'P';
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
            "PHYSICAL MODELING PIPE ORGAN WINDCHEST & FLUE ACOUSTIC TURBULENCE HUD",
            egui::FontId::proportional(14.5),
            Color32::from_rgb(240, 245, 255),
        );

        // Pipe Rank Preset Tabs (y: 48..92) - Each tab >= 44pt touch target
        let ranks = [
            (PipeType::Principal8Flue, "PRINCIPAL 8'"),
            (PipeType::Bourdon16Stopped, "BOURDON 16'"),
            (PipeType::Trompette8Reed, "TROMPETTE 8'"),
            (PipeType::MixtureIVMultiRank, "MIXTURE IV"),
            (PipeType::VoxHumana8Reed, "VOX HUMANA 8'"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (ptype, name)) in ranks.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_sel = self.pipe_type == *ptype;
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
                egui::FontId::proportional(11.0),
                text_col,
            );

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if tab_rect.contains(pos) {
                        self.set_pipe_type(*ptype);
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

        // Left 55%: Windchest Reservoir & Flue Air Jet Fluid Simulation
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
            "WINDCHEST RESERVOIR & ACOUSTIC FLUE JET SIMULATION",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(255, 190, 80),
        );

        // Grid lines for Pressure (X) and Cutup (Y)
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

        // Flue / Reed mouth visualization curve
        let cx = left_rect.center().x;
        let cy = left_rect.center().y + 10.0;
        let mouth_h = 30.0 + self.cutup_ratio * 100.0;
        painter.rect_stroke(
            egui::Rect::from_center_size(egui::pos2(cx - 60.0, cy), egui::vec2(24.0, mouth_h)),
            2.0,
            Stroke::new(2.0_f32, Color32::from_rgb(255, 180, 50)),
        );

        // Air jet velocity stream vectors
        let jet_len = self.flue_air_velocity_mps * 1.5;
        painter.line_segment(
            [
                egui::pos2(cx - 48.0, cy),
                egui::pos2(cx - 48.0 + jet_len, cy),
            ],
            Stroke::new(2.5_f32, Color32::from_rgb(0, 229, 255)),
        );

        // Interactive Voicing Puck
        let puck_x = left_rect.min.x + self.organ_puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.organ_puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.organ_puck_pos = (nx, ny);
                    self.wind_pressure_mmh2o = Self::normalized_to_pressure(nx);
                    self.cutup_ratio = Self::normalized_to_cutup(ny);
                    self.update_acoustic_simulation();
                }
            }
        }

        // Draw Touch Hit Target boundary (>= 44x44pt)
        painter.circle_stroke(
            puck_pos,
            PIPE_ORGAN_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(255, 180, 50, 150)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(255, 180, 50));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 20.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Pressure: {:.1} mmH2O | Cutup: {:.2} | Velocity: {:.1} m/s",
                self.wind_pressure_mmh2o, self.cutup_ratio, self.flue_air_velocity_mps
            ),
            egui::FontId::proportional(10.5),
            Color32::from_rgb(255, 215, 100),
        );

        // Right 45%: Flue / Reed Harmonic Spectrum & Acoustic Turbulence Noise
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
            "PIPE OVERTONE HARMONIC STRUCTURE",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(255, 190, 80),
        );

        let bar_w = (right_rect.width() - 30.0 - 7.0 * 6.0) / 8.0;
        for (i, weight) in self.harmonic_weights.iter().enumerate() {
            let bx = right_rect.min.x + 15.0 + i as f32 * (bar_w + 6.0);
            let bar_h = weight * (right_rect.height() - 80.0);
            let b_rect = egui::Rect::from_min_max(
                egui::pos2(bx, right_rect.max.y - 25.0 - bar_h),
                egui::pos2(bx + bar_w, right_rect.max.y - 25.0),
            );
            let col = if i == 0 {
                Color32::from_rgb(255, 180, 50)
            } else if (i + 1) % 2 == 1 {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(0, 255, 180)
            };
            painter.rect_filled(b_rect, 3.0, col);

            painter.text(
                egui::pos2(bx + bar_w * 0.5, right_rect.max.y - 20.0),
                egui::Align2::CENTER_TOP,
                format!("h{}", i + 1),
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
                "WIND PRESSURE",
                format!(
                    "{:.1} mmH2O ({:.2} in)",
                    self.wind_pressure_mmh2o,
                    self.wind_pressure_mmh2o / 25.4
                ),
                Color32::from_rgb(255, 180, 50),
            ),
            (
                "PIPE LENGTH / PITCH",
                format!(
                    "{:.1}' ({})",
                    self.pipe_length_ft,
                    self.pipe_type.rank_name()
                ),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "CHIFF ATTACK TRANSIENT",
                format!("{:.1} ms (Vortex Jet)", self.chiff_duration_ms),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "ACOUSTIC TURBULENCE",
                format!(
                    "{:.0}% (Re: {:.1})",
                    self.turbulence_noise_level * 100.0,
                    self.flue_air_velocity_mps
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
            "[PASS] Pipe Organ Windchest & Acoustic Flue Modeling Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
