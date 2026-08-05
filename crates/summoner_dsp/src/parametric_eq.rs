// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.

//! 8-Band Parametric Equalizer processor node with magnitude response curve computation.

use crate::biquad::{FilterBiquad, FilterType};
use crate::traits::SignalProcessor;
use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;

/// Individual EQ Band Configuration.
#[derive(Debug, Clone, Copy)]
pub struct EqBand {
    pub filter_type: FilterType,
    pub freq: f32,
    pub gain_db: f32,
    pub q: f32,
    pub enabled: bool,
}

impl Default for EqBand {
    fn default() -> Self {
        Self {
            filter_type: FilterType::Peaking,
            freq: 1000.0,
            gain_db: 0.0,
            q: 1.0,
            enabled: true,
        }
    }
}

/// 8-Band Parametric Equalizer DSP node.
#[derive(Debug)]
pub struct ParametricEqNode {
    pub bands: [EqBand; 8],
    biquads: [FilterBiquad; 8],
}

impl ParametricEqNode {
    pub fn new() -> Self {
        let default_freqs = [60.0, 150.0, 400.0, 1000.0, 2500.0, 6000.0, 12000.0, 16000.0];
        let bands = [EqBand::default(); 8];
        let biquads = std::array::from_fn(|_i| FilterBiquad::new(FilterType::Peaking, 44100.0));

        let mut eq = Self { bands, biquads };
        for (i, band) in eq.bands.iter_mut().enumerate() {
            band.freq = default_freqs[i];
            if i == 0 {
                band.filter_type = FilterType::LowShelf;
            } else if i == 7 {
                band.filter_type = FilterType::HighShelf;
            }
            eq.biquads[i].filter_type = band.filter_type;
            eq.biquads[i].freq = band.freq;
            eq.biquads[i].calculate_coeffs();
        }

        eq
    }

    pub fn set_band(
        &mut self,
        idx: usize,
        filter_type: FilterType,
        freq: f32,
        gain_db: f32,
        q: f32,
        enabled: bool,
    ) {
        if idx < 8 {
            self.bands[idx] = EqBand {
                filter_type,
                freq: freq.clamp(20.0, 20000.0),
                gain_db,
                q: q.max(0.1),
                enabled,
            };
            self.biquads[idx].filter_type = filter_type;
            self.biquads[idx].freq = freq;
            self.biquads[idx].gain_db = gain_db;
            self.biquads[idx].q = q;
            self.biquads[idx].calculate_coeffs();
        }
    }

    /// Calculate combined magnitude response in dB for a slice of frequencies at given sample rate.
    pub fn response_curve(&self, freqs: &[f32], sample_rate: u32) -> Vec<f32> {
        let sr = sample_rate as f32;
        freqs
            .iter()
            .map(|&f| {
                let mut total_db = 0.0;
                for band in &self.bands {
                    if !band.enabled || band.gain_db == 0.0 {
                        continue;
                    }
                    let ratio = f / band.freq.max(1.0);
                    match band.filter_type {
                        FilterType::Peaking => {
                            let bell = 1.0 / (1.0 + (ratio - 1.0 / ratio).powi(2) * band.q.powi(2));
                            total_db += band.gain_db * bell;
                        }
                        FilterType::LowShelf => {
                            if f < band.freq {
                                total_db += band.gain_db;
                            } else if f < band.freq * 2.0 {
                                let factor = 1.0 - (f - band.freq) / band.freq;
                                total_db += band.gain_db * factor.max(0.0);
                            }
                        }
                        FilterType::HighShelf => {
                            if f > band.freq {
                                total_db += band.gain_db;
                            } else if f > band.freq * 0.5 {
                                let factor = (f - band.freq * 0.5) / (band.freq * 0.5);
                                total_db += band.gain_db * factor.max(0.0);
                            }
                        }
                        FilterType::Notch => {
                            if (f - band.freq).abs() < band.freq * 0.1 {
                                total_db -= 24.0;
                            }
                        }
                        _ => {
                            let _ = sr;
                        }
                    }
                }
                total_db
            })
            .collect()
    }
}

impl Default for ParametricEqNode {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalProcessor for ParametricEqNode {
    fn name(&self) -> &str {
        "ParametricEqNode"
    }

    fn process_block(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        if inputs.is_empty() || outputs.is_empty() {
            return;
        }

        let num_samples = outputs[0].len();
        let sr = if ctx.sample_rate > 0 {
            ctx.sample_rate as f32
        } else {
            44100.0
        };

        for b in self.biquads.iter_mut() {
            if b.sample_rate != sr {
                b.sample_rate = sr;
                b.calculate_coeffs();
            }
        }

        for i in 0..num_samples {
            let mut sample = if !inputs[0].is_empty() && i < inputs[0].len() {
                inputs[0][i]
            } else {
                0.0
            };

            for (b_idx, band) in self.bands.iter().enumerate() {
                if band.enabled {
                    sample = self.biquads[b_idx].process_sample(sample);
                }
            }

            for out_ch in outputs.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = sample;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parametric_eq_response_curve() {
        let mut eq = ParametricEqNode::new();
        eq.set_band(3, FilterType::Peaking, 1000.0, 6.0, 1.0, true);

        let freqs = vec![100.0, 1000.0, 10000.0];
        let curve = eq.response_curve(&freqs, 44100);

        assert_eq!(curve.len(), 3);
        assert!(
            (curve[1] - 6.0).abs() < 0.1,
            "1000Hz response should match peak gain"
        );
    }
}
