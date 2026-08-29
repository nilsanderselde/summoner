// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Spatial Trajectory Path Interpolator & Dynamic 3D Automation Engine (Milestone 12).
//!
//! Provides lock-free 3D position vector interpolation (Azimuth, Elevation, Distance, Spread)
//! supporting parametric geometric paths (Orbit, Lissajous3D, Spiral, Flyby) and keyframed
//! cubic Catmull-Rom splines, with full TOML serialization and sample-accurate dispatch into
//! `SpatialBus` and `ParamBus`.
//!
//! Enforces zero heap allocations in real-time execution loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::f32::consts::{PI, TAU};
use summoner_core::allocator::AllocGuard;
use summoner_core::param_bus::{ParamBus, ParamId};
use summoner_core::spatial_bus::{SpatialBus, SpatialVector3D};

/// Interpolation curve mode between discrete spatial keyframes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SpatialInterpolationCurve {
    #[default]
    Linear,
    CubicCatmullRom,
    SmoothStep,
    Hold,
}

/// Parametric trajectory path geometry generator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TrajectoryPathType {
    /// Circular or elliptical orbit around listener with inclination and speed.
    Orbit {
        radius_m: f32,
        speed_cycles_per_beat: f32,
        tilt_rad: f32,
        eccentricity: f32,
        center_x: f32,
        center_y: f32,
        center_z: f32,
    },
    /// 3D harmonic Lissajous knot / figure.
    Lissajous3D {
        freq_x: f32,
        freq_y: f32,
        freq_z: f32,
        phase_x: f32,
        phase_y: f32,
        phase_z: f32,
        scale_x: f32,
        scale_y: f32,
        scale_z: f32,
    },
    /// Inward / outward conical spiral.
    Spiral {
        start_radius_m: f32,
        end_radius_m: f32,
        turns: f32,
        height_delta_m: f32,
        base_height_m: f32,
    },
    /// Linear flyby from start coordinate to end coordinate.
    Flyby {
        start_x: f32,
        start_y: f32,
        start_z: f32,
        end_x: f32,
        end_y: f32,
        end_z: f32,
        smooth_ease: bool,
    },
    /// User-defined keyframe waypoints with cubic Catmull-Rom spline interpolation.
    WaypointSpline,
}

impl Default for TrajectoryPathType {
    fn default() -> Self {
        TrajectoryPathType::Orbit {
            radius_m: 3.0,
            speed_cycles_per_beat: 0.25,
            tilt_rad: 0.0,
            eccentricity: 0.0,
            center_x: 0.0,
            center_y: 0.0,
            center_z: 0.0,
        }
    }
}

/// A single keyframed 3D spatial waypoint.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpatialWaypoint {
    /// Timeline beat position in quarter note beats.
    pub beat: f64,
    /// Azimuth angle in radians ($-\pi$ to $+\pi$).
    pub azimuth_rad: f32,
    /// Elevation angle in radians ($-\pi/2$ to $+\pi/2$).
    pub elevation_rad: f32,
    /// Distance in meters.
    pub distance_m: f32,
    /// Spatial spread factor (0.0 to 1.0).
    pub spread: f32,
    /// Interpolation curve to next waypoint.
    pub curve: SpatialInterpolationCurve,
}

impl Default for SpatialWaypoint {
    fn default() -> Self {
        Self {
            beat: 0.0,
            azimuth_rad: 0.0,
            elevation_rad: 0.0,
            distance_m: 2.0,
            spread: 0.0,
            curve: SpatialInterpolationCurve::CubicCatmullRom,
        }
    }
}

/// A spatial trajectory track orchestrating 3D movement for a single track / audio object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpatialTrajectoryTrack {
    /// Associated track identifier.
    pub track_id: usize,
    /// Associated spatial bus object slot index (0..MAX_SPATIAL_OBJECTS).
    pub object_id: usize,
    /// Descriptive name (e.g. "Lead Vocal Orbit").
    pub name: String,
    /// Trajectory path generator type.
    pub path_type: TrajectoryPathType,
    /// Keyframed waypoints (used when `path_type` is `WaypointSpline` or as anchors).
    pub waypoints: Vec<SpatialWaypoint>,
    /// Loop cycle duration in beats (0.0 = one-shot or continuous).
    pub cycle_beats: f64,
    /// Base spread parameter.
    pub spread: f32,
    /// Enable / bypass flag.
    pub enabled: bool,
    /// Bound ParamBus IDs for direct sample-accurate parameter updates (optional).
    pub param_azimuth: Option<ParamId>,
    pub param_elevation: Option<ParamId>,
    pub param_distance: Option<ParamId>,
}

impl Default for SpatialTrajectoryTrack {
    fn default() -> Self {
        Self {
            track_id: 0,
            object_id: 0,
            name: "Spatial Track".to_string(),
            path_type: TrajectoryPathType::default(),
            waypoints: Vec::new(),
            cycle_beats: 16.0,
            spread: 0.0,
            enabled: true,
            param_azimuth: None,
            param_elevation: None,
            param_distance: None,
        }
    }
}

impl SpatialTrajectoryTrack {
    /// Creates a new spatial trajectory track.
    pub fn new(track_id: usize, object_id: usize, name: impl Into<String>) -> Self {
        Self {
            track_id,
            object_id,
            name: name.into(),
            ..Default::default()
        }
    }

    /// Evaluates the 3D spatial position vector at a given song beat position.
    pub fn evaluate_at_beat(&self, beat: f64) -> SpatialVector3D {
        if !self.enabled {
            return SpatialVector3D::default();
        }

        let effective_beat = if self.cycle_beats > 0.0 {
            beat.rem_euclid(self.cycle_beats)
        } else {
            beat
        };

        match &self.path_type {
            TrajectoryPathType::Orbit {
                radius_m,
                speed_cycles_per_beat,
                tilt_rad,
                eccentricity,
                center_x,
                center_y,
                center_z,
            } => {
                let phase = (effective_beat as f32 * speed_cycles_per_beat * TAU).rem_euclid(TAU);
                let r_x = radius_m * (1.0 + eccentricity * 0.5);
                let r_y = radius_m * (1.0 - eccentricity * 0.5);

                let local_x = r_x * phase.sin();
                let local_y = r_y * phase.cos() * tilt_rad.cos();
                let local_z = r_y * phase.cos() * tilt_rad.sin();

                let x = center_x + local_x;
                let y = center_y + local_y;
                let z = center_z + local_z;

                SpatialVector3D::from_cartesian(x, y, z, self.spread)
            }
            TrajectoryPathType::Lissajous3D {
                freq_x,
                freq_y,
                freq_z,
                phase_x,
                phase_y,
                phase_z,
                scale_x,
                scale_y,
                scale_z,
            } => {
                let t = effective_beat as f32;
                let x = scale_x * (t * freq_x * TAU + phase_x).sin();
                let y = scale_y * (t * freq_y * TAU + phase_y).cos();
                let z = scale_z * (t * freq_z * TAU + phase_z).sin();

                SpatialVector3D::from_cartesian(x, y, z, self.spread)
            }
            TrajectoryPathType::Spiral {
                start_radius_m,
                end_radius_m,
                turns,
                height_delta_m,
                base_height_m,
            } => {
                let frac = if self.cycle_beats > 0.0 {
                    (effective_beat / self.cycle_beats) as f32
                } else {
                    (effective_beat.fract()) as f32
                };
                let frac_clamped = frac.clamp(0.0, 1.0);
                let r = start_radius_m + (end_radius_m - start_radius_m) * frac_clamped;
                let angle = frac_clamped * turns * TAU;
                let z = base_height_m + height_delta_m * frac_clamped;

                let x = r * angle.sin();
                let y = r * angle.cos();

                SpatialVector3D::from_cartesian(x, y, z, self.spread)
            }
            TrajectoryPathType::Flyby {
                start_x,
                start_y,
                start_z,
                end_x,
                end_y,
                end_z,
                smooth_ease,
            } => {
                let mut frac = if self.cycle_beats > 0.0 {
                    (effective_beat / self.cycle_beats) as f32
                } else {
                    (effective_beat.fract()) as f32
                };
                frac = frac.clamp(0.0, 1.0);
                if *smooth_ease {
                    // SmoothStep S-curve: 3t^2 - 2t^3
                    frac = frac * frac * (3.0 - 2.0 * frac);
                }

                let x = start_x + (end_x - start_x) * frac;
                let y = start_y + (end_y - start_y) * frac;
                let z = start_z + (end_z - start_z) * frac;

                SpatialVector3D::from_cartesian(x, y, z, self.spread)
            }
            TrajectoryPathType::WaypointSpline => {
                self.evaluate_waypoint_spline(effective_beat)
            }
        }
    }

    /// Evaluates keyframe waypoints with Catmull-Rom cubic spline smoothing.
    fn evaluate_waypoint_spline(&self, beat: f64) -> SpatialVector3D {
        if self.waypoints.is_empty() {
            return SpatialVector3D::default();
        }

        if self.waypoints.len() == 1 {
            let wp = &self.waypoints[0];
            return SpatialVector3D::new_spherical(wp.azimuth_rad, wp.elevation_rad, wp.distance_m, wp.spread);
        }

        // Find surrounding waypoint segment [wp_idx, wp_idx + 1]
        let n = self.waypoints.len();
        let mut idx = 0;
        while idx + 1 < n && self.waypoints[idx + 1].beat <= beat {
            idx += 1;
        }

        if idx + 1 >= n {
            let wp = &self.waypoints[n - 1];
            return SpatialVector3D::new_spherical(wp.azimuth_rad, wp.elevation_rad, wp.distance_m, wp.spread);
        }

        let p1 = &self.waypoints[idx];
        let p2 = &self.waypoints[idx + 1];
        let span = (p2.beat - p1.beat).max(1e-5);
        let t = ((beat - p1.beat) / span).clamp(0.0, 1.0) as f32;

        match p1.curve {
            SpatialInterpolationCurve::Hold => {
                SpatialVector3D::new_spherical(p1.azimuth_rad, p1.elevation_rad, p1.distance_m, p1.spread)
            }
            SpatialInterpolationCurve::Linear => {
                let az = unwrap_lerp_angle(p1.azimuth_rad, p2.azimuth_rad, t);
                let el = p1.elevation_rad + (p2.elevation_rad - p1.elevation_rad) * t;
                let dist = p1.distance_m + (p2.distance_m - p1.distance_m) * t;
                let sp = p1.spread + (p2.spread - p1.spread) * t;
                SpatialVector3D::new_spherical(az, el, dist, sp)
            }
            SpatialInterpolationCurve::SmoothStep => {
                let s = t * t * (3.0 - 2.0 * t);
                let az = unwrap_lerp_angle(p1.azimuth_rad, p2.azimuth_rad, s);
                let el = p1.elevation_rad + (p2.elevation_rad - p1.elevation_rad) * s;
                let dist = p1.distance_m + (p2.distance_m - p1.distance_m) * s;
                let sp = p1.spread + (p2.spread - p1.spread) * s;
                SpatialVector3D::new_spherical(az, el, dist, sp)
            }
            SpatialInterpolationCurve::CubicCatmullRom => {
                let p0 = if idx > 0 { &self.waypoints[idx - 1] } else { p1 };
                let p3 = if idx + 2 < n { &self.waypoints[idx + 2] } else { p2 };

                // Evaluate Catmull-Rom cubic spline for Cartesian coordinates to avoid spherical singularities
                let (x0, y0, z0) = SpatialVector3D::new_spherical(p0.azimuth_rad, p0.elevation_rad, p0.distance_m, p0.spread).to_cartesian();
                let (x1, y1, z1) = SpatialVector3D::new_spherical(p1.azimuth_rad, p1.elevation_rad, p1.distance_m, p1.spread).to_cartesian();
                let (x2, y2, z2) = SpatialVector3D::new_spherical(p2.azimuth_rad, p2.elevation_rad, p2.distance_m, p2.spread).to_cartesian();
                let (x3, y3, z3) = SpatialVector3D::new_spherical(p3.azimuth_rad, p3.elevation_rad, p3.distance_m, p3.spread).to_cartesian();

                let x = catmull_rom(x0, x1, x2, x3, t);
                let y = catmull_rom(y0, y1, y2, y3, t);
                let z = catmull_rom(z0, z1, z2, z3, t);
                let sp = catmull_rom(p0.spread, p1.spread, p2.spread, p3.spread, t);

                SpatialVector3D::from_cartesian(x, y, z, sp)
            }
        }
    }
}

/// Dynamic 3D Spatial Automation Engine managing multiple trajectory tracks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SpatialAutomationEngine {
    /// Active spatial trajectory tracks.
    pub tracks: Vec<SpatialTrajectoryTrack>,
}

impl SpatialAutomationEngine {
    /// Creates a new empty spatial automation engine.
    pub fn new() -> Self {
        Self { tracks: Vec::new() }
    }

    /// Adds a spatial trajectory track.
    pub fn add_track(&mut self, track: SpatialTrajectoryTrack) {
        self.tracks.push(track);
    }

    /// Serializes this automation engine configuration to a deterministic TOML string.
    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Deserializes a spatial automation configuration from a TOML string.
    pub fn from_toml_str(toml_str: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(toml_str)
    }

    /// Evaluates all spatial tracks at current song beat and atomically dispatches 3D coordinates
    /// into `SpatialBus` and `ParamBus` handles without heap allocation.
    pub fn evaluate_and_dispatch(
        &self,
        beat: f64,
        spatial_bus: &SpatialBus,
        param_bus: &ParamBus,
    ) {
        let _guard = AllocGuard::new();
        for track in &self.tracks {
            if !track.enabled {
                continue;
            }

            let vec = track.evaluate_at_beat(beat);
            spatial_bus.set_spherical(
                track.object_id,
                vec.azimuth_rad,
                vec.elevation_rad,
                vec.distance_m,
                vec.spread,
            );

            if let (Some(p_az), Some(p_el), Some(p_dist)) = (
                track.param_azimuth,
                track.param_elevation,
                track.param_distance,
            ) {
                param_bus.set(p_az, vec.azimuth_rad);
                param_bus.set(p_el, vec.elevation_rad);
                param_bus.set(p_dist, vec.distance_m);
            }
        }
    }
}

/// Computes Catmull-Rom cubic spline interpolation between $p_1$ and $p_2$ given neighbors $p_0$ and $p_3$.
#[inline]
fn catmull_rom(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;
    0.5 * (
        (2.0 * p1)
            + (-p0 + p2) * t
            + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
            + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3
    )
}

/// Unwraps angles across the $[-\pi, +\pi]$ boundary to interpolate along the shortest circular arc.
#[inline]
fn unwrap_lerp_angle(a0: f32, a1: f32, t: f32) -> f32 {
    let diff = (a1 - a0 + PI).rem_euclid(TAU) - PI;
    a0 + diff * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orbit_trajectory_evaluation() {
        let mut track = SpatialTrajectoryTrack::new(1, 0, "Lead Synth Orbit");
        track.path_type = TrajectoryPathType::Orbit {
            radius_m: 4.0,
            speed_cycles_per_beat: 0.25, // 1 complete rotation every 4 beats
            tilt_rad: 0.0,
            eccentricity: 0.0,
            center_x: 0.0,
            center_y: 0.0,
            center_z: 0.0,
        };
        track.cycle_beats = 4.0;

        // Beat 0: Angle = 0 -> Front (x = 0, y = 4, z = 0) -> Azimuth = 0
        let v0 = track.evaluate_at_beat(0.0);
        assert!(v0.azimuth_rad.abs() < 1e-4);
        assert!((v0.distance_m - 4.0).abs() < 1e-4);

        // Beat 1: Angle = PI/2 -> Right (x = 4, y = 0, z = 0) -> Azimuth = PI/2
        let v1 = track.evaluate_at_beat(1.0);
        assert!((v1.azimuth_rad - PI / 2.0).abs() < 1e-4);
        assert!((v1.distance_m - 4.0).abs() < 1e-4);

        // Beat 2: Angle = PI -> Back (x = 0, y = -4, z = 0) -> Azimuth = PI
        let v2 = track.evaluate_at_beat(2.0);
        assert!((v2.azimuth_rad.abs() - PI).abs() < 1e-4);

        // Beat 3: Angle = 3PI/2 -> Left (x = -4, y = 0, z = 0) -> Azimuth = -PI/2
        let v3 = track.evaluate_at_beat(3.0);
        assert!((v3.azimuth_rad - (-PI / 2.0)).abs() < 1e-4);
    }

    #[test]
    fn test_waypoint_spline_catmull_rom_interpolation() {
        let mut track = SpatialTrajectoryTrack::new(2, 1, "Vocal Spline");
        track.path_type = TrajectoryPathType::WaypointSpline;
        track.waypoints = vec![
            SpatialWaypoint {
                beat: 0.0,
                azimuth_rad: 0.0,
                elevation_rad: 0.0,
                distance_m: 2.0,
                spread: 0.0,
                curve: SpatialInterpolationCurve::CubicCatmullRom,
            },
            SpatialWaypoint {
                beat: 4.0,
                azimuth_rad: PI / 2.0,
                elevation_rad: PI / 6.0,
                distance_m: 3.0,
                spread: 0.2,
                curve: SpatialInterpolationCurve::CubicCatmullRom,
            },
            SpatialWaypoint {
                beat: 8.0,
                azimuth_rad: PI,
                elevation_rad: 0.0,
                distance_m: 4.0,
                spread: 0.5,
                curve: SpatialInterpolationCurve::CubicCatmullRom,
            },
        ];

        let mid_v = track.evaluate_at_beat(2.0);
        assert!(mid_v.distance_m > 2.0 && mid_v.distance_m < 3.5);
        assert!(mid_v.azimuth_rad > 0.0 && mid_v.azimuth_rad < PI / 2.0);
    }

    #[test]
    fn test_spatial_automation_engine_toml_roundtrip() {
        let mut engine = SpatialAutomationEngine::new();
        let mut track1 = SpatialTrajectoryTrack::new(0, 0, "Synth 1 Orbit");
        track1.param_azimuth = Some(ParamId(10));
        track1.param_elevation = Some(ParamId(11));
        track1.param_distance = Some(ParamId(12));

        let mut track2 = SpatialTrajectoryTrack::new(1, 1, "Drums Flyby");
        track2.path_type = TrajectoryPathType::Flyby {
            start_x: -10.0,
            start_y: 2.0,
            start_z: 1.0,
            end_x: 10.0,
            end_y: 2.0,
            end_z: 1.0,
            smooth_ease: true,
        };

        engine.add_track(track1);
        engine.add_track(track2);

        let toml_str = engine.to_toml_string().expect("TOML serialization failed");
        assert!(toml_str.contains("Synth 1 Orbit"));
        assert!(toml_str.contains("Drums Flyby"));

        let restored = SpatialAutomationEngine::from_toml_str(&toml_str)
            .expect("TOML deserialization failed");
        assert_eq!(engine, restored);
    }

    #[test]
    fn test_spatial_automation_dispatch_zero_alloc() {
        let mut engine = SpatialAutomationEngine::new();
        let mut track = SpatialTrajectoryTrack::new(0, 0, "Test Dispatch");
        track.param_azimuth = Some(ParamId(20));
        track.param_elevation = Some(ParamId(21));
        track.param_distance = Some(ParamId(22));
        engine.add_track(track);

        let spatial_bus = SpatialBus::new();
        let mut param_bus = ParamBus::new();
        let _p1 = param_bus.register(ParamId(20), 0.0);
        let _p2 = param_bus.register(ParamId(21), 0.0);
        let _p3 = param_bus.register(ParamId(22), 0.0);

        engine.evaluate_and_dispatch(1.0, &spatial_bus, &param_bus);

        let v = spatial_bus.get_spherical(0);
        assert!(v.distance_m > 0.0);
        assert!((param_bus.get(ParamId(22)).unwrap() - v.distance_m).abs() < 1e-5);
    }
}
