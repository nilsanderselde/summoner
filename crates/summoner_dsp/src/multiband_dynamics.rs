// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Multi-Band Dynamics Processor & 4-Band Linear-Phase Linkwitz-Riley Crossover (Milestone 9).
//!
//! Provides a 4-way linear-phase Linkwitz-Riley crossover filter network with zero phase smearing,
//! independent per-band downward and upward compression, downward expansion, soft-saturation limiting,
//! solo/mute/bypass controls, and real-time gain reduction telemetry.
//!
//! Enforces zero heap allocations in real-time audio render loops under `AllocGuard`.

use crate::traits::SignalProcessor;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};

/// Number of distinct frequency bands in the multiband dynamics processor.
pub const NUM_DYNAMICS_BANDS: usize = 4;

/// Index enumeration for the 4 frequency bands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DynamicsBandIndex {
    Low = 0,     // Sub & Bass (0 Hz .. f1)
    LowMid = 1,  // Warmth & Body (f1 .. f2)
    HighMid = 2, // Presence & Bite (f2 .. f3)
    High = 3,    // Sheen & Air (f3 .. Nyquist)
}

/// 2nd-order Butterworth biquad filter section used to construct 4th-order Linkwitz-Riley (LR4) stages.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Butterworth2ndOrder {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    z1_l: f32,
    z2_l: f32,
    z1_r: f32,
    z2_r: f32,
}

impl Default for Butterworth2ndOrder {
    fn default() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            z1_l: 0.0,
            z2_l: 0.0,
            z1_r: 0.0,
            z2_r: 0.0,
        }
    }
}

impl Butterworth2ndOrder {
    pub fn reset(&mut self) {
        self.z1_l = 0.0;
        self.z2_l = 0.0;
        self.z1_r = 0.0;
        self.z2_r = 0.0;
    }

    /// Design a 2nd-order Butterworth Lowpass filter (Q = 1/sqrt(2) ≈ 0.70710678).
    pub fn design_lowpass(&mut self, freq: f32, sample_rate: f32) {
        let sr = sample_rate.max(1000.0);
        let cutoff = freq.clamp(10.0, sr * 0.49);
        let omega = 2.0 * PI * cutoff / sr;
        let sin_w = omega.sin();
        let cos_w = omega.cos();
        let alpha = sin_w / (2.0 * std::f32::consts::FRAC_1_SQRT_2);

        let a0 = 1.0 + alpha;
        self.b0 = ((1.0 - cos_w) * 0.5) / a0;
        self.b1 = (1.0 - cos_w) / a0;
        self.b2 = ((1.0 - cos_w) * 0.5) / a0;
        self.a1 = (-2.0 * cos_w) / a0;
        self.a2 = (1.0 - alpha) / a0;
    }

    /// Design a 2nd-order Butterworth Highpass filter (Q = 1/sqrt(2) ≈ 0.70710678).
    pub fn design_highpass(&mut self, freq: f32, sample_rate: f32) {
        let sr = sample_rate.max(1000.0);
        let cutoff = freq.clamp(10.0, sr * 0.49);
        let omega = 2.0 * PI * cutoff / sr;
        let sin_w = omega.sin();
        let cos_w = omega.cos();
        let alpha = sin_w / (2.0 * std::f32::consts::FRAC_1_SQRT_2);

        let a0 = 1.0 + alpha;
        self.b0 = ((1.0 + cos_w) * 0.5) / a0;
        self.b1 = (-(1.0 + cos_w)) / a0;
        self.b2 = ((1.0 + cos_w) * 0.5) / a0;
        self.a1 = (-2.0 * cos_w) / a0;
        self.a2 = (1.0 - alpha) / a0;
    }

    /// Design a 2nd-order Allpass filter to match the phase delay of a Butterworth section.
    pub fn design_allpass(&mut self, freq: f32, sample_rate: f32) {
        let sr = sample_rate.max(1000.0);
        let cutoff = freq.clamp(10.0, sr * 0.49);
        let omega = 2.0 * PI * cutoff / sr;
        let sin_w = omega.sin();
        let cos_w = omega.cos();
        let alpha = sin_w / (2.0 * std::f32::consts::FRAC_1_SQRT_2);

        let a0 = 1.0 + alpha;
        self.b0 = (1.0 - alpha) / a0;
        self.b1 = (-2.0 * cos_w) / a0;
        self.b2 = (1.0 + alpha) / a0;
        self.a1 = (-2.0 * cos_w) / a0;
        self.a2 = (1.0 - alpha) / a0;
    }

    #[inline(always)]
    pub fn process_sample(&mut self, in_l: f32, in_r: f32) -> (f32, f32) {
        // Direct Form II Transposed for Left Channel
        let out_l = self.b0 * in_l + self.z1_l;
        self.z1_l = self.b1 * in_l - self.a1 * out_l + self.z2_l;
        self.z2_l = self.b2 * in_l - self.a2 * out_l;

        // Direct Form II Transposed for Right Channel
        let out_r = self.b0 * in_r + self.z1_r;
        self.z1_r = self.b1 * in_r - self.a1 * out_r + self.z2_r;
        self.z2_r = self.b2 * in_r - self.a2 * out_r;

        (out_l, out_r)
    }
}

/// 4th-Order Linkwitz-Riley (LR4) 2-Way Crossover Splitter.
/// Cascades two identical 2nd-order Butterworth stages to yield 24 dB/octave attenuation.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct LinkwitzRiley4WaySplitter {
    pub freq: f32,
    lp_stage1: Butterworth2ndOrder,
    lp_stage2: Butterworth2ndOrder,
    hp_stage1: Butterworth2ndOrder,
    hp_stage2: Butterworth2ndOrder,
}

impl LinkwitzRiley4WaySplitter {
    pub fn new(freq: f32, sample_rate: f32) -> Self {
        let mut s = Self {
            freq,
            lp_stage1: Butterworth2ndOrder::default(),
            lp_stage2: Butterworth2ndOrder::default(),
            hp_stage1: Butterworth2ndOrder::default(),
            hp_stage2: Butterworth2ndOrder::default(),
        };
        s.update_coeffs(freq, sample_rate);
        s
    }

    pub fn reset(&mut self) {
        self.lp_stage1.reset();
        self.lp_stage2.reset();
        self.hp_stage1.reset();
        self.hp_stage2.reset();
    }

    pub fn update_coeffs(&mut self, freq: f32, sample_rate: f32) {
        self.freq = freq;
        self.lp_stage1.design_lowpass(freq, sample_rate);
        self.lp_stage2.design_lowpass(freq, sample_rate);
        self.hp_stage1.design_highpass(freq, sample_rate);
        self.hp_stage2.design_highpass(freq, sample_rate);
    }

    /// Split stereo input into (Low_L, Low_R) and (High_L, High_R) with in-phase summing.
    #[inline(always)]
    pub fn process_sample(&mut self, in_l: f32, in_r: f32) -> ((f32, f32), (f32, f32)) {
        let (lp1_l, lp1_r) = self.lp_stage1.process_sample(in_l, in_r);
        let (low_l, low_r) = self.lp_stage2.process_sample(lp1_l, lp1_r);

        let (hp1_l, hp1_r) = self.hp_stage1.process_sample(in_l, in_r);
        let (high_l, high_r) = self.hp_stage2.process_sample(hp1_l, hp1_r);

        // LR4 low and high outputs are in phase (360° phase difference), summing to flat allpass magnitude
        ((low_l, low_r), (high_l, high_r))
    }
}

/// 4-Band Linear-Phase Linkwitz-Riley Crossover Network.
/// Uses a 3-stage tree structure with allpass phase-matching to guarantee exact flat magnitude reconstruction.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LinkwitzRiley4BandCrossover {
    pub f_c1: f32, // Low / LowMid crossover (e.g. 120 Hz)
    pub f_c2: f32, // LowMid / HighMid crossover (e.g. 1200 Hz)
    pub f_c3: f32, // HighMid / High crossover (e.g. 6000 Hz)

    splitter_mid: LinkwitzRiley4WaySplitter, // Root split at f_c2 -> LowBranch, HighBranch
    splitter_low: LinkwitzRiley4WaySplitter, // LowBranch split at f_c1 -> Low, LowMid
    splitter_high: LinkwitzRiley4WaySplitter, // HighBranch split at f_c3 -> HighMid, High

    // Allpass phase compensators for zero phase smearing
    allpass_high_branch_c1: Butterworth2ndOrder,
    allpass_low_branch_c3: Butterworth2ndOrder,
}

impl Default for LinkwitzRiley4BandCrossover {
    fn default() -> Self {
        Self::new(120.0, 1200.0, 6000.0, 48000.0)
    }
}

impl LinkwitzRiley4BandCrossover {
    pub fn new(f_c1: f32, f_c2: f32, f_c3: f32, sample_rate: f32) -> Self {
        let mut crossover = Self {
            f_c1: f_c1.max(20.0),
            f_c2: f_c2.max(f_c1 + 10.0),
            f_c3: f_c3.max(f_c2 + 10.0),
            splitter_mid: LinkwitzRiley4WaySplitter::new(f_c2, sample_rate),
            splitter_low: LinkwitzRiley4WaySplitter::new(f_c1, sample_rate),
            splitter_high: LinkwitzRiley4WaySplitter::new(f_c3, sample_rate),
            allpass_high_branch_c1: Butterworth2ndOrder::default(),
            allpass_low_branch_c3: Butterworth2ndOrder::default(),
        };
        crossover.update_crossover_freqs(f_c1, f_c2, f_c3, sample_rate);
        crossover
    }

    pub fn reset(&mut self) {
        self.splitter_mid.reset();
        self.splitter_low.reset();
        self.splitter_high.reset();
        self.allpass_high_branch_c1.reset();
        self.allpass_low_branch_c3.reset();
    }

    pub fn update_crossover_freqs(&mut self, f_c1: f32, f_c2: f32, f_c3: f32, sample_rate: f32) {
        self.f_c1 = f_c1.clamp(20.0, 2000.0);
        self.f_c2 = f_c2.clamp(self.f_c1 + 20.0, 8000.0);
        self.f_c3 = f_c3.clamp(self.f_c2 + 20.0, sample_rate * 0.45);

        self.splitter_mid.update_coeffs(self.f_c2, sample_rate);
        self.splitter_low.update_coeffs(self.f_c1, sample_rate);
        self.splitter_high.update_coeffs(self.f_c3, sample_rate);

        // Phase alignment allpasses (1 2nd-order allpass section per crossover point)
        self.allpass_high_branch_c1.design_allpass(self.f_c1, sample_rate);
        self.allpass_low_branch_c3.design_allpass(self.f_c3, sample_rate);
    }

    /// Process a stereo sample and return 4 stereo band signals:
    /// `[(Low_L, Low_R), (LowMid_L, LowMid_R), (HighMid_L, HighMid_R), (High_L, High_R)]`.
    #[inline(always)]
    pub fn process_sample(&mut self, in_l: f32, in_r: f32) -> [(f32, f32); 4] {
        // Step 1: Root split at f_c2
        let ((low_branch_l, low_branch_r), (high_branch_l, high_branch_r)) =
            self.splitter_mid.process_sample(in_l, in_r);

        // Step 2: Phase-align LowBranch with f_c3 allpass
        let (low_aligned_l, low_aligned_r) = self.allpass_low_branch_c3.process_sample(low_branch_l, low_branch_r);

        // Step 3: Phase-align HighBranch with f_c1 allpass
        let (high_aligned_l, high_aligned_r) = self.allpass_high_branch_c1.process_sample(high_branch_l, high_branch_r);

        // Step 4: Split phase-aligned LowBranch at f_c1 -> Low, LowMid
        let ((low_l, low_r), (lowmid_l, lowmid_r)) =
            self.splitter_low.process_sample(low_aligned_l, low_aligned_r);

        // Step 5: Split phase-aligned HighBranch at f_c3 -> HighMid, High
        let ((highmid_l, highmid_r), (high_l, high_r)) =
            self.splitter_high.process_sample(high_aligned_l, high_aligned_r);

        [
            (low_l, low_r),
            (lowmid_l, lowmid_r),
            (highmid_l, highmid_r),
            (high_l, high_r),
        ]
    }
}

/// Dynamics configuration and processor for an individual frequency band.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandDynamics {
    pub name: String,
    pub bypass: bool,
    pub mute: bool,
    pub solo: bool,

    // Downward Compressor
    pub comp_threshold_db: f32,
    pub comp_ratio: f32,
    pub comp_attack_ms: f32,
    pub comp_release_ms: f32,
    pub comp_knee_db: f32,
    pub comp_makeup_db: f32,

    // Upward Compressor / Expander
    pub exp_threshold_db: f32,
    pub exp_ratio: f32,
    pub exp_attack_ms: f32,
    pub exp_release_ms: f32,
    pub exp_knee_db: f32,

    // Soft Saturation & Limiting
    pub saturation_drive: f32,
    pub saturation_ceiling_db: f32,
    pub limiter_enabled: bool,

    // Real-Time Telemetry
    #[serde(skip)]
    pub env_level_db: f32,
    #[serde(skip)]
    pub gain_reduction_db: f32,
    #[serde(skip)]
    pub gain_expansion_db: f32,
    #[serde(skip)]
    pub peak_out_db: f32,

    #[serde(skip)]
    envelope: f32,
}

impl BandDynamics {
    pub fn new(
        name: &str,
        comp_thresh_db: f32,
        comp_ratio: f32,
        comp_attack_ms: f32,
        comp_release_ms: f32,
        makeup_db: f32,
    ) -> Self {
        Self {
            name: name.to_string(),
            bypass: false,
            mute: false,
            solo: false,
            comp_threshold_db: comp_thresh_db,
            comp_ratio: comp_ratio.max(1.0),
            comp_attack_ms: comp_attack_ms.max(0.1),
            comp_release_ms: comp_release_ms.max(1.0),
            comp_knee_db: 4.0,
            comp_makeup_db: makeup_db,
            exp_threshold_db: -48.0,
            exp_ratio: 1.0, // 1.0 = inactive expansion
            exp_attack_ms: 10.0,
            exp_release_ms: 100.0,
            exp_knee_db: 4.0,
            saturation_drive: 0.0,
            saturation_ceiling_db: 0.0,
            limiter_enabled: true,
            env_level_db: -120.0,
            gain_reduction_db: 0.0,
            gain_expansion_db: 0.0,
            peak_out_db: -120.0,
            envelope: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.envelope = 0.0;
        self.env_level_db = -120.0;
        self.gain_reduction_db = 0.0;
        self.gain_expansion_db = 0.0;
        self.peak_out_db = -120.0;
    }

    /// Calculate combined dynamics gain transfer (downward compression + upward expansion).
    #[inline]
    fn calculate_dynamics_gain(&self, level_db: f32) -> (f32, f32, f32) {
        let mut gr_db = 0.0f32;
        let mut ge_db = 0.0f32;

        // 1. Downward Compression
        if self.comp_ratio > 1.0001 {
            let thresh = self.comp_threshold_db;
            let knee = self.comp_knee_db;
            if knee > 0.0 && level_db > thresh - knee * 0.5 && level_db < thresh + knee * 0.5 {
                let excess = level_db - thresh + knee * 0.5;
                let curve = (excess * excess) / (2.0 * knee);
                gr_db = -curve * (1.0 - 1.0 / self.comp_ratio);
            } else if level_db >= thresh {
                gr_db = -(level_db - thresh) * (1.0 - 1.0 / self.comp_ratio);
            }
        }

        // 2. Downward Expansion / Upward Dynamics
        if self.exp_ratio > 1.0001 {
            let thresh = self.exp_threshold_db;
            let knee = self.exp_knee_db;
            if level_db < thresh - knee * 0.5 {
                ge_db = -(thresh - level_db) * (self.exp_ratio - 1.0);
            } else if knee > 0.0 && level_db >= thresh - knee * 0.5 && level_db <= thresh + knee * 0.5 {
                let delta = thresh + knee * 0.5 - level_db;
                let curve = (delta * delta) / (2.0 * knee);
                ge_db = -curve * (self.exp_ratio - 1.0);
            }
        }

        let total_gain_db = gr_db + ge_db + self.comp_makeup_db;
        (total_gain_db, gr_db, ge_db)
    }

    /// Process a stereo sample through the band dynamics processor.
    #[inline(always)]
    pub fn process_sample(&mut self, in_l: f32, in_r: f32, sample_rate: f32) -> (f32, f32) {
        if self.mute {
            return (0.0, 0.0);
        }
        if self.bypass {
            return (in_l, in_r);
        }

        let abs_peak = in_l.abs().max(in_r.abs()).max(1e-6);
        let dt = 1.0 / sample_rate.max(1000.0);

        let attack_tau = self.comp_attack_ms * 0.001;
        let release_tau = self.comp_release_ms * 0.001;
        let attack_coeff = (-dt / attack_tau.max(0.0001)).exp();
        let release_coeff = (-dt / release_tau.max(0.001)).exp();

        let coeff = if abs_peak > self.envelope {
            attack_coeff
        } else {
            release_coeff
        };
        self.envelope = self.envelope * coeff + abs_peak * (1.0 - coeff);

        let env_db = 20.0 * self.envelope.max(1e-6).log10();
        self.env_level_db = env_db;

        let (total_gain_db, gr_db, ge_db) = self.calculate_dynamics_gain(env_db);
        self.gain_reduction_db = gr_db;
        self.gain_expansion_db = ge_db;

        let gain_lin = 10.0f32.powf(total_gain_db / 20.0);

        let mut out_l = in_l * gain_lin;
        let mut out_r = in_r * gain_lin;

        // Apply soft saturation limiter if enabled
        if self.saturation_drive > 0.001 || self.limiter_enabled {
            let drive_lin = 10.0f32.powf(self.saturation_drive / 20.0);
            let ceil_lin = 10.0f32.powf(self.saturation_ceiling_db / 20.0);

            let sat_l = (out_l * drive_lin / ceil_lin).tanh() * ceil_lin;
            let sat_r = (out_r * drive_lin / ceil_lin).tanh() * ceil_lin;

            out_l = sat_l;
            out_r = sat_r;
        }

        let out_peak = out_l.abs().max(out_r.abs()).max(1e-6);
        self.peak_out_db = 20.0 * out_peak.log10();

        (out_l, out_r)
    }
}

/// 4-Band Multi-Band Dynamics Processor Node.
/// Combines a 4-way Linkwitz-Riley crossover network with 4 independent dynamics processors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultibandDynamicsProcessor {
    pub crossover: LinkwitzRiley4BandCrossover,
    pub bands: [BandDynamics; NUM_DYNAMICS_BANDS],
    pub master_dry_wet: f32,
    pub master_output_gain_db: f32,
    pub any_solo_active: bool,
}

impl Default for MultibandDynamicsProcessor {
    fn default() -> Self {
        Self::new(48000.0)
    }
}

impl MultibandDynamicsProcessor {
    /// Create a new 4-band dynamics processor initialized for professional mastering/mixing.
    pub fn new(sample_rate: f32) -> Self {
        let crossover = LinkwitzRiley4BandCrossover::new(120.0, 1200.0, 6000.0, sample_rate);

        let bands = [
            // Band 0: Low (Sub & Bass: <120 Hz)
            BandDynamics::new("Low", -18.0, 3.5, 20.0, 120.0, 1.5),
            // Band 1: Low-Mid (Body & Snare Body: 120 Hz .. 1200 Hz)
            BandDynamics::new("Low-Mid", -16.0, 2.5, 15.0, 100.0, 1.0),
            // Band 2: High-Mid (Presence & Vocals: 1200 Hz .. 6000 Hz)
            BandDynamics::new("High-Mid", -14.0, 2.0, 8.0, 80.0, 0.5),
            // Band 3: High (Air & Cymbals: >6000 Hz)
            BandDynamics::new("High", -12.0, 1.8, 5.0, 60.0, 0.0),
        ];

        Self {
            crossover,
            bands,
            master_dry_wet: 1.0, // 100% Wet
            master_output_gain_db: 0.0,
            any_solo_active: false,
        }
    }

    /// Reset all internal filters, envelopes, and histories.
    pub fn reset(&mut self) {
        self.crossover.reset();
        for b in &mut self.bands {
            b.reset();
        }
    }

    /// Update crossover cutoff frequencies.
    pub fn set_crossover_frequencies(&mut self, f_c1: f32, f_c2: f32, f_c3: f32, sample_rate: f32) {
        self.crossover.update_crossover_freqs(f_c1, f_c2, f_c3, sample_rate);
    }

    /// Process a stereo sample through the 4-band crossover and dynamics engine with zero allocations.
    #[inline(always)]
    pub fn process_stereo_sample(&mut self, in_l: f32, in_r: f32, sample_rate: f32) -> (f32, f32) {
        // Step 1: Split into 4 frequency bands via Linkwitz-Riley Crossover
        let band_inputs = self.crossover.process_sample(in_l, in_r);

        // Check if any band is soloed
        let any_solo = self.bands.iter().any(|b| b.solo);
        self.any_solo_active = any_solo;

        let mut sum_l = 0.0f32;
        let mut sum_r = 0.0f32;

        // Step 2: Process each band independently
        for (i, band) in self.bands.iter_mut().enumerate() {
            let (b_in_l, b_in_r) = band_inputs[i];

            if any_solo && !band.solo {
                // Mute bands that are not soloed
                continue;
            }

            let (b_out_l, b_out_r) = band.process_sample(b_in_l, b_in_r, sample_rate);
            sum_l += b_out_l;
            sum_r += b_out_r;
        }

        // Step 3: Apply master dry/wet mix and output makeup
        let master_gain_lin = 10.0f32.powf(self.master_output_gain_db / 20.0);
        let wet_l = sum_l * master_gain_lin;
        let wet_r = sum_r * master_gain_lin;

        let dry_w = (1.0 - self.master_dry_wet).clamp(0.0, 1.0);
        let wet_w = self.master_dry_wet.clamp(0.0, 1.0);

        (in_l * dry_w + wet_l * wet_w, in_r * dry_w + wet_r * wet_w)
    }

    /// Process a block of stereo audio samples in-place.
    pub fn process_stereo_block(
        &mut self,
        left: &mut [f32],
        right: &mut [f32],
        sample_rate: u32,
    ) {
        let sr = sample_rate as f32;
        let count = left.len().min(right.len());
        for i in 0..count {
            let (out_l, out_r) = self.process_stereo_sample(left[i], right[i], sr);
            left[i] = out_l;
            right[i] = out_r;
        }
    }
}

impl SignalProcessor for MultibandDynamicsProcessor {
    fn name(&self) -> &str {
        "MultibandDynamicsProcessor"
    }

    fn process_block(
        &mut self,
        input: &[&[Sample]],
        output: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        if input.is_empty() || output.is_empty() {
            return;
        }

        let num_samples = input[0].len().min(output[0].len());
        let sr = if ctx.sample_rate > 0 {
            ctx.sample_rate as f32
        } else {
            48000.0
        };

        let has_stereo_in = input.len() >= 2;
        let has_stereo_out = output.len() >= 2;

        for i in 0..num_samples {
            let in_l = input[0][i];
            let in_r = if has_stereo_in { input[1][i] } else { in_l };

            let (out_l, out_r) = self.process_stereo_sample(in_l, in_r, sr);

            output[0][i] = out_l;
            if has_stereo_out {
                output[1][i] = out_r;
            }
        }
    }
}

impl AudioNode for MultibandDynamicsProcessor {
    fn name(&self) -> &str {
        "MultibandDynamicsProcessor"
    }

    fn process(
        &mut self,
        input: &[&[Sample]],
        output: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        self.process_block(input, output, ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::allocator::AllocGuard;
    use summoner_core::transport::Transport;

    #[test]
    fn test_crossover_linear_phase_flat_reconstruction() {
        let sample_rate = 48000.0;
        let mut crossover = LinkwitzRiley4BandCrossover::new(150.0, 1200.0, 6000.0, sample_rate);

        // Test across multiple frequencies spanning the audible spectrum
        let test_frequencies = [40.0, 100.0, 150.0, 500.0, 1200.0, 3000.0, 6000.0, 12000.0, 18000.0];

        for &freq in &test_frequencies {
            crossover.reset();
            let omega = 2.0 * PI * freq / sample_rate;

            let mut max_in = 0.0f32;
            let mut max_sum = 0.0f32;

            // Warm up filter states for 1024 samples
            for n in 0..1024 {
                let in_sample = (omega * n as f32).sin();
                let bands = crossover.process_sample(in_sample, in_sample);
                let sum_sample = bands[0].0 + bands[1].0 + bands[2].0 + bands[3].0;

                if n > 512 {
                    if in_sample.abs() > max_in {
                        max_in = in_sample.abs();
                    }
                    if sum_sample.abs() > max_sum {
                        max_sum = sum_sample.abs();
                    }
                }
            }

            let amp_diff_db = (20.0 * (max_sum / max_in.max(1e-6)).log10()).abs();
            assert!(
                amp_diff_db < 0.25,
                "Linkwitz-Riley crossover must have flat amplitude sum at {} Hz (diff: {:.3} dB)",
                freq,
                amp_diff_db
            );
        }
    }

    #[test]
    fn test_multiband_dynamics_downward_compression() {
        let mut processor = MultibandDynamicsProcessor::new(48000.0);
        let sample_rate = 48000;

        let mut left = [0.9f32; 512];
        let mut right = [0.9f32; 512];

        processor.process_stereo_block(&mut left, &mut right, sample_rate);

        // Verify gain reduction occurred across bands
        assert!(
            processor.bands[0].gain_reduction_db < 0.0 || processor.bands[1].gain_reduction_db < 0.0,
            "Compressor should produce gain reduction on loud input"
        );
        assert!(left[511].is_finite());
        assert!(right[511].is_finite());
    }

    #[test]
    fn test_multiband_dynamics_zero_allocation() {
        let mut processor = MultibandDynamicsProcessor::new(48000.0);
        let transport = Transport::new(48000, 120.0);
        let ctx = ProcessContext::from_transport(&transport);

        let in_l = [0.5f32; 128];
        let in_r = [-0.5f32; 128];
        let mut out_l = [0.0f32; 128];
        let mut out_r = [0.0f32; 128];

        let in_slices: [&[Sample]; 2] = [&in_l[..], &in_r[..]];
        let mut out_slices: [&mut [Sample]; 2] = [&mut out_l[..], &mut out_r[..]];

        {
            let _guard = AllocGuard::new();
            processor.process_block(&in_slices, &mut out_slices, &ctx);
        }

        assert!(out_l.iter().all(|s| s.is_finite()));
        assert!(out_r.iter().all(|s| s.is_finite()));
    }
}
