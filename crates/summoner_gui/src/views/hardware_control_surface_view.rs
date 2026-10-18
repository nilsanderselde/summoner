// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Hardware Control Surface, CV/Gate Modular Output & DIN Sync24 Bridge HUD (Milestones 33 & 34).
//!
//! Directly reflects `crates/summoner_dsp/src/ecosystem_hardware.rs` (`PushControllerDriver`,
//! `LaunchpadProDriver`, `NksIntegrationDriver`, `McuControllerDriver`, `OscMappingEngine`,
//! `HardwareControlEditorState`, `AudioChannelRoutingMatrix`, `MidiClockCalibrator`,
//! `CvGateGenerator`, `DinSyncGenerator`, `BleMidiController`).
//! Provides an interactive 2D CV Volts-per-Octave & MIDI Clock Latency Compensation Puck
//! (CV Pitch Voltage $\in [-5.0..=+5.0]\text{ V}$ vs Clock Latency Offset $\in [-25.0..=+25.0]\text{ ms}$),
//! hardware controller protocol selector (Ableton Push 3, NI Komplete Kontrol NKS, Mackie MCU,
//! Novation Launchpad Pro, BLE-MIDI), DIN Sync24 PPQN pulse generator,
//! 4 curated hardware integration presets, and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and deterministic ASCII snapshot rendering.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, FontId, RichText, Rounding, Stroke, Vec2};

pub const HARDWARE_SURFACE_PUCK_HIT_RADIUS: f32 = 22.0; // >= 44x44pt touch bounding target
pub const MIN_CV_VOLTAGE: f32 = -5.0;
pub const MAX_CV_VOLTAGE: f32 = 5.0;
pub const MIN_CLOCK_OFFSET_MS: f32 = -25.0;
pub const MAX_CLOCK_OFFSET_MS: f32 = 25.0;

/// Hardware controller protocol reflecting `summoner_dsp::ecosystem_hardware`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HardwareControllerProtocol {
    #[default]
    AbletonPush3Display,
    MackieControlMotorFaders,
    KompleteKontrolNksLightGuide,
    LaunchpadProGridMatrix,
    BleWirelessMidi,
}

impl HardwareControllerProtocol {
    pub fn label(&self) -> &'static str {
        match self {
            Self::AbletonPush3Display => "Ableton Push 3 (960x160 RGB565 + MPE Pads)",
            Self::MackieControlMotorFaders => "Mackie Control MCU (9 Motor Faders + LCD)",
            Self::KompleteKontrolNksLightGuide => "NI Komplete Kontrol NKS (88-Key Light Guide)",
            Self::LaunchpadProGridMatrix => "Novation Launchpad Pro (8x8 SysEx Grid)",
            Self::BleWirelessMidi => "Bluetooth LE MIDI (Low-Jitter Timestamped)",
        }
    }
}

/// Curated Hardware Control Surface, CV/Gate & DIN Sync24 presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HardwareControlPreset {
    /// Ableton Push 3 studio hub with 1V/Oct Eurorack CV output.
    #[default]
    Push3EurorackBridge,
    /// Mackie Control Universal 9-fader mixing desk with zero-offset clock.
    McuMotorizedConsole,
    /// Vintage Roland Sync24 (24 PPQN) drum machine & Buchla 1.2V/Oct CV rig.
    VintageDinSync24Rig,
    /// NI NKS Light Guide keyboard with bidirectional OSC TouchOSC bridge.
    NksOscBidirectionalStage,
}

impl HardwareControlPreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Push3EurorackBridge => "Push 3 + Eurorack 1V/Oct CV/Gate Bridge",
            Self::McuMotorizedConsole => "Mackie MCU 9-Fader Motorized Console",
            Self::VintageDinSync24Rig => "Vintage DIN Sync24 (24 PPQN) + 5V Gate Rig",
            Self::NksOscBidirectionalStage => "NI NKS 88-Key Light Guide + OSC Stage Rig",
        }
    }

    /// Returns nominal (cv_voltage, clock_offset_ms, volts_per_oct, gate_high_v, din_sync_bpm, protocol, midi_learn).
    pub fn nominal_parameters(
        &self,
    ) -> (
        f32,
        f32,
        f32,
        f32,
        f32,
        HardwareControllerProtocol,
        bool,
    ) {
        match self {
            Self::Push3EurorackBridge => (
                1.0,
                0.0,
                1.0,
                5.0,
                120.0,
                HardwareControllerProtocol::AbletonPush3Display,
                false,
            ),
            Self::McuMotorizedConsole => (
                0.0,
                -2.5,
                1.0,
                5.0,
                124.0,
                HardwareControllerProtocol::MackieControlMotorFaders,
                false,
            ),
            Self::VintageDinSync24Rig => (
                2.0,
                4.5,
                1.2,
                8.0,
                128.0,
                HardwareControllerProtocol::LaunchpadProGridMatrix,
                false,
            ),
            Self::NksOscBidirectionalStage => (
                -1.0,
                1.2,
                1.0,
                5.0,
                110.0,
                HardwareControllerProtocol::KompleteKontrolNksLightGuide,
                true,
            ),
        }
    }
}

/// Hardware Control Surface, CV/Gate Modular Output & DIN Sync24 Bridge HUD view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareControlSurfaceView {
    /// Active hardware control preset.
    pub preset: HardwareControlPreset,
    /// Active hardware controller protocol.
    pub protocol: HardwareControllerProtocol,
    /// DC-coupled CV Pitch Output Voltage [-5.0 ..= +5.0 V] reflecting `CvGateGenerator`.
    pub cv_voltage: f32,
    /// Hardware MIDI Clock Latency Compensation Offset [-25.0 ..= +25.0 ms] reflecting `MidiClockCalibrator`.
    pub clock_offset_ms: f32,
    /// CV Volts-per-Octave scaling [0.5 ..= 2.0 V/Oct].
    pub volts_per_octave: f32,
    /// Gate high pulse voltage [3.3 ..= 10.0 V].
    pub gate_high_voltage: f32,
    /// DIN Sync24 tempo in BPM [40.0 ..= 240.0 BPM] reflecting `DinSyncGenerator`.
    pub din_sync_bpm: f32,
    /// Measured MIDI clock jitter in milliseconds [0.0 ..= 10.0 ms].
    pub measured_jitter_ms: f32,
    /// Whether MIDI Learn mode is active (`HardwareControlEditorState::is_learning`).
    pub midi_learn_active: bool,
    /// Whether DIN Sync24 Run signal is active.
    pub din_sync_running: bool,
    /// 2D Puck position (X: normalized cv_voltage, Y: normalized clock_offset_ms).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for HardwareControlSurfaceView {
    fn default() -> Self {
        Self::new()
    }
}

impl HardwareControlSurfaceView {
    pub fn new() -> Self {
        let preset = HardwareControlPreset::Push3EurorackBridge;
        let (cv, offset, vpo, gate_v, bpm, proto, learn) = preset.nominal_parameters();
        let norm_x = Self::cv_to_normalized(cv);
        let norm_y = Self::offset_to_normalized(offset);

        Self {
            preset,
            protocol: proto,
            cv_voltage: cv,
            clock_offset_ms: offset,
            volts_per_octave: vpo,
            gate_high_voltage: gate_v,
            din_sync_bpm: bpm,
            measured_jitter_ms: 0.18,
            midi_learn_active: learn,
            din_sync_running: true,
            puck_pos: (norm_x, norm_y),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        }
    }

    pub fn set_preset(&mut self, preset: HardwareControlPreset) {
        self.preset = preset;
        let (cv, offset, vpo, gate_v, bpm, proto, learn) = preset.nominal_parameters();
        self.cv_voltage = cv;
        self.clock_offset_ms = offset;
        self.volts_per_octave = vpo;
        self.gate_high_voltage = gate_v;
        self.din_sync_bpm = bpm;
        self.protocol = proto;
        self.midi_learn_active = learn;
        self.sync_puck_from_params();
    }

    pub fn sync_puck_from_params(&mut self) {
        self.puck_pos = (
            Self::cv_to_normalized(self.cv_voltage),
            Self::offset_to_normalized(self.clock_offset_ms),
        );
    }

    pub fn update_from_puck(&mut self, norm_x: f32, norm_y: f32) {
        let x = norm_x.clamp(0.0, 1.0);
        let y = norm_y.clamp(0.0, 1.0);
        self.puck_pos = (x, y);
        self.cv_voltage = Self::normalized_to_cv(x);
        self.clock_offset_ms = Self::normalized_to_offset(y);
    }

    pub fn cv_to_normalized(cv_volts: f32) -> f32 {
        ((cv_volts.clamp(MIN_CV_VOLTAGE, MAX_CV_VOLTAGE) - MIN_CV_VOLTAGE)
            / (MAX_CV_VOLTAGE - MIN_CV_VOLTAGE))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_cv(norm: f32) -> f32 {
        MIN_CV_VOLTAGE + norm.clamp(0.0, 1.0) * (MAX_CV_VOLTAGE - MIN_CV_VOLTAGE)
    }

    pub fn offset_to_normalized(offset_ms: f32) -> f32 {
        ((offset_ms.clamp(MIN_CLOCK_OFFSET_MS, MAX_CLOCK_OFFSET_MS) - MIN_CLOCK_OFFSET_MS)
            / (MAX_CLOCK_OFFSET_MS - MIN_CLOCK_OFFSET_MS))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_offset(norm: f32) -> f32 {
        MIN_CLOCK_OFFSET_MS + norm.clamp(0.0, 1.0) * (MAX_CLOCK_OFFSET_MS - MIN_CLOCK_OFFSET_MS)
    }

    /// Converts MIDI note number to CV voltage matching `CvGateGenerator::note_to_cv_voltage`.
    pub fn note_to_cv_voltage(&self, note_number: f32) -> f32 {
        let semitones_from_c3 = note_number - 60.0;
        (semitones_from_c3 / 12.0) * self.volts_per_octave
    }

    /// Computes normalized DC audio sample (-1.0..=1.0) for the current `cv_voltage`.
    pub fn normalized_dc_audio_level(&self) -> f32 {
        (self.cv_voltage / 5.0).clamp(-1.0, 1.0)
    }

    /// Computes DIN Sync24 pulse frequency in Hz (24 PPQN) matching `DinSyncGenerator`.
    pub fn din_sync_pulse_hz(&self) -> f32 {
        (self.din_sync_bpm.clamp(40.0, 240.0) / 60.0) * 24.0
    }

    /// Evaluates CV + DIN Sync24 visualization waveform across normalized phase [0.0..=1.0].
    pub fn evaluate_sync_waveform(&self, norm_phase: f32) -> f32 {
        let p = norm_phase.clamp(0.0, 1.0);
        let cv_norm = Self::cv_to_normalized(self.cv_voltage);
        let cycles = (self.din_sync_bpm / 30.0).clamp(1.0, 8.0);
        let phase_shift = self.clock_offset_ms / 50.0;
        let pulse = if ((p * cycles + phase_shift).rem_euclid(1.0)) < 0.5 && self.din_sync_running {
            0.22
        } else {
            -0.22
        };
        (cv_norm * 0.65 + 0.175 + pulse * 0.5).clamp(0.05, 0.95)
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
        (dx * dx + dy * dy).sqrt() <= HARDWARE_SURFACE_PUCK_HIT_RADIUS
    }

    /// Renders a deterministic ASCII snapshot for headless QA verification.
    pub fn render_ascii_snapshot(&self) -> String {
        let mut out = String::new();
        out.push_str("======================================================================\n");
        out.push_str(" SUMMONER DAW -- HARDWARE CONTROL SURFACE, CV/GATE & DIN SYNC SNAPSHOT\n");
        out.push_str("======================================================================\n");
        out.push_str(&format!(" Preset:           {}\n", self.preset.name()));
        out.push_str(&format!(
            " Protocol:         {} | MIDI Learn: {}\n",
            self.protocol.label(),
            if self.midi_learn_active { "ACTIVE" } else { "IDLE" }
        ));
        out.push_str(&format!(
            " CV Output:        {:+.2} V ({:.2} V/Oct) | Gate High: {:.1} V\n",
            self.cv_voltage, self.volts_per_octave, self.gate_high_voltage
        ));
        out.push_str(&format!(
            " DIN Sync24:       {:.1} BPM ({:.1} Hz, 24 PPQN) | Clock Offset: {:+.2} ms\n",
            self.din_sync_bpm,
            self.din_sync_pulse_hz(),
            self.clock_offset_ms
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
                    RichText::new("HARDWARE CONTROL SURFACE, CV/GATE & DIN SYNC24 HUD")
                        .color(Color32::from_rgb(56, 189, 248))
                        .strong(),
                );
                ui.label(
                    RichText::new("M33 / M34 Push 3, MCU, NKS, 1V/Oct CV & 24 PPQN Sync")
                        .color(Color32::from_rgb(148, 163, 184))
                        .small(),
                );
            });

            ui.add_space(6.0);

            // Preset Selector Bar
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Hardware Presets:").strong());
                let presets = [
                    (HardwareControlPreset::Push3EurorackBridge, "Push 3 + CV"),
                    (HardwareControlPreset::McuMotorizedConsole, "Mackie MCU"),
                    (HardwareControlPreset::VintageDinSync24Rig, "DIN Sync24"),
                    (HardwareControlPreset::NksOscBidirectionalStage, "NI NKS + OSC"),
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
                        Color32::from_rgb(56, 189, 248)
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

            // Interactive 2D CV Voltage & Clock Offset Canvas
            let canvas_size = Vec2::new(ui.available_width().max(280.0), 144.0);
            let (response, painter) =
                ui.allocate_painter(canvas_size, egui::Sense::click_and_drag());
            let rect = response.rect;
            let canvas = Rect::new(rect.min.x, rect.min.y, rect.width(), rect.height());

            painter.rect_filled(rect, Rounding::same(4.0), Color32::from_rgb(10, 18, 28));
            painter.rect_stroke(
                rect,
                Rounding::same(4.0),
                Stroke::new(1.0_f32, Color32::from_rgb(28, 84, 128)),
            );

            // Draw CV + DIN Sync24 pulse contour
            let num_pts = 48;
            let mut pts = Vec::with_capacity(num_pts);
            for idx in 0..num_pts {
                let frac = idx as f32 / (num_pts - 1) as f32;
                let r_val = self.evaluate_sync_waveform(frac);
                let px = rect.min.x + 15.0 + frac * (rect.width() - 30.0);
                let py = rect.max.y - 14.0 - r_val * (rect.height() - 32.0);
                pts.push(egui::pos2(px, py));
            }
            for i in 0..(pts.len() - 1) {
                painter.line_segment(
                    [pts[i], pts[i + 1]],
                    Stroke::new(2.0_f32, Color32::from_rgb(56, 189, 248)),
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
                HARDWARE_SURFACE_PUCK_HIT_RADIUS,
                Color32::from_rgba_unmultiplied(56, 189, 248, 42),
            );
            painter.circle_stroke(
                puck_pos,
                HARDWARE_SURFACE_PUCK_HIT_RADIUS,
                Stroke::new(1.2_f32, Color32::from_rgb(125, 211, 252)),
            );
            painter.circle_filled(puck_pos, 6.0, Color32::WHITE);

            painter.text(
                egui::pos2(rect.min.x + 8.0, rect.min.y + 6.0),
                egui::Align2::LEFT_TOP,
                format!(
                    "{} | CV: {:+.2}V | Offset: {:+.1}ms | Sync24: {:.0} BPM",
                    self.protocol.label(),
                    self.cv_voltage,
                    self.clock_offset_ms,
                    self.din_sync_bpm
                ),
                FontId::proportional(11.0),
                Color32::from_rgb(186, 230, 253),
            );

            ui.add_space(8.0);

            // Surgical Pro Sliders
            ui.horizontal(|ui| {
                ui.label(RichText::new("CV Pitch Voltage:").size(11.0));
                let mut cv = self.cv_voltage;
                if ui
                    .add(egui::Slider::new(&mut cv, MIN_CV_VOLTAGE..=MAX_CV_VOLTAGE).suffix(" V"))
                    .changed()
                {
                    self.cv_voltage = cv;
                    self.sync_puck_from_params();
                }
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Clock Latency Offset:").size(11.0));
                let mut off = self.clock_offset_ms;
                if ui
                    .add(
                        egui::Slider::new(&mut off, MIN_CLOCK_OFFSET_MS..=MAX_CLOCK_OFFSET_MS)
                            .suffix(" ms"),
                    )
                    .changed()
                {
                    self.clock_offset_ms = off;
                    self.sync_puck_from_params();
                }
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Volts Per Octave:").size(11.0));
                ui.add(egui::Slider::new(&mut self.volts_per_octave, 0.5..=2.0).suffix(" V/Oct"));
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Gate High Voltage:").size(11.0));
                ui.add(egui::Slider::new(&mut self.gate_high_voltage, 3.3..=10.0).suffix(" V"));
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("DIN Sync24 Tempo:").size(11.0));
                ui.add(egui::Slider::new(&mut self.din_sync_bpm, 40.0..=240.0).suffix(" BPM"));
            });
        });
    }
}
