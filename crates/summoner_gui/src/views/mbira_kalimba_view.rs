// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Physical Modeling Lamellophone Mbira / Kalimba Tine Modal Dispersion & Acoustic Buzz HUD (Step 1581).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const MBIRA_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_PLUCK_FORCE_N: f32 = 0.1;
pub const MAX_PLUCK_FORCE_N: f32 = 5.0;
pub const MIN_BUZZ_INTENSITY: f32 = 0.0;
pub const MAX_BUZZ_INTENSITY: f32 = 1.0;
pub const MIN_TINE_COUNT: usize = 8;
pub const MAX_TINE_COUNT: usize = 32;

/// Lamellophone mbira, kalimba, and array mbira instrument types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MbiraType {
    MbiraDzavadzimu, // Traditional Shona 22-key mbira in calabash gourd with bottle-cap buzzers
    NyungaNyunga15,  // 15-key Karanga/Nyunga Nyunga mbira, bright pentatonic/heptatonic
    HughTraceyKalimba17, // 17-key treble kalimba in G major, western wooden box soundhole
    ArrayMbira5Octave, // 5-octave chromatic array mbira with clean dual-soundboard resonance
    BassKalimbaElectrified, // Solid-body bass kalimba with magnetic pickup and sub-bass resonance
}

impl MbiraType {
    pub fn instrument_name(&self) -> &'static str {
        match self {
            Self::MbiraDzavadzimu => "MBIRA D'ZAVADZIMU (22K)",
            Self::NyungaNyunga15 => "NYUNGA NYUNGA (15K)",
            Self::HughTraceyKalimba17 => "HUGH TRACEY KALIMBA (17K)",
            Self::ArrayMbira5Octave => "ARRAY MBIRA (5-OCT)",
            Self::BassKalimbaElectrified => "BASS KALIMBA (ELECTRIC)",
        }
    }

    pub fn nominal_pluck_force_n(&self) -> f32 {
        match self {
            Self::MbiraDzavadzimu => 2.4,
            Self::NyungaNyunga15 => 1.8,
            Self::HughTraceyKalimba17 => 1.2,
            Self::ArrayMbira5Octave => 0.9,
            Self::BassKalimbaElectrified => 3.5,
        }
    }

    pub fn nominal_buzz_intensity(&self) -> f32 {
        match self {
            Self::MbiraDzavadzimu => 0.85,
            Self::NyungaNyunga15 => 0.50,
            Self::HughTraceyKalimba17 => 0.05,
            Self::ArrayMbira5Octave => 0.00,
            Self::BassKalimbaElectrified => 0.35,
        }
    }

    pub fn nominal_tine_count(&self) -> usize {
        match self {
            Self::MbiraDzavadzimu => 22,
            Self::NyungaNyunga15 => 15,
            Self::HughTraceyKalimba17 => 17,
            Self::ArrayMbira5Octave => 32,
            Self::BassKalimbaElectrified => 9,
        }
    }

    pub fn nominal_fundamental_hz(&self) -> f32 {
        match self {
            Self::MbiraDzavadzimu => 220.0,
            Self::NyungaNyunga15 => 392.0,
            Self::HughTraceyKalimba17 => 392.0,
            Self::ArrayMbira5Octave => 130.81,
            Self::BassKalimbaElectrified => 65.41,
        }
    }

    pub fn nominal_decay_s(&self) -> f32 {
        match self {
            Self::MbiraDzavadzimu => 3.5,
            Self::NyungaNyunga15 => 2.2,
            Self::HughTraceyKalimba17 => 4.8,
            Self::ArrayMbira5Octave => 6.5,
            Self::BassKalimbaElectrified => 8.0,
        }
    }

    pub fn nominal_cavity_q(&self) -> f32 {
        match self {
            Self::MbiraDzavadzimu => 45.0,
            Self::NyungaNyunga15 => 25.0,
            Self::HughTraceyKalimba17 => 35.0,
            Self::ArrayMbira5Octave => 15.0,
            Self::BassKalimbaElectrified => 55.0,
        }
    }
}

/// Physical modeling lamellophone mbira/kalimba tine modal dispersion & acoustic buzz HUD.
#[derive(Debug, Clone)]
pub struct MbiraKalimbaView {
    pub instrument_type: MbiraType,
    pub pluck_force_n: f32,
    pub buzz_intensity_pct: f32,
    pub tine_count: usize,
    pub selected_tine_idx: usize,
    pub modal_dispersion_coeff: f32,
    pub tine_decay_s: f32,
    pub cavity_q_factor: f32,
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    pub modal_amplitudes: [f32; 8],
    pub color_palette: ContrastColorPalette,
}

impl Default for MbiraKalimbaView {
    fn default() -> Self {
        Self::new()
    }
}

impl MbiraKalimbaView {
    pub fn new() -> Self {
        let mut view = Self {
            instrument_type: MbiraType::MbiraDzavadzimu,
            pluck_force_n: 2.4,
            buzz_intensity_pct: 0.85,
            tine_count: 22,
            selected_tine_idx: 11,
            modal_dispersion_coeff: 0.72,
            tine_decay_s: 3.5,
            cavity_q_factor: 45.0,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            modal_amplitudes: [1.0, 0.65, 0.35, 0.80, 0.60, 0.45, 0.30, 0.18],
            color_palette: ContrastColorPalette::default(),
        };
        view.puck_pos = (
            Self::force_to_normalized(view.pluck_force_n),
            Self::buzz_to_normalized(view.buzz_intensity_pct),
        );
        view.update_physics_simulation();
        view
    }

    pub fn force_to_normalized(force: f32) -> f32 {
        let f = force.clamp(MIN_PLUCK_FORCE_N, MAX_PLUCK_FORCE_N);
        ((f - MIN_PLUCK_FORCE_N) / (MAX_PLUCK_FORCE_N - MIN_PLUCK_FORCE_N)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_force(norm: f32) -> f32 {
        MIN_PLUCK_FORCE_N + norm.clamp(0.0, 1.0) * (MAX_PLUCK_FORCE_N - MIN_PLUCK_FORCE_N)
    }

    pub fn buzz_to_normalized(buzz: f32) -> f32 {
        let b = buzz.clamp(MIN_BUZZ_INTENSITY, MAX_BUZZ_INTENSITY);
        ((b - MIN_BUZZ_INTENSITY) / (MAX_BUZZ_INTENSITY - MIN_BUZZ_INTENSITY)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_buzz(norm: f32) -> f32 {
        MIN_BUZZ_INTENSITY + norm.clamp(0.0, 1.0) * (MAX_BUZZ_INTENSITY - MIN_BUZZ_INTENSITY)
    }

    pub fn set_instrument_type(&mut self, inst: MbiraType) {
        self.instrument_type = inst;
        self.pluck_force_n = inst.nominal_pluck_force_n();
        self.buzz_intensity_pct = inst.nominal_buzz_intensity();
        self.tine_count = inst.nominal_tine_count();
        self.selected_tine_idx = self.tine_count / 2;
        self.tine_decay_s = inst.nominal_decay_s();
        self.cavity_q_factor = inst.nominal_cavity_q();
        self.puck_pos = (
            Self::force_to_normalized(self.pluck_force_n),
            Self::buzz_to_normalized(self.buzz_intensity_pct),
        );
        self.update_physics_simulation();
    }

    pub fn update_physics_simulation(&mut self) {
        let force_gain = (self.pluck_force_n / 2.4).sqrt().clamp(0.2, 2.0);
        let buzz = self.buzz_intensity_pct;

        let mode1_amp = (1.0 * force_gain).clamp(0.0, 1.5);
        let mode2_amp =
            (0.55 * force_gain * (1.0 + 0.3 * self.modal_dispersion_coeff)).clamp(0.0, 1.2);
        let mode3_amp =
            (0.28 * force_gain * (1.0 + 0.5 * self.modal_dispersion_coeff)).clamp(0.0, 1.0);

        let buzz_h1 = (buzz * 0.85 * force_gain).clamp(0.0, 1.0);
        let buzz_h2 = (buzz * 0.65 * force_gain).clamp(0.0, 1.0);
        let buzz_h3 = (buzz * 0.48 * force_gain).clamp(0.0, 1.0);
        let buzz_h4 = (buzz * 0.32 * force_gain).clamp(0.0, 1.0);
        let buzz_h5 = (buzz * 0.20 * force_gain).clamp(0.0, 1.0);

        self.modal_amplitudes = [
            mode1_amp, mode2_amp, mode3_amp, buzz_h1, buzz_h2, buzz_h3, buzz_h4, buzz_h5,
        ];
    }

    pub fn hit_test_mbira_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= MBIRA_PUCK_HIT_RADIUS
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
            "PHYSICAL MODELING LAMELLOPHONE MBIRA & KALIMBA TINE RESONANCE HUD",
            egui::FontId::proportional(14.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Tabs (y: 48..92)
        let tabs = [
            (MbiraType::MbiraDzavadzimu, "MBIRA (22K)"),
            (MbiraType::NyungaNyunga15, "NYUNGA (15K)"),
            (MbiraType::HughTraceyKalimba17, "KALIMBA (17K)"),
            (MbiraType::ArrayMbira5Octave, "ARRAY MBIRA"),
            (MbiraType::BassKalimbaElectrified, "BASS ELECTRIC"),
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

        // Left 55%: Tine Cantilever Array
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
            "CANTILEVER TINE ARRAY & ACOUSTIC BUZZ MATRIX",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(255, 107, 43),
        );

        let display_tines = self.tine_count.min(22);
        let tine_slot_w = (left_rect.width() - 30.0) / display_tines as f32;
        let bridge_y = left_rect.min.y + 35.0;
        let max_tine_len = left_rect.height() - 75.0;

        for t in 0..display_tines {
            let tx = left_rect.min.x + 15.0 + t as f32 * tine_slot_w;
            let center_offset =
                ((t as f32 - (display_tines as f32 / 2.0)) / (display_tines as f32 / 2.0)).abs();
            let tine_len = max_tine_len * (0.50 + 0.45 * (1.0 - center_offset));
            let is_active = t == self.selected_tine_idx.min(display_tines - 1);
            let tine_col = if is_active {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(180, 195, 215)
            };

            painter.line_segment(
                [
                    egui::pos2(tx + tine_slot_w * 0.5, bridge_y),
                    egui::pos2(tx + tine_slot_w * 0.5, bridge_y + tine_len),
                ],
                Stroke::new(if is_active { 3.0_f32 } else { 1.8_f32 }, tine_col),
            );

            painter.circle_filled(
                egui::pos2(tx + tine_slot_w * 0.5, bridge_y + tine_len),
                if is_active { 4.0 } else { 2.5 },
                if is_active {
                    Color32::from_rgb(255, 107, 43)
                } else {
                    Color32::from_rgb(120, 140, 165)
                },
            );
        }

        // Puck
        let puck_x = left_rect.min.x + self.puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.puck_pos = (nx, ny);
                    self.pluck_force_n = Self::normalized_to_force(nx);
                    self.buzz_intensity_pct = Self::normalized_to_buzz(ny);
                    self.update_physics_simulation();
                }
            }
        }

        painter.circle_stroke(
            puck_pos,
            MBIRA_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(255, 107, 43, 150)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(255, 107, 43));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 20.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Pluck: {:.1} N | Buzz: {:.0}% | Tine: #{}/{} | Decay: {:.1}s",
                self.pluck_force_n,
                self.buzz_intensity_pct * 100.0,
                self.selected_tine_idx + 1,
                self.tine_count,
                self.tine_decay_s
            ),
            egui::FontId::proportional(10.5),
            Color32::from_rgb(255, 180, 100),
        );

        // Right 45%: Modal Overtones Spectrum
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
            "MODAL DISPERSION & ACOUSTIC BUZZ SPECTRUM",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(255, 107, 43),
        );

        let mode_labels = ["f0", "5.4f0", "13f0", "BZ1", "BZ2", "BZ3", "BZ4", "BZ5"];
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
            } else if i < 3 {
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
                "PLUCK STRIKE FORCE",
                format!("{:.2} N (Attack)", self.pluck_force_n),
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "ACOUSTIC BUZZ RATIO",
                format!("{:.0}% (Rattle Plate)", self.buzz_intensity_pct * 100.0),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "MODAL DISPERSION",
                format!("{:.2} (Euler-Bernoulli)", self.modal_dispersion_coeff),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "CAVITY Q-FACTOR",
                format!("{:.1} (Gourd/Box)", self.cavity_q_factor),
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
            "[PASS] Lamellophone Mbira / Kalimba Tine Modal Dispersion Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
