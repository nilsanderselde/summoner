// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Lock-Free Articulation Bus & Real-Time Bowing Gesture Parameter Routing (Milestone 16).
//!
//! Provides thread-safe lock-free exchange of continuous physical bowing articulation vectors
//! (Bow Velocity $v_b$, Normal Force $F_N$, Bridge Proximity $\beta$, Rosin Adhesion, String Damping,
//! Vibrato, and Technique Enums) across audio, sequencer, and GUI threads.
//!
//! Enforces zero heap allocations in real-time audio threads under `AllocGuard`.

use crate::param_bus::{ParamBus, ParamId};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};

/// Classical and modern extended bowing techniques.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u32)]
pub enum BowingTechnique {
    /// Smooth continuous legato bow stroke.
    #[default]
    Legato = 0,
    /// Separated alternating detaché strokes.
    Detache = 1,
    /// Accented hammered martelé attack.
    Martele = 2,
    /// Bouncing spiccato off-the-string articulation.
    Spiccato = 3,
    /// Crisp crisp staccato stopped stroke.
    Staccato = 4,
    /// Rapid unmeasured tremolo stroke alternation.
    Tremolo = 5,
    /// Sul Ponticello: bowing near bridge ($\beta \approx 0.04$) for shrill glassy overtones.
    SulPonticello = 6,
    /// Sul Tasto: bowing over fingerboard ($\beta \approx 0.28$) for airy flute-like tone.
    SulTasto = 7,
    /// Col Legno Tratto: dragging the wood of the bow across the string.
    ColLegno = 8,
    /// Plucked pizzicato impulse excitation.
    Pizzicato = 9,
}

impl BowingTechnique {
    pub fn from_u32(val: u32) -> Self {
        match val {
            0 => Self::Legato,
            1 => Self::Detache,
            2 => Self::Martele,
            3 => Self::Spiccato,
            4 => Self::Staccato,
            5 => Self::Tremolo,
            6 => Self::SulPonticello,
            7 => Self::SulTasto,
            8 => Self::ColLegno,
            9 => Self::Pizzicato,
            _ => Self::Legato,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Legato => "Legato",
            Self::Detache => "Détaché",
            Self::Martele => "Martelé",
            Self::Spiccato => "Spiccato",
            Self::Staccato => "Staccato",
            Self::Tremolo => "Tremolo",
            Self::SulPonticello => "Sul Ponticello",
            Self::SulTasto => "Sul Tasto",
            Self::ColLegno => "Col Legno",
            Self::Pizzicato => "Pizzicato",
        }
    }

    /// Returns nominal default $(\text{velocity\_mps}, \text{force\_n}, \beta)$ for this technique.
    pub fn nominal_parameters(&self) -> (f32, f32, f32) {
        match self {
            Self::Legato => (0.35, 1.20, 0.12),
            Self::Detache => (0.50, 1.40, 0.12),
            Self::Martele => (0.75, 2.80, 0.10),
            Self::Spiccato => (0.45, 0.60, 0.14),
            Self::Staccato => (0.40, 1.80, 0.12),
            Self::Tremolo => (0.85, 0.90, 0.08),
            Self::SulPonticello => (0.30, 0.70, 0.04),
            Self::SulTasto => (0.45, 0.50, 0.28),
            Self::ColLegno => (0.25, 0.80, 0.15),
            Self::Pizzicato => (0.00, 3.50, 0.18),
        }
    }
}

/// Instantaneous lock-free snapshot of bowing articulation bus parameters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ArticulationBusSnapshot {
    pub technique: BowingTechnique,
    pub bow_velocity_mps: f32,
    pub bow_force_n: f32,
    pub bridge_proximity_beta: f32,
    pub rosin_adhesion: f32,
    pub string_damping: f32,
    pub vibrato_depth: f32,
    pub vibrato_rate_hz: f32,
    pub gesture_progress: f32,
    pub is_active: bool,
}

impl Default for ArticulationBusSnapshot {
    fn default() -> Self {
        Self {
            technique: BowingTechnique::Legato,
            bow_velocity_mps: 0.35,
            bow_force_n: 1.20,
            bridge_proximity_beta: 0.12,
            rosin_adhesion: 0.80,
            string_damping: 0.15,
            vibrato_depth: 0.0,
            vibrato_rate_hz: 5.5,
            gesture_progress: 0.0,
            is_active: false,
        }
    }
}

/// Lock-free physical bowing articulation vector bus.
pub struct ArticulationBus {
    technique_raw: AtomicU32,
    velocity_bits: AtomicU32,
    force_bits: AtomicU32,
    beta_bits: AtomicU32,
    rosin_bits: AtomicU32,
    damping_bits: AtomicU32,
    vib_depth_bits: AtomicU32,
    vib_rate_bits: AtomicU32,
    progress_bits: AtomicU32,
    flags: AtomicU32,
}

impl Default for ArticulationBus {
    fn default() -> Self {
        Self::new()
    }
}

impl ArticulationBus {
    /// Creates a new ArticulationBus initialized to standard Legato defaults.
    pub fn new() -> Self {
        let (vel, force, beta) = BowingTechnique::Legato.nominal_parameters();
        Self {
            technique_raw: AtomicU32::new(BowingTechnique::Legato as u32),
            velocity_bits: AtomicU32::new(vel.to_bits()),
            force_bits: AtomicU32::new(force.to_bits()),
            beta_bits: AtomicU32::new(beta.to_bits()),
            rosin_bits: AtomicU32::new(0.80f32.to_bits()),
            damping_bits: AtomicU32::new(0.15f32.to_bits()),
            vib_depth_bits: AtomicU32::new(0.0f32.to_bits()),
            vib_rate_bits: AtomicU32::new(5.5f32.to_bits()),
            progress_bits: AtomicU32::new(0.0f32.to_bits()),
            flags: AtomicU32::new(0),
        }
    }

    /// Sets the active bowing technique and loads nominal parameter defaults.
    pub fn set_technique(&self, technique: BowingTechnique) {
        self.technique_raw.store(technique as u32, Ordering::Release);
        let (vel, force, beta) = technique.nominal_parameters();
        self.set_gesture_vector(vel, force, beta, 0.80, 0.15);
    }

    /// Gets the active bowing technique.
    pub fn get_technique(&self) -> BowingTechnique {
        BowingTechnique::from_u32(self.technique_raw.load(Ordering::Acquire))
    }

    /// Sets physical bowing vector $(v_b, F_N, \beta, \text{rosin}, \text{damping})$.
    pub fn set_gesture_vector(
        &self,
        velocity_mps: f32,
        force_n: f32,
        bridge_proximity_beta: f32,
        rosin_adhesion: f32,
        string_damping: f32,
    ) {
        self.velocity_bits.store(velocity_mps.clamp(-2.0, 2.0).to_bits(), Ordering::Release);
        self.force_bits.store(force_n.clamp(0.0, 5.0).to_bits(), Ordering::Release);
        self.beta_bits.store(bridge_proximity_beta.clamp(0.02, 0.50).to_bits(), Ordering::Release);
        self.rosin_bits.store(rosin_adhesion.clamp(0.0, 1.0).to_bits(), Ordering::Release);
        self.damping_bits.store(string_damping.clamp(0.0, 1.0).to_bits(), Ordering::Release);
    }

    /// Gets physical bowing vector $(v_b, F_N, \beta, \text{rosin}, \text{damping})$.
    pub fn get_gesture_vector(&self) -> (f32, f32, f32, f32, f32) {
        (
            f32::from_bits(self.velocity_bits.load(Ordering::Acquire)),
            f32::from_bits(self.force_bits.load(Ordering::Acquire)),
            f32::from_bits(self.beta_bits.load(Ordering::Acquire)),
            f32::from_bits(self.rosin_bits.load(Ordering::Acquire)),
            f32::from_bits(self.damping_bits.load(Ordering::Acquire)),
        )
    }

    /// Sets vibrato depth (semitones) and rate (Hz).
    pub fn set_vibrato(&self, depth_semitones: f32, rate_hz: f32) {
        self.vib_depth_bits.store(depth_semitones.clamp(0.0, 2.0).to_bits(), Ordering::Release);
        self.vib_rate_bits.store(rate_hz.clamp(0.5, 15.0).to_bits(), Ordering::Release);
    }

    /// Gets vibrato depth and rate.
    pub fn get_vibrato(&self) -> (f32, f32) {
        (
            f32::from_bits(self.vib_depth_bits.load(Ordering::Acquire)),
            f32::from_bits(self.vib_rate_bits.load(Ordering::Acquire)),
        )
    }

    /// Sets continuous gesture trajectory progress $[0.0 ..= 1.0]$.
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

    /// Captures a complete atomic snapshot of the articulation bus.
    pub fn snapshot(&self) -> ArticulationBusSnapshot {
        let technique = self.get_technique();
        let (bow_velocity_mps, bow_force_n, bridge_proximity_beta, rosin_adhesion, string_damping) =
            self.get_gesture_vector();
        let (vibrato_depth, vibrato_rate_hz) = self.get_vibrato();
        let progress = f32::from_bits(self.progress_bits.load(Ordering::Acquire));
        let flags = self.flags.load(Ordering::Acquire);

        ArticulationBusSnapshot {
            technique,
            bow_velocity_mps,
            bow_force_n,
            bridge_proximity_beta,
            rosin_adhesion,
            string_damping,
            vibrato_depth,
            vibrato_rate_hz,
            gesture_progress: progress,
            is_active: (flags & 1) != 0,
        }
    }

    /// Dispatches articulation bus values into `ParamBus` handles.
    pub fn dispatch_to_param_bus(&self, bus: &ParamBus, base_param_id: ParamId) {
        let snap = self.snapshot();
        bus.set(ParamId(base_param_id.0), snap.technique as u32 as f32);
        bus.set(ParamId(base_param_id.0 + 1), snap.bow_velocity_mps);
        bus.set(ParamId(base_param_id.0 + 2), snap.bow_force_n);
        bus.set(ParamId(base_param_id.0 + 3), snap.bridge_proximity_beta);
        bus.set(ParamId(base_param_id.0 + 4), snap.rosin_adhesion);
        bus.set(ParamId(base_param_id.0 + 5), snap.string_damping);
        bus.set(ParamId(base_param_id.0 + 6), snap.vibrato_depth);
        bus.set(ParamId(base_param_id.0 + 7), snap.vibrato_rate_hz);
        bus.set(ParamId(base_param_id.0 + 8), snap.gesture_progress);
    }

    /// Synchronizes articulation bus values from `ParamBus` handles.
    pub fn sync_from_param_bus(&self, bus: &ParamBus, base_param_id: ParamId) {
        let tech_idx = bus.get(ParamId(base_param_id.0)).unwrap_or(0.0).max(0.0).round() as u32;
        let vel = bus.get(ParamId(base_param_id.0 + 1)).unwrap_or(0.35);
        let force = bus.get(ParamId(base_param_id.0 + 2)).unwrap_or(1.20);
        let beta = bus.get(ParamId(base_param_id.0 + 3)).unwrap_or(0.12);
        let rosin = bus.get(ParamId(base_param_id.0 + 4)).unwrap_or(0.80);
        let damp = bus.get(ParamId(base_param_id.0 + 5)).unwrap_or(0.15);
        let vib_d = bus.get(ParamId(base_param_id.0 + 6)).unwrap_or(0.0);
        let vib_r = bus.get(ParamId(base_param_id.0 + 7)).unwrap_or(5.5);
        let prog = bus.get(ParamId(base_param_id.0 + 8)).unwrap_or(0.0);

        self.technique_raw.store(tech_idx, Ordering::Release);
        self.set_gesture_vector(vel, force, beta, rosin, damp);
        self.set_vibrato(vib_d, vib_r);
        self.set_progress(prog, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::allocator::AllocGuard;

    #[test]
    fn test_articulation_bus_atomic_read_write_zero_allocation() {
        let bus = ArticulationBus::new();
        bus.set_technique(BowingTechnique::SulPonticello);
        bus.set_gesture_vector(0.42, 0.85, 0.05, 0.90, 0.22);
        bus.set_vibrato(0.35, 6.0);
        bus.set_progress(0.68, true);

        {
            let _guard = AllocGuard::new();
            let snap = bus.snapshot();
            assert_eq!(snap.technique, BowingTechnique::SulPonticello);
            assert!((snap.bow_velocity_mps - 0.42).abs() < 1e-4);
            assert!((snap.bow_force_n - 0.85).abs() < 1e-4);
            assert!((snap.bridge_proximity_beta - 0.05).abs() < 1e-4);
            assert!((snap.vibrato_depth - 0.35).abs() < 1e-4);
            assert!(snap.is_active);
        }
    }

    #[test]
    fn test_articulation_bus_param_bus_round_trip() {
        let art_bus = ArticulationBus::new();
        let mut param_bus = ParamBus::new();
        for i in 0..12 {
            param_bus.register(ParamId(500 + i), 0.0);
        }

        art_bus.set_technique(BowingTechnique::Martele);
        art_bus.set_gesture_vector(0.80, 2.50, 0.09, 0.95, 0.10);
        art_bus.dispatch_to_param_bus(&param_bus, ParamId(500));

        let sync_bus = ArticulationBus::new();
        sync_bus.sync_from_param_bus(&param_bus, ParamId(500));

        let snap = sync_bus.snapshot();
        assert_eq!(snap.technique, BowingTechnique::Martele);
        assert!((snap.bow_velocity_mps - 0.80).abs() < 1e-4);
        assert!((snap.bow_force_n - 2.50).abs() < 1e-4);
    }
}
