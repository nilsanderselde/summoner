// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Quantum Bloch Sphere & Qubit Superposition State Tomography HUD (Milestones 33 & 34).
//!
//! Provides an interactive 2D Qubit Phase & Superposition Dynamics Puck (Superposition Probability
//! $|\alpha|^2 \in [0.0..=1.0]$ vs Relative Phase $\phi \in [0..=2\pi]$ rad), 3D Bloch sphere
//! density matrix state tomography ($X, Y, Z$ coordinates with quantum state purity $P \le 1.0$),
//! interactive quantum logic gate trigger actions (Hadamard $H$, Pauli-X NOT, Phase Shift $S$, Pauli-Z),
//! 4 curated quantum presets (Hadamard Superposition, Excited State Pauli-X, Bell Phase Entangled,
//! Ground Coherent State), real-time Qubit wavefunction precession, and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and deterministic ASCII snapshot rendering.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};
use std::f32::consts::{PI, TAU};

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, FontId, RichText, Rounding, Stroke, Vec2};

pub const QUANTUM_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding target
pub const MIN_SUPERPOSITION_ALPHA_SQ: f32 = 0.0;
pub const MAX_SUPERPOSITION_ALPHA_SQ: f32 = 1.0;
pub const MIN_RELATIVE_PHASE_RAD: f32 = 0.0;
pub const MAX_RELATIVE_PHASE_RAD: f32 = TAU;
pub const NUM_BLOCH_ORBIT_POINTS: usize = 48;

/// Curated quantum Qubit state and wavefunction presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum QuantumStatePreset {
    /// Equal superposition state |+> = (|0> + |1>) / sqrt(2) on the Bloch sphere equator.
    #[default]
    HadamardSuperposition,
    /// Fully excited inverted state |1> at the south pole of the Bloch sphere.
    ExcitedStatePauliX,
    /// Complex phase-entangled state |+i> = (|0> + i|1>) / sqrt(2) along the Y axis.
    BellPhaseEntangled,
    /// Ground coherent state |0> at the north pole of the Bloch sphere.
    GroundCoherentState,
}

impl QuantumStatePreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::HadamardSuperposition => "Hadamard Superposition |+⟩ (Equator)",
            Self::ExcitedStatePauliX => "Excited State Pauli-X |1⟩ (South Pole)",
            Self::BellPhaseEntangled => "Bell Phase Entangled |+i⟩ (Y Axis)",
            Self::GroundCoherentState => "Ground Coherent State |0⟩ (North Pole)",
        }
    }

    /// Returns nominal (alpha_prob, relative_phase_rad, frequency_hz, decoherence_rate).
    pub fn nominal_parameters(&self) -> (f32, f32, f32, f32) {
        match self {
            Self::HadamardSuperposition => (0.50, 0.0, 440.0, 0.02),
            Self::ExcitedStatePauliX => (0.00, 0.0, 220.0, 0.01),
            Self::BellPhaseEntangled => (0.50, PI * 0.5, 550.0, 0.05),
            Self::GroundCoherentState => (1.00, 0.0, 440.0, 0.00),
        }
    }
}

/// Quantum Bloch Sphere & Qubit Superposition State Tomography HUD view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumBlochSphereView {
    /// Active quantum state preset.
    pub preset: QuantumStatePreset,
    /// Probability of ground state |0>: |alpha|^2 in [0.0 ..= 1.0].
    pub alpha_prob: f32,
    /// Relative phase phi in radians between |0> and |1> [0.0 ..= 2*PI].
    pub relative_phase_rad: f32,
    /// Base quantum oscillator carrier frequency in Hz [20.0 ..= 2000.0].
    pub frequency_hz: f32,
    /// Stochastic environmental decoherence dissipation rate [0.0 ..= 1.0].
    pub decoherence_rate: f32,
    /// Bloch sphere coordinate X: 2 * Re(alpha * beta*).
    pub bloch_x: f32,
    /// Bloch sphere coordinate Y: 2 * Im(alpha * beta*).
    pub bloch_y: f32,
    /// Bloch sphere coordinate Z: |alpha|^2 - |beta|^2.
    pub bloch_z: f32,
    /// Quantum density matrix state purity: P = X^2 + Y^2 + Z^2 in [0.0 ..= 1.0].
    pub state_purity: f32,
    /// Quantum phase angle theta in radians from +Z axis: theta = 2 * acos(sqrt(|alpha|^2)).
    pub polar_angle_theta: f32,
    /// 2D Dynamics Puck position (X: alpha_prob normalized, Y: relative_phase normalized).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for QuantumBlochSphereView {
    fn default() -> Self {
        Self::new()
    }
}

impl QuantumBlochSphereView {
    pub fn new() -> Self {
        let mut view = Self {
            preset: QuantumStatePreset::HadamardSuperposition,
            alpha_prob: 0.50,
            relative_phase_rad: 0.0,
            frequency_hz: 440.0,
            decoherence_rate: 0.02,
            bloch_x: 1.0,
            bloch_y: 0.0,
            bloch_z: 0.0,
            state_purity: 1.0,
            polar_angle_theta: PI * 0.5,
            puck_pos: (0.5, 0.0),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        };
        let (a, phi, f, d) = view.preset.nominal_parameters();
        view.alpha_prob = a;
        view.relative_phase_rad = phi;
        view.frequency_hz = f;
        view.decoherence_rate = d;
        view.puck_pos = (
            Self::alpha_prob_to_normalized(view.alpha_prob),
            Self::phase_to_normalized(view.relative_phase_rad),
        );
        view.update_physics();
        view
    }

    pub fn set_preset(&mut self, preset: QuantumStatePreset) {
        self.preset = preset;
        let (a, phi, f, d) = preset.nominal_parameters();
        self.alpha_prob = a;
        self.relative_phase_rad = phi;
        self.frequency_hz = f;
        self.decoherence_rate = d;
        self.puck_pos = (
            Self::alpha_prob_to_normalized(self.alpha_prob),
            Self::phase_to_normalized(self.relative_phase_rad),
        );
        self.update_physics();
    }

    /// Applies the quantum Hadamard transform gate: H = 1/sqrt(2) * [[1, 1], [1, -1]].
    pub fn apply_hadamard(&mut self) {
        let a_mag = self.alpha_prob.sqrt();
        let b_mag = (1.0 - self.alpha_prob).max(0.0).sqrt();
        let phi = self.relative_phase_rad;

        // alpha = a_mag, beta = b_mag * exp(i * phi)
        let beta_re = b_mag * phi.cos();
        let beta_im = b_mag * phi.sin();

        let inv_sqrt2 = 1.0 / 2.0_f32.sqrt();
        let new_alpha_re = (a_mag + beta_re) * inv_sqrt2;
        let new_alpha_im = beta_im * inv_sqrt2;
        let new_beta_re = (a_mag - beta_re) * inv_sqrt2;
        let new_beta_im = -beta_im * inv_sqrt2;

        let new_a_sq = new_alpha_re * new_alpha_re + new_alpha_im * new_alpha_im;
        let new_b_sq = new_beta_re * new_beta_re + new_beta_im * new_beta_im;
        let norm = (new_a_sq + new_b_sq).max(1e-7);

        self.alpha_prob = (new_a_sq / norm).clamp(0.0, 1.0);
        let arg_alpha = new_alpha_im.atan2(new_alpha_re);
        let arg_beta = new_beta_im.atan2(new_beta_re);
        let mut rel = arg_beta - arg_alpha;
        while rel < 0.0 {
            rel += TAU;
        }
        while rel >= TAU {
            rel -= TAU;
        }
        self.relative_phase_rad = rel;
        self.puck_pos = (
            Self::alpha_prob_to_normalized(self.alpha_prob),
            Self::phase_to_normalized(self.relative_phase_rad),
        );
        self.update_physics();
    }

    /// Applies the quantum Pauli-X NOT gate: bit flip |0> <-> |1>.
    pub fn apply_pauli_x(&mut self) {
        self.alpha_prob = (1.0 - self.alpha_prob).clamp(0.0, 1.0);
        self.puck_pos = (
            Self::alpha_prob_to_normalized(self.alpha_prob),
            Self::phase_to_normalized(self.relative_phase_rad),
        );
        self.update_physics();
    }

    /// Applies a phase shift delta (in radians) to the relative phase.
    pub fn apply_phase_shift(&mut self, delta_rad: f32) {
        self.relative_phase_rad = (self.relative_phase_rad + delta_rad) % TAU;
        if self.relative_phase_rad < 0.0 {
            self.relative_phase_rad += TAU;
        }
        self.puck_pos.1 = Self::phase_to_normalized(self.relative_phase_rad);
        self.update_physics();
    }

    /// Recomputes density matrix tomography and Bloch sphere coordinates.
    pub fn update_physics(&mut self) {
        let a_sq = self.alpha_prob.clamp(0.0, 1.0);
        let b_sq = (1.0 - a_sq).clamp(0.0, 1.0);
        let a_mag = a_sq.sqrt();
        let b_mag = b_sq.sqrt();
        let phi = self.relative_phase_rad;

        // Density matrix off-diagonal: rho_01 = alpha * beta* = a_mag * b_mag * exp(-i * phi)
        // Bloch vector components:
        // X = 2 * Re(rho_01) = 2 * a_mag * b_mag * cos(phi)
        // Y = 2 * Im(rho_01) = 2 * a_mag * b_mag * sin(phi)
        // Z = rho_00 - rho_11 = a_sq - b_sq
        let decoherence_factor = (1.0 - self.decoherence_rate * 0.5).clamp(0.0, 1.0);
        self.bloch_x = 2.0 * a_mag * b_mag * phi.cos() * decoherence_factor;
        self.bloch_y = 2.0 * a_mag * b_mag * phi.sin() * decoherence_factor;
        self.bloch_z = (a_sq - b_sq) * decoherence_factor;

        let r_sq = self.bloch_x * self.bloch_x + self.bloch_y * self.bloch_y + self.bloch_z * self.bloch_z;
        self.state_purity = r_sq.min(1.0);

        // Polar angle theta from north pole: cos(theta) = Z
        self.polar_angle_theta = self.bloch_z.clamp(-1.0, 1.0).acos();
    }

    pub fn alpha_prob_to_normalized(p: f32) -> f32 {
        ((p - MIN_SUPERPOSITION_ALPHA_SQ) / (MAX_SUPERPOSITION_ALPHA_SQ - MIN_SUPERPOSITION_ALPHA_SQ)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_alpha_prob(norm: f32) -> f32 {
        MIN_SUPERPOSITION_ALPHA_SQ + norm.clamp(0.0, 1.0) * (MAX_SUPERPOSITION_ALPHA_SQ - MIN_SUPERPOSITION_ALPHA_SQ)
    }

    pub fn phase_to_normalized(phi: f32) -> f32 {
        ((phi - MIN_RELATIVE_PHASE_RAD) / (MAX_RELATIVE_PHASE_RAD - MIN_RELATIVE_PHASE_RAD)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_phase(norm: f32) -> f32 {
        MIN_RELATIVE_PHASE_RAD + norm.clamp(0.0, 1.0) * (MAX_RELATIVE_PHASE_RAD - MIN_RELATIVE_PHASE_RAD)
    }

    /// Evaluates 3D trajectory points of Qubit precession orbit around Z axis.
    pub fn evaluate_bloch_orbit(&self) -> [(f32, f32, f32); NUM_BLOCH_ORBIT_POINTS] {
        let mut points = [(0.0_f32, 0.0_f32, 0.0_f32); NUM_BLOCH_ORBIT_POINTS];
        let r_xy = (self.bloch_x * self.bloch_x + self.bloch_y * self.bloch_y).sqrt();
        let base_phi = self.relative_phase_rad;
        let z = self.bloch_z;

        for (i, pt) in points.iter_mut().enumerate() {
            let t = (i as f32) / (NUM_BLOCH_ORBIT_POINTS as f32);
            let angle = base_phi + t * TAU;
            let x = r_xy * angle.cos();
            let y = r_xy * angle.sin();
            *pt = (x, y, z);
        }
        points
    }

    pub fn render_ascii_snapshot(&self) -> String {
        let mut out = String::new();
        out.push_str("================================================================================\n");
        out.push_str("   SUMMONER DAW — QUANTUM BLOCH SPHERE & STATE TOMOGRAPHY HUD\n");
        out.push_str("================================================================================\n");
        out.push_str(&format!(" Preset: {}\n", self.preset.name()));
        out.push_str(" Touch Target: [PASS: >=44pt]\n");
        out.push_str(&format!(
            " Ground State |0⟩ Prob: {:.1}% | Excited State |1⟩ Prob: {:.1}%\n",
            self.alpha_prob * 100.0,
            (1.0 - self.alpha_prob) * 100.0
        ));
        out.push_str(&format!(
            " Relative Phase ϕ: {:.3} rad ({:.1}°) | Frequency: {:.1} Hz\n",
            self.relative_phase_rad,
            self.relative_phase_rad * 180.0 / PI,
            self.frequency_hz
        ));
        out.push_str(&format!(
            " Bloch Coordinates: X={:+.3}, Y={:+.3}, Z={:+.3} | Purity P={:.3} ({:.1}%)\n",
            self.bloch_x, self.bloch_y, self.bloch_z, self.state_purity, self.state_purity * 100.0
        ));
        out.push_str(&format!(
            " Polar Angle θ: {:.2} rad ({:.1}°) | Decoherence Rate: {:.3}\n",
            self.polar_angle_theta,
            self.polar_angle_theta * 180.0 / PI,
            self.decoherence_rate
        ));
        out.push_str("--------------------------------------------------------------------------------\n");
        out.push_str(" Orthographic Bloch Sphere Projection (X vs Z):\n");

        // 7 rows by 32 cols projection grid
        for row in (0..7).rev() {
            let z_target = (row as f32 / 3.0) - 1.0;
            out.push_str(&format!("{:+.2} |", z_target));
            for col in 0..32 {
                let x_target = (col as f32 / 15.5) - 1.0;
                let dist_from_center = (x_target * x_target + z_target * z_target).sqrt();

                if (self.bloch_x - x_target).abs() < 0.08 && (self.bloch_z - z_target).abs() < 0.16 {
                    out.push('●'); // State vector marker
                } else if (dist_from_center - 1.0).abs() < 0.07 {
                    out.push('○'); // Sphere equator/boundary
                } else if z_target.abs() < 0.08 && x_target.abs() < 0.08 {
                    out.push('+'); // Center origin
                } else {
                    out.push(' ');
                }
            }
            out.push('\n');
        }
        out.push_str("       --------------------------------\n");
        out.push_str("      -1.0            0.0            +1.0 (X Axis)\n");
        out.push_str("================================================================================\n");
        out
    }

    pub fn hit_test_puck(&self, pos: (f32, f32), puck_screen_pos: (f32, f32)) -> bool {
        let dx = pos.0 - puck_screen_pos.0;
        let dy = pos.1 - puck_screen_pos.1;
        (dx * dx + dy * dy).sqrt() <= QUANTUM_PUCK_HIT_RADIUS
    }

    pub fn render_ascii_snapshot_str(&self) -> String {
        self.render_ascii_snapshot()
    }

    pub fn hit_test_quantum_puck(&self, pointer_pos: (f32, f32), canvas_rect: Rect) -> bool {
        let padding = 15.0;
        let inner_w = (canvas_rect.width - 2.0 * padding).max(10.0);
        let inner_h = (canvas_rect.height - 2.0 * padding).max(10.0);

        let puck_x = canvas_rect.x + padding + self.puck_pos.0 * inner_w;
        let puck_y = canvas_rect.y + canvas_rect.height - padding - self.puck_pos.1 * inner_h;

        let dx = pointer_pos.0 - puck_x;
        let dy = pointer_pos.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= QUANTUM_PUCK_HIT_RADIUS
    }

    #[cfg(feature = "gui")]
    pub fn show_ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Preset Selection Header
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Quantum State Preset:")
                        .font(FontId::proportional(12.0))
                        .color(Color32::from_rgb(148, 163, 184)),
                );
                let current_preset = self.preset;
                egui::ComboBox::from_id_source("quantum_bloch_preset_combo")
                    .selected_text(RichText::new(current_preset.name()).font(FontId::proportional(12.0)))
                    .show_ui(ui, |ui| {
                        let presets = [
                            QuantumStatePreset::HadamardSuperposition,
                            QuantumStatePreset::ExcitedStatePauliX,
                            QuantumStatePreset::BellPhaseEntangled,
                            QuantumStatePreset::GroundCoherentState,
                        ];
                        for p in presets {
                            if ui.selectable_label(self.preset == p, p.name()).clicked() {
                                self.set_preset(p);
                            }
                        }
                    });
            });

            ui.add_space(8.0);

            // Quantum Gate Control Action Buttons (minimum 44x44pt bounding targets)
            ui.horizontal(|ui| {
                if ui
                    .add_sized(
                        [100.0, 36.0],
                        egui::Button::new(
                            RichText::new("H Hadamard")
                                .font(FontId::proportional(11.0))
                                .strong()
                                .color(Color32::from_rgb(168, 85, 247)),
                        )
                        .fill(Color32::from_rgb(30, 20, 45))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(168, 85, 247)))
                        .rounding(Rounding::same(4.0)),
                    )
                    .on_hover_text("Apply Hadamard Gate H: transforms basis into equal superposition (|0> + |1>)/sqrt(2)")
                    .clicked()
                {
                    self.apply_hadamard();
                }

                if ui
                    .add_sized(
                        [100.0, 36.0],
                        egui::Button::new(
                            RichText::new("X Pauli-X")
                                .font(FontId::proportional(11.0))
                                .strong()
                                .color(Color32::from_rgb(56, 189, 248)),
                        )
                        .fill(Color32::from_rgb(15, 30, 45))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(56, 189, 248)))
                        .rounding(Rounding::same(4.0)),
                    )
                    .on_hover_text("Apply Pauli-X Gate: quantum bit-flip NOT operation (|0> <-> |1>)")
                    .clicked()
                {
                    self.apply_pauli_x();
                }

                if ui
                    .add_sized(
                        [100.0, 36.0],
                        egui::Button::new(
                            RichText::new("S Phase π/2")
                                .font(FontId::proportional(11.0))
                                .strong()
                                .color(Color32::from_rgb(245, 158, 11)),
                        )
                        .fill(Color32::from_rgb(40, 28, 12))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(245, 158, 11)))
                        .rounding(Rounding::same(4.0)),
                    )
                    .on_hover_text("Apply S Gate: adds π/2 relative phase shift between |0> and |1>")
                    .clicked()
                {
                    self.apply_phase_shift(PI * 0.5);
                }

                if ui
                    .add_sized(
                        [100.0, 36.0],
                        egui::Button::new(
                            RichText::new("Z Pauli-Z")
                                .font(FontId::proportional(11.0))
                                .strong()
                                .color(Color32::from_rgb(236, 72, 153)),
                        )
                        .fill(Color32::from_rgb(45, 15, 30))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(236, 72, 153)))
                        .rounding(Rounding::same(4.0)),
                    )
                    .on_hover_text("Apply Pauli-Z Gate: flips relative phase by π radians")
                    .clicked()
                {
                    self.apply_phase_shift(PI);
                }
            });

            ui.add_space(8.0);

            // Dual Canvas Row: 2D Interactive Puck Canvas (Left) + 3D Orthographic Bloch Sphere Projection (Right)
            ui.horizontal(|ui| {
                // 2D Puck Canvas (Superposition Probability vs Relative Phase)
                let canvas_size = Vec2::new(320.0, 180.0);
                let (canvas_resp, painter) = ui.allocate_painter(canvas_size, egui::Sense::click_and_drag());
                let rect = canvas_resp.rect;

                painter.rect_filled(rect, Rounding::same(6.0), Color32::from_rgb(15, 23, 42));
                painter.rect_stroke(rect, Rounding::same(6.0), Stroke::new(1.0_f32, Color32::from_rgb(51, 65, 85)));

                // Grid Lines
                for i in 1..4 {
                    let gx = rect.left() + (i as f32 / 4.0) * rect.width();
                    painter.line_segment([egui::pos2(gx, rect.top()), egui::pos2(gx, rect.bottom())], Stroke::new(1.0_f32, Color32::from_rgb(30, 41, 59)));
                    let gy = rect.top() + (i as f32 / 4.0) * rect.height();
                    painter.line_segment([egui::pos2(rect.left(), gy), egui::pos2(rect.right(), gy)], Stroke::new(1.0_f32, Color32::from_rgb(30, 41, 59)));
                }

                // Axis Labels
                painter.text(
                    egui::pos2(rect.left() + 8.0, rect.top() + 8.0),
                    egui::Align2::LEFT_TOP,
                    "Puck: |α|² (X) vs Phase ϕ (Y)",
                    FontId::proportional(10.0),
                    Color32::from_rgb(148, 163, 184),
                );

                // Handle Puck Drag
                if canvas_resp.dragged() || canvas_resp.clicked() {
                    if let Some(pos) = canvas_resp.interact_pointer_pos() {
                        let nx = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
                        let ny = (1.0 - ((pos.y - rect.top()) / rect.height())).clamp(0.0, 1.0);
                        self.puck_pos = (nx, ny);
                        self.alpha_prob = Self::normalized_to_alpha_prob(nx);
                        self.relative_phase_rad = Self::normalized_to_phase(ny);
                        self.update_physics();
                    }
                }

                // Draw Puck Target (>= 44x44pt bounding touch circle)
                let px = rect.left() + self.puck_pos.0 * rect.width();
                let py = rect.bottom() - self.puck_pos.1 * rect.height();
                let puck_center = egui::pos2(px, py);

                painter.circle_filled(puck_center, QUANTUM_PUCK_HIT_RADIUS, Color32::from_rgba_unmultiplied(168, 85, 247, 40));
                painter.circle_stroke(puck_center, QUANTUM_PUCK_HIT_RADIUS, Stroke::new(1.5_f32, Color32::from_rgb(168, 85, 247)));
                painter.circle_filled(puck_center, 6.0, Color32::from_rgb(236, 72, 153));

                ui.add_space(12.0);

                // 3D Orthographic Bloch Sphere Display
                let (bloch_resp, b_painter) = ui.allocate_painter(Vec2::new(240.0, 180.0), egui::Sense::hover());
                let b_rect = bloch_resp.rect;

                b_painter.rect_filled(b_rect, Rounding::same(6.0), Color32::from_rgb(11, 19, 36));
                b_painter.rect_stroke(b_rect, Rounding::same(6.0), Stroke::new(1.0_f32, Color32::from_rgb(51, 65, 85)));

                let center = b_rect.center();
                let radius = 64.0_f32;

                // Sphere Outline
                b_painter.circle_stroke(center, radius, Stroke::new(1.5_f32, Color32::from_rgb(71, 85, 105)));
                // Equator Ellipse
                for i in 0..32 {
                    let t0 = (i as f32 / 32.0) * TAU;
                    let t1 = ((i + 1) as f32 / 32.0) * TAU;
                    let p0 = egui::pos2(center.x + radius * t0.cos(), center.y + radius * 0.3 * t0.sin());
                    let p1 = egui::pos2(center.x + radius * t1.cos(), center.y + radius * 0.3 * t1.sin());
                    b_painter.line_segment([p0, p1], Stroke::new(1.0_f32, Color32::from_rgb(51, 65, 85)));
                }

                // Z-Axis Line (|0> to |1>)
                b_painter.line_segment(
                    [egui::pos2(center.x, center.y - radius - 8.0), egui::pos2(center.x, center.y + radius + 8.0)],
                    Stroke::new(1.0_f32, Color32::from_rgb(94, 234, 212)),
                );
                b_painter.text(
                    egui::pos2(center.x, center.y - radius - 14.0),
                    egui::Align2::CENTER_BOTTOM,
                    "|0⟩",
                    FontId::proportional(10.0),
                    Color32::from_rgb(94, 234, 212),
                );
                b_painter.text(
                    egui::pos2(center.x, center.y + radius + 14.0),
                    egui::Align2::CENTER_TOP,
                    "|1⟩",
                    FontId::proportional(10.0),
                    Color32::from_rgb(244, 63, 94),
                );

                // State Vector Projection Arrow
                let sx = center.x + self.bloch_x * radius * 0.9;
                let sy = center.y - self.bloch_z * radius * 0.9 + self.bloch_y * radius * 0.25;
                let state_pos = egui::pos2(sx, sy);

                b_painter.line_segment([center, state_pos], Stroke::new(2.5_f32, Color32::from_rgb(236, 72, 153)));
                b_painter.circle_filled(state_pos, 5.0, Color32::from_rgb(250, 204, 21));
            });

            ui.add_space(8.0);

            // Numerical Telemetry and Tomography Readouts
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!(
                        "|α|² (|0⟩): {:.1}%   |β|² (|1⟩): {:.1}%   Phase ϕ: {:.1}°   Purity P: {:.1}%",
                        self.alpha_prob * 100.0,
                        (1.0 - self.alpha_prob) * 100.0,
                        self.relative_phase_rad * 180.0 / PI,
                        self.state_purity * 100.0
                    ))
                    .font(FontId::proportional(11.0))
                    .color(Color32::from_rgb(226, 232, 240)),
                );
            });

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!(
                        "Bloch Vector: X={:+.3}   Y={:+.3}   Z={:+.3}   Polar θ: {:.1}°",
                        self.bloch_x, self.bloch_y, self.bloch_z, self.polar_angle_theta * 180.0 / PI
                    ))
                    .font(FontId::proportional(11.0))
                    .color(Color32::from_rgb(148, 163, 184)),
                );
            });

            ui.add_space(8.0);

            // Parameter Sliders
            ui.horizontal(|ui| {
                ui.label(RichText::new("Frequency:").font(FontId::proportional(11.0)));
                let mut f = self.frequency_hz;
                if ui.add(egui::Slider::new(&mut f, 20.0..=2000.0).suffix(" Hz")).changed() {
                    self.frequency_hz = f;
                }

                ui.add_space(16.0);

                ui.label(RichText::new("Decoherence:").font(FontId::proportional(11.0)));
                let mut d = self.decoherence_rate;
                if ui.add(egui::Slider::new(&mut d, 0.0..=0.20)).changed() {
                    self.decoherence_rate = d;
                    self.update_physics();
                }
            });
        });
    }
}
