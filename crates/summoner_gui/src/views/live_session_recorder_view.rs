// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Master Output Live Session Recorder & Direct-to-Disk WAV/FLAC Capture HUD (Milestones 33 & 34).
//!
//! Directly reflects `crates/summoner_dsp/src/live_session_recorder.rs` (`LiveSessionRecorder`,
//! `RecordingFormat`, `RecordingStats`, stereo PCM clamping and real-time buffered disk streaming).
//! Provides an interactive 2D Input Trim Gain & Peak Safety Ceiling Puck
//! (Input Trim $\in [-18.0..=+6.0]\text{ dB}$ vs Safety Ceiling $\in [-6.0..=0.0]\text{ dBFS}$),
//! real-time stereo L/R waveform capture monitor, disk bitrate/size estimator,
//! 4 curated session capture presets, and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and deterministic ASCII snapshot rendering.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, FontId, RichText, Rounding, Stroke, Vec2};

pub const LIVE_RECORDER_PUCK_HIT_RADIUS: f32 = 22.0; // >= 44x44pt touch bounding target
pub const MIN_INPUT_TRIM_DB: f32 = -18.0;
pub const MAX_INPUT_TRIM_DB: f32 = 6.0;
pub const MIN_SAFETY_CEILING_DB: f32 = -6.0;
pub const MAX_SAFETY_CEILING_DB: f32 = 0.0;

/// Supported live session disk recording formats reflecting `summoner_dsp::live_session_recorder::RecordingFormat`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LiveRecordDiskFormat {
    #[default]
    Wav16BitPcm,
    Wav24BitPcm,
    FlacLossless24,
}

impl LiveRecordDiskFormat {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Wav16BitPcm => "WAV 16-bit PCM",
            Self::Wav24BitPcm => "WAV 24-bit PCM",
            Self::FlacLossless24 => "FLAC 24-bit Lossless",
        }
    }

    pub fn bytes_per_sample(&self) -> u64 {
        match self {
            Self::Wav16BitPcm => 2,
            Self::Wav24BitPcm => 3,
            Self::FlacLossless24 => 2, // ~65% lossless compression ratio of 3 bytes
        }
    }
}

/// Curated master output live session recorder presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LiveRecorderPreset {
    /// Standard 44.1 kHz / 16-bit Red Book WAV stereo master capture.
    #[default]
    MasterStereoWav44k,
    /// Studio archival 48 kHz / 24-bit lossless FLAC capture.
    ArchivalFlac48k,
    /// Broadcast safety-limited -1.0 dBFS ceiling master session capture.
    BroadcastSafeMaster,
    /// High-headroom -6.0 dB input trim live jam & modular improvisation capture.
    ModularJamHeadroom,
}

impl LiveRecorderPreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::MasterStereoWav44k => "Master Stereo WAV (44.1 kHz / 16-bit)",
            Self::ArchivalFlac48k => "Archival Studio FLAC (48 kHz / 24-bit)",
            Self::BroadcastSafeMaster => "Broadcast Safe Master (-1.0 dBFS Ceiling)",
            Self::ModularJamHeadroom => "Modular Live Jam (-6.0 dB Headroom)",
        }
    }

    /// Returns nominal (input_trim_db, safety_ceiling_db, sample_rate_hz, format, auto_tail_sec).
    pub fn nominal_parameters(&self) -> (f32, f32, u32, LiveRecordDiskFormat, f32) {
        match self {
            Self::MasterStereoWav44k => (0.0, -0.1, 44100, LiveRecordDiskFormat::Wav16BitPcm, 2.0),
            Self::ArchivalFlac48k => (0.0, -0.3, 48000, LiveRecordDiskFormat::FlacLossless24, 3.5),
            Self::BroadcastSafeMaster => (-1.5, -1.0, 48000, LiveRecordDiskFormat::Wav24BitPcm, 1.5),
            Self::ModularJamHeadroom => (-6.0, -0.5, 44100, LiveRecordDiskFormat::Wav24BitPcm, 4.0),
        }
    }
}

/// Master Output Live Session Recorder & Direct-to-Disk Capture HUD view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveSessionRecorderView {
    /// Active session capture preset.
    pub preset: LiveRecorderPreset,
    /// Master input trim gain in dB [-18.0 ..= +6.0].
    pub input_trim_db: f32,
    /// Brickwall safety ceiling in dBFS [-6.0 ..= 0.0].
    pub safety_ceiling_db: f32,
    /// Target disk sample rate in Hz (44100, 48000, 96000).
    pub sample_rate: u32,
    /// Target disk recording format (WAV / FLAC).
    pub format: LiveRecordDiskFormat,
    /// Auto-release reverb tail capture duration in seconds [0.0 ..= 10.0].
    pub auto_tail_sec: f32,
    /// Whether live disk capture is currently armed and recording.
    pub is_recording: bool,
    /// Total stereo frames captured in current or last session.
    pub captured_frames: u64,
    /// Target file path string.
    pub output_file_path: String,
    /// 2D Puck position (X: normalized input trim dB, Y: normalized safety ceiling dBFS).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for LiveSessionRecorderView {
    fn default() -> Self {
        Self::new()
    }
}

impl LiveSessionRecorderView {
    pub fn new() -> Self {
        let preset = LiveRecorderPreset::MasterStereoWav44k;
        let (trim, ceiling, sr, fmt, tail) = preset.nominal_parameters();
        let norm_x = Self::trim_to_normalized(trim);
        let norm_y = Self::ceiling_to_normalized(ceiling);

        Self {
            preset,
            input_trim_db: trim,
            safety_ceiling_db: ceiling,
            sample_rate: sr,
            format: fmt,
            auto_tail_sec: tail,
            is_recording: false,
            captured_frames: 44100 * 12, // 12.0s nominal preview
            output_file_path: "renders/live_session_master.wav".to_string(),
            puck_pos: (norm_x, norm_y),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        }
    }

    pub fn set_preset(&mut self, preset: LiveRecorderPreset) {
        self.preset = preset;
        let (trim, ceiling, sr, fmt, tail) = preset.nominal_parameters();
        self.input_trim_db = trim;
        self.safety_ceiling_db = ceiling;
        self.sample_rate = sr;
        self.format = fmt;
        self.auto_tail_sec = tail;
        self.output_file_path = match fmt {
            LiveRecordDiskFormat::FlacLossless24 => "renders/live_session_master.flac".to_string(),
            _ => "renders/live_session_master.wav".to_string(),
        };
        self.sync_puck_from_params();
    }

    pub fn sync_puck_from_params(&mut self) {
        self.puck_pos = (
            Self::trim_to_normalized(self.input_trim_db),
            Self::ceiling_to_normalized(self.safety_ceiling_db),
        );
    }

    pub fn update_from_puck(&mut self, norm_x: f32, norm_y: f32) {
        let x = norm_x.clamp(0.0, 1.0);
        let y = norm_y.clamp(0.0, 1.0);
        self.puck_pos = (x, y);
        self.input_trim_db = Self::normalized_to_trim(x);
        self.safety_ceiling_db = Self::normalized_to_ceiling(y);
    }

    pub fn trim_to_normalized(trim_db: f32) -> f32 {
        let t = trim_db.clamp(MIN_INPUT_TRIM_DB, MAX_INPUT_TRIM_DB);
        ((t - MIN_INPUT_TRIM_DB) / (MAX_INPUT_TRIM_DB - MIN_INPUT_TRIM_DB)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_trim(norm: f32) -> f32 {
        let n = norm.clamp(0.0, 1.0);
        MIN_INPUT_TRIM_DB + n * (MAX_INPUT_TRIM_DB - MIN_INPUT_TRIM_DB)
    }

    pub fn ceiling_to_normalized(ceiling_db: f32) -> f32 {
        let c = ceiling_db.clamp(MIN_SAFETY_CEILING_DB, MAX_SAFETY_CEILING_DB);
        ((c - MIN_SAFETY_CEILING_DB) / (MAX_SAFETY_CEILING_DB - MIN_SAFETY_CEILING_DB))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_ceiling(norm: f32) -> f32 {
        let n = norm.clamp(0.0, 1.0);
        MIN_SAFETY_CEILING_DB + n * (MAX_SAFETY_CEILING_DB - MIN_SAFETY_CEILING_DB)
    }

    /// Computes linear amplitude gain from `input_trim_db`.
    pub fn linear_trim_gain(&self) -> f32 {
        10.0_f32.powf(self.input_trim_db / 20.0)
    }

    /// Computes linear safety ceiling clamp threshold from `safety_ceiling_db`.
    pub fn linear_ceiling_limit(&self) -> f32 {
        10.0_f32.powf(self.safety_ceiling_db / 20.0).clamp(0.1, 1.0)
    }

    /// Clamps and quantizes a floating-point sample to 16-bit PCM matching `LiveSessionRecorder::process_block`.
    pub fn quantize_sample_i16(&self, sample: f32) -> i16 {
        let trimmed = sample * self.linear_trim_gain();
        let ceil = self.linear_ceiling_limit();
        let clamped = trimmed.clamp(-ceil, ceil).clamp(-1.0, 1.0);
        (clamped * i16::MAX as f32).round() as i16
    }

    /// Computes recorded duration in seconds.
    pub fn duration_seconds(&self) -> f64 {
        if self.sample_rate == 0 {
            0.0
        } else {
            self.captured_frames as f64 / self.sample_rate as f64
        }
    }

    /// Estimates disk file size in bytes (44-byte WAV header + stereo sample payload).
    pub fn estimated_file_size_bytes(&self) -> u64 {
        44 + self.captured_frames * 2 * self.format.bytes_per_sample()
    }

    /// Evaluates synthetic master stereo waveform envelope at normalized time t in [0.0..=1.0].
    pub fn evaluate_waveform_envelope(&self, norm_t: f32) -> f32 {
        let t = norm_t.clamp(0.0, 1.0);
        let raw = ((t * 6.0 * PI).sin() * 0.65 + (t * 14.0 * PI).cos() * 0.35) * self.linear_trim_gain();
        let ceil = self.linear_ceiling_limit();
        raw.clamp(-ceil, ceil)
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
        (dx * dx + dy * dy).sqrt() <= LIVE_RECORDER_PUCK_HIT_RADIUS
    }

    /// Renders a deterministic ASCII snapshot for headless QA verification.
    pub fn render_ascii_snapshot(&self) -> String {
        let mut out = String::new();
        out.push_str("======================================================================\n");
        out.push_str(" SUMMONER DAW -- MASTER LIVE SESSION RECORDER & DISK CAPTURE SNAPSHOT\n");
        out.push_str("======================================================================\n");
        out.push_str(&format!(" Preset:           {}\n", self.preset.name()));
        out.push_str(&format!(
            " Status:           {} | Format: {} @ {} Hz\n",
            if self.is_recording { "RECORDING [ON-AIR]" } else { "STANDBY [READY]" },
            self.format.label(),
            self.sample_rate
        ));
        out.push_str(&format!(
            " Input Trim:       {:+.1} dB ({:.2}x) | Safety Ceiling: {:.1} dBFS\n",
            self.input_trim_db,
            self.linear_trim_gain(),
            self.safety_ceiling_db
        ));
        out.push_str(&format!(
            " Duration:         {:.2} s ({} frames) | Disk Size: {} KB\n",
            self.duration_seconds(),
            self.captured_frames,
            self.estimated_file_size_bytes() / 1024
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
                    RichText::new("MASTER LIVE SESSION RECORDER & DISK CAPTURE HUD")
                        .color(Color32::from_rgb(239, 68, 68))
                        .strong(),
                );
                ui.label(
                    RichText::new("M33 / M34 Direct-to-Disk WAV/FLAC Streamer")
                        .color(Color32::from_rgb(148, 163, 184))
                        .small(),
                );
            });

            ui.add_space(6.0);

            // Preset Selector Bar
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Presets:").strong());
                let presets = [
                    (LiveRecorderPreset::MasterStereoWav44k, "WAV 44.1k"),
                    (LiveRecorderPreset::ArchivalFlac48k, "FLAC 48k"),
                    (LiveRecorderPreset::BroadcastSafeMaster, "Broadcast -1dB"),
                    (LiveRecorderPreset::ModularJamHeadroom, "Jam -6dB"),
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
                        Color32::from_rgb(239, 68, 68)
                    } else {
                        Color32::from_rgb(30, 41, 59)
                    })
                    .min_size(Vec2::new(82.0, 24.0));

                    if ui.add(btn).clicked() {
                        self.set_preset(p);
                    }
                }

                let rec_label = if self.is_recording { "⏹ STOP REC" } else { "⏺ ARM & RECORD" };
                let rec_btn = egui::Button::new(
                    RichText::new(rec_label)
                        .color(Color32::WHITE)
                        .strong()
                        .size(11.0),
                )
                .fill(if self.is_recording {
                    Color32::from_rgb(220, 38, 38)
                } else {
                    Color32::from_rgb(127, 29, 29)
                })
                .min_size(Vec2::new(108.0, 24.0));

                if ui.add(rec_btn).clicked() {
                    self.is_recording = !self.is_recording;
                }
            });

            ui.add_space(8.0);

            // Interactive 2D Input Trim & Safety Ceiling Canvas
            let canvas_size = Vec2::new(ui.available_width().max(280.0), 144.0);
            let (response, painter) =
                ui.allocate_painter(canvas_size, egui::Sense::click_and_drag());
            let rect = response.rect;
            let canvas = Rect::new(rect.min.x, rect.min.y, rect.width(), rect.height());

            painter.rect_filled(rect, Rounding::same(4.0), Color32::from_rgb(22, 10, 14));
            painter.rect_stroke(
                rect,
                Rounding::same(4.0),
                Stroke::new(1.0_f32, Color32::from_rgb(110, 32, 46)),
            );

            // Draw safety ceiling lines
            let ceil_frac = self.linear_ceiling_limit();
            let top_ceil_y = rect.center().y - ceil_frac * (rect.height() * 0.38);
            let bot_ceil_y = rect.center().y + ceil_frac * (rect.height() * 0.38);
            for cy in [top_ceil_y, bot_ceil_y] {
                painter.line_segment(
                    [egui::pos2(rect.min.x, cy), egui::pos2(rect.max.x, cy)],
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(248, 113, 113, 95)),
                );
            }

            // Draw captured waveform preview
            let num_pts = 48;
            let mut pts = Vec::with_capacity(num_pts);
            for idx in 0..num_pts {
                let frac = idx as f32 / (num_pts - 1) as f32;
                let w_val = self.evaluate_waveform_envelope(frac);
                let px = rect.min.x + 15.0 + frac * (rect.width() - 30.0);
                let py = rect.center().y - w_val * (rect.height() * 0.38);
                pts.push(egui::pos2(px, py));
            }
            for i in 0..(pts.len() - 1) {
                painter.line_segment(
                    [pts[i], pts[i + 1]],
                    Stroke::new(1.8_f32, Color32::from_rgb(248, 113, 113)),
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
                LIVE_RECORDER_PUCK_HIT_RADIUS,
                Color32::from_rgba_unmultiplied(239, 68, 68, 42),
            );
            painter.circle_stroke(
                puck_pos,
                LIVE_RECORDER_PUCK_HIT_RADIUS,
                Stroke::new(1.2_f32, Color32::from_rgb(248, 113, 113)),
            );
            painter.circle_filled(puck_pos, 6.0, Color32::WHITE);

            painter.text(
                egui::pos2(rect.min.x + 8.0, rect.min.y + 6.0),
                egui::Align2::LEFT_TOP,
                format!(
                    "Trim: {:+.1} dB | Ceiling: {:.1} dBFS | {:.1}s ({} KB)",
                    self.input_trim_db,
                    self.safety_ceiling_db,
                    self.duration_seconds(),
                    self.estimated_file_size_bytes() / 1024
                ),
                FontId::proportional(11.0),
                Color32::from_rgb(252, 165, 165),
            );

            ui.add_space(8.0);

            // Surgical Pro Sliders
            ui.horizontal(|ui| {
                ui.label(RichText::new("Input Trim Gain:").size(11.0));
                let mut t = self.input_trim_db;
                if ui
                    .add(egui::Slider::new(&mut t, MIN_INPUT_TRIM_DB..=MAX_INPUT_TRIM_DB).suffix(" dB"))
                    .changed()
                {
                    self.input_trim_db = t;
                    self.sync_puck_from_params();
                }
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Safety Ceiling:").size(11.0));
                let mut c = self.safety_ceiling_db;
                if ui
                    .add(
                        egui::Slider::new(&mut c, MIN_SAFETY_CEILING_DB..=MAX_SAFETY_CEILING_DB)
                            .suffix(" dBFS"),
                    )
                    .changed()
                {
                    self.safety_ceiling_db = c;
                    self.sync_puck_from_params();
                }
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Auto-Tail Capture:").size(11.0));
                ui.add(egui::Slider::new(&mut self.auto_tail_sec, 0.0..=10.0).suffix(" s"));
            });
        });
    }
}
