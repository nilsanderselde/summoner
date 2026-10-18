// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Multi-Sample SFZ Keyzone & Velocity Layer Mapper HUD (Milestones 33 & 34).
//!
//! Directly reflects `crates/summoner_dsp/src/sampler.rs` (`MultiSamplerNode`, `MultiSampleBank`,
//! `SampleRegion`, `LoopMode`, `SamplerNode`).
//! Provides an interactive 2D Loop Start & Loop Length Window Puck
//! (Normalized Loop Start $\in [0.0..=0.85]$ vs Loop Window Length $\in [0.05..=1.0]$),
//! 5 SFZ loop topologies (`NoLoop`, `Loop`, `LoopContinuous`, `PingPong`, `LoopRelease`),
//! multi-region keyzone auto-mapping (`auto_map_regions_by_keycenter`), velocity-layer region matching,
//! semitone pitch transposition math ($2^{(\text{note} - \text{keycenter}) / 12}$),
//! 4 curated multi-sample SFZ presets, and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and deterministic ASCII snapshot rendering.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, FontId, RichText, Rounding, Stroke, Vec2};

pub const SFZ_SAMPLER_PUCK_HIT_RADIUS: f32 = 22.0; // >= 44x44pt touch bounding target
pub const MIN_LOOP_START_NORM: f32 = 0.0;
pub const MAX_LOOP_START_NORM: f32 = 0.85;
pub const MIN_LOOP_LENGTH_NORM: f32 = 0.05;
pub const MAX_LOOP_LENGTH_NORM: f32 = 1.0;

/// SFZ sample loop playback topology reflecting `summoner_dsp::sampler::LoopMode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SfzLoopTopology {
    NoLoop,
    Loop,
    #[default]
    LoopContinuous,
    PingPong,
    LoopRelease,
}

impl SfzLoopTopology {
    pub fn label(&self) -> &'static str {
        match self {
            Self::NoLoop => "One-Shot (NoLoop)",
            Self::Loop => "Forward Sustain Loop",
            Self::LoopContinuous => "Continuous Loop",
            Self::PingPong => "Bidirectional Ping-Pong Loop",
            Self::LoopRelease => "Sustain Loop + Release Tail",
        }
    }
}

/// Lightweight SFZ keyzone & velocity region descriptor reflecting `summoner_dsp::sampler::SampleRegion`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SfzRegionSlot {
    pub lokey: u8,
    pub hikey: u8,
    pub pitch_keycenter: u8,
    pub lovel: u8,
    pub hivel: u8,
    pub loop_mode: SfzLoopTopology,
    pub sample_label: String,
}

impl SfzRegionSlot {
    pub fn new(
        lokey: u8,
        hikey: u8,
        pitch_keycenter: u8,
        lovel: u8,
        hivel: u8,
        loop_mode: SfzLoopTopology,
        sample_label: &str,
    ) -> Self {
        Self {
            lokey,
            hikey,
            pitch_keycenter,
            lovel,
            hivel,
            loop_mode,
            sample_label: sample_label.to_string(),
        }
    }

    /// Matches MIDI note and velocity against keyzone & velocity layer bounds (`SampleRegion::matches`).
    pub fn matches(&self, note: u8, velocity: u8) -> bool {
        note >= self.lokey
            && note <= self.hikey
            && velocity >= self.lovel
            && velocity <= self.hivel
    }
}

/// Curated Multi-Sample SFZ Instrument presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SfzSamplerPreset {
    /// Velocity-layered Concert Grand Piano (`LoopRelease`, Root 60, Crossfade 15 ms).
    #[default]
    ConcertGrandVelocityLayers,
    /// Continuous sustain-looped Orchestral Strings (`LoopContinuous`, Root 60, Crossfade 45 ms).
    OrchestralStringsSustainLoop,
    /// Bidirectional Ping-Pong looped Ambient Glass Pad (`PingPong`, Root 64, Crossfade 80 ms).
    AmbientPadPingPong,
    /// One-shot MPC Drum & Phrase Keyzone Chops (`NoLoop`, Root 48, Crossfade 2 ms).
    VintageMpcChopsNoLoop,
}

impl SfzSamplerPreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::ConcertGrandVelocityLayers => "Concert Grand Piano (Velocity Layers + LoopRelease)",
            Self::OrchestralStringsSustainLoop => "Orchestral Strings Ensemble (Continuous Loop)",
            Self::AmbientPadPingPong => "Crystalline Ambient Pad (Bidirectional Ping-Pong)",
            Self::VintageMpcChopsNoLoop => "Vintage MPC Keyzone Chops (One-Shot NoLoop)",
        }
    }

    /// Returns nominal `(loop_start_norm, loop_length_norm, root_keycenter, fine_tune_cents, crossfade_ms, vel_tracking, loop_mode)`.
    pub fn nominal_parameters(&self) -> (f32, f32, u8, f32, f32, f32, SfzLoopTopology) {
        match self {
            Self::ConcertGrandVelocityLayers => (
                0.25,
                0.55,
                60,
                0.0,
                15.0,
                0.85,
                SfzLoopTopology::LoopRelease,
            ),
            Self::OrchestralStringsSustainLoop => (
                0.20,
                0.70,
                60,
                2.0,
                45.0,
                0.65,
                SfzLoopTopology::LoopContinuous,
            ),
            Self::AmbientPadPingPong => (
                0.15,
                0.75,
                64,
                -4.0,
                80.0,
                0.50,
                SfzLoopTopology::PingPong,
            ),
            Self::VintageMpcChopsNoLoop => (
                0.00,
                1.00,
                48,
                0.0,
                2.0,
                1.00,
                SfzLoopTopology::NoLoop,
            ),
        }
    }
}

/// Multi-Sample SFZ Keyzone & Velocity Layer Mapper HUD view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SfzSamplerView {
    /// Active SFZ instrument preset.
    pub preset: SfzSamplerPreset,
    /// Active loop playback topology (`LoopMode`).
    pub loop_mode: SfzLoopTopology,
    /// Normalized loop start point [0.0 ..= 0.85].
    pub loop_start_norm: f32,
    /// Normalized loop window length [0.05 ..= 1.0].
    pub loop_length_norm: f32,
    /// Active region pitch keycenter MIDI note [0 ..= 127].
    pub root_keycenter: u8,
    /// Fine pitch tuning offset in cents [-100.0 ..= +100.0].
    pub fine_tune_cents: f32,
    /// Loop boundary equal-power crossfade duration in ms [0.0 ..= 200.0].
    pub crossfade_ms: f32,
    /// Velocity amplitude sensitivity tracking [0.0 ..= 1.0].
    pub velocity_tracking: f32,
    /// Audition MIDI note [0 ..= 127].
    pub audition_note: u8,
    /// Audition MIDI velocity [1 ..= 127].
    pub audition_velocity: u8,
    /// Mapped SFZ keyzone & velocity regions.
    pub regions: Vec<SfzRegionSlot>,
    /// 2D Puck position (X: normalized loop_start_norm, Y: normalized loop_length_norm).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for SfzSamplerView {
    fn default() -> Self {
        Self::new()
    }
}

impl SfzSamplerView {
    pub fn new() -> Self {
        let preset = SfzSamplerPreset::ConcertGrandVelocityLayers;
        let (l_start, l_len, root_key, fine_c, xfade, vel_trk, l_mode) =
            preset.nominal_parameters();
        let norm_x = Self::loop_start_to_normalized(l_start);
        let norm_y = Self::loop_length_to_normalized(l_len);

        let regions = vec![
            SfzRegionSlot::new(0, 48, 36, 0, 127, l_mode, "Grand_C2_v127.flac"),
            SfzRegionSlot::new(49, 64, 60, 0, 90, l_mode, "Grand_C4_Soft.flac"),
            SfzRegionSlot::new(49, 64, 60, 91, 127, l_mode, "Grand_C4_Forte.flac"),
            SfzRegionSlot::new(65, 127, 84, 0, 127, l_mode, "Grand_C6_v127.flac"),
        ];

        Self {
            preset,
            loop_mode: l_mode,
            loop_start_norm: l_start,
            loop_length_norm: l_len,
            root_keycenter: root_key,
            fine_tune_cents: fine_c,
            crossfade_ms: xfade,
            velocity_tracking: vel_trk,
            audition_note: 60,
            audition_velocity: 100,
            regions,
            puck_pos: (norm_x, norm_y),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        }
    }

    pub fn set_preset(&mut self, preset: SfzSamplerPreset) {
        self.preset = preset;
        let (l_start, l_len, root_key, fine_c, xfade, vel_trk, l_mode) =
            preset.nominal_parameters();
        self.loop_start_norm = l_start;
        self.loop_length_norm = l_len;
        self.root_keycenter = root_key;
        self.fine_tune_cents = fine_c;
        self.crossfade_ms = xfade;
        self.velocity_tracking = vel_trk;
        self.loop_mode = l_mode;
        for r in &mut self.regions {
            r.loop_mode = l_mode;
        }
        self.sync_puck_from_params();
    }

    pub fn sync_puck_from_params(&mut self) {
        self.loop_start_norm = self
            .loop_start_norm
            .clamp(MIN_LOOP_START_NORM, MAX_LOOP_START_NORM);
        self.loop_length_norm = self
            .loop_length_norm
            .clamp(MIN_LOOP_LENGTH_NORM, MAX_LOOP_LENGTH_NORM);
        self.puck_pos = (
            Self::loop_start_to_normalized(self.loop_start_norm),
            Self::loop_length_to_normalized(self.loop_length_norm),
        );
    }

    pub fn update_from_puck(&mut self, norm_x: f32, norm_y: f32) {
        let x = norm_x.clamp(0.0, 1.0);
        let y = norm_y.clamp(0.0, 1.0);
        self.puck_pos = (x, y);
        self.loop_start_norm = Self::normalized_to_loop_start(x);
        self.loop_length_norm = Self::normalized_to_loop_length(y);
    }

    pub fn loop_start_to_normalized(start: f32) -> f32 {
        ((start.clamp(MIN_LOOP_START_NORM, MAX_LOOP_START_NORM) - MIN_LOOP_START_NORM)
            / (MAX_LOOP_START_NORM - MIN_LOOP_START_NORM))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_loop_start(norm: f32) -> f32 {
        MIN_LOOP_START_NORM + norm.clamp(0.0, 1.0) * (MAX_LOOP_START_NORM - MIN_LOOP_START_NORM)
    }

    pub fn loop_length_to_normalized(len: f32) -> f32 {
        ((len.clamp(MIN_LOOP_LENGTH_NORM, MAX_LOOP_LENGTH_NORM) - MIN_LOOP_LENGTH_NORM)
            / (MAX_LOOP_LENGTH_NORM - MIN_LOOP_LENGTH_NORM))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_loop_length(norm: f32) -> f32 {
        MIN_LOOP_LENGTH_NORM
            + norm.clamp(0.0, 1.0) * (MAX_LOOP_LENGTH_NORM - MIN_LOOP_LENGTH_NORM)
    }

    /// Computes effective normalized loop end point `(loop_start + loop_length).min(1.0)`.
    pub fn effective_loop_end_norm(&self) -> f32 {
        (self.loop_start_norm + self.loop_length_norm).clamp(0.05, 1.0)
    }

    /// Computes linear pitch playback rate matching `MultiSamplerNode::trigger_note` + fine tune cents.
    pub fn compute_playback_rate(&self, note: u8) -> f32 {
        let semitones =
            (note as f32 - self.root_keycenter as f32) + (self.fine_tune_cents / 100.0);
        2.0_f32.powf(semitones / 12.0)
    }

    /// Auto-maps region keyzones by sorting `pitch_keycenter` and splitting midpoints,
    /// matching `MultiSampleBank::auto_map_regions_by_keycenter`.
    pub fn auto_map_regions_by_keycenter(&mut self) {
        if self.regions.is_empty() {
            return;
        }
        self.regions.sort_by_key(|r| r.pitch_keycenter);
        let n = self.regions.len();
        for i in 0..n {
            let lokey = if i == 0 {
                0
            } else {
                ((self.regions[i - 1].pitch_keycenter as u16
                    + self.regions[i].pitch_keycenter as u16)
                    / 2
                    + 1) as u8
            };
            let hikey = if i == n - 1 {
                127
            } else {
                ((self.regions[i].pitch_keycenter as u16
                    + self.regions[i + 1].pitch_keycenter as u16)
                    / 2) as u8
            };
            self.regions[i].lokey = lokey;
            self.regions[i].hikey = hikey;
        }
    }

    /// Finds matching or nearest region for `(note, velocity)` matching `MultiSampleBank::find_nearest_region`.
    pub fn find_matching_region(&self, note: u8, velocity: u8) -> Option<&SfzRegionSlot> {
        if let Some(exact) = self.regions.iter().find(|r| r.matches(note, velocity)) {
            return Some(exact);
        }
        self.regions
            .iter()
            .min_by_key(|r| (note as i32 - r.pitch_keycenter as i32).abs())
    }

    /// Evaluates synthetic sample waveform envelope [0.0..=1.0] across normalized sample position [0.0..=1.0].
    pub fn evaluate_waveform_envelope(&self, norm_pos: f32) -> f32 {
        let p = norm_pos.clamp(0.0, 1.0);
        let decay = (-p * 2.2).exp();
        let carrier = (p * std::f32::consts::TAU * 6.0).sin() * 0.38 * decay;
        let harmonic = (p * std::f32::consts::TAU * 18.0).cos() * 0.12 * decay;
        (0.5 + carrier + harmonic).clamp(0.05, 0.95)
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
        (dx * dx + dy * dy).sqrt() <= SFZ_SAMPLER_PUCK_HIT_RADIUS
    }

    /// Renders a deterministic ASCII snapshot for headless QA verification.
    pub fn render_ascii_snapshot(&self) -> String {
        let matched_label = self
            .find_matching_region(self.audition_note, self.audition_velocity)
            .map(|r| r.sample_label.as_str())
            .unwrap_or("None");
        let rate = self.compute_playback_rate(self.audition_note);

        let mut out = String::new();
        out.push_str("======================================================================\n");
        out.push_str(" SUMMONER DAW -- MULTI-SAMPLE SFZ KEYZONE & VELOCITY MAPPER SNAPSHOT\n");
        out.push_str("======================================================================\n");
        out.push_str(&format!(" Preset:           {}\n", self.preset.name()));
        out.push_str(&format!(
            " Loop Topology:    {} | Regions: {}\n",
            self.loop_mode.label(),
            self.regions.len()
        ));
        out.push_str(&format!(
            " Loop Window:      Start {:.1}% .. End {:.1}% (Len {:.1}%) | Xfade: {:.1} ms\n",
            self.loop_start_norm * 100.0,
            self.effective_loop_end_norm() * 100.0,
            self.loop_length_norm * 100.0,
            self.crossfade_ms
        ));
        out.push_str(&format!(
            " Pitch & Audition: Root {} | Fine {:+.1} c | Note {} -> {:.3}x ({})\n",
            self.root_keycenter,
            self.fine_tune_cents,
            self.audition_note,
            rate,
            matched_label
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
                    RichText::new("MULTI-SAMPLE SFZ KEYZONE & VELOCITY MAPPER HUD")
                        .color(Color32::from_rgb(192, 132, 252))
                        .strong(),
                );
                ui.label(
                    RichText::new("M33 / M34 MultiSamplerNode + MultiSampleBank + LoopMode Engine")
                        .color(Color32::from_rgb(148, 163, 184))
                        .small(),
                );
            });

            ui.add_space(6.0);

            // Preset Selector Bar
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("SFZ Presets:").strong());
                let presets = [
                    (
                        SfzSamplerPreset::ConcertGrandVelocityLayers,
                        "Concert Grand",
                    ),
                    (
                        SfzSamplerPreset::OrchestralStringsSustainLoop,
                        "Orchestral Strings",
                    ),
                    (SfzSamplerPreset::AmbientPadPingPong, "Ping-Pong Pad"),
                    (SfzSamplerPreset::VintageMpcChopsNoLoop, "MPC Chops"),
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
                        Color32::from_rgb(192, 132, 252)
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

            // Interactive 2D Sample Loop Window & Waveform Canvas
            let canvas_size = Vec2::new(ui.available_width().max(280.0), 144.0);
            let (response, painter) =
                ui.allocate_painter(canvas_size, egui::Sense::click_and_drag());
            let rect = response.rect;
            let canvas = Rect::new(rect.min.x, rect.min.y, rect.width(), rect.height());

            painter.rect_filled(rect, Rounding::same(4.0), Color32::from_rgb(18, 12, 28));
            painter.rect_stroke(
                rect,
                Rounding::same(4.0),
                Stroke::new(1.0_f32, Color32::from_rgb(88, 28, 135)),
            );

            // Highlight active loop window region
            let l_start_x =
                rect.min.x + 15.0 + self.loop_start_norm * (rect.width() - 30.0);
            let l_end_x =
                rect.min.x + 15.0 + self.effective_loop_end_norm() * (rect.width() - 30.0);
            let loop_rect = egui::Rect::from_min_max(
                egui::pos2(l_start_x, rect.min.y + 4.0),
                egui::pos2(l_end_x.max(l_start_x + 4.0), rect.max.y - 4.0),
            );
            painter.rect_filled(
                loop_rect,
                Rounding::same(2.0),
                Color32::from_rgba_unmultiplied(168, 85, 247, 36),
            );

            // Draw sample waveform envelope
            let num_pts = 56;
            let mut pts = Vec::with_capacity(num_pts);
            for idx in 0..num_pts {
                let frac = idx as f32 / (num_pts - 1) as f32;
                let r_val = self.evaluate_waveform_envelope(frac);
                let px = rect.min.x + 15.0 + frac * (rect.width() - 30.0);
                let py = rect.max.y - 15.0 - r_val * (rect.height() - 30.0);
                pts.push(egui::pos2(px, py));
            }
            for i in 0..(pts.len() - 1) {
                painter.line_segment(
                    [pts[i], pts[i + 1]],
                    Stroke::new(2.0_f32, Color32::from_rgb(192, 132, 252)),
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
                SFZ_SAMPLER_PUCK_HIT_RADIUS,
                Color32::from_rgba_unmultiplied(192, 132, 252, 44),
            );
            painter.circle_stroke(
                puck_pos,
                SFZ_SAMPLER_PUCK_HIT_RADIUS,
                Stroke::new(1.2_f32, Color32::from_rgb(216, 180, 254)),
            );
            painter.circle_filled(puck_pos, 6.0, Color32::WHITE);

            painter.text(
                egui::pos2(rect.min.x + 8.0, rect.min.y + 6.0),
                egui::Align2::LEFT_TOP,
                format!(
                    "{} | Loop: {:.0}%..{:.0}% | Root: {} | Rate: {:.2}x",
                    self.loop_mode.label(),
                    self.loop_start_norm * 100.0,
                    self.effective_loop_end_norm() * 100.0,
                    self.root_keycenter,
                    self.compute_playback_rate(self.audition_note)
                ),
                FontId::proportional(11.0),
                Color32::from_rgb(233, 213, 255),
            );

            ui.add_space(8.0);

            // Surgical Pro Sliders
            ui.horizontal(|ui| {
                ui.label(RichText::new("Loop Start Point:").size(11.0));
                let mut s = self.loop_start_norm;
                if ui
                    .add(egui::Slider::new(
                        &mut s,
                        MIN_LOOP_START_NORM..=MAX_LOOP_START_NORM,
                    ))
                    .changed()
                {
                    self.loop_start_norm = s;
                    self.sync_puck_from_params();
                }
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Loop Window Length:").size(11.0));
                let mut l = self.loop_length_norm;
                if ui
                    .add(egui::Slider::new(
                        &mut l,
                        MIN_LOOP_LENGTH_NORM..=MAX_LOOP_LENGTH_NORM,
                    ))
                    .changed()
                {
                    self.loop_length_norm = l;
                    self.sync_puck_from_params();
                }
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Fine Tune Offset (cents):").size(11.0));
                ui.add(egui::Slider::new(&mut self.fine_tune_cents, -100.0..=100.0).suffix(" c"));
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Loop Crossfade (ms):").size(11.0));
                ui.add(egui::Slider::new(&mut self.crossfade_ms, 0.0..=200.0).suffix(" ms"));
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Velocity Sensitivity:").size(11.0));
                ui.add(egui::Slider::new(&mut self.velocity_tracking, 0.0..=1.0));
            });
        });
    }
}
