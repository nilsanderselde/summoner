// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Tactile Audio Tape Cassette Emulator & Magnetic Saturation Hysteresis View (Step 1405).

use crate::layout_math::Rect;
use crate::touch_controls::{ContrastColorPalette, MIN_HIT_TARGET_PT};

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const TAPE_DRIVE_HANDLE_HIT_RADIUS: f32 = 22.0; // 44x44pt touch area
pub const TAPE_SPOOL_RADIUS_PT: f32 = 45.0;

/// Tape transport speed in Inches Per Second (IPS).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TapeSpeedIps {
    Ips3_75, // 3.75 IPS - Lo-Fi Cassette
    Ips7_5,  // 7.5 IPS - Vintage Reel-to-Reel
    Ips15,   // 15 IPS - Standard Studio Master
    Ips30,   // 30 IPS - High-Fidelity Audiophile Master
}

impl TapeSpeedIps {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Ips3_75 => "3.75 IPS (Cassette)",
            Self::Ips7_5 => "7.5 IPS (Vintage Reel)",
            Self::Ips15 => "15 IPS (Studio)",
            Self::Ips30 => "30 IPS (Master)",
        }
    }

    pub fn frequency_cutoff_hz(&self) -> f32 {
        match self {
            Self::Ips3_75 => 12000.0,
            Self::Ips7_5 => 16000.0,
            Self::Ips15 => 20000.0,
            Self::Ips30 => 24000.0,
        }
    }
}

/// Magnetic tape formulation characteristics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TapeFormulation {
    TypeINormal,         // Standard Ferric Oxide
    TypeIIChrome,        // Chromium Dioxide (High Bias)
    TypeIVMetal,         // Pure Metal Particle
    Master900HighOutput, // Ultra High Headroom Analog Studio Master Tape
}

impl TapeFormulation {
    pub fn name(&self) -> &'static str {
        match self {
            Self::TypeINormal => "Type I (Normal Ferric)",
            Self::TypeIIChrome => "Type II (CrO2 Chrome)",
            Self::TypeIVMetal => "Type IV (Metal Particle)",
            Self::Master900HighOutput => "Master 900 (High Output)",
        }
    }

    pub fn max_output_level_db(&self) -> f32 {
        match self {
            Self::TypeINormal => 3.0,
            Self::TypeIIChrome => 5.5,
            Self::TypeIVMetal => 8.0,
            Self::Master900HighOutput => 11.0,
        }
    }
}

/// Tactile Audio Tape Emulator View (Step 1405).
#[derive(Debug, Clone)]
pub struct TapeEmulatorView {
    pub speed_ips: TapeSpeedIps,
    pub formulation: TapeFormulation,
    pub input_drive_db: f32,         // -12.0 ..= +24.0 dB
    pub bias_trim_db: f32,           // -6.0 ..= +6.0 dB
    pub saturation_hardness: f32,    // 0.0 ..= 1.0
    pub wow_flutter_rate_hz: f32,    // 0.1 ..= 10.0 Hz
    pub wow_flutter_depth_pct: f32,  // 0.0 ..= 100.0%
    pub tape_hiss_level_db: f32,     // -96.0 ..= -30.0 dB
    pub output_level_db: f32,        // -24.0 ..= +12.0 dB
    pub spool_rotation_rad: f32,     // Visual rotation angle
    pub tape_tension_left_pct: f32,  // 0.0 ..= 100.0%
    pub tape_tension_right_pct: f32, // 0.0 ..= 100.0%
    pub is_tape_running: bool,
    pub color_palette: ContrastColorPalette,
}

impl Default for TapeEmulatorView {
    fn default() -> Self {
        Self::new()
    }
}

impl TapeEmulatorView {
    pub fn new() -> Self {
        Self {
            speed_ips: TapeSpeedIps::Ips15,
            formulation: TapeFormulation::Master900HighOutput,
            input_drive_db: 6.0,
            bias_trim_db: 0.0,
            saturation_hardness: 0.50,
            wow_flutter_rate_hz: 1.2,
            wow_flutter_depth_pct: 15.0,
            tape_hiss_level_db: -72.0,
            output_level_db: 0.0,
            spool_rotation_rad: 0.0,
            tape_tension_left_pct: 65.0,
            tape_tension_right_pct: 35.0,
            is_tape_running: true,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Calculate magnetic tape saturation non-linear transfer function (tanh with hysteresis shape).
    pub fn evaluate_saturation(&self, x: f32) -> f32 {
        let drive = 10.0_f32.powf(self.input_drive_db / 20.0);
        let driven_x = x * drive;
        let hardness = self.saturation_hardness;
        let saturated = (driven_x / (1.0 + driven_x.abs().powf(1.0 + hardness))).tanh();
        let max_out = 10.0_f32.powf(self.formulation.max_output_level_db() / 20.0);
        (saturated * max_out).clamp(-2.0, 2.0)
    }

    /// Advance spool rotation physics animation.
    pub fn step_physics(&mut self, delta_seconds: f32) {
        if !self.is_tape_running {
            return;
        }
        let speed_factor = match self.speed_ips {
            TapeSpeedIps::Ips3_75 => 1.5,
            TapeSpeedIps::Ips7_5 => 3.0,
            TapeSpeedIps::Ips15 => 6.0,
            TapeSpeedIps::Ips30 => 12.0,
        };
        self.spool_rotation_rad = (self.spool_rotation_rad + delta_seconds * speed_factor)
            .rem_euclid(std::f32::consts::TAU);
    }

    /// Hit-test input drive puck on saturation curve (>=44x44pt).
    pub fn hit_test_drive_handle(&self, pos: (f32, f32), canvas: Rect) -> bool {
        let norm_drive = ((self.input_drive_db + 12.0) / 36.0).clamp(0.0, 1.0);
        let hx = canvas.x + norm_drive * canvas.width;
        let hy = canvas.y + canvas.height * 0.5;
        let dist = ((pos.0 - hx).powi(2) + (pos.1 - hy).powi(2)).sqrt();
        dist <= TAPE_DRIVE_HANDLE_HIT_RADIUS
    }

    /// Deterministic ASCII render of the cassette tape mechanism.
    pub fn render_ascii(&self, width: usize) -> String {
        let mut buf = vec!['-'; width];
        if width >= 10 {
            buf[1] = '(';
            buf[2] = 'O';
            buf[3] = ')';
            buf[width - 4] = '(';
            buf[width - 3] = 'O';
            buf[width - 2] = ')';
            let mid = width / 2;
            buf[mid - 1] = '[';
            buf[mid] = 'H';
            buf[mid + 1] = ']';
        }
        buf.into_iter().collect()
    }
}

#[cfg(feature = "gui")]
impl TapeEmulatorView {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // 1. Header Toolbar
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("VINTAGE ANALOG TAPE CASSETTE EMULATOR")
                        .color(Color32::from_rgb(240, 245, 255))
                        .strong(),
                );
                ui.separator();
                ui.label(
                    egui::RichText::new(format!(
                        "Speed: {} | Drive: {:+.1} dB | Hiss: {:.0} dB",
                        self.speed_ips.name(),
                        self.input_drive_db,
                        self.tape_hiss_level_db
                    ))
                    .color(Color32::from_rgb(0, 229, 255))
                    .strong(),
                );
            });

            ui.add_space(6.0);

            // Tape Speed Selectors (>=44pt Touch Targets)
            ui.horizontal(|ui| {
                let speeds = [
                    (TapeSpeedIps::Ips3_75, "3.75 IPS"),
                    (TapeSpeedIps::Ips7_5, "7.5 IPS"),
                    (TapeSpeedIps::Ips15, "15 IPS"),
                    (TapeSpeedIps::Ips30, "30 IPS"),
                ];

                for (spd, lbl) in speeds {
                    let is_act = self.speed_ips == spd;
                    let btn = egui::Button::new(
                        egui::RichText::new(lbl)
                            .color(if is_act {
                                Color32::from_rgb(10, 14, 22)
                            } else {
                                Color32::from_rgb(220, 235, 255)
                            })
                            .strong(),
                    )
                    .min_size(Vec2::new(90.0, MIN_HIT_TARGET_PT))
                    .fill(if is_act {
                        Color32::from_rgb(0, 229, 255)
                    } else {
                        Color32::from_rgb(30, 40, 60)
                    });

                    if ui.add(btn).clicked() {
                        self.speed_ips = spd;
                    }
                }

                ui.separator();

                let forms = [
                    (TapeFormulation::TypeINormal, "Type I"),
                    (TapeFormulation::TypeIIChrome, "Type II"),
                    (TapeFormulation::TypeIVMetal, "Type IV"),
                    (TapeFormulation::Master900HighOutput, "Master 900"),
                ];

                for (form, flbl) in forms {
                    let is_act = self.formulation == form;
                    let btn = egui::Button::new(
                        egui::RichText::new(flbl)
                            .color(if is_act {
                                Color32::from_rgb(10, 14, 22)
                            } else {
                                Color32::from_rgb(220, 235, 255)
                            })
                            .strong(),
                    )
                    .min_size(Vec2::new(90.0, MIN_HIT_TARGET_PT))
                    .fill(if is_act {
                        Color32::from_rgb(255, 215, 0)
                    } else {
                        Color32::from_rgb(30, 40, 60)
                    });

                    if ui.add(btn).clicked() {
                        self.formulation = form;
                    }
                }
            });

            ui.add_space(8.0);

            // 2. Dual Panel: Cassette Spool Mechanism (Left) + Hysteresis Curve (Right)
            ui.horizontal(|ui| {
                // Left Panel: Rotating Spools Cassette HUD
                let spool_size = Vec2::new(340.0, 220.0);
                let (spool_resp, painter) = ui.allocate_painter(spool_size, egui::Sense::hover());

                // Cassette Body
                painter.rect_filled(spool_resp.rect, 8.0_f32, Color32::from_rgb(16, 22, 34));
                painter.rect_stroke(
                    spool_resp.rect,
                    8.0_f32,
                    Stroke::new(2.0_f32, Color32::from_rgb(50, 68, 95)),
                );

                let left_center =
                    egui::pos2(spool_resp.rect.min.x + 85.0, spool_resp.rect.min.y + 110.0);
                let right_center =
                    egui::pos2(spool_resp.rect.min.x + 255.0, spool_resp.rect.min.y + 110.0);

                // Left Supply Spool
                painter.circle_filled(
                    left_center,
                    TAPE_SPOOL_RADIUS_PT,
                    Color32::from_rgb(30, 40, 60),
                );
                painter.circle_stroke(
                    left_center,
                    TAPE_SPOOL_RADIUS_PT,
                    Stroke::new(2.0_f32, Color32::from_rgb(0, 229, 255)),
                );
                // Right Take-up Spool
                painter.circle_filled(
                    right_center,
                    TAPE_SPOOL_RADIUS_PT,
                    Color32::from_rgb(30, 40, 60),
                );
                painter.circle_stroke(
                    right_center,
                    TAPE_SPOOL_RADIUS_PT,
                    Stroke::new(2.0_f32, Color32::from_rgb(0, 229, 255)),
                );

                // Spool Spokes (with rotation)
                for i in 0..3 {
                    let a = self.spool_rotation_rad + i as f32 * (std::f32::consts::TAU / 3.0);
                    let lx = left_center.x + a.cos() * 32.0;
                    let ly = left_center.y + a.sin() * 32.0;
                    painter.line_segment(
                        [left_center, egui::pos2(lx, ly)],
                        Stroke::new(2.0_f32, Color32::from_rgb(200, 220, 250)),
                    );

                    let rx = right_center.x + a.cos() * 32.0;
                    let ry = right_center.y + a.sin() * 32.0;
                    painter.line_segment(
                        [right_center, egui::pos2(rx, ry)],
                        Stroke::new(2.0_f32, Color32::from_rgb(200, 220, 250)),
                    );
                }

                // Tape Path across Heads
                let head_pos =
                    egui::pos2(spool_resp.rect.min.x + 170.0, spool_resp.rect.min.y + 180.0);
                painter.rect_filled(
                    egui::Rect::from_center_size(head_pos, Vec2::new(40.0, 20.0)),
                    3.0_f32,
                    Color32::from_rgb(255, 215, 0),
                );
                painter.text(
                    head_pos,
                    egui::Align2::CENTER_CENTER,
                    "HEAD",
                    egui::FontId::proportional(9.0),
                    Color32::from_rgb(10, 14, 22),
                );

                painter.line_segment(
                    [left_center, head_pos],
                    Stroke::new(3.0_f32, Color32::from_rgb(140, 90, 60)),
                );
                painter.line_segment(
                    [head_pos, right_center],
                    Stroke::new(3.0_f32, Color32::from_rgb(140, 90, 60)),
                );

                ui.add_space(10.0);

                // Right Panel: Hysteresis Transfer Curve
                let curve_size = Vec2::new(340.0, 220.0);
                let (curve_resp, curve_painter) =
                    ui.allocate_painter(curve_size, egui::Sense::hover());

                curve_painter.rect_filled(curve_resp.rect, 8.0_f32, Color32::from_rgb(10, 14, 22));
                curve_painter.rect_stroke(
                    curve_resp.rect,
                    8.0_f32,
                    Stroke::new(1.5_f32, Color32::from_rgb(45, 60, 85)),
                );

                curve_painter.text(
                    egui::pos2(curve_resp.rect.min.x + 12.0, curve_resp.rect.min.y + 12.0),
                    egui::Align2::LEFT_TOP,
                    "MAGNETIC SATURATION HYSTERESIS (B-H) TRANSFER",
                    egui::FontId::proportional(10.0),
                    Color32::from_rgb(255, 215, 0),
                );

                // Draw Hysteresis S-Curve
                let center_x = curve_resp.rect.min.x + curve_size.x * 0.5;
                let center_y = curve_resp.rect.min.y + curve_size.y * 0.5;
                let mut prev_pt: Option<egui::Pos2> = None;

                for i in 0..60 {
                    let norm_x = (i as f32 / 59.0) * 2.0 - 1.0;
                    let sat_y = self.evaluate_saturation(norm_x);
                    let px = center_x + norm_x * 140.0;
                    let py = center_y - sat_y * 70.0;
                    let pt = egui::pos2(px, py);
                    if let Some(prev) = prev_pt {
                        curve_painter.line_segment(
                            [prev, pt],
                            Stroke::new(2.5_f32, Color32::from_rgb(255, 107, 43)),
                        );
                    }
                    prev_pt = Some(pt);
                }
            });

            ui.add_space(8.0);

            // 3. Physical Parameters Sliders (>=44pt Touch Targets)
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Input Drive").strong());
                    ui.add(egui::Slider::new(&mut self.input_drive_db, -12.0..=24.0).text("dB"));
                });

                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Bias Trim").strong());
                    ui.add(egui::Slider::new(&mut self.bias_trim_db, -6.0..=6.0).text("dB"));
                });

                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Wow / Flutter").strong());
                    ui.add(
                        egui::Slider::new(&mut self.wow_flutter_depth_pct, 0.0..=100.0).text("%"),
                    );
                });

                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Tape Hiss").strong());
                    ui.add(
                        egui::Slider::new(&mut self.tape_hiss_level_db, -96.0..=-30.0).text("dB"),
                    );
                });
            });
        });
    }
}
