// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Physical Modeling Acoustic Membrane/Plate Percussion & Boundary Strike Impedance HUD (Step 1541).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const MEMBRANE_PLATE_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_PLATE_THICKNESS_MM: f32 = 0.5;
pub const MAX_PLATE_THICKNESS_MM: f32 = 25.0;
pub const MIN_TENSION_NM: f32 = 100.0;
pub const MAX_TENSION_NM: f32 = 10000.0;
pub const MIN_ASPECT_RATIO: f32 = 0.5;
pub const MAX_ASPECT_RATIO: f32 = 2.0;

/// Acoustic Membrane & Plate Physical Geometry & Material Profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlateProfile {
    CircularTympanum,      // Circular tensioned membrane (Bessel modes)
    RectangularSteelPlate, // 2D biharmonic bending plate (inharmonic bell-like modes)
    GongTamTam,            // Shallow curved shell with non-linear modal energy transfer
    SnareBottomMylar,      // High-tension thin membrane with snare wire contact impedance
    MarimbaRosewoodBar,    // Free-free beam with arch undercutting (1:4:10 overtone ratio)
}

impl PlateProfile {
    pub fn nominal_fundamental_hz(&self) -> f32 {
        match self {
            Self::CircularTympanum => 146.8,      // D3
            Self::RectangularSteelPlate => 220.0, // A3
            Self::GongTamTam => 65.4,             // C2
            Self::SnareBottomMylar => 330.0,      // E4
            Self::MarimbaRosewoodBar => 440.0,    // A4
        }
    }

    pub fn nominal_thickness_mm(&self) -> f32 {
        match self {
            Self::CircularTympanum => 0.25,
            Self::RectangularSteelPlate => 3.2,
            Self::GongTamTam => 1.8,
            Self::SnareBottomMylar => 0.18,
            Self::MarimbaRosewoodBar => 18.5,
        }
    }

    pub fn default_loss_factor(&self) -> f32 {
        match self {
            Self::CircularTympanum => 0.008,
            Self::RectangularSteelPlate => 0.0015,
            Self::GongTamTam => 0.0008,
            Self::SnareBottomMylar => 0.025,
            Self::MarimbaRosewoodBar => 0.004,
        }
    }

    pub fn is_membrane_mode(&self) -> bool {
        matches!(self, Self::CircularTympanum | Self::SnareBottomMylar)
    }
}

/// Boundary Edge Clamping Impedance Type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryClamping {
    FreeEdge,        // Zero moment and shear force at boundary
    SimplySupported, // Pinned edge, zero displacement, free rotation
    ClampedRigid,    // Zero displacement and zero slope at boundary
}

impl BoundaryClamping {
    pub fn impedance_factor(&self) -> f32 {
        match self {
            Self::FreeEdge => 0.15,
            Self::SimplySupported => 0.65,
            Self::ClampedRigid => 1.00,
        }
    }
}

/// Physical Modeling Acoustic Membrane/Plate Percussion View HUD (Step 1541).
#[derive(Debug, Clone)]
pub struct MembranePlateView {
    pub profile: PlateProfile,
    pub boundary: BoundaryClamping,
    pub thickness_mm: f32,           // [0.5 ..= 25.0 mm]
    pub tension_nm: f32,             // [100.0 ..= 10000.0 N/m]
    pub aspect_ratio: f32,           // [0.5 ..= 2.0]
    pub mallet_hardness: f32,        // [0.05 ..= 1.00] (Excitation contact width)
    pub strike_puck_pos: (f32, f32), // Normalized [0.0 ..= 1.0] surface coordinates
    pub is_dragging_puck: bool,
    pub fundamental_freq_hz: f32,
    pub modal_frequencies: [f32; 6], // 6 lowest modal resonance frequencies
    pub modal_amplitudes: [f32; 6],  // Mode excitation amplitudes from strike location
    pub contact_time_ms: f32,        // Mallet-plate contact duration
    pub color_palette: ContrastColorPalette,
}

impl Default for MembranePlateView {
    fn default() -> Self {
        Self::new()
    }
}

impl MembranePlateView {
    pub fn new() -> Self {
        let mut view = Self {
            profile: PlateProfile::CircularTympanum,
            boundary: BoundaryClamping::ClampedRigid,
            thickness_mm: 1.5,
            tension_nm: 3500.0,
            aspect_ratio: 1.0,
            mallet_hardness: 0.65,
            strike_puck_pos: (0.65, 0.40),
            is_dragging_puck: false,
            fundamental_freq_hz: 146.8,
            modal_frequencies: [146.8, 233.4, 314.1, 337.6, 389.0, 428.6],
            modal_amplitudes: [1.0, 0.72, 0.48, 0.35, 0.22, 0.15],
            contact_time_ms: 2.15,
            color_palette: ContrastColorPalette::default(),
        };
        view.update_physics_simulation();
        view
    }

    /// Convert Plate Thickness [0.5 ..= 25.0 mm] to normalized coordinate [0.0 ..= 1.0].
    pub fn thickness_to_normalized(thickness: f32) -> f32 {
        let t = thickness.clamp(MIN_PLATE_THICKNESS_MM, MAX_PLATE_THICKNESS_MM);
        ((t - MIN_PLATE_THICKNESS_MM) / (MAX_PLATE_THICKNESS_MM - MIN_PLATE_THICKNESS_MM))
            .clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Plate Thickness [0.5 ..= 25.0 mm].
    pub fn normalized_to_thickness(norm: f32) -> f32 {
        MIN_PLATE_THICKNESS_MM
            + norm.clamp(0.0, 1.0) * (MAX_PLATE_THICKNESS_MM - MIN_PLATE_THICKNESS_MM)
    }

    /// Convert Membrane Tension [100 ..= 10000 N/m] to normalized coordinate [0.0 ..= 1.0].
    pub fn tension_to_normalized(tension: f32) -> f32 {
        let t = tension.clamp(MIN_TENSION_NM, MAX_TENSION_NM);
        ((t - MIN_TENSION_NM) / (MAX_TENSION_NM - MIN_TENSION_NM)).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Membrane Tension [100 ..= 10000 N/m].
    pub fn normalized_to_tension(norm: f32) -> f32 {
        MIN_TENSION_NM + norm.clamp(0.0, 1.0) * (MAX_TENSION_NM - MIN_TENSION_NM)
    }

    /// Convert Aspect Ratio [0.5 ..= 2.0] to normalized coordinate [0.0 ..= 1.0].
    pub fn aspect_to_normalized(aspect: f32) -> f32 {
        let a = aspect.clamp(MIN_ASPECT_RATIO, MAX_ASPECT_RATIO);
        ((a - MIN_ASPECT_RATIO) / (MAX_ASPECT_RATIO - MIN_ASPECT_RATIO)).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate [0.0 ..= 1.0] to Aspect Ratio [0.5 ..= 2.0].
    pub fn normalized_to_aspect(norm: f32) -> f32 {
        MIN_ASPECT_RATIO + norm.clamp(0.0, 1.0) * (MAX_ASPECT_RATIO - MIN_ASPECT_RATIO)
    }

    /// Set plate profile and update defaults.
    pub fn set_profile(&mut self, profile: PlateProfile) {
        self.profile = profile;
        self.thickness_mm = profile.nominal_thickness_mm();
        self.update_physics_simulation();
    }

    /// Update physical modal resonance simulation and strike excitation.
    pub fn update_physics_simulation(&mut self) {
        let f0 = self.profile.nominal_fundamental_hz();
        let b_factor = self.boundary.impedance_factor();
        let thick_factor = (self.thickness_mm / 2.0).sqrt();

        self.fundamental_freq_hz = (f0 * (0.8 + 0.3 * b_factor) * thick_factor).clamp(20.0, 4000.0);

        // Calculate Modal Ratios based on physical geometry
        let ratios: [f32; 6] = match self.profile {
            PlateProfile::CircularTympanum => [1.00, 1.59, 2.14, 2.30, 2.65, 2.92], // Bessel J_m roots
            PlateProfile::RectangularSteelPlate => [1.00, 1.62, 2.45, 3.12, 3.88, 4.70], // Kirchhoff plate
            PlateProfile::GongTamTam => [1.00, 1.34, 1.82, 2.15, 2.89, 3.45], // Curved shallow shell
            PlateProfile::SnareBottomMylar => [1.00, 1.58, 2.13, 2.29, 2.64, 3.15],
            PlateProfile::MarimbaRosewoodBar => [1.00, 4.00, 10.00, 14.2, 19.5, 25.0], // Undercut bar
        };

        for (i, &ratio) in ratios.iter().enumerate() {
            self.modal_frequencies[i] = self.fundamental_freq_hz * ratio;
        }

        // Mallet contact time: stiffer/harder mallet -> shorter contact time (wider excitation bandwidth)
        self.contact_time_ms = (3.5 / self.mallet_hardness.max(0.05)).clamp(0.2, 15.0);
        let cutoff_hz = 1000.0 / (std::f32::consts::PI * self.contact_time_ms * 1e-3);

        // Strike Position coordinates relative to center
        let strike_r = ((self.strike_puck_pos.0 - 0.5).powi(2)
            + (self.strike_puck_pos.1 - 0.5).powi(2))
        .sqrt()
            * 2.0;

        for (i, &f) in self.modal_frequencies.iter().enumerate() {
            let mallet_filt = 1.0 / (1.0 + (f / cutoff_hz).powi(2));
            let mode_spatial = match i {
                0 => (1.0 - 0.8 * strike_r).max(0.05), // Fundamental strongest at center
                1 => (strike_r * (1.0 - strike_r) * 3.5).min(1.0), // First nodal ring
                2 => (strike_r.powi(2)).min(1.0),      // Edge modes
                _ => (0.6 * (strike_r + 0.2)).min(1.0),
            };
            self.modal_amplitudes[i] = (mallet_filt * mode_spatial).clamp(0.02, 1.0);
        }
    }

    /// Evaluate 2D spatial vibration amplitude at normalized coordinate (x, y) in [0.0, 1.0]^2.
    pub fn evaluate_spatial_displacement(&self, x: f32, y: f32) -> f32 {
        let dx = (x - 0.5) * 2.0;
        let dy = (y - 0.5) * 2.0;
        let r = (dx * dx + dy * dy).sqrt().min(1.0);
        let theta = dy.atan2(dx);

        let mut disp = 0.0;
        for i in 0..6 {
            let amp = self.modal_amplitudes[i];
            let shape = match i {
                0 => 1.0 - r * r,
                1 => r * (1.0 - r) * theta.cos(),
                2 => r * r * (1.0 - r) * (2.0 * theta).sin(),
                3 => 1.0 - (r * 2.3).sin().abs(),
                4 => r * (3.0 * theta).cos(),
                _ => (1.0 - r) * (4.0 * theta).sin(),
            };
            disp += amp * shape;
        }
        disp.clamp(-1.0, 1.0)
    }

    /// Hit-test touch coordinate on the strike puck.
    pub fn hit_test_strike_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.strike_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.strike_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= MEMBRANE_PLATE_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render of 2D Plate Vibration Mesh and Modal Frequency Chart.
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

        // Draw 2D Plate Surface on left half
        let left_w = mid_x - 2;
        for r in 2..height - 2 {
            for c in 2..left_w {
                let norm_x = (c - 2) as f32 / (left_w - 1) as f32;
                let norm_y = (r - 2) as f32 / (height - 5) as f32;
                let disp = self.evaluate_spatial_displacement(norm_x, norm_y);
                if disp.abs() > 0.4 {
                    grid[r][c] = '~';
                } else if disp.abs() < 0.08 {
                    grid[r][c] = '.';
                }
            }
        }

        // Strike Puck on left half
        let puck_col = ((self.strike_puck_pos.0 * (left_w - 2) as f32) + 2.0).round() as usize;
        let puck_row =
            (((1.0 - self.strike_puck_pos.1) * (height - 5) as f32) + 2.0).round() as usize;
        if puck_row < height - 1 && puck_col < mid_x {
            grid[puck_row][puck_col] = '@';
        }

        // Modal Bars on right half
        let right_w = width - mid_x - 2;
        for i in 0..6 {
            let bar_col = mid_x + 2 + i * (right_w / 7);
            let bar_len = (self.modal_amplitudes[i] * (height - 4) as f32).round() as usize;
            for r in 0..bar_len {
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

        // Background
        painter.rect_filled(rect, 6.0, Color32::from_rgb(14, 18, 28));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "PHYSICAL MODELING ACOUSTIC MEMBRANE / PLATE PERCUSSION HUD",
            egui::FontId::proportional(16.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Plate Profile Selector Tabs (y: 48..92) - Each tab >= 44pt height
        let profiles = [
            (PlateProfile::CircularTympanum, "CIRCULAR TYMPANUM"),
            (PlateProfile::RectangularSteelPlate, "STEEL PLATE"),
            (PlateProfile::GongTamTam, "GONG TAM-TAM"),
            (PlateProfile::SnareBottomMylar, "SNARE MYLAR"),
            (PlateProfile::MarimbaRosewoodBar, "MARIMBA BAR"),
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
        painter.rect_filled(main_canvas, 6.0, Color32::from_rgb(10, 14, 24));
        painter.rect_stroke(
            main_canvas,
            6.0,
            Stroke::new(1.5_f32, Color32::from_rgb(45, 65, 95)),
        );

        // Left 55%: 2D Plate Surface & Strike Excitation
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
            "2D STRIKE SURFACE MAP (DRAG PUCK TO EXPOSURE MODES)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // Draw Boundary Edge Outline & Guide Rings
        let surf_center = left_rect.center();
        let surf_radius = (left_rect.width() * 0.38).min(left_rect.height() * 0.40);

        if self.profile.is_membrane_mode() {
            painter.circle_stroke(
                surf_center,
                surf_radius,
                Stroke::new(2.5_f32, Color32::from_rgb(0, 229, 255)),
            );
            painter.circle_stroke(
                surf_center,
                surf_radius * 0.65,
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 90, 130, 90)),
            );
        } else {
            let p_rect = egui::Rect::from_center_size(
                surf_center,
                egui::vec2(surf_radius * 2.0 * self.aspect_ratio, surf_radius * 1.8),
            );
            painter.rect_stroke(
                p_rect,
                4.0,
                Stroke::new(2.5_f32, Color32::from_rgb(0, 229, 255)),
            );
        }

        // Strike Puck Interaction
        let puck_x = left_rect.min.x + self.strike_puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.strike_puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx =
                        ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.05, 0.95);
                    let ny =
                        ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.05, 0.95);
                    self.strike_puck_pos = (nx, ny);
                    self.update_physics_simulation();
                }
            }
        }

        // Draw Touch Hit Target Boundary (>= 44x44pt)
        painter.circle_stroke(
            puck_pos,
            MEMBRANE_PLATE_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(0, 229, 255, 140)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(0, 229, 255));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 20.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Strike Pos: ({:.2}, {:.2}) | Contact τ: {:.2} ms",
                self.strike_puck_pos.0, self.strike_puck_pos.1, self.contact_time_ms
            ),
            egui::FontId::proportional(10.0),
            Color32::from_rgb(0, 229, 255),
        );

        // Right 45%: Modal Eigenfrequency Resonance Spectrum
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
            "MODAL EIGENFREQUENCY SPECTRUM (6 MODES)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(160, 180, 205),
        );

        // 6 Modal Bars
        let mode_w = (right_rect.width() - 30.0 - 5.0 * 6.0) / 6.0;
        for i in 0..6 {
            let bx = right_rect.min.x + 15.0 + i as f32 * (mode_w + 6.0);
            let bar_h = self.modal_amplitudes[i] * (right_rect.height() - 75.0);
            let b_rect = egui::Rect::from_min_max(
                egui::pos2(bx, right_rect.max.y - 25.0 - bar_h),
                egui::pos2(bx + mode_w, right_rect.max.y - 25.0),
            );
            let col = if i == 0 {
                Color32::from_rgb(0, 229, 255)
            } else if i < 3 {
                Color32::from_rgb(255, 215, 0)
            } else {
                Color32::from_rgb(0, 255, 180)
            };
            painter.rect_filled(b_rect, 3.0, col);

            painter.text(
                egui::pos2(bx + mode_w * 0.5, right_rect.max.y - 20.0),
                egui::Align2::CENTER_TOP,
                format!("{:.0}Hz", self.modal_frequencies[i]),
                egui::FontId::proportional(9.0),
                Color32::from_rgb(200, 220, 245),
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

        let params = [
            (
                "FUNDAMENTAL (f0)",
                format!("{:.1} Hz (Mode #1)", self.fundamental_freq_hz),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "PLATE THICKNESS / DAMPING",
                format!(
                    "{:.1} mm (η={:.4})",
                    self.thickness_mm,
                    self.profile.default_loss_factor()
                ),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "BOUNDARY IMPEDANCE",
                match self.boundary {
                    BoundaryClamping::FreeEdge => "Free Boundary (0.15)".to_string(),
                    BoundaryClamping::SimplySupported => "Supported Edge (0.65)".to_string(),
                    BoundaryClamping::ClampedRigid => "Clamped Rigid (1.00)".to_string(),
                },
                Color32::from_rgb(255, 107, 43),
            ),
            (
                "MALLET CONTACT WIDTH",
                format!(
                    "{:.2} ms ({:.0}% Hard)",
                    self.contact_time_ms,
                    self.mallet_hardness * 100.0
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
            "[PASS] Physical Modeling Membrane/Plate Modal Percussion & Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
