// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Physical Modeling Japanese Shakuhachi Bamboo Flute / Blowing Edge Chiff & Pitch-Bend Microtone HUD (Step 1631).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const SHAKUHACHI_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_JET_VELOCITY_MPS: f32 = 4.0;
pub const MAX_JET_VELOCITY_MPS: f32 = 42.0;
pub const MIN_UTAGUCHI_ANGLE_DEG: f32 = 10.0;
pub const MAX_UTAGUCHI_ANGLE_DEG: f32 = 60.0;
pub const MIN_MERI_KARI_CENTS: f32 = -200.0;
pub const MAX_MERI_KARI_CENTS: f32 = 100.0;

/// Classical Japanese Shakuhachi flute lengths and pitch reference types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShakuhachiLengthType {
    IchishakuHassun,  // 1.8 Shaku (~54.5cm), standard D4 classical Honkyoku reference
    NishakuYonsun,    // 2.4 Shaku (~72.7cm), deep meditative A3 Jinashi Zen flute
    IchishakuRokusun, // 1.6 Shaku (~48.5cm), bright E4 treble flute for Min'yo folk
    NishakuIssun,     // 2.1 Shaku (~63.6cm), warm low B3 flute for ensemble Sankyoku
    SanShakuKyotaku,  // 3.0 Shaku (~90.9cm), giant sub-bass D3 Kyotaku temple flute
}

impl ShakuhachiLengthType {
    pub fn flute_name(&self) -> &'static str {
        match self {
            Self::IchishakuHassun => "1.8 SHAKU (54.5cm D4 HONKYOKU)",
            Self::NishakuYonsun => "2.4 SHAKU (72.7cm A3 JINASHI ZEN)",
            Self::IchishakuRokusun => "1.6 SHAKU (48.5cm E4 MIN'YO)",
            Self::NishakuIssun => "2.1 SHAKU (63.6cm B3 SANKYOKU)",
            Self::SanShakuKyotaku => "3.0 SHAKU (90.9cm D3 KYOTAKU)",
        }
    }

    pub fn nominal_jet_velocity_mps(&self) -> f32 {
        match self {
            Self::IchishakuHassun => 19.0,
            Self::NishakuYonsun => 14.5,
            Self::IchishakuRokusun => 24.0,
            Self::NishakuIssun => 16.5,
            Self::SanShakuKyotaku => 11.0,
        }
    }

    pub fn nominal_utaguchi_angle_deg(&self) -> f32 {
        match self {
            Self::IchishakuHassun => 38.0,
            Self::NishakuYonsun => 44.0,
            Self::IchishakuRokusun => 34.0,
            Self::NishakuIssun => 40.0,
            Self::SanShakuKyotaku => 48.0,
        }
    }

    pub fn nominal_length_cm(&self) -> f32 {
        match self {
            Self::IchishakuHassun => 54.5,
            Self::NishakuYonsun => 72.7,
            Self::IchishakuRokusun => 48.5,
            Self::NishakuIssun => 63.6,
            Self::SanShakuKyotaku => 90.9,
        }
    }

    pub fn nominal_chiff_noise(&self) -> f32 {
        match self {
            Self::IchishakuHassun => 0.35,
            Self::NishakuYonsun => 0.45,
            Self::IchishakuRokusun => 0.28,
            Self::NishakuIssun => 0.38,
            Self::SanShakuKyotaku => 0.55,
        }
    }

    pub fn nominal_bore_q(&self) -> f32 {
        match self {
            Self::IchishakuHassun => 52.0,
            Self::NishakuYonsun => 44.0,
            Self::IchishakuRokusun => 58.0,
            Self::NishakuIssun => 48.0,
            Self::SanShakuKyotaku => 38.0,
        }
    }
}

/// Traditional Shakuhachi pentatonic fingering modes (5 tone scale).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShakuhachiFingeringNote {
    Ro,  // All 5 holes closed (Root D)
    Tsu, // Hole 1 open (F)
    Re,  // Holes 1 & 2 open (G)
    Chi, // Holes 1, 2 & 3 open (A)
    Ri,  // Holes 1, 2, 3 & 4 open (C)
}

impl ShakuhachiFingeringNote {
    pub fn note_name(&self) -> &'static str {
        match self {
            Self::Ro => "RO (呂 - Root D)",
            Self::Tsu => "TSU (ツ - Minor 3rd F)",
            Self::Re => "RE (レ - 4th G)",
            Self::Chi => "CHI (チ - 5th A)",
            Self::Ri => "RI (リ - Minor 7th C)",
        }
    }

    pub fn open_hole_ratio(&self) -> f32 {
        match self {
            Self::Ro => 0.0,
            Self::Tsu => 0.25,
            Self::Re => 0.50,
            Self::Chi => 0.75,
            Self::Ri => 1.00,
        }
    }
}

/// Physical modeling Japanese Shakuhachi flute / blowing edge chiff HUD.
#[derive(Debug, Clone)]
pub struct ShakuhachiView {
    pub length_type: ShakuhachiLengthType,
    pub fingering_note: ShakuhachiFingeringNote,
    pub jet_velocity_mps: f32,
    pub utaguchi_angle_deg: f32,
    pub meri_kari_cents: f32,
    pub chiff_noise_level: f32,
    pub bamboo_node_dispersion: f32,
    pub acoustic_bore_q: f32,
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    pub modal_amplitudes: [f32; 8],
    pub color_palette: ContrastColorPalette,
}

impl Default for ShakuhachiView {
    fn default() -> Self {
        Self::new()
    }
}

impl ShakuhachiView {
    pub fn new() -> Self {
        let mut view = Self {
            length_type: ShakuhachiLengthType::IchishakuHassun,
            fingering_note: ShakuhachiFingeringNote::Ro,
            jet_velocity_mps: 19.0,
            utaguchi_angle_deg: 38.0,
            meri_kari_cents: 0.0,
            chiff_noise_level: 0.35,
            bamboo_node_dispersion: 0.15,
            acoustic_bore_q: 52.0,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            modal_amplitudes: [1.0, 0.60, 0.35, 0.20, 0.35, 0.40, 0.70, 0.50],
            color_palette: ContrastColorPalette::default(),
        };
        view.puck_pos = (
            Self::angle_to_normalized(view.utaguchi_angle_deg),
            Self::velocity_to_normalized(view.jet_velocity_mps),
        );
        view.update_shakuhachi_simulation();
        view
    }

    pub fn velocity_to_normalized(v: f32) -> f32 {
        let val = v.clamp(MIN_JET_VELOCITY_MPS, MAX_JET_VELOCITY_MPS);
        ((val - MIN_JET_VELOCITY_MPS) / (MAX_JET_VELOCITY_MPS - MIN_JET_VELOCITY_MPS))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_velocity(norm: f32) -> f32 {
        MIN_JET_VELOCITY_MPS + norm.clamp(0.0, 1.0) * (MAX_JET_VELOCITY_MPS - MIN_JET_VELOCITY_MPS)
    }

    pub fn angle_to_normalized(a: f32) -> f32 {
        let val = a.clamp(MIN_UTAGUCHI_ANGLE_DEG, MAX_UTAGUCHI_ANGLE_DEG);
        ((val - MIN_UTAGUCHI_ANGLE_DEG) / (MAX_UTAGUCHI_ANGLE_DEG - MIN_UTAGUCHI_ANGLE_DEG))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_angle(norm: f32) -> f32 {
        MIN_UTAGUCHI_ANGLE_DEG
            + norm.clamp(0.0, 1.0) * (MAX_UTAGUCHI_ANGLE_DEG - MIN_UTAGUCHI_ANGLE_DEG)
    }

    pub fn set_length_type(&mut self, len_type: ShakuhachiLengthType) {
        self.length_type = len_type;
        self.jet_velocity_mps = len_type.nominal_jet_velocity_mps();
        self.utaguchi_angle_deg = len_type.nominal_utaguchi_angle_deg();
        self.chiff_noise_level = len_type.nominal_chiff_noise();
        self.acoustic_bore_q = len_type.nominal_bore_q();
        self.puck_pos = (
            Self::angle_to_normalized(self.utaguchi_angle_deg),
            Self::velocity_to_normalized(self.jet_velocity_mps),
        );
        self.update_shakuhachi_simulation();
    }

    pub fn set_fingering(&mut self, note: ShakuhachiFingeringNote) {
        self.fingering_note = note;
        self.update_shakuhachi_simulation();
    }

    pub fn update_shakuhachi_simulation(&mut self) {
        let v = self.jet_velocity_mps;
        let angle = self.utaguchi_angle_deg;
        let meri_kari = self.meri_kari_cents;
        let noise = self.chiff_noise_level;
        let disp = self.bamboo_node_dispersion;
        let q = self.acoustic_bore_q;
        let hole_ratio = self.fingering_note.open_hole_ratio();

        // Utaguchi edge vortex shedding physics
        let optimal_angle = 38.0;
        let angle_alignment = (1.0 - ((angle - optimal_angle).abs() / 32.0)).clamp(0.1, 1.0);
        let jet_pressure = (v / 22.0).clamp(0.2, 1.8);
        let pitch_bend_factor = 1.0 + (meri_kari / 1200.0) * 0.15;

        let fundamental_f0 = (angle_alignment
            * jet_pressure
            * (q / 50.0)
            * (1.0 - disp * 0.3)
            * (1.0 + hole_ratio * 0.25)
            * pitch_bend_factor)
            .clamp(0.1, 1.25);
        let octave_kan_mode =
            ((v / 26.0) * angle_alignment * 0.88 * pitch_bend_factor).clamp(0.0, 1.20);
        let twelfth_dai_kan = (((v - 18.0).max(0.0) / 20.0) * 0.60).clamp(0.0, 0.95);
        let double_octave = (((v - 25.0).max(0.0) / 22.0) * 0.40).clamp(0.0, 0.75);

        let blowing_edge_chiff =
            (noise * (v / 18.0) * (1.0 + (angle - optimal_angle).abs() * 0.025)).clamp(0.05, 1.0);
        let breath_turbulence = (noise * 0.80 + (1.0 - angle_alignment) * 0.45).clamp(0.05, 1.0);
        let bamboo_node_resonance = ((1.0 + disp * 0.5) * (q / 50.0) * 0.75).clamp(0.2, 1.2);
        let meri_kari_inflection = ((meri_kari.abs() / 200.0) * 0.65 + 0.35).clamp(0.1, 1.0);

        self.modal_amplitudes = [
            fundamental_f0,
            octave_kan_mode,
            twelfth_dai_kan,
            double_octave,
            blowing_edge_chiff,
            breath_turbulence,
            bamboo_node_resonance,
            meri_kari_inflection,
        ];
    }

    pub fn hit_test_shakuhachi_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= SHAKUHACHI_PUCK_HIT_RADIUS
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
            grid[p_row][p_col] = 'S';
        }

        let right_w = width - mid_x - 3;
        for (i, &amp) in self.modal_amplitudes.iter().enumerate().take(height - 4) {
            let row = 2 + i;
            let bar_len = ((amp.clamp(0.0, 1.25) / 1.25) * right_w as f32).round() as usize;
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
                    egui::RichText::new("JAPANESE SHAKUHACHI PHYSICAL MODELING & CHIFF HUD")
                        .size(18.0)
                        .color(text_white)
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(16.0);
                    ui.label(
                        egui::RichText::new(self.length_type.flute_name())
                            .size(13.0)
                            .color(accent_amber)
                            .strong(),
                    );
                });
            });

            ui.add_space(6.0);

            // Flute length selector tabs (min 44pt touch hit targets)
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                let lengths = [
                    (ShakuhachiLengthType::IchishakuHassun, "1.8 Standard (D4)"),
                    (ShakuhachiLengthType::NishakuYonsun, "2.4 Jinashi (A3)"),
                    (ShakuhachiLengthType::IchishakuRokusun, "1.6 Treble (E4)"),
                    (ShakuhachiLengthType::NishakuIssun, "2.1 Sankyoku (B3)"),
                    (ShakuhachiLengthType::SanShakuKyotaku, "3.0 Kyotaku (D3)"),
                ];

                for (len_type, label) in lengths {
                    let is_active = self.length_type == len_type;
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
                        self.set_length_type(len_type);
                    }
                    ui.add_space(4.0);
                }
            });

            ui.add_space(8.0);

            // Main interactive 2D Canvas split: Left XY Pad (Utaguchi Angle vs Jet Velocity), Right Modal Resonances
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

            // Draw Left Section (XY Pad)
            painter.line_segment(
                [
                    egui::pos2(left_rect.max.x, left_rect.min.y + 10.0),
                    egui::pos2(left_rect.max.x, left_rect.max.y - 10.0),
                ],
                Stroke::new(1.0_f32, border_color),
            );

            // Subdivided Grid
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

            // XY Puck Handling
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
                    self.utaguchi_angle_deg = Self::normalized_to_angle(norm_x);
                    self.jet_velocity_mps = Self::normalized_to_velocity(norm_y);
                    self.update_shakuhachi_simulation();
                }
            }

            let puck_screen_x = pad_inner.min.x + self.puck_pos.0 * pad_inner.width();
            let puck_screen_y = pad_inner.min.y + (1.0 - self.puck_pos.1) * pad_inner.height();
            let puck_center = egui::pos2(puck_screen_x, puck_screen_y);

            // Crosshairs
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

            // Outer 44pt touch bounding target
            painter.circle_stroke(
                puck_center,
                SHAKUHACHI_PUCK_HIT_RADIUS,
                Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 140)),
            );
            painter.circle_filled(puck_center, 12.0, accent_cyan);
            painter.circle_filled(puck_center, 4.0, Color32::WHITE);

            // Pad text annotations
            painter.text(
                egui::pos2(pad_inner.min.x, pad_inner.min.y - 12.0),
                egui::Align2::LEFT_BOTTOM,
                "UTAGUCHI BLOWING ANGLE / JET VELOCITY",
                egui::FontId::proportional(11.0),
                accent_cyan,
            );

            // Draw Right Section (Acoustic Modal Resonances & Chiff Spectrum)
            painter.text(
                egui::pos2(right_rect.min.x + 16.0, right_rect.min.y + 14.0),
                egui::Align2::LEFT_TOP,
                "MODAL HARMONICS & CHIFF ENERGY",
                egui::FontId::proportional(11.0),
                accent_green,
            );

            let modal_names = [
                "Ro Fundamental (f0)",
                "Kan Octave (2f0)",
                "Dai-Kan (3f0)",
                "Double Octave (4f0)",
                "Utaguchi Chiff",
                "Breath Turbulence",
                "Node Resonance",
                "Meri-Kari Bend",
            ];

            let bar_y_start = right_rect.min.y + 36.0;
            let bar_h = 16.0;
            let bar_spacing = 22.0;
            let bar_max_w = right_rect.width() - 170.0;

            for (i, &amp) in self.modal_amplitudes.iter().enumerate() {
                let y = bar_y_start + i as f32 * bar_spacing;
                let label = modal_names[i];

                painter.text(
                    egui::pos2(right_rect.min.x + 16.0, y + 8.0),
                    egui::Align2::LEFT_CENTER,
                    label,
                    egui::FontId::proportional(10.0),
                    text_dim,
                );

                let bar_x = right_rect.min.x + 130.0;
                let bar_rect_bg =
                    egui::Rect::from_min_size(egui::pos2(bar_x, y), egui::vec2(bar_max_w, bar_h));
                painter.rect_filled(bar_rect_bg, 3.0, Color32::from_rgb(12, 16, 24));
                painter.rect_stroke(bar_rect_bg, 3.0, Stroke::new(1.0_f32, border_color));

                let fill_w = (amp.clamp(0.0, 1.25) / 1.25) * bar_max_w;
                let fill_color = if i == 4 || i == 5 {
                    Color32::from_rgb(255, 107, 43) // Noise chiff / breath in orange
                } else if i >= 6 {
                    accent_amber // Resonances
                } else {
                    accent_cyan // Pitched modes
                };

                let bar_fill =
                    egui::Rect::from_min_size(egui::pos2(bar_x, y), egui::vec2(fill_w, bar_h));
                painter.rect_filled(bar_fill, 3.0, fill_color);

                let val_str = format!("{:.2}", amp);
                painter.text(
                    egui::pos2(bar_x + bar_max_w + 8.0, y + 8.0),
                    egui::Align2::LEFT_CENTER,
                    val_str,
                    egui::FontId::proportional(10.0),
                    text_white,
                );
            }

            ui.add_space(8.0);

            // Bottom controls: Fingering Tabs (Ro, Tsu, Re, Chi, Ri) and Sliders
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new("FINGERING:")
                        .size(12.0)
                        .color(accent_cyan)
                        .strong(),
                );
                ui.add_space(6.0);

                let fingerings = [
                    (ShakuhachiFingeringNote::Ro, "Ro (呂)"),
                    (ShakuhachiFingeringNote::Tsu, "Tsu (ツ)"),
                    (ShakuhachiFingeringNote::Re, "Re (レ)"),
                    (ShakuhachiFingeringNote::Chi, "Chi (チ)"),
                    (ShakuhachiFingeringNote::Ri, "Ri (リ)"),
                ];

                for (fn_note, label) in fingerings {
                    let is_active = self.fingering_note == fn_note;
                    let btn_bg = if is_active {
                        accent_green
                    } else {
                        Color32::from_rgb(26, 36, 52)
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
                    .min_size(egui::vec2(72.0, 44.0));

                    if ui.add(btn).clicked() {
                        self.set_fingering(fn_note);
                    }
                    ui.add_space(4.0);
                }

                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new("MERI/KARI:")
                        .size(12.0)
                        .color(accent_amber)
                        .strong(),
                );
                let meri_slider = egui::Slider::new(&mut self.meri_kari_cents, -200.0..=100.0)
                    .suffix(" ct")
                    .show_value(true);
                if ui.add(meri_slider).changed() {
                    self.update_shakuhachi_simulation();
                }

                ui.add_space(12.0);
                ui.label(egui::RichText::new("CHIFF:").size(12.0).color(text_dim));
                let chiff_slider =
                    egui::Slider::new(&mut self.chiff_noise_level, 0.0..=1.0).show_value(true);
                if ui.add(chiff_slider).changed() {
                    self.update_shakuhachi_simulation();
                }
            });
        });
    }
}
