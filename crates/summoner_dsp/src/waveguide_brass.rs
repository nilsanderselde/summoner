// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Waveguide Brass Lip-Reed Modeler (Milestone 14).
//!
//! Provides a physical acoustic brass synthesizer based on a non-linear Bernoulli lip-reed
//! excitation model coupled with a dual-delay waveguide bore and a frequency-dependent Bessel
//! horn radiation reflection filter.
//!
//! Enforces zero heap allocations in audio processing loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};
use crate::horn_reflection::{HornFlarePreset, HornMuteType, HornReflectionFilter};
use crate::traits::SignalProcessor;

/// Maximum delay buffer capacity in samples (~20 Hz at 192 kHz).
pub const MAX_BRASS_DELAY: usize = 8192;

/// Acoustic brass instrument preset profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BrassInstrumentType {
    /// Standard Bb Trumpet with bright attack and 3 piston valves.
    #[default]
    Trumpet,
    /// French Horn with warm harmonic series and deep hyperbolic flare.
    FrenchHorn,
    /// Tenor Trombone with continuous slide position.
    Trombone,
    /// Bass Tuba with massive bore volume and heavy low-end acoustic mass.
    Tuba,
}

impl BrassInstrumentType {
    /// Returns default fundamental frequency in Hz.
    pub fn default_fundamental_hz(&self) -> f32 {
        match self {
            Self::Trumpet => 233.08,   // Bb3
            Self::FrenchHorn => 174.61, // F3
            Self::Trombone => 116.54,   // Bb2
            Self::Tuba => 58.27,       // Bb1
        }
    }

    /// Returns default horn flare preset matching this instrument.
    pub fn horn_preset(&self) -> HornFlarePreset {
        match self {
            Self::Trumpet => HornFlarePreset::Trumpet,
            Self::FrenchHorn => HornFlarePreset::FrenchHorn,
            Self::Trombone => HornFlarePreset::Trombone,
            Self::Tuba => HornFlarePreset::Tuba,
        }
    }

    /// Returns nominal lip mass coefficient.
    pub fn lip_mass(&self) -> f32 {
        match self {
            Self::Trumpet => 0.0015,
            Self::FrenchHorn => 0.0022,
            Self::Trombone => 0.0035,
            Self::Tuba => 0.0070,
        }
    }
}

fn default_brass_buffer() -> [f32; MAX_BRASS_DELAY] {
    [0.0; MAX_BRASS_DELAY]
}

/// Physical Waveguide Brass Lip-Reed Synthesizer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveguideBrass {
    /// Instrument profile.
    pub instrument: BrassInstrumentType,
    /// Fundamental bore target frequency in Hz.
    pub frequency_hz: f32,
    /// Blowing mouth pressure [0.0 ..= 1.5].
    pub blowing_pressure: f32,
    /// Lip tension parameter [0.0 ..= 2.0] scaling lip mechanical resonance.
    pub lip_tension: f32,
    /// Lip aperture resting offset [0.01 ..= 0.50].
    pub lip_aperture: f32,
    /// Lip mechanical damping factor.
    pub lip_damping: f32,
    /// Piston valve 1 (lowers pitch by 2 semitones).
    pub valve_1: bool,
    /// Piston valve 2 (lowers pitch by 1 semitone).
    pub valve_2: bool,
    /// Piston valve 3 (lowers pitch by 3 semitones).
    pub valve_3: bool,
    /// Continuous trombone slide position [0.0 ..= 1.0].
    pub slide_position: f32,
    /// Bore wall loss damping factor [0.95 ..= 0.999].
    pub bore_loss: f32,
    /// Acoustic horn flare convolver & reflection filter.
    pub horn: HornReflectionFilter,
    /// Audio sample rate.
    pub sample_rate: u32,

    // Internal delay lines for bidirectional waveguide bore
    #[serde(skip, default = "default_brass_buffer")]
    fwd_delay: [f32; MAX_BRASS_DELAY],
    #[serde(skip, default = "default_brass_buffer")]
    rev_delay: [f32; MAX_BRASS_DELAY],
    #[serde(skip)]
    fwd_write_idx: usize,
    #[serde(skip)]
    rev_write_idx: usize,

    // Internal lip oscillator physical state
    #[serde(skip)]
    lip_x_curr: f32,
    #[serde(skip)]
    lip_x_prev: f32,
    #[serde(skip)]
    prev_bore_loss_sample: f32,
}

impl Default for WaveguideBrass {
    fn default() -> Self {
        Self::new(48000)
    }
}

impl WaveguideBrass {
    /// Creates a new Waveguide Brass modeler for the given sample rate.
    pub fn new(sample_rate: u32) -> Self {
        let sr = sample_rate.max(8000);
        let instrument = BrassInstrumentType::Trumpet;
        let mut brass = Self {
            instrument,
            frequency_hz: instrument.default_fundamental_hz(),
            blowing_pressure: 0.85,
            lip_tension: 1.0,
            lip_aperture: 0.15,
            lip_damping: 0.08,
            valve_1: false,
            valve_2: false,
            valve_3: false,
            slide_position: 0.0,
            bore_loss: 0.997,
            horn: HornReflectionFilter::new(sr),
            sample_rate: sr,
            fwd_delay: [0.0; MAX_BRASS_DELAY],
            rev_delay: [0.0; MAX_BRASS_DELAY],
            fwd_write_idx: 0,
            rev_write_idx: 0,
            lip_x_curr: 0.0,
            lip_x_prev: 0.0,
            prev_bore_loss_sample: 0.0,
        };
        brass.horn.set_preset(instrument.horn_preset());
        brass
    }

    /// Sets instrument preset profile and configures physical horn/lip parameters.
    pub fn set_instrument(&mut self, instrument: BrassInstrumentType) {
        self.instrument = instrument;
        self.frequency_hz = instrument.default_fundamental_hz();
        self.horn.set_preset(instrument.horn_preset());
    }

    /// Sets the target fundamental pitch frequency in Hz.
    pub fn set_frequency(&mut self, frequency_hz: f32) {
        self.frequency_hz = frequency_hz.clamp(20.0, self.sample_rate as f32 * 0.45);
    }

    /// Sets blowing mouth pressure [0.0 ..= 1.5].
    pub fn set_blowing_pressure(&mut self, pressure: f32) {
        self.blowing_pressure = pressure.clamp(0.0, 1.5);
    }

    /// Sets lip tension factor [0.0 ..= 2.0].
    pub fn set_lip_tension(&mut self, tension: f32) {
        self.lip_tension = tension.clamp(0.01, 3.0);
    }

    /// Sets 3-valve states for brass piston switching.
    pub fn set_valves(&mut self, v1: bool, v2: bool, v3: bool) {
        self.valve_1 = v1;
        self.valve_2 = v2;
        self.valve_3 = v3;
    }

    /// Sets continuous slide extension [0.0 ..= 1.0].
    pub fn set_slide(&mut self, slide: f32) {
        self.slide_position = slide.clamp(0.0, 1.0);
    }

    /// Sets horn mute coloration type.
    pub fn set_mute(&mut self, mute_type: HornMuteType) {
        self.horn.set_mute(mute_type);
    }

    /// Applies a tonguing articulation attack impulse to trigger lip oscillation.
    pub fn tongue_articulation(&mut self, velocity: f32) {
        let vel = velocity.clamp(0.0, 1.0);
        self.lip_x_curr += vel * 0.25;
        self.lip_x_prev = self.lip_x_curr - vel * 0.15;
    }

    /// Resets all delay lines and lip physical states to zero.
    pub fn reset(&mut self) {
        self.fwd_delay = [0.0; MAX_BRASS_DELAY];
        self.rev_delay = [0.0; MAX_BRASS_DELAY];
        self.fwd_write_idx = 0;
        self.rev_write_idx = 0;
        self.lip_x_curr = 0.0;
        self.lip_x_prev = 0.0;
        self.prev_bore_loss_sample = 0.0;
        self.horn.reset();
    }

    /// Computes effective total bore delay length in fractional samples.
    #[inline]
    pub fn compute_bore_delay_samples(&self) -> f32 {
        let sr = self.sample_rate as f32;
        let mut valve_ratio = 1.0f32;
        if self.valve_1 {
            valve_ratio *= 2.0f32.powf(2.0 / 12.0); // 2 semitones lower
        }
        if self.valve_2 {
            valve_ratio *= 2.0f32.powf(1.0 / 12.0); // 1 semitone lower
        }
        if self.valve_3 {
            valve_ratio *= 2.0f32.powf(3.0 / 12.0); // 3 semitones lower
        }
        let slide_mult = 1.0 + self.slide_position * 0.4142; // Up to 6 semitones slide
        let total_mult = valve_ratio * slide_mult;

        // Round-trip bore time is 2 * L / c = 1 / f0. Delay per one-way line is sr / (2 * f0)
        let effective_f0 = self.frequency_hz.max(10.0);
        let one_way_delay = (sr / (2.0 * effective_f0)) * total_mult;
        one_way_delay.clamp(2.0, (MAX_BRASS_DELAY - 4) as f32)
    }

    /// Reads sample from delay buffer with 4-point Hermite fractional interpolation.
    #[inline]
    fn read_fractional_delay(buffer: &[f32; MAX_BRASS_DELAY], write_idx: usize, delay_samples: f32) -> f32 {
        let int_delay = delay_samples.floor() as usize;
        let frac = delay_samples - int_delay as f32;

        let base_idx = if write_idx >= int_delay {
            write_idx - int_delay
        } else {
            MAX_BRASS_DELAY + write_idx - int_delay
        };

        let i0 = if base_idx == 0 { MAX_BRASS_DELAY - 1 } else { base_idx - 1 };
        let i1 = base_idx;
        let i2 = (base_idx + 1) % MAX_BRASS_DELAY;
        let i3 = (base_idx + 2) % MAX_BRASS_DELAY;

        let y0 = buffer[i0];
        let y1 = buffer[i1];
        let y2 = buffer[i2];
        let y3 = buffer[i3];

        // 4-point Hermite cubic interpolation
        let c0 = y1;
        let c1 = 0.5 * (y2 - y0);
        let c2 = y0 - 2.5 * y1 + 2.0 * y2 - 0.5 * y3;
        let c3 = 0.5 * (y3 - y0) + 1.5 * (y1 - y2);

        ((c3 * frac + c2) * frac + c1) * frac + c0
    }

    /// Computes discrete time step of the Bernoulli non-linear lip-reed valve model.
    #[inline]
    fn step_lip_model(&mut self, mouth_pressure: f32, backward_wave: f32) -> f32 {
        let sr = self.sample_rate as f32;
        let dt = 1.0 / sr;

        // Differential pressure across lips
        let delta_p = mouth_pressure - backward_wave;

        // Lip mechanical resonance tracked with note frequency and lip tension
        let f_lip = self.frequency_hz * self.lip_tension.clamp(0.2, 3.0);
        let w_lip = 2.0 * PI * f_lip.clamp(20.0, sr * 0.40);
        let damping = self.lip_damping.clamp(0.001, 0.5);
        let mass = self.instrument.lip_mass();

        // Harmonic oscillator with driving force and nonlinear collision bounce
        let restoring_force = -w_lip * w_lip * (self.lip_x_curr - self.lip_aperture);
        let damping_force = -2.0 * damping * w_lip * (self.lip_x_curr - self.lip_x_prev) / dt;
        let driving_force = delta_p / mass;

        let mut total_accel = restoring_force + damping_force + driving_force;

        // Lip collision spring (lips cannot penetrate teeth / flesh)
        if self.lip_x_curr < -self.lip_aperture {
            let penetration = -self.lip_aperture - self.lip_x_curr;
            total_accel += penetration * w_lip * w_lip * 15.0; // Hard non-linear bounce
        }

        // Verlet integration for lip displacement
        let x_next = 2.0 * self.lip_x_curr - self.lip_x_prev + total_accel * dt * dt;
        self.lip_x_prev = self.lip_x_curr;
        self.lip_x_curr = x_next.clamp(-0.5, 1.5);

        // Bernoulli volume flow through lip aperture
        let opening = (self.lip_x_curr + self.lip_aperture).max(0.0);
        let flow_velocity = if delta_p >= 0.0 {
            (2.0 * delta_p).sqrt()
        } else {
            -(2.0 * (-delta_p)).sqrt()
        };

        // Injected forward pressure pulse
        let u_flow = opening * flow_velocity * 0.45;
        (backward_wave + u_flow).clamp(-1.5, 1.5)
    }

    /// Computes and returns the next output audio sample.
    #[inline]
    pub fn process_sample(&mut self, excitation: f32) -> Sample {
        let delay_len = self.compute_bore_delay_samples();

        // 1. Read backward wave emerging from bore back to mouthpiece
        let bore_back_wave = Self::read_fractional_delay(&self.rev_delay, self.rev_write_idx, delay_len);

        // 2. Modulate mouth pressure with external excitation / tonguing
        let eff_mouth_pressure = (self.blowing_pressure + excitation).clamp(0.0, 2.0);

        // 3. Step non-linear Bernoulli lip model to inject forward wave
        let fwd_wave_in = self.step_lip_model(eff_mouth_pressure, bore_back_wave);

        // Write forward wave into forward delay line
        self.fwd_delay[self.fwd_write_idx] = fwd_wave_in;
        self.fwd_write_idx = (self.fwd_write_idx + 1) % MAX_BRASS_DELAY;

        // 4. Read forward wave emerging at the flared horn bell
        let fwd_bell_wave = Self::read_fractional_delay(&self.fwd_delay, self.fwd_write_idx, delay_len);

        // Apply bore wall damping loss (one-pole lowpass)
        let loss = self.bore_loss.clamp(0.90, 0.999);
        let fwd_bell_damped = loss * fwd_bell_wave + (1.0 - loss) * self.prev_bore_loss_sample;
        self.prev_bore_loss_sample = fwd_bell_damped;

        // 5. Process reflection and radiation at horn flare
        let (reflected_wave, radiated_out) = self.horn.process_reflection(fwd_bell_damped);

        // Write reflected wave into reverse delay line
        self.rev_delay[self.rev_write_idx] = reflected_wave;
        self.rev_write_idx = (self.rev_write_idx + 1) % MAX_BRASS_DELAY;

        // Output sample soft-clamped to [-1.0, 1.0]
        radiated_out.clamp(-1.0, 1.0)
    }
}

impl SignalProcessor for WaveguideBrass {
    fn name(&self) -> &str {
        "WaveguideBrass"
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

/// AudioNode wrapper for Waveguide Brass physical synthesizer.
#[derive(Debug)]
pub struct WaveguideBrassNode {
    pub brass: WaveguideBrass,
}

impl WaveguideBrassNode {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            brass: WaveguideBrass::new(sample_rate),
        }
    }
}

impl AudioNode for WaveguideBrassNode {
    fn name(&self) -> &str {
        "WaveguideBrassNode"
    }

    fn process(
        &mut self,
        input: &[&[Sample]],
        output: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        self.brass.process_block(input, output, ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::allocator::AllocGuard;
    use summoner_core::transport::Transport;

    #[test]
    fn test_waveguide_brass_bernoulli_oscillation_and_stability() {
        let mut brass = WaveguideBrass::new(48000);
        brass.set_instrument(BrassInstrumentType::Trumpet);
        brass.set_blowing_pressure(0.90);
        brass.tongue_articulation(0.8);

        let mut has_oscillated = false;
        for _ in 0..2000 {
            let sample = brass.process_sample(0.0);
            assert!(sample.is_finite());
            assert!((-1.0..=1.0).contains(&sample));
            if sample.abs() > 0.05 {
                has_oscillated = true;
            }
        }
        assert!(has_oscillated, "Brass lip-reed should self-oscillate under blowing pressure");
    }

    #[test]
    fn test_waveguide_brass_valve_switching_extends_bore() {
        let mut brass = WaveguideBrass::new(48000);
        let d_open = brass.compute_bore_delay_samples();

        brass.set_valves(true, false, false);
        let d_v1 = brass.compute_bore_delay_samples();
        assert!(d_v1 > d_open, "Valve 1 must increase bore acoustic delay");

        brass.set_valves(true, true, true);
        let d_all = brass.compute_bore_delay_samples();
        assert!(d_all > d_v1, "All valves open must maximize bore delay");
    }

    #[test]
    fn test_waveguide_brass_zero_allocation_in_process_block() {
        let mut brass = WaveguideBrass::new(48000);
        brass.set_instrument(BrassInstrumentType::Trombone);
        brass.set_blowing_pressure(0.85);
        brass.tongue_articulation(0.9);

        let transport = Transport::new(48000, 120.0);
        let ctx = ProcessContext::from_transport(&transport);

        let in_buf = [0.0f32; 64];
        let mut out_l = [0.0f32; 64];
        let mut out_r = [0.0f32; 64];

        {
            let _guard = AllocGuard::new();
            brass.process_block(
                &[&in_buf[..]],
                &mut [&mut out_l[..], &mut out_r[..]],
                &ctx,
            );
        }

        assert!(out_l.iter().all(|s| s.is_finite()));
        assert!(out_l.iter().all(|s| (-1.0..=1.0).contains(s)));
    }
}
