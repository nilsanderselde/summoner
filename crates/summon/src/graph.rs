// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

use summoner_core::node::{AudioNode, GainNode};
use summoner_core::track::Track;
use summoner_project::schema::ProjectConfig;
use summoner_dsp::traits::ProcessorNodeAdapter;
use summoner_dsp::oscillators::{OscSaw, OscPulse, OscSine, OscTriangle, NoiseGen};
use summoner_dsp::filters::{FilterLadder, FilterSVF, FilterComb};
use summoner_dsp::SamplerNode;
use summoner_dsp::{FilterBiquad, CompressorNode, LimiterNode, EffectChorus, EffectFlanger, EffectPhaser, RingModulator, FrequencyShifter, LufsMeterNode, GranularSynthNode};

pub struct NodeFactory;

impl NodeFactory {
    pub fn create_node(kind: &str) -> Option<Box<dyn AudioNode>> {
        match kind {
            "OscSaw" => Some(Box::new(ProcessorNodeAdapter::new(OscSaw::new(440.0)))),
            "OscPulse" => Some(Box::new(ProcessorNodeAdapter::new(OscPulse::new(440.0, 0.5)))),
            "OscSine" => Some(Box::new(ProcessorNodeAdapter::new(OscSine::new(440.0)))),
            "OscTriangle" => Some(Box::new(ProcessorNodeAdapter::new(OscTriangle::new(440.0)))),
            "NoiseGen" => Some(Box::new(ProcessorNodeAdapter::new(NoiseGen::new(summoner_dsp::oscillators::NoiseType::White)))),
            "FilterLadder" => Some(Box::new(ProcessorNodeAdapter::new(FilterLadder::new(1000.0, 0.0)))),
            "FilterSVF" => Some(Box::new(ProcessorNodeAdapter::new(FilterSVF::new(1000.0, 0.0)))),
            "FilterComb" => Some(Box::new(ProcessorNodeAdapter::new(FilterComb::new(440.0, 0.5)))),
            "SamplerNode" => Some(Box::new(ProcessorNodeAdapter::new(SamplerNode::new()))),
            "GainNode" => Some(Box::new(GainNode::new(1.0))),
            "BiquadFilter" => Some(Box::new(ProcessorNodeAdapter::new(FilterBiquad::new(summoner_dsp::biquad::FilterType::Lowpass, 1000.0)))),
            "CompressorNode" => Some(Box::new(ProcessorNodeAdapter::new(CompressorNode::new()))),
            "LimiterNode" => Some(Box::new(ProcessorNodeAdapter::new(LimiterNode::new(64)))),
            "EffectChorus" => Some(Box::new(ProcessorNodeAdapter::new(EffectChorus::new()))),
            "EffectFlanger" => Some(Box::new(ProcessorNodeAdapter::new(EffectFlanger::new()))),
            "EffectPhaser" => Some(Box::new(ProcessorNodeAdapter::new(EffectPhaser::new()))),
            "RingModulator" => Some(Box::new(ProcessorNodeAdapter::new(RingModulator::new()))),
            "FrequencyShifter" => Some(Box::new(ProcessorNodeAdapter::new(FrequencyShifter::new()))),
            "LufsMeterNode" => Some(Box::new(ProcessorNodeAdapter::new(LufsMeterNode::new()))),
            "GranularSynthNode" => Some(Box::new(ProcessorNodeAdapter::new(GranularSynthNode::new(44100)))),
            _ => None,
        }
    }
}

pub struct GraphRunner {
    pub tracks: Vec<Track>,
}

impl GraphRunner {
    pub fn new(project: &ProjectConfig) -> Self {
        let mut tracks = Vec::new();
        for tc in &project.tracks {
            let mut track = Track::new(tc.id, tc.name.clone(), tc.channels);
            track.gain = tc.gain;
            track.pan = tc.pan;
            track.muted = tc.muted;
            
            for nc in &tc.nodes {
                if let Some(node) = NodeFactory::create_node(&nc.kind) {
                    track.add_node(node);
                } else {
                    eprintln!("Warning: Unknown node kind '{}'", nc.kind);
                }
            }
            tracks.push(track);
        }
        Self { tracks }
    }

    pub fn process_block(
        &mut self,
        block_size: usize,
        ctx: &summoner_core::node::ProcessContext,
        out_buffers: &mut [&mut [summoner_core::audio::Sample]],
    ) {
        for out in out_buffers.iter_mut() {
            out[..block_size].fill(0.0);
        }

        let mut track_out: Vec<Vec<summoner_core::audio::Sample>> = vec![vec![0.0; block_size]; out_buffers.len()];
        for track in &mut self.tracks {
            let mut track_out_slices: Vec<&mut [summoner_core::audio::Sample]> = track_out.iter_mut().map(|v| &mut v[..block_size]).collect();
            track.process(block_size, ctx, &mut track_out_slices);

            for ch in 0..out_buffers.len() {
                for i in 0..block_size {
                    out_buffers[ch][i] += track_out_slices[ch][i] * track.gain;
                }
            }
        }
    }
}
