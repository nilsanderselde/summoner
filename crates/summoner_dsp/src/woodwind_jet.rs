// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Woodwind Air-Jet Embouchure & Tonehole Radiation Impedance Waveguide (Milestone 17).
//!
//! Provides a physical acoustic woodwind synthesizer based on a non-linear air-jet flue embouchure
//! excitation model (Bernoulli jet velocity, jet transit delay line, hyperbolic/polynomial jet deflection,
//! labium splitting edge) coupled to a bidirectional digital waveguide acoustic bore with discrete
//! 6-tonehole lattice scattering junctions and radiation filters.
//!
//! Enforces zero heap allocations in audio processing loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};
use crate::tonehole_grid::{NUM_TONEHOLES, ToneholeLattice, SPEED_OF_SOUND_MPS, AIR_DENSITY_KG_M3};
use crate::traits::SignalProcessor;

/// Maximum delay line capacity in samples (~20 Hz at 192 kHz).
pub const MAX_WOODWIND_DELAY: usize = 8192;

/// Maximum air-jet propagation delay buffer capacity in samples (~50 ms at 192 kHz).
pub const MAX_JET_DELAY: usize = 4096;

/// Woodwind acoustic instrument preset profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WoodwindInstrumentProfile {
    /// Concert Flute C4 with cylindrical headjoint and open toneholes.
    #[default]
    FluteC,
    /// Piccolo C5 with conical tapered body and high radiation cutoff.
    PiccoloC,
    /// Alto Recorder with fixed fipple flue duct.
    RecorderAlto,
    /// Japanese Shakuhachi bamboo flute with wide open blowing edge and heavy breath modulation.
    Shakuhachi,
    /// Pan Flute stopped pipe array with strong odd harmonic distribution.
    PanFlute,
}

impl WoodwindInstrumentProfile {
    /// Returns default fundamental frequency in Hz (all holes closed).
    pub fn default_fundamental_hz(&self) -> f32 {
        match self {
            Self::FluteC => 261.63,       // C4
            Self::PiccoloC => 523.25,     // C5
            Self::RecorderAlto => 349.23, // F4
            Self::Shakuhachi => 293.66,   // D4
            Self::PanFlute => 261.63,     // C4
        }
    }

    /// Returns nominal total bore physical length in meters.
    pub fn nominal_bore_length_m(&self) -> f32 {
        match self {
            Self::FluteC => 0.60,
            Self::PiccoloC => 0.32,
            Self::RecorderAlto => 0.47,
            Self::Shakuhachi => 0.545,
            Self::PanFlute => 0.30,
        }
    }

    /// Returns nominal embouchure jet distance in millimeters.
    pub fn nominal_jet_distance_mm(&self) -> f32 {
        match self {
            Self::FluteC => 7.0,
            Self::PiccoloC => 4.5,
            Self::RecorderAlto => 3.5,
            Self::Shakuhachi => 10.0,
            Self::PanFlute => 5.0,
        }
    }

    /// Returns whether this instrument is an acoustically stopped pipe (e.g. Pan Flute).
    pub fn is_stopped_pipe(&self) -> bool {
        matches!(self, Self::PanFlute)
    }

    /// Returns nominal air breath noise injection fraction.
    pub fn breath_noise_fraction(&self) -> f32 {
        match self {
            Self::FluteC => 0.08,
            Self::PiccoloC => 0.05,
            Self::RecorderAlto => 0.04,
            Self::Shakuhachi => 0.18,
            Self::PanFlute => 0.12,
        }
    }
}

fn default_bore_buffer() -> [f32; MAX_WOODWIND_DELAY] {
    [0.0; MAX_WOODWIND_DELAY]
}

fn default_jet_buffer() -> [f32; MAX_JET_DELAY] {
    [0.0; MAX_JET_DELAY]
}

/// Physical Modeling Woodwind Air-Jet Waveguide Synthesizer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WoodwindJet {
    /// Active instrument profile.
    pub instrument: WoodwindInstrumentProfile,
    /// Target fundamental frequency in Hz.
    pub frequency_hz: f32,
    /// Blowing mouth pressure in kPa [0.05 ..= 4.00].
    pub blowing_pressure_kpa: f32,
    /// Distance from lip/flue slit to labium splitting edge in mm [1.5 ..= 15.0].
    pub jet_distance_mm: f32,
    /// Embouchure jet angle relative to labium in radians [-0.5 ..= 0.5].
    pub embouchure_angle_rad: f32,
    /// Non-linear jet amplification gain coefficient alpha [0.5 ..= 4.0].
    pub jet_gain: f32,
    /// Bore wall thermal-viscous damping loss [0.95 ..= 0.999].
    pub bore_loss: f32,
    /// Bell / open end radiation reflection coefficient [-0.99 ..= -0.80].
    pub bell_reflection_coeff: f32,
    /// Discrete 6-tonehole acoustic lattice.
    pub lattice: ToneholeLattice,
    /// Vibrato depth in semitones [0.0 ..= 2.0].
    pub vibrato_depth: f32,
    /// Vibrato rate in Hz [0.5 ..= 15.0].
    pub vibrato_rate_hz: f32,
    /// Flutter tongue frequency in Hz [0.0 = disabled, 10.0 ..= 40.0].
    pub flutter_rate_hz: f32,
    /// Breath turbulence air noise mix [0.0 ..= 0.50].
    pub breath_noise_mix: f32,
    /// Audio sample rate.
    pub sample_rate: u32,

    // Internal delay lines for bidirectional acoustic bore
    #[serde(skip, default = "default_bore_buffer")]
    fwd_delay: [f32; MAX_WOODWIND_DELAY],
    #[serde(skip, default = "default_bore_buffer")]
    rev_delay: [f32; MAX_WOODWIND_DELAY],
    #[serde(skip)]
    fwd_write_idx: usize,
    #[serde(skip)]
    rev_write_idx: usize,

    // Internal delay line for air-jet travel from flue slit to labium edge
    #[serde(skip, default = "default_jet_buffer")]
    jet_delay: [f32; MAX_JET_DELAY],
    #[serde(skip)]
    jet_write_idx: usize,

    // Damping and filter history
    #[serde(skip)]
    prev_bore_loss_sample: f32,
    #[serde(skip)]
    prev_radiation_out: f32,
    #[serde(skip)]
    prev_rad_diff: f32,

    // Phase accumulators for modulation
    #[serde(skip)]
    vibrato_phase: f32,
    #[serde(skip)]
    flutter_phase: f32,
    #[serde(skip)]
    prng_state: u64,
}

impl Default for WoodwindJet {
    fn default() -> Self {
        Self::new(48000)
    }
}

impl WoodwindJet {
    /// Creates a new Woodwind Air-Jet modeler initialized for the given sample rate.
    pub fn new(sample_rate: u32) -> Self {
        let sr = sample_rate.max(8000);
        let instrument = WoodwindInstrumentProfile::FluteC;
        let mut woodwind = Self {
            instrument,
            frequency_hz: instrument.default_fundamental_hz(),
            blowing_pressure_kpa: 1.25,
            jet_distance_mm: instrument.nominal_jet_distance_mm(),
            embouchure_angle_rad: 0.0,
            jet_gain: 1.8,
            bore_loss: 0.997,
            bell_reflection_coeff: -0.92,
            lattice: ToneholeLattice::new_standard_flute(instrument.nominal_bore_length_m()),
            vibrato_depth: 0.0,
            vibrato_rate_hz: 5.5,
            flutter_rate_hz: 0.0,
            breath_noise_mix: instrument.breath_noise_fraction(),
            sample_rate: sr,
            fwd_delay: [0.0; MAX_WOODWIND_DELAY],
            rev_delay: [0.0; MAX_WOODWIND_DELAY],
            fwd_write_idx: 0,
            rev_write_idx: 0,
            jet_delay: [0.0; MAX_JET_DELAY],
            jet_write_idx: 0,
            prev_bore_loss_sample: 0.0,
            prev_radiation_out: 0.0,
            prev_rad_diff: 0.0,
            vibrato_phase: 0.0,
            flutter_phase: 0.0,
            prng_state: 0x9E3779B97F4A7C15,
        };
        woodwind.update_frequency_tuning();
        woodwind
    }

    /// Sets instrument preset profile and adjusts physical geometry.
    pub fn set_instrument(&mut self, instrument: WoodwindInstrumentProfile) {
        self.instrument = instrument;
        self.frequency_hz = instrument.default_fundamental_hz();
        self.jet_distance_mm = instrument.nominal_jet_distance_mm();
        self.breath_noise_mix = instrument.breath_noise_fraction();
        self.lattice = ToneholeLattice::new_standard_flute(instrument.nominal_bore_length_m());
        self.bell_reflection_coeff = if instrument.is_stopped_pipe() { 0.95 } else { -0.92 };
        self.update_frequency_tuning();
    }

    /// Sets target fundamental frequency in Hz.
    pub fn set_frequency(&mut self, frequency_hz: f32) {
        self.frequency_hz = frequency_hz.clamp(20.0, self.sample_rate as f32 * 0.45);
        self.update_frequency_tuning();
    }

    /// Sets blowing mouth pressure in kPa [0.05 ..= 4.00].
    pub fn set_blowing_pressure(&mut self, pressure_kpa: f32) {
        self.blowing_pressure_kpa = pressure_kpa.clamp(0.05, 4.00);
    }

    /// Sets embouchure distance in mm [1.5 ..= 15.0].
    pub fn set_jet_distance(&mut self, distance_mm: f32) {
        self.jet_distance_mm = distance_mm.clamp(1.5, 15.0);
    }

    /// Sets embouchure jet angle relative to labium in radians [-0.5 ..= 0.5].
    pub fn set_embouchure_angle(&mut self, angle_rad: f32) {
        self.embouchure_angle_rad = angle_rad.clamp(-0.5, 0.5);
    }

    /// Sets 6-tonehole fingering bitmask (bits 0..5: 1 = closed, 0 = open).
    pub fn set_fingering_mask(&mut self, mask: u8) {
        self.lattice.set_fingering_mask(mask);
    }

    /// Sets discrete boolean states for all 6 toneholes.
    pub fn set_tonehole_states(&mut self, states: [bool; NUM_TONEHOLES]) {
        self.lattice.set_tonehole_states(states);
    }

    /// Sets continuous fractional openings for all 6 toneholes for microtonal shading.
    pub fn set_open_fractions(&mut self, fractions: [f32; NUM_TONEHOLES]) {
        self.lattice.set_open_fractions(fractions);
    }

    /// Sets continuous vibrato parameters (depth in semitones, rate in Hz).
    pub fn set_vibrato(&mut self, depth_semitones: f32, rate_hz: f32) {
        self.vibrato_depth = depth_semitones.clamp(0.0, 2.0);
        self.vibrato_rate_hz = rate_hz.clamp(0.5, 15.0);
    }

    /// Sets flutter tongue rate in Hz (0.0 = disabled).
    pub fn set_flutter_tongue(&mut self, flutter_hz: f32) {
        self.flutter_rate_hz = flutter_hz.clamp(0.0, 50.0);
    }

    /// Sets breath air turbulence noise mix fraction [0.0 ..= 0.50].
    pub fn set_breath_noise(&mut self, noise_mix: f32) {
        self.breath_noise_mix = noise_mix.clamp(0.0, 0.50);
    }

    /// Applies a tonguing articulation attack impulse.
    pub fn tongue_articulation(&mut self, velocity: f32) {
        let vel = velocity.clamp(0.0, 1.0);
        self.prev_radiation_out += vel * 0.35;
        let mut idx = self.jet_write_idx;
        for _ in 0..16 {
            idx = if idx == 0 { MAX_JET_DELAY - 1 } else { idx - 1 };
            self.jet_delay[idx] += vel * 0.15;
        }
    }

    /// Updates internal geometry based on pitch and instrument settings.
    fn update_frequency_tuning(&mut self) {
        let eff_len = if self.instrument.is_stopped_pipe() {
            SPEED_OF_SOUND_MPS / (4.0 * self.frequency_hz.max(20.0))
        } else {
            SPEED_OF_SOUND_MPS / (2.0 * self.frequency_hz.max(20.0))
        };
        self.lattice.total_bore_length_m = eff_len.clamp(0.10, 2.00);
        self.lattice.recalculate_lattice_cutoff();
    }

    /// Computes Bernoulli jet velocity $V_{jet} = \sqrt{\frac{2 P_m}{\rho_{air}}}$.
    #[inline]
    pub fn compute_jet_velocity_mps(&self, modulated_pressure_kpa: f32) -> f32 {
        let p_pa = (modulated_pressure_kpa * 1000.0).max(10.0);
        (2.0 * p_pa / AIR_DENSITY_KG_M3).sqrt().clamp(5.0, 140.0)
    }

    /// Computes air-jet transit delay in samples $\tau_{jet} = \frac{d_{jet}}{0.4 \cdot V_{jet}} \cdot f_s$.
    #[inline]
    pub fn compute_jet_delay_samples(&self, v_jet_mps: f32) -> f32 {
        let d_m = self.jet_distance_mm * 1e-3;
        let v_profile = (0.4 * v_jet_mps).max(1.0);
        let tau_sec = d_m / v_profile;
        (tau_sec * self.sample_rate as f32).clamp(1.0, (MAX_JET_DELAY - 4) as f32)
    }

    /// Computes total acoustic bore delay in samples.
    #[inline]
    pub fn compute_bore_delay_samples(&self) -> f32 {
        let sr = self.sample_rate as f32;
        let eff_length_m = self.lattice.effective_acoustic_length_m();
        let transit_time_sec = eff_length_m / SPEED_OF_SOUND_MPS;
        (transit_time_sec * sr).clamp(2.0, (MAX_WOODWIND_DELAY - 4) as f32)
    }

    /// Non-allocating fast pseudo-random number generator for breath turbulence noise.
    #[inline]
    fn next_noise_sample(&mut self) -> f32 {
        self.prng_state = self.prng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let bits = (self.prng_state >> 32) as u32;
        (bits as f32 / u32::MAX as f32) * 2.0 - 1.0
    }

    /// Reads fractional delay using 4-point Hermite cubic interpolation.
    #[inline]
    fn read_fractional_delay(buffer: &[f32], max_cap: usize, write_idx: usize, delay_samples: f32) -> f32 {
        let int_delay = delay_samples.floor() as usize;
        let frac = delay_samples - int_delay as f32;

        let base_idx = if write_idx >= int_delay {
            write_idx - int_delay
        } else {
            max_cap + write_idx - int_delay
        };

        let i0 = if base_idx == 0 { max_cap - 1 } else { base_idx - 1 };
        let i1 = base_idx;
        let i2 = (base_idx + 1) % max_cap;
        let i3 = (base_idx + 2) % max_cap;

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

    /// Evaluates non-linear air-jet splitting and amplification at the labium edge.
    #[inline]
    fn step_jet_nonlinearity(&self, jet_in: f32) -> f32 {
        let u = (jet_in + self.embouchure_angle_rad * 0.5) * self.jet_gain;
        // Hyperbolic tangent non-linear saturation characteristic of air-jet splitting
        u.tanh()
    }

    /// Computes and returns the next output audio sample.
    #[inline]
    pub fn process_sample(&mut self, excitation: f32) -> Sample {
        let sr = self.sample_rate as f32;

        // 1. Advance vibrato and flutter tongue phase accumulators
        let vibrato_mod = if self.vibrato_depth > 0.001 {
            let mod_val = (self.vibrato_phase).sin() * (self.vibrato_depth / 12.0);
            self.vibrato_phase = (self.vibrato_phase + 2.0 * PI * self.vibrato_rate_hz / sr) % (2.0 * PI);
            2.0f32.powf(mod_val)
        } else {
            1.0
        };

        let flutter_mod = if self.flutter_rate_hz > 1.0 {
            let f_val = 0.5 * (1.0 + (self.flutter_phase).sin());
            self.flutter_phase = (self.flutter_phase + 2.0 * PI * self.flutter_rate_hz / sr) % (2.0 * PI);
            0.6 + 0.4 * f_val
        } else {
            1.0
        };

        // 2. Modulate blowing pressure with excitation, flutter, and breath noise
        let noise = self.next_noise_sample();
        let noise_term = noise * self.breath_noise_mix * 0.35;
        let eff_pressure = ((self.blowing_pressure_kpa * flutter_mod + excitation + noise_term) * vibrato_mod)
            .clamp(0.02, 5.0);

        // 3. Calculate aerodynamic jet velocity and propagation delay
        let v_jet = self.compute_jet_velocity_mps(eff_pressure);
        let jet_delay_len = self.compute_jet_delay_samples(v_jet);

        // 4. Read backward acoustic wave emerging from bore back to embouchure
        let bore_delay_len = self.compute_bore_delay_samples();
        let bore_back_wave = Self::read_fractional_delay(&self.rev_delay, MAX_WOODWIND_DELAY, self.rev_write_idx, bore_delay_len);

        // 5. Air-jet excitation: coupling between mouth pressure, acoustic feedback, and jet delay
        // Perturbation injected into the jet is proportional to acoustic feedback wave
        let jet_input = bore_back_wave * 0.40 + noise_term * 0.20;
        self.jet_delay[self.jet_write_idx] = jet_input;
        self.jet_write_idx = (self.jet_write_idx + 1) % MAX_JET_DELAY;

        let delayed_jet = Self::read_fractional_delay(&self.jet_delay, MAX_JET_DELAY, self.jet_write_idx, jet_delay_len);

        // Non-linear amplification at labium edge
        let jet_flow = self.step_jet_nonlinearity(delayed_jet);

        // Forward wave injected into bore: superposition of acoustic feedback and non-linear jet volume flow
        let fwd_bore_in = (-bore_back_wave * 0.65 + jet_flow * (eff_pressure * 0.45).min(1.5)).clamp(-2.0, 2.0);

        self.fwd_delay[self.fwd_write_idx] = fwd_bore_in;
        self.fwd_write_idx = (self.fwd_write_idx + 1) % MAX_WOODWIND_DELAY;

        // 6. Forward wave propagates down the bore to tonehole lattice
        let fwd_bell_wave = Self::read_fractional_delay(&self.fwd_delay, MAX_WOODWIND_DELAY, self.fwd_write_idx, bore_delay_len);

        // Apply thermal-viscous wall loss damping (one-pole lowpass)
        let loss = self.bore_loss.clamp(0.92, 0.999);
        let fwd_damped = loss * fwd_bell_wave + (1.0 - loss) * self.prev_bore_loss_sample;
        self.prev_bore_loss_sample = fwd_damped;

        // 7. Scattering through discrete 6-tonehole lattice
        let mut radiated_total = 0.0f32;
        let mut current_fwd = fwd_damped;
        let mut current_rev = 0.0f32;

        // Process scattering at active toneholes
        for hole in self.lattice.toneholes.iter_mut() {
            let res = hole.process_scattering(current_fwd, current_rev, self.lattice.junction_loss);
            current_fwd = res.transmitted_downstream;
            current_rev += res.reflected_upstream;
            radiated_total += res.radiated_pressure;
        }

        // Open bell / end reflection
        let bell_reflected = current_fwd * self.bell_reflection_coeff;
        let bell_radiated = current_fwd * (1.0 + self.bell_reflection_coeff.abs()) * 0.5;
        let total_return_rev = current_rev + bell_reflected;

        self.rev_delay[self.rev_write_idx] = total_return_rev.clamp(-2.0, 2.0);
        self.rev_write_idx = (self.rev_write_idx + 1) % MAX_WOODWIND_DELAY;

        // 8. Total radiated acoustic pressure (toneholes + bell)
        let total_radiated = radiated_total + bell_radiated;

        // Highpass differentiator for acoustic spherical radiation
        let rad_diff = total_radiated - self.prev_radiation_out;
        self.prev_radiation_out = total_radiated;

        let alpha_rad = 0.92;
        let final_out = alpha_rad * (self.prev_rad_diff + rad_diff);
        self.prev_rad_diff = final_out;

        final_out.clamp(-1.0, 1.0)
    }

    /// Resets all delay lines, filters, and phase accumulators to zero.
    pub fn reset(&mut self) {
        self.fwd_delay = [0.0; MAX_WOODWIND_DELAY];
        self.rev_delay = [0.0; MAX_WOODWIND_DELAY];
        self.fwd_write_idx = 0;
        self.rev_write_idx = 0;
        self.jet_delay = [0.0; MAX_JET_DELAY];
        self.jet_write_idx = 0;
        self.prev_bore_loss_sample = 0.0;
        self.prev_radiation_out = 0.0;
        self.prev_rad_diff = 0.0;
        self.vibrato_phase = 0.0;
        self.flutter_phase = 0.0;
        self.lattice.reset();
    }
}

impl SignalProcessor for WoodwindJet {
    fn name(&self) -> &str {
        "WoodwindJet"
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

/// AudioNode wrapper for Woodwind Jet physical modeling synthesizer.
#[derive(Debug)]
pub struct WoodwindJetNode {
    pub woodwind: WoodwindJet,
}

impl WoodwindJetNode {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            woodwind: WoodwindJet::new(sample_rate),
        }
    }
}

impl AudioNode for WoodwindJetNode {
    fn name(&self) -> &str {
        "WoodwindJetNode"
    }

    fn process(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        self.woodwind.process_block(inputs, outputs, ctx);
    }
}
