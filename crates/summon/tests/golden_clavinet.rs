// Summoner - Deterministic Golden Clavinet & Auto-Wah Regression Suite
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

use blake3::Hasher;
use std::fs;
use std::path::Path;
use summoner_core::allocator::AllocGuard;
use summoner_core::clavinet_bus::{
    ClavinetArticulation, ClavinetBus, ClavinetPickupMode,
};
use summoner_core::node::ProcessContext;
use summoner_core::param_bus::{ParamBus, ParamId};
use summoner_core::transport::Transport;
use summoner_dsp::auto_wah::{AutoWah, AutoWahDirection, AutoWahFilterMode};
use summoner_dsp::clavinet::{Clavinet, ClavinetProfile};
use summoner_dsp::traits::SignalProcessor;
#[cfg(feature = "gui")]
use summoner_gui::views::auto_wah_view::AutoWahView;
#[cfg(feature = "gui")]
use summoner_gui::views::clavinet_view::ClavinetView;
use summoner_sequencer::clavinet_gesture::{
    ClavinetGestureEngine, ClavinetGesturePattern, ClavinetInterpolationCurve, ClavinetWaypoint,
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
            "Golden clavinet hash mismatch for {}",
            hash_filename
        );
    } else {
        fs::write(&hash_file, digest).expect("Failed to write golden hash");
    }
}

#[test]
fn test_golden_clavinet_deterministic() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    // 1. Stevie Superstition Funk (Eb minor 7th: Eb3=51, Gb3=54, Bb3=58, Db4=61)
    let mut stevie = Clavinet::new(sample_rate);
    stevie.set_profile(ClavinetProfile::StevieSuperstition);
    stevie.note_on(51, 0.90);
    stevie.note_on(54, 0.85);
    stevie.note_on(58, 0.85);
    stevie.note_on(61, 0.80);

    // 2. Out-of-Phase Funk Quack (E9 chord: E3=52, G#3=56, B3=59, D4=62, F#4=66)
    let mut quack = Clavinet::new(sample_rate);
    quack.set_profile(ClavinetProfile::OutPhaseFunkQuack);
    quack.note_on(52, 0.95);
    quack.note_on(56, 0.90);
    quack.note_on(59, 0.90);
    quack.note_on(62, 0.85);
    quack.note_on(66, 0.80);

    // 3. Classic D6 Clean (C major 7th: C3=48, G3=55, E4=64, B4=71)
    let mut clean = Clavinet::new(sample_rate);
    clean.set_profile(ClavinetProfile::ClassicD6Clean);
    clean.note_on(48, 0.80);
    clean.note_on(55, 0.75);
    clean.note_on(64, 0.75);
    clean.note_on(71, 0.70);

    // 4. Mellow Chamber Clavinet (A minor: A2=45, E3=52, A3=57, C4=60)
    let mut chamber = Clavinet::new(sample_rate);
    chamber.set_profile(ClavinetProfile::MellowChamber);
    chamber.note_on(45, 0.65);
    chamber.note_on(52, 0.60);
    chamber.note_on(57, 0.60);
    chamber.note_on(60, 0.55);

    // 5. Screaming Funk Auto-Wah (F7 funk riff: F3=53, A3=57, C4=60, Eb4=63)
    let mut autowah_clav = Clavinet::new(sample_rate);
    autowah_clav.set_profile(ClavinetProfile::ScreamingAutoWah);
    autowah_clav.note_on(53, 0.95);
    autowah_clav.note_on(57, 0.90);
    autowah_clav.note_on(60, 0.90);
    autowah_clav.note_on(63, 0.85);

    // 6. Twangy Bridge Lead (D4=62, F#4=66, A4=69, D5=74)
    let mut bridge_lead = Clavinet::new(sample_rate);
    bridge_lead.set_profile(ClavinetProfile::TwangyBridgeLead);
    bridge_lead.note_on(62, 0.90);
    bridge_lead.note_on(66, 0.85);
    bridge_lead.note_on(69, 0.85);
    bridge_lead.note_on(74, 0.80);

    let mut out_stev_l = [0.0f32; BLOCK_SIZE];
    let mut out_stev_r = [0.0f32; BLOCK_SIZE];
    let mut out_quak_l = [0.0f32; BLOCK_SIZE];
    let mut out_quak_r = [0.0f32; BLOCK_SIZE];
    let mut out_clen_l = [0.0f32; BLOCK_SIZE];
    let mut out_clen_r = [0.0f32; BLOCK_SIZE];
    let mut out_chmb_l = [0.0f32; BLOCK_SIZE];
    let mut out_chmb_r = [0.0f32; BLOCK_SIZE];
    let mut out_wahc_l = [0.0f32; BLOCK_SIZE];
    let mut out_wahc_r = [0.0f32; BLOCK_SIZE];
    let mut out_brid_l = [0.0f32; BLOCK_SIZE];
    let mut out_brid_r = [0.0f32; BLOCK_SIZE];

    let mut hasher = Hasher::new();

    // Process 64 blocks of audio (4096 samples)
    for block_idx in 0..64 {
        let ctx = ProcessContext::from_transport(&transport);

        {
            let _guard = AllocGuard::new();
            stevie.process_block(&[], &mut [&mut out_stev_l[..], &mut out_stev_r[..]], &ctx);
            quack.process_block(&[], &mut [&mut out_quak_l[..], &mut out_quak_r[..]], &ctx);
            clean.process_block(&[], &mut [&mut out_clen_l[..], &mut out_clen_r[..]], &ctx);
            chamber.process_block(&[], &mut [&mut out_chmb_l[..], &mut out_chmb_r[..]], &ctx);
            autowah_clav.process_block(&[], &mut [&mut out_wahc_l[..], &mut out_wahc_r[..]], &ctx);
            bridge_lead.process_block(&[], &mut [&mut out_brid_l[..], &mut out_brid_r[..]], &ctx);
        }

        if block_idx == 32 {
            // Note off and note transition halfway
            stevie.note_off(61);
            stevie.note_on(63, 0.85); // Eb4
            quack.note_off(52);
            quack.note_on(55, 0.90); // G3
        }

        for i in 0..BLOCK_SIZE {
            assert!(out_stev_l[i].is_finite() && out_stev_r[i].is_finite());
            assert!(out_quak_l[i].is_finite() && out_quak_r[i].is_finite());
            assert!(out_clen_l[i].is_finite() && out_clen_r[i].is_finite());
            assert!(out_chmb_l[i].is_finite() && out_chmb_r[i].is_finite());
            assert!(out_wahc_l[i].is_finite() && out_wahc_r[i].is_finite());
            assert!(out_brid_l[i].is_finite() && out_brid_r[i].is_finite());

            assert!((-1.0..=1.0).contains(&out_stev_l[i]) && (-1.0..=1.0).contains(&out_stev_r[i]));
            assert!((-1.0..=1.0).contains(&out_quak_l[i]) && (-1.0..=1.0).contains(&out_quak_r[i]));
            assert!((-1.0..=1.0).contains(&out_clen_l[i]) && (-1.0..=1.0).contains(&out_clen_r[i]));
            assert!((-1.0..=1.0).contains(&out_chmb_l[i]) && (-1.0..=1.0).contains(&out_chmb_r[i]));
            assert!((-1.0..=1.0).contains(&out_wahc_l[i]) && (-1.0..=1.0).contains(&out_wahc_r[i]));
            assert!((-1.0..=1.0).contains(&out_brid_l[i]) && (-1.0..=1.0).contains(&out_brid_r[i]));

            hasher.update(&out_stev_l[i].to_le_bytes());
            hasher.update(&out_stev_r[i].to_le_bytes());
            hasher.update(&out_quak_l[i].to_le_bytes());
            hasher.update(&out_quak_r[i].to_le_bytes());
            hasher.update(&out_clen_l[i].to_le_bytes());
            hasher.update(&out_clen_r[i].to_le_bytes());
            hasher.update(&out_chmb_l[i].to_le_bytes());
            hasher.update(&out_chmb_r[i].to_le_bytes());
            hasher.update(&out_wahc_l[i].to_le_bytes());
            hasher.update(&out_wahc_r[i].to_le_bytes());
            hasher.update(&out_brid_l[i].to_le_bytes());
            hasher.update(&out_brid_r[i].to_le_bytes());
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save("golden_clavinet.hash", &digest);
}

#[test]
fn test_golden_auto_wah_filter_deterministic() {
    let sample_rate = 48000;

    let mut wah_bp = AutoWah::new();
    wah_bp.filter_mode = AutoWahFilterMode::Bandpass;
    wah_bp.set_params(0.85, 350.0, 4.0, 7.5, 0.90);

    let mut wah_lp = AutoWah::new();
    wah_lp.filter_mode = AutoWahFilterMode::Lowpass;
    wah_lp.set_params(0.70, 200.0, 3.0, 5.0, 0.85);

    let mut wah_pk = AutoWah::new();
    wah_pk.filter_mode = AutoWahFilterMode::PeakingWah;
    wah_pk.direction = AutoWahDirection::Down;
    wah_pk.set_params(0.90, 800.0, 2.5, 9.0, 1.0);

    let mut hasher = Hasher::new();

    for step in 0..2048 {
        let t = step as f32 / 48000.0;
        // Dynamic pulsed test signal
        let pulse_env = (t * 10.0 * std::f32::consts::PI).sin().abs().powi(4);
        let test_sig = (t * 440.0 * 2.0 * std::f32::consts::PI).sin() * pulse_env;

        let (s_bp_l, s_bp_r, s_lp_l, s_lp_r, s_pk_l, s_pk_r) = {
            let _guard = AllocGuard::new();
            let (bp_l, bp_r) = wah_bp.process_sample(test_sig, test_sig, sample_rate);
            let (lp_l, lp_r) = wah_lp.process_sample(test_sig, test_sig, sample_rate);
            let (pk_l, pk_r) = wah_pk.process_sample(test_sig, test_sig, sample_rate);
            (bp_l, bp_r, lp_l, lp_r, pk_l, pk_r)
        };

        assert!(s_bp_l.is_finite() && s_bp_r.is_finite());
        assert!(s_lp_l.is_finite() && s_lp_r.is_finite());
        assert!(s_pk_l.is_finite() && s_pk_r.is_finite());

        hasher.update(&s_bp_l.to_le_bytes());
        hasher.update(&s_bp_r.to_le_bytes());
        hasher.update(&s_lp_l.to_le_bytes());
        hasher.update(&s_lp_r.to_le_bytes());
        hasher.update(&s_pk_l.to_le_bytes());
        hasher.update(&s_pk_r.to_le_bytes());
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save("golden_auto_wah.hash", &digest);
}

#[test]
fn test_golden_clavinet_gesture_and_bus_deterministic() {
    let mut engine = ClavinetGestureEngine::new(ClavinetGesturePattern::CustomSpline);
    engine.default_curve = ClavinetInterpolationCurve::CubicCatmullRom;
    engine.loop_length_beats = 8.0;
    engine.loop_enabled = true;

    engine.waypoints = vec![
        ClavinetWaypoint {
            beat: 0.0,
            pickup_mode: ClavinetPickupMode::ParallelInPhase,
            pickup_blend: 0.50,
            pickup_phase_deg: 0.0,
            anvil_hardness: 0.75,
            yarn_damping: 0.70,
            tone_switches_mask: 1 | 2 | 4,
            auto_wah_enabled: false,
            auto_wah_sensitivity: 0.70,
            auto_wah_freq_hz: 350.0,
            auto_wah_resonance_q: 6.0,
            auto_wah_mix: 0.85,
            articulation: ClavinetArticulation::StevieFunk,
            curve: ClavinetInterpolationCurve::CubicCatmullRom,
        },
        ClavinetWaypoint {
            beat: 2.0,
            pickup_mode: ClavinetPickupMode::ParallelOutOfPhase,
            pickup_blend: 0.50,
            pickup_phase_deg: 180.0,
            anvil_hardness: 0.90,
            yarn_damping: 0.85,
            tone_switches_mask: 1 | 2,
            auto_wah_enabled: true,
            auto_wah_sensitivity: 0.85,
            auto_wah_freq_hz: 450.0,
            auto_wah_resonance_q: 8.5,
            auto_wah_mix: 0.95,
            articulation: ClavinetArticulation::ScreamingAutoWah,
            curve: ClavinetInterpolationCurve::CubicCatmullRom,
        },
        ClavinetWaypoint {
            beat: 4.0,
            pickup_mode: ClavinetPickupMode::BridgeOnly,
            pickup_blend: 1.0,
            pickup_phase_deg: 0.0,
            anvil_hardness: 0.95,
            yarn_damping: 0.90,
            tone_switches_mask: 1 | 2,
            auto_wah_enabled: false,
            auto_wah_sensitivity: 0.60,
            auto_wah_freq_hz: 300.0,
            auto_wah_resonance_q: 5.0,
            auto_wah_mix: 0.80,
            articulation: ClavinetArticulation::TwangyBridge,
            curve: ClavinetInterpolationCurve::CubicCatmullRom,
        },
        ClavinetWaypoint {
            beat: 6.0,
            pickup_mode: ClavinetPickupMode::ParallelInPhase,
            pickup_blend: 0.50,
            pickup_phase_deg: 0.0,
            anvil_hardness: 0.75,
            yarn_damping: 0.70,
            tone_switches_mask: 1 | 2 | 4,
            auto_wah_enabled: false,
            auto_wah_sensitivity: 0.70,
            auto_wah_freq_hz: 350.0,
            auto_wah_resonance_q: 6.0,
            auto_wah_mix: 0.85,
            articulation: ClavinetArticulation::StevieFunk,
            curve: ClavinetInterpolationCurve::CubicCatmullRom,
        },
    ];

    let bus = ClavinetBus::new();
    let mut param_bus = ParamBus::new();
    for i in 0..20 {
        param_bus.register(ParamId(980 + i), 0.0);
    }

    let mut hasher = Hasher::new();

    for step in 0..160 {
        let beat = step as f64 * 0.05; // 8 beats total

        let snap = {
            let _guard = AllocGuard::new();
            engine.apply_to_bus(beat, &bus);
            engine.dispatch_to_param_bus(beat, &param_bus, ParamId(980));
            bus.snapshot()
        };

        assert!(snap.pickup_blend.is_finite());
        assert!(snap.pickup_phase_deg.is_finite());
        assert!(snap.anvil_hardness.is_finite());
        assert!(snap.yarn_damping.is_finite());
        assert!(snap.auto_wah_sensitivity.is_finite());
        assert!(snap.auto_wah_freq_hz.is_finite());
        assert!(snap.auto_wah_resonance_q.is_finite());
        assert!(snap.auto_wah_mix.is_finite());
        assert!(snap.gesture_progress.is_finite());

        hasher.update(&(snap.articulation as u32).to_le_bytes());
        hasher.update(&(snap.pickup_mode as u32).to_le_bytes());
        hasher.update(&snap.pickup_blend.to_le_bytes());
        hasher.update(&snap.pickup_phase_deg.to_le_bytes());
        hasher.update(&snap.anvil_hardness.to_le_bytes());
        hasher.update(&snap.yarn_damping.to_le_bytes());
        hasher.update(&snap.tone_switches_mask.to_le_bytes());
        hasher.update(&(if snap.auto_wah_enabled { 1u32 } else { 0u32 }).to_le_bytes());
        hasher.update(&snap.auto_wah_sensitivity.to_le_bytes());
        hasher.update(&snap.auto_wah_freq_hz.to_le_bytes());
        hasher.update(&snap.auto_wah_resonance_q.to_le_bytes());
        hasher.update(&snap.auto_wah_mix.to_le_bytes());
        hasher.update(&snap.gesture_progress.to_le_bytes());
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save("golden_clavinet_gesture.hash", &digest);
}

#[cfg(feature = "gui")]
#[test]
fn test_golden_clavinet_views_ascii_and_snapshots() {
    let clav_view = ClavinetView::new();
    let wah_view = AutoWahView::new();

    let clav_ascii = clav_view.render_ascii(80, 16);
    let wah_ascii = wah_view.render_ascii(80, 16);

    let mut hasher = Hasher::new();
    for line in &clav_ascii {
        hasher.update(line.as_bytes());
    }
    for line in &wah_ascii {
        hasher.update(line.as_bytes());
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save("golden_clavinet_views.hash", &digest);

    // Save PNG snapshots across the project render directories
    let _ = clav_view.render_snapshot_png("scratch/renders/clavinet_view.png", 800, 520);
    let _ = clav_view.render_snapshot_png("../../scratch/renders/clavinet_view.png", 800, 520);
    let _ = clav_view.render_snapshot_png("crates/summon/scratch/renders/clavinet_view.png", 800, 520);
    let _ = clav_view.render_snapshot_png("crates/summoner_gui/scratch/renders/clavinet_view.png", 800, 520);

    let _ = wah_view.render_snapshot_png("scratch/renders/auto_wah_view.png", 800, 520);
    let _ = wah_view.render_snapshot_png("../../scratch/renders/auto_wah_view.png", 800, 520);
    let _ = wah_view.render_snapshot_png("crates/summon/scratch/renders/auto_wah_view.png", 800, 520);
    let _ = wah_view.render_snapshot_png("crates/summoner_gui/scratch/renders/auto_wah_view.png", 800, 520);
}

fn check_or_save(filename: &str, digest: &str) {
    check_or_save_golden(filename, digest);
}
