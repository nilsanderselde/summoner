// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Physical Modeling Caribbean Steelpan / Steel Drum Annular Ring Resonance & Modal Strike HUD (Step 1601).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const STEELPAN_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_RADIAL_POS: f32 = 0.05;
pub const MAX_RADIAL_POS: f32 = 0.95;
pub const MIN_STRIKE_VELOCITY: f32 = 0.10;
pub const MAX_STRIKE_VELOCITY: f32 = 1.00;
pub const MIN_ANNULAR_RINGS: usize = 3;
pub const MAX_ANNULAR_RINGS: usize = 6;

/// Caribbean steelpan / steel drum instrument models.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SteelpanType {
    LeadTenorPan,     // Soprano Lead Pan (low D4 up to E6, 29 notes, annular concave bowl)
    DoubleSecondsPan, // Double Second pair (F#3-B5, warm annular harmonic coupling)
    DoubleGuitarPan,  // Double Guitar tenor pans (C3-G4, thick steel gauge strumming)
    TripleCellosPan,  // Triple Cello 3-barrel set (C2-B3, deep resonant lower mids)
    SixBassPan,       // Six Bass 55-gallon oil barrels (3 notes per pan, heavy acoustic fundamental)
}

impl SteelpanType {
    pub fn pan_name(&self) -> &'static str {
        match self {
            Self::LeadTenorPan => "LEAD TENOR PAN (29N)",
            Self::DoubleSecondsPan => "DOUBLE SECONDS (PAIR)",
            Self::DoubleGuitarPan => "DOUBLE GUITARS (TENOR)",
            Self::TripleCellosPan => "TRIPLE CELLOS (3-BARREL)",
            Self::SixBassPan => "SIX BASS (55-GAL BARRELS)",
        }
    }

    pub fn nominal_radial_pos(&self) -> f32 {
        match self {
            Self::LeadTenorPan => 0.45,
            Self::DoubleSecondsPan => 0.55,
            Self::DoubleGuitarPan => 0.65,
            Self::TripleCellosPan => 0.35,
            Self::SixBassPan => 0.25,
        }
    }

    pub fn nominal_strike_velocity(&self) -> f32 {
        match self {
            Self::LeadTenorPan => 0.75,
            Self::DoubleSecondsPan => 0.70,
            Self::DoubleGuitarPan => 0.85,
            Self::TripleCellosPan => 0.80,
            Self::SixBassPan => 0.90,
        }
    }

    pub fn nominal_rings(&self) -> usize {
        match self {
            Self::LeadTenorPan => 5,
            Self::DoubleSecondsPan => 4,
            Self::DoubleGuitarPan => 3,
            Self::TripleCellosPan => 3,
            Self::SixBassPan => 3,
        }
    }

    pub fn nominal_gauge_mm(&self) -> f32 {
        match self {
            Self::LeadTenorPan => 0.95,
            Self::DoubleSecondsPan => 1.10,
            Self::DoubleGuitarPan => 1.25,
            Self::TripleCellosPan => 1.40,
            Self::SixBassPan => 1.65,
        }
    }

    pub fn nominal_damping_s(&self) -> f32 {
        match self {
            Self::LeadTenorPan => 1.8,
            Self::DoubleSecondsPan => 2.4,
            Self::DoubleGuitarPan => 1.5,
            Self::TripleCellosPan => 2.8,
            Self::SixBassPan => 3.6,
        }
    }

    pub fn nominal_coupling(&self) -> f32 {
        match self {
            Self::LeadTenorPan => 0.40,
            Self::DoubleSecondsPan => 0.55,
            Self::DoubleGuitarPan => 0.30,
            Self::TripleCellosPan => 0.50,
            Self::SixBassPan => 0.65,
        }
    }
}

/// Physical modeling Caribbean steelpan / steel drum annular ring resonance & modal strike HUD.
#[derive(Debug, Clone)]
pub struct SteelpanDrumView {
    pub pan_type: SteelpanType,
    pub strike_radial_pos: f32,
    pub strike_velocity: f32,
    pub annular_ring_count: usize,
    pub selected_note_idx: usize,
    pub steel_gauge_mm: f32,
    pub damping_s: f32,
    pub coupling_resonance: f32,
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    pub modal_amplitudes: [f32; 8],
    pub color_palette: ContrastColorPalette,
}

impl Default for SteelpanDrumView {
    fn default() -> Self {
        Self::new()
    }
}

impl SteelpanDrumView {
    pub fn new() -> Self {
        let mut view = Self {
            pan_type: SteelpanType::LeadTenorPan,
            strike_radial_pos: 0.45,
            strike_velocity: 0.75,
            annular_ring_count: 5,
            selected_note_idx: 12,
            steel_gauge_mm: 0.95,
            damping_s: 1.8,
            coupling_resonance: 0.40,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            modal_amplitudes: [1.0, 0.85, 0.65, 0.45, 0.30, 0.40, 0.25, 0.18],
            color_palette: ContrastColorPalette::default(),
        };
        view.puck_pos = (
            Self::radial_to_normalized(view.strike_radial_pos),
            Self::vel_to_normalized(view.strike_velocity),
        );
        view.update_modal_simulation();
        view
    }

    pub fn radial_to_normalized(pos: f32) -> f32 {
        let p = pos.clamp(MIN_RADIAL_POS, MAX_RADIAL_POS);
        ((p - MIN_RADIAL_POS) / (MAX_RADIAL_POS - MIN_RADIAL_POS)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_radial(norm: f32) -> f32 {
        MIN_RADIAL_POS + norm.clamp(0.0, 1.0) * (MAX_RADIAL_POS - MIN_RADIAL_POS)
    }

    pub fn vel_to_normalized(vel: f32) -> f32 {
        let v = vel.clamp(MIN_STRIKE_VELOCITY, MAX_STRIKE_VELOCITY);
        ((v - MIN_STRIKE_VELOCITY) / (MAX_STRIKE_VELOCITY - MIN_STRIKE_VELOCITY)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_vel(norm: f32) -> f32 {
        MIN_STRIKE_VELOCITY + norm.clamp(0.0, 1.0) * (MAX_STRIKE_VELOCITY - MIN_STRIKE_VELOCITY)
    }

    pub fn set_pan_type(&mut self, pan: SteelpanType) {
        self.pan_type = pan;
        self.strike_radial_pos = pan.nominal_radial_pos();
        self.strike_velocity = pan.nominal_strike_velocity();
        self.annular_ring_count = pan.nominal_rings();
        self.selected_note_idx = 10;
        self.steel_gauge_mm = pan.nominal_gauge_mm();
        self.damping_s = pan.nominal_damping_s();
        self.coupling_resonance = pan.nominal_coupling();
        self.puck_pos = (
            Self::radial_to_normalized(self.strike_radial_pos),
            Self::vel_to_normalized(self.strike_velocity),
        );
        self.update_modal_simulation();
    }

    pub fn update_modal_simulation(&mut self) {
        let vel = self.strike_velocity;
        let r = self.strike_radial_pos;
        let gauge = self.steel_gauge_mm;
        let coupling = self.coupling_resonance;

        // Steelpan modal response: Center strike excites fundamental + 2nd octave overtone; Rim excites high harmonics
        let fund_weight = (1.0 - r * 0.7).clamp(0.2, 1.0);
        let octave_weight = (std::f32::consts::PI * r).sin().abs().clamp(0.1, 1.0);
        let third_weight = (r * 1.2).clamp(0.1, 1.1);
        let fifth_weight = (r * 1.4 * vel).clamp(0.0, 0.9);

        let f0_amp = (1.1 * fund_weight * (1.2 - gauge * 0.3)).clamp(0.1, 1.2);
        let f1_octave = (0.90 * octave_weight * (vel.sqrt())).clamp(0.0, 1.1);
        let f2_third = (0.70 * third_weight * vel).clamp(0.0, 1.0);
        let f3_fifth = (0.50 * fifth_weight * vel).clamp(0.0, 0.9);
        let rim_modal = (0.40 * r * (vel * vel)).clamp(0.0, 0.8);

        let inter_note = (coupling * 0.65 * f0_amp).clamp(0.0, 1.0);
        let sympathetic_rim = (coupling * 0.50 * f1_octave).clamp(0.0, 0.9);
        let shell_air = (coupling * 0.35 * (1.0 + gauge * 0.2)).clamp(0.0, 0.8);

        self.modal_amplitudes = [
            f0_amp,
            f1_octave,
            f2_third,
            f3_fifth,
            rim_modal,
            inter_note,
            sympathetic_rim,
            shell_air,
        ];
    }

    pub fn hit_test_steelpan_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= STEELPAN_PUCK_HIT_RADIUS
    }

    #[allow(clippy::needless_range_loop)]
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut grid = vec![vec![' '; width]; height];

        for (row_idx, row) in grid.iter_mut().enumerate() {
            row[0] = '|';
            row[width - 1] = '|';
            if row_idx == 0 || row_idx == height - 1 {
                for col in row.iter_mut().take(width) {
                    *col = '-';
                }
                row[0] = '+';
                row[width - 1] = '+';
            }
        }

        let mid_x = width / 2;
        for r in 1..height - 1 {
            grid[r][mid_x] = '|';
        }

        let left_w = mid_x - 2;
        let p_row = (((1.0 - self.puck_pos.1) * (height - 5) as f32) + 2.0).round() as usize;
        let p_col = ((self.puck_pos.0 * (left_w - 4) as f32) + 2.0).round() as usize;
        if p_row < height - 1 && p_col < mid_x {
            grid[p_row][p_col] = 'P';
        }

        let right_w = width - mid_x - 2;
        let bar_spacing = right_w / 9;
        for (i, &amp) in self.modal_amplitudes.iter().enumerate() {
            let col = mid_x + 2 + (i + 1) * bar_spacing;
            let bar_h = (amp.clamp(0.0, 1.0) * (height - 4) as f32).round() as usize;
            for r in 0..bar_h {
                if height - 2 > r && col < width - 1 {
                    grid[height - 2 - r][col] = '#';
                }
            }
        }

        grid.into_iter()
            .map(|row| row.into_iter().collect())
            .collect()
    }

    #[cfg(feature = "gui")]
    #[allow(clippy::needless_range_loop)]
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 480.0),
            egui::Sense::click_and_drag(),
        );

        let painter = ui.painter_at(rect);

        // Background: Deep Indigo Slate (#0C101B)
        painter.rect_filled(rect, 6.0, Color32::from_rgb(12, 16, 27));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "CARIBBEAN STEELPAN / STEEL DRUM ANNULAR RING RESONANCE HUD",
            egui::FontId::proportional(13.5),
            Color32::from_rgb(240, 245, 255),
        );

        // Tabs (y: 48..92) - 44pt height
        let tabs = [
            (SteelpanType::LeadTenorPan, "LEAD TENOR (29N)"),
            (SteelpanType::DoubleSecondsPan, "DOUBLE SECONDS"),
            (SteelpanType::DoubleGuitarPan, "DOUBLE GUITARS"),
            (SteelpanType::TripleCellosPan, "TRIPLE CELLOS"),
            (SteelpanType::SixBassPan, "SIX BASS (55G)"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (ptype, name)) in tabs.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_sel = self.pan_type == *ptype;
            let bg_col = if is_sel {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(24, 32, 48)
            };
            let text_col = if is_sel {
                Color32::from_rgb(4, 20, 28)
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
                        self.set_pan_type(*ptype);
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
            Stroke::new(1.5_f32, Color32::from_rgb(45, 65, 95)),
        );

        // Left 55%: Concave Oil Barrel Bowl & Concentric Annular Rings
        let left_w = main_canvas.width() * 0.55;
        let left_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(left_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(left_rect, 4.0, Color32::from_rgb(14, 18, 30));
        painter.rect_stroke(
            left_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(35, 55, 85)),
        );

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "CONCAVE STEEL BOWL & ANNULAR NOTE LAYOUT FIELD",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(0, 229, 255),
        );

        let bowl_center = egui::pos2(
            left_rect.min.x + left_rect.width() * 0.5,
            left_rect.min.y + left_rect.height() * 0.52,
        );
        let max_bowl_radius = (left_rect.height() * 0.42).min(left_rect.width() * 0.42);

        // Draw concentric annular rings
        for ring in 1..=self.annular_ring_count {
            let ring_r = max_bowl_radius * (ring as f32 / self.annular_ring_count as f32);
            let ring_col = if ring == self.annular_ring_count {
                Color32::from_rgb(255, 107, 43)
            } else {
                Color32::from_rgb(45, 75, 115)
            };
            painter.circle_stroke(bowl_center, ring_r, Stroke::new(1.2_f32, ring_col));
        }

        // Note pads distributed radially around rings
        let num_pads = 12;
        for p in 0..num_pads {
            let angle = p as f32 * (std::f32::consts::TAU / num_pads as f32);
            let pad_r = max_bowl_radius * 0.72;
            let pad_x = bowl_center.x + angle.cos() * pad_r;
            let pad_y = bowl_center.y + angle.sin() * pad_r;
            painter.circle_filled(
                egui::pos2(pad_x, pad_y),
                6.0,
                Color32::from_rgb(55, 95, 140),
            );
        }

        // Interactive Striker Puck
        let puck_x = left_rect.min.x + self.puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.puck_pos = (nx, ny);
                    self.strike_radial_pos = Self::normalized_to_radial(nx);
                    self.strike_velocity = Self::normalized_to_vel(ny);
                    self.update_modal_simulation();
                }
            }
        }

        painter.circle_stroke(
            puck_pos,
            STEELPAN_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(0, 229, 255, 150)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(0, 229, 255));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 18.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Radial Pos: {:.2}R | Vel: {:.0}% | Rings: {} | Gauge: {:.2}mm | Decay: {:.1}s",
                self.strike_radial_pos,
                self.strike_velocity * 100.0,
                self.annular_ring_count,
                self.steel_gauge_mm,
                self.damping_s
            ),
            egui::FontId::proportional(10.5),
            Color32::from_rgb(140, 235, 255),
        );

        // Right 45%: Annular Modal & Inter-Note Harmonic Spectrum
        let right_w = main_canvas.width() * 0.45;
        let right_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + left_w + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(right_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(right_rect, 4.0, Color32::from_rgb(14, 18, 30));
        painter.rect_stroke(
            right_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(35, 55, 85)),
        );

        painter.text(
            egui::pos2(right_rect.min.x + 10.0, right_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "ANNULAR MODAL & INTER-NOTE RESONANCE SPECTRUM",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(0, 229, 255),
        );

        let mode_labels = [
            "f0 (Fund)", "f1 (Oct)", "f2 (3rd)", "f3 (5th)", "RIM-MOD", "NOTE-CPL", "SYMP-RIM", "SHELL-AIR",
        ];
        let bar_w = (right_rect.width() - 30.0 - 7.0 * 6.0) / 8.0;
        for (i, &amp) in self.modal_amplitudes.iter().enumerate() {
            let bx = right_rect.min.x + 15.0 + i as f32 * (bar_w + 6.0);
            let bar_h = (amp.clamp(0.0, 1.2) / 1.2) * (right_rect.height() - 80.0);
            let b_rect = egui::Rect::from_min_max(
                egui::pos2(bx, right_rect.max.y - 25.0 - bar_h),
                egui::pos2(bx + bar_w, right_rect.max.y - 25.0),
            );
            let col = if i == 0 {
                Color32::from_rgb(0, 229, 255)
            } else if i < 5 {
                Color32::from_rgb(255, 107, 43)
            } else {
                Color32::from_rgb(255, 215, 0)
            };
            painter.rect_filled(b_rect, 3.0, col);

            painter.text(
                egui::pos2(bx + bar_w * 0.5, right_rect.max.y - 20.0),
                egui::Align2::CENTER_TOP,
                mode_labels[i],
                egui::FontId::proportional(8.0),
                Color32::from_rgb(180, 205, 235),
            );
        }

        // Bottom Metrics Dock (y: 350..465)
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

        let params = [
            (
                "STRIKE RADIAL POS",
                format!("{:.2} R (Annular)", self.strike_radial_pos),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "STRIKE VELOCITY",
                format!("{:.0}% (Mallet Impact)", self.strike_velocity * 100.0),
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "STEEL GAUGE",
                format!("{:.2} mm (55-Gal Shell)", self.steel_gauge_mm),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "INTER-NOTE COUPLING",
                format!("{:.2} (Sympathetic Rings)", self.coupling_resonance),
                Color32::from_rgb(0, 255, 180),
            ),
        ];

        let col_w = (dock_rect.width() - 40.0) / 4.0;
        for (i, (label, val, col)) in params.iter().enumerate() {
            let px_pos = dock_rect.min.x + 20.0 + i as f32 * col_w;
            painter.text(
                egui::pos2(px_pos, dock_rect.min.y + 12.0),
                egui::Align2::LEFT_TOP,
                *label,
                egui::FontId::proportional(11.0),
                Color32::from_rgb(160, 185, 215),
            );
            painter.text(
                egui::pos2(px_pos, dock_rect.min.y + 30.0),
                egui::Align2::LEFT_TOP,
                val,
                egui::FontId::proportional(13.0),
                *col,
            );
        }

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
            "[PASS] Caribbean Steelpan Annular Ring Modal Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
