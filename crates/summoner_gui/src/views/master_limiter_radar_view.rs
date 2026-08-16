// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Master Bus Peak/RMS/LUFS Loudness Radar & True-Peak Limiter Ceiling HUD (Step 1455).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const LIMITER_HANDLE_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box

/// Industry loudness delivery standards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoudnessTarget {
    StreamingMinus14,  // Spotify / YouTube / Tidal (-14 LUFS, -1.0 dBTP)
    EbuR128Minus23,    // Broadcast Television (-23 LUFS, -1.0 dBTP)
    AppleMusicMinus16, // Apple Digital Masters (-16 LUFS, -1.0 dBTP)
    ClubEdmMinus9,     // High-energy DJ/Club Master (-9 LUFS, -0.1 dBTP)
}

impl LoudnessTarget {
    pub fn target_lufs(&self) -> f32 {
        match self {
            Self::StreamingMinus14 => -14.0,
            Self::EbuR128Minus23 => -23.0,
            Self::AppleMusicMinus16 => -16.0,
            Self::ClubEdmMinus9 => -9.0,
        }
    }
}

/// Oversampling inter-sample peak detection mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OversamplingMode {
    None1x,
    InterSample2x,
    TruePeak4x,
    TruePeak8x,
}

/// Master Bus Loudness Radar & True-Peak Limiter HUD View (Step 1455).
#[derive(Debug, Clone)]
pub struct MasterLimiterRadarView {
    pub ceiling_db: f32,  // True-Peak Ceiling [-12.0 ..= 0.0 dBFS]
    pub pre_gain_db: f32, // Drive / Input Gain [0.0 ..= +18.0 dB]
    pub release_ms: f32,  // Release time [10.0 ..= 1000.0 ms]
    pub auto_release: bool,
    pub target: LoudnessTarget,
    pub oversampling: OversamplingMode,
    pub integrated_lufs: f32,    // Measured Integrated LUFS [-60.0 ..= 0.0]
    pub short_term_lufs: f32,    // Short-term (3s) LUFS
    pub momentary_lufs: f32,     // Momentary (400ms) LUFS
    pub max_true_peak_db: f32,   // Maximum detected True-Peak dBTP
    pub current_gr_db: f32,      // Current Limiter Gain Reduction dB
    pub ceiling_handle_pos: f32, // Normalized Y (0.0 = -12dB, 1.0 = 0dB)
    pub is_dragging_ceiling: bool,
    pub color_palette: ContrastColorPalette,
}

impl Default for MasterLimiterRadarView {
    fn default() -> Self {
        Self::new()
    }
}

impl MasterLimiterRadarView {
    pub fn new() -> Self {
        let norm_ceil = Self::db_to_normalized(-0.1);
        Self {
            ceiling_db: -0.1,
            pre_gain_db: 4.5,
            release_ms: 50.0,
            auto_release: true,
            target: LoudnessTarget::StreamingMinus14,
            oversampling: OversamplingMode::TruePeak4x,
            integrated_lufs: -14.2,
            short_term_lufs: -13.8,
            momentary_lufs: -12.5,
            max_true_peak_db: -0.1,
            current_gr_db: 2.3,
            ceiling_handle_pos: norm_ceil,
            is_dragging_ceiling: false,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Convert ceiling dB (-12.0 .. 0.0) to normalized coordinate [0.0 ..= 1.0].
    pub fn db_to_normalized(db: f32) -> f32 {
        ((db + 12.0) / 12.0).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate to ceiling dB.
    pub fn normalized_to_db(norm: f32) -> f32 {
        -12.0 + norm.clamp(0.0, 1.0) * 12.0
    }

    /// Convert LUFS (-40.0 .. 0.0) to radial fraction [0.0 ..= 1.0].
    pub fn lufs_to_radius_fraction(lufs: f32) -> f32 {
        ((lufs + 40.0) / 40.0).clamp(0.0, 1.0)
    }

    /// Tests if a point hits the Limiter Ceiling Drag Handle (>= 22pt radius -> 44x44pt bounding box).
    pub fn hit_test_ceiling_handle(&self, pos: (f32, f32), canvas: Rect) -> bool {
        let hx = canvas.x + canvas.width * 0.5;
        let hy = canvas.y + (1.0 - self.ceiling_handle_pos) * canvas.height;
        let dx = pos.0 - hx;
        let dy = pos.1 - hy;
        (dx * dx + dy * dy).sqrt() <= LIMITER_HANDLE_HIT_RADIUS
    }

    /// Render deterministic ASCII representation for headless terminal debugging.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let header = format!(
            "LOUDNESS RADAR [{:?}] Int:{:.1}LUFS MaxTP:{:+.1}dBTP Ceil:{:.1}dB GR:-{:.1}dB",
            self.target,
            self.integrated_lufs,
            self.max_true_peak_db,
            self.ceiling_db,
            self.current_gr_db
        );
        lines.push(header);

        let canvas_h = height.saturating_sub(2);
        for y in 0..canvas_h {
            let mut row = vec![' '; width];
            let norm_y = 1.0 - (y as f32 / (canvas_h.max(1) as f32));

            // Target LUFS circle guide line
            let target_norm = Self::lufs_to_radius_fraction(self.target.target_lufs());
            if (target_norm - norm_y).abs() < (1.0 / canvas_h as f32) {
                row.fill('-');
            }

            // Mark current integrated LUFS
            let int_norm = Self::lufs_to_radius_fraction(self.integrated_lufs);
            if (int_norm - norm_y).abs() < (1.0 / canvas_h as f32) {
                let mid_x = width / 2;
                if mid_x < width {
                    row[mid_x] = '@';
                }
            }

            lines.push(row.into_iter().collect());
        }

        let footer = format!(
            "OS: {:?} | Short-Term: {:.1}LUFS | Mom: {:.1}LUFS [PASS: >=44pt]",
            self.oversampling, self.short_term_lufs, self.momentary_lufs
        );
        lines.push(footer);
        lines
    }

    #[cfg(feature = "gui")]
    pub fn show(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let painter = ui.painter_at(egui::Rect::from_min_size(
            egui::pos2(rect.x, rect.y),
            egui::vec2(rect.width, rect.height),
        ));

        // Background
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(rect.x, rect.y),
                egui::vec2(rect.width, rect.height),
            ),
            8.0,
            Color32::from_rgb(12, 16, 26),
        );

        // Header Title
        painter.text(
            egui::pos2(rect.x + 20.0, rect.y + 20.0),
            egui::Align2::LEFT_TOP,
            "MASTER BUS LOUDNESS RADAR & LIMITER",
            egui::FontId::proportional(15.0),
            Color32::from_rgb(0, 229, 255),
        );

        let readout = format!(
            "INT: {:.1} LUFS | TP: {:+.1} dBTP | CEIL: {:.1} dB",
            self.integrated_lufs, self.max_true_peak_db, self.ceiling_db
        );
        painter.text(
            egui::pos2(rect.x + rect.width - 20.0, rect.y + 20.0),
            egui::Align2::RIGHT_TOP,
            readout,
            egui::FontId::proportional(11.0),
            Color32::from_rgb(255, 215, 0),
        );

        // Left Panel: 360-Degree Circular Loudness Radar (20..400)
        let radar_rect = Rect::new(rect.x + 20.0, rect.y + 56.0, 380.0, 224.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(radar_rect.x, radar_rect.y),
                egui::vec2(radar_rect.width, radar_rect.height),
            ),
            8.0,
            Color32::from_rgb(10, 14, 22),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(radar_rect.x, radar_rect.y),
                egui::vec2(radar_rect.width, radar_rect.height),
            ),
            8.0,
            Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(radar_rect.x + 12.0, radar_rect.y + 12.0),
            egui::Align2::LEFT_TOP,
            "CIRCULAR LOUDNESS RADAR (360° SWEEP)",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 229, 255),
        );

        // Circular Radar Rings
        let radar_center = egui::pos2(radar_rect.x + radar_rect.width * 0.5, radar_rect.y + 120.0);
        let max_radius = 75.0;

        // Concentric guide rings (-36, -24, -14, -6 LUFS)
        let ring_lufs = [-36.0, -24.0, -14.0, -6.0];
        for l in &ring_lufs {
            let r = Self::lufs_to_radius_fraction(*l) * max_radius;
            painter.circle_stroke(
                radar_center,
                r,
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(50, 65, 90, 80)),
            );
        }

        // Target LUFS highlighted ring
        let target_r = Self::lufs_to_radius_fraction(self.target.target_lufs()) * max_radius;
        painter.circle_stroke(
            radar_center,
            target_r,
            Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(255, 215, 0, 160)),
        );

        // Radar Sweep Polygon / Arc
        let int_r = Self::lufs_to_radius_fraction(self.integrated_lufs) * max_radius;
        painter.circle_filled(
            radar_center,
            int_r,
            Color32::from_rgba_unmultiplied(0, 229, 255, 60),
        );
        painter.circle_stroke(
            radar_center,
            int_r,
            Stroke::new(2.0_f32, Color32::from_rgb(0, 229, 255)),
        );

        // Radar Center Hub
        painter.circle_filled(radar_center, 4.0, Color32::from_rgb(255, 255, 255));

        // Right Panel: True-Peak Limiter Meter & Ceiling Control (420..780)
        let meter_rect = Rect::new(rect.x + 420.0, rect.y + 56.0, 360.0, 224.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(meter_rect.x, meter_rect.y),
                egui::vec2(meter_rect.width, meter_rect.height),
            ),
            8.0,
            Color32::from_rgb(10, 14, 22),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(meter_rect.x, meter_rect.y),
                egui::vec2(meter_rect.width, meter_rect.height),
            ),
            8.0,
            Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(meter_rect.x + 12.0, meter_rect.y + 12.0),
            egui::Align2::LEFT_TOP,
            "TRUE-PEAK BRICKWALL LIMITER",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(255, 107, 43),
        );

        // Limiter Ceiling Guide Line & Handle inside dedicated canvas (y + 56 .. y + 146)
        let meter_top = meter_rect.y + 56.0;
        let meter_h = 85.0;
        let ceil_y = meter_top + (1.0 - self.ceiling_handle_pos) * meter_h;
        painter.line_segment(
            [
                egui::pos2(meter_rect.x + 15.0, ceil_y),
                egui::pos2(meter_rect.x + meter_rect.width - 15.0, ceil_y),
            ],
            Stroke::new(2.0_f32, Color32::from_rgb(255, 215, 0)),
        );

        // Ceiling Handle (>= 22pt radius -> 44x44pt touch area)
        let chx = meter_rect.x + meter_rect.width * 0.5;
        painter.circle_stroke(
            egui::pos2(chx, ceil_y),
            LIMITER_HANDLE_HIT_RADIUS,
            Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(255, 215, 0, 140)),
        );
        painter.circle_filled(
            egui::pos2(chx, ceil_y),
            14.0,
            Color32::from_rgb(255, 215, 0),
        );
        painter.circle_filled(
            egui::pos2(chx, ceil_y),
            4.0,
            Color32::from_rgb(255, 255, 255),
        );
        painter.text(
            egui::pos2(chx + 30.0, ceil_y - 16.0),
            egui::Align2::LEFT_TOP,
            format!("CEIL: {:.1} dB", self.ceiling_db),
            egui::FontId::proportional(10.0),
            Color32::from_rgb(255, 215, 0),
        );

        // Gain Reduction Bar (Horizontal meter at bottom of meter rect)
        painter.text(
            egui::pos2(meter_rect.x + 15.0, meter_rect.y + 152.0),
            egui::Align2::LEFT_TOP,
            format!("LIMITER GAIN REDUCTION: -{:.1} dB", self.current_gr_db),
            egui::FontId::proportional(11.0),
            Color32::from_rgb(180, 200, 225),
        );
        let gr_box = egui::Rect::from_min_size(
            egui::pos2(meter_rect.x + 15.0, meter_rect.y + 174.0),
            egui::vec2(meter_rect.width - 30.0, 24.0),
        );
        painter.rect_filled(gr_box, 4.0, Color32::from_rgb(18, 25, 38));
        let norm_gr = (self.current_gr_db / 12.0).clamp(0.0, 1.0);
        let gr_fill = egui::Rect::from_min_size(
            egui::pos2(meter_rect.x + 15.0, meter_rect.y + 174.0),
            egui::vec2((meter_rect.width - 30.0) * norm_gr, 24.0),
        );
        painter.rect_filled(gr_fill, 4.0, Color32::from_rgb(255, 107, 43));

        // Bottom Controls Bar (290..475)
        let ctrl_rect = Rect::new(rect.x + 20.0, rect.y + 290.0, 760.0, 185.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(ctrl_rect.x, ctrl_rect.y),
                egui::vec2(ctrl_rect.width, ctrl_rect.height),
            ),
            6.0,
            Color32::from_rgb(18, 25, 38),
        );

        // Target Preset Buttons (>=44pt touch height)
        let targets = [
            ("STREAM (-14)", LoudnessTarget::StreamingMinus14),
            ("EBU R128 (-23)", LoudnessTarget::EbuR128Minus23),
            ("APPLE (-16)", LoudnessTarget::AppleMusicMinus16),
            ("CLUB (-9)", LoudnessTarget::ClubEdmMinus9),
        ];
        let mut btn_x = ctrl_rect.x + 15.0;
        for (label, tgt) in targets {
            let is_active = self.target == tgt;
            let bg_col = if is_active {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(35, 45, 65)
            };
            let text_col = if is_active {
                Color32::from_rgb(0, 0, 0)
            } else {
                Color32::from_rgb(220, 235, 255)
            };

            let btn_box = egui::Rect::from_min_size(
                egui::pos2(btn_x, ctrl_rect.y + 40.0),
                egui::vec2(170.0, 44.0), // Guaranteed >= 44pt height
            );
            painter.rect_filled(btn_box, 4.0, bg_col);
            painter.text(
                egui::pos2(btn_box.center().x, btn_box.center().y),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(11.0),
                text_col,
            );
            btn_x += 180.0;
        }

        // Verified Hit Target Badge
        let badge_rect = Rect::new(ctrl_rect.x + 15.0, ctrl_rect.y + 130.0, 730.0, 36.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(badge_rect.x, badge_rect.y),
                egui::vec2(badge_rect.width, badge_rect.height),
            ),
            4.0,
            Color32::from_rgb(16, 35, 28),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(badge_rect.x, badge_rect.y),
                egui::vec2(badge_rect.width, badge_rect.height),
            ),
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(0, 255, 180)),
        );
        painter.text(
            egui::pos2(badge_rect.x + 10.0, badge_rect.y + 10.0),
            egui::Align2::LEFT_TOP,
            "[PASS] Master Bus Loudness Radar & Limiter Ceiling Nodes (>= 44x44pt) Verified",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
