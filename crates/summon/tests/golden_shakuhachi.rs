// Summoner - Deterministic Golden Shakuhachi Physical Modeling Regression Suite
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

use blake3::Hasher;
use std::fs;
use std::path::Path;
use summoner_core::allocator::AllocGuard;
use summoner_core::node::ProcessContext;
use summoner_core::param_bus::ParamBus;
use summoner_core::shakuhachi_bus::{
    register_shakuhachi_params, ShakuhachiArticulation, ShakuhachiBus, SHAKUHACHI_BASE_PARAM_ID,
};
use summoner_core::transport::Transport;
use summoner_dsp::air_reed::{jet_velocity_from_pressure, AirReed};
use summoner_dsp::shakuhachi::{
    Shakuhachi, ShakuhachiProfile, HOLE_1_TSU, HOLE_2_RE, HOLE_3_CHI, HOLE_4_RI,
};
use summoner_dsp::traits::SignalProcessor;
#[cfg(feature = "gui")]
use summoner_gui::views::embouchure_angle_view::EmbouchureAngleView;
#[cfg(feature = "gui")]
use summoner_gui::views::shakuhachi_view::ShakuhachiView;
use summoner_sequencer::shakuhachi_gesture::{
    ShakuhachiGestureEngine, ShakuhachiInterpolationCurve, ShakuhachiWaypoint,
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
            "Golden Shakuhachi hash mismatch for {}",
            hash_filename
        );
    } else {
        fs::write(&hash_file, digest).expect("Failed to write golden hash");
    }
}

#[test]
fn test_golden_shakuhachi_deterministic() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    // 1. Classical Honkyoku Zen Flute (1.8 Shaku D4=62)
    let mut honkyoku = Shakuhachi::new(sample_rate);
    honkyoku.set_profile(ShakuhachiProfile::HonkyokuTraditional);
    honkyoku.note_on(62, 0.90);

    // 2. Meditative Jinashi Zen Flute (2.4 Shaku A3=57)
    let mut jinashi = Shakuhachi::new(sample_rate);
    jinashi.set_profile(ShakuhachiProfile::ZenMeditative);
    jinashi.set_meri_kari(-60.0);
    jinashi.note_on(57, 0.85);

    // 3. Min'yo Folk Flute (1.6 Shaku E4=64) with pentatonic fingerings
    let mut minyo = Shakuhachi::new(sample_rate);
    minyo.set_profile(ShakuhachiProfile::MinyoFolk);
    minyo.set_hole_mask(HOLE_1_TSU | HOLE_2_RE);
    minyo.note_on(64, 0.88);

    // 4. Sankyoku Chamber Ensemble (2.1 Shaku B3=59)
    let mut sankyoku = Shakuhachi::new(sample_rate);
    sankyoku.set_profile(ShakuhachiProfile::SankyokuEnsemble);
    sankyoku.note_on(59, 0.80);

    // 5. Explosive Murai-Iki Technique (1.8 Shaku D4=62)
    let mut murai = Shakuhachi::new(sample_rate);
    murai.set_profile(ShakuhachiProfile::MuraiIkiExplosive);
    murai.set_murai_iki(0.85);
    murai.note_on(62, 0.95);

    // 6. Giant Sub-bass Kyotaku Temple Flute (3.0 Shaku D3=50)
    let mut kyotaku = Shakuhachi::new(sample_rate);
    kyotaku.set_profile(ShakuhachiProfile::KyotakuTemple);
    kyotaku.set_meri_kari(-100.0);
    kyotaku.note_on(50, 0.90);

    let mut out_honk_l = [0.0f32; BLOCK_SIZE];
    let mut out_honk_r = [0.0f32; BLOCK_SIZE];
    let mut out_jina_l = [0.0f32; BLOCK_SIZE];
    let mut out_jina_r = [0.0f32; BLOCK_SIZE];
    let mut out_miny_l = [0.0f32; BLOCK_SIZE];
    let mut out_miny_r = [0.0f32; BLOCK_SIZE];
    let mut out_sank_l = [0.0f32; BLOCK_SIZE];
    let mut out_sank_r = [0.0f32; BLOCK_SIZE];
    let mut out_mura_l = [0.0f32; BLOCK_SIZE];
    let mut out_mura_r = [0.0f32; BLOCK_SIZE];
    let mut out_kyot_l = [0.0f32; BLOCK_SIZE];
    let mut out_kyot_r = [0.0f32; BLOCK_SIZE];

    let mut hasher = Hasher::new();

    // Process 64 blocks of audio (4096 samples)
    for block_idx in 0..64 {
        let ctx = ProcessContext::from_transport(&transport);

        {
            let _guard = AllocGuard::new();
            honkyoku.process_block(&[], &mut [&mut out_honk_l[..], &mut out_honk_r[..]], &ctx);
            jinashi.process_block(&[], &mut [&mut out_jina_l[..], &mut out_jina_r[..]], &ctx);
            minyo.process_block(&[], &mut [&mut out_miny_l[..], &mut out_miny_r[..]], &ctx);
            sankyoku.process_block(&[], &mut [&mut out_sank_l[..], &mut out_sank_r[..]], &ctx);
            murai.process_block(&[], &mut [&mut out_mura_l[..], &mut out_mura_r[..]], &ctx);
            kyotaku.process_block(&[], &mut [&mut out_kyot_l[..], &mut out_kyot_r[..]], &ctx);
        }

        if block_idx == 32 {
            // Note off and note transition halfway
            honkyoku.note_off(62);
            honkyoku.note_on(65, 0.85); // F4 (Tsu)
            jinashi.set_meri_kari(-180.0); // Deep meri bend
            minyo.set_hole_mask(HOLE_1_TSU | HOLE_2_RE | HOLE_3_CHI | HOLE_4_RI); // Ri note
            murai.set_murai_iki(0.35); // Decay of burst
        }

        for i in 0..BLOCK_SIZE {
            assert!(out_honk_l[i].is_finite() && out_honk_r[i].is_finite());
            assert!(out_jina_l[i].is_finite() && out_jina_r[i].is_finite());
            assert!(out_miny_l[i].is_finite() && out_miny_r[i].is_finite());
            assert!(out_sank_l[i].is_finite() && out_sank_r[i].is_finite());
            assert!(out_mura_l[i].is_finite() && out_mura_r[i].is_finite());
            assert!(out_kyot_l[i].is_finite() && out_kyot_r[i].is_finite());

            assert!((-5.0..=5.0).contains(&out_honk_l[i]) && (-5.0..=5.0).contains(&out_honk_r[i]));
            assert!((-5.0..=5.0).contains(&out_jina_l[i]) && (-5.0..=5.0).contains(&out_jina_r[i]));
            assert!((-5.0..=5.0).contains(&out_miny_l[i]) && (-5.0..=5.0).contains(&out_miny_r[i]));
            assert!((-5.0..=5.0).contains(&out_sank_l[i]) && (-5.0..=5.0).contains(&out_sank_r[i]));
            assert!((-5.0..=5.0).contains(&out_mura_l[i]) && (-5.0..=5.0).contains(&out_mura_r[i]));
            assert!((-5.0..=5.0).contains(&out_kyot_l[i]) && (-5.0..=5.0).contains(&out_kyot_r[i]));

            hasher.update(&out_honk_l[i].to_le_bytes());
            hasher.update(&out_honk_r[i].to_le_bytes());
            hasher.update(&out_jina_l[i].to_le_bytes());
            hasher.update(&out_jina_r[i].to_le_bytes());
            hasher.update(&out_miny_l[i].to_le_bytes());
            hasher.update(&out_miny_r[i].to_le_bytes());
            hasher.update(&out_sank_l[i].to_le_bytes());
            hasher.update(&out_sank_r[i].to_le_bytes());
            hasher.update(&out_mura_l[i].to_le_bytes());
            hasher.update(&out_mura_r[i].to_le_bytes());
            hasher.update(&out_kyot_l[i].to_le_bytes());
            hasher.update(&out_kyot_r[i].to_le_bytes());
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_shakuhachi.hash", &digest);
}

#[test]
fn test_golden_shakuhachi_air_reed_aerodynamics_deterministic() {
    let sample_rate = 48000;
    let mut air_reed = AirReed::new(sample_rate);
    air_reed.set_blowing_pressure(900.0);
    air_reed.set_meri_kari(-40.0);
    air_reed.set_murai_iki(0.25);

    // Verify Bernoulli velocity formula: v = sqrt(2 * P / rho)
    // For P = 900 Pa, rho = 1.204 kg/m^3: v = sqrt(1800 / 1.204) = sqrt(1495.0166) ~ 38.665 m/s
    let v_calc = jet_velocity_from_pressure(900.0);
    let v_expected = (2.0 * 900.0 / 1.204f32).sqrt();
    assert!((v_calc - v_expected).abs() < 1e-3);

    let mut hasher = Hasher::new();

    // Process 4096 samples of air-reed excitation
    for step in 0..4096 {
        let t = step as f32 / 48000.0;
        let bore_feedback = (2.0 * std::f32::consts::PI * 293.66 * t).sin() * 0.25;

        let out = {
            let _guard = AllocGuard::new();
            air_reed.step(bore_feedback)
        };

        assert!(out.is_finite());
        assert!((-3.0..=3.0).contains(&out));

        hasher.update(&out.to_le_bytes());
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_shakuhachi_air_reed.hash", &digest);
}

#[test]
fn test_golden_shakuhachi_gesture_and_bus_deterministic() {
    let mut engine = ShakuhachiGestureEngine::new();
    engine.default_curve = ShakuhachiInterpolationCurve::CubicCatmullRom;
    engine.loop_length_beats = 8.0;
    engine.loop_enabled = true;

    engine.waypoints = vec![
        ShakuhachiWaypoint {
            beat: 0.0,
            blowing_pressure_pa: 600.0,
            embouchure_angle_deg: 38.0,
            meri_kari_cents: 0.0,
            jet_distance_mm: 10.0,
            murai_iki_intensity: 0.05,
            hole_mask: 0,
            master_gain: 0.80,
            articulation: ShakuhachiArticulation::HonkyokuTraditional,
            curve: ShakuhachiInterpolationCurve::CubicCatmullRom,
        },
        ShakuhachiWaypoint {
            beat: 2.0,
            blowing_pressure_pa: 2200.0,
            embouchure_angle_deg: 32.0,
            meri_kari_cents: 40.0,
            jet_distance_mm: 9.0,
            murai_iki_intensity: 0.80,
            hole_mask: 0,
            master_gain: 0.92,
            articulation: ShakuhachiArticulation::MuraiIkiExplosive,
            curve: ShakuhachiInterpolationCurve::Exponential,
        },
        ShakuhachiWaypoint {
            beat: 4.0,
            blowing_pressure_pa: 750.0,
            embouchure_angle_deg: 22.0,
            meri_kari_cents: -200.0,
            jet_distance_mm: 12.0,
            murai_iki_intensity: 0.15,
            hole_mask: 0,
            master_gain: 0.82,
            articulation: ShakuhachiArticulation::MeriMicrotoneBend,
            curve: ShakuhachiInterpolationCurve::CubicCatmullRom,
        },
        ShakuhachiWaypoint {
            beat: 6.0,
            blowing_pressure_pa: 1100.0,
            embouchure_angle_deg: 40.0,
            meri_kari_cents: 0.0,
            jet_distance_mm: 9.5,
            murai_iki_intensity: 0.08,
            hole_mask: HOLE_1_TSU | HOLE_2_RE | HOLE_3_CHI,
            master_gain: 0.88,
            articulation: ShakuhachiArticulation::MinyoFolk,
            curve: ShakuhachiInterpolationCurve::CubicCatmullRom,
        },
    ];

    let bus = ShakuhachiBus::new();
    let mut param_bus = ParamBus::new();
    register_shakuhachi_params(&mut param_bus, SHAKUHACHI_BASE_PARAM_ID);

    let mut hasher = Hasher::new();

    for step in 0..160 {
        let beat = step as f64 * 0.05; // 8 beats total

        let snap = {
            let _guard = AllocGuard::new();
            let s = engine.evaluate_at_beat(beat);
            engine.dispatch_to_bus(&s, &bus);
            engine.dispatch_to_param_bus(&s, &param_bus, SHAKUHACHI_BASE_PARAM_ID);
            bus.snapshot()
        };

        assert!(snap.blowing_pressure_pa.is_finite());
        assert!(snap.embouchure_angle_deg.is_finite());
        assert!(snap.meri_kari_cents.is_finite());
        assert!(snap.jet_distance_mm.is_finite());
        assert!(snap.murai_iki_intensity.is_finite());
        assert!(snap.master_gain.is_finite());

        hasher.update(&(snap.articulation as u32).to_le_bytes());
        hasher.update(&snap.blowing_pressure_pa.to_le_bytes());
        hasher.update(&snap.embouchure_angle_deg.to_le_bytes());
        hasher.update(&snap.meri_kari_cents.to_le_bytes());
        hasher.update(&snap.jet_distance_mm.to_le_bytes());
        hasher.update(&snap.murai_iki_intensity.to_le_bytes());
        hasher.update(&snap.hole_mask.to_le_bytes());
        hasher.update(&snap.master_gain.to_le_bytes());
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_shakuhachi_gesture.hash", &digest);
}

#[cfg(feature = "gui")]
#[test]
fn test_golden_shakuhachi_views_ascii_and_snapshots() {
    let shakuhachi_view = ShakuhachiView::new();
    let embouchure_view = EmbouchureAngleView::new();

    let shakuhachi_ascii = shakuhachi_view.render_ascii(80, 16);
    let embouchure_ascii = embouchure_view.render_ascii(80, 16);

    let mut hasher = Hasher::new();
    for line in &shakuhachi_ascii {
        hasher.update(line.as_bytes());
    }
    for line in &embouchure_ascii {
        hasher.update(line.as_bytes());
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_shakuhachi_views.hash", &digest);

    // Save PNG snapshots across the project render directories
    let _ = shakuhachi_view.render_snapshot_png("scratch/renders/shakuhachi_view.png", 800, 520);
    let _ = shakuhachi_view.render_snapshot_png("../../scratch/renders/shakuhachi_view.png", 800, 520);
    let _ = shakuhachi_view.render_snapshot_png("crates/summon/scratch/renders/shakuhachi_view.png", 800, 520);
    let _ = shakuhachi_view.render_snapshot_png("crates/summoner_gui/scratch/renders/shakuhachi_view.png", 800, 520);

    let _ = embouchure_view.render_snapshot_png("scratch/renders/embouchure_angle_view.png", 800, 520);
    let _ = embouchure_view.render_snapshot_png("../../scratch/renders/embouchure_angle_view.png", 800, 520);
    let _ = embouchure_view.render_snapshot_png("crates/summon/scratch/renders/embouchure_angle_view.png", 800, 520);
    let _ = embouchure_view.render_snapshot_png("crates/summoner_gui/scratch/renders/embouchure_angle_view.png", 800, 520);
}
