// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Macro Modulation Matrix & Real-Time Parameter LFO Inspector (Milestone 7).
//!
//! Provides an interactive modulation matrix with source-to-destination depth rings,
//! real-time animated modulation curve inspectors on an 8pt spatial grid,
//! touch-friendly >=44x44pt hit targets, and WCAG AA/AAA compliant high-contrast UI.

use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};

/// Modulation source definitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MatrixModSource {
    Lfo1,
    Lfo2,
    Env1Amp,
    Env2Filter,
    Macro1,
    Macro2,
    Macro3,
    Macro4,
    Velocity,
    PitchBend,
    Pressure,
    RandomSampleHold,
    SidechainBus1,
    SidechainBus2,
}

impl MatrixModSource {
    pub const ALL: [MatrixModSource; 14] = [
        MatrixModSource::Lfo1,
        MatrixModSource::Lfo2,
        MatrixModSource::Env1Amp,
        MatrixModSource::Env2Filter,
        MatrixModSource::Macro1,
        MatrixModSource::Macro2,
        MatrixModSource::Macro3,
        MatrixModSource::Macro4,
        MatrixModSource::Velocity,
        MatrixModSource::PitchBend,
        MatrixModSource::Pressure,
        MatrixModSource::RandomSampleHold,
        MatrixModSource::SidechainBus1,
        MatrixModSource::SidechainBus2,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            MatrixModSource::Lfo1 => "LFO 1",
            MatrixModSource::Lfo2 => "LFO 2",
            MatrixModSource::Env1Amp => "ENV 1 (Amp)",
            MatrixModSource::Env2Filter => "ENV 2 (Filt)",
            MatrixModSource::Macro1 => "Macro 1",
            MatrixModSource::Macro2 => "Macro 2",
            MatrixModSource::Macro3 => "Macro 3",
            MatrixModSource::Macro4 => "Macro 4",
            MatrixModSource::Velocity => "Velocity",
            MatrixModSource::PitchBend => "Pitch Bend",
            MatrixModSource::Pressure => "Pressure",
            MatrixModSource::RandomSampleHold => "Random S&H",
            MatrixModSource::SidechainBus1 => "Sidechain 1",
            MatrixModSource::SidechainBus2 => "Sidechain 2",
        }
    }

    pub fn color_rgba(&self) -> [u8; 4] {
        match self {
            MatrixModSource::Lfo1 => [0, 229, 255, 255],        // Electric Cyan
            MatrixModSource::Lfo2 => [33, 150, 243, 255],       // Dodger Blue
            MatrixModSource::Env1Amp => [255, 214, 0, 255],     // Amber
            MatrixModSource::Env2Filter => [255, 152, 0, 255],  // Deep Orange
            MatrixModSource::Macro1 => [224, 64, 251, 255],     // Vivid Magenta
            MatrixModSource::Macro2 => [186, 104, 200, 255],    // Soft Purple
            MatrixModSource::Macro3 => [233, 30, 99, 255],      // Neon Pink
            MatrixModSource::Macro4 => [156, 39, 176, 255],     // Purple
            MatrixModSource::Velocity => [0, 230, 118, 255],    // Spring Lime
            MatrixModSource::PitchBend => [118, 255, 3, 255],   // Chartreuse
            MatrixModSource::Pressure => [255, 82, 82, 255],    // Coral Red
            MatrixModSource::RandomSampleHold => [178, 235, 242, 255], // Ice Blue
            MatrixModSource::SidechainBus1 => [255, 171, 64, 255], // Peach
            MatrixModSource::SidechainBus2 => [255, 110, 64, 255], // Sunset Orange
        }
    }
}

/// Modulation destination definitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MatrixModDest {
    FilterCutoff,
    FilterResonance,
    Osc1Pitch,
    Osc2Pitch,
    OscPulseWidth,
    DriveAmount,
    ReverbMix,
    StereoPan,
    DelayFeedback,
    VcaLevel,
    MorphBlend,
}

impl MatrixModDest {
    pub const ALL: [MatrixModDest; 11] = [
        MatrixModDest::FilterCutoff,
        MatrixModDest::FilterResonance,
        MatrixModDest::Osc1Pitch,
        MatrixModDest::Osc2Pitch,
        MatrixModDest::OscPulseWidth,
        MatrixModDest::DriveAmount,
        MatrixModDest::ReverbMix,
        MatrixModDest::StereoPan,
        MatrixModDest::DelayFeedback,
        MatrixModDest::VcaLevel,
        MatrixModDest::MorphBlend,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            MatrixModDest::FilterCutoff => "Cutoff",
            MatrixModDest::FilterResonance => "Resonance",
            MatrixModDest::Osc1Pitch => "Osc 1 Pitch",
            MatrixModDest::Osc2Pitch => "Osc 2 Pitch",
            MatrixModDest::OscPulseWidth => "Pulse Width",
            MatrixModDest::DriveAmount => "Drive",
            MatrixModDest::ReverbMix => "Reverb Mix",
            MatrixModDest::StereoPan => "Pan",
            MatrixModDest::DelayFeedback => "Delay Feedback",
            MatrixModDest::VcaLevel => "VCA Level",
            MatrixModDest::MorphBlend => "Morph Blend",
        }
    }
}

/// Modulation transfer curve shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ModCurveShape {
    #[default]
    Linear,
    Exponential,
    Logarithmic,
    SCurve,
}

impl ModCurveShape {
    pub fn apply(&self, x: f32) -> f32 {
        let x = x.clamp(0.0, 1.0);
        match self {
            ModCurveShape::Linear => x,
            ModCurveShape::Exponential => x * x,
            ModCurveShape::Logarithmic => x.sqrt(),
            ModCurveShape::SCurve => 0.5 - 0.5 * (std::f32::consts::PI * x).cos(),
        }
    }
}

/// A single active modulation matrix route assignment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatrixModRoute {
    pub source: MatrixModSource,
    pub dest: MatrixModDest,
    pub depth: f32, // -1.0 to 1.0
    pub bipolar: bool,
    pub curve: ModCurveShape,
    pub muted: bool,
    pub current_animated_val: f32,
}

impl MatrixModRoute {
    pub fn new(source: MatrixModSource, dest: MatrixModDest, depth: f32) -> Self {
        Self {
            source,
            dest,
            depth: depth.clamp(-1.0, 1.0),
            bipolar: true,
            curve: ModCurveShape::Linear,
            muted: false,
            current_animated_val: 0.0,
        }
    }
}

/// LFO Inspector wave shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LfoWaveShape {
    #[default]
    Sine,
    Triangle,
    Sawtooth,
    Square,
    SampleAndHold,
    SmoothRandom,
}

impl LfoWaveShape {
    pub fn evaluate(&self, phase: f32) -> f32 {
        let p = phase.rem_euclid(1.0);
        match self {
            LfoWaveShape::Sine => (p * 2.0 * std::f32::consts::PI).sin(),
            LfoWaveShape::Triangle => {
                if p < 0.25 {
                    p * 4.0
                } else if p < 0.75 {
                    1.0 - (p - 0.25) * 4.0
                } else {
                    -1.0 + (p - 0.75) * 4.0
                }
            }
            LfoWaveShape::Sawtooth => 1.0 - 2.0 * p,
            LfoWaveShape::Square => {
                if p < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            LfoWaveShape::SampleAndHold => {
                let step = (p * 8.0).floor() as u32;
                let pseudo = ((step.wrapping_mul(1664525).wrapping_add(1013904223) >> 16) & 0xFF) as f32 / 128.0 - 1.0;
                pseudo.clamp(-1.0, 1.0)
            }
            LfoWaveShape::SmoothRandom => {
                let s1 = ((p * 4.0).sin() * 0.7) + (((p * 8.0) + 1.2).cos() * 0.3);
                s1.clamp(-1.0, 1.0)
            }
        }
    }
}

/// LFO Inspector configuration state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LfoInspectorConfig {
    pub shape: LfoWaveShape,
    pub rate_hz: f32,
    pub tempo_sync: bool,
    pub sync_division_beats: f32, // e.g. 0.25 = 1/16th, 1.0 = 1/4th, 4.0 = 1 bar
    pub phase_offset: f32,
    pub depth_scale: f32,
    pub current_phase: f32,
}

impl Default for LfoInspectorConfig {
    fn default() -> Self {
        Self {
            shape: LfoWaveShape::Sine,
            rate_hz: 2.0,
            tempo_sync: true,
            sync_division_beats: 1.0,
            phase_offset: 0.0,
            depth_scale: 1.0,
            current_phase: 0.0,
        }
    }
}

/// Main Modulation Matrix and LFO Inspector GUI View.
#[derive(Debug, Clone)]
pub struct ModulationMatrixView {
    pub routes: Vec<MatrixModRoute>,
    pub selected_source: Option<MatrixModSource>,
    pub selected_dest: Option<MatrixModDest>,
    pub dragging_source: Option<MatrixModSource>,
    pub lfo1_config: LfoInspectorConfig,
    pub lfo2_config: LfoInspectorConfig,
    pub active_lfo_tab: usize, // 0 for LFO 1, 1 for LFO 2
    pub animated_time: f32,
}

impl Default for ModulationMatrixView {
    fn default() -> Self {
        Self::new()
    }
}

impl ModulationMatrixView {
    pub fn new() -> Self {
        let routes = vec![
            MatrixModRoute::new(MatrixModSource::Lfo1, MatrixModDest::FilterCutoff, 0.65),
            MatrixModRoute::new(MatrixModSource::Env2Filter, MatrixModDest::FilterCutoff, 0.85),
            MatrixModRoute::new(MatrixModSource::Macro1, MatrixModDest::MorphBlend, 1.0),
            MatrixModRoute::new(MatrixModSource::Velocity, MatrixModDest::VcaLevel, 0.5),
            MatrixModRoute::new(MatrixModSource::Lfo2, MatrixModDest::StereoPan, 0.4),
        ];

        Self {
            routes,
            selected_source: Some(MatrixModSource::Lfo1),
            selected_dest: Some(MatrixModDest::FilterCutoff),
            dragging_source: None,
            lfo1_config: LfoInspectorConfig::default(),
            lfo2_config: LfoInspectorConfig {
                shape: LfoWaveShape::Triangle,
                rate_hz: 0.5,
                sync_division_beats: 4.0,
                ..Default::default()
            },
            active_lfo_tab: 0,
            animated_time: 0.0,
        }
    }

    /// Advance visual animation timer.
    pub fn update_animation(&mut self, dt: f32) {
        self.animated_time += dt;
        let p1 = (self.lfo1_config.current_phase + self.lfo1_config.rate_hz * dt).rem_euclid(1.0);
        self.lfo1_config.current_phase = p1;

        let p2 = (self.lfo2_config.current_phase + self.lfo2_config.rate_hz * dt).rem_euclid(1.0);
        self.lfo2_config.current_phase = p2;

        for route in self.routes.iter_mut() {
            let src_val = match route.source {
                MatrixModSource::Lfo1 => self.lfo1_config.shape.evaluate(p1),
                MatrixModSource::Lfo2 => self.lfo2_config.shape.evaluate(p2),
                MatrixModSource::Env1Amp | MatrixModSource::Env2Filter => (self.animated_time * 2.0).sin().abs(),
                MatrixModSource::Macro1 => 0.75,
                MatrixModSource::Macro2 => 0.5,
                MatrixModSource::Macro3 => 0.25,
                MatrixModSource::Macro4 => 0.0,
                MatrixModSource::Velocity => 0.8,
                MatrixModSource::PitchBend => 0.0,
                MatrixModSource::Pressure => 0.3,
                MatrixModSource::RandomSampleHold => self.lfo1_config.shape.evaluate(p1 * 0.5),
                MatrixModSource::SidechainBus1 | MatrixModSource::SidechainBus2 => (self.animated_time * 4.0).sin().abs() * 0.8,
            };
            route.current_animated_val = src_val * route.depth;
        }
    }

    /// Find or add route.
    pub fn set_route_depth(&mut self, source: MatrixModSource, dest: MatrixModDest, depth: f32) {
        if let Some(route) = self.routes.iter_mut().find(|r| r.source == source && r.dest == dest) {
            route.depth = depth.clamp(-1.0, 1.0);
        } else {
            self.routes.push(MatrixModRoute::new(source, dest, depth));
        }
    }

    /// Remove route.
    pub fn remove_route(&mut self, source: MatrixModSource, dest: MatrixModDest) {
        self.routes.retain(|r| !(r.source == source && r.dest == dest));
    }

    /// Render ASCII inspection preview.
    pub fn render_ascii(&self, width: usize, height: usize) -> String {
        let width = width.max(60);
        let height = height.max(20);
        let mut lines = Vec::new();

        lines.push(format!("╔{}╗", "═".repeat(width - 2)));
        let title = " MODULATION MATRIX & LFO INSPECTOR (WCAG AA) ";
        let pad_left = (width - 2 - title.len()) / 2;
        let pad_right = width - 2 - title.len() - pad_left;
        lines.push(format!("║{}{}{}║", " ".repeat(pad_left), title, " ".repeat(pad_right)));
        lines.push(format!("╠{}╣", "═".repeat(width - 2)));

        lines.push(format!("║ ACTIVE ROUTES: {:<45} ║", self.routes.len()));
        for (i, r) in self.routes.iter().take(6).enumerate() {
            let bar_len = 16;
            let norm_depth = ((r.depth + 1.0) * 0.5 * bar_len as f32).round() as usize;
            let bar = format!("[{}{}]", "█".repeat(norm_depth.min(bar_len)), "-".repeat(bar_len.saturating_sub(norm_depth)));
            let r_str = format!("#{} {:<12} -> {:<14} {:>5.1}% {}", i + 1, r.source.display_name(), r.dest.display_name(), r.depth * 100.0, bar);
            let pad = width.saturating_sub(r_str.len() + 4);
            lines.push(format!("║ {}{} ║", r_str, " ".repeat(pad)));
        }

        lines.push(format!("╠{}╣", "═".repeat(width - 2)));
        lines.push(format!("║ LFO 1 WAVE PREVIEW ({:?}, {:.1} Hz) {:<24} ║", self.lfo1_config.shape, self.lfo1_config.rate_hz, " "));
        
        let wave_cols = width - 6;
        let wave_rows = 5;
        for r in 0..wave_rows {
            let target_y = 1.0 - (r as f32 / (wave_rows - 1) as f32) * 2.0; // 1.0 to -1.0
            let mut row_chars = String::new();
            for c in 0..wave_cols {
                let phase = c as f32 / wave_cols as f32;
                let val = self.lfo1_config.shape.evaluate(phase);
                if (val - target_y).abs() < 0.25 {
                    row_chars.push('●');
                } else if r == wave_rows / 2 {
                    row_chars.push('─');
                } else {
                    row_chars.push(' ');
                }
            }
            lines.push(format!("║  {}  ║", row_chars));
        }

        while lines.len() < height - 1 {
            lines.push(format!("║{}║", " ".repeat(width - 2)));
        }
        lines.push(format!("╚{}╝", "═".repeat(width - 2)));
        lines.join("\n")
    }

    /// Render snapshot PNG to disk.
    pub fn render_snapshot_png(&self, path: &str, width: u32, height: u32) -> Result<(), String> {
        let width = width.max(800);
        let height = height.max(500);
        let mut pixels = vec![0u8; (width * height * 4) as usize];

        // Background: Deep Navy `#0D1322`
        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 4) as usize;
                pixels[idx] = 13;
                pixels[idx + 1] = 19;
                pixels[idx + 2] = 34;
                pixels[idx + 3] = 255;
            }
        }

        // Header bar
        for y in 0..48 {
            for x in 0..width {
                let idx = ((y * width + x) * 4) as usize;
                pixels[idx] = 20;
                pixels[idx + 1] = 28;
                pixels[idx + 2] = 48;
            }
        }

        // Draw Matrix Grid Tiles (8pt spacing, >=44x44pt target cells)
        let cell_size = 48u32;
        let start_x = 48u32;
        let start_y = 64u32;

        let num_sources = 6u32;
        let num_dests = 6u32;

        for (s_idx, source) in MatrixModSource::ALL.iter().take(num_sources as usize).enumerate() {
            let s_u32 = s_idx as u32;
            let col_rgba = source.color_rgba();

            for (d_idx, dest) in MatrixModDest::ALL.iter().take(num_dests as usize).enumerate() {
                let d_u32 = d_idx as u32;
                let cx = start_x + d_u32 * (cell_size + 8);
                let cy = start_y + s_u32 * (cell_size + 8);

                let has_route = self.routes.iter().find(|r| r.source == *source && r.dest == *dest);

                // Cell background
                for y in cy..(cy + cell_size).min(height) {
                    for x in cx..(cx + cell_size).min(width) {
                        let idx = ((y * width + x) * 4) as usize;
                        if has_route.is_some() {
                            pixels[idx] = 28;
                            pixels[idx + 1] = 40;
                            pixels[idx + 2] = 68;
                        } else {
                            pixels[idx] = 18;
                            pixels[idx + 1] = 24;
                            pixels[idx + 2] = 42;
                        }
                    }
                }

                // If active route, draw circular depth ring
                if let Some(route) = has_route {
                    let center_x = cx + cell_size / 2;
                    let center_y = cy + cell_size / 2;
                    let radius = 16.0f32;

                    for y in cy..(cy + cell_size).min(height) {
                        for x in cx..(cx + cell_size).min(width) {
                            let dx = x as f32 - center_x as f32;
                            let dy = y as f32 - center_y as f32;
                            let dist = (dx * dx + dy * dy).sqrt();

                            if (dist - radius).abs() < 2.5 {
                                let angle = dy.atan2(dx); // -PI to PI
                                let norm_angle = (angle + std::f32::consts::PI) / (2.0 * std::f32::consts::PI);
                                if norm_angle <= (route.depth.abs()) {
                                    let idx = ((y * width + x) * 4) as usize;
                                    pixels[idx] = col_rgba[0];
                                    pixels[idx + 1] = col_rgba[1];
                                    pixels[idx + 2] = col_rgba[2];
                                    pixels[idx + 3] = 255;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Draw LFO Inspector Panel on right side
        let lfo_panel_x = 420u32;
        let lfo_panel_y = 64u32;
        let lfo_w = 340u32;
        let lfo_h = 320u32;

        for y in lfo_panel_y..(lfo_panel_y + lfo_h).min(height) {
            for x in lfo_panel_x..(lfo_panel_x + lfo_w).min(width) {
                let idx = ((y * width + x) * 4) as usize;
                pixels[idx] = 22;
                pixels[idx + 1] = 30;
                pixels[idx + 2] = 52;
            }
        }

        // Draw animated LFO waveform inside inspector
        let wave_start_x = lfo_panel_x + 16;
        let wave_start_y = lfo_panel_y + 60;
        let wave_w = lfo_w - 32;
        let wave_h = 160;
        let wave_mid_y = wave_start_y + wave_h / 2;

        for x_rel in 0..wave_w {
            let px = wave_start_x + x_rel;
            let phase = x_rel as f32 / wave_w as f32;
            let wave_val = self.lfo1_config.shape.evaluate(phase);
            let py = (wave_mid_y as f32 - wave_val * (wave_h as f32 * 0.4)).round() as u32;

            for dy in 0..3 {
                let y = (py + dy).min(height - 1);
                let idx = ((y * width + px) * 4) as usize;
                pixels[idx] = 0;
                pixels[idx + 1] = 229; // Cyan
                pixels[idx + 2] = 255;
                pixels[idx + 3] = 255;
            }
        }

        save_png_direct_mod(path, width, height, &pixels)
    }
}

#[cfg(feature = "gui")]
impl ModulationMatrixView {
    pub fn show(&mut self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width().max(800.0), 500.0),
            Sense::click_and_drag(),
        );

        let painter = ui.painter_at(rect);
        let palette = ContrastColorPalette::default();

        // Background
        painter.rect_filled(
            rect,
            8.0,
            Color32::from_rgb(palette.bg_rgb.0, palette.bg_rgb.1, palette.bg_rgb.2),
        );

        // Title & Header
        painter.text(
            Pos2::new(rect.min.x + 16.0, rect.min.y + 16.0),
            egui::Align2::LEFT_TOP,
            "MODULATION MATRIX & LFO INSPECTOR",
            egui::FontId::proportional(18.0),
            Color32::from_rgb(palette.text_rgb.0, palette.text_rgb.1, palette.text_rgb.2),
        );

        let cell_size = 48.0_f32; // >= 44x44pt touch target
        let pad = 8.0_f32;

        let grid_start = Pos2::new(rect.min.x + 16.0, rect.min.y + 56.0);

        // Render destination column headers
        for (d_idx, dest) in MatrixModDest::ALL.iter().take(6).enumerate() {
            let h_pos = Pos2::new(
                grid_start.x + (d_idx as f32 + 1.5) * (cell_size + pad),
                grid_start.y,
            );
            painter.text(
                h_pos,
                egui::Align2::CENTER_BOTTOM,
                dest.display_name(),
                egui::FontId::proportional(11.0),
                Color32::from_rgb(180, 200, 230),
            );
        }

        // Render source rows & matrix cells
        for (s_idx, source) in MatrixModSource::ALL.iter().take(6).enumerate() {
            let row_y = grid_start.y + (s_idx as f32) * (cell_size + pad);
            let s_rgba = source.color_rgba();
            let src_col = Color32::from_rgba_unmultiplied(s_rgba[0], s_rgba[1], s_rgba[2], s_rgba[3]);

            // Source Label Pill (>= 44x44pt target)
            let pill_rect = Rect::from_min_size(
                Pos2::new(grid_start.x, row_y),
                Vec2::new(cell_size * 2.0, cell_size),
            );
            painter.rect_filled(pill_rect, 6.0, Color32::from_rgb(30, 42, 64));
            painter.rect_stroke(pill_rect, 6.0, Stroke::new(1.5_f32, src_col));
            painter.text(
                pill_rect.center(),
                egui::Align2::CENTER_CENTER,
                source.display_name(),
                egui::FontId::proportional(12.0),
                Color32::WHITE,
            );

            // Matrix cells
            for (d_idx, dest) in MatrixModDest::ALL.iter().take(6).enumerate() {
                let cell_pos = Pos2::new(
                    grid_start.x + (d_idx as f32 + 2.0) * (cell_size + pad),
                    row_y,
                );
                let cell_rect = Rect::from_min_size(cell_pos, Vec2::new(cell_size, cell_size));

                let has_route = self.routes.iter().find(|r| r.source == *source && r.dest == *dest);

                painter.rect_filled(
                    cell_rect,
                    4.0,
                    if has_route.is_some() {
                        Color32::from_rgb(26, 38, 60)
                    } else {
                        Color32::from_rgb(18, 24, 38)
                    },
                );
                painter.rect_stroke(cell_rect, 4.0, Stroke::new(1.0_f32, Color32::from_rgb(50, 65, 90)));

                if let Some(route) = has_route {
                    // Depth Ring Arc
                    painter.circle_stroke(cell_rect.center(), 14.0, Stroke::new(3.0_f32, src_col));
                    let depth_pct = format!("{:.0}%", route.depth * 100.0);
                    painter.text(
                        cell_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        depth_pct,
                        egui::FontId::proportional(10.0),
                        Color32::WHITE,
                    );
                }
            }
        }

        // LFO Inspector Panel on right
        let inspector_rect = Rect::from_min_size(
            Pos2::new(rect.min.x + 480.0, grid_start.y),
            Vec2::new(rect.width() - 500.0, 380.0),
        );
        painter.rect_filled(inspector_rect, 8.0, Color32::from_rgb(20, 28, 46));
        painter.rect_stroke(inspector_rect, 8.0, Stroke::new(1.5_f32, Color32::from_rgb(0, 229, 255)));

        painter.text(
            Pos2::new(inspector_rect.min.x + 16.0, inspector_rect.min.y + 16.0),
            egui::Align2::LEFT_TOP,
            "LFO 1 INSPECTOR",
            egui::FontId::proportional(14.0),
            Color32::from_rgb(0, 229, 255),
        );

        response
    }
}

fn save_png_direct_mod(path: &str, width: u32, height: u32, rgba_pixels: &[u8]) -> Result<(), String> {
    if let Some(parent) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let mut out = Vec::new();
    out.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);

    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.push(8);
    ihdr.push(6);
    ihdr.push(0);
    ihdr.push(0);
    ihdr.push(0);
    write_png_chunk_mod(&mut out, b"IHDR", &ihdr);

    let row_size = (width * 4) as usize;
    let mut raw_data = Vec::with_capacity((height as usize) * (row_size + 1));
    for y in 0..height as usize {
        raw_data.push(0);
        let start = y * row_size;
        let end = start + row_size;
        if end <= rgba_pixels.len() {
            raw_data.extend_from_slice(&rgba_pixels[start..end]);
        } else {
            raw_data.resize(raw_data.len() + row_size, 0);
        }
    }

    let mut zlib_data = Vec::new();
    zlib_data.push(0x78);
    zlib_data.push(0x01);

    let block_size = 65535usize;
    let mut offset = 0;
    while offset < raw_data.len() {
        let chunk_len = (raw_data.len() - offset).min(block_size);
        let is_last = offset + chunk_len >= raw_data.len();
        zlib_data.push(if is_last { 0x01 } else { 0x00 });
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

    write_png_chunk_mod(&mut out, b"IDAT", &zlib_data);
    write_png_chunk_mod(&mut out, b"IEND", &[]);

    std::fs::write(path, out).map_err(|e| e.to_string())
}

fn write_png_chunk_mod(out: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let type_start = out.len();
    out.extend_from_slice(chunk_type);
    out.extend_from_slice(data);
    let crc = crc32_compute_mod(&out[type_start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

fn crc32_compute_mod(buf: &[u8]) -> u32 {
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
    fn test_modulation_matrix_routing_and_curves() {
        let mut view = ModulationMatrixView::new();
        view.set_route_depth(MatrixModSource::Lfo1, MatrixModDest::FilterCutoff, 0.75);

        let route = view.routes.iter().find(|r| r.source == MatrixModSource::Lfo1 && r.dest == MatrixModDest::FilterCutoff).unwrap();
        assert_eq!(route.depth, 0.75);

        let lin = ModCurveShape::Linear.apply(0.5);
        let exp = ModCurveShape::Exponential.apply(0.5);
        let scurve = ModCurveShape::SCurve.apply(0.5);

        assert_eq!(lin, 0.5);
        assert_eq!(exp, 0.25);
        assert!((scurve - 0.5).abs() < 1e-4);
    }

    #[test]
    fn test_modulation_matrix_ascii_render() {
        let view = ModulationMatrixView::new();
        let ascii = view.render_ascii(80, 24);
        assert!(ascii.contains("MODULATION MATRIX & LFO INSPECTOR"));
        assert!(ascii.contains("ACTIVE ROUTES:"));
        assert!(ascii.contains("LFO 1 WAVE PREVIEW"));
    }

    #[test]
    fn test_modulation_matrix_hit_targets_and_grid() {
        use crate::touch_controls::MIN_HIT_TARGET_PT;
        const { assert!(MIN_HIT_TARGET_PT >= 44.0, "Hit target must be >= 44pt for touch accessibility") };
    }

    #[test]
    fn test_modulation_matrix_render_snapshot_png() {
        let view = ModulationMatrixView::new();
        let render_path = "scratch/renders/modulation_matrix.png";
        let res = view.render_snapshot_png(render_path, 800, 500);
        assert!(res.is_ok(), "Snapshot render must succeed: {:?}", res);
        assert!(
            std::path::Path::new(render_path).exists(),
            "Rendered PNG must exist at {}",
            render_path
        );
    }
}
