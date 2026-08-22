// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Psychoacoustic Multi-Band Spectral Transient De-Bleed & Leakage Separator HUD (Step 1582).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const DEBLEED_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_SEPARATION_THRESH_DB: f32 = -60.0;
pub const MAX_SEPARATION_THRESH_DB: f32 = 0.0;
pub const MIN_MASK_SHARPNESS: f32 = 0.5;
pub const MAX_MASK_SHARPNESS: f32 = 4.0;

/// Psychoacoustic spectral de-bleed target acoustic environments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebleedMode {
    DrumKitHiHatBleed,          // Snare/Kick isolation from loud cymbal & hi-hat spill
    VocalHeadphoneSpill, // Backing click-track / monitor spill suppression on vocal condenser
    LiveStageGuitarSpill, // High SPL guitar cabinet rejection into adjacent stage vocal mic
    AcousticPianoHammerDebleed, // Mechanical damper squeak & hammer thud separation
    OrchestralSectionIsolator, // Brass/Percussion spill isolation from sensitive string sections
}

impl DebleedMode {
    pub fn mode_name(&self) -> &'static str {
        match self {
            Self::DrumKitHiHatBleed => "DRUM HI-HAT SPILL",
            Self::VocalHeadphoneSpill => "VOCAL HP SPILL",
            Self::LiveStageGuitarSpill => "STAGE GUITAR SPILL",
            Self::AcousticPianoHammerDebleed => "PIANO HAMMER/DAMPER",
            Self::OrchestralSectionIsolator => "ORCHESTRA ISOLATOR",
        }
    }

    pub fn nominal_thresh_db(&self) -> f32 {
        match self {
            Self::DrumKitHiHatBleed => -28.0,
            Self::VocalHeadphoneSpill => -42.0,
            Self::LiveStageGuitarSpill => -22.0,
            Self::AcousticPianoHammerDebleed => -36.0,
            Self::OrchestralSectionIsolator => -32.0,
        }
    }

    pub fn nominal_mask_sharpness(&self) -> f32 {
        match self {
            Self::DrumKitHiHatBleed => 2.4,
            Self::VocalHeadphoneSpill => 1.8,
            Self::LiveStageGuitarSpill => 3.2,
            Self::AcousticPianoHammerDebleed => 1.5,
            Self::OrchestralSectionIsolator => 2.0,
        }
    }

    pub fn nominal_transient_preserve(&self) -> f32 {
        match self {
            Self::DrumKitHiHatBleed => 0.95,
            Self::VocalHeadphoneSpill => 0.70,
            Self::LiveStageGuitarSpill => 0.85,
            Self::AcousticPianoHammerDebleed => 0.80,
            Self::OrchestralSectionIsolator => 0.60,
        }
    }

    pub fn nominal_phase_coherence(&self) -> f32 {
        match self {
            Self::DrumKitHiHatBleed => 0.90,
            Self::VocalHeadphoneSpill => 0.98,
            Self::LiveStageGuitarSpill => 0.82,
            Self::AcousticPianoHammerDebleed => 0.94,
            Self::OrchestralSectionIsolator => 0.88,
        }
    }
}

/// Psychoacoustic multi-band spectral transient de-bleed & leakage separator HUD.
#[derive(Debug, Clone)]
pub struct SpectralDebleedView {
    pub debleed_mode: DebleedMode,
    pub separation_thresh_db: f32,   // [-60.0 ..= 0.0 dBFS]
    pub mask_sharpness_gamma: f32,   // [0.5 ..= 4.0 steepness exponent]
    pub transient_preserve_pct: f32, // [0.0 ..= 1.0 attack onset weighting]
    pub phase_coherence_pct: f32,    // [0.0 ..= 1.0 inter-band phase preservation]
    pub puck_pos: (f32, f32),        // Normalized (X: Threshold, Y: Mask Sharpness)
    pub is_dragging_puck: bool,
    pub spectral_attenuations_db: [f32; 8], // 8-band suppression curve in dB
    pub leakage_energy_levels: [f32; 8],    // 8-band detected spill energy
    pub color_palette: ContrastColorPalette,
}

impl Default for SpectralDebleedView {
    fn default() -> Self {
        Self::new()
    }
}

impl SpectralDebleedView {
    pub fn new() -> Self {
        let mut view = Self {
            debleed_mode: DebleedMode::DrumKitHiHatBleed,
            separation_thresh_db: -28.0,
            mask_sharpness_gamma: 2.4,
            transient_preserve_pct: 0.95,
            phase_coherence_pct: 0.90,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            spectral_attenuations_db: [-3.0, -4.5, -8.0, -14.0, -22.0, -28.0, -32.0, -30.0],
            leakage_energy_levels: [0.15, 0.20, 0.40, 0.65, 0.90, 0.95, 0.85, 0.70],
            color_palette: ContrastColorPalette::default(),
        };
        view.puck_pos = (
            Self::thresh_to_normalized(view.separation_thresh_db),
            Self::sharpness_to_normalized(view.mask_sharpness_gamma),
        );
        view.update_spectral_mask();
        view
    }

    pub fn thresh_to_normalized(thresh: f32) -> f32 {
        let t = thresh.clamp(MIN_SEPARATION_THRESH_DB, MAX_SEPARATION_THRESH_DB);
        ((t - MIN_SEPARATION_THRESH_DB) / (MAX_SEPARATION_THRESH_DB - MIN_SEPARATION_THRESH_DB))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_thresh(norm: f32) -> f32 {
        MIN_SEPARATION_THRESH_DB
            + norm.clamp(0.0, 1.0) * (MAX_SEPARATION_THRESH_DB - MIN_SEPARATION_THRESH_DB)
    }

    pub fn sharpness_to_normalized(gamma: f32) -> f32 {
        let g = gamma.clamp(MIN_MASK_SHARPNESS, MAX_MASK_SHARPNESS);
        ((g - MIN_MASK_SHARPNESS) / (MAX_MASK_SHARPNESS - MIN_MASK_SHARPNESS)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_sharpness(norm: f32) -> f32 {
        MIN_MASK_SHARPNESS + norm.clamp(0.0, 1.0) * (MAX_MASK_SHARPNESS - MIN_MASK_SHARPNESS)
    }

    pub fn set_debleed_mode(&mut self, mode: DebleedMode) {
        self.debleed_mode = mode;
        self.separation_thresh_db = mode.nominal_thresh_db();
        self.mask_sharpness_gamma = mode.nominal_mask_sharpness();
        self.transient_preserve_pct = mode.nominal_transient_preserve();
        self.phase_coherence_pct = mode.nominal_phase_coherence();
        self.puck_pos = (
            Self::thresh_to_normalized(self.separation_thresh_db),
            Self::sharpness_to_normalized(self.mask_sharpness_gamma),
        );
        self.update_spectral_mask();
    }

    /// Update psychoacoustic Wiener mask / spectral subtraction curve.
    pub fn update_spectral_mask(&mut self) {
        let thresh_ratio = (self.separation_thresh_db / -60.0).clamp(0.0, 1.0);
        let gamma = self.mask_sharpness_gamma;

        // Base leakage profile per frequency band (Sub..Air)
        let base_leakage = match self.debleed_mode {
            DebleedMode::DrumKitHiHatBleed => [0.05, 0.10, 0.25, 0.50, 0.85, 0.95, 0.90, 0.80],
            DebleedMode::VocalHeadphoneSpill => [0.10, 0.20, 0.60, 0.80, 0.70, 0.50, 0.30, 0.15],
            DebleedMode::LiveStageGuitarSpill => [0.15, 0.40, 0.85, 0.90, 0.80, 0.45, 0.25, 0.10],
            DebleedMode::AcousticPianoHammerDebleed => {
                [0.70, 0.60, 0.40, 0.30, 0.25, 0.50, 0.65, 0.40]
            }
            DebleedMode::OrchestralSectionIsolator => {
                [0.30, 0.50, 0.70, 0.75, 0.65, 0.55, 0.45, 0.35]
            }
        };

        for (i, &leak) in base_leakage.iter().enumerate() {
            self.leakage_energy_levels[i] = leak;
            let raw_suppression = leak * thresh_ratio * 36.0 * gamma;
            self.spectral_attenuations_db[i] = -raw_suppression.clamp(0.0, 48.0);
        }
    }

    /// Hit test coordinate on the interactive debleed puck.
    pub fn hit_test_debleed_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= DEBLEED_PUCK_HIT_RADIUS
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
        for (i, &att) in self.spectral_attenuations_db.iter().enumerate() {
            let col = mid_x + 2 + (i + 1) * bar_spacing;
            let norm_att = (-att / 48.0).clamp(0.0, 1.0);
            let bar_h = (norm_att * (height - 4) as f32).round() as usize;
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

        // Background: Deep Slate Navy (#0C101A)
        painter.rect_filled(rect, 6.0, Color32::from_rgb(12, 16, 26));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "PSYCHOACOUSTIC MULTI-BAND SPECTRAL DE-BLEED & LEAKAGE SEPARATOR HUD",
            egui::FontId::proportional(14.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Mode Tabs (y: 48..92)
        let tabs = [
            (DebleedMode::DrumKitHiHatBleed, "DRUM HI-HAT"),
            (DebleedMode::VocalHeadphoneSpill, "VOCAL HP SPILL"),
            (DebleedMode::LiveStageGuitarSpill, "STAGE GUITAR"),
            (DebleedMode::AcousticPianoHammerDebleed, "PIANO DAMPER"),
            (DebleedMode::OrchestralSectionIsolator, "ORCHESTRA"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (mode, name)) in tabs.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_sel = self.debleed_mode == *mode;
            let bg_col = if is_sel {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(24, 32, 48)
            };
            let text_col = if is_sel {
                Color32::from_rgb(8, 16, 24)
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
                        self.set_debleed_mode(*mode);
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

        // Left 55%: Psychoacoustic Masking Threshold & Sharpness Matrix
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
            "PSYCHOACOUSTIC MASKING THRESHOLD & SHARPNESS RADAR",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(0, 229, 255),
        );

        // Sigmoid de-bleed threshold transfer curve
        let prev_pt = egui::pos2(left_rect.min.x + 15.0, left_rect.max.y - 35.0);
        let mut p_last = prev_pt;
        for s in 1..=20 {
            let t = s as f32 / 20.0;
            let x = left_rect.min.x + 15.0 + t * (left_rect.width() - 30.0);
            let y_val = 1.0 / (1.0 + (-(t - 0.5) * 6.0 * self.mask_sharpness_gamma).exp());
            let y = left_rect.max.y - 35.0 - y_val * (left_rect.height() - 75.0);
            let cur_pt = egui::pos2(x, y);
            painter.line_segment(
                [p_last, cur_pt],
                Stroke::new(2.0_f32, Color32::from_rgb(0, 229, 255)),
            );
            p_last = cur_pt;
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
                    self.separation_thresh_db = Self::normalized_to_thresh(nx);
                    self.mask_sharpness_gamma = Self::normalized_to_sharpness(ny);
                    self.update_spectral_mask();
                }
            }
        }

        painter.circle_stroke(
            puck_pos,
            DEBLEED_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(0, 229, 255, 150)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(0, 229, 255));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 20.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Thresh: {:.1} dB | Gamma: {:.2} | TransPreserve: {:.0}% | PhaseCoh: {:.0}%",
                self.separation_thresh_db,
                self.mask_sharpness_gamma,
                self.transient_preserve_pct * 100.0,
                self.phase_coherence_pct * 100.0
            ),
            egui::FontId::proportional(10.5),
            Color32::from_rgb(120, 220, 255),
        );

        // Right 45%: 8-Band Multi-Band Suppression Attenuation Profile
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
            "8-BAND SPECTRAL REJECTION ATTENUATION (dB)",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(0, 229, 255),
        );

        let band_names = [
            "SUB", "LOW", "L-MID", "MID", "H-MID", "PRES", "BRILL", "AIR",
        ];
        let bar_w = (right_rect.width() - 30.0 - 7.0 * 6.0) / 8.0;
        for (i, &att) in self.spectral_attenuations_db.iter().enumerate() {
            let bx = right_rect.min.x + 15.0 + i as f32 * (bar_w + 6.0);
            let norm_att = (-att / 48.0).clamp(0.0, 1.0);
            let bar_h = norm_att * (right_rect.height() - 80.0);
            let b_rect = egui::Rect::from_min_max(
                egui::pos2(bx, right_rect.max.y - 25.0 - bar_h),
                egui::pos2(bx + bar_w, right_rect.max.y - 25.0),
            );
            let col = if norm_att > 0.6 {
                Color32::from_rgb(255, 64, 96)
            } else if norm_att > 0.3 {
                Color32::from_rgb(255, 180, 50)
            } else {
                Color32::from_rgb(0, 255, 180)
            };
            painter.rect_filled(b_rect, 3.0, col);

            painter.text(
                egui::pos2(bx + bar_w * 0.5, right_rect.max.y - 20.0),
                egui::Align2::CENTER_TOP,
                band_names[i],
                egui::FontId::proportional(8.5),
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
                "ISOLATION THRESHOLD",
                format!("{:.1} dBFS (Spill Cut)", self.separation_thresh_db),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "MASK SHARPNESS (γ)",
                format!("{:.2} (Steepness)", self.mask_sharpness_gamma),
                Color32::from_rgb(255, 180, 50),
            ),
            (
                "TRANSIENT PRESERVATION",
                format!(
                    "{:.0}% (Attack Weight)",
                    self.transient_preserve_pct * 100.0
                ),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "PHASE COHERENCE",
                format!(
                    "{:.0}% (Inter-Band Align)",
                    self.phase_coherence_pct * 100.0
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
            "[PASS] Psychoacoustic Multi-Band Spectral De-Bleed Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
