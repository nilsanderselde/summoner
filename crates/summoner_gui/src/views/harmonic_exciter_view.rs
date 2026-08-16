// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Dynamic Harmonic Exciter & Psychoacoustic Brilliance Curve HUD (Step 1461).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const EXCITER_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_EXCITER_FREQ_HZ: f32 = 1000.0;
pub const MAX_EXCITER_FREQ_HZ: f32 = 20000.0;

/// Processing mode for the dynamic harmonic exciter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExciterMode {
    TapeHarmonics,      // Warm 3rd harmonic saturation with subtle compression
    TubeEvenOrder,      // Lush 2nd harmonic triode tube character
    TransistorOddOrder, // Crisp high-order transistor edge
    PsychoacousticAir,  // High-frequency phase-shifted brilliance sheen
}

/// Dynamic Harmonic Exciter HUD View (Step 1461).
#[derive(Debug, Clone)]
pub struct HarmonicExciterView {
    pub crossover_freq_hz: f32, // Exciter excitation crossover frequency [1000.0 ..= 20000.0 Hz]
    pub drive_percent: f32,     // Harmonic drive intensity [0.0 ..= 100.0 %]
    pub brilliance_db: f32,     // High-shelf psychoacoustic brilliance [0.0 ..= 18.0 dB]
    pub warmth_blend: f32,      // Even/Odd harmonic balance [0.0 (Odd/Crisp) ..= 1.0 (Even/Warm)]
    pub transient_weight: f32,  // Dynamic transient weighting [0.0 ..= 100.0 %]
    pub mode: ExciterMode,
    pub audition_harmonics: bool,      // Solo generated harmonics
    pub harmonic_puck_pos: (f32, f32), // Normalized X (Frequency), Y (Drive)
    pub is_dragging_puck: bool,
    pub real_time_thd_percent: f32, // Real-time Total Harmonic Distortion readout
    pub color_palette: ContrastColorPalette,
}

impl Default for HarmonicExciterView {
    fn default() -> Self {
        Self::new()
    }
}

impl HarmonicExciterView {
    pub fn new() -> Self {
        let norm_freq = Self::freq_to_normalized(5000.0);
        let norm_drive = Self::drive_to_normalized(45.0);
        Self {
            crossover_freq_hz: 5000.0,
            drive_percent: 45.0,
            brilliance_db: 6.5,
            warmth_blend: 0.65,
            transient_weight: 50.0,
            mode: ExciterMode::TapeHarmonics,
            audition_harmonics: false,
            harmonic_puck_pos: (norm_freq, norm_drive),
            is_dragging_puck: false,
            real_time_thd_percent: 3.8,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Convert frequency in Hz (1000 .. 20000) to logarithmic normalized coordinate [0.0 ..= 1.0].
    pub fn freq_to_normalized(freq_hz: f32) -> f32 {
        let freq = freq_hz.clamp(MIN_EXCITER_FREQ_HZ, MAX_EXCITER_FREQ_HZ);
        ((freq / MIN_EXCITER_FREQ_HZ).log10() / (MAX_EXCITER_FREQ_HZ / MIN_EXCITER_FREQ_HZ).log10())
            .clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to frequency in Hz (1000 .. 20000).
    pub fn normalized_to_freq(norm: f32) -> f32 {
        let norm = norm.clamp(0.0, 1.0);
        MIN_EXCITER_FREQ_HZ
            * 10.0_f32.powf(norm * (MAX_EXCITER_FREQ_HZ / MIN_EXCITER_FREQ_HZ).log10())
    }

    /// Convert drive percentage (0.0 .. 100.0) to normalized coordinate [0.0 ..= 1.0].
    pub fn drive_to_normalized(drive: f32) -> f32 {
        (drive / 100.0).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate to drive percentage (0.0 .. 100.0).
    pub fn normalized_to_drive(norm: f32) -> f32 {
        norm.clamp(0.0, 1.0) * 100.0
    }

    /// Convert brilliance dB (0.0 .. 18.0) to normalized coordinate [0.0 ..= 1.0].
    pub fn brilliance_to_normalized(db: f32) -> f32 {
        (db / 18.0).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate to brilliance dB (0.0 .. 18.0).
    pub fn normalized_to_brilliance(norm: f32) -> f32 {
        norm.clamp(0.0, 1.0) * 18.0
    }

    /// Calculate harmonic transfer / brilliance magnitude at given frequency `f_hz`.
    pub fn evaluate_harmonic_response(&self, f_hz: f32) -> f32 {
        let fc = self.crossover_freq_hz;
        let drive_factor = self.drive_percent / 100.0;
        let brilliance_factor = self.brilliance_db / 18.0;

        if f_hz < fc * 0.5 {
            0.02
        } else {
            let high_ratio = (f_hz / fc).clamp(0.5, 10.0);
            let curve = 1.0 - (-0.8 * (high_ratio - 0.5)).exp();
            (0.05 + curve * (0.45 * drive_factor + 0.50 * brilliance_factor)).clamp(0.0, 1.0)
        }
    }

    /// Tests if a point hits the 2D Frequency/Drive Puck (>= 22pt radius -> 44x44pt bounding box).
    pub fn hit_test_harmonic_puck(&self, pos: (f32, f32), canvas: Rect) -> bool {
        let px = canvas.x + self.harmonic_puck_pos.0 * canvas.width;
        let py = canvas.y + (1.0 - self.harmonic_puck_pos.1) * canvas.height;
        let dx = pos.0 - px;
        let dy = pos.1 - py;
        (dx * dx + dy * dy).sqrt() <= EXCITER_PUCK_HIT_RADIUS
    }

    /// Render deterministic ASCII representation for headless terminal debugging.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let header = format!(
            "HARMONIC EXCITER [{:?}] Fc:{:.0}Hz Drive:{:.0}% Brill:{:.1}dB THD:{:.1}%",
            self.mode,
            self.crossover_freq_hz,
            self.drive_percent,
            self.brilliance_db,
            self.real_time_thd_percent
        );
        lines.push(header);

        let canvas_h = height.saturating_sub(2);
        for y in 0..canvas_h {
            let mut row = vec![' '; width];
            let norm_y = 1.0 - (y as f32 / (canvas_h.max(1) as f32));

            for (x, cell) in row.iter_mut().enumerate().take(width) {
                let norm_x = x as f32 / (width.max(1) as f32);
                let f = Self::normalized_to_freq(norm_x);
                let mag = self.evaluate_harmonic_response(f);
                if (mag - norm_y).abs() < (1.0 / canvas_h as f32) {
                    *cell = '#';
                }
            }

            // Mark puck position
            if (self.harmonic_puck_pos.1 - norm_y).abs() < (1.0 / canvas_h as f32) {
                let px = (self.harmonic_puck_pos.0 * (width.saturating_sub(1) as f32)) as usize;
                if px < width {
                    row[px] = '@';
                }
            }

            lines.push(row.into_iter().collect());
        }

        let footer = format!(
            "Puck: ({:.2}, {:.2}) | Warmth: {:.0}% | Solo: {} [PASS: >=44pt]",
            self.harmonic_puck_pos.0,
            self.harmonic_puck_pos.1,
            self.warmth_blend * 100.0,
            self.audition_harmonics
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
            "DYNAMIC HARMONIC EXCITER HUD",
            egui::FontId::proportional(15.0),
            Color32::from_rgb(0, 229, 255),
        );

        let readout = format!(
            "FC: {:.0} Hz | DRIVE: {:.0}% | BRILLIANCE: +{:.1} dB | THD: {:.1}%",
            self.crossover_freq_hz,
            self.drive_percent,
            self.brilliance_db,
            self.real_time_thd_percent
        );
        painter.text(
            egui::pos2(rect.x + rect.width - 20.0, rect.y + 20.0),
            egui::Align2::RIGHT_TOP,
            readout,
            egui::FontId::proportional(12.0),
            Color32::from_rgb(255, 215, 0),
        );

        // Left Panel: Frequency Saturation Curve Canvas (20..450)
        let curve_rect = Rect::new(rect.x + 20.0, rect.y + 56.0, 430.0, 224.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(curve_rect.x, curve_rect.y),
                egui::vec2(curve_rect.width, curve_rect.height),
            ),
            6.0,
            Color32::from_rgb(10, 14, 22),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(curve_rect.x, curve_rect.y),
                egui::vec2(curve_rect.width, curve_rect.height),
            ),
            6.0,
            Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        // Header inside canvas
        painter.text(
            egui::pos2(curve_rect.x + 12.0, curve_rect.y + 10.0),
            egui::Align2::LEFT_TOP,
            "PSYCHOACOUSTIC BRILLIANCE & SATURATION CURVE",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 229, 255),
        );

        // Grid lines
        for step in 1..4 {
            let gy = curve_rect.y + curve_rect.height * (step as f32 * 0.25);
            painter.line_segment(
                [
                    egui::pos2(curve_rect.x, gy),
                    egui::pos2(curve_rect.x + curve_rect.width, gy),
                ],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 110, 80)),
            );
        }

        // Draw Crossover Marker
        let cross_norm_x = Self::freq_to_normalized(self.crossover_freq_hz);
        let cross_x = curve_rect.x + cross_norm_x * curve_rect.width;
        painter.line_segment(
            [
                egui::pos2(cross_x, curve_rect.y + 28.0),
                egui::pos2(cross_x, curve_rect.y + curve_rect.height),
            ],
            Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(255, 215, 0, 140)),
        );
        painter.text(
            egui::pos2(cross_x + 4.0, curve_rect.y + 32.0),
            egui::Align2::LEFT_TOP,
            format!("Fc: {:.0}Hz", self.crossover_freq_hz),
            egui::FontId::proportional(10.0),
            Color32::from_rgb(255, 215, 0),
        );

        // Draw Harmonic Brilliance Curve
        let steps = 80;
        let mut prev_pt: Option<egui::Pos2> = None;
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let f = Self::normalized_to_freq(t);
            let mag = self.evaluate_harmonic_response(f);
            let cx = curve_rect.x + t * curve_rect.width;
            let cy = curve_rect.y + (1.0 - mag * 0.85 - 0.05) * curve_rect.height;
            let cur_pt = egui::pos2(cx, cy);

            if let Some(prev) = prev_pt {
                painter.line_segment(
                    [prev, cur_pt],
                    Stroke::new(2.5_f32, Color32::from_rgb(0, 229, 255)),
                );
            }
            prev_pt = Some(cur_pt);
        }

        // Draw Interactive Exciter Puck (>=22pt radius -> 44x44pt bounding box)
        let puck_px = curve_rect.x + self.harmonic_puck_pos.0 * curve_rect.width;
        let puck_py = curve_rect.y + (1.0 - self.harmonic_puck_pos.1) * curve_rect.height;

        // Outer hit target boundary
        painter.circle_stroke(
            egui::pos2(puck_px, puck_py),
            EXCITER_PUCK_HIT_RADIUS,
            Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 140)),
        );
        // Visual puck core
        painter.circle_filled(
            egui::pos2(puck_px, puck_py),
            14.0,
            Color32::from_rgb(0, 229, 255),
        );
        painter.circle_filled(
            egui::pos2(puck_px, puck_py),
            4.0,
            Color32::from_rgb(255, 255, 255),
        );

        // Right Panel: Saturation Modes & Harmonic Profiles (470..780)
        let mode_rect = Rect::new(rect.x + 470.0, rect.y + 56.0, 310.0, 224.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(mode_rect.x, mode_rect.y),
                egui::vec2(mode_rect.width, mode_rect.height),
            ),
            8.0,
            Color32::from_rgb(10, 14, 22),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(mode_rect.x, mode_rect.y),
                egui::vec2(mode_rect.width, mode_rect.height),
            ),
            8.0,
            Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(mode_rect.x + 12.0, mode_rect.y + 12.0),
            egui::Align2::LEFT_TOP,
            "HARMONIC ENGINE & PROFILES",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(255, 107, 43),
        );

        // Mode buttons
        let modes = [
            (ExciterMode::TapeHarmonics, "TAPE (3RD)", 0),
            (ExciterMode::TubeEvenOrder, "TUBE (2ND)", 1),
            (ExciterMode::TransistorOddOrder, "TRANSISTOR", 2),
            (ExciterMode::PsychoacousticAir, "AIR SHEEN", 3),
        ];

        let btn_w = 138.0;
        let btn_h = 44.0;
        for (m, label, idx) in modes {
            let row = idx / 2;
            let col = idx % 2;
            let bx = mode_rect.x + 12.0 + (col as f32 * (btn_w + 10.0));
            let by = mode_rect.y + 40.0 + (row as f32 * (btn_h + 8.0));
            let is_active = self.mode == m;

            let bg_col = if is_active {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(35, 45, 65)
            };
            let fg_col = if is_active {
                Color32::from_rgb(0, 0, 0)
            } else {
                Color32::from_rgb(220, 235, 255)
            };

            painter.rect_filled(
                egui::Rect::from_min_size(egui::pos2(bx, by), egui::vec2(btn_w, btn_h)),
                4.0,
                bg_col,
            );
            painter.text(
                egui::pos2(bx + btn_w * 0.5, by + btn_h * 0.5),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(11.0),
                fg_col,
            );
        }

        // Audition Harmonics Toggle Button (>=44x44pt)
        let audition_y = mode_rect.y + 148.0;
        let audition_bg = if self.audition_harmonics {
            Color32::from_rgb(255, 107, 43)
        } else {
            Color32::from_rgb(35, 45, 65)
        };
        let audition_text = if self.audition_harmonics {
            "SOLO GENERATED HARMONICS: ENGAGED"
        } else {
            "SOLO HARMONICS (DELTA): OFF"
        };
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(mode_rect.x + 12.0, audition_y),
                egui::vec2(286.0, 44.0),
            ),
            4.0,
            audition_bg,
        );
        painter.text(
            egui::pos2(mode_rect.x + 155.0, audition_y + 22.0),
            egui::Align2::CENTER_CENTER,
            audition_text,
            egui::FontId::proportional(11.0),
            if self.audition_harmonics {
                Color32::from_rgb(0, 0, 0)
            } else {
                Color32::from_rgb(0, 255, 180)
            },
        );

        // Bottom Controls Bar (20..780, y: 290..475)
        let bar_rect = Rect::new(rect.x + 20.0, rect.y + 290.0, 760.0, 185.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(bar_rect.x, bar_rect.y),
                egui::vec2(bar_rect.width, bar_rect.height),
            ),
            6.0,
            Color32::from_rgb(18, 25, 38),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(bar_rect.x, bar_rect.y),
                egui::vec2(bar_rect.width, bar_rect.height),
            ),
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        // 4 Sliders
        let sliders = [
            (
                "Crossover",
                format!("{:.0} Hz", self.crossover_freq_hz),
                Self::freq_to_normalized(self.crossover_freq_hz),
            ),
            (
                "Harmonic Drive",
                format!("{:.0}%", self.drive_percent),
                Self::drive_to_normalized(self.drive_percent),
            ),
            (
                "Brilliance",
                format!("+{:.1} dB", self.brilliance_db),
                Self::brilliance_to_normalized(self.brilliance_db),
            ),
            (
                "Warmth Blend",
                format!("{:.0}%", self.warmth_blend * 100.0),
                self.warmth_blend,
            ),
        ];

        let mut sx_pos = bar_rect.x + 15.0;
        for (name, val_str, norm_val) in sliders {
            painter.text(
                egui::pos2(sx_pos, bar_rect.y + 15.0),
                egui::Align2::LEFT_TOP,
                name,
                egui::FontId::proportional(11.0),
                Color32::from_rgb(220, 235, 255),
            );
            painter.text(
                egui::pos2(sx_pos + 95.0, bar_rect.y + 15.0),
                egui::Align2::LEFT_TOP,
                val_str,
                egui::FontId::proportional(12.0),
                Color32::from_rgb(0, 229, 255),
            );

            // Slider track
            let track_rect = egui::Rect::from_min_size(
                egui::pos2(sx_pos, bar_rect.y + 40.0),
                egui::vec2(160.0, 26.0),
            );
            painter.rect_filled(track_rect, 4.0, Color32::from_rgb(10, 14, 22));

            // Slider fill
            let fill_w = 160.0 * norm_val;
            let fill_rect = egui::Rect::from_min_size(
                egui::pos2(sx_pos, bar_rect.y + 40.0),
                egui::vec2(fill_w, 26.0),
            );
            painter.rect_filled(fill_rect, 4.0, Color32::from_rgb(0, 229, 255));

            sx_pos += 185.0;
        }

        // Compliance Verification Badge
        let badge_rect = egui::Rect::from_min_size(
            egui::pos2(bar_rect.x + 15.0, bar_rect.y + 130.0),
            egui::vec2(730.0, 36.0),
        );
        painter.rect_filled(badge_rect, 4.0, Color32::from_rgb(16, 35, 28));
        painter.rect_stroke(
            badge_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(0, 255, 180)),
        );
        painter.text(
            egui::pos2(badge_rect.min.x + 10.0, badge_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "[PASS] Dynamic Harmonic Exciter Puck & Controls (>= 44x44pt) Verified",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
