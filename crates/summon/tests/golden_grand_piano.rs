// Summoner - Deterministic Golden Concert Grand Piano Physical Modeling Regression Suite
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

use blake3::Hasher;
use std::fs;
use std::path::Path;
use summoner_core::allocator::AllocGuard;
use summoner_core::grand_piano_bus::{
    register_grand_piano_params, GrandPianoBus, GRAND_PIANO_BASE_PARAM_ID,
};
use summoner_core::node::ProcessContext;
use summoner_core::param_bus::ParamBus;
use summoner_core::transport::Transport;
use summoner_dsp::grand_piano::{ConcertGrandPiano, GrandPianoProfile};
use summoner_dsp::soundboard_bridge::{
    BridgeWaveCoupler, GrandPianoFeltHammer, SoundboardProfile, SpruceSoundboard,
};
use summoner_dsp::traits::SignalProcessor;
#[cfg(feature = "gui")]
use summoner_gui::views::grand_piano_view::GrandPianoView;
#[cfg(feature = "gui")]
use summoner_gui::views::soundboard_bridge_view::SoundboardBridgeView;
use summoner_sequencer::grand_piano_gesture::{
    GrandPianoGestureEngine, GrandPianoGesturePattern,
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
            "Golden grand piano hash mismatch for {}",
            hash_filename
        );
    } else {
        fs::write(&hash_file, digest).expect("Failed to write golden hash");
    }
}

#[test]
fn test_golden_grand_piano_deterministic() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    // 1. Steinway D-274 Concert Grand (C major triad + bass A0: A0=21, C4=60, E4=64, G4=67)
    let mut steinway = ConcertGrandPiano::new(sample_rate);
    steinway.set_profile(GrandPianoProfile::SteinwayD9Foot);
    steinway.note_on(21, 0.90);
    steinway.note_on(60, 0.85);
    steinway.note_on(64, 0.80);
    steinway.note_on(67, 0.85);

    // 2. Bösendorfer Imperial 290 with deep resonance & sustain (F minor: F2=41, C3=48, Ab3=56, C4=60)
    let mut bosendorfer = ConcertGrandPiano::new(sample_rate);
    bosendorfer.set_profile(GrandPianoProfile::BösendorferImperial);
    bosendorfer.set_sustain_pedal(0.85);
    bosendorfer.note_on(41, 0.92);
    bosendorfer.note_on(48, 0.88);
    bosendorfer.note_on(56, 0.85);
    bosendorfer.note_on(60, 0.80);

    // 3. Yamaha CFX Concert Grand (Bright treble passage: C5=72, E5=76, G5=79, C6=84)
    let mut yamaha = ConcertGrandPiano::new(sample_rate);
    yamaha.set_profile(GrandPianoProfile::YamahaCFX);
    yamaha.note_on(72, 0.90);
    yamaha.note_on(76, 0.88);
    yamaha.note_on(79, 0.85);
    yamaha.note_on(84, 0.92);

    // 4. Impressionist Una Corda Grand (Soft pedal shimmer: D4=62, F#4=66, A4=69, D5=74)
    let mut una_corda = ConcertGrandPiano::new(sample_rate);
    una_corda.set_profile(GrandPianoProfile::ImpressionistUnaCorda);
    una_corda.set_una_corda_pedal(1.0);
    una_corda.set_sustain_pedal(0.75);
    una_corda.note_on(62, 0.65);
    una_corda.note_on(66, 0.60);
    una_corda.note_on(69, 0.60);
    una_corda.note_on(74, 0.55);

    // 5. Intimate Studio Grand (Mellow felt: G3=55, B3=59, D4=62, G4=67)
    let mut intimate = ConcertGrandPiano::new(sample_rate);
    intimate.set_profile(GrandPianoProfile::IntimateStudio);
    intimate.set_sustain_pedal(0.40);
    intimate.note_on(55, 0.70);
    intimate.note_on(59, 0.68);
    intimate.note_on(62, 0.65);
    intimate.note_on(67, 0.60);

    // 6. Prepared Avant-Garde Grand (Percussive boundary clatter: Eb3=51, A3=57, D4=62, Ab4=68)
    let mut prepared = ConcertGrandPiano::new(sample_rate);
    prepared.set_profile(GrandPianoProfile::PreparedAvantGarde);
    prepared.note_on(51, 0.95);
    prepared.note_on(57, 0.90);
    prepared.note_on(62, 0.90);
    prepared.note_on(68, 0.85);

    let mut out_stein_l = [0.0f32; BLOCK_SIZE];
    let mut out_stein_r = [0.0f32; BLOCK_SIZE];
    let mut out_bose_l = [0.0f32; BLOCK_SIZE];
    let mut out_bose_r = [0.0f32; BLOCK_SIZE];
    let mut out_yama_l = [0.0f32; BLOCK_SIZE];
    let mut out_yama_r = [0.0f32; BLOCK_SIZE];
    let mut out_unac_l = [0.0f32; BLOCK_SIZE];
    let mut out_unac_r = [0.0f32; BLOCK_SIZE];
    let mut out_inti_l = [0.0f32; BLOCK_SIZE];
    let mut out_inti_r = [0.0f32; BLOCK_SIZE];
    let mut out_prep_l = [0.0f32; BLOCK_SIZE];
    let mut out_prep_r = [0.0f32; BLOCK_SIZE];

    let mut hasher = Hasher::new();

    // Process 64 blocks of audio (4096 samples)
    for block_idx in 0..64 {
        let ctx = ProcessContext::from_transport(&transport);

        {
            let _guard = AllocGuard::new();
            steinway.process_block(&[], &mut [&mut out_stein_l[..], &mut out_stein_r[..]], &ctx);
            bosendorfer.process_block(&[], &mut [&mut out_bose_l[..], &mut out_bose_r[..]], &ctx);
            yamaha.process_block(&[], &mut [&mut out_yama_l[..], &mut out_yama_r[..]], &ctx);
            una_corda.process_block(&[], &mut [&mut out_unac_l[..], &mut out_unac_r[..]], &ctx);
            intimate.process_block(&[], &mut [&mut out_inti_l[..], &mut out_inti_r[..]], &ctx);
            prepared.process_block(&[], &mut [&mut out_prep_l[..], &mut out_prep_r[..]], &ctx);
        }

        if block_idx == 32 {
            // Note off and note transition halfway, test Sostenuto latch
            steinway.note_off(60);
            steinway.note_on(62, 0.88); // D4
            bosendorfer.set_sostenuto_pedal(true);
            bosendorfer.note_off(41);
            yamaha.note_off(72);
            yamaha.note_on(81, 0.90); // A5
            una_corda.set_una_corda_pedal(0.5); // Half una corda
        }

        for i in 0..BLOCK_SIZE {
            assert!(out_stein_l[i].is_finite() && out_stein_r[i].is_finite());
            assert!(out_bose_l[i].is_finite() && out_bose_r[i].is_finite());
            assert!(out_yama_l[i].is_finite() && out_yama_r[i].is_finite());
            assert!(out_unac_l[i].is_finite() && out_unac_r[i].is_finite());
            assert!(out_inti_l[i].is_finite() && out_inti_r[i].is_finite());
            assert!(out_prep_l[i].is_finite() && out_prep_r[i].is_finite());

            assert!((-1.0..=1.0).contains(&out_stein_l[i]) && (-1.0..=1.0).contains(&out_stein_r[i]));
            assert!((-1.0..=1.0).contains(&out_bose_l[i]) && (-1.0..=1.0).contains(&out_bose_r[i]));
            assert!((-1.0..=1.0).contains(&out_yama_l[i]) && (-1.0..=1.0).contains(&out_yama_r[i]));
            assert!((-1.0..=1.0).contains(&out_unac_l[i]) && (-1.0..=1.0).contains(&out_unac_r[i]));
            assert!((-1.0..=1.0).contains(&out_inti_l[i]) && (-1.0..=1.0).contains(&out_inti_r[i]));
            assert!((-1.0..=1.0).contains(&out_prep_l[i]) && (-1.0..=1.0).contains(&out_prep_r[i]));

            hasher.update(&out_stein_l[i].to_le_bytes());
            hasher.update(&out_stein_r[i].to_le_bytes());
            hasher.update(&out_bose_l[i].to_le_bytes());
            hasher.update(&out_bose_r[i].to_le_bytes());
            hasher.update(&out_yama_l[i].to_le_bytes());
            hasher.update(&out_yama_r[i].to_le_bytes());
            hasher.update(&out_unac_l[i].to_le_bytes());
            hasher.update(&out_unac_r[i].to_le_bytes());
            hasher.update(&out_inti_l[i].to_le_bytes());
            hasher.update(&out_inti_r[i].to_le_bytes());
            hasher.update(&out_prep_l[i].to_le_bytes());
            hasher.update(&out_prep_r[i].to_le_bytes());
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_grand_piano.hash", &digest);
}

#[test]
fn test_golden_grand_piano_soundboard_bridge_deterministic() {
    let sample_rate = 48000;

    let mut soundboard = SpruceSoundboard::new(sample_rate, SoundboardProfile::SteinwayD9Foot);
    let mut bridge = BridgeWaveCoupler::new(420.0, 0.35);
    let mut hammer = GrandPianoFeltHammer::new(0.005, 1.2e8, 2.4, 0.35, 0.125);
    hammer.trigger_strike(4.0);

    let mut hasher = Hasher::new();

    for step in 0..2048 {
        let t = step as f32 / 48000.0;
        let synthetic_disp = (t * 261.63 * 2.0 * std::f32::consts::PI).sin() * 0.0003;

        let (f_hammer, f_trans, sb_l, sb_r) = {
            let _guard = AllocGuard::new();
            let fh = hammer.step(synthetic_disp, sample_rate);
            let in_forces = [fh, fh * 0.95, fh * 1.05];
            let mut refl = [0.0; 3];
            let ft = bridge.process_scattering_junction(&in_forces, &mut refl);
            let (l, r) = soundboard.process_bridge_force(ft * 0.001);
            (fh, ft, l, r)
        };

        assert!(f_hammer.is_finite());
        assert!(f_trans.is_finite());
        assert!(sb_l.is_finite() && sb_r.is_finite());

        hasher.update(&f_hammer.to_le_bytes());
        hasher.update(&f_trans.to_le_bytes());
        hasher.update(&sb_l.to_le_bytes());
        hasher.update(&sb_r.to_le_bytes());
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_grand_piano_soundboard.hash", &digest);
}

#[test]
fn test_golden_grand_piano_gesture_and_bus_deterministic() {
    let mut engine = GrandPianoGestureEngine::with_pattern(
        GrandPianoGesturePattern::ChopinRecitalNocturne {
            phrase_length_beats: 8.0,
            max_velocity: 0.90,
        },
    );
    engine.is_looping = true;

    let bus = GrandPianoBus::new();
    let mut param_bus = ParamBus::new();
    register_grand_piano_params(&mut param_bus, GRAND_PIANO_BASE_PARAM_ID);

    let mut hasher = Hasher::new();

    for step in 0..160 {
        let beat = step as f64 * 0.05; // 8 beats total

        let snap = {
            let _guard = AllocGuard::new();
            engine.dispatch_to_bus(beat, &bus, Some(&param_bus), GRAND_PIANO_BASE_PARAM_ID);
            bus.snapshot()
        };

        assert!(snap.strike_velocity.is_finite());
        assert!(snap.damper_lift_pos.is_finite());
        assert!(snap.una_corda_shift.is_finite());
        assert!(snap.bridge_bleed.is_finite());
        assert!(snap.unison_detune_cents.is_finite());
        assert!(snap.hammer_hardness.is_finite());
        assert!(snap.inharmonicity_b.is_finite());
        assert!(snap.soundboard_decay.is_finite());
        assert!(snap.master_gain.is_finite());

        hasher.update(&(snap.articulation as u32).to_le_bytes());
        hasher.update(&snap.strike_velocity.to_le_bytes());
        hasher.update(&snap.damper_lift_pos.to_le_bytes());
        hasher.update(&snap.una_corda_shift.to_le_bytes());
        hasher.update(&snap.sostenuto_latch_low.to_le_bytes());
        hasher.update(&snap.sostenuto_latch_mid.to_le_bytes());
        hasher.update(&snap.sostenuto_latch_high.to_le_bytes());
        hasher.update(&snap.bridge_bleed.to_le_bytes());
        hasher.update(&snap.unison_detune_cents.to_le_bytes());
        hasher.update(&snap.hammer_hardness.to_le_bytes());
        hasher.update(&snap.inharmonicity_b.to_le_bytes());
        hasher.update(&snap.soundboard_decay.to_le_bytes());
        hasher.update(&snap.master_gain.to_le_bytes());
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_grand_piano_gesture.hash", &digest);
}

#[cfg(feature = "gui")]
#[test]
fn test_golden_grand_piano_views_ascii_and_snapshots() {
    let piano_view = GrandPianoView::new();
    let soundboard_view = SoundboardBridgeView::new();

    let piano_ascii = piano_view.render_ascii(80, 16);
    let soundboard_ascii = soundboard_view.render_ascii(80, 16);

    let mut hasher = Hasher::new();
    for line in &piano_ascii {
        hasher.update(line.as_bytes());
    }
    for line in &soundboard_ascii {
        hasher.update(line.as_bytes());
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_grand_piano_views.hash", &digest);

    // Save PNG snapshots across the project render directories
    let _ = piano_view.render_snapshot_png("scratch/renders/grand_piano_view.png", 800, 520);
    let _ = piano_view.render_snapshot_png("../../scratch/renders/grand_piano_view.png", 800, 520);
    let _ = piano_view.render_snapshot_png("crates/summon/scratch/renders/grand_piano_view.png", 800, 520);
    let _ = piano_view.render_snapshot_png("crates/summoner_gui/scratch/renders/grand_piano_view.png", 800, 520);

    let _ = soundboard_view.render_snapshot_png("scratch/renders/soundboard_bridge_view.png", 800, 520);
    let _ = soundboard_view.render_snapshot_png("../../scratch/renders/soundboard_bridge_view.png", 800, 520);
    let _ = soundboard_view.render_snapshot_png("crates/summon/scratch/renders/soundboard_bridge_view.png", 800, 520);
    let _ = soundboard_view.render_snapshot_png("crates/summoner_gui/scratch/renders/soundboard_bridge_view.png", 800, 520);
}
