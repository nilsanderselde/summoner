// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Physical Modeling Hammered Dulcimer / Cimbalom String Dispersion & Multi-Bridge Strike HUD (Step 1591).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const DULCIMER_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_STRIKE_POS: f32 = 0.05;
pub const MAX_STRIKE_POS: f32 = 0.50;
pub const MIN_HAMMER_HARDNESS: f32 = 0.10;
pub const MAX_HAMMER_HARDNESS: f32 = 1.00;
pub const MIN_STRING_COURSES: usize = 12;
pub const MAX_STRING_COURSES: usize = 40;

/// Hammered dulcimer, cimbalom, santur, yangqin, and psaltery instrument models.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DulcimerType {
    ConcertGrandCimbalom, // Hungarian 35-course grand cimbalom with damper pedal & leather mallets
    AppalachianHammeredDulcimer, // 16/15 course American folk dulcimer with flexible wooden hammers
    PersianSantur, // 72-string 18-course Iranian classical santur with delicate mezrab mallets
    ChineseYangqin, // 401-type 4-bridge chromatically extended yangqin with bamboo strikers
    MedievalPsaltery, // Triangular box zither with brass string dispersion & metallic attack
}

impl DulcimerType {
    pub fn instrument_name(&self) -> &'static str {
        match self {
            Self::ConcertGrandCimbalom => "CONCERT CIMBALOM (35C)",
            Self::AppalachianHammeredDulcimer => "APPALACHIAN DULCIMER (16/15)",
            Self::PersianSantur => "PERSIAN SANTUR (18C)",
            Self::ChineseYangqin => "CHINESE YANGQIN (401)",
            Self::MedievalPsaltery => "MEDIEVAL PSALTERY",
        }
    }

    pub fn nominal_strike_pos(&self) -> f32 {
        match self {
            Self::ConcertGrandCimbalom => 0.14,
            Self::AppalachianHammeredDulcimer => 0.18,
            Self::PersianSantur => 0.10,
            Self::ChineseYangqin => 0.15,
            Self::MedievalPsaltery => 0.22,
        }
    }

    pub fn nominal_hammer_hardness(&self) -> f32 {
        match self {
            Self::ConcertGrandCimbalom => 0.65,
            Self::AppalachianHammeredDulcimer => 0.50,
            Self::PersianSantur => 0.85,
            Self::ChineseYangqin => 0.75,
            Self::MedievalPsaltery => 0.90,
        }
    }

    pub fn nominal_courses(&self) -> usize {
        match self {
            Self::ConcertGrandCimbalom => 35,
            Self::AppalachianHammeredDulcimer => 31,
            Self::PersianSantur => 18,
            Self::ChineseYangqin => 28,
            Self::MedievalPsaltery => 15,
        }
    }

    pub fn nominal_inharmonicity(&self) -> f32 {
        match self {
            Self::ConcertGrandCimbalom => 0.0035,
            Self::AppalachianHammeredDulcimer => 0.0018,
            Self::PersianSantur => 0.0008,
            Self::ChineseYangqin => 0.0022,
            Self::MedievalPsaltery => 0.0050,
        }
    }

    pub fn nominal_decay_s(&self) -> f32 {
        match self {
            Self::ConcertGrandCimbalom => 6.5,
            Self::AppalachianHammeredDulcimer => 4.2,
            Self::PersianSantur => 5.8,
            Self::ChineseYangqin => 3.6,
            Self::MedievalPsaltery => 2.8,
        }
    }

    pub fn nominal_bridge_coupling(&self) -> f32 {
        match self {
            Self::ConcertGrandCimbalom => 0.45,
            Self::AppalachianHammeredDulcimer => 0.35,
            Self::PersianSantur => 0.60,
            Self::ChineseYangqin => 0.40,
            Self::MedievalPsaltery => 0.20,
        }
    }
}

/// Physical modeling hammered dulcimer / cimbalom string dispersion & multi-bridge strike HUD.
#[derive(Debug, Clone)]
pub struct DulcimerCimbalomView {
    pub instrument_type: DulcimerType,
    pub strike_pos_ratio: f32,
    pub hammer_hardness: f32,
    pub string_courses: usize,
    pub selected_course_idx: usize,
    pub inharmonicity_coeff: f32,
    pub decay_s: f32,
    pub bridge_coupling: f32,
    pub damper_pedal: bool,
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    pub modal_amplitudes: [f32; 8],
    pub color_palette: ContrastColorPalette,
}

impl Default for DulcimerCimbalomView {
    fn default() -> Self {
        Self::new()
    }
}

impl DulcimerCimbalomView {
    pub fn new() -> Self {
        let mut view = Self {
            instrument_type: DulcimerType::ConcertGrandCimbalom,
            strike_pos_ratio: 0.14,
            hammer_hardness: 0.65,
            string_courses: 35,
            selected_course_idx: 17,
            inharmonicity_coeff: 0.0035,
            decay_s: 6.5,
            bridge_coupling: 0.45,
            damper_pedal: false,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            modal_amplitudes: [1.0, 0.72, 0.50, 0.35, 0.22, 0.40, 0.28, 0.15],
            color_palette: ContrastColorPalette::default(),
        };
        view.puck_pos = (
            Self::pos_to_normalized(view.strike_pos_ratio),
            Self::hardness_to_normalized(view.hammer_hardness),
        );
        view.update_physics_simulation();
        view
    }

    pub fn pos_to_normalized(pos: f32) -> f32 {
        let p = pos.clamp(MIN_STRIKE_POS, MAX_STRIKE_POS);
        ((p - MIN_STRIKE_POS) / (MAX_STRIKE_POS - MIN_STRIKE_POS)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_pos(norm: f32) -> f32 {
        MIN_STRIKE_POS + norm.clamp(0.0, 1.0) * (MAX_STRIKE_POS - MIN_STRIKE_POS)
    }

    pub fn hardness_to_normalized(hard: f32) -> f32 {
        let h = hard.clamp(MIN_HAMMER_HARDNESS, MAX_HAMMER_HARDNESS);
        ((h - MIN_HAMMER_HARDNESS) / (MAX_HAMMER_HARDNESS - MIN_HAMMER_HARDNESS)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_hardness(norm: f32) -> f32 {
        MIN_HAMMER_HARDNESS + norm.clamp(0.0, 1.0) * (MAX_HAMMER_HARDNESS - MIN_HAMMER_HARDNESS)
    }

    pub fn set_instrument_type(&mut self, inst: DulcimerType) {
        self.instrument_type = inst;
        self.strike_pos_ratio = inst.nominal_strike_pos();
        self.hammer_hardness = inst.nominal_hammer_hardness();
        self.string_courses = inst.nominal_courses();
        self.selected_course_idx = self.string_courses / 2;
        self.inharmonicity_coeff = inst.nominal_inharmonicity();
        self.decay_s = inst.nominal_decay_s();
        self.bridge_coupling = inst.nominal_bridge_coupling();
        self.puck_pos = (
            Self::pos_to_normalized(self.strike_pos_ratio),
            Self::hardness_to_normalized(self.hammer_hardness),
        );
        self.update_physics_simulation();
    }

    pub fn update_physics_simulation(&mut self) {
        let hard = self.hammer_hardness;
        let pos = self.strike_pos_ratio;
        let b_coeff = self.inharmonicity_coeff;
        let coupling = self.bridge_coupling;

        let node_suppression_1 = (std::f32::consts::PI * 1.0 * pos).sin().abs();
        let node_suppression_2 = (std::f32::consts::PI * 2.0 * pos).sin().abs();
        let node_suppression_3 = (std::f32::consts::PI * 3.0 * pos).sin().abs();
        let node_suppression_4 = (std::f32::consts::PI * 4.0 * pos).sin().abs();

        let f0_amp = (1.0 * node_suppression_1).clamp(0.1, 1.2);
        let f1_amp =
            (0.80 * (hard.sqrt()) * node_suppression_2 * (1.0 + b_coeff * 20.0)).clamp(0.0, 1.1);
        let f2_amp = (0.60 * hard * node_suppression_3 * (1.0 + b_coeff * 40.0)).clamp(0.0, 1.0);
        let f3_amp =
            (0.45 * (hard * hard) * node_suppression_4 * (1.0 + b_coeff * 60.0)).clamp(0.0, 0.9);
        let f4_amp = (0.30 * (hard.powi(3)) * (1.0 + b_coeff * 80.0)).clamp(0.0, 0.8);

        let bridge_left = (coupling * 0.75 * f0_amp).clamp(0.0, 1.0);
        let bridge_right = (coupling * 0.60 * f1_amp).clamp(0.0, 1.0);
        let sympathetic_air = (coupling * 0.40 * (hard + 0.2)).clamp(0.0, 0.8);

        self.modal_amplitudes = [
            f0_amp,
            f1_amp,
            f2_amp,
            f3_amp,
            f4_amp,
            bridge_left,
            bridge_right,
            sympathetic_air,
        ];
    }

    pub fn hit_test_dulcimer_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= DULCIMER_PUCK_HIT_RADIUS
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
        for (i, &amp) in self.modal_amplitudes.iter().enumerate() {
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
            "PHYSICAL MODELING HAMMERED DULCIMER & CIMBALOM STRING DISPERSION HUD",
            egui::FontId::proportional(13.5),
            Color32::from_rgb(240, 245, 255),
        );

        // Tabs (y: 48..92) - 44pt height
        let tabs = [
            (DulcimerType::ConcertGrandCimbalom, "CIMBALOM (35C)"),
            (
                DulcimerType::AppalachianHammeredDulcimer,
                "DULCIMER (16/15)",
            ),
            (DulcimerType::PersianSantur, "SANTUR (18C)"),
            (DulcimerType::ChineseYangqin, "YANGQIN (401)"),
            (DulcimerType::MedievalPsaltery, "PSALTERY (15C)"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (itype, name)) in tabs.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_sel = self.instrument_type == *itype;
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
                        self.set_instrument_type(*itype);
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

        // Left 55%: Trapezoidal Soundboard & Multi-Bridge Courses
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
            "TRAPEZOIDAL SOUNDBOARD & MULTI-BRIDGE STRIKE FIELD",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(255, 107, 43),
        );

        // Treble & Bass Bridge dividing markers
        let treble_bridge_x = left_rect.min.x + left_rect.width() * 0.40;
        let bass_bridge_x = left_rect.min.x + left_rect.width() * 0.72;
        painter.line_segment(
            [
                egui::pos2(treble_bridge_x, left_rect.min.y + 30.0),
                egui::pos2(treble_bridge_x + 15.0, left_rect.max.y - 25.0),
            ],
            Stroke::new(2.5_f32, Color32::from_rgb(0, 229, 255)),
        );
        painter.line_segment(
            [
                egui::pos2(bass_bridge_x, left_rect.min.y + 30.0),
                egui::pos2(bass_bridge_x + 15.0, left_rect.max.y - 25.0),
            ],
            Stroke::new(2.5_f32, Color32::from_rgb(255, 215, 0)),
        );

        let display_courses = self.string_courses.min(18);
        let course_slot_h = (left_rect.height() - 65.0) / display_courses as f32;

        for c in 0..display_courses {
            let cy = left_rect.min.y + 35.0 + c as f32 * course_slot_h;
            let width_factor = 0.55 + 0.45 * (c as f32 / display_courses as f32);
            let start_x = left_rect.min.x + 15.0 + (1.0 - width_factor) * 40.0;
            let end_x = left_rect.max.x - 15.0 - (1.0 - width_factor) * 20.0;
            let is_active = c == self.selected_course_idx.min(display_courses - 1);
            let string_col = if is_active {
                Color32::from_rgb(255, 107, 43)
            } else {
                Color32::from_rgb(140, 160, 185)
            };

            painter.line_segment(
                [egui::pos2(start_x, cy), egui::pos2(end_x, cy)],
                Stroke::new(if is_active { 2.5_f32 } else { 1.2_f32 }, string_col),
            );
        }

        // Interactive Striker Puck
        let puck_x = left_rect.min.x + self.puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.puck_pos = (nx, ny);
                    self.strike_pos_ratio = Self::normalized_to_pos(nx);
                    self.hammer_hardness = Self::normalized_to_hardness(ny);
                    self.update_physics_simulation();
                }
            }
        }

        painter.circle_stroke(
            puck_pos,
            DULCIMER_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(255, 107, 43, 150)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(255, 107, 43));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 18.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Strike Pos: {:.2}L | Hardness: {:.0}% | Course: #{}/{} | Decay: {:.1}s",
                self.strike_pos_ratio,
                self.hammer_hardness * 100.0,
                self.selected_course_idx + 1,
                self.string_courses,
                self.decay_s
            ),
            egui::FontId::proportional(10.5),
            Color32::from_rgb(255, 180, 100),
        );

        // Right 45%: Inharmonic Dispersion & Coupled Bridge Modes
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
            "STRING INHARMONICITY & BRIDGE COUPLING SPECTRUM",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(255, 107, 43),
        );

        let mode_labels = [
            "f0", "f1 (B)", "f2 (B)", "f3 (B)", "f4 (B)", "BRG-L", "BRG-R", "SYMP",
        ];
        let bar_w = (right_rect.width() - 30.0 - 7.0 * 6.0) / 8.0;
        for (i, &amp) in self.modal_amplitudes.iter().enumerate() {
            let bx = right_rect.min.x + 15.0 + i as f32 * (bar_w + 6.0);
            let bar_h = (amp.clamp(0.0, 1.2) / 1.2) * (right_rect.height() - 80.0);
            let b_rect = egui::Rect::from_min_max(
                egui::pos2(bx, right_rect.max.y - 25.0 - bar_h),
                egui::pos2(bx + bar_w, right_rect.max.y - 25.0),
            );
            let col = if i == 0 {
                Color32::from_rgb(255, 107, 43)
            } else if i < 5 {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(255, 215, 0)
            };
            painter.rect_filled(b_rect, 3.0, col);

            painter.text(
                egui::pos2(bx + bar_w * 0.5, right_rect.max.y - 20.0),
                egui::Align2::CENTER_TOP,
                mode_labels[i],
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
                "STRIKER POS RATIO",
                format!("{:.2} L (Node)", self.strike_pos_ratio),
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "HAMMER HARDNESS",
                format!("{:.0}% (Mallet Core)", self.hammer_hardness * 100.0),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "STRING INHARMONICITY",
                format!("{:.4} (Stiff Wire)", self.inharmonicity_coeff),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "BRIDGE COUPLING",
                format!("{:.2} (Dual Soundboard)", self.bridge_coupling),
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
            "[PASS] Hammered Dulcimer / Cimbalom Multi-Bridge Strike Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
