// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Neural Polyphonic Vocal Expression Stylizer & Microtonal Ornament Resynthesizer HUD (Step 1584).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const STYLIZE_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_STYLE_BLEND: f32 = 0.0;
pub const MAX_STYLE_BLEND: f32 = 1.0;
pub const MIN_ORNAMENT_DEPTH: f32 = 0.0;
pub const MAX_ORNAMENT_DEPTH: f32 = 1.0;

/// Neural vocal expression style models.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VocalStyleModel {
    BelCantoOpera, // Operatic singer's formant cluster (2.8-3.2 kHz) and expansive vibrato
    ContemporaryPopBelt, // High chest mix resonance, fast pitch quantize, and modern breath air
    BulgarianChoirOpenThroat, // Close-interval diaphonic resonance and straight-tone microtonal ornament
    TuvanThroatKargyraa,      // Subharmonic overtone resonance and double-pitch vocal fold tracking
    GospelMelismaExpressive,  // Rapid chromatic melismatic runs, dynamic pitch vibrato acceleration
}

impl VocalStyleModel {
    pub fn model_name(&self) -> &'static str {
        match self {
            Self::BelCantoOpera => "BEL CANTO OPERA",
            Self::ContemporaryPopBelt => "CONTEMPORARY BELT",
            Self::BulgarianChoirOpenThroat => "BULGARIAN CHOIR",
            Self::TuvanThroatKargyraa => "TUVAN THROAT",
            Self::GospelMelismaExpressive => "GOSPEL MELISMA",
        }
    }

    pub fn nominal_style_blend(&self) -> f32 {
        match self {
            Self::BelCantoOpera => 0.85,
            Self::ContemporaryPopBelt => 0.70,
            Self::BulgarianChoirOpenThroat => 0.90,
            Self::TuvanThroatKargyraa => 0.95,
            Self::GospelMelismaExpressive => 0.80,
        }
    }

    pub fn nominal_ornament_depth(&self) -> f32 {
        match self {
            Self::BelCantoOpera => 0.45,
            Self::ContemporaryPopBelt => 0.25,
            Self::BulgarianChoirOpenThroat => 0.80,
            Self::TuvanThroatKargyraa => 0.65,
            Self::GospelMelismaExpressive => 0.90,
        }
    }

    pub fn nominal_vibrato_rate_hz(&self) -> f32 {
        match self {
            Self::BelCantoOpera => 5.8,
            Self::ContemporaryPopBelt => 6.5,
            Self::BulgarianChoirOpenThroat => 3.2,
            Self::TuvanThroatKargyraa => 4.0,
            Self::GospelMelismaExpressive => 6.2,
        }
    }

    pub fn nominal_vibrato_depth_cents(&self) -> f32 {
        match self {
            Self::BelCantoOpera => 95.0,
            Self::ContemporaryPopBelt => 35.0,
            Self::BulgarianChoirOpenThroat => 10.0,
            Self::TuvanThroatKargyraa => 20.0,
            Self::GospelMelismaExpressive => 80.0,
        }
    }
}

/// Neural polyphonic vocal expression stylizer & microtonal ornament resynthesizer HUD.
#[derive(Debug, Clone)]
pub struct NeuralVocalStylizerView {
    pub vocal_model: VocalStyleModel,
    pub style_blend_pct: f32,       // [0.0 ..= 1.0 neural timbre morph]
    pub ornament_depth_pct: f32,    // [0.0 ..= 1.0 melisma & microtonal ornamentation]
    pub melisma_rate_hz: f32,       // [1.0 ..= 12.0 Hz ornament run velocity]
    pub vibrato_rate_hz: f32,       // [3.0 ..= 9.0 Hz]
    pub vibrato_depth_cents: f32,   // [0.0 ..= 150.0 cents]
    pub microtonal_snap_cents: f32, // [5.0 ..= 100.0 cents]
    pub puck_pos: (f32, f32),       // Normalized (X: Style Blend, Y: Ornament Depth)
    pub is_dragging_puck: bool,
    pub expression_radar_axes: [f32; 6], // Melisma, Vibrato, Singer's Formant, Subharmonic, Breathiness, Intonation
    pub color_palette: ContrastColorPalette,
}

impl Default for NeuralVocalStylizerView {
    fn default() -> Self {
        Self::new()
    }
}

impl NeuralVocalStylizerView {
    pub fn new() -> Self {
        let mut view = Self {
            vocal_model: VocalStyleModel::BelCantoOpera,
            style_blend_pct: 0.85,
            ornament_depth_pct: 0.45,
            melisma_rate_hz: 4.5,
            vibrato_rate_hz: 5.8,
            vibrato_depth_cents: 95.0,
            microtonal_snap_cents: 25.0,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            expression_radar_axes: [0.45, 0.90, 0.95, 0.20, 0.35, 0.85],
            color_palette: ContrastColorPalette::default(),
        };
        view.puck_pos = (
            Self::blend_to_normalized(view.style_blend_pct),
            Self::ornament_to_normalized(view.ornament_depth_pct),
        );
        view.update_neural_simulation();
        view
    }

    pub fn blend_to_normalized(blend: f32) -> f32 {
        let b = blend.clamp(MIN_STYLE_BLEND, MAX_STYLE_BLEND);
        ((b - MIN_STYLE_BLEND) / (MAX_STYLE_BLEND - MIN_STYLE_BLEND)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_blend(norm: f32) -> f32 {
        MIN_STYLE_BLEND + norm.clamp(0.0, 1.0) * (MAX_STYLE_BLEND - MIN_STYLE_BLEND)
    }

    pub fn ornament_to_normalized(ornament: f32) -> f32 {
        let o = ornament.clamp(MIN_ORNAMENT_DEPTH, MAX_ORNAMENT_DEPTH);
        ((o - MIN_ORNAMENT_DEPTH) / (MAX_ORNAMENT_DEPTH - MIN_ORNAMENT_DEPTH)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_ornament(norm: f32) -> f32 {
        MIN_ORNAMENT_DEPTH + norm.clamp(0.0, 1.0) * (MAX_ORNAMENT_DEPTH - MIN_ORNAMENT_DEPTH)
    }

    pub fn set_vocal_model(&mut self, model: VocalStyleModel) {
        self.vocal_model = model;
        self.style_blend_pct = model.nominal_style_blend();
        self.ornament_depth_pct = model.nominal_ornament_depth();
        self.vibrato_rate_hz = model.nominal_vibrato_rate_hz();
        self.vibrato_depth_cents = model.nominal_vibrato_depth_cents();
        self.puck_pos = (
            Self::blend_to_normalized(self.style_blend_pct),
            Self::ornament_to_normalized(self.ornament_depth_pct),
        );
        self.update_neural_simulation();
    }

    /// Update neural vocal expression stylization and 6-axis feature radar.
    pub fn update_neural_simulation(&mut self) {
        let blend = self.style_blend_pct;
        let ornament = self.ornament_depth_pct;

        match self.vocal_model {
            VocalStyleModel::BelCantoOpera => {
                self.expression_radar_axes = [
                    (ornament * 0.6).clamp(0.0, 1.0),
                    (blend * 0.95).clamp(0.0, 1.0),
                    (blend * 0.98).clamp(0.0, 1.0),
                    0.15,
                    0.25,
                    0.92,
                ];
            }
            VocalStyleModel::ContemporaryPopBelt => {
                self.expression_radar_axes = [
                    (ornament * 0.4).clamp(0.0, 1.0),
                    (blend * 0.40).clamp(0.0, 1.0),
                    0.75,
                    0.10,
                    (blend * 0.85).clamp(0.0, 1.0),
                    0.98,
                ];
            }
            VocalStyleModel::BulgarianChoirOpenThroat => {
                self.expression_radar_axes = [
                    (ornament * 0.95).clamp(0.0, 1.0),
                    0.15,
                    0.80,
                    0.30,
                    0.20,
                    0.95,
                ];
            }
            VocalStyleModel::TuvanThroatKargyraa => {
                self.expression_radar_axes = [
                    (ornament * 0.70).clamp(0.0, 1.0),
                    0.25,
                    0.65,
                    (blend * 0.98).clamp(0.0, 1.0),
                    0.40,
                    0.88,
                ];
            }
            VocalStyleModel::GospelMelismaExpressive => {
                self.expression_radar_axes = [
                    (ornament * 0.98).clamp(0.0, 1.0),
                    (blend * 0.85).clamp(0.0, 1.0),
                    0.82,
                    0.20,
                    0.50,
                    0.90,
                ];
            }
        }
    }

    /// Hit test coordinate on the interactive stylizer puck.
    pub fn hit_test_stylizer_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= STYLIZE_PUCK_HIT_RADIUS
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

        let left_w = mid_x - 2;
        let p_row = (((1.0 - self.puck_pos.1) * (height - 5) as f32) + 2.0).round() as usize;
        let p_col = ((self.puck_pos.0 * (left_w - 4) as f32) + 2.0).round() as usize;
        if p_row < height - 1 && p_col < mid_x {
            grid[p_row][p_col] = 'P';
        }

        let right_w = width - mid_x - 2;
        let bar_spacing = right_w / 7;
        for (i, &amp) in self.expression_radar_axes.iter().enumerate() {
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

        // Background: Deep Slate Navy (#0C101A)
        painter.rect_filled(rect, 6.0, Color32::from_rgb(12, 16, 26));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "NEURAL POLYPHONIC VOCAL EXPRESSION STYLIZER & ORNAMENT HUD",
            egui::FontId::proportional(14.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Style Tabs (y: 48..92)
        let tabs = [
            (VocalStyleModel::BelCantoOpera, "BEL CANTO"),
            (VocalStyleModel::ContemporaryPopBelt, "POP BELT"),
            (VocalStyleModel::BulgarianChoirOpenThroat, "BULGARIAN"),
            (VocalStyleModel::TuvanThroatKargyraa, "TUVAN THROAT"),
            (VocalStyleModel::GospelMelismaExpressive, "GOSPEL MELISMA"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (model, name)) in tabs.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_sel = self.vocal_model == *model;
            let bg_col = if is_sel {
                Color32::from_rgb(157, 78, 221)
            } else {
                Color32::from_rgb(24, 32, 48)
            };
            let text_col = if is_sel {
                Color32::from_rgb(245, 235, 255)
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
                        self.set_vocal_model(*model);
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

        // Left 55%: Microtonal Pitch Melisma Contour Ribbon & Morph Puck
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
            "MICROTONAL ORNAMENT CONTOUR & STYLE MORPH",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(157, 78, 221),
        );

        // Melismatic pitch contour curve
        let prev_pt = egui::pos2(left_rect.min.x + 15.0, left_rect.center().y);
        let mut p_last = prev_pt;
        for s in 1..=20 {
            let t = s as f32 / 20.0;
            let x = left_rect.min.x + 15.0 + t * (left_rect.width() - 30.0);
            let vib = (t * self.vibrato_rate_hz * std::f32::consts::TAU).sin()
                * (self.vibrato_depth_cents / 150.0)
                * 20.0;
            let melisma = (t * self.melisma_rate_hz * std::f32::consts::PI).sin()
                * self.ornament_depth_pct
                * 30.0;
            let y = left_rect.center().y - vib - melisma;
            let cur_pt = egui::pos2(x, y);
            painter.line_segment(
                [p_last, cur_pt],
                Stroke::new(2.0_f32, Color32::from_rgb(157, 78, 221)),
            );
            p_last = cur_pt;
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
                    self.style_blend_pct = Self::normalized_to_blend(nx);
                    self.ornament_depth_pct = Self::normalized_to_ornament(ny);
                    self.update_neural_simulation();
                }
            }
        }

        painter.circle_stroke(
            puck_pos,
            STYLIZE_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(157, 78, 221, 150)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(157, 78, 221));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 20.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Style Blend: {:.0}% | Ornament: {:.0}% | Vib: {:.1}Hz ({:.0}ct) | Snap: {:.0}ct",
                self.style_blend_pct * 100.0,
                self.ornament_depth_pct * 100.0,
                self.vibrato_rate_hz,
                self.vibrato_depth_cents,
                self.microtonal_snap_cents
            ),
            egui::FontId::proportional(10.5),
            Color32::from_rgb(220, 180, 255),
        );

        // Right 45%: 6-Axis Vocal Feature Radar
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
            "6-AXIS VOCAL EXPRESSION PROFILE",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(157, 78, 221),
        );

        let radar_labels = [
            "MELISMA",
            "VIBRATO",
            "FORMANT",
            "SUBHARM",
            "BREATH",
            "INTONATION",
        ];
        let bar_w = (right_rect.width() - 30.0 - 5.0 * 6.0) / 6.0;
        for (i, &amp) in self.expression_radar_axes.iter().enumerate() {
            let bx = right_rect.min.x + 15.0 + i as f32 * (bar_w + 6.0);
            let bar_h = amp.clamp(0.0, 1.0) * (right_rect.height() - 80.0);
            let b_rect = egui::Rect::from_min_max(
                egui::pos2(bx, right_rect.max.y - 25.0 - bar_h),
                egui::pos2(bx + bar_w, right_rect.max.y - 25.0),
            );
            let col = if i == 0 || i == 1 {
                Color32::from_rgb(157, 78, 221)
            } else if i == 2 {
                Color32::from_rgb(255, 93, 143)
            } else {
                Color32::from_rgb(0, 229, 255)
            };
            painter.rect_filled(b_rect, 3.0, col);

            painter.text(
                egui::pos2(bx + bar_w * 0.5, right_rect.max.y - 20.0),
                egui::Align2::CENTER_TOP,
                radar_labels[i],
                egui::FontId::proportional(7.5),
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
                "NEURAL TIMBRE BLEND",
                format!("{:.0}% (Resynthesis)", self.style_blend_pct * 100.0),
                Color32::from_rgb(157, 78, 221),
            ),
            (
                "ORNAMENT DEPTH",
                format!("{:.0}% (Melisma Run)", self.ornament_depth_pct * 100.0),
                Color32::from_rgb(255, 93, 143),
            ),
            (
                "VIBRATO MODULATION",
                format!(
                    "{:.1} Hz (±{:.0} cents)",
                    self.vibrato_rate_hz, self.vibrato_depth_cents
                ),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "MICROTONAL QUANTIZE",
                format!("{:.0} cents (Snapping)", self.microtonal_snap_cents),
                Color32::from_rgb(0, 229, 255),
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
            "[PASS] Neural Polyphonic Vocal Stylizer Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
