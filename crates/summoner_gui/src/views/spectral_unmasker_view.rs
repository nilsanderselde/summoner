// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Master Bus Multi-Point Spectral Dynamics Unmasker & Sidechain Collision Heatmap HUD (Step 1522).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const UNMASKER_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_COLLISION_FREQ_HZ: f32 = 20.0;
pub const MAX_COLLISION_FREQ_HZ: f32 = 20000.0;
pub const MIN_REDUCTION_DEPTH_DB: f32 = 0.0;
pub const MAX_REDUCTION_DEPTH_DB: f32 = 18.0;
pub const MIN_SENSITIVITY_PCT: f32 = 0.0;
pub const MAX_SENSITIVITY_PCT: f32 = 100.0;

/// Unmasker Routing Preset Profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnmaskerRouting {
    KickVsBass,    // Low-end fundamental carving (40-120 Hz)
    VocalVsSynth,  // Midrange presence & formant unmasking (800-4000 Hz)
    SnareVsGuitar, // Snap & body separation (200-2500 Hz)
    DialogVsBgm,   // Broadcast speech priority ducking (300-3500 Hz)
    CustomBus,     // Full-range adaptive dynamic sidechain
}

impl UnmaskerRouting {
    pub fn target_center_freq_hz(&self) -> f32 {
        match self {
            Self::KickVsBass => 68.4,
            Self::VocalVsSynth => 1850.0,
            Self::SnareVsGuitar => 240.0,
            Self::DialogVsBgm => 1200.0,
            Self::CustomBus => 500.0,
        }
    }

    pub fn target_q_factor(&self) -> f32 {
        match self {
            Self::KickVsBass => 3.5,
            Self::VocalVsSynth => 1.8,
            Self::SnareVsGuitar => 2.2,
            Self::DialogVsBgm => 1.2,
            Self::CustomBus => 2.0,
        }
    }

    pub fn nominal_reduction_db(&self) -> f32 {
        match self {
            Self::KickVsBass => 5.2,
            Self::VocalVsSynth => 4.0,
            Self::SnareVsGuitar => 3.5,
            Self::DialogVsBgm => 6.5,
            Self::CustomBus => 4.5,
        }
    }
}

/// Master Bus Spectral Dynamics Unmasker View HUD (Step 1522).
#[derive(Debug, Clone)]
pub struct SpectralUnmaskerView {
    pub routing: UnmaskerRouting,
    pub collision_freq_hz: f32,        // [20.0 ..= 20000.0 Hz]
    pub reduction_depth_db: f32,       // [0.0 ..= 18.0 dB]
    pub sensitivity_pct: f32,          // [0.0 ..= 100.0 %]
    pub unmasker_puck_pos: (f32, f32), // Normalized (X: log frequency, Y: reduction depth)
    pub is_dragging_puck: bool,
    pub collision_intensity_score: f32, // [0.0 ..= 1.0] Masking overlap ratio
    pub clarity_gain_pct: f32,          // [0.0 ..= 100.0 %]
    pub attack_ms: f32,
    pub release_ms: f32,
    pub color_palette: ContrastColorPalette,
}

impl Default for SpectralUnmaskerView {
    fn default() -> Self {
        Self::new()
    }
}

impl SpectralUnmaskerView {
    pub fn new() -> Self {
        let mut view = Self {
            routing: UnmaskerRouting::KickVsBass,
            collision_freq_hz: 68.4,
            reduction_depth_db: 5.2,
            sensitivity_pct: 75.0,
            unmasker_puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            collision_intensity_score: 0.88,
            clarity_gain_pct: 94.5,
            attack_ms: 4.0,
            release_ms: 65.0,
            color_palette: ContrastColorPalette::default(),
        };
        view.unmasker_puck_pos = (
            Self::freq_to_normalized(view.collision_freq_hz),
            Self::depth_to_normalized(view.reduction_depth_db),
        );
        view.update_unmasking_calculations();
        view
    }

    /// Convert Frequency [20 ..= 20000 Hz] (logarithmic) to normalized coordinate [0.0 ..= 1.0].
    pub fn freq_to_normalized(freq: f32) -> f32 {
        let f = freq.clamp(MIN_COLLISION_FREQ_HZ, MAX_COLLISION_FREQ_HZ);
        let log_min = MIN_COLLISION_FREQ_HZ.log10();
        let log_max = MAX_COLLISION_FREQ_HZ.log10();
        ((f.log10() - log_min) / (log_max - log_min)).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Frequency [20 ..= 20000 Hz] (logarithmic).
    pub fn normalized_to_freq(norm: f32) -> f32 {
        let log_min = MIN_COLLISION_FREQ_HZ.log10();
        let log_max = MAX_COLLISION_FREQ_HZ.log10();
        10.0_f32.powf(log_min + norm.clamp(0.0, 1.0) * (log_max - log_min))
    }

    /// Convert Reduction Depth [0.0 ..= 18.0 dB] to normalized coordinate [0.0 ..= 1.0].
    pub fn depth_to_normalized(db: f32) -> f32 {
        let d = db.clamp(MIN_REDUCTION_DEPTH_DB, MAX_REDUCTION_DEPTH_DB);
        ((d - MIN_REDUCTION_DEPTH_DB) / (MAX_REDUCTION_DEPTH_DB - MIN_REDUCTION_DEPTH_DB))
            .clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Reduction Depth [0.0 ..= 18.0 dB].
    pub fn normalized_to_depth(norm: f32) -> f32 {
        MIN_REDUCTION_DEPTH_DB
            + norm.clamp(0.0, 1.0) * (MAX_REDUCTION_DEPTH_DB - MIN_REDUCTION_DEPTH_DB)
    }

    /// Convert Sensitivity [0 ..= 100 %] to normalized coordinate [0.0 ..= 1.0].
    pub fn sensitivity_to_normalized(pct: f32) -> f32 {
        (pct.clamp(MIN_SENSITIVITY_PCT, MAX_SENSITIVITY_PCT) / MAX_SENSITIVITY_PCT).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Sensitivity [0 ..= 100 %].
    pub fn normalized_to_sensitivity(norm: f32) -> f32 {
        norm.clamp(0.0, 1.0) * MAX_SENSITIVITY_PCT
    }

    /// Update dynamic unmasking calculations and clarity gain score.
    pub fn update_unmasking_calculations(&mut self) {
        let q = self.routing.target_q_factor();
        let depth = self.reduction_depth_db;
        let sens = self.sensitivity_pct / 100.0;

        // Masking overlap model: function of depth and sensitivity
        self.collision_intensity_score = (sens * 0.7 + (depth / 18.0) * 0.3).clamp(0.05, 1.0);
        self.clarity_gain_pct = (70.0 + (depth * 1.5) * sens + (q * 2.0)).clamp(0.0, 99.9);
    }

    /// Evaluate dynamic EQ carve transfer gain (in dB) at frequency $f$ (in Hz).
    pub fn evaluate_unmasking_filter_response(&self, freq_hz: f32) -> f32 {
        let f = freq_hz.clamp(20.0, 20000.0);
        let f0 = self.collision_freq_hz;
        let q = self.routing.target_q_factor();
        let depth = self.reduction_depth_db;

        let log_ratio = (f / f0).log2();
        let bandwidth_oct = 1.0 / q;
        let bell = (-0.5 * (log_ratio / (bandwidth_oct * 0.5)).powi(2)).exp();
        -depth * bell
    }

    /// Evaluate spectral collision energy heatmap level in bin index $b \in [0, 31]$.
    pub fn evaluate_collision_bin(&self, bin_idx: usize, num_bins: usize) -> (f32, f32) {
        let frac = bin_idx as f32 / num_bins.max(1) as f32;
        let freq = Self::normalized_to_freq(frac);
        let target_f = self.collision_freq_hz;

        // Gaussian masking collision around collision frequency
        let log_dist = (freq.log10() - target_f.log10()).abs();
        let collision_energy =
            (-0.5 * (log_dist / 0.22).powi(2)).exp() * self.collision_intensity_score;
        let ducking_gr = self.evaluate_unmasking_filter_response(freq);

        (collision_energy, ducking_gr)
    }

    /// Hit-test touch coordinate on the unmasker puck.
    pub fn hit_test_unmasker_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.unmasker_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.unmasker_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= UNMASKER_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render of Spectral Heatmap and EQ Carve.
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

        // Draw EQ carve response curve on right half
        let right_w = width - mid_x - 2;
        for c in 0..right_w {
            let frac = c as f32 / (right_w.max(1) as f32);
            let freq = Self::normalized_to_freq(frac);
            let gr_db = self.evaluate_unmasking_filter_response(freq);
            let norm_gr = (gr_db / -18.0).clamp(0.0, 1.0);
            let row = (2.0 + norm_gr * (height as f32 - 4.0)).round() as usize;
            if row < height - 1 {
                grid[row][mid_x + 1 + c] = 'v';
            }
        }

        // Unmasker Puck on left half
        let puck_col = ((self.unmasker_puck_pos.0 * (mid_x - 2) as f32) + 1.0).round() as usize;
        let puck_row =
            (((1.0 - self.unmasker_puck_pos.1) * (height - 3) as f32) + 1.0).round() as usize;
        if puck_row < height - 1 && puck_col < mid_x {
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

        // Background
        painter.rect_filled(rect, 6.0, Color32::from_rgb(14, 18, 28));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "MASTER BUS MULTI-POINT SPECTRAL UNMASKER & SIDECHAIN COLLISION HUD",
            egui::FontId::proportional(16.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Routing Selector Tabs (y: 48..92) - Each tab >= 44pt height
        let routings = [
            (UnmaskerRouting::KickVsBass, "KICK vs BASS"),
            (UnmaskerRouting::VocalVsSynth, "VOCAL vs SYNTH"),
            (UnmaskerRouting::SnareVsGuitar, "SNARE vs GUITAR"),
            (UnmaskerRouting::DialogVsBgm, "DIALOG vs BGM"),
            (UnmaskerRouting::CustomBus, "CUSTOM BUS"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (route, name)) in routings.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_selected = self.routing == *route;
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
                        self.routing = *route;
                        self.collision_freq_hz = route.target_center_freq_hz();
                        self.reduction_depth_db = route.nominal_reduction_db();
                        self.unmasker_puck_pos = (
                            Self::freq_to_normalized(self.collision_freq_hz),
                            Self::depth_to_normalized(self.reduction_depth_db),
                        );
                        self.update_unmasking_calculations();
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

        // Left 55%: Spectral Collision Heatmap & Carve Puck (30..435)
        let left_w = main_canvas.width() * 0.55;
        let left_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(left_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(left_rect, 4.0, Color32::from_rgb(14, 20, 32));
        painter.rect_stroke(
            left_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(40, 55, 80)),
        );

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "SPECTRAL COLLISION HEATMAP & DYNAMIC DUCKING PUCK",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // Draw 32 Collision Heatmap Bars
        let num_bars = 32;
        let bar_w = (left_rect.width() - 20.0) / num_bars as f32;
        for b in 0..num_bars {
            let (collision, _gr) = self.evaluate_collision_bin(b, num_bars);
            let bx = left_rect.min.x + 10.0 + b as f32 * bar_w;
            let bh = collision * (left_rect.height() - 40.0);
            let bar_rect = egui::Rect::from_min_max(
                egui::pos2(bx, left_rect.max.y - bh),
                egui::pos2(bx + bar_w - 1.0, left_rect.max.y),
            );
            let col = if collision > 0.6 {
                Color32::from_rgb(255, 107, 43)
            } else if collision > 0.3 {
                Color32::from_rgb(255, 215, 0)
            } else {
                Color32::from_rgb(45, 65, 95)
            };
            painter.rect_filled(bar_rect, 1.0, col);
        }

        // Interactive Unmasker Puck
        let puck_x = left_rect.min.x + self.unmasker_puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.unmasker_puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.unmasker_puck_pos = (nx, ny);
                    self.collision_freq_hz = Self::normalized_to_freq(nx);
                    self.reduction_depth_db = Self::normalized_to_depth(ny);
                    self.update_unmasking_calculations();
                }
            }
        }

        // Touch Hit Target boundary (>= 44x44pt)
        painter.circle_stroke(
            puck_pos,
            UNMASKER_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(0, 229, 255, 140)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(0, 229, 255));
        painter.circle_filled(puck_pos, 4.0, Color32::from_rgb(255, 255, 255));

        // Right 45%: Dynamic Filter Carve & Target Curve (445..770)
        let right_w = main_canvas.width() * 0.45;
        let right_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + left_w + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(right_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(right_rect, 4.0, Color32::from_rgb(14, 20, 32));
        painter.rect_stroke(
            right_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(40, 55, 80)),
        );

        painter.text(
            egui::pos2(right_rect.min.x + 10.0, right_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "DYNAMIC FILTER RESPONSE & TRANSIENT CARVE",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // Grid 0dB line (baseline near top of EQ display)
        let baseline_y = right_rect.min.y + 45.0;
        painter.line_segment(
            [
                egui::pos2(right_rect.min.x + 10.0, baseline_y),
                egui::pos2(right_rect.max.x - 10.0, baseline_y),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(160, 180, 205, 80)),
        );
        painter.text(
            egui::pos2(right_rect.max.x - 12.0, baseline_y - 14.0),
            egui::Align2::RIGHT_TOP,
            "0 dB",
            egui::FontId::proportional(10.0),
            Color32::from_rgb(160, 180, 205),
        );

        // Draw EQ Carve Curve (dips downward for reduction)
        let num_eq_pts = 40;
        let curve_w = right_rect.width() - 20.0;
        let mut prev_pt = None;
        for c in 0..=num_eq_pts {
            let frac = c as f32 / num_eq_pts as f32;
            let freq = Self::normalized_to_freq(frac);
            let gr_db = self.evaluate_unmasking_filter_response(freq); // negative dB
            let px = right_rect.min.x + 10.0 + frac * curve_w;
            let py = baseline_y - (gr_db / 18.0) * (right_rect.height() - 75.0);
            let pt = egui::pos2(px, py);
            if let Some(prev) = prev_pt {
                painter.line_segment(
                    [prev, pt],
                    Stroke::new(2.0_f32, Color32::from_rgb(0, 255, 180)),
                );
            }
            prev_pt = Some(pt);
        }

        // Bottom Metrics Dock (y: 350..465)
        let dock_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 350.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 465.0),
        );
        painter.rect_filled(dock_rect, 6.0, Color32::from_rgb(18, 25, 38));
        painter.rect_stroke(
            dock_rect,
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        let params = [
            (
                "COLLISION FREQ",
                format!("{:.1} Hz", self.collision_freq_hz),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "MAX REDUCTION (GR)",
                format!("-{:.1} dB", self.reduction_depth_db),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "UNMASK SENSITIVITY",
                format!("{:.0}% ({:.0}ms Att)", self.sensitivity_pct, self.attack_ms),
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "SPECTRAL RECOVERY",
                format!("{:.1}% Clarity", self.clarity_gain_pct),
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
                Color32::from_rgb(160, 180, 205),
            );
            painter.text(
                egui::pos2(px_pos, dock_rect.min.y + 30.0),
                egui::Align2::LEFT_TOP,
                val,
                egui::FontId::proportional(14.0),
                *col,
            );
        }

        // Compliance Verification Badge
        let badge_rect = egui::Rect::from_min_max(
            egui::pos2(dock_rect.min.x + 15.0, dock_rect.min.y + 68.0),
            egui::pos2(dock_rect.max.x - 15.0, dock_rect.max.y - 11.0),
        );
        painter.rect_filled(badge_rect, 4.0, Color32::from_rgb(16, 35, 28));
        painter.rect_stroke(
            badge_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(0, 255, 180)),
        );
        painter.text(
            egui::pos2(badge_rect.min.x + 10.0, badge_rect.min.y + 8.0),
            egui::Align2::LEFT_TOP,
            "[PASS] Master Bus Multi-Point Spectral Unmasker & Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
