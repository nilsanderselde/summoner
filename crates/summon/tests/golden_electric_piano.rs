// Summoner - Deterministic Golden Electric Piano Regression Suite
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

use blake3::Hasher;
use std::fs;
use std::path::Path;
use summoner_core::allocator::AllocGuard;
use summoner_core::ep_bus::{EpArticulation, EpBus, EpModelType};
use summoner_core::node::ProcessContext;
use summoner_core::param_bus::{ParamBus, ParamId};
use summoner_core::transport::Transport;
use summoner_dsp::electric_piano::{ElectricPiano, ElectricPianoProfile};
use summoner_dsp::tine_resonator::{ElectricPianoModel, TineResonator};
use summoner_dsp::traits::SignalProcessor;
#[cfg(feature = "gui")]
use summoner_gui::views::electric_piano_view::ElectricPianoView;
#[cfg(feature = "gui")]
use summoner_gui::views::tine_resonator_view::TineResonatorView;
use summoner_sequencer::ep_gesture::{
    EpGestureEngine, EpGesturePattern, EpInterpolationCurve, EpWaypoint,
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
            "Golden electric piano hash mismatch for {}",
            hash_filename
        );
    } else {
        fs::write(&hash_file, digest).expect("Failed to write golden hash");
    }
}

#[test]
fn test_golden_electric_piano_deterministic() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    // 1. Classic Rhodes Suitcase (C major 7th chord: C4=60, E4=64, G4=67, B4=71)
    let mut suitcase = ElectricPiano::new(sample_rate);
    suitcase.set_profile(ElectricPianoProfile::ClassicRhodesSuitcase);
    suitcase.note_on(60, 0.85);
    suitcase.note_on(64, 0.80);
    suitcase.note_on(67, 0.80);
    suitcase.note_on(71, 0.75);

    // 2. Barking Dyno Rhodes (A3 minor triad: A3=57, C4=60, E4=64)
    let mut dyno = ElectricPiano::new(sample_rate);
    dyno.set_profile(ElectricPianoProfile::BarkingDynoRhodes);
    dyno.note_on(57, 0.95);
    dyno.note_on(60, 0.90);
    dyno.note_on(64, 0.90);

    // 3. Mellow Rhodes Stage 73 (F3, A3, C4, E4)
    let mut stage = ElectricPiano::new(sample_rate);
    stage.set_profile(ElectricPianoProfile::MellowRhodesStage);
    stage.note_on(53, 0.70);
    stage.note_on(57, 0.65);
    stage.note_on(60, 0.65);
    stage.note_on(64, 0.65);

    // 4. Classic Wurlitzer 200A Reed (D3, F#3=54, A3, C4)
    let mut wurli = ElectricPiano::new(sample_rate);
    wurli.set_profile(ElectricPianoProfile::ClassicWurlitzer200A);
    wurli.note_on(50, 0.85);
    wurli.note_on(54, 0.80);
    wurli.note_on(57, 0.80);
    wurli.note_on(60, 0.75);

    // 5. Soul Overdriven Wurlitzer (G2=43, D3=50, G3=55, B3=59)
    let mut soul = ElectricPiano::new(sample_rate);
    soul.set_profile(ElectricPianoProfile::SoulOverdrivenWurli);
    soul.note_on(43, 0.95);
    soul.note_on(50, 0.90);
    soul.note_on(55, 0.90);
    soul.note_on(59, 0.85);

    // 6. Belled Ambient Rhodes (Eb3=51, Bb3=58, G4=67)
    let mut ambient = ElectricPiano::new(sample_rate);
    ambient.set_profile(ElectricPianoProfile::BelledAmbientRhodes);
    ambient.note_on(51, 0.75);
    ambient.note_on(58, 0.75);
    ambient.note_on(67, 0.70);

    let mut out_suit_l = [0.0f32; BLOCK_SIZE];
    let mut out_suit_r = [0.0f32; BLOCK_SIZE];
    let mut out_dyno_l = [0.0f32; BLOCK_SIZE];
    let mut out_dyno_r = [0.0f32; BLOCK_SIZE];
    let mut out_stag_l = [0.0f32; BLOCK_SIZE];
    let mut out_stag_r = [0.0f32; BLOCK_SIZE];
    let mut out_wurl_l = [0.0f32; BLOCK_SIZE];
    let mut out_wurl_r = [0.0f32; BLOCK_SIZE];
    let mut out_soul_l = [0.0f32; BLOCK_SIZE];
    let mut out_soul_r = [0.0f32; BLOCK_SIZE];
    let mut out_ambi_l = [0.0f32; BLOCK_SIZE];
    let mut out_ambi_r = [0.0f32; BLOCK_SIZE];

    let mut hasher = Hasher::new();

    // Process 64 blocks of audio (4096 samples)
    for block_idx in 0..64 {
        let ctx = ProcessContext::from_transport(&transport);

        {
            let _guard = AllocGuard::new();
            suitcase.process_block(&[], &mut [&mut out_suit_l[..], &mut out_suit_r[..]], &ctx);
            dyno.process_block(&[], &mut [&mut out_dyno_l[..], &mut out_dyno_r[..]], &ctx);
            stage.process_block(&[], &mut [&mut out_stag_l[..], &mut out_stag_r[..]], &ctx);
            wurli.process_block(&[], &mut [&mut out_wurl_l[..], &mut out_wurl_r[..]], &ctx);
            soul.process_block(&[], &mut [&mut out_soul_l[..], &mut out_soul_r[..]], &ctx);
            ambient.process_block(&[], &mut [&mut out_ambi_l[..], &mut out_ambi_r[..]], &ctx);
        }

        if block_idx == 32 {
            // Note off and note transition halfway
            suitcase.note_off(71);
            suitcase.note_on(72, 0.85); // High C5
            wurli.note_off(50);
            wurli.note_on(52, 0.80); // E3
        }

        for i in 0..BLOCK_SIZE {
            assert!(out_suit_l[i].is_finite() && out_suit_r[i].is_finite());
            assert!(out_dyno_l[i].is_finite() && out_dyno_r[i].is_finite());
            assert!(out_stag_l[i].is_finite() && out_stag_r[i].is_finite());
            assert!(out_wurl_l[i].is_finite() && out_wurl_r[i].is_finite());
            assert!(out_soul_l[i].is_finite() && out_soul_r[i].is_finite());
            assert!(out_ambi_l[i].is_finite() && out_ambi_r[i].is_finite());

            assert!((-1.0..=1.0).contains(&out_suit_l[i]) && (-1.0..=1.0).contains(&out_suit_r[i]));
            assert!((-1.0..=1.0).contains(&out_dyno_l[i]) && (-1.0..=1.0).contains(&out_dyno_r[i]));
            assert!((-1.0..=1.0).contains(&out_stag_l[i]) && (-1.0..=1.0).contains(&out_stag_r[i]));
            assert!((-1.0..=1.0).contains(&out_wurl_l[i]) && (-1.0..=1.0).contains(&out_wurl_r[i]));
            assert!((-1.0..=1.0).contains(&out_soul_l[i]) && (-1.0..=1.0).contains(&out_soul_r[i]));
            assert!((-1.0..=1.0).contains(&out_ambi_l[i]) && (-1.0..=1.0).contains(&out_ambi_r[i]));

            hasher.update(&out_suit_l[i].to_le_bytes());
            hasher.update(&out_suit_r[i].to_le_bytes());
            hasher.update(&out_dyno_l[i].to_le_bytes());
            hasher.update(&out_dyno_r[i].to_le_bytes());
            hasher.update(&out_stag_l[i].to_le_bytes());
            hasher.update(&out_stag_r[i].to_le_bytes());
            hasher.update(&out_wurl_l[i].to_le_bytes());
            hasher.update(&out_wurl_r[i].to_le_bytes());
            hasher.update(&out_soul_l[i].to_le_bytes());
            hasher.update(&out_soul_r[i].to_le_bytes());
            hasher.update(&out_ambi_l[i].to_le_bytes());
            hasher.update(&out_ambi_r[i].to_le_bytes());
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save("golden_electric_piano.hash", &digest);
}

#[test]
fn test_golden_tine_resonator_and_inductive_pickup_deterministic() {
    let sample_rate = 48000;

    let mut rhodes_tine = TineResonator::new(ElectricPianoModel::RhodesTine, 261.63, sample_rate);
    rhodes_tine.note_on(60, 261.63, 0.90, 0.65);

    let mut wurli_reed = TineResonator::new(ElectricPianoModel::WurlitzerReed, 440.0, sample_rate);
    wurli_reed.note_on(69, 440.0, 0.85, 0.80);

    let mut dyno_tine = TineResonator::new(ElectricPianoModel::RhodesTine, 523.25, sample_rate);
    dyno_tine.pickup.air_gap_mm = 0.9;
    dyno_tine.pickup.bark_drive = 0.95;
    dyno_tine.tonebar.coupling_stiffness = 0.90;
    dyno_tine.note_on(72, 523.25, 0.98, 0.90);

    let mut hasher = Hasher::new();

    for step in 0..2048 {
        if step == 1024 {
            rhodes_tine.note_off();
            wurli_reed.note_off();
            dyno_tine.note_off();
        }

        let (s_rhodes, s_wurli, s_dyno) = {
            let _guard = AllocGuard::new();
            (
                rhodes_tine.process_sample(),
                wurli_reed.process_sample(),
                dyno_tine.process_sample(),
            )
        };

        assert!(s_rhodes.is_finite());
        assert!(s_wurli.is_finite());
        assert!(s_dyno.is_finite());

        hasher.update(&s_rhodes.to_le_bytes());
        hasher.update(&s_wurli.to_le_bytes());
        hasher.update(&s_dyno.to_le_bytes());
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save("golden_tine_resonator.hash", &digest);
}

#[test]
fn test_golden_ep_gesture_and_ep_bus_deterministic() {
    let mut engine = EpGestureEngine::new(EpGesturePattern::CustomSpline);
    engine.default_curve = EpInterpolationCurve::CubicCatmullRom;
    engine.loop_length_beats = 8.0;
    engine.loop_enabled = true;

    engine.waypoints = vec![
        EpWaypoint {
            beat: 0.0,
            model_type: EpModelType::RhodesTine,
            hammer_hardness: 0.40,
            air_gap_mm: 2.0,
            bark_drive: 0.40,
            tonebar_coupling: 0.75,
            damper_clunk_volume: 0.25,
            tremolo_enabled: true,
            tremolo_rate_hz: 4.8,
            tremolo_depth: 0.40,
            tremolo_stereo: true,
            tube_drive_db: 3.0,
            bass_db: 2.0,
            treble_db: 2.0,
            articulation: EpArticulation::BalladChime,
            curve: EpInterpolationCurve::CubicCatmullRom,
        },
        EpWaypoint {
            beat: 2.0,
            model_type: EpModelType::RhodesTine,
            hammer_hardness: 0.75,
            air_gap_mm: 1.2,
            bark_drive: 0.80,
            tonebar_coupling: 0.85,
            damper_clunk_volume: 0.35,
            tremolo_enabled: true,
            tremolo_rate_hz: 5.6,
            tremolo_depth: 0.75,
            tremolo_stereo: true,
            tube_drive_db: 7.0,
            bass_db: 1.0,
            treble_db: 5.0,
            articulation: EpArticulation::BarkingDyno,
            curve: EpInterpolationCurve::CubicCatmullRom,
        },
        EpWaypoint {
            beat: 4.0,
            model_type: EpModelType::WurlitzerReed,
            hammer_hardness: 0.85,
            air_gap_mm: 0.9,
            bark_drive: 0.95,
            tonebar_coupling: 0.40,
            damper_clunk_volume: 0.50,
            tremolo_enabled: true,
            tremolo_rate_hz: 7.0,
            tremolo_depth: 0.50,
            tremolo_stereo: false,
            tube_drive_db: 14.0,
            bass_db: 2.5,
            treble_db: 6.0,
            articulation: EpArticulation::SoulOverdrive,
            curve: EpInterpolationCurve::CubicCatmullRom,
        },
        EpWaypoint {
            beat: 6.0,
            model_type: EpModelType::RhodesTine,
            hammer_hardness: 0.50,
            air_gap_mm: 1.8,
            bark_drive: 0.55,
            tonebar_coupling: 0.70,
            damper_clunk_volume: 0.35,
            tremolo_enabled: true,
            tremolo_rate_hz: 5.2,
            tremolo_depth: 0.65,
            tremolo_stereo: true,
            tube_drive_db: 4.0,
            bass_db: 1.5,
            treble_db: 2.0,
            articulation: EpArticulation::ClassicSuitcase,
            curve: EpInterpolationCurve::CubicCatmullRom,
        },
    ];

    let bus = EpBus::new();
    let mut param_bus = ParamBus::new();
    for i in 0..20 {
        param_bus.register(ParamId(950 + i), 0.0);
    }

    let mut hasher = Hasher::new();

    for step in 0..160 {
        let beat = step as f64 * 0.05; // 8 beats total

        let snap = {
            let _guard = AllocGuard::new();
            engine.apply_to_bus(beat, &bus);
            engine.dispatch_to_param_bus(beat, &param_bus, ParamId(950));
            bus.snapshot()
        };

        assert!(snap.hammer_hardness.is_finite());
        assert!(snap.air_gap_mm.is_finite());
        assert!(snap.bark_drive.is_finite());
        assert!(snap.tonebar_coupling.is_finite());
        assert!(snap.damper_clunk_volume.is_finite());
        assert!(snap.tremolo_rate_hz.is_finite());
        assert!(snap.tremolo_depth.is_finite());
        assert!(snap.tube_drive_db.is_finite());
        assert!(snap.bass_db.is_finite());
        assert!(snap.treble_db.is_finite());
        assert!(snap.gesture_progress.is_finite());

        hasher.update(&(snap.articulation as u32).to_le_bytes());
        hasher.update(&(snap.model_type as u32).to_le_bytes());
        hasher.update(&snap.hammer_hardness.to_le_bytes());
        hasher.update(&snap.air_gap_mm.to_le_bytes());
        hasher.update(&snap.bark_drive.to_le_bytes());
        hasher.update(&snap.tonebar_coupling.to_le_bytes());
        hasher.update(&snap.damper_clunk_volume.to_le_bytes());
        hasher.update(&snap.tremolo_rate_hz.to_le_bytes());
        hasher.update(&snap.tremolo_depth.to_le_bytes());
        hasher.update(&snap.tube_drive_db.to_le_bytes());
        hasher.update(&snap.bass_db.to_le_bytes());
        hasher.update(&snap.treble_db.to_le_bytes());
        hasher.update(&snap.gesture_progress.to_le_bytes());
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save("golden_ep_gesture.hash", &digest);
}

#[cfg(feature = "gui")]
#[test]
fn test_golden_electric_piano_views_ascii_and_snapshots() {
    let ep_view = ElectricPianoView::new();
    let tine_view = TineResonatorView::new();

    let ep_ascii = ep_view.render_ascii(80, 16);
    let tine_ascii = tine_view.render_ascii(80, 16);

    let mut hasher = Hasher::new();
    for line in &ep_ascii {
        hasher.update(line.as_bytes());
    }
    for line in &tine_ascii {
        hasher.update(line.as_bytes());
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save("golden_electric_piano_views.hash", &digest);

    // Save PNG snapshots across the project render directories
    let _ = ep_view.render_snapshot_png("scratch/renders/electric_piano_view.png", 800, 520);
    let _ = ep_view.render_snapshot_png("../../scratch/renders/electric_piano_view.png", 800, 520);
    let _ = ep_view.render_snapshot_png("crates/summon/scratch/renders/electric_piano_view.png", 800, 520);
    let _ = ep_view.render_snapshot_png("crates/summoner_gui/scratch/renders/electric_piano_view.png", 800, 520);

    let _ = tine_view.render_snapshot_png("scratch/renders/tine_resonator_view.png", 800, 520);
    let _ = tine_view.render_snapshot_png("../../scratch/renders/tine_resonator_view.png", 800, 520);
    let _ = tine_view.render_snapshot_png("crates/summon/scratch/renders/tine_resonator_view.png", 800, 520);
    let _ = tine_view.render_snapshot_png("crates/summoner_gui/scratch/renders/tine_resonator_view.png", 800, 520);
}

fn check_or_save(filename: &str, digest: &str) {
    check_or_save_golden(filename, digest);
}
