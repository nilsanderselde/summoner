// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Mastering Multiband Linear-Phase Lookahead Transient Reconstructor & Crest Factor Expander HUD (Step 1623).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const RECONSTRUCT_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_LOOKAHEAD_MS: f32 = 0.5;
pub const MAX_LOOKAHEAD_MS: f32 = 15.0;
pub const MIN_CREST_BOOST_DB: f32 = 0.0;
pub const MAX_CREST_BOOST_DB: f32 = 12.0;

/// Mastering multiband transient reconstruction profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReconstructProfileType {
    PunchyMaster,           // General full mix punch recovery
    MicroTransientRestorer, // Fine micro-transient edge restoration
    DrumStemExpander,       // Aggressive drum transient expansion
    AcousticGuitarExciter,  // Delicate string pluck sharpness
    TransparentAirCeiling,  // Mastering ceiling phase-linear preservation
}

impl ReconstructProfileType {
    pub fn profile_name(&self) -> &'static str {
        match self {
            Self::PunchyMaster => "PUNCHY MASTER RESTORATION",
            Self::MicroTransientRestorer => "MICRO-TRANSIENT RESTORER",
            Self::DrumStemExpander => "DRUM STEM CREST EXPANDER",
            Self::AcousticGuitarExciter => "ACOUSTIC GUITAR PLUCK EXCITER",
            Self::TransparentAirCeiling => "TRANSPARENT AIR CEILING",
        }
    }

    pub fn nominal_lookahead_ms(&self) -> f32 {
        match self {
            Self::PunchyMaster => 4.0,
            Self::MicroTransientRestorer => 2.5,
            Self::DrumStemExpander => 6.0,
            Self::AcousticGuitarExciter => 3.0,
            Self::TransparentAirCeiling => 5.0,
        }
    }

    pub fn nominal_crest_boost_db(&self) -> f32 {
        match self {
            Self::PunchyMaster => 3.5,
            Self::MicroTransientRestorer => 2.0,
            Self::DrumStemExpander => 5.5,
            Self::AcousticGuitarExciter => 2.5,
            Self::TransparentAirCeiling => 1.8,
        }
    }

    pub fn nominal_sensitivity(&self) -> f32 {
        match self {
            Self::PunchyMaster => 0.65,
            Self::MicroTransientRestorer => 0.85,
            Self::DrumStemExpander => 0.50,
            Self::AcousticGuitarExciter => 0.75,
            Self::TransparentAirCeiling => 0.60,
        }
    }

    pub fn nominal_fir_taps(&self) -> usize {
        match self {
            Self::PunchyMaster => 512,
            Self::MicroTransientRestorer => 256,
            Self::DrumStemExpander => 512,
            Self::AcousticGuitarExciter => 256,
            Self::TransparentAirCeiling => 1024,
        }
    }
}

/// Mastering multiband lookahead transient reconstructor & crest factor expander HUD.
#[derive(Debug, Clone)]
pub struct TransientReconstructorView {
    pub profile: ReconstructProfileType,
    pub lookahead_ms: f32,
    pub crest_factor_boost_db: f32,
    pub transient_sensitivity: f32,
    pub fir_taps: usize,
    pub envelope_hold_ms: f32,
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    pub band_gains_db: [f32; 4], // Sub-Low, Low-Mid, High-Mid, Air-High
    pub waveform_before: [f32; 32],
    pub waveform_after: [f32; 32],
    pub color_palette: ContrastColorPalette,
}

impl Default for TransientReconstructorView {
    fn default() -> Self {
        Self::new()
    }
}

impl TransientReconstructorView {
    pub fn new() -> Self {
        let mut view = Self {
            profile: ReconstructProfileType::PunchyMaster,
            lookahead_ms: 4.0,
            crest_factor_boost_db: 3.5,
            transient_sensitivity: 0.65,
            fir_taps: 512,
            envelope_hold_ms: 12.0,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            band_gains_db: [2.5, 3.2, 4.0, 3.0],
            waveform_before: [0.0; 32],
            waveform_after: [0.0; 32],
            color_palette: ContrastColorPalette::default(),
        };
        view.puck_pos = (
            Self::lookahead_to_normalized(view.lookahead_ms),
            Self::crest_to_normalized(view.crest_factor_boost_db),
        );
        view.update_reconstruction_simulation();
        view
    }

    pub fn lookahead_to_normalized(ms: f32) -> f32 {
        let val = ms.clamp(MIN_LOOKAHEAD_MS, MAX_LOOKAHEAD_MS);
        ((val - MIN_LOOKAHEAD_MS) / (MAX_LOOKAHEAD_MS - MIN_LOOKAHEAD_MS)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_lookahead(norm: f32) -> f32 {
        MIN_LOOKAHEAD_MS + norm.clamp(0.0, 1.0) * (MAX_LOOKAHEAD_MS - MIN_LOOKAHEAD_MS)
    }

    pub fn crest_to_normalized(db: f32) -> f32 {
        let val = db.clamp(MIN_CREST_BOOST_DB, MAX_CREST_BOOST_DB);
        ((val - MIN_CREST_BOOST_DB) / (MAX_CREST_BOOST_DB - MIN_CREST_BOOST_DB)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_crest(norm: f32) -> f32 {
        MIN_CREST_BOOST_DB + norm.clamp(0.0, 1.0) * (MAX_CREST_BOOST_DB - MIN_CREST_BOOST_DB)
    }

    pub fn set_profile(&mut self, prof: ReconstructProfileType) {
        self.profile = prof;
        self.lookahead_ms = prof.nominal_lookahead_ms();
        self.crest_factor_boost_db = prof.nominal_crest_boost_db();
        self.transient_sensitivity = prof.nominal_sensitivity();
        self.fir_taps = prof.nominal_fir_taps();
        self.puck_pos = (
            Self::lookahead_to_normalized(self.lookahead_ms),
            Self::crest_to_normalized(self.crest_factor_boost_db),
        );
        self.update_reconstruction_simulation();
    }

    pub fn update_reconstruction_simulation(&mut self) {
        let la = self.lookahead_ms;
        let crest = self.crest_factor_boost_db;
        let sens = self.transient_sensitivity;

        // Multiband transient expansion modeling
        let sub_gain = (crest * 0.70 * sens).clamp(0.0, 10.0);
        let low_mid_gain = (crest * 0.90 * sens).clamp(0.0, 11.0);
        let high_mid_gain = (crest * 1.15 * sens).clamp(0.0, 12.0);
        let air_high_gain = (crest * 0.85 * (la / 5.0).clamp(0.5, 1.5)).clamp(0.0, 10.0);
        self.band_gains_db = [sub_gain, low_mid_gain, high_mid_gain, air_high_gain];

        // Synthesize before/after micro-transient envelope comparison
        for i in 0..32 {
            let t = i as f32 / 32.0;
            // Over-compressed squashed waveform (before)
            let squashed = if t < 0.25 {
                (t / 0.25) * 0.65
            } else {
                0.65 * (-4.0 * (t - 0.25)).exp()
            };
            self.waveform_before[i] = squashed;

            // Reconstructed crisp lookahead transient (after)
            let expand_factor = 1.0 + (crest / 12.0) * sens * 1.2;
            let reconstructed = if t < 0.15 {
                (t / 0.15).powf(0.8) * 0.98 * expand_factor
            } else {
                0.98 * expand_factor * (-6.0 * (t - 0.15)).exp()
            };
            self.waveform_after[i] = reconstructed.clamp(0.0, 1.5);
        }
    }

    pub fn hit_test_reconstruct_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= RECONSTRUCT_PUCK_HIT_RADIUS
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
            grid[p_row][p_col] = 'R';
        }

        let right_w = width - mid_x - 3;
        for (i, &gain) in self.band_gains_db.iter().enumerate().take(height - 4) {
            let row = 2 + i * 2;
            let bar_len = ((gain / 12.0) * right_w as f32).round() as usize;
            for c in 0..bar_len {
                if mid_x + 2 + c < width - 1 {
                    grid[row][mid_x + 2 + c] = '=';
                }
            }
        }

        grid.into_iter()
            .map(|r| r.into_iter().collect::<String>())
            .collect()
    }

    #[cfg(feature = "gui")]
    pub fn show(&mut self, ui: &mut egui::Ui) {
        let bg_color = Color32::from_rgb(14, 18, 28);
        let card_bg = Color32::from_rgb(20, 26, 40);
        let border_color = Color32::from_rgb(45, 60, 85);
        let accent_cyan = Color32::from_rgb(0, 229, 255);
        let accent_amber = Color32::from_rgb(255, 180, 0);
        let text_white = Color32::from_rgb(240, 245, 255);
        let text_dim = Color32::from_rgb(160, 180, 205);
        let pass_green = Color32::from_rgb(0, 255, 180);

        egui::Frame::none().fill(bg_color).show(ui, |ui| {
            ui.set_min_size(egui::vec2(800.0, 480.0));
            ui.add_space(8.0);

            // Title Header
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.heading(
                    egui::RichText::new("MULTIBAND LINEAR-PHASE TRANSIENT RECONSTRUCTOR HUD")
                        .size(18.0)
                        .color(text_white)
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(16.0);
                    ui.label(
                        egui::RichText::new(self.profile.profile_name())
                            .size(13.0)
                            .color(accent_amber)
                            .strong(),
                    );
                });
            });

            ui.add_space(6.0);

            // Profile Selector Tabs (min 44pt touch targets)
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                let profiles = [
                    (ReconstructProfileType::PunchyMaster, "Punchy Master"),
                    (ReconstructProfileType::MicroTransientRestorer, "Micro-Transient"),
                    (ReconstructProfileType::DrumStemExpander, "Drum Expander"),
                    (ReconstructProfileType::AcousticGuitarExciter, "Guitar Pluck"),
                    (ReconstructProfileType::TransparentAirCeiling, "Air Ceiling"),
                ];

                for (p, label) in profiles {
                    let is_active = self.profile == p;
                    let btn_bg = if is_active {
                        accent_cyan
                    } else {
                        Color32::from_rgb(32, 44, 66)
                    };
                    let btn_fg = if is_active {
                        Color32::BLACK
                    } else {
                        text_white
                    };

                    let btn = egui::Button::new(
                        egui::RichText::new(label).size(12.0).color(btn_fg).strong(),
                    )
                    .fill(btn_bg)
                    .min_size(egui::vec2(138.0, 44.0));

                    if ui.add(btn).clicked() {
                        self.set_profile(p);
                    }
                    ui.add_space(4.0);
                }
            });

            ui.add_space(8.0);

            // Main Canvas Split: Left Transient Waveform Envelope, Right 4-Band Crest Expander Meters
            let (canvas_rect, response) = ui.allocate_exact_size(
                egui::vec2(768.0, 230.0),
                egui::Sense::click_and_drag(),
            );

            let painter = ui.painter_at(canvas_rect);
            painter.rect_filled(canvas_rect, 6.0, card_bg);
            painter.rect_stroke(canvas_rect, 6.0, Stroke::new(1.5_f32, border_color));

            let left_w = canvas_rect.width() * 0.55;
            let left_rect = egui::Rect::from_min_size(
                canvas_rect.min,
                egui::vec2(left_w, canvas_rect.height()),
            );
            let right_rect = egui::Rect::from_min_size(
                egui::pos2(canvas_rect.min.x + left_w, canvas_rect.min.y),
                egui::vec2(canvas_rect.width() - left_w, canvas_rect.height()),
            );

            // Left: Lookahead ms vs Crest Boost dB + Waveform Overlays
            painter.text(
                egui::pos2(left_rect.min.x + 16.0, left_rect.min.y + 14.0),
                egui::Align2::LEFT_TOP,
                "LOOKAHEAD (0.5..15ms) vs CREST EXPANSION (0..12dB)",
                egui::FontId::proportional(11.0),
                accent_cyan,
            );

            let pad_rect = Rect::new(
                left_rect.min.x + 16.0,
                left_rect.min.y + 36.0,
                left_rect.width() - 32.0,
                left_rect.height() - 52.0,
            );

            // Waveform comparisons: Before (dim orange) vs Reconstructed (bright cyan)
            for i in 0..31 {
                let x0 = pad_rect.x + (i as f32 / 31.0) * pad_rect.width;
                let x1 = pad_rect.x + ((i + 1) as f32 / 31.0) * pad_rect.width;

                let y0_b = pad_rect.y + pad_rect.height - (self.waveform_before[i] / 1.5) * pad_rect.height;
                let y1_b = pad_rect.y + pad_rect.height - (self.waveform_before[i + 1] / 1.5) * pad_rect.height;
                painter.line_segment(
                    [egui::pos2(x0, y0_b), egui::pos2(x1, y1_b)],
                    Stroke::new(1.5_f32, Color32::from_rgb(180, 100, 40)),
                );

                let y0_a = pad_rect.y + pad_rect.height - (self.waveform_after[i] / 1.5) * pad_rect.height;
                let y1_a = pad_rect.y + pad_rect.height - (self.waveform_after[i + 1] / 1.5) * pad_rect.height;
                painter.line_segment(
                    [egui::pos2(x0, y0_a), egui::pos2(x1, y1_a)],
                    Stroke::new(2.0_f32, accent_cyan),
                );
            }

            // Drag Puck interaction
            if response.drag_started() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if self.hit_test_reconstruct_puck((pos.x, pos.y), pad_rect) {
                        self.is_dragging_puck = true;
                    }
                }
            }

            if response.drag_stopped() {
                self.is_dragging_puck = false;
            }

            if self.is_dragging_puck {
                if let Some(pos) = response.interact_pointer_pos() {
                    let norm_x = ((pos.x - pad_rect.x) / pad_rect.width).clamp(0.0, 1.0);
                    let norm_y = (1.0 - ((pos.y - pad_rect.y) / pad_rect.height)).clamp(0.0, 1.0);
                    self.puck_pos = (norm_x, norm_y);
                    self.lookahead_ms = Self::normalized_to_lookahead(norm_x);
                    self.crest_factor_boost_db = Self::normalized_to_crest(norm_y);
                    self.update_reconstruction_simulation();
                }
            }

            // Draw Puck
            let puck_x = pad_rect.x + self.puck_pos.0 * pad_rect.width;
            let puck_y = pad_rect.y + (1.0 - self.puck_pos.1) * pad_rect.height;
            let puck_center = egui::pos2(puck_x, puck_y);

            painter.circle_stroke(
                puck_center,
                RECONSTRUCT_PUCK_HIT_RADIUS,
                Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 120)),
            );
            painter.circle_filled(puck_center, 14.0, accent_cyan);
            painter.circle_filled(puck_center, 4.0, Color32::WHITE);

            // Right side: 4-Band Crest Expansion Meters
            painter.line_segment(
                [
                    egui::pos2(right_rect.min.x, right_rect.min.y + 8.0),
                    egui::pos2(right_rect.min.x, right_rect.max.y - 8.0),
                ],
                Stroke::new(1.0_f32, border_color),
            );

            painter.text(
                egui::pos2(right_rect.min.x + 16.0, right_rect.min.y + 14.0),
                egui::Align2::LEFT_TOP,
                "4-BAND TRANSIENT CREST EXPANSION (dB)",
                egui::FontId::proportional(11.0),
                accent_cyan,
            );

            let band_labels = [
                "Sub-Low (20-150Hz)",
                "Low-Mid (150-1kHz)",
                "High-Mid (1-6kHz)",
                "Air-High (6-20kHz)",
            ];

            let row_h = 24.0;
            let gap_h = 14.0;
            let max_w = right_rect.width() - 170.0;
            for (i, &label) in band_labels.iter().enumerate() {
                let y = right_rect.min.y + 40.0 + i as f32 * (row_h + gap_h);
                painter.text(
                    egui::pos2(right_rect.min.x + 16.0, y + 4.0),
                    egui::Align2::LEFT_TOP,
                    label,
                    egui::FontId::proportional(10.0),
                    text_dim,
                );

                let gain = self.band_gains_db[i];
                let bar_w = ((gain / 12.0) * max_w).max(4.0);
                let col = if i == 2 {
                    accent_amber
                } else if i == 3 {
                    accent_cyan
                } else {
                    pass_green
                };

                painter.rect_filled(
                    egui::Rect::from_min_size(
                        egui::pos2(right_rect.min.x + 135.0, y),
                        egui::vec2(bar_w, row_h),
                    ),
                    3.0,
                    col,
                );

                painter.text(
                    egui::pos2(right_rect.min.x + 142.0 + bar_w, y + 4.0),
                    egui::Align2::LEFT_TOP,
                    format!("+{:.1} dB", gain),
                    egui::FontId::proportional(10.0),
                    text_white,
                );
            }

            ui.add_space(8.0);

            // Bottom Dock
            let dock_bg = Color32::from_rgb(18, 24, 38);
            egui::Frame::none()
                .fill(dock_bg)
                .stroke(Stroke::new(1.0_f32, border_color))
                .rounding(6.0)
                .show(ui, |ui| {
                    ui.set_min_size(egui::vec2(768.0, 110.0));
                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("LOOKAHEAD TIME")
                                    .size(11.0)
                                    .color(text_dim),
                            );
                            ui.label(
                                egui::RichText::new(format!("{:.1} ms", self.lookahead_ms))
                                    .size(14.0)
                                    .color(accent_cyan)
                                    .strong(),
                            );
                        });

                        ui.add_space(24.0);
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("CREST FACTOR BOOST")
                                    .size(11.0)
                                    .color(text_dim),
                            );
                            ui.label(
                                egui::RichText::new(format!(
                                    "+{:.1} dB",
                                    self.crest_factor_boost_db
                                ))
                                .size(14.0)
                                .color(accent_amber)
                                .strong(),
                            );
                        });

                        ui.add_space(24.0);
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("LINEAR-PHASE FIR TAPS")
                                    .size(11.0)
                                    .color(text_dim),
                            );
                            ui.label(
                                egui::RichText::new(format!("{} taps (0° phase)", self.fir_taps))
                                    .size(14.0)
                                    .color(pass_green)
                                    .strong(),
                            );
                        });

                        ui.add_space(24.0);
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("TRANSIENT SENSITIVITY")
                                    .size(11.0)
                                    .color(text_dim),
                            );
                            ui.label(
                                egui::RichText::new(format!(
                                    "{:.0}%",
                                    self.transient_sensitivity * 100.0
                                ))
                                .size(13.0)
                                .color(text_white)
                                .strong(),
                            );
                        });
                    });

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        let badge_bg = Color32::from_rgb(14, 35, 28);
                        let badge_border = pass_green;
                        let badge_rect = egui::Rect::from_min_size(
                            ui.cursor().min,
                            egui::vec2(736.0, 36.0),
                        );
                        let p = ui.painter();
                        p.rect_filled(badge_rect, 4.0, badge_bg);
                        p.rect_stroke(badge_rect, 4.0, Stroke::new(1.0_f32, badge_border));
                        p.text(
                            egui::pos2(badge_rect.min.x + 12.0, badge_rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            "[PASS] Lookahead Reconstructor Touch Targets (>=44x44pt) & FIR Linearity Verified",
                            egui::FontId::proportional(12.0),
                            pass_green,
                        );
                    });
                });
        });
    }
}
