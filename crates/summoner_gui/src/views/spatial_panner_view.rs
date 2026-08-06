// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// 3D Spatial Panner Visualizer & Interactive GUI Canvas (Steps 1064, 1074, 1301, 1306).

use crate::layout_math::Rect;
use summoner_core::audio::ChannelLayout;
use summoner_dsp::spatial_audio::{HeadTrackerReceiver, Position3D};

#[cfg(feature = "gui")]
use eframe::egui;

/// Canvas constants for 3D Spatial Audio Panner (Step 1301).
pub const CANVAS_SIZE: f32 = 400.0;
pub const CENTER_X: f32 = 200.0;
pub const CENTER_Y: f32 = 200.0;
pub const HEAD_RADIUS: f32 = 40.0;
pub const ATTENUATION_RINGS: [f32; 3] = [80.0, 120.0, 160.0];
pub const MIN_HIT_TARGET_SIZE: f32 = 44.0;
pub const MIN_HIT_TARGET_RADIUS: f32 = 22.0;
pub const DEFAULT_MAX_DISTANCE_METERS: f32 = 5.0;

/// 3D Spatial Panner GUI View (Steps 1064, 1074, 1301, 1306).
#[derive(Debug, Clone)]
pub struct SpatialPannerView {
    pub layout: ChannelLayout,
    pub listener_pos: Position3D,
    pub head_tracker: HeadTrackerReceiver,
    pub sources: Vec<(String, Position3D)>,
    pub grid_bounds: (f32, f32, f32), // Width, Depth, Height
    pub is_hmd_active: bool,          // Step 1074 VR/AR HMD companion view active flag
    pub head_tracking_enabled: bool,  // Step 1301 Head-tracking toggle
    pub selected_source_index: Option<usize>,
    pub max_distance_meters: f32,
    pub drag_active_source: Option<usize>,
    pub azimuth_deg: f32,
    pub distance_meters: f32,
}

impl Default for SpatialPannerView {
    fn default() -> Self {
        Self::new(ChannelLayout::Surround7_1_4)
    }
}

impl SpatialPannerView {
    pub fn new(layout: ChannelLayout) -> Self {
        Self {
            layout,
            listener_pos: Position3D::zero(),
            head_tracker: HeadTrackerReceiver::new(),
            sources: vec![
                ("Vocals".into(), Position3D::new(0.0, 1.5, 0.2)),
                ("Guitar".into(), Position3D::new(-1.2, 2.0, 0.0)),
                ("Synth".into(), Position3D::new(1.2, 2.0, 0.0)),
            ],
            grid_bounds: (10.0, 10.0, 4.0),
            is_hmd_active: false,
            head_tracking_enabled: true,
            selected_source_index: Some(0),
            max_distance_meters: DEFAULT_MAX_DISTANCE_METERS,
            drag_active_source: None,
            azimuth_deg: 0.0,
            distance_meters: 1.0,
        }
    }

    pub fn add_source(&mut self, name: impl Into<String>, pos: Position3D) {
        self.sources.push((name.into(), pos));
    }

    pub fn set_hmd_active(&mut self, active: bool) {
        self.is_hmd_active = active;
    }

    pub fn set_head_tracking_enabled(&mut self, enabled: bool) {
        self.head_tracking_enabled = enabled;
    }

    pub fn select_source(&mut self, index: Option<usize>) {
        if let Some(idx) = index {
            if idx < self.sources.len() {
                self.selected_source_index = Some(idx);
                let pos = &self.sources[idx].1;
                self.azimuth_deg = pos.azimuth().to_degrees();
                self.distance_meters = pos.distance();
            }
        } else {
            self.selected_source_index = None;
        }
    }

    // --- Spatial Math (Step 1301) ---

    /// Distance attenuation formula: attenuation = 1.0 / (1.0 + distance_meters * 0.5)
    pub fn calculate_attenuation(distance_meters: f32) -> f32 {
        1.0 / (1.0 + distance_meters.max(0.0) * 0.5)
    }

    /// Instance method for distance attenuation based on current distance_meters field
    pub fn distance_attenuation(&self) -> f32 {
        Self::calculate_attenuation(self.distance_meters)
    }

    /// Coordinate math: x = center_x + dist_scale * sin(azimuth_rad), y = center_y - dist_scale * cos(azimuth_rad)
    pub fn polar_to_canvas(
        center_x: f32,
        center_y: f32,
        azimuth_rad: f32,
        dist_scale: f32,
    ) -> (f32, f32) {
        let x = center_x + dist_scale * azimuth_rad.sin();
        let y = center_y - dist_scale * azimuth_rad.cos();
        (x, y)
    }

    /// Calculate source pixel position on canvas given center and radius (dist_scale)
    pub fn calculate_source_pos_px(&self, center_x: f32, center_y: f32, radius: f32) -> (f32, f32) {
        let az_rad = self.azimuth_deg.to_radians();
        Self::polar_to_canvas(center_x, center_y, az_rad, radius)
    }

    /// Get hit target bounding box for source handle (min hit target size >= 44.0pt)
    pub fn source_hit_target_bounds(&self, x: f32, y: f32) -> Rect {
        Rect::new(
            x - MIN_HIT_TARGET_RADIUS,
            y - MIN_HIT_TARGET_RADIUS,
            MIN_HIT_TARGET_SIZE,
            MIN_HIT_TARGET_SIZE,
        )
    }

    /// Inverse coordinate math: canvas (x, y) to (azimuth_rad, dist_scale)
    pub fn canvas_to_polar(
        center_x: f32,
        center_y: f32,
        canvas_x: f32,
        canvas_y: f32,
    ) -> (f32, f32) {
        let dx = canvas_x - center_x;
        let dy = canvas_y - center_y;
        let azimuth_rad = dx.atan2(-dy);
        let dist_scale = (dx * dx + dy * dy).sqrt();
        (azimuth_rad, dist_scale)
    }

    /// Convert Position3D to 2D canvas coordinates (x, y) in points (0..400).
    pub fn pos3d_to_canvas(&self, pos: &Position3D) -> (f32, f32) {
        let dist_m = (pos.x * pos.x + pos.y * pos.y).sqrt();
        let az_rad = pos.azimuth();
        let scale = 160.0 / self.max_distance_meters.max(0.1);
        let dist_scale = (dist_m * scale).min(180.0);
        Self::polar_to_canvas(CENTER_X, CENTER_Y, az_rad, dist_scale)
    }

    /// Convert 2D canvas coordinates (x, y) back to Position3D at elevation z.
    pub fn canvas_to_pos3d(&self, canvas_x: f32, canvas_y: f32, elevation_z: f32) -> Position3D {
        let (az_rad, dist_scale) = Self::canvas_to_polar(CENTER_X, CENTER_Y, canvas_x, canvas_y);
        let scale = 160.0 / self.max_distance_meters.max(0.1);
        let dist_m = dist_scale / scale;
        let x = dist_m * az_rad.sin();
        let y = dist_m * az_rad.cos();
        Position3D::new(x, y, elevation_z)
    }

    /// Check if a click point is within the min hit target region (radius >= 22.0pt / size >= 44.0pt).
    pub fn is_hit_target(source_canvas_pos: (f32, f32), click_canvas_pos: (f32, f32)) -> bool {
        let dx = source_canvas_pos.0 - click_canvas_pos.0;
        let dy = source_canvas_pos.1 - click_canvas_pos.1;
        (dx * dx + dy * dy).sqrt() <= MIN_HIT_TARGET_RADIUS
    }

    /// Perform hit test across all sources to find selected node index.
    pub fn hit_test(&self, click_canvas_pos: (f32, f32)) -> Option<usize> {
        for (i, (_, pos)) in self.sources.iter().enumerate() {
            let canvas_pos = self.pos3d_to_canvas(pos);
            if Self::is_hit_target(canvas_pos, click_canvas_pos) {
                return Some(i);
            }
        }
        None
    }

    /// Format status bar info readout matching required format:
    /// Azimuth: 45° | Elevation: 0° | Dist: 2.5m | Head-Track: ON
    pub fn format_status_line(&self, source_idx: usize) -> String {
        if let Some((_, pos)) = self.sources.get(source_idx) {
            let az_deg = pos.azimuth().to_degrees();
            let elev_deg = pos.elevation().to_degrees();
            let dist_m = pos.distance();
            let head_track_str = if self.head_tracking_enabled {
                "ON"
            } else {
                "OFF"
            };
            format!(
                "Azimuth: {:.0}° | Elevation: {:.0}° | Dist: {:.1}m | Head-Track: {}",
                az_deg, elev_deg, dist_m, head_track_str
            )
        } else {
            let head_track_str = if self.head_tracking_enabled {
                "ON"
            } else {
                "OFF"
            };
            format!(
                "Azimuth: 0° | Elevation: 0° | Dist: 0.0m | Head-Track: {}",
                head_track_str
            )
        }
    }

    /// Render ASCII/CLI representation of 3D spatial room.
    pub fn render_ascii(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "[3D Spatial Panner View - Layout: {:?}]\n",
            self.layout
        ));
        out.push_str(&format!(
            "Listener (Head Tracker): Yaw {:.1} deg | Pitch {:.1} deg | Roll {:.1} deg\n",
            self.head_tracker.yaw_deg, self.head_tracker.pitch_deg, self.head_tracker.roll_deg
        ));
        out.push_str(&format!(
            "HMD Companion Mode: {}\n",
            if self.is_hmd_active {
                "ACTIVE (OpenXR)"
            } else {
                "OFF"
            }
        ));
        out.push_str("Sources:\n");
        for (name, pos) in &self.sources {
            let att = Self::calculate_attenuation(pos.distance());
            out.push_str(&format!(
                " - {:<12}: X={:+.2}m, Y={:+.2}m, Z={:+.2}m (Az: {:.1}deg, Dist: {:.2}m, Att: {:.2})\n",
                name,
                pos.x,
                pos.y,
                pos.z,
                pos.azimuth().to_degrees(),
                pos.distance(),
                att
            ));
        }
        if let Some(idx) = self.selected_source_index {
            out.push_str(&format!(
                "Selected Status: {}\n",
                self.format_status_line(idx)
            ));
        }
        out
    }
}

#[cfg(feature = "gui")]
impl SpatialPannerView {
    /// Render egui interactive 3D spatial panner widget (400x400pt canvas).
    pub fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical(|ui| {
            ui.heading("SPATIAL AUDIO 3D PANNER CANVAS");

            let (response, painter) = ui.allocate_painter(
                egui::vec2(CANVAS_SIZE, CANVAS_SIZE),
                egui::Sense::click_and_drag(),
            );

            let rect = response.rect;
            let origin = rect.min;
            let center = origin + egui::vec2(CENTER_X, CENTER_Y);

            // Canvas Background
            painter.rect_filled(rect, 8.0_f32, egui::Color32::from_rgb(18, 22, 32));
            painter.rect_stroke(
                rect,
                8.0_f32,
                egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(45, 55, 75)),
            );

            // Attenuation Rings (80pt, 120pt, 160pt)
            for &r in &ATTENUATION_RINGS {
                painter.circle_stroke(
                    center,
                    r,
                    egui::Stroke::new(
                        1.0_f32,
                        egui::Color32::from_rgba_unmultiplied(100, 150, 220, 60),
                    ),
                );
                // Distance ring labels
                let dist_m = (r / 160.0) * self.max_distance_meters;
                let att = Self::calculate_attenuation(dist_m);
                let label = format!("{:.1}m ({:.0}%)", dist_m, att * 100.0);
                painter.text(
                    center + egui::vec2(5.0, -r + 2.0),
                    egui::Align2::LEFT_TOP,
                    label,
                    egui::FontId::proportional(10.0),
                    egui::Color32::from_rgba_unmultiplied(140, 170, 210, 120),
                );
            }

            // Binaural Head Orientation Head-Ring (Radius 40pt)
            let head_color = if self.head_tracking_enabled {
                egui::Color32::from_rgb(0, 220, 255)
            } else {
                egui::Color32::from_rgb(150, 160, 180)
            };
            painter.circle_filled(
                center,
                HEAD_RADIUS,
                egui::Color32::from_rgba_unmultiplied(30, 80, 140, 70),
            );
            painter.circle_stroke(center, HEAD_RADIUS, egui::Stroke::new(2.0_f32, head_color));

            // Head Orientation Pointer / Nose
            let yaw_rad = if self.head_tracking_enabled {
                self.head_tracker.yaw_deg.to_radians()
            } else {
                0.0
            };
            let nose_pos =
                center + egui::vec2(HEAD_RADIUS * yaw_rad.sin(), -HEAD_RADIUS * yaw_rad.cos());
            painter.line_segment([center, nose_pos], egui::Stroke::new(2.0_f32, head_color));
            painter.circle_filled(nose_pos, 4.0_f32, head_color);

            // Listener label
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                "(O)",
                egui::FontId::proportional(12.0),
                egui::Color32::WHITE,
            );

            // Cardinal Direction Markers
            let card_font = egui::FontId::proportional(11.0);
            let card_color = egui::Color32::from_rgb(180, 195, 215);
            painter.text(
                origin + egui::vec2(CENTER_X, 14.0),
                egui::Align2::CENTER_CENTER,
                "[ 0 deg - FRONT ]",
                card_font.clone(),
                card_color,
            );
            painter.text(
                origin + egui::vec2(CANVAS_SIZE - 25.0, CENTER_Y),
                egui::Align2::RIGHT_CENTER,
                "[ 90 R]",
                card_font.clone(),
                card_color,
            );
            painter.text(
                origin + egui::vec2(CENTER_X, CANVAS_SIZE - 14.0),
                egui::Align2::CENTER_CENTER,
                "[ 180 deg - BACK ]",
                card_font.clone(),
                card_color,
            );
            painter.text(
                origin + egui::vec2(25.0, CENTER_Y),
                egui::Align2::LEFT_CENTER,
                "[270 L]",
                card_font.clone(),
                card_color,
            );

            // Render Sources (*)
            for (idx, (name, pos)) in self.sources.iter().enumerate() {
                let (cx, cy) = self.pos3d_to_canvas(pos);
                let src_pos = origin + egui::vec2(cx, cy);
                let is_selected = self.selected_source_index == Some(idx);
                let att = Self::calculate_attenuation(pos.distance());

                // Attenuation visualizer halo radius proportional to attenuation
                let halo_r = 15.0 + att * 20.0;
                painter.circle_filled(
                    src_pos,
                    halo_r,
                    egui::Color32::from_rgba_unmultiplied(255, 180, 50, (att * 50.0) as u8),
                );

                // Hit target outline (min hit target size 44pt => radius 22pt)
                let stroke_color = if is_selected {
                    egui::Color32::from_rgb(255, 215, 0)
                } else {
                    egui::Color32::from_rgb(0, 190, 255)
                };
                let stroke_w = if is_selected { 2.5_f32 } else { 1.0_f32 };
                painter.circle_stroke(
                    src_pos,
                    MIN_HIT_TARGET_RADIUS,
                    egui::Stroke::new(stroke_w, stroke_color),
                );

                // Source node handle center dot (*)
                painter.circle_filled(src_pos, 6.0_f32, stroke_color);

                // Source name label
                painter.text(
                    src_pos + egui::vec2(0.0, -18.0),
                    egui::Align2::CENTER_BOTTOM,
                    format!("{} (*)", name),
                    egui::FontId::proportional(11.0),
                    egui::Color32::WHITE,
                );
            }

            // Handle Input (Interaction)
            if response.clicked() {
                if let Some(mouse_pos) = response.interact_pointer_pos() {
                    let canvas_click = (mouse_pos.x - origin.x, mouse_pos.y - origin.y);
                    if let Some(hit_idx) = self.hit_test(canvas_click) {
                        self.selected_source_index = Some(hit_idx);
                        let pos = &self.sources[hit_idx].1;
                        self.azimuth_deg = pos.azimuth().to_degrees();
                        self.distance_meters = pos.distance();
                    }
                }
            }

            if response.dragged() {
                if let Some(mouse_pos) = response.interact_pointer_pos() {
                    let canvas_drag = (mouse_pos.x - origin.x, mouse_pos.y - origin.y);
                    let sel_idx = self.selected_source_index.unwrap_or(0);
                    if sel_idx < self.sources.len() {
                        let cur_z = self.sources[sel_idx].1.z;
                        let new_pos = self.canvas_to_pos3d(canvas_drag.0, canvas_drag.1, cur_z);
                        self.sources[sel_idx].1 = new_pos;
                        self.azimuth_deg = new_pos.azimuth().to_degrees();
                        self.distance_meters = new_pos.distance();
                    }
                }
            }

            // Status bar readout line at bottom
            let status_str = if let Some(idx) = self.selected_source_index {
                self.format_status_line(idx)
            } else {
                self.format_status_line(0)
            };

            ui.add_space(6.0);
            ui.label(
                egui::RichText::new(status_str)
                    .monospace()
                    .color(egui::Color32::from_rgb(0, 220, 255)),
            );

            response
        })
        .inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_panner_view_ascii_render() {
        let mut view = SpatialPannerView::new(ChannelLayout::Surround7_1_4);
        view.head_tracker.yaw_deg = 15.0;
        view.set_hmd_active(true);
        let ascii = view.render_ascii();
        assert!(ascii.contains("7_1_4"));
        assert!(ascii.contains("ACTIVE (OpenXR)"));
        assert!(ascii.contains("Vocals"));
    }
}
