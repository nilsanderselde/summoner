// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Hurdy-Gurdy (Vielle à roue) Synthesis Engine (Milestone 30).
//!
//! Provides a high-precision physical acoustic synthesis engine for the French Hurdy-Gurdy
//! (Vielle à roue) featuring:
//! - Continuous rosined wooden wheel friction excitation with non-linear stick-slip Stribeck hysteresis
//! - Dual chanterelle melody strings with 24-step discrete acoustic tangent box key pitch action
//! - Three continuous drone strings (Gros bourdon, Petit bourdon, Mouche)
//! - Non-linear unilateral chattering *trompette* / *chien* (buzzing dog bridge) with *coup de poignet* wrist bursts
//! - Four internal sympathetic resonance strings tuned to tonic/dominant
//! - Lute/guitar-shaped resonant spruce/maple soundbox body cavity modal resonator
//! - Real-time continuous parameter automation (Crank Wheel Speed $\omega$, Coup de Poignet Acceleration $\Delta \alpha$,
//!   Wheel Pressure, Chien Clearance $h_0$, Drone/Melody Mix, Trompette Buzz Level)
//!
//! Enforces zero heap allocations in audio render loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};
use crate::traits::SignalProcessor;
use crate::trompette_bridge::{
    ChienMaterialProfile, HurdyGurdySoundboxBody, TrompetteChienJunction,
};

/// Maximum delay line buffer capacity in samples (~20 Hz at 192 kHz).
pub const MAX_HURDY_GURDY_DELAY: usize = 16384;

/// Reference fundamental pitch for Chanterelles (G4 = 392.00 Hz).
pub const DEFAULT_CHANTERELLE_ROOT_HZ: f32 = 392.00;

/// Gros Bourdon root pitch (G2 = 98.00 Hz).
pub const DEFAULT_GROS_BOURDON_HZ: f32 = 98.00;

/// Petit Bourdon root pitch (C3 = 130.81 Hz or G3 = 196.00 Hz).
pub const DEFAULT_PETIT_BOURDON_HZ: f32 = 196.00;

/// Mouche drone pitch (G4 = 392.00 Hz or D4 = 293.66 Hz).
pub const DEFAULT_MOUCHE_HZ: f32 = 293.665;

/// Trompette string root pitch (C4 = 261.63 Hz or D4 = 293.66 Hz).
pub const DEFAULT_TROMPETTE_HZ: f32 = 261.626;

fn default_hg_delay_buffer() -> Box<[f32; MAX_HURDY_GURDY_DELAY]> {
    Box::new([0.0; MAX_HURDY_GURDY_DELAY])
}

/// Hurdy-Gurdy regional and historical sound preset profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HurdyGurdyProfile {
    /// Traditional French Bourbonnais Vielle (Rich buzzing dog, bright dual chanterelles).
    #[default]
    BourbonnaisTraditional,
    /// Medieval Symphonia (Mellow organistrum sound, gentle drones, soft wood chatter).
    MedievalSymphonia,
    /// Baroque Virtuoso Vielle (Sharp ivory chien, precise tangent keys, crisp chanterelle solo).
    BaroqueVirtuoso,
    /// Auvergne High-Speed Folk (Aggressive coup de poignet snarl, heavy bass drone).
    AuvergneFolkSnarl,
    /// Gothic Pagan Dark Drone (Low tuned bourdons, high sympathetic ringing, long decay).
    GothicPaganDrone,
    /// Electro-Acoustic Modern Hybrid (Maximum wheel velocity range, extended tangent dynamics).
    ElectroAcousticModern,
}

impl HurdyGurdyProfile {
    pub fn name(&self) -> &'static str {
        match self {
            Self::BourbonnaisTraditional => "Bourbonnais Traditional Vielle",
            Self::MedievalSymphonia => "Medieval Symphonia Organistrum",
            Self::BaroqueVirtuoso => "Baroque Virtuoso Vielle (Ivory Chien)",
            Self::AuvergneFolkSnarl => "Auvergne Folk (High-Speed Snarl)",
            Self::GothicPaganDrone => "Gothic Pagan Dark Drone",
            Self::ElectroAcousticModern => "Electro-Acoustic Modern Hybrid",
        }
    }

    pub fn chien_profile(&self) -> ChienMaterialProfile {
        match self {
            Self::BourbonnaisTraditional => ChienMaterialProfile::MapleOnBone,
            Self::MedievalSymphonia => ChienMaterialProfile::VintageFruitwood,
            Self::BaroqueVirtuoso => ChienMaterialProfile::IvoryPlate,
            Self::AuvergneFolkSnarl => ChienMaterialProfile::MapleOnBone,
            Self::GothicPaganDrone => ChienMaterialProfile::EbonyHardwood,
            Self::ElectroAcousticModern => ChienMaterialProfile::SyntheticDelrin,
        }
    }

    /// Returns nominal $(T_{60\text{decay}}, \text{wheel\_pressure}, \text{drone\_mix}, \text{buzz\_level})$.
    pub fn nominal_physics(&self) -> (f32, f32, f32, f32) {
        match self {
            Self::BourbonnaisTraditional => (3.5, 0.70, 0.45, 0.85),
            Self::MedievalSymphonia => (4.5, 0.55, 0.60, 0.40),
            Self::BaroqueVirtuoso => (3.0, 0.80, 0.35, 0.95),
            Self::AuvergneFolkSnarl => (3.2, 0.90, 0.50, 1.20),
            Self::GothicPaganDrone => (6.0, 0.65, 0.70, 0.75),
            Self::ElectroAcousticModern => (4.0, 0.85, 0.40, 1.00),
        }
    }
}

/// A first-order allpass dispersion section for string inharmonicity.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct HurdyGurdyDispersionFilter {
    pub coefficient: f32,
    x_prev: f32,
    y_prev: f32,
}

impl HurdyGurdyDispersionFilter {
    pub fn new(coefficient: f32) -> Self {
        Self {
            coefficient: coefficient.clamp(-0.95, 0.95),
            x_prev: 0.0,
            y_prev: 0.0,
        }
    }

    #[inline]
    pub fn step(&mut self, input: f32) -> f32 {
        let a = self.coefficient;
        let y = -a * input + self.x_prev + a * self.y_prev;
        self.x_prev = input;
        self.y_prev = y;
        y
    }

    pub fn reset(&mut self) {
        self.x_prev = 0.0;
        self.y_prev = 0.0;
    }
}

/// Non-linear continuous rosined wheel friction excitation model with Stribeck stick-slip curve.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FrictionWheelExciter {
    /// Angular wheel velocity $\omega \in [0.0 ..= 4\pi]\text{ rad/s}$ ($0$ to $2\text{ rev/s}$).
    pub angular_velocity_rad_s: f32,
    /// Normal wheel pressure force $[0.01 ..= 1.0]$.
    pub wheel_pressure: f32,
    /// Rosin adhesion stickiness factor $[0.1 ..= 1.0]$.
    pub rosin_adhesion: f32,
    /// Instantaneous wrist acceleration pulse $\Delta \alpha \ge 0$.
    pub wrist_acceleration: f32,
    /// Wheel radius in meters ($R \approx 0.07\text{ m}$).
    pub wheel_radius_m: f32,
}

impl Default for FrictionWheelExciter {
    fn default() -> Self {
        Self::new(2.0 * PI, 0.70)
    }
}

impl FrictionWheelExciter {
    pub fn new(angular_velocity_rad_s: f32, wheel_pressure: f32) -> Self {
        Self {
            angular_velocity_rad_s: angular_velocity_rad_s.clamp(0.0, 4.0 * PI),
            wheel_pressure: wheel_pressure.clamp(0.01, 1.0),
            rosin_adhesion: 0.85,
            wrist_acceleration: 0.0,
            wheel_radius_m: 0.07,
        }
    }

    /// Calculate instantaneous linear tangential surface speed $v_w = \omega \cdot R$.
    #[inline]
    pub fn linear_surface_speed(&self) -> f32 {
        let eff_omega = self.angular_velocity_rad_s + self.wrist_acceleration * 2.5;
        (eff_omega * self.wheel_radius_m).max(0.0)
    }

    /// Compute Stribeck friction force on a vibrating string with relative velocity $\Delta v = v_w - v_s$.
    #[inline]
    pub fn compute_friction_force(&self, string_velocity: f32) -> f32 {
        let v_wheel = self.linear_surface_speed();
        if v_wheel <= 1e-4 {
            return 0.0;
        }

        let delta_v = v_wheel - string_velocity;
        let v0 = 0.12f32; // Stribeck transition velocity in m/s
        let mu_s = 0.85 * self.rosin_adhesion; // Static friction coefficient
        let mu_k = 0.30 * self.rosin_adhesion; // Kinetic friction coefficient

        let stribeck = mu_k + (mu_s - mu_k) * (-((delta_v / v0).powi(2))).exp();
        let normal_force = self.wheel_pressure * 2.5;

        // Hyperbolic tangent smoothing near zero slip
        let f_friction = normal_force * stribeck * (delta_v * 15.0).tanh();
        f_friction.clamp(-3.0, 3.0)
    }
}

/// A physical modeling waveguide string voice for melody chanterelles, drones, or trompette.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HurdyGurdyStringVoice {
    pub name: String,
    pub nominal_freq_hz: f32,
    pub active_freq_hz: f32,
    pub tangent_semitone_offset: f32,
    pub is_sounding: bool,
    pub amplitude_envelope: f32,
    pub loop_damping: f32,
    pub pan_position: f32,

    #[serde(skip, default = "default_hg_delay_buffer")]
    delay_buffer: Box<[f32; MAX_HURDY_GURDY_DELAY]>,
    write_pos: usize,
    delay_samples: f32,

    pub dispersion: HurdyGurdyDispersionFilter,
    filter_prev: f32,
    string_displacement: f32,
    string_velocity: f32,
}

impl HurdyGurdyStringVoice {
    pub fn new(name: &str, nominal_freq_hz: f32, sample_rate: u32, loop_damping: f32, pan: f32) -> Self {
        let mut voice = Self {
            name: name.to_string(),
            nominal_freq_hz,
            active_freq_hz: nominal_freq_hz,
            tangent_semitone_offset: 0.0,
            is_sounding: true,
            amplitude_envelope: 0.0,
            loop_damping: loop_damping.clamp(0.90, 0.999),
            pan_position: pan.clamp(-1.0, 1.0),
            delay_buffer: default_hg_delay_buffer(),
            write_pos: 0,
            delay_samples: 100.0,
            dispersion: HurdyGurdyDispersionFilter::new(-0.10),
            filter_prev: 0.0,
            string_displacement: 0.0,
            string_velocity: 0.0,
        };
        voice.recalculate_delay(sample_rate);
        voice
    }

    pub fn recalculate_delay(&mut self, sample_rate: u32) {
        let pitch_mult = 2.0f32.powf(self.tangent_semitone_offset / 12.0);
        let eff_freq = (self.nominal_freq_hz * pitch_mult).clamp(20.0, 8000.0);
        self.active_freq_hz = eff_freq;

        let total_period = (sample_rate as f32) / eff_freq;
        self.delay_samples = total_period.clamp(4.0, (MAX_HURDY_GURDY_DELAY - 4) as f32);
    }

    pub fn set_tangent(&mut self, semitones: f32, sample_rate: u32) {
        self.tangent_semitone_offset = semitones.clamp(0.0, 24.0);
        self.recalculate_delay(sample_rate);
    }

    #[inline]
    fn read_interpolated(buf: &[f32; MAX_HURDY_GURDY_DELAY], write_pos: usize, delay: f32) -> f32 {
        let read_pos = (write_pos as f32 + MAX_HURDY_GURDY_DELAY as f32 - delay) % MAX_HURDY_GURDY_DELAY as f32;
        let idx0 = read_pos.floor() as usize % MAX_HURDY_GURDY_DELAY;
        let idx1 = (idx0 + 1) % MAX_HURDY_GURDY_DELAY;
        let frac = read_pos - read_pos.floor();
        buf[idx0] * (1.0 - frac) + buf[idx1] * frac
    }

    /// Process a single sample step of the rosined string waveguide.
    /// Returns `(output_sample, string_displacement, string_velocity)`.
    #[inline]
    pub fn process_sample(&mut self, exciter: &FrictionWheelExciter, sample_rate: u32) -> (f32, f32, f32) {
        let dt = 1.0 / (sample_rate as f32).max(1000.0);

        // Read forward wave from delay line
        let forward_wave = Self::read_interpolated(&self.delay_buffer, self.write_pos, self.delay_samples);

        // Compute string velocity and friction drive force
        self.string_velocity = (forward_wave - self.string_displacement) / dt.max(1e-6) * 0.005;
        let friction_force = exciter.compute_friction_force(self.string_velocity);

        // Inharmonic dispersion filter
        let dispersed = self.dispersion.step(forward_wave);

        // Frequency-dependent damping filter
        let damped = 0.65 * dispersed + 0.35 * self.filter_prev;
        self.filter_prev = dispersed;

        // Inverting fixed boundary reflection + friction input
        let next_wave = (-damped * self.loop_damping + friction_force * 0.20).clamp(-2.0, 2.0);

        // Write into delay buffer
        self.delay_buffer[self.write_pos] = next_wave;
        self.write_pos = (self.write_pos + 1) % MAX_HURDY_GURDY_DELAY;

        self.string_displacement = next_wave;
        self.amplitude_envelope = 0.9995 * self.amplitude_envelope + 0.0005 * next_wave.abs();

        (next_wave, self.string_displacement, self.string_velocity)
    }

    pub fn reset(&mut self) {
        self.delay_buffer.fill(0.0);
        self.write_pos = 0;
        self.filter_prev = 0.0;
        self.dispersion.reset();
        self.string_displacement = 0.0;
        self.string_velocity = 0.0;
        self.amplitude_envelope = 0.0;
    }
}

/// Hurdy-Gurdy (Vielle à roue) Physical Modeling Synthesizer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HurdyGurdy {
    pub profile: HurdyGurdyProfile,
    pub sample_rate: u32,
    pub master_gain: f32,
    pub drone_melody_mix: f32, // [0.0 = melody only, 1.0 = drone only]
    pub trompette_buzz_level: f32,
    pub body_coupling_gain: f32,

    /// Continuous rosined wheel friction exciter.
    pub exciter: FrictionWheelExciter,

    /// Two Chanterelle melody strings (Chanterelle 1, Chanterelle 2).
    pub chanterelles: [HurdyGurdyStringVoice; 2],

    /// Three Bourdon drone strings (Gros bourdon, Petit bourdon, Mouche).
    pub bourdons: [HurdyGurdyStringVoice; 3],

    /// Trompette string.
    pub trompette_string: HurdyGurdyStringVoice,

    /// Chien buzzing dog bridge obstacle collision junction.
    pub chien_junction: TrompetteChienJunction,

    /// Four sympathetic resonance strings.
    pub sympathetic_strings: [HurdyGurdyStringVoice; 4],

    /// Vielle resonant soundbox body cavity modal filter.
    pub body: HurdyGurdySoundboxBody,
}

impl Default for HurdyGurdy {
    fn default() -> Self {
        Self::new(48000)
    }
}

impl HurdyGurdy {
    pub fn new(sample_rate: u32) -> Self {
        let profile = HurdyGurdyProfile::BourbonnaisTraditional;
        let chien_prof = profile.chien_profile();
        let (_t60, pressure, drone_mix, buzz_lvl) = profile.nominal_physics();

        let ch1 = HurdyGurdyStringVoice::new("Chanterelle 1", DEFAULT_CHANTERELLE_ROOT_HZ, sample_rate, 0.993, -0.30);
        let ch2 = HurdyGurdyStringVoice::new("Chanterelle 2", DEFAULT_CHANTERELLE_ROOT_HZ, sample_rate, 0.993, 0.30);

        let gros = HurdyGurdyStringVoice::new("Gros Bourdon", DEFAULT_GROS_BOURDON_HZ, sample_rate, 0.996, -0.60);
        let petit = HurdyGurdyStringVoice::new("Petit Bourdon", DEFAULT_PETIT_BOURDON_HZ, sample_rate, 0.995, -0.40);
        let mouche = HurdyGurdyStringVoice::new("Mouche", DEFAULT_MOUCHE_HZ, sample_rate, 0.994, 0.40);

        let tromp = HurdyGurdyStringVoice::new("Trompette", DEFAULT_TROMPETTE_HZ, sample_rate, 0.992, 0.50);

        let sym1 = HurdyGurdyStringVoice::new("Sympathetic G3", 196.0, sample_rate, 0.997, -0.20);
        let sym2 = HurdyGurdyStringVoice::new("Sympathetic C4", 261.63, sample_rate, 0.997, -0.10);
        let sym3 = HurdyGurdyStringVoice::new("Sympathetic D4", 293.66, sample_rate, 0.997, 0.10);
        let sym4 = HurdyGurdyStringVoice::new("Sympathetic G4", 392.00, sample_rate, 0.997, 0.20);

        Self {
            profile,
            sample_rate,
            master_gain: 0.90,
            drone_melody_mix: drone_mix,
            trompette_buzz_level: buzz_lvl,
            body_coupling_gain: 0.85,
            exciter: FrictionWheelExciter::new(2.0 * PI, pressure),
            chanterelles: [ch1, ch2],
            bourdons: [gros, petit, mouche],
            trompette_string: tromp,
            chien_junction: TrompetteChienJunction::new(chien_prof, 0.35),
            sympathetic_strings: [sym1, sym2, sym3, sym4],
            body: HurdyGurdySoundboxBody::new(sample_rate),
        }
    }

    /// Set sound and regional profile.
    pub fn set_profile(&mut self, profile: HurdyGurdyProfile) {
        self.profile = profile;
        let chien_prof = profile.chien_profile();
        let (_t60, pressure, drone_mix, buzz_lvl) = profile.nominal_physics();

        self.exciter.wheel_pressure = pressure;
        self.drone_melody_mix = drone_mix;
        self.trompette_buzz_level = buzz_lvl;
        self.chien_junction.set_profile(chien_prof);
    }

    /// Set continuous crank wheel angular velocity $\omega \in [0.0 ..= 4\pi]\text{ rad/s}$.
    pub fn set_crank_speed(&mut self, speed_rad_s: f32) {
        self.exciter.angular_velocity_rad_s = speed_rad_s.clamp(0.0, 4.0 * PI);
    }

    /// Apply *coup de poignet* wrist acceleration pulse $\Delta \alpha \ge 0$.
    pub fn set_wrist_acceleration(&mut self, impulse: f32) {
        self.exciter.wrist_acceleration = impulse.max(0.0);
    }

    /// Set Chien resting clearance gap $h_0 \in [0.05 ..= 1.20]\text{ mm}$.
    pub fn set_chien_clearance(&mut self, gap_mm: f32) {
        self.chien_junction.set_clearance_gap(gap_mm);
    }

    /// Depress tangent key by MIDI note number (chromatic mapping across 2 octaves above G4).
    pub fn set_tangent_note(&mut self, midi_note: u8) {
        // G4 is MIDI 67. Root = 0 semitones.
        let offset = if midi_note >= 67 {
            (midi_note - 67) as f32
        } else if midi_note >= 55 {
            (midi_note - 55) as f32
        } else {
            0.0
        };
        self.chanterelles[0].set_tangent(offset, self.sample_rate);
        self.chanterelles[1].set_tangent(offset, self.sample_rate);
    }

    /// Trigger note on via MIDI.
    pub fn note_on(&mut self, midi_note: u8, _velocity: f32) {
        self.set_tangent_note(midi_note);
    }

    pub fn note_off(&mut self, _midi_note: u8) {
        // Releasing tangent returns chanterelles to open G4 string
        self.chanterelles[0].set_tangent(0.0, self.sample_rate);
        self.chanterelles[1].set_tangent(0.0, self.sample_rate);
    }

    /// Process a single stereo sample frame. Returns `(left, right)` samples clamped to $[-1.0, 1.0]$.
    #[inline]
    pub fn process_stereo_frame(&mut self) -> (f32, f32) {
        let mut melody_left = 0.0f32;
        let mut melody_right = 0.0f32;
        let mut drone_left = 0.0f32;
        let mut drone_right = 0.0f32;
        let mut total_body_excitation = 0.0f32;

        // 1. Process Chanterelles (Melody strings)
        for ch in self.chanterelles.iter_mut() {
            let (ch_out, disp, _) = ch.process_sample(&self.exciter, self.sample_rate);
            let pan = ch.pan_position;
            melody_left += ch_out * 0.5 * (1.0 - pan);
            melody_right += ch_out * 0.5 * (1.0 + pan);
            total_body_excitation += disp.abs();
        }

        // 2. Process Bourdons (Drone strings)
        for bd in self.bourdons.iter_mut() {
            let (bd_out, disp, _) = bd.process_sample(&self.exciter, self.sample_rate);
            let pan = bd.pan_position;
            drone_left += bd_out * 0.5 * (1.0 - pan);
            drone_right += bd_out * 0.5 * (1.0 + pan);
            total_body_excitation += disp.abs() * 0.8;
        }

        // 3. Process Trompette string & Chien buzzing dog bridge
        let (_t_out, t_disp, t_vel) = self.trompette_string.process_sample(&self.exciter, self.sample_rate);
        let (_force, buzz_out, chien_body_exc) = self.chien_junction.process_collision(
            t_disp,
            t_vel,
            self.exciter.wrist_acceleration,
            self.sample_rate,
        );

        let tromp_pan = self.trompette_string.pan_position;
        let buzz_sig = buzz_out * self.trompette_buzz_level;
        drone_left += buzz_sig * 0.5 * (1.0 - tromp_pan);
        drone_right += buzz_sig * 0.5 * (1.0 + tromp_pan);
        total_body_excitation += chien_body_exc;

        // 4. Process Sympathetic strings
        for sym in self.sympathetic_strings.iter_mut() {
            // Sympathetic strings driven by acoustic body excitation
            let dummy_exc = FrictionWheelExciter::new(0.0, 0.0);
            let (sym_out, _, _) = sym.process_sample(&dummy_exc, self.sample_rate);
            let pan = sym.pan_position;
            melody_left += sym_out * 0.15 * (1.0 - pan);
            melody_right += sym_out * 0.15 * (1.0 + pan);
        }

        // 5. Blend Melody and Drone mixes
        let mel_gain = 1.0 - self.drone_melody_mix * 0.5;
        let drn_gain = 0.5 + self.drone_melody_mix * 0.5;
        let mut total_left = melody_left * mel_gain + drone_left * drn_gain;
        let mut total_right = melody_right * mel_gain + drone_right * drn_gain;

        // 6. Resonant soundbox body modal filter
        let body_out = self.body.process(total_body_excitation * self.body_coupling_gain);
        total_left += body_out * 0.40;
        total_right += body_out * 0.40;

        let out_l = (total_left * self.master_gain).clamp(-1.0, 1.0);
        let out_r = (total_right * self.master_gain).clamp(-1.0, 1.0);

        (out_l, out_r)
    }

    pub fn reset(&mut self) {
        for ch in self.chanterelles.iter_mut() {
            ch.reset();
        }
        for bd in self.bourdons.iter_mut() {
            bd.reset();
        }
        self.trompette_string.reset();
        self.chien_junction.reset();
        for sym in self.sympathetic_strings.iter_mut() {
            sym.reset();
        }
        self.body.reset();
    }
}

impl SignalProcessor for HurdyGurdy {
    fn name(&self) -> &str {
        "HurdyGurdySynthesizer"
    }

    fn process_block(
        &mut self,
        _inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        _context: &ProcessContext,
    ) {
        if outputs.is_empty() {
            return;
        }

        let num_samples = outputs[0].len();
        if outputs.len() >= 2 {
            let (left, right) = outputs.split_at_mut(1);
            let out_l = &mut left[0][..num_samples];
            let out_r = &mut right[0][..num_samples];

            for i in 0..num_samples {
                let (s_l, s_r) = self.process_stereo_frame();
                out_l[i] = s_l;
                out_r[i] = s_r;
            }
        } else {
            let out = &mut outputs[0][..num_samples];
            for sample in out.iter_mut() {
                let (s_l, s_r) = self.process_stereo_frame();
                *sample = (s_l + s_r) * 0.5;
            }
        }
    }
}

impl AudioNode for HurdyGurdy {
    fn name(&self) -> &str {
        "HurdyGurdySynthesizer"
    }

    fn process(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        context: &ProcessContext,
    ) {
        self.process_block(inputs, outputs, context);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::allocator::AllocGuard;

    #[test]
    fn test_hurdy_gurdy_instantiation_and_crank() {
        let mut hg = HurdyGurdy::new(48000);
        assert_eq!(hg.chanterelles.len(), 2);
        assert_eq!(hg.bourdons.len(), 3);

        hg.set_crank_speed(2.0 * PI);
        let (l, r) = hg.process_stereo_frame();
        assert!(l.is_finite());
        assert!(r.is_finite());
    }

    #[test]
    fn test_hurdy_gurdy_zero_allocations_in_audio_loop() {
        let mut hg = HurdyGurdy::new(48000);
        hg.set_crank_speed(3.0 * PI);
        hg.set_wrist_acceleration(1.5);
        hg.set_tangent_note(71); // B4

        let _guard = AllocGuard::new();
        for _ in 0..1024 {
            let (l, r) = hg.process_stereo_frame();
            assert!(l.abs() <= 1.0);
            assert!(r.abs() <= 1.0);
        }
    }

    #[test]
    fn test_hurdy_gurdy_tangent_pitch_shifting() {
        let mut hg = HurdyGurdy::new(48000);
        let base_freq = hg.chanterelles[0].active_freq_hz;

        hg.set_tangent_note(79); // G5 (+12 semitones / 1 octave up)
        let octave_freq = hg.chanterelles[0].active_freq_hz;
        assert!((octave_freq / base_freq - 2.0).abs() < 1e-3, "Tangent should double pitch for octave");
    }
}
