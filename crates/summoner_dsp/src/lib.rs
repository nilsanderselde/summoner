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

//! SIMD synthesis primitives & DSP algorithms for Summoner DAW.

pub mod composites;
pub mod distortion;
pub mod effects;
pub mod filters;
pub mod glitch;
pub mod math;
pub mod modal;
pub mod modulators;
pub mod oscillators;
pub mod traits;
pub mod waveguide;

pub use composites::{
    AetherSynth, AtmosphericPadSynth, CyberpunkSubSynth, FmOperatorPair, GlitchAetherMachine,
    GlitchPercussionSynth, PluckSynth,
};
pub use distortion::{DistortionNode, DistortionType};
pub use effects::{EffectDelay, EffectReverb};
pub use filters::{FilterComb, FilterLadder, FilterSVF};
pub use glitch::{AudioReverse, GlitchGate, GlitchShuffle, GlitchStutter, TapeStop};
pub use math::{MathAdd, MathMult, VCA};
pub use modal::ModalResonator;
pub use modulators::{EnvADSR, LfoShape, MacroKnob, LFO};
pub use oscillators::{NoiseGen, NoiseType, OscPulse, OscSaw, OscSine, OscTriangle};
pub use traits::{ProcessorNodeAdapter, SignalProcessor};
pub use waveguide::KarplusStrongString;



use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};

/// Waveguide Synth Node implementing `AudioNode` for signal graph routing.
#[derive(Debug)]
pub struct PluckedStringNode {
    pub string: KarplusStrongString,
}

impl PluckedStringNode {
    pub fn new(frequency: f32, sample_rate: u32) -> Self {
        let mut string = KarplusStrongString::new(frequency, sample_rate, 0.99);
        string.pluck(1.0);
        Self { string }
    }
}

impl AudioNode for PluckedStringNode {
    fn name(&self) -> &str {
        "PluckedStringNode"
    }

    fn process(
        &mut self,
        _input: &[&[Sample]],
        output: &mut [&mut [Sample]],
        _ctx: &ProcessContext,
    ) {
        if output.is_empty() {
            return;
        }

        let num_samples = output[0].len();
        for i in 0..num_samples {
            let sample = self.string.process_sample();
            for out_ch in output.iter_mut() {
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
    use summoner_core::transport::Transport;

    #[test]
    fn test_waveguide_pluck_decay() {
        let mut string = KarplusStrongString::new(440.0, 44100, 0.95);
        string.pluck(1.0);

        let initial_sample = string.process_sample();
        assert!(initial_sample.abs() > 0.0);

        for _ in 0..10000 {
            string.process_sample();
        }

        let decayed_sample = string.process_sample();
        assert!(decayed_sample.abs() < initial_sample.abs());
    }

    #[test]
    fn test_modal_resonator() {
        let mut modal = ModalResonator::new(440.0, 10.0, 44100);
        let impulse_resp = modal.process_sample(1.0);
        assert!(impulse_resp != 0.0);
    }

    #[test]
    fn test_atomic_dsp_primitives() {
        let transport = Transport::new(44100, 120.0);
        let ctx = ProcessContext::from_transport(&transport);

        let mut saw = OscSaw::new(440.0);
        let mut pulse = OscPulse::new(440.0, 0.5);
        let mut sine = OscSine::new(440.0);
        let mut tri = OscTriangle::new(440.0);
        let mut noise = NoiseGen::new(NoiseType::White);

        let mut buf_out = vec![0.0f32; 64];
        let dummy_in: [&[Sample]; 0] = [];

        saw.process_block(&dummy_in, &mut [&mut buf_out[..]], &ctx);
        assert!(buf_out.iter().any(|v| *v != 0.0));

        pulse.process_block(&dummy_in, &mut [&mut buf_out[..]], &ctx);
        assert!(buf_out.iter().any(|v| *v != 0.0));

        sine.process_block(&dummy_in, &mut [&mut buf_out[..]], &ctx);
        assert!(buf_out.iter().any(|v| *v != 0.0));

        tri.process_block(&dummy_in, &mut [&mut buf_out[..]], &ctx);
        assert!(buf_out.iter().any(|v| *v != 0.0));

        noise.process_block(&dummy_in, &mut [&mut buf_out[..]], &ctx);
        assert!(buf_out.iter().any(|v| *v != 0.0));
    }


    #[test]
    fn test_composite_devices() {
        let transport = Transport::new(44100, 120.0);
        let ctx = ProcessContext::from_transport(&transport);

        let mut aether = AetherSynth::new(440.0);
        aether.trigger(true);

        let mut pluck = PluckSynth::new(440.0);
        pluck.pluck();

        let mut fm = FmOperatorPair::new(440.0, 2.0);
        fm.trigger(true);

        let mut glitch_aether = GlitchAetherMachine::new(440.0);
        glitch_aether.trigger(true);

        let mut cyberpunk = CyberpunkSubSynth::new(55.0);
        cyberpunk.trigger(true);

        let mut pad = AtmosphericPadSynth::new(220.0);
        pad.trigger(true);

        let mut glitch_perc = GlitchPercussionSynth::new(100.0);
        glitch_perc.trigger();

        let mut buf_aether = vec![0.0f32; 64];
        let mut buf_pluck = vec![0.0f32; 64];
        let mut buf_fm = vec![0.0f32; 64];
        let mut buf_glitch_aether = vec![0.0f32; 64];
        let mut buf_cyberpunk = vec![0.0f32; 64];
        let mut buf_pad = vec![0.0f32; 64];
        let mut buf_glitch_perc = vec![0.0f32; 64];

        let dummy_in: [&[Sample]; 0] = [];
        aether.process_block(&dummy_in, &mut [&mut buf_aether[..]], &ctx);
        pluck.process_block(&dummy_in, &mut [&mut buf_pluck[..]], &ctx);
        fm.process_block(&dummy_in, &mut [&mut buf_fm[..]], &ctx);
        glitch_aether.process_block(&dummy_in, &mut [&mut buf_glitch_aether[..]], &ctx);
        cyberpunk.process_block(&dummy_in, &mut [&mut buf_cyberpunk[..]], &ctx);
        pad.process_block(&dummy_in, &mut [&mut buf_pad[..]], &ctx);
        glitch_perc.process_block(&dummy_in, &mut [&mut buf_glitch_perc[..]], &ctx);

        assert!(buf_aether.iter().any(|v| *v != 0.0));
        assert!(buf_pluck.iter().any(|v| *v != 0.0));
        assert!(buf_fm.iter().any(|v| *v != 0.0));
        assert!(buf_glitch_aether.iter().any(|v| *v != 0.0));
        assert!(buf_cyberpunk.iter().any(|v| *v != 0.0));
        assert!(buf_pad.iter().any(|v| *v != 0.0));
        assert!(buf_glitch_perc.iter().any(|v| *v != 0.0));
    }

}
