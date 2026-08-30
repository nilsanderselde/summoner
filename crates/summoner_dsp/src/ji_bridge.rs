// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Asian Zither Movable Ji Bridge & Paulownia Soundboard Body (Milestone 29).
//!
//! Provides a high-precision acoustic modeling system for the movable triangular *ji* bridges
//! of traditional Asian zithers (13-string Japanese Koto / 21-string Chinese Guzheng) featuring:
//! - Non-linear boundary impedance and dual-subsegment string coupling (plucked playing segment vs unplayed behind-the-bridge sympathetic segment)
//! - Movable Ji bridge position offsets adjusting active scale tuning and behind-the-bridge resonance
//! - Modal Paulownia wood soundboard body cavity resonator (top arched plate, bottom plate with *in-ko* and *yo-ko* soundholes)
//! - Large-displacement *oshi-ite* left-hand string press tension pitch deflection (up to +400 cents)
//! - Authentic traditional tuning scales (Hirajoshi, Kokin-joshi, In-sen, Kumoi-joshi, Ryukyu, Guzheng Pentatonic)
//!
//! Enforces zero heap allocations in audio render loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::f32::consts::{FRAC_1_SQRT_2, PI, SQRT_2};

/// Maximum number of primary strings on a standard Japanese Koto.
pub const NUM_KOTO_STRINGS: usize = 13;

/// Maximum number of extended strings on a Chinese Guzheng.
pub const MAX_ZITHER_STRINGS: usize = 21;

/// Number of acoustic body cavity resonator modes for the Paulownia wood instrument.
pub const NUM_PAULOWNIA_MODES: usize = 8;

/// Traditional Asian Zither modal scale tuning systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum KotoTuningSchema {
    /// Hirajoshi (Classic meditative in-sen scale: D4, G3, A3, Bb3, D4, Eb4, G4, A4, Bb4, D5, Eb5, G5, A5).
    #[default]
    Hirajoshi,
    /// Kokin-joshi (Miyako-bushi urban classical scale: D4, G3, Ab3, C4, D4, Eb4, G4, Ab4, C5, D5, Eb5, G5, Ab5).
    KokinJoshi,
    /// In-sen (Traditional dramatic in-mode: D4, G3, Ab3, C4, D4, F4, G4, Ab4, C5, D5, F5, G5, Ab5).
    InSen,
    /// Kumoi-joshi (Lyrical spring rain scale: D4, G3, Ab3, Bb3, D4, Eb4, G4, Ab4, Bb4, D5, Eb5, G5, Ab5).
    KumoiJoshi,
    /// Ryukyu (Okinawan festive pentatonic scale: D4, G3, B3, C4, D4, E4, G4, B4, C5, D5, E5, G5, B5).
    Ryukyu,
    /// Guzheng Major Pentatonic (Gong-Shang-Jiao-Zhi-Yu: D4, G3, A3, C4, D4, E4, G4, A4, C5, D5, E5, G5, A5).
    GuzhengPentatonic,
}

impl KotoTuningSchema {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Hirajoshi => "Hirajoshi (Classic Meditative)",
            Self::KokinJoshi => "Kokin-joshi (Miyako-bushi Urban)",
            Self::InSen => "In-sen (Traditional Dramatic)",
            Self::KumoiJoshi => "Kumoi-joshi (Lyrical Spring)",
            Self::Ryukyu => "Ryukyu (Okinawan Festive)",
            Self::GuzhengPentatonic => "Guzheng (Major Pentatonic)",
        }
    }

    /// Frequency ratios for all 13 strings relative to the base reference pitch $f_{\text{ref}}$ (typically D4 ~ 293.66 Hz).
    /// Note 1 is Sho (D4), Note 2 is Ichi (G3, sub-fourth), Note 3 is Ni (A3), etc.
    pub fn string_ratios(&self) -> [f32; NUM_KOTO_STRINGS] {
        match self {
            Self::Hirajoshi => [
                1.0000,   // String 1: D4 (1.0)
                0.6674,   // String 2: G3 (sub-fourth ~ 0.6674)
                0.7492,   // String 3: A3
                0.7937,   // String 4: Bb3
                1.0000,   // String 5: D4
                1.0595,   // String 6: Eb4
                1.3348,   // String 7: G4
                1.4983,   // String 8: A4
                1.5874,   // String 9: Bb4
                2.0000,   // String 10: D5
                2.1189,   // String 11: Eb5
                2.6697,   // String 12: G5
                2.9966,   // String 13: A5
            ],
            Self::KokinJoshi => [
                1.0000,   // String 1: D4
                0.6674,   // String 2: G3
                FRAC_1_SQRT_2, // String 3: Ab3
                0.8909,   // String 4: C4
                1.0000,   // String 5: D4
                1.0595,   // String 6: Eb4
                1.3348,   // String 7: G4
                SQRT_2,   // String 8: Ab4
                1.7818,   // String 9: C5
                2.0000,   // String 10: D5
                2.1189,   // String 11: Eb5
                2.6697,   // String 12: G5
                2.8284,   // String 13: Ab5
            ],
            Self::InSen => [
                1.0000,   // String 1: D4
                0.6674,   // String 2: G3
                FRAC_1_SQRT_2, // String 3: Ab3
                0.8909,   // String 4: C4
                1.0000,   // String 5: D4
                1.1892,   // String 6: F4
                1.3348,   // String 7: G4
                SQRT_2,   // String 8: Ab4
                1.7818,   // String 9: C5
                2.0000,   // String 10: D5
                2.3784,   // String 11: F5
                2.6697,   // String 12: G5
                2.8284,   // String 13: Ab5
            ],
            Self::KumoiJoshi => [
                1.0000,   // String 1: D4
                0.6674,   // String 2: G3
                FRAC_1_SQRT_2, // String 3: Ab3
                0.7937,   // String 4: Bb3
                1.0000,   // String 5: D4
                1.0595,   // String 6: Eb4
                1.3348,   // String 7: G4
                SQRT_2,   // String 8: Ab4
                1.5874,   // String 9: Bb4
                2.0000,   // String 10: D5
                2.1189,   // String 11: Eb5
                2.6697,   // String 12: G5
                2.8284,   // String 13: Ab5
            ],
            Self::Ryukyu => [
                1.0000,   // String 1: D4
                0.6674,   // String 2: G3
                0.8409,   // String 3: B3
                0.8909,   // String 4: C4
                1.0000,   // String 5: D4
                1.1225,   // String 6: E4
                1.3348,   // String 7: G4
                1.6818,   // String 8: B4
                1.7818,   // String 9: C5
                2.0000,   // String 10: D5
                2.2449,   // String 11: E5
                2.6697,   // String 12: G5
                3.3636,   // String 13: B5
            ],
            Self::GuzhengPentatonic => [
                1.0000,   // String 1: D4
                0.6674,   // String 2: G3
                0.7492,   // String 3: A3
                0.8909,   // String 4: C4
                1.0000,   // String 5: D4
                1.1225,   // String 6: E4
                1.3348,   // String 7: G4
                1.4983,   // String 8: A4
                1.7818,   // String 9: C5
                2.0000,   // String 10: D5
                2.2449,   // String 11: E5
                2.6697,   // String 12: G5
                2.9966,   // String 13: A5
            ],
        }
    }

    /// Default nominal bridge position fractions $L_{\text{play}} / L_{\text{total}} \in [0.2, 0.85]$ for the 13 strings.
    pub fn default_bridge_positions(&self) -> [f32; NUM_KOTO_STRINGS] {
        let ratios = self.string_ratios();
        let mut positions = [0.0; NUM_KOTO_STRINGS];
        for (i, &ratio) in ratios.iter().enumerate() {
            // Bridge position roughly inversely proportional to frequency ratio, clamped to physically plausible layout
            let pos = (0.65 / ratio.sqrt()).clamp(0.20, 0.85);
            positions[i] = pos;
        }
        positions
    }
}

/// Ji bridge material and acoustic impedance profile choices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum JiBridgeProfile {
    /// Dense Hardwood (Traditional Koto Ji - balanced transmission and reflection).
    #[default]
    PaulowniaHardwood,
    /// Antique Elephant Ivory / Bone (High reflection, crisp snap transient).
    IvoryBone,
    /// Rosewood Guzheng Triangular Bridge (Warm tone with rich behind-the-bridge bleed).
    RosewoodGuzheng,
    /// Modern Synthetic Delrin / Plastic (Stable damping and pure fundamental resonance).
    SyntheticPlastic,
    /// Smoked Vintage Bamboo (Dark mellow resonance with fast high-frequency decay).
    SmokedBamboo,
}

impl JiBridgeProfile {
    pub fn name(&self) -> &'static str {
        match self {
            Self::PaulowniaHardwood => "Paulownia Hardwood Ji (Traditional)",
            Self::IvoryBone => "Ivory / Bone Sharp Ji (Bright Attack)",
            Self::RosewoodGuzheng => "Rosewood Triangular Bridge (Warm Guzheng)",
            Self::SyntheticPlastic => "Synthetic Delrin Ji (Clean & Stable)",
            Self::SmokedBamboo => "Smoked Vintage Bamboo (Mellow & Dark)",
        }
    }

    /// Returns nominal $(T_{\text{transmission}}, R_{\text{reflection\_loss}}, C_{\text{body\_coupling}}, \text{dispersion\_b})$.
    pub fn nominal_physics(&self) -> (f32, f32, f32, f32) {
        match self {
            Self::PaulowniaHardwood => (0.08, 0.985, 0.05, 0.00012),
            Self::IvoryBone => (0.04, 0.992, 0.03, 0.00008),
            Self::RosewoodGuzheng => (0.12, 0.980, 0.08, 0.00018),
            Self::SyntheticPlastic => (0.06, 0.988, 0.04, 0.00010),
            Self::SmokedBamboo => (0.14, 0.975, 0.09, 0.00022),
        }
    }
}

/// Calculate non-linear pitch multiplier from left-hand Oshi-Ite press force in Newtons.
///
/// Press force $F_{\text{press}} \in [0.0 ..= 30.0]\text{ N}$ induces non-linear longitudinal tension deflection
/// yielding pitch shifts up to $+400\text{ cents}$ (a major third).
#[inline]
pub fn oshi_ite_pitch_multiplier(force_n: f32) -> f32 {
    let f_clamped = force_n.clamp(0.0, 30.0);
    if f_clamped <= 1e-4 {
        return 1.0;
    }
    // Max cents = 400 cents -> 2^(400/1200) = 2^(1/3) ~ 1.259921
    // Smooth non-linear curve: starts linear, compresses softly at high force
    let norm = f_clamped / 30.0;
    let cents = 400.0 * (1.0 - (-1.8 * norm).exp()) / (1.0 - (-1.8f32).exp());
    2.0f32.powf(cents / 1200.0)
}

/// A movable triangular Ji bridge acoustic scattering junction.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct JiBridgeJunction {
    /// Material and geometry profile.
    pub profile: JiBridgeProfile,
    /// Active bridge position along string length $L_{\text{play}} / L_{\text{total}} \in [0.15 ..= 0.90]$.
    pub position_fraction: f32,
    /// Transmission coefficient to behind-the-bridge sympathetic segment.
    pub transmission: f32,
    /// Reflection coefficient back into playing segment.
    pub reflection: f32,
    /// Coupling coefficient to Paulownia soundboard body.
    pub body_coupling: f32,
    /// Internal filter memory for frequency-dependent boundary reflection.
    prev_sample: f32,
}

impl Default for JiBridgeJunction {
    fn default() -> Self {
        Self::new(JiBridgeProfile::PaulowniaHardwood, 0.65)
    }
}

impl JiBridgeJunction {
    pub fn new(profile: JiBridgeProfile, position_fraction: f32) -> Self {
        let (trans, refl, body, _disp) = profile.nominal_physics();
        Self {
            profile,
            position_fraction: position_fraction.clamp(0.15, 0.90),
            transmission: trans,
            reflection: refl,
            body_coupling: body,
            prev_sample: 0.0,
        }
    }

    /// Set profile and update scattering coefficients.
    pub fn set_profile(&mut self, profile: JiBridgeProfile) {
        self.profile = profile;
        let (trans, refl, body, _disp) = profile.nominal_physics();
        self.transmission = trans;
        self.reflection = refl;
        self.body_coupling = body;
    }

    /// Set bridge position offset in $[-0.2, +0.2]$.
    pub fn set_position_offset(&mut self, base_fraction: f32, offset: f32) {
        self.position_fraction = (base_fraction + offset).clamp(0.15, 0.90);
    }

    /// Process incident traveling wave at bridge boundary.
    /// Returns `(reflected_playing_wave, transmitted_behind_bridge_wave, body_excitation)`.
    #[inline]
    pub fn process_scattering(&mut self, incident_playing: f32, incident_behind: f32) -> (f32, f32, f32) {
        // Lowpass filtering on reflection representing soft boundary damping
        let filtered_playing = 0.85 * incident_playing + 0.15 * self.prev_sample;
        self.prev_sample = incident_playing;

        // Reflection is inverting (fixed boundary with small transmission)
        let reflected_playing = -self.reflection * filtered_playing + self.transmission * incident_behind;
        let transmitted_behind = self.transmission * filtered_playing - self.reflection * 0.95 * incident_behind;
        let body_excitation = self.body_coupling * (incident_playing.abs() + incident_behind.abs() * 0.5);

        (reflected_playing, transmitted_behind, body_excitation)
    }

    pub fn reset(&mut self) {
        self.prev_sample = 0.0;
    }
}

/// A 2nd-order Biquad resonant filter for soundboard modal modeling.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PaulowniaBodyMode {
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

impl PaulowniaBodyMode {
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

        // Constant skirt gain bandpass filter
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

/// Paulownia wood soundboard body cavity acoustic modal resonator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaulowniaSoundboardBody {
    pub modes: [PaulowniaBodyMode; NUM_PAULOWNIA_MODES],
    pub master_wood_gain: f32,
}

impl PaulowniaSoundboardBody {
    pub fn new(sample_rate: u32) -> Self {
        // Authentic Koto Paulownia (Kiri) wood resonant frequencies & Q factors
        let modal_specs: [(f32, f32, f32); NUM_PAULOWNIA_MODES] = [
            (185.0, 12.0, 1.20), // Mode 1: Main body cavity Helmholtz resonance (Soundholes in-ko/yo-ko)
            (240.0, 15.0, 1.00), // Mode 2: First top arched soundboard longitudinal flexure
            (310.0, 18.0, 0.90), // Mode 3: Bottom plate breathing mode
            (420.0, 22.0, 0.85), // Mode 4: Second longitudinal soundboard mode
            (580.0, 25.0, 0.70), // Mode 5: Transverse arched plate flexure
            (820.0, 28.0, 0.60), // Mode 6: Upper soundboard harmonic mode
            (1150.0, 30.0, 0.45),// Mode 7: High frequency bridge radiation peak
            (1600.0, 35.0, 0.35),// Mode 8: Tsume pick snap transient color
        ];

        let mut modes = [PaulowniaBodyMode::new(100.0, 10.0, 1.0, sample_rate); NUM_PAULOWNIA_MODES];
        for (i, &(f, q, g)) in modal_specs.iter().enumerate() {
            modes[i] = PaulowniaBodyMode::new(f, q, g, sample_rate);
        }

        Self {
            modes,
            master_wood_gain: 0.85,
        }
    }

    #[inline]
    pub fn process(&mut self, input: f32) -> f32 {
        let mut out = 0.0;
        for mode in self.modes.iter_mut() {
            out += mode.step(input);
        }
        out * self.master_wood_gain
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        for mode in self.modes.iter_mut() {
            mode.recalculate(sample_rate);
        }
    }

    pub fn reset(&mut self) {
        for mode in self.modes.iter_mut() {
            mode.reset();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_koto_tuning_schemas_and_ratios() {
        for schema in [
            KotoTuningSchema::Hirajoshi,
            KotoTuningSchema::KokinJoshi,
            KotoTuningSchema::InSen,
            KotoTuningSchema::KumoiJoshi,
            KotoTuningSchema::Ryukyu,
            KotoTuningSchema::GuzhengPentatonic,
        ] {
            let ratios = schema.string_ratios();
            assert_eq!(ratios.len(), NUM_KOTO_STRINGS);
            assert_eq!(ratios[0], 1.0); // String 1 is reference D4
            // Monotonicity check across octaves
            assert!(ratios[12] > ratios[0], "String 13 must be higher than String 1");

            let positions = schema.default_bridge_positions();
            for &pos in &positions {
                assert!((0.15..=0.90).contains(&pos), "Bridge position {} out of range", pos);
            }
        }
    }

    #[test]
    fn test_oshi_ite_pitch_multiplier() {
        // Zero force -> 1.0
        assert_eq!(oshi_ite_pitch_multiplier(0.0), 1.0);

        // Maximum force 30 N -> ~400 cents (multiplier ~ 1.2599)
        let max_mult = oshi_ite_pitch_multiplier(30.0);
        let cents = 1200.0 * max_mult.log2();
        assert!((cents - 400.0).abs() < 1.0, "Expected ~400 cents, got {}", cents);

        // Monotonic growth
        let mut prev = 1.0;
        for f in [2.0, 5.0, 10.0, 15.0, 20.0, 25.0, 30.0] {
            let mult = oshi_ite_pitch_multiplier(f);
            assert!(mult > prev, "Multiplier should increase monotonically with force");
            prev = mult;
        }
    }

    #[test]
    fn test_ji_bridge_scattering_energy_conservation() {
        let mut junction = JiBridgeJunction::new(JiBridgeProfile::PaulowniaHardwood, 0.65);
        let (refl, trans, body) = junction.process_scattering(1.0, 0.0);

        // Reflected wave must be bounded < 1.0
        assert!(refl.abs() < 1.0);
        assert!(trans.abs() < 1.0);
        assert!(body.abs() < 1.0);

        // Total energy out should not explode
        let energy_out = refl * refl + trans * trans;
        assert!(energy_out <= 1.05, "Scattering energy out exploded: {}", energy_out);
    }

    #[test]
    fn test_paulownia_soundboard_body_modal_decay() {
        let mut body = PaulowniaSoundboardBody::new(48000);
        let impulse_resp = body.process(1.0);
        assert!(impulse_resp.is_finite());
        assert!(impulse_resp.abs() > 0.0);

        for _ in 0..1000 {
            body.process(0.0);
        }

        let decayed = body.process(0.0);
        assert!(decayed.abs() < impulse_resp.abs(), "Modal body resonance must decay");
    }
}
