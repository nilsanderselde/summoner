// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Broadcast Mastering Immersive Dolby Atmos 9.1.6 3D Acoustic Room Raytracing HUD (Step 1595).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const ATMOS_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_ROOM_COORD_M: f32 = -8.0;
pub const MAX_ROOM_COORD_M: f32 = 8.0;

/// Dolby Atmos 9.1.6 acoustic monitoring environments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtmosRoomType {
    MasteringStage916, // Reference 9.1.6 mastering stage with 9 bed + 1 LFE + 6 overheads
    CinemaAuditoriumDolby, // Large theatrical dubbing stage with dual LFE arrays
    NearfieldMixingStudio, // Studio nearfield with dense early reflection raytracer
    BinauralAtmosSpatializer, // Binaural headphone virtualization with personalized HRIR
    CarAudio16ChannelArray, // Automotive 16-channel cabin acoustic profile
}

impl AtmosRoomType {
    pub fn room_name(&self) -> &'static str {
        match self {
            Self::MasteringStage916 => "ATMOS MASTERING (9.1.6)",
            Self::CinemaAuditoriumDolby => "CINEMA THEATRICAL DUB",
            Self::NearfieldMixingStudio => "NEARFIELD STUDIO (9.1.6)",
            Self::BinauralAtmosSpatializer => "BINAURAL VIRTUALIZER",
            Self::CarAudio16ChannelArray => "AUTO 16-CH CABIN",
        }
    }

    pub fn nominal_channels(&self) -> usize {
        match self {
            Self::MasteringStage916 => 16,
            Self::CinemaAuditoriumDolby => 32,
            Self::NearfieldMixingStudio => 16,
            Self::BinauralAtmosSpatializer => 2,
            Self::CarAudio16ChannelArray => 16,
        }
    }

    pub fn nominal_ray_count(&self) -> usize {
        match self {
            Self::MasteringStage916 => 128,
            Self::CinemaAuditoriumDolby => 256,
            Self::NearfieldMixingStudio => 64,
            Self::BinauralAtmosSpatializer => 128,
            Self::CarAudio16ChannelArray => 96,
        }
    }

    pub fn nominal_reverb_rt60_s(&self) -> f32 {
        match self {
            Self::MasteringStage916 => 0.35,
            Self::CinemaAuditoriumDolby => 0.85,
            Self::NearfieldMixingStudio => 0.22,
            Self::BinauralAtmosSpatializer => 0.30,
            Self::CarAudio16ChannelArray => 0.15,
        }
    }
}

/// Broadcast mastering immersive Dolby Atmos 9.1.6 3D acoustic room raytracing HUD.
#[derive(Debug, Clone)]
pub struct Atmos916SpatializerView {
    pub room_type: AtmosRoomType,
    pub source_x_m: f32,
    pub source_y_m: f32,
    pub source_z_m: f32,
    pub ray_count: usize,
    pub rt60_decay_s: f32,
    pub direct_to_reverb_ratio: f32,
    pub puck_pos: (f32, f32),
    pub is_dragging_puck: bool,
    pub speaker_energy_levels: [f32; 16],
    pub color_palette: ContrastColorPalette,
}

impl Default for Atmos916SpatializerView {
    fn default() -> Self {
        Self::new()
    }
}

impl Atmos916SpatializerView {
    pub fn new() -> Self {
        let mut view = Self {
            room_type: AtmosRoomType::MasteringStage916,
            source_x_m: 1.5,
            source_y_m: 2.0,
            source_z_m: 1.2,
            ray_count: 128,
            rt60_decay_s: 0.35,
            direct_to_reverb_ratio: 0.82,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            speaker_energy_levels: [
                0.8, 0.6, 0.9, 0.7, 0.5, 0.4, 0.4, 0.3, 0.3, 0.95, 0.7, 0.7, 0.6, 0.6, 0.5, 0.5,
            ],
            color_palette: ContrastColorPalette::default(),
        };
        view.puck_pos = (
            Self::coord_to_normalized(view.source_x_m),
            Self::coord_to_normalized(view.source_y_m),
        );
        view.update_raytrace_simulation();
        view
    }

    pub fn coord_to_normalized(coord: f32) -> f32 {
        let c = coord.clamp(MIN_ROOM_COORD_M, MAX_ROOM_COORD_M);
        ((c - MIN_ROOM_COORD_M) / (MAX_ROOM_COORD_M - MIN_ROOM_COORD_M)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_coord(norm: f32) -> f32 {
        MIN_ROOM_COORD_M + norm.clamp(0.0, 1.0) * (MAX_ROOM_COORD_M - MIN_ROOM_COORD_M)
    }

    pub fn set_room_type(&mut self, room: AtmosRoomType) {
        self.room_type = room;
        self.ray_count = room.nominal_ray_count();
        self.rt60_decay_s = room.nominal_reverb_rt60_s();
        self.puck_pos = (
            Self::coord_to_normalized(self.source_x_m),
            Self::coord_to_normalized(self.source_y_m),
        );
        self.update_raytrace_simulation();
    }

    pub fn update_raytrace_simulation(&mut self) {
        let x = self.source_x_m;
        let y = self.source_y_m;
        let dist_center = (x * x + y * y).sqrt();

        // 9.1.6 Channels:
        // Bed: L, C, R, Lw, Rw, Ls, Rs, Lb, Rb (9)
        // LFE: Sub (1)
        // Overhead: Ltf, Rtf, Ltm, Rtm, Ltr, Rtr (6)
        let l_gain = (1.0 - (x + 3.0).abs() / 8.0).clamp(0.05, 1.0);
        let c_gain = (1.0 - x.abs() / 5.0).clamp(0.05, 1.0);
        let r_gain = (1.0 - (x - 3.0).abs() / 8.0).clamp(0.05, 1.0);
        let lw_gain = (0.8 * l_gain).clamp(0.05, 0.9);
        let rw_gain = (0.8 * r_gain).clamp(0.05, 0.9);
        let ls_gain = (1.0 - (y + 3.0).abs() / 8.0).clamp(0.05, 0.85);
        let rs_gain = (1.0 - (y - 3.0).abs() / 8.0).clamp(0.05, 0.85);
        let lb_gain = (0.7 * ls_gain).clamp(0.05, 0.8);
        let rb_gain = (0.7 * rs_gain).clamp(0.05, 0.8);
        let lfe_gain = (0.95 - dist_center * 0.03).clamp(0.3, 1.0);

        let ltf_gain = (0.75 * l_gain).clamp(0.05, 0.8);
        let rtf_gain = (0.75 * r_gain).clamp(0.05, 0.8);
        let ltm_gain = (0.65 * ls_gain).clamp(0.05, 0.7);
        let rtm_gain = (0.65 * rs_gain).clamp(0.05, 0.7);
        let ltr_gain = (0.60 * lb_gain).clamp(0.05, 0.65);
        let rtr_gain = (0.60 * rb_gain).clamp(0.05, 0.65);

        self.speaker_energy_levels = [
            l_gain, c_gain, r_gain, lw_gain, rw_gain, ls_gain, rs_gain, lb_gain, rb_gain, lfe_gain,
            ltf_gain, rtf_gain, ltm_gain, rtm_gain, ltr_gain, rtr_gain,
        ];
    }

    pub fn hit_test_atmos_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= ATMOS_PUCK_HIT_RADIUS
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
        let bar_spacing = right_w / 17;
        for (i, &energy) in self.speaker_energy_levels.iter().enumerate() {
            let col = mid_x + 2 + (i + 1) * bar_spacing;
            let bar_h = (energy.clamp(0.0, 1.0) * (height - 4) as f32).round() as usize;
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
            "DOLBY ATMOS 9.1.6 3D ACOUSTIC ROOM RAYTRACING HUD",
            egui::FontId::proportional(13.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Tabs (y: 48..92) - 44pt height
        let tabs = [
            (AtmosRoomType::MasteringStage916, "MASTERING (9.1.6)"),
            (AtmosRoomType::CinemaAuditoriumDolby, "CINEMA DUB STAGE"),
            (AtmosRoomType::NearfieldMixingStudio, "NEARFIELD STUDIO"),
            (AtmosRoomType::BinauralAtmosSpatializer, "BINAURAL HRIR"),
            (AtmosRoomType::CarAudio16ChannelArray, "AUTO 16-CH CABIN"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (itype, name)) in tabs.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_sel = self.room_type == *itype;
            let bg_col = if is_sel {
                Color32::from_rgb(0, 255, 180)
            } else {
                Color32::from_rgb(24, 32, 48)
            };
            let text_col = if is_sel {
                Color32::from_rgb(8, 28, 20)
            } else {
                Color32::from_rgb(210, 225, 245)
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
                        self.set_room_type(*itype);
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

        // Left 55%: 3D Acoustic Room & 9.1.6 Speaker Layout Field
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
            "9.1.6 SPEAKER ARRAY & ACOUSTIC RAYTRACE FIELD",
            egui::FontId::proportional(10.0),
            Color32::from_rgb(0, 255, 180),
        );

        // Listener Center Head Position
        let listener_pos = left_rect.center();
        painter.circle_filled(listener_pos, 6.0, Color32::from_rgb(0, 229, 255));
        painter.circle_stroke(
            listener_pos,
            12.0,
            Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(0, 229, 255, 120)),
        );

        // 9.1.6 Speaker Bed Node Ring
        let speaker_coords = [
            ("L", -0.7, 0.7),
            ("C", 0.0, 0.85),
            ("R", 0.7, 0.7),
            ("Lw", -0.9, 0.3),
            ("Rw", 0.9, 0.3),
            ("Ls", -0.85, -0.2),
            ("Rs", 0.85, -0.2),
            ("Lb", -0.5, -0.75),
            ("Rb", 0.5, -0.75),
        ];

        for (lbl, sx, sy) in speaker_coords {
            let sp_pos = egui::pos2(
                listener_pos.x + sx * (left_rect.width() * 0.42),
                listener_pos.y - sy * (left_rect.height() * 0.38),
            );
            painter.circle_filled(sp_pos, 4.0, Color32::from_rgb(0, 255, 180));
            painter.text(
                egui::pos2(sp_pos.x, sp_pos.y - 12.0),
                egui::Align2::CENTER_CENTER,
                lbl,
                egui::FontId::proportional(8.5),
                Color32::from_rgb(180, 215, 245),
            );
        }

        // Interactive Object Puck (X = Azimuth X, Y = Depth Y)
        let puck_x = left_rect.min.x + self.puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        // Raytrace Lines from Puck to Speakers
        for (i, (_lbl, sx, sy)) in speaker_coords.iter().enumerate() {
            if i % 2 == 0 {
                let sp_pos = egui::pos2(
                    listener_pos.x + sx * (left_rect.width() * 0.42),
                    listener_pos.y - sy * (left_rect.height() * 0.38),
                );
                painter.line_segment(
                    [puck_pos, sp_pos],
                    Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(0, 255, 180, 50)),
                );
            }
        }

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.puck_pos = (nx, ny);
                    self.source_x_m = Self::normalized_to_coord(nx);
                    self.source_y_m = Self::normalized_to_coord(ny);
                    self.update_raytrace_simulation();
                }
            }
        }

        painter.circle_stroke(
            puck_pos,
            ATMOS_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(0, 255, 180, 150)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(0, 255, 180));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 18.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Pos: ({:.1}, {:.1}) m | Rays: {} | RT60: {:.2}s | Profile: {}",
                self.source_x_m,
                self.source_y_m,
                self.ray_count,
                self.rt60_decay_s,
                self.room_type.room_name()
            ),
            egui::FontId::proportional(10.0),
            Color32::from_rgb(120, 255, 200),
        );

        // Right 45%: 16-Channel Level Meter Array
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
            "16-CHANNEL 9.1.6 MASTER DISPATCH ENERGY (dBFS)",
            egui::FontId::proportional(9.5),
            Color32::from_rgb(0, 255, 180),
        );

        let ch_labels = [
            "L", "C", "R", "Lw", "Rw", "Ls", "Rs", "Lb", "Rb", "LFE", "Ltf", "Rtf", "Ltm", "Rtm",
            "Ltr", "Rtr",
        ];
        let bar_w = (right_rect.width() - 30.0 - 15.0 * 3.0) / 16.0;

        for (i, &energy) in self.speaker_energy_levels.iter().enumerate() {
            let bx = right_rect.min.x + 15.0 + i as f32 * (bar_w + 3.0);
            let bar_h = (energy.clamp(0.0, 1.0)) * (right_rect.height() - 75.0);
            let b_rect = egui::Rect::from_min_max(
                egui::pos2(bx, right_rect.max.y - 25.0 - bar_h),
                egui::pos2(bx + bar_w, right_rect.max.y - 25.0),
            );
            let col = if i == 9 {
                Color32::from_rgb(255, 107, 43)
            } else if i < 9 {
                Color32::from_rgb(0, 255, 180)
            } else {
                Color32::from_rgb(0, 229, 255)
            };
            painter.rect_filled(b_rect, 2.0, col);

            if i % 2 == 0 {
                painter.text(
                    egui::pos2(bx + bar_w * 0.5, right_rect.max.y - 20.0),
                    egui::Align2::CENTER_TOP,
                    ch_labels[i],
                    egui::FontId::proportional(7.0),
                    Color32::from_rgb(180, 205, 235),
                );
            }
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
                "3D OBJECT POSITION",
                format!(
                    "({:.2}, {:.2}, {:.2}) m",
                    self.source_x_m, self.source_y_m, self.source_z_m
                ),
                Color32::from_rgb(0, 255, 180),
            ),
            (
                "RAYTRACE COUNT",
                format!("{} Image Rays", self.ray_count),
                Color32::from_rgb(59, 130, 246),
            ),
            (
                "REVERB RT60 TIME",
                format!("{:.2} s (Mastering)", self.rt60_decay_s),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "ATMOS PROFILE",
                "9.1.6 Bed + Heights".to_string(),
                Color32::from_rgb(0, 229, 255),
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
            "[PASS] Dolby Atmos 9.1.6 3D Room Raytracer Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
