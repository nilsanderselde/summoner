// Summoner - Deterministic Golden Bowed String Waveguide & Friction Dynamics Regression Suite
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

use blake3::Hasher;
use std::fs;
use std::path::Path;
use summoner_core::allocator::AllocGuard;
use summoner_core::articulation_bus::{ArticulationBus, BowingTechnique};
use summoner_core::node::ProcessContext;
use summoner_core::param_bus::{ParamBus, ParamId};
use summoner_core::transport::Transport;
use summoner_dsp::bowed_string::{BowedInstrument, BowedString};
use summoner_dsp::friction_model::{FrictionModel, RosinType};
use summoner_dsp::traits::SignalProcessor;
#[cfg(feature = "gui")]
use summoner_gui::views::bowed_string_view::BowedStringView;
#[cfg(feature = "gui")]
use summoner_gui::views::friction_orbit_view::FrictionOrbitView;
use summoner_sequencer::bowing_gesture::{
    BowingGestureEngine, BowingGesturePattern, BowingWaypoint, GestureInterpolationCurve,
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
            "Golden bowed string & friction dynamics hash mismatch for {}",
            hash_filename
        );
    } else {
        fs::write(&hash_file, digest).expect("Failed to write golden hash");
    }
}

#[test]
fn test_golden_bowed_string_helmholtz_oscillation_deterministic() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    // 1. Concert Violin with Legato stroke
    let mut violin = BowedString::new(sample_rate);
    violin.set_instrument(BowedInstrument::Violin);
    violin.set_frequency(440.0);
    violin.set_bow_velocity(0.45);
    violin.set_bow_force(1.25);
    violin.set_bow_position(0.12);
    violin.set_vibrato(0.25, 5.5);

    // 2. Concert Viola with Sul Ponticello overtones
    let mut viola = BowedString::new(sample_rate);
    viola.set_instrument(BowedInstrument::Viola);
    viola.set_frequency(220.0);
    viola.set_bow_velocity(0.30);
    viola.set_bow_force(0.85);
    viola.set_bow_position(0.04); // Sul Ponticello

    // 3. Concert Cello with Sul Tasto flute-like tone
    let mut cello = BowedString::new(sample_rate);
    cello.set_instrument(BowedInstrument::Cello);
    cello.set_frequency(110.0);
    cello.set_bow_velocity(0.50);
    cello.set_bow_force(0.60);
    cello.set_bow_position(0.28); // Sul Tasto

    // 4. Orchestral Double Bass with heavy bite
    let mut double_bass = BowedString::new(sample_rate);
    double_bass.set_instrument(BowedInstrument::DoubleBass);
    double_bass.set_frequency(55.0);
    double_bass.set_bow_velocity(0.70);
    double_bass.set_bow_force(2.80);
    double_bass.set_bow_position(0.10);

    let in_dummy = [0.0f32; BLOCK_SIZE];
    let mut out_violin = [0.0f32; BLOCK_SIZE];
    let mut out_viola = [0.0f32; BLOCK_SIZE];
    let mut out_cello = [0.0f32; BLOCK_SIZE];
    let mut out_bass = [0.0f32; BLOCK_SIZE];

    let mut hasher = Hasher::new();

    // Process 64 blocks of audio (4096 samples)
    for _block_idx in 0..64 {
        let ctx = ProcessContext::from_transport(&transport);

        {
            let _guard = AllocGuard::new();
            violin.process_block(&[&in_dummy[..]], &mut [&mut out_violin[..]], &ctx);
            viola.process_block(&[&in_dummy[..]], &mut [&mut out_viola[..]], &ctx);
            cello.process_block(&[&in_dummy[..]], &mut [&mut out_cello[..]], &ctx);
            double_bass.process_block(&[&in_dummy[..]], &mut [&mut out_bass[..]], &ctx);
        }

        for i in 0..BLOCK_SIZE {
            assert!(out_violin[i].is_finite());
            assert!(out_viola[i].is_finite());
            assert!(out_cello[i].is_finite());
            assert!(out_bass[i].is_finite());

            assert!((-1.0..=1.0).contains(&out_violin[i]));
            assert!((-1.0..=1.0).contains(&out_viola[i]));
            assert!((-1.0..=1.0).contains(&out_cello[i]));
            assert!((-1.0..=1.0).contains(&out_bass[i]));

            hasher.update(&out_violin[i].to_le_bytes());
            hasher.update(&out_viola[i].to_le_bytes());
            hasher.update(&out_cello[i].to_le_bytes());
            hasher.update(&out_bass[i].to_le_bytes());
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_bowed_string.hash", &digest);
}

#[test]
fn test_golden_friction_model_dynamics_deterministic() {
    let sample_rate = 48000;
    let mut model_violin = FrictionModel::new(sample_rate);
    model_violin.set_rosin(RosinType::LightViolin);

    let mut model_cello = FrictionModel::new(sample_rate);
    model_cello.set_rosin(RosinType::MediumCello);

    let mut model_bass = FrictionModel::new(sample_rate);
    model_bass.set_rosin(RosinType::DarkDoubleBass);

    let mut model_synth = FrictionModel::new(sample_rate);
    model_synth.set_rosin(RosinType::Synthetic);

    let mut hasher = Hasher::new();

    for step in 0..256 {
        let v_in_nut = (step as f32 * 0.05).sin() * 0.35;
        let v_in_bridge = (step as f32 * 0.07).cos() * 0.25;
        let bow_vel = 0.40 + 0.30 * (step as f32 * 0.03).sin();
        let bow_force = 1.00 + 0.80 * (step as f32 * 0.02).cos();

        let (res_v, res_c, res_b, res_s) = {
            let _guard = AllocGuard::new();
            (
                model_violin.solve(v_in_nut, v_in_bridge, bow_vel, bow_force),
                model_cello.solve(v_in_nut, v_in_bridge, bow_vel, bow_force),
                model_bass.solve(v_in_nut, v_in_bridge, bow_vel, bow_force),
                model_synth.solve(v_in_nut, v_in_bridge, bow_vel, bow_force),
            )
        };

        assert!(res_v.string_velocity.is_finite());
        assert!(res_c.string_velocity.is_finite());
        assert!(res_b.string_velocity.is_finite());
        assert!(res_s.string_velocity.is_finite());

        hasher.update(&res_v.string_velocity.to_le_bytes());
        hasher.update(&res_v.out_nut.to_le_bytes());
        hasher.update(&res_v.out_bridge.to_le_bytes());
        hasher.update(&res_v.friction_force.to_le_bytes());
        hasher.update(&[if res_v.is_sticking { 1u8 } else { 0u8 }]);

        hasher.update(&res_c.string_velocity.to_le_bytes());
        hasher.update(&res_c.out_nut.to_le_bytes());
        hasher.update(&res_c.out_bridge.to_le_bytes());
        hasher.update(&res_c.friction_force.to_le_bytes());
        hasher.update(&[if res_c.is_sticking { 1u8 } else { 0u8 }]);

        hasher.update(&res_b.string_velocity.to_le_bytes());
        hasher.update(&res_b.out_nut.to_le_bytes());
        hasher.update(&res_b.out_bridge.to_le_bytes());
        hasher.update(&res_b.friction_force.to_le_bytes());
        hasher.update(&[if res_b.is_sticking { 1u8 } else { 0u8 }]);

        hasher.update(&res_s.string_velocity.to_le_bytes());
        hasher.update(&res_s.out_nut.to_le_bytes());
        hasher.update(&res_s.out_bridge.to_le_bytes());
        hasher.update(&res_s.friction_force.to_le_bytes());
        hasher.update(&[if res_s.is_sticking { 1u8 } else { 0u8 }]);
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_friction_model.hash", &digest);
}

#[test]
fn test_golden_bowing_gesture_timeline_and_articulation_bus_deterministic() {
    let mut engine = BowingGestureEngine::new();
    engine.pattern = BowingGesturePattern::CustomSpline;
    engine.interpolation = GestureInterpolationCurve::CubicCatmullRom;
    engine.loop_length_beats = Some(8.0);

    engine.add_waypoint(BowingWaypoint {
        beat: 0.0,
        bow_velocity_mps: 0.25,
        bow_force_n: 0.90,
        bridge_proximity_beta: 0.04,
        technique: BowingTechnique::SulPonticello,
        rosin_adhesion: 0.85,
        string_damping: 0.10,
        vibrato_depth: 0.0,
        vibrato_rate_hz: 5.5,
    });

    engine.add_waypoint(BowingWaypoint {
        beat: 2.0,
        bow_velocity_mps: 0.65,
        bow_force_n: 2.20,
        bridge_proximity_beta: 0.12,
        technique: BowingTechnique::Martele,
        rosin_adhesion: 0.95,
        string_damping: 0.15,
        vibrato_depth: 0.10,
        vibrato_rate_hz: 6.0,
    });

    engine.add_waypoint(BowingWaypoint {
        beat: 4.0,
        bow_velocity_mps: 0.40,
        bow_force_n: 0.70,
        bridge_proximity_beta: 0.28,
        technique: BowingTechnique::SulTasto,
        rosin_adhesion: 0.75,
        string_damping: 0.20,
        vibrato_depth: 0.30,
        vibrato_rate_hz: 5.0,
    });

    engine.add_waypoint(BowingWaypoint {
        beat: 6.0,
        bow_velocity_mps: 0.80,
        bow_force_n: 1.10,
        bridge_proximity_beta: 0.08,
        technique: BowingTechnique::Tremolo,
        rosin_adhesion: 0.88,
        string_damping: 0.12,
        vibrato_depth: 0.0,
        vibrato_rate_hz: 7.0,
    });

    engine.add_waypoint(BowingWaypoint {
        beat: 8.0,
        bow_velocity_mps: 0.25,
        bow_force_n: 0.90,
        bridge_proximity_beta: 0.04,
        technique: BowingTechnique::SulPonticello,
        rosin_adhesion: 0.85,
        string_damping: 0.10,
        vibrato_depth: 0.0,
        vibrato_rate_hz: 5.5,
    });

    let articulation_bus = ArticulationBus::new();
    let mut param_bus = ParamBus::new();
    for i in 0..20 {
        param_bus.register(ParamId(700 + i), 0.0);
    }

    let mut hasher = Hasher::new();

    // Evaluate gesture across 64 steps
    for step in 0..=64 {
        let beat = step as f64 * (8.0 / 64.0);
        let snap = engine.evaluate_beat(beat);

        {
            let _guard = AllocGuard::new();
            engine.dispatch_to_articulation_bus(beat, &articulation_bus);
            engine.dispatch_to_param_bus(beat, &param_bus, ParamId(700));
        }

        let read_snap = articulation_bus.snapshot();
        assert_eq!(read_snap.technique, snap.technique);
        assert!((read_snap.bow_velocity_mps - snap.bow_velocity_mps).abs() < 1e-4);
        assert!((read_snap.bow_force_n - snap.bow_force_n).abs() < 1e-4);

        hasher.update(&(snap.technique as u32).to_le_bytes());
        hasher.update(&snap.bow_velocity_mps.to_le_bytes());
        hasher.update(&snap.bow_force_n.to_le_bytes());
        hasher.update(&snap.bridge_proximity_beta.to_le_bytes());
        hasher.update(&snap.rosin_adhesion.to_le_bytes());
        hasher.update(&snap.string_damping.to_le_bytes());
        hasher.update(&snap.vibrato_depth.to_le_bytes());
        hasher.update(&snap.vibrato_rate_hz.to_le_bytes());
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_bowing_gesture.hash", &digest);
}

#[cfg(feature = "gui")]
#[test]
fn test_golden_bowed_string_and_friction_orbit_views_snapshot() {
    let mut bowed_view = BowedStringView::new();
    bowed_view.bow_speed_mps = 0.55;
    bowed_view.bow_force_n = 1.65;
    bowed_view.bridge_proximity_beta = 0.10;
    bowed_view.update_physics_simulation();

    let ascii_bowed = bowed_view.render_ascii(80, 16);
    assert!(!ascii_bowed.is_empty());

    let res_bowed = bowed_view.render_snapshot_png("scratch/renders/bowed_string_view.png", 800, 520);
    let _ = bowed_view.render_snapshot_png("../../scratch/renders/bowed_string_view.png", 800, 520);
    let _ = bowed_view.render_snapshot_png("C:/Users/Nils/Code/Summoner/scratch/renders/bowed_string_view.png", 800, 520);
    assert!(res_bowed.is_ok(), "Bowed string snapshot render failed: {:?}", res_bowed);

    let mut orbit_view = FrictionOrbitView::new();
    orbit_view.bow_velocity_mps = 0.50;
    orbit_view.bow_force_n = 1.40;
    orbit_view.bridge_proximity_beta = 0.08;

    let ascii_orbit = orbit_view.render_ascii(80, 16);
    assert!(!ascii_orbit.is_empty());

    let res_orbit = orbit_view.render_snapshot_png("scratch/renders/friction_orbit_view.png", 800, 520);
    let _ = orbit_view.render_snapshot_png("../../scratch/renders/friction_orbit_view.png", 800, 520);
    let _ = orbit_view.render_snapshot_png("C:/Users/Nils/Code/Summoner/scratch/renders/friction_orbit_view.png", 800, 520);
    assert!(res_orbit.is_ok(), "Friction orbit snapshot render failed: {:?}", res_orbit);

    let mut hasher = Hasher::new();
    for line in &ascii_bowed {
        hasher.update(line.as_bytes());
    }
    for line in &ascii_orbit {
        hasher.update(line.as_bytes());
    }
    hasher.update(&bowed_view.bow_speed_mps.to_le_bytes());
    hasher.update(&bowed_view.bow_force_n.to_le_bytes());
    hasher.update(&bowed_view.bridge_proximity_beta.to_le_bytes());
    hasher.update(&orbit_view.bow_velocity_mps.to_le_bytes());
    hasher.update(&orbit_view.bow_force_n.to_le_bytes());
    hasher.update(&orbit_view.bridge_proximity_beta.to_le_bytes());

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_bowed_views.hash", &digest);
}
