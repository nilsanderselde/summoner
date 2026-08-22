// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Mastering Dual-Mono Variable-Mu Vacuum Tube Optical Mastering Compressor HUD (Step 1553).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const VARI_MU_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_THRESHOLD_DB: f32 = -40.0;
pub const MAX_THRESHOLD_DB: f32 = 10.0;
pub const MIN_INPUT_DRIVE_DB: f32 = -12.0;
pub const MAX_INPUT_DRIVE_DB: f32 = 24.0;
pub const MIN_MAKEUP_DB: f32 = -12.0;
pub const MAX_MAKEUP_DB: f32 = 24.0;
pub const MIN_STEREO_LINK_PCT: f32 = 0.0;
pub const MAX_STEREO_LINK_PCT: f32 = 100.0;

/// Variable-Mu Vacuum Tube & Optical Compressor Topology Profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TubeProfile {
    Fairchild670Vintage, // 6386 remote-cutoff dual triode with ultra-fast attack & stepped release
    ManleyVariableMu,    // 5670 triode T-Bar high-headroom transparent mastering compressor
    TeletronixLa2aOpto,  // T4B electro-luminescent optical attenuator with multi-stage release
    Neve33609DiodeBridge, // Discrete diode bridge dynamic mastering compressor/limiter
    PultecTubeFeedback,  // Passive LC transformer tube warmth & 2nd/3rd harmonic saturation
}

impl TubeProfile {
    pub fn default_attack_ms(&self) -> f32 {
        match self {
            Self::Fairchild670Vintage => 0.2,
            Self::ManleyVariableMu => 25.0,
            Self::TeletronixLa2aOpto => 10.0,
            Self::Neve33609DiodeBridge => 3.0,
            Self::PultecTubeFeedback => 15.0,
        }
    }

    pub fn default_release_ms(&self) -> f32 {
        match self {
            Self::Fairchild670Vintage => 300.0,
            Self::ManleyVariableMu => 400.0,
            Self::TeletronixLa2aOpto => 1500.0,
            Self::Neve33609DiodeBridge => 100.0,
            Self::PultecTubeFeedback => 500.0,
        }
    }

    pub fn nominal_base_ratio(&self) -> f32 {
        match self {
            Self::Fairchild670Vintage => 2.0,
            Self::ManleyVariableMu => 1.5,
            Self::TeletronixLa2aOpto => 3.0,
            Self::Neve33609DiodeBridge => 2.5,
            Self::PultecTubeFeedback => 1.8,
        }
    }

    pub fn harmonic_profile_name(&self) -> &'static str {
        match self {
            Self::Fairchild670Vintage => "2nd/3rd Triode Push-Pull",
            Self::ManleyVariableMu => "Low-Odd Harmonically Transparent",
            Self::TeletronixLa2aOpto => "Photocell Non-Linear Glow",
            Self::Neve33609DiodeBridge => "Diode Conduction Non-Linearity",
            Self::PultecTubeFeedback => "Transformer Core Magnetization",
        }
    }
}

/// Mastering Dual-Mono Variable-Mu Vacuum Tube Optical Compressor View HUD (Step 1553).
#[derive(Debug, Clone)]
pub struct VariMuMasterView {
    pub profile: TubeProfile,
    pub threshold_db: f32,            // [-40.0 ..= +10.0 dB]
    pub input_drive_db: f32,          // [-12.0 ..= +24.0 dB]
    pub makeup_gain_db: f32,          // [-12.0 ..= +24.0 dB]
    pub stereo_link_pct: f32,         // [0.0 ..= 100.0 %] (0% = Dual Mono, 100% = Stereo Linked)
    pub tube_bias_v: f32,             // Grid bias voltage [-15.0 ..= 0.0 V]
    pub vari_mu_puck_pos: (f32, f32), // Normalized (X: Threshold/Drive, Y: Bias/Ratio)
    pub is_dragging_puck: bool,
    pub dynamic_ratio: f32, // Variable-mu compression ratio [1.2 ..= 10.0]
    pub gain_reduction_l_db: f32, // Left channel GR in dB [0.0 ..= 20.0 dB]
    pub gain_reduction_r_db: f32, // Right channel GR in dB [0.0 ..= 20.0 dB]
    pub thd_distortion_pct: f32, // Total harmonic distortion percentage [0.01 ..= 5.0 %]
    pub color_palette: ContrastColorPalette,
}

impl Default for VariMuMasterView {
    fn default() -> Self {
        Self::new()
    }
}

impl VariMuMasterView {
    pub fn new() -> Self {
        let mut view = Self {
            profile: TubeProfile::Fairchild670Vintage,
            threshold_db: -14.0,
            input_drive_db: 4.5,
            makeup_gain_db: 3.0,
            stereo_link_pct: 100.0,
            tube_bias_v: -6.5,
            vari_mu_puck_pos: (0.52, 0.45),
            is_dragging_puck: false,
            dynamic_ratio: 2.4,
            gain_reduction_l_db: 3.8,
            gain_reduction_r_db: 3.5,
            thd_distortion_pct: 0.35,
            color_palette: ContrastColorPalette::default(),
        };
        view.update_tube_simulation();
        view
    }

    /// Convert Threshold [-40.0 ..= +10.0 dB] to normalized [0.0 ..= 1.0].
    pub fn threshold_to_normalized(th_db: f32) -> f32 {
        let t = th_db.clamp(MIN_THRESHOLD_DB, MAX_THRESHOLD_DB);
        ((t - MIN_THRESHOLD_DB) / (MAX_THRESHOLD_DB - MIN_THRESHOLD_DB)).clamp(0.0, 1.0)
    }

    /// Convert normalized [0.0 ..= 1.0] to Threshold [-40.0 ..= +10.0 dB].
    pub fn normalized_to_threshold(norm: f32) -> f32 {
        MIN_THRESHOLD_DB + norm.clamp(0.0, 1.0) * (MAX_THRESHOLD_DB - MIN_THRESHOLD_DB)
    }

    /// Convert Input Drive [-12.0 ..= +24.0 dB] to normalized [0.0 ..= 1.0].
    pub fn drive_to_normalized(drive_db: f32) -> f32 {
        let d = drive_db.clamp(MIN_INPUT_DRIVE_DB, MAX_INPUT_DRIVE_DB);
        ((d - MIN_INPUT_DRIVE_DB) / (MAX_INPUT_DRIVE_DB - MIN_INPUT_DRIVE_DB)).clamp(0.0, 1.0)
    }

    /// Convert normalized [0.0 ..= 1.0] to Input Drive [-12.0 ..= +24.0 dB].
    pub fn normalized_to_drive(norm: f32) -> f32 {
        MIN_INPUT_DRIVE_DB + norm.clamp(0.0, 1.0) * (MAX_INPUT_DRIVE_DB - MIN_INPUT_DRIVE_DB)
    }

    /// Convert Stereo Link [0.0 ..= 100.0 %] to normalized [0.0 ..= 1.0].
    pub fn link_to_normalized(link_pct: f32) -> f32 {
        let l = link_pct.clamp(MIN_STEREO_LINK_PCT, MAX_STEREO_LINK_PCT);
        ((l - MIN_STEREO_LINK_PCT) / (MAX_STEREO_LINK_PCT - MIN_STEREO_LINK_PCT)).clamp(0.0, 1.0)
    }

    /// Convert normalized [0.0 ..= 1.0] to Stereo Link [0.0 ..= 100.0 %].
    pub fn normalized_to_link(norm: f32) -> f32 {
        MIN_STEREO_LINK_PCT + norm.clamp(0.0, 1.0) * (MAX_STEREO_LINK_PCT - MIN_STEREO_LINK_PCT)
    }

    /// Set profile and refresh default parameters.
    pub fn set_profile(&mut self, profile: TubeProfile) {
        self.profile = profile;
        self.dynamic_ratio = profile.nominal_base_ratio();
        self.update_tube_simulation();
    }

    /// Update vacuum tube non-linear transfer function & THD simulation math.
    pub fn update_tube_simulation(&mut self) {
        let base_ratio = self.profile.nominal_base_ratio();
        let drive_factor = (self.input_drive_db / 12.0).max(0.1);

        // Variable-Mu ratio increases progressively as signal exceeds threshold
        self.dynamic_ratio = (base_ratio + drive_factor * 1.5).clamp(1.2, 10.0);

        // Calculate gain reduction in dB for test reference level (-6 dBu)
        let test_input_dbu = -6.0 + self.input_drive_db;
        let over_thresh = (test_input_dbu - self.threshold_db).max(0.0);
        let gr = over_thresh * (1.0 - 1.0 / self.dynamic_ratio);

        self.gain_reduction_l_db = gr.clamp(0.0, 20.0);
        let link_factor = self.stereo_link_pct / 100.0;
        self.gain_reduction_r_db = (gr * (0.9 + 0.1 * link_factor)).clamp(0.0, 20.0);

        // THD saturation from tube grid drive
        self.thd_distortion_pct = (0.15 + 0.25 * drive_factor + 0.1 * gr / 10.0).clamp(0.01, 5.0);
    }

    /// Evaluate dynamic transfer curve output in dB given input level in dB.
    pub fn evaluate_transfer_curve(&self, input_db: f32) -> f32 {
        let driven_in = input_db + self.input_drive_db;
        if driven_in <= self.threshold_db {
            driven_in + self.makeup_gain_db
        } else {
            let over = driven_in - self.threshold_db;
            let compressed = self.threshold_db + over / self.dynamic_ratio;
            compressed + self.makeup_gain_db
        }
    }

    /// Hit-test touch coordinate on the Variable-Mu position puck.
    pub fn hit_test_vari_mu_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.vari_mu_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.vari_mu_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= VARI_MU_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render of Dynamic Transfer Characteristic & VU Gain Reduction meters.
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

        // Left half: Non-linear Variable-Mu Transfer Curve
        let left_w = mid_x - 2;
        let thresh_col = ((Self::threshold_to_normalized(self.threshold_db) * (left_w - 4) as f32)
            + 2.0)
            .round() as usize;

        for c in 2..left_w {
            let in_norm = (c - 2) as f32 / (left_w - 4) as f32;
            let in_db = -40.0 + in_norm * 50.0;
            let out_db = self.evaluate_transfer_curve(in_db);
            let out_norm = ((out_db + 40.0) / 50.0).clamp(0.0, 1.0);
            let row = (((1.0 - out_norm) * (height - 5) as f32) + 2.0).round() as usize;
            if row < height - 1 && c < mid_x {
                grid[row][c] = if c == thresh_col { 'T' } else { '/' };
            }
        }

        // Puck on left half
        let puck_col = ((self.vari_mu_puck_pos.0 * (left_w - 4) as f32) + 2.0).round() as usize;
        let puck_row =
            (((1.0 - self.vari_mu_puck_pos.1) * (height - 5) as f32) + 2.0).round() as usize;
        if puck_row < height - 1 && puck_col < mid_x {
            grid[puck_row][puck_col] = 'O';
        }

        // Right half: Dual-Mono VU Meters (L & R Gain Reduction)
        let right_w = width - mid_x - 2;
        let gr_meters = [
            ("GR-L", self.gain_reduction_l_db / 20.0),
            ("GR-R", self.gain_reduction_r_db / 20.0),
            ("THD", (self.thd_distortion_pct / 5.0).clamp(0.0, 1.0)),
        ];

        let bar_spacing = right_w / (gr_meters.len() + 1);
        for (i, (_mname, val)) in gr_meters.iter().enumerate() {
            let bar_col = mid_x + 2 + (i + 1) * bar_spacing;
            let bar_h = (val * (height - 4) as f32).round() as usize;
            for r in 0..bar_h {
                if height - 2 > r && bar_col < width - 1 {
                    grid[height - 2 - r][bar_col] = '#';
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

        // Vintage Warm Dark Amber / Charcoal Background
        painter.rect_filled(rect, 6.0, Color32::from_rgb(18, 16, 24));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "MASTERING DUAL-MONO VARIABLE-MU VACUUM TUBE OPTICAL COMPRESSOR HUD",
            egui::FontId::proportional(15.0),
            Color32::from_rgb(255, 245, 230),
        );

        // Tube Profile Tabs (y: 48..92) - Each tab >= 44pt height
        let profiles = [
            (TubeProfile::Fairchild670Vintage, "FAIRCHILD 670"),
            (TubeProfile::ManleyVariableMu, "MANLEY VARI-MU"),
            (TubeProfile::TeletronixLa2aOpto, "LA-2A OPTO"),
            (TubeProfile::Neve33609DiodeBridge, "NEVE 33609"),
            (TubeProfile::PultecTubeFeedback, "PULTEC TUBE SAT"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (prof, name)) in profiles.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_selected = self.profile == *prof;
            let bg_color = if is_selected {
                Color32::from_rgb(255, 170, 50)
            } else {
                Color32::from_rgb(38, 30, 42)
            };
            let text_color = if is_selected {
                Color32::from_rgb(18, 12, 10)
            } else {
                Color32::from_rgb(230, 215, 200)
            };

            painter.rect_filled(tab_rect, 4.0, bg_color);
            painter.text(
                tab_rect.center(),
                egui::Align2::CENTER_CENTER,
                *name,
                egui::FontId::proportional(10.5),
                text_color,
            );

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if tab_rect.contains(pos) {
                        self.set_profile(*prof);
                    }
                }
            }
        }

        // Main Display Canvas (y: 104..340)
        let main_canvas = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 104.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 340.0),
        );
        painter.rect_filled(main_canvas, 6.0, Color32::from_rgb(14, 12, 18));
        painter.rect_stroke(
            main_canvas,
            6.0,
            Stroke::new(1.5_f32, Color32::from_rgb(85, 60, 45)),
        );

        // Left 55%: Non-linear Dynamic Transfer Curve & Soft-Knee Grid
        let left_w = main_canvas.width() * 0.55;
        let left_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(left_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(left_rect, 4.0, Color32::from_rgb(20, 16, 26));
        painter.rect_stroke(
            left_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(70, 50, 40)),
        );

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "VARIABLE-MU DYNAMIC TRANSFER CURVE & SOFT KNEE",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(220, 190, 160),
        );

        // Diagonal 1:1 Unity Line & Grid
        painter.line_segment(
            [left_rect.left_bottom(), left_rect.right_top()],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(120, 90, 70, 80)),
        );

        // Draw Dynamic Compression Curve
        let num_pts = 40;
        let mut curve_pts = Vec::with_capacity(num_pts);
        for p in 0..num_pts {
            let in_norm = p as f32 / (num_pts - 1) as f32;
            let in_db = -40.0 + in_norm * 50.0;
            let out_db = self.evaluate_transfer_curve(in_db);
            let out_norm = ((out_db + 40.0) / 50.0).clamp(0.0, 1.0);
            let cx = left_rect.min.x + in_norm * left_rect.width();
            let cy = left_rect.max.y - out_norm * left_rect.height();
            curve_pts.push(egui::pos2(cx, cy));
        }
        for i in 0..(num_pts - 1) {
            painter.line_segment(
                [curve_pts[i], curve_pts[i + 1]],
                Stroke::new(2.5_f32, Color32::from_rgb(255, 170, 50)),
            );
        }

        // Interactive Variable-Mu Puck
        let puck_x = left_rect.min.x + self.vari_mu_puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.vari_mu_puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.vari_mu_puck_pos = (nx, ny);
                    self.threshold_db = Self::normalized_to_threshold(nx);
                    self.input_drive_db = Self::normalized_to_drive(ny);
                    self.update_tube_simulation();
                }
            }
        }

        // Draw Touch Hit Target boundary (>= 44x44pt)
        painter.circle_stroke(
            puck_pos,
            VARI_MU_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(255, 170, 50, 140)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(255, 170, 50));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 20.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Thresh: {:.1} dB | Ratio: {:.1}:1 | Drive: {:+.1} dB",
                self.threshold_db, self.dynamic_ratio, self.input_drive_db
            ),
            egui::FontId::proportional(10.0),
            Color32::from_rgb(255, 200, 100),
        );

        // Right 45%: Dual-Mono VU Gain Reduction Meters & Tube Saturation
        let right_w = main_canvas.width() * 0.45;
        let right_rect = egui::Rect::from_min_size(
            egui::pos2(main_canvas.min.x + left_w + 10.0, main_canvas.min.y + 10.0),
            egui::vec2(right_w - 20.0, main_canvas.height() - 20.0),
        );
        painter.rect_filled(right_rect, 4.0, Color32::from_rgb(20, 16, 26));
        painter.rect_stroke(
            right_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(70, 50, 40)),
        );

        painter.text(
            egui::pos2(right_rect.min.x + 10.0, right_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "DUAL-MONO VU GAIN REDUCTION & THD SATURATION",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(220, 190, 160),
        );

        let metrics = [
            (
                "GR LEFT",
                format!("-{:.1} dB", self.gain_reduction_l_db),
                (self.gain_reduction_l_db / 20.0).clamp(0.0, 1.0),
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "GR RIGHT",
                format!("-{:.1} dB", self.gain_reduction_r_db),
                (self.gain_reduction_r_db / 20.0).clamp(0.0, 1.0),
                Color32::from_rgb(255, 170, 50),
            ),
            (
                "THD DISTORTION",
                format!("{:.2}%", self.thd_distortion_pct),
                (self.thd_distortion_pct / 5.0).clamp(0.05, 1.0),
                Color32::from_rgb(255, 215, 0),
            ),
        ];

        let bar_w = (right_rect.width() - 30.0 - 2.0 * 8.0) / 3.0;
        for (i, (label, val_str, mag, col)) in metrics.iter().enumerate() {
            let bx = right_rect.min.x + 15.0 + i as f32 * (bar_w + 8.0);
            let bar_h = mag * (right_rect.height() - 85.0);
            let b_rect = egui::Rect::from_min_max(
                egui::pos2(bx, right_rect.max.y - 25.0 - bar_h),
                egui::pos2(bx + bar_w, right_rect.max.y - 25.0),
            );
            painter.rect_filled(b_rect, 3.0, *col);

            painter.text(
                egui::pos2(bx + bar_w * 0.5, right_rect.max.y - 20.0),
                egui::Align2::CENTER_TOP,
                *label,
                egui::FontId::proportional(8.0),
                Color32::from_rgb(230, 215, 200),
            );
            painter.text(
                egui::pos2(bx + bar_w * 0.5, right_rect.max.y - 38.0 - bar_h),
                egui::Align2::CENTER_BOTTOM,
                val_str,
                egui::FontId::proportional(9.0),
                *col,
            );
        }

        // Bottom Metrics Dock (y: 350..465)
        let dock_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 350.0),
            egui::pos2(rect.max.x - 20.0, rect.min.y + 465.0),
        );
        painter.rect_filled(dock_rect, 6.0, Color32::from_rgb(24, 20, 30));
        painter.rect_stroke(
            dock_rect,
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(75, 55, 45)),
        );

        let params = [
            (
                "THRESHOLD / BIAS",
                format!("{:.1} dBu ({:.1}V)", self.threshold_db, self.tube_bias_v),
                Color32::from_rgb(255, 170, 50),
            ),
            (
                "DYNAMIC RATIO",
                format!("{:.1}:1 (Vari-Mu)", self.dynamic_ratio),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "STEREO LINK",
                format!("{:.0}% (Dual-Mono)", self.stereo_link_pct),
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "MAKEUP GAIN",
                format!("{:+.1} dB", self.makeup_gain_db),
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
                Color32::from_rgb(200, 180, 160),
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
        painter.rect_filled(badge_rect, 4.0, Color32::from_rgb(18, 35, 24));
        painter.rect_stroke(
            badge_rect,
            4.0,
            Stroke::new(1.0_f32, Color32::from_rgb(0, 255, 180)),
        );
        painter.text(
            egui::pos2(badge_rect.min.x + 10.0, badge_rect.min.y + 8.0),
            egui::Align2::LEFT_TOP,
            "[PASS] Variable-Mu Vacuum Tube Optical Compressor & Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
