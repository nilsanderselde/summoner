// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Microtonal Tuning Scale Isomorphic Keyboard Visualizer with Scala (SCL/KBM) Intervals (Step 1363).

use crate::touch_controls::{ContrastColorPalette, MIN_HIT_TARGET_PT};

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const DEFAULT_KEY_RADIUS_PT: f32 = 26.0; // Diameter ~52pt (> 44x44pt hit target)

/// Interval category for microtonal color grading and harmonic function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntervalCategory {
    RootUnison,
    PerfectFifth,
    MajorThird,
    MinorThird,
    NeutralThird,
    SubminorSupermajor,
    Octave,
    OtherMicrotonal,
}

impl IntervalCategory {
    pub fn color_rgb(&self) -> (u8, u8, u8) {
        match self {
            Self::RootUnison => (0, 255, 180),           // Mint green
            Self::PerfectFifth => (0, 229, 255),         // Cyan
            Self::MajorThird => (255, 215, 0),           // Gold
            Self::MinorThird => (255, 140, 60),          // Amber orange
            Self::NeutralThird => (255, 64, 129),        // Rose violet
            Self::SubminorSupermajor => (179, 136, 255), // Lavender
            Self::Octave => (0, 255, 180),               // Mint green
            Self::OtherMicrotonal => (76, 201, 240),     // Light blue
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::RootUnison => "1/1 Root",
            Self::PerfectFifth => "3/2 P5",
            Self::MajorThird => "5/4 M3",
            Self::MinorThird => "6/5 m3",
            Self::NeutralThird => "Neutral 3rd",
            Self::SubminorSupermajor => "Sub/Super Int.",
            Self::Octave => "2/1 Octave",
            Self::OtherMicrotonal => "Microtonal Step",
        }
    }
}

/// A single key node in the 2D isomorphic hexagonal keyboard.
#[derive(Debug, Clone, PartialEq)]
pub struct IsomorphicKey {
    pub col: i32,
    pub row: i32,
    pub step_index: i32, // Step index relative to root in EDO/Scale
    pub cents_from_root: f64,
    pub frequency_hz: f64,
    pub note_name: String,
    pub category: IntervalCategory,
    pub is_root: bool,
    pub is_scale_member: bool,
    pub is_pressed: bool,
}

impl IsomorphicKey {
    pub fn new(
        col: i32,
        row: i32,
        step_index: i32,
        edo: u16,
        root_hz: f64,
        key_name: impl Into<String>,
    ) -> Self {
        let edo_f = edo.max(1) as f64;
        let cents = (step_index as f64) * (1200.0 / edo_f);
        let freq = root_hz * 2.0_f64.powf(step_index as f64 / edo_f);

        // Normalize step into single octave [0 .. edo)
        let oct_step = step_index.rem_euclid(edo as i32);
        let oct_cents = (oct_step as f64) * (1200.0 / edo_f);

        let (category, is_root) = if oct_step == 0 {
            (IntervalCategory::RootUnison, true)
        } else if (oct_cents - 700.0).abs() < 35.0 {
            (IntervalCategory::PerfectFifth, false)
        } else if (oct_cents - 400.0).abs() < 30.0 {
            (IntervalCategory::MajorThird, false)
        } else if (oct_cents - 300.0).abs() < 30.0 {
            (IntervalCategory::MinorThird, false)
        } else if (oct_cents - 350.0).abs() < 30.0 {
            (IntervalCategory::NeutralThird, false)
        } else if (oct_cents - 1200.0).abs() < 10.0 {
            (IntervalCategory::Octave, false)
        } else {
            (IntervalCategory::OtherMicrotonal, false)
        };

        Self {
            col,
            row,
            step_index,
            cents_from_root: cents,
            frequency_hz: freq,
            note_name: key_name.into(),
            category,
            is_root,
            is_scale_member: true,
            is_pressed: false,
        }
    }
}

/// Microtonal Tuning Scale Isomorphic Keyboard Visualizer View (Step 1363).
#[derive(Debug, Clone)]
pub struct IsomorphicTuningKeyboardView {
    pub edo_division: u16, // e.g. 19, 22, 31, 12
    pub root_hz: f64,      // e.g. 261.63 (C4)
    pub root_note_name: String,
    pub key_radius_pt: f32,
    pub row_generator_steps: i32, // e.g. +7 in 12/19-EDO (Fifth)
    pub col_generator_steps: i32, // e.g. +2 in 12/19-EDO (Whole tone)
    pub keys: Vec<IsomorphicKey>,
    pub pressed_keys: Vec<(i32, i32)>, // List of currently held (col, row)
    pub show_cents: bool,
    pub scale_name: String,
    pub color_palette: ContrastColorPalette,
}

impl Default for IsomorphicTuningKeyboardView {
    fn default() -> Self {
        Self::new(19, 261.6255653, "C4", "19-EDO Equal Temperament")
    }
}

impl IsomorphicTuningKeyboardView {
    pub fn new(
        edo_division: u16,
        root_hz: f64,
        root_name: impl Into<String>,
        scale_name: impl Into<String>,
    ) -> Self {
        let edo = edo_division.max(5);
        let root_str = root_name.into();
        let mut view = Self {
            edo_division: edo,
            root_hz,
            root_note_name: root_str,
            key_radius_pt: DEFAULT_KEY_RADIUS_PT,
            row_generator_steps: if edo == 19 { 11 } else { 7 },
            col_generator_steps: if edo == 19 { 3 } else { 2 },
            keys: Vec::new(),
            pressed_keys: Vec::new(),
            show_cents: true,
            scale_name: scale_name.into(),
            color_palette: ContrastColorPalette::default(),
        };

        view.rebuild_grid(7, 4); // 7 columns x 4 rows
        view
    }

    /// Rebuild hexagonal isomorphic grid for given dimensions.
    pub fn rebuild_grid(&mut self, num_cols: i32, num_rows: i32) {
        self.keys.clear();
        for r in 0..num_rows {
            for c in 0..num_cols {
                let step_idx = c * self.col_generator_steps + r * self.row_generator_steps;
                let note_name = format!("K[{},{}]", c, r);
                let key =
                    IsomorphicKey::new(c, r, step_idx, self.edo_division, self.root_hz, note_name);
                self.keys.push(key);
            }
        }
    }

    /// Calculate center coordinate in screen points for key at (col, row).
    pub fn calculate_key_center(
        col: i32,
        row: i32,
        origin: (f32, f32),
        radius_pt: f32,
    ) -> (f32, f32) {
        let hex_w = radius_pt * 1.7320508_f32; // sqrt(3) * r
        let hex_h = radius_pt * 1.5_f32;

        let offset_x = if row % 2 != 0 {
            hex_w * 0.5_f32
        } else {
            0.0_f32
        };
        let cx = origin.0 + col as f32 * (hex_w + 4.0_f32) + offset_x;
        let cy = origin.1 + row as f32 * (hex_h + 4.0_f32);
        (cx, cy)
    }

    /// Hit test screen point to locate tapped key (col, row).
    pub fn hit_test_key(&self, pos: (f32, f32), origin: (f32, f32)) -> Option<usize> {
        let hit_radius = self.key_radius_pt.max(MIN_HIT_TARGET_PT * 0.5_f32);
        for (idx, key) in self.keys.iter().enumerate() {
            let (cx, cy) = Self::calculate_key_center(key.col, key.row, origin, self.key_radius_pt);
            let dx = pos.0 - cx;
            let dy = pos.1 - cy;
            if (dx * dx + dy * dy).sqrt() <= hit_radius {
                return Some(idx);
            }
        }
        None
    }

    /// Press key down.
    pub fn press_key(&mut self, idx: usize) {
        if idx < self.keys.len() {
            self.keys[idx].is_pressed = true;
            let pair = (self.keys[idx].col, self.keys[idx].row);
            if !self.pressed_keys.contains(&pair) {
                self.pressed_keys.push(pair);
            }
        }
    }

    /// Release key.
    pub fn release_key(&mut self, idx: usize) {
        if idx < self.keys.len() {
            self.keys[idx].is_pressed = false;
            let pair = (self.keys[idx].col, self.keys[idx].row);
            self.pressed_keys.retain(|p| *p != pair);
        }
    }

    /// Release all held keys.
    pub fn release_all(&mut self) {
        for key in &mut self.keys {
            key.is_pressed = false;
        }
        self.pressed_keys.clear();
    }

    /// Generate deterministic ASCII representation of the isomorphic key grid.
    pub fn render_ascii(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "Isomorphic Keyboard [{} EDO] Root: {} ({:.1} Hz)\n",
            self.edo_division, self.root_note_name, self.root_hz
        ));
        for r in 0..4 {
            let indent = if r % 2 != 0 { "  " } else { "" };
            out.push_str(indent);
            for c in 0..7 {
                if let Some(key) = self.keys.iter().find(|k| k.col == c && k.row == r) {
                    let mark = if key.is_pressed {
                        '#'
                    } else if key.is_root {
                        'R'
                    } else {
                        'o'
                    };
                    out.push_str(&format!("[{}{:+3}] ", mark, key.step_index));
                }
            }
            out.push('\n');
        }
        out
    }
}

#[cfg(feature = "gui")]
impl IsomorphicTuningKeyboardView {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // 1. Header Toolbar
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("MICROTONAL ISOMORPHIC TUNING KEYBOARD")
                        .color(Color32::from_rgb(240, 245, 255))
                        .strong(),
                );
                ui.separator();
                ui.label(
                    egui::RichText::new(&self.scale_name)
                        .color(Color32::from_rgb(0, 229, 255))
                        .strong(),
                );
                ui.separator();
                ui.label(format!(
                    "Root: {} ({:.2} Hz)",
                    self.root_note_name, self.root_hz
                ));
            });

            // 2. Interval Legend & Mode Bar
            ui.horizontal(|ui| {
                let legend = [
                    (IntervalCategory::RootUnison, "1/1 Root"),
                    (IntervalCategory::PerfectFifth, "3/2 Fifth"),
                    (IntervalCategory::MajorThird, "5/4 Maj3"),
                    (IntervalCategory::NeutralThird, "Neutral 3rd"),
                ];
                for (cat, label) in legend {
                    let (r, g, b) = cat.color_rgb();
                    ui.colored_label(Color32::from_rgb(r, g, b), format!("● {}", label));
                }
                ui.separator();
                ui.checkbox(&mut self.show_cents, "Show Cents");
            });

            ui.add_space(8.0_f32);

            // 3. Hexagonal Key Canvas
            let canvas_w = ui.available_width().max(650.0_f32);
            let canvas_h = 320.0_f32;

            let (response, painter) =
                ui.allocate_painter(Vec2::new(canvas_w, canvas_h), egui::Sense::click_and_drag());

            let origin = (
                response.rect.min.x + 35.0_f32,
                response.rect.min.y + 35.0_f32,
            );

            // Handle touch interactions
            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if let Some(idx) = self.hit_test_key((pos.x, pos.y), origin) {
                        let was_pressed = self.keys[idx].is_pressed;
                        if was_pressed {
                            self.release_key(idx);
                        } else {
                            self.press_key(idx);
                        }
                    }
                }
            }

            // Draw Keys
            for key in &self.keys {
                let (cx, cy) =
                    Self::calculate_key_center(key.col, key.row, origin, self.key_radius_pt);
                let center_pos = egui::pos2(cx, cy);

                let (r, g, b) = key.category.color_rgb();
                let fill_col = if key.is_pressed {
                    Color32::from_rgb(255, 255, 255)
                } else if key.is_root {
                    Color32::from_rgb(0, 255, 180)
                } else {
                    Color32::from_rgb(r / 3 + 10, g / 3 + 12, b / 3 + 20)
                };

                let stroke_col = if key.is_pressed {
                    Color32::from_rgb(0, 229, 255)
                } else {
                    Color32::from_rgb(r, g, b)
                };

                // Draw hexagon disc / key body (Radius >= 26pt -> Diameter >= 52pt > 44pt)
                painter.circle_filled(center_pos, self.key_radius_pt, fill_col);
                let stroke_w = if key.is_root { 2.5_f32 } else { 1.5_f32 };
                painter.circle_stroke(
                    center_pos,
                    self.key_radius_pt,
                    Stroke::new(stroke_w, stroke_col),
                );

                // Text labels inside key
                let txt_color = if key.is_pressed {
                    Color32::from_rgb(10, 14, 20)
                } else {
                    Color32::from_rgb(240, 245, 255)
                };

                // Step number
                painter.text(
                    egui::pos2(cx, cy - 6.0_f32),
                    egui::Align2::CENTER_CENTER,
                    format!("{:+}", key.step_index),
                    egui::FontId::proportional(12.0_f32),
                    txt_color,
                );

                // Cents / Hz readout
                if self.show_cents {
                    painter.text(
                        egui::pos2(cx, cy + 8.0_f32),
                        egui::Align2::CENTER_CENTER,
                        format!("{:.0}¢", key.cents_from_root),
                        egui::FontId::proportional(9.0_f32),
                        if key.is_pressed {
                            Color32::from_rgb(20, 24, 30)
                        } else {
                            Color32::from_rgb(180, 205, 235)
                        },
                    );
                }
            }

            ui.add_space(8.0_f32);

            // 4. Held Keys Inspector
            if !self.pressed_keys.is_empty() {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "HELD KEYS: {} active",
                            self.pressed_keys.len()
                        ))
                        .color(Color32::from_rgb(0, 255, 180))
                        .strong(),
                    );
                    if ui.button("Release All").clicked() {
                        self.release_all();
                    }
                });
            }
        });
    }
}
