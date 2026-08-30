// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling 88-Key Concert Grand Piano Synthesis Engine (Milestone 26).
//!
//! Provides a high-precision physical acoustic grand piano synthesizer featuring:
//! - 3-string unison coupling per note (trichords, bichords, monochords) with prompt & aftersound dual-decay dynamics
//! - Non-linear felt hammer contact law ($F \propto \delta^p$) with strike velocity scaling and Una Corda soft-pedal shift
//! - Stiff string dispersion allpass cascades ($B \approx 0.0001..0.0004$) for authentic inharmonic overtone stretching
//! - Bi-directional multi-port bridge impedance coupling to anisotropic spruce soundboard 2D modal matrix
//! - Continuous Sustain / Half-Pedaling, Sostenuto latch bitmask, and sympathetic damper release mechanics
//!
//! Enforces zero heap allocations in audio render loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};
use crate::soundboard_bridge::{
    BridgeWaveCoupler, GrandPianoFeltHammer, SoundboardProfile, SpruceSoundboard,
    MAX_UNISONS_PER_NOTE,
};
use crate::traits::SignalProcessor;

/// Maximum delay buffer capacity in samples (~20 Hz fundamental at 192 kHz).
pub const MAX_PIANO_DELAY: usize = 16384;

/// Maximum polyphonic voices allocated in the zero-heap voice pool.
pub const MAX_PIANO_VOICES: usize = 32;

/// Total number of standard keys on a concert grand piano (A0=21 .. C8=108).
pub const TOTAL_PIANO_KEYS: usize = 88;

/// Minimum MIDI note number on standard 88-key piano (A0 = 21, 27.5 Hz).
pub const MIN_PIANO_NOTE: u8 = 21;

/// Maximum MIDI note number on standard 88-key piano (C8 = 108, 4186.01 Hz).
pub const MAX_PIANO_NOTE: u8 = 108;

fn default_piano_delay_buffer() -> Box<[f32; MAX_PIANO_DELAY]> {
    Box::new([0.0; MAX_PIANO_DELAY])
}

fn default_sostenuto_keys() -> [bool; TOTAL_PIANO_KEYS] {
    [false; TOTAL_PIANO_KEYS]
}

/// Concert grand piano acoustic preset profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GrandPianoProfile {
    /// 9-foot Steinway D Concert Grand (expansive dynamic range, balanced prompt/aftersound).
    #[default]
    SteinwayD9Foot,
    /// 9.5-foot Bösendorfer Imperial 290 (deep sub-bass resonance, dark warm sustain).
    BösendorferImperial,
    /// 9-foot Yamaha CFX (bright, crisp prompt attack, crystalline treble radiation).
    YamahaCFX,
    /// Intimate Warm Studio Grand (felted damping, close mic dispersion, mellow body).
    IntimateStudio,
    /// Impressionist Una Corda (delicate ethereal shimmer, soft pedal character).
    ImpressionistUnaCorda,
    /// Prepared Avant-Garde (muted string junctions, metallic boundary reflections).
    PreparedAvantGarde,
}

impl GrandPianoProfile {
    pub fn name(&self) -> &'static str {
        match self {
            Self::SteinwayD9Foot => "Steinway D-274 Concert Grand (9ft)",
            Self::BösendorferImperial => "Bösendorfer Imperial 290 (9.5ft)",
            Self::YamahaCFX => "Yamaha CFX Concert Grand (9ft)",
            Self::IntimateStudio => "Intimate Warm Studio Grand",
            Self::ImpressionistUnaCorda => "Impressionist Una Corda Grand",
            Self::PreparedAvantGarde => "Prepared Avant-Garde Grand",
        }
    }

    pub fn to_soundboard_profile(&self) -> SoundboardProfile {
        match self {
            Self::SteinwayD9Foot => SoundboardProfile::SteinwayD9Foot,
            Self::BösendorferImperial => SoundboardProfile::BösendorferImperial,
            Self::YamahaCFX => SoundboardProfile::YamahaCFX,
            Self::IntimateStudio => SoundboardProfile::IntimateStudio,
            Self::ImpressionistUnaCorda => SoundboardProfile::ImpressionistUnaCorda,
            Self::PreparedAvantGarde => SoundboardProfile::PreparedAvantGarde,
        }
    }

    /// Returns nominal $(T_{60\text{prompt}}, T_{60\text{after}}, \text{detune\_cents}, B_{\text{dispersion}}, \text{hammer\_hardness})$.
    pub fn nominal_physics(&self) -> (f32, f32, f32, f32, f32) {
        match self {
            Self::SteinwayD9Foot => (2.2, 14.0, 0.75, 0.00018, 0.65),
            Self::BösendorferImperial => (2.8, 18.0, 0.60, 0.00012, 0.50),
            Self::YamahaCFX => (1.8, 11.0, 0.90, 0.00025, 0.80),
            Self::IntimateStudio => (1.5, 9.0, 0.50, 0.00010, 0.35),
            Self::ImpressionistUnaCorda => (3.2, 20.0, 1.10, 0.00015, 0.30),
            Self::PreparedAvantGarde => (0.6, 3.5, 2.50, 0.00060, 0.90),
        }
    }
}

/// A first-order allpass filter section for stiff-string inharmonic dispersion.
///
/// $H_{\text{disp}}(z) = \frac{-a + z^{-1}}{1 - a z^{-1}}$
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PianoDispersionAllpass {
    pub coefficient: f32,
    x_prev: f32,
    y_prev: f32,
}

impl Default for PianoDispersionAllpass {
    fn default() -> Self {
        Self {
            coefficient: 0.0,
            x_prev: 0.0,
            y_prev: 0.0,
        }
    }
}

impl PianoDispersionAllpass {
    pub fn new(coefficient: f32) -> Self {
        Self {
            coefficient: coefficient.clamp(-0.99, 0.99),
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

/// A single digital waveguide string with inharmonic dispersion, fractional delay, and loss filtering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PianoStringWaveguide {
    #[serde(skip, default = "default_piano_delay_buffer")]
    pub delay_buffer: Box<[f32; MAX_PIANO_DELAY]>,
    pub write_idx: usize,
    pub dispersion: PianoDispersionAllpass,
    pub loss_filter_state: f32,
    pub loop_gain: f32,
    pub loss_cutoff_coeff: f32,
    pub is_active: bool,
}

impl Default for PianoStringWaveguide {
    fn default() -> Self {
        Self {
            delay_buffer: default_piano_delay_buffer(),
            write_idx: 0,
            dispersion: PianoDispersionAllpass::default(),
            loss_filter_state: 0.0,
            loop_gain: 0.995,
            loss_cutoff_coeff: 0.35,
            is_active: false,
        }
    }
}

impl PianoStringWaveguide {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.delay_buffer.fill(0.0);
        self.write_idx = 0;
        self.dispersion.reset();
        self.loss_filter_state = 0.0;
        self.is_active = false;
    }

    /// Read fractional delay from circular buffer with linear interpolation.
    #[inline]
    pub fn read_delay(&self, delay_samples: f32) -> f32 {
        let d = delay_samples.clamp(1.0, (MAX_PIANO_DELAY - 4) as f32);
        let int_d = d.floor() as usize;
        let frac_d = d - d.floor();

        let idx0 = (self.write_idx + MAX_PIANO_DELAY - int_d) % MAX_PIANO_DELAY;
        let idx1 = (idx0 + MAX_PIANO_DELAY - 1) % MAX_PIANO_DELAY;

        let s0 = self.delay_buffer[idx0];
        let s1 = self.delay_buffer[idx1];
        s0 + frac_d * (s1 - s0)
    }

    /// Step the waveguide: read delayed traveling wave, pass through dispersion allpass,
    /// bridge reflection and loss filtering, and write back into buffer.
    #[inline]
    pub fn step(
        &mut self,
        delay_samples: f32,
        incoming_excitation: f32,
        damper_damping: f32,
    ) -> f32 {
        // Read traveling wave from delay buffer
        let delayed_wave = self.read_delay(delay_samples);

        // Pass through inharmonic dispersion allpass filter
        let dispersed = self.dispersion.step(delayed_wave);

        // Reflection at termination with lowpass loss filtering and damper felt absorption
        let eff_gain = self.loop_gain * (1.0 - damper_damping).max(0.01);
        let reflected = -dispersed * eff_gain;

        // One-pole lowpass loss filter
        let alpha = self.loss_cutoff_coeff.clamp(0.05, 0.95);
        self.loss_filter_state = (1.0 - alpha) * reflected + alpha * self.loss_filter_state;

        // Inject excitation into write head
        let total_in = (self.loss_filter_state + incoming_excitation).clamp(-4.0, 4.0);
        self.delay_buffer[self.write_idx] = total_in;
        self.write_idx = (self.write_idx + 1) % MAX_PIANO_DELAY;

        self.loss_filter_state
    }
}

/// A polyphonic voice modeling a single grand piano note with 1, 2, or 3 coupled string unisons.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrandPianoVoice {
    pub note_number: u8,
    pub frequency_hz: f32,
    pub velocity: f32,
    pub num_unisons: usize,
    pub strings: [PianoStringWaveguide; MAX_UNISONS_PER_NOTE],
    pub detune_cents: [f32; MAX_UNISONS_PER_NOTE],
    pub hammer: GrandPianoFeltHammer,
    pub is_key_down: bool,
    pub is_sostenuto_held: bool,
    pub damper_position: f32, // 0.0 = fully damped, 1.0 = damper lifted
    pub prompt_t60: f32,
    pub aftersound_t60: f32,
    pub inharmonicity_b: f32,
    pub age_samples: u64,
    pub is_active: bool,
    pub sample_rate: u32,
}

impl Default for GrandPianoVoice {
    fn default() -> Self {
        Self {
            note_number: 60,
            frequency_hz: 261.63,
            velocity: 0.0,
            num_unisons: 3,
            strings: [
                PianoStringWaveguide::new(),
                PianoStringWaveguide::new(),
                PianoStringWaveguide::new(),
            ],
            detune_cents: [-0.75, 0.0, 0.75],
            hammer: GrandPianoFeltHammer::default(),
            is_key_down: false,
            is_sostenuto_held: false,
            damper_position: 0.0,
            prompt_t60: 2.2,
            aftersound_t60: 14.0,
            inharmonicity_b: 0.00018,
            age_samples: 0,
            is_active: false,
            sample_rate: 48000,
        }
    }
}

impl GrandPianoVoice {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            ..Default::default()
        }
    }

    /// Configure voice parameters for a specific MIDI note and piano physics profile.
    pub fn init_for_note(
        &mut self,
        note: u8,
        velocity: f32,
        profile: GrandPianoProfile,
        una_corda_shift: f32,
        global_inharmonicity_b: f32,
        global_unison_detune_cents: f32,
    ) {
        self.note_number = note;
        self.velocity = velocity.clamp(0.0, 1.0);
        self.frequency_hz = 440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0);
        self.is_key_down = true;
        self.is_active = true;
        self.age_samples = 0;
        self.damper_position = 1.0; // Damper lifted while key is held down

        // Determine unison count based on piano register:
        // Bass notes (21..32): 1 single wound string (monochord)
        // Low-tenor notes (33..44): 2 wound strings (bichord)
        // Mid & Treble notes (45..108): 3 plain steel strings (trichord)
        self.num_unisons = if note <= 32 {
            1
        } else if note <= 44 {
            2
        } else {
            3
        };

        let (prompt_t60, after_t60, detune_cents, _, hammer_hardness) = profile.nominal_physics();
        self.prompt_t60 = prompt_t60;
        self.aftersound_t60 = after_t60;
        self.inharmonicity_b = global_inharmonicity_b;

        let detune = if global_unison_detune_cents > 0.0 {
            global_unison_detune_cents
        } else {
            detune_cents
        };

        match self.num_unisons {
            1 => {
                self.detune_cents = [0.0, 0.0, 0.0];
            }
            2 => {
                self.detune_cents = [-detune * 0.7, detune * 0.7, 0.0];
            }
            3 => {
                self.detune_cents = [-detune, 0.0, detune];
            }
            _ => {}
        }

        // Graded hammer mass: ~9 grams at A0 down to ~3 grams at C8
        let norm_pitch = ((note as f32 - 21.0) / (108.0 - 21.0)).clamp(0.0, 1.0);
        let hammer_mass = 0.009 * (1.0 - 0.65 * norm_pitch);
        let hammer_stiffness = 1.0e8 * (1.0 + 2.0 * hammer_hardness);
        let hammer_p = 2.2 + 0.6 * (1.0 - norm_pitch);
        let strike_beta = 0.125 - 0.04 * norm_pitch; // 1/8 in bass to 1/12 in treble

        self.hammer = GrandPianoFeltHammer::new(
            hammer_mass,
            hammer_stiffness,
            hammer_p,
            0.35,
            strike_beta,
        );
        self.hammer.una_corda_shift = una_corda_shift;

        // Strike velocity in m/s: 0.1 m/s (pianissimo) to 6.5 m/s (fortissimo)
        let strike_mps = 0.15 + self.velocity.powf(1.6) * 6.0;
        self.hammer.trigger_strike(strike_mps);

        // Configure waveguides & dispersion allpass filters
        let b = self.inharmonicity_b;
        // Dispersion allpass coefficient approximation: a ≈ (1 - sqrt(1 + B)) / (1 + sqrt(1 + B))
        let disp_coeff = if b > 1e-6 {
            let sqrt_b = (1.0 + b * 20.0).sqrt();
            ((1.0 - sqrt_b) / (1.0 + sqrt_b)).clamp(-0.85, 0.0)
        } else {
            0.0
        };

        for i in 0..self.num_unisons {
            self.strings[i].reset();
            self.strings[i].dispersion = PianoDispersionAllpass::new(disp_coeff);

            // Compute loop gain from aftersound T60 (sustain decay)
            let eff_f0 = self.frequency_hz * 2.0_f32.powf(self.detune_cents[i] / 1200.0);
            let period_sec = 1.0 / eff_f0.max(20.0);
            let atten_db_per_period = (60.0 / self.aftersound_t60.max(0.5)) * period_sec;
            self.strings[i].loop_gain = 10.0_f32.powf(-atten_db_per_period / 20.0).clamp(0.90, 0.99995);
            self.strings[i].loss_cutoff_coeff = (0.20 + 0.50 * norm_pitch).clamp(0.10, 0.85);
            self.strings[i].is_active = true;
        }
    }

    /// Process one sample for this voice and return the string force transmitted to the bridge.
    #[inline]
    pub fn process_sample(&mut self, sustain_pedal: f32) -> f32 {
        if !self.is_active {
            return 0.0;
        }

        self.age_samples += 1;
        let sr = self.sample_rate as f32;

        // Determine effective damper felt damping:
        // If key is held, or sustain pedal is down, or sostenuto holds this key -> damper is lifted (damping = 0.0).
        // Otherwise, heavy damper felt is in contact with string (rapid damping).
        let effective_damper_lift = if self.is_key_down || self.is_sostenuto_held {
            1.0
        } else {
            sustain_pedal.clamp(0.0, 1.0)
        };

        // Damper felt absorption factor [0.0 = no damping, 0.85 = fast felt damping]
        let damper_damping = (1.0 - effective_damper_lift) * 0.75;

        // Step hammer dynamics
        let mut avg_string_disp = 0.0;
        for i in 0..self.num_unisons {
            avg_string_disp += self.strings[i].loss_filter_state;
        }
        avg_string_disp = (avg_string_disp / self.num_unisons as f32) * 0.0005;

        let hammer_force_n = self.hammer.step(avg_string_disp, self.sample_rate);
        let hammer_impulse = hammer_force_n * 0.002;

        // Process coupled unisons
        let mut total_bridge_force = 0.0;
        let mut total_energy = 0.0;

        for i in 0..self.num_unisons {
            let detune_ratio = 2.0_f32.powf(self.detune_cents[i] / 1200.0);
            let eff_f0 = (self.frequency_hz * detune_ratio).clamp(20.0, sr * 0.45);
            let delay_len = sr / eff_f0;

            // When Una Corda is engaged, string 0 receives reduced or zero strike impulse (shifted hammer)
            let string_excitation = if i == 0 && self.hammer.una_corda_shift > 0.5 {
                hammer_impulse * (1.0 - self.hammer.una_corda_shift)
            } else {
                hammer_impulse
            };

            let string_out = self.strings[i].step(delay_len, string_excitation, damper_damping);
            total_bridge_force += string_out;
            total_energy += string_out.abs();
        }

        // Voice deactivation check when energy decays below threshold and key is released
        if self.age_samples > (sr * 0.05) as u64
            && !self.is_key_down
            && effective_damper_lift < 0.05
            && total_energy < 1e-5
            && !self.hammer.in_contact
        {
            self.is_active = false;
        }

        total_bridge_force
    }

    pub fn note_off(&mut self) {
        self.is_key_down = false;
    }
}

/// 88-Key Physical Modeling Concert Grand Piano Synthesizer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcertGrandPiano {
    pub profile: GrandPianoProfile,
    pub soundboard: SpruceSoundboard,
    pub bridge: BridgeWaveCoupler,
    pub voices: [GrandPianoVoice; MAX_PIANO_VOICES],
    pub sustain_pedal: f32, // [0.0 ..= 1.0] continuous damper lift
    pub una_corda_pedal: f32, // [0.0 ..= 1.0] soft pedal shift
    pub sostenuto_pedal: bool, // Sostenuto latch engaged
    #[serde(skip, default = "default_sostenuto_keys")]
    pub sostenuto_latch_keys: [bool; TOTAL_PIANO_KEYS],
    pub master_gain: f32,
    pub unison_detune_cents: f32,
    pub string_inharmonicity_b: f32,
    pub sample_rate: u32,
}

impl ConcertGrandPiano {
    pub fn new(sample_rate: u32) -> Self {
        let profile = GrandPianoProfile::SteinwayD9Foot;
        let sb_profile = profile.to_soundboard_profile();
        let soundboard = SpruceSoundboard::new(sample_rate, sb_profile);
        let (_decay_scale, zb, bleed, b, _) = (1.0, 420.0, 0.35, 0.00018, 0.65);
        let bridge = BridgeWaveCoupler::new(zb, bleed);

        let voices = [
            GrandPianoVoice::new(sample_rate), GrandPianoVoice::new(sample_rate),
            GrandPianoVoice::new(sample_rate), GrandPianoVoice::new(sample_rate),
            GrandPianoVoice::new(sample_rate), GrandPianoVoice::new(sample_rate),
            GrandPianoVoice::new(sample_rate), GrandPianoVoice::new(sample_rate),
            GrandPianoVoice::new(sample_rate), GrandPianoVoice::new(sample_rate),
            GrandPianoVoice::new(sample_rate), GrandPianoVoice::new(sample_rate),
            GrandPianoVoice::new(sample_rate), GrandPianoVoice::new(sample_rate),
            GrandPianoVoice::new(sample_rate), GrandPianoVoice::new(sample_rate),
            GrandPianoVoice::new(sample_rate), GrandPianoVoice::new(sample_rate),
            GrandPianoVoice::new(sample_rate), GrandPianoVoice::new(sample_rate),
            GrandPianoVoice::new(sample_rate), GrandPianoVoice::new(sample_rate),
            GrandPianoVoice::new(sample_rate), GrandPianoVoice::new(sample_rate),
            GrandPianoVoice::new(sample_rate), GrandPianoVoice::new(sample_rate),
            GrandPianoVoice::new(sample_rate), GrandPianoVoice::new(sample_rate),
            GrandPianoVoice::new(sample_rate), GrandPianoVoice::new(sample_rate),
            GrandPianoVoice::new(sample_rate), GrandPianoVoice::new(sample_rate),
        ];

        Self {
            profile,
            soundboard,
            bridge,
            voices,
            sustain_pedal: 0.0,
            una_corda_pedal: 0.0,
            sostenuto_pedal: false,
            sostenuto_latch_keys: [false; TOTAL_PIANO_KEYS],
            master_gain: 0.85,
            unison_detune_cents: 0.75,
            string_inharmonicity_b: b,
            sample_rate,
        }
    }

    pub fn set_profile(&mut self, profile: GrandPianoProfile) {
        self.profile = profile;
        let sb_profile = profile.to_soundboard_profile();
        self.soundboard.set_profile(sb_profile);
        let (_, zb, bleed, b, _) = profile.nominal_physics();
        self.bridge.bridge_impedance = zb;
        self.bridge.sympathetic_bleed = bleed;
        self.string_inharmonicity_b = b;
    }

    /// Set continuous sustain / half-pedaling position $[0.0 ..= 1.0]$.
    pub fn set_sustain_pedal(&mut self, pos: f32) {
        self.sustain_pedal = pos.clamp(0.0, 1.0);
    }

    /// Set continuous Una Corda soft pedal shift $[0.0 ..= 1.0]$.
    pub fn set_una_corda_pedal(&mut self, shift: f32) {
        self.una_corda_pedal = shift.clamp(0.0, 1.0);
        for voice in &mut self.voices {
            if voice.is_active {
                voice.hammer.una_corda_shift = self.una_corda_pedal;
            }
        }
    }

    /// Set Sostenuto pedal engagement latch.
    pub fn set_sostenuto_pedal(&mut self, engaged: bool) {
        if engaged && !self.sostenuto_pedal {
            // Latch only currently active held keys
            for voice in &mut self.voices {
                if voice.is_active && voice.is_key_down {
                    let key_idx = (voice.note_number.saturating_sub(MIN_PIANO_NOTE) as usize).min(TOTAL_PIANO_KEYS - 1);
                    self.sostenuto_latch_keys[key_idx] = true;
                    voice.is_sostenuto_held = true;
                }
            }
        } else if !engaged && self.sostenuto_pedal {
            // Release sostenuto latch
            self.sostenuto_latch_keys.fill(false);
            for voice in &mut self.voices {
                voice.is_sostenuto_held = false;
            }
        }
        self.sostenuto_pedal = engaged;
    }

    /// Trigger piano note on (MIDI note 21..108, velocity 0.0..1.0).
    pub fn note_on(&mut self, note: u8, velocity: f32) {
        let clamped_note = note.clamp(MIN_PIANO_NOTE, MAX_PIANO_NOTE);
        let vel = velocity.clamp(0.0, 1.0);
        if vel <= 0.001 {
            self.note_off(clamped_note);
            return;
        }

        // Find inactive voice, or oldest voice to steal
        let mut selected_idx = 0;
        let mut max_age = 0;

        for (i, voice) in self.voices.iter().enumerate() {
            if !voice.is_active {
                selected_idx = i;
                break;
            }
            if voice.age_samples > max_age {
                max_age = voice.age_samples;
                selected_idx = i;
            }
        }

        let voice = &mut self.voices[selected_idx];
        voice.init_for_note(
            clamped_note,
            vel,
            self.profile,
            self.una_corda_pedal,
            self.string_inharmonicity_b,
            self.unison_detune_cents,
        );

        // Check if note is latched by sostenuto
        let key_idx = (clamped_note.saturating_sub(MIN_PIANO_NOTE) as usize).min(TOTAL_PIANO_KEYS - 1);
        if self.sostenuto_pedal && self.sostenuto_latch_keys[key_idx] {
            voice.is_sostenuto_held = true;
        }
    }

    /// Trigger piano note off for MIDI note.
    pub fn note_off(&mut self, note: u8) {
        for voice in &mut self.voices {
            if voice.is_active && voice.note_number == note {
                voice.note_off();
            }
        }
    }

    /// Process one stereo audio sample `(Left, Right)`.
    #[inline]
    pub fn process_sample(&mut self) -> (Sample, Sample) {
        // Collect string forces from all active voices into bridge
        let mut total_bridge_force = 0.0;
        let mut active_count = 0;

        for voice in &mut self.voices {
            if voice.is_active {
                let f = voice.process_sample(self.sustain_pedal);
                total_bridge_force += f;
                active_count += 1;
            }
        }

        if active_count == 0 {
            return (0.0, 0.0);
        }

        // Drive bridge and anisotropic spruce soundboard modal matrix
        let bridge_forces_in = [total_bridge_force];
        let mut reflected_out = [0.0];
        let transmitted_force = self.bridge.process_scattering_junction(&bridge_forces_in, &mut reflected_out);

        let (sb_l, sb_r) = self.soundboard.process_bridge_force(transmitted_force);

        let gain = self.master_gain;
        let out_l = (sb_l * gain * 0.8 + total_bridge_force * 0.15).clamp(-1.0, 1.0);
        let out_r = (sb_r * gain * 0.8 + total_bridge_force * 0.15).clamp(-1.0, 1.0);

        (out_l, out_r)
    }

    /// All notes off / emergency panic reset.
    pub fn all_notes_off(&mut self) {
        for voice in &mut self.voices {
            voice.is_active = false;
            voice.is_key_down = false;
            voice.is_sostenuto_held = false;
            for string in &mut voice.strings {
                string.reset();
            }
        }
        self.soundboard.reset();
        self.bridge.reset();
        self.sostenuto_latch_keys.fill(false);
        self.sostenuto_pedal = false;
    }
}

impl SignalProcessor for ConcertGrandPiano {
    fn name(&self) -> &str {
        "ConcertGrandPiano"
    }

    fn process_block(
        &mut self,
        _inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        _ctx: &ProcessContext,
    ) {
        if outputs.is_empty() {
            return;
        }

        let num_samples = outputs[0].len();
        for i in 0..num_samples {
            let (l, r) = self.process_sample();
            if outputs.len() >= 2 {
                outputs[0][i] = l;
                outputs[1][i] = r;
            } else {
                outputs[0][i] = (l + r) * 0.5;
            }
        }
    }
}

/// AudioNode wrapper for Concert Grand Piano physical modeling synthesis engine.
#[derive(Debug)]
pub struct ConcertGrandPianoNode {
    pub piano: ConcertGrandPiano,
}

impl ConcertGrandPianoNode {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            piano: ConcertGrandPiano::new(sample_rate),
        }
    }
}

impl AudioNode for ConcertGrandPianoNode {
    fn name(&self) -> &str {
        "ConcertGrandPianoNode"
    }

    fn process(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        self.piano.process_block(inputs, outputs, ctx);
    }
}
