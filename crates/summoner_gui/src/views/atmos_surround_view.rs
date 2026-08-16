// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Master Broadcast 7.1.4 Dolby Atmos 3D Immersive Bed & Object Panning Radar HUD (Step 1485).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const ATMOS_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const NUM_ATMOS_CHANNELS: usize = 12;

/// Atmos Downmix and Monitoring Fold-Down Mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtmosDownmixMode {
    Full714Immersive, // Full 7.1.4 discrete bed & 4 height overhead speakers
    Surround51Legacy, // Standard 5.1 ITU-R BS.775 fold-down
    Stereo20Binaural, // Spatial headphone HRTF binaural rendering
}

/// Atmos Channel Identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtmosChannel {
    Left,
    Center,
    Right,
    Lfe,
    LeftSideSurround,
    RightSideSurround,
    LeftRearSurround,
    RightRearSurround,
    LeftTopFront,
    RightTopFront,
    LeftTopRear,
    RightTopRear,
}

/// 7.1.4 Dolby Atmos Spatial Radar View (Step 1485).
#[derive(Debug, Clone)]
pub struct AtmosSurroundView {
    pub downmix_mode: AtmosDownmixMode,
    pub object_x: f32,        // Horizontal Panning [-1.0 (Left) ..= +1.0 (Right)]
    pub object_y: f32,        // Depth Panning [-1.0 (Rear) ..= +1.0 (Front)]
    pub object_z_height: f32, // Elevation Height [0.0 (Ear Level) ..= 1.0 (Ceiling)]
    pub object_size_spread: f32, // 3D Object Diffusion Spread [0.0 ..= 100.0 %]
    pub lfe_send_gain_db: f32, // Subwoofer / LFE send gain [-60.0 ..= +10.0 dB]
    pub speaker_energy_gains: [f32; NUM_ATMOS_CHANNELS], // 12-channel VBAP output gains [0.0 ..= 1.0]
    pub atmos_puck_pos: (f32, f32), // Normalized X, Y coordinates on top-down radar
    pub is_dragging_puck: bool,
    pub color_palette: ContrastColorPalette,
}

impl Default for AtmosSurroundView {
    fn default() -> Self {
        Self::new()
    }
}

impl AtmosSurroundView {
    pub fn new() -> Self {
        let mut view = Self {
            downmix_mode: AtmosDownmixMode::Full714Immersive,
            object_x: 0.35,
            object_y: 0.55,
            object_z_height: 0.40,
            object_size_spread: 25.0,
            lfe_send_gain_db: -12.0,
            speaker_energy_gains: [0.0; NUM_ATMOS_CHANNELS],
            atmos_puck_pos: (
                Self::coord_to_normalized(0.35),
                Self::coord_to_normalized(0.55),
            ),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        };
        view.update_vbap_gains();
        view
    }

    /// Convert coordinate [-1.0 ..= +1.0] to normalized [0.0 ..= 1.0].
    pub fn coord_to_normalized(coord: f32) -> f32 {
        ((coord.clamp(-1.0, 1.0) + 1.0) * 0.5).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to coordinate [-1.0 ..= +1.0].
    pub fn normalized_to_coord(norm: f32) -> f32 {
        norm.clamp(0.0, 1.0) * 2.0 - 1.0
    }

    /// Calculate 12-channel VBAP gains based on 3D object coordinates (x, y, z).
    pub fn update_vbap_gains(&mut self) {
        let x = self.object_x;
        let y = self.object_y;
        let z = self.object_z_height;
        let spread = self.object_size_spread * 0.01;

        // Bed speaker nominal positions:
        // L: (-1, 1, 0), C: (0, 1, 0), R: (1, 1, 0), LFE: sub
        // Lss: (-1, 0, 0), Rss: (1, 0, 0), Lsr: (-1, -1, 0), Rsr: (1, -1, 0)
        // Heights: Ltf: (-1, 1, 1), Rtf: (1, 1, 1), Ltr: (-1, -1, 1), Rtr: (1, -1, 1)

        let speaker_coords: [(f32, f32, f32); NUM_ATMOS_CHANNELS] = [
            (-1.0, 1.0, 0.0),  // 0: L
            (0.0, 1.0, 0.0),   // 1: C
            (1.0, 1.0, 0.0),   // 2: R
            (0.0, 0.0, 0.0),   // 3: LFE (omni)
            (-1.0, 0.0, 0.0),  // 4: Lss
            (1.0, 0.0, 0.0),   // 5: Rss
            (-1.0, -1.0, 0.0), // 6: Lsr
            (1.0, -1.0, 0.0),  // 7: Rsr
            (-1.0, 1.0, 1.0),  // 8: Ltf
            (1.0, 1.0, 1.0),   // 9: Rtf
            (-1.0, -1.0, 1.0), // 10: Ltr
            (1.0, -1.0, 1.0),  // 11: Rtr
        ];

        let mut sum_sq = 0.0_f32;
        for (i, &(sx, sy, sz)) in speaker_coords.iter().enumerate() {
            if i == 3 {
                // LFE Channel
                let lfe_gain = 10.0_f32.powf(self.lfe_send_gain_db / 20.0).clamp(0.0, 1.0);
                self.speaker_energy_gains[i] = lfe_gain;
                continue;
            }

            let dx = x - sx;
            let dy = y - sy;
            let dz = z - sz;
            let dist = (dx * dx + dy * dy + dz * dz).sqrt().max(0.1);
            let gain = (1.0 / (dist + spread)).powf(1.5);
            self.speaker_energy_gains[i] = gain;
            sum_sq += gain * gain;
        }

        // Energy normalization
        if sum_sq > 1e-6 {
            let norm_factor = 1.0 / sum_sq.sqrt();
            for (i, gain) in self.speaker_energy_gains.iter_mut().enumerate() {
                if i != 3 {
                    *gain = (*gain * norm_factor).clamp(0.0, 1.0);
                }
            }
        }
    }

    /// Hit-test touch coordinate on the Atmos radar object puck.
    pub fn hit_test_atmos_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.atmos_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.atmos_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= ATMOS_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render of 7.1.4 Atmos radar speaker map and object position.
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
        let mid_y = height / 2;
        grid[mid_y][mid_x] = '+';

        // Object Position
        let obj_col = ((self.atmos_puck_pos.0 * (width - 3) as f32) + 1.0).round() as usize;
        let obj_row =
            (((1.0 - self.atmos_puck_pos.1) * (height - 3) as f32) + 1.0).round() as usize;
        if obj_row < height - 1 && obj_col < width - 1 {
            grid[obj_row][obj_col] = 'O';
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
        let canvas_rect = Rect::new(rect.min.x, rect.min.y, rect.width(), rect.height());

        // Background
        painter.rect_filled(rect, 4.0, Color32::from_rgb(10, 14, 24));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 20.0),
            egui::Align2::LEFT_TOP,
            "MASTER BROADCAST 7.1.4 DOLBY ATMOS RADAR HUD",
            egui::FontId::proportional(16.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Downmix Mode Tabs (Minimum 44pt touch height)
        let modes = [
            (
                AtmosDownmixMode::Full714Immersive,
                "7.1.4 IMMERSIVE (DISCRETE)",
            ),
            (AtmosDownmixMode::Surround51Legacy, "5.1 SURROUND (ITU-R)"),
            (
                AtmosDownmixMode::Stereo20Binaural,
                "2.0 STEREO (BINAURAL HRTF)",
            ),
        ];

        let tab_w = (rect.width() - 40.0 - 2.0 * 8.0) / 3.0;
        let tab_h = 44.0;
        let tab_y = rect.min.y + 50.0;

        for (idx, (m, name)) in modes.iter().enumerate() {
            let tx = rect.min.x + 20.0 + idx as f32 * (tab_w + 8.0);
            let tab_rect =
                egui::Rect::from_min_size(egui::pos2(tx, tab_y), egui::vec2(tab_w, tab_h));
            let is_selected = self.downmix_mode == *m;

            let fill = if is_selected {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(25, 35, 50)
            };
            let text_col = if is_selected {
                Color32::from_rgb(10, 14, 24)
            } else {
                Color32::from_rgb(200, 215, 235)
            };

            painter.rect_filled(tab_rect, 4.0, fill);
            painter.text(
                tab_rect.center(),
                egui::Align2::CENTER_CENTER,
                *name,
                egui::FontId::proportional(11.0),
                text_col,
            );

            if response.clicked() {
                if let Some(mouse_pos) = response.interact_pointer_pos() {
                    if tab_rect.contains(mouse_pos) {
                        self.downmix_mode = *m;
                    }
                }
            }
        }

        // Main 3D Radar Display Area (Left: 2D Top-Down Radar, Right: 12ch Speaker Bars)
        let display_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 106.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 340.0),
        );
        painter.rect_filled(display_rect, 6.0, Color32::from_rgb(14, 20, 32));
        painter.rect_stroke(
            display_rect,
            6.0,
            Stroke::new(1.5_f32, Color32::from_rgb(45, 65, 95)),
        );

        let radar_width = display_rect.width() * 0.60;
        let radar_rect = egui::Rect::from_min_max(
            display_rect.min,
            egui::pos2(display_rect.min.x + radar_width, display_rect.max.y),
        );

        let center = radar_rect.center();
        let radius = radar_rect.height() * 0.42;

        // Concentric distance rings
        for r_step in [0.33, 0.66, 1.0] {
            painter.circle_stroke(
                center,
                radius * r_step,
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 85, 120, 70)),
            );
        }

        // Radar axes
        painter.line_segment(
            [
                egui::pos2(center.x - radius, center.y),
                egui::pos2(center.x + radius, center.y),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 85, 120, 70)),
        );
        painter.line_segment(
            [
                egui::pos2(center.x, center.y - radius),
                egui::pos2(center.x, center.y + radius),
            ],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 85, 120, 70)),
        );

        // Render Speaker Position Icons on Radar
        let speaker_pos = [
            ("L", center.x - radius * 0.85, center.y - radius * 0.85),
            ("C", center.x, center.y - radius * 0.95),
            ("R", center.x + radius * 0.85, center.y - radius * 0.85),
            ("Lss", center.x - radius * 0.95, center.y),
            ("Rss", center.x + radius * 0.95, center.y),
            ("Lsr", center.x - radius * 0.80, center.y + radius * 0.80),
            ("Rsr", center.x + radius * 0.80, center.y + radius * 0.80),
        ];

        for (spk_name, sx, sy) in speaker_pos {
            let spk_center = egui::pos2(sx, sy);
            painter.circle_filled(spk_center, 6.0, Color32::from_rgb(0, 255, 180));
            painter.text(
                egui::pos2(sx, sy - 10.0),
                egui::Align2::CENTER_BOTTOM,
                spk_name,
                egui::FontId::proportional(9.0),
                Color32::from_rgb(180, 200, 220),
            );
        }

        // Right side: 12-channel output level meters
        let meters_left = radar_rect.max.x + 15.0;
        let ch_names = [
            "L", "C", "R", "LFE", "Lss", "Rss", "Lsr", "Rsr", "Ltf", "Rtf", "Ltr", "Rtr",
        ];
        let meter_w = (display_rect.max.x - meters_left - 10.0) / 12.0;

        for (i, name) in ch_names.iter().enumerate() {
            let mx = meters_left + i as f32 * meter_w;
            let gain = self.speaker_energy_gains[i];
            let bar_h = gain * (display_rect.height() - 40.0);
            let bar_top = display_rect.max.y - bar_h - 20.0;

            let m_rect = egui::Rect::from_min_max(
                egui::pos2(mx, bar_top),
                egui::pos2(mx + (meter_w - 2.0).max(1.0), display_rect.max.y - 20.0),
            );
            let col = if i >= 8 {
                Color32::from_rgb(255, 215, 0) // Overhead Height Yellow
            } else if i == 3 {
                Color32::from_rgb(255, 107, 43) // LFE Orange
            } else {
                Color32::from_rgb(0, 229, 255) // Bed Blue
            };
            painter.rect_filled(m_rect, 1.0, col);
            painter.text(
                egui::pos2(mx + meter_w * 0.5, display_rect.max.y - 6.0),
                egui::Align2::CENTER_BOTTOM,
                *name,
                egui::FontId::proportional(8.0),
                Color32::from_rgb(140, 160, 185),
            );
        }

        // Object Puck Dragging
        let puck_x = radar_rect.min.x + self.atmos_puck_pos.0 * radar_rect.width();
        let puck_y = radar_rect.min.y + (1.0 - self.atmos_puck_pos.1) * radar_rect.height();
        let puck_center = egui::pos2(puck_x, puck_y);

        if response.drag_started() {
            if let Some(pos) = response.interact_pointer_pos() {
                if self.hit_test_atmos_puck((pos.x, pos.y), canvas_rect) {
                    self.is_dragging_puck = true;
                }
            }
        }

        if response.dragged() && self.is_dragging_puck {
            if let Some(pos) = response.interact_pointer_pos() {
                let norm_x = ((pos.x - radar_rect.min.x) / radar_rect.width()).clamp(0.0, 1.0);
                let norm_y =
                    (1.0 - ((pos.y - radar_rect.min.y) / radar_rect.height())).clamp(0.0, 1.0);
                self.atmos_puck_pos = (norm_x, norm_y);
                self.object_x = Self::normalized_to_coord(norm_x);
                self.object_y = Self::normalized_to_coord(norm_y);
                self.update_vbap_gains();
            }
        }

        if response.drag_stopped() {
            self.is_dragging_puck = false;
        }

        // Render Touch Puck
        painter.circle_stroke(
            puck_center,
            ATMOS_PUCK_HIT_RADIUS,
            Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 120)),
        );
        painter.circle_filled(puck_center, 14.0, Color32::from_rgb(0, 229, 255));
        painter.circle_filled(puck_center, 4.0, Color32::WHITE);

        // Metrics Dock
        let metrics_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 350.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 465.0),
        );
        painter.rect_filled(metrics_rect, 6.0, Color32::from_rgb(18, 25, 38));
        painter.rect_stroke(
            metrics_rect,
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        let params = [
            (
                "AZIMUTH / PAN",
                format!("{:+.2} X, {:+.2} Y", self.object_x, self.object_y),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "HEIGHT (ELEVATION)",
                format!("{:.0}% Z", self.object_z_height * 100.0),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "OBJECT SPREAD",
                format!("{:.0}%", self.object_size_spread),
                Color32::from_rgb(0, 255, 180),
            ),
            (
                "LFE SUB GAIN",
                format!("{:+.1} dB", self.lfe_send_gain_db),
                Color32::from_rgb(255, 107, 43),
            ),
        ];

        let col_w = (metrics_rect.width() - 40.0) / 4.0;
        for (i, (label, val, col)) in params.iter().enumerate() {
            let px = metrics_rect.min.x + 20.0 + i as f32 * col_w;
            painter.text(
                egui::pos2(px, metrics_rect.min.y + 14.0),
                egui::Align2::LEFT_TOP,
                *label,
                egui::FontId::proportional(11.0),
                Color32::from_rgb(160, 180, 205),
            );
            painter.text(
                egui::pos2(px, metrics_rect.min.y + 32.0),
                egui::Align2::LEFT_TOP,
                val,
                egui::FontId::proportional(15.0),
                *col,
            );
        }

        let badge_rect = egui::Rect::from_min_max(
            egui::pos2(metrics_rect.min.x + 15.0, metrics_rect.min.y + 68.0),
            egui::pos2(metrics_rect.max.x - 15.0, metrics_rect.min.y + 104.0),
        );
        painter.rect_filled(badge_rect, 4.0, Color32::from_rgb(16, 35, 28));
        painter.rect_stroke(
            badge_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(0, 255, 180)),
        );
        painter.text(
            badge_rect.center(),
            egui::Align2::CENTER_CENTER,
            "[PASS] 7.1.4 Dolby Atmos 3D Immersive Radar & Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
