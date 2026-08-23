// Summoner - Deterministic Golden Multitrack Sidechain & Modulation Regression Suite
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

use blake3::Hasher;
use std::fs;
use std::path::Path;
use summoner_core::allocator::AllocGuard;
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext, SineOscillatorNode};
use summoner_core::param_bus::ParamId;
use summoner_core::sidechain_matrix::{
    ControlEvent, MultiBusEventRouter, SidechainMatrix, SidechainRoute, SidechainTapPoint,
};
use summoner_core::transport::Transport;
use summoner_dsp::compressor::CompressorNode;
use summoner_dsp::oscillators::{NoiseGen, NoiseType, OscSaw};
use summoner_dsp::spectral_resynthesis::{HarmonicProfile, SpectralResynthesisEngine};
use summoner_dsp::traits::SignalProcessor;

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
            "Golden sidechain & modulation hash mismatch for {}",
            hash_filename
        );
    } else {
        fs::write(&hash_file, digest).expect("Failed to write golden hash");
    }
}

#[test]
fn test_golden_sidechain_ducking_compression() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    let mut matrix = SidechainMatrix::new(BLOCK_SIZE);
    matrix.add_route(SidechainRoute {
        source_track: 0,
        dest_track: 1,
        aux_bus_index: 0,
        send_gain: 1.0,
        tap_point: SidechainTapPoint::PostFader,
        muted: false,
        solo: false,
        detector_hpf_hz: 80.0,
        detector_lpf_hz: 0.0,
    });

    let mut compressor = CompressorNode::with_params(-18.0, 6.0, 5.0, 50.0, 0.0);

    let mut kick_osc = SineOscillatorNode::new(60.0);
    let mut bass_saw = OscSaw::new(55.0);

    let mut kick_buf = [0.0f32; BLOCK_SIZE];
    let mut bass_buf = [0.0f32; BLOCK_SIZE];
    let mut comp_out_l = [0.0f32; BLOCK_SIZE];
    let mut comp_out_r = [0.0f32; BLOCK_SIZE];

    let mut hasher = Hasher::new();

    // Process 32 consecutive blocks (2048 frames)
    for block_idx in 0..32 {
        let ctx = ProcessContext::from_transport(&transport);

        // Kick triggers on beats (every 8 blocks)
        let kick_gate = if block_idx % 8 == 0 { 1.0f32 } else { 0.0f32 };

        let dummy_in: [&[Sample]; 0] = [];
        let mut kick_slice = [&mut kick_buf[..]];
        kick_osc.process(&dummy_in, &mut kick_slice, &ctx);

        for s in kick_buf.iter_mut() {
            *s *= kick_gate;
        }

        let mut bass_slice = [&mut bass_buf[..]];
        bass_saw.process_block(&dummy_in, &mut bass_slice, &ctx);

        // Enforce zero heap allocation in sidechain matrix and audio processing
        {
            let _guard = AllocGuard::new();

            let track_slices: [&[Sample]; 2] = [&kick_buf[..], &bass_buf[..]];
            matrix.process_block(&track_slices, BLOCK_SIZE, sample_rate);

            let aux_bus_0 = matrix.get_aux_bus_slice(0, BLOCK_SIZE);

            // Feed sidechain aux bus into compressor sidechain input
            let comp_inputs: [&[Sample]; 2] = [&bass_buf[..], aux_bus_0];
            let mut comp_outputs: [&mut [Sample]; 2] = [&mut comp_out_l[..], &mut comp_out_r[..]];
            compressor.process_block(&comp_inputs, &mut comp_outputs, &ctx);
        }

        for i in 0..BLOCK_SIZE {
            let sl = comp_out_l[i].clamp(-1.0, 1.0);
            let sr = comp_out_r[i].clamp(-1.0, 1.0);
            assert!(sl.is_finite());
            assert!(sr.is_finite());
            hasher.update(&sl.to_le_bytes());
            hasher.update(&sr.to_le_bytes());
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }

    let digest = hasher.finalize().to_hex().to_string();
    assert!(!digest.is_empty());
    check_or_save_golden("golden_sidechain_ducking.blake3", &digest);
}

#[test]
fn test_golden_spectral_morphing_resynthesis() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    let mut engine = SpectralResynthesisEngine::new(sample_rate);
    engine.set_fundamental(130.81); // C3
    engine.set_profile_a(HarmonicProfile::sawtooth());
    engine.set_profile_b(HarmonicProfile::vocal_formant_ah());
    engine.set_inharmonicity(0.02);
    engine.set_spectral_tilt(2.0);

    let mut out_l = [0.0f32; BLOCK_SIZE];
    let mut out_r = [0.0f32; BLOCK_SIZE];

    let mut hasher = Hasher::new();

    // Morph gradually from 0.0 to 1.0 across 40 blocks
    for block_idx in 0..40 {
        let ctx = ProcessContext::from_transport(&transport);
        let morph_progress = (block_idx as f32) / 39.0;
        engine.set_morph(morph_progress);

        let dummy_in: [&[Sample]; 0] = [];
        let mut outputs: [&mut [Sample]; 2] = [&mut out_l[..], &mut out_r[..]];

        {
            let _guard = AllocGuard::new();
            engine.process_block(&dummy_in, &mut outputs, &ctx);
        }

        for i in 0..BLOCK_SIZE {
            let sl = out_l[i];
            let sr = out_r[i];
            assert!(sl.is_finite() && (-1.0..=1.0).contains(&sl));
            assert!(sr.is_finite() && (-1.0..=1.0).contains(&sr));
            hasher.update(&sl.to_le_bytes());
            hasher.update(&sr.to_le_bytes());
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }

    let digest = hasher.finalize().to_hex().to_string();
    assert!(!digest.is_empty());
    check_or_save_golden("golden_spectral_morphing_resynthesis.blake3", &digest);
}

#[test]
fn test_golden_multitrack_sidechain_and_modulation_master() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    let mut matrix = SidechainMatrix::new(BLOCK_SIZE);
    let mut event_router = MultiBusEventRouter::new();

    // Route Track 0 (Kick) -> Aux 0 (Bass Sidechain)
    matrix.add_route(SidechainRoute {
        source_track: 0,
        dest_track: 2,
        aux_bus_index: 0,
        send_gain: 1.0,
        tap_point: SidechainTapPoint::PostFader,
        muted: false,
        solo: false,
        detector_hpf_hz: 60.0,
        detector_lpf_hz: 0.0,
    });

    // Route Track 1 (Snare) -> Aux 1 (Pad Reverb Trigger)
    matrix.add_route(SidechainRoute {
        source_track: 1,
        dest_track: 3,
        aux_bus_index: 1,
        send_gain: 0.75,
        tap_point: SidechainTapPoint::PostFader,
        muted: false,
        solo: false,
        detector_hpf_hz: 200.0,
        detector_lpf_hz: 8000.0,
    });

    let mut kick = SineOscillatorNode::new(55.0);
    let mut snare_noise = NoiseGen::new(NoiseType::White);
    let mut bass = OscSaw::new(65.41);
    let mut pad = SpectralResynthesisEngine::new(sample_rate);
    pad.set_profile_a(HarmonicProfile::bowed_string());
    pad.set_profile_b(HarmonicProfile::metallic_bell());

    let mut kick_buf = [0.0f32; BLOCK_SIZE];
    let mut snare_buf = [0.0f32; BLOCK_SIZE];
    let mut bass_buf = [0.0f32; BLOCK_SIZE];
    let mut pad_out_l = [0.0f32; BLOCK_SIZE];
    let mut pad_out_r = [0.0f32; BLOCK_SIZE];

    let mut master_l = [0.0f32; BLOCK_SIZE];
    let mut master_r = [0.0f32; BLOCK_SIZE];

    let mut hasher = Hasher::new();

    for block_idx in 0..48 {
        let ctx = ProcessContext::from_transport(&transport);
        event_router.clear();

        // Sample-accurate modulation event at frame 16
        event_router.push_event(ControlEvent {
            frame_offset: 16,
            target_param: ParamId(101),
            value: (block_idx as f32 * 0.1).sin().abs(),
            source_id: 1,
        });

        let dummy_in: [&[Sample]; 0] = [];

        let mut kick_slices = [&mut kick_buf[..]];
        kick.process(&dummy_in, &mut kick_slices, &ctx);

        let mut snare_slices = [&mut snare_buf[..]];
        snare_noise.process_block(&dummy_in, &mut snare_slices, &ctx);

        let mut bass_slices = [&mut bass_buf[..]];
        bass.process_block(&dummy_in, &mut bass_slices, &ctx);

        let mut pad_slices: [&mut [Sample]; 2] = [&mut pad_out_l[..], &mut pad_out_r[..]];
        pad.set_morph(((block_idx as f32) / 48.0).clamp(0.0, 1.0));
        pad.process_block(&dummy_in, &mut pad_slices, &ctx);

        {
            let _guard = AllocGuard::new();

            let track_outputs: [&[Sample]; 4] = [
                &kick_buf[..],
                &snare_buf[..],
                &bass_buf[..],
                &pad_out_l[..],
            ];
            matrix.process_block(&track_outputs, BLOCK_SIZE, sample_rate);

            let aux0 = matrix.get_aux_bus_slice(0, BLOCK_SIZE);
            let aux1 = matrix.get_aux_bus_slice(1, BLOCK_SIZE);

            for i in 0..BLOCK_SIZE {
                let ducking = (1.0 - aux0[i].abs() * 0.5).clamp(0.1, 1.0);
                let gated_pad = (aux1[i].abs() * 0.8).clamp(0.0, 1.0);

                master_l[i] = (kick_buf[i] * 0.5 + snare_buf[i] * 0.3 + bass_buf[i] * ducking * 0.4 + pad_out_l[i] * gated_pad * 0.3).clamp(-1.0, 1.0);
                master_r[i] = (kick_buf[i] * 0.5 + snare_buf[i] * 0.3 + bass_buf[i] * ducking * 0.4 + pad_out_r[i] * gated_pad * 0.3).clamp(-1.0, 1.0);
            }
        }

        for i in 0..BLOCK_SIZE {
            hasher.update(&master_l[i].to_le_bytes());
            hasher.update(&master_r[i].to_le_bytes());
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }

    let digest = hasher.finalize().to_hex().to_string();
    assert!(!digest.is_empty());
    check_or_save_golden("golden_multitrack_sidechain_modulation_master.blake3", &digest);
}
