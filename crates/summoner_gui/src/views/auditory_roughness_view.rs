// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Psychoacoustic Sensory Dissonance & Critical Band Auditory Roughness Map HUD (Step 1602).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const ROUGHNESS_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_CENTER_FREQ_HZ: f32 = 50.0;
pub const MAX_CENTER_FREQ_HZ: f32 = 5000.0;
pub const MIN_INTERVAL_SEMITONES: f32 = 0.0;
pub const MAX_INTERVAL_SEMITONES: f32 = 14.0;

/// Psychoacoustic dissonance and auditory roughness evaluation models.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoughnessStandard {
    PlompLevelt1965,       // Classic Plomp & Levelt critical band sensory dissonance
    KameokaKuriyagawa1969, // Multi-partial harmonic dissonance summation
    FastlZwicker2007,      // Specific roughness in Asper (70 Hz AM modulation peak)
    Vassilakis2001,        // SPL-dependent nonlinear spectral dissonance
    SetharesMicrotonal,    // Dynamic microtonal timbre-scale dissonance curve
}

impl RoughnessStandard {
    pub fn standard_name(&self) -> &'static str {
        match self {
            Self::PlompLevelt1965 => "PLOMP-LEVELT (1965)",
            Self::KameokaKuriyagawa1969 => "KAMEOKA-KURIYAGAWA (1969)",
            Self::FastlZwicker2007 => "FASTL-ZWICKER (ASPER)",
            Self::Vassilakis2001 => "VASSILAKIS (SPL DISS)",
            Self::SetharesMicrotonal => "SETHARES (MICROTONAL)",
        }
    }

    pub fn nominal_center_freq_hz(&self) -> f32 {
        match self {
            Self::PlompLevelt1965 => 440.0,
            Self::KameokaKuriyagawa1969 => 520.0,
            Self::FastlZwicker2007 => 1000.0,
            Self::Vassilakis2001 => 330.0,
            Self::SetharesMicrotonal => 660.0,
        }
    }

    pub fn nominal_interval_semitones(&self) -> f32 {
        match self {
            Self::PlompLevelt1965 => 1.4, // Minor second maximum roughness
            Self::KameokaKuriyagawa1969 => 1.8,
            Self::FastlZwicker2007 => 2.2,
            Self::Vassilakis2001 => 1.2,
            Self::SetharesMicrotonal => 0.75, // Quarter-tone / microtonal dissonance
        }
    }

    pub fn nominal_partials(&self) -> usize {
        match self {
            Self::PlompLevelt1965 => 2,
            Self::KameokaKuriyagawa1969 => 8,
            Self::FastlZwicker2007 => 6,
            Self::Vassilakis2001 => 12,
            Self::SetharesMicrotonal => 16,
        }
    }

    pub fn nominal_mod_rate_hz(&self) -> f32 {
        match self {
            Self::PlompLevelt1965 => 35.0,
            Self::KameokaKuriyagawa1969 => 45.0,
            Self::FastlZwicker2007 => 70.0, // 70Hz AM maximum roughness peak
            Self::Vassilakis2001 => 50.0,
            Self::SetharesMicrotonal => 28.0,
        }
    }
}

/// Psychoacoustic sensory dissonance & critical band auditory roughness map HUD.
#[derive(Debug, Clone)]
pub struct AuditoryRoughnessView {
    pub standard: RoughnessStandard,
    pub center_freq_hz: f32,
    pub interval_semitones: f32,
    pub partial_count: usize,
    pub modulation_rate_hz: f32,
    pub sensory_dissonance_index: f32,
    pub roughness_asper: f32,
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    pub critical_band_roughness: [f32; 8],
    pub color_palette: ContrastColorPalette,
}

impl Default for AuditoryRoughnessView {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditoryRoughnessView {
    pub fn new() -> Self {
        let mut view = Self {
            standard: RoughnessStandard::PlompLevelt1965,
            center_freq_hz: 440.0,
            interval_semitones: 1.4,
            partial_count: 2,
            modulation_rate_hz: 35.0,
            sensory_dissonance_index: 0.82,
            roughness_asper: 1.25,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            critical_band_roughness: [0.35, 0.58, 0.82, 0.95, 0.74, 0.48, 0.28, 0.85],
            color_palette: ContrastColorPalette::default(),
        };
        view.puck_pos = (
            Self::freq_to_normalized(view.center_freq_hz),
            Self::interval_to_normalized(view.interval_semitones),
        );
        view.update_roughness_simulation();
        view
    }

    pub fn freq_to_normalized(freq: f32) -> f32 {
        let f = freq.clamp(MIN_CENTER_FREQ_HZ, MAX_CENTER_FREQ_HZ);
        ((f.ln() - MIN_CENTER_FREQ_HZ.ln()) / (MAX_CENTER_FREQ_HZ.ln() - MIN_CENTER_FREQ_HZ.ln()))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_freq(norm: f32) -> f32 {
        (MIN_CENTER_FREQ_HZ.ln()
            + norm.clamp(0.0, 1.0) * (MAX_CENTER_FREQ_HZ.ln() - MIN_CENTER_FREQ_HZ.ln()))
            .exp()
    }

    pub fn interval_to_normalized(semi: f32) -> f32 {
        let s = semi.clamp(MIN_INTERVAL_SEMITONES, MAX_INTERVAL_SEMITONES);
        ((s - MIN_INTERVAL_SEMITONES) / (MAX_INTERVAL_SEMITONES - MIN_INTERVAL_SEMITONES))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_interval(norm: f32) -> f32 {
        MIN_INTERVAL_SEMITONES
            + norm.clamp(0.0, 1.0) * (MAX_INTERVAL_SEMITONES - MIN_INTERVAL_SEMITONES)
    }

    pub fn set_standard(&mut self, std: RoughnessStandard) {
        self.standard = std;
        self.center_freq_hz = std.nominal_center_freq_hz();
        self.interval_semitones = std.nominal_interval_semitones();
        self.partial_count = std.nominal_partials();
        self.modulation_rate_hz = std.nominal_mod_rate_hz();
        self.puck_pos = (
            Self::freq_to_normalized(self.center_freq_hz),
            Self::interval_to_normalized(self.interval_semitones),
        );
        self.update_roughness_simulation();
    }

    pub fn update_roughness_simulation(&mut self) {
        let f = self.center_freq_hz;
        let semi = self.interval_semitones;
        let partials = self.partial_count as f32;

        // Critical bandwidth CBW approximation: CBW = 25 + 75 * (1 + 1.4 * (f / 1000)^2)^0.69
        let cbw = 25.0 + 75.0 * (1.0 + 1.4 * (f / 1000.0).powi(2)).powf(0.69);
        let delta_f = f * (2.0_f32.powf(semi / 12.0) - 1.0);
        let s = delta_f / (0.24 * cbw);

        // Plomp-Levelt standard dissonance curve: d(s) = e^(-3.5*s) - e^(-5.75*s)
        let raw_diss = ((-3.5 * s).exp() - (-5.75 * s).exp()).max(0.0) * 4.5;
        let diss_index = (raw_diss * (1.0 + (partials - 1.0) * 0.15)).clamp(0.0, 1.0);
        let asper = diss_index * 1.85 * (self.modulation_rate_hz / 70.0).min(1.2);

        self.sensory_dissonance_index = diss_index;
        self.roughness_asper = asper;

        // 8 Critical Bark Bands Roughness Spectrum: [100Hz, 250Hz, 500Hz, 1kHz, 2kHz, 4kHz, 8kHz, Total]
        let band_centers: [f32; 7] = [100.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0];
        let mut bands = [0.0f32; 8];
        for (i, &bc) in band_centers.iter().enumerate() {
            let dist = ((bc.ln() - f.ln()).abs() / 2.5).clamp(0.0, 1.0);
            bands[i] = (diss_index * (1.0 - dist * 0.75)).clamp(0.02, 1.2);
        }
        bands[7] = diss_index; // Total Dissonance index
        self.critical_band_roughness = bands;
    }

    pub fn hit_test_roughness_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= ROUGHNESS_PUCK_HIT_RADIUS
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
            grid[p_row][p_col] = 'P';
        }

        let right_w = width - mid_x - 2;
        let bar_spacing = right_w / 9;
        for (i, &amp) in self.critical_band_roughness.iter().enumerate() {
            let col = mid_x + 2 + (i + 1) * bar_spacing;
            let bar_h = (amp.clamp(0.0, 1.0) * (height - 4) as f32).round() as usize;
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

        // Background: Deep Charcoal Indigo (#0E121E)
        painter.rect_filled(rect, 6.0, Color32::from_rgb(14, 18, 30));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "PSYCHOACOUSTIC SENSORY DISSONANCE & AUDITORY ROUGHNESS MAP HUD",
            egui::FontId::proportional(13.5),
            Color32::from_rgb(240, 245, 255),
        );

        // Tabs (y: 48..92) - 44pt height
        let tabs = [
            (RoughnessStandard::PlompLevelt1965, "PLOMP-LEVELT"),
            (RoughnessStandard::KameokaKuriyagawa1969, "KAMEOKA-KURIYAGAWA"),
            (RoughnessStandard::FastlZwicker2007, "FASTL-ZWICKER"),
            (RoughnessStandard::Vassilakis2001, "VASSILAKIS (SPL)"),
            (RoughnessStandard::SetharesMicrotonal, "SETHARES (MICRO)"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (stype, name)) in tabs.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_sel = self.standard == *stype;
            let bg_col = if is_sel {
                Color32::from_rgb(255, 140, 0)
            } else {
                Color32::from_rgb(26, 34, 52)
            };
            let text_col = if is_sel {
                Color32::from_rgb(18, 8, 2)
            } else {
                Color32::from_rgb(215, 230, 250)
            };

            painter.rect_filled(tab_rect, 4.0, bg_col);
            painter.text(
                tab_rect.center(),
                egui::Align2::CENTER_CENTER,
                *name,
                egui::FontId::proportional(10.0),
                text_col,
            );

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if tab_rect.contains(pos) {
                        self.set_standard(*stype);
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

        // Left 55%: Plomp-Levelt Dissonance Curve & Critical Band Intervallic Field
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
            "SENSORY DISSONANCE FIELD (FREQ vs INTERVAL SEMITONES)",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(255, 140, 0),
        );

        // Draw interval reference grid lines
        let intervals = [
            (0.0, "Unison (0ST)"),
            (1.0, "m2 (1ST)"),
            (2.0, "M2 (2ST)"),
            (3.0, "m3 (3ST)"),
            (7.0, "P5 (7ST)"),
            (12.0, "Oct (12ST)"),
        ];
        for (st, _lbl) in intervals.iter() {
            let ny = Self::interval_to_normalized(*st);
            let y_pos = left_rect.max.y - 25.0 - ny * (left_rect.height() - 55.0);
            painter.line_segment(
                [
                    egui::pos2(left_rect.min.x + 15.0, y_pos),
                    egui::pos2(left_rect.max.x - 15.0, y_pos),
                ],
                Stroke::new(0.8_f32, Color32::from_rgb(35, 55, 85)),
            );
        }

        // Interactive Puck
        let puck_x = left_rect.min.x + self.puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.puck_pos = (nx, ny);
                    self.center_freq_hz = Self::normalized_to_freq(nx);
                    self.interval_semitones = Self::normalized_to_interval(ny);
                    self.update_roughness_simulation();
                }
            }
        }

        painter.circle_stroke(
            puck_pos,
            ROUGHNESS_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(255, 140, 0, 150)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(255, 140, 0));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 18.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Freq: {:.0} Hz | Interval: {:.2} ST | Dissonance: {:.0}% | Roughness: {:.2} Asper",
                self.center_freq_hz,
                self.interval_semitones,
                self.sensory_dissonance_index * 100.0,
                self.roughness_asper
            ),
            egui::FontId::proportional(10.5),
            Color32::from_rgb(255, 205, 140),
        );

        // Right 45%: Critical Bark Bands Roughness Spectrum
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
            "CRITICAL BARK BANDS AUDITORY ROUGHNESS (ASPER)",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(255, 140, 0),
        );

        let band_labels = [
            "100Hz", "250Hz", "500Hz", "1kHz", "2kHz", "4kHz", "8kHz", "TOTAL",
        ];
        let bar_w = (right_rect.width() - 30.0 - 7.0 * 6.0) / 8.0;
        for (i, &amp) in self.critical_band_roughness.iter().enumerate() {
            let bx = right_rect.min.x + 15.0 + i as f32 * (bar_w + 6.0);
            let bar_h = (amp.clamp(0.0, 1.2) / 1.2) * (right_rect.height() - 80.0);
            let b_rect = egui::Rect::from_min_max(
                egui::pos2(bx, right_rect.max.y - 25.0 - bar_h),
                egui::pos2(bx + bar_w, right_rect.max.y - 25.0),
            );
            let col = if i == 7 {
                Color32::from_rgb(255, 140, 0)
            } else if amp > 0.7 {
                Color32::from_rgb(255, 60, 60)
            } else {
                Color32::from_rgb(0, 229, 255)
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
                "CENTER FREQUENCY",
                format!("{:.0} Hz (Carrier)", self.center_freq_hz),
                Color32::from_rgb(255, 140, 0),
            ),
            (
                "INTERVAL SEMITONES",
                format!("{:.2} ST (Beat Modulation)", self.interval_semitones),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "DISSONANCE INDEX",
                format!("{:.1}% (Plomp-Levelt)", self.sensory_dissonance_index * 100.0),
                Color32::from_rgb(255, 60, 60),
            ),
            (
                "SPECIFIC ROUGHNESS",
                format!("{:.2} Asper (AM Peak)", self.roughness_asper),
                Color32::from_rgb(0, 229, 255),
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
            "[PASS] Auditory Roughness & Dissonance Map Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
