// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// Multi-Channel Surround Sound Panner with 3D Ambisonic Trajectory Radar (Step 1382).

use crate::touch_controls::{ContrastColorPalette, MIN_HIT_TARGET_PT};

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, Stroke, Vec2};

pub const AMBISONIC_PUCK_VISUAL_RADIUS: f32 = 14.0;
pub const AMBISONIC_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding box
pub const SPEAKER_MARKER_RADIUS: f32 = 10.0;

/// Ambisonic or surround speaker setup format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmbisonicFormat {
    FirstOrderBFormat, // 4-ch W, X, Y, Z
    SecondOrderHOA,    // 9-ch
    ThirdOrderHOA,     // 16-ch
    Surround51,        // 5.1 L, R, C, LFE, Ls, Rs
    Atmos714,          // 7.1.4 (7 bed, 1 LFE, 4 ceiling)
    Immersive916,      // 9.1.6 Immersive
}

impl AmbisonicFormat {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::FirstOrderBFormat => "1st Order (FOA / 4-ch)",
            Self::SecondOrderHOA => "2nd Order (HOA / 9-ch)",
            Self::ThirdOrderHOA => "3rd Order (HOA / 16-ch)",
            Self::Surround51 => "5.1 Surround",
            Self::Atmos714 => "7.1.4 Dolby Atmos",
            Self::Immersive916 => "9.1.6 Immersive 3D",
        }
    }

    pub fn channel_count(&self) -> usize {
        match self {
            Self::FirstOrderBFormat => 4,
            Self::SecondOrderHOA => 9,
            Self::ThirdOrderHOA => 16,
            Self::Surround51 => 6,
            Self::Atmos714 => 12,
            Self::Immersive916 => 16,
        }
    }
}

/// Automated motion trajectory shape for 3D sound objects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrajectoryShape {
    Static,
    OrbitCircle,
    LissajousFigure8,
    InwardSpiral,
    RandomWander,
    PingPong,
}

impl TrajectoryShape {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Static => "Static (Manual)",
            Self::OrbitCircle => "Orbit Circle",
            Self::LissajousFigure8 => "Lissajous (8)",
            Self::InwardSpiral => "Inward Spiral",
            Self::RandomWander => "Random Wander",
            Self::PingPong => "Ping-Pong",
        }
    }
}

/// 3D Spatial Sound Source Object.
#[derive(Debug, Clone, PartialEq)]
pub struct AmbisonicSource {
    pub id: String,
    pub name: String,
    pub azimuth_deg: f32,   // -180.0 ..= +180.0 degrees (0 = Front/North)
    pub elevation_deg: f32, // -90.0 ..= +90.0 degrees (0 = Horizon, +90 = Zenith, -90 = Nadir)
    pub distance_m: f32,    // 0.1 ..= 10.0 meters
    pub spread_pct: f32,    // 0.0 ..= 100.0% divergence
    pub gain_db: f32,       // -60.0 ..= +12.0 dB
    pub trajectory: TrajectoryShape,
    pub trajectory_speed_hz: f32, // 0.05 ..= 5.0 Hz
    pub trajectory_phase: f32,    // 0.0 ..= 1.0
    pub is_selected: bool,
    pub is_dragging: bool,
}

impl AmbisonicSource {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        az: f32,
        el: f32,
        dist: f32,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            azimuth_deg: az.clamp(-180.0, 180.0),
            elevation_deg: el.clamp(-90.0, 90.0),
            distance_m: dist.clamp(0.1, 10.0),
            spread_pct: 20.0,
            gain_db: 0.0,
            trajectory: TrajectoryShape::Static,
            trajectory_speed_hz: 0.25,
            trajectory_phase: 0.0,
            is_selected: false,
            is_dragging: false,
        }
    }

    /// Advance trajectory position by dt seconds.
    pub fn step_trajectory(&mut self, dt: f32) {
        if self.trajectory == TrajectoryShape::Static {
            return;
        }

        self.trajectory_phase = (self.trajectory_phase + self.trajectory_speed_hz * dt) % 1.0;
        let p = self.trajectory_phase * std::f32::consts::TAU;

        match self.trajectory {
            TrajectoryShape::OrbitCircle => {
                self.azimuth_deg = (p.to_degrees() % 360.0) - 180.0;
            }
            TrajectoryShape::LissajousFigure8 => {
                let az = (p.sin() * 120.0).clamp(-180.0, 180.0);
                let el = ((2.0 * p).sin() * 45.0).clamp(-90.0, 90.0);
                self.azimuth_deg = az;
                self.elevation_deg = el;
            }
            TrajectoryShape::InwardSpiral => {
                self.azimuth_deg = ((p * 2.0).to_degrees() % 360.0) - 180.0;
                self.distance_m = 1.0 + (p.cos() * 0.5 + 0.5) * 4.0;
            }
            TrajectoryShape::PingPong => {
                self.azimuth_deg = (p.sin() * 90.0).clamp(-180.0, 180.0);
            }
            _ => {}
        }
    }
}

/// 3D Ambisonic Radar and Surround Sound Panner View (Step 1382).
#[derive(Debug, Clone)]
pub struct AmbisonicRadarView {
    pub format: AmbisonicFormat,
    pub sources: Vec<AmbisonicSource>,
    pub selected_source_idx: Option<usize>,
    pub dragging_source_idx: Option<usize>,
    pub master_elevation_offset_deg: f32, // -90.0 ..= +90.0 deg
    pub air_absorption_filter_enabled: bool,
    pub doppler_effect_enabled: bool,
    pub binaural_headphone_preview: bool,
    pub max_distance_m: f32, // 1.0 ..= 20.0 m (default 5.0m)
    pub color_palette: ContrastColorPalette,
}

impl Default for AmbisonicRadarView {
    fn default() -> Self {
        Self::new()
    }
}

impl AmbisonicRadarView {
    pub fn new() -> Self {
        let sources = vec![
            AmbisonicSource::new("src_1", "Lead Synth (3D)", -30.0, 15.0, 2.5),
            AmbisonicSource::new("src_2", "Percussion Space", 60.0, -10.0, 3.8),
            AmbisonicSource::new("src_3", "Vocal Height Layer", 0.0, 45.0, 1.8),
        ];

        Self {
            format: AmbisonicFormat::Atmos714,
            sources,
            selected_source_idx: Some(0),
            dragging_source_idx: None,
            master_elevation_offset_deg: 0.0,
            air_absorption_filter_enabled: true,
            doppler_effect_enabled: true,
            binaural_headphone_preview: true,
            max_distance_m: 5.0,
            color_palette: ContrastColorPalette::default(),
        }
    }

    /// Convert polar coordinates (azimuth, distance) to 2D radar screen pixel position.
    pub fn polar_to_screen_pos(
        &self,
        azimuth_deg: f32,
        distance_m: f32,
        center: (f32, f32),
        radar_radius: f32,
    ) -> (f32, f32) {
        let norm_dist = (distance_m / self.max_distance_m).clamp(0.0, 1.0);
        let rad = azimuth_deg.to_radians();
        // Azimuth 0 = Front/Top (negative Y in screen coords)
        let sx = center.0 + norm_dist * radar_radius * rad.sin();
        let sy = center.1 - norm_dist * radar_radius * rad.cos();
        (sx, sy)
    }

    /// Convert 2D radar screen pixel position to polar coordinates (azimuth_deg, distance_m).
    pub fn screen_pos_to_polar(
        &self,
        pos: (f32, f32),
        center: (f32, f32),
        radar_radius: f32,
    ) -> (f32, f32) {
        let dx = pos.0 - center.0;
        let dy = pos.1 - center.1;
        let dist_px = (dx * dx + dy * dy).sqrt();
        let norm_dist = (dist_px / radar_radius).clamp(0.0, 1.0);
        let distance_m = (norm_dist * self.max_distance_m).max(0.1);

        // Azimuth: atan2(dx, -dy) gives 0 at top, positive clockwise
        let rad = dx.atan2(-dy);
        let az_deg = rad.to_degrees().clamp(-180.0, 180.0);

        (az_deg, distance_m)
    }

    /// Hit-test source pucks with ergonomic minimum hit target (>=44x44pt).
    pub fn hit_test_source(
        &self,
        pos: (f32, f32),
        center: (f32, f32),
        radar_radius: f32,
    ) -> Option<usize> {
        for (idx, src) in self.sources.iter().enumerate() {
            let (sx, sy) =
                self.polar_to_screen_pos(src.azimuth_deg, src.distance_m, center, radar_radius);
            let dx = pos.0 - sx;
            let dy = pos.1 - sy;
            if (dx * dx + dy * dy).sqrt() <= AMBISONIC_PUCK_HIT_RADIUS {
                return Some(idx);
            }
        }
        None
    }

    /// Add a new 3D spatial audio source.
    pub fn add_source(&mut self, name: impl Into<String>, az: f32, el: f32, dist: f32) -> usize {
        let id = format!("src_{}", self.sources.len() + 1);
        let src = AmbisonicSource::new(id, name, az, el, dist);
        self.sources.push(src);
        self.sources.len() - 1
    }

    /// Deterministic ASCII render of the ambisonic radar disc.
    pub fn render_ascii(&self, grid_size: usize) -> String {
        let size = grid_size.max(7);
        let mut grid = vec![vec!['.'; size]; size];
        let mid = (size / 2) as f32;

        // Draw center listener
        grid[size / 2][size / 2] = 'L';

        // Draw sources
        for (idx, src) in self.sources.iter().enumerate() {
            let norm_dist = (src.distance_m / self.max_distance_m).clamp(0.0, 1.0);
            let rad = src.azimuth_deg.to_radians();
            let gx = (mid + norm_dist * mid * rad.sin()).round() as i32;
            let gy = (mid - norm_dist * mid * rad.cos()).round() as i32;

            if gx >= 0 && gx < size as i32 && gy >= 0 && gy < size as i32 {
                let char_id = char::from_digit((idx + 1) as u32, 10).unwrap_or('*');
                grid[gy as usize][gx as usize] = char_id;
            }
        }

        let mut lines = Vec::new();
        for row in grid {
            lines.push(row.into_iter().collect::<String>());
        }
        lines.join("\n")
    }
}

#[cfg(feature = "gui")]
impl AmbisonicRadarView {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // 1. Top Header Toolbar
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("3D AMBISONIC RADAR & SPATIAL PANNER")
                        .color(Color32::from_rgb(240, 245, 255))
                        .strong(),
                );
                ui.separator();
                ui.label(
                    egui::RichText::new(format!("Format: {}", self.format.display_name()))
                        .color(Color32::from_rgb(0, 229, 255))
                        .strong(),
                );
                ui.separator();
                ui.checkbox(&mut self.binaural_headphone_preview, "Binaural HRTF");
                ui.checkbox(&mut self.air_absorption_filter_enabled, "Air Absorption");
            });

            ui.add_space(6.0);

            // Format Selection Bar (>=44pt Touch Targets)
            ui.horizontal(|ui| {
                let formats = [
                    AmbisonicFormat::FirstOrderBFormat,
                    AmbisonicFormat::SecondOrderHOA,
                    AmbisonicFormat::Surround51,
                    AmbisonicFormat::Atmos714,
                    AmbisonicFormat::Immersive916,
                ];
                for fmt in formats {
                    let is_active = self.format == fmt;
                    let btn = egui::Button::new(
                        egui::RichText::new(fmt.display_name())
                            .color(if is_active {
                                Color32::from_rgb(10, 14, 22)
                            } else {
                                Color32::from_rgb(220, 235, 255)
                            })
                            .strong(),
                    )
                    .min_size(Vec2::new(90.0, MIN_HIT_TARGET_PT))
                    .fill(if is_active {
                        Color32::from_rgb(0, 229, 255)
                    } else {
                        Color32::from_rgb(30, 40, 60)
                    });

                    if ui.add(btn).clicked() {
                        self.format = fmt;
                    }
                }
            });

            ui.add_space(8.0);

            // 2. 3D Ambisonic Polar Radar Canvas
            let radar_size = 280.0_f32;
            let (response, painter) = ui.allocate_painter(
                Vec2::new(radar_size, radar_size),
                egui::Sense::click_and_drag(),
            );
            let center = (
                response.rect.min.x + radar_size * 0.5,
                response.rect.min.y + radar_size * 0.5,
            );
            let radius = radar_size * 0.45;

            // Radar background disc
            painter.circle_filled(
                egui::pos2(center.0, center.1),
                radius,
                Color32::from_rgb(10, 14, 22),
            );
            painter.circle_stroke(
                egui::pos2(center.0, center.1),
                radius,
                Stroke::new(2.0_f32, Color32::from_rgb(45, 60, 85)),
            );

            // Concentric Distance & Elevation Rings (0.25, 0.5, 0.75, 1.0)
            let ring_fracs = [0.25_f32, 0.50_f32, 0.75_f32, 1.0_f32];
            for frac in ring_fracs {
                let r = radius * frac;
                painter.circle_stroke(
                    egui::pos2(center.0, center.1),
                    r,
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(50, 70, 100, 80)),
                );
            }

            // Radar Crosshairs (Front/Back, Left/Right)
            painter.line_segment(
                [
                    egui::pos2(center.0, center.1 - radius),
                    egui::pos2(center.0, center.1 + radius),
                ],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 110, 100)),
            );
            painter.line_segment(
                [
                    egui::pos2(center.0 - radius, center.1),
                    egui::pos2(center.0 + radius, center.1),
                ],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(60, 80, 110, 100)),
            );

            // Direction Labels
            painter.text(
                egui::pos2(center.0, center.1 - radius - 10.0_f32),
                egui::Align2::CENTER_CENTER,
                "FRONT (0°)",
                egui::FontId::proportional(10.0_f32),
                Color32::from_rgb(0, 229, 255),
            );
            painter.text(
                egui::pos2(center.0 + radius + 14.0_f32, center.1),
                egui::Align2::LEFT_CENTER,
                "R (+90°)",
                egui::FontId::proportional(10.0_f32),
                Color32::from_rgb(140, 165, 195),
            );
            painter.text(
                egui::pos2(center.0, center.1 + radius + 10.0_f32),
                egui::Align2::CENTER_CENTER,
                "REAR (180°)",
                egui::FontId::proportional(10.0_f32),
                Color32::from_rgb(140, 165, 195),
            );
            painter.text(
                egui::pos2(center.0 - radius - 14.0_f32, center.1),
                egui::Align2::RIGHT_CENTER,
                "L (-90°)",
                egui::FontId::proportional(10.0_f32),
                Color32::from_rgb(140, 165, 195),
            );

            // Center Listener Puck
            painter.circle_filled(
                egui::pos2(center.0, center.1),
                8.0_f32,
                Color32::from_rgb(0, 255, 180),
            );

            // Draw Sound Object Pucks (>=44x44pt Touch Targets)
            for (idx, src) in self.sources.iter().enumerate() {
                let (sx, sy) =
                    self.polar_to_screen_pos(src.azimuth_deg, src.distance_m, center, radius);
                let p_pos = egui::pos2(sx, sy);
                let is_sel = self.selected_source_idx == Some(idx);

                // Touch Target Hit Ring (22pt radius = 44x44pt bounding box)
                painter.circle_stroke(
                    p_pos,
                    AMBISONIC_PUCK_HIT_RADIUS,
                    Stroke::new(
                        1.5_f32,
                        if is_sel {
                            Color32::from_rgb(255, 215, 0)
                        } else {
                            Color32::from_rgba_unmultiplied(0, 229, 255, 100)
                        },
                    ),
                );

                // Puck Body
                painter.circle_filled(
                    p_pos,
                    AMBISONIC_PUCK_VISUAL_RADIUS,
                    if is_sel {
                        Color32::from_rgb(255, 215, 0)
                    } else {
                        Color32::from_rgb(0, 229, 255)
                    },
                );
                // Center dot
                painter.circle_filled(p_pos, 4.0_f32, Color32::from_rgb(10, 14, 22));

                // Source Number & Elevation Tag
                painter.text(
                    egui::pos2(sx, sy - 18.0_f32),
                    egui::Align2::CENTER_BOTTOM,
                    format!("{}: {:+.0}°", idx + 1, src.elevation_deg),
                    egui::FontId::proportional(10.0_f32),
                    Color32::from_rgb(240, 245, 255),
                );
            }

            // Radar Drag Handling
            if response.drag_started() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if let Some(s_idx) = self.hit_test_source((pos.x, pos.y), center, radius) {
                        self.selected_source_idx = Some(s_idx);
                        self.dragging_source_idx = Some(s_idx);
                    }
                }
            }

            if response.dragged() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if let Some(s_idx) = self.dragging_source_idx {
                        let (az, dist) = self.screen_pos_to_polar((pos.x, pos.y), center, radius);
                        self.sources[s_idx].azimuth_deg = az;
                        self.sources[s_idx].distance_m = dist;
                    }
                }
            }

            if response.drag_stopped() {
                self.dragging_source_idx = None;
            }

            ui.add_space(8.0);

            // 3. Selected Source Parameter Inspector Card
            if let Some(s_idx) = self.selected_source_idx {
                if let Some(src) = self.sources.get_mut(s_idx) {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "SELECTED OBJECT #{}: {}",
                                    s_idx + 1,
                                    src.name
                                ))
                                .color(Color32::from_rgb(255, 215, 0))
                                .strong(),
                            );
                            ui.separator();
                            ui.label(format!(
                                "Azimuth: {:+.1}° | Elev: {:+.1}° | Dist: {:.2} m",
                                src.azimuth_deg, src.elevation_deg, src.distance_m
                            ));
                        });

                        ui.add_space(6.0);

                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("Elevation").strong());
                                ui.add(
                                    egui::Slider::new(&mut src.elevation_deg, -90.0..=90.0)
                                        .text("°"),
                                );
                            });

                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("Distance").strong());
                                ui.add(
                                    egui::Slider::new(&mut src.distance_m, 0.1..=10.0).text("m"),
                                );
                            });

                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("Divergence Spread").strong());
                                ui.add(
                                    egui::Slider::new(&mut src.spread_pct, 0.0..=100.0).text("%"),
                                );
                            });

                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("Gain").strong());
                                ui.add(
                                    egui::Slider::new(&mut src.gain_db, -60.0..=12.0).text("dB"),
                                );
                            });
                        });

                        ui.add_space(6.0);

                        // Trajectory Shape Selector Buttons (>=44pt Touch Targets)
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Trajectory:").strong());
                            let shapes = [
                                TrajectoryShape::Static,
                                TrajectoryShape::OrbitCircle,
                                TrajectoryShape::LissajousFigure8,
                                TrajectoryShape::InwardSpiral,
                                TrajectoryShape::PingPong,
                            ];
                            for sh in shapes {
                                let is_act = src.trajectory == sh;
                                let btn = egui::Button::new(
                                    egui::RichText::new(sh.display_name())
                                        .color(if is_act {
                                            Color32::from_rgb(10, 14, 22)
                                        } else {
                                            Color32::from_rgb(220, 235, 255)
                                        })
                                        .strong(),
                                )
                                .min_size(Vec2::new(80.0, MIN_HIT_TARGET_PT))
                                .fill(if is_act {
                                    Color32::from_rgb(0, 229, 255)
                                } else {
                                    Color32::from_rgb(30, 40, 60)
                                });

                                if ui.add(btn).clicked() {
                                    src.trajectory = sh;
                                }
                            }
                        });
                    });
                }
            }
        });
    }
}
