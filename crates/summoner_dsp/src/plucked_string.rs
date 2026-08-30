// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Acoustic Plucked & Struck String Commuted Waveguide Synthesizer (Milestone 18).
//!
//! Provides a high-precision dual-polarization commuted digital waveguide string model
//! with non-linear felt hammer & plectrum dynamics, string stiffness dispersion allpass filtering,
//! bridge reflection & palm mute damping, sympathetic resonance matrix coupling, and
//! acoustic soundboard body radiation.
//!
//! Enforces zero heap allocations in audio processing loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};
use crate::hammer_strike::{ExcitationType, HammerStrike, SympatheticResonatorMatrix, NUM_SYMPATHETIC_STRINGS};
use crate::traits::SignalProcessor;

/// Maximum delay buffer capacity in samples (~15 Hz fundamental at 192 kHz).
pub const MAX_PLUCKED_DELAY: usize = 16384;

/// Plucked and struck acoustic instrument preset profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PluckedInstrumentProfile {
    /// Steel-string Acoustic Guitar with bright crisp transient, high stiffness dispersion, and resonant wooden body.
    #[default]
    AcousticSteel,
    /// Classical Nylon Guitar with mellow finger flesh excitation, warm sustain, and low dispersion.
    ClassicalNylon,
    /// Concert Grand Piano with heavy felt hammer strike, 3-string unison coupling, and large soundboard sustain.
    GrandPiano,
    /// Flemish Harpsichord with plucked quill plectrum, rich overtone spectrum, and fast natural release.
    Harpsichord,
    /// Concert Pedal Harp with long ringing sustain, open soundboard radiation, and high sympathetic bleed.
    Harp,
    /// Sitar / Koto with non-linear flat jawari bridge buzz modulation and pitch flexibility.
    SitarKoto,
}

impl PluckedInstrumentProfile {
    /// Returns default fundamental frequency in Hz (Standard pitch).
    pub fn default_fundamental_hz(&self) -> f32 {
        match self {
            Self::AcousticSteel => 196.00,  // G3
            Self::ClassicalNylon => 246.94, // B3
            Self::GrandPiano => 261.63,     // C4
            Self::Harpsichord => 440.00,    // A4
            Self::Harp => 220.00,           // A3
            Self::SitarKoto => 146.83,      // D3
        }
    }

    /// Returns default excitation mechanism for this instrument.
    pub fn default_excitation(&self) -> ExcitationType {
        match self {
            Self::AcousticSteel => ExcitationType::PlectrumPluck,
            Self::ClassicalNylon => ExcitationType::FingerNylon,
            Self::GrandPiano => ExcitationType::HammerStrike,
            Self::Harpsichord => ExcitationType::PlectrumPluck,
            Self::Harp => ExcitationType::FingerNylon,
            Self::SitarKoto => ExcitationType::PlectrumPluck,
        }
    }

    /// Returns nominal $(T_{60}, \text{stiffness}, \text{detune\_cents}, \text{sympathetic\_bleed})$ characteristics.
    pub fn nominal_physics(&self) -> (f32, f32, f32, f32) {
        match self {
            Self::AcousticSteel => (4.5, 0.25, 3.5, 0.15),
            Self::ClassicalNylon => (3.2, 0.08, 2.0, 0.10),
            Self::GrandPiano => (8.0, 0.45, 1.8, 0.35),
            Self::Harpsichord => (2.5, 0.18, 4.0, 0.12),
            Self::Harp => (9.5, 0.05, 1.5, 0.40),
            Self::SitarKoto => (3.8, 0.30, 6.0, 0.25),
        }
    }

    /// Returns 4 soundboard body modal resonance frequencies $(F_1, F_2, F_3, F_4)$ in Hz.
    pub fn soundboard_modes(&self) -> (f32, f32, f32, f32) {
        match self {
            Self::AcousticSteel => (105.0, 210.0, 420.0, 1850.0), // Main air, top plate, back plate, bridge hill
            Self::ClassicalNylon => (95.0, 190.0, 380.0, 1450.0),
            Self::GrandPiano => (65.0, 140.0, 320.0, 2200.0),
            Self::Harpsichord => (120.0, 260.0, 520.0, 2600.0),
            Self::Harp => (80.0, 175.0, 390.0, 1600.0),
            Self::SitarKoto => (110.0, 230.0, 480.0, 2100.0),
        }
    }
}

/// A first-order allpass filter section for string stiffness inharmonic dispersion.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DispersionAllpass {
    pub coefficient: f32,
    x_prev: f32,
    y_prev: f32,
}

impl Default for DispersionAllpass {
    fn default() -> Self {
        Self {
            coefficient: 0.0,
            x_prev: 0.0,
            y_prev: 0.0,
        }
    }
}

impl DispersionAllpass {
    pub fn new(coefficient: f32) -> Self {
        Self {
            coefficient: coefficient.clamp(-0.95, 0.95),
            x_prev: 0.0,
            y_prev: 0.0,
        }
    }

    #[inline]
    pub fn step(&mut self, input: f32) -> f32 {
        // Allpass difference equation: y[n] = a * x[n] + x[n-1] - a * y[n-1]
        let a = self.coefficient;
        let out = a * input + self.x_prev - a * self.y_prev;
        self.x_prev = input;
        self.y_prev = if out.is_finite() { out } else { 0.0 };
        self.y_prev
    }

    pub fn reset(&mut self) {
        self.x_prev = 0.0;
        self.y_prev = 0.0;
    }
}

/// A second-order bandpass biquad section for soundboard body resonance.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SoundboardMode {
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

impl Default for SoundboardMode {
    fn default() -> Self {
        Self::new(200.0, 8.0, 1.0, 48000)
    }
}

impl SoundboardMode {
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

fn default_string_buffer() -> [f32; MAX_PLUCKED_DELAY] {
    [0.0; MAX_PLUCKED_DELAY]
}

/// Physical Modeling Dual-Polarization Commuted Waveguide String Synthesizer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommutedPluckedString {
    /// Active instrument profile.
    pub instrument: PluckedInstrumentProfile,
    /// Fundamental pitch in Hz.
    pub frequency_hz: f32,
    /// Pluck / strike position along the string $\beta \in [0.05, 0.95]$.
    pub pluck_position: f32,
    /// Strike / pluck attack velocity $[0.0 ..= 1.0]$.
    pub strike_velocity: f32,
    /// Hammer felt / plectrum hardness $[0.0 ..= 1.0]$.
    pub hammer_hardness: f32,
    /// Palm mute damping coefficient $[0.0 ..= 1.0]$ (0.0 = open ring, 1.0 = heavy palm mute).
    pub palm_mute: f32,
    /// String stiffness dispersion coefficient $[0.0 ..= 1.0]$.
    pub string_stiffness: f32,
    /// Dual polarization cross-coupling coefficient $[0.0 ..= 1.0]$.
    pub polarization_coupling: f32,
    /// Polarization detuning in cents $[0.0 ..= 25.0]$.
    pub polarization_detune_cents: f32,
    /// Decay time $T_{60}$ in seconds $[0.2 ..= 30.0]$.
    pub decay_t60_sec: f32,
    /// Soundboard body resonance wet mix $[0.0 ..= 1.0]$.
    pub soundboard_mix: f32,
    /// Audio sample rate.
    pub sample_rate: u32,
    /// Non-linear felt hammer & plectrum contact solver.
    pub hammer_strike: HammerStrike,
    /// Sympathetic resonance coupling matrix.
    pub sympathetic_matrix: SympatheticResonatorMatrix,
    /// Soundboard body modal resonator bank.
    pub soundboard_modes: [SoundboardMode; 4],
    /// Vertical polarization dispersion filter.
    pub dispersion_vert: DispersionAllpass,
    /// Horizontal polarization dispersion filter.
    pub dispersion_horiz: DispersionAllpass,

    // Dual-polarization bidirectional delay buffers
    #[serde(skip, default = "default_string_buffer")]
    delay_vert: [f32; MAX_PLUCKED_DELAY],
    #[serde(skip, default = "default_string_buffer")]
    delay_horiz: [f32; MAX_PLUCKED_DELAY],
    #[serde(skip)]
    write_idx_vert: usize,
    #[serde(skip)]
    write_idx_horiz: usize,

    // Bridge termination damping filter states
    #[serde(skip)]
    damp_vert_state: f32,
    #[serde(skip)]
    damp_horiz_state: f32,
    #[serde(skip)]
    dc_vert_state: f32,
    #[serde(skip)]
    dc_horiz_state: f32,
    #[serde(skip)]
    prev_dc_vert_in: f32,
    #[serde(skip)]
    prev_dc_horiz_in: f32,
}

impl Default for CommutedPluckedString {
    fn default() -> Self {
        Self::new(48000)
    }
}

impl CommutedPluckedString {
    /// Creates a new Commuted Plucked String synthesizer.
    pub fn new(sample_rate: u32) -> Self {
        let sr = sample_rate.max(8000);
        let instrument = PluckedInstrumentProfile::AcousticSteel;
        let (t60, stiff, detune, bleed) = instrument.nominal_physics();
        let (f1, f2, f3, f4) = instrument.soundboard_modes();

        let mut synth = Self {
            instrument,
            frequency_hz: instrument.default_fundamental_hz(),
            pluck_position: 0.15,
            strike_velocity: 0.8,
            hammer_hardness: 0.5,
            palm_mute: 0.0,
            string_stiffness: stiff,
            polarization_coupling: 0.35,
            polarization_detune_cents: detune,
            decay_t60_sec: t60,
            soundboard_mix: 0.60,
            sample_rate: sr,
            hammer_strike: HammerStrike::new(sr),
            sympathetic_matrix: SympatheticResonatorMatrix::new(),
            soundboard_modes: [
                SoundboardMode::new(f1, 12.0, 1.2, sr),
                SoundboardMode::new(f2, 16.0, 1.5, sr),
                SoundboardMode::new(f3, 10.0, 1.0, sr),
                SoundboardMode::new(f4, 6.0, 0.8, sr),
            ],
            dispersion_vert: DispersionAllpass::new(stiff * -0.6),
            dispersion_horiz: DispersionAllpass::new(stiff * -0.55),
            delay_vert: [0.0; MAX_PLUCKED_DELAY],
            delay_horiz: [0.0; MAX_PLUCKED_DELAY],
            write_idx_vert: 0,
            write_idx_horiz: 0,
            damp_vert_state: 0.0,
            damp_horiz_state: 0.0,
            dc_vert_state: 0.0,
            dc_horiz_state: 0.0,
            prev_dc_vert_in: 0.0,
            prev_dc_horiz_in: 0.0,
        };
        synth.hammer_strike.set_excitation_type(instrument.default_excitation());
        synth.sympathetic_matrix.set_coupling_strength(bleed);
        synth
    }

    /// Sets the instrument preset profile and reconfigures physics, body modes, and excitation.
    pub fn set_instrument(&mut self, instrument: PluckedInstrumentProfile) {
        self.instrument = instrument;
        self.frequency_hz = instrument.default_fundamental_hz();
        let (t60, stiff, detune, bleed) = instrument.nominal_physics();
        self.decay_t60_sec = t60;
        self.string_stiffness = stiff;
        self.polarization_detune_cents = detune;
        self.hammer_strike.set_excitation_type(instrument.default_excitation());
        self.sympathetic_matrix.set_coupling_strength(bleed);

        let (f1, f2, f3, f4) = instrument.soundboard_modes();
        self.soundboard_modes[0] = SoundboardMode::new(f1, 12.0, 1.2, self.sample_rate);
        self.soundboard_modes[1] = SoundboardMode::new(f2, 16.0, 1.5, self.sample_rate);
        self.soundboard_modes[2] = SoundboardMode::new(f3, 10.0, 1.0, self.sample_rate);
        self.soundboard_modes[3] = SoundboardMode::new(f4, 6.0, 0.8, self.sample_rate);

        self.dispersion_vert.coefficient = stiff * -0.6;
        self.dispersion_horiz.coefficient = stiff * -0.55;
    }

    /// Sets target fundamental frequency in Hz.
    pub fn set_frequency(&mut self, freq_hz: f32) {
        self.frequency_hz = freq_hz.clamp(20.0, self.sample_rate as f32 * 0.45);
    }

    /// Sets pluck / strike position along the string $\beta \in [0.05, 0.95]$.
    pub fn set_pluck_position(&mut self, beta: f32) {
        self.pluck_position = beta.clamp(0.05, 0.95);
        self.hammer_strike.params.strike_position = self.pluck_position;
    }

    /// Sets palm mute damping factor $[0.0 ..= 1.0]$.
    pub fn set_palm_mute(&mut self, mute: f32) {
        self.palm_mute = mute.clamp(0.0, 1.0);
    }

    /// Sets string stiffness dispersion coefficient $[0.0 ..= 1.0]$.
    pub fn set_string_stiffness(&mut self, stiffness: f32) {
        self.string_stiffness = stiffness.clamp(0.0, 1.0);
        self.dispersion_vert.coefficient = self.string_stiffness * -0.6;
        self.dispersion_horiz.coefficient = self.string_stiffness * -0.55;
    }

    /// Triggers a pluck or strike excitation with given velocity and hardness.
    pub fn trigger_pluck(&mut self, velocity: f32, hardness: f32) {
        let vel = velocity.clamp(0.01, 1.0);
        let hard = hardness.clamp(0.0, 1.0);
        self.strike_velocity = vel;
        self.hammer_hardness = hard;
        self.hammer_strike.trigger(vel, hard);

        // Inject initial spatial excitation comb burst directly into waveguide delay lines
        let (beta, amp) = self.hammer_strike.spatial_comb_coefficients();
        let total_period = self.sample_rate as f32 / self.frequency_hz.max(20.0);
        let strike_idx = (total_period * beta).clamp(1.0, (MAX_PLUCKED_DELAY - 4) as f32) as usize;

        let strike_impulse = vel * amp * 0.85;
        let w_idx_v = self.write_idx_vert;
        let w_idx_h = self.write_idx_horiz;

        let target_v = (w_idx_v + strike_idx) % MAX_PLUCKED_DELAY;
        let target_h = (w_idx_h + strike_idx) % MAX_PLUCKED_DELAY;

        self.delay_vert[target_v] += strike_impulse * 0.80;
        self.delay_horiz[target_h] += strike_impulse * 0.60;
    }

    /// Resets all delay lines, filters, and hammer states.
    pub fn reset(&mut self) {
        self.delay_vert = [0.0; MAX_PLUCKED_DELAY];
        self.delay_horiz = [0.0; MAX_PLUCKED_DELAY];
        self.write_idx_vert = 0;
        self.write_idx_horiz = 0;
        self.damp_vert_state = 0.0;
        self.damp_horiz_state = 0.0;
        self.dc_vert_state = 0.0;
        self.dc_horiz_state = 0.0;
        self.prev_dc_vert_in = 0.0;
        self.prev_dc_horiz_in = 0.0;
        self.hammer_strike.reset();
        self.sympathetic_matrix.reset();
        self.dispersion_vert.reset();
        self.dispersion_horiz.reset();
        for mode in &mut self.soundboard_modes {
            mode.reset();
        }
    }

    /// Reads fractional delay sample using 4-point Hermite cubic interpolation.
    #[inline]
    fn read_fractional_delay(buffer: &[f32; MAX_PLUCKED_DELAY], write_idx: usize, delay_samples: f32) -> f32 {
        let int_delay = delay_samples.floor() as usize;
        let frac = delay_samples - int_delay as f32;

        let base_idx = if write_idx >= int_delay {
            write_idx - int_delay
        } else {
            MAX_PLUCKED_DELAY + write_idx - int_delay
        };

        let i0 = if base_idx == 0 { MAX_PLUCKED_DELAY - 1 } else { base_idx - 1 };
        let i1 = base_idx;
        let i2 = (base_idx + 1) % MAX_PLUCKED_DELAY;
        let i3 = (base_idx + 2) % MAX_PLUCKED_DELAY;

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

    /// Computes loop gain factor from $T_{60}$ decay specification.
    #[inline]
    fn compute_loop_gain(&self, fundamental_hz: f32) -> f32 {
        let eff_t60 = if self.palm_mute > 0.01 {
            self.decay_t60_sec * (1.0 - 0.85 * self.palm_mute).max(0.05)
        } else {
            self.decay_t60_sec
        };
        let period_sec = 1.0 / fundamental_hz.max(20.0);
        let attenuation_db_per_sec = 60.0 / eff_t60.max(0.1);
        let attenuation_db_per_period = attenuation_db_per_sec * period_sec;
        10.0f32.powf(-attenuation_db_per_period / 20.0).clamp(0.85, 0.9999)
    }

    /// Computes and returns the next output audio sample.
    #[inline]
    pub fn process_sample(&mut self, external_excitation: f32) -> Sample {
        let sr = self.sample_rate as f32;

        // 1. Calculate delay lengths for vertical and horizontal polarizations
        let f0_vert = self.frequency_hz.clamp(20.0, sr * 0.45);
        let detune_ratio = 2.0f32.powf((self.polarization_detune_cents / 1200.0) / 12.0);
        let f0_horiz = (f0_vert * detune_ratio).clamp(20.0, sr * 0.45);

        let delay_vert_len = (sr / f0_vert).clamp(2.0, (MAX_PLUCKED_DELAY - 4) as f32);
        let delay_horiz_len = (sr / f0_horiz).clamp(2.0, (MAX_PLUCKED_DELAY - 4) as f32);

        // 2. Read circulating traveling waves from delay buffers
        let wave_v = Self::read_fractional_delay(&self.delay_vert, self.write_idx_vert, delay_vert_len);
        let wave_h = Self::read_fractional_delay(&self.delay_horiz, self.write_idx_horiz, delay_horiz_len);

        // 3. String stiffness inharmonic dispersion (allpass filtering)
        let disp_v = self.dispersion_vert.step(wave_v);
        let disp_h = self.dispersion_horiz.step(wave_h);

        // 4. Non-linear hammer contact dynamics step
        let string_disp = (disp_v + disp_h) * 0.001; // Approximate displacement
        let contact_res = self.hammer_strike.step(string_disp, 0.0);
        let hammer_force = contact_res.contact_force_n * 0.05 + external_excitation;

        // 5. Cross-polarization coupling between vertical and horizontal planes
        let k_pol = self.polarization_coupling.clamp(0.0, 1.0) * 0.15;
        let coupled_v = disp_v * (1.0 - k_pol) + disp_h * k_pol;
        let coupled_h = disp_h * (1.0 - k_pol) + disp_v * k_pol;

        // 6. Bridge reflection and termination loss filtering (one-pole lowpass + DC blocker)
        let loop_gain_v = self.compute_loop_gain(f0_vert);
        let loop_gain_h = self.compute_loop_gain(f0_horiz);

        let damp_coeff_v = (0.25 + 0.50 * self.palm_mute).clamp(0.10, 0.95);
        let damp_coeff_h = (0.35 + 0.50 * self.palm_mute).clamp(0.10, 0.95);

        // Phase inversion -1.0 at rigid bridge boundary
        let reflected_v = -coupled_v * loop_gain_v;
        let reflected_h = -coupled_h * loop_gain_h;

        self.damp_vert_state = (1.0 - damp_coeff_v) * reflected_v + damp_coeff_v * self.damp_vert_state;
        self.damp_horiz_state = (1.0 - damp_coeff_h) * reflected_h + damp_coeff_h * self.damp_horiz_state;

        // DC blocker: y[n] = x[n] - x[n-1] + 0.995 * y[n-1]
        let dc_v = self.damp_vert_state - self.prev_dc_vert_in + 0.995 * self.dc_vert_state;
        self.prev_dc_vert_in = self.damp_vert_state;
        self.dc_vert_state = dc_v;

        let dc_h = self.damp_horiz_state - self.prev_dc_horiz_in + 0.995 * self.dc_horiz_state;
        self.prev_dc_horiz_in = self.damp_horiz_state;
        self.dc_horiz_state = dc_h;

        // 7. Multi-string Sympathetic Resonance Coupling
        let mut sym_in = [0.0f32; NUM_SYMPATHETIC_STRINGS];
        sym_in[0] = dc_v;
        sym_in[1] = dc_h;
        let mut sym_out = [0.0f32; NUM_SYMPATHETIC_STRINGS];
        self.sympathetic_matrix.process_junction(&sym_in, &mut sym_out);

        // 8. Inject hammer contact force and feedback waves back into delay lines
        let in_v = (sym_out[0] + hammer_force * 0.70).clamp(-2.0, 2.0);
        let in_h = (sym_out[1] + hammer_force * 0.40).clamp(-2.0, 2.0);

        self.delay_vert[self.write_idx_vert] = in_v;
        self.write_idx_vert = (self.write_idx_vert + 1) % MAX_PLUCKED_DELAY;

        self.delay_horiz[self.write_idx_horiz] = in_h;
        self.write_idx_horiz = (self.write_idx_horiz + 1) % MAX_PLUCKED_DELAY;

        // 9. Soundboard body resonance radiation convolver
        let bridge_driving_force = dc_v * 1.2 + dc_h * 0.8 + hammer_force * 0.3;
        let mut body_radiated = 0.0f32;
        for mode in &mut self.soundboard_modes {
            body_radiated += mode.step(bridge_driving_force);
        }

        // 10. Dry / Wet Soundboard Mix
        let dry_string = (dc_v + dc_h * 0.7) * 0.5;
        let mix = self.soundboard_mix.clamp(0.0, 1.0);
        let out_sample = (1.0 - mix) * dry_string + mix * body_radiated * 0.5;

        out_sample.clamp(-1.0, 1.0)
    }
}

impl SignalProcessor for CommutedPluckedString {
    fn name(&self) -> &str {
        "CommutedPluckedString"
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

/// AudioNode wrapper for Commuted Plucked String physical modeling synthesizer.
#[derive(Debug)]
pub struct CommutedPluckedStringNode {
    pub plucked_string: CommutedPluckedString,
}

impl CommutedPluckedStringNode {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            plucked_string: CommutedPluckedString::new(sample_rate),
        }
    }
}

impl AudioNode for CommutedPluckedStringNode {
    fn name(&self) -> &str {
        "CommutedPluckedStringNode"
    }

    fn process(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        self.plucked_string.process_block(inputs, outputs, ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::allocator::AllocGuard;

    #[test]
    fn test_commuted_plucked_string_decay_and_soundboard() {
        let mut synth = CommutedPluckedString::new(48000);
        synth.set_instrument(PluckedInstrumentProfile::AcousticSteel);
        synth.set_frequency(196.0);
        synth.trigger_pluck(0.9, 0.6);

        let mut peak_amp = 0.0f32;
        let mut late_amp = 0.0f32;

        for i in 0..4800 {
            let s = synth.process_sample(0.0);
            assert!(s.is_finite());
            assert!((-1.0..=1.0).contains(&s));
            if i < 480 {
                peak_amp = peak_amp.max(s.abs());
            } else if i > 4000 {
                late_amp = late_amp.max(s.abs());
            }
        }

        assert!(peak_amp > 0.05, "String attack must produce non-zero sound");
        assert!(late_amp < peak_amp, "String vibration must naturally decay");
    }

    #[test]
    fn test_commuted_plucked_string_zero_allocation_in_loop() {
        let mut synth = CommutedPluckedString::new(48000);
        synth.trigger_pluck(0.8, 0.5);

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
