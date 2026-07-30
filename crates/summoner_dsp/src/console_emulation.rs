// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! Console emulation DSP node modeling analog mixing desk character (Neve, SSL, API).

use crate::traits::SignalProcessor;
use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;

/// Console emulation mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleMode {
    /// Warm UK transformer saturation with low-end weight (Neve-style).
    Neve = 0,
    /// Punchy, clean drive with tight bass and presence (SSL-style).
    SSL = 1,
    /// Aggressive mid-range punch and transformer bite (API-style).
    API = 2,
}

impl ConsoleMode {
    pub fn from_f32(val: f32) -> Self {
        match val.round() as i32 {
            0 => ConsoleMode::Neve,
            1 => ConsoleMode::SSL,
            _ => ConsoleMode::API,
        }
    }
}

/// Console emulation character node.
#[derive(Debug)]
pub struct ConsoleEmulationNode {
    pub mode: ConsoleMode,
    pub drive: f32,
    pub warmth: f32,
    hp_state: f32,
    lp_state: f32,
}

impl ConsoleEmulationNode {
    pub fn new(mode: ConsoleMode, drive: f32) -> Self {
        Self {
            mode,
            drive: drive.max(0.0),
            warmth: 0.5,
            hp_state: 0.0,
            lp_state: 0.0,
        }
    }
}

impl Default for ConsoleEmulationNode {
    fn default() -> Self {
        Self::new(ConsoleMode::Neve, 1.0)
    }
}

impl SignalProcessor for ConsoleEmulationNode {
    fn name(&self) -> &str {
        "ConsoleEmulationNode"
    }

    fn process_block(&mut self, input: &[&[Sample]], output: &mut [&mut [Sample]], _ctx: &ProcessContext) {
        if input.is_empty() || output.is_empty() {
            return;
        }

        let num_samples = input[0].len().min(output[0].len());
        let drive_factor = 1.0 + self.drive * 1.5;

        for i in 0..num_samples {
            let x = input[0][i] * drive_factor;

            let saturated = match self.mode {
                ConsoleMode::Neve => {
                    // Soft transformer saturation with low-frequency harmonic boost
                    let sat = (x * 1.2).tanh();
                    let low_bump = x - self.hp_state;
                    self.hp_state = self.hp_state * 0.95 + x * 0.05;
                    sat + low_bump * 0.15 * self.warmth
                }
                ConsoleMode::SSL => {
                    // Crisp, tight drive with slight high-frequency sheen
                    let sat = (x * 1.1).atan() * 1.1;
                    let high_sheen = sat - self.lp_state;
                    self.lp_state = self.lp_state * 0.7 + sat * 0.3;
                    sat + high_sheen * 0.1
                }
                ConsoleMode::API => {
                    // Aggressive mid-range punch
                    let asymmetrical = x + 0.1 * x * x;
                    (asymmetrical * 1.3).tanh()
                }
            };

            for out_ch in output.iter_mut() {
                if i < out_ch.len() {
                    out_ch[i] = saturated;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_emulation_modes() {
        let ctx = ProcessContext::new(44100, 120.0, 0);
        let input_sig = vec![0.5f32; 64];

        for mode in [ConsoleMode::Neve, ConsoleMode::SSL, ConsoleMode::API] {
            let mut console = ConsoleEmulationNode::new(mode, 1.5);
            let mut out_sig = vec![0.0f32; 64];

            console.process_block(&[&input_sig[..]], &mut [&mut out_sig[..]], &ctx);

            assert!(out_sig.iter().all(|s| s.is_finite()));
            assert!(out_sig[63] != 0.0, "Console emulation mode {:?} should process audio", mode);
        }
    }
}
