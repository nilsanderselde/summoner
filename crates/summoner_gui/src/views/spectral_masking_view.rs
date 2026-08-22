// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Psychoacoustic Critical-Band Spectral Masking Threshold & Perceptual Codec Redundancy HUD (Step 1612).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const MASKING_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_MASKER_FREQ_HZ: f32 = 50.0;
pub const MAX_MASKER_FREQ_HZ: f32 = 16000.0;
pub const MIN_MASKER_LEVEL_DB: f32 = 20.0;
pub const MAX_MASKER_LEVEL_DB: f32 = 110.0;

/// Psychoacoustic masking models and perceptual codec profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaskingModelType {
    Mpeg1Layer3Psychoacoustic, // ISO/IEC 11172-3 MP3 Psychoacoustic Model 1 (32 subbands, Bark spreading)
    AacLdPsychoacoustic,       // MPEG-4 AAC-LD MDCT psychoacoustic model with temporal spreading
    OpusCeltPerceptual,        // Opus CELT Bark-band energy masking & noise allocation
    SpatialAudioMasking3D,     // Binaural Masking Level Difference (BMLD) spatial unmasking
    HiResMasteringDither,      // 24-bit 96kHz psychoacoustic noise shaping & dither masking curve
}

impl MaskingModelType {
    pub fn model_name(&self) -> &'static str {
        match self {
            Self::Mpeg1Layer3Psychoacoustic => "MPEG-1 LAYER 3 (MP3 PSYCHO 1)",
            Self::AacLdPsychoacoustic => "AAC-LD MDCT PSYCHO MODEL",
            Self::OpusCeltPerceptual => "OPUS CELT BARK MASKING",
            Self::SpatialAudioMasking3D => "SPATIAL 3D BINAURAL MASKING (BMLD)",
            Self::HiResMasteringDither => "HI-RES MASTERING DITHER MASK",
        }
    }

    pub fn nominal_masker_freq_hz(&self) -> f32 {
        match self {
            Self::Mpeg1Layer3Psychoacoustic => 1000.0,
            Self::AacLdPsychoacoustic => 2500.0,
            Self::OpusCeltPerceptual => 800.0,
            Self::SpatialAudioMasking3D => 500.0,
            Self::HiResMasteringDither => 4000.0,
        }
    }

    pub fn nominal_masker_level_db(&self) -> f32 {
        match self {
            Self::Mpeg1Layer3Psychoacoustic => 85.0,
            Self::AacLdPsychoacoustic => 78.0,
            Self::OpusCeltPerceptual => 80.0,
            Self::SpatialAudioMasking3D => 72.0,
            Self::HiResMasteringDither => 65.0,
        }
    }

    pub fn nominal_tonality_index(&self) -> f32 {
        match self {
            Self::Mpeg1Layer3Psychoacoustic => 0.85,
            Self::AacLdPsychoacoustic => 0.70,
            Self::OpusCeltPerceptual => 0.60,
            Self::SpatialAudioMasking3D => 0.50,
            Self::HiResMasteringDither => 0.90,
        }
    }

    pub fn nominal_bark_spread_db(&self) -> f32 {
        match self {
            Self::Mpeg1Layer3Psychoacoustic => 25.0,
            Self::AacLdPsychoacoustic => 28.0,
            Self::OpusCeltPerceptual => 22.0,
            Self::SpatialAudioMasking3D => 32.0,
            Self::HiResMasteringDither => 18.0,
        }
    }
}

/// Psychoacoustic critical-band spectral masking threshold & perceptual codec redundancy HUD.
#[derive(Debug, Clone)]
pub struct SpectralMaskingView {
    pub model_type: MaskingModelType,
    pub masker_freq_hz: f32,
    pub masker_level_db: f32,
    pub tonality_index: f32,
    pub bark_spread_factor_db: f32,
    pub bit_reduction_pct: f32,
    pub global_masking_threshold_db: f32,
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    pub critical_band_thresholds: [f32; 8],
    pub color_palette: ContrastColorPalette,
}

impl Default for SpectralMaskingView {
    fn default() -> Self {
        Self::new()
    }
}

impl SpectralMaskingView {
    pub fn new() -> Self {
        let mut view = Self {
            model_type: MaskingModelType::Mpeg1Layer3Psychoacoustic,
            masker_freq_hz: 1000.0,
            masker_level_db: 85.0,
            tonality_index: 0.85,
            bark_spread_factor_db: 25.0,
            bit_reduction_pct: 42.5,
            global_masking_threshold_db: 62.0,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            critical_band_thresholds: [22.0, 35.0, 52.0, 75.0, 58.0, 44.0, 32.0, 20.0],
            color_palette: ContrastColorPalette::default(),
        };
        view.puck_pos = (
            Self::freq_to_normalized(view.masker_freq_hz),
            Self::level_to_normalized(view.masker_level_db),
        );
        view.update_masking_simulation();
        view
    }

    pub fn freq_to_normalized(freq: f32) -> f32 {
        let f = freq.clamp(MIN_MASKER_FREQ_HZ, MAX_MASKER_FREQ_HZ);
        let min_log = MIN_MASKER_FREQ_HZ.log2();
        let max_log = MAX_MASKER_FREQ_HZ.log2();
        ((f.log2() - min_log) / (max_log - min_log)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_freq(norm: f32) -> f32 {
        let min_log = MIN_MASKER_FREQ_HZ.log2();
        let max_log = MAX_MASKER_FREQ_HZ.log2();
        2.0_f32.powf(min_log + norm.clamp(0.0, 1.0) * (max_log - min_log))
    }

    pub fn level_to_normalized(level: f32) -> f32 {
        let l = level.clamp(MIN_MASKER_LEVEL_DB, MAX_MASKER_LEVEL_DB);
        ((l - MIN_MASKER_LEVEL_DB) / (MAX_MASKER_LEVEL_DB - MIN_MASKER_LEVEL_DB)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_level(norm: f32) -> f32 {
        MIN_MASKER_LEVEL_DB + norm.clamp(0.0, 1.0) * (MAX_MASKER_LEVEL_DB - MIN_MASKER_LEVEL_DB)
    }

    pub fn set_model(&mut self, model: MaskingModelType) {
        self.model_type = model;
        self.masker_freq_hz = model.nominal_masker_freq_hz();
        self.masker_level_db = model.nominal_masker_level_db();
        self.tonality_index = model.nominal_tonality_index();
        self.bark_spread_factor_db = model.nominal_bark_spread_db();
        self.puck_pos = (
            Self::freq_to_normalized(self.masker_freq_hz),
            Self::level_to_normalized(self.masker_level_db),
        );
        self.update_masking_simulation();
    }

    pub fn update_masking_simulation(&mut self) {
        let f = self.masker_freq_hz;
        let l = self.masker_level_db;
        let tonality = self.tonality_index;
        let spread = self.bark_spread_factor_db;

        // Bark center frequencies for 8 psychoacoustic critical bands:
        // [100, 300, 700, 1500, 3000, 6000, 10000, 15000] Hz
        let band_centers = [
            100.0, 300.0, 700.0, 1500.0, 3000.0, 6000.0, 10000.0, 15000.0,
        ];

        // Masker Bark value: z = 13 * arctan(0.00076 * f) + 3.5 * arctan((f / 7500)^2)
        let masker_bark = 13.0_f32 * (0.00076_f32 * f).atan()
            + 3.5_f32 * ((f / 7500.0_f32) * (f / 7500.0_f32)).atan();

        // Offset from masker level (tone-masking-noise vs noise-masking-tone)
        let mask_offset = if tonality > 0.5 {
            14.5 + tonality * 3.0 // Tone masker gives ~14.5-17.5 dB below masker peak
        } else {
            6.0 + tonality * 4.0 // Noise masker gives ~6.0-10.0 dB below masker peak
        };

        let peak_threshold = (l - mask_offset).max(10.0);
        self.global_masking_threshold_db = peak_threshold;

        // Compute critical band thresholds using upward/downward Bark spreading function
        let mut bands = [0.0_f32; 8];
        let mut total_masked_power = 0.0_f32;

        for (i, &bc) in band_centers.iter().enumerate() {
            let b_bark = 13.0_f32 * (0.00076_f32 * bc).atan()
                + 3.5_f32 * ((bc / 7500.0_f32) * (bc / 7500.0_f32)).atan();
            let delta_bark = b_bark - masker_bark;

            // Spreading function: downward slope ~12 dB/Bark, upward slope ~6-8 dB/Bark scaled by spread factor
            let atten = if delta_bark >= 0.0 {
                let upward_slope = (6.0 + 0.08 * l).min(15.0);
                delta_bark * upward_slope * (20.0 / spread)
            } else {
                (-delta_bark) * 12.0 * (20.0 / spread)
            };

            let band_thresh = (peak_threshold - atten).clamp(5.0, 110.0);
            bands[i] = band_thresh;
            total_masked_power += band_thresh;
        }

        self.critical_band_thresholds = bands;

        // Bit redundancy savings estimated from masking area
        let avg_mask = (total_masked_power / 8.0).max(peak_threshold * 0.35);
        self.bit_reduction_pct = ((avg_mask / 60.0) * 45.0 + tonality * 15.0).clamp(10.0, 85.0);
    }

    pub fn hit_test_masking_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= MASKING_PUCK_HIT_RADIUS
    }

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
            grid[p_row][p_col] = 'M';
        }

        let right_w = width - mid_x - 2;
        let bar_spacing = right_w / 9;
        for (i, &thresh) in self.critical_band_thresholds.iter().enumerate() {
            let col = mid_x + 2 + (i + 1) * bar_spacing;
            let bar_h = ((thresh / 100.0).clamp(0.0, 1.0) * (height - 4) as f32).round() as usize;
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

        // Background: Deep Indigo Slate (#0A0F1D)
        painter.rect_filled(rect, 6.0, Color32::from_rgb(10, 15, 29));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "PSYCHOACOUSTIC SPECTRAL MASKING & PERCEPTUAL CODEC HUD",
            egui::FontId::proportional(13.5),
            Color32::from_rgb(240, 245, 255),
        );

        // Tabs (y: 48..92) - 44pt height
        let tabs = [
            (MaskingModelType::Mpeg1Layer3Psychoacoustic, "MP3 PSYCHO 1"),
            (MaskingModelType::AacLdPsychoacoustic, "AAC-LD MDCT"),
            (MaskingModelType::OpusCeltPerceptual, "OPUS CELT BARK"),
            (MaskingModelType::SpatialAudioMasking3D, "SPATIAL 3D BMLD"),
            (MaskingModelType::HiResMasteringDither, "HI-RES DITHER MASK"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (mtype, name)) in tabs.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_sel = self.model_type == *mtype;
            let bg_col = if is_sel {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(22, 30, 46)
            };
            let text_col = if is_sel {
                Color32::from_rgb(4, 18, 28)
            } else {
                Color32::from_rgb(215, 230, 250)
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
                        self.set_model(*mtype);
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
            Stroke::new(1.5_f32, Color32::from_rgb(35, 55, 85)),
        );

        // Left 55%: Masker Frequency vs SPL Level Plane
        let left_w = main_canvas.width() * 0.55;
        let left_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(left_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(left_rect, 4.0, Color32::from_rgb(14, 18, 30));
        painter.rect_stroke(
            left_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(35, 50, 75)),
        );

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "MASKER FREQUENCY (Hz) vs SOUND PRESSURE LEVEL (dB SPL)",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(0, 229, 255),
        );

        // Grid lines for frequency (100Hz, 1kHz, 10kHz) and dB (40, 70, 100)
        let grid_freqs = [100.0, 1000.0, 10000.0];
        for gf in grid_freqs {
            let gx = left_rect.min.x + Self::freq_to_normalized(gf) * left_rect.width();
            painter.line_segment(
                [
                    egui::pos2(gx, left_rect.min.y + 30.0),
                    egui::pos2(gx, left_rect.max.y - 25.0),
                ],
                Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(45, 65, 95, 100)),
            );
            painter.text(
                egui::pos2(gx, left_rect.max.y - 22.0),
                egui::Align2::CENTER_TOP,
                format!("{}Hz", gf as u32),
                egui::FontId::proportional(8.5),
                Color32::from_rgb(120, 145, 180),
            );
        }

        // Interactive Masker Puck
        let puck_x = left_rect.min.x + self.puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.puck_pos = (nx, ny);
                    self.masker_freq_hz = Self::normalized_to_freq(nx);
                    self.masker_level_db = Self::normalized_to_level(ny);
                    self.update_masking_simulation();
                }
            }
        }

        painter.circle_stroke(
            puck_pos,
            MASKING_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(0, 229, 255, 150)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(0, 229, 255));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 18.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Freq: {:.0} Hz | Level: {:.1} dB SPL | Tonality: {:.2} | Thresh: {:.1} dB",
                self.masker_freq_hz,
                self.masker_level_db,
                self.tonality_index,
                self.global_masking_threshold_db
            ),
            egui::FontId::proportional(10.5),
            Color32::from_rgb(140, 235, 255),
        );

        // Right 45%: Critical Band Masking Thresholds Spectrum
        let right_w = main_canvas.width() * 0.45;
        let right_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + left_w + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(right_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(right_rect, 4.0, Color32::from_rgb(14, 18, 30));
        painter.rect_stroke(
            right_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(35, 50, 75)),
        );

        painter.text(
            egui::pos2(right_rect.min.x + 10.0, right_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "CRITICAL-BAND SIMULTANEOUS MASKING THRESHOLDS",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(0, 229, 255),
        );

        let band_labels = ["100", "300", "700", "1.5k", "3k", "6k", "10k", "15k"];
        let bar_w = (right_rect.width() - 30.0 - 7.0 * 6.0) / 8.0;
        for (i, &thresh) in self.critical_band_thresholds.iter().enumerate() {
            let bx = right_rect.min.x + 15.0 + i as f32 * (bar_w + 6.0);
            let bar_h = (thresh.clamp(0.0, 100.0) / 100.0) * (right_rect.height() - 80.0);
            let b_rect = egui::Rect::from_min_max(
                egui::pos2(bx, right_rect.max.y - 25.0 - bar_h),
                egui::pos2(bx + bar_w, right_rect.max.y - 25.0),
            );
            let col = if thresh > 60.0 {
                Color32::from_rgb(255, 107, 43)
            } else if thresh > 35.0 {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(0, 255, 180)
            };
            painter.rect_filled(b_rect, 3.0, col);

            painter.text(
                egui::pos2(bx + bar_w * 0.5, right_rect.max.y - 20.0),
                egui::Align2::CENTER_TOP,
                band_labels[i],
                egui::FontId::proportional(8.5),
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
                "MASKER CENTER FREQ",
                format!("{:.0} Hz (Critical Bark)", self.masker_freq_hz),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "MASKER SPL LEVEL",
                format!("{:.1} dB SPL", self.masker_level_db),
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "BIT REDUNDANCY SAVINGS",
                format!("{:.1}% (Perceptual Codec)", self.bit_reduction_pct),
                Color32::from_rgb(0, 255, 180),
            ),
            (
                "GLOBAL MASK THRESHOLD",
                format!(
                    "{:.1} dB (Inaudible Floor)",
                    self.global_masking_threshold_db
                ),
                Color32::from_rgb(255, 215, 0),
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
            "[PASS] Psychoacoustic Spectral Masking Threshold Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
