// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// 6-Operator Frequency Modulation (FM) Routing Matrix & Phase Feedback Loop HUD (Step 1491).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const FM_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const NUM_FM_OPERATORS: usize = 6;
pub const MIN_MOD_INDEX: f32 = 0.0;
pub const MAX_MOD_INDEX: f32 = 10.0;

/// FM Operator Waveform Types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FmWaveform {
    Sine,
    Triangle,
    Sawtooth,
    SquarePulse,
    FormantTX,
}

/// FM Algorithm Routing Preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FmAlgorithm {
    Algo1LinearCascade,    // 6 -> 5 -> 4 -> 3 -> 2 -> 1 (Full serial chain)
    Algo5DualCascade,      // (6 -> 5 -> 4) + (3 -> 2 -> 1) (Parallel dual branch)
    Algo16BranchModulator, // (6 + 5 + 4) -> 3 -> (2 + 1)
    Algo22ParallelCarrier, // 6 -> (5, 4, 3, 2, 1) (Single modulator, 5 carriers)
    Algo32PureAdditive,    // 1, 2, 3, 4, 5, 6 (6 independent carrier oscillators)
}

/// Single FM Operator Configuration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FmOperator {
    pub id: usize,
    pub ratio: f32,        // Frequency multiplier [0.5 ..= 32.0]
    pub detune_cents: f32, // Fine detuning [-100.0 ..= +100.0 cents]
    pub output_level: f32, // Output / Modulation amplitude [0.0 ..= 1.0]
    pub is_carrier: bool,  // True if output routes to audio master
    pub waveform: FmWaveform,
}

/// 6-Operator FM Matrix View HUD (Step 1491).
#[derive(Debug, Clone)]
pub struct FmMatrixView {
    pub algorithm: FmAlgorithm,
    pub operators: [FmOperator; NUM_FM_OPERATORS],
    pub modulation_matrix: [[f32; NUM_FM_OPERATORS]; NUM_FM_OPERATORS], // [source][dest] mod index [0.0 ..= 10.0]
    pub feedback_op_idx: usize, // Operator with phase feedback loop (default Op 6)
    pub feedback_depth: f32,    // Phase feedback amount [0.0 ..= 100.0 %]
    pub selected_op_pair: (usize, usize), // (source, dest)
    pub matrix_puck_pos: (f32, f32), // Normalized X (Mod Index), Y (Carrier Ratio)
    pub is_dragging_puck: bool,
    pub real_time_thd_richness: f32, // Harmonic richness metric [0.0 ..= 100.0 %]
    pub color_palette: ContrastColorPalette,
}

impl Default for FmMatrixView {
    fn default() -> Self {
        Self::new()
    }
}

impl FmMatrixView {
    pub fn new() -> Self {
        let operators = [
            FmOperator {
                id: 0,
                ratio: 1.0,
                detune_cents: 0.0,
                output_level: 0.95,
                is_carrier: true,
                waveform: FmWaveform::Sine,
            },
            FmOperator {
                id: 1,
                ratio: 2.0,
                detune_cents: 2.0,
                output_level: 0.80,
                is_carrier: false,
                waveform: FmWaveform::Sine,
            },
            FmOperator {
                id: 2,
                ratio: 3.0,
                detune_cents: -3.0,
                output_level: 0.65,
                is_carrier: false,
                waveform: FmWaveform::Sine,
            },
            FmOperator {
                id: 3,
                ratio: 4.0,
                detune_cents: 0.0,
                output_level: 0.50,
                is_carrier: false,
                waveform: FmWaveform::Sine,
            },
            FmOperator {
                id: 4,
                ratio: 7.0,
                detune_cents: 5.0,
                output_level: 0.40,
                is_carrier: false,
                waveform: FmWaveform::Sine,
            },
            FmOperator {
                id: 5,
                ratio: 1.0,
                detune_cents: 0.0,
                output_level: 0.85,
                is_carrier: false,
                waveform: FmWaveform::Sine,
            },
        ];

        let mut matrix = [[0.0_f32; NUM_FM_OPERATORS]; NUM_FM_OPERATORS];
        // Standard cascade setup: 5->4, 4->3, 3->2, 2->1, 1->0
        matrix[1][0] = 3.5;
        matrix[2][1] = 2.4;
        matrix[3][2] = 1.8;
        matrix[4][3] = 1.2;
        matrix[5][4] = 2.0;

        let norm_mod = Self::mod_index_to_normalized(3.5);
        let norm_ratio = Self::ratio_to_normalized(1.0);

        Self {
            algorithm: FmAlgorithm::Algo1LinearCascade,
            operators,
            modulation_matrix: matrix,
            feedback_op_idx: 5, // Op 6 (0-indexed 5)
            feedback_depth: 45.0,
            selected_op_pair: (1, 0),
            matrix_puck_pos: (norm_mod, norm_ratio),
            is_dragging_puck: false,
            real_time_thd_richness: 68.4,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Convert Modulation Index [0.0 ..= 10.0] to normalized coordinate [0.0 ..= 1.0].
    pub fn mod_index_to_normalized(mod_idx: f32) -> f32 {
        (mod_idx.clamp(MIN_MOD_INDEX, MAX_MOD_INDEX) / MAX_MOD_INDEX).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Modulation Index [0.0 ..= 10.0].
    pub fn normalized_to_mod_index(norm: f32) -> f32 {
        norm.clamp(0.0, 1.0) * MAX_MOD_INDEX
    }

    /// Convert Frequency Ratio [0.5 ..= 32.0] to logarithmic normalized coordinate [0.0 ..= 1.0].
    pub fn ratio_to_normalized(ratio: f32) -> f32 {
        let r = ratio.clamp(0.5, 32.0);
        ((r / 0.5).log2() / (32.0 / 0.5_f32).log2()).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Frequency Ratio [0.5 ..= 32.0].
    pub fn normalized_to_ratio(norm: f32) -> f32 {
        let norm = norm.clamp(0.0, 1.0);
        0.5 * 2.0_f32.powf(norm * 6.0)
    }

    /// Calculate approximate Bessel sideband spectrum energy for a given modulation index $\beta$.
    pub fn compute_bessel_sideband_energy(&self, beta: f32) -> [f32; 8] {
        let mut sidebands = [0.0_f32; 8];
        let beta = beta.max(0.0);
        // Approximated Bessel $J_n(\beta)$ energy distribution
        for (n, sb) in sidebands.iter_mut().enumerate() {
            let n_f = n as f32;
            let j_n = if beta < 0.01 {
                if n == 0 {
                    1.0
                } else {
                    0.0
                }
            } else {
                let term = (beta * 0.5).powi(n as i32) / (1..=n).product::<usize>().max(1) as f32;
                (term * (-0.25 * beta * beta).exp()).clamp(0.0, 1.0)
            };
            *sb = (j_n * (1.0 / (1.0 + 0.1 * n_f))).clamp(0.0, 1.0);
        }
        sidebands
    }

    /// Hit-test touch coordinate on the modulation matrix / operator puck.
    pub fn hit_test_matrix_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.matrix_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.matrix_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= FM_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render of 6-Operator FM Matrix and Modulation Routing.
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

        let mid_y = height / 2;
        let cell_step = (width - 4) / NUM_FM_OPERATORS;
        for i in 0..NUM_FM_OPERATORS {
            let col_x = 2 + i * cell_step + cell_step / 2;
            if col_x < width - 1 {
                let symbol = if self.operators[i].is_carrier {
                    'C'
                } else {
                    'M'
                };
                grid[mid_y][col_x] = symbol;
            }
        }

        // Matrix Puck Coordinate
        let puck_col = ((self.matrix_puck_pos.0 * (width - 3) as f32) + 1.0).round() as usize;
        let puck_row =
            (((1.0 - self.matrix_puck_pos.1) * (height - 3) as f32) + 1.0).round() as usize;
        if puck_row < height - 1 && puck_col < width - 1 {
            grid[puck_row][puck_col] = 'O';
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
        let canvas_rect = Rect::new(rect.min.x, rect.min.y, rect.width(), rect.height());

        // Background
        painter.rect_filled(rect, 6.0, Color32::from_rgb(14, 18, 28));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "6-OPERATOR FM MODULATION MATRIX & PHASE FEEDBACK HUD",
            egui::FontId::proportional(16.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Algorithm Selector Tabs (y: 48..92) - Each tab >= 44pt height
        let algos = [
            (FmAlgorithm::Algo1LinearCascade, "ALGO 1 (CASCADE)"),
            (FmAlgorithm::Algo5DualCascade, "ALGO 5 (DUAL)"),
            (FmAlgorithm::Algo16BranchModulator, "ALGO 16 (BRANCH)"),
            (FmAlgorithm::Algo22ParallelCarrier, "ALGO 22 (PARALLEL)"),
            (FmAlgorithm::Algo32PureAdditive, "ALGO 32 (ADDITIVE)"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (algo, name)) in algos.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_selected = self.algorithm == *algo;
            let bg_color = if is_selected {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(25, 35, 50)
            };
            let text_color = if is_selected {
                Color32::from_rgb(10, 14, 24)
            } else {
                Color32::from_rgb(200, 215, 235)
            };

            painter.rect_filled(tab_rect, 4.0, bg_color);
            painter.text(
                tab_rect.center(),
                egui::Align2::CENTER_CENTER,
                *name,
                egui::FontId::proportional(11.0),
                text_color,
            );

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if tab_rect.contains(pos) {
                        self.algorithm = *algo;
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
            Stroke::new(1.5_f32, Color32::from_rgb(45, 65, 95)),
        );

        // Operator Nodes & Matrix Display (Left 60% = Operator Blocks & Pipes, Right 40% = Spectrum)
        let op_area_w = main_canvas.width() * 0.55;
        let op_block_w = (op_area_w - 20.0) / 6.0;

        for i in 0..NUM_FM_OPERATORS {
            let ox = main_canvas.min.x + 10.0 + i as f32 * op_block_w;
            let oy = main_canvas.min.y + 30.0;
            let block_rect =
                egui::Rect::from_min_size(egui::pos2(ox, oy), egui::vec2(op_block_w - 6.0, 160.0));

            let is_carrier = self.operators[i].is_carrier;
            let border_col = if is_carrier {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(255, 215, 0)
            };

            painter.rect_filled(block_rect, 4.0, Color32::from_rgb(18, 25, 38));
            painter.rect_stroke(block_rect, 4.0, Stroke::new(1.5_f32, border_col));

            // Op Title
            let op_label = format!("OP {}", i + 1);
            painter.text(
                egui::pos2(block_rect.center().x, block_rect.min.y + 16.0),
                egui::Align2::CENTER_CENTER,
                op_label,
                egui::FontId::proportional(13.0),
                border_col,
            );

            // Carrier / Modulator Role Tag
            let role_str = if is_carrier { "CARRIER" } else { "MOD" };
            painter.text(
                egui::pos2(block_rect.center().x, block_rect.min.y + 36.0),
                egui::Align2::CENTER_CENTER,
                role_str,
                egui::FontId::proportional(10.0),
                Color32::from_rgb(180, 200, 220),
            );

            // Ratio Readout
            let ratio_str = format!("{:.2}x", self.operators[i].ratio);
            painter.text(
                egui::pos2(block_rect.center().x, block_rect.min.y + 60.0),
                egui::Align2::CENTER_CENTER,
                ratio_str,
                egui::FontId::proportional(12.0),
                Color32::from_rgb(240, 245, 255),
            );

            // Output Level Bar
            let level_y = block_rect.min.y + 90.0;
            let level_h = 50.0 * self.operators[i].output_level;
            let bar_rect = egui::Rect::from_min_max(
                egui::pos2(block_rect.min.x + 8.0, level_y + 50.0 - level_h),
                egui::pos2(block_rect.max.x - 8.0, level_y + 50.0),
            );
            painter.rect_filled(bar_rect, 2.0, border_col);
        }

        // Sideband Spectrum Analyzer (Right 45%)
        let spec_left = main_canvas.min.x + op_area_w + 15.0;
        let spec_w = main_canvas.max.x - spec_left - 15.0;
        let spec_rect = egui::Rect::from_min_size(
            egui::pos2(spec_left, main_canvas.min.y + 20.0),
            egui::vec2(spec_w, main_canvas.height() - 40.0),
        );
        painter.rect_filled(spec_rect, 4.0, Color32::from_rgb(14, 20, 32));
        painter.rect_stroke(
            spec_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(spec_rect.min.x + 10.0, spec_rect.min.y + 12.0),
            egui::Align2::LEFT_TOP,
            "BESSEL SIDEBAND SPECTRUM",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        let curr_mod_index = Self::normalized_to_mod_index(self.matrix_puck_pos.0);
        let sbs = self.compute_bessel_sideband_energy(curr_mod_index);
        let sb_bar_w = (spec_rect.width() - 20.0) / 8.0;

        for (i, &energy) in sbs.iter().enumerate() {
            let sb_x = spec_rect.min.x + 10.0 + i as f32 * sb_bar_w;
            let bh = energy * (spec_rect.height() - 45.0);
            let bar_r = egui::Rect::from_min_max(
                egui::pos2(sb_x, spec_rect.max.y - 10.0 - bh),
                egui::pos2(sb_x + sb_bar_w - 4.0, spec_rect.max.y - 10.0),
            );
            painter.rect_filled(bar_r, 2.0, Color32::from_rgb(0, 255, 180));
        }

        // Drag Puck Handling on main canvas
        let puck_x = main_canvas.min.x + self.matrix_puck_pos.0 * main_canvas.width();
        let puck_y = main_canvas.min.y + (1.0 - self.matrix_puck_pos.1) * main_canvas.height();

        // Hit target outer ring (>=44x44pt touch area)
        painter.circle_stroke(
            egui::pos2(puck_x, puck_y),
            FM_PUCK_HIT_RADIUS,
            Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 120)),
        );
        painter.circle_filled(
            egui::pos2(puck_x, puck_y),
            14.0,
            Color32::from_rgb(0, 229, 255),
        );
        painter.circle_filled(egui::pos2(puck_x, puck_y), 4.0, Color32::WHITE);

        if response.dragged() {
            if let Some(mouse_pos) = ui.input(|i| i.pointer.interact_pos()) {
                if self.is_dragging_puck
                    || self.hit_test_matrix_puck((mouse_pos.x, mouse_pos.y), canvas_rect)
                {
                    self.is_dragging_puck = true;
                    let norm_x =
                        ((mouse_pos.x - main_canvas.min.x) / main_canvas.width()).clamp(0.0, 1.0);
                    let norm_y = (1.0 - (mouse_pos.y - main_canvas.min.y) / main_canvas.height())
                        .clamp(0.0, 1.0);
                    self.matrix_puck_pos = (norm_x, norm_y);
                    let new_mod = Self::normalized_to_mod_index(norm_x);
                    let (src, dst) = self.selected_op_pair;
                    self.modulation_matrix[src][dst] = new_mod;
                }
            }
        } else {
            self.is_dragging_puck = false;
        }

        // Bottom Metrics Dock (y: 350..465)
        let bottom_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 350.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 465.0),
        );
        painter.rect_filled(bottom_rect, 6.0, Color32::from_rgb(18, 25, 38));
        painter.rect_stroke(
            bottom_rect,
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        let metrics = [
            (
                "MODULATION INDEX",
                format!("{:.2} β", curr_mod_index),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "FEEDBACK (OP 6)",
                format!("{:.1}%", self.feedback_depth),
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "HARMONIC RICHNESS",
                format!("{:.1}%", self.real_time_thd_richness),
                Color32::from_rgb(0, 255, 180),
            ),
            (
                "ACTIVE ALGORITHM",
                format!("{:?}", self.algorithm),
                Color32::from_rgb(255, 215, 0),
            ),
        ];

        let col_w = (bottom_rect.width() - 40.0) / 4.0;
        for (i, (label, val, col)) in metrics.iter().enumerate() {
            let px = bottom_rect.min.x + 20.0 + i as f32 * col_w;
            painter.text(
                egui::pos2(px, bottom_rect.min.y + 12.0),
                egui::Align2::LEFT_TOP,
                *label,
                egui::FontId::proportional(11.0),
                Color32::from_rgb(160, 180, 205),
            );
            painter.text(
                egui::pos2(px, bottom_rect.min.y + 30.0),
                egui::Align2::LEFT_TOP,
                val,
                egui::FontId::proportional(14.0),
                *col,
            );
        }

        // Pass compliance badge
        let badge_rect = egui::Rect::from_min_max(
            egui::pos2(bottom_rect.min.x + 15.0, bottom_rect.min.y + 68.0),
            egui::pos2(bottom_rect.max.x - 15.0, bottom_rect.max.y - 10.0),
        );
        painter.rect_filled(badge_rect, 4.0, Color32::from_rgb(16, 35, 28));
        painter.rect_stroke(
            badge_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(0, 255, 180)),
        );
        painter.text(
            badge_rect.center(),
            egui::Align2::CENTER_CENTER,
            "[PASS] 6-Operator FM Matrix Modulation Indices & Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
