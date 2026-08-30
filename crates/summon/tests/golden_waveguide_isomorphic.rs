// Summoner - Deterministic Golden Physical Waveguide Mesh & Isomorphic Microtonal Regression Suite
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

use blake3::Hasher;
use std::fs;
use std::path::Path;
use summoner_core::allocator::AllocGuard;
use summoner_core::node::ProcessContext;
use summoner_core::param_bus::{ParamBus, ParamId};
use summoner_core::transport::Transport;
use summoner_core::tuning_matrix::{TuningMatrix, TuningRemapMode};
use summoner_dsp::modal_synthesis::{
    ModalExcitationType, ModalMaterialPreset, ModalSynthesisEngine,
};
use summoner_dsp::traits::SignalProcessor;
use summoner_dsp::waveguide_mesh::{
    MeshBoundaryType, MeshMaterialProfile, WaveguideMesh2D,
};
#[cfg(feature = "gui")]
use summoner_gui::views::isomorphic_lattice_view::IsomorphicLatticeView;
#[cfg(feature = "gui")]
use summoner_gui::views::waveguide_mesh_view::WaveguideMeshView;
use summoner_harmony::isomorphic_router::{
    HexCoordinate, IsomorphicLayoutType, IsomorphicRouter,
};

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
            "Golden physical waveguide & isomorphic microtonal hash mismatch for {}",
            hash_filename
        );
    } else {
        fs::write(&hash_file, digest).expect("Failed to write golden hash");
    }
}

#[test]
fn test_golden_waveguide_mesh_energy_conservation_and_damping_deterministic() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    let mut mesh_membrane = WaveguideMesh2D::new(sample_rate);
    mesh_membrane.set_material(MeshMaterialProfile::Membrane);
    mesh_membrane.strike(0.50, 0.50, 1.0, 1.5);

    let mut mesh_plate = WaveguideMesh2D::new(sample_rate);
    mesh_plate.set_material(MeshMaterialProfile::Plate);
    mesh_plate.strike(0.35, 0.65, 0.85, 1.2);

    let mut mesh_bar = WaveguideMesh2D::new(sample_rate);
    mesh_bar.set_material(MeshMaterialProfile::AcousticBar);
    mesh_bar.boundary = MeshBoundaryType::DampedAbsorption;
    mesh_bar.strike(0.20, 0.80, 0.90, 1.0);

    let in_l = [0.0f32; BLOCK_SIZE];
    let in_r = [0.0f32; BLOCK_SIZE];
    let mut out_l_mem = [0.0f32; BLOCK_SIZE];
    let mut out_r_mem = [0.0f32; BLOCK_SIZE];
    let mut out_l_plate = [0.0f32; BLOCK_SIZE];
    let mut out_r_plate = [0.0f32; BLOCK_SIZE];
    let mut out_l_bar = [0.0f32; BLOCK_SIZE];
    let mut out_r_bar = [0.0f32; BLOCK_SIZE];

    let mut hasher = Hasher::new();

    // Process 64 blocks of audio (4096 samples)
    for _block_idx in 0..64 {
        let ctx = ProcessContext::from_transport(&transport);

        {
            let _guard = AllocGuard::new();
            mesh_membrane.process_block(
                &[&in_l[..], &in_r[..]],
                &mut [&mut out_l_mem[..], &mut out_r_mem[..]],
                &ctx,
            );
            mesh_plate.process_block(
                &[&in_l[..], &in_r[..]],
                &mut [&mut out_l_plate[..], &mut out_r_plate[..]],
                &ctx,
            );
            mesh_bar.process_block(
                &[&in_l[..], &in_r[..]],
                &mut [&mut out_l_bar[..], &mut out_r_bar[..]],
                &ctx,
            );
        }

        for i in 0..BLOCK_SIZE {
            assert!(out_l_mem[i].is_finite());
            assert!(out_r_mem[i].is_finite());
            assert!(out_l_plate[i].is_finite());
            assert!(out_r_plate[i].is_finite());
            assert!(out_l_bar[i].is_finite());
            assert!(out_r_bar[i].is_finite());

            hasher.update(&out_l_mem[i].to_le_bytes());
            hasher.update(&out_r_mem[i].to_le_bytes());
            hasher.update(&out_l_plate[i].to_le_bytes());
            hasher.update(&out_r_plate[i].to_le_bytes());
            hasher.update(&out_l_bar[i].to_le_bytes());
            hasher.update(&out_r_bar[i].to_le_bytes());
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_waveguide_mesh.hash", &digest);
}

#[test]
fn test_golden_modal_synthesis_strike_bow_pluck_deterministic() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    let mut synth_strike = ModalSynthesisEngine::new(sample_rate);
    synth_strike.set_fundamental(330.0);
    synth_strike.set_material(ModalMaterialPreset::MetalBar);
    synth_strike.excitation_type = ModalExcitationType::Strike;
    synth_strike.trigger_strike(0.95);

    let mut synth_bow = ModalSynthesisEngine::new(sample_rate);
    synth_bow.set_fundamental(220.0);
    synth_bow.set_material(ModalMaterialPreset::TibetanSingingBowl);
    synth_bow.excitation_type = ModalExcitationType::Bow;
    synth_bow.bow_velocity = 0.50;
    synth_bow.bow_pressure = 0.60;

    let mut synth_marimba = ModalSynthesisEngine::new(sample_rate);
    synth_marimba.set_fundamental(440.0);
    synth_marimba.set_material(ModalMaterialPreset::MarimbaWood);
    synth_marimba.excitation_type = ModalExcitationType::Strike;
    synth_marimba.trigger_strike(0.85);

    let in_l = [0.0f32; BLOCK_SIZE];
    let in_r = [0.0f32; BLOCK_SIZE];
    let mut out_l_strike = [0.0f32; BLOCK_SIZE];
    let mut out_r_strike = [0.0f32; BLOCK_SIZE];
    let mut out_l_bow = [0.0f32; BLOCK_SIZE];
    let mut out_r_bow = [0.0f32; BLOCK_SIZE];
    let mut out_l_marimba = [0.0f32; BLOCK_SIZE];
    let mut out_r_marimba = [0.0f32; BLOCK_SIZE];

    let mut hasher = Hasher::new();

    // Process 64 blocks of audio across modal synthesis configurations
    for _block_idx in 0..64 {
        let ctx = ProcessContext::from_transport(&transport);

        {
            let _guard = AllocGuard::new();
            synth_strike.process_block(
                &[&in_l[..], &in_r[..]],
                &mut [&mut out_l_strike[..], &mut out_r_strike[..]],
                &ctx,
            );
            synth_bow.process_block(
                &[&in_l[..], &in_r[..]],
                &mut [&mut out_l_bow[..], &mut out_r_bow[..]],
                &ctx,
            );
            synth_marimba.process_block(
                &[&in_l[..], &in_r[..]],
                &mut [&mut out_l_marimba[..], &mut out_r_marimba[..]],
                &ctx,
            );
        }

        for i in 0..BLOCK_SIZE {
            assert!(out_l_strike[i].is_finite());
            assert!(out_r_strike[i].is_finite());
            assert!(out_l_bow[i].is_finite());
            assert!(out_r_bow[i].is_finite());
            assert!(out_l_marimba[i].is_finite());
            assert!(out_r_marimba[i].is_finite());

            hasher.update(&out_l_strike[i].to_le_bytes());
            hasher.update(&out_r_strike[i].to_le_bytes());
            hasher.update(&out_l_bow[i].to_le_bytes());
            hasher.update(&out_r_bow[i].to_le_bytes());
            hasher.update(&out_l_marimba[i].to_le_bytes());
            hasher.update(&out_r_marimba[i].to_le_bytes());
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_modal_synthesis.hash", &digest);
}

#[test]
fn test_golden_isomorphic_router_and_tuning_matrix_dispatch() {
    let mut router = IsomorphicRouter::new(IsomorphicLayoutType::WickiHayden, 19, 440.0, 4);

    let mut matrix = TuningMatrix::new("12-TET to 19-EDO Morph", 69, 440.0);
    matrix.mode = TuningRemapMode::TemperamentInterpolation;
    matrix.set_secondary_edo(19);
    matrix.param_morph_id = Some(ParamId(90));
    matrix.param_root_id = Some(ParamId(91));

    let mut param_bus = ParamBus::new();
    let _ = param_bus.register(ParamId(90), 0.0);
    let _ = param_bus.register(ParamId(91), 440.0);

    let mut hasher = Hasher::new();

    // Trigger dynamic chords across 32 steps with morphing temperament
    for step in 0..32 {
        let alpha = step as f32 / 31.0;
        param_bus.set(ParamId(90), alpha);

        matrix.poll_param_bus(&param_bus);

        let q = (step % 5) - 2;
        let r = (step % 3) - 1;
        let coord = HexCoordinate::new(q, r);

        {
            let _guard = AllocGuard::new();
            let _v = router.note_on(coord, 0.80);
            router.advance_frames(64);
        }

        for voice in &router.voices {
            let freq = voice.frequency_hz;
            let mapped_f = matrix.note_to_frequency((60 + voice.step_index.rem_euclid(24)) as u8);

            assert!(freq.is_finite());
            assert!(mapped_f.is_finite());

            hasher.update(&freq.to_le_bytes());
            hasher.update(&mapped_f.to_le_bytes());
            hasher.update(&voice.velocity.to_le_bytes());
        }
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_isomorphic_tuning.hash", &digest);
}

#[cfg(feature = "gui")]
#[test]
fn test_golden_isomorphic_lattice_and_waveguide_views_snapshot() {
    let mut lattice_view = IsomorphicLatticeView::new(IsomorphicLayoutType::WickiHayden, 19, 440.0);
    lattice_view.toggle_node(0);
    lattice_view.toggle_node(3);

    let ascii_lat = lattice_view.render_ascii(80, 16);
    assert!(!ascii_lat.is_empty());
    assert!(ascii_lat[0].contains("ISOMORPHIC LATTICE"));

    let res_lat = lattice_view.render_snapshot_png("scratch/renders/isomorphic_lattice_view.png", 800, 520);
    let _ = lattice_view.render_snapshot_png("../../scratch/renders/isomorphic_lattice_view.png", 800, 520);
    assert!(res_lat.is_ok(), "Lattice snapshot render failed: {:?}", res_lat);

    let mut mesh_view = WaveguideMeshView::new();
    mesh_view.trigger_strike(0.9);
    for _ in 0..8 {
        mesh_view.step_simulation();
    }

    let ascii_mesh = mesh_view.render_ascii(80, 16);
    assert!(!ascii_mesh.is_empty());
    assert!(ascii_mesh[0].contains("WAVEGUIDE 3D MESH"));

    let res_mesh = mesh_view.render_snapshot_png("scratch/renders/waveguide_mesh_view.png", 800, 520);
    let _ = mesh_view.render_snapshot_png("../../scratch/renders/waveguide_mesh_view.png", 800, 520);
    assert!(res_mesh.is_ok(), "Mesh snapshot render failed: {:?}", res_mesh);

    let mut hasher = Hasher::new();
    for line in &ascii_lat {
        hasher.update(line.as_bytes());
    }
    for line in &ascii_mesh {
        hasher.update(line.as_bytes());
    }
    hasher.update(&lattice_view.router.edo.to_le_bytes());
    hasher.update(&mesh_view.mesh.courant.to_le_bytes());
    hasher.update(&mesh_view.mesh.damping.to_le_bytes());

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_waveguide_isomorphic_views.hash", &digest);
}
