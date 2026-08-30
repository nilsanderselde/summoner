// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Asian Zither (Koto & Guzheng) Synthesis Engine (Milestone 29).
//!
//! Provides a high-precision physical acoustic synthesis engine for Japanese Koto (13-string)
//! and Chinese Guzheng (21-string) featuring:
//! - Movable triangular wooden *ji* bridges with non-linear termination boundary impedance
//! - Dual-subsegment string coupling (plucked playing segment vs unplayed behind-the-bridge sympathetic segment)
//! - Non-linear ivory/tortoiseshell *tsume* plectrum slip dynamics with snap release transients
//! - Frequency-dependent silk/nylon internal damping and dispersion allpass filters
//! - Authentic Paulownia wood (*kiri*) soundboard body cavity modal resonator
//! - Dynamic *oshi-ite* left-hand string press tension pitch deflection (up to +400 cents)
//! - Interleaved stereo spatialization across the 13 string positions
//!
//! Enforces zero heap allocations in audio render loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};
use crate::ji_bridge::{
    oshi_ite_pitch_multiplier, JiBridgeJunction, JiBridgeProfile, KotoTuningSchema,
    PaulowniaSoundboardBody, NUM_KOTO_STRINGS,
};
use crate::traits::SignalProcessor;

/// Maximum delay buffer capacity in samples (~20 Hz fundamental at 192 kHz).
pub const MAX_KOTO_DELAY: usize = 16384;

/// Reference root fundamental frequency (Sho / Note 1: D4 = 293.665 Hz).
pub const DEFAULT_KOTO_ROOT_HZ: f32 = 293.665;

fn default_koto_delay_buffer() -> Box<[f32; MAX_KOTO_DELAY]> {
    Box::new([0.0; MAX_KOTO_DELAY])
}

/// Koto instrument sound preset profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum KotoProfile {
    /// Traditional 13-String Koto with pure silk strings and Paulownia body (Hirajoshi tuning).
    #[default]
    HirajoshiTraditionalSilk,
    /// Miyako-bushi Urban Classical Koto (Kokin-joshi tuning, bright ivory tsume).
    KokinJoshiUrbanClassical,
    /// Dramatic Contemporary Koto (In-sen tuning, strong oshi-ite pitch bends).
    InSenContemporaryDramatic,
    /// Lyrical Spring Rain Koto (Kumoi-joshi tuning, soft mellow bamboo bridge).
    KumoiJoshiSpringRain,
    /// Okinawan Festive Koto (Ryukyu pentatonic scale, lively glissandi).
    RyukyuFestiveOkinawa,
    /// 21-String Concert Guzheng (Major pentatonic, rich behind-the-bridge sympathetic ringing).
    ConcertGuzhengVirtuoso,
}

impl KotoProfile {
    pub fn name(&self) -> &'static str {
        match self {
            Self::HirajoshiTraditionalSilk => "Hirajoshi Traditional Silk Koto",
            Self::KokinJoshiUrbanClassical => "Kokin-joshi Urban Classical Koto",
            Self::InSenContemporaryDramatic => "In-sen Contemporary Dramatic Koto",
            Self::KumoiJoshiSpringRain => "Kumoi-joshi Spring Rain Koto",
            Self::RyukyuFestiveOkinawa => "Ryukyu Festive Okinawan Koto",
            Self::ConcertGuzhengVirtuoso => "Concert Guzheng Virtuoso (21-String)",
        }
    }

    pub fn tuning_schema(&self) -> KotoTuningSchema {
        match self {
            Self::HirajoshiTraditionalSilk => KotoTuningSchema::Hirajoshi,
            Self::KokinJoshiUrbanClassical => KotoTuningSchema::KokinJoshi,
            Self::InSenContemporaryDramatic => KotoTuningSchema::InSen,
            Self::KumoiJoshiSpringRain => KotoTuningSchema::KumoiJoshi,
            Self::RyukyuFestiveOkinawa => KotoTuningSchema::Ryukyu,
            Self::ConcertGuzhengVirtuoso => KotoTuningSchema::GuzhengPentatonic,
        }
    }

    pub fn bridge_profile(&self) -> JiBridgeProfile {
        match self {
            Self::HirajoshiTraditionalSilk => JiBridgeProfile::PaulowniaHardwood,
            Self::KokinJoshiUrbanClassical => JiBridgeProfile::IvoryBone,
            Self::InSenContemporaryDramatic => JiBridgeProfile::SyntheticPlastic,
            Self::KumoiJoshiSpringRain => JiBridgeProfile::SmokedBamboo,
            Self::RyukyuFestiveOkinawa => JiBridgeProfile::PaulowniaHardwood,
            Self::ConcertGuzhengVirtuoso => JiBridgeProfile::RosewoodGuzheng,
        }
    }

    /// Returns nominal $(T_{60\text{decay}}, \text{decay\_damping}, \text{behind\_bridge\_bleed}, \text{body\_gain})$.
    pub fn nominal_physics(&self) -> (f32, f32, f32, f32) {
        match self {
            Self::HirajoshiTraditionalSilk => (4.2, 0.992, 0.25, 0.85),
            Self::KokinJoshiUrbanClassical => (3.8, 0.990, 0.20, 0.80),
            Self::InSenContemporaryDramatic => (5.0, 0.994, 0.30, 0.90),
            Self::KumoiJoshiSpringRain => (3.5, 0.988, 0.22, 0.75),
            Self::RyukyuFestiveOkinawa => (4.0, 0.991, 0.20, 0.82),
            Self::ConcertGuzhengVirtuoso => (6.5, 0.996, 0.40, 0.95),
        }
    }
}

/// A first-order allpass dispersion section for string inharmonicity.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct KotoDispersionFilter {
    pub coefficient: f32,
    x_prev: f32,
    y_prev: f32,
}

impl KotoDispersionFilter {
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

/// A single physical modeling Koto string waveguide voice with dual playing and behind-the-bridge delay segments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KotoStringVoice {
    pub string_index: usize,
    pub nominal_freq_hz: f32,
    pub active_freq_hz: f32,
    pub oshi_ite_force_n: f32,
    pub hiki_iro_release: f32,
    pub is_sounding: bool,
    pub amplitude_envelope: f32,
    pub pan_position: f32, // [-1.0 = left, +1.0 = right]

    // Playing segment delay line
    #[serde(skip, default = "default_koto_delay_buffer")]
    playing_buffer: Box<[f32; MAX_KOTO_DELAY]>,
    playing_write_pos: usize,
    playing_delay_samples: f32,

    // Behind-the-bridge sympathetic delay line
    #[serde(skip, default = "default_koto_delay_buffer")]
    behind_buffer: Box<[f32; MAX_KOTO_DELAY]>,
    behind_write_pos: usize,
    behind_delay_samples: f32,

    // Boundary scattering junction
    pub ji_junction: JiBridgeJunction,

    // Inharmonic dispersion filter
    pub dispersion: KotoDispersionFilter,

    // Loop damping filter state
    loop_damping: f32,
    filter_prev: f32,

    // Tsume plectrum excitation state
    pluck_phase: f32,
    pluck_duration: f32,
    pluck_velocity: f32,
    pluck_active: bool,
}

impl KotoStringVoice {
    pub fn new(string_index: usize, nominal_freq_hz: f32, sample_rate: u32, bridge_profile: JiBridgeProfile, bridge_pos: f32) -> Self {
        // Pan position spread from left (-0.7) for string 1 (bass) to right (+0.7) for string 13 (treble)
        let pan = -0.7 + 1.4 * (string_index as f32 / (NUM_KOTO_STRINGS.max(2) - 1) as f32);

        let mut voice = Self {
            string_index,
            nominal_freq_hz,
            active_freq_hz: nominal_freq_hz,
            oshi_ite_force_n: 0.0,
            hiki_iro_release: 0.0,
            is_sounding: false,
            amplitude_envelope: 0.0,
            pan_position: pan.clamp(-1.0, 1.0),
            playing_buffer: default_koto_delay_buffer(),
            playing_write_pos: 0,
            playing_delay_samples: 100.0,
            behind_buffer: default_koto_delay_buffer(),
            behind_write_pos: 0,
            behind_delay_samples: 50.0,
            ji_junction: JiBridgeJunction::new(bridge_profile, bridge_pos),
            dispersion: KotoDispersionFilter::new(-0.15),
            loop_damping: 0.992,
            filter_prev: 0.0,
            pluck_phase: 0.0,
            pluck_duration: 0.002, // 2 ms snap
            pluck_velocity: 0.0,
            pluck_active: false,
        };
        voice.recalculate_delays(sample_rate);
        voice
    }

    pub fn recalculate_delays(&mut self, sample_rate: u32) {
        let sr = sample_rate as f32;
        let pitch_mult = oshi_ite_pitch_multiplier(self.oshi_ite_force_n) * (1.0 - 0.15 * self.hiki_iro_release);
        let eff_freq = (self.nominal_freq_hz * pitch_mult).clamp(20.0, 10000.0);
        self.active_freq_hz = eff_freq;

        let total_period = sr / eff_freq;
        let pos = self.ji_junction.position_fraction;

        let play_len = (total_period * pos).clamp(4.0, (MAX_KOTO_DELAY - 4) as f32);
        let behind_len = (total_period * (1.0 - pos) * 0.9).clamp(4.0, (MAX_KOTO_DELAY - 4) as f32);

        self.playing_delay_samples = play_len;
        self.behind_delay_samples = behind_len;
    }

    pub fn pluck(&mut self, velocity: f32, sample_rate: u32) {
        self.pluck_velocity = velocity.clamp(0.01, 1.0);
        self.pluck_phase = 0.0;
        self.pluck_duration = (0.003 - 0.0015 * velocity).max(0.0005) * sample_rate as f32;
        self.pluck_active = true;
        self.is_sounding = true;
        self.amplitude_envelope = velocity;
    }

    pub fn set_oshi_ite(&mut self, force_n: f32, sample_rate: u32) {
        self.oshi_ite_force_n = force_n.clamp(0.0, 30.0);
        self.recalculate_delays(sample_rate);
    }

    pub fn set_hiki_iro(&mut self, release: f32, sample_rate: u32) {
        self.hiki_iro_release = release.clamp(0.0, 1.0);
        self.recalculate_delays(sample_rate);
    }

    #[inline]
    fn read_interpolated(buf: &[f32; MAX_KOTO_DELAY], write_pos: usize, delay: f32) -> f32 {
        let read_pos = (write_pos as f32 + MAX_KOTO_DELAY as f32 - delay) % MAX_KOTO_DELAY as f32;
        let idx0 = read_pos.floor() as usize % MAX_KOTO_DELAY;
        let idx1 = (idx0 + 1) % MAX_KOTO_DELAY;
        let frac = read_pos - read_pos.floor();
        buf[idx0] * (1.0 - frac) + buf[idx1] * frac
    }

    /// Process a single audio sample step for this string.
    /// Returns `(playing_sample, body_excitation)`.
    #[inline]
    pub fn process_sample(&mut self, sympathetic_excitation: f32) -> (f32, f32) {
        if !self.is_sounding && !self.pluck_active && sympathetic_excitation.abs() < 1e-6 {
            return (0.0, 0.0);
        }

        // Tsume pick snap excitation
        let mut pluck_impulse = 0.0;
        if self.pluck_active {
            self.pluck_phase += 1.0;
            let norm_t = self.pluck_phase / self.pluck_duration;
            if norm_t <= 1.0 {
                // Non-linear tsume slip curve: sharp rise, exponential snap release
                let snap = (norm_t * std::f32::consts::PI).sin();
                pluck_impulse = self.pluck_velocity * snap * 1.5;
            } else {
                self.pluck_active = false;
            }
        }

        // Read forward propagating waves from delay lines
        let incident_playing = Self::read_interpolated(&self.playing_buffer, self.playing_write_pos, self.playing_delay_samples);
        let incident_behind = Self::read_interpolated(&self.behind_buffer, self.behind_write_pos, self.behind_delay_samples);

        // Process boundary scattering at Ji bridge
        let (refl_playing, trans_behind, body_exc) = self.ji_junction.process_scattering(incident_playing, incident_behind);

        // Apply inharmonic dispersion filter
        let dispersed_playing = self.dispersion.step(refl_playing);

        // Apply frequency-dependent loop damping
        let damped_playing = 0.7 * dispersed_playing + 0.3 * self.filter_prev;
        self.filter_prev = dispersed_playing;
        let next_playing = (damped_playing * self.loop_damping + pluck_impulse + sympathetic_excitation * 0.05).clamp(-2.0, 2.0);

        // Write back into delay lines
        self.playing_buffer[self.playing_write_pos] = next_playing;
        self.playing_write_pos = (self.playing_write_pos + 1) % MAX_KOTO_DELAY;

        let next_behind = (trans_behind * self.loop_damping * 0.98).clamp(-2.0, 2.0);
        self.behind_buffer[self.behind_write_pos] = next_behind;
        self.behind_write_pos = (self.behind_write_pos + 1) % MAX_KOTO_DELAY;

        let output_sample = next_playing + next_behind * 0.3;

        // Check if string energy has decayed
        self.amplitude_envelope = 0.9995 * self.amplitude_envelope + 0.0005 * output_sample.abs();
        if self.amplitude_envelope < 1e-5 && !self.pluck_active {
            self.is_sounding = false;
        }

        (output_sample, body_exc)
    }

    pub fn reset(&mut self) {
        self.playing_buffer.fill(0.0);
        self.behind_buffer.fill(0.0);
        self.playing_write_pos = 0;
        self.behind_write_pos = 0;
        self.filter_prev = 0.0;
        self.dispersion.reset();
        self.ji_junction.reset();
        self.pluck_active = false;
        self.is_sounding = false;
        self.amplitude_envelope = 0.0;
    }
}

/// Asian Zither Physical Acoustic Synthesizer (13-String Koto & 21-String Guzheng).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Koto {
    pub profile: KotoProfile,
    pub tuning_schema: KotoTuningSchema,
    pub root_freq_hz: f32,
    pub sample_rate: u32,
    pub master_gain: f32,
    pub body_coupling_gain: f32,
    pub behind_bridge_bleed: f32,

    /// 13 physical modeling string voices.
    pub strings: Vec<KotoStringVoice>,

    /// Paulownia soundboard body cavity modal resonator.
    pub body: PaulowniaSoundboardBody,
}

impl Default for Koto {
    fn default() -> Self {
        Self::new(48000)
    }
}

impl Koto {
    pub fn new(sample_rate: u32) -> Self {
        let profile = KotoProfile::HirajoshiTraditionalSilk;
        let schema = profile.tuning_schema();
        let bridge_prof = profile.bridge_profile();
        let (_t60, _damp, bleed, bgain) = profile.nominal_physics();
        let root_hz = DEFAULT_KOTO_ROOT_HZ;

        let ratios = schema.string_ratios();
        let bridge_positions = schema.default_bridge_positions();

        let mut strings = Vec::with_capacity(NUM_KOTO_STRINGS);
        for i in 0..NUM_KOTO_STRINGS {
            let freq = root_hz * ratios[i];
            let voice = KotoStringVoice::new(i, freq, sample_rate, bridge_prof, bridge_positions[i]);
            strings.push(voice);
        }

        Self {
            profile,
            tuning_schema: schema,
            root_freq_hz: root_hz,
            sample_rate,
            master_gain: 0.90,
            body_coupling_gain: bgain,
            behind_bridge_bleed: bleed,
            strings,
            body: PaulowniaSoundboardBody::new(sample_rate),
        }
    }

    /// Set sound and acoustic profile, re-initializing tuning and bridge properties.
    pub fn set_profile(&mut self, profile: KotoProfile) {
        self.profile = profile;
        self.tuning_schema = profile.tuning_schema();
        let bridge_prof = profile.bridge_profile();
        let (_t60, damp, bleed, bgain) = profile.nominal_physics();
        self.behind_bridge_bleed = bleed;
        self.body_coupling_gain = bgain;

        let ratios = self.tuning_schema.string_ratios();
        let bridge_positions = self.tuning_schema.default_bridge_positions();

        for i in 0..NUM_KOTO_STRINGS {
            if i < self.strings.len() {
                self.strings[i].nominal_freq_hz = self.root_freq_hz * ratios[i];
                self.strings[i].loop_damping = damp;
                self.strings[i].ji_junction.set_profile(bridge_prof);
                self.strings[i].ji_junction.position_fraction = bridge_positions[i];
                self.strings[i].recalculate_delays(self.sample_rate);
            }
        }
    }

    /// Set active tuning scale schema.
    pub fn set_tuning_schema(&mut self, schema: KotoTuningSchema) {
        self.tuning_schema = schema;
        let ratios = schema.string_ratios();
        let bridge_positions = schema.default_bridge_positions();

        for i in 0..NUM_KOTO_STRINGS {
            if i < self.strings.len() {
                self.strings[i].nominal_freq_hz = self.root_freq_hz * ratios[i];
                self.strings[i].ji_junction.position_fraction = bridge_positions[i];
                self.strings[i].recalculate_delays(self.sample_rate);
            }
        }
    }

    /// Pluck a string by index (0..12) with velocity in $[0.01 ..= 1.0]$.
    pub fn pluck_string(&mut self, string_idx: usize, velocity: f32) {
        if string_idx < self.strings.len() {
            self.strings[string_idx].pluck(velocity, self.sample_rate);
        }
    }

    /// Apply left-hand Oshi-Ite press force in Newtons $[0.0 ..= 30.0]$ to a string.
    pub fn set_oshi_ite(&mut self, string_idx: usize, force_n: f32) {
        if string_idx < self.strings.len() {
            self.strings[string_idx].set_oshi_ite(force_n, self.sample_rate);
        }
    }

    /// Apply left-hand Hiki-Iro release $[0.0 ..= 1.0]$ to a string.
    pub fn set_hiki_iro(&mut self, string_idx: usize, release: f32) {
        if string_idx < self.strings.len() {
            self.strings[string_idx].set_hiki_iro(release, self.sample_rate);
        }
    }

    /// Adjust movable Ji bridge position offset for a string.
    pub fn set_bridge_offset(&mut self, string_idx: usize, offset: f32) {
        if string_idx < self.strings.len() {
            let base_pos = self.tuning_schema.default_bridge_positions()[string_idx];
            self.strings[string_idx].ji_junction.set_position_offset(base_pos, offset);
            self.strings[string_idx].recalculate_delays(self.sample_rate);
        }
    }

    /// Trigger note by MIDI pitch number. Maps to the closest string or wraps to 13 strings.
    pub fn note_on(&mut self, midi_note: u8, velocity: f32) {
        // Map MIDI notes: D4 = 62 -> String 1, G3 = 55 -> String 2, A3 = 57 -> String 3, etc.
        let string_idx = match midi_note {
            62 => 0,  // D4
            55 => 1,  // G3
            57 => 2,  // A3
            58 => 3,  // Bb3 / Ab3 / B3
            60 => 3,  // C4
            63 => 5,  // Eb4 / F4 / E4
            65 => 5,  // F4
            67 => 6,  // G4
            68 => 7,  // Ab4
            69 => 7,  // A4
            70 => 8,  // Bb4
            72 => 8,  // C5
            74 => 9,  // D5
            75 => 10, // Eb5
            77 => 10, // F5
            79 => 11, // G5
            80 => 12, // Ab5
            81 => 12, // A5
            _ => (midi_note as usize) % NUM_KOTO_STRINGS,
        };
        self.pluck_string(string_idx, velocity);
    }

    pub fn note_off(&mut self, _midi_note: u8) {
        // Strings ring out naturally as in authentic Asian zithers (no hard damper on key release)
    }

    /// Process a single stereo sample frame. Returns `(left, right)` samples clamped to $[-1.0, 1.0]$.
    #[inline]
    pub fn process_stereo_frame(&mut self) -> (f32, f32) {
        let mut total_left = 0.0f32;
        let mut total_right = 0.0f32;
        let mut total_body_excitation = 0.0f32;

        // 1. Process all 13 strings
        for s in self.strings.iter_mut() {
            let (str_out, body_exc) = s.process_sample(0.0);
            total_body_excitation += body_exc;

            // Pan string across stereo field
            let pan = s.pan_position;
            let left_gain = (0.5 * (1.0 - pan)).clamp(0.0, 1.0);
            let right_gain = (0.5 * (1.0 + pan)).clamp(0.0, 1.0);

            total_left += str_out * left_gain;
            total_right += str_out * right_gain;
        }

        // 2. Process Paulownia soundboard acoustic body
        let body_out = self.body.process(total_body_excitation * self.body_coupling_gain);
        total_left += body_out * 0.5;
        total_right += body_out * 0.5;

        let out_l = (total_left * self.master_gain).clamp(-1.0, 1.0);
        let out_r = (total_right * self.master_gain).clamp(-1.0, 1.0);

        (out_l, out_r)
    }

    pub fn reset(&mut self) {
        for s in self.strings.iter_mut() {
            s.reset();
        }
        self.body.reset();
    }
}

impl SignalProcessor for Koto {
    fn name(&self) -> &str {
        "KotoSynthesizer"
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

impl AudioNode for Koto {
    fn name(&self) -> &str {
        "KotoSynthesizer"
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
    fn test_koto_instantiation_and_pluck() {
        let mut koto = Koto::new(48000);
        assert_eq!(koto.strings.len(), NUM_KOTO_STRINGS);

        koto.pluck_string(0, 0.90);
        let (l, r) = koto.process_stereo_frame();
        assert!(l.is_finite());
        assert!(r.is_finite());
    }

    #[test]
    fn test_koto_zero_allocations_in_audio_loop() {
        let mut koto = Koto::new(48000);
        koto.pluck_string(0, 0.85);
        koto.pluck_string(4, 0.80);
        koto.pluck_string(9, 0.75);

        let _guard = AllocGuard::new();
        for _ in 0..1024 {
            let (l, r) = koto.process_stereo_frame();
            assert!(l.abs() <= 1.0);
            assert!(r.abs() <= 1.0);
        }
    }

    #[test]
    fn test_koto_oshi_ite_pitch_bend() {
        let mut koto = Koto::new(48000);
        let base_freq = koto.strings[0].active_freq_hz;

        koto.set_oshi_ite(0, 20.0);
        let bent_freq = koto.strings[0].active_freq_hz;
        assert!(bent_freq > base_freq, "Oshi-Ite press must increase string frequency");
    }
}
