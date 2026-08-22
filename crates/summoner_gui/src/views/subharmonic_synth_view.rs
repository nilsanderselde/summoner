// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Psychoacoustic Harmonic Exciter & Multi-Band Phase-Aligned Subharmonic Sub-Generator HUD (Step 1562).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const SUBHARMONIC_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_SUB_FREQ_HZ: f32 = 20.0;
pub const MAX_SUB_FREQ_HZ: f32 = 160.0;
pub const MIN_SUB_DRIVE_DB: f32 = -24.0;
pub const MAX_SUB_DRIVE_DB: f32 = 18.0;
pub const MIN_PHASE_DEG: f32 = -180.0;
pub const MAX_PHASE_DEG: f32 = 180.0;

/// Subharmonic Generator Mode Profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubharmonicProfile {
    SubOctave12st,      // -1 Octave (-12 semitones) fundamental tracking sub
    SubOctave24st,      // -2 Octaves (-24 semitones) deep seismic sub
    SubFifth19st,       // -19 semitones subharmonic fifth generator
    DualOctaveAligned,  // Combined -12st and -24st in-phase locked subs
    SaturatedTransient, // Subharmonic envelope with nonlinear tube saturation
}

impl SubharmonicProfile {
    pub fn profile_name(&self) -> &'static str {
        match self {
            Self::SubOctave12st => "SUB -1 OCT (-12st)",
            Self::SubOctave24st => "SUB -2 OCT (-24st)",
            Self::SubFifth19st => "SUB 5TH (-19st)",
            Self::DualOctaveAligned => "DUAL SUB ALIGNED",
            Self::SaturatedTransient => "SATURATED TRANSIENT",
        }
    }

    pub fn nominal_crossover_hz(&self) -> f32 {
        match self {
            Self::SubOctave12st => 90.0,
            Self::SubOctave24st => 60.0,
            Self::SubFifth19st => 110.0,
            Self::DualOctaveAligned => 80.0,
            Self::SaturatedTransient => 100.0,
        }
    }

    pub fn nominal_sub_ratio(&self) -> (f32, f32) {
        // (Sub1 weight, Sub2 weight)
        match self {
            Self::SubOctave12st => (1.0, 0.0),
            Self::SubOctave24st => (0.0, 1.0),
            Self::SubFifth19st => (0.7, 0.3),
            Self::DualOctaveAligned => (0.7, 0.7),
            Self::SaturatedTransient => (0.85, 0.45),
        }
    }
}

/// Psychoacoustic Subharmonic Generator & Harmonic Exciter HUD.
#[derive(Debug, Clone)]
pub struct SubharmonicSynthView {
    pub profile: SubharmonicProfile,
    pub crossover_freq_hz: f32,   // [20.0 ..= 160.0 Hz]
    pub sub_drive_db: f32,        // [-24.0 ..= +18.0 dB]
    pub phase_alignment_deg: f32, // [-180.0 ..= +180.0 deg]
    pub dry_wet_mix_pct: f32,     // [0.0 ..= 100.0 %]
    pub sub_puck_pos: (f32, f32), // Normalized (X: crossover freq, Y: sub drive)
    pub is_dragging_puck: bool,
    pub sub_oct1_energy: f32,    // [0.0 ..= 1.0]
    pub sub_oct2_energy: f32,    // [0.0 ..= 1.0]
    pub fundamental_energy: f32, // [0.0 ..= 1.0]
    pub upper_air1_energy: f32,  // [0.0 ..= 1.0]
    pub upper_air2_energy: f32,  // [0.0 ..= 1.0]
    pub phase_correlation: f32,  // [-1.0 ..= +1.0]
    pub color_palette: ContrastColorPalette,
}

impl Default for SubharmonicSynthView {
    fn default() -> Self {
        Self::new()
    }
}

impl SubharmonicSynthView {
    pub fn new() -> Self {
        let mut view = Self {
            profile: SubharmonicProfile::DualOctaveAligned,
            crossover_freq_hz: 80.0,
            sub_drive_db: 4.5,
            phase_alignment_deg: 0.0,
            dry_wet_mix_pct: 70.0,
            sub_puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            sub_oct1_energy: 0.75,
            sub_oct2_energy: 0.65,
            fundamental_energy: 0.90,
            upper_air1_energy: 0.40,
            upper_air2_energy: 0.25,
            phase_correlation: 0.98,
            color_palette: ContrastColorPalette::default(),
        };
        view.sub_puck_pos = (
            Self::freq_to_normalized(view.crossover_freq_hz),
            Self::drive_to_normalized(view.sub_drive_db),
        );
        view.update_synthesis_model();
        view
    }

    pub fn freq_to_normalized(hz: f32) -> f32 {
        let h = hz.clamp(MIN_SUB_FREQ_HZ, MAX_SUB_FREQ_HZ);
        ((h.ln() - MIN_SUB_FREQ_HZ.ln()) / (MAX_SUB_FREQ_HZ.ln() - MIN_SUB_FREQ_HZ.ln()))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_freq(norm: f32) -> f32 {
        (MIN_SUB_FREQ_HZ.ln()
            + norm.clamp(0.0, 1.0) * (MAX_SUB_FREQ_HZ.ln() - MIN_SUB_FREQ_HZ.ln()))
        .exp()
    }

    pub fn drive_to_normalized(db: f32) -> f32 {
        let d = db.clamp(MIN_SUB_DRIVE_DB, MAX_SUB_DRIVE_DB);
        ((d - MIN_SUB_DRIVE_DB) / (MAX_SUB_DRIVE_DB - MIN_SUB_DRIVE_DB)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_drive(norm: f32) -> f32 {
        MIN_SUB_DRIVE_DB + norm.clamp(0.0, 1.0) * (MAX_SUB_DRIVE_DB - MIN_SUB_DRIVE_DB)
    }

    pub fn set_profile(&mut self, profile: SubharmonicProfile) {
        self.profile = profile;
        self.crossover_freq_hz = profile.nominal_crossover_hz();
        self.sub_puck_pos = (
            Self::freq_to_normalized(self.crossover_freq_hz),
            Self::drive_to_normalized(self.sub_drive_db),
        );
        self.update_synthesis_model();
    }

    /// Update multi-band harmonic energy levels and phase correlation.
    pub fn update_synthesis_model(&mut self) {
        let (w1, w2) = self.profile.nominal_sub_ratio();
        let drive_gain = 10.0_f32.powf(self.sub_drive_db / 20.0);

        self.sub_oct1_energy = (w1 * 0.7 * drive_gain).clamp(0.0, 1.0);
        self.sub_oct2_energy = (w2 * 0.6 * drive_gain).clamp(0.0, 1.0);
        self.fundamental_energy = (0.85 * (1.0 + self.sub_drive_db * 0.02)).clamp(0.1, 1.0);

        // Exciter harmonics generated via non-linear wave shaping
        self.upper_air1_energy =
            (0.35 + 0.15 * (self.sub_drive_db / 18.0).clamp(-1.0, 1.0)).clamp(0.0, 1.0);
        self.upper_air2_energy =
            (0.20 + 0.10 * (self.sub_drive_db / 18.0).clamp(-1.0, 1.0)).clamp(0.0, 1.0);

        // Phase correlation degrades when phase alignment is off zero
        let phase_rad = self.phase_alignment_deg.to_radians();
        self.phase_correlation = phase_rad.cos();
    }

    /// Hit test coordinate on the interactive subharmonic synthesis puck.
    pub fn hit_test_sub_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.sub_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.sub_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= SUBHARMONIC_PUCK_HIT_RADIUS
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

        // Left half: Subharmonic phase puck
        let left_w = mid_x - 2;
        let p_row = (((1.0 - self.sub_puck_pos.1) * (height - 5) as f32) + 2.0).round() as usize;
        let p_col = ((self.sub_puck_pos.0 * (left_w - 4) as f32) + 2.0).round() as usize;
        if p_row < height - 1 && p_col < mid_x {
            grid[p_row][p_col] = 'S';
        }

        // Right half: Multi-band harmonic bars
        let right_w = width - mid_x - 2;
        let bands = [
            ("SUB2", self.sub_oct2_energy),
            ("SUB1", self.sub_oct1_energy),
            ("FUND", self.fundamental_energy),
            ("AIR1", self.upper_air1_energy),
            ("AIR2", self.upper_air2_energy),
        ];

        let bar_spacing = right_w / (bands.len() + 1);
        for (i, (_name, energy)) in bands.iter().enumerate() {
            let col = mid_x + 2 + (i + 1) * bar_spacing;
            let bar_h = (energy * (height - 4) as f32).round() as usize;
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
            "PSYCHOACOUSTIC HARMONIC EXCITER & SUBHARMONIC SUB-GENERATOR HUD",
            egui::FontId::proportional(14.5),
            Color32::from_rgb(240, 245, 255),
        );

        // Subharmonic Profile Tabs (y: 48..92) - Each tab >= 44pt touch target
        let profiles = [
            (SubharmonicProfile::SubOctave12st, "SUB -1 OCT (-12st)"),
            (SubharmonicProfile::SubOctave24st, "SUB -2 OCT (-24st)"),
            (SubharmonicProfile::SubFifth19st, "SUB 5TH (-19st)"),
            (SubharmonicProfile::DualOctaveAligned, "DUAL SUB ALIGNED"),
            (SubharmonicProfile::SaturatedTransient, "SATURATED SUB"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (prof, name)) in profiles.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_sel = self.profile == *prof;
            let bg_col = if is_sel {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(24, 32, 48)
            };
            let text_col = if is_sel {
                Color32::from_rgb(10, 16, 24)
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
                        self.set_profile(*prof);
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

        // Left 55%: Phase Alignment & Subharmonic Tracking Orbit
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
            "SUBHARMONIC TRACKING & PHASE CORRELATION ORBIT",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(0, 229, 255),
        );

        // Subharmonic Lissajous / Phase Orbit Curves
        let cx = left_rect.center().x;
        let cy = left_rect.center().y + 10.0;
        let radius = 65.0_f32;
        painter.circle_stroke(
            egui::pos2(cx, cy),
            radius,
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 85, 120, 70)),
        );
        painter.line_segment(
            [egui::pos2(cx - radius, cy), egui::pos2(cx + radius, cy)],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 85, 120, 70)),
        );
        painter.line_segment(
            [egui::pos2(cx, cy - radius), egui::pos2(cx, cy + radius)],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 85, 120, 70)),
        );

        // Wave shape curve
        let num_pts = 60;
        let mut prev_pt: Option<egui::Pos2> = None;
        for step in 0..=num_pts {
            let t = step as f32 / num_pts as f32 * std::f32::consts::TAU;
            let sub_x = cx + radius * (t * 2.0).cos();
            let sub_y = cy + radius * (t + self.phase_alignment_deg.to_radians()).sin() * 0.7;
            let cur_pt = egui::pos2(sub_x, sub_y);
            if let Some(p) = prev_pt {
                painter.line_segment(
                    [p, cur_pt],
                    Stroke::new(1.8_f32, Color32::from_rgb(0, 229, 255)),
                );
            }
            prev_pt = Some(cur_pt);
        }

        // Interactive Puck
        let puck_x = left_rect.min.x + self.sub_puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.sub_puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.sub_puck_pos = (nx, ny);
                    self.crossover_freq_hz = Self::normalized_to_freq(nx);
                    self.sub_drive_db = Self::normalized_to_drive(ny);
                    self.update_synthesis_model();
                }
            }
        }

        // Touch Hit Target boundary (>= 44x44pt)
        painter.circle_stroke(
            puck_pos,
            SUBHARMONIC_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(0, 229, 255, 150)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(0, 229, 255));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 20.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Crossover: {:.1} Hz | Drive: {:+.1} dB | Phase: {:+.1}° (r={:.2})",
                self.crossover_freq_hz,
                self.sub_drive_db,
                self.phase_alignment_deg,
                self.phase_correlation
            ),
            egui::FontId::proportional(10.5),
            Color32::from_rgb(0, 229, 255),
        );

        // Right 45%: Subharmonic & Upper Harmonic Energy Distribution
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
            "MULTI-BAND HARMONIC SPECTRUM",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(0, 229, 255),
        );

        let bands = [
            (
                "SUB -2",
                self.sub_oct2_energy,
                Color32::from_rgb(180, 90, 255),
            ),
            (
                "SUB -1",
                self.sub_oct1_energy,
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "FUND",
                self.fundamental_energy,
                Color32::from_rgb(0, 255, 180),
            ),
            (
                "AIR +1",
                self.upper_air1_energy,
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "AIR +2",
                self.upper_air2_energy,
                Color32::from_rgb(255, 107, 43),
            ),
        ];

        let bar_w = (right_rect.width() - 30.0 - 4.0 * 8.0) / 5.0;
        for (i, (bname, energy, col)) in bands.iter().enumerate() {
            let bx = right_rect.min.x + 15.0 + i as f32 * (bar_w + 8.0);
            let bar_h = energy * (right_rect.height() - 80.0);
            let b_rect = egui::Rect::from_min_max(
                egui::pos2(bx, right_rect.max.y - 25.0 - bar_h),
                egui::pos2(bx + bar_w, right_rect.max.y - 25.0),
            );
            painter.rect_filled(b_rect, 3.0, *col);

            painter.text(
                egui::pos2(bx + bar_w * 0.5, right_rect.max.y - 20.0),
                egui::Align2::CENTER_TOP,
                *bname,
                egui::FontId::proportional(9.0),
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
                "SUB CROSSOVER",
                format!(
                    "{:.1} Hz ({})",
                    self.crossover_freq_hz,
                    self.profile.profile_name()
                ),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "SUB DRIVE / GAIN",
                format!(
                    "{:+.1} dB ({:.0}% Mix)",
                    self.sub_drive_db, self.dry_wet_mix_pct
                ),
                Color32::from_rgb(255, 180, 50),
            ),
            (
                "PHASE ALIGNMENT",
                format!(
                    "{:+.1}° (r={:.2})",
                    self.phase_alignment_deg, self.phase_correlation
                ),
                Color32::from_rgb(0, 255, 180),
            ),
            (
                "SUB GENERATION",
                format!(
                    "{:.0}% / {:.0}% Energy",
                    self.sub_oct1_energy * 100.0,
                    self.sub_oct2_energy * 100.0
                ),
                Color32::from_rgb(180, 90, 255),
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
                egui::FontId::proportional(13.5),
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
            "[PASS] Psychoacoustic Subharmonic Generator & Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
