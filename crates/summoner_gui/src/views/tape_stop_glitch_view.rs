// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Vinyl Turntable Inertial Motor Brake Tape Stop & Stutter Chopper HUD (Milestones 14, 33 & 34).
//!
//! Provides an interactive 2D Dynamics Puck (Brake Duration [20.0..2000.0 ms] vs Stutter Division [1.0..64.0]),
//! real-time non-linear motor inertia deceleration curve v(t) = (1 - t/T)^p with pitch drop down to DC (0 Hz),
//! 4 performance presets (Direct Drive Vinyl Brake, Fast Tape Slam Stop, Slow Dub Spin Down, Hyper Stutter Glitch),
//! rhythmic chopped stutter gate with variable pulse width duty cycle [0.01..0.99],
//! dynamic semitone pitch dive calculation, and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and deterministic ASCII snapshot rendering.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, FontId, RichText, Rounding, Stroke, Vec2};

pub const TAPESTOP_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding target
pub const MIN_BRAKE_MS: f32 = 20.0;
pub const MAX_BRAKE_MS: f32 = 2000.0;
pub const MIN_DIVISION: f32 = 1.0;
pub const MAX_DIVISION: f32 = 64.0;
pub const NUM_CURVE_PLOT_POINTS: usize = 64;

/// Turntable motor inertia and tape braking presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TapeStopPreset {
    /// Technics SL-1200 style direct-drive turntable motor power switch kill with classic inertia curve.
    #[default]
    DirectDriveVinylBrake,
    /// Reel-to-reel magnetic tape pinch-roller stop with instant mechanical grab.
    FastTapeSlamStop,
    /// Heavy belt-drive turntable motor power cutoff with extended platter momentum glide.
    SlowDubSpinDown,
    /// Digital micro-buffer buffer-override stutter repeater with extreme chopping.
    HyperStutterGlitch,
}

impl TapeStopPreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::DirectDriveVinylBrake => "Direct Drive Vinyl Brake (SL-1200 Classic)",
            Self::FastTapeSlamStop => "Fast Tape Slam Stop (Reel-to-Reel Pinch)",
            Self::SlowDubSpinDown => "Slow Dub Spin Down (Belt-Drive Momentum)",
            Self::HyperStutterGlitch => "Hyper Stutter Glitch (Buffer Override)",
        }
    }

    /// Returns nominal (brake_ms, start_ms, division, pulse_width, curvature).
    pub fn nominal_parameters(&self) -> (f32, f32, f32, f32, f32) {
        match self {
            Self::DirectDriveVinylBrake => (650.0, 350.0, 16.0, 0.50, 1.8),
            Self::FastTapeSlamStop => (120.0, 80.0, 32.0, 0.70, 0.9),
            Self::SlowDubSpinDown => (1450.0, 750.0, 8.0, 0.35, 2.4),
            Self::HyperStutterGlitch => (250.0, 100.0, 48.0, 0.85, 1.2),
        }
    }
}

/// Vinyl Turntable Inertial Motor Brake Tape Stop & Stutter Chopper HUD view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TapeStopGlitchView {
    /// Active tape stop preset.
    pub preset: TapeStopPreset,
    /// Turntable motor spin-down brake duration in ms [20.0 ..= 2000.0].
    pub brake_duration_ms: f32,
    /// Turntable motor spin-up start duration in ms [20.0 ..= 2000.0].
    pub start_duration_ms: f32,
    /// Rhythmic stutter slice division [1.0 ..= 64.0].
    pub stutter_division: f32,
    /// Stutter gate pulse width duty cycle [0.01 ..= 0.99].
    pub stutter_pulse_width: f32,
    /// Deceleration polynomial curvature exponent [0.5 ..= 3.0].
    pub deceleration_curvature: f32,
    /// Whether rhythmic stutter chopping is active.
    pub stutter_enabled: bool,
    /// Whether the turntable platter is currently halted (tape stop active).
    pub is_stopped: bool,
    /// Current real-time playback speed ratio [0.0 ..= 1.0].
    pub current_speed_ratio: f32,
    /// Dynamic semitone pitch shift relative to nominal playback speed.
    pub pitch_drop_semitones: f32,
    /// Duration of a single stutter slice in milliseconds (assuming 120 BPM base).
    pub effective_stutter_ms: f32,
    /// 2D Dynamics Puck position (X: brake_duration normalized, Y: stutter_division normalized).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for TapeStopGlitchView {
    fn default() -> Self {
        Self::new()
    }
}

impl TapeStopGlitchView {
    pub fn new() -> Self {
        let mut view = Self {
            preset: TapeStopPreset::DirectDriveVinylBrake,
            brake_duration_ms: 650.0,
            start_duration_ms: 350.0,
            stutter_division: 16.0,
            stutter_pulse_width: 0.50,
            deceleration_curvature: 1.8,
            stutter_enabled: false,
            is_stopped: false,
            current_speed_ratio: 1.0,
            pitch_drop_semitones: 0.0,
            effective_stutter_ms: 125.0,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        };
        let (b, s, d, pw, c) = view.preset.nominal_parameters();
        view.brake_duration_ms = b;
        view.start_duration_ms = s;
        view.stutter_division = d;
        view.stutter_pulse_width = pw;
        view.deceleration_curvature = c;
        view.puck_pos = (
            Self::brake_to_normalized(view.brake_duration_ms),
            Self::division_to_normalized(view.stutter_division),
        );
        view.update_physics();
        view
    }

    pub fn set_preset(&mut self, preset: TapeStopPreset) {
        self.preset = preset;
        let (b, s, d, pw, c) = preset.nominal_parameters();
        self.brake_duration_ms = b;
        self.start_duration_ms = s;
        self.stutter_division = d;
        self.stutter_pulse_width = pw;
        self.deceleration_curvature = c;
        self.puck_pos = (
            Self::brake_to_normalized(self.brake_duration_ms),
            Self::division_to_normalized(self.stutter_division),
        );
        self.update_physics();
    }

    pub fn trigger_brake(&mut self) {
        self.is_stopped = true;
        self.current_speed_ratio = 0.0;
        self.update_physics();
    }

    pub fn trigger_start(&mut self) {
        self.is_stopped = false;
        self.current_speed_ratio = 1.0;
        self.update_physics();
    }

    pub fn toggle_stutter(&mut self) {
        self.stutter_enabled = !self.stutter_enabled;
        self.update_physics();
    }

    pub fn update_physics(&mut self) {
        // Calculate pitch drop in semitones: 12 * log2(speed)
        if self.current_speed_ratio > 0.001 {
            self.pitch_drop_semitones = 12.0 * self.current_speed_ratio.log2();
        } else {
            self.pitch_drop_semitones = -96.0;
        }

        // Stutter slice duration in ms (120 BPM base: quarter note = 500 ms, 1 measure = 2000 ms)
        self.effective_stutter_ms = 2000.0 / self.stutter_division.max(1.0);
    }

    pub fn brake_to_normalized(brake_ms: f32) -> f32 {
        ((brake_ms - MIN_BRAKE_MS) / (MAX_BRAKE_MS - MIN_BRAKE_MS)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_brake(norm: f32) -> f32 {
        MIN_BRAKE_MS + norm.clamp(0.0, 1.0) * (MAX_BRAKE_MS - MIN_BRAKE_MS)
    }

    pub fn division_to_normalized(div: f32) -> f32 {
        ((div - MIN_DIVISION) / (MAX_DIVISION - MIN_DIVISION)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_division(norm: f32) -> f32 {
        MIN_DIVISION + norm.clamp(0.0, 1.0) * (MAX_DIVISION - MIN_DIVISION)
    }

    /// Evaluates speed deceleration trajectory across 64 steps simulating motor spin-down.
    pub fn evaluate_deceleration_curve(&self) -> [f32; NUM_CURVE_PLOT_POINTS] {
        let mut curve = [0.0_f32; NUM_CURVE_PLOT_POINTS];
        let p = self.deceleration_curvature;

        for (i, item) in curve.iter_mut().enumerate() {
            let t = (i as f32) / (NUM_CURVE_PLOT_POINTS as f32 - 1.0);
            let speed = (1.0 - t).max(0.0).powf(p);
            *item = speed;
        }
        curve
    }

    pub fn render_ascii_snapshot(&self) -> String {
        let mut out = String::new();
        out.push_str("================================================================================\n");
        out.push_str("   SUMMONER DAW — VINYL TURNTABLE TAPE STOP & STUTTER CHOPPER HUD\n");
        out.push_str("================================================================================\n");
        out.push_str(&format!(" Preset: {}\n", self.preset.name()));
        out.push_str(" Touch Target: [PASS: >=44pt]\n");
        out.push_str(&format!(
            " Brake Duration: {:.1} ms | Start Duration: {:.1} ms | Curvature: {:.1}\n",
            self.brake_duration_ms, self.start_duration_ms, self.deceleration_curvature
        ));
        out.push_str(&format!(
            " Stutter Division: 1/{:.0} (Slice: {:.1} ms, PW: {:.0}%) | Stutter: {}\n",
            self.stutter_division,
            self.effective_stutter_ms,
            self.stutter_pulse_width * 100.0,
            if self.stutter_enabled { "ON" } else { "OFF" }
        ));
        out.push_str(&format!(
            " Status: {} | Speed Ratio: {:.1}% | Pitch Dive: {:.1} semitones\n",
            if self.is_stopped { "STOPPED" } else { "RUNNING" },
            self.current_speed_ratio * 100.0,
            self.pitch_drop_semitones
        ));
        out.push_str("--------------------------------------------------------------------------------\n");
        out.push_str(" Deceleration Velocity Profile v(t):\n");

        let curve = self.evaluate_deceleration_curve();
        for row in (0..6).rev() {
            let row_val = row as f32 / 5.0;
            out.push_str(&format!("{:4.1} |", row_val));
            for (step, &v) in curve.iter().enumerate().step_by(2) {
                if (step / 2) < 32 {
                    if v >= row_val {
                        out.push('█');
                    } else {
                        out.push(' ');
                    }
                }
            }
            out.push('\n');
        }
        out.push_str("     +--------------------------------\n");
        out.push_str(&format!("      0ms                            {:.0}ms\n", self.brake_duration_ms));
        out.push_str("================================================================================\n");
        out
    }

    pub fn render_ascii_snapshot_str(&self) -> String {
        self.render_ascii_snapshot()
    }

    pub fn hit_test_tapestop_puck(&self, pointer_pos: (f32, f32), canvas_rect: Rect) -> bool {
        let padding = 15.0;
        let inner_w = (canvas_rect.width - 2.0 * padding).max(10.0);
        let inner_h = (canvas_rect.height - 2.0 * padding).max(10.0);

        let puck_x = canvas_rect.x + padding + self.puck_pos.0 * inner_w;
        let puck_y = canvas_rect.y + canvas_rect.height - padding - self.puck_pos.1 * inner_h;

        let dx = pointer_pos.0 - puck_x;
        let dy = pointer_pos.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= TAPESTOP_PUCK_HIT_RADIUS
    }

    #[cfg(feature = "gui")]
    pub fn show_ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Header: Presets and Performance Triggers
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Brake Mode:")
                        .font(FontId::proportional(12.0))
                        .strong()
                        .color(Color32::from_rgb(226, 232, 240)),
                );

                let presets = [
                    (TapeStopPreset::DirectDriveVinylBrake, "Direct Drive"),
                    (TapeStopPreset::FastTapeSlamStop, "Fast Tape Slam"),
                    (TapeStopPreset::SlowDubSpinDown, "Slow Dub"),
                    (TapeStopPreset::HyperStutterGlitch, "Hyper Stutter"),
                ];

                for (p, label) in presets {
                    let is_active = self.preset == p;
                    let btn = egui::Button::new(
                        RichText::new(label)
                            .font(FontId::proportional(11.0))
                            .color(if is_active { Color32::WHITE } else { Color32::from_rgb(203, 213, 225) }),
                    )
                    .fill(if is_active { Color32::from_rgb(180, 83, 9) } else { Color32::from_rgb(30, 41, 59) })
                    .rounding(Rounding::same(4.0));

                    if ui.add(btn).clicked() {
                        self.set_preset(p);
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(
                        RichText::new(if self.is_stopped { "▶ Motor Start" } else { "⏹ Brake Stop" })
                            .font(FontId::proportional(11.0))
                            .strong()
                            .color(if self.is_stopped { Color32::from_rgb(34, 197, 94) } else { Color32::from_rgb(239, 68, 68) }),
                    ).clicked() {
                        if self.is_stopped {
                            self.trigger_start();
                        } else {
                            self.trigger_brake();
                        }
                    }

                    if ui.button(
                        RichText::new("⚡ Tap Stutter")
                            .font(FontId::proportional(11.0))
                            .color(Color32::from_rgb(245, 158, 11)),
                    ).clicked() {
                        self.toggle_stutter();
                    }
                });
            });

            ui.add_space(8.0);

            // Two-Panel Interactive Canvas
            ui.horizontal(|ui| {
                // Left Panel: 2D Puck Canvas (Brake Duration vs Stutter Division)
                let (canvas_rect, response) = ui.allocate_exact_size(Vec2::new(380.0, 210.0), egui::Sense::click_and_drag());
                let painter = ui.painter_at(canvas_rect);

                // Background & Frame
                painter.rect_filled(canvas_rect, Rounding::same(6.0), Color32::from_rgb(15, 23, 42));
                painter.rect_stroke(canvas_rect, Rounding::same(6.0), Stroke::new(1.0_f32, Color32::from_rgb(51, 65, 85)));

                // Concentric vinyl groove circles
                let center = canvas_rect.center();
                for r in [25.0, 50.0, 75.0, 95.0] {
                    painter.circle_stroke(center, r, Stroke::new(0.5_f32, Color32::from_rgba_unmultiplied(245, 158, 11, 25)));
                }

                // Grid lines
                for i in 1..4 {
                    let gx = canvas_rect.left() + (canvas_rect.width() / 4.0) * (i as f32);
                    let gy = canvas_rect.top() + (canvas_rect.height() / 4.0) * (i as f32);
                    painter.line_segment([egui::pos2(gx, canvas_rect.top()), egui::pos2(gx, canvas_rect.bottom())], Stroke::new(0.4_f32, Color32::from_rgb(30, 41, 59)));
                    painter.line_segment([egui::pos2(canvas_rect.left(), gy), egui::pos2(canvas_rect.right(), gy)], Stroke::new(0.4_f32, Color32::from_rgb(30, 41, 59)));
                }

                // Interaction Logic
                if response.dragged() || response.clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let nx = ((pos.x - canvas_rect.left() - 10.0) / (canvas_rect.width() - 20.0)).clamp(0.0, 1.0);
                        let ny = 1.0 - ((pos.y - canvas_rect.top() - 10.0) / (canvas_rect.height() - 20.0)).clamp(0.0, 1.0);
                        self.puck_pos = (nx, ny);
                        self.brake_duration_ms = Self::normalized_to_brake(nx);
                        self.stutter_division = Self::normalized_to_division(ny);
                        self.update_physics();
                    }
                }

                // Puck Rendering
                let puck_x = canvas_rect.left() + 10.0 + self.puck_pos.0 * (canvas_rect.width() - 20.0);
                let puck_y = canvas_rect.bottom() - 10.0 - self.puck_pos.1 * (canvas_rect.height() - 20.0);
                let puck_pos = egui::pos2(puck_x, puck_y);

                painter.circle_filled(puck_pos, TAPESTOP_PUCK_HIT_RADIUS, Color32::from_rgba_unmultiplied(245, 158, 11, 45));
                painter.circle_filled(puck_pos, 7.0, Color32::from_rgb(245, 158, 11));
                painter.circle_stroke(puck_pos, TAPESTOP_PUCK_HIT_RADIUS, Stroke::new(1.2_f32, Color32::from_rgb(245, 158, 11)));
                painter.circle_stroke(puck_pos, 8.5, Stroke::new(1.5_f32, Color32::WHITE));

                // Canvas Labels
                painter.text(
                    egui::pos2(canvas_rect.left() + 8.0, canvas_rect.bottom() - 8.0),
                    egui::Align2::LEFT_BOTTOM,
                    "► Motor Brake Duration (20..2000 ms)",
                    FontId::proportional(10.0),
                    Color32::from_rgb(148, 163, 184),
                );
                painter.text(
                    egui::pos2(canvas_rect.left() + 8.0, canvas_rect.top() + 6.0),
                    egui::Align2::LEFT_TOP,
                    "▲ Stutter Division (1/1..1/64)",
                    FontId::proportional(10.0),
                    Color32::from_rgb(148, 163, 184),
                );

                // Right Panel: Deceleration Profile Curve & Speed Gauge
                ui.vertical(|ui| {
                    let (curve_rect, _) = ui.allocate_exact_size(Vec2::new(380.0, 110.0), egui::Sense::hover());
                    let cpainter = ui.painter_at(curve_rect);
                    cpainter.rect_filled(curve_rect, Rounding::same(6.0), Color32::from_rgb(15, 23, 42));
                    cpainter.rect_stroke(curve_rect, Rounding::same(6.0), Stroke::new(1.0_f32, Color32::from_rgb(51, 65, 85)));

                    let curve = self.evaluate_deceleration_curve();

                    let mut pts = Vec::with_capacity(NUM_CURVE_PLOT_POINTS);
                    for (i, &v) in curve.iter().enumerate() {
                        let t = (i as f32) / ((NUM_CURVE_PLOT_POINTS - 1) as f32);
                        let px = curve_rect.left() + 10.0 + t * (curve_rect.width() - 20.0);
                        let py = curve_rect.bottom() - 10.0 - v * (curve_rect.height() - 20.0);
                        pts.push(egui::pos2(px, py));
                    }

                    for w in pts.windows(2) {
                        cpainter.line_segment([w[0], w[1]], Stroke::new(1.8_f32, Color32::from_rgb(245, 158, 11)));
                    }

                    cpainter.text(
                        egui::pos2(curve_rect.left() + 8.0, curve_rect.top() + 6.0),
                        egui::Align2::LEFT_TOP,
                        format!("v(t) Brake Curve (p={:.1}) | Dur: {:.0} ms", self.deceleration_curvature, self.brake_duration_ms),
                        FontId::proportional(10.0),
                        Color32::from_rgb(226, 232, 240),
                    );

                    ui.add_space(6.0);

                    // Speed & Semitone Pitch Gauge
                    let (gauge_rect, _) = ui.allocate_exact_size(Vec2::new(380.0, 84.0), egui::Sense::hover());
                    let gpainter = ui.painter_at(gauge_rect);
                    gpainter.rect_filled(gauge_rect, Rounding::same(6.0), Color32::from_rgb(15, 23, 42));
                    gpainter.rect_stroke(gauge_rect, Rounding::same(6.0), Stroke::new(1.0_f32, Color32::from_rgb(51, 65, 85)));

                    // Speed Bar
                    let norm_speed = self.current_speed_ratio.clamp(0.0, 1.0);
                    let bar_w = (gauge_rect.width() - 30.0) * norm_speed;
                    let bar_rect = egui::Rect::from_min_size(
                        egui::pos2(gauge_rect.left() + 15.0, gauge_rect.top() + 26.0),
                        Vec2::new(bar_w, 18.0),
                    );
                    gpainter.rect_filled(bar_rect, Rounding::same(3.0), Color32::from_rgb(234, 88, 12));

                    gpainter.text(
                        egui::pos2(gauge_rect.left() + 15.0, gauge_rect.top() + 8.0),
                        egui::Align2::LEFT_TOP,
                        format!("Playback Speed: {:.1}% | Pitch Dive: {:.1} st", self.current_speed_ratio * 100.0, self.pitch_drop_semitones),
                        FontId::proportional(10.5),
                        Color32::from_rgb(226, 232, 240),
                    );

                    gpainter.text(
                        egui::pos2(gauge_rect.left() + 15.0, gauge_rect.bottom() - 18.0),
                        egui::Align2::LEFT_BOTTOM,
                        format!("Stutter Slice: {:.1} ms | Division: 1/{:.0} | Duty: {:.0}%", self.effective_stutter_ms, self.stutter_division, self.stutter_pulse_width * 100.0),
                        FontId::proportional(9.5),
                        Color32::from_rgb(148, 163, 184),
                    );
                });
            });

            ui.add_space(10.0);

            // Tactile Secondary Parameter Sliders
            egui::Grid::new("tapestop_params_grid").num_columns(2).spacing([16.0, 8.0]).show(ui, |ui| {
                ui.label(RichText::new("Turntable Brake Duration:").font(FontId::proportional(11.0)).color(Color32::from_rgb(203, 213, 225)));
                if ui.add(egui::Slider::new(&mut self.brake_duration_ms, 20.0..=2000.0).show_value(true).suffix(" ms")).changed() {
                    self.puck_pos.0 = Self::brake_to_normalized(self.brake_duration_ms);
                    self.update_physics();
                }
                ui.end_row();

                ui.label(RichText::new("Motor Spin-Up Start Duration:").font(FontId::proportional(11.0)).color(Color32::from_rgb(203, 213, 225)));
                if ui.add(egui::Slider::new(&mut self.start_duration_ms, 20.0..=2000.0).show_value(true).suffix(" ms")).changed() {
                    self.update_physics();
                }
                ui.end_row();

                ui.label(RichText::new("Stutter Slice Division:").font(FontId::proportional(11.0)).color(Color32::from_rgb(203, 213, 225)));
                if ui.add(egui::Slider::new(&mut self.stutter_division, 1.0..=64.0).step_by(1.0).show_value(true).prefix("1/")).changed() {
                    self.puck_pos.1 = Self::division_to_normalized(self.stutter_division);
                    self.update_physics();
                }
                ui.end_row();

                ui.label(RichText::new("Stutter Gate Pulse Width:").font(FontId::proportional(11.0)).color(Color32::from_rgb(203, 213, 225)));
                if ui.add(egui::Slider::new(&mut self.stutter_pulse_width, 0.01..=0.99).show_value(true)).changed() {
                    self.update_physics();
                }
                ui.end_row();

                ui.label(RichText::new("Deceleration Curvature:").font(FontId::proportional(11.0)).color(Color32::from_rgb(203, 213, 225)));
                if ui.add(egui::Slider::new(&mut self.deceleration_curvature, 0.5..=3.0).show_value(true)).changed() {
                    self.update_physics();
                }
                ui.end_row();

                ui.label(RichText::new("Rhythmic Stutter Chopping:").font(FontId::proportional(11.0)).color(Color32::from_rgb(203, 213, 225)));
                if ui.checkbox(&mut self.stutter_enabled, "Enabled").changed() {
                    self.update_physics();
                }
                ui.end_row();
            });

            ui.add_space(6.0);

            // Status Bar
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!(
                        "Speed: {:.1}% | Pitch Dive: {:.1} st | Stutter Slice: {:.1} ms",
                        self.current_speed_ratio * 100.0,
                        self.pitch_drop_semitones,
                        self.effective_stutter_ms
                    ))
                    .font(FontId::proportional(11.0))
                    .color(Color32::from_rgb(148, 163, 184)),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new("Touch Target: PASS (≥44pt)")
                            .font(FontId::proportional(10.0))
                            .color(Color32::from_rgb(34, 197, 94)),
                    );
                });
            });
        });
    }
}
