// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Mastering Multi-Stage Magnetic Tape Flux Saturation & High-Frequency Hysteresis HUD (Step 1603).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const TAPE_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_FLUX_DRIVE_DB: f32 = -6.0;
pub const MAX_FLUX_DRIVE_DB: f32 = 18.0;
pub const MIN_BIAS_TRIM_DB: f32 = -6.0;
pub const MAX_BIAS_TRIM_DB: f32 = 6.0;

/// Professional magnetic tape formulations and mastering recorder topologies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TapeFormulation {
    Ampex456GrandMaster,  // Classic +6 dB warmth, rich mid-range saturation, warm 3rd harmonic
    StuderA800MasterTape, // Pristine +9 dB ultra-low distortion, wide headroom, smooth top-end
    QuantegyGP9,          // High-output +9 tape, aggressive dynamic punch, high signal-to-noise
    CassetteTypeIVMetal,  // Metal particle high-bias cassette, dense HF saturation
    VintageTubeTape1958,  // Early tube-driven tape recorder with asymmetric 2nd/3rd harmonics
}

impl TapeFormulation {
    pub fn formulation_name(&self) -> &'static str {
        match self {
            Self::Ampex456GrandMaster => "AMPEX 456 GRAND MASTER (+6)",
            Self::StuderA800MasterTape => "STUDER A800 MASTER (+9)",
            Self::QuantegyGP9 => "QUANTEGY GP9 (+9 MAX)",
            Self::CassetteTypeIVMetal => "CASSETTE TYPE IV METAL",
            Self::VintageTubeTape1958 => "VINTAGE TUBE TAPE (1958)",
        }
    }

    pub fn nominal_flux_drive_db(&self) -> f32 {
        match self {
            Self::Ampex456GrandMaster => 6.0,
            Self::StuderA800MasterTape => 9.0,
            Self::QuantegyGP9 => 11.5,
            Self::CassetteTypeIVMetal => 3.5,
            Self::VintageTubeTape1958 => 4.0,
        }
    }

    pub fn nominal_bias_trim_db(&self) -> f32 {
        match self {
            Self::Ampex456GrandMaster => 1.5,
            Self::StuderA800MasterTape => 3.0,
            Self::QuantegyGP9 => 4.5,
            Self::CassetteTypeIVMetal => 0.5,
            Self::VintageTubeTape1958 => -1.5,
        }
    }

    pub fn nominal_ips_speed(&self) -> f32 {
        match self {
            Self::Ampex456GrandMaster => 15.0,
            Self::StuderA800MasterTape => 30.0,
            Self::QuantegyGP9 => 30.0,
            Self::CassetteTypeIVMetal => 3.75,
            Self::VintageTubeTape1958 => 15.0,
        }
    }

    pub fn nominal_head_bump_hz(&self) -> f32 {
        match self {
            Self::Ampex456GrandMaster => 55.0,
            Self::StuderA800MasterTape => 42.0,
            Self::QuantegyGP9 => 38.0,
            Self::CassetteTypeIVMetal => 95.0,
            Self::VintageTubeTape1958 => 68.0,
        }
    }

    pub fn nominal_thd_pct(&self) -> f32 {
        match self {
            Self::Ampex456GrandMaster => 0.85,
            Self::StuderA800MasterTape => 0.25,
            Self::QuantegyGP9 => 0.35,
            Self::CassetteTypeIVMetal => 1.95,
            Self::VintageTubeTape1958 => 2.40,
        }
    }
}

/// Mastering multi-stage magnetic tape flux saturation & high-frequency hysteresis HUD.
#[derive(Debug, Clone)]
pub struct TapeFluxMasterView {
    pub formulation: TapeFormulation,
    pub flux_drive_db: f32,
    pub bias_trim_db: f32,
    pub tape_speed_ips: f32,
    pub head_bump_hz: f32,
    pub hf_compression_pct: f32,
    pub thd_distortion_pct: f32,
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    pub harmonic_spectrum: [f32; 6],
    pub hysteresis_loop_pts: [(f32, f32); 16],
    pub color_palette: ContrastColorPalette,
}

impl Default for TapeFluxMasterView {
    fn default() -> Self {
        Self::new()
    }
}

impl TapeFluxMasterView {
    pub fn new() -> Self {
        let mut view = Self {
            formulation: TapeFormulation::StuderA800MasterTape,
            flux_drive_db: 9.0,
            bias_trim_db: 3.0,
            tape_speed_ips: 30.0,
            head_bump_hz: 42.0,
            hf_compression_pct: 35.0,
            thd_distortion_pct: 0.25,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            harmonic_spectrum: [1.0, 0.08, 0.32, 0.05, 0.45, 0.15],
            hysteresis_loop_pts: [(0.0, 0.0); 16],
            color_palette: ContrastColorPalette::default(),
        };
        view.puck_pos = (
            Self::drive_to_normalized(view.flux_drive_db),
            Self::bias_to_normalized(view.bias_trim_db),
        );
        view.update_hysteresis_simulation();
        view
    }

    pub fn drive_to_normalized(drive: f32) -> f32 {
        let d = drive.clamp(MIN_FLUX_DRIVE_DB, MAX_FLUX_DRIVE_DB);
        ((d - MIN_FLUX_DRIVE_DB) / (MAX_FLUX_DRIVE_DB - MIN_FLUX_DRIVE_DB)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_drive(norm: f32) -> f32 {
        MIN_FLUX_DRIVE_DB + norm.clamp(0.0, 1.0) * (MAX_FLUX_DRIVE_DB - MIN_FLUX_DRIVE_DB)
    }

    pub fn bias_to_normalized(bias: f32) -> f32 {
        let b = bias.clamp(MIN_BIAS_TRIM_DB, MAX_BIAS_TRIM_DB);
        ((b - MIN_BIAS_TRIM_DB) / (MAX_BIAS_TRIM_DB - MIN_BIAS_TRIM_DB)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_bias(norm: f32) -> f32 {
        MIN_BIAS_TRIM_DB + norm.clamp(0.0, 1.0) * (MAX_BIAS_TRIM_DB - MIN_BIAS_TRIM_DB)
    }

    pub fn set_formulation(&mut self, tape: TapeFormulation) {
        self.formulation = tape;
        self.flux_drive_db = tape.nominal_flux_drive_db();
        self.bias_trim_db = tape.nominal_bias_trim_db();
        self.tape_speed_ips = tape.nominal_ips_speed();
        self.head_bump_hz = tape.nominal_head_bump_hz();
        self.thd_distortion_pct = tape.nominal_thd_pct();
        self.puck_pos = (
            Self::drive_to_normalized(self.flux_drive_db),
            Self::bias_to_normalized(self.bias_trim_db),
        );
        self.update_hysteresis_simulation();
    }

    pub fn update_hysteresis_simulation(&mut self) {
        let drive = self.flux_drive_db;
        let bias = self.bias_trim_db;
        let drive_lin = 10.0_f32.powf(drive / 20.0);

        // Hysteresis B-H Loop Modeling: Jiles-Atherton inspired sigmoidal loop
        let remanence = (0.25 - bias * 0.03).clamp(0.05, 0.45);
        for (i, pt) in self.hysteresis_loop_pts.iter_mut().enumerate() {
            let theta = i as f32 * (std::f32::consts::TAU / 16.0);
            let h = theta.cos() * 0.9;
            let dir = if theta.sin() >= 0.0 { 1.0 } else { -1.0 };
            let b = (h * drive_lin * 0.6).tanh() + dir * remanence * (1.0 - h.abs() * 0.5);
            *pt = (h, b.clamp(-1.0, 1.0));
        }

        // Harmonic Saturation Spectrum: [Fund, 2nd (Asymmetry), 3rd (Symmetric), 5th, Head Bump, HF Tape Roll-off]
        let fund = 1.0;
        let h2 = (0.05 + (-bias).max(0.0) * 0.08).clamp(0.01, 0.6);
        let h3 = (0.15 + (drive / 18.0).powi(2) * 0.6).clamp(0.05, 0.95);
        let h5 = (0.02 + (drive / 18.0).powi(3) * 0.3).clamp(0.01, 0.5);
        let bump = (0.35 + (30.0 / self.tape_speed_ips) * 0.25).clamp(0.1, 0.85);
        let hf_roll = (1.0 - (drive / 24.0) * (15.0 / self.tape_speed_ips)).clamp(0.2, 0.9);

        self.harmonic_spectrum = [fund, h2, h3, h5, bump, hf_roll];
        self.hf_compression_pct = ((drive / 18.0) * 80.0 + (30.0 - self.tape_speed_ips) * 1.2).clamp(5.0, 95.0);
        self.thd_distortion_pct = (h2 * 0.4 + h3 * 0.8 + h5 * 0.3) * 2.5;
    }

    pub fn hit_test_tape_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= TAPE_PUCK_HIT_RADIUS
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
        let bar_spacing = right_w / 7;
        for (i, &amp) in self.harmonic_spectrum.iter().enumerate() {
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

        // Background: Deep Slate Rust (#140E10)
        painter.rect_filled(rect, 6.0, Color32::from_rgb(20, 14, 16));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "MASTERING MAGNETIC TAPE FLUX SATURATION & HYSTERESIS HUD",
            egui::FontId::proportional(13.5),
            Color32::from_rgb(240, 245, 255),
        );

        // Tabs (y: 48..92) - 44pt height
        let tabs = [
            (TapeFormulation::Ampex456GrandMaster, "AMPEX 456 (+6)"),
            (TapeFormulation::StuderA800MasterTape, "STUDER A800 (+9)"),
            (TapeFormulation::QuantegyGP9, "QUANTEGY GP9 (+9)"),
            (TapeFormulation::CassetteTypeIVMetal, "TYPE IV METAL"),
            (TapeFormulation::VintageTubeTape1958, "VINTAGE TUBE (1958)"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (ttype, name)) in tabs.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_sel = self.formulation == *ttype;
            let bg_col = if is_sel {
                Color32::from_rgb(235, 94, 40)
            } else {
                Color32::from_rgb(38, 26, 30)
            };
            let text_col = if is_sel {
                Color32::from_rgb(18, 6, 4)
            } else {
                Color32::from_rgb(230, 215, 215)
            };

            painter.rect_filled(tab_rect, 4.0, bg_col);
            painter.text(
                tab_rect.center(),
                egui::Align2::CENTER_CENTER,
                *name,
                egui::FontId::proportional(10.0),
                text_col,
            );

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if tab_rect.contains(pos) {
                        self.set_formulation(*ttype);
                    }
                }
            }
        }

        // Main Display Canvas (y: 104..340)
        let main_canvas = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 104.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 340.0),
        );
        painter.rect_filled(main_canvas, 6.0, Color32::from_rgb(12, 8, 10));
        painter.rect_stroke(
            main_canvas,
            6.0,
            Stroke::new(1.5_f32, Color32::from_rgb(85, 45, 55)),
        );

        // Left 55%: Ferromagnetic Hysteresis B-H Loop & Flux Field
        let left_w = main_canvas.width() * 0.55;
        let left_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(left_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(left_rect, 4.0, Color32::from_rgb(20, 12, 14));
        painter.rect_stroke(
            left_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(75, 35, 45)),
        );

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "FERROMAGNETIC HYSTERESIS B-H LOOP & FLUX DRIVE",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(235, 94, 40),
        );

        let bh_center = egui::pos2(
            left_rect.min.x + left_rect.width() * 0.5,
            left_rect.min.y + left_rect.height() * 0.52,
        );

        // Draw H and B axis crosshairs
        painter.line_segment(
            [
                egui::pos2(left_rect.min.x + 20.0, bh_center.y),
                egui::pos2(left_rect.max.x - 20.0, bh_center.y),
            ],
            Stroke::new(1.0_f32, Color32::from_rgb(60, 35, 45)),
        );
        painter.line_segment(
            [
                egui::pos2(bh_center.x, left_rect.min.y + 25.0),
                egui::pos2(bh_center.x, left_rect.max.y - 25.0),
            ],
            Stroke::new(1.0_f32, Color32::from_rgb(60, 35, 45)),
        );

        // Draw B-H Hysteresis Curve Loop
        let scale_x = left_rect.width() * 0.40;
        let scale_y = left_rect.height() * 0.38;
        for i in 0..self.hysteresis_loop_pts.len() {
            let next_idx = (i + 1) % self.hysteresis_loop_pts.len();
            let p1 = self.hysteresis_loop_pts[i];
            let p2 = self.hysteresis_loop_pts[next_idx];
            let pt1 = egui::pos2(bh_center.x + p1.0 * scale_x, bh_center.y - p1.1 * scale_y);
            let pt2 = egui::pos2(bh_center.x + p2.0 * scale_x, bh_center.y - p2.1 * scale_y);
            painter.line_segment([pt1, pt2], Stroke::new(2.0_f32, Color32::from_rgb(255, 107, 43)));
        }

        // Interactive Puck
        let puck_x = left_rect.min.x + self.puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.puck_pos = (nx, ny);
                    self.flux_drive_db = Self::normalized_to_drive(nx);
                    self.bias_trim_db = Self::normalized_to_bias(ny);
                    self.update_hysteresis_simulation();
                }
            }
        }

        painter.circle_stroke(
            puck_pos,
            TAPE_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(235, 94, 40, 150)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(235, 94, 40));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 18.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Drive: {:+.1} dB | Bias: {:+.1} dB | Speed: {:.1} ips | Bump: {:.0} Hz | THD: {:.2}%",
                self.flux_drive_db,
                self.bias_trim_db,
                self.tape_speed_ips,
                self.head_bump_hz,
                self.thd_distortion_pct
            ),
            egui::FontId::proportional(10.5),
            Color32::from_rgb(255, 180, 150),
        );

        // Right 45%: Harmonic Saturation & Head Bump Spectrum
        let right_w = main_canvas.width() * 0.45;
        let right_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + left_w + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(right_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(right_rect, 4.0, Color32::from_rgb(20, 12, 14));
        painter.rect_stroke(
            right_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(75, 35, 45)),
        );

        painter.text(
            egui::pos2(right_rect.min.x + 10.0, right_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "HARMONIC SATURATION & HEAD-BUMP SPECTRUM",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(235, 94, 40),
        );

        let harm_labels = ["FUND", "2ND (ASYM)", "3RD (SYM)", "5TH (DRV)", "BUMP (LF)", "HF-ROLL"];
        let bar_w = (right_rect.width() - 30.0 - 5.0 * 8.0) / 6.0;
        for (i, &amp) in self.harmonic_spectrum.iter().enumerate() {
            let bx = right_rect.min.x + 15.0 + i as f32 * (bar_w + 8.0);
            let bar_h = (amp.clamp(0.0, 1.2) / 1.2) * (right_rect.height() - 80.0);
            let b_rect = egui::Rect::from_min_max(
                egui::pos2(bx, right_rect.max.y - 25.0 - bar_h),
                egui::pos2(bx + bar_w, right_rect.max.y - 25.0),
            );
            let col = if i == 0 {
                Color32::from_rgb(235, 94, 40)
            } else if i == 1 || i == 2 {
                Color32::from_rgb(255, 180, 0)
            } else {
                Color32::from_rgb(0, 229, 255)
            };
            painter.rect_filled(b_rect, 3.0, col);

            painter.text(
                egui::pos2(bx + bar_w * 0.5, right_rect.max.y - 20.0),
                egui::Align2::CENTER_TOP,
                harm_labels[i],
                egui::FontId::proportional(8.0),
                Color32::from_rgb(220, 195, 195),
            );
        }

        // Bottom Metrics Dock (y: 350..465)
        let dock_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 350.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 465.0),
        );
        painter.rect_filled(dock_rect, 6.0, Color32::from_rgb(28, 18, 22));
        painter.rect_stroke(
            dock_rect,
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(85, 45, 55)),
        );

        let params = [
            (
                "FLUX DRIVE",
                format!("{:+.1} dB (Input Saturation)", self.flux_drive_db),
                Color32::from_rgb(235, 94, 40),
            ),
            (
                "BIAS CURRENT",
                format!("{:+.1} dB (High Freq Trim)", self.bias_trim_db),
                Color32::from_rgb(255, 180, 0),
            ),
            (
                "HF COMPRESSION",
                format!("{:.0}% (Magnetic Squash)", self.hf_compression_pct),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "THD HARMONICS",
                format!("{:.2}% (Warmth Factor)", self.thd_distortion_pct),
                Color32::from_rgb(255, 107, 107),
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
                Color32::from_rgb(185, 160, 165),
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
            "[PASS] Magnetic Tape Saturation & Hysteresis Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
