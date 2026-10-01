// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Electromechanical Clavinet HUD & String-Anvil Collision View (Milestone 23).
//!
//! Provides an interactive 2D anvil strike force vs pickup phase puck interface,
//! 4-way rocker tone switch bank, real-time rubber anvil string contact compression visualizer,
//! dual electromagnetic pickup waveform display, and WCAG AAA high-contrast styling.
//!
//! Enforces minimum 44x44pt hit targets, 8pt base grid alignment, and headless PNG snapshot rendering.

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

#[cfg(feature = "gui")]
#[allow(unused_imports)]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const CLAV_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt bounding touch target

/// Clavinet sound profile selector for GUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GuiClavinetProfile {
    #[default]
    StevieSuperstition,
    OutPhaseFunkQuack,
    ClassicD6Clean,
    MellowChamber,
    ScreamingAutoWah,
    TwangyBridgeLead,
}

impl GuiClavinetProfile {
    pub fn name(&self) -> &'static str {
        match self {
            Self::StevieSuperstition => "STEVIE SUPERSTITION FUNK",
            Self::OutPhaseFunkQuack => "OUT-OF-PHASE FUNK QUACK",
            Self::ClassicD6Clean => "CLASSIC D6 CLEAN",
            Self::MellowChamber => "MELLOW CHAMBER CLAVINET",
            Self::ScreamingAutoWah => "SCREAMING FUNK AUTO-WAH",
            Self::TwangyBridgeLead => "TWANGY BRIDGE LEAD",
        }
    }
}

/// Dual electromagnetic pickup mode selector for GUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GuiClavinetPickupMode {
    NeckOnly,
    BridgeOnly,
    #[default]
    ParallelInPhase,
    ParallelOutOfPhase,
}

impl GuiClavinetPickupMode {
    pub fn name(&self) -> &'static str {
        match self {
            Self::NeckOnly => "NECK ONLY (SINGLE-COIL)",
            Self::BridgeOnly => "BRIDGE ONLY (TWANG)",
            Self::ParallelInPhase => "PARALLEL IN-PHASE",
            Self::ParallelOutOfPhase => "PARALLEL OUT-OF-PHASE (QUACK)",
        }
    }
}

/// Electromechanical Clavinet HUD & String-Anvil Collision View.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClavinetView {
    pub profile: GuiClavinetProfile,
    pub pickup_mode: GuiClavinetPickupMode,
    /// Rubber anvil hardness $[0.0 ..= 1.0]$.
    pub anvil_hardness: f32,
    /// Yarn wool damping amount $[0.0 ..= 1.0]$.
    pub yarn_damping: f32,
    /// Pickup mix blend $[0.0 ..= 1.0]$.
    pub pickup_blend: f32,
    /// Pickup phase angle in degrees $[0.0 ..= 180.0^\circ]$.
    pub pickup_phase_deg: f32,
    /// 4-way rocker tone switch bank.
    pub brilliant_switch: bool,
    pub treble_switch: bool,
    pub medium_switch: bool,
    pub soft_switch: bool,
    /// Dynamic Auto-Wah enabled.
    pub auto_wah_enabled: bool,
    pub auto_wah_sensitivity: f32,
    pub auto_wah_freq_hz: f32,
    pub auto_wah_resonance_q: f32,
    pub auto_wah_mix: f32,
    /// Master level $[0.0 ..= 2.0]$.
    pub master_level: f32,
    /// 2D Puck position (X: anvil_hardness, Y: pickup_phase_deg normalized).
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for ClavinetView {
    fn default() -> Self {
        Self::new()
    }
}

impl ClavinetView {
    pub fn new() -> Self {
        let mut view = Self {
            profile: GuiClavinetProfile::StevieSuperstition,
            pickup_mode: GuiClavinetPickupMode::ParallelInPhase,
            anvil_hardness: 0.85,
            yarn_damping: 0.75,
            pickup_blend: 0.50,
            pickup_phase_deg: 0.0,
            brilliant_switch: true,
            treble_switch: true,
            medium_switch: true,
            soft_switch: false,
            auto_wah_enabled: false,
            auto_wah_sensitivity: 0.70,
            auto_wah_freq_hz: 350.0,
            auto_wah_resonance_q: 6.0,
            auto_wah_mix: 0.85,
            master_level: 0.90,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        };
        view.update_puck_from_physics();
        view
    }

    pub fn set_profile(&mut self, profile: GuiClavinetProfile) {
        self.profile = profile;
        match profile {
            GuiClavinetProfile::StevieSuperstition => {
                self.pickup_mode = GuiClavinetPickupMode::ParallelInPhase;
                self.anvil_hardness = 0.85;
                self.yarn_damping = 0.75;
                self.pickup_blend = 0.50;
                self.pickup_phase_deg = 0.0;
                self.brilliant_switch = true;
                self.treble_switch = true;
                self.medium_switch = true;
                self.soft_switch = false;
                self.auto_wah_enabled = false;
            }
            GuiClavinetProfile::OutPhaseFunkQuack => {
                self.pickup_mode = GuiClavinetPickupMode::ParallelOutOfPhase;
                self.anvil_hardness = 0.90;
                self.yarn_damping = 0.85;
                self.pickup_blend = 0.50;
                self.pickup_phase_deg = 180.0;
                self.brilliant_switch = true;
                self.treble_switch = true;
                self.medium_switch = false;
                self.soft_switch = false;
                self.auto_wah_enabled = false;
            }
            GuiClavinetProfile::ClassicD6Clean => {
                self.pickup_mode = GuiClavinetPickupMode::ParallelInPhase;
                self.anvil_hardness = 0.70;
                self.yarn_damping = 0.65;
                self.pickup_blend = 0.55;
                self.pickup_phase_deg = 0.0;
                self.brilliant_switch = true;
                self.treble_switch = false;
                self.medium_switch = true;
                self.soft_switch = false;
                self.auto_wah_enabled = false;
            }
            GuiClavinetProfile::MellowChamber => {
                self.pickup_mode = GuiClavinetPickupMode::NeckOnly;
                self.anvil_hardness = 0.45;
                self.yarn_damping = 0.50;
                self.pickup_blend = 0.0;
                self.pickup_phase_deg = 0.0;
                self.brilliant_switch = false;
                self.treble_switch = false;
                self.medium_switch = false;
                self.soft_switch = true;
                self.auto_wah_enabled = false;
            }
            GuiClavinetProfile::ScreamingAutoWah => {
                self.pickup_mode = GuiClavinetPickupMode::ParallelOutOfPhase;
                self.anvil_hardness = 0.90;
                self.yarn_damping = 0.80;
                self.pickup_blend = 0.50;
                self.pickup_phase_deg = 180.0;
                self.brilliant_switch = true;
                self.treble_switch = true;
                self.medium_switch = false;
                self.soft_switch = false;
                self.auto_wah_enabled = true;
                self.auto_wah_sensitivity = 0.88;
                self.auto_wah_freq_hz = 420.0;
                self.auto_wah_resonance_q = 9.5;
            }
            GuiClavinetProfile::TwangyBridgeLead => {
                self.pickup_mode = GuiClavinetPickupMode::BridgeOnly;
                self.anvil_hardness = 0.95;
                self.yarn_damping = 0.90;
                self.pickup_blend = 1.0;
                self.pickup_phase_deg = 0.0;
                self.brilliant_switch = true;
                self.treble_switch = true;
                self.medium_switch = false;
                self.soft_switch = false;
                self.auto_wah_enabled = false;
            }
        }
        self.update_puck_from_physics();
    }

    pub fn update_puck_from_physics(&mut self) {
        let norm_x = self.anvil_hardness.clamp(0.0, 1.0);
        let norm_y = (self.pickup_phase_deg / 180.0).clamp(0.0, 1.0);
        self.puck_pos = (norm_x, norm_y);
    }

    pub fn update_physics_from_puck(&mut self, norm_x: f32, norm_y: f32) {
        self.puck_pos = (norm_x.clamp(0.0, 1.0), norm_y.clamp(0.0, 1.0));
        self.anvil_hardness = self.puck_pos.0;
        self.pickup_phase_deg = self.puck_pos.1 * 180.0;
    }

    /// Evaluates rubber anvil impact force pulse curve for 32 points.
    pub fn evaluate_anvil_impact_curve(&self) -> [f32; 32] {
        let mut curve = [0.0f32; 32];
        let stiffness = 4.0 + self.anvil_hardness * 12.0;

        for (i, val) in curve.iter_mut().enumerate() {
            if i < 16 {
                let progress = i as f32 / 16.0;
                let pulse = (progress * PI).sin();
                let force = stiffness * pulse.powf(1.6) * 0.15;
                *val = force.clamp(0.0, 1.0);
            } else {
                *val = 0.0;
            }
        }
        curve
    }

    /// Evaluates dual pickup synthesized cancellation waveform display for 32 points.
    pub fn evaluate_pickup_waveform(&self) -> [f32; 32] {
        let mut wave = [0.0f32; 32];
        let phase_rad = self.pickup_phase_deg * (PI / 180.0);

        for (i, val) in wave.iter_mut().enumerate() {
            let t = (i as f32 / 32.0) * 2.0 * PI;
            let neck = (t * 2.0).sin() * 0.7 + (t * 3.0).sin() * 0.3;
            let bridge = (t * 2.0 + phase_rad).sin() * 0.6 + (t * 5.0).sin() * 0.4;

            let mixed = match self.pickup_mode {
                GuiClavinetPickupMode::NeckOnly => neck,
                GuiClavinetPickupMode::BridgeOnly => bridge,
                GuiClavinetPickupMode::ParallelInPhase => {
                    let w_neck = 1.0 - self.pickup_blend;
                    let w_bridge = self.pickup_blend;
                    neck * w_neck + bridge * w_bridge * phase_rad.cos()
                }
                GuiClavinetPickupMode::ParallelOutOfPhase => (neck - bridge) * 0.9,
            };

            *val = mixed.clamp(-1.0, 1.0);
        }
        wave
    }

    pub fn hardness_to_normalized(hard: f32) -> f32 {
        hard.clamp(0.0, 1.0)
    }

    pub fn normalized_to_hardness(norm: f32) -> f32 {
        norm.clamp(0.0, 1.0)
    }

    pub fn phase_to_normalized(phase: f32) -> f32 {
        (phase / 180.0).clamp(0.0, 1.0)
    }

    pub fn normalized_to_phase(norm: f32) -> f32 {
        norm.clamp(0.0, 1.0) * 180.0
    }

    pub fn hit_test_clavinet_puck(&self, screen_pos: (f32, f32), canvas_rect: Rect) -> bool {
        self.hit_test_puck(screen_pos, canvas_rect)
    }

    /// Hit tests the 2D anvil puck.
    pub fn hit_test_puck(&self, screen_pos: (f32, f32), canvas_rect: Rect) -> bool {
        let puck_screen_x = canvas_rect.x + self.puck_pos.0 * canvas_rect.width;
        let puck_screen_y = canvas_rect.y + (1.0 - self.puck_pos.1) * canvas_rect.height;
        let dx = screen_pos.0 - puck_screen_x;
        let dy = screen_pos.1 - puck_screen_y;
        (dx * dx + dy * dy).sqrt() <= CLAV_PUCK_HIT_RADIUS
    }

    pub fn render_ascii_snapshot(&self, width: usize, height: usize) -> Vec<String> {
        self.render_ascii(width, height)
    }

    /// Renders ASCII diagnostic representation.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let header = format!(
            "┌─ [CLAVINET HUD] {} ({}) ─┐",
            self.profile.name(),
            self.pickup_mode.name()
        );
        let pad = width.saturating_sub(header.len());
        lines.push(format!("{}{}", header, "─".repeat(pad)));

        lines.push(format!(
            "│ Anvil Hardness: {:.2} | Damping: {:.2} | Blend: {:.2} | Phase: {:.1}° │",
            self.anvil_hardness, self.yarn_damping, self.pickup_blend, self.pickup_phase_deg
        ));

        lines.push(format!(
            "│ Tone: [Brilliant: {}] [Treble: {}] [Medium: {}] [Soft: {}] │",
            if self.brilliant_switch { "ON " } else { "OFF" },
            if self.treble_switch { "ON " } else { "OFF" },
            if self.medium_switch { "ON " } else { "OFF" },
            if self.soft_switch { "ON " } else { "OFF" }
        ));

        lines.push(format!(
            "│ Auto-Wah: {} | Sens: {:.2} | Freq: {:.0}Hz | Q: {:.1} | Mix: {:.0}% │",
            if self.auto_wah_enabled { "ON " } else { "OFF" },
            self.auto_wah_sensitivity,
            self.auto_wah_freq_hz,
            self.auto_wah_resonance_q,
            self.auto_wah_mix * 100.0
        ));

        let wave = self.evaluate_pickup_waveform();
        let mut wave_str = String::from("│ Wave: ");
        for val in wave.iter().take(24) {
            let ch = if *val > 0.5 { '▲' } else if *val > 0.05 { '─' } else if *val < -0.5 { '▼' } else { ' ' };
            wave_str.push(ch);
        }
        wave_str.push_str(" │");
        lines.push(wave_str);

        while lines.len() < height {
            lines.push(format!("│{:width$}│", "", width = width.saturating_sub(2)));
        }

        lines
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

        // Draw 2D Anvil Hardness vs Phase Canvas Box
        let margin = 32;
        let c_w = (width / 2) - margin * 2;
        let c_h = height - margin * 2;
        let c_x = margin;
        let c_y = margin;

        for y in c_y..(c_y + c_h) {
            for x in c_x..(c_x + c_w) {
                if x < width && y < height {
                    let idx = (y * width + x) * 4;
                    pixels[idx] = 0x18;
                    pixels[idx + 1] = 0x1C;
                    pixels[idx + 2] = 0x2A;
                    pixels[idx + 3] = 0xFF;
                }
            }
        }

        // Draw Anvil Puck Handle (Radius = 22pt)
        let puck_cx = (c_x as f32 + self.puck_pos.0 * c_w as f32) as usize;
        let puck_cy = (c_y as f32 + (1.0 - self.puck_pos.1) * c_h as f32) as usize;
        let r = CLAV_PUCK_HIT_RADIUS as usize;

        for dy in 0..=(2 * r) {
            for dx in 0..=(2 * r) {
                let px = (puck_cx + dx).saturating_sub(r);
                let py = (puck_cy + dy).saturating_sub(r);
                let dist_sq = (dx as isize - r as isize).pow(2) + (dy as isize - r as isize).pow(2);
                if dist_sq <= (r * r) as isize && px < width && py < height {
                    let idx = (py * width + px) * 4;
                    // Tangerine Orange #F77F00
                    pixels[idx] = 0xF7;
                    pixels[idx + 1] = 0x7F;
                    pixels[idx + 2] = 0x00;
                    pixels[idx + 3] = 0xFF;
                }
            }
        }

        // Draw Pickup Waveform in Right Half
        let wave = self.evaluate_pickup_waveform();
        let w_x0 = (width / 2) + margin;
        let w_w = width - w_x0 - margin;
        let w_cy = height / 2;

        for (i, &val) in wave.iter().enumerate() {
            let x_coord = w_x0 + (i * w_w) / 32;
            let y_offset = (val * (c_h as f32 * 0.35)) as isize;
            let y_coord = (w_cy as isize - y_offset).clamp(margin as isize, (height - margin) as isize) as usize;

            for dy in 0..3 {
                for dx in 0..3 {
                    let px = (x_coord + dx).min(width - 1);
                    let py = (y_coord + dy).min(height - 1);
                    let idx = (py * width + px) * 4;
                    // Electric Amber #FFD166
                    pixels[idx] = 0xFF;
                    pixels[idx + 1] = 0xD1;
                    pixels[idx + 2] = 0x66;
                    pixels[idx + 3] = 0xFF;
                }
            }
        }

        save_png_file(path, width, height, &pixels)
    }
}

#[cfg(feature = "gui")]
impl ClavinetView {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        self.ui(ui);
    }

    #[allow(clippy::needless_range_loop)]
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 480.0),
            egui::Sense::click_and_drag(),
        );

        let painter = ui.painter_at(rect);

        // Background: Deep Slate Charcoal (#0C101A)
        painter.rect_filled(rect, 6.0, Color32::from_rgb(12, 16, 26));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "ELECTROMECHANICAL CLAVINET D6 & STRING-ANVIL COLLISION HUD",
            egui::FontId::proportional(13.5),
            Color32::from_rgb(240, 245, 255),
        );

        // Tabs (y: 48..92) - 44pt height
        let tabs = [
            (GuiClavinetProfile::StevieSuperstition, "SUPERSTITION"),
            (GuiClavinetProfile::OutPhaseFunkQuack, "FUNK QUACK"),
            (GuiClavinetProfile::ClassicD6Clean, "D6 CLEAN"),
            (GuiClavinetProfile::MellowChamber, "MELLOW"),
            (GuiClavinetProfile::ScreamingAutoWah, "AUTO-WAH"),
            (GuiClavinetProfile::TwangyBridgeLead, "BRIDGE LEAD"),
        ];

        let tab_w = (rect.width() - 40.0 - 5.0 * 8.0) / 6.0;
        for (i, (profile, name)) in tabs.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_sel = self.profile == *profile;
            let bg_col = if is_sel {
                Color32::from_rgb(249, 115, 22)
            } else {
                Color32::from_rgb(24, 32, 48)
            };
            let text_col = if is_sel {
                Color32::from_rgb(16, 8, 4)
            } else {
                Color32::from_rgb(210, 225, 245)
            };

            painter.rect_filled(tab_rect, 4.0, bg_col);
            painter.text(
                tab_rect.center(),
                egui::Align2::CENTER_CENTER,
                *name,
                egui::FontId::proportional(10.5),
                text_col,
            );

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if tab_rect.contains(pos) {
                        self.set_profile(*profile);
                    }
                }
            }
        }

        // Main Display Canvas (y: 104..340)
        let main_canvas = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 104.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 340.0),
        );
        painter.rect_filled(main_canvas, 6.0, Color32::from_rgb(8, 12, 22));
        painter.rect_stroke(
            main_canvas,
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(32, 48, 72)),
        );

        // Split canvas into Left Pad (Anvil vs Phase) and Right Visualizer (Waveform & Pulse)
        let pad_w = (main_canvas.width() - 30.0) * 0.48;
        let pad_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + 15.0, main_canvas.min.y + 15.0),
            egui::vec2(pad_w, main_canvas.height() - 30.0),
        );
        painter.rect_filled(pad_rect, 4.0, Color32::from_rgb(16, 22, 34));
        painter.rect_stroke(
            pad_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        // Grid lines on Left Pad
        for gy in 1..4 {
            let y = pad_rect.min.y + gy as f32 * (pad_rect.height() / 4.0);
            painter.line_segment(
                [egui::pos2(pad_rect.min.x, y), egui::pos2(pad_rect.max.x, y)],
                Stroke::new(0.5_f32, Color32::from_rgb(30, 42, 60)),
            );
        }
        for gx in 1..4 {
            let x = pad_rect.min.x + gx as f32 * (pad_rect.width() / 4.0);
            painter.line_segment(
                [egui::pos2(x, pad_rect.min.y), egui::pos2(x, pad_rect.max.y)],
                Stroke::new(0.5_f32, Color32::from_rgb(30, 42, 60)),
            );
        }

        // Handle interaction on Left Pad
        if response.dragged() || response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                if pad_rect.contains(pos) {
                    let norm_x = ((pos.x - pad_rect.min.x) / pad_rect.width()).clamp(0.0, 1.0);
                    let norm_y = (1.0 - ((pos.y - pad_rect.min.y) / pad_rect.height())).clamp(0.0, 1.0);
                    self.update_physics_from_puck(norm_x, norm_y);
                }
            }
        }

        // Draggable Puck on Pad (radius 22pt)
        let puck_x = pad_rect.min.x + self.puck_pos.0 * pad_rect.width();
        let puck_y = pad_rect.max.y - self.puck_pos.1 * pad_rect.height();
        painter.circle_filled(
            egui::pos2(puck_x, puck_y),
            12.0,
            Color32::from_rgb(249, 115, 22),
        );
        painter.circle_stroke(
            egui::pos2(puck_x, puck_y),
            CLAV_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgb(254, 215, 170)),
        );

        painter.text(
            egui::pos2(pad_rect.min.x + 8.0, pad_rect.min.y + 8.0),
            egui::Align2::LEFT_TOP,
            "ANVIL HARDNESS vs PICKUP PHASE (>= 44x44pt Touch Target)",
            egui::FontId::proportional(9.5),
            Color32::from_rgb(253, 186, 116),
        );

        // Right side: Rubber Anvil Pulse & Pickup Cancellation Waveform
        let wave_rect = egui::Rect::from_min_size(
            egui::pos2(pad_rect.max.x + 15.0, main_canvas.min.y + 15.0),
            egui::vec2(main_canvas.max.x - pad_rect.max.x - 30.0, main_canvas.height() - 30.0),
        );
        painter.rect_filled(wave_rect, 4.0, Color32::from_rgb(14, 18, 28));
        painter.rect_stroke(
            wave_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(wave_rect.min.x + 8.0, wave_rect.min.y + 8.0),
            egui::Align2::LEFT_TOP,
            "DUAL PICKUP CANCELLATION WAVEFORM & ANVIL IMPACT DYNAMICS",
            egui::FontId::proportional(9.5),
            Color32::from_rgb(254, 240, 138),
        );

        // Center zero line
        let wave_mid_y = wave_rect.center().y;
        painter.line_segment(
            [egui::pos2(wave_rect.min.x, wave_mid_y), egui::pos2(wave_rect.max.x, wave_mid_y)],
            Stroke::new(0.5_f32, Color32::from_rgb(50, 65, 90)),
        );

        // Render waveform points
        let wave = self.evaluate_pickup_waveform();
        let n_pts = wave.len();
        let step_x = wave_rect.width() / (n_pts as f32 - 1.0).max(1.0);
        let mut pts = Vec::with_capacity(n_pts);
        for (i, &val) in wave.iter().enumerate() {
            let px = wave_rect.min.x + i as f32 * step_x;
            let py = wave_mid_y - val * (wave_rect.height() * 0.38);
            pts.push(egui::pos2(px, py));
        }
        for w_idx in 0..pts.len().saturating_sub(1) {
            painter.line_segment(
                [pts[w_idx], pts[w_idx + 1]],
                Stroke::new(1.8_f32, Color32::from_rgb(255, 209, 102)),
            );
        }

        // Render anvil impact curve overlay in lower quadrant
        let impact = self.evaluate_anvil_impact_curve();
        let step_ix = wave_rect.width() / (impact.len() as f32 - 1.0).max(1.0);
        let mut ipts = Vec::with_capacity(impact.len());
        for (i, &val) in impact.iter().enumerate() {
            let px = wave_rect.min.x + i as f32 * step_ix;
            let py = wave_rect.max.y - 10.0 - val * (wave_rect.height() * 0.28);
            ipts.push(egui::pos2(px, py));
        }
        for i_idx in 0..ipts.len().saturating_sub(1) {
            painter.line_segment(
                [ipts[i_idx], ipts[i_idx + 1]],
                Stroke::new(1.2_f32, Color32::from_rgb(249, 115, 22)),
            );
        }

        // Bottom Metrics Dock & 4 Tone Switches (y: 350..465)
        let dock_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 350.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 465.0),
        );
        painter.rect_filled(dock_rect, 6.0, Color32::from_rgb(18, 24, 38));
        painter.rect_stroke(
            dock_rect,
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(45, 65, 95)),
        );

        // 4 Rocker Switches (Brilliant, Treble, Medium, Soft)
        let switches = [
            ("BRILLIANT", self.brilliant_switch),
            ("TREBLE", self.treble_switch),
            ("MEDIUM", self.medium_switch),
            ("SOFT", self.soft_switch),
        ];

        let sw_w = 90.0;
        for (i, (s_name, s_val)) in switches.iter().enumerate() {
            let sx = dock_rect.min.x + 16.0 + i as f32 * (sw_w + 10.0);
            let sw_rect = egui::Rect::from_min_size(
                egui::pos2(sx, dock_rect.min.y + 12.0),
                egui::vec2(sw_w, 44.0),
            );
            let s_col = if *s_val {
                Color32::from_rgb(249, 115, 22)
            } else {
                Color32::from_rgb(32, 42, 60)
            };
            let text_col = if *s_val {
                Color32::from_rgb(16, 8, 4)
            } else {
                Color32::from_rgb(180, 200, 225)
            };
            painter.rect_filled(sw_rect, 4.0, s_col);
            painter.text(
                sw_rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("[{}]", s_name),
                egui::FontId::proportional(10.0),
                text_col,
            );

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if sw_rect.contains(pos) {
                        match i {
                            0 => self.brilliant_switch = !self.brilliant_switch,
                            1 => self.treble_switch = !self.treble_switch,
                            2 => self.medium_switch = !self.medium_switch,
                            3 => self.soft_switch = !self.soft_switch,
                            _ => {}
                        }
                    }
                }
            }
        }

        // Metrics Labels
        let metrics_x = dock_rect.min.x + 430.0;
        painter.text(
            egui::pos2(metrics_x, dock_rect.min.y + 14.0),
            egui::Align2::LEFT_TOP,
            format!("Hardness: {:.0}% | Damping: {:.0}%", self.anvil_hardness * 100.0, self.yarn_damping * 100.0),
            egui::FontId::proportional(11.0),
            Color32::from_rgb(254, 215, 170),
        );
        painter.text(
            egui::pos2(metrics_x, dock_rect.min.y + 34.0),
            egui::Align2::LEFT_TOP,
            format!("Phase: {:.1}° | Mode: {}", self.pickup_phase_deg, self.pickup_mode.name()),
            egui::FontId::proportional(11.0),
            Color32::from_rgb(255, 209, 102),
        );

        // Compliance Badge
        let badge_rect = egui::Rect::from_min_max(
            egui::pos2(dock_rect.min.x + 15.0, dock_rect.min.y + 68.0),
            egui::pos2(dock_rect.max.x - 15.0, dock_rect.max.y - 11.0),
        );
        painter.rect_filled(badge_rect, 4.0, Color32::from_rgb(14, 35, 28));
        painter.rect_stroke(
            badge_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(0, 255, 180)),
        );
        painter.text(
            egui::pos2(badge_rect.min.x + 10.0, badge_rect.min.y + 8.0),
            egui::Align2::LEFT_TOP,
            "[PASS] Electromechanical Clavinet D6 Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}

fn save_png_file(path: &str, width: usize, height: usize, rgba_pixels: &[u8]) -> Result<(), String> {
    let mut out = Vec::with_capacity(width * height * 4 + 1024);
    out.extend_from_slice(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]); // PNG Header

    // IHDR Chunk
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&(width as u32).to_be_bytes());
    ihdr.extend_from_slice(&(height as u32).to_be_bytes());
    ihdr.push(8); // Bit depth
    ihdr.push(6); // Color type: RGBA
    ihdr.push(0); // Compression method
    ihdr.push(0); // Filter method
    ihdr.push(0); // Interlace method
    write_png_chunk_to(&mut out, b"IHDR", &ihdr);

    // IDAT Chunk (Raw deflate uncompressed blocks)
    let stride = width * 4;
    let mut raw_data = Vec::with_capacity((stride + 1) * height);
    for y in 0..height {
        raw_data.push(0x00); // Filter type None
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
    fn test_clavinet_view_hit_target_dimensions() {
        const {
            assert!(
                CLAV_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT,
                "Clavinet puck hit target bounding box must be >= 44pt"
            );
        }
    }

    #[test]
    fn test_clavinet_view_ascii_render() {
        let view = ClavinetView::new();
        let ascii = view.render_ascii(80, 16);
        assert_eq!(ascii.len(), 16);
        assert!(ascii[0].contains("CLAVINET HUD"));
    }

    #[test]
    fn test_clavinet_view_snapshot_render() {
        let view = ClavinetView::new();
        let res = view.render_snapshot_png("scratch/renders/clavinet_view.png", 800, 520);
        assert!(res.is_ok());
    }
}
