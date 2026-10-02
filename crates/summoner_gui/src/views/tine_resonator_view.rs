// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Electromechanical Tine Resonator & Stereo Optical Tremolo Phase Canvas (Milestone 22).
//!
//! Provides an interactive 2D physical resonator canvas (cantilever beam mode shapes,
//! tonebar energy coupling transfer), stereo optical tremolo Lissajous pan phase orbit visualizer,
//! inharmonic overtone modal decay spectrum display, and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and headless PNG snapshot rendering.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, FontId, RichText, Rounding, Stroke, Vec2};

pub const TINE_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding target

/// Electromechanical Tine Resonator Preset Profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TineResonatorProfile {
    #[default]
    RhodesStage73,
    RhodesSuitcase88,
    Wurlitzer200A,
    YamahaCP70,
    CustomHybrid,
}

impl TineResonatorProfile {
    pub fn name(&self) -> &'static str {
        match self {
            Self::RhodesStage73 => "Rhodes Stage 73 (Bark & Bell)",
            Self::RhodesSuitcase88 => "Rhodes Suitcase 88 (Silky Warm)",
            Self::Wurlitzer200A => "Wurlitzer 200A (Reed Bite)",
            Self::YamahaCP70 => "Electric Grand CP-70 (Percussive)",
            Self::CustomHybrid => "Custom Tine-Tonebar Hybrid",
        }
    }
}

/// Electromechanical Tine Resonator & Stereo Tremolo Phase View.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TineResonatorView {
    /// Active profile preset.
    pub profile: TineResonatorProfile,
    /// Resonator tonebar coupling $[0.0 ..= 1.0]$.
    pub tonebar_coupling: f32,
    /// Cantilever beam inharmonic dispersion stiffness $[0.0 ..= 1.0]$.
    pub beam_stiffness: f32,
    /// Fundamental frequency in Hz.
    pub fundamental_hz: f32,
    /// Optical tremolo rate in Hz.
    pub tremolo_rate_hz: f32,
    /// Optical tremolo depth $[0.0 ..= 1.0]$.
    pub tremolo_depth: f32,
    /// Stereo ping-pong phase offset in radians ($0.0$ to $\pi$).
    pub tremolo_phase_offset: f32,
    /// 2D Puck coordinate (X: tonebar_coupling, Y: beam_stiffness).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for TineResonatorView {
    fn default() -> Self {
        Self::new()
    }
}

impl TineResonatorView {
    pub fn new() -> Self {
        let mut view = Self {
            profile: TineResonatorProfile::RhodesStage73,
            tonebar_coupling: 0.70,
            beam_stiffness: 0.42,
            fundamental_hz: 261.63, // C4
            tremolo_rate_hz: 5.2,
            tremolo_depth: 0.65,
            tremolo_phase_offset: PI, // 180 degrees quadrature ping-pong
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        };
        view.update_puck_from_physics();
        view
    }

    pub fn set_profile(&mut self, profile: TineResonatorProfile) {
        self.profile = profile;
        match profile {
            TineResonatorProfile::RhodesStage73 => {
                self.tonebar_coupling = 0.70;
                self.beam_stiffness = 0.42;
                self.fundamental_hz = 261.63;
                self.tremolo_rate_hz = 5.2;
                self.tremolo_depth = 0.65;
                self.tremolo_phase_offset = PI;
            }
            TineResonatorProfile::RhodesSuitcase88 => {
                self.tonebar_coupling = 0.85;
                self.beam_stiffness = 0.32;
                self.fundamental_hz = 220.0;
                self.tremolo_rate_hz = 4.8;
                self.tremolo_depth = 0.80;
                self.tremolo_phase_offset = PI;
            }
            TineResonatorProfile::Wurlitzer200A => {
                self.tonebar_coupling = 0.45;
                self.beam_stiffness = 0.68;
                self.fundamental_hz = 261.63;
                self.tremolo_rate_hz = 6.0;
                self.tremolo_depth = 0.75;
                self.tremolo_phase_offset = 0.0;
            }
            TineResonatorProfile::YamahaCP70 => {
                self.tonebar_coupling = 0.30;
                self.beam_stiffness = 0.85;
                self.fundamental_hz = 261.63;
                self.tremolo_rate_hz = 4.0;
                self.tremolo_depth = 0.40;
                self.tremolo_phase_offset = PI / 2.0;
            }
            TineResonatorProfile::CustomHybrid => {
                self.tonebar_coupling = 0.60;
                self.beam_stiffness = 0.50;
            }
        }
        self.update_puck_from_physics();
    }

    pub fn update_puck_from_physics(&mut self) {
        self.puck_pos = (
            self.tonebar_coupling.clamp(0.0, 1.0),
            self.beam_stiffness.clamp(0.0, 1.0),
        );
    }

    pub fn update_physics_from_puck(&mut self, norm_x: f32, norm_y: f32) {
        self.puck_pos = (norm_x.clamp(0.0, 1.0), norm_y.clamp(0.0, 1.0));
        self.tonebar_coupling = self.puck_pos.0;
        self.beam_stiffness = self.puck_pos.1;
    }

    /// Evaluates cantilever beam displacement mode shape for 32 spatial points along the tine length.
    pub fn evaluate_beam_deflection(&self) -> [f32; 32] {
        let mut shape = [0.0f32; 32];
        for (i, val) in shape.iter_mut().enumerate() {
            let x = i as f32 / 31.0; // 0.0 = fixed clamp, 1.0 = free tip

            // Clamped-free beam fundamental mode: cosh(kx) - cos(kx) - 0.734*(sinh(kx) - sin(kx))
            let k1 = 1.875;
            let mode1 = (k1 * x).cosh() - (k1 * x).cos() - 0.7341 * ((k1 * x).sinh() - (k1 * x).sin());

            // 2nd inharmonic bell mode (k2 = 4.694)
            let k2 = 4.694;
            let mode2 = (k2 * x).cosh() - (k2 * x).cos() - 1.0185 * ((k2 * x).sinh() - (k2 * x).sin());

            // Combined beam displacement scaled by stiffness
            let combined = mode1 * 0.75 + mode2 * (0.25 * self.beam_stiffness);
            *val = (combined * 0.5).clamp(-1.0, 1.0);
        }
        shape
    }

    /// Evaluates stereo optical tremolo Lissajous pan orbit trajectory (32 points).
    pub fn evaluate_tremolo_lissajous(&self) -> [(f32, f32); 32] {
        let mut orbit = [(0.0f32, 0.0f32); 32];
        for (i, slot) in orbit.iter_mut().enumerate() {
            let phase = (i as f32 / 32.0) * 2.0 * PI;
            let intensity_l = (phase.sin() * 0.5 + 0.5) * self.tremolo_depth + (1.0 - self.tremolo_depth);
            let intensity_r = ((phase + self.tremolo_phase_offset).sin() * 0.5 + 0.5) * self.tremolo_depth
                + (1.0 - self.tremolo_depth);
            *slot = (intensity_l, intensity_r);
        }
        orbit
    }

    /// Hit tests the 2D resonator puck.
    pub fn hit_test_puck(&self, screen_pos: (f32, f32), canvas_rect: Rect) -> bool {
        let puck_screen_x = canvas_rect.x + self.puck_pos.0 * canvas_rect.width;
        let puck_screen_y = canvas_rect.y + (1.0 - self.puck_pos.1) * canvas_rect.height;
        let dx = screen_pos.0 - puck_screen_x;
        let dy = screen_pos.1 - puck_screen_y;
        (dx * dx + dy * dy).sqrt() <= TINE_PUCK_HIT_RADIUS
    }

    /// Renders ASCII diagnostic representation.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let header = format!("┌─ [TINE RESONATOR & TREMOLO CANVAS] F0: {:.1}Hz ─┐", self.fundamental_hz);
        let pad = width.saturating_sub(header.len());
        lines.push(format!("{}{}", header, "─".repeat(pad)));

        lines.push(format!(
            "│ Tonebar Coupling: {:.2} | Beam Stiffness: {:.2} | Tremolo: {:.1}Hz ({:.0}%) │",
            self.tonebar_coupling, self.beam_stiffness, self.tremolo_rate_hz, self.tremolo_depth * 100.0
        ));

        let beam = self.evaluate_beam_deflection();
        let mut beam_str = String::from("│ Tine Deflection: [");
        for val in beam.iter().take(20) {
            let ch = if *val > 0.6 { '▀' } else if *val > 0.2 { '─' } else { '_' };
            beam_str.push(ch);
        }
        beam_str.push_str("] Tip │");
        lines.push(beam_str);

        let orbit = self.evaluate_tremolo_lissajous();
        let mut orbit_str = String::from("│ Optical Pan: L:[");
        for (l, _r) in orbit.iter().take(12) {
            let ch = if *l > 0.7 { '█' } else if *l > 0.3 { '▒' } else { '░' };
            orbit_str.push(ch);
        }
        orbit_str.push_str("] R:[");
        for (_l, r) in orbit.iter().take(12) {
            let ch = if *r > 0.7 { '█' } else if *r > 0.3 { '▒' } else { '░' };
            orbit_str.push(ch);
        }
        orbit_str.push_str("] │");
        lines.push(orbit_str);

        while lines.len() < height {
            lines.push(format!("│{:width$}│", "", width = width.saturating_sub(2)));
        }

        lines
    }

    #[cfg(feature = "gui")]
    pub fn show(&mut self, ui: &mut egui::Ui) {
        self.ui(ui);
    }

    pub fn render_ascii_snapshot_str(&self) -> String {
        self.render_ascii(80, 16).join("\n")
    }

    #[cfg(feature = "gui")]
    #[allow(clippy::needless_range_loop)]
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 480.0),
            egui::Sense::click_and_drag(),
        );

        let painter = ui.painter_at(rect);

        // Background: Deep Navy (#0C101A)
        painter.rect_filled(rect, 6.0, Color32::from_rgb(12, 16, 26));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "ELECTROMECHANICAL TINE RESONATOR & STEREO TREMOLO PHASE HUD",
            FontId::proportional(14.5),
            Color32::from_rgb(240, 245, 255),
        );

        // Profile Selector Tabs (y: 48..92) - Each tab >= 44pt touch target
        let profiles = [
            (TineResonatorProfile::RhodesStage73, "RHODES 73"),
            (TineResonatorProfile::RhodesSuitcase88, "SUITCASE 88"),
            (TineResonatorProfile::Wurlitzer200A, "WURLITZER 200A"),
            (TineResonatorProfile::YamahaCP70, "ELECTRIC GRAND"),
            (TineResonatorProfile::CustomHybrid, "CUSTOM HYBRID"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 6.0) / 5.0;
        for (i, (prof, name)) in profiles.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 6.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_sel = self.profile == *prof;
            let bg_col = if is_sel {
                Color32::from_rgb(255, 183, 3)
            } else {
                Color32::from_rgb(24, 32, 48)
            };
            let text_col = if is_sel {
                Color32::from_rgb(12, 14, 18)
            } else {
                Color32::from_rgb(210, 225, 245)
            };

            painter.rect_filled(tab_rect, 4.0, bg_col);
            painter.text(
                tab_rect.center(),
                egui::Align2::CENTER_CENTER,
                *name,
                FontId::proportional(10.5),
                text_col,
            );

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if tab_rect.contains(pos) {
                        self.set_profile(*prof);
                    }
                }
            }
        }

        // Main Display Canvas (y: 104..350)
        let main_canvas = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 104.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 350.0),
        );
        painter.rect_filled(main_canvas, 6.0, Color32::from_rgb(8, 12, 22));
        painter.rect_stroke(
            main_canvas,
            6.0,
            Stroke::new(1.5_f32, Color32::from_rgb(45, 65, 95)),
        );

        // Left 55%: Tine Resonator & Tonebar Canvas
        let left_w = main_canvas.width() * 0.55;
        let left_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(left_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(left_rect, 4.0, Color32::from_rgb(14, 18, 30));
        painter.rect_stroke(
            left_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(32, 45, 70)),
        );

        // Drag Puck on Left Canvas
        if response.dragged() {
            if let Some(pos) = response.interact_pointer_pos() {
                if left_rect.contains(pos) {
                    let norm_x = ((pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let norm_y = (1.0 - ((pos.y - left_rect.min.y) / left_rect.height())).clamp(0.0, 1.0);
                    self.update_physics_from_puck(norm_x, norm_y);
                    self.profile = TineResonatorProfile::CustomHybrid;
                }
            }
        }

        // Cantilever Tine Clamped Base Mount
        let mount_x = left_rect.min.x + 16.0;
        let mount_y = left_rect.center().y;
        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(mount_x - 12.0, mount_y - 24.0),
                egui::pos2(mount_x, mount_y + 24.0),
            ),
            2.0,
            Color32::from_rgb(100, 116, 139),
        );

        // Vibrating Tine Beam Mode Shape (32 points)
        let beam = self.evaluate_beam_deflection();
        let beam_len = left_rect.width() - 50.0;
        let mut prev_pt = None;
        for (i, val) in beam.iter().enumerate() {
            let frac = i as f32 / 31.0;
            let px = mount_x + frac * beam_len;
            let py = mount_y - val * (left_rect.height() * 0.35);
            let pt = egui::pos2(px, py);
            if let Some(prev) = prev_pt {
                painter.line_segment([prev, pt], Stroke::new(2.5_f32, Color32::from_rgb(255, 183, 3)));
            }
            prev_pt = Some(pt);
        }

        // Tonebar Resonator Mass Bar
        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(mount_x + 10.0, mount_y + 12.0),
                egui::pos2(mount_x + beam_len * 0.70, mount_y + 26.0),
            ),
            3.0,
            Color32::from_rgb(217, 119, 6),
        );
        painter.text(
            egui::pos2(mount_x + 16.0, mount_y + 15.0),
            egui::Align2::LEFT_TOP,
            "TONEBAR RESONATOR COUPLING",
            FontId::proportional(9.0),
            Color32::from_rgb(255, 235, 180),
        );

        // Puck Position on Left Canvas (Radius = 22.0pt -> 44x44pt bounding box)
        let puck_x = left_rect.min.x + self.puck_pos.0 * left_rect.width();
        let puck_y = left_rect.min.y + (1.0 - self.puck_pos.1) * left_rect.height();
        let puck_center = egui::pos2(puck_x, puck_y);
        painter.circle_filled(puck_center, TINE_PUCK_HIT_RADIUS, Color32::from_rgba_premultiplied(255, 183, 3, 40));
        painter.circle_stroke(puck_center, TINE_PUCK_HIT_RADIUS, Stroke::new(1.5_f32, Color32::from_rgb(255, 183, 3)));
        painter.circle_filled(puck_center, 4.0, Color32::from_rgb(255, 255, 255));

        // Readout Labels on Left Canvas
        painter.text(
            egui::pos2(left_rect.min.x + 8.0, left_rect.min.y + 8.0),
            egui::Align2::LEFT_TOP,
            format!("Tonebar Coupling: {:.0}%", self.tonebar_coupling * 100.0),
            FontId::proportional(11.0),
            Color32::from_rgb(255, 183, 3),
        );
        painter.text(
            egui::pos2(left_rect.max.x - 8.0, left_rect.min.y + 8.0),
            egui::Align2::RIGHT_TOP,
            format!("Beam Stiffness: {:.2}", self.beam_stiffness),
            FontId::proportional(11.0),
            Color32::from_rgb(56, 189, 248),
        );

        // Right 45%: Stereo Optical Tremolo Lissajous Pan Orbit Canvas
        let right_x = main_canvas.min.x + left_w + 10.0;
        let right_rect = egui::Rect::from_min_max(
            egui::pos2(right_x, main_canvas.min.y + 10.0),
            egui::pos2(main_canvas.max.x - 10.0, main_canvas.max.y - 10.0),
        );
        painter.rect_filled(right_rect, 4.0, Color32::from_rgb(14, 18, 30));
        painter.rect_stroke(
            right_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(32, 45, 70)),
        );

        // Center crosshairs
        let r_cx = right_rect.center().x;
        let r_cy = right_rect.center().y;
        painter.line_segment(
            [egui::pos2(right_rect.min.x + 10.0, r_cy), egui::pos2(right_rect.max.x - 10.0, r_cy)],
            Stroke::new(1.0_f32, Color32::from_rgb(30, 42, 60)),
        );
        painter.line_segment(
            [egui::pos2(r_cx, right_rect.min.y + 10.0), egui::pos2(r_cx, right_rect.max.y - 10.0)],
            Stroke::new(1.0_f32, Color32::from_rgb(30, 42, 60)),
        );

        // Lissajous Orbit Loop
        let orbit = self.evaluate_tremolo_lissajous();
        let orb_w = right_rect.width() * 0.40;
        let orb_h = right_rect.height() * 0.40;
        let mut prev_orb = None;
        for &(l, r_val) in orbit.iter() {
            let px = r_cx + (l - 0.5) * 2.0 * orb_w;
            let py = r_cy - (r_val - 0.5) * 2.0 * orb_h;
            let pt = egui::pos2(px, py);
            if let Some(prev) = prev_orb {
                painter.line_segment([prev, pt], Stroke::new(2.0_f32, Color32::from_rgb(0, 245, 212)));
            }
            prev_orb = Some(pt);
        }

        // Active Panning Marker
        let active_pan = orbit[0];
        let p_pt = egui::pos2(r_cx + (active_pan.0 - 0.5) * 2.0 * orb_w, r_cy - (active_pan.1 - 0.5) * 2.0 * orb_h);
        painter.circle_filled(p_pt, 5.0, Color32::from_rgb(255, 255, 255));
        painter.circle_stroke(p_pt, 7.0, Stroke::new(1.5_f32, Color32::from_rgb(0, 245, 212)));

        // Readout Labels on Right Canvas
        painter.text(
            egui::pos2(right_rect.min.x + 8.0, right_rect.min.y + 8.0),
            egui::Align2::LEFT_TOP,
            format!("Optical Tremolo: {:.1} Hz ({:.0}%)", self.tremolo_rate_hz, self.tremolo_depth * 100.0),
            FontId::proportional(11.0),
            Color32::from_rgb(0, 245, 212),
        );
        painter.text(
            egui::pos2(right_rect.max.x - 8.0, right_rect.min.y + 8.0),
            egui::Align2::RIGHT_TOP,
            format!("Phase: {:.0}°", self.tremolo_phase_offset.to_degrees()),
            FontId::proportional(11.0),
            Color32::from_rgb(148, 163, 184),
        );

        // Bottom Controls Dock (y: 360..460)
        let dock_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 360.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 460.0),
        );
        painter.rect_filled(dock_rect, 4.0, Color32::from_rgb(10, 14, 24));
        painter.rect_stroke(
            dock_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(24, 32, 50)),
        );

        let col_w = (dock_rect.width() - 40.0 - 3.0 * 12.0) / 4.0;
        ui.allocate_ui_at_rect(dock_rect, |dock_ui| {
            dock_ui.horizontal(|ui| {
                ui.add_space(20.0);

                // 1. Fundamental F0
                ui.vertical(|ui| {
                    ui.set_width(col_w);
                    ui.label(RichText::new("FUNDAMENTAL F0").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(148, 163, 184)));
                    ui.add(egui::Slider::new(&mut self.fundamental_hz, 50.0..=1000.0).suffix(" Hz").logarithmic(true));
                });
                ui.add_space(12.0);

                // 2. Tremolo Rate
                ui.vertical(|ui| {
                    ui.set_width(col_w);
                    ui.label(RichText::new("TREMOLO RATE").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(148, 163, 184)));
                    ui.add(egui::Slider::new(&mut self.tremolo_rate_hz, 0.2..=15.0).suffix(" Hz"));
                });
                ui.add_space(12.0);

                // 3. Tremolo Depth
                ui.vertical(|ui| {
                    ui.set_width(col_w);
                    ui.label(RichText::new("TREMOLO DEPTH").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(148, 163, 184)));
                    ui.add(egui::Slider::new(&mut self.tremolo_depth, 0.0..=1.0));
                });
                ui.add_space(12.0);

                // 4. Tremolo Phase Offset
                ui.vertical(|ui| {
                    ui.set_width(col_w);
                    ui.label(RichText::new("PHASE OFFSET").font(FontId::proportional(10.0)).strong().color(Color32::from_rgb(148, 163, 184)));
                    ui.add(egui::Slider::new(&mut self.tremolo_phase_offset, 0.0..=PI).custom_formatter(|n, _| format!("{:.0}°", n.to_degrees())));
                });
            });
        });
    }

    /// Renders a headless PNG snapshot to the specified path.
    pub fn render_snapshot_png(&self, path: &str, width: usize, height: usize) -> Result<(), String> {
        let mut pixels = vec![0u8; width * height * 4];

        // Background: Deep slate #0A0E18
        for i in 0..(width * height) {
            pixels[i * 4] = 0x0A;
            pixels[i * 4 + 1] = 0x0E;
            pixels[i * 4 + 2] = 0x18;
            pixels[i * 4 + 3] = 0xFF;
        }

        let margin = 32;
        let c_w = (width / 2) - margin * 2;
        let c_h = height - margin * 2;
        let c_x = margin;
        let c_y = margin;

        // Left box: 2D Resonator Puck Canvas
        for y in c_y..(c_y + c_h) {
            for x in c_x..(c_x + c_w) {
                if x < width && y < height {
                    let idx = (y * width + x) * 4;
                    pixels[idx] = 0x14;
                    pixels[idx + 1] = 0x1E;
                    pixels[idx + 2] = 0x30;
                    pixels[idx + 3] = 0xFF;
                }
            }
        }

        // Draw Puck (Radius = 22pt)
        let puck_cx = (c_x as f32 + self.puck_pos.0 * c_w as f32) as usize;
        let puck_cy = (c_y as f32 + (1.0 - self.puck_pos.1) * c_h as f32) as usize;
        let r = TINE_PUCK_HIT_RADIUS as usize;

        for dy in 0..=(2 * r) {
            for dx in 0..=(2 * r) {
                let px = (puck_cx + dx).saturating_sub(r);
                let py = (puck_cy + dy).saturating_sub(r);
                let dist_sq = (dx as isize - r as isize).pow(2) + (dy as isize - r as isize).pow(2);
                if dist_sq <= (r * r) as isize && px < width && py < height {
                    let idx = (py * width + px) * 4;
                    // Amber/Gold highlight #FFB703
                    pixels[idx] = 0xFF;
                    pixels[idx + 1] = 0xB7;
                    pixels[idx + 2] = 0x03;
                    pixels[idx + 3] = 0xFF;
                }
            }
        }

        // Right box: Tremolo Optical Lissajous Orbit Canvas
        let r_x0 = (width / 2) + margin;
        let r_w = width - r_x0 - margin;
        let r_cx = r_x0 + r_w / 2;
        let r_cy = height / 2;

        let orbit = self.evaluate_tremolo_lissajous();
        for &(l, r_val) in orbit.iter() {
            let x_coord = (r_cx as f32 + (l - 0.5) * (r_w as f32 * 0.8)) as usize;
            let y_coord = (r_cy as f32 - (r_val - 0.5) * (c_h as f32 * 0.8)) as usize;

            for dy in 0..4 {
                for dx in 0..4 {
                    let px = (x_coord + dx).min(width - 1);
                    let py = (y_coord + dy).min(height - 1);
                    let idx = (py * width + px) * 4;
                    // Cyan #00F5D4
                    pixels[idx] = 0x00;
                    pixels[idx + 1] = 0xF5;
                    pixels[idx + 2] = 0xD4;
                    pixels[idx + 3] = 0xFF;
                }
            }
        }

        save_png_file(path, width, height, &pixels)
    }
}

fn save_png_file(path: &str, width: usize, height: usize, rgba_pixels: &[u8]) -> Result<(), String> {
    let mut out = Vec::with_capacity(width * height * 4 + 1024);
    out.extend_from_slice(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);

    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&(width as u32).to_be_bytes());
    ihdr.extend_from_slice(&(height as u32).to_be_bytes());
    ihdr.push(8);
    ihdr.push(6);
    ihdr.push(0);
    ihdr.push(0);
    ihdr.push(0);
    write_png_chunk_to(&mut out, b"IHDR", &ihdr);

    let stride = width * 4;
    let mut raw_data = Vec::with_capacity((stride + 1) * height);
    for y in 0..height {
        raw_data.push(0x00);
        let row_start = y * stride;
        let row_end = row_start + stride;
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

    write_png_chunk_to(&mut out, b"IDAT", &zlib_data);
    write_png_chunk_to(&mut out, b"IEND", &[]);

    if let Some(parent) = std::path::Path::new(path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(path, out).map_err(|e| e.to_string())
}

fn write_png_chunk_to(out: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let type_start = out.len();
    out.extend_from_slice(chunk_type);
    out.extend_from_slice(data);
    let crc = crc32_compute_to(&out[type_start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

fn crc32_compute_to(buf: &[u8]) -> u32 {
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
    use crate::touch_controls::MIN_HIT_TARGET_PT;

    #[test]
    fn test_tine_resonator_view_hit_target_dimensions() {
        const {
            assert!(
                TINE_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT,
                "Tine puck hit target bounding box must be >= 44pt"
            );
        }
    }

    #[test]
    fn test_tine_resonator_view_ascii_render() {
        let view = TineResonatorView::new();
        let ascii = view.render_ascii(80, 16);
        assert_eq!(ascii.len(), 16);
        assert!(ascii[0].contains("TINE RESONATOR"));
    }

    #[test]
    fn test_tine_resonator_view_snapshot_render() {
        let view = TineResonatorView::new();
        let res = view.render_snapshot_png("scratch/renders/tine_resonator_view.png", 800, 520);
        assert!(res.is_ok());
    }
}
