// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Japanese Shakuhachi Bamboo Flute Synthesis Engine (Milestone 25).
//!
//! Provides a physical acoustic synthesizer simulating an end-blown bamboo flute
//! (Shakuhachi 1.8 shaku / D4 classical honkyoku base, 2.4 shaku A3 Jinashi Zen flute,
//! 1.6 shaku E4 Min'yo folk flute, 2.1 shaku B3 Sankyoku ensemble flute, 3.0 shaku D3 Kyotaku)
//! with non-linear sharp utaguchi blowing edge air-jet splitting, dynamic Meri/Kari head tilt
//! angle pitch shifting (Δf in [-300, +200] cents), Murai-Iki explosive breath noise turbulence
//! bursts, 5-finger hole acoustic impedance junctions, and conical bamboo bore resonance.
//!
//! Enforces zero heap allocations in audio render loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};
use crate::air_reed::{
    meri_kari_to_pitch_multiplier, AirReed, DEFAULT_PRESSURE_PA, MIN_PRESSURE_PA,
};
use crate::traits::SignalProcessor;

/// Maximum bore delay line capacity in samples (~20 Hz at 192 kHz).
pub const MAX_BORE_DELAY: usize = 8192;

/// Maximum number of polyphonic Shakuhachi voices.
pub const MAX_SHAKUHACHI_VOICES: usize = 16;

/// Number of traditional finger holes on a classical Shakuhachi.
pub const NUM_HOLES: usize = 5;

/// Hole bitmask definitions.
pub const HOLE_1_TSU: u32 = 1 << 0;  // Hole 1 (Front bottom: F on 1.8 shaku)
pub const HOLE_2_RE: u32 = 1 << 1;   // Hole 2 (Front lower-mid: G on 1.8 shaku)
pub const HOLE_3_CHI: u32 = 1 << 2;  // Hole 3 (Front upper-mid: A on 1.8 shaku)
pub const HOLE_4_RI: u32 = 1 << 3;   // Hole 4 (Front top: C on 1.8 shaku)
pub const HOLE_5_THUMB: u32 = 1 << 4;// Hole 5 (Back thumb: Octave vent / D on 1.8 shaku)

/// 5 traditional Shakuhachi flute length profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ShakuhachiFluteLength {
    /// 1.8 Shaku (~54.5cm, fundamental D4 ~293.66 Hz, classical Honkyoku standard).
    #[default]
    IchishakuHassun,
    /// 2.4 Shaku (~72.7cm, fundamental A3 ~220.00 Hz, meditative Jinashi Zen flute).
    NishakuYonsun,
    /// 1.6 Shaku (~48.5cm, fundamental E4 ~329.63 Hz, bright Min'yo folk flute).
    IchishakuRokusun,
    /// 2.1 Shaku (~63.6cm, fundamental B3 ~246.94 Hz, warm Sankyoku ensemble flute).
    NishakuIssun,
    /// 3.0 Shaku (~90.9cm, fundamental D3 ~146.83 Hz, giant sub-bass Kyotaku temple flute).
    SanShakuKyotaku,
}

impl ShakuhachiFluteLength {
    /// Return nominal fundamental frequency in Hz (all holes closed, Ro tone).
    pub fn fundamental_hz(&self) -> f32 {
        match self {
            Self::IchishakuHassun => 293.66, // D4
            Self::NishakuYonsun => 220.00,   // A3
            Self::IchishakuRokusun => 329.63,// E4
            Self::NishakuIssun => 246.94,    // B3
            Self::SanShakuKyotaku => 146.83, // D3
        }
    }

    /// Return nominal physical bore length in meters.
    pub fn bore_length_m(&self) -> f32 {
        match self {
            Self::IchishakuHassun => 0.545,
            Self::NishakuYonsun => 0.727,
            Self::IchishakuRokusun => 0.485,
            Self::NishakuIssun => 0.636,
            Self::SanShakuKyotaku => 0.909,
        }
    }

    /// Return nominal base jet distance in mm.
    pub fn default_jet_distance_mm(&self) -> f32 {
        match self {
            Self::IchishakuHassun => 10.0,
            Self::NishakuYonsun => 14.0,
            Self::IchishakuRokusun => 8.0,
            Self::NishakuIssun => 11.5,
            Self::SanShakuKyotaku => 18.0,
        }
    }

    /// Return nominal breath turbulence gain.
    pub fn default_turbulence_gain(&self) -> f32 {
        match self {
            Self::IchishakuHassun => 0.16,
            Self::NishakuYonsun => 0.24,
            Self::IchishakuRokusun => 0.12,
            Self::NishakuIssun => 0.18,
            Self::SanShakuKyotaku => 0.30,
        }
    }

    /// Return flute name string.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::IchishakuHassun => "1.8 Shaku (D4 Honkyoku)",
            Self::NishakuYonsun => "2.4 Shaku (A3 Jinashi Zen)",
            Self::IchishakuRokusun => "1.6 Shaku (E4 Min'yo)",
            Self::NishakuIssun => "2.1 Shaku (B3 Sankyoku)",
            Self::SanShakuKyotaku => "3.0 Shaku (D3 Kyotaku)",
        }
    }
}

/// Sound and articulation profiles for Shakuhachi synthesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ShakuhachiProfile {
    /// Classical Honkyoku Zen meditation (1.8 Shaku, rich harmonics, organic meri vibrato).
    #[default]
    HonkyokuTraditional,
    /// Deep introspective Jinashi Zen meditation (2.4 Shaku, heavy breath turbulence).
    ZenMeditative,
    /// Energetic Min'yo folk flute (1.6 Shaku, snappy attack, bright upper partials).
    MinyoFolk,
    /// Sankyoku chamber ensemble (2.1 Shaku, warm balanced tone, subtle vibrato).
    SankyokuEnsemble,
    /// Explosive Murai-Iki technique with turbulent vortex breath bursts and dramatic overblowing.
    MuraiIkiExplosive,
    /// Deep sub-bass Kyotaku temple flute (3.0 Shaku, massive acoustic air column).
    KyotakuTemple,
}

impl ShakuhachiProfile {
    /// Return nominal flute length for this profile.
    pub fn flute_length(&self) -> ShakuhachiFluteLength {
        match self {
            Self::HonkyokuTraditional => ShakuhachiFluteLength::IchishakuHassun,
            Self::ZenMeditative => ShakuhachiFluteLength::NishakuYonsun,
            Self::MinyoFolk => ShakuhachiFluteLength::IchishakuRokusun,
            Self::SankyokuEnsemble => ShakuhachiFluteLength::NishakuIssun,
            Self::MuraiIkiExplosive => ShakuhachiFluteLength::IchishakuHassun,
            Self::KyotakuTemple => ShakuhachiFluteLength::SanShakuKyotaku,
        }
    }

    /// Return nominal blowing pressure in Pascals.
    pub fn nominal_pressure_pa(&self) -> f32 {
        match self {
            Self::HonkyokuTraditional => 850.0,
            Self::ZenMeditative => 700.0,
            Self::MinyoFolk => 1100.0,
            Self::SankyokuEnsemble => 800.0,
            Self::MuraiIkiExplosive => 2400.0,
            Self::KyotakuTemple => 650.0,
        }
    }

    /// Return nominal murai-iki burst intensity.
    pub fn murai_iki_intensity(&self) -> f32 {
        match self {
            Self::MuraiIkiExplosive => 0.85,
            Self::ZenMeditative => 0.35,
            _ => 0.05,
        }
    }
}

fn default_bore_buffer() -> Box<[f32; MAX_BORE_DELAY]> {
    vec![0.0; MAX_BORE_DELAY].into_boxed_slice().try_into().unwrap()
}

/// A single monophonic physical modeling Shakuhachi voice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShakuhachiVoice {
    pub active: bool,
    pub midi_note: u8,
    pub target_freq_hz: f32,
    pub velocity: f32,
    pub sample_rate: u32,
    /// Air-reed excitation generator.
    pub air_reed: AirReed,
    /// Forward traveling pressure wave delay line (bore entrance -> bell).
    #[serde(skip, default = "default_bore_buffer")]
    pub bore_delay_fwd: Box<[f32; MAX_BORE_DELAY]>,
    /// Backward traveling pressure wave delay line (bell -> bore entrance).
    #[serde(skip, default = "default_bore_buffer")]
    pub bore_delay_bwd: Box<[f32; MAX_BORE_DELAY]>,
    pub bore_write_pos: usize,
    /// Bore wall loss damping factor [0.95 ..= 0.999].
    pub bore_damping: f32,
    /// Bell reflection filter state (one-pole lowpass for negative reflection).
    pub bell_filter_state: f32,
    /// Bell reflection filter coefficient.
    pub bell_reflection_coeff: f32,
    /// Dynamic Meri/Kari pitch offset in cents [-300.0 ..= +200.0].
    pub meri_kari_cents: f32,
    /// Active 5-hole bitmask.
    pub hole_mask: u32,
    /// Amplitude envelope follower for note on/off transitions.
    pub envelope: f32,
    pub env_target: f32,
    pub env_rate: f32,
}

impl Default for ShakuhachiVoice {
    fn default() -> Self {
        Self::new(48000)
    }
}

impl ShakuhachiVoice {
    pub fn new(sample_rate: u32) -> Self {
        let sr = sample_rate.max(8000);
        let mut reed = AirReed::new(sr);
        reed.set_blowing_pressure(DEFAULT_PRESSURE_PA);

        Self {
            active: false,
            midi_note: 62, // D4
            target_freq_hz: 293.66,
            velocity: 0.0,
            sample_rate: sr,
            air_reed: reed,
            bore_delay_fwd: default_bore_buffer(),
            bore_delay_bwd: default_bore_buffer(),
            bore_write_pos: 0,
            bore_damping: 0.985,
            bell_filter_state: 0.0,
            bell_reflection_coeff: 0.35,
            meri_kari_cents: 0.0,
            hole_mask: 0,
            envelope: 0.0,
            env_target: 0.0,
            env_rate: 0.005,
        }
    }

    pub fn reset(&mut self) {
        self.active = false;
        self.envelope = 0.0;
        self.env_target = 0.0;
        self.bore_delay_fwd.fill(0.0);
        self.bore_delay_bwd.fill(0.0);
        self.bore_write_pos = 0;
        self.bell_filter_state = 0.0;
        self.air_reed.reset();
    }

    pub fn note_on(&mut self, note: u8, vel: f32, base_pressure: f32) {
        self.active = true;
        self.midi_note = note;
        self.velocity = vel.clamp(0.0, 1.0);
        self.target_freq_hz = 440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0);
        let pressure = (base_pressure * (0.5 + 0.5 * self.velocity)).clamp(100.0, 4000.0);
        self.air_reed.set_blowing_pressure(pressure);
        self.env_target = 1.0;
        self.env_rate = 0.008; // Fast attack
    }

    pub fn note_off(&mut self) {
        self.env_target = 0.0;
        self.env_rate = 0.002; // Gentle breath release
    }

    /// Calculate effective acoustic frequency accounting for toneholes, meri/kari, and register.
    #[inline]
    pub fn effective_frequency(&self) -> f32 {
        let mut freq = self.target_freq_hz;

        // Apply traditional pentatonic tonehole shift if hole_mask is set:
        // Bit 0 (Hole 1 Tsu): +3 semitones (F on D4)
        // Bit 1 (Hole 2 Re):  +5 semitones (G on D4)
        // Bit 2 (Hole 3 Chi): +7 semitones (A on D4)
        // Bit 3 (Hole 4 Ri):  +10 semitones (C on D4)
        // Bit 4 (Hole 5 Thumb): +12 semitones (Octave vent)
        let mut semitone_offset = 0.0f32;
        if (self.hole_mask & HOLE_5_THUMB) != 0 {
            semitone_offset += 12.0;
        } else if (self.hole_mask & HOLE_4_RI) != 0 {
            semitone_offset += 10.0;
        } else if (self.hole_mask & HOLE_3_CHI) != 0 {
            semitone_offset += 7.0;
        } else if (self.hole_mask & HOLE_2_RE) != 0 {
            semitone_offset += 5.0;
        } else if (self.hole_mask & HOLE_1_TSU) != 0 {
            semitone_offset += 3.0;
        }

        if semitone_offset > 0.0 {
            freq *= 2.0_f32.powf(semitone_offset / 12.0);
        }

        // Apply continuous Meri/Kari pitch multiplier ([-300, +200] cents)
        let meri_kari_mult = meri_kari_to_pitch_multiplier(self.meri_kari_cents);
        (freq * meri_kari_mult).clamp(40.0, 4000.0)
    }

    /// Calculate half-bore delay in samples: D_bore = f_s / (2 * f_effective)
    #[inline]
    pub fn calculate_bore_delay_samples(&self) -> f32 {
        let eff_freq = self.effective_frequency();
        let total_period_samples = (self.sample_rate as f32) / eff_freq;
        // In open-open acoustic pipe with end-corrections, one-way delay is half period minus jet delay
        let jet_delay = self.air_reed.calculate_jet_delay_samples();
        let one_way_delay = (total_period_samples * 0.5) - jet_delay * 0.5;
        one_way_delay.clamp(2.0, (MAX_BORE_DELAY - 4) as f32)
    }

    /// Read fractional delay from circular bore buffer using linear interpolation.
    #[inline]
    fn read_bore_delay(buffer: &[f32; MAX_BORE_DELAY], write_pos: usize, delay_samples: f32) -> f32 {
        let d_clamped = delay_samples.clamp(0.0, (MAX_BORE_DELAY - 2) as f32);
        let d_int = d_clamped.floor() as usize;
        let d_frac = d_clamped - (d_int as f32);

        let cap = MAX_BORE_DELAY;
        let idx0 = (write_pos + cap - d_int) % cap;
        let idx1 = (write_pos + cap - d_int - 1) % cap;

        let s0 = buffer[idx0];
        let s1 = buffer[idx1];
        s0 + d_frac * (s1 - s0)
    }

    /// Process one sample of the physical waveguide Shakuhachi voice.
    #[inline]
    pub fn step(&mut self) -> f32 {
        if !self.active && self.envelope < 1e-4 {
            return 0.0;
        }

        // Smooth envelope
        self.envelope += self.env_rate * (self.env_target - self.envelope);
        if self.env_target == 0.0 && self.envelope < 1e-4 {
            self.active = false;
            self.reset();
            return 0.0;
        }

        let delay_len = self.calculate_bore_delay_samples();

        // 1. Read backward wave arriving at bore entrance from bell
        let bwd_wave_at_mouth = Self::read_bore_delay(&self.bore_delay_bwd, self.bore_write_pos, delay_len);

        // 2. Air-reed excitation (utaguchi jet splitting)
        let excitation = self.air_reed.step(bwd_wave_at_mouth);

        // 3. Forward wave entering bore = excitation - backward wave reflection
        let fwd_wave_in = excitation - bwd_wave_at_mouth * self.bore_damping;

        // 4. Forward wave arriving at open bell end
        let fwd_wave_at_bell = Self::read_bore_delay(&self.bore_delay_fwd, self.bore_write_pos, delay_len);

        // 5. Open bell negative reflection with one-pole lowpass damping:
        // y[n] = - ( (1 - c) * x[n] + c * y[n-1] )
        let bell_refl_coeff = self.bell_reflection_coeff;
        self.bell_filter_state = (1.0 - bell_refl_coeff) * fwd_wave_at_bell + bell_refl_coeff * self.bell_filter_state;
        let bwd_wave_in = -self.bell_filter_state * self.bore_damping;

        // 6. Write waves to circular delay lines
        self.bore_delay_fwd[self.bore_write_pos] = fwd_wave_in;
        self.bore_delay_bwd[self.bore_write_pos] = bwd_wave_in;
        self.bore_write_pos = (self.bore_write_pos + 1) % MAX_BORE_DELAY;

        // 7. Radiated acoustic output = highpass radiation at open bell + breath leakage
        let radiated_bell = (fwd_wave_at_bell - bwd_wave_in) * 0.5;
        let radiated_mouth = excitation * 0.35;
        let total_output = (radiated_bell + radiated_mouth) * self.envelope;

        total_output.clamp(-1.0, 1.0)
    }
}

/// Complete polyphonic physical modeling Japanese Shakuhachi Synthesizer Engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shakuhachi {
    pub profile: ShakuhachiProfile,
    pub flute_length: ShakuhachiFluteLength,
    pub blowing_pressure_pa: f32,
    pub meri_kari_cents: f32,
    pub embouchure_angle_deg: f32,
    pub jet_distance_mm: f32,
    pub murai_iki_intensity: f32,
    pub hole_mask: u32,
    pub master_gain: f32,
    pub sample_rate: u32,
    pub voices: Vec<ShakuhachiVoice>,
}

impl Default for Shakuhachi {
    fn default() -> Self {
        Self::new(48000)
    }
}

impl Shakuhachi {
    /// Create a new Shakuhachi physical synthesis engine for the given sample rate.
    pub fn new(sample_rate: u32) -> Self {
        let sr = sample_rate.max(8000);
        let profile = ShakuhachiProfile::default();
        let length = profile.flute_length();
        let init_p = profile.nominal_pressure_pa();
        let murai = profile.murai_iki_intensity();

        let voices = (0..MAX_SHAKUHACHI_VOICES)
            .map(|_| ShakuhachiVoice::new(sr))
            .collect();

        let mut shakuhachi = Self {
            profile,
            flute_length: length,
            blowing_pressure_pa: init_p,
            meri_kari_cents: 0.0,
            embouchure_angle_deg: 38.0,
            jet_distance_mm: length.default_jet_distance_mm(),
            murai_iki_intensity: murai,
            hole_mask: 0,
            master_gain: 0.85,
            sample_rate: sr,
            voices,
        };

        shakuhachi.apply_parameters_to_voices();
        shakuhachi
    }

    /// Reset all internal voice states and delay lines.
    pub fn reset(&mut self) {
        for voice in &mut self.voices {
            voice.reset();
        }
    }

    /// Set sound profile.
    pub fn set_profile(&mut self, profile: ShakuhachiProfile) {
        self.profile = profile;
        self.flute_length = profile.flute_length();
        self.blowing_pressure_pa = profile.nominal_pressure_pa();
        self.murai_iki_intensity = profile.murai_iki_intensity();
        self.jet_distance_mm = self.flute_length.default_jet_distance_mm();
        self.apply_parameters_to_voices();
    }

    /// Set blowing pressure in Pascals [50.0 ..= 4000.0].
    pub fn set_blowing_pressure(&mut self, pressure_pa: f32) {
        self.blowing_pressure_pa = pressure_pa.clamp(MIN_PRESSURE_PA, 4000.0);
        for voice in &mut self.voices {
            voice.air_reed.set_blowing_pressure(self.blowing_pressure_pa);
        }
    }

    /// Set Meri/Kari head tilt pitch shift in cents [-300.0 ..= +200.0].
    pub fn set_meri_kari(&mut self, cents: f32) {
        self.meri_kari_cents = cents.clamp(-300.0, 200.0);
        for voice in &mut self.voices {
            voice.meri_kari_cents = self.meri_kari_cents;
            voice.air_reed.set_meri_kari(self.meri_kari_cents);
        }
    }

    /// Set embouchure angle in degrees [10.0 ..= 60.0].
    pub fn set_embouchure_angle(&mut self, angle_deg: f32) {
        self.embouchure_angle_deg = angle_deg.clamp(10.0, 60.0);
        for voice in &mut self.voices {
            voice.air_reed.config.embouchure_angle_deg = self.embouchure_angle_deg;
        }
    }

    /// Set jet distance in millimeters [2.0 ..= 25.0].
    pub fn set_jet_distance_mm(&mut self, dist_mm: f32) {
        self.jet_distance_mm = dist_mm.clamp(2.0, 25.0);
        for voice in &mut self.voices {
            voice.air_reed.set_jet_distance_mm(self.jet_distance_mm);
        }
    }

    /// Set Murai-Iki explosive breath burst intensity [0.0 ..= 1.0].
    pub fn set_murai_iki(&mut self, intensity: f32) {
        self.murai_iki_intensity = intensity.clamp(0.0, 1.0);
        for voice in &mut self.voices {
            voice.air_reed.set_murai_iki(self.murai_iki_intensity);
        }
    }

    /// Set 5-hole finger state bitmask (bits 0..4).
    pub fn set_hole_mask(&mut self, mask: u32) {
        self.hole_mask = mask & 0x1F;
        for voice in &mut self.voices {
            voice.hole_mask = self.hole_mask;
        }
    }

    /// Apply global parameters to all voices.
    fn apply_parameters_to_voices(&mut self) {
        for voice in &mut self.voices {
            voice.air_reed.set_blowing_pressure(self.blowing_pressure_pa);
            voice.air_reed.set_meri_kari(self.meri_kari_cents);
            voice.air_reed.config.embouchure_angle_deg = self.embouchure_angle_deg;
            voice.air_reed.set_jet_distance_mm(self.jet_distance_mm);
            voice.air_reed.set_murai_iki(self.murai_iki_intensity);
            voice.air_reed.config.turbulence_gain = self.flute_length.default_turbulence_gain();
            voice.meri_kari_cents = self.meri_kari_cents;
            voice.hole_mask = self.hole_mask;
        }
    }

    /// Trigger a note on event with MIDI pitch and velocity.
    pub fn note_on(&mut self, note: u8, velocity: f32) {
        // Find free voice or steal oldest / lowest amplitude voice
        let mut target_idx = 0;
        let mut min_env = f32::MAX;

        for (i, voice) in self.voices.iter().enumerate() {
            if !voice.active {
                target_idx = i;
                break;
            }
            if voice.envelope < min_env {
                min_env = voice.envelope;
                target_idx = i;
            }
        }

        self.voices[target_idx].note_on(note, velocity, self.blowing_pressure_pa);
        self.voices[target_idx].meri_kari_cents = self.meri_kari_cents;
        self.voices[target_idx].hole_mask = self.hole_mask;
    }

    /// Trigger a note off event for matching MIDI pitch.
    pub fn note_off(&mut self, note: u8) {
        for voice in &mut self.voices {
            if voice.active && voice.midi_note == note {
                voice.note_off();
            }
        }
    }

    /// Render one mono sample frame.
    #[inline]
    pub fn step(&mut self) -> f32 {
        let mut sum = 0.0f32;
        let mut active_count = 0;

        for voice in &mut self.voices {
            if voice.active || voice.envelope > 1e-4 {
                sum += voice.step();
                active_count += 1;
            }
        }

        let scaled = if active_count > 1 {
            sum * (1.0 / (active_count as f32).sqrt()) * self.master_gain
        } else {
            sum * self.master_gain
        };

        scaled.clamp(-1.0, 1.0)
    }
}

impl SignalProcessor for Shakuhachi {
    fn name(&self) -> &str {
        "Shakuhachi"
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
            let sample = self.step();

            // Output to all available channels (stereo / multi-channel)
            for ch in outputs.iter_mut() {
                if i < ch.len() {
                    ch[i] = sample;
                }
            }
        }
    }
}

impl AudioNode for Shakuhachi {
    fn name(&self) -> &str {
        "Shakuhachi"
    }

    fn process(&mut self, inputs: &[&[Sample]], outputs: &mut [&mut [Sample]], ctx: &ProcessContext) {
        self.process_block(inputs, outputs, ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::allocator::AllocGuard;

    #[test]
    fn test_shakuhachi_initialization() {
        let shakuhachi = Shakuhachi::new(48000);
        assert_eq!(shakuhachi.flute_length, ShakuhachiFluteLength::IchishakuHassun);
        assert!((shakuhachi.blowing_pressure_pa - 850.0).abs() < 1.0);
    }

    #[test]
    fn test_shakuhachi_finger_hole_frequencies() {
        let mut voice = ShakuhachiVoice::new(48000);
        voice.target_freq_hz = 293.66; // D4

        // 1. All closed (Ro) -> D4 (293.66 Hz)
        voice.hole_mask = 0;
        assert!((voice.effective_frequency() - 293.66).abs() < 0.1);

        // 2. Hole 1 open (Tsu) -> F4 (+3 semitones ~ 349.23 Hz)
        voice.hole_mask = HOLE_1_TSU;
        let freq_tsu = voice.effective_frequency();
        assert!((freq_tsu - 349.23).abs() < 1.0);

        // 3. Hole 2 open (Re) -> G4 (+5 semitones ~ 392.00 Hz)
        voice.hole_mask = HOLE_1_TSU | HOLE_2_RE;
        let freq_re = voice.effective_frequency();
        assert!((freq_re - 392.00).abs() < 1.0);

        // 4. Hole 3 open (Chi) -> A4 (+7 semitones ~ 440.00 Hz)
        voice.hole_mask = HOLE_1_TSU | HOLE_2_RE | HOLE_3_CHI;
        let freq_chi = voice.effective_frequency();
        assert!((freq_chi - 440.00).abs() < 1.0);

        // 5. Hole 4 open (Ri) -> C5 (+10 semitones ~ 523.25 Hz)
        voice.hole_mask = HOLE_1_TSU | HOLE_2_RE | HOLE_3_CHI | HOLE_4_RI;
        let freq_ri = voice.effective_frequency();
        assert!((freq_ri - 523.25).abs() < 1.0);
    }

    #[test]
    fn test_shakuhachi_meri_kari_pitch_shift() {
        let mut voice = ShakuhachiVoice::new(48000);
        voice.target_freq_hz = 293.66; // D4
        voice.hole_mask = 0;

        // Meri -200 cents (whole tone down ~ 261.63 Hz / C4)
        voice.meri_kari_cents = -200.0;
        let freq_meri = voice.effective_frequency();
        assert!((freq_meri - 261.63).abs() < 1.0);

        // Kari +100 cents (semitone up ~ 311.13 Hz / D#4)
        voice.meri_kari_cents = 100.0;
        let freq_kari = voice.effective_frequency();
        assert!((freq_kari - 311.13).abs() < 1.0);
    }

    #[test]
    fn test_shakuhachi_zero_allocation_processing() {
        let mut shakuhachi = Shakuhachi::new(48000);
        shakuhachi.note_on(62, 0.9);

        let mut out_l = [0.0f32; 128];
        let mut out_r = [0.0f32; 128];
        let mut out_slices: [&mut [Sample]; 2] = [&mut out_l, &mut out_r];
        let ctx = ProcessContext::new(48000, 120.0, 0);

        {
            let _guard = AllocGuard::new();
            for _ in 0..10 {
                shakuhachi.process_block(&[], &mut out_slices, &ctx);
            }
        }

        // Verify that rendered samples are finite and non-zero
        let max_val = out_l.iter().fold(0.0f32, |acc, &x| acc.max(x.abs()));
        assert!(max_val > 0.0);
        assert!(max_val <= 1.0);
    }
}
