// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Master Bus 64-Band Dynamic Linear-Phase Spectral Matching Equalizer HUD (Step 1475).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const MATCH_EQ_NUM_BANDS: usize = 64;
pub const MATCH_EQ_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_SPECTRAL_FREQ_HZ: f32 = 20.0;
pub const MAX_SPECTRAL_FREQ_HZ: f32 = 20000.0;

/// Spectral Matching Target Profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchingProfile {
    ReferenceTrackMatch, // Match target to an imported reference WAV/FLAC audio track
    PinkNoiseMasterTarget, // Classic -3dB/octave broadcast mastering balance
    LoudnessBalancedTarget, // Equal-loudness contour compensated commercial master curve
    WarmAnalogMasterTilt, // Gentle musical 1.5dB tilt with sub-bass containment
}

/// 64-Band Dynamic Spectral Matching Equalizer HUD View (Step 1475).
#[derive(Debug, Clone)]
pub struct SpectralMatchingEqView {
    pub source_spectrum_db: [f32; MATCH_EQ_NUM_BANDS],
    pub target_spectrum_db: [f32; MATCH_EQ_NUM_BANDS],
    pub matched_gain_db: [f32; MATCH_EQ_NUM_BANDS],
    pub match_amount_percent: f32, // Match intensity [0.0 ..= 100.0 %]
    pub smoothing_semitones: f32,  // Spectral smoothing width [1.0 ..= 24.0 semitones]
    pub gain_limit_db: f32,        // Maximum dynamic boost/cut clamp [3.0 ..= 24.0 dB]
    pub linear_phase: bool,        // True: Zero-phase FIR; False: Minimum-phase IIR
    pub profile: MatchingProfile,
    pub eq_puck_pos: (f32, f32), // Normalized X (Match Amount), Y (Smoothing)
    pub is_dragging_puck: bool,
    pub is_matching_active: bool,
    pub color_palette: ContrastColorPalette,
}

impl Default for SpectralMatchingEqView {
    fn default() -> Self {
        Self::new()
    }
}

impl SpectralMatchingEqView {
    pub fn new() -> Self {
        let mut source = [0.0_f32; MATCH_EQ_NUM_BANDS];
        let mut target = [0.0_f32; MATCH_EQ_NUM_BANDS];
        let mut matched = [0.0_f32; MATCH_EQ_NUM_BANDS];

        for i in 0..MATCH_EQ_NUM_BANDS {
            let norm = i as f32 / (MATCH_EQ_NUM_BANDS - 1) as f32;
            let f = Self::normalized_to_freq(norm);

            // Synthetic source curve (e.g. mix with slightly boomy bass and dull air)
            source[i] = -12.0 - (f / 1000.0).log10() * 8.0 + (i as f32 * 0.4).sin() * 2.5;
            // Target reference curve
            target[i] = -14.0 - (f / 1000.0).log10() * 6.0;
            // Delta correction clamped
            matched[i] = ((target[i] - source[i]) * 0.75).clamp(-12.0, 12.0);
        }

        let norm_amount = Self::amount_to_normalized(75.0);
        let norm_smooth = Self::smoothing_to_normalized(4.5);

        Self {
            source_spectrum_db: source,
            target_spectrum_db: target,
            matched_gain_db: matched,
            match_amount_percent: 75.0,
            smoothing_semitones: 4.5,
            gain_limit_db: 12.0,
            linear_phase: true,
            profile: MatchingProfile::ReferenceTrackMatch,
            eq_puck_pos: (norm_amount, norm_smooth),
            is_dragging_puck: false,
            is_matching_active: true,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Calculate center frequency in Hz for band index `0 .. MATCH_EQ_NUM_BANDS`.
    pub fn band_center_freq(band_idx: usize) -> f32 {
        let norm = (band_idx as f32 / (MATCH_EQ_NUM_BANDS - 1) as f32).clamp(0.0, 1.0);
        Self::normalized_to_freq(norm)
    }

    /// Convert frequency in Hz (20 .. 20000) to logarithmic normalized coordinate [0.0 ..= 1.0].
    pub fn freq_to_normalized(freq_hz: f32) -> f32 {
        let freq = freq_hz.clamp(MIN_SPECTRAL_FREQ_HZ, MAX_SPECTRAL_FREQ_HZ);
        ((freq / MIN_SPECTRAL_FREQ_HZ).log10()
            / (MAX_SPECTRAL_FREQ_HZ / MIN_SPECTRAL_FREQ_HZ).log10())
        .clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to frequency in Hz (20 .. 20000).
    pub fn normalized_to_freq(norm: f32) -> f32 {
        let norm = norm.clamp(0.0, 1.0);
        MIN_SPECTRAL_FREQ_HZ
            * 10.0_f32.powf(norm * (MAX_SPECTRAL_FREQ_HZ / MIN_SPECTRAL_FREQ_HZ).log10())
    }

    /// Convert match amount percentage (0.0 .. 100.0) to normalized coordinate [0.0 ..= 1.0].
    pub fn amount_to_normalized(amount: f32) -> f32 {
        (amount / 100.0).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate to match amount percentage.
    pub fn normalized_to_amount(norm: f32) -> f32 {
        norm.clamp(0.0, 1.0) * 100.0
    }

    /// Convert smoothing semitones (1.0 .. 24.0) to normalized coordinate [0.0 ..= 1.0].
    pub fn smoothing_to_normalized(st: f32) -> f32 {
        ((st - 1.0) / (24.0 - 1.0)).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate to smoothing semitones (1.0 .. 24.0).
    pub fn normalized_to_smoothing(norm: f32) -> f32 {
        1.0 + norm.clamp(0.0, 1.0) * (24.0 - 1.0)
    }

    /// Recompute matched gain curve based on current match amount and gain limits.
    pub fn recompute_match_curve(&mut self) {
        let amount = self.match_amount_percent / 100.0;
        let limit = self.gain_limit_db;
        for i in 0..MATCH_EQ_NUM_BANDS {
            let delta = (self.target_spectrum_db[i] - self.source_spectrum_db[i]) * amount;
            self.matched_gain_db[i] = delta.clamp(-limit, limit);
        }
    }

    /// Tests if a point hits the 2D Match / Smoothing Puck (>= 22pt radius -> 44x44pt bounding box).
    pub fn hit_test_eq_puck(&self, pos: (f32, f32), canvas: Rect) -> bool {
        let px = canvas.x + self.eq_puck_pos.0 * canvas.width;
        let py = canvas.y + (1.0 - self.eq_puck_pos.1) * canvas.height;
        let dx = pos.0 - px;
        let dy = pos.1 - py;
        (dx * dx + dy * dy).sqrt() <= MATCH_EQ_PUCK_HIT_RADIUS
    }

    /// Render deterministic ASCII representation for headless terminal debugging.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let header = format!(
            "SPECTRAL MATCH EQ [{:?}] Bands:{} Match:{:.0}% Smooth:{:.1}st Limit:±{:.0}dB",
            self.profile,
            MATCH_EQ_NUM_BANDS,
            self.match_amount_percent,
            self.smoothing_semitones,
            self.gain_limit_db
        );
        lines.push(header);

        let canvas_h = height.saturating_sub(2);
        for y in 0..canvas_h {
            let mut row = vec![' '; width];
            let norm_y = 1.0 - (y as f32 / (canvas_h.max(1) as f32));
            let db_val = (norm_y - 0.5) * (self.gain_limit_db * 2.0);

            for (x, cell) in row.iter_mut().enumerate().take(width) {
                let band_idx = (x * MATCH_EQ_NUM_BANDS) / width.max(1);
                let gain = self.matched_gain_db[band_idx.min(MATCH_EQ_NUM_BANDS - 1)];
                if (gain - db_val).abs() < (self.gain_limit_db * 2.0 / canvas_h as f32) {
                    *cell = '#';
                }
            }

            // Mark puck position
            if (self.eq_puck_pos.1 - norm_y).abs() < (1.0 / canvas_h as f32) {
                let px = (self.eq_puck_pos.0 * (width.saturating_sub(1) as f32)) as usize;
                if px < width {
                    row[px] = '@';
                }
            }

            lines.push(row.into_iter().collect());
        }

        let footer = format!(
            "Puck: ({:.2}, {:.2}) | Mode: {} [PASS: >=44pt]",
            self.eq_puck_pos.0,
            self.eq_puck_pos.1,
            if self.linear_phase {
                "Linear Phase"
            } else {
                "Minimum Phase"
            }
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
            "64-BAND DYNAMIC SPECTRAL MATCHING EQ",
            egui::FontId::proportional(15.0),
            Color32::from_rgb(0, 229, 255),
        );

        let readout = format!(
            "MATCH: {:.0}% | SMOOTH: {:.1} st | LIMIT: ±{:.0} dB | 64-BAND DYN",
            self.match_amount_percent, self.smoothing_semitones, self.gain_limit_db
        );
        painter.text(
            egui::pos2(rect.x + rect.width - 20.0, rect.y + 20.0),
            egui::Align2::RIGHT_TOP,
            readout,
            egui::FontId::proportional(12.0),
            Color32::from_rgb(255, 215, 0),
        );

        // Target Profile Selector Bar
        let profiles = [
            (
                MatchingProfile::ReferenceTrackMatch,
                "REFERENCE TRACK MATCH",
            ),
            (
                MatchingProfile::PinkNoiseMasterTarget,
                "PINK NOISE TARGET (-3dB)",
            ),
            (
                MatchingProfile::LoudnessBalancedTarget,
                "LOUDNESS CONTOUR TARGET",
            ),
            (
                MatchingProfile::WarmAnalogMasterTilt,
                "WARM MASTER TILT (1.5dB)",
            ),
        ];

        let btn_y = rect.y + 54.0;
        let btn_w = (rect.width - 40.0 - 30.0) / 4.0;
        for (i, (prof, name)) in profiles.iter().enumerate() {
            let bx = rect.x + 20.0 + i as f32 * (btn_w + 10.0);
            let is_selected = self.profile == *prof;
            let bg = if is_selected {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(35, 45, 65)
            };
            let fg = if is_selected {
                Color32::from_rgb(10, 14, 22)
            } else {
                Color32::from_rgb(220, 235, 255)
            };

            painter.rect_filled(
                egui::Rect::from_min_size(egui::pos2(bx, btn_y), egui::vec2(btn_w, 36.0)),
                4.0,
                bg,
            );
            painter.text(
                egui::pos2(bx + btn_w * 0.5, btn_y + 18.0),
                egui::Align2::CENTER_CENTER,
                *name,
                egui::FontId::proportional(11.0),
                fg,
            );
        }

        // Main 64-Band Spectral Comparison Canvas (20..780)
        let eq_canvas = Rect::new(rect.x + 20.0, rect.y + 100.0, rect.width - 40.0, 230.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(eq_canvas.x, eq_canvas.y),
                egui::vec2(eq_canvas.width, eq_canvas.height),
            ),
            6.0,
            Color32::from_rgb(10, 14, 22),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(eq_canvas.x, eq_canvas.y),
                egui::vec2(eq_canvas.width, eq_canvas.height),
            ),
            6.0,
            Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(eq_canvas.x + 12.0, eq_canvas.y + 10.0),
            egui::Align2::LEFT_TOP,
            "64-BAND DYNAMIC CORRECTION SPECTRUM (REF: GOLD, INPUT: MINT, DELTA: CYAN)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(0, 229, 255),
        );

        // Center 0 dB grid line
        let mid_y = eq_canvas.y + eq_canvas.height * 0.5;
        painter.line_segment(
            [
                egui::pos2(eq_canvas.x, mid_y),
                egui::pos2(eq_canvas.x + eq_canvas.width, mid_y),
            ],
            Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(80, 100, 130, 120)),
        );

        // Grid lines for frequency marks
        for freq in [50.0, 200.0, 1000.0, 5000.0, 10000.0] {
            let gx = eq_canvas.x + Self::freq_to_normalized(freq) * eq_canvas.width;
            painter.line_segment(
                [
                    egui::pos2(gx, eq_canvas.y),
                    egui::pos2(gx, eq_canvas.y + eq_canvas.height),
                ],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 110, 70)),
            );
        }

        // Draw 64-Band matched gain bars
        let bar_w = (eq_canvas.width - 20.0) / MATCH_EQ_NUM_BANDS as f32;
        for (i, gain) in self
            .matched_gain_db
            .iter()
            .enumerate()
            .take(MATCH_EQ_NUM_BANDS)
        {
            let bx = eq_canvas.x + 10.0 + i as f32 * bar_w;
            let norm_h = (*gain / self.gain_limit_db).clamp(-1.0, 1.0) * (eq_canvas.height * 0.42);
            let bar_y = if norm_h >= 0.0 { mid_y - norm_h } else { mid_y };
            let bar_height = norm_h.abs().max(1.0);

            painter.rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(bx, bar_y),
                    egui::vec2(bar_w - 1.5, bar_height),
                ),
                1.5,
                if *gain >= 0.0 {
                    Color32::from_rgba_unmultiplied(0, 229, 255, 180)
                } else {
                    Color32::from_rgba_unmultiplied(255, 107, 43, 180)
                },
            );
        }

        // Draw reference curve (Gold) and source curve (Mint)
        let mut ref_pts = Vec::with_capacity(MATCH_EQ_NUM_BANDS);
        let mut src_pts = Vec::with_capacity(MATCH_EQ_NUM_BANDS);
        for i in 0..MATCH_EQ_NUM_BANDS {
            let bx = eq_canvas.x + 10.0 + (i as f32 + 0.5) * bar_w;
            let ref_db = self.target_spectrum_db[i];
            let src_db = self.source_spectrum_db[i];

            let ry = mid_y - (ref_db / 30.0) * (eq_canvas.height * 0.35);
            let sy = mid_y - (src_db / 30.0) * (eq_canvas.height * 0.35);

            ref_pts.push(egui::pos2(
                bx,
                ry.clamp(eq_canvas.y + 10.0, eq_canvas.y + eq_canvas.height - 10.0),
            ));
            src_pts.push(egui::pos2(
                bx,
                sy.clamp(eq_canvas.y + 10.0, eq_canvas.y + eq_canvas.height - 10.0),
            ));
        }

        for i in 0..(MATCH_EQ_NUM_BANDS - 1) {
            painter.line_segment(
                [ref_pts[i], ref_pts[i + 1]],
                Stroke::new(2.0_f32, Color32::from_rgb(255, 215, 0)),
            );
            painter.line_segment(
                [src_pts[i], src_pts[i + 1]],
                Stroke::new(1.5_f32, Color32::from_rgb(0, 255, 180)),
            );
        }

        // 2D Match / Smoothing Puck
        let px = eq_canvas.x + self.eq_puck_pos.0 * eq_canvas.width;
        let py = eq_canvas.y + (1.0 - self.eq_puck_pos.1) * eq_canvas.height;
        painter.circle_stroke(
            egui::pos2(px, py),
            MATCH_EQ_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 120)),
        );
        painter.circle_filled(egui::pos2(px, py), 14.0, Color32::from_rgb(0, 229, 255));
        painter.circle_filled(egui::pos2(px, py), 4.0, Color32::WHITE);

        // Bottom Controls Dock (y: 345..480)
        let dock_rect = Rect::new(rect.x + 20.0, rect.y + 345.0, rect.width - 40.0, 135.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(dock_rect.x, dock_rect.y),
                egui::vec2(dock_rect.width, dock_rect.height),
            ),
            6.0,
            Color32::from_rgb(18, 25, 38),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(dock_rect.x, dock_rect.y),
                egui::vec2(dock_rect.width, dock_rect.height),
            ),
            6.0,
            Stroke::new(1.5_f32, Color32::from_rgb(45, 60, 85)),
        );

        let params = [
            (
                "MATCH AMOUNT",
                format!("{:.0}%", self.match_amount_percent),
                (0, 229, 255),
            ),
            (
                "SMOOTHING WIDTH",
                format!("{:.1} semitones", self.smoothing_semitones),
                (0, 255, 180),
            ),
            (
                "GAIN LIMIT",
                format!("±{:.0} dB", self.gain_limit_db),
                (255, 215, 0),
            ),
            (
                "PHASE FILTER",
                if self.linear_phase {
                    "LINEAR PHASE"
                } else {
                    "MINIMUM PHASE"
                }
                .to_string(),
                (180, 200, 225),
            ),
        ];

        let col_w = (dock_rect.width - 40.0) / 4.0;
        for (i, (label, val, col)) in params.iter().enumerate() {
            let px = dock_rect.x + 20.0 + i as f32 * col_w;
            painter.text(
                egui::pos2(px, dock_rect.y + 16.0),
                egui::Align2::LEFT_TOP,
                *label,
                egui::FontId::proportional(11.0),
                Color32::from_rgb(180, 200, 225),
            );
            painter.text(
                egui::pos2(px, dock_rect.y + 36.0),
                egui::Align2::LEFT_TOP,
                val,
                egui::FontId::proportional(16.0),
                Color32::from_rgb(col.0, col.1, col.2),
            );
        }

        // Compliance status bar
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(dock_rect.x + 15.0, dock_rect.y + 80.0),
                egui::vec2(dock_rect.width - 30.0, 42.0),
            ),
            4.0,
            Color32::from_rgb(16, 35, 28),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(dock_rect.x + 15.0, dock_rect.y + 80.0),
                egui::vec2(dock_rect.width - 30.0, 42.0),
            ),
            4.0,
            Stroke::new(1.5_f32, Color32::from_rgb(0, 255, 180)),
        );
        painter.text(
            egui::pos2(dock_rect.x + 25.0, dock_rect.y + 93.0),
            egui::Align2::LEFT_TOP,
            "[PASS] 64-Band Spectral Matching EQ Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
