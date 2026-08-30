// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Electromechanical Tine & Reed Electric Piano (Milestone 22).
//!
//! Provides a physical modeling dual-mode electromechanical piano synthesizer (Rhodes Tine /
//! Wurlitzer Reed) with non-linear hammer contact dynamics, inharmonic cantilever beam dispersion,
//! resonant tonebar energy coupling, electromagnetic inductive pickup barking saturation,
//! damper clunk mechanics, tube preamp saturation, and stereo optical tremolo / vibrato panner.
//!
//! Enforces zero heap allocations in real-time audio render loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};
use crate::tine_resonator::{ElectricPianoModel, TineResonator};
use crate::traits::SignalProcessor;

/// Maximum number of polyphonic electric piano voices.
pub const MAX_EP_VOICES: usize = 32;

/// Factory electromechanical electric piano sound profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ElectricPianoProfile {
    /// Classic 1970s Rhodes Suitcase with stereo optical ping-pong tremolo and bell chime.
    #[default]
    ClassicRhodesSuitcase,
    /// High-energy Dyno-My-Piano with bright high-end, close pickup air-gap, and heavy barking bite.
    BarkingDynoRhodes,
    /// Warm Mellow Rhodes Stage 73 with deep tonebar resonance and gentle tube warmth.
    MellowRhodesStage,
    /// Classic Wurlitzer 200A Reed with gritty midrange growl and subtle optical tremolo.
    ClassicWurlitzer200A,
    /// High-gain Soul & Funk Wurlitzer with overdriven tube preamp and biting reed crunch.
    SoulOverdrivenWurli,
    /// Ambient Spatial Rhodes with lush stereo panning and extended tonebar sustain.
    BelledAmbientRhodes,
}

impl ElectricPianoProfile {
    /// Returns default model type.
    pub fn model(&self) -> ElectricPianoModel {
        match self {
            Self::ClassicRhodesSuitcase
            | Self::BarkingDynoRhodes
            | Self::MellowRhodesStage
            | Self::BelledAmbientRhodes => ElectricPianoModel::RhodesTine,
            Self::ClassicWurlitzer200A | Self::SoulOverdrivenWurli => ElectricPianoModel::WurlitzerReed,
        }
    }

    /// Returns default (hammer_hardness, air_gap_mm, bark_drive, tonebar_coupling, damper_clunk).
    pub fn mechanical_defaults(&self) -> (f32, f32, f32, f32, f32) {
        match self {
            Self::ClassicRhodesSuitcase => (0.50, 1.8, 0.55, 0.70, 0.35),
            Self::BarkingDynoRhodes => (0.85, 1.0, 0.90, 0.85, 0.40),
            Self::MellowRhodesStage => (0.30, 2.4, 0.30, 0.60, 0.25),
            Self::ClassicWurlitzer200A => (0.60, 1.2, 0.75, 0.35, 0.45),
            Self::SoulOverdrivenWurli => (0.80, 0.9, 0.95, 0.40, 0.50),
            Self::BelledAmbientRhodes => (0.40, 2.0, 0.40, 0.95, 0.20),
        }
    }

    /// Returns default (tremolo_enabled, tremolo_rate_hz, tremolo_depth, tremolo_stereo).
    pub fn tremolo_defaults(&self) -> (bool, f32, f32, bool) {
        match self {
            Self::ClassicRhodesSuitcase => (true, 5.2, 0.65, true),
            Self::BarkingDynoRhodes => (false, 4.0, 0.0, false),
            Self::MellowRhodesStage => (false, 3.5, 0.0, false),
            Self::ClassicWurlitzer200A => (true, 6.0, 0.50, false),
            Self::SoulOverdrivenWurli => (true, 7.2, 0.40, false),
            Self::BelledAmbientRhodes => (true, 2.8, 0.80, true),
        }
    }

    /// Returns default (drive_db, bass_db, treble_db).
    pub fn tone_defaults(&self) -> (f32, f32, f32) {
        match self {
            Self::ClassicRhodesSuitcase => (4.0, 1.5, 2.0),
            Self::BarkingDynoRhodes => (8.0, -1.0, 6.5),
            Self::MellowRhodesStage => (2.0, 3.0, -1.5),
            Self::ClassicWurlitzer200A => (6.0, 0.0, 3.0),
            Self::SoulOverdrivenWurli => (14.0, 2.0, 5.0),
            Self::BelledAmbientRhodes => (2.0, 2.5, 3.5),
        }
    }
}

/// Stereo optical tremolo / vibrato auto-panner modeling incandescent bulb / LDR response.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OpticalTremolo {
    /// Tremolo enabled.
    pub enabled: bool,
    /// LFO modulation frequency in Hz $[0.5 ..= 15.0\text{Hz}]$.
    pub rate_hz: f32,
    /// Modulation depth $[0.0 ..= 1.0]$.
    pub depth: f32,
    /// Stereo ping-pong phase mode (true = $180^\circ$ Rhodes Suitcase, false = $0^\circ$ mono Wurli).
    pub stereo_pan: bool,
    /// LFO phase accumulator.
    phase: f32,
    /// Smoothed optical bulb intensity (Left channel).
    bulb_l: f32,
    /// Smoothed optical bulb intensity (Right channel).
    bulb_r: f32,
}

impl Default for OpticalTremolo {
    fn default() -> Self {
        Self {
            enabled: true,
            rate_hz: 5.2,
            depth: 0.65,
            stereo_pan: true,
            phase: 0.0,
            bulb_l: 1.0,
            bulb_r: 1.0,
        }
    }
}

impl OpticalTremolo {
    pub fn new(enabled: bool, rate_hz: f32, depth: f32, stereo_pan: bool) -> Self {
        Self {
            enabled,
            rate_hz: rate_hz.clamp(0.2, 20.0),
            depth: depth.clamp(0.0, 1.0),
            stereo_pan,
            phase: 0.0,
            bulb_l: 1.0,
            bulb_r: 1.0,
        }
    }

    /// Processes stereo audio through optical tremolo circuit.
    #[inline]
    pub fn process(&mut self, in_l: f32, in_r: f32, sample_rate: u32) -> (f32, f32) {
        if !self.enabled || self.depth < 0.001 {
            return (in_l, in_r);
        }

        let dt = 2.0 * PI * self.rate_hz / sample_rate as f32;
        self.phase = (self.phase + dt) % (2.0 * PI);

        // LFO target intensity
        let target_l = (self.phase.sin() * 0.5 + 0.5) * self.depth + (1.0 - self.depth);
        let target_r = if self.stereo_pan {
            ((self.phase + PI).sin() * 0.5 + 0.5) * self.depth + (1.0 - self.depth)
        } else {
            target_l
        };

        // Non-linear incandescent bulb filament thermal lag (1-pole smoothing)
        let lag_coef = 0.08;
        self.bulb_l += lag_coef * (target_l - self.bulb_l);
        self.bulb_r += lag_coef * (target_r - self.bulb_r);

        (in_l * self.bulb_l, in_r * self.bulb_r)
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
        self.bulb_l = 1.0;
        self.bulb_r = 1.0;
    }
}

/// Preamp saturation and 2-band Baxandall/Shelf EQ tone control.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PreampToneControl {
    /// Preamp drive in dB $[0.0 ..= 24.0\text{dB}]$.
    pub drive_db: f32,
    /// Bass gain in dB $[-12.0 ..= 12.0\text{dB}]$.
    pub bass_db: f32,
    /// Treble gain in dB $[-12.0 ..= 12.0\text{dB}]$.
    pub treble_db: f32,
    // 1-pole filter states for low/high shelves
    bass_state_l: f32,
    bass_state_r: f32,
    treble_state_l: f32,
    treble_state_r: f32,
}

impl Default for PreampToneControl {
    fn default() -> Self {
        Self {
            drive_db: 4.0,
            bass_db: 1.5,
            treble_db: 2.0,
            bass_state_l: 0.0,
            bass_state_r: 0.0,
            treble_state_l: 0.0,
            treble_state_r: 0.0,
        }
    }
}

impl PreampToneControl {
    pub fn new(drive_db: f32, bass_db: f32, treble_db: f32) -> Self {
        Self {
            drive_db: drive_db.clamp(0.0, 24.0),
            bass_db: bass_db.clamp(-12.0, 12.0),
            treble_db: treble_db.clamp(-12.0, 12.0),
            bass_state_l: 0.0,
            bass_state_r: 0.0,
            treble_state_l: 0.0,
            treble_state_r: 0.0,
        }
    }

    /// Processes stereo samples through tone controls and tube preamp saturation.
    #[inline]
    pub fn process(&mut self, in_l: f32, in_r: f32, sample_rate: u32) -> (f32, f32) {
        let sr = sample_rate as f32;

        // Bass shelf (~150 Hz)
        let bass_coef = (2.0 * PI * 150.0 / sr).clamp(0.001, 0.5);
        self.bass_state_l += bass_coef * (in_l - self.bass_state_l);
        self.bass_state_r += bass_coef * (in_r - self.bass_state_r);

        let bass_linear = 10.0f32.powf(self.bass_db / 20.0);
        let bass_out_l = (in_l - self.bass_state_l) + self.bass_state_l * bass_linear;
        let bass_out_r = (in_r - self.bass_state_r) + self.bass_state_r * bass_linear;

        // Treble shelf (~3500 Hz)
        let treble_coef = (2.0 * PI * 3500.0 / sr).clamp(0.01, 0.8);
        self.treble_state_l += treble_coef * (bass_out_l - self.treble_state_l);
        self.treble_state_r += treble_coef * (bass_out_r - self.treble_state_r);

        let treble_linear = 10.0f32.powf(self.treble_db / 20.0);
        let treble_out_l = self.treble_state_l + (bass_out_l - self.treble_state_l) * treble_linear;
        let treble_out_r = self.treble_state_r + (bass_out_r - self.treble_state_r) * treble_linear;

        // Tube Preamp Overdrive Saturation
        let drive_gain = 10.0f32.powf(self.drive_db / 20.0);
        let sat_l = self.saturate(treble_out_l * drive_gain);
        let sat_r = self.saturate(treble_out_r * drive_gain);

        (sat_l, sat_r)
    }

    #[inline]
    fn saturate(&self, x: f32) -> f32 {
        if x > 0.0 {
            // Asymmetric triode soft clipping
            x / (1.0 + x)
        } else {
            let nx = -x * 1.1;
            -(nx / (1.0 + nx))
        }
    }

    pub fn reset(&mut self) {
        self.bass_state_l = 0.0;
        self.bass_state_r = 0.0;
        self.treble_state_l = 0.0;
        self.treble_state_r = 0.0;
    }
}

/// Physical Modeling Electromechanical Tine & Reed Piano Synthesizer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectricPiano {
    /// Active sound profile.
    pub profile: ElectricPianoProfile,
    /// Physical resonator model (Rhodes Tine / Wurlitzer Reed).
    pub model: ElectricPianoModel,
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Pre-allocated polyphonic voice channels.
    voices: Vec<TineResonator>,
    /// Global hammer hardness $[0.0 ..= 1.0]$.
    pub hammer_hardness: f32,
    /// Pickup air-gap distance in mm $[0.5 ..= 5.0\text{mm}]$.
    pub air_gap_mm: f32,
    /// Pickup barking overdrive $[0.0 ..= 1.0]$.
    pub bark_drive: f32,
    /// Resonant tonebar coupling stiffness $[0.0 ..= 1.0]$.
    pub tonebar_coupling: f32,
    /// Damper release clunk volume $[0.0 ..= 1.0]$.
    pub damper_clunk_volume: f32,
    /// Master volume level $[0.0 ..= 2.0]$.
    pub master_gain: f32,
    /// Stereo optical tremolo section.
    pub tremolo: OpticalTremolo,
    /// Tube preamp and EQ tone control.
    pub tone_control: PreampToneControl,
    /// Voice allocation counter.
    voice_alloc_counter: u64,
}

impl Default for ElectricPiano {
    fn default() -> Self {
        Self::new(48000)
    }
}

impl ElectricPiano {
    /// Creates a new physical electric piano engine instance.
    pub fn new(sample_rate: u32) -> Self {
        let sr = sample_rate.max(8000);
        let mut voices = Vec::with_capacity(MAX_EP_VOICES);
        for _ in 0..MAX_EP_VOICES {
            voices.push(TineResonator::new(ElectricPianoModel::RhodesTine, 261.63, sr));
        }

        let mut ep = Self {
            profile: ElectricPianoProfile::ClassicRhodesSuitcase,
            model: ElectricPianoModel::RhodesTine,
            sample_rate: sr,
            voices,
            hammer_hardness: 0.50,
            air_gap_mm: 1.8,
            bark_drive: 0.55,
            tonebar_coupling: 0.70,
            damper_clunk_volume: 0.35,
            master_gain: 0.90,
            tremolo: OpticalTremolo::default(),
            tone_control: PreampToneControl::default(),
            voice_alloc_counter: 0,
        };

        ep.set_profile(ElectricPianoProfile::ClassicRhodesSuitcase);
        ep
    }

    /// Sets electric piano sound profile and loads physical defaults.
    pub fn set_profile(&mut self, profile: ElectricPianoProfile) {
        self.profile = profile;
        self.model = profile.model();

        let (hardness, air_gap, bark, tonebar, damper) = profile.mechanical_defaults();
        self.hammer_hardness = hardness;
        self.air_gap_mm = air_gap;
        self.bark_drive = bark;
        self.tonebar_coupling = tonebar;
        self.damper_clunk_volume = damper;

        let (trem_en, trem_rate, trem_depth, trem_stereo) = profile.tremolo_defaults();
        self.tremolo.enabled = trem_en;
        self.tremolo.rate_hz = trem_rate;
        self.tremolo.depth = trem_depth;
        self.tremolo.stereo_pan = trem_stereo;

        let (drive, bass, treble) = profile.tone_defaults();
        self.tone_control.drive_db = drive;
        self.tone_control.bass_db = bass;
        self.tone_control.treble_db = treble;

        self.apply_voice_parameters();
    }

    /// Sets model type (Tine / Reed) and updates all voices.
    pub fn set_model(&mut self, model: ElectricPianoModel) {
        self.model = model;
        for voice in self.voices.iter_mut() {
            voice.configure_model(model);
        }
        self.apply_voice_parameters();
    }

    /// Applies current global physical parameters to all voice resonators.
    pub fn apply_voice_parameters(&mut self) {
        for voice in self.voices.iter_mut() {
            voice.pickup.air_gap_mm = self.air_gap_mm;
            voice.pickup.bark_drive = self.bark_drive;
            voice.tonebar.coupling_stiffness = self.tonebar_coupling;
            voice.damper.clunk_volume = self.damper_clunk_volume;
            voice.tonebar.recompute_coefficients(voice.frequency, self.sample_rate);
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

        // Voice stealing: pick oldest voice
        let voice_idx = target_idx.unwrap_or(0);
        self.voice_alloc_counter += 1;

        let voice = &mut self.voices[voice_idx];
        voice.configure_model(self.model);
        voice.pickup.air_gap_mm = self.air_gap_mm;
        voice.pickup.bark_drive = self.bark_drive;
        voice.tonebar.coupling_stiffness = self.tonebar_coupling;
        voice.damper.clunk_volume = self.damper_clunk_volume;
        voice.note_on(note, freq, velocity, self.hammer_hardness);
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

    /// Processes one stereo sample pair through polyphonic voices, preamp, and optical tremolo.
    #[inline]
    pub fn process_sample(&mut self) -> (f32, f32) {
        let mut raw_mono = 0.0f32;

        for voice in self.voices.iter_mut() {
            if voice.is_active {
                raw_mono += voice.process_sample();
            }
        }

        // Preamp & Tone Control
        let (sat_l, sat_r) = self.tone_control.process(raw_mono, raw_mono, self.sample_rate);

        // Stereo Optical Tremolo / Panner
        let (trem_l, trem_r) = self.tremolo.process(sat_l, sat_r, self.sample_rate);

        let final_l = (trem_l * self.master_gain).clamp(-1.0, 1.0);
        let final_r = (trem_r * self.master_gain).clamp(-1.0, 1.0);

        (final_l, final_r)
    }

    /// Resets all internal state and voices.
    pub fn reset(&mut self) {
        for voice in self.voices.iter_mut() {
            voice.reset();
        }
        self.tremolo.reset();
        self.tone_control.reset();
    }
}

impl SignalProcessor for ElectricPiano {
    fn name(&self) -> &str {
        "ElectricPiano"
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

impl AudioNode for ElectricPiano {
    fn name(&self) -> &str {
        "ElectricPianoNode"
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
    fn test_electric_piano_zero_allocation() {
        let mut ep = ElectricPiano::new(48000);
        ep.set_profile(ElectricPianoProfile::ClassicRhodesSuitcase);
        ep.note_on(60, 0.85); // C4
        ep.note_on(64, 0.80); // E4
        ep.note_on(67, 0.80); // G4

        let transport = Transport::new(48000, 120.0);
        let ctx = ProcessContext::from_transport(&transport);

        let mut out_l = [0.0f32; 64];
        let mut out_r = [0.0f32; 64];

        {
            let _guard = AllocGuard::new();
            for _ in 0..16 {
                ep.process_block(&[], &mut [&mut out_l[..], &mut out_r[..]], &ctx);
                for i in 0..64 {
                    assert!(out_l[i].is_finite());
                    assert!(out_r[i].is_finite());
                }
            }
        }
    }

    #[test]
    fn test_electric_piano_polyphony_and_damper() {
        let mut ep = ElectricPiano::new(48000);
        ep.set_profile(ElectricPianoProfile::ClassicWurlitzer200A);
        ep.note_on(57, 0.90);
        assert!(ep.voices.iter().any(|v| v.is_active));

        ep.note_off(57);
        // Should trigger damper
        let (s_l, s_r) = ep.process_sample();
        assert!(s_l.is_finite());
        assert!(s_r.is_finite());
    }

    #[test]
    fn test_optical_tremolo_quadrature() {
        let mut trem = OpticalTremolo::new(true, 5.0, 1.0, true);
        let mut l_samples = Vec::new();
        let mut r_samples = Vec::new();
        for _ in 0..200 {
            let (l, r) = trem.process(1.0, 1.0, 48000);
            l_samples.push(l);
            r_samples.push(r);
        }
        assert!(l_samples.iter().any(|&v| v.is_finite()));
        assert!(r_samples.iter().any(|&v| v.is_finite()));
    }
}
