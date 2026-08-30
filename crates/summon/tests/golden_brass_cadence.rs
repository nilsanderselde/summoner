// Summoner - Deterministic Golden Waveguide Brass & Harmonic Cadence Regression Suite
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

use blake3::Hasher;
use std::fs;
use std::path::Path;
use summoner_core::allocator::AllocGuard;
use summoner_core::cadence_bus::{CadenceBus, ChordQuality};
use summoner_core::node::ProcessContext;
use summoner_core::param_bus::{ParamBus, ParamId};
use summoner_core::transport::Transport;
use summoner_dsp::horn_reflection::{HornFlarePreset, HornMuteType, HornReflectionFilter};
use summoner_dsp::traits::SignalProcessor;
use summoner_dsp::waveguide_brass::{BrassInstrumentType, WaveguideBrass};
#[cfg(feature = "gui")]
use summoner_gui::views::cadence_flow_view::CadenceFlowView;
#[cfg(feature = "gui")]
use summoner_gui::views::waveguide_brass_view::WaveguideBrassView;
use summoner_harmony::cadence_graph::{CadencePathResolver, VoiceLeadingOptimizer};

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
            "Golden waveguide brass & harmonic cadence hash mismatch for {}",
            hash_filename
        );
    } else {
        fs::write(&hash_file, digest).expect("Failed to write golden hash");
    }
}

#[test]
fn test_golden_waveguide_brass_bernoulli_oscillation_deterministic() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    // 1. Bb Trumpet with piston valve combinations
    let mut trumpet = WaveguideBrass::new(sample_rate);
    trumpet.set_instrument(BrassInstrumentType::Trumpet);
    trumpet.set_blowing_pressure(0.92);
    trumpet.tongue_articulation(0.85);
    trumpet.set_valves(true, false, false); // Valve 1 depressed

    // 2. French Horn with hyperbolic flare
    let mut french_horn = WaveguideBrass::new(sample_rate);
    french_horn.set_instrument(BrassInstrumentType::FrenchHorn);
    french_horn.set_blowing_pressure(0.80);
    french_horn.tongue_articulation(0.70);

    // 3. Tenor Trombone with slide extension
    let mut trombone = WaveguideBrass::new(sample_rate);
    trombone.set_instrument(BrassInstrumentType::Trombone);
    trombone.set_blowing_pressure(0.88);
    trombone.set_slide(0.35);
    trombone.tongue_articulation(0.90);

    // 4. Bass Tuba with massive acoustic bore
    let mut tuba = WaveguideBrass::new(sample_rate);
    tuba.set_instrument(BrassInstrumentType::Tuba);
    tuba.set_blowing_pressure(0.95);
    tuba.tongue_articulation(0.95);
    tuba.set_valves(true, true, true); // All valves depressed

    let in_l = [0.0f32; BLOCK_SIZE];
    let in_r = [0.0f32; BLOCK_SIZE];
    let mut out_l_tr = [0.0f32; BLOCK_SIZE];
    let mut out_r_tr = [0.0f32; BLOCK_SIZE];
    let mut out_l_fh = [0.0f32; BLOCK_SIZE];
    let mut out_r_fh = [0.0f32; BLOCK_SIZE];
    let mut out_l_tb = [0.0f32; BLOCK_SIZE];
    let mut out_r_tb = [0.0f32; BLOCK_SIZE];
    let mut out_l_tu = [0.0f32; BLOCK_SIZE];
    let mut out_r_tu = [0.0f32; BLOCK_SIZE];

    let mut hasher = Hasher::new();

    // Process 64 blocks of audio (4096 samples)
    for _block_idx in 0..64 {
        let ctx = ProcessContext::from_transport(&transport);

        {
            let _guard = AllocGuard::new();
            trumpet.process_block(
                &[&in_l[..], &in_r[..]],
                &mut [&mut out_l_tr[..], &mut out_r_tr[..]],
                &ctx,
            );
            french_horn.process_block(
                &[&in_l[..], &in_r[..]],
                &mut [&mut out_l_fh[..], &mut out_r_fh[..]],
                &ctx,
            );
            trombone.process_block(
                &[&in_l[..], &in_r[..]],
                &mut [&mut out_l_tb[..], &mut out_r_tb[..]],
                &ctx,
            );
            tuba.process_block(
                &[&in_l[..], &in_r[..]],
                &mut [&mut out_l_tu[..], &mut out_r_tu[..]],
                &ctx,
            );
        }

        for i in 0..BLOCK_SIZE {
            assert!(out_l_tr[i].is_finite());
            assert!(out_r_tr[i].is_finite());
            assert!(out_l_fh[i].is_finite());
            assert!(out_r_fh[i].is_finite());
            assert!(out_l_tb[i].is_finite());
            assert!(out_r_tb[i].is_finite());
            assert!(out_l_tu[i].is_finite());
            assert!(out_r_tu[i].is_finite());

            assert!((-1.0..=1.0).contains(&out_l_tr[i]));
            assert!((-1.0..=1.0).contains(&out_r_tr[i]));
            assert!((-1.0..=1.0).contains(&out_l_fh[i]));
            assert!((-1.0..=1.0).contains(&out_r_fh[i]));
            assert!((-1.0..=1.0).contains(&out_l_tb[i]));
            assert!((-1.0..=1.0).contains(&out_r_tb[i]));
            assert!((-1.0..=1.0).contains(&out_l_tu[i]));
            assert!((-1.0..=1.0).contains(&out_r_tu[i]));

            hasher.update(&out_l_tr[i].to_le_bytes());
            hasher.update(&out_r_tr[i].to_le_bytes());
            hasher.update(&out_l_fh[i].to_le_bytes());
            hasher.update(&out_r_fh[i].to_le_bytes());
            hasher.update(&out_l_tb[i].to_le_bytes());
            hasher.update(&out_r_tb[i].to_le_bytes());
            hasher.update(&out_l_tu[i].to_le_bytes());
            hasher.update(&out_r_tu[i].to_le_bytes());
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_waveguide_brass.hash", &digest);
}

#[test]
fn test_golden_horn_reflection_and_radiation_deterministic() {
    let sample_rate = 48000;

    let mut filter_trumpet = HornReflectionFilter::new(sample_rate);
    filter_trumpet.set_preset(HornFlarePreset::Trumpet);
    filter_trumpet.set_mute(HornMuteType::Straight);

    let mut filter_horn = HornReflectionFilter::new(sample_rate);
    filter_horn.set_preset(HornFlarePreset::FrenchHorn);
    filter_horn.set_mute(HornMuteType::Harmon);

    let mut filter_trombone = HornReflectionFilter::new(sample_rate);
    filter_trombone.set_preset(HornFlarePreset::Trombone);
    filter_trombone.set_mute(HornMuteType::Cup);

    let mut filter_tuba = HornReflectionFilter::new(sample_rate);
    filter_tuba.set_preset(HornFlarePreset::Tuba);
    filter_tuba.set_mute(HornMuteType::Open);

    let mut hasher = Hasher::new();

    for i in 0..2048 {
        let test_stimulus = ((i as f32) * 0.05).sin() * 0.6 + ((i as f32) * 0.17).cos() * 0.3;

        let (refl_tr, rad_tr);
        let (refl_fh, rad_fh);
        let (refl_tb, rad_tb);
        let (refl_tu, rad_tu);

        {
            let _guard = AllocGuard::new();
            let res_tr = filter_trumpet.process_reflection(test_stimulus);
            let res_fh = filter_horn.process_reflection(test_stimulus);
            let res_tb = filter_trombone.process_reflection(test_stimulus);
            let res_tu = filter_tuba.process_reflection(test_stimulus);

            refl_tr = res_tr.0;
            rad_tr = res_tr.1;
            refl_fh = res_fh.0;
            rad_fh = res_fh.1;
            refl_tb = res_tb.0;
            rad_tb = res_tb.1;
            refl_tu = res_tu.0;
            rad_tu = res_tu.1;
        }

        assert!(refl_tr.is_finite());
        assert!(rad_tr.is_finite());
        assert!(refl_fh.is_finite());
        assert!(rad_fh.is_finite());
        assert!(refl_tb.is_finite());
        assert!(rad_tb.is_finite());
        assert!(refl_tu.is_finite());
        assert!(rad_tu.is_finite());

        hasher.update(&refl_tr.to_le_bytes());
        hasher.update(&rad_tr.to_le_bytes());
        hasher.update(&refl_fh.to_le_bytes());
        hasher.update(&rad_fh.to_le_bytes());
        hasher.update(&refl_tb.to_le_bytes());
        hasher.update(&rad_tb.to_le_bytes());
        hasher.update(&refl_tu.to_le_bytes());
        hasher.update(&rad_tu.to_le_bytes());
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_horn_reflection.hash", &digest);
}

#[test]
fn test_golden_cadence_path_resolver_and_voice_leading_deterministic() {
    let resolver = CadencePathResolver::new();
    let cadence_bus = CadenceBus::new();
    let mut param_bus = ParamBus::new();
    for i in 0..10 {
        param_bus.register(ParamId(300 + i), 0.0);
    }

    // Resolve multiple cadential sequences
    let path_ii_v_i = resolver.resolve_cadence_path(2, ChordQuality::Minor, 0, ChordQuality::Major);
    let path_neapolitan = resolver.resolve_cadence_path(1, ChordQuality::Major, 0, ChordQuality::Major);
    let path_ger6 = resolver.resolve_cadence_path(8, ChordQuality::Dominant7, 0, ChordQuality::Major);
    let path_deceptive = resolver.resolve_cadence_path(7, ChordQuality::Dominant7, 9, ChordQuality::Minor);
    let path_plagal = resolver.resolve_cadence_path(5, ChordQuality::Major, 0, ChordQuality::Major);

    let mut hasher = Hasher::new();

    let all_paths = [&path_ii_v_i, &path_neapolitan, &path_ger6, &path_deceptive, &path_plagal];
    for path in all_paths {
        for (step_idx, step) in path.iter().enumerate() {
            hasher.update(&step.root_step.to_le_bytes());
            hasher.update(&(step.quality as u32).to_le_bytes());
            hasher.update(&step.tension.to_le_bytes());
            hasher.update(&step.voice_leading_cost.to_le_bytes());
            hasher.update(&(step.cadence_type as u32).to_le_bytes());
            for v in &step.voicing {
                hasher.update(&v.to_le_bytes());
            }

            {
                let _guard = AllocGuard::new();
                resolver.dispatch_step(step, step_idx, path.len(), &cadence_bus, &param_bus, ParamId(300));
            }

            let snap = cadence_bus.snapshot();
            assert_eq!(snap.current_root_step, step.root_step);
            assert_eq!(snap.current_quality, step.quality);
            assert_eq!(snap.predicted_cadence, step.cadence_type);
        }
    }

    // Test VoiceLeadingOptimizer directly
    let v0 = VoiceLeadingOptimizer::generate_initial_voicing(0, ChordQuality::Major);
    let (v1, cost1) = VoiceLeadingOptimizer::optimize_next_voicing(&v0, 5, ChordQuality::Major); // IV
    let (v2, cost2) = VoiceLeadingOptimizer::optimize_next_voicing(&v1, 7, ChordQuality::Dominant7); // V7
    let (v3, cost3) = VoiceLeadingOptimizer::optimize_next_voicing(&v2, 0, ChordQuality::Major); // I

    hasher.update(&cost1.to_le_bytes());
    hasher.update(&cost2.to_le_bytes());
    hasher.update(&cost3.to_le_bytes());
    for v in [&v0, &v1, &v2, &v3] {
        for note in *v {
            hasher.update(&note.to_le_bytes());
        }
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_cadence_path_resolver.hash", &digest);
}

#[cfg(feature = "gui")]
#[test]
fn test_golden_brass_cadence_views_snapshot() {
    let mut brass_view = WaveguideBrassView::new();
    brass_view.valve_state = [true, false, true];
    brass_view.update_physics_simulation();

    let ascii_brass = brass_view.render_ascii(80, 16);
    assert!(!ascii_brass.is_empty());

    let res_brass = brass_view.render_snapshot_png("scratch/renders/waveguide_brass_view.png", 800, 520);
    let _ = brass_view.render_snapshot_png("../../scratch/renders/waveguide_brass_view.png", 800, 520);
    assert!(res_brass.is_ok(), "Brass snapshot render failed: {:?}", res_brass);

    let mut cadence_view = CadenceFlowView::new();
    cadence_view.set_start_chord(2, ChordQuality::Minor);
    cadence_view.set_target_chord(0, ChordQuality::Major);
    cadence_view.step_forward();

    let ascii_cadence = cadence_view.render_ascii(80, 16);
    assert!(!ascii_cadence.is_empty());
    assert!(ascii_cadence[1].contains("HARMONIC CADENCE FLOW"));

    let res_cadence = cadence_view.render_snapshot_png("scratch/renders/cadence_flow_view.png", 800, 520);
    let _ = cadence_view.render_snapshot_png("../../scratch/renders/cadence_flow_view.png", 800, 520);
    assert!(res_cadence.is_ok(), "Cadence snapshot render failed: {:?}", res_cadence);

    let mut hasher = Hasher::new();
    for line in &ascii_brass {
        hasher.update(line.as_bytes());
    }
    for line in &ascii_cadence {
        hasher.update(line.as_bytes());
    }
    hasher.update(&brass_view.lip_tension_hz.to_le_bytes());
    hasher.update(&brass_view.blowing_pressure_kpa.to_le_bytes());
    hasher.update(&brass_view.bore_length_m.to_le_bytes());
    hasher.update(&cadence_view.start_root.to_le_bytes());
    hasher.update(&(cadence_view.start_quality as u32).to_le_bytes());
    hasher.update(&cadence_view.target_root.to_le_bytes());
    hasher.update(&(cadence_view.target_quality as u32).to_le_bytes());

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_brass_cadence_views.hash", &digest);
}
