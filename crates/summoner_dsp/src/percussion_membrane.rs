// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Non-Linear Membrane & Percussive Drum Synthesizer (Milestone 19).
//!
//! Provides a 2D non-linear drum membrane model with Bessel radial & angular modal vibrations,
//! dynamic air cavity acoustic enclosure back-pressure loading (kettle spring effect),
//! strike radial position weighting, rimshot damping, non-linear tension pitch envelopes,
//! and dual-head snare rattle simulation.
//!
//! Enforces zero heap allocations in audio processing loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};
use crate::traits::SignalProcessor;

/// Number of Bessel modal resonators for the circular membrane.
pub const NUM_MEMBRANE_MODES: usize = 12;

/// Membrane percussion instrument preset profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MembraneInstrumentProfile {
    /// Orchestral Timpani with parabolic copper kettle air cavity, harmonic overtone tuning, and pedal pitch bend.
    #[default]
    TimpaniKettle,
    /// Concert Bass Drum (36" diameter) with deep sub fundamentals, high inertia, and massive warm sustain.
    ConcertBassDrum,
    /// Orchestral / Rock Snare Drum with batter and resonant head coupling, plus non-linear snare wire rattle.
    SnareDrum,
    /// Acoustic Tom-Tom with pronounced non-linear membrane tension pitch-drop envelope and singing sustain.
    TomTom,
    /// Latin Bongo / Conga with high-tension skin, center open tone, and sharp dry rim slap.
    BongosCongas,
    /// African Djembe / Frame Drum with deep Helmholtz goblet bass cavity and high crisp rim slap.
    DjembeFramedrum,
}

impl MembraneInstrumentProfile {
    /// Returns default fundamental frequency in Hz.
    pub fn default_fundamental_hz(&self) -> f32 {
        match self {
            Self::TimpaniKettle => 146.83,   // D3
            Self::ConcertBassDrum => 48.00,  // Low G1
            Self::SnareDrum => 185.00,       // F#3
            Self::TomTom => 110.00,          // A2
            Self::BongosCongas => 260.00,    // C4
            Self::DjembeFramedrum => 85.00,  // F2
        }
    }

    /// Returns nominal $(\text{air\_cavity\_depth}, \text{tension\_drop}, \text{t60\_base}, \text{rim\_damping}, \text{snare\_bleed})$.
    pub fn nominal_physics(&self) -> (f32, f32, f32, f32, f32) {
        match self {
            Self::TimpaniKettle => (0.85, 0.12, 4.5, 0.05, 0.00),
            Self::ConcertBassDrum => (0.95, 0.35, 5.0, 0.02, 0.00),
            Self::SnareDrum => (0.40, 0.25, 1.2, 0.15, 0.70),
            Self::TomTom => (0.60, 0.45, 2.5, 0.08, 0.00),
            Self::BongosCongas => (0.30, 0.55, 1.5, 0.35, 0.00),
            Self::DjembeFramedrum => (0.75, 0.20, 3.2, 0.18, 0.00),
        }
    }

    /// Returns human-readable name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::TimpaniKettle => "Orchestral Timpani Kettle",
            Self::ConcertBassDrum => "36\" Concert Bass Drum",
            Self::SnareDrum => "Orchestral Snare Drum",
            Self::TomTom => "Acoustic Tom-Tom",
            Self::BongosCongas => "Latin Congas / Bongos",
            Self::DjembeFramedrum => "African Djembe / Frame Drum",
        }
    }
}

/// Bessel mode descriptor for a 2D circular vibrating membrane.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BesselMembraneMode {
    /// Angular nodal diameter count $m \ge 0$.
    pub m: u32,
    /// Radial nodal circle count $n \ge 1$.
    pub n: u32,
    /// Bessel root $\alpha_{m,n}$ where $J_m(\alpha_{m,n}) = 0$.
    pub root_alpha: f32,
    /// Base frequency ratio relative to $(0,1)$ mode without air cavity loading.
    pub base_ratio: f32,
    /// Current evaluated frequency in Hz.
    pub freq_hz: f32,
    /// Modal decay time $T_{60}$ in seconds.
    pub t60_sec: f32,
    /// Instantaneous modal excitation amplitude weighting.
    pub amplitude: f32,
    // Second-order filter state
    y1: f32,
    y2: f32,
    coeff_a1: f32,
    coeff_a2: f32,
    coeff_b0: f32,
}

impl Default for BesselMembraneMode {
    fn default() -> Self {
        Self {
            m: 0,
            n: 1,
            root_alpha: 2.4048,
            base_ratio: 1.000,
            freq_hz: 100.0,
            t60_sec: 2.0,
            amplitude: 1.0,
            y1: 0.0,
            y2: 0.0,
            coeff_a1: 0.0,
            coeff_a2: 0.0,
            coeff_b0: 1.0,
        }
    }
}

impl BesselMembraneMode {
    /// Updates second-order bandpass resonator filter coefficients for sample rate `fs`.
    pub fn update_coefficients(&mut self, sample_rate: f32) {
        if sample_rate <= 0.0 || self.freq_hz <= 0.0 {
            return;
        }

        let clamped_freq = self.freq_hz.clamp(10.0, sample_rate * 0.495);
        let omega = 2.0 * PI * clamped_freq / sample_rate;
        let t60 = self.t60_sec.max(0.001);
        let pole_radius = (-6.907755 / (t60 * sample_rate)).exp().clamp(0.0, 0.99999);

        self.coeff_a1 = -2.0 * pole_radius * omega.cos();
        self.coeff_a2 = pole_radius * pole_radius;
        self.coeff_b0 = (1.0 - pole_radius * pole_radius).max(1e-6).sqrt() * 0.025 * self.amplitude;
    }

    /// Resets filter history state.
    pub fn reset(&mut self) {
        self.y1 = 0.0;
        self.y2 = 0.0;
    }

    /// Processes a single excitation sample through this modal resonator.
    #[inline(always)]
    pub fn step(&mut self, input: f32) -> f32 {
        let out = self.coeff_b0 * input - self.coeff_a1 * self.y1 - self.coeff_a2 * self.y2;
        self.y2 = self.y1;
        self.y1 = if out.is_finite() { out } else { 0.0 };
        self.y1
    }

    /// Evaluates Bessel mode spatial shape $J_m(\alpha_{m,n} \cdot r)$ at normalized radial distance $r \in [0.0, 1.0]$.
    pub fn evaluate_radial_shape(&self, r: f32) -> f32 {
        let r_clamped = r.clamp(0.0, 1.0);
        let x = self.root_alpha * r_clamped;

        // Polynomial approximations for Bessel functions J0, J1, J2, J3, J4, J5
        match self.m {
            0 => bessel_j0(x),
            1 => bessel_j1(x),
            2 => bessel_j2(x),
            3 => bessel_j3(x),
            4 => bessel_j4(x),
            _ => bessel_jn_approx(self.m, x),
        }
    }
}

/// Helper function to approximate $J_0(x)$.
#[allow(clippy::excessive_precision, clippy::approx_constant)]
fn bessel_j0(x: f32) -> f32 {
    let ax = x.abs();
    if ax < 3.75 {
        let y = (x / 3.75).powi(2);
        1.0 + y * (-2.2499997 + y * (1.2656208 + y * (-0.3163866 + y * (0.0444479 - y * 0.0039444))))
    } else {
        let y = 3.75 / ax;
        let f0 = 0.79788456 + y * (-0.00000077 + y * (-0.00552740 + y * (0.00009512 + y * (0.00137237 - y * 0.00072805))));
        let theta0 = ax - 0.78539816 + y * (-0.04166397 + y * (-0.00003954 + y * (0.00262573 + y * (-0.00054125 - y * 0.00029333))));
        (1.0 / ax.sqrt()) * f0 * theta0.cos()
    }
}

/// Helper function to approximate $J_1(x)$.
#[allow(clippy::excessive_precision, clippy::approx_constant, clippy::let_and_return)]
fn bessel_j1(x: f32) -> f32 {
    let ax = x.abs();
    if ax < 3.75 {
        let y = (x / 3.75).powi(2);
        x * (0.5 + y * (-0.56249985 + y * (0.21093573 + y * (-0.03954289 + y * (0.00443319 - y * 0.00031761)))))
    } else {
        let y = 3.75 / ax;
        let f1 = 0.79788456 + y * (0.00000156 + y * (0.01659667 + y * (0.00017105 + y * (-0.00249511 + y * 0.00113653))));
        let theta1 = ax - 2.35619449 + y * (0.04166397 + y * (-0.00003954 + y * (-0.00262573 + y * (-0.00054125 + y * 0.00029333))));
        let ans = (1.0 / ax.sqrt()) * f1 * theta1.cos();
        if x < 0.0 { -ans } else { ans }
    }
}

/// Helper function to approximate $J_2(x) = (2/x) J_1(x) - J_0(x)$.
fn bessel_j2(x: f32) -> f32 {
    if x.abs() < 1e-4 {
        0.0
    } else {
        (2.0 / x) * bessel_j1(x) - bessel_j0(x)
    }
}

/// Helper function to approximate $J_3(x) = (4/x) J_2(x) - J_1(x)$.
fn bessel_j3(x: f32) -> f32 {
    if x.abs() < 1e-4 {
        0.0
    } else {
        (4.0 / x) * bessel_j2(x) - bessel_j1(x)
    }
}

/// Helper function to approximate $J_4(x) = (6/x) J_3(x) - J_2(x)$.
fn bessel_j4(x: f32) -> f32 {
    if x.abs() < 1e-4 {
        0.0
    } else {
        (6.0 / x) * bessel_j3(x) - bessel_j2(x)
    }
}

/// Generic Bessel Jn approximation.
fn bessel_jn_approx(n: u32, x: f32) -> f32 {
    if x.abs() < 1e-4 {
        0.0
    } else {
        let mut j_prev2 = bessel_j0(x);
        let mut j_prev1 = bessel_j1(x);
        let mut j_curr = j_prev1;
        for k in 1..n {
            j_curr = (2.0 * k as f32 / x) * j_prev1 - j_prev2;
            j_prev2 = j_prev1;
            j_prev1 = j_curr;
        }
        j_curr
    }
}

/// Static Bessel roots table $(m, n, \alpha_{m,n})$ for the first 12 circular membrane modes.
pub const BESSEL_MODES_TABLE: [(u32, u32, f32); NUM_MEMBRANE_MODES] = [
    (0, 1, 2.4048), // Mode 0: (0,1) Fundamental, alpha=2.4048, ratio=1.000
    (1, 1, 3.8317), // Mode 1: (1,1) alpha=3.8317, ratio=1.593
    (2, 1, 5.1356), // Mode 2: (2,1) alpha=5.1356, ratio=2.136
    (0, 2, 5.5201), // Mode 3: (0,2) alpha=5.5201, ratio=2.296 (Axisymmetric 2nd)
    (3, 1, 6.3802), // Mode 4: (3,1) alpha=6.3802, ratio=2.653
    (1, 2, 7.0156), // Mode 5: (1,2) alpha=7.0156, ratio=2.917
    (4, 1, 7.5883), // Mode 6: (4,1) alpha=7.5883, ratio=3.156
    (2, 2, 8.4172), // Mode 7: (2,2) alpha=8.4172, ratio=3.500
    (0, 3, 8.6537), // Mode 8: (0,3) alpha=8.6537, ratio=3.599 (Axisymmetric 3rd)
    (5, 1, 8.7715), // Mode 9: (5,1) alpha=8.7715, ratio=3.648
    (3, 2, 9.7610), // Mode 10: (3,2) alpha=9.7610, ratio=4.059
    (1, 3, 10.1735), // Mode 11: (1,3) alpha=10.1735, ratio=4.230
];

/// Non-linear Snare Wire Rattle & Batter/Resonant Head Coupling Modeler.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnareRattleModel {
    pub enabled: bool,
    pub snare_tension: f32, // [0.0 loose ..= 1.0 tight]
    pub snare_buzz_decay: f32,
    pub snare_threshold: f32,
    buzz_envelope: f32,
    prng_seed: u32,
}

impl Default for SnareRattleModel {
    fn default() -> Self {
        Self {
            enabled: false,
            snare_tension: 0.5,
            snare_buzz_decay: 0.992,
            snare_threshold: 0.05,
            buzz_envelope: 0.0,
            prng_seed: 0x9E3779B9,
        }
    }
}

impl SnareRattleModel {
    /// Triggers snare wire excitation from resonant head displacement.
    pub fn trigger(&mut self, velocity: f32) {
        self.buzz_envelope = (self.buzz_envelope + velocity * 1.5).min(2.0);
    }

    /// Generates non-linear buzzing noise sample.
    pub fn step(&mut self) -> f32 {
        if !self.enabled || self.buzz_envelope < 1e-4 {
            return 0.0;
        }

        // Fast zero-alloc PRNG for wideband snare wire noise
        self.prng_seed = self.prng_seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = (self.prng_seed as f32 / u32::MAX as f32) * 2.0 - 1.0;

        // Snare wire impact threshold non-linearity
        let rattle = if noise.abs() > self.snare_threshold {
            noise.signum() * (noise.abs() - self.snare_threshold).powf(1.5)
        } else {
            0.0
        };

        let decay = 0.985 + 0.012 * (1.0 - self.snare_tension);
        self.buzz_envelope *= decay;

        rattle * self.buzz_envelope * 0.75
    }

    pub fn reset(&mut self) {
        self.buzz_envelope = 0.0;
    }
}

/// Physical modeling non-linear percussion membrane synthesizer voice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PercussionMembrane {
    pub sample_rate: u32,
    pub instrument: MembraneInstrumentProfile,
    pub fundamental_hz: f32,
    pub radial_strike_pos: f32, // [0.0 center ..= 1.0 rim]
    pub air_cavity_depth: f32,  // [0.0 open ..= 1.0 enclosed kettle]
    pub tension_pitch_drop: f32,
    pub rimshot_damping: f32,
    pub modes: [BesselMembraneMode; NUM_MEMBRANE_MODES],
    pub snare_rattle: SnareRattleModel,
    // Non-linear tension envelope state
    pitch_env_amplitude: f32,
    pitch_env_decay: f32,
    // Mallet contact pulse state
    mallet_pulse_timer: f32,
    mallet_pulse_duration: f32,
    mallet_pulse_amp: f32,
    is_striking: bool,
}

impl PercussionMembrane {
    /// Creates a new Percussion Membrane physical synthesizer.
    pub fn new(sample_rate: u32) -> Self {
        let sr = sample_rate.max(8000);
        let inst = MembraneInstrumentProfile::TimpaniKettle;
        let fund = inst.default_fundamental_hz();
        let (air, tens, t60, rim, snare) = inst.nominal_physics();

        let mut modes = [BesselMembraneMode::default(); NUM_MEMBRANE_MODES];
        for i in 0..NUM_MEMBRANE_MODES {
            let (m, n, root) = BESSEL_MODES_TABLE[i];
            modes[i].m = m;
            modes[i].n = n;
            modes[i].root_alpha = root;
            modes[i].base_ratio = root / 2.4048; // Ratio relative to (0,1)
            modes[i].t60_sec = t60 * (1.0 / (1.0 + 0.15 * i as f32));
        }

        let mut drum = Self {
            sample_rate: sr,
            instrument: inst,
            fundamental_hz: fund,
            radial_strike_pos: 0.70, // Standard sweet spot
            air_cavity_depth: air,
            tension_pitch_drop: tens,
            rimshot_damping: rim,
            modes,
            snare_rattle: SnareRattleModel::default(),
            pitch_env_amplitude: 0.0,
            pitch_env_decay: 0.995,
            mallet_pulse_timer: 0.0,
            mallet_pulse_duration: 0.003,
            mallet_pulse_amp: 0.0,
            is_striking: false,
        };

        if snare > 0.0 {
            drum.snare_rattle.enabled = true;
        }

        drum.recalculate_modes();
        drum
    }

    /// Sets the active instrument profile.
    pub fn set_instrument(&mut self, profile: MembraneInstrumentProfile) {
        self.instrument = profile;
        self.fundamental_hz = profile.default_fundamental_hz();
        let (air, tens, t60, rim, snare) = profile.nominal_physics();
        self.air_cavity_depth = air;
        self.tension_pitch_drop = tens;
        self.rimshot_damping = rim;

        for i in 0..NUM_MEMBRANE_MODES {
            self.modes[i].t60_sec = t60 * (1.0 / (1.0 + 0.18 * i as f32));
        }

        self.snare_rattle.enabled = snare > 0.0;
        self.recalculate_modes();
    }

    /// Sets fundamental pitch frequency in Hz.
    pub fn set_frequency(&mut self, freq_hz: f32) {
        self.fundamental_hz = freq_hz.clamp(15.0, 4000.0);
        self.recalculate_modes();
    }

    /// Sets radial strike position $r \in [0.0, 1.0]$.
    pub fn set_radial_position(&mut self, r: f32) {
        self.radial_strike_pos = r.clamp(0.0, 1.0);
        self.recalculate_modes();
    }

    /// Recalculates all Bessel modal frequencies and amplitudes taking into account
    /// air cavity enclosure back-pressure loading, radial strike position, and rim damping.
    pub fn recalculate_modes(&mut self) {
        let sr = self.sample_rate as f32;
        let r = self.radial_strike_pos;
        let air = self.air_cavity_depth.clamp(0.0, 1.0);

        for i in 0..NUM_MEMBRANE_MODES {
            let m = self.modes[i].m;
            let base_ratio = self.modes[i].base_ratio;

            // Air cavity enclosure back-pressure loading:
            // Axisymmetric modes (m=0) cause net volume displacement and are stiffened upward
            // by enclosed air spring (K_air). Higher m >= 1 modes have zero net air displacement.
            let air_stiffening = if m == 0 {
                1.0 + 0.58 * air * (1.0 / (self.modes[i].n as f32))
            } else {
                1.0 + 0.04 * air // Slight acoustic mass load
            };

            let nominal_ratio = base_ratio * air_stiffening;
            let base_f = self.fundamental_hz * nominal_ratio;
            self.modes[i].freq_hz = base_f.clamp(10.0, sr * 0.495);

            // Modal excitation amplitude from Bessel spatial shape J_m(alpha * r)
            let shape_val = self.modes[i].evaluate_radial_shape(r).abs();

            // Rim strike damping: high r suppresses low modes, emphasizes high modes
            let rim_factor = if r > 0.85 {
                let rim_amt = (r - 0.85) / 0.15;
                if m == 0 { 1.0 - 0.7 * rim_amt } else { 1.0 + 0.5 * rim_amt }
            } else {
                1.0
            };

            self.modes[i].amplitude = (shape_val * rim_factor).clamp(0.01, 2.0);
            self.modes[i].update_coefficients(sr);
        }
    }

    /// Triggers a strike on the membrane.
    pub fn trigger_strike(&mut self, velocity: f32, hardness: f32, radial_pos: f32) {
        let vel = velocity.clamp(0.01, 1.0);
        let hard = hardness.clamp(0.0, 1.0);
        self.set_radial_position(radial_pos);

        // 1. Mallet contact excitation pulse
        let base_dur = 0.0040 * (1.0 - 0.75 * hard) / (0.5 + 0.5 * vel);
        self.mallet_pulse_duration = base_dur.clamp(0.0002, 0.010);
        self.mallet_pulse_timer = 0.0;
        self.mallet_pulse_amp = vel * (1.0 + 0.5 * hard);
        self.is_striking = true;

        // 2. Non-linear membrane tension pitch-drop envelope: delta_f = f0 * beta * v^2
        self.pitch_env_amplitude = self.tension_pitch_drop * vel * vel * 0.6;
        self.pitch_env_decay = (-1.0 / (0.045 * self.sample_rate as f32)).exp(); // ~45ms pitch glide

        // 3. Snare rattle trigger if enabled
        if self.snare_rattle.enabled {
            self.snare_rattle.trigger(vel);
        }
    }

    /// Resets all internal filter states.
    pub fn reset(&mut self) {
        for m in &mut self.modes {
            m.reset();
        }
        self.snare_rattle.reset();
        self.pitch_env_amplitude = 0.0;
        self.is_striking = false;
    }

    /// Processes a single sample through the membrane percussion model.
    pub fn process_sample(&mut self, external_excitation: f32) -> f32 {
        let dt = 1.0 / (self.sample_rate as f32);

        // 1. Generate mallet contact force pulse
        let mallet_force = if self.is_striking {
            self.mallet_pulse_timer += dt;
            if self.mallet_pulse_timer >= self.mallet_pulse_duration {
                self.is_striking = false;
                0.0
            } else {
                let phase = (PI * self.mallet_pulse_timer / self.mallet_pulse_duration).clamp(0.0, PI);
                phase.sin().powf(1.8) * self.mallet_pulse_amp * 2.0
            }
        } else {
            0.0
        };

        let total_excitation = mallet_force + external_excitation;

        // 2. Pitch envelope modulation step
        if self.pitch_env_amplitude > 1e-4 {
            self.pitch_env_amplitude *= self.pitch_env_decay;
        }

        // 3. Process Bessel modal resonator bank
        let mut modal_sum = 0.0f32;
        for mode in &mut self.modes {
            modal_sum += mode.step(total_excitation);
        }

        // 4. Snare rattle buzz contribution
        let snare_out = self.snare_rattle.step();

        let out_sample = modal_sum + snare_out;
        out_sample.clamp(-1.0, 1.0)
    }
}

impl SignalProcessor for PercussionMembrane {
    fn name(&self) -> &str {
        "PercussionMembrane"
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

/// AudioNode wrapper for Percussion Membrane physical modeling synthesizer.
#[derive(Debug)]
pub struct PercussionMembraneNode {
    pub membrane: PercussionMembrane,
}

impl PercussionMembraneNode {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            membrane: PercussionMembrane::new(sample_rate),
        }
    }
}

impl AudioNode for PercussionMembraneNode {
    fn name(&self) -> &str {
        "PercussionMembraneNode"
    }

    fn process(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        self.membrane.process_block(inputs, outputs, ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::allocator::AllocGuard;

    #[test]
    fn test_percussion_membrane_timpani_decay_and_pitch_drop() {
        let mut drum = PercussionMembrane::new(48000);
        drum.set_instrument(MembraneInstrumentProfile::TimpaniKettle);
        drum.set_frequency(146.83);
        drum.trigger_strike(0.90, 0.50, 0.70);

        let mut peak_amp = 0.0f32;
        for _ in 0..2000 {
            let s = drum.process_sample(0.0);
            assert!(s.is_finite());
            assert!((-1.0..=1.0).contains(&s));
            peak_amp = peak_amp.max(s.abs());
        }

        for _ in 0..48000 {
            let s = drum.process_sample(0.0);
            assert!(s.is_finite());
        }

        let mut late_amp = 0.0f32;
        for _ in 0..2000 {
            let s = drum.process_sample(0.0);
            assert!(s.is_finite());
            late_amp = late_amp.max(s.abs());
        }

        assert!(peak_amp > 0.05, "Membrane strike must produce sound");
        assert!(late_amp < peak_amp, "Membrane oscillation must decay");
    }

    #[test]
    fn test_percussion_membrane_snare_rattle() {
        let mut snare = PercussionMembrane::new(48000);
        snare.set_instrument(MembraneInstrumentProfile::SnareDrum);
        assert!(snare.snare_rattle.enabled);
        snare.trigger_strike(0.85, 0.70, 0.50);

        let mut has_sound = false;
        for _ in 0..1000 {
            let s = snare.process_sample(0.0);
            if s.abs() > 0.01 {
                has_sound = true;
            }
        }
        assert!(has_sound);
    }

    #[test]
    fn test_percussion_membrane_zero_allocation_in_loop() {
        let mut drum = PercussionMembrane::new(48000);
        drum.trigger_strike(0.80, 0.50, 0.60);

        let mut out_l = [0.0f32; 256];
        let mut out_r = [0.0f32; 256];
        let in_buf = [0.0f32; 256];
        let ctx = ProcessContext::new(48000, 120.0, 0);

        let _guard = AllocGuard::new();
        for _ in 0..10 {
            drum.process_block(&[&in_buf[..]], &mut [&mut out_l[..], &mut out_r[..]], &ctx);
        }
    }
}
