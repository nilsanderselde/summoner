// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Struck Idiophone Resonator Bank & Mallet Dynamics (Milestone 19).
//!
//! Provides a high-precision modal synthesis engine for struck percussive idiophones
//! (Marimba, Xylophone, Vibraphone with motorized tremolo, Trinidad Steelpan, Kalimba/Mbira tines,
//! and Orchestral Glockenspiel Bells) coupled with non-linear Hertzian mallet contact dynamics,
//! acoustic resonator tube coupling, and sympathetic inter-note bleed.
//!
//! Enforces zero heap allocations in audio processing loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};
use crate::traits::SignalProcessor;

/// Maximum number of modal resonators per idiophone voice.
pub const NUM_IDIOPHONE_MODES: usize = 8;

/// Number of sympathetically coupled notes in the idiophone instrument bank.
pub const NUM_IDIOPHONE_BANK_NOTES: usize = 6;

/// Idiophone instrument preset profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum IdiophoneInstrumentProfile {
    /// African Rosewood Marimba with undercut arch (1 : 4 : 10 harmonic overtones) and tuned resonator pipes.
    #[default]
    MarimbaWood,
    /// Honduran Rosewood Xylophone with shallow undercut (1 : 3 : 9 overtones) and bright sharp transient.
    XylophoneRosewood,
    /// Concert Vibraphone with aluminum alloy bars (1 : 4 : 10 overtones) and motorized rotating disc tremolo.
    VibraphoneAluminum,
    /// Trinidad Tenor Steelpan with concave stamped elliptic shell note areas (1 : 2 : 3 harmonic coupling).
    SteelpanTrinidad,
    /// African Kalimba / Mbira with clamped-free forged steel tines (1 : 6.27 : 17.55 inharmonic overtones).
    KalimbaMbira,
    /// Orchestral Glockenspiel / Bell Lyre with high-carbon steel bars and crystalline high overtones.
    GlockenspielBell,
}

impl IdiophoneInstrumentProfile {
    /// Returns default fundamental frequency in Hz.
    pub fn default_fundamental_hz(&self) -> f32 {
        match self {
            Self::MarimbaWood => 261.63,       // C4
            Self::XylophoneRosewood => 523.25,  // C5
            Self::VibraphoneAluminum => 440.00, // A4
            Self::SteelpanTrinidad => 293.66,   // D4
            Self::KalimbaMbira => 329.63,       // E4
            Self::GlockenspielBell => 1046.50,  // C6
        }
    }

    /// Returns nominal modal frequency ratios relative to fundamental $f_0$.
    pub fn modal_ratios(&self) -> [f32; NUM_IDIOPHONE_MODES] {
        match self {
            // Marimba: 1.0 (fundamental), 4.0 (2 octaves up), 10.0 (3 octaves + major 3rd), plus higher modes
            Self::MarimbaWood => [1.000, 4.000, 10.000, 14.250, 18.600, 24.500, 31.200, 38.800],
            // Xylophone: 1.0 (fundamental), 3.0 (octave + fifth), 9.0 (3 octaves + major 2nd), higher modes
            Self::XylophoneRosewood => [1.000, 3.000, 9.000, 13.500, 18.000, 23.400, 29.800, 37.100],
            // Vibraphone: 1.0, 4.0, 10.0, with clean metallic sustain
            Self::VibraphoneAluminum => [1.000, 3.980, 9.950, 15.200, 21.400, 28.100, 35.600, 43.900],
            // Steelpan: 1.0, 2.0 (octave), 3.0 (fifth above octave), plus shell cross-modes
            Self::SteelpanTrinidad => [1.000, 2.000, 3.000, 4.120, 5.350, 6.780, 8.420, 10.250],
            // Kalimba clamped-free bar: 1.0, 6.267, 17.55, 34.39 (Rayleigh beam theory)
            Self::KalimbaMbira => [1.000, 6.267, 17.550, 34.390, 56.840, 84.880, 118.50, 157.70],
            // Glockenspiel: 1.0, 2.76, 5.40, 8.93 (Uniform free-free rectangular steel bar)
            Self::GlockenspielBell => [1.000, 2.756, 5.404, 8.933, 13.345, 18.640, 24.815, 31.870],
        }
    }

    /// Returns nominal modal decay times $T_{60}$ in seconds for each mode.
    pub fn modal_t60s(&self) -> [f32; NUM_IDIOPHONE_MODES] {
        match self {
            Self::MarimbaWood => [3.2, 1.4, 0.6, 0.3, 0.15, 0.08, 0.04, 0.02],
            Self::XylophoneRosewood => [1.8, 0.8, 0.35, 0.18, 0.09, 0.05, 0.025, 0.012],
            Self::VibraphoneAluminum => [6.5, 4.2, 2.8, 1.6, 0.9, 0.5, 0.25, 0.12],
            Self::SteelpanTrinidad => [4.0, 2.8, 2.0, 1.2, 0.7, 0.4, 0.2, 0.1],
            Self::KalimbaMbira => [5.5, 1.8, 0.6, 0.2, 0.08, 0.03, 0.015, 0.008],
            Self::GlockenspielBell => [7.0, 4.5, 3.0, 2.0, 1.2, 0.7, 0.35, 0.18],
        }
    }

    /// Returns nominal modal amplitudes $[A_0, \dots, A_7]$.
    pub fn modal_amplitudes(&self) -> [f32; NUM_IDIOPHONE_MODES] {
        match self {
            Self::MarimbaWood => [1.00, 0.75, 0.40, 0.20, 0.10, 0.05, 0.025, 0.01],
            Self::XylophoneRosewood => [1.00, 0.85, 0.55, 0.30, 0.18, 0.09, 0.04, 0.02],
            Self::VibraphoneAluminum => [1.00, 0.70, 0.35, 0.18, 0.08, 0.04, 0.02, 0.01],
            Self::SteelpanTrinidad => [1.00, 0.80, 0.60, 0.35, 0.20, 0.12, 0.06, 0.03],
            Self::KalimbaMbira => [1.00, 0.65, 0.30, 0.12, 0.05, 0.02, 0.01, 0.005],
            Self::GlockenspielBell => [1.00, 0.90, 0.70, 0.50, 0.35, 0.22, 0.12, 0.06],
        }
    }

    /// Name identifier.
    pub fn name(&self) -> &'static str {
        match self {
            Self::MarimbaWood => "Concert Rosewood Marimba",
            Self::XylophoneRosewood => "Orchestral Xylophone",
            Self::VibraphoneAluminum => "Concert Vibraphone",
            Self::SteelpanTrinidad => "Trinidad Tenor Steelpan",
            Self::KalimbaMbira => "African Steel Kalimba",
            Self::GlockenspielBell => "Orchestral Glockenspiel",
        }
    }
}

/// Mallet material classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MalletMaterial {
    /// Soft yarn / padded felt (deep fundamental excitation, long contact ~4ms).
    #[default]
    SoftYarn,
    /// Medium rubber (balanced fundamental and upper harmonic overtones ~2ms).
    MediumRubber,
    /// Hard rosewood / acrylic (sharp cutting attack, high mode excitation ~0.8ms).
    HardWood,
    /// Brass / metal striker (intense metallic transient, maximum high frequencies ~0.3ms).
    BrassMetal,
    /// Thumb fingertip / flesh (Kalimba pluck, soft rounded transient ~3ms).
    FingertipFlesh,
}

impl MalletMaterial {
    /// Returns nominal $(K_{\text{mallet}}, p, \text{contact\_duration\_sec})$ for the material.
    pub fn nominal_properties(&self) -> (f32, f32, f32) {
        match self {
            Self::SoftYarn => (2.0e7, 1.8, 0.0040),
            Self::MediumRubber => (8.0e7, 2.2, 0.0020),
            Self::HardWood => (3.5e8, 2.6, 0.0008),
            Self::BrassMetal => (1.2e9, 3.2, 0.0003),
            Self::FingertipFlesh => (3.0e7, 1.6, 0.0032),
        }
    }
}

/// A second-order resonant bandpass modal filter section.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ModalFilterSection {
    pub freq_hz: f32,
    pub t60_sec: f32,
    pub amplitude: f32,
    pub phase: f32,
    // Filter internal state
    y1: f32,
    y2: f32,
    coeff_a1: f32,
    coeff_a2: f32,
    coeff_b0: f32,
}

impl Default for ModalFilterSection {
    fn default() -> Self {
        Self {
            freq_hz: 440.0,
            t60_sec: 2.0,
            amplitude: 1.0,
            phase: 0.0,
            y1: 0.0,
            y2: 0.0,
            coeff_a1: 0.0,
            coeff_a2: 0.0,
            coeff_b0: 1.0,
        }
    }
}

impl ModalFilterSection {
    /// Updates second-order resonator filter coefficients for the specified sample rate.
    pub fn update_coefficients(&mut self, sample_rate: f32) {
        if sample_rate <= 0.0 || self.freq_hz <= 0.0 {
            return;
        }

        let clamped_freq = self.freq_hz.clamp(10.0, sample_rate * 0.495);
        let omega = 2.0 * PI * clamped_freq / sample_rate;
        let t60 = self.t60_sec.max(0.001);
        // Pole radius r derived from T60 decay: r = exp(-3.0 * ln(10) / (t60 * fs)) = exp(-6.907755 / (t60 * fs))
        let pole_radius = (-6.907755 / (t60 * sample_rate)).exp().clamp(0.0, 0.99999);

        // a1 = -2 * r * cos(omega), a2 = r^2
        self.coeff_a1 = -2.0 * pole_radius * omega.cos();
        self.coeff_a2 = pole_radius * pole_radius;
        self.coeff_b0 = (1.0 - pole_radius * pole_radius).max(1e-6).sqrt() * 0.035 * self.amplitude;
    }

    /// Resets filter history states.
    pub fn reset(&mut self) {
        self.y1 = 0.0;
        self.y2 = 0.0;
    }

    /// Processes a single input excitation sample through this modal resonator.
    #[inline(always)]
    pub fn step(&mut self, input: f32) -> f32 {
        let out = self.coeff_b0 * input - self.coeff_a1 * self.y1 - self.coeff_a2 * self.y2;
        self.y2 = self.y1;
        self.y1 = if out.is_finite() { out } else { 0.0 };
        self.y1
    }
}

/// Non-linear Hertzian mallet contact generator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MalletStrikeGenerator {
    pub material: MalletMaterial,
    pub velocity: f32,
    pub hardness: f32,
    pub strike_pos: f32, // Position along bar (0.0 center, 1.0 node/edge)
    contact_timer_sec: f32,
    contact_duration_sec: f32,
    is_striking: bool,
}

impl Default for MalletStrikeGenerator {
    fn default() -> Self {
        Self {
            material: MalletMaterial::SoftYarn,
            velocity: 0.8,
            hardness: 0.5,
            strike_pos: 0.2,
            contact_timer_sec: 0.0,
            contact_duration_sec: 0.002,
            is_striking: false,
        }
    }
}

impl MalletStrikeGenerator {
    /// Triggers a new mallet strike with the given velocity and hardness.
    pub fn trigger(&mut self, velocity: f32, hardness: f32, strike_pos: f32) {
        self.velocity = velocity.clamp(0.01, 1.0);
        self.hardness = hardness.clamp(0.0, 1.0);
        self.strike_pos = strike_pos.clamp(0.0, 1.0);

        let (_k, _p, base_dur) = self.material.nominal_properties();
        // Contact duration scales inversely with hardness and strike velocity
        let duration_scale = (1.0 - 0.7 * self.hardness) / (0.6 + 0.4 * self.velocity);
        self.contact_duration_sec = (base_dur * duration_scale).clamp(0.0001, 0.010);
        self.contact_timer_sec = 0.0;
        self.is_striking = true;
    }

    /// Evaluates the instantaneous contact force sample at sample rate `fs`.
    pub fn step(&mut self, dt: f32) -> f32 {
        if !self.is_striking {
            return 0.0;
        }

        self.contact_timer_sec += dt;
        if self.contact_timer_sec >= self.contact_duration_sec {
            self.is_striking = false;
            return 0.0;
        }

        // Half-sine / raised cosine compression pulse: F(t) = v * sin(pi * t / T_c)^p
        let phase = (PI * self.contact_timer_sec / self.contact_duration_sec).clamp(0.0, PI);
        let (_, p, _) = self.material.nominal_properties();
        let exponent = p * (0.8 + 0.4 * self.hardness);
        let pulse = phase.sin().powf(exponent);

        // Scale by velocity
        self.velocity * pulse * 2.5
    }

    /// Returns true if mallet is currently in contact with the bar.
    pub fn in_contact(&self) -> bool {
        self.is_striking
    }
}

/// Physical modeling struck idiophone resonator bank voice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StruckIdiophoneResonator {
    pub sample_rate: u32,
    pub instrument: IdiophoneInstrumentProfile,
    pub fundamental_hz: f32,
    pub modes: [ModalFilterSection; NUM_IDIOPHONE_MODES],
    pub mallet: MalletStrikeGenerator,
    /// Quarter-wave acoustic resonator tube tuning (matching fundamental frequency).
    pub tube_resonance_hz: f32,
    pub tube_q: f32,
    pub tube_mix: f32,
    tube_filter: ModalFilterSection,
    /// Vibraphone motorized rotating disc tremolo parameters.
    pub tremolo_rate_hz: f32,
    pub tremolo_depth: f32,
    pub tremolo_phase: f32,
    pub tremolo_enabled: bool,
    /// Sympathetic bleed coupling ratio [0.0 ..= 1.0].
    pub sympathetic_bleed: f32,
}

impl StruckIdiophoneResonator {
    /// Creates a new Struck Idiophone Resonator instance.
    pub fn new(sample_rate: u32) -> Self {
        let sr = sample_rate.max(8000);
        let inst = IdiophoneInstrumentProfile::MarimbaWood;
        let fund = inst.default_fundamental_hz();

        let mut inst_voice = Self {
            sample_rate: sr,
            instrument: inst,
            fundamental_hz: fund,
            modes: [ModalFilterSection::default(); NUM_IDIOPHONE_MODES],
            mallet: MalletStrikeGenerator::default(),
            tube_resonance_hz: fund,
            tube_q: 12.0,
            tube_mix: 0.45,
            tube_filter: ModalFilterSection::default(),
            tremolo_rate_hz: 3.5,
            tremolo_depth: 0.0,
            tremolo_phase: 0.0,
            tremolo_enabled: false,
            sympathetic_bleed: 0.15,
        };

        inst_voice.set_instrument(inst);
        inst_voice
    }

    /// Configures the voice for a specific instrument profile.
    pub fn set_instrument(&mut self, profile: IdiophoneInstrumentProfile) {
        self.instrument = profile;
        self.fundamental_hz = profile.default_fundamental_hz();
        self.tube_resonance_hz = self.fundamental_hz;

        match profile {
            IdiophoneInstrumentProfile::MarimbaWood => {
                self.mallet.material = MalletMaterial::SoftYarn;
                self.tube_mix = 0.50;
                self.tremolo_enabled = false;
                self.tremolo_depth = 0.0;
            }
            IdiophoneInstrumentProfile::XylophoneRosewood => {
                self.mallet.material = MalletMaterial::HardWood;
                self.tube_mix = 0.25;
                self.tremolo_enabled = false;
                self.tremolo_depth = 0.0;
            }
            IdiophoneInstrumentProfile::VibraphoneAluminum => {
                self.mallet.material = MalletMaterial::MediumRubber;
                self.tube_mix = 0.60;
                self.tremolo_enabled = true;
                self.tremolo_depth = 0.75;
                self.tremolo_rate_hz = 4.2;
            }
            IdiophoneInstrumentProfile::SteelpanTrinidad => {
                self.mallet.material = MalletMaterial::MediumRubber;
                self.tube_mix = 0.10;
                self.tremolo_enabled = false;
                self.tremolo_depth = 0.0;
            }
            IdiophoneInstrumentProfile::KalimbaMbira => {
                self.mallet.material = MalletMaterial::FingertipFlesh;
                self.tube_mix = 0.35;
                self.tremolo_enabled = false;
                self.tremolo_depth = 0.0;
            }
            IdiophoneInstrumentProfile::GlockenspielBell => {
                self.mallet.material = MalletMaterial::BrassMetal;
                self.tube_mix = 0.05;
                self.tremolo_enabled = false;
                self.tremolo_depth = 0.0;
            }
        }

        self.recalculate_modes();
    }

    /// Sets the fundamental pitch frequency in Hz.
    pub fn set_frequency(&mut self, freq_hz: f32) {
        self.fundamental_hz = freq_hz.clamp(20.0, 8000.0);
        self.tube_resonance_hz = self.fundamental_hz;
        self.recalculate_modes();
    }

    /// Recalculates all modal resonator filters for the current fundamental and instrument profile.
    pub fn recalculate_modes(&mut self) {
        let sr = self.sample_rate as f32;
        let ratios = self.instrument.modal_ratios();
        let t60s = self.instrument.modal_t60s();
        let amps = self.instrument.modal_amplitudes();

        for i in 0..NUM_IDIOPHONE_MODES {
            self.modes[i].freq_hz = (self.fundamental_hz * ratios[i]).clamp(10.0, sr * 0.495);
            self.modes[i].t60_sec = t60s[i];
            self.modes[i].amplitude = amps[i];
            self.modes[i].update_coefficients(sr);
        }

        // Resonator tube filter
        self.tube_filter.freq_hz = self.tube_resonance_hz.clamp(10.0, sr * 0.495);
        self.tube_filter.t60_sec = 0.6;
        self.tube_filter.amplitude = 1.0;
        self.tube_filter.update_coefficients(sr);
    }

    /// Triggers a mallet strike with strike velocity, mallet hardness, and bar position.
    pub fn trigger_strike(&mut self, velocity: f32, hardness: f32, strike_pos: f32) {
        self.mallet.trigger(velocity, hardness, strike_pos);
    }

    /// Resets all internal filter states.
    pub fn reset(&mut self) {
        for m in &mut self.modes {
            m.reset();
        }
        self.tube_filter.reset();
        self.tremolo_phase = 0.0;
    }

    /// Processes a single audio sample through the idiophone resonator voice.
    pub fn process_sample(&mut self, external_excitation: f32) -> f32 {
        let dt = 1.0 / (self.sample_rate as f32);

        // 1. Generate mallet contact force pulse
        let mallet_force = self.mallet.step(dt);
        let total_excitation = mallet_force + external_excitation;

        // 2. Process modal resonator filter bank
        let mut modal_sum = 0.0f32;
        for mode in &mut self.modes {
            modal_sum += mode.step(total_excitation);
        }

        // 3. Acoustic resonator tube coupling
        let tube_out = self.tube_filter.step(total_excitation);
        let mixed = modal_sum + self.tube_mix * tube_out;

        // 4. Vibraphone motorized rotating disc tremolo AM modulation
        let tremolo_gain = if self.tremolo_enabled && self.tremolo_depth > 0.0 {
            self.tremolo_phase = (self.tremolo_phase + 2.0 * PI * self.tremolo_rate_hz * dt) % (2.0 * PI);
            1.0 - self.tremolo_depth * 0.5 * (1.0 + self.tremolo_phase.sin())
        } else {
            1.0
        };

        let out_sample = mixed * tremolo_gain;
        out_sample.clamp(-1.0, 1.0)
    }
}

impl SignalProcessor for StruckIdiophoneResonator {
    fn name(&self) -> &str {
        "StruckIdiophoneResonator"
    }

    fn process_block(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        _ctx: &ProcessContext,
    ) {
        if outputs.is_empty() {
            return;
        }

        let num_samples = outputs[0].len();
        for i in 0..num_samples {
            let excitation = if !inputs.is_empty() && !inputs[0].is_empty() && i < inputs[0].len() {
                inputs[0][i]
            } else {
                0.0
            };

            let out_sample = self.process_sample(excitation);
            for ch in outputs.iter_mut() {
                if i < ch.len() {
                    ch[i] = out_sample;
                }
            }
        }
    }
}

/// AudioNode wrapper for Struck Idiophone Resonator synthesis node.
#[derive(Debug)]
pub struct StruckIdiophoneResonatorNode {
    pub resonator: StruckIdiophoneResonator,
}

impl StruckIdiophoneResonatorNode {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            resonator: StruckIdiophoneResonator::new(sample_rate),
        }
    }
}

impl AudioNode for StruckIdiophoneResonatorNode {
    fn name(&self) -> &str {
        "StruckIdiophoneResonatorNode"
    }

    fn process(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        self.resonator.process_block(inputs, outputs, ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::allocator::AllocGuard;

    #[test]
    fn test_struck_idiophone_marimba_decay_and_overtones() {
        let mut synth = StruckIdiophoneResonator::new(48000);
        synth.set_instrument(IdiophoneInstrumentProfile::MarimbaWood);
        synth.set_frequency(261.63);
        synth.trigger_strike(0.85, 0.40, 0.20);

        let mut peak_amp = 0.0f32;
        for _ in 0..2000 {
            let s = synth.process_sample(0.0);
            assert!(s.is_finite());
            assert!((-1.0..=1.0).contains(&s));
            peak_amp = peak_amp.max(s.abs());
        }

        for _ in 0..48000 {
            let s = synth.process_sample(0.0);
            assert!(s.is_finite());
        }

        let mut late_amp = 0.0f32;
        for _ in 0..2000 {
            let s = synth.process_sample(0.0);
            assert!(s.is_finite());
            late_amp = late_amp.max(s.abs());
        }

        assert!(peak_amp > 0.02, "Idiophone strike must produce audible output");
        assert!(late_amp < peak_amp, "Idiophone resonance must naturally decay");
    }

    #[test]
    fn test_struck_idiophone_zero_allocation_in_loop() {
        let mut synth = StruckIdiophoneResonator::new(48000);
        synth.trigger_strike(0.80, 0.50, 0.25);

        let mut out_l = [0.0f32; 256];
        let mut out_r = [0.0f32; 256];
        let in_buf = [0.0f32; 256];
        let ctx = ProcessContext::new(48000, 120.0, 0);

        let _guard = AllocGuard::new();
        for _ in 0..10 {
            synth.process_block(&[&in_buf[..]], &mut [&mut out_l[..], &mut out_r[..]], &ctx);
        }
    }
}
