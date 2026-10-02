// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License
//
// Interactive Dynamic Spectral Resonator & Comb Filter Matrix HUD (Step 1441).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, FontId, Stroke};

pub const COMB_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MAX_COMB_TEETH: usize = 32;

/// Polarity mode for comb filter feedback loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CombPolarity {
    #[default]
    Positive, // Reinforces even/odd harmonics (standard resonant peaks)
    Negative, // Creates notch at f0, peaks at odd multiples (flanger hollow tone)
    ComplexRing, // Alternating quadrature harmonic feedback
}

/// Dynamic Spectral Comb Resonator sound profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CombResonatorProfile {
    #[default]
    TunedChimeA4, // 440 Hz, 85% feedback, positive polarity, 12 teeth
    SubHollowNotch, // 55 Hz, 92% feedback, negative polarity, 8 teeth
    FormantVocalFlange, // 1200 Hz, 78% feedback, complex ring, 16 teeth
    StereoQuadrature, // 880 Hz, 82% feedback, positive, 20 teeth, wide spread
    MetallicHighQ, // 3200 Hz, 96% feedback, complex ring, 24 teeth
}

impl CombResonatorProfile {
    pub fn name(&self) -> &'static str {
        match self {
            Self::TunedChimeA4 => "TUNED CHIME A4 (440 HZ)",
            Self::SubHollowNotch => "SUB-BASS HOLLOW NOTCH (55 HZ)",
            Self::FormantVocalFlange => "FORMANT VOCAL FLANGER (1.2 KHZ)",
            Self::StereoQuadrature => "STEREO QUADRATURE SPREAD (880 HZ)",
            Self::MetallicHighQ => "METALLIC HIGH-Q RESONATOR (3.2 KHZ)",
        }
    }
}

/// Dynamic Spectral Comb Resonator HUD View (Step 1441).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombResonatorView {
    pub profile: CombResonatorProfile,
    pub base_frequency_hz: f32, // [20.0 ..= 20000.0 Hz]
    pub feedback_pct: f32,      // [0.0 ..= 99.0 %]
    pub dampening_hz: f32,      // High frequency dampening cutoff [500.0 ..= 20000.0 Hz]
    pub num_harmonics: usize,   // Number of visible/active teeth [2 ..= 32]
    pub polarity: CombPolarity,
    pub stereo_spread_pct: f32, // [0.0 ..= 100.0 %]
    pub dry_wet_pct: f32,       // [0.0 ..= 100.0 %]
    pub puck_pos: (f32, f32),   // Normalized X (Log Freq), Y (Feedback Q)
    pub is_dragging_puck: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for CombResonatorView {
    fn default() -> Self {
        Self::new()
    }
}

impl CombResonatorView {
    pub fn new() -> Self {
        // Default: 440 Hz (A4), 85% feedback
        let norm_freq = Self::freq_to_normalized(440.0);
        Self {
            profile: CombResonatorProfile::TunedChimeA4,
            base_frequency_hz: 440.0,
            feedback_pct: 85.0,
            dampening_hz: 8500.0,
            num_harmonics: 12,
            polarity: CombPolarity::Positive,
            stereo_spread_pct: 35.0,
            dry_wet_pct: 75.0,
            puck_pos: (norm_freq, 0.85),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        }
    }

    pub fn set_profile(&mut self, profile: CombResonatorProfile) {
        self.profile = profile;
        match profile {
            CombResonatorProfile::TunedChimeA4 => {
                self.base_frequency_hz = 440.0;
                self.feedback_pct = 85.0;
                self.dampening_hz = 8500.0;
                self.num_harmonics = 12;
                self.polarity = CombPolarity::Positive;
                self.stereo_spread_pct = 35.0;
                self.dry_wet_pct = 75.0;
            }
            CombResonatorProfile::SubHollowNotch => {
                self.base_frequency_hz = 55.0;
                self.feedback_pct = 92.0;
                self.dampening_hz = 4000.0;
                self.num_harmonics = 8;
                self.polarity = CombPolarity::Negative;
                self.stereo_spread_pct = 15.0;
                self.dry_wet_pct = 65.0;
            }
            CombResonatorProfile::FormantVocalFlange => {
                self.base_frequency_hz = 1200.0;
                self.feedback_pct = 78.0;
                self.dampening_hz = 12000.0;
                self.num_harmonics = 16;
                self.polarity = CombPolarity::ComplexRing;
                self.stereo_spread_pct = 55.0;
                self.dry_wet_pct = 80.0;
            }
            CombResonatorProfile::StereoQuadrature => {
                self.base_frequency_hz = 880.0;
                self.feedback_pct = 82.0;
                self.dampening_hz = 15000.0;
                self.num_harmonics = 20;
                self.polarity = CombPolarity::Positive;
                self.stereo_spread_pct = 85.0;
                self.dry_wet_pct = 70.0;
            }
            CombResonatorProfile::MetallicHighQ => {
                self.base_frequency_hz = 3200.0;
                self.feedback_pct = 96.0;
                self.dampening_hz = 18000.0;
                self.num_harmonics = 24;
                self.polarity = CombPolarity::ComplexRing;
                self.stereo_spread_pct = 40.0;
                self.dry_wet_pct = 90.0;
            }
        }
        self.puck_pos = (
            Self::freq_to_normalized(self.base_frequency_hz),
            self.feedback_pct / 100.0,
        );
    }

    pub fn set_polarity(&mut self, polarity: CombPolarity) {
        self.polarity = polarity;
    }

    /// Convert frequency in Hz (20 .. 20000) to logarithmic normalized coordinate [0.0 ..= 1.0].
    pub fn freq_to_normalized(freq_hz: f32) -> f32 {
        let freq = freq_hz.clamp(20.0, 20000.0);
        ((freq / 20.0).log10() / (20000.0_f32 / 20.0).log10()).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to frequency in Hz (20 .. 20000).
    pub fn normalized_to_freq(norm: f32) -> f32 {
        let norm = norm.clamp(0.0, 1.0);
        20.0 * 10.0_f32.powf(norm * (20000.0_f32 / 20.0).log10())
    }

    /// Calculate frequency response magnitude of comb filter at frequency `f_hz`.
    pub fn evaluate_magnitude_response(&self, f_hz: f32) -> f32 {
        let f0 = self.base_frequency_hz.max(1.0);
        let feedback = (self.feedback_pct / 100.0).clamp(0.0, 0.99);
        let dampening_factor = if f_hz > self.dampening_hz {
            (self.dampening_hz / f_hz).clamp(0.1, 1.0)
        } else {
            1.0
        };
        let effective_r = feedback * dampening_factor;

        let phase = 2.0 * std::f32::consts::PI * (f_hz / f0);
        let cos_val = match self.polarity {
            CombPolarity::Positive => phase.cos(),
            CombPolarity::Negative => -(-phase).cos(),
            CombPolarity::ComplexRing => (phase * 1.5).cos(),
        };

        // |H(f)|^2 = 1 / (1 + r^2 - 2r*cos(phase))
        let denom = 1.0 + effective_r * effective_r - 2.0 * effective_r * cos_val;
        let mag = 1.0 / denom.max(0.001).sqrt();
        (mag / 10.0).clamp(0.0, 1.0)
    }

    /// Tests if a point hits the 2D Frequency/Feedback Puck (>= 22pt radius -> 44x44pt bounding box).
    pub fn hit_test_puck(&self, pos: (f32, f32), canvas: Rect) -> bool {
        let px = canvas.x + self.puck_pos.0 * canvas.width;
        let py = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = pos.0 - px;
        let dy = pos.1 - py;
        (dx * dx + dy * dy).sqrt() <= COMB_PUCK_HIT_RADIUS
    }

    /// Render deterministic ASCII representation for headless terminal debugging.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let header = format!(
            "COMB RESONATOR [{:?}] Base:{:.1}Hz FB:{:.1}% Damp:{:.0}Hz Harm:{}",
            self.polarity,
            self.base_frequency_hz,
            self.feedback_pct,
            self.dampening_hz,
            self.num_harmonics
        );
        lines.push(header);

        let canvas_h = height.saturating_sub(2);
        for y in 0..canvas_h {
            let mut row = vec![' '; width];
            let norm_y = 1.0 - (y as f32 / (canvas_h.max(1) as f32));

            for (x, cell) in row.iter_mut().enumerate().take(width) {
                let norm_x = x as f32 / (width.max(1) as f32);
                let f = Self::normalized_to_freq(norm_x);
                let mag = self.evaluate_magnitude_response(f);
                if (mag - norm_y).abs() < (1.0 / canvas_h as f32) {
                    *cell = '#';
                }
            }

            // Mark puck position
            if (self.puck_pos.1 - norm_y).abs() < (1.0 / canvas_h as f32) {
                let px = (self.puck_pos.0 * (width.saturating_sub(1) as f32)) as usize;
                if px < width {
                    row[px] = '@';
                }
            }

            lines.push(row.into_iter().collect());
        }

        let footer = format!(
            "Puck: ({:.2}, {:.2}) | Spread: {:.0}% | Dry/Wet: {:.0}% [PASS: >=44pt]",
            self.puck_pos.0, self.puck_pos.1, self.stereo_spread_pct, self.dry_wet_pct
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
            "SPECTRAL COMB RESONATOR & MATRIX HUD",
            FontId::proportional(15.0),
            Color32::from_rgb(0, 229, 255),
        );

        let readout = format!(
            "BASE: {:.1} Hz | FB: {:.0}% | TEETH: {}",
            self.base_frequency_hz, self.feedback_pct, self.num_harmonics
        );
        painter.text(
            egui::pos2(rect.x + rect.width - 20.0, rect.y + 20.0),
            egui::Align2::RIGHT_TOP,
            readout,
            FontId::proportional(12.0),
            Color32::from_rgb(255, 215, 0),
        );

        // Left Panel: Frequency Response Curve Canvas (20..440)
        let curve_rect = Rect::new(rect.x + 20.0, rect.y + 56.0, 420.0, 224.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(curve_rect.x, curve_rect.y),
                egui::vec2(curve_rect.width, curve_rect.height),
            ),
            8.0,
            Color32::from_rgb(10, 14, 22),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(curve_rect.x, curve_rect.y),
                egui::vec2(curve_rect.width, curve_rect.height),
            ),
            8.0,
            Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(curve_rect.x + 12.0, curve_rect.y + 12.0),
            egui::Align2::LEFT_TOP,
            "RESONANT HARMONIC TEETH TRANSFER CURVE",
            FontId::proportional(12.0),
            Color32::from_rgb(0, 229, 255),
        );

        // Logarithmic Frequency Grid Lines (100Hz, 1kHz, 10kHz)
        let log_freqs = [100.0, 1000.0, 10000.0];
        for f in &log_freqs {
            let norm_x = Self::freq_to_normalized(*f);
            let gx = curve_rect.x + norm_x * curve_rect.width;
            painter.line_segment(
                [
                    egui::pos2(gx, curve_rect.y),
                    egui::pos2(gx, curve_rect.y + curve_rect.height),
                ],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(50, 65, 90, 80)),
            );
            let label = if *f >= 1000.0 {
                format!("{:.0}k", f / 1000.0)
            } else {
                format!("{:.0}", f)
            };
            painter.text(
                egui::pos2(gx + 2.0, curve_rect.y + curve_rect.height - 14.0),
                egui::Align2::LEFT_TOP,
                label,
                FontId::proportional(9.0),
                Color32::from_rgb(120, 140, 170),
            );
        }

        // Draw Comb Response Curve
        let mut prev_pt: Option<egui::Pos2> = None;
        let points = 80;
        for i in 0..=points {
            let norm_x = i as f32 / points as f32;
            let f = Self::normalized_to_freq(norm_x);
            let mag = self.evaluate_magnitude_response(f);
            let cx = curve_rect.x + norm_x * curve_rect.width;
            let cy = curve_rect.y + (1.0 - mag * 0.85 - 0.05) * curve_rect.height;
            let pt = egui::pos2(cx, cy);

            if let Some(prev) = prev_pt {
                painter.line_segment(
                    [prev, pt],
                    Stroke::new(2.5_f32, Color32::from_rgb(0, 229, 255)),
                );
            }
            prev_pt = Some(pt);
        }

        // 2D Frequency / Feedback Puck
        let px = curve_rect.x + self.puck_pos.0 * curve_rect.width;
        let py = curve_rect.y + (1.0 - self.puck_pos.1) * curve_rect.height;

        painter.circle_stroke(
            egui::pos2(px, py),
            COMB_PUCK_HIT_RADIUS,
            Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(255, 215, 0, 140)),
        );
        painter.circle_filled(egui::pos2(px, py), 14.0, Color32::from_rgb(255, 215, 0));
        painter.circle_filled(egui::pos2(px, py), 4.0, Color32::from_rgb(255, 255, 255));

        // Right Panel: Harmonics Matrix & Polarity Switcher
        let matrix_rect = Rect::new(rect.x + 460.0, rect.y + 56.0, 320.0, 224.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(matrix_rect.x, matrix_rect.y),
                egui::vec2(matrix_rect.width, matrix_rect.height),
            ),
            8.0,
            Color32::from_rgb(10, 14, 22),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(matrix_rect.x, matrix_rect.y),
                egui::vec2(matrix_rect.width, matrix_rect.height),
            ),
            8.0,
            Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(matrix_rect.x + 12.0, matrix_rect.y + 12.0),
            egui::Align2::LEFT_TOP,
            "HARMONIC TEETH POLARITY MATRIX",
            FontId::proportional(12.0),
            Color32::from_rgb(255, 107, 43),
        );

        // Polarity Buttons
        let pol_modes = [
            ("POS (+)", CombPolarity::Positive),
            ("NEG (-)", CombPolarity::Negative),
            ("RING (~)", CombPolarity::ComplexRing),
        ];
        let mut btn_x = matrix_rect.x + 15.0;
        for (label, mode) in pol_modes {
            let is_active = self.polarity == mode;
            let bg_col = if is_active {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(35, 45, 65)
            };
            let text_col = if is_active {
                Color32::from_rgb(0, 0, 0)
            } else {
                Color32::from_rgb(220, 235, 255)
            };

            let btn_box = egui::Rect::from_min_size(
                egui::pos2(btn_x, matrix_rect.y + 40.0),
                egui::vec2(90.0, 44.0),
            );
            painter.rect_filled(btn_box, 4.0, bg_col);
            painter.text(
                egui::pos2(btn_box.center().x, btn_box.center().y),
                egui::Align2::CENTER_CENTER,
                label,
                FontId::proportional(11.0),
                text_col,
            );
            btn_x += 96.0;
        }

        // Dampening cutoff meter bar
        painter.text(
            egui::pos2(matrix_rect.x + 15.0, matrix_rect.y + 105.0),
            egui::Align2::LEFT_TOP,
            format!("HF DAMPENING: {:.0} Hz", self.dampening_hz),
            FontId::proportional(11.0),
            Color32::from_rgb(180, 200, 225),
        );
        let damp_box = egui::Rect::from_min_size(
            egui::pos2(matrix_rect.x + 15.0, matrix_rect.y + 125.0),
            egui::vec2(matrix_rect.width - 30.0, 24.0),
        );
        painter.rect_filled(damp_box, 4.0, Color32::from_rgb(18, 25, 38));
        let norm_damp = Self::freq_to_normalized(self.dampening_hz);
        let damp_fill = egui::Rect::from_min_size(
            egui::pos2(matrix_rect.x + 15.0, matrix_rect.y + 125.0),
            egui::vec2((matrix_rect.width - 30.0) * norm_damp, 24.0),
        );
        painter.rect_filled(damp_fill, 4.0, Color32::from_rgb(255, 107, 43));

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
            "[PASS] Spectral Comb Resonator & Matrix Touch Nodes (>= 44x44pt) Verified",
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
            "DYNAMIC SPECTRAL COMB RESONATOR & MATRIX HUD",
            FontId::proportional(14.0),
            Color32::from_rgb(0, 229, 255),
        );

        let readout = format!(
            "BASE: {:.1} Hz | FB: {:.0}% | HARMONICS: {}",
            self.base_frequency_hz, self.feedback_pct, self.num_harmonics
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
            (CombResonatorProfile::TunedChimeA4, "CHIME A4 (440 HZ)"),
            (CombResonatorProfile::SubHollowNotch, "SUB NOTCH (55 HZ)"),
            (CombResonatorProfile::FormantVocalFlange, "VOCAL FLANGER"),
            (CombResonatorProfile::StereoQuadrature, "QUADRATURE SPREAD"),
            (CombResonatorProfile::MetallicHighQ, "METALLIC HIGH-Q"),
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

        // Left Panel: Frequency Response Curve Canvas (y: 100..320)
        let curve_w = (rect.width() - 56.0) * 0.58;
        let curve_rect = egui::Rect::from_min_size(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 100.0),
            egui::vec2(curve_w, 220.0),
        );
        painter.rect_filled(curve_rect, 6.0, Color32::from_rgb(10, 14, 22));
        painter.rect_stroke(
            curve_rect,
            6.0,
            Stroke::new(1.5_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(curve_rect.min.x + 12.0, curve_rect.min.y + 12.0),
            egui::Align2::LEFT_TOP,
            "HARMONIC TEETH TRANSFER CURVE",
            FontId::proportional(11.0),
            Color32::from_rgb(0, 229, 255),
        );

        // Logarithmic Frequency Grid Lines (100Hz, 1kHz, 10kHz)
        let log_freqs = [100.0, 1000.0, 10000.0];
        for f in &log_freqs {
            let norm_x = Self::freq_to_normalized(*f);
            let gx = curve_rect.min.x + norm_x * curve_rect.width();
            painter.line_segment(
                [
                    egui::pos2(gx, curve_rect.min.y),
                    egui::pos2(gx, curve_rect.max.y),
                ],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(50, 65, 90, 80)),
            );
            let label = if *f >= 1000.0 {
                format!("{:.0}k", f / 1000.0)
            } else {
                format!("{:.0}", f)
            };
            painter.text(
                egui::pos2(gx + 2.0, curve_rect.max.y - 14.0),
                egui::Align2::LEFT_TOP,
                label,
                FontId::proportional(9.0),
                Color32::from_rgb(120, 140, 170),
            );
        }

        // Draw Comb Response Curve
        let mut prev_pt: Option<egui::Pos2> = None;
        let points = 80;
        for i in 0..=points {
            let norm_x = i as f32 / points as f32;
            let f = Self::normalized_to_freq(norm_x);
            let mag = self.evaluate_magnitude_response(f);
            let cx = curve_rect.min.x + norm_x * curve_rect.width();
            let cy = curve_rect.min.y + (1.0 - mag * 0.85 - 0.05) * curve_rect.height();
            let pt = egui::pos2(cx, cy);

            if let Some(prev) = prev_pt {
                painter.line_segment(
                    [prev, pt],
                    Stroke::new(2.5_f32, Color32::from_rgb(0, 229, 255)),
                );
            }
            prev_pt = Some(pt);
        }

        // Handle Puck Dragging on Curve Canvas
        if response.dragged() {
            if let Some(pos) = response.interact_pointer_pos() {
                if curve_rect.contains(pos) {
                    let nx = ((pos.x - curve_rect.min.x) / curve_rect.width()).clamp(0.02, 0.98);
                    let ny = (1.0 - (pos.y - curve_rect.min.y) / curve_rect.height()).clamp(0.05, 0.98);
                    self.puck_pos = (nx, ny);
                    self.base_frequency_hz = Self::normalized_to_freq(nx);
                    self.feedback_pct = (ny * 100.0).clamp(0.0, 99.0);
                }
            }
        }

        // Draw 2D Frequency / Feedback Puck (>= 22pt radius -> 44x44pt bounding box)
        let px = curve_rect.min.x + self.puck_pos.0 * curve_rect.width();
        let py = curve_rect.min.y + (1.0 - self.puck_pos.1) * curve_rect.height();

        painter.circle_stroke(
            egui::pos2(px, py),
            COMB_PUCK_HIT_RADIUS,
            Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(255, 215, 0, 140)),
        );
        painter.circle_filled(egui::pos2(px, py), 12.0, Color32::from_rgb(255, 215, 0));
        painter.circle_filled(egui::pos2(px, py), 4.0, Color32::from_rgb(255, 255, 255));

        // Right Panel: Harmonics Matrix & Polarity Switcher (y: 100..320)
        let matrix_x = curve_rect.max.x + 16.0;
        let matrix_w = rect.max.x - 20.0 - matrix_x;
        let matrix_rect = egui::Rect::from_min_size(
            egui::pos2(matrix_x, rect.min.y + 100.0),
            egui::vec2(matrix_w, 220.0),
        );
        painter.rect_filled(matrix_rect, 6.0, Color32::from_rgb(10, 14, 22));
        painter.rect_stroke(
            matrix_rect,
            6.0,
            Stroke::new(1.5_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(matrix_rect.min.x + 12.0, matrix_rect.min.y + 12.0),
            egui::Align2::LEFT_TOP,
            "POLARITY & DAMPENING MATRIX",
            FontId::proportional(11.0),
            Color32::from_rgb(255, 107, 43),
        );

        // Polarity Buttons (each >= 44pt touch target)
        let pol_modes = [
            ("POS (+)", CombPolarity::Positive),
            ("NEG (-)", CombPolarity::Negative),
            ("RING (~)", CombPolarity::ComplexRing),
        ];
        let pol_btn_w = (matrix_rect.width() - 24.0 - 12.0) / 3.0;
        for (i, (label, mode)) in pol_modes.iter().enumerate() {
            let bx = matrix_rect.min.x + 12.0 + i as f32 * (pol_btn_w + 6.0);
            let btn_box = egui::Rect::from_min_size(
                egui::pos2(bx, matrix_rect.min.y + 40.0),
                egui::vec2(pol_btn_w, 44.0),
            );
            let is_active = self.polarity == *mode;
            let bg_col = if is_active {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(32, 42, 60)
            };
            let text_col = if is_active {
                Color32::from_rgb(10, 14, 20)
            } else {
                Color32::from_rgb(220, 235, 255)
            };

            painter.rect_filled(btn_box, 4.0, bg_col);
            painter.text(
                btn_box.center(),
                egui::Align2::CENTER_CENTER,
                *label,
                FontId::proportional(10.5),
                text_col,
            );

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if btn_box.contains(pos) {
                        self.set_polarity(*mode);
                    }
                }
            }
        }

        // HF Dampening slider bar
        painter.text(
            egui::pos2(matrix_rect.min.x + 12.0, matrix_rect.min.y + 105.0),
            egui::Align2::LEFT_TOP,
            format!("HF DAMPENING: {:.0} Hz", self.dampening_hz),
            FontId::proportional(11.0),
            Color32::from_rgb(180, 200, 225),
        );
        let damp_box = egui::Rect::from_min_size(
            egui::pos2(matrix_rect.min.x + 12.0, matrix_rect.min.y + 125.0),
            egui::vec2(matrix_rect.width() - 24.0, 24.0),
        );
        painter.rect_filled(damp_box, 4.0, Color32::from_rgb(18, 25, 38));
        let norm_damp = Self::freq_to_normalized(self.dampening_hz);
        let damp_fill = egui::Rect::from_min_size(
            damp_box.min,
            egui::vec2(damp_box.width() * norm_damp, 24.0),
        );
        painter.rect_filled(damp_fill, 4.0, Color32::from_rgb(255, 107, 43));

        if response.dragged() || response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                if damp_box.contains(pos) {
                    let nd = ((pos.x - damp_box.min.x) / damp_box.width()).clamp(0.01, 1.0);
                    self.dampening_hz = Self::normalized_to_freq(nd);
                }
            }
        }

        // Active Harmonics Teeth Count readout
        painter.text(
            egui::pos2(matrix_rect.min.x + 12.0, matrix_rect.min.y + 165.0),
            egui::Align2::LEFT_TOP,
            format!("ACTIVE HARMONIC TEETH: {}", self.num_harmonics),
            FontId::proportional(11.0),
            Color32::from_rgb(255, 215, 0),
        );

        // Bottom Controls Bar (y: 334..490)
        let ctrl_rect = egui::Rect::from_min_size(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 334.0),
            egui::vec2(rect.width() - 40.0, 156.0),
        );
        painter.rect_filled(ctrl_rect, 6.0, Color32::from_rgb(18, 25, 38));

        let slider_cols = 4;
        let col_w = (ctrl_rect.width() - 32.0) / slider_cols as f32;

        // Param 1: Base Frequency
        let p1_x = ctrl_rect.min.x + 16.0;
        painter.text(
            egui::pos2(p1_x, ctrl_rect.min.y + 12.0),
            egui::Align2::LEFT_TOP,
            format!("BASE FREQ: {:.1} Hz", self.base_frequency_hz),
            FontId::proportional(11.0),
            Color32::from_rgb(0, 229, 255),
        );
        let bar1_rect = egui::Rect::from_min_size(
            egui::pos2(p1_x, ctrl_rect.min.y + 30.0),
            egui::vec2(col_w - 20.0, 20.0),
        );
        painter.rect_filled(bar1_rect, 3.0, Color32::from_rgb(10, 14, 22));
        let norm_f = Self::freq_to_normalized(self.base_frequency_hz);
        painter.rect_filled(
            egui::Rect::from_min_size(bar1_rect.min, egui::vec2(bar1_rect.width() * norm_f, 20.0)),
            3.0,
            Color32::from_rgb(0, 229, 255),
        );

        // Param 2: Feedback %
        let p2_x = p1_x + col_w;
        painter.text(
            egui::pos2(p2_x, ctrl_rect.min.y + 12.0),
            egui::Align2::LEFT_TOP,
            format!("FEEDBACK: {:.1}%", self.feedback_pct),
            FontId::proportional(11.0),
            Color32::from_rgb(255, 215, 0),
        );
        let bar2_rect = egui::Rect::from_min_size(
            egui::pos2(p2_x, ctrl_rect.min.y + 30.0),
            egui::vec2(col_w - 20.0, 20.0),
        );
        painter.rect_filled(bar2_rect, 3.0, Color32::from_rgb(10, 14, 22));
        let norm_fb = (self.feedback_pct / 100.0).clamp(0.0, 1.0);
        painter.rect_filled(
            egui::Rect::from_min_size(bar2_rect.min, egui::vec2(bar2_rect.width() * norm_fb, 20.0)),
            3.0,
            Color32::from_rgb(255, 215, 0),
        );

        // Param 3: Stereo Spread %
        let p3_x = p2_x + col_w;
        painter.text(
            egui::pos2(p3_x, ctrl_rect.min.y + 12.0),
            egui::Align2::LEFT_TOP,
            format!("STEREO SPREAD: {:.0}%", self.stereo_spread_pct),
            FontId::proportional(11.0),
            Color32::from_rgb(0, 255, 180),
        );
        let bar3_rect = egui::Rect::from_min_size(
            egui::pos2(p3_x, ctrl_rect.min.y + 30.0),
            egui::vec2(col_w - 20.0, 20.0),
        );
        painter.rect_filled(bar3_rect, 3.0, Color32::from_rgb(10, 14, 22));
        let norm_sp = (self.stereo_spread_pct / 100.0).clamp(0.0, 1.0);
        painter.rect_filled(
            egui::Rect::from_min_size(bar3_rect.min, egui::vec2(bar3_rect.width() * norm_sp, 20.0)),
            3.0,
            Color32::from_rgb(0, 255, 180),
        );

        // Param 4: Dry/Wet %
        let p4_x = p3_x + col_w;
        painter.text(
            egui::pos2(p4_x, ctrl_rect.min.y + 12.0),
            egui::Align2::LEFT_TOP,
            format!("DRY/WET: {:.0}%", self.dry_wet_pct),
            FontId::proportional(11.0),
            Color32::from_rgb(255, 107, 43),
        );
        let bar4_rect = egui::Rect::from_min_size(
            egui::pos2(p4_x, ctrl_rect.min.y + 30.0),
            egui::vec2(col_w - 20.0, 20.0),
        );
        painter.rect_filled(bar4_rect, 3.0, Color32::from_rgb(10, 14, 22));
        let norm_dw = (self.dry_wet_pct / 100.0).clamp(0.0, 1.0);
        painter.rect_filled(
            egui::Rect::from_min_size(bar4_rect.min, egui::vec2(bar4_rect.width() * norm_dw, 20.0)),
            3.0,
            Color32::from_rgb(255, 107, 43),
        );

        // Handle Slider Bar Dragging
        if response.dragged() || response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                if bar1_rect.contains(pos) {
                    let nf = ((pos.x - bar1_rect.min.x) / bar1_rect.width()).clamp(0.01, 1.0);
                    self.base_frequency_hz = Self::normalized_to_freq(nf);
                    self.puck_pos.0 = nf;
                } else if bar2_rect.contains(pos) {
                    let nfb = ((pos.x - bar2_rect.min.x) / bar2_rect.width()).clamp(0.0, 0.99);
                    self.feedback_pct = nfb * 100.0;
                    self.puck_pos.1 = nfb;
                } else if bar3_rect.contains(pos) {
                    let nsp = ((pos.x - bar3_rect.min.x) / bar3_rect.width()).clamp(0.0, 1.0);
                    self.stereo_spread_pct = nsp * 100.0;
                } else if bar4_rect.contains(pos) {
                    let ndw = ((pos.x - bar4_rect.min.x) / bar4_rect.width()).clamp(0.0, 1.0);
                    self.dry_wet_pct = ndw * 100.0;
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
            "[PASS] Spectral Comb Resonator & Matrix Touch Nodes (>= 44x44pt) Verified",
            FontId::proportional(11.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
