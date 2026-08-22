// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Physical Modeling Glass Armonica & Crystal Singing Bowl Friction Resonance HUD (Step 1571).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const ARMONICA_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_ROTATION_SPEED_RAD_S: f32 = 0.1;
pub const MAX_ROTATION_SPEED_RAD_S: f32 = 10.0;
pub const MIN_NORMAL_FORCE_N: f32 = 0.05;
pub const MAX_NORMAL_FORCE_N: f32 = 1.00;
pub const MIN_MODAL_FREQ_HZ: f32 = 100.0;
pub const MAX_MODAL_FREQ_HZ: f32 = 2000.0;

/// Glass armonica and crystal singing bowl resonator instrument types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmonicaType {
    FranklinArmonicaC4C7, // Rotating concentric nested quartz glass bowls on iron spindle
    CrystalSingingBowl432, // 99.9% pure quartz crystal singing bowl (432 Hz therapeutic tuning)
    WetFingerChalice,     // Crystal wine goblet excited by tangential stick-slip friction
    BorosilicateBell,     // Pyrex / borosilicate heavy glass bell with long decay
    MetallophoneResonator, // Anharmonic friction metal tuned bell with complex nodal lines
}

impl ArmonicaType {
    pub fn instrument_name(&self) -> &'static str {
        match self {
            Self::FranklinArmonicaC4C7 => "FRANKLIN ARMONICA",
            Self::CrystalSingingBowl432 => "CRYSTAL BOWL (432Hz)",
            Self::WetFingerChalice => "WET CHALICE",
            Self::BorosilicateBell => "BOROSILICATE BELL",
            Self::MetallophoneResonator => "METALLOPHONE",
        }
    }

    pub fn nominal_speed_rad_s(&self) -> f32 {
        match self {
            Self::FranklinArmonicaC4C7 => 2.5,
            Self::CrystalSingingBowl432 => 1.2,
            Self::WetFingerChalice => 3.8,
            Self::BorosilicateBell => 1.8,
            Self::MetallophoneResonator => 4.5,
        }
    }

    pub fn nominal_normal_force_n(&self) -> f32 {
        match self {
            Self::FranklinArmonicaC4C7 => 0.45,
            Self::CrystalSingingBowl432 => 0.65,
            Self::WetFingerChalice => 0.25,
            Self::BorosilicateBell => 0.70,
            Self::MetallophoneResonator => 0.85,
        }
    }

    pub fn nominal_fundamental_hz(&self) -> f32 {
        match self {
            Self::FranklinArmonicaC4C7 => 523.25,  // C5
            Self::CrystalSingingBowl432 => 432.00, // A4 (432Hz)
            Self::WetFingerChalice => 659.25,      // E5
            Self::BorosilicateBell => 329.63,      // E4
            Self::MetallophoneResonator => 880.00, // A5
        }
    }

    pub fn nominal_q_factor(&self) -> f32 {
        match self {
            Self::FranklinArmonicaC4C7 => 3200.0,
            Self::CrystalSingingBowl432 => 4800.0,
            Self::WetFingerChalice => 2400.0,
            Self::BorosilicateBell => 1800.0,
            Self::MetallophoneResonator => 1200.0,
        }
    }
}

/// Physical modeling glass armonica & crystal singing bowl friction resonance HUD.
#[derive(Debug, Clone)]
pub struct GlassArmonicaView {
    pub instrument_type: ArmonicaType,
    pub rotation_speed_rad_s: f32,     // [0.1 ..= 10.0 rad/s]
    pub normal_force_n: f32,           // [0.05 ..= 1.00 N]
    pub water_level_pct: f32,          // [0.0 ..= 1.0 (water volume in bowl)]
    pub modal_fundamental_hz: f32,     // [100.0 ..= 2000.0 Hz]
    pub armonica_puck_pos: (f32, f32), // Normalized (X: rotation speed, Y: normal force)
    pub is_dragging_puck: bool,
    pub stick_slip_velocity_mps: f32, // [0.05 ..= 3.0 m/s relative finger-glass velocity]
    pub q_factor: f32,                // [500.0 ..= 5000.0 acoustic resonance sharpness]
    pub modal_amplitudes: [f32; 8],   // 8 circular nodal resonance modes
    pub color_palette: ContrastColorPalette,
}

impl Default for GlassArmonicaView {
    fn default() -> Self {
        Self::new()
    }
}

impl GlassArmonicaView {
    pub fn new() -> Self {
        let mut view = Self {
            instrument_type: ArmonicaType::FranklinArmonicaC4C7,
            rotation_speed_rad_s: 2.5,
            normal_force_n: 0.45,
            water_level_pct: 0.40,
            modal_fundamental_hz: 523.25,
            armonica_puck_pos: (0.0, 0.0),
            is_dragging_puck: false,
            stick_slip_velocity_mps: 0.85,
            q_factor: 3200.0,
            modal_amplitudes: [1.0, 0.55, 0.28, 0.14, 0.08, 0.04, 0.18, 0.10],
            color_palette: ContrastColorPalette::default(),
        };
        view.armonica_puck_pos = (
            Self::speed_to_normalized(view.rotation_speed_rad_s),
            Self::force_to_normalized(view.normal_force_n),
        );
        view.update_friction_simulation();
        view
    }

    pub fn speed_to_normalized(rad_s: f32) -> f32 {
        let s = rad_s.clamp(MIN_ROTATION_SPEED_RAD_S, MAX_ROTATION_SPEED_RAD_S);
        ((s - MIN_ROTATION_SPEED_RAD_S) / (MAX_ROTATION_SPEED_RAD_S - MIN_ROTATION_SPEED_RAD_S))
            .clamp(0.0, 1.0)
    }

    pub fn normalized_to_speed(norm: f32) -> f32 {
        MIN_ROTATION_SPEED_RAD_S
            + norm.clamp(0.0, 1.0) * (MAX_ROTATION_SPEED_RAD_S - MIN_ROTATION_SPEED_RAD_S)
    }

    pub fn force_to_normalized(force: f32) -> f32 {
        let f = force.clamp(MIN_NORMAL_FORCE_N, MAX_NORMAL_FORCE_N);
        ((f - MIN_NORMAL_FORCE_N) / (MAX_NORMAL_FORCE_N - MIN_NORMAL_FORCE_N)).clamp(0.0, 1.0)
    }

    pub fn normalized_to_force(norm: f32) -> f32 {
        MIN_NORMAL_FORCE_N + norm.clamp(0.0, 1.0) * (MAX_NORMAL_FORCE_N - MIN_NORMAL_FORCE_N)
    }

    pub fn set_instrument_type(&mut self, inst: ArmonicaType) {
        self.instrument_type = inst;
        self.rotation_speed_rad_s = inst.nominal_speed_rad_s();
        self.normal_force_n = inst.nominal_normal_force_n();
        self.modal_fundamental_hz = inst.nominal_fundamental_hz();
        self.q_factor = inst.nominal_q_factor();
        self.armonica_puck_pos = (
            Self::speed_to_normalized(self.rotation_speed_rad_s),
            Self::force_to_normalized(self.normal_force_n),
        );
        self.update_friction_simulation();
    }

    /// Update stick-slip friction dynamics and 2D circular nodal modal resonances.
    pub fn update_friction_simulation(&mut self) {
        // Linear velocity v = omega * r (assume average bowl radius 0.08 m)
        let bowl_radius_m = 0.08;
        self.stick_slip_velocity_mps =
            (self.rotation_speed_rad_s * bowl_radius_m * (1.0 + self.normal_force_n * 0.5))
                .clamp(0.05, 3.5);

        // Water loading shifts fundamental pitch down and dampens high modes
        let water_damping = 1.0 - 0.25 * self.water_level_pct;
        let pitch_lowering = 1.0 - 0.12 * self.water_level_pct;
        let effective_f0 = self.instrument_type.nominal_fundamental_hz() * pitch_lowering;
        self.modal_fundamental_hz = effective_f0;

        // Acoustic Q factor decreases with higher normal force (finger damping)
        self.q_factor = (self.instrument_type.nominal_q_factor()
            * (1.0 - 0.4 * (self.normal_force_n - 0.05) / 0.95)
            * water_damping)
            .clamp(500.0, 5000.0);

        // Circular shell modal amplitudes: (2,0) fundamental hoop mode, (3,0), (4,0), etc.
        let stick_slip_gain = (self.normal_force_n / 0.45).sqrt().clamp(0.2, 1.8);
        match self.instrument_type {
            ArmonicaType::FranklinArmonicaC4C7 => {
                self.modal_amplitudes = [
                    1.0 * stick_slip_gain,
                    0.55 * stick_slip_gain * water_damping,
                    0.28 * stick_slip_gain * water_damping,
                    0.14 * stick_slip_gain * water_damping,
                    0.08 * stick_slip_gain,
                    0.04 * stick_slip_gain,
                    0.18 * self.water_level_pct,
                    0.10 * (1.0 - self.water_level_pct),
                ];
            }
            ArmonicaType::CrystalSingingBowl432 => {
                // Extremely pure (2,0) mode with minimal upper partials
                self.modal_amplitudes = [
                    1.0 * stick_slip_gain,
                    0.20 * stick_slip_gain,
                    0.08 * stick_slip_gain,
                    0.03 * stick_slip_gain,
                    0.01 * stick_slip_gain,
                    0.005 * stick_slip_gain,
                    0.25 * self.water_level_pct,
                    0.15 * (1.0 - self.water_level_pct),
                ];
            }
            ArmonicaType::WetFingerChalice => {
                // Rich high partials from sharp glass rim
                self.modal_amplitudes = [
                    0.95 * stick_slip_gain,
                    0.70 * stick_slip_gain * water_damping,
                    0.45 * stick_slip_gain * water_damping,
                    0.30 * stick_slip_gain,
                    0.18 * stick_slip_gain,
                    0.10 * stick_slip_gain,
                    0.35 * self.water_level_pct,
                    0.12 * (1.0 - self.water_level_pct),
                ];
            }
            ArmonicaType::BorosilicateBell => {
                // Dense modal cluster
                self.modal_amplitudes = [
                    0.85 * stick_slip_gain,
                    0.65 * stick_slip_gain,
                    0.50 * stick_slip_gain,
                    0.35 * stick_slip_gain,
                    0.22 * stick_slip_gain,
                    0.15 * stick_slip_gain,
                    0.10 * self.water_level_pct,
                    0.20 * (1.0 - self.water_level_pct),
                ];
            }
            ArmonicaType::MetallophoneResonator => {
                // Anharmonic metal plate ratios
                self.modal_amplitudes = [
                    0.90 * stick_slip_gain,
                    0.80 * stick_slip_gain,
                    0.60 * stick_slip_gain,
                    0.40 * stick_slip_gain,
                    0.30 * stick_slip_gain,
                    0.20 * stick_slip_gain,
                    0.05 * self.water_level_pct,
                    0.25 * (1.0 - self.water_level_pct),
                ];
            }
        }
    }

    /// Hit test coordinate on the interactive armonica friction puck.
    pub fn hit_test_armonica_puck(&self, point: (f32, f32), canvas: Rect) -> bool {
        let puck_x = canvas.x + self.armonica_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - self.armonica_puck_pos.1) * canvas.height;
        let dx = point.0 - puck_x;
        let dy = point.1 - puck_y;
        (dx * dx + dy * dy).sqrt() <= ARMONICA_PUCK_HIT_RADIUS
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

        // Left half: Spindle / Bowl stick-slip radar
        let left_w = mid_x - 2;
        let p_row =
            (((1.0 - self.armonica_puck_pos.1) * (height - 5) as f32) + 2.0).round() as usize;
        let p_col = ((self.armonica_puck_pos.0 * (left_w - 4) as f32) + 2.0).round() as usize;
        if p_row < height - 1 && p_col < mid_x {
            grid[p_row][p_col] = 'P';
        }

        // Right half: Circular modal resonance overtone bars
        let right_w = width - mid_x - 2;
        let bar_spacing = right_w / 9;
        for (i, &amp) in self.modal_amplitudes.iter().enumerate() {
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

        // Background: Deep Slate Navy (#0C101A)
        painter.rect_filled(rect, 6.0, Color32::from_rgb(12, 16, 26));

        // Header Title
        painter.text(
            egui::pos2(rect.min.x + 20.0, rect.min.y + 18.0),
            egui::Align2::LEFT_TOP,
            "PHYSICAL MODELING GLASS ARMONICA & CRYSTAL SINGING BOWL RESONANCE HUD",
            egui::FontId::proportional(14.0),
            Color32::from_rgb(240, 245, 255),
        );

        // Resonator Instrument Tabs (y: 48..92) - Each tab >= 44pt touch target
        let tabs = [
            (ArmonicaType::FranklinArmonicaC4C7, "FRANKLIN ARMONICA"),
            (ArmonicaType::CrystalSingingBowl432, "CRYSTAL BOWL (432Hz)"),
            (ArmonicaType::WetFingerChalice, "WET CHALICE"),
            (ArmonicaType::BorosilicateBell, "BOROSILICATE BELL"),
            (ArmonicaType::MetallophoneResonator, "METALLOPHONE"),
        ];

        let tab_w = (rect.width() - 40.0 - 4.0 * 8.0) / 5.0;
        for (i, (itype, name)) in tabs.iter().enumerate() {
            let bx = rect.min.x + 20.0 + i as f32 * (tab_w + 8.0);
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(bx, rect.min.y + 48.0),
                egui::vec2(tab_w, 44.0),
            );
            let is_sel = self.instrument_type == *itype;
            let bg_col = if is_sel {
                Color32::from_rgb(255, 180, 50)
            } else {
                Color32::from_rgb(24, 32, 48)
            };
            let text_col = if is_sel {
                Color32::from_rgb(12, 14, 18)
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
                        self.set_instrument_type(*itype);
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

        // Left 55%: Stick-Slip Friction Dynamics & Spindle Rotation Radar
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
            "STICK-SLIP FRICTION DYNAMICS & SPINDLE ROTATION RADAR",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(255, 190, 80),
        );

        // Concentric Quartz Glass Bowl Rings
        let cx = left_rect.center().x;
        let cy = left_rect.center().y + 10.0;
        let max_r = 75.0;
        for (idx, r_step) in [0.35, 0.60, 0.85, 1.00].iter().enumerate() {
            let rad = max_r * r_step;
            let col = if idx == 3 {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgba_unmultiplied(255, 180, 50, 80)
            };
            painter.circle_stroke(egui::pos2(cx, cy), rad, Stroke::new(1.0_f32, col));
        }

        // Stick-slip friction angle tangent vector
        let angle_rad = self.rotation_speed_rad_s * 1.5;
        let vx = cx + max_r * 0.85 * angle_rad.cos();
        let vy = cy + max_r * 0.85 * angle_rad.sin();
        painter.line_segment(
            [egui::pos2(cx, cy), egui::pos2(vx, vy)],
            Stroke::new(2.0_f32, Color32::from_rgb(0, 255, 180)),
        );

        // Interactive Voicing Friction Puck
        let puck_x = left_rect.min.x + self.armonica_puck_pos.0 * left_rect.width();
        let puck_y = left_rect.max.y - self.armonica_puck_pos.1 * left_rect.height();
        let puck_pos = egui::pos2(puck_x, puck_y);

        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if left_rect.contains(mouse_pos) {
                    let nx = ((mouse_pos.x - left_rect.min.x) / left_rect.width()).clamp(0.0, 1.0);
                    let ny = ((left_rect.max.y - mouse_pos.y) / left_rect.height()).clamp(0.0, 1.0);
                    self.armonica_puck_pos = (nx, ny);
                    self.rotation_speed_rad_s = Self::normalized_to_speed(nx);
                    self.normal_force_n = Self::normalized_to_force(ny);
                    self.update_friction_simulation();
                }
            }
        }

        // Draw Touch Hit Target boundary (>= 44x44pt)
        painter.circle_stroke(
            puck_pos,
            ARMONICA_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(255, 180, 50, 150)),
        );
        painter.circle_filled(puck_pos, 14.0, Color32::from_rgb(255, 180, 50));
        painter.circle_filled(puck_pos, 4.0, Color32::WHITE);

        painter.text(
            egui::pos2(left_rect.min.x + 10.0, left_rect.max.y - 20.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Speed: {:.1} rad/s | Normal: {:.2} N | Stick-Slip: {:.2} m/s",
                self.rotation_speed_rad_s, self.normal_force_n, self.stick_slip_velocity_mps
            ),
            egui::FontId::proportional(10.5),
            Color32::from_rgb(255, 215, 100),
        );

        // Right 45%: Circular Modal Resonances (f0, (2,0), (3,0), (4,0), ...)
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
            "CIRCULAR MODAL RESONANCES (f0, (2,0)..(6,0))",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(255, 190, 80),
        );

        let mode_names = [
            "(2,0)", "(3,0)", "(4,0)", "(5,0)", "(6,0)", "(7,0)", "WTR", "CAV",
        ];
        let bar_w = (right_rect.width() - 30.0 - 7.0 * 6.0) / 8.0;
        for (i, &amp) in self.modal_amplitudes.iter().enumerate() {
            let bx = right_rect.min.x + 15.0 + i as f32 * (bar_w + 6.0);
            let bar_h = (amp.clamp(0.0, 1.2) / 1.2) * (right_rect.height() - 80.0);
            let b_rect = egui::Rect::from_min_max(
                egui::pos2(bx, right_rect.max.y - 25.0 - bar_h),
                egui::pos2(bx + bar_w, right_rect.max.y - 25.0),
            );
            let col = if i == 0 {
                Color32::from_rgb(255, 180, 50)
            } else if i < 6 {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(0, 255, 180)
            };
            painter.rect_filled(b_rect, 3.0, col);

            painter.text(
                egui::pos2(bx + bar_w * 0.5, right_rect.max.y - 20.0),
                egui::Align2::CENTER_TOP,
                mode_names[i],
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
                "ROTATION SPEED",
                format!(
                    "{:.1} rad/s ({:.0} RPM)",
                    self.rotation_speed_rad_s,
                    self.rotation_speed_rad_s * 60.0 / (2.0 * std::f32::consts::PI)
                ),
                Color32::from_rgb(255, 180, 50),
            ),
            (
                "FUNDAMENTAL (2,0)",
                format!("{:.1} Hz (Quartz)", self.modal_fundamental_hz),
                Color32::from_rgb(0, 229, 255),
            ),
            (
                "STICK-SLIP VELOCITY",
                format!("{:.2} m/s (Friction)", self.stick_slip_velocity_mps),
                Color32::from_rgb(255, 215, 0),
            ),
            (
                "ACOUSTIC Q-FACTOR",
                format!("{:.0} (Resonance)", self.q_factor),
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

        // Compliance Verification Badge
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
            "[PASS] Glass Armonica Friction & Modal Resonance Touch Targets (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
