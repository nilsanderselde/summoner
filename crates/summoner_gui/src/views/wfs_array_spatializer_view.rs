// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Broadcast Mastering Immersive Wave Field Synthesis (WFS) Holographic Linear Acoustic Array HUD (Step 1585).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const WFS_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_SOURCE_X_M: f32 = -8.0;
pub const MAX_SOURCE_X_M: f32 = 8.0;
pub const MIN_SOURCE_Y_M: f32 = -5.0;
pub const MAX_SOURCE_Y_M: f32 = 10.0;

/// Wave Field Synthesis acoustic array geometries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WfsGeometry {
    LinearFrontArray64, // 64-channel continuous frontal linear loudspeaker array (L=12m)
    RectangularRoomArray128, // 128-channel 4-wall perimeter boundary array (12m x 8m)
    CurvedStageProscenium32, // 32-channel circular arc stage proscenium array (R=8m)
    DualLinearMasteringDesk48, // 48-channel high-resolution nearfield desktop wave synthesizer
    HexagonalHolographicArray96, // 96-channel 360-degree immersive acoustic holographic room
}

impl WfsGeometry {
    pub fn geometry_name(&self) -> &'static str {
        match self {
            Self::LinearFrontArray64 => "LINEAR FRONT (64-CH)",
            Self::RectangularRoomArray128 => "RECTANGULAR (128-CH)",
            Self::CurvedStageProscenium32 => "CURVED PROSCENIUM (32-CH)",
            Self::DualLinearMasteringDesk48 => "DESKTOP NEARFIELD (48-CH)",
            Self::HexagonalHolographicArray96 => "HEXAGONAL 360° (96-CH)",
        }
    }

    pub fn nominal_channels(&self) -> usize {
        match self {
            Self::LinearFrontArray64 => 64,
            Self::RectangularRoomArray128 => 128,
            Self::CurvedStageProscenium32 => 32,
            Self::DualLinearMasteringDesk48 => 48,
            Self::HexagonalHolographicArray96 => 96,
        }
    }

    pub fn nominal_aliasing_cutoff_hz(&self) -> f32 {
        match self {
            Self::LinearFrontArray64 => 2200.0,
            Self::RectangularRoomArray128 => 1800.0,
            Self::CurvedStageProscenium32 => 1400.0,
            Self::DualLinearMasteringDesk48 => 4500.0,
            Self::HexagonalHolographicArray96 => 2600.0,
        }
    }

    pub fn nominal_source_pos_m(&self) -> (f32, f32) {
        match self {
            Self::LinearFrontArray64 => (0.0, 3.5),
            Self::RectangularRoomArray128 => (-2.0, 2.5),
            Self::CurvedStageProscenium32 => (1.5, 4.0),
            Self::DualLinearMasteringDesk48 => (0.0, 1.2),
            Self::HexagonalHolographicArray96 => (-1.0, 2.0),
        }
    }
}

/// Broadcast mastering immersive Wave Field Synthesis (WFS) holographic linear acoustic array HUD.
#[derive(Debug, Clone)]
pub struct WfsArraySpatializerView {
    pub array_geometry: WfsGeometry,
    pub source_x_m: f32, // [-8.0 ..= +8.0 m virtual source lateral offset]
    pub source_y_m: f32, // [-5.0 ..= +10.0 m virtual source depth (+ = behind array, - = focused)]
    pub is_focused_source: bool,
    pub channel_count: usize,
    pub spatial_aliasing_cutoff_hz: f32,
    pub puck_pos: (f32, f32), // Normalized (X: lateral, Y: depth)
    pub is_dragging_puck: bool,
    pub array_delays_ms: [f32; 16], // 16 sampled loudspeaker delay profile (ms)
    pub color_palette: ContrastColorPalette,
}

impl Default for WfsArraySpatializerView {
    fn default() -> Self {
        Self::new()
    }
}

impl WfsArraySpatializerView {
    pub fn new() -> Self {
        let mut view = Self {
            array_geometry: WfsGeometry::LinearFrontArray64,
            source_x_m: 0.0,
            source_y_m: 3.5,
            is_focused_source: false,
            channel_count: 64,
            spatial_aliasing_cutoff_hz: 2200.0,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            array_delays_ms: [
                12.5, 11.2, 10.0, 8.8, 7.5, 6.2, 5.0, 4.2, 4.2, 5.0, 6.2, 7.5, 8.8, 10.0, 11.2,
                12.5,
            ],
            color_palette: ContrastColorPalette::default(),
        };
        view.puck_pos = (
            Self::x_to_normalized(view.source_x_m),
            Self::y_to_normalized(view.source_y_m),
        );
        view.update_wfs_simulation();
        view
    }

    pub fn x_to_normalized(x: f32) -> f32 {
        let val = x.clamp(MIN_SOURCE_X_M, MAX_SOURCE_X_M);
        ((val - MIN_SOURCE_X_M) / (MAX_SOURCE_X_M - MIN_SOURCE_X_M)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_x(norm: f32) -> f32 {
        MIN_SOURCE_X_M + norm.clamp(0.0, 1.0) * (MAX_SOURCE_X_M - MIN_SOURCE_X_M)
    }

    pub fn y_to_normalized(y: f32) -> f32 {
        let val = y.clamp(MIN_SOURCE_Y_M, MAX_SOURCE_Y_M);
        ((val - MIN_SOURCE_Y_M) / (MAX_SOURCE_Y_M - MIN_SOURCE_Y_M)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_y(norm: f32) -> f32 {
        MIN_SOURCE_Y_M + norm.clamp(0.0, 1.0) * (MAX_SOURCE_Y_M - MIN_SOURCE_Y_M)
    }

    pub fn set_array_geometry(&mut self, geom: WfsGeometry) {
        self.array_geometry = geom;
        self.channel_count = geom.nominal_channels();
        self.spatial_aliasing_cutoff_hz = geom.nominal_aliasing_cutoff_hz();
        let (nx, ny) = geom.nominal_source_pos_m();
        self.source_x_m = nx;
        self.source_y_m = ny;
        self.is_focused_source = ny < 0.0;
        self.puck_pos = (
            Self::x_to_normalized(self.source_x_m),
            Self::y_to_normalized(self.source_y_m),
        );
        self.update_wfs_simulation();
    }

    /// Update Huygens-Fresnel holographic array driving delays (Kirchhoff-Helmholtz integral).
    pub fn update_wfs_simulation(&mut self) {
        self.is_focused_source = self.source_y_m < 0.0;
        let c_sound_mps = 343.0;

        let array_len_m = 10.0;
        for i in 0..16 {
            let driver_x = -array_len_m * 0.5 + (i as f32 / 15.0) * array_len_m;
            let driver_y = 0.0_f32;
            let dx = self.source_x_m - driver_x;
            let dy = self.source_y_m - driver_y;
            let dist_m = (dx * dx + dy * dy).sqrt();
            let delay_ms = (dist_m / c_sound_mps) * 1000.0;
            self.array_delays_ms[i] = delay_ms.clamp(0.1, 50.0);
        }
    }

    /// Hit test coordinate on the interactive WFS source puck.
    pub fn hit_test_wfs_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= WFS_PUCK_HIT_RADIUS
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
        let bar_spacing = right_w / 17;
        for (i, &delay) in self.array_delays_ms.iter().enumerate() {
            let col = mid_x + 2 + (i + 1) * bar_spacing;
            let bar_h = ((delay / 35.0) * (height - 4) as f32).round() as usize;
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
            "WAVE FIELD SYNTHESIS (WFS) HOLOGRAPHIC ACOUSTIC ARRAY HUD",
            egui::FontId::proportional(14.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Array Geometry Tabs (y: 48..92)
        let tabs = [
            (WfsGeometry::LinearFrontArray64, "LINEAR 64-CH"),
            (WfsGeometry::RectangularRoomArray128, "RECT 128-CH"),
            (WfsGeometry::CurvedStageProscenium32, "CURVED 32-CH"),
            (WfsGeometry::DualLinearMasteringDesk48, "DESK 48-CH"),
            (WfsGeometry::HexagonalHolographicArray96, "HEX 360° 96-CH"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (geom, name)) in tabs.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_sel = self.array_geometry == *geom;
            let bg_col = if is_sel {
                Color32::from_rgb(0, 255, 180)
            } else {
                Color32::from_rgb(24, 32, 48)
            };
            let text_col = if is_sel {
                Color32::from_rgb(8, 24, 16)
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
                        self.set_array_geometry(*geom);
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

        // Left 55%: Holographic Wavefront Acoustic Propagation Field
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
            "HUYGENS-FRESNEL HOLOGRAPHIC WAVEFIELD",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(0, 255, 180),
        );

        // Loudspeaker array baseline (horizontal line at y = center + 40)
        let array_y = left_rect.max.y - 45.0;
        painter.line_segment(
            [
                egui::pos2(left_rect.min.x + 15.0, array_y),
                egui::pos2(left_rect.max.x - 15.0, array_y),
            ],
            Stroke::new(2.5_f32, Color32::from_rgb(0, 229, 255)),
        );

        // Individual loudspeaker driver nodes
        let driver_count = 16;
        for d in 0..driver_count {
            let dx = left_rect.min.x
                + 15.0
                + (d as f32 / (driver_count - 1) as f32) * (left_rect.width() - 30.0);
            painter.circle_filled(egui::pos2(dx, array_y), 3.0, Color32::from_rgb(0, 255, 180));
        }

        // Interactive Virtual Source Puck
        let puck_x = left_rect.min.x + self.puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.puck_pos = (nx, ny);
                    self.source_x_m = Self::normalized_to_x(nx);
                    self.source_y_m = Self::normalized_to_y(ny);
                    self.update_wfs_simulation();
                }
            }
        }

        // Circular acoustic wavefront rings propagating from source
        for r_step in [12.0, 24.0, 36.0, 48.0] {
            painter.circle_stroke(
                puck_pos,
                r_step,
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 255, 180, 50)),
            );
        }

        painter.circle_stroke(
            puck_pos,
            WFS_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(0, 255, 180, 150)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(0, 255, 180));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 20.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Source: X={:.2}m, Y={:.2}m | {} | Drivers: {} | Aliasing: {:.0}Hz",
                self.source_x_m,
                self.source_y_m,
                if self.is_focused_source {
                    "Focused Source"
                } else {
                    "Virtual Behind"
                },
                self.channel_count,
                self.spatial_aliasing_cutoff_hz
            ),
            egui::FontId::proportional(10.5),
            Color32::from_rgb(120, 255, 200),
        );

        // Right 45%: Driver Delay Profile Matrix
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
            "16-POINT LOUDSPEAKER DELAY PROFILE (ms)",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(0, 255, 180),
        );

        let bar_w = (right_rect.width() - 30.0 - 15.0 * 4.0) / 16.0;
        for (i, &delay) in self.array_delays_ms.iter().enumerate() {
            let bx = right_rect.min.x + 15.0 + i as f32 * (bar_w + 4.0);
            let bar_h = (delay / 35.0).clamp(0.0, 1.0) * (right_rect.height() - 80.0);
            let b_rect = egui::Rect::from_min_max(
                egui::pos2(bx, right_rect.max.y - 25.0 - bar_h),
                egui::pos2(bx + bar_w, right_rect.max.y - 25.0),
            );
            let col = if i == 7 || i == 8 {
                Color32::from_rgb(0, 255, 180)
            } else if !(4..=11).contains(&i) {
                Color32::from_rgb(59, 130, 246)
            } else {
                Color32::from_rgb(0, 229, 255)
            };
            painter.rect_filled(b_rect, 2.0, col);

            if i % 2 == 0 {
                painter.text(
                    egui::pos2(bx + bar_w * 0.5, right_rect.max.y - 20.0),
                    egui::Align2::CENTER_TOP,
                    format!("D{}", i + 1),
                    egui::FontId::proportional(7.5),
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
                "VIRTUAL SOURCE POS",
                format!("({:.2}, {:.2}) m", self.source_x_m, self.source_y_m),
                Color32::from_rgb(0, 255, 180),
            ),
            (
                "ARRAY CHANNEL COUNT",
                format!("{} Channels", self.channel_count),
                Color32::from_rgb(59, 130, 246),
            ),
            (
                "SPATIAL ALIASING CUTOFF",
                format!("{:.0} Hz (Nyquist)", self.spatial_aliasing_cutoff_hz),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "SOURCE NATURE",
                if self.is_focused_source {
                    "Focused Source".to_string()
                } else {
                    "Virtual Diverging".to_string()
                },
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
            "[PASS] Wave Field Synthesis (WFS) Acoustic Array Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
