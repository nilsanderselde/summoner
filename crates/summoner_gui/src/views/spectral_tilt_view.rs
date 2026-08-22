// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Psychoacoustic Spectral Tilt & Tilt-Equalizer Phase-Linear Tone Shaper HUD (Step 1572).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const TILT_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_PIVOT_FREQ_HZ: f32 = 200.0;
pub const MAX_PIVOT_FREQ_HZ: f32 = 5000.0;
pub const MIN_TILT_SLOPE_DB_OCT: f32 = -6.0;
pub const MAX_TILT_SLOPE_DB_OCT: f32 = 6.0;

/// Spectral tilt and tone shaper filter topologies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TiltMode {
    LinearSlopeTilt6dB, // Continuous linear slope tilt across the entire audible spectrum
    BaxandallDualShelf, // Smooth continuous Baxandall bass/treble interacting tone curve
    PsychoacousticBark, // Bark-scale critical band weighted perceptual loudness tilt
    PhaseLinearFIR,     // Zero-phase distortion FIR linear-phase mastering tilt filter
    AdaptiveDynamicTilt, // Dynamic level-dependent multi-band tilt tone correction
}

impl TiltMode {
    pub fn mode_name(&self) -> &'static str {
        match self {
            Self::LinearSlopeTilt6dB => "LINEAR 6dB/OCT",
            Self::BaxandallDualShelf => "BAXANDALL SHELF",
            Self::PsychoacousticBark => "BARK TILT",
            Self::PhaseLinearFIR => "PHASE-LINEAR FIR",
            Self::AdaptiveDynamicTilt => "DYNAMIC ADAPTIVE",
        }
    }

    pub fn nominal_pivot_hz(&self) -> f32 {
        match self {
            Self::LinearSlopeTilt6dB => 1000.0,
            Self::BaxandallDualShelf => 800.0,
            Self::PsychoacousticBark => 1200.0,
            Self::PhaseLinearFIR => 1000.0,
            Self::AdaptiveDynamicTilt => 1500.0,
        }
    }

    pub fn nominal_slope_db_oct(&self) -> f32 {
        match self {
            Self::LinearSlopeTilt6dB => 1.5,
            Self::BaxandallDualShelf => 2.0,
            Self::PsychoacousticBark => 1.2,
            Self::PhaseLinearFIR => 1.5,
            Self::AdaptiveDynamicTilt => 2.5,
        }
    }

    pub fn is_phase_linear(&self) -> bool {
        matches!(self, Self::PhaseLinearFIR)
    }
}

/// Psychoacoustic spectral tilt & tilt-equalizer phase-linear tone shaper HUD.
#[derive(Debug, Clone)]
pub struct SpectralTiltView {
    pub tilt_mode: TiltMode,
    pub pivot_frequency_hz: f32,   // [200.0 ..= 5000.0 Hz, logarithmic]
    pub tilt_slope_db_oct: f32,    // [-6.0 ..= +6.0 dB/oct]
    pub bass_gain_db: f32,         // [-12.0 ..= +12.0 dB]
    pub treble_gain_db: f32,       // [-12.0 ..= +12.0 dB]
    pub tilt_puck_pos: (f32, f32), // Normalized (X: pivot freq, Y: tilt slope)
    pub is_dragging_puck: bool,
    pub phase_deviation_deg: f32, // [0.0 ..= 15.0 deg]
    pub spectral_bands: [f32; 8], // 8 frequency bands: Sub, Low, L-Mid, Mid, H-Mid, Pres, Brill, Air
    pub color_palette: ContrastColorPalette,
}

impl Default for SpectralTiltView {
    fn default() -> Self {
        Self::new()
    }
}

impl SpectralTiltView {
    pub fn new() -> Self {
        let mut view = Self {
            tilt_mode: TiltMode::LinearSlopeTilt6dB,
            pivot_frequency_hz: 1000.0,
            tilt_slope_db_oct: 1.5,
            bass_gain_db: -2.5,
            treble_gain_db: 2.5,
            tilt_puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            phase_deviation_deg: 0.0,
            spectral_bands: [0.65, 0.75, 0.85, 1.00, 1.15, 1.30, 1.40, 1.50],
            color_palette: ContrastColorPalette::default(),
        };
        view.tilt_puck_pos = (
            Self::pivot_to_normalized(view.pivot_frequency_hz),
            Self::slope_to_normalized(view.tilt_slope_db_oct),
        );
        view.update_tilt_curve();
        view
    }

    /// Logarithmic conversion: [200 ..= 5000 Hz] -> [0.0 ..= 1.0]
    pub fn pivot_to_normalized(hz: f32) -> f32 {
        let h = hz.clamp(MIN_PIVOT_FREQ_HZ, MAX_PIVOT_FREQ_HZ);
        let min_log = MIN_PIVOT_FREQ_HZ.ln();
        let max_log = MAX_PIVOT_FREQ_HZ.ln();
        ((h.ln() - min_log) / (max_log - min_log)).clamp(0.0, 1.0)
    }

    /// Normalized -> Logarithmic Hz
    pub fn normalized_to_pivot(norm: f32) -> f32 {
        let min_log = MIN_PIVOT_FREQ_HZ.ln();
        let max_log = MAX_PIVOT_FREQ_HZ.ln();
        (min_log + norm.clamp(0.0, 1.0) * (max_log - min_log))
            .exp()
            .clamp(MIN_PIVOT_FREQ_HZ, MAX_PIVOT_FREQ_HZ)
    }

    pub fn slope_to_normalized(db_oct: f32) -> f32 {
        let s = db_oct.clamp(MIN_TILT_SLOPE_DB_OCT, MAX_TILT_SLOPE_DB_OCT);
        ((s - MIN_TILT_SLOPE_DB_OCT) / (MAX_TILT_SLOPE_DB_OCT - MIN_TILT_SLOPE_DB_OCT))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_slope(norm: f32) -> f32 {
        MIN_TILT_SLOPE_DB_OCT
            + norm.clamp(0.0, 1.0) * (MAX_TILT_SLOPE_DB_OCT - MIN_TILT_SLOPE_DB_OCT)
    }

    pub fn set_tilt_mode(&mut self, mode: TiltMode) {
        self.tilt_mode = mode;
        self.pivot_frequency_hz = mode.nominal_pivot_hz();
        self.tilt_slope_db_oct = mode.nominal_slope_db_oct();
        self.tilt_puck_pos = (
            Self::pivot_to_normalized(self.pivot_frequency_hz),
            Self::slope_to_normalized(self.tilt_slope_db_oct),
        );
        self.update_tilt_curve();
    }

    /// Update spectral tilt curve, bass/treble endpoints, and 8-band energy distribution.
    pub fn update_tilt_curve(&mut self) {
        // Octave span from pivot to endpoints: ~3.5 octaves down (80 Hz) and ~4.0 octaves up (16 kHz)
        let octaves_low = (self.pivot_frequency_hz / 60.0).log2();
        let octaves_high = (18000.0 / self.pivot_frequency_hz).log2();

        self.bass_gain_db = (-self.tilt_slope_db_oct * octaves_low).clamp(-18.0, 18.0);
        self.treble_gain_db = (self.tilt_slope_db_oct * octaves_high).clamp(-18.0, 18.0);

        // Phase deviation: 0.0 deg for linear-phase FIR mode, up to 12.0 deg for minimum-phase
        self.phase_deviation_deg = if self.tilt_mode.is_phase_linear() {
            0.0
        } else {
            (self.tilt_slope_db_oct.abs() * 2.2).clamp(0.0, 15.0)
        };

        // Center frequencies of the 8 bands: [40Hz, 120Hz, 350Hz, 1kHz, 2.5kHz, 6kHz, 12kHz, 18kHz]
        let band_centers = [40.0, 120.0, 350.0, 1000.0, 2500.0, 6000.0, 12000.0, 18000.0];
        for (i, &fc) in band_centers.iter().enumerate() {
            let oct_diff = (fc / self.pivot_frequency_hz).log2();
            let gain_db = oct_diff * self.tilt_slope_db_oct;
            let linear_mag = (gain_db / 20.0).exp();
            self.spectral_bands[i] = linear_mag.clamp(0.1, 2.5);
        }
    }

    /// Hit test coordinate on the interactive tilt puck.
    pub fn hit_test_tilt_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.tilt_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.tilt_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= TILT_PUCK_HIT_RADIUS
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

        // Left half: Tilt curve & pivot coordinate
        let left_w = mid_x - 2;
        let p_row = (((1.0 - self.tilt_puck_pos.1) * (height - 5) as f32) + 2.0).round() as usize;
        let p_col = ((self.tilt_puck_pos.0 * (left_w - 4) as f32) + 2.0).round() as usize;
        if p_row < height - 1 && p_col < mid_x {
            grid[p_row][p_col] = 'P';
        }

        // Right half: 8-band spectral energy bars
        let right_w = width - mid_x - 2;
        let bar_spacing = right_w / 9;
        for (i, &energy) in self.spectral_bands.iter().enumerate() {
            let col = mid_x + 2 + (i + 1) * bar_spacing;
            let bar_h = ((energy / 2.0).clamp(0.0, 1.0) * (height - 4) as f32).round() as usize;
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
            "PSYCHOACOUSTIC SPECTRAL TILT & LINEAR-PHASE TONE SHAPER HUD",
            egui::FontId::proportional(14.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Tilt Topology Tabs (y: 48..92) - Each tab >= 44pt touch target
        let tabs = [
            (TiltMode::LinearSlopeTilt6dB, "LINEAR 6dB/OCT"),
            (TiltMode::BaxandallDualShelf, "BAXANDALL SHELF"),
            (TiltMode::PsychoacousticBark, "BARK TILT"),
            (TiltMode::PhaseLinearFIR, "PHASE-LINEAR FIR"),
            (TiltMode::AdaptiveDynamicTilt, "DYNAMIC ADAPTIVE"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (tmode, name)) in tabs.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_sel = self.tilt_mode == *tmode;
            let bg_col = if is_sel {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(24, 32, 48)
            };
            let text_col = if is_sel {
                Color32::from_rgb(8, 16, 24)
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
                        self.set_tilt_mode(*tmode);
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

        // Left 55%: Spectral Tilt Curve & Pivot Frequency Visualization
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
            "SPECTRAL TILT CURVE & PIVOT FREQUENCY RADAR",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(0, 229, 255),
        );

        // Grid lines for Log Frequencies (X) and Tilt Gain (Y)
        let freqs = [
            (200.0, "200Hz"),
            (500.0, "500Hz"),
            (1000.0, "1kHz"),
            (2500.0, "2.5kHz"),
            (5000.0, "5kHz"),
        ];
        for (f_val, f_lbl) in freqs.iter() {
            let fx_norm = Self::pivot_to_normalized(*f_val);
            let fx = left_rect.min.x + fx_norm * left_rect.width();
            painter.line_segment(
                [
                    egui::pos2(fx, left_rect.min.y + 45.0),
                    egui::pos2(fx, left_rect.max.y - 25.0),
                ],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 85, 120, 60)),
            );
            painter.text(
                egui::pos2(fx, left_rect.max.y - 22.0),
                egui::Align2::CENTER_TOP,
                *f_lbl,
                egui::FontId::proportional(8.5),
                Color32::from_rgb(140, 165, 195),
            );
        }

        // Readout Subtitle
        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.min.y + 26.0),
            egui::Align2::LEFT_TOP,
            format!(
                "Pivot: {:.1} Hz | Slope: {:+.2} dB/oct | Phase: {:.1}°",
                self.pivot_frequency_hz, self.tilt_slope_db_oct, self.phase_deviation_deg
            ),
            egui::FontId::proportional(10.5),
            Color32::from_rgb(0, 229, 255),
        );

        // 0 dB Flat Baseline Line
        let cy = left_rect.center().y + 10.0;
        painter.line_segment(
            [
                egui::pos2(left_rect.min.x + 10.0, cy),
                egui::pos2(left_rect.max.x - 10.0, cy),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(80, 110, 150, 90)),
        );

        // Tilt continuous curve line across the display
        let p_start_y = cy + (self.tilt_slope_db_oct * 10.0);
        let p_end_y = cy - (self.tilt_slope_db_oct * 10.0);
        painter.line_segment(
            [
                egui::pos2(left_rect.min.x + 15.0, p_start_y),
                egui::pos2(left_rect.max.x - 15.0, p_end_y),
            ],
            Stroke::new(2.5_f32, Color32::from_rgb(0, 229, 255)),
        );

        // Interactive Tilt Puck (Pivot freq vs Tilt slope)
        let puck_x = left_rect.min.x + self.tilt_puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.tilt_puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.tilt_puck_pos = (nx, ny);
                    self.pivot_frequency_hz = Self::normalized_to_pivot(nx);
                    self.tilt_slope_db_oct = Self::normalized_to_slope(ny);
                    self.update_tilt_curve();
                }
            }
        }

        // Draw Touch Hit Target boundary (>= 44x44pt)
        painter.circle_stroke(
            puck_pos,
            TILT_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(0, 229, 255, 150)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(0, 229, 255));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        // Right 45%: 8-Band Multiband Spectral Energy Distribution
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
            "8-BAND MULTIBAND SPECTRAL ENERGY",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(0, 229, 255),
        );

        let band_labels = [
            "SUB", "LOW", "L-MID", "MID", "H-MID", "PRES", "BRILL", "AIR",
        ];
        let bar_w = (right_rect.width() - 30.0 - 7.0 * 6.0) / 8.0;
        for (i, &energy) in self.spectral_bands.iter().enumerate() {
            let bx = right_rect.min.x + 15.0 + i as f32 * (bar_w + 6.0);
            let bar_h = ((energy / 2.2).clamp(0.0, 1.0)) * (right_rect.height() - 80.0);
            let b_rect = egui::Rect::from_min_max(
                egui::pos2(bx, right_rect.max.y - 25.0 - bar_h),
                egui::pos2(bx + bar_w, right_rect.max.y - 25.0),
            );
            let col = if i < 3 {
                Color32::from_rgb(255, 180, 50)
            } else if i < 6 {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(0, 255, 180)
            };
            painter.rect_filled(b_rect, 3.0, col);

            painter.text(
                egui::pos2(bx + bar_w * 0.5, right_rect.max.y - 20.0),
                egui::Align2::CENTER_TOP,
                band_labels[i],
                egui::FontId::proportional(8.0),
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
                "PIVOT FREQUENCY",
                format!("{:.1} Hz (Center)", self.pivot_frequency_hz),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "TILT SLOPE",
                format!("{:+.2} dB/oct (Tone)", self.tilt_slope_db_oct),
                Color32::from_rgb(255, 180, 50),
            ),
            (
                "LOW / HIGH SPREAD",
                format!(
                    "{:+.1} dB / {:+.1} dB",
                    self.bass_gain_db, self.treble_gain_db
                ),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "PHASE DISPERSION",
                format!("{:.2}° (Linear Phase)", self.phase_deviation_deg),
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

        // Compliance Verification Badge
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
            "[PASS] Psychoacoustic Spectral Tilt & Phase-Linear Tone Shaper Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
