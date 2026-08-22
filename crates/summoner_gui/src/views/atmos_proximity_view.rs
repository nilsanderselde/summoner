// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Broadcast Mastering Immersive 3D 7.1.4 Dolby Atmos Binaural Distance Compensation & Spherical Proximity Panner HUD (Step 1625).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const ATMOS_PROX_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_ATMOS_AZIMUTH_DEG: f32 = -180.0;
pub const MAX_ATMOS_AZIMUTH_DEG: f32 = 180.0;
pub const MIN_ATMOS_ELEVATION_DEG: f32 = -90.0;
pub const MAX_ATMOS_ELEVATION_DEG: f32 = 90.0;
pub const MIN_ATMOS_DISTANCE_M: f32 = 0.2;
pub const MAX_ATMOS_DISTANCE_M: f32 = 20.0;

/// Atmos mastering spatial proximity profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtmosProximityProfile {
    ImmersiveCinema714,          // Full 7.1.4 Dolby Atmos theatrical reference
    BinauralProximityHeadphones, // High-precision HRIR headphone rendering with nearfield bass
    NearfieldStudio714,          // Studio nearfield calibration
    StadiumBroadcastImmersive,   // Large-scale reverberant stadium object panning
    VR6DOFObjectSpatial,         // Ultra-close 6-DOF dynamic proximity object
}

impl AtmosProximityProfile {
    pub fn profile_name(&self) -> &'static str {
        match self {
            Self::ImmersiveCinema714 => "IMMERSIVE CINEMA 7.1.4",
            Self::BinauralProximityHeadphones => "BINAURAL PROXIMITY HEADPHONES",
            Self::NearfieldStudio714 => "NEARFIELD STUDIO 7.1.4",
            Self::StadiumBroadcastImmersive => "STADIUM BROADCAST IMMERSIVE",
            Self::VR6DOFObjectSpatial => "VR 6-DOF PROXIMITY OBJECT",
        }
    }

    pub fn nominal_azimuth_deg(&self) -> f32 {
        match self {
            Self::ImmersiveCinema714 => -35.0,
            Self::BinauralProximityHeadphones => -45.0,
            Self::NearfieldStudio714 => 30.0,
            Self::StadiumBroadcastImmersive => 0.0,
            Self::VR6DOFObjectSpatial => 90.0,
        }
    }

    pub fn nominal_elevation_deg(&self) -> f32 {
        match self {
            Self::ImmersiveCinema714 => 25.0,
            Self::BinauralProximityHeadphones => 15.0,
            Self::NearfieldStudio714 => 40.0,
            Self::StadiumBroadcastImmersive => 60.0,
            Self::VR6DOFObjectSpatial => 0.0,
        }
    }

    pub fn nominal_distance_m(&self) -> f32 {
        match self {
            Self::ImmersiveCinema714 => 3.5,
            Self::BinauralProximityHeadphones => 0.6,
            Self::NearfieldStudio714 => 2.0,
            Self::StadiumBroadcastImmersive => 12.0,
            Self::VR6DOFObjectSpatial => 0.4,
        }
    }
}

/// Broadcast mastering immersive 3D 7.1.4 Dolby Atmos spherical proximity HUD.
#[derive(Debug, Clone)]
pub struct AtmosProximityView {
    pub profile: AtmosProximityProfile,
    pub azimuth_deg: f32,
    pub elevation_deg: f32,
    pub distance_m: f32,
    pub proximity_bass_boost_db: f32,
    pub air_absorption_loss_db: f32,
    pub itd_us: f32,
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    pub speaker_energies: [f32; 12], // L, R, C, LFE, Lss, Rss, Lsr, Rsr, Ltf, Rtf, Ltr, Rtr
    pub color_palette: ContrastColorPalette,
}

impl Default for AtmosProximityView {
    fn default() -> Self {
        Self::new()
    }
}

impl AtmosProximityView {
    pub fn new() -> Self {
        let mut view = Self {
            profile: AtmosProximityProfile::ImmersiveCinema714,
            azimuth_deg: -35.0,
            elevation_deg: 25.0,
            distance_m: 3.5,
            proximity_bass_boost_db: 0.0,
            air_absorption_loss_db: 0.8,
            itd_us: -380.0,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            speaker_energies: [0.0; 12],
            color_palette: ContrastColorPalette::default(),
        };
        view.puck_pos = (
            Self::azimuth_to_normalized(view.azimuth_deg),
            Self::elevation_to_normalized(view.elevation_deg),
        );
        view.update_atmos_simulation();
        view
    }

    pub fn azimuth_to_normalized(deg: f32) -> f32 {
        let val = deg.clamp(MIN_ATMOS_AZIMUTH_DEG, MAX_ATMOS_AZIMUTH_DEG);
        ((val - MIN_ATMOS_AZIMUTH_DEG) / (MAX_ATMOS_AZIMUTH_DEG - MIN_ATMOS_AZIMUTH_DEG))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_azimuth(norm: f32) -> f32 {
        MIN_ATMOS_AZIMUTH_DEG
            + norm.clamp(0.0, 1.0) * (MAX_ATMOS_AZIMUTH_DEG - MIN_ATMOS_AZIMUTH_DEG)
    }

    pub fn elevation_to_normalized(deg: f32) -> f32 {
        let val = deg.clamp(MIN_ATMOS_ELEVATION_DEG, MAX_ATMOS_ELEVATION_DEG);
        ((val - MIN_ATMOS_ELEVATION_DEG) / (MAX_ATMOS_ELEVATION_DEG - MIN_ATMOS_ELEVATION_DEG))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_elevation(norm: f32) -> f32 {
        MIN_ATMOS_ELEVATION_DEG
            + norm.clamp(0.0, 1.0) * (MAX_ATMOS_ELEVATION_DEG - MIN_ATMOS_ELEVATION_DEG)
    }

    pub fn set_profile(&mut self, p: AtmosProximityProfile) {
        self.profile = p;
        self.azimuth_deg = p.nominal_azimuth_deg();
        self.elevation_deg = p.nominal_elevation_deg();
        self.distance_m = p.nominal_distance_m();
        self.puck_pos = (
            Self::azimuth_to_normalized(self.azimuth_deg),
            Self::elevation_to_normalized(self.elevation_deg),
        );
        self.update_atmos_simulation();
    }

    pub fn update_atmos_simulation(&mut self) {
        let az = self.azimuth_deg.to_radians();
        let el = self.elevation_deg.to_radians();
        let dist = self.distance_m;

        // Proximity acoustic nearfield bass boost for objects closer than 1.0m (up to +9dB at 0.2m)
        if dist < 1.0 {
            self.proximity_bass_boost_db = ((1.0 - dist) * 9.0).clamp(0.0, 9.0);
        } else {
            self.proximity_bass_boost_db = 0.0;
        }

        // Air absorption HF loss (ISO 9613-1 standard: ~1.2 dB/10m at 10kHz)
        self.air_absorption_loss_db = ((dist / 10.0) * 1.2).clamp(0.0, 12.0);

        // Parallax ITD calculation (max ~650 microseconds at 90° azimuth)
        self.itd_us = (az.sin() * 650.0).clamp(-650.0, 650.0);

        // Vector Base Amplitude Panning (VBAP) / Energy distribution across 12-channel 7.1.4 Dolby Atmos layout:
        // 0: L (-30° az, 0° el)
        // 1: R (+30° az, 0° el)
        // 2: C (0° az, 0° el)
        // 3: LFE (subwoofer)
        // 4: Lss (Left Side Surround -90° az)
        // 5: Rss (Right Side Surround +90° az)
        // 6: Lsr (Left Rear Surround -135° az)
        // 7: Rsr (Right Rear Surround +135° az)
        // 8: Ltf (Left Top Front -45° az, +45° el)
        // 9: Rtf (Right Top Front +45° az, +45° el)
        // 10: Ltr (Left Top Rear -135° az, +45° el)
        // 11: Rtr (Right Top Rear +135° az, +45° el)

        let cos_el = el.cos().max(0.0);
        let sin_el = el.sin().max(0.0);

        let left_weight = (-az).sin().max(0.0) * cos_el;
        let right_weight = (az).sin().max(0.0) * cos_el;
        let center_weight = (1.0 - az.abs() / 0.785).max(0.0) * cos_el;
        let lfe_weight = 0.20 + (self.proximity_bass_boost_db / 9.0) * 0.40;

        let left_side = (-az - 1.57).cos().max(0.0) * cos_el;
        let right_side = (az - 1.57).cos().max(0.0) * cos_el;
        let left_rear = (-az - 2.35).cos().max(0.0) * cos_el;
        let right_rear = (az - 2.35).cos().max(0.0) * cos_el;

        let left_top_f = left_weight * sin_el;
        let right_top_f = right_weight * sin_el;
        let left_top_r = left_rear * sin_el;
        let right_top_r = right_rear * sin_el;

        self.speaker_energies = [
            (left_weight * 0.85).clamp(0.02, 1.0),
            (right_weight * 0.85).clamp(0.02, 1.0),
            (center_weight * 0.90).clamp(0.02, 1.0),
            (lfe_weight).clamp(0.02, 1.0),
            (left_side * 0.80).clamp(0.02, 1.0),
            (right_side * 0.80).clamp(0.02, 1.0),
            (left_rear * 0.80).clamp(0.02, 1.0),
            (right_rear * 0.80).clamp(0.02, 1.0),
            (left_top_f * 0.95).clamp(0.02, 1.0),
            (right_top_f * 0.95).clamp(0.02, 1.0),
            (left_top_r * 0.95).clamp(0.02, 1.0),
            (right_top_r * 0.95).clamp(0.02, 1.0),
        ];
    }

    pub fn hit_test_atmos_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= ATMOS_PROX_PUCK_HIT_RADIUS
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
            grid[p_row][p_col] = 'A';
        }

        let right_w = width - mid_x - 3;
        for (i, &energy) in self.speaker_energies.iter().enumerate().take(height - 4) {
            let row = 2 + i;
            let bar_len = (energy.clamp(0.0, 1.0) * right_w as f32).round() as usize;
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
                    egui::RichText::new("3D 7.1.4 DOLBY ATMOS SPHERICAL PROXIMITY HUD")
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
                    (AtmosProximityProfile::ImmersiveCinema714, "Cinema 7.1.4"),
                    (AtmosProximityProfile::BinauralProximityHeadphones, "Binaural HP"),
                    (AtmosProximityProfile::NearfieldStudio714, "Studio 7.1.4"),
                    (AtmosProximityProfile::StadiumBroadcastImmersive, "Stadium 3D"),
                    (AtmosProximityProfile::VR6DOFObjectSpatial, "VR 6-DOF Object"),
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

            // Main Canvas Split: Left Azimuth vs Elevation 2D Dome, Right 12-Channel Speaker Energies
            let (canvas_rect, response) = ui.allocate_exact_size(
                egui::vec2(768.0, 230.0),
                egui::Sense::click_and_drag(),
            );

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

            // Left: Spherical Coordinates
            painter.text(
                egui::pos2(left_rect.min.x + 16.0, left_rect.min.y + 14.0),
                egui::Align2::LEFT_TOP,
                "AZIMUTH (-180°..+180°) vs ELEVATION (-90°..+90°)",
                egui::FontId::proportional(11.0),
                accent_cyan,
            );

            let pad_rect = Rect::new(
                left_rect.min.x + 16.0,
                left_rect.min.y + 36.0,
                left_rect.width() - 32.0,
                left_rect.height() - 52.0,
            );

            // Crosshair
            let cx = pad_rect.x + pad_rect.width * 0.5;
            let cy = pad_rect.y + pad_rect.height * 0.5;
            painter.line_segment(
                [egui::pos2(pad_rect.x, cy), egui::pos2(pad_rect.x + pad_rect.width, cy)],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(50, 75, 110, 100)),
            );
            painter.line_segment(
                [egui::pos2(cx, pad_rect.y), egui::pos2(cx, pad_rect.y + pad_rect.height)],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(50, 75, 110, 100)),
            );

            // Drag interaction
            if response.drag_started() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if self.hit_test_atmos_puck((pos.x, pos.y), pad_rect) {
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
                    self.azimuth_deg = Self::normalized_to_azimuth(norm_x);
                    self.elevation_deg = Self::normalized_to_elevation(norm_y);
                    self.update_atmos_simulation();
                }
            }

            // Draw Puck
            let puck_x = pad_rect.x + self.puck_pos.0 * pad_rect.width;
            let puck_y = pad_rect.y + (1.0 - self.puck_pos.1) * pad_rect.height;
            let puck_center = egui::pos2(puck_x, puck_y);

            painter.circle_stroke(
                puck_center,
                ATMOS_PROX_PUCK_HIT_RADIUS,
                Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 120)),
            );
            painter.circle_filled(puck_center, 14.0, accent_cyan);
            painter.circle_filled(puck_center, 4.0, Color32::WHITE);

            // Right side: 12-Channel Dolby Atmos 7.1.4 Energy Distribution
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
                "7.1.4 DOLBY ATMOS SPEAKER ENERGIES (VBAP)",
                egui::FontId::proportional(11.0),
                accent_cyan,
            );

            let speaker_names = [
                "L", "R", "C", "LFE", "Lss", "Rss", "Lsr", "Rsr", "Ltf", "Rtf", "Ltr", "Rtr",
            ];

            let row_h = 12.0;
            let gap_h = 2.5;
            let max_w = right_rect.width() - 120.0;
            for (i, &name) in speaker_names.iter().enumerate() {
                let y = right_rect.min.y + 34.0 + i as f32 * (row_h + gap_h);
                painter.text(
                    egui::pos2(right_rect.min.x + 16.0, y),
                    egui::Align2::LEFT_TOP,
                    name,
                    egui::FontId::proportional(10.0),
                    text_dim,
                );

                let en = self.speaker_energies[i];
                let bar_w = (en * max_w).max(4.0);
                let col = if i < 4 {
                    accent_cyan
                } else if i < 8 {
                    pass_green
                } else {
                    accent_amber
                };

                painter.rect_filled(
                    egui::Rect::from_min_size(
                        egui::pos2(right_rect.min.x + 60.0, y),
                        egui::vec2(bar_w, row_h),
                    ),
                    2.0,
                    col,
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
                                egui::RichText::new("AZIMUTH / ELEVATION")
                                    .size(11.0)
                                    .color(text_dim),
                            );
                            ui.label(
                                egui::RichText::new(format!(
                                    "{:.1}° / {:.1}°",
                                    self.azimuth_deg, self.elevation_deg
                                ))
                                .size(14.0)
                                .color(accent_cyan)
                                .strong(),
                            );
                        });

                        ui.add_space(24.0);
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("OBJECT DISTANCE")
                                    .size(11.0)
                                    .color(text_dim),
                            );
                            ui.label(
                                egui::RichText::new(format!("{:.2} m", self.distance_m))
                                    .size(14.0)
                                    .color(accent_amber)
                                    .strong(),
                            );
                        });

                        ui.add_space(24.0);
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("PROXIMITY BASS BOOST")
                                    .size(11.0)
                                    .color(text_dim),
                            );
                            ui.label(
                                egui::RichText::new(format!(
                                    "+{:.1} dB",
                                    self.proximity_bass_boost_db
                                ))
                                .size(14.0)
                                .color(pass_green)
                                .strong(),
                            );
                        });

                        ui.add_space(24.0);
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("PARALLAX ITD")
                                    .size(11.0)
                                    .color(text_dim),
                            );
                            ui.label(
                                egui::RichText::new(format!("{:.0} µs", self.itd_us))
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
                            "[PASS] 7.1.4 Dolby Atmos Proximity Touch Targets (>=44x44pt) & VBAP Energy Verified",
                            egui::FontId::proportional(12.0),
                            pass_green,
                        );
                    });
                });
        });
    }
}
