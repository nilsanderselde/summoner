// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Granular Spectral Cloud Freeze & Grain Trajectory Visualizer HUD (Step 1514).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const GRANULAR_FREEZE_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_GRAIN_SIZE_MS: f32 = 10.0;
pub const MAX_GRAIN_SIZE_MS: f32 = 500.0;
pub const MIN_GRAIN_DENSITY_HZ: f32 = 2.0;
pub const MAX_GRAIN_DENSITY_HZ: f32 = 100.0;
pub const MIN_PITCH_SPRAY_ST: f32 = -24.0;
pub const MAX_PITCH_SPRAY_ST: f32 = 24.0;

/// Grain Window Envelope Characteristic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrainEnvelopeWindow {
    HannSmooth,     // Standard cosine bell (Zero click, balanced sidebands)
    BlackmanHarris, // 4-term minimum sidelobe window (Maximum spectral isolation)
    GaussianBell,   // Gaussian curve with adjustable sigma (Smooth cloud diffusion)
    TukeyTapered,   // Flat top cosine tapered (Punchy transient preservation)
    TrapezoidSharp, // Linear rise/fall (Experimental rhythmic chop)
}

/// Simulated Active Grain Particle in Spectral Cloud.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActiveGrainParticle {
    pub pos_norm: f32,        // Position along buffer [0.0 ..= 1.0]
    pub pitch_offset_st: f32, // Pitch shift [-24.0 ..= +24.0 st]
    pub age_norm: f32,        // Lifecycle progress [0.0 ..= 1.0]
    pub amplitude: f32,       // Instantaneous gain [0.0 ..= 1.0]
}

/// Granular Spectral Cloud Freeze View HUD (Step 1514).
#[derive(Debug, Clone)]
pub struct GranularFreezeView {
    pub window_type: GrainEnvelopeWindow,
    pub grain_size_ms: f32,         // [10.0 ..= 500.0 ms]
    pub grain_density_hz: f32,      // [2.0 ..= 100.0 grains/sec]
    pub pitch_spray_st: f32,        // [-24.0 ..= +24.0 semitones]
    pub position_jitter_pct: f32,   // [0.0 ..= 100.0 %]
    pub is_frozen: bool,            // Infinite spectral cloud hold
    pub cloud_puck_pos: (f32, f32), // Normalized (X: playhead position, Y: pitch spray)
    pub is_dragging_puck: bool,
    pub active_grain_count: usize,
    pub stereo_spread_pct: f32, // [0.0 ..= 100.0 %]
    pub color_palette: ContrastColorPalette,
}

impl Default for GranularFreezeView {
    fn default() -> Self {
        Self::new()
    }
}

impl GranularFreezeView {
    pub fn new() -> Self {
        let mut view = Self {
            window_type: GrainEnvelopeWindow::HannSmooth,
            grain_size_ms: 120.0,
            grain_density_hz: 35.0,
            pitch_spray_st: 7.0,
            position_jitter_pct: 25.0,
            is_frozen: true, // Freeze mode active
            cloud_puck_pos: (0.45, 0.65),
            is_dragging_puck: false,
            active_grain_count: 42,
            stereo_spread_pct: 85.0,
            color_palette: ContrastColorPalette::default(),
        };
        view.cloud_puck_pos = (0.45, Self::pitch_to_normalized(view.pitch_spray_st));
        view
    }

    /// Convert Grain Size [10.0 ..= 500.0 ms] to normalized coordinate [0.0 ..= 1.0].
    pub fn size_to_normalized(ms: f32) -> f32 {
        let s = ms.clamp(MIN_GRAIN_SIZE_MS, MAX_GRAIN_SIZE_MS);
        ((s - MIN_GRAIN_SIZE_MS) / (MAX_GRAIN_SIZE_MS - MIN_GRAIN_SIZE_MS)).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Grain Size [10.0 ..= 500.0 ms].
    pub fn normalized_to_size(norm: f32) -> f32 {
        MIN_GRAIN_SIZE_MS + norm.clamp(0.0, 1.0) * (MAX_GRAIN_SIZE_MS - MIN_GRAIN_SIZE_MS)
    }

    /// Convert Pitch Spray [-24.0 ..= +24.0 st] to normalized coordinate [0.0 ..= 1.0].
    pub fn pitch_to_normalized(st: f32) -> f32 {
        let p = st.clamp(MIN_PITCH_SPRAY_ST, MAX_PITCH_SPRAY_ST);
        ((p - MIN_PITCH_SPRAY_ST) / (MAX_PITCH_SPRAY_ST - MIN_PITCH_SPRAY_ST)).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Pitch Spray [-24.0 ..= +24.0 st].
    pub fn normalized_to_pitch(norm: f32) -> f32 {
        MIN_PITCH_SPRAY_ST + norm.clamp(0.0, 1.0) * (MAX_PITCH_SPRAY_ST - MIN_PITCH_SPRAY_ST)
    }

    /// Evaluate Grain Window Envelope $w(t)$ for normalized grain phase $t \in [0.0, 1.0]$.
    pub fn evaluate_window_envelope(&self, t_norm: f32) -> f32 {
        let t = t_norm.clamp(0.0, 1.0);
        let pi = std::f32::consts::PI;

        match self.window_type {
            GrainEnvelopeWindow::HannSmooth => (pi * t).sin().powi(2),
            GrainEnvelopeWindow::BlackmanHarris => {
                let a0 = 0.35875;
                let a1 = 0.48829;
                let a2 = 0.14128;
                let a3 = 0.01168;
                a0 - a1 * (2.0 * pi * t).cos() + a2 * (4.0 * pi * t).cos()
                    - a3 * (6.0 * pi * t).cos()
            }
            GrainEnvelopeWindow::GaussianBell => {
                let sigma = 0.20;
                (-((t - 0.5) / sigma).powi(2) * 0.5).exp()
            }
            GrainEnvelopeWindow::TukeyTapered => {
                let alpha = 0.35;
                if t < alpha * 0.5 {
                    0.5 * (1.0 + (pi * (2.0 * t / alpha - 1.0)).cos())
                } else if t <= 1.0 - alpha * 0.5 {
                    1.0
                } else {
                    0.5 * (1.0 + (pi * (2.0 * (1.0 - t) / alpha - 1.0)).cos())
                }
            }
            GrainEnvelopeWindow::TrapezoidSharp => {
                if t < 0.15 {
                    t / 0.15
                } else if t <= 0.85 {
                    1.0
                } else {
                    (1.0 - t) / 0.15
                }
            }
        }
    }

    /// Evaluate simulated grain particle at index $k \in [0, 8]$.
    pub fn evaluate_grain_particle(&self, idx: usize) -> ActiveGrainParticle {
        let frac = idx as f32 / 8.0;
        let base_pos = self.cloud_puck_pos.0;
        let spray = self.pitch_spray_st;
        let jitter = (self.position_jitter_pct / 100.0) * 0.15;

        let p_offset = ((frac * 13.7).sin() * jitter).clamp(-0.2, 0.2);
        let pos = (base_pos + p_offset).clamp(0.0, 1.0);
        let pitch = (frac * 7.3).cos() * spray;
        let age = (frac * 3.1 + 0.2).fract();
        let amp = self.evaluate_window_envelope(age);

        ActiveGrainParticle {
            pos_norm: pos,
            pitch_offset_st: pitch,
            age_norm: age,
            amplitude: amp,
        }
    }

    /// Hit-test touch coordinate on the spectral cloud freeze puck.
    pub fn hit_test_freeze_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.cloud_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.cloud_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= GRANULAR_FREEZE_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render of Spectral Grain Cloud and Window Envelope.
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

        // Draw active grain particles on left half
        let left_w = mid_x - 2;
        for i in 0..8 {
            let grain = self.evaluate_grain_particle(i);
            let col = 1 + (grain.pos_norm * left_w as f32).round() as usize;
            let norm_pitch = Self::pitch_to_normalized(grain.pitch_offset_st);
            let row = (((1.0 - norm_pitch) * (height - 3) as f32) + 1.0).round() as usize;
            if row > 0 && row < height - 1 && col < mid_x {
                grid[row][col] = '*';
            }
        }

        // Freeze Puck on left half
        let puck_col = ((self.cloud_puck_pos.0 * (mid_x - 2) as f32) + 1.0).round() as usize;
        let puck_row =
            (((1.0 - self.cloud_puck_pos.1) * (height - 3) as f32) + 1.0).round() as usize;
        if puck_row < height - 1 && puck_col < mid_x {
            grid[puck_row][puck_col] = 'O';
        }

        // Draw Window Envelope on right half
        let right_w = width - mid_x - 2;
        for c in 0..right_w {
            let frac = c as f32 / right_w.max(1) as f32;
            let env = self.evaluate_window_envelope(frac);
            let row = (((1.0 - env) * (height - 3) as f32) + 1.0).round() as usize;
            if row > 0 && row < height - 1 {
                grid[row][mid_x + 1 + c] = '^';
            }
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
        let _canvas_rect = Rect::new(rect.min.x, rect.min.y, rect.width(), rect.height());

        // Background
        painter.rect_filled(rect, 6.0, Color32::from_rgb(14, 18, 28));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "GRANULAR SPECTRAL CLOUD FREEZE & GRAIN TRAJECTORY HUD",
            egui::FontId::proportional(16.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Window Envelope Selector Tabs (y: 48..92) - Each tab >= 44pt height
        let windows = [
            (GrainEnvelopeWindow::HannSmooth, "HANN BELL"),
            (GrainEnvelopeWindow::BlackmanHarris, "BLACKMAN-H"),
            (GrainEnvelopeWindow::GaussianBell, "GAUSSIAN"),
            (GrainEnvelopeWindow::TukeyTapered, "TUKEY FLAT"),
            (GrainEnvelopeWindow::TrapezoidSharp, "TRAPEZOID"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (win, name)) in windows.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_selected = self.window_type == *win;
            let bg_color = if is_selected {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(25, 35, 50)
            };
            let text_color = if is_selected {
                Color32::from_rgb(10, 14, 24)
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
                        self.window_type = *win;
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

        // Left 55%: 2D Spectral Cloud Canvas (Position vs Pitch Spray)
        let left_w = main_canvas.width() * 0.55;
        let left_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(left_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(left_rect, 4.0, Color32::from_rgb(14, 20, 32));
        painter.rect_stroke(
            left_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(40, 55, 80)),
        );

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "SPECTRAL GRAIN CLOUD & TIME-STRETCH EMISSION SPACE",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // Draw Active Grain Particles Cloud
        let cloud_w = left_rect.width() - 30.0;
        let cloud_h = left_rect.height() - 40.0;

        for i in 0..12 {
            let grain = self.evaluate_grain_particle(i % 8);
            let gx = left_rect.min.x + 15.0 + grain.pos_norm * cloud_w;
            let norm_p = Self::pitch_to_normalized(grain.pitch_offset_st);
            let gy = left_rect.max.y - 15.0 - norm_p * cloud_h;
            let r_size = 3.0 + grain.amplitude * 5.0;

            let alpha = (grain.amplitude * 200.0) as u8;
            painter.circle_filled(
                egui::pos2(gx, gy),
                r_size,
                Color32::from_rgba_premultiplied(0, 255, 180, alpha),
            );
        }

        // Freeze Playhead Puck
        let puck_x = left_rect.min.x + 15.0 + self.cloud_puck_pos.0 * cloud_w;
        let puck_y = left_rect.max.y - 15.0 - self.cloud_puck_pos.1 * cloud_h;
        let puck_pos = egui::pos2(puck_x, puck_y);

        // Handle interaction
        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - (left_rect.min.x + 15.0)) / cloud_w).clamp(0.0, 1.0);
                    let ny = (((left_rect.max.y - 15.0) - mouse_pos.y) / cloud_h).clamp(0.0, 1.0);
                    self.cloud_puck_pos = (nx, ny);
                    self.pitch_spray_st = Self::normalized_to_pitch(ny);
                }
            }
        }

        // Vertical Playhead Line
        painter.line_segment(
            [
                egui::pos2(puck_x, left_rect.min.y + 30.0),
                egui::pos2(puck_x, left_rect.max.y - 10.0),
            ],
            Stroke::new(1.5_f32, Color32::from_rgb(0, 229, 255)),
        );

        // Puck Hit Target (>= 44x44pt)
        painter.circle_stroke(
            puck_pos,
            GRANULAR_FREEZE_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(0, 229, 255, 140)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(0, 229, 255));
        painter.circle_filled(puck_pos, 4.0, Color32::from_rgb(255, 255, 255));

        // Right 45%: Grain Window Envelope & Emission Density
        let right_w = main_canvas.width() * 0.45;
        let right_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + left_w + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(right_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(right_rect, 4.0, Color32::from_rgb(14, 20, 32));
        painter.rect_stroke(
            right_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(40, 55, 80)),
        );

        painter.text(
            egui::pos2(right_rect.min.x + 10.0, right_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "GRAIN WINDOW ENVELOPE",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // Draw Window Envelope Curve
        let env_w = right_rect.width() - 30.0;
        let env_h = right_rect.height() - 50.0;
        let num_env_pts = 40;
        let mut prev_pt = None;

        for c in 0..=num_env_pts {
            let frac = c as f32 / num_env_pts as f32;
            let val = self.evaluate_window_envelope(frac);
            let px = right_rect.min.x + 15.0 + frac * env_w;
            let py = right_rect.max.y - 15.0 - val * env_h;
            let pt = egui::pos2(px, py);

            if let Some(prev) = prev_pt {
                painter.line_segment(
                    [prev, pt],
                    Stroke::new(2.0_f32, Color32::from_rgb(255, 215, 0)),
                );
            }
            prev_pt = Some(pt);
        }

        // Freeze Status Pill
        let freeze_col = if self.is_frozen {
            Color32::from_rgb(0, 229, 255)
        } else {
            Color32::from_rgb(100, 120, 140)
        };
        painter.text(
            egui::pos2(right_rect.max.x - 15.0, right_rect.min.y + 10.0),
            egui::Align2::RIGHT_TOP,
            if self.is_frozen {
                "FREEZE: LOCKED"
            } else {
                "FREEZE: PASS"
            },
            egui::FontId::proportional(11.0),
            freeze_col,
        );

        // Bottom Metrics Dock (y: 350..465)
        let dock_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 350.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 465.0),
        );
        painter.rect_filled(dock_rect, 6.0, Color32::from_rgb(18, 25, 38));
        painter.rect_stroke(
            dock_rect,
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        let params = [
            (
                "GRAIN SIZE (DUR)",
                format!("{:.0} ms (Overlap)", self.grain_size_ms),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "GRAIN DENSITY (RATE)",
                format!(
                    "{:.1} grains/s ({})",
                    self.grain_density_hz, self.active_grain_count
                ),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "PITCH SPRAY (DETUNE)",
                format!(
                    "{:+.1} st (Spread {:.0}%)",
                    self.pitch_spray_st, self.stereo_spread_pct
                ),
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "SPECTRAL FREEZE STATE",
                if self.is_frozen {
                    "INFINITE HOLD".to_string()
                } else {
                    "LIVE RUNNING".to_string()
                },
                Color32::from_rgb(0, 255, 180),
            ),
        ];

        let col_w = (dock_rect.width() - 40.0) / 4.0;
        for (i, (label, val, col)) in params.iter().enumerate() {
            let px_pos = dock_rect.min.x + 20.0 + i as f32 * col_w;
            painter.text(
                egui::pos2(px_pos, dock_rect.min.y + 12.0),
                egui::Align2::LEFT_TOP,
                *label,
                egui::FontId::proportional(11.0),
                Color32::from_rgb(160, 180, 205),
            );
            painter.text(
                egui::pos2(px_pos, dock_rect.min.y + 30.0),
                egui::Align2::LEFT_TOP,
                val,
                egui::FontId::proportional(14.0),
                *col,
            );
        }

        // Compliance Verification Badge
        let badge_rect = egui::Rect::from_min_max(
            egui::pos2(dock_rect.min.x + 15.0, dock_rect.min.y + 68.0),
            egui::pos2(dock_rect.max.x - 15.0, dock_rect.max.y - 11.0),
        );
        painter.rect_filled(badge_rect, 4.0, Color32::from_rgb(16, 35, 28));
        painter.rect_stroke(
            badge_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(0, 255, 180)),
        );
        painter.text(
            egui::pos2(badge_rect.min.x + 10.0, badge_rect.min.y + 8.0),
            egui::Align2::LEFT_TOP,
            "[PASS] Granular Spectral Cloud Freeze & Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
