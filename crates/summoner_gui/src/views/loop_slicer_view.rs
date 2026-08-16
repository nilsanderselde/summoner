// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Tactile Audio Loop Slicer and Beat Repeat Glitch Slice Pad Matrix (Step 1385).

use crate::layout_math::Rect;
use crate::touch_controls::{ContrastColorPalette, MIN_HIT_TARGET_PT};

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const SLICE_MARKER_HIT_RADIUS: f32 = 22.0; // 44x44pt touch box
pub const PAD_SPACING_PT: f32 = 8.0;

/// Glitch and playback mode for a loop slice pad.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlitchPadMode {
    Forward,
    Reverse,
    HalfSpeed,
    DoubleSpeed,
    StutterGate,
    TapeStop,
}

impl GlitchPadMode {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Forward => "FWD (1x)",
            Self::Reverse => "REV (1x)",
            Self::HalfSpeed => "1/2x Slow",
            Self::DoubleSpeed => "2x Fast",
            Self::StutterGate => "Stutter",
            Self::TapeStop => "Tape Stop",
        }
    }
}

/// A single audio slice definition.
#[derive(Debug, Clone, PartialEq)]
pub struct AudioSlice {
    pub id: usize,
    pub start_sample: usize,
    pub end_sample: usize,
    pub pitch_semitones: f32, // -24.0 ..= +24.0 st
    pub gain_db: f32,         // -24.0 ..= +12.0 dB
    pub pan: f32,             // -1.0 ..= +1.0
    pub mode: GlitchPadMode,
    pub choke_group: u8, // 1..=4
    pub is_playing: bool,
}

impl AudioSlice {
    pub fn new(id: usize, start: usize, end: usize) -> Self {
        Self {
            id,
            start_sample: start,
            end_sample: end,
            pitch_semitones: 0.0,
            gain_db: 0.0,
            pan: 0.0,
            mode: GlitchPadMode::Forward,
            choke_group: 1,
            is_playing: false,
        }
    }
}

/// Tactile Audio Loop Slicer and Beat Repeat Pad Matrix View (Step 1385).
#[derive(Debug, Clone)]
pub struct LoopSlicerView {
    pub total_samples: usize,
    pub sample_rate: u32,
    pub bpm: f64,
    pub slices: Vec<AudioSlice>,
    pub selected_slice_idx: usize,
    pub active_playing_slice_idx: Option<usize>,
    pub transient_sensitivity_db: f32, // -48.0 ..= -6.0 dB
    pub min_slice_duration_ms: f32,    // 10.0 ..= 500.0 ms
    pub snap_to_grid: bool,
    pub grid_divisions: u32, // 16 = 16th notes
    pub dragging_marker_idx: Option<usize>,
    pub color_palette: ContrastColorPalette,
}

impl Default for LoopSlicerView {
    fn default() -> Self {
        Self::new(44100 * 2, 44100, 120.0, 16)
    }
}

impl LoopSlicerView {
    pub fn new(total_samples: usize, sample_rate: u32, bpm: f64, num_slices: usize) -> Self {
        let total = total_samples.max(1000);
        let count = num_slices.clamp(4, 32);
        let slice_len = total / count;

        let mut slices = Vec::with_capacity(count);
        for i in 0..count {
            let start = i * slice_len;
            let end = if i == count - 1 {
                total
            } else {
                (i + 1) * slice_len
            };
            let mut slice = AudioSlice::new(i, start, end);
            if i == 3 || i == 7 {
                slice.mode = GlitchPadMode::Reverse;
            } else if i == 11 {
                slice.mode = GlitchPadMode::StutterGate;
            } else if i == 15 {
                slice.mode = GlitchPadMode::TapeStop;
            }
            slices.push(slice);
        }

        Self {
            total_samples: total,
            sample_rate,
            bpm,
            slices,
            selected_slice_idx: 0,
            active_playing_slice_idx: Some(0),
            transient_sensitivity_db: -24.0,
            min_slice_duration_ms: 50.0,
            snap_to_grid: true,
            grid_divisions: 16,
            dragging_marker_idx: None,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Convert sample position to screen X coordinate on waveform strip.
    pub fn sample_to_screen_x(&self, sample: usize, canvas: Rect) -> f32 {
        if self.total_samples == 0 {
            return canvas.x;
        }
        let norm = (sample as f32 / self.total_samples as f32).clamp(0.0, 1.0);
        canvas.x + norm * canvas.width
    }

    /// Convert screen X coordinate to sample position.
    pub fn screen_x_to_sample(&self, screen_x: f32, canvas: Rect) -> usize {
        if canvas.width <= 0.0 {
            return 0;
        }
        let norm = ((screen_x - canvas.x) / canvas.width).clamp(0.0, 1.0);
        (norm * self.total_samples as f32) as usize
    }

    /// Calculate bounding rectangle for a pad in the 4x4 matrix.
    pub fn calculate_pad_rect(
        &self,
        pad_idx: usize,
        origin: (f32, f32),
        pad_size: (f32, f32),
    ) -> Rect {
        let row = pad_idx / 4;
        let col = pad_idx % 4;
        let x = origin.0 + col as f32 * (pad_size.0 + PAD_SPACING_PT);
        let y = origin.1 + row as f32 * (pad_size.1 + PAD_SPACING_PT);
        Rect::new(x, y, pad_size.0, pad_size.1)
    }

    /// Hit-test pad matrix cells (guaranteed >= 44x44pt).
    pub fn hit_test_pad(
        &self,
        pos: (f32, f32),
        origin: (f32, f32),
        pad_size: (f32, f32),
    ) -> Option<usize> {
        let count = self.slices.len().min(16);
        for idx in 0..count {
            let rect = self.calculate_pad_rect(idx, origin, pad_size);
            if rect.contains(pos.0, pos.1) {
                return Some(idx);
            }
        }
        None
    }

    /// Hit-test slice boundary handles on waveform strip (>=44x44pt touch area).
    pub fn hit_test_slice_marker(&self, pos: (f32, f32), canvas: Rect) -> Option<usize> {
        if pos.1 < canvas.y - 10.0 || pos.1 > canvas.y + canvas.height + 10.0 {
            return None;
        }

        for (idx, slice) in self.slices.iter().enumerate() {
            let sx = self.sample_to_screen_x(slice.start_sample, canvas);
            if (pos.0 - sx).abs() <= SLICE_MARKER_HIT_RADIUS {
                return Some(idx);
            }
        }
        None
    }

    /// Trigger a slice pad to play.
    pub fn trigger_pad(&mut self, pad_idx: usize) {
        if pad_idx < self.slices.len() {
            self.selected_slice_idx = pad_idx;
            self.active_playing_slice_idx = Some(pad_idx);
            for (i, slice) in self.slices.iter_mut().enumerate() {
                slice.is_playing = i == pad_idx;
            }
        }
    }

    /// Deterministic ASCII render of slices and pads.
    pub fn render_ascii(&self, num_cols: usize) -> String {
        let w = num_cols.max(16);
        let mut buf = vec!['.'; w];

        for (idx, slice) in self.slices.iter().enumerate() {
            let pos = ((slice.start_sample as f32 / self.total_samples as f32) * (w - 1) as f32)
                .round() as usize;
            if pos < w {
                let symbol = match slice.mode {
                    GlitchPadMode::Forward => '|',
                    GlitchPadMode::Reverse => 'R',
                    GlitchPadMode::HalfSpeed => 'S',
                    GlitchPadMode::DoubleSpeed => 'F',
                    GlitchPadMode::StutterGate => 'G',
                    GlitchPadMode::TapeStop => 'T',
                };
                buf[pos] = symbol;
            }
            if Some(idx) == self.active_playing_slice_idx && pos < w {
                buf[pos] = '*';
            }
        }

        buf.into_iter().collect()
    }
}

#[cfg(feature = "gui")]
impl LoopSlicerView {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // 1. Header Toolbar
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("TACTILE LOOP SLICER & GLITCH PAD MATRIX")
                        .color(Color32::from_rgb(240, 245, 255))
                        .strong(),
                );
                ui.separator();
                ui.label(
                    egui::RichText::new(format!(
                        "Slices: {} | Active Pad: #{}",
                        self.slices.len(),
                        self.selected_slice_idx + 1
                    ))
                    .color(Color32::from_rgb(0, 229, 255))
                    .strong(),
                );
                ui.separator();
                ui.checkbox(&mut self.snap_to_grid, "Snap to Grid");
            });

            ui.add_space(6.0);

            // 2. Waveform Overview Strip Canvas
            let canvas_w = ui.available_width().max(680.0);
            let canvas_h = 100.0;
            let (response, painter) =
                ui.allocate_painter(Vec2::new(canvas_w, canvas_h), egui::Sense::click_and_drag());
            let canvas = Rect::new(
                response.rect.min.x,
                response.rect.min.y,
                response.rect.width(),
                response.rect.height(),
            );

            // Background
            painter.rect_filled(response.rect, 6.0_f32, Color32::from_rgb(10, 14, 22));
            painter.rect_stroke(
                response.rect,
                6.0_f32,
                Stroke::new(1.5_f32, Color32::from_rgb(40, 55, 80)),
            );

            // Draw Simulated Audio Waveform Peaks
            let num_peaks = 120;
            for i in 0..num_peaks {
                let t = i as f32 / num_peaks as f32;
                let x = canvas.x + t * canvas.width;
                // Simulated drum loop transient pattern
                let beat_pulse = ((t * 8.0_f32 * std::f32::consts::PI).sin().abs()).powi(4);
                let noise = ((t * 43.1_f32).cos().abs()) * 0.3_f32;
                let amp = (beat_pulse * 0.8_f32 + noise).clamp(0.05_f32, 0.95_f32);

                let top_y = canvas.y + (canvas.height * 0.5_f32) - amp * (canvas.height * 0.45_f32);
                let bot_y = canvas.y + (canvas.height * 0.5_f32) + amp * (canvas.height * 0.45_f32);

                painter.line_segment(
                    [egui::pos2(x, top_y), egui::pos2(x, bot_y)],
                    Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(0, 180, 215, 160)),
                );
            }

            // Draw Slice Boundaries and Highlight Active Slice
            for (idx, slice) in self.slices.iter().enumerate() {
                let sx = self.sample_to_screen_x(slice.start_sample, canvas);
                let ex = self.sample_to_screen_x(slice.end_sample, canvas);

                if idx == self.selected_slice_idx {
                    let sel_rect = egui::Rect::from_min_max(
                        egui::pos2(sx, canvas.y),
                        egui::pos2(ex, canvas.y + canvas.height),
                    );
                    painter.rect_filled(
                        sel_rect,
                        0.0_f32,
                        Color32::from_rgba_unmultiplied(255, 215, 0, 45),
                    );
                }

                // Slice Marker Vertical Line
                painter.line_segment(
                    [
                        egui::pos2(sx, canvas.y),
                        egui::pos2(sx, canvas.y + canvas.height),
                    ],
                    Stroke::new(
                        if idx == self.selected_slice_idx {
                            2.0_f32
                        } else {
                            1.0_f32
                        },
                        if idx == self.selected_slice_idx {
                            Color32::from_rgb(255, 215, 0)
                        } else {
                            Color32::from_rgb(0, 229, 255)
                        },
                    ),
                );

                // Touch marker puck at top (22pt radius = 44x44pt touch area)
                let handle_center = egui::pos2(sx, canvas.y + 12.0_f32);
                painter.circle_filled(handle_center, 6.0_f32, Color32::from_rgb(0, 229, 255));
                painter.text(
                    egui::pos2(sx + 3.0_f32, canvas.y + 24.0_f32),
                    egui::Align2::LEFT_TOP,
                    format!("{}", idx + 1),
                    egui::FontId::proportional(9.0_f32),
                    Color32::from_rgb(220, 235, 255),
                );
            }

            // Waveform Drag Interaction
            if response.drag_started() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if let Some(m_idx) = self.hit_test_slice_marker((pos.x, pos.y), canvas) {
                        self.dragging_marker_idx = Some(m_idx);
                        self.selected_slice_idx = m_idx;
                    }
                }
            }

            if response.dragged() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if let Some(m_idx) = self.dragging_marker_idx {
                        let new_sample = self.screen_x_to_sample(pos.x, canvas);
                        self.slices[m_idx].start_sample = new_sample;
                    }
                }
            }

            if response.drag_stopped() {
                self.dragging_marker_idx = None;
            }

            ui.add_space(8.0);

            // 3. Tactile 4x4 Beat Repeat Pad Matrix (>=44x44pt Touch Targets)
            let pad_w = 120.0_f32;
            let pad_h = MIN_HIT_TARGET_PT + 12.0_f32; // 56.0pt > 44pt

            ui.horizontal(|ui| {
                // Left column: 4x4 Pad Grid
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("PAD MATRIX (4x4 MPC SLICE TRIGGER)").strong());
                    for row in 0..4 {
                        ui.horizontal(|ui| {
                            for col in 0..4 {
                                let pad_idx = row * 4 + col;
                                if pad_idx < self.slices.len() {
                                    let slice = &self.slices[pad_idx];
                                    let is_sel = pad_idx == self.selected_slice_idx;
                                    let is_play = self.active_playing_slice_idx == Some(pad_idx);

                                    let bg_col = if is_play {
                                        Color32::from_rgb(0, 255, 180)
                                    } else if is_sel {
                                        Color32::from_rgb(255, 215, 0)
                                    } else {
                                        match slice.mode {
                                            GlitchPadMode::Forward => Color32::from_rgb(30, 42, 62),
                                            GlitchPadMode::Reverse => Color32::from_rgb(55, 30, 65),
                                            GlitchPadMode::HalfSpeed => {
                                                Color32::from_rgb(30, 55, 55)
                                            }
                                            GlitchPadMode::DoubleSpeed => {
                                                Color32::from_rgb(60, 45, 25)
                                            }
                                            GlitchPadMode::StutterGate => {
                                                Color32::from_rgb(65, 30, 30)
                                            }
                                            GlitchPadMode::TapeStop => {
                                                Color32::from_rgb(45, 45, 45)
                                            }
                                        }
                                    };

                                    let text_col = if is_play || is_sel {
                                        Color32::from_rgb(10, 14, 22)
                                    } else {
                                        Color32::from_rgb(240, 245, 255)
                                    };

                                    let pad_btn = egui::Button::new(
                                        egui::RichText::new(format!(
                                            "PAD {:02}\n{}",
                                            pad_idx + 1,
                                            slice.mode.display_name()
                                        ))
                                        .color(text_col)
                                        .strong(),
                                    )
                                    .min_size(Vec2::new(pad_w, pad_h))
                                    .fill(bg_col);

                                    if ui.add(pad_btn).clicked() {
                                        self.trigger_pad(pad_idx);
                                    }
                                }
                            }
                        });
                    }
                });

                ui.separator();

                // Right column: Selected Pad Inspector Card
                if let Some(curr_slice) = self.slices.get_mut(self.selected_slice_idx) {
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "PAD #{:02} CONFIGURATION",
                                    curr_slice.id + 1
                                ))
                                .color(Color32::from_rgb(255, 215, 0))
                                .strong(),
                            );
                            ui.separator();

                            // Mode Selector Buttons (>=44pt Touch Targets)
                            ui.label(egui::RichText::new("Glitch Mode:").strong());
                            let modes = [
                                GlitchPadMode::Forward,
                                GlitchPadMode::Reverse,
                                GlitchPadMode::HalfSpeed,
                                GlitchPadMode::DoubleSpeed,
                                GlitchPadMode::StutterGate,
                                GlitchPadMode::TapeStop,
                            ];
                            for m in modes {
                                let is_act = curr_slice.mode == m;
                                let btn = egui::Button::new(
                                    egui::RichText::new(m.display_name())
                                        .color(if is_act {
                                            Color32::from_rgb(10, 14, 22)
                                        } else {
                                            Color32::from_rgb(220, 235, 255)
                                        })
                                        .strong(),
                                )
                                .min_size(Vec2::new(140.0, MIN_HIT_TARGET_PT))
                                .fill(if is_act {
                                    Color32::from_rgb(0, 229, 255)
                                } else {
                                    Color32::from_rgb(30, 40, 60)
                                });

                                if ui.add(btn).clicked() {
                                    curr_slice.mode = m;
                                }
                            }

                            ui.add_space(6.0);

                            ui.label(egui::RichText::new("Pitch Transpose").strong());
                            ui.add(
                                egui::Slider::new(&mut curr_slice.pitch_semitones, -24.0..=24.0)
                                    .text("st"),
                            );

                            ui.label(egui::RichText::new("Gain").strong());
                            ui.add(
                                egui::Slider::new(&mut curr_slice.gain_db, -24.0..=12.0).text("dB"),
                            );

                            ui.label(egui::RichText::new("Pan").strong());
                            ui.add(egui::Slider::new(&mut curr_slice.pan, -1.0..=1.0));
                        });
                    });
                }
            });
        });
    }
}
