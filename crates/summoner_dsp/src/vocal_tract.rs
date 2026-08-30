// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! 1D 44-Cylinder Finite-Difference Time-Domain (FDTD) Acoustic Waveguide Vocal Tract (Milestone 15).
//!
//! Models the acoustic wave propagation, scattering junctions, velum branching, and lip/nose
//! radiation of the human vocal tract as a 44-cylinder Kelly-Lochbaum waveguide network coupled
//! with a 16-cylinder nasal tract.
//!
//! Enforces zero heap allocations in audio processing loops under `AllocGuard`.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use std::fmt;
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};
use crate::glottal_pulse::GlottalPulse;
use crate::traits::SignalProcessor;

/// Number of cylindrical sections in the main oral/pharyngeal vocal tract (~17.5 cm).
pub const NUM_CYLINDERS: usize = 44;

/// Number of cylindrical sections in the nasal side branch tract (~12.5 cm).
pub const NUM_NASAL_CYLINDERS: usize = 16;

/// Cylinder index where the velopharyngeal port branches into the nasal tract.
pub const VELUM_BRANCH_INDEX: usize = 17;

mod serde_cylinder_array {
    use super::*;

    pub fn serialize<S>(data: &[f32; NUM_CYLINDERS], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(NUM_CYLINDERS))?;
        for elem in data.iter() {
            seq.serialize_element(elem)?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[f32; NUM_CYLINDERS], D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ArrayVisitor;
        impl<'de> Visitor<'de> for ArrayVisitor {
            type Value = [f32; NUM_CYLINDERS];
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an array of 44 f32 elements")
            }
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut arr = [2.5f32; NUM_CYLINDERS];
                let mut idx = 0;
                while let Some(val) = seq.next_element()? {
                    if idx < NUM_CYLINDERS {
                        arr[idx] = val;
                        idx += 1;
                    }
                }
                Ok(arr)
            }
        }
        deserializer.deserialize_seq(ArrayVisitor)
    }
}

/// Preset standard phonetic vowel shapes for 44-cylinder cross-sectional area configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum VowelPreset {
    /// /i/ as in "beet" (high front: wide pharynx, tight palatal constriction, spread lips).
    #[default]
    I,
    /// /e/ as in "bait" (mid-high front).
    E,
    /// /ɛ/ as in "bet" (open-mid front).
    Epsilon,
    /// /æ/ as in "bat" (low front).
    Ash,
    /// /a/ as in "father" (low back: constricted pharynx, wide open mouth/lips).
    A,
    /// /ɔ/ as in "bought" (open-mid back rounded).
    OpenO,
    /// /o/ as in "boat" (close-mid back rounded).
    O,
    /// /u/ as in "boot" (high back: narrow velar constriction, rounded protruded lips).
    U,
    /// /ə/ as in "sofa" / neutral schwa (uniform cylindrical tube).
    Schwa,
}

impl VowelPreset {
    /// Computes nominal 44-cylinder cross-sectional areas in $\text{cm}^2$.
    pub fn cross_sectional_areas(&self) -> [f32; NUM_CYLINDERS] {
        let mut a = [2.5f32; NUM_CYLINDERS];
        match self {
            Self::Schwa => {
                // Uniform 2.5 cm^2 tube with slight natural bell opening
                for (i, val) in a.iter_mut().enumerate() {
                    let x = i as f32 / (NUM_CYLINDERS - 1) as f32;
                    *val = 2.0 + 0.8 * x;
                }
            }
            Self::I => {
                // /i/: Wide pharynx (cylinders 0..16), tight constriction at palate (cylinders 26..36), wide lips
                for (i, val) in a.iter_mut().enumerate() {
                    if i < 16 {
                        *val = 4.5 - 0.1 * (i as f32);
                    } else if i < 34 {
                        let t = (i - 16) as f32 / 18.0;
                        *val = 3.0 * (1.0 - t) + 0.45 * t;
                    } else {
                        let t = (i - 34) as f32 / 10.0;
                        *val = 0.45 * (1.0 - t) + 3.2 * t;
                    }
                }
            }
            Self::E => {
                // /e/: Moderately wide pharynx, front constriction, open lips
                for (i, val) in a.iter_mut().enumerate() {
                    if i < 16 {
                        *val = 3.8 - 0.05 * (i as f32);
                    } else if i < 32 {
                        let t = (i - 16) as f32 / 16.0;
                        *val = 3.0 * (1.0 - t) + 0.9 * t;
                    } else {
                        let t = (i - 32) as f32 / 12.0;
                        *val = 0.9 * (1.0 - t) + 3.5 * t;
                    }
                }
            }
            Self::Epsilon => {
                // /ɛ/: Open-mid front vowel
                for (i, val) in a.iter_mut().enumerate() {
                    if i < 18 {
                        *val = 3.2;
                    } else if i < 32 {
                        let t = (i - 18) as f32 / 14.0;
                        *val = 3.2 * (1.0 - t) + 1.4 * t;
                    } else {
                        let t = (i - 32) as f32 / 12.0;
                        *val = 1.4 * (1.0 - t) + 4.0 * t;
                    }
                }
            }
            Self::Ash => {
                // /æ/: Low front vowel with wider oral opening
                for (i, val) in a.iter_mut().enumerate() {
                    if i < 18 {
                        *val = 2.4;
                    } else if i < 30 {
                        *val = 2.0;
                    } else {
                        let t = (i - 30) as f32 / 14.0;
                        *val = 2.0 + 3.5 * t;
                    }
                }
            }
            Self::A => {
                // /a/: Constricted lower pharynx (cylinders 0..16), wide flare towards lips (cylinders 20..43)
                for (i, val) in a.iter_mut().enumerate() {
                    if i < 16 {
                        let t = i as f32 / 16.0;
                        *val = 0.6 + 0.4 * t;
                    } else {
                        let t = (i - 16) as f32 / 28.0;
                        *val = 1.0 + 5.5 * t * t;
                    }
                }
            }
            Self::OpenO => {
                // /ɔ/: Constricted pharynx, medium oral cavity, rounded lips
                for (i, val) in a.iter_mut().enumerate() {
                    if i < 16 {
                        *val = 0.9 + 0.05 * (i as f32);
                    } else if i < 34 {
                        *val = 3.5;
                    } else {
                        let t = (i - 34) as f32 / 10.0;
                        *val = 3.5 * (1.0 - t) + 1.2 * t;
                    }
                }
            }
            Self::O => {
                // /o/: Low-mid back constricted pharynx, rounded closed lips
                for (i, val) in a.iter_mut().enumerate() {
                    if i < 18 {
                        *val = 1.2 + 0.08 * (i as f32);
                    } else if i < 36 {
                        *val = 4.2;
                    } else {
                        let t = (i - 36) as f32 / 8.0;
                        *val = 4.2 * (1.0 - t) + 0.7 * t;
                    }
                }
            }
            Self::U => {
                // /u/: Wide pharynx (cylinders 0..16), narrow constriction at velum (cylinders 18..28), tight rounded lips (cylinders 38..43)
                for (i, val) in a.iter_mut().enumerate() {
                    if i < 16 {
                        *val = 3.6 - 0.05 * (i as f32);
                    } else if i < 28 {
                        let t = (i - 16) as f32 / 12.0;
                        *val = 2.8 * (1.0 - t) + 0.55 * t;
                    } else if i < 38 {
                        *val = 3.2;
                    } else {
                        let t = (i - 38) as f32 / 6.0;
                        *val = 3.2 * (1.0 - t) + 0.40 * t;
                    }
                }
            }
        }
        a
    }
}

fn default_cylinders() -> [f32; NUM_CYLINDERS] {
    [0.0; NUM_CYLINDERS]
}

fn default_nasal_cylinders() -> [f32; NUM_NASAL_CYLINDERS] {
    [0.0; NUM_NASAL_CYLINDERS]
}

/// 1D 44-Cylinder FDTD Acoustic Vocal Tract Modeler.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocalTract {
    /// Cross-sectional area function $A_k$ in $\text{cm}^2$ for 44 main cylinders.
    #[serde(with = "serde_cylinder_array")]
    pub areas: [f32; NUM_CYLINDERS],
    /// Cross-sectional areas in $\text{cm}^2$ for 16 nasal tract cylinders.
    pub nasal_areas: [f32; NUM_NASAL_CYLINDERS],
    /// Velum port opening factor $[0.0 ..= 1.0]$ (0.0 = oral only, 1.0 = full nasal coupling).
    pub velum_opening: f32,
    /// Tongue position normalized along tract $[0.0 ..= 1.0]$ (0 = back pharynx, 1 = dental/lips).
    pub tongue_position: f32,
    /// Tongue constriction height $[0.0 ..= 1.0]$ (0 = low/flat, 1 = high palate constriction).
    pub tongue_height: f32,
    /// Lip opening modifier $[0.1 ..= 2.0]$.
    pub lip_opening: f32,
    /// Internal glottal excitation pulse generator.
    pub glottis: GlottalPulse,
    /// Master acoustic output gain.
    pub master_gain: f32,
    /// Audio sample rate.
    pub sample_rate: u32,

    // Internal FDTD waveguide wave states
    #[serde(skip, default = "default_cylinders")]
    fwd: [f32; NUM_CYLINDERS],
    #[serde(skip, default = "default_cylinders")]
    rev: [f32; NUM_CYLINDERS],
    #[serde(skip, default = "default_nasal_cylinders")]
    nasal_fwd: [f32; NUM_NASAL_CYLINDERS],
    #[serde(skip, default = "default_nasal_cylinders")]
    nasal_rev: [f32; NUM_NASAL_CYLINDERS],

    // Next buffer step states (double buffered for synchronous FDTD update)
    #[serde(skip, default = "default_cylinders")]
    next_fwd: [f32; NUM_CYLINDERS],
    #[serde(skip, default = "default_cylinders")]
    next_rev: [f32; NUM_CYLINDERS],
    #[serde(skip, default = "default_nasal_cylinders")]
    next_nasal_fwd: [f32; NUM_NASAL_CYLINDERS],
    #[serde(skip, default = "default_nasal_cylinders")]
    next_nasal_rev: [f32; NUM_NASAL_CYLINDERS],

    // Radiation filter memory states
    #[serde(skip)]
    prev_lip_out: f32,
    #[serde(skip)]
    prev_nose_out: f32,
}

impl Default for VocalTract {
    fn default() -> Self {
        Self::new(48000)
    }
}

impl VocalTract {
    /// Creates a new Vocal Tract FDTD modeler for the given sample rate.
    pub fn new(sample_rate: u32) -> Self {
        let sr = sample_rate.max(8000);
        let default_preset = VowelPreset::I;
        let mut nasal_areas = [1.2f32; NUM_NASAL_CYLINDERS];
        // Standard tapering nasal cavity
        for (i, val) in nasal_areas.iter_mut().enumerate() {
            let t = i as f32 / (NUM_NASAL_CYLINDERS - 1) as f32;
            *val = 0.8 + 1.2 * (1.0 - (t - 0.5).powi(2) * 4.0).max(0.0);
        }

        Self {
            areas: default_preset.cross_sectional_areas(),
            nasal_areas,
            velum_opening: 0.0,
            tongue_position: 0.70,
            tongue_height: 0.85,
            lip_opening: 1.0,
            glottis: GlottalPulse::new(sr),
            master_gain: 1.0,
            sample_rate: sr,
            fwd: [0.0; NUM_CYLINDERS],
            rev: [0.0; NUM_CYLINDERS],
            nasal_fwd: [0.0; NUM_NASAL_CYLINDERS],
            nasal_rev: [0.0; NUM_NASAL_CYLINDERS],
            next_fwd: [0.0; NUM_CYLINDERS],
            next_rev: [0.0; NUM_CYLINDERS],
            next_nasal_fwd: [0.0; NUM_NASAL_CYLINDERS],
            next_nasal_rev: [0.0; NUM_NASAL_CYLINDERS],
            prev_lip_out: 0.0,
            prev_nose_out: 0.0,
        }
    }

    /// Sets the vocal tract area function to a predefined vowel shape.
    pub fn set_vowel_preset(&mut self, preset: VowelPreset) {
        self.areas = preset.cross_sectional_areas();
    }

    /// Updates the 44-cylinder area function dynamically based on tongue position and height.
    pub fn update_tongue_articulation(&mut self, position: f32, height: f32) {
        self.tongue_position = position.clamp(0.0, 1.0);
        self.tongue_height = height.clamp(0.0, 1.0);

        // Center cylinder index of tongue constriction (between cylinder 12 and 36)
        let center_idx = 12.0 + self.tongue_position * 24.0;
        let width = 6.0; // Constriction width in cylinders
        let min_area = 0.35 + (1.0 - self.tongue_height) * 2.8;

        for (i, area) in self.areas.iter_mut().enumerate() {
            let dist = (i as f32 - center_idx).abs();
            if dist < width {
                let factor = (1.0 - (dist / width)).powi(2);
                let constricted = 2.5 * (1.0 - factor) + min_area * factor;
                *area = constricted.max(0.15);
            } else {
                *area = 2.5;
            }
        }

        // Apply lip opening modifier at final cylinders
        for i in 38..NUM_CYLINDERS {
            self.areas[i] = (self.areas[i] * self.lip_opening).clamp(0.1, 8.0);
        }
    }

    /// Sets the velum opening (nasalization) $[0.0 ..= 1.0]$.
    pub fn set_velum_opening(&mut self, opening: f32) {
        self.velum_opening = opening.clamp(0.0, 1.0);
    }

    /// Sets the lip opening factor $[0.1 ..= 2.5]$.
    pub fn set_lip_opening(&mut self, opening: f32) {
        self.lip_opening = opening.clamp(0.1, 2.5);
    }

    /// Resets all delay and pressure wave states to zero.
    pub fn reset(&mut self) {
        self.fwd = [0.0; NUM_CYLINDERS];
        self.rev = [0.0; NUM_CYLINDERS];
        self.nasal_fwd = [0.0; NUM_NASAL_CYLINDERS];
        self.nasal_rev = [0.0; NUM_NASAL_CYLINDERS];
        self.next_fwd = [0.0; NUM_CYLINDERS];
        self.next_rev = [0.0; NUM_CYLINDERS];
        self.next_nasal_fwd = [0.0; NUM_NASAL_CYLINDERS];
        self.next_nasal_rev = [0.0; NUM_NASAL_CYLINDERS];
        self.prev_lip_out = 0.0;
        self.prev_nose_out = 0.0;
        self.glottis.reset();
    }

    /// Processes one discrete sample of the FDTD waveguide network with external excitation.
    #[inline]
    pub fn process_sample(&mut self, external_excitation: f32) -> Sample {
        let wall_loss = 0.994f32;

        // 1. Compute glottal pulse excitation source
        let glottal_ex = self.glottis.process_sample();
        let total_excitation = (glottal_ex * 0.4 + external_excitation).clamp(-2.0, 2.0);

        // 2. Glottal boundary condition at cylinder 0 (reflection + source injection)
        let r_glottis = 0.65f32;
        self.next_fwd[0] = (total_excitation + r_glottis * self.rev[0]).clamp(-3.0, 3.0);

        // 3. Scattering junctions along the 44 main cylinders
        for k in 0..(NUM_CYLINDERS - 1) {
            if k + 1 == VELUM_BRANCH_INDEX && self.velum_opening > 0.001 {
                // 3-way acoustic scattering junction at velopharyngeal port
                let a_in = self.areas[k].max(0.1);
                let a_out = self.areas[k + 1].max(0.1);
                let a_nasal = self.nasal_areas[0].max(0.1) * self.velum_opening;
                let a_total = (a_in + a_out + a_nasal).max(0.1);

                // Junction pressure
                let p_j = (2.0 * (a_in * self.fwd[k] + a_out * self.rev[k + 1] + a_nasal * self.nasal_rev[0])) / a_total;

                self.next_rev[k] = ((p_j - self.fwd[k]) * wall_loss).clamp(-3.0, 3.0);
                self.next_fwd[k + 1] = ((p_j - self.rev[k + 1]) * wall_loss).clamp(-3.0, 3.0);
                self.next_nasal_fwd[0] = ((p_j - self.nasal_rev[0]) * wall_loss).clamp(-3.0, 3.0);
            } else {
                // Standard 2-cylinder Kelly-Lochbaum scattering junction
                let a1 = self.areas[k].max(0.1);
                let a2 = self.areas[k + 1].max(0.1);
                let r_k = ((a1 - a2) / (a1 + a2)).clamp(-0.95, 0.95);

                let w = r_k * (self.fwd[k] - self.rev[k + 1]);
                self.next_fwd[k + 1] = ((self.fwd[k] - w) * wall_loss).clamp(-3.0, 3.0);
                self.next_rev[k] = ((self.rev[k + 1] + w) * wall_loss).clamp(-3.0, 3.0);
            }
        }

        // 4. Scattering junctions along the 16 nasal cylinders
        if self.velum_opening > 0.001 {
            for n in 0..(NUM_NASAL_CYLINDERS - 1) {
                let a1 = self.nasal_areas[n].max(0.1);
                let a2 = self.nasal_areas[n + 1].max(0.1);
                let r_n = ((a1 - a2) / (a1 + a2)).clamp(-0.95, 0.95);

                let w = r_n * (self.nasal_fwd[n] - self.nasal_rev[n + 1]);
                self.next_nasal_fwd[n + 1] = ((self.nasal_fwd[n] - w) * wall_loss).clamp(-3.0, 3.0);
                self.next_nasal_rev[n] = ((self.nasal_rev[n + 1] + w) * wall_loss).clamp(-3.0, 3.0);
            }
        }

        // 5. Lip radiation boundary (open end reflection and 1st-order highpass radiation)
        let r_lip = -0.80f32;
        let lip_fwd = self.fwd[NUM_CYLINDERS - 1];
        self.next_rev[NUM_CYLINDERS - 1] = (lip_fwd * r_lip * wall_loss).clamp(-3.0, 3.0);

        let lip_radiated = (lip_fwd * (1.0 + r_lip) - self.prev_lip_out * 0.45).clamp(-2.0, 2.0);
        self.prev_lip_out = lip_fwd.clamp(-3.0, 3.0);

        // 6. Nose radiation boundary
        let nose_radiated = if self.velum_opening > 0.001 {
            let r_nose = -0.80f32;
            let nose_fwd = self.nasal_fwd[NUM_NASAL_CYLINDERS - 1];
            self.next_nasal_rev[NUM_NASAL_CYLINDERS - 1] = (nose_fwd * r_nose * wall_loss).clamp(-3.0, 3.0);

            let rad = (nose_fwd * (1.0 + r_nose) - self.prev_nose_out * 0.45).clamp(-2.0, 2.0);
            self.prev_nose_out = nose_fwd.clamp(-3.0, 3.0);
            rad
        } else {
            0.0
        };

        // 7. Synchronous update of FDTD buffer states
        self.fwd.copy_from_slice(&self.next_fwd);
        self.rev.copy_from_slice(&self.next_rev);
        self.nasal_fwd.copy_from_slice(&self.next_nasal_fwd);
        self.nasal_rev.copy_from_slice(&self.next_nasal_rev);

        // Combined radiated output scaled by master gain
        let total_output = (lip_radiated + nose_radiated) * self.master_gain * 3.5;
        total_output.clamp(-1.0, 1.0)
    }
}

impl SignalProcessor for VocalTract {
    fn name(&self) -> &str {
        "VocalTract"
    }

    fn process_block(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        _ctx: &ProcessContext,
    ) {
        if outputs.is_empty() {
            return;
        }

        let num_samples = outputs[0].len();
        for i in 0..num_samples {
            let ext_in = if !inputs.is_empty() && !inputs[0].is_empty() && i < inputs[0].len() {
                inputs[0][i]
            } else {
                0.0
            };

            let out_sample = self.process_sample(ext_in);
            for ch in outputs.iter_mut() {
                if i < ch.len() {
                    ch[i] = out_sample;
                }
            }
        }
    }
}

/// AudioNode wrapper for 44-Cylinder FDTD Vocal Tract Modeler.
#[derive(Debug)]
pub struct VocalTractNode {
    pub tract: VocalTract,
}

impl VocalTractNode {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            tract: VocalTract::new(sample_rate),
        }
    }
}

impl AudioNode for VocalTractNode {
    fn name(&self) -> &str {
        "VocalTractNode"
    }

    fn process(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        self.tract.process_block(inputs, outputs, ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::allocator::AllocGuard;
    use summoner_core::transport::Transport;

    #[test]
    fn test_vocal_tract_fdtd_cylinders_resonance_and_stability() {
        let mut tract = VocalTract::new(48000);
        tract.set_vowel_preset(VowelPreset::A);
        tract.glottis.set_frequency(180.0);

        let mut has_sound = false;
        for _ in 0..2000 {
            let sample = tract.process_sample(0.0);
            assert!(sample.is_finite());
            assert!((-1.0..=1.0).contains(&sample));
            if sample.abs() > 0.05 {
                has_sound = true;
            }
        }

        assert!(has_sound, "Vocal tract must produce resonant vowel sound");
    }

    #[test]
    fn test_vocal_tract_vowel_presets_and_velum() {
        let mut tract = VocalTract::new(48000);

        for vowel in [
            VowelPreset::I,
            VowelPreset::E,
            VowelPreset::Epsilon,
            VowelPreset::Ash,
            VowelPreset::A,
            VowelPreset::OpenO,
            VowelPreset::O,
            VowelPreset::U,
            VowelPreset::Schwa,
        ] {
            tract.set_vowel_preset(vowel);
            tract.set_velum_opening(0.3);
            for _ in 0..100 {
                let sample = tract.process_sample(0.0);
                assert!(sample.is_finite());
                assert!((-1.0..=1.0).contains(&sample));
            }
        }
    }

    #[test]
    fn test_vocal_tract_zero_allocation_in_process_block() {
        let mut tract = VocalTract::new(48000);
        let transport = Transport::new(48000, 120.0);
        let ctx = ProcessContext::from_transport(&transport);

        let in_buf = [0.0f32; 64];
        let mut out_l = [0.0f32; 64];
        let mut out_r = [0.0f32; 64];

        {
            let _guard = AllocGuard::new();
            tract.process_block(
                &[&in_buf[..]],
                &mut [&mut out_l[..], &mut out_r[..]],
                &ctx,
            );
        }

        assert!(out_l.iter().all(|s| s.is_finite()));
        assert!(out_l.iter().any(|s| *s != 0.0));
    }
}
