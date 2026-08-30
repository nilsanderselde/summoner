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

pub mod ai_mixing;
pub mod air_reed;
pub mod auto_wah;
pub mod autotune;
pub mod biquad;
pub mod bitcrusher;
pub mod bowed_string;
pub mod clavinet;
pub mod composites;
pub mod compressor;
pub mod console_emulation;
pub mod delay;
pub mod distortion;
pub mod dither;
pub mod drum_machine;
pub mod ecosystem_hardware;
pub mod effects;
pub mod electric_piano;
pub mod filters;
pub mod friction_model;
pub mod glitch;
pub mod glottal_pulse;
pub mod grand_piano;
pub mod granular;
pub mod granular_cloud;
pub mod hammer_strike;
pub mod horn_reflection;
pub mod idiophone_resonator;
pub mod jawari_bridge;
pub mod limiter;
pub mod live_session_recorder;
pub mod math;
pub mod meter;
pub mod midside;
pub mod mod_fx;
pub mod modal;
pub mod modal_synthesis;
pub mod modulators;
pub mod multiband_compressor;
pub mod multiband_dynamics;
pub mod neural_dsp;
pub mod neuro_synthesis;
pub mod oscillators;
pub mod oversampling;
pub mod parametric_eq;
pub mod percussion_membrane;
pub mod pitch_shift;
pub mod pitch_shifter;
pub mod pipe_organ;
pub mod plate_tank;
pub mod plucked_string;
pub mod plugin_host;
pub mod quantum_audio;
pub mod reverb;
pub mod ring_mod;
pub mod sample_editor;
pub mod rotary_speaker;
pub mod sampler;
pub mod shakuhachi;
pub mod sitar;
pub mod slicer;
pub mod soundboard_bridge;
pub mod spatial_audio;
pub mod spatial_room;
pub mod spectral_eq;
pub mod spectral_resynthesis;
pub mod spectrogram_art;
pub mod spring_lattice;
pub mod stem_separator;
pub mod tape_saturation;
pub mod tine_resonator;
pub mod tonehole_grid;
pub mod tonewheel_organ;
pub mod track_dsp;
pub mod traits;
pub mod true_peak_limiter;
pub mod tube_saturation;
pub mod tuner;
pub mod visualizer_engine;
pub mod vocal_tract;
pub mod wavefolder;
pub mod waveguide;
pub mod waveguide_brass;
pub mod waveguide_mesh;
pub mod windchest;
pub mod woodwind_jet;
pub mod zero_gravity_fluid;

pub use ai_mixing::*;
pub use air_reed::{
    jet_velocity_from_pressure, meri_kari_to_pitch_multiplier, utaguchi_splitting_function,
    AirReed, AirReedConfig, DEFAULT_PRESSURE_PA, MAX_PRESSURE_PA, MIN_PRESSURE_PA,
};
pub use auto_wah::*;
pub use bowed_string::*;
pub use clavinet::*;
pub use console_emulation::{ConsoleEmulationNode, ConsoleMode};
pub use ecosystem_hardware::*;
pub use electric_piano::*;
pub use friction_model::*;
pub use glottal_pulse::*;
pub use grand_piano::*;
pub use granular_cloud::*;
pub use hammer_strike::*;
pub use horn_reflection::*;
pub use jawari_bridge::*;
pub use live_session_recorder::*;
pub use modal_synthesis::*;
pub use multiband_compressor::MultibandCompressorNode;
pub use multiband_dynamics::*;
pub use neural_dsp::*;
pub use neuro_synthesis::*;
pub use pipe_organ::*;
pub use plate_tank::*;
pub use plucked_string::*;
pub use quantum_audio::*;
pub use rotary_speaker::*;
pub use sample_editor::*;
pub use shakuhachi::*;
pub use sitar::*;
pub use soundboard_bridge::*;
pub use spatial_audio::*;
pub use spatial_room::*;
pub mod spectral_eq_alias {
    pub use crate::spectral_eq::*;
}
pub use spectral_eq::MultiChannelSpectralEqualizerNode;
pub use spectral_resynthesis::*;
pub use spectrogram_art::*;
pub use spring_lattice::*;
pub use tape_saturation::*;
pub use tine_resonator::*;
pub use tonehole_grid::*;
pub use tonewheel_organ::*;
pub use track_dsp::*;
pub use true_peak_limiter::*;
pub use tube_saturation::{TubeChannelState, TubeSaturationNode, TubeTopology};
pub use visualizer_engine::*;
pub use vocal_tract::*;
pub use waveguide_brass::*;
pub use waveguide_mesh::*;
pub use windchest::*;
pub use woodwind_jet::*;
pub use zero_gravity_fluid::*;

pub use plugin_host::{
    scan_plugin_directory, PluginAudioNode, PluginDescriptor, PluginFormat, PluginParamInfo,
    PluginStateConfig,
};

pub use biquad::{FilterBiquad, FilterType};
pub use bitcrusher::BitcrusherNode;
pub use composites::{
    AetherSynth, AtmosphericPadSynth, CyberpunkSubSynth, FmOperatorPair, GlitchAetherMachine,
    GlitchPercussionSynth, PluckSynth, SamplerDevice,
};
pub use compressor::CompressorNode;
pub use delay::EffectDelay;
pub use distortion::{DistortionNode, DistortionType};
pub use dither::{apply_dither, DitherType};
pub use drum_machine::{DrumMachineDevice, DrumPad, MAX_PADS};
pub use effects::{
    DeesserNode, EffectDelay as LegacyEffectDelay, EffectReverb as LegacyEffectReverb,
    HarmonicExciterNode, NoiseGateNode,
};
pub use filters::{
    DcBlockFilter, FilterComb, FilterLadder, FilterSVF, HighCutFilter, LowCutFilter,
};
pub use glitch::{AudioReverse, GlitchGate, GlitchShuffle, GlitchStutter, TapeStop};
pub use granular::GranularSynthNode;
pub use limiter::{LimiterNode, MasterLimiter};
pub use math::{MathAdd, MathMult, VCA};
pub use meter::{
    k_system_headroom, EbuR128LoudnessMeter, KSystemScale, LufsMeterNode, PeakHeadroomAnalyzer,
    TruePeakMeter,
};
pub use midside::{MidSideNode, StereoImager};
pub use mod_fx::{EffectChorus, EffectFlanger, EffectPhaser};
pub use modal::ModalResonator;
pub use modulators::{
    EnvADSR, EnvState, LfoShape, MacroKnob, MacroModulationMatrix, ModulationAssignment,
    ModulationCurve, ModulationSourceId, ModulationTarget, ModulationTargetId, LFO,
};
pub use oscillators::{
    render_buffer_to_wavetable, NoiseGen, NoiseType, OscPulse, OscSaw, OscSine, OscTriangle,
    OscWavetable, SimdPolyVoice, SimdPolyWavetableOscillator, DEFAULT_MAX_VOICES, WAVETABLE_SIZE,
};
pub use oversampling::Oversampler;
pub use parametric_eq::{EqBand, ParametricEqNode};
pub use pitch_shift::{ElasticWarpEngine, WarpMarker, WsolaTimeStretcher};
pub use pitch_shifter::PitchShifterNode;
pub use reverb::{ConvolutionReverbNode, EffectReverb};
pub use ring_mod::{FrequencyShifter, RingModWaveform, RingModulator};
pub use sampler::{SampleBuffer, SamplerNode};
pub use slicer::{AutoSlicer, SliceMarker, ONNX_TRANSIENT_MODEL_BYTES};
pub use stem_separator::{
    MultiTrackAudioRouter, StemMetadata, StemMetadataParser, StemSeparator,
    ONNX_STEM_SEPARATOR_MODEL_BYTES,
};
pub use traits::{ProcessorNodeAdapter, SignalProcessor};
pub use wavefolder::WavefolderNode;
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
    fn test_tier13_dsp_nodes_integration() {
        let transport = Transport::new(44100, 120.0);
        let ctx = ProcessContext::from_transport(&transport);

        let mut wavefolder = WavefolderNode::new(0.5, 4, 2.0);
        let mut pitch_shifter = PitchShifterNode::new(3.0);
        let mut bitcrusher = BitcrusherNode::new(8, 2);
        let mut midside = MidSideNode::new(1.5);
        let mut eq = ParametricEqNode::new();

        let in_buf_l = vec![0.5f32; 64];
        let in_buf_r = vec![-0.5f32; 64];
        let mut out_l = vec![0.0f32; 64];
        let mut out_r = vec![0.0f32; 64];

        wavefolder.process_block(&[&in_buf_l[..]], &mut [&mut out_l[..]], &ctx);
        pitch_shifter.process_block(&[&in_buf_l[..]], &mut [&mut out_l[..]], &ctx);
        bitcrusher.process_block(&[&in_buf_l[..]], &mut [&mut out_l[..]], &ctx);
        midside.process_block(
            &[&in_buf_l[..], &in_buf_r[..]],
            &mut [&mut out_l[..], &mut out_r[..]],
            &ctx,
        );
        eq.process_block(&[&in_buf_l[..]], &mut [&mut out_l[..]], &ctx);

        assert!(out_l.iter().all(|s| s.is_finite()));
        assert!(out_r.iter().all(|s| s.is_finite()));
    }

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
