// Summoner - Deterministic Golden Tonewheel Organ & Rotary Speaker Cabinet Regression Suite
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

use blake3::Hasher;
use std::fs;
use std::path::Path;
use summoner_core::allocator::AllocGuard;
use summoner_core::node::ProcessContext;
use summoner_core::param_bus::{ParamBus, ParamId};
use summoner_core::rotary_bus::{RotaryArticulation, RotaryBus};
use summoner_core::transport::Transport;
use summoner_dsp::rotary_speaker::{RotaryCabinetModel, RotarySpeaker, RotarySpeed};
use summoner_dsp::tonewheel_organ::{TonewheelOrgan, TonewheelOrganProfile};
use summoner_dsp::traits::SignalProcessor;
#[cfg(feature = "gui")]
use summoner_gui::views::rotary_speaker_view::RotarySpeakerView;
#[cfg(feature = "gui")]
use summoner_gui::views::tonewheel_organ_view::TonewheelOrganView;
use summoner_sequencer::rotary_gesture::{
    RotaryGestureEngine, RotaryGesturePattern, RotaryInterpolationCurve, RotaryWaypoint,
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
            "Golden tonewheel organ & rotary cabinet hash mismatch for {}",
            hash_filename
        );
    } else {
        fs::write(&hash_file, digest).expect("Failed to write golden hash");
    }
}

#[test]
fn test_golden_tonewheel_organ_deterministic() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    // 1. Classic Gospel Organ (C4 triad: C4=60, E4=64, G4=67)
    let mut gospel = TonewheelOrgan::new(sample_rate);
    gospel.set_profile(TonewheelOrganProfile::ClassicGospel);
    gospel.note_on(60, 0.90);
    gospel.note_on(64, 0.85);
    gospel.note_on(67, 0.85);

    // 2. Jazz Trio 888 (F3 minor 7: F3=53, Ab3=56, C4=60, Eb4=63)
    let mut jazz = TonewheelOrgan::new(sample_rate);
    jazz.set_profile(TonewheelOrganProfile::JazzTrio888);
    jazz.note_on(53, 0.80);
    jazz.note_on(56, 0.75);
    jazz.note_on(60, 0.75);
    jazz.note_on(63, 0.75);

    // 3. Full Rock Power (A2 power chord: A2=45, E3=52, A3=57)
    let mut rock = TonewheelOrgan::new(sample_rate);
    rock.set_profile(TonewheelOrganProfile::FullRockPower);
    rock.note_on(45, 0.95);
    rock.note_on(52, 0.90);
    rock.note_on(57, 0.90);

    // 4. Mellow Theater Flutes (G4, D5)
    let mut theater = TonewheelOrgan::new(sample_rate);
    theater.set_profile(TonewheelOrganProfile::MellowTheater);
    theater.note_on(67, 0.70);
    theater.note_on(74, 0.70);

    // 5. Funky Groove (D3, F3, A3)
    let mut funky = TonewheelOrgan::new(sample_rate);
    funky.set_profile(TonewheelOrganProfile::FunkyGroove);
    funky.note_on(50, 0.85);
    funky.note_on(53, 0.80);
    funky.note_on(57, 0.80);

    // 6. Ambient Swell (Bb2, F3, D4)
    let mut ambient = TonewheelOrgan::new(sample_rate);
    ambient.set_profile(TonewheelOrganProfile::AmbientSwell);
    ambient.note_on(46, 0.75);
    ambient.note_on(53, 0.75);
    ambient.note_on(62, 0.75);

    let mut out_gospel_l = [0.0f32; BLOCK_SIZE];
    let mut out_gospel_r = [0.0f32; BLOCK_SIZE];
    let mut out_jazz_l = [0.0f32; BLOCK_SIZE];
    let mut out_jazz_r = [0.0f32; BLOCK_SIZE];
    let mut out_rock_l = [0.0f32; BLOCK_SIZE];
    let mut out_rock_r = [0.0f32; BLOCK_SIZE];
    let mut out_theat_l = [0.0f32; BLOCK_SIZE];
    let mut out_theat_r = [0.0f32; BLOCK_SIZE];
    let mut out_funk_l = [0.0f32; BLOCK_SIZE];
    let mut out_funk_r = [0.0f32; BLOCK_SIZE];
    let mut out_amb_l = [0.0f32; BLOCK_SIZE];
    let mut out_amb_r = [0.0f32; BLOCK_SIZE];

    let mut hasher = Hasher::new();

    // Process 64 blocks of audio (4096 samples)
    for block_idx in 0..64 {
        let ctx = ProcessContext::from_transport(&transport);

        {
            let _guard = AllocGuard::new();
            gospel.process_block(&[], &mut [&mut out_gospel_l[..], &mut out_gospel_r[..]], &ctx);
            jazz.process_block(&[], &mut [&mut out_jazz_l[..], &mut out_jazz_r[..]], &ctx);
            rock.process_block(&[], &mut [&mut out_rock_l[..], &mut out_rock_r[..]], &ctx);
            theater.process_block(&[], &mut [&mut out_theat_l[..], &mut out_theat_r[..]], &ctx);
            funky.process_block(&[], &mut [&mut out_funk_l[..], &mut out_funk_r[..]], &ctx);
            ambient.process_block(&[], &mut [&mut out_amb_l[..], &mut out_amb_r[..]], &ctx);
        }

        if block_idx == 32 {
            // Note changes halfway
            gospel.note_off(64);
            gospel.note_on(65, 0.85); // Resolve to F4
        }

        for i in 0..BLOCK_SIZE {
            assert!(out_gospel_l[i].is_finite() && out_gospel_r[i].is_finite());
            assert!(out_jazz_l[i].is_finite() && out_jazz_r[i].is_finite());
            assert!(out_rock_l[i].is_finite() && out_rock_r[i].is_finite());
            assert!(out_theat_l[i].is_finite() && out_theat_r[i].is_finite());
            assert!(out_funk_l[i].is_finite() && out_funk_r[i].is_finite());
            assert!(out_amb_l[i].is_finite() && out_amb_r[i].is_finite());

            assert!((-1.0..=1.0).contains(&out_gospel_l[i]) && (-1.0..=1.0).contains(&out_gospel_r[i]));
            assert!((-1.0..=1.0).contains(&out_jazz_l[i]) && (-1.0..=1.0).contains(&out_jazz_r[i]));
            assert!((-1.0..=1.0).contains(&out_rock_l[i]) && (-1.0..=1.0).contains(&out_rock_r[i]));
            assert!((-1.0..=1.0).contains(&out_theat_l[i]) && (-1.0..=1.0).contains(&out_theat_r[i]));
            assert!((-1.0..=1.0).contains(&out_funk_l[i]) && (-1.0..=1.0).contains(&out_funk_r[i]));
            assert!((-1.0..=1.0).contains(&out_amb_l[i]) && (-1.0..=1.0).contains(&out_amb_r[i]));

            hasher.update(&out_gospel_l[i].to_le_bytes());
            hasher.update(&out_gospel_r[i].to_le_bytes());
            hasher.update(&out_jazz_l[i].to_le_bytes());
            hasher.update(&out_jazz_r[i].to_le_bytes());
            hasher.update(&out_rock_l[i].to_le_bytes());
            hasher.update(&out_rock_r[i].to_le_bytes());
            hasher.update(&out_theat_l[i].to_le_bytes());
            hasher.update(&out_theat_r[i].to_le_bytes());
            hasher.update(&out_funk_l[i].to_le_bytes());
            hasher.update(&out_funk_r[i].to_le_bytes());
            hasher.update(&out_amb_l[i].to_le_bytes());
            hasher.update(&out_amb_r[i].to_le_bytes());
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save("golden_tonewheel_organ.hash", &digest);
}

#[test]
fn test_golden_rotary_speaker_cabinet_deterministic() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    // 1. Leslie 122 Vintage Tube (Tremolo fast)
    let mut leslie122 = RotarySpeaker::new(sample_rate);
    leslie122.set_cabinet_model(RotaryCabinetModel::Leslie122VintageTube);
    leslie122.set_speed(RotarySpeed::Tremolo);

    // 2. Leslie 147 Open-Back (Chorale slow)
    let mut leslie147 = RotarySpeaker::new(sample_rate);
    leslie147.set_cabinet_model(RotaryCabinetModel::Leslie147OpenBack);
    leslie147.set_speed(RotarySpeed::Chorale);

    // 3. Leslie 760 Solid-State (Brake deceleration)
    let mut leslie760 = RotarySpeaker::new(sample_rate);
    leslie760.set_cabinet_model(RotaryCabinetModel::Leslie760SolidState);
    leslie760.set_speed(RotarySpeed::Brake);

    // 4. Custom Twin-Horn Spatial (Tremolo fast)
    let mut twin_horn = RotarySpeaker::new(sample_rate);
    twin_horn.set_cabinet_model(RotaryCabinetModel::CustomTwinHornSpatial);
    twin_horn.set_speed(RotarySpeed::Tremolo);

    let mut in_organ = [0.0f32; BLOCK_SIZE];
    // Generate organ-like multi-harmonic test signal
    for (i, in_slot) in in_organ.iter_mut().enumerate() {
        let t = i as f32 / sample_rate as f32;
        *in_slot = (std::f32::consts::TAU * 220.0 * t).sin() * 0.4
            + (std::f32::consts::TAU * 440.0 * t).sin() * 0.3
            + (std::f32::consts::TAU * 880.0 * t).sin() * 0.2;
    }

    let mut out_122_l = [0.0f32; BLOCK_SIZE];
    let mut out_122_r = [0.0f32; BLOCK_SIZE];
    let mut out_147_l = [0.0f32; BLOCK_SIZE];
    let mut out_147_r = [0.0f32; BLOCK_SIZE];
    let mut out_760_l = [0.0f32; BLOCK_SIZE];
    let mut out_760_r = [0.0f32; BLOCK_SIZE];
    let mut out_twin_l = [0.0f32; BLOCK_SIZE];
    let mut out_twin_r = [0.0f32; BLOCK_SIZE];

    let mut hasher = Hasher::new();

    // Process 64 blocks of audio (4096 samples)
    for block_idx in 0..64 {
        let ctx = ProcessContext::from_transport(&transport);

        if block_idx == 30 {
            leslie122.set_speed(RotarySpeed::Chorale);
            leslie147.set_speed(RotarySpeed::Tremolo);
        }

        {
            let _guard = AllocGuard::new();
            leslie122.process_block(&[&in_organ[..]], &mut [&mut out_122_l[..], &mut out_122_r[..]], &ctx);
            leslie147.process_block(&[&in_organ[..]], &mut [&mut out_147_l[..], &mut out_147_r[..]], &ctx);
            leslie760.process_block(&[&in_organ[..]], &mut [&mut out_760_l[..], &mut out_760_r[..]], &ctx);
            twin_horn.process_block(&[&in_organ[..]], &mut [&mut out_twin_l[..], &mut out_twin_r[..]], &ctx);
        }

        for i in 0..BLOCK_SIZE {
            assert!(out_122_l[i].is_finite() && out_122_r[i].is_finite());
            assert!(out_147_l[i].is_finite() && out_147_r[i].is_finite());
            assert!(out_760_l[i].is_finite() && out_760_r[i].is_finite());
            assert!(out_twin_l[i].is_finite() && out_twin_r[i].is_finite());

            assert!((-1.0..=1.0).contains(&out_122_l[i]) && (-1.0..=1.0).contains(&out_122_r[i]));
            assert!((-1.0..=1.0).contains(&out_147_l[i]) && (-1.0..=1.0).contains(&out_147_r[i]));
            assert!((-1.0..=1.0).contains(&out_760_l[i]) && (-1.0..=1.0).contains(&out_760_r[i]));
            assert!((-1.0..=1.0).contains(&out_twin_l[i]) && (-1.0..=1.0).contains(&out_twin_r[i]));

            hasher.update(&out_122_l[i].to_le_bytes());
            hasher.update(&out_122_r[i].to_le_bytes());
            hasher.update(&out_147_l[i].to_le_bytes());
            hasher.update(&out_147_r[i].to_le_bytes());
            hasher.update(&out_760_l[i].to_le_bytes());
            hasher.update(&out_760_r[i].to_le_bytes());
            hasher.update(&out_twin_l[i].to_le_bytes());
            hasher.update(&out_twin_r[i].to_le_bytes());
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save("golden_rotary_speaker.hash", &digest);
}

#[test]
fn test_golden_rotary_gesture_and_rotary_bus_deterministic() {
    let mut engine = RotaryGestureEngine::new(RotaryGesturePattern::CustomSpline);
    engine.default_curve = RotaryInterpolationCurve::CubicCatmullRom;
    engine.loop_length_beats = 8.0;
    engine.loop_enabled = true;

    engine.waypoints = vec![
        RotaryWaypoint {
            beat: 0.0,
            speed_state: 1, // Chorale
            drawbars: [8.0, 8.0, 8.0, 0.0, 0.0, 0.0, 0.0, 0.0, 8.0],
            tube_drive_db: 4.0,
            horn_drum_balance_pct: 60.0,
            percussion_enabled: true,
            percussion_harmonic: 3,
            percussion_fast: true,
            vibrato_mode: 6,
            articulation: RotaryArticulation::ChoraleSlow,
            curve: RotaryInterpolationCurve::CubicCatmullRom,
        },
        RotaryWaypoint {
            beat: 2.0,
            speed_state: 2, // Tremolo
            drawbars: [8.0, 8.0, 8.0, 4.0, 4.0, 4.0, 0.0, 4.0, 8.0],
            tube_drive_db: 8.0,
            horn_drum_balance_pct: 65.0,
            percussion_enabled: true,
            percussion_harmonic: 3,
            percussion_fast: true,
            vibrato_mode: 6,
            articulation: RotaryArticulation::TremoloFast,
            curve: RotaryInterpolationCurve::CubicCatmullRom,
        },
        RotaryWaypoint {
            beat: 4.0,
            speed_state: 2,
            drawbars: [8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0],
            tube_drive_db: 14.0,
            horn_drum_balance_pct: 70.0,
            percussion_enabled: false,
            percussion_harmonic: 3,
            percussion_fast: true,
            vibrato_mode: 6,
            articulation: RotaryArticulation::RockOverdrive,
            curve: RotaryInterpolationCurve::CubicCatmullRom,
        },
        RotaryWaypoint {
            beat: 6.0,
            speed_state: 3, // Brake
            drawbars: [8.0, 8.0, 8.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            tube_drive_db: 3.0,
            horn_drum_balance_pct: 50.0,
            percussion_enabled: true,
            percussion_harmonic: 2,
            percussion_fast: false,
            vibrato_mode: 0,
            articulation: RotaryArticulation::BrakeStop,
            curve: RotaryInterpolationCurve::CubicCatmullRom,
        },
    ];

    let bus = RotaryBus::new();
    let mut param_bus = ParamBus::new();
    for i in 0..20 {
        param_bus.register(ParamId(900 + i), 0.0);
    }

    let mut hasher = Hasher::new();

    for step in 0..160 {
        let beat = step as f64 * 0.05; // 8 beats total

        let snap = {
            let _guard = AllocGuard::new();
            engine.dispatch_to_bus(beat, &bus);
            engine.dispatch_to_param_bus(beat, ParamId(900), &param_bus);
            bus.snapshot()
        };

        assert!(snap.horn_rpm.is_finite());
        assert!(snap.drum_rpm.is_finite());
        assert!(snap.tube_drive_db.is_finite());
        assert!(snap.horn_drum_balance_pct.is_finite());
        for d in 0..9 {
            assert!(snap.drawbars[d].is_finite());
            hasher.update(&snap.drawbars[d].to_le_bytes());
        }

        hasher.update(&(snap.articulation as u32).to_le_bytes());
        hasher.update(&snap.speed_state.to_le_bytes());
        hasher.update(&snap.horn_rpm.to_le_bytes());
        hasher.update(&snap.drum_rpm.to_le_bytes());
        hasher.update(&snap.tube_drive_db.to_le_bytes());
        hasher.update(&snap.horn_drum_balance_pct.to_le_bytes());
        hasher.update(&snap.gesture_progress.to_le_bytes());
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save("golden_rotary_gesture.hash", &digest);
}

#[cfg(feature = "gui")]
#[test]
fn test_golden_rotary_tonewheel_views_ascii_and_snapshots() {
    let organ_view = TonewheelOrganView::new();
    let speaker_view = RotarySpeakerView::new();

    let organ_ascii = organ_view.render_ascii(64, 16);
    let speaker_ascii = speaker_view.render_ascii(64, 16);

    let mut hasher = Hasher::new();
    for line in &organ_ascii {
        hasher.update(line.as_bytes());
    }
    for line in &speaker_ascii {
        hasher.update(line.as_bytes());
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save("golden_rotary_tonewheel_views.hash", &digest);

    // Save PNG snapshots across the project render directories
    let _ = organ_view.render_snapshot_png("scratch/renders/tonewheel_organ_view.png", 800, 520);
    let _ = organ_view.render_snapshot_png("crates/summon/scratch/renders/tonewheel_organ_view.png", 800, 520);
    let _ = organ_view.render_snapshot_png("crates/summoner_gui/scratch/renders/tonewheel_organ_view.png", 800, 520);

    let _ = speaker_view.render_snapshot_png("scratch/renders/rotary_speaker_view.png", 800, 520);
    let _ = speaker_view.render_snapshot_png("crates/summon/scratch/renders/rotary_speaker_view.png", 800, 520);
    let _ = speaker_view.render_snapshot_png("crates/summoner_gui/scratch/renders/rotary_speaker_view.png", 800, 520);
}

fn check_or_save(filename: &str, digest: &str) {
    check_or_save_golden(filename, digest);
}
