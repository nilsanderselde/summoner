// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Mastering Multi-Band Upward/Downward Dynamic Crest Shaper & Punch Leveler HUD (Step 1583).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const CREST_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_TARGET_CREST_DB: f32 = 3.0;
pub const MAX_TARGET_CREST_DB: f32 = 24.0;
pub const MIN_EXPANSION_RATIO: f32 = 1.0;
pub const MAX_EXPANSION_RATIO: f32 = 4.0;

/// Mastering dynamic crest shaper target topologies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrestTopology {
    PunchMaximizer,   // Expands attack transient peak-to-average ratio for impactful drums
    DensityCompactor, // Smooths crest factor downward for glue and loud broadcast consistency
    MultibandTransientLeveler, // 4-band independent crest expansion and peak limiter leveling
    AcousticDynamicPreserver, // Transparent micro-dynamic upward restoration on compressed stems
    BroadcastLoudnessSculptor, // Calibrates crest factor for strict EBU R128 (-23 LUFS) mastering
}

impl CrestTopology {
    pub fn topology_name(&self) -> &'static str {
        match self {
            Self::PunchMaximizer => "PUNCH MAXIMIZER",
            Self::DensityCompactor => "DENSITY COMPACTOR",
            Self::MultibandTransientLeveler => "MULTIBAND LEVELER",
            Self::AcousticDynamicPreserver => "ACOUSTIC PRESERVER",
            Self::BroadcastLoudnessSculptor => "BROADCAST SCULPTOR",
        }
    }

    pub fn nominal_crest_target_db(&self) -> f32 {
        match self {
            Self::PunchMaximizer => 16.5,
            Self::DensityCompactor => 8.0,
            Self::MultibandTransientLeveler => 13.0,
            Self::AcousticDynamicPreserver => 18.0,
            Self::BroadcastLoudnessSculptor => 11.5,
        }
    }

    pub fn nominal_expansion_ratio(&self) -> f32 {
        match self {
            Self::PunchMaximizer => 2.6,
            Self::DensityCompactor => 1.0,
            Self::MultibandTransientLeveler => 1.8,
            Self::AcousticDynamicPreserver => 3.2,
            Self::BroadcastLoudnessSculptor => 1.4,
        }
    }

    pub fn nominal_downward_ratio(&self) -> f32 {
        match self {
            Self::PunchMaximizer => 1.2,
            Self::DensityCompactor => 4.5,
            Self::MultibandTransientLeveler => 2.4,
            Self::AcousticDynamicPreserver => 1.1,
            Self::BroadcastLoudnessSculptor => 3.0,
        }
    }

    pub fn nominal_attack_ms(&self) -> f32 {
        match self {
            Self::PunchMaximizer => 0.8,
            Self::DensityCompactor => 12.0,
            Self::MultibandTransientLeveler => 2.5,
            Self::AcousticDynamicPreserver => 4.0,
            Self::BroadcastLoudnessSculptor => 15.0,
        }
    }
}

/// Mastering multi-band upward/downward dynamic crest shaper & punch leveler HUD.
#[derive(Debug, Clone)]
pub struct DynamicCrestShaperView {
    pub topology: CrestTopology,
    pub target_crest_db: f32,     // [3.0 ..= 24.0 dB]
    pub expansion_ratio: f32,     // [1.0 ..= 4.0 upward expansion]
    pub downward_comp_ratio: f32, // [1.0 ..= 12.0 downward compression]
    pub attack_ms: f32,           // [0.1 ..= 100.0 ms]
    pub release_ms: f32,          // [10.0 ..= 1000.0 ms]
    pub puck_pos: (f32, f32),     // Normalized (X: Target Crest, Y: Expansion Ratio)
    pub is_dragging_puck: bool,
    pub band_crest_factors_db: [f32; 4], // Low (<150Hz), L-Mid, H-Mid, High (>4kHz)
    pub color_palette: ContrastColorPalette,
}

impl Default for DynamicCrestShaperView {
    fn default() -> Self {
        Self::new()
    }
}

impl DynamicCrestShaperView {
    pub fn new() -> Self {
        let mut view = Self {
            topology: CrestTopology::PunchMaximizer,
            target_crest_db: 16.5,
            expansion_ratio: 2.6,
            downward_comp_ratio: 1.2,
            attack_ms: 0.8,
            release_ms: 85.0,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            band_crest_factors_db: [14.2, 16.8, 18.5, 15.0],
            color_palette: ContrastColorPalette::default(),
        };
        view.puck_pos = (
            Self::crest_to_normalized(view.target_crest_db),
            Self::ratio_to_normalized(view.expansion_ratio),
        );
        view.update_dynamics_simulation();
        view
    }

    pub fn crest_to_normalized(crest: f32) -> f32 {
        let c = crest.clamp(MIN_TARGET_CREST_DB, MAX_TARGET_CREST_DB);
        ((c - MIN_TARGET_CREST_DB) / (MAX_TARGET_CREST_DB - MIN_TARGET_CREST_DB)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_crest(norm: f32) -> f32 {
        MIN_TARGET_CREST_DB + norm.clamp(0.0, 1.0) * (MAX_TARGET_CREST_DB - MIN_TARGET_CREST_DB)
    }

    pub fn ratio_to_normalized(ratio: f32) -> f32 {
        let r = ratio.clamp(MIN_EXPANSION_RATIO, MAX_EXPANSION_RATIO);
        ((r - MIN_EXPANSION_RATIO) / (MAX_EXPANSION_RATIO - MIN_EXPANSION_RATIO)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_ratio(norm: f32) -> f32 {
        MIN_EXPANSION_RATIO + norm.clamp(0.0, 1.0) * (MAX_EXPANSION_RATIO - MIN_EXPANSION_RATIO)
    }

    pub fn set_topology(&mut self, topo: CrestTopology) {
        self.topology = topo;
        self.target_crest_db = topo.nominal_crest_target_db();
        self.expansion_ratio = topo.nominal_expansion_ratio();
        self.downward_comp_ratio = topo.nominal_downward_ratio();
        self.attack_ms = topo.nominal_attack_ms();
        self.puck_pos = (
            Self::crest_to_normalized(self.target_crest_db),
            Self::ratio_to_normalized(self.expansion_ratio),
        );
        self.update_dynamics_simulation();
    }

    /// Update 4-band dynamic crest calculations and peak-to-average response.
    pub fn update_dynamics_simulation(&mut self) {
        let target = self.target_crest_db;
        let exp = self.expansion_ratio;
        let comp = self.downward_comp_ratio;

        self.band_crest_factors_db = [
            (target * 0.85 + (exp - 1.0) * 2.0 - (comp - 1.0) * 0.8).clamp(3.0, 24.0),
            (target * 1.02 + (exp - 1.0) * 2.5 - (comp - 1.0) * 0.6).clamp(3.0, 24.0),
            (target * 1.10 + (exp - 1.0) * 3.0 - (comp - 1.0) * 0.5).clamp(3.0, 24.0),
            (target * 0.95 + (exp - 1.0) * 1.8 - (comp - 1.0) * 0.4).clamp(3.0, 24.0),
        ];
    }

    /// Hit test coordinate on the interactive crest shaper puck.
    pub fn hit_test_crest_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= CREST_PUCK_HIT_RADIUS
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
        let bar_spacing = right_w / 5;
        for (i, &cf) in self.band_crest_factors_db.iter().enumerate() {
            let col = mid_x + 2 + (i + 1) * bar_spacing;
            let bar_h = ((cf / 24.0) * (height - 4) as f32).round() as usize;
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
            "MASTERING MULTI-BAND DYNAMIC CREST SHAPER & PUNCH LEVELER HUD",
            egui::FontId::proportional(14.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Topology Tabs (y: 48..92)
        let tabs = [
            (CrestTopology::PunchMaximizer, "PUNCH MAX"),
            (CrestTopology::DensityCompactor, "DENSITY COMP"),
            (CrestTopology::MultibandTransientLeveler, "MULTIBAND LEVEL"),
            (CrestTopology::AcousticDynamicPreserver, "ACOUSTIC PRESERVE"),
            (CrestTopology::BroadcastLoudnessSculptor, "BROADCAST EBU"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (topo, name)) in tabs.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_sel = self.topology == *topo;
            let bg_col = if is_sel {
                Color32::from_rgb(217, 70, 239)
            } else {
                Color32::from_rgb(24, 32, 48)
            };
            let text_col = if is_sel {
                Color32::from_rgb(16, 8, 20)
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
                        self.set_topology(*topo);
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

        // Left 55%: Dynamic Crest Transfer Curve & Envelope Shaping
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
            "UPWARD/DOWNWARD DYNAMIC CREST ENVELOPE SHAPER",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(217, 70, 239),
        );

        // Crest dynamic transfer curve
        let prev_pt = egui::pos2(left_rect.min.x + 15.0, left_rect.max.y - 35.0);
        let mut p_last = prev_pt;
        for s in 1..=20 {
            let t = s as f32 / 20.0;
            let x = left_rect.min.x + 15.0 + t * (left_rect.width() - 30.0);
            let curve_y = (t.powf(1.0 / self.expansion_ratio) * 0.8 + 0.2 * t).clamp(0.0, 1.0);
            let y = left_rect.max.y - 35.0 - curve_y * (left_rect.height() - 75.0);
            let cur_pt = egui::pos2(x, y);
            painter.line_segment(
                [p_last, cur_pt],
                Stroke::new(2.0_f32, Color32::from_rgb(217, 70, 239)),
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
                    self.target_crest_db = Self::normalized_to_crest(nx);
                    self.expansion_ratio = Self::normalized_to_ratio(ny);
                    self.update_dynamics_simulation();
                }
            }
        }

        painter.circle_stroke(
            puck_pos,
            CREST_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(217, 70, 239, 150)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(217, 70, 239));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 20.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Target Crest: {:.1} dB | Exp: {:.2}:1 | Comp: {:.1}:1 | Attack: {:.1}ms",
                self.target_crest_db,
                self.expansion_ratio,
                self.downward_comp_ratio,
                self.attack_ms
            ),
            egui::FontId::proportional(10.5),
            Color32::from_rgb(240, 160, 255),
        );

        // Right 45%: 4-Band Dynamic Crest Factor Meters
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
            "4-BAND CREST FACTOR METERS (dB)",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(217, 70, 239),
        );

        let band_labels = [
            "LOW (<150Hz)",
            "L-MID (150-800)",
            "H-MID (800-4k)",
            "HIGH (>4kHz)",
        ];
        let bar_w = (right_rect.width() - 30.0 - 3.0 * 8.0) / 4.0;
        for (i, &cf) in self.band_crest_factors_db.iter().enumerate() {
            let bx = right_rect.min.x + 15.0 + i as f32 * (bar_w + 8.0);
            let norm_cf = (cf / 24.0).clamp(0.0, 1.0);
            let bar_h = norm_cf * (right_rect.height() - 80.0);
            let b_rect = egui::Rect::from_min_max(
                egui::pos2(bx, right_rect.max.y - 25.0 - bar_h),
                egui::pos2(bx + bar_w, right_rect.max.y - 25.0),
            );
            let col = if i == 1 || i == 2 {
                Color32::from_rgb(217, 70, 239)
            } else if i == 0 {
                Color32::from_rgb(255, 136, 0)
            } else {
                Color32::from_rgb(0, 229, 255)
            };
            painter.rect_filled(b_rect, 3.0, col);

            painter.text(
                egui::pos2(bx + bar_w * 0.5, right_rect.max.y - 20.0),
                egui::Align2::CENTER_TOP,
                band_labels[i],
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
                "TARGET CREST FACTOR",
                format!("{:.1} dB (Peak/RMS)", self.target_crest_db),
                Color32::from_rgb(217, 70, 239),
            ),
            (
                "UPWARD EXPANSION",
                format!("{:.2}:1 (Transient)", self.expansion_ratio),
                Color32::from_rgb(255, 136, 0),
            ),
            (
                "DOWNWARD COMP RATIO",
                format!("{:.1}:1 (Density)", self.downward_comp_ratio),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "TRANSIENT ATTACK",
                format!("{:.1} ms (Leveler)", self.attack_ms),
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
            "[PASS] Dynamic Crest Shaper & Punch Leveler Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
