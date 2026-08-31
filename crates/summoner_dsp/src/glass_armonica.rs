// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Franklin Glass Armonica Synthesis Engine (Milestone 31).
//!
//! Provides a high-precision physical acoustic synthesis engine for Benjamin Franklin's
//! Glass Armonica (37 concentric nested quartz glass bowls mounted on a rotating horizontal iron spindle)
//! featuring:
//! - Continuous horizontal spindle rotation ($\omega \in [0.0 ..= 6\pi]\text{ rad/s}$)
//! - 37 tuned quartz glass bowls spanning 3 chromatic octaves (C4 to C7, MIDI 60 to 96)
//! - Nested bowl radius scaling ($R_k \in [0.04, 0.18]\text{ m}$) determining linear rubbing surface speeds
//! - Wet-finger stick-slip friction excitation with non-linear Stribeck velocity curves
//! - 2D circular thin-shell modal resonance with $(2,0)$ hoop doublet splitting for acoustic beating shimmer
//! - Continuous hydro-acoustic water trough / bowl water filling mass loading pitch lowering
//! - Resonant mahogany chassis and soundbox cavity acoustic coupling
//! - Polyphonic multi-touch note activation and velocity-sensitive finger pressure
//!
//! Enforces zero heap allocations in audio render loops under `AllocGuard`.

use crate::crystal_resonator::{CrystalMaterialProfile, CrystalResonator};
use crate::traits::SignalProcessor;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};

fn default_armonica_bowls() -> [GlassArmonicaBowl; NUM_ARMONICA_BOWLS] {
    let mut bowls_vec = Vec::with_capacity(NUM_ARMONICA_BOWLS);
    for i in 0..NUM_ARMONICA_BOWLS {
        bowls_vec.push(GlassArmonicaBowl::new(i, 48000));
    }
    match bowls_vec.try_into() {
        Ok(arr) => arr,
        Err(_) => unreachable!(),
    }
}

/// Number of chromatic glass bowls in the Franklin Armonica (3 full octaves C4 to C7).
pub const NUM_ARMONICA_BOWLS: usize = 37;

/// Lowest MIDI note for the Armonica (C4 = MIDI 60, 261.63 Hz).
pub const ARMONICA_BASE_MIDI: u8 = 60;

/// Base fundamental pitch for Bowl 0 (C4 = 261.626 Hz).
pub const ARMONICA_BASE_FREQ_HZ: f32 = 261.62557;

/// Historical and material preset profiles for the Glass Armonica.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ArmonicaProfile {
    /// Authentic 1761 Franklin Quartz Glass (Pure crystalline tone, subtle doublet shimmer).
    #[default]
    FranklinAuthentic1761,
    /// Mesmer Magnetic Healing Chalice (Slow spindle speed, heavy water loading, deep beating).
    MesmerHealingChalice,
    /// Mozart Adagio & Rondo in C Minor (Virtuoso concert tuning, crisp attack, fast response).
    MozartConcertArmonica,
    /// Ethereal Modern Borosilicate (High-frequency overtone ring, ultra-long sustain).
    EtherealBorosilicate,
    /// Water-Tuned Crystal Chime (Dynamic water-glissando pitch drops, shimmering chorus).
    WaterTunedCrystalChime,
}

impl ArmonicaProfile {
    pub fn name(&self) -> &'static str {
        match self {
            Self::FranklinAuthentic1761 => "1761 Franklin Authentic Quartz Armonica",
            Self::MesmerHealingChalice => "Mesmer Magnetic Healing Chalice",
            Self::MozartConcertArmonica => "Mozart Concert Virtuoso Armonica",
            Self::EtherealBorosilicate => "Ethereal Modern Borosilicate Glass",
            Self::WaterTunedCrystalChime => "Water-Tuned Crystal Chime",
        }
    }

    pub fn material_profile(&self) -> CrystalMaterialProfile {
        match self {
            Self::FranklinAuthentic1761 => CrystalMaterialProfile::FranklinQuartzGlass,
            Self::MesmerHealingChalice => CrystalMaterialProfile::PureQuartzCrystal,
            Self::MozartConcertArmonica => CrystalMaterialProfile::FranklinQuartzGlass,
            Self::EtherealBorosilicate => CrystalMaterialProfile::BorosilicateBell,
            Self::WaterTunedCrystalChime => CrystalMaterialProfile::WetCrystalGoblet,
        }
    }

    /// Returns nominal $(\omega_{\text{spindle}}, F_N, \text{water\_fill}, \text{chassis\_res}, \text{master\_gain})$.
    pub fn nominal_physics(&self) -> (f32, f32, f32, f32, f32) {
        match self {
            Self::FranklinAuthentic1761 => (2.5 * PI, 0.45, 0.05, 0.75, 0.90),
            Self::MesmerHealingChalice => (1.5 * PI, 0.65, 0.35, 0.85, 0.95),
            Self::MozartConcertArmonica => (3.0 * PI, 0.50, 0.02, 0.70, 0.92),
            Self::EtherealBorosilicate => (2.2 * PI, 0.40, 0.10, 0.80, 0.88),
            Self::WaterTunedCrystalChime => (2.0 * PI, 0.55, 0.45, 0.65, 0.90),
        }
    }
}

/// A single rotating quartz glass bowl on the common spindle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlassArmonicaBowl {
    pub bowl_index: usize,
    pub midi_note: u8,
    pub nominal_freq_hz: f32,
    pub bowl_radius_m: f32,
    pub is_touched: bool,
    pub touch_force_n: f32,
    pub pan_position: f32, // Stereo panning along horizontal spindle [-1.0 = bass left, +1.0 = treble right]
    pub resonator: CrystalResonator,
}

impl GlassArmonicaBowl {
    pub fn new(bowl_index: usize, sample_rate: u32) -> Self {
        let midi_note = ARMONICA_BASE_MIDI + (bowl_index as u8).min(NUM_ARMONICA_BOWLS as u8 - 1);
        let semitones = bowl_index as f32;
        let freq_hz = ARMONICA_BASE_FREQ_HZ * 2.0f32.powf(semitones / 12.0);

        // Nested radius: lowest bowl (C4) ~0.16m down to highest bowl (C7) ~0.04m
        let radius_m = 0.16 - 0.12 * (bowl_index as f32 / (NUM_ARMONICA_BOWLS - 1) as f32);

        // Panning along spindle length
        let pan = -0.80 + 1.60 * (bowl_index as f32 / (NUM_ARMONICA_BOWLS - 1) as f32);

        let mut resonator = CrystalResonator::new(freq_hz, sample_rate);
        resonator.set_profile(CrystalMaterialProfile::FranklinQuartzGlass);
        resonator.exciter.rim_radius_m = radius_m;

        Self {
            bowl_index,
            midi_note,
            nominal_freq_hz: freq_hz,
            bowl_radius_m: radius_m,
            is_touched: false,
            touch_force_n: 0.0,
            pan_position: pan.clamp(-1.0, 1.0),
            resonator,
        }
    }

    /// Touch bowl with wet finger applying specified normal force in Newtons ($0.05 ..= 2.0$).
    pub fn touch(&mut self, normal_force_n: f32) {
        self.is_touched = true;
        self.touch_force_n = normal_force_n.clamp(0.02, 2.5);
        self.resonator.exciter.normal_force_n = self.touch_force_n;
    }

    /// Lift finger off the bowl.
    pub fn release(&mut self) {
        self.is_touched = false;
        self.touch_force_n = 0.0;
        self.resonator.exciter.normal_force_n = 0.0;
    }

    /// Strike bowl with a soft mallet or tap.
    pub fn strike(&mut self, velocity: f32) {
        self.resonator.strike_mallet(velocity);
    }

    /// Update spindle angular velocity for this bowl.
    pub fn set_spindle_speed(&mut self, speed_rad_s: f32) {
        self.resonator.exciter.wand_speed_rad_s = speed_rad_s;
    }

    /// Update water fill level for this bowl.
    pub fn set_water_fill(&mut self, fill: f32) {
        self.resonator.set_water_fill_level(fill);
    }

    /// Process single audio sample step for this bowl.
    #[inline]
    pub fn process_sample(&mut self) -> f32 {
        let (out, _, _) = self.resonator.process_sample();
        out
    }

    pub fn reset(&mut self) {
        self.is_touched = false;
        self.touch_force_n = 0.0;
        self.resonator.reset();
    }
}

/// A 2nd-order resonant filter for the mahogany armonica case / acoustic trough resonator.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ArmonicaChassisMode {
    pub freq_hz: f32,
    pub q: f32,
    pub gain: f32,
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl ArmonicaChassisMode {
    pub fn new(freq_hz: f32, q: f32, gain: f32, sample_rate: u32) -> Self {
        let mut mode = Self {
            freq_hz,
            q: q.max(0.5),
            gain,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        };
        mode.recalculate(sample_rate);
        mode
    }

    pub fn recalculate(&mut self, sample_rate: u32) {
        let sr = sample_rate as f32;
        let omega = 2.0 * PI * (self.freq_hz / sr).clamp(0.001, 0.49);
        let sin_w = omega.sin();
        let cos_w = omega.cos();
        let alpha = sin_w / (2.0 * self.q);

        let b0 = alpha * self.gain;
        let b1 = 0.0;
        let b2 = -alpha * self.gain;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_w;
        let a2 = 1.0 - alpha;

        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }

    #[inline]
    pub fn step(&mut self, input: f32) -> f32 {
        let y = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2 - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }

    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
}

/// Mahogany case soundbox & water trough acoustic cavity resonator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArmonicaChassisResonator {
    pub modes: [ArmonicaChassisMode; 4],
    pub master_chassis_gain: f32,
}

impl ArmonicaChassisResonator {
    pub fn new(sample_rate: u32) -> Self {
        let modes = [
            ArmonicaChassisMode::new(135.0, 8.0, 1.2, sample_rate), // Main wooden casing Helmholtz mode
            ArmonicaChassisMode::new(280.0, 12.0, 0.95, sample_rate),// Soundboard table flexure
            ArmonicaChassisMode::new(510.0, 15.0, 0.70, sample_rate),// Water trough fluid-structure mode
            ArmonicaChassisMode::new(920.0, 20.0, 0.50, sample_rate),// Spindle iron axle bearing resonance
        ];
        Self {
            modes,
            master_chassis_gain: 0.75,
        }
    }

    #[inline]
    pub fn process(&mut self, input: f32) -> f32 {
        let mut sum = 0.0;
        for mode in self.modes.iter_mut() {
            sum += mode.step(input);
        }
        sum * self.master_chassis_gain
    }

    pub fn reset(&mut self) {
        for mode in self.modes.iter_mut() {
            mode.reset();
        }
    }
}

/// Franklin Glass Armonica Physical Modeling Synthesizer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlassArmonica {
    pub profile: ArmonicaProfile,
    pub sample_rate: u32,
    pub spindle_speed_rad_s: f32,
    pub global_water_fill: f32,
    pub master_gain: f32,
    pub chassis_coupling_gain: f32,

    /// 37 Tuned concentric quartz glass bowls.
    #[serde(skip, default = "default_armonica_bowls")]
    pub bowls: [GlassArmonicaBowl; NUM_ARMONICA_BOWLS],

    /// Wooden case & trough acoustic cavity resonator.
    pub chassis: ArmonicaChassisResonator,
}

impl Default for GlassArmonica {
    fn default() -> Self {
        Self::new(48000)
    }
}

impl GlassArmonica {
    pub fn new(sample_rate: u32) -> Self {
        let profile = ArmonicaProfile::FranklinAuthentic1761;
        let (speed, _, water, chassis_gain, mgain) = profile.nominal_physics();

        let mut bowls_vec = Vec::with_capacity(NUM_ARMONICA_BOWLS);
        for i in 0..NUM_ARMONICA_BOWLS {
            let mut b = GlassArmonicaBowl::new(i, sample_rate);
            b.set_spindle_speed(speed);
            b.set_water_fill(water);
            b.resonator.set_profile(profile.material_profile());
            bowls_vec.push(b);
        }

        let bowls: [GlassArmonicaBowl; NUM_ARMONICA_BOWLS] = match bowls_vec.try_into() {
            Ok(arr) => arr,
            Err(_) => unreachable!(),
        };

        Self {
            profile,
            sample_rate,
            spindle_speed_rad_s: speed,
            global_water_fill: water,
            master_gain: mgain,
            chassis_coupling_gain: chassis_gain,
            bowls,
            chassis: ArmonicaChassisResonator::new(sample_rate),
        }
    }

    /// Set instrument and sound preset profile.
    pub fn set_profile(&mut self, profile: ArmonicaProfile) {
        self.profile = profile;
        let (speed, _, water, chassis_gain, mgain) = profile.nominal_physics();
        self.spindle_speed_rad_s = speed;
        self.global_water_fill = water;
        self.chassis_coupling_gain = chassis_gain;
        self.master_gain = mgain;

        for bowl in self.bowls.iter_mut() {
            bowl.resonator.set_profile(profile.material_profile());
            bowl.set_spindle_speed(speed);
            bowl.set_water_fill(water);
        }
    }

    /// Set continuous horizontal spindle rotation speed in rad/s ($0.0 ..= 6\pi$).
    pub fn set_spindle_speed(&mut self, speed_rad_s: f32) {
        self.spindle_speed_rad_s = speed_rad_s.clamp(0.0, 6.0 * PI);
        for bowl in self.bowls.iter_mut() {
            bowl.set_spindle_speed(self.spindle_speed_rad_s);
        }
    }

    /// Set global water trough / bowl water fill fraction $h \in [0.0 ..= 1.0]$.
    pub fn set_water_fill(&mut self, level: f32) {
        self.global_water_fill = level.clamp(0.0, 1.0);
        for bowl in self.bowls.iter_mut() {
            bowl.set_water_fill(self.global_water_fill);
        }
    }

    /// Touch bowl by MIDI note number with specified finger pressure $[0.05 ..= 2.0]\text{ N}$.
    pub fn touch_note(&mut self, midi_note: u8, normal_force_n: f32) {
        if (ARMONICA_BASE_MIDI..ARMONICA_BASE_MIDI + NUM_ARMONICA_BOWLS as u8).contains(&midi_note) {
            let idx = (midi_note - ARMONICA_BASE_MIDI) as usize;
            self.bowls[idx].touch(normal_force_n);
        }
    }

    /// Lift finger off bowl by MIDI note number.
    pub fn release_note(&mut self, midi_note: u8) {
        if (ARMONICA_BASE_MIDI..ARMONICA_BASE_MIDI + NUM_ARMONICA_BOWLS as u8).contains(&midi_note) {
            let idx = (midi_note - ARMONICA_BASE_MIDI) as usize;
            self.bowls[idx].release();
        }
    }

    /// Strike bowl by MIDI note number with a soft mallet.
    pub fn strike_note(&mut self, midi_note: u8, velocity: f32) {
        if (ARMONICA_BASE_MIDI..ARMONICA_BASE_MIDI + NUM_ARMONICA_BOWLS as u8).contains(&midi_note) {
            let idx = (midi_note - ARMONICA_BASE_MIDI) as usize;
            self.bowls[idx].strike(velocity);
        }
    }

    /// MIDI note on handler.
    pub fn note_on(&mut self, midi_note: u8, velocity: f32) {
        let force = 0.20 + velocity.clamp(0.0, 1.0) * 0.80;
        self.touch_note(midi_note, force);
    }

    /// MIDI note off handler.
    pub fn note_off(&mut self, midi_note: u8) {
        self.release_note(midi_note);
    }

    /// Process a single stereo sample frame. Returns `(left, right)` samples clamped to $[-1.0, 1.0]$.
    #[inline]
    pub fn process_stereo_frame(&mut self) -> (f32, f32) {
        let mut total_left = 0.0f32;
        let mut total_right = 0.0f32;
        let mut chassis_drive = 0.0f32;

        for bowl in self.bowls.iter_mut() {
            let sample = bowl.process_sample();
            if sample.abs() > 1e-6 {
                let pan = bowl.pan_position;
                let left_gain = 0.5 * (1.0 - pan);
                let right_gain = 0.5 * (1.0 + pan);

                total_left += sample * left_gain;
                total_right += sample * right_gain;
                chassis_drive += sample.abs();
            }
        }

        // Wooden chassis cavity resonator
        let chassis_out = self.chassis.process(chassis_drive * self.chassis_coupling_gain * 0.25);
        total_left += chassis_out * 0.30;
        total_right += chassis_out * 0.30;

        let out_l = (total_left * self.master_gain).clamp(-1.0, 1.0);
        let out_r = (total_right * self.master_gain).clamp(-1.0, 1.0);

        (out_l, out_r)
    }

    pub fn reset(&mut self) {
        for bowl in self.bowls.iter_mut() {
            bowl.reset();
        }
        self.chassis.reset();
    }
}

impl SignalProcessor for GlassArmonica {
    fn name(&self) -> &str {
        "GlassArmonicaSynthesizer"
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

impl AudioNode for GlassArmonica {
    fn name(&self) -> &str {
        "GlassArmonicaSynthesizer"
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
    fn test_glass_armonica_instantiation_and_37_bowls() {
        let armonica = GlassArmonica::new(48000);
        assert_eq!(armonica.bowls.len(), 37);
        assert_eq!(armonica.bowls[0].midi_note, 60); // C4
        assert_eq!(armonica.bowls[36].midi_note, 96); // C7

        // Lowest bowl radius > highest bowl radius
        assert!(armonica.bowls[0].bowl_radius_m > armonica.bowls[36].bowl_radius_m);
    }

    #[test]
    fn test_glass_armonica_zero_allocations_in_audio_loop() {
        let mut armonica = GlassArmonica::new(48000);
        armonica.set_spindle_speed(2.5 * PI);
        armonica.touch_note(60, 0.50); // C4
        armonica.touch_note(64, 0.45); // E4
        armonica.touch_note(67, 0.40); // G4

        let _guard = AllocGuard::new();
        for _ in 0..1024 {
            let (l, r) = armonica.process_stereo_frame();
            assert!(l.abs() <= 1.0);
            assert!(r.abs() <= 1.0);
        }
    }

    #[test]
    fn test_glass_armonica_polyphonic_chords_and_release() {
        let mut armonica = GlassArmonica::new(48000);
        armonica.set_spindle_speed(3.0 * PI);

        armonica.note_on(60, 0.80);
        armonica.note_on(67, 0.70);

        let mut sound_detected = false;
        for _ in 0..2048 {
            let (l, r) = armonica.process_stereo_frame();
            if l.abs() > 0.001 || r.abs() > 0.001 {
                sound_detected = true;
            }
        }
        assert!(sound_detected, "Touching rotating bowls must generate audible friction sound");

        armonica.note_off(60);
        armonica.note_off(67);
        assert!(!armonica.bowls[0].is_touched);
        assert!(!armonica.bowls[7].is_touched);
    }
}
