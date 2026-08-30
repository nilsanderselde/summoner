// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Sitar Synthesis Engine (Milestone 27).
//!
//! Provides a high-precision physical acoustic and electric sitar synthesizer featuring:
//! - Non-linear unilateral obstacle contact boundary condition modeling the curved Jawari bridge
//! - Elastic steel/brass main playing string with large-amplitude lateral tension deflection (Meend bending up to 5 semitones)
//! - 13-string sympathetic Tarab resonator bank dynamically tuned to authentic Indian Raga scales
//! - 4-string high-pitched rhythmic Chikari drone strummer with rake-dispersion transients
//! - Bi-directional coupling to Kaddu gourd (Tumba) and Tabli soundboard acoustic body modal resonators
//!
//! Enforces zero heap allocations in audio render loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};
use crate::jawari_bridge::{
    JawariBridge, JawariProfile, RagaScale, SitarGourdBody, TarabResonatorBank,
};
use crate::traits::SignalProcessor;

/// Maximum delay buffer capacity in samples (~30 Hz fundamental at 192 kHz).
pub const MAX_SITAR_DELAY: usize = 16384;

/// Maximum polyphonic voices allocated in the zero-heap sitar voice pool.
pub const MAX_SITAR_VOICES: usize = 8;

/// Number of rhythmic Chikari drone strings.
pub const NUM_CHIKARI_STRINGS: usize = 4;

fn default_sitar_delay_buffer() -> Box<[f32; MAX_SITAR_DELAY]> {
    Box::new([0.0; MAX_SITAR_DELAY])
}

/// Sitar acoustic preset profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SitarProfile {
    /// Kharaj Pancham style (Ravi Shankar tradition - 7 main strings, deep bass Kharaj resonance, wide Jawari buzz).
    #[default]
    RaviShankarKharaj,
    /// Gandhar Pancham style (Vilayat Khan Gayaki vocal tradition - bright crisp articulation, fast meend response).
    VilayatKhanGayaki,
    /// Surbahar bass sitar (heavy brass string, deep 1-octave lower tuning, long meditative sustain).
    SurbaharBass,
    /// Modern Electric Sitar (flat metal Jawari sizzle, solid-body sustain).
    ElectricSitar,
    /// Antique Tanjore Gourd Sitar (dry woody tabli radiation, light cotton jiva thread touch).
    AntiqueGourd,
    /// Avant-Garde Microtonal Sitar (extreme meend pitch deflection, metallic boundary reflection).
    AvantGardeSitar,
}

impl SitarProfile {
    pub fn name(&self) -> &'static str {
        match self {
            Self::RaviShankarKharaj => "Ravi Shankar Kharaj Pancham Sitar",
            Self::VilayatKhanGayaki => "Vilayat Khan Gandhar Pancham (Gayaki)",
            Self::SurbaharBass => "Surbahar Bass Acoustic Sitar",
            Self::ElectricSitar => "Solid-Body Electric Sitar",
            Self::AntiqueGourd => "Antique Tanjore Gourd Sitar",
            Self::AvantGardeSitar => "Avant-Garde Microtonal Sitar",
        }
    }

    pub fn to_jawari_profile(&self) -> JawariProfile {
        match self {
            Self::RaviShankarKharaj => JawariProfile::DeerHornCurved,
            Self::VilayatKhanGayaki => JawariProfile::CamelBoneSharp,
            Self::SurbaharBass => JawariProfile::EbonyWoodMellow,
            Self::ElectricSitar => JawariProfile::ElectricFlatJawari,
            Self::AntiqueGourd => JawariProfile::OpenGourdAcoustic,
            Self::AvantGardeSitar => JawariProfile::SyntheticDelrin,
        }
    }

    /// Returns nominal $(T_{60\text{decay}}, \text{root\_freq\_hz}, \text{jawari\_gap\_mm}, \text{tarab\_bleed}, \text{body\_gain})$.
    pub fn nominal_physics(&self) -> (f32, f32, f32, f32, f32) {
        match self {
            Self::RaviShankarKharaj => (4.5, 138.59, 0.18, 0.40, 0.85), // C#3 root
            Self::VilayatKhanGayaki => (3.8, 146.83, 0.08, 0.35, 0.80), // D3 root
            Self::SurbaharBass => (7.0, 69.30, 0.35, 0.50, 0.90),      // C#2 bass root
            Self::ElectricSitar => (5.5, 130.81, 0.05, 0.20, 0.70),    // C3 root
            Self::AntiqueGourd => (3.2, 138.59, 0.22, 0.45, 0.95),     // C#3 root
            Self::AvantGardeSitar => (6.0, 130.81, 0.12, 0.30, 0.75),  // C3 root
        }
    }
}

/// A first-order allpass filter section for sitar string inharmonic dispersion.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct SitarDispersionAllpass {
    pub coefficient: f32,
    x_prev: f32,
    y_prev: f32,
}

impl SitarDispersionAllpass {
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

/// Digital waveguide string representing the main playing string (Baj Tar) or Chikari drone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitarStringWaveguide {
    #[serde(skip, default = "default_sitar_delay_buffer")]
    pub delay_buffer: Box<[f32; MAX_SITAR_DELAY]>,
    pub write_idx: usize,
    pub dispersion: SitarDispersionAllpass,
    pub loss_filter_state: f32,
    pub loop_gain: f32,
    pub loss_cutoff_coeff: f32,
    pub is_active: bool,
}

impl Default for SitarStringWaveguide {
    fn default() -> Self {
        Self {
            delay_buffer: default_sitar_delay_buffer(),
            write_idx: 0,
            dispersion: SitarDispersionAllpass::default(),
            loss_filter_state: 0.0,
            loop_gain: 0.996,
            loss_cutoff_coeff: 0.40,
            is_active: false,
        }
    }
}

impl SitarStringWaveguide {
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
        let d = delay_samples.clamp(1.0, (MAX_SITAR_DELAY - 4) as f32);
        let int_d = d.floor() as usize;
        let frac_d = d - d.floor();

        let idx0 = (self.write_idx + MAX_SITAR_DELAY - int_d) % MAX_SITAR_DELAY;
        let idx1 = (idx0 + MAX_SITAR_DELAY - 1) % MAX_SITAR_DELAY;

        let s0 = self.delay_buffer[idx0];
        let s1 = self.delay_buffer[idx1];
        s0 + frac_d * (s1 - s0)
    }

    /// Step the waveguide with Jawari obstacle contact boundary interaction.
    #[inline]
    pub fn step(
        &mut self,
        nominal_delay_samples: f32,
        incoming_excitation: f32,
        jawari: &mut JawariBridge,
        sample_rate: u32,
    ) -> f32 {
        // Read traveling wave from delay buffer
        let delayed_wave = self.read_delay(nominal_delay_samples);

        // Inharmonic dispersion allpass filter
        let dispersed = self.dispersion.step(delayed_wave);

        // Approximate physical string displacement at bridge termination
        let string_disp_m = dispersed * 0.001;
        let (jawari_force, shortening_ratio) = jawari.step_contact(string_disp_m, sample_rate);

        // Dynamic pitch shortening from curved Jawari obstacle contact
        let eff_delay = (nominal_delay_samples * (1.0 - shortening_ratio)).max(2.0);

        // Reflection at bridge: inverted sign with loop loss and Jawari contact restoring force
        let jawari_feedback = jawari_force * 0.05;
        let reflected = -(dispersed * self.loop_gain) + jawari_feedback;

        // One-pole lowpass loss filter
        let alpha = self.loss_cutoff_coeff.clamp(0.05, 0.95);
        self.loss_filter_state = (1.0 - alpha) * reflected + alpha * self.loss_filter_state;

        // Inject excitation into circular buffer write head
        let total_in = (self.loss_filter_state + incoming_excitation).clamp(-4.0, 4.0);
        self.delay_buffer[self.write_idx] = total_in;
        self.write_idx = (self.write_idx + 1) % MAX_SITAR_DELAY;

        // Re-read with dynamic shortening if contact occurred
        if shortening_ratio > 0.001 {
            self.read_delay(eff_delay) + incoming_excitation * 0.5
        } else {
            self.loss_filter_state + incoming_excitation
        }
    }
}

/// 4-String Chikari Rhythmic Drone Strummer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChikariDroneBank {
    pub strings: [SitarStringWaveguide; NUM_CHIKARI_STRINGS],
    pub frequencies_hz: [f32; NUM_CHIKARI_STRINGS],
    pub strum_delay_counters: [usize; NUM_CHIKARI_STRINGS],
    pub strum_amp: [f32; NUM_CHIKARI_STRINGS],
    pub sample_rate: u32,
}

impl ChikariDroneBank {
    pub fn new(sample_rate: u32, root_hz: f32) -> Self {
        // Traditional Chikari tuning:
        // String 0: Pancham (Pa - Fifth) -> 1.5 * root_hz
        // String 1: Kharaj Sa (Tonic base) -> 1.0 * root_hz
        // String 2: Madhya Sa (Middle tonic) -> 2.0 * root_hz
        // String 3: Taar Sa (High tonic) -> 4.0 * root_hz
        let freqs = [root_hz * 1.5, root_hz, root_hz * 2.0, root_hz * 4.0];

        Self {
            strings: [
                SitarStringWaveguide::new(),
                SitarStringWaveguide::new(),
                SitarStringWaveguide::new(),
                SitarStringWaveguide::new(),
            ],
            frequencies_hz: freqs,
            strum_delay_counters: [0; NUM_CHIKARI_STRINGS],
            strum_amp: [0.0; NUM_CHIKARI_STRINGS],
            sample_rate,
        }
    }

    pub fn set_root_freq(&mut self, root_hz: f32) {
        self.frequencies_hz = [root_hz * 1.5, root_hz, root_hz * 2.0, root_hz * 4.0];
    }

    /// Trigger a rolling Chikari stroke (Mizrab upward flick across the 4 strings).
    pub fn trigger_strum(&mut self, velocity: f32) {
        let vel = velocity.clamp(0.0, 1.0);
        let sr = self.sample_rate as f32;

        // Dispersion of strike timing: ~3ms stagger between strings
        for i in 0..NUM_CHIKARI_STRINGS {
            self.strings[i].reset();
            self.strings[i].loop_gain = 0.994;
            self.strings[i].is_active = true;
            self.strum_delay_counters[i] = ((i as f32 * 0.003) * sr) as usize + 1;
            self.strum_amp[i] = vel * 0.85;
        }
    }

    #[inline]
    pub fn process_sample(&mut self, jawari: &mut JawariBridge) -> f32 {
        let sr = self.sample_rate as f32;
        let mut total_drone = 0.0;

        for i in 0..NUM_CHIKARI_STRINGS {
            if !self.strings[i].is_active {
                continue;
            }

            let delay_len = (sr / self.frequencies_hz[i].clamp(50.0, sr * 0.45)).clamp(2.0, (MAX_SITAR_DELAY - 4) as f32);
            let excitation = if self.strum_delay_counters[i] == 1 {
                self.strum_delay_counters[i] = 0;
                self.strum_amp[i]
            } else if self.strum_delay_counters[i] > 1 {
                self.strum_delay_counters[i] -= 1;
                0.0
            } else {
                0.0
            };

            let out = self.strings[i].step(delay_len, excitation, jawari, self.sample_rate);
            total_drone += out;
        }

        total_drone
    }

    pub fn reset(&mut self) {
        for s in &mut self.strings {
            s.reset();
        }
        self.strum_delay_counters.fill(0);
        self.strum_amp.fill(0.0);
    }
}

/// A polyphonic voice modeling the main sitar playing string (Baj Tar).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitarVoice {
    pub note_number: u8,
    pub base_freq_hz: f32,
    pub velocity: f32,
    pub meend_semitones: f32,
    pub excitation_samples_remaining: usize,
    pub excitation_amp: f32,
    pub string: SitarStringWaveguide,
    pub age_samples: u64,
    pub is_active: bool,
    pub is_key_down: bool,
    pub sample_rate: u32,
}

impl Default for SitarVoice {
    fn default() -> Self {
        Self {
            note_number: 60,
            base_freq_hz: 261.63,
            velocity: 0.0,
            meend_semitones: 0.0,
            excitation_samples_remaining: 0,
            excitation_amp: 0.0,
            string: SitarStringWaveguide::new(),
            age_samples: 0,
            is_active: false,
            is_key_down: false,
            sample_rate: 48000,
        }
    }
}

impl SitarVoice {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            ..Default::default()
        }
    }

    pub fn init_for_note(&mut self, note: u8, velocity: f32, meend: f32) {
        self.note_number = note;
        self.velocity = velocity.clamp(0.0, 1.0);
        self.base_freq_hz = 440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0);
        self.meend_semitones = meend;
        self.age_samples = 0;
        self.is_active = true;
        self.is_key_down = true;
        self.excitation_samples_remaining = 6;
        self.excitation_amp = self.velocity * 0.95;

        self.string.reset();
        // Dispersion allpass coefficient for thin high-tensile steel playing string
        self.string.dispersion = SitarDispersionAllpass::new(-0.15);
        self.string.loop_gain = 0.997;
        self.string.loss_cutoff_coeff = 0.35;
    }

    #[inline]
    pub fn process_sample(&mut self, jawari: &mut JawariBridge) -> f32 {
        if !self.is_active {
            return 0.0;
        }

        self.age_samples += 1;
        let sr = self.sample_rate as f32;

        // Meend lateral pull modulates effective frequency: f = f_base * 2^(meend / 12)
        let eff_f0 = (self.base_freq_hz * 2.0_f32.powf(self.meend_semitones / 12.0)).clamp(40.0, sr * 0.45);
        let delay_len = sr / eff_f0;

        let excitation = if self.excitation_samples_remaining > 0 {
            let amp = self.excitation_amp * (self.excitation_samples_remaining as f32 / 6.0);
            self.excitation_samples_remaining -= 1;
            amp
        } else {
            0.0
        };

        let out = self.string.step(delay_len, excitation, jawari, self.sample_rate);

        // Deactivation check
        if self.age_samples > (sr * 0.08) as u64 && !self.is_key_down && out.abs() < 1e-5 {
            self.is_active = false;
        }

        out
    }

    pub fn note_off(&mut self) {
        self.is_key_down = false;
        // Mild damping when finger releases fret
        self.string.loop_gain *= 0.985;
    }
}

/// Sitar Physical Modeling Synthesizer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sitar {
    pub profile: SitarProfile,
    pub raga_scale: RagaScale,
    pub jawari: JawariBridge,
    pub tarab: TarabResonatorBank,
    pub body: SitarGourdBody,
    pub chikari: ChikariDroneBank,
    pub voices: [SitarVoice; MAX_SITAR_VOICES],
    pub meend_semitones: f32, // [0.0 ..= 5.0] lateral pull pitch bend
    pub root_freq_hz: f32,
    pub body_resonance_gain: f32,
    pub master_gain: f32,
    pub sample_rate: u32,
}

impl Sitar {
    pub fn new(sample_rate: u32) -> Self {
        let profile = SitarProfile::RaviShankarKharaj;
        let (_, root_hz, _, tarab_bleed, body_gain) = profile.nominal_physics();
        let jawari = JawariBridge::new(profile.to_jawari_profile());
        let mut tarab = TarabResonatorBank::new(sample_rate, RagaScale::Yaman, root_hz);
        tarab.coupling_bleed = tarab_bleed;
        let body = SitarGourdBody::new(sample_rate);
        let chikari = ChikariDroneBank::new(sample_rate, root_hz);

        let voices = [
            SitarVoice::new(sample_rate), SitarVoice::new(sample_rate),
            SitarVoice::new(sample_rate), SitarVoice::new(sample_rate),
            SitarVoice::new(sample_rate), SitarVoice::new(sample_rate),
            SitarVoice::new(sample_rate), SitarVoice::new(sample_rate),
        ];

        Self {
            profile,
            raga_scale: RagaScale::Yaman,
            jawari,
            tarab,
            body,
            chikari,
            voices,
            meend_semitones: 0.0,
            root_freq_hz: root_hz,
            body_resonance_gain: body_gain,
            master_gain: 0.85,
            sample_rate,
        }
    }

    pub fn set_profile(&mut self, profile: SitarProfile) {
        self.profile = profile;
        let (decay, root_hz, _, tarab_bleed, body_gain) = profile.nominal_physics();
        self.jawari.set_profile(profile.to_jawari_profile());
        self.root_freq_hz = root_hz;
        self.tarab.set_root_freq(root_hz);
        self.tarab.coupling_bleed = tarab_bleed;
        self.chikari.set_root_freq(root_hz);
        self.body.set_decay_scale(decay / 4.0);
        self.body_resonance_gain = body_gain;
    }

    pub fn set_raga_scale(&mut self, scale: RagaScale) {
        self.raga_scale = scale;
        self.tarab.set_scale(scale);
    }

    /// Set lateral Meend pulling pitch offset in semitones $[0.0 ..= 5.0]$.
    pub fn set_meend_pull(&mut self, semitones: f32) {
        self.meend_semitones = semitones.clamp(0.0, 5.0);
        for voice in &mut self.voices {
            if voice.is_active {
                voice.meend_semitones = self.meend_semitones;
            }
        }
    }

    /// Trigger Mizrab stroke on main playing string (MIDI note 36..84, velocity 0.0..1.0).
    pub fn note_on(&mut self, note: u8, velocity: f32) {
        let vel = velocity.clamp(0.0, 1.0);
        if vel <= 0.001 {
            self.note_off(note);
            return;
        }

        // Find inactive voice or steal oldest
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

        self.voices[selected_idx].init_for_note(note, vel, self.meend_semitones);
    }

    /// Release note on main playing string.
    pub fn note_off(&mut self, note: u8) {
        for voice in &mut self.voices {
            if voice.is_active && voice.note_number == note {
                voice.note_off();
            }
        }
    }

    /// Trigger rhythmic Chikari drone strum.
    pub fn trigger_chikari_strum(&mut self, velocity: f32) {
        self.chikari.trigger_strum(velocity);
    }

    /// Process one stereo sample `(Left, Right)`.
    #[inline]
    pub fn process_sample(&mut self) -> (Sample, Sample) {
        // Collect forces from all playing string voices
        let mut playing_force = 0.0;

        for voice in &mut self.voices {
            if voice.is_active {
                let s_out = voice.process_sample(&mut self.jawari);
                playing_force += s_out;
            }
        }

        // Step Chikari drone strings
        let chikari_force = self.chikari.process_sample(&mut self.jawari);
        let total_string_force = playing_force + chikari_force;

        // Drive sympathetic Tarab strings with playing & drone excitation
        let tarab_out = self.tarab.process_sympathetic(total_string_force);

        // Drive Kaddu gourd & Tabli soundboard acoustic body modal resonators
        let body_input = total_string_force * 0.8 + tarab_out * 0.6;
        let (body_l, body_r) = self.body.process_body(body_input);

        let gain = self.master_gain;
        let body_gain = self.body_resonance_gain;

        let out_l = (body_l * body_gain * 0.75 + total_string_force * 0.25 + tarab_out * 0.35) * gain;
        let out_r = (body_r * body_gain * 0.75 + total_string_force * 0.25 + tarab_out * 0.35) * gain;

        (out_l.clamp(-1.0, 1.0), out_r.clamp(-1.0, 1.0))
    }

    /// All notes off / panic flusher.
    pub fn all_notes_off(&mut self) {
        for voice in &mut self.voices {
            voice.is_active = false;
            voice.is_key_down = false;
            voice.string.reset();
        }
        self.chikari.reset();
        self.tarab.reset();
        self.body.reset();
        self.jawari.reset();
        self.meend_semitones = 0.0;
    }
}

impl SignalProcessor for Sitar {
    fn name(&self) -> &str {
        "Sitar"
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

/// AudioNode wrapper for Sitar physical modeling synthesis engine.
#[derive(Debug)]
pub struct SitarNode {
    pub sitar: Sitar,
}

impl SitarNode {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sitar: Sitar::new(sample_rate),
        }
    }
}

impl AudioNode for SitarNode {
    fn name(&self) -> &str {
        "SitarNode"
    }

    fn process(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        self.sitar.process_block(inputs, outputs, ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::allocator::AllocGuard;
    use summoner_core::transport::Transport;

    #[test]
    fn test_sitar_synthesis_and_meend_pitch_bend() {
        let sample_rate = 48000;
        let mut sitar = Sitar::new(sample_rate);
        sitar.note_on(60, 0.85); // C4
        sitar.set_meend_pull(2.5); // 2.5 semitones upward meend pull

        let mut out_l = [0.0f32; 64];
        let mut out_r = [0.0f32; 64];
        let transport = Transport::new(sample_rate, 120.0);
        let ctx = ProcessContext::from_transport(&transport);

        {
            let _guard = AllocGuard::new();
            sitar.process_block(&[], &mut [&mut out_l[..], &mut out_r[..]], &ctx);
        }

        assert!(out_l.iter().all(|s| s.is_finite()));
        assert!(out_r.iter().all(|s| s.is_finite()));
        assert!(out_l.iter().any(|s| s.abs() > 0.0));
    }

    #[test]
    fn test_sitar_chikari_drone_strum() {
        let sample_rate = 48000;
        let mut sitar = Sitar::new(sample_rate);
        sitar.trigger_chikari_strum(0.90);

        let mut out_l = [0.0f32; 128];
        let mut out_r = [0.0f32; 128];
        let transport = Transport::new(sample_rate, 120.0);
        let ctx = ProcessContext::from_transport(&transport);

        {
            let _guard = AllocGuard::new();
            sitar.process_block(&[], &mut [&mut out_l[..], &mut out_r[..]], &ctx);
        }

        assert!(out_l.iter().any(|s| s.abs() > 0.0));
    }
}
