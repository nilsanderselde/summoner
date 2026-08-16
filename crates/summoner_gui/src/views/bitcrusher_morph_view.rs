// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Tactile Bitcrusher Morphology Canvas with Downsampling Anti-Aliasing Curve & Quantization Jitter HUD (Step 1422).

use crate::layout_math::Rect;
use crate::touch_controls::{ContrastColorPalette, MIN_HIT_TARGET_PT};

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const BITCRUSHER_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box

/// Quantization morphology curve mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MorphQuantizeMode {
    Linear,
    CompandedMuLaw,
    CompandedALaw,
    ChaoticFoldback,
}

/// Tactile Bitcrusher Morphology View (Step 1422).
#[derive(Debug, Clone)]
pub struct BitcrusherMorphView {
    pub bit_depth: f32,            // 1.0 ..= 24.0 bits
    pub downsample_ratio: f32,     // 1.0 ..= 64.0x
    pub jitter_pct: f32,           // 0.0 ..= 100.0%
    pub anti_alias_cutoff_hz: f32, // 200.0 ..= 20000.0 Hz
    pub pre_drive_db: f32,         // -12.0 ..= +24.0 dB
    pub mode: MorphQuantizeMode,
    pub mix_pct: f32,         // 0.0 ..= 100.0%
    pub puck_pos: (f32, f32), // Normalized X (bit depth), Y (downsample)
    pub is_dragging_puck: bool,
    pub color_palette: ContrastColorPalette,
}

impl Default for BitcrusherMorphView {
    fn default() -> Self {
        Self::new()
    }
}

impl BitcrusherMorphView {
    pub fn new() -> Self {
        Self {
            bit_depth: 6.5,
            downsample_ratio: 8.0,
            jitter_pct: 12.0,
            anti_alias_cutoff_hz: 8000.0,
            pre_drive_db: 4.5,
            mode: MorphQuantizeMode::Linear,
            mix_pct: 100.0,
            puck_pos: (0.24, 0.12),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Quantize a normalized sample [-1.0 ..= 1.0] given bit depth and quantization mode.
    pub fn quantize_sample(
        sample: f32,
        bit_depth: f32,
        mode: MorphQuantizeMode,
        jitter: f32,
    ) -> f32 {
        let clamped = sample.clamp(-1.0, 1.0);
        let levels = 2.0_f32.powf(bit_depth.clamp(1.0, 24.0));
        let half_levels = levels * 0.5;

        // Apply pseudo-deterministic dither/jitter modulation
        let dither = (sample * 1337.1337).sin() * (jitter / 100.0) * (1.0 / levels);
        let dithered = (clamped + dither).clamp(-1.0, 1.0);

        let out = match mode {
            MorphQuantizeMode::Linear => (dithered * half_levels).round() / half_levels,
            MorphQuantizeMode::CompandedMuLaw => {
                // ITU-T G.711 mu-law companding: sgn(x)*ln(1+mu*|x|)/ln(1+mu), mu=255
                let mu = 255.0_f32;
                let sign = if dithered >= 0.0 { 1.0 } else { -1.0 };
                let compressed = sign * (1.0 + mu * dithered.abs()).ln() / (1.0 + mu).ln();
                let quantized = (compressed * half_levels).round() / half_levels;
                let q_sign = if quantized >= 0.0 { 1.0 } else { -1.0 };
                q_sign * ((1.0 + mu).powf(quantized.abs()) - 1.0) / mu
            }
            MorphQuantizeMode::CompandedALaw => {
                // A-law companding (A=87.6)
                let a = 87.6_f32;
                let sign = if dithered >= 0.0 { 1.0 } else { -1.0 };
                let abs_x = dithered.abs();
                let compressed = if abs_x < 1.0 / a {
                    sign * (a * abs_x) / (1.0 + a.ln())
                } else {
                    sign * (1.0 + (a * abs_x).ln()) / (1.0 + a.ln())
                };
                let quantized = (compressed * half_levels).round() / half_levels;
                let q_sign = if quantized >= 0.0 { 1.0 } else { -1.0 };
                let q_abs = quantized.abs();
                let expanded = if q_abs < 1.0 / (1.0 + a.ln()) {
                    q_abs * (1.0 + a.ln()) / a
                } else {
                    ((q_abs * (1.0 + a.ln()) - 1.0).exp()) / a
                };
                q_sign * expanded
            }
            MorphQuantizeMode::ChaoticFoldback => {
                let scaled = dithered * half_levels;
                (scaled.sin() * half_levels).round() / half_levels
            }
        };
        out.clamp(-1.0, 1.0)
    }

    /// Evaluates transfer curve points across [-1.0 ..= 1.0].
    pub fn calculate_transfer_curve(&self, num_points: usize) -> Vec<(f32, f32)> {
        let mut curve = Vec::with_capacity(num_points);
        let drive = 10.0_f32.powf(self.pre_drive_db / 20.0);
        for i in 0..num_points {
            let x = (i as f32 / (num_points - 1) as f32) * 2.0 - 1.0;
            let driven_x = (x * drive).clamp(-1.0, 1.0);
            let y = Self::quantize_sample(driven_x, self.bit_depth, self.mode, self.jitter_pct);
            curve.push((x, y));
        }
        curve
    }

    /// Tests if a screen coordinate hits the 2D Morph Puck (>= 22pt radius -> 44x44pt).
    pub fn hit_test_morph_puck(&self, pos: (f32, f32), canvas: Rect) -> bool {
        let px = canvas.x + self.puck_pos.0 * canvas.width;
        let py = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = pos.0 - px;
        let dy = pos.1 - py;
        (dx * dx + dy * dy).sqrt() <= BITCRUSHER_PUCK_HIT_RADIUS
    }

    /// Render deterministic ASCII representation.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let header = format!(
            "BITCRUSHER [{:?}] Bits:{:.1} Downsample:{:.1}x Jitter:{:.0}% Drive:+{:.1}dB",
            self.mode, self.bit_depth, self.downsample_ratio, self.jitter_pct, self.pre_drive_db
        );
        lines.push(header);

        let curve = self.calculate_transfer_curve(width);
        for y in 1..height {
            let mut row = String::with_capacity(width);
            let target_y = 1.0 - (y as f32 / height as f32) * 2.0; // [-1.0 ..= 1.0]
            for pt in curve.iter().take(width) {
                let curve_y = pt.1;
                if (curve_y - target_y).abs() < (2.0 / height as f32) {
                    row.push('#');
                } else if target_y.abs() < 0.05 {
                    row.push('-');
                } else if (pt.0).abs() < 0.03 {
                    row.push('|');
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
            // 1. Top Header Bar & Mode Selector
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("TACTILE BITCRUSHER & MORPHOLOGY HUD")
                        .size(15.0)
                        .color(Color32::from_rgb(255, 107, 43))
                        .strong(),
                );
                ui.separator();

                let modes = [
                    (MorphQuantizeMode::Linear, "LINEAR"),
                    (MorphQuantizeMode::CompandedMuLaw, "μ-LAW LOG"),
                    (MorphQuantizeMode::CompandedALaw, "A-LAW LOG"),
                    (MorphQuantizeMode::ChaoticFoldback, "CHAOTIC FOLDBACK"),
                ];

                for (m, lbl) in modes {
                    let is_active = self.mode == m;
                    let btn = egui::Button::new(
                        egui::RichText::new(lbl)
                            .color(if is_active {
                                Color32::from_rgb(10, 14, 22)
                            } else {
                                Color32::from_rgb(240, 245, 255)
                            })
                            .strong(),
                    )
                    .min_size(Vec2::new(90.0, MIN_HIT_TARGET_PT))
                    .fill(if is_active {
                        Color32::from_rgb(255, 215, 0)
                    } else {
                        Color32::from_rgb(32, 45, 66)
                    });

                    if ui.add(btn).clicked() {
                        self.mode = m;
                    }
                }
            });

            ui.add_space(8.0);

            // 2. Dual Canvas: Left (Transfer Curve) & Right (2D Morph XY Pad)
            ui.horizontal(|ui| {
                let half_width = (ui.available_width() - 16.0) * 0.5;

                // Left Canvas: Transfer Function
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("QUANTIZATION STAIRCASE TRANSFER")
                            .color(Color32::from_rgb(0, 229, 255))
                            .strong(),
                    );
                    let (res_l, painter_l) = ui.allocate_painter(
                        Vec2::new(half_width.max(150.0), 210.0),
                        egui::Sense::hover(),
                    );
                    let rect_l = res_l.rect;

                    painter_l.rect_filled(rect_l, 6.0, Color32::from_rgb(10, 14, 22));
                    painter_l.rect_stroke(
                        rect_l,
                        6.0,
                        Stroke::new(1.5_f32, Color32::from_rgb(45, 60, 85)),
                    );

                    // Grid & Axes
                    let cx = rect_l.min.x + rect_l.width() * 0.5;
                    let cy = rect_l.min.y + rect_l.height() * 0.5;
                    painter_l.line_segment(
                        [egui::pos2(rect_l.min.x, cy), egui::pos2(rect_l.max.x, cy)],
                        Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 115, 100)),
                    );
                    painter_l.line_segment(
                        [egui::pos2(cx, rect_l.min.y), egui::pos2(cx, rect_l.max.y)],
                        Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 115, 100)),
                    );

                    // Draw Staircase / Transfer Curve
                    let curve = self.calculate_transfer_curve(64);
                    let mut prev_pt: Option<egui::Pos2> = None;
                    for (x, y) in curve {
                        let sx = cx + x * (rect_l.width() * 0.45);
                        let sy = cy - y * (rect_l.height() * 0.45);
                        let pt = egui::pos2(sx, sy);
                        if let Some(prev) = prev_pt {
                            painter_l.line_segment(
                                [prev, pt],
                                Stroke::new(2.5_f32, Color32::from_rgb(255, 107, 43)),
                            );
                        }
                        prev_pt = Some(pt);
                    }
                });

                // Right Canvas: 2D Morph XY Pad
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("2D MORPH XY PAD (Bits vs Downsample)")
                            .color(Color32::from_rgb(0, 255, 180))
                            .strong(),
                    );
                    let (res_r, painter_r) = ui.allocate_painter(
                        Vec2::new(half_width.max(150.0), 210.0),
                        egui::Sense::click_and_drag(),
                    );
                    let rect_r = res_r.rect;

                    painter_r.rect_filled(rect_r, 6.0, Color32::from_rgb(14, 18, 28));
                    painter_r.rect_stroke(
                        rect_r,
                        6.0,
                        Stroke::new(1.5_f32, Color32::from_rgb(45, 60, 85)),
                    );

                    // 4x4 Grid
                    for g in 1..4 {
                        let gx = rect_r.min.x + rect_r.width() * (g as f32 * 0.25);
                        let gy = rect_r.min.y + rect_r.height() * (g as f32 * 0.25);
                        painter_r.line_segment(
                            [egui::pos2(gx, rect_r.min.y), egui::pos2(gx, rect_r.max.y)],
                            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(50, 65, 90, 80)),
                        );
                        painter_r.line_segment(
                            [egui::pos2(rect_r.min.x, gy), egui::pos2(rect_r.max.x, gy)],
                            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(50, 65, 90, 80)),
                        );
                    }

                    // Puck Position
                    let puck_x = rect_r.min.x + self.puck_pos.0 * rect_r.width();
                    let puck_y = rect_r.min.y + (1.0 - self.puck_pos.1) * rect_r.height();

                    // Crosshairs
                    painter_r.line_segment(
                        [
                            egui::pos2(rect_r.min.x, puck_y),
                            egui::pos2(rect_r.max.x, puck_y),
                        ],
                        Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 120)),
                    );
                    painter_r.line_segment(
                        [
                            egui::pos2(puck_x, rect_r.min.y),
                            egui::pos2(puck_x, rect_r.max.y),
                        ],
                        Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 120)),
                    );

                    // Outer Touch Hit Target (>=22pt radius -> 44x44pt bounding box)
                    painter_r.circle_stroke(
                        egui::pos2(puck_x, puck_y),
                        BITCRUSHER_PUCK_HIT_RADIUS,
                        Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 140)),
                    );
                    // Visual Puck Body
                    painter_r.circle_filled(
                        egui::pos2(puck_x, puck_y),
                        14.0,
                        Color32::from_rgb(0, 229, 255),
                    );
                    painter_r.circle_filled(
                        egui::pos2(puck_x, puck_y),
                        4.0,
                        Color32::from_rgb(255, 255, 255),
                    );

                    // Handle Dragging
                    if res_r.dragged() || res_r.clicked() {
                        if let Some(pos) = res_r.interact_pointer_pos() {
                            let nx = ((pos.x - rect_r.min.x) / rect_r.width()).clamp(0.0, 1.0);
                            let ny =
                                1.0 - ((pos.y - rect_r.min.y) / rect_r.height()).clamp(0.0, 1.0);
                            self.puck_pos = (nx, ny);
                            self.bit_depth = 1.0 + nx * 23.0; // 1.0 to 24.0 bits
                            self.downsample_ratio = 1.0 + (1.0 - ny) * 63.0; // 1.0 to 64.0x
                        }
                    }
                });
            });

            ui.add_space(8.0);

            // 3. Tactile Sliders Bar (>=44pt Touch Targets)
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Bit Depth").strong());
                        if ui
                            .add(egui::Slider::new(&mut self.bit_depth, 1.0..=24.0).text("bits"))
                            .changed()
                        {
                            self.puck_pos.0 = ((self.bit_depth - 1.0) / 23.0).clamp(0.0, 1.0);
                        }
                    });

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Downsampling").strong());
                        if ui
                            .add(
                                egui::Slider::new(&mut self.downsample_ratio, 1.0..=64.0).text("x"),
                            )
                            .changed()
                        {
                            self.puck_pos.1 =
                                (1.0 - (self.downsample_ratio - 1.0) / 63.0).clamp(0.0, 1.0);
                        }
                    });

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Jitter / Dither").strong());
                        ui.add(egui::Slider::new(&mut self.jitter_pct, 0.0..=100.0).text("%"));
                    });

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Anti-Alias Filter").strong());
                        ui.add(
                            egui::Slider::new(&mut self.anti_alias_cutoff_hz, 200.0..=20000.0)
                                .text("Hz"),
                        );
                    });

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Pre-Drive").strong());
                        ui.add(egui::Slider::new(&mut self.pre_drive_db, -12.0..=24.0).text("dB"));
                    });

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Mix").strong());
                        ui.add(egui::Slider::new(&mut self.mix_pct, 0.0..=100.0).text("%"));
                    });
                });
            });
        });
    }
}
