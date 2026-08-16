// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Multi-Touch Polyphonic Ribbon Expression Strip Controller (Step 1402).

use crate::layout_math::Rect;
use crate::touch_controls::{ContrastColorPalette, MIN_HIT_TARGET_PT};

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const RIBBON_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const RIBBON_PUCK_VISUAL_RADIUS: f32 = 14.0;
pub const MAX_POLYPHONIC_TOUCHES: usize = 8;

/// Quantization mode for ribbon pitch snapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RibbonQuantizeMode {
    ContinuousGlissando,
    SemitoneStep,
    MajorScale,
    MinorPentatonic,
    MicrotonalEdo(u16), // e.g. 19-EDO, 31-EDO, 53-EDO
}

/// Single active touch contact point on the polyphonic ribbon.
#[derive(Debug, Clone, PartialEq)]
pub struct RibbonTouchPoint {
    pub id: usize,
    pub note_pitch: f32, // Fractional MIDI pitch (60.0 = C4)
    pub y_timbre: f32,   // 0.0 ..= 1.0 (MPE Y-axis / CC74 Brightness)
    pub pressure: f32,   // 0.0 ..= 1.0 (Channel Pressure / Aftertouch)
    pub is_active: bool,
}

impl RibbonTouchPoint {
    pub fn new(id: usize, note_pitch: f32, y_timbre: f32, pressure: f32) -> Self {
        Self {
            id,
            note_pitch,
            y_timbre: y_timbre.clamp(0.0, 1.0),
            pressure: pressure.clamp(0.0, 1.0),
            is_active: true,
        }
    }
}

/// Interactive Polyphonic Ribbon Expression Controller View (Step 1402).
#[derive(Debug, Clone)]
pub struct RibbonControllerView {
    pub touches: Vec<RibbonTouchPoint>,
    pub base_note_midi: u8, // default 36 (C2)
    pub num_octaves: u8,    // default 4 (48 semitones range)
    pub quantize_mode: RibbonQuantizeMode,
    pub glissando_rate_ms: f32, // 0.0 ..= 200.0 ms
    pub mpe_y_cc_target: u8,    // default 74 (Brightness / Timbre)
    pub active_touch_id: Option<usize>,
    pub octave_shift: i8, // -2 ..= +2 octaves
    pub scale_root_midi: u8,
    pub color_palette: ContrastColorPalette,
}

impl Default for RibbonControllerView {
    fn default() -> Self {
        Self::new()
    }
}

impl RibbonControllerView {
    pub fn new() -> Self {
        let initial_touches = vec![
            RibbonTouchPoint::new(0, 48.0, 0.70, 0.85), // C3
            RibbonTouchPoint::new(1, 55.0, 0.45, 0.60), // G3
            RibbonTouchPoint::new(2, 64.0, 0.90, 0.75), // E4
        ];

        Self {
            touches: initial_touches,
            base_note_midi: 36, // C2
            num_octaves: 4,     // C2 to C6
            quantize_mode: RibbonQuantizeMode::ContinuousGlissando,
            glissando_rate_ms: 25.0,
            mpe_y_cc_target: 74,
            active_touch_id: Some(0),
            octave_shift: 0,
            scale_root_midi: 0, // C
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Total semitones spanned by the ribbon controller.
    pub fn total_semitones(&self) -> f32 {
        (self.num_octaves as f32 * 12.0).max(12.0)
    }

    /// Effective base MIDI note including octave shift.
    pub fn effective_base_note(&self) -> f32 {
        let shifted = self.base_note_midi as i16 + (self.octave_shift as i16 * 12);
        shifted.clamp(0, 108) as f32
    }

    /// Convert fractional MIDI pitch to normalized horizontal ribbon position [0.0 ..= 1.0].
    pub fn pitch_to_norm_x(&self, pitch: f32) -> f32 {
        let base = self.effective_base_note();
        let total = self.total_semitones();
        ((pitch - base) / total).clamp(0.0, 1.0)
    }

    /// Convert normalized horizontal ribbon position [0.0 ..= 1.0] to fractional MIDI pitch.
    pub fn norm_x_to_pitch(&self, norm_x: f32) -> f32 {
        let base = self.effective_base_note();
        let total = self.total_semitones();
        let raw_pitch = base + norm_x.clamp(0.0, 1.0) * total;
        self.apply_quantization(raw_pitch)
    }

    /// Apply active quantization mode to pitch.
    pub fn apply_quantization(&self, pitch: f32) -> f32 {
        match self.quantize_mode {
            RibbonQuantizeMode::ContinuousGlissando => pitch,
            RibbonQuantizeMode::SemitoneStep => pitch.round(),
            RibbonQuantizeMode::MajorScale => {
                let note_in_oct =
                    (pitch.round() as i32 - self.scale_root_midi as i32).rem_euclid(12);
                let major_intervals = [0, 2, 4, 5, 7, 9, 11];
                let nearest = major_intervals
                    .iter()
                    .min_by_key(|&&intv| (intv - note_in_oct).abs())
                    .copied()
                    .unwrap_or(0);
                pitch.round() - (note_in_oct - nearest) as f32
            }
            RibbonQuantizeMode::MinorPentatonic => {
                let note_in_oct =
                    (pitch.round() as i32 - self.scale_root_midi as i32).rem_euclid(12);
                let pent_intervals = [0, 3, 5, 7, 10];
                let nearest = pent_intervals
                    .iter()
                    .min_by_key(|&&intv| (intv - note_in_oct).abs())
                    .copied()
                    .unwrap_or(0);
                pitch.round() - (note_in_oct - nearest) as f32
            }
            RibbonQuantizeMode::MicrotonalEdo(edo) => {
                let divisions = (edo as f32).max(1.0);
                let semitone_ratio = 12.0 / divisions;
                (pitch / semitone_ratio).round() * semitone_ratio
            }
        }
    }

    /// Convert pitch to canvas screen X coordinate.
    pub fn pitch_to_screen_x(&self, pitch: f32, canvas: Rect) -> f32 {
        canvas.x + self.pitch_to_norm_x(pitch) * canvas.width
    }

    /// Convert canvas screen X coordinate to pitch.
    pub fn screen_x_to_pitch(&self, screen_x: f32, canvas: Rect) -> f32 {
        if canvas.width <= 0.0 {
            return self.effective_base_note();
        }
        let norm_x = ((screen_x - canvas.x) / canvas.width).clamp(0.0, 1.0);
        self.norm_x_to_pitch(norm_x)
    }

    /// Convert canvas screen Y coordinate to normalized MPE Y Timbre value [0.0 ..= 1.0].
    pub fn screen_y_to_timbre(&self, screen_y: f32, canvas: Rect) -> f32 {
        if canvas.height <= 0.0 {
            return 0.5;
        }
        (1.0 - (screen_y - canvas.y) / canvas.height).clamp(0.0, 1.0)
    }

    /// Convert normalized MPE Y Timbre value to canvas screen Y coordinate.
    pub fn timbre_to_screen_y(&self, timbre: f32, canvas: Rect) -> f32 {
        canvas.y + (1.0 - timbre.clamp(0.0, 1.0)) * canvas.height
    }

    /// Hit-test active touch contact point with ergonomic minimum hit target (>=44x44pt).
    pub fn hit_test_touch(&self, pos: (f32, f32), canvas: Rect) -> Option<usize> {
        for (i, touch) in self.touches.iter().enumerate() {
            if !touch.is_active {
                continue;
            }
            let tx = self.pitch_to_screen_x(touch.note_pitch, canvas);
            let ty = self.timbre_to_screen_y(touch.y_timbre, canvas);
            let dist = ((pos.0 - tx).powi(2) + (pos.1 - ty).powi(2)).sqrt();
            if dist <= RIBBON_PUCK_HIT_RADIUS {
                return Some(i);
            }
        }
        None
    }

    /// Add or update touch contact point on ribbon press.
    pub fn trigger_touch(&mut self, pos: (f32, f32), canvas: Rect, pressure: f32) -> usize {
        let pitch = self.screen_x_to_pitch(pos.0, canvas);
        let timbre = self.screen_y_to_timbre(pos.1, canvas);

        if let Some(hit_idx) = self.hit_test_touch(pos, canvas) {
            let touch = &mut self.touches[hit_idx];
            touch.note_pitch = pitch;
            touch.y_timbre = timbre;
            touch.pressure = pressure.clamp(0.0, 1.0);
            self.active_touch_id = Some(hit_idx);
            hit_idx
        } else if self.touches.len() < MAX_POLYPHONIC_TOUCHES {
            let new_id = self.touches.len();
            self.touches
                .push(RibbonTouchPoint::new(new_id, pitch, timbre, pressure));
            self.active_touch_id = Some(new_id);
            new_id
        } else {
            0
        }
    }

    /// Get standard note name string (e.g. "C4", "F#3 +12.4c").
    pub fn pitch_to_note_string(pitch: f32) -> String {
        let note_names = [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ];
        let semitone_round = pitch.round() as i32;
        let cents = (pitch - pitch.round()) * 100.0;
        let note_idx = semitone_round.rem_euclid(12) as usize;
        let octave = (semitone_round / 12) - 1;
        if cents.abs() < 1.0 {
            format!("{}{}", note_names[note_idx], octave)
        } else {
            format!("{}{}{:+0.1}c", note_names[note_idx], octave, cents)
        }
    }

    /// Deterministic ASCII render of the ribbon expression controller.
    pub fn render_ascii(&self, width: usize) -> String {
        let mut buf = vec!['.'; width];
        for touch in &self.touches {
            if !touch.is_active {
                continue;
            }
            let norm = self.pitch_to_norm_x(touch.note_pitch);
            let pos = ((norm * (width - 1) as f32).round() as usize).min(width - 1);
            buf[pos] = '*';
        }
        buf.into_iter().collect()
    }
}

#[cfg(feature = "gui")]
impl RibbonControllerView {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // 1. Header Toolbar
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("POLYPHONIC RIBBON EXPRESSION CONTROLLER")
                        .color(Color32::from_rgb(240, 245, 255))
                        .strong(),
                );
                ui.separator();
                ui.label(
                    egui::RichText::new(format!(
                        "Touches: {}/{} | Base: C{} | Range: {} Octaves",
                        self.touches.iter().filter(|t| t.is_active).count(),
                        MAX_POLYPHONIC_TOUCHES,
                        (self.effective_base_note() / 12.0) as i32 - 1,
                        self.num_octaves
                    ))
                    .color(Color32::from_rgb(0, 229, 255))
                    .strong(),
                );
            });

            ui.add_space(6.0);

            // 2. Ribbon Expression Canvas (>=44pt Touch Targets & Visual Hit Radii)
            let canvas_w = ui.available_width().max(680.0);
            let canvas_h = 200.0;
            let (response, painter) =
                ui.allocate_painter(Vec2::new(canvas_w, canvas_h), egui::Sense::click_and_drag());
            let canvas = Rect::new(
                response.rect.min.x,
                response.rect.min.y,
                response.rect.width(),
                response.rect.height(),
            );

            // Canvas Background
            painter.rect_filled(response.rect, 8.0_f32, Color32::from_rgb(12, 16, 26));
            painter.rect_stroke(
                response.rect,
                8.0_f32,
                Stroke::new(1.5_f32, Color32::from_rgb(45, 60, 85)),
            );

            // Draw Chromatic Key Markers & Octave Separator Lines
            let total_semitones = self.total_semitones() as usize;
            let semitone_w = canvas.width / total_semitones as f32;
            let base_pitch = self.effective_base_note() as usize;

            for s in 0..total_semitones {
                let note_val = base_pitch + s;
                let is_accidental = matches!(note_val % 12, 1 | 3 | 6 | 8 | 10);
                let sx = canvas.x + s as f32 * semitone_w;

                if is_accidental {
                    painter.rect_filled(
                        egui::Rect::from_min_max(
                            egui::pos2(sx, canvas.y),
                            egui::pos2(sx + semitone_w, canvas.y + canvas.height * 0.6),
                        ),
                        0.0_f32,
                        Color32::from_rgba_unmultiplied(20, 26, 40, 180),
                    );
                }

                // Octave C line
                if note_val.is_multiple_of(12) {
                    painter.line_segment(
                        [
                            egui::pos2(sx, canvas.y),
                            egui::pos2(sx, canvas.y + canvas.height),
                        ],
                        Stroke::new(1.5_f32, Color32::from_rgb(0, 229, 255)),
                    );
                    painter.text(
                        egui::pos2(sx + 4.0, canvas.y + canvas.height - 14.0),
                        egui::Align2::LEFT_BOTTOM,
                        format!("C{}", (note_val / 12) as i32 - 1),
                        egui::FontId::proportional(12.0),
                        Color32::from_rgb(0, 229, 255),
                    );
                }
            }

            // Draw Active Polyphonic Touch Points
            for (idx, touch) in self.touches.iter().enumerate() {
                if !touch.is_active {
                    continue;
                }
                let px = self.pitch_to_screen_x(touch.note_pitch, canvas);
                let py = self.timbre_to_screen_y(touch.y_timbre, canvas);
                let is_sel = self.active_touch_id == Some(idx);

                let puck_color = if is_sel {
                    Color32::from_rgb(255, 215, 0)
                } else {
                    Color32::from_rgb(0, 229, 255)
                };

                // Vertical pitch & timbre projection guide lines
                painter.line_segment(
                    [
                        egui::pos2(px, canvas.y),
                        egui::pos2(px, canvas.y + canvas.height),
                    ],
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 80)),
                );

                // Pressure glow ring
                let dynamic_radius = RIBBON_PUCK_VISUAL_RADIUS * (1.0 + touch.pressure * 0.4);
                painter.circle_filled(
                    egui::pos2(px, py),
                    dynamic_radius + 4.0,
                    Color32::from_rgba_unmultiplied(
                        puck_color.r(),
                        puck_color.g(),
                        puck_color.b(),
                        40,
                    ),
                );

                // Outer 44x44pt Touch Target boundary ring (Radius 22pt)
                painter.circle_stroke(
                    egui::pos2(px, py),
                    RIBBON_PUCK_HIT_RADIUS,
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 120)),
                );

                // Puck center
                painter.circle_filled(egui::pos2(px, py), dynamic_radius, puck_color);
                painter.circle_filled(egui::pos2(px, py), 4.0, Color32::from_rgb(10, 14, 22));

                // Touch Note Name Badge
                let note_str = Self::pitch_to_note_string(touch.note_pitch);
                painter.text(
                    egui::pos2(px, py - dynamic_radius - 12.0),
                    egui::Align2::CENTER_BOTTOM,
                    note_str,
                    egui::FontId::proportional(11.0),
                    Color32::from_rgb(240, 245, 255),
                );
            }

            // Drag and Click Interaction Handling
            if response.drag_started() || response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let tidx = self.trigger_touch((pos.x, pos.y), canvas, 0.80);
                    self.active_touch_id = Some(tidx);
                }
            }

            if response.dragged() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let pitch = self.screen_x_to_pitch(pos.x, canvas);
                    let timbre = self.screen_y_to_timbre(pos.y, canvas);
                    if let Some(tidx) = self.active_touch_id {
                        if tidx < self.touches.len() {
                            self.touches[tidx].note_pitch = pitch;
                            self.touches[tidx].y_timbre = timbre;
                        }
                    }
                }
            }

            ui.add_space(10.0);

            // 3. Quantization Mode Selector (>=44pt Touch Targets)
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("QUANTIZATION:").strong());
                let modes = [
                    (RibbonQuantizeMode::ContinuousGlissando, "Glissando (Free)"),
                    (RibbonQuantizeMode::SemitoneStep, "12-EDO Semitone"),
                    (RibbonQuantizeMode::MajorScale, "Major Scale"),
                    (RibbonQuantizeMode::MinorPentatonic, "Pentatonic"),
                    (RibbonQuantizeMode::MicrotonalEdo(19), "19-EDO Microtonal"),
                    (RibbonQuantizeMode::MicrotonalEdo(31), "31-EDO Microtonal"),
                ];

                for (m, lbl) in modes {
                    let is_act = self.quantize_mode == m;
                    let btn = egui::Button::new(
                        egui::RichText::new(lbl)
                            .color(if is_act {
                                Color32::from_rgb(10, 14, 22)
                            } else {
                                Color32::from_rgb(220, 235, 255)
                            })
                            .strong(),
                    )
                    .min_size(Vec2::new(100.0, MIN_HIT_TARGET_PT))
                    .fill(if is_act {
                        Color32::from_rgb(0, 229, 255)
                    } else {
                        Color32::from_rgb(30, 40, 60)
                    });

                    if ui.add(btn).clicked() {
                        self.quantize_mode = m;
                    }
                }
            });

            ui.add_space(8.0);

            // 4. Expression & MPE Routing Controls
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Octave Shift").strong());
                ui.add(egui::Slider::new(&mut self.octave_shift, -2..=2).text("oct"));

                ui.separator();
                ui.label(egui::RichText::new("Glissando Rate").strong());
                ui.add(egui::Slider::new(&mut self.glissando_rate_ms, 0.0..=200.0).text("ms"));

                ui.separator();
                ui.label(egui::RichText::new("MPE Y Target CC").strong());
                ui.add(egui::Slider::new(&mut self.mpe_y_cc_target, 1..=127).text("CC"));
            });
        });
    }
}
