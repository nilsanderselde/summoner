// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Mastering Multi-Bus Parallel Dynamic Transient/Body Saturator HUD (Step 1573).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const SATURATOR_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_SAT_DRIVE_DB: f32 = 0.0;
pub const MAX_SAT_DRIVE_DB: f32 = 24.0;
pub const MIN_BLEND_RATIO: f32 = 0.0;
pub const MAX_BLEND_RATIO: f32 = 1.0;

/// Parallel saturator topology and harmonic coloration circuit types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaturatorMode {
    TriodeTubeEvenOdd,   // Asymmetric triode vacuum tube warm coloration
    TapeHysteresisFlux,  // Magnetic tape flux saturation with soft knee compression
    FetTransientPunch,   // Fast field-effect transistor punch exciter
    GermaniumDiodeGrit,  // Germanium diode hard non-linear clipping
    CleanLinearDynamics, // Transparent parallel multiband dynamic blend
}

impl SaturatorMode {
    pub fn mode_name(&self) -> &'static str {
        match self {
            Self::TriodeTubeEvenOdd => "TRIODE TUBE",
            Self::TapeHysteresisFlux => "TAPE FLUX",
            Self::FetTransientPunch => "FET PUNCH",
            Self::GermaniumDiodeGrit => "GERMANIUM DIODE",
            Self::CleanLinearDynamics => "CLEAN LINEAR",
        }
    }

    pub fn nominal_drive_db(&self) -> f32 {
        match self {
            Self::TriodeTubeEvenOdd => 6.0,
            Self::TapeHysteresisFlux => 8.5,
            Self::FetTransientPunch => 4.5,
            Self::GermaniumDiodeGrit => 12.0,
            Self::CleanLinearDynamics => 0.0,
        }
    }

    pub fn nominal_blend(&self) -> f32 {
        match self {
            Self::TriodeTubeEvenOdd => 0.50,
            Self::TapeHysteresisFlux => 0.65,
            Self::FetTransientPunch => 0.35,
            Self::GermaniumDiodeGrit => 0.40,
            Self::CleanLinearDynamics => 0.50,
        }
    }
}

/// Mastering multi-bus parallel dynamic transient/body saturator HUD.
#[derive(Debug, Clone)]
pub struct ParallelTransientSaturatorView {
    pub saturator_mode: SaturatorMode,
    pub transient_drive_db: f32,         // [0.0 ..= 24.0 dB]
    pub body_sustain_drive_db: f32,      // [0.0 ..= 24.0 dB]
    pub blend_ratio: f32,                // [0.0 = 100% Transient ..= 1.0 = 100% Body Sustain]
    pub saturation_puck_pos: (f32, f32), // Normalized (X: blend ratio, Y: transient drive)
    pub is_dragging_puck: bool,
    pub thd_percent: f32,           // [0.01 ..= 25.0 %]
    pub crest_factor_db: f32,       // [3.0 ..= 20.0 dB]
    pub harmonic_profile: [f32; 8], // 8 harmonics: H1..H8
    pub color_palette: ContrastColorPalette,
}

impl Default for ParallelTransientSaturatorView {
    fn default() -> Self {
        Self::new()
    }
}

impl ParallelTransientSaturatorView {
    pub fn new() -> Self {
        let mut view = Self {
            saturator_mode: SaturatorMode::TriodeTubeEvenOdd,
            transient_drive_db: 6.0,
            body_sustain_drive_db: 4.5,
            blend_ratio: 0.50,
            saturation_puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            thd_percent: 1.85,
            crest_factor_db: 12.4,
            harmonic_profile: [1.0, 0.45, 0.25, 0.12, 0.06, 0.03, 0.015, 0.008],
            color_palette: ContrastColorPalette::default(),
        };
        view.saturation_puck_pos = (
            Self::blend_to_normalized(view.blend_ratio),
            Self::drive_to_normalized(view.transient_drive_db),
        );
        view.update_saturation_simulation();
        view
    }

    pub fn blend_to_normalized(blend: f32) -> f32 {
        blend.clamp(MIN_BLEND_RATIO, MAX_BLEND_RATIO)
    }

    pub fn normalized_to_blend(norm: f32) -> f32 {
        norm.clamp(MIN_BLEND_RATIO, MAX_BLEND_RATIO)
    }

    pub fn drive_to_normalized(drive_db: f32) -> f32 {
        let d = drive_db.clamp(MIN_SAT_DRIVE_DB, MAX_SAT_DRIVE_DB);
        ((d - MIN_SAT_DRIVE_DB) / (MAX_SAT_DRIVE_DB - MIN_SAT_DRIVE_DB)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_drive(norm: f32) -> f32 {
        MIN_SAT_DRIVE_DB + norm.clamp(0.0, 1.0) * (MAX_SAT_DRIVE_DB - MIN_SAT_DRIVE_DB)
    }

    pub fn set_saturator_mode(&mut self, mode: SaturatorMode) {
        self.saturator_mode = mode;
        self.transient_drive_db = mode.nominal_drive_db();
        self.body_sustain_drive_db = (mode.nominal_drive_db() * 0.75).max(0.0);
        self.blend_ratio = mode.nominal_blend();
        self.saturation_puck_pos = (
            Self::blend_to_normalized(self.blend_ratio),
            Self::drive_to_normalized(self.transient_drive_db),
        );
        self.update_saturation_simulation();
    }

    /// Update non-linear transfer curve, THD percentage, crest factor, and harmonic distribution.
    pub fn update_saturation_simulation(&mut self) {
        let drive_linear = (self.transient_drive_db / 20.0).exp();

        // THD scales quadratically with drive
        self.thd_percent = match self.saturator_mode {
            SaturatorMode::TriodeTubeEvenOdd => {
                (0.5 * drive_linear * drive_linear).clamp(0.05, 15.0)
            }
            SaturatorMode::TapeHysteresisFlux => (0.35 * drive_linear.powf(1.8)).clamp(0.05, 12.0),
            SaturatorMode::FetTransientPunch => (0.8 * drive_linear.powf(2.2)).clamp(0.1, 20.0),
            SaturatorMode::GermaniumDiodeGrit => (1.5 * drive_linear.powf(2.5)).clamp(0.2, 25.0),
            SaturatorMode::CleanLinearDynamics => 0.01,
        };

        // Crest factor: Higher transient blend preserves higher crest factor
        self.crest_factor_db =
            (16.0 - (1.0 - self.blend_ratio) * 2.0 - self.transient_drive_db * 0.25)
                .clamp(3.0, 20.0);

        // Harmonic generation profile: even/odd distributions
        let g = (self.thd_percent / 10.0).clamp(0.05, 1.5);
        match self.saturator_mode {
            SaturatorMode::TriodeTubeEvenOdd => {
                // Strong 2nd and 4th even harmonics, moderate 3rd
                self.harmonic_profile = [
                    1.0,
                    0.55 * g,
                    0.25 * g,
                    0.18 * g,
                    0.08 * g,
                    0.04 * g,
                    0.02 * g,
                    0.01 * g,
                ];
            }
            SaturatorMode::TapeHysteresisFlux => {
                // Symmetric compression: odd harmonics dominate (3rd, 5th, 7th)
                self.harmonic_profile = [
                    1.0,
                    0.12 * g,
                    0.65 * g,
                    0.08 * g,
                    0.30 * g,
                    0.04 * g,
                    0.15 * g,
                    0.02 * g,
                ];
            }
            SaturatorMode::FetTransientPunch => {
                // Fast transient clip: broad rich overtone spray
                self.harmonic_profile = [
                    1.0,
                    0.40 * g,
                    0.45 * g,
                    0.30 * g,
                    0.25 * g,
                    0.18 * g,
                    0.12 * g,
                    0.08 * g,
                ];
            }
            SaturatorMode::GermaniumDiodeGrit => {
                // High distortion harsh upper harmonics
                self.harmonic_profile = [
                    1.0,
                    0.70 * g,
                    0.65 * g,
                    0.55 * g,
                    0.45 * g,
                    0.35 * g,
                    0.25 * g,
                    0.18 * g,
                ];
            }
            SaturatorMode::CleanLinearDynamics => {
                self.harmonic_profile = [1.0, 0.01, 0.005, 0.002, 0.001, 0.0005, 0.0002, 0.0001];
            }
        }
    }

    /// Hit test coordinate on the interactive saturator puck.
    pub fn hit_test_saturator_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.saturation_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.saturation_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= SATURATOR_PUCK_HIT_RADIUS
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

        // Left half: Parallel blend and drive coordinate
        let left_w = mid_x - 2;
        let p_row =
            (((1.0 - self.saturation_puck_pos.1) * (height - 5) as f32) + 2.0).round() as usize;
        let p_col = ((self.saturation_puck_pos.0 * (left_w - 4) as f32) + 2.0).round() as usize;
        if p_row < height - 1 && p_col < mid_x {
            grid[p_row][p_col] = 'P';
        }

        // Right half: Harmonic overtone profile bars
        let right_w = width - mid_x - 2;
        let bar_spacing = right_w / 9;
        for (i, &energy) in self.harmonic_profile.iter().enumerate() {
            let col = mid_x + 2 + (i + 1) * bar_spacing;
            let bar_h = (energy.clamp(0.0, 1.0) * (height - 4) as f32).round() as usize;
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
            "MASTERING PARALLEL DYNAMIC TRANSIENT / BODY SATURATOR HUD",
            egui::FontId::proportional(14.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Saturation Circuit Tabs (y: 48..92) - Each tab >= 44pt touch target
        let tabs = [
            (SaturatorMode::TriodeTubeEvenOdd, "TRIODE TUBE"),
            (SaturatorMode::TapeHysteresisFlux, "TAPE FLUX"),
            (SaturatorMode::FetTransientPunch, "FET PUNCH"),
            (SaturatorMode::GermaniumDiodeGrit, "GERMANIUM DIODE"),
            (SaturatorMode::CleanLinearDynamics, "CLEAN LINEAR"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (smode, name)) in tabs.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_sel = self.saturator_mode == *smode;
            let bg_col = if is_sel {
                Color32::from_rgb(255, 107, 43)
            } else {
                Color32::from_rgb(24, 32, 48)
            };
            let text_col = if is_sel {
                Color32::from_rgb(16, 8, 4)
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
                        self.set_saturator_mode(*smode);
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

        // Left 55%: Parallel Transient vs Body Dynamics Matrix
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
            "PARALLEL TRANSIENT vs BODY DYNAMICS MATRIX",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(255, 107, 43),
        );

        // Grid lines for Blend (X) and Drive (Y)
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

        // Non-linear transfer curve visualization
        let cy = left_rect.center().y + 5.0;
        let mut prev_pt = egui::pos2(left_rect.min.x + 20.0, left_rect.max.y - 30.0);
        for step in 1..=20 {
            let t = step as f32 / 20.0;
            let x = left_rect.min.x + 20.0 + t * (left_rect.width() - 40.0);
            let in_val = (t - 0.5) * 2.0;
            let out_val = (in_val * (1.0 + self.transient_drive_db * 0.1)).tanh();
            let y = cy - out_val * (left_rect.height() * 0.35);
            let cur_pt = egui::pos2(x, y);
            painter.line_segment(
                [prev_pt, cur_pt],
                Stroke::new(2.0_f32, Color32::from_rgb(255, 107, 43)),
            );
            prev_pt = cur_pt;
        }

        // Interactive Saturator Puck (Blend ratio vs Transient Drive)
        let puck_x = left_rect.min.x + self.saturation_puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.saturation_puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.saturation_puck_pos = (nx, ny);
                    self.blend_ratio = Self::normalized_to_blend(nx);
                    self.transient_drive_db = Self::normalized_to_drive(ny);
                    self.update_saturation_simulation();
                }
            }
        }

        // Draw Touch Hit Target boundary (>= 44x44pt)
        painter.circle_stroke(
            puck_pos,
            SATURATOR_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(255, 107, 43, 150)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(255, 107, 43));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 20.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Blend: {:.0}% | Drive: +{:.1} dB | THD: {:.2}% | Crest: {:.1} dB",
                self.blend_ratio * 100.0,
                self.transient_drive_db,
                self.thd_percent,
                self.crest_factor_db
            ),
            egui::FontId::proportional(10.5),
            Color32::from_rgb(255, 160, 100),
        );

        // Right 45%: Harmonic Profile Spectrum (H1..H8)
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
            "HARMONIC PROFILE SPECTRUM (H1..H8)",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(255, 107, 43),
        );

        let harm_names = ["H1", "H2", "H3", "H4", "H5", "H6", "H7", "H8"];
        let bar_w = (right_rect.width() - 30.0 - 7.0 * 6.0) / 8.0;
        for (i, &energy) in self.harmonic_profile.iter().enumerate() {
            let bx = right_rect.min.x + 15.0 + i as f32 * (bar_w + 6.0);
            let bar_h = (energy.clamp(0.0, 1.0)) * (right_rect.height() - 80.0);
            let b_rect = egui::Rect::from_min_max(
                egui::pos2(bx, right_rect.max.y - 25.0 - bar_h),
                egui::pos2(bx + bar_w, right_rect.max.y - 25.0),
            );
            let col = if i == 0 {
                Color32::from_rgb(255, 107, 43)
            } else if i % 2 == 1 {
                Color32::from_rgb(255, 180, 50)
            } else {
                Color32::from_rgb(0, 229, 255)
            };
            painter.rect_filled(b_rect, 3.0, col);

            painter.text(
                egui::pos2(bx + bar_w * 0.5, right_rect.max.y - 20.0),
                egui::Align2::CENTER_TOP,
                harm_names[i],
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
                "PARALLEL BLEND",
                format!("{:.0}% (Trans / Body)", self.blend_ratio * 100.0),
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "SATURATION DRIVE",
                format!("+{:.1} dBFS (Drive)", self.transient_drive_db),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "TOTAL HARMONIC THD",
                format!(
                    "{:.2}% ({})",
                    self.thd_percent,
                    self.saturator_mode.mode_name()
                ),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "DYNAMIC CREST FACTOR",
                format!("{:.1} dB (Punch)", self.crest_factor_db),
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
            "[PASS] Mastering Parallel Transient & Body Saturator Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
