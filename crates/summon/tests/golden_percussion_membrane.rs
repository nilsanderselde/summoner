// Summoner - Deterministic Golden Percussive Membrane & Idiophone Waveguide Regression Suite
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

use blake3::Hasher;
use std::fs;
use std::path::Path;
use summoner_core::allocator::AllocGuard;
use summoner_core::mallet_bus::{MalletArticulation, MalletBus};
use summoner_core::node::ProcessContext;
use summoner_core::param_bus::{ParamBus, ParamId};
use summoner_core::transport::Transport;
use summoner_dsp::idiophone_resonator::{IdiophoneInstrumentProfile, StruckIdiophoneResonator};
use summoner_dsp::percussion_membrane::{MembraneInstrumentProfile, PercussionMembrane};
use summoner_dsp::traits::SignalProcessor;
#[cfg(feature = "gui")]
use summoner_gui::views::idiophone_spectrum_view::IdiophoneSpectrumView;
#[cfg(feature = "gui")]
use summoner_gui::views::membrane_cavity_view::MembraneCavityView;
use summoner_sequencer::mallet_gesture::{
    MalletGestureEngine, MalletGesturePattern, MalletInterpolationCurve, MalletWaypoint,
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
            "Golden percussion membrane & idiophone waveguide hash mismatch for {}",
            hash_filename
        );
    } else {
        fs::write(&hash_file, digest).expect("Failed to write golden hash");
    }
}

#[test]
fn test_golden_percussion_membrane_bessel_deterministic() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    // 1. Timpani Kettle D3
    let mut timpani = PercussionMembrane::new(sample_rate);
    timpani.set_instrument(MembraneInstrumentProfile::TimpaniKettle);
    timpani.set_frequency(146.83);
    timpani.set_radial_position(0.70);
    timpani.trigger_strike(0.90, 0.50, 0.70);

    // 2. Concert Bass Drum Low G1
    let mut bass_drum = PercussionMembrane::new(sample_rate);
    bass_drum.set_instrument(MembraneInstrumentProfile::ConcertBassDrum);
    bass_drum.set_frequency(48.00);
    bass_drum.set_radial_position(0.50);
    bass_drum.trigger_strike(0.95, 0.30, 0.50);

    // 3. Orchestral Snare Drum F#3
    let mut snare = PercussionMembrane::new(sample_rate);
    snare.set_instrument(MembraneInstrumentProfile::SnareDrum);
    snare.set_frequency(185.00);
    snare.set_radial_position(0.65);
    snare.trigger_strike(0.85, 0.75, 0.65);

    // 4. Acoustic Tom-Tom A2
    let mut tomtom = PercussionMembrane::new(sample_rate);
    tomtom.set_instrument(MembraneInstrumentProfile::TomTom);
    tomtom.set_frequency(110.00);
    tomtom.set_radial_position(0.60);
    tomtom.trigger_strike(0.80, 0.60, 0.60);

    // 5. Latin Bongos/Congas C4
    let mut bongos = PercussionMembrane::new(sample_rate);
    bongos.set_instrument(MembraneInstrumentProfile::BongosCongas);
    bongos.set_frequency(260.00);
    bongos.set_radial_position(0.85);
    bongos.trigger_strike(0.90, 0.85, 0.85);

    // 6. African Djembe F2
    let mut djembe = PercussionMembrane::new(sample_rate);
    djembe.set_instrument(MembraneInstrumentProfile::DjembeFramedrum);
    djembe.set_frequency(85.00);
    djembe.set_radial_position(0.35);
    djembe.trigger_strike(0.90, 0.40, 0.35);

    let in_dummy = [0.0f32; BLOCK_SIZE];
    let mut out_timpani = [0.0f32; BLOCK_SIZE];
    let mut out_bass = [0.0f32; BLOCK_SIZE];
    let mut out_snare = [0.0f32; BLOCK_SIZE];
    let mut out_tomtom = [0.0f32; BLOCK_SIZE];
    let mut out_bongos = [0.0f32; BLOCK_SIZE];
    let mut out_djembe = [0.0f32; BLOCK_SIZE];

    let mut hasher = Hasher::new();

    // Process 64 blocks of audio (4096 samples)
    for _block_idx in 0..64 {
        let ctx = ProcessContext::from_transport(&transport);

        {
            let _guard = AllocGuard::new();
            timpani.process_block(&[&in_dummy[..]], &mut [&mut out_timpani[..]], &ctx);
            bass_drum.process_block(&[&in_dummy[..]], &mut [&mut out_bass[..]], &ctx);
            snare.process_block(&[&in_dummy[..]], &mut [&mut out_snare[..]], &ctx);
            tomtom.process_block(&[&in_dummy[..]], &mut [&mut out_tomtom[..]], &ctx);
            bongos.process_block(&[&in_dummy[..]], &mut [&mut out_bongos[..]], &ctx);
            djembe.process_block(&[&in_dummy[..]], &mut [&mut out_djembe[..]], &ctx);
        }

        for i in 0..BLOCK_SIZE {
            assert!(out_timpani[i].is_finite());
            assert!(out_bass[i].is_finite());
            assert!(out_snare[i].is_finite());
            assert!(out_tomtom[i].is_finite());
            assert!(out_bongos[i].is_finite());
            assert!(out_djembe[i].is_finite());

            assert!((-1.0..=1.0).contains(&out_timpani[i]));
            assert!((-1.0..=1.0).contains(&out_bass[i]));
            assert!((-1.0..=1.0).contains(&out_snare[i]));
            assert!((-1.0..=1.0).contains(&out_tomtom[i]));
            assert!((-1.0..=1.0).contains(&out_bongos[i]));
            assert!((-1.0..=1.0).contains(&out_djembe[i]));

            hasher.update(&out_timpani[i].to_le_bytes());
            hasher.update(&out_bass[i].to_le_bytes());
            hasher.update(&out_snare[i].to_le_bytes());
            hasher.update(&out_tomtom[i].to_le_bytes());
            hasher.update(&out_bongos[i].to_le_bytes());
            hasher.update(&out_djembe[i].to_le_bytes());
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_percussion_membrane.hash", &digest);
}

#[test]
fn test_golden_struck_idiophone_modal_bank_deterministic() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    // 1. Concert Rosewood Marimba C4
    let mut marimba = StruckIdiophoneResonator::new(sample_rate);
    marimba.set_instrument(IdiophoneInstrumentProfile::MarimbaWood);
    marimba.set_frequency(261.63);
    marimba.trigger_strike(0.85, 0.35, 0.20);

    // 2. Orchestral Xylophone C5
    let mut xylophone = StruckIdiophoneResonator::new(sample_rate);
    xylophone.set_instrument(IdiophoneInstrumentProfile::XylophoneRosewood);
    xylophone.set_frequency(523.25);
    xylophone.trigger_strike(0.90, 0.85, 0.25);

    // 3. Concert Vibraphone A4 with Motorized Tremolo
    let mut vibraphone = StruckIdiophoneResonator::new(sample_rate);
    vibraphone.set_instrument(IdiophoneInstrumentProfile::VibraphoneAluminum);
    vibraphone.set_frequency(440.00);
    vibraphone.trigger_strike(0.80, 0.50, 0.30);

    // 4. Trinidad Tenor Steelpan D4
    let mut steelpan = StruckIdiophoneResonator::new(sample_rate);
    steelpan.set_instrument(IdiophoneInstrumentProfile::SteelpanTrinidad);
    steelpan.set_frequency(293.66);
    steelpan.trigger_strike(0.75, 0.55, 0.15);

    // 5. African Kalimba E4
    let mut kalimba = StruckIdiophoneResonator::new(sample_rate);
    kalimba.set_instrument(IdiophoneInstrumentProfile::KalimbaMbira);
    kalimba.set_frequency(329.63);
    kalimba.trigger_strike(0.70, 0.30, 0.10);

    // 6. Orchestral Glockenspiel C6
    let mut glockenspiel = StruckIdiophoneResonator::new(sample_rate);
    glockenspiel.set_instrument(IdiophoneInstrumentProfile::GlockenspielBell);
    glockenspiel.set_frequency(1046.50);
    glockenspiel.trigger_strike(0.85, 0.95, 0.40);

    let in_dummy = [0.0f32; BLOCK_SIZE];
    let mut out_marimba = [0.0f32; BLOCK_SIZE];
    let mut out_xylo = [0.0f32; BLOCK_SIZE];
    let mut out_vibes = [0.0f32; BLOCK_SIZE];
    let mut out_steelpan = [0.0f32; BLOCK_SIZE];
    let mut out_kalimba = [0.0f32; BLOCK_SIZE];
    let mut out_glock = [0.0f32; BLOCK_SIZE];

    let mut hasher = Hasher::new();

    // Process 64 blocks of audio (4096 samples)
    for _block_idx in 0..64 {
        let ctx = ProcessContext::from_transport(&transport);

        {
            let _guard = AllocGuard::new();
            marimba.process_block(&[&in_dummy[..]], &mut [&mut out_marimba[..]], &ctx);
            xylophone.process_block(&[&in_dummy[..]], &mut [&mut out_xylo[..]], &ctx);
            vibraphone.process_block(&[&in_dummy[..]], &mut [&mut out_vibes[..]], &ctx);
            steelpan.process_block(&[&in_dummy[..]], &mut [&mut out_steelpan[..]], &ctx);
            kalimba.process_block(&[&in_dummy[..]], &mut [&mut out_kalimba[..]], &ctx);
            glockenspiel.process_block(&[&in_dummy[..]], &mut [&mut out_glock[..]], &ctx);
        }

        for i in 0..BLOCK_SIZE {
            assert!(out_marimba[i].is_finite());
            assert!(out_xylo[i].is_finite());
            assert!(out_vibes[i].is_finite());
            assert!(out_steelpan[i].is_finite());
            assert!(out_kalimba[i].is_finite());
            assert!(out_glock[i].is_finite());

            assert!((-1.0..=1.0).contains(&out_marimba[i]));
            assert!((-1.0..=1.0).contains(&out_xylo[i]));
            assert!((-1.0..=1.0).contains(&out_vibes[i]));
            assert!((-1.0..=1.0).contains(&out_steelpan[i]));
            assert!((-1.0..=1.0).contains(&out_kalimba[i]));
            assert!((-1.0..=1.0).contains(&out_glock[i]));

            hasher.update(&out_marimba[i].to_le_bytes());
            hasher.update(&out_xylo[i].to_le_bytes());
            hasher.update(&out_vibes[i].to_le_bytes());
            hasher.update(&out_steelpan[i].to_le_bytes());
            hasher.update(&out_kalimba[i].to_le_bytes());
            hasher.update(&out_glock[i].to_le_bytes());
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_idiophone_resonator.hash", &digest);
}

#[test]
fn test_golden_mallet_gesture_and_mallet_bus_deterministic() {
    let mut engine = MalletGestureEngine::new(MalletGesturePattern::CustomSpline);
    engine.default_curve = MalletInterpolationCurve::CubicCatmullRom;
    engine.loop_length_beats = 8.0;
    engine.loop_enabled = true;

    engine.waypoints = vec![
        MalletWaypoint {
            beat: 0.0,
            radial_strike_pos: 0.70,
            strike_velocity: 0.75,
            mallet_hardness: 0.40,
            rimshot_damping: 0.05,
            roll_tremolo_rate_hz: 0.0,
            kettle_tension_cents: 0.0,
            air_cavity_depth: 0.85,
            membrane_tension: 0.50,
            articulation: MalletArticulation::EdgeSweetSpot,
            curve: MalletInterpolationCurve::CubicCatmullRom,
        },
        MalletWaypoint {
            beat: 2.0,
            radial_strike_pos: 0.15,
            strike_velocity: 0.85,
            mallet_hardness: 0.50,
            rimshot_damping: 0.02,
            roll_tremolo_rate_hz: 0.0,
            kettle_tension_cents: 200.0,
            air_cavity_depth: 0.85,
            membrane_tension: 0.55,
            articulation: MalletArticulation::CenterStrike,
            curve: MalletInterpolationCurve::CubicCatmullRom,
        },
        MalletWaypoint {
            beat: 4.0,
            radial_strike_pos: 0.95,
            strike_velocity: 0.95,
            mallet_hardness: 0.85,
            rimshot_damping: 0.80,
            roll_tremolo_rate_hz: 0.0,
            kettle_tension_cents: 400.0,
            air_cavity_depth: 0.60,
            membrane_tension: 0.65,
            articulation: MalletArticulation::Rimshot,
            curve: MalletInterpolationCurve::CubicCatmullRom,
        },
        MalletWaypoint {
            beat: 6.0,
            radial_strike_pos: 0.65,
            strike_velocity: 0.60,
            mallet_hardness: 0.45,
            rimshot_damping: 0.05,
            roll_tremolo_rate_hz: 14.0,
            kettle_tension_cents: 600.0,
            air_cavity_depth: 0.85,
            membrane_tension: 0.70,
            articulation: MalletArticulation::MalletRollTremolo,
            curve: MalletInterpolationCurve::CubicCatmullRom,
        },
    ];

    let bus = MalletBus::new();
    let mut param_bus = ParamBus::new();
    for i in 0..12 {
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

        assert!(snap.radial_strike_pos.is_finite());
        assert!(snap.strike_velocity.is_finite());
        assert!(snap.mallet_hardness.is_finite());
        assert!(snap.rimshot_damping.is_finite());
        assert!(snap.roll_tremolo_rate_hz.is_finite());
        assert!(snap.kettle_tension_cents.is_finite());
        assert!(snap.air_cavity_depth.is_finite());
        assert!(snap.membrane_tension.is_finite());

        hasher.update(&(snap.articulation as u32).to_le_bytes());
        hasher.update(&snap.radial_strike_pos.to_le_bytes());
        hasher.update(&snap.strike_velocity.to_le_bytes());
        hasher.update(&snap.mallet_hardness.to_le_bytes());
        hasher.update(&snap.rimshot_damping.to_le_bytes());
        hasher.update(&snap.roll_tremolo_rate_hz.to_le_bytes());
        hasher.update(&snap.kettle_tension_cents.to_le_bytes());
        hasher.update(&snap.air_cavity_depth.to_le_bytes());
        hasher.update(&snap.membrane_tension.to_le_bytes());
        hasher.update(&snap.gesture_progress.to_le_bytes());
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_mallet_gesture.hash", &digest);
}

#[cfg(feature = "gui")]
#[test]
fn test_golden_membrane_idiophone_views_ascii_renders() {
    let membrane_view = MembraneCavityView::new();
    let idiophone_view = IdiophoneSpectrumView::new();

    let membrane_ascii = membrane_view.render_ascii(64, 16);
    let idiophone_ascii = idiophone_view.render_ascii(64, 16);

    let mut hasher = Hasher::new();
    for line in &membrane_ascii {
        hasher.update(line.as_bytes());
    }
    for line in &idiophone_ascii {
        hasher.update(line.as_bytes());
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_membrane_idiophone_views.hash", &digest);
}
