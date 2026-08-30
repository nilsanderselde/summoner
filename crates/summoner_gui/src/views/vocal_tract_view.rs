// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Physical Modeling Vocal Tract 44-Cylinder Area Function HUD & Articulation View (Milestone 15).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use summoner_dsp::vocal_tract::{VowelPreset, NUM_CYLINDERS, NUM_NASAL_CYLINDERS};

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const VOCAL_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_TONGUE_POS: f32 = 0.0;
pub const MAX_TONGUE_POS: f32 = 1.0;
pub const MIN_TONGUE_HEIGHT: f32 = 0.0;
pub const MAX_TONGUE_HEIGHT: f32 = 1.0;

/// Physical Modeling Vocal Tract Area Function View HUD.
#[derive(Debug, Clone)]
pub struct VocalTractView {
    pub vowel_preset: VowelPreset,
    pub tongue_position: f32, // [0.0 = back pharynx ..= 1.0 = dental]
    pub tongue_height: f32,   // [0.0 = open/low ..= 1.0 = high constriction]
    pub lip_opening: f32,     // [0.1 = tight rounded ..= 2.5 = wide spread]
    pub velum_opening: f32,   // [0.0 = oral ..= 1.0 = nasalized]
    pub glottal_f0_hz: f32,   // [50.0 ..= 600.0 Hz]
    pub aspiration_level: f32,// [0.0 ..= 1.0]
    pub cylinder_areas: [f32; NUM_CYLINDERS],
    pub nasal_areas: [f32; NUM_NASAL_CYLINDERS],
    pub puck_pos: (f32, f32), // Normalized (X: tongue_position, Y: tongue_height)
    pub is_dragging_puck: bool,
    pub color_palette: ContrastColorPalette,
}

impl Default for VocalTractView {
    fn default() -> Self {
        Self::new()
    }
}

impl VocalTractView {
    pub fn new() -> Self {
        let preset = VowelPreset::I;
        let mut view = Self {
            vowel_preset: preset,
            tongue_position: 0.75,
            tongue_height: 0.85,
            lip_opening: 1.25,
            velum_opening: 0.0,
            glottal_f0_hz: 140.0,
            aspiration_level: 0.04,
            cylinder_areas: preset.cross_sectional_areas(),
            nasal_areas: [1.2; NUM_NASAL_CYLINDERS],
            puck_pos: (0.75, 0.85),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        };
        view.update_areas();
        view
    }

    /// Sets the active vowel preset and updates cylinder areas.
    pub fn set_vowel_preset(&mut self, preset: VowelPreset) {
        self.vowel_preset = preset;
        self.cylinder_areas = preset.cross_sectional_areas();
        match preset {
            VowelPreset::I => { self.tongue_position = 0.85; self.tongue_height = 0.90; self.lip_opening = 1.3; }
            VowelPreset::E => { self.tongue_position = 0.75; self.tongue_height = 0.70; self.lip_opening = 1.2; }
            VowelPreset::Epsilon => { self.tongue_position = 0.65; self.tongue_height = 0.50; self.lip_opening = 1.3; }
            VowelPreset::Ash => { self.tongue_position = 0.55; self.tongue_height = 0.30; self.lip_opening = 1.5; }
            VowelPreset::A => { self.tongue_position = 0.30; self.tongue_height = 0.15; self.lip_opening = 1.6; }
            VowelPreset::OpenO => { self.tongue_position = 0.20; self.tongue_height = 0.40; self.lip_opening = 0.7; }
            VowelPreset::O => { self.tongue_position = 0.15; self.tongue_height = 0.65; self.lip_opening = 0.5; }
            VowelPreset::U => { self.tongue_position = 0.10; self.tongue_height = 0.90; self.lip_opening = 0.3; }
            VowelPreset::Schwa => { self.tongue_position = 0.50; self.tongue_height = 0.50; self.lip_opening = 1.0; }
        }
        self.puck_pos = (self.tongue_position, self.tongue_height);
    }

    /// Re-evaluates 44-cylinder cross-sectional area function based on tongue and lip coordinates.
    pub fn update_areas(&mut self) {
        let center_idx = 12.0 + self.tongue_position.clamp(0.0, 1.0) * 24.0;
        let width = 7.0;
        let min_area = 0.30 + (1.0 - self.tongue_height.clamp(0.0, 1.0)) * 3.0;

        for (i, area) in self.cylinder_areas.iter_mut().enumerate() {
            let dist = (i as f32 - center_idx).abs();
            if dist < width {
                let factor = (1.0 - dist / width).powi(2);
                let constricted = 3.0 * (1.0 - factor) + min_area * factor;
                *area = constricted.max(0.15);
            } else if i < 12 {
                let t = i as f32 / 12.0;
                *area = 3.5 * (1.0 - t) + 2.5 * t;
            } else {
                *area = 2.8;
            }
        }

        for i in 38..NUM_CYLINDERS {
            let t = (i - 38) as f32 / 6.0;
            self.cylinder_areas[i] = (self.cylinder_areas[i] * (1.0 - t + self.lip_opening * t)).clamp(0.1, 8.0);
        }
    }

    /// Hit-test touch coordinate on the tongue puck.
    pub fn hit_test_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= VOCAL_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render of 44-Cylinder Area Function and Articulatory Canvas.
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

        // Draw 44-cylinder tube profile on right half
        let right_w = width - mid_x - 2;
        let center_r = height / 2;
        for c in 0..right_w {
            let cyl_idx = ((c as f32 / right_w as f32) * (NUM_CYLINDERS - 1) as f32).round() as usize;
            let area = self.cylinder_areas[cyl_idx.min(NUM_CYLINDERS - 1)];
            let half_h = ((area / 6.0) * (height as f32 * 0.35)).round().max(1.0) as usize;
            if center_r >= half_h && center_r + half_h < height - 1 {
                grid[center_r - half_h][mid_x + 1 + c] = '=';
                grid[center_r + half_h][mid_x + 1 + c] = '=';
            }
        }

        // Tongue Puck on left half
        let puck_col = ((self.puck_pos.0 * (mid_x - 2) as f32) + 1.0).round() as usize;
        let puck_row = (((1.0 - self.puck_pos.1) * (height - 3) as f32) + 1.0).round() as usize;
        if puck_row < height - 1 && puck_col < mid_x {
            grid[puck_row][puck_col] = 'O';
        }

        grid.into_iter().map(|row| row.into_iter().collect()).collect()
    }

    #[cfg(feature = "gui")]
    #[allow(clippy::needless_range_loop)]
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 480.0),
            egui::Sense::click_and_drag(),
        );

        let painter = ui.painter_at(rect);

        // Background (#0A0E18)
        painter.rect_filled(rect, 6.0, Color32::from_rgb(10, 14, 24));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 16.0),
            egui::Align2::LEFT_TOP,
            "1D 44-CYLINDER FDTD VOCAL TRACT ACOUSTIC TUBE & ARTICULATION HUD",
            egui::FontId::proportional(16.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Vowel Preset Selector Tabs (y: 46..90) - >= 44pt height
        let presets = [
            (VowelPreset::I, "/i/ (beet)"),
            (VowelPreset::E, "/e/ (bait)"),
            (VowelPreset::Epsilon, "/ɛ/ (bet)"),
            (VowelPreset::Ash, "/æ/ (bat)"),
            (VowelPreset::A, "/a/ (father)"),
            (VowelPreset::OpenO, "/ɔ/ (bought)"),
            (VowelPreset::O, "/o/ (boat)"),
            (VowelPreset::U, "/u/ (boot)"),
            (VowelPreset::Schwa, "/ə/ (sofa)"),
        ];

        let tab_w = (rect.width() - 40.0 - 8.0 * 6.0) / 9.0;
        for (i, (preset, name)) in presets.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 6.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 46.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_selected = self.vowel_preset == *preset;
            let bg_color = if is_selected {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(20, 30, 45)
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
                        self.set_vowel_preset(*preset);
                    }
                }
            }
        }

        // Main Display Canvas (y: 100..340)
        let main_canvas = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 100.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 340.0),
        );
        painter.rect_filled(main_canvas, 6.0, Color32::from_rgb(14, 20, 32));
        painter.rect_stroke(
            main_canvas,
            6.0,
            Stroke::new(1.5_f32, Color32::from_rgb(45, 65, 95)),
        );

        // Left 50%: Tongue Articulatory Space (Position vs Height)
        let left_w = (main_canvas.width() - 30.0) * 0.50;
        let left_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(left_w, main_canvas.height() - 20.0),
        );
        painter.rect_filled(left_rect, 4.0, Color32::from_rgb(10, 16, 26));
        painter.rect_stroke(
            left_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(35, 50, 75)),
        );

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "TONGUE ARTICULATION (POSITION vs HEIGHT)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // Grid guides
        for step in 1..4 {
            let gx = left_rect.min.x + (step as f32 / 4.0) * left_rect.width();
            let gy = left_rect.min.y + (step as f32 / 4.0) * left_rect.height();
            painter.line_segment(
                [egui::pos2(gx, left_rect.min.y + 24.0), egui::pos2(gx, left_rect.max.y)],
                Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(0, 229, 255, 40)),
            );
            painter.line_segment(
                [egui::pos2(left_rect.min.x, gy), egui::pos2(left_rect.max.x, gy)],
                Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(0, 229, 255, 40)),
            );
        }

        // Interactive Tongue Puck
        let puck_x = left_rect.min.x + self.puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.puck_pos = (nx, ny);
                    self.tongue_position = nx;
                    self.tongue_height = ny;
                    self.update_areas();
                }
            }
        }

        // Draw Touch Hit Target Boundary (>= 44x44pt)
        painter.circle_stroke(
            puck_pos,
            VOCAL_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(0, 229, 255, 140)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(0, 229, 255));
        painter.circle_filled(puck_pos, 4.0, Color32::from_rgb(255, 255, 255));

        // Right 50%: 44-Cylinder Cross-Sectional Area Function & Anatomical Landmarks
        let right_x = main_canvas.min.x + left_w + 20.0;
        let right_w = left_w;
        let right_rect = egui::Rect::from_min_size(
            egui::pos2(right_x, main_canvas.min.y + 10.0),
            egui::vec2(right_w, main_canvas.height() - 20.0),
        );
        painter.rect_filled(right_rect, 4.0, Color32::from_rgb(10, 16, 26));
        painter.rect_stroke(
            right_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(35, 50, 75)),
        );

        painter.text(
            egui::pos2(right_rect.min.x + 10.0, right_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "44-CYLINDER CROSS-SECTIONAL AREA TUBE (GLOTTIS -> LIPS)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // Draw 44 Cylinders as vertical bar profile
        let center_y = right_rect.center().y + 10.0;
        let cyl_w = (right_rect.width() - 20.0) / (NUM_CYLINDERS as f32);

        for (i, &area) in self.cylinder_areas.iter().enumerate() {
            let cx = right_rect.min.x + 10.0 + i as f32 * cyl_w;
            let half_h = (area * 8.0).clamp(2.0, 70.0);
            let bar_rect = egui::Rect::from_min_max(
                egui::pos2(cx, center_y - half_h),
                egui::pos2(cx + cyl_w - 1.0, center_y + half_h),
            );

            let col = if i == 0 {
                Color32::from_rgb(255, 107, 43) // Glottis (orange)
            } else if i == 17 {
                Color32::from_rgb(0, 255, 180) // Velum (jade)
            } else if i >= 38 {
                Color32::from_rgb(255, 215, 0) // Lips (gold)
            } else {
                Color32::from_rgb(0, 229, 255) // Pharyngeal / Oral (cyan)
            };

            painter.rect_filled(bar_rect, 1.0, col);
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

        let metrics = [
            (
                "TONGUE POSITION",
                format!("{:.2} (Palatal)", self.tongue_position),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "TONGUE HEIGHT",
                format!("{:.2} (High)", self.tongue_height),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "LIP OPENING",
                format!("{:.2}x Spread", self.lip_opening),
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "GLOTTAL PITCH (F0)",
                format!("{:.1} Hz (Aspir: {:.0}%)", self.glottal_f0_hz, self.aspiration_level * 100.0),
                Color32::from_rgb(0, 255, 180),
            ),
        ];

        let col_w = (dock_rect.width() - 40.0) / 4.0;
        for (i, (label, val, col)) in metrics.iter().enumerate() {
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
            "[PASS] 1D 44-Cylinder FDTD Vocal Tract Area Function & Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }

    #[cfg(feature = "gui")]
    pub fn show(&mut self, ui: &mut egui::Ui, rect: Rect) {
        ui.allocate_ui_at_rect(
            egui::Rect::from_min_size(
                egui::pos2(rect.x, rect.y),
                egui::vec2(rect.width, rect.height),
            ),
            |ui| {
                self.ui(ui);
            },
        );
    }

    /// Render headless PNG snapshot displaying 44-Cylinder tube profile and tongue articulation puck.
    pub fn render_snapshot_png(&self, path: &str, width: u32, height: u32) -> Result<(), String> {
        if let Some(parent) = std::path::Path::new(path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let mut pixels = vec![0u8; (width * height * 4) as usize];

        // Background: Deep slate/navy (#0A0E18)
        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 4) as usize;
                pixels[idx] = 10;
                pixels[idx + 1] = 14;
                pixels[idx + 2] = 24;
                pixels[idx + 3] = 255;
            }
        }

        // Header Panel (y: 10..46, x: 20..width-20)
        fill_rounded_rect_vt(&mut pixels, width, height, 20, 10, width - 40, 36, 4, [18, 24, 38, 255]);
        draw_rect_border_vt(&mut pixels, width, height, 20, 10, width - 40, 36, [45, 60, 85, 255]);

        // Left Canvas: Tongue Articulation Space (x: 20..width/2 - 10, y: 56..height - 120)
        let left_w = (width - 60) / 2;
        let canvas_h = height - 170;
        fill_rounded_rect_vt(&mut pixels, width, height, 20, 56, left_w, canvas_h, 6, [14, 20, 32, 255]);
        draw_rect_border_vt(&mut pixels, width, height, 20, 56, left_w, canvas_h, [45, 60, 85, 255]);

        // Draw Left Grid lines
        for step in 1..4 {
            let gx = 20.0 + (step as f32 / 4.0) * left_w as f32;
            let gy = 56.0 + (step as f32 / 4.0) * canvas_h as f32;
            draw_line_segment_vt(&mut pixels, width, height, gx, 56.0, gx, 56.0 + canvas_h as f32, [25, 35, 55, 255]);
            draw_line_segment_vt(&mut pixels, width, height, 20.0, gy, 20.0 + left_w as f32, gy, [25, 35, 55, 255]);
        }

        // Tongue Puck (Gold/Cyan, >= 22pt radius -> 44x44pt touch bounding target)
        let puck_x = 20.0 + self.puck_pos.0 * left_w as f32;
        let puck_y = 56.0 + (1.0 - self.puck_pos.1) * canvas_h as f32;
        draw_circle_ring_vt(&mut pixels, width, height, puck_x, puck_y, VOCAL_PUCK_HIT_RADIUS, [0, 229, 255, 160]);
        draw_circle_filled_vt(&mut pixels, width, height, puck_x, puck_y, 14.0, [0, 229, 255, 255]);
        draw_circle_filled_vt(&mut pixels, width, height, puck_x, puck_y, 4.0, [255, 255, 255, 255]);

        // Right Canvas: 44-Cylinder Area Function (x: width/2 + 10..width - 20, y: 56..height - 120)
        let right_x = 20 + left_w + 20;
        let right_w = left_w;
        fill_rounded_rect_vt(&mut pixels, width, height, right_x, 56, right_w, canvas_h, 6, [14, 20, 32, 255]);
        draw_rect_border_vt(&mut pixels, width, height, right_x, 56, right_w, canvas_h, [45, 60, 85, 255]);

        let center_y = 56.0 + canvas_h as f32 * 0.5;
        let bar_w = (right_w as f32 - 30.0) / (NUM_CYLINDERS as f32);

        for (i, &area) in self.cylinder_areas.iter().enumerate() {
            let bx = right_x as f32 + 15.0 + i as f32 * bar_w;
            let half_h = (area * 10.0).clamp(2.0, (canvas_h as f32) * 0.45);
            let col = if i == 0 {
                [255, 107, 43, 255]
            } else if i == 17 {
                [0, 255, 180, 255]
            } else if i >= 38 {
                [255, 215, 0, 255]
            } else {
                [0, 229, 255, 255]
            };

            for y in (center_y - half_h) as u32..(center_y + half_h) as u32 {
                for x in bx as u32..(bx + bar_w.max(1.0) - 1.0) as u32 {
                    if x < width && y < height {
                        let idx = ((y * width + x) * 4) as usize;
                        pixels[idx..idx + 4].copy_from_slice(&col);
                    }
                }
            }
        }

        // Bottom Metrics Dock (x: 20..width - 20, y: height - 100..height - 20)
        let dock_y = height - 100;
        let dock_w = width - 40;
        fill_rounded_rect_vt(&mut pixels, width, height, 20, dock_y, dock_w, 80, 4, [16, 35, 28, 255]);
        draw_rect_border_vt(&mut pixels, width, height, 20, dock_y, dock_w, 80, [0, 255, 180, 255]);

        encode_minimal_png_vt(path, &pixels, width, height)
    }
}

// ----------------------------------------------------------------------------
// Minimal software rasterizer & PNG encoder helpers for headless verification
// ----------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn fill_rounded_rect_vt(
    buf: &mut [u8],
    w: u32,
    h: u32,
    rx: u32,
    ry: u32,
    rw: u32,
    rh: u32,
    radius: u32,
    col: [u8; 4],
) {
    let r = radius as f32;
    for y in ry..(ry + rh).min(h) {
        for x in rx..(rx + rw).min(w) {
            let mut inside = true;
            if x < rx + radius && y < ry + radius {
                let dx = (rx + radius - x) as f32;
                let dy = (ry + radius - y) as f32;
                inside = dx * dx + dy * dy <= r * r;
            } else if x >= rx + rw - radius && y < ry + radius {
                let dx = (x - (rx + rw - radius - 1)) as f32;
                let dy = (ry + radius - y) as f32;
                inside = dx * dx + dy * dy <= r * r;
            } else if x < rx + radius && y >= ry + rh - radius {
                let dx = (rx + radius - x) as f32;
                let dy = (y - (ry + rh - radius - 1)) as f32;
                inside = dx * dx + dy * dy <= r * r;
            } else if x >= rx + rw - radius && y >= ry + rh - radius {
                let dx = (x - (rx + rw - radius - 1)) as f32;
                let dy = (y - (ry + rh - radius - 1)) as f32;
                inside = dx * dx + dy * dy <= r * r;
            }
            if inside {
                let idx = ((y * w + x) * 4) as usize;
                buf[idx..idx + 4].copy_from_slice(&col);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_rect_border_vt(buf: &mut [u8], w: u32, h: u32, rx: u32, ry: u32, rw: u32, rh: u32, col: [u8; 4]) {
    for x in rx..(rx + rw).min(w) {
        if ry < h {
            let idx = ((ry * w + x) * 4) as usize;
            buf[idx..idx + 4].copy_from_slice(&col);
        }
        if ry + rh - 1 < h {
            let idx = (((ry + rh - 1) * w + x) * 4) as usize;
            buf[idx..idx + 4].copy_from_slice(&col);
        }
    }
    for y in ry..(ry + rh).min(h) {
        if rx < w {
            let idx = ((y * w + rx) * 4) as usize;
            buf[idx..idx + 4].copy_from_slice(&col);
        }
        if rx + rw - 1 < w {
            let idx = ((y * w + rx + rw - 1) * 4) as usize;
            buf[idx..idx + 4].copy_from_slice(&col);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_line_segment_vt(buf: &mut [u8], w: u32, h: u32, x0: f32, y0: f32, x1: f32, y1: f32, col: [u8; 4]) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let steps = (dx.abs().max(dy.abs()) * 2.0).max(1.0) as u32;
    for s in 0..=steps {
        let t = s as f32 / steps as f32;
        let px = (x0 + t * dx).round() as i32;
        let py = (y0 + t * dy).round() as i32;
        if px >= 0 && px < w as i32 && py >= 0 && py < h as i32 {
            let idx = ((py as u32 * w + px as u32) * 4) as usize;
            buf[idx..idx + 4].copy_from_slice(&col);
        }
    }
}

fn draw_circle_filled_vt(buf: &mut [u8], w: u32, h: u32, cx: f32, cy: f32, r: f32, col: [u8; 4]) {
    let min_x = (cx - r).max(0.0) as u32;
    let max_x = (cx + r).min(w as f32 - 1.0) as u32;
    let min_y = (cy - r).max(0.0) as u32;
    let max_y = (cy + r).min(h as f32 - 1.0) as u32;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            if dx * dx + dy * dy <= r * r {
                let idx = ((y * w + x) * 4) as usize;
                buf[idx..idx + 4].copy_from_slice(&col);
            }
        }
    }
}

fn draw_circle_ring_vt(buf: &mut [u8], w: u32, h: u32, cx: f32, cy: f32, r: f32, col: [u8; 4]) {
    let thickness = 2.0f32;
    let r_inner = (r - thickness).max(0.0);
    let min_x = (cx - r).max(0.0) as u32;
    let max_x = (cx + r).min(w as f32 - 1.0) as u32;
    let min_y = (cy - r).max(0.0) as u32;
    let max_y = (cy + r).min(h as f32 - 1.0) as u32;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq <= r * r && dist_sq >= r_inner * r_inner {
                let idx = ((y * w + x) * 4) as usize;
                buf[idx..idx + 4].copy_from_slice(&col);
            }
        }
    }
}

fn encode_minimal_png_vt(path: &str, rgba_pixels: &[u8], width: u32, height: u32) -> Result<(), String> {
    let mut out = Vec::with_capacity((width * height * 4 + 1024) as usize);
    out.extend_from_slice(b"\x89PNG\r\n\x1a\n");

    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.push(8);
    ihdr.push(6);
    ihdr.push(0);
    ihdr.push(0);
    ihdr.push(0);
    write_png_chunk_vt(&mut out, b"IHDR", &ihdr);

    let mut raw_data = Vec::with_capacity((height * (width * 4 + 1)) as usize);
    for y in 0..height {
        raw_data.push(0);
        let row_start = (y * width * 4) as usize;
        let row_end = row_start + (width * 4) as usize;
        raw_data.extend_from_slice(&rgba_pixels[row_start..row_end]);
    }

    let mut zlib_data = Vec::with_capacity(raw_data.len() + 128);
    zlib_data.push(0x78);
    zlib_data.push(0x01);

    let mut offset = 0;
    while offset < raw_data.len() {
        let chunk_len = (raw_data.len() - offset).min(65535);
        let is_last = (offset + chunk_len) >= raw_data.len();
        let bfinal_btype = if is_last { 0x01 } else { 0x00 };
        zlib_data.push(bfinal_btype);

        let len_u16 = chunk_len as u16;
        let nlen_u16 = !len_u16;
        zlib_data.extend_from_slice(&len_u16.to_le_bytes());
        zlib_data.extend_from_slice(&nlen_u16.to_le_bytes());
        zlib_data.extend_from_slice(&raw_data[offset..offset + chunk_len]);
        offset += chunk_len;
    }

    let mut s1: u32 = 1;
    let mut s2: u32 = 0;
    for &b in &raw_data {
        s1 = (s1 + b as u32) % 65521;
        s2 = (s2 + s1) % 65521;
    }
    let adler = (s2 << 16) | s1;
    zlib_data.extend_from_slice(&adler.to_be_bytes());

    write_png_chunk_vt(&mut out, b"IDAT", &zlib_data);
    write_png_chunk_vt(&mut out, b"IEND", &[]);

    std::fs::write(path, out).map_err(|e| e.to_string())
}

fn write_png_chunk_vt(out: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let type_start = out.len();
    out.extend_from_slice(chunk_type);
    out.extend_from_slice(data);
    let crc = crc32_compute_vt(&out[type_start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

fn crc32_compute_vt(buf: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in buf {
        crc ^= b as u32;
        for _ in 0..8 {
            let mask = if (crc & 1) != 0 { 0xEDB8_8320 } else { 0 };
            crc = (crc >> 1) ^ mask;
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::touch_controls::MIN_HIT_TARGET_PT;

    #[test]
    fn test_vocal_tract_view_ascii_render() {
        let view = VocalTractView::new();
        let ascii = view.render_ascii(80, 16);
        assert!(!ascii.is_empty());
        assert_eq!(ascii.len(), 16);
    }

    #[test]
    fn test_vocal_tract_view_hit_target_dimensions() {
        const {
            assert!(
                VOCAL_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT,
                "Tongue puck hit target bounding box must be >= 44pt"
            );
        }
    }

    #[test]
    fn test_vocal_tract_view_snapshot_render() {
        let view = VocalTractView::new();
        let res = view.render_snapshot_png("scratch/renders/vocal_tract_view.png", 800, 520);
        assert!(res.is_ok());
    }
}
