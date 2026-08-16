// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Psychoacoustic Stereo Widener & Haas Effect Delay HUD with Phase Vector Scope (Step 1403).

use crate::layout_math::Rect;
use crate::touch_controls::{ContrastColorPalette, MIN_HIT_TARGET_PT};

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const STEREO_CROSSOVER_HANDLE_HIT_RADIUS: f32 = 22.0; // 44x44pt touch area
pub const VECTOR_SCOPE_RADIUS_PT: f32 = 90.0;

/// Stereo spatial width processing mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StereoWidenerMode {
    MidSideBandWidth,
    HaasMicroDelay,
    FrequencySplitMultiband,
    BinauralHRTFSpatializer,
}

/// Dynamic Psychoacoustic Stereo Widener View (Step 1403).
#[derive(Debug, Clone)]
pub struct StereoWidenerView {
    pub mode: StereoWidenerMode,
    pub low_band_width_pct: f32, // 0.0 ..= 200.0% (default 0% = Mono Bass)
    pub mid_band_width_pct: f32, // 0.0 ..= 200.0% (default 100%)
    pub high_band_width_pct: f32, // 0.0 ..= 200.0% (default 140%)
    pub crossover_low_hz: f32,   // 60.0 ..= 400.0 Hz (default 180 Hz)
    pub crossover_high_hz: f32,  // 1000.0 ..= 10000.0 Hz (default 4000 Hz)
    pub haas_delay_ms: f32,      // 0.0 ..= 30.0 ms
    pub haas_channel_offset: f32, // -1.0 (Left earlier) ..= +1.0 (Right earlier)
    pub phase_correlation: f32,  // -1.0 (Out of phase) ..= +1.0 (Mono)
    pub balance_lr: f32,         // -1.0 ..= +1.0
    pub mono_bass_enabled: bool,
    pub mono_compatibility_check: bool,
    pub vector_scope_points: Vec<(f32, f32)>, // Lissajous normalized (L, R) points
    pub color_palette: ContrastColorPalette,
}

impl Default for StereoWidenerView {
    fn default() -> Self {
        Self::new()
    }
}

impl StereoWidenerView {
    pub fn new() -> Self {
        // Generate simulated Lissajous stereo phase cloud
        let mut points = Vec::with_capacity(64);
        for i in 0..64 {
            let t = i as f32 / 64.0 * std::f32::consts::TAU * 3.0;
            let l = (t * 1.5).sin() * 0.75 + (t * 4.2).cos() * 0.15;
            let r = (t * 1.5 + 0.35).sin() * 0.70 + (t * 4.2).sin() * 0.15;
            points.push((l, r));
        }

        Self {
            mode: StereoWidenerMode::FrequencySplitMultiband,
            low_band_width_pct: 0.0,    // Mono Bass
            mid_band_width_pct: 100.0,  // Standard Stereo
            high_band_width_pct: 140.0, // Expanded Air Width
            crossover_low_hz: 180.0,
            crossover_high_hz: 4000.0,
            haas_delay_ms: 8.5,
            haas_channel_offset: 0.5,
            phase_correlation: 0.82, // Healthy in-phase correlation
            balance_lr: 0.0,
            mono_bass_enabled: true,
            mono_compatibility_check: false,
            vector_scope_points: points,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Project stereo (Left, Right) audio sample to (X, Y) Vector Scope coordinates.
    /// In standard audio vector scope:
    ///   - M = (L + R) / sqrt(2) is vertical (Y)
    ///   - S = (L - R) / sqrt(2) is horizontal (X)
    pub fn project_vector_scope(l: f32, r: f32, center: (f32, f32), radius: f32) -> (f32, f32) {
        let inv_sqrt2 = 1.0 / std::f32::consts::SQRT_2;
        let side = (l - r) * inv_sqrt2;
        let mid = (l + r) * inv_sqrt2;
        let sx = center.0 + side.clamp(-1.2, 1.2) * radius;
        let sy = center.1 - mid.clamp(-1.2, 1.2) * radius;
        (sx, sy)
    }

    /// Convert frequency (20Hz - 20kHz) to normalized position [0.0 ..= 1.0].
    pub fn freq_to_norm_x(freq_hz: f32) -> f32 {
        let log_min = 20.0_f32.log10();
        let log_max = 20000.0_f32.log10();
        ((freq_hz.clamp(20.0, 20000.0).log10() - log_min) / (log_max - log_min)).clamp(0.0, 1.0)
    }

    /// Convert normalized position to frequency in Hz.
    pub fn norm_x_to_freq(norm_x: f32) -> f32 {
        let log_min = 20.0_f32.log10();
        let log_max = 20000.0_f32.log10();
        let log_val = log_min + norm_x.clamp(0.0, 1.0) * (log_max - log_min);
        10.0_f32.powf(log_val).clamp(20.0, 20000.0)
    }

    /// Hit-test crossover split handle with ergonomic minimum hit target (>=44x44pt).
    pub fn hit_test_crossover_handle(&self, pos: (f32, f32), canvas: Rect) -> Option<usize> {
        let low_x = canvas.x + Self::freq_to_norm_x(self.crossover_low_hz) * canvas.width;
        if (pos.0 - low_x).abs() <= STEREO_CROSSOVER_HANDLE_HIT_RADIUS
            && (pos.1 >= canvas.y && pos.1 <= canvas.y + canvas.height)
        {
            return Some(0);
        }

        let high_x = canvas.x + Self::freq_to_norm_x(self.crossover_high_hz) * canvas.width;
        if (pos.0 - high_x).abs() <= STEREO_CROSSOVER_HANDLE_HIT_RADIUS
            && (pos.1 >= canvas.y && pos.1 <= canvas.y + canvas.height)
        {
            return Some(1);
        }

        None
    }

    /// Update phase correlation value.
    pub fn update_correlation(&mut self, correlation: f32) {
        self.phase_correlation = correlation.clamp(-1.0, 1.0);
    }

    /// Deterministic ASCII render of the 3-band stereo width layout.
    pub fn render_ascii(&self, width: usize) -> String {
        let mut buf = vec!['='; width];
        let low_pos = ((Self::freq_to_norm_x(self.crossover_low_hz) * (width - 1) as f32).round()
            as usize)
            .min(width - 1);
        let high_pos = ((Self::freq_to_norm_x(self.crossover_high_hz) * (width - 1) as f32).round()
            as usize)
            .min(width - 1);

        for (i, item) in buf.iter_mut().enumerate() {
            if i < low_pos {
                *item = if self.mono_bass_enabled { 'M' } else { 'L' };
            } else if i == low_pos || i == high_pos {
                *item = '|';
            } else if i < high_pos {
                *item = 'W';
            } else {
                *item = 'H';
            }
        }
        buf.into_iter().collect()
    }
}

#[cfg(feature = "gui")]
impl StereoWidenerView {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // 1. Header Bar with Phase Correlation Badge
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("PSYCHOACOUSTIC STEREO WIDENER & VECTOR SCOPE")
                        .color(Color32::from_rgb(240, 245, 255))
                        .strong(),
                );
                ui.separator();

                // Phase Correlation Readout Badge
                let corr_color = if self.phase_correlation >= 0.5 {
                    Color32::from_rgb(0, 255, 180) // Safe green
                } else if self.phase_correlation >= 0.0 {
                    Color32::from_rgb(255, 215, 0) // Wide stereo yellow
                } else {
                    Color32::from_rgb(255, 60, 80) // Out of phase red
                };

                let corr_status = if self.phase_correlation >= 0.5 {
                    "IN PHASE"
                } else if self.phase_correlation >= 0.0 {
                    "WIDE STEREO"
                } else {
                    "PHASE CANCEL"
                };

                ui.label(
                    egui::RichText::new(format!(
                        "Correlation: {:+.2} [{}]",
                        self.phase_correlation, corr_status
                    ))
                    .color(corr_color)
                    .strong(),
                );
                ui.separator();

                // Mono Compatibility Check Button (>=60x44pt)
                let mono_btn = egui::Button::new(
                    egui::RichText::new(if self.mono_compatibility_check {
                        "MONO AUDITION: ON"
                    } else {
                        "Mono Check: OFF"
                    })
                    .color(if self.mono_compatibility_check {
                        Color32::from_rgb(10, 14, 22)
                    } else {
                        Color32::from_rgb(220, 235, 255)
                    })
                    .strong(),
                )
                .min_size(Vec2::new(120.0, MIN_HIT_TARGET_PT))
                .fill(if self.mono_compatibility_check {
                    Color32::from_rgb(255, 215, 0)
                } else {
                    Color32::from_rgb(35, 45, 65)
                });

                if ui.add(mono_btn).clicked() {
                    self.mono_compatibility_check = !self.mono_compatibility_check;
                }
            });

            ui.add_space(6.0);

            // 2. Main Dual Panel: Vector Scope (Left) + Multi-Band Width HUD (Right)
            ui.horizontal(|ui| {
                // Left: Lissajous Phase Vector Scope
                let scope_size = Vec2::new(260.0, 260.0);
                let (scope_resp, painter) =
                    ui.allocate_painter(scope_size, egui::Sense::click_and_drag());
                let scope_center = egui::pos2(
                    scope_resp.rect.min.x + scope_size.x * 0.5,
                    scope_resp.rect.min.y + scope_size.y * 0.5,
                );

                // Scope Circular Background
                painter.rect_filled(scope_resp.rect, 8.0_f32, Color32::from_rgb(10, 14, 22));
                painter.rect_stroke(
                    scope_resp.rect,
                    8.0_f32,
                    Stroke::new(1.5_f32, Color32::from_rgb(45, 60, 85)),
                );

                // Polar Grid Circles (30, 60, 90 pt)
                for r in [30.0_f32, 60.0_f32, VECTOR_SCOPE_RADIUS_PT] {
                    painter.circle_stroke(
                        scope_center,
                        r,
                        Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(50, 70, 100, 80)),
                    );
                }

                // Mid/Side Axis (Vertical = Mono M, Horizontal = Side S)
                painter.line_segment(
                    [
                        egui::pos2(scope_center.x, scope_center.y - VECTOR_SCOPE_RADIUS_PT),
                        egui::pos2(scope_center.x, scope_center.y + VECTOR_SCOPE_RADIUS_PT),
                    ],
                    Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 120)),
                );
                painter.line_segment(
                    [
                        egui::pos2(scope_center.x - VECTOR_SCOPE_RADIUS_PT, scope_center.y),
                        egui::pos2(scope_center.x + VECTOR_SCOPE_RADIUS_PT, scope_center.y),
                    ],
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 107, 43, 100)),
                );

                // 45-degree L and R Diagonals
                let diag = VECTOR_SCOPE_RADIUS_PT * std::f32::consts::FRAC_1_SQRT_2;
                painter.line_segment(
                    [
                        egui::pos2(scope_center.x - diag, scope_center.y - diag),
                        egui::pos2(scope_center.x + diag, scope_center.y + diag),
                    ],
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 115, 80)),
                );
                painter.line_segment(
                    [
                        egui::pos2(scope_center.x + diag, scope_center.y - diag),
                        egui::pos2(scope_center.x - diag, scope_center.y + diag),
                    ],
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 115, 80)),
                );

                // Scope Labels
                painter.text(
                    egui::pos2(
                        scope_center.x,
                        scope_center.y - VECTOR_SCOPE_RADIUS_PT - 4.0,
                    ),
                    egui::Align2::CENTER_BOTTOM,
                    "+M (MONO)",
                    egui::FontId::proportional(10.0),
                    Color32::from_rgb(0, 229, 255),
                );
                painter.text(
                    egui::pos2(
                        scope_center.x + VECTOR_SCOPE_RADIUS_PT + 4.0,
                        scope_center.y,
                    ),
                    egui::Align2::LEFT_CENTER,
                    "+S (SIDE)",
                    egui::FontId::proportional(10.0),
                    Color32::from_rgb(255, 107, 43),
                );

                // Draw Lissajous Samples Trail
                let mut prev_pt: Option<egui::Pos2> = None;
                for &(l, r) in &self.vector_scope_points {
                    let (sx, sy) = Self::project_vector_scope(
                        l,
                        r,
                        (scope_center.x, scope_center.y),
                        VECTOR_SCOPE_RADIUS_PT * 0.85,
                    );
                    let pt = egui::pos2(sx, sy);
                    if let Some(prev) = prev_pt {
                        painter.line_segment(
                            [prev, pt],
                            Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(0, 255, 180, 180)),
                        );
                    }
                    painter.circle_filled(pt, 2.0_f32, Color32::from_rgb(255, 255, 255));
                    prev_pt = Some(pt);
                }

                ui.add_space(10.0);

                // Right: 3-Band Width HUD & Haas Delay Editor
                ui.vertical(|ui| {
                    ui.group(|ui| {
                        ui.label(
                            egui::RichText::new("3-BAND FREQUENCY WIDTH CONTROL")
                                .color(Color32::from_rgb(255, 215, 0))
                                .strong(),
                        );
                        ui.add_space(4.0);

                        // Low Band / Mono Bass
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut self.mono_bass_enabled, "Mono Bass Lock");
                            ui.label(format!("(< {:.0} Hz)", self.crossover_low_hz));
                            ui.add(
                                egui::Slider::new(&mut self.low_band_width_pct, 0.0..=200.0)
                                    .text("Low Width %"),
                            );
                        });

                        // Mid Band
                        ui.horizontal(|ui| {
                            ui.label(format!(
                                "Mid Width ({:.0} Hz - {:.0} Hz):",
                                self.crossover_low_hz, self.crossover_high_hz
                            ));
                            ui.add(
                                egui::Slider::new(&mut self.mid_band_width_pct, 0.0..=200.0)
                                    .text("%"),
                            );
                        });

                        // High Band (Air)
                        ui.horizontal(|ui| {
                            ui.label(format!(
                                "High / Air Width (> {:.0} Hz):",
                                self.crossover_high_hz
                            ));
                            ui.add(
                                egui::Slider::new(&mut self.high_band_width_pct, 0.0..=200.0)
                                    .text("%"),
                            );
                        });

                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.label("Low X-Over:");
                            ui.add(
                                egui::Slider::new(&mut self.crossover_low_hz, 60.0..=400.0)
                                    .text("Hz"),
                            );
                            ui.separator();
                            ui.label("High X-Over:");
                            ui.add(
                                egui::Slider::new(&mut self.crossover_high_hz, 1000.0..=10000.0)
                                    .text("Hz"),
                            );
                        });
                    });

                    ui.add_space(6.0);

                    // Haas Effect Micro-Delay Card
                    ui.group(|ui| {
                        ui.label(
                            egui::RichText::new("HAAS EFFECT PSYCHOACOUSTIC MICRO-DELAY")
                                .color(Color32::from_rgb(0, 229, 255))
                                .strong(),
                        );
                        ui.add_space(4.0);

                        ui.horizontal(|ui| {
                            ui.label("Delay Time:");
                            ui.add(
                                egui::Slider::new(&mut self.haas_delay_ms, 0.0..=30.0).text("ms"),
                            );

                            ui.separator();
                            ui.label("Channel Offset:");
                            ui.add(
                                egui::Slider::new(&mut self.haas_channel_offset, -1.0..=1.0)
                                    .text("L/R"),
                            );
                        });
                    });
                });
            });

            ui.add_space(8.0);

            // 3. Global Stereo Balance Bar
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Stereo Balance").strong());
                ui.add(egui::Slider::new(&mut self.balance_lr, -1.0..=1.0).text("L/R"));
            });
        });
    }
}
