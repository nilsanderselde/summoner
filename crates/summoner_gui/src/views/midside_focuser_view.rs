// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Mastering Linear-Phase Stereo Mid/Side Dynamic Punch & Multiband Mono-Bass Focuser HUD (Step 1633).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const MIDSIDE_FOCUSER_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_MONO_CUTOFF_HZ: f32 = 20.0;
pub const MAX_MONO_CUTOFF_HZ: f32 = 300.0;
pub const MIN_MID_PUNCH_DB: f32 = 0.0;
pub const MAX_MID_PUNCH_DB: f32 = 12.0;
pub const MIN_SIDE_WIDTH_RATIO: f32 = 0.0;
pub const MAX_SIDE_WIDTH_RATIO: f32 = 2.0;

/// Mastering Mid/Side focusing and punch profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MidSideProfileType {
    ClubSubTightener,     // Strict mono sub below 120Hz, expanded wide side air
    VinylMasterMonofier,  // Strict elliptical mono below 150Hz for needle tracking
    AcousticLiveFocus,    // Gentle mono bass below 80Hz, natural transparent mid punch
    DynamicEdmSlam,       // Aggressive 140Hz mono cutoff, +6.5dB mid punch transient
    BroadcastClaritySafe, // EBU R128 safe mono bass below 100Hz, +1.0 stereo correlation
}

impl MidSideProfileType {
    pub fn profile_name(&self) -> &'static str {
        match self {
            Self::ClubSubTightener => "CLUB SUB TIGHTENER (120Hz)",
            Self::VinylMasterMonofier => "VINYL MASTER ELLIPTICAL MONO",
            Self::AcousticLiveFocus => "ACOUSTIC LIVE MID-FOCUS",
            Self::DynamicEdmSlam => "DYNAMIC EDM STEREO SLAM",
            Self::BroadcastClaritySafe => "BROADCAST CLARITY SAFE",
        }
    }

    pub fn nominal_mono_cutoff_hz(&self) -> f32 {
        match self {
            Self::ClubSubTightener => 120.0,
            Self::VinylMasterMonofier => 150.0,
            Self::AcousticLiveFocus => 80.0,
            Self::DynamicEdmSlam => 140.0,
            Self::BroadcastClaritySafe => 100.0,
        }
    }

    pub fn nominal_mid_punch_db(&self) -> f32 {
        match self {
            Self::ClubSubTightener => 4.0,
            Self::VinylMasterMonofier => 2.0,
            Self::AcousticLiveFocus => 1.5,
            Self::DynamicEdmSlam => 6.5,
            Self::BroadcastClaritySafe => 2.5,
        }
    }

    pub fn nominal_side_width(&self) -> f32 {
        match self {
            Self::ClubSubTightener => 1.35,
            Self::VinylMasterMonofier => 0.95,
            Self::AcousticLiveFocus => 1.10,
            Self::DynamicEdmSlam => 1.60,
            Self::BroadcastClaritySafe => 1.00,
        }
    }

    pub fn nominal_fir_taps(&self) -> usize {
        match self {
            Self::ClubSubTightener => 512,
            Self::VinylMasterMonofier => 1024,
            Self::AcousticLiveFocus => 256,
            Self::DynamicEdmSlam => 512,
            Self::BroadcastClaritySafe => 512,
        }
    }
}

/// Mastering linear-phase stereo mid/side dynamic punch & mono-bass focuser HUD.
#[derive(Debug, Clone)]
pub struct MidSideFocuserView {
    pub profile: MidSideProfileType,
    pub mono_cutoff_hz: f32,
    pub mid_punch_boost_db: f32,
    pub side_width_ratio: f32,
    pub fir_taps: usize,
    pub stereo_correlation: f32, // [-1.0, +1.0]
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    pub band_energies: [f32; 4], // Sub-Mono, Low-Mid, Mid-High, High-Side
    pub color_palette: ContrastColorPalette,
}

impl Default for MidSideFocuserView {
    fn default() -> Self {
        Self::new()
    }
}

impl MidSideFocuserView {
    pub fn new() -> Self {
        let mut view = Self {
            profile: MidSideProfileType::ClubSubTightener,
            mono_cutoff_hz: 120.0,
            mid_punch_boost_db: 4.0,
            side_width_ratio: 1.35,
            fir_taps: 512,
            stereo_correlation: 0.85,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            band_energies: [1.0, 0.82, 0.74, 0.90],
            color_palette: ContrastColorPalette::default(),
        };
        view.puck_pos = (
            Self::cutoff_to_normalized(view.mono_cutoff_hz),
            Self::punch_to_normalized(view.mid_punch_boost_db),
        );
        view.update_midside_simulation();
        view
    }

    pub fn cutoff_to_normalized(hz: f32) -> f32 {
        let val = hz.clamp(MIN_MONO_CUTOFF_HZ, MAX_MONO_CUTOFF_HZ);
        ((val - MIN_MONO_CUTOFF_HZ) / (MAX_MONO_CUTOFF_HZ - MIN_MONO_CUTOFF_HZ)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_cutoff(norm: f32) -> f32 {
        MIN_MONO_CUTOFF_HZ + norm.clamp(0.0, 1.0) * (MAX_MONO_CUTOFF_HZ - MIN_MONO_CUTOFF_HZ)
    }

    pub fn punch_to_normalized(db: f32) -> f32 {
        let val = db.clamp(MIN_MID_PUNCH_DB, MAX_MID_PUNCH_DB);
        ((val - MIN_MID_PUNCH_DB) / (MAX_MID_PUNCH_DB - MIN_MID_PUNCH_DB)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_punch(norm: f32) -> f32 {
        MIN_MID_PUNCH_DB + norm.clamp(0.0, 1.0) * (MAX_MID_PUNCH_DB - MIN_MID_PUNCH_DB)
    }

    pub fn set_profile(&mut self, prof: MidSideProfileType) {
        self.profile = prof;
        self.mono_cutoff_hz = prof.nominal_mono_cutoff_hz();
        self.mid_punch_boost_db = prof.nominal_mid_punch_db();
        self.side_width_ratio = prof.nominal_side_width();
        self.fir_taps = prof.nominal_fir_taps();
        self.puck_pos = (
            Self::cutoff_to_normalized(self.mono_cutoff_hz),
            Self::punch_to_normalized(self.mid_punch_boost_db),
        );
        self.update_midside_simulation();
    }

    pub fn update_midside_simulation(&mut self) {
        let cutoff = self.mono_cutoff_hz.clamp(20.0, 300.0);
        let punch = self.mid_punch_boost_db.clamp(0.0, 12.0);
        let width = self.side_width_ratio.clamp(0.0, 2.0);

        // Correlation calculation based on side width and mono bass summing
        let base_corr = (1.0 - (width - 1.0) * 0.45).clamp(0.1, 1.0);
        let mono_boost = (cutoff / 300.0) * 0.25;
        self.stereo_correlation = (base_corr + mono_boost).clamp(-1.0, 1.0);

        // 4-band Mid/Side energy calculations
        let sub_mono_energy = (1.0 + (cutoff / 200.0) * 0.2).clamp(0.2, 1.35);
        let low_mid_energy = (0.75 + (punch / 12.0) * 0.4).clamp(0.2, 1.35);
        let mid_high_energy = (0.70 + (punch / 15.0) * 0.3 + (width - 1.0) * 0.15).clamp(0.2, 1.35);
        let high_side_energy = (width * 0.65 + 0.25).clamp(0.1, 1.45);

        self.band_energies = [
            sub_mono_energy,
            low_mid_energy,
            mid_high_energy,
            high_side_energy,
        ];
    }

    pub fn hit_test_focuser_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= MIDSIDE_FOCUSER_PUCK_HIT_RADIUS
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

        let right_w = width - mid_x - 3;
        for (i, &energy) in self.band_energies.iter().enumerate().take(height - 4) {
            let row = 2 + i;
            let bar_len = ((energy.clamp(0.0, 1.45) / 1.45) * right_w as f32).round() as usize;
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
        let accent_green = Color32::from_rgb(0, 255, 180);
        let accent_amber = Color32::from_rgb(255, 180, 0);
        let text_white = Color32::from_rgb(240, 245, 255);
        let text_dim = Color32::from_rgb(160, 180, 205);

        egui::Frame::none().fill(bg_color).show(ui, |ui| {
            ui.set_min_size(egui::vec2(800.0, 480.0));
            ui.add_space(8.0);

            // Title and Header
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.heading(
                    egui::RichText::new(
                        "LINEAR-PHASE STEREO MID/SIDE DYNAMIC PUNCH & MONO-BASS HUD",
                    )
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

            // Profile selector tabs (min 44pt touch hit targets)
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                let profiles = [
                    (MidSideProfileType::ClubSubTightener, "Club Sub (120Hz)"),
                    (MidSideProfileType::VinylMasterMonofier, "Vinyl Master Mono"),
                    (MidSideProfileType::AcousticLiveFocus, "Acoustic Live Focus"),
                    (MidSideProfileType::DynamicEdmSlam, "Dynamic EDM Slam"),
                    (MidSideProfileType::BroadcastClaritySafe, "Broadcast Safe"),
                ];

                for (prof, label) in profiles {
                    let is_active = self.profile == prof;
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
                        self.set_profile(prof);
                    }
                    ui.add_space(4.0);
                }
            });

            ui.add_space(8.0);

            // Main interactive 2D Canvas split: Left XY Pad (Mono Cutoff vs Mid Punch), Right Multiband M/S & Correlation
            let (canvas_rect, response) =
                ui.allocate_exact_size(egui::vec2(768.0, 230.0), egui::Sense::click_and_drag());

            let painter = ui.painter_at(canvas_rect);
            painter.rect_filled(canvas_rect, 6.0, card_bg);
            painter.rect_stroke(canvas_rect, 6.0, Stroke::new(1.5_f32, border_color));

            let left_w = canvas_rect.width() * 0.52;
            let left_rect = egui::Rect::from_min_size(
                canvas_rect.min,
                egui::vec2(left_w, canvas_rect.height()),
            );
            let right_rect = egui::Rect::from_min_size(
                egui::pos2(canvas_rect.min.x + left_w, canvas_rect.min.y),
                egui::vec2(canvas_rect.width() - left_w, canvas_rect.height()),
            );

            // Left Section (XY Pad)
            painter.line_segment(
                [
                    egui::pos2(left_rect.max.x, left_rect.min.y + 10.0),
                    egui::pos2(left_rect.max.x, left_rect.max.y - 10.0),
                ],
                Stroke::new(1.0_f32, border_color),
            );

            for g in 1..4 {
                let frac = g as f32 * 0.25;
                let gx = left_rect.min.x + 12.0 + (left_rect.width() - 24.0) * frac;
                let gy = left_rect.min.y + 12.0 + (left_rect.height() - 24.0) * frac;

                painter.line_segment(
                    [
                        egui::pos2(gx, left_rect.min.y + 12.0),
                        egui::pos2(gx, left_rect.max.y - 12.0),
                    ],
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 110, 80)),
                );
                painter.line_segment(
                    [
                        egui::pos2(left_rect.min.x + 12.0, gy),
                        egui::pos2(left_rect.max.x - 12.0, gy),
                    ],
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 110, 80)),
                );
            }

            let pad_inner = egui::Rect::from_min_max(
                egui::pos2(left_rect.min.x + 16.0, left_rect.min.y + 16.0),
                egui::pos2(left_rect.max.x - 16.0, left_rect.max.y - 16.0),
            );

            if response.dragged() || response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let norm_x = ((pos.x - pad_inner.min.x) / pad_inner.width()).clamp(0.0, 1.0);
                    let norm_y =
                        (1.0 - ((pos.y - pad_inner.min.y) / pad_inner.height())).clamp(0.0, 1.0);
                    self.puck_pos = (norm_x, norm_y);
                    self.mono_cutoff_hz = Self::normalized_to_cutoff(norm_x);
                    self.mid_punch_boost_db = Self::normalized_to_punch(norm_y);
                    self.update_midside_simulation();
                }
            }

            let puck_screen_x = pad_inner.min.x + self.puck_pos.0 * pad_inner.width();
            let puck_screen_y = pad_inner.min.y + (1.0 - self.puck_pos.1) * pad_inner.height();
            let puck_center = egui::pos2(puck_screen_x, puck_screen_y);

            painter.line_segment(
                [
                    egui::pos2(pad_inner.min.x, puck_center.y),
                    egui::pos2(pad_inner.max.x, puck_center.y),
                ],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 120)),
            );
            painter.line_segment(
                [
                    egui::pos2(puck_center.x, pad_inner.min.y),
                    egui::pos2(puck_center.x, pad_inner.max.y),
                ],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 120)),
            );

            painter.circle_stroke(
                puck_center,
                MIDSIDE_FOCUSER_PUCK_HIT_RADIUS,
                Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 140)),
            );
            painter.circle_filled(puck_center, 12.0, accent_cyan);
            painter.circle_filled(puck_center, 4.0, Color32::WHITE);

            painter.text(
                egui::pos2(pad_inner.min.x, pad_inner.min.y - 12.0),
                egui::Align2::LEFT_BOTTOM,
                "MONO-BASS CUTOFF (X) / MID PUNCH BOOST (Y)",
                egui::FontId::proportional(11.0),
                accent_cyan,
            );

            // Right Section (Multiband M/S & Correlation)
            painter.text(
                egui::pos2(right_rect.min.x + 16.0, right_rect.min.y + 14.0),
                egui::Align2::LEFT_TOP,
                "MULTIBAND MID/SIDE ENERGY & CORRELATION",
                egui::FontId::proportional(11.0),
                accent_green,
            );

            let band_labels = [
                "Sub-Bass (Mono-Focused)",
                "Low-Mid Punch Band",
                "Mid-High Presence",
                "High-Side Air Expansion",
            ];

            let bar_y_start = right_rect.min.y + 36.0;
            let bar_h = 18.0;
            let bar_spacing = 26.0;
            let bar_max_w = right_rect.width() - 180.0;

            for (i, &energy) in self.band_energies.iter().enumerate() {
                let y = bar_y_start + i as f32 * bar_spacing;
                let label = band_labels[i];

                painter.text(
                    egui::pos2(right_rect.min.x + 16.0, y + 9.0),
                    egui::Align2::LEFT_CENTER,
                    label,
                    egui::FontId::proportional(10.0),
                    text_dim,
                );

                let bar_x = right_rect.min.x + 145.0;
                let bar_rect_bg =
                    egui::Rect::from_min_size(egui::pos2(bar_x, y), egui::vec2(bar_max_w, bar_h));
                painter.rect_filled(bar_rect_bg, 3.0, Color32::from_rgb(12, 16, 24));
                painter.rect_stroke(bar_rect_bg, 3.0, Stroke::new(1.0_f32, border_color));

                let fill_w = (energy.clamp(0.0, 1.5) / 1.5) * bar_max_w;
                let fill_color = if i == 0 {
                    accent_cyan // Sub-mono
                } else if i == 1 {
                    accent_green // Mid punch
                } else if i == 2 {
                    accent_amber // Mid-high
                } else {
                    Color32::from_rgb(255, 107, 43) // Side air
                };

                let bar_fill =
                    egui::Rect::from_min_size(egui::pos2(bar_x, y), egui::vec2(fill_w, bar_h));
                painter.rect_filled(bar_fill, 3.0, fill_color);

                let val_str = format!("{:.2}", energy);
                painter.text(
                    egui::pos2(bar_x + bar_max_w + 8.0, y + 9.0),
                    egui::Align2::LEFT_CENTER,
                    val_str,
                    egui::FontId::proportional(10.0),
                    text_white,
                );
            }

            // Correlation Meter at bottom of right rect
            let corr_y = bar_y_start + 4.0 * bar_spacing + 4.0;
            painter.text(
                egui::pos2(right_rect.min.x + 16.0, corr_y + 8.0),
                egui::Align2::LEFT_CENTER,
                "Stereo Correlation",
                egui::FontId::proportional(10.0),
                text_dim,
            );
            let corr_bar_x = right_rect.min.x + 145.0;
            let corr_bar_bg = egui::Rect::from_min_size(
                egui::pos2(corr_bar_x, corr_y),
                egui::vec2(bar_max_w, 14.0),
            );
            painter.rect_filled(corr_bar_bg, 2.0, Color32::from_rgb(12, 16, 24));
            painter.rect_stroke(corr_bar_bg, 2.0, Stroke::new(1.0_f32, border_color));

            let corr_norm = ((self.stereo_correlation + 1.0) / 2.0).clamp(0.0, 1.0);
            let corr_puck_x = corr_bar_x + corr_norm * bar_max_w;
            painter.circle_filled(egui::pos2(corr_puck_x, corr_y + 7.0), 6.0, accent_green);

            painter.text(
                egui::pos2(corr_bar_x + bar_max_w + 8.0, corr_y + 8.0),
                egui::Align2::LEFT_CENTER,
                format!("{:.2}", self.stereo_correlation),
                egui::FontId::proportional(10.0),
                text_white,
            );

            ui.add_space(8.0);

            // Bottom controls: Sliders
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new("MONO CUTOFF:")
                        .size(12.0)
                        .color(accent_cyan)
                        .strong(),
                );
                let cutoff_slider = egui::Slider::new(&mut self.mono_cutoff_hz, 20.0..=300.0)
                    .suffix(" Hz")
                    .show_value(true);
                if ui.add(cutoff_slider).changed() {
                    self.puck_pos.0 = Self::cutoff_to_normalized(self.mono_cutoff_hz);
                    self.update_midside_simulation();
                }

                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new("MID PUNCH:")
                        .size(12.0)
                        .color(accent_green)
                        .strong(),
                );
                let punch_slider = egui::Slider::new(&mut self.mid_punch_boost_db, 0.0..=12.0)
                    .suffix(" dB")
                    .show_value(true);
                if ui.add(punch_slider).changed() {
                    self.puck_pos.1 = Self::punch_to_normalized(self.mid_punch_boost_db);
                    self.update_midside_simulation();
                }

                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new("SIDE AIR:")
                        .size(12.0)
                        .color(accent_amber)
                        .strong(),
                );
                let width_slider =
                    egui::Slider::new(&mut self.side_width_ratio, 0.0..=2.0).show_value(true);
                if ui.add(width_slider).changed() {
                    self.update_midside_simulation();
                }
            });
        });
    }
}
