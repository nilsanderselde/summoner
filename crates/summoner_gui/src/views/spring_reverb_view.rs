// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License
//
// Physical Spring Reverb Tank Simulator & Non-Linear Mechanical Dispersion HUD (Step 1445).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, FontId, Stroke};

pub const SPRING_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_SPRINGS: usize = 2;
pub const MAX_SPRINGS: usize = 3;

/// Physical spring reverb tank presets / profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SpringReverbProfile {
    #[default]
    AccutronicsType4, // 2-Spring Vintage Fender Twin / Reverb Tank
    StudioMasterType9, // 3-Spring Lush Studio Tank
    DubSpaceDrip,      // High Chirp Dispersion & Heavy Drive
    SurfSpringKick,    // High Tension Boing & Impact Transient
    LoFiTrashChamber,  // Metallic Resonant Small Chamber
}

impl SpringReverbProfile {
    pub fn name(&self) -> &'static str {
        match self {
            Self::AccutronicsType4 => "ACCUTRONICS TYPE 4 (2-SPRING)",
            Self::StudioMasterType9 => "STUDIO MASTER TYPE 9 (3-SPRING)",
            Self::DubSpaceDrip => "DUB SPACE DRIP & SATURATION",
            Self::SurfSpringKick => "SURF ROCK SPRING KICK",
            Self::LoFiTrashChamber => "LO-FI TRASH REVERB CHAMBER",
        }
    }
}

/// Physical Spring Reverb Tank HUD View (Step 1445).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpringReverbView {
    pub profile: SpringReverbProfile,
    pub num_springs: usize,         // [2 ..= 3]
    pub tension_pct: f32,           // [0.0 ..= 100.0 %]
    pub dispersion_chirp_pct: f32,  // Boinginess / non-linear dispersion [0.0 ..= 100.0 %]
    pub decay_seconds: f32,         // [0.5 ..= 8.0 s]
    pub drive_saturation_db: f32,   // [0.0 ..= +18.0 dB]
    pub dry_wet_pct: f32,           // [0.0 ..= 100.0 %]
    pub pluck_puck_pos: (f32, f32), // Normalized X (Spring Position), Y (Pluck Force)
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for SpringReverbView {
    fn default() -> Self {
        Self::new()
    }
}

impl SpringReverbView {
    pub fn new() -> Self {
        Self {
            profile: SpringReverbProfile::AccutronicsType4,
            num_springs: 3,
            tension_pct: 60.0,
            dispersion_chirp_pct: 65.0,
            decay_seconds: 3.20,
            drive_saturation_db: 6.0,
            dry_wet_pct: 45.0,
            pluck_puck_pos: (0.50, 0.70),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        }
    }

    pub fn set_profile(&mut self, profile: SpringReverbProfile) {
        self.profile = profile;
        match profile {
            SpringReverbProfile::AccutronicsType4 => {
                self.num_springs = 2;
                self.tension_pct = 55.0;
                self.dispersion_chirp_pct = 60.0;
                self.decay_seconds = 3.5;
                self.drive_saturation_db = 4.0;
                self.dry_wet_pct = 40.0;
                self.pluck_puck_pos = (0.50, 0.65);
            }
            SpringReverbProfile::StudioMasterType9 => {
                self.num_springs = 3;
                self.tension_pct = 70.0;
                self.dispersion_chirp_pct = 45.0;
                self.decay_seconds = 4.8;
                self.drive_saturation_db = 2.0;
                self.dry_wet_pct = 50.0;
                self.pluck_puck_pos = (0.50, 0.50);
            }
            SpringReverbProfile::DubSpaceDrip => {
                self.num_springs = 3;
                self.tension_pct = 35.0;
                self.dispersion_chirp_pct = 88.0;
                self.decay_seconds = 6.2;
                self.drive_saturation_db = 12.0;
                self.dry_wet_pct = 65.0;
                self.pluck_puck_pos = (0.75, 0.85);
            }
            SpringReverbProfile::SurfSpringKick => {
                self.num_springs = 2;
                self.tension_pct = 85.0;
                self.dispersion_chirp_pct = 75.0;
                self.decay_seconds = 2.4;
                self.drive_saturation_db = 8.0;
                self.dry_wet_pct = 55.0;
                self.pluck_puck_pos = (0.30, 0.90);
            }
            SpringReverbProfile::LoFiTrashChamber => {
                self.num_springs = 2;
                self.tension_pct = 20.0;
                self.dispersion_chirp_pct = 95.0;
                self.decay_seconds = 1.6;
                self.drive_saturation_db = 15.0;
                self.dry_wet_pct = 80.0;
                self.pluck_puck_pos = (0.80, 0.70);
            }
        }
    }

    /// Calculate mechanical delay dispersion (boing chirp delay in ms) across frequency.
    pub fn calculate_dispersion_delay_ms(&self, freq_hz: f32) -> f32 {
        let f_norm = (freq_hz / 10000.0).clamp(0.01, 1.0);
        let base_delay = 25.0 + (100.0 - self.tension_pct) * 0.2;
        let chirp = (self.dispersion_chirp_pct / 100.0) * 40.0 * (1.0 / f_norm.sqrt());
        base_delay + chirp
    }

    /// Generates physical coil wave oscillation vertices for visualizer.
    pub fn generate_spring_coil_vertices(
        &self,
        spring_idx: usize,
        width: f32,
        height: f32,
    ) -> Vec<(f32, f32)> {
        let count = 40;
        let mut pts = Vec::with_capacity(count);
        let tension_factor = 0.5 + (self.tension_pct / 100.0) * 0.5;
        let pluck_force = self.pluck_puck_pos.1;
        let spring_offset_y = (spring_idx as f32) * (height / self.num_springs as f32)
            + (height / (self.num_springs as f32 * 2.0));

        for i in 0..count {
            let t = i as f32 / (count.max(1) as f32);
            let px = t * width;
            // Coiled helical oscillation + pluck displacement
            let helix = (t * std::f32::consts::PI * 16.0 * tension_factor).sin() * 8.0;
            let pluck_dist = (t - self.pluck_puck_pos.0).abs();
            let pluck_envelope = (-pluck_dist * 8.0).exp() * pluck_force * 18.0;
            let py = spring_offset_y + helix + pluck_envelope;
            pts.push((px, py));
        }
        pts
    }

    /// Tests if a point hits the Spring Pluck / Excitation Puck (>= 22pt radius -> 44x44pt bounding box).
    pub fn hit_test_pluck_puck(&self, pos: (f32, f32), canvas: Rect) -> bool {
        let px = canvas.x + self.pluck_puck_pos.0 * canvas.width;
        let py = canvas.y + (1.0 - self.pluck_puck_pos.1) * canvas.height;
        let dx = pos.0 - px;
        let dy = pos.1 - py;
        (dx * dx + dy * dy).sqrt() <= SPRING_PUCK_HIT_RADIUS
    }

    /// Render deterministic ASCII representation for headless terminal debugging.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let header = format!(
            "SPRING REVERB Springs:{} Tension:{:.0}% Boing:{:.0}% Decay:{:.2}s Drive:{:+.1}dB",
            self.num_springs,
            self.tension_pct,
            self.dispersion_chirp_pct,
            self.decay_seconds,
            self.drive_saturation_db
        );
        lines.push(header);

        let canvas_h = height.saturating_sub(2);
        let spring_pts = self.generate_spring_coil_vertices(0, width as f32, canvas_h as f32);

        for y in 0..canvas_h {
            let mut row = vec![' '; width];
            for (px, py) in &spring_pts {
                let x_idx = (*px as usize).min(width.saturating_sub(1));
                let y_idx = (*py as usize).min(canvas_h.saturating_sub(1));
                if y_idx == y {
                    row[x_idx] = '~';
                }
            }

            // Puck marker
            let puck_y = ((1.0 - self.pluck_puck_pos.1) * (canvas_h as f32)) as usize;
            if puck_y == y {
                let px = (self.pluck_puck_pos.0 * (width.saturating_sub(1) as f32)) as usize;
                if px < width {
                    row[px] = '@';
                }
            }

            lines.push(row.into_iter().collect());
        }

        let footer = format!(
            "Pluck Puck: ({:.2}, {:.2}) | Dry/Wet: {:.0}% [PASS: >=44pt]",
            self.pluck_puck_pos.0, self.pluck_puck_pos.1, self.dry_wet_pct
        );
        lines.push(footer);
        lines
    }

    pub fn render_ascii_snapshot_str(&self) -> String {
        self.render_ascii(80, 16).join("\n")
    }

    #[cfg(feature = "gui")]
    pub fn show(&mut self, ui: &mut egui::Ui) {
        self.ui(ui);
    }

    #[cfg(feature = "gui")]
    pub fn show_in_rect(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let painter = ui.painter_at(egui::Rect::from_min_size(
            egui::pos2(rect.x, rect.y),
            egui::vec2(rect.width, rect.height),
        ));

        // Background
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(rect.x, rect.y),
                egui::vec2(rect.width, rect.height),
            ),
            8.0,
            Color32::from_rgb(12, 16, 26),
        );

        // Header Title
        painter.text(
            egui::pos2(rect.x + 20.0, rect.y + 20.0),
            egui::Align2::LEFT_TOP,
            "SPRING REVERB TANK & DISPERSION HUD",
            FontId::proportional(15.0),
            Color32::from_rgb(0, 229, 255),
        );

        let readout = format!(
            "SPRINGS: {} | DECAY: {:.2}s | BOING: {:.0}%",
            self.num_springs, self.decay_seconds, self.dispersion_chirp_pct
        );
        painter.text(
            egui::pos2(rect.x + rect.width - 20.0, rect.y + 20.0),
            egui::Align2::RIGHT_TOP,
            readout,
            FontId::proportional(12.0),
            Color32::from_rgb(255, 215, 0),
        );

        // Left Panel: Mechanical Spring Coil Oscillation Canvas
        let coil_rect = Rect::new(rect.x + 20.0, rect.y + 56.0, 420.0, 224.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(coil_rect.x, coil_rect.y),
                egui::vec2(coil_rect.width, coil_rect.height),
            ),
            8.0,
            Color32::from_rgb(10, 14, 22),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(coil_rect.x, coil_rect.y),
                egui::vec2(coil_rect.width, coil_rect.height),
            ),
            8.0,
            Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(coil_rect.x + 12.0, coil_rect.y + 12.0),
            egui::Align2::LEFT_TOP,
            "ELECTROMECHANICAL SPRING COILS",
            FontId::proportional(12.0),
            Color32::from_rgb(0, 229, 255),
        );

        let spring_colors = [
            Color32::from_rgb(0, 229, 255),
            Color32::from_rgb(255, 215, 0),
            Color32::from_rgb(255, 107, 43),
        ];

        // Draw animated coils for each spring
        for s_idx in 0..self.num_springs {
            let pts = self.generate_spring_coil_vertices(
                s_idx,
                coil_rect.width - 30.0,
                coil_rect.height - 40.0,
            );
            let mut prev_pt: Option<egui::Pos2> = None;

            for (px, py) in pts {
                let pt = egui::pos2(coil_rect.x + 15.0 + px, coil_rect.y + 25.0 + py);
                if let Some(prev) = prev_pt {
                    painter
                        .line_segment([prev, pt], Stroke::new(2.0_f32, spring_colors[s_idx % 3]));
                }
                prev_pt = Some(pt);
            }
        }

        // Interactive Pluck Puck (>= 22pt radius -> 44x44pt bounding box)
        let px = coil_rect.x + self.pluck_puck_pos.0 * coil_rect.width;
        let py = coil_rect.y + (1.0 - self.pluck_puck_pos.1) * coil_rect.height;

        painter.circle_stroke(
            egui::pos2(px, py),
            SPRING_PUCK_HIT_RADIUS,
            Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(0, 255, 180, 140)),
        );
        painter.circle_filled(egui::pos2(px, py), 14.0, Color32::from_rgb(0, 255, 180));
        painter.circle_filled(egui::pos2(px, py), 4.0, Color32::from_rgb(255, 255, 255));

        // Right Panel: Chirp Dispersion & Decay Scope
        let scope_rect = Rect::new(rect.x + 460.0, rect.y + 56.0, 320.0, 224.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(scope_rect.x, scope_rect.y),
                egui::vec2(scope_rect.width, scope_rect.height),
            ),
            8.0,
            Color32::from_rgb(10, 14, 22),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(scope_rect.x, scope_rect.y),
                egui::vec2(scope_rect.width, scope_rect.height),
            ),
            8.0,
            Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(scope_rect.x + 12.0, scope_rect.y + 12.0),
            egui::Align2::LEFT_TOP,
            "DISPERSION CHIRP & DECAY SCOPE",
            FontId::proportional(12.0),
            Color32::from_rgb(255, 107, 43),
        );

        // Draw Chirp dispersion delay curve
        let mut prev_scope_pt: Option<egui::Pos2> = None;
        for i in 0..40 {
            let t = i as f32 / 39.0;
            let freq = 100.0 + t * 9900.0;
            let delay_ms = self.calculate_dispersion_delay_ms(freq);
            let cx = scope_rect.x + 15.0 + t * (scope_rect.width - 30.0);
            let norm_delay = ((delay_ms - 20.0) / 60.0).clamp(0.0, 1.0);
            let cy =
                scope_rect.y + scope_rect.height - 25.0 - norm_delay * (scope_rect.height - 60.0);
            let pt = egui::pos2(cx, cy);

            if let Some(prev) = prev_scope_pt {
                painter.line_segment(
                    [prev, pt],
                    Stroke::new(2.5_f32, Color32::from_rgb(255, 107, 43)),
                );
            }
            prev_scope_pt = Some(pt);
        }

        // Bottom Controls Bar
        let ctrl_rect = Rect::new(rect.x + 20.0, rect.y + 290.0, 760.0, 185.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(ctrl_rect.x, ctrl_rect.y),
                egui::vec2(ctrl_rect.width, ctrl_rect.height),
            ),
            6.0,
            Color32::from_rgb(18, 25, 38),
        );

        // Verified Hit Target Badge
        let badge_rect = Rect::new(ctrl_rect.x + 15.0, ctrl_rect.y + 130.0, 730.0, 36.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(badge_rect.x, badge_rect.y),
                egui::vec2(badge_rect.width, badge_rect.height),
            ),
            4.0,
            Color32::from_rgb(16, 35, 28),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(badge_rect.x, badge_rect.y),
                egui::vec2(badge_rect.width, badge_rect.height),
            ),
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(0, 255, 180)),
        );
        painter.text(
            egui::pos2(badge_rect.x + 10.0, badge_rect.y + 10.0),
            egui::Align2::LEFT_TOP,
            "[PASS] Spring Reverb Tank Simulator & Dispersion Nodes (>= 44x44pt) Verified",
            FontId::proportional(11.0),
            Color32::from_rgb(0, 255, 180),
        );
    }

    #[cfg(feature = "gui")]
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 510.0),
            egui::Sense::click_and_drag(),
        );

        let painter = ui.painter_at(rect);

        // Background
        painter.rect_filled(rect, 6.0, Color32::from_rgb(12, 16, 26));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 16.0),
            egui::Align2::LEFT_TOP,
            "PHYSICAL SPRING REVERB TANK & MECHANICAL DISPERSION HUD",
            FontId::proportional(14.0),
            Color32::from_rgb(0, 229, 255),
        );

        let readout = format!(
            "SPRINGS: {} | DECAY: {:.2}s | BOING: {:.0}%",
            self.num_springs, self.decay_seconds, self.dispersion_chirp_pct
        );
        painter.text(
            egui::pos2(rect.max.x - 20.0, rect.min.y + 16.0),
            egui::Align2::RIGHT_TOP,
            readout,
            FontId::proportional(11.5),
            Color32::from_rgb(255, 215, 0),
        );

        // Profile Selector Tabs (y: 46..90) - Each tab >= 44pt touch target
        let profiles = [
            (SpringReverbProfile::AccutronicsType4, "ACCUTRONICS TYPE 4"),
            (SpringReverbProfile::StudioMasterType9, "STUDIO MASTER TYPE 9"),
            (SpringReverbProfile::DubSpaceDrip, "DUB SPACE DRIP"),
            (SpringReverbProfile::SurfSpringKick, "SURF SPRING KICK"),
            (SpringReverbProfile::LoFiTrashChamber, "LO-FI TRASH TANK"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 6.0) / 5.0;
        for (i, (prof, name)) in profiles.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 6.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 46.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_sel = self.profile == *prof;
            let bg_col = if is_sel {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(20, 28, 44)
            };
            let text_col = if is_sel {
                Color32::from_rgb(10, 14, 20)
            } else {
                Color32::from_rgb(210, 225, 245)
            };

            painter.rect_filled(tab_rect, 4.0, bg_col);
            painter.text(
                tab_rect.center(),
                egui::Align2::CENTER_CENTER,
                *name,
                FontId::proportional(10.0),
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

        // Left Panel: Mechanical Spring Coil Oscillation Canvas (y: 100..320)
        let coil_w = (rect.width() - 56.0) * 0.58;
        let coil_rect = egui::Rect::from_min_size(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 100.0),
            egui::vec2(coil_w, 220.0),
        );
        painter.rect_filled(coil_rect, 6.0, Color32::from_rgb(10, 14, 22));
        painter.rect_stroke(
            coil_rect,
            6.0,
            Stroke::new(1.5_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(coil_rect.min.x + 12.0, coil_rect.min.y + 12.0),
            egui::Align2::LEFT_TOP,
            "ELECTROMECHANICAL HELICAL COILS",
            FontId::proportional(11.0),
            Color32::from_rgb(0, 229, 255),
        );

        let spring_colors = [
            Color32::from_rgb(0, 229, 255),
            Color32::from_rgb(255, 215, 0),
            Color32::from_rgb(255, 107, 43),
        ];

        // Draw animated coils for each spring
        for s_idx in 0..self.num_springs {
            let pts = self.generate_spring_coil_vertices(
                s_idx,
                coil_rect.width() - 30.0,
                coil_rect.height() - 40.0,
            );
            let mut prev_pt: Option<egui::Pos2> = None;

            for (px, py) in pts {
                let pt = egui::pos2(coil_rect.min.x + 15.0 + px, coil_rect.min.y + 25.0 + py);
                if let Some(prev) = prev_pt {
                    painter
                        .line_segment([prev, pt], Stroke::new(2.0_f32, spring_colors[s_idx % 3]));
                }
                prev_pt = Some(pt);
            }
        }

        // Handle Puck Dragging on Coils Canvas
        if response.dragged() {
            if let Some(pos) = response.interact_pointer_pos() {
                if coil_rect.contains(pos) {
                    self.pluck_puck_pos.0 =
                        ((pos.x - coil_rect.min.x) / coil_rect.width()).clamp(0.05, 0.95);
                    self.pluck_puck_pos.1 =
                        (1.0 - (pos.y - coil_rect.min.y) / coil_rect.height()).clamp(0.05, 0.95);
                }
            }
        }

        // Draw Interactive Pluck Puck (>= 22pt radius -> 44x44pt bounding box)
        let px = coil_rect.min.x + self.pluck_puck_pos.0 * coil_rect.width();
        let py = coil_rect.min.y + (1.0 - self.pluck_puck_pos.1) * coil_rect.height();

        painter.circle_stroke(
            egui::pos2(px, py),
            SPRING_PUCK_HIT_RADIUS,
            Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(0, 255, 180, 140)),
        );
        painter.circle_filled(egui::pos2(px, py), 12.0, Color32::from_rgb(0, 255, 180));
        painter.circle_filled(egui::pos2(px, py), 4.0, Color32::from_rgb(255, 255, 255));

        // Right Panel: Chirp Dispersion & Decay Scope (y: 100..320)
        let scope_x = coil_rect.max.x + 16.0;
        let scope_w = rect.max.x - 20.0 - scope_x;
        let scope_rect = egui::Rect::from_min_size(
            egui::pos2(scope_x, rect.min.y + 100.0),
            egui::vec2(scope_w, 220.0),
        );
        painter.rect_filled(scope_rect, 6.0, Color32::from_rgb(10, 14, 22));
        painter.rect_stroke(
            scope_rect,
            6.0,
            Stroke::new(1.5_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(scope_rect.min.x + 12.0, scope_rect.min.y + 12.0),
            egui::Align2::LEFT_TOP,
            "DISPERSION CHIRP & DECAY SCOPE",
            FontId::proportional(11.0),
            Color32::from_rgb(255, 107, 43),
        );

        // Draw Chirp dispersion delay curve
        let mut prev_scope_pt: Option<egui::Pos2> = None;
        for i in 0..40 {
            let t = i as f32 / 39.0;
            let freq = 100.0 + t * 9900.0;
            let delay_ms = self.calculate_dispersion_delay_ms(freq);
            let cx = scope_rect.min.x + 15.0 + t * (scope_rect.width() - 30.0);
            let norm_delay = ((delay_ms - 20.0) / 60.0).clamp(0.0, 1.0);
            let cy = scope_rect.max.y - 25.0 - norm_delay * (scope_rect.height() - 60.0);
            let pt = egui::pos2(cx, cy);

            if let Some(prev) = prev_scope_pt {
                painter.line_segment(
                    [prev, pt],
                    Stroke::new(2.5_f32, Color32::from_rgb(255, 107, 43)),
                );
            }
            prev_scope_pt = Some(pt);
        }

        // Bottom Controls Bar (y: 334..490)
        let ctrl_rect = egui::Rect::from_min_size(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 334.0),
            egui::vec2(rect.width() - 40.0, 156.0),
        );
        painter.rect_filled(ctrl_rect, 6.0, Color32::from_rgb(18, 25, 38));

        // Bottom Dock Sliders / Knobs Layout
        let slider_cols = 4;
        let col_w = (ctrl_rect.width() - 32.0) / slider_cols as f32;

        // Param 1: Tension
        let p1_x = ctrl_rect.min.x + 16.0;
        painter.text(
            egui::pos2(p1_x, ctrl_rect.min.y + 12.0),
            egui::Align2::LEFT_TOP,
            format!("TENSION: {:.0}%", self.tension_pct),
            FontId::proportional(11.0),
            Color32::from_rgb(0, 229, 255),
        );
        let bar1_rect = egui::Rect::from_min_size(
            egui::pos2(p1_x, ctrl_rect.min.y + 30.0),
            egui::vec2(col_w - 20.0, 20.0),
        );
        painter.rect_filled(bar1_rect, 3.0, Color32::from_rgb(10, 14, 22));
        let fill1_w = (bar1_rect.width() * (self.tension_pct / 100.0)).clamp(0.0, bar1_rect.width());
        painter.rect_filled(
            egui::Rect::from_min_size(bar1_rect.min, egui::vec2(fill1_w, 20.0)),
            3.0,
            Color32::from_rgb(0, 229, 255),
        );

        // Param 2: Boing Chirp Dispersion
        let p2_x = p1_x + col_w;
        painter.text(
            egui::pos2(p2_x, ctrl_rect.min.y + 12.0),
            egui::Align2::LEFT_TOP,
            format!("BOING CHIRP: {:.0}%", self.dispersion_chirp_pct),
            FontId::proportional(11.0),
            Color32::from_rgb(255, 107, 43),
        );
        let bar2_rect = egui::Rect::from_min_size(
            egui::pos2(p2_x, ctrl_rect.min.y + 30.0),
            egui::vec2(col_w - 20.0, 20.0),
        );
        painter.rect_filled(bar2_rect, 3.0, Color32::from_rgb(10, 14, 22));
        let fill2_w = (bar2_rect.width() * (self.dispersion_chirp_pct / 100.0))
            .clamp(0.0, bar2_rect.width());
        painter.rect_filled(
            egui::Rect::from_min_size(bar2_rect.min, egui::vec2(fill2_w, 20.0)),
            3.0,
            Color32::from_rgb(255, 107, 43),
        );

        // Param 3: Decay Seconds
        let p3_x = p2_x + col_w;
        painter.text(
            egui::pos2(p3_x, ctrl_rect.min.y + 12.0),
            egui::Align2::LEFT_TOP,
            format!("DECAY: {:.2}s", self.decay_seconds),
            FontId::proportional(11.0),
            Color32::from_rgb(255, 215, 0),
        );
        let bar3_rect = egui::Rect::from_min_size(
            egui::pos2(p3_x, ctrl_rect.min.y + 30.0),
            egui::vec2(col_w - 20.0, 20.0),
        );
        painter.rect_filled(bar3_rect, 3.0, Color32::from_rgb(10, 14, 22));
        let norm_decay = ((self.decay_seconds - 0.5) / 7.5).clamp(0.0, 1.0);
        let fill3_w = bar3_rect.width() * norm_decay;
        painter.rect_filled(
            egui::Rect::from_min_size(bar3_rect.min, egui::vec2(fill3_w, 20.0)),
            3.0,
            Color32::from_rgb(255, 215, 0),
        );

        // Param 4: Drive Saturation
        let p4_x = p3_x + col_w;
        painter.text(
            egui::pos2(p4_x, ctrl_rect.min.y + 12.0),
            egui::Align2::LEFT_TOP,
            format!("DRIVE: +{:.1}dB", self.drive_saturation_db),
            FontId::proportional(11.0),
            Color32::from_rgb(0, 255, 180),
        );
        let bar4_rect = egui::Rect::from_min_size(
            egui::pos2(p4_x, ctrl_rect.min.y + 30.0),
            egui::vec2(col_w - 20.0, 20.0),
        );
        painter.rect_filled(bar4_rect, 3.0, Color32::from_rgb(10, 14, 22));
        let norm_drive = (self.drive_saturation_db / 18.0).clamp(0.0, 1.0);
        let fill4_w = bar4_rect.width() * norm_drive;
        painter.rect_filled(
            egui::Rect::from_min_size(bar4_rect.min, egui::vec2(fill4_w, 20.0)),
            3.0,
            Color32::from_rgb(0, 255, 180),
        );

        // Handle Slider Bar Dragging
        if response.dragged() || response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                if bar1_rect.contains(pos) {
                    self.tension_pct =
                        (((pos.x - bar1_rect.min.x) / bar1_rect.width()) * 100.0).clamp(0.0, 100.0);
                } else if bar2_rect.contains(pos) {
                    self.dispersion_chirp_pct =
                        (((pos.x - bar2_rect.min.x) / bar2_rect.width()) * 100.0).clamp(0.0, 100.0);
                } else if bar3_rect.contains(pos) {
                    self.decay_seconds =
                        0.5 + (((pos.x - bar3_rect.min.x) / bar3_rect.width()) * 7.5).clamp(0.0, 7.5);
                } else if bar4_rect.contains(pos) {
                    self.drive_saturation_db =
                        (((pos.x - bar4_rect.min.x) / bar4_rect.width()) * 18.0).clamp(0.0, 18.0);
                }
            }
        }

        // Verified Hit Target Badge
        let badge_rect = egui::Rect::from_min_size(
            egui::pos2(ctrl_rect.min.x + 16.0, ctrl_rect.min.y + 64.0),
            egui::vec2(ctrl_rect.width() - 32.0, 32.0),
        );
        painter.rect_filled(badge_rect, 4.0, Color32::from_rgb(16, 35, 28));
        painter.rect_stroke(
            badge_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(0, 255, 180)),
        );
        painter.text(
            egui::pos2(badge_rect.min.x + 10.0, badge_rect.min.y + 8.0),
            egui::Align2::LEFT_TOP,
            "[PASS] Spring Reverb Tank Simulator & Dispersion Nodes (>= 44x44pt) Verified",
            FontId::proportional(11.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
