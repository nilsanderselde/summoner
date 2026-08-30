// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Bowed String Non-Linear Digital Waveguide Synthesizer (Milestone 16).
//!
//! Provides a physical acoustic bowed string synthesizer based on a dual-delay digital waveguide
//! string model with non-linear stick-slip friction interaction (hyperbolic friction curve,
//! normal bow force, bow velocity, rosin adhesion, string damping filter) coupled to a
//! 5-mode acoustic instrument body resonator.
//!
//! Enforces zero heap allocations in audio processing loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};
use crate::friction_model::{FrictionModel, RosinType};
use crate::traits::SignalProcessor;

/// Maximum delay buffer capacity in samples (~20 Hz fundamental at 192 kHz).
pub const MAX_BOWED_DELAY: usize = 8192;

/// Acoustic bowed string instrument preset profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BowedInstrument {
    /// Concert Violin with bright overtone projection and rapid Helmholtz attack.
    #[default]
    Violin,
    /// Concert Viola with warm, rich alto acoustic body resonances.
    Viola,
    /// Concert Cello with deep harmonic resonance and expressive dynamic range.
    Cello,
    /// Orchestral Double Bass with massive low-end acoustic mass and high string tension.
    DoubleBass,
}

impl BowedInstrument {
    /// Returns default fundamental frequency in Hz (A string / open fundamental).
    pub fn default_fundamental_hz(&self) -> f32 {
        match self {
            Self::Violin => 440.0,     // A4
            Self::Viola => 220.0,      // A3
            Self::Cello => 110.0,      // A2
            Self::DoubleBass => 55.0,  // A1
        }
    }

    /// Returns default rosin type best suited for this instrument.
    pub fn default_rosin(&self) -> RosinType {
        match self {
            Self::Violin => RosinType::LightViolin,
            Self::Viola => RosinType::MediumCello,
            Self::Cello => RosinType::MediumCello,
            Self::DoubleBass => RosinType::DarkDoubleBass,
        }
    }

    /// Returns 5 body resonance modal frequencies $(A_0, B_1^-, B_1^+, C_3, \text{BridgeHill})$ in Hz.
    pub fn body_modal_frequencies(&self) -> (f32, f32, f32, f32, f32) {
        match self {
            Self::Violin => (280.0, 460.0, 550.0, 1100.0, 2500.0),
            Self::Viola => (230.0, 360.0, 440.0, 900.0, 2000.0),
            Self::Cello => (100.0, 170.0, 220.0, 450.0, 1200.0),
            Self::DoubleBass => (60.0, 100.0, 140.0, 280.0, 800.0),
        }
    }
}

/// A second-order resonant bandpass biquad section for acoustic body modeling.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BodyResonatorMode {
    pub freq_hz: f32,
    pub q: f32,
    pub gain: f32,
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl Default for BodyResonatorMode {
    fn default() -> Self {
        Self::new(440.0, 10.0, 1.0, 48000)
    }
}

impl BodyResonatorMode {
    pub fn new(freq_hz: f32, q: f32, gain: f32, sample_rate: u32) -> Self {
        let mut mode = Self {
            freq_hz,
            q: q.max(0.5),
            gain,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        };
        mode.recalculate(sample_rate);
        mode
    }

    pub fn recalculate(&mut self, sample_rate: u32) {
        let sr = (sample_rate.max(8000)) as f32;
        let w0 = 2.0 * PI * (self.freq_hz.clamp(20.0, sr * 0.45)) / sr;
        let alpha = (w0.sin()) / (2.0 * self.q.max(0.1));
        let cos_w0 = w0.cos();

        let a0 = 1.0 + alpha;
        self.b0 = (alpha * self.gain) / a0;
        self.b1 = 0.0;
        self.b2 = (-alpha * self.gain) / a0;
        self.a1 = (-2.0 * cos_w0) / a0;
        self.a2 = (1.0 - alpha) / a0;
    }

    #[inline]
    pub fn step(&mut self, input: f32) -> f32 {
        let out = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2 - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = if out.is_finite() { out } else { 0.0 };
        self.y1
    }

    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
}

fn default_bowed_buffer() -> [f32; MAX_BOWED_DELAY] {
    [0.0; MAX_BOWED_DELAY]
}

/// Physical Modeling Bowed String Synthesizer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BowedString {
    /// Instrument acoustic profile.
    pub instrument: BowedInstrument,
    /// Target fundamental frequency in Hz.
    pub frequency_hz: f32,
    /// Transverse bow velocity (m/s) [0.0 ..= 2.0].
    pub bow_velocity: f32,
    /// Clamping normal bow force ($F_N$ in Newtons) [0.0 ..= 5.0].
    pub bow_force: f32,
    /// Bridge proximity ratio $\beta = x_{bow} / L$ [0.02 ..= 0.50].
    pub bow_position: f32,
    /// String damping loss coefficient [0.0 ..= 1.0].
    pub string_damping: f32,
    /// Acoustic instrument body resonator mix [0.0 ..= 1.0].
    pub body_mix: f32,
    /// Vibrato depth in semitones [0.0 ..= 2.0].
    pub vibrato_depth: f32,
    /// Vibrato rate in Hz [0.5 ..= 15.0].
    pub vibrato_rate: f32,
    /// Sample rate.
    pub sample_rate: u32,
    /// Non-linear stick-slip friction solver.
    pub friction_model: FrictionModel,
    /// 5-mode acoustic body resonator bank.
    pub body_modes: [BodyResonatorMode; 5],

    #[serde(skip, default = "default_bowed_buffer")]
    nut_delay: [f32; MAX_BOWED_DELAY],
    #[serde(skip, default = "default_bowed_buffer")]
    bridge_delay: [f32; MAX_BOWED_DELAY],
    #[serde(skip)]
    nut_write_idx: usize,
    #[serde(skip)]
    bridge_write_idx: usize,
    #[serde(skip)]
    bridge_damping_state: f32,
    #[serde(skip)]
    nut_damping_state: f32,
    #[serde(skip)]
    vibrato_phase: f32,
}

impl Default for BowedString {
    fn default() -> Self {
        Self::new(48000)
    }
}

impl BowedString {
    /// Creates a new Bowed String synthesizer initialized to Violin profile at 440 Hz.
    pub fn new(sample_rate: u32) -> Self {
        let sr = sample_rate.max(8000);
        let instrument = BowedInstrument::Violin;
        let (f_a0, f_b1m, f_b1p, f_c3, f_br) = instrument.body_modal_frequencies();

        let body_modes = [
            BodyResonatorMode::new(f_a0, 10.0, 1.2, sr),
            BodyResonatorMode::new(f_b1m, 14.0, 1.5, sr),
            BodyResonatorMode::new(f_b1p, 18.0, 1.8, sr),
            BodyResonatorMode::new(f_c3, 8.0, 0.9, sr),
            BodyResonatorMode::new(f_br, 4.0, 0.7, sr),
        ];

        let mut bowed = Self {
            instrument,
            frequency_hz: instrument.default_fundamental_hz(),
            bow_velocity: 0.35,
            bow_force: 1.20,
            bow_position: 0.12, // Normale bowing position
            string_damping: 0.15,
            body_mix: 0.65,
            vibrato_depth: 0.0,
            vibrato_rate: 5.5,
            sample_rate: sr,
            friction_model: FrictionModel::new(sr),
            body_modes,
            nut_delay: [0.0; MAX_BOWED_DELAY],
            bridge_delay: [0.0; MAX_BOWED_DELAY],
            nut_write_idx: 0,
            bridge_write_idx: 0,
            bridge_damping_state: 0.0,
            nut_damping_state: 0.0,
            vibrato_phase: 0.0,
        };
        bowed.friction_model.set_rosin(instrument.default_rosin());
        bowed
    }

    /// Sets the instrument preset profile and reconfigures body modes and rosin.
    pub fn set_instrument(&mut self, instrument: BowedInstrument) {
        self.instrument = instrument;
        self.frequency_hz = instrument.default_fundamental_hz();
        self.friction_model.set_rosin(instrument.default_rosin());

        let (f_a0, f_b1m, f_b1p, f_c3, f_br) = instrument.body_modal_frequencies();
        self.body_modes[0] = BodyResonatorMode::new(f_a0, 10.0, 1.2, self.sample_rate);
        self.body_modes[1] = BodyResonatorMode::new(f_b1m, 14.0, 1.5, self.sample_rate);
        self.body_modes[2] = BodyResonatorMode::new(f_b1p, 18.0, 1.8, self.sample_rate);
        self.body_modes[3] = BodyResonatorMode::new(f_c3, 8.0, 0.9, self.sample_rate);
        self.body_modes[4] = BodyResonatorMode::new(f_br, 4.0, 0.7, self.sample_rate);
    }

    /// Sets target fundamental pitch in Hz.
    pub fn set_frequency(&mut self, frequency_hz: f32) {
        self.frequency_hz = frequency_hz.clamp(20.0, self.sample_rate as f32 * 0.45);
    }

    /// Sets transverse bow velocity in m/s [0.0 ..= 2.0].
    pub fn set_bow_velocity(&mut self, velocity: f32) {
        self.bow_velocity = velocity.clamp(0.0, 2.0);
    }

    /// Sets normal clamping bow force in Newtons [0.0 ..= 5.0].
    pub fn set_bow_force(&mut self, force_n: f32) {
        self.bow_force = force_n.clamp(0.0, 5.0);
    }

    /// Sets bridge proximity ratio $\beta = x_{bow} / L$ [0.02 ..= 0.50].
    pub fn set_bow_position(&mut self, beta: f32) {
        self.bow_position = beta.clamp(0.02, 0.50);
    }

    /// Sets high frequency string damping factor [0.0 ..= 1.0].
    pub fn set_string_damping(&mut self, damping: f32) {
        self.string_damping = damping.clamp(0.0, 1.0);
    }

    /// Sets acoustic body resonator wet mix [0.0 ..= 1.0].
    pub fn set_body_mix(&mut self, mix: f32) {
        self.body_mix = mix.clamp(0.0, 1.0);
    }

    /// Sets vibrato depth in semitones and rate in Hz.
    pub fn set_vibrato(&mut self, depth_semitones: f32, rate_hz: f32) {
        self.vibrato_depth = depth_semitones.clamp(0.0, 2.0);
        self.vibrato_rate = rate_hz.clamp(0.5, 15.0);
    }

    /// Resets all internal delay buffers, friction states, and resonator filters.
    pub fn reset(&mut self) {
        self.nut_delay = [0.0; MAX_BOWED_DELAY];
        self.bridge_delay = [0.0; MAX_BOWED_DELAY];
        self.nut_write_idx = 0;
        self.bridge_write_idx = 0;
        self.bridge_damping_state = 0.0;
        self.nut_damping_state = 0.0;
        self.vibrato_phase = 0.0;
        self.friction_model.reset();
        for mode in &mut self.body_modes {
            mode.reset();
        }
    }

    /// Reads sample from delay buffer with 4-point Hermite cubic fractional interpolation.
    #[inline]
    fn read_fractional_delay(buffer: &[f32; MAX_BOWED_DELAY], write_idx: usize, delay_samples: f32) -> f32 {
        let int_delay = delay_samples.floor() as usize;
        let frac = delay_samples - int_delay as f32;

        let base_idx = if write_idx >= int_delay {
            write_idx - int_delay
        } else {
            MAX_BOWED_DELAY + write_idx - int_delay
        };

        let i0 = if base_idx == 0 { MAX_BOWED_DELAY - 1 } else { base_idx - 1 };
        let i1 = base_idx;
        let i2 = (base_idx + 1) % MAX_BOWED_DELAY;
        let i3 = (base_idx + 2) % MAX_BOWED_DELAY;

        let y0 = buffer[i0];
        let y1 = buffer[i1];
        let y2 = buffer[i2];
        let y3 = buffer[i3];

        let c0 = y1;
        let c1 = 0.5 * (y2 - y0);
        let c2 = y0 - 2.5 * y1 + 2.0 * y2 - 0.5 * y3;
        let c3 = 0.5 * (y3 - y0) + 1.5 * (y1 - y2);

        ((c3 * frac + c2) * frac + c1) * frac + c0
    }

    /// Computes effective total period and dual-delay lengths in fractional samples.
    #[inline]
    fn compute_delays(&mut self) -> (f32, f32) {
        let sr = self.sample_rate as f32;
        let dt = 1.0 / sr;

        // Advance vibrato LFO
        let vib_pitch_mult = if self.vibrato_depth > 0.0 {
            let vib_lfo = (self.vibrato_phase * 2.0 * PI).sin();
            self.vibrato_phase = (self.vibrato_phase + self.vibrato_rate * dt) % 1.0;
            2.0f32.powf((self.vibrato_depth * vib_lfo) / 12.0)
        } else {
            1.0
        };

        let f0_eff = (self.frequency_hz * vib_pitch_mult).clamp(20.0, sr * 0.45);
        let total_period = sr / f0_eff;

        let beta = self.bow_position.clamp(0.02, 0.50);
        let d_bridge = (total_period * beta).clamp(2.0, (MAX_BOWED_DELAY / 2 - 4) as f32);
        let d_nut = (total_period * (1.0 - beta)).clamp(2.0, (MAX_BOWED_DELAY / 2 - 4) as f32);

        (d_nut, d_bridge)
    }

    /// Computes and returns the next output audio sample.
    #[inline]
    pub fn process_sample(&mut self, excitation: f32) -> Sample {
        let (d_nut, d_bridge) = self.compute_delays();

        // 1. Read wave returning from nut section (reflected with phase inversion -1.0)
        let nut_wave_raw = Self::read_fractional_delay(&self.nut_delay, self.nut_write_idx, d_nut);
        let nut_loss = 0.998;
        let nut_wave_reflected = -nut_loss * nut_wave_raw;
        // Mild nut damping lowpass
        self.nut_damping_state = 0.85 * nut_wave_reflected + 0.15 * self.nut_damping_state;
        let v_in_nut = self.nut_damping_state;

        // 2. Read wave returning from bridge section (reflected with phase inversion -1.0 and bridge damping)
        let bridge_wave_raw = Self::read_fractional_delay(&self.bridge_delay, self.bridge_write_idx, d_bridge);
        let bridge_loss = 0.995;
        let bridge_wave_reflected = -bridge_loss * bridge_wave_raw;

        // String bridge termination loss filter (frequency dependent damping)
        let damp_coeff = (0.20 + 0.50 * self.string_damping).clamp(0.05, 0.90);
        self.bridge_damping_state = (1.0 - damp_coeff) * bridge_wave_reflected + damp_coeff * self.bridge_damping_state;
        let v_in_bridge = self.bridge_damping_state;

        // 3. Modulate bow velocity and force with external excitation / articulation
        let eff_bow_vel = (self.bow_velocity + excitation * 0.20).clamp(0.0, 2.0);
        let eff_bow_force = self.bow_force.clamp(0.0, 5.0);

        // 4. Solve non-linear stick-slip friction scattering junction
        let friction_res = self.friction_model.solve(v_in_nut, v_in_bridge, eff_bow_vel, eff_bow_force);

        // 5. Write scattered waves into delay lines
        self.nut_delay[self.nut_write_idx] = friction_res.out_nut;
        self.nut_write_idx = (self.nut_write_idx + 1) % MAX_BOWED_DELAY;

        self.bridge_delay[self.bridge_write_idx] = friction_res.out_bridge;
        self.bridge_write_idx = (self.bridge_write_idx + 1) % MAX_BOWED_DELAY;

        // 6. Bridge force transmitted into acoustic body: F_bridge = 2 * Z_0 * v_bridge
        let bridge_driving_force = 2.0 * self.friction_model.params.string_impedance * friction_res.out_bridge;

        // 7. Process through 5-mode acoustic body resonator bank
        let mut body_output = 0.0f32;
        for mode in &mut self.body_modes {
            body_output += mode.step(bridge_driving_force);
        }

        // 8. Mix raw string velocity with body resonance
        let dry_sample = friction_res.string_velocity * 0.65;
        let mix = self.body_mix.clamp(0.0, 1.0);
        let final_out = (1.0 - mix) * dry_sample + mix * body_output * 0.45;

        final_out.clamp(-1.0, 1.0)
    }
}

impl SignalProcessor for BowedString {
    fn name(&self) -> &str {
        "BowedString"
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

/// AudioNode wrapper for Bowed String physical synthesizer.
#[derive(Debug)]
pub struct BowedStringNode {
    pub bowed_string: BowedString,
}

impl BowedStringNode {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            bowed_string: BowedString::new(sample_rate),
        }
    }
}

impl AudioNode for BowedStringNode {
    fn name(&self) -> &str {
        "BowedStringNode"
    }

    fn process(
        &mut self,
        input: &[&[Sample]],
        output: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        self.bowed_string.process_block(input, output, ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::allocator::AllocGuard;

    #[test]
    fn test_bowed_string_helmholtz_oscillation() {
        let mut synth = BowedString::new(48000);
        synth.set_instrument(BowedInstrument::Violin);
        synth.set_frequency(440.0);
        synth.set_bow_velocity(0.40);
        synth.set_bow_force(1.50);
        synth.set_bow_position(0.12);

        let mut max_amp = 0.0f32;
        for _ in 0..4800 {
            let s = synth.process_sample(0.0);
            assert!(s.is_finite());
            assert!((-1.0..=1.0).contains(&s));
            max_amp = max_amp.max(s.abs());
        }
        assert!(max_amp > 0.05, "Bowed string should produce non-zero Helmholtz oscillation");
    }

    #[test]
    fn test_bowed_string_zero_allocation_in_audio_loop() {
        let mut synth = BowedString::new(48000);
        let mut buf = [0.0f32; 512];
        let mut out_slices = [&mut buf[..]];
        let in_buf = [0.0f32; 512];
        let in_slices = [&in_buf[..]];
        let ctx = ProcessContext::new(48000, 120.0, 0);

        let _guard = AllocGuard::new();
        for _ in 0..10 {
            synth.process_block(&in_slices, &mut out_slices, &ctx);
        }
    }
}
