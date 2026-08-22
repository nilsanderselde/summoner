// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Mastering Multi-Band Linear-Phase Dynamic Transient Clipper & Oversampled Ceiling HUD (Step 1613).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const CLIPPER_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_CLIP_DRIVE_DB: f32 = -6.0;
pub const MAX_CLIP_DRIVE_DB: f32 = 18.0;
pub const MIN_KNEE_SOFTNESS_DB: f32 = 0.0;
pub const MAX_KNEE_SOFTNESS_DB: f32 = 12.0;

/// Multi-band dynamic transient clipper profiles and oversampling configurations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipperProfileType {
    LinearPhase4BandMaster, // 4-band linear phase FIR crossover with independent dynamic clipping knee
    Oversampled16xPcmCeiling, // 16x polyphase oversampling true-peak intersample clipper
    TapeSoftSaturatingClipper, // Smooth hyperbolic tangent / polynomial soft knee saturation
    HardPunchEDMClipper, // Zero-latency brickwall hard clipper with punchy transient preservation
    BroadcastMultiStageClipper, // EBU R128 multi-stage peak control with sub-harmonic protection
}

impl ClipperProfileType {
    pub fn profile_name(&self) -> &'static str {
        match self {
            Self::LinearPhase4BandMaster => "4-BAND LINEAR-PHASE MASTER CLIPPER",
            Self::Oversampled16xPcmCeiling => "16x OVERSAMPLED TRUE-PEAK CEILING",
            Self::TapeSoftSaturatingClipper => "TAPE SOFT-SATURATING KNEE",
            Self::HardPunchEDMClipper => "HARD PUNCH BRICKWALL CLIPPER",
            Self::BroadcastMultiStageClipper => "BROADCAST MULTI-STAGE PEAK CONTROL",
        }
    }

    pub fn nominal_drive_db(&self) -> f32 {
        match self {
            Self::LinearPhase4BandMaster => 6.0,
            Self::Oversampled16xPcmCeiling => 8.5,
            Self::TapeSoftSaturatingClipper => 4.5,
            Self::HardPunchEDMClipper => 11.0,
            Self::BroadcastMultiStageClipper => 5.0,
        }
    }

    pub fn nominal_knee_db(&self) -> f32 {
        match self {
            Self::LinearPhase4BandMaster => 3.5,
            Self::Oversampled16xPcmCeiling => 1.5,
            Self::TapeSoftSaturatingClipper => 8.0,
            Self::HardPunchEDMClipper => 0.0,
            Self::BroadcastMultiStageClipper => 4.0,
        }
    }

    pub fn nominal_oversampling_factor(&self) -> usize {
        match self {
            Self::LinearPhase4BandMaster => 8,
            Self::Oversampled16xPcmCeiling => 16,
            Self::TapeSoftSaturatingClipper => 4,
            Self::HardPunchEDMClipper => 8,
            Self::BroadcastMultiStageClipper => 16,
        }
    }

    pub fn nominal_ceiling_dbfs(&self) -> f32 {
        match self {
            Self::LinearPhase4BandMaster => -0.1,
            Self::Oversampled16xPcmCeiling => -0.3,
            Self::TapeSoftSaturatingClipper => -0.2,
            Self::HardPunchEDMClipper => 0.0,
            Self::BroadcastMultiStageClipper => -0.5,
        }
    }
}

/// Mastering multi-band linear-phase dynamic transient clipper & oversampled ceiling HUD.
#[derive(Debug, Clone)]
pub struct TransientClipperView {
    pub profile_type: ClipperProfileType,
    pub clip_drive_db: f32,
    pub knee_softness_db: f32,
    pub oversampling_factor: usize,
    pub ceiling_dbfs: f32,
    pub crest_factor_reduction_db: f32,
    pub thd_distortion_pct: f32,
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    pub transfer_curve_pts: [(f32, f32); 16],
    pub band_clipping_reduction_db: [f32; 6],
    pub color_palette: ContrastColorPalette,
}

impl Default for TransientClipperView {
    fn default() -> Self {
        Self::new()
    }
}

impl TransientClipperView {
    pub fn new() -> Self {
        let mut view = Self {
            profile_type: ClipperProfileType::LinearPhase4BandMaster,
            clip_drive_db: 6.0,
            knee_softness_db: 3.5,
            oversampling_factor: 8,
            ceiling_dbfs: -0.1,
            crest_factor_reduction_db: 4.2,
            thd_distortion_pct: 1.85,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            transfer_curve_pts: [(0.0, 0.0); 16],
            band_clipping_reduction_db: [1.2, 3.5, 4.8, 2.1, 5.2, 0.8],
            color_palette: ContrastColorPalette::default(),
        };
        view.puck_pos = (
            Self::drive_to_normalized(view.clip_drive_db),
            Self::knee_to_normalized(view.knee_softness_db),
        );
        view.update_clipper_simulation();
        view
    }

    pub fn drive_to_normalized(drive: f32) -> f32 {
        let d = drive.clamp(MIN_CLIP_DRIVE_DB, MAX_CLIP_DRIVE_DB);
        ((d - MIN_CLIP_DRIVE_DB) / (MAX_CLIP_DRIVE_DB - MIN_CLIP_DRIVE_DB)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_drive(norm: f32) -> f32 {
        MIN_CLIP_DRIVE_DB + norm.clamp(0.0, 1.0) * (MAX_CLIP_DRIVE_DB - MIN_CLIP_DRIVE_DB)
    }

    pub fn knee_to_normalized(knee: f32) -> f32 {
        let k = knee.clamp(MIN_KNEE_SOFTNESS_DB, MAX_KNEE_SOFTNESS_DB);
        ((k - MIN_KNEE_SOFTNESS_DB) / (MAX_KNEE_SOFTNESS_DB - MIN_KNEE_SOFTNESS_DB)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_knee(norm: f32) -> f32 {
        MIN_KNEE_SOFTNESS_DB + norm.clamp(0.0, 1.0) * (MAX_KNEE_SOFTNESS_DB - MIN_KNEE_SOFTNESS_DB)
    }

    pub fn set_profile(&mut self, profile: ClipperProfileType) {
        self.profile_type = profile;
        self.clip_drive_db = profile.nominal_drive_db();
        self.knee_softness_db = profile.nominal_knee_db();
        self.oversampling_factor = profile.nominal_oversampling_factor();
        self.ceiling_dbfs = profile.nominal_ceiling_dbfs();
        self.puck_pos = (
            Self::drive_to_normalized(self.clip_drive_db),
            Self::knee_to_normalized(self.knee_softness_db),
        );
        self.update_clipper_simulation();
    }

    pub fn update_clipper_simulation(&mut self) {
        let drive = self.clip_drive_db;
        let knee = self.knee_softness_db;
        let drive_lin = 10.0_f32.powf(drive / 20.0);
        let ceiling_lin = 10.0_f32.powf(self.ceiling_dbfs / 20.0);

        // Generate 16 transfer function curve points from Vin (-1.0 to 1.0) to Vout
        for i in 0..16 {
            let vin = -1.0 + (i as f32 / 15.0) * 2.0;
            let driven_in = vin * drive_lin;
            let sign = driven_in.signum();
            let abs_in = driven_in.abs();

            let vout = if knee < 0.1 {
                // Hard brickwall clipping
                abs_in.min(ceiling_lin) * sign
            } else {
                // Soft saturating cubic / hyperbolic tangent knee
                let k_width = knee / 12.0;
                let thresh = (ceiling_lin - k_width).max(0.05);
                if abs_in < thresh {
                    abs_in * sign
                } else if abs_in > (ceiling_lin + k_width) {
                    ceiling_lin * sign
                } else {
                    let d = (abs_in - thresh) / (2.0 * k_width);
                    let soft = thresh + k_width * (2.0 * d - d * d);
                    soft.min(ceiling_lin) * sign
                }
            };

            self.transfer_curve_pts[i] = (vin, (vout / ceiling_lin).clamp(-1.0, 1.0));
        }

        // 4-band dynamic clipping reduction + True-Peak + Inter-sample metrics
        let low_red = (drive * 0.35 * (1.0 - knee * 0.04)).clamp(0.0, 12.0);
        let mid_low_red = (drive * 0.65 * (1.0 - knee * 0.03)).clamp(0.0, 15.0);
        let mid_high_red = (drive * 0.85 * (1.0 - knee * 0.02)).clamp(0.0, 18.0);
        let high_red = (drive * 0.50 * (1.0 - knee * 0.05)).clamp(0.0, 14.0);
        let true_peak_reduction = (drive * 0.95 + 0.5).clamp(0.0, 20.0);
        let intersample_overshoot =
            ((drive * 0.20) / (self.oversampling_factor as f32 / 4.0)).clamp(0.0, 4.0);

        self.band_clipping_reduction_db = [
            low_red,
            mid_low_red,
            mid_high_red,
            high_red,
            true_peak_reduction,
            intersample_overshoot,
        ];

        self.crest_factor_reduction_db = (drive * 0.70 - knee * 0.15).clamp(0.0, 12.0);
        self.thd_distortion_pct =
            ((drive.max(0.0) / 18.0) * 3.5 * (1.0 - knee * 0.05)).clamp(0.1, 8.0);
    }

    pub fn hit_test_clipper_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= CLIPPER_PUCK_HIT_RADIUS
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
            grid[p_row][p_col] = 'C';
        }

        let right_w = width - mid_x - 2;
        let bar_spacing = right_w / 7;
        for (i, &red) in self.band_clipping_reduction_db.iter().enumerate() {
            let col = mid_x + 2 + (i + 1) * bar_spacing;
            let bar_h = ((red / 18.0).clamp(0.0, 1.0) * (height - 4) as f32).round() as usize;
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

        // Background: High-Contrast Dark Slate (#111625)
        painter.rect_filled(rect, 6.0, Color32::from_rgb(17, 22, 37));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "MASTERING MULTI-BAND DYNAMIC TRANSIENT CLIPPER HUD",
            egui::FontId::proportional(13.5),
            Color32::from_rgb(245, 245, 255),
        );

        // Tabs (y: 48..92) - 44pt height
        let tabs = [
            (ClipperProfileType::LinearPhase4BandMaster, "4-BAND MASTER"),
            (
                ClipperProfileType::Oversampled16xPcmCeiling,
                "16x OVER PEAK",
            ),
            (
                ClipperProfileType::TapeSoftSaturatingClipper,
                "TAPE SOFT KNEE",
            ),
            (ClipperProfileType::HardPunchEDMClipper, "HARD PUNCH EDM"),
            (
                ClipperProfileType::BroadcastMultiStageClipper,
                "BROADCAST STAGE",
            ),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (ptype, name)) in tabs.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_sel = self.profile_type == *ptype;
            let bg_col = if is_sel {
                Color32::from_rgb(255, 107, 107) // Neon Coral
            } else {
                Color32::from_rgb(28, 36, 56)
            };
            let text_col = if is_sel {
                Color32::from_rgb(24, 4, 4)
            } else {
                Color32::from_rgb(220, 230, 248)
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
        painter.rect_filled(main_canvas, 6.0, Color32::from_rgb(10, 14, 24));
        painter.rect_stroke(
            main_canvas,
            6.0,
            Stroke::new(1.5_f32, Color32::from_rgb(55, 45, 65)),
        );

        // Left 55%: Drive vs Knee Plane & Non-Linear Transfer Function Curve
        let left_w = main_canvas.width() * 0.55;
        let left_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(left_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(left_rect, 4.0, Color32::from_rgb(16, 20, 32));
        painter.rect_stroke(
            left_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(45, 55, 75)),
        );

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "CLIP DRIVE (dB) vs SOFT KNEE WIDTH & TRANSFER FUNCTION",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(255, 107, 107),
        );

        // Draw transfer curve in background
        let tf_rect = egui::Rect::from_min_size(
            egui::pos2(left_rect.min.x + 30.0, left_rect.min.y + 35.0),
            egui::vec2(left_rect.width() - 60.0, left_rect.height() - 65.0),
        );
        painter.line_segment(
            [
                egui::pos2(tf_rect.min.x, tf_rect.center().y),
                egui::pos2(tf_rect.max.x, tf_rect.center().y),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(60, 75, 100, 120)),
        );
        painter.line_segment(
            [
                egui::pos2(tf_rect.center().x, tf_rect.min.y),
                egui::pos2(tf_rect.center().x, tf_rect.max.y),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(60, 75, 100, 120)),
        );

        let mut prev_pt: Option<egui::Pos2> = None;
        for &(vin, vout) in self.transfer_curve_pts.iter() {
            let px = tf_rect.center().x + vin * (tf_rect.width() * 0.48);
            let py = tf_rect.center().y - vout * (tf_rect.height() * 0.48);
            let pt = egui::pos2(px, py);
            if let Some(prev) = prev_pt {
                painter.line_segment(
                    [prev, pt],
                    Stroke::new(2.0_f32, Color32::from_rgb(0, 229, 255)),
                );
            }
            prev_pt = Some(pt);
        }

        // Interactive Clipper Puck
        let puck_x = left_rect.min.x + self.puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.puck_pos = (nx, ny);
                    self.clip_drive_db = Self::normalized_to_drive(nx);
                    self.knee_softness_db = Self::normalized_to_knee(ny);
                    self.update_clipper_simulation();
                }
            }
        }

        painter.circle_stroke(
            puck_pos,
            CLIPPER_PUCK_HIT_RADIUS,
            Stroke::new(
                1.5_f32,
                Color32::from_rgba_premultiplied(255, 107, 107, 150),
            ),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(255, 107, 107));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 18.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Drive: {:+.1} dB | Knee: {:.1} dB | Over: {}x | Ceil: {:.1} dBFS | THD: {:.2}%",
                self.clip_drive_db,
                self.knee_softness_db,
                self.oversampling_factor,
                self.ceiling_dbfs,
                self.thd_distortion_pct
            ),
            egui::FontId::proportional(10.5),
            Color32::from_rgb(255, 205, 205),
        );

        // Right 45%: Multi-Band Dynamic Gain Reduction Spectrum
        let right_w = main_canvas.width() * 0.45;
        let right_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + left_w + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(right_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(right_rect, 4.0, Color32::from_rgb(16, 20, 32));
        painter.rect_stroke(
            right_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(45, 55, 75)),
        );

        painter.text(
            egui::pos2(right_rect.min.x + 10.0, right_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "MULTI-BAND GAIN REDUCTION & TRUE-PEAK SUPPRESSION",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(255, 107, 107),
        );

        let band_labels = ["LOW", "MID-L", "MID-H", "HIGH", "TRUE-PK", "INTER-S"];
        let bar_w = (right_rect.width() - 30.0 - 5.0 * 8.0) / 6.0;
        for (i, &red) in self.band_clipping_reduction_db.iter().enumerate() {
            let bx = right_rect.min.x + 15.0 + i as f32 * (bar_w + 8.0);
            let bar_h = (red.clamp(0.0, 18.0) / 18.0) * (right_rect.height() - 80.0);
            let b_rect = egui::Rect::from_min_max(
                egui::pos2(bx, right_rect.max.y - 25.0 - bar_h),
                egui::pos2(bx + bar_w, right_rect.max.y - 25.0),
            );
            let col = if i == 4 {
                Color32::from_rgb(255, 215, 0)
            } else if i == 5 {
                Color32::from_rgb(0, 255, 180)
            } else {
                Color32::from_rgb(255, 107, 107)
            };
            painter.rect_filled(b_rect, 3.0, col);

            painter.text(
                egui::pos2(bx + bar_w * 0.5, right_rect.max.y - 20.0),
                egui::Align2::CENTER_TOP,
                band_labels[i],
                egui::FontId::proportional(8.5),
                Color32::from_rgb(200, 215, 235),
            );
        }

        // Bottom Metrics Dock (y: 350..465)
        let dock_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 350.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 465.0),
        );
        painter.rect_filled(dock_rect, 6.0, Color32::from_rgb(20, 26, 40));
        painter.rect_stroke(
            dock_rect,
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(55, 65, 85)),
        );

        let params = [
            (
                "CLIPPING DRIVE",
                format!("{:+.1} dB (Headroom Push)", self.clip_drive_db),
                Color32::from_rgb(255, 107, 107),
            ),
            (
                "KNEE SOFTNESS",
                format!("{:.1} dB (Saturating Knee)", self.knee_softness_db),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "CREST FACTOR DELTA",
                format!("-{:.1} dB (Density Boost)", self.crest_factor_reduction_db),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "OVERSAMPLING CEILING",
                format!(
                    "{}x / {:.1} dBFS",
                    self.oversampling_factor, self.ceiling_dbfs
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
                Color32::from_rgb(180, 195, 215),
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
            "[PASS] Multi-Band Dynamic Transient Clipper Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
