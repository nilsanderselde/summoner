// Summoner - Deterministic Golden Vocal Tract FDTD & Vowel Formant Regression Suite
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

use blake3::Hasher;
use std::fs;
use std::path::Path;
use summoner_core::allocator::AllocGuard;
use summoner_core::formant_bus::FormantBus;
#[cfg(feature = "gui")]
use summoner_core::formant_bus::IpaVowel;
use summoner_core::node::ProcessContext;
use summoner_core::param_bus::{ParamBus, ParamId};
use summoner_core::transport::Transport;
use summoner_dsp::glottal_pulse::{GlottalPulse, GlottalVoicingMode};
use summoner_dsp::traits::SignalProcessor;
use summoner_dsp::vocal_tract::{VocalTract, VowelPreset};
#[cfg(feature = "gui")]
use summoner_gui::views::vocal_tract_view::VocalTractView;
#[cfg(feature = "gui")]
use summoner_gui::views::vowel_space_view::VowelSpaceView;
use summoner_harmony::vowel_graph::VowelGraph;

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
            "Golden vocal tract & vowel formant hash mismatch for {}",
            hash_filename
        );
    } else {
        fs::write(&hash_file, digest).expect("Failed to write golden hash");
    }
}

#[test]
fn test_golden_glottal_pulse_oscillation_deterministic() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    let mut pulse_modal = GlottalPulse::new(sample_rate);
    pulse_modal.set_frequency(130.81); // C3
    pulse_modal.set_mode(GlottalVoicingMode::Modal);

    let mut pulse_falsetto = GlottalPulse::new(sample_rate);
    pulse_falsetto.set_frequency(261.63); // C4
    pulse_falsetto.set_mode(GlottalVoicingMode::Falsetto);

    let mut pulse_creaky = GlottalPulse::new(sample_rate);
    pulse_creaky.set_frequency(65.41); // C2
    pulse_creaky.set_mode(GlottalVoicingMode::Creaky);

    let mut pulse_breathy = GlottalPulse::new(sample_rate);
    pulse_breathy.set_frequency(196.00); // G3
    pulse_breathy.set_mode(GlottalVoicingMode::Breathy);

    let mut pulse_whisper = GlottalPulse::new(sample_rate);
    pulse_whisper.set_mode(GlottalVoicingMode::Whisper);

    let dummy_in: [&[f32]; 0] = [];
    let mut out_m = [0.0f32; BLOCK_SIZE];
    let mut out_f = [0.0f32; BLOCK_SIZE];
    let mut out_c = [0.0f32; BLOCK_SIZE];
    let mut out_b = [0.0f32; BLOCK_SIZE];
    let mut out_w = [0.0f32; BLOCK_SIZE];

    let mut hasher = Hasher::new();

    for _block_idx in 0..64 {
        let ctx = ProcessContext::from_transport(&transport);

        {
            let _guard = AllocGuard::new();
            pulse_modal.process_block(&dummy_in, &mut [&mut out_m[..]], &ctx);
            pulse_falsetto.process_block(&dummy_in, &mut [&mut out_f[..]], &ctx);
            pulse_creaky.process_block(&dummy_in, &mut [&mut out_c[..]], &ctx);
            pulse_breathy.process_block(&dummy_in, &mut [&mut out_b[..]], &ctx);
            pulse_whisper.process_block(&dummy_in, &mut [&mut out_w[..]], &ctx);
        }

        for i in 0..BLOCK_SIZE {
            assert!(out_m[i].is_finite());
            assert!(out_f[i].is_finite());
            assert!(out_c[i].is_finite());
            assert!(out_b[i].is_finite());
            assert!(out_w[i].is_finite());

            assert!((-1.0..=1.0).contains(&out_m[i]));
            assert!((-1.0..=1.0).contains(&out_f[i]));
            assert!((-1.0..=1.0).contains(&out_c[i]));
            assert!((-1.0..=1.0).contains(&out_b[i]));
            assert!((-1.0..=1.0).contains(&out_w[i]));

            hasher.update(&out_m[i].to_le_bytes());
            hasher.update(&out_f[i].to_le_bytes());
            hasher.update(&out_c[i].to_le_bytes());
            hasher.update(&out_b[i].to_le_bytes());
            hasher.update(&out_w[i].to_le_bytes());
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_glottal_pulse.hash", &digest);
}

#[test]
fn test_golden_vocal_tract_fdtd_resonance_deterministic() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    let mut tract_i = VocalTract::new(sample_rate);
    tract_i.set_vowel_preset(VowelPreset::I);
    tract_i.glottis.set_frequency(150.0);

    let mut tract_a = VocalTract::new(sample_rate);
    tract_a.set_vowel_preset(VowelPreset::A);
    tract_a.glottis.set_frequency(150.0);

    let mut tract_u = VocalTract::new(sample_rate);
    tract_u.set_vowel_preset(VowelPreset::U);
    tract_u.glottis.set_frequency(150.0);

    let mut tract_nasal = VocalTract::new(sample_rate);
    tract_nasal.set_vowel_preset(VowelPreset::A);
    tract_nasal.set_velum_opening(0.85); // Heavy nasal coupling
    tract_nasal.glottis.set_frequency(150.0);

    let in_ext = [0.0f32; BLOCK_SIZE];
    let mut out_i = [0.0f32; BLOCK_SIZE];
    let mut out_a = [0.0f32; BLOCK_SIZE];
    let mut out_u = [0.0f32; BLOCK_SIZE];
    let mut out_nasal = [0.0f32; BLOCK_SIZE];

    let mut hasher = Hasher::new();

    for _block_idx in 0..64 {
        let ctx = ProcessContext::from_transport(&transport);

        {
            let _guard = AllocGuard::new();
            tract_i.process_block(&[&in_ext[..]], &mut [&mut out_i[..]], &ctx);
            tract_a.process_block(&[&in_ext[..]], &mut [&mut out_a[..]], &ctx);
            tract_u.process_block(&[&in_ext[..]], &mut [&mut out_u[..]], &ctx);
            tract_nasal.process_block(&[&in_ext[..]], &mut [&mut out_nasal[..]], &ctx);
        }

        for i in 0..BLOCK_SIZE {
            assert!(out_i[i].is_finite());
            assert!(out_a[i].is_finite());
            assert!(out_u[i].is_finite());
            assert!(out_nasal[i].is_finite());

            assert!((-1.0..=1.0).contains(&out_i[i]));
            assert!((-1.0..=1.0).contains(&out_a[i]));
            assert!((-1.0..=1.0).contains(&out_u[i]));
            assert!((-1.0..=1.0).contains(&out_nasal[i]));

            hasher.update(&out_i[i].to_le_bytes());
            hasher.update(&out_a[i].to_le_bytes());
            hasher.update(&out_u[i].to_le_bytes());
            hasher.update(&out_nasal[i].to_le_bytes());
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_vocal_tract.hash", &digest);
}

#[test]
fn test_golden_vowel_graph_catmull_rom_and_formant_bus_deterministic() {
    let graph = VowelGraph::build_standard_ipa_graph();
    let formant_bus = FormantBus::new();
    let mut param_bus = ParamBus::new();
    for i in 0..20 {
        param_bus.register(ParamId(600 + i), 0.0);
    }

    let mut hasher = Hasher::new();

    // Sweep continuous morph trajectory from 0.0 to 1.0
    for step in 0..=32 {
        let t = step as f32 / 32.0;
        let pt = graph.evaluate_trajectory(t);

        for f in &pt.formants_hz {
            hasher.update(&f.to_le_bytes());
        }
        for b in &pt.bandwidths_hz {
            hasher.update(&b.to_le_bytes());
        }
        hasher.update(&pt.tongue_position.to_le_bytes());
        hasher.update(&pt.tongue_height.to_le_bytes());
        hasher.update(&pt.lip_opening.to_le_bytes());
        hasher.update(&pt.velum_opening.to_le_bytes());
        hasher.update(&(pt.nearest_vowel as u32).to_le_bytes());
        for a in &pt.cylinder_areas {
            hasher.update(&a.to_le_bytes());
        }

        {
            let _guard = AllocGuard::new();
            graph.dispatch_trajectory_point(t, &formant_bus, &param_bus, ParamId(600));
        }

        let snap = formant_bus.snapshot();
        assert_eq!(snap.active_vowel, pt.nearest_vowel);
        assert!((snap.trajectory_progress - t).abs() < 1e-4);
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_vowel_formant.hash", &digest);
}

#[cfg(feature = "gui")]
#[test]
fn test_golden_vocal_tract_and_vowel_space_views_snapshot() {
    let mut tract_view = VocalTractView::new();
    tract_view.set_vowel_preset(VowelPreset::A);
    tract_view.velum_opening = 0.45;
    tract_view.update_areas();

    let ascii_tract = tract_view.render_ascii(80, 16);
    assert!(!ascii_tract.is_empty());

    let res_tract = tract_view.render_snapshot_png("scratch/renders/vocal_tract_view.png", 800, 520);
    let _ = tract_view.render_snapshot_png("../../scratch/renders/vocal_tract_view.png", 800, 520);
    let _ = tract_view.render_snapshot_png("C:/Users/Nils/Code/Summoner/scratch/renders/vocal_tract_view.png", 800, 520);
    assert!(res_tract.is_ok(), "Vocal tract snapshot render failed: {:?}", res_tract);

    let mut vowel_view = VowelSpaceView::new();
    vowel_view.set_vowel(IpaVowel::OpenFrontA);
    vowel_view.step_morph(0.5);

    let ascii_vowel = vowel_view.render_ascii(80, 16);
    assert!(!ascii_vowel.is_empty());

    let res_vowel = vowel_view.render_snapshot_png("scratch/renders/vowel_space_view.png", 800, 520);
    let _ = vowel_view.render_snapshot_png("../../scratch/renders/vowel_space_view.png", 800, 520);
    let _ = vowel_view.render_snapshot_png("C:/Users/Nils/Code/Summoner/scratch/renders/vowel_space_view.png", 800, 520);
    assert!(res_vowel.is_ok(), "Vowel space snapshot render failed: {:?}", res_vowel);

    let mut hasher = Hasher::new();
    for line in &ascii_tract {
        hasher.update(line.as_bytes());
    }
    for line in &ascii_vowel {
        hasher.update(line.as_bytes());
    }
    hasher.update(&tract_view.tongue_position.to_le_bytes());
    hasher.update(&tract_view.tongue_height.to_le_bytes());
    hasher.update(&tract_view.lip_opening.to_le_bytes());
    hasher.update(&vowel_view.f1_hz.to_le_bytes());
    hasher.update(&vowel_view.f2_hz.to_le_bytes());
    hasher.update(&vowel_view.morph_progress.to_le_bytes());

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_vocal_views.hash", &digest);
}
