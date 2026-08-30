// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Electromechanical Tine & Reed Resonator and Inductive Pickup (Milestone 22).
//!
//! Provides a high-precision electromechanical cantilever beam / clamped vibrating reed model
//! with non-linear rubber/felt hammer contact compliance ($F \propto \delta^{1.5}$), inharmonic
//! overtone dispersion allpass cascade, resonant tonebar energy coupling, damper clunk transient
//! mechanics, and non-linear electromagnetic inductive pickup transfer function with air-gap
//! asymmetry and variable velocity barking saturation.
//!
//! Enforces zero heap allocations in audio processing loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

/// Maximum number of allpass dispersion stages for beam stiffness simulation.
pub const MAX_DISPERSION_STAGES: usize = 4;

/// Maximum delay buffer capacity in samples (~20 Hz fundamental at 192 kHz).
pub const MAX_TINE_DELAY: usize = 16384;

/// Physical electromechanical resonator model type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ElectricPianoModel {
    /// Rhodes Tine: Clamped cantilever wire tine coupled to a massive tuned steel tonebar.
    /// Rich inharmonic bell transient with long sustaining sinusoidal fundamental.
    #[default]
    RhodesTine,
    /// Wurlitzer Reed: Clamped vibrating flat steel reed with electrostatic/inductive pickup.
    /// Grittier, hollower overtone spectrum with aggressive midrange bite and fast bark.
    WurlitzerReed,
}

/// Rubber / Neoprene / Felt hammer strike compliance model.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HammerModel {
    /// Hammer tip hardness $[0.0 ..= 1.0]$ (0.0 = soft felt, 1.0 = hard aged rubber).
    pub hardness: f32,
    /// Strike velocity $[0.0 ..= 1.0]$.
    pub velocity: f32,
    /// Contact duration remaining in samples.
    pub contact_samples: usize,
    /// Total contact duration in samples.
    pub total_contact_samples: usize,
    /// Hammer displacement coordinate.
    pub hammer_pos: f32,
    /// Hammer velocity.
    pub hammer_vel: f32,
    /// Compression exponent (typically 1.5 for Hertzian contact).
    pub exponent: f32,
}

impl Default for HammerModel {
    fn default() -> Self {
        Self {
            hardness: 0.5,
            velocity: 0.8,
            contact_samples: 0,
            total_contact_samples: 32,
            hammer_pos: 0.0,
            hammer_vel: 0.0,
            exponent: 1.5,
        }
    }
}

impl HammerModel {
    /// Strikes the resonator with a given velocity and sample rate.
    pub fn strike(&mut self, velocity: f32, hardness: f32, sample_rate: u32) {
        self.velocity = velocity.clamp(0.0, 1.0);
        self.hardness = hardness.clamp(0.0, 1.0);

        // Harder strikes and higher velocities result in shorter contact duration
        let base_duration_ms = 4.5 - self.hardness * 2.8 - self.velocity * 1.0;
        let duration_ms = base_duration_ms.max(0.4);
        let duration_samples = ((duration_ms * 0.001) * sample_rate as f32).round() as usize;

        self.total_contact_samples = duration_samples.max(4);
        self.contact_samples = self.total_contact_samples;
        self.hammer_vel = self.velocity * (1.0 + self.hardness * 0.8);
        self.hammer_pos = 0.0;
    }

    /// Computes non-linear contact force for the current sample.
    #[inline]
    pub fn compute_force(&mut self, tine_displacement: f32) -> f32 {
        if self.contact_samples == 0 {
            return 0.0;
        }

        self.contact_samples -= 1;
        let progress = 1.0 - (self.contact_samples as f32 / self.total_contact_samples as f32);

        // Semi-elliptical / raised cosine compliance window scaled by velocity and hardness
        let window = (progress * PI).sin();
        let penetration = (self.hammer_vel * window - tine_displacement).max(0.0);

        // Hertzian contact force F = k * delta^1.5
        let stiffness = 2.5 + self.hardness * 6.5;
        let force = stiffness * penetration.powf(self.exponent) * window;

        force.clamp(0.0, 4.0)
    }
}

fn default_tine_buffer() -> [f32; MAX_TINE_DELAY] {
    [0.0; MAX_TINE_DELAY]
}

/// First-order allpass filter section for cantilever beam inharmonic dispersion.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TineDispersionAllpass {
    pub coefficient: f32,
    x_prev: f32,
    y_prev: f32,
}

impl Default for TineDispersionAllpass {
    fn default() -> Self {
        Self {
            coefficient: 0.0,
            x_prev: 0.0,
            y_prev: 0.0,
        }
    }
}

impl TineDispersionAllpass {
    pub fn new(coefficient: f32) -> Self {
        Self {
            coefficient: coefficient.clamp(-0.99, 0.99),
            x_prev: 0.0,
            y_prev: 0.0,
        }
    }

    pub fn set_coefficient(&mut self, coef: f32) {
        self.coefficient = coef.clamp(-0.99, 0.99);
    }

    #[inline]
    pub fn process(&mut self, input: f32) -> f32 {
        let output = self.coefficient * input + self.x_prev - self.coefficient * self.y_prev;
        self.x_prev = input;
        self.y_prev = output;
        output
    }

    pub fn reset(&mut self) {
        self.x_prev = 0.0;
        self.y_prev = 0.0;
    }
}

/// Cascade of 4 allpass filter sections providing cantilever beam dispersion.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DispersionAllpassChain {
    pub stages: [TineDispersionAllpass; MAX_DISPERSION_STAGES],
    pub stiffness: f32,
}

impl Default for DispersionAllpassChain {
    fn default() -> Self {
        Self {
            stages: [TineDispersionAllpass::default(); MAX_DISPERSION_STAGES],
            stiffness: 0.35,
        }
    }
}

impl DispersionAllpassChain {
    pub fn new(stiffness: f32) -> Self {
        let mut chain = Self::default();
        chain.set_stiffness(stiffness);
        chain
    }

    pub fn set_stiffness(&mut self, stiffness: f32) {
        self.stiffness = stiffness.clamp(0.0, 1.0);
        let coef = self.stiffness * 0.65;
        for (i, stage) in self.stages.iter_mut().enumerate() {
            // Slight coefficient spread per stage for natural mode dispersion
            let stage_coef = (coef * (1.0 + i as f32 * 0.05)).min(0.95);
            stage.set_coefficient(stage_coef);
        }
    }

    #[inline]
    pub fn process(&mut self, input: f32) -> f32 {
        let mut sig = input;
        for stage in self.stages.iter_mut() {
            sig = stage.process(sig);
        }
        sig
    }

    pub fn reset(&mut self) {
        for stage in self.stages.iter_mut() {
            stage.reset();
        }
    }
}

/// Resonant tonebar energy coupling and damping filter.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TonebarCoupling {
    /// Tonebar mass coupling stiffness $[0.0 ..= 1.0]$.
    pub coupling_stiffness: f32,
    /// Tonebar $Q$ / resonance factor.
    pub tonebar_q: f32,
    /// Tonebar state variables.
    y1: f32,
    y2: f32,
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
}

impl Default for TonebarCoupling {
    fn default() -> Self {
        Self {
            coupling_stiffness: 0.65,
            tonebar_q: 45.0,
            y1: 0.0,
            y2: 0.0,
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
        }
    }
}

impl TonebarCoupling {
    pub fn new(fundamental_hz: f32, sample_rate: u32, coupling_stiffness: f32) -> Self {
        let mut tonebar = Self {
            coupling_stiffness: coupling_stiffness.clamp(0.0, 1.0),
            ..Default::default()
        };
        tonebar.recompute_coefficients(fundamental_hz, sample_rate);
        tonebar
    }

    pub fn recompute_coefficients(&mut self, fundamental_hz: f32, sample_rate: u32) {
        let sr = sample_rate as f32;
        let w0 = (2.0 * PI * fundamental_hz / sr).clamp(0.0001, PI - 0.01);
        let q = self.tonebar_q * (1.0 + self.coupling_stiffness * 0.5);
        let alpha = (w0.sin() / (2.0 * q)).clamp(0.00001, 1.0);

        let cos_w0 = w0.cos();
        let a0 = 1.0 + alpha;

        // Bandpass filter centered at fundamental
        self.b0 = (alpha * self.coupling_stiffness * 2.0) / a0;
        self.b1 = 0.0;
        self.b2 = (-alpha * self.coupling_stiffness * 2.0) / a0;
        self.a1 = (-2.0 * cos_w0) / a0;
        self.a2 = (1.0 - alpha) / a0;
    }

    #[inline]
    pub fn process(&mut self, input: f32) -> f32 {
        let out = self.b0 * input + self.b1 * self.y1 + self.b2 * self.y2 - self.a1 * self.y1 - self.a2 * self.y2;
        self.y2 = self.y1;
        self.y1 = out;
        out
    }

    pub fn reset(&mut self) {
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
}

/// Damper clunk and felt release transient mechanics.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DamperClunk {
    /// Clunk volume level $[0.0 ..= 1.0]$.
    pub clunk_volume: f32,
    /// Clunk decay envelope.
    clunk_env: f32,
    clunk_decay: f32,
    clunk_phase: f32,
    clunk_freq: f32,
}

impl Default for DamperClunk {
    fn default() -> Self {
        Self {
            clunk_volume: 0.35,
            clunk_env: 0.0,
            clunk_decay: 0.92,
            clunk_phase: 0.0,
            clunk_freq: 280.0,
        }
    }
}

impl DamperClunk {
    pub fn new(clunk_volume: f32) -> Self {
        Self {
            clunk_volume: clunk_volume.clamp(0.0, 1.0),
            clunk_env: 0.0,
            clunk_decay: 0.92,
            clunk_phase: 0.0,
            clunk_freq: 280.0,
        }
    }

    /// Triggers damper release clunk transient.
    pub fn trigger_release(&mut self, fundamental_hz: f32, sample_rate: u32) {
        self.clunk_env = 1.0;
        // Damper thud frequency scales gently with note register
        self.clunk_freq = (180.0 + fundamental_hz * 0.25).clamp(120.0, 600.0);
        let decay_time_ms = 25.0;
        let decay_samples = (decay_time_ms * 0.001 * sample_rate as f32).max(1.0);
        self.clunk_decay = (-3.0 / decay_samples).exp();
    }

    #[inline]
    pub fn process_sample(&mut self, sample_rate: u32) -> f32 {
        if self.clunk_env < 0.0001 {
            return 0.0;
        }

        let dt = 2.0 * PI * self.clunk_freq / sample_rate as f32;
        self.clunk_phase = (self.clunk_phase + dt) % (2.0 * PI);

        let noise = ((self.clunk_phase * 12.3456).sin() * 0.5 + 0.5) * 0.3;
        let tone = self.clunk_phase.sin() * 0.7;
        let sample = (tone + noise) * self.clunk_env * self.clunk_volume * 0.5;

        self.clunk_env *= self.clunk_decay;
        sample
    }

    pub fn reset(&mut self) {
        self.clunk_env = 0.0;
        self.clunk_phase = 0.0;
    }
}

/// Non-linear electromagnetic inductive pickup transfer function.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct InductivePickup {
    /// Air-gap distance in mm $[0.5 ..= 5.0\text{mm}]$.
    pub air_gap_mm: f32,
    /// Vertical alignment offset in mm $[-2.0 ..= 2.0\text{mm}]$ relative to pole piece center.
    pub alignment_offset_mm: f32,
    /// Barking saturation drive $[0.0 ..= 1.0]$.
    pub bark_drive: f32,
    /// Previous displacement for numerical derivative $dy/dt$.
    y_prev: f32,
}

impl Default for InductivePickup {
    fn default() -> Self {
        Self {
            air_gap_mm: 1.8,
            alignment_offset_mm: 0.45, // Slight asymmetric offset for classic Rhodes even harmonics
            bark_drive: 0.5,
            y_prev: 0.0,
        }
    }
}

impl InductivePickup {
    pub fn new(air_gap_mm: f32, alignment_offset_mm: f32, bark_drive: f32) -> Self {
        Self {
            air_gap_mm: air_gap_mm.clamp(0.4, 6.0),
            alignment_offset_mm: alignment_offset_mm.clamp(-3.0, 3.0),
            bark_drive: bark_drive.clamp(0.0, 1.0),
            y_prev: 0.0,
        }
    }

    /// Evaluates non-linear magnetic flux derivative and asymmetric barking saturation.
    #[inline]
    pub fn process_displacement(&mut self, displacement: f32, sample_rate: u32) -> f32 {
        let sr = sample_rate as f32;
        let velocity = (displacement - self.y_prev) * sr * 0.0001;
        self.y_prev = displacement;

        // Relative distance from pole piece: r = sqrt(d^2 + (y - h)^2)
        let d = self.air_gap_mm;
        let y_rel = displacement * 1.5 - self.alignment_offset_mm;
        let r_sq = (d * d + y_rel * y_rel).max(0.05);

        // Magnetic flux gradient dPhi/dy proportional to (y - h) / (d^2 + (y-h)^2)^(5/2)
        let flux_grad = y_rel / (r_sq * r_sq.sqrt());

        // Induced EMF V = -N * dPhi/dt = -N * (dPhi/dy) * (dy/dt)
        let raw_voltage = flux_grad * velocity * 12.0;

        // Non-linear asymmetric saturation ("barking") at high strike displacements
        let drive = 1.0 + self.bark_drive * 3.5;
        let scaled_v = raw_voltage * drive;

        // Asymmetric soft-clipping saturation function
        let pos_sat = if scaled_v > 0.0 {
            scaled_v / (1.0 + scaled_v.abs())
        } else {
            // Slight compression on negative swing modeling magnetic reluctance
            let neg_v = scaled_v * 1.15;
            neg_v / (1.0 + neg_v.abs())
        };

        pos_sat.clamp(-1.5, 1.5)
    }

    pub fn reset(&mut self) {
        self.y_prev = 0.0;
    }
}

/// A complete physical electromechanical tine resonator voice channel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TineResonator {
    /// Resonator model type.
    pub model: ElectricPianoModel,
    /// Fundamental frequency in Hz.
    pub frequency: f32,
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Delay buffer for traveling wave / cantilever vibration.
    #[serde(skip, default = "default_tine_buffer")]
    delay_buffer: [f32; MAX_TINE_DELAY],
    write_pos: usize,
    delay_len: f32,
    /// Hammer compliance model.
    pub hammer: HammerModel,
    /// Cantilever beam inharmonic dispersion filter chain.
    pub dispersion: DispersionAllpassChain,
    /// Resonant tonebar coupling.
    pub tonebar: TonebarCoupling,
    /// Non-linear inductive pickup.
    pub pickup: InductivePickup,
    /// Damper clunk generator.
    pub damper: DamperClunk,
    /// Loop decay feedback coefficient.
    pub feedback_gain: f32,
    /// High-frequency damping 1-pole filter state.
    damping_state: f32,
    pub damping_coef: f32,
    /// Key state.
    pub is_active: bool,
    pub key_held: bool,
    pub note_number: u8,
}

impl Default for TineResonator {
    fn default() -> Self {
        Self::new(ElectricPianoModel::RhodesTine, 261.63, 48000)
    }
}

impl TineResonator {
    /// Creates a new physical tine resonator instance.
    pub fn new(model: ElectricPianoModel, frequency: f32, sample_rate: u32) -> Self {
        let mut resonator = Self {
            model,
            frequency: frequency.clamp(20.0, 5000.0),
            sample_rate: sample_rate.max(8000),
            delay_buffer: [0.0; MAX_TINE_DELAY],
            write_pos: 0,
            delay_len: 100.0,
            hammer: HammerModel::default(),
            dispersion: DispersionAllpassChain::new(0.35),
            tonebar: TonebarCoupling::new(frequency, sample_rate, 0.65),
            pickup: InductivePickup::default(),
            damper: DamperClunk::default(),
            feedback_gain: 0.996,
            damping_state: 0.0,
            damping_coef: 0.25,
            is_active: false,
            key_held: false,
            note_number: 60,
        };

        resonator.set_frequency(frequency);
        resonator.configure_model(model);
        resonator
    }

    /// Configures default physical attributes according to the model type.
    pub fn configure_model(&mut self, model: ElectricPianoModel) {
        self.model = model;
        match model {
            ElectricPianoModel::RhodesTine => {
                self.dispersion.set_stiffness(0.42);
                self.tonebar.coupling_stiffness = 0.70;
                self.pickup.air_gap_mm = 1.8;
                self.pickup.alignment_offset_mm = 0.45;
                self.pickup.bark_drive = 0.55;
                self.damping_coef = 0.18;
                self.feedback_gain = 0.997;
            }
            ElectricPianoModel::WurlitzerReed => {
                self.dispersion.set_stiffness(0.18);
                self.tonebar.coupling_stiffness = 0.35; // Lighter resonator plate
                self.pickup.air_gap_mm = 1.2;
                self.pickup.alignment_offset_mm = 0.15;
                self.pickup.bark_drive = 0.75; // More aggressive midrange reed bark
                self.damping_coef = 0.32;
                self.feedback_gain = 0.993; // Shorter natural sustain
            }
        }
        self.tonebar.recompute_coefficients(self.frequency, self.sample_rate);
    }

    /// Sets the fundamental frequency in Hz and updates delay line length.
    pub fn set_frequency(&mut self, freq: f32) {
        self.frequency = freq.clamp(20.0, 5000.0);
        let period_samples = self.sample_rate as f32 / self.frequency;
        // Compensate for allpass filter group delay (~4 samples) and damping filter (~1 sample)
        let delay = (period_samples - 5.0).clamp(2.0, (MAX_TINE_DELAY - 1) as f32);
        self.delay_len = delay;
        self.tonebar.recompute_coefficients(self.frequency, self.sample_rate);
    }

    /// Triggers a note strike with velocity and hammer hardness.
    pub fn note_on(&mut self, note: u8, freq: f32, velocity: f32, hardness: f32) {
        self.note_number = note;
        self.set_frequency(freq);
        self.is_active = true;
        self.key_held = true;
        self.hammer.strike(velocity, hardness, self.sample_rate);
    }

    /// Releases the note, engaging the felt damper.
    pub fn note_off(&mut self) {
        if self.key_held {
            self.key_held = false;
            self.damper.trigger_release(self.frequency, self.sample_rate);
        }
    }

    /// Processes one sample through the physical electromechanical tine resonator.
    #[inline]
    pub fn process_sample(&mut self) -> f32 {
        if !self.is_active {
            return 0.0;
        }

        // Read from delay line with linear interpolation
        let read_pos = (self.write_pos as f32 - self.delay_len + MAX_TINE_DELAY as f32) % (MAX_TINE_DELAY as f32);
        let read_idx0 = read_pos.floor() as usize % MAX_TINE_DELAY;
        let read_idx1 = (read_idx0 + 1) % MAX_TINE_DELAY;
        let frac = read_pos.fract();

        let raw_delayed = self.delay_buffer[read_idx0] * (1.0 - frac) + self.delay_buffer[read_idx1] * frac;

        // Dispersion allpass cascade for cantilever beam inharmonicity
        let dispersed = self.dispersion.process(raw_delayed);

        // High-frequency damping filter (1-pole lowpass)
        let effective_damping = if self.key_held {
            self.damping_coef
        } else {
            // Rapid felt damping on release
            0.85
        };
        self.damping_state += effective_damping * (dispersed - self.damping_state);
        let damped = self.damping_state;

        // Tonebar resonant coupling
        let tonebar_res = self.tonebar.process(damped);
        let combined_vibration = damped * 0.55 + tonebar_res * 0.45;

        // Hammer excitation force injection
        let hammer_force = self.hammer.compute_force(combined_vibration);

        // Feedback loop calculation
        let loop_gain = if self.key_held {
            self.feedback_gain
        } else {
            // Felt damper active
            0.70
        };

        let new_tine_sample = combined_vibration * loop_gain + hammer_force;
        self.delay_buffer[self.write_pos] = new_tine_sample.clamp(-4.0, 4.0);
        self.write_pos = (self.write_pos + 1) % MAX_TINE_DELAY;

        // Inductive pickup electromagnetic conversion & asymmetric bark overdrive
        let pickup_audio = self.pickup.process_displacement(new_tine_sample, self.sample_rate);

        // Damper mechanical clunk thud
        let clunk_audio = self.damper.process_sample(self.sample_rate);

        let final_audio = pickup_audio + clunk_audio;

        // Voice deactivation check
        if !self.key_held && new_tine_sample.abs() < 1e-5 && self.damper.clunk_env < 1e-4 {
            self.is_active = false;
        }

        final_audio
    }

    /// Resets all delay lines, filters, and state variables to silence.
    pub fn reset(&mut self) {
        self.delay_buffer.fill(0.0);
        self.write_pos = 0;
        self.dispersion.reset();
        self.tonebar.reset();
        self.pickup.reset();
        self.damper.reset();
        self.damping_state = 0.0;
        self.is_active = false;
        self.key_held = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::allocator::AllocGuard;

    #[test]
    fn test_tine_resonator_zero_allocation() {
        let mut tine = TineResonator::new(ElectricPianoModel::RhodesTine, 440.0, 48000);
        tine.note_on(69, 440.0, 0.85, 0.5);

        {
            let _guard = AllocGuard::new();
            for _ in 0..1024 {
                let sample = tine.process_sample();
                assert!(sample.is_finite());
            }
        }
    }

    #[test]
    fn test_hammer_force_compliance() {
        let mut hammer = HammerModel::default();
        hammer.strike(0.9, 0.6, 48000);
        assert!(hammer.contact_samples > 0);

        let force1 = hammer.compute_force(0.0);
        assert!(force1 >= 0.0);
    }

    #[test]
    fn test_inductive_pickup_barking() {
        let mut pickup = InductivePickup::default();
        let s1 = pickup.process_displacement(0.1, 48000);
        let s2 = pickup.process_displacement(0.9, 48000);
        assert!(s1.is_finite());
        assert!(s2.is_finite());
    }

    #[test]
    fn test_damper_clunk_release() {
        let mut damper = DamperClunk::new(0.5);
        damper.trigger_release(261.63, 48000);
        let out = damper.process_sample(48000);
        assert!(out.is_finite());
    }
}
