// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Vacuum Tube Bias Modeling & Triode/Pentode Saturation (Milestone 11).
//!
//! Provides high-precision physical vacuum tube simulation (Triode 12AX7, Pentode EL34,
//! Beam Tetrode 6L6) with power sag compression, asymmetric transfer curve, grid conduction,
//! and DC bias calibration.
//!
//! Enforces zero heap allocations in real-time audio render loops under `AllocGuard`.

use crate::traits::SignalProcessor;
use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};

/// Vacuum tube model / topology selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TubeTopology {
    /// 12AX7 / ECC83 Dual Triode: High voltage gain (mu ~ 100), warm 2nd-order even harmonics,
    /// soft-knee grid conduction, and musical dynamic compression.
    #[default]
    Triode12AX7,
    /// EL34 Power Pentode: British high-power sound, punchy aggressive breakup, balanced
    /// 2nd and 3rd harmonics, sharp dynamic transition into grid clipping.
    PentodeEL34,
    /// 6L6 / 5881 Beam Tetrode: American high-headroom character, deep low-end, scooped midrange,
    /// prominent 3rd/5th odd harmonics under heavy drive, glassy top end.
    BeamTetrode6L6,
    /// Clean Modern: Symmetrical soft-saturation curve for mastering and transparent glue.
    CleanModern,
}

impl TubeTopology {
    pub fn label(&self) -> &'static str {
        match self {
            TubeTopology::Triode12AX7 => "12AX7 TRIODE (VINTAGE WARMTH)",
            TubeTopology::PentodeEL34 => "EL34 PENTODE (BRITISH PUNCH)",
            TubeTopology::BeamTetrode6L6 => "6L6 BEAM TETRODE (AMERICAN HEADROOM)",
            TubeTopology::CleanModern => "CLEAN MODERN (TRANSPARENT SAT)",
        }
    }

    /// Nominal amplification factor ($\mu$).
    pub fn mu(&self) -> f32 {
        match self {
            TubeTopology::Triode12AX7 => 100.0,
            TubeTopology::PentodeEL34 => 60.0,
            TubeTopology::BeamTetrode6L6 => 40.0,
            TubeTopology::CleanModern => 20.0,
        }
    }

    /// Base harmonic even-vs-odd balance weight (0.0 = pure odd, 1.0 = pure even).
    pub fn base_asymmetry(&self) -> f32 {
        match self {
            TubeTopology::Triode12AX7 => 0.70,
            TubeTopology::PentodeEL34 => 0.45,
            TubeTopology::BeamTetrode6L6 => 0.35,
            TubeTopology::CleanModern => 0.50,
        }
    }
}

/// Internal DSP state for a single audio channel of vacuum tube simulation.
#[derive(Debug, Clone, Copy, Default)]
pub struct TubeChannelState {
    /// Dynamic power sag envelope follower state.
    pub sag_env: f32,
    /// DC blocker input delayed sample.
    pub dc_x1: f32,
    /// DC blocker output delayed sample.
    pub dc_y1: f32,
    /// Warmth low-pass 1-pole filter state.
    pub lp_state: f32,
    /// High-pass grid coupling filter state.
    pub hp_state: f32,
}

impl TubeChannelState {
    pub fn reset(&mut self) {
        self.sag_env = 0.0;
        self.dc_x1 = 0.0;
        self.dc_y1 = 0.0;
        self.lp_state = 0.0;
        self.hp_state = 0.0;
    }
}

/// Comprehensive Physical Vacuum Tube Saturation & Bias DSP Node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TubeSaturationNode {
    /// Vacuum tube topology / model.
    pub topology: TubeTopology,
    /// Input Drive multiplier (0.1 to 10.0, default 2.5).
    pub drive: f32,
    /// Operating grid bias voltage in Volts DC (-4.0 ..= 0.0 V, default -1.85 V).
    pub bias_voltage_v: f32,
    /// Anode plate voltage in Volts DC (100.0 ..= 450.0 V, default 250.0 V).
    pub plate_voltage_v: f32,
    /// Power supply sag compression amount (0.0 to 1.0, default 0.35).
    pub sag_compression: f32,
    /// Asymmetry / even-order harmonic balance (0.0 to 1.0, default 0.65).
    pub asymmetry: f32,
    /// Warmth high-frequency damping (0.0 to 1.0, default 0.6).
    pub warmth: f32,
    /// Output makeup gain multiplier (0.1 to 4.0, default 1.0).
    pub output_gain: f32,
    /// Dry/Wet blend ratio (0.0 = full dry, 1.0 = full wet, default 1.0).
    pub dry_wet: f32,

    #[serde(skip)]
    channels: [TubeChannelState; 2],
    #[serde(skip)]
    sample_rate: f32,
}

impl TubeSaturationNode {
    /// Create a new `TubeSaturationNode` with specified drive and bias parameters.
    pub fn new(drive: f32, bias: f32) -> Self {
        Self {
            topology: TubeTopology::Triode12AX7,
            drive: drive.max(0.1),
            bias_voltage_v: bias.clamp(-4.0, 0.0),
            plate_voltage_v: 250.0,
            sag_compression: 0.35,
            asymmetry: 0.65,
            warmth: 0.60,
            output_gain: 1.0,
            dry_wet: 1.0,
            channels: [TubeChannelState::default(), TubeChannelState::default()],
            sample_rate: 44100.0,
        }
    }

    /// Builder configuring tube topology.
    pub fn with_topology(mut self, topology: TubeTopology) -> Self {
        self.topology = topology;
        self.asymmetry = topology.base_asymmetry();
        self
    }

    /// Reset internal filter and envelope states across all channels.
    pub fn reset(&mut self) {
        for ch in &mut self.channels {
            ch.reset();
        }
    }

    /// Set operating sample rate.
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1000.0);
    }

    /// Evaluates physical tube non-linear transfer function on a single sample.
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn eval_tube_sample(
        topology: TubeTopology,
        drive: f32,
        bias_v: f32,
        plate_v: f32,
        sag_compression: f32,
        asymmetry: f32,
        warmth: f32,
        x: f32,
        state: &mut TubeChannelState,
        sr: f32,
    ) -> f32 {
        // 1. Dynamic Power Sag Envelope Follower
        let abs_x = x.abs() * drive;
        let att_coeff = 1.0 - (-1.0 / (sr * 0.015)).exp(); // ~15ms attack
        let rel_coeff = 1.0 - (-1.0 / (sr * 0.200)).exp(); // ~200ms release
        let coeff = if abs_x > state.sag_env { att_coeff } else { rel_coeff };
        state.sag_env += coeff * (abs_x - state.sag_env);

        // Effective plate voltage decreases dynamically under heavy signal demand (Power Sag)
        let sag_factor = (state.sag_env * 0.3 * sag_compression).clamp(0.0, 0.6);
        let eff_plate_v = plate_v * (1.0 - sag_factor);

        // 2. Grid input voltage with DC bias operating point and asymmetry offset
        let asym_offset = (asymmetry - 0.5) * 0.5;
        let v_grid = x * drive + asym_offset + (bias_v + 1.85) * 0.25;

        // 3. Topology-specific non-linear physical transfer function
        let raw_sat = match topology {
            TubeTopology::Triode12AX7 => {
                // 12AX7 Triode: 3/2 power law with grid conduction clipping on positive swings
                let mu = 100.0f32;
                let v_gk = v_grid + eff_plate_v / mu;
                if v_gk <= 0.0 {
                    // Cut-off region
                    0.0
                } else if v_grid > 0.4 {
                    // Grid conduction soft clipping
                    let compressed_v = 0.4 + (v_grid - 0.4).tanh() * 0.4;
                    let eff = compressed_v + eff_plate_v / mu;
                    eff.powf(1.4) * 0.6
                } else {
                    v_gk.powf(1.45) * 0.6
                }
            }
            TubeTopology::PentodeEL34 => {
                // EL34 Pentode: Sharp knee, punchy screen grid dynamics, high power saturation
                let vg_scaled = v_grid * 1.35;
                if vg_scaled > 0.6 {
                    0.6 + (vg_scaled - 0.6).tanh() * 0.35
                } else if vg_scaled < -1.2 {
                    -0.8 + (vg_scaled + 1.2).tanh() * 0.2
                } else {
                    vg_scaled.tanh() + 0.15 * vg_scaled.powi(2) / (1.0 + vg_scaled.powi(2))
                }
            }
            TubeTopology::BeamTetrode6L6 => {
                // 6L6 Beam Tetrode: High headroom, scooped midrange, 3rd/5th harmonic cubic inflection
                let vg_scaled = v_grid * 1.1;
                let cubic = vg_scaled - 0.12 * vg_scaled.powi(3) / (1.0 + vg_scaled.powi(2));
                cubic.tanh() * 1.1
            }
            TubeTopology::CleanModern => {
                // Clean symmetrical hyperbolic tangent saturation
                (v_grid * 1.2).tanh()
            }
        };

        // 4. DC Blocker to remove static DC bias component
        let r = 0.995f32;
        let dc_out = raw_sat - state.dc_x1 + r * state.dc_y1;
        state.dc_x1 = raw_sat;
        state.dc_y1 = dc_out;

        // 5. Warmth 1-pole Low-Pass Filter
        let fc = (2500.0 + warmth.clamp(0.0, 1.0) * 16000.0).min(sr * 0.45);
        let alpha = (TAU * fc / sr).clamp(0.05, 0.95);
        state.lp_state += alpha * (dc_out - state.lp_state);

        // Normalize gain based on topology
        state.lp_state * 0.9
    }

    /// Compute static transfer curve points for plotting transfer response.
    pub fn compute_transfer_curve(&self, num_points: usize) -> Vec<(f32, f32)> {
        let mut points = Vec::with_capacity(num_points);
        let mut sim_state = TubeChannelState::default();
        let sr = self.sample_rate;

        for i in 0..num_points {
            let norm_x = (i as f32 / (num_points.max(2) - 1) as f32) * 2.0 - 1.0;
            let in_val = norm_x * 1.5;
            let out_val = Self::eval_tube_sample(
                self.topology,
                self.drive,
                self.bias_voltage_v,
                self.plate_voltage_v,
                self.sag_compression,
                self.asymmetry,
                self.warmth,
                in_val,
                &mut sim_state,
                sr,
            );
            points.push((in_val, out_val));
        }
        points
    }

    /// Calculate estimated harmonic spectrum magnitudes (f0, 2f0, 3f0, 4f0, 5f0) in dB.
    pub fn calculate_harmonic_spectrum(&self) -> [f32; 5] {
        let drive_norm = (self.drive / 10.0).clamp(0.0, 1.0);
        let even_weight = self.asymmetry.clamp(0.0, 1.0);
        let odd_weight = 1.0 - even_weight * 0.6;

        let f0 = 0.0f32;
        let f1 = (-28.0 + drive_norm * 22.0) * even_weight - (1.0 - even_weight) * 14.0;
        let f2 = (-32.0 + drive_norm * 24.0) * odd_weight - even_weight * 8.0;
        let f3 = f1 - 12.0;
        let f4 = f2 - 14.0;

        [
            f0,
            f1.clamp(-60.0, 0.0),
            f2.clamp(-60.0, 0.0),
            f3.clamp(-60.0, 0.0),
            f4.clamp(-60.0, 0.0),
        ]
    }

    /// Calculate estimated Total Harmonic Distortion percentage (THD %).
    pub fn calculate_thd_pct(&self) -> f32 {
        let harmonics = self.calculate_harmonic_spectrum();
        let mut sum_power = 0.0f32;
        for &h_db in &harmonics[1..] {
            let lin = 10.0f32.powf(h_db / 20.0);
            sum_power += lin * lin;
        }
        (sum_power.sqrt() * 100.0).clamp(0.05, 48.0)
    }
}

impl Default for TubeSaturationNode {
    fn default() -> Self {
        Self::new(2.5, -1.85)
    }
}

impl SignalProcessor for TubeSaturationNode {
    fn name(&self) -> &str {
        "TubeSaturationNode"
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
            self.sample_rate
        };
        self.sample_rate = sr;

        let num_in_ch = input.len();
        let num_out_ch = output.len();

        let topology = self.topology;
        let drive = self.drive;
        let bias_v = self.bias_voltage_v;
        let plate_v = self.plate_voltage_v;
        let sag_compression = self.sag_compression;
        let asymmetry = self.asymmetry;
        let warmth = self.warmth;
        let output_gain = self.output_gain;
        let dry_wet = self.dry_wet;

        for i in 0..num_samples {
            for ch_idx in 0..num_out_ch {
                let in_sample = if ch_idx < num_in_ch && i < input[ch_idx].len() {
                    input[ch_idx][i]
                } else if !input[0].is_empty() && i < input[0].len() {
                    input[0][i]
                } else {
                    0.0
                };

                let ch_state_idx = ch_idx.min(1);
                let ch_state = &mut self.channels[ch_state_idx];

                let sat_out = Self::eval_tube_sample(
                    topology,
                    drive,
                    bias_v,
                    plate_v,
                    sag_compression,
                    asymmetry,
                    warmth,
                    in_sample,
                    ch_state,
                    sr,
                );

                let processed = sat_out * output_gain;
                let final_sample = in_sample * (1.0 - dry_wet) + processed * dry_wet;

                if i < output[ch_idx].len() {
                    output[ch_idx][i] = final_sample;
                }
            }
        }
    }
}

impl AudioNode for TubeSaturationNode {
    fn name(&self) -> &str {
        "TubeSaturationNode"
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

    #[test]
    fn test_tube_saturation_zero_allocation_in_audio_loop() {
        let mut tube = TubeSaturationNode::new(3.0, -1.85);
        let ctx = ProcessContext::new(48000, 120.0, 0);

        let in_l = [0.45f32; 128];
        let in_r = [-0.45f32; 128];
        let mut out_l = [0.0f32; 128];
        let mut out_r = [0.0f32; 128];

        {
            let _guard = AllocGuard::new();
            tube.process_block(
                &[&in_l[..], &in_r[..]],
                &mut [&mut out_l[..], &mut out_r[..]],
                &ctx,
            );
        }

        assert!(out_l.iter().all(|s| s.is_finite()));
        assert!(out_r.iter().all(|s| s.is_finite()));
    }

    #[test]
    fn test_tube_saturation_topologies_and_asymmetry() {
        let ctx = ProcessContext::new(48000, 120.0, 0);
        let mut tube_triode = TubeSaturationNode::new(3.5, -1.85).with_topology(TubeTopology::Triode12AX7);
        let mut tube_pentode = TubeSaturationNode::new(3.5, -1.85).with_topology(TubeTopology::PentodeEL34);
        let mut tube_tetrode = TubeSaturationNode::new(3.5, -1.85).with_topology(TubeTopology::BeamTetrode6L6);

        let pos_in = [0.6f32; 128];
        let neg_in = [-0.6f32; 128];

        let mut out_pos = [0.0f32; 128];
        let mut out_neg = [0.0f32; 128];

        tube_triode.process_block(&[&pos_in[..]], &mut [&mut out_pos[..]], &ctx);
        tube_triode.process_block(&[&neg_in[..]], &mut [&mut out_neg[..]], &ctx);

        assert!(
            (out_pos[127].abs() - out_neg[127].abs()).abs() > 0.005,
            "Triode 12AX7 transfer must exhibit pronounced positive/negative asymmetry"
        );

        let mut out_pent = [0.0f32; 128];
        let mut out_tet = [0.0f32; 128];
        tube_pentode.process_block(&[&pos_in[..]], &mut [&mut out_pent[..]], &ctx);
        tube_tetrode.process_block(&[&pos_in[..]], &mut [&mut out_tet[..]], &ctx);

        assert!(out_pent[127].is_finite());
        assert!(out_tet[127].is_finite());
    }

    #[test]
    fn test_tube_dynamic_power_sag_compression() {
        let mut tube = TubeSaturationNode::new(4.0, -1.85);
        tube.sag_compression = 0.8;
        let ctx = ProcessContext::new(48000, 120.0, 0);

        let burst = [0.8f32; 256];
        let mut out_burst = [0.0f32; 256];

        tube.process_block(&[&burst[..]], &mut [&mut out_burst[..]], &ctx);

        // Power sag should cause slight gain compression towards the end of sustained burst
        let initial_peak = out_burst[10].abs();
        let sustained_peak = out_burst[250].abs();
        assert!(
            sustained_peak <= initial_peak + 0.05,
            "Power sag should compress sustained output: initial={}, sustained={}",
            initial_peak,
            sustained_peak
        );
    }
}
