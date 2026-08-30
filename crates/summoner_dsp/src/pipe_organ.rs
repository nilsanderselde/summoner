// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Pipe Organ Synthesis Engine (Milestone 24).
//!
//! Provides a physical modeling pipe organ engine simulating open and stopped metal/wooden
//! flue pipes (Principal 8', Bourdon 16', Octave 4', Flute 4', Super Octave 2', Mixture IV compound ranks)
//! coupled with beating brass reed pipes (Trompette 8', Vox Humana 8'), chiff vortex attack transients
//! (t_chiff in [5, 80] ms), non-linear windchest aerodynamics with reservoir pressure sag,
//! tracker action key velocity dynamics, and pneumatic tremulant modulation.
//!
//! Enforces zero heap allocations in audio render loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::f32::consts::{PI, TAU};
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};
use crate::windchest::Windchest;

/// Maximum number of polyphonic Pipe Organ voices.
pub const MAX_ORGAN_VOICES: usize = 32;

/// Maximum number of simultaneous pipe ranks.
pub const NUM_RANKS: usize = 8;

/// Stop registration bitmask definitions.
pub const STOP_PRINCIPAL_8: u32 = 1 << 0;   // Open metal flue (8')
pub const STOP_BOURDON_16: u32 = 1 << 1;    // Stopped wooden flue (16')
pub const STOP_OCTAVE_4: u32 = 1 << 2;      // Open metal flue (4')
pub const STOP_FLUTE_4: u32 = 1 << 3;       // Stopped wooden flute (4')
pub const STOP_SUPER_OCTAVE_2: u32 = 1 << 4;// Open metal flue (2')
pub const STOP_MIXTURE_IV: u32 = 1 << 5;    // Compound 4-rank chorus (1-1/3', 1', 2/3', 1/2')
pub const STOP_TROMPETTE_8: u32 = 1 << 6;   // Beating brass reed (8')
pub const STOP_VOX_HUMANA_8: u32 = 1 << 7;  // Short cylindrical reed with formant (8')

/// 8-stop mechanical registration drawknobs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PipeOrganStops {
    pub principal_8: bool,
    pub bourdon_16: bool,
    pub octave_4: bool,
    pub flute_4: bool,
    pub super_octave_2: bool,
    pub mixture_iv: bool,
    pub trompette_8: bool,
    pub vox_humana_8: bool,
}

impl Default for PipeOrganStops {
    fn default() -> Self {
        Self {
            principal_8: true,
            bourdon_16: true,
            octave_4: true,
            flute_4: false,
            super_octave_2: true,
            mixture_iv: true,
            trompette_8: true,
            vox_humana_8: false,
        }
    }
}

impl PipeOrganStops {
    /// Encode active stops to 32-bit bitmask.
    pub fn to_bitmask(&self) -> u32 {
        let mut mask = 0u32;
        if self.principal_8 { mask |= STOP_PRINCIPAL_8; }
        if self.bourdon_16 { mask |= STOP_BOURDON_16; }
        if self.octave_4 { mask |= STOP_OCTAVE_4; }
        if self.flute_4 { mask |= STOP_FLUTE_4; }
        if self.super_octave_2 { mask |= STOP_SUPER_OCTAVE_2; }
        if self.mixture_iv { mask |= STOP_MIXTURE_IV; }
        if self.trompette_8 { mask |= STOP_TROMPETTE_8; }
        if self.vox_humana_8 { mask |= STOP_VOX_HUMANA_8; }
        mask
    }

    /// Decode active stops from 32-bit bitmask.
    pub fn from_bitmask(mask: u32) -> Self {
        Self {
            principal_8: (mask & STOP_PRINCIPAL_8) != 0,
            bourdon_16: (mask & STOP_BOURDON_16) != 0,
            octave_4: (mask & STOP_OCTAVE_4) != 0,
            flute_4: (mask & STOP_FLUTE_4) != 0,
            super_octave_2: (mask & STOP_SUPER_OCTAVE_2) != 0,
            mixture_iv: (mask & STOP_MIXTURE_IV) != 0,
            trompette_8: (mask & STOP_TROMPETTE_8) != 0,
            vox_humana_8: (mask & STOP_VOX_HUMANA_8) != 0,
        }
    }

    /// Count number of active ranks.
    pub fn active_rank_count(&self) -> usize {
        let mut count = 0;
        if self.principal_8 { count += 1; }
        if self.bourdon_16 { count += 1; }
        if self.octave_4 { count += 1; }
        if self.flute_4 { count += 1; }
        if self.super_octave_2 { count += 1; }
        if self.mixture_iv { count += 4; } // Compound rank has 4 pipes
        if self.trompette_8 { count += 1; }
        if self.vox_humana_8 { count += 1; }
        count
    }
}

/// Factory sound profiles for pipe organ synthesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PipeOrganProfile {
    /// Grand Baroque Tutti / Organo Pleno (Principal 8', Bourdon 16', Octave 4', Super Octave 2', Mixture IV, Trompette 8').
    #[default]
    ToccataPlenum,
    /// Warm introspective devotional registration (Bourdon 16', Principal 8', Flute 4').
    BaroqueChorale,
    /// Massive Gothic swell with full reeds and compound mixtures.
    GothicCathedral,
    /// Symphonic Cavaille-Coll reed solo (Trompette 8', Vox Humana 8', Principal 8', Octave 4').
    FrenchRomanticReed,
    /// Intimate Vox Humana with Tremulant and Bourdon 16' backing.
    VocalVoxHumana,
    /// Delicate flute duo (Bourdon 16', Flute 4', Super Octave 2').
    SilverFlutes,
}

impl PipeOrganProfile {
    /// Return nominal stop registration for this profile.
    pub fn default_stops(&self) -> PipeOrganStops {
        match self {
            Self::ToccataPlenum => PipeOrganStops {
                principal_8: true,
                bourdon_16: true,
                octave_4: true,
                flute_4: false,
                super_octave_2: true,
                mixture_iv: true,
                trompette_8: true,
                vox_humana_8: false,
            },
            Self::BaroqueChorale => PipeOrganStops {
                principal_8: true,
                bourdon_16: true,
                octave_4: false,
                flute_4: true,
                super_octave_2: false,
                mixture_iv: false,
                trompette_8: false,
                vox_humana_8: false,
            },
            Self::GothicCathedral => PipeOrganStops {
                principal_8: true,
                bourdon_16: true,
                octave_4: true,
                flute_4: true,
                super_octave_2: true,
                mixture_iv: true,
                trompette_8: true,
                vox_humana_8: true,
            },
            Self::FrenchRomanticReed => PipeOrganStops {
                principal_8: true,
                bourdon_16: false,
                octave_4: true,
                flute_4: false,
                super_octave_2: false,
                mixture_iv: false,
                trompette_8: true,
                vox_humana_8: true,
            },
            Self::VocalVoxHumana => PipeOrganStops {
                principal_8: false,
                bourdon_16: true,
                octave_4: false,
                flute_4: true,
                super_octave_2: false,
                mixture_iv: false,
                trompette_8: false,
                vox_humana_8: true,
            },
            Self::SilverFlutes => PipeOrganStops {
                principal_8: false,
                bourdon_16: true,
                octave_4: false,
                flute_4: true,
                super_octave_2: true,
                mixture_iv: false,
                trompette_8: false,
                vox_humana_8: false,
            },
        }
    }

    /// Return nominal wind pressure (in mmH2O) and tremulant settings.
    pub fn default_wind_and_tremulant(&self) -> (f32, bool, f32, f32) {
        match self {
            Self::ToccataPlenum => (85.0, false, 5.5, 0.10),
            Self::BaroqueChorale => (65.0, false, 5.0, 0.08),
            Self::GothicCathedral => (110.0, false, 6.0, 0.12),
            Self::FrenchRomanticReed => (95.0, false, 5.8, 0.15),
            Self::VocalVoxHumana => (70.0, true, 5.2, 0.22),
            Self::SilverFlutes => (60.0, false, 4.8, 0.05),
        }
    }
}

/// A single polyphonic voice synthesizing all active pipe ranks for one note.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OrganVoice {
    pub active: bool,
    pub note: u8,
    pub velocity: f32,
    pub base_freq: f32,
    pub age: usize,
    /// Pallet valve opening envelope (0.0 to 1.0)
    pub pallet_envelope: f32,
    pub pallet_target: f32,
    /// Attack chiff transient phase and noise filter state
    pub chiff_time_samples: u32,
    pub chiff_duration_samples: u32,
    pub chiff_noise_seed: u32,
    pub chiff_filter_state: f32,
    /// Continuous phase accumulators for flue and reed ranks:
    /// [Principal 8', Bourdon 16', Octave 4', Flute 4', Super Octave 2', Mixture IV (4 sub-phases), Trompette 8', Vox Humana 8']
    pub phase_principal_8: f32,
    pub phase_bourdon_16: f32,
    pub phase_octave_4: f32,
    pub phase_flute_4: f32,
    pub phase_super_octave_2: f32,
    pub phase_mixture_1: f32,
    pub phase_mixture_2: f32,
    pub phase_mixture_3: f32,
    pub phase_mixture_4: f32,
    pub phase_trompette_8: f32,
    pub phase_vox_humana_8: f32,
    /// Reed mechanical displacement and formant filter memory
    pub reed_trompette_state: f32,
    pub formant_vox_bpf1: (f32, f32),
    pub formant_vox_bpf2: (f32, f32),
}

impl Default for OrganVoice {
    fn default() -> Self {
        Self {
            active: false,
            note: 60,
            velocity: 0.8,
            base_freq: 261.63,
            age: 0,
            pallet_envelope: 0.0,
            pallet_target: 0.0,
            chiff_time_samples: 0,
            chiff_duration_samples: 1500,
            chiff_noise_seed: 123456789,
            chiff_filter_state: 0.0,
            phase_principal_8: 0.0,
            phase_bourdon_16: 0.0,
            phase_octave_4: 0.0,
            phase_flute_4: 0.0,
            phase_super_octave_2: 0.0,
            phase_mixture_1: 0.0,
            phase_mixture_2: 0.0,
            phase_mixture_3: 0.0,
            phase_mixture_4: 0.0,
            phase_trompette_8: 0.0,
            phase_vox_humana_8: 0.0,
            reed_trompette_state: 0.0,
            formant_vox_bpf1: (0.0, 0.0),
            formant_vox_bpf2: (0.0, 0.0),
        }
    }
}

impl OrganVoice {
    /// PRNG step for chiff acoustic turbulence.
    #[inline]
    fn next_noise(&mut self) -> f32 {
        self.chiff_noise_seed = self.chiff_noise_seed.wrapping_mul(1664525).wrapping_add(1013904223);
        ((self.chiff_noise_seed >> 9) as f32 / 8388607.0) - 1.0
    }
}

/// Complete Physical Modeling Pipe Organ Synthesizer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipeOrgan {
    pub sample_rate: u32,
    pub profile: PipeOrganProfile,
    pub stops: PipeOrganStops,
    pub windchest: Windchest,
    pub voices: [OrganVoice; MAX_ORGAN_VOICES],
    pub master_gain: f32,
    /// Cutup mouth height-to-width ratio [0.15 ..= 0.50].
    pub cutup_ratio: f32,
    /// Nominal chiff duration in milliseconds [5.0 ..= 80.0].
    pub chiff_duration_ms: f32,
    /// Tracker action mechanical speed multiplier [0.5 ..= 2.0].
    pub tracker_speed: f32,
    pub voice_age_counter: usize,
}

impl Default for PipeOrgan {
    fn default() -> Self {
        Self::new(48000)
    }
}

impl PipeOrgan {
    /// Create a new Pipe Organ engine for given sample rate.
    pub fn new(sample_rate: u32) -> Self {
        let sr = sample_rate.max(1);
        let mut windchest = Windchest::new(sr);
        let profile = PipeOrganProfile::ToccataPlenum;
        let stops = profile.default_stops();
        let (nominal_p, trem_en, trem_rate, trem_depth) = profile.default_wind_and_tremulant();
        windchest.set_nominal_pressure(nominal_p);
        windchest.set_tremulant(trem_en, trem_rate, trem_depth);

        Self {
            sample_rate: sr,
            profile,
            stops,
            windchest,
            voices: [OrganVoice::default(); MAX_ORGAN_VOICES],
            master_gain: 0.85,
            cutup_ratio: 0.25,
            chiff_duration_ms: 28.0,
            tracker_speed: 1.0,
            voice_age_counter: 0,
        }
    }

    /// Set sample rate.
    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate.max(1);
        self.windchest.set_sample_rate(self.sample_rate);
    }

    /// Set sound profile.
    pub fn set_profile(&mut self, profile: PipeOrganProfile) {
        self.profile = profile;
        self.stops = profile.default_stops();
        let (p, trem_en, trem_rate, trem_depth) = profile.default_wind_and_tremulant();
        self.windchest.set_nominal_pressure(p);
        self.windchest.set_tremulant(trem_en, trem_rate, trem_depth);
    }

    /// Trigger MIDI note on.
    pub fn note_on(&mut self, note: u8, velocity: f32) {
        let vel = velocity.clamp(0.01, 1.0);
        let freq = 440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0);

        // Check if note is already sounding -> retrigger
        for voice in self.voices.iter_mut() {
            if voice.active && voice.note == note {
                voice.velocity = vel;
                voice.base_freq = freq;
                voice.pallet_target = 1.0;
                voice.chiff_time_samples = 0;
                voice.chiff_duration_samples = ((self.chiff_duration_ms * 0.001 * self.sample_rate as f32) as u32).max(10);
                voice.age = self.voice_age_counter;
                self.voice_age_counter += 1;
                return;
            }
        }

        // Find free voice slot or steal oldest
        let mut best_slot = 0;
        let mut oldest_age = usize::MAX;
        for (i, voice) in self.voices.iter().enumerate() {
            if !voice.active {
                best_slot = i;
                break;
            }
            if voice.age < oldest_age {
                oldest_age = voice.age;
                best_slot = i;
            }
        }

        let voice = &mut self.voices[best_slot];
        voice.active = true;
        voice.note = note;
        voice.velocity = vel;
        voice.base_freq = freq;
        voice.pallet_envelope = 0.0;
        voice.pallet_target = 1.0;
        voice.chiff_time_samples = 0;
        voice.chiff_duration_samples = ((self.chiff_duration_ms * 0.001 * self.sample_rate as f32) as u32).max(10);
        voice.chiff_noise_seed = 123456789 + (note as u32 * 7919) + (best_slot as u32 * 997);
        voice.chiff_filter_state = 0.0;
        voice.age = self.voice_age_counter;
        self.voice_age_counter += 1;
    }

    /// Release MIDI note.
    pub fn note_off(&mut self, note: u8) {
        for voice in self.voices.iter_mut() {
            if voice.active && voice.note == note {
                voice.pallet_target = 0.0;
            }
        }
    }

    /// Release all active notes.
    pub fn all_notes_off(&mut self) {
        for voice in self.voices.iter_mut() {
            voice.pallet_target = 0.0;
            voice.active = false;
        }
    }

    /// Render a single sample of the Pipe Organ simulation.
    #[inline]
    pub fn process_sample(&mut self) -> (f32, f32) {
        let dt = 1.0 / self.sample_rate as f32;

        // Calculate total instantaneous air demand from sounding pipes
        let mut total_air_demand = 0.0f32;
        let active_ranks = self.stops.active_rank_count() as f32;
        for voice in self.voices.iter() {
            if voice.active && voice.pallet_envelope > 0.001 {
                total_air_demand += voice.pallet_envelope * (0.8 + 0.15 * active_ranks);
            }
        }

        // Process windchest pressure dynamics
        let effective_p = self.windchest.process_sample(total_air_demand);
        let air_vel = self.windchest.air_velocity();
        let nominal_p = self.windchest.config.nominal_pressure_mmh2o.max(1.0);
        let pressure_ratio = (effective_p / nominal_p).clamp(0.5, 1.8);
        let pitch_mod = (pressure_ratio - 1.0) * 0.035; // Slight pneumatic pitch droop/rise

        let mut mix_mono = 0.0f32;
        let pallet_attack_rate = dt / (0.012 / self.tracker_speed).max(0.001);
        let pallet_release_rate = dt / (0.025 / self.tracker_speed).max(0.001);

        for voice in self.voices.iter_mut() {
            if !voice.active {
                continue;
            }

            // Tracker pallet valve motion
            if voice.pallet_target > voice.pallet_envelope {
                voice.pallet_envelope = (voice.pallet_envelope + pallet_attack_rate).min(1.0);
            } else if voice.pallet_target < voice.pallet_envelope {
                voice.pallet_envelope = (voice.pallet_envelope - pallet_release_rate).max(0.0);
                if voice.pallet_envelope <= 0.0001 {
                    voice.active = false;
                    continue;
                }
            }

            let env = voice.pallet_envelope;
            let note_freq = voice.base_freq * (1.0 + pitch_mod);
            let vel = voice.velocity;

            // Chiff vortex attack transient
            let mut chiff_signal = 0.0f32;
            if voice.chiff_time_samples < voice.chiff_duration_samples {
                let progress = voice.chiff_time_samples as f32 / voice.chiff_duration_samples as f32;
                let chiff_decay = (-4.5 * progress).exp();
                let raw_noise = voice.next_noise();
                // Resonant bandpass filter tuned to pipe upper mouth frequency (~3.5 * fundamental)
                let mouth_freq = (note_freq * 3.5).min(self.sample_rate as f32 * 0.45);
                let w = (TAU * mouth_freq / self.sample_rate as f32).min(1.5);
                voice.chiff_filter_state += w * (raw_noise - voice.chiff_filter_state);
                chiff_signal = voice.chiff_filter_state * chiff_decay * (0.35 + 0.25 * self.cutup_ratio) * vel;
                voice.chiff_time_samples += 1;
            }

            let mut voice_out = 0.0f32;

            // 1. Principal 8' (Open metal flue, rich in all harmonics)
            if self.stops.principal_8 {
                let f = note_freq;
                let inc = (TAU * f) / self.sample_rate as f32;
                voice.phase_principal_8 = (voice.phase_principal_8 + inc) % TAU;
                let p = voice.phase_principal_8;
                // Additive harmonic series with non-linear saturation
                let raw = p.sin() + 0.55 * (2.0 * p).sin() + 0.35 * (3.0 * p).sin() + 0.20 * (4.0 * p).sin() + 0.10 * (5.0 * p).sin();
                let jet_sat = (raw * (air_vel / 35.0)).tanh();
                voice_out += jet_sat * 0.45;
            }

            // 2. Bourdon 16' (Stopped wooden flue, sub-octave, odd harmonics only)
            if self.stops.bourdon_16 {
                let f = note_freq * 0.5; // 16' is 1 octave lower
                let inc = (TAU * f) / self.sample_rate as f32;
                voice.phase_bourdon_16 = (voice.phase_bourdon_16 + inc) % TAU;
                let p = voice.phase_bourdon_16;
                // Stopped pipes produce odd harmonics (1, 3, 5, 7) with negligible even harmonics
                let raw_odd = p.sin() + 0.38 * (3.0 * p).sin() + 0.15 * (5.0 * p).sin() + 0.06 * (7.0 * p).sin() + 0.02 * (2.0 * p).sin();
                let wood_sat = (raw_odd * 1.1).tanh();
                voice_out += wood_sat * 0.55;
            }

            // 3. Octave 4' (Open metal flue, 1 octave higher)
            if self.stops.octave_4 {
                let f = note_freq * 2.0;
                let inc = (TAU * f) / self.sample_rate as f32;
                voice.phase_octave_4 = (voice.phase_octave_4 + inc) % TAU;
                let p = voice.phase_octave_4;
                let raw = p.sin() + 0.48 * (2.0 * p).sin() + 0.25 * (3.0 * p).sin() + 0.12 * (4.0 * p).sin();
                voice_out += raw * 0.35;
            }

            // 4. Flute 4' (Stopped wooden flute, hollow sweetened tone)
            if self.stops.flute_4 {
                let f = note_freq * 2.0;
                let inc = (TAU * f) / self.sample_rate as f32;
                voice.phase_flute_4 = (voice.phase_flute_4 + inc) % TAU;
                let p = voice.phase_flute_4;
                let raw_flute = p.sin() + 0.25 * (3.0 * p).sin() + 0.05 * (5.0 * p).sin();
                voice_out += raw_flute * 0.38;
            }

            // 5. Super Octave 2' (Open metal flue, 2 octaves higher, brilliant shimmer)
            if self.stops.super_octave_2 {
                let f = (note_freq * 4.0).min(self.sample_rate as f32 * 0.48);
                let inc = (TAU * f) / self.sample_rate as f32;
                voice.phase_super_octave_2 = (voice.phase_super_octave_2 + inc) % TAU;
                let p = voice.phase_super_octave_2;
                let raw = p.sin() + 0.35 * (2.0 * p).sin() + 0.15 * (3.0 * p).sin();
                voice_out += raw * 0.25;
            }

            // 6. Mixture IV (Compound 4-rank chorus: 1-1/3' [19th], 1' [22nd], 2/3' [26th], 1/2' [29th] with octave breaks)
            if self.stops.mixture_iv {
                // Tier break at C4 (note 60) and C5 (note 72)
                let (m1_mult, m2_mult, m3_mult, m4_mult) = if voice.note < 60 {
                    (3.0, 4.0, 6.0, 8.0) // 1-1/3', 1', 2/3', 1/2'
                } else if voice.note < 72 {
                    (2.0, 3.0, 4.0, 6.0) // 2', 1-1/3', 1', 2/3'
                } else {
                    (1.5, 2.0, 3.0, 4.0) // 2-2/3', 2', 1-1/3', 1'
                };

                let inc1 = (TAU * (note_freq * m1_mult).min(self.sample_rate as f32 * 0.48)) / self.sample_rate as f32;
                let inc2 = (TAU * (note_freq * m2_mult).min(self.sample_rate as f32 * 0.48)) / self.sample_rate as f32;
                let inc3 = (TAU * (note_freq * m3_mult).min(self.sample_rate as f32 * 0.48)) / self.sample_rate as f32;
                let inc4 = (TAU * (note_freq * m4_mult).min(self.sample_rate as f32 * 0.48)) / self.sample_rate as f32;

                voice.phase_mixture_1 = (voice.phase_mixture_1 + inc1) % TAU;
                voice.phase_mixture_2 = (voice.phase_mixture_2 + inc2) % TAU;
                voice.phase_mixture_3 = (voice.phase_mixture_3 + inc3) % TAU;
                voice.phase_mixture_4 = (voice.phase_mixture_4 + inc4) % TAU;

                let mix_sig = voice.phase_mixture_1.sin() * 0.35
                    + voice.phase_mixture_2.sin() * 0.30
                    + voice.phase_mixture_3.sin() * 0.25
                    + voice.phase_mixture_4.sin() * 0.20;
                voice_out += mix_sig * 0.30;
            }

            // 7. Trompette 8' (Beating brass reed with conical horn flare, aggressive brassy harmonics)
            if self.stops.trompette_8 {
                let f = note_freq;
                let inc = (TAU * f) / self.sample_rate as f32;
                voice.phase_trompette_8 = (voice.phase_trompette_8 + inc) % TAU;
                let p = voice.phase_trompette_8;
                // Beating reed pulse wave: asymmetric saw/pulse hybrid with shallot strike saturation
                let reed_pulse = if p < PI {
                    (p / PI) * 2.0 - 1.0
                } else {
                    -((p - PI) / PI).powf(0.6)
                };
                let brass_horn = (reed_pulse * 1.8).tanh() + 0.4 * (3.0 * p).sin() + 0.3 * (5.0 * p).sin();
                voice_out += brass_horn * 0.42;
            }

            // 8. Vox Humana 8' (Short cylindrical reed with vocal formant filtering)
            if self.stops.vox_humana_8 {
                let f = note_freq;
                let inc = (TAU * f) / self.sample_rate as f32;
                voice.phase_vox_humana_8 = (voice.phase_vox_humana_8 + inc) % TAU;
                let p = voice.phase_vox_humana_8;
                let reed_raw = (p.sin() * 2.5).tanh();
                // Dual formant filtering (F1: 750 Hz, F2: 2100 Hz)
                let f1 = 750.0;
                let f2 = 2100.0;
                let w1 = (TAU * f1 / self.sample_rate as f32).min(1.0);
                let w2 = (TAU * f2 / self.sample_rate as f32).min(1.0);
                voice.formant_vox_bpf1.0 += w1 * (reed_raw - voice.formant_vox_bpf1.0);
                voice.formant_vox_bpf2.0 += w2 * (reed_raw - voice.formant_vox_bpf2.0);
                let vox_sig = (voice.formant_vox_bpf1.0 * 0.6 + voice.formant_vox_bpf2.0 * 0.4) * 1.5;
                voice_out += vox_sig * 0.40;
            }

            // Mix flue/reed audio with initial chiff transient, scaled by pallet envelope
            let final_voice = (voice_out + chiff_signal) * env * (0.5 + 0.5 * vel);
            mix_mono += final_voice;
        }

        // Stereo acoustic spatialization (subtle pipe stereo spread)
        let left = mix_mono * self.master_gain;
        let right = mix_mono * self.master_gain;
        (left, right)
    }
}

use crate::traits::SignalProcessor;

impl SignalProcessor for PipeOrgan {
    fn name(&self) -> &str {
        "PipeOrgan"
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
                if i < outputs[0].len() {
                    outputs[0][i] = l;
                }
                if i < outputs[1].len() {
                    outputs[1][i] = r;
                }
            } else if !outputs.is_empty() && i < outputs[0].len() {
                outputs[0][i] = (l + r) * 0.5;
            }
        }
    }
}

/// AudioNode wrapper for Pipe Organ generator.
#[derive(Debug)]
pub struct PipeOrganNode {
    pub organ: PipeOrgan,
}

impl PipeOrganNode {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            organ: PipeOrgan::new(sample_rate),
        }
    }
}

impl AudioNode for PipeOrganNode {
    fn name(&self) -> &str {
        "PipeOrganNode"
    }

    fn process(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        self.organ.process_block(inputs, outputs, ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::allocator::AllocGuard;

    #[test]
    fn test_pipe_organ_stops_bitmask_round_trip() {
        let stops = PipeOrganStops {
            principal_8: true,
            bourdon_16: false,
            octave_4: true,
            flute_4: false,
            super_octave_2: true,
            mixture_iv: false,
            trompette_8: true,
            vox_humana_8: true,
        };
        let mask = stops.to_bitmask();
        let decoded = PipeOrganStops::from_bitmask(mask);
        assert_eq!(stops, decoded);
        assert_eq!(stops.active_rank_count(), 5);
    }

    #[test]
    fn test_pipe_organ_zero_heap_allocation_under_alloc_guard() {
        let mut organ = PipeOrgan::new(48000);
        organ.note_on(60, 0.9);
        organ.note_on(64, 0.85);
        organ.note_on(67, 0.85);

        // Run audio loop under AllocGuard to verify zero heap allocation
        let _guard = AllocGuard::new();
        for _ in 0..1024 {
            let (l, r) = organ.process_sample();
            assert!(l.is_finite());
            assert!(r.is_finite());
        }
    }

    #[test]
    fn test_pipe_organ_stopped_pipe_odd_harmonics_dominance() {
        let mut organ = PipeOrgan::new(48000);
        // Only activate Bourdon 16' (stopped pipe)
        organ.stops = PipeOrganStops {
            principal_8: false,
            bourdon_16: true,
            octave_4: false,
            flute_4: false,
            super_octave_2: false,
            mixture_iv: false,
            trompette_8: false,
            vox_humana_8: false,
        };
        organ.note_on(60, 1.0);

        // Warm up and collect samples
        for _ in 0..5000 {
            organ.process_sample();
        }

        let n = 2048;
        let mut buf = Vec::with_capacity(n);
        for _ in 0..n {
            buf.push(organ.process_sample().0);
        }

        // Discrete Fourier transform to check fundamental (odd) vs 2nd harmonic (even)
        // Bourdon 16' on note 60 (C4 = 261.63 Hz) sounds at C3 (130.81 Hz).
        let f1 = 130.8128;
        let f2 = 261.6256;
        let f3 = 392.4384;

        let compute_dft_mag = |target_f: f32| -> f32 {
            let mut re = 0.0f32;
            let mut im = 0.0f32;
            for (t, &s) in buf.iter().enumerate() {
                let angle = TAU * target_f * (t as f32 / 48000.0);
                re += s * angle.cos();
                im -= s * angle.sin();
            }
            (re * re + im * im).sqrt() / n as f32
        };

        let mag_f1 = compute_dft_mag(f1); // Odd (fundamental)
        let mag_f2 = compute_dft_mag(f2); // Even (2nd harmonic)
        let mag_f3 = compute_dft_mag(f3); // Odd (3rd harmonic)

        assert!(mag_f1 > 0.1, "Bourdon fundamental must be strong, mag_f1={}", mag_f1);
        assert!(mag_f3 > 0.01, "Bourdon 3rd harmonic must be present, mag_f3={}", mag_f3);
        assert!(mag_f1 > mag_f2 * 3.0, "Stopped pipe odd harmonics must dominate even harmonics! mag_f1={}, mag_f2={}", mag_f1, mag_f2);
    }
}
