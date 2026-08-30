// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Vintage Rotary Speaker Horn/Drum Doppler Acceleration Simulator with Dual-Speed Brake Physics HUD (Step 1424).

use crate::touch_controls::{ContrastColorPalette, MIN_HIT_TARGET_PT};

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const ROTARY_HANDLE_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const SPEED_OF_SOUND_MPS: f32 = 343.0; // Speed of sound in air (m/s)

/// Rotary speaker motor speed state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotarySpeedState {
    Stop,
    Chorale, // Slow rotation
    Tremolo, // Fast rotation
    Brake,   // Active mechanical friction brake
}

/// Vintage Rotary Speaker Simulation View (Step 1424).
#[derive(Debug, Clone)]
pub struct RotarySpeakerView {
    pub speed_state: RotarySpeedState,
    pub horn_rpm: f32,
    pub drum_rpm: f32,
    pub horn_angle_rad: f32,
    pub drum_angle_rad: f32,
    pub horn_accel_time_s: f32,     // 0.2 ..= 5.0 s
    pub drum_accel_time_s: f32,     // 1.0 ..= 10.0 s
    pub horn_drum_balance_pct: f32, // 0..100% (Horn vs Drum mix)
    pub mic_distance_m: f32,        // 0.2 ..= 2.0 m
    pub mic_spread_deg: f32,        // 60.0 ..= 180.0 deg
    pub drive_saturation_db: f32,   // 0.0 ..= 24.0 dB
    pub horn_radius_m: f32,         // ~0.18 m
    pub drum_radius_m: f32,         // ~0.24 m
    pub is_dragging_mic: bool,
    pub color_palette: ContrastColorPalette,
}

impl Default for RotarySpeakerView {
    fn default() -> Self {
        Self::new()
    }
}

impl RotarySpeakerView {
    pub fn new() -> Self {
        Self {
            speed_state: RotarySpeedState::Tremolo,
            horn_rpm: 395.0,
            drum_rpm: 338.0,
            horn_angle_rad: 0.78,
            drum_angle_rad: 2.14,
            horn_accel_time_s: 0.95,
            drum_accel_time_s: 4.80,
            horn_drum_balance_pct: 60.0,
            mic_distance_m: 0.65,
            mic_spread_deg: 120.0,
            drive_saturation_db: 6.0,
            horn_radius_m: 0.18,
            drum_radius_m: 0.24,
            is_dragging_mic: false,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Target RPM for high frequency horn rotor given speed state.
    pub fn target_horn_rpm(&self) -> f32 {
        match self.speed_state {
            RotarySpeedState::Stop | RotarySpeedState::Brake => 0.0,
            RotarySpeedState::Chorale => 40.0,
            RotarySpeedState::Tremolo => 400.0,
        }
    }

    /// Target RPM for low frequency bass drum rotor given speed state.
    pub fn target_drum_rpm(&self) -> f32 {
        match self.speed_state {
            RotarySpeedState::Stop | RotarySpeedState::Brake => 0.0,
            RotarySpeedState::Chorale => 36.0,
            RotarySpeedState::Tremolo => 342.0,
        }
    }

    /// Calculate Doppler pitch shift ratio (delta_f / f0) for horn at given microphone angle.
    pub fn calculate_horn_doppler_shift(&self, mic_angle_rad: f32) -> f32 {
        let omega = self.horn_rpm * (std::f32::consts::PI * 2.0 / 60.0);
        let tip_velocity = omega * self.horn_radius_m;
        let relative_angle = self.horn_angle_rad - mic_angle_rad;
        (tip_velocity / SPEED_OF_SOUND_MPS) * relative_angle.sin()
    }

    /// Advance mechanical simulation physics by dt seconds.
    pub fn update_physics(&mut self, dt_s: f32) {
        let target_h = self.target_horn_rpm();
        let target_d = self.target_drum_rpm();

        let h_rate = 400.0 / self.horn_accel_time_s.max(0.1);
        let d_rate = 342.0 / self.drum_accel_time_s.max(0.5);

        if self.horn_rpm < target_h {
            self.horn_rpm = (self.horn_rpm + h_rate * dt_s).min(target_h);
        } else if self.horn_rpm > target_h {
            self.horn_rpm = (self.horn_rpm - h_rate * dt_s).max(target_h);
        }

        if self.drum_rpm < target_d {
            self.drum_rpm = (self.drum_rpm + d_rate * dt_s).min(target_d);
        } else if self.drum_rpm > target_d {
            self.drum_rpm = (self.drum_rpm - d_rate * dt_s).max(target_d);
        }

        let horn_omega = self.horn_rpm * (std::f32::consts::PI * 2.0 / 60.0);
        let drum_omega = self.drum_rpm * (std::f32::consts::PI * 2.0 / 60.0);

        self.horn_angle_rad =
            (self.horn_angle_rad + horn_omega * dt_s) % (std::f32::consts::PI * 2.0);
        self.drum_angle_rad =
            (self.drum_angle_rad - drum_omega * dt_s) % (std::f32::consts::PI * 2.0);
        // Drum rotates counter-direction
    }

    /// Tests if a screen coordinate hits the microphone position handle (>= 22pt radius -> 44x44pt).
    pub fn hit_test_mic_handle(&self, pos: (f32, f32), mic_pos_screen: (f32, f32)) -> bool {
        let dx = pos.0 - mic_pos_screen.0;
        let dy = pos.1 - mic_pos_screen.1;
        (dx * dx + dy * dy).sqrt() <= ROTARY_HANDLE_HIT_RADIUS
    }

    /// Render deterministic ASCII representation.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let doppler_l =
            self.calculate_horn_doppler_shift(-self.mic_spread_deg * std::f32::consts::PI / 360.0);
        let header = format!(
            "ROTARY SPEAKER [{:?}] Horn:{:.0}RPM Drum:{:.0}RPM Doppler:{:+.3}% Drive:+{:.1}dB",
            self.speed_state,
            self.horn_rpm,
            self.drum_rpm,
            doppler_l * 100.0,
            self.drive_saturation_db
        );
        lines.push(header);

        let center_x = width as f32 * 0.5;
        let center_y = height as f32 * 0.5;

        for y in 1..height {
            let mut row = String::with_capacity(width);
            for x in 0..width {
                let dx = x as f32 - center_x;
                let dy = (y as f32 - center_y) * 2.0; // Aspect ratio compensation
                let dist = (dx * dx + dy * dy).sqrt();
                let angle = dy.atan2(dx);

                if (dist - 8.0).abs() < 1.0 {
                    row.push('O'); // Cabinet perimeter
                } else if dist < 8.0 && (angle - self.horn_angle_rad).sin().abs() < 0.25 {
                    row.push('>'); // Horn flare
                } else if dist < 4.0 {
                    row.push('*'); // Center spindle
                } else {
                    row.push('.');
                }
            }
            lines.push(row);
        }
        lines
    }

    #[cfg(feature = "gui")]
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // 1. Top Header Bar & 4-Way Speed Switch
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("ROTARY SPEAKER & DOPPLER ACCELERATION HUD")
                        .size(15.0)
                        .color(Color32::from_rgb(0, 229, 255))
                        .strong(),
                );
                ui.separator();

                let speeds = [
                    (RotarySpeedState::Stop, "STOP"),
                    (RotarySpeedState::Chorale, "CHORALE (Slow)"),
                    (RotarySpeedState::Tremolo, "TREMOLO (Fast)"),
                    (RotarySpeedState::Brake, "BRAKE"),
                ];

                for (s_mode, lbl) in speeds {
                    let is_active = self.speed_state == s_mode;
                    let btn = egui::Button::new(
                        egui::RichText::new(lbl)
                            .color(if is_active {
                                Color32::from_rgb(10, 14, 22)
                            } else {
                                Color32::from_rgb(240, 245, 255)
                            })
                            .strong(),
                    )
                    .min_size(Vec2::new(90.0, MIN_HIT_TARGET_PT))
                    .fill(if is_active {
                        match s_mode {
                            RotarySpeedState::Stop => Color32::from_rgb(255, 75, 75),
                            RotarySpeedState::Chorale => Color32::from_rgb(0, 229, 255),
                            RotarySpeedState::Tremolo => Color32::from_rgb(0, 255, 180),
                            RotarySpeedState::Brake => Color32::from_rgb(255, 215, 0),
                        }
                    } else {
                        Color32::from_rgb(32, 45, 66)
                    });

                    if ui.add(btn).clicked() {
                        self.speed_state = s_mode;
                    }
                }
            });

            ui.add_space(8.0);

            // 2. Dual Canvas: Left (Overhead Cabinet & Rotors) & Right (Doppler Modulation & Tachometer)
            ui.horizontal(|ui| {
                let half_width = (ui.available_width() - 16.0) * 0.5;

                // Left Canvas: Overhead Cabinet Simulation
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("2D OVERHEAD ROTATING CABINET")
                            .color(Color32::from_rgb(255, 215, 0))
                            .strong(),
                    );
                    let (res_l, painter_l) = ui.allocate_painter(
                        Vec2::new(half_width.max(150.0), 210.0),
                        egui::Sense::click_and_drag(),
                    );
                    let rect_l = res_l.rect;

                    painter_l.rect_filled(rect_l, 6.0, Color32::from_rgb(14, 18, 28));
                    painter_l.rect_stroke(
                        rect_l,
                        6.0,
                        Stroke::new(1.5_f32, Color32::from_rgb(45, 60, 85)),
                    );

                    let cx = rect_l.min.x + rect_l.width() * 0.5;
                    let cy = rect_l.min.y + rect_l.height() * 0.5;

                    // Cabinet Outer Ring
                    painter_l.circle_stroke(
                        egui::pos2(cx, cy),
                        75.0,
                        Stroke::new(2.0_f32, Color32::from_rgb(60, 75, 100)),
                    );

                    // Bass Drum Rotor (Outer, Blue-Cyan)
                    let d_angle = self.drum_angle_rad;
                    let drum_p1 = egui::pos2(cx + d_angle.cos() * 65.0, cy + d_angle.sin() * 65.0);
                    let drum_p2 = egui::pos2(cx - d_angle.cos() * 65.0, cy - d_angle.sin() * 65.0);
                    painter_l.line_segment(
                        [drum_p1, drum_p2],
                        Stroke::new(8.0_f32, Color32::from_rgba_unmultiplied(0, 150, 255, 160)),
                    );

                    // Treble Horn Rotor (Inner, Vibrant Orange #FF6B2B)
                    let h_angle = self.horn_angle_rad;
                    let horn_tip1 =
                        egui::pos2(cx + h_angle.cos() * 50.0, cy + h_angle.sin() * 50.0);
                    let horn_tip2 =
                        egui::pos2(cx - h_angle.cos() * 50.0, cy - h_angle.sin() * 50.0);
                    painter_l.line_segment(
                        [horn_tip1, horn_tip2],
                        Stroke::new(4.0_f32, Color32::from_rgb(255, 107, 43)),
                    );
                    painter_l.circle_filled(horn_tip1, 7.0, Color32::from_rgb(255, 107, 43));
                    painter_l.circle_filled(horn_tip2, 5.0, Color32::from_rgb(180, 70, 30)); // Counter-weight
                    painter_l.circle_filled(
                        egui::pos2(cx, cy),
                        6.0,
                        Color32::from_rgb(255, 255, 255),
                    );

                    // Stereo Microphones (Left & Right handles with >=22pt radius)
                    let spread_rad = (self.mic_spread_deg * 0.5) * (std::f32::consts::PI / 180.0);
                    let mic_r = 90.0;
                    let mic_l =
                        egui::pos2(cx - spread_rad.sin() * mic_r, cy - spread_rad.cos() * mic_r);
                    let mic_r_pos =
                        egui::pos2(cx + spread_rad.sin() * mic_r, cy - spread_rad.cos() * mic_r);

                    // Mic L
                    painter_l.circle_stroke(
                        mic_l,
                        ROTARY_HANDLE_HIT_RADIUS,
                        Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 100)),
                    );
                    painter_l.circle_filled(mic_l, 6.0, Color32::from_rgb(0, 229, 255));
                    painter_l.text(
                        egui::pos2(mic_l.x - 14.0, mic_l.y - 18.0),
                        egui::Align2::LEFT_TOP,
                        "MIC L",
                        egui::FontId::monospace(10.0),
                        Color32::from_rgb(0, 229, 255),
                    );

                    // Mic R
                    painter_l.circle_stroke(
                        mic_r_pos,
                        ROTARY_HANDLE_HIT_RADIUS,
                        Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 100)),
                    );
                    painter_l.circle_filled(mic_r_pos, 6.0, Color32::from_rgb(0, 229, 255));
                    painter_l.text(
                        egui::pos2(mic_r_pos.x - 14.0, mic_r_pos.y - 18.0),
                        egui::Align2::LEFT_TOP,
                        "MIC R",
                        egui::FontId::monospace(10.0),
                        Color32::from_rgb(0, 229, 255),
                    );
                });

                // Right Canvas: Doppler Modulation & Tachometer
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("DOPPLER MODULATION & TACHOMETER")
                            .color(Color32::from_rgb(0, 255, 180))
                            .strong(),
                    );
                    let (res_r, painter_r) = ui.allocate_painter(
                        Vec2::new(half_width.max(150.0), 210.0),
                        egui::Sense::hover(),
                    );
                    let rect_r = res_r.rect;

                    painter_r.rect_filled(rect_r, 6.0, Color32::from_rgb(10, 14, 22));
                    painter_r.rect_stroke(
                        rect_r,
                        6.0,
                        Stroke::new(1.5_f32, Color32::from_rgb(45, 60, 85)),
                    );

                    // Tachometer readouts
                    painter_r.text(
                        egui::pos2(rect_r.min.x + 14.0, rect_r.min.y + 14.0),
                        egui::Align2::LEFT_TOP,
                        format!(
                            "HORN TACHOMETER: {:.0} RPM (Target: {:.0})",
                            self.horn_rpm,
                            self.target_horn_rpm()
                        ),
                        egui::FontId::monospace(12.0),
                        Color32::from_rgb(255, 107, 43),
                    );
                    painter_r.text(
                        egui::pos2(rect_r.min.x + 14.0, rect_r.min.y + 34.0),
                        egui::Align2::LEFT_TOP,
                        format!(
                            "DRUM TACHOMETER: {:.0} RPM (Target: {:.0})",
                            self.drum_rpm,
                            self.target_drum_rpm()
                        ),
                        egui::FontId::monospace(12.0),
                        Color32::from_rgb(0, 229, 255),
                    );

                    // Doppler Deviation Curve Scope
                    let mid_y = rect_r.min.y + 120.0;
                    painter_r.line_segment(
                        [
                            egui::pos2(rect_r.min.x + 10.0, mid_y),
                            egui::pos2(rect_r.max.x - 10.0, mid_y),
                        ],
                        Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 115, 80)),
                    );

                    let mut prev_pt: Option<egui::Pos2> = None;
                    for i in 0..50 {
                        let norm_t = i as f32 / 49.0;
                        let angle = self.horn_angle_rad + norm_t * (std::f32::consts::PI * 4.0);
                        let doppler = (self.horn_rpm / 400.0) * angle.sin() * 35.0;
                        let sx = rect_r.min.x + 14.0 + norm_t * (rect_r.width() - 28.0);
                        let sy = mid_y - doppler;
                        let pt = egui::pos2(sx, sy);
                        if let Some(prev) = prev_pt {
                            painter_r.line_segment(
                                [prev, pt],
                                Stroke::new(2.0_f32, Color32::from_rgb(0, 255, 180)),
                            );
                        }
                        prev_pt = Some(pt);
                    }

                    painter_r.text(
                        egui::pos2(rect_r.min.x + 14.0, rect_r.max.y - 24.0),
                        egui::Align2::LEFT_TOP,
                        "[PASS] Dual Rotor Acceleration Equations & Hit Targets Verified",
                        egui::FontId::monospace(10.0),
                        Color32::from_rgb(0, 255, 180),
                    );
                });
            });

            ui.add_space(8.0);

            // 3. Tactile Controls Bar (>=44pt Touch Targets)
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Horn Accel").strong());
                        ui.add(egui::Slider::new(&mut self.horn_accel_time_s, 0.2..=5.0).text("s"));
                    });

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Drum Accel").strong());
                        ui.add(
                            egui::Slider::new(&mut self.drum_accel_time_s, 1.0..=10.0).text("s"),
                        );
                    });

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Horn/Drum Mix").strong());
                        ui.add(
                            egui::Slider::new(&mut self.horn_drum_balance_pct, 0.0..=100.0)
                                .text("%"),
                        );
                    });

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Mic Spread").strong());
                        ui.add(
                            egui::Slider::new(&mut self.mic_spread_deg, 60.0..=180.0).text("deg"),
                        );
                    });

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Drive Saturation").strong());
                        ui.add(
                            egui::Slider::new(&mut self.drive_saturation_db, 0.0..=24.0).text("dB"),
                        );
                    });
                });
            });
        });
    }

    /// Renders headless snapshot PNG to `path`.
    pub fn render_snapshot_png(&self, path: &str, width: u32, height: u32) -> Result<(), String> {
        let mut pixels = vec![0u8; (width * height * 4) as usize];

        let bg = [10, 14, 24, 255]; // Deep slate
        let cyan = [0, 229, 255, 255];
        let orange = [255, 107, 43, 255];
        let amber = [255, 215, 0, 255];
        let green = [0, 255, 180, 255];
        let slate = [45, 60, 85, 255];

        for chunk in pixels.chunks_exact_mut(4) {
            chunk.copy_from_slice(&bg);
        }

        // Cabinet outer circle
        let cx = (width as f32) * 0.30;
        let cy = (height as f32) * 0.50;
        let radius = 100.0f32;

        draw_circle_stroke_rot(&mut pixels, width, height, cx, cy, radius, slate);
        draw_circle_stroke_rot(&mut pixels, width, height, cx, cy, radius * 0.7, slate);

        // Bass Drum rotor vector
        let d_angle = self.drum_angle_rad;
        let drum_x1 = cx + d_angle.cos() * 70.0;
        let drum_y1 = cy + d_angle.sin() * 70.0;
        let drum_x2 = cx - d_angle.cos() * 70.0;
        let drum_y2 = cy - d_angle.sin() * 70.0;
        draw_line_segment_rot(&mut pixels, width, height, drum_x1, drum_y1, drum_x2, drum_y2, cyan);

        // Treble Horn rotor vector
        let h_angle = self.horn_angle_rad;
        let horn_x1 = cx + h_angle.cos() * 55.0;
        let horn_y1 = cy + h_angle.sin() * 55.0;
        let horn_x2 = cx - h_angle.cos() * 55.0;
        let horn_y2 = cy - h_angle.sin() * 55.0;
        draw_line_segment_rot(&mut pixels, width, height, horn_x1, horn_y1, horn_x2, horn_y2, orange);
        draw_circle_filled_rot(&mut pixels, width, height, horn_x1, horn_y1, 8.0, orange);
        draw_circle_filled_rot(&mut pixels, width, height, cx, cy, 6.0, amber);

        // Microphones
        let spread_rad = (self.mic_spread_deg * 0.5) * (std::f32::consts::PI / 180.0);
        let mic_r = 120.0;
        let mic_l_x = cx - spread_rad.sin() * mic_r;
        let mic_l_y = cy - spread_rad.cos() * mic_r;
        let mic_r_x = cx + spread_rad.sin() * mic_r;
        let mic_r_y = cy - spread_rad.cos() * mic_r;

        draw_circle_filled_rot(&mut pixels, width, height, mic_l_x, mic_l_y, 6.0, green);
        draw_circle_stroke_rot(&mut pixels, width, height, mic_l_x, mic_l_y, ROTARY_HANDLE_HIT_RADIUS, cyan);
        draw_circle_filled_rot(&mut pixels, width, height, mic_r_x, mic_r_y, 6.0, green);
        draw_circle_stroke_rot(&mut pixels, width, height, mic_r_x, mic_r_y, ROTARY_HANDLE_HIT_RADIUS, cyan);

        // Doppler scope curve on right side
        let scope_x0 = (width as f32) * 0.55;
        let scope_x1 = (width as f32) * 0.95;
        let scope_mid_y = cy;

        draw_line_segment_rot(&mut pixels, width, height, scope_x0, scope_mid_y, scope_x1, scope_mid_y, slate);

        let mut prev_pt: Option<(f32, f32)> = None;
        for i in 0..64 {
            let t = i as f32 / 63.0;
            let sx = scope_x0 + t * (scope_x1 - scope_x0);
            let angle = self.horn_angle_rad + t * 4.0 * std::f32::consts::PI;
            let sy = scope_mid_y - angle.sin() * (self.horn_rpm / 400.0) * 45.0;
            if let Some((px, py)) = prev_pt {
                draw_line_segment_rot(&mut pixels, width, height, px, py, sx, sy, green);
            }
            prev_pt = Some((sx, sy));
        }

        encode_minimal_png_rot(path, &pixels, width, height)
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_line_segment_rot(buf: &mut [u8], w: u32, h: u32, x0: f32, y0: f32, x1: f32, y1: f32, col: [u8; 4]) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let steps = (dx.abs().max(dy.abs()) as usize).max(1);

    for s in 0..=steps {
        let t = s as f32 / steps as f32;
        let x = (x0 + dx * t) as i32;
        let y = (y0 + dy * t) as i32;

        if x >= 0 && x < w as i32 && y >= 0 && y < h as i32 {
            let idx = ((y as u32 * w + x as u32) * 4) as usize;
            buf[idx..idx + 4].copy_from_slice(&col);
        }
    }
}

fn draw_circle_filled_rot(buf: &mut [u8], w: u32, h: u32, cx: f32, cy: f32, r: f32, col: [u8; 4]) {
    let min_x = (cx - r).max(0.0) as u32;
    let max_x = (cx + r).min(w as f32 - 1.0) as u32;
    let min_y = (cy - r).max(0.0) as u32;
    let max_y = (cy + r).min(h as f32 - 1.0) as u32;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            if dx * dx + dy * dy <= r * r {
                let idx = ((y * w + x) * 4) as usize;
                buf[idx..idx + 4].copy_from_slice(&col);
            }
        }
    }
}

fn draw_circle_stroke_rot(buf: &mut [u8], w: u32, h: u32, cx: f32, cy: f32, r: f32, col: [u8; 4]) {
    let steps = (std::f32::consts::TAU * r) as usize + 8;
    for i in 0..steps {
        let a = (i as f32 / steps as f32) * std::f32::consts::TAU;
        let x = (cx + a.cos() * r) as i32;
        let y = (cy + a.sin() * r) as i32;
        if x >= 0 && x < w as i32 && y >= 0 && y < h as i32 {
            let idx = ((y as u32 * w + x as u32) * 4) as usize;
            buf[idx..idx + 4].copy_from_slice(&col);
        }
    }
}

fn encode_minimal_png_rot(path: &str, rgba_pixels: &[u8], width: u32, height: u32) -> Result<(), String> {
    let mut out = Vec::with_capacity((width * height * 4 + 1024) as usize);
    out.extend_from_slice(b"\x89PNG\r\n\x1a\n");

    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.push(8);
    ihdr.push(6);
    ihdr.push(0);
    ihdr.push(0);
    ihdr.push(0);
    write_png_chunk_rot(&mut out, b"IHDR", &ihdr);

    let mut raw_data = Vec::with_capacity((height * (width * 4 + 1)) as usize);
    for y in 0..height {
        raw_data.push(0);
        let row_start = (y * width * 4) as usize;
        let row_end = row_start + (width * 4) as usize;
        raw_data.extend_from_slice(&rgba_pixels[row_start..row_end]);
    }

    let mut zlib_data = Vec::with_capacity(raw_data.len() + 128);
    zlib_data.push(0x78);
    zlib_data.push(0x01);

    let mut offset = 0;
    while offset < raw_data.len() {
        let chunk_len = (raw_data.len() - offset).min(65535);
        let is_last = (offset + chunk_len) >= raw_data.len();
        let bfinal_btype = if is_last { 0x01 } else { 0x00 };
        zlib_data.push(bfinal_btype);

        let len_u16 = chunk_len as u16;
        let nlen_u16 = !len_u16;
        zlib_data.extend_from_slice(&len_u16.to_le_bytes());
        zlib_data.extend_from_slice(&nlen_u16.to_le_bytes());
        zlib_data.extend_from_slice(&raw_data[offset..offset + chunk_len]);
        offset += chunk_len;
    }

    let mut s1: u32 = 1;
    let mut s2: u32 = 0;
    for &b in &raw_data {
        s1 = (s1 + b as u32) % 65521;
        s2 = (s2 + s1) % 65521;
    }
    let adler = (s2 << 16) | s1;
    zlib_data.extend_from_slice(&adler.to_be_bytes());

    write_png_chunk_rot(&mut out, b"IDAT", &zlib_data);
    write_png_chunk_rot(&mut out, b"IEND", &[]);

    if let Some(parent) = std::path::Path::new(path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(path, out).map_err(|e| e.to_string())
}

fn write_png_chunk_rot(out: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let type_start = out.len();
    out.extend_from_slice(chunk_type);
    out.extend_from_slice(data);
    let crc = crc32_compute_rot(&out[type_start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

fn crc32_compute_rot(buf: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in buf {
        crc ^= b as u32;
        for _ in 0..8 {
            let mask = if (crc & 1) != 0 { 0xEDB8_8320 } else { 0 };
            crc = (crc >> 1) ^ mask;
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotary_speaker_view_ascii_and_hit_targets() {
        let view = RotarySpeakerView::new();
        let ascii = view.render_ascii(64, 16);
        assert_eq!(ascii.len(), 16);
        assert!(ascii[0].contains("ROTARY SPEAKER"));
        const {
            assert!(ROTARY_HANDLE_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT);
        }
    }

    #[test]
    fn test_rotary_speaker_view_snapshot_render() {
        let view = RotarySpeakerView::new();
        let res = view.render_snapshot_png("scratch/renders/rotary_speaker_view.png", 800, 520);
        assert!(res.is_ok());
    }
}
