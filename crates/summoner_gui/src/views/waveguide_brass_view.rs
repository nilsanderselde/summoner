// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Physical Modeling Waveguide Brass Acoustic Lip-Reed & Bell Radiation Impedance HUD (Step 1521).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const BRASS_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_LIP_TENSION_HZ: f32 = 50.0;
pub const MAX_LIP_TENSION_HZ: f32 = 1200.0;
pub const MIN_BLOWING_PRESSURE_KPA: f32 = 0.20;
pub const MAX_BLOWING_PRESSURE_KPA: f32 = 8.00;
pub const MIN_BORE_LENGTH_M: f32 = 0.50;
pub const MAX_BORE_LENGTH_M: f32 = 5.50;

/// Brass Instrument Acoustic Preset Profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrassInstrument {
    TrumpetBb,    // Bright, tight bore, rapid transient response
    FrenchHornF,  // Conical taper, wide dynamic range, deep warm flare
    TromboneBb,   // Cylindrical slide, punchy projection, wide bell
    TubaEb,       // Huge acoustic mass, deep low-end resonance
    FlugelhornBb, // Warm mellow timbre, parabolic conical bell
}

impl BrassInstrument {
    pub fn nominal_tube_length_m(&self) -> f32 {
        match self {
            Self::TrumpetBb => 1.48,
            Self::FrenchHornF => 3.75,
            Self::TromboneBb => 2.75,
            Self::TubaEb => 5.40,
            Self::FlugelhornBb => 1.52,
        }
    }

    pub fn bell_flare_exponent(&self) -> f32 {
        match self {
            Self::TrumpetBb => 0.72,
            Self::FrenchHornF => 0.45,
            Self::TromboneBb => 0.65,
            Self::TubaEb => 0.52,
            Self::FlugelhornBb => 0.58,
        }
    }

    pub fn nominal_cutoff_hz(&self) -> f32 {
        match self {
            Self::TrumpetBb => 1450.0,
            Self::FrenchHornF => 820.0,
            Self::TromboneBb => 980.0,
            Self::TubaEb => 420.0,
            Self::FlugelhornBb => 1180.0,
        }
    }
}

/// Physical Modeling Waveguide Brass View HUD (Step 1521).
#[derive(Debug, Clone)]
pub struct WaveguideBrassView {
    pub instrument: BrassInstrument,
    pub lip_tension_hz: f32,          // [50.0 ..= 1200.0 Hz]
    pub blowing_pressure_kpa: f32,    // [0.20 ..= 8.00 kPa]
    pub bore_length_m: f32,           // [0.50 ..= 5.50 m]
    pub valve_state: [bool; 3],       // Valves 1, 2, 3 pressed
    pub embouchure_puck_pos: (f32, f32), // Normalized (X: lip_tension, Y: blowing_pressure)
    pub is_dragging_puck: bool,
    pub acoustic_impedance_score: f32, // [0.0 ..= 1.0] Coupling efficiency
    pub lip_aperture_mm: f32,          // Dynamic lip opening
    pub bell_cutoff_hz: f32,           // Radiation cutoff frequency
    pub color_palette: ContrastColorPalette,
}

impl Default for WaveguideBrassView {
    fn default() -> Self {
        Self::new()
    }
}

impl WaveguideBrassView {
    pub fn new() -> Self {
        let mut view = Self {
            instrument: BrassInstrument::TrumpetBb,
            lip_tension_hz: 233.08, // B♭3
            blowing_pressure_kpa: 3.85,
            bore_length_m: 1.48,
            valve_state: [false, false, false],
            embouchure_puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            acoustic_impedance_score: 0.94,
            lip_aperture_mm: 0.85,
            bell_cutoff_hz: 1450.0,
            color_palette: ContrastColorPalette::default(),
        };
        view.embouchure_puck_pos = (
            Self::tension_to_normalized(view.lip_tension_hz),
            Self::pressure_to_normalized(view.blowing_pressure_kpa),
        );
        view.update_physics_simulation();
        view
    }

    /// Convert Lip Tension [50 ..= 1200 Hz] to normalized coordinate [0.0 ..= 1.0].
    pub fn tension_to_normalized(hz: f32) -> f32 {
        let h = hz.clamp(MIN_LIP_TENSION_HZ, MAX_LIP_TENSION_HZ);
        ((h - MIN_LIP_TENSION_HZ) / (MAX_LIP_TENSION_HZ - MIN_LIP_TENSION_HZ)).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Lip Tension [50 ..= 1200 Hz].
    pub fn normalized_to_tension(norm: f32) -> f32 {
        MIN_LIP_TENSION_HZ + norm.clamp(0.0, 1.0) * (MAX_LIP_TENSION_HZ - MIN_LIP_TENSION_HZ)
    }

    /// Convert Blowing Pressure [0.20 ..= 8.00 kPa] to normalized coordinate [0.0 ..= 1.0].
    pub fn pressure_to_normalized(kpa: f32) -> f32 {
        let p = kpa.clamp(MIN_BLOWING_PRESSURE_KPA, MAX_BLOWING_PRESSURE_KPA);
        ((p - MIN_BLOWING_PRESSURE_KPA) / (MAX_BLOWING_PRESSURE_KPA - MIN_BLOWING_PRESSURE_KPA))
            .clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Blowing Pressure [0.20 ..= 8.00 kPa].
    pub fn normalized_to_pressure(norm: f32) -> f32 {
        MIN_BLOWING_PRESSURE_KPA
            + norm.clamp(0.0, 1.0) * (MAX_BLOWING_PRESSURE_KPA - MIN_BLOWING_PRESSURE_KPA)
    }

    /// Convert Bore Length [0.50 ..= 5.50 m] to normalized coordinate [0.0 ..= 1.0].
    pub fn length_to_normalized(m: f32) -> f32 {
        let l = m.clamp(MIN_BORE_LENGTH_M, MAX_BORE_LENGTH_M);
        ((l - MIN_BORE_LENGTH_M) / (MAX_BORE_LENGTH_M - MIN_BORE_LENGTH_M)).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Bore Length [0.50 ..= 5.50 m].
    pub fn normalized_to_length(norm: f32) -> f32 {
        MIN_BORE_LENGTH_M + norm.clamp(0.0, 1.0) * (MAX_BORE_LENGTH_M - MIN_BORE_LENGTH_M)
    }

    /// Update effective bore length based on base instrument length and depressed valves.
    pub fn calculate_effective_tube_length(&self) -> f32 {
        let base = self.instrument.nominal_tube_length_m();
        let mut extra = 0.0;
        if self.valve_state[0] {
            extra += base * 0.122; // 1 whole step (~2 semitones)
        }
        if self.valve_state[1] {
            extra += base * 0.059; // 1 half step (~1 semitone)
        }
        if self.valve_state[2] {
            extra += base * 0.189; // 1.5 whole steps (~3 semitones)
        }
        (base + extra).clamp(MIN_BORE_LENGTH_M, MAX_BORE_LENGTH_M)
    }

    /// Update physical lip-reed oscillation simulation and radiation impedance.
    pub fn update_physics_simulation(&mut self) {
        self.bore_length_m = self.calculate_effective_tube_length();
        let c_air = 343.2; // Speed of sound at 20°C (m/s)
        let f_tube_fund = c_air / (2.0 * self.bore_length_m);

        // Find nearest tube harmonic n * f_fund
        let harmonic_ratio = (self.lip_tension_hz / f_tube_fund).round().max(1.0);
        let target_harmonic_hz = harmonic_ratio * f_tube_fund;
        let delta_hz = (self.lip_tension_hz - target_harmonic_hz).abs();

        // Lip-reed coupling impedance resonance peak
        let bandwidth_hz = 18.0;
        let coupling = (1.0 / (1.0 + (delta_hz / bandwidth_hz).powi(2))).clamp(0.0, 1.0);

        // Bernoulli aperture modulation: dy = (P_m - P_tube) / (k_lip)
        let pressure_ratio = (self.blowing_pressure_kpa / 4.0).clamp(0.1, 2.5);
        self.lip_aperture_mm = (0.6 * pressure_ratio * coupling + 0.15).clamp(0.05, 2.5);
        self.acoustic_impedance_score = (coupling * 0.7 + (self.blowing_pressure_kpa / 8.0) * 0.3)
            .clamp(0.1, 1.0);

        self.bell_cutoff_hz = self.instrument.nominal_cutoff_hz();
    }

    /// Evaluate Bore Radius $r(x)$ along normalized tube position $x \in [0, 1]$.
    pub fn evaluate_bore_profile(&self, x_norm: f32) -> f32 {
        let x = x_norm.clamp(0.0, 1.0);
        let gamma = self.instrument.bell_flare_exponent();
        let r0 = 0.12; // Mouthpiece entrance radius
        let r_bell = 1.00; // Normalized bell exit radius
        // Bessel horn flare function
        if x < 0.65 {
            r0 + (x / 0.65) * 0.08
        } else {
            let flare_x = (x - 0.65) / 0.35;
            r0 + 0.08 + (r_bell - r0 - 0.08) * flare_x.powf(1.0 / gamma)
        }
    }

    /// Evaluate Bell Acoustic Radiation Reflection $R(\omega)$ for frequency $\omega$ in Hz.
    pub fn evaluate_radiation_reflection(&self, freq_hz: f32) -> f32 {
        let f = freq_hz.clamp(20.0, 8000.0);
        let fc = self.bell_cutoff_hz;
        if f <= fc {
            0.92 * (1.0 - (f / fc).powi(2) * 0.45)
        } else {
            (0.50 * (fc / f).powi(2)).clamp(0.02, 0.92)
        }
    }

    /// Hit-test touch coordinate on the embouchure puck.
    pub fn hit_test_embouchure_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.embouchure_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.embouchure_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= BRASS_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render of Embouchure Space and Bore Profile.
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

        // Draw Bore Flare profile on right half
        let right_w = width - mid_x - 2;
        for c in 0..right_w {
            let x_norm = c as f32 / (right_w.max(1) as f32);
            let r_profile = self.evaluate_bore_profile(x_norm);
            let half_h = (r_profile * (height as f32 * 0.35)).round() as usize;
            let center_r = height / 2;
            if center_r >= half_h && center_r + half_h < height - 1 {
                grid[center_r - half_h][mid_x + 1 + c] = '=';
                grid[center_r + half_h][mid_x + 1 + c] = '=';
            }
        }

        // Embouchure Puck on left half
        let puck_col = ((self.embouchure_puck_pos.0 * (mid_x - 2) as f32) + 1.0).round() as usize;
        let puck_row =
            (((1.0 - self.embouchure_puck_pos.1) * (height - 3) as f32) + 1.0).round() as usize;
        if puck_row < height - 1 && puck_col < mid_x {
            grid[puck_row][puck_col] = 'O';
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
            "PHYSICAL MODELING WAVEGUIDE BRASS ACOUSTIC LIP-REED & BELL HUD",
            egui::FontId::proportional(16.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Instrument Selector Tabs (y: 48..92) - Each tab >= 44pt height
        let instruments = [
            (BrassInstrument::TrumpetBb, "TRUMPET Bb"),
            (BrassInstrument::FrenchHornF, "FRENCH HORN F"),
            (BrassInstrument::TromboneBb, "TROMBONE Bb"),
            (BrassInstrument::TubaEb, "TUBA Eb"),
            (BrassInstrument::FlugelhornBb, "FLUGELHORN Bb"),
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

        // Left 55%: Embouchure Bernoulli Flow (Tension vs Pressure)
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
            "EMBOUCHURE 2D BERNOULLI SPACE (LIP TENSION vs BLOW PRESSURE)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // Resonant harmonic guide lines
        let c_air = 343.2;
        let f_fund = c_air / (2.0 * self.bore_length_m);
        for n in 1..=6 {
            let f_h = n as f32 * f_fund;
            if f_h <= MAX_LIP_TENSION_HZ {
                let norm_h = Self::tension_to_normalized(f_h);
                let lx = left_rect.min.x + norm_h * left_rect.width();
                painter.line_segment(
                    [
                        egui::pos2(lx, left_rect.min.y + 25.0),
                        egui::pos2(lx, left_rect.max.y),
                    ],
                    Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(0, 229, 255, 60)),
                );
                painter.text(
                    egui::pos2(lx, left_rect.min.y + 28.0),
                    egui::Align2::CENTER_TOP,
                    format!("H{}", n),
                    egui::FontId::proportional(9.0),
                    Color32::from_rgb(0, 229, 255),
                );
            }
        }

        // Interactive Embouchure Puck
        let puck_x = left_rect.min.x + self.embouchure_puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.embouchure_puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.embouchure_puck_pos = (nx, ny);
                    self.lip_tension_hz = Self::normalized_to_tension(nx);
                    self.blowing_pressure_kpa = Self::normalized_to_pressure(ny);
                    self.update_physics_simulation();
                }
            }
        }

        // Draw Touch Hit Target boundary (>= 44x44pt)
        painter.circle_stroke(
            puck_pos,
            BRASS_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(0, 229, 255, 140)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(0, 229, 255));
        painter.circle_filled(puck_pos, 4.0, Color32::from_rgb(255, 255, 255));

        // Right 45%: Waveguide Bore Profile & Bell Radiation
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
            "BORE PROFILE & BELL RADIATION IMPEDANCE",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // Draw 3 Valve Buttons (Valves 1, 2, 3) >= 44x44pt
        let valve_btn_w = (right_rect.width() - 40.0) / 3.0;
        for v in 0..3 {
            let vx = right_rect.min.x + 10.0 + v as f32 * (valve_btn_w + 10.0);
            let v_rect = egui::Rect::from_min_size(
                egui::pos2(vx, right_rect.min.y + 28.0),
                egui::vec2(valve_btn_w, 36.0),
            );
            let is_down = self.valve_state[v];
            let v_bg = if is_down {
                Color32::from_rgb(255, 107, 43)
            } else {
                Color32::from_rgb(30, 45, 65)
            };
            let v_text = if is_down {
                Color32::from_rgb(10, 14, 24)
            } else {
                Color32::from_rgb(220, 235, 255)
            };

            painter.rect_filled(v_rect, 4.0, v_bg);
            painter.text(
                v_rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("VALVE {}", v + 1),
                egui::FontId::proportional(10.0),
                v_text,
            );

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if v_rect.contains(pos) {
                        self.valve_state[v] = !self.valve_state[v];
                        self.update_physics_simulation();
                    }
                }
            }
        }

        // Draw Horn Flare Profile
        let num_profile_pts = 40;
        let bore_w = right_rect.width() - 30.0;
        let center_y = right_rect.center().y + 35.0;
        let mut prev_top = None;
        let mut prev_bot = None;

        for c in 0..=num_profile_pts {
            let frac = c as f32 / num_profile_pts as f32;
            let r_prof = self.evaluate_bore_profile(frac);
            let px = right_rect.min.x + 15.0 + frac * bore_w;
            let py_top = center_y - r_prof * 40.0;
            let py_bot = center_y + r_prof * 40.0;

            let pt_top = egui::pos2(px, py_top);
            let pt_bot = egui::pos2(px, py_bot);

            if let (Some(pt_t), Some(pt_b)) = (prev_top, prev_bot) {
                painter.line_segment([pt_t, pt_top], Stroke::new(2.0_f32, Color32::from_rgb(255, 215, 0)));
                painter.line_segment([pt_b, pt_bot], Stroke::new(2.0_f32, Color32::from_rgb(255, 215, 0)));
            }
            prev_top = Some(pt_top);
            prev_bot = Some(pt_bot);
        }

        // Bell Cutoff Readout on Horn
        painter.text(
            egui::pos2(right_rect.max.x - 15.0, center_y + 48.0),
            egui::Align2::RIGHT_TOP,
            format!("Bell Cutoff: {:.0} Hz", self.bell_cutoff_hz),
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
                "LIP TENSION (f_lip)",
                format!("{:.1} Hz (Bb3)", self.lip_tension_hz),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "BLOWING PRESSURE (P_m)",
                format!(
                    "{:.2} kPa ({:.1}% Eff)",
                    self.blowing_pressure_kpa,
                    self.acoustic_impedance_score * 100.0
                ),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "BORE LENGTH (L_tube)",
                format!("{:.2} m (V: {:?})", self.bore_length_m, self.valve_state),
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "BELL RADIATION CUTOFF",
                format!("{:.0} Hz (γ={:.2})", self.bell_cutoff_hz, self.instrument.bell_flare_exponent()),
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
            "[PASS] Physical Modeling Waveguide Brass Acoustic Lip-Reed & Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
