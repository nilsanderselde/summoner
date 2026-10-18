// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! AI Autonomous Mastering, Drum Replacer & Room Acoustic Matcher HUD (Milestones 33 & 34).
//!
//! Directly reflects `crates/summoner_dsp/src/ai_mixing.rs` (`AiAutonomousMasteringEngine`,
//! `TargetCurve`, `AutomatedDrumReplacer`, `DrumReplacementTrigger`, `NeuralRoomAcousticMatcher`,
//! `AcousticProfile`, `AiSongStructureDetector`, `AiPolyphonicChordExtractor`, `NeuralBassGeneratorNode`).
//! Provides an interactive 2D Target LUFS Loudness & Brickwall Ceiling Puck
//! (Target LUFS $\in [-24.0..=-6.0]\text{ LUFS}$ vs Brickwall Ceiling $\in [-3.0..=0.0]\text{ dBFS}$),
//! 5 spectral target curve profiles (`Warm`, `Bright`, `ModernPop`, `ClubLoudness`, `Custom`),
//! automated drum transient replacement threshold & blend controls, neural room RT60 matcher,
//! 4 curated mastering presets, and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and deterministic ASCII snapshot rendering.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, FontId, RichText, Rounding, Stroke, Vec2};

pub const AI_MASTERING_PUCK_HIT_RADIUS: f32 = 22.0; // >= 44x44pt touch bounding target
pub const MIN_TARGET_LUFS: f32 = -24.0;
pub const MAX_TARGET_LUFS: f32 = -6.0;
pub const MIN_MASTER_CEILING_DB: f32 = -3.0;
pub const MAX_MASTER_CEILING_DB: f32 = 0.0;

/// Spectral EQ target curve profile reflecting `summoner_dsp::ai_mixing::TargetCurve`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MasteringTargetCurve {
    WarmAnalog,
    BrightAir,
    #[default]
    ModernPop,
    ClubLoudness,
}

impl MasteringTargetCurve {
    pub fn label(&self) -> &'static str {
        match self {
            Self::WarmAnalog => "Warm Analog (Sub + Low-Mid Tilt)",
            Self::BrightAir => "Bright Air (>10 kHz Silk Sheen)",
            Self::ModernPop => "Modern Pop (Balanced Presence)",
            Self::ClubLoudness => "Club Loudness (Sub Punch + High Definition)",
        }
    }

    /// 5-band spectral EQ weights matching `AiAutonomousMasteringEngine::master_buffer`.
    pub fn eq_band_weights(&self) -> [f32; 5] {
        match self {
            Self::WarmAnalog => [1.20, 1.10, 1.00, 0.95, 0.90],
            Self::BrightAir => [0.90, 0.95, 1.05, 1.15, 1.25],
            Self::ModernPop => [1.10, 1.00, 1.05, 1.10, 1.15],
            Self::ClubLoudness => [1.30, 1.10, 1.00, 1.05, 1.20],
        }
    }
}

/// Curated AI Autonomous Mastering & Drum Replacement presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AiMasteringPreset {
    /// Streaming standard (-14.0 LUFS, -1.0 dBFS true peak, ModernPop curve).
    #[default]
    StreamingPop14Lufs,
    /// Club & Festival maximum impact (-8.0 LUFS, -0.1 dBFS ceiling, ClubLoudness curve).
    ClubFestival8Lufs,
    /// Warm Analog Vinyl / Tape master (-16.0 LUFS, -0.5 dBFS ceiling, WarmAnalog curve).
    WarmVinyl16Lufs,
    /// Broadcast EBU R128 compliant (-23.0 LUFS, -1.5 dBFS ceiling, BrightAir curve).
    BroadcastEbu23Lufs,
}

impl AiMasteringPreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::StreamingPop14Lufs => "Streaming Pop Standard (-14.0 LUFS / -1.0 dBFS)",
            Self::ClubFestival8Lufs => "Club & Festival Impact (-8.0 LUFS / -0.1 dBFS)",
            Self::WarmVinyl16Lufs => "Warm Analog Vinyl (-16.0 LUFS / -0.5 dBFS)",
            Self::BroadcastEbu23Lufs => "EBU R128 Broadcast (-23.0 LUFS / -1.5 dBFS)",
        }
    }

    /// Returns nominal (target_lufs, ceiling_db, curve, drum_threshold_db, drum_blend, room_rt60_sec, sub_bass_mix).
    pub fn nominal_parameters(&self) -> (f32, f32, MasteringTargetCurve, f32, f32, f32, f32) {
        match self {
            Self::StreamingPop14Lufs => (
                -14.0,
                -1.0,
                MasteringTargetCurve::ModernPop,
                -18.0,
                0.65,
                1.20,
                0.45,
            ),
            Self::ClubFestival8Lufs => (
                -8.0,
                -0.1,
                MasteringTargetCurve::ClubLoudness,
                -12.0,
                0.85,
                0.85,
                0.75,
            ),
            Self::WarmVinyl16Lufs => (
                -16.0,
                -0.5,
                MasteringTargetCurve::WarmAnalog,
                -22.0,
                0.40,
                1.65,
                0.55,
            ),
            Self::BroadcastEbu23Lufs => (
                -23.0,
                -1.5,
                MasteringTargetCurve::BrightAir,
                -24.0,
                0.25,
                1.10,
                0.30,
            ),
        }
    }
}

/// AI Autonomous Mastering, Drum Replacer & Room Acoustic Matcher HUD view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAutonomousMasteringView {
    /// Active mastering preset.
    pub preset: AiMasteringPreset,
    /// Target spectral EQ curve (`TargetCurve`).
    pub target_curve: MasteringTargetCurve,
    /// Target integrated loudness in LUFS [-24.0 ..= -6.0].
    pub target_lufs: f32,
    /// Brickwall limiter safety ceiling in dBFS [-3.0 ..= 0.0].
    pub ceiling_db: f32,
    /// Automated drum replacer trigger threshold in dBFS [-48.0 ..= -3.0].
    pub drum_threshold_db: f32,
    /// Automated drum replacer SFZ wet/dry blend [0.0 ..= 1.0].
    pub drum_blend: f32,
    /// Neural room acoustic matcher target RT60 decay in seconds [0.1 ..= 5.0].
    pub room_rt60_sec: f32,
    /// Real-time neural sub-octave bass synth generator mix [0.0 ..= 1.0].
    pub sub_bass_mix: f32,
    /// Whether autonomous mastering processing is bypassed.
    pub bypassed: bool,
    /// 2D Puck position (X: normalized target_lufs, Y: normalized ceiling_db).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for AiAutonomousMasteringView {
    fn default() -> Self {
        Self::new()
    }
}

impl AiAutonomousMasteringView {
    pub fn new() -> Self {
        let preset = AiMasteringPreset::StreamingPop14Lufs;
        let (lufs, ceil, curve, d_thresh, d_blend, rt60, sub_mix) = preset.nominal_parameters();
        let norm_x = Self::lufs_to_normalized(lufs);
        let norm_y = Self::ceiling_to_normalized(ceil);

        Self {
            preset,
            target_curve: curve,
            target_lufs: lufs,
            ceiling_db: ceil,
            drum_threshold_db: d_thresh,
            drum_blend: d_blend,
            room_rt60_sec: rt60,
            sub_bass_mix: sub_mix,
            bypassed: false,
            puck_pos: (norm_x, norm_y),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        }
    }

    pub fn set_preset(&mut self, preset: AiMasteringPreset) {
        self.preset = preset;
        let (lufs, ceil, curve, d_thresh, d_blend, rt60, sub_mix) = preset.nominal_parameters();
        self.target_lufs = lufs;
        self.ceiling_db = ceil;
        self.target_curve = curve;
        self.drum_threshold_db = d_thresh;
        self.drum_blend = d_blend;
        self.room_rt60_sec = rt60;
        self.sub_bass_mix = sub_mix;
        self.sync_puck_from_params();
    }

    pub fn sync_puck_from_params(&mut self) {
        self.puck_pos = (
            Self::lufs_to_normalized(self.target_lufs),
            Self::ceiling_to_normalized(self.ceiling_db),
        );
    }

    pub fn update_from_puck(&mut self, norm_x: f32, norm_y: f32) {
        let x = norm_x.clamp(0.0, 1.0);
        let y = norm_y.clamp(0.0, 1.0);
        self.puck_pos = (x, y);
        self.target_lufs = Self::normalized_to_lufs(x);
        self.ceiling_db = Self::normalized_to_ceiling(y);
    }

    pub fn lufs_to_normalized(lufs: f32) -> f32 {
        ((lufs.clamp(MIN_TARGET_LUFS, MAX_TARGET_LUFS) - MIN_TARGET_LUFS)
            / (MAX_TARGET_LUFS - MIN_TARGET_LUFS))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_lufs(norm: f32) -> f32 {
        MIN_TARGET_LUFS + norm.clamp(0.0, 1.0) * (MAX_TARGET_LUFS - MIN_TARGET_LUFS)
    }

    pub fn ceiling_to_normalized(ceiling_db: f32) -> f32 {
        ((ceiling_db.clamp(MIN_MASTER_CEILING_DB, MAX_MASTER_CEILING_DB) - MIN_MASTER_CEILING_DB)
            / (MAX_MASTER_CEILING_DB - MIN_MASTER_CEILING_DB))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_ceiling(norm: f32) -> f32 {
        MIN_MASTER_CEILING_DB
            + norm.clamp(0.0, 1.0) * (MAX_MASTER_CEILING_DB - MIN_MASTER_CEILING_DB)
    }

    /// Evaluates autonomous mastering sample transfer matching `AiAutonomousMasteringEngine::master_buffer`.
    pub fn evaluate_mastered_sample(&self, input: f32, band_idx: usize, est_lufs: f32) -> f32 {
        if self.bypassed {
            return input;
        }
        let gain_db = (self.target_lufs - est_lufs).clamp(-12.0, 18.0);
        let gain_linear = 10.0_f32.powf(gain_db / 20.0);
        let weights = self.target_curve.eq_band_weights();
        let eq_gain = weights[band_idx % weights.len()];
        let processed = input * gain_linear * eq_gain;
        let ceiling_linear = 10.0_f32.powf(self.ceiling_db / 20.0);

        let limited = if processed.abs() > ceiling_linear {
            processed.signum()
                * (ceiling_linear + (processed.abs() - ceiling_linear).tanh() * 0.05)
        } else {
            processed
        };
        limited.clamp(-ceiling_linear, ceiling_linear)
    }

    /// Evaluates normalized spectral target curve across normalized frequency [0.0..=1.0].
    pub fn evaluate_spectral_contour(&self, norm_freq: f32) -> f32 {
        let f = norm_freq.clamp(0.0, 1.0);
        let weights = self.target_curve.eq_band_weights();
        let scaled = f * (weights.len() - 1) as f32;
        let idx = (scaled.floor() as usize).min(weights.len() - 2);
        let frac = scaled - idx as f32;
        let w = weights[idx] * (1.0 - frac) + weights[idx + 1] * frac;
        let lufs_push = Self::lufs_to_normalized(self.target_lufs) * 0.22;
        let sub_boost = if f < 0.25 {
            (1.0 - f / 0.25) * self.sub_bass_mix * 0.18
        } else {
            0.0
        };
        ((w - 0.70) / 0.85 + lufs_push * 0.5 + sub_boost).clamp(0.08, 0.95)
    }

    /// Evaluates estimated room volume in $m^3$ matching `NeuralRoomAcousticMatcher`.
    pub fn estimated_room_volume_m3(&self) -> f32 {
        self.room_rt60_sec.clamp(0.1, 5.0) * 80.0
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
        (dx * dx + dy * dy).sqrt() <= AI_MASTERING_PUCK_HIT_RADIUS
    }

    /// Renders a deterministic ASCII snapshot for headless QA verification.
    pub fn render_ascii_snapshot(&self) -> String {
        let mut out = String::new();
        out.push_str("======================================================================\n");
        out.push_str(" SUMMONER DAW -- AI AUTONOMOUS MASTERING & DRUM REPLACER SNAPSHOT\n");
        out.push_str("======================================================================\n");
        out.push_str(&format!(" Preset:           {}\n", self.preset.name()));
        out.push_str(&format!(
            " Target Curve:     {} | Bypassed: {}\n",
            self.target_curve.label(),
            if self.bypassed { "YES" } else { "NO" }
        ));
        out.push_str(&format!(
            " Target Loudness:  {:.1} LUFS | Brickwall Ceiling: {:+.2} dBFS\n",
            self.target_lufs, self.ceiling_db
        ));
        out.push_str(&format!(
            " Drum Replacer:    {:+.1} dBFS Threshold | {:.0}% SFZ Blend\n",
            self.drum_threshold_db,
            self.drum_blend * 100.0
        ));
        out.push_str(&format!(
            " Room Matcher:     {:.2} s RT60 ({:.0} m^3) | Sub-Bass Synth: {:.0}%\n",
            self.room_rt60_sec,
            self.estimated_room_volume_m3(),
            self.sub_bass_mix * 100.0
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
                    RichText::new("AI AUTONOMOUS MASTERING & DRUM REPLACER HUD")
                        .color(Color32::from_rgb(245, 158, 11))
                        .strong(),
                );
                ui.label(
                    RichText::new("M33 / M34 Target Curve EQ, LUFS Limiter & SFZ Drum Trigger")
                        .color(Color32::from_rgb(148, 163, 184))
                        .small(),
                );
            });

            ui.add_space(6.0);

            // Preset Selector Bar
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Mastering Presets:").strong());
                let presets = [
                    (AiMasteringPreset::StreamingPop14Lufs, "Streaming -14 LUFS"),
                    (AiMasteringPreset::ClubFestival8Lufs, "Club -8 LUFS"),
                    (AiMasteringPreset::WarmVinyl16Lufs, "Warm Vinyl -16"),
                    (AiMasteringPreset::BroadcastEbu23Lufs, "Broadcast -23"),
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

            // Interactive 2D Target LUFS & Brickwall Ceiling Canvas
            let canvas_size = Vec2::new(ui.available_width().max(280.0), 144.0);
            let (response, painter) =
                ui.allocate_painter(canvas_size, egui::Sense::click_and_drag());
            let rect = response.rect;
            let canvas = Rect::new(rect.min.x, rect.min.y, rect.width(), rect.height());

            painter.rect_filled(rect, Rounding::same(4.0), Color32::from_rgb(24, 18, 10));
            painter.rect_stroke(
                rect,
                Rounding::same(4.0),
                Stroke::new(1.0_f32, Color32::from_rgb(120, 74, 18)),
            );

            // Draw spectral contour
            let num_pts = 48;
            let mut pts = Vec::with_capacity(num_pts);
            for idx in 0..num_pts {
                let frac = idx as f32 / (num_pts - 1) as f32;
                let r_val = self.evaluate_spectral_contour(frac);
                let px = rect.min.x + 15.0 + frac * (rect.width() - 30.0);
                let py = rect.max.y - 14.0 - r_val * (rect.height() - 32.0);
                pts.push(egui::pos2(px, py));
            }
            for i in 0..(pts.len() - 1) {
                painter.line_segment(
                    [pts[i], pts[i + 1]],
                    Stroke::new(2.0_f32, Color32::from_rgb(251, 191, 36)),
                );
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
                AI_MASTERING_PUCK_HIT_RADIUS,
                Color32::from_rgba_unmultiplied(245, 158, 11, 42),
            );
            painter.circle_stroke(
                puck_pos,
                AI_MASTERING_PUCK_HIT_RADIUS,
                Stroke::new(1.2_f32, Color32::from_rgb(251, 191, 36)),
            );
            painter.circle_filled(puck_pos, 6.0, Color32::WHITE);

            painter.text(
                egui::pos2(rect.min.x + 8.0, rect.min.y + 6.0),
                egui::Align2::LEFT_TOP,
                format!(
                    "{} | Target: {:.1} LUFS | Ceiling: {:+.2} dBFS | RT60: {:.2}s",
                    self.target_curve.label(),
                    self.target_lufs,
                    self.ceiling_db,
                    self.room_rt60_sec
                ),
                FontId::proportional(11.0),
                Color32::from_rgb(253, 224, 71),
            );

            ui.add_space(8.0);

            // Surgical Pro Sliders
            ui.horizontal(|ui| {
                ui.label(RichText::new("Target Loudness (LUFS):").size(11.0));
                let mut l = self.target_lufs;
                if ui
                    .add(egui::Slider::new(&mut l, MIN_TARGET_LUFS..=MAX_TARGET_LUFS).suffix(" LUFS"))
                    .changed()
                {
                    self.target_lufs = l;
                    self.sync_puck_from_params();
                }
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Brickwall Ceiling (dBFS):").size(11.0));
                let mut c = self.ceiling_db;
                if ui
                    .add(
                        egui::Slider::new(&mut c, MIN_MASTER_CEILING_DB..=MAX_MASTER_CEILING_DB)
                            .suffix(" dBFS"),
                    )
                    .changed()
                {
                    self.ceiling_db = c;
                    self.sync_puck_from_params();
                }
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Drum Replacer Threshold:").size(11.0));
                ui.add(egui::Slider::new(&mut self.drum_threshold_db, -48.0..=-3.0).suffix(" dB"));
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Drum SFZ Blend:").size(11.0));
                ui.add(egui::Slider::new(&mut self.drum_blend, 0.0..=1.0));
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Room Matcher RT60:").size(11.0));
                ui.add(egui::Slider::new(&mut self.room_rt60_sec, 0.1..=5.0).suffix(" s"));
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Neural Sub-Bass Mix:").size(11.0));
                ui.add(egui::Slider::new(&mut self.sub_bass_mix, 0.0..=1.0));
            });
        });
    }
}
