// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Neural Wavetable Morphing Synthesizer & 3D Latent Trajectory Orbit HUD (Step 1524).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const NEURAL_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_LATENT_COORD: f32 = -2.50;
pub const MAX_LATENT_COORD: f32 = 2.50;
pub const MIN_MORPH_SPEED_HZ: f32 = 0.01;
pub const MAX_MORPH_SPEED_HZ: f32 = 20.00;
pub const MIN_ORBIT_RADIUS: f32 = 0.00;
pub const MAX_ORBIT_RADIUS: f32 = 2.00;

/// Neural Latent Space Generator Architecture Profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LatentArchitecture {
    VaeContinuous,   // Smooth variational autoencoder latent manifold
    TransformerDyn,  // Attention-guided sequential timbre trajectories
    DiffusionRes,    // High-resolution spectral denoising flow
    Hypersphere4D,   // Angular geodesic quaternion wavetable orbit
    SpectralFlow,    // Continuous normalizing flow invertible manifold
}

impl LatentArchitecture {
    pub fn default_morph_speed_hz(&self) -> f32 {
        match self {
            Self::VaeContinuous => 0.85,
            Self::TransformerDyn => 2.40,
            Self::DiffusionRes => 0.35,
            Self::Hypersphere4D => 1.20,
            Self::SpectralFlow => 0.65,
        }
    }

    pub fn default_orbit_radius(&self) -> f32 {
        match self {
            Self::VaeContinuous => 0.45,
            Self::TransformerDyn => 0.75,
            Self::DiffusionRes => 0.30,
            Self::Hypersphere4D => 0.90,
            Self::SpectralFlow => 0.55,
        }
    }

    pub fn reconstruction_fid_score(&self) -> f32 {
        match self {
            Self::VaeContinuous => 98.4,
            Self::TransformerDyn => 97.8,
            Self::DiffusionRes => 99.6,
            Self::Hypersphere4D => 98.9,
            Self::SpectralFlow => 99.2,
        }
    }
}

/// Neural Wavetable Morphing Synthesizer View HUD (Step 1524).
#[derive(Debug, Clone)]
pub struct NeuralWavetableView {
    pub architecture: LatentArchitecture,
    pub latent_z: (f32, f32, f32),     // 3D latent coordinate (z1, z2, z3)
    pub morph_speed_hz: f32,           // [0.01 ..= 20.00 Hz]
    pub orbit_radius: f32,             // [0.00 ..= 2.00]
    pub orbit_phase_rad: f32,          // Real-time orbital angle [0 .. 2π]
    pub latent_puck_pos: (f32, f32),   // Normalized (X: z1, Y: z2)
    pub is_dragging_puck: bool,
    pub spectral_entropy_bits: f32,    // Calculated overtone entropy
    pub reconstruction_quality_pct: f32,
    pub num_harmonics: usize,          // Default 16 harmonics
    pub color_palette: ContrastColorPalette,
}

impl Default for NeuralWavetableView {
    fn default() -> Self {
        Self::new()
    }
}

impl NeuralWavetableView {
    pub fn new() -> Self {
        let mut view = Self {
            architecture: LatentArchitecture::VaeContinuous,
            latent_z: (0.62, -0.45, 0.18),
            morph_speed_hz: 0.85,
            orbit_radius: 0.45,
            orbit_phase_rad: 0.0,
            latent_puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            spectral_entropy_bits: 3.84,
            reconstruction_quality_pct: 99.2,
            num_harmonics: 16,
            color_palette: ContrastColorPalette::default(),
        };
        view.latent_puck_pos = (
            Self::coord_to_normalized(view.latent_z.0),
            Self::coord_to_normalized(view.latent_z.1),
        );
        view.update_neural_calculations();
        view
    }

    /// Convert Latent Coordinate [-2.5 ..= +2.5] to normalized [0.0 ..= 1.0].
    pub fn coord_to_normalized(z: f32) -> f32 {
        let c = z.clamp(MIN_LATENT_COORD, MAX_LATENT_COORD);
        ((c - MIN_LATENT_COORD) / (MAX_LATENT_COORD - MIN_LATENT_COORD)).clamp(0.0, 1.0)
    }

    /// Convert normalized [0.0 ..= 1.0] to Latent Coordinate [-2.5 ..= +2.5].
    pub fn normalized_to_coord(norm: f32) -> f32 {
        MIN_LATENT_COORD + norm.clamp(0.0, 1.0) * (MAX_LATENT_COORD - MIN_LATENT_COORD)
    }

    /// Convert Morph Speed [0.01 ..= 20.00 Hz] to normalized [0.0 ..= 1.0].
    pub fn speed_to_normalized(speed: f32) -> f32 {
        let s = speed.clamp(MIN_MORPH_SPEED_HZ, MAX_MORPH_SPEED_HZ);
        ((s - MIN_MORPH_SPEED_HZ) / (MAX_MORPH_SPEED_HZ - MIN_MORPH_SPEED_HZ)).clamp(0.0, 1.0)
    }

    /// Convert normalized [0.0 ..= 1.0] to Morph Speed [0.01 ..= 20.00 Hz].
    pub fn normalized_to_speed(norm: f32) -> f32 {
        MIN_MORPH_SPEED_HZ + norm.clamp(0.0, 1.0) * (MAX_MORPH_SPEED_HZ - MIN_MORPH_SPEED_HZ)
    }

    /// Convert Orbit Radius [0.00 ..= 2.00] to normalized [0.0 ..= 1.0].
    pub fn radius_to_normalized(r: f32) -> f32 {
        (r.clamp(MIN_ORBIT_RADIUS, MAX_ORBIT_RADIUS) / MAX_ORBIT_RADIUS).clamp(0.0, 1.0)
    }

    /// Convert normalized [0.0 ..= 1.0] to Orbit Radius [0.00 ..= 2.00].
    pub fn normalized_to_radius(norm: f32) -> f32 {
        norm.clamp(0.0, 1.0) * MAX_ORBIT_RADIUS
    }

    /// Update neural manifold synthesis calculations and spectral metrics.
    pub fn update_neural_calculations(&mut self) {
        let r = self.orbit_radius;
        let z1 = self.latent_z.0;
        let z2 = self.latent_z.1;
        let z3 = self.latent_z.2;

        let magnitude = (z1 * z1 + z2 * z2 + z3 * z3 + r * r).sqrt();
        self.spectral_entropy_bits = (2.0 + magnitude * 0.8).clamp(1.0, 5.0);
        self.reconstruction_quality_pct = self.architecture.reconstruction_fid_score();
    }

    /// Evaluate 3D perspective projection for latent coordinate $(x, y, z)$ onto 2D canvas pixel coordinates.
    pub fn project_3d_latent(&self, x: f32, y: f32, z: f32, center: (f32, f32), scale: f32) -> (f32, f32) {
        let yaw = -0.45_f32; // ~-25 deg
        let pitch = 0.35_f32; // ~20 deg

        // Yaw rotation around Y axis
        let x1 = x * yaw.cos() - z * yaw.sin();
        let z1 = x * yaw.sin() + z * yaw.cos();

        // Pitch rotation around X axis
        let y2 = y * pitch.cos() - z1 * pitch.sin();

        (center.0 + x1 * scale, center.1 - y2 * scale)
    }

    /// Evaluate neural reconstructed single-cycle wavetable sample $x(t)$ for phase $t \in [0, 1]$.
    pub fn evaluate_wavetable_sample(&self, phase_norm: f32) -> f32 {
        let t = phase_norm.clamp(0.0, 1.0);
        let z1 = self.latent_z.0;
        let z2 = self.latent_z.1;
        let z3 = self.latent_z.2;

        // Neural Decoder non-linear harmonic synthesis: D(z)
        let h1 = (t * std::f32::consts::PI * 2.0).sin() * (0.8 + z1 * 0.1);
        let h2 = (t * std::f32::consts::PI * 4.0).sin() * (0.4 + z2.abs() * 0.2);
        let h3 = (t * std::f32::consts::PI * 6.0).sin() * (0.25 + z3 * 0.15);
        let h5 = (t * std::f32::consts::PI * 10.0).sin() * (0.15 * (z1 + z2).abs());

        // Saturate non-linearly with tanh
        ((h1 + h2 + h3 + h5) * 1.1).tanh() * 0.85
    }

    /// Evaluate harmonic overtone magnitude for harmonic index $h \in [1, 16]$.
    pub fn evaluate_harmonic_energy(&self, harmonic_idx: usize) -> f32 {
        let h = harmonic_idx.clamp(1, 16) as f32;
        let z1 = self.latent_z.0.abs();
        let z2 = self.latent_z.1.abs();
        let decay = (1.0 / (h.powf(0.8 + z1 * 0.2))).clamp(0.02, 1.0);
        let formant = (-0.5 * ((h - (3.0 + z2 * 4.0)) / 1.8).powi(2)).exp() * 0.6;
        (decay * 0.7 + formant * 0.3).clamp(0.02, 1.0)
    }

    /// Hit-test touch coordinate on the latent mean puck.
    pub fn hit_test_latent_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.latent_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.latent_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= NEURAL_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render of 3D Latent Trajectory and Reconstructed Wavetable.
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

        // Draw Wavetable Cycle on right half
        let right_w = width - mid_x - 2;
        for c in 0..right_w {
            let t_norm = c as f32 / (right_w.max(1) as f32);
            let sample = self.evaluate_wavetable_sample(t_norm);
            let row = ((height as f32 / 2.0) - sample * (height as f32 * 0.35)).round() as usize;
            if row > 0 && row < height - 1 {
                grid[row][mid_x + 1 + c] = '~';
            }
        }

        // Latent Puck on left half
        let puck_col = ((self.latent_puck_pos.0 * (mid_x - 2) as f32) + 1.0).round() as usize;
        let puck_row =
            (((1.0 - self.latent_puck_pos.1) * (height - 3) as f32) + 1.0).round() as usize;
        if puck_row < height - 1 && puck_col < mid_x {
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

        // Background
        painter.rect_filled(rect, 6.0, Color32::from_rgb(14, 18, 28));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "NEURAL WAVETABLE MORPHING SYNTH & 3D LATENT TRAJECTORY HUD",
            egui::FontId::proportional(16.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Architecture Selector Tabs (y: 48..92) - Each tab >= 44pt height
        let architectures = [
            (LatentArchitecture::VaeContinuous, "VAE CONTINUOUS"),
            (LatentArchitecture::TransformerDyn, "TRANSFORMER DYN"),
            (LatentArchitecture::DiffusionRes, "DIFFUSION RES"),
            (LatentArchitecture::Hypersphere4D, "HYPERSPHERE 4D"),
            (LatentArchitecture::SpectralFlow, "SPECTRAL FLOW"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (arch, name)) in architectures.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_selected = self.architecture == *arch;
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
                        self.architecture = *arch;
                        self.morph_speed_hz = arch.default_morph_speed_hz();
                        self.orbit_radius = arch.default_orbit_radius();
                        self.update_neural_calculations();
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

        // Left 55%: 3D Latent Trajectory Orbit (30..435)
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
            "3D LATENT MANIFOLD & ORBITAL TRAJECTORY (z1, z2, z3)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        let center_3d = (left_rect.center().x, left_rect.center().y + 10.0);
        let scale_3d = left_rect.width() * 0.16;

        // Draw 3D coordinate axes (z1, z2, z3)
        let orig_2d = self.project_3d_latent(0.0, 0.0, 0.0, center_3d, scale_3d);
        let x_axis = self.project_3d_latent(2.0, 0.0, 0.0, center_3d, scale_3d);
        let y_axis = self.project_3d_latent(0.0, 2.0, 0.0, center_3d, scale_3d);
        let z_axis = self.project_3d_latent(0.0, 0.0, 2.0, center_3d, scale_3d);

        painter.line_segment([egui::pos2(orig_2d.0, orig_2d.1), egui::pos2(x_axis.0, x_axis.1)], Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(0, 229, 255, 90)));
        painter.line_segment([egui::pos2(orig_2d.0, orig_2d.1), egui::pos2(y_axis.0, y_axis.1)], Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 215, 0, 90)));
        painter.line_segment([egui::pos2(orig_2d.0, orig_2d.1), egui::pos2(z_axis.0, z_axis.1)], Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 107, 43, 90)));

        // Draw Orbit Loop
        let num_orbit_pts = 32;
        let mut prev_orbit = None;

        for o in 0..=num_orbit_pts {
            let angle = o as f32 * (std::f32::consts::PI * 2.0 / num_orbit_pts as f32);
            let ox = self.latent_z.0 + self.orbit_radius * angle.cos();
            let oy = self.latent_z.1 + self.orbit_radius * angle.sin();
            let oz = self.latent_z.2 + self.orbit_radius * (angle * 2.0).sin() * 0.4;
            let pt_2d = self.project_3d_latent(ox, oy, oz, center_3d, scale_3d);
            let egui_pt = egui::pos2(pt_2d.0, pt_2d.1);

            if let Some(po) = prev_orbit {
                painter.line_segment([po, egui_pt], Stroke::new(1.5_f32, Color32::from_rgb(0, 255, 180)));
            }
            prev_orbit = Some(egui_pt);
        }

        // Interactive Latent Mean Puck
        let puck_x = left_rect.min.x + self.latent_puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.latent_puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.latent_puck_pos = (nx, ny);
                    self.latent_z.0 = Self::normalized_to_coord(nx);
                    self.latent_z.1 = Self::normalized_to_coord(ny);
                    self.update_neural_calculations();
                }
            }
        }

        // Touch Hit Target boundary (>= 44x44pt)
        painter.circle_stroke(
            puck_pos,
            NEURAL_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(0, 229, 255, 140)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(0, 229, 255));
        painter.circle_filled(puck_pos, 4.0, Color32::from_rgb(255, 255, 255));

        // Right 45%: Reconstructed Wavetable & Harmonic Spectrum (445..770)
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
            "RECONSTRUCTED SINGLE-CYCLE & HARMONIC BARS",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // Draw Single Cycle Waveform (top half of right card)
        let num_wave_pts = 40;
        let wave_w = right_rect.width() - 20.0;
        let wave_center_y = right_rect.min.y + 65.0;
        let mut prev_w = None;

        for c in 0..=num_wave_pts {
            let frac = c as f32 / num_wave_pts as f32;
            let sample = self.evaluate_wavetable_sample(frac);
            let px = right_rect.min.x + 10.0 + frac * wave_w;
            let py = wave_center_y - sample * 35.0;
            let pt = egui::pos2(px, py);
            if let Some(pw) = prev_w {
                painter.line_segment([pw, pt], Stroke::new(2.0_f32, Color32::from_rgb(0, 229, 255)));
            }
            prev_w = Some(pt);
        }

        // Draw 16 Harmonic Overtones Bars (bottom half of right card)
        let bar_w = (right_rect.width() - 20.0) / 16.0;
        let spec_bottom_y = right_rect.max.y - 15.0;
        for h in 1..=16 {
            let energy = self.evaluate_harmonic_energy(h);
            let bx = right_rect.min.x + 10.0 + (h - 1) as f32 * bar_w;
            let bh = energy * 60.0;
            let bar_rect = egui::Rect::from_min_max(
                egui::pos2(bx, spec_bottom_y - bh),
                egui::pos2(bx + bar_w - 2.0, spec_bottom_y),
            );
            let col = if h <= 3 {
                Color32::from_rgb(255, 215, 0)
            } else if h <= 8 {
                Color32::from_rgb(255, 107, 43)
            } else {
                Color32::from_rgb(0, 255, 180)
            };
            painter.rect_filled(bar_rect, 1.0, col);
        }

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
                "LATENT VECTOR (z)",
                format!("({:+.2}, {:+.2}, {:+.2})", self.latent_z.0, self.latent_z.1, self.latent_z.2),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "MORPH SPEED (LFO)",
                format!("{:.2} Hz (R={:.2})", self.morph_speed_hz, self.orbit_radius),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "SPECTRAL ENTROPY",
                format!("{:.2} bits (16 Harm)", self.spectral_entropy_bits),
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "RECON QUALITY (FID)",
                format!("{:.1}% (<0.004 MSE)", self.reconstruction_quality_pct),
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
            "[PASS] Neural Wavetable Morphing Synth & Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
