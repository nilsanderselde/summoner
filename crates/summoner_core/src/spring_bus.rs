// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Lock-Free Spring Bus & Real-Time Mechanical Dispersion Parameter Routing (Milestone 20).
//!
//! Provides thread-safe lock-free exchange of continuous physical spring-mass lattice and
//! mechanical plate reverb parameters (Drive Force, Dispersion Factor, Boundary Tension,
//! Duffing Spring Non-Linearity, Transducer Pickup Angle, Damper Damping, Decay $T_{60}$,
//! and Articulation Technique Enums) across audio, sequencer, and GUI threads.
//!
//! Enforces zero heap allocations in real-time audio threads under `AllocGuard`.

use crate::param_bus::{ParamBus, ParamId};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};

/// Classical and contemporary mechanical spring-mass & plate articulation techniques.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u32)]
pub enum SpringArticulation {
    /// Transducer driver continuous electromagnetic drive excitation.
    #[default]
    DriverDrive = 0,
    /// Mechanical damper pad absorption mute ($T_{60} < 0.8\text{s}$).
    DamperMute = 1,
    /// Direct mechanical transient pluck of helical spring coils.
    SpringPluck = 2,
    /// High-displacement non-linear shaker table excitation.
    ShakerExcite = 3,
    /// Continuous dynamic modulation of boundary frame tension.
    TensionModulation = 4,
    /// Orbital spatial rotation sweep of pickup transducers.
    TransducerSweep = 5,
    /// High-energy corner boundary reflection acoustic accent.
    EdgeReflect = 6,
    /// High-amplitude Duffing non-linear cubic saturation.
    NonLinearSaturate = 7,
    /// Flexural wave dispersion warp across all-pass delay network.
    DispersionWarp = 8,
}

impl SpringArticulation {
    pub fn from_u32(val: u32) -> Self {
        match val {
            0 => Self::DriverDrive,
            1 => Self::DamperMute,
            2 => Self::SpringPluck,
            3 => Self::ShakerExcite,
            4 => Self::TensionModulation,
            5 => Self::TransducerSweep,
            6 => Self::EdgeReflect,
            7 => Self::NonLinearSaturate,
            8 => Self::DispersionWarp,
            _ => Self::DriverDrive,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::DriverDrive => "Transducer Driver Excitation",
            Self::DamperMute => "Mechanical Damper Mute",
            Self::SpringPluck => "Spring Coil Pluck",
            Self::ShakerExcite => "Nonlinear Shaker Excitation",
            Self::TensionModulation => "Boundary Tension Modulation",
            Self::TransducerSweep => "Pickup Transducer Orbit Sweep",
            Self::EdgeReflect => "Corner Reflection Accent",
            Self::NonLinearSaturate => "Duffing Cubic Saturation",
            Self::DispersionWarp => "Flexural Dispersion Warp",
        }
    }

    /// Returns nominal default $(F_{\text{drive}}, D_{\text{dispersion}}, T_{\text{boundary}}, \beta_{\text{nonlin}}, \theta_{\text{pickup}}, \gamma_{\text{damping}})$ parameters.
    pub fn nominal_parameters(&self) -> (f32, f32, f32, f32, f32, f32) {
        match self {
            Self::DriverDrive => (0.80, 0.60, 0.85, 0.20, 0.0, 0.15),
            Self::DamperMute => (0.50, 0.30, 0.90, 0.10, 0.0, 0.85),
            Self::SpringPluck => (0.95, 0.75, 0.70, 0.50, 0.0, 0.08),
            Self::ShakerExcite => (0.90, 0.40, 0.60, 0.85, 0.0, 0.20),
            Self::TensionModulation => (0.70, 0.65, 0.95, 0.30, 0.0, 0.10),
            Self::TransducerSweep => (0.75, 0.55, 0.80, 0.15, 1.57, 0.12),
            Self::EdgeReflect => (0.85, 0.70, 0.90, 0.40, 0.78, 0.05),
            Self::NonLinearSaturate => (1.00, 0.80, 0.50, 0.95, 0.0, 0.18),
            Self::DispersionWarp => (0.65, 0.95, 0.85, 0.25, 0.0, 0.10),
        }
    }
}

/// Instantaneous lock-free snapshot of spring and plate bus parameters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpringBusSnapshot {
    pub articulation: SpringArticulation,
    pub drive_force: f32,
    pub dispersion_factor: f32,
    pub boundary_tension: f32,
    pub spring_nonlinearity: f32,
    pub transducer_pickup_angle: f32,
    pub damper_damping: f32,
    pub decay_t60_sec: f32,
    pub gesture_progress: f32,
    pub is_active: bool,
}

impl Default for SpringBusSnapshot {
    fn default() -> Self {
        let (f, disp, tens, nonlin, angle, damp) = SpringArticulation::DriverDrive.nominal_parameters();
        Self {
            articulation: SpringArticulation::DriverDrive,
            drive_force: f,
            dispersion_factor: disp,
            boundary_tension: tens,
            spring_nonlinearity: nonlin,
            transducer_pickup_angle: angle,
            damper_damping: damp,
            decay_t60_sec: 3.5,
            gesture_progress: 0.0,
            is_active: false,
        }
    }
}

/// Lock-free physical spring-mass and plate reverb parameter routing bus.
pub struct SpringBus {
    articulation_raw: AtomicU32,
    drive_force_bits: AtomicU32,
    dispersion_bits: AtomicU32,
    tension_bits: AtomicU32,
    nonlinearity_bits: AtomicU32,
    pickup_angle_bits: AtomicU32,
    damping_bits: AtomicU32,
    t60_bits: AtomicU32,
    progress_bits: AtomicU32,
    flags: AtomicU32,
}

impl Default for SpringBus {
    fn default() -> Self {
        Self::new()
    }
}

impl SpringBus {
    /// Creates a new SpringBus initialized to standard DriverDrive defaults.
    pub fn new() -> Self {
        let (f, disp, tens, nonlin, angle, damp) = SpringArticulation::DriverDrive.nominal_parameters();
        Self {
            articulation_raw: AtomicU32::new(SpringArticulation::DriverDrive as u32),
            drive_force_bits: AtomicU32::new(f.to_bits()),
            dispersion_bits: AtomicU32::new(disp.to_bits()),
            tension_bits: AtomicU32::new(tens.to_bits()),
            nonlinearity_bits: AtomicU32::new(nonlin.to_bits()),
            pickup_angle_bits: AtomicU32::new(angle.to_bits()),
            damping_bits: AtomicU32::new(damp.to_bits()),
            t60_bits: AtomicU32::new(3.5f32.to_bits()),
            progress_bits: AtomicU32::new(0.0f32.to_bits()),
            flags: AtomicU32::new(0),
        }
    }

    /// Sets active articulation and loads nominal default parameters.
    pub fn set_articulation(&self, articulation: SpringArticulation) {
        self.articulation_raw.store(articulation as u32, Ordering::Release);
        let (f, disp, tens, nonlin, angle, damp) = articulation.nominal_parameters();
        self.set_gesture_vector(f, disp, tens, nonlin, angle, damp);
    }

    /// Gets active articulation.
    pub fn get_articulation(&self) -> SpringArticulation {
        SpringArticulation::from_u32(self.articulation_raw.load(Ordering::Acquire))
    }

    /// Sets continuous mechanical gesture vector $(F_{\text{drive}}, D_{\text{dispersion}}, T_{\text{boundary}}, \beta_{\text{nonlin}}, \theta_{\text{pickup}}, \gamma_{\text{damping}})$.
    pub fn set_gesture_vector(
        &self,
        drive_force: f32,
        dispersion_factor: f32,
        boundary_tension: f32,
        spring_nonlinearity: f32,
        transducer_pickup_angle: f32,
        damper_damping: f32,
    ) {
        self.drive_force_bits.store(drive_force.clamp(0.0, 1.0).to_bits(), Ordering::Release);
        self.dispersion_bits.store(dispersion_factor.clamp(0.0, 1.0).to_bits(), Ordering::Release);
        self.tension_bits.store(boundary_tension.clamp(0.0, 1.0).to_bits(), Ordering::Release);
        self.nonlinearity_bits.store(spring_nonlinearity.clamp(0.0, 1.0).to_bits(), Ordering::Release);
        self.pickup_angle_bits.store(transducer_pickup_angle.clamp(0.0, std::f32::consts::TAU).to_bits(), Ordering::Release);
        self.damping_bits.store(damper_damping.clamp(0.0, 1.0).to_bits(), Ordering::Release);
    }

    /// Gets continuous mechanical gesture vector $(F_{\text{drive}}, D_{\text{dispersion}}, T_{\text{boundary}}, \beta_{\text{nonlin}}, \theta_{\text{pickup}}, \gamma_{\text{damping}})$.
    pub fn get_gesture_vector(&self) -> (f32, f32, f32, f32, f32, f32) {
        (
            f32::from_bits(self.drive_force_bits.load(Ordering::Acquire)),
            f32::from_bits(self.dispersion_bits.load(Ordering::Acquire)),
            f32::from_bits(self.tension_bits.load(Ordering::Acquire)),
            f32::from_bits(self.nonlinearity_bits.load(Ordering::Acquire)),
            f32::from_bits(self.pickup_angle_bits.load(Ordering::Acquire)),
            f32::from_bits(self.damping_bits.load(Ordering::Acquire)),
        )
    }

    /// Sets plate decay time $T_{60}$ in seconds.
    pub fn set_decay_t60(&self, t60_sec: f32) {
        self.t60_bits.store(t60_sec.clamp(0.1, 15.0).to_bits(), Ordering::Release);
    }

    /// Gets plate decay time $T_{60}$ in seconds.
    pub fn get_decay_t60(&self) -> f32 {
        f32::from_bits(self.t60_bits.load(Ordering::Acquire))
    }

    /// Sets gesture trajectory progress $[0.0 ..= 1.0]$.
    pub fn set_progress(&self, progress: f32, is_active: bool) {
        self.progress_bits.store(progress.clamp(0.0, 1.0).to_bits(), Ordering::Release);

        let mut curr = self.flags.load(Ordering::Acquire);
        loop {
            let next = if is_active { curr | 1 } else { curr & !1 };
            match self.flags.compare_exchange_weak(curr, next, Ordering::Release, Ordering::Acquire) {
                Ok(_) => break,
                Err(actual) => curr = actual,
            }
        }
    }

    /// Captures a complete atomic snapshot of the spring bus.
    pub fn snapshot(&self) -> SpringBusSnapshot {
        let articulation = self.get_articulation();
        let (f, disp, tens, nonlin, angle, damp) = self.get_gesture_vector();
        let t60 = self.get_decay_t60();
        let progress = f32::from_bits(self.progress_bits.load(Ordering::Acquire));
        let flags = self.flags.load(Ordering::Acquire);

        SpringBusSnapshot {
            articulation,
            drive_force: f,
            dispersion_factor: disp,
            boundary_tension: tens,
            spring_nonlinearity: nonlin,
            transducer_pickup_angle: angle,
            damper_damping: damp,
            decay_t60_sec: t60,
            gesture_progress: progress,
            is_active: (flags & 1) != 0,
        }
    }

    /// Applies a complete snapshot into this bus.
    pub fn apply_snapshot(&self, snap: &SpringBusSnapshot) {
        self.articulation_raw.store(snap.articulation as u32, Ordering::Release);
        self.drive_force_bits.store(snap.drive_force.to_bits(), Ordering::Release);
        self.dispersion_bits.store(snap.dispersion_factor.to_bits(), Ordering::Release);
        self.tension_bits.store(snap.boundary_tension.to_bits(), Ordering::Release);
        self.nonlinearity_bits.store(snap.spring_nonlinearity.to_bits(), Ordering::Release);
        self.pickup_angle_bits.store(snap.transducer_pickup_angle.to_bits(), Ordering::Release);
        self.damping_bits.store(snap.damper_damping.to_bits(), Ordering::Release);
        self.t60_bits.store(snap.decay_t60_sec.to_bits(), Ordering::Release);
        self.progress_bits.store(snap.gesture_progress.to_bits(), Ordering::Release);
        self.flags.store(if snap.is_active { 1 } else { 0 }, Ordering::Release);
    }

    /// Dispatches current spring bus snapshot values to central `ParamBus`.
    pub fn dispatch_to_param_bus(&self, bus: &ParamBus, base_id: ParamId) {
        let snap = self.snapshot();
        bus.set(ParamId(base_id.0), snap.articulation as u32 as f32);
        bus.set(ParamId(base_id.0 + 1), snap.drive_force);
        bus.set(ParamId(base_id.0 + 2), snap.dispersion_factor);
        bus.set(ParamId(base_id.0 + 3), snap.boundary_tension);
        bus.set(ParamId(base_id.0 + 4), snap.spring_nonlinearity);
        bus.set(ParamId(base_id.0 + 5), snap.transducer_pickup_angle);
        bus.set(ParamId(base_id.0 + 6), snap.damper_damping);
        bus.set(ParamId(base_id.0 + 7), snap.decay_t60_sec);
        bus.set(ParamId(base_id.0 + 8), snap.gesture_progress);
    }

    /// Pulls updated parameter values from a central `ParamBus`.
    pub fn sync_from_param_bus(&self, bus: &ParamBus, base_id: ParamId) {
        let art_idx = bus.get(ParamId(base_id.0)).unwrap_or(0.0).max(0.0).round() as u32;
        let drive = bus.get(ParamId(base_id.0 + 1)).unwrap_or(0.80);
        let disp = bus.get(ParamId(base_id.0 + 2)).unwrap_or(0.60);
        let tens = bus.get(ParamId(base_id.0 + 3)).unwrap_or(0.85);
        let nonlin = bus.get(ParamId(base_id.0 + 4)).unwrap_or(0.20);
        let angle = bus.get(ParamId(base_id.0 + 5)).unwrap_or(0.0);
        let damp = bus.get(ParamId(base_id.0 + 6)).unwrap_or(0.15);
        let t60 = bus.get(ParamId(base_id.0 + 7)).unwrap_or(3.5);
        let prog = bus.get(ParamId(base_id.0 + 8)).unwrap_or(0.0);

        self.articulation_raw.store(art_idx, Ordering::Release);
        self.set_gesture_vector(drive, disp, tens, nonlin, angle, damp);
        self.set_decay_t60(t60);
        self.set_progress(prog, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::allocator::AllocGuard;

    #[test]
    fn test_spring_bus_snapshot_and_zero_allocation() {
        let bus = SpringBus::new();
        bus.set_articulation(SpringArticulation::SpringPluck);
        bus.set_gesture_vector(0.95, 0.75, 0.70, 0.50, 1.20, 0.08);
        bus.set_decay_t60(4.8);
        bus.set_progress(0.45, true);

        {
            let _guard = AllocGuard::new();
            let snap = bus.snapshot();
            assert_eq!(snap.articulation, SpringArticulation::SpringPluck);
            assert!((snap.drive_force - 0.95).abs() < 1e-4);
            assert!((snap.dispersion_factor - 0.75).abs() < 1e-4);
            assert!((snap.boundary_tension - 0.70).abs() < 1e-4);
            assert!((snap.spring_nonlinearity - 0.50).abs() < 1e-4);
            assert!((snap.transducer_pickup_angle - 1.20).abs() < 1e-4);
            assert!((snap.decay_t60_sec - 4.8).abs() < 1e-4);
            assert!(snap.is_active);
        }
    }

    #[test]
    fn test_spring_bus_param_bus_round_trip() {
        let spring_bus = SpringBus::new();
        let mut param_bus = ParamBus::new();
        for i in 0..12 {
            param_bus.register(ParamId(850 + i), 0.0);
        }

        spring_bus.set_articulation(SpringArticulation::NonLinearSaturate);
        spring_bus.set_gesture_vector(1.00, 0.80, 0.50, 0.95, 0.0, 0.18);
        spring_bus.dispatch_to_param_bus(&param_bus, ParamId(850));

        let sync_bus = SpringBus::new();
        sync_bus.sync_from_param_bus(&param_bus, ParamId(850));

        let snap = sync_bus.snapshot();
        assert_eq!(snap.articulation, SpringArticulation::NonLinearSaturate);
        assert!((snap.drive_force - 1.00).abs() < 1e-4);
        assert!((snap.spring_nonlinearity - 0.95).abs() < 1e-4);
    }
}
