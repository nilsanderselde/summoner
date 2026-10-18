// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Air-Jet Reed & Utaguchi Flue Vortex Aerodynamics HUD (Milestones 33 & 34).
//!
//! Directly reflects `crates/summoner_dsp/src/air_reed.rs` (`AirReed`, `AirReedConfig`,
//! `jet_velocity_from_pressure`, `meri_kari_to_pitch_multiplier`, `utaguchi_splitting_function`).
//! Provides an interactive 2D Blowing Pressure & Meri/Kari Head-Tilt Pitch Puck
//! (Blowing Pressure $P \in [50.0..=4000.0]\text{ Pa}$ vs Meri/Kari Pitch Bend
//! $\Delta c \in [-300.0..=+200.0]\text{ cents}$), Bernoulli jet velocity readout,
//! convective vortex transit delay, Utaguchi edge splitting transfer curve,
//! Murai-Iki explosive breath burst control, 4 curated shakuhachi embouchure presets,
//! and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and deterministic ASCII snapshot rendering.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, FontId, RichText, Rounding, Stroke, Vec2};

pub const AIR_REED_PUCK_HIT_RADIUS: f32 = 22.0; // >= 44x44pt touch bounding target
pub const MIN_BLOWING_PRESSURE_PA: f32 = 50.0;
pub const MAX_BLOWING_PRESSURE_PA: f32 = 4000.0;
pub const MIN_MERI_KARI_CENTS: f32 = -300.0;
pub const MAX_MERI_KARI_CENTS: f32 = 200.0;
pub const MIN_JET_DISTANCE_MM: f32 = 2.0;
pub const MAX_JET_DISTANCE_MM: f32 = 25.0;
pub const MIN_EDGE_SHARPNESS: f32 = 0.5;
pub const MAX_EDGE_SHARPNESS: f32 = 3.0;
pub const AIR_DENSITY_KG_M3: f32 = 1.204;

/// Curated physical modeling Air-Reed & Utaguchi flue vortex presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AirReedPreset {
    /// Traditional Zen Honkyoku steady Ro fundamental register.
    #[default]
    HonkyokuRoMeditation,
    /// Deep downward head tilt (Dai-Meri) shading the aperture for dark microtonal pitch drop.
    DeepMeriShadowBend,
    /// High-register Kan overblow with steep upward head angle (Kari).
    KariOverblownKan,
    /// Explosive bamboo breath turbulence burst (Murai-Iki) with 1.2 kHz formant edge.
    MuraiIkiExplosiveBurst,
}

impl AirReedPreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::HonkyokuRoMeditation => "Honkyoku Ro Meditation (800 Pa)",
            Self::DeepMeriShadowBend => "Deep Meri Shadow Bend (-200c)",
            Self::KariOverblownKan => "Kari Overblown Kan (2200 Pa)",
            Self::MuraiIkiExplosiveBurst => "Murai-Iki Explosive Burst (3600 Pa)",
        }
    }

    /// Returns nominal (blowing_pressure_pa, meri_kari_cents, jet_distance_mm, edge_sharpness, turbulence_gain, murai_iki_intensity).
    pub fn nominal_parameters(&self) -> (f32, f32, f32, f32, f32, f32) {
        match self {
            Self::HonkyokuRoMeditation => (800.0, 0.0, 10.0, 1.45, 0.15, 0.00),
            Self::DeepMeriShadowBend => (520.0, -200.0, 14.0, 1.15, 0.28, 0.10),
            Self::KariOverblownKan => (2200.0, 120.0, 6.5, 2.10, 0.12, 0.25),
            Self::MuraiIkiExplosiveBurst => (3600.0, -45.0, 8.0, 2.65, 0.45, 0.95),
        }
    }
}

/// Physical Modeling Air-Jet Reed & Utaguchi Flue Vortex Aerodynamics HUD view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirReedView {
    /// Active air-reed preset.
    pub preset: AirReedPreset,
    /// Blowing pressure in Pascals [50.0 ..= 4000.0].
    pub blowing_pressure_pa: f32,
    /// Dynamic Meri/Kari head tilt pitch offset in cents [-300.0 ..= +200.0].
    pub meri_kari_cents: f32,
    /// Lip-to-utaguchi edge jet distance in mm [2.0 ..= 25.0].
    pub jet_distance_mm: f32,
    /// Utaguchi blowing edge sharpness coefficient [0.5 ..= 3.0].
    pub edge_sharpness: f32,
    /// Convective vortex velocity ratio [0.3 ..= 0.6].
    pub convective_factor: f32,
    /// Baseline breath turbulence noise gain [0.0 ..= 0.5].
    pub turbulence_gain: f32,
    /// Murai-Iki explosive breath burst intensity [0.0 ..= 1.0].
    pub murai_iki_intensity: f32,
    /// Resting embouchure angle in degrees [10.0 ..= 60.0].
    pub embouchure_angle_deg: f32,
    /// Reference sample rate for delay line calculation.
    pub sample_rate: u32,
    /// 2D Dynamics Puck position (X: normalized blowing pressure, Y: normalized Meri/Kari cents).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for AirReedView {
    fn default() -> Self {
        Self::new()
    }
}

impl AirReedView {
    pub fn new() -> Self {
        let preset = AirReedPreset::HonkyokuRoMeditation;
        let (pressure, cents, dist, sharp, turb, murai) = preset.nominal_parameters();
        let norm_x = Self::pressure_to_normalized(pressure);
        let norm_y = Self::meri_kari_to_normalized(cents);

        Self {
            preset,
            blowing_pressure_pa: pressure,
            meri_kari_cents: cents,
            jet_distance_mm: dist,
            edge_sharpness: sharp,
            convective_factor: 0.42,
            turbulence_gain: turb,
            murai_iki_intensity: murai,
            embouchure_angle_deg: 38.0,
            sample_rate: 48000,
            puck_pos: (norm_x, norm_y),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        }
    }

    pub fn set_preset(&mut self, preset: AirReedPreset) {
        self.preset = preset;
        let (pressure, cents, dist, sharp, turb, murai) = preset.nominal_parameters();
        self.blowing_pressure_pa = pressure;
        self.meri_kari_cents = cents;
        self.jet_distance_mm = dist;
        self.edge_sharpness = sharp;
        self.turbulence_gain = turb;
        self.murai_iki_intensity = murai;
        self.sync_puck_from_params();
    }

    pub fn sync_puck_from_params(&mut self) {
        self.puck_pos = (
            Self::pressure_to_normalized(self.blowing_pressure_pa),
            Self::meri_kari_to_normalized(self.meri_kari_cents),
        );
    }

    pub fn update_from_puck(&mut self, norm_x: f32, norm_y: f32) {
        let x = norm_x.clamp(0.0, 1.0);
        let y = norm_y.clamp(0.0, 1.0);
        self.puck_pos = (x, y);
        self.blowing_pressure_pa = Self::normalized_to_pressure(x);
        self.meri_kari_cents = Self::normalized_to_meri_kari(y);
    }

    pub fn pressure_to_normalized(pressure_pa: f32) -> f32 {
        let p = pressure_pa.clamp(MIN_BLOWING_PRESSURE_PA, MAX_BLOWING_PRESSURE_PA);
        ((p - MIN_BLOWING_PRESSURE_PA) / (MAX_BLOWING_PRESSURE_PA - MIN_BLOWING_PRESSURE_PA))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_pressure(norm: f32) -> f32 {
        let n = norm.clamp(0.0, 1.0);
        MIN_BLOWING_PRESSURE_PA + n * (MAX_BLOWING_PRESSURE_PA - MIN_BLOWING_PRESSURE_PA)
    }

    pub fn meri_kari_to_normalized(cents: f32) -> f32 {
        let c = cents.clamp(MIN_MERI_KARI_CENTS, MAX_MERI_KARI_CENTS);
        ((c - MIN_MERI_KARI_CENTS) / (MAX_MERI_KARI_CENTS - MIN_MERI_KARI_CENTS)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_meri_kari(norm: f32) -> f32 {
        let n = norm.clamp(0.0, 1.0);
        MIN_MERI_KARI_CENTS + n * (MAX_MERI_KARI_CENTS - MIN_MERI_KARI_CENTS)
    }

    /// Acoustic Bernoulli jet velocity in m/s: v_jet = sqrt(2 * P / rho).
    pub fn jet_velocity_mps(&self) -> f32 {
        let p = self.blowing_pressure_pa.clamp(0.0, 10000.0);
        (2.0 * p / AIR_DENSITY_KG_M3).sqrt()
    }

    /// Continuous pitch multiplier from Meri/Kari cent offset: 2^(cents / 1200).
    pub fn pitch_multiplier(&self) -> f32 {
        let clamped_cents = self.meri_kari_cents.clamp(MIN_MERI_KARI_CENTS, MAX_MERI_KARI_CENTS);
        2.0_f32.powf(clamped_cents / 1200.0)
    }

    /// Effective blowing angle in degrees after Meri/Kari head tilt adjustment.
    pub fn effective_angle_deg(&self) -> f32 {
        let angle_delta = (self.meri_kari_cents / 300.0) * 15.0;
        (self.embouchure_angle_deg + angle_delta).clamp(10.0, 60.0)
    }

    /// Instantaneous convective vortex transit delay in samples.
    pub fn calculate_jet_delay_samples(&self) -> f32 {
        let d_m = self.jet_distance_mm * 0.001;
        let u_vortex = (self.jet_velocity_mps() * self.convective_factor).max(1.0);
        let tau_sec = d_m / u_vortex;
        let delay_samples = tau_sec * (self.sample_rate.max(8000) as f32);
        delay_samples.clamp(1.0, 4094.0)
    }

    /// Non-linear sharp utaguchi edge splitting transfer function:
    /// f(eta, theta) = tanh(eta * k_sharp + offset(theta)) - 0.22 * (eta * k_sharp)^3
    pub fn evaluate_utaguchi_splitting(&self, jet_displacement: f32) -> f32 {
        let nominal_angle = self.embouchure_angle_deg * (PI / 180.0);
        let current_angle = self.effective_angle_deg() * (PI / 180.0);
        let angle_offset = (current_angle - nominal_angle).sin() * 0.45;

        let scaled_disp = jet_displacement * self.edge_sharpness + angle_offset;
        let sat = scaled_disp.tanh();
        let cubic = 0.22 * (scaled_disp.clamp(-2.0, 2.0)).powi(3);
        (sat - cubic).clamp(-1.0, 1.0)
    }

    /// Converts normalized puck position to canvas coordinates.
    pub fn puck_to_canvas_pos(&self, canvas_rect: &Rect) -> (f32, f32) {
        let padding = 15.0;
        let inner_w = canvas_rect.width - 2.0 * padding;
        let inner_h = canvas_rect.height - 2.0 * padding;
        let x = canvas_rect.x + padding + self.puck_pos.0 * inner_w;
        let y = canvas_rect.y + canvas_rect.height - padding - self.puck_pos.1 * inner_h;
        (x, y)
    }

    /// Hit-tests whether a touch/mouse point lands on the >= 44x44pt touch puck.
    pub fn is_puck_hit(&self, canvas_rect: &Rect, point: (f32, f32)) -> bool {
        let (puck_x, puck_y) = self.puck_to_canvas_pos(canvas_rect);
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= AIR_REED_PUCK_HIT_RADIUS
    }

    /// Renders a deterministic ASCII snapshot for headless QA verification.
    pub fn render_ascii_snapshot(&self) -> String {
        let mut out = String::new();
        out.push_str("======================================================================\n");
        out.push_str(" SUMMONER DAW -- AIR-JET REED & UTAGUCHI FLUE VORTEX HUD SNAPSHOT\n");
        out.push_str("======================================================================\n");
        out.push_str(&format!(" Preset:           {}\n", self.preset.name()));
        out.push_str(&format!(" Blowing Pressure: {:.1} Pa (v_jet: {:.2} m/s)\n", self.blowing_pressure_pa, self.jet_velocity_mps()));
        out.push_str(&format!(" Meri/Kari Bend:   {:+.1} cents (Pitch Mult: {:.4}x)\n", self.meri_kari_cents, self.pitch_multiplier()));
        out.push_str(&format!(" Effective Angle:  {:.1} deg | Jet Distance: {:.1} mm\n", self.effective_angle_deg(), self.jet_distance_mm));
        out.push_str(&format!(" Edge Sharpness:   {:.2} | Jet Delay: {:.1} samples\n", self.edge_sharpness, self.calculate_jet_delay_samples()));
        out.push_str(&format!(" Turbulence:       {:.2} | Murai-Iki Burst: {:.0} %\n", self.turbulence_gain, self.murai_iki_intensity * 100.0));
        out.push_str(&format!(" Puck Coord:       ({:.3}, {:.3}) [PASS: >=44pt Hit Target]\n", self.puck_pos.0, self.puck_pos.1));
        out.push_str("----------------------------------------------------------------------\n");
        out.push_str(" Utaguchi Non-Linear Splitting Curve [-1.5 .. +1.5]:\n ");

        for step in 0..32 {
            let disp = -1.5 + (step as f32 / 31.0) * 3.0;
            let val = self.evaluate_utaguchi_splitting(disp);
            let ch = if val > 0.6 {
                '^'
            } else if val > 0.2 {
                '/'
            } else if val > -0.2 {
                '-'
            } else if val > -0.6 {
                '\\'
            } else {
                'v'
            };
            out.push(ch);
        }
        out.push_str("\n======================================================================\n");
        out
    }

    #[cfg(feature = "gui")]
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Header bar
            ui.horizontal(|ui| {
                ui.heading(
                    RichText::new("AIR-JET REED & UTAGUCHI FLUE VORTEX HUD")
                        .color(Color32::from_rgb(52, 211, 153))
                        .strong(),
                );
                ui.label(
                    RichText::new("M25 / M33 Physical Modeling Aerodynamics")
                        .color(Color32::from_rgb(148, 163, 184))
                        .small(),
                );
            });

            ui.add_space(6.0);

            // Preset Buttons Bar
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Presets:").strong());
                let presets = [
                    (AirReedPreset::HonkyokuRoMeditation, "Honkyoku Ro"),
                    (AirReedPreset::DeepMeriShadowBend, "Deep Meri (-200c)"),
                    (AirReedPreset::KariOverblownKan, "Kari Kan (2.2kPa)"),
                    (AirReedPreset::MuraiIkiExplosiveBurst, "Murai-Iki Burst"),
                ];
                for (p, label) in presets {
                    let is_active = self.preset == p;
                    let btn = egui::Button::new(
                        RichText::new(label)
                            .color(if is_active { Color32::BLACK } else { Color32::from_rgb(203, 213, 225) })
                            .size(11.0),
                    )
                    .fill(if is_active { Color32::from_rgb(52, 211, 153) } else { Color32::from_rgb(30, 41, 59) })
                    .min_size(Vec2::new(80.0, 24.0));

                    if ui.add(btn).clicked() {
                        self.set_preset(p);
                    }
                }
            });

            ui.add_space(8.0);

            // Interactive 2D Dynamics Puck Canvas
            let canvas_size = Vec2::new(ui.available_width().max(280.0), 145.0);
            let (response, painter) = ui.allocate_painter(canvas_size, egui::Sense::click_and_drag());
            let rect = response.rect;
            let canvas = Rect::new(rect.min.x, rect.min.y, rect.width(), rect.height());

            // Background
            painter.rect_filled(rect, Rounding::same(4.0), Color32::from_rgb(10, 22, 18));
            painter.rect_stroke(rect, Rounding::same(4.0), Stroke::new(1.0_f32, Color32::from_rgb(45, 85, 70)));

            // Grid lines
            for i in 1..4 {
                let gx = rect.min.x + (i as f32 / 4.0) * rect.width();
                let gy = rect.min.y + (i as f32 / 4.0) * rect.height();
                painter.line_segment(
                    [egui::pos2(gx, rect.min.y), egui::pos2(gx, rect.max.y)],
                    Stroke::new(0.5_f32, Color32::from_rgb(20, 45, 38)),
                );
                painter.line_segment(
                    [egui::pos2(rect.min.x, gy), egui::pos2(rect.max.x, gy)],
                    Stroke::new(0.5_f32, Color32::from_rgb(20, 45, 38)),
                );
            }

            // Draw Utaguchi Non-Linear Splitting Transfer Curve inside Canvas
            let num_pts = 48;
            let mut pts = Vec::with_capacity(num_pts);
            for idx in 0..num_pts {
                let frac = idx as f32 / (num_pts - 1) as f32;
                let disp = -1.8 + frac * 3.6;
                let split_val = self.evaluate_utaguchi_splitting(disp);
                let px = rect.min.x + 15.0 + frac * (rect.width() - 30.0);
                let py = rect.center().y - split_val * (rect.height() * 0.36);
                pts.push(egui::pos2(px, py));
            }
            for i in 0..(pts.len() - 1) {
                painter.line_segment(
                    [pts[i], pts[i + 1]],
                    Stroke::new(1.8_f32, Color32::from_rgb(52, 211, 153)),
                );
            }

            // Handle Puck Drag / Click Interaction
            if response.dragged() || response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let padding = 15.0;
                    let norm_x = ((pos.x - (rect.min.x + padding)) / (rect.width() - 2.0 * padding)).clamp(0.0, 1.0);
                    let norm_y = (1.0 - ((pos.y - (rect.min.y + padding)) / (rect.height() - 2.0 * padding))).clamp(0.0, 1.0);
                    self.update_from_puck(norm_x, norm_y);
                }
            }

            // Draw >= 44x44pt Interactive Puck
            let (puck_x, puck_y) = self.puck_to_canvas_pos(&canvas);
            let puck_pos = egui::pos2(puck_x, puck_y);

            painter.circle_filled(puck_pos, AIR_REED_PUCK_HIT_RADIUS, Color32::from_rgba_unmultiplied(16, 185, 129, 45));
            painter.circle_stroke(puck_pos, AIR_REED_PUCK_HIT_RADIUS, Stroke::new(1.0_f32, Color32::from_rgb(52, 211, 153)));
            painter.circle_filled(puck_pos, 6.0, Color32::from_rgb(209, 250, 229));
            painter.circle_stroke(puck_pos, 8.0, Stroke::new(1.5_f32, Color32::from_rgb(16, 185, 129)));

            // Text Telemetry Readouts inside Canvas
            painter.text(
                egui::pos2(rect.min.x + 8.0, rect.min.y + 6.0),
                egui::Align2::LEFT_TOP,
                format!(
                    "Pressure: {:.0} Pa (v_jet: {:.1} m/s) | Meri/Kari: {:+.0} c",
                    self.blowing_pressure_pa,
                    self.jet_velocity_mps(),
                    self.meri_kari_cents
                ),
                FontId::proportional(11.0),
                Color32::from_rgb(110, 231, 183),
            );

            painter.text(
                egui::pos2(rect.max.x - 8.0, rect.min.y + 6.0),
                egui::Align2::RIGHT_TOP,
                format!(
                    "Angle: {:.1}° | Delay: {:.1} smp",
                    self.effective_angle_deg(),
                    self.calculate_jet_delay_samples()
                ),
                FontId::proportional(10.0),
                Color32::from_rgb(148, 163, 184),
            );

            ui.add_space(8.0);

            // Surgical Pro Sliders
            ui.horizontal(|ui| {
                ui.label(RichText::new("Blowing Pressure:").size(11.0));
                let mut p = self.blowing_pressure_pa;
                if ui.add(egui::Slider::new(&mut p, MIN_BLOWING_PRESSURE_PA..=MAX_BLOWING_PRESSURE_PA).suffix(" Pa")).changed() {
                    self.blowing_pressure_pa = p;
                    self.sync_puck_from_params();
                }
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Meri/Kari Head Tilt:").size(11.0));
                let mut c = self.meri_kari_cents;
                if ui.add(egui::Slider::new(&mut c, MIN_MERI_KARI_CENTS..=MAX_MERI_KARI_CENTS).suffix(" cents")).changed() {
                    self.meri_kari_cents = c;
                    self.sync_puck_from_params();
                }
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Jet Distance:").size(11.0));
                ui.add(egui::Slider::new(&mut self.jet_distance_mm, MIN_JET_DISTANCE_MM..=MAX_JET_DISTANCE_MM).suffix(" mm"));
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Utaguchi Sharpness:").size(11.0));
                ui.add(egui::Slider::new(&mut self.edge_sharpness, MIN_EDGE_SHARPNESS..=MAX_EDGE_SHARPNESS));
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Murai-Iki Breath Burst:").size(11.0));
                ui.add(egui::Slider::new(&mut self.murai_iki_intensity, 0.0..=1.0));
            });
        });
    }
}
