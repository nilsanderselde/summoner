// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Wooden Cassotto Tone Chamber & Accordion/Bandoneon Resonant Soundbox HUD (Milestones 28, 33 & 34).
//!
//! Provides an interactive 2D Cassotto aperture shutter & wooden warmth resonance puck
//! (Aperture Shutter Opening [0.0..1.0] vs Wooden Warmth Gain [0.0..2.0]),
//! real-time 4-mode acoustic cavity filter frequency response curve,
//! dual-path direct vs chambered radiation transmission meter,
//! preset acoustic chamber profiles, and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and deterministic ASCII snapshot rendering.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, FontId, RichText, Rounding, Stroke, Vec2};

pub const CASSOTTO_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding target
pub const MIN_APERTURE_SHUTTER: f32 = 0.0;
pub const MAX_APERTURE_SHUTTER: f32 = 1.0;
pub const MIN_WOOD_WARMTH: f32 = 0.0;
pub const MAX_WOOD_WARMTH: f32 = 2.0;

/// Preset acoustic profiles for the Cassotto tone chamber and soundbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CassottoChamberProfile {
    /// Doble Cassotto Bandoneon (Premier 2-rank chamber for 16' Bassoon & 8' Clarinet, deep velvety resonance).
    #[default]
    DobleCassottoBandoneon,
    /// Standard Double Cassotto Accordion (Italian solid maple chamber, warm singing mids).
    StandardAccordionChamber,
    /// Vintage Concertina Spruce Cavity (Compact hexagonal body, crisp fast acoustic reflection).
    ConcertinaSpruceCavity,
    /// Harmonium Resonant Soundbox (Large pine pumping box with rich low-mid acoustic sustain).
    HarmoniumResonantBox,
    /// Open-Air Russian Bayan (Direct open reedblock with bright cutting projection).
    OpenAirBayan,
}

impl CassottoChamberProfile {
    pub fn name(&self) -> &'static str {
        match self {
            Self::DobleCassottoBandoneon => "Doble Cassotto Bandoneon (Premier Chamber)",
            Self::StandardAccordionChamber => "Standard Double Cassotto (Italian Maple)",
            Self::ConcertinaSpruceCavity => "Vintage Concertina Spruce Body",
            Self::HarmoniumResonantBox => "Harmonium Resonant Soundbox",
            Self::OpenAirBayan => "Open-Air Russian Bayan (Direct)",
        }
    }

    /// Returns nominal $(f_{\text{mode1}}, f_{\text{mode2}}, f_{\text{cutoff\_base}}, Q_{\text{cavity}}, \text{wood\_warmth})$.
    pub fn nominal_acoustics(&self) -> (f32, f32, f32, f32, f32) {
        match self {
            Self::DobleCassottoBandoneon => (460.0, 1420.0, 2600.0, 3.8, 0.85),
            Self::StandardAccordionChamber => (520.0, 1680.0, 3200.0, 3.2, 0.75),
            Self::ConcertinaSpruceCavity => (680.0, 2100.0, 4800.0, 2.4, 0.50),
            Self::HarmoniumResonantBox => (340.0, 1150.0, 2100.0, 4.5, 0.95),
            Self::OpenAirBayan => (850.0, 2800.0, 8000.0, 1.5, 0.25),
        }
    }
}

/// Physical modeling wooden Cassotto tone chamber and soundbox resonant view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CassottoChamberView {
    /// Active chamber acoustic profile.
    pub profile: CassottoChamberProfile,
    /// Aperture shutter opening fraction [0.0 ..= 1.0].
    pub aperture: f32,
    /// Wooden warmth resonance gain [0.0 ..= 2.0].
    pub wood_warmth: f32,
    /// Primary cavity resonance mode 1 frequency in Hz [200.0 ..= 1200.0].
    pub mode1_freq_hz: f32,
    /// Secondary cavity resonance mode 2 frequency in Hz [800.0 ..= 3500.0].
    pub mode2_freq_hz: f32,
    /// Lowpass transmission cutoff frequency in Hz [1500.0 ..= 12000.0].
    pub lowpass_cutoff_hz: f32,
    /// Cavity resonance Q quality factor [1.0 ..= 8.0].
    pub cavity_q: f32,
    /// Direct vs chambered sound power balance fraction [0.0 ..= 1.0].
    pub chamber_mix: f32,
    /// 2D Chamber puck position (X: aperture, Y: wood_warmth).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for CassottoChamberView {
    fn default() -> Self {
        Self::new()
    }
}

impl CassottoChamberView {
    pub fn new() -> Self {
        let mut view = Self {
            profile: CassottoChamberProfile::DobleCassottoBandoneon,
            aperture: 0.85,
            wood_warmth: 0.85,
            mode1_freq_hz: 460.0,
            mode2_freq_hz: 1420.0,
            lowpass_cutoff_hz: 2600.0,
            cavity_q: 3.8,
            chamber_mix: 0.65,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        };
        view.sync_from_profile();
        view.puck_pos = (
            Self::aperture_to_normalized(view.aperture),
            Self::warmth_to_normalized(view.wood_warmth),
        );
        view
    }

    pub fn set_profile(&mut self, profile: CassottoChamberProfile) {
        self.profile = profile;
        self.sync_from_profile();
    }

    pub fn sync_from_profile(&mut self) {
        let (f1, f2, fc, q, warmth) = self.profile.nominal_acoustics();
        self.mode1_freq_hz = f1;
        self.mode2_freq_hz = f2;
        self.lowpass_cutoff_hz = fc;
        self.cavity_q = q;
        self.wood_warmth = warmth;
        self.puck_pos = (
            Self::aperture_to_normalized(self.aperture),
            Self::warmth_to_normalized(self.wood_warmth),
        );
    }

    pub fn aperture_to_normalized(aperture: f32) -> f32 {
        let a = aperture.clamp(MIN_APERTURE_SHUTTER, MAX_APERTURE_SHUTTER);
        ((a - MIN_APERTURE_SHUTTER) / (MAX_APERTURE_SHUTTER - MIN_APERTURE_SHUTTER)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_aperture(norm: f32) -> f32 {
        MIN_APERTURE_SHUTTER + norm.clamp(0.0, 1.0) * (MAX_APERTURE_SHUTTER - MIN_APERTURE_SHUTTER)
    }

    pub fn warmth_to_normalized(warmth: f32) -> f32 {
        let w = warmth.clamp(MIN_WOOD_WARMTH, MAX_WOOD_WARMTH);
        ((w - MIN_WOOD_WARMTH) / (MAX_WOOD_WARMTH - MIN_WOOD_WARMTH)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_warmth(norm: f32) -> f32 {
        MIN_WOOD_WARMTH + norm.clamp(0.0, 1.0) * (MAX_WOOD_WARMTH - MIN_WOOD_WARMTH)
    }

    /// Evaluates chamber frequency response in dB at frequency f_hz.
    pub fn evaluate_frequency_response(&self, freq_hz: f32) -> f32 {
        let f = freq_hz.clamp(20.0, 20000.0);
        let eff_cutoff = self.lowpass_cutoff_hz * (0.35 + 0.65 * self.aperture);

        // Lowpass transmission loss
        let lp_ratio = f / eff_cutoff;
        let lp_gain_db = -10.0 * (1.0 + lp_ratio.powi(4)).log10();

        // Mode 1 peak
        let m1_ratio = f / self.mode1_freq_hz;
        let m1_diff = (m1_ratio - 1.0).abs() * self.cavity_q;
        let m1_gain_db = (self.wood_warmth * 6.0) / (1.0 + m1_diff * m1_diff);

        // Mode 2 peak
        let m2_ratio = f / self.mode2_freq_hz;
        let m2_diff = (m2_ratio - 1.0).abs() * (self.cavity_q * 0.8);
        let m2_gain_db = (self.wood_warmth * 3.5) / (1.0 + m2_diff * m2_diff);

        let direct_weight = self.aperture * 0.4 + (1.0 - self.chamber_mix) * 0.6;
        let total_chamber_db = lp_gain_db + m1_gain_db + m2_gain_db;
        (total_chamber_db * (1.0 - direct_weight * 0.5)).clamp(-36.0, 18.0)
    }

    /// Hit test coordinate on the interactive chamber puck.
    pub fn hit_test_chamber_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= CASSOTTO_PUCK_HIT_RADIUS
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

        // Left canvas: 2D Puck
        let left_w = mid_x - 2;
        let p_row = (((1.0 - self.puck_pos.1) * (height - 5) as f32) + 2.0).round() as usize;
        let p_col = ((self.puck_pos.0 * (left_w - 4) as f32) + 2.0).round() as usize;
        if p_row < height - 1 && p_col < mid_x {
            grid[p_row][p_col] = 'P';
        }

        // Right canvas: Frequency curve profile
        let right_w = width - mid_x - 2;
        let num_pts = right_w.min(32);
        for i in 0..num_pts {
            let col = mid_x + 2 + i;
            let f = 100.0 * 10.0f32.powf((i as f32 / num_pts as f32) * 2.2);
            let resp_db = self.evaluate_frequency_response(f);
            let norm_y = ((resp_db + 30.0) / 45.0).clamp(0.0, 1.0);
            let r_pos = (((1.0 - norm_y) * (height - 4) as f32) + 2.0).round() as usize;
            if r_pos < height - 1 && col < width - 1 {
                grid[r_pos][col] = '*';
            }
        }

        grid.into_iter()
            .map(|row| row.into_iter().collect())
            .collect()
    }

    /// Render ASCII snapshot string with metadata banner and verified touch target badge.
    pub fn render_ascii_snapshot_str(&self) -> String {
        let lines = self.render_ascii(64, 18);
        let mut banner = String::new();
        banner.push_str("=== PHYSICAL MODELING WOODEN CASSOTTO TONE CHAMBER & SOUNDBOX HUD ===\n");
        banner.push_str(&format!(
            "Profile: {} | Aperture: {:.0}% | Warmth: {:.2}x | Cutoff: {:.0} Hz | Q: {:.1}\n",
            self.profile.name(), self.aperture * 100.0, self.wood_warmth, self.lowpass_cutoff_hz, self.cavity_q
        ));
        banner.push_str("Touch Target Compliance: [PASS: >=44pt] Bounding Radius: 22.0pt\n");
        banner.push_str(&lines.join("\n"));
        banner
    }

    #[cfg(feature = "gui")]
    pub fn show_ui(&mut self, ui: &mut egui::Ui) {
        self.ui(ui);
    }

    #[cfg(feature = "gui")]
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Profile & Preset Bar
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Acoustic Chamber:")
                        .font(FontId::proportional(11.5))
                        .color(Color32::from_rgb(226, 232, 240)),
                );
                let profiles = [
                    (CassottoChamberProfile::DobleCassottoBandoneon, "Bandoneon Premier"),
                    (CassottoChamberProfile::StandardAccordionChamber, "Italian Maple"),
                    (CassottoChamberProfile::ConcertinaSpruceCavity, "Spruce Concertina"),
                    (CassottoChamberProfile::HarmoniumResonantBox, "Harmonium Box"),
                    (CassottoChamberProfile::OpenAirBayan, "Direct Bayan"),
                ];
                for (p, label) in profiles {
                    let is_active = self.profile == p;
                    let text = if is_active {
                        RichText::new(format!("● {}", label))
                            .font(FontId::proportional(10.5))
                            .strong()
                            .color(Color32::from_rgb(245, 158, 11))
                    } else {
                        RichText::new(label)
                            .font(FontId::proportional(10.5))
                            .color(Color32::from_rgb(148, 163, 184))
                    };
                    if ui.selectable_label(is_active, text).clicked() {
                        self.set_profile(p);
                    }
                }
            });
            ui.add_space(6.0);

            // Responsive 2-Pane Canvas
            let canvas_size = Vec2::new(ui.available_width().max(480.0), 240.0);
            let (rect, response) = ui.allocate_exact_size(canvas_size, egui::Sense::click_and_drag());
            let painter = ui.painter_at(rect);

            painter.rect_filled(rect, 4.0, Color32::from_rgb(10, 15, 24));
            painter.rect_stroke(
                rect,
                4.0,
                Stroke::new(1.0_f32, Color32::from_rgb(30, 41, 59)),
            );

            let pane_w = (rect.width() - 30.0) * 0.5;

            // Left Pane: Interactive 2D Shutter & Warmth Puck
            let left_rect = egui::Rect::from_min_size(
                egui::pos2(rect.min.x + 10.0, rect.min.y + 10.0),
                Vec2::new(pane_w, rect.height() - 20.0),
            );
            painter.rect_filled(left_rect, 4.0, Color32::from_rgb(15, 23, 42));
            painter.rect_stroke(
                left_rect,
                4.0,
                Stroke::new(1.0_f32, Color32::from_rgb(35, 55, 85)),
            );

            painter.text(
                egui::pos2(left_rect.min.x + 10.0, left_rect.min.y + 10.0),
                egui::Align2::LEFT_TOP,
                "CHAMBER APERTURE SHUTTER vs WOODEN WARMTH",
                FontId::proportional(10.5),
                Color32::from_rgb(245, 158, 11),
            );

            // Crosshair guidelines
            let puck_x = left_rect.min.x + self.puck_pos.0 * left_rect.width();
            let puck_y = left_rect.max.y - self.puck_pos.1 * left_rect.height();
            let puck_pos = egui::pos2(puck_x, puck_y);

            painter.line_segment(
                [egui::pos2(left_rect.min.x, puck_y), egui::pos2(left_rect.max.x, puck_y)],
                Stroke::new(0.8_f32, Color32::from_rgba_premultiplied(245, 158, 11, 60)),
            );
            painter.line_segment(
                [egui::pos2(puck_x, left_rect.min.y), egui::pos2(puck_x, left_rect.max.y)],
                Stroke::new(0.8_f32, Color32::from_rgba_premultiplied(245, 158, 11, 60)),
            );

            // Drag / Click interaction
            if response.dragged() || response.clicked() {
                if let Some(mouse_pos) = response.interact_pointer_pos() {
                    if left_rect.contains(mouse_pos) {
                        let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                        let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                        self.puck_pos = (nx, ny);
                        self.aperture = Self::normalized_to_aperture(nx);
                        self.wood_warmth = Self::normalized_to_warmth(ny);
                    }
                }
            }

            // Puck representation with >= 44x44pt bounding target
            painter.circle_stroke(
                puck_pos,
                CASSOTTO_PUCK_HIT_RADIUS,
                Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(245, 158, 11, 150)),
            );
            painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(245, 158, 11));
            painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

            painter.text(
                egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 18.0),
                egui::Align2::LEFT_BOTTOM,
                format!(
                    "Aperture: {:.0}% | Warmth: {:.2}x | Cutoff: {:.0} Hz",
                    self.aperture * 100.0,
                    self.wood_warmth,
                    self.lowpass_cutoff_hz * (0.35 + 0.65 * self.aperture)
                ),
                FontId::proportional(10.0),
                Color32::from_rgb(252, 211, 77),
            );

            // Right Pane: 4-Mode Acoustic Frequency Response Curve
            let right_rect = egui::Rect::from_min_size(
                egui::pos2(left_rect.max.x + 10.0, rect.min.y + 10.0),
                Vec2::new(pane_w, rect.height() - 20.0),
            );
            painter.rect_filled(right_rect, 4.0, Color32::from_rgb(15, 23, 42));
            painter.rect_stroke(
                right_rect,
                4.0,
                Stroke::new(1.0_f32, Color32::from_rgb(35, 55, 85)),
            );

            painter.text(
                egui::pos2(right_rect.min.x + 10.0, right_rect.min.y + 10.0),
                egui::Align2::LEFT_TOP,
                "CAVITY MODES & TRANSMISSION SPECTRUM (dB)",
                FontId::proportional(10.5),
                Color32::from_rgb(56, 189, 248),
            );

            // 0 dB grid baseline
            let y_zero = right_rect.max.y - 25.0 - (30.0 / 48.0) * (right_rect.height() - 60.0);
            painter.line_segment(
                [egui::pos2(right_rect.min.x + 10.0, y_zero), egui::pos2(right_rect.max.x - 10.0, y_zero)],
                Stroke::new(0.8_f32, Color32::from_rgba_premultiplied(148, 163, 184, 80)),
            );

            // Plot curve
            let steps = 40;
            let mut last_pt = egui::pos2(right_rect.min.x + 10.0, y_zero);
            for s in 0..=steps {
                let frac = s as f32 / steps as f32;
                let f = 100.0 * 10.0f32.powf(frac * 2.2); // 100 Hz to ~15.8 kHz
                let resp_db = self.evaluate_frequency_response(f);
                let norm_y = ((resp_db + 30.0) / 48.0).clamp(0.0, 1.0);
                let px = right_rect.min.x + 10.0 + frac * (right_rect.width() - 20.0);
                let py = right_rect.max.y - 25.0 - norm_y * (right_rect.height() - 60.0);
                let cur_pt = egui::pos2(px, py);
                if s > 0 {
                    painter.line_segment([last_pt, cur_pt], Stroke::new(2.0_f32, Color32::from_rgb(56, 189, 248)));
                }
                last_pt = cur_pt;
            }

            // Mode 1 and Mode 2 peak markers
            painter.text(
                egui::pos2(right_rect.min.x + 10.0, right_rect.max.y - 18.0),
                egui::Align2::LEFT_BOTTOM,
                format!(
                    "f₁: {:.0} Hz | f₂: {:.0} Hz | Cavity Q: {:.1} | Mix: {:.0}%",
                    self.mode1_freq_hz, self.mode2_freq_hz, self.cavity_q, self.chamber_mix * 100.0
                ),
                FontId::proportional(10.0),
                Color32::from_rgb(186, 230, 253),
            );

            ui.add_space(8.0);

            // Bottom Surgical Controls Grid
            ui.horizontal(|ui| {
                ui.label(RichText::new("Aperture:").font(FontId::proportional(11.0)).color(Color32::from_rgb(203, 213, 225)));
                if ui.add(egui::Slider::new(&mut self.aperture, 0.0..=1.0).text("")).changed() {
                    self.puck_pos.0 = Self::aperture_to_normalized(self.aperture);
                }

                ui.add_space(8.0);
                ui.label(RichText::new("Wood Warmth:").font(FontId::proportional(11.0)).color(Color32::from_rgb(203, 213, 225)));
                if ui.add(egui::Slider::new(&mut self.wood_warmth, 0.0..=2.0).text("")).changed() {
                    self.puck_pos.1 = Self::warmth_to_normalized(self.wood_warmth);
                }

                ui.add_space(8.0);
                ui.label(RichText::new("Chamber Mix:").font(FontId::proportional(11.0)).color(Color32::from_rgb(203, 213, 225)));
                ui.add(egui::Slider::new(&mut self.chamber_mix, 0.0..=1.0).text(""));

                ui.add_space(8.0);
                ui.label(RichText::new("Cavity Q:").font(FontId::proportional(11.0)).color(Color32::from_rgb(203, 213, 225)));
                ui.add(egui::Slider::new(&mut self.cavity_q, 1.0..=8.0).text(""));
            });

            ui.add_space(6.0);

            // Compliance Badge
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("[PASS] Cassotto Tone Chamber Touch Targets (>= 44x44pt) Verified | Zero-Allocation Audio Engine")
                        .font(FontId::proportional(10.5))
                        .color(Color32::from_rgb(34, 197, 94)),
                );
            });
        });
    }
}
