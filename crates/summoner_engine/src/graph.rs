// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

use summoner_core::graph::{Edge, NodeGraph};
use summoner_core::node::{AudioNode, GainNode};
use summoner_core::track::Track;
use summoner_dsp::filters::{FilterComb, FilterLadder, FilterSVF};
use summoner_dsp::math::VCA;
use summoner_dsp::modulators::EnvADSR;
use summoner_dsp::oscillators::{NoiseGen, OscPulse, OscSaw, OscSine, OscTriangle, OscWavetable};
use summoner_dsp::traits::{ProcessorNodeAdapter, VoiceNode};
use summoner_dsp::SamplerNode;
use summoner_dsp::{
    CompressorNode, EffectChorus, EffectFlanger, EffectPhaser, FilterBiquad, FrequencyShifter,
    GranularSynthNode, LimiterNode, LufsMeterNode, RingModulator,
};
use summoner_core::node::NodeEvent;
use summoner_core::track::ScheduledEvent;
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
    pub fn create_node_with_rate(config: &NodeConfig, sample_rate: u32) -> Option<Box<dyn AudioNode>> {
        let params = &config.params;
        match config.kind.as_str() {
            "OscSaw" => Some(Box::new(VoiceNode::new(OscSaw::new(
                *params.get("freq").unwrap_or(&440.0),
            )))),
            "OscPulse" => Some(Box::new(VoiceNode::new(OscPulse::new(
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
            "OscTriangle" => Some(Box::new(VoiceNode::new(OscTriangle::new(
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
                Some(Box::new(VoiceNode::new(osc)))
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
                let bias = *params.get("bias").or_else(|| params.get("bias_voltage_v")).unwrap_or(&-1.85);
                let mut node = summoner_dsp::TubeSaturationNode::new(drive, bias);
                if let Some(&topo) = params.get("topology") {
                    match topo.round() as i32 {
                        0 => node.topology = summoner_dsp::TubeTopology::Triode12AX7,
                        1 => node.topology = summoner_dsp::TubeTopology::PentodeEL34,
                        2 => node.topology = summoner_dsp::TubeTopology::BeamTetrode6L6,
                        _ => node.topology = summoner_dsp::TubeTopology::CleanModern,
                    }
                }
                if let Some(&plate) = params.get("plate_voltage").or_else(|| params.get("plate_voltage_v")) {
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
                if let Some(&iron) = params.get("transformer_iron").or_else(|| params.get("iron")) {
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
            if let Some(node) = NodeFactory::create_node_with_rate(nc, project.transport.sample_rate) {
                graph.add_node(node);
            } else {
                eprintln!("Warning: Unknown node kind '{}'; using passthrough", nc.kind);
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
    let mut events = Vec::new();
    for seq in tc.sequence.iter().chain(tc.clips.iter()) {
        for (i, step) in seq.steps.iter().enumerate() {
            if !step.active || step.muted || step.probability <= 0.0 {
                continue;
            }
            let note = (step.note + seq.pitch_offset + step.pitch_offset as f64)
                .round()
                .clamp(0.0, 127.0) as u8;
            let beat = seq.start_beat + i as f64 * seq.step_division;
            let on = (beat * frames_per_beat).round().max(0.0) as u64;
            let off = ((beat + step.gate as f64 * seq.step_division) * frames_per_beat)
                .round()
                .max(0.0) as u64;
            events.push(ScheduledEvent {
                frame: on,
                event: NodeEvent::NoteOn {
                    note,
                    velocity: (step.velocity * seq.gain as f32).clamp(0.0, 1.0),
                },
            });
            events.push(ScheduledEvent {
                frame: off.max(on + 1),
                event: NodeEvent::NoteOff { note },
            });
        }
    }
    if !events.is_empty() {
        events.push(ScheduledEvent {
            frame: 0,
            event: NodeEvent::NoteOff { note: 0 },
        });
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
        let n = spec
            .trim()
            .trim_start_matches('n')
            .parse::<usize>()
            .ok()?;
        Some((n, 0))
    }
}

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
            track.soloed = tc.soloed;
            track.tuning_edo = tc.tuning_edo;
            track.tuning_root_hz = tc.tuning_root_hz;

            if !tc.connections.is_empty() {
                let mut dag = NodeGraph::new(
                    tc.name.clone(),
                    MAX_RUNNER_BLOCK_SIZE,
                    tc.channels.max(2),
                );
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

        let any_soloed = self.tracks.iter().any(|t| t.soloed);

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

            let (slice_l, slice_r) = (
                &mut self.scratch_l[..block_size],
                &mut self.scratch_r[..block_size],
            );
            let mut track_out_slices: [&mut [f32]; 2] = [slice_l, slice_r];

            track.process(
                block_size,
                &local_ctx,
                &mut track_out_slices[..out_buffers.len().min(2)],
            );

            if any_soloed && !track.soloed {
                continue;
            }

            let pan = track.pan.clamp(-1.0, 1.0);
            let num_out = out_buffers.len().min(2);
            if num_out == 2 {
                let left_gain = track.gain * (1.0 - pan).min(1.0);
                let right_gain = track.gain * (1.0 + pan).min(1.0);
                let scratch_l = &self.scratch_l[..block_size];
                let scratch_r = &self.scratch_r[..block_size];
                for i in 0..block_size {
                    out_buffers[0][i] += scratch_l[i] * left_gain;
                    out_buffers[1][i] += scratch_r[i] * right_gain;
                }
            } else if num_out == 1 {
                let scratch_l = &self.scratch_l[..block_size];
                for i in 0..block_size {
                    out_buffers[0][i] += scratch_l[i] * track.gain;
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
}
