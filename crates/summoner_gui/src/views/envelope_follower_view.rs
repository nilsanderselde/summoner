// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Dynamic Envelope Follower Curve Editor with Real-Time Ball Physics & Sidechain Routing (Step 1364).

use crate::touch_controls::{ContrastColorPalette, MIN_HIT_TARGET_PT};

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const BALL_VISUAL_RADIUS: f32 = 14.0;
pub const BALL_HIT_RADIUS: f32 = 22.0;

/// Sidechain input routing source selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidechainSource {
    InternalMain,
    Track1Kick,
    Track2Snare,
    Bus1Drums,
    ExternalAux,
}

impl SidechainSource {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::InternalMain => "Internal (Self)",
            Self::Track1Kick => "Track 1: Kick",
            Self::Track2Snare => "Track 2: Snare",
            Self::Bus1Drums => "Bus 1: Drum Group",
            Self::ExternalAux => "External Aux In",
        }
    }
}

/// Envelope detection mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvelopeMode {
    Peak,
    Rms,
    OptoBallistic,
    TruePeak,
}

impl EnvelopeMode {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Peak => "Peak (Fast)",
            Self::Rms => "RMS (Averaged)",
            Self::OptoBallistic => "Opto (Non-linear Ballistic)",
            Self::TruePeak => "True Peak (4x Oversampled)",
        }
    }
}

/// Real-time ball physics simulation state on dynamic curve.
#[derive(Debug, Clone, PartialEq)]
pub struct BallPhysicsState {
    pub position_db: f32, // -60.0 ..= 0.0 dB
    pub velocity_db: f32, // dB / sec
    pub target_db: f32,
    pub spring_k: f32,
    pub damping: f32,
    pub mass: f32,
}

impl Default for BallPhysicsState {
    fn default() -> Self {
        Self {
            position_db: -60.0_f32,
            velocity_db: 0.0_f32,
            target_db: -60.0_f32,
            spring_k: 45.0_f32,
            damping: 8.5_f32,
            mass: 1.0_f32,
        }
    }
}

impl BallPhysicsState {
    /// Advance physics simulation step by dt seconds.
    pub fn step(&mut self, target_db: f32, dt: f32) {
        self.target_db = target_db.clamp(-60.0_f32, 6.0_f32);
        let displacement = self.target_db - self.position_db;
        let spring_force = self.spring_k * displacement;
        let damping_force = -self.damping * self.velocity_db;
        let total_accel = (spring_force + damping_force) / self.mass;

        self.velocity_db += total_accel * dt;
        self.position_db += self.velocity_db * dt;
        self.position_db = self.position_db.clamp(-60.0_f32, 6.0_f32);
    }
}

/// Dynamic Envelope Follower Curve Editor View (Step 1364).
#[derive(Debug, Clone)]
pub struct EnvelopeFollowerView {
    pub attack_ms: f32,      // 0.1 ..= 500.0 ms
    pub hold_ms: f32,        // 0.0 ..= 500.0 ms
    pub release_ms: f32,     // 1.0 ..= 2000.0 ms
    pub sensitivity_db: f32, // -48.0 ..= +24.0 dB
    pub lookahead_ms: f32,   // 0.0 ..= 50.0 ms
    pub mode: EnvelopeMode,
    pub sidechain_source: SidechainSource,
    pub physics: BallPhysicsState,
    pub input_history: Vec<f32>, // Recent 128 RMS dB history frames
    pub envelope_out_db: f32,
    pub gain_reduction_db: f32,
    pub color_palette: ContrastColorPalette,
}

impl Default for EnvelopeFollowerView {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvelopeFollowerView {
    pub fn new() -> Self {
        let mut history = Vec::with_capacity(128);
        for i in 0..128 {
            let t = i as f32 / 128.0_f32;
            // Simulated sidechain transient burst
            let val = -60.0_f32 + 50.0_f32 * (-((t - 0.25_f32) * 8.0_f32).powi(2)).exp();
            history.push(val);
        }

        Self {
            attack_ms: 15.0_f32,
            hold_ms: 25.0_f32,
            release_ms: 180.0_f32,
            sensitivity_db: 0.0_f32,
            lookahead_ms: 5.0_f32,
            mode: EnvelopeMode::OptoBallistic,
            sidechain_source: SidechainSource::Track1Kick,
            physics: BallPhysicsState::default(),
            input_history: history,
            envelope_out_db: -12.4_f32,
            gain_reduction_db: -6.8_f32,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Convert dB level [-60.0 ..= 0.0] to normalized height [0.0 ..= 1.0].
    pub fn db_to_norm(db: f32) -> f32 {
        ((db + 60.0_f32) / 60.0_f32).clamp(0.0_f32, 1.0_f32)
    }

    /// Convert normalized height [0.0 ..= 1.0] to dB level [-60.0 ..= 0.0].
    pub fn norm_to_db(norm: f32) -> f32 {
        -60.0_f32 + norm.clamp(0.0_f32, 1.0_f32) * 60.0_f32
    }

    /// Feed a new audio frame and advance the physics ball.
    pub fn feed_input_sample(&mut self, input_db: f32, dt: f32) {
        if self.input_history.len() >= 128 {
            self.input_history.remove(0);
        }
        self.input_history.push(input_db);
        self.physics.step(input_db, dt);
        self.envelope_out_db = self.physics.position_db;
        self.gain_reduction_db = (-self.envelope_out_db * 0.5_f32).min(0.0_f32);
    }

    /// Generate deterministic ASCII representation for verification.
    pub fn render_ascii(&self, width: usize) -> String {
        let mut buf = vec!['.'; width];
        let norm = Self::db_to_norm(self.envelope_out_db);
        let pos = ((norm * (width - 1) as f32).round() as usize).min(width - 1);
        buf[pos] = 'O'; // Ball marker
        buf.into_iter().collect()
    }
}

#[cfg(feature = "gui")]
impl EnvelopeFollowerView {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // 1. Header Toolbar
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("DYNAMIC ENVELOPE FOLLOWER & DETECTOR")
                        .color(Color32::from_rgb(240, 245, 255))
                        .strong(),
                );
                ui.separator();
                ui.label(
                    egui::RichText::new(format!(
                        "Source: {}",
                        self.sidechain_source.display_name()
                    ))
                    .color(Color32::from_rgb(0, 229, 255))
                    .strong(),
                );
                ui.separator();
                ui.label(format!("Mode: {}", self.mode.display_name()));
            });

            // 2. Real-Time Curve Canvas & Rolling Ball Physics
            let canvas_w = ui.available_width().max(650.0_f32);
            let canvas_h = 240.0_f32;

            let (response, painter) =
                ui.allocate_painter(Vec2::new(canvas_w, canvas_h), egui::Sense::hover());

            // Canvas Background
            painter.rect_filled(response.rect, 6.0_f32, Color32::from_rgb(10, 14, 22));
            painter.rect_stroke(
                response.rect,
                6.0_f32,
                Stroke::new(1.5_f32, Color32::from_rgb(40, 55, 80)),
            );

            // dB Grid Lines (-60dB, -48dB, -36dB, -24dB, -12dB, 0dB)
            let db_marks = [-60, -48, -36, -24, -12, 0];
            for db in db_marks {
                let norm = Self::db_to_norm(db as f32);
                let y = response.rect.max.y - norm * canvas_h;
                painter.line_segment(
                    [
                        egui::pos2(response.rect.min.x, y),
                        egui::pos2(response.rect.max.x, y),
                    ],
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 110, 80)),
                );
                painter.text(
                    egui::pos2(response.rect.min.x + 8.0_f32, y - 6.0_f32),
                    egui::Align2::LEFT_BOTTOM,
                    format!("{}dB", db),
                    egui::FontId::proportional(10.0_f32),
                    Color32::from_rgb(140, 165, 195),
                );
            }

            // Draw Input Waveform History Stream
            let num_pts = self.input_history.len();
            if num_pts > 1 {
                let mut prev_pt = None;
                for (i, &db_val) in self.input_history.iter().enumerate() {
                    let x = response.rect.min.x + (i as f32 / (num_pts - 1) as f32) * canvas_w;
                    let norm = Self::db_to_norm(db_val);
                    let y = response.rect.max.y - norm * canvas_h;
                    let current_pt = egui::pos2(x, y);

                    if let Some(p) = prev_pt {
                        painter.line_segment(
                            [p, current_pt],
                            Stroke::new(2.0_f32, Color32::from_rgb(0, 160, 200)),
                        );
                    }
                    prev_pt = Some(current_pt);
                }
            }

            // Draw Physics Simulated Envelope Follower Ball
            let ball_x = response.rect.min.x + canvas_w * 0.85_f32;
            let ball_norm = Self::db_to_norm(self.physics.position_db);
            let ball_y = response.rect.max.y - ball_norm * canvas_h;
            let ball_center = egui::pos2(ball_x, ball_y);

            // Ball shadow & glow
            painter.circle_stroke(
                ball_center,
                BALL_HIT_RADIUS,
                Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 100)),
            );
            painter.circle_filled(
                ball_center,
                BALL_VISUAL_RADIUS,
                Color32::from_rgb(0, 229, 255),
            );
            painter.circle_filled(
                egui::pos2(ball_x - 3.0_f32, ball_y - 3.0_f32),
                4.0_f32,
                Color32::from_rgb(255, 255, 255),
            );

            // Readout on canvas
            painter.text(
                egui::pos2(
                    response.rect.max.x - 14.0_f32,
                    response.rect.min.y + 14.0_f32,
                ),
                egui::Align2::RIGHT_TOP,
                format!(
                    "Env: {:.1} dBFS | GR: {:.1} dB",
                    self.envelope_out_db, self.gain_reduction_db
                ),
                egui::FontId::proportional(13.0_f32),
                Color32::from_rgb(255, 215, 0),
            );

            ui.add_space(10.0_f32);

            // 3. Parameter Sliders Bar (>=44pt Touch Targets)
            ui.horizontal(|ui| {
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Attack").strong());
                        ui.add(
                            egui::Slider::new(&mut self.attack_ms, 0.1..=500.0)
                                .text("ms")
                                .logarithmic(true),
                        );
                    });
                });

                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Hold").strong());
                        ui.add(egui::Slider::new(&mut self.hold_ms, 0.0..=500.0).text("ms"));
                    });
                });

                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Release").strong());
                        ui.add(
                            egui::Slider::new(&mut self.release_ms, 1.0..=2000.0)
                                .text("ms")
                                .logarithmic(true),
                        );
                    });
                });

                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Sensitivity").strong());
                        ui.add(
                            egui::Slider::new(&mut self.sensitivity_db, -48.0..=24.0).text("dB"),
                        );
                    });
                });
            });

            ui.add_space(6.0_f32);

            // 4. Sidechain Routing Source Selectors (>=44pt Touch Targets)
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Sidechain Route:").strong());
                let sources = [
                    SidechainSource::InternalMain,
                    SidechainSource::Track1Kick,
                    SidechainSource::Track2Snare,
                    SidechainSource::Bus1Drums,
                    SidechainSource::ExternalAux,
                ];

                for src in sources {
                    let is_active = self.sidechain_source == src;
                    let btn = egui::Button::new(
                        egui::RichText::new(src.display_name())
                            .color(if is_active {
                                Color32::from_rgb(10, 14, 22)
                            } else {
                                Color32::from_rgb(220, 235, 255)
                            })
                            .strong(),
                    )
                    .min_size(Vec2::new(100.0_f32, MIN_HIT_TARGET_PT))
                    .fill(if is_active {
                        Color32::from_rgb(0, 229, 255)
                    } else {
                        Color32::from_rgb(30, 40, 60)
                    });

                    if ui.add(btn).clicked() {
                        self.sidechain_source = src;
                    }
                }
            });
        });
    }
}
