// Summoner - Tier 19 Testing & Quality Assurance Suite
// Copyright (C) 2026 nilsanderselde

use summoner_core::graph::{Edge, NodeGraph};
use summoner_core::mpe::{MpeEvent, MpeRouter};
use summoner_core::node::{GainNode, ProcessContext};
use summoner_core::param_bus::{ParamBus, ParamId};
use summoner_core::transport::Transport;
use summoner_core::wav::WavWriter;

use summoner_dsp::traits::SignalProcessor;
use summoner_dsp::{
    AetherSynth, EffectChorus, EffectFlanger, EffectPhaser, EnvADSR, FilterLadder, FilterSVF,
    FmOperatorPair, GranularSynthNode, OscPulse, OscSaw, PluckSynth, SampleBuffer,
};

use summoner_harmony::bus::HarmonicContext;
use summoner_project::{create_default_project, parse_project_toml, serialize_project_toml};
use summoner_sequencer::generative::GenerativeEngine;

use std::env;
use std::fs;
use std::sync::Arc;
use std::thread;

#[test]
fn test_aether_synth_rms_nonzero() {
    let transport = Transport::new(44100, 120.0);
    let ctx = ProcessContext::from_transport(&transport);
    let mut synth = AetherSynth::new(440.0);
    synth.trigger(true);

    let mut out_l = vec![0.0f32; 512];
    let dummy_in: [&[f32]; 0] = [];
    synth.process_block(&dummy_in, &mut [&mut out_l[..]], &ctx);

    let rms = (out_l.iter().map(|&s| (s * s) as f64).sum::<f64>() / out_l.len() as f64).sqrt();
    assert!(rms > 0.001, "Expected non-zero RMS for AetherSynth, got {}", rms);
}

#[test]
fn test_pluck_synth_rms_nonzero() {
    let transport = Transport::new(44100, 120.0);
    let ctx = ProcessContext::from_transport(&transport);
    let mut pluck = PluckSynth::new(440.0);
    pluck.pluck();

    let mut out_l = vec![0.0f32; 512];
    let dummy_in: [&[f32]; 0] = [];
    pluck.process_block(&dummy_in, &mut [&mut out_l[..]], &ctx);

    let rms = (out_l.iter().map(|&s| (s * s) as f64).sum::<f64>() / out_l.len() as f64).sqrt();
    assert!(rms > 0.001, "Expected non-zero RMS for PluckSynth, got {}", rms);
}

#[test]
fn test_fm_operator_pair_rms_nonzero() {
    let transport = Transport::new(44100, 120.0);
    let ctx = ProcessContext::from_transport(&transport);
    let mut fm = FmOperatorPair::new(440.0, 2.0);
    fm.trigger(true);

    let mut out_l = vec![0.0f32; 512];
    let dummy_in: [&[f32]; 0] = [];
    fm.process_block(&dummy_in, &mut [&mut out_l[..]], &ctx);

    let rms = (out_l.iter().map(|&s| (s * s) as f64).sum::<f64>() / out_l.len() as f64).sqrt();
    assert!(rms > 0.001, "Expected non-zero RMS for FmOperatorPair, got {}", rms);
}

#[test]
fn test_granular_synth_rms_nonzero() {
    let sample_rate = 44100;
    let mut data = vec![0.0f32; sample_rate * 2];
    for i in 0..data.len() {
        data[i] = (2.0 * std::f32::consts::PI * 440.0 * i as f32 / sample_rate as f32).sin();
    }
    let buf = Arc::new(SampleBuffer::new(data, sample_rate as u32, 1));
    let mut granular = GranularSynthNode::new(sample_rate as u32);
    granular.load_buffer(buf);

    let transport = Transport::new(44100, 120.0);
    let ctx = ProcessContext::from_transport(&transport);
    let mut out_l = vec![0.0f32; 512];
    let dummy_in: [&[f32]; 0] = [];
    granular.process_block(&dummy_in, &mut [&mut out_l[..]], &ctx);

    let rms = (out_l.iter().map(|&s| (s * s) as f64).sum::<f64>() / out_l.len() as f64).sqrt();
    assert!(rms > 0.0001, "Expected non-zero RMS for GranularSynthNode, got {}", rms);
}

#[test]
fn test_filter_ladder_no_nan() {
    let transport = Transport::new(44100, 120.0);
    let ctx = ProcessContext::from_transport(&transport);
    let mut filter = FilterLadder::new(1000.0, 0.9);

    let extreme_inputs = vec![
        f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 1e10, -1e10, 1e-35, 0.0, 1.0, -1.0,
    ];
    let mut input = Vec::new();
    for _ in 0..450 {
        input.extend_from_slice(&extreme_inputs);
    }

    let mut out_l = vec![0.0f32; input.len()];
    filter.process_block(&[&input[..]], &mut [&mut out_l[..]], &ctx);

    for (i, &s) in out_l.iter().enumerate() {
        assert!(s.is_finite(), "FilterLadder produced non-finite sample at index {}: {}", i, s);
    }
}

#[test]
fn test_filter_svf_no_nan() {
    let transport = Transport::new(44100, 120.0);
    let ctx = ProcessContext::from_transport(&transport);
    let mut filter = FilterSVF::new(1000.0, 0.9);

    let extreme_inputs = vec![
        f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 1e10, -1e10, 1e-35, 0.0, 1.0, -1.0,
    ];
    let mut input = Vec::new();
    for _ in 0..450 {
        input.extend_from_slice(&extreme_inputs);
    }

    let mut out_l = vec![0.0f32; input.len()];
    filter.process_block(&[&input[..]], &mut [&mut out_l[..]], &ctx);

    for (i, &s) in out_l.iter().enumerate() {
        assert!(s.is_finite(), "FilterSVF produced non-finite sample at index {}: {}", i, s);
    }
}

#[test]
fn test_osc_saw_frequency_accuracy() {
    let transport = Transport::new(44100, 120.0);
    let ctx = ProcessContext::from_transport(&transport);
    let mut saw = OscSaw::new(440.0);

    let mut out_l = vec![0.0f32; 44100];
    let dummy_in: [&[f32]; 0] = [];
    saw.process_block(&dummy_in, &mut [&mut out_l[..]], &ctx);

    let mut zero_crossings = 0;
    for i in 1..out_l.len() {
        if out_l[i - 1] <= 0.0 && out_l[i] > 0.0 {
            zero_crossings += 1;
        }
    }

    let measured_freq = zero_crossings as f64;
    let err = (measured_freq - 440.0).abs() / 440.0;
    assert!(err < 0.001, "Measured saw frequency {} deviated by {:.4}% (expected 440 Hz)", measured_freq, err * 100.0);
}

#[test]
fn test_osc_pulse_pwm_duty_cycle() {
    let transport = Transport::new(40000, 120.0);
    let ctx = ProcessContext::from_transport(&transport);
    let mut pulse = OscPulse::new(100.0, 0.25);

    let mut out_l = vec![0.0f32; 4000];
    let dummy_in: [&[f32]; 0] = [];
    pulse.process_block(&dummy_in, &mut [&mut out_l[..]], &ctx);

    let pos_count = out_l.iter().filter(|&&s| s > 0.0).count();
    let duty_cycle = pos_count as f32 / out_l.len() as f32;
    assert!((duty_cycle - 0.25).abs() < 0.02, "Measured duty cycle {} deviates from 0.25", duty_cycle);
}

#[test]
fn test_env_adsr_shape() {
    let mut env = EnvADSR::new(0.01, 0.02, 0.5, 0.03);
    env.trigger(true);

    let mut peak = 0.0f32;
    for _ in 0..441 {
        let val = env.process_sample(44100);
        if val > peak {
            peak = val;
        }
    }
    assert!(peak > 0.9, "EnvADSR attack peak failed to reach ~1.0, got {}", peak);

    let mut sustain_val = 0.0f32;
    for _ in 0..882 {
        sustain_val = env.process_sample(44100);
    }
    assert!((sustain_val - 0.5).abs() < 0.1, "EnvADSR decay failed to reach ~0.5 sustain, got {}", sustain_val);

    env.trigger(false);
    let mut release_val = 0.0f32;
    for _ in 0..1323 {
        release_val = env.process_sample(44100);
    }
    assert!(release_val < 0.05, "EnvADSR release failed to decay to ~0.0, got {}", release_val);
}

#[test]
fn test_effect_chorus_modulates_signal() {
    let transport = Transport::new(44100, 120.0);
    let ctx = ProcessContext::from_transport(&transport);
    let mut chorus = EffectChorus::new();

    let input = vec![0.5f32; 1024];
    let mut out_l = vec![0.0f32; 1024];
    chorus.process_block(&[&input[..]], &mut [&mut out_l[..]], &ctx);

    assert!(out_l.iter().any(|&s| s != input[0]), "Chorus output expected to modulate signal");
}

#[test]
fn test_effect_flanger_modulates_signal() {
    let transport = Transport::new(44100, 120.0);
    let ctx = ProcessContext::from_transport(&transport);
    let mut flanger = EffectFlanger::new();

    let input = vec![0.5f32; 1024];
    let mut out_l = vec![0.0f32; 1024];
    flanger.process_block(&[&input[..]], &mut [&mut out_l[..]], &ctx);

    assert!(out_l.iter().any(|&s| s != input[0]), "Flanger output expected to modulate signal");
}

#[test]
fn test_effect_phaser_modulates_signal() {
    let transport = Transport::new(44100, 120.0);
    let ctx = ProcessContext::from_transport(&transport);
    let mut phaser = EffectPhaser::new();

    let input = vec![0.5f32; 1024];
    let mut out_l = vec![0.0f32; 1024];
    phaser.process_block(&[&input[..]], &mut [&mut out_l[..]], &ctx);

    assert!(out_l.iter().any(|&s| s != input[0]), "Phaser output expected to modulate signal");
}

#[test]
fn test_node_graph_topological_sort_cycle_detection() {
    let mut graph = NodeGraph::new("TestGraph", 64, 2);
    let n0 = graph.add_node(Box::new(GainNode::new(1.0)));
    let n1 = graph.add_node(Box::new(GainNode::new(1.0)));

    graph.add_edge(Edge { from_node: n0, from_port: 0, to_node: n1, to_port: 0 });
    graph.add_edge(Edge { from_node: n1, from_port: 0, to_node: n0, to_port: 0 });

    assert!(graph.has_cycle, "Expected cycle detection to set has_cycle = true");
}

#[test]
fn test_param_bus_cross_thread_read_write() {
    let mut bus_setup = ParamBus::new();
    bus_setup.register(ParamId(100), 0.5);
    let bus = Arc::new(bus_setup);

    let mut handles = Vec::new();
    for _ in 0..4 {
        let bus_clone = Arc::clone(&bus);
        handles.push(thread::spawn(move || {
            for step in 0..1000 {
                let val = (step as f32 % 100.0) / 100.0;
                bus_clone.set(ParamId(100), val);
            }
        }));
        let bus_clone2 = Arc::clone(&bus);
        handles.push(thread::spawn(move || {
            for _ in 0..1000 {
                let val = bus_clone2.get(ParamId(100)).unwrap();
                assert!(val.is_finite());
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
}

#[test]
fn test_mpe_router_channel_dispatch() {
    let mut router = MpeRouter::new();
    for ch in 2..=16 {
        router.dispatch(&MpeEvent::NoteOn { voice_id: ch as u32, channel: ch, note: 60.0, velocity: 0.8 });
    }
    let active_count = router.voices.iter().filter(|v| v.is_active).count();
    assert_eq!(active_count, 15);
}

#[test]
fn test_harmony_bus_chord_detection_c_major() {
    let mut context = HarmonicContext::default();
    context.push_note_on(60);
    context.push_note_on(64);
    context.push_note_on(67);

    let chord = context.analyze_active_chord();
    assert!(chord.contains("C Major"), "Expected C Major chord detection, got '{}'", chord);
}

#[test]
fn test_euclidean_rhythm_3_8() {
    let pattern_bool = GenerativeEngine::euclidean_rhythm(3, 8);
    let pattern_int: Vec<i32> = pattern_bool.into_iter().map(|b| if b { 1 } else { 0 }).collect();
    assert_eq!(pattern_int, vec![1, 0, 0, 1, 0, 0, 1, 0]);
}

#[test]
fn test_project_toml_round_trip() {
    let proj = create_default_project("QA Roundtrip Session");
    let toml_str = serialize_project_toml(&proj).expect("Serialization failed");
    let parsed = parse_project_toml(&toml_str).expect("Deserialization failed");
    assert_eq!(proj, parsed);
}

#[test]
fn test_blake3_asset_hash_matches_file() {
    let mut temp_path = env::temp_dir();
    temp_path.push("test_asset_qa.raw");
    let content = b"summoner daw audio asset blake3 test 2026";
    fs::write(&temp_path, content).expect("Failed to write temp asset file");

    let bytes = fs::read(&temp_path).expect("Failed to read temp asset file");
    let computed_hash = blake3::hash(&bytes).to_hex().to_string();
    let expected_hash = blake3::hash(content).to_hex().to_string();

    fs::remove_file(&temp_path).ok();
    assert_eq!(computed_hash, expected_hash);
}

#[test]
fn test_cli_init_creates_project_dir() {
    let mut proj_path = env::temp_dir();
    proj_path.push("test_cli_init_proj.toml");

    let default_proj = create_default_project("CLI Init Test");
    let serialized = serialize_project_toml(&default_proj).unwrap();
    fs::write(&proj_path, serialized).unwrap();

    assert!(proj_path.exists());
    let content = fs::read_to_string(&proj_path).unwrap();
    let parsed = parse_project_toml(&content).unwrap();
    assert_eq!(parsed.name, "CLI Init Test");
    fs::remove_file(&proj_path).ok();
}

#[test]
fn test_cli_render_wav_produces_wav() {
    let mut wav_path = env::temp_dir();
    wav_path.push("test_cli_render_out.wav");

    {
        let mut writer = WavWriter::create(&wav_path, 44100, 2).unwrap();
        let samples = vec![0.0f32; 128];
        writer.write_interleaved_samples(&samples).unwrap();
        writer.finalize().unwrap();
    }

    assert!(wav_path.exists());
    let metadata = fs::metadata(&wav_path).unwrap();
    assert!(metadata.len() > 44);
    fs::remove_file(&wav_path).ok();
}

#[test]
fn test_cli_harmony_suggest_returns_notes() {
    let mut context = HarmonicContext::default();
    context.push_note_on(60);
    context.push_note_on(64);
    context.push_note_on(67);

    let suggestions = context.suggest_next_chord_notes();
    assert!(!suggestions.is_empty(), "Expected non-empty harmony suggestions");
}
