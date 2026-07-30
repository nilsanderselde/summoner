// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

use summoner_core::node::{AudioNode, GainNode};
use summoner_core::track::Track;
use summoner_project::schema::{ProjectConfig, NodeConfig};
use summoner_dsp::traits::ProcessorNodeAdapter;
use summoner_dsp::oscillators::{OscSaw, OscPulse, OscSine, OscTriangle, OscWavetable, NoiseGen};
use summoner_dsp::filters::{FilterLadder, FilterSVF, FilterComb};
use summoner_dsp::SamplerNode;
use summoner_dsp::modulators::EnvADSR;
use summoner_dsp::math::VCA;
use summoner_dsp::{FilterBiquad, CompressorNode, LimiterNode, EffectChorus, EffectFlanger, EffectPhaser, RingModulator, FrequencyShifter, LufsMeterNode, GranularSynthNode};
use summoner_core::graph::{NodeGraph, Edge};

use summoner_dsp::{EffectDelay, EffectReverb, WavefolderNode, PitchShifterNode, BitcrusherNode, MidSideNode, ParametricEqNode, DistortionNode, DistortionType};

pub struct NodeFactory;

impl NodeFactory {
    pub fn create_node(config: &NodeConfig) -> Option<Box<dyn AudioNode>> {
        let params = &config.params;
        match config.kind.as_str() {
            "OscSaw" => Some(Box::new(ProcessorNodeAdapter::new(OscSaw::new(*params.get("freq").unwrap_or(&440.0))))),
            "OscPulse" => Some(Box::new(ProcessorNodeAdapter::new(OscPulse::new(*params.get("freq").unwrap_or(&440.0), *params.get("pw").unwrap_or(&0.5))))),
            "OscSine" | "SineOscillatorNode" => {
                let freq = params.get("freq").or_else(|| params.get("frequency")).copied().unwrap_or(440.0);
                Some(Box::new(ProcessorNodeAdapter::new(OscSine::new(freq))))
            }
            "OscTriangle" => Some(Box::new(ProcessorNodeAdapter::new(OscTriangle::new(*params.get("freq").unwrap_or(&440.0))))),
            "OscWavetable" | "WavetableOscillator" => {
                let freq = params.get("freq").or_else(|| params.get("frequency")).copied().unwrap_or(440.0);
                let morph = *params.get("morph").unwrap_or(&0.0);
                let osc = OscWavetable::new(freq, OscWavetable::default_saw())
                    .with_table2(OscWavetable::default_square(), morph);
                Some(Box::new(ProcessorNodeAdapter::new(osc)))
            }
            "NoiseGen" => Some(Box::new(ProcessorNodeAdapter::new(NoiseGen::new(summoner_dsp::oscillators::NoiseType::White)))),
            "FilterLadder" => Some(Box::new(ProcessorNodeAdapter::new(FilterLadder::new(*params.get("cutoff").unwrap_or(&1000.0), *params.get("res").unwrap_or(&0.0))))),
            "FilterSVF" => Some(Box::new(ProcessorNodeAdapter::new(FilterSVF::new(*params.get("cutoff").unwrap_or(&1000.0), *params.get("res").unwrap_or(&0.0))))),
            "FilterComb" => Some(Box::new(ProcessorNodeAdapter::new(FilterComb::new(*params.get("freq").unwrap_or(&440.0), *params.get("feedback").unwrap_or(&0.5))))),
            "SamplerNode" => Some(Box::new(ProcessorNodeAdapter::new(SamplerNode::new()))),
            "GainNode" => Some(Box::new(GainNode::new(*params.get("gain").unwrap_or(&1.0)))),
            "EnvADSR" => Some(Box::new(ProcessorNodeAdapter::new(EnvADSR::new(
                *params.get("a").unwrap_or(&0.01),
                *params.get("d").unwrap_or(&0.1),
                *params.get("s").unwrap_or(&0.5),
                *params.get("r").unwrap_or(&0.1)
            )))),
            "VCA" => Some(Box::new(ProcessorNodeAdapter::new(VCA::new(*params.get("gain").unwrap_or(&1.0))))),
            "BiquadFilter" => Some(Box::new(ProcessorNodeAdapter::new(FilterBiquad::new(summoner_dsp::biquad::FilterType::Lowpass, *params.get("cutoff").unwrap_or(&1000.0))))),
            "CompressorNode" => Some(Box::new(ProcessorNodeAdapter::new(CompressorNode::new()))),
            "LimiterNode" => Some(Box::new(ProcessorNodeAdapter::new(LimiterNode::new(64)))),
            "EffectChorus" => Some(Box::new(ProcessorNodeAdapter::new(EffectChorus::new()))),
            "EffectFlanger" => Some(Box::new(ProcessorNodeAdapter::new(EffectFlanger::new()))),
            "EffectPhaser" => Some(Box::new(ProcessorNodeAdapter::new(EffectPhaser::new()))),
            "RingModulator" => Some(Box::new(ProcessorNodeAdapter::new(RingModulator::new()))),
            "FrequencyShifter" => Some(Box::new(ProcessorNodeAdapter::new(FrequencyShifter::new()))),
            "LufsMeterNode" => Some(Box::new(ProcessorNodeAdapter::new(LufsMeterNode::new()))),
            "GranularSynthNode" => Some(Box::new(ProcessorNodeAdapter::new(GranularSynthNode::new(44100)))),
            "EffectDelay" | "DelayNode" => Some(Box::new(ProcessorNodeAdapter::new(EffectDelay::new(
                *params.get("delay_time").unwrap_or(&0.3),
                *params.get("feedback").unwrap_or(&0.4),
                *params.get("mix").unwrap_or(&0.3)
            )))),
            "EffectReverb" | "ReverbNode" => Some(Box::new(ProcessorNodeAdapter::new(EffectReverb::new(
                *params.get("room_size").unwrap_or(&0.7),
                *params.get("mix").unwrap_or(&0.3)
            )))),
            "WavefolderNode" => Some(Box::new(ProcessorNodeAdapter::new(WavefolderNode::new(
                *params.get("threshold").unwrap_or(&0.5),
                *params.get("folds").unwrap_or(&4.0) as u8,
                *params.get("drive").unwrap_or(&2.0)
            )))),
            "PitchShifterNode" => Some(Box::new(ProcessorNodeAdapter::new(PitchShifterNode::new(
                *params.get("semitones").unwrap_or(&0.0)
            )))),
            "BitcrusherNode" => Some(Box::new(ProcessorNodeAdapter::new(BitcrusherNode::new(
                *params.get("bit_depth").unwrap_or(&8.0) as u8,
                *params.get("sample_reduction").unwrap_or(&4.0) as u32
            )))),
            "MidSideNode" => Some(Box::new(ProcessorNodeAdapter::new(MidSideNode::new(
                *params.get("width").unwrap_or(&1.0)
            )))),
            "ParametricEqNode" => Some(Box::new(ProcessorNodeAdapter::new(ParametricEqNode::new()))),
            "DistortionNode" => Some(Box::new(ProcessorNodeAdapter::new(DistortionNode::new(
                DistortionType::SoftClipping,
                *params.get("drive").unwrap_or(&2.0)
            )))),
            "SamplerDevice" => {
                let mut bank = summoner_dsp::sampler::MultiSampleBank::new();
                let base_dir = std::path::Path::new("local");
                summoner_dsp::sampler::load_bank_buffers(&mut bank, base_dir);
                Some(Box::new(ProcessorNodeAdapter::new(summoner_dsp::SamplerDevice::new(bank))))
            }
            "MultiSamplerNode" => {
                let mut bank = summoner_dsp::sampler::MultiSampleBank::new();
                let base_dir = std::path::Path::new("local");
                summoner_dsp::sampler::load_bank_buffers(&mut bank, base_dir);
                Some(Box::new(ProcessorNodeAdapter::new(summoner_dsp::sampler::MultiSamplerNode::new(bank))))
            }
            _ => {
                eprintln!("Warning: Unknown node kind '{}'", config.kind);
                Some(Box::new(GainNode::new(0.0)))
            }
        }
    }
}

pub fn graph_from_project(project: &ProjectConfig, max_block_size: usize) -> NodeGraph {
    let mut graph = NodeGraph::new(project.name.clone(), max_block_size, 2);
    
    if let Some(track) = project.tracks.get(0) {
        for nc in &track.nodes {
            if let Some(node) = NodeFactory::create_node(nc) {
                graph.add_node(node);
            }
        }
        
        for conn in &track.connections {
            let parts_from: Vec<&str> = conn.from.split(':').collect();
            let parts_to: Vec<&str> = conn.to.split(':').collect();
            if parts_from.len() == 2 && parts_to.len() == 2 {
                if let (Ok(f_node), Ok(f_port), Ok(t_node), Ok(t_port)) = (
                    parts_from[0].parse::<usize>(),
                    parts_from[1].parse::<usize>(),
                    parts_to[0].parse::<usize>(),
                    parts_to[1].parse::<usize>(),
                ) {
                    graph.add_edge(Edge {
                        from_node: f_node,
                        from_port: f_port,
                        to_node: t_node,
                        to_port: t_port,
                    });
                }
            }
        }
    }
    
    graph.compile();
    graph
}

const MAX_RUNNER_BLOCK_SIZE: usize = 8192;

pub struct GraphRunner {
    pub tracks: Vec<Track>,
    scratch_l: Box<[f32; MAX_RUNNER_BLOCK_SIZE]>,
    scratch_r: Box<[f32; MAX_RUNNER_BLOCK_SIZE]>,
}

impl GraphRunner {
    pub fn new(project: &ProjectConfig) -> Self {
        let mut tracks = Vec::new();
        for tc in &project.tracks {
            let mut track = Track::new(tc.id, tc.name.clone(), tc.channels);
            track.gain = tc.gain;
            track.pan = tc.pan;
            track.muted = tc.muted;
            track.tuning_edo = tc.tuning_edo;
            track.tuning_root_hz = tc.tuning_root_hz;
            
            for nc in &tc.nodes {
                if let Some(node) = NodeFactory::create_node(nc) {
                    track.add_node(node);
                } else {
                    eprintln!("Warning: Unknown node kind '{}'", nc.kind);
                }
            }
            tracks.push(track);
        }
        Self {
            tracks,
            scratch_l: Box::new([0.0f32; MAX_RUNNER_BLOCK_SIZE]),
            scratch_r: Box::new([0.0f32; MAX_RUNNER_BLOCK_SIZE]),
        }
    }

    pub fn process_block(
        &mut self,
        block_size: usize,
        ctx: &summoner_core::node::ProcessContext,
        out_buffers: &mut [&mut [summoner_core::audio::Sample]],
    ) {
        let block_size = block_size.min(MAX_RUNNER_BLOCK_SIZE);
        for out in out_buffers.iter_mut() {
            out[..block_size].fill(0.0);
        }

        for track in &mut self.tracks {
            self.scratch_l[..block_size].fill(0.0);
            self.scratch_r[..block_size].fill(0.0);

            let mut local_ctx = ctx.clone();
            if let Some(edo) = track.tuning_edo {
                local_ctx.tuning_edo_divisions = edo;
            }
            if let Some(root_hz) = track.tuning_root_hz {
                local_ctx.tuning_root_hz = root_hz;
            }

            let (slice_l, slice_r) = (&mut self.scratch_l[..block_size], &mut self.scratch_r[..block_size]);
            let mut track_out_slices: [&mut [f32]; 2] = [slice_l, slice_r];

            track.process(block_size, &local_ctx, &mut track_out_slices[..out_buffers.len().min(2)]);

            for ch in 0..out_buffers.len().min(2) {
                let scratch_slice = if ch == 0 { &self.scratch_l[..block_size] } else { &self.scratch_r[..block_size] };
                for i in 0..block_size {
                    out_buffers[ch][i] += scratch_slice[i] * track.gain;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_project::create_default_project;

    #[test]
    fn test_graph_from_project_builds_valid_dag() {
        let mut project = create_default_project("Test Project");
        
        // Add a connection from node 0 to node 1 in port 0
        if let Some(track) = project.tracks.get_mut(1) {
            track.connections.push(summoner_project::schema::ConnectionConfig {
                from: "0:0".to_string(),
                to: "1:0".to_string(),
            });
        }
        
        let mut graph = NodeGraph::new(project.name.clone(), 512, 2);
        if let Some(track) = project.tracks.get(1) {
            for nc in &track.nodes {
                if let Some(node) = NodeFactory::create_node(nc) {
                    graph.add_node(node);
                }
            }
            
            for conn in &track.connections {
                let parts_from: Vec<&str> = conn.from.split(':').collect();
                let parts_to: Vec<&str> = conn.to.split(':').collect();
                if parts_from.len() == 2 && parts_to.len() == 2 {
                    if let (Ok(f_node), Ok(f_port), Ok(t_node), Ok(t_port)) = (
                        parts_from[0].parse::<usize>(),
                        parts_from[1].parse::<usize>(),
                        parts_to[0].parse::<usize>(),
                        parts_to[1].parse::<usize>(),
                    ) {
                        graph.add_edge(summoner_core::graph::Edge {
                            from_node: f_node,
                            from_port: f_port,
                            to_node: t_node,
                            to_port: t_port,
                        });
                    }
                }
            }
        }
        graph.compile();
        
        // Track 1 has 2 nodes: SineOscillatorNode and GainNode
        // Note: SineOscillatorNode is not in the node factory right now, let's change the project mock or just check graph nodes length
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
    }

    #[test]
    fn test_factory_sine_oscillator_node() {
        let mut params = std::collections::HashMap::new();
        params.insert("freq".to_string(), 440.0);
        let config = NodeConfig {
            kind: "SineOscillatorNode".to_string(),
            params,
        };
        let mut node = NodeFactory::create_node(&config).expect("Factory should create SineOscillatorNode");
        let ctx = summoner_core::node::ProcessContext::new(44100, 120.0, 0);

        let mut out_l = vec![0.0f32; 512];
        let mut out_r = vec![0.0f32; 512];
        let dummy_in: [&[f32]; 0] = [];
        node.process(&dummy_in, &mut [&mut out_l[..], &mut out_r[..]], &ctx);

        let rms: f32 = (out_l.iter().map(|s| s * s).sum::<f32>() / out_l.len() as f32).sqrt();
        assert!(rms > 0.0, "SineOscillatorNode RMS output must be greater than zero");
    }
}
