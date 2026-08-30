// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Lock-Free Bellows Bus Parameter Routing (Milestone 28).
//!
//! Provides thread-safe lock-free exchange of continuous physical modeling free-reed
//! aerophone and bellows dynamics parameters (Bellows Pressure Force [-1200..+1200 Pa],
//! Push/Pull Direction, Pallet Valve Key Velocity, Cassotto Mute Aperture [0..1],
//! Musette Detune Spread [0..35 cents], Register Switch Bitmask, Reed Stiffness Factor,
//! and Master Output Gain) across audio, sequencer, and GUI threads.
//!
//! Enforces zero heap allocations in real-time audio threads under `AllocGuard`.

use crate::param_bus::{ParamBus, ParamId};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};

/// Classical and modern Bellows performance styles and gesture articulations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u32)]
pub enum BellowsArticulation {
    /// Dramatic Argentine Tango Marcato (Sharp percussive bellows accents with heavy push/pull changes).
    #[default]
    TangoMarcatoAccented = 0,
    /// French Café Musette Waltz (Smooth undulating bellows swells with rich wet tremolo).
    MusetteWaltzSwell = 1,
    /// Russian Bayan Virtuoso Bellows Shake (Rapid micro-tremolo bellows shaking across dense chords).
    BayanVirtuosoBellowsShake = 2,
    /// Meditative Harmonium Suction Drone (Continuous steady low-pressure airflow with rich lower ranks).
    HarmoniumMeditativeDrone = 3,
    /// English Concertina Fast Staccato (Rapid crisp pallet valve triggering with springy pressure).
    ConcertinaFastStaccato = 4,
    /// Double Cassotto Intimate Solo (Warm, dark, mellow lead line enclosed in wooden chamber).
    CassottoIntimateSolo = 5,
    /// Grand Tutti Fortissimo Climax (All 5 ranks open under maximum bellows compression).
    TuttiFortissimoClimax = 6,
    /// Custom multi-keyframe Catmull-Rom spline trajectory.
    CustomSpline = 7,
}

/// Full nominal tuple parameters for Bellows articulation:
/// (bellows_pressure_pa, push_direction, valve_velocity, cassotto_aperture,
///  musette_detune_cents, register_mask, reed_stiffness, master_gain)
pub type BellowsNominalParameters = (
    f32, // bellows_pressure_pa [-1200.0 ..= +1200.0]
    f32, // push_direction [1.0 = push, -1.0 = pull]
    f32, // valve_velocity [0.01 ..= 1.0]
    f32, // cassotto_aperture [0.0 ..= 1.0]
    f32, // musette_detune_cents [0.0 ..= 35.0]
    u32, // register_mask (bitmask for 16', 8', 8'+, 8'-, 4')
    f32, // reed_stiffness [0.5 ..= 2.0]
    f32, // master_gain [0.0 ..= 2.0]
);

impl BellowsArticulation {
    pub fn from_u32(val: u32) -> Self {
        match val {
            0 => Self::TangoMarcatoAccented,
            1 => Self::MusetteWaltzSwell,
            2 => Self::BayanVirtuosoBellowsShake,
            3 => Self::HarmoniumMeditativeDrone,
            4 => Self::ConcertinaFastStaccato,
            5 => Self::CassottoIntimateSolo,
            6 => Self::TuttiFortissimoClimax,
            7 => Self::CustomSpline,
            _ => Self::TangoMarcatoAccented,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::TangoMarcatoAccented => "Tango Marcato (Accented Bellows)",
            Self::MusetteWaltzSwell => "Musette Waltz (Undulating Swell)",
            Self::BayanVirtuosoBellowsShake => "Bayan Bellows Shake (Tremolo)",
            Self::HarmoniumMeditativeDrone => "Harmonium Meditative Drone",
            Self::ConcertinaFastStaccato => "Concertina Fast Staccato",
            Self::CassottoIntimateSolo => "Double Cassotto Intimate Solo",
            Self::TuttiFortissimoClimax => "Grand Tutti Fortissimo",
            Self::CustomSpline => "Custom Spline Gesture",
        }
    }

    /// Returns nominal default parameters tuple.
    pub fn nominal_parameters(&self) -> BellowsNominalParameters {
        match self {
            Self::TangoMarcatoAccented => (680.0, 1.0, 0.90, 0.70, 2.0, 0b00011, 1.25, 0.90), // 16'+8' Bandoneon
            Self::MusetteWaltzSwell => (420.0, 1.0, 0.75, 0.85, 18.0, 0b01110, 1.00, 0.85),    // 8'+8'+8'- Musette
            Self::BayanVirtuosoBellowsShake => (850.0, 1.0, 0.95, 1.00, 4.0, 0b11111, 1.40, 0.95),// Full Master
            Self::HarmoniumMeditativeDrone => (280.0, -1.0, 0.60, 0.50, 8.0, 0b00111, 0.85, 0.80), // Suction pull
            Self::ConcertinaFastStaccato => (520.0, 1.0, 0.85, 0.90, 0.0, 0b00010, 1.15, 0.88),  // 8' Solo
            Self::CassottoIntimateSolo => (360.0, 1.0, 0.70, 0.35, 1.0, 0b00010, 1.10, 0.85),    // 8' Deep Cassotto
            Self::TuttiFortissimoClimax => (1100.0, 1.0, 1.00, 1.00, 16.0, 0b11111, 1.30, 1.00), // Maximum Tutti
            Self::CustomSpline => (550.0, 1.0, 0.80, 0.80, 8.0, 0b11111, 1.10, 0.85),
        }
    }
}

/// Instantaneous lock-free snapshot of Bellows bus parameters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BellowsBusSnapshot {
    pub articulation: BellowsArticulation,
    pub bellows_pressure_pa: f32,
    pub push_direction: f32,
    pub valve_velocity: f32,
    pub cassotto_aperture: f32,
    pub musette_detune_cents: f32,
    pub register_mask: u32,
    pub reed_stiffness: f32,
    pub master_gain: f32,
}

/// Default base ParamId for Bellows & Free-Reed synthesis.
pub const BELLOWS_BASE_PARAM_ID: ParamId = ParamId(1250);

/// Helper to pre-register all Bellows parameters into a ParamBus.
pub fn register_bellows_params(param_bus: &mut ParamBus, base_id: ParamId) {
    let (p_pa, dir, vel, cassotto, musette, mask, stiff, mgain) =
        BellowsArticulation::TangoMarcatoAccented.nominal_parameters();

    param_bus.register(ParamId(base_id.0), BellowsArticulation::TangoMarcatoAccented as u32 as f32);
    param_bus.register(ParamId(base_id.0 + 1), p_pa);
    param_bus.register(ParamId(base_id.0 + 2), dir);
    param_bus.register(ParamId(base_id.0 + 3), vel);
    param_bus.register(ParamId(base_id.0 + 4), cassotto);
    param_bus.register(ParamId(base_id.0 + 5), musette);
    param_bus.register(ParamId(base_id.0 + 6), mask as f32);
    param_bus.register(ParamId(base_id.0 + 7), stiff);
    param_bus.register(ParamId(base_id.0 + 8), mgain);
}

/// Complete thread-safe lock-free Bellows parameter bus.
#[derive(Debug)]
pub struct BellowsBus {
    pub articulation: AtomicU32,
    pub bellows_pressure_pa: AtomicU32,
    pub push_direction: AtomicU32,
    pub valve_velocity: AtomicU32,
    pub cassotto_aperture: AtomicU32,
    pub musette_detune_cents: AtomicU32,
    pub register_mask: AtomicU32,
    pub reed_stiffness: AtomicU32,
    pub master_gain: AtomicU32,
}

impl Default for BellowsBus {
    fn default() -> Self {
        Self::new()
    }
}

impl BellowsBus {
    /// Create a new bellows parameter bus with default TangoMarcatoAccented settings.
    pub fn new() -> Self {
        let (p_pa, dir, vel, cassotto, musette, mask, stiff, mgain) =
            BellowsArticulation::TangoMarcatoAccented.nominal_parameters();

        Self {
            articulation: AtomicU32::new(BellowsArticulation::TangoMarcatoAccented as u32),
            bellows_pressure_pa: AtomicU32::new(p_pa.to_bits()),
            push_direction: AtomicU32::new(dir.to_bits()),
            valve_velocity: AtomicU32::new(vel.to_bits()),
            cassotto_aperture: AtomicU32::new(cassotto.to_bits()),
            musette_detune_cents: AtomicU32::new(musette.to_bits()),
            register_mask: AtomicU32::new(mask),
            reed_stiffness: AtomicU32::new(stiff.to_bits()),
            master_gain: AtomicU32::new(mgain.to_bits()),
        }
    }

    /// Read an instantaneous atomic snapshot. Safe for real-time audio threads.
    #[inline]
    pub fn snapshot(&self) -> BellowsBusSnapshot {
        let art_u32 = self.articulation.load(Ordering::Relaxed);
        let p_bits = self.bellows_pressure_pa.load(Ordering::Relaxed);
        let dir_bits = self.push_direction.load(Ordering::Relaxed);
        let vel_bits = self.valve_velocity.load(Ordering::Relaxed);
        let cass_bits = self.cassotto_aperture.load(Ordering::Relaxed);
        let mus_bits = self.musette_detune_cents.load(Ordering::Relaxed);
        let mask = self.register_mask.load(Ordering::Relaxed);
        let stiff_bits = self.reed_stiffness.load(Ordering::Relaxed);
        let mgain_bits = self.master_gain.load(Ordering::Relaxed);

        BellowsBusSnapshot {
            articulation: BellowsArticulation::from_u32(art_u32),
            bellows_pressure_pa: f32::from_bits(p_bits),
            push_direction: f32::from_bits(dir_bits),
            valve_velocity: f32::from_bits(vel_bits),
            cassotto_aperture: f32::from_bits(cass_bits),
            musette_detune_cents: f32::from_bits(mus_bits),
            register_mask: mask,
            reed_stiffness: f32::from_bits(stiff_bits),
            master_gain: f32::from_bits(mgain_bits),
        }
    }

    /// Update articulation and set nominal parameters atomically.
    pub fn set_articulation(&self, art: BellowsArticulation) {
        let (p_pa, dir, vel, cassotto, musette, mask, stiff, mgain) =
            art.nominal_parameters();

        self.articulation.store(art as u32, Ordering::Relaxed);
        self.bellows_pressure_pa.store(p_pa.to_bits(), Ordering::Relaxed);
        self.push_direction.store(dir.to_bits(), Ordering::Relaxed);
        self.valve_velocity.store(vel.to_bits(), Ordering::Relaxed);
        self.cassotto_aperture.store(cassotto.to_bits(), Ordering::Relaxed);
        self.musette_detune_cents.store(musette.to_bits(), Ordering::Relaxed);
        self.register_mask.store(mask, Ordering::Relaxed);
        self.reed_stiffness.store(stiff.to_bits(), Ordering::Relaxed);
        self.master_gain.store(mgain.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_bellows_pressure(&self, pressure_pa: f32) {
        self.bellows_pressure_pa.store(pressure_pa.to_bits(), Ordering::Relaxed);
        let dir = if pressure_pa >= 0.0 { 1.0f32 } else { -1.0f32 };
        self.push_direction.store(dir.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_valve_velocity(&self, vel: f32) {
        self.valve_velocity.store(vel.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_cassotto_aperture(&self, aperture: f32) {
        self.cassotto_aperture.store(aperture.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_musette_detune(&self, cents: f32) {
        self.musette_detune_cents.store(cents.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_register_mask(&self, mask: u32) {
        self.register_mask.store(mask, Ordering::Relaxed);
    }

    #[inline]
    pub fn set_reed_stiffness(&self, stiff: f32) {
        self.reed_stiffness.store(stiff.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn set_master_gain(&self, gain: f32) {
        self.master_gain.store(gain.to_bits(), Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bellows_bus_atomic_updates_and_round_trip() {
        let bus = BellowsBus::new();
        let snap = bus.snapshot();

        assert_eq!(snap.articulation, BellowsArticulation::TangoMarcatoAccented);
        assert!((snap.bellows_pressure_pa - 680.0).abs() < 1e-4);
        assert!((snap.valve_velocity - 0.90).abs() < 1e-4);

        // Update articulation
        bus.set_articulation(BellowsArticulation::MusetteWaltzSwell);
        let snap2 = bus.snapshot();
        assert_eq!(snap2.articulation, BellowsArticulation::MusetteWaltzSwell);
        assert!((snap2.musette_detune_cents - 18.0).abs() < 1e-4);

        // Individual setters
        bus.set_bellows_pressure(-450.0);
        let snap3 = bus.snapshot();
        assert!((snap3.bellows_pressure_pa - (-450.0)).abs() < 1e-4);
        assert!((snap3.push_direction - (-1.0)).abs() < 1e-4);
    }

    #[test]
    fn test_bellows_bus_param_bus_registration() {
        let mut param_bus = ParamBus::new();
        register_bellows_params(&mut param_bus, BELLOWS_BASE_PARAM_ID);

        let p_val = param_bus.get(ParamId(BELLOWS_BASE_PARAM_ID.0 + 1));
        assert!(p_val.is_some());
        assert!((p_val.unwrap() - 680.0).abs() < 1e-4);
    }
}
