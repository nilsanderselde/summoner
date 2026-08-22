// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Broadcast Mastering Immersive 22.2 Multichannel Audio (NHK 22.2) 3-Layer Spherical Listener Immersion HUD (Step 1635).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const NHK222_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_AZIMUTH_DEG: f32 = -180.0;
pub const MAX_AZIMUTH_DEG: f32 = 180.0;
pub const MIN_ELEVATION_DEG: f32 = -90.0;
pub const MAX_ELEVATION_DEG: f32 = 90.0;
pub const MIN_DISTANCE_M: f32 = 0.5;
pub const MAX_DISTANCE_M: f32 = 20.0;

/// NHK 22.2 3-layer broadcast mastering immersion profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nhk222ProfileType {
    OrchestralSymphony222, // Full concert hall top dome ceiling reflection immersion
    StadiumSportsLive222,  // Upper dome crowd ambiance + ear-level field action
    TheatricalFilmEpic,    // 3-layer dynamic trajectory panning for high-energy cinema
    DomePlanetarium,       // Spherical ceiling-focused astronomical spatial immersion
    Binaural222Headphones, // 22.2 Multichannel virtualized into binaural HRTF headphones
}

impl Nhk222ProfileType {
    pub fn profile_name(&self) -> &'static str {
        match self {
            Self::OrchestralSymphony222 => "ORCHESTRAL SYMPHONY 22.2 (NHK)",
            Self::StadiumSportsLive222 => "STADIUM SPORTS LIVE 22.2",
            Self::TheatricalFilmEpic => "THEATRICAL FILM EPIC 3-LAYER",
            Self::DomePlanetarium => "DOME PLANETARIUM IMMERSION",
            Self::Binaural222Headphones => "BINAURAL 22.2 VIRTUALIZATION",
        }
    }

    pub fn nominal_azimuth_deg(&self) -> f32 {
        match self {
            Self::OrchestralSymphony222 => 0.0,
            Self::StadiumSportsLive222 => -45.0,
            Self::TheatricalFilmEpic => 60.0,
            Self::DomePlanetarium => 0.0,
            Self::Binaural222Headphones => -30.0,
        }
    }

    pub fn nominal_elevation_deg(&self) -> f32 {
        match self {
            Self::OrchestralSymphony222 => 35.0,
            Self::StadiumSportsLive222 => 45.0,
            Self::TheatricalFilmEpic => 20.0,
            Self::DomePlanetarium => 75.0,
            Self::Binaural222Headphones => 15.0,
        }
    }

    pub fn nominal_distance_m(&self) -> f32 {
        match self {
            Self::OrchestralSymphony222 => 8.0,
            Self::StadiumSportsLive222 => 14.0,
            Self::TheatricalFilmEpic => 4.5,
            Self::DomePlanetarium => 10.0,
            Self::Binaural222Headphones => 1.5,
        }
    }
}

/// Broadcast mastering immersive 22.2 Multichannel audio (NHK 22.2) 3-layer HUD.
#[derive(Debug, Clone)]
pub struct Nhk222ImmersionView {
    pub profile: Nhk222ProfileType,
    pub azimuth_deg: f32,   // [-180.0, +180.0]
    pub elevation_deg: f32, // [-90.0, +90.0]
    pub distance_m: f32,    // [0.5, 20.0]
    pub top_layer_energy: f32,
    pub middle_layer_energy: f32,
    pub bottom_layer_energy: f32,
    pub lfe_energy: f32,
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    // 24 Channels:
    // Top (9): TpFL, TpFC, TpFR, TpSiL, TpSiR, TpBL, TpBC, TpBR, TpC
    // Mid (10): FL, FLC, FC, FRC, FR, SiL, SiR, BL, BC, BR
    // Bottom (3): BtFL, BtFC, BtFR
    // LFE (2): LFE1, LFE2
    pub speaker_energies: [f32; 24],
    pub color_palette: ContrastColorPalette,
}

impl Default for Nhk222ImmersionView {
    fn default() -> Self {
        Self::new()
    }
}

impl Nhk222ImmersionView {
    pub fn new() -> Self {
        let mut view = Self {
            profile: Nhk222ProfileType::OrchestralSymphony222,
            azimuth_deg: 0.0,
            elevation_deg: 35.0,
            distance_m: 8.0,
            top_layer_energy: 0.85,
            middle_layer_energy: 0.90,
            bottom_layer_energy: 0.40,
            lfe_energy: 0.60,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            speaker_energies: [0.0; 24],
            color_palette: ContrastColorPalette::default(),
        };
        view.puck_pos = (
            Self::azimuth_to_normalized(view.azimuth_deg),
            Self::elevation_to_normalized(view.elevation_deg),
        );
        view.update_nhk222_simulation();
        view
    }

    pub fn azimuth_to_normalized(az: f32) -> f32 {
        let val = az.clamp(MIN_AZIMUTH_DEG, MAX_AZIMUTH_DEG);
        ((val - MIN_AZIMUTH_DEG) / (MAX_AZIMUTH_DEG - MIN_AZIMUTH_DEG)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_azimuth(norm: f32) -> f32 {
        MIN_AZIMUTH_DEG + norm.clamp(0.0, 1.0) * (MAX_AZIMUTH_DEG - MIN_AZIMUTH_DEG)
    }

    pub fn elevation_to_normalized(el: f32) -> f32 {
        let val = el.clamp(MIN_ELEVATION_DEG, MAX_ELEVATION_DEG);
        ((val - MIN_ELEVATION_DEG) / (MAX_ELEVATION_DEG - MIN_ELEVATION_DEG)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_elevation(norm: f32) -> f32 {
        MIN_ELEVATION_DEG + norm.clamp(0.0, 1.0) * (MAX_ELEVATION_DEG - MIN_ELEVATION_DEG)
    }

    pub fn set_profile(&mut self, prof: Nhk222ProfileType) {
        self.profile = prof;
        self.azimuth_deg = prof.nominal_azimuth_deg();
        self.elevation_deg = prof.nominal_elevation_deg();
        self.distance_m = prof.nominal_distance_m();
        self.puck_pos = (
            Self::azimuth_to_normalized(self.azimuth_deg),
            Self::elevation_to_normalized(self.elevation_deg),
        );
        self.update_nhk222_simulation();
    }

    pub fn update_nhk222_simulation(&mut self) {
        let az = self.azimuth_deg;
        let el = self.elevation_deg;
        let dist = self.distance_m.clamp(0.5, 20.0);
        let dist_att = (1.0 / (1.0 + (dist - 1.0) * 0.08)).clamp(0.2, 1.0);

        let el_rad = el.to_radians();

        let top_weight = (el_rad.sin().max(0.0) * 1.2).clamp(0.0, 1.0);
        let bottom_weight = ((-el_rad.sin()).max(0.0) * 1.2).clamp(0.0, 1.0);
        let mid_weight = (el_rad.cos() - top_weight * 0.3 - bottom_weight * 0.3).clamp(0.1, 1.0);

        self.top_layer_energy = top_weight * dist_att;
        self.middle_layer_energy = mid_weight * dist_att;
        self.bottom_layer_energy = bottom_weight * dist_att;
        self.lfe_energy = (0.5 + bottom_weight * 0.3).clamp(0.2, 1.0) * dist_att;

        // VBAP-like 3-layer spherical distribution across 24 speaker positions
        // Top 9 ch: 0..9
        for i in 0..9 {
            let spk_az = -180.0 + (i as f32 * 45.0);
            let diff = (az - spk_az).to_radians().cos().max(0.0);
            self.speaker_energies[i] = (diff * top_weight * dist_att).clamp(0.02, 1.0);
        }
        // Middle 10 ch: 9..19
        for i in 0..10 {
            let spk_az = -180.0 + (i as f32 * 36.0);
            let diff = (az - spk_az).to_radians().cos().max(0.0);
            self.speaker_energies[9 + i] = (diff * mid_weight * dist_att).clamp(0.02, 1.0);
        }
        // Bottom 3 ch: 19..22
        for i in 0..3 {
            let spk_az = -45.0 + (i as f32 * 45.0);
            let diff = (az - spk_az).to_radians().cos().max(0.0);
            self.speaker_energies[19 + i] = (diff * bottom_weight * dist_att).clamp(0.02, 1.0);
        }
        // LFE 2 ch: 22..24
        self.speaker_energies[22] = self.lfe_energy * 0.9;
        self.speaker_energies[23] = self.lfe_energy * 0.9;
    }

    pub fn hit_test_nhk222_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= NHK222_PUCK_HIT_RADIUS
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
            grid[p_row][p_col] = 'N';
        }

        let right_w = width - mid_x - 3;
        for (i, &amp) in self.speaker_energies.iter().enumerate().take(height - 4) {
            let row = 2 + i;
            let bar_len = ((amp.clamp(0.0, 1.0)) * right_w as f32).round() as usize;
            for c in 0..bar_len {
                if mid_x + 2 + c < width - 1 {
                    grid[row][mid_x + 2 + c] = '#';
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
                    egui::RichText::new("NHK 22.2 MULTICHANNEL 3-LAYER SPHERICAL IMMERSION HUD")
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
                    (
                        Nhk222ProfileType::OrchestralSymphony222,
                        "Orchestral (22.2)",
                    ),
                    (Nhk222ProfileType::StadiumSportsLive222, "Stadium Sports"),
                    (Nhk222ProfileType::TheatricalFilmEpic, "Theatrical Epic"),
                    (Nhk222ProfileType::DomePlanetarium, "Planetarium"),
                    (Nhk222ProfileType::Binaural222Headphones, "Binaural (HRTF)"),
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

            // Main interactive 2D Canvas split: Left XY Pad (Azimuth vs Elevation), Right 3-Layer Speaker Bar Graph
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
                    self.azimuth_deg = Self::normalized_to_azimuth(norm_x);
                    self.elevation_deg = Self::normalized_to_elevation(norm_y);
                    self.update_nhk222_simulation();
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
                NHK222_PUCK_HIT_RADIUS,
                Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 140)),
            );
            painter.circle_filled(puck_center, 12.0, accent_cyan);
            painter.circle_filled(puck_center, 4.0, Color32::WHITE);

            painter.text(
                egui::pos2(pad_inner.min.x, pad_inner.min.y - 12.0),
                egui::Align2::LEFT_BOTTOM,
                "AZIMUTH (-180..+180°) / ELEVATION (-90..+90°)",
                egui::FontId::proportional(11.0),
                accent_cyan,
            );

            // Right Section (3-Layer Speaker Energy Meters)
            painter.text(
                egui::pos2(right_rect.min.x + 16.0, right_rect.min.y + 14.0),
                egui::Align2::LEFT_TOP,
                "NHK 22.2 LAYER ENERGY PROFILE",
                egui::FontId::proportional(11.0),
                accent_green,
            );

            let layer_names = [
                "Top Dome Layer (9ch)",
                "Middle Ear Layer (10ch)",
                "Bottom Floor Layer (3ch)",
                "Subwoofer LFE (2ch)",
            ];
            let layer_values = [
                self.top_layer_energy,
                self.middle_layer_energy,
                self.bottom_layer_energy,
                self.lfe_energy,
            ];

            let bar_y_start = right_rect.min.y + 36.0;
            let bar_h = 18.0;
            let bar_spacing = 26.0;
            let bar_max_w = right_rect.width() - 190.0;

            for (i, &energy) in layer_values.iter().enumerate() {
                let y = bar_y_start + i as f32 * bar_spacing;
                let label = layer_names[i];

                painter.text(
                    egui::pos2(right_rect.min.x + 16.0, y + 9.0),
                    egui::Align2::LEFT_CENTER,
                    label,
                    egui::FontId::proportional(10.0),
                    text_dim,
                );

                let bar_x = right_rect.min.x + 155.0;
                let bar_rect_bg =
                    egui::Rect::from_min_size(egui::pos2(bar_x, y), egui::vec2(bar_max_w, bar_h));
                painter.rect_filled(bar_rect_bg, 3.0, Color32::from_rgb(12, 16, 24));
                painter.rect_stroke(bar_rect_bg, 3.0, Stroke::new(1.0_f32, border_color));

                let fill_w = energy.clamp(0.0, 1.0) * bar_max_w;
                let fill_color = if i == 0 {
                    accent_cyan // Top dome
                } else if i == 1 {
                    accent_green // Mid ear
                } else if i == 2 {
                    accent_amber // Bottom floor
                } else {
                    Color32::from_rgb(255, 107, 43) // LFE
                };

                let bar_fill =
                    egui::Rect::from_min_size(egui::pos2(bar_x, y), egui::vec2(fill_w, bar_h));
                painter.rect_filled(bar_fill, 3.0, fill_color);

                let val_str = format!("{:.0}%", energy * 100.0);
                painter.text(
                    egui::pos2(bar_x + bar_max_w + 8.0, y + 9.0),
                    egui::Align2::LEFT_CENTER,
                    val_str,
                    egui::FontId::proportional(10.0),
                    text_white,
                );
            }

            // 24-channel micro-grid visualization
            let grid_y = bar_y_start + 4.0 * bar_spacing + 4.0;
            let spk_w = (bar_max_w / 24.0).max(2.0);
            let grid_x_start = right_rect.min.x + 155.0;
            for (idx, &spk_gain) in self.speaker_energies.iter().enumerate() {
                let gx = grid_x_start + idx as f32 * spk_w;
                let gh = (spk_gain * 14.0).clamp(1.0, 14.0);
                let gy = grid_y + (14.0 - gh);
                let r = egui::Rect::from_min_size(egui::pos2(gx, gy), egui::vec2(spk_w - 1.0, gh));
                painter.rect_filled(r, 1.0, accent_cyan);
            }

            ui.add_space(8.0);

            // Bottom controls: Sliders
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new("AZIMUTH:")
                        .size(12.0)
                        .color(accent_cyan)
                        .strong(),
                );
                let az_slider = egui::Slider::new(&mut self.azimuth_deg, -180.0..=180.0)
                    .suffix("°")
                    .show_value(true);
                if ui.add(az_slider).changed() {
                    self.puck_pos.0 = Self::azimuth_to_normalized(self.azimuth_deg);
                    self.update_nhk222_simulation();
                }

                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new("ELEVATION:")
                        .size(12.0)
                        .color(accent_green)
                        .strong(),
                );
                let el_slider = egui::Slider::new(&mut self.elevation_deg, -90.0..=90.0)
                    .suffix("°")
                    .show_value(true);
                if ui.add(el_slider).changed() {
                    self.puck_pos.1 = Self::elevation_to_normalized(self.elevation_deg);
                    self.update_nhk222_simulation();
                }

                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new("DISTANCE:")
                        .size(12.0)
                        .color(accent_amber)
                        .strong(),
                );
                let dist_slider = egui::Slider::new(&mut self.distance_m, 0.5..=20.0)
                    .suffix(" m")
                    .show_value(true);
                if ui.add(dist_slider).changed() {
                    self.update_nhk222_simulation();
                }
            });
        });
    }
}
