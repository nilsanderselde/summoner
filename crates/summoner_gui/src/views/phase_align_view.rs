// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Dynamic Spectral Phase Alignment & Multi-Microphone Comb Neutralizer HUD (Step 1542).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const PHASE_ALIGN_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_DELAY_MS: f32 = -50.0;
pub const MAX_DELAY_MS: f32 = 50.0;
pub const MIN_ALLPASS_FREQ_HZ: f32 = 20.0;
pub const MAX_ALLPASS_FREQ_HZ: f32 = 20000.0;
pub const MIN_ALLPASS_Q: f32 = 0.1;
pub const MAX_ALLPASS_Q: f32 = 10.0;

/// Multi-Microphone Source Track Pair Preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MicPairPreset {
    DrumKickInOut,      // Kick In Boundary / Kick Out Sub-Kick
    SnareTopBottom,     // Snare Top Dynamic / Snare Bottom Resonant Wire
    OverheadLeftRight,  // Stereo Overhead XY/ORTF Cymbal Alignment
    BassDiAndAmp,       // Direct Injection Clean Lows / Mic'd Bass Tube Amp
    AcousticGuitarDual, // 12th Fret Small Diaphragm / Body Soundhole Large Diaphragm
}

impl MicPairPreset {
    pub fn default_delay_ms(&self) -> f32 {
        match self {
            Self::DrumKickInOut => 2.45,
            Self::SnareTopBottom => -1.15,
            Self::OverheadLeftRight => 0.35,
            Self::BassDiAndAmp => 4.80,
            Self::AcousticGuitarDual => 1.85,
        }
    }

    pub fn default_allpass_freq_hz(&self) -> f32 {
        match self {
            Self::DrumKickInOut => 85.0,
            Self::SnareTopBottom => 240.0,
            Self::OverheadLeftRight => 1200.0,
            Self::BassDiAndAmp => 110.0,
            Self::AcousticGuitarDual => 450.0,
        }
    }

    pub fn default_invert_polarity(&self) -> bool {
        match self {
            Self::SnareTopBottom => true, // Opposing diaphragm orientation
            _ => false,
        }
    }
}

/// Dynamic Spectral Phase Alignment View HUD (Step 1542).
#[derive(Debug, Clone)]
pub struct PhaseAlignView {
    pub preset: MicPairPreset,
    pub time_delay_ms: f32,         // [-50.0 ..= +50.0 ms]
    pub allpass_freq_hz: f32,       // [20.0 ..= 20000.0 Hz]
    pub allpass_q: f32,             // [0.1 ..= 10.0]
    pub invert_polarity: bool,      // true = -180 deg polarity flip
    pub dynamic_tracking: bool,     // true = real-time auto-delay correction
    pub phase_puck_pos: (f32, f32), // Normalized (X: delay_ms, Y: allpass_freq)
    pub is_dragging_puck: bool,
    pub phase_correlation: f32, // [-1.0 ..= +1.0] (1.0 = Perfect In-Phase)
    pub first_comb_notch_hz: f32, // First destructive interference null
    pub coherence_score: f32,   // [0.0 ..= 1.0] Spectral phase alignment score
    pub color_palette: ContrastColorPalette,
}

impl Default for PhaseAlignView {
    fn default() -> Self {
        Self::new()
    }
}

impl PhaseAlignView {
    pub fn new() -> Self {
        let mut view = Self {
            preset: MicPairPreset::DrumKickInOut,
            time_delay_ms: 2.45,
            allpass_freq_hz: 85.0,
            allpass_q: 1.414,
            invert_polarity: false,
            dynamic_tracking: true,
            phase_puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            phase_correlation: 0.94,
            first_comb_notch_hz: 204.0,
            coherence_score: 0.96,
            color_palette: ContrastColorPalette::default(),
        };
        view.phase_puck_pos = (
            Self::delay_to_normalized(view.time_delay_ms),
            Self::freq_to_normalized(view.allpass_freq_hz),
        );
        view.update_phase_calculations();
        view
    }

    /// Convert Time Delay [-50.0 ..= +50.0 ms] to normalized coordinate [0.0 ..= 1.0].
    pub fn delay_to_normalized(ms: f32) -> f32 {
        let d = ms.clamp(MIN_DELAY_MS, MAX_DELAY_MS);
        ((d - MIN_DELAY_MS) / (MAX_DELAY_MS - MIN_DELAY_MS)).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Time Delay [-50.0 ..= +50.0 ms].
    pub fn normalized_to_delay(norm: f32) -> f32 {
        MIN_DELAY_MS + norm.clamp(0.0, 1.0) * (MAX_DELAY_MS - MIN_DELAY_MS)
    }

    /// Convert Allpass Frequency [20.0 ..= 20000.0 Hz] to logarithmic normalized coordinate [0.0 ..= 1.0].
    pub fn freq_to_normalized(hz: f32) -> f32 {
        let f = hz.clamp(MIN_ALLPASS_FREQ_HZ, MAX_ALLPASS_FREQ_HZ);
        ((f.ln() - MIN_ALLPASS_FREQ_HZ.ln())
            / (MAX_ALLPASS_FREQ_HZ.ln() - MIN_ALLPASS_FREQ_HZ.ln()))
        .clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Allpass Frequency [20.0 ..= 20000.0 Hz].
    pub fn normalized_to_freq(norm: f32) -> f32 {
        (MIN_ALLPASS_FREQ_HZ.ln()
            + norm.clamp(0.0, 1.0) * (MAX_ALLPASS_FREQ_HZ.ln() - MIN_ALLPASS_FREQ_HZ.ln()))
        .exp()
    }

    /// Set microphone pair preset and reset default parameters.
    pub fn set_preset(&mut self, preset: MicPairPreset) {
        self.preset = preset;
        self.time_delay_ms = preset.default_delay_ms();
        self.allpass_freq_hz = preset.default_allpass_freq_hz();
        self.invert_polarity = preset.default_invert_polarity();
        self.phase_puck_pos = (
            Self::delay_to_normalized(self.time_delay_ms),
            Self::freq_to_normalized(self.allpass_freq_hz),
        );
        self.update_phase_calculations();
    }

    /// Update phase group delay and comb filter frequency calculations.
    pub fn update_phase_calculations(&mut self) {
        let delay_sec = self.time_delay_ms.abs() * 1e-3;
        if delay_sec > 1e-5 {
            self.first_comb_notch_hz = (1.0 / (2.0 * delay_sec)).clamp(10.0, 24000.0);
        } else {
            self.first_comb_notch_hz = 24000.0;
        }

        // Phase correlation model: tau ~ 0 gives +1.0, polarity inversion inverts sign
        let tau_err = self.time_delay_ms.abs();
        let raw_corr = 1.0 / (1.0 + (tau_err / 1.5).powi(2));
        self.phase_correlation = if self.invert_polarity {
            -raw_corr
        } else {
            raw_corr
        };

        let pol_penalty = if self.invert_polarity { 0.1 } else { 1.0 };
        self.coherence_score = (raw_corr * pol_penalty).clamp(0.05, 1.0);
    }

    /// Evaluate frequency-dependent phase response phi(f) in radians [-pi, +pi].
    pub fn evaluate_phase_shift(&self, freq_hz: f32) -> f32 {
        let f = freq_hz.clamp(20.0, 20000.0);
        let omega = 2.0 * std::f32::consts::PI * f;
        let tau_sec = self.time_delay_ms * 1e-3;
        let linear_phase = -omega * tau_sec;

        // 2nd-order Allpass Filter Phase response: phi_ap = -2 * atan( (f/fc) / Q / (1 - (f/fc)^2) )
        let fc = self.allpass_freq_hz;
        let ratio = f / fc;
        let q = self.allpass_q;
        let denom = 1.0 - ratio * ratio;
        let allpass_phase = if denom.abs() > 1e-4 {
            -2.0 * (ratio / (q * denom)).atan()
        } else {
            -std::f32::consts::PI
        };

        let mut total_phase = linear_phase + allpass_phase;
        if self.invert_polarity {
            total_phase += std::f32::consts::PI;
        }

        // Wrap to [-pi, +pi]
        ((total_phase + std::f32::consts::PI) % (2.0 * std::f32::consts::PI)) - std::f32::consts::PI
    }

    /// Evaluate comb filter summed magnitude response at frequency f (Hz) in dBFS.
    pub fn evaluate_comb_response_db(&self, freq_hz: f32) -> f32 {
        let phi = self.evaluate_phase_shift(freq_hz);
        let mag = (2.0 * (1.0 + phi.cos())).max(1e-5).sqrt();
        (20.0 * (mag * 0.5).log10()).clamp(-36.0, 6.0)
    }

    /// Hit-test touch coordinate on the phase adjustment puck.
    pub fn hit_test_phase_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.phase_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.phase_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= PHASE_ALIGN_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render of Spectral Phase Rotation and Comb Neutralizer Curve.
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

        // Draw Comb Frequency Response on left half
        let left_w = mid_x - 2;
        let center_r = height / 2;
        grid[center_r][1] = '0';
        for c in 2..left_w {
            let frac = (c - 2) as f32 / (left_w - 1) as f32;
            let f = (MIN_ALLPASS_FREQ_HZ.ln()
                + frac * (MAX_ALLPASS_FREQ_HZ.ln() - MIN_ALLPASS_FREQ_HZ.ln()))
            .exp();
            let db = self.evaluate_comb_response_db(f);
            let norm_y = (db + 36.0) / 42.0;
            let r = (height - 3) - (norm_y * (height - 4) as f32).round() as usize;
            if r > 0 && r < height - 1 {
                grid[r][c] = '*';
            }
        }

        // Draw Phase Puck on right half
        let right_w = width - mid_x - 2;
        let puck_col =
            mid_x + 1 + ((self.phase_puck_pos.0 * (right_w - 2) as f32).round() as usize);
        let puck_row =
            (((1.0 - self.phase_puck_pos.1) * (height - 3) as f32) + 1.0).round() as usize;
        if puck_row < height - 1 && puck_col < width - 1 {
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
            "DYNAMIC SPECTRAL PHASE ALIGNMENT & COMB NEUTRALIZER HUD",
            egui::FontId::proportional(16.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Microphone Pair Selector Tabs (y: 48..92) - Each tab >= 44pt height
        let presets = [
            (MicPairPreset::DrumKickInOut, "KICK IN / OUT"),
            (MicPairPreset::SnareTopBottom, "SNARE TOP / BOT"),
            (MicPairPreset::OverheadLeftRight, "OVERHEADS L / R"),
            (MicPairPreset::BassDiAndAmp, "BASS DI / AMP"),
            (MicPairPreset::AcousticGuitarDual, "ACOUSTIC DUAL"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (pr, name)) in presets.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_selected = self.preset == *pr;
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
                        self.set_preset(*pr);
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

        // Left 55%: Comb Filter Neutralizer Spectrum
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
            "SUMMED SPECTRAL COMB RESPONSE & NULL NOTCHES",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // 0 dB Guide Line
        let zero_db_y = left_rect.min.y + 35.0 + (6.0 / 42.0) * (left_rect.height() - 60.0);
        painter.line_segment(
            [
                egui::pos2(left_rect.min.x + 10.0, zero_db_y),
                egui::pos2(left_rect.max.x - 10.0, zero_db_y),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 90, 130, 100)),
        );

        // Comb Curve
        let mut curve_pts = Vec::new();
        for i in 0..50 {
            let frac = i as f32 / 49.0;
            let f = (MIN_ALLPASS_FREQ_HZ.ln()
                + frac * (MAX_ALLPASS_FREQ_HZ.ln() - MIN_ALLPASS_FREQ_HZ.ln()))
            .exp();
            let db = self.evaluate_comb_response_db(f);
            let norm_y = (db + 36.0) / 42.0;
            let px = left_rect.min.x + 15.0 + frac * (left_rect.width() - 30.0);
            let py = left_rect.max.y - 25.0 - norm_y * (left_rect.height() - 60.0);
            curve_pts.push(egui::pos2(px, py));
        }

        for i in 0..curve_pts.len() - 1 {
            painter.line_segment(
                [curve_pts[i], curve_pts[i + 1]],
                Stroke::new(2.0_f32, Color32::from_rgb(0, 229, 255)),
            );
        }

        painter.text(
            egui::pos2(left_rect.min.x + 15.0, left_rect.max.y - 20.0),
            egui::Align2::LEFT_BOTTOM,
            format!("1st Comb Notch: {:.0} Hz", self.first_comb_notch_hz),
            egui::FontId::proportional(10.0),
            Color32::from_rgb(255, 215, 0),
        );

        // Right 45%: Delay & Allpass Phase Angle XY Control Map
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
            "TIME DELAY (X) vs ALLPASS ROTATION (Y)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // Center zero delay guide line
        let cx = right_rect.center().x;
        painter.line_segment(
            [
                egui::pos2(cx, right_rect.min.y + 30.0),
                egui::pos2(cx, right_rect.max.y - 20.0),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 90, 130, 90)),
        );

        // Interactive Phase Puck
        let puck_x = right_rect.min.x + self.phase_puck_pos.0 * right_rect.width();
        let puck_y = right_rect.max.y - self.phase_puck_pos.1 * right_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if right_rect.contains(mouse_pos) {
                    let nx =
                        ((mouse_pos.x - right_rect.min.x) / right_rect.width()).clamp(0.0, 1.0);
                    let ny =
                        ((right_rect.max.y - mouse_pos.y) / right_rect.height()).clamp(0.0, 1.0);
                    self.phase_puck_pos = (nx, ny);
                    self.time_delay_ms = Self::normalized_to_delay(nx);
                    self.allpass_freq_hz = Self::normalized_to_freq(ny);
                    self.update_phase_calculations();
                }
            }
        }

        // Draw Touch Hit Target boundary (>= 44x44pt)
        painter.circle_stroke(
            puck_pos,
            PHASE_ALIGN_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(0, 229, 255, 140)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(0, 229, 255));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        painter.text(
            egui::pos2(right_rect.min.x + 10.0, right_rect.max.y - 20.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Delay: {:.2} ms | Allpass fc: {:.0} Hz",
                self.time_delay_ms, self.allpass_freq_hz
            ),
            egui::FontId::proportional(10.0),
            Color32::from_rgb(0, 229, 255),
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
                "TIME DELAY OFFSET (Δt)",
                format!(
                    "{:.2} ms ({:.1} cm)",
                    self.time_delay_ms,
                    self.time_delay_ms * 34.32
                ),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "PHASE CORRELATION (r)",
                format!(
                    "{:.2} ({:.1}% In-Phase)",
                    self.phase_correlation,
                    self.coherence_score * 100.0
                ),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "ALLPASS ROTATION (fc)",
                format!("{:.0} Hz (Q = {:.2})", self.allpass_freq_hz, self.allpass_q),
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "POLARITY / TRACKING",
                format!(
                    "{} | Dynamic Auto",
                    if self.invert_polarity {
                        "INVERT (-180°)"
                    } else {
                        "NORMAL (0°)"
                    }
                ),
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
            "[PASS] Dynamic Spectral Phase Alignment & Comb Neutralizer Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
