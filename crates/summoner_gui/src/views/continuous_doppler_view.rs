// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Continuous 3D Kinematic Spatializer & Doppler Shift Engine HUD (Step 1560).

use crate::layout_math::Rect;
use crate::touch_controls::ContrastColorPalette;

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke};

pub const DOPPLER_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const MIN_POS_RANGE_M: f32 = -25.0;
pub const MAX_POS_RANGE_M: f32 = 25.0;
pub const MIN_HEIGHT_RANGE_M: f32 = -10.0;
pub const MAX_HEIGHT_RANGE_M: f32 = 10.0;
pub const MIN_VEL_RANGE_MPS: f32 = -50.0;
pub const MAX_VEL_RANGE_MPS: f32 = 50.0;

/// Interpolation algorithm for fractional delay Doppler modeling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HermiteInterpolationOrder {
    CubicHermite4Pt, // 4-point cubic Hermite polynomial (zero-artifact continuous pitch)
    Linear2Pt,       // Linear 2-point interpolation (low-CPU draft)
    WindowedSinc,    // Kaiser-windowed sinc interpolation (ultra-high fidelity)
}

/// Continuous 3D Dynamic Spatializer & Doppler Engine HUD View (Step 1560).
#[derive(Debug, Clone)]
pub struct ContinuousDopplerView {
    pub pos_x: f32, // Lateral offset [-25.0 ..= +25.0 m] (Left - / Right +)
    pub pos_y: f32, // Distance depth [-25.0 ..= +25.0 m] (Back - / Front +)
    pub pos_z: f32, // Elevation height [-10.0 ..= +10.0 m] (Below - / Above +)
    pub target_x: f32,
    pub target_y: f32,
    pub target_z: f32,
    pub vel_x: f32, // Lateral velocity [-50.0 ..= +50.0 m/s]
    pub vel_y: f32, // Depth velocity [-50.0 ..= +50.0 m/s]
    pub vel_z: f32, // Elevation velocity [-50.0 ..= +50.0 m/s]
    pub speed_of_sound_mps: f32, // Speed of sound [300.0 ..= 400.0 m/s] (standard: 343.0 m/s)
    pub air_absorption_coeff: f32, // High-frequency air absorption [0.0 ..= 1.0] (default: 0.35)
    pub interpolation_order: HermiteInterpolationOrder,
    pub radial_velocity_mps: f32,
    pub doppler_ratio: f32,
    pub doppler_cents: f32,
    pub air_cutoff_hz: f32,
    pub distance_gain_db: f32,
    pub is_dragging_source: bool,
    pub color_palette: ContrastColorPalette,
}

impl Default for ContinuousDopplerView {
    fn default() -> Self {
        Self::new()
    }
}

impl ContinuousDopplerView {
    pub fn new() -> Self {
        let mut view = Self {
            pos_x: 0.0,
            pos_y: 3.5,
            pos_z: 1.0,
            target_x: 0.0,
            target_y: 3.5,
            target_z: 1.0,
            vel_x: 12.0,
            vel_y: -8.0,
            vel_z: 0.0,
            speed_of_sound_mps: 343.0,
            air_absorption_coeff: 0.35,
            interpolation_order: HermiteInterpolationOrder::CubicHermite4Pt,
            radial_velocity_mps: 0.0,
            doppler_ratio: 1.0,
            doppler_cents: 0.0,
            air_cutoff_hz: 14000.0,
            distance_gain_db: 0.0,
            is_dragging_source: false,
            color_palette: ContrastColorPalette::default(),
        };
        view.update_kinematics();
        view
    }

    /// Calculate distance from origin (meters).
    pub fn distance(&self) -> f32 {
        (self.pos_x * self.pos_x + self.pos_y * self.pos_y + self.pos_z * self.pos_z)
            .sqrt()
            .max(0.1)
    }

    /// Azimuth angle in degrees (-180.0 .. +180.0 deg). 0 is front (+Y).
    pub fn azimuth_deg(&self) -> f32 {
        self.pos_x.atan2(self.pos_y).to_degrees()
    }

    /// Elevation angle in degrees (-90.0 .. +90.0 deg).
    pub fn elevation_deg(&self) -> f32 {
        let dist = self.distance();
        (self.pos_z / dist).clamp(-1.0, 1.0).asin().to_degrees()
    }

    /// Update calculated Doppler ratio, cents deviation, air cutoff, and distance attenuation.
    pub fn update_kinematics(&mut self) {
        let dist = self.distance();
        let (v_rad, ratio, cents) = Self::calculate_doppler_ratio(
            self.pos_x,
            self.pos_y,
            self.pos_z,
            self.vel_x,
            self.vel_y,
            self.vel_z,
            self.speed_of_sound_mps,
        );
        self.radial_velocity_mps = v_rad;
        self.doppler_ratio = ratio;
        self.doppler_cents = cents;

        let el_rad = (self.pos_z / dist).clamp(-1.0, 1.0).asin();
        self.air_cutoff_hz = (20000.0
            / (1.0 + 0.08 * dist * (1.0 - 0.2 * el_rad.sin()) * (self.air_absorption_coeff / 0.35)))
            .clamp(800.0, 20000.0);

        let gain = (1.0 / dist).clamp(0.05, 2.0);
        self.distance_gain_db = 20.0 * gain.log10();
    }

    /// Calculate acoustic radial velocity (m/s), Doppler ratio, and pitch shift in cents.
    pub fn calculate_doppler_ratio(
        px: f32,
        py: f32,
        pz: f32,
        vx: f32,
        vy: f32,
        vz: f32,
        c: f32,
    ) -> (f32, f32, f32) {
        let dist = (px * px + py * py + pz * pz).sqrt();
        if dist < 1e-4 {
            return (0.0, 1.0, 0.0);
        }
        // Radial velocity towards listener (positive = approaching, negative = receding)
        // Listener is at origin (0, 0, 0).
        // Unit vector from source to listener is (-px/d, -py/d, -pz/d).
        let v_rad = -(px * vx + py * vy + pz * vz) / dist;
        let c_safe = c.max(10.0);
        let ratio = 1.0 + (v_rad / c_safe).clamp(-0.85, 0.85);
        let cents = 1200.0 * (ratio.max(1e-4)).log2();
        (v_rad, ratio, cents)
    }

    /// Normalized coordinate mapping [-25.0 ..= +25.0 m] -> [0.0 ..= 1.0].
    pub fn pos_to_normalized(val: f32, min_val: f32, max_val: f32) -> f32 {
        ((val - min_val) / (max_val - min_val)).clamp(0.0, 1.0)
    }

    /// Normalized coordinate mapping [0.0 ..= 1.0] -> [-25.0 ..= +25.0 m].
    pub fn normalized_to_pos(norm: f32, min_val: f32, max_val: f32) -> f32 {
        min_val + norm.clamp(0.0, 1.0) * (max_val - min_val)
    }

    /// Hit-test touch puck (>= 44x44pt hit bounding box).
    pub fn hit_test_source_puck(&self, pos: (f32, f32), canvas: Rect) -> bool {
        let cx = canvas.x + canvas.width * 0.5;
        let cy = canvas.y + canvas.height * 0.5;
        let max_r = (canvas.width.min(canvas.height) * 0.44).max(10.0);

        let norm_x = (self.pos_x / MAX_POS_RANGE_M).clamp(-1.0, 1.0);
        let norm_y = (self.pos_y / MAX_POS_RANGE_M).clamp(-1.0, 1.0);

        let px = cx + norm_x * max_r;
        let py = cy - norm_y * max_r; // Canvas Y inverted

        let dx = pos.0 - px;
        let dy = pos.1 - py;
        (dx * dx + dy * dy).sqrt() <= DOPPLER_PUCK_HIT_RADIUS
    }

    /// Render deterministic ASCII representation for headless terminal testing.
    pub fn render_ascii(&self, width: usize, height: usize) -> Vec<String> {
        let mut lines = Vec::with_capacity(height);
        let header = format!(
            "DOPPLER 3D [{:?}] Pos:({:.1},{:.1},{:.1})m Vel:({:.1},{:.1})m/s Vrad:{:.1}m/s Shift:{:+.1}c Gain:{:+.1}dB",
            self.interpolation_order,
            self.pos_x,
            self.pos_y,
            self.pos_z,
            self.vel_x,
            self.vel_y,
            self.radial_velocity_mps,
            self.doppler_cents,
            self.distance_gain_db
        );
        lines.push(header);

        let canvas_h = height.saturating_sub(2);
        let cx = width / 2;
        let cy = canvas_h / 2;

        for y in 0..canvas_h {
            let mut row = vec![' '; width];
            for (x, cell) in row.iter_mut().enumerate().take(width) {
                let dx = x as isize - cx as isize;
                let dy = y as isize - cy as isize;
                let r = ((dx * dx + dy * dy) as f32).sqrt();

                if r < 1.4 {
                    *cell = 'O'; // Central listener
                } else if (r - (canvas_h as f32 * 0.42)).abs() < 0.8 {
                    *cell = '.'; // Radar outer perimeter
                }
            }

            // Draw sound source puck
            let norm_x = (self.pos_x / MAX_POS_RANGE_M).clamp(-1.0, 1.0);
            let norm_y = (self.pos_y / MAX_POS_RANGE_M).clamp(-1.0, 1.0);
            let px = (cx as f32 + norm_x * (cx as f32 * 0.85)).round() as isize;
            let py = (cy as f32 - norm_y * (cy as f32 * 0.85)).round() as isize;

            if py == y as isize && px >= 0 && (px as usize) < width {
                row[px as usize] = '*';
            }

            lines.push(row.into_iter().collect());
        }

        let footer = format!(
            "SoundSpeed: {:.1}m/s | AirCutoff: {:.0}Hz | Cubic Hermite 4-Pt Delay",
            self.speed_of_sound_mps, self.air_cutoff_hz
        );
        lines.push(footer);
        lines
    }

    pub fn render_ascii_snapshot_str(&self) -> String {
        self.render_ascii(40, 16).join("\n")
    }

    #[cfg(feature = "gui")]
    pub fn show(&mut self, ui: &mut egui::Ui) {
        let avail = ui.available_size();
        let rect = Rect::new(0.0, 0.0, avail.x.max(400.0), avail.y.max(380.0));
        self.show_rect(ui, rect);
    }

    #[cfg(feature = "gui")]
    pub fn show_rect(&mut self, ui: &mut egui::Ui, layout_rect: Rect) {
        self.update_kinematics();

        let total_size = egui::vec2(layout_rect.width, layout_rect.height);
        let (response, painter) = ui.allocate_painter(total_size, egui::Sense::click_and_drag());
        let rect = response.rect;

        // Background
        painter.rect_filled(rect, 8.0, Color32::from_rgb(11, 15, 25));
        painter.rect_stroke(
            rect,
            8.0,
            Stroke::new(1.0_f32, Color32::from_rgb(30, 41, 59)),
        );

        // Header
        let header_rect = egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), 48.0));
        painter.rect_filled(header_rect, 8.0, Color32::from_rgb(15, 23, 42));
        painter.text(
            egui::pos2(header_rect.min.x + 16.0, header_rect.min.y + 12.0),
            egui::Align2::LEFT_TOP,
            "🚀 CONTINUOUS 3D DOPPLER KINEMATIC SPATIALIZER",
            egui::FontId::proportional(14.0),
            Color32::from_rgb(255, 255, 255),
        );

        let order_label = match self.interpolation_order {
            HermiteInterpolationOrder::CubicHermite4Pt => "Cubic Hermite 4-Pt",
            HermiteInterpolationOrder::Linear2Pt => "Linear 2-Pt",
            HermiteInterpolationOrder::WindowedSinc => "Windowed Sinc",
        };
        painter.text(
            egui::pos2(header_rect.max.x - 16.0, header_rect.min.y + 14.0),
            egui::Align2::RIGHT_TOP,
            format!("Interpolation: {} | c = {:.1} m/s", order_label, self.speed_of_sound_mps),
            egui::FontId::proportional(11.0),
            Color32::from_rgb(56, 189, 248),
        );

        // Radar Canvas Area
        let canvas_w = (rect.width() - 32.0).min(rect.height() - 170.0).max(200.0);
        let canvas_rect = egui::Rect::from_center_size(
            egui::pos2(rect.center().x, rect.min.y + 56.0 + canvas_w * 0.5),
            egui::vec2(canvas_w, canvas_w),
        );

        painter.rect_filled(canvas_rect, 6.0, Color32::from_rgb(6, 10, 18));
        painter.rect_stroke(
            canvas_rect,
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(30, 41, 59)),
        );

        let ccx = canvas_rect.center().x;
        let ccy = canvas_rect.center().y;
        let max_r = canvas_w * 0.44;

        // Concentric distance rings
        for (i, dist_label) in [("5m", 0.2), ("10m", 0.4), ("15m", 0.6), ("20m", 0.8), ("25m", 1.0)] {
            let r = max_r * dist_label;
            painter.circle_stroke(
                egui::pos2(ccx, ccy),
                r,
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(56, 189, 248, 40)),
            );
            painter.text(
                egui::pos2(ccx + 4.0, ccy - r - 2.0),
                egui::Align2::LEFT_BOTTOM,
                i,
                egui::FontId::proportional(9.0),
                Color32::from_rgb(100, 116, 139),
            );
        }

        // Crosshairs
        painter.line_segment(
            [egui::pos2(ccx - max_r, ccy), egui::pos2(ccx + max_r, ccy)],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(148, 163, 184, 50)),
        );
        painter.line_segment(
            [egui::pos2(ccx, ccy - max_r), egui::pos2(ccx, ccy + max_r)],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(148, 163, 184, 50)),
        );

        // Central listener marker
        painter.circle_filled(egui::pos2(ccx, ccy), 6.0, Color32::from_rgb(56, 189, 248));
        painter.circle_stroke(
            egui::pos2(ccx, ccy),
            10.0,
            Stroke::new(1.5_f32, Color32::from_rgb(14, 165, 233)),
        );

        // Sound source position
        let norm_x = (self.pos_x / MAX_POS_RANGE_M).clamp(-1.0, 1.0);
        let norm_y = (self.pos_y / MAX_POS_RANGE_M).clamp(-1.0, 1.0);
        let puck_x = ccx + norm_x * max_r;
        let puck_y = ccy - norm_y * max_r;
        let puck_pos = egui::pos2(puck_x, puck_y);

        // Velocity vector tail
        let vel_scale = max_r * 0.025;
        let vel_end = egui::pos2(
            puck_x + (self.vel_x * vel_scale).clamp(-max_r * 0.5, max_r * 0.5),
            puck_y - (self.vel_y * vel_scale).clamp(-max_r * 0.5, max_r * 0.5),
        );
        painter.line_segment(
            [puck_pos, vel_end],
            Stroke::new(2.5_f32, Color32::from_rgb(249, 115, 22)),
        );
        painter.circle_filled(vel_end, 3.0, Color32::from_rgb(251, 146, 60));

        // Drag interaction
        let canvas_bb = Rect::new(
            canvas_rect.min.x,
            canvas_rect.min.y,
            canvas_rect.width(),
            canvas_rect.height(),
        );

        if response.drag_started() {
            if let Some(pos) = response.interact_pointer_pos() {
                if self.hit_test_source_puck((pos.x, pos.y), canvas_bb) {
                    self.is_dragging_source = true;
                }
            }
        }

        if response.drag_stopped() {
            self.is_dragging_source = false;
        }

        if self.is_dragging_source {
            if let Some(pos) = response.interact_pointer_pos() {
                let dx = (pos.x - ccx) / max_r;
                let dy = (ccy - pos.y) / max_r;
                self.pos_x = (dx * MAX_POS_RANGE_M).clamp(MIN_POS_RANGE_M, MAX_POS_RANGE_M);
                self.pos_y = (dy * MAX_POS_RANGE_M).clamp(MIN_POS_RANGE_M, MAX_POS_RANGE_M);
                self.update_kinematics();
            }
        }

        // Draw Source Puck (Touch-target compliant: 44x44pt hit bounding box)
        let puck_color = if self.is_dragging_source {
            Color32::from_rgb(255, 230, 0)
        } else {
            Color32::from_rgb(0, 255, 200)
        };
        painter.circle_filled(puck_pos, 10.0, puck_color);
        painter.circle_stroke(
            puck_pos,
            DOPPLER_PUCK_HIT_RADIUS,
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 255, 200, 60)),
        );

        // Live Readouts & Metrics Panel
        let info_y = canvas_rect.max.y + 12.0;
        let info_rect = egui::Rect::from_min_size(
            egui::pos2(rect.min.x + 16.0, info_y),
            egui::vec2(rect.width() - 32.0, rect.max.y - info_y - 12.0),
        );
        painter.rect_filled(info_rect, 6.0, Color32::from_rgb(15, 23, 42));
        painter.rect_stroke(
            info_rect,
            6.0,
            Stroke::new(1.0_f32, Color32::from_rgb(30, 41, 59)),
        );

        let metrics_text = format!(
            "Distance: {:.2} m | Azimuth: {:.1}° | Elevation: {:.1}° | Radial Vel: {:+.1} m/s\nDoppler Ratio: {:.4} ({:+.1} cents) | Air Cutoff: {:.0} Hz | Attenuation: {:+.1} dB",
            self.distance(),
            self.azimuth_deg(),
            self.elevation_deg(),
            self.radial_velocity_mps,
            self.doppler_ratio,
            self.doppler_cents,
            self.air_cutoff_hz,
            self.distance_gain_db
        );
        painter.text(
            egui::pos2(info_rect.min.x + 12.0, info_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            metrics_text,
            egui::FontId::proportional(12.0),
            Color32::from_rgb(226, 232, 240),
        );
    }
}
