// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Multi-Touch Polyphonic Step Sequencer Matrix Grid with Per-Step Velocity & Probability (Step 1362).

use crate::layout_math::Rect;
use crate::touch_controls::{ContrastColorPalette, MIN_HIT_TARGET_PT};

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const DEFAULT_NUM_STEPS: usize = 16;
pub const CELL_PADDING_PT: f32 = 4.0;

/// Editing mode for the step sequencer matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepEditMode {
    Trigger,
    Velocity,
    Probability,
    Ratchet,
}

/// A single step trigger within a track lane.
#[derive(Debug, Clone, PartialEq)]
pub struct StepCell {
    pub active: bool,
    pub velocity: u8,     // 0 ..= 127
    pub probability: f32, // 0.0 ..= 1.0
    pub tied: bool,
    pub ratchet_count: u8, // 1, 2, 3, or 4
}

impl Default for StepCell {
    fn default() -> Self {
        Self {
            active: false,
            velocity: 100,
            probability: 1.0,
            tied: false,
            ratchet_count: 1,
        }
    }
}

impl StepCell {
    pub fn active(velocity: u8) -> Self {
        Self {
            active: true,
            velocity: velocity.clamp(1, 127),
            probability: 1.0,
            tied: false,
            ratchet_count: 1,
        }
    }
}

/// A polyphonic track lane representing an instrument or MIDI pitch.
#[derive(Debug, Clone, PartialEq)]
pub struct SequencerLane {
    pub name: String,
    pub midi_pitch: u8,
    pub color: (u8, u8, u8),
    pub mute: bool,
    pub solo: bool,
    pub steps: Vec<StepCell>,
}

impl SequencerLane {
    pub fn new(
        name: impl Into<String>,
        midi_pitch: u8,
        color: (u8, u8, u8),
        num_steps: usize,
    ) -> Self {
        Self {
            name: name.into(),
            midi_pitch,
            color,
            mute: false,
            solo: false,
            steps: vec![StepCell::default(); num_steps],
        }
    }
}

/// Multi-Touch Polyphonic Step Sequencer Matrix View (Step 1362).
#[derive(Debug, Clone)]
pub struct StepSequencerMatrixView {
    pub num_steps: usize,
    pub lanes: Vec<SequencerLane>,
    pub current_step: usize,
    pub bpm: f64,
    pub swing_pct: f32, // 0.0 ..= 100.0%
    pub edit_mode: StepEditMode,
    pub selected_step: Option<(usize, usize)>, // (lane_idx, step_idx)
    pub is_playing: bool,
    pub color_palette: ContrastColorPalette,
}

impl Default for StepSequencerMatrixView {
    fn default() -> Self {
        Self::new(DEFAULT_NUM_STEPS, 120.0)
    }
}

impl StepSequencerMatrixView {
    pub fn new(num_steps: usize, bpm: f64) -> Self {
        let n = num_steps.max(4);
        let mut view = Self {
            num_steps: n,
            lanes: Vec::new(),
            current_step: 0,
            bpm,
            swing_pct: 0.0,
            edit_mode: StepEditMode::Trigger,
            selected_step: None,
            is_playing: false,
            color_palette: ContrastColorPalette::default(),
        };

        // Standard 6-track drum kit setup
        let mut kick = SequencerLane::new("Kick", 36, (255, 107, 43), n);
        let mut snare = SequencerLane::new("Snare", 38, (0, 229, 255), n);
        let mut clap = SequencerLane::new("Clap", 39, (255, 215, 0), n);
        let mut ch_hat = SequencerLane::new("CH Hat", 42, (0, 255, 180), n);
        let mut oh_hat = SequencerLane::new("OH Hat", 46, (76, 201, 240), n);
        let perc = SequencerLane::new("Perc / Synth", 60, (180, 120, 255), n);

        // Pre-populate standard four-on-the-floor groove
        for i in (0..n).step_by(4) {
            kick.steps[i] = StepCell::active(118);
        }
        for i in [4, 12] {
            if i < n {
                snare.steps[i] = StepCell::active(110);
                clap.steps[i] = StepCell::active(95);
            }
        }
        for i in 0..n {
            if i % 2 == 0 {
                ch_hat.steps[i] = StepCell::active(85);
            }
        }
        if 2 < n {
            oh_hat.steps[2] = StepCell::active(90);
        }
        if 10 < n {
            oh_hat.steps[10] = StepCell::active(90);
        }

        view.lanes = vec![kick, snare, clap, ch_hat, oh_hat, perc];
        view
    }

    /// Advance playhead by one step.
    pub fn advance_step(&mut self) {
        self.current_step = (self.current_step + 1) % self.num_steps;
    }

    /// Toggle step active state at (lane_idx, step_idx).
    pub fn toggle_step(&mut self, lane_idx: usize, step_idx: usize) -> bool {
        if lane_idx < self.lanes.len() && step_idx < self.num_steps {
            let cell = &mut self.lanes[lane_idx].steps[step_idx];
            cell.active = !cell.active;
            self.selected_step = Some((lane_idx, step_idx));
            cell.active
        } else {
            false
        }
    }

    /// Set velocity for step at (lane_idx, step_idx).
    pub fn set_step_velocity(&mut self, lane_idx: usize, step_idx: usize, vel: u8) {
        if lane_idx < self.lanes.len() && step_idx < self.num_steps {
            self.lanes[lane_idx].steps[step_idx].velocity = vel.clamp(1, 127);
        }
    }

    /// Set probability for step at (lane_idx, step_idx).
    pub fn set_step_probability(&mut self, lane_idx: usize, step_idx: usize, prob: f32) {
        if lane_idx < self.lanes.len() && step_idx < self.num_steps {
            self.lanes[lane_idx].steps[step_idx].probability = prob.clamp(0.0, 1.0);
        }
    }

    /// Set ratchet count for step at (lane_idx, step_idx).
    pub fn set_step_ratchet(&mut self, lane_idx: usize, step_idx: usize, ratchet: u8) {
        if lane_idx < self.lanes.len() && step_idx < self.num_steps {
            self.lanes[lane_idx].steps[step_idx].ratchet_count = ratchet.clamp(1, 4);
        }
    }

    /// Calculate bounding rectangle for a given cell in logical points.
    pub fn calculate_cell_rect(
        &self,
        lane_idx: usize,
        step_idx: usize,
        grid_origin: (f32, f32),
        cell_size: (f32, f32),
    ) -> Rect {
        let x = grid_origin.0 + step_idx as f32 * (cell_size.0 + CELL_PADDING_PT);
        let y = grid_origin.1 + lane_idx as f32 * (cell_size.1 + CELL_PADDING_PT);
        Rect::new(x, y, cell_size.0, cell_size.1)
    }

    /// Hit test coordinate to determine (lane_idx, step_idx).
    pub fn hit_test_cell(
        &self,
        pos: (f32, f32),
        grid_origin: (f32, f32),
        cell_size: (f32, f32),
    ) -> Option<(usize, usize)> {
        let step_stride = cell_size.0 + CELL_PADDING_PT;
        let lane_stride = cell_size.1 + CELL_PADDING_PT;

        let rel_x = pos.0 - grid_origin.0;
        let rel_y = pos.1 - grid_origin.1;

        if rel_x < 0.0 || rel_y < 0.0 {
            return None;
        }

        let step_idx = (rel_x / step_stride) as usize;
        let lane_idx = (rel_y / lane_stride) as usize;

        if lane_idx < self.lanes.len() && step_idx < self.num_steps {
            // Check within cell bounds without trailing padding
            let cell_x = grid_origin.0 + step_idx as f32 * step_stride;
            let cell_y = grid_origin.1 + lane_idx as f32 * lane_stride;
            if pos.0 <= cell_x + cell_size.0 && pos.1 <= cell_y + cell_size.1 {
                return Some((lane_idx, step_idx));
            }
        }
        None
    }

    /// Generate deterministic ASCII representation of the sequencer matrix.
    pub fn render_ascii(&self) -> String {
        let mut out = String::new();
        for lane in &self.lanes {
            out.push_str(&format!("{:10} |", lane.name));
            for s_idx in 0..self.num_steps {
                let cell = &lane.steps[s_idx];
                let ch = if s_idx == self.current_step {
                    if cell.active {
                        '#'
                    } else {
                        'v'
                    }
                } else if cell.active {
                    'X'
                } else {
                    '.'
                };
                out.push(ch);
                if (s_idx + 1) % 4 == 0 && s_idx + 1 < self.num_steps {
                    out.push('|');
                }
            }
            out.push_str("|\n");
        }
        out
    }
}

#[cfg(feature = "gui")]
impl StepSequencerMatrixView {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // 1. Toolbar & Mode Selectors
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("POLYPHONIC STEP SEQUENCER MATRIX")
                        .color(Color32::from_rgb(240, 245, 255))
                        .strong(),
                );
                ui.separator();

                // Play / Pause button
                let play_txt = if self.is_playing { "PAUSE" } else { "PLAY" };
                if ui.button(play_txt).clicked() {
                    self.is_playing = !self.is_playing;
                }

                ui.separator();
                ui.label(
                    egui::RichText::new(format!(
                        "BPM: {:.0}  Step: {}/{}",
                        self.bpm,
                        self.current_step + 1,
                        self.num_steps
                    ))
                    .color(Color32::from_rgb(0, 229, 255)),
                );
            });

            // Mode Selector Buttons (>=44pt Touch Targets)
            ui.horizontal(|ui| {
                let modes = [
                    (StepEditMode::Trigger, "1: Trigger (Toggle)"),
                    (StepEditMode::Velocity, "2: Velocity (Vel)"),
                    (StepEditMode::Probability, "3: Probability (Prob)"),
                    (StepEditMode::Ratchet, "4: Ratchet (Burst)"),
                ];
                for (mode, label) in modes {
                    let is_active = self.edit_mode == mode;
                    let btn = egui::Button::new(
                        egui::RichText::new(label)
                            .color(if is_active {
                                Color32::from_rgb(10, 14, 22)
                            } else {
                                Color32::from_rgb(220, 235, 255)
                            })
                            .strong(),
                    )
                    .min_size(Vec2::new(120.0_f32, MIN_HIT_TARGET_PT))
                    .fill(if is_active {
                        Color32::from_rgb(0, 229, 255)
                    } else {
                        Color32::from_rgb(30, 40, 60)
                    });

                    if ui.add(btn).clicked() {
                        self.edit_mode = mode;
                    }
                }
            });

            ui.add_space(8.0_f32);

            // 2. Matrix Grid Painter
            let header_w = 110.0_f32;
            let cell_w = MIN_HIT_TARGET_PT;
            let cell_h = MIN_HIT_TARGET_PT;
            let total_grid_w = header_w + self.num_steps as f32 * (cell_w + CELL_PADDING_PT);
            let total_grid_h = self.lanes.len() as f32 * (cell_h + CELL_PADDING_PT);

            let (response, painter) = ui.allocate_painter(
                Vec2::new(total_grid_w, total_grid_h),
                egui::Sense::click_and_drag(),
            );

            let grid_origin = (response.rect.min.x + header_w, response.rect.min.y);

            // Handle touch clicks
            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if let Some((lane_idx, step_idx)) =
                        self.hit_test_cell((pos.x, pos.y), grid_origin, (cell_w, cell_h))
                    {
                        match self.edit_mode {
                            StepEditMode::Trigger => {
                                self.toggle_step(lane_idx, step_idx);
                            }
                            _ => {
                                self.selected_step = Some((lane_idx, step_idx));
                            }
                        }
                    }
                }
            }

            // Draw Lanes and Matrix Cells
            for (lane_idx, lane) in self.lanes.iter().enumerate() {
                let lane_y = response.rect.min.y + lane_idx as f32 * (cell_h + CELL_PADDING_PT);
                let lane_color = Color32::from_rgb(lane.color.0, lane.color.1, lane.color.2);

                // Draw Lane Header Box
                let header_rect = egui::Rect::from_min_size(
                    egui::pos2(response.rect.min.x, lane_y),
                    Vec2::new(header_w - 6.0_f32, cell_h),
                );
                painter.rect_filled(header_rect, 4.0_f32, Color32::from_rgb(20, 26, 38));
                painter.rect_stroke(
                    header_rect,
                    4.0_f32,
                    Stroke::new(1.0_f32, Color32::from_rgb(45, 55, 75)),
                );

                painter.text(
                    egui::pos2(header_rect.min.x + 8.0_f32, header_rect.min.y + 14.0_f32),
                    egui::Align2::LEFT_CENTER,
                    &lane.name,
                    egui::FontId::proportional(13.0_f32),
                    lane_color,
                );

                // Draw Steps
                for step_idx in 0..self.num_steps {
                    let cell = &lane.steps[step_idx];
                    let cell_rect_math =
                        self.calculate_cell_rect(lane_idx, step_idx, grid_origin, (cell_w, cell_h));
                    let cell_rect = egui::Rect::from_min_size(
                        egui::pos2(cell_rect_math.x, cell_rect_math.y),
                        Vec2::new(cell_w, cell_h),
                    );

                    let is_current_playhead = step_idx == self.current_step;
                    let is_selected = self.selected_step == Some((lane_idx, step_idx));
                    let is_quarter_downbeat = step_idx % 4 == 0;

                    let bg_color = if cell.active {
                        lane_color
                    } else if is_quarter_downbeat {
                        Color32::from_rgb(28, 36, 52)
                    } else {
                        Color32::from_rgb(18, 24, 34)
                    };

                    painter.rect_filled(cell_rect, 4.0_f32, bg_color);

                    // Border / Playhead stroke
                    let stroke_col = if is_current_playhead {
                        Color32::from_rgb(255, 255, 255)
                    } else if is_selected {
                        Color32::from_rgb(0, 229, 255)
                    } else {
                        Color32::from_rgb(45, 60, 85)
                    };
                    let stroke_w = if is_current_playhead || is_selected {
                        2.5_f32
                    } else {
                        1.0_f32
                    };
                    painter.rect_stroke(cell_rect, 4.0_f32, Stroke::new(stroke_w, stroke_col));

                    // Text / Mode Display overlay inside cell
                    if cell.active {
                        let inner_txt = match self.edit_mode {
                            StepEditMode::Velocity => format!("{}", cell.velocity),
                            StepEditMode::Probability => {
                                format!("{:.0}%", cell.probability * 100.0_f32)
                            }
                            StepEditMode::Ratchet => format!("{}x", cell.ratchet_count),
                            StepEditMode::Trigger => {
                                if cell.ratchet_count > 1 {
                                    format!("{}x", cell.ratchet_count)
                                } else {
                                    String::new()
                                }
                            }
                        };
                        if !inner_txt.is_empty() {
                            painter.text(
                                cell_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                inner_txt,
                                egui::FontId::proportional(11.0_f32),
                                Color32::from_rgb(10, 14, 20),
                            );
                        }
                    }
                }
            }

            ui.add_space(10.0_f32);

            // 3. Step Detail Inspector Panel
            if let Some((lane_idx, step_idx)) = self.selected_step {
                if lane_idx < self.lanes.len() && step_idx < self.num_steps {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            let lane_name = self.lanes[lane_idx].name.clone();
                            let cell = &mut self.lanes[lane_idx].steps[step_idx];

                            ui.label(
                                egui::RichText::new(format!(
                                    "EDIT STEP: {} | Step {}",
                                    lane_name,
                                    step_idx + 1
                                ))
                                .color(Color32::from_rgb(255, 215, 0))
                                .strong(),
                            );

                            ui.separator();
                            ui.checkbox(&mut cell.active, "Active");

                            ui.separator();
                            let mut vel = cell.velocity as i32;
                            ui.label("Velocity:");
                            if ui
                                .add(egui::Slider::new(&mut vel, 1..=127).text(""))
                                .changed()
                            {
                                cell.velocity = vel as u8;
                            }

                            ui.separator();
                            let mut prob_pct = (cell.probability * 100.0_f32) as i32;
                            ui.label("Probability:");
                            if ui
                                .add(egui::Slider::new(&mut prob_pct, 0..=100).text("%"))
                                .changed()
                            {
                                cell.probability = prob_pct as f32 / 100.0_f32;
                            }

                            ui.separator();
                            let mut ratchet = cell.ratchet_count as i32;
                            ui.label("Ratchet:");
                            if ui
                                .add(egui::Slider::new(&mut ratchet, 1..=4).text("x"))
                                .changed()
                            {
                                cell.ratchet_count = ratchet as u8;
                            }
                        });
                    });
                }
            }
        });
    }
}
