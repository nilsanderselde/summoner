// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Lock-Free 3D Spatial Bus & Multi-Channel Coordinate Routing Registry (Milestone 12).
//!
//! Provides thread-safe lock-free 3D position vector management (Azimuth, Elevation, Distance, Spread)
//! across audio and sequencer threads, coordinate transformations (Spherical <-> Cartesian),
//! and sample-accurate dispatch into `ParamBus` handles.
//!
//! Enforces zero heap allocations in real-time audio threads under `AllocGuard`.

use crate::allocator::AllocGuard;
use crate::param_bus::{ParamBus, ParamId};
use serde::{Deserialize, Serialize};
use std::f32::consts::{PI, TAU};
use std::sync::atomic::{AtomicU32, Ordering};

pub const MAX_SPATIAL_OBJECTS: usize = 64;

/// 3D Spatial Vector in spherical coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpatialVector3D {
    /// Azimuth angle in radians ($-\pi$ to $+\pi$, 0 = front, $+\pi/2$ = right, $-\pi/2$ = left).
    pub azimuth_rad: f32,
    /// Elevation angle in radians ($-\pi/2$ to $+\pi/2$, 0 = horizon, $+\pi/2$ = zenith, $-\pi/2$ = nadir).
    pub elevation_rad: f32,
    /// Distance from listener origin in meters (0.1 to 100.0 m).
    pub distance_m: f32,
    /// Spatial width / spread factor (0.0 = point source, 1.0 = diffuse omnidirectional).
    pub spread: f32,
    /// Linear volume / gain scaling factor.
    pub gain: f32,
}

impl Default for SpatialVector3D {
    fn default() -> Self {
        Self {
            azimuth_rad: 0.0,
            elevation_rad: 0.0,
            distance_m: 1.0,
            spread: 0.0,
            gain: 1.0,
        }
    }
}

impl SpatialVector3D {
    /// Creates a new spatial vector from spherical parameters.
    pub fn new_spherical(azimuth_rad: f32, elevation_rad: f32, distance_m: f32, spread: f32) -> Self {
        let az = (azimuth_rad + PI).rem_euclid(TAU) - PI;
        let el = elevation_rad.clamp(-PI / 2.0, PI / 2.0);
        let dist = distance_m.max(0.1);
        let sp = spread.clamp(0.0, 1.0);
        Self {
            azimuth_rad: az,
            elevation_rad: el,
            distance_m: dist,
            spread: sp,
            gain: 1.0,
        }
    }

    /// Creates a spatial vector from 3D Cartesian coordinates $(x, y, z)$ in meters.
    ///
    /// Convention:
    /// - $x$: Right (+) / Left (-)
    /// - $y$: Front (+) / Back (-)
    /// - $z$: Above (+) / Below (-)
    pub fn from_cartesian(x: f32, y: f32, z: f32, spread: f32) -> Self {
        let dist = (x * x + y * y + z * z).sqrt().max(0.001);
        let el = (z / dist).clamp(-1.0, 1.0).asin();
        let az = x.atan2(y);
        Self {
            azimuth_rad: az,
            elevation_rad: el,
            distance_m: dist,
            spread: spread.clamp(0.0, 1.0),
            gain: 1.0,
        }
    }

    /// Converts this spherical vector to 3D Cartesian coordinates $(x, y, z)$ in meters.
    #[inline]
    pub fn to_cartesian(&self) -> (f32, f32, f32) {
        let cos_el = self.elevation_rad.cos();
        let x = self.distance_m * cos_el * self.azimuth_rad.sin();
        let y = self.distance_m * cos_el * self.azimuth_rad.cos();
        let z = self.distance_m * self.elevation_rad.sin();
        (x, y, z)
    }

    /// Normalized azimuth in $[-1.0, 1.0]$ where $-1.0 = -\pi$ and $+1.0 = +\pi$.
    #[inline]
    pub fn normalized_azimuth(&self) -> f32 {
        (self.azimuth_rad / PI).clamp(-1.0, 1.0)
    }

    /// Normalized elevation in $[-1.0, 1.0]$ where $-1.0 = -\pi/2$ and $+1.0 = +\pi/2$.
    #[inline]
    pub fn normalized_elevation(&self) -> f32 {
        (self.elevation_rad / (PI / 2.0)).clamp(-1.0, 1.0)
    }
}

/// An individual atomic lock-free slot storing a single track's 3D spatial state.
pub struct AtomicSpatialSlot {
    azimuth_bits: AtomicU32,
    elevation_bits: AtomicU32,
    distance_bits: AtomicU32,
    spread_bits: AtomicU32,
    gain_bits: AtomicU32,
}

impl Default for AtomicSpatialSlot {
    fn default() -> Self {
        Self {
            azimuth_bits: AtomicU32::new(0.0f32.to_bits()),
            elevation_bits: AtomicU32::new(0.0f32.to_bits()),
            distance_bits: AtomicU32::new(1.0f32.to_bits()),
            spread_bits: AtomicU32::new(0.0f32.to_bits()),
            gain_bits: AtomicU32::new(1.0f32.to_bits()),
        }
    }
}

impl AtomicSpatialSlot {
    #[inline]
    pub fn load(&self) -> SpatialVector3D {
        let az = f32::from_bits(self.azimuth_bits.load(Ordering::Acquire));
        let el = f32::from_bits(self.elevation_bits.load(Ordering::Acquire));
        let dist = f32::from_bits(self.distance_bits.load(Ordering::Acquire));
        let spread = f32::from_bits(self.spread_bits.load(Ordering::Acquire));
        let gain = f32::from_bits(self.gain_bits.load(Ordering::Acquire));
        SpatialVector3D {
            azimuth_rad: az,
            elevation_rad: el,
            distance_m: dist,
            spread,
            gain,
        }
    }

    #[inline]
    pub fn store(&self, vec: &SpatialVector3D) {
        self.azimuth_bits.store(vec.azimuth_rad.to_bits(), Ordering::Release);
        self.elevation_bits.store(vec.elevation_rad.to_bits(), Ordering::Release);
        self.distance_bits.store(vec.distance_m.to_bits(), Ordering::Release);
        self.spread_bits.store(vec.spread.to_bits(), Ordering::Release);
        self.gain_bits.store(vec.gain.to_bits(), Ordering::Release);
    }
}

/// Global Lock-Free 3D Spatial Bus.
pub struct SpatialBus {
    slots: [AtomicSpatialSlot; MAX_SPATIAL_OBJECTS],
}

impl Default for SpatialBus {
    fn default() -> Self {
        Self::new()
    }
}

impl SpatialBus {
    /// Creates a new initialized lock-free `SpatialBus`.
    pub fn new() -> Self {
        let mut slots = Vec::with_capacity(MAX_SPATIAL_OBJECTS);
        for _ in 0..MAX_SPATIAL_OBJECTS {
            slots.push(AtomicSpatialSlot::default());
        }
        let boxed_slice = slots.into_boxed_slice();
        let array_box: Box<[AtomicSpatialSlot; MAX_SPATIAL_OBJECTS]> = match boxed_slice.try_into() {
            Ok(arr) => arr,
            Err(_) => unreachable!(),
        };
        Self { slots: *array_box }
    }

    /// Stores a 3D position vector in spherical coordinates for a given object/track index.
    #[inline]
    pub fn set_spherical(
        &self,
        object_id: usize,
        azimuth_rad: f32,
        elevation_rad: f32,
        distance_m: f32,
        spread: f32,
    ) {
        if object_id < MAX_SPATIAL_OBJECTS {
            let vec = SpatialVector3D::new_spherical(azimuth_rad, elevation_rad, distance_m, spread);
            self.slots[object_id].store(&vec);
        }
    }

    /// Stores a 3D position vector in Cartesian coordinates for a given object/track index.
    #[inline]
    pub fn set_cartesian(&self, object_id: usize, x: f32, y: f32, z: f32, spread: f32) {
        if object_id < MAX_SPATIAL_OBJECTS {
            let vec = SpatialVector3D::from_cartesian(x, y, z, spread);
            self.slots[object_id].store(&vec);
        }
    }

    /// Reads the spherical 3D spatial vector for a given object/track index.
    #[inline]
    pub fn get_spherical(&self, object_id: usize) -> SpatialVector3D {
        if object_id < MAX_SPATIAL_OBJECTS {
            self.slots[object_id].load()
        } else {
            SpatialVector3D::default()
        }
    }

    /// Reads the 3D Cartesian coordinates $(x, y, z)$ for a given object/track index.
    #[inline]
    pub fn get_cartesian(&self, object_id: usize) -> (f32, f32, f32) {
        let vec = self.get_spherical(object_id);
        vec.to_cartesian()
    }

    /// Atomically dispatches the current 3D position of an object into `ParamBus` parameter IDs.
    #[inline]
    pub fn dispatch_to_param_bus(
        &self,
        object_id: usize,
        param_bus: &ParamBus,
        param_azimuth: ParamId,
        param_elevation: ParamId,
        param_distance: ParamId,
    ) {
        let _guard = AllocGuard::new();
        let vec = self.get_spherical(object_id);
        param_bus.set(param_azimuth, vec.azimuth_rad);
        param_bus.set(param_elevation, vec.elevation_rad);
        param_bus.set(param_distance, vec.distance_m);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spherical_cartesian_round_trip() {
        let test_points = [
            (0.0, 0.0, 1.0),
            (PI / 2.0, 0.0, 2.0),
            (-PI / 2.0, 0.0, 3.5),
            (0.0, PI / 4.0, 5.0),
            (PI / 4.0, -PI / 6.0, 10.0),
        ];

        for &(az, el, dist) in &test_points {
            let vec1 = SpatialVector3D::new_spherical(az, el, dist, 0.2);
            let (x, y, z) = vec1.to_cartesian();
            let vec2 = SpatialVector3D::from_cartesian(x, y, z, 0.2);

            assert!((vec1.azimuth_rad - vec2.azimuth_rad).abs() < 1e-4);
            assert!((vec1.elevation_rad - vec2.elevation_rad).abs() < 1e-4);
            assert!((vec1.distance_m - vec2.distance_m).abs() < 1e-4);
        }
    }

    #[test]
    fn test_spatial_bus_lock_free_operations() {
        let bus = SpatialBus::new();
        bus.set_spherical(0, 1.25, 0.35, 4.5, 0.5);

        let read_vec = bus.get_spherical(0);
        assert!((read_vec.azimuth_rad - 1.25).abs() < 1e-5);
        assert!((read_vec.elevation_rad - 0.35).abs() < 1e-5);
        assert!((read_vec.distance_m - 4.5).abs() < 1e-5);

        let mut param_bus = ParamBus::new();
        let _p_az = param_bus.register(ParamId(101), 0.0);
        let _p_el = param_bus.register(ParamId(102), 0.0);
        let _p_dist = param_bus.register(ParamId(103), 0.0);

        bus.dispatch_to_param_bus(0, &param_bus, ParamId(101), ParamId(102), ParamId(103));

        assert!((param_bus.get(ParamId(101)).unwrap() - 1.25).abs() < 1e-5);
        assert!((param_bus.get(ParamId(102)).unwrap() - 0.35).abs() < 1e-5);
        assert!((param_bus.get(ParamId(103)).unwrap() - 4.5).abs() < 1e-5);
    }
}
