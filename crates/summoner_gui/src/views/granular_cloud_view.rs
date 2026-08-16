// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Touch-Responsive Granular Synthesis Cloud Grain Dispersion Canvas (Step 1383).

use crate::layout_math::Rect;
use crate::touch_controls::{ContrastColorPalette, MIN_HIT_TARGET_PT};

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const EMITTER_PUCK_VISUAL_RADIUS: f32 = 14.0;
pub const EMITTER_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box

/// Grain envelope window shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrainWindowShape {
    Hanning,
    Blackman,
    Gaussian,
    Trapezoid,
    ExponentialDecay,
}

impl GrainWindowShape {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Hanning => "Hanning (Smooth)",
            Self::Blackman => "Blackman (Clean)",
            Self::Gaussian => "Gaussian (Warm)",
            Self::Trapezoid => "Trapezoid (Punch)",
            Self::ExponentialDecay => "Exp Decay (Percussive)",
        }
    }

    /// Compute window amplitude at normalized phase [0.0 ..= 1.0].
    pub fn evaluate(&self, phase: f32) -> f32 {
        let p = phase.clamp(0.0, 1.0);
        match self {
            Self::Hanning => 0.5 * (1.0 - (p * std::f32::consts::TAU).cos()),
            Self::Blackman => {
                0.42 - 0.5 * (p * std::f32::consts::TAU).cos()
                    + 0.08 * (2.0 * p * std::f32::consts::TAU).cos()
            }
            Self::Gaussian => {
                let sigma = 0.2;
                (-((p - 0.5) / sigma).powi(2)).exp()
            }
            Self::Trapezoid => {
                if p < 0.15 {
                    p / 0.15
                } else if p > 0.85 {
                    (1.0 - p) / 0.15
                } else {
                    1.0
                }
            }
            Self::ExponentialDecay => (-p * 5.0).exp(),
        }
    }
}

/// A single simulated grain particle for real-time visualization.
#[derive(Debug, Clone, PartialEq)]
pub struct GrainParticle {
    pub pos_norm: f32,        // 0.0 ..= 1.0 (X-axis)
    pub pitch_semitones: f32, // -24.0 ..= +24.0 (Y-axis)
    pub duration_ms: f32,
    pub age_ms: f32,
    pub pan: f32,       // -1.0 ..= +1.0
    pub amplitude: f32, // 0.0 ..= 1.0
    pub is_reverse: bool,
}

/// Touch-Responsive Granular Synthesis Cloud Dispersion Canvas View (Step 1383).
#[derive(Debug, Clone)]
pub struct GranularCloudView {
    pub emitter_pos_norm: f32,        // 0.0 ..= 1.0 (X-axis)
    pub emitter_pitch_semitones: f32, // -24.0 ..= +24.0 (Y-axis)
    pub spray_width_norm: f32,        // 0.0 ..= 0.5 (X jitter)
    pub spray_height_semitones: f32,  // 0.0 ..= 12.0 (Pitch jitter)
    pub grain_rate_hz: f32,           // 1.0 ..= 200.0 Hz
    pub grain_size_ms: f32,           // 5.0 ..= 500.0 ms
    pub density: u32,                 // 1 ..= 64 simultaneous grains
    pub pan_spread_pct: f32,          // 0.0 ..= 100.0%
    pub reverse_probability_pct: f32, // 0.0 ..= 100.0%
    pub freeze_buffer: bool,
    pub window_shape: GrainWindowShape,
    pub active_grains: Vec<GrainParticle>,
    pub is_dragging_emitter: bool,
    pub color_palette: ContrastColorPalette,
}

impl Default for GranularCloudView {
    fn default() -> Self {
        Self::new()
    }
}

impl GranularCloudView {
    pub fn new() -> Self {
        let mut view = Self {
            emitter_pos_norm: 0.45,
            emitter_pitch_semitones: 0.0,
            spray_width_norm: 0.15,
            spray_height_semitones: 3.5,
            grain_rate_hz: 35.0,
            grain_size_ms: 80.0,
            density: 16,
            pan_spread_pct: 50.0,
            reverse_probability_pct: 10.0,
            freeze_buffer: false,
            window_shape: GrainWindowShape::Hanning,
            active_grains: Vec::new(),
            is_dragging_emitter: false,
            color_palette: ContrastColorPalette::default(),
        };

        // Populate initial grain particles
        view.spawn_sample_grains();
        view
    }

    /// Populate simulated grain particles around emitter.
    pub fn spawn_sample_grains(&mut self) {
        self.active_grains.clear();
        for i in 0..self.density {
            let t = i as f32 / self.density.max(1) as f32;
            let jitter_x = ((t * 17.3).sin() * self.spray_width_norm).clamp(-0.4, 0.4);
            let jitter_y = ((t * 29.7).cos() * self.spray_height_semitones).clamp(-12.0, 12.0);

            let pos = (self.emitter_pos_norm + jitter_x).clamp(0.0, 1.0);
            let pitch = (self.emitter_pitch_semitones + jitter_y).clamp(-24.0, 24.0);

            self.active_grains.push(GrainParticle {
                pos_norm: pos,
                pitch_semitones: pitch,
                duration_ms: self.grain_size_ms,
                age_ms: (t * self.grain_size_ms) % self.grain_size_ms,
                pan: (t * 2.0 - 1.0) * (self.pan_spread_pct / 100.0),
                amplitude: self.window_shape.evaluate(t),
                is_reverse: (i % 8) == 0,
            });
        }
    }

    /// Convert 2D coordinates to screen pixel position on canvas.
    pub fn cloud_coords_to_screen(
        &self,
        pos_norm: f32,
        pitch_semitones: f32,
        canvas: Rect,
    ) -> (f32, f32) {
        let sx = canvas.x + pos_norm.clamp(0.0, 1.0) * canvas.width;
        // Pitch: -24 at bottom (canvas.y + canvas.height), +24 at top (canvas.y)
        let norm_pitch = ((pitch_semitones + 24.0) / 48.0).clamp(0.0, 1.0);
        let sy = canvas.y + (1.0 - norm_pitch) * canvas.height;
        (sx, sy)
    }

    /// Convert screen pixel position to cloud coordinates (pos_norm, pitch_semitones).
    pub fn screen_to_cloud_coords(&self, pos: (f32, f32), canvas: Rect) -> (f32, f32) {
        if canvas.width <= 0.0 || canvas.height <= 0.0 {
            return (0.0, 0.0);
        }
        let norm_x = ((pos.0 - canvas.x) / canvas.width).clamp(0.0, 1.0);
        let norm_y = 1.0 - ((pos.1 - canvas.y) / canvas.height).clamp(0.0, 1.0);
        let pitch_st = -24.0 + norm_y * 48.0;
        (norm_x, pitch_st.clamp(-24.0, 24.0))
    }

    /// Hit-test emitter puck with ergonomic touch radius (>=44x44pt).
    pub fn hit_test_emitter(&self, pos: (f32, f32), canvas: Rect) -> bool {
        let (sx, sy) = self.cloud_coords_to_screen(
            self.emitter_pos_norm,
            self.emitter_pitch_semitones,
            canvas,
        );
        let dx = pos.0 - sx;
        let dy = pos.1 - sy;
        (dx * dx + dy * dy).sqrt() <= EMITTER_PUCK_HIT_RADIUS
    }

    /// Advance active grains by dt milliseconds.
    pub fn step_grains(&mut self, dt_ms: f32) {
        for grain in &mut self.active_grains {
            grain.age_ms += dt_ms;
            if grain.age_ms >= grain.duration_ms {
                grain.age_ms = 0.0;
            }
            let phase = (grain.age_ms / grain.duration_ms).clamp(0.0, 1.0);
            grain.amplitude = self.window_shape.evaluate(phase);
        }
    }

    /// Deterministic ASCII representation of the granular cloud dispersion.
    pub fn render_ascii(&self, width: usize, height: usize) -> String {
        let w = width.max(10);
        let h = height.max(5);
        let mut grid = vec![vec!['.'; w]; h];

        // Draw grains
        for grain in &self.active_grains {
            let gx = ((grain.pos_norm * (w - 1) as f32).round() as usize).min(w - 1);
            let norm_p = ((grain.pitch_semitones + 24.0) / 48.0).clamp(0.0, 1.0);
            let gy = (((1.0 - norm_p) * (h - 1) as f32).round() as usize).min(h - 1);
            grid[gy][gx] = '*';
        }

        // Draw emitter center
        let ex = ((self.emitter_pos_norm * (w - 1) as f32).round() as usize).min(w - 1);
        let norm_ep = ((self.emitter_pitch_semitones + 24.0) / 48.0).clamp(0.0, 1.0);
        let ey = (((1.0 - norm_ep) * (h - 1) as f32).round() as usize).min(h - 1);
        grid[ey][ex] = 'E';

        let mut lines = Vec::new();
        for row in grid {
            lines.push(row.into_iter().collect::<String>());
        }
        lines.join("\n")
    }
}

#[cfg(feature = "gui")]
impl GranularCloudView {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // 1. Header Toolbar
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("GRANULAR CLOUD SYNTHESIS EMITTER")
                        .color(Color32::from_rgb(240, 245, 255))
                        .strong(),
                );
                ui.separator();
                ui.label(
                    egui::RichText::new(format!(
                        "Pos: {:.1}% | Pitch: {:+.1} st | Density: {} grains",
                        self.emitter_pos_norm * 100.0,
                        self.emitter_pitch_semitones,
                        self.density
                    ))
                    .color(Color32::from_rgb(0, 229, 255))
                    .strong(),
                );
                ui.separator();
                ui.checkbox(&mut self.freeze_buffer, "FREEZE BUFFER");
            });

            ui.add_space(6.0);

            // 2. Grain Dispersion 2D Canvas
            let canvas_w = ui.available_width().max(680.0);
            let canvas_h = 220.0;
            let (response, painter) =
                ui.allocate_painter(Vec2::new(canvas_w, canvas_h), egui::Sense::click_and_drag());
            let canvas = Rect::new(
                response.rect.min.x,
                response.rect.min.y,
                response.rect.width(),
                response.rect.height(),
            );

            // Canvas Background
            painter.rect_filled(response.rect, 6.0_f32, Color32::from_rgb(10, 14, 22));
            painter.rect_stroke(
                response.rect,
                6.0_f32,
                Stroke::new(1.5_f32, Color32::from_rgb(40, 55, 80)),
            );

            // Pitch Grid Guides (-24st, -12st, 0st, +12st, +24st)
            let pitch_guides = [-24.0_f32, -12.0_f32, 0.0_f32, 12.0_f32, 24.0_f32];
            for p in pitch_guides {
                let (_, gy) = self.cloud_coords_to_screen(0.0_f32, p, canvas);
                painter.line_segment(
                    [
                        egui::pos2(canvas.x, gy),
                        egui::pos2(canvas.x + canvas.width, gy),
                    ],
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 110, 70)),
                );
                painter.text(
                    egui::pos2(canvas.x + 8.0_f32, gy - 6.0_f32),
                    egui::Align2::LEFT_BOTTOM,
                    format!("{:+.0} st", p),
                    egui::FontId::proportional(10.0_f32),
                    Color32::from_rgb(140, 165, 195),
                );
            }

            // Position Grid Guides (0%, 25%, 50%, 75%, 100%)
            for pos_pct in [0.25_f32, 0.50_f32, 0.75_f32] {
                let (gx, _) = self.cloud_coords_to_screen(pos_pct, 0.0_f32, canvas);
                painter.line_segment(
                    [
                        egui::pos2(gx, canvas.y),
                        egui::pos2(gx, canvas.y + canvas.height),
                    ],
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 110, 50)),
                );
            }

            // Draw Spray Dispersion Ellipse Bounds
            let (ex, ey) = self.cloud_coords_to_screen(
                self.emitter_pos_norm,
                self.emitter_pitch_semitones,
                canvas,
            );
            let spray_w_px = self.spray_width_norm * canvas.width;
            let spray_h_px = (self.spray_height_semitones / 48.0_f32) * canvas.height;

            let spray_rect = egui::Rect::from_center_size(
                egui::pos2(ex, ey),
                Vec2::new(spray_w_px * 2.0_f32, spray_h_px * 2.0_f32),
            );
            painter.rect_filled(
                spray_rect,
                12.0_f32,
                Color32::from_rgba_unmultiplied(0, 229, 255, 20),
            );
            painter.rect_stroke(
                spray_rect,
                12.0_f32,
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 90)),
            );

            // Draw Active Simulated Grains (Glowing particles)
            for grain in &self.active_grains {
                let (gx, gy) =
                    self.cloud_coords_to_screen(grain.pos_norm, grain.pitch_semitones, canvas);
                let alpha = (grain.amplitude * 220.0_f32) as u8;
                let grain_col = if grain.is_reverse {
                    Color32::from_rgba_unmultiplied(255, 107, 43, alpha)
                } else {
                    Color32::from_rgba_unmultiplied(0, 255, 180, alpha)
                };

                // Grain particle streak
                let streak_w = (grain.duration_ms / 500.0_f32) * 20.0_f32 + 4.0_f32;
                painter.line_segment(
                    [
                        egui::pos2(gx - streak_w * 0.5_f32, gy),
                        egui::pos2(gx + streak_w * 0.5_f32, gy),
                    ],
                    Stroke::new(2.5_f32, grain_col),
                );
                painter.circle_filled(
                    egui::pos2(gx, gy),
                    3.0_f32,
                    Color32::from_rgb(255, 255, 255),
                );
            }

            // Draw Emitter Puck (>=44x44pt Touch Target)
            let emitter_pos = egui::pos2(ex, ey);
            painter.circle_stroke(
                emitter_pos,
                EMITTER_PUCK_HIT_RADIUS,
                Stroke::new(1.5_f32, Color32::from_rgb(255, 215, 0)),
            );
            painter.circle_filled(
                emitter_pos,
                EMITTER_PUCK_VISUAL_RADIUS,
                Color32::from_rgb(0, 229, 255),
            );
            painter.circle_filled(emitter_pos, 4.0_f32, Color32::from_rgb(255, 255, 255));

            // Emitter Tag Readout
            painter.text(
                egui::pos2(ex, ey - 22.0_f32),
                egui::Align2::CENTER_BOTTOM,
                format!(
                    "{:.0}% | {:+.1}st",
                    self.emitter_pos_norm * 100.0,
                    self.emitter_pitch_semitones
                ),
                egui::FontId::proportional(11.0),
                Color32::from_rgb(255, 215, 0),
            );

            // Canvas Interaction Handling
            if response.drag_started() {
                if let Some(pos) = response.interact_pointer_pos() {
                    self.is_dragging_emitter = self.hit_test_emitter((pos.x, pos.y), canvas);
                    if !self.is_dragging_emitter {
                        let (norm_x, pitch_st) =
                            self.screen_to_cloud_coords((pos.x, pos.y), canvas);
                        self.emitter_pos_norm = norm_x;
                        self.emitter_pitch_semitones = pitch_st;
                        self.spawn_sample_grains();
                    }
                }
            }

            if response.dragged() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let (norm_x, pitch_st) = self.screen_to_cloud_coords((pos.x, pos.y), canvas);
                    self.emitter_pos_norm = norm_x;
                    self.emitter_pitch_semitones = pitch_st;
                    self.spawn_sample_grains();
                }
            }

            if response.drag_stopped() {
                self.is_dragging_emitter = false;
            }

            ui.add_space(8.0);

            // 3. Envelope Window Selectors (>=44pt Touch Targets)
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Window Envelope:").strong());
                let windows = [
                    GrainWindowShape::Hanning,
                    GrainWindowShape::Blackman,
                    GrainWindowShape::Gaussian,
                    GrainWindowShape::Trapezoid,
                    GrainWindowShape::ExponentialDecay,
                ];
                for w in windows {
                    let is_act = self.window_shape == w;
                    let btn = egui::Button::new(
                        egui::RichText::new(w.display_name())
                            .color(if is_act {
                                Color32::from_rgb(10, 14, 22)
                            } else {
                                Color32::from_rgb(220, 235, 255)
                            })
                            .strong(),
                    )
                    .min_size(Vec2::new(80.0, MIN_HIT_TARGET_PT))
                    .fill(if is_act {
                        Color32::from_rgb(0, 229, 255)
                    } else {
                        Color32::from_rgb(30, 40, 60)
                    });

                    if ui.add(btn).clicked() {
                        self.window_shape = w;
                        self.spawn_sample_grains();
                    }
                }
            });

            ui.add_space(8.0);

            // 4. Granular Cloud Control Sliders Bar
            ui.horizontal(|ui| {
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Grain Rate").strong());
                        ui.add(egui::Slider::new(&mut self.grain_rate_hz, 1.0..=200.0).text("Hz"));
                    });
                });

                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Grain Size").strong());
                        ui.add(egui::Slider::new(&mut self.grain_size_ms, 5.0..=500.0).text("ms"));
                    });
                });

                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Spray Width").strong());
                        ui.add(
                            egui::Slider::new(&mut self.spray_width_norm, 0.0..=0.5).text("Jitter"),
                        );
                    });
                });

                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Pitch Jitter").strong());
                        ui.add(
                            egui::Slider::new(&mut self.spray_height_semitones, 0.0..=12.0)
                                .text("st"),
                        );
                    });
                });

                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Pan Spread").strong());
                        ui.add(egui::Slider::new(&mut self.pan_spread_pct, 0.0..=100.0).text("%"));
                    });
                });
            });
        });
    }
}
