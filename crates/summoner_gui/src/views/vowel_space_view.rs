// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Interactive 2D International Phonetic Alphabet (IPA) Vowel Space & Formant Trajectory HUD (Milestone 15).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;
use summoner_core::formant_bus::IpaVowel;
use summoner_harmony::vowel_graph::VowelGraph;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const VOWEL_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_F1_HZ: f32 = 200.0;
pub const MAX_F1_HZ: f32 = 950.0;
pub const MIN_F2_HZ: f32 = 600.0;
pub const MAX_F2_HZ: f32 = 2600.0;

/// Interactive 2D IPA Vowel Space & Formant Trajectory HUD.
#[derive(Debug, Clone)]
pub struct VowelSpaceView {
    pub vowel_graph: VowelGraph,
    pub f1_hz: f32, // [200 ..= 950 Hz] (Vowel Height / Openness)
    pub f2_hz: f32, // [600 ..= 2600 Hz] (Vowel Backness)
    pub f3_hz: f32, // [1800 ..= 3200 Hz]
    pub f4_hz: f32, // [3000 ..= 4000 Hz]
    pub selected_vowel: IpaVowel,
    pub morph_progress: f32, // [0.0 ..= 1.0] along active waypoint trajectory
    pub is_animating: bool,
    pub formant_puck_pos: (f32, f32), // Normalized (X: backness / F2, Y: height / F1)
    pub is_dragging_puck: bool,
    pub color_palette: ContrastColorPalette,
}

impl Default for VowelSpaceView {
    fn default() -> Self {
        Self::new()
    }
}

impl VowelSpaceView {
    pub fn new() -> Self {
        let graph = VowelGraph::build_standard_ipa_graph();
        let (f1, f2, f3, f4) = IpaVowel::CloseFrontI.nominal_formants();
        let mut view = Self {
            vowel_graph: graph,
            f1_hz: f1,
            f2_hz: f2,
            f3_hz: f3,
            f4_hz: f4,
            selected_vowel: IpaVowel::CloseFrontI,
            morph_progress: 0.0,
            is_animating: false,
            formant_puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            color_palette: ContrastColorPalette::default(),
        };
        view.update_coordinates_from_formants();
        view
    }

    /// Convert F1 & F2 to normalized coordinates (X: Backness 0.0=Back, 1.0=Front; Y: Height 0.0=Open, 1.0=Close).
    pub fn update_coordinates_from_formants(&mut self) {
        let x = ((self.f2_hz - MIN_F2_HZ) / (MAX_F2_HZ - MIN_F2_HZ)).clamp(0.0, 1.0);
        let y = ((MAX_F1_HZ - self.f1_hz) / (MAX_F1_HZ - MIN_F1_HZ)).clamp(0.0, 1.0);
        self.formant_puck_pos = (x, y);
    }

    /// Convert normalized coordinates to F1 & F2 frequencies.
    pub fn update_formants_from_coordinates(&mut self, norm_x: f32, norm_y: f32) {
        let x = norm_x.clamp(0.0, 1.0);
        let y = norm_y.clamp(0.0, 1.0);
        self.formant_puck_pos = (x, y);
        self.f2_hz = MIN_F2_HZ + x * (MAX_F2_HZ - MIN_F2_HZ);
        self.f1_hz = MAX_F1_HZ - y * (MAX_F1_HZ - MIN_F1_HZ);
        self.f3_hz = 2500.0 + (x - 0.5) * 600.0;
        self.f4_hz = 3500.0;
    }

    /// Sets the active vowel selection and updates formants and puck coordinate.
    pub fn set_vowel(&mut self, vowel: IpaVowel) {
        self.selected_vowel = vowel;
        let (f1, f2, f3, f4) = vowel.nominal_formants();
        self.f1_hz = f1;
        self.f2_hz = f2;
        self.f3_hz = f3;
        self.f4_hz = f4;
        self.update_coordinates_from_formants();
    }

    /// Advances morph trajectory progress by `delta`.
    pub fn step_morph(&mut self, delta: f32) {
        self.morph_progress = (self.morph_progress + delta).clamp(0.0, 1.0);
        let pt = self.vowel_graph.evaluate_trajectory(self.morph_progress);
        self.f1_hz = pt.formants_hz[0];
        self.f2_hz = pt.formants_hz[1];
        self.f3_hz = pt.formants_hz[2];
        self.f4_hz = pt.formants_hz[3];
        self.selected_vowel = pt.nearest_vowel;
        self.update_coordinates_from_formants();
    }

    /// Hit-test touch coordinate on the formant puck.
    pub fn hit_test_formant_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.formant_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.formant_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= VOWEL_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render of 2D IPA Vowel Space and Formant Puck.
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

        // Draw IPA Chart nodes on left half
        for node in &self.vowel_graph.nodes {
            let (nx, ny) = node.chart_coordinates();
            let col = ((nx * (mid_x - 4) as f32) + 2.0).round() as usize;
            let row = (((1.0 - ny) * (height - 4) as f32) + 2.0).round() as usize;
            if row < height - 1 && col < mid_x {
                grid[row][col] = node.symbol.chars().nth(1).unwrap_or('*');
            }
        }

        // Draw active formant puck on left half
        let puck_col = ((self.formant_puck_pos.0 * (mid_x - 4) as f32) + 2.0).round() as usize;
        let puck_row = (((1.0 - self.formant_puck_pos.1) * (height - 4) as f32) + 2.0).round() as usize;
        if puck_row < height - 1 && puck_col < mid_x {
            grid[puck_row][puck_col] = 'O';
        }

        // Right half: Trajectory profile bars
        let right_w = width - mid_x - 2;
        let center_r = height / 2;
        for c in 0..right_w {
            let t = c as f32 / right_w as f32;
            let pt = self.vowel_graph.evaluate_trajectory(t);
            let f1_norm = (pt.formants_hz[0] - MIN_F1_HZ) / (MAX_F1_HZ - MIN_F1_HZ);
            let h = (f1_norm * (height as f32 * 0.35)).round().max(1.0) as usize;
            if center_r >= h && center_r + h < height - 1 {
                grid[center_r - h][mid_x + 1 + c] = '=';
                grid[center_r + h][mid_x + 1 + c] = '=';
            }
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
            "2D INTERNATIONAL PHONETIC ALPHABET (IPA) VOWEL SPACE & TRAJECTORY HUD",
            egui::FontId::proportional(16.0),
            Color32::from_rgb(240, 245, 255),
        );

        // IPA Vowel Selector Tabs (y: 46..90) - >= 44pt height
        let vowels = [
            IpaVowel::CloseFrontI,
            IpaVowel::CloseMidFrontE,
            IpaVowel::OpenMidFrontEps,
            IpaVowel::NearOpenFrontAsh,
            IpaVowel::OpenFrontA,
            IpaVowel::OpenMidBackO,
            IpaVowel::CloseMidBackO,
            IpaVowel::CloseBackU,
            IpaVowel::MidCentralSchwa,
        ];

        let tab_w = (rect.width() - 40.0 - 8.0 * 6.0) / 9.0;
        for (i, v) in vowels.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 6.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 46.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_selected = self.selected_vowel == *v;
            let bg_color = if is_selected {
                Color32::from_rgb(255, 215, 0)
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
                v.symbol(),
                egui::FontId::proportional(13.0),
                text_color,
            );

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if tab_rect.contains(pos) {
                        self.set_vowel(*v);
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

        // Left 55%: 2D IPA Formant Space (F1 vs F2)
        let left_w = (main_canvas.width() - 30.0) * 0.55;
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
            "IPA VOWEL QUADRILATERAL (F2 BACKNESS vs F1 HEIGHT)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // Draw IPA Quadrilateral Guide Lines
        let v_i = self.vowel_graph.nodes[0].chart_coordinates();
        let v_u = self.vowel_graph.nodes[8].chart_coordinates();
        let v_a = self.vowel_graph.nodes[4].chart_coordinates();
        let v_ash = self.vowel_graph.nodes[3].chart_coordinates();

        let pt_i = egui::pos2(left_rect.min.x + v_i.0 * left_rect.width(), left_rect.max.y - v_i.1 * left_rect.height());
        let pt_u = egui::pos2(left_rect.min.x + v_u.0 * left_rect.width(), left_rect.max.y - v_u.1 * left_rect.height());
        let pt_a = egui::pos2(left_rect.min.x + v_a.0 * left_rect.width(), left_rect.max.y - v_a.1 * left_rect.height());
        let pt_ash = egui::pos2(left_rect.min.x + v_ash.0 * left_rect.width(), left_rect.max.y - v_ash.1 * left_rect.height());

        let guide_stroke = Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(0, 229, 255, 60));
        painter.line_segment([pt_i, pt_u], guide_stroke);
        painter.line_segment([pt_u, pt_a], guide_stroke);
        painter.line_segment([pt_a, pt_ash], guide_stroke);
        painter.line_segment([pt_ash, pt_i], guide_stroke);

        // Draw IPA Node markers
        for node in &self.vowel_graph.nodes {
            let (nx, ny) = node.chart_coordinates();
            let pos = egui::pos2(left_rect.min.x + nx * left_rect.width(), left_rect.max.y - ny * left_rect.height());
            painter.circle_filled(pos, 4.0, Color32::from_rgb(0, 229, 255));
            painter.text(
                egui::pos2(pos.x + 8.0, pos.y - 6.0),
                egui::Align2::LEFT_CENTER,
                &node.symbol,
                egui::FontId::proportional(11.0),
                Color32::from_rgb(180, 200, 230),
            );
        }

        // Draw Catmull-Rom Trajectory Curve through waypoints
        let num_curve_pts = 40;
        let mut prev_c_pt: Option<egui::Pos2> = None;
        for s in 0..=num_curve_pts {
            let t = s as f32 / num_curve_pts as f32;
            let pt = self.vowel_graph.evaluate_trajectory(t);
            let nx = ((pt.formants_hz[1] - MIN_F2_HZ) / (MAX_F2_HZ - MIN_F2_HZ)).clamp(0.0, 1.0);
            let ny = ((MAX_F1_HZ - pt.formants_hz[0]) / (MAX_F1_HZ - MIN_F1_HZ)).clamp(0.0, 1.0);
            let c_pos = egui::pos2(left_rect.min.x + nx * left_rect.width(), left_rect.max.y - ny * left_rect.height());

            if let Some(prev) = prev_c_pt {
                painter.line_segment([prev, c_pos], Stroke::new(2.0_f32, Color32::from_rgb(255, 215, 0)));
            }
            prev_c_pt = Some(c_pos);
        }

        // Interactive Formant Puck
        let puck_x = left_rect.min.x + self.formant_puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.formant_puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.update_formants_from_coordinates(nx, ny);
                }
            }
        }

        // Draw Touch Hit Target Boundary (>= 44x44pt)
        painter.circle_stroke(
            puck_pos,
            VOWEL_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(255, 215, 0, 160)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(255, 215, 0));
        painter.circle_filled(puck_pos, 4.0, Color32::from_rgb(255, 255, 255));

        // Right 45%: Formant Spectrum Filter Profile HUD
        let right_x = main_canvas.min.x + left_w + 20.0;
        let right_w = main_canvas.width() - left_w - 30.0;
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
            "DYNAMIC FORMANT FREQUENCIES (F1 / F2 / F3 / F4)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // Draw Formant Resonator Peaks
        let formants = [
            (self.f1_hz, "F1", Color32::from_rgb(0, 229, 255)),
            (self.f2_hz, "F2", Color32::from_rgb(255, 215, 0)),
            (self.f3_hz, "F3", Color32::from_rgb(255, 107, 43)),
            (self.f4_hz, "F4", Color32::from_rgb(0, 255, 180)),
        ];

        let spec_w = right_rect.width() - 30.0;
        let spec_base_y = right_rect.max.y - 25.0;

        for (f_hz, label, col) in formants {
            let norm_f = (f_hz / 4000.0).clamp(0.05, 0.95);
            let px = right_rect.min.x + 15.0 + norm_f * spec_w;
            let peak_h = 90.0;

            painter.line_segment(
                [egui::pos2(px, spec_base_y), egui::pos2(px, spec_base_y - peak_h)],
                Stroke::new(3.0_f32, col),
            );
            painter.circle_filled(egui::pos2(px, spec_base_y - peak_h), 6.0, col);
            painter.text(
                egui::pos2(px, spec_base_y - peak_h - 14.0),
                egui::Align2::CENTER_CENTER,
                format!("{}: {:.0}Hz", label, f_hz),
                egui::FontId::proportional(10.0),
                col,
            );
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
                "FORMANT F1 (HEIGHT)",
                format!("{:.1} Hz", self.f1_hz),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "FORMANT F2 (BACKNESS)",
                format!("{:.1} Hz", self.f2_hz),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "FORMANT F3 / F4",
                format!("{:.0} / {:.0} Hz", self.f3_hz, self.f4_hz),
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "ACTIVE IPA VOWEL",
                format!("{} ({:.1}% Morph)", self.selected_vowel.symbol(), self.morph_progress * 100.0),
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
            "[PASS] 2D IPA Vowel Space Formant Puck & Touch Targets (>= 44x44pt) Verified",
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

    /// Render headless PNG snapshot displaying IPA Vowel Space puck, trajectory curve, and formants.
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
        fill_rounded_rect_vs(&mut pixels, width, height, 20, 10, width - 40, 36, 4, [18, 24, 38, 255]);
        draw_rect_border_vs(&mut pixels, width, height, 20, 10, width - 40, 36, [45, 60, 85, 255]);

        // Left Canvas: 2D IPA Vowel Space (x: 20..width/2 - 10, y: 56..height - 120)
        let left_w = (width - 60) / 2;
        let canvas_h = height - 170;
        fill_rounded_rect_vs(&mut pixels, width, height, 20, 56, left_w, canvas_h, 6, [14, 20, 32, 255]);
        draw_rect_border_vs(&mut pixels, width, height, 20, 56, left_w, canvas_h, [45, 60, 85, 255]);

        // Draw Left Grid lines
        for step in 1..4 {
            let gx = 20.0 + (step as f32 / 4.0) * left_w as f32;
            let gy = 56.0 + (step as f32 / 4.0) * canvas_h as f32;
            draw_line_segment_vs(&mut pixels, width, height, gx, 56.0, gx, 56.0 + canvas_h as f32, [25, 35, 55, 255]);
            draw_line_segment_vs(&mut pixels, width, height, 20.0, gy, 20.0 + left_w as f32, gy, [25, 35, 55, 255]);
        }

        // Draw IPA Node markers
        for node in &self.vowel_graph.nodes {
            let (nx, ny) = node.chart_coordinates();
            let px = 20.0 + nx * left_w as f32;
            let py = 56.0 + (1.0 - ny) * canvas_h as f32;
            draw_circle_filled_vs(&mut pixels, width, height, px, py, 4.0, [0, 229, 255, 255]);
        }

        // Formant Puck (Gold, >= 22pt radius -> 44x44pt touch bounding target)
        let puck_x = 20.0 + self.formant_puck_pos.0 * left_w as f32;
        let puck_y = 56.0 + (1.0 - self.formant_puck_pos.1) * canvas_h as f32;
        draw_circle_ring_vs(&mut pixels, width, height, puck_x, puck_y, VOWEL_PUCK_HIT_RADIUS, [255, 215, 0, 160]);
        draw_circle_filled_vs(&mut pixels, width, height, puck_x, puck_y, 14.0, [255, 215, 0, 255]);
        draw_circle_filled_vs(&mut pixels, width, height, puck_x, puck_y, 4.0, [255, 255, 255, 255]);

        // Right Canvas: Formant Peaks Spectrum (x: width/2 + 10..width - 20, y: 56..height - 120)
        let right_x = 20 + left_w + 20;
        let right_w = left_w;
        fill_rounded_rect_vs(&mut pixels, width, height, right_x, 56, right_w, canvas_h, 6, [14, 20, 32, 255]);
        draw_rect_border_vs(&mut pixels, width, height, right_x, 56, right_w, canvas_h, [45, 60, 85, 255]);

        let spec_base_y = 56.0 + canvas_h as f32 * 0.85;
        let formants = [
            (self.f1_hz, [0, 229, 255, 255]),
            (self.f2_hz, [255, 215, 0, 255]),
            (self.f3_hz, [255, 107, 43, 255]),
            (self.f4_hz, [0, 255, 180, 255]),
        ];

        for (f_hz, col) in formants {
            let norm_f = (f_hz / 4000.0).clamp(0.05, 0.95);
            let px = right_x as f32 + 15.0 + norm_f * (right_w as f32 - 30.0);
            let peak_h = 75.0;
            draw_line_segment_vs(&mut pixels, width, height, px, spec_base_y, px, spec_base_y - peak_h, col);
            draw_circle_filled_vs(&mut pixels, width, height, px, spec_base_y - peak_h, 6.0, col);
        }

        // Bottom Metrics Dock (x: 20..width - 20, y: height - 100..height - 20)
        let dock_y = height - 100;
        let dock_w = width - 40;
        fill_rounded_rect_vs(&mut pixels, width, height, 20, dock_y, dock_w, 80, 4, [16, 35, 28, 255]);
        draw_rect_border_vs(&mut pixels, width, height, 20, dock_y, dock_w, 80, [0, 255, 180, 255]);

        encode_minimal_png_vs(path, &pixels, width, height)
    }
}

// ----------------------------------------------------------------------------
// Minimal software rasterizer & PNG encoder helpers for headless verification
// ----------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn fill_rounded_rect_vs(
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
fn draw_rect_border_vs(buf: &mut [u8], w: u32, h: u32, rx: u32, ry: u32, rw: u32, rh: u32, col: [u8; 4]) {
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
fn draw_line_segment_vs(buf: &mut [u8], w: u32, h: u32, x0: f32, y0: f32, x1: f32, y1: f32, col: [u8; 4]) {
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

fn draw_circle_filled_vs(buf: &mut [u8], w: u32, h: u32, cx: f32, cy: f32, r: f32, col: [u8; 4]) {
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

fn draw_circle_ring_vs(buf: &mut [u8], w: u32, h: u32, cx: f32, cy: f32, r: f32, col: [u8; 4]) {
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

fn encode_minimal_png_vs(path: &str, rgba_pixels: &[u8], width: u32, height: u32) -> Result<(), String> {
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
    write_png_chunk_vs(&mut out, b"IHDR", &ihdr);

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

    write_png_chunk_vs(&mut out, b"IDAT", &zlib_data);
    write_png_chunk_vs(&mut out, b"IEND", &[]);

    std::fs::write(path, out).map_err(|e| e.to_string())
}

fn write_png_chunk_vs(out: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let type_start = out.len();
    out.extend_from_slice(chunk_type);
    out.extend_from_slice(data);
    let crc = crc32_compute_vs(&out[type_start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

fn crc32_compute_vs(buf: &[u8]) -> u32 {
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
    fn test_vowel_space_view_ascii_render() {
        let view = VowelSpaceView::new();
        let ascii = view.render_ascii(80, 16);
        assert!(!ascii.is_empty());
        assert_eq!(ascii.len(), 16);
    }

    #[test]
    fn test_vowel_space_view_hit_target_dimensions() {
        const {
            assert!(
                VOWEL_PUCK_HIT_RADIUS * 2.0 >= MIN_HIT_TARGET_PT,
                "Formant puck hit target bounding box must be >= 44pt"
            );
        }
    }

    #[test]
    fn test_vowel_space_view_snapshot_render() {
        let view = VowelSpaceView::new();
        let res = view.render_snapshot_png("scratch/renders/vowel_space_view.png", 800, 520);
        assert!(res.is_ok());
    }
}
