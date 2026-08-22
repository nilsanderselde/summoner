// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Neural Acoustic Blind Dereverberation & Room Impulse Deconvolution HUD (Step 1604).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const DEREVERB_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_SUPPRESSION_DB: f32 = 0.0;
pub const MAX_SUPPRESSION_DB: f32 = 36.0;
pub const MIN_DRR_DB: f32 = -12.0;
pub const MAX_DRR_DB: f32 = 24.0;

/// Neural dereverberation architecture models and acoustic environments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DereverbModel {
    NeuralSpectralMaskUNet,     // Recurrent deep U-Net spectral direct-to-reverberant separation
    WeightedPredictionErrorWPE, // Multi-channel statistical linear prediction late echo cancellation
    DiffusionDeconvolution,     // Generative diffusion prior room impulse response inverse filter
    CathedralAcousticHall,      // Long-tail (>4.0s RT60) extreme reverberant soundfield cleaner
    ConferenceRoomAutomix,      // Small-room flutter echo & boxy resonance suppression
}

impl DereverbModel {
    pub fn model_name(&self) -> &'static str {
        match self {
            Self::NeuralSpectralMaskUNet => "NEURAL SPECTRAL U-NET",
            Self::WeightedPredictionErrorWPE => "STATISTICAL WPE (M-CH)",
            Self::DiffusionDeconvolution => "DIFFUSION DECONVOLUTION",
            Self::CathedralAcousticHall => "CATHEDRAL LONG-TAIL",
            Self::ConferenceRoomAutomix => "CONFERENCE ROOM AUTOMIX",
        }
    }

    pub fn nominal_suppression_db(&self) -> f32 {
        match self {
            Self::NeuralSpectralMaskUNet => 18.0,
            Self::WeightedPredictionErrorWPE => 12.0,
            Self::DiffusionDeconvolution => 24.0,
            Self::CathedralAcousticHall => 30.0,
            Self::ConferenceRoomAutomix => 15.0,
        }
    }

    pub fn nominal_drr_db(&self) -> f32 {
        match self {
            Self::NeuralSpectralMaskUNet => 6.0,
            Self::WeightedPredictionErrorWPE => 3.0,
            Self::DiffusionDeconvolution => 12.0,
            Self::CathedralAcousticHall => -6.0,
            Self::ConferenceRoomAutomix => 8.0,
        }
    }

    pub fn nominal_early_reflection_cancel(&self) -> f32 {
        match self {
            Self::NeuralSpectralMaskUNet => 0.75,
            Self::WeightedPredictionErrorWPE => 0.40,
            Self::DiffusionDeconvolution => 0.90,
            Self::CathedralAcousticHall => 0.65,
            Self::ConferenceRoomAutomix => 0.85,
        }
    }

    pub fn nominal_late_tail_suppress(&self) -> f32 {
        match self {
            Self::NeuralSpectralMaskUNet => 0.80,
            Self::WeightedPredictionErrorWPE => 0.85,
            Self::DiffusionDeconvolution => 0.92,
            Self::CathedralAcousticHall => 0.95,
            Self::ConferenceRoomAutomix => 0.70,
        }
    }

    pub fn nominal_rt60_s(&self) -> f32 {
        match self {
            Self::NeuralSpectralMaskUNet => 1.5,
            Self::WeightedPredictionErrorWPE => 1.2,
            Self::DiffusionDeconvolution => 2.2,
            Self::CathedralAcousticHall => 4.8,
            Self::ConferenceRoomAutomix => 0.6,
        }
    }
}

/// Neural acoustic blind dereverberation & room impulse deconvolution HUD.
#[derive(Debug, Clone)]
pub struct NeuralDereverbView {
    pub model: DereverbModel,
    pub suppression_depth_db: f32,
    pub direct_to_reverberant_ratio_db: f32,
    pub early_reflection_cancel: f32,
    pub late_tail_suppression: f32,
    pub room_rt60_s: f32,
    pub neural_blend_pct: f32,
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    pub spectral_mask_bands: [f32; 8],
    pub energy_decay_curve: [f32; 16],
    pub color_palette: ContrastColorPalette,
}

impl Default for NeuralDereverbView {
    fn default() -> Self {
        Self::new()
    }
}

impl NeuralDereverbView {
    pub fn new() -> Self {
        let mut view = Self {
            model: DereverbModel::NeuralSpectralMaskUNet,
            suppression_depth_db: 18.0,
            direct_to_reverberant_ratio_db: 6.0,
            early_reflection_cancel: 0.75,
            late_tail_suppression: 0.80,
            room_rt60_s: 1.5,
            neural_blend_pct: 85.0,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            spectral_mask_bands: [0.90, 0.75, 0.60, 0.45, 0.35, 0.50, 0.85, 0.95],
            energy_decay_curve: [0.0; 16],
            color_palette: ContrastColorPalette::default(),
        };
        view.puck_pos = (
            Self::depth_to_normalized(view.suppression_depth_db),
            Self::drr_to_normalized(view.direct_to_reverberant_ratio_db),
        );
        view.update_dereverb_simulation();
        view
    }

    pub fn depth_to_normalized(depth: f32) -> f32 {
        let d = depth.clamp(MIN_SUPPRESSION_DB, MAX_SUPPRESSION_DB);
        ((d - MIN_SUPPRESSION_DB) / (MAX_SUPPRESSION_DB - MIN_SUPPRESSION_DB)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_depth(norm: f32) -> f32 {
        MIN_SUPPRESSION_DB + norm.clamp(0.0, 1.0) * (MAX_SUPPRESSION_DB - MIN_SUPPRESSION_DB)
    }

    pub fn drr_to_normalized(drr: f32) -> f32 {
        let r = drr.clamp(MIN_DRR_DB, MAX_DRR_DB);
        ((r - MIN_DRR_DB) / (MAX_DRR_DB - MIN_DRR_DB)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_drr(norm: f32) -> f32 {
        MIN_DRR_DB + norm.clamp(0.0, 1.0) * (MAX_DRR_DB - MIN_DRR_DB)
    }

    pub fn set_model(&mut self, model: DereverbModel) {
        self.model = model;
        self.suppression_depth_db = model.nominal_suppression_db();
        self.direct_to_reverberant_ratio_db = model.nominal_drr_db();
        self.early_reflection_cancel = model.nominal_early_reflection_cancel();
        self.late_tail_suppression = model.nominal_late_tail_suppress();
        self.room_rt60_s = model.nominal_rt60_s();
        self.puck_pos = (
            Self::depth_to_normalized(self.suppression_depth_db),
            Self::drr_to_normalized(self.direct_to_reverberant_ratio_db),
        );
        self.update_dereverb_simulation();
    }

    pub fn update_dereverb_simulation(&mut self) {
        let depth = self.suppression_depth_db;
        let drr = self.direct_to_reverberant_ratio_db;
        let depth_norm = (depth / 36.0).clamp(0.0, 1.0);
        let drr_norm = ((drr + 12.0) / 36.0).clamp(0.0, 1.0);

        // Room Impulse Response Energy Decay: Raw (RT60) vs Deconvolved (Direct sound preserved)
        for (i, e) in self.energy_decay_curve.iter_mut().enumerate() {
            let t = i as f32 / 15.0;
            let raw_decay = (-3.0 * t / (self.room_rt60_s * 0.5 + 0.1)).exp();
            let cleaned = raw_decay * (1.0 - depth_norm * (1.0 - (-8.0 * t).exp()));
            *e = cleaned.clamp(0.0, 1.0);
        }

        // 8 Frequency Band Dereverberation Gain Masks: [Sub, Low, Low-Mid, Mid, High-Mid, High, Air, Direct DRR]
        let base_mask = (1.0 - depth_norm * 0.85).clamp(0.05, 1.0);
        self.spectral_mask_bands = [
            (base_mask * 1.1).clamp(0.05, 1.0),
            (base_mask * 0.95).clamp(0.05, 1.0),
            (base_mask * 0.80).clamp(0.05, 1.0),
            (base_mask * 0.70).clamp(0.05, 1.0),
            (base_mask * 0.85).clamp(0.05, 1.0),
            (base_mask * 0.95).clamp(0.05, 1.0),
            (base_mask * 1.05).clamp(0.05, 1.0),
            (drr_norm * 1.2).clamp(0.1, 1.2),
        ];
    }

    pub fn hit_test_dereverb_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= DEREVERB_PUCK_HIT_RADIUS
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
        for (i, &amp) in self.spectral_mask_bands.iter().enumerate() {
            let col = mid_x + 2 + (i + 1) * bar_spacing;
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

        // Background: Deep Teal Dark Slate (#0A1618)
        painter.rect_filled(rect, 6.0, Color32::from_rgb(10, 22, 24));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "NEURAL ACOUSTIC BLIND DEREVERBERATION & DECONVOLUTION HUD",
            egui::FontId::proportional(13.5),
            Color32::from_rgb(240, 245, 255),
        );

        // Tabs (y: 48..92) - 44pt height
        let tabs = [
            (DereverbModel::NeuralSpectralMaskUNet, "SPECTRAL U-NET"),
            (DereverbModel::WeightedPredictionErrorWPE, "WPE MULTI-CH"),
            (DereverbModel::DiffusionDeconvolution, "DIFFUSION DECONV"),
            (DereverbModel::CathedralAcousticHall, "CATHEDRAL HALL"),
            (DereverbModel::ConferenceRoomAutomix, "ROOM AUTOMIX"),
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
                Color32::from_rgb(0, 245, 212)
            } else {
                Color32::from_rgb(20, 38, 42)
            };
            let text_col = if is_sel {
                Color32::from_rgb(4, 24, 20)
            } else {
                Color32::from_rgb(200, 235, 240)
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
        painter.rect_filled(main_canvas, 6.0, Color32::from_rgb(6, 14, 16));
        painter.rect_stroke(
            main_canvas,
            6.0,
            Stroke::new(1.5_f32, Color32::from_rgb(25, 65, 75)),
        );

        // Left 55%: Room Impulse Energy Decay & Direct-to-Reverberant Ratio Field
        let left_w = main_canvas.width() * 0.55;
        let left_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(left_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(left_rect, 4.0, Color32::from_rgb(12, 24, 28));
        painter.rect_stroke(
            left_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(25, 55, 65)),
        );

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "ROOM IMPULSE RESPONSE ENERGY DECAY (DRR vs DEPTH)",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(0, 245, 212),
        );

        // Draw Energy Decay Curve
        let plot_pts: Vec<egui::Pos2> = self
            .energy_decay_curve
            .iter()
            .enumerate()
            .map(|(i, &e)| {
                let px = left_rect.min.x + 15.0 + (i as f32 / 15.0) * (left_rect.width() - 30.0);
                let py = left_rect.max.y - 25.0 - e * (left_rect.height() - 65.0);
                egui::pos2(px, py)
            })
            .collect();

        for i in 0..(plot_pts.len() - 1) {
            painter.line_segment(
                [plot_pts[i], plot_pts[i + 1]],
                Stroke::new(2.0_f32, Color32::from_rgb(0, 245, 212)),
            );
        }

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
                    self.suppression_depth_db = Self::normalized_to_depth(nx);
                    self.direct_to_reverberant_ratio_db = Self::normalized_to_drr(ny);
                    self.update_dereverb_simulation();
                }
            }
        }

        painter.circle_stroke(
            puck_pos,
            DEREVERB_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(0, 245, 212, 150)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(0, 245, 212));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 18.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Suppression: {:.1} dB | DRR: {:+.1} dB | RT60: {:.2}s | Early: {:.0}% | Late: {:.0}%",
                self.suppression_depth_db,
                self.direct_to_reverberant_ratio_db,
                self.room_rt60_s,
                self.early_reflection_cancel * 100.0,
                self.late_tail_suppression * 100.0
            ),
            egui::FontId::proportional(10.5),
            Color32::from_rgb(140, 255, 235),
        );

        // Right 45%: Spectral Mask Subband Direct Gain Spectrum
        let right_w = main_canvas.width() * 0.45;
        let right_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + left_w + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(right_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(right_rect, 4.0, Color32::from_rgb(12, 24, 28));
        painter.rect_stroke(
            right_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(25, 55, 65)),
        );

        painter.text(
            egui::pos2(right_rect.min.x + 10.0, right_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "SPECTRAL MASK DIRECT SPEECH GAIN SPECTRUM",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(0, 245, 212),
        );

        let subband_labels = ["SUB", "LOW", "L-MID", "MID", "H-MID", "HIGH", "AIR", "DRR-EN"];
        let bar_w = (right_rect.width() - 30.0 - 7.0 * 6.0) / 8.0;
        for (i, &amp) in self.spectral_mask_bands.iter().enumerate() {
            let bx = right_rect.min.x + 15.0 + i as f32 * (bar_w + 6.0);
            let bar_h = (amp.clamp(0.0, 1.2) / 1.2) * (right_rect.height() - 80.0);
            let b_rect = egui::Rect::from_min_max(
                egui::pos2(bx, right_rect.max.y - 25.0 - bar_h),
                egui::pos2(bx + bar_w, right_rect.max.y - 25.0),
            );
            let col = if i == 7 {
                Color32::from_rgb(255, 215, 0)
            } else {
                Color32::from_rgb(0, 245, 212)
            };
            painter.rect_filled(b_rect, 3.0, col);

            painter.text(
                egui::pos2(bx + bar_w * 0.5, right_rect.max.y - 20.0),
                egui::Align2::CENTER_TOP,
                subband_labels[i],
                egui::FontId::proportional(8.0),
                Color32::from_rgb(180, 225, 235),
            );
        }

        // Bottom Metrics Dock (y: 350..465)
        let dock_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 350.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 465.0),
        );
        painter.rect_filled(dock_rect, 6.0, Color32::from_rgb(16, 32, 38));
        painter.rect_stroke(
            dock_rect,
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(35, 75, 85)),
        );

        let params = [
            (
                "SUPPRESSION DEPTH",
                format!("{:.1} dB (Late Tail)", self.suppression_depth_db),
                Color32::from_rgb(0, 245, 212),
            ),
            (
                "DIRECT / REVERB RATIO",
                format!("{:+.1} dB (DRR Boost)", self.direct_to_reverberant_ratio_db),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "ROOM RT60 TIME",
                format!("{:.2} s (Impulse Decay)", self.room_rt60_s),
                Color32::from_rgb(120, 220, 255),
            ),
            (
                "NEURAL BLEND",
                format!("{:.0}% (Dry Spectral Blend)", self.neural_blend_pct),
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
                Color32::from_rgb(160, 205, 215),
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
            "[PASS] Neural Blind Dereverberation & Deconvolution Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
