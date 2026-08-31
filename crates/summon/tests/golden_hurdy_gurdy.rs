// Summoner - Deterministic Golden Hurdy-Gurdy (Vielle à roue) Physical Modeling Regression Suite
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

use blake3::Hasher;
use std::f32::consts::PI;
use std::fs;
use std::path::Path;
use summoner_core::allocator::AllocGuard;
use summoner_core::hurdy_gurdy_bus::{
    register_hurdy_gurdy_params, HurdyGurdyBus, HURDY_GURDY_BASE_PARAM_ID,
};
use summoner_core::node::ProcessContext;
use summoner_core::param_bus::ParamBus;
use summoner_core::transport::Transport;
use summoner_dsp::hurdy_gurdy::{HurdyGurdy, HurdyGurdyProfile};
use summoner_dsp::traits::SignalProcessor;
use summoner_dsp::trompette_bridge::{
    ChienMaterialProfile, HurdyGurdySoundboxBody, TrompetteChienJunction,
};
#[cfg(feature = "gui")]
use summoner_gui::views::hurdy_gurdy_view::HurdyGurdyView;
#[cfg(feature = "gui")]
use summoner_gui::views::trompette_bridge_view::TrompetteBridgeView;
use summoner_sequencer::hurdy_gurdy_gesture::{
    HurdyGurdyGestureEngine, HurdyGurdyGesturePattern,
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
            "Golden Hurdy-Gurdy hash mismatch for {}",
            hash_filename
        );
    } else {
        fs::write(&hash_file, digest).expect("Failed to write golden hash");
    }
}

#[test]
fn test_golden_hurdy_gurdy_deterministic() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    // 1. Bourbonnais Traditional Vielle
    let mut bourbonnais = HurdyGurdy::new(sample_rate);
    bourbonnais.set_profile(HurdyGurdyProfile::BourbonnaisTraditional);
    bourbonnais.set_crank_speed(2.0 * PI);
    bourbonnais.set_wrist_acceleration(1.2);
    bourbonnais.set_tangent_note(67); // G4

    // 2. Medieval Symphonia Organistrum
    let mut symphonia = HurdyGurdy::new(sample_rate);
    symphonia.set_profile(HurdyGurdyProfile::MedievalSymphonia);
    symphonia.set_crank_speed(1.5 * PI);
    symphonia.set_wrist_acceleration(0.0);
    symphonia.set_tangent_note(72); // C5

    // 3. Baroque Virtuoso Vielle (Sharp ivory chien)
    let mut baroque = HurdyGurdy::new(sample_rate);
    baroque.set_profile(HurdyGurdyProfile::BaroqueVirtuoso);
    baroque.set_crank_speed(2.2 * PI);
    baroque.set_wrist_acceleration(0.6);
    baroque.set_tangent_note(74); // D5

    // 4. Auvergne High-Speed Folk Snarl
    let mut auvergne = HurdyGurdy::new(sample_rate);
    auvergne.set_profile(HurdyGurdyProfile::AuvergneFolkSnarl);
    auvergne.set_crank_speed(3.0 * PI);
    auvergne.set_wrist_acceleration(2.5);
    auvergne.set_tangent_note(76); // E5

    // 5. Gothic Pagan Dark Drone
    let mut gothic = HurdyGurdy::new(sample_rate);
    gothic.set_profile(HurdyGurdyProfile::GothicPaganDrone);
    gothic.set_crank_speed(1.2 * PI);
    gothic.set_wrist_acceleration(0.8);
    gothic.set_tangent_note(69); // A4

    // 6. Electro-Acoustic Modern Hybrid
    let mut modern = HurdyGurdy::new(sample_rate);
    modern.set_profile(HurdyGurdyProfile::ElectroAcousticModern);
    modern.set_crank_speed(2.8 * PI);
    modern.set_wrist_acceleration(1.8);
    modern.set_tangent_note(79); // G5

    let mut out_bourb_l = [0.0f32; BLOCK_SIZE];
    let mut out_bourb_r = [0.0f32; BLOCK_SIZE];
    let mut out_symph_l = [0.0f32; BLOCK_SIZE];
    let mut out_symph_r = [0.0f32; BLOCK_SIZE];
    let mut out_baroq_l = [0.0f32; BLOCK_SIZE];
    let mut out_baroq_r = [0.0f32; BLOCK_SIZE];
    let mut out_auver_l = [0.0f32; BLOCK_SIZE];
    let mut out_auver_r = [0.0f32; BLOCK_SIZE];
    let mut out_goth_l = [0.0f32; BLOCK_SIZE];
    let mut out_goth_r = [0.0f32; BLOCK_SIZE];
    let mut out_mod_l = [0.0f32; BLOCK_SIZE];
    let mut out_mod_r = [0.0f32; BLOCK_SIZE];

    let mut hasher = Hasher::new();

    // Process 64 blocks of audio (4096 samples)
    for block_idx in 0..64 {
        let ctx = ProcessContext::from_transport(&transport);

        {
            let _guard = AllocGuard::new();
            bourbonnais.process_block(&[], &mut [&mut out_bourb_l[..], &mut out_bourb_r[..]], &ctx);
            symphonia.process_block(&[], &mut [&mut out_symph_l[..], &mut out_symph_r[..]], &ctx);
            baroque.process_block(&[], &mut [&mut out_baroq_l[..], &mut out_baroq_r[..]], &ctx);
            auvergne.process_block(&[], &mut [&mut out_auver_l[..], &mut out_auver_r[..]], &ctx);
            gothic.process_block(&[], &mut [&mut out_goth_l[..], &mut out_goth_r[..]], &ctx);
            modern.process_block(&[], &mut [&mut out_mod_l[..], &mut out_mod_r[..]], &ctx);
        }

        if block_idx == 32 {
            // Mid-phrase Coup de Poignet wrist acceleration impulse & tangent step shift
            bourbonnais.set_wrist_acceleration(2.8);
            bourbonnais.set_tangent_note(71); // B4

            symphonia.set_crank_speed(1.8 * PI);
            symphonia.set_tangent_note(76); // E5

            baroque.set_wrist_acceleration(1.5);
            baroque.set_tangent_note(77); // F5

            auvergne.set_wrist_acceleration(4.0);
            auvergne.set_chien_clearance(0.10);
            auvergne.set_tangent_note(81); // A5

            gothic.set_wrist_acceleration(1.8);
            gothic.set_tangent_note(65); // F4

            modern.set_wrist_acceleration(3.0);
            modern.set_tangent_note(83); // B5
        }

        for i in 0..BLOCK_SIZE {
            assert!(out_bourb_l[i].is_finite() && out_bourb_r[i].is_finite());
            assert!(out_symph_l[i].is_finite() && out_symph_r[i].is_finite());
            assert!(out_baroq_l[i].is_finite() && out_baroq_r[i].is_finite());
            assert!(out_auver_l[i].is_finite() && out_auver_r[i].is_finite());
            assert!(out_goth_l[i].is_finite() && out_goth_r[i].is_finite());
            assert!(out_mod_l[i].is_finite() && out_mod_r[i].is_finite());

            assert!((-1.0..=1.0).contains(&out_bourb_l[i]) && (-1.0..=1.0).contains(&out_bourb_r[i]));
            assert!((-1.0..=1.0).contains(&out_symph_l[i]) && (-1.0..=1.0).contains(&out_symph_r[i]));
            assert!((-1.0..=1.0).contains(&out_baroq_l[i]) && (-1.0..=1.0).contains(&out_baroq_r[i]));
            assert!((-1.0..=1.0).contains(&out_auver_l[i]) && (-1.0..=1.0).contains(&out_auver_r[i]));
            assert!((-1.0..=1.0).contains(&out_goth_l[i]) && (-1.0..=1.0).contains(&out_goth_r[i]));
            assert!((-1.0..=1.0).contains(&out_mod_l[i]) && (-1.0..=1.0).contains(&out_mod_r[i]));

            hasher.update(&out_bourb_l[i].to_le_bytes());
            hasher.update(&out_bourb_r[i].to_le_bytes());
            hasher.update(&out_symph_l[i].to_le_bytes());
            hasher.update(&out_symph_r[i].to_le_bytes());
            hasher.update(&out_baroq_l[i].to_le_bytes());
            hasher.update(&out_baroq_r[i].to_le_bytes());
            hasher.update(&out_auver_l[i].to_le_bytes());
            hasher.update(&out_auver_r[i].to_le_bytes());
            hasher.update(&out_goth_l[i].to_le_bytes());
            hasher.update(&out_goth_r[i].to_le_bytes());
            hasher.update(&out_mod_l[i].to_le_bytes());
            hasher.update(&out_mod_r[i].to_le_bytes());
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_hurdy_gurdy.hash", &digest);
}

#[test]
fn test_golden_hurdy_gurdy_chien_and_tangent_deterministic() {
    // 1. Chien contact obstacle collision stability
    for profile in [
        ChienMaterialProfile::MapleOnBone,
        ChienMaterialProfile::IvoryPlate,
        ChienMaterialProfile::EbonyHardwood,
        ChienMaterialProfile::SyntheticDelrin,
        ChienMaterialProfile::VintageFruitwood,
    ] {
        let mut junction = TrompetteChienJunction::new(profile, 0.25);
        let mut total_contact_force = 0.0f32;

        for _ in 0..500 {
            let (force, buzz, body_exc) = junction.process_collision(-0.50, -1.5, 2.0, 48000);
            total_contact_force += force;
            assert!(buzz.is_finite());
            assert!(body_exc >= 0.0);
        }

        assert!(total_contact_force > 0.0, "Chien should register contacts under excitation");
    }

    // 2. Tangent key action pitch accuracy
    let mut hg = HurdyGurdy::new(48000);
    let open_g4 = hg.chanterelles[0].active_freq_hz;
    assert!((open_g4 - 392.0).abs() < 1.0);

    hg.set_tangent_note(79); // G5 (+12 st)
    let g5 = hg.chanterelles[0].active_freq_hz;
    assert!((g5 - 784.0).abs() < 2.0);

    hg.set_tangent_note(91); // G6 (+24 st / 2 octaves)
    let g6 = hg.chanterelles[0].active_freq_hz;
    assert!((g6 - 1568.0).abs() < 4.0);

    // 3. Soundbox modal resonance decay
    let mut soundbox = HurdyGurdySoundboxBody::new(48000);
    let init_resp = soundbox.process(1.0);
    assert!(init_resp.abs() > 0.0);

    for _ in 0..2048 {
        soundbox.process(0.0);
    }
    let decayed = soundbox.process(0.0);
    assert!(decayed.abs() < init_resp.abs(), "Soundbox resonance must decay");
}

#[test]
fn test_hurdy_gurdy_gesture_dispatch_and_param_bus_roundtrip() {
    let mut param_bus = ParamBus::new();
    register_hurdy_gurdy_params(&mut param_bus, HURDY_GURDY_BASE_PARAM_ID);
    let bus = HurdyGurdyBus::new();

    for pattern in [
        HurdyGurdyGesturePattern::BourbonnaisClassicCoupDePoignet { phrase_length_beats: 4.0, pulse_intensity: 1.5 },
        HurdyGurdyGesturePattern::MedievalMonasticDrone { phrase_length_beats: 4.0 },
        HurdyGurdyGesturePattern::BaroqueVirtuosoChanterelle { phrase_length_beats: 4.0 },
        HurdyGurdyGesturePattern::AuvergneHighSpeedBuzz { phrase_length_beats: 6.0, crank_speed_multiplier: 1.2 },
        HurdyGurdyGesturePattern::GothicPaganDrone { phrase_length_beats: 4.0 },
        HurdyGurdyGesturePattern::ElectroAcousticHybrid { phrase_length_beats: 4.0 },
    ] {
        let engine = HurdyGurdyGestureEngine::with_pattern(pattern);

        for beat in [0.0, 1.0, 2.5, 3.8] {
            engine.dispatch_to_bus(beat, &bus, Some(&param_bus), HURDY_GURDY_BASE_PARAM_ID);
            let snap = bus.snapshot();

            assert!(snap.crank_speed_rad_s > 0.0);
            assert!(snap.wheel_pressure > 0.0);
            assert!(snap.chien_clearance_mm > 0.0);
            assert!(snap.drone_melody_mix >= 0.0);
            assert!(snap.body_wood_resonance > 0.0);
            assert!(snap.master_gain > 0.0);

            let bus_speed = param_bus.get(summoner_core::param_bus::ParamId(HURDY_GURDY_BASE_PARAM_ID.0 + 1));
            assert!(bus_speed.is_some());
            assert!((bus_speed.unwrap() - snap.crank_speed_rad_s).abs() < 1e-4);
        }
    }
}

#[test]
#[cfg(feature = "gui")]
fn test_hurdy_gurdy_gui_views_headless_snapshots() {
    let mut hg_view = HurdyGurdyView::new();
    hg_view.set_preset(summoner_gui::views::hurdy_gurdy_view::HurdyGurdyHudPreset::BourbonnaisTraditional);

    let hg_png_paths = [
        "scratch/renders/hurdy_gurdy_view.png",
        "crates/summoner_gui/scratch/renders/hurdy_gurdy_view.png",
        "crates/summon/scratch/renders/hurdy_gurdy_view.png",
    ];
    for path in hg_png_paths {
        let res = hg_view.render_snapshot_png(path, 800, 520);
        assert!(res.is_ok(), "Failed to render HurdyGurdyView PNG to {}", path);
    }

    let mut chien_view = TrompetteBridgeView::new();
    chien_view.set_profile(summoner_gui::views::trompette_bridge_view::ChienViewProfile::MapleOnBone);

    let chien_png_paths = [
        "scratch/renders/trompette_bridge_view.png",
        "crates/summoner_gui/scratch/renders/trompette_bridge_view.png",
        "crates/summon/scratch/renders/trompette_bridge_view.png",
    ];
    for path in chien_png_paths {
        let res = chien_view.render_snapshot_png(path, 800, 520);
        assert!(res.is_ok(), "Failed to render TrompetteBridgeView PNG to {}", path);
    }
}
