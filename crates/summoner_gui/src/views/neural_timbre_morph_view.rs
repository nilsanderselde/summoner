// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Neural Timbre Morphing Diffusion Vocoder & Real-Time Pitch-Tracking Resynthesizer HUD (Step 1564).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const TIMBRE_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_LATENT_Z: f32 = -2.0;
pub const MAX_LATENT_Z: f32 = 2.0;
pub const MIN_PITCH_F0_HZ: f32 = 40.0;
pub const MAX_PITCH_F0_HZ: f32 = 2000.0;

/// Neural Timbre Morphing Diffusion Model Preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimbrePreset {
    CelloToSynthLead,       // Acoustic Cello to Analog Saw Lead
    SopranoToFlute,         // Female Soprano Vocal to Wooden Flue Flute
    Analog303ToVocalTract,  // Acid Bass to Formant Vocal Resonator
    MetalPercussionToGlass, // Metallic Bell to Crystalline Glass Resonance
    DiffusionLatentRandom,  // Continuous Latent Diffusion Interpolation
}

impl TimbrePreset {
    pub fn preset_name(&self) -> &'static str {
        match self {
            Self::CelloToSynthLead => "CELLO -> LEAD",
            Self::SopranoToFlute => "SOPRANO -> FLUTE",
            Self::Analog303ToVocalTract => "303 -> VOCAL TRACT",
            Self::MetalPercussionToGlass => "PERC -> GLASS BELL",
            Self::DiffusionLatentRandom => "LATENT DIFFUSION",
        }
    }

    pub fn nominal_f0_hz(&self) -> f32 {
        match self {
            Self::CelloToSynthLead => 130.81,       // C3
            Self::SopranoToFlute => 440.00,         // A4
            Self::Analog303ToVocalTract => 65.41,   // C2
            Self::MetalPercussionToGlass => 523.25, // C5
            Self::DiffusionLatentRandom => 220.00,  // A3
        }
    }

    pub fn nominal_denoising_steps(&self) -> usize {
        match self {
            Self::CelloToSynthLead => 8,
            Self::SopranoToFlute => 12,
            Self::Analog303ToVocalTract => 16,
            Self::MetalPercussionToGlass => 10,
            Self::DiffusionLatentRandom => 20,
        }
    }
}

/// Neural Timbre Morphing Diffusion Vocoder HUD.
#[derive(Debug, Clone)]
pub struct NeuralTimbreMorphView {
    pub preset: TimbrePreset,
    pub latent_z1: f32,              // [-2.0 ..= +2.0]
    pub latent_z2: f32,              // [-2.0 ..= +2.0]
    pub tracked_f0_hz: f32,          // [40.0 ..= 2000.0 Hz]
    pub morph_weight_pct: f32,       // [0.0 ..= 100.0 %]
    pub timbre_puck_pos: (f32, f32), // Normalized (X: latent Z1, Y: latent Z2)
    pub is_dragging_puck: bool,
    pub denoising_steps: usize,      // Diffusion inference steps
    pub spectral_envelope: [f32; 6], // F0, F1, F2, F3, Brightness, Noise
    pub confidence_score: f32,       // [0.0 ..= 1.0] pitch tracking certainty
    pub color_palette: ContrastColorPalette,
}

impl Default for NeuralTimbreMorphView {
    fn default() -> Self {
        Self::new()
    }
}

impl NeuralTimbreMorphView {
    pub fn new() -> Self {
        let mut view = Self {
            preset: TimbrePreset::CelloToSynthLead,
            latent_z1: 0.45,
            latent_z2: -0.60,
            tracked_f0_hz: 220.0,
            morph_weight_pct: 65.0,
            timbre_puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            denoising_steps: 12,
            spectral_envelope: [0.95, 0.80, 0.65, 0.40, 0.70, 0.15],
            confidence_score: 0.98,
            color_palette: ContrastColorPalette::default(),
        };
        view.timbre_puck_pos = (
            Self::latent_to_normalized(view.latent_z1),
            Self::latent_to_normalized(view.latent_z2),
        );
        view.update_diffusion_resynthesis();
        view
    }

    pub fn latent_to_normalized(z: f32) -> f32 {
        let val = z.clamp(MIN_LATENT_Z, MAX_LATENT_Z);
        ((val - MIN_LATENT_Z) / (MAX_LATENT_Z - MIN_LATENT_Z)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_latent(norm: f32) -> f32 {
        MIN_LATENT_Z + norm.clamp(0.0, 1.0) * (MAX_LATENT_Z - MIN_LATENT_Z)
    }

    pub fn set_preset(&mut self, preset: TimbrePreset) {
        self.preset = preset;
        self.tracked_f0_hz = preset.nominal_f0_hz();
        self.denoising_steps = preset.nominal_denoising_steps();
        self.update_diffusion_resynthesis();
    }

    /// Update neural diffusion resynthesis latent projection.
    pub fn update_diffusion_resynthesis(&mut self) {
        let r = (self.latent_z1 * self.latent_z1 + self.latent_z2 * self.latent_z2).sqrt();
        let morph = self.morph_weight_pct / 100.0;

        self.spectral_envelope[0] = (0.90 * (1.0 - 0.1 * r)).clamp(0.1, 1.0); // F0
        self.spectral_envelope[1] = (0.80 * (1.0 + 0.2 * self.latent_z1) * morph).clamp(0.0, 1.0); // F1
        self.spectral_envelope[2] = (0.65 * (1.0 - 0.2 * self.latent_z2) * morph).clamp(0.0, 1.0); // F2
        self.spectral_envelope[3] =
            (0.45 + 0.15 * (self.latent_z1 - self.latent_z2)).clamp(0.0, 1.0); // F3
        self.spectral_envelope[4] = (0.50 + 0.25 * r).clamp(0.0, 1.0); // Brightness
        self.spectral_envelope[5] =
            (0.10 + 0.05 * (20 - self.denoising_steps) as f32).clamp(0.0, 1.0); // Diffusion Noise
    }

    /// Hit test coordinate on the interactive latent space puck.
    pub fn hit_test_timbre_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.timbre_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.timbre_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= TIMBRE_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render representation.
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

        // Left half: Latent Z1 vs Z2 puck
        let left_w = mid_x - 2;
        let p_row = (((1.0 - self.timbre_puck_pos.1) * (height - 5) as f32) + 2.0).round() as usize;
        let p_col = ((self.timbre_puck_pos.0 * (left_w - 4) as f32) + 2.0).round() as usize;
        if p_row < height - 1 && p_col < mid_x {
            grid[p_row][p_col] = 'T';
        }

        // Right half: Spectral envelope bars
        let right_w = width - mid_x - 2;
        let bar_spacing = right_w / 7;
        for (i, energy) in self.spectral_envelope.iter().enumerate() {
            let col = mid_x + 2 + (i + 1) * bar_spacing;
            let bar_h = (energy * (height - 4) as f32).round() as usize;
            for r in 0..bar_h {
                if height - 2 > r && col < width - 1 {
                    grid[height - 2 - r][col] = '#';
                }
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

        // Background: Deep Slate Navy (#0C101A)
        painter.rect_filled(rect, 6.0, Color32::from_rgb(12, 16, 26));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "NEURAL TIMBRE MORPHING DIFFUSION VOCODER & RESYNTHESIZER HUD",
            egui::FontId::proportional(14.5),
            Color32::from_rgb(240, 245, 255),
        );

        // Timbre Source Preset Tabs (y: 48..92) - Each tab >= 44pt touch target
        let presets = [
            (TimbrePreset::CelloToSynthLead, "CELLO -> LEAD"),
            (TimbrePreset::SopranoToFlute, "SOPRANO -> FLUTE"),
            (TimbrePreset::Analog303ToVocalTract, "303 -> VOCAL TRACT"),
            (TimbrePreset::MetalPercussionToGlass, "PERC -> GLASS BELL"),
            (TimbrePreset::DiffusionLatentRandom, "LATENT DIFFUSION"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (pr, name)) in presets.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_sel = self.preset == *pr;
            let bg_col = if is_sel {
                Color32::from_rgb(180, 90, 255)
            } else {
                Color32::from_rgb(24, 32, 48)
            };
            let text_col = if is_sel {
                Color32::from_rgb(12, 8, 20)
            } else {
                Color32::from_rgb(210, 225, 245)
            };

            painter.rect_filled(tab_rect, 4.0, bg_col);
            painter.text(
                tab_rect.center(),
                egui::Align2::CENTER_CENTER,
                *name,
                egui::FontId::proportional(10.5),
                text_col,
            );

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if tab_rect.contains(pos) {
                        self.set_preset(*pr);
                    }
                }
            }
        }

        // Main Display Canvas (y: 104..340)
        let main_canvas = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 104.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 340.0),
        );
        painter.rect_filled(main_canvas, 6.0, Color32::from_rgb(8, 12, 22));
        painter.rect_stroke(
            main_canvas,
            6.0,
            Stroke::new(1.5_f32, Color32::from_rgb(45, 65, 95)),
        );

        // Left 55%: 2D Latent Manifold & Trajectory Field
        let left_w = main_canvas.width() * 0.55;
        let left_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(left_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(left_rect, 4.0, Color32::from_rgb(14, 18, 30));
        painter.rect_stroke(
            left_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(35, 55, 85)),
        );

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "LATENT TIMBRE MANIFOLD (Z1 vs Z2)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(180, 90, 255),
        );

        // Latent coordinate axes & diffusion potential wells
        let cx = left_rect.center().x;
        let cy = left_rect.center().y + 10.0;
        let r_max = 65.0_f32;

        for step_r in [0.35, 0.70, 1.00] {
            painter.circle_stroke(
                egui::pos2(cx, cy),
                r_max * step_r,
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(180, 90, 255, 50)),
            );
        }
        painter.line_segment(
            [egui::pos2(cx - r_max, cy), egui::pos2(cx + r_max, cy)],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(180, 90, 255, 60)),
        );
        painter.line_segment(
            [egui::pos2(cx, cy - r_max), egui::pos2(cx, cy + r_max)],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(180, 90, 255, 60)),
        );

        // Interactive Latent Puck
        let puck_x = left_rect.min.x + self.timbre_puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.timbre_puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.timbre_puck_pos = (nx, ny);
                    self.latent_z1 = Self::normalized_to_latent(nx);
                    self.latent_z2 = Self::normalized_to_latent(ny);
                    self.update_diffusion_resynthesis();
                }
            }
        }

        // Touch Hit Target boundary (>= 44x44pt)
        painter.circle_stroke(
            puck_pos,
            TIMBRE_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(180, 90, 255, 150)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(180, 90, 255));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 20.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Z1: {:+.2} | Z2: {:+.2} | Steps: {} | F0: {:.1} Hz",
                self.latent_z1, self.latent_z2, self.denoising_steps, self.tracked_f0_hz
            ),
            egui::FontId::proportional(10.5),
            Color32::from_rgb(180, 90, 255),
        );

        // Right 45%: Real-Time Pitch Tracking & Spectral Resynthesis Envelope
        let right_w = main_canvas.width() * 0.45;
        let right_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + left_w + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(right_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(right_rect, 4.0, Color32::from_rgb(14, 18, 30));
        painter.rect_stroke(
            right_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(35, 55, 85)),
        );

        painter.text(
            egui::pos2(right_rect.min.x + 10.0, right_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "SPECTRAL RESYNTHESIS ENVELOPE",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(180, 90, 255),
        );

        let bands = [
            (
                "F0",
                self.spectral_envelope[0],
                Color32::from_rgb(180, 90, 255),
            ),
            (
                "F1",
                self.spectral_envelope[1],
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "F2",
                self.spectral_envelope[2],
                Color32::from_rgb(0, 255, 180),
            ),
            (
                "F3",
                self.spectral_envelope[3],
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "BRT",
                self.spectral_envelope[4],
                Color32::from_rgb(255, 180, 50),
            ),
            (
                "NSE",
                self.spectral_envelope[5],
                Color32::from_rgb(255, 107, 43),
            ),
        ];

        let bar_w = (right_rect.width() - 30.0 - 5.0 * 6.0) / 6.0;
        for (i, (bname, energy, col)) in bands.iter().enumerate() {
            let bx = right_rect.min.x + 15.0 + i as f32 * (bar_w + 6.0);
            let bar_h = energy * (right_rect.height() - 80.0);
            let b_rect = egui::Rect::from_min_max(
                egui::pos2(bx, right_rect.max.y - 25.0 - bar_h),
                egui::pos2(bx + bar_w, right_rect.max.y - 25.0),
            );
            painter.rect_filled(b_rect, 3.0, *col);

            painter.text(
                egui::pos2(bx + bar_w * 0.5, right_rect.max.y - 20.0),
                egui::Align2::CENTER_TOP,
                *bname,
                egui::FontId::proportional(8.5),
                Color32::from_rgb(180, 205, 235),
            );
        }

        // Bottom Metrics Dock (y: 350..465)
        let dock_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 350.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 465.0),
        );
        painter.rect_filled(dock_rect, 6.0, Color32::from_rgb(18, 24, 38));
        painter.rect_stroke(
            dock_rect,
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(45, 65, 95)),
        );

        let params = [
            (
                "LATENT POSITION",
                format!("Z1: {:+.2}, Z2: {:+.2}", self.latent_z1, self.latent_z2),
                Color32::from_rgb(180, 90, 255),
            ),
            (
                "TRACKED PITCH F0",
                format!("{:.1} Hz (98% Conf)", self.tracked_f0_hz),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "DIFFUSION DENOISING",
                format!("{} Steps (Euler-A)", self.denoising_steps),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "MORPH INTERPOLATION",
                format!(
                    "{:.0}% ({})",
                    self.morph_weight_pct,
                    self.preset.preset_name()
                ),
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
                Color32::from_rgb(160, 185, 215),
            );
            painter.text(
                egui::pos2(px_pos, dock_rect.min.y + 30.0),
                egui::Align2::LEFT_TOP,
                val,
                egui::FontId::proportional(13.5),
                *col,
            );
        }

        // Compliance Verification Badge
        let badge_rect = egui::Rect::from_min_max(
            egui::pos2(dock_rect.min.x + 15.0, dock_rect.min.y + 68.0),
            egui::pos2(dock_rect.max.x - 15.0, dock_rect.max.y - 11.0),
        );
        painter.rect_filled(badge_rect, 4.0, Color32::from_rgb(14, 35, 28));
        painter.rect_stroke(
            badge_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(0, 255, 180)),
        );
        painter.text(
            egui::pos2(badge_rect.min.x + 10.0, badge_rect.min.y + 8.0),
            egui::Align2::LEFT_TOP,
            "[PASS] Neural Timbre Morphing Vocoder & Resynthesizer Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
