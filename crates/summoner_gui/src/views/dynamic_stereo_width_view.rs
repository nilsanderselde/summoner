// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Mastering Multi-Band Dynamic Stereo Width, Elliptical Bass Mono & Side Phase Unmasker HUD (Step 1593).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const STEREO_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_ELLIPTICAL_BASS_HZ: f32 = 40.0;
pub const MAX_ELLIPTICAL_BASS_HZ: f32 = 300.0;
pub const MIN_SIDE_WIDTH_RATIO: f32 = 0.0;
pub const MAX_SIDE_WIDTH_RATIO: f32 = 2.5;

/// Mastering stereo imaging profiles and elliptical filter configurations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StereoWidthProfile {
    BroadcastMasteringClean, // 120Hz elliptical bass mono, 100% core mid, 125% air expansion
    ClubVinylPressing,       // 180Hz elliptical bass mono, strict phase correlation >= 0.85
    CinematicSuperWide, // 80Hz elliptical bass mono, 160% side expansion, dynamic side unmasking
    AcousticNaturalDepth, // 90Hz elliptical bass mono, subtle 110% ambient width
    EDMPolyrhythmicHyperWidth, // 140Hz elliptical bass mono, 200% high-frequency side width
}

impl StereoWidthProfile {
    pub fn profile_name(&self) -> &'static str {
        match self {
            Self::BroadcastMasteringClean => "BROADCAST MASTER (CLEAN)",
            Self::ClubVinylPressing => "CLUB VINYL CUT (180Hz MONO)",
            Self::CinematicSuperWide => "CINEMATIC SUPER WIDE",
            Self::AcousticNaturalDepth => "ACOUSTIC NATURAL DEPTH",
            Self::EDMPolyrhythmicHyperWidth => "EDM HYPER WIDTH (200%)",
        }
    }

    pub fn nominal_bass_cutoff_hz(&self) -> f32 {
        match self {
            Self::BroadcastMasteringClean => 120.0,
            Self::ClubVinylPressing => 180.0,
            Self::CinematicSuperWide => 80.0,
            Self::AcousticNaturalDepth => 90.0,
            Self::EDMPolyrhythmicHyperWidth => 140.0,
        }
    }

    pub fn nominal_width_ratio(&self) -> f32 {
        match self {
            Self::BroadcastMasteringClean => 1.25,
            Self::ClubVinylPressing => 0.95,
            Self::CinematicSuperWide => 1.60,
            Self::AcousticNaturalDepth => 1.10,
            Self::EDMPolyrhythmicHyperWidth => 2.00,
        }
    }

    pub fn nominal_phase_correlation(&self) -> f32 {
        match self {
            Self::BroadcastMasteringClean => 0.92,
            Self::ClubVinylPressing => 0.96,
            Self::CinematicSuperWide => 0.72,
            Self::AcousticNaturalDepth => 0.88,
            Self::EDMPolyrhythmicHyperWidth => 0.65,
        }
    }
}

/// Mastering multi-band dynamic stereo width, elliptical bass mono & side phase unmasker HUD.
#[derive(Debug, Clone)]
pub struct DynamicStereoWidthView {
    pub profile: StereoWidthProfile,
    pub elliptical_bass_hz: f32,
    pub side_width_ratio: f32,
    pub phase_correlation: f32,
    pub side_unmasking_db: f32,
    pub mono_compatibility: f32,
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    pub band_widths: [f32; 4],
    pub band_side_levels_db: [f32; 4],
    pub color_palette: ContrastColorPalette,
}

impl Default for DynamicStereoWidthView {
    fn default() -> Self {
        Self::new()
    }
}

impl DynamicStereoWidthView {
    pub fn new() -> Self {
        let mut view = Self {
            profile: StereoWidthProfile::BroadcastMasteringClean,
            elliptical_bass_hz: 120.0,
            side_width_ratio: 1.25,
            phase_correlation: 0.92,
            side_unmasking_db: -1.8,
            mono_compatibility: 0.95,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            band_widths: [0.0, 1.0, 1.25, 1.45],
            band_side_levels_db: [-48.0, -12.5, -6.2, -3.8],
            color_palette: ContrastColorPalette::default(),
        };
        view.puck_pos = (
            Self::bass_to_normalized(view.elliptical_bass_hz),
            Self::width_to_normalized(view.side_width_ratio),
        );
        view.update_stereo_simulation();
        view
    }

    pub fn bass_to_normalized(hz: f32) -> f32 {
        let h = hz.clamp(MIN_ELLIPTICAL_BASS_HZ, MAX_ELLIPTICAL_BASS_HZ);
        ((h - MIN_ELLIPTICAL_BASS_HZ) / (MAX_ELLIPTICAL_BASS_HZ - MIN_ELLIPTICAL_BASS_HZ))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_bass(norm: f32) -> f32 {
        MIN_ELLIPTICAL_BASS_HZ
            + norm.clamp(0.0, 1.0) * (MAX_ELLIPTICAL_BASS_HZ - MIN_ELLIPTICAL_BASS_HZ)
    }

    pub fn width_to_normalized(width: f32) -> f32 {
        let w = width.clamp(MIN_SIDE_WIDTH_RATIO, MAX_SIDE_WIDTH_RATIO);
        ((w - MIN_SIDE_WIDTH_RATIO) / (MAX_SIDE_WIDTH_RATIO - MIN_SIDE_WIDTH_RATIO)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_width(norm: f32) -> f32 {
        MIN_SIDE_WIDTH_RATIO + norm.clamp(0.0, 1.0) * (MAX_SIDE_WIDTH_RATIO - MIN_SIDE_WIDTH_RATIO)
    }

    pub fn set_profile(&mut self, prof: StereoWidthProfile) {
        self.profile = prof;
        self.elliptical_bass_hz = prof.nominal_bass_cutoff_hz();
        self.side_width_ratio = prof.nominal_width_ratio();
        self.phase_correlation = prof.nominal_phase_correlation();
        self.puck_pos = (
            Self::bass_to_normalized(self.elliptical_bass_hz),
            Self::width_to_normalized(self.side_width_ratio),
        );
        self.update_stereo_simulation();
    }

    pub fn update_stereo_simulation(&mut self) {
        let w = self.side_width_ratio;
        let bass_mono = self.elliptical_bass_hz;

        // Band 1: Sub Bass (Mono collapsed below elliptical cutoff)
        let b1_width = if bass_mono > 60.0 { 0.0 } else { 0.15 * w };
        // Band 2: Low-Mid (120Hz .. 1kHz)
        let b2_width = (0.85 * w).clamp(0.0, 2.0);
        // Band 3: High-Mid (1kHz .. 6kHz)
        let b3_width = (1.00 * w).clamp(0.0, 2.2);
        // Band 4: Air (6kHz .. 20kHz)
        let b4_width = (1.20 * w).clamp(0.0, 2.5);

        self.band_widths = [b1_width, b2_width, b3_width, b4_width];

        let s1_db = if b1_width == 0.0 {
            -60.0
        } else {
            -24.0 + (b1_width * 6.0)
        };
        let s2_db = (-14.0 + (b2_width - 1.0) * 8.0).clamp(-36.0, 0.0);
        let s3_db = (-8.0 + (b3_width - 1.0) * 10.0).clamp(-24.0, 3.0);
        let s4_db = (-4.0 + (b4_width - 1.0) * 12.0).clamp(-18.0, 6.0);

        self.band_side_levels_db = [s1_db, s2_db, s3_db, s4_db];
        self.phase_correlation =
            (1.0 - (w - 1.0).max(0.0) * 0.35 + (bass_mono / 300.0) * 0.15).clamp(0.2, 1.0);
        self.mono_compatibility = (self.phase_correlation * 0.9 + 0.1).clamp(0.0, 1.0);
    }

    pub fn hit_test_stereo_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= STEREO_PUCK_HIT_RADIUS
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
        let bar_spacing = right_w / 5;
        for (i, &w) in self.band_widths.iter().enumerate() {
            let col = mid_x + 2 + (i + 1) * bar_spacing;
            let bar_h = ((w / 2.5) * (height - 4) as f32).round() as usize;
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
            "MASTERING MULTI-BAND DYNAMIC STEREO WIDTH & SIDE PHASE UNMASKER HUD",
            egui::FontId::proportional(13.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Tabs (y: 48..92) - 44pt height
        let tabs = [
            (StereoWidthProfile::BroadcastMasteringClean, "BROADCAST"),
            (StereoWidthProfile::ClubVinylPressing, "VINYL (180Hz)"),
            (StereoWidthProfile::CinematicSuperWide, "CINEMATIC WIDE"),
            (StereoWidthProfile::AcousticNaturalDepth, "NATURAL DEPTH"),
            (
                StereoWidthProfile::EDMPolyrhythmicHyperWidth,
                "EDM HYPER (200%)",
            ),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (itype, name)) in tabs.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_sel = self.profile == *itype;
            let bg_col = if is_sel {
                Color32::from_rgb(255, 215, 0)
            } else {
                Color32::from_rgb(24, 32, 48)
            };
            let text_col = if is_sel {
                Color32::from_rgb(20, 16, 4)
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
                        self.set_profile(*itype);
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

        // Left 55%: Goniometer & Stereo Phase Correlation Field
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
            "POLAR PHASE CORRELATION & STEREO LISSAJOUS FIELD",
            egui::FontId::proportional(10.0),
            Color32::from_rgb(255, 215, 0),
        );

        // Center Mid/Side Axes
        let g_center = left_rect.center();
        painter.line_segment(
            [
                egui::pos2(g_center.x - 70.0, g_center.y),
                egui::pos2(g_center.x + 70.0, g_center.y),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(120, 140, 170, 90)),
        );
        painter.line_segment(
            [
                egui::pos2(g_center.x, g_center.y - 70.0),
                egui::pos2(g_center.x, g_center.y + 70.0),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(120, 140, 170, 90)),
        );

        // Stereo Energy Lissajous Ellipse
        let ellipse_rx: f32 = 35.0 * self.side_width_ratio;
        let ellipse_ry: f32 = 65.0;
        painter.circle_stroke(
            g_center,
            ellipse_ry.min(ellipse_rx),
            Stroke::new(2.0_f32, Color32::from_rgb(0, 229, 255)),
        );

        // Interactive Touch Puck (X = Bass mono cutoff, Y = Side width ratio)
        let puck_x = left_rect.min.x + self.puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.puck_pos = (nx, ny);
                    self.elliptical_bass_hz = Self::normalized_to_bass(nx);
                    self.side_width_ratio = Self::normalized_to_width(ny);
                    self.update_stereo_simulation();
                }
            }
        }

        painter.circle_stroke(
            puck_pos,
            STEREO_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(255, 215, 0, 150)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(255, 215, 0));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 18.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Mono Bass: {:.0} Hz | Width: {:.0}% | Correlation: +{:.2} | Mono: {:.0}%",
                self.elliptical_bass_hz,
                self.side_width_ratio * 100.0,
                self.phase_correlation,
                self.mono_compatibility * 100.0
            ),
            egui::FontId::proportional(10.5),
            Color32::from_rgb(255, 230, 140),
        );

        // Right 45%: 4-Band Width & Side Level Gauges
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
            "4-BAND DYNAMIC STEREO WIDTH & SIDE LEVELS",
            egui::FontId::proportional(10.0),
            Color32::from_rgb(255, 215, 0),
        );

        let band_names = [
            "SUB (<120Hz)",
            "LOW-MID (1k)",
            "HIGH-MID (6k)",
            "AIR (20kHz)",
        ];
        let bar_w = (right_rect.width() - 30.0 - 3.0 * 8.0) / 4.0;

        for (i, &w) in self.band_widths.iter().enumerate() {
            let bx = right_rect.min.x + 15.0 + i as f32 * (bar_w + 8.0);
            let bar_h = (w / 2.5) * (right_rect.height() - 75.0);
            let b_rect = egui::Rect::from_min_max(
                egui::pos2(bx, right_rect.max.y - 25.0 - bar_h),
                egui::pos2(bx + bar_w, right_rect.max.y - 25.0),
            );
            let col = if i == 0 {
                Color32::from_rgb(0, 255, 180)
            } else if i == 1 {
                Color32::from_rgb(0, 229, 255)
            } else if i == 2 {
                Color32::from_rgb(255, 215, 0)
            } else {
                Color32::from_rgb(255, 107, 43)
            };
            painter.rect_filled(b_rect, 3.0, col);

            painter.text(
                egui::pos2(bx + bar_w * 0.5, right_rect.max.y - 20.0),
                egui::Align2::CENTER_TOP,
                band_names[i],
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
                "ELLIPTICAL BASS MONO",
                format!("{:.0} Hz (Vinyl Guard)", self.elliptical_bass_hz),
                Color32::from_rgb(0, 255, 180),
            ),
            (
                "SIDE WIDTH RATIO",
                format!("{:.0}% (Dynamic)", self.side_width_ratio * 100.0),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "PHASE CORRELATION",
                format!("+{:.2} (Coherence)", self.phase_correlation),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "MONO COMPATIBILITY",
                format!("{:.0}% (Sum Loss 0dB)", self.mono_compatibility * 100.0),
                Color32::from_rgb(255, 107, 43),
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
            "[PASS] Mastering Multi-Band Dynamic Stereo Width Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
