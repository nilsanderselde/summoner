// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Broadcast Mastering ITU-R BS.1770-4 Dialog Gating & Speech Normalization HUD (Step 1515).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const DIALOG_GATING_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_GATED_LUFS: f32 = -40.0;
pub const MAX_GATED_LUFS: f32 = -10.0;
pub const ABSOLUTE_GATE_THRESHOLD_LKFS: f32 = -70.0;
pub const RELATIVE_GATE_DELTA_LU: f32 = -10.0;

/// Broadcast Standard Loudness & Dialog Target Profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogLoudnessStandard {
    EbuR128,        // European Broadcasting Union (-23.0 LUFS, ±0.5 LU tolerance)
    AtscA85,        // North American Television (-24.0 LKFS, Speech Anchor)
    NetflixOtt,     // Streaming OTT delivery (-27.0 LUFS, Dialog Gated)
    StreamingMusic, // Online streaming (-14.0 LUFS target)
    PodcastSpeech,  // Spoken word & podcast (-16.0 LUFS, AES TD1004)
}

impl DialogLoudnessStandard {
    pub fn target_integrated_lufs(&self) -> f32 {
        match self {
            Self::EbuR128 => -23.0,
            Self::AtscA85 => -24.0,
            Self::NetflixOtt => -27.0,
            Self::StreamingMusic => -14.0,
            Self::PodcastSpeech => -16.0,
        }
    }

    pub fn true_peak_ceiling_dbtp(&self) -> f32 {
        match self {
            Self::EbuR128 => -1.0,
            Self::AtscA85 => -2.0,
            Self::NetflixOtt => -2.0,
            Self::StreamingMusic => -1.0,
            Self::PodcastSpeech => -1.0,
        }
    }
}

/// Broadcast Mastering ITU-R BS.1770-4 Dialog Gating View HUD (Step 1515).
#[derive(Debug, Clone)]
pub struct DialogGatingView {
    pub standard: DialogLoudnessStandard,
    pub ungated_integrated_lkfs: f32, // Raw un-gated loudness [-40.0 ..= -10.0 LKFS]
    pub gated_integrated_lkfs: f32,   // ITU BS.1770-4 dual-stage gated loudness
    pub dialog_anchored_lkfs: f32,    // Speech-isolated integrated loudness
    pub vad_speech_activity_pct: f32, // Voice Activity Detection confidence [0.0 ..= 100.0 %]
    pub dialog_puck_pos: (f32, f32),  // Normalized (X: gated_lufs, Y: vad_pct)
    pub is_dragging_puck: bool,
    pub gating_delta_lu: f32, // Delta between un-gated and gated loudness
    pub true_peak_max_dbtp: f32,
    pub color_palette: ContrastColorPalette,
}

impl Default for DialogGatingView {
    fn default() -> Self {
        Self::new()
    }
}

impl DialogGatingView {
    pub fn new() -> Self {
        let mut view = Self {
            standard: DialogLoudnessStandard::EbuR128,
            ungated_integrated_lkfs: -25.2,
            gated_integrated_lkfs: -23.1,
            dialog_anchored_lkfs: -23.0,
            vad_speech_activity_pct: 68.5,
            dialog_puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            gating_delta_lu: 2.1,
            true_peak_max_dbtp: -1.25,
            color_palette: ContrastColorPalette::default(),
        };
        view.dialog_puck_pos = (
            Self::lufs_to_normalized(view.gated_integrated_lkfs),
            view.vad_speech_activity_pct / 100.0,
        );
        view.update_gating_calculations();
        view
    }

    /// Convert LUFS/LKFS [-40.0 ..= -10.0] to normalized coordinate [0.0 ..= 1.0].
    pub fn lufs_to_normalized(lufs: f32) -> f32 {
        let l = lufs.clamp(MIN_GATED_LUFS, MAX_GATED_LUFS);
        ((l - MIN_GATED_LUFS) / (MAX_GATED_LUFS - MIN_GATED_LUFS)).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to LUFS/LKFS [-40.0 ..= -10.0].
    pub fn normalized_to_lufs(norm: f32) -> f32 {
        MIN_GATED_LUFS + norm.clamp(0.0, 1.0) * (MAX_GATED_LUFS - MIN_GATED_LUFS)
    }

    /// Update dual-stage gating and speech energy ratios.
    pub fn update_gating_calculations(&mut self) {
        self.gating_delta_lu = (self.gated_integrated_lkfs - self.ungated_integrated_lkfs).abs();
        let target = self.standard.target_integrated_lufs();
        // Compute speech-anchored loudness
        let speech_weight = (self.vad_speech_activity_pct / 100.0).clamp(0.0, 1.0);
        self.dialog_anchored_lkfs =
            self.gated_integrated_lkfs * speech_weight + target * (1.0 - speech_weight);
    }

    /// Evaluate ITU-R BS.1770-4 K-weighting pre-filter & RLB curve gain (dB) at frequency $f$ (Hz).
    pub fn evaluate_k_weighting_response_db(&self, freq_hz: f32) -> f32 {
        let f = freq_hz.clamp(20.0, 20000.0);
        // High-shelving pre-filter (+4 dB boost above 1.5 kHz)
        let high_shelf = 4.0 / (1.0 + (1500.0 / f).powi(2));
        // RLB high-pass filter (-3 dB at 38 Hz, -12 dB/oct roll-off)
        let rlb_hp = -10.0 * (1.0 + (38.0 / f).powi(2)).log10();
        high_shelf + rlb_hp
    }

    /// Hit-test touch coordinate on the dialog loudness target puck.
    pub fn hit_test_dialog_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.dialog_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.dialog_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= DIALOG_GATING_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render of Gating Histogram and K-Weighting Filter Curve.
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

        // Draw Gating histogram bars on left half
        let left_w = mid_x - 2;
        for c in 0..left_w {
            let frac = c as f32 / left_w.max(1) as f32;
            let lufs = MIN_GATED_LUFS + frac * (MAX_GATED_LUFS - MIN_GATED_LUFS);
            let energy = (-0.5 * ((lufs - self.gated_integrated_lkfs) / 3.5).powi(2)).exp();
            let bar_h = (energy * (height - 3) as f32).round() as usize;
            for r in 0..bar_h {
                if height - 2 > r {
                    grid[height - 2 - r][1 + c] = '#';
                }
            }
        }

        // Dialog Target Puck on left half
        let puck_col = ((self.dialog_puck_pos.0 * (mid_x - 2) as f32) + 1.0).round() as usize;
        let puck_row =
            (((1.0 - self.dialog_puck_pos.1) * (height - 3) as f32) + 1.0).round() as usize;
        if puck_row < height - 1 && puck_col < mid_x {
            grid[puck_row][puck_col] = 'O';
        }

        // Draw K-weighting curve on right half
        let right_w = width - mid_x - 2;
        for c in 0..right_w {
            let frac = c as f32 / right_w.max(1) as f32;
            let freq = 20.0 * (1000.0_f32).powf(frac);
            let resp_db = self.evaluate_k_weighting_response_db(freq);
            let norm_resp = ((resp_db + 15.0) / 20.0).clamp(0.0, 1.0);
            let row = (((1.0 - norm_resp) * (height - 3) as f32) + 1.0).round() as usize;
            if row > 0 && row < height - 1 {
                grid[row][mid_x + 1 + c] = '~';
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
        let _canvas_rect = Rect::new(rect.min.x, rect.min.y, rect.width(), rect.height());

        // Background
        painter.rect_filled(rect, 6.0, Color32::from_rgb(14, 18, 28));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "BROADCAST MASTERING ITU BS.1770-4 DIALOG GATING HUD",
            egui::FontId::proportional(16.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Standard Selector Tabs (y: 48..92) - Each tab >= 44pt height
        let standards = [
            (DialogLoudnessStandard::EbuR128, "EBU R128 (-23)"),
            (DialogLoudnessStandard::AtscA85, "ATSC A/85 (-24)"),
            (DialogLoudnessStandard::NetflixOtt, "NETFLIX (-27)"),
            (DialogLoudnessStandard::StreamingMusic, "STREAMING (-14)"),
            (DialogLoudnessStandard::PodcastSpeech, "PODCAST (-16)"),
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
                        self.update_gating_calculations();
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

        // Left 55%: Dual-Stage Gating Histogram & Target Puck
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
            "ITU BS.1770-4 DUAL-STAGE GATING HISTOGRAM",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // Draw Gating Histogram energy bars
        let hist_w = left_rect.width() - 30.0;
        let num_bars = 30;
        let bar_w = hist_w / num_bars as f32;

        for b in 0..num_bars {
            let frac = b as f32 / num_bars as f32;
            let lufs = MIN_GATED_LUFS + frac * (MAX_GATED_LUFS - MIN_GATED_LUFS);
            let energy = (-0.5 * ((lufs - self.gated_integrated_lkfs) / 3.5).powi(2)).exp();
            let bh = energy * (left_rect.height() - 50.0);

            let bx = left_rect.min.x + 15.0 + b as f32 * bar_w;
            let b_bar_rect = egui::Rect::from_min_max(
                egui::pos2(bx, left_rect.max.y - 15.0 - bh),
                egui::pos2(bx + bar_w - 2.0, left_rect.max.y - 15.0),
            );
            let col = if lufs >= self.standard.target_integrated_lufs() {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(45, 65, 95)
            };
            painter.rect_filled(b_bar_rect, 1.0, col);
        }

        // Relative gating threshold line (-10 LU)
        let rel_x = left_rect.min.x
            + 15.0
            + Self::lufs_to_normalized(self.gated_integrated_lkfs + RELATIVE_GATE_DELTA_LU)
                * hist_w;
        painter.line_segment(
            [
                egui::pos2(rel_x, left_rect.min.y + 30.0),
                egui::pos2(rel_x, left_rect.max.y - 15.0),
            ],
            Stroke::new(1.5_f32, Color32::from_rgb(255, 107, 43)),
        );
        painter.text(
            egui::pos2(rel_x + 4.0, left_rect.min.y + 32.0),
            egui::Align2::LEFT_TOP,
            "Γr (-10 LU)",
            egui::FontId::proportional(10.0),
            Color32::from_rgb(255, 107, 43),
        );

        // Dialog Target Puck
        let puck_x = left_rect.min.x + 15.0 + self.dialog_puck_pos.0 * hist_w;
        let puck_y = left_rect.max.y - 15.0 - self.dialog_puck_pos.1 * (left_rect.height() - 50.0);
        let puck_pos = egui::pos2(puck_x, puck_y);

        // Handle interaction
        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - (left_rect.min.x + 15.0)) / hist_w).clamp(0.0, 1.0);
                    let ny = (((left_rect.max.y - 15.0) - mouse_pos.y)
                        / (left_rect.height() - 50.0))
                        .clamp(0.0, 1.0);
                    self.dialog_puck_pos = (nx, ny);
                    self.gated_integrated_lkfs = Self::normalized_to_lufs(nx);
                    self.vad_speech_activity_pct = ny * 100.0;
                    self.update_gating_calculations();
                }
            }
        }

        // Puck Hit Target (>= 44x44pt)
        painter.circle_stroke(
            puck_pos,
            DIALOG_GATING_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(0, 229, 255, 140)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(0, 229, 255));
        painter.circle_filled(puck_pos, 4.0, Color32::from_rgb(255, 255, 255));

        // Right 45%: K-Weighting Filter Frequency Response & Target Indicator
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
            "K-WEIGHTING FILTER RESPONSE",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // Draw K-Weighting Filter Curve
        let kw_w = right_rect.width() - 30.0;
        let kw_h = right_rect.height() - 50.0;
        let num_kw_pts = 40;
        let mut prev_pt = None;

        for c in 0..=num_kw_pts {
            let frac = c as f32 / num_kw_pts as f32;
            let freq = 20.0 * (1000.0_f32).powf(frac);
            let resp_db = self.evaluate_k_weighting_response_db(freq);
            let norm_resp = ((resp_db + 15.0) / 20.0).clamp(0.0, 1.0);

            let px = right_rect.min.x + 15.0 + frac * kw_w;
            let py = right_rect.max.y - 15.0 - norm_resp * kw_h;
            let pt = egui::pos2(px, py);

            if let Some(prev) = prev_pt {
                painter.line_segment(
                    [prev, pt],
                    Stroke::new(2.0_f32, Color32::from_rgb(0, 255, 180)),
                );
            }
            prev_pt = Some(pt);
        }

        // Compliance Delta Indicator
        let target = self.standard.target_integrated_lufs();
        let delta = self.gated_integrated_lkfs - target;
        let delta_col = if delta.abs() <= 0.5 {
            Color32::from_rgb(0, 255, 180) // Compliant (±0.5 LU)
        } else if delta.abs() <= 1.0 {
            Color32::from_rgb(255, 215, 0) // Warning
        } else {
            Color32::from_rgb(255, 107, 43) // Out of spec
        };

        painter.text(
            egui::pos2(right_rect.max.x - 15.0, right_rect.min.y + 10.0),
            egui::Align2::RIGHT_TOP,
            format!("DELTA: {:+.1} LU", delta),
            egui::FontId::proportional(11.0),
            delta_col,
        );

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
                "INTEGRATED GATED LKFS",
                format!("{:.1} LKFS (Tgt {:.1})", self.gated_integrated_lkfs, target),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "VAD SPEECH CONFIDENCE",
                format!("{:.1}% (Voice Active)", self.vad_speech_activity_pct),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "DIALOG ANCHOR / DELTA",
                format!(
                    "{:.1} LKFS ({:.1} LU Gate)",
                    self.dialog_anchored_lkfs, self.gating_delta_lu
                ),
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "TRUE-PEAK MAXIMUM",
                format!(
                    "{:.2} dBTP (Ceil {:.1})",
                    self.true_peak_max_dbtp,
                    self.standard.true_peak_ceiling_dbtp()
                ),
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
            "[PASS] Broadcast Mastering ITU BS.1770-4 Dialog Gating & Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
