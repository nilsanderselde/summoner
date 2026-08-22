// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Neural Acoustic Wavefront Boundary Reflection Raytracing & Diffraction Kernel Neural Network HUD (Step 1634).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const WAVEFRONT_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_SCATTERING_COEFF: f32 = 0.0;
pub const MAX_SCATTERING_COEFF: f32 = 1.0;
pub const MIN_ABSORPTION_ALPHA: f32 = 0.0;
pub const MAX_ABSORPTION_ALPHA: f32 = 1.0;
pub const MIN_RAY_COUNT: usize = 100;
pub const MAX_RAY_COUNT: usize = 5000;

/// Acoustic boundary reflection and room architecture profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomArchitectureType {
    ConcertHallHorseshoe, // Classical proscenium hall with rich diffuse lateral reflections
    ShoeboxStudioControl, // LEDE (Live-End Dead-End) front reflection-free zone
    CathedralStoneVault,  // Long reverberant multi-bounce specular reflection field
    ModularVocalBooth,    // High absorption porous boundary treatment
    AsymmetricalGallery,  // Complex non-parallel geometry with multi-edge diffraction
}

impl RoomArchitectureType {
    pub fn architecture_name(&self) -> &'static str {
        match self {
            Self::ConcertHallHorseshoe => "CONCERT HALL HORSESHOE",
            Self::ShoeboxStudioControl => "SHOEBOX STUDIO CONTROL (LEDE)",
            Self::CathedralStoneVault => "CATHEDRAL STONE VAULT",
            Self::ModularVocalBooth => "MODULAR VOCAL BOOTH",
            Self::AsymmetricalGallery => "ASYMMETRICAL GALLERY",
        }
    }

    pub fn nominal_scattering(&self) -> f32 {
        match self {
            Self::ConcertHallHorseshoe => 0.45,
            Self::ShoeboxStudioControl => 0.25,
            Self::CathedralStoneVault => 0.15,
            Self::ModularVocalBooth => 0.85,
            Self::AsymmetricalGallery => 0.55,
        }
    }

    pub fn nominal_absorption(&self) -> f32 {
        match self {
            Self::ConcertHallHorseshoe => 0.20,
            Self::ShoeboxStudioControl => 0.60,
            Self::CathedralStoneVault => 0.08,
            Self::ModularVocalBooth => 0.80,
            Self::AsymmetricalGallery => 0.35,
        }
    }

    pub fn nominal_ray_count(&self) -> usize {
        match self {
            Self::ConcertHallHorseshoe => 2500,
            Self::ShoeboxStudioControl => 1800,
            Self::CathedralStoneVault => 4000,
            Self::ModularVocalBooth => 1000,
            Self::AsymmetricalGallery => 3000,
        }
    }

    pub fn nominal_early_window_ms(&self) -> f32 {
        match self {
            Self::ConcertHallHorseshoe => 80.0,
            Self::ShoeboxStudioControl => 35.0,
            Self::CathedralStoneVault => 120.0,
            Self::ModularVocalBooth => 15.0,
            Self::AsymmetricalGallery => 65.0,
        }
    }
}

/// Neural acoustic wavefront boundary reflection raytracing HUD.
#[derive(Debug, Clone)]
pub struct WavefrontReflectionView {
    pub architecture: RoomArchitectureType,
    pub surface_scattering_coeff: f32,  // [0.0, 1.0]
    pub boundary_absorption_alpha: f32, // [0.0, 1.0]
    pub ray_count: usize,
    pub early_window_ms: f32,
    pub neural_kernel_depth: f32, // [0.0, 1.0]
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    pub reflection_energy_taps: [f32; 16], // 16-tap time-domain impulse energy profile
    pub color_palette: ContrastColorPalette,
}

impl Default for WavefrontReflectionView {
    fn default() -> Self {
        Self::new()
    }
}

impl WavefrontReflectionView {
    pub fn new() -> Self {
        let mut view = Self {
            architecture: RoomArchitectureType::ConcertHallHorseshoe,
            surface_scattering_coeff: 0.45,
            boundary_absorption_alpha: 0.20,
            ray_count: 2500,
            early_window_ms: 80.0,
            neural_kernel_depth: 0.75,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            reflection_energy_taps: [
                1.0, 0.85, 0.72, 0.65, 0.58, 0.50, 0.42, 0.36, 0.30, 0.25, 0.20, 0.16, 0.12, 0.09,
                0.06, 0.04,
            ],
            color_palette: ContrastColorPalette::default(),
        };
        view.puck_pos = (
            view.surface_scattering_coeff,
            1.0 - view.boundary_absorption_alpha,
        );
        view.update_wavefront_simulation();
        view
    }

    pub fn set_architecture(&mut self, arch: RoomArchitectureType) {
        self.architecture = arch;
        self.surface_scattering_coeff = arch.nominal_scattering();
        self.boundary_absorption_alpha = arch.nominal_absorption();
        self.ray_count = arch.nominal_ray_count();
        self.early_window_ms = arch.nominal_early_window_ms();
        self.puck_pos = (
            self.surface_scattering_coeff,
            1.0 - self.boundary_absorption_alpha,
        );
        self.update_wavefront_simulation();
    }

    pub fn update_wavefront_simulation(&mut self) {
        let s = self.surface_scattering_coeff.clamp(0.0, 1.0);
        let a = self.boundary_absorption_alpha.clamp(0.0, 1.0);
        let r_factor = (self.ray_count as f32 / 3000.0).clamp(0.2, 1.5);
        let neural = self.neural_kernel_depth.clamp(0.0, 1.0);

        // Compute 16 early reflection energy taps with exponential decay and diffusion scattering
        let decay_rate = 0.85 * (1.0 - a * 0.75) * r_factor;
        for i in 0..16 {
            let t = i as f32;
            let specular = (-0.18 * t * (1.0 + a * 1.5)).exp();
            let diffuse = (s * 0.45 * (-0.08 * t).exp() * (1.0 + neural * 0.25)).clamp(0.0, 0.6);
            let energy = ((specular + diffuse) * decay_rate).clamp(0.01, 1.25);
            self.reflection_energy_taps[i] = energy;
        }
    }

    pub fn hit_test_wavefront_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= WAVEFRONT_PUCK_HIT_RADIUS
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
            grid[p_row][p_col] = 'W';
        }

        let right_w = width - mid_x - 3;
        for (i, &amp) in self
            .reflection_energy_taps
            .iter()
            .enumerate()
            .take(height - 4)
        {
            let row = 2 + i;
            let bar_len = ((amp.clamp(0.0, 1.25) / 1.25) * right_w as f32).round() as usize;
            for c in 0..bar_len {
                if mid_x + 2 + c < width - 1 {
                    grid[row][mid_x + 2 + c] = '*';
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

        egui::Frame::none().fill(bg_color).show(ui, |ui| {
            ui.set_min_size(egui::vec2(800.0, 480.0));
            ui.add_space(8.0);

            // Title and Header
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.heading(
                    egui::RichText::new("NEURAL ACOUSTIC BOUNDARY RAYTRACING & DIFFRACTION HUD")
                        .size(18.0)
                        .color(text_white)
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(16.0);
                    ui.label(
                        egui::RichText::new(self.architecture.architecture_name())
                            .size(13.0)
                            .color(accent_amber)
                            .strong(),
                    );
                });
            });

            ui.add_space(6.0);

            // Architecture selector tabs (min 44pt touch hit targets)
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                let archs = [
                    (RoomArchitectureType::ConcertHallHorseshoe, "Concert Hall"),
                    (RoomArchitectureType::ShoeboxStudioControl, "Studio Control"),
                    (RoomArchitectureType::CathedralStoneVault, "Cathedral Vault"),
                    (RoomArchitectureType::ModularVocalBooth, "Vocal Booth"),
                    (RoomArchitectureType::AsymmetricalGallery, "Gallery Stage"),
                ];

                for (arch, label) in archs {
                    let is_active = self.architecture == arch;
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
                        self.set_architecture(arch);
                    }
                    ui.add_space(4.0);
                }
            });

            ui.add_space(8.0);

            // Main interactive 2D Canvas split: Left XY Pad (Scattering vs Reflection), Right 16-Tap Early Reflection Profile
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
                    self.surface_scattering_coeff = norm_x;
                    self.boundary_absorption_alpha = 1.0 - norm_y;
                    self.update_wavefront_simulation();
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
                WAVEFRONT_PUCK_HIT_RADIUS,
                Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 140)),
            );
            painter.circle_filled(puck_center, 12.0, accent_cyan);
            painter.circle_filled(puck_center, 4.0, Color32::WHITE);

            painter.text(
                egui::pos2(pad_inner.min.x, pad_inner.min.y - 12.0),
                egui::Align2::LEFT_BOTTOM,
                "SURFACE SCATTERING (X) / REFLECTION (1-ALPHA) (Y)",
                egui::FontId::proportional(11.0),
                accent_cyan,
            );

            // Right Section (16-Tap Early Reflection Energy Histogram)
            painter.text(
                egui::pos2(right_rect.min.x + 16.0, right_rect.min.y + 14.0),
                egui::Align2::LEFT_TOP,
                "EARLY REFLECTION IMPULSE DECAY (16 TAPS)",
                egui::FontId::proportional(11.0),
                accent_green,
            );

            let hist_rect = egui::Rect::from_min_max(
                egui::pos2(right_rect.min.x + 16.0, right_rect.min.y + 38.0),
                egui::pos2(right_rect.max.x - 16.0, right_rect.max.y - 20.0),
            );
            painter.rect_filled(hist_rect, 4.0, Color32::from_rgb(12, 16, 24));
            painter.rect_stroke(hist_rect, 4.0, Stroke::new(1.0_f32, border_color));

            let tap_w = hist_rect.width() / 16.0;
            for (i, &amp) in self.reflection_energy_taps.iter().enumerate() {
                let x0 = hist_rect.min.x + i as f32 * tap_w + 2.0;
                let bar_w = (tap_w - 4.0).max(2.0);
                let bar_h = (amp.clamp(0.0, 1.25) / 1.25) * (hist_rect.height() - 16.0);
                let y0 = hist_rect.max.y - bar_h - 8.0;

                let bar_rect =
                    egui::Rect::from_min_size(egui::pos2(x0, y0), egui::vec2(bar_w, bar_h));
                let color = if i < 4 {
                    accent_cyan // Early specular
                } else if i < 10 {
                    accent_green // Mixed diffuse
                } else {
                    accent_amber // Late tail
                };
                painter.rect_filled(bar_rect, 2.0, color);
            }

            ui.add_space(8.0);

            // Bottom controls: Sliders
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new("SCATTERING:")
                        .size(12.0)
                        .color(accent_cyan)
                        .strong(),
                );
                let scat_slider = egui::Slider::new(&mut self.surface_scattering_coeff, 0.0..=1.0)
                    .show_value(true);
                if ui.add(scat_slider).changed() {
                    self.puck_pos.0 = self.surface_scattering_coeff;
                    self.update_wavefront_simulation();
                }

                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new("ABSORPTION:")
                        .size(12.0)
                        .color(accent_green)
                        .strong(),
                );
                let abs_slider = egui::Slider::new(&mut self.boundary_absorption_alpha, 0.0..=1.0)
                    .show_value(true);
                if ui.add(abs_slider).changed() {
                    self.puck_pos.1 = 1.0 - self.boundary_absorption_alpha;
                    self.update_wavefront_simulation();
                }

                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new("NEURAL DEPTH:")
                        .size(12.0)
                        .color(accent_amber)
                        .strong(),
                );
                let neural_slider =
                    egui::Slider::new(&mut self.neural_kernel_depth, 0.0..=1.0).show_value(true);
                if ui.add(neural_slider).changed() {
                    self.update_wavefront_simulation();
                }
            });
        });
    }
}
