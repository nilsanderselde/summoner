// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Electromechanical Clavinet Synthesizer (Milestone 23).
//!
//! Provides a high-precision electromechanical Clavinet D6/E7 physical modeling synthesis engine
//! with rubber anvil key-strike impact compliance, yarn-damper release mechanics, string dispersion
//! allpass filter cascade, dual electromagnetic pickups (Neck/Bridge) with variable polarity and
//! phase cancellation, 4-way rocker tone switch filter bank (Brilliant, Treble, Medium, Soft),
//! and integrated non-linear dynamic state-variable auto-wah envelope follower filter.
//!
//! Enforces zero heap allocations in audio processing loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};
use crate::auto_wah::{AutoWah, AutoWahDirection, AutoWahFilterMode};
use crate::traits::SignalProcessor;

/// Maximum number of polyphonic Clavinet voices.
pub const MAX_CLAV_VOICES: usize = 32;

/// Maximum delay buffer capacity in samples (~20 Hz fundamental at 192 kHz).
pub const MAX_CLAV_DELAY: usize = 16384;

/// Dual magnetic pickup selection modes on classic Clavinet D6/E7.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ClavinetPickupSelection {
    /// Neck pickup only (Single-coil, warm, round, punchy).
    NeckOnly,
    /// Bridge pickup only (Single-coil, bright, twangy, biting).
    BridgeOnly,
    /// Parallel In-Phase (Both pickups in phase, full humbucking warmth).
    #[default]
    ParallelInPhase,
    /// Parallel Out-of-Phase (Destructive magnetic cancellation, hollow funk "quack").
    ParallelOutOfPhase,
}

/// 4-way rocker tone switch filter bank configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClavinetToneSwitches {
    /// Brilliant: High-pass & upper treble peaking shelf.
    pub brilliant: bool,
    /// Treble: Upper-midrange presence peaking filter.
    pub treble: bool,
    /// Medium: Midrange resonance boost / bandpass filter.
    pub medium: bool,
    /// Soft: Gentle high-frequency rolloff / mellow lowpass filter.
    pub soft: bool,
}

impl Default for ClavinetToneSwitches {
    fn default() -> Self {
        Self {
            brilliant: true,
            treble: true,
            medium: false,
            soft: false,
        }
    }
}

impl ClavinetToneSwitches {
    /// Returns 4-bit bitmask encoding.
    pub fn to_bitmask(&self) -> u32 {
        let mut mask = 0u32;
        if self.brilliant { mask |= 1; }
        if self.treble { mask |= 2; }
        if self.medium { mask |= 4; }
        if self.soft { mask |= 8; }
        mask
    }

    /// Decodes from 4-bit bitmask.
    pub fn from_bitmask(mask: u32) -> Self {
        Self {
            brilliant: (mask & 1) != 0,
            treble: (mask & 2) != 0,
            medium: (mask & 4) != 0,
            soft: (mask & 8) != 0,
        }
    }
}

/// Factory sound profiles for Clavinet synthesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ClavinetProfile {
    /// Classic Stevie 1970s Funk with punchy neck/bridge bite and snappy rubber anvil.
    #[default]
    StevieSuperstition,
    /// Deep hollow Funk "Quack" with out-of-phase pickup cancellation and Brilliant filter.
    OutPhaseFunkQuack,
    /// Clean and articulate D6 studio setup with balanced tone and natural yarn decay.
    ClassicD6Clean,
    /// Mellow Chamber Clavinet with Soft tone switch and gentle velvet attack.
    MellowChamber,
    /// Screaming Funk Auto-Wah with dynamic envelope follower sweep and sharp resonance.
    ScreamingAutoWah,
    /// Twangy Bridge Lead with aggressive treble emphasis and fast staccato damping.
    TwangyBridgeLead,
}

impl ClavinetProfile {
    pub fn name(&self) -> &'static str {
        match self {
            Self::StevieSuperstition => "STEVIE SUPERSTITION FUNK",
            Self::OutPhaseFunkQuack => "OUT-OF-PHASE FUNK QUACK",
            Self::ClassicD6Clean => "CLASSIC D6 CLEAN",
            Self::MellowChamber => "MELLOW CHAMBER CLAVINET",
            Self::ScreamingAutoWah => "SCREAMING FUNK AUTO-WAH",
            Self::TwangyBridgeLead => "TWANGY BRIDGE LEAD",
        }
    }

    /// Returns default pickup mode, blend, phase angle, and tone switches.
    pub fn electronic_defaults(&self) -> (ClavinetPickupSelection, f32, f32, ClavinetToneSwitches) {
        match self {
            Self::StevieSuperstition => (
                ClavinetPickupSelection::ParallelInPhase,
                0.50,
                0.0,
                ClavinetToneSwitches { brilliant: true, treble: true, medium: true, soft: false },
            ),
            Self::OutPhaseFunkQuack => (
                ClavinetPickupSelection::ParallelOutOfPhase,
                0.50,
                180.0,
                ClavinetToneSwitches { brilliant: true, treble: true, medium: false, soft: false },
            ),
            Self::ClassicD6Clean => (
                ClavinetPickupSelection::ParallelInPhase,
                0.55,
                0.0,
                ClavinetToneSwitches { brilliant: true, treble: false, medium: true, soft: false },
            ),
            Self::MellowChamber => (
                ClavinetPickupSelection::NeckOnly,
                0.0,
                0.0,
                ClavinetToneSwitches { brilliant: false, treble: false, medium: false, soft: true },
            ),
            Self::ScreamingAutoWah => (
                ClavinetPickupSelection::ParallelOutOfPhase,
                0.50,
                180.0,
                ClavinetToneSwitches { brilliant: true, treble: true, medium: false, soft: false },
            ),
            Self::TwangyBridgeLead => (
                ClavinetPickupSelection::BridgeOnly,
                1.0,
                0.0,
                ClavinetToneSwitches { brilliant: true, treble: true, medium: false, soft: false },
            ),
        }
    }

    /// Returns default (anvil_hardness, yarn_damping, key_strike_velocity).
    pub fn mechanical_defaults(&self) -> (f32, f32, f32) {
        match self {
            Self::StevieSuperstition => (0.85, 0.75, 0.90),
            Self::OutPhaseFunkQuack => (0.90, 0.85, 0.92),
            Self::ClassicD6Clean => (0.70, 0.65, 0.80),
            Self::MellowChamber => (0.45, 0.50, 0.65),
            Self::ScreamingAutoWah => (0.90, 0.80, 0.95),
            Self::TwangyBridgeLead => (0.95, 0.90, 0.88),
        }
    }

    /// Returns default (auto_wah_enabled, sensitivity, base_freq_hz, resonance_q, mix).
    pub fn auto_wah_defaults(&self) -> (bool, f32, f32, f32, f32) {
        match self {
            Self::StevieSuperstition => (false, 0.70, 350.0, 6.0, 0.85),
            Self::OutPhaseFunkQuack => (false, 0.80, 400.0, 7.5, 0.90),
            Self::ClassicD6Clean => (false, 0.60, 300.0, 5.0, 0.75),
            Self::MellowChamber => (false, 0.40, 250.0, 3.5, 0.60),
            Self::ScreamingAutoWah => (true, 0.88, 420.0, 9.5, 0.95),
            Self::TwangyBridgeLead => (false, 0.75, 450.0, 6.0, 0.80),
        }
    }
}

/// Rubber anvil hammer impact compliance model.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ClavinetAnvilModel {
    /// Rubber anvil hardness $[0.0 ..= 1.0]$ (0.0 = soft rubber, 1.0 = hard aged neoprene tip).
    pub hardness: f32,
    /// Strike velocity $[0.0 ..= 1.0]$.
    pub velocity: f32,
    /// Active contact samples remaining.
    pub contact_samples: usize,
    /// Total contact samples.
    pub total_contact_samples: usize,
    /// Initial impact velocity.
    pub impact_vel: f32,
    /// Non-linear Hertzian contact exponent.
    pub exponent: f32,
}

impl Default for ClavinetAnvilModel {
    fn default() -> Self {
        Self {
            hardness: 0.80,
            velocity: 0.85,
            contact_samples: 0,
            total_contact_samples: 16,
            impact_vel: 0.0,
            exponent: 1.6,
        }
    }
}

impl ClavinetAnvilModel {
    /// Triggers an anvil strike against the metal bar.
    pub fn strike(&mut self, velocity: f32, hardness: f32, sample_rate: u32) {
        self.velocity = velocity.clamp(0.0, 1.0);
        self.hardness = hardness.clamp(0.0, 1.0);

        // Clavinet anvil strike is very fast and sharp (0.2 .. 2.0 ms)
        let base_duration_ms = 2.0 - self.hardness * 1.5 - self.velocity * 0.3;
        let duration_ms = base_duration_ms.max(0.15);
        let duration_samples = ((duration_ms * 0.001) * sample_rate as f32).round() as usize;

        self.total_contact_samples = duration_samples.max(3);
        self.contact_samples = self.total_contact_samples;
        self.impact_vel = self.velocity * (1.2 + self.hardness * 1.0);
    }

    /// Computes non-linear impact force for the current sample.
    #[inline]
    pub fn compute_force(&mut self, string_disp: f32) -> f32 {
        if self.contact_samples == 0 {
            return 0.0;
        }

        self.contact_samples -= 1;
        let progress = 1.0 - (self.contact_samples as f32 / self.total_contact_samples as f32);

        // Raised cosine impact pulse
        let pulse = (progress * PI).sin();
        let penetration = (self.impact_vel * pulse - string_disp).max(0.0);

        let stiffness = 4.0 + self.hardness * 12.0;
        let force = stiffness * penetration.powf(self.exponent) * pulse;

        force.clamp(0.0, 6.0)
    }
}

/// Yarn damping and key release transient mechanics.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct YarnDamper {
    /// Yarn wool damping coefficient $[0.0 ..= 1.0]$.
    pub damping_amount: f32,
    /// Release transient thud envelope.
    thud_env: f32,
    thud_decay: f32,
    thud_phase: f32,
    thud_freq: f32,
}

impl Default for YarnDamper {
    fn default() -> Self {
        Self {
            damping_amount: 0.80,
            thud_env: 0.0,
            thud_decay: 0.90,
            thud_phase: 0.0,
            thud_freq: 320.0,
        }
    }
}

impl YarnDamper {
    pub fn new(damping_amount: f32) -> Self {
        Self {
            damping_amount: damping_amount.clamp(0.0, 1.0),
            thud_env: 0.0,
            thud_decay: 0.90,
            thud_phase: 0.0,
            thud_freq: 320.0,
        }
    }

    /// Triggers release damper thud when key is released.
    pub fn trigger_release(&mut self, fundamental_hz: f32, sample_rate: u32) {
        self.thud_env = 1.0;
        self.thud_freq = (fundamental_hz * 0.6 + 150.0).clamp(100.0, 800.0);
        let decay_ms = 18.0 / (0.5 + self.damping_amount);
        let decay_samples = (decay_ms * 0.001 * sample_rate as f32).max(1.0);
        self.thud_decay = (-3.0 / decay_samples).exp();
    }

    #[inline]
    pub fn process_sample(&mut self, sample_rate: u32) -> f32 {
        if self.thud_env < 0.0001 {
            return 0.0;
        }

        let dt = 2.0 * PI * self.thud_freq / sample_rate as f32;
        self.thud_phase = (self.thud_phase + dt) % (2.0 * PI);

        let noise = ((self.thud_phase * 17.123).sin() * 0.5 + 0.5) * 0.35;
        let tone = self.thud_phase.sin() * 0.65;
        let sample = (tone + noise) * self.thud_env * self.damping_amount * 0.4;

        self.thud_env *= self.thud_decay;
        sample
    }

    pub fn reset(&mut self) {
        self.thud_env = 0.0;
        self.thud_phase = 0.0;
    }
}

fn default_clav_buffer() -> [f32; MAX_CLAV_DELAY] {
    [0.0; MAX_CLAV_DELAY]
}

/// A single physical Clavinet voice channel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClavinetVoice {
    /// Fundamental note frequency in Hz.
    pub frequency: f32,
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Delay buffer for traveling waves on the steel string.
    #[serde(skip, default = "default_clav_buffer")]
    delay_buffer: [f32; MAX_CLAV_DELAY],
    write_pos: usize,
    delay_len: f32,
    /// Strike anvil compliance model.
    pub anvil: ClavinetAnvilModel,
    /// Yarn damper.
    pub damper: YarnDamper,
    /// Dispersion allpass state.
    disp_x: f32,
    disp_y: f32,
    pub disp_coef: f32,
    /// Damping 1-pole filter state.
    damping_state: f32,
    pub loop_feedback: f32,
    /// Dual pickup sampling taps along delay line $[0.0 ..= 1.0]$.
    pub neck_tap_ratio: f32,
    pub bridge_tap_ratio: f32,
    /// Voice state.
    pub is_active: bool,
    pub key_held: bool,
    pub note_number: u8,
}

impl Default for ClavinetVoice {
    fn default() -> Self {
        Self::new(261.63, 48000)
    }
}

impl ClavinetVoice {
    /// Creates a new Clavinet voice channel.
    pub fn new(frequency: f32, sample_rate: u32) -> Self {
        let mut voice = Self {
            frequency: frequency.clamp(20.0, 5000.0),
            sample_rate: sample_rate.max(8000),
            delay_buffer: [0.0; MAX_CLAV_DELAY],
            write_pos: 0,
            delay_len: 100.0,
            anvil: ClavinetAnvilModel::default(),
            damper: YarnDamper::default(),
            disp_x: 0.0,
            disp_y: 0.0,
            disp_coef: 0.38,
            damping_state: 0.0,
            loop_feedback: 0.995,
            neck_tap_ratio: 0.22,
            bridge_tap_ratio: 0.88,
            is_active: false,
            key_held: false,
            note_number: 60,
        };
        voice.set_frequency(frequency);
        voice
    }

    /// Sets the fundamental frequency in Hz and calculates delay lengths.
    pub fn set_frequency(&mut self, freq: f32) {
        self.frequency = freq.clamp(20.0, 5000.0);
        let period_samples = self.sample_rate as f32 / self.frequency;
        let delay = (period_samples - 3.0).clamp(2.0, (MAX_CLAV_DELAY - 1) as f32);
        self.delay_len = delay;
    }

    /// Triggers note on with velocity and anvil hardness.
    pub fn note_on(&mut self, note: u8, freq: f32, velocity: f32, hardness: f32) {
        self.note_number = note;
        self.set_frequency(freq);
        self.is_active = true;
        self.key_held = true;
        self.anvil.strike(velocity, hardness, self.sample_rate);
    }

    /// Releases the note and engages yarn wool damping.
    pub fn note_off(&mut self) {
        if self.key_held {
            self.key_held = false;
            self.damper.trigger_release(self.frequency, self.sample_rate);
        }
    }

    /// Processes one sample through the physical string waveguide and returns (neck_pickup, bridge_pickup).
    #[inline]
    pub fn process_sample(&mut self) -> (f32, f32) {
        if !self.is_active {
            return (0.0, 0.0);
        }

        // Read fundamental delayed sample
        let read_pos = (self.write_pos as f32 - self.delay_len + MAX_CLAV_DELAY as f32) % (MAX_CLAV_DELAY as f32);
        let r0 = read_pos.floor() as usize % MAX_CLAV_DELAY;
        let r1 = (r0 + 1) % MAX_CLAV_DELAY;
        let frac = read_pos.fract();

        let delayed_raw = self.delay_buffer[r0] * (1.0 - frac) + self.delay_buffer[r1] * frac;

        // Dispersion allpass filter for steel string stiffness
        let dispersed = self.disp_coef * delayed_raw + self.disp_x - self.disp_coef * self.disp_y;
        self.disp_x = delayed_raw;
        self.disp_y = dispersed;

        // 1-pole damping filter
        let effective_damping = if self.key_held { 0.22 } else { 0.88 * self.damper.damping_amount };
        self.damping_state += effective_damping * (dispersed - self.damping_state);
        let damped = self.damping_state;

        // Anvil strike force injection
        let anvil_force = self.anvil.compute_force(damped);

        let loop_gain = if self.key_held { self.loop_feedback } else { 0.65 };
        let new_string_sample = (damped * loop_gain + anvil_force).clamp(-4.0, 4.0);

        self.delay_buffer[self.write_pos] = new_string_sample;

        // Sample Neck Pickup tap
        let neck_delay = (self.delay_len * self.neck_tap_ratio).clamp(1.0, self.delay_len);
        let neck_pos = (self.write_pos as f32 - neck_delay + MAX_CLAV_DELAY as f32) % (MAX_CLAV_DELAY as f32);
        let n0 = neck_pos.floor() as usize % MAX_CLAV_DELAY;
        let n1 = (n0 + 1) % MAX_CLAV_DELAY;
        let n_frac = neck_pos.fract();
        let neck_sample = self.delay_buffer[n0] * (1.0 - n_frac) + self.delay_buffer[n1] * n_frac;

        // Sample Bridge Pickup tap
        let bridge_delay = (self.delay_len * self.bridge_tap_ratio).clamp(1.0, self.delay_len);
        let bridge_pos = (self.write_pos as f32 - bridge_delay + MAX_CLAV_DELAY as f32) % (MAX_CLAV_DELAY as f32);
        let b0 = bridge_pos.floor() as usize % MAX_CLAV_DELAY;
        let b1 = (b0 + 1) % MAX_CLAV_DELAY;
        let b_frac = bridge_pos.fract();
        let bridge_sample = self.delay_buffer[b0] * (1.0 - b_frac) + self.delay_buffer[b1] * b_frac;

        self.write_pos = (self.write_pos + 1) % MAX_CLAV_DELAY;

        // Damper thud transient
        let thud = self.damper.process_sample(self.sample_rate);

        // Deactivation check
        if !self.key_held && new_string_sample.abs() < 1e-5 && self.damper.thud_env < 1e-4 {
            self.is_active = false;
        }

        (neck_sample + thud, bridge_sample + thud)
    }

    /// Resets voice to silence.
    pub fn reset(&mut self) {
        self.delay_buffer.fill(0.0);
        self.write_pos = 0;
        self.disp_x = 0.0;
        self.disp_y = 0.0;
        self.damping_state = 0.0;
        self.damper.reset();
        self.is_active = false;
        self.key_held = false;
    }
}

/// 4-band Tone Switch Filter Bank modeling the Clavinet D6 EQ circuitry.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ClavinetFilterBank {
    pub switches: ClavinetToneSwitches,
    // 1-pole / 2-pole filter states
    brilliant_state: f32,
    treble_state: f32,
    medium_s1: f32,
    medium_s2: f32,
    soft_state: f32,
}

impl Default for ClavinetFilterBank {
    fn default() -> Self {
        Self {
            switches: ClavinetToneSwitches::default(),
            brilliant_state: 0.0,
            treble_state: 0.0,
            medium_s1: 0.0,
            medium_s2: 0.0,
            soft_state: 0.0,
        }
    }
}

impl ClavinetFilterBank {
    pub fn new(switches: ClavinetToneSwitches) -> Self {
        Self {
            switches,
            brilliant_state: 0.0,
            treble_state: 0.0,
            medium_s1: 0.0,
            medium_s2: 0.0,
            soft_state: 0.0,
        }
    }

    /// Processes an audio sample through the 4-way rocker filter bank.
    #[inline]
    pub fn process(&mut self, input: f32, sample_rate: u32) -> f32 {
        let sr = sample_rate as f32;
        let mut out = input;

        // 1. Brilliant Switch: Highpass/Shelf (> 2500 Hz)
        if self.switches.brilliant {
            let coef = (2.0 * PI * 2500.0 / sr).clamp(0.01, 0.8);
            self.brilliant_state += coef * (out - self.brilliant_state);
            let hp = out - self.brilliant_state;
            out = self.brilliant_state * 0.4 + hp * 1.8;
        }

        // 2. Treble Switch: Peaking High-Mid (~1800 Hz)
        if self.switches.treble {
            let coef = (2.0 * PI * 1800.0 / sr).clamp(0.01, 0.7);
            self.treble_state += coef * (out - self.treble_state);
            let high_mid = out - self.treble_state;
            out += high_mid * 0.75;
        }

        // 3. Medium Switch: Bandpass Resonant Boost (~800 Hz)
        if self.switches.medium {
            let g = (PI * 800.0 / sr).tan().clamp(0.001, 5.0);
            let k = 1.0 / 2.5; // Q ~ 2.5
            let a1 = 1.0 / (1.0 + g * (g + k));
            let a2 = g * a1;
            let a3 = g * a2;

            let v0 = out;
            let v1 = a1 * self.medium_s1 + a2 * (v0 - self.medium_s2);
            let v2 = self.medium_s2 + a2 * self.medium_s1 + a3 * (v0 - self.medium_s2);
            self.medium_s1 = 2.0 * v1 - self.medium_s1;
            self.medium_s2 = 2.0 * v2 - self.medium_s2;

            let bp = v1;
            out += bp * 1.2;
        }

        // 4. Soft Switch: Lowpass High-Damping Filter (~1200 Hz rolloff)
        if self.switches.soft {
            let coef = (2.0 * PI * 1200.0 / sr).clamp(0.01, 0.6);
            self.soft_state += coef * (out - self.soft_state);
            out = self.soft_state * 0.85;
        }

        out
    }

    pub fn reset(&mut self) {
        self.brilliant_state = 0.0;
        self.treble_state = 0.0;
        self.medium_s1 = 0.0;
        self.medium_s2 = 0.0;
        self.soft_state = 0.0;
    }
}

/// Physical Modeling Electromechanical Clavinet Synthesizer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clavinet {
    /// Active sound profile.
    pub profile: ClavinetProfile,
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Pre-allocated polyphonic voices.
    voices: Vec<ClavinetVoice>,
    /// Rubber anvil tip hardness $[0.0 ..= 1.0]$.
    pub anvil_hardness: f32,
    /// Yarn wool damping amount $[0.0 ..= 1.0]$.
    pub yarn_damping: f32,
    /// Dual electromagnetic pickup selection mode.
    pub pickup_mode: ClavinetPickupSelection,
    /// Pickup mix blend $[0.0 ..= 1.0]$ (0.0 = Neck, 1.0 = Bridge).
    pub pickup_blend: f32,
    /// Continuous pickup phase angle in degrees $[0.0 ..= 180.0^\circ]$.
    pub pickup_phase_deg: f32,
    /// 4-way rocker tone switch filter bank.
    pub tone_filters: ClavinetFilterBank,
    /// Integrated dynamic state-variable auto-wah.
    pub auto_wah: AutoWah,
    /// Master volume level $[0.0 ..= 2.0]$.
    pub master_level: f32,
    /// Voice allocation counter.
    voice_alloc_counter: u64,
}

impl Default for Clavinet {
    fn default() -> Self {
        Self::new(48000)
    }
}

impl Clavinet {
    /// Creates a new physical Clavinet synthesis engine.
    pub fn new(sample_rate: u32) -> Self {
        let sr = sample_rate.max(8000);
        let mut voices = Vec::with_capacity(MAX_CLAV_VOICES);
        for _ in 0..MAX_CLAV_VOICES {
            voices.push(ClavinetVoice::new(261.63, sr));
        }

        let mut clav = Self {
            profile: ClavinetProfile::StevieSuperstition,
            sample_rate: sr,
            voices,
            anvil_hardness: 0.85,
            yarn_damping: 0.75,
            pickup_mode: ClavinetPickupSelection::ParallelInPhase,
            pickup_blend: 0.50,
            pickup_phase_deg: 0.0,
            tone_filters: ClavinetFilterBank::default(),
            auto_wah: AutoWah::new(),
            master_level: 0.90,
            voice_alloc_counter: 0,
        };

        clav.set_profile(ClavinetProfile::StevieSuperstition);
        clav
    }

    /// Sets sound profile and loads physical & electronic defaults.
    pub fn set_profile(&mut self, profile: ClavinetProfile) {
        self.profile = profile;

        let (mode, blend, phase, switches) = profile.electronic_defaults();
        self.pickup_mode = mode;
        self.pickup_blend = blend;
        self.pickup_phase_deg = phase;
        self.tone_filters.switches = switches;

        let (hardness, damping, _) = profile.mechanical_defaults();
        self.anvil_hardness = hardness;
        self.yarn_damping = damping;

        let (wah_en, wah_sens, wah_freq, wah_res, wah_mix) = profile.auto_wah_defaults();
        self.auto_wah.enabled = wah_en;
        self.auto_wah.sensitivity = wah_sens;
        self.auto_wah.base_freq_hz = wah_freq;
        self.auto_wah.resonance_q = wah_res;
        self.auto_wah.mix = wah_mix;
        self.auto_wah.filter_mode = AutoWahFilterMode::Bandpass;
        self.auto_wah.direction = AutoWahDirection::Up;

        self.apply_voice_parameters();
    }

    /// Applies current parameters to all voices.
    pub fn apply_voice_parameters(&mut self) {
        for voice in self.voices.iter_mut() {
            voice.damper.damping_amount = self.yarn_damping;
        }
    }

    /// Converts MIDI note number to frequency in Hz.
    #[inline]
    pub fn note_to_freq(note: u8) -> f32 {
        440.0 * 2.0f32.powf((note as f32 - 69.0) / 12.0)
    }

    /// Triggers a note on.
    pub fn note_on(&mut self, note: u8, velocity: f32) {
        let freq = Self::note_to_freq(note);

        // Find existing voice on this note or an inactive voice
        let mut target_idx = None;
        for (i, v) in self.voices.iter().enumerate() {
            if v.is_active && v.note_number == note {
                target_idx = Some(i);
                break;
            }
        }

        if target_idx.is_none() {
            for (i, v) in self.voices.iter().enumerate() {
                if !v.is_active {
                    target_idx = Some(i);
                    break;
                }
            }
        }

        let voice_idx = target_idx.unwrap_or(0);
        self.voice_alloc_counter += 1;

        let voice = &mut self.voices[voice_idx];
        voice.damper.damping_amount = self.yarn_damping;
        voice.note_on(note, freq, velocity, self.anvil_hardness);
    }

    /// Releases a note.
    pub fn note_off(&mut self, note: u8) {
        for voice in self.voices.iter_mut() {
            if voice.is_active && voice.note_number == note && voice.key_held {
                voice.note_off();
            }
        }
    }

    /// Releases all active notes.
    pub fn all_notes_off(&mut self) {
        for voice in self.voices.iter_mut() {
            if voice.is_active {
                voice.note_off();
            }
        }
    }

    /// Processes one stereo sample through all polyphonic voices, pickup matrix, tone bank, and auto-wah.
    #[inline]
    pub fn process_sample(&mut self) -> (f32, f32) {
        let mut neck_sum = 0.0f32;
        let mut bridge_sum = 0.0f32;

        for voice in self.voices.iter_mut() {
            if voice.is_active {
                let (neck, bridge) = voice.process_sample();
                neck_sum += neck;
                bridge_sum += bridge;
            }
        }

        // Dual pickup electronic mixing & phase vector cancellation
        let pickup_audio = match self.pickup_mode {
            ClavinetPickupSelection::NeckOnly => neck_sum,
            ClavinetPickupSelection::BridgeOnly => bridge_sum,
            ClavinetPickupSelection::ParallelInPhase => {
                let phase_rad = self.pickup_phase_deg * (PI / 180.0);
                let neck_w = (1.0 - self.pickup_blend) * 1.2;
                let bridge_w = self.pickup_blend * 1.2;
                neck_w * neck_sum + bridge_w * (bridge_sum * phase_rad.cos())
            }
            ClavinetPickupSelection::ParallelOutOfPhase => {
                // Out-of-phase destructive magnetic cancellation: Neck - Bridge
                let neck_w = 1.0 - self.pickup_blend;
                let bridge_w = self.pickup_blend;
                (neck_sum * neck_w - bridge_sum * bridge_w) * 1.6
            }
        };

        // 4-Way Rocker Tone Switch EQ Filter Bank
        let eq_audio = self.tone_filters.process(pickup_audio, self.sample_rate);

        // Dynamic State-Variable Auto-Wah
        let (wah_l, wah_r) = self.auto_wah.process_sample(eq_audio, eq_audio, self.sample_rate);

        let final_l = (wah_l * self.master_level).clamp(-1.0, 1.0);
        let final_r = (wah_r * self.master_level).clamp(-1.0, 1.0);

        (final_l, final_r)
    }

    /// Resets all internal voices, filters, and auto-wah state.
    pub fn reset(&mut self) {
        for voice in self.voices.iter_mut() {
            voice.reset();
        }
        self.tone_filters.reset();
        self.auto_wah.reset();
    }
}

impl SignalProcessor for Clavinet {
    fn name(&self) -> &str {
        "Clavinet"
    }

    #[allow(clippy::needless_range_loop)]
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
        let is_stereo = outputs.len() >= 2;

        for i in 0..num_samples {
            let (l, r) = self.process_sample();
            outputs[0][i] = l;
            if is_stereo {
                outputs[1][i] = r;
            }
        }
    }
}

impl AudioNode for Clavinet {
    fn name(&self) -> &str {
        "ClavinetNode"
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
    use summoner_core::transport::Transport;

    #[test]
    fn test_clavinet_zero_allocation() {
        let mut clav = Clavinet::new(48000);
        clav.set_profile(ClavinetProfile::StevieSuperstition);
        clav.note_on(58, 0.90); // Eb3
        clav.note_on(61, 0.85); // Gb3
        clav.note_on(65, 0.85); // Bb3

        let transport = Transport::new(48000, 120.0);
        let ctx = ProcessContext::from_transport(&transport);

        let mut out_l = [0.0f32; 64];
        let mut out_r = [0.0f32; 64];

        {
            let _guard = AllocGuard::new();
            for _ in 0..16 {
                clav.process_block(&[], &mut [&mut out_l[..], &mut out_r[..]], &ctx);
                for i in 0..64 {
                    assert!(out_l[i].is_finite());
                    assert!(out_r[i].is_finite());
                }
            }
        }
    }

    #[test]
    fn test_clavinet_pickup_modes_and_phase_cancellation() {
        let mut clav = Clavinet::new(48000);
        clav.note_on(60, 0.90);

        // In-Phase
        clav.pickup_mode = ClavinetPickupSelection::ParallelInPhase;
        let (in_l, _) = clav.process_sample();

        // Out-of-Phase
        clav.pickup_mode = ClavinetPickupSelection::ParallelOutOfPhase;
        let (out_l, _) = clav.process_sample();

        assert!(in_l.is_finite());
        assert!(out_l.is_finite());
    }

    #[test]
    fn test_tone_switch_bitmask() {
        let switches = ClavinetToneSwitches {
            brilliant: true,
            treble: false,
            medium: true,
            soft: false,
        };
        let mask = switches.to_bitmask();
        assert_eq!(mask, 1 | 4);

        let decoded = ClavinetToneSwitches::from_bitmask(mask);
        assert_eq!(switches, decoded);
    }
}
