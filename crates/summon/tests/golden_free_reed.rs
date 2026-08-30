// Summoner - Deterministic Golden Free-Reed & Bellows Aerodynamics Regression Suite
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

use blake3::Hasher;
use std::fs;
use std::path::Path;
use summoner_core::allocator::AllocGuard;
use summoner_core::bellows_bus::{
    register_bellows_params, BellowsBus, BELLOWS_BASE_PARAM_ID,
};
use summoner_core::node::ProcessContext;
use summoner_core::param_bus::ParamBus;
use summoner_core::transport::Transport;
use summoner_dsp::cassotto_chamber::{CassottoChamber, CassottoProfile};
use summoner_dsp::free_reed::{
    FreeReed, FreeReedOscillator, FreeReedProfile, REGISTER_ACCORDION, REGISTER_BANDONEON,
    REGISTER_CLARINET, REGISTER_MASTER, REGISTER_MUSETTE,
};
use summoner_dsp::traits::SignalProcessor;
#[cfg(feature = "gui")]
use summoner_gui::views::bellows_view::BellowsView;
#[cfg(feature = "gui")]
use summoner_gui::views::free_reed_view::FreeReedView;
use summoner_sequencer::bellows_gesture::{
    BellowsGestureEngine, BellowsGesturePattern,
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
            "Golden free-reed hash mismatch for {}",
            hash_filename
        );
    } else {
        fs::write(&hash_file, digest).expect("Failed to write golden hash");
    }
}

#[test]
fn test_golden_free_reed_deterministic() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    // 1. Tango Bandoneon (142-tone Alfred Arnold, 16'+8' Doble Cassotto)
    let mut bandoneon = FreeReed::new(sample_rate);
    bandoneon.set_profile(FreeReedProfile::TangoBandoneon);
    bandoneon.set_register_mask(REGISTER_BANDONEON);
    bandoneon.set_bellows_pressure(650.0); // Push stroke
    bandoneon.note_on(57, 0.90); // A3
    bandoneon.note_on(60, 0.85); // C4
    bandoneon.note_on(64, 0.82); // E4

    // 2. French Musette Accordion (Triple 8' musette shimmer)
    let mut musette = FreeReed::new(sample_rate);
    musette.set_profile(FreeReedProfile::FrenchMusetteAccordion);
    musette.set_register_mask(REGISTER_MUSETTE);
    musette.set_musette_detune(18.0); // 18 cents detune spread
    musette.set_bellows_pressure(420.0);
    musette.note_on(60, 0.80); // C4
    musette.note_on(64, 0.78); // E4
    musette.note_on(67, 0.75); // G4

    // 3. Russian Chromatic Bayan (Solid duralumin reedblocks, Full Master)
    let mut bayan = FreeReed::new(sample_rate);
    bayan.set_profile(FreeReedProfile::RussianBayan);
    bayan.set_register_mask(REGISTER_MASTER);
    bayan.set_bellows_pressure(850.0);
    bayan.note_on(36, 0.95); // C2 deep bass
    bayan.note_on(48, 0.90); // C3
    bayan.note_on(60, 0.88); // C4

    // 4. Vintage Suction Harmonium (Continuous suction pull)
    let mut harmonium = FreeReed::new(sample_rate);
    harmonium.set_profile(FreeReedProfile::VintageHarmonium);
    harmonium.set_register_mask(REGISTER_ACCORDION);
    harmonium.set_bellows_pressure(-320.0); // Suction pull
    harmonium.note_on(48, 0.75);
    harmonium.note_on(55, 0.70);
    harmonium.note_on(62, 0.72);

    // 5. English Treble Concertina (Fast steel reeds, solo 8' clarinet)
    let mut concertina = FreeReed::new(sample_rate);
    concertina.set_profile(FreeReedProfile::EnglishConcertina);
    concertina.set_register_mask(REGISTER_CLARINET);
    concertina.set_bellows_pressure(520.0);
    concertina.note_on(72, 0.85); // C5
    concertina.note_on(76, 0.82); // E5

    let mut out_band_l = [0.0f32; BLOCK_SIZE];
    let mut out_band_r = [0.0f32; BLOCK_SIZE];
    let mut out_muse_l = [0.0f32; BLOCK_SIZE];
    let mut out_muse_r = [0.0f32; BLOCK_SIZE];
    let mut out_bayan_l = [0.0f32; BLOCK_SIZE];
    let mut out_bayan_r = [0.0f32; BLOCK_SIZE];
    let mut out_harm_l = [0.0f32; BLOCK_SIZE];
    let mut out_harm_r = [0.0f32; BLOCK_SIZE];
    let mut out_conc_l = [0.0f32; BLOCK_SIZE];
    let mut out_conc_r = [0.0f32; BLOCK_SIZE];

    let mut hasher = Hasher::new();

    // Process 64 blocks of audio (4096 samples)
    for block_idx in 0..64 {
        let ctx = ProcessContext::from_transport(&transport);

        {
            let _guard = AllocGuard::new();
            bandoneon.process_block(&[], &mut [&mut out_band_l[..], &mut out_band_r[..]], &ctx);
            musette.process_block(&[], &mut [&mut out_muse_l[..], &mut out_muse_r[..]], &ctx);
            bayan.process_block(&[], &mut [&mut out_bayan_l[..], &mut out_bayan_r[..]], &ctx);
            harmonium.process_block(&[], &mut [&mut out_harm_l[..], &mut out_harm_r[..]], &ctx);
            concertina.process_block(&[], &mut [&mut out_conc_l[..], &mut out_conc_r[..]], &ctx);
        }

        if block_idx == 32 {
            // Push-to-pull stroke reversal & harmonic transitions halfway
            bandoneon.set_bellows_pressure(-700.0); // Pull stroke accent
            bandoneon.note_off(57);
            bandoneon.note_on(69, 0.92); // A4

            musette.set_bellows_pressure(580.0); // Swell crest
            musette.set_musette_detune(22.0);

            bayan.set_bellows_pressure(-900.0); // Violent bellows shake pull
            bayan.note_on(72, 0.95);

            harmonium.set_cassotto_aperture(0.35); // Darken tone chamber

            concertina.note_off(72);
            concertina.note_on(79, 0.88); // G5
        }

        for i in 0..BLOCK_SIZE {
            assert!(out_band_l[i].is_finite() && out_band_r[i].is_finite());
            assert!(out_muse_l[i].is_finite() && out_muse_r[i].is_finite());
            assert!(out_bayan_l[i].is_finite() && out_bayan_r[i].is_finite());
            assert!(out_harm_l[i].is_finite() && out_harm_r[i].is_finite());
            assert!(out_conc_l[i].is_finite() && out_conc_r[i].is_finite());

            assert!((-1.0..=1.0).contains(&out_band_l[i]) && (-1.0..=1.0).contains(&out_band_r[i]));
            assert!((-1.0..=1.0).contains(&out_muse_l[i]) && (-1.0..=1.0).contains(&out_muse_r[i]));
            assert!((-1.0..=1.0).contains(&out_bayan_l[i]) && (-1.0..=1.0).contains(&out_bayan_r[i]));
            assert!((-1.0..=1.0).contains(&out_harm_l[i]) && (-1.0..=1.0).contains(&out_harm_r[i]));
            assert!((-1.0..=1.0).contains(&out_conc_l[i]) && (-1.0..=1.0).contains(&out_conc_r[i]));

            hasher.update(&out_band_l[i].to_le_bytes());
            hasher.update(&out_band_r[i].to_le_bytes());
            hasher.update(&out_muse_l[i].to_le_bytes());
            hasher.update(&out_muse_r[i].to_le_bytes());
            hasher.update(&out_bayan_l[i].to_le_bytes());
            hasher.update(&out_bayan_r[i].to_le_bytes());
            hasher.update(&out_harm_l[i].to_le_bytes());
            hasher.update(&out_harm_r[i].to_le_bytes());
            hasher.update(&out_conc_l[i].to_le_bytes());
            hasher.update(&out_conc_r[i].to_le_bytes());
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_free_reed.hash", &digest);
}

#[test]
fn test_golden_free_reed_cassotto_and_aeroelastic_deterministic() {
    let sample_rate = 48000;
    let sr = sample_rate as f32;

    let mut reed_osc = FreeReedOscillator::new();
    let mut cassotto = CassottoChamber::new(sample_rate);
    cassotto.set_profile(CassottoProfile::DobleCassottoBandoneon);

    let mut hasher = Hasher::new();

    for step in 0..2048 {
        let is_push = step < 1024;
        let pressure_pa = if is_push { 600.0 } else { -600.0 };

        let (acoustic_reed, chamber_out) = {
            let _guard = AllocGuard::new();
            let reed_s = reed_osc.process_step(440.0, pressure_pa, sr, 1.25, is_push);
            let ch_s = cassotto.process_sample(reed_s, reed_s * 0.5);
            (reed_s, ch_s)
        };

        assert!(acoustic_reed.is_finite());
        assert!(chamber_out.is_finite());

        hasher.update(&acoustic_reed.to_le_bytes());
        hasher.update(&chamber_out.to_le_bytes());
        hasher.update(&reed_osc.displacement.to_le_bytes());
        hasher.update(&reed_osc.velocity.to_le_bytes());
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_free_reed_cassotto.hash", &digest);
}

#[test]
fn test_golden_free_reed_gesture_and_bus_deterministic() {
    let mut engine = BellowsGestureEngine::with_pattern(
        BellowsGesturePattern::TangoBandoneonAccented {
            phrase_length_beats: 4.0,
            max_pressure_pa: 800.0,
        },
    );
    engine.is_looping = true;

    let bus = BellowsBus::new();
    let mut param_bus = ParamBus::new();
    register_bellows_params(&mut param_bus, BELLOWS_BASE_PARAM_ID);

    let mut hasher = Hasher::new();

    for step in 0..160 {
        let beat = step as f64 * 0.05; // 8 beats total (2 loops)

        let snap = {
            let _guard = AllocGuard::new();
            engine.dispatch_to_bus(beat, &bus, Some(&param_bus), BELLOWS_BASE_PARAM_ID);
            bus.snapshot()
        };

        assert!(snap.bellows_pressure_pa.is_finite());
        assert!(snap.valve_velocity.is_finite());
        assert!(snap.cassotto_aperture.is_finite());
        assert!(snap.musette_detune_cents.is_finite());
        assert!(snap.reed_stiffness.is_finite());
        assert!(snap.master_gain.is_finite());

        hasher.update(&(snap.articulation as u32).to_le_bytes());
        hasher.update(&snap.bellows_pressure_pa.to_le_bytes());
        hasher.update(&snap.push_direction.to_le_bytes());
        hasher.update(&snap.valve_velocity.to_le_bytes());
        hasher.update(&snap.cassotto_aperture.to_le_bytes());
        hasher.update(&snap.musette_detune_cents.to_le_bytes());
        hasher.update(&snap.register_mask.to_le_bytes());
        hasher.update(&snap.reed_stiffness.to_le_bytes());
        hasher.update(&snap.master_gain.to_le_bytes());
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_free_reed_gesture.hash", &digest);
}

#[cfg(feature = "gui")]
#[test]
fn test_golden_free_reed_views_ascii_and_snapshots() {
    let bellows_view = BellowsView::new();
    let free_reed_view = FreeReedView::new();

    let bellows_ascii = bellows_view.render_ascii(80, 16);
    let free_reed_ascii = free_reed_view.render_ascii(80, 16);

    let mut hasher = Hasher::new();
    for line in &bellows_ascii {
        hasher.update(line.as_bytes());
    }
    for line in &free_reed_ascii {
        hasher.update(line.as_bytes());
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_free_reed_views.hash", &digest);

    // Save PNG snapshots across the project render directories
    let _ = bellows_view.render_snapshot_png("scratch/renders/bellows_view.png", 800, 520);
    let _ = bellows_view.render_snapshot_png("../../scratch/renders/bellows_view.png", 800, 520);
    let _ = bellows_view.render_snapshot_png("crates/summon/scratch/renders/bellows_view.png", 800, 520);
    let _ = bellows_view.render_snapshot_png("crates/summoner_gui/scratch/renders/bellows_view.png", 800, 520);

    let _ = free_reed_view.render_snapshot_png("scratch/renders/free_reed_view.png", 800, 520);
    let _ = free_reed_view.render_snapshot_png("../../scratch/renders/free_reed_view.png", 800, 520);
    let _ = free_reed_view.render_snapshot_png("crates/summon/scratch/renders/free_reed_view.png", 800, 520);
    let _ = free_reed_view.render_snapshot_png("crates/summoner_gui/scratch/renders/free_reed_view.png", 800, 520);
}
