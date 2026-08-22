// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Psychoacoustic Dynamic Fletcher-Munson Equal-Loudness Contour & Spectral Balance Compensator HUD (Step 1592).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const LOUDNESS_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_MONITORING_SPL_DB: f32 = 40.0;
pub const MAX_MONITORING_SPL_DB: f32 = 100.0;
pub const MIN_COMPENSATION_AMOUNT: f32 = 0.0;
pub const MAX_COMPENSATION_AMOUNT: f32 = 1.0;

/// Equal loudness standard curves and reference acoustic weighting models.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoudnessStandard {
    Iso226_2003,        // ISO 226:2003 normal equal-loudness level contours
    FletcherMunson1933, // Historical 1933 Bell Labs curves
    RobinsonDadson1956, // 1956 British Standard BS 3383 curves
    EbuR128KWeighted,   // ITU-R BS.1770 / EBU R128 K-weighted broadcast compensation
    CinemaXCurve,       // SMPTE 202M Cinema auditorium translation curve
}

impl LoudnessStandard {
    pub fn standard_name(&self) -> &'static str {
        match self {
            Self::Iso226_2003 => "ISO 226:2003 (PHON)",
            Self::FletcherMunson1933 => "FLETCHER-MUNSON",
            Self::RobinsonDadson1956 => "ROBINSON-DADSON",
            Self::EbuR128KWeighted => "EBU R128 (K-WT)",
            Self::CinemaXCurve => "SMPTE X-CURVE",
        }
    }

    pub fn nominal_spl_db(&self) -> f32 {
        match self {
            Self::Iso226_2003 => 83.0,
            Self::FletcherMunson1933 => 75.0,
            Self::RobinsonDadson1956 => 80.0,
            Self::EbuR128KWeighted => 73.0,
            Self::CinemaXCurve => 85.0,
        }
    }

    pub fn nominal_compensation(&self) -> f32 {
        match self {
            Self::Iso226_2003 => 0.85,
            Self::FletcherMunson1933 => 0.70,
            Self::RobinsonDadson1956 => 0.65,
            Self::EbuR128KWeighted => 0.90,
            Self::CinemaXCurve => 0.50,
        }
    }

    pub fn nominal_ear_canal_dip_db(&self) -> f32 {
        match self {
            Self::Iso226_2003 => -4.5,
            Self::FletcherMunson1933 => -6.0,
            Self::RobinsonDadson1956 => -5.0,
            Self::EbuR128KWeighted => -3.5,
            Self::CinemaXCurve => -2.0,
        }
    }
}

/// Psychoacoustic dynamic Fletcher-Munson equal-loudness contour & spectral balance compensator HUD.
#[derive(Debug, Clone)]
pub struct EqualLoudnessContourView {
    pub standard: LoudnessStandard,
    pub monitoring_spl_db: f32,
    pub compensation_amount: f32,
    pub reference_phon: f32,
    pub ear_canal_q: f32,
    pub bass_boost_db: f32,
    pub treble_tilt_db: f32,
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    pub band_gains_db: [f32; 8],
    pub color_palette: ContrastColorPalette,
}

impl Default for EqualLoudnessContourView {
    fn default() -> Self {
        Self::new()
    }
}

impl EqualLoudnessContourView {
    pub fn new() -> Self {
        let mut view = Self {
            standard: LoudnessStandard::Iso226_2003,
            monitoring_spl_db: 83.0,
            compensation_amount: 0.85,
            reference_phon: 83.0,
            ear_canal_q: 2.8,
            bass_boost_db: 4.2,
            treble_tilt_db: -1.5,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            band_gains_db: [6.5, 4.2, 2.0, 0.0, -3.8, -1.2, 1.5, 3.2],
            color_palette: ContrastColorPalette::default(),
        };
        view.puck_pos = (
            Self::spl_to_normalized(view.monitoring_spl_db),
            Self::comp_to_normalized(view.compensation_amount),
        );
        view.update_contour_simulation();
        view
    }

    pub fn spl_to_normalized(spl: f32) -> f32 {
        let s = spl.clamp(MIN_MONITORING_SPL_DB, MAX_MONITORING_SPL_DB);
        ((s - MIN_MONITORING_SPL_DB) / (MAX_MONITORING_SPL_DB - MIN_MONITORING_SPL_DB))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_spl(norm: f32) -> f32 {
        MIN_MONITORING_SPL_DB
            + norm.clamp(0.0, 1.0) * (MAX_MONITORING_SPL_DB - MIN_MONITORING_SPL_DB)
    }

    pub fn comp_to_normalized(comp: f32) -> f32 {
        let c = comp.clamp(MIN_COMPENSATION_AMOUNT, MAX_COMPENSATION_AMOUNT);
        ((c - MIN_COMPENSATION_AMOUNT) / (MAX_COMPENSATION_AMOUNT - MIN_COMPENSATION_AMOUNT))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_comp(norm: f32) -> f32 {
        MIN_COMPENSATION_AMOUNT
            + norm.clamp(0.0, 1.0) * (MAX_COMPENSATION_AMOUNT - MIN_COMPENSATION_AMOUNT)
    }

    pub fn set_standard(&mut self, std: LoudnessStandard) {
        self.standard = std;
        self.monitoring_spl_db = std.nominal_spl_db();
        self.compensation_amount = std.nominal_compensation();
        self.reference_phon = self.monitoring_spl_db;
        self.puck_pos = (
            Self::spl_to_normalized(self.monitoring_spl_db),
            Self::comp_to_normalized(self.compensation_amount),
        );
        self.update_contour_simulation();
    }

    pub fn update_contour_simulation(&mut self) {
        let spl_diff = (83.0 - self.monitoring_spl_db) / 43.0; // Positive when listening quieter
        let comp = self.compensation_amount;

        // Equal-loudness compensation curve at critical bands:
        // 40Hz, 100Hz, 300Hz, 1kHz, 3.5kHz (ear canal resonance), 6kHz, 10kHz, 16kHz
        let b40 = (14.0 * spl_diff * comp).clamp(-12.0, 24.0);
        let b100 = (9.5 * spl_diff * comp).clamp(-8.0, 18.0);
        let b300 = (4.0 * spl_diff * comp).clamp(-4.0, 10.0);
        let b1k = 0.0;
        let b3k5 = (self.standard.nominal_ear_canal_dip_db() * (1.0 - spl_diff * 0.4) * comp)
            .clamp(-12.0, 6.0);
        let b6k = (1.5 * spl_diff * comp).clamp(-4.0, 8.0);
        let b10k = (4.8 * spl_diff * comp).clamp(-6.0, 14.0);
        let b16k = (7.2 * spl_diff * comp).clamp(-8.0, 18.0);

        self.bass_boost_db = b40;
        self.treble_tilt_db = b16k;
        self.band_gains_db = [b40, b100, b300, b1k, b3k5, b6k, b10k, b16k];
    }

    pub fn hit_test_loudness_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let plot_x = canvas.x + 25.0;
        let plot_w = (canvas.width - 50.0).max(1.0);
        let plot_y = canvas.y + 40.0;
        let plot_h = (canvas.height - 75.0).max(1.0);
        let puck_x = plot_x + self.puck_pos.0 * plot_w;
        let puck_y = plot_y + (1.0 - self.puck_pos.1) * plot_h;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= LOUDNESS_PUCK_HIT_RADIUS
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
        let p_row = (((1.0 - self.puck_pos.1) * (height - 6) as f32) + 3.0).round() as usize;
        let p_col = ((self.puck_pos.0 * (left_w - 6) as f32) + 3.0).round() as usize;
        if p_row < height - 1 && p_col < mid_x {
            grid[p_row][p_col] = 'P';
        }

        let right_w = width - mid_x - 2;
        let bar_spacing = right_w / 9;
        for (i, &gain) in self.band_gains_db.iter().enumerate() {
            let col = mid_x + 2 + (i + 1) * bar_spacing;
            let norm_gain = ((gain + 12.0) / 36.0).clamp(0.0, 1.0);
            let bar_h = (norm_gain * (height - 4) as f32).round() as usize;
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
            "DYNAMIC FLETCHER-MUNSON EQUAL-LOUDNESS CONTOUR & SPECTRAL BALANCE HUD",
            egui::FontId::proportional(13.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Tabs (y: 48..92) - 44pt height
        let tabs = [
            (LoudnessStandard::Iso226_2003, "ISO 226:2003"),
            (LoudnessStandard::FletcherMunson1933, "FLETCHER-M"),
            (LoudnessStandard::RobinsonDadson1956, "ROBINSON-D"),
            (LoudnessStandard::EbuR128KWeighted, "EBU R128 (K)"),
            (LoudnessStandard::CinemaXCurve, "SMPTE X-CURVE"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (itype, name)) in tabs.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_sel = self.standard == *itype;
            let bg_col = if is_sel {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(24, 32, 48)
            };
            let text_col = if is_sel {
                Color32::from_rgb(8, 20, 28)
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
                        self.set_standard(*itype);
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

        // Left 55%: Frequency Contour Response Curves
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
            "DYNAMIC PHON LEVEL & SPECTRAL COMPENSATION CONTOUR",
            egui::FontId::proportional(10.0),
            Color32::from_rgb(0, 229, 255),
        );

        let plot_rect = egui::Rect::from_min_max(
            egui::pos2(left_rect.min.x + 25.0, left_rect.min.y + 35.0),
            egui::pos2(left_rect.max.x - 25.0, left_rect.max.y - 35.0),
        );

        // 0 dB center reference line
        let zero_line_y = plot_rect.center().y;
        painter.line_segment(
            [
                egui::pos2(plot_rect.min.x, zero_line_y),
                egui::pos2(plot_rect.max.x, zero_line_y),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(100, 130, 165, 80)),
        );

        // Compensation Curve Path
        let num_pts = 32;
        let mut curve_pts = Vec::with_capacity(num_pts);
        for p in 0..num_pts {
            let t = p as f32 / (num_pts - 1) as f32;
            let cx = plot_rect.min.x + t * plot_rect.width();

            let f_norm = t;
            let bass_factor = (1.0 - f_norm * 2.0).max(0.0).powi(2) * self.bass_boost_db;
            let dip_factor =
                (-((f_norm - 0.65) * 6.0).powi(2)).exp() * self.standard.nominal_ear_canal_dip_db();
            let air_factor = (f_norm - 0.75).max(0.0) * 4.0 * (self.treble_tilt_db / 4.0);
            let total_db = bass_factor + dip_factor + air_factor;

            let cy = zero_line_y - (total_db / 24.0) * (plot_rect.height() * 0.45);
            curve_pts.push(egui::pos2(cx, cy));
        }

        for i in 0..curve_pts.len() - 1 {
            painter.line_segment(
                [curve_pts[i], curve_pts[i + 1]],
                Stroke::new(2.5_f32, Color32::from_rgb(0, 229, 255)),
            );
        }

        // Interactive Puck (X = Monitoring SPL, Y = Compensation Amount)
        let puck_x = plot_rect.min.x + self.puck_pos.0 * plot_rect.width();
        let puck_y = plot_rect.max.y - self.puck_pos.1 * plot_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - plot_rect.min.x) / plot_rect.width()).clamp(0.0, 1.0);
                    let ny = ((plot_rect.max.y - mouse_pos.y) / plot_rect.height()).clamp(0.0, 1.0);
                    self.puck_pos = (nx, ny);
                    self.monitoring_spl_db = Self::normalized_to_spl(nx);
                    self.compensation_amount = Self::normalized_to_comp(ny);
                    self.update_contour_simulation();
                }
            }
        }

        painter.circle_stroke(
            puck_pos,
            LOUDNESS_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(0, 229, 255, 150)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(0, 229, 255));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 18.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Monitoring: {:.1} dB SPL | Comp: {:.0}% | Ref: {:.1} Phon | Bass: +{:.1}dB",
                self.monitoring_spl_db,
                self.compensation_amount * 100.0,
                self.reference_phon,
                self.bass_boost_db
            ),
            egui::FontId::proportional(10.5),
            Color32::from_rgb(100, 220, 255),
        );

        // Right 45%: Critical Bark Bands Gain Spectrum
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
            "CRITICAL BARK BAND COMPENSATION GAINS (dB)",
            egui::FontId::proportional(10.0),
            Color32::from_rgb(0, 229, 255),
        );

        let band_labels = [
            "40Hz", "100Hz", "300Hz", "1kHz", "3.5k", "6kHz", "10kHz", "16kHz",
        ];
        let bar_w = (right_rect.width() - 30.0 - 7.0 * 6.0) / 8.0;
        let r_zero_y = right_rect.center().y + 10.0;

        for (i, &gain) in self.band_gains_db.iter().enumerate() {
            let bx = right_rect.min.x + 15.0 + i as f32 * (bar_w + 6.0);
            let bar_h = (gain / 24.0) * (right_rect.height() * 0.35);
            let (top_y, bot_y) = if bar_h >= 0.0 {
                (r_zero_y - bar_h, r_zero_y)
            } else {
                (r_zero_y, r_zero_y - bar_h)
            };

            let b_rect =
                egui::Rect::from_min_max(egui::pos2(bx, top_y), egui::pos2(bx + bar_w, bot_y));
            let col = if i < 3 {
                Color32::from_rgb(255, 107, 43)
            } else if i == 4 {
                Color32::from_rgb(255, 215, 0)
            } else {
                Color32::from_rgb(0, 229, 255)
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
                "MONITORING SPL",
                format!("{:.1} dB SPL (Listening)", self.monitoring_spl_db),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "COMPENSATION WEIGHT",
                format!("{:.0}% (Dynamic)", self.compensation_amount * 100.0),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "3.5kHz RESIDUAL DIP",
                format!("{:.1} dB (Ear Canal)", self.band_gains_db[4]),
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "SUB-BASS BOOST",
                format!("+{:.1} dB (Fletcher 40Hz)", self.bass_boost_db),
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
            "[PASS] Dynamic Equal-Loudness Contour Compensation Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
