// Summoner - Deterministic Golden Vacuum Tube Saturation & Console Emulation Regression Suite
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

use blake3::Hasher;
use std::f32::consts::PI;
use std::fs;
use std::path::Path;
use summoner_core::allocator::AllocGuard;
use summoner_core::node::ProcessContext;
use summoner_core::param_bus::{ParamBus, ParamId};
use summoner_core::transport::Transport;
use summoner_dsp::console_emulation::{ConsoleEmulationNode, ConsoleMode};
use summoner_dsp::traits::SignalProcessor;
use summoner_dsp::tube_saturation::{TubeSaturationNode, TubeTopology};
#[cfg(feature = "gui")]
use summoner_gui::views::tube_bias_view::TubeBiasView;
use summoner_harmony::bus::{HarmonicBusBridge, HarmonicContext};
use summoner_harmony::cadence::{CadenceEngine, CadenceType, Chord};

const BLOCK_SIZE: usize = 64;

fn check_or_save_golden(hash_filename: &str, digest: &str) {
    let golden_dir = Path::new("tests/golden");
    if !golden_dir.exists() {
        let _ = fs::create_dir_all(golden_dir);
    }

    let hash_file = golden_dir.join(hash_filename);
    if hash_file.exists() {
        let expected = fs::read_to_string(&hash_file)
            .expect("Failed to read golden hash")
            .trim()
            .to_string();
        assert_eq!(
            digest, expected,
            "Golden vacuum tube & console emulation hash mismatch for {}",
            hash_filename
        );
    } else {
        fs::write(&hash_file, digest).expect("Failed to write golden hash");
    }
}

#[test]
fn test_golden_vacuum_tube_saturation_multitopology_deterministic() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    let mut tube_triode = TubeSaturationNode::new(3.2, -1.85).with_topology(TubeTopology::Triode12AX7);
    let mut tube_pentode = TubeSaturationNode::new(3.5, -1.50).with_topology(TubeTopology::PentodeEL34);
    let mut tube_tetrode = TubeSaturationNode::new(2.8, -2.10).with_topology(TubeTopology::BeamTetrode6L6);

    tube_triode.sag_compression = 0.45;
    tube_triode.warmth = 0.65;

    tube_pentode.sag_compression = 0.35;
    tube_pentode.warmth = 0.70;

    tube_tetrode.sag_compression = 0.30;
    tube_tetrode.warmth = 0.75;

    let mut in_l = [0.0f32; BLOCK_SIZE];
    let mut in_r = [0.0f32; BLOCK_SIZE];
    let mut out_l_tri = [0.0f32; BLOCK_SIZE];
    let mut out_r_tri = [0.0f32; BLOCK_SIZE];
    let mut out_l_pent = [0.0f32; BLOCK_SIZE];
    let mut out_r_pent = [0.0f32; BLOCK_SIZE];
    let mut out_l_tet = [0.0f32; BLOCK_SIZE];
    let mut out_r_tet = [0.0f32; BLOCK_SIZE];

    let mut hasher = Hasher::new();

    // Process 64 blocks of audio across all tube topologies (4096 samples)
    for block_idx in 0..64 {
        let ctx = ProcessContext::from_transport(&transport);

        for i in 0..BLOCK_SIZE {
            let n = (block_idx * BLOCK_SIZE + i) as f32;
            let low = (2.0 * PI * 55.0 * n / 48000.0).sin() * 0.55;
            let mid = (2.0 * PI * 880.0 * n / 48000.0).sin() * 0.40;
            let high = (2.0 * PI * 6500.0 * n / 48000.0).cos() * 0.25;
            in_l[i] = low + mid + high;
            in_r[i] = low * 0.95 + mid * 1.05 + high * 0.85;
        }

        {
            let _guard = AllocGuard::new();
            tube_triode.process_block(
                &[&in_l[..], &in_r[..]],
                &mut [&mut out_l_tri[..], &mut out_r_tri[..]],
                &ctx,
            );
            tube_pentode.process_block(
                &[&in_l[..], &in_r[..]],
                &mut [&mut out_l_pent[..], &mut out_r_pent[..]],
                &ctx,
            );
            tube_tetrode.process_block(
                &[&in_l[..], &in_r[..]],
                &mut [&mut out_l_tet[..], &mut out_r_tet[..]],
                &ctx,
            );
        }

        for i in 0..BLOCK_SIZE {
            assert!(out_l_tri[i].is_finite());
            assert!(out_r_tri[i].is_finite());
            assert!(out_l_pent[i].is_finite());
            assert!(out_r_pent[i].is_finite());
            assert!(out_l_tet[i].is_finite());
            assert!(out_r_tet[i].is_finite());

            hasher.update(&out_l_tri[i].to_le_bytes());
            hasher.update(&out_r_tri[i].to_le_bytes());
            hasher.update(&out_l_pent[i].to_le_bytes());
            hasher.update(&out_r_pent[i].to_le_bytes());
            hasher.update(&out_l_tet[i].to_le_bytes());
            hasher.update(&out_r_tet[i].to_le_bytes());
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_tube_saturation.hash", &digest);
}

#[test]
fn test_golden_analog_console_desk_emulation_multimode_deterministic() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    let mut console_neve = ConsoleEmulationNode::new(ConsoleMode::Neve, 2.4);
    let mut console_ssl = ConsoleEmulationNode::new(ConsoleMode::SSL, 2.0);
    let mut console_api = ConsoleEmulationNode::new(ConsoleMode::API, 2.8);

    console_neve.warmth = 0.70;
    console_neve.transformer_iron = 0.60;
    console_neve.crosstalk = 0.10;

    console_ssl.warmth = 0.50;
    console_ssl.transformer_iron = 0.30;
    console_ssl.crosstalk = 0.05;

    console_api.warmth = 0.60;
    console_api.transformer_iron = 0.70;
    console_api.crosstalk = 0.08;

    let mut in_l = [0.0f32; BLOCK_SIZE];
    let mut in_r = [0.0f32; BLOCK_SIZE];
    let mut out_l_neve = [0.0f32; BLOCK_SIZE];
    let mut out_r_neve = [0.0f32; BLOCK_SIZE];
    let mut out_l_ssl = [0.0f32; BLOCK_SIZE];
    let mut out_r_ssl = [0.0f32; BLOCK_SIZE];
    let mut out_l_api = [0.0f32; BLOCK_SIZE];
    let mut out_r_api = [0.0f32; BLOCK_SIZE];

    let mut hasher = Hasher::new();

    // Process 64 blocks of audio across all console modes (4096 samples)
    for block_idx in 0..64 {
        let ctx = ProcessContext::from_transport(&transport);

        for i in 0..BLOCK_SIZE {
            let n = (block_idx * BLOCK_SIZE + i) as f32;
            let low = (2.0 * PI * 52.0 * n / 48000.0).sin() * 0.50;
            let mid = (2.0 * PI * 2500.0 * n / 48000.0).sin() * 0.40;
            let high = (2.0 * PI * 10000.0 * n / 48000.0).cos() * 0.30;
            in_l[i] = low + mid + high;
            in_r[i] = low * 0.90 + mid * 1.10 + high * 0.80;
        }

        {
            let _guard = AllocGuard::new();
            console_neve.process_block(
                &[&in_l[..], &in_r[..]],
                &mut [&mut out_l_neve[..], &mut out_r_neve[..]],
                &ctx,
            );
            console_ssl.process_block(
                &[&in_l[..], &in_r[..]],
                &mut [&mut out_l_ssl[..], &mut out_r_ssl[..]],
                &ctx,
            );
            console_api.process_block(
                &[&in_l[..], &in_r[..]],
                &mut [&mut out_l_api[..], &mut out_r_api[..]],
                &ctx,
            );
        }

        for i in 0..BLOCK_SIZE {
            assert!(out_l_neve[i].is_finite());
            assert!(out_r_neve[i].is_finite());
            assert!(out_l_ssl[i].is_finite());
            assert!(out_r_ssl[i].is_finite());
            assert!(out_l_api[i].is_finite());
            assert!(out_r_api[i].is_finite());

            hasher.update(&out_l_neve[i].to_le_bytes());
            hasher.update(&out_r_neve[i].to_le_bytes());
            hasher.update(&out_l_ssl[i].to_le_bytes());
            hasher.update(&out_r_ssl[i].to_le_bytes());
            hasher.update(&out_l_api[i].to_le_bytes());
            hasher.update(&out_r_api[i].to_le_bytes());
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_console_emulation.hash", &digest);
}

#[test]
fn test_golden_harmonic_tension_cadence_and_parambus_dispatch() {
    let mut context = HarmonicContext::default();
    let mut param_bus = ParamBus::new();
    let p_tension = param_bus.register(ParamId(50), 0.0);
    let p_cadence = param_bus.register(ParamId(51), 0.0);
    let p_urgency = param_bus.register(ParamId(52), 0.0);

    let bridge = HarmonicBusBridge::new();
    let engine = CadenceEngine::new();

    let mut hasher = Hasher::new();

    // Test progression of chords: C Major -> G7 -> F Major -> Am -> Dm
    let chords = vec![
        Chord::new("I", 0, vec![0, 4, 7]),
        Chord::new("V7", 7, vec![0, 4, 7, 10]),
        Chord::new("IV", 5, vec![0, 4, 7]),
        Chord::new("vi", 9, vec![0, 3, 7]),
        Chord::new("ii", 2, vec![0, 3, 7]),
    ];

    for chord in &chords {
        context.clear_notes();
        for &interval in &chord.intervals {
            let note = (60 + chord.root_step + interval) as u8;
            context.push_note_on(note);
        }

        let tension = context.analyze_current_tension();
        engine.dispatch_to_param_bus(&tension, &param_bus, ParamId(50), ParamId(51), ParamId(52));
        bridge.update_from_tension(context.root_note, &tension);

        let t_val = p_tension.get();
        let c_val = p_cadence.get();
        let u_val = p_urgency.get();

        assert!(t_val.is_finite());
        assert!(c_val.is_finite());
        assert!(u_val.is_finite());

        hasher.update(&t_val.to_le_bytes());
        hasher.update(&c_val.to_le_bytes());
        hasher.update(&u_val.to_le_bytes());
        hasher.update(&tension.roughness.to_le_bytes());
        hasher.update(&tension.tonal_distance.to_le_bytes());

        let suggestions = CadenceEngine::suggest_next_chords(chord, &context);
        for sug in &suggestions {
            hasher.update(sug.name.as_bytes());
            hasher.update(&sug.root_step.to_le_bytes());
        }
    }

    let authentic_prog = CadenceEngine::generate_progression(&context, CadenceType::Authentic);
    for ch in &authentic_prog {
        let freqs = ch.frequencies(&context);
        for &f in &freqs {
            hasher.update(&f.to_le_bytes());
        }
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_harmonic_tension.hash", &digest);
}

#[cfg(feature = "gui")]
#[test]
fn test_golden_tube_bias_view_snapshot_and_telemetry() {
    let mut view = TubeBiasView::new();
    view.topology = TubeTopology::Triode12AX7;
    view.bias_voltage_v = -1.85;
    view.plate_voltage_v = 250.0;
    view.drive_warmth_db = 8.5;
    view.sag_compression_pct = 35.0;
    view.asymmetry_balance_pct = 65.0;

    let ascii = view.render_ascii(80, 24);
    assert!(!ascii.is_empty());
    assert!(ascii[0].contains("TUBE BIAS"));
    assert!(ascii[0].contains("12AX7"));

    let render_path = "scratch/renders/tube_bias_view.png";
    let res = view.render_snapshot_png(render_path, 800, 520);
    let _ = view.render_snapshot_png("../../scratch/renders/tube_bias_view.png", 800, 520);
    assert!(res.is_ok(), "Tube bias view snapshot render must succeed: {:?}", res);

    let mut hasher = Hasher::new();
    for line in &ascii {
        hasher.update(line.as_bytes());
    }
    hasher.update(&view.bias_voltage_v.to_le_bytes());
    hasher.update(&view.plate_voltage_v.to_le_bytes());
    hasher.update(&view.drive_warmth_db.to_le_bytes());
    hasher.update(&view.sag_compression_pct.to_le_bytes());
    hasher.update(&view.asymmetry_balance_pct.to_le_bytes());

    let spectrum = view.calculate_harmonic_spectrum();
    for &h in &spectrum {
        hasher.update(&h.to_le_bytes());
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_tube_bias_view.hash", &digest);
}
