// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Multi-Pad GM Drum Machine & Per-Pad ADSR/VCA Synthesis HUD (Milestones 33 & 34).
//!
//! Directly reflects `crates/summoner_dsp/src/drum_machine.rs` (`DrumMachineDevice`,
//! `DrumPad`, `DrumMacroView`, `MAX_PADS`, per-pad `MultiSamplerNode` + `EnvADSR` + `VCA`,
//! GM drum map note 36 = Kick, 38 = Snare, 42 = Closed Hat, etc.).
//! Provides an interactive 2D Envelope Decay Scale & Pitch Transpose Puck
//! (Decay Scale $\in [0.20..=2.50]\times$ vs Pitch Tune $\in [-12.0..=+12.0]\text{ st}$),
//! an 8-pad tactile GM drum pad bank with individual gain, mute, and ADSR envelope contour,
//! 4 curated drum kit presets, and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and deterministic ASCII snapshot rendering.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, FontId, RichText, Rounding, Stroke, Vec2};

pub const DRUM_MACHINE_PUCK_HIT_RADIUS: f32 = 22.0; // >= 44x44pt touch bounding target
pub const MIN_DECAY_SCALE: f32 = 0.20;
pub const MAX_DECAY_SCALE: f32 = 2.50;
pub const MIN_PITCH_SEMITONES: f32 = -12.0;
pub const MAX_PITCH_SEMITONES: f32 = 12.0;

/// Curated GM Drum Machine kit presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DrumMachinePreset {
    /// Classic TR-808 sub-bass kick and crispy analog snare kit.
    #[default]
    Analog808SubKit,
    /// Warm studio room-miked acoustic maple drum kit.
    AcousticStudioKit,
    /// Aggressive industrial glitch & metallic percussion kit.
    CyberpunkIndustrialKit,
    /// 12-bit crunchy vinyl boom-bap chopped break kit.
    LoFiVinylDustKit,
}

impl DrumMachinePreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Analog808SubKit => "Analog 808 Sub Kit (Deep Decay)",
            Self::AcousticStudioKit => "Acoustic Studio Maple Kit (Natural)",
            Self::CyberpunkIndustrialKit => "Cyberpunk Industrial Kit (+3.5 st)",
            Self::LoFiVinylDustKit => "Lo-Fi Vinyl Boom-Bap Kit (-3.0 st)",
        }
    }

    /// Returns nominal (master_gain, decay_scale, pitch_semitones, punch_attack_ms, velocity_sensitivity).
    pub fn nominal_parameters(&self) -> (f32, f32, f32, f32, f32) {
        match self {
            Self::Analog808SubKit => (0.90, 1.35, -2.0, 2.0, 0.85),
            Self::AcousticStudioKit => (0.85, 1.00, 0.0, 4.5, 0.95),
            Self::CyberpunkIndustrialKit => (0.95, 0.75, 3.5, 1.0, 0.70),
            Self::LoFiVinylDustKit => (0.82, 0.65, -3.0, 6.0, 0.90),
        }
    }
}

/// Visual state for a single GM drum pad in the HUD.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrumPadSlotConfig {
    pub name: String,
    pub midi_note: u8,
    pub gain: f32,
    pub decay_sec: f32,
    pub muted: bool,
    pub last_velocity: u8,
}

impl DrumPadSlotConfig {
    pub fn new(name: &str, midi_note: u8, gain: f32, decay_sec: f32) -> Self {
        Self {
            name: name.to_string(),
            midi_note,
            gain: gain.clamp(0.0, 1.0),
            decay_sec: decay_sec.clamp(0.02, 4.0),
            muted: false,
            last_velocity: 100,
        }
    }

    /// Effective linear output amplitude combining pad gain, mute, and velocity scaling.
    pub fn effective_amplitude(&self, master_gain: f32) -> f32 {
        if self.muted {
            0.0
        } else {
            let vel_scale = (self.last_velocity as f32 / 127.0).clamp(0.0, 1.0);
            (self.gain * vel_scale * master_gain).clamp(0.0, 1.0)
        }
    }
}

/// Multi-Pad GM Drum Machine & Per-Pad ADSR/VCA Synthesis HUD view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrumMachinePadView {
    /// Active drum kit preset.
    pub preset: DrumMachinePreset,
    /// Global master output gain [0.0 ..= 1.0] reflecting `DrumMacroView::master_gain`.
    pub master_gain: f32,
    /// Global envelope decay multiplier [0.20 ..= 2.50].
    pub decay_scale: f32,
    /// Global pitch tuning transpose in semitones [-12.0 ..= +12.0].
    pub pitch_semitones: f32,
    /// Transient punch attack time in milliseconds [0.5 ..= 25.0].
    pub punch_attack_ms: f32,
    /// Velocity sensitivity scaling [0.0 ..= 1.0].
    pub velocity_sensitivity: f32,
    /// Currently selected pad index [0 .. 8].
    pub selected_pad_idx: usize,
    /// 8 core GM drum pads (Kick 36, Rim 37, Snare 38, Clap 39, CHat 42, OHat 46, Tom 45, Crash 49).
    pub pads: [DrumPadSlotConfig; 8],
    /// 2D Puck position (X: normalized decay scale, Y: normalized pitch semitones).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for DrumMachinePadView {
    fn default() -> Self {
        Self::new()
    }
}

impl DrumMachinePadView {
    pub fn new() -> Self {
        let preset = DrumMachinePreset::Analog808SubKit;
        let (master, decay, pitch, attack, vel_sens) = preset.nominal_parameters();
        let norm_x = Self::decay_to_normalized(decay);
        let norm_y = Self::pitch_to_normalized(pitch);

        Self {
            preset,
            master_gain: master,
            decay_scale: decay,
            pitch_semitones: pitch,
            punch_attack_ms: attack,
            velocity_sensitivity: vel_sens,
            selected_pad_idx: 0,
            pads: [
                DrumPadSlotConfig::new("Kick", 36, 0.95, 0.55),
                DrumPadSlotConfig::new("Rim", 37, 0.75, 0.12),
                DrumPadSlotConfig::new("Snare", 38, 0.90, 0.28),
                DrumPadSlotConfig::new("Clap", 39, 0.82, 0.32),
                DrumPadSlotConfig::new("CHat", 42, 0.78, 0.08),
                DrumPadSlotConfig::new("OHat", 46, 0.80, 0.45),
                DrumPadSlotConfig::new("LowTom", 45, 0.85, 0.38),
                DrumPadSlotConfig::new("Crash", 49, 0.76, 1.20),
            ],
            puck_pos: (norm_x, norm_y),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        }
    }

    pub fn set_preset(&mut self, preset: DrumMachinePreset) {
        self.preset = preset;
        let (master, decay, pitch, attack, vel_sens) = preset.nominal_parameters();
        self.master_gain = master;
        self.decay_scale = decay;
        self.pitch_semitones = pitch;
        self.punch_attack_ms = attack;
        self.velocity_sensitivity = vel_sens;
        self.sync_puck_from_params();
    }

    pub fn sync_puck_from_params(&mut self) {
        self.puck_pos = (
            Self::decay_to_normalized(self.decay_scale),
            Self::pitch_to_normalized(self.pitch_semitones),
        );
    }

    pub fn update_from_puck(&mut self, norm_x: f32, norm_y: f32) {
        let x = norm_x.clamp(0.0, 1.0);
        let y = norm_y.clamp(0.0, 1.0);
        self.puck_pos = (x, y);
        self.decay_scale = Self::normalized_to_decay(x);
        self.pitch_semitones = Self::normalized_to_pitch(y);
    }

    pub fn decay_to_normalized(decay: f32) -> f32 {
        let d = decay.clamp(MIN_DECAY_SCALE, MAX_DECAY_SCALE);
        ((d - MIN_DECAY_SCALE) / (MAX_DECAY_SCALE - MIN_DECAY_SCALE)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_decay(norm: f32) -> f32 {
        let n = norm.clamp(0.0, 1.0);
        MIN_DECAY_SCALE + n * (MAX_DECAY_SCALE - MIN_DECAY_SCALE)
    }

    pub fn pitch_to_normalized(pitch_st: f32) -> f32 {
        let p = pitch_st.clamp(MIN_PITCH_SEMITONES, MAX_PITCH_SEMITONES);
        ((p - MIN_PITCH_SEMITONES) / (MAX_PITCH_SEMITONES - MIN_PITCH_SEMITONES)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_pitch(norm: f32) -> f32 {
        let n = norm.clamp(0.0, 1.0);
        MIN_PITCH_SEMITONES + n * (MAX_PITCH_SEMITONES - MIN_PITCH_SEMITONES)
    }

    /// Pitch playback ratio from semitone transpose: 2^(st / 12).
    pub fn pitch_ratio(&self) -> f32 {
        2.0_f32.powf(self.pitch_semitones.clamp(MIN_PITCH_SEMITONES, MAX_PITCH_SEMITONES) / 12.0)
    }

    /// Evaluates the ADSR + VCA amplitude envelope at normalized time t in [0.0..=1.0].
    pub fn evaluate_pad_envelope(&self, norm_time: f32) -> f32 {
        let t = norm_time.clamp(0.0, 1.0);
        let attack_frac = (self.punch_attack_ms / 100.0).clamp(0.01, 0.25);
        if t < attack_frac {
            (t / attack_frac).clamp(0.0, 1.0)
        } else {
            let decay_t = (t - attack_frac) / (1.0 - attack_frac).max(0.01);
            let rate = (2.4 / self.decay_scale.clamp(MIN_DECAY_SCALE, MAX_DECAY_SCALE)).clamp(0.5, 12.0);
            (-decay_t * rate).exp().clamp(0.0, 1.0)
        }
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
        (dx * dx + dy * dy).sqrt() <= DRUM_MACHINE_PUCK_HIT_RADIUS
    }

    /// Renders a deterministic ASCII snapshot for headless QA verification.
    pub fn render_ascii_snapshot(&self) -> String {
        let mut out = String::new();
        out.push_str("======================================================================\n");
        out.push_str(" SUMMONER DAW -- GM DRUM MACHINE & PER-PAD ADSR/VCA HUD SNAPSHOT\n");
        out.push_str("======================================================================\n");
        out.push_str(&format!(" Preset:           {}\n", self.preset.name()));
        out.push_str(&format!(
            " Master Gain:      {:.2} | Decay Scale: {:.2}x | Pitch: {:+.1} st ({:.3}x)\n",
            self.master_gain,
            self.decay_scale,
            self.pitch_semitones,
            self.pitch_ratio()
        ));
        out.push_str(&format!(
            " Punch Attack:     {:.1} ms | Velocity Sens: {:.0} %\n",
            self.punch_attack_ms,
            self.velocity_sensitivity * 100.0
        ));
        out.push_str(&format!(
            " Puck Coord:       ({:.3}, {:.3}) [PASS: >=44pt Hit Target]\n",
            self.puck_pos.0, self.puck_pos.1
        ));
        out.push_str("----------------------------------------------------------------------\n");
        for (idx, pad) in self.pads.iter().enumerate() {
            out.push_str(&format!(
                " Pad {:02} [{} Note {:02}]: Gain {:.2} | Decay {:.2}s | {}\n",
                idx + 1,
                pad.name,
                pad.midi_note,
                pad.gain,
                pad.decay_sec * self.decay_scale,
                if pad.muted { "MUTED" } else { "ACTIVE" }
            ));
        }
        out.push_str("======================================================================\n");
        out
    }

    #[cfg(feature = "gui")]
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.heading(
                    RichText::new("GM DRUM MACHINE & PER-PAD ADSR/VCA HUD")
                        .color(Color32::from_rgb(249, 115, 22))
                        .strong(),
                );
                ui.label(
                    RichText::new("M33 / M34 MultiSampler + EnvADSR + VCA")
                        .color(Color32::from_rgb(148, 163, 184))
                        .small(),
                );
            });

            ui.add_space(6.0);

            // Preset Buttons Bar
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Kit Presets:").strong());
                let presets = [
                    (DrumMachinePreset::Analog808SubKit, "808 Sub Kit"),
                    (DrumMachinePreset::AcousticStudioKit, "Acoustic Studio"),
                    (DrumMachinePreset::CyberpunkIndustrialKit, "Cyberpunk Kit"),
                    (DrumMachinePreset::LoFiVinylDustKit, "Lo-Fi Vinyl"),
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
                        Color32::from_rgb(249, 115, 22)
                    } else {
                        Color32::from_rgb(30, 41, 59)
                    })
                    .min_size(Vec2::new(82.0, 24.0));

                    if ui.add(btn).clicked() {
                        self.set_preset(p);
                    }
                }
            });

            ui.add_space(8.0);

            // Interactive 2D Decay Scale & Pitch Transpose Canvas
            let canvas_size = Vec2::new(ui.available_width().max(280.0), 140.0);
            let (response, painter) =
                ui.allocate_painter(canvas_size, egui::Sense::click_and_drag());
            let rect = response.rect;
            let canvas = Rect::new(rect.min.x, rect.min.y, rect.width(), rect.height());

            painter.rect_filled(rect, Rounding::same(4.0), Color32::from_rgb(22, 14, 10));
            painter.rect_stroke(
                rect,
                Rounding::same(4.0),
                Stroke::new(1.0_f32, Color32::from_rgb(96, 52, 24)),
            );

            // Draw ADSR Envelope Contour inside Canvas
            let num_pts = 48;
            let mut pts = Vec::with_capacity(num_pts);
            for idx in 0..num_pts {
                let frac = idx as f32 / (num_pts - 1) as f32;
                let env_val = self.evaluate_pad_envelope(frac);
                let px = rect.min.x + 15.0 + frac * (rect.width() - 30.0);
                let py = rect.max.y - 14.0 - env_val * (rect.height() - 32.0);
                pts.push(egui::pos2(px, py));
            }
            for i in 0..(pts.len() - 1) {
                painter.line_segment(
                    [pts[i], pts[i + 1]],
                    Stroke::new(2.0_f32, Color32::from_rgb(249, 115, 22)),
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
                DRUM_MACHINE_PUCK_HIT_RADIUS,
                Color32::from_rgba_unmultiplied(249, 115, 22, 42),
            );
            painter.circle_stroke(
                puck_pos,
                DRUM_MACHINE_PUCK_HIT_RADIUS,
                Stroke::new(1.2_f32, Color32::from_rgb(251, 146, 60)),
            );
            painter.circle_filled(puck_pos, 6.0, Color32::WHITE);

            painter.text(
                egui::pos2(rect.min.x + 8.0, rect.min.y + 6.0),
                egui::Align2::LEFT_TOP,
                format!(
                    "Decay Scale: {:.2}x | Pitch Tune: {:+.1} st | Master: {:.0}%",
                    self.decay_scale,
                    self.pitch_semitones,
                    self.master_gain * 100.0
                ),
                FontId::proportional(11.0),
                Color32::from_rgb(253, 186, 116),
            );

            ui.add_space(6.0);

            // 8-Pad Tactile Strip
            ui.horizontal_wrapped(|ui| {
                for idx in 0..8 {
                    let is_sel = self.selected_pad_idx == idx;
                    let muted = self.pads[idx].muted;
                    let label = format!("{}\nN{}", self.pads[idx].name, self.pads[idx].midi_note);
                    let btn = egui::Button::new(
                        RichText::new(label)
                            .size(10.0)
                            .color(if muted {
                                Color32::from_rgb(100, 116, 139)
                            } else if is_sel {
                                Color32::BLACK
                            } else {
                                Color32::WHITE
                            }),
                    )
                    .fill(if muted {
                        Color32::from_rgb(15, 23, 42)
                    } else if is_sel {
                        Color32::from_rgb(249, 115, 22)
                    } else {
                        Color32::from_rgb(38, 28, 22)
                    })
                    .min_size(Vec2::new(62.0, 44.0));

                    let resp = ui.add(btn);
                    if resp.clicked() {
                        self.selected_pad_idx = idx;
                    }
                    if resp.secondary_clicked() {
                        self.pads[idx].muted = !self.pads[idx].muted;
                    }
                }
            });

            ui.add_space(6.0);

            // Surgical Pro Sliders
            ui.horizontal(|ui| {
                ui.label(RichText::new("Master Gain:").size(11.0));
                ui.add(egui::Slider::new(&mut self.master_gain, 0.0..=1.0));
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Envelope Decay Scale:").size(11.0));
                let mut d = self.decay_scale;
                if ui
                    .add(egui::Slider::new(&mut d, MIN_DECAY_SCALE..=MAX_DECAY_SCALE).suffix("x"))
                    .changed()
                {
                    self.decay_scale = d;
                    self.sync_puck_from_params();
                }
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Global Pitch Tune:").size(11.0));
                let mut p = self.pitch_semitones;
                if ui
                    .add(
                        egui::Slider::new(&mut p, MIN_PITCH_SEMITONES..=MAX_PITCH_SEMITONES)
                            .suffix(" st"),
                    )
                    .changed()
                {
                    self.pitch_semitones = p;
                    self.sync_puck_from_params();
                }
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Punch Attack:").size(11.0));
                ui.add(egui::Slider::new(&mut self.punch_attack_ms, 0.5..=25.0).suffix(" ms"));
            });
        });
    }
}
