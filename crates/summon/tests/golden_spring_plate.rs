// Summoner - Deterministic Golden Spring-Mass Lattice & Mechanical Plate Reverb Regression Suite
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

use blake3::Hasher;
use std::fs;
use std::path::Path;
use summoner_core::allocator::AllocGuard;
use summoner_core::node::ProcessContext;
use summoner_core::param_bus::{ParamBus, ParamId};
use summoner_core::spring_bus::{SpringArticulation, SpringBus};
use summoner_core::transport::Transport;
use summoner_dsp::plate_tank::{PlateReverbProfile, PlateTank};
use summoner_dsp::spring_lattice::{SpringLattice, SpringLatticeProfile};
use summoner_dsp::traits::SignalProcessor;
#[cfg(feature = "gui")]
use summoner_gui::views::plate_dispersion_view::PlateDispersionView;
#[cfg(feature = "gui")]
use summoner_gui::views::spring_lattice_view::SpringLatticeView;
use summoner_sequencer::spring_gesture::{
    SpringGestureEngine, SpringGesturePattern, SpringInterpolationCurve, SpringWaypoint,
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
            "Golden spring-mass lattice & plate reverb hash mismatch for {}",
            hash_filename
        );
    } else {
        fs::write(&hash_file, digest).expect("Failed to write golden hash");
    }
}

#[test]
fn test_golden_spring_lattice_deterministic() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    // 1. Vintage Accutronics Spring Tank G2
    let mut accutronics = SpringLattice::new(sample_rate);
    accutronics.set_profile(SpringLatticeProfile::VintageAccutronicsSpring);
    accutronics.set_frequency(98.0);
    accutronics.trigger_strike(0.90, 0.50);

    // 2. Studio Plate Suspension E3
    let mut studio_plate = SpringLattice::new(sample_rate);
    studio_plate.set_profile(SpringLatticeProfile::StudioPlateSuspension);
    studio_plate.set_frequency(164.81);
    studio_plate.trigger_strike(0.85, 0.60);

    // 3. Dual Helical Spring Tank A2
    let mut dual_helical = SpringLattice::new(sample_rate);
    dual_helical.set_profile(SpringLatticeProfile::DualHelicalSpringTank);
    dual_helical.set_frequency(110.0);
    dual_helical.trigger_strike(0.80, 0.40);

    // 4. Resonant Helical Coil A3
    let mut resonant_coil = SpringLattice::new(sample_rate);
    resonant_coil.set_profile(SpringLatticeProfile::ResonantHelicalCoil);
    resonant_coil.set_frequency(220.0);
    resonant_coil.trigger_strike(0.95, 0.70);

    // 5. Non-Linear Shaker Table C2
    let mut shaker = SpringLattice::new(sample_rate);
    shaker.set_profile(SpringLatticeProfile::NonlinearShakerTable);
    shaker.set_frequency(65.41);
    shaker.trigger_strike(1.00, 0.85);

    // 6. Multi-Axis Spring Lattice C3
    let mut multi_axis = SpringLattice::new(sample_rate);
    multi_axis.set_profile(SpringLatticeProfile::MultiAxisSpringLattice);
    multi_axis.set_frequency(130.81);
    multi_axis.trigger_strike(0.75, 0.35);

    let in_dummy = [0.0f32; BLOCK_SIZE];
    let mut out_acc_l = [0.0f32; BLOCK_SIZE];
    let mut out_acc_r = [0.0f32; BLOCK_SIZE];
    let mut out_stu_l = [0.0f32; BLOCK_SIZE];
    let mut out_stu_r = [0.0f32; BLOCK_SIZE];
    let mut out_dual_l = [0.0f32; BLOCK_SIZE];
    let mut out_dual_r = [0.0f32; BLOCK_SIZE];
    let mut out_res_l = [0.0f32; BLOCK_SIZE];
    let mut out_res_r = [0.0f32; BLOCK_SIZE];
    let mut out_shk_l = [0.0f32; BLOCK_SIZE];
    let mut out_shk_r = [0.0f32; BLOCK_SIZE];
    let mut out_max_l = [0.0f32; BLOCK_SIZE];
    let mut out_max_r = [0.0f32; BLOCK_SIZE];

    let mut hasher = Hasher::new();

    // Process 64 blocks of audio (4096 samples)
    for _block_idx in 0..64 {
        let ctx = ProcessContext::from_transport(&transport);

        {
            let _guard = AllocGuard::new();
            accutronics.process_block(&[&in_dummy[..]], &mut [&mut out_acc_l[..], &mut out_acc_r[..]], &ctx);
            studio_plate.process_block(&[&in_dummy[..]], &mut [&mut out_stu_l[..], &mut out_stu_r[..]], &ctx);
            dual_helical.process_block(&[&in_dummy[..]], &mut [&mut out_dual_l[..], &mut out_dual_r[..]], &ctx);
            resonant_coil.process_block(&[&in_dummy[..]], &mut [&mut out_res_l[..], &mut out_res_r[..]], &ctx);
            shaker.process_block(&[&in_dummy[..]], &mut [&mut out_shk_l[..], &mut out_shk_r[..]], &ctx);
            multi_axis.process_block(&[&in_dummy[..]], &mut [&mut out_max_l[..], &mut out_max_r[..]], &ctx);
        }

        for i in 0..BLOCK_SIZE {
            assert!(out_acc_l[i].is_finite() && out_acc_r[i].is_finite());
            assert!(out_stu_l[i].is_finite() && out_stu_r[i].is_finite());
            assert!(out_dual_l[i].is_finite() && out_dual_r[i].is_finite());
            assert!(out_res_l[i].is_finite() && out_res_r[i].is_finite());
            assert!(out_shk_l[i].is_finite() && out_shk_r[i].is_finite());
            assert!(out_max_l[i].is_finite() && out_max_r[i].is_finite());

            assert!((-1.0..=1.0).contains(&out_acc_l[i]) && (-1.0..=1.0).contains(&out_acc_r[i]));
            assert!((-1.0..=1.0).contains(&out_stu_l[i]) && (-1.0..=1.0).contains(&out_stu_r[i]));
            assert!((-1.0..=1.0).contains(&out_dual_l[i]) && (-1.0..=1.0).contains(&out_dual_r[i]));
            assert!((-1.0..=1.0).contains(&out_res_l[i]) && (-1.0..=1.0).contains(&out_res_r[i]));
            assert!((-1.0..=1.0).contains(&out_shk_l[i]) && (-1.0..=1.0).contains(&out_shk_r[i]));
            assert!((-1.0..=1.0).contains(&out_max_l[i]) && (-1.0..=1.0).contains(&out_max_r[i]));

            hasher.update(&out_acc_l[i].to_le_bytes());
            hasher.update(&out_acc_r[i].to_le_bytes());
            hasher.update(&out_stu_l[i].to_le_bytes());
            hasher.update(&out_stu_r[i].to_le_bytes());
            hasher.update(&out_dual_l[i].to_le_bytes());
            hasher.update(&out_dual_r[i].to_le_bytes());
            hasher.update(&out_res_l[i].to_le_bytes());
            hasher.update(&out_res_r[i].to_le_bytes());
            hasher.update(&out_shk_l[i].to_le_bytes());
            hasher.update(&out_shk_r[i].to_le_bytes());
            hasher.update(&out_max_l[i].to_le_bytes());
            hasher.update(&out_max_r[i].to_le_bytes());
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_spring_lattice.hash", &digest);
}

#[test]
fn test_golden_plate_tank_dispersion_deterministic() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    // 1. Vintage EMT 140 Cold-Rolled Steel Plate
    let mut emt140 = PlateTank::new(sample_rate);
    emt140.set_profile(PlateReverbProfile::VintageEmt140Steel);
    emt140.wet_mix = 1.0;
    emt140.trigger_impulse(0.90);

    // 2. Studio Suspension Plate
    let mut studio_plate = PlateTank::new(sample_rate);
    studio_plate.set_profile(PlateReverbProfile::StudioPlateSuspension);
    studio_plate.wet_mix = 1.0;
    studio_plate.trigger_impulse(0.85);

    // 3. EMT 240 Gold Foil Diaphragm
    let mut gold_foil = PlateTank::new(sample_rate);
    gold_foil.set_profile(PlateReverbProfile::GoldFoilPlate);
    gold_foil.wet_mix = 1.0;
    gold_foil.trigger_impulse(0.80);

    // 4. Compact Mechanical Tank
    let mut compact = PlateTank::new(sample_rate);
    compact.set_profile(PlateReverbProfile::CompactMechanicalTank);
    compact.wet_mix = 1.0;
    compact.trigger_impulse(0.95);

    // 5. High-Tension Resonator Plate
    let mut high_tension = PlateTank::new(sample_rate);
    high_tension.set_profile(PlateReverbProfile::HighTensionResonator);
    high_tension.wet_mix = 1.0;
    high_tension.trigger_impulse(0.75);

    let in_dummy = [0.0f32; BLOCK_SIZE];
    let mut out_emt_l = [0.0f32; BLOCK_SIZE];
    let mut out_emt_r = [0.0f32; BLOCK_SIZE];
    let mut out_stu_l = [0.0f32; BLOCK_SIZE];
    let mut out_stu_r = [0.0f32; BLOCK_SIZE];
    let mut out_gold_l = [0.0f32; BLOCK_SIZE];
    let mut out_gold_r = [0.0f32; BLOCK_SIZE];
    let mut out_cmp_l = [0.0f32; BLOCK_SIZE];
    let mut out_cmp_r = [0.0f32; BLOCK_SIZE];
    let mut out_ht_l = [0.0f32; BLOCK_SIZE];
    let mut out_ht_r = [0.0f32; BLOCK_SIZE];

    let mut hasher = Hasher::new();

    // Process 64 blocks of audio (4096 samples)
    for _block_idx in 0..64 {
        let ctx = ProcessContext::from_transport(&transport);

        {
            let _guard = AllocGuard::new();
            emt140.process_block(&[&in_dummy[..]], &mut [&mut out_emt_l[..], &mut out_emt_r[..]], &ctx);
            studio_plate.process_block(&[&in_dummy[..]], &mut [&mut out_stu_l[..], &mut out_stu_r[..]], &ctx);
            gold_foil.process_block(&[&in_dummy[..]], &mut [&mut out_gold_l[..], &mut out_gold_r[..]], &ctx);
            compact.process_block(&[&in_dummy[..]], &mut [&mut out_cmp_l[..], &mut out_cmp_r[..]], &ctx);
            high_tension.process_block(&[&in_dummy[..]], &mut [&mut out_ht_l[..], &mut out_ht_r[..]], &ctx);
        }

        for i in 0..BLOCK_SIZE {
            assert!(out_emt_l[i].is_finite() && out_emt_r[i].is_finite());
            assert!(out_stu_l[i].is_finite() && out_stu_r[i].is_finite());
            assert!(out_gold_l[i].is_finite() && out_gold_r[i].is_finite());
            assert!(out_cmp_l[i].is_finite() && out_cmp_r[i].is_finite());
            assert!(out_ht_l[i].is_finite() && out_ht_r[i].is_finite());

            assert!((-1.0..=1.0).contains(&out_emt_l[i]) && (-1.0..=1.0).contains(&out_emt_r[i]));
            assert!((-1.0..=1.0).contains(&out_stu_l[i]) && (-1.0..=1.0).contains(&out_stu_r[i]));
            assert!((-1.0..=1.0).contains(&out_gold_l[i]) && (-1.0..=1.0).contains(&out_gold_r[i]));
            assert!((-1.0..=1.0).contains(&out_cmp_l[i]) && (-1.0..=1.0).contains(&out_cmp_r[i]));
            assert!((-1.0..=1.0).contains(&out_ht_l[i]) && (-1.0..=1.0).contains(&out_ht_r[i]));

            hasher.update(&out_emt_l[i].to_le_bytes());
            hasher.update(&out_emt_r[i].to_le_bytes());
            hasher.update(&out_stu_l[i].to_le_bytes());
            hasher.update(&out_stu_r[i].to_le_bytes());
            hasher.update(&out_gold_l[i].to_le_bytes());
            hasher.update(&out_gold_r[i].to_le_bytes());
            hasher.update(&out_cmp_l[i].to_le_bytes());
            hasher.update(&out_cmp_r[i].to_le_bytes());
            hasher.update(&out_ht_l[i].to_le_bytes());
            hasher.update(&out_ht_r[i].to_le_bytes());
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_plate_tank.hash", &digest);
}

#[test]
fn test_golden_spring_gesture_and_spring_bus_deterministic() {
    let mut engine = SpringGestureEngine::new(SpringGesturePattern::CustomSpline);
    engine.default_curve = SpringInterpolationCurve::CubicCatmullRom;
    engine.loop_length_beats = 8.0;
    engine.loop_enabled = true;

    engine.waypoints = vec![
        SpringWaypoint {
            beat: 0.0,
            drive_force: 0.80,
            dispersion_factor: 0.60,
            boundary_tension: 0.85,
            spring_nonlinearity: 0.20,
            transducer_pickup_angle: 0.0,
            damper_damping: 0.10,
            decay_t60_sec: 3.5,
            articulation: SpringArticulation::DriverDrive,
            curve: SpringInterpolationCurve::CubicCatmullRom,
        },
        SpringWaypoint {
            beat: 2.0,
            drive_force: 1.00,
            dispersion_factor: 0.85,
            boundary_tension: 0.60,
            spring_nonlinearity: 0.90,
            transducer_pickup_angle: std::f32::consts::FRAC_PI_2,
            damper_damping: 0.15,
            decay_t60_sec: 4.8,
            articulation: SpringArticulation::NonLinearSaturate,
            curve: SpringInterpolationCurve::CubicCatmullRom,
        },
        SpringWaypoint {
            beat: 4.0,
            drive_force: 0.95,
            dispersion_factor: 0.70,
            boundary_tension: 0.75,
            spring_nonlinearity: 0.50,
            transducer_pickup_angle: std::f32::consts::PI,
            damper_damping: 0.08,
            decay_t60_sec: 4.0,
            articulation: SpringArticulation::SpringPluck,
            curve: SpringInterpolationCurve::CubicCatmullRom,
        },
        SpringWaypoint {
            beat: 6.0,
            drive_force: 0.40,
            dispersion_factor: 0.40,
            boundary_tension: 0.95,
            spring_nonlinearity: 0.10,
            transducer_pickup_angle: 3.0 * std::f32::consts::FRAC_PI_2,
            damper_damping: 0.80,
            decay_t60_sec: 0.8,
            articulation: SpringArticulation::DamperMute,
            curve: SpringInterpolationCurve::CubicCatmullRom,
        },
    ];

    let bus = SpringBus::new();
    let mut param_bus = ParamBus::new();
    for i in 0..12 {
        param_bus.register(ParamId(800 + i), 0.0);
    }

    let mut hasher = Hasher::new();

    for step in 0..160 {
        let beat = step as f64 * 0.05; // 8 beats total

        let snap = {
            let _guard = AllocGuard::new();
            engine.dispatch_to_bus(beat, &bus);
            engine.dispatch_to_param_bus(beat, ParamId(800), &param_bus);
            bus.snapshot()
        };

        assert!(snap.drive_force.is_finite());
        assert!(snap.dispersion_factor.is_finite());
        assert!(snap.boundary_tension.is_finite());
        assert!(snap.spring_nonlinearity.is_finite());
        assert!(snap.transducer_pickup_angle.is_finite());
        assert!(snap.damper_damping.is_finite());
        assert!(snap.decay_t60_sec.is_finite());

        hasher.update(&(snap.articulation as u32).to_le_bytes());
        hasher.update(&snap.drive_force.to_le_bytes());
        hasher.update(&snap.dispersion_factor.to_le_bytes());
        hasher.update(&snap.boundary_tension.to_le_bytes());
        hasher.update(&snap.spring_nonlinearity.to_le_bytes());
        hasher.update(&snap.transducer_pickup_angle.to_le_bytes());
        hasher.update(&snap.damper_damping.to_le_bytes());
        hasher.update(&snap.decay_t60_sec.to_le_bytes());
        hasher.update(&snap.gesture_progress.to_le_bytes());
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_spring_gesture.hash", &digest);
}

#[cfg(feature = "gui")]
#[test]
fn test_golden_spring_plate_views_ascii_and_snapshots() {
    let spring_view = SpringLatticeView::new();
    let plate_view = PlateDispersionView::new();

    let spring_ascii = spring_view.render_ascii(64, 16);
    let plate_ascii = plate_view.render_ascii(64, 16);

    let mut hasher = Hasher::new();
    for line in &spring_ascii {
        hasher.update(line.as_bytes());
    }
    for line in &plate_ascii {
        hasher.update(line.as_bytes());
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_spring_plate_views.hash", &digest);

    // Save PNG snapshots across the project render directories
    let _ = spring_view.render_snapshot_png("scratch/renders/spring_lattice_view.png", 800, 520);
    let _ = spring_view.render_snapshot_png("crates/summon/scratch/renders/spring_lattice_view.png", 800, 520);
    let _ = spring_view.render_snapshot_png("crates/summoner_gui/scratch/renders/spring_lattice_view.png", 800, 520);

    let _ = plate_view.render_snapshot_png("scratch/renders/plate_dispersion_view.png", 800, 520);
    let _ = plate_view.render_snapshot_png("crates/summon/scratch/renders/plate_dispersion_view.png", 800, 520);
    let _ = plate_view.render_snapshot_png("crates/summoner_gui/scratch/renders/plate_dispersion_view.png", 800, 520);
}
