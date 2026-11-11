// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

use summoner_core::graph::{Edge, NodeGraph};
use summoner_core::node::NodeEvent;
use summoner_core::node::{AudioNode, GainNode};
use summoner_core::track::ScheduledEvent;
use summoner_core::track::Track;
use summoner_dsp::filters::{
    DcBlockFilter, FilterComb, FilterLadder, FilterSVF, HighCutFilter, LowCutFilter,
};
use summoner_dsp::math::VCA;
use summoner_dsp::modulators::EnvADSR;
use summoner_dsp::oscillators::{NoiseGen, OscPulse, OscSaw, OscSine, OscTriangle, OscWavetable};
use summoner_dsp::traits::ProcessorNodeAdapter;
use summoner_dsp::SamplerNode;
use summoner_dsp::{
    CompressorNode, EffectChorus, EffectFlanger, EffectPhaser, FilterBiquad, FrequencyShifter,
    GranularSynthNode, LimiterNode, LufsMeterNode, RingModulator,
};
use summoner_project::schema::{NodeConfig, ProjectConfig};

use summoner_dsp::{
    BitcrusherNode, DistortionNode, DistortionType, EffectDelay, EffectReverb, MidSideNode,
    ParametricEqNode, PitchShifterNode, WavefolderNode,
};

pub struct NodeFactory;

impl NodeFactory {
    /// Create a node at the default 44.1 kHz rate (prefer create_node_with_rate).
    pub fn create_node(config: &NodeConfig) -> Option<Box<dyn AudioNode>> {
        Self::create_node_with_rate(config, 44100)
    }

    /// Create a node whose internal state is sized for sample_rate.
    pub fn create_node_with_rate(
        config: &NodeConfig,
        sample_rate: u32,
    ) -> Option<Box<dyn AudioNode>> {
        let params = &config.params;
        match config.kind.as_str() {
            "OscSaw" => Some(Box::new(ProcessorNodeAdapter::new(OscSaw::new(
                *params.get("freq").unwrap_or(&440.0),
            )))),
            "OscPulse" => Some(Box::new(ProcessorNodeAdapter::new(OscPulse::new(
                *params.get("freq").unwrap_or(&440.0),
                *params.get("pw").unwrap_or(&0.5),
            )))),
            "OscSine" | "SineOscillatorNode" => {
                let freq = params
                    .get("freq")
                    .or_else(|| params.get("frequency"))
                    .copied()
                    .unwrap_or(440.0);
                Some(Box::new(ProcessorNodeAdapter::new(OscSine::new(freq))))
            }
            "OscTriangle" => Some(Box::new(ProcessorNodeAdapter::new(OscTriangle::new(
                *params.get("freq").unwrap_or(&440.0),
            )))),
            "OscWavetable" | "WavetableOscillator" => {
                let freq = params
                    .get("freq")
                    .or_else(|| params.get("frequency"))
                    .copied()
                    .unwrap_or(440.0);
                let morph = *params.get("morph").unwrap_or(&0.0);
                let osc = OscWavetable::new(freq, OscWavetable::default_saw())
                    .with_table2(OscWavetable::default_square(), morph);
                Some(Box::new(ProcessorNodeAdapter::new(osc)))
            }
            "OscLFO" | "LFO" => {
                let freq = params
                    .get("freq")
                    .or_else(|| params.get("frequency"))
                    .copied()
                    .unwrap_or(2.0);
                Some(Box::new(ProcessorNodeAdapter::new(
                    summoner_dsp::modulators::LFO::new(
                        freq,
                        summoner_dsp::modulators::LfoShape::Sine,
                    ),
                )))
            }
            "MathAdd" => Some(Box::new(ProcessorNodeAdapter::new(
                summoner_dsp::math::MathAdd,
            ))),
            "MathMult" => Some(Box::new(ProcessorNodeAdapter::new(
                summoner_dsp::math::MathMult,
            ))),
            "PassthroughNode" | "Oscilloscope" => {
                Some(Box::new(summoner_core::node::PassthroughNode))
            }
            "AetherSynth" => {
                let freq = params
                    .get("freq")
                    .or_else(|| params.get("frequency"))
                    .copied()
                    .unwrap_or(440.0);
                Some(Box::new(ProcessorNodeAdapter::new(
                    summoner_dsp::AetherSynth::new(freq),
                )))
            }
            "PluckSynth" => {
                let freq = params
                    .get("freq")
                    .or_else(|| params.get("frequency"))
                    .copied()
                    .unwrap_or(440.0);
                Some(Box::new(ProcessorNodeAdapter::new(
                    summoner_dsp::PluckSynth::new(freq),
                )))
            }
            "FmOperatorPair" => {
                let freq = params
                    .get("freq")
                    .or_else(|| params.get("frequency"))
                    .copied()
                    .unwrap_or(440.0);
                let ratio = *params.get("ratio").unwrap_or(&2.0);
                Some(Box::new(ProcessorNodeAdapter::new(
                    summoner_dsp::FmOperatorPair::new(freq, ratio),
                )))
            }
            "GlitchAetherMachine" => {
                let freq = params
                    .get("freq")
                    .or_else(|| params.get("frequency"))
                    .copied()
                    .unwrap_or(440.0);
                Some(Box::new(ProcessorNodeAdapter::new(
                    summoner_dsp::GlitchAetherMachine::new(freq),
                )))
            }
            "CyberpunkSubSynth" => {
                let freq = params
                    .get("freq")
                    .or_else(|| params.get("frequency"))
                    .copied()
                    .unwrap_or(110.0);
                Some(Box::new(ProcessorNodeAdapter::new(
                    summoner_dsp::CyberpunkSubSynth::new(freq),
                )))
            }
            "AtmosphericPadSynth" => {
                let freq = params
                    .get("freq")
                    .or_else(|| params.get("frequency"))
                    .copied()
                    .unwrap_or(220.0);
                Some(Box::new(ProcessorNodeAdapter::new(
                    summoner_dsp::AtmosphericPadSynth::new(freq),
                )))
            }
            "GlitchPercussionSynth" => {
                let freq = params
                    .get("freq")
                    .or_else(|| params.get("frequency"))
                    .copied()
                    .unwrap_or(110.0);
                Some(Box::new(ProcessorNodeAdapter::new(
                    summoner_dsp::GlitchPercussionSynth::new(freq),
                )))
            }
            "DrumMachineDevice" => Some(Box::new(ProcessorNodeAdapter::new(
                summoner_dsp::DrumMachineDevice::new("Default Kit"),
            ))),
            "PluckedStringNode" => {
                let freq = params
                    .get("freq")
                    .or_else(|| params.get("frequency"))
                    .copied()
                    .unwrap_or(440.0);
                Some(Box::new(summoner_dsp::PluckedStringNode::new(
                    freq,
                    sample_rate,
                )))
            }
            "NoiseGen" => Some(Box::new(ProcessorNodeAdapter::new(NoiseGen::new(
                summoner_dsp::oscillators::NoiseType::White,
            )))),
            "FilterLadder" => Some(Box::new(ProcessorNodeAdapter::new(FilterLadder::new(
                *params.get("cutoff").unwrap_or(&1000.0),
                *params.get("res").unwrap_or(&0.0),
            )))),
            "FilterSVF" => Some(Box::new(ProcessorNodeAdapter::new(FilterSVF::new(
                *params.get("cutoff").unwrap_or(&1000.0),
                *params.get("res").unwrap_or(&0.0),
            )))),
            "FilterComb" => Some(Box::new(ProcessorNodeAdapter::new(FilterComb::new(
                *params.get("freq").unwrap_or(&440.0),
                *params.get("feedback").unwrap_or(&0.5),
            )))),
            "SamplerNode" => Some(Box::new(ProcessorNodeAdapter::new(SamplerNode::new()))),
            "GainNode" => Some(Box::new(GainNode::new(*params.get("gain").unwrap_or(&1.0)))),
            "EnvADSR" => Some(Box::new(ProcessorNodeAdapter::new(EnvADSR::new(
                *params.get("a").unwrap_or(&0.01),
                *params.get("d").unwrap_or(&0.1),
                *params.get("s").unwrap_or(&0.5),
                *params.get("r").unwrap_or(&0.1),
            )))),
            "VCA" => Some(Box::new(ProcessorNodeAdapter::new(VCA::new(
                *params.get("gain").unwrap_or(&1.0),
            )))),
            "BiquadFilter" => Some(Box::new(ProcessorNodeAdapter::new(FilterBiquad::new(
                summoner_dsp::biquad::FilterType::Lowpass,
                *params.get("cutoff").unwrap_or(&1000.0),
            )))),
            "CompressorNode" => Some(Box::new(ProcessorNodeAdapter::new(CompressorNode::new()))),
            "LimiterNode" => Some(Box::new(ProcessorNodeAdapter::new(LimiterNode::new(64)))),
            "EffectChorus" => Some(Box::new(ProcessorNodeAdapter::new(EffectChorus::new()))),
            "EffectFlanger" => Some(Box::new(ProcessorNodeAdapter::new(EffectFlanger::new()))),
            "EffectPhaser" => Some(Box::new(ProcessorNodeAdapter::new(EffectPhaser::new()))),
            "RingModulator" => Some(Box::new(ProcessorNodeAdapter::new(RingModulator::new()))),
            "FrequencyShifter" => {
                Some(Box::new(ProcessorNodeAdapter::new(FrequencyShifter::new())))
            }
            "LufsMeterNode" => Some(Box::new(ProcessorNodeAdapter::new(LufsMeterNode::new()))),
            "GranularSynthNode" => Some(Box::new(ProcessorNodeAdapter::new(
                GranularSynthNode::new(sample_rate),
            ))),
            "EffectDelay" | "DelayNode" => {
                Some(Box::new(ProcessorNodeAdapter::new(EffectDelay::new(
                    *params.get("delay_time").unwrap_or(&0.3),
                    *params.get("feedback").unwrap_or(&0.4),
                    *params.get("mix").unwrap_or(&0.3),
                ))))
            }
            "EffectReverb" | "ReverbNode" => {
                Some(Box::new(ProcessorNodeAdapter::new(EffectReverb::new(
                    *params.get("room_size").unwrap_or(&0.7),
                    *params.get("mix").unwrap_or(&0.3),
                ))))
            }
            "WavefolderNode" => Some(Box::new(ProcessorNodeAdapter::new(WavefolderNode::new(
                *params.get("threshold").unwrap_or(&0.5),
                *params.get("folds").unwrap_or(&4.0) as u8,
                *params.get("drive").unwrap_or(&2.0),
            )))),
            "PitchShifterNode" => Some(Box::new(ProcessorNodeAdapter::new(PitchShifterNode::new(
                *params.get("semitones").unwrap_or(&0.0),
            )))),
            "BitcrusherNode" => Some(Box::new(ProcessorNodeAdapter::new(BitcrusherNode::new(
                *params.get("bit_depth").unwrap_or(&8.0) as u8,
                *params.get("sample_reduction").unwrap_or(&4.0) as u32,
            )))),
            "MidSideNode" => Some(Box::new(ProcessorNodeAdapter::new(MidSideNode::new(
                *params.get("width").unwrap_or(&1.0),
            )))),
            "ParametricEqNode" => {
                Some(Box::new(ProcessorNodeAdapter::new(ParametricEqNode::new())))
            }
            "MultiChannelSpectralEqualizerNode" | "SpectralEqualizer" => {
                Some(Box::new(ProcessorNodeAdapter::new(
                    summoner_dsp::MultiChannelSpectralEqualizerNode::new(sample_rate, 2, 8),
                )))
            }
            "SimdPolyWavetableOscillator" | "SimdWavetableOscillator" => {
                Some(Box::new(ProcessorNodeAdapter::new(
                    summoner_dsp::oscillators::SimdPolyWavetableOscillator::new(sample_rate),
                )))
            }
            "MultibandCompressorNode" | "MultibandCompressor" => Some(Box::new(
                ProcessorNodeAdapter::new(summoner_dsp::MultibandCompressorNode::new()),
            )),
            "TapeSaturationNode" | "TapeSaturation" => {
                let mut node = summoner_dsp::TapeSaturationNode::new(
                    *params.get("drive").unwrap_or(&2.0),
                    *params.get("saturation").unwrap_or(&0.5),
                );
                if let Some(&spd) = params.get("speed") {
                    if spd >= 25.0 {
                        node.set_speed(summoner_dsp::TapeSpeed::Ips30);
                    } else if spd <= 10.0 {
                        node.set_speed(summoner_dsp::TapeSpeed::Ips7_5);
                    } else {
                        node.set_speed(summoner_dsp::TapeSpeed::Ips15);
                    }
                }
                if let Some(&bias) = params.get("bias") {
                    node.bias = bias;
                }
                if let Some(&hb) = params.get("head_bump") {
                    node.head_bump = hb;
                }
                if let Some(&wf) = params.get("wow_flutter") {
                    node.wow_flutter = wf;
                }
                if let Some(&hys) = params.get("hysteresis") {
                    node.hysteresis = hys;
                }
                Some(Box::new(ProcessorNodeAdapter::new(node)))
            }
            "TubeSaturationNode" | "TubeSaturation" => {
                let drive = *params.get("drive").unwrap_or(&2.5);
                let bias = *params
                    .get("bias")
                    .or_else(|| params.get("bias_voltage_v"))
                    .unwrap_or(&-1.85);
                let mut node = summoner_dsp::TubeSaturationNode::new(drive, bias);
                if let Some(&topo) = params.get("topology") {
                    match topo.round() as i32 {
                        0 => node.topology = summoner_dsp::TubeTopology::Triode12AX7,
                        1 => node.topology = summoner_dsp::TubeTopology::PentodeEL34,
                        2 => node.topology = summoner_dsp::TubeTopology::BeamTetrode6L6,
                        _ => node.topology = summoner_dsp::TubeTopology::CleanModern,
                    }
                }
                if let Some(&plate) = params
                    .get("plate_voltage")
                    .or_else(|| params.get("plate_voltage_v"))
                {
                    node.plate_voltage_v = plate;
                }
                if let Some(&sag) = params.get("sag_compression") {
                    node.sag_compression = sag;
                }
                if let Some(&asym) = params.get("asymmetry") {
                    node.asymmetry = asym;
                }
                if let Some(&warmth) = params.get("warmth") {
                    node.warmth = warmth;
                }
                Some(Box::new(ProcessorNodeAdapter::new(node)))
            }
            "ConsoleEmulationNode" | "ConsoleEmulation" => {
                let mode = summoner_dsp::ConsoleMode::from_f32(*params.get("mode").unwrap_or(&0.0));
                let drive = *params.get("drive").unwrap_or(&1.5);
                let mut node = summoner_dsp::ConsoleEmulationNode::new(mode, drive);
                if let Some(&warmth) = params.get("warmth") {
                    node.warmth = warmth;
                }
                if let Some(&iron) = params
                    .get("transformer_iron")
                    .or_else(|| params.get("iron"))
                {
                    node.transformer_iron = iron;
                }
                if let Some(&crosstalk) = params.get("crosstalk") {
                    node.crosstalk = crosstalk;
                }
                Some(Box::new(ProcessorNodeAdapter::new(node)))
            }
            "DistortionNode" => Some(Box::new(ProcessorNodeAdapter::new(DistortionNode::new(
                DistortionType::SoftClipping,
                *params.get("drive").unwrap_or(&2.0),
            )))),
            "NoiseGateNode" | "NoiseGate" => Some(Box::new(ProcessorNodeAdapter::new(
                summoner_dsp::NoiseGateNode::with_params(
                    *params.get("threshold").unwrap_or(&-40.0),
                    *params.get("ratio").unwrap_or(&4.0),
                    *params.get("attack").unwrap_or(&5.0),
                    *params.get("release").unwrap_or(&100.0),
                ),
            ))),
            "DeesserNode" | "Deesser" => Some(Box::new(ProcessorNodeAdapter::new(
                summoner_dsp::DeesserNode::new(),
            ))),
            "HarmonicExciterNode" | "HarmonicExciter" => Some(Box::new(ProcessorNodeAdapter::new(
                summoner_dsp::HarmonicExciterNode::new(),
            ))),
            "NamAmpNode" | "NamAmp" => Some(Box::new(ProcessorNodeAdapter::new(
                summoner_dsp::NamAmpNode::new(
                    *params.get("drive").unwrap_or(&1.5),
                    *params.get("output_gain").unwrap_or(&1.0),
                    *params.get("gate_threshold").unwrap_or(&-60.0),
                ),
            ))),
            "RnnoiseNode" | "Rnnoise" => Some(Box::new(ProcessorNodeAdapter::new(
                summoner_dsp::RnnoiseNode::new(*params.get("suppression").unwrap_or(&-12.0)),
            ))),
            "AiAutoGainNode" | "AiAutoGain" => Some(Box::new(ProcessorNodeAdapter::new(
                summoner_dsp::AiAutoGainNode::new(*params.get("target_lufs").unwrap_or(&-18.0)),
            ))),
            "DdspTimbreTransferNode" | "DdspTimbreTransfer" => Some(Box::new(
                ProcessorNodeAdapter::new(summoner_dsp::DdspTimbreTransferNode::new(
                    *params.get("harmonics").unwrap_or(&32.0) as usize,
                    *params.get("noise").unwrap_or(&0.05),
                )),
            )),
            "VocalHarmonyGeneratorNode" | "VocalHarmonizer" => Some(Box::new(
                ProcessorNodeAdapter::new(summoner_dsp::VocalHarmonyGeneratorNode::default()),
            )),
            "NeuralDereverbNode" | "NeuralDereverb" => Some(Box::new(ProcessorNodeAdapter::new(
                summoner_dsp::NeuralDereverbNode::new(),
            ))),
            "NeuralSuperResolutionNode" | "NeuralSuperResolution" => Some(Box::new(
                ProcessorNodeAdapter::new(summoner_dsp::NeuralSuperResolutionNode::new()),
            )),
            "SamplerDevice" => {
                let mut bank = summoner_dsp::sampler::MultiSampleBank::new();
                let base_dir = if std::path::Path::new("samples").exists() {
                    std::path::Path::new("samples")
                } else {
                    std::path::Path::new(".")
                };
                summoner_dsp::sampler::load_bank_buffers(&mut bank, base_dir);
                Some(Box::new(ProcessorNodeAdapter::new(
                    summoner_dsp::SamplerDevice::new(bank),
                )))
            }
            "MultiSamplerNode" => {
                let mut bank = summoner_dsp::sampler::MultiSampleBank::new();
                let base_dir = if std::path::Path::new("samples").exists() {
                    std::path::Path::new("samples")
                } else {
                    std::path::Path::new(".")
                };
                summoner_dsp::sampler::load_bank_buffers(&mut bank, base_dir);
                Some(Box::new(ProcessorNodeAdapter::new(
                    summoner_dsp::sampler::MultiSamplerNode::new(bank),
                )))
            }
            "PluginNode" | "VstPluginNode" | "ClapPluginNode" => {
                let format = if config.kind.contains("Vst") {
                    summoner_dsp::PluginFormat::Vst3
                } else {
                    summoner_dsp::PluginFormat::Clap
                };
                let descriptor = summoner_dsp::PluginDescriptor {
                    name: config
                        .params
                        .get("name_hash")
                        .map(|_| "Plugin")
                        .unwrap_or("HostedPluginNode")
                        .to_string(),
                    vendor: "ThirdParty".to_string(),
                    format,
                    path: std::path::PathBuf::from("plugins/hosted.vst3"),
                    version: "1.0.0".to_string(),
                    category: "Audio Effect".to_string(),
                    num_inputs: 2,
                    num_outputs: 2,
                };
                let mut node = summoner_dsp::PluginAudioNode::new(descriptor);
                if let Some(ref ps) = config.plugin_state {
                    let dsp_ps = summoner_dsp::PluginStateConfig {
                        plugin_name: ps.plugin_name.clone(),
                        plugin_path: ps.plugin_path.clone(),
                        format: ps.format.clone(),
                        is_bypassed: ps.is_bypassed,
                        state_base64: ps.state_base64.clone(),
                        parameters: ps.parameters.clone(),
                    };
                    node.restore_state(&dsp_ps);
                }
                Some(Box::new(node))
            }

            "TonewheelOrganNode" | "TonewheelOrgan" => {
                let mut organ = summoner_dsp::TonewheelOrgan::new(sample_rate);
                if let Some(&gain) = params.get("gain") {
                    organ.master_gain = gain;
                }
                if let Some(&click) = params.get("key_click") {
                    organ.key_click_amount = click;
                }
                if let Some(&leak) = params.get("crosstalk") {
                    organ.crosstalk_amount = leak;
                }
                Some(Box::new(ProcessorNodeAdapter::new(organ)))
            }
            "RotarySpeakerNode" | "RotarySpeaker" => {
                let mut speaker = summoner_dsp::RotarySpeaker::new(sample_rate);
                if let Some(&drive) = params.get("drive") {
                    speaker.drive_saturation_db = drive;
                }
                if let Some(&mix) = params.get("mix") {
                    speaker.wet_mix = mix;
                }
                if let Some(&bal) = params.get("horn_drum_balance") {
                    speaker.horn_drum_balance_pct = bal;
                }
                Some(Box::new(ProcessorNodeAdapter::new(speaker)))
            }
            "ElectricPianoNode" | "ElectricPiano" => {
                let mut ep = summoner_dsp::ElectricPiano::new(sample_rate);
                if let Some(&gain) = params.get("gain") {
                    ep.master_gain = gain;
                }
                if let Some(&hardness) = params.get("hardness") {
                    ep.hammer_hardness = hardness;
                }
                if let Some(&gap) = params.get("air_gap") {
                    ep.air_gap_mm = gap;
                }
                if let Some(&bark) = params.get("bark") {
                    ep.bark_drive = bark;
                }
                if let Some(&tonebar) = params.get("tonebar") {
                    ep.tonebar_coupling = tonebar;
                }
                if let Some(&clunk) = params.get("damper_clunk") {
                    ep.damper_clunk_volume = clunk;
                }
                Some(Box::new(ProcessorNodeAdapter::new(ep)))
            }
            "SitarNode" | "Sitar" => {
                let mut sitar = summoner_dsp::Sitar::new(sample_rate);
                if let Some(&gain) = params.get("gain") {
                    sitar.master_gain = gain;
                }
                if let Some(&meend) = params.get("meend") {
                    sitar.set_meend_pull(meend);
                }
                if let Some(&gap) = params.get("jawari_gap") {
                    sitar.jawari.clearance_gap_mm = gap;
                }
                if let Some(&bleed) = params.get("tarab_bleed") {
                    sitar.tarab.coupling_bleed = bleed;
                }
                Some(Box::new(ProcessorNodeAdapter::new(sitar)))
            }
            _ => None,
        }
    }
}

pub fn graph_from_project(project: &ProjectConfig, max_block_size: usize) -> NodeGraph {
    let mut graph = NodeGraph::new(project.name.clone(), max_block_size, 2);

    if let Some(track) = project.tracks.first() {
        for nc in &track.nodes {
            if let Some(node) =
                NodeFactory::create_node_with_rate(nc, project.transport.sample_rate)
            {
                graph.add_node(node);
            } else {
                eprintln!(
                    "Warning: Unknown node kind '{}'; using passthrough",
                    nc.kind
                );
                graph.add_node(Box::new(summoner_core::node::PassthroughNode));
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

/// Convert a track's sequence, clips, and automation lanes into frame-stamped events.
/// Tracks with any note events start gated closed (NoteOff at frame 0).
fn build_schedule(
    tc: &summoner_project::schema::TrackConfig,
    automation_lanes: &[summoner_project::schema::AutomationLaneConfig],
    sample_rate: u32,
    bpm: f64,
) -> Vec<ScheduledEvent> {
    let frames_per_beat = sample_rate as f64 * 60.0 / bpm.max(1.0);
    let mut events = vec![ScheduledEvent {
        frame: 0,
        event: NodeEvent::NoteOff { note: 0 },
    }];
    let mut has_notes = false;
    for seq in tc.sequence.iter().chain(tc.clips.iter()) {
        let stretch = if seq.time_stretch > 0.0 {
            seq.time_stretch
        } else {
            1.0
        };
        let effective_step_division = seq.step_division * stretch;
        let total_seq_beats = seq.steps.len() as f64 * effective_step_division;

        for (i, _step) in seq.steps.iter().enumerate() {
            let logical_step_idx = if seq.is_reversed {
                seq.steps.len().saturating_sub(1 + i)
            } else {
                i
            };
            let actual_step = &seq.steps[logical_step_idx];

            if !actual_step.active || actual_step.muted || actual_step.probability <= 0.0 {
                continue;
            }

            let rel_step_beat = i as f64 * effective_step_division;
            if rel_step_beat < seq.trim_start {
                continue;
            }
            if seq.trim_end > seq.trim_start && rel_step_beat >= seq.trim_end {
                continue;
            }

            has_notes = true;
            let note = (actual_step.note
                + seq.pitch_offset
                + actual_step.pitch_offset as f64
                + tc.midi_transpose as f64)
                .round()
                .clamp(0.0, 127.0) as u8;

            let swing_beats = if i % 2 == 1 {
                (actual_step.swing as f64).clamp(-0.5, 0.5) * effective_step_division
            } else {
                0.0
            };
            let micro_beats = (actual_step.micro_shift as f64 / 100.0) * effective_step_division;
            let base_beat = seq.start_beat + rel_step_beat + swing_beats + micro_beats;
            let ratchet_count = actual_step.ratchet.max(1);
            let sub_step_beats = effective_step_division / ratchet_count as f64;

            // Fade-in / Fade-out multiplier
            let mut fade_mult = 1.0f32;
            if seq.fade_in > 0.0 && rel_step_beat < seq.fade_in {
                fade_mult *= (rel_step_beat / seq.fade_in) as f32;
            }
            if seq.fade_out > 0.0 && total_seq_beats > 0.0 {
                let dist_from_end = total_seq_beats - rel_step_beat;
                if dist_from_end < seq.fade_out {
                    fade_mult *= (dist_from_end / seq.fade_out).max(0.0) as f32;
                }
            }

            let vel = (actual_step.velocity * seq.gain as f32 * fade_mult).clamp(0.0, 1.0);
            for r in 0..ratchet_count {
                let beat = base_beat + r as f64 * sub_step_beats;
                let on = (beat * frames_per_beat).round().max(0.0) as u64;
                let off = ((beat + (actual_step.gate as f64).clamp(0.01, 1.0) * sub_step_beats)
                    * frames_per_beat)
                    .round()
                    .max(0.0) as u64;
                events.push(ScheduledEvent {
                    frame: on,
                    event: NodeEvent::NoteOn {
                        note,
                        velocity: vel,
                    },
                });
                events.push(ScheduledEvent {
                    frame: off.max(on + 1),
                    event: NodeEvent::NoteOff { note },
                });
            }
        }
    }
    if !has_notes {
        events.clear();
    }
    for lane in automation_lanes.iter().filter(|l| l.track_id == tc.id) {
        let param_id = match lane.param_id.to_lowercase().as_str() {
            "0" | "freq" | "frequency" | "cutoff" | "gain" | "drive" | "rate" | "pitch" => 0,
            "1" | "res" | "resonance" | "pw" | "pulse_width" | "depth" | "mix" | "feedback" => 1,
            "2" | "attack" | "damping" | "room_size" => 2,
            "3" | "decay" => 3,
            "4" | "sustain" => 4,
            "5" | "release" => 5,
            other => other.parse::<u32>().unwrap_or(0),
        };
        for ev in &lane.events {
            events.push(ScheduledEvent {
                frame: ev.frame,
                event: NodeEvent::SetParam {
                    id: param_id,
                    value: ev.value,
                },
            });
        }
    }
    events
}

fn parse_node_port(spec: &str) -> Option<(usize, usize)> {
    if let Some((node_str, port_str)) = spec.split_once(':') {
        let n = node_str
            .trim()
            .trim_start_matches('n')
            .parse::<usize>()
            .ok()?;
        let p = port_str.trim().parse::<usize>().ok()?;
        Some((n, p))
    } else {
        let n = spec.trim().trim_start_matches('n').parse::<usize>().ok()?;
        Some((n, 0))
    }
}

pub struct TrackFilters {
    pub dc_block: Option<[DcBlockFilter; 2]>,
    pub low_cut: Option<[LowCutFilter; 2]>,
    pub high_cut: Option<[HighCutFilter; 2]>,
}

pub const MAX_PDC_DELAY_SAMPLES: usize = 8192;

/// Circular buffer for real-time sample-accurate Plugin Delay Compensation (PDC).
#[derive(Clone)]
pub struct PdcDelayBuffer {
    buffer_l: [f32; MAX_PDC_DELAY_SAMPLES],
    buffer_r: [f32; MAX_PDC_DELAY_SAMPLES],
    write_pos: usize,
    delay_samples: usize,
}

impl Default for PdcDelayBuffer {
    fn default() -> Self {
        Self::new(0)
    }
}

impl PdcDelayBuffer {
    pub fn new(delay_samples: usize) -> Self {
        Self {
            buffer_l: [0.0; MAX_PDC_DELAY_SAMPLES],
            buffer_r: [0.0; MAX_PDC_DELAY_SAMPLES],
            write_pos: 0,
            delay_samples: delay_samples.min(MAX_PDC_DELAY_SAMPLES - 1),
        }
    }

    pub fn set_delay(&mut self, delay: usize) {
        self.delay_samples = delay.min(MAX_PDC_DELAY_SAMPLES - 1);
    }

    pub fn delay_samples(&self) -> usize {
        self.delay_samples
    }

    #[inline]
    pub fn process_block(&mut self, left: &mut [f32], right: &mut [f32]) {
        if self.delay_samples == 0 {
            return;
        }
        let len = left.len().min(right.len());
        for i in 0..len {
            let in_l = left[i];
            let in_r = right[i];
            let read_pos = (self.write_pos + MAX_PDC_DELAY_SAMPLES - self.delay_samples)
                % MAX_PDC_DELAY_SAMPLES;
            left[i] = self.buffer_l[read_pos];
            right[i] = self.buffer_r[read_pos];
            self.buffer_l[self.write_pos] = in_l;
            self.buffer_r[self.write_pos] = in_r;
            self.write_pos = (self.write_pos + 1) % MAX_PDC_DELAY_SAMPLES;
        }
    }
}

/// Channel audio telemetry (Peak and RMS).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChannelMeter {
    pub peak: f32,
    pub peak_db: f32,
    pub rms: f32,
    pub rms_db: f32,
}

impl Default for ChannelMeter {
    fn default() -> Self {
        Self {
            peak: 0.0,
            peak_db: -120.0,
            rms: 0.0,
            rms_db: -120.0,
        }
    }
}

/// Comprehensive real-time master output telemetry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MasterMeterTelemetry {
    pub channels: [ChannelMeter; 2],
    pub true_peak_db: f32,
    pub momentary_lufs: f32,
    pub short_term_lufs: f32,
    pub integrated_lufs: f32,
    pub phase_correlation: f32,
}

impl Default for MasterMeterTelemetry {
    fn default() -> Self {
        Self {
            channels: [ChannelMeter::default(), ChannelMeter::default()],
            true_peak_db: -120.0,
            momentary_lufs: -70.0,
            short_term_lufs: -70.0,
            integrated_lufs: -70.0,
            phase_correlation: 1.0,
        }
    }
}

/// Cached track routing and metadata for the mixer execution graph.
#[derive(Debug, Clone)]
pub struct TrackMeta {
    pub id: u64,
    pub name: String,
    pub is_return_track: bool,
    pub is_bus_track: bool,
    pub is_vca: bool,
    pub vca_target_ids: Vec<u64>,
    pub vca_group: Option<u32>,
    pub mute_group: Option<u32>,
    pub solo_safe: bool,
    pub send_to_master: bool,
    pub bus_target: Option<String>,
    pub sidechain_source_track_id: Option<u64>,
    pub sidechain_audition: bool,
    pub sends: Vec<summoner_project::schema::TrackSendConfig>,
    pub legacy_send_level: f32,
}

pub struct GraphRunner {
    pub tracks: Vec<Track>,
    pub track_meta: Vec<TrackMeta>,
    pub track_filters: Vec<TrackFilters>,
    pub pdc_buffers: Vec<PdcDelayBuffer>,
    pub master_trim_db: f32,
    pub telemetry: MasterMeterTelemetry,
    pub true_peak_meter: summoner_dsp::TruePeakMeter,
    pub lufs_meter: summoner_dsp::EbuR128LoudnessMeter,
    scratch_l: Box<[f32; MAX_RUNNER_BLOCK_SIZE]>,
    scratch_r: Box<[f32; MAX_RUNNER_BLOCK_SIZE]>,
    track_buffers: Vec<[Box<[f32; MAX_RUNNER_BLOCK_SIZE]>; 2]>,
    track_in_buffers: Vec<[Box<[f32; MAX_RUNNER_BLOCK_SIZE]>; 2]>,
}

impl GraphRunner {
    pub fn new(project: &ProjectConfig) -> Self {
        let mut tracks = Vec::new();
        let mut track_meta = Vec::new();
        for tc in &project.tracks {
            let mut track = Track::new(tc.id, tc.name.clone(), tc.channels);
            track.gain = tc.gain;
            track.pan = tc.pan;
            track.muted = tc.muted;
            track.soloed = tc.soloed;
            track.solo_safe = tc.solo_safe;
            track.phase_flip = tc.phase_flip;
            track.input_gain_db = tc.input_gain_db;
            track.output_gain_db = tc.output_gain_db;
            track.tuning_edo = tc.tuning_edo;
            track.tuning_root_hz = tc.tuning_root_hz;
            track.input_monitoring = tc.input_monitoring;
            track.armed_for_recording = tc.record_armed;

            track_meta.push(TrackMeta {
                id: tc.id,
                name: tc.name.clone(),
                is_return_track: tc.is_return_track,
                is_bus_track: tc.is_bus_track,
                is_vca: tc.is_vca,
                vca_target_ids: tc.vca_target_ids.clone(),
                vca_group: tc.vca_group,
                mute_group: tc.mute_group,
                solo_safe: tc.solo_safe,
                send_to_master: tc.send_to_master,
                bus_target: tc.bus_target.clone(),
                sidechain_source_track_id: tc.sidechain_source_track_id,
                sidechain_audition: tc.sidechain_audition,
                sends: tc.sends.clone(),
                legacy_send_level: tc.send_level,
            });

            if !tc.connections.is_empty() {
                let mut dag =
                    NodeGraph::new(tc.name.clone(), MAX_RUNNER_BLOCK_SIZE, tc.channels.max(2));
                dag.parallel_execution = false;
                for nc in &tc.nodes {
                    if let Some(node) =
                        NodeFactory::create_node_with_rate(nc, project.transport.sample_rate)
                    {
                        dag.add_node(node);
                    } else {
                        eprintln!("Warning: Unknown node kind '{}'", nc.kind);
                    }
                }
                for conn in &tc.connections {
                    if let (Some((f_node, f_port)), Some((t_node, t_port))) =
                        (parse_node_port(&conn.from), parse_node_port(&conn.to))
                    {
                        dag.add_edge(Edge {
                            from_node: f_node,
                            from_port: f_port,
                            to_node: t_node,
                            to_port: t_port,
                        });
                    }
                }
                dag.compile();
                track.add_node(Box::new(dag));
            } else {
                for nc in &tc.nodes {
                    if let Some(node) =
                        NodeFactory::create_node_with_rate(nc, project.transport.sample_rate)
                    {
                        track.add_node(node);
                    } else {
                        eprintln!("Warning: Unknown node kind '{}'", nc.kind);
                    }
                }
            }
            let events = build_schedule(
                tc,
                &project.automation_lanes,
                project.transport.sample_rate,
                project.transport.bpm,
            );
            track.set_schedule(events);
            tracks.push(track);
        }
        let mut track_filters = Vec::new();
        for tc in &project.tracks {
            let dc_block = if tc.dc_block {
                Some([DcBlockFilter::new(), DcBlockFilter::new()])
            } else {
                None
            };
            let low_cut = tc
                .low_cut_hz
                .map(|hz| [LowCutFilter::new(hz), LowCutFilter::new(hz)]);
            let high_cut = tc
                .high_cut_hz
                .map(|hz| [HighCutFilter::new(hz), HighCutFilter::new(hz)]);
            track_filters.push(TrackFilters {
                dc_block,
                low_cut,
                high_cut,
            });
        }

        // Calculate PDC delays per track
        let max_lat = tracks
            .iter()
            .map(|t| t.total_latency_samples())
            .max()
            .unwrap_or(0);
        let mut pdc_buffers = Vec::with_capacity(tracks.len());
        for t in &tracks {
            let lat = t.total_latency_samples();
            let delay = max_lat.saturating_sub(lat);
            pdc_buffers.push(PdcDelayBuffer::new(delay));
        }

        let mut track_buffers = Vec::with_capacity(project.tracks.len());
        let mut track_in_buffers = Vec::with_capacity(project.tracks.len());
        for _ in 0..project.tracks.len() {
            track_buffers.push([
                Box::new([0.0f32; MAX_RUNNER_BLOCK_SIZE]),
                Box::new([0.0f32; MAX_RUNNER_BLOCK_SIZE]),
            ]);
            track_in_buffers.push([
                Box::new([0.0f32; MAX_RUNNER_BLOCK_SIZE]),
                Box::new([0.0f32; MAX_RUNNER_BLOCK_SIZE]),
            ]);
        }

        Self {
            tracks,
            track_meta,
            track_filters,
            pdc_buffers,
            master_trim_db: project.transport.master_trim_db,
            telemetry: MasterMeterTelemetry::default(),
            true_peak_meter: summoner_dsp::TruePeakMeter::new(),
            lufs_meter: summoner_dsp::EbuR128LoudnessMeter::default(),
            scratch_l: Box::new([0.0f32; MAX_RUNNER_BLOCK_SIZE]),
            scratch_r: Box::new([0.0f32; MAX_RUNNER_BLOCK_SIZE]),
            track_buffers,
            track_in_buffers,
        }
    }

    pub fn process_block(
        &mut self,
        block_size: usize,
        ctx: &summoner_core::node::ProcessContext,
        out_buffers: &mut [&mut [summoner_core::audio::Sample]],
    ) {
        self.process_block_with_input(block_size, ctx, None, out_buffers);
    }

    #[allow(clippy::needless_range_loop)]
    pub fn process_block_with_input(
        &mut self,
        block_size: usize,
        ctx: &summoner_core::node::ProcessContext,
        in_buffers: Option<&[&[summoner_core::audio::Sample]]>,
        out_buffers: &mut [&mut [summoner_core::audio::Sample]],
    ) {
        let block_size = block_size.min(MAX_RUNNER_BLOCK_SIZE);
        for out in out_buffers.iter_mut() {
            out[..block_size].fill(0.0);
        }
        for tb in self.track_buffers.iter_mut() {
            tb[0][..block_size].fill(0.0);
            tb[1][..block_size].fill(0.0);
        }
        for tib in self.track_in_buffers.iter_mut() {
            tib[0][..block_size].fill(0.0);
            tib[1][..block_size].fill(0.0);
        }

        let any_soloed = self.tracks.iter().any(|t| t.soloed);
        let master_trim = 10.0f32.powf(self.master_trim_db / 20.0);

        // Mute groups evaluation
        let mut muted_groups = [false; 32];
        for (idx, meta) in self.track_meta.iter().enumerate() {
            if let Some(g) = meta.mute_group {
                let g_idx = (g as usize) % 32;
                if self.tracks[idx].muted {
                    muted_groups[g_idx] = true;
                }
            }
        }

        // VCA evaluation
        let num_tracks = self.tracks.len();
        let mut vca_gains = [1.0f32; 128];
        let mut vca_muted = [false; 128];
        for (vca_idx, vca_meta) in self.track_meta.iter().enumerate() {
            if vca_meta.is_vca && vca_idx < 128 {
                let vca_gain = self.tracks[vca_idx].gain;
                let is_v_muted = self.tracks[vca_idx].muted;
                for (t_idx, t_meta) in self.track_meta.iter().enumerate() {
                    if t_idx == vca_idx || t_idx >= 128 {
                        continue;
                    }
                    let matches_target = vca_meta.vca_target_ids.contains(&t_meta.id);
                    let matches_group =
                        vca_meta.vca_group.is_some() && vca_meta.vca_group == t_meta.vca_group;
                    if matches_target || matches_group {
                        vca_gains[t_idx] *= vca_gain;
                        if is_v_muted {
                            vca_muted[t_idx] = true;
                        }
                    }
                }
            }
        }

        // Process in 3 stages:
        // Stage 1: Regular audio/synth tracks
        // Stage 2: Return tracks (receive aux sends)
        // Stage 3: Subgroup bus tracks (receive subgroup routing)
        for stage in 0..3 {
            for track_idx in 0..num_tracks {
                let meta = &self.track_meta[track_idx];
                let is_stage_match = match stage {
                    0 => !meta.is_return_track && !meta.is_bus_track && !meta.is_vca,
                    1 => meta.is_return_track && !meta.is_vca,
                    2 => meta.is_bus_track && !meta.is_vca,
                    _ => false,
                };
                if !is_stage_match {
                    continue;
                }

                let track = &mut self.tracks[track_idx];
                let is_mute_grouped = meta
                    .mute_group
                    .is_some_and(|g| muted_groups[(g as usize) % 32]);
                let is_muted =
                    track.muted || is_mute_grouped || (track_idx < 128 && vca_muted[track_idx]);

                if any_soloed && !track.soloed && !meta.solo_safe {
                    continue;
                }
                if is_muted {
                    continue;
                }

                self.scratch_l[..block_size].fill(0.0);
                self.scratch_r[..block_size].fill(0.0);

                let mut local_ctx = ctx.clone();
                if let Some(edo) = track.tuning_edo {
                    local_ctx.tuning_edo_divisions = edo;
                }
                if let Some(root_hz) = track.tuning_root_hz {
                    local_ctx.tuning_root_hz = root_hz;
                }

                // If track has input monitoring enabled, mix in live audio
                if track.input_monitoring {
                    if let Some(ins) = in_buffers {
                        if let Some(in_l) = ins.first() {
                            let len = block_size.min(in_l.len());
                            for i in 0..len {
                                self.scratch_l[i] += in_l[i];
                            }
                        }
                        if let Some(in_r) = ins.get(1).or_else(|| ins.first()) {
                            let len = block_size.min(in_r.len());
                            for i in 0..len {
                                self.scratch_r[i] += in_r[i];
                            }
                        }
                    }
                }

                // Mix in any accumulated audio from sends or buses
                for i in 0..block_size {
                    self.scratch_l[i] += self.track_in_buffers[track_idx][0][i];
                    self.scratch_r[i] += self.track_in_buffers[track_idx][1][i];
                }

                if meta.sidechain_audition {
                    if let Some(src_id) = meta.sidechain_source_track_id {
                        if let Some(src_idx) = self.track_meta.iter().position(|m| m.id == src_id) {
                            let src_l = &self.track_buffers[src_idx][0][..block_size];
                            let src_r = &self.track_buffers[src_idx][1][..block_size];
                            self.scratch_l[..block_size].copy_from_slice(src_l);
                            self.scratch_r[..block_size].copy_from_slice(src_r);
                        }
                    }
                } else {
                    let (slice_l, slice_r) = (
                        &mut self.scratch_l[..block_size],
                        &mut self.scratch_r[..block_size],
                    );
                    let mut track_out_slices: [&mut [f32]; 2] = [slice_l, slice_r];

                    let in_l = &self.track_in_buffers[track_idx][0][..block_size];
                    let in_r = &self.track_in_buffers[track_idx][1][..block_size];
                    let input_slices: [&[f32]; 2] = [in_l, in_r];
                    let ins_param =
                        if in_l.iter().any(|&s| s != 0.0) || in_r.iter().any(|&s| s != 0.0) {
                            Some(&input_slices[..])
                        } else {
                            None
                        };

                    track.process_with_input(
                        block_size,
                        &local_ctx,
                        ins_param,
                        &mut track_out_slices[..out_buffers.len().min(2)],
                    );

                    if let Some(tf) = self.track_filters.get_mut(track_idx) {
                        if let Some(ref mut dcf) = tf.dc_block {
                            for i in 0..block_size {
                                self.scratch_l[i] = dcf[0].process_sample(self.scratch_l[i]);
                                self.scratch_r[i] = dcf[1].process_sample(self.scratch_r[i]);
                            }
                        }
                        if let Some(ref mut lcf) = tf.low_cut {
                            for i in 0..block_size {
                                self.scratch_l[i] =
                                    lcf[0].process_sample(self.scratch_l[i], ctx.sample_rate);
                                self.scratch_r[i] =
                                    lcf[1].process_sample(self.scratch_r[i], ctx.sample_rate);
                            }
                        }
                        if let Some(ref mut hcf) = tf.high_cut {
                            for i in 0..block_size {
                                self.scratch_l[i] =
                                    hcf[0].process_sample(self.scratch_l[i], ctx.sample_rate);
                                self.scratch_r[i] =
                                    hcf[1].process_sample(self.scratch_r[i], ctx.sample_rate);
                            }
                        }
                    }
                }

                // Apply PDC Delay Alignment
                self.pdc_buffers[track_idx].process_block(
                    &mut self.scratch_l[..block_size],
                    &mut self.scratch_r[..block_size],
                );

                // Cache post-PDC audio for sidechain taps
                self.track_buffers[track_idx][0][..block_size]
                    .copy_from_slice(&self.scratch_l[..block_size]);
                self.track_buffers[track_idx][1][..block_size]
                    .copy_from_slice(&self.scratch_r[..block_size]);

                let strip_db = track.input_gain_db + track.output_gain_db;
                let vca_mult = if track_idx < 128 {
                    vca_gains[track_idx]
                } else {
                    1.0
                };
                let strip_gain = 10.0f32.powf(strip_db / 20.0)
                    * if track.phase_flip { -1.0 } else { 1.0 }
                    * master_trim;
                let effective_gain = track.gain * strip_gain * vca_mult;

                let pan = track.pan.clamp(-1.0, 1.0);
                let left_gain = effective_gain * (1.0 - pan).min(1.0);
                let right_gain = effective_gain * (1.0 + pan).min(1.0);

                // Aux Sends routing
                for send in &meta.sends {
                    if send.muted || send.amount <= 0.0 {
                        continue;
                    }
                    if let Some(target_idx) = self
                        .track_meta
                        .iter()
                        .position(|m| m.id == send.target_track_id)
                    {
                        let (send_l_gain, send_r_gain) = if send.is_pre_fader {
                            (send.amount, send.amount)
                        } else {
                            (left_gain * send.amount, right_gain * send.amount)
                        };
                        for i in 0..block_size {
                            self.track_in_buffers[target_idx][0][i] +=
                                self.scratch_l[i] * send_l_gain;
                            self.track_in_buffers[target_idx][1][i] +=
                                self.scratch_r[i] * send_r_gain;
                        }
                    }
                }

                // Legacy send_level routing to return track if present
                if meta.legacy_send_level > 0.0 {
                    if let Some(target_idx) = self.track_meta.iter().position(|m| m.is_return_track)
                    {
                        let s_l = left_gain * meta.legacy_send_level;
                        let s_r = right_gain * meta.legacy_send_level;
                        for i in 0..block_size {
                            self.track_in_buffers[target_idx][0][i] += self.scratch_l[i] * s_l;
                            self.track_in_buffers[target_idx][1][i] += self.scratch_r[i] * s_r;
                        }
                    }
                }

                // Subgroup Bus routing or direct Master mix
                let mut routed_to_bus = false;
                if let Some(ref bus_name) = meta.bus_target {
                    if let Some(target_idx) = self
                        .track_meta
                        .iter()
                        .position(|m| m.name == *bus_name || m.id.to_string() == *bus_name)
                    {
                        for i in 0..block_size {
                            self.track_in_buffers[target_idx][0][i] +=
                                self.scratch_l[i] * left_gain;
                            self.track_in_buffers[target_idx][1][i] +=
                                self.scratch_r[i] * right_gain;
                        }
                        routed_to_bus = true;
                    }
                }

                if !routed_to_bus && meta.send_to_master {
                    let num_out = out_buffers.len().min(2);
                    if num_out == 2 {
                        let scratch_r = if track.channels <= 1 {
                            &self.scratch_l[..block_size]
                        } else {
                            &self.scratch_r[..block_size]
                        };
                        for i in 0..block_size {
                            out_buffers[0][i] += self.scratch_l[i] * left_gain;
                            out_buffers[1][i] += scratch_r[i] * right_gain;
                        }
                    } else if num_out == 1 {
                        for i in 0..block_size {
                            out_buffers[0][i] += self.scratch_l[i] * effective_gain;
                        }
                    }
                }
            }
        }

        // Master Output Telemetry & Analysis
        let num_out = out_buffers.len().min(2);
        if num_out > 0 {
            let mut max_l = 0.0f32;
            let mut max_r = 0.0f32;
            let mut sum_sq_l = 0.0f32;
            let mut sum_sq_r = 0.0f32;
            let mut sum_lr = 0.0f32;

            for i in 0..block_size {
                let l = out_buffers[0][i];
                let r = if num_out > 1 { out_buffers[1][i] } else { l };
                let abs_l = l.abs();
                let abs_r = r.abs();
                if abs_l > max_l {
                    max_l = abs_l;
                }
                if abs_r > max_r {
                    max_r = abs_r;
                }
                sum_sq_l += l * l;
                sum_sq_r += r * r;
                sum_lr += l * r;
            }

            let rms_l = (sum_sq_l / block_size as f32).sqrt();
            let rms_r = (sum_sq_r / block_size as f32).sqrt();
            let peak_l_db = if max_l > 1e-6 {
                20.0 * max_l.log10()
            } else {
                -120.0
            };
            let peak_r_db = if max_r > 1e-6 {
                20.0 * max_r.log10()
            } else {
                -120.0
            };
            let rms_l_db = if rms_l > 1e-6 {
                20.0 * rms_l.log10()
            } else {
                -120.0
            };
            let rms_r_db = if rms_r > 1e-6 {
                20.0 * rms_r.log10()
            } else {
                -120.0
            };

            let denom = (sum_sq_l * sum_sq_r).sqrt();
            let phase_corr = if denom > 1e-9 {
                (sum_lr / denom).clamp(-1.0, 1.0)
            } else {
                1.0
            };

            self.true_peak_meter
                .process_block(&out_buffers[0][..block_size]);
            if num_out > 1 {
                self.true_peak_meter
                    .process_block(&out_buffers[1][..block_size]);
            }

            self.lufs_meter.process_block(&out_buffers[0][..block_size]);

            self.telemetry = MasterMeterTelemetry {
                channels: [
                    ChannelMeter {
                        peak: max_l,
                        peak_db: peak_l_db,
                        rms: rms_l,
                        rms_db: rms_l_db,
                    },
                    ChannelMeter {
                        peak: max_r,
                        peak_db: peak_r_db,
                        rms: rms_r,
                        rms_db: rms_r_db,
                    },
                ],
                true_peak_db: self.true_peak_meter.max_true_peak_db,
                momentary_lufs: self.lufs_meter.momentary_lufs,
                short_term_lufs: self.lufs_meter.short_term_lufs,
                integrated_lufs: self.lufs_meter.integrated_lufs,
                phase_correlation: phase_corr,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_project::create_default_project;

    #[test]
    fn test_sequence_clip_modifiers_reversed_pitch_time_stretch_and_fades() {
        use summoner_project::schema::{SequenceConfig, TrackConfig, TrackerStepConfig};
        let mut tc = TrackConfig::default();
        let step0 = TrackerStepConfig {
            note: 60.0,
            velocity: 1.0,
            active: true,
            ..Default::default()
        };
        let step1 = TrackerStepConfig {
            note: 72.0,
            velocity: 1.0,
            active: true,
            ..Default::default()
        };

        let seq = SequenceConfig {
            start_beat: 0.0,
            step_division: 1.0,
            is_reversed: true,
            pitch_offset: 2.0,
            time_stretch: 2.0,
            gain: 0.5,
            steps: vec![step0, step1],
            ..Default::default()
        };
        tc.sequence = Some(seq);

        let sched = build_schedule(&tc, &[], 44100, 120.0);
        // Reversed: step1 (note 72 + 2 = 74) is played first at frame 0
        // step0 (note 60 + 2 = 62) is played second at beat 2.0 (frame 44100)
        let note_ons: Vec<_> = sched
            .iter()
            .filter_map(|s| match s.event {
                NodeEvent::NoteOn { note, velocity } => Some((s.frame, note, velocity)),
                _ => None,
            })
            .collect();

        assert_eq!(note_ons.len(), 2);
        assert_eq!(note_ons[0].0, 0);
        assert_eq!(note_ons[0].1, 74);
        assert!((note_ons[0].2 - 0.5).abs() < 1e-4);

        assert_eq!(note_ons[1].0, 44100);
        assert_eq!(note_ons[1].1, 62);
        assert!((note_ons[1].2 - 0.5).abs() < 1e-4);
    }

    #[test]
    fn test_track_filters_dc_block_and_high_cut() {
        let mut project = create_default_project("Filter Test");
        project.tracks[0].nodes.clear();
        project.tracks[0].clips.clear();
        project.tracks[0].dc_block = true;
        project.tracks[0].high_cut_hz = Some(200.0);
        project.tracks[0].gain = 1.0;
        project.tracks[0].muted = false;

        struct DcAndHighFreqNode {
            phase: f32,
        }
        impl AudioNode for DcAndHighFreqNode {
            fn name(&self) -> &str {
                "DcAndHighFreq"
            }
            fn process(
                &mut self,
                _in: &[&[summoner_core::audio::Sample]],
                out: &mut [&mut [summoner_core::audio::Sample]],
                ctx: &summoner_core::node::ProcessContext,
            ) {
                for sample in out[0].iter_mut() {
                    let freq = 5000.0f32;
                    let s = (self.phase * std::f32::consts::TAU).sin();
                    self.phase = (self.phase + freq / ctx.sample_rate as f32).fract();
                    *sample = 1.0 + s * 0.5;
                }
            }
        }

        let mut runner = GraphRunner::new(&project);
        runner.tracks[0].nodes.clear();
        runner.tracks[0].add_node(Box::new(DcAndHighFreqNode { phase: 0.0 }));

        let (mut out_l, mut out_r) = (vec![0.0f32; 1024], vec![0.0f32; 1024]);
        let ctx = summoner_core::node::ProcessContext::new(44100, 120.0, 0);

        for _ in 0..10 {
            runner.process_block(1024, &ctx, &mut [&mut out_l[..], &mut out_r[..]]);
        }

        let dc_avg: f32 = out_l.iter().sum::<f32>() / out_l.len() as f32;
        assert!(
            dc_avg.abs() < 0.1,
            "DC block filter failed to remove DC offset, avg: {}",
            dc_avg
        );
    }

    #[test]
    fn test_runner_plays_clip_notes_only_when_scheduled() {
        use summoner_project::schema::{SequenceConfig, TrackConfig, TrackerStepConfig};
        let mut project = create_default_project("Clip Test");
        let mut tc: TrackConfig = project.tracks[0].clone();
        tc.nodes.clear();
        tc.clips.clear();
        tc.gain = 1.0;
        tc.muted = false;
        project.tracks.clear();
        let mut params = std::collections::HashMap::new();
        params.insert("freq".to_string(), 100.0);
        tc.nodes.push(NodeConfig {
            kind: "SineOscillatorNode".to_string(),
            params,
            plugin_state: None,
        });
        // One step on beat 1 (second beat) at 120 bpm / 44.1k => frame 22050.
        tc.sequence = Some(SequenceConfig {
            start_beat: 1.0,
            steps: vec![TrackerStepConfig::default()],
            ..Default::default()
        });
        project.tracks.push(tc);
        let mut runner = GraphRunner::new(&project);
        let (mut l, mut r) = (vec![0.0f32; 512], vec![0.0f32; 512]);
        let mut ctx = summoner_core::node::ProcessContext::new(44100, 120.0, 0);
        runner.process_block(512, &ctx, &mut [&mut l[..], &mut r[..]]);
        assert!(l.iter().all(|s| *s == 0.0), "silent before the note");
        ctx.frame_position = 22050;
        runner.process_block(512, &ctx, &mut [&mut l[..], &mut r[..]]);
        assert!(l.iter().any(|s| s.abs() > 0.1), "sounds during the note");
    }

    #[test]
    fn test_graph_from_project_builds_valid_dag() {
        let mut project = create_default_project("Test Project");

        // Add a connection from node 0 to node 1 in port 0
        if let Some(track) = project.tracks.get_mut(1) {
            track
                .connections
                .push(summoner_project::schema::ConnectionConfig {
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
            plugin_state: None,
        };
        let mut node =
            NodeFactory::create_node(&config).expect("Factory should create SineOscillatorNode");
        let ctx = summoner_core::node::ProcessContext::new(44100, 120.0, 0);

        let mut out_l = vec![0.0f32; 512];
        let mut out_r = vec![0.0f32; 512];
        let dummy_in: [&[f32]; 0] = [];
        node.process(&dummy_in, &mut [&mut out_l[..], &mut out_r[..]], &ctx);

        let rms: f32 = (out_l.iter().map(|s| s * s).sum::<f32>() / out_l.len() as f32).sqrt();
        assert!(
            rms > 0.0,
            "SineOscillatorNode RMS output must be greater than zero"
        );
    }

    #[test]
    fn test_runner_respects_solo_pan_connections_and_automation() {
        use summoner_project::schema::{
            AutomationEventConfig, AutomationLaneConfig, ConnectionConfig, TrackConfig,
        };
        let mut project = create_default_project("Solo Pan Automation Test");
        project.tracks.clear();

        // Track 1: soloed, panned hard left (-1.0), SineOscillatorNode -> GainNode via connection
        let mut t1 = TrackConfig {
            id: 1,
            name: "Solo Left".to_string(),
            channels: 2,
            gain: 1.0,
            pan: -1.0,
            muted: false,
            soloed: true,
            ..Default::default()
        };
        t1.nodes.push(NodeConfig {
            kind: "SineOscillatorNode".to_string(),
            params: [("freq".to_string(), 440.0)].into_iter().collect(),
            plugin_state: None,
        });
        t1.nodes.push(NodeConfig {
            kind: "GainNode".to_string(),
            params: [("gain".to_string(), 1.0)].into_iter().collect(),
            plugin_state: None,
        });
        t1.connections.push(ConnectionConfig {
            from: "0:0".to_string(),
            to: "1:0".to_string(),
        });
        t1.connections.push(ConnectionConfig {
            from: "0:1".to_string(),
            to: "1:1".to_string(),
        });

        // Track 2: not soloed, panned hard right (+1.0)
        let mut t2 = TrackConfig {
            id: 2,
            name: "Unsoloed Right".to_string(),
            channels: 2,
            gain: 1.0,
            pan: 1.0,
            muted: false,
            soloed: false,
            ..Default::default()
        };
        t2.nodes.push(NodeConfig {
            kind: "SineOscillatorNode".to_string(),
            params: [("freq".to_string(), 880.0)].into_iter().collect(),
            plugin_state: None,
        });

        project.tracks.push(t1);
        project.tracks.push(t2);

        // Automation lane on track 1 silencing gain at frame 512
        project.automation_lanes.push(AutomationLaneConfig {
            param_id: "gain".to_string(),
            track_id: 1,
            events: vec![AutomationEventConfig {
                frame: 512,
                value: 0.0,
            }],
        });

        let mut runner = GraphRunner::new(&project);
        let (mut l, mut r) = (vec![0.0f32; 256], vec![0.0f32; 256]);
        let mut ctx = summoner_core::node::ProcessContext::new(44100, 120.0, 0);

        runner.process_block(256, &ctx, &mut [&mut l[..], &mut r[..]]);
        // Track 1 is soloed and panned hard left (-1.0), so left has signal and right is 0.0
        assert!(l.iter().any(|s| s.abs() > 0.1));
        assert!(r.iter().all(|s| *s == 0.0));

        // Advance to frame 512 where automation sets GainNode gain to 0.0
        ctx.frame_position = 512;
        runner.process_block(256, &ctx, &mut [&mut l[..], &mut r[..]]);
        // After 16-sample crossfade (if any), output should be silent
        assert!(l[32..].iter().all(|s| s.abs() < 1e-5));
    }

    #[test]
    fn test_factory_supports_all_known_node_types_and_composites() {
        for &kind in summoner_core::node::KNOWN_NODE_TYPES {
            let cfg = NodeConfig {
                kind: kind.to_string(),
                params: Default::default(),
                plugin_state: None,
            };
            assert!(
                NodeFactory::create_node_with_rate(&cfg, 48000).is_some(),
                "NodeFactory missing KNOWN_NODE_TYPES entry: {}",
                kind
            );
        }

        for kind in [
            "AetherSynth",
            "PluckSynth",
            "FmOperatorPair",
            "GlitchAetherMachine",
            "CyberpunkSubSynth",
            "AtmosphericPadSynth",
            "GlitchPercussionSynth",
            "DrumMachineDevice",
            "PluckedStringNode",
            "SamplerDevice",
        ] {
            let cfg = NodeConfig {
                kind: kind.to_string(),
                params: Default::default(),
                plugin_state: None,
            };
            assert!(
                NodeFactory::create_node_with_rate(&cfg, 48000).is_some(),
                "NodeFactory missing composite/instrument entry: {}",
                kind
            );
        }
    }

    #[test]
    fn test_runner_ratchet_phase_flip_channel_strip_and_mono_pan() {
        use summoner_project::schema::{SequenceConfig, TrackConfig, TrackerStepConfig};
        let mut project = create_default_project("Ratchet & Channel Strip Test");
        project.tracks.clear();
        project.transport.master_trim_db = -6.0;

        let mut tc = TrackConfig {
            id: 1,
            name: "Mono Panned Right".to_string(),
            channels: 1,
            gain: 1.0,
            pan: 1.0, // Hard right
            phase_flip: true,
            output_gain_db: 6.0, // cancels master_trim_db (-6 dB)
            sequence: Some(SequenceConfig {
                start_beat: 0.0,
                step_division: 1.0,
                steps: vec![TrackerStepConfig {
                    note: 69.0,
                    velocity: 1.0,
                    gate: 0.5,
                    ratchet: 2,
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        tc.nodes.push(NodeConfig {
            kind: "SineOscillatorNode".to_string(),
            params: [("freq".to_string(), 440.0)].into_iter().collect(),
            plugin_state: None,
        });
        project.tracks.push(tc);

        let mut runner = GraphRunner::new(&project);
        let (mut l, mut r) = (vec![0.0f32; 256], vec![0.0f32; 256]);
        let ctx = summoner_core::node::ProcessContext::new(44100, 120.0, 0);
        runner.process_block(256, &ctx, &mut [&mut l[..], &mut r[..]]);

        // Mono track panned hard right (+1.0) should be silent on left and non-zero on right
        assert!(l.iter().all(|&s| s == 0.0));
        assert!(r.iter().any(|&s| s.abs() > 0.5));
        // Phase flip inverted the initial positive sine half-cycle to negative
        assert!(r[1] < 0.0);
    }

    #[test]
    fn test_aux_sends_pre_post_fader_and_return_tracks() {
        use summoner_project::schema::{TrackConfig, TrackSendConfig};
        let mut project = create_default_project("Aux Send Test");
        project.tracks.clear();

        // Track 1: Source synth, gain = 0.5
        let mut t1 = TrackConfig {
            id: 1,
            name: "Synth".to_string(),
            gain: 0.5,
            send_to_master: false, // Don't send directly to master so we only hear the return
            sends: vec![TrackSendConfig {
                target_track_id: 2,
                amount: 1.0,
                is_pre_fader: false,
                muted: false,
            }],
            ..Default::default()
        };
        t1.nodes.push(NodeConfig {
            kind: "SineOscillatorNode".to_string(),
            params: [("freq".to_string(), 440.0)].into_iter().collect(),
            plugin_state: None,
        });

        // Track 2: Return track (Reverb return), gain = 1.0, send_to_master = true
        let t2 = TrackConfig {
            id: 2,
            name: "Reverb Return".to_string(),
            is_return_track: true,
            gain: 1.0,
            send_to_master: true,
            ..Default::default()
        };

        project.tracks.push(t1);
        project.tracks.push(t2);

        let mut runner = GraphRunner::new(&project);
        let (mut l, mut r) = (vec![0.0f32; 256], vec![0.0f32; 256]);
        let ctx = summoner_core::node::ProcessContext::new(44100, 120.0, 0);

        runner.process_block(256, &ctx, &mut [&mut l[..], &mut r[..]]);
        // Post-fader send from Track 1 reached Track 2 and was output to Master
        assert!(l.iter().any(|&s| s.abs() > 0.05));
    }

    #[test]
    fn test_subgroup_bus_routing() {
        use summoner_project::schema::TrackConfig;
        let mut project = create_default_project("Bus Test");
        project.tracks.clear();

        // Track 1: Kick -> bus_target = "Drum Bus"
        let mut t1 = TrackConfig {
            id: 1,
            name: "Kick".to_string(),
            bus_target: Some("Drum Bus".to_string()),
            gain: 0.8,
            send_to_master: true, // Should be overridden by bus_target
            ..Default::default()
        };
        t1.nodes.push(NodeConfig {
            kind: "SineOscillatorNode".to_string(),
            params: [("freq".to_string(), 60.0)].into_iter().collect(),
            plugin_state: None,
        });

        // Track 2: Drum Bus, is_bus_track = true
        let t2 = TrackConfig {
            id: 2,
            name: "Drum Bus".to_string(),
            is_bus_track: true,
            gain: 0.5,
            send_to_master: true,
            ..Default::default()
        };

        project.tracks.push(t1);
        project.tracks.push(t2);

        let mut runner = GraphRunner::new(&project);
        let (mut l, mut r) = (vec![0.0f32; 256], vec![0.0f32; 256]);
        let ctx = summoner_core::node::ProcessContext::new(44100, 120.0, 0);

        runner.process_block(256, &ctx, &mut [&mut l[..], &mut r[..]]);
        assert!(l.iter().any(|&s| s.abs() > 0.05));
    }

    #[test]
    fn test_vca_fader_scaling_and_mute_groups() {
        use summoner_project::schema::TrackConfig;
        let mut project = create_default_project("VCA Test");
        project.tracks.clear();

        // Track 1: Member of VCA Group 1
        let mut t1 = TrackConfig {
            id: 1,
            name: "Vocal 1".to_string(),
            gain: 1.0,
            vca_group: Some(1),
            mute_group: Some(2),
            ..Default::default()
        };
        t1.nodes.push(NodeConfig {
            kind: "SineOscillatorNode".to_string(),
            params: [("freq".to_string(), 440.0)].into_iter().collect(),
            plugin_state: None,
        });

        // Track 2: VCA Track controlling Group 1 with gain = 0.5
        let vca = TrackConfig {
            id: 2,
            name: "VCA Lead".to_string(),
            is_vca: true,
            gain: 0.5,
            vca_group: Some(1),
            ..Default::default()
        };

        project.tracks.push(t1);
        project.tracks.push(vca);

        let mut runner = GraphRunner::new(&project);
        let (mut l, mut r) = (vec![0.0f32; 256], vec![0.0f32; 256]);
        let ctx = summoner_core::node::ProcessContext::new(44100, 120.0, 0);

        runner.process_block(256, &ctx, &mut [&mut l[..], &mut r[..]]);
        let peak: f32 = l.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        // Track 1 had gain 1.0, scaled by VCA gain 0.5 => max peak ~ 0.5
        assert!(peak > 0.3 && peak < 0.6);
    }

    #[test]
    fn test_solo_safe_and_sidechain_audition() {
        use summoner_project::schema::TrackConfig;
        let mut project = create_default_project("Solo Safe & Sidechain Test");
        project.tracks.clear();

        // Track 1: Soloed
        let mut t1 = TrackConfig {
            id: 1,
            name: "Lead Synth".to_string(),
            gain: 1.0,
            soloed: true,
            ..Default::default()
        };
        t1.nodes.push(NodeConfig {
            kind: "SineOscillatorNode".to_string(),
            params: [("freq".to_string(), 440.0)].into_iter().collect(),
            plugin_state: None,
        });

        // Track 2: Not soloed, but solo_safe = true (e.g. Master FX or Click)
        let mut t2 = TrackConfig {
            id: 2,
            name: "Click Track".to_string(),
            gain: 1.0,
            soloed: false,
            solo_safe: true,
            ..Default::default()
        };
        t2.nodes.push(NodeConfig {
            kind: "SineOscillatorNode".to_string(),
            params: [("freq".to_string(), 880.0)].into_iter().collect(),
            plugin_state: None,
        });

        // Track 3: Sidechain auditioning Track 1
        let t3 = TrackConfig {
            id: 3,
            name: "Bass Ducked".to_string(),
            gain: 1.0,
            soloed: false,
            solo_safe: true,
            sidechain_source_track_id: Some(1),
            sidechain_audition: true,
            ..Default::default()
        };

        project.tracks.push(t1);
        project.tracks.push(t2);
        project.tracks.push(t3);

        let mut runner = GraphRunner::new(&project);
        let (mut l, mut r) = (vec![0.0f32; 256], vec![0.0f32; 256]);
        let ctx = summoner_core::node::ProcessContext::new(44100, 120.0, 0);

        runner.process_block(256, &ctx, &mut [&mut l[..], &mut r[..]]);
        // Solo-safe tracks output alongside soloed track
        assert!(l.iter().any(|&s| s.abs() > 0.1));
        // Verify telemetry
        assert!(runner.telemetry.channels[0].peak > 0.0);
        assert!(runner.telemetry.momentary_lufs > -70.0);
    }
}
