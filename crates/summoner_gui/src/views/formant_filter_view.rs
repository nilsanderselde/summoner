// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Interactive 5-Vowel Phonetic Formant Filter Resonator View with Vowel Morphing 2D Trajectory Pad (Step 1423).

use crate::layout_math::Rect;
use crate::touch_controls::{ContrastColorPalette, MIN_HIT_TARGET_PT};

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const FORMANT_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const NUM_FORMANTS: usize = 5;

/// Trajectory modulation mode for vowel morphing pad.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormantTrajectoryMode {
    Static,
    CircularLfo,
    Figure8Lfo,
    EnvelopeFollower,
}

/// Standard vowel definitions with reference formant frequencies (Hz).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VowelDefinition {
    pub name: &'static str,
    pub ipa: &'static str,
    pub f1_hz: f32,
    pub f2_hz: f32,
    pub f3_hz: f32,
    pub f4_hz: f32,
    pub f5_hz: f32,
    pub pad_norm_pos: (f32, f32), // (F2 norm, F1 norm)
}

pub const STANDARD_VOWELS: [VowelDefinition; 5] = [
    VowelDefinition {
        name: "A (Father)",
        ipa: "/ɑ/",
        f1_hz: 800.0,
        f2_hz: 1200.0,
        f3_hz: 2500.0,
        f4_hz: 3500.0,
        f5_hz: 4500.0,
        pad_norm_pos: (0.40, 0.85),
    },
    VowelDefinition {
        name: "E (Bed)",
        ipa: "/ɛ/",
        f1_hz: 500.0,
        f2_hz: 1800.0,
        f3_hz: 2500.0,
        f4_hz: 3600.0,
        f5_hz: 4500.0,
        pad_norm_pos: (0.70, 0.50),
    },
    VowelDefinition {
        name: "I (See)",
        ipa: "/i/",
        f1_hz: 280.0,
        f2_hz: 2300.0,
        f3_hz: 3000.0,
        f4_hz: 3700.0,
        f5_hz: 4600.0,
        pad_norm_pos: (0.90, 0.15),
    },
    VowelDefinition {
        name: "O (Boat)",
        ipa: "/o/",
        f1_hz: 450.0,
        f2_hz: 800.0,
        f3_hz: 2500.0,
        f4_hz: 3500.0,
        f5_hz: 4500.0,
        pad_norm_pos: (0.20, 0.45),
    },
    VowelDefinition {
        name: "U (Boot)",
        ipa: "/u/",
        f1_hz: 300.0,
        f2_hz: 700.0,
        f3_hz: 2200.0,
        f4_hz: 3400.0,
        f5_hz: 4500.0,
        pad_norm_pos: (0.10, 0.15),
    },
];

/// Interactive 5-Vowel Formant Filter Resonator View (Step 1423).
#[derive(Debug, Clone)]
pub struct FormantFilterView {
    pub active_vowel_idx: usize,
    pub morph_pos: (f32, f32), // Normalized (F2, F1) coordinate [0.0 ..= 1.0]
    pub trajectory_mode: FormantTrajectoryMode,
    pub trajectory_speed_hz: f32, // 0.1 ..= 10.0 Hz
    pub vocal_tract_scale: f32,   // 0.6x (child) .. 1.0x (neutral) .. 1.4x (deep male)
    pub resonance_q: f32,         // 1.0 ..= 25.0
    pub peak_boost_db: f32,       // 0.0 ..= +18.0 dB
    pub drive_warmth_db: f32,     // 0.0 ..= +12.0 dB
    pub dry_wet_pct: f32,         // 0.0 ..= 100.0%
    pub is_dragging_puck: bool,
    pub color_palette: ContrastColorPalette,
}

impl Default for FormantFilterView {
    fn default() -> Self {
        Self::new()
    }
}

impl FormantFilterView {
    pub fn new() -> Self {
        Self {
            active_vowel_idx: 0,
            morph_pos: (0.40, 0.85), // Default to /a/
            trajectory_mode: FormantTrajectoryMode::Static,
            trajectory_speed_hz: 1.0,
            vocal_tract_scale: 1.0,
            resonance_q: 8.0,
            peak_boost_db: 9.0,
            drive_warmth_db: 3.0,
            dry_wet_pct: 100.0,
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Interpolate formant frequencies based on current 2D morph position.
    pub fn calculate_interpolated_formants(&self) -> [f32; NUM_FORMANTS] {
        let mut weights = [0.0_f32; 5];
        let mut total_weight = 0.0_f32;

        for (i, v) in STANDARD_VOWELS.iter().enumerate() {
            let dx = self.morph_pos.0 - v.pad_norm_pos.0;
            let dy = self.morph_pos.1 - v.pad_norm_pos.1;
            let dist = (dx * dx + dy * dy).sqrt().max(0.01);
            let w = 1.0 / (dist * dist);
            weights[i] = w;
            total_weight += w;
        }

        let mut formants = [0.0_f32; NUM_FORMANTS];
        for i in 0..5 {
            let norm_w = weights[i] / total_weight;
            let v = &STANDARD_VOWELS[i];
            formants[0] += v.f1_hz * norm_w;
            formants[1] += v.f2_hz * norm_w;
            formants[2] += v.f3_hz * norm_w;
            formants[3] += v.f4_hz * norm_w;
            formants[4] += v.f5_hz * norm_w;
        }

        // Apply vocal tract length scaling
        let tract_inv = 1.0 / self.vocal_tract_scale.clamp(0.5, 2.0);
        for f in &mut formants {
            *f *= tract_inv;
        }

        formants
    }

    /// Evaluates frequency response gain (dB) across spectrum [50 Hz to 10000 Hz].
    pub fn evaluate_frequency_response(&self, freq_hz: f32) -> f32 {
        let formants = self.calculate_interpolated_formants();
        let mut total_gain_lin = 0.02_f32; // base passband floor

        for f_center in formants {
            let bw = (f_center / self.resonance_q).max(20.0);
            let delta = (freq_hz - f_center).abs();
            let bell = (-0.5 * (delta / bw).powi(2)).exp();
            let peak_gain_lin = 10.0_f32.powf(self.peak_boost_db / 20.0) - 1.0;
            total_gain_lin += bell * peak_gain_lin;
        }

        20.0 * total_gain_lin.clamp(0.001, 100.0).log10()
    }

    /// Tests if a screen coordinate hits the 2D Vowel Morph Puck (>= 22pt radius -> 44x44pt).
    pub fn hit_test_vowel_puck(&self, pos: (f32, f32), canvas: Rect) -> bool {
        let px = canvas.x + self.morph_pos.0 * canvas.width;
        let py = canvas.y + (1.0 - self.morph_pos.1) * canvas.height;
        let dx = pos.0 - px;
        let dy = pos.1 - py;
        (dx * dx + dy * dy).sqrt() <= FORMANT_PUCK_HIT_RADIUS
    }

    /// Render deterministic ASCII representation.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let formants = self.calculate_interpolated_formants();
        let header = format!(
            "FORMANT FILTER Traj:{:?} F1:{:.0}Hz F2:{:.0}Hz F3:{:.0}Hz Q:{:.1} Tract:{:.2}x",
            self.trajectory_mode,
            formants[0],
            formants[1],
            formants[2],
            self.resonance_q,
            self.vocal_tract_scale
        );
        lines.push(header);

        for y in 1..height {
            let mut row = String::with_capacity(width);
            let target_db = 18.0 - (y as f32 / height as f32) * 36.0; // [+18dB .. -18dB]
            for x in 0..width {
                let norm_f = x as f32 / width as f32;
                let freq = 50.0 * (10000.0 / 50.0_f32).powf(norm_f);
                let resp_db = self.evaluate_frequency_response(freq);
                if (resp_db - target_db).abs() < (36.0 / height as f32) {
                    row.push('#');
                } else if target_db.abs() < 1.0 {
                    row.push('-');
                } else {
                    row.push('.');
                }
            }
            lines.push(row);
        }
        lines
    }

    #[cfg(feature = "gui")]
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // 1. Top Header Bar & Vowel Preset Selectors
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("5-VOWEL PHONETIC FORMANT RESONATOR")
                        .size(15.0)
                        .color(Color32::from_rgb(255, 215, 0))
                        .strong(),
                );
                ui.separator();

                for (idx, v) in STANDARD_VOWELS.iter().enumerate() {
                    let is_active = self.active_vowel_idx == idx;
                    let btn = egui::Button::new(
                        egui::RichText::new(format!("{} {}", v.ipa, v.name))
                            .color(if is_active {
                                Color32::from_rgb(10, 14, 22)
                            } else {
                                Color32::from_rgb(240, 245, 255)
                            })
                            .strong(),
                    )
                    .min_size(Vec2::new(72.0, MIN_HIT_TARGET_PT))
                    .fill(if is_active {
                        Color32::from_rgb(255, 215, 0)
                    } else {
                        Color32::from_rgb(32, 45, 66)
                    });

                    if ui.add(btn).clicked() {
                        self.active_vowel_idx = idx;
                        self.morph_pos = v.pad_norm_pos;
                    }
                }

                ui.separator();
                let trajs = [
                    (FormantTrajectoryMode::Static, "STATIC"),
                    (FormantTrajectoryMode::CircularLfo, "CIRCLE LFO"),
                    (FormantTrajectoryMode::Figure8Lfo, "FIG-8"),
                ];
                for (t_mode, lbl) in trajs {
                    let is_act = self.trajectory_mode == t_mode;
                    let btn = egui::Button::new(egui::RichText::new(lbl).color(if is_act {
                        Color32::from_rgb(10, 14, 22)
                    } else {
                        Color32::from_rgb(200, 220, 245)
                    }))
                    .min_size(Vec2::new(60.0, MIN_HIT_TARGET_PT))
                    .fill(if is_act {
                        Color32::from_rgb(0, 229, 255)
                    } else {
                        Color32::from_rgb(24, 32, 48)
                    });
                    if ui.add(btn).clicked() {
                        self.trajectory_mode = t_mode;
                    }
                }
            });

            ui.add_space(8.0);

            // 2. Dual Canvas: Left (2D Vowel Morph Pad) & Right (Frequency Response Resonator Spectrum)
            ui.horizontal(|ui| {
                let half_width = (ui.available_width() - 16.0) * 0.5;

                // Left Canvas: 2D Vowel Morph Pad
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("2D VOWEL PHONETIC MORPH PAD (F1 vs F2)")
                            .color(Color32::from_rgb(0, 229, 255))
                            .strong(),
                    );
                    let (res_l, painter_l) = ui.allocate_painter(
                        Vec2::new(half_width.max(150.0), 210.0),
                        egui::Sense::click_and_drag(),
                    );
                    let rect_l = res_l.rect;

                    painter_l.rect_filled(rect_l, 6.0, Color32::from_rgb(14, 18, 28));
                    painter_l.rect_stroke(
                        rect_l,
                        6.0,
                        Stroke::new(1.5_f32, Color32::from_rgb(45, 60, 85)),
                    );

                    // Draw reference vowel landmark points
                    for v in &STANDARD_VOWELS {
                        let vx = rect_l.min.x + v.pad_norm_pos.0 * rect_l.width();
                        let vy = rect_l.min.y + (1.0 - v.pad_norm_pos.1) * rect_l.height();
                        painter_l.circle_filled(
                            egui::pos2(vx, vy),
                            6.0,
                            Color32::from_rgb(255, 215, 0),
                        );
                        painter_l.text(
                            egui::pos2(vx + 8.0, vy - 6.0),
                            egui::Align2::LEFT_TOP,
                            format!("{} ({})", v.ipa, v.name.split(' ').next().unwrap_or("")),
                            egui::FontId::monospace(11.0),
                            Color32::from_rgb(220, 235, 255),
                        );
                    }

                    // Morph Puck
                    let px = rect_l.min.x + self.morph_pos.0 * rect_l.width();
                    let py = rect_l.min.y + (1.0 - self.morph_pos.1) * rect_l.height();

                    // Puck hit target ring (>=22pt radius -> 44x44pt touch bounding box)
                    painter_l.circle_stroke(
                        egui::pos2(px, py),
                        FORMANT_PUCK_HIT_RADIUS,
                        Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 140)),
                    );
                    painter_l.circle_filled(
                        egui::pos2(px, py),
                        14.0,
                        Color32::from_rgb(0, 229, 255),
                    );
                    painter_l.circle_filled(
                        egui::pos2(px, py),
                        4.0,
                        Color32::from_rgb(255, 255, 255),
                    );

                    // Dragging
                    if res_l.dragged() || res_l.clicked() {
                        if let Some(pos) = res_l.interact_pointer_pos() {
                            let nx = ((pos.x - rect_l.min.x) / rect_l.width()).clamp(0.0, 1.0);
                            let ny =
                                1.0 - ((pos.y - rect_l.min.y) / rect_l.height()).clamp(0.0, 1.0);
                            self.morph_pos = (nx, ny);
                        }
                    }
                });

                // Right Canvas: Resonator Spectrum Response
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("5-FORMANT RESONANCE SPECTRUM")
                            .color(Color32::from_rgb(255, 107, 43))
                            .strong(),
                    );
                    let (res_r, painter_r) = ui.allocate_painter(
                        Vec2::new(half_width.max(150.0), 210.0),
                        egui::Sense::hover(),
                    );
                    let rect_r = res_r.rect;

                    painter_r.rect_filled(rect_r, 6.0, Color32::from_rgb(10, 14, 22));
                    painter_r.rect_stroke(
                        rect_r,
                        6.0,
                        Stroke::new(1.5_f32, Color32::from_rgb(45, 60, 85)),
                    );

                    // Frequency grid lines (100Hz, 1kHz, 5kHz)
                    for f in [100.0, 500.0, 1000.0, 2500.0, 5000.0, 10000.0] {
                        let norm_f = (f / 50.0_f32).log10() / (10000.0 / 50.0_f32).log10();
                        let sx = rect_r.min.x + norm_f * rect_r.width();
                        painter_r.line_segment(
                            [egui::pos2(sx, rect_r.min.y), egui::pos2(sx, rect_r.max.y)],
                            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 115, 60)),
                        );
                    }

                    // Spectrum Response Curve
                    let mut prev_pt: Option<egui::Pos2> = None;
                    for i in 0..60 {
                        let norm_x = i as f32 / 59.0;
                        let freq = 50.0 * (10000.0 / 50.0_f32).powf(norm_x);
                        let gain_db = self.evaluate_frequency_response(freq);
                        let norm_y = ((gain_db + 18.0) / 36.0).clamp(0.0, 1.0);
                        let sx = rect_r.min.x + norm_x * rect_r.width();
                        let sy = rect_r.max.y - norm_y * rect_r.height();
                        let pt = egui::pos2(sx, sy);
                        if let Some(prev) = prev_pt {
                            painter_r.line_segment(
                                [prev, pt],
                                Stroke::new(2.5_f32, Color32::from_rgb(255, 215, 0)),
                            );
                        }
                        prev_pt = Some(pt);
                    }
                });
            });

            ui.add_space(8.0);

            // 3. Tactile Sliders Bar (>=44pt Touch Targets)
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Resonance (Q)").strong());
                        ui.add(egui::Slider::new(&mut self.resonance_q, 1.0..=25.0).text("Q"));
                    });

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Peak Boost").strong());
                        ui.add(egui::Slider::new(&mut self.peak_boost_db, 0.0..=18.0).text("dB"));
                    });

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Vocal Tract Length").strong());
                        ui.add(
                            egui::Slider::new(&mut self.vocal_tract_scale, 0.6..=1.4).text("scale"),
                        );
                    });

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Drive Warmth").strong());
                        ui.add(egui::Slider::new(&mut self.drive_warmth_db, 0.0..=12.0).text("dB"));
                    });

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Dry / Wet").strong());
                        ui.add(egui::Slider::new(&mut self.dry_wet_pct, 0.0..=100.0).text("%"));
                    });
                });
            });
        });
    }
}
