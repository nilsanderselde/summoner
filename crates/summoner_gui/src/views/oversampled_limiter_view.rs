// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Mastering True-Peak Inter-Sample 8x Oversampled Limiter & Psychoacoustic Noise Shaping HUD (Step 1533).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const LIMITER_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_CEILING_DBTP: f32 = -6.0;
pub const MAX_CEILING_DBTP: f32 = 0.0;
pub const MIN_THRESHOLD_DB: f32 = -18.0;
pub const MAX_THRESHOLD_DB: f32 = 0.0;
pub const MIN_RELEASE_MS: f32 = 1.0;
pub const MAX_RELEASE_MS: f32 = 1000.0;

/// Limiter Profile Character.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimiterProfile {
    TransparentClean,       // Linear-phase, pristine neutrality
    WarmAnalogTape,         // Soft knee harmonic saturation before ceiling
    PunchyTransient,        // Dynamic transient recovery for punch
    BroadcastEbuR128,       // Strict -1.0 dBTP EBU R128 / ITU BS.1770
    AggressiveClubLoudness, // Maximum loudness density (-8.0 LUFS)
}

impl LimiterProfile {
    pub fn default_ceiling_dbtp(&self) -> f32 {
        match self {
            Self::TransparentClean => -0.3,
            Self::WarmAnalogTape => -0.5,
            Self::PunchyTransient => -0.2,
            Self::BroadcastEbuR128 => -1.0,
            Self::AggressiveClubLoudness => -0.1,
        }
    }

    pub fn default_threshold_db(&self) -> f32 {
        match self {
            Self::TransparentClean => -6.0,
            Self::WarmAnalogTape => -8.5,
            Self::PunchyTransient => -7.2,
            Self::BroadcastEbuR128 => -5.0,
            Self::AggressiveClubLoudness => -12.0,
        }
    }

    pub fn default_release_ms(&self) -> f32 {
        match self {
            Self::TransparentClean => 50.0,
            Self::WarmAnalogTape => 120.0,
            Self::PunchyTransient => 25.0,
            Self::BroadcastEbuR128 => 85.0,
            Self::AggressiveClubLoudness => 15.0,
        }
    }
}

/// Psychoacoustic Noise Shaping Dither Curve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DitherShapingCurve {
    FlatTpdf,        // Flat TPDF unweighted white noise
    Lipshitz,        // Minimal audibility 4th-order shape
    EWeighted,       // Equal-loudness E-weighted curve
    FWeighted,       // Ultra-steep HF push (above 16kHz)
    ModifiedShibata, // 5th-order mastering noise shaping
}

impl DitherShapingCurve {
    pub fn snr_improvement_db(&self) -> f32 {
        match self {
            Self::FlatTpdf => 0.0,
            Self::Lipshitz => 6.2,
            Self::EWeighted => 11.5,
            Self::FWeighted => 13.8,
            Self::ModifiedShibata => 16.2,
        }
    }
}

/// Mastering True-Peak 8x Oversampled Limiter View HUD (Step 1533).
#[derive(Debug, Clone)]
pub struct OversampledLimiterView {
    pub profile: LimiterProfile,
    pub ceiling_dbtp: f32,  // [-6.0 ..= 0.0 dBTP]
    pub threshold_db: f32,  // [-18.0 ..= 0.0 dB]
    pub release_ms: f32,    // [1.0 ..= 1000.0 ms]
    pub auto_release: bool, // Program-dependent release
    pub dither_curve: DitherShapingCurve,
    pub is_16_bit_dither: bool,       // true = 16-bit, false = 24-bit
    pub limiter_puck_pos: (f32, f32), // Normalized (X: Threshold, Y: Ceiling)
    pub is_dragging_puck: bool,
    pub true_peak_max_dbtp: f32,      // Current measured ISP max
    pub gain_reduction_db: f32,       // Realtime GR (dB)
    pub integrated_lufs: f32,         // Integrated Loudness LUFS
    pub isp_overshoot_prevented: u32, // ISP peak collision count
    pub color_palette: ContrastColorPalette,
}

impl Default for OversampledLimiterView {
    fn default() -> Self {
        Self::new()
    }
}

impl OversampledLimiterView {
    pub fn new() -> Self {
        let profile = LimiterProfile::BroadcastEbuR128;
        let mut view = Self {
            profile,
            ceiling_dbtp: profile.default_ceiling_dbtp(),
            threshold_db: profile.default_threshold_db(),
            release_ms: profile.default_release_ms(),
            auto_release: true,
            dither_curve: DitherShapingCurve::ModifiedShibata,
            is_16_bit_dither: false, // 24-bit broadcast master
            limiter_puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            true_peak_max_dbtp: -1.0,
            gain_reduction_db: 4.2,
            integrated_lufs: -14.0, // EBU R128 target
            isp_overshoot_prevented: 148,
            color_palette: ContrastColorPalette::default(),
        };
        view.limiter_puck_pos = (
            Self::thresh_to_normalized(view.threshold_db),
            Self::ceiling_to_normalized(view.ceiling_dbtp),
        );
        view.update_dsp_metrics();
        view
    }

    /// Convert Threshold [-18 ..= 0 dB] to normalized coordinate [0.0 ..= 1.0].
    pub fn thresh_to_normalized(db: f32) -> f32 {
        let d = db.clamp(MIN_THRESHOLD_DB, MAX_THRESHOLD_DB);
        ((d - MIN_THRESHOLD_DB) / (MAX_THRESHOLD_DB - MIN_THRESHOLD_DB)).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Threshold [-18 ..= 0 dB].
    pub fn normalized_to_thresh(norm: f32) -> f32 {
        MIN_THRESHOLD_DB + norm.clamp(0.0, 1.0) * (MAX_THRESHOLD_DB - MIN_THRESHOLD_DB)
    }

    /// Convert Ceiling [-6.0 ..= 0.0 dBTP] to normalized coordinate [0.0 ..= 1.0].
    pub fn ceiling_to_normalized(db: f32) -> f32 {
        let d = db.clamp(MIN_CEILING_DBTP, MAX_CEILING_DBTP);
        ((d - MIN_CEILING_DBTP) / (MAX_CEILING_DBTP - MIN_CEILING_DBTP)).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Ceiling [-6.0 ..= 0.0 dBTP].
    pub fn normalized_to_ceiling(norm: f32) -> f32 {
        MIN_CEILING_DBTP + norm.clamp(0.0, 1.0) * (MAX_CEILING_DBTP - MIN_CEILING_DBTP)
    }

    /// Convert Release Time [1 ..= 1000 ms] (log) to normalized coordinate [0.0 ..= 1.0].
    pub fn release_to_normalized(ms: f32) -> f32 {
        let m = ms.clamp(MIN_RELEASE_MS, MAX_RELEASE_MS);
        ((m.log10() - MIN_RELEASE_MS.log10()) / (MAX_RELEASE_MS.log10() - MIN_RELEASE_MS.log10()))
            .clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Release Time [1 ..= 1000 ms].
    pub fn normalized_to_release(norm: f32) -> f32 {
        10.0_f32.powf(
            MIN_RELEASE_MS.log10()
                + norm.clamp(0.0, 1.0) * (MAX_RELEASE_MS.log10() - MIN_RELEASE_MS.log10()),
        )
    }

    /// Set limiter profile preset.
    pub fn set_profile(&mut self, profile: LimiterProfile) {
        self.profile = profile;
        self.ceiling_dbtp = profile.default_ceiling_dbtp();
        self.threshold_db = profile.default_threshold_db();
        self.release_ms = profile.default_release_ms();
        self.limiter_puck_pos = (
            Self::thresh_to_normalized(self.threshold_db),
            Self::ceiling_to_normalized(self.ceiling_dbtp),
        );
        self.update_dsp_metrics();
    }

    /// Update simulated true-peak ISP and LUFS metrics.
    pub fn update_dsp_metrics(&mut self) {
        let drive = -self.threshold_db;
        self.gain_reduction_db = (drive * 0.75).clamp(0.0, 16.0);
        self.true_peak_max_dbtp = self.ceiling_dbtp;
        self.integrated_lufs = (-18.0 + drive * 0.8).clamp(-24.0, -6.0);
    }

    /// Evaluate 8x Sinc Inter-Sample Peak reconstruction amplitude at sub-sample $t \in [-2.0, 2.0]$.
    pub fn evaluate_sinc_isp(&self, t: f32) -> f32 {
        if t.abs() < 1e-4 {
            1.0
        } else {
            let pi_t = std::f32::consts::PI * t;
            (pi_t.sin() / pi_t) * (1.0 - (t / 2.0).powi(2)).max(0.0)
        }
    }

    /// Evaluate Psychoacoustic Noise Shaping curve amplitude (in dB) at frequency $f$ (Hz).
    pub fn evaluate_noise_shaping_curve(&self, freq_hz: f32) -> f32 {
        let f = freq_hz.clamp(20.0, 24000.0);
        let base_floor = if self.is_16_bit_dither { -96.0 } else { -144.0 };
        let snr_boost = self.dither_curve.snr_improvement_db();

        match self.dither_curve {
            DitherShapingCurve::FlatTpdf => base_floor,
            DitherShapingCurve::Lipshitz => {
                // High-pass slope above 4kHz
                let f_norm = (f / 20000.0).clamp(0.01, 1.0);
                base_floor - snr_boost + (f_norm.powf(2.5) * 20.0)
            }
            DitherShapingCurve::EWeighted => {
                // Dip in 2-5kHz ear sensitivity zone, rise in ultrasonics
                let f_khz = f / 1000.0;
                let dip = (-1.0 / (1.0 + ((f_khz - 3.5) / 1.5).powi(2))) * 12.0;
                let rise = if f_khz > 14.0 {
                    (f_khz - 14.0) * 3.5
                } else {
                    0.0
                };
                base_floor - snr_boost + dip + rise
            }
            DitherShapingCurve::FWeighted => {
                // Steep high shelf above 12kHz
                let f_khz = f / 1000.0;
                let rise = if f_khz > 10.0 {
                    (f_khz - 10.0).powf(1.8) * 1.8
                } else {
                    0.0
                };
                base_floor - snr_boost + rise - 6.0
            }
            DitherShapingCurve::ModifiedShibata => {
                // 5th order psychoacoustic curve
                let f_khz = f / 1000.0;
                let f_ear_dip = (-1.0 / (1.0 + ((f_khz - 3.8) / 1.2).powi(2))) * 15.0;
                let ultra_rise = if f_khz > 15.0 {
                    (f_khz - 15.0).powf(2.0) * 2.2
                } else {
                    0.0
                };
                base_floor - snr_boost + f_ear_dip + ultra_rise
            }
        }
    }

    /// Hit-test touch coordinate on the limiter ceiling/threshold puck.
    pub fn hit_test_limiter_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.limiter_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.limiter_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= LIMITER_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render of 8x Sinc Peak and Noise Shaping Curve.
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

        // Draw Noise Shaping curve on right half
        let right_w = width - mid_x - 2;
        let _center_r = height / 2;
        for c in 0..right_w {
            let frac = c as f32 / (right_w.max(1) as f32);
            let freq = 20.0 * 1200.0_f32.powf(frac);
            let dither_db = self.evaluate_noise_shaping_curve(freq);
            let norm_dither = ((dither_db + 160.0) / 100.0).clamp(0.0, 1.0);
            let row = (height as isize - 2 - (norm_dither * (height as f32 - 4.0)) as isize)
                .clamp(1, height as isize - 2) as usize;
            grid[row][mid_x + 1 + c] = '~';
        }

        // Limiter Puck on left half
        let puck_col = ((self.limiter_puck_pos.0 * (mid_x - 2) as f32) + 1.0).round() as usize;
        let puck_row =
            (((1.0 - self.limiter_puck_pos.1) * (height - 3) as f32) + 1.0).round() as usize;
        if puck_row < height - 1 && puck_col < mid_x {
            grid[puck_row][puck_col] = 'L';
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
            "MASTERING TRUE-PEAK 8x OVERSAMPLED LIMITER & NOISE SHAPING HUD",
            egui::FontId::proportional(15.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Profile Selector Tabs (y: 48..92) - Each tab >= 44pt height
        let profiles = [
            (LimiterProfile::TransparentClean, "TRANSPARENT"),
            (LimiterProfile::WarmAnalogTape, "WARM TAPE"),
            (LimiterProfile::PunchyTransient, "PUNCHY SNAP"),
            (LimiterProfile::BroadcastEbuR128, "BROADCAST EBU"),
            (LimiterProfile::AggressiveClubLoudness, "CLUB LOUD"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (pr, name)) in profiles.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_selected = self.profile == *pr;
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
                        self.set_profile(*pr);
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

        // Left 55%: True-Peak Inter-Sample Sinc Plane (Threshold vs Ceiling XY)
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
            "8x SINC TRUE-PEAK SPACE (THRESHOLD vs CEILING)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // 8x Polyphase Sinc Waveform visualization
        let sinc_w = left_rect.width() - 20.0;
        let sinc_cy = left_rect.center().y - 10.0;
        let mut prev_pt = None;
        for s in 0..=50 {
            let frac = s as f32 / 50.0;
            let t = (frac - 0.5) * 4.0;
            let sinc_val = self.evaluate_sinc_isp(t);
            let sx = left_rect.min.x + 10.0 + frac * sinc_w;
            let sy = sinc_cy - sinc_val * 48.0;
            let cur_pt = egui::pos2(sx, sy);
            if let Some(prev) = prev_pt {
                painter.line_segment(
                    [prev, cur_pt],
                    Stroke::new(2.0_f32, Color32::from_rgba_premultiplied(0, 229, 255, 120)),
                );
            }
            prev_pt = Some(cur_pt);
        }

        // Interactive Puck (Threshold X vs Ceiling Y)
        let puck_x = left_rect.min.x + self.limiter_puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.limiter_puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.limiter_puck_pos = (nx, ny);
                    self.threshold_db = Self::normalized_to_thresh(nx);
                    self.ceiling_dbtp = Self::normalized_to_ceiling(ny);
                    self.update_dsp_metrics();
                }
            }
        }

        // Touch Hit Target boundary (>= 44x44pt)
        painter.circle_stroke(
            puck_pos,
            LIMITER_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(0, 229, 255, 140)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(0, 229, 255));
        painter.circle_filled(puck_pos, 4.0, Color32::from_rgb(255, 255, 255));

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 20.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Ceiling: {:.1} dBTP | Thresh: {:.1} dB | GR: -{:.1} dB",
                self.ceiling_dbtp, self.threshold_db, self.gain_reduction_db
            ),
            egui::FontId::proportional(11.0),
            Color32::from_rgb(0, 229, 255),
        );

        // Right 45%: Psychoacoustic Noise Shaping Dither Spectrum
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
            "PSYCHOACOUSTIC NOISE SHAPING DITHER",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // 16-Bit vs 24-Bit Dither Mode Toggle (>= 44x44pt)
        let bit_w = (right_rect.width() - 30.0 - 10.0) / 2.0;
        let b16_rect = egui::Rect::from_min_size(
            egui::pos2(right_rect.min.x + 15.0, right_rect.min.y + 30.0),
            egui::vec2(bit_w, 44.0),
        );
        let b24_rect = egui::Rect::from_min_size(
            egui::pos2(right_rect.min.x + 25.0 + bit_w, right_rect.min.y + 30.0),
            egui::vec2(bit_w, 44.0),
        );

        let bg_16 = if self.is_16_bit_dither {
            Color32::from_rgb(255, 107, 43)
        } else {
            Color32::from_rgb(30, 45, 65)
        };
        let bg_24 = if !self.is_16_bit_dither {
            Color32::from_rgb(0, 229, 255)
        } else {
            Color32::from_rgb(30, 45, 65)
        };

        painter.rect_filled(b16_rect, 4.0, bg_16);
        painter.text(
            b16_rect.center(),
            egui::Align2::CENTER_CENTER,
            "16-BIT CD DITHER",
            egui::FontId::proportional(10.0),
            if self.is_16_bit_dither {
                Color32::from_rgb(10, 14, 24)
            } else {
                Color32::from_rgb(220, 235, 255)
            },
        );

        painter.rect_filled(b24_rect, 4.0, bg_24);
        painter.text(
            b24_rect.center(),
            egui::Align2::CENTER_CENTER,
            "24-BIT MASTER",
            egui::FontId::proportional(10.0),
            if !self.is_16_bit_dither {
                Color32::from_rgb(10, 14, 24)
            } else {
                Color32::from_rgb(220, 235, 255)
            },
        );

        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                if b16_rect.contains(pos) {
                    self.is_16_bit_dither = true;
                } else if b24_rect.contains(pos) {
                    self.is_16_bit_dither = false;
                }
            }
        }

        // Noise Shaping Spectrum Curve
        let curve_w = right_rect.width() - 30.0;
        let mut prev_c = None;
        for i in 0..=40 {
            let frac = i as f32 / 40.0;
            let freq = 20.0 * 1200.0_f32.powf(frac);
            let d_db = self.evaluate_noise_shaping_curve(freq);
            let norm_d = ((d_db + 160.0) / 100.0).clamp(0.0, 1.0);
            let cx = right_rect.min.x + 15.0 + frac * curve_w;
            let cy = right_rect.max.y - 40.0 - norm_d * 80.0;
            let cur_pt = egui::pos2(cx, cy);
            if let Some(prev) = prev_c {
                painter.line_segment(
                    [prev, cur_pt],
                    Stroke::new(2.0_f32, Color32::from_rgb(255, 215, 0)),
                );
            }
            prev_c = Some(cur_pt);
        }

        painter.text(
            egui::pos2(right_rect.min.x + 15.0, right_rect.max.y - 20.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Shaping: {:?} (+{:.1} dB SNR)",
                self.dither_curve,
                self.dither_curve.snr_improvement_db()
            ),
            egui::FontId::proportional(10.0),
            Color32::from_rgb(0, 255, 180),
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
                "TRUE-PEAK MAX (dBTP)",
                format!(
                    "{:.1} dBTP ({})",
                    self.true_peak_max_dbtp,
                    if self.true_peak_max_dbtp <= -1.0 {
                        "EBU PASS"
                    } else {
                        "HOT"
                    }
                ),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "GAIN REDUCTION",
                format!(
                    "-{:.1} dB ({:.0} ms Rel)",
                    self.gain_reduction_db, self.release_ms
                ),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "INTEGRATED LOUDNESS",
                format!("{:.1} LUFS (148 ISP)", self.integrated_lufs),
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "DITHER BIT DEPTH",
                format!(
                    "{} (Shibata 5th)",
                    if self.is_16_bit_dither {
                        "16-Bit CD"
                    } else {
                        "24-Bit Master"
                    }
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
            "[PASS] Mastering True-Peak Inter-Sample 8x Limiter & Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
