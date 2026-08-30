// Summoner - Deterministic Golden Woodwind Air-Jet & Tonehole Radiation Regression Suite
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

use blake3::Hasher;
use std::fs;
use std::path::Path;
use summoner_core::allocator::AllocGuard;
use summoner_core::breath_bus::{BreathBus, WoodwindArticulation};
use summoner_core::node::ProcessContext;
use summoner_core::param_bus::{ParamBus, ParamId};
use summoner_core::transport::Transport;
use summoner_dsp::tonehole_grid::ToneholeLattice;
use summoner_dsp::traits::SignalProcessor;
use summoner_dsp::woodwind_jet::{WoodwindInstrumentProfile, WoodwindJet};
#[cfg(feature = "gui")]
use summoner_gui::views::tonehole_matrix_view::ToneholeMatrixView;
#[cfg(feature = "gui")]
use summoner_gui::views::woodwind_jet_view::WoodwindJetView;
use summoner_sequencer::woodwind_fingering::{
    WoodwindFingeringEngine, WoodwindGesturePattern, WoodwindInterpolationCurve, WoodwindWaypoint,
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
            "Golden woodwind air-jet & tonehole radiation hash mismatch for {}",
            hash_filename
        );
    } else {
        fs::write(&hash_file, digest).expect("Failed to write golden hash");
    }
}

#[test]
fn test_golden_woodwind_jet_oscillation_deterministic() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    // 1. Concert Flute C4
    let mut flute = WoodwindJet::new(sample_rate);
    flute.set_instrument(WoodwindInstrumentProfile::FluteC);
    flute.set_frequency(261.63);
    flute.set_blowing_pressure(1.30);
    flute.set_jet_distance(7.0);
    flute.set_vibrato(0.25, 5.5);

    // 2. Piccolo C5
    let mut piccolo = WoodwindJet::new(sample_rate);
    piccolo.set_instrument(WoodwindInstrumentProfile::PiccoloC);
    piccolo.set_frequency(523.25);
    piccolo.set_blowing_pressure(1.60);
    piccolo.set_jet_distance(4.5);
    piccolo.set_fingering_mask(0x1F); // 1 hole open

    // 3. Alto Recorder F4
    let mut recorder = WoodwindJet::new(sample_rate);
    recorder.set_instrument(WoodwindInstrumentProfile::RecorderAlto);
    recorder.set_frequency(349.23);
    recorder.set_blowing_pressure(1.10);
    recorder.set_jet_distance(3.5);
    recorder.set_fingering_mask(0x07); // 3 holes open

    // 4. Shakuhachi D4
    let mut shakuhachi = WoodwindJet::new(sample_rate);
    shakuhachi.set_instrument(WoodwindInstrumentProfile::Shakuhachi);
    shakuhachi.set_frequency(293.66);
    shakuhachi.set_blowing_pressure(1.80);
    shakuhachi.set_jet_distance(10.0);
    shakuhachi.set_breath_noise(0.20);
    shakuhachi.set_embouchure_angle(-0.15); // Meri pitch bend

    // 5. Pan Flute C4
    let mut panflute = WoodwindJet::new(sample_rate);
    panflute.set_instrument(WoodwindInstrumentProfile::PanFlute);
    panflute.set_frequency(261.63);
    panflute.set_blowing_pressure(1.40);
    panflute.set_jet_distance(5.0);

    let in_dummy = [0.0f32; BLOCK_SIZE];
    let mut out_flute = [0.0f32; BLOCK_SIZE];
    let mut out_piccolo = [0.0f32; BLOCK_SIZE];
    let mut out_recorder = [0.0f32; BLOCK_SIZE];
    let mut out_shakuhachi = [0.0f32; BLOCK_SIZE];
    let mut out_panflute = [0.0f32; BLOCK_SIZE];

    let mut hasher = Hasher::new();

    // Process 64 blocks of audio (4096 samples)
    for _block_idx in 0..64 {
        let ctx = ProcessContext::from_transport(&transport);

        {
            let _guard = AllocGuard::new();
            flute.process_block(&[&in_dummy[..]], &mut [&mut out_flute[..]], &ctx);
            piccolo.process_block(&[&in_dummy[..]], &mut [&mut out_piccolo[..]], &ctx);
            recorder.process_block(&[&in_dummy[..]], &mut [&mut out_recorder[..]], &ctx);
            shakuhachi.process_block(&[&in_dummy[..]], &mut [&mut out_shakuhachi[..]], &ctx);
            panflute.process_block(&[&in_dummy[..]], &mut [&mut out_panflute[..]], &ctx);
        }

        for i in 0..BLOCK_SIZE {
            assert!(out_flute[i].is_finite());
            assert!(out_piccolo[i].is_finite());
            assert!(out_recorder[i].is_finite());
            assert!(out_shakuhachi[i].is_finite());
            assert!(out_panflute[i].is_finite());

            assert!((-1.0..=1.0).contains(&out_flute[i]));
            assert!((-1.0..=1.0).contains(&out_piccolo[i]));
            assert!((-1.0..=1.0).contains(&out_recorder[i]));
            assert!((-1.0..=1.0).contains(&out_shakuhachi[i]));
            assert!((-1.0..=1.0).contains(&out_panflute[i]));

            hasher.update(&out_flute[i].to_le_bytes());
            hasher.update(&out_piccolo[i].to_le_bytes());
            hasher.update(&out_recorder[i].to_le_bytes());
            hasher.update(&out_shakuhachi[i].to_le_bytes());
            hasher.update(&out_panflute[i].to_le_bytes());
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_woodwind_jet.hash", &digest);
}

#[test]
fn test_golden_tonehole_lattice_scattering_deterministic() {
    let mut lattice = ToneholeLattice::new_standard_flute(0.60);
    let mut hasher = Hasher::new();

    let fingerings: [u8; 8] = [0x3F, 0x1F, 0x0F, 0x07, 0x03, 0x01, 0x00, 0x2A];

    for &mask in &fingerings {
        lattice.set_fingering_mask(mask);
        let eff_len = lattice.effective_acoustic_length_m();
        let fc = lattice.lattice_cutoff_hz;

        hasher.update(&eff_len.to_le_bytes());
        hasher.update(&fc.to_le_bytes());

        for step in 0..64 {
            let fwd_in = (step as f32 * 0.1).sin() * 0.5;
            let rev_in = (step as f32 * 0.15).cos() * 0.3;

            let mut cur_fwd = fwd_in;
            let mut cur_rev = rev_in;
            let mut total_rad = 0.0f32;

            {
                let _guard = AllocGuard::new();
                for hole in lattice.toneholes.iter_mut() {
                    let res = hole.process_scattering(cur_fwd, cur_rev, 0.998);
                    cur_fwd = res.transmitted_downstream;
                    cur_rev += res.reflected_upstream;
                    total_rad += res.radiated_pressure;
                }
            }

            hasher.update(&cur_fwd.to_le_bytes());
            hasher.update(&cur_rev.to_le_bytes());
            hasher.update(&total_rad.to_le_bytes());
        }
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_tonehole_grid.hash", &digest);
}

#[test]
fn test_golden_woodwind_fingering_and_breath_bus_deterministic() {
    let mut engine = WoodwindFingeringEngine::new();
    engine.pattern = WoodwindGesturePattern::CustomSpline;
    engine.interpolation = WoodwindInterpolationCurve::CubicCatmullRom;
    engine.loop_length_beats = Some(8.0);

    engine.add_waypoint(WoodwindWaypoint {
        beat: 0.0,
        blowing_pressure_kpa: 1.00,
        jet_distance_mm: 7.5,
        embouchure_angle_rad: 0.0,
        fingering_mask: 0x3F,
        articulation: WoodwindArticulation::Legato,
        vibrato_depth: 0.10,
        vibrato_rate_hz: 5.5,
        flutter_rate_hz: 0.0,
        breath_noise_mix: 0.08,
    });

    engine.add_waypoint(WoodwindWaypoint {
        beat: 2.0,
        blowing_pressure_kpa: 1.80,
        jet_distance_mm: 6.0,
        embouchure_angle_rad: 0.08,
        fingering_mask: 0x1F,
        articulation: WoodwindArticulation::TonguedStaccato,
        vibrato_depth: 0.0,
        vibrato_rate_hz: 5.5,
        flutter_rate_hz: 0.0,
        breath_noise_mix: 0.15,
    });

    engine.add_waypoint(WoodwindWaypoint {
        beat: 4.0,
        blowing_pressure_kpa: 2.60,
        jet_distance_mm: 4.5,
        embouchure_angle_rad: 0.18,
        fingering_mask: 0x07,
        articulation: WoodwindArticulation::OverblowHarmonic,
        vibrato_depth: 0.15,
        vibrato_rate_hz: 6.0,
        flutter_rate_hz: 25.0,
        breath_noise_mix: 0.20,
    });

    engine.add_waypoint(WoodwindWaypoint {
        beat: 6.0,
        blowing_pressure_kpa: 1.40,
        jet_distance_mm: 9.0,
        embouchure_angle_rad: -0.15,
        fingering_mask: 0x00,
        articulation: WoodwindArticulation::Legato,
        vibrato_depth: 0.25,
        vibrato_rate_hz: 5.0,
        flutter_rate_hz: 0.0,
        breath_noise_mix: 0.12,
    });

    let bus = BreathBus::new();
    let mut param_bus = ParamBus::new();
    for i in 0..12 {
        param_bus.register(ParamId(700 + i), 0.0);
    }

    let mut hasher = Hasher::new();

    for step in 0..160 {
        let beat = step as f64 * 0.05; // 8 beats total

        let snap = {
            let _guard = AllocGuard::new();
            engine.dispatch_to_breath_bus(beat, &bus);
            engine.dispatch_to_param_bus(beat, &param_bus, ParamId(700));
            bus.snapshot()
        };

        assert!(snap.blowing_pressure_kpa.is_finite());
        assert!(snap.jet_distance_mm.is_finite());
        assert!(snap.embouchure_angle_rad.is_finite());

        hasher.update(&(snap.articulation as u32).to_le_bytes());
        hasher.update(&snap.blowing_pressure_kpa.to_le_bytes());
        hasher.update(&snap.jet_distance_mm.to_le_bytes());
        hasher.update(&snap.embouchure_angle_rad.to_le_bytes());
        hasher.update(&[snap.fingering_mask]);
        hasher.update(&snap.vibrato_depth.to_le_bytes());
        hasher.update(&snap.vibrato_rate_hz.to_le_bytes());
        hasher.update(&snap.flutter_rate_hz.to_le_bytes());
        hasher.update(&snap.breath_noise_mix.to_le_bytes());
        hasher.update(&snap.gesture_progress.to_le_bytes());
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_woodwind_fingering.hash", &digest);
}

#[cfg(feature = "gui")]
#[test]
fn test_golden_woodwind_views_ascii_renders() {
    let jet_view = WoodwindJetView::new();
    let tonehole_view = ToneholeMatrixView::new();

    let jet_ascii = jet_view.render_ascii(64, 16);
    let tonehole_ascii = tonehole_view.render_ascii(64, 16);

    let mut hasher = Hasher::new();
    for line in &jet_ascii {
        hasher.update(line.as_bytes());
    }
    for line in &tonehole_ascii {
        hasher.update(line.as_bytes());
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_woodwind_views.hash", &digest);
}
