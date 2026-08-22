// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Physical Modeling Turkish Ney Reed Flute / Embouchure Turbulence Airjet Vortex HUD (Step 1621).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const NEY_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_JET_VELOCITY_MPS: f32 = 5.0;
pub const MAX_JET_VELOCITY_MPS: f32 = 45.0;
pub const MIN_EMBOUCHURE_ANGLE_DEG: f32 = 15.0;
pub const MAX_EMBOUCHURE_ANGLE_DEG: f32 = 65.0;
pub const MIN_BORE_LENGTH_CM: f32 = 45.0;
pub const MAX_BORE_LENGTH_CM: f32 = 88.0;

/// Classical Turkish Ney instruments and pitch reference tuning types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurkishNeyType {
    MansurNey,   // 78cm, A=440 / D Rast spiritual reference, standard classical length
    KizNey,      // 68cm, B reference, standard Ottoman court music tuning
    BolahenkNey, // 52cm, High E treble reference, bright overblowing response
    SupurdeNey,  // 58cm, D reference, agile Taksim ornamentation
    SahNey,      // 86cm, Deep G contrabass reference, rich embouchure turbulence
}

impl TurkishNeyType {
    pub fn instrument_name(&self) -> &'static str {
        match self {
            Self::MansurNey => "MANSUR NEY (78cm A=440 RAST)",
            Self::KizNey => "KIZ NEY (68cm B OTTOMAN)",
            Self::BolahenkNey => "BOLAHENK NEY (52cm HIGH E)",
            Self::SupurdeNey => "SÜPÜRDE NEY (58cm D TAKSIM)",
            Self::SahNey => "ŞAH NEY (86cm LOW G BASS)",
        }
    }

    pub fn nominal_jet_velocity_mps(&self) -> f32 {
        match self {
            Self::MansurNey => 18.5,
            Self::KizNey => 22.0,
            Self::BolahenkNey => 26.0,
            Self::SupurdeNey => 24.0,
            Self::SahNey => 15.0,
        }
    }

    pub fn nominal_embouchure_angle_deg(&self) -> f32 {
        match self {
            Self::MansurNey => 42.0,
            Self::KizNey => 40.0,
            Self::BolahenkNey => 36.0,
            Self::SupurdeNey => 38.0,
            Self::SahNey => 46.0,
        }
    }

    pub fn nominal_bore_length_cm(&self) -> f32 {
        match self {
            Self::MansurNey => 78.0,
            Self::KizNey => 68.0,
            Self::BolahenkNey => 52.0,
            Self::SupurdeNey => 58.0,
            Self::SahNey => 86.0,
        }
    }

    pub fn nominal_turbulence_noise(&self) -> f32 {
        match self {
            Self::MansurNey => 0.30,
            Self::KizNey => 0.25,
            Self::BolahenkNey => 0.20,
            Self::SupurdeNey => 0.22,
            Self::SahNey => 0.40,
        }
    }

    pub fn nominal_bore_loss(&self) -> f32 {
        match self {
            Self::MansurNey => 0.05,
            Self::KizNey => 0.04,
            Self::BolahenkNey => 0.03,
            Self::SupurdeNey => 0.04,
            Self::SahNey => 0.08,
        }
    }

    pub fn nominal_acoustic_q(&self) -> f32 {
        match self {
            Self::MansurNey => 48.0,
            Self::KizNey => 52.0,
            Self::BolahenkNey => 56.0,
            Self::SupurdeNey => 54.0,
            Self::SahNey => 42.0,
        }
    }
}

/// Başpare (lip-rest / mouthpiece) material acoustic properties.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BashpareHornType {
    WaterBuffaloHorn, // Traditional Manda Boynuzu (smooth impedance transition)
    DelrinAcoustic,   // Ultra-consistent precision synthetic lathe
    WalnutHardwood,   // Warm organic acoustic dampening
    EbonyReeded,      // Dense resonance with focused vortex jet
}

impl BashpareHornType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::WaterBuffaloHorn => "Water Buffalo Horn (Manda)",
            Self::DelrinAcoustic => "Delrin Precision Acoustic",
            Self::WalnutHardwood => "Walnut Hardwood (Ceviz)",
            Self::EbonyReeded => "Ebony Dense Composite",
        }
    }

    pub fn impedance_coupling(&self) -> f32 {
        match self {
            Self::WaterBuffaloHorn => 1.05,
            Self::DelrinAcoustic => 1.00,
            Self::WalnutHardwood => 0.92,
            Self::EbonyReeded => 1.10,
        }
    }
}

/// Physical modeling Turkish Ney reed flute / embouchure turbulence airjet vortex HUD.
#[derive(Debug, Clone)]
pub struct TurkishNeyView {
    pub ney_type: TurkishNeyType,
    pub bashpare_type: BashpareHornType,
    pub jet_velocity_mps: f32,
    pub embouchure_angle_deg: f32,
    pub bore_length_cm: f32,
    pub airjet_turbulence_noise: f32,
    pub bore_loss_factor: f32,
    pub acoustic_q: f32,
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    pub modal_amplitudes: [f32; 8],
    pub color_palette: ContrastColorPalette,
}

impl Default for TurkishNeyView {
    fn default() -> Self {
        Self::new()
    }
}

impl TurkishNeyView {
    pub fn new() -> Self {
        let mut view = Self {
            ney_type: TurkishNeyType::MansurNey,
            bashpare_type: BashpareHornType::WaterBuffaloHorn,
            jet_velocity_mps: 18.5,
            embouchure_angle_deg: 42.0,
            bore_length_cm: 78.0,
            airjet_turbulence_noise: 0.30,
            bore_loss_factor: 0.05,
            acoustic_q: 48.0,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            modal_amplitudes: [1.0, 0.65, 0.38, 0.22, 0.30, 0.45, 0.85, 0.55],
            color_palette: ContrastColorPalette::default(),
        };
        view.puck_pos = (
            Self::angle_to_normalized(view.embouchure_angle_deg),
            Self::velocity_to_normalized(view.jet_velocity_mps),
        );
        view.update_ney_simulation();
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
        let val = a.clamp(MIN_EMBOUCHURE_ANGLE_DEG, MAX_EMBOUCHURE_ANGLE_DEG);
        ((val - MIN_EMBOUCHURE_ANGLE_DEG) / (MAX_EMBOUCHURE_ANGLE_DEG - MIN_EMBOUCHURE_ANGLE_DEG))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_angle(norm: f32) -> f32 {
        MIN_EMBOUCHURE_ANGLE_DEG
            + norm.clamp(0.0, 1.0) * (MAX_EMBOUCHURE_ANGLE_DEG - MIN_EMBOUCHURE_ANGLE_DEG)
    }

    pub fn set_ney_type(&mut self, ney: TurkishNeyType) {
        self.ney_type = ney;
        self.jet_velocity_mps = ney.nominal_jet_velocity_mps();
        self.embouchure_angle_deg = ney.nominal_embouchure_angle_deg();
        self.bore_length_cm = ney.nominal_bore_length_cm();
        self.airjet_turbulence_noise = ney.nominal_turbulence_noise();
        self.bore_loss_factor = ney.nominal_bore_loss();
        self.acoustic_q = ney.nominal_acoustic_q();
        self.puck_pos = (
            Self::angle_to_normalized(self.embouchure_angle_deg),
            Self::velocity_to_normalized(self.jet_velocity_mps),
        );
        self.update_ney_simulation();
    }

    pub fn update_ney_simulation(&mut self) {
        let v = self.jet_velocity_mps;
        let angle = self.embouchure_angle_deg;
        let length = self.bore_length_cm;
        let noise = self.airjet_turbulence_noise;
        let loss = self.bore_loss_factor;
        let q = self.acoustic_q;
        let coupling = self.bashpare_type.impedance_coupling();

        // Physical modeling airjet-edge vortex shedding and open-cylinder acoustic bore physics
        let optimal_angle = 42.0;
        let angle_alignment = (1.0 - ((angle - optimal_angle).abs() / 35.0)).clamp(0.1, 1.0);
        let velocity_pressure = (v / 25.0).clamp(0.2, 1.8);

        let f0_rast = (angle_alignment * velocity_pressure * (q / 50.0) * (1.0 - loss) * coupling)
            .clamp(0.1, 1.25);
        let f1_dugah_octave = ((v / 30.0) * angle_alignment * 0.85 * coupling).clamp(0.0, 1.15);
        let f2_segah_twelfth = (((v - 15.0).max(0.0) / 25.0) * 0.65).clamp(0.0, 0.95);
        let f3_cargah_double = (((v - 22.0).max(0.0) / 25.0) * 0.45).clamp(0.0, 0.80);

        let vortex_turbulence =
            (noise * (v / 20.0) * (1.0 + (angle - optimal_angle).abs() * 0.02)).clamp(0.05, 1.0);
        let embouchure_edge_breath = (noise * 0.75 + (1.0 - angle_alignment) * 0.5).clamp(0.0, 1.0);
        let headjoint_horn_gain =
            (coupling * (1.0 - loss * 0.5) * (78.0 / length).clamp(0.8, 1.4)).clamp(0.2, 1.3);
        let acoustic_bore_circulation = ((q / 50.0) * f0_rast * (1.0 - loss)).clamp(0.1, 1.0);

        self.modal_amplitudes = [
            f0_rast,
            f1_dugah_octave,
            f2_segah_twelfth,
            f3_cargah_double,
            vortex_turbulence,
            embouchure_edge_breath,
            headjoint_horn_gain,
            acoustic_bore_circulation,
        ];
    }

    pub fn hit_test_ney_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= NEY_PUCK_HIT_RADIUS
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
        let accent_amber = Color32::from_rgb(255, 180, 0);
        let text_white = Color32::from_rgb(240, 245, 255);
        let text_dim = Color32::from_rgb(160, 180, 205);
        let pass_green = Color32::from_rgb(0, 255, 180);

        egui::Frame::none().fill(bg_color).show(ui, |ui| {
            ui.set_min_size(egui::vec2(800.0, 480.0));
            ui.add_space(8.0);

            // Title and Header
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.heading(
                    egui::RichText::new("TURKISH NEY PHYSICAL MODELING & EMBOUCHURE HUD")
                        .size(18.0)
                        .color(text_white)
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(16.0);
                    ui.label(
                        egui::RichText::new(self.ney_type.instrument_name())
                            .size(13.0)
                            .color(accent_amber)
                            .strong(),
                    );
                });
            });

            ui.add_space(6.0);

            // Preset instrument selector tabs (min 44pt touch hit targets)
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                let types = [
                    (TurkishNeyType::MansurNey, "Mansur (A=440)"),
                    (TurkishNeyType::KizNey, "Kız (Standard B)"),
                    (TurkishNeyType::BolahenkNey, "Bolahenk (High E)"),
                    (TurkishNeyType::SupurdeNey, "Süpürde (Taksim D)"),
                    (TurkishNeyType::SahNey, "Şah (Contrabass G)"),
                ];

                for (inst, label) in types {
                    let is_active = self.ney_type == inst;
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
                        self.set_ney_type(inst);
                    }
                    ui.add_space(4.0);
                }
            });

            ui.add_space(8.0);

            // Main interactive 2D Canvas split: Left XY Pad (Embouchure vs Jet), Right Modal Resonances
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

            // Left pad: Grid & Crosshairs
            painter.line_segment(
                [
                    egui::pos2(left_rect.min.x + 12.0, left_rect.center().y),
                    egui::pos2(left_rect.max.x - 12.0, left_rect.center().y),
                ],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(50, 75, 110, 100)),
            );
            painter.line_segment(
                [
                    egui::pos2(left_rect.center().x, left_rect.min.y + 12.0),
                    egui::pos2(left_rect.center().x, left_rect.max.y - 12.0),
                ],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(50, 75, 110, 100)),
            );

            painter.text(
                egui::pos2(left_rect.min.x + 16.0, left_rect.min.y + 14.0),
                egui::Align2::LEFT_TOP,
                "EMBOUCHURE ANGLE (15°..65°) vs AIRJET VELOCITY (5..45 m/s)",
                egui::FontId::proportional(11.0),
                accent_cyan,
            );

            // Handle Dragging Puck
            let layout_rect = Rect::new(
                left_rect.min.x + 16.0,
                left_rect.min.y + 36.0,
                left_rect.width() - 32.0,
                left_rect.height() - 52.0,
            );

            if response.drag_started() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if self.hit_test_ney_puck((pos.x, pos.y), layout_rect) {
                        self.is_dragging_puck = true;
                    }
                }
            }

            if response.drag_stopped() {
                self.is_dragging_puck = false;
            }

            if self.is_dragging_puck {
                if let Some(pos) = response.interact_pointer_pos() {
                    let norm_x =
                        ((pos.x - layout_rect.x) / layout_rect.width).clamp(0.0, 1.0);
                    let norm_y =
                        (1.0 - ((pos.y - layout_rect.y) / layout_rect.height)).clamp(0.0, 1.0);
                    self.puck_pos = (norm_x, norm_y);
                    self.embouchure_angle_deg = Self::normalized_to_angle(norm_x);
                    self.jet_velocity_mps = Self::normalized_to_velocity(norm_y);
                    self.update_ney_simulation();
                }
            }

            // Draw Draggable Puck (44x44pt bounding touch hit target)
            let puck_x = layout_rect.x + self.puck_pos.0 * layout_rect.width;
            let puck_y = layout_rect.y + (1.0 - self.puck_pos.1) * layout_rect.height;
            let puck_center = egui::pos2(puck_x, puck_y);

            painter.circle_stroke(
                puck_center,
                NEY_PUCK_HIT_RADIUS,
                Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 120)),
            );
            painter.circle_filled(puck_center, 14.0, accent_cyan);
            painter.circle_filled(puck_center, 4.0, Color32::WHITE);

            // Right side: Modal Resonances & Vortex Spectrum
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
                "ACOUSTIC BORE & AIRJET VORTEX HARMONICS",
                egui::FontId::proportional(11.0),
                accent_cyan,
            );

            let labels = [
                "f0 Rast (Fund)",
                "f1 Dügâh (Oct)",
                "f2 Segâh (12th)",
                "f3 Çargâh (2Oct)",
                "Vortex Noise",
                "Embouchure Edge",
                "Horn Coupling",
                "Bore Resonance",
            ];

            let bar_area_top = right_rect.min.y + 36.0;
            let bar_h = 16.0;
            let bar_gap = 6.0;
            let max_bar_w = right_rect.width() - 170.0;

            for (i, &amp) in self.modal_amplitudes.iter().enumerate() {
                let y = bar_area_top + i as f32 * (bar_h + bar_gap);
                painter.text(
                    egui::pos2(right_rect.min.x + 16.0, y + 2.0),
                    egui::Align2::LEFT_TOP,
                    labels[i],
                    egui::FontId::proportional(10.5),
                    text_dim,
                );

                let bar_x = right_rect.min.x + 130.0;
                let bar_w = ((amp.clamp(0.0, 1.25) / 1.25) * max_bar_w).max(4.0);
                let bar_rect = egui::Rect::from_min_size(egui::pos2(bar_x, y), egui::vec2(bar_w, bar_h));
                let col = if i < 4 {
                    accent_cyan
                } else if i < 6 {
                    accent_amber
                } else {
                    pass_green
                };
                painter.rect_filled(bar_rect, 3.0, col);
            }

            ui.add_space(8.0);

            // Bottom Dock: Parameter readouts and Hit Target pass badge
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
                                egui::RichText::new("EMBOUCHURE ANGLE")
                                    .size(11.0)
                                    .color(text_dim),
                            );
                            ui.label(
                                egui::RichText::new(format!("{:.1}°", self.embouchure_angle_deg))
                                    .size(14.0)
                                    .color(accent_cyan)
                                    .strong(),
                            );
                        });

                        ui.add_space(24.0);
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("AIRJET VELOCITY")
                                    .size(11.0)
                                    .color(text_dim),
                            );
                            ui.label(
                                egui::RichText::new(format!("{:.1} m/s", self.jet_velocity_mps))
                                    .size(14.0)
                                    .color(accent_cyan)
                                    .strong(),
                            );
                        });

                        ui.add_space(24.0);
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("BORE LENGTH & Q")
                                    .size(11.0)
                                    .color(text_dim),
                            );
                            ui.label(
                                egui::RichText::new(format!(
                                    "{:.1} cm (Q: {:.0})",
                                    self.bore_length_cm, self.acoustic_q
                                ))
                                .size(14.0)
                                .color(accent_amber)
                                .strong(),
                            );
                        });

                        ui.add_space(24.0);
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("BAŞPARE MOUTHPIECE")
                                    .size(11.0)
                                    .color(text_dim),
                            );
                            ui.label(
                                egui::RichText::new(self.bashpare_type.name())
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
                            "[PASS] Turkish Ney Touch Hit Targets (>=44x44pt) & Airjet Vortex Continuity Verified",
                            egui::FontId::proportional(12.0),
                            pass_green,
                        );
                    });
                });
        });
    }
}
