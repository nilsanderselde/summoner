// Summoner - Deterministic Golden Plucked & Struck String Waveguide & Sympathetic Coupling Regression Suite
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

use blake3::Hasher;
use std::fs;
use std::path::Path;
use summoner_core::allocator::AllocGuard;
use summoner_core::node::ProcessContext;
use summoner_core::param_bus::{ParamBus, ParamId};
use summoner_core::plectrum_bus::{PlectrumBus, PluckArticulation};
use summoner_core::transport::Transport;
use summoner_dsp::hammer_strike::{ExcitationType, HammerStrike, SympatheticResonatorMatrix, NUM_SYMPATHETIC_STRINGS};
use summoner_dsp::plucked_string::{CommutedPluckedString, PluckedInstrumentProfile};
use summoner_dsp::traits::SignalProcessor;
#[cfg(feature = "gui")]
use summoner_gui::views::plucked_string_view::PluckedStringView;
#[cfg(feature = "gui")]
use summoner_gui::views::sympathetic_coupling_view::SympatheticCouplingView;
use summoner_sequencer::pluck_gesture::{
    PluckGestureEngine, PluckGesturePattern, PluckInterpolationCurve, PluckWaypoint,
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
            "Golden plucked string & sympathetic coupling hash mismatch for {}",
            hash_filename
        );
    } else {
        fs::write(&hash_file, digest).expect("Failed to write golden hash");
    }
}

#[test]
fn test_golden_plucked_string_commuted_waveguide_deterministic() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    // 1. Acoustic Steel Guitar G3
    let mut steel = CommutedPluckedString::new(sample_rate);
    steel.set_instrument(PluckedInstrumentProfile::AcousticSteel);
    steel.set_frequency(196.00);
    steel.set_pluck_position(0.15);
    steel.trigger_pluck(0.85, 0.60);

    // 2. Classical Nylon Guitar B3
    let mut nylon = CommutedPluckedString::new(sample_rate);
    nylon.set_instrument(PluckedInstrumentProfile::ClassicalNylon);
    nylon.set_frequency(246.94);
    nylon.set_pluck_position(0.20);
    nylon.trigger_pluck(0.70, 0.35);

    // 3. Concert Grand Piano C4
    let mut piano = CommutedPluckedString::new(sample_rate);
    piano.set_instrument(PluckedInstrumentProfile::GrandPiano);
    piano.set_frequency(261.63);
    piano.set_pluck_position(0.12);
    piano.trigger_pluck(0.90, 0.75);

    // 4. Harpsichord A4
    let mut harpsichord = CommutedPluckedString::new(sample_rate);
    harpsichord.set_instrument(PluckedInstrumentProfile::Harpsichord);
    harpsichord.set_frequency(440.00);
    harpsichord.set_pluck_position(0.10);
    harpsichord.trigger_pluck(0.75, 0.85);

    // 5. Concert Harp A3
    let mut harp = CommutedPluckedString::new(sample_rate);
    harp.set_instrument(PluckedInstrumentProfile::Harp);
    harp.set_frequency(220.00);
    harp.set_pluck_position(0.25);
    harp.trigger_pluck(0.65, 0.40);

    // 6. Sitar / Koto D3
    let mut sitar = CommutedPluckedString::new(sample_rate);
    sitar.set_instrument(PluckedInstrumentProfile::SitarKoto);
    sitar.set_frequency(146.83);
    sitar.set_pluck_position(0.22);
    sitar.trigger_pluck(0.80, 0.65);

    let in_dummy = [0.0f32; BLOCK_SIZE];
    let mut out_steel = [0.0f32; BLOCK_SIZE];
    let mut out_nylon = [0.0f32; BLOCK_SIZE];
    let mut out_piano = [0.0f32; BLOCK_SIZE];
    let mut out_harpsichord = [0.0f32; BLOCK_SIZE];
    let mut out_harp = [0.0f32; BLOCK_SIZE];
    let mut out_sitar = [0.0f32; BLOCK_SIZE];

    let mut hasher = Hasher::new();

    // Process 64 blocks of audio (4096 samples)
    for _block_idx in 0..64 {
        let ctx = ProcessContext::from_transport(&transport);

        {
            let _guard = AllocGuard::new();
            steel.process_block(&[&in_dummy[..]], &mut [&mut out_steel[..]], &ctx);
            nylon.process_block(&[&in_dummy[..]], &mut [&mut out_nylon[..]], &ctx);
            piano.process_block(&[&in_dummy[..]], &mut [&mut out_piano[..]], &ctx);
            harpsichord.process_block(&[&in_dummy[..]], &mut [&mut out_harpsichord[..]], &ctx);
            harp.process_block(&[&in_dummy[..]], &mut [&mut out_harp[..]], &ctx);
            sitar.process_block(&[&in_dummy[..]], &mut [&mut out_sitar[..]], &ctx);
        }

        for i in 0..BLOCK_SIZE {
            assert!(out_steel[i].is_finite());
            assert!(out_nylon[i].is_finite());
            assert!(out_piano[i].is_finite());
            assert!(out_harpsichord[i].is_finite());
            assert!(out_harp[i].is_finite());
            assert!(out_sitar[i].is_finite());

            assert!((-1.0..=1.0).contains(&out_steel[i]));
            assert!((-1.0..=1.0).contains(&out_nylon[i]));
            assert!((-1.0..=1.0).contains(&out_piano[i]));
            assert!((-1.0..=1.0).contains(&out_harpsichord[i]));
            assert!((-1.0..=1.0).contains(&out_harp[i]));
            assert!((-1.0..=1.0).contains(&out_sitar[i]));

            hasher.update(&out_steel[i].to_le_bytes());
            hasher.update(&out_nylon[i].to_le_bytes());
            hasher.update(&out_piano[i].to_le_bytes());
            hasher.update(&out_harpsichord[i].to_le_bytes());
            hasher.update(&out_harp[i].to_le_bytes());
            hasher.update(&out_sitar[i].to_le_bytes());
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_plucked_string.hash", &digest);
}

#[test]
fn test_golden_hammer_strike_and_sympathetic_matrix_deterministic() {
    let sample_rate = 48000;
    let mut hammer = HammerStrike::new(sample_rate);
    let mut matrix = SympatheticResonatorMatrix::new();
    matrix.set_coupling_strength(0.20);

    let mut hasher = Hasher::new();

    let excitations = [
        (ExcitationType::HammerStrike, 0.85, 0.70),
        (ExcitationType::PlectrumPluck, 0.90, 0.80),
        (ExcitationType::FingerNylon, 0.65, 0.30),
        (ExcitationType::SlapBass, 1.00, 0.95),
    ];

    for (ex_type, vel, hard) in excitations {
        hammer.set_excitation_type(ex_type);
        hammer.trigger(vel, hard);

        for step in 0..128 {
            let string_disp = (step as f32 * 0.05).sin() * 0.0005;

            let res = {
                let _guard = AllocGuard::new();
                hammer.step(string_disp, 0.0)
            };

            assert!(res.contact_force_n.is_finite());
            assert!(res.hammer_disp_m.is_finite());
            assert!(res.hammer_vel_mps.is_finite());
            assert!(res.compression_m.is_finite());

            hasher.update(&res.contact_force_n.to_le_bytes());
            hasher.update(&res.hammer_disp_m.to_le_bytes());
            hasher.update(&res.hammer_vel_mps.to_le_bytes());
            hasher.update(&res.compression_m.to_le_bytes());
            hasher.update(&[if res.is_in_contact { 1 } else { 0 }]);

            // Sympathetic Matrix coupling step
            let mut sym_in = [0.0f32; NUM_SYMPATHETIC_STRINGS];
            sym_in[0] = res.contact_force_n * 0.01;
            sym_in[1] = string_disp * 100.0;
            let mut sym_out = [0.0f32; NUM_SYMPATHETIC_STRINGS];

            {
                let _guard = AllocGuard::new();
                matrix.process_junction(&sym_in, &mut sym_out);
            }

            for out_val in &sym_out {
                assert!(out_val.is_finite());
                hasher.update(&out_val.to_le_bytes());
            }
        }
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_hammer_sympathetic.hash", &digest);
}

#[test]
fn test_golden_pluck_gesture_and_plectrum_bus_deterministic() {
    let mut engine = PluckGestureEngine::new();
    engine.pattern = PluckGesturePattern::CustomSpline;
    engine.interpolation = PluckInterpolationCurve::CubicCatmullRom;
    engine.loop_length_beats = Some(8.0);

    engine.add_waypoint(PluckWaypoint {
        beat: 0.0,
        pluck_position_beta: 0.12,
        strike_velocity: 0.60,
        hammer_hardness: 0.40,
        plectrum_angle_rad: 0.0,
        palm_mute_damping: 0.0,
        sympathetic_bleed: 0.15,
        string_stiffness: 0.20,
        articulation: PluckArticulation::PluckNormal,
    });

    engine.add_waypoint(PluckWaypoint {
        beat: 2.0,
        pluck_position_beta: 0.22,
        strike_velocity: 0.85,
        hammer_hardness: 0.65,
        plectrum_angle_rad: 0.15,
        palm_mute_damping: 0.70,
        sympathetic_bleed: 0.08,
        string_stiffness: 0.35,
        articulation: PluckArticulation::PalmMute,
    });

    engine.add_waypoint(PluckWaypoint {
        beat: 4.0,
        pluck_position_beta: 0.08,
        strike_velocity: 1.00,
        hammer_hardness: 0.90,
        plectrum_angle_rad: -0.10,
        palm_mute_damping: 0.0,
        sympathetic_bleed: 0.25,
        string_stiffness: 0.30,
        articulation: PluckArticulation::SnapPizzicato,
    });

    engine.add_waypoint(PluckWaypoint {
        beat: 6.0,
        pluck_position_beta: 0.18,
        strike_velocity: 0.75,
        hammer_hardness: 0.50,
        plectrum_angle_rad: 0.05,
        palm_mute_damping: 0.0,
        sympathetic_bleed: 0.20,
        string_stiffness: 0.22,
        articulation: PluckArticulation::FingerRestStroke,
    });

    let bus = PlectrumBus::new();
    let mut param_bus = ParamBus::new();
    for i in 0..12 {
        param_bus.register(ParamId(800 + i), 0.0);
    }

    let mut hasher = Hasher::new();

    for step in 0..160 {
        let beat = step as f64 * 0.05; // 8 beats total

        let snap = {
            let _guard = AllocGuard::new();
            engine.dispatch_to_plectrum_bus(beat, &bus);
            engine.dispatch_to_param_bus(beat, &param_bus, ParamId(800));
            bus.snapshot()
        };

        assert!(snap.pluck_position_beta.is_finite());
        assert!(snap.strike_velocity.is_finite());
        assert!(snap.hammer_hardness.is_finite());
        assert!(snap.plectrum_angle_rad.is_finite());
        assert!(snap.palm_mute_damping.is_finite());
        assert!(snap.sympathetic_bleed.is_finite());
        assert!(snap.string_stiffness.is_finite());

        hasher.update(&(snap.articulation as u32).to_le_bytes());
        hasher.update(&snap.pluck_position_beta.to_le_bytes());
        hasher.update(&snap.strike_velocity.to_le_bytes());
        hasher.update(&snap.hammer_hardness.to_le_bytes());
        hasher.update(&snap.plectrum_angle_rad.to_le_bytes());
        hasher.update(&snap.palm_mute_damping.to_le_bytes());
        hasher.update(&snap.sympathetic_bleed.to_le_bytes());
        hasher.update(&snap.string_stiffness.to_le_bytes());
        hasher.update(&snap.gesture_progress.to_le_bytes());
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_pluck_gesture.hash", &digest);
}

#[cfg(feature = "gui")]
#[test]
fn test_golden_plucked_string_views_ascii_renders() {
    let pluck_view = PluckedStringView::new();
    let sympathetic_view = SympatheticCouplingView::new();

    let pluck_ascii = pluck_view.render_ascii(64, 16);
    let sympathetic_ascii = sympathetic_view.render_ascii(64, 16);

    let mut hasher = Hasher::new();
    for line in &pluck_ascii {
        hasher.update(line.as_bytes());
    }
    for line in &sympathetic_ascii {
        hasher.update(line.as_bytes());
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_plucked_views.hash", &digest);
}
