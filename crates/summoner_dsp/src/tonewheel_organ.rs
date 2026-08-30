// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Electromechanical Tonewheel Organ Generator (Milestone 21).
//!
//! Provides a physical modeling 91-tonewheel synchronous electromechanical generator
//! with harmonic gear ratios, variable magnetic induction crosstalk, 9-drawbar additive mixing
//! bus (16', 5-1/3', 8', 4', 2-2/3', 2', 1-3/5', 1-1/3', 1'), non-legato 2nd/3rd harmonic percussion
//! circuit, key-click electrical contact bounce transient modeling, and delay scanner vibrato/chorus.
//!
//! Enforces zero heap allocations in real-time audio render loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};
use crate::traits::SignalProcessor;

fn default_active_keys() -> [bool; 128] {
    [false; 128]
}

fn default_key_floats() -> [f32; 128] {
    [0.0; 128]
}

fn default_tonewheels() -> [f32; NUM_TONEWHEELS] {
    [0.0; NUM_TONEWHEELS]
}

fn default_scanner_buffer() -> [f32; 2048] {
    [0.0; 2048]
}

/// Total number of synchronous electromagnetic tonewheels in a vintage console generator.
pub const NUM_TONEWHEELS: usize = 91;

/// Number of harmonic drawbars in the standard mixing register.
pub const NUM_DRAWBARS: usize = 9;

/// Standard drawbar harmonic intervals in semitones relative to fundamental note.
/// [16', 5-1/3', 8', 4', 2-2/3', 2', 1-3/5', 1-1/3', 1']
pub const DRAWBAR_SEMITONE_OFFSETS: [i32; NUM_DRAWBARS] = [
    -12, // 16' (Sub-octave)
    7,   // 5-1/3' (Sub-5th / Quint)
    0,   // 8' (Unison / Fundamental)
    12,  // 4' (8ve)
    19,  // 2-2/3' (12th / Nazard)
    24,  // 2' (15th / Blockflöte)
    28,  // 1-3/5' (17th / Tierce)
    31,  // 1-1/3' (19th / Larigot)
    36,  // 1' (22nd / Sifflöte)
];

/// Harmonic percussion mode selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PercussionHarmonic {
    /// 2nd harmonic (4' pitch).
    #[default]
    SecondHarmonic,
    /// 3rd harmonic (2-2/3' pitch).
    ThirdHarmonic,
}

/// Scanner vibrato / chorus mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum VibratoChorusMode {
    #[default]
    Off,
    V1, // Light Vibrato
    C1, // Light Chorus
    V2, // Standard Vibrato
    C2, // Standard Chorus
    V3, // Deep Vibrato
    C3, // Deep Chorus
}

/// Preset tonewheel organ sound profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TonewheelOrganProfile {
    /// Classic Gospel & Soul drawbar registration: 88 8000 008 with 3rd harmonic fast percussion.
    #[default]
    ClassicGospel,
    /// Warm Jazz Trio "Jimmy Smith" registration: 88 8000 000 with 3rd soft fast percussion.
    JazzTrio888,
    /// Full Rock & Blues screaming registration: 88 8888 888 with C3 chorus and drive.
    FullRockPower,
    /// Mellow Theater / Flute drawbar setting: 80 0008 000 with light V1 vibrato.
    MellowTheater,
    /// Funky Clav/Organ percussive setup: 80 8000 008 with 2nd fast percussion.
    FunkyGroove,
    /// Modern Ambient Swell registration: 80 0008 888 with deep C3 chorus.
    AmbientSwell,
}

impl TonewheelOrganProfile {
    /// Returns default 9-drawbar levels in range $[0.0 ..= 8.0]$.
    pub fn default_drawbars(&self) -> [f32; NUM_DRAWBARS] {
        match self {
            Self::ClassicGospel => [8.0, 8.0, 8.0, 0.0, 0.0, 0.0, 0.0, 0.0, 8.0],
            Self::JazzTrio888 => [8.0, 8.0, 8.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            Self::FullRockPower => [8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0],
            Self::MellowTheater => [8.0, 0.0, 0.0, 0.0, 0.0, 8.0, 0.0, 0.0, 0.0],
            Self::FunkyGroove => [8.0, 0.0, 8.0, 0.0, 0.0, 0.0, 0.0, 0.0, 8.0],
            Self::AmbientSwell => [8.0, 0.0, 0.0, 0.0, 0.0, 8.0, 8.0, 8.0, 8.0],
        }
    }

    /// Returns default percussion settings (enabled, harmonic, fast, soft).
    pub fn default_percussion(&self) -> (bool, PercussionHarmonic, bool, bool) {
        match self {
            Self::ClassicGospel => (true, PercussionHarmonic::ThirdHarmonic, true, false),
            Self::JazzTrio888 => (true, PercussionHarmonic::ThirdHarmonic, true, true),
            Self::FullRockPower => (false, PercussionHarmonic::SecondHarmonic, false, false),
            Self::MellowTheater => (false, PercussionHarmonic::SecondHarmonic, false, false),
            Self::FunkyGroove => (true, PercussionHarmonic::SecondHarmonic, true, false),
            Self::AmbientSwell => (false, PercussionHarmonic::ThirdHarmonic, false, true),
        }
    }

    /// Returns default vibrato/chorus mode.
    pub fn default_vibrato(&self) -> VibratoChorusMode {
        match self {
            Self::ClassicGospel => VibratoChorusMode::C3,
            Self::JazzTrio888 => VibratoChorusMode::C3,
            Self::FullRockPower => VibratoChorusMode::C3,
            Self::MellowTheater => VibratoChorusMode::V1,
            Self::FunkyGroove => VibratoChorusMode::Off,
            Self::AmbientSwell => VibratoChorusMode::C3,
        }
    }

    /// Returns human-readable name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::ClassicGospel => "Classic Gospel Drawbars (88 8000 008)",
            Self::JazzTrio888 => "Jazz Trio 888 (88 8000 000)",
            Self::FullRockPower => "Full Rock Power Organ (88 8888 888)",
            Self::MellowTheater => "Mellow Theater Flutes (80 0008 000)",
            Self::FunkyGroove => "Funky Percussive Groove (80 8000 008)",
            Self::AmbientSwell => "Ambient Swell Chorus (80 0008 888)",
        }
    }
}

/// Physical modeling tonewheel electromagnetic generator node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TonewheelOrgan {
    pub sample_rate: u32,
    pub profile: TonewheelOrganProfile,
    /// 9 Drawbar levels in range $[0.0 ..= 8.0]$.
    pub drawbars: [f32; NUM_DRAWBARS],
    /// Key click contact bounce transient amplitude $[0.0 ..= 1.0]$.
    pub key_click_amount: f32,
    /// Magnetic induction crosstalk leakage ratio $[0.0 ..= 1.0]$.
    pub crosstalk_amount: f32,
    /// Harmonic percussion enabled.
    pub percussion_enabled: bool,
    /// Percussion harmonic (2nd or 3rd).
    pub percussion_harmonic: PercussionHarmonic,
    /// Percussion fast decay (~0.2s) vs slow decay (~1.0s).
    pub percussion_fast: bool,
    /// Percussion soft level (reduces volume, preserves normal organ gain).
    pub percussion_soft: bool,
    /// Scanner vibrato/chorus mode.
    pub vibrato_mode: VibratoChorusMode,
    /// Master organ gain.
    pub master_gain: f32,
    // Active note state (61-key manual: MIDI notes 36..=96)
    #[serde(skip, default = "default_active_keys")]
    active_keys: [bool; 128],
    #[serde(skip, default = "default_key_floats")]
    key_velocities: [f32; 128],
    #[serde(skip, default = "default_key_floats")]
    key_click_timers: [f32; 128],
    // 91 Tonewheel phase accumulators and base frequencies
    #[serde(skip, default = "default_tonewheels")]
    tonewheel_phases: [f32; NUM_TONEWHEELS],
    #[serde(skip, default = "default_tonewheels")]
    tonewheel_freqs: [f32; NUM_TONEWHEELS],
    // Harmonic percussion envelope state
    #[serde(skip)]
    percussion_envelope: f32,
    #[serde(skip)]
    percussion_gate: bool,
    #[serde(skip)]
    percussion_phase: f32,
    // Scanner delay buffer for vibrato/chorus (~10ms buffer)
    #[serde(skip, default = "default_scanner_buffer")]
    scanner_buffer: [f32; 2048],
    #[serde(skip)]
    scanner_write_pos: usize,
    #[serde(skip)]
    scanner_lfo_phase: f32,
    // DC block filter states
    #[serde(skip)]
    dc_x1: f32,
    #[serde(skip)]
    dc_y1: f32,
}

impl TonewheelOrgan {
    /// Creates a new physical modeling tonewheel organ synthesizer.
    pub fn new(sample_rate: u32) -> Self {
        let sr = sample_rate.max(8000);
        let prof = TonewheelOrganProfile::ClassicGospel;

        // Compute 91 Tonewheel frequencies corresponding to standard Hammond generator
        // Tonewheel #1 is C1 (MIDI 24, ~32.703 Hz) up to Tonewheel #91 (F#8, MIDI 114, ~5919.9 Hz)
        let mut tonewheel_freqs = [0.0f32; NUM_TONEWHEELS];
        for (i, freq) in tonewheel_freqs.iter_mut().enumerate() {
            let midi_note = 24.0 + i as f32;
            let f = 440.0 * 2.0_f32.powf((midi_note - 69.0) / 12.0);
            *freq = f;
        }

        let (perc_on, perc_harm, perc_fast, perc_soft) = prof.default_percussion();

        let mut organ = Self {
            sample_rate: sr,
            profile: prof,
            drawbars: prof.default_drawbars(),
            key_click_amount: 0.35,
            crosstalk_amount: 0.08,
            percussion_enabled: perc_on,
            percussion_harmonic: perc_harm,
            percussion_fast: perc_fast,
            percussion_soft: perc_soft,
            vibrato_mode: prof.default_vibrato(),
            master_gain: 0.85,
            active_keys: [false; 128],
            key_velocities: [0.0; 128],
            key_click_timers: [0.0; 128],
            tonewheel_phases: [0.0; NUM_TONEWHEELS],
            tonewheel_freqs,
            percussion_envelope: 0.0,
            percussion_gate: false,
            percussion_phase: 0.0,
            scanner_buffer: [0.0; 2048],
            scanner_write_pos: 0,
            scanner_lfo_phase: 0.0,
            dc_x1: 0.0,
            dc_y1: 0.0,
        };

        // Initialize pseudo-random initial phases across wheels to prevent phase alignment spikes
        for i in 0..NUM_TONEWHEELS {
            organ.tonewheel_phases[i] = ((i * 7919) % 1000) as f32 / 1000.0 * TAU;
        }

        organ
    }

    /// Sets the active sound profile and loads nominal drawbar & percussion settings.
    pub fn set_profile(&mut self, profile: TonewheelOrganProfile) {
        self.profile = profile;
        self.drawbars = profile.default_drawbars();
        let (p_on, p_harm, p_fast, p_soft) = profile.default_percussion();
        self.percussion_enabled = p_on;
        self.percussion_harmonic = p_harm;
        self.percussion_fast = p_fast;
        self.percussion_soft = p_soft;
        self.vibrato_mode = profile.default_vibrato();
    }

    /// Sets a specific drawbar level $[0.0 ..= 8.0]$.
    pub fn set_drawbar(&mut self, index: usize, level: f32) {
        if index < NUM_DRAWBARS {
            self.drawbars[index] = level.clamp(0.0, 8.0);
        }
    }

    /// Triggers a MIDI Note-On event.
    pub fn note_on(&mut self, midi_note: u8, velocity: f32) {
        let note = midi_note as usize;
        if note < 128 {
            let was_any_active = self.active_keys.iter().any(|&k| k);
            self.active_keys[note] = true;
            self.key_velocities[note] = velocity.clamp(0.01, 1.0);
            self.key_click_timers[note] = 0.0035; // 3.5ms key click contact bounce

            // Non-legato percussion trigger: fires if all notes were previously released
            if self.percussion_enabled && !was_any_active {
                self.percussion_envelope = 1.0;
                self.percussion_gate = true;
            }
        }
    }

    /// Triggers a MIDI Note-Off event.
    pub fn note_off(&mut self, midi_note: u8) {
        let note = midi_note as usize;
        if note < 128 {
            self.active_keys[note] = false;
            self.key_click_timers[note] = 0.0020; // Release contact bounce click

            if !self.active_keys.iter().any(|&k| k) {
                self.percussion_gate = false;
            }
        }
    }

    /// Releases all active notes (All-Notes-Off).
    pub fn panic(&mut self) {
        self.active_keys.fill(false);
        self.key_velocities.fill(0.0);
        self.key_click_timers.fill(0.0);
        self.percussion_envelope = 0.0;
        self.percussion_gate = false;
    }

    /// Calculates tonewheel wheel index for a given MIDI note and harmonic offset.
    #[inline(always)]
    fn note_to_tonewheel(midi_note: i32, semitone_offset: i32) -> Option<usize> {
        let target_note = midi_note + semitone_offset;
        let wheel_idx = target_note - 24; // Tonewheel #1 = MIDI 24 (C1)
        if (0..NUM_TONEWHEELS as i32).contains(&wheel_idx) {
            Some(wheel_idx as usize)
        } else {
            // Foldback: In vintage organs, notes below #1 or above #91 fold back to valid octaves
            if wheel_idx < 0 {
                let folded = wheel_idx + 12;
                if (0..NUM_TONEWHEELS as i32).contains(&folded) {
                    Some(folded as usize)
                } else {
                    None
                }
            } else {
                let folded = wheel_idx - 12;
                if (0..NUM_TONEWHEELS as i32).contains(&folded) {
                    Some(folded as usize)
                } else {
                    None
                }
            }
        }
    }

    /// Processes one sample through the electromechanical generator and scanner vibrato.
    pub fn process_sample(&mut self) -> f32 {
        let dt = 1.0 / (self.sample_rate as f32);

        // 1. Advance all 91 synchronous tonewheel phase accumulators
        for i in 0..NUM_TONEWHEELS {
            let inc = TAU * self.tonewheel_freqs[i] * dt;
            self.tonewheel_phases[i] = (self.tonewheel_phases[i] + inc) % TAU;
        }

        // 2. Accumulate additive tone from 9 drawbars across all active keys
        let mut raw_organ_signal = 0.0f32;
        let mut key_click_signal = 0.0f32;

        // Convert drawbar 0..8 values to linear gains
        let mut drawbar_gains = [0.0f32; NUM_DRAWBARS];
        for (d, gain) in drawbar_gains.iter_mut().enumerate() {
            let lvl = self.drawbars[d];
            if lvl > 0.0 {
                // Approximate 3dB/step scale: lvl=8 is 1.0, lvl=1 is ~0.08
                *gain = (lvl / 8.0).powf(1.4);
            }
        }

        let mut active_count = 0;
        for note in 0..128 {
            if self.active_keys[note] {
                active_count += 1;
                let vel = self.key_velocities[note];

                for d in 0..NUM_DRAWBARS {
                    let gain = drawbar_gains[d];
                    if gain > 0.0 {
                        if let Some(wheel) = Self::note_to_tonewheel(note as i32, DRAWBAR_SEMITONE_OFFSETS[d]) {
                            let ph = self.tonewheel_phases[wheel];
                            // Pure fundamental sine plus subtle magnetic tooth saturation
                            let sine = ph.sin();
                            let tooth_harm = 0.03 * (2.0 * ph).sin() + 0.015 * (3.0 * ph).sin();
                            raw_organ_signal += (sine + tooth_harm) * gain * vel;
                        }
                    }
                }
            }

            // Key click transient spark generator
            if self.key_click_timers[note] > 0.0 {
                self.key_click_timers[note] -= dt;
                // High-frequency pseudo-random contact noise burst
                let hash = ((note as u32).wrapping_mul(2654435761) ^ (self.scanner_write_pos as u32).wrapping_mul(805306457)) as f32;
                let noise = ((hash % 1000.0) / 500.0) - 1.0;
                key_click_signal += noise * self.key_click_amount * 0.15;
            }
        }

        // 3. Magnetic induction crosstalk leakage from adjacent tonewheels
        if self.crosstalk_amount > 0.0 && active_count > 0 {
            let mut leak_sum = 0.0f32;
            for i in (0..NUM_TONEWHEELS).step_by(6) {
                leak_sum += self.tonewheel_phases[i].sin();
            }
            raw_organ_signal += leak_sum * (self.crosstalk_amount * 0.015);
        }

        // 4. Harmonic Percussion synthesis
        let mut percussion_signal = 0.0f32;
        if self.percussion_enabled && self.percussion_envelope > 0.001 {
            let decay_rate = if self.percussion_fast { 5.0 } else { 1.0 }; // ~0.2s or ~1.0s decay
            self.percussion_envelope -= decay_rate * dt * self.percussion_envelope;

            // Find highest active note for percussion pitch
            let mut highest_note = None;
            for note in (0..128).rev() {
                if self.active_keys[note] {
                    highest_note = Some(note);
                    break;
                }
            }

            if let Some(top_note) = highest_note {
                let perc_semitone = match self.percussion_harmonic {
                    PercussionHarmonic::SecondHarmonic => 12, // 4'
                    PercussionHarmonic::ThirdHarmonic => 19,  // 2-2/3'
                };
                if let Some(wheel) = Self::note_to_tonewheel(top_note as i32, perc_semitone) {
                    let f = self.tonewheel_freqs[wheel];
                    self.percussion_phase = (self.percussion_phase + TAU * f * dt) % TAU;
                    let perc_gain = if self.percussion_soft { 0.45 } else { 0.90 };
                    percussion_signal = self.percussion_phase.sin() * self.percussion_envelope * perc_gain;
                }
            }
        }

        // Combine organ drawbars, percussion, and key click
        let organ_mix_gain = if self.percussion_enabled && !self.percussion_soft && self.percussion_envelope > 0.01 {
            0.70 // Vintage circuit attenuates drawbars when normal percussion is active
        } else {
            1.0
        };

        let combined = (raw_organ_signal * organ_mix_gain + percussion_signal + key_click_signal) * 0.15;

        // 5. Scanner Vibrato / Chorus delay-line modulation
        let modulated = if self.vibrato_mode != VibratoChorusMode::Off {
            let buf_len = self.scanner_buffer.len();
            self.scanner_buffer[self.scanner_write_pos] = combined;

            // LFO rate ~6.85 Hz standard Hammond scanner speed
            self.scanner_lfo_phase = (self.scanner_lfo_phase + TAU * 6.85 * dt) % TAU;
            let lfo = self.scanner_lfo_phase.sin();

            let depth_ms = match self.vibrato_mode {
                VibratoChorusMode::V1 | VibratoChorusMode::C1 => 0.75,
                VibratoChorusMode::V2 | VibratoChorusMode::C2 => 1.50,
                VibratoChorusMode::V3 | VibratoChorusMode::C3 => 2.50,
                VibratoChorusMode::Off => 0.0,
            };

            let delay_samples = (3.0 + lfo * depth_ms) * (self.sample_rate as f32 / 1000.0);
            let read_pos = (self.scanner_write_pos as f32 + buf_len as f32 - delay_samples) % (buf_len as f32);

            let idx0 = read_pos.floor() as usize % buf_len;
            let idx1 = (idx0 + 1) % buf_len;
            let frac = read_pos.fract();
            let delayed_sample = self.scanner_buffer[idx0] * (1.0 - frac) + self.scanner_buffer[idx1] * frac;

            self.scanner_write_pos = (self.scanner_write_pos + 1) % buf_len;

            match self.vibrato_mode {
                VibratoChorusMode::V1 | VibratoChorusMode::V2 | VibratoChorusMode::V3 => delayed_sample,
                VibratoChorusMode::C1 | VibratoChorusMode::C2 | VibratoChorusMode::C3 => {
                    0.55 * combined + 0.45 * delayed_sample // Chorus mix (dry + wet)
                }
                VibratoChorusMode::Off => combined,
            }
        } else {
            combined
        };

        // 6. DC Block filter
        let out_unfiltered = modulated * self.master_gain;
        let dc_out = out_unfiltered - self.dc_x1 + 0.995 * self.dc_y1;
        self.dc_x1 = out_unfiltered;
        self.dc_y1 = if dc_out.is_finite() { dc_out } else { 0.0 };

        self.dc_y1.clamp(-1.0, 1.0)
    }
}

impl SignalProcessor for TonewheelOrgan {
    fn name(&self) -> &str {
        "TonewheelOrgan"
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
            let sample = self.process_sample();
            for out_ch in outputs.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = sample;
                }
            }
        }
    }
}

/// AudioNode wrapper for Tonewheel Organ generator.
#[derive(Debug)]
pub struct TonewheelOrganNode {
    pub organ: TonewheelOrgan,
}

impl TonewheelOrganNode {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            organ: TonewheelOrgan::new(sample_rate),
        }
    }
}

impl AudioNode for TonewheelOrganNode {
    fn name(&self) -> &str {
        "TonewheelOrganNode"
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
    fn test_tonewheel_organ_frequencies_and_profiles() {
        let mut organ = TonewheelOrgan::new(48000);
        assert_eq!(organ.tonewheel_freqs.len(), 91);
        // Tonewheel #1 (~32.7 Hz), Wheel #46 (440 Hz)
        assert!((organ.tonewheel_freqs[0] - 32.7).abs() < 1.0);
        assert!((organ.tonewheel_freqs[45] - 440.0).abs() < 0.1);

        organ.set_profile(TonewheelOrganProfile::ClassicGospel);
        assert_eq!(organ.drawbars[0], 8.0);
        assert_eq!(organ.drawbars[8], 8.0);
    }

    #[test]
    fn test_tonewheel_organ_note_on_sound_and_percussion() {
        let mut organ = TonewheelOrgan::new(48000);
        organ.note_on(60, 0.9); // Middle C (C4)

        let mut peak = 0.0f32;
        for _ in 0..2400 {
            let s = organ.process_sample();
            assert!(s.is_finite());
            assert!((-1.0..=1.0).contains(&s));
            peak = peak.max(s.abs());
        }
        assert!(peak > 0.01, "Organ must produce sound on note_on");

        organ.note_off(60);
        for _ in 0..10000 {
            let s = organ.process_sample();
            assert!(s.is_finite());
        }
    }

    #[test]
    fn test_tonewheel_organ_zero_allocation_in_loop() {
        let mut organ = TonewheelOrgan::new(48000);
        organ.note_on(60, 0.8);
        organ.note_on(64, 0.8);
        organ.note_on(67, 0.8);

        let mut out_l = [0.0f32; 256];
        let mut out_r = [0.0f32; 256];
        let ctx = ProcessContext::new(48000, 120.0, 0);

        let _guard = AllocGuard::new();
        for _ in 0..10 {
            organ.process_block(&[], &mut [&mut out_l[..], &mut out_r[..]], &ctx);
        }
    }
}
