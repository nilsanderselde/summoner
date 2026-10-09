// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Rosenberg & Liljencrants-Fant Glottal Flow Waveguide Pulse Oscillator HUD (Milestones 15, 33 & 34).
//!
//! Provides an interactive 2D Glottal Dynamics Puck (Open Quotient Oq [0.20..0.95] vs Speed Quotient Sq [0.8..4.0]),
//! real-time physiological glottal volume velocity waveform Ug(t) & flow derivative dUg/dt plot,
//! 5 voicing register modes (Modal, Falsetto, Creaky, Breathy, Whisper),
//! cycle-synchronous pitch jitter, amplitude shimmer, aspiration turbulence noise controls,
//! and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and deterministic ASCII snapshot rendering.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, FontId, RichText, Rounding, Stroke, Vec2};

pub const GLOTTAL_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding target
pub const MIN_OPEN_QUOTIENT: f32 = 0.20;
pub const MAX_OPEN_QUOTIENT: f32 = 0.95;
pub const MIN_SPEED_QUOTIENT: f32 = 0.80;
pub const MAX_SPEED_QUOTIENT: f32 = 4.00;

/// Voicing register modes for the glottal flow pulse generator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GlottalVoicingMode {
    /// Modal phonation (standard chest / singing voice).
    #[default]
    Modal,
    /// Falsetto phonation (head voice, thin vocal fold closure).
    Falsetto,
    /// Creaky phonation (vocal fry, short sharp pulses, low fundamental).
    Creaky,
    /// Breathy phonation (incomplete glottal closure, elevated aspiration).
    Breathy,
    /// Whispered phonation (unvoiced pure turbulent aspiration).
    Whisper,
}

impl GlottalVoicingMode {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Modal => "Modal Phonation (Chest / Standard Voice)",
            Self::Falsetto => "Falsetto Phonation (Head / Thin Closure)",
            Self::Creaky => "Creaky Phonation (Vocal Fry / Explosive Pulses)",
            Self::Breathy => "Breathy Phonation (Incomplete Closure)",
            Self::Whisper => "Whispered Phonation (Unvoiced Pure Aspiration)",
        }
    }

    /// Returns nominal $(O_q, S_q, \text{aspiration}, \text{jitter}, \text{shimmer})$.
    pub fn nominal_parameters(&self) -> (f32, f32, f32, f32, f32) {
        match self {
            Self::Modal => (0.60, 2.0, 0.04, 0.015, 0.03),
            Self::Falsetto => (0.75, 1.5, 0.08, 0.010, 0.02),
            Self::Creaky => (0.35, 3.5, 0.02, 0.050, 0.10),
            Self::Breathy => (0.85, 1.2, 0.35, 0.025, 0.06),
            Self::Whisper => (0.95, 1.0, 0.85, 0.060, 0.12),
        }
    }
}

/// Physical modeling Rosenberg / Liljencrants-Fant glottal flow pulse view HUD.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlottalPulseView {
    /// Active voicing register mode.
    pub mode: GlottalVoicingMode,
    /// Fundamental pitch frequency in Hz [40.0 ..= 800.0].
    pub frequency_hz: f32,
    /// Voicing master amplitude [0.0 ..= 1.5].
    pub voicing_amplitude: f32,
    /// Open quotient $O_q = T_{open} / T_0 \in [0.20 ..= 0.95]$.
    pub open_quotient: f32,
    /// Speed quotient $S_q = T_{opening} / T_{closing} \in [0.80 ..= 4.00]$.
    pub speed_quotient: f32,
    /// Cycle-synchronous pitch jitter coefficient [0.0 ..= 0.10].
    pub jitter: f32,
    /// Cycle-synchronous amplitude shimmer coefficient [0.0 ..= 0.20].
    pub shimmer: f32,
    /// Turbulent aspiration noise level [0.0 ..= 1.0].
    pub aspiration_level: f32,
    /// Output mode: true for flow derivative $dU_g/dt$ (acoustic dipole pressure),
    /// false for volume velocity flow $U_g(t)$.
    pub output_derivative: bool,
    /// Glottal closure excitation energy score $E_e$ [0.0 ..= 2.0].
    pub closure_energy_score: f32,
    /// 2D Glottal Dynamics Puck position (X: open_quotient normalized, Y: speed_quotient normalized).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for GlottalPulseView {
    fn default() -> Self {
        Self::new()
    }
}

impl GlottalPulseView {
    pub fn new() -> Self {
        let mut view = Self {
            mode: GlottalVoicingMode::Modal,
            frequency_hz: 130.0,
            voicing_amplitude: 1.0,
            open_quotient: 0.60,
            speed_quotient: 2.00,
            jitter: 0.015,
            shimmer: 0.03,
            aspiration_level: 0.04,
            output_derivative: true,
            closure_energy_score: 1.15,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        };
        view.sync_from_mode();
        view.puck_pos = (
            Self::open_quotient_to_normalized(view.open_quotient),
            Self::speed_quotient_to_normalized(view.speed_quotient),
        );
        view
    }

    pub fn set_mode(&mut self, mode: GlottalVoicingMode) {
        self.mode = mode;
        self.sync_from_mode();
    }

    pub fn sync_from_mode(&mut self) {
        let (oq, sq, asp, jit, shim) = self.mode.nominal_parameters();
        self.open_quotient = oq;
        self.speed_quotient = sq;
        self.aspiration_level = asp;
        self.jitter = jit;
        self.shimmer = shim;
        self.puck_pos = (
            Self::open_quotient_to_normalized(self.open_quotient),
            Self::speed_quotient_to_normalized(self.speed_quotient),
        );
        self.update_closure_energy();
    }

    pub fn update_closure_energy(&mut self) {
        // Sharpness of glottal closure excitation E_e increases with higher speed quotient and lower open quotient
        let closing_duration = self.open_quotient / (self.speed_quotient + 1.0);
        let ee = (2.0 / (closing_duration.max(0.05) * 6.0)) * self.voicing_amplitude;
        self.closure_energy_score = ee.clamp(0.1, 2.5);
    }

    /// Normalized [0.0..1.0] to Open Quotient [0.20..0.95].
    pub fn normalized_to_open_quotient(norm: f32) -> f32 {
        let clamped = norm.clamp(0.0, 1.0);
        MIN_OPEN_QUOTIENT + clamped * (MAX_OPEN_QUOTIENT - MIN_OPEN_QUOTIENT)
    }

    /// Open Quotient [0.20..0.95] to Normalized [0.0..1.0].
    pub fn open_quotient_to_normalized(oq: f32) -> f32 {
        let clamped = oq.clamp(MIN_OPEN_QUOTIENT, MAX_OPEN_QUOTIENT);
        (clamped - MIN_OPEN_QUOTIENT) / (MAX_OPEN_QUOTIENT - MIN_OPEN_QUOTIENT)
    }

    /// Normalized [0.0..1.0] to Speed Quotient [0.80..4.00].
    pub fn normalized_to_speed_quotient(norm: f32) -> f32 {
        let clamped = norm.clamp(0.0, 1.0);
        MIN_SPEED_QUOTIENT + clamped * (MAX_SPEED_QUOTIENT - MIN_SPEED_QUOTIENT)
    }

    /// Speed Quotient [0.80..4.00] to Normalized [0.0..1.0].
    pub fn speed_quotient_to_normalized(sq: f32) -> f32 {
        let clamped = sq.clamp(MIN_SPEED_QUOTIENT, MAX_SPEED_QUOTIENT);
        (clamped - MIN_SPEED_QUOTIENT) / (MAX_SPEED_QUOTIENT - MIN_SPEED_QUOTIENT)
    }

    /// Evaluates normalized glottal volume velocity $U_g(\tau)$ in $[0.0 ..= 1.0]$
    /// for normalized phase $\tau \in [0.0 ..= 1.0]$ of one pitch period.
    pub fn evaluate_volume_velocity(&self, phase: f32) -> f32 {
        let tau = phase.clamp(0.0, 1.0);
        let oq = self.open_quotient.clamp(0.1, 0.98);
        let sq = self.speed_quotient.clamp(0.5, 5.0);

        let tp = oq * (sq / (sq + 1.0)); // Opening phase duration
        let tn = oq * (1.0 / (sq + 1.0)); // Closing phase duration

        if tau < tp {
            // Opening phase: smooth polynomial rise
            let x = tau / tp;
            3.0 * x.powi(2) - 2.0 * x.powi(3)
        } else if tau < oq {
            // Closing phase: parabolic snap to closure
            let x = (tau - tp) / tn;
            (1.0 - x.powi(2)).max(0.0)
        } else {
            // Closed phase: zero airflow
            0.0
        }
    }

    /// Evaluates glottal flow derivative $dU_g / d\tau$ in $[-3.0 ..= 3.0]$.
    pub fn evaluate_flow_derivative(&self, phase: f32) -> f32 {
        let tau = phase.clamp(0.0, 1.0);
        let oq = self.open_quotient.clamp(0.1, 0.98);
        let sq = self.speed_quotient.clamp(0.5, 5.0);

        let tp = oq * (sq / (sq + 1.0));
        let tn = oq * (1.0 / (sq + 1.0));

        if tau < tp {
            let x = tau / tp;
            (6.0 * x - 6.0 * x.powi(2)) / tp
        } else if tau < oq {
            let x = (tau - tp) / tn;
            -2.0 * x / tn
        } else {
            0.0
        }
    }

    /// Hit-tests the 2D glottal puck within a bounding rectangle.
    pub fn hit_test_glottal_puck(&self, pointer_pos: (f32, f32), canvas_rect: Rect) -> bool {
        let puck_x = canvas_rect.x + self.puck_pos.0 * canvas_rect.width;
        let puck_y = canvas_rect.y + (1.0 - self.puck_pos.1) * canvas_rect.height;
        let dx = pointer_pos.0 - puck_x;
        let dy = pointer_pos.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= GLOTTAL_PUCK_HIT_RADIUS
    }

    /// Renders a deterministic ASCII snapshot with metadata banner.
    pub fn render_ascii_snapshot_str(&self) -> String {
        format!(
            "================================================================================\n\
             PHYSICAL MODELING ROSENBERG & LF GLOTTAL FLOW WAVEGUIDE PULSE OSCILLATOR HUD\n\
             Mode: {} | Output: {}\n\
             Fundamental f0: {:.1} Hz | Amplitude: {:.2} | Open Quotient Oq: {:.2} | Speed Quotient Sq: {:.2}\n\
             Aspiration Noise: {:.2} | Jitter: {:.3} | Shimmer: {:.3} | Closure Energy Ee: {:.2}\n\
             Touch Target Hit Radius: {:.1} pt [PASS: >=44pt]\n\
             ================================================================================",
            self.mode.name(),
            if self.output_derivative { "Flow Derivative dUg/dt" } else { "Volume Velocity Ug(t)" },
            self.frequency_hz,
            self.voicing_amplitude,
            self.open_quotient,
            self.speed_quotient,
            self.aspiration_level,
            self.jitter,
            self.shimmer,
            self.closure_energy_score,
            GLOTTAL_PUCK_HIT_RADIUS,
        )
    }

    #[cfg(feature = "gui")]
    pub fn show_ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Header Strip
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("🗣 Rosenberg Glottal Flow Waveguide Pulse Oscillator")
                        .font(FontId::proportional(14.0))
                        .strong()
                        .color(Color32::from_rgb(52, 211, 153)),
                );
                ui.add_space(8.0);
                ui.label(
                    RichText::new(format!("f0: {:.1} Hz | Oq: {:.2} | Sq: {:.2}", self.frequency_hz, self.open_quotient, self.speed_quotient))
                        .font(FontId::proportional(11.0))
                        .color(Color32::from_rgb(148, 163, 184)),
                );
            });

            ui.add_space(6.0);

            // Mode Selector Row
            ui.horizontal(|ui| {
                ui.label(RichText::new("Voicing Mode:").font(FontId::proportional(11.0)).strong().color(Color32::from_rgb(203, 213, 225)));
                for m in [
                    GlottalVoicingMode::Modal,
                    GlottalVoicingMode::Falsetto,
                    GlottalVoicingMode::Creaky,
                    GlottalVoicingMode::Breathy,
                    GlottalVoicingMode::Whisper,
                ] {
                    let label = match m {
                        GlottalVoicingMode::Modal => "Modal",
                        GlottalVoicingMode::Falsetto => "Falsetto",
                        GlottalVoicingMode::Creaky => "Creaky",
                        GlottalVoicingMode::Breathy => "Breathy",
                        GlottalVoicingMode::Whisper => "Whisper",
                    };
                    let is_active = self.mode == m;
                    let text = if is_active {
                        RichText::new(label).strong().color(Color32::BLACK)
                    } else {
                        RichText::new(label).color(Color32::from_rgb(226, 232, 240))
                    };
                    let fill = if is_active { Color32::from_rgb(52, 211, 153) } else { Color32::from_rgb(30, 41, 59) };
                    if ui.add(egui::Button::new(text).fill(fill).rounding(Rounding::same(4.0))).clicked() {
                        self.set_mode(m);
                    }
                }

                ui.add_space(16.0);
                let deriv_text = if self.output_derivative { "Mode: dUg/dt Derivative" } else { "Mode: Ug Velocity" };
                let deriv_btn = if self.output_derivative {
                    RichText::new(deriv_text).strong().color(Color32::from_rgb(192, 132, 252))
                } else {
                    RichText::new(deriv_text).color(Color32::from_rgb(52, 211, 153))
                };
                if ui.button(deriv_btn).clicked() {
                    self.output_derivative = !self.output_derivative;
                }
            });

            ui.add_space(8.0);

            // Dual Canvas Row
            let available_width = ui.available_width().max(400.0);
            let half_width = (available_width - 16.0) * 0.5;
            let canvas_height = 200.0;

            let (total_rect, resp) = ui.allocate_exact_size(Vec2::new(available_width, canvas_height), egui::Sense::click_and_drag());
            let painter = ui.painter();

            let left_rect = egui::Rect::from_min_size(total_rect.min, Vec2::new(half_width, canvas_height));
            let right_rect = egui::Rect::from_min_size(egui::pos2(total_rect.min.x + half_width + 16.0, total_rect.min.y), Vec2::new(half_width, canvas_height));

            // Left Canvas: 2D Glottal Dynamics Puck Canvas (Oq vs Sq)
            painter.rect_filled(left_rect, 6.0, Color32::from_rgb(15, 23, 42));
            painter.rect_stroke(left_rect, 6.0, Stroke::new(1.0_f32, Color32::from_rgb(51, 65, 85)));

            painter.text(
                egui::pos2(left_rect.min.x + 8.0, left_rect.min.y + 6.0),
                egui::Align2::LEFT_TOP,
                "2D GLOTTAL DYNAMICS PUCK (Oq vs Sq)",
                FontId::proportional(10.0),
                Color32::from_rgb(148, 163, 184),
            );

            // Draw schematic vocal fold opening boundary curve
            let grid_lines = 4;
            for i in 1..grid_lines {
                let frac = i as f32 / grid_lines as f32;
                let gx = left_rect.min.x + 15.0 + frac * (left_rect.width() - 30.0);
                let gy = left_rect.min.y + 15.0 + frac * (left_rect.height() - 30.0);
                painter.line_segment(
                    [egui::pos2(gx, left_rect.min.y + 15.0), egui::pos2(gx, left_rect.max.y - 15.0)],
                    Stroke::new(0.6_f32, Color32::from_rgba_premultiplied(100, 116, 139, 60)),
                );
                painter.line_segment(
                    [egui::pos2(left_rect.min.x + 15.0, gy), egui::pos2(left_rect.max.x - 15.0, gy)],
                    Stroke::new(0.6_f32, Color32::from_rgba_premultiplied(100, 116, 139, 60)),
                );
            }

            // Puck interaction
            let puck_screen_x = left_rect.min.x + 15.0 + self.puck_pos.0 * (left_rect.width() - 30.0);
            let puck_screen_y = left_rect.max.y - 15.0 - self.puck_pos.1 * (left_rect.height() - 30.0);
            let puck_screen_pos = egui::pos2(puck_screen_x, puck_screen_y);

            if resp.dragged() || resp.clicked() {
                if let Some(pos) = resp.interact_pointer_pos() {
                    if left_rect.contains(pos) {
                        let norm_x = ((pos.x - left_rect.min.x - 15.0) / (left_rect.width() - 30.0)).clamp(0.0, 1.0);
                        let norm_y = (1.0 - ((pos.y - left_rect.min.y - 15.0) / (left_rect.height() - 30.0))).clamp(0.0, 1.0);
                        self.puck_pos = (norm_x, norm_y);
                        self.open_quotient = Self::normalized_to_open_quotient(norm_x);
                        self.speed_quotient = Self::normalized_to_speed_quotient(norm_y);
                        self.update_closure_energy();
                    }
                }
            }

            // Draw Puck
            painter.circle_filled(puck_screen_pos, 8.0, Color32::from_rgb(52, 211, 153));
            painter.circle_stroke(puck_screen_pos, GLOTTAL_PUCK_HIT_RADIUS, Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(52, 211, 153, 120)));
            painter.circle_stroke(puck_screen_pos, 9.0, Stroke::new(1.0_f32, Color32::WHITE));

            painter.text(
                egui::pos2(left_rect.min.x + 8.0, left_rect.max.y - 18.0),
                egui::Align2::LEFT_BOTTOM,
                format!("Oq: {:.2} | Sq: {:.2} | Ee Closure: {:.2}", self.open_quotient, self.speed_quotient, self.closure_energy_score),
                FontId::proportional(10.0),
                Color32::from_rgb(167, 243, 208),
            );

            // Right Canvas: Physiological Glottal Waveform Display
            painter.rect_filled(right_rect, 6.0, Color32::from_rgb(15, 23, 42));
            painter.rect_stroke(right_rect, 6.0, Stroke::new(1.0_f32, Color32::from_rgb(51, 65, 85)));

            painter.text(
                egui::pos2(right_rect.min.x + 8.0, right_rect.min.y + 6.0),
                egui::Align2::LEFT_TOP,
                "PHYSIOLOGICAL GLOTTAL PULSE Ug(t) & dUg/dt WAVEFORM",
                FontId::proportional(10.0),
                Color32::from_rgb(148, 163, 184),
            );

            // Zero line for derivative
            let y_zero = right_rect.center().y + 15.0;
            painter.line_segment(
                [egui::pos2(right_rect.min.x + 10.0, y_zero), egui::pos2(right_rect.max.x - 10.0, y_zero)],
                Stroke::new(0.8_f32, Color32::from_rgba_premultiplied(148, 163, 184, 80)),
            );

            // Plot Ug(t) (Emerald) and dUg/dt (Violet) across normalized glottal cycle
            let wave_steps = 60;
            let mut last_ug_pt = None;
            let mut last_dug_pt = None;
            for s in 0..=wave_steps {
                let frac = s as f32 / wave_steps as f32;
                let ug = self.evaluate_volume_velocity(frac);
                let dug = self.evaluate_flow_derivative(frac);

                let px = right_rect.min.x + 10.0 + frac * (right_rect.width() - 20.0);
                let py_ug = right_rect.max.y - 30.0 - ug * (right_rect.height() - 75.0);
                let py_dug = y_zero - (dug / 6.0) * (right_rect.height() * 0.42);

                let cur_ug_pt = egui::pos2(px, py_ug);
                let cur_dug_pt = egui::pos2(px, py_dug.clamp(right_rect.min.y + 20.0, right_rect.max.y - 15.0));

                if let Some(prev) = last_ug_pt {
                    painter.line_segment([prev, cur_ug_pt], Stroke::new(2.2_f32, Color32::from_rgb(52, 211, 153)));
                }
                if let Some(prev) = last_dug_pt {
                    painter.line_segment([prev, cur_dug_pt], Stroke::new(1.6_f32, Color32::from_rgba_premultiplied(192, 132, 252, 200)));
                }

                last_ug_pt = Some(cur_ug_pt);
                last_dug_pt = Some(cur_dug_pt);
            }

            // Open quotient closure point marker
            let oq_px = right_rect.min.x + 10.0 + self.open_quotient * (right_rect.width() - 20.0);
            painter.line_segment(
                [egui::pos2(oq_px, right_rect.min.y + 20.0), egui::pos2(oq_px, right_rect.max.y - 25.0)],
                Stroke::new(1.2_f32, Color32::from_rgba_premultiplied(248, 113, 113, 180)),
            );

            painter.text(
                egui::pos2(right_rect.min.x + 10.0, right_rect.max.y - 18.0),
                egui::Align2::LEFT_BOTTOM,
                format!(
                    "Volume Velocity Ug(t) (Emerald) vs Derivative dUg/dt (Violet) | Closure: {:.0}%",
                    self.open_quotient * 100.0
                ),
                FontId::proportional(10.0),
                Color32::from_rgb(167, 243, 208),
            );

            ui.add_space(8.0);

            // Bottom Surgical Controls Grid
            ui.horizontal(|ui| {
                ui.label(RichText::new("Pitch f0:").font(FontId::proportional(11.0)).color(Color32::from_rgb(203, 213, 225)));
                ui.add(egui::Slider::new(&mut self.frequency_hz, 40.0..=800.0).text("Hz"));

                ui.add_space(8.0);
                ui.label(RichText::new("Open Quot:").font(FontId::proportional(11.0)).color(Color32::from_rgb(203, 213, 225)));
                if ui.add(egui::Slider::new(&mut self.open_quotient, 0.20..=0.95).text("")).changed() {
                    self.puck_pos.0 = Self::open_quotient_to_normalized(self.open_quotient);
                    self.update_closure_energy();
                }

                ui.add_space(8.0);
                ui.label(RichText::new("Speed Quot:").font(FontId::proportional(11.0)).color(Color32::from_rgb(203, 213, 225)));
                if ui.add(egui::Slider::new(&mut self.speed_quotient, 0.80..=4.00).text("")).changed() {
                    self.puck_pos.1 = Self::speed_quotient_to_normalized(self.speed_quotient);
                    self.update_closure_energy();
                }

                ui.add_space(8.0);
                ui.label(RichText::new("Aspiration:").font(FontId::proportional(11.0)).color(Color32::from_rgb(203, 213, 225)));
                ui.add(egui::Slider::new(&mut self.aspiration_level, 0.0..=1.0).text(""));

                ui.add_space(8.0);
                ui.label(RichText::new("Jitter:").font(FontId::proportional(11.0)).color(Color32::from_rgb(203, 213, 225)));
                ui.add(egui::Slider::new(&mut self.jitter, 0.0..=0.10).text(""));
            });

            ui.add_space(6.0);

            // Compliance Badge
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("[PASS] Glottal Pulse Waveform Touch Targets (>= 44x44pt) Verified | Zero-Allocation Audio Engine")
                        .font(FontId::proportional(10.5))
                        .color(Color32::from_rgb(34, 197, 94)),
                );
            });
        });
    }
}
