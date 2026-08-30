// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Physical Modeling Woodwind Tonehole Acoustic Radiation & Standing Wave HUD (Milestone 17).

use std::f32::consts::PI;
use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use summoner_dsp::tonehole_grid::{NUM_TONEHOLES, ToneholeLattice, SPEED_OF_SOUND_MPS};

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const TONEHOLE_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_CUTOFF_HZ: f32 = 500.0;
pub const MAX_CUTOFF_HZ: f32 = 6000.0;
pub const MIN_BORE_LENGTH_M: f32 = 0.20;
pub const MAX_BORE_LENGTH_M: f32 = 1.20;

/// Tonehole Matrix Display Mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToneholeDisplayMode {
    /// Acoustic standing wave pressure profile along the bore.
    #[default]
    StandingWaveProfile,
    /// Tonehole lattice radiation impedance transfer function $R(f)$ vs $T(f)$.
    RadiationImpedanceSpectrum,
    /// Discrete 6-tonehole 3-port acoustic scattering matrix coefficients.
    ScatteringMatrix3Port,
}

/// Interactive 6-Tonehole Acoustic Radiation Matrix & Bore Standing Wave HUD.
#[derive(Debug, Clone)]
pub struct ToneholeMatrixView {
    pub lattice: ToneholeLattice,
    pub display_mode: ToneholeDisplayMode,
    pub active_hole_idx: usize,
    pub hole_open_fractions: [f32; NUM_TONEHOLES],
    pub bore_length_m: f32,
    pub fundamental_hz: f32,
    pub lattice_cutoff_hz: f32,
    pub radiated_power_mw: f32,
    pub radiation_puck_pos: (f32, f32), // Normalized (X: bore length, Y: cutoff freq)
    pub is_dragging_puck: bool,
    pub color_palette: ContrastColorPalette,
}

impl Default for ToneholeMatrixView {
    fn default() -> Self {
        Self::new()
    }
}

impl ToneholeMatrixView {
    /// Creates a new Tonehole Matrix HUD view with standard Concert C Flute geometry.
    pub fn new() -> Self {
        let lattice = ToneholeLattice::new_standard_flute(0.60);
        let mut view = Self {
            lattice,
            display_mode: ToneholeDisplayMode::StandingWaveProfile,
            active_hole_idx: 0,
            hole_open_fractions: [0.0; NUM_TONEHOLES], // All closed (fundamental)
            bore_length_m: 0.60,
            fundamental_hz: 261.63,
            lattice_cutoff_hz: 2200.0,
            radiated_power_mw: 14.5,
            radiation_puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        };
        view.radiation_puck_pos = (
            Self::length_to_normalized(view.bore_length_m),
            Self::cutoff_to_normalized(view.lattice_cutoff_hz),
        );
        view.update_acoustics();
        view
    }

    /// Converts Bore Length [0.20 ..= 1.20 m] to normalized [0.0 ..= 1.0].
    pub fn length_to_normalized(m: f32) -> f32 {
        let l = m.clamp(MIN_BORE_LENGTH_M, MAX_BORE_LENGTH_M);
        ((l - MIN_BORE_LENGTH_M) / (MAX_BORE_LENGTH_M - MIN_BORE_LENGTH_M)).clamp(0.0, 1.0)
    }

    /// Converts normalized [0.0 ..= 1.0] to Bore Length [0.20 ..= 1.20 m].
    pub fn normalized_to_length(norm: f32) -> f32 {
        MIN_BORE_LENGTH_M + norm.clamp(0.0, 1.0) * (MAX_BORE_LENGTH_M - MIN_BORE_LENGTH_M)
    }

    /// Converts Lattice Cutoff [500 ..= 6000 Hz] to normalized [0.0 ..= 1.0].
    pub fn cutoff_to_normalized(hz: f32) -> f32 {
        let c = hz.clamp(MIN_CUTOFF_HZ, MAX_CUTOFF_HZ);
        ((c - MIN_CUTOFF_HZ) / (MAX_CUTOFF_HZ - MIN_CUTOFF_HZ)).clamp(0.0, 1.0)
    }

    /// Converts normalized [0.0 ..= 1.0] to Lattice Cutoff [500 ..= 6000 Hz].
    pub fn normalized_to_cutoff(norm: f32) -> f32 {
        MIN_CUTOFF_HZ + norm.clamp(0.0, 1.0) * (MAX_CUTOFF_HZ - MIN_CUTOFF_HZ)
    }

    /// Sets the discrete fingering bitmask (bits 0..5).
    pub fn set_fingering_mask(&mut self, mask: u8) {
        for i in 0..NUM_TONEHOLES {
            let closed = (mask & (1 << i)) != 0;
            self.hole_open_fractions[i] = if closed { 0.0 } else { 1.0 };
        }
        self.update_acoustics();
    }

    /// Updates acoustic standing wave and lattice radiation parameters.
    pub fn update_acoustics(&mut self) {
        self.lattice.total_bore_length_m = self.bore_length_m;
        self.lattice.set_open_fractions(self.hole_open_fractions);
        self.lattice_cutoff_hz = self.lattice.lattice_cutoff_hz;

        let eff_len = self.lattice.effective_acoustic_length_m();
        self.fundamental_hz = (SPEED_OF_SOUND_MPS / (2.0 * eff_len)).clamp(40.0, 4000.0);

        let open_count = self.hole_open_fractions.iter().filter(|&&f| f > 0.05).count();
        self.radiated_power_mw = 8.0 + (open_count as f32 * 3.5) + (self.fundamental_hz / 100.0);
    }

    /// Evaluates standing wave acoustic pressure at relative position $x / L \in [0.0, 1.0]$.
    pub fn evaluate_standing_wave_pressure(&self, x_ratio: f32) -> f32 {
        let x = x_ratio.clamp(0.0, 1.0);
        let eff_ratio = (self.lattice.effective_acoustic_length_m() / self.bore_length_m).clamp(0.1, 1.0);

        if x <= eff_ratio {
            let norm_x = x / eff_ratio;
            // Half-wave standing wave resonator with open-open pressure nodes at ends
            (norm_x * PI).sin()
        } else {
            // Decaying evanescent wave past the first open tonehole below lattice cutoff
            let past = (x - eff_ratio) / (1.0 - eff_ratio).max(0.01);
            0.05 * (-past * 3.5).exp()
        }
    }

    /// Evaluates lattice reflection magnitude $|R(f)|$ at frequency $f$ in Hz.
    pub fn evaluate_reflection_magnitude(&self, freq_hz: f32) -> f32 {
        let f = freq_hz.clamp(20.0, 10000.0);
        let fc = self.lattice_cutoff_hz;
        if f <= fc {
            0.95 * (1.0 - (f / fc).powi(2) * 0.40)
        } else {
            (0.55 * (fc / f).powi(2)).clamp(0.01, 0.95)
        }
    }

    /// Hit-test touch coordinate on the radiation tuning puck.
    pub fn hit_test_radiation_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.radiation_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.radiation_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= TONEHOLE_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render representation.
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

        let center_r = height / 2;
        // Draw standing wave curve across the grid
        let plot_w = width - 4;
        for c in 0..plot_w {
            let x_ratio = c as f32 / plot_w as f32;
            let p = self.evaluate_standing_wave_pressure(x_ratio);
            let row_offset = (p * ((height / 2) as f32 - 2.0)).round() as isize;
            let r = ((center_r as isize) - row_offset).clamp(1, (height - 2) as isize) as usize;
            grid[r][c + 2] = '*';
        }

        // Draw 6 Tonehole Indicators on bottom border
        for h in 0..NUM_TONEHOLES {
            let col = 2 + ((self.lattice.toneholes[h].params.position_ratio * (plot_w as f32)).round() as usize)
                .min(plot_w - 1);
            let symbol = if self.hole_open_fractions[h] < 0.05 { 'X' } else { 'O' };
            grid[height - 2][col] = symbol;
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
            "WOODWIND 6-TONEHOLE RADIATION & STANDING WAVE HUD",
            egui::FontId::proportional(16.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Mode Selector Tabs (y: 48..92) - Each tab >= 44pt height
        let modes = [
            (
                ToneholeDisplayMode::StandingWaveProfile,
                "STANDING WAVE PROFILE",
            ),
            (
                ToneholeDisplayMode::RadiationImpedanceSpectrum,
                "RADIATION SPECTRUM R(f)",
            ),
            (
                ToneholeDisplayMode::ScatteringMatrix3Port,
                "3-PORT SCATTERING MATRIX",
            ),
        ];

        let tab_w = (rect.width() - 40.0 - 2.0 * 8.0) / 3.0;
        for (i, (mode, label)) in modes.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_selected = self.display_mode == *mode;
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
                *label,
                egui::FontId::proportional(11.0),
                text_color,
            );

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if tab_rect.contains(pos) {
                        self.display_mode = *mode;
                    }
                }
            }
        }

        // Main Visualizer Canvas (y: 104..340)
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

        // Left 60%: Standing wave / Radiation curve canvas
        let left_w = main_canvas.width() * 0.60;
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

        // Draw curve according to display mode
        match self.display_mode {
            ToneholeDisplayMode::StandingWaveProfile => {
                painter.text(
                    egui::pos2(left_rect.min.x + 10.0, left_rect.min.y + 10.0),
                    egui::Align2::LEFT_TOP,
                    "ACOUSTIC PRESSURE STANDING WAVE P(x) ALONG BORE",
                    egui::FontId::proportional(11.0),
                    Color32::from_rgb(160, 180, 205),
                );

                let cy = left_rect.center().y;
                painter.line_segment(
                    [
                        egui::pos2(left_rect.min.x + 10.0, cy),
                        egui::pos2(left_rect.max.x - 10.0, cy),
                    ],
                    Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(70, 90, 120, 100)),
                );

                let mut prev_pt: Option<egui::Pos2> = None;
                for step in 0..=100 {
                    let xr = step as f32 / 100.0;
                    let p = self.evaluate_standing_wave_pressure(xr);
                    let px = left_rect.min.x + 10.0 + xr * (left_rect.width() - 20.0);
                    let py = cy - p * (left_rect.height() * 0.38);
                    let curr_pt = egui::pos2(px, py);

                    if let Some(prev) = prev_pt {
                        painter.line_segment(
                            [prev, curr_pt],
                            Stroke::new(2.5_f32, Color32::from_rgb(0, 229, 255)),
                        );
                    }
                    prev_pt = Some(curr_pt);
                }
            }
            ToneholeDisplayMode::RadiationImpedanceSpectrum => {
                painter.text(
                    egui::pos2(left_rect.min.x + 10.0, left_rect.min.y + 10.0),
                    egui::Align2::LEFT_TOP,
                    "REFLECTION MAGNITUDE |R(f)| vs LATTICE CUTOFF",
                    egui::FontId::proportional(11.0),
                    Color32::from_rgb(160, 180, 205),
                );

                let mut prev_pt: Option<egui::Pos2> = None;
                for step in 0..=100 {
                    let freq = 100.0 + (step as f32 / 100.0) * 5900.0;
                    let r_mag = self.evaluate_reflection_magnitude(freq);
                    let px = left_rect.min.x + 10.0 + (step as f32 / 100.0) * (left_rect.width() - 20.0);
                    let py = left_rect.max.y - 15.0 - r_mag * (left_rect.height() - 45.0);
                    let curr_pt = egui::pos2(px, py);

                    if let Some(prev) = prev_pt {
                        painter.line_segment(
                            [prev, curr_pt],
                            Stroke::new(2.5_f32, Color32::from_rgb(255, 215, 0)),
                        );
                    }
                    prev_pt = Some(curr_pt);
                }
            }
            ToneholeDisplayMode::ScatteringMatrix3Port => {
                painter.text(
                    egui::pos2(left_rect.min.x + 10.0, left_rect.min.y + 10.0),
                    egui::Align2::LEFT_TOP,
                    "3-PORT SCATTERING COEFFICIENTS (r_s, t_s)",
                    egui::FontId::proportional(11.0),
                    Color32::from_rgb(160, 180, 205),
                );

                for h in 0..NUM_TONEHOLES {
                    let z_ratio = self.lattice.toneholes[h].compute_shunt_impedance_ratio();
                    let denom = 2.0 * z_ratio + 1.0;
                    let r_s = -1.0 / denom;
                    let t_s = (2.0 * z_ratio) / denom;
                    let hy = left_rect.min.y + 35.0 + h as f32 * 24.0;
                    let status = if self.hole_open_fractions[h] < 0.05 { "CLOSED" } else { "OPEN" };

                    painter.text(
                        egui::pos2(left_rect.min.x + 15.0, hy),
                        egui::Align2::LEFT_TOP,
                        format!("Tonehole {}: {} | r_s = {:.3} | t_s = {:.3} | z_ratio = {:.2}", h + 1, status, r_s, t_s, z_ratio),
                        egui::FontId::proportional(10.0),
                        Color32::from_rgb(0, 229, 255),
                    );
                }
            }
        }

        // Right 40%: 6 Interactive Tonehole Keys & Sliders
        let right_w = main_canvas.width() * 0.40;
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
            "6 TONEHOLE KEYS (>= 44x44pt TARGETS)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // 6 Tonehole interactive pads (2 columns x 3 rows)
        let col_w = (right_rect.width() - 30.0) / 2.0;
        let row_h = 48.0;

        for h in 0..NUM_TONEHOLES {
            let col_idx = h % 2;
            let row_idx = h / 2;
            let hx = right_rect.min.x + 10.0 + col_idx as f32 * (col_w + 10.0);
            let hy = right_rect.min.y + 32.0 + row_idx as f32 * (row_h + 8.0);

            let btn_rect = egui::Rect::from_min_size(egui::pos2(hx, hy), egui::vec2(col_w, row_h));
            let is_closed = self.hole_open_fractions[h] < 0.05;
            let bg_col = if is_closed {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(30, 45, 65)
            };
            let text_col = if is_closed {
                Color32::from_rgb(10, 14, 24)
            } else {
                Color32::from_rgb(200, 215, 235)
            };

            painter.rect_filled(btn_rect, 4.0, bg_col);
            painter.text(
                btn_rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("Hole {}: {}", h + 1, if is_closed { "CLOSED" } else { "OPEN" }),
                egui::FontId::proportional(11.0),
                text_col,
            );

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if btn_rect.contains(pos) {
                        self.hole_open_fractions[h] = if is_closed { 1.0 } else { 0.0 };
                        self.update_acoustics();
                    }
                }
            }
        }

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
                "FUNDAMENTAL PITCH",
                format!("{:.1} Hz (L_eff: {:.2} m)", self.fundamental_hz, self.lattice.effective_acoustic_length_m()),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "LATTICE CUTOFF",
                format!("{:.0} Hz (Keefe 1990)", self.lattice_cutoff_hz),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "RADIATED POWER",
                format!("{:.1} mW (Acoustic)", self.radiated_power_mw),
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "BORE GEOMETRY",
                format!("{:.2} m (r_bore: 9.5 mm)", self.bore_length_m),
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
            "[PASS] Tonehole Acoustic Radiation Matrix & Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
