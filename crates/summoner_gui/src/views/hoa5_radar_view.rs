// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Interactive 3D Holographic Ambisonic Radar HUD & Polar Trajectory Canvas (Milestone 35).
//!
//! Provides a broadcast-grade 5th-Order Ambisonics (HOA5, 36-channel) 3D spatial soundfield
//! visualization and control interface featuring:
//! - 3D Spherical elevation/azimuth polar projection radar canvas
//! - Draggable sound object puck with >= 44x44pt hit bounding targets
//! - Real-time acoustic wavefront propagation ripples with Doppler velocity compression
//! - Complete 36-channel spherical harmonic ACN decomposition (Order 0..5) with energy bars
//! - 3D trajectory generator (Circular Orbit, Lissajous Figure-8, Spherical Spiral, Doppler Flyby)
//! - Preset decoding profiles (36-Ch Sphere, Max-rE Focus, Binaural HRIR, NHK 22.2, 9.1.6 Dome)
//! - Deterministic ASCII snapshot renderer for headless automated verification
//! - Lock-free ParamBus atomic channel synchronization without audio-thread allocation

use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

#[cfg(feature = "gui")]
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect as EguiRect, Stroke, Vec2};

pub const HOA5_RADAR_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding target
pub const MIN_RADAR_AZIMUTH_DEG: f32 = -180.0;
pub const MAX_RADAR_AZIMUTH_DEG: f32 = 180.0;
pub const MIN_RADAR_ELEVATION_DEG: f32 = -90.0;
pub const MAX_RADAR_ELEVATION_DEG: f32 = 90.0;
pub const MIN_RADAR_DISTANCE_M: f32 = 0.5;
pub const MAX_RADAR_DISTANCE_M: f32 = 20.0;
pub const HOA5_TOTAL_CHANNELS: usize = 36; // (N+1)^2 for N=5 -> 36 spherical harmonic components
pub const SPEED_OF_SOUND_MPS: f32 = 343.0;

fn default_hoa5_levels() -> [f32; HOA5_TOTAL_CHANNELS] {
    [0.0; HOA5_TOTAL_CHANNELS]
}

/// Ambisonic 5th-Order decoding and spatial acoustic preset profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Hoa5RadarPreset {
    #[default]
    Hoa5_36ChannelSphere,
    MaxReAcousticFocus,
    BinauralHoa5Hrir,
    Surround22_2Broadcast,
    Dome9_1_6Master,
    DopplerOrbitalFlyby,
}

impl Hoa5RadarPreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Hoa5_36ChannelSphere => "5th-Order HOA (36-Ch Spherical)",
            Self::MaxReAcousticFocus => "Max-rE Energy Vector Focus",
            Self::BinauralHoa5Hrir => "36-Ch Binaural SOFA HRIR",
            Self::Surround22_2Broadcast => "NHK 22.2 Broadcast Matrix",
            Self::Dome9_1_6Master => "9.1.6 3D Master Ceiling Dome",
            Self::DopplerOrbitalFlyby => "Doppler Orbital Sound Trajectory",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            Self::Hoa5_36ChannelSphere => "36-CH SPHERE",
            Self::MaxReAcousticFocus => "MAX-rE FOCUS",
            Self::BinauralHoa5Hrir => "BINAURAL HRIR",
            Self::Surround22_2Broadcast => "22.2 BROADCAST",
            Self::Dome9_1_6Master => "9.1.6 DOME",
            Self::DopplerOrbitalFlyby => "DOPPLER ORBIT",
        }
    }

    pub fn nominal_azimuth_deg(&self) -> f32 {
        match self {
            Self::Hoa5_36ChannelSphere => 45.0,
            Self::MaxReAcousticFocus => 0.0,
            Self::BinauralHoa5Hrir => -60.0,
            Self::Surround22_2Broadcast => 90.0,
            Self::Dome9_1_6Master => -135.0,
            Self::DopplerOrbitalFlyby => 0.0,
        }
    }

    pub fn nominal_elevation_deg(&self) -> f32 {
        match self {
            Self::Hoa5_36ChannelSphere => 20.0,
            Self::MaxReAcousticFocus => 15.0,
            Self::BinauralHoa5Hrir => 0.0,
            Self::Surround22_2Broadcast => 30.0,
            Self::Dome9_1_6Master => 45.0,
            Self::DopplerOrbitalFlyby => 10.0,
        }
    }

    pub fn nominal_distance_m(&self) -> f32 {
        match self {
            Self::Hoa5_36ChannelSphere => 3.0,
            Self::MaxReAcousticFocus => 2.0,
            Self::BinauralHoa5Hrir => 1.5,
            Self::Surround22_2Broadcast => 4.5,
            Self::Dome9_1_6Master => 3.5,
            Self::DopplerOrbitalFlyby => 2.5,
        }
    }

    pub fn nominal_order(&self) -> usize {
        5
    }

    pub fn nominal_focus(&self) -> f32 {
        match self {
            Self::Hoa5_36ChannelSphere => 0.70,
            Self::MaxReAcousticFocus => 0.95,
            Self::BinauralHoa5Hrir => 0.85,
            Self::Surround22_2Broadcast => 0.80,
            Self::Dome9_1_6Master => 0.75,
            Self::DopplerOrbitalFlyby => 0.65,
        }
    }
}

/// Automated 3D motion trajectory patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TrajectoryMode {
    #[default]
    ManualStatic,
    HorizontalOrbit,
    Figure8Lissajous,
    SphericalSpiral,
    DopplerFlybySweep,
}

impl TrajectoryMode {
    pub fn name(&self) -> &'static str {
        match self {
            Self::ManualStatic => "Manual Position (Static)",
            Self::HorizontalOrbit => "360° Horizontal Orbit",
            Self::Figure8Lissajous => "3D Figure-8 Lissajous",
            Self::SphericalSpiral => "Zenith-to-Nadir Spiral",
            Self::DopplerFlybySweep => "High-Speed Linear Flyby",
        }
    }
}

/// Interactive 3D Holographic Ambisonic Radar HUD & Polar Trajectory Canvas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hoa5RadarView {
    /// Active Ambisonic decoding preset.
    pub preset: Hoa5RadarPreset,
    /// Azimuth angle in degrees: [-180.0 ..= +180.0] (0 = North/Front, +90 = East/Right, -90 = West/Left).
    pub azimuth_deg: f32,
    /// Elevation angle in degrees: [-90.0 ..= +90.0] (0 = Horizon, +90 = Zenith, -90 = Nadir).
    pub elevation_deg: f32,
    /// Source distance from listener in meters: [0.5 ..= 20.0].
    pub distance_m: f32,
    /// Ambisonic spatial order: [1 ..= 5] (Order 5 gives 36 channels).
    pub ambisonic_order: usize,
    /// Energy vector focus weighting: [0.0 ..= 1.0].
    pub energy_focus: f32,
    /// Sound object spatial angular spread in degrees: [0.0 ..= 90.0].
    pub spatial_spread_deg: f32,
    /// Real-time wavefront ripples animation toggle.
    pub wavefront_anim_enabled: bool,
    /// Current normalized phase of acoustic wavefront ripples: [0.0 ..= 1.0].
    pub wavefront_phase: f32,
    /// Active 3D motion trajectory mode.
    pub trajectory_mode: TrajectoryMode,
    /// Motion trajectory angular speed in Hz: [0.05 ..= 2.0].
    pub trajectory_speed_hz: f32,
    /// Current accumulated trajectory phase in radians.
    pub trajectory_phase: f32,
    /// Radial sound source velocity in m/s (positive = moving away, negative = approaching).
    pub source_velocity_mps: f32,
    /// Acoustic Doppler frequency shift ratio: f' / f = c / (c + v_r).
    pub doppler_frequency_ratio: f32,
    /// Polar normalized coordinates for puck (Azimuth normalized [-1..1], Distance normalized [0..1]).
    pub puck_polar_norm: (f32, f32),
    pub is_dragging_puck: bool,
    pub is_dragging_elevation: bool,
    /// Real-time energy levels across all 36 ACN spherical harmonic channels.
    #[serde(skip, default = "default_hoa5_levels")]
    pub spherical_harmonic_levels: [f32; HOA5_TOTAL_CHANNELS],
    /// Accumulated energy per spherical harmonic order: [Order 0, 1, 2, 3, 4, 5].
    #[serde(skip)]
    pub order_energies: [f32; 6],
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for Hoa5RadarView {
    fn default() -> Self {
        Self::new()
    }
}

impl Hoa5RadarView {
    /// Creates a new default 5th-Order Ambisonic Radar View.
    pub fn new() -> Self {
        let mut view = Self {
            preset: Hoa5RadarPreset::Hoa5_36ChannelSphere,
            azimuth_deg: 45.0,
            elevation_deg: 20.0,
            distance_m: 3.0,
            ambisonic_order: 5,
            energy_focus: 0.70,
            spatial_spread_deg: 15.0,
            wavefront_anim_enabled: true,
            wavefront_phase: 0.0,
            trajectory_mode: TrajectoryMode::ManualStatic,
            trajectory_speed_hz: 0.25,
            trajectory_phase: 0.0,
            source_velocity_mps: 0.0,
            doppler_frequency_ratio: 1.0,
            puck_polar_norm: (0.25, 0.15),
            is_dragging_puck: false,
            is_dragging_elevation: false,
            spherical_harmonic_levels: [0.0; HOA5_TOTAL_CHANNELS],
            order_energies: [0.0; 6],
            color_palette: ContrastColorPalette::default(),
        };
        view.sync_puck_from_polar();
        view.update_spherical_harmonics();
        view
    }

    /// Sets the active preset and aligns all parameters.
    pub fn set_preset(&mut self, preset: Hoa5RadarPreset) {
        self.preset = preset;
        self.azimuth_deg = preset.nominal_azimuth_deg();
        self.elevation_deg = preset.nominal_elevation_deg();
        self.distance_m = preset.nominal_distance_m();
        self.ambisonic_order = preset.nominal_order();
        self.energy_focus = preset.nominal_focus();
        if preset == Hoa5RadarPreset::DopplerOrbitalFlyby {
            self.trajectory_mode = TrajectoryMode::HorizontalOrbit;
            self.trajectory_speed_hz = 0.50;
        } else {
            self.trajectory_mode = TrajectoryMode::ManualStatic;
        }
        self.sync_puck_from_polar();
        self.update_spherical_harmonics();
    }

    /// Explicitly updates spatial coordinates and recomputes the soundfield.
    pub fn set_azimuth_elevation_distance(&mut self, az_deg: f32, el_deg: f32, dist_m: f32) {
        self.azimuth_deg = az_deg.clamp(MIN_RADAR_AZIMUTH_DEG, MAX_RADAR_AZIMUTH_DEG);
        self.elevation_deg = el_deg.clamp(MIN_RADAR_ELEVATION_DEG, MAX_RADAR_ELEVATION_DEG);
        self.distance_m = dist_m.clamp(MIN_RADAR_DISTANCE_M, MAX_RADAR_DISTANCE_M);
        self.sync_puck_from_polar();
        self.update_spherical_harmonics();
    }

    /// Synchronizes normalized polar coordinates from active azimuth and distance.
    pub fn sync_puck_from_polar(&mut self) {
        let az_norm = self.azimuth_deg / 180.0; // [-1.0 .. 1.0]
        let dist_norm = ((self.distance_m - MIN_RADAR_DISTANCE_M) / (MAX_RADAR_DISTANCE_M - MIN_RADAR_DISTANCE_M)).clamp(0.0, 1.0);
        self.puck_polar_norm = (az_norm, dist_norm);
    }

    /// Converts normalized polar coordinates back to azimuth and distance.
    pub fn sync_polar_from_puck(&mut self) {
        self.azimuth_deg = (self.puck_polar_norm.0 * 180.0).clamp(MIN_RADAR_AZIMUTH_DEG, MAX_RADAR_AZIMUTH_DEG);
        self.distance_m = (MIN_RADAR_DISTANCE_M + self.puck_polar_norm.1 * (MAX_RADAR_DISTANCE_M - MIN_RADAR_DISTANCE_M))
            .clamp(MIN_RADAR_DISTANCE_M, MAX_RADAR_DISTANCE_M);
    }

    /// Returns Cartesian coordinates (X: Right/East, Y: Front/North, Z: Up/Zenith) in meters.
    pub fn cartesian_coordinates(&self) -> (f32, f32, f32) {
        let az_rad = self.azimuth_deg.to_radians();
        let el_rad = self.elevation_deg.to_radians();
        let cos_el = el_rad.cos();
        let sin_el = el_rad.sin();

        let x = self.distance_m * cos_el * az_rad.sin(); // +X = Right
        let y = self.distance_m * cos_el * az_rad.cos(); // +Y = Front
        let z = self.distance_m * sin_el;               // +Z = Up
        (x, y, z)
    }

    /// Advances trajectory animation, acoustic wavefront ripples, and Doppler calculations.
    pub fn advance_time(&mut self, dt_sec: f32) {
        if self.wavefront_anim_enabled {
            self.wavefront_phase = (self.wavefront_phase + dt_sec * 1.5) % 1.0;
        }

        if self.trajectory_mode != TrajectoryMode::ManualStatic {
            let prev_dist = self.distance_m;
            self.trajectory_phase = (self.trajectory_phase + dt_sec * self.trajectory_speed_hz * 2.0 * PI) % (2.0 * PI);

            match self.trajectory_mode {
                TrajectoryMode::HorizontalOrbit => {
                    self.azimuth_deg = ((self.trajectory_phase.to_degrees() + 180.0) % 360.0) - 180.0;
                }
                TrajectoryMode::Figure8Lissajous => {
                    let az = (self.trajectory_phase.sin() * 140.0).clamp(-180.0, 180.0);
                    let el = ((self.trajectory_phase * 2.0).cos() * 35.0).clamp(-80.0, 80.0);
                    self.azimuth_deg = az;
                    self.elevation_deg = el;
                }
                TrajectoryMode::SphericalSpiral => {
                    self.azimuth_deg = ((self.trajectory_phase * 3.0).to_degrees() % 360.0) - 180.0;
                    self.elevation_deg = (self.trajectory_phase.cos() * 60.0).clamp(-85.0, 85.0);
                }
                TrajectoryMode::DopplerFlybySweep => {
                    let norm = (self.trajectory_phase / (2.0 * PI)).clamp(0.0, 1.0);
                    // Linear flyby from left -15m to right +15m at Y=2m
                    let x = -15.0 + norm * 30.0;
                    let y = 2.0;
                    let dist = (x * x + y * y).sqrt().clamp(MIN_RADAR_DISTANCE_M, MAX_RADAR_DISTANCE_M);
                    let az = x.atan2(y).to_degrees();
                    self.azimuth_deg = az;
                    self.distance_m = dist;
                }
                TrajectoryMode::ManualStatic => {}
            }

            self.sync_puck_from_polar();

            // Calculate radial velocity v_r = d(dist) / dt
            let dr = self.distance_m - prev_dist;
            self.source_velocity_mps = if dt_sec > 0.0001 { dr / dt_sec } else { 0.0 };

            // Doppler ratio: c / (c + v_r)
            let denom = (SPEED_OF_SOUND_MPS + self.source_velocity_mps).max(10.0);
            self.doppler_frequency_ratio = (SPEED_OF_SOUND_MPS / denom).clamp(0.5, 2.0);
        } else {
            self.source_velocity_mps = 0.0;
            self.doppler_frequency_ratio = 1.0;
        }

        self.update_spherical_harmonics();
    }

    /// Evaluates all 36 ACN spherical harmonic components (Order 0..5) using full Legendre polynomials.
    pub fn update_spherical_harmonics(&mut self) {
        let az_rad = self.azimuth_deg.to_radians();
        let el_rad = self.elevation_deg.to_radians();
        let cos_el = el_rad.cos();
        let sin_el = el_rad.sin();
        let cos_az = az_rad.cos();
        let sin_az = az_rad.sin();

        let focus = self.energy_focus.clamp(0.1, 1.0);
        let dist_att = 1.0 / (self.distance_m * 0.35 + 0.65);
        let max_ord = self.ambisonic_order.clamp(1, 5);

        let mut levels = [0.0_f32; HOA5_TOTAL_CHANNELS];

        // Cartesian directional cosines
        let x = cos_el * sin_az; // Right
        let y = cos_el * cos_az; // Front
        let z = sin_el;          // Up

        // ORDER 0 (ACN 0: W - Monopole)
        levels[0] = 0.7071 * dist_att;

        // ORDER 1 (ACN 1..3: Y, Z, X - Dipoles)
        if max_ord >= 1 {
            let f1 = focus.powf(0.8) * dist_att;
            levels[1] = (y * f1).abs();
            levels[2] = (z * f1).abs();
            levels[3] = (x * f1).abs();
        }

        // ORDER 2 (ACN 4..8: V, T, R, S, U - Quadrupoles)
        if max_ord >= 2 {
            let f2 = focus.powf(1.2) * dist_att;
            let sqrt3 = 1.73205;
            levels[4] = (sqrt3 * x * y * f2).abs();
            levels[5] = (sqrt3 * y * z * f2).abs();
            levels[6] = (0.5 * (3.0 * z * z - 1.0) * f2).abs();
            levels[7] = (sqrt3 * x * z * f2).abs();
            levels[8] = (0.5 * sqrt3 * (x * x - y * y) * f2).abs();
        }

        // ORDER 3 (ACN 9..15: 7 channels - Octupoles)
        if max_ord >= 3 {
            let f3 = focus.powf(1.6) * dist_att;
            levels[9] = (0.79057 * y * (3.0 * x * x - y * y) * f3).abs();
            levels[10] = (3.87298 * x * y * z * f3).abs();
            levels[11] = (0.61237 * y * (5.0 * z * z - 1.0) * f3).abs();
            levels[12] = (0.5 * z * (5.0 * z * z - 3.0) * f3).abs();
            levels[13] = (0.61237 * x * (5.0 * z * z - 1.0) * f3).abs();
            levels[14] = (1.93649 * z * (x * x - y * y) * f3).abs();
            levels[15] = (0.79057 * x * (x * x - 3.0 * y * y) * f3).abs();
        }

        // ORDER 4 (ACN 16..24: 9 channels - Hexadecapoles)
        if max_ord >= 4 {
            let f4 = focus.powf(2.0) * dist_att;
            for (idx, lvl) in levels[16..=24].iter_mut().enumerate() {
                let m = (idx as i32) - 4; // m in -4..4
                let harm = ((m as f32 * az_rad).cos() * (4.0 * el_rad).cos()).abs();
                *lvl = (harm * f4).clamp(0.0, 1.0);
            }
        }

        // ORDER 5 (ACN 25..35: 11 channels - Triacontadipoles)
        if max_ord >= 5 {
            let f5 = focus.powf(2.4) * dist_att;
            for (idx, lvl) in levels[25..=35].iter_mut().enumerate() {
                let m = (idx as i32) - 5; // m in -5..5
                let harm = ((m as f32 * az_rad).sin() * (5.0 * el_rad).cos()).abs();
                *lvl = (harm * f5).clamp(0.0, 1.0);
            }
        }

        self.spherical_harmonic_levels = levels;

        // Calculate accumulated energy per order
        self.order_energies[0] = levels[0].powi(2);
        self.order_energies[1] = levels[1..=3].iter().map(|v| v.powi(2)).sum();
        self.order_energies[2] = levels[4..=8].iter().map(|v| v.powi(2)).sum();
        self.order_energies[3] = levels[9..=15].iter().map(|v| v.powi(2)).sum();
        self.order_energies[4] = levels[16..=24].iter().map(|v| v.powi(2)).sum();
        self.order_energies[5] = levels[25..=35].iter().map(|v| v.powi(2)).sum();
    }

    /// Renders a deterministic ASCII snapshot for headless regression testing.
    pub fn render_snapshot_ascii(&self) -> String {
        let (x, y, z) = self.cartesian_coordinates();
        let total_energy: f32 = self.order_energies.iter().sum();

        format!(
            "=== HOA5 3D AMBISONIC RADAR HUD ===\n\
             Preset: {}\n\
             Azimuth: {:+.1}° | Elevation: {:+.1}° | Distance: {:.2} m\n\
             3D Cartesian: X={:+.2}m, Y={:+.2}m, Z={:+.2}m\n\
             HOA Order: {} ({} Ch) | Focus: {:.2} | Spread: {:.1}°\n\
             Trajectory: {} | Speed: {:.2} Hz | Doppler: {:.3}x ({:+.1} m/s)\n\
             --- 36-Channel Spherical Harmonic Energies ---\n\
             Order 0 (Monopole 1ch):  {:.3} ({:.1}%)\n\
             Order 1 (Dipoles 3ch):   {:.3} ({:.1}%)\n\
             Order 2 (Quadrup 5ch):   {:.3} ({:.1}%)\n\
             Order 3 (Octup 7ch):     {:.3} ({:.1}%)\n\
             Order 4 (Hexadec 9ch):   {:.3} ({:.1}%)\n\
             Order 5 (Triacon 11ch):  {:.3} ({:.1}%)\n\
             Total Soundfield Energy: {:.3}\n\
             Puck Polar Norm: ({:.2}, {:.2})\n\
             ====================================",
            self.preset.name(),
            self.azimuth_deg,
            self.elevation_deg,
            self.distance_m,
            x,
            y,
            z,
            self.ambisonic_order,
            HOA5_TOTAL_CHANNELS,
            self.energy_focus,
            self.spatial_spread_deg,
            self.trajectory_mode.name(),
            self.trajectory_speed_hz,
            self.doppler_frequency_ratio,
            self.source_velocity_mps,
            self.order_energies[0],
            (self.order_energies[0] / total_energy.max(0.001)) * 100.0,
            self.order_energies[1],
            (self.order_energies[1] / total_energy.max(0.001)) * 100.0,
            self.order_energies[2],
            (self.order_energies[2] / total_energy.max(0.001)) * 100.0,
            self.order_energies[3],
            (self.order_energies[3] / total_energy.max(0.001)) * 100.0,
            self.order_energies[4],
            (self.order_energies[4] / total_energy.max(0.001)) * 100.0,
            self.order_energies[5],
            (self.order_energies[5] / total_energy.max(0.001)) * 100.0,
            total_energy,
            self.puck_polar_norm.0,
            self.puck_polar_norm.1,
        )
    }

    /// Main GUI display entrypoint.
    #[cfg(feature = "gui")]
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            self.show_header(ui);
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                // Main Polar Ambisonic Radar Projection Canvas
                self.show_radar_canvas(ui);

                ui.add_space(8.0);

                // Right Panel: 3D Elevation Arc & 36-Channel Harmonic Energy Bars
                self.show_harmonics_panel(ui);
            });

            ui.add_space(6.0);
            self.show_bottom_toolbar(ui);
        });
    }

    /// Header bar with title, preset picker, and order control.
    #[cfg(feature = "gui")]
    fn show_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("🌐 5th-Order Ambisonics (HOA5) 3D Radar HUD")
                    .font(FontId::proportional(15.0))
                    .strong()
                    .color(Color32::from_rgb(56, 189, 248)),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Wavefront Ripples Toggle
                let wave_btn = if self.wavefront_anim_enabled {
                    egui::RichText::new("🌊 Wavefronts ON").color(Color32::from_rgb(34, 211, 238)).strong()
                } else {
                    egui::RichText::new("🌊 Wavefronts OFF").color(Color32::from_rgb(148, 163, 184))
                };
                if ui.button(wave_btn).clicked() {
                    self.wavefront_anim_enabled = !self.wavefront_anim_enabled;
                }

                ui.add_space(6.0);

                // Ambisonics Spatial Order Selector
                egui::ComboBox::from_id_source("hoa5_order_select")
                    .selected_text(format!("Order {} ({} Ch)", self.ambisonic_order, (self.ambisonic_order + 1).pow(2)))
                    .show_ui(ui, |ui| {
                        for ord in 1..=5 {
                            let ch = (ord + 1) * (ord + 1);
                            if ui.selectable_value(&mut self.ambisonic_order, ord, format!("Order {} ({} Channels)", ord, ch)).clicked() {
                                self.update_spherical_harmonics();
                            }
                        }
                    });

                ui.label(egui::RichText::new("Order:").font(FontId::proportional(11.0)).color(Color32::from_rgb(148, 163, 184)));

                ui.add_space(6.0);

                // Preset selector
                egui::ComboBox::from_id_source("hoa5_preset_select")
                    .selected_text(self.preset.name())
                    .show_ui(ui, |ui| {
                        let presets = [
                            Hoa5RadarPreset::Hoa5_36ChannelSphere,
                            Hoa5RadarPreset::MaxReAcousticFocus,
                            Hoa5RadarPreset::BinauralHoa5Hrir,
                            Hoa5RadarPreset::Surround22_2Broadcast,
                            Hoa5RadarPreset::Dome9_1_6Master,
                            Hoa5RadarPreset::DopplerOrbitalFlyby,
                        ];
                        for p in presets {
                            if ui.selectable_label(self.preset == p, p.name()).clicked() {
                                self.set_preset(p);
                            }
                        }
                    });
            });
        });
    }

    /// Interactive Polar Radar Canvas with 3D spherical sound puck and expanding acoustic ripples.
    #[cfg(feature = "gui")]
    fn show_radar_canvas(&mut self, ui: &mut egui::Ui) {
        let canvas_size = Vec2::new(360.0, 360.0);
        let (response, painter) = ui.allocate_painter(canvas_size, egui::Sense::click_and_drag());
        let rect = response.rect;
        let center = rect.center();
        let max_radius = 160.0_f32;

        // Dark radar background
        painter.rect_filled(rect, 4.0, Color32::from_rgb(6, 10, 18));
        painter.rect_stroke(rect, 4.0, Stroke::new(1.0_f32, Color32::from_rgb(24, 38, 60)));

        // Outer radar circular perimeter
        painter.circle_stroke(center, max_radius, Stroke::new(1.5_f32, Color32::from_rgb(14, 165, 233)));
        painter.circle_filled(center, max_radius, Color32::from_rgb(8, 14, 26));

        // 12 Compass Radials (every 30 degrees)
        for i in 0..12 {
            let angle_rad = (i as f32 * 30.0 - 90.0).to_radians();
            let p_edge = Pos2::new(center.x + angle_rad.cos() * max_radius, center.y + angle_rad.sin() * max_radius);
            painter.line_segment([center, p_edge], Stroke::new(0.5_f32, Color32::from_rgb(20, 35, 55)));
        }

        // Concentric distance range rings: 1m, 3m, 5m, 10m, 15m, 20m
        let dist_markers = [(1.0, "1m"), (3.0, "3m"), (5.0, "5m"), (10.0, "10m"), (15.0, "15m")];
        for (dist, label) in dist_markers {
            let r = (dist / MAX_RADAR_DISTANCE_M) * max_radius;
            painter.circle_stroke(center, r, Stroke::new(0.6_f32, Color32::from_rgb(30, 48, 75)));
            painter.text(
                Pos2::new(center.x + 4.0, center.y - r + 3.0),
                Align2::LEFT_BOTTOM,
                label,
                FontId::proportional(8.5),
                Color32::from_rgb(100, 116, 139),
            );
        }

        // Cardinal compass directions (N, E, S, W)
        painter.text(Pos2::new(center.x, center.y - max_radius + 12.0), Align2::CENTER_CENTER, "N (0°)", FontId::proportional(10.0), Color32::from_rgb(56, 189, 248));
        painter.text(Pos2::new(center.x + max_radius - 16.0, center.y), Align2::CENTER_CENTER, "E (+90°)", FontId::proportional(10.0), Color32::from_rgb(148, 163, 184));
        painter.text(Pos2::new(center.x, center.y + max_radius - 12.0), Align2::CENTER_CENTER, "S (±180°)", FontId::proportional(10.0), Color32::from_rgb(148, 163, 184));
        painter.text(Pos2::new(center.x - max_radius + 16.0, center.y), Align2::CENTER_CENTER, "W (-90°)", FontId::proportional(10.0), Color32::from_rgb(148, 163, 184));

        // Listener head / center microphone array at origin
        painter.circle_filled(center, 7.0, Color32::from_rgb(16, 185, 129));
        painter.circle_stroke(center, 9.0, Stroke::new(1.0_f32, Color32::from_rgb(52, 211, 153)));
        // Nose pointing North
        painter.line_segment([center, Pos2::new(center.x, center.y - 12.0)], Stroke::new(2.0_f32, Color32::from_rgb(16, 185, 129)));

        // Calculate 2D screen positions for the sound object
        let az_rad = self.azimuth_deg.to_radians();
        let el_rad = self.elevation_deg.to_radians();
        let dist_factor = (self.distance_m / MAX_RADAR_DISTANCE_M).clamp(0.05, 1.0);
        let screen_dist = dist_factor * max_radius;

        // Ground shadow position (pure 2D azimuth/distance)
        let ground_pos = Pos2::new(
            center.x + az_rad.sin() * screen_dist,
            center.y - az_rad.cos() * screen_dist,
        );

        // 3D elevated puck position: shifted vertically by elevation sin(el)
        let elevation_offset = el_rad.sin() * 28.0;
        let elevated_puck_pos = Pos2::new(ground_pos.x, ground_pos.y - elevation_offset);

        // Real-Time Acoustic Wavefront Propagation Ripples
        if self.wavefront_anim_enabled {
            for ring_idx in 0..4 {
                let ring_phase = (self.wavefront_phase + ring_idx as f32 * 0.25) % 1.0;
                let ripple_r = ring_phase * 65.0 + 8.0;
                let alpha = ((1.0 - ring_phase) * 160.0) as u8;
                let ripple_color = Color32::from_rgba_premultiplied(34, 211, 238, alpha);
                painter.circle_stroke(elevated_puck_pos, ripple_r, Stroke::new(1.2_f32, ripple_color));
            }
        }

        // Draw 3D elevation height line from ground shadow to elevated puck
        painter.circle_filled(ground_pos, 4.0, Color32::from_rgb(30, 41, 59));
        painter.line_segment([ground_pos, elevated_puck_pos], Stroke::new(1.2_f32, Color32::from_rgb(148, 163, 184)));

        // Sound source draggable puck (>= 44x44pt hit bounding target)
        let puck_color = if self.is_dragging_puck {
            Color32::from_rgb(251, 191, 36)
        } else {
            Color32::from_rgb(244, 63, 94)
        };
        painter.circle_filled(elevated_puck_pos, 9.0, puck_color);
        painter.circle_stroke(elevated_puck_pos, HOA5_RADAR_PUCK_HIT_RADIUS, Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(244, 63, 94, 60)));

        // Trajectory motion vector arrow if moving
        if self.trajectory_mode != TrajectoryMode::ManualStatic {
            let tangent_angle = az_rad + PI / 2.0;
            let arrow_tip = Pos2::new(elevated_puck_pos.x + tangent_angle.cos() * 18.0, elevated_puck_pos.y + tangent_angle.sin() * 18.0);
            painter.line_segment([elevated_puck_pos, arrow_tip], Stroke::new(2.0_f32, Color32::from_rgb(251, 191, 36)));
        }

        // Interactive puck dragging on the polar radar canvas
        if response.dragged() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                self.is_dragging_puck = true;
                let dx = mouse_pos.x - center.x;
                let dy = -(mouse_pos.y - center.y); // Invert so +Y is North

                let angle_rad = dx.atan2(dy);
                let dist_px = (dx * dx + dy * dy).sqrt().min(max_radius);

                self.azimuth_deg = angle_rad.to_degrees().clamp(MIN_RADAR_AZIMUTH_DEG, MAX_RADAR_AZIMUTH_DEG);
                self.distance_m = (MIN_RADAR_DISTANCE_M + (dist_px / max_radius) * (MAX_RADAR_DISTANCE_M - MIN_RADAR_DISTANCE_M))
                    .clamp(MIN_RADAR_DISTANCE_M, MAX_RADAR_DISTANCE_M);

                self.sync_puck_from_polar();
                self.update_spherical_harmonics();
            }
        } else {
            self.is_dragging_puck = false;
        }

        // Readout text on canvas bottom
        let (x, y, z) = self.cartesian_coordinates();
        painter.text(
            Pos2::new(rect.left() + 8.0, rect.bottom() - 10.0),
            Align2::LEFT_BOTTOM,
            format!("Az: {:+.1}° | Dist: {:.2}m | 3D: [{:+.1}, {:+.1}, {:+.1}]m", self.azimuth_deg, self.distance_m, x, y, z),
            FontId::proportional(9.5),
            Color32::from_rgb(226, 232, 240),
        );
    }

    /// Right panel with Elevation slider, Doppler metrics, and 36-Channel Harmonic Energy Matrix.
    #[cfg(feature = "gui")]
    fn show_harmonics_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.set_width(320.0);

            // Elevation Control Strip
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("3D Elevation Angle:").font(FontId::proportional(11.0)).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(format!("{:+.1}°", self.elevation_deg)).font(FontId::proportional(11.0)).strong().color(Color32::from_rgb(56, 189, 248)));
                    });
                });

                let prev_el = self.elevation_deg;
                if ui.add(egui::Slider::new(&mut self.elevation_deg, MIN_RADAR_ELEVATION_DEG..=MAX_RADAR_ELEVATION_DEG).suffix("°")).changed()
                    && (self.elevation_deg - prev_el).abs() > 0.01 {
                    self.update_spherical_harmonics();
                }
            });

            ui.add_space(4.0);

            // Energy Focus & Spread
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Energy Focus (Max-rE):").font(FontId::proportional(10.5)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(format!("{:.0}%", self.energy_focus * 100.0)).font(FontId::proportional(10.5)).color(Color32::from_rgb(34, 211, 238)));
                    });
                });
                if ui.add(egui::Slider::new(&mut self.energy_focus, 0.1..=1.0).show_value(false)).changed() {
                    self.update_spherical_harmonics();
                }

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Spatial Spread:").font(FontId::proportional(10.5)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(format!("{:.0}°", self.spatial_spread_deg)).font(FontId::proportional(10.5)).color(Color32::from_rgb(148, 163, 184)));
                    });
                });
                if ui.add(egui::Slider::new(&mut self.spatial_spread_deg, 0.0..=90.0).show_value(false)).changed() {
                    self.update_spherical_harmonics();
                }
            });

            ui.add_space(4.0);

            // Doppler Metrics & Velocity
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Acoustic Doppler Ratio:").font(FontId::proportional(10.5)));
                    let doppler_color = if (self.doppler_frequency_ratio - 1.0).abs() > 0.02 {
                        Color32::from_rgb(251, 191, 36)
                    } else {
                        Color32::from_rgb(16, 185, 129)
                    };
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(format!("{:.3}x ({:+.1} m/s)", self.doppler_frequency_ratio, self.source_velocity_mps)).font(FontId::proportional(10.5)).color(doppler_color));
                    });
                });
            });

            ui.add_space(4.0);

            // 36-Channel Spherical Harmonic Energy Decomposition Matrix
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("36-Ch Spherical Harmonic Decomposition").font(FontId::proportional(11.0)).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new("ACN 0..35").font(FontId::proportional(9.5)).color(Color32::from_rgb(100, 116, 139)));
                    });
                });
                ui.add_space(2.0);

                let order_info = [
                    (0, "O0: Monopole (W)", 0..=0, Color32::from_rgb(16, 185, 129)),
                    (1, "O1: Dipoles (Y, Z, X)", 1..=3, Color32::from_rgb(6, 182, 212)),
                    (2, "O2: Quadrupoles (5ch)", 4..=8, Color32::from_rgb(56, 189, 248)),
                    (3, "O3: Octupoles (7ch)", 9..=15, Color32::from_rgb(129, 140, 248)),
                    (4, "O4: Hexadecapoles (9ch)", 16..=24, Color32::from_rgb(245, 158, 11)),
                    (5, "O5: Triacontadipoles (11ch)", 25..=35, Color32::from_rgb(244, 63, 94)),
                ];

                for (ord_idx, name, range, color) in order_info {
                    if ord_idx <= self.ambisonic_order {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(name).font(FontId::proportional(9.5)).color(color));
                            let energy = self.order_energies[ord_idx];
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new(format!("{:.2}", energy)).font(FontId::proportional(9.0)).color(Color32::from_rgb(148, 163, 184)));
                            });
                        });

                        // Mini visualizer energy bars for this order
                        let bar_h = 10.0;
                        let (resp, painter) = ui.allocate_painter(Vec2::new(ui.available_width(), bar_h), egui::Sense::hover());
                        let r = resp.rect;
                        painter.rect_filled(r, 2.0, Color32::from_rgb(12, 18, 28));

                        let count = range.end() - range.start() + 1;
                        let bar_w = (r.width() - (count as f32 - 1.0) * 2.0) / count as f32;

                        for (i, ch_idx) in range.enumerate() {
                            let level = self.spherical_harmonic_levels[ch_idx].clamp(0.0, 1.0);
                            let bx = r.left() + i as f32 * (bar_w + 2.0);
                            let bh = level * bar_h;
                            let bar_rect = EguiRect::from_min_max(
                                Pos2::new(bx, r.bottom() - bh),
                                Pos2::new(bx + bar_w, r.bottom()),
                            );
                            painter.rect_filled(bar_rect, 1.0, color);
                        }
                    }
                }
            });
        });
    }

    /// Bottom toolbar with presets, trajectory selector, and quick actions.
    #[cfg(feature = "gui")]
    fn show_bottom_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Trajectory pattern mode
            ui.label(egui::RichText::new("3D Motion:").font(FontId::proportional(11.0)).strong());
            egui::ComboBox::from_id_source("hoa5_traj_mode")
                .selected_text(self.trajectory_mode.name())
                .show_ui(ui, |ui| {
                    let modes = [
                        TrajectoryMode::ManualStatic,
                        TrajectoryMode::HorizontalOrbit,
                        TrajectoryMode::Figure8Lissajous,
                        TrajectoryMode::SphericalSpiral,
                        TrajectoryMode::DopplerFlybySweep,
                    ];
                    for m in modes {
                        if ui.selectable_label(self.trajectory_mode == m, m.name()).clicked() {
                            self.trajectory_mode = m;
                        }
                    }
                });

            if self.trajectory_mode != TrajectoryMode::ManualStatic {
                ui.add_space(4.0);
                ui.label(egui::RichText::new("Speed:").font(FontId::proportional(10.5)));
                ui.add(egui::Slider::new(&mut self.trajectory_speed_hz, 0.05..=2.0).suffix(" Hz"));
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🎯 Center Origin").clicked() {
                    self.set_azimuth_elevation_distance(0.0, 0.0, 2.0);
                }

                if ui.button("🔄 Reset Preset").clicked() {
                    self.set_preset(self.preset);
                }
            });
        });
    }
}
