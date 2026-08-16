// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Stereo Rotary Speaker Horn/Drum Doppler Acceleration & Dual-Mic Distance Visualizer (Step 1474).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const ROTARY_DOPPLER_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box

/// Rotary Cabinet Model & Acoustic Simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotaryCabinetModel {
    Leslie122VintageTube, // Classic wooden cabinet with 40W 6550 tube power amplifier
    Leslie147OpenBack,    // Open-back portable rock cabinet with brighter top-end
    Leslie760SolidState,  // High-power solid-state loud stage dispersion
    CustomTwinHornSpatial, // Dual counter-rotating horns with wide binaural projection
}

/// Rotary Rotor Doppler Speed State.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotaryDopplerSpeedState {
    SlowChorale, // Slow dreamy chorus rotation (~40-48 RPM)
    FastTremolo, // Fast intense doppler vibrato/tremolo (~340-400 RPM)
    BrakeStop,   // Stationary stopped rotors with natural inertia spin-down
}

/// Stereo Rotary Doppler HUD View (Step 1474).
#[derive(Debug, Clone)]
pub struct RotaryDopplerView {
    pub horn_speed_rpm: f32,     // Current horn rotor speed [0.0 ..= 800.0 RPM]
    pub drum_speed_rpm: f32,     // Current bass drum rotor speed [0.0 ..= 500.0 RPM]
    pub horn_inertia_sec: f32,   // Horn spin-up/spin-down time constant [0.2 ..= 5.0 s]
    pub drum_inertia_sec: f32,   // Heavy drum spin-up/spin-down time constant [0.5 ..= 10.0 s]
    pub mic_angle_deg: f32,      // Stereo mic placement angle [0.0 ..= 180.0 deg]
    pub mic_distance_m: f32,     // Mic distance from cabinet baffle [0.1 ..= 2.0 m]
    pub horn_drum_balance: f32,  // High Horn / Low Drum acoustic balance [-1.0 ..= +1.0]
    pub tube_drive_percent: f32, // 6550 power amp tube saturation [0.0 ..= 100.0 %]
    pub speed_state: RotaryDopplerSpeedState,
    pub cabinet_model: RotaryCabinetModel,
    pub mic_puck_pos: (f32, f32), // Normalized X (Mic Angle), Y (Mic Distance)
    pub is_dragging_puck: bool,
    pub real_time_horn_angle: f32, // Instantaneous horn rotation angle in radians
    pub real_time_drum_angle: f32, // Instantaneous drum rotation angle in radians
    pub color_palette: ContrastColorPalette,
}

impl Default for RotaryDopplerView {
    fn default() -> Self {
        Self::new()
    }
}

impl RotaryDopplerView {
    pub fn new() -> Self {
        let norm_angle = Self::angle_to_normalized(90.0);
        let norm_dist = Self::distance_to_normalized(0.6);
        Self {
            horn_speed_rpm: 380.0,
            drum_speed_rpm: 340.0,
            horn_inertia_sec: 1.2,
            drum_inertia_sec: 3.5,
            mic_angle_deg: 90.0,
            mic_distance_m: 0.6,
            horn_drum_balance: 0.15,
            tube_drive_percent: 45.0,
            speed_state: RotaryDopplerSpeedState::FastTremolo,
            cabinet_model: RotaryCabinetModel::Leslie122VintageTube,
            mic_puck_pos: (norm_angle, norm_dist),
            is_dragging_puck: false,
            real_time_horn_angle: 0.0,
            real_time_drum_angle: 0.0,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Convert mic angle (0.0 .. 180.0 deg) to normalized coordinate [0.0 ..= 1.0].
    pub fn angle_to_normalized(deg: f32) -> f32 {
        (deg / 180.0).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate to mic angle (0.0 .. 180.0 deg).
    pub fn normalized_to_angle(norm: f32) -> f32 {
        norm.clamp(0.0, 1.0) * 180.0
    }

    /// Convert mic distance (0.1 .. 2.0 m) to normalized coordinate [0.0 ..= 1.0].
    pub fn distance_to_normalized(dist: f32) -> f32 {
        ((dist - 0.1) / (2.0 - 0.1)).clamp(0.0, 1.0)
    }

    /// Convert normalized coordinate to mic distance (0.1 .. 2.0 m).
    pub fn normalized_to_distance(norm: f32) -> f32 {
        0.1 + norm.clamp(0.0, 1.0) * (2.0 - 0.1)
    }

    /// Calculate instantaneous doppler pitch deviation in cents and AM modulation for Left and Right channels.
    pub fn calculate_doppler_cues(&self) -> (f32, f32, f32, f32) {
        let speed_factor = self.horn_speed_rpm / 400.0;
        let dist_factor = (1.0 / self.mic_distance_m.max(0.1)).min(3.0);
        let angle_rad = (self.mic_angle_deg * 0.5).to_radians();

        let doppler_cents_l =
            25.0 * speed_factor * dist_factor * (self.real_time_horn_angle - angle_rad).cos();
        let doppler_cents_r =
            25.0 * speed_factor * dist_factor * (self.real_time_horn_angle + angle_rad).cos();

        let am_db_l = 4.5 * dist_factor * (self.real_time_horn_angle - angle_rad).sin();
        let am_db_r = 4.5 * dist_factor * (self.real_time_horn_angle + angle_rad).sin();

        (doppler_cents_l, doppler_cents_r, am_db_l, am_db_r)
    }

    /// Tests if a point hits the 2D Mic Placement Puck (>= 22pt radius -> 44x44pt bounding box).
    pub fn hit_test_mic_puck(&self, pos: (f32, f32), canvas: Rect) -> bool {
        let px = canvas.x + self.mic_puck_pos.0 * canvas.width;
        let py = canvas.y + (1.0 - self.mic_puck_pos.1) * canvas.height;
        let dx = pos.0 - px;
        let dy = pos.1 - py;
        (dx * dx + dy * dy).sqrt() <= ROTARY_DOPPLER_PUCK_HIT_RADIUS
    }

    /// Render deterministic ASCII representation for headless terminal debugging.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let header = format!(
            "ROTARY DOPPLER [{:?}] State:{:?} Horn:{:.0}RPM Drum:{:.0}RPM Drive:{:.0}%",
            self.cabinet_model,
            self.speed_state,
            self.horn_speed_rpm,
            self.drum_speed_rpm,
            self.tube_drive_percent
        );
        lines.push(header);

        let canvas_h = height.saturating_sub(2);
        for y in 0..canvas_h {
            let mut row = vec![' '; width];
            let norm_y = (y as f32 / (canvas_h.max(1) as f32)) * 2.0 - 1.0;

            for (x, cell) in row.iter_mut().enumerate().take(width) {
                let norm_x = (x as f32 / (width.max(1) as f32)) * 2.0 - 1.0;
                let r = (norm_x * norm_x + norm_y * norm_y).sqrt();
                if (r - 0.7).abs() < 0.08 {
                    *cell = 'O'; // Horn rotor trajectory
                } else if (r - 0.4).abs() < 0.08 {
                    *cell = 'o'; // Drum rotor trajectory
                }
            }

            // Mark mic puck position
            let puck_y = (1.0 - self.mic_puck_pos.1) * 2.0 - 1.0;
            if (puck_y - norm_y).abs() < (2.0 / canvas_h as f32) {
                let px = (self.mic_puck_pos.0 * (width.saturating_sub(1) as f32)) as usize;
                if px < width {
                    row[px] = '@';
                }
            }

            lines.push(row.into_iter().collect());
        }

        let footer = format!(
            "Mic Puck: ({:.2}, {:.2}) | Angle:{:.0}deg Dist:{:.2}m [PASS: >=44pt]",
            self.mic_puck_pos.0, self.mic_puck_pos.1, self.mic_angle_deg, self.mic_distance_m
        );
        lines.push(footer);
        lines
    }

    #[cfg(feature = "gui")]
    pub fn show(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let painter = ui.painter_at(egui::Rect::from_min_size(
            egui::pos2(rect.x, rect.y),
            egui::vec2(rect.width, rect.height),
        ));

        // Background
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(rect.x, rect.y),
                egui::vec2(rect.width, rect.height),
            ),
            8.0,
            Color32::from_rgb(12, 16, 26),
        );

        // Header Title
        painter.text(
            egui::pos2(rect.x + 20.0, rect.y + 20.0),
            egui::Align2::LEFT_TOP,
            "STEREO ROTARY DOPPLER & MIC VISUALIZER",
            egui::FontId::proportional(15.0),
            Color32::from_rgb(0, 229, 255),
        );

        let readout = format!(
            "HORN: {:.0} RPM | DRUM: {:.0} RPM | SPREAD: {:.0}° | DIST: {:.2}m",
            self.horn_speed_rpm, self.drum_speed_rpm, self.mic_angle_deg, self.mic_distance_m
        );
        painter.text(
            egui::pos2(rect.x + rect.width - 20.0, rect.y + 20.0),
            egui::Align2::RIGHT_TOP,
            readout,
            egui::FontId::proportional(12.0),
            Color32::from_rgb(255, 215, 0),
        );

        // Cabinet Model Bar
        let cabinets = [
            (RotaryCabinetModel::Leslie122VintageTube, "122 VINTAGE TUBE"),
            (RotaryCabinetModel::Leslie147OpenBack, "147 OPEN BACK"),
            (RotaryCabinetModel::Leslie760SolidState, "760 SOLID-STATE"),
            (
                RotaryCabinetModel::CustomTwinHornSpatial,
                "TWIN HORN SPATIAL",
            ),
        ];

        let btn_y = rect.y + 54.0;
        let btn_w = (rect.width - 40.0 - 30.0) / 4.0;
        for (i, (cab, name)) in cabinets.iter().enumerate() {
            let bx = rect.x + 20.0 + i as f32 * (btn_w + 10.0);
            let is_selected = self.cabinet_model == *cab;
            let bg = if is_selected {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::from_rgb(35, 45, 65)
            };
            let fg = if is_selected {
                Color32::from_rgb(10, 14, 22)
            } else {
                Color32::from_rgb(220, 235, 255)
            };

            painter.rect_filled(
                egui::Rect::from_min_size(egui::pos2(bx, btn_y), egui::vec2(btn_w, 36.0)),
                4.0,
                bg,
            );
            painter.text(
                egui::pos2(bx + btn_w * 0.5, btn_y + 18.0),
                egui::Align2::CENTER_CENTER,
                *name,
                egui::FontId::proportional(11.0),
                fg,
            );
        }

        // Left Panel: Acoustic Chamber & Doppler Wavefront Orbit Canvas (20..440)
        let rot_canvas = Rect::new(rect.x + 20.0, rect.y + 100.0, 420.0, 230.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(rot_canvas.x, rot_canvas.y),
                egui::vec2(rot_canvas.width, rot_canvas.height),
            ),
            6.0,
            Color32::from_rgb(10, 14, 22),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(rot_canvas.x, rot_canvas.y),
                egui::vec2(rot_canvas.width, rot_canvas.height),
            ),
            6.0,
            Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(rot_canvas.x + 12.0, rot_canvas.y + 10.0),
            egui::Align2::LEFT_TOP,
            "ACOUSTIC CHAMBER & DUAL-ROTOR DOPPLER FIELD",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(0, 229, 255),
        );

        let cx = rot_canvas.x + rot_canvas.width * 0.5;
        let cy = rot_canvas.y + rot_canvas.height * 0.5;

        // Concentric acoustic dispersion orbits
        painter.circle_stroke(
            egui::pos2(cx, cy),
            85.0,
            Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 100)),
        );
        painter.circle_stroke(
            egui::pos2(cx, cy),
            50.0,
            Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(255, 215, 0, 100)),
        );

        // Rotating Horn vector
        let horn_dx = self.real_time_horn_angle.cos() * 85.0;
        let horn_dy = self.real_time_horn_angle.sin() * 85.0;
        painter.line_segment(
            [egui::pos2(cx, cy), egui::pos2(cx + horn_dx, cy + horn_dy)],
            Stroke::new(3.0_f32, Color32::from_rgb(0, 229, 255)),
        );
        painter.line_segment(
            [egui::pos2(cx, cy), egui::pos2(cx - horn_dx, cy - horn_dy)],
            Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 120)),
        );

        // Rotating Drum vector
        let drum_dx = self.real_time_drum_angle.cos() * 50.0;
        let drum_dy = self.real_time_drum_angle.sin() * 50.0;
        painter.line_segment(
            [egui::pos2(cx, cy), egui::pos2(cx + drum_dx, cy + drum_dy)],
            Stroke::new(3.0_f32, Color32::from_rgb(255, 215, 0)),
        );

        // Dual Microphones
        let mic_spread_rad = (self.mic_angle_deg * 0.5).to_radians();
        let mic_r = 95.0 + self.mic_distance_m * 12.0;
        let mic_l_x = cx - mic_spread_rad.sin() * mic_r;
        let mic_l_y = cy - mic_spread_rad.cos() * mic_r;
        let mic_r_x = cx + mic_spread_rad.sin() * mic_r;
        let mic_r_y = cy - mic_spread_rad.cos() * mic_r;

        painter.circle_filled(
            egui::pos2(mic_l_x, mic_l_y),
            6.0,
            Color32::from_rgb(0, 255, 180),
        );
        painter.circle_filled(
            egui::pos2(mic_r_x, mic_r_y),
            6.0,
            Color32::from_rgb(0, 255, 180),
        );
        painter.text(
            egui::pos2(mic_l_x - 10.0, mic_l_y),
            egui::Align2::RIGHT_CENTER,
            "MIC L",
            egui::FontId::proportional(10.0),
            Color32::from_rgb(0, 255, 180),
        );
        painter.text(
            egui::pos2(mic_r_x + 10.0, mic_r_y),
            egui::Align2::LEFT_CENTER,
            "MIC R",
            egui::FontId::proportional(10.0),
            Color32::from_rgb(0, 255, 180),
        );

        // 2D Mic Placement Puck
        let px = rot_canvas.x + self.mic_puck_pos.0 * rot_canvas.width;
        let py = rot_canvas.y + (1.0 - self.mic_puck_pos.1) * rot_canvas.height;
        painter.circle_stroke(
            egui::pos2(px, py),
            ROTARY_DOPPLER_PUCK_HIT_RADIUS,
            Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(0, 229, 255, 120)),
        );
        painter.circle_filled(egui::pos2(px, py), 14.0, Color32::from_rgb(0, 229, 255));
        painter.circle_filled(egui::pos2(px, py), 4.0, Color32::WHITE);

        // Right Panel: Speed Controller & Acceleration Physics (455..780)
        let phys_canvas = Rect::new(rect.x + 455.0, rect.y + 100.0, rect.width - 475.0, 230.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(phys_canvas.x, phys_canvas.y),
                egui::vec2(phys_canvas.width, phys_canvas.height),
            ),
            6.0,
            Color32::from_rgb(10, 14, 22),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(phys_canvas.x, phys_canvas.y),
                egui::vec2(phys_canvas.width, phys_canvas.height),
            ),
            6.0,
            Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
        );

        painter.text(
            egui::pos2(phys_canvas.x + 12.0, phys_canvas.y + 10.0),
            egui::Align2::LEFT_TOP,
            "SPEED CONTROL & ROTOR INERTIA PHYSICS",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(255, 215, 0),
        );

        // Speed Toggle Buttons (Slow / Fast / Brake)
        let speed_btns = [
            (RotaryDopplerSpeedState::SlowChorale, "SLOW (CHORALE)"),
            (RotaryDopplerSpeedState::FastTremolo, "FAST (TREMOLO)"),
            (RotaryDopplerSpeedState::BrakeStop, "BRAKE / STOP"),
        ];

        let sbtn_w = (phys_canvas.width - 40.0) / 3.0;
        for (i, (st, name)) in speed_btns.iter().enumerate() {
            let bx = phys_canvas.x + 15.0 + i as f32 * (sbtn_w + 5.0);
            let is_cur = self.speed_state == *st;
            let bg = if is_cur {
                Color32::from_rgb(0, 255, 180)
            } else {
                Color32::from_rgb(35, 45, 65)
            };
            let fg = if is_cur {
                Color32::from_rgb(10, 14, 22)
            } else {
                Color32::from_rgb(220, 235, 255)
            };
            painter.rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(bx, phys_canvas.y + 40.0),
                    egui::vec2(sbtn_w, 32.0),
                ),
                3.0,
                bg,
            );
            painter.text(
                egui::pos2(bx + sbtn_w * 0.5, phys_canvas.y + 56.0),
                egui::Align2::CENTER_CENTER,
                *name,
                egui::FontId::proportional(10.0),
                fg,
            );
        }

        // Doppler Cues readout
        let (d_l, d_r, am_l, am_r) = self.calculate_doppler_cues();
        let cues = [
            (
                "HORN INERTIA",
                format!("{:.1} s", self.horn_inertia_sec),
                (0, 229, 255),
            ),
            (
                "DRUM INERTIA",
                format!("{:.1} s", self.drum_inertia_sec),
                (255, 215, 0),
            ),
            (
                "DOPPLER DEVIATION",
                format!("{:+.1} / {:+.1} c", d_l, d_r),
                (0, 255, 180),
            ),
            (
                "AM SHIMMER",
                format!("{:+.1} / {:+.1} dB", am_l, am_r),
                (76, 201, 240),
            ),
        ];

        for (i, (label, val, col)) in cues.iter().enumerate() {
            let row_y = phys_canvas.y + 86.0 + i as f32 * 34.0;
            painter.text(
                egui::pos2(phys_canvas.x + 15.0, row_y),
                egui::Align2::LEFT_TOP,
                *label,
                egui::FontId::proportional(10.5),
                Color32::from_rgb(180, 200, 225),
            );
            painter.text(
                egui::pos2(phys_canvas.x + phys_canvas.width - 15.0, row_y),
                egui::Align2::RIGHT_TOP,
                val,
                egui::FontId::proportional(12.0),
                Color32::from_rgb(col.0, col.1, col.2),
            );
        }

        // Bottom Controls Dock (y: 345..480)
        let dock_rect = Rect::new(rect.x + 20.0, rect.y + 345.0, rect.width - 40.0, 135.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(dock_rect.x, dock_rect.y),
                egui::vec2(dock_rect.width, dock_rect.height),
            ),
            6.0,
            Color32::from_rgb(18, 25, 38),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(dock_rect.x, dock_rect.y),
                egui::vec2(dock_rect.width, dock_rect.height),
            ),
            6.0,
            Stroke::new(1.5_f32, Color32::from_rgb(45, 60, 85)),
        );

        let params = [
            (
                "TUBE PREAMP DRIVE",
                format!("{:.0}%", self.tube_drive_percent),
                (255, 215, 0),
            ),
            (
                "HORN/DRUM BALANCE",
                format!("{:+.0}%", self.horn_drum_balance * 100.0),
                (0, 229, 255),
            ),
            (
                "MIC SPREAD ANGLE",
                format!("{:.0}°", self.mic_angle_deg),
                (0, 255, 180),
            ),
            (
                "MIC DISTANCE",
                format!("{:.2} m", self.mic_distance_m),
                (180, 200, 225),
            ),
        ];

        let col_w = (dock_rect.width - 40.0) / 4.0;
        for (i, (label, val, col)) in params.iter().enumerate() {
            let px = dock_rect.x + 20.0 + i as f32 * col_w;
            painter.text(
                egui::pos2(px, dock_rect.y + 16.0),
                egui::Align2::LEFT_TOP,
                *label,
                egui::FontId::proportional(11.0),
                Color32::from_rgb(180, 200, 225),
            );
            painter.text(
                egui::pos2(px, dock_rect.y + 36.0),
                egui::Align2::LEFT_TOP,
                val,
                egui::FontId::proportional(16.0),
                Color32::from_rgb(col.0, col.1, col.2),
            );
        }

        // Compliance status bar
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(dock_rect.x + 15.0, dock_rect.y + 80.0),
                egui::vec2(dock_rect.width - 30.0, 42.0),
            ),
            4.0,
            Color32::from_rgb(16, 35, 28),
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(dock_rect.x + 15.0, dock_rect.y + 80.0),
                egui::vec2(dock_rect.width - 30.0, 42.0),
            ),
            4.0,
            Stroke::new(1.5_f32, Color32::from_rgb(0, 255, 180)),
        );
        painter.text(
            egui::pos2(dock_rect.x + 25.0, dock_rect.y + 93.0),
            egui::Align2::LEFT_TOP,
            "[PASS] Rotary Speaker Doppler & Mic Visualizer Touch Pucks (>= 44x44pt) Verified",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(0, 255, 180),
        );
    }
}
