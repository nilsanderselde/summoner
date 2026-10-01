// Summoner - Deterministic Golden Glass Armonica & Crystal Singing Bowl Physical Modeling Regression Suite
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

use blake3::Hasher;
use std::f32::consts::PI;
use std::fs;
use std::path::Path;
use summoner_core::allocator::AllocGuard;
use summoner_core::glass_bus::{
    register_glass_armonica_params, GlassArmonicaBus, GLASS_ARMONICA_BASE_PARAM_ID,
};
use summoner_core::node::ProcessContext;
use summoner_core::param_bus::ParamBus;
use summoner_core::transport::Transport;
use summoner_dsp::crystal_resonator::{CrystalMaterialProfile, CrystalResonator};
use summoner_dsp::glass_armonica::{ArmonicaProfile, GlassArmonica};
use summoner_dsp::traits::SignalProcessor;
#[cfg(feature = "gui")]
use summoner_gui::views::crystal_resonator_view::CrystalResonatorView;
#[cfg(feature = "gui")]
use summoner_gui::views::glass_armonica_view::GlassArmonicaView;
use summoner_sequencer::glass_gesture::{
    GlassGestureEngine, GlassGesturePattern,
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
            "Golden Glass Armonica hash mismatch for {}",
            hash_filename
        );
    } else {
        fs::write(&hash_file, digest).expect("Failed to write golden hash");
    }
}

#[test]
fn test_golden_glass_armonica_deterministic() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    // 1. Franklin Authentic 1761 Quartz Armonica
    let mut franklin = GlassArmonica::new(sample_rate);
    franklin.set_profile(ArmonicaProfile::FranklinAuthentic1761);
    franklin.set_spindle_speed(2.5 * PI);
    franklin.touch_note(60, 0.75); // C4
    franklin.touch_note(72, 0.60); // C5

    // 2. Mesmer Magnetic Healing Chalice
    let mut mesmer = GlassArmonica::new(sample_rate);
    mesmer.set_profile(ArmonicaProfile::MesmerHealingChalice);
    mesmer.set_spindle_speed(1.5 * PI);
    mesmer.set_water_fill(0.35);
    mesmer.touch_note(64, 0.80); // E4
    mesmer.touch_note(67, 0.70); // G4

    // 3. Mozart Concert Virtuoso Armonica
    let mut mozart = GlassArmonica::new(sample_rate);
    mozart.set_profile(ArmonicaProfile::MozartConcertArmonica);
    mozart.set_spindle_speed(3.2 * PI);
    mozart.touch_note(69, 0.85); // A4
    mozart.touch_note(76, 0.65); // E5

    // 4. Ethereal Borosilicate Armonica
    let mut ethereal = GlassArmonica::new(sample_rate);
    ethereal.set_profile(ArmonicaProfile::EtherealBorosilicate);
    ethereal.set_spindle_speed(2.2 * PI);
    ethereal.set_water_fill(0.15);
    ethereal.touch_note(79, 0.70); // G5
    ethereal.touch_note(84, 0.60); // C6

    // 5. Water-Tuned Crystal Chime
    let mut chime = GlassArmonica::new(sample_rate);
    chime.set_profile(ArmonicaProfile::WaterTunedCrystalChime);
    chime.set_spindle_speed(2.0 * PI);
    chime.set_water_fill(0.50);
    chime.touch_note(88, 0.75); // E6

    // 6. Quartz Crystal Resonator (432 Hz Pure)
    let mut crystal_pure = CrystalResonator::new(432.0, sample_rate);
    crystal_pure.set_profile(CrystalMaterialProfile::PureQuartzCrystal);
    crystal_pure.strike_mallet(0.85);
    crystal_pure.exciter.wand_speed_rad_s = 2.5;
    crystal_pure.exciter.normal_force_n = 0.55;

    // 7. Franklin Quartz Glass Chalice Resonator
    let mut crystal_chalice = CrystalResonator::new(523.25, sample_rate);
    crystal_chalice.set_profile(CrystalMaterialProfile::FranklinQuartzGlass);
    crystal_chalice.set_water_fill_level(0.20);
    crystal_chalice.strike_mallet(0.60);
    crystal_chalice.exciter.wand_speed_rad_s = 3.0;
    crystal_chalice.exciter.normal_force_n = 0.40;

    // 8. Wet Crystal Wine Goblet Resonator
    let mut crystal_goblet = CrystalResonator::new(659.25, sample_rate);
    crystal_goblet.set_profile(CrystalMaterialProfile::WetCrystalGoblet);
    crystal_goblet.set_water_fill_level(0.40);
    crystal_goblet.exciter.wand_speed_rad_s = 3.8;
    crystal_goblet.exciter.normal_force_n = 0.70;

    let mut out_frank_l = [0.0f32; BLOCK_SIZE];
    let mut out_frank_r = [0.0f32; BLOCK_SIZE];
    let mut out_mesm_l = [0.0f32; BLOCK_SIZE];
    let mut out_mesm_r = [0.0f32; BLOCK_SIZE];
    let mut out_moza_l = [0.0f32; BLOCK_SIZE];
    let mut out_moza_r = [0.0f32; BLOCK_SIZE];
    let mut out_ethe_l = [0.0f32; BLOCK_SIZE];
    let mut out_ethe_r = [0.0f32; BLOCK_SIZE];
    let mut out_chim_l = [0.0f32; BLOCK_SIZE];
    let mut out_chim_r = [0.0f32; BLOCK_SIZE];

    let mut hasher = Hasher::new();

    // Process 64 blocks of audio (4096 samples)
    for block_idx in 0..64 {
        let ctx = ProcessContext::from_transport(&transport);

        {
            let _guard = AllocGuard::new();
            franklin.process_block(&[], &mut [&mut out_frank_l[..], &mut out_frank_r[..]], &ctx);
            mesmer.process_block(&[], &mut [&mut out_mesm_l[..], &mut out_mesm_r[..]], &ctx);
            mozart.process_block(&[], &mut [&mut out_moza_l[..], &mut out_moza_r[..]], &ctx);
            ethereal.process_block(&[], &mut [&mut out_ethe_l[..], &mut out_ethe_r[..]], &ctx);
            chime.process_block(&[], &mut [&mut out_chim_l[..], &mut out_chim_r[..]], &ctx);
        }

        if block_idx == 32 {
            // Mid-phrase spindle acceleration and water filling pitch shifts
            franklin.set_spindle_speed(3.0 * PI);
            franklin.touch_note(79, 0.70); // G5

            mesmer.set_water_fill(0.60);
            mesmer.set_spindle_speed(1.8 * PI);

            mozart.set_spindle_speed(3.8 * PI);
            mozart.touch_note(84, 0.80); // C6

            chime.set_water_fill(0.75);

            crystal_pure.strike_mallet(0.95);
            crystal_chalice.set_water_fill_level(0.50);
            crystal_goblet.exciter.wand_speed_rad_s = 4.5;
            crystal_goblet.exciter.normal_force_n = 0.90;
        }

        for i in 0..BLOCK_SIZE {
            let (pure_s, _, _) = crystal_pure.process_sample();
            let (chal_s, _, _) = crystal_chalice.process_sample();
            let (gobl_s, _, _) = crystal_goblet.process_sample();

            assert!(out_frank_l[i].is_finite() && out_frank_r[i].is_finite());
            assert!(out_mesm_l[i].is_finite() && out_mesm_r[i].is_finite());
            assert!(out_moza_l[i].is_finite() && out_moza_r[i].is_finite());
            assert!(out_ethe_l[i].is_finite() && out_ethe_r[i].is_finite());
            assert!(out_chim_l[i].is_finite() && out_chim_r[i].is_finite());
            assert!(pure_s.is_finite());
            assert!(chal_s.is_finite());
            assert!(gobl_s.is_finite());

            assert!((-1.0..=1.0).contains(&out_frank_l[i]) && (-1.0..=1.0).contains(&out_frank_r[i]));
            assert!((-1.0..=1.0).contains(&out_mesm_l[i]) && (-1.0..=1.0).contains(&out_mesm_r[i]));
            assert!((-1.0..=1.0).contains(&out_moza_l[i]) && (-1.0..=1.0).contains(&out_moza_r[i]));
            assert!((-1.0..=1.0).contains(&out_ethe_l[i]) && (-1.0..=1.0).contains(&out_ethe_r[i]));
            assert!((-1.0..=1.0).contains(&out_chim_l[i]) && (-1.0..=1.0).contains(&out_chim_r[i]));

            hasher.update(&out_frank_l[i].to_le_bytes());
            hasher.update(&out_frank_r[i].to_le_bytes());
            hasher.update(&out_mesm_l[i].to_le_bytes());
            hasher.update(&out_mesm_r[i].to_le_bytes());
            hasher.update(&out_moza_l[i].to_le_bytes());
            hasher.update(&out_moza_r[i].to_le_bytes());
            hasher.update(&out_ethe_l[i].to_le_bytes());
            hasher.update(&out_ethe_r[i].to_le_bytes());
            hasher.update(&out_chim_l[i].to_le_bytes());
            hasher.update(&out_chim_r[i].to_le_bytes());
            hasher.update(&pure_s.to_le_bytes());
            hasher.update(&chal_s.to_le_bytes());
            hasher.update(&gobl_s.to_le_bytes());
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_glass_armonica.hash", &digest);
}

#[test]
fn test_golden_glass_armonica_friction_and_doublets_deterministic() {
    // 1. Thin-shell modal doublet splitting accuracy
    for profile in [
        CrystalMaterialProfile::PureQuartzCrystal,
        CrystalMaterialProfile::FranklinQuartzGlass,
        CrystalMaterialProfile::WetCrystalGoblet,
        CrystalMaterialProfile::BorosilicateBell,
        CrystalMaterialProfile::MetallophoneAlloy,
    ] {
        let mut crystal = CrystalResonator::new(432.0, 48000);
        crystal.set_profile(profile);
        crystal.recalculate_modes();

        let (_q, _stiff, expected_split, _, _) = profile.nominal_physics();

        let f0_a = crystal.modes[0].freq_hz;
        let f0_b = crystal.modes[1].freq_hz;
        let actual_split = (f0_b - f0_a).abs();

        assert!(
            (actual_split - expected_split).abs() < 0.05,
            "Mode doublet split mismatch for {:?}: actual={}, expected={}",
            profile,
            actual_split,
            expected_split
        );
    }

    // 2. Water mass loading continuous pitch shift monotonicity
    let mut bowl = CrystalResonator::new(440.0, 48000);
    bowl.set_water_fill_level(0.0);
    let f0_dry = bowl.modes[0].freq_hz;

    bowl.set_water_fill_level(0.3);
    let f0_low = bowl.modes[0].freq_hz;
    assert!(f0_low < f0_dry, "Water fill 0.3 must reduce fundamental frequency");

    bowl.set_water_fill_level(0.7);
    let f0_mid = bowl.modes[0].freq_hz;
    assert!(f0_mid < f0_low, "Water fill 0.7 must further reduce frequency");

    bowl.set_water_fill_level(1.0);
    let f0_full = bowl.modes[0].freq_hz;
    assert!(f0_full < f0_mid, "Water fill 1.0 must produce lowest frequency");

    // 3. Stribeck wet-finger stick-slip excitation energy conservation & decay
    let mut armonica = GlassArmonica::new(48000);
    armonica.touch_note(60, 0.25);
    armonica.set_spindle_speed(1.5 * PI);

    let mut max_excited_amp = 0.0f32;
    for _ in 0..4000 {
        let (s_l, s_r) = armonica.process_stereo_frame();
        max_excited_amp = max_excited_amp.max(s_l.abs()).max(s_r.abs());
    }
    assert!(max_excited_amp > 0.0, "Rubbed armonica bowl must generate audio");

    // Release finger contact
    armonica.release_note(60);
    for _ in 0..48000 {
        armonica.process_stereo_frame();
    }
    let mut max_decay_amp = 0.0f32;
    for _ in 0..200 {
        let (s_l, s_r) = armonica.process_stereo_frame();
        max_decay_amp = max_decay_amp.max(s_l.abs()).max(s_r.abs());
    }
    assert!(
        max_decay_amp < max_excited_amp,
        "Armonica ringing must decay naturally when finger is released: decay={}, max={}",
        max_decay_amp,
        max_excited_amp
    );
}

#[test]
fn test_glass_armonica_gesture_dispatch_and_param_bus_roundtrip() {
    let mut param_bus = ParamBus::new();
    register_glass_armonica_params(&mut param_bus, GLASS_ARMONICA_BASE_PARAM_ID);
    let bus = GlassArmonicaBus::new();

    for pattern in [
        GlassGesturePattern::FranklinAuthenticContinuous { phrase_length_beats: 4.0 },
        GlassGesturePattern::MesmerMagneticHealing { phrase_length_beats: 4.0, water_swell_depth: 0.35 },
        GlassGesturePattern::MozartVirtuosoAdagio { phrase_length_beats: 4.0 },
        GlassGesturePattern::EtherealBorosilicateSwell { phrase_length_beats: 4.0 },
        GlassGesturePattern::WaterTunedCrystalGlissando { phrase_length_beats: 6.0, max_water_level: 0.8 },
        GlassGesturePattern::CrystalSingingBowlMeditation { phrase_length_beats: 4.0, mallet_interval_beats: 1.0 },
    ] {
        let engine = GlassGestureEngine::with_pattern(pattern);

        for beat in [0.0, 1.0, 2.5, 3.8] {
            engine.dispatch_to_bus(beat, &bus, Some(&param_bus), GLASS_ARMONICA_BASE_PARAM_ID);
            let snap = bus.snapshot();

            assert!(snap.spindle_speed_rad_s > 0.0);
            assert!(snap.normal_force_n > 0.0);
            assert!(snap.water_fill_level >= 0.0);
            assert!(snap.moisture_lubrication > 0.0);
            assert!(snap.chassis_resonance > 0.0);
            assert!(snap.q_scale > 0.0);
            assert!(snap.master_gain > 0.0);

            let bus_speed = param_bus.get(summoner_core::param_bus::ParamId(GLASS_ARMONICA_BASE_PARAM_ID.0 + 1));
            assert!(bus_speed.is_some());
            assert!((bus_speed.unwrap() - snap.spindle_speed_rad_s).abs() < 1e-4);
        }
    }
}

#[test]
#[cfg(feature = "gui")]
fn test_glass_armonica_gui_views_headless_snapshots() {
    let armonica_view = GlassArmonicaView::new();

    let armonica_png_paths = [
        "scratch/renders/glass_armonica_view.png",
        "crates/summoner_gui/scratch/renders/glass_armonica_view.png",
        "crates/summon/scratch/renders/glass_armonica_view.png",
        "../../scratch/renders/glass_armonica_view.png",
    ];
    for path in armonica_png_paths {
        let res = armonica_view.render_snapshot_png(path, 800, 520);
        assert!(res.is_ok(), "Failed to render GlassArmonicaView PNG to {}", path);
    }

    let crystal_view = CrystalResonatorView::new();

    let crystal_png_paths = [
        "scratch/renders/crystal_resonator_view.png",
        "crates/summoner_gui/scratch/renders/crystal_resonator_view.png",
        "crates/summon/scratch/renders/crystal_resonator_view.png",
        "../../scratch/renders/crystal_resonator_view.png",
    ];
    for path in crystal_png_paths {
        let res = crystal_view.render_snapshot_png(path, 800, 520);
        assert!(res.is_ok(), "Failed to render CrystalResonatorView PNG to {}", path);
    }
}
