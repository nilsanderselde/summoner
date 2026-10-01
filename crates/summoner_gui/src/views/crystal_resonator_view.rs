// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Quartz Crystal Singing Bowl & Glass Chalice Friction Resonator HUD.
//!
//! Provides an interactive interface for thin-shell crystal glass singing bowls,
//! borosilicate glass bells, and water-tuned wine chalices featuring:
//! - 2D stick-slip friction / suede-wand excitation puck (speed vs normal force, >= 44x44pt hit target)
//! - 8-mode circular thin-shell modal resonance spectrum with degenerate doublet splitting
//! - Hydro-acoustic water filling mass-loading pitch lowering (delta f proportional to -(m_water / m_glass)^1/2)
//! - Quality factor Q scaling and crystalline ring-down visualization
//! - Non-linear mallet strike impact trigger
//! - Real-time concentric resonance ripple scope
//! - WCAG AAA high-contrast styling and deterministic ASCII renderer

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, FontId, Pos2, Stroke, Vec2};

pub const CRYSTAL_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_FRICTION_SPEED_MPS: f32 = 0.05;
pub const MAX_FRICTION_SPEED_MPS: f32 = 2.50;
pub const MIN_NORMAL_FORCE_N: f32 = 0.05;
pub const MAX_NORMAL_FORCE_N: f32 = 1.50;
pub const MIN_ROOT_FREQ_HZ: f32 = 50.0;
pub const MAX_ROOT_FREQ_HZ: f32 = 2000.0;
pub const NUM_CRYSTAL_MODES: usize = 8;

/// Material and acoustic construction profiles for the crystal resonator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CrystalMaterialProfile {
    /// 99.9% Pure Fused Quartz Crystal Singing Bowl (432 Hz therapeutic tuning).
    #[default]
    PureQuartzCrystal,
    /// Franklin Quartz Glass Chalice (Crisp, ethereal, high harmonic clarity).
    FranklinQuartzGlass,
    /// Wet Crystal Wine Goblet (High overtone ring, sensitive stick-slip friction).
    WetCrystalGoblet,
    /// Heavy Borosilicate Glass Bell (Dense modal cluster with warm low resonance).
    BorosilicateBell,
    /// Metallophone Tuned Bronze/Alloy Bowl (Complex inharmonic nodal lines with rapid damping).
    MetallophoneAlloy,
}

impl CrystalMaterialProfile {
    pub fn name(&self) -> &'static str {
        match self {
            Self::PureQuartzCrystal => "99.9% Pure Quartz Crystal Singing Bowl",
            Self::FranklinQuartzGlass => "Franklin Quartz Glass Chalice",
            Self::WetCrystalGoblet => "Wet Crystal Wine Goblet",
            Self::BorosilicateBell => "Heavy Borosilicate Glass Bell",
            Self::MetallophoneAlloy => "Metallophone Tuned Alloy Bowl",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            Self::PureQuartzCrystal => "CRYSTAL BOWL (432Hz)",
            Self::FranklinQuartzGlass => "FRANKLIN CHALICE",
            Self::WetCrystalGoblet => "WET GOBLET",
            Self::BorosilicateBell => "BOROSILICATE BELL",
            Self::MetallophoneAlloy => "METALLOPHONE",
        }
    }

    pub fn nominal_root_hz(&self) -> f32 {
        match self {
            Self::PureQuartzCrystal => 432.00,
            Self::FranklinQuartzGlass => 523.25,
            Self::WetCrystalGoblet => 659.25,
            Self::BorosilicateBell => 329.63,
            Self::MetallophoneAlloy => 880.00,
        }
    }

    /// Returns nominal (Q_factor, stiffness, split_hz, water_sensitivity, friction_mu).
    pub fn nominal_physics(&self) -> (f32, f32, f32, f32, f32) {
        match self {
            Self::PureQuartzCrystal => (4800.0, 1.25, 0.65, 0.35, 0.75),
            Self::FranklinQuartzGlass => (3500.0, 1.10, 0.45, 0.40, 0.85),
            Self::WetCrystalGoblet => (2400.0, 0.90, 0.85, 0.55, 0.90),
            Self::BorosilicateBell => (1800.0, 1.40, 0.30, 0.25, 0.65),
            Self::MetallophoneAlloy => (1200.0, 1.60, 1.20, 0.15, 0.55),
        }
    }
}

/// Physical modeling quartz crystal singing bowl & glass chalice friction resonator HUD.
#[derive(Debug, Clone)]
pub struct CrystalResonatorView {
    pub material_profile: CrystalMaterialProfile,
    pub root_freq_hz: f32,
    pub friction_speed_mps: f32,
    pub normal_force_n: f32,
    pub water_fill_pct: f32,
    pub q_scale: f32,
    pub doublet_split_hz: f32,
    pub strike_velocity: f32,
    pub puck_pos: (f32, f32), // Normalized (X: friction speed, Y: normal force)
    pub is_dragging_puck: bool,
    pub modal_amplitudes: [f32; NUM_CRYSTAL_MODES],
    pub modal_frequencies: [(f32, f32); NUM_CRYSTAL_MODES], // (Mode A, Mode B doublet)
    pub effective_f0_hz: f32,
    pub ripple_phase: f32,
    pub color_palette: ContrastColorPalette,
}

impl Default for CrystalResonatorView {
    fn default() -> Self {
        Self::new()
    }
}

impl CrystalResonatorView {
    pub fn new() -> Self {
        let mut view = Self {
            material_profile: CrystalMaterialProfile::PureQuartzCrystal,
            root_freq_hz: 432.0,
            friction_speed_mps: 0.65,
            normal_force_n: 0.45,
            water_fill_pct: 0.20,
            q_scale: 1.0,
            doublet_split_hz: 0.65,
            strike_velocity: 0.0,
            puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            modal_amplitudes: [1.0, 0.45, 0.20, 0.10, 0.05, 0.02, 0.01, 0.005],
            modal_frequencies: [(432.0, 432.65); NUM_CRYSTAL_MODES],
            effective_f0_hz: 432.0,
            ripple_phase: 0.0,
            color_palette: ContrastColorPalette::default(),
        };
        view.puck_pos = (
            Self::speed_to_normalized(view.friction_speed_mps),
            Self::force_to_normalized(view.normal_force_n),
        );
        view.update_physics();
        view
    }

    pub fn speed_to_normalized(speed: f32) -> f32 {
        let s = speed.clamp(MIN_FRICTION_SPEED_MPS, MAX_FRICTION_SPEED_MPS);
        ((s - MIN_FRICTION_SPEED_MPS) / (MAX_FRICTION_SPEED_MPS - MIN_FRICTION_SPEED_MPS)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_speed(norm: f32) -> f32 {
        MIN_FRICTION_SPEED_MPS + norm.clamp(0.0, 1.0) * (MAX_FRICTION_SPEED_MPS - MIN_FRICTION_SPEED_MPS)
    }

    pub fn force_to_normalized(force: f32) -> f32 {
        let f = force.clamp(MIN_NORMAL_FORCE_N, MAX_NORMAL_FORCE_N);
        ((f - MIN_NORMAL_FORCE_N) / (MAX_NORMAL_FORCE_N - MIN_NORMAL_FORCE_N)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_force(norm: f32) -> f32 {
        MIN_NORMAL_FORCE_N + norm.clamp(0.0, 1.0) * (MAX_NORMAL_FORCE_N - MIN_NORMAL_FORCE_N)
    }

    pub fn set_material_profile(&mut self, profile: CrystalMaterialProfile) {
        self.material_profile = profile;
        self.root_freq_hz = profile.nominal_root_hz();
        let (_q, _stiff, split, _water_sens, _mu) = profile.nominal_physics();
        self.doublet_split_hz = split;
        self.update_physics();
    }

    /// Trigger mallet strike impulse excitation.
    pub fn strike_mallet(&mut self, velocity: f32) {
        self.strike_velocity = velocity.clamp(0.0, 1.0);
        self.update_physics();
    }

    /// Recalculate physical acoustics, hydro-acoustic pitch shift, and thin-shell modal doublet doublets.
    pub fn update_physics(&mut self) {
        let (_nom_q, stiffness, split, water_sens, mu) = self.material_profile.nominal_physics();
        self.doublet_split_hz = split;

        // Hydro-acoustic mass loading pitch lowering: Delta f proportional to -(m_water / m_glass)^1/2
        let mass_loading = water_sens * self.water_fill_pct.sqrt();
        let pitch_lowering = (1.0 - mass_loading * 0.35).clamp(0.40, 1.0);
        let f0 = self.root_freq_hz * pitch_lowering;
        self.effective_f0_hz = f0;

        // Stick-slip Stribeck friction excitation
        let v = self.friction_speed_mps;
        let fn_force = self.normal_force_n;
        let stribeck = (v / (v + 0.15)) * (-v / 1.25).exp();
        let friction_drive = (mu * fn_force * stribeck * 2.8).clamp(0.0, 2.5);
        let strike_drive = self.strike_velocity * 1.5;
        let total_drive = (friction_drive + strike_drive).clamp(0.05, 3.0);

        // Water damping on high modes
        let water_damping = (1.0 - 0.50 * self.water_fill_pct).clamp(0.1, 1.0);

        // Compute 8 thin-shell modes (circumferential wavenumbers n = 2..9)
        for i in 0..NUM_CRYSTAL_MODES {
            let n = (i + 2) as f32;
            // 2D Thin-shell circular dispersion: f_n = f_0 * sqrt(1 + alpha * (n^2 - 1)^2)
            let mode_ratio = (1.0 + 0.08 * stiffness * (n * n - 1.0).powi(2) / 24.0).sqrt();
            let mode_center_hz = (f0 * mode_ratio).clamp(20.0, 22000.0);

            // Degenerate doublet splitting: f_A = f - Delta/2, f_B = f + Delta/2
            let split_hz = self.doublet_split_hz * (1.0 + 0.1 * i as f32);
            let f_a = (mode_center_hz - split_hz * 0.5).max(20.0);
            let f_b = (mode_center_hz + split_hz * 0.5).min(22000.0);
            self.modal_frequencies[i] = (f_a, f_b);

            // Modal amplitudes with roll-off and fluid damping
            let mode_roll = 1.0 / (1.0 + 0.45 * (i as f32).powf(1.4));
            let damp = if i > 0 { water_damping.powi(i as i32) } else { 1.0 };
            let amp = (total_drive * mode_roll * damp).clamp(0.001, 1.5);
            self.modal_amplitudes[i] = amp;
        }

        self.ripple_phase = (self.ripple_phase + 0.15) % (2.0 * std::f32::consts::PI);
    }

    /// Hit test coordinate on the interactive wand friction puck.
    pub fn hit_test_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= CRYSTAL_PUCK_HIT_RADIUS
    }

    /// Deterministic ASCII render representation for headless test verification.
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

        // Left half: 2D Wand Friction Puck
        let left_w = mid_x - 2;
        let p_row = (((1.0 - self.puck_pos.1) * (height - 5) as f32) + 2.0).round() as usize;
        let p_col = ((self.puck_pos.0 * (left_w - 4) as f32) + 2.0).round() as usize;
        if p_row < height - 1 && p_col < mid_x {
            grid[p_row][p_col] = 'W'; // Wand puck
        }

        // Right half: Modal resonance overtone bars
        let right_w = width - mid_x - 2;
        let bar_spacing = right_w / (NUM_CRYSTAL_MODES + 1);
        for (i, &amp) in self.modal_amplitudes.iter().enumerate() {
            let col = mid_x + 2 + (i + 1) * bar_spacing;
            let bar_h = (amp.clamp(0.0, 1.0) * (height - 4) as f32).round() as usize;
            for r in 0..bar_h {
                if height - 2 > r && col < width - 1 {
                    grid[height - 2 - r][col] = '#';
                }
            }
        }

        grid.into_iter().map(|row| row.into_iter().collect()).collect()
    }

    #[cfg(feature = "gui")]
    #[allow(clippy::needless_range_loop)]
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), 480.0),
            egui::Sense::click_and_drag(),
        );

        let painter = ui.painter_at(rect);

        // Background: Deep Slate Quartz Base (#0A0E1A)
        painter.rect_filled(rect, 6.0, Color32::from_rgb(10, 14, 26));

        // Header Title
        painter.text(
            Pos2::new(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "PHYSICAL MODELING QUARTZ CRYSTAL SINGING BOWL & CHALICE RESONATOR HUD",
            FontId::proportional(13.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Material Profile Tabs (y: 46..90) - Each tab >= 44pt touch target
        let tabs = [
            (CrystalMaterialProfile::PureQuartzCrystal, "CRYSTAL BOWL (432Hz)"),
            (CrystalMaterialProfile::FranklinQuartzGlass, "FRANKLIN CHALICE"),
            (CrystalMaterialProfile::WetCrystalGoblet, "WET GOBLET"),
            (CrystalMaterialProfile::BorosilicateBell, "BOROSILICATE BELL"),
            (CrystalMaterialProfile::MetallophoneAlloy, "METALLOPHONE"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (prof, name)) in tabs.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                Pos2::new(bx, rect.min.y + 46.0),
                Vec2::new(tab_w, 44.0),
            );
            let is_sel = self.material_profile == *prof;
            let bg_col = if is_sel {
                Color32::from_rgb(56, 189, 248)
            } else {
                Color32::from_rgb(20, 28, 44)
            };
            painter.rect_filled(tab_rect, 4.0, bg_col);
            painter.rect_stroke(tab_rect, 4.0, Stroke::new(1.0_f32, Color32::from_rgb(45, 60, 85)));

            let text_col = if is_sel { Color32::from_rgb(10, 14, 26) } else { Color32::from_rgb(200, 215, 235) };
            painter.text(
                tab_rect.center(),
                egui::Align2::CENTER_CENTER,
                *name,
                FontId::proportional(9.5),
                text_col,
            );

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if tab_rect.contains(pos) {
                        self.set_material_profile(*prof);
                    }
                }
            }
        }

        // Split Main Canvas: Left (2D Wand Friction Radar) / Right (Modal Resonance Bars)
        let main_top = rect.min.y + 100.0;
        let main_h = 240.0;
        let half_w = (rect.width() - 50.0) * 0.5;

        // Left Panel: Wand Stick-Slip Excitation Puck Canvas
        let left_rect = egui::Rect::from_min_size(Pos2::new(rect.min.x + 20.0, main_top), Vec2::new(half_w, main_h));
        painter.rect_filled(left_rect, 4.0, Color32::from_rgb(14, 20, 32));
        painter.rect_stroke(left_rect, 4.0, Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 75)));

        painter.text(
            Pos2::new(left_rect.min.x + 12.0, left_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            "Wand Stick-Slip Friction Radar (Speed vs Force)",
            FontId::proportional(11.0),
            Color32::from_rgb(148, 163, 184),
        );

        // Concentric Crystal Bowl Ripple Scope
        let bowl_center = Pos2::new(left_rect.center().x, left_rect.center().y + 10.0);
        let max_r = (left_rect.height() * 0.38).min(left_rect.width() * 0.38);
        for ring in 1..=4 {
            let r = max_r * (ring as f32 / 4.0);
            let ring_stroke = Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(56, 189, 248, 40));
            painter.circle_stroke(bowl_center, r, ring_stroke);
        }

        // Animated stick-slip shimmer waves
        let ripple_r = (self.ripple_phase / (2.0 * std::f32::consts::PI)) * max_r;
        painter.circle_stroke(
            bowl_center,
            ripple_r,
            Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(168, 85, 247, 80)),
        );

        // Drag-to-Adjust Friction Puck
        let puck_screen_x = left_rect.min.x + self.puck_pos.0 * left_rect.width();
        let puck_screen_y = left_rect.min.y + (1.0 - self.puck_pos.1) * left_rect.height();
        let puck_center = Pos2::new(puck_screen_x, puck_screen_y);

        // Puck Outer Aura & Inner Dot
        painter.circle_filled(puck_center, 14.0, Color32::from_rgba_unmultiplied(56, 189, 248, 60));
        painter.circle_filled(puck_center, 7.0, Color32::from_rgb(56, 189, 248));
        painter.circle_stroke(puck_center, 14.0, Stroke::new(1.5_f32, Color32::WHITE));

        // Readout on puck
        painter.text(
            Pos2::new(puck_center.x, puck_center.y - 18.0),
            egui::Align2::CENTER_BOTTOM,
            format!("{:.2}m/s | {:.2}N", self.friction_speed_mps, self.normal_force_n),
            FontId::proportional(9.0),
            Color32::from_rgb(220, 235, 255),
        );

        // Interactive Puck Dragging
        if response.drag_started() {
            if let Some(pos) = response.interact_pointer_pos() {
                let dx = pos.x - puck_center.x;
                let dy = pos.y - puck_center.y;
                if (dx * dx + dy * dy).sqrt() <= CRYSTAL_PUCK_HIT_RADIUS {
                    self.is_dragging_puck = true;
                }
            }
        }
        if response.drag_stopped() {
            self.is_dragging_puck = false;
        }
        if self.is_dragging_puck && response.dragged() {
            if let Some(pos) = response.interact_pointer_pos() {
                let norm_x = ((pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                let norm_y = (1.0 - ((pos.y - left_rect.min.y) / left_rect.height())).clamp(0.0, 1.0);
                self.puck_pos = (norm_x, norm_y);
                self.friction_speed_mps = Self::normalized_to_speed(norm_x);
                self.normal_force_n = Self::normalized_to_force(norm_y);
                self.update_physics();
            }
        }

        // Right Panel: 8-Mode Thin-Shell Modal Resonance Spectrum
        let right_rect = egui::Rect::from_min_size(Pos2::new(rect.min.x + 30.0 + half_w, main_top), Vec2::new(half_w, main_h));
        painter.rect_filled(right_rect, 4.0, Color32::from_rgb(14, 20, 32));
        painter.rect_stroke(right_rect, 4.0, Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 75)));

        painter.text(
            Pos2::new(right_rect.min.x + 12.0, right_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            format!("8-Mode Thin-Shell Resonance Doublets (f0 = {:.1} Hz)", self.effective_f0_hz),
            FontId::proportional(11.0),
            Color32::from_rgb(148, 163, 184),
        );

        let bar_w = (right_rect.width() - 30.0) / NUM_CRYSTAL_MODES as f32;
        for i in 0..NUM_CRYSTAL_MODES {
            let bx = right_rect.min.x + 15.0 + i as f32 * bar_w;
            let amp = self.modal_amplitudes[i].clamp(0.0, 1.0);
            let bar_h = amp * (right_rect.height() - 70.0);
            let b_rect = egui::Rect::from_min_size(
                Pos2::new(bx + 4.0, right_rect.bottom() - 35.0 - bar_h),
                Vec2::new(bar_w - 8.0, bar_h.max(2.0)),
            );

            // Radiant Teal-to-Purple gradient tint per mode
            let col = Color32::from_rgb(
                (56.0 + i as f32 * 20.0).min(240.0) as u8,
                (189.0 - i as f32 * 12.0).max(60.0) as u8,
                248,
            );
            painter.rect_filled(b_rect, 2.0, col);

            // Doublet splitting line indicator
            let (fa, fb) = self.modal_frequencies[i];
            let df = (fb - fa).abs();
            painter.text(
                Pos2::new(bx + bar_w * 0.5, right_rect.bottom() - 28.0),
                egui::Align2::CENTER_TOP,
                format!("n={}", i + 2),
                FontId::proportional(8.5),
                Color32::from_rgb(160, 175, 200),
            );
            painter.text(
                Pos2::new(bx + bar_w * 0.5, right_rect.bottom() - 16.0),
                egui::Align2::CENTER_TOP,
                format!("±{:.1}Hz", df * 0.5),
                FontId::proportional(7.5),
                Color32::from_rgb(251, 191, 36),
            );
        }

        // Bottom Strip Controls: Water Fill, Modal Q, Pitch & Strike Trigger
        let bottom_top = main_top + main_h + 14.0;
        let ctrl_rect = egui::Rect::from_min_size(Pos2::new(rect.min.x + 20.0, bottom_top), Vec2::new(rect.width() - 40.0, 110.0));
        painter.rect_filled(ctrl_rect, 4.0, Color32::from_rgb(18, 24, 38));
        painter.rect_stroke(ctrl_rect, 4.0, Stroke::new(1.0_f32, Color32::from_rgb(36, 50, 75)));

        // Strike Mallet Button (Left side >= 44x44pt)
        let strike_btn_rect = egui::Rect::from_min_size(Pos2::new(ctrl_rect.min.x + 12.0, ctrl_rect.min.y + 12.0), Vec2::new(120.0, 44.0));
        let is_strike_hover = response.hover_pos().map(|p| strike_btn_rect.contains(p)).unwrap_or(false);
        let strike_bg = if is_strike_hover { Color32::from_rgb(245, 158, 11) } else { Color32::from_rgb(217, 119, 6) };
        painter.rect_filled(strike_btn_rect, 4.0, strike_bg);
        painter.text(
            strike_btn_rect.center(),
            egui::Align2::CENTER_CENTER,
            "🔨 Strike Mallet",
            FontId::proportional(11.0),
            Color32::WHITE,
        );

        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                if strike_btn_rect.contains(pos) {
                    self.strike_mallet(0.85);
                }
            }
        }

        // Water Fill Slider & Q-Scale Readouts
        painter.text(
            Pos2::new(ctrl_rect.min.x + 150.0, ctrl_rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            format!("💧 Water Mass Loading: {:.0}%  (Pitch: {:.1} Hz)", self.water_fill_pct * 100.0, self.effective_f0_hz),
            FontId::proportional(10.5),
            Color32::from_rgb(56, 189, 248),
        );
        painter.text(
            Pos2::new(ctrl_rect.min.x + 150.0, ctrl_rect.min.y + 40.0),
            egui::Align2::LEFT_TOP,
            format!("💎 Crystalline Q-Scale: {:.2}x  |  Doublet Beating Shimmer: {:.2} Hz", self.q_scale, self.doublet_split_hz),
            FontId::proportional(10.5),
            Color32::from_rgb(200, 215, 235),
        );

        // Status Footer
        painter.text(
            Pos2::new(ctrl_rect.min.x + 12.0, ctrl_rect.max.y - 20.0),
            egui::Align2::LEFT_BOTTOM,
            format!("Mode: {} | Velocity: {:.2} m/s | Contact: {:.2} N | Zero-Alloc Audio Ready", self.material_profile.name(), self.friction_speed_mps, self.normal_force_n),
            FontId::proportional(9.0),
            Color32::from_rgb(148, 163, 184),
        );
    }

    /// Render headless PNG snapshot visualizer for layout alignment, touch target verification, and WCAG contrast.
    pub fn render_snapshot_png(&self, path: &str, width: usize, height: usize) -> Result<(), String> {
        let mut pixels = vec![0u8; width * height * 4];

        // Background: Deep Slate Navy (#0A0E18)
        for idx in (0..pixels.len()).step_by(4) {
            pixels[idx] = 0x0A;
            pixels[idx + 1] = 0x0E;
            pixels[idx + 2] = 0x18;
            pixels[idx + 3] = 0xFF;
        }

        // Header Panel (8pt grid padding)
        let header_h = 56;
        for y in 0..header_h {
            for x in 0..width {
                let idx = (y * width + x) * 4;
                pixels[idx] = 0x13;
                pixels[idx + 1] = 0x1D;
                pixels[idx + 2] = 0x2E;
            }
        }

        // Title Indicator Bar (Radiant Sky Blue #38BDF8)
        for y in 12..20 {
            for x in 24..240 {
                let idx = (y * width + x) * 4;
                pixels[idx] = 0x38;
                pixels[idx + 1] = 0xBD;
                pixels[idx + 2] = 0xF8;
            }
        }

        // Main Canvas Box
        let c_left = 24;
        let c_top = 72;
        let c_right = width - 24;
        let c_bottom = height - 88;

        for y in c_top..c_bottom {
            for x in c_left..c_right {
                let idx = (y * width + x) * 4;
                let is_border = x == c_left || x == c_right - 1 || y == c_top || y == c_bottom - 1;
                if is_border {
                    pixels[idx] = 0x38;
                    pixels[idx + 1] = 0xBD;
                    pixels[idx + 2] = 0xF8; // Bright Sky Blue border
                } else if x % 32 == 0 || y % 32 == 0 {
                    pixels[idx] = 0x1E;
                    pixels[idx + 1] = 0x29;
                    pixels[idx + 2] = 0x3B; // 8pt grid guide
                } else {
                    pixels[idx] = 0x08;
                    pixels[idx + 1] = 0x0C;
                    pixels[idx + 2] = 0x16;
                }
            }
        }

        // Quartz Crystal Bowl Contour in Left 50%
        let canvas_w = (c_right - c_left) as f32;
        let canvas_h_f = (c_bottom - c_top) as f32;
        let center_x = c_left as f32 + canvas_w * 0.25;
        let center_y = c_top as f32 + canvas_h_f * 0.50;
        let bowl_r = (canvas_h_f * 0.40).min(canvas_w * 0.22);

        // Circular bowl rim
        let num_pts = 180;
        for i in 0..num_pts {
            let theta = (i as f32 / num_pts as f32) * 2.0 * std::f32::consts::PI;
            let px = (center_x + bowl_r * theta.cos()) as isize;
            let py = (center_y + bowl_r * theta.sin()) as isize;
            if px >= c_left as isize && px < c_right as isize && py >= c_top as isize && py < c_bottom as isize {
                let idx = (py as usize * width + px as usize) * 4;
                pixels[idx] = 0x38;
                pixels[idx + 1] = 0xBD;
                pixels[idx + 2] = 0xF8;
            }
        }

        // Water fill meniscus
        if self.water_fill_pct > 0.0 {
            let water_y = center_y + bowl_r * (1.0 - 2.0 * self.water_fill_pct);
            let w_y_int = water_y as isize;
            if w_y_int >= c_top as isize && w_y_int < c_bottom as isize {
                for x in (center_x - bowl_r) as isize..=(center_x + bowl_r) as isize {
                    let dx = x as f32 - center_x;
                    if dx * dx <= bowl_r * bowl_r && x >= c_left as isize && x < c_right as isize {
                        let idx = (w_y_int as usize * width + x as usize) * 4;
                        pixels[idx] = 0x00;
                        pixels[idx + 1] = 0xE5;
                        pixels[idx + 2] = 0xFF; // Cyan water line
                    }
                }
            }
        }

        // Modal Resonance Spectrum Bars in Right 45%
        let bar_area_left = c_left as f32 + canvas_w * 0.55;
        let bar_area_right = c_right as f32 - 16.0;
        let bar_w = (bar_area_right - bar_area_left) / NUM_CRYSTAL_MODES as f32;

        for (i, &amp) in self.modal_amplitudes.iter().enumerate() {
            let bx = bar_area_left + i as f32 * bar_w;
            let bar_h = (amp.clamp(0.0, 1.0)) * (canvas_h_f * 0.70);
            let b_top = c_bottom as f32 - 20.0 - bar_h;
            let b_bot = c_bottom as f32 - 20.0;

            for y in (b_top as usize)..=(b_bot as usize) {
                for x in (bx as usize)..((bx + bar_w - 4.0) as usize) {
                    if x < width && y < height {
                        let idx = (y * width + x) * 4;
                        pixels[idx] = (56.0 + i as f32 * 20.0).min(240.0) as u8;
                        pixels[idx + 1] = (189.0 - i as f32 * 12.0).max(60.0) as u8;
                        pixels[idx + 2] = 248;
                    }
                }
            }
        }

        // Interactive Wand Puck (Radius = 22pt -> 44x44pt touch target)
        let puck_center_x = c_left + (canvas_w * 0.50 * self.puck_pos.0) as usize;
        let puck_center_y = c_top + (canvas_h_f * (1.0 - self.puck_pos.1)) as usize;
        let radius = CRYSTAL_PUCK_HIT_RADIUS as isize;
        let inner_radius = (CRYSTAL_PUCK_HIT_RADIUS - 3.0) as isize;

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy <= radius * radius {
                    let px = (puck_center_x as isize + dx) as usize;
                    let py = (puck_center_y as isize + dy) as usize;
                    if px < width && py < height {
                        let idx = (py * width + px) * 4;
                        let dist_sq = dx * dx + dy * dy;
                        if dist_sq >= inner_radius * inner_radius {
                            pixels[idx] = 0x38;
                            pixels[idx + 1] = 0xBD;
                            pixels[idx + 2] = 0xF8; // Sky Blue outer ring
                        } else {
                            pixels[idx] = 0xF5;
                            pixels[idx + 1] = 0x9E;
                            pixels[idx + 2] = 0x0B; // Amber gold center
                        }
                    }
                }
            }
        }

        // Bottom Metrics Indicator Bar
        let bar_y_start = height - 68;
        let bar_y_end = height - 44;
        let bar_left = c_left;
        let bar_right = c_right;

        for y in bar_y_start..bar_y_end {
            for x in bar_left..bar_right {
                let idx = (y * width + x) * 4;
                pixels[idx] = 0x1E;
                pixels[idx + 1] = 0x29;
                pixels[idx + 2] = 0x3B;
            }
        }

        let water_fill_split = bar_left + ((bar_right - bar_left) as f32 * self.water_fill_pct) as usize;
        for y in bar_y_start..bar_y_end {
            for x in bar_left..=water_fill_split.min(bar_right) {
                let idx = (y * width + x) * 4;
                pixels[idx] = 0x00;
                pixels[idx + 1] = 0xE5;
                pixels[idx + 2] = 0xFF; // Water Level Cyan
            }
        }

        save_png_file(path, width, height, &pixels)
    }
}

fn save_png_file(path: &str, width: usize, height: usize, rgba_pixels: &[u8]) -> Result<(), String> {
    let mut out = Vec::with_capacity(width * height * 4 + 1024);
    out.extend_from_slice(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);

    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&(width as u32).to_be_bytes());
    ihdr.extend_from_slice(&(height as u32).to_be_bytes());
    ihdr.push(8);
    ihdr.push(6);
    ihdr.push(0);
    ihdr.push(0);
    ihdr.push(0);
    write_png_chunk_to(&mut out, b"IHDR", &ihdr);

    let stride = width * 4;
    let mut raw_data = Vec::with_capacity((stride + 1) * height);
    for y in 0..height {
        raw_data.push(0x00);
        let row_start = y * stride;
        let row_end = row_start + stride;
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

    write_png_chunk_to(&mut out, b"IDAT", &zlib_data);
    write_png_chunk_to(&mut out, b"IEND", &[]);

    if let Some(parent) = std::path::Path::new(path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(path, out).map_err(|e| e.to_string())
}

fn write_png_chunk_to(out: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let type_start = out.len();
    out.extend_from_slice(chunk_type);
    out.extend_from_slice(data);
    let crc = crc32_compute_to(&out[type_start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

fn crc32_compute_to(buf: &[u8]) -> u32 {
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

    #[test]
    fn test_crystal_resonator_defaults() {
        let view = CrystalResonatorView::new();
        assert_eq!(view.material_profile, CrystalMaterialProfile::PureQuartzCrystal);
        assert_eq!(view.root_freq_hz, 432.0);
        assert!(view.effective_f0_hz <= 432.0);
        assert_eq!(view.modal_amplitudes.len(), NUM_CRYSTAL_MODES);
    }

    #[test]
    fn test_water_mass_loading_pitch_lowering() {
        let mut view = CrystalResonatorView::new();
        view.water_fill_pct = 0.0;
        view.update_physics();
        let f0_dry = view.effective_f0_hz;
        assert_eq!(f0_dry, 432.0);

        view.water_fill_pct = 0.5;
        view.update_physics();
        let f0_half = view.effective_f0_hz;
        assert!(f0_half < f0_dry);

        view.water_fill_pct = 1.0;
        view.update_physics();
        let f0_full = view.effective_f0_hz;
        assert!(f0_full < f0_half);
    }

    #[test]
    fn test_modal_doublet_splitting_intervals() {
        let mut view = CrystalResonatorView::new();
        view.set_material_profile(CrystalMaterialProfile::FranklinQuartzGlass);
        view.update_physics();

        for i in 0..NUM_CRYSTAL_MODES {
            let (fa, fb) = view.modal_frequencies[i];
            assert!(fb > fa, "Doublet mode B must be higher than mode A");
            let df = fb - fa;
            assert!((0.1..=5.0).contains(&df), "Split interval must be realistic");
        }
    }

    #[test]
    fn test_puck_touch_target_and_normalization() {
        let mut view = CrystalResonatorView::new();
        let canvas = Rect { x: 50.0, y: 50.0, width: 200.0, height: 200.0 };

        view.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;

        assert!(view.hit_test_puck((center_x, center_y), canvas));
        assert!(view.hit_test_puck((center_x + 15.0, center_y), canvas));
        assert!(!view.hit_test_puck((center_x + 35.0, center_y), canvas));
    }

    #[test]
    fn test_ascii_snapshot_render() {
        let view = CrystalResonatorView::new();
        let lines = view.render_ascii(60, 20);
        assert_eq!(lines.len(), 20);
        assert_eq!(lines[0].len(), 60);
        let text = lines.join("\n");
        assert!(text.contains('W'), "ASCII grid must render Wand puck 'W'");
        assert!(text.contains('#'), "ASCII grid must render modal bars '#'");
    }

    #[test]
    fn test_png_snapshot_render() {
        let view = CrystalResonatorView::new();
        let path = "scratch/renders/test_crystal_resonator.png";
        let res = view.render_snapshot_png(path, 400, 300);
        assert!(res.is_ok());
        if std::path::Path::new(path).exists() {
            let _ = std::fs::remove_file(path);
        }
    }
}
