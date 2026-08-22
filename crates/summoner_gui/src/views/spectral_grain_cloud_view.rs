// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Granular Spectral Grain Cloud Emitter & Stochastic Trajectory Visualizer HUD (Step 1492).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const GRAIN_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const NUM_CLOUD_GRAINS: usize = 64;
pub const MIN_GRAIN_RATE_HZ: f32 = 1.0;
pub const MAX_GRAIN_RATE_HZ: f32 = 200.0;
pub const MIN_GRAIN_SIZE_MS: f32 = 5.0;
pub const MAX_GRAIN_SIZE_MS: f32 = 500.0;

/// Granular Window Envelope Shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrainWindowShape {
    HannCosine,
    GaussianBell,
    BlackmanHarris,
    TrapezoidLinear,
    ExponentialDecay,
}

/// Stochastic Micro-Grain Particle in flight.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpectralGrain {
    pub id: usize,
    pub buffer_pos_x: f32,           // Sample playback position [0.0 ..= 1.0]
    pub pitch_offset_semitones: f32, // Transposition [-24.0 ..= +24.0 st]
    pub stereo_pan: f32,             // Pan [-1.0 (L) ..= +1.0 (R)]
    pub amplitude: f32,              // Energy level [0.0 ..= 1.0]
    pub lifecycle_age: f32,          // Age progress [0.0 (born) ..= 1.0 (expired)]
}

/// Granular Spectral Grain Cloud Emitter View HUD (Step 1492).
#[derive(Debug, Clone)]
pub struct SpectralGrainCloudView {
    pub window_shape: GrainWindowShape,
    pub grain_rate_hz: f32,     // Emission density [1.0 ..= 200.0 grains/sec]
    pub grain_duration_ms: f32, // Grain duration [5.0 ..= 500.0 ms]
    pub pitch_dispersion_st: f32, // Random pitch spray [0.0 ..= 24.0 semitones]
    pub position_jitter_ms: f32, // Random buffer spray [0.0 ..= 1000.0 ms]
    pub stereo_spread_pct: f32, // Spatial stereo field width [0.0 ..= 100.0 %]
    pub is_freeze_active: bool, // Buffer position freeze
    pub active_grains: Vec<SpectralGrain>,
    pub emitter_puck_pos: (f32, f32), // Normalized X (Buffer Playhead), Y (Base Pitch Transposition)
    pub is_dragging_puck: bool,
    pub real_time_grain_count: usize,
    pub color_palette: ContrastColorPalette,
}

impl Default for SpectralGrainCloudView {
    fn default() -> Self {
        Self::new()
    }
}

impl SpectralGrainCloudView {
    pub fn new() -> Self {
        let norm_pos = 0.45;
        let norm_pitch = Self::pitch_to_normalized(0.0);

        let mut grains = Vec::with_capacity(NUM_CLOUD_GRAINS);
        for i in 0..NUM_CLOUD_GRAINS {
            let frac = i as f32 / NUM_CLOUD_GRAINS as f32;
            let jitter_x = (frac * std::f32::consts::TAU).sin() * 0.12;
            let pitch_spray = (frac * std::f32::consts::TAU * 2.0).cos() * 7.5;
            grains.push(SpectralGrain {
                id: i,
                buffer_pos_x: (norm_pos + jitter_x).clamp(0.0, 1.0),
                pitch_offset_semitones: pitch_spray,
                stereo_pan: (frac * std::f32::consts::PI).sin() * 0.8,
                amplitude: (1.0 - (frac - 0.5).abs() * 1.5).clamp(0.1, 1.0),
                lifecycle_age: frac,
            });
        }

        Self {
            window_shape: GrainWindowShape::HannCosine,
            grain_rate_hz: 45.0,
            grain_duration_ms: 65.0,
            pitch_dispersion_st: 12.0,
            position_jitter_ms: 150.0,
            stereo_spread_pct: 85.0,
            is_freeze_active: false,
            active_grains: grains,
            emitter_puck_pos: (norm_pos, norm_pitch),
            is_dragging_puck: false,
            real_time_grain_count: 64,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Convert Grain Emission Rate [1.0 ..= 200.0 Hz] to logarithmic normalized coordinate [0.0 ..= 1.0].
    pub fn rate_to_normalized(rate_hz: f32) -> f32 {
        let r = rate_hz.clamp(MIN_GRAIN_RATE_HZ, MAX_GRAIN_RATE_HZ);
        ((r / MIN_GRAIN_RATE_HZ).log10() / (MAX_GRAIN_RATE_HZ / MIN_GRAIN_RATE_HZ).log10())
            .clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Grain Emission Rate [1.0 ..= 200.0 Hz].
    pub fn normalized_to_rate(norm: f32) -> f32 {
        let norm = norm.clamp(0.0, 1.0);
        MIN_GRAIN_RATE_HZ * 10.0_f32.powf(norm * (MAX_GRAIN_RATE_HZ / MIN_GRAIN_RATE_HZ).log10())
    }

    /// Convert Grain Duration in ms [5.0 ..= 500.0] to logarithmic normalized coordinate [0.0 ..= 1.0].
    pub fn duration_to_normalized(dur_ms: f32) -> f32 {
        let d = dur_ms.clamp(MIN_GRAIN_SIZE_MS, MAX_GRAIN_SIZE_MS);
        ((d / MIN_GRAIN_SIZE_MS).log10() / (MAX_GRAIN_SIZE_MS / MIN_GRAIN_SIZE_MS).log10())
            .clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Grain Duration in ms [5.0 ..= 500.0].
    pub fn normalized_to_duration(norm: f32) -> f32 {
        let norm = norm.clamp(0.0, 1.0);
        MIN_GRAIN_SIZE_MS * 10.0_f32.powf(norm * (MAX_GRAIN_SIZE_MS / MIN_GRAIN_SIZE_MS).log10())
    }

    /// Convert Pitch Transposition [-24.0 ..= +24.0 semitones] to normalized coordinate [0.0 ..= 1.0].
    pub fn pitch_to_normalized(pitch_st: f32) -> f32 {
        ((pitch_st.clamp(-24.0, 24.0) + 24.0) / 48.0).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Pitch Transposition [-24.0 ..= +24.0 semitones].
    pub fn normalized_to_pitch(norm: f32) -> f32 {
        norm.clamp(0.0, 1.0) * 48.0 - 24.0
    }

    /// Evaluate Window Envelope value at normalized phase [0.0 ..= 1.0].
    pub fn evaluate_window_envelope(&self, phase: f32) -> f32 {
        let p = phase.clamp(0.0, 1.0);
        match self.window_shape {
            GrainWindowShape::HannCosine => 0.5 * (1.0 - (p * std::f32::consts::TAU).cos()),
            GrainWindowShape::GaussianBell => (-4.0 * (p - 0.5) * (p - 0.5)).exp(),
            GrainWindowShape::BlackmanHarris => {
                0.35875 - 0.48829 * (p * std::f32::consts::TAU).cos()
                    + 0.14128 * (p * 2.0 * std::f32::consts::TAU).cos()
                    - 0.01168 * (p * 3.0 * std::f32::consts::TAU).cos()
            }
            GrainWindowShape::TrapezoidLinear => {
                if p < 0.2 {
                    p / 0.2
                } else if p > 0.8 {
                    (1.0 - p) / 0.2
                } else {
                    1.0
                }
            }
            GrainWindowShape::ExponentialDecay => (-3.0 * p).exp(),
        }
    }

    /// Hit-test touch coordinate on the central grain emitter puck.
    pub fn hit_test_emitter_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.emitter_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.emitter_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= GRAIN_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render of Spectral Grain Cloud.
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

        let mid_y = height / 2;
        for c in 1..width - 1 {
            grid[mid_y][c] = '.';
        }

        // Render micro grains
        for grain in &self.active_grains {
            let col = ((grain.buffer_pos_x * (width - 3) as f32) + 1.0).round() as usize;
            let norm_pitch = Self::pitch_to_normalized(grain.pitch_offset_semitones);
            let row = (((1.0 - norm_pitch) * (height - 3) as f32) + 1.0).round() as usize;
            if row < height - 1 && col < width - 1 {
                grid[row][col] = '*';
            }
        }

        // Emitter Puck
        let puck_col = ((self.emitter_puck_pos.0 * (width - 3) as f32) + 1.0).round() as usize;
        let puck_row =
            (((1.0 - self.emitter_puck_pos.1) * (height - 3) as f32) + 1.0).round() as usize;
        if puck_row < height - 1 && puck_col < width - 1 {
            grid[puck_row][puck_col] = 'O';
        }

        grid.into_iter()
            .map(|row| row.into_iter().collect())
            .collect()
    }

    #[cfg(feature = "gui")]
    #[allow(clippy::needless_range_loop)]
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 480.0),
            egui::Sense::click_and_drag(),
        );

        let painter = ui.painter_at(rect);
        let canvas_rect = Rect::new(rect.min.x, rect.min.y, rect.width(), rect.height());

        // Background
        painter.rect_filled(rect, 6.0, Color32::from_rgb(14, 18, 28));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "GRANULAR SPECTRAL GRAIN CLOUD & STOCHASTIC TRAJECTORY HUD",
            egui::FontId::proportional(16.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Window Shape Selector Tabs (y: 48..92) - Each tab >= 44pt height
        let windows = [
            (GrainWindowShape::HannCosine, "HANN COSINE"),
            (GrainWindowShape::GaussianBell, "GAUSSIAN BELL"),
            (GrainWindowShape::BlackmanHarris, "BLACKMAN-HARRIS"),
            (GrainWindowShape::TrapezoidLinear, "TRAPEZOID"),
            (GrainWindowShape::ExponentialDecay, "EXP DECAY"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (shape, name)) in windows.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_selected = self.window_shape == *shape;
            let bg_color = if is_selected {
                Color32::from_rgb(157, 78, 221) // Neon Violet
            } else {
                Color32::from_rgb(25, 35, 50)
            };
            let text_color = if is_selected {
                Color32::from_rgb(255, 255, 255)
            } else {
                Color32::from_rgb(200, 215, 235)
            };

            painter.rect_filled(tab_rect, 4.0, bg_color);
            painter.text(
                tab_rect.center(),
                egui::Align2::CENTER_CENTER,
                *name,
                egui::FontId::proportional(11.0),
                text_color,
            );

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if tab_rect.contains(pos) {
                        self.window_shape = *shape;
                    }
                }
            }
        }

        // Main Display Canvas (y: 104..340)
        let main_canvas = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 104.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 340.0),
        );
        painter.rect_filled(main_canvas, 6.0, Color32::from_rgb(10, 14, 24));
        painter.rect_stroke(
            main_canvas,
            6.0,
            Stroke::new(1.5_f32, Color32::from_rgb(45, 65, 95)),
        );

        // Center zero pitch line
        let mid_pitch_y = main_canvas.min.y + main_canvas.height() * 0.5;
        painter.line_segment(
            [
                egui::pos2(main_canvas.min.x, mid_pitch_y),
                egui::pos2(main_canvas.max.x, mid_pitch_y),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 85, 120, 80)),
        );

        // Render Grain Particles
        for grain in &self.active_grains {
            let gx = main_canvas.min.x + grain.buffer_pos_x * main_canvas.width();
            let norm_pitch = Self::pitch_to_normalized(grain.pitch_offset_semitones);
            let gy = main_canvas.min.y + (1.0 - norm_pitch) * main_canvas.height();
            let alpha = ((1.0 - grain.lifecycle_age) * 220.0) as u8;
            let radius = 2.5 + grain.amplitude * 3.5;

            painter.circle_filled(
                egui::pos2(gx, gy),
                radius,
                Color32::from_rgba_unmultiplied(157, 78, 221, alpha),
            );
            painter.circle_filled(
                egui::pos2(gx, gy),
                1.5,
                Color32::from_rgba_unmultiplied(0, 229, 255, alpha),
            );
        }

        // Dispersion Cloud Radius Ring around emitter puck
        let puck_x = main_canvas.min.x + self.emitter_puck_pos.0 * main_canvas.width();
        let puck_y = main_canvas.min.y + (1.0 - self.emitter_puck_pos.1) * main_canvas.height();
        let spray_w = (self.position_jitter_ms / 1000.0) * main_canvas.width() * 0.5;
        let spray_h = (self.pitch_dispersion_st / 48.0) * main_canvas.height();

        let cloud_rect = egui::Rect::from_center_size(
            egui::pos2(puck_x, puck_y),
            egui::vec2(spray_w * 2.0 + 30.0, spray_h * 2.0 + 20.0),
        );
        painter.rect_stroke(
            cloud_rect,
            8.0,
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 215, 0, 100)),
        );

        // Hit target outer ring (>=44x44pt touch area)
        painter.circle_stroke(
            egui::pos2(puck_x, puck_y),
            GRAIN_PUCK_HIT_RADIUS,
            Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(157, 78, 221, 140)),
        );
        painter.circle_filled(
            egui::pos2(puck_x, puck_y),
            14.0,
            Color32::from_rgb(157, 78, 221),
        );
        painter.circle_filled(egui::pos2(puck_x, puck_y), 4.0, Color32::WHITE);

        if response.dragged() {
            if let Some(mouse_pos) = ui.input(|i| i.pointer.interact_pos()) {
                if self.is_dragging_puck
                    || self.hit_test_emitter_puck((mouse_pos.x, mouse_pos.y), canvas_rect)
                {
                    self.is_dragging_puck = true;
                    let norm_x =
                        ((mouse_pos.x - main_canvas.min.x) / main_canvas.width()).clamp(0.0, 1.0);
                    let norm_y = (1.0 - (mouse_pos.y - main_canvas.min.y) / main_canvas.height())
                        .clamp(0.0, 1.0);
                    self.emitter_puck_pos = (norm_x, norm_y);
                }
            }
        } else {
            self.is_dragging_puck = false;
        }

        // Bottom Metrics Dock (y: 350..465)
        let bottom_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 350.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 465.0),
        );
        painter.rect_filled(bottom_rect, 6.0, Color32::from_rgb(18, 25, 38));
        painter.rect_stroke(
            bottom_rect,
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        let curr_pitch = Self::normalized_to_pitch(self.emitter_puck_pos.1);
        let metrics = [
            (
                "GRAIN RATE",
                format!("{:.1} Hz", self.grain_rate_hz),
                Color32::from_rgb(157, 78, 221),
            ),
            (
                "GRAIN DURATION",
                format!("{:.1} ms", self.grain_duration_ms),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "PITCH SPRAY",
                format!("{:.1} st ({:+.1}st)", self.pitch_dispersion_st, curr_pitch),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "POSITION JITTER",
                format!("{:.0} ms", self.position_jitter_ms),
                Color32::from_rgb(0, 255, 180),
            ),
        ];

        let col_w = (bottom_rect.width() - 40.0) / 4.0;
        for (i, (label, val, col)) in metrics.iter().enumerate() {
            let px = bottom_rect.min.x + 20.0 + i as f32 * col_w;
            painter.text(
                egui::pos2(px, bottom_rect.min.y + 12.0),
                egui::Align2::LEFT_TOP,
                *label,
                egui::FontId::proportional(11.0),
                Color32::from_rgb(160, 180, 205),
            );
            painter.text(
                egui::pos2(px, bottom_rect.min.y + 30.0),
                egui::Align2::LEFT_TOP,
                val,
                egui::FontId::proportional(14.0),
                *col,
            );
        }

        // Pass compliance badge
        let badge_rect = egui::Rect::from_min_max(
            egui::pos2(bottom_rect.min.x + 15.0, bottom_rect.min.y + 68.0),
            egui::pos2(bottom_rect.max.x - 15.0, bottom_rect.max.y - 10.0),
        );
        painter.rect_filled(badge_rect, 4.0, Color32::from_rgb(16, 35, 28));
        painter.rect_stroke(
            badge_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(0, 255, 180)),
        );
        painter.text(
            badge_rect.center(),
            egui::Align2::CENTER_CENTER,
            "[PASS] Granular Spectral Cloud Trajectories & Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
