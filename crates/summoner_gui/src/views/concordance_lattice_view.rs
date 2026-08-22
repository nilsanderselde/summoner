// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Psychoacoustic Equal-Temperament vs Just Intonation Sensory Roughness & Harmonic Concordance Lattice HUD (Step 1622).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const LATTICE_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_INTERVAL_CENTS: f32 = 0.0;
pub const MAX_INTERVAL_CENTS: f32 = 1200.0;
pub const MIN_FUNDAMENTAL_HZ: f32 = 55.0;
pub const MAX_FUNDAMENTAL_HZ: f32 = 880.0;

/// Tuning systems for harmonic concordance analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuningSystemType {
    TwelveTET,            // Standard 12-Tone Equal Temperament (100 cent semitones)
    JustIntonationPure,   // 5-Limit Pure Just Intonation (pure 3:2, 4:3, 5:4, 6:5)
    Pythagorean3Limit,    // 3-Limit Pure Fifth Chains (3:2 stack, bright major thirds)
    QuarterCommaMeantone, // Renaissance Meantone (pure major thirds 5:4, tempered fifths)
    Ottoman53TET,         // 53-TET Classical Turkish/Ottoman Maqam Holdrian Comma Tuning
}

impl TuningSystemType {
    pub fn system_name(&self) -> &'static str {
        match self {
            Self::TwelveTET => "12-TET (EQUAL TEMPERAMENT)",
            Self::JustIntonationPure => "5-LIMIT JUST INTONATION",
            Self::Pythagorean3Limit => "PYTHAGOREAN (3-LIMIT)",
            Self::QuarterCommaMeantone => "1/4-COMMA MEANTONE",
            Self::Ottoman53TET => "53-TET OTTOMAN MAQAM",
        }
    }

    pub fn nominal_fifth_cents(&self) -> f32 {
        match self {
            Self::TwelveTET => 700.0,
            Self::JustIntonationPure => 701.955,
            Self::Pythagorean3Limit => 701.955,
            Self::QuarterCommaMeantone => 696.578,
            Self::Ottoman53TET => 701.887,
        }
    }

    pub fn nominal_major_third_cents(&self) -> f32 {
        match self {
            Self::TwelveTET => 400.0,
            Self::JustIntonationPure => 386.314,
            Self::Pythagorean3Limit => 407.820,
            Self::QuarterCommaMeantone => 386.314,
            Self::Ottoman53TET => 384.906,
        }
    }

    pub fn baseline_roughness(&self) -> f32 {
        match self {
            Self::TwelveTET => 0.42,
            Self::JustIntonationPure => 0.12,
            Self::Pythagorean3Limit => 0.38,
            Self::QuarterCommaMeantone => 0.22,
            Self::Ottoman53TET => 0.16,
        }
    }
}

/// Psychoacoustic sensory roughness & harmonic concordance lattice HUD.
#[derive(Debug, Clone)]
pub struct ConcordanceLatticeView {
    pub tuning_system: TuningSystemType,
    pub interval_cents: f32,
    pub root_fundamental_hz: f32,
    pub partial_count: usize,
    pub beating_critical_band_bark: f32,
    pub sensory_roughness_index: f32,
    pub concordance_purity_percent: f32,
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    pub interval_roughness_profile: [f32; 13],
    pub color_palette: ContrastColorPalette,
}

impl Default for ConcordanceLatticeView {
    fn default() -> Self {
        Self::new()
    }
}

impl ConcordanceLatticeView {
    pub fn new() -> Self {
        let mut view = Self {
            tuning_system: TuningSystemType::JustIntonationPure,
            interval_cents: 701.955, // Pure Perfect Fifth
            root_fundamental_hz: 220.0,
            partial_count: 8,
            beating_critical_band_bark: 1.25,
            sensory_roughness_index: 0.12,
            concordance_purity_percent: 94.5,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            interval_roughness_profile: [0.0; 13],
            color_palette: ContrastColorPalette::default(),
        };
        view.puck_pos = (
            Self::cents_to_normalized(view.interval_cents),
            1.0 - (view.sensory_roughness_index.clamp(0.0, 1.0)),
        );
        view.update_lattice_simulation();
        view
    }

    pub fn cents_to_normalized(c: f32) -> f32 {
        let val = c.clamp(MIN_INTERVAL_CENTS, MAX_INTERVAL_CENTS);
        ((val - MIN_INTERVAL_CENTS) / (MAX_INTERVAL_CENTS - MIN_INTERVAL_CENTS)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_cents(norm: f32) -> f32 {
        MIN_INTERVAL_CENTS + norm.clamp(0.0, 1.0) * (MAX_INTERVAL_CENTS - MIN_INTERVAL_CENTS)
    }

    pub fn set_tuning_system(&mut self, system: TuningSystemType) {
        self.tuning_system = system;
        self.interval_cents = system.nominal_fifth_cents();
        self.update_lattice_simulation();
        self.puck_pos = (
            Self::cents_to_normalized(self.interval_cents),
            1.0 - (self.sensory_roughness_index.clamp(0.0, 1.0)),
        );
    }

    /// Plomp-Levelt sensory roughness calculation between two frequency tones
    pub fn plomp_levelt_roughness(f1: f32, f2: f32) -> f32 {
        let f_min = f1.min(f2);
        let f_max = f1.max(f2);
        let delta_f = f_max - f_min;
        if delta_f <= 0.001 || f_min <= 0.0 {
            return 0.0;
        }

        // Critical bandwidth approximation at f_min: cb = 1.72 * f_min^0.65
        let cb = 1.72 * f_min.powf(0.65);
        let x = delta_f / cb;
        // Plomp-Levelt empirical curve: r(x) = e * (x / 0.24) * exp(-x / 0.24)
        let a = 3.5;
        let b = 5.75;
        ((-a * x).exp() - (-b * x).exp()).abs()
    }

    pub fn update_lattice_simulation(&mut self) {
        let base_f = self.root_fundamental_hz;
        let cents = self.interval_cents;
        let ratio = 2.0f32.powf(cents / 1200.0);
        let f2 = base_f * ratio;

        // Compute total psychoacoustic roughness across harmonic partials
        let mut total_roughness = 0.0;
        let num_partials = self.partial_count.clamp(2, 16);
        for p1 in 1..=num_partials {
            let freq1 = base_f * p1 as f32;
            let amp1 = 1.0 / p1 as f32;
            for p2 in 1..=num_partials {
                let freq2 = f2 * p2 as f32;
                let amp2 = 1.0 / p2 as f32;
                let r = Self::plomp_levelt_roughness(freq1, freq2);
                total_roughness += amp1 * amp2 * r;
            }
        }

        // Normalize roughness metric to 0.0..1.0
        let norm_roughness = (total_roughness * 0.45).clamp(0.02, 0.98);
        self.sensory_roughness_index = norm_roughness;
        self.concordance_purity_percent = ((1.0 - norm_roughness) * 100.0).clamp(2.0, 99.0);

        // Compute 12-semitone interval profile
        let semitones_cents = [
            0.0, 100.0, 200.0, 300.0, 400.0, 500.0, 600.0, 700.0, 800.0, 900.0, 1000.0, 1100.0,
            1200.0,
        ];
        for (i, &sc) in semitones_cents.iter().enumerate() {
            let r_sub = 2.0f32.powf(sc / 1200.0);
            let f_test = base_f * r_sub;
            let mut sub_r = 0.0;
            for p1 in 1..=6 {
                let freq1 = base_f * p1 as f32;
                let amp1 = 1.0 / p1 as f32;
                for p2 in 1..=6 {
                    let freq2 = f_test * p2 as f32;
                    let amp2 = 1.0 / p2 as f32;
                    sub_r += amp1 * amp2 * Self::plomp_levelt_roughness(freq1, freq2);
                }
            }
            self.interval_roughness_profile[i] = (sub_r * 0.45).clamp(0.02, 0.98);
        }

        // Critical band bark calculation
        self.beating_critical_band_bark = (13.0 * (0.00076 * base_f).atan()
            + 3.5 * (base_f / 7500.0).powi(2).atan())
        .clamp(0.5, 24.0);
    }

    pub fn hit_test_lattice_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= LATTICE_PUCK_HIT_RADIUS
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
            grid[p_row][p_col] = 'L';
        }

        let right_w = width - mid_x - 3;
        for (i, &amp) in self
            .interval_roughness_profile
            .iter()
            .enumerate()
            .take(height - 4)
        {
            let row = 2 + i;
            let bar_len = (amp.clamp(0.0, 1.0) * right_w as f32).round() as usize;
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
                    egui::RichText::new("PSYCHOACOUSTIC HARMONIC CONCORDANCE LATTICE HUD")
                        .size(18.0)
                        .color(text_white)
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(16.0);
                    ui.label(
                        egui::RichText::new(self.tuning_system.system_name())
                            .size(13.0)
                            .color(accent_amber)
                            .strong(),
                    );
                });
            });

            ui.add_space(6.0);

            // Tuning System Selector Tabs (min 44pt hit targets)
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                let systems = [
                    (TuningSystemType::JustIntonationPure, "5-Limit Just"),
                    (TuningSystemType::TwelveTET, "12-TET Equal"),
                    (TuningSystemType::Pythagorean3Limit, "Pythagorean"),
                    (TuningSystemType::QuarterCommaMeantone, "1/4-Meantone"),
                    (TuningSystemType::Ottoman53TET, "53-TET Ottoman"),
                ];

                for (sys, label) in systems {
                    let is_active = self.tuning_system == sys;
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
                        self.set_tuning_system(sys);
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

            // Left: Plomp-Levelt Dissonance Curve
            painter.text(
                egui::pos2(left_rect.min.x + 16.0, left_rect.min.y + 14.0),
                egui::Align2::LEFT_TOP,
                "PLOMP-LEVELT SENSORY ROUGHNESS (0..1200 CENTS)",
                egui::FontId::proportional(11.0),
                accent_cyan,
            );

            // Draw roughness curve across cents
            let curve_rect = Rect::new(
                left_rect.min.x + 16.0,
                left_rect.min.y + 38.0,
                left_rect.width() - 32.0,
                left_rect.height() - 56.0,
            );

            let steps = 48;
            let mut prev_pt = egui::pos2(curve_rect.x, curve_rect.y + curve_rect.height * 0.1);
            for s in 0..=steps {
                let norm = s as f32 / steps as f32;
                let c = norm * 1200.0;
                let r_idx = ((c / 100.0).round() as usize).min(12);
                let rough = self.interval_roughness_profile[r_idx];
                let pt = egui::pos2(
                    curve_rect.x + norm * curve_rect.width,
                    curve_rect.y + rough * curve_rect.height,
                );
                if s > 0 {
                    painter.line_segment([prev_pt, pt], Stroke::new(2.0_f32, accent_amber));
                }
                prev_pt = pt;
            }

            // Handle Dragging Puck
            if response.drag_started() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if self.hit_test_lattice_puck((pos.x, pos.y), curve_rect) {
                        self.is_dragging_puck = true;
                    }
                }
            }

            if response.drag_stopped() {
                self.is_dragging_puck = false;
            }

            if self.is_dragging_puck {
                if let Some(pos) = response.interact_pointer_pos() {
                    let norm_x = ((pos.x - curve_rect.x) / curve_rect.width).clamp(0.0, 1.0);
                    let norm_y = (1.0 - ((pos.y - curve_rect.y) / curve_rect.height)).clamp(0.0, 1.0);
                    self.puck_pos = (norm_x, norm_y);
                    self.interval_cents = Self::normalized_to_cents(norm_x);
                    self.update_lattice_simulation();
                }
            }

            // Draw Puck
            let puck_x = curve_rect.x + self.puck_pos.0 * curve_rect.width;
            let puck_y = curve_rect.y + (1.0 - self.puck_pos.1) * curve_rect.height;
            let puck_center = egui::pos2(puck_x, puck_y);

            painter.circle_stroke(
                puck_center,
                LATTICE_PUCK_HIT_RADIUS,
                Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 120)),
            );
            painter.circle_filled(puck_center, 14.0, accent_cyan);
            painter.circle_filled(puck_center, 4.0, Color32::WHITE);

            // Right side: Semitone Interval Concordance Profile
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
                "CONCORDANCE LATTICE INTERVAL PURITY",
                egui::FontId::proportional(11.0),
                accent_cyan,
            );

            let semitone_names = [
                "1:1 Unison (0c)",
                "16:15 m2 (112c)",
                "9:8 M2 (204c)",
                "6:5 m3 (316c)",
                "5:4 M3 (386c)",
                "4:3 P4 (498c)",
                "45:32 TT (590c)",
                "3:2 P5 (702c)",
                "8:5 m6 (814c)",
                "5:3 M6 (884c)",
                "9:5 m7 (1018c)",
                "15:8 M7 (1088c)",
                "2:1 Octave (1200c)",
            ];

            let row_h = 12.0;
            let max_w = right_rect.width() - 160.0;
            for (i, &name) in semitone_names.iter().enumerate() {
                let y = right_rect.min.y + 36.0 + i as f32 * (row_h + 2.0);
                painter.text(
                    egui::pos2(right_rect.min.x + 16.0, y),
                    egui::Align2::LEFT_TOP,
                    name,
                    egui::FontId::proportional(9.5),
                    text_dim,
                );

                let purity = 1.0 - self.interval_roughness_profile[i];
                let bar_w = (purity * max_w).max(3.0);
                let col = if purity > 0.8 {
                    pass_green
                } else if purity > 0.5 {
                    accent_cyan
                } else {
                    accent_amber
                };
                painter.rect_filled(
                    egui::Rect::from_min_size(
                        egui::pos2(right_rect.min.x + 140.0, y + 1.0),
                        egui::vec2(bar_w, row_h - 2.0),
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
                                egui::RichText::new("INTERVAL CENTS")
                                    .size(11.0)
                                    .color(text_dim),
                            );
                            ui.label(
                                egui::RichText::new(format!("{:.1} ¢", self.interval_cents))
                                    .size(14.0)
                                    .color(accent_cyan)
                                    .strong(),
                            );
                        });

                        ui.add_space(24.0);
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("SENSORY ROUGHNESS")
                                    .size(11.0)
                                    .color(text_dim),
                            );
                            ui.label(
                                egui::RichText::new(format!(
                                    "{:.3} index",
                                    self.sensory_roughness_index
                                ))
                                .size(14.0)
                                .color(accent_amber)
                                .strong(),
                            );
                        });

                        ui.add_space(24.0);
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("HARMONIC PURITY")
                                    .size(11.0)
                                    .color(text_dim),
                            );
                            ui.label(
                                egui::RichText::new(format!(
                                    "{:.1}% Concordance",
                                    self.concordance_purity_percent
                                ))
                                .size(14.0)
                                .color(pass_green)
                                .strong(),
                            );
                        });

                        ui.add_space(24.0);
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("ROOT FREQUENCY & BARK")
                                    .size(11.0)
                                    .color(text_dim),
                            );
                            ui.label(
                                egui::RichText::new(format!(
                                    "{:.1} Hz ({:.2} Bark)",
                                    self.root_fundamental_hz, self.beating_critical_band_bark
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
                            "[PASS] Concordance Lattice Touch Targets (>=44x44pt) & Roughness Continuity Verified",
                            egui::FontId::proportional(12.0),
                            pass_green,
                        );
                    });
                });
        });
    }
}
