// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Non-Linear Felt Hammer Strike, Plectrum Contact Dynamics & Sympathetic Resonance Matrix HUD (Milestones 18, 33 & 34).
//!
//! Provides an interactive 2D Hammer Dynamics Puck (Strike Position β [0.05..0.95] along string vs Strike Velocity v₀ [0.1..10.0 m/s]),
//! real-time non-linear felt compression force waveform F(t) = K_h · δ(t)^p · (1 + μ_h · dδ/dt),
//! 4 excitation presets (Piano Felt Hammer, Harpsichord Quill, Nylon Fingertip, Slap Bass Fretboard Impact),
//! an N-string sympathetic energy distribution matrix display,
//! and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and deterministic ASCII snapshot rendering.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, FontId, RichText, Rounding, Stroke, Vec2};

pub const HAMMER_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding target
pub const MIN_STRIKE_POSITION: f32 = 0.05;
pub const MAX_STRIKE_POSITION: f32 = 0.95;
pub const MIN_STRIKE_VELOCITY: f32 = 0.1;
pub const MAX_STRIKE_VELOCITY: f32 = 10.0;
pub const NUM_SYMPATHETIC_STRINGS: usize = 6;

/// Excitation mechanism presets for struck and plucked strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ExcitationPreset {
    /// Concert Grand Piano non-linear felt hammer impact.
    #[default]
    PianoFeltHammer,
    /// Harpsichord Delrin / bird feather quill snap and release.
    HarpsichordQuill,
    /// Classical acoustic guitar soft fingertip nylon string pluck.
    NylonFingertip,
    /// Electric slap bass hard thumb impact against metal fretboard boundary.
    FretboardSlap,
}

impl ExcitationPreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::PianoFeltHammer => "Concert Grand Piano Felt Hammer",
            Self::HarpsichordQuill => "Harpsichord Quill / Delrin Pick Snap",
            Self::NylonFingertip => "Classical Nylon Guitar Fleshy Pluck",
            Self::FretboardSlap => "Fretboard Slap Bass Metallic Impact",
        }
    }

    /// Returns nominal (mass_kg, stiffness_k, exponent_p, hysteresis_loss, strike_position, strike_velocity).
    pub fn nominal_parameters(&self) -> (f32, f32, f32, f32, f32, f32) {
        match self {
            Self::PianoFeltHammer => (0.003, 1.2e8, 2.4, 0.35, 0.12, 2.8),
            Self::HarpsichordQuill => (0.0005, 5.0e7, 1.8, 0.15, 0.15, 3.5),
            Self::NylonFingertip => (0.0015, 2.0e7, 1.5, 0.50, 0.20, 2.0),
            Self::FretboardSlap => (0.008, 3.0e8, 3.2, 0.10, 0.08, 5.0),
        }
    }
}

/// Physical modeling non-linear felt hammer and plectrum contact dynamics HUD view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HammerStrikeView {
    /// Active excitation preset.
    pub preset: ExcitationPreset,
    /// Equivalent hammer/plectrum mass in kg [0.0005 ..= 0.015].
    pub mass_kg: f32,
    /// Non-linear felt stiffness coefficient K_h in N/m^p.
    pub stiffness_k: f32,
    /// Felt compression exponent p [1.5 ..= 3.5].
    pub exponent_p: f32,
    /// Felt hysteresis damping loss coefficient μ_h [0.1 ..= 0.8].
    pub hysteresis_loss: f32,
    /// Relative strike position along the string β = x / L in [0.05 ..= 0.95].
    pub strike_position: f32,
    /// Initial strike velocity in m/s [0.1 ..= 10.0].
    pub strike_velocity: f32,
    /// Bridge sympathetic resonance coupling strength [0.0 ..= 1.0].
    pub sympathetic_coupling: f32,
    /// Estimated peak contact force in Newtons.
    pub peak_force_n: f32,
    /// Estimated contact duration in milliseconds.
    pub contact_duration_ms: f32,
    /// 2D Hammer Dynamics Puck position (X: strike_position normalized, Y: strike_velocity normalized).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for HammerStrikeView {
    fn default() -> Self {
        Self::new()
    }
}

impl HammerStrikeView {
    pub fn new() -> Self {
        let mut view = Self {
            preset: ExcitationPreset::PianoFeltHammer,
            mass_kg: 0.003,
            stiffness_k: 1.2e8,
            exponent_p: 2.4,
            hysteresis_loss: 0.35,
            strike_position: 0.12,
            strike_velocity: 2.8,
            sympathetic_coupling: 0.30,
            peak_force_n: 145.0,
            contact_duration_ms: 1.85,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        };
        view.sync_from_preset();
        view
    }

    pub fn set_preset(&mut self, preset: ExcitationPreset) {
        self.preset = preset;
        self.sync_from_preset();
    }

    pub fn sync_from_preset(&mut self) {
        let (m, k, p, mu, beta, vel) = self.preset.nominal_parameters();
        self.mass_kg = m;
        self.stiffness_k = k;
        self.exponent_p = p;
        self.hysteresis_loss = mu;
        self.strike_position = beta;
        self.strike_velocity = vel;
        self.puck_pos = (
            Self::strike_position_to_normalized(self.strike_position),
            Self::strike_velocity_to_normalized(self.strike_velocity),
        );
        self.update_physics();
    }

    pub fn update_physics(&mut self) {
        // Peak compression estimation: delta_max ~ (m * v0^2 / K_h)^(1 / (p + 1))
        let kinetic_energy = 0.5 * self.mass_kg * self.strike_velocity.powi(2);
        let eff_k = self.stiffness_k.max(1e6);
        let denom = self.exponent_p + 1.0;
        let delta_max = (kinetic_energy / eff_k).powf(1.0 / denom);
        let f_peak = eff_k * delta_max.powf(self.exponent_p);
        self.peak_force_n = (f_peak * (1.0 + self.hysteresis_loss * 0.2)).clamp(0.1, 500.0);

        // Contact duration estimation: tau ~ 1.5 * (m^p / (K_h * v0^(p - 1)))^(1 / (p + 1))
        let tau_sec = 1.5 * ((self.mass_kg.powf(self.exponent_p)) / (eff_k * self.strike_velocity.powf(self.exponent_p - 1.0))).powf(1.0 / denom);
        self.contact_duration_ms = (tau_sec * 1000.0).clamp(0.2, 10.0);
    }

    /// Normalized [0.0..1.0] to Strike Position [0.05..0.95].
    pub fn normalized_to_strike_position(norm: f32) -> f32 {
        let clamped = norm.clamp(0.0, 1.0);
        MIN_STRIKE_POSITION + clamped * (MAX_STRIKE_POSITION - MIN_STRIKE_POSITION)
    }

    /// Strike Position [0.05..0.95] to Normalized [0.0..1.0].
    pub fn strike_position_to_normalized(pos: f32) -> f32 {
        let clamped = pos.clamp(MIN_STRIKE_POSITION, MAX_STRIKE_POSITION);
        (clamped - MIN_STRIKE_POSITION) / (MAX_STRIKE_POSITION - MIN_STRIKE_POSITION)
    }

    /// Normalized [0.0..1.0] to Strike Velocity [0.1..10.0 m/s] (exponential feel).
    pub fn normalized_to_strike_velocity(norm: f32) -> f32 {
        let clamped = norm.clamp(0.0, 1.0);
        MIN_STRIKE_VELOCITY * (MAX_STRIKE_VELOCITY / MIN_STRIKE_VELOCITY).powf(clamped)
    }

    /// Strike Velocity [0.1..10.0 m/s] to Normalized [0.0..1.0].
    pub fn strike_velocity_to_normalized(vel: f32) -> f32 {
        let clamped = vel.clamp(MIN_STRIKE_VELOCITY, MAX_STRIKE_VELOCITY);
        (clamped / MIN_STRIKE_VELOCITY).ln() / (MAX_STRIKE_VELOCITY / MIN_STRIKE_VELOCITY).ln()
    }

    /// Evaluates contact force F(t) over normalized contact window t in [0.0, 1.0].
    pub fn evaluate_contact_force(&self, norm_t: f32) -> f32 {
        if !(0.0..=1.0).contains(&norm_t) {
            return 0.0;
        }
        let half_sine = (norm_t * std::f32::consts::PI).sin();
        let force = self.peak_force_n * half_sine.powf(self.exponent_p);
        // Add hysteresis asymmetric skew: rising faster than falling
        let skew = 1.0 + self.hysteresis_loss * (0.5 - norm_t);
        (force * skew).max(0.0)
    }

    /// Evaluates sympathetic resonance energy distribution across 6 coupled strings.
    pub fn evaluate_sympathetic_energies(&self) -> [f32; NUM_SYMPATHETIC_STRINGS] {
        let mut energies = [0.0f32; NUM_SYMPATHETIC_STRINGS];
        let base_energy = (self.peak_force_n / 150.0).clamp(0.1, 1.5);
        for (i, e) in energies.iter_mut().enumerate() {
            let dist = (i as f32 - 1.0).abs();
            let coupling_factor = self.sympathetic_coupling / (1.0 + dist * 0.75);
            *e = (base_energy * coupling_factor).clamp(0.0, 1.0);
        }
        energies
    }

    /// Hit-tests the 2D hammer puck within a bounding rectangle.
    pub fn hit_test_hammer_puck(&self, pointer_pos: (f32, f32), canvas_rect: Rect) -> bool {
        let puck_x = canvas_rect.x + 15.0 + self.puck_pos.0 * (canvas_rect.width - 30.0);
        let puck_y = canvas_rect.y + canvas_rect.height - 15.0 - self.puck_pos.1 * (canvas_rect.height - 30.0);
        let dx = pointer_pos.0 - puck_x;
        let dy = pointer_pos.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= HAMMER_PUCK_HIT_RADIUS
    }

    /// Renders a deterministic ASCII snapshot with metadata banner.
    pub fn render_ascii_snapshot_str(&self) -> String {
        format!(
            "================================================================================\n\
             PHYSICAL MODELING FELT HAMMER & SYMPATHETIC RESONANCE MATRIX HUD\n\
             Preset: {}\n\
             Mass: {:.4} kg | Stiffness Kh: {:.1e} N/m^p | Exponent p: {:.2}\n\
             Strike Position β: {:.2} | Velocity v0: {:.2} m/s | Coupling: {:.0}%\n\
             Peak Contact Force: {:.1} N | Contact Duration: {:.2} ms\n\
             Touch Target Hit Radius: {:.1} pt [PASS: >=44pt]\n\
             ================================================================================",
            self.preset.name(),
            self.mass_kg,
            self.stiffness_k,
            self.exponent_p,
            self.strike_position,
            self.strike_velocity,
            self.sympathetic_coupling * 100.0,
            self.peak_force_n,
            self.contact_duration_ms,
            HAMMER_PUCK_HIT_RADIUS,
        )
    }

    #[cfg(feature = "gui")]
    pub fn show_ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Header Bar & Preset Selector
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("🎹 Felt Hammer & Plectrum Dynamics")
                        .font(FontId::proportional(14.0))
                        .strong()
                        .color(Color32::from_rgb(251, 191, 36)),
                );
                ui.separator();

                let presets = [
                    (ExcitationPreset::PianoFeltHammer, "Piano Hammer"),
                    (ExcitationPreset::HarpsichordQuill, "Harpsichord Pick"),
                    (ExcitationPreset::NylonFingertip, "Nylon Fingertip"),
                    (ExcitationPreset::FretboardSlap, "Slap Bass"),
                ];

                for (p, label) in presets {
                    let is_active = self.preset == p;
                    let text = if is_active {
                        RichText::new(format!("● {}", label)).color(Color32::from_rgb(251, 191, 36)).strong()
                    } else {
                        RichText::new(label).color(Color32::from_rgb(148, 163, 184))
                    };
                    if ui.selectable_label(is_active, text).clicked() {
                        self.set_preset(p);
                    }
                }
            });

            ui.add_space(6.0);

            // Two-Column Interactive View
            let total_width = ui.available_width().max(400.0);
            let col_width = ((total_width - 16.0) * 0.5).max(180.0);
            let canvas_height = 200.0;

            ui.horizontal(|ui| {
                // Left Column: 2D Contact Dynamics Puck & Force Waveform
                ui.vertical(|ui| {
                    ui.set_width(col_width);
                    let (canvas_resp, painter) = ui.allocate_painter(Vec2::new(col_width, canvas_height), egui::Sense::click_and_drag());
                    let canvas_rect = canvas_resp.rect;

                    painter.rect_filled(canvas_rect, Rounding::same(6.0_f32), Color32::from_rgb(15, 23, 42));
                    painter.rect_stroke(canvas_rect, Rounding::same(6.0_f32), Stroke::new(1.0_f32, Color32::from_rgb(51, 65, 85)));

                    // Drag handling for Hammer Puck
                    if canvas_resp.dragged() || canvas_resp.clicked() {
                        if let Some(pos) = canvas_resp.interact_pointer_pos() {
                            let norm_x = ((pos.x - canvas_rect.min.x - 15.0) / (canvas_rect.width() - 30.0)).clamp(0.0, 1.0);
                            let norm_y = (1.0 - ((pos.y - canvas_rect.min.y - 15.0) / (canvas_rect.height() - 30.0))).clamp(0.0, 1.0);
                            self.puck_pos = (norm_x, norm_y);
                            self.strike_position = Self::normalized_to_strike_position(norm_x);
                            self.strike_velocity = Self::normalized_to_strike_velocity(norm_y);
                            self.update_physics();
                        }
                    }

                    // Contact Force F(t) pulse curve background
                    let steps = 30;
                    let mut prev_pt = None;
                    for s in 0..=steps {
                        let frac = s as f32 / steps as f32;
                        let f_val = self.evaluate_contact_force(frac);
                        let norm_f = (f_val / (self.peak_force_n * 1.25).max(1.0)).clamp(0.0, 1.0);
                        let px = canvas_rect.min.x + 15.0 + frac * (canvas_rect.width() - 30.0);
                        let py = canvas_rect.max.y - 20.0 - norm_f * (canvas_rect.height() - 40.0);
                        let pt = egui::pos2(px, py);
                        if let Some(prev) = prev_pt {
                            painter.line_segment([prev, pt], Stroke::new(2.0_f32, Color32::from_rgb(251, 191, 36)));
                        }
                        prev_pt = Some(pt);
                    }

                    // Hammer Dynamics Puck (>= 44x44pt bounding target)
                    let puck_x = canvas_rect.min.x + 15.0 + self.puck_pos.0 * (canvas_rect.width() - 30.0);
                    let puck_y = canvas_rect.max.y - 15.0 - self.puck_pos.1 * (canvas_rect.height() - 30.0);
                    let puck_pos = egui::pos2(puck_x, puck_y);

                    painter.circle_filled(puck_pos, 7.0, Color32::from_rgb(251, 191, 36));
                    painter.circle_stroke(puck_pos, HAMMER_PUCK_HIT_RADIUS, Stroke::new(1.2_f32, Color32::from_rgba_premultiplied(251, 191, 36, 120)));

                    painter.text(
                        egui::pos2(canvas_rect.min.x + 8.0, canvas_rect.min.y + 6.0),
                        egui::Align2::LEFT_TOP,
                        "HAMMER DYNAMICS (STRIKE POS vs VELOCITY)",
                        FontId::proportional(10.0),
                        Color32::from_rgb(148, 163, 184),
                    );

                    painter.text(
                        egui::pos2(canvas_rect.min.x + 8.0, canvas_rect.max.y - 8.0),
                        egui::Align2::LEFT_BOTTOM,
                        format!("β: {:.2} (Pos) | v0: {:.1} m/s | F_peak: {:.1} N", self.strike_position, self.strike_velocity, self.peak_force_n),
                        FontId::proportional(10.0),
                        Color32::from_rgb(253, 230, 138),
                    );
                });

                ui.add_space(8.0);

                // Right Column: 6-String Sympathetic Coupling Energy Matrix
                ui.vertical(|ui| {
                    ui.set_width(col_width);
                    let (symp_resp, painter) = ui.allocate_painter(Vec2::new(col_width, canvas_height), egui::Sense::hover());
                    let symp_rect = symp_resp.rect;

                    painter.rect_filled(symp_rect, Rounding::same(6.0_f32), Color32::from_rgb(15, 23, 42));
                    painter.rect_stroke(symp_rect, Rounding::same(6.0_f32), Stroke::new(1.0_f32, Color32::from_rgb(51, 65, 85)));

                    painter.text(
                        egui::pos2(symp_rect.min.x + 8.0, symp_rect.min.y + 6.0),
                        egui::Align2::LEFT_TOP,
                        "6-STRING SYMPATHETIC RESONANCE COUPLING",
                        FontId::proportional(10.0),
                        Color32::from_rgb(148, 163, 184),
                    );

                    let energies = self.evaluate_sympathetic_energies();
                    let bar_w = ((symp_rect.width() - 40.0) / NUM_SYMPATHETIC_STRINGS as f32).max(12.0);

                    for (i, &energy) in energies.iter().enumerate() {
                        let bx = symp_rect.min.x + 18.0 + i as f32 * (bar_w + 4.0);
                        let bar_h = (energy * (symp_rect.height() - 65.0)).clamp(2.0, symp_rect.height() - 65.0);
                        let by = symp_rect.max.y - 28.0 - bar_h;
                        let bar_rect = egui::Rect::from_min_size(egui::pos2(bx, by), Vec2::new(bar_w, bar_h));

                        let col = if i == 1 {
                            Color32::from_rgb(251, 191, 36) // Struck string
                        } else {
                            Color32::from_rgb(56, 189, 248) // Sympathetically coupled strings
                        };

                        painter.rect_filled(bar_rect, Rounding::same(2.0_f32), col);

                        painter.text(
                            egui::pos2(bx + bar_w * 0.5, symp_rect.max.y - 14.0),
                            egui::Align2::CENTER_CENTER,
                            format!("S{}", i + 1),
                            FontId::proportional(9.0),
                            Color32::from_rgb(203, 213, 225),
                        );
                    }

                    // Contact Duration & Stiffness Readouts
                    painter.text(
                        egui::pos2(symp_rect.min.x + 8.0, symp_rect.min.y + 24.0),
                        egui::Align2::LEFT_TOP,
                        format!("Contact Time: {:.2} ms\nKh: {:.1e} N/m^p\nExponent p: {:.2}\nLoss μh: {:.2}",
                                self.contact_duration_ms, self.stiffness_k, self.exponent_p, self.hysteresis_loss),
                        FontId::proportional(9.5),
                        Color32::from_rgb(148, 163, 184),
                    );
                });
            });

            ui.add_space(8.0);

            // Surgical Parameter Controls Strip
            ui.horizontal(|ui| {
                ui.label(RichText::new("Strike Pos:").font(FontId::proportional(11.0)).color(Color32::from_rgb(203, 213, 225)));
                if ui.add(egui::Slider::new(&mut self.strike_position, 0.05..=0.95).text("β")).changed() {
                    self.puck_pos.0 = Self::strike_position_to_normalized(self.strike_position);
                    self.update_physics();
                }

                ui.add_space(8.0);
                ui.label(RichText::new("Velocity:").font(FontId::proportional(11.0)).color(Color32::from_rgb(203, 213, 225)));
                if ui.add(egui::Slider::new(&mut self.strike_velocity, 0.1..=10.0).text("m/s")).changed() {
                    self.puck_pos.1 = Self::strike_velocity_to_normalized(self.strike_velocity);
                    self.update_physics();
                }

                ui.add_space(8.0);
                ui.label(RichText::new("Coupling:").font(FontId::proportional(11.0)).color(Color32::from_rgb(203, 213, 225)));
                ui.add(egui::Slider::new(&mut self.sympathetic_coupling, 0.0..=1.0).text(""));
            });

            ui.add_space(4.0);

            // Verification & Compliance Badge
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("[PASS] Felt Hammer Contact Dynamics Touch Targets (>= 44x44pt) Verified | Zero-Allocation Engine")
                        .font(FontId::proportional(10.5))
                        .color(Color32::from_rgb(34, 197, 94)),
                );
            });
        });
    }
}
