// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Lock-Free Breath Bus & Real-Time Woodwind Fingering Parameter Routing (Milestone 17).
//!
//! Provides thread-safe lock-free exchange of continuous physical woodwind breath gesture vectors
//! (Blowing Pressure $P_m$, Jet Distance $d_{jet}$, Embouchure Angle $\theta_{jet}$, 6-Tonehole Fingering Mask,
//! Vibrato, Flutter Tongue, and Articulation Enums) across audio, sequencer, and GUI threads.
//!
//! Enforces zero heap allocations in real-time audio threads under `AllocGuard`.

use crate::param_bus::{ParamBus, ParamId};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};

/// Classical and modern extended woodwind articulation techniques.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u32)]
pub enum WoodwindArticulation {
    /// Smooth continuous singing legato breath flow.
    #[default]
    Legato = 0,
    /// Crisp tongued staccato attack.
    TonguedStaccato = 1,
    /// Rapid double-tonguing alternating attack (T-K-T-K).
    DoubleTongue = 2,
    /// Extended flutter-tonguing / Frullato modulation.
    FlutterTongue = 3,
    /// Overblowing to excited upper octave / higher harmonic registers.
    OverblowHarmonic = 4,
    /// Slap-tonguing percussive release pop.
    SlapTongue = 5,
    /// Mechanical key click percussion impulse without air jet.
    KeyClick = 6,
    /// Expressive sinusoidal diaphragmatic breath vibrato.
    Vibrato = 7,
    /// Vocal growling guttural multiphonic timbral excitation.
    Growl = 8,
}

impl WoodwindArticulation {
    pub fn from_u32(val: u32) -> Self {
        match val {
            0 => Self::Legato,
            1 => Self::TonguedStaccato,
            2 => Self::DoubleTongue,
            3 => Self::FlutterTongue,
            4 => Self::OverblowHarmonic,
            5 => Self::SlapTongue,
            6 => Self::KeyClick,
            7 => Self::Vibrato,
            8 => Self::Growl,
            _ => Self::Legato,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Legato => "Legato",
            Self::TonguedStaccato => "Tongued Staccato",
            Self::DoubleTongue => "Double Tongue",
            Self::FlutterTongue => "Flutter Tongue",
            Self::OverblowHarmonic => "Overblow Harmonic",
            Self::SlapTongue => "Slap Tongue",
            Self::KeyClick => "Key Click",
            Self::Vibrato => "Breath Vibrato",
            Self::Growl => "Vocal Growl",
        }
    }

    /// Returns nominal default $(\text{pressure\_kpa}, \text{jet\_dist\_mm}, \text{angle\_rad}, \text{noise})$ parameters.
    pub fn nominal_parameters(&self) -> (f32, f32, f32, f32) {
        match self {
            Self::Legato => (1.25, 7.0, 0.0, 0.08),
            Self::TonguedStaccato => (1.60, 6.0, 0.05, 0.12),
            Self::DoubleTongue => (1.45, 6.5, 0.0, 0.10),
            Self::FlutterTongue => (1.80, 7.5, -0.05, 0.22),
            Self::OverblowHarmonic => (2.80, 4.5, 0.15, 0.06),
            Self::SlapTongue => (0.30, 8.0, -0.20, 0.35),
            Self::KeyClick => (0.05, 10.0, 0.0, 0.02),
            Self::Vibrato => (1.30, 7.0, 0.0, 0.08),
            Self::Growl => (2.10, 8.5, -0.10, 0.30),
        }
    }
}

/// Instantaneous lock-free snapshot of woodwind breath bus parameters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BreathBusSnapshot {
    pub articulation: WoodwindArticulation,
    pub blowing_pressure_kpa: f32,
    pub jet_distance_mm: f32,
    pub embouchure_angle_rad: f32,
    pub fingering_mask: u8,
    pub vibrato_depth: f32,
    pub vibrato_rate_hz: f32,
    pub flutter_rate_hz: f32,
    pub breath_noise_mix: f32,
    pub gesture_progress: f32,
    pub is_active: bool,
}

impl Default for BreathBusSnapshot {
    fn default() -> Self {
        Self {
            articulation: WoodwindArticulation::Legato,
            blowing_pressure_kpa: 1.25,
            jet_distance_mm: 7.0,
            embouchure_angle_rad: 0.0,
            fingering_mask: 0x3F, // All 6 holes closed (fundamental)
            vibrato_depth: 0.0,
            vibrato_rate_hz: 5.5,
            flutter_rate_hz: 0.0,
            breath_noise_mix: 0.08,
            gesture_progress: 0.0,
            is_active: false,
        }
    }
}

/// Lock-free physical woodwind breath & fingering parameter routing bus.
pub struct BreathBus {
    articulation_raw: AtomicU32,
    pressure_bits: AtomicU32,
    jet_dist_bits: AtomicU32,
    angle_bits: AtomicU32,
    fingering_mask_raw: AtomicU32,
    vib_depth_bits: AtomicU32,
    vib_rate_bits: AtomicU32,
    flutter_rate_bits: AtomicU32,
    noise_bits: AtomicU32,
    progress_bits: AtomicU32,
    flags: AtomicU32,
}

impl Default for BreathBus {
    fn default() -> Self {
        Self::new()
    }
}

impl BreathBus {
    /// Creates a new BreathBus initialized to standard Legato defaults.
    pub fn new() -> Self {
        let (p, d, a, n) = WoodwindArticulation::Legato.nominal_parameters();
        Self {
            articulation_raw: AtomicU32::new(WoodwindArticulation::Legato as u32),
            pressure_bits: AtomicU32::new(p.to_bits()),
            jet_dist_bits: AtomicU32::new(d.to_bits()),
            angle_bits: AtomicU32::new(a.to_bits()),
            fingering_mask_raw: AtomicU32::new(0x3F), // 6 holes closed
            vib_depth_bits: AtomicU32::new(0.0f32.to_bits()),
            vib_rate_bits: AtomicU32::new(5.5f32.to_bits()),
            flutter_rate_bits: AtomicU32::new(0.0f32.to_bits()),
            noise_bits: AtomicU32::new(n.to_bits()),
            progress_bits: AtomicU32::new(0.0f32.to_bits()),
            flags: AtomicU32::new(0),
        }
    }

    /// Sets the active woodwind articulation and loads nominal defaults.
    pub fn set_articulation(&self, articulation: WoodwindArticulation) {
        self.articulation_raw.store(articulation as u32, Ordering::Release);
        let (p, d, a, n) = articulation.nominal_parameters();
        self.set_breath_vector(p, d, a, n);
    }

    /// Gets the active woodwind articulation.
    pub fn get_articulation(&self) -> WoodwindArticulation {
        WoodwindArticulation::from_u32(self.articulation_raw.load(Ordering::Acquire))
    }

    /// Sets continuous breath gesture vector $(P_m, d_{jet}, \theta_{jet}, \text{noise})$.
    pub fn set_breath_vector(
        &self,
        pressure_kpa: f32,
        jet_distance_mm: f32,
        embouchure_angle_rad: f32,
        breath_noise_mix: f32,
    ) {
        self.pressure_bits.store(pressure_kpa.clamp(0.05, 4.0).to_bits(), Ordering::Release);
        self.jet_dist_bits.store(jet_distance_mm.clamp(1.5, 15.0).to_bits(), Ordering::Release);
        self.angle_bits.store(embouchure_angle_rad.clamp(-0.5, 0.5).to_bits(), Ordering::Release);
        self.noise_bits.store(breath_noise_mix.clamp(0.0, 0.50).to_bits(), Ordering::Release);
    }

    /// Gets continuous breath gesture vector $(P_m, d_{jet}, \theta_{jet}, \text{noise})$.
    pub fn get_breath_vector(&self) -> (f32, f32, f32, f32) {
        (
            f32::from_bits(self.pressure_bits.load(Ordering::Acquire)),
            f32::from_bits(self.jet_dist_bits.load(Ordering::Acquire)),
            f32::from_bits(self.angle_bits.load(Ordering::Acquire)),
            f32::from_bits(self.noise_bits.load(Ordering::Acquire)),
        )
    }

    /// Sets 6-tonehole fingering bitmask (lower 6 bits).
    pub fn set_fingering_mask(&self, mask: u8) {
        self.fingering_mask_raw.store((mask & 0x3F) as u32, Ordering::Release);
    }

    /// Gets 6-tonehole fingering bitmask.
    pub fn get_fingering_mask(&self) -> u8 {
        (self.fingering_mask_raw.load(Ordering::Acquire) & 0x3F) as u8
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

    /// Sets flutter tongue modulation frequency in Hz.
    pub fn set_flutter_tongue(&self, flutter_rate_hz: f32) {
        self.flutter_rate_bits.store(flutter_rate_hz.clamp(0.0, 50.0).to_bits(), Ordering::Release);
    }

    /// Gets flutter tongue modulation frequency in Hz.
    pub fn get_flutter_tongue(&self) -> f32 {
        f32::from_bits(self.flutter_rate_bits.load(Ordering::Acquire))
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

    /// Captures a complete atomic snapshot of the breath bus.
    pub fn snapshot(&self) -> BreathBusSnapshot {
        let articulation = self.get_articulation();
        let (blowing_pressure_kpa, jet_distance_mm, embouchure_angle_rad, breath_noise_mix) =
            self.get_breath_vector();
        let fingering_mask = self.get_fingering_mask();
        let (vibrato_depth, vibrato_rate_hz) = self.get_vibrato();
        let flutter_rate_hz = self.get_flutter_tongue();
        let progress = f32::from_bits(self.progress_bits.load(Ordering::Acquire));
        let flags = self.flags.load(Ordering::Acquire);

        BreathBusSnapshot {
            articulation,
            blowing_pressure_kpa,
            jet_distance_mm,
            embouchure_angle_rad,
            fingering_mask,
            vibrato_depth,
            vibrato_rate_hz,
            flutter_rate_hz,
            breath_noise_mix,
            gesture_progress: progress,
            is_active: (flags & 1) != 0,
        }
    }

    /// Applies a complete snapshot into this bus.
    pub fn apply_snapshot(&self, snap: &BreathBusSnapshot) {
        self.articulation_raw.store(snap.articulation as u32, Ordering::Release);
        self.pressure_bits.store(snap.blowing_pressure_kpa.to_bits(), Ordering::Release);
        self.jet_dist_bits.store(snap.jet_distance_mm.to_bits(), Ordering::Release);
        self.angle_bits.store(snap.embouchure_angle_rad.to_bits(), Ordering::Release);
        self.fingering_mask_raw.store(snap.fingering_mask as u32, Ordering::Release);
        self.vib_depth_bits.store(snap.vibrato_depth.to_bits(), Ordering::Release);
        self.vib_rate_bits.store(snap.vibrato_rate_hz.to_bits(), Ordering::Release);
        self.flutter_rate_bits.store(snap.flutter_rate_hz.to_bits(), Ordering::Release);
        self.noise_bits.store(snap.breath_noise_mix.to_bits(), Ordering::Release);
        self.progress_bits.store(snap.gesture_progress.to_bits(), Ordering::Release);
        self.flags.store(if snap.is_active { 1 } else { 0 }, Ordering::Release);
    }

    /// Dispatches breath bus values into `ParamBus` handles.
    pub fn dispatch_to_param_bus(&self, bus: &ParamBus, base_param_id: ParamId) {
        let snap = self.snapshot();
        bus.set(ParamId(base_param_id.0), snap.articulation as u32 as f32);
        bus.set(ParamId(base_param_id.0 + 1), snap.blowing_pressure_kpa);
        bus.set(ParamId(base_param_id.0 + 2), snap.jet_distance_mm);
        bus.set(ParamId(base_param_id.0 + 3), snap.embouchure_angle_rad);
        bus.set(ParamId(base_param_id.0 + 4), snap.fingering_mask as f32);
        bus.set(ParamId(base_param_id.0 + 5), snap.vibrato_depth);
        bus.set(ParamId(base_param_id.0 + 6), snap.vibrato_rate_hz);
        bus.set(ParamId(base_param_id.0 + 7), snap.flutter_rate_hz);
        bus.set(ParamId(base_param_id.0 + 8), snap.breath_noise_mix);
        bus.set(ParamId(base_param_id.0 + 9), snap.gesture_progress);
    }

    /// Synchronizes breath bus values from `ParamBus` handles.
    pub fn sync_from_param_bus(&self, bus: &ParamBus, base_param_id: ParamId) {
        let art_idx = bus.get(ParamId(base_param_id.0)).unwrap_or(0.0).max(0.0).round() as u32;
        let pressure = bus.get(ParamId(base_param_id.0 + 1)).unwrap_or(1.25);
        let distance = bus.get(ParamId(base_param_id.0 + 2)).unwrap_or(7.0);
        let angle = bus.get(ParamId(base_param_id.0 + 3)).unwrap_or(0.0);
        let fingering = bus.get(ParamId(base_param_id.0 + 4)).unwrap_or(63.0).max(0.0).round() as u8;
        let vib_d = bus.get(ParamId(base_param_id.0 + 5)).unwrap_or(0.0);
        let vib_r = bus.get(ParamId(base_param_id.0 + 6)).unwrap_or(5.5);
        let flutter = bus.get(ParamId(base_param_id.0 + 7)).unwrap_or(0.0);
        let noise = bus.get(ParamId(base_param_id.0 + 8)).unwrap_or(0.08);
        let prog = bus.get(ParamId(base_param_id.0 + 9)).unwrap_or(0.0);

        self.articulation_raw.store(art_idx, Ordering::Release);
        self.set_breath_vector(pressure, distance, angle, noise);
        self.set_fingering_mask(fingering);
        self.set_vibrato(vib_d, vib_r);
        self.set_flutter_tongue(flutter);
        self.set_progress(prog, true);
    }
}
