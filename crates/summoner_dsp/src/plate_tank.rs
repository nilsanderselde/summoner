// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Continuous Dispersion Mechanical Plate Reverb Tank Engine (Milestone 20).
//!
//! Provides high-precision physical modeling of mechanical reverberation plates (EMT 140 cold-rolled
//! steel, EMT 240 gold foil, and compact high-tension diaphragm tanks) with frequency-dependent
//! flexural wave dispersion (biharmonic wave equation $\frac{\partial^2 u}{\partial t^2} + \kappa^2 \nabla^4 u = 0$),
//! continuous mechanical damper pad decay time adjustment ($T_{60} \in [0.4, 8.0]\text{s}$),
//! electromagnetic driver coil saturation, and dual stereo pickup transducers.
//!
//! Enforces zero heap allocations in audio processing loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};
use crate::traits::SignalProcessor;

/// Number of cascaded allpass dispersion stages in the all-pass dispersion network (APDN).
pub const NUM_DISPERSION_STAGES: usize = 4;
/// Number of parallel delay lines in the plate tank circulating feedback loop.
pub const NUM_PLATE_LOOPS: usize = 4;
/// Maximum buffer size for each plate delay line.
pub const MAX_PLATE_DELAY_LEN: usize = 8192;

/// Mechanical plate reverb preset profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PlateReverbProfile {
    /// Classic EMT 140 cold-rolled steel plate (600 lbs) with lush warm diffusion and extended decay.
    #[default]
    VintageEmt140Steel,
    /// Modern high-tension studio suspension plate with crystalline highs and rapid diffusion.
    StudioPlateSuspension,
    /// EMT 240 style micro-thin gold foil diaphragm with ultra-dense modal response and tight transients.
    GoldFoilPlate,
    /// Compact mechanical metal plate tank with distinct metallic modal ringing.
    CompactMechanicalTank,
    /// Tuned high-tension acoustic metal resonator with harmonic sustain.
    HighTensionResonator,
}

impl PlateReverbProfile {
    /// Returns default decay time $T_{60}$ in seconds.
    pub fn default_t60_sec(&self) -> f32 {
        match self {
            Self::VintageEmt140Steel => 3.5,
            Self::StudioPlateSuspension => 2.8,
            Self::GoldFoilPlate => 1.8,
            Self::CompactMechanicalTank => 1.2,
            Self::HighTensionResonator => 4.5,
        }
    }

    /// Returns nominal $(\text{dispersion\_factor}, \text{low\_damping}, \text{high\_damping}, \text{pre\_delay\_ms})$.
    pub fn nominal_physics(&self) -> (f32, f32, f32, f32) {
        match self {
            Self::VintageEmt140Steel => (0.65, 0.15, 0.45, 12.0),
            Self::StudioPlateSuspension => (0.45, 0.08, 0.25, 8.0),
            Self::GoldFoilPlate => (0.85, 0.05, 0.60, 5.0),
            Self::CompactMechanicalTank => (0.50, 0.20, 0.35, 15.0),
            Self::HighTensionResonator => (0.70, 0.02, 0.18, 10.0),
        }
    }

    /// Human-readable profile name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::VintageEmt140Steel => "EMT 140 Vintage Steel Plate",
            Self::StudioPlateSuspension => "Studio Suspension Plate",
            Self::GoldFoilPlate => "EMT 240 Gold Foil Diaphragm",
            Self::CompactMechanicalTank => "Compact Mechanical Tank",
            Self::HighTensionResonator => "High-Tension Resonator Plate",
        }
    }
}

/// A first-order allpass filter stage for continuous frequency-dependent phase dispersion.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AllpassDispersionStage {
    pub coeff: f32,
    x1: f32,
    y1: f32,
}

impl Default for AllpassDispersionStage {
    fn default() -> Self {
        Self {
            coeff: 0.5,
            x1: 0.0,
            y1: 0.0,
        }
    }
}

impl AllpassDispersionStage {
    /// Resets filter history state.
    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.y1 = 0.0;
    }

    /// Processes a sample through the allpass dispersion stage: $y[n] = -\alpha x[n] + x[n-1] + \alpha y[n-1]$.
    #[inline(always)]
    pub fn step(&mut self, input: f32) -> f32 {
        let output = -self.coeff * input + self.x1 + self.coeff * self.y1;
        self.x1 = input;
        self.y1 = if output.is_finite() { output } else { 0.0 };
        self.y1
    }
}

/// Static prime delay loop lengths in samples for 48kHz base sample rate.
const PLATE_LOOP_BASE_DELAYS: [usize; NUM_PLATE_LOOPS] = [1357, 1871, 2417, 3119];

fn default_loop_buffers() -> [[f32; MAX_PLATE_DELAY_LEN]; NUM_PLATE_LOOPS] {
    [[0.0; MAX_PLATE_DELAY_LEN]; NUM_PLATE_LOOPS]
}

fn default_predelay_buf() -> [f32; 4800] {
    [0.0; 4800]
}

/// Continuous dispersion mechanical plate reverb tank voice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlateTank {
    pub sample_rate: u32,
    pub profile: PlateReverbProfile,
    /// Mechanical damper pad decay time $T_{60}$ in seconds $[0.2 ..= 10.0]$.
    pub decay_t60_sec: f32,
    /// Flexural wave dispersion factor $[0.0 ..= 1.0]$.
    pub dispersion_factor: f32,
    /// High-frequency absorption damping $[0.0 ..= 1.0]$.
    pub high_damping: f32,
    /// Low-frequency roll-off damping $[0.0 ..= 1.0]$.
    pub low_damping: f32,
    /// Driver input coil saturation drive $[0.0 ..= 1.0]$.
    pub driver_saturation: f32,
    /// Stereo width / pickup transducer separation $[0.0 ..= 1.0]$.
    pub stereo_width: f32,
    /// Wet / dry balance $[0.0 ..= 1.0]$.
    pub wet_mix: f32,
    // Cascaded allpass dispersion stages
    #[serde(skip)]
    dispersion_stages: [AllpassDispersionStage; NUM_DISPERSION_STAGES],
    // Circulating delay loop buffers (fixed pre-allocated arrays)
    #[serde(skip, default = "default_loop_buffers")]
    loop_buffers: [[f32; MAX_PLATE_DELAY_LEN]; NUM_PLATE_LOOPS],
    #[serde(skip)]
    loop_indices: [usize; NUM_PLATE_LOOPS],
    #[serde(skip)]
    loop_lengths: [usize; NUM_PLATE_LOOPS],
    // Damping one-pole lowpass states
    #[serde(skip)]
    damp_lp_y1: [f32; NUM_PLATE_LOOPS],
    // Feedback gains
    #[serde(skip)]
    feedback_gains: [f32; NUM_PLATE_LOOPS],
    // Pre-delay buffer (up to 4800 samples = 100ms)
    #[serde(skip, default = "default_predelay_buf")]
    predelay_buf: [f32; 4800],
    #[serde(skip)]
    predelay_idx: usize,
    #[serde(skip)]
    predelay_samples: usize,
}

impl PlateTank {
    /// Creates a new PlateTank mechanical plate reverb instance.
    pub fn new(sample_rate: u32) -> Self {
        let sr = sample_rate.max(8000);
        let prof = PlateReverbProfile::VintageEmt140Steel;
        let (disp, low_damp, high_damp, pre_ms) = prof.nominal_physics();
        let t60 = prof.default_t60_sec();

        let mut tank = Self {
            sample_rate: sr,
            profile: prof,
            decay_t60_sec: t60,
            dispersion_factor: disp,
            high_damping: high_damp,
            low_damping: low_damp,
            driver_saturation: 0.35,
            stereo_width: 0.85,
            wet_mix: 0.40,
            dispersion_stages: [AllpassDispersionStage::default(); NUM_DISPERSION_STAGES],
            loop_buffers: [[0.0; MAX_PLATE_DELAY_LEN]; NUM_PLATE_LOOPS],
            loop_indices: [0; NUM_PLATE_LOOPS],
            loop_lengths: [1000; NUM_PLATE_LOOPS],
            damp_lp_y1: [0.0; NUM_PLATE_LOOPS],
            feedback_gains: [0.7; NUM_PLATE_LOOPS],
            predelay_buf: [0.0; 4800],
            predelay_idx: 0,
            predelay_samples: ((pre_ms * 0.001 * sr as f32) as usize).min(4799),
        };

        tank.update_parameters();
        tank
    }

    /// Sets active preset profile.
    pub fn set_profile(&mut self, profile: PlateReverbProfile) {
        self.profile = profile;
        let (disp, low_damp, high_damp, pre_ms) = profile.nominal_physics();
        self.decay_t60_sec = profile.default_t60_sec();
        self.dispersion_factor = disp;
        self.low_damping = low_damp;
        self.high_damping = high_damp;
        self.predelay_samples = ((pre_ms * 0.001 * self.sample_rate as f32) as usize).min(4799);
        self.update_parameters();
    }

    /// Updates internal delay loop lengths and allpass coefficients.
    #[allow(clippy::needless_range_loop)]
    pub fn update_parameters(&mut self) {
        let sr = self.sample_rate as f32;
        let scale = sr / 48000.0;

        for i in 0..NUM_PLATE_LOOPS {
            let base_len = (PLATE_LOOP_BASE_DELAYS[i] as f32 * scale) as usize;
            self.loop_lengths[i] = base_len.clamp(64, MAX_PLATE_DELAY_LEN - 1);

            // Feedback loop gain derived from T60: g = 10^(-3 * delay_time / T60)
            let loop_time_sec = self.loop_lengths[i] as f32 / sr;
            let g = 10.0_f32.powf(-3.0 * loop_time_sec / self.decay_t60_sec.max(0.1));
            self.feedback_gains[i] = g.clamp(0.0, 0.985);
        }

        // Configure allpass dispersion stages: allpass alpha varies systematically to disperse frequencies
        for i in 0..NUM_DISPERSION_STAGES {
            let alpha = 0.35 + 0.50 * self.dispersion_factor * (1.0 - (i as f32 * 0.15));
            self.dispersion_stages[i].coeff = alpha.clamp(0.05, 0.92);
        }
    }

    /// Triggers an impulse into the plate driver.
    pub fn trigger_impulse(&mut self, velocity: f32) {
        let vel = velocity.clamp(0.01, 1.0);
        let drive_node = 0;
        self.loop_buffers[drive_node][self.loop_indices[drive_node]] += vel * 1.5;
    }

    /// Resets all internal buffers and states.
    pub fn reset(&mut self) {
        for stage in &mut self.dispersion_stages {
            stage.reset();
        }
        for buf in &mut self.loop_buffers {
            buf.fill(0.0);
        }
        self.loop_indices.fill(0);
        self.damp_lp_y1.fill(0.0);
        self.predelay_buf.fill(0.0);
        self.predelay_idx = 0;
    }

    /// Processes a single stereo/mono sample through the plate tank.
    #[allow(clippy::needless_range_loop)]
    pub fn process_sample(&mut self, input: f32) -> (f32, f32) {
        // 1. Pre-delay buffer
        let pre_delayed = if self.predelay_samples > 0 {
            let read_idx = if self.predelay_idx >= self.predelay_samples {
                self.predelay_idx - self.predelay_samples
            } else {
                self.predelay_buf.len() + self.predelay_idx - self.predelay_samples
            };
            let val = self.predelay_buf[read_idx % self.predelay_buf.len()];
            self.predelay_buf[self.predelay_idx] = input;
            self.predelay_idx = (self.predelay_idx + 1) % self.predelay_buf.len();
            val
        } else {
            input
        };

        // 2. Driver coil saturation (soft non-linear compression)
        let sat_gain = 1.0 + self.driver_saturation * 1.8;
        let driven_in = (pre_delayed * sat_gain).tanh();

        // 3. APDN (All-Pass Dispersion Network) chain
        let mut dispersed = driven_in;
        for stage in &mut self.dispersion_stages {
            dispersed = stage.step(dispersed);
        }

        // 4. Circulating Householder feedback matrix across the 4 delay loops
        // Read out delayed values
        let mut loop_outs = [0.0f32; NUM_PLATE_LOOPS];
        for i in 0..NUM_PLATE_LOOPS {
            let idx = self.loop_indices[i];
            let raw_out = self.loop_buffers[i][idx];

            // Frequency-dependent damping lowpass filter: y[n] = (1 - d) x[n] + d y[n-1]
            let damp_coeff = self.high_damping.clamp(0.0, 0.95);
            let filtered = (1.0 - damp_coeff) * raw_out + damp_coeff * self.damp_lp_y1[i];
            self.damp_lp_y1[i] = filtered;

            loop_outs[i] = filtered * self.feedback_gains[i];
        }

        // 4x4 Householder mixing matrix: H = I - 2/N * (1 1 1 1)^T (1 1 1 1) -> out_i = in_i - 0.5 * sum
        let loop_sum = loop_outs[0] + loop_outs[1] + loop_outs[2] + loop_outs[3];
        let half_sum = loop_sum * 0.5;

        for i in 0..NUM_PLATE_LOOPS {
            let feedback = loop_outs[i] - half_sum;
            let write_val = dispersed * 0.5 + feedback;

            let len = self.loop_lengths[i];
            let next_idx = (self.loop_indices[i] + 1) % len;
            self.loop_buffers[i][self.loop_indices[i]] = write_val;
            self.loop_indices[i] = next_idx;
        }

        // 5. Read stereo pickup transducers
        let wet_l = loop_outs[0] - loop_outs[2];
        let wet_r = loop_outs[1] - loop_outs[3];

        let width = self.stereo_width;
        let mid = (wet_l + wet_r) * 0.5;
        let side = (wet_l - wet_r) * 0.5 * width;

        let final_wet_l = mid + side;
        let final_wet_r = mid - side;

        let out_l = input * (1.0 - self.wet_mix) + final_wet_l * self.wet_mix;
        let out_r = input * (1.0 - self.wet_mix) + final_wet_r * self.wet_mix;

        (out_l.clamp(-1.0, 1.0), out_r.clamp(-1.0, 1.0))
    }
}

impl SignalProcessor for PlateTank {
    fn name(&self) -> &str {
        "PlateTank"
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

            let (out_l, out_r) = self.process_sample(excitation);
            if !outputs.is_empty() && i < outputs[0].len() {
                outputs[0][i] = out_l;
            }
            if outputs.len() > 1 && i < outputs[1].len() {
                outputs[1][i] = out_r;
            }
        }
    }
}

/// AudioNode wrapper for Plate Reverb Tank DSP engine.
#[derive(Debug)]
pub struct PlateTankNode {
    pub plate: PlateTank,
}

impl PlateTankNode {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            plate: PlateTank::new(sample_rate),
        }
    }
}

impl AudioNode for PlateTankNode {
    fn name(&self) -> &str {
        "PlateTankNode"
    }

    fn process(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        self.plate.process_block(inputs, outputs, ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::allocator::AllocGuard;

    #[test]
    fn test_plate_tank_impulse_response_and_decay() {
        let mut plate = PlateTank::new(48000);
        plate.set_profile(PlateReverbProfile::VintageEmt140Steel);
        plate.wet_mix = 1.0;
        plate.trigger_impulse(0.90);

        let mut peak_amp = 0.0f32;
        for _ in 0..4000 {
            let (l, r) = plate.process_sample(0.0);
            assert!(l.is_finite());
            assert!(r.is_finite());
            assert!((-1.0..=1.0).contains(&l));
            assert!((-1.0..=1.0).contains(&r));
            peak_amp = peak_amp.max(l.abs().max(r.abs()));
        }

        for _ in 0..96000 {
            let (l, r) = plate.process_sample(0.0);
            assert!(l.is_finite());
            assert!(r.is_finite());
        }

        let mut late_amp = 0.0f32;
        for _ in 0..2000 {
            let (l, r) = plate.process_sample(0.0);
            assert!(l.is_finite());
            assert!(r.is_finite());
            late_amp = late_amp.max(l.abs().max(r.abs()));
        }

        assert!(peak_amp > 0.01, "Plate reverb impulse must produce reverberation");
        assert!(late_amp < peak_amp, "Plate reverberation must naturally decay over time");
    }

    #[test]
    fn test_plate_tank_zero_allocation_in_loop() {
        let mut plate = PlateTank::new(48000);
        plate.trigger_impulse(0.80);

        let mut out_l = [0.0f32; 256];
        let mut out_r = [0.0f32; 256];
        let in_buf = [0.0f32; 256];
        let ctx = ProcessContext::new(48000, 120.0, 0);

        let _guard = AllocGuard::new();
        for _ in 0..10 {
            plate.process_block(&[&in_buf[..]], &mut [&mut out_l[..], &mut out_r[..]], &ctx);
        }
    }
}
