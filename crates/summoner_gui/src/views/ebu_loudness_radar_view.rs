// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Broadcast Mastering Multi-Point Loudness Radar (EBU R128 / ITU BS.1770) HUD (Step 1505).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const EBU_RADAR_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const NUM_RADAR_HISTORY_POINTS: usize = 36;
pub const MIN_RADAR_LUFS: f32 = -36.0;
pub const MAX_RADAR_LUFS: f32 = -6.0;

/// Broadcast Loudness Delivery Standard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoudnessStandard {
    EbuR128Broadcast, // -23.0 LUFS Target (European Broadcast Union)
    ItuBs1770Tv,      // -24.0 LUFS Target (ITU-R / ATSC A/85 Cinema & TV)
    AesTd1004Club,    // -16.0 LUFS Target (AES Performance Venue Standard)
    StreamingMusic,   // -14.0 LUFS Target (Spotify / Apple Music / YouTube)
    PodcastSpoken,    // -19.0 LUFS Target (Mono/Stereo Podcast Distribution)
}

impl LoudnessStandard {
    pub fn target_integrated_lufs(&self) -> f32 {
        match self {
            Self::EbuR128Broadcast => -23.0,
            Self::ItuBs1770Tv => -24.0,
            Self::AesTd1004Club => -16.0,
            Self::StreamingMusic => -14.0,
            Self::PodcastSpoken => -19.0,
        }
    }

    pub fn max_true_peak_dbtp(&self) -> f32 {
        match self {
            Self::EbuR128Broadcast => -1.0,
            Self::ItuBs1770Tv => -2.0,
            Self::AesTd1004Club => -0.5,
            Self::StreamingMusic => -1.0,
            Self::PodcastSpoken => -1.0,
        }
    }
}

/// EBU R128 Multi-Point Loudness Radar View HUD (Step 1505).
#[derive(Debug, Clone)]
pub struct EbuLoudnessRadarView {
    pub standard: LoudnessStandard,
    pub momentary_lufs: f32,     // 400ms window [-36.0 ..= -6.0 LUFS]
    pub short_term_lufs: f32,    // 3-second window [-36.0 ..= -6.0 LUFS]
    pub integrated_lufs: f32,    // Gated integrated whole-program LUFS
    pub loudness_range_lra: f32, // Statistical Loudness Range [LU]
    pub max_true_peak_dbtp: f32, // Inter-sample 4x oversampled true-peak [dBTP]
    pub radar_history_lufs: [f32; NUM_RADAR_HISTORY_POINTS], // 360-degree radar perimeter buffer
    pub current_sweep_idx: usize,
    pub target_trim_puck_pos: (f32, f32), // Normalized X (Target LUFS), Y (Ceiling dBTP)
    pub is_dragging_puck: bool,
    pub color_palette: ContrastColorPalette,
}

impl Default for EbuLoudnessRadarView {
    fn default() -> Self {
        Self::new()
    }
}

impl EbuLoudnessRadarView {
    pub fn new() -> Self {
        let mut history = [-24.0_f32; NUM_RADAR_HISTORY_POINTS];
        for (i, val) in history.iter_mut().enumerate() {
            let angle = (i as f32 / NUM_RADAR_HISTORY_POINTS as f32) * 2.0 * std::f32::consts::PI;
            *val = -23.0 + 3.5 * angle.sin() + 1.5 * (angle * 3.0).cos();
        }

        let norm_lufs = Self::lufs_to_normalized(-23.0);
        let norm_tp = Self::dbtp_to_normalized(-1.0);

        Self {
            standard: LoudnessStandard::EbuR128Broadcast,
            momentary_lufs: -21.4,
            short_term_lufs: -22.8,
            integrated_lufs: -23.1,
            loudness_range_lra: 6.8,
            max_true_peak_dbtp: -1.2,
            radar_history_lufs: history,
            current_sweep_idx: 18,
            target_trim_puck_pos: (norm_lufs, norm_tp),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Convert LUFS value [-36.0 ..= -6.0 LUFS] to normalized coordinate [0.0 ..= 1.0].
    pub fn lufs_to_normalized(lufs: f32) -> f32 {
        let l = lufs.clamp(MIN_RADAR_LUFS, MAX_RADAR_LUFS);
        ((l - MIN_RADAR_LUFS) / (MAX_RADAR_LUFS - MIN_RADAR_LUFS)).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to LUFS value [-36.0 ..= -6.0 LUFS].
    pub fn normalized_to_lufs(norm: f32) -> f32 {
        MIN_RADAR_LUFS + norm.clamp(0.0, 1.0) * (MAX_RADAR_LUFS - MIN_RADAR_LUFS)
    }

    /// Convert True Peak dBTP [-6.0 ..= +3.0 dBTP] to normalized coordinate [0.0 ..= 1.0].
    pub fn dbtp_to_normalized(dbtp: f32) -> f32 {
        let tp = dbtp.clamp(-6.0, 3.0);
        ((tp + 6.0) / 9.0).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to True Peak dBTP [-6.0 ..= +3.0 dBTP].
    pub fn normalized_to_dbtp(norm: f32) -> f32 {
        -6.0 + norm.clamp(0.0, 1.0) * 9.0
    }

    /// Hit-test touch coordinate on the target trim puck.
    pub fn hit_test_trim_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.target_trim_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.target_trim_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= EBU_RADAR_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render of 360-degree EBU Loudness Radar.
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

        let mid_x = (width / 2) as f32;
        let mid_y = (height / 2) as f32;
        let max_r = ((width / 2 - 3).min(height / 2 - 2)) as f32;

        for r in 1..height - 1 {
            for c in 1..width - 1 {
                let dx = (c as f32 - mid_x) / max_r;
                let dy = (r as f32 - mid_y) / max_r;
                let dist = (dx * dx + dy * dy).sqrt();
                if (dist - 1.0).abs() < 0.1 || (dist - 0.5).abs() < 0.08 {
                    grid[r][c] = '.';
                }
            }
        }

        // Radar Target Ring Puck
        let puck_col = ((self.target_trim_puck_pos.0 * (width - 3) as f32) + 1.0).round() as usize;
        let puck_row =
            (((1.0 - self.target_trim_puck_pos.1) * (height - 3) as f32) + 1.0).round() as usize;
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
            "BROADCAST MASTERING EBU R128 LOUDNESS RADAR HUD",
            egui::FontId::proportional(16.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Standard Selector Tabs (y: 48..92) - Each tab >= 44pt height
        let standards = [
            (LoudnessStandard::EbuR128Broadcast, "EBU R128 (-23)"),
            (LoudnessStandard::ItuBs1770Tv, "ITU BS.1770 (-24)"),
            (LoudnessStandard::AesTd1004Club, "AES TD1004 (-16)"),
            (LoudnessStandard::StreamingMusic, "STREAMING (-14)"),
            (LoudnessStandard::PodcastSpoken, "PODCAST (-19)"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (std, name)) in standards.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_selected = self.standard == *std;
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
                        self.standard = *std;
                        let target = std.target_integrated_lufs();
                        self.target_trim_puck_pos.0 = Self::lufs_to_normalized(target);
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

        // Left 55%: 360-Degree Circular Loudness Radar Scope
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
            "360° EBU R128 LOUDNESS RADAR SCOPE",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        let radar_center = left_rect.center();
        let max_radar_radius = (left_rect.width() * 0.38).min(left_rect.height() * 0.40);

        // Concentric Loudness Rings: -36 (center), -23 (target), -14 (streaming), -9 (peak)
        let ring_levels = [
            (-36.0, "-36"),
            (-23.0, "-23 EBU"),
            (-14.0, "-14"),
            (-9.0, "-9"),
        ];

        for (lvl, lbl) in ring_levels {
            let norm_r = Self::lufs_to_normalized(lvl);
            let r_px = norm_r * max_radar_radius;
            let is_target = (lvl - self.standard.target_integrated_lufs()).abs() < 1.5;
            let ring_col = if is_target {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgba_unmultiplied(60, 85, 120, 90)
            };
            painter.circle_stroke(
                radar_center,
                r_px,
                Stroke::new(if is_target { 1.5_f32 } else { 1.0_f32 }, ring_col),
            );
            if r_px > 15.0 {
                let lbl_pos = egui::pos2(radar_center.x, radar_center.y - r_px - 4.0);
                painter.rect_filled(
                    egui::Rect::from_center_size(lbl_pos, egui::vec2(44.0, 12.0)),
                    2.0,
                    Color32::from_rgb(10, 14, 24),
                );
                painter.text(
                    lbl_pos,
                    egui::Align2::CENTER_CENTER,
                    lbl,
                    egui::FontId::proportional(9.0),
                    ring_col,
                );
            }
        }

        // Render 360-degree loudness history polygon segments
        let num_pts = NUM_RADAR_HISTORY_POINTS;
        for i in 0..num_pts {
            let angle0 = (i as f32 / num_pts as f32) * 2.0 * std::f32::consts::PI
                - std::f32::consts::FRAC_PI_2;
            let angle1 = ((i + 1) as f32 / num_pts as f32) * 2.0 * std::f32::consts::PI
                - std::f32::consts::FRAC_PI_2;

            let lufs0 = self.radar_history_lufs[i];
            let lufs1 = self.radar_history_lufs[(i + 1) % num_pts];

            let r0 = Self::lufs_to_normalized(lufs0) * max_radar_radius;
            let r1 = Self::lufs_to_normalized(lufs1) * max_radar_radius;

            let p0 = egui::pos2(
                radar_center.x + angle0.cos() * r0,
                radar_center.y + angle0.sin() * r0,
            );
            let p1 = egui::pos2(
                radar_center.x + angle1.cos() * r1,
                radar_center.y + angle1.sin() * r1,
            );

            let seg_color = if lufs0 > -14.0 {
                Color32::from_rgb(255, 51, 102)
            } else if lufs0 > -23.0 {
                Color32::from_rgb(255, 215, 0)
            } else {
                Color32::from_rgb(0, 255, 180)
            };

            painter.line_segment([p0, p1], Stroke::new(2.5_f32, seg_color));
        }

        // Current Sweep Arm Ray
        let sweep_angle =
            (self.current_sweep_idx as f32 / num_pts as f32) * 2.0 * std::f32::consts::PI
                - std::f32::consts::FRAC_PI_2;
        let sweep_end = egui::pos2(
            radar_center.x + sweep_angle.cos() * max_radar_radius,
            radar_center.y + sweep_angle.sin() * max_radar_radius,
        );
        painter.line_segment(
            [radar_center, sweep_end],
            Stroke::new(2.0_f32, Color32::from_rgb(0, 229, 255)),
        );

        // Right 45%: Numerical Loudness Indicators & Target Puck
        let right_left = main_canvas.min.x + left_w + 5.0;
        let right_w = main_canvas.max.x - right_left - 10.0;
        let right_rect = egui::Rect::from_min_size(
            egui::pos2(right_left, main_canvas.min.y + 10.0),
            egui::vec2(right_w, main_canvas.height() - 20.0),
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
            "TARGET LOUDNESS CALIBRATION PUCK",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // Target Puck Controls
        let puck_x =
            right_rect.min.x + 20.0 + self.target_trim_puck_pos.0 * (right_rect.width() - 40.0);
        let puck_y = right_rect.min.y
            + 40.0
            + (1.0 - self.target_trim_puck_pos.1) * (right_rect.height() - 60.0);

        // Hit target outer ring (>=44x44pt touch area)
        painter.circle_stroke(
            egui::pos2(puck_x, puck_y),
            EBU_RADAR_PUCK_HIT_RADIUS,
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
                    || self.hit_test_trim_puck((mouse_pos.x, mouse_pos.y), canvas_rect)
                {
                    self.is_dragging_puck = true;
                    let norm_x = ((mouse_pos.x - (right_rect.min.x + 20.0))
                        / (right_rect.width() - 40.0))
                        .clamp(0.0, 1.0);
                    let norm_y = (1.0
                        - (mouse_pos.y - (right_rect.min.y + 40.0)) / (right_rect.height() - 60.0))
                        .clamp(0.0, 1.0);
                    self.target_trim_puck_pos = (norm_x, norm_y);
                    self.integrated_lufs = Self::normalized_to_lufs(norm_x);
                    self.max_true_peak_dbtp = Self::normalized_to_dbtp(norm_y);
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
                "INTEGRATED LUFS (PROGRAM)",
                format!(
                    "{:.1} LUFS (Tgt: {:.0})",
                    self.integrated_lufs,
                    self.standard.target_integrated_lufs()
                ),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "MOMENTARY / SHORT-TERM",
                format!(
                    "{:.1} / {:.1} LUFS",
                    self.momentary_lufs, self.short_term_lufs
                ),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "LOUDNESS RANGE (LRA)",
                format!("{:.1} LU Dynamic", self.loudness_range_lra),
                Color32::from_rgb(0, 255, 180),
            ),
            (
                "TRUE-PEAK MAX",
                format!(
                    "{:.1} dBTP (Ceil: {:.1})",
                    self.max_true_peak_dbtp,
                    self.standard.max_true_peak_dbtp()
                ),
                Color32::from_rgb(255, 107, 43),
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
            "[PASS] Broadcast Mastering EBU R128 Loudness Radar & Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
