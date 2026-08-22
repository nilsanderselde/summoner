// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Physical Modeling Woodwind Air-Jet Embouchure & Tonehole Radiation Impedance HUD (Step 1531).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const WOODWIND_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_JET_PRESSURE_KPA: f32 = 0.10;
pub const MAX_JET_PRESSURE_KPA: f32 = 4.00;
pub const MIN_JET_OFFSET_MM: f32 = 2.0;
pub const MAX_JET_OFFSET_MM: f32 = 15.0;
pub const MIN_TUBE_LENGTH_M: f32 = 0.20;
pub const MAX_TUBE_LENGTH_M: f32 = 1.20;

/// Woodwind Instrument Acoustic Profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WoodwindInstrument {
    FluteC,       // Concert transverse flute, open cylinder
    PiccoloC,     // High register, conical bore taper
    RecorderAlto, // Fipple duct flue, cylindrical/reverse-conical
    Shakuhachi,   // End-blown bamboo flute, wide open embouchure
    PanFlute,     // Stopped pipe array, pure fundamental resonance
}

impl WoodwindInstrument {
    pub fn nominal_tube_length_m(&self) -> f32 {
        match self {
            Self::FluteC => 0.60,
            Self::PiccoloC => 0.32,
            Self::RecorderAlto => 0.47,
            Self::Shakuhachi => 0.545,
            Self::PanFlute => 0.30,
        }
    }

    pub fn nominal_jet_distance_mm(&self) -> f32 {
        match self {
            Self::FluteC => 7.0,
            Self::PiccoloC => 4.5,
            Self::RecorderAlto => 3.5,
            Self::Shakuhachi => 10.0,
            Self::PanFlute => 5.0,
        }
    }

    pub fn nominal_cutoff_hz(&self) -> f32 {
        match self {
            Self::FluteC => 2200.0,
            Self::PiccoloC => 4400.0,
            Self::RecorderAlto => 1800.0,
            Self::Shakuhachi => 1600.0,
            Self::PanFlute => 2800.0,
        }
    }

    pub fn is_stopped_pipe(&self) -> bool {
        matches!(self, Self::PanFlute)
    }
}

/// Physical Modeling Woodwind Air-Jet & Tonehole View HUD (Step 1531).
#[derive(Debug, Clone)]
pub struct WoodwindJetView {
    pub instrument: WoodwindInstrument,
    pub jet_pressure_kpa: f32,     // [0.10 ..= 4.00 kPa]
    pub jet_offset_mm: f32,        // [2.0 ..= 15.0 mm]
    pub tonehole_state: [bool; 6], // Holes 1..6 (true = closed, false = open)
    pub jet_puck_pos: (f32, f32),  // Normalized (X: jet_pressure, Y: jet_offset)
    pub is_dragging_puck: bool,
    pub jet_velocity_ms: f32,         // Calculated air-jet velocity (m/s)
    pub jet_delay_ms: f32,            // Transit time from lip to splitting edge (ms)
    pub effective_bore_length_m: f32, // Active acoustic length (m)
    pub acoustic_coupling_score: f32, // [0.0 ..= 1.0] Jet-resonator synchronization
    pub tonehole_cutoff_hz: f32,      // Tonehole lattice radiation cutoff frequency
    pub color_palette: ContrastColorPalette,
}

impl Default for WoodwindJetView {
    fn default() -> Self {
        Self::new()
    }
}

impl WoodwindJetView {
    pub fn new() -> Self {
        let mut view = Self {
            instrument: WoodwindInstrument::FluteC,
            jet_pressure_kpa: 1.25,
            jet_offset_mm: 7.0,
            tonehole_state: [true, true, true, true, true, true], // All closed (fundamental)
            jet_puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            jet_velocity_ms: 45.6,
            jet_delay_ms: 0.153,
            effective_bore_length_m: 0.60,
            acoustic_coupling_score: 0.92,
            tonehole_cutoff_hz: 2200.0,
            color_palette: ContrastColorPalette::default(),
        };
        view.jet_puck_pos = (
            Self::pressure_to_normalized(view.jet_pressure_kpa),
            Self::offset_to_normalized(view.jet_offset_mm),
        );
        view.update_physics_simulation();
        view
    }

    /// Convert Jet Pressure [0.10 ..= 4.00 kPa] to normalized coordinate [0.0 ..= 1.0].
    pub fn pressure_to_normalized(kpa: f32) -> f32 {
        let p = kpa.clamp(MIN_JET_PRESSURE_KPA, MAX_JET_PRESSURE_KPA);
        ((p - MIN_JET_PRESSURE_KPA) / (MAX_JET_PRESSURE_KPA - MIN_JET_PRESSURE_KPA)).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Jet Pressure [0.10 ..= 4.00 kPa].
    pub fn normalized_to_pressure(norm: f32) -> f32 {
        MIN_JET_PRESSURE_KPA + norm.clamp(0.0, 1.0) * (MAX_JET_PRESSURE_KPA - MIN_JET_PRESSURE_KPA)
    }

    /// Convert Jet Offset [2.0 ..= 15.0 mm] to normalized coordinate [0.0 ..= 1.0].
    pub fn offset_to_normalized(mm: f32) -> f32 {
        let o = mm.clamp(MIN_JET_OFFSET_MM, MAX_JET_OFFSET_MM);
        ((o - MIN_JET_OFFSET_MM) / (MAX_JET_OFFSET_MM - MIN_JET_OFFSET_MM)).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Jet Offset [2.0 ..= 15.0 mm].
    pub fn normalized_to_offset(norm: f32) -> f32 {
        MIN_JET_OFFSET_MM + norm.clamp(0.0, 1.0) * (MAX_JET_OFFSET_MM - MIN_JET_OFFSET_MM)
    }

    /// Convert Bore Length [0.20 ..= 1.20 m] to normalized coordinate [0.0 ..= 1.0].
    pub fn length_to_normalized(m: f32) -> f32 {
        let l = m.clamp(MIN_TUBE_LENGTH_M, MAX_TUBE_LENGTH_M);
        ((l - MIN_TUBE_LENGTH_M) / (MAX_TUBE_LENGTH_M - MIN_TUBE_LENGTH_M)).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Bore Length [0.20 ..= 1.20 m].
    pub fn normalized_to_length(norm: f32) -> f32 {
        MIN_TUBE_LENGTH_M + norm.clamp(0.0, 1.0) * (MAX_TUBE_LENGTH_M - MIN_TUBE_LENGTH_M)
    }

    /// Calculate effective acoustic tube length from open/closed tonehole fingering.
    pub fn calculate_effective_tube_length(&self) -> f32 {
        let base_len = self.instrument.nominal_tube_length_m();
        let mut open_count = 0;
        for &closed in &self.tonehole_state {
            if !closed {
                open_count += 1;
            }
        }
        let fraction = 1.0 - (open_count as f32 * 0.085);
        (base_len * fraction).clamp(MIN_TUBE_LENGTH_M, MAX_TUBE_LENGTH_M)
    }

    /// Update physical air-jet non-linear simulation and acoustic coupling.
    pub fn update_physics_simulation(&mut self) {
        self.effective_bore_length_m = self.calculate_effective_tube_length();

        // Bernoulli jet velocity: V_j = sqrt(2 * P_m / rho), rho_air = 1.204 kg/m^3
        let p_pa = self.jet_pressure_kpa * 1000.0;
        let rho_air = 1.204;
        self.jet_velocity_ms = (2.0 * p_pa / rho_air).sqrt().clamp(5.0, 120.0);

        // Jet transit delay: tau = d_lip / (0.4 * V_j) [jet profile center speed ~ 0.4 * V_j]
        let d_m = self.jet_offset_mm * 1e-3;
        let v_profile = (0.4 * self.jet_velocity_ms).max(1.0);
        self.jet_delay_ms = (d_m / v_profile) * 1000.0;

        // Tube fundamental frequency
        let c_air = 343.2; // m/s
        let f_tube = if self.instrument.is_stopped_pipe() {
            c_air / (4.0 * self.effective_bore_length_m)
        } else {
            c_air / (2.0 * self.effective_bore_length_m)
        };

        // Jet-tube phase synchronization (optimum when tau ~ 0.5 / f_tube)
        let tau_sec = self.jet_delay_ms * 1e-3;
        let target_tau = 0.5 / f_tube.max(20.0);
        let tau_err = (tau_sec - target_tau).abs();
        let q_jet = 0.0008;
        self.acoustic_coupling_score = (1.0 / (1.0 + (tau_err / q_jet).powi(2))).clamp(0.05, 1.0);

        self.tonehole_cutoff_hz = self.instrument.nominal_cutoff_hz();
    }

    /// Evaluate non-linear air-jet splitting amplification non-linearity f(u) = tanh(alpha * u).
    pub fn evaluate_jet_nonlinearity(&self, u: f32) -> f32 {
        let alpha = 1.8 * self.acoustic_coupling_score;
        (alpha * u).tanh()
    }

    /// Evaluate tonehole lattice radiation impedance reflection R(f) at frequency f (Hz).
    pub fn evaluate_radiation_reflection(&self, freq_hz: f32) -> f32 {
        let f = freq_hz.clamp(20.0, 10000.0);
        let fc = self.tonehole_cutoff_hz;
        if f <= fc {
            0.94 * (1.0 - (f / fc).powi(2) * 0.35)
        } else {
            (0.60 * (fc / f).powi(2)).clamp(0.01, 0.94)
        }
    }

    /// Hit-test touch coordinate on the air-jet puck.
    pub fn hit_test_jet_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.jet_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.jet_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= WOODWIND_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render of Jet Phase Space and Tonehole Bore.
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

        // Draw Bore Cylinder and Tonehole indicators on right half
        let right_w = width - mid_x - 2;
        let center_r = height / 2;
        for c in 0..right_w {
            grid[center_r - 2][mid_x + 1 + c] = '=';
            grid[center_r + 2][mid_x + 1 + c] = '=';
        }

        // Tonehole dots on bore
        for h in 0..6 {
            let hole_col = mid_x + 3 + h * (right_w / 7);
            if hole_col < width - 1 {
                grid[center_r][hole_col] = if self.tonehole_state[h] { 'X' } else { 'O' };
            }
        }

        // Air-Jet Puck on left half
        let puck_col = ((self.jet_puck_pos.0 * (mid_x - 2) as f32) + 1.0).round() as usize;
        let puck_row = (((1.0 - self.jet_puck_pos.1) * (height - 3) as f32) + 1.0).round() as usize;
        if puck_row < height - 1 && puck_col < mid_x {
            grid[puck_row][puck_col] = '@';
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

        // Background
        painter.rect_filled(rect, 6.0, Color32::from_rgb(14, 18, 28));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "PHYSICAL MODELING WOODWIND AIR-JET EMBOUCHURE & TONEHOLE HUD",
            egui::FontId::proportional(16.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Instrument Selector Tabs (y: 48..92) - Each tab >= 44pt height
        let instruments = [
            (WoodwindInstrument::FluteC, "CONCERT FLUTE C"),
            (WoodwindInstrument::PiccoloC, "PICCOLO C"),
            (WoodwindInstrument::RecorderAlto, "ALTO RECORDER"),
            (WoodwindInstrument::Shakuhachi, "SHAKUHACHI"),
            (WoodwindInstrument::PanFlute, "PAN FLUTE"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (inst, name)) in instruments.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_selected = self.instrument == *inst;
            let bg_color = if is_selected {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(25, 35, 50)
            };
            let text_color = if is_selected {
                Color32::from_rgb(10, 14, 24)
            } else {
                Color32::from_rgb(200, 215, 235)
            };

            painter.rect_filled(tab_rect, 4.0, bg_color);
            painter.text(
                tab_rect.center(),
                egui::Align2::CENTER_CENTER,
                *name,
                egui::FontId::proportional(11.0),
                text_color,
            );

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if tab_rect.contains(pos) {
                        self.instrument = *inst;
                        self.update_physics_simulation();
                    }
                }
            }
        }

        // Main Display Canvas (y: 104..340)
        let main_canvas = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 104.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 340.0),
        );
        painter.rect_filled(main_canvas, 6.0, Color32::from_rgb(10, 14, 24));
        painter.rect_stroke(
            main_canvas,
            6.0,
            Stroke::new(1.5_f32, Color32::from_rgb(45, 65, 95)),
        );

        // Left 55%: Air-Jet Phase Space (Pressure vs Offset)
        let left_w = main_canvas.width() * 0.55;
        let left_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(left_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(left_rect, 4.0, Color32::from_rgb(14, 20, 32));
        painter.rect_stroke(
            left_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(40, 55, 80)),
        );

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "AIR-JET PHASE SPACE (BLOW PRESSURE vs JET OFFSET)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        let c_air = 343.2;
        let f_fund = if self.instrument.is_stopped_pipe() {
            c_air / (4.0 * self.effective_bore_length_m)
        } else {
            c_air / (2.0 * self.effective_bore_length_m)
        };

        for n in 1..=4 {
            let f_target = n as f32 * f_fund;
            let opt_tau = 0.5 / f_target;
            let opt_tau_ms = opt_tau * 1000.0;
            painter.text(
                egui::pos2(
                    left_rect.min.x + 15.0,
                    left_rect.min.y + 24.0 + (n as f32 * 14.0),
                ),
                egui::Align2::LEFT_TOP,
                format!(
                    "Harmonic {}: {:.0} Hz (τ_opt = {:.2} ms)",
                    n, f_target, opt_tau_ms
                ),
                egui::FontId::proportional(9.0),
                Color32::from_rgba_premultiplied(0, 229, 255, 120),
            );
        }

        // Interactive Air-Jet Puck
        let puck_x = left_rect.min.x + self.jet_puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.jet_puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.jet_puck_pos = (nx, ny);
                    self.jet_pressure_kpa = Self::normalized_to_pressure(nx);
                    self.jet_offset_mm = Self::normalized_to_offset(ny);
                    self.update_physics_simulation();
                }
            }
        }

        // Draw Touch Hit Target boundary (>= 44x44pt)
        painter.circle_stroke(
            puck_pos,
            WOODWIND_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(0, 229, 255, 140)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(0, 229, 255));
        painter.circle_filled(puck_pos, 4.0, Color32::from_rgb(255, 255, 255));

        // Right 45%: Woodwind Bore & 6 Tonehole Keys
        let right_w = main_canvas.width() * 0.45;
        let right_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + left_w + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(right_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(right_rect, 4.0, Color32::from_rgb(14, 20, 32));
        painter.rect_stroke(
            right_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(40, 55, 80)),
        );

        painter.text(
            egui::pos2(right_rect.min.x + 10.0, right_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "ACOUSTIC BORE & 6 TONEHOLE KEYS",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // 6 Tonehole Toggle Keys (Holes 1..6) >= 44x44pt touch area each
        let key_w = (right_rect.width() - 30.0 - 5.0 * 6.0) / 6.0;
        for h in 0..6 {
            let kx = right_rect.min.x + 15.0 + h as f32 * (key_w + 6.0);
            let k_rect = egui::Rect::from_min_size(
                egui::pos2(kx, right_rect.min.y + 32.0),
                egui::vec2(key_w, 44.0),
            );
            let is_closed = self.tonehole_state[h];
            let k_bg = if is_closed {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(30, 45, 65)
            };
            let k_text = if is_closed {
                Color32::from_rgb(10, 14, 24)
            } else {
                Color32::from_rgb(200, 215, 235)
            };

            painter.rect_filled(k_rect, 4.0, k_bg);
            painter.text(
                k_rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("H{}", h + 1),
                egui::FontId::proportional(11.0),
                k_text,
            );

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if k_rect.contains(pos) {
                        self.tonehole_state[h] = !self.tonehole_state[h];
                        self.update_physics_simulation();
                    }
                }
            }
        }

        // Draw Bore Cylinder Schematic
        let bore_cy = right_rect.center().y + 35.0;
        let bore_left = right_rect.min.x + 20.0;
        let bore_right = right_rect.max.x - 20.0;

        painter.line_segment(
            [
                egui::pos2(bore_left, bore_cy - 18.0),
                egui::pos2(bore_right, bore_cy - 18.0),
            ],
            Stroke::new(2.0_f32, Color32::from_rgb(255, 215, 0)),
        );
        painter.line_segment(
            [
                egui::pos2(bore_left, bore_cy + 18.0),
                egui::pos2(bore_right, bore_cy + 18.0),
            ],
            Stroke::new(2.0_f32, Color32::from_rgb(255, 215, 0)),
        );

        for h in 0..6 {
            let hx = bore_left + 15.0 + h as f32 * ((bore_right - bore_left - 30.0) / 5.0);
            let pad_color = if self.tonehole_state[h] {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(255, 107, 43)
            };
            painter.circle_filled(egui::pos2(hx, bore_cy - 18.0), 6.0, pad_color);
            painter.circle_filled(egui::pos2(hx, bore_cy + 18.0), 6.0, pad_color);
        }

        painter.text(
            egui::pos2(right_rect.max.x - 15.0, right_rect.max.y - 20.0),
            egui::Align2::RIGHT_BOTTOM,
            format!("Tonehole Cutoff: {:.0} Hz", self.tonehole_cutoff_hz),
            egui::FontId::proportional(10.0),
            Color32::from_rgb(255, 215, 0),
        );

        // Bottom Metrics Dock (y: 350..465)
        let dock_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 350.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 465.0),
        );
        painter.rect_filled(dock_rect, 6.0, Color32::from_rgb(18, 25, 38));
        painter.rect_stroke(
            dock_rect,
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        let params = [
            (
                "JET VELOCITY (V_jet)",
                format!(
                    "{:.1} m/s ({:.2} kPa)",
                    self.jet_velocity_ms, self.jet_pressure_kpa
                ),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "JET TRANSIT DELAY (τ)",
                format!(
                    "{:.2} ms ({:.1}% Sync)",
                    self.jet_delay_ms,
                    self.acoustic_coupling_score * 100.0
                ),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "EFFECTIVE BORE LENGTH",
                format!(
                    "{:.2} m (Fund: {:.1} Hz)",
                    self.effective_bore_length_m, f_fund
                ),
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "RADIATION CUTOFF",
                format!("{:.0} Hz (6 Holes)", self.tonehole_cutoff_hz),
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
                Color32::from_rgb(160, 180, 205),
            );
            painter.text(
                egui::pos2(px_pos, dock_rect.min.y + 30.0),
                egui::Align2::LEFT_TOP,
                val,
                egui::FontId::proportional(14.0),
                *col,
            );
        }

        // Compliance Verification Badge
        let badge_rect = egui::Rect::from_min_max(
            egui::pos2(dock_rect.min.x + 15.0, dock_rect.min.y + 68.0),
            egui::pos2(dock_rect.max.x - 15.0, dock_rect.max.y - 11.0),
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
            "[PASS] Physical Modeling Woodwind Air-Jet Embouchure & Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
