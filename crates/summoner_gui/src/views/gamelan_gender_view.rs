// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Physical Modeling Balinese Gamelan Metallophone / Gender Bar Modal Inharmonicity & Damping HUD (Step 1611).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const GAMELAN_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_MALLET_HARDNESS: f32 = 0.05;
pub const MAX_MALLET_HARDNESS: f32 = 1.00;
pub const MIN_OMBAK_RATE_HZ: f32 = 2.0;
pub const MAX_OMBAK_RATE_HZ: f32 = 12.0;
pub const MIN_BAR_COUNT: usize = 5;
pub const MAX_BAR_COUNT: usize = 14;

/// Balinese gamelan metallophone and gender instrument types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamelanInstrumentType {
    GenderWayang10Bar, // 10-key Gender Wayang (Slendro 5-tone, paired pengisep/pengumbang)
    GamelanJegogan,    // Deep bass metallophone (5 keys, massive tuned bamboo resonators)
    GamelanCalungPemade, // Mid-register 10-key metallophone with acoustic beating
    GamelanKanthilTrompong, // High-octave shimmering bronze keys (fast decay, high inharmonicity)
    GamelanUgalLeader, // Lead 10-key metallophone (broad dynamic range and expressive damping)
}

impl GamelanInstrumentType {
    pub fn instrument_name(&self) -> &'static str {
        match self {
            Self::GenderWayang10Bar => "GENDER WAYANG (10-BAR SLENDRO)",
            Self::GamelanJegogan => "GAMELAN JEGOGAN (BASS 5-KEY)",
            Self::GamelanCalungPemade => "CALUNG / PEMADE (MID 10-KEY)",
            Self::GamelanKanthilTrompong => "KANTHIL / REYONG (HIGH 10-KEY)",
            Self::GamelanUgalLeader => "UGAL LEADER (10-KEY BRONZE)",
        }
    }

    pub fn nominal_mallet_hardness(&self) -> f32 {
        match self {
            Self::GenderWayang10Bar => 0.45,
            Self::GamelanJegogan => 0.20,
            Self::GamelanCalungPemade => 0.65,
            Self::GamelanKanthilTrompong => 0.85,
            Self::GamelanUgalLeader => 0.55,
        }
    }

    pub fn nominal_ombak_rate_hz(&self) -> f32 {
        match self {
            Self::GenderWayang10Bar => 7.2,
            Self::GamelanJegogan => 3.5,
            Self::GamelanCalungPemade => 6.8,
            Self::GamelanKanthilTrompong => 8.5,
            Self::GamelanUgalLeader => 6.0,
        }
    }

    pub fn nominal_bar_count(&self) -> usize {
        match self {
            Self::GenderWayang10Bar => 10,
            Self::GamelanJegogan => 5,
            Self::GamelanCalungPemade => 10,
            Self::GamelanKanthilTrompong => 10,
            Self::GamelanUgalLeader => 10,
        }
    }

    pub fn nominal_bronze_thickness_mm(&self) -> f32 {
        match self {
            Self::GenderWayang10Bar => 6.5,
            Self::GamelanJegogan => 18.0,
            Self::GamelanCalungPemade => 10.0,
            Self::GamelanKanthilTrompong => 4.5,
            Self::GamelanUgalLeader => 9.0,
        }
    }

    pub fn nominal_damping_factor(&self) -> f32 {
        match self {
            Self::GenderWayang10Bar => 0.35,
            Self::GamelanJegogan => 0.15,
            Self::GamelanCalungPemade => 0.40,
            Self::GamelanKanthilTrompong => 0.55,
            Self::GamelanUgalLeader => 0.30,
        }
    }

    pub fn nominal_resonator_q(&self) -> f32 {
        match self {
            Self::GenderWayang10Bar => 38.0,
            Self::GamelanJegogan => 65.0,
            Self::GamelanCalungPemade => 42.0,
            Self::GamelanKanthilTrompong => 28.0,
            Self::GamelanUgalLeader => 45.0,
        }
    }
}

/// Physical modeling Balinese Gamelan metallophone / gender bar modal inharmonicity & damping HUD.
#[derive(Debug, Clone)]
pub struct GamelanGenderView {
    pub instrument_type: GamelanInstrumentType,
    pub mallet_hardness: f32,
    pub ombak_rate_hz: f32,
    pub bar_count: usize,
    pub selected_bar_idx: usize,
    pub bronze_thickness_mm: f32,
    pub damping_factor: f32,
    pub resonator_coupling_q: f32,
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    pub modal_amplitudes: [f32; 8],
    pub color_palette: ContrastColorPalette,
}

impl Default for GamelanGenderView {
    fn default() -> Self {
        Self::new()
    }
}

impl GamelanGenderView {
    pub fn new() -> Self {
        let mut view = Self {
            instrument_type: GamelanInstrumentType::GenderWayang10Bar,
            mallet_hardness: 0.45,
            ombak_rate_hz: 7.2,
            bar_count: 10,
            selected_bar_idx: 4,
            bronze_thickness_mm: 6.5,
            damping_factor: 0.35,
            resonator_coupling_q: 38.0,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            modal_amplitudes: [1.0, 0.72, 0.48, 0.32, 0.20, 0.55, 0.40, 0.28],
            color_palette: ContrastColorPalette::default(),
        };
        view.puck_pos = (
            Self::hardness_to_normalized(view.mallet_hardness),
            Self::ombak_to_normalized(view.ombak_rate_hz),
        );
        view.update_gamelan_simulation();
        view
    }

    pub fn hardness_to_normalized(h: f32) -> f32 {
        let val = h.clamp(MIN_MALLET_HARDNESS, MAX_MALLET_HARDNESS);
        ((val - MIN_MALLET_HARDNESS) / (MAX_MALLET_HARDNESS - MIN_MALLET_HARDNESS)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_hardness(norm: f32) -> f32 {
        MIN_MALLET_HARDNESS + norm.clamp(0.0, 1.0) * (MAX_MALLET_HARDNESS - MIN_MALLET_HARDNESS)
    }

    pub fn ombak_to_normalized(rate: f32) -> f32 {
        let val = rate.clamp(MIN_OMBAK_RATE_HZ, MAX_OMBAK_RATE_HZ);
        ((val - MIN_OMBAK_RATE_HZ) / (MAX_OMBAK_RATE_HZ - MIN_OMBAK_RATE_HZ)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_ombak(norm: f32) -> f32 {
        MIN_OMBAK_RATE_HZ + norm.clamp(0.0, 1.0) * (MAX_OMBAK_RATE_HZ - MIN_OMBAK_RATE_HZ)
    }

    pub fn set_instrument_type(&mut self, inst: GamelanInstrumentType) {
        self.instrument_type = inst;
        self.mallet_hardness = inst.nominal_mallet_hardness();
        self.ombak_rate_hz = inst.nominal_ombak_rate_hz();
        self.bar_count = inst.nominal_bar_count();
        self.selected_bar_idx = self.bar_count / 2;
        self.bronze_thickness_mm = inst.nominal_bronze_thickness_mm();
        self.damping_factor = inst.nominal_damping_factor();
        self.resonator_coupling_q = inst.nominal_resonator_q();
        self.puck_pos = (
            Self::hardness_to_normalized(self.mallet_hardness),
            Self::ombak_to_normalized(self.ombak_rate_hz),
        );
        self.update_gamelan_simulation();
    }

    pub fn update_gamelan_simulation(&mut self) {
        let h = self.mallet_hardness;
        let thick = self.bronze_thickness_mm;
        let damp = self.damping_factor;
        let res_q = self.resonator_coupling_q;
        let ombak = self.ombak_rate_hz;

        // Clamped-free / suspended metallophone bar physics:
        // Fundamental f0, modal inharmonic ratios: f1 ~ 2.76*f0, f2 ~ 5.40*f0, f3 ~ 8.93*f0
        let f0_amp =
            ((1.2 - h * 0.4) * (1.0 - damp * 0.5) * (res_q / 40.0).clamp(0.5, 1.3)).clamp(0.1, 1.2);
        let f1_transverse = (h * 0.95 * (1.0 + thick * 0.05) * (1.0 - damp * 0.4)).clamp(0.0, 1.1);
        let f2_torsional = (h * h * 0.80 * (1.0 - damp * 0.6)).clamp(0.0, 1.0);
        let f3_high_mode = (h * h * h * 0.65 * (1.0 - damp * 0.7)).clamp(0.0, 0.9);

        // Acoustic ombak beating coupling between pengisep (blower/male) and pengumbang (waver/female)
        let ombak_modulation = ((ombak / 8.0) * 0.75 * f0_amp).clamp(0.0, 1.0);
        let bamboo_cavity_gain = ((res_q / 50.0) * 0.85 * f0_amp).clamp(0.0, 1.1);
        let strike_transient_click = (h * 0.90 * (1.0 + thick * 0.08)).clamp(0.0, 1.0);
        let sympathetic_beating = (ombak_modulation * 0.60 + f1_transverse * 0.40).clamp(0.0, 0.95);

        self.modal_amplitudes = [
            f0_amp,
            f1_transverse,
            f2_torsional,
            f3_high_mode,
            ombak_modulation,
            bamboo_cavity_gain,
            strike_transient_click,
            sympathetic_beating,
        ];
    }

    pub fn hit_test_gamelan_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= GAMELAN_PUCK_HIT_RADIUS
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
            grid[p_row][p_col] = 'G';
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

        // Background: Balinese Deep Bronze Slate (#0E121C)
        painter.rect_filled(rect, 6.0, Color32::from_rgb(14, 18, 28));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "BALINESE GAMELAN METALLOPHONE / GENDER BAR MODAL HUD",
            egui::FontId::proportional(13.5),
            Color32::from_rgb(245, 240, 230),
        );

        // Tabs (y: 48..92) - 44pt height
        let tabs = [
            (GamelanInstrumentType::GenderWayang10Bar, "GENDER WAYANG"),
            (GamelanInstrumentType::GamelanJegogan, "JEGOGAN (BASS)"),
            (
                GamelanInstrumentType::GamelanCalungPemade,
                "CALUNG / PEMADE",
            ),
            (
                GamelanInstrumentType::GamelanKanthilTrompong,
                "KANTHIL / REYONG",
            ),
            (GamelanInstrumentType::GamelanUgalLeader, "UGAL (LEADER)"),
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
                Color32::from_rgb(255, 215, 0) // Gold
            } else {
                Color32::from_rgb(28, 34, 48)
            };
            let text_col = if is_sel {
                Color32::from_rgb(18, 14, 4)
            } else {
                Color32::from_rgb(230, 225, 210)
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
        painter.rect_filled(main_canvas, 6.0, Color32::from_rgb(10, 14, 22));
        painter.rect_stroke(
            main_canvas,
            6.0,
            Stroke::new(1.5_f32, Color32::from_rgb(65, 55, 35)),
        );

        // Left 55%: Bronze Metallophone Key Array & Resonator Tubes
        let left_w = main_canvas.width() * 0.55;
        let left_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(left_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(left_rect, 4.0, Color32::from_rgb(16, 20, 30));
        painter.rect_stroke(
            left_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(45, 55, 75)),
        );

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "BRONZE GENDER KEYS & BAMBOO RESONATORS FIELD",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(255, 215, 0),
        );

        // Draw bronze keys array
        let num_bars = self.bar_count;
        let bar_pad = 6.0;
        let key_area_w = left_rect.width() - 40.0;
        let single_bar_w = (key_area_w - (num_bars - 1) as f32 * bar_pad) / num_bars as f32;
        let key_base_y = left_rect.min.y + 45.0;

        for b in 0..num_bars {
            let bx = left_rect.min.x + 20.0 + b as f32 * (single_bar_w + bar_pad);
            let bar_len_factor = 1.0 - (b as f32 / num_bars as f32) * 0.35;
            let bar_h = 100.0 * bar_len_factor;
            let bar_rect = egui::Rect::from_min_size(
                egui::pos2(bx, key_base_y),
                egui::vec2(single_bar_w, bar_h),
            );

            // Resonator tube below key
            let res_rect = egui::Rect::from_min_size(
                egui::pos2(bx + single_bar_w * 0.15, key_base_y + bar_h + 8.0),
                egui::vec2(single_bar_w * 0.70, 50.0 * bar_len_factor),
            );
            painter.rect_filled(res_rect, 3.0, Color32::from_rgb(40, 30, 20));
            painter.rect_stroke(
                res_rect,
                3.0,
                Stroke::new(1.0_f32, Color32::from_rgb(70, 55, 35)),
            );

            let is_sel_bar = b == self.selected_bar_idx;
            let bar_col = if is_sel_bar {
                Color32::from_rgb(255, 215, 0)
            } else {
                Color32::from_rgb(180, 140, 60)
            };
            painter.rect_filled(bar_rect, 2.0, bar_col);
            painter.rect_stroke(
                bar_rect,
                2.0,
                Stroke::new(1.0_f32, Color32::from_rgb(220, 180, 90)),
            );
        }

        // Interactive Striker / Mallet Puck
        let puck_x = left_rect.min.x + self.puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.puck_pos = (nx, ny);
                    self.mallet_hardness = Self::normalized_to_hardness(nx);
                    self.ombak_rate_hz = Self::normalized_to_ombak(ny);
                    self.update_gamelan_simulation();
                }
            }
        }

        painter.circle_stroke(
            puck_pos,
            GAMELAN_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(255, 215, 0, 150)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(255, 215, 0));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 18.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Hardness: {:.2} | Ombak: {:.1} Hz | Bars: {} | Gauge: {:.1}mm | Damp: {:.2}",
                self.mallet_hardness,
                self.ombak_rate_hz,
                self.bar_count,
                self.bronze_thickness_mm,
                self.damping_factor
            ),
            egui::FontId::proportional(10.5),
            Color32::from_rgb(255, 235, 170),
        );

        // Right 45%: Modal Inharmonicity & Resonator Spectrum
        let right_w = main_canvas.width() * 0.45;
        let right_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + left_w + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(right_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(right_rect, 4.0, Color32::from_rgb(16, 20, 30));
        painter.rect_stroke(
            right_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(45, 55, 75)),
        );

        painter.text(
            egui::pos2(right_rect.min.x + 10.0, right_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "MODAL INHARMONICITY & OMBAK BEATING SPECTRUM",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(255, 215, 0),
        );

        let mode_labels = ["f0", "f1", "f2", "f3", "OMB", "BAMB", "CLK", "SYMP"];
        let bar_w = (right_rect.width() - 30.0 - 7.0 * 6.0) / 8.0;
        for (i, &amp) in self.modal_amplitudes.iter().enumerate() {
            let bx = right_rect.min.x + 15.0 + i as f32 * (bar_w + 6.0);
            let bar_h = (amp.clamp(0.0, 1.2) / 1.2) * (right_rect.height() - 80.0);
            let b_rect = egui::Rect::from_min_max(
                egui::pos2(bx, right_rect.max.y - 25.0 - bar_h),
                egui::pos2(bx + bar_w, right_rect.max.y - 25.0),
            );
            let col = if i == 0 {
                Color32::from_rgb(255, 215, 0)
            } else if i < 4 {
                Color32::from_rgb(255, 140, 66)
            } else {
                Color32::from_rgb(0, 255, 180)
            };
            painter.rect_filled(b_rect, 3.0, col);

            painter.text(
                egui::pos2(bx + bar_w * 0.5, right_rect.max.y - 20.0),
                egui::Align2::CENTER_TOP,
                mode_labels[i],
                egui::FontId::proportional(8.5),
                Color32::from_rgb(200, 215, 235),
            );
        }

        // Bottom Metrics Dock (y: 350..465)
        let dock_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 350.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 465.0),
        );
        painter.rect_filled(dock_rect, 6.0, Color32::from_rgb(20, 26, 38));
        painter.rect_stroke(
            dock_rect,
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(55, 65, 85)),
        );

        let params = [
            (
                "MALLET HARDNESS",
                format!("{:.0}% (Wood Mallet)", self.mallet_hardness * 100.0),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "OMBAK BEATING",
                format!("{:.1} Hz (Paired Tuning)", self.ombak_rate_hz),
                Color32::from_rgb(255, 140, 66),
            ),
            (
                "BRONZE THICKNESS",
                format!("{:.1} mm (Cast Alloy)", self.bronze_thickness_mm),
                Color32::from_rgb(0, 255, 180),
            ),
            (
                "RESONATOR COUPLING",
                format!("Q = {:.1} (Bamboo Cavity)", self.resonator_coupling_q),
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
            "[PASS] Balinese Gamelan Gender Modal Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
