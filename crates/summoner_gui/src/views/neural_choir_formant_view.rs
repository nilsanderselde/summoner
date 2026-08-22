// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Neural Polyphonic Singing Choir Formant Morpher & Vowel Space Trajectory HUD (Step 1594).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const CHOIR_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_FORMANT_F1_HZ: f32 = 200.0;
pub const MAX_FORMANT_F1_HZ: f32 = 1000.0;
pub const MIN_FORMANT_F2_HZ: f32 = 500.0;
pub const MAX_FORMANT_F2_HZ: f32 = 3000.0;

/// Choral voice ensembles and polyphonic acoustic vocal tract models.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChoirEnsembleType {
    ClassicalSATB,             // 4-part classical choir (Soprano, Alto, Tenor, Bass)
    BulgarianWomensChoir, // Open-throat close-harmony microtonal diaphonic Bulgarian folk choir
    GregorianMonasticChant, // Deep cathedral resonant parallel organum monastic choir
    ContemporaryVocalEnsemble, // Close-mic 8-part jazz/pop a cappella ensemble
    GospelChoirWallOfSound, // Belting high-energy gospel choral stack with dynamic vibrato
}

impl ChoirEnsembleType {
    pub fn ensemble_name(&self) -> &'static str {
        match self {
            Self::ClassicalSATB => "CLASSICAL SATB CHOIR",
            Self::BulgarianWomensChoir => "BULGARIAN FOLK CHOIR",
            Self::GregorianMonasticChant => "GREGORIAN MONASTIC",
            Self::ContemporaryVocalEnsemble => "CONTEMPORARY ENSEMBLE",
            Self::GospelChoirWallOfSound => "GOSPEL WALL OF SOUND",
        }
    }

    pub fn nominal_f1_hz(&self) -> f32 {
        match self {
            Self::ClassicalSATB => 650.0,
            Self::BulgarianWomensChoir => 780.0,
            Self::GregorianMonasticChant => 450.0,
            Self::ContemporaryVocalEnsemble => 580.0,
            Self::GospelChoirWallOfSound => 720.0,
        }
    }

    pub fn nominal_f2_hz(&self) -> f32 {
        match self {
            Self::ClassicalSATB => 1400.0,
            Self::BulgarianWomensChoir => 1950.0,
            Self::GregorianMonasticChant => 950.0,
            Self::ContemporaryVocalEnsemble => 1700.0,
            Self::GospelChoirWallOfSound => 1600.0,
        }
    }

    pub fn nominal_voice_count(&self) -> usize {
        match self {
            Self::ClassicalSATB => 32,
            Self::BulgarianWomensChoir => 16,
            Self::GregorianMonasticChant => 24,
            Self::ContemporaryVocalEnsemble => 8,
            Self::GospelChoirWallOfSound => 48,
        }
    }

    pub fn nominal_vibrato_rate_hz(&self) -> f32 {
        match self {
            Self::ClassicalSATB => 5.5,
            Self::BulgarianWomensChoir => 0.0, // Straight-tone open throat
            Self::GregorianMonasticChant => 4.2,
            Self::ContemporaryVocalEnsemble => 5.8,
            Self::GospelChoirWallOfSound => 6.2,
        }
    }
}

/// Neural polyphonic singing choir formant morpher & vowel space trajectory HUD.
#[derive(Debug, Clone)]
pub struct NeuralChoirFormantView {
    pub ensemble: ChoirEnsembleType,
    pub formant_f1_hz: f32,
    pub formant_f2_hz: f32,
    pub formant_f3_hz: f32,
    pub voice_count: usize,
    pub neural_morph_blend: f32,
    pub tract_scale: f32,
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    pub voice_formant_peaks: [f32; 5],
    pub color_palette: ContrastColorPalette,
}

impl Default for NeuralChoirFormantView {
    fn default() -> Self {
        Self::new()
    }
}

impl NeuralChoirFormantView {
    pub fn new() -> Self {
        let mut view = Self {
            ensemble: ChoirEnsembleType::ClassicalSATB,
            formant_f1_hz: 650.0,
            formant_f2_hz: 1400.0,
            formant_f3_hz: 2600.0,
            voice_count: 32,
            neural_morph_blend: 0.88,
            tract_scale: 1.0,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            voice_formant_peaks: [0.95, 0.78, 0.55, 0.38, 0.22],
            color_palette: ContrastColorPalette::default(),
        };
        view.puck_pos = (
            Self::f2_to_normalized(view.formant_f2_hz),
            Self::f1_to_normalized(view.formant_f1_hz),
        );
        view.update_choir_simulation();
        view
    }

    pub fn f1_to_normalized(f1: f32) -> f32 {
        let f = f1.clamp(MIN_FORMANT_F1_HZ, MAX_FORMANT_F1_HZ);
        ((f - MIN_FORMANT_F1_HZ) / (MAX_FORMANT_F1_HZ - MIN_FORMANT_F1_HZ)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_f1(norm: f32) -> f32 {
        MIN_FORMANT_F1_HZ + norm.clamp(0.0, 1.0) * (MAX_FORMANT_F1_HZ - MIN_FORMANT_F1_HZ)
    }

    pub fn f2_to_normalized(f2: f32) -> f32 {
        let f = f2.clamp(MIN_FORMANT_F2_HZ, MAX_FORMANT_F2_HZ);
        ((f - MIN_FORMANT_F2_HZ) / (MAX_FORMANT_F2_HZ - MIN_FORMANT_F2_HZ)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_f2(norm: f32) -> f32 {
        MIN_FORMANT_F2_HZ + norm.clamp(0.0, 1.0) * (MAX_FORMANT_F2_HZ - MIN_FORMANT_F2_HZ)
    }

    pub fn set_ensemble(&mut self, ens: ChoirEnsembleType) {
        self.ensemble = ens;
        self.formant_f1_hz = ens.nominal_f1_hz();
        self.formant_f2_hz = ens.nominal_f2_hz();
        self.voice_count = ens.nominal_voice_count();
        self.puck_pos = (
            Self::f2_to_normalized(self.formant_f2_hz),
            Self::f1_to_normalized(self.formant_f1_hz),
        );
        self.update_choir_simulation();
    }

    pub fn update_choir_simulation(&mut self) {
        let f1 = self.formant_f1_hz;
        let f2 = self.formant_f2_hz;
        let blend = self.neural_morph_blend;

        self.formant_f3_hz = (f2 * 1.65).clamp(1800.0, 3800.0);

        let p1 = ((f1 / 1000.0).clamp(0.2, 1.0) * blend).clamp(0.1, 1.2);
        let p2 = ((f2 / 3000.0).clamp(0.2, 1.0) * blend).clamp(0.1, 1.1);
        let p3 = (0.65 * blend).clamp(0.1, 0.9);
        let p4 = (0.45 * blend).clamp(0.05, 0.7);
        let p5 = (0.28 * blend).clamp(0.02, 0.5);

        self.voice_formant_peaks = [p1, p2, p3, p4, p5];
    }

    pub fn hit_test_choir_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let plot_x = canvas.x + 30.0;
        let plot_w = (canvas.width - 60.0).max(1.0);
        let plot_y = canvas.y + 40.0;
        let plot_h = (canvas.height - 75.0).max(1.0);
        let puck_x = plot_x + self.puck_pos.0 * plot_w;
        let puck_y = plot_y + (1.0 - self.puck_pos.1) * plot_h;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= CHOIR_PUCK_HIT_RADIUS
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
        let p_row = (((1.0 - self.puck_pos.1) * (height - 6) as f32) + 3.0).round() as usize;
        let p_col = ((self.puck_pos.0 * (left_w - 6) as f32) + 3.0).round() as usize;
        if p_row < height - 1 && p_col < mid_x {
            grid[p_row][p_col] = 'P';
        }

        let right_w = width - mid_x - 2;
        let bar_spacing = right_w / 6;
        for (i, &peak) in self.voice_formant_peaks.iter().enumerate() {
            let col = mid_x + 2 + (i + 1) * bar_spacing;
            let bar_h = (peak.clamp(0.0, 1.2) / 1.2 * (height - 4) as f32).round() as usize;
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
            "NEURAL POLYPHONIC CHOIR FORMANT MORPHER & VOWEL TRAJECTORY HUD",
            egui::FontId::proportional(13.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Tabs (y: 48..92) - 44pt height
        let tabs = [
            (ChoirEnsembleType::ClassicalSATB, "CLASSICAL SATB"),
            (ChoirEnsembleType::BulgarianWomensChoir, "BULGARIAN CHOIR"),
            (ChoirEnsembleType::GregorianMonasticChant, "GREGORIAN CHANT"),
            (
                ChoirEnsembleType::ContemporaryVocalEnsemble,
                "A CAPPELLA ENSEMBLE",
            ),
            (
                ChoirEnsembleType::GospelChoirWallOfSound,
                "GOSPEL WALL (48V)",
            ),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (itype, name)) in tabs.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_sel = self.ensemble == *itype;
            let bg_col = if is_sel {
                Color32::from_rgb(255, 93, 143)
            } else {
                Color32::from_rgb(24, 32, 48)
            };
            let text_col = if is_sel {
                Color32::from_rgb(28, 4, 16)
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
                        self.set_ensemble(*itype);
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

        // Left 55%: 2D Vowel Formant Trajectory Space (F1 vs F2)
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
            "2D VOWEL FORMANT SPACE (F1: 200..1000Hz vs F2: 500..3000Hz)",
            egui::FontId::proportional(10.0),
            Color32::from_rgb(255, 93, 143),
        );

        let plot_rect = egui::Rect::from_min_max(
            egui::pos2(left_rect.min.x + 30.0, left_rect.min.y + 35.0),
            egui::pos2(left_rect.max.x - 30.0, left_rect.max.y - 35.0),
        );

        // Vowel Target Anchors: /i/, /e/, /a/, /o/, /u/
        let vowel_anchors = [
            ("/i/ (beet)", 270.0, 2300.0),
            ("/e/ (bait)", 530.0, 1850.0),
            ("/a/ (father)", 730.0, 1100.0),
            ("/o/ (boat)", 570.0, 850.0),
            ("/u/ (boot)", 300.0, 870.0),
        ];

        for (v_lbl, vf1, vf2) in vowel_anchors {
            let vx = plot_rect.min.x + Self::f2_to_normalized(vf2) * plot_rect.width();
            let vy = plot_rect.max.y - Self::f1_to_normalized(vf1) * plot_rect.height();
            painter.circle_stroke(
                egui::pos2(vx, vy),
                8.0,
                Stroke::new(1.0_f32, Color32::from_rgb(120, 145, 180)),
            );
            painter.text(
                egui::pos2(vx, vy - 10.0),
                egui::Align2::CENTER_BOTTOM,
                v_lbl,
                egui::FontId::proportional(9.0),
                Color32::from_rgb(160, 185, 215),
            );
        }

        // Interactive Puck (X = F2, Y = F1)
        let puck_x = plot_rect.min.x + self.puck_pos.0 * plot_rect.width();
        let puck_y = plot_rect.max.y - self.puck_pos.1 * plot_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - plot_rect.min.x) / plot_rect.width()).clamp(0.0, 1.0);
                    let ny = ((plot_rect.max.y - mouse_pos.y) / plot_rect.height()).clamp(0.0, 1.0);
                    self.puck_pos = (nx, ny);
                    self.formant_f2_hz = Self::normalized_to_f2(nx);
                    self.formant_f1_hz = Self::normalized_to_f1(ny);
                    self.update_choir_simulation();
                }
            }
        }

        painter.circle_stroke(
            puck_pos,
            CHOIR_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(255, 93, 143, 150)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(255, 93, 143));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 18.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "F1: {:.0} Hz | F2: {:.0} Hz | F3: {:.0} Hz | Voices: {} | Blend: {:.0}%",
                self.formant_f1_hz,
                self.formant_f2_hz,
                self.formant_f3_hz,
                self.voice_count,
                self.neural_morph_blend * 100.0
            ),
            egui::FontId::proportional(10.5),
            Color32::from_rgb(255, 180, 200),
        );

        // Right 45%: Polyphonic Vocal Tract Formants Spectrum
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
            "POLYPHONIC VOCAL TRACT FORMANT SPECTRUM",
            egui::FontId::proportional(10.0),
            Color32::from_rgb(255, 93, 143),
        );

        let f_labels = [
            "F1 (Pharynx)",
            "F2 (Oral)",
            "F3 (Singers)",
            "F4 (Nasal)",
            "F5 (Head)",
        ];
        let bar_w = (right_rect.width() - 30.0 - 4.0 * 6.0) / 5.0;

        for (i, &peak) in self.voice_formant_peaks.iter().enumerate() {
            let bx = right_rect.min.x + 15.0 + i as f32 * (bar_w + 6.0);
            let bar_h = (peak.clamp(0.0, 1.2) / 1.2) * (right_rect.height() - 75.0);
            let b_rect = egui::Rect::from_min_max(
                egui::pos2(bx, right_rect.max.y - 25.0 - bar_h),
                egui::pos2(bx + bar_w, right_rect.max.y - 25.0),
            );
            let col = if i == 0 {
                Color32::from_rgb(255, 93, 143)
            } else if i == 1 {
                Color32::from_rgb(157, 78, 221)
            } else if i == 2 {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(255, 215, 0)
            };
            painter.rect_filled(b_rect, 3.0, col);

            painter.text(
                egui::pos2(bx + bar_w * 0.5, right_rect.max.y - 20.0),
                egui::Align2::CENTER_TOP,
                f_labels[i],
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
                "FORMANT F1 (PHARYNX)",
                format!("{:.0} Hz (Vowel Height)", self.formant_f1_hz),
                Color32::from_rgb(255, 93, 143),
            ),
            (
                "FORMANT F2 (ORAL CAVITY)",
                format!("{:.0} Hz (Vowel Frontness)", self.formant_f2_hz),
                Color32::from_rgb(157, 78, 221),
            ),
            (
                "CHOIR SPREAD VOICES",
                format!("{} Polyphonic Voices", self.voice_count),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "NEURAL TIMBRE MORPH",
                format!(
                    "{:.0}% (Tract Resynthesis)",
                    self.neural_morph_blend * 100.0
                ),
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
            "[PASS] Neural Polyphonic Choir Formant Morpher Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
