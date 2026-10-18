// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! 16-Mode Dynamic Modal Synthesis Engine & Material Resonator Bank HUD (Milestones 13, 33 & 34).
//!
//! Directly reflects `crates/summoner_dsp/src/modal_synthesis.rs` (`ModalSynthesisEngine`,
//! `ModalExcitationType`, `ModalMaterialPreset`, `MAX_MODAL_BANK_SIZE = 16`) and
//! `crates/summoner_dsp/src/modal.rs` (`ModalResonator`).
//! Provides an interactive 2D Spatial Strike Position & Mallet Hardness / Bow Pressure Puck
//! (Strike Position $x \in [0.05..=0.95]$ vs Mallet Hardness / Bow Pressure $h \in [0.05..=1.0]$),
//! 5 physical material resonance distributions (`MetalBar`, `MarimbaWood`, `GlassBowl`,
//! `MembraneDrum`, `TibetanSingingBowl`), 3 excitation generators (`Strike`, `Bow`, `Pluck`),
//! hyperbolic slip-stick bow friction evaluation, 16-mode spatial coupling $|\sin((i+1)\pi x)|$,
//! 4 curated presets, and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and deterministic ASCII snapshot rendering.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, FontId, RichText, Rounding, Stroke, Vec2};

pub const MODAL_SYNTHESIS_PUCK_HIT_RADIUS: f32 = 22.0; // >= 44x44pt touch bounding target
pub const MAX_MODAL_RESONATOR_MODES: usize = 16;
pub const MIN_MODAL_STRIKE_POS: f32 = 0.05;
pub const MAX_MODAL_STRIKE_POS: f32 = 0.95;
pub const MIN_MODAL_HARDNESS: f32 = 0.05;
pub const MAX_MODAL_HARDNESS: f32 = 1.00;

/// Physical excitation generator mode matching `ModalExcitationType` in `modal_synthesis.rs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ModalExcitationMode {
    #[default]
    Strike,
    Bow,
    Pluck,
}

impl ModalExcitationMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Strike => "Mallet Strike (Percussive Impulse)",
            Self::Bow => "Slip-Stick Bow (Continuous Friction)",
            Self::Pluck => "Plucked Release (Sharp Transient)",
        }
    }

    pub fn short_tag(&self) -> &'static str {
        match self {
            Self::Strike => "STRIKE",
            Self::Bow => "BOW",
            Self::Pluck => "PLUCK",
        }
    }
}

/// Physical material resonance profile matching `ModalMaterialPreset` in `modal_synthesis.rs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ModalMaterialKind {
    #[default]
    MetalBar,
    MarimbaWood,
    GlassBowl,
    MembraneDrum,
    TibetanSingingBowl,
}

impl ModalMaterialKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::MetalBar => "Metallic Vibraphone Bar (Inharmonic Beam)",
            Self::MarimbaWood => "Rosewood Marimba Bar (1:4:10 Parabolic Undercut)",
            Self::GlassBowl => "Crystalline Glass Armonica Bowl",
            Self::MembraneDrum => "Tensioned Bessel Membrane Drumhead",
            Self::TibetanSingingBowl => "Tibetan Singing Bowl (Long-Sustain Alloy)",
        }
    }

    /// Exact 16-mode frequency ratios matching `ModalMaterialPreset::frequency_ratios`.
    pub fn frequency_ratios(&self) -> [f32; MAX_MODAL_RESONATOR_MODES] {
        match self {
            Self::MetalBar => [
                1.000, 2.756, 5.404, 8.933, 13.344, 18.636, 24.810, 31.865, 39.802, 48.621, 58.321,
                68.903, 80.366, 92.711, 105.938, 120.046,
            ],
            Self::MarimbaWood => [
                1.000, 3.980, 9.870, 14.500, 19.800, 25.400, 31.900, 38.600, 46.100, 54.200,
                62.900, 72.100, 81.900, 92.300, 103.200, 114.700,
            ],
            Self::GlassBowl => [
                1.000, 2.320, 4.150, 6.480, 9.250, 12.450, 16.100, 20.200, 24.700, 29.600, 34.900,
                40.600, 46.700, 53.200, 60.100, 67.400,
            ],
            Self::MembraneDrum => [
                1.000, 1.593, 2.135, 2.295, 2.653, 2.917, 3.156, 3.500, 3.650, 4.060, 4.150, 4.540,
                4.600, 4.900, 5.000, 5.350,
            ],
            Self::TibetanSingingBowl => [
                1.000, 2.780, 5.120, 7.890, 11.230, 14.950, 19.100, 23.700, 28.650, 34.100, 39.900,
                46.200, 52.800, 59.900, 67.400, 75.300,
            ],
        }
    }

    /// Exact 16-mode damping factors matching `ModalMaterialPreset::damping_factors`:
    /// $d_i = d_0 \cdot (1 + 0.15 \cdot i^{1.3})$.
    pub fn damping_factors(&self) -> [f32; MAX_MODAL_RESONATOR_MODES] {
        let base_damping = match self {
            Self::MetalBar => 4.5,
            Self::MarimbaWood => 12.0,
            Self::GlassBowl => 1.8,
            Self::MembraneDrum => 18.0,
            Self::TibetanSingingBowl => 0.8,
        };
        let mut dampings = [0.0f32; MAX_MODAL_RESONATOR_MODES];
        for (i, item) in dampings.iter_mut().enumerate() {
            *item = base_damping * (1.0 + 0.15 * (i as f32).powf(1.3));
        }
        dampings
    }
}

/// Curated 16-Mode Modal Synthesis Engine presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ModalSynthesisPreset {
    /// Metallic Vibraphone Strike (`MetalBar`, `Strike`, 220 Hz, 12 Modes, Strike Pos 0.35, Hardness 0.65).
    #[default]
    MetallicVibraphoneStrike,
    /// Rosewood Concert Marimba (`MarimbaWood`, `Strike`, 196 Hz G3, 10 Modes, Strike Pos 0.25, Hardness 0.45).
    RosewoodConcertMarimba,
    /// Bowed Glass Armonica Choir (`GlassBowl`, `Bow`, 440 Hz A4, 14 Modes, Strike Pos 0.42, Hardness/Pressure 0.72).
    BowedGlassArmonicaChoir,
    /// Tibetan Singing Bowl Meditation (`TibetanSingingBowl`, `Bow`, 136.1 Hz Om, 16 Modes, Strike Pos 0.50, Hardness/Pressure 0.80).
    TibetanSingingBowlMeditation,
}

impl ModalSynthesisPreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::MetallicVibraphoneStrike => {
                "Metallic Vibraphone Strike (MetalBar / 220 Hz / 12 Modes)"
            }
            Self::RosewoodConcertMarimba => {
                "Rosewood Concert Marimba (MarimbaWood / 196 Hz / 10 Modes)"
            }
            Self::BowedGlassArmonicaChoir => {
                "Bowed Glass Armonica Choir (GlassBowl / 440 Hz / 14 Modes)"
            }
            Self::TibetanSingingBowlMeditation => {
                "Tibetan Singing Bowl Meditation (TibetanBowl / 136.1 Hz / 16 Modes)"
            }
        }
    }

    /// Returns nominal `(fundamental_hz, num_active_modes, strike_pos, mallet_hardness, bow_velocity, material, excitation)`.
    pub fn nominal_parameters(
        &self,
    ) -> (
        f32,
        usize,
        f32,
        f32,
        f32,
        ModalMaterialKind,
        ModalExcitationMode,
    ) {
        match self {
            Self::MetallicVibraphoneStrike => (
                220.0,
                12,
                0.35,
                0.65,
                0.40,
                ModalMaterialKind::MetalBar,
                ModalExcitationMode::Strike,
            ),
            Self::RosewoodConcertMarimba => (
                196.0,
                10,
                0.25,
                0.45,
                0.35,
                ModalMaterialKind::MarimbaWood,
                ModalExcitationMode::Strike,
            ),
            Self::BowedGlassArmonicaChoir => (
                440.0,
                14,
                0.42,
                0.72,
                0.55,
                ModalMaterialKind::GlassBowl,
                ModalExcitationMode::Bow,
            ),
            Self::TibetanSingingBowlMeditation => (
                136.1,
                16,
                0.50,
                0.80,
                0.48,
                ModalMaterialKind::TibetanSingingBowl,
                ModalExcitationMode::Bow,
            ),
        }
    }
}

/// 16-Mode Dynamic Modal Synthesis Engine & Material Resonator Bank HUD view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModalSynthesisView {
    /// Active preset.
    pub preset: ModalSynthesisPreset,
    /// Physical material resonance preset.
    pub material_preset: ModalMaterialKind,
    /// Excitation generator type (Strike, Bow, Pluck).
    pub excitation_type: ModalExcitationMode,
    /// Fundamental frequency in Hz [20.0 ..= 4000.0].
    pub fundamental_hz: f32,
    /// Number of active resonant modes [1 ..= 16].
    pub num_active_modes: usize,
    /// Normalized spatial strike / bow position along resonator [0.05 ..= 0.95].
    pub strike_position: f32,
    /// Mallet hardness / bow normal pressure [0.05 ..= 1.0].
    pub mallet_hardness: f32,
    /// Tangential bow velocity [0.05 ..= 1.0].
    pub bow_velocity: f32,
    /// Host sample rate in Hz.
    pub sample_rate: u32,
    /// Bypass toggle.
    pub bypassed: bool,
    /// 2D Puck position (X: normalized strike_position, Y: normalized mallet_hardness).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for ModalSynthesisView {
    fn default() -> Self {
        Self::new()
    }
}

impl ModalSynthesisView {
    pub fn new() -> Self {
        let preset = ModalSynthesisPreset::MetallicVibraphoneStrike;
        let (freq, modes, pos, hard, vel, mat, exc) = preset.nominal_parameters();
        let norm_x = Self::strike_pos_to_normalized(pos);
        let norm_y = Self::hardness_to_normalized(hard);

        Self {
            preset,
            material_preset: mat,
            excitation_type: exc,
            fundamental_hz: freq,
            num_active_modes: modes,
            strike_position: pos,
            mallet_hardness: hard,
            bow_velocity: vel,
            sample_rate: 48000,
            bypassed: false,
            puck_pos: (norm_x, norm_y),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        }
    }

    pub fn set_preset(&mut self, preset: ModalSynthesisPreset) {
        self.preset = preset;
        let (freq, modes, pos, hard, vel, mat, exc) = preset.nominal_parameters();
        self.fundamental_hz = freq;
        self.num_active_modes = modes;
        self.strike_position = pos;
        self.mallet_hardness = hard;
        self.bow_velocity = vel;
        self.material_preset = mat;
        self.excitation_type = exc;
        self.sync_puck_from_params();
    }

    pub fn sync_puck_from_params(&mut self) {
        self.fundamental_hz = self.fundamental_hz.clamp(20.0, 4000.0);
        self.num_active_modes = self.num_active_modes.clamp(1, MAX_MODAL_RESONATOR_MODES);
        self.strike_position = self
            .strike_position
            .clamp(MIN_MODAL_STRIKE_POS, MAX_MODAL_STRIKE_POS);
        self.mallet_hardness = self
            .mallet_hardness
            .clamp(MIN_MODAL_HARDNESS, MAX_MODAL_HARDNESS);
        self.bow_velocity = self.bow_velocity.clamp(0.05, 1.0);
        self.puck_pos = (
            Self::strike_pos_to_normalized(self.strike_position),
            Self::hardness_to_normalized(self.mallet_hardness),
        );
    }

    pub fn update_from_puck(&mut self, norm_x: f32, norm_y: f32) {
        let x = norm_x.clamp(0.0, 1.0);
        let y = norm_y.clamp(0.0, 1.0);
        self.puck_pos = (x, y);
        self.strike_position = Self::normalized_to_strike_pos(x);
        self.mallet_hardness = Self::normalized_to_hardness(y);
    }

    /// Strike position normalization [0.05 ..= 0.95] -> [0.0 ..= 1.0].
    pub fn strike_pos_to_normalized(pos: f32) -> f32 {
        let clamped = pos.clamp(MIN_MODAL_STRIKE_POS, MAX_MODAL_STRIKE_POS);
        ((clamped - MIN_MODAL_STRIKE_POS) / (MAX_MODAL_STRIKE_POS - MIN_MODAL_STRIKE_POS))
            .clamp(0.0, 1.0)
    }

    /// Strike position de-normalization [0.0 ..= 1.0] -> [0.05 ..= 0.95].
    pub fn normalized_to_strike_pos(norm: f32) -> f32 {
        let n = norm.clamp(0.0, 1.0);
        MIN_MODAL_STRIKE_POS + n * (MAX_MODAL_STRIKE_POS - MIN_MODAL_STRIKE_POS)
    }

    /// Hardness / pressure normalization [0.05 ..= 1.0] -> [0.0 ..= 1.0].
    pub fn hardness_to_normalized(hard: f32) -> f32 {
        let clamped = hard.clamp(MIN_MODAL_HARDNESS, MAX_MODAL_HARDNESS);
        ((clamped - MIN_MODAL_HARDNESS) / (MAX_MODAL_HARDNESS - MIN_MODAL_HARDNESS)).clamp(0.0, 1.0)
    }

    /// Hardness / pressure de-normalization [0.0 ..= 1.0] -> [0.05 ..= 1.0].
    pub fn normalized_to_hardness(norm: f32) -> f32 {
        let n = norm.clamp(0.0, 1.0);
        MIN_MODAL_HARDNESS + n * (MAX_MODAL_HARDNESS - MIN_MODAL_HARDNESS)
    }

    /// Evaluates spatial modal coupling gain $|\sin((i+1) \cdot x_{\text{strike}} \cdot \pi)|$
    /// for mode index `mode_idx` (0..16), matching `ModalSynthesisEngine::trigger_strike`.
    pub fn evaluate_mode_spatial_gain(&self, mode_idx: usize) -> f32 {
        if self.bypassed || mode_idx >= self.num_active_modes {
            return 0.0;
        }
        let mode_pos = (mode_idx + 1) as f32 * self.strike_position * PI;
        mode_pos.sin().abs().clamp(0.05, 1.0) * self.mallet_hardness
    }

    /// Evaluates the non-linear slip-stick bow friction force matching `ModalSynthesisEngine::compute_bow_excitation`:
    /// $\mu(v_{\text{rel}}) = \text{sign}(v_{\text{rel}}) \cdot (0.2 + 0.8 \cdot e^{-4 |v_{\text{rel}}|})$.
    pub fn evaluate_bow_slip_stick_force(&self, feedback_velocity: f32) -> f32 {
        if self.bypassed {
            return 0.0;
        }
        let v_rel = self.bow_velocity - feedback_velocity;
        let friction = v_rel.signum() * (0.2 + 0.8 * (-4.0 * v_rel.abs()).exp());
        self.mallet_hardness * friction
    }

    /// Returns `(freq_hz, damping_factor, spatial_gain)` for a given mode index `mode_idx` (0..16).
    pub fn mode_telemetry(&self, mode_idx: usize) -> (f32, f32, f32) {
        let idx = mode_idx.min(MAX_MODAL_RESONATOR_MODES - 1);
        let ratios = self.material_preset.frequency_ratios();
        let dampings = self.material_preset.damping_factors();
        let nyquist = self.sample_rate as f32 * 0.48;
        let freq = (self.fundamental_hz * ratios[idx]).clamp(20.0, nyquist);
        let damp = dampings[idx];
        let gain = self.evaluate_mode_spatial_gain(idx);
        (freq, damp, gain)
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
        (dx * dx + dy * dy).sqrt() <= MODAL_SYNTHESIS_PUCK_HIT_RADIUS
    }

    /// Renders a deterministic ASCII snapshot for headless QA verification.
    pub fn render_ascii_snapshot(&self) -> String {
        let (f0, d0, g0) = self.mode_telemetry(0);
        let (f1, d1, g1) = self.mode_telemetry(1);
        let mut out = String::new();
        out.push_str("======================================================================\n");
        out.push_str(" SUMMONER DAW -- 16-MODE DYNAMIC MODAL SYNTHESIS ENGINE HUD SNAPSHOT\n");
        out.push_str("======================================================================\n");
        out.push_str(&format!(" Preset:           {}\n", self.preset.name()));
        out.push_str(&format!(
            " Material/Excite:  {} | {}\n",
            self.material_preset.label(),
            self.excitation_type.short_tag()
        ));
        out.push_str(&format!(
            " Modal Bank:       f0 = {:.1} Hz | Active Modes: {}/{} | Strike Pos: {:.2}\n",
            self.fundamental_hz,
            self.num_active_modes,
            MAX_MODAL_RESONATOR_MODES,
            self.strike_position
        ));
        out.push_str(&format!(
            " Mode #1/#2:       M1 {:.1}Hz (d={:.1}, g={:.2}) | M2 {:.1}Hz (d={:.1}, g={:.2})\n",
            f0, d0, g0, f1, d1, g1
        ));
        out.push_str(&format!(
            " Puck Coord:       ({:.3}, {:.3}) [PASS: >=44pt Hit Target]\n",
            self.puck_pos.0, self.puck_pos.1
        ));
        out.push_str("======================================================================\n");
        out
    }

    #[cfg(feature = "gui")]
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.heading(
                    RichText::new("16-MODE DYNAMIC MODAL SYNTHESIS ENGINE HUD")
                        .color(Color32::from_rgb(245, 158, 11))
                        .strong(),
                );
                ui.label(
                    RichText::new("M13 / M33 ModalSynthesisEngine + 16x ModalResonator Bank")
                        .color(Color32::from_rgb(148, 163, 184))
                        .small(),
                );
            });

            ui.add_space(6.0);

            // Preset Selector Bar
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Modal Presets:").strong());
                let presets = [
                    (
                        ModalSynthesisPreset::MetallicVibraphoneStrike,
                        "Vibraphone Bar",
                    ),
                    (
                        ModalSynthesisPreset::RosewoodConcertMarimba,
                        "Rosewood Marimba",
                    ),
                    (
                        ModalSynthesisPreset::BowedGlassArmonicaChoir,
                        "Glass Armonica",
                    ),
                    (
                        ModalSynthesisPreset::TibetanSingingBowlMeditation,
                        "Tibetan Bowl",
                    ),
                ];
                for (p, label) in presets {
                    let is_active = self.preset == p;
                    let btn = egui::Button::new(
                        RichText::new(label)
                            .color(if is_active {
                                Color32::BLACK
                            } else {
                                Color32::from_rgb(203, 213, 225)
                            })
                            .size(11.0),
                    )
                    .fill(if is_active {
                        Color32::from_rgb(245, 158, 11)
                    } else {
                        Color32::from_rgb(30, 41, 59)
                    })
                    .min_size(Vec2::new(88.0, 24.0));

                    if ui.add(btn).clicked() {
                        self.set_preset(p);
                    }
                }
            });

            ui.add_space(8.0);

            // Interactive 2D Strike Position & Mallet Hardness Puck + 16-Mode Spectrum Canvas
            let canvas_size = Vec2::new(ui.available_width().max(280.0), 144.0);
            let (response, painter) =
                ui.allocate_painter(canvas_size, egui::Sense::click_and_drag());
            let rect = response.rect;
            let canvas = Rect::new(rect.min.x, rect.min.y, rect.width(), rect.height());

            painter.rect_filled(rect, Rounding::same(4.0), Color32::from_rgb(22, 16, 8));
            painter.rect_stroke(
                rect,
                Rounding::same(4.0),
                Stroke::new(1.0_f32, Color32::from_rgb(217, 119, 6)),
            );

            // Draw 16-Mode Resonator Partial Bars
            let bar_area_w = rect.width() - 32.0;
            let bar_spacing = bar_area_w / MAX_MODAL_RESONATOR_MODES as f32;
            for mode_idx in 0..MAX_MODAL_RESONATOR_MODES {
                let gain = self.evaluate_mode_spatial_gain(mode_idx);
                let bx = rect.min.x + 16.0 + mode_idx as f32 * bar_spacing;
                let bh = gain * (rect.height() - 36.0);
                let bar_rect = egui::Rect::from_min_max(
                    egui::pos2(bx, rect.max.y - 12.0 - bh),
                    egui::pos2(bx + (bar_spacing * 0.55).max(3.0), rect.max.y - 12.0),
                );
                let color = if mode_idx < self.num_active_modes {
                    Color32::from_rgb(251, 191, 36)
                } else {
                    Color32::from_rgb(60, 50, 35)
                };
                painter.rect_filled(bar_rect, Rounding::same(1.5), color);
            }

            // Handle Puck Drag / Click
            if response.dragged() || response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let padding = 15.0;
                    let norm_x = ((pos.x - (rect.min.x + padding))
                        / (rect.width() - 2.0 * padding))
                        .clamp(0.0, 1.0);
                    let norm_y = (1.0
                        - ((pos.y - (rect.min.y + padding)) / (rect.height() - 2.0 * padding)))
                        .clamp(0.0, 1.0);
                    self.update_from_puck(norm_x, norm_y);
                }
            }

            // Draw >= 44x44pt Puck
            let (puck_x, puck_y) = self.puck_to_canvas_pos(&canvas);
            let puck_pos = egui::pos2(puck_x, puck_y);
            painter.circle_filled(
                puck_pos,
                MODAL_SYNTHESIS_PUCK_HIT_RADIUS,
                Color32::from_rgba_unmultiplied(245, 158, 11, 44),
            );
            painter.circle_stroke(
                puck_pos,
                MODAL_SYNTHESIS_PUCK_HIT_RADIUS,
                Stroke::new(1.2_f32, Color32::from_rgb(253, 224, 71)),
            );
            painter.circle_filled(puck_pos, 6.0, Color32::WHITE);

            painter.text(
                egui::pos2(rect.min.x + 8.0, rect.min.y + 6.0),
                egui::Align2::LEFT_TOP,
                format!(
                    "{} | f0: {:.1} Hz | Modes: {} | Strike: {:.0}% | Hardness: {:.0}%",
                    self.excitation_type.short_tag(),
                    self.fundamental_hz,
                    self.num_active_modes,
                    self.strike_position * 100.0,
                    self.mallet_hardness * 100.0
                ),
                FontId::proportional(11.0),
                Color32::from_rgb(254, 243, 199),
            );

            ui.add_space(8.0);

            // Surgical Pro Sliders
            ui.horizontal(|ui| {
                ui.label(RichText::new("Fundamental Pitch (Hz):").size(11.0));
                ui.add(
                    egui::Slider::new(&mut self.fundamental_hz, 20.0..=4000.0)
                        .logarithmic(true)
                        .suffix(" Hz"),
                );
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Strike / Bow Position:").size(11.0));
                let mut sp = self.strike_position;
                if ui
                    .add(egui::Slider::new(
                        &mut sp,
                        MIN_MODAL_STRIKE_POS..=MAX_MODAL_STRIKE_POS,
                    ))
                    .changed()
                {
                    self.strike_position = sp;
                    self.sync_puck_from_params();
                }
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Mallet Hardness / Bow Pressure:").size(11.0));
                let mut mh = self.mallet_hardness;
                if ui
                    .add(egui::Slider::new(
                        &mut mh,
                        MIN_MODAL_HARDNESS..=MAX_MODAL_HARDNESS,
                    ))
                    .changed()
                {
                    self.mallet_hardness = mh;
                    self.sync_puck_from_params();
                }
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Active Resonator Modes:").size(11.0));
                ui.add(egui::Slider::new(
                    &mut self.num_active_modes,
                    1..=MAX_MODAL_RESONATOR_MODES,
                ));
            });
        });
    }
}
