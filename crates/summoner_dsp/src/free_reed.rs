// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Free-Reed Aerophone Engine (Milestone 28).
//!
//! Provides a high-precision physical acoustic modeling synthesizer for free-reed aerophones
//! (Bandoneon, Accordion, Bayan, Harmonium, Concertina) featuring:
//! - Dual-chamber dynamic bellows airflow pressure mechanics (P_bellows in [-1200, +1200] Pa with push/pull asymmetry)
//! - Non-linear brass/steel free-reed aeroelastic oscillation (geometric slot clearance, non-linear flow detachment, pressure-dependent pitch shift)
//! - Multi-rank tremolo / musette detune phase interference (16' Bassoon, 8' Clarinet, 8'+/- Musette detuned ranks, 4' Piccolo)
//! - Pallet valve acoustic opening transients (t_valve in [2, 40] ms)
//! - Wooden Cassotto tone chamber acoustic waveguide filtering with soundbox cavity resonance
//!
//! Enforces zero heap allocations in audio render loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};
use crate::cassotto_chamber::{CassottoChamber, CassottoProfile};
use crate::traits::SignalProcessor;

/// Maximum number of polyphonic Free-Reed voices.
pub const MAX_FREE_REED_VOICES: usize = 16;

/// Total number of physical reed ranks per voice.
pub const NUM_REED_RANKS: usize = 5;

// Rank Indices
pub const RANK_BASSOON_16: usize = 0;   // 16' Sub-octave (in Cassotto)
pub const RANK_CLARINET_8: usize = 1;   // 8' Master pitch (in Cassotto)
pub const RANK_MUSETTE_PLUS_8: usize = 2; // 8'+ Sharp detuned (+8 to +25 cents, direct)
pub const RANK_MUSETTE_MINUS_8: usize = 3;// 8'- Flat detuned (-8 to -25 cents, direct)
pub const RANK_PICCOLO_4: usize = 4;    // 4' High octave (direct)

// Register Switch Bitmasks
pub const REGISTER_MASTER: u32 = 0b11111;     // All 5 ranks active
pub const REGISTER_BANDONEON: u32 = (1 << RANK_BASSOON_16) | (1 << RANK_CLARINET_8); // 16' + 8' (Pure Cassotto)
pub const REGISTER_ACCORDION: u32 = (1 << RANK_BASSOON_16) | (1 << RANK_CLARINET_8) | (1 << RANK_MUSETTE_PLUS_8); // 16' + 8' + 8'+
pub const REGISTER_MUSETTE: u32 = (1 << RANK_CLARINET_8) | (1 << RANK_MUSETTE_PLUS_8) | (1 << RANK_MUSETTE_MINUS_8); // 8' + 8'+ + 8'-
pub const REGISTER_CLARINET: u32 = 1 << RANK_CLARINET_8; // 8' Solo Cassotto
pub const REGISTER_BASSOON: u32 = 1 << RANK_BASSOON_16;  // 16' Deep Cassotto
pub const REGISTER_PICCOLO: u32 = 1 << RANK_PICCOLO_4;   // 4' Solo High
pub const REGISTER_HARMONICON: u32 = (1 << RANK_CLARINET_8) | (1 << RANK_MUSETTE_PLUS_8); // 8' + 8'+ subtle beating

/// Free-Reed aerophone sound preset profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FreeReedProfile {
    /// 142-Tone Tango Bandoneon (Alfred Arnold style - punchy zinc reedplate, dry 16'+8' cassotto bite).
    #[default]
    TangoBandoneon,
    /// French Musette Accordion (Triple 8' musette shimmer, warm maple cassotto body).
    FrenchMusetteAccordion,
    /// Russian Chromatic Bayan (Solid duralumin reedblocks, thunderous bass, wide expressive dynamic range).
    RussianBayan,
    /// Vintage Suction Harmonium (Damped brass free-reeds in continuous pumping windbox).
    VintageHarmonium,
    /// English Treble Concertina (Fast responding steel reeds, compact crisp transients).
    EnglishConcertina,
}

impl FreeReedProfile {
    pub fn name(&self) -> &'static str {
        match self {
            Self::TangoBandoneon => "Alfred Arnold Tango Bandoneon (142-Tone)",
            Self::FrenchMusetteAccordion => "French Musette Accordion (Triple 8')",
            Self::RussianBayan => "Russian Chromatic Bayan (Solid Reedblock)",
            Self::VintageHarmonium => "Vintage Suction Harmonium (Brass Reeds)",
            Self::EnglishConcertina => "English Treble Concertina (Steel Reeds)",
        }
    }

    pub fn to_cassotto_profile(&self) -> CassottoProfile {
        match self {
            Self::TangoBandoneon => CassottoProfile::DobleCassottoBandoneon,
            Self::FrenchMusetteAccordion => CassottoProfile::StandardAccordionChamber,
            Self::RussianBayan => CassottoProfile::OpenAirBayan,
            Self::VintageHarmonium => CassottoProfile::HarmoniumResonantBox,
            Self::EnglishConcertina => CassottoProfile::ConcertinaSpruceCavity,
        }
    }

    /// Returns nominal $(P_{\text{nominal\_pa}}, \text{musette\_detune\_cents}, t_{\text{valve\_ms}}, \text{reed\_stiffness}, \text{push\_pull\_asymmetry})$.
    pub fn nominal_physics(&self) -> (f32, f32, f32, f32, f32) {
        match self {
            Self::TangoBandoneon => (550.0, 1.5, 6.0, 1.25, 1.08),
            Self::FrenchMusetteAccordion => (420.0, 16.0, 12.0, 1.00, 1.04),
            Self::RussianBayan => (650.0, 4.0, 8.0, 1.40, 1.02),
            Self::VintageHarmonium => (280.0, 8.0, 25.0, 0.85, 1.00),
            Self::EnglishConcertina => (480.0, 0.0, 4.0, 1.15, 1.05),
        }
    }

    pub fn default_register_mask(&self) -> u32 {
        match self {
            Self::TangoBandoneon => REGISTER_BANDONEON,
            Self::FrenchMusetteAccordion => REGISTER_MUSETTE,
            Self::RussianBayan => REGISTER_MASTER,
            Self::VintageHarmonium => REGISTER_ACCORDION,
            Self::EnglishConcertina => REGISTER_CLARINET,
        }
    }
}

/// Aeroelastic state of an individual vibrating free-reed tongue.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct FreeReedOscillator {
    /// Continuous phase accumulator wrapped modulo 2*PI.
    pub phase: f32,
    /// Instantaneous normalized reed tip displacement y in [-1.0 .. 1.0].
    pub displacement: f32,
    /// Instantaneous reed tip velocity dy/dt.
    pub velocity: f32,
    /// Previous volume flow rate for pressure gradient derivation.
    pub prev_flow: f32,
}

impl FreeReedOscillator {
    pub fn new() -> Self {
        Self {
            phase: 0.0,
            displacement: 0.0,
            velocity: 0.0,
            prev_flow: 0.0,
        }
    }

    /// Advance free-reed aeroelastic oscillation for a single sample.
    /// `freq_hz`: nominal fundamental frequency.
    /// `pressure_diff_pa`: instantaneous driving pressure difference across reed plate.
    /// `sample_rate`: system sample rate.
    /// `reed_stiffness`: reed material stiffness multiplier.
    /// `is_push`: true if bellows is in push stroke (compressing), false if pull (expanding).
    #[inline(always)]
    pub fn process_step(
        &mut self,
        freq_hz: f32,
        pressure_diff_pa: f32,
        sample_rate: f32,
        reed_stiffness: f32,
        is_push: bool,
    ) -> f32 {
        let abs_p = pressure_diff_pa.abs();
        if abs_p < 2.0 {
            // Reed at rest under negligible pressure
            self.displacement *= 0.992;
            self.velocity *= 0.990;
            self.prev_flow = 0.0;
            return 0.0;
        }

        // Pressure-dependent aeroelastic pitch sag: high pressure drops pitch slightly (~1-15 cents)
        let pitch_sag_ratio = (abs_p / 1200.0).clamp(0.0, 1.0) * 0.008;
        let effective_freq = freq_hz * (1.0 - pitch_sag_ratio);

        let dt = 1.0 / sample_rate;
        let omega = TAU * effective_freq;

        // Advance phase state
        self.phase += omega * dt;
        if self.phase >= TAU {
            self.phase %= TAU;
        }

        // Aeroelastic flow coupling: asymmetry between push and pull stroke
        let geom_asymmetry = if is_push { 1.0 } else { 0.96 };
        let driving_force = (abs_p / 400.0).clamp(0.0, 3.0) * geom_asymmetry;

        // 2nd-order non-linear Duffing aeroelastic oscillator approximation:
        // y'' + 2*gamma*y' + omega^2 * (y + alpha*y^3) = F_aero(t)
        let natural_displacement = self.phase.sin();
        let target_disp = natural_displacement * driving_force.min(1.2);
        
        let damping = 0.08 * omega;
        let non_lin_k = 1.0 + 0.3 * (self.displacement * self.displacement) * reed_stiffness;
        let spring_accel = -(omega * omega) * non_lin_k * (self.displacement - target_disp);
        let damp_accel = -damping * self.velocity;
        
        let total_accel = spring_accel + damp_accel;
        self.velocity += total_accel * dt;
        self.velocity = self.velocity.clamp(-1200.0, 1200.0);
        self.displacement += self.velocity * dt;
        self.displacement = self.displacement.clamp(-1.8, 1.8);

        // Bernoulli volume flow detachment through slot:
        // Slot clearance h0 ~ 0.05, flow detached when displacement exceeds clearance
        let clearance_h0 = 0.06;
        let open_fraction = (self.displacement.abs() - clearance_h0).max(0.02);
        let jet_velocity = (2.0 * abs_p / 1.204).sqrt(); // Air density rho = 1.204 kg/m^3
        let instant_flow = open_fraction * jet_velocity * if is_push { 1.0 } else { -1.0 };

        // Acoustic pressure radiated is proportional to time derivative of volume flow
        let acoustic_out = (instant_flow - self.prev_flow) * 0.015;
        self.prev_flow = instant_flow;

        acoustic_out.clamp(-2.0, 2.0)
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
        self.displacement = 0.0;
        self.velocity = 0.0;
        self.prev_flow = 0.0;
    }
}

/// A polyphonic voice instance of the Free-Reed engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeReedVoice {
    pub active: bool,
    pub note: u8,
    pub freq: f32,
    pub velocity: f32,
    /// Pallet valve opening state $[0.0 ..= 1.0]$.
    pub valve_opening: f32,
    /// Target valve opening for envelope slew ($1.0$ on note_on, $0.0$ on note_off).
    pub target_valve: f32,
    /// 5 physical reed rank oscillators (16', 8', 8'+, 8'-, 4').
    pub ranks: [FreeReedOscillator; NUM_REED_RANKS],
}

impl Default for FreeReedVoice {
    fn default() -> Self {
        Self::new()
    }
}

impl FreeReedVoice {
    pub fn new() -> Self {
        Self {
            active: false,
            note: 60,
            freq: 261.63,
            velocity: 0.0,
            valve_opening: 0.0,
            target_valve: 0.0,
            ranks: [FreeReedOscillator::new(); NUM_REED_RANKS],
        }
    }

    pub fn note_on(&mut self, note: u8, velocity: f32) {
        self.active = true;
        self.note = note;
        self.freq = 440.0 * 2.0f32.powf((note as f32 - 69.0) / 12.0);
        self.velocity = velocity.clamp(0.01, 1.0);
        self.target_valve = 1.0;
    }

    pub fn note_off(&mut self) {
        self.target_valve = 0.0;
    }

    pub fn reset(&mut self) {
        self.active = false;
        self.valve_opening = 0.0;
        self.target_valve = 0.0;
        for r in &mut self.ranks {
            r.reset();
        }
    }
}

/// High-Precision Physical Modeling Free-Reed Aerophone Synthesizer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeReed {
    pub profile: FreeReedProfile,
    pub sample_rate: u32,
    /// Target driving bellows pressure in Pascals $[-1200.0 ..= +1200.0]$ (positive = push, negative = pull).
    pub target_bellows_pressure_pa: f32,
    /// Instantaneous filtered bellows pressure.
    pub current_bellows_pressure_pa: f32,
    /// Active register switch bitmask.
    pub register_mask: u32,
    /// Musette detuning spread in cents $[0.0 ..= 35.0]$.
    pub musette_detune_cents: f32,
    /// Pallet valve opening transient slew time in milliseconds $[2.0 ..= 40.0]$.
    pub valve_slew_ms: f32,
    /// Reed material stiffness factor $[0.5 ..= 2.0]$.
    pub reed_stiffness: f32,
    /// Master acoustic output gain $[0.0 ..= 2.0]$.
    pub master_gain: f32,
    /// Cassotto tone chamber and soundbox resonator.
    pub cassotto: CassottoChamber,
    /// Polyphonic voices allocated in zero-heap buffer.
    pub voices: [FreeReedVoice; MAX_FREE_REED_VOICES],
}

impl Default for FreeReed {
    fn default() -> Self {
        Self::new(48000)
    }
}

impl FreeReed {
    pub fn new(sample_rate: u32) -> Self {
        let sr = sample_rate.max(8000);
        let profile = FreeReedProfile::TangoBandoneon;
        let (nom_p, musette_cents, valve_ms, stiffness, _) = profile.nominal_physics();

        let mut inst = Self {
            profile,
            sample_rate: sr,
            target_bellows_pressure_pa: nom_p,
            current_bellows_pressure_pa: nom_p,
            register_mask: profile.default_register_mask(),
            musette_detune_cents: musette_cents,
            valve_slew_ms: valve_ms,
            reed_stiffness: stiffness,
            master_gain: 0.85,
            cassotto: CassottoChamber::new(sr),
            voices: std::array::from_fn(|_| FreeReedVoice::new()),
        };
        inst.cassotto.set_profile(profile.to_cassotto_profile());
        inst
    }

    pub fn set_profile(&mut self, profile: FreeReedProfile) {
        self.profile = profile;
        let (nom_p, musette_cents, valve_ms, stiffness, _) = profile.nominal_physics();
        self.target_bellows_pressure_pa = nom_p;
        self.musette_detune_cents = musette_cents;
        self.valve_slew_ms = valve_ms;
        self.reed_stiffness = stiffness;
        self.register_mask = profile.default_register_mask();
        self.cassotto.set_profile(profile.to_cassotto_profile());
    }

    pub fn set_bellows_pressure(&mut self, pressure_pa: f32) {
        self.target_bellows_pressure_pa = pressure_pa.clamp(-1200.0, 1200.0);
    }

    pub fn set_register_mask(&mut self, mask: u32) {
        self.register_mask = mask & REGISTER_MASTER;
    }

    pub fn set_musette_detune(&mut self, cents: f32) {
        self.musette_detune_cents = cents.clamp(0.0, 35.0);
    }

    pub fn set_cassotto_aperture(&mut self, aperture: f32) {
        self.cassotto.set_aperture(aperture);
    }

    pub fn note_on(&mut self, note: u8, velocity: f32) {
        // Find existing voice on same note or steal oldest inactive
        let mut slot = None;
        for (i, v) in self.voices.iter().enumerate() {
            if v.active && v.note == note {
                slot = Some(i);
                break;
            }
        }
        if slot.is_none() {
            for (i, v) in self.voices.iter().enumerate() {
                if !v.active {
                    slot = Some(i);
                    break;
                }
            }
        }
        let voice_idx = slot.unwrap_or(0);
        self.voices[voice_idx].note_on(note, velocity);
    }

    pub fn note_off(&mut self, note: u8) {
        for v in &mut self.voices {
            if v.active && v.note == note {
                v.note_off();
            }
        }
    }

    pub fn all_notes_off(&mut self) {
        for v in &mut self.voices {
            v.note_off();
        }
    }

    pub fn reset(&mut self) {
        for v in &mut self.voices {
            v.reset();
        }
        self.cassotto.reset();
        self.current_bellows_pressure_pa = self.target_bellows_pressure_pa;
    }

    /// Process a single stereo audio sample pair from all active free-reed voices and the cassotto chamber.
    #[inline(always)]
    pub fn process_sample(&mut self) -> (f32, f32) {
        // Slew bellows pressure dynamically (smooth bellows inertia)
        let pressure_slew = 0.005;
        self.current_bellows_pressure_pa += (self.target_bellows_pressure_pa - self.current_bellows_pressure_pa) * pressure_slew;

        let p_bellows = self.current_bellows_pressure_pa;
        let is_push = p_bellows >= 0.0;
        let abs_p = p_bellows.abs();
        let sr = self.sample_rate as f32;

        let valve_rate = 1.0 / ((self.valve_slew_ms * 0.001 * sr).max(1.0));

        let mut cassotto_sum = 0.0f32;
        let mut direct_sum = 0.0f32;

        let musette_mult_plus = 2.0f32.powf(self.musette_detune_cents / 1200.0);
        let musette_mult_minus = 2.0f32.powf(-self.musette_detune_cents / 1200.0);

        for voice in &mut self.voices {
            if !voice.active {
                continue;
            }

            // Slew pallet valve opening
            if voice.valve_opening < voice.target_valve {
                voice.valve_opening = (voice.valve_opening + valve_rate).min(voice.target_valve);
            } else if voice.valve_opening > voice.target_valve {
                voice.valve_opening = (voice.valve_opening - valve_rate).max(voice.target_valve);
                if voice.valve_opening <= 0.0001 && voice.target_valve <= 0.0 {
                    voice.active = false;
                    for r in &mut voice.ranks {
                        r.reset();
                    }
                    continue;
                }
            }

            let valve_gain = voice.valve_opening * voice.velocity;
            if valve_gain <= 1e-5 {
                continue;
            }

            let base_freq = voice.freq;

            // 1. Rank 0: Bassoon 16' (Cassotto)
            if (self.register_mask & (1 << RANK_BASSOON_16)) != 0 {
                let freq_16 = base_freq * 0.5;
                let s = voice.ranks[RANK_BASSOON_16].process_step(freq_16, abs_p, sr, self.reed_stiffness * 1.15, is_push);
                cassotto_sum += s * valve_gain * 1.10;
            }

            // 2. Rank 1: Clarinet 8' (Cassotto)
            if (self.register_mask & (1 << RANK_CLARINET_8)) != 0 {
                let freq_8 = base_freq;
                let s = voice.ranks[RANK_CLARINET_8].process_step(freq_8, abs_p, sr, self.reed_stiffness, is_push);
                cassotto_sum += s * valve_gain * 1.00;
            }

            // 3. Rank 2: Musette+ 8' (Direct)
            if (self.register_mask & (1 << RANK_MUSETTE_PLUS_8)) != 0 {
                let freq_musette_p = base_freq * musette_mult_plus;
                let s = voice.ranks[RANK_MUSETTE_PLUS_8].process_step(freq_musette_p, abs_p, sr, self.reed_stiffness * 0.98, is_push);
                direct_sum += s * valve_gain * 0.90;
            }

            // 4. Rank 3: Musette- 8' (Direct)
            if (self.register_mask & (1 << RANK_MUSETTE_MINUS_8)) != 0 {
                let freq_musette_m = base_freq * musette_mult_minus;
                let s = voice.ranks[RANK_MUSETTE_MINUS_8].process_step(freq_musette_m, abs_p, sr, self.reed_stiffness * 0.98, is_push);
                direct_sum += s * valve_gain * 0.90;
            }

            // 5. Rank 4: Piccolo 4' (Direct)
            if (self.register_mask & (1 << RANK_PICCOLO_4)) != 0 {
                let freq_4 = base_freq * 2.0;
                let s = voice.ranks[RANK_PICCOLO_4].process_step(freq_4, abs_p, sr, self.reed_stiffness * 0.90, is_push);
                direct_sum += s * valve_gain * 0.75;
            }
        }

        // Process cassotto & direct through soundbox chamber
        let acoustic_mono = self.cassotto.process_sample(cassotto_sum, direct_sum) * self.master_gain;

        // Slight spatial stereo spread from left/right soundbox radiation
        let left = (acoustic_mono * 1.02 + cassotto_sum * 0.05).clamp(-1.0, 1.0);
        let right = (acoustic_mono * 0.98 + direct_sum * 0.05).clamp(-1.0, 1.0);

        (left, right)
    }
}

impl SignalProcessor for FreeReed {
    fn name(&self) -> &str {
        "FreeReed"
    }

    fn process_block(
        &mut self,
        _inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        _ctx: &ProcessContext,
    ) {
        if outputs.is_empty() {
            return;
        }

        let num_samples = outputs[0].len();
        if outputs.len() >= 2 {
            let (left_chan, right_chan) = outputs.split_at_mut(1);
            let left = &mut left_chan[0][..num_samples];
            let right = &mut right_chan[0][..num_samples];
            for i in 0..num_samples {
                let (l, r) = self.process_sample();
                left[i] = l;
                right[i] = r;
            }
        } else {
            let out = &mut outputs[0][..num_samples];
            for sample in out.iter_mut() {
                let (l, r) = self.process_sample();
                *sample = (l + r) * 0.5;
            }
        }
    }
}

/// AudioNode wrapper for FreeReed physical modeling synthesis engine.
#[derive(Debug)]
pub struct FreeReedNode {
    pub free_reed: FreeReed,
}

impl FreeReedNode {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            free_reed: FreeReed::new(sample_rate),
        }
    }
}

impl AudioNode for FreeReedNode {
    fn name(&self) -> &str {
        "FreeReedNode"
    }

    fn process(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        self.free_reed.process_block(inputs, outputs, ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_free_reed_profiles_and_registers() {
        let mut reed = FreeReed::new(48000);

        for prof in [
            FreeReedProfile::TangoBandoneon,
            FreeReedProfile::FrenchMusetteAccordion,
            FreeReedProfile::RussianBayan,
            FreeReedProfile::VintageHarmonium,
            FreeReedProfile::EnglishConcertina,
        ] {
            reed.set_profile(prof);
            assert!(!prof.name().is_empty());
            assert_eq!(reed.profile, prof);

            reed.note_on(60, 0.85);
            let mut non_zero = false;
            for _ in 0..500 {
                let (l, r) = reed.process_sample();
                assert!(l.is_finite());
                assert!(r.is_finite());
                if l.abs() > 0.001 || r.abs() > 0.001 {
                    non_zero = true;
                }
            }
            assert!(non_zero, "Free reed must produce audio for profile {}", prof.name());

            reed.note_off(60);
            for _ in 0..1000 {
                reed.process_sample();
            }
        }
    }

    #[test]
    fn test_free_reed_push_pull_asymmetry() {
        let mut reed = FreeReed::new(48000);
        reed.set_profile(FreeReedProfile::TangoBandoneon);

        // Push test (+600 Pa)
        reed.set_bellows_pressure(600.0);
        reed.note_on(60, 0.90);
        let mut push_energy = 0.0f32;
        for _ in 0..1000 {
            let (l, r) = reed.process_sample();
            push_energy += l * l + r * r;
        }

        reed.reset();

        // Pull test (-600 Pa)
        reed.set_bellows_pressure(-600.0);
        reed.note_on(60, 0.90);
        let mut pull_energy = 0.0f32;
        for _ in 0..1000 {
            let (l, r) = reed.process_sample();
            pull_energy += l * l + r * r;
        }

        assert!(push_energy > 0.0);
        assert!(pull_energy > 0.0);
        // Due to physical asymmetric clearance, push and pull energy differ slightly
        assert!((push_energy - pull_energy).abs() > 1e-4);
    }

    #[test]
    fn test_free_reed_zero_allocation_allocguard() {
        use summoner_core::allocator::AllocGuard;
        use summoner_core::transport::Transport;

        let sample_rate = 48000;
        let mut reed = FreeReed::new(sample_rate);
        reed.set_profile(FreeReedProfile::TangoBandoneon);
        reed.note_on(60, 0.85);

        let transport = Transport::new(sample_rate, 120.0);
        let ctx = ProcessContext::from_transport(&transport);

        let mut out_l = [0.0f32; 64];
        let mut out_r = [0.0f32; 64];
        let in_dummy: [&[Sample]; 0] = [];

        // Warm up / initialize
        reed.process_block(&in_dummy, &mut [&mut out_l[..], &mut out_r[..]], &ctx);

        // Run under AllocGuard
        let _guard = AllocGuard::new();
        for _ in 0..10 {
            reed.process_block(&in_dummy, &mut [&mut out_l[..], &mut out_r[..]], &ctx);
        }
    }
}
