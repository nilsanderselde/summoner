// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Interactive Multi-Algorithm Reverb Space & Early Reflection Ray-Tracer (Step 1404).

use crate::layout_math::Rect;
use crate::touch_controls::{ContrastColorPalette, MIN_HIT_TARGET_PT};

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const REVERB_OBJECT_HIT_RADIUS: f32 = 22.0; // 44x44pt touch area
pub const REVERB_OBJECT_VISUAL_RADIUS: f32 = 14.0;
pub const DAMPING_HANDLE_HIT_RADIUS: f32 = 22.0;

/// Reverb algorithm architecture presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReverbAlgorithm {
    PlateReverb,
    HallConcert,
    CathedralSpace,
    RoomChamber,
    ShimmerEthereal,
    NonLinearGate,
}

impl ReverbAlgorithm {
    pub fn name(&self) -> &'static str {
        match self {
            Self::PlateReverb => "Plate Reverb",
            Self::HallConcert => "Concert Hall",
            Self::CathedralSpace => "Cathedral",
            Self::RoomChamber => "Acoustic Chamber",
            Self::ShimmerEthereal => "Shimmer Ethereal",
            Self::NonLinearGate => "Non-Linear Gate",
        }
    }
}

/// Simulated geometric acoustic ray reflection path.
#[derive(Debug, Clone, PartialEq)]
pub struct AcousticRayPath {
    pub points: Vec<(f32, f32)>, // Normalized room coordinates [0.0 ..= 1.0]
    pub initial_angle_rad: f32,
    pub energy: f32, // 0.0 ..= 1.0
    pub order: usize,
}

/// Interactive Reverb Space View (Step 1404).
#[derive(Debug, Clone)]
pub struct ReverbSpaceView {
    pub algorithm: ReverbAlgorithm,
    pub room_size_m: f32,                   // 5.0 ..= 100.0 m
    pub decay_time_rt60_s: f32,             // 0.2 ..= 20.0 s
    pub pre_delay_ms: f32,                  // 0.0 ..= 250.0 ms
    pub damping_high_freq_hz: f32,          // 1000.0 ..= 18000.0 Hz
    pub damping_high_ratio: f32,            // 0.1 ..= 1.0
    pub early_reflections_db: f32,          // -24.0 ..= +6.0 dB
    pub late_reverberation_db: f32,         // -24.0 ..= +6.0 dB
    pub diffusion_pct: f32,                 // 0.0 ..= 100.0%
    pub source_pos_norm: (f32, f32),        // Normalized (X, Y) in room [0..1]
    pub listener_pos_norm: (f32, f32),      // Normalized (X, Y) in room [0..1]
    pub active_dragging_obj: Option<usize>, // 0: Source, 1: Listener
    pub ray_traces: Vec<AcousticRayPath>,
    pub color_palette: ContrastColorPalette,
}

impl Default for ReverbSpaceView {
    fn default() -> Self {
        Self::new()
    }
}

impl ReverbSpaceView {
    pub fn new() -> Self {
        let mut view = Self {
            algorithm: ReverbAlgorithm::HallConcert,
            room_size_m: 35.0,
            decay_time_rt60_s: 2.8,
            pre_delay_ms: 24.0,
            damping_high_freq_hz: 6500.0,
            damping_high_ratio: 0.45,
            early_reflections_db: -3.0,
            late_reverberation_db: 0.0,
            diffusion_pct: 85.0,
            source_pos_norm: (0.30, 0.65),
            listener_pos_norm: (0.70, 0.40),
            active_dragging_obj: None,
            ray_traces: Vec::new(),
            color_palette: ContrastColorPalette::default(),
        };
        view.recalculate_ray_traces();
        view
    }

    /// Recalculate 2D specular ray reflections inside the rectangular room enclosure.
    pub fn recalculate_ray_traces(&mut self) {
        self.ray_traces.clear();
        let num_rays = 12;
        let max_bounces = 3;

        for i in 0..num_rays {
            let angle = i as f32 / num_rays as f32 * std::f32::consts::TAU;
            let mut cur_pos = self.source_pos_norm;
            let mut cur_dir = (angle.cos(), angle.sin());
            let mut pts = vec![cur_pos];
            let mut energy = 1.0_f32;

            for bounce in 0..max_bounces {
                // Find ray intersection with unit box [0..1, 0..1]
                let (hit_pos, new_dir) = Self::ray_box_intersect(cur_pos, cur_dir);
                pts.push(hit_pos);
                cur_pos = hit_pos;
                cur_dir = new_dir;
                energy *= 0.70; // 30% absorption per wall bounce
                if bounce == max_bounces - 1 {
                    break;
                }
            }

            self.ray_traces.push(AcousticRayPath {
                points: pts,
                initial_angle_rad: angle,
                energy,
                order: max_bounces,
            });
        }
    }

    /// Compute 2D ray intersection with [0, 1]x[0, 1] unit square room boundaries.
    fn ray_box_intersect(origin: (f32, f32), dir: (f32, f32)) -> ((f32, f32), (f32, f32)) {
        let mut t_min = f32::MAX;
        let mut normal = (0.0, 0.0);

        // Check Left wall (x = 0)
        if dir.0 < -1e-4 {
            let t = -origin.0 / dir.0;
            if t > 1e-4 && t < t_min {
                t_min = t;
                normal = (1.0, 0.0);
            }
        }
        // Check Right wall (x = 1)
        if dir.0 > 1e-4 {
            let t = (1.0 - origin.0) / dir.0;
            if t > 1e-4 && t < t_min {
                t_min = t;
                normal = (-1.0, 0.0);
            }
        }
        // Check Bottom wall (y = 0)
        if dir.1 < -1e-4 {
            let t = -origin.1 / dir.1;
            if t > 1e-4 && t < t_min {
                t_min = t;
                normal = (0.0, 1.0);
            }
        }
        // Check Top wall (y = 1)
        if dir.1 > 1e-4 {
            let t = (1.0 - origin.1) / dir.1;
            if t > 1e-4 && t < t_min {
                t_min = t;
                normal = (0.0, -1.0);
            }
        }

        let hit_x = (origin.0 + dir.0 * t_min).clamp(0.0, 1.0);
        let hit_y = (origin.1 + dir.1 * t_min).clamp(0.0, 1.0);

        // Specular reflection: r = d - 2*(d.n)*n
        let dot = dir.0 * normal.0 + dir.1 * normal.1;
        let ref_x = dir.0 - 2.0 * dot * normal.0;
        let ref_y = dir.1 - 2.0 * dot * normal.1;

        ((hit_x, hit_y), (ref_x, ref_y))
    }

    /// Convert room normalized coordinates to screen pixel coordinates.
    pub fn room_to_screen_pos(&self, norm: (f32, f32), canvas: Rect) -> (f32, f32) {
        let sx = canvas.x + norm.0.clamp(0.0, 1.0) * canvas.width;
        let sy = canvas.y + (1.0 - norm.1.clamp(0.0, 1.0)) * canvas.height;
        (sx, sy)
    }

    /// Convert screen pixel coordinates to room normalized coordinates.
    pub fn screen_to_room_pos(&self, screen: (f32, f32), canvas: Rect) -> (f32, f32) {
        if canvas.width <= 0.0 || canvas.height <= 0.0 {
            return (0.5, 0.5);
        }
        let nx = ((screen.0 - canvas.x) / canvas.width).clamp(0.0, 1.0);
        let ny = (1.0 - (screen.1 - canvas.y) / canvas.height).clamp(0.0, 1.0);
        (nx, ny)
    }

    /// Hit-test Source or Listener puck with ergonomic minimum hit target (>=44x44pt).
    pub fn hit_test_source_or_listener(&self, pos: (f32, f32), canvas: Rect) -> Option<usize> {
        let src_s = self.room_to_screen_pos(self.source_pos_norm, canvas);
        let dist_src = ((pos.0 - src_s.0).powi(2) + (pos.1 - src_s.1).powi(2)).sqrt();
        if dist_src <= REVERB_OBJECT_HIT_RADIUS {
            return Some(0); // Source
        }

        let lis_s = self.room_to_screen_pos(self.listener_pos_norm, canvas);
        let dist_lis = ((pos.0 - lis_s.0).powi(2) + (pos.1 - lis_s.1).powi(2)).sqrt();
        if dist_lis <= REVERB_OBJECT_HIT_RADIUS {
            return Some(1); // Listener
        }

        None
    }

    /// Deterministic ASCII render of the 2D room geometry.
    pub fn render_ascii(&self, width: usize, height: usize) -> String {
        let mut grid = vec![vec!['.'; width]; height];
        let src_col =
            ((self.source_pos_norm.0 * (width - 1) as f32).round() as usize).min(width - 1);
        let src_row = (((1.0 - self.source_pos_norm.1) * (height - 1) as f32).round() as usize)
            .min(height - 1);
        let lis_col =
            ((self.listener_pos_norm.0 * (width - 1) as f32).round() as usize).min(width - 1);
        let lis_row = (((1.0 - self.listener_pos_norm.1) * (height - 1) as f32).round() as usize)
            .min(height - 1);

        grid[src_row][src_col] = 'S';
        grid[lis_row][lis_col] = 'L';

        grid.into_iter()
            .map(|row| row.into_iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(feature = "gui")]
impl ReverbSpaceView {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // 1. Header & Algorithm Presets Bar
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("ALGORITHMIC REVERB SPACE & RAY-TRACER")
                        .color(Color32::from_rgb(240, 245, 255))
                        .strong(),
                );
                ui.separator();
                ui.label(
                    egui::RichText::new(format!(
                        "RT60: {:.1}s | Size: {:.0}m | Pre: {:.0}ms",
                        self.decay_time_rt60_s, self.room_size_m, self.pre_delay_ms
                    ))
                    .color(Color32::from_rgb(0, 229, 255))
                    .strong(),
                );
            });

            ui.add_space(6.0);

            // Algorithm Selector Buttons (>=44pt Hit Targets)
            ui.horizontal(|ui| {
                let algos = [
                    ReverbAlgorithm::PlateReverb,
                    ReverbAlgorithm::HallConcert,
                    ReverbAlgorithm::CathedralSpace,
                    ReverbAlgorithm::RoomChamber,
                    ReverbAlgorithm::ShimmerEthereal,
                    ReverbAlgorithm::NonLinearGate,
                ];

                for algo in algos {
                    let is_act = self.algorithm == algo;
                    let btn = egui::Button::new(
                        egui::RichText::new(algo.name())
                            .color(if is_act {
                                Color32::from_rgb(10, 14, 22)
                            } else {
                                Color32::from_rgb(220, 235, 255)
                            })
                            .strong(),
                    )
                    .min_size(Vec2::new(100.0, MIN_HIT_TARGET_PT))
                    .fill(if is_act {
                        Color32::from_rgb(0, 229, 255)
                    } else {
                        Color32::from_rgb(30, 40, 60)
                    });

                    if ui.add(btn).clicked() {
                        self.algorithm = algo;
                    }
                }
            });

            ui.add_space(8.0);

            // 2. Dual Canvas: 2D Room Ray-Tracer (Left) + Damping & RT60 Curve (Right)
            ui.horizontal(|ui| {
                // Left Canvas: 2D Geometric Room Ray-Tracer
                let room_size = Vec2::new(340.0, 240.0);
                let (room_resp, painter) =
                    ui.allocate_painter(room_size, egui::Sense::click_and_drag());
                let room_canvas = Rect::new(
                    room_resp.rect.min.x,
                    room_resp.rect.min.y,
                    room_resp.rect.width(),
                    room_resp.rect.height(),
                );

                // Room Background
                painter.rect_filled(room_resp.rect, 8.0_f32, Color32::from_rgb(10, 14, 22));
                painter.rect_stroke(
                    room_resp.rect,
                    8.0_f32,
                    Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
                );

                // Draw Specular Reflection Rays
                for ray in &self.ray_traces {
                    for i in 0..ray.points.len() - 1 {
                        let p0 = self.room_to_screen_pos(ray.points[i], room_canvas);
                        let p1 = self.room_to_screen_pos(ray.points[i + 1], room_canvas);
                        let alpha = (ray.energy * 180.0) as u8;
                        painter.line_segment(
                            [egui::pos2(p0.0, p0.1), egui::pos2(p1.0, p1.1)],
                            Stroke::new(
                                1.5_f32,
                                Color32::from_rgba_unmultiplied(0, 229, 255, alpha),
                            ),
                        );
                    }
                }

                // Source Puck (Orange #FF6B2B)
                let src_screen = self.room_to_screen_pos(self.source_pos_norm, room_canvas);
                painter.circle_stroke(
                    egui::pos2(src_screen.0, src_screen.1),
                    REVERB_OBJECT_HIT_RADIUS,
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 107, 43, 140)),
                );
                painter.circle_filled(
                    egui::pos2(src_screen.0, src_screen.1),
                    REVERB_OBJECT_VISUAL_RADIUS,
                    Color32::from_rgb(255, 107, 43),
                );
                painter.text(
                    egui::pos2(src_screen.0, src_screen.1 - 18.0),
                    egui::Align2::CENTER_BOTTOM,
                    "SOURCE (S)",
                    egui::FontId::proportional(10.0),
                    Color32::from_rgb(255, 107, 43),
                );

                // Listener Puck (Cyan #00E5FF)
                let lis_screen = self.room_to_screen_pos(self.listener_pos_norm, room_canvas);
                painter.circle_stroke(
                    egui::pos2(lis_screen.0, lis_screen.1),
                    REVERB_OBJECT_HIT_RADIUS,
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 140)),
                );
                painter.circle_filled(
                    egui::pos2(lis_screen.0, lis_screen.1),
                    REVERB_OBJECT_VISUAL_RADIUS,
                    Color32::from_rgb(0, 229, 255),
                );
                painter.text(
                    egui::pos2(lis_screen.0, lis_screen.1 - 18.0),
                    egui::Align2::CENTER_BOTTOM,
                    "LISTENER (L)",
                    egui::FontId::proportional(10.0),
                    Color32::from_rgb(0, 229, 255),
                );

                // Handle Object Dragging
                if room_resp.drag_started() || room_resp.clicked() {
                    if let Some(pos) = room_resp.interact_pointer_pos() {
                        self.active_dragging_obj =
                            self.hit_test_source_or_listener((pos.x, pos.y), room_canvas);
                    }
                }

                if room_resp.dragged() {
                    if let Some(pos) = room_resp.interact_pointer_pos() {
                        let room_pos = self.screen_to_room_pos((pos.x, pos.y), room_canvas);
                        match self.active_dragging_obj {
                            Some(0) => {
                                self.source_pos_norm = room_pos;
                                self.recalculate_ray_traces();
                            }
                            Some(1) => {
                                self.listener_pos_norm = room_pos;
                            }
                            _ => {}
                        }
                    }
                }

                if room_resp.drag_stopped() {
                    self.active_dragging_obj = None;
                }

                ui.add_space(10.0);

                // Right Canvas: RT60 Decay & High-Frequency Damping Curve
                let curve_size = Vec2::new(340.0, 240.0);
                let (curve_resp, curve_painter) =
                    ui.allocate_painter(curve_size, egui::Sense::hover());

                curve_painter.rect_filled(curve_resp.rect, 8.0_f32, Color32::from_rgb(12, 16, 26));
                curve_painter.rect_stroke(
                    curve_resp.rect,
                    8.0_f32,
                    Stroke::new(1.5_f32, Color32::from_rgb(45, 60, 85)),
                );

                curve_painter.text(
                    egui::pos2(curve_resp.rect.min.x + 12.0, curve_resp.rect.min.y + 12.0),
                    egui::Align2::LEFT_TOP,
                    "RT60 SPECTRAL DECAY & DAMPING ENVELOPE",
                    egui::FontId::proportional(11.0),
                    Color32::from_rgb(255, 215, 0),
                );

                // Draw Log Decay Curve
                let mut prev_pt: Option<egui::Pos2> = None;
                for i in 0..40 {
                    let t = i as f32 / 39.0;
                    let cx = curve_resp.rect.min.x + 20.0 + t * 300.0;
                    let decay = (-t * (3.0 / self.decay_time_rt60_s)).exp();
                    let cy = curve_resp.rect.max.y - 20.0 - decay * 160.0;
                    let pt = egui::pos2(cx, cy);
                    if let Some(prev) = prev_pt {
                        curve_painter.line_segment(
                            [prev, pt],
                            Stroke::new(2.5_f32, Color32::from_rgb(255, 215, 0)),
                        );
                    }
                    prev_pt = Some(pt);
                }
            });

            ui.add_space(8.0);

            // 3. Reverb Physical Parameters Controls
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Room Size").strong());
                    ui.add(egui::Slider::new(&mut self.room_size_m, 5.0..=100.0).text("m"));
                });

                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("RT60 Decay").strong());
                    ui.add(egui::Slider::new(&mut self.decay_time_rt60_s, 0.2..=20.0).text("s"));
                });

                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Pre-Delay").strong());
                    ui.add(egui::Slider::new(&mut self.pre_delay_ms, 0.0..=250.0).text("ms"));
                });

                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Diffusion").strong());
                    ui.add(egui::Slider::new(&mut self.diffusion_pct, 0.0..=100.0).text("%"));
                });
            });
        });
    }
}
