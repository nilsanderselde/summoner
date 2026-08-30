// Summoner - Deterministic Golden Asian Zither & Koto Physical Modeling Regression Suite
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

use blake3::Hasher;
use std::fs;
use std::path::Path;
use summoner_core::allocator::AllocGuard;
use summoner_core::koto_bus::{
    register_koto_params, KotoBus, KOTO_BASE_PARAM_ID,
};
use summoner_core::node::ProcessContext;
use summoner_core::param_bus::ParamBus;
use summoner_core::transport::Transport;
use summoner_dsp::ji_bridge::{
    oshi_ite_pitch_multiplier, JiBridgeJunction, JiBridgeProfile,
    PaulowniaSoundboardBody,
};
use summoner_dsp::koto::{Koto, KotoProfile};
use summoner_dsp::traits::SignalProcessor;
#[cfg(feature = "gui")]
use summoner_gui::views::ji_bridge_view::JiBridgeView;
#[cfg(feature = "gui")]
use summoner_gui::views::koto_view::KotoView;
use summoner_sequencer::koto_gesture::{
    KotoGestureEngine, KotoGesturePattern,
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
            "Golden Koto hash mismatch for {}",
            hash_filename
        );
    } else {
        fs::write(&hash_file, digest).expect("Failed to write golden hash");
    }
}

#[test]
fn test_golden_koto_deterministic() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    // 1. Hirajoshi Traditional Silk Koto
    let mut hirajoshi = Koto::new(sample_rate);
    hirajoshi.set_profile(KotoProfile::HirajoshiTraditionalSilk);
    hirajoshi.pluck_string(0, 0.90); // String 1: D4
    hirajoshi.pluck_string(2, 0.85); // String 3: A3
    hirajoshi.pluck_string(4, 0.88); // String 5: D4

    // 2. Kokin-joshi Urban Classical Koto (Bright ivory tsume)
    let mut kokin = Koto::new(sample_rate);
    kokin.set_profile(KotoProfile::KokinJoshiUrbanClassical);
    kokin.pluck_string(0, 0.92);
    kokin.pluck_string(3, 0.88); // String 4: C4
    kokin.pluck_string(5, 0.85); // String 6: Eb4

    // 3. In-sen Contemporary Dramatic Koto (High oshi-ite tension bends)
    let mut insen = Koto::new(sample_rate);
    insen.set_profile(KotoProfile::InSenContemporaryDramatic);
    insen.set_oshi_ite(0, 15.0); // +200 cents on string 1
    insen.pluck_string(0, 0.95);
    insen.pluck_string(5, 0.90); // String 6: F4
    insen.pluck_string(9, 0.92); // String 10: D5

    // 4. Kumoi-joshi Spring Rain Koto (Soft bamboo bridge)
    let mut kumoi = Koto::new(sample_rate);
    kumoi.set_profile(KotoProfile::KumoiJoshiSpringRain);
    kumoi.pluck_string(1, 0.75); // String 2: G3
    kumoi.pluck_string(3, 0.80); // String 4: Bb3
    kumoi.pluck_string(6, 0.82); // String 7: G4

    // 5. Ryukyu Festive Okinawan Koto (Okinawan major 3rd scale)
    let mut ryukyu = Koto::new(sample_rate);
    ryukyu.set_profile(KotoProfile::RyukyuFestiveOkinawa);
    ryukyu.pluck_string(0, 0.85); // String 1: D4
    ryukyu.pluck_string(2, 0.82); // String 3: B3
    ryukyu.pluck_string(5, 0.88); // String 6: E4

    // 6. Concert Guzheng Virtuoso (Major Pentatonic)
    let mut guzheng = Koto::new(sample_rate);
    guzheng.set_profile(KotoProfile::ConcertGuzhengVirtuoso);
    guzheng.pluck_string(0, 0.95);
    guzheng.pluck_string(4, 0.90);
    guzheng.pluck_string(7, 0.88);
    guzheng.pluck_string(11, 0.92);

    let mut out_hira_l = [0.0f32; BLOCK_SIZE];
    let mut out_hira_r = [0.0f32; BLOCK_SIZE];
    let mut out_kokin_l = [0.0f32; BLOCK_SIZE];
    let mut out_kokin_r = [0.0f32; BLOCK_SIZE];
    let mut out_insen_l = [0.0f32; BLOCK_SIZE];
    let mut out_insen_r = [0.0f32; BLOCK_SIZE];
    let mut out_kumoi_l = [0.0f32; BLOCK_SIZE];
    let mut out_kumoi_r = [0.0f32; BLOCK_SIZE];
    let mut out_ryuk_l = [0.0f32; BLOCK_SIZE];
    let mut out_ryuk_r = [0.0f32; BLOCK_SIZE];
    let mut out_guzh_l = [0.0f32; BLOCK_SIZE];
    let mut out_guzh_r = [0.0f32; BLOCK_SIZE];

    let mut hasher = Hasher::new();

    // Process 64 blocks of audio (4096 samples)
    for block_idx in 0..64 {
        let ctx = ProcessContext::from_transport(&transport);

        {
            let _guard = AllocGuard::new();
            hirajoshi.process_block(&[], &mut [&mut out_hira_l[..], &mut out_hira_r[..]], &ctx);
            kokin.process_block(&[], &mut [&mut out_kokin_l[..], &mut out_kokin_r[..]], &ctx);
            insen.process_block(&[], &mut [&mut out_insen_l[..], &mut out_insen_r[..]], &ctx);
            kumoi.process_block(&[], &mut [&mut out_kumoi_l[..], &mut out_kumoi_r[..]], &ctx);
            ryukyu.process_block(&[], &mut [&mut out_ryuk_l[..], &mut out_ryuk_r[..]], &ctx);
            guzheng.process_block(&[], &mut [&mut out_guzh_l[..], &mut out_guzh_r[..]], &ctx);
        }

        if block_idx == 32 {
            // Mid-phrase oshi-ite pitch deflection and additional plucks
            hirajoshi.set_oshi_ite(0, 12.0); // +150 cents press
            hirajoshi.pluck_string(6, 0.92); // String 7: G4

            kokin.pluck_string(7, 0.90);    // String 8: Ab4
            kokin.set_hiki_iro(3, 0.30);    // Yuri release

            insen.set_oshi_ite(0, 28.0);    // +400 cents (Major 3rd deflection)
            insen.pluck_string(11, 0.95);   // String 12: G5

            kumoi.pluck_string(8, 0.85);    // String 9: Bb4
            kumoi.set_bridge_offset(8, 0.05);

            ryukyu.pluck_string(9, 0.90);   // String 10: D5
            ryukyu.pluck_string(12, 0.92);  // String 13: B5

            guzheng.pluck_string(2, 0.88);
            guzheng.pluck_string(9, 0.94);
        }

        for i in 0..BLOCK_SIZE {
            assert!(out_hira_l[i].is_finite() && out_hira_r[i].is_finite());
            assert!(out_kokin_l[i].is_finite() && out_kokin_r[i].is_finite());
            assert!(out_insen_l[i].is_finite() && out_insen_r[i].is_finite());
            assert!(out_kumoi_l[i].is_finite() && out_kumoi_r[i].is_finite());
            assert!(out_ryuk_l[i].is_finite() && out_ryuk_r[i].is_finite());
            assert!(out_guzh_l[i].is_finite() && out_guzh_r[i].is_finite());

            assert!((-1.0..=1.0).contains(&out_hira_l[i]) && (-1.0..=1.0).contains(&out_hira_r[i]));
            assert!((-1.0..=1.0).contains(&out_kokin_l[i]) && (-1.0..=1.0).contains(&out_kokin_r[i]));
            assert!((-1.0..=1.0).contains(&out_insen_l[i]) && (-1.0..=1.0).contains(&out_insen_r[i]));
            assert!((-1.0..=1.0).contains(&out_kumoi_l[i]) && (-1.0..=1.0).contains(&out_kumoi_r[i]));
            assert!((-1.0..=1.0).contains(&out_ryuk_l[i]) && (-1.0..=1.0).contains(&out_ryuk_r[i]));
            assert!((-1.0..=1.0).contains(&out_guzh_l[i]) && (-1.0..=1.0).contains(&out_guzh_r[i]));

            hasher.update(&out_hira_l[i].to_le_bytes());
            hasher.update(&out_hira_r[i].to_le_bytes());
            hasher.update(&out_kokin_l[i].to_le_bytes());
            hasher.update(&out_kokin_r[i].to_le_bytes());
            hasher.update(&out_insen_l[i].to_le_bytes());
            hasher.update(&out_insen_r[i].to_le_bytes());
            hasher.update(&out_kumoi_l[i].to_le_bytes());
            hasher.update(&out_kumoi_r[i].to_le_bytes());
            hasher.update(&out_ryuk_l[i].to_le_bytes());
            hasher.update(&out_ryuk_r[i].to_le_bytes());
            hasher.update(&out_guzh_l[i].to_le_bytes());
            hasher.update(&out_guzh_r[i].to_le_bytes());
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_koto.hash", &digest);
}

#[test]
fn test_golden_koto_ji_bridge_and_oshi_ite_deterministic() {
    // 1. Oshi-Ite non-linear pitch shift accuracy
    let base_mult = oshi_ite_pitch_multiplier(0.0);
    assert_eq!(base_mult, 1.0);

    let half_step = oshi_ite_pitch_multiplier(7.5);
    assert!(half_step > 1.05 && half_step < 1.15, "Expected ~100 cents bend, got multiplier {}", half_step);

    let whole_step = oshi_ite_pitch_multiplier(15.0);
    assert!(whole_step > 1.10 && whole_step < 1.20, "Expected ~200 cents bend, got multiplier {}", whole_step);

    let major_third = oshi_ite_pitch_multiplier(30.0);
    let cents_30n = 1200.0 * major_third.log2();
    assert!((cents_30n - 400.0).abs() < 1.0, "Expected ~400 cents, got {}", cents_30n);

    // 2. Ji bridge scattering energy conservation
    for profile in [
        JiBridgeProfile::PaulowniaHardwood,
        JiBridgeProfile::IvoryBone,
        JiBridgeProfile::RosewoodGuzheng,
        JiBridgeProfile::SyntheticPlastic,
        JiBridgeProfile::SmokedBamboo,
    ] {
        let mut junction = JiBridgeJunction::new(profile, 0.60);
        let (refl, trans, body) = junction.process_scattering(1.0, 0.0);
        assert!(refl.abs() <= 1.0);
        assert!(trans.abs() <= 1.0);
        assert!(body >= 0.0);

        let energy_sum = refl * refl + trans * trans;
        assert!(energy_sum <= 1.05, "Scattering energy non-conservation for {:?}", profile);
    }

    // 3. Paulownia soundboard modal resonance stability
    let mut soundboard = PaulowniaSoundboardBody::new(48000);
    let mut energy_sum = 0.0f32;
    let initial_resp = soundboard.process(1.0);
    energy_sum += initial_resp.abs();

    for _ in 0..2048 {
        let resp = soundboard.process(0.0);
        energy_sum += resp.abs();
        assert!(resp.is_finite());
    }
    assert!(energy_sum > 0.0);
}

#[test]
fn test_koto_gesture_dispatch_and_param_bus_roundtrip() {
    let mut param_bus = ParamBus::new();
    register_koto_params(&mut param_bus, KOTO_BASE_PARAM_ID);
    let bus = KotoBus::new();

    for pattern in [
        KotoGesturePattern::HirajoshiMeditativeAlap { phrase_length_beats: 4.0, max_oshi_ite_n: 15.0 },
        KotoGesturePattern::KokinJoshiFastMiyako { phrase_length_beats: 4.0 },
        KotoGesturePattern::InSenDramaticGendai { phrase_length_beats: 6.0, max_pitch_bend_n: 25.0 },
        KotoGesturePattern::KumoiJoshiSpringRain { phrase_length_beats: 4.0 },
        KotoGesturePattern::RyukyuIslandSwell { phrase_length_beats: 4.0 },
        KotoGesturePattern::GuzhengVirtuosoWaterfall { phrase_length_beats: 8.0, arpeggio_rate_hz: 12.0 },
    ] {
        let engine = KotoGestureEngine::with_pattern(pattern);

        for beat in [0.0, 1.0, 2.5, 3.8] {
            engine.dispatch_to_bus(beat, &bus, Some(&param_bus), KOTO_BASE_PARAM_ID);
            let snap = bus.snapshot();

            assert!(snap.tsume_velocity > 0.0);
            assert!(snap.oshi_ite_force_n >= 0.0);
            assert!(snap.hiki_iro_release >= 0.0);
            assert!(snap.behind_bridge_bleed >= 0.0);
            assert!(snap.body_wood_resonance >= 0.0);
            assert!(snap.master_gain > 0.0);

            let bus_vel = param_bus.get(summoner_core::param_bus::ParamId(KOTO_BASE_PARAM_ID.0 + 1));
            assert!(bus_vel.is_some());
            assert!((bus_vel.unwrap() - snap.tsume_velocity).abs() < 1e-4);
        }
    }
}

#[test]
#[cfg(feature = "gui")]
fn test_koto_gui_views_headless_snapshots() {
    let mut koto_view = KotoView::new();
    koto_view.set_preset(summoner_gui::views::koto_view::KotoHudPreset::HirajoshiTraditionalSilk);

    let koto_png_paths = [
        "scratch/renders/koto_view.png",
        "crates/summoner_gui/scratch/renders/koto_view.png",
        "crates/summon/scratch/renders/koto_view.png",
    ];
    for path in koto_png_paths {
        let res = koto_view.render_snapshot_png(path, 800, 520);
        assert!(res.is_ok(), "Failed to render KotoView PNG to {}", path);
    }

    let mut ji_view = JiBridgeView::new();
    ji_view.set_profile(summoner_gui::views::ji_bridge_view::JiBridgeViewProfile::PaulowniaHardwood);

    let ji_png_paths = [
        "scratch/renders/ji_bridge_view.png",
        "crates/summoner_gui/scratch/renders/ji_bridge_view.png",
        "crates/summon/scratch/renders/ji_bridge_view.png",
    ];
    for path in ji_png_paths {
        let res = ji_view.render_snapshot_png(path, 800, 520);
        assert!(res.is_ok(), "Failed to render JiBridgeView PNG to {}", path);
    }
}
