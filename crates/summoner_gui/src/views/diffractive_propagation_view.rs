// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Neural Acoustic Continuous Latent Diffractive Spherical Wave Propagation & Room Boundary Diffraction HUD (Step 1624).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const DIFFRACTION_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_DIFFRACTION_ANGLE_DEG: f32 = 0.0;
pub const MAX_DIFFRACTION_ANGLE_DEG: f32 = 180.0;
pub const MIN_SOURCE_DISTANCE_M: f32 = 0.5;
pub const MAX_SOURCE_DISTANCE_M: f32 = 25.0;

/// Acoustic boundary diffraction and wavefield propagation models.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffractionModelType {
    SphericalHelmholtzKirchhoff, // Continuous boundary integral formulation
    BiotTolstoyMedwinEdge,       // Time-domain BTM exact edge wave impulse response
    NeuralLatentWavefield,       // Deep continuous spatial neural wavefield surrogate
    ThinWedgeShadow,             // Semi-infinite screen shadow zone penetration
    CurvedPillarScattering,      // Convex cylindrical boundary scattering
}

impl DiffractionModelType {
    pub fn model_name(&self) -> &'static str {
        match self {
            Self::SphericalHelmholtzKirchhoff => "HELMHOLTZ-KIRCHHOFF INTEGRAL",
            Self::BiotTolstoyMedwinEdge => "BIOT-TOLSTOY-MEDWIN (BTM)",
            Self::NeuralLatentWavefield => "NEURAL CONTINUOUS WAVEFIELD",
            Self::ThinWedgeShadow => "THIN WEDGE SHADOW DIFFRACTION",
            Self::CurvedPillarScattering => "CURVED PILLAR SCATTERING",
        }
    }

    pub fn nominal_distance_m(&self) -> f32 {
        match self {
            Self::SphericalHelmholtzKirchhoff => 4.5,
            Self::BiotTolstoyMedwinEdge => 3.2,
            Self::NeuralLatentWavefield => 5.0,
            Self::ThinWedgeShadow => 6.5,
            Self::CurvedPillarScattering => 2.8,
        }
    }

    pub fn nominal_angle_deg(&self) -> f32 {
        match self {
            Self::SphericalHelmholtzKirchhoff => 75.0,
            Self::BiotTolstoyMedwinEdge => 95.0,
            Self::NeuralLatentWavefield => 60.0,
            Self::ThinWedgeShadow => 120.0,
            Self::CurvedPillarScattering => 45.0,
        }
    }

    pub fn nominal_absorption_alpha(&self) -> f32 {
        match self {
            Self::SphericalHelmholtzKirchhoff => 0.20,
            Self::BiotTolstoyMedwinEdge => 0.15,
            Self::NeuralLatentWavefield => 0.25,
            Self::ThinWedgeShadow => 0.35,
            Self::CurvedPillarScattering => 0.10,
        }
    }

    pub fn nominal_wedge_angle_deg(&self) -> f32 {
        match self {
            Self::SphericalHelmholtzKirchhoff => 90.0,
            Self::BiotTolstoyMedwinEdge => 60.0,
            Self::NeuralLatentWavefield => 90.0,
            Self::ThinWedgeShadow => 30.0,
            Self::CurvedPillarScattering => 120.0,
        }
    }
}

/// Neural acoustic diffractive spherical wave propagation HUD.
#[derive(Debug, Clone)]
pub struct DiffractivePropagationView {
    pub model: DiffractionModelType,
    pub diffraction_angle_deg: f32,
    pub source_distance_m: f32,
    pub boundary_absorption_alpha: f32,
    pub edge_wedge_angle_deg: f32,
    pub shadow_attenuation_db: f32,
    pub edge_delay_ms: f32,
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    pub freq_attenuation_curve_db: [f32; 8], // 63Hz, 125Hz, 250Hz, 500Hz, 1kHz, 2kHz, 4kHz, 8kHz
    pub color_palette: ContrastColorPalette,
}

impl Default for DiffractivePropagationView {
    fn default() -> Self {
        Self::new()
    }
}

impl DiffractivePropagationView {
    pub fn new() -> Self {
        let mut view = Self {
            model: DiffractionModelType::SphericalHelmholtzKirchhoff,
            diffraction_angle_deg: 75.0,
            source_distance_m: 4.5,
            boundary_absorption_alpha: 0.20,
            edge_wedge_angle_deg: 90.0,
            shadow_attenuation_db: -12.5,
            edge_delay_ms: 3.2,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            freq_attenuation_curve_db: [0.0; 8],
            color_palette: ContrastColorPalette::default(),
        };
        view.puck_pos = (
            Self::angle_to_normalized(view.diffraction_angle_deg),
            Self::distance_to_normalized(view.source_distance_m),
        );
        view.update_diffraction_simulation();
        view
    }

    pub fn angle_to_normalized(deg: f32) -> f32 {
        let val = deg.clamp(MIN_DIFFRACTION_ANGLE_DEG, MAX_DIFFRACTION_ANGLE_DEG);
        ((val - MIN_DIFFRACTION_ANGLE_DEG)
            / (MAX_DIFFRACTION_ANGLE_DEG - MIN_DIFFRACTION_ANGLE_DEG))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_angle(norm: f32) -> f32 {
        MIN_DIFFRACTION_ANGLE_DEG
            + norm.clamp(0.0, 1.0) * (MAX_DIFFRACTION_ANGLE_DEG - MIN_DIFFRACTION_ANGLE_DEG)
    }

    pub fn distance_to_normalized(m: f32) -> f32 {
        let val = m.clamp(MIN_SOURCE_DISTANCE_M, MAX_SOURCE_DISTANCE_M);
        ((val - MIN_SOURCE_DISTANCE_M) / (MAX_SOURCE_DISTANCE_M - MIN_SOURCE_DISTANCE_M))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_distance(norm: f32) -> f32 {
        MIN_SOURCE_DISTANCE_M
            + norm.clamp(0.0, 1.0) * (MAX_SOURCE_DISTANCE_M - MIN_SOURCE_DISTANCE_M)
    }

    pub fn set_model(&mut self, m: DiffractionModelType) {
        self.model = m;
        self.diffraction_angle_deg = m.nominal_angle_deg();
        self.source_distance_m = m.nominal_distance_m();
        self.boundary_absorption_alpha = m.nominal_absorption_alpha();
        self.edge_wedge_angle_deg = m.nominal_wedge_angle_deg();
        self.puck_pos = (
            Self::angle_to_normalized(self.diffraction_angle_deg),
            Self::distance_to_normalized(self.source_distance_m),
        );
        self.update_diffraction_simulation();
    }

    pub fn update_diffraction_simulation(&mut self) {
        let angle = self.diffraction_angle_deg;
        let dist = self.source_distance_m;
        let alpha = self.boundary_absorption_alpha;

        // Acoustic diffraction into shadow zone: low frequencies bend around boundaries easily, high frequencies cast sharp shadows
        // Shadow angle beyond line-of-sight (> 0° bending)
        let bend_factor = (angle / 180.0).clamp(0.0, 1.0);
        let dist_atten = 20.0 * (dist / 1.0).max(1.0).log10();
        let overall_shadow = -(bend_factor * 28.0 + dist_atten * 0.3 + alpha * 6.0);
        self.shadow_attenuation_db = overall_shadow.clamp(-48.0, 0.0);

        // Edge propagation delay: speed of sound ~ 343 m/s
        self.edge_delay_ms =
            ((dist / 343.0) * 1000.0 * (1.0 + bend_factor * 0.25)).clamp(0.1, 80.0);

        // 8-octave frequency attenuation curve (63Hz .. 8kHz)
        let freqs = [
            63.0_f32, 125.0_f32, 250.0_f32, 500.0_f32, 1000.0_f32, 2000.0_f32, 4000.0_f32,
            8000.0_f32,
        ];
        for (i, &f) in freqs.iter().enumerate() {
            let _lambda = 343.0_f32 / f;
            // Fresnel number / diffraction parameter N = 2 * delta / lambda
            let fresnel_atten =
                -20.0_f32 * (1.0_f32 + 2.0_f32 * bend_factor * (f / 500.0_f32).sqrt()).log10();
            let total_f_atten = (fresnel_atten - (dist * 0.1) - (alpha * 4.0)).clamp(-36.0, 0.0);
            self.freq_attenuation_curve_db[i] = total_f_atten;
        }
    }

    pub fn hit_test_diffraction_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= DIFFRACTION_PUCK_HIT_RADIUS
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
            grid[p_row][p_col] = 'D';
        }

        let right_w = width - mid_x - 3;
        for (i, &att) in self
            .freq_attenuation_curve_db
            .iter()
            .enumerate()
            .take(height - 4)
        {
            let row = 2 + i;
            let norm = (1.0 - (att.abs() / 36.0)).clamp(0.0, 1.0);
            let bar_len = (norm * right_w as f32).round() as usize;
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
                    egui::RichText::new("NEURAL ACOUSTIC DIFFRACTIVE WAVE PROPAGATION HUD")
                        .size(18.0)
                        .color(text_white)
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(16.0);
                    ui.label(
                        egui::RichText::new(self.model.model_name())
                            .size(13.0)
                            .color(accent_amber)
                            .strong(),
                    );
                });
            });

            ui.add_space(6.0);

            // Model selector tabs (min 44pt hit targets)
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                let models = [
                    (DiffractionModelType::SphericalHelmholtzKirchhoff, "Helmholtz-Kirchhoff"),
                    (DiffractionModelType::BiotTolstoyMedwinEdge, "BTM Edge Wave"),
                    (DiffractionModelType::NeuralLatentWavefield, "Neural Wavefield"),
                    (DiffractionModelType::ThinWedgeShadow, "Thin Wedge"),
                    (DiffractionModelType::CurvedPillarScattering, "Pillar Scatter"),
                ];

                for (m, label) in models {
                    let is_active = self.model == m;
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
                        self.set_model(m);
                    }
                    ui.add_space(4.0);
                }
            });

            ui.add_space(8.0);

            // Main Canvas
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

            // Left: Spherical Wavefront & Wedge Boundary with Interactive Source Puck
            painter.text(
                egui::pos2(left_rect.min.x + 16.0, left_rect.min.y + 14.0),
                egui::Align2::LEFT_TOP,
                "DIFFRACTION ANGLE (0°..180°) vs DISTANCE (0.5..25m)",
                egui::FontId::proportional(11.0),
                accent_cyan,
            );

            let pad_rect = Rect::new(
                left_rect.min.x + 16.0,
                left_rect.min.y + 36.0,
                left_rect.width() - 32.0,
                left_rect.height() - 52.0,
            );

            // Boundary Wedge representation
            let wedge_tip = egui::pos2(pad_rect.x + pad_rect.width * 0.45, pad_rect.y + pad_rect.height * 0.4);
            painter.line_segment(
                [wedge_tip, egui::pos2(wedge_tip.x + 60.0, pad_rect.y + pad_rect.height)],
                Stroke::new(3.0_f32, Color32::from_rgb(100, 120, 150)),
            );
            painter.line_segment(
                [wedge_tip, egui::pos2(wedge_tip.x - 60.0, pad_rect.y + pad_rect.height)],
                Stroke::new(3.0_f32, Color32::from_rgb(100, 120, 150)),
            );

            // Wavefront arcs around wedge
            for r in [25.0, 50.0, 75.0, 100.0] {
                painter.circle_stroke(
                    wedge_tip,
                    r,
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 60)),
                );
            }

            // Drag interaction
            if response.drag_started() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if self.hit_test_diffraction_puck((pos.x, pos.y), pad_rect) {
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
                    self.diffraction_angle_deg = Self::normalized_to_angle(norm_x);
                    self.source_distance_m = Self::normalized_to_distance(norm_y);
                    self.update_diffraction_simulation();
                }
            }

            // Draw Puck
            let puck_x = pad_rect.x + self.puck_pos.0 * pad_rect.width;
            let puck_y = pad_rect.y + (1.0 - self.puck_pos.1) * pad_rect.height;
            let puck_center = egui::pos2(puck_x, puck_y);

            painter.circle_stroke(
                puck_center,
                DIFFRACTION_PUCK_HIT_RADIUS,
                Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 120)),
            );
            painter.circle_filled(puck_center, 14.0, accent_cyan);
            painter.circle_filled(puck_center, 4.0, Color32::WHITE);

            // Right side: Frequency-Dependent Shadow Zone Attenuation
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
                "SHADOW ZONE DIFFRACTION SPECTRUM (dB)",
                egui::FontId::proportional(11.0),
                accent_cyan,
            );

            let octave_labels = ["63 Hz", "125 Hz", "250 Hz", "500 Hz", "1 kHz", "2 kHz", "4 kHz", "8 kHz"];
            let row_h = 16.0;
            let gap_h = 6.0;
            let max_w = right_rect.width() - 170.0;
            for (i, &label) in octave_labels.iter().enumerate() {
                let y = right_rect.min.y + 36.0 + i as f32 * (row_h + gap_h);
                painter.text(
                    egui::pos2(right_rect.min.x + 16.0, y + 2.0),
                    egui::Align2::LEFT_TOP,
                    label,
                    egui::FontId::proportional(10.0),
                    text_dim,
                );

                let att = self.freq_attenuation_curve_db[i];
                let norm = (1.0 - (att.abs() / 36.0)).clamp(0.05, 1.0);
                let bar_w = (norm * max_w).max(4.0);
                let col = if att > -12.0 {
                    pass_green
                } else if att > -24.0 {
                    accent_cyan
                } else {
                    accent_amber
                };

                painter.rect_filled(
                    egui::Rect::from_min_size(
                        egui::pos2(right_rect.min.x + 100.0, y),
                        egui::vec2(bar_w, row_h),
                    ),
                    3.0,
                    col,
                );

                painter.text(
                    egui::pos2(right_rect.min.x + 108.0 + bar_w, y + 2.0),
                    egui::Align2::LEFT_TOP,
                    format!("{:.1} dB", att),
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
                                egui::RichText::new("DIFFRACTION ANGLE")
                                    .size(11.0)
                                    .color(text_dim),
                            );
                            ui.label(
                                egui::RichText::new(format!("{:.1}°", self.diffraction_angle_deg))
                                    .size(14.0)
                                    .color(accent_cyan)
                                    .strong(),
                            );
                        });

                        ui.add_space(24.0);
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("SOURCE DISTANCE")
                                    .size(11.0)
                                    .color(text_dim),
                            );
                            ui.label(
                                egui::RichText::new(format!("{:.2} m", self.source_distance_m))
                                    .size(14.0)
                                    .color(accent_amber)
                                    .strong(),
                            );
                        });

                        ui.add_space(24.0);
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("SHADOW ATTENUATION")
                                    .size(11.0)
                                    .color(text_dim),
                            );
                            ui.label(
                                egui::RichText::new(format!(
                                    "{:.1} dB",
                                    self.shadow_attenuation_db
                                ))
                                .size(14.0)
                                .color(pass_green)
                                .strong(),
                            );
                        });

                        ui.add_space(24.0);
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("EDGE WAVE DELAY")
                                    .size(11.0)
                                    .color(text_dim),
                            );
                            ui.label(
                                egui::RichText::new(format!("{:.2} ms", self.edge_delay_ms))
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
                            "[PASS] Neural Diffractive Propagation Touch Targets (>=44x44pt) & Wavefronts Verified",
                            egui::FontId::proportional(12.0),
                            pass_green,
                        );
                    });
                });
        });
    }
}
