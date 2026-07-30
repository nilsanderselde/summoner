// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Multiband compressor DSP node splitting audio into low/mid/high bands.

use crate::compressor::CompressorNode;
use crate::filters::FilterSVF;
use crate::traits::SignalProcessor;
use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;

/// Multiband compressor processing 3 distinct frequency ranges (Low, Mid, High).
#[derive(Debug)]
pub struct MultibandCompressorNode {
    pub low_mid_freq: f32,
    pub mid_high_freq: f32,
    pub low_comp: CompressorNode,
    pub mid_comp: CompressorNode,
    pub high_comp: CompressorNode,
    filter_low: FilterSVF,
    filter_mid_low: FilterSVF,
    filter_mid_high: FilterSVF,
    filter_high: FilterSVF,
}

impl MultibandCompressorNode {
    pub fn new() -> Self {
        let low_mid_freq = 200.0;
        let mid_high_freq = 2000.0;
        Self {
            low_mid_freq,
            mid_high_freq,
            low_comp: CompressorNode::with_params(-18.0, 3.0, 10.0, 100.0, 0.0),
            mid_comp: CompressorNode::with_params(-16.0, 2.5, 15.0, 120.0, 0.0),
            high_comp: CompressorNode::with_params(-14.0, 2.0, 5.0, 80.0, 0.0),
            filter_low: FilterSVF::new(low_mid_freq, 0.707),
            filter_mid_low: FilterSVF::new(low_mid_freq, 0.707),
            filter_mid_high: FilterSVF::new(mid_high_freq, 0.707),
            filter_high: FilterSVF::new(mid_high_freq, 0.707),
        }
    }
}

impl Default for MultibandCompressorNode {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalProcessor for MultibandCompressorNode {
    fn name(&self) -> &str {
        "MultibandCompressorNode"
    }

    fn process_block(&mut self, input: &[&[Sample]], output: &mut [&mut [Sample]], ctx: &ProcessContext) {
        if input.is_empty() || output.is_empty() {
            return;
        }

        let num_samples = input[0].len().min(output[0].len());
        let sr = if ctx.sample_rate > 0 { ctx.sample_rate } else { 44100 };

        let mut low_in = vec![0.0f32; num_samples];
        let mut mid_in = vec![0.0f32; num_samples];
        let mut high_in = vec![0.0f32; num_samples];

        let mut low_out = vec![0.0f32; num_samples];
        let mut mid_out = vec![0.0f32; num_samples];
        let mut high_out = vec![0.0f32; num_samples];

        // Split input into Low, Mid, High bands using SVF crossovers
        for i in 0..num_samples {
            let sample = input[0][i];
            let (low, _lp_b, _lp_h) = self.filter_low.process_sample(sample, sr);
            let (_hp_l, _hp_b, mid_raw) = self.filter_mid_low.process_sample(sample, sr);
            let (mid, _m_b, _m_h) = self.filter_mid_high.process_sample(mid_raw, sr);
            let (_h_l, _h_b, high) = self.filter_high.process_sample(sample, sr);

            low_in[i] = low;
            mid_in[i] = mid;
            high_in[i] = high;
        }

        // Compress each band independently
        self.low_comp.process_block(&[&low_in[..]], &mut [&mut low_out[..]], ctx);
        self.mid_comp.process_block(&[&mid_in[..]], &mut [&mut mid_out[..]], ctx);
        self.high_comp.process_block(&[&high_in[..]], &mut [&mut high_out[..]], ctx);

        // Sum bands back together into output channels
        for i in 0..num_samples {
            let summed = low_out[i] + mid_out[i] + high_out[i];
            for out_ch in output.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = summed;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiband_compressor_processing() {
        let mut mb_comp = MultibandCompressorNode::new();
        let ctx = ProcessContext::new(44100, 120.0, 0);

        let input_sig = vec![0.8f32; 256];
        let mut out_sig = vec![0.0f32; 256];

        mb_comp.process_block(&[&input_sig[..]], &mut [&mut out_sig[..]], &ctx);

        assert!(out_sig.iter().all(|s| s.is_finite()), "Multiband output must be finite");
        assert!(out_sig.iter().any(|s| *s != 0.0), "Multiband output should produce non-zero signal");
    }
}
