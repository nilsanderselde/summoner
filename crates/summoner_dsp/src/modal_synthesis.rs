// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Dynamic Modal Synthesis Engine (Milestone 13).
//!
//! Provides a polyphonic bank of damped second-order resonant bandpass filters
//! coupled with multiple physical excitation generators (Strike, Bow, Pluck),
//! nonlinear slip-stick friction modeling, and material resonance presets
//! (MetalBar, MarimbaWood, GlassBowl, MembraneDrum, TibetanSingingBowl).
//!
//! Enforces zero heap allocations in audio processing loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};
use crate::modal::ModalResonator;
use crate::traits::SignalProcessor;

/// Maximum number of concurrent resonant modes in the modal filter bank.
pub const MAX_MODAL_BANK_SIZE: usize = 16;

/// Excitation mode for physical modal synthesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModalExcitationType {
    /// Percussive mallet strike with configurable hardness.
    Strike,
    /// Continuous friction bowing with slip-stick velocity curve.
    Bow,
    /// Sharp plucked release excitation.
    Pluck,
}

impl Default for ModalExcitationType {
    fn default() -> Self {
        Self::Strike
    }
}

/// Material resonance profile for modal frequency and damping distribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModalMaterialPreset {
    /// Metallic bar / vibraphone.
    MetalBar,
    /// Rosewood marimba bar with parabolic undercut.
    MarimbaWood,
    /// Resonant glass bowl / armonica.
    GlassBowl,
    /// Tensioned acoustic membrane drum head.
    MembraneDrum,
    /// Tibetan singing bowl with rich inharmonic overtones.
    TibetanSingingBowl,
}

impl Default for ModalMaterialPreset {
    fn default() -> Self {
        Self::MetalBar
    }
}

impl ModalMaterialPreset {
    /// Returns default frequency ratios for up to 16 modes.
    pub fn frequency_ratios(&self) -> [f32; MAX_MODAL_BANK_SIZE] {
        match self {
            Self::MetalBar => [
                1.000, 2.756, 5.404, 8.933, 13.344, 18.636, 24.810, 31.865,
                39.802, 48.621, 58.321, 68.903, 80.366, 92.711, 105.938, 120.046,
            ],
            Self::MarimbaWood => [
                1.000, 3.980, 9.870, 14.500, 19.800, 25.400, 31.900, 38.600,
                46.100, 54.200, 62.900, 72.100, 81.900, 92.300, 103.200, 114.700,
            ],
            Self::GlassBowl => [
                1.000, 2.320, 4.150, 6.480, 9.250, 12.450, 16.100, 20.200,
                24.700, 29.600, 34.900, 40.600, 46.700, 53.200, 60.100, 67.400,
            ],
            Self::MembraneDrum => [
                1.000, 1.593, 2.135, 2.295, 2.653, 2.917, 3.156, 3.500,
                3.650, 4.060, 4.150, 4.540, 4.600, 4.900, 5.000, 5.350,
            ],
            Self::TibetanSingingBowl => [
                1.000, 2.780, 5.120, 7.890, 11.230, 14.950, 19.100, 23.700,
                28.650, 34.100, 39.900, 46.200, 52.800, 59.900, 67.400, 75.300,
            ],
        }
    }

    /// Returns default damping factors for up to 16 modes.
    pub fn damping_factors(&self) -> [f32; MAX_MODAL_BANK_SIZE] {
        let base_damping = match self {
            Self::MetalBar => 4.5,
            Self::MarimbaWood => 12.0,
            Self::GlassBowl => 1.8,
            Self::MembraneDrum => 18.0,
            Self::TibetanSingingBowl => 0.8,
        };

        let mut dampings = [0.0f32; MAX_MODAL_BANK_SIZE];
        for i in 0..MAX_MODAL_BANK_SIZE {
            // High modes damp faster: d_i = d_0 * (1 + 0.15 * i^1.3)
            dampings[i] = base_damping * (1.0 + 0.15 * (i as f32).powf(1.3));
        }
        dampings
    }
}

/// Dynamic Modal Synthesis Engine with second-order resonant filter bank.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModalSynthesisEngine {
    #[serde(skip)]
    resonators: Vec<ModalResonator>,
    pub fundamental_hz: f32,
    pub excitation_type: ModalExcitationType,
    pub material_preset: ModalMaterialPreset,
    pub num_active_modes: usize,
    pub strike_position: f32,
    pub mallet_hardness: f32,
    pub bow_pressure: f32,
    pub bow_velocity: f32,
    pub is_excited: bool,
    #[serde(skip)]
    bow_phase: f32,
    #[serde(skip)]
    prng_seed: u32,
    pub sample_rate: u32,
}

impl Default for ModalSynthesisEngine {
    fn default() -> Self {
        Self::new(48000)
    }
}

impl ModalSynthesisEngine {
    pub fn new(sample_rate: u32) -> Self {
        let sr = sample_rate.max(8000);
        let mut engine = Self {
            resonators: Vec::with_capacity(MAX_MODAL_BANK_SIZE),
            fundamental_hz: 220.0,
            excitation_type: ModalExcitationType::Strike,
            material_preset: ModalMaterialPreset::MetalBar,
            num_active_modes: 8,
            strike_position: 0.35,
            mallet_hardness: 0.60,
            bow_pressure: 0.50,
            bow_velocity: 0.40,
            is_excited: false,
            bow_phase: 0.0,
            prng_seed: 0x5A5A5A5A,
            sample_rate: sr,
        };

        for _ in 0..MAX_MODAL_BANK_SIZE {
            engine.resonators.push(ModalResonator::new(220.0, 5.0, sr));
        }

        engine.update_modes();
        engine
    }

    /// Sets fundamental frequency in Hertz and updates all modal resonator coefficients.
    pub fn set_fundamental(&mut self, fundamental_hz: f32) {
        self.fundamental_hz = fundamental_hz.clamp(20.0, 8000.0);
        self.update_modes();
    }

    /// Sets material preset and reconfigures modal bank.
    pub fn set_material(&mut self, preset: ModalMaterialPreset) {
        self.material_preset = preset;
        self.update_modes();
    }

    /// Updates modal frequencies and damping based on preset and strike position.
    pub fn update_modes(&mut self) {
        let ratios = self.material_preset.frequency_ratios();
        let dampings = self.material_preset.damping_factors();
        let nyquist = self.sample_rate as f32 * 0.48;

        for i in 0..self.resonators.len().min(MAX_MODAL_BANK_SIZE) {
            let freq = (self.fundamental_hz * ratios[i]).clamp(20.0, nyquist);
            let damp = dampings[i];
            self.resonators[i].frequency = freq;
            self.resonators[i].damping = damp;
            self.resonators[i].update_coefficients(self.sample_rate);
        }
    }

    /// Triggers a percussive strike excitation.
    pub fn trigger_strike(&mut self, velocity: f32) {
        self.is_excited = true;
        let hardness = self.mallet_hardness.clamp(0.05, 1.0);
        let amp = velocity.clamp(0.0, 1.0) * hardness;

        for (i, res) in self.resonators.iter_mut().enumerate().take(self.num_active_modes) {
            let mode_pos = (i + 1) as f32 * self.strike_position * PI;
            let spatial_gain = mode_pos.sin().abs().clamp(0.05, 1.0);
            let impulse = amp * spatial_gain;
            let _ = res.process_sample(impulse);
        }
    }

    /// Generates friction-based bowing excitation force using hyperbolic secant slip-stick curve.
    #[inline]
    fn compute_bow_excitation(&mut self, feedback_velocity: f32) -> f32 {
        let v_rel = self.bow_velocity - feedback_velocity;
        // Non-linear friction curve: mu(v_rel) = sign(v_rel) * (0.2 + 0.8 * exp(-4.0 * |v_rel|))
        let friction = v_rel.signum() * (0.2 + 0.8 * (-4.0 * v_rel.abs()).exp());
        self.bow_pressure * friction
    }

    /// Computes one audio sample with zero heap allocation.
    #[inline]
    pub fn process_sample(&mut self, external_input: Sample) -> Sample {
        let mut total_output = 0.0f32;

        let excitation = match self.excitation_type {
            ModalExcitationType::Strike => external_input,
            ModalExcitationType::Bow => {
                let bow_force = self.compute_bow_excitation(self.bow_phase);
                external_input + bow_force * 0.15
            }
            ModalExcitationType::Pluck => external_input,
        };

        for (i, res) in self.resonators.iter_mut().enumerate().take(self.num_active_modes) {
            let mode_pos = (i + 1) as f32 * self.strike_position * PI;
            let spatial_coupling = mode_pos.sin().abs().clamp(0.05, 1.0);

            let mode_out = res.process_sample(excitation * spatial_coupling);
            total_output += mode_out;
            self.bow_phase += mode_out * (1.0 / (i + 1) as f32);
        }
        self.bow_phase *= 0.95; // Damp bow velocity feedback

        (total_output * 0.5).clamp(-1.0, 1.0)
    }

    /// Resets all modal filter states.
    pub fn reset(&mut self) {
        for res in &mut self.resonators {
            res.reset();
        }
        self.bow_phase = 0.0;
        self.is_excited = false;
    }
}

impl SignalProcessor for ModalSynthesisEngine {
    fn name(&self) -> &str {
        "ModalSynthesisEngine"
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
            let excitation = if !inputs.is_empty() && !inputs[0].is_empty() && i < inputs[0].len() {
                inputs[0][i]
            } else {
                0.0
            };

            let out_sample = self.process_sample(excitation);
            for ch in outputs.iter_mut() {
                if i < ch.len() {
                    ch[i] = out_sample;
                }
            }
        }
    }
}

/// AudioNode wrapper for Modal Synthesis Engine.
#[derive(Debug)]
pub struct ModalSynthesisNode {
    pub engine: ModalSynthesisEngine,
}

impl ModalSynthesisNode {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            engine: ModalSynthesisEngine::new(sample_rate),
        }
    }
}

impl AudioNode for ModalSynthesisNode {
    fn name(&self) -> &str {
        "ModalSynthesisNode"
    }

    fn process(
        &mut self,
        input: &[&[Sample]],
        output: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        self.engine.process_block(input, output, ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::allocator::AllocGuard;
    use summoner_core::transport::Transport;

    #[test]
    fn test_modal_synthesis_strike_and_decay() {
        let mut engine = ModalSynthesisEngine::new(48000);
        engine.set_fundamental(440.0);
        engine.set_material(ModalMaterialPreset::MetalBar);
        engine.trigger_strike(0.9);

        let initial_sample = engine.process_sample(0.0);
        assert!(initial_sample.abs() > 0.0);

        for _ in 0..5000 {
            let s = engine.process_sample(0.0);
            assert!(s.is_finite());
            assert!(s >= -1.0 && s <= 1.0);
        }

        let decayed_sample = engine.process_sample(0.0);
        assert!(decayed_sample.abs() < initial_sample.abs() || decayed_sample.abs() < 0.1);
    }

    #[test]
    fn test_modal_synthesis_zero_allocation_in_process_block() {
        let mut engine = ModalSynthesisEngine::new(48000);
        engine.set_fundamental(330.0);
        engine.trigger_strike(1.0);

        let transport = Transport::new(48000, 120.0);
        let ctx = ProcessContext::from_transport(&transport);

        let in_buf = [0.0f32; 64];
        let mut out_l = [0.0f32; 64];
        let mut out_r = [0.0f32; 64];

        {
            let _guard = AllocGuard::new();
            engine.process_block(
                &[&in_buf[..]],
                &mut [&mut out_l[..], &mut out_r[..]],
                &ctx,
            );
        }

        assert!(out_l.iter().any(|s| *s != 0.0));
        assert!(out_l.iter().all(|s| s.is_finite()));
    }
}
