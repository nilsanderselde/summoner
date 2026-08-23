// Summoner - Deterministic Golden Analog Tape Saturation & Generative Markov Regression Suite
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

use blake3::Hasher;
use std::f32::consts::PI;
use std::fs;
use std::path::Path;
use summoner_core::allocator::AllocGuard;
use summoner_core::node::ProcessContext;
use summoner_core::param_bus::{ParamBus, ParamId};
use summoner_core::transport::Transport;
use summoner_dsp::tape_saturation::{TapeSaturationNode, TapeSpeed};
use summoner_dsp::traits::SignalProcessor;
#[cfg(feature = "gui")]
use summoner_gui::views::tape_saturation_view::TapeSaturationView;
use summoner_sequencer::markov_mutator::{
    MarkovOrder, MarkovSequenceMutator, MarkovStepEvent, PolyrhythmicDivision,
};
use summoner_sequencer::timeline::TimelineArranger;

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
            "Golden tape saturation & Markov mutator hash mismatch for {}",
            hash_filename
        );
    } else {
        fs::write(&hash_file, digest).expect("Failed to write golden hash");
    }
}

#[test]
fn test_golden_analog_tape_saturation_and_hysteresis_multispeed() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    let mut tape_7_5 = TapeSaturationNode::new(2.8, 0.75).with_speed(TapeSpeed::Ips7_5);
    let mut tape_15 = TapeSaturationNode::new(2.0, 0.60).with_speed(TapeSpeed::Ips15);
    let mut tape_30 = TapeSaturationNode::new(1.4, 0.40).with_speed(TapeSpeed::Ips30);

    tape_7_5.bias = 0.15;
    tape_7_5.hysteresis = 0.7;
    tape_7_5.wow_flutter = 0.25;

    tape_15.bias = 0.0;
    tape_15.hysteresis = 0.5;
    tape_15.wow_flutter = 0.15;

    tape_30.bias = -0.1;
    tape_30.hysteresis = 0.3;
    tape_30.wow_flutter = 0.05;

    let mut in_l = [0.0f32; BLOCK_SIZE];
    let mut in_r = [0.0f32; BLOCK_SIZE];
    let mut out_l_75 = [0.0f32; BLOCK_SIZE];
    let mut out_r_75 = [0.0f32; BLOCK_SIZE];
    let mut out_l_15 = [0.0f32; BLOCK_SIZE];
    let mut out_r_15 = [0.0f32; BLOCK_SIZE];
    let mut out_l_30 = [0.0f32; BLOCK_SIZE];
    let mut out_r_30 = [0.0f32; BLOCK_SIZE];

    let mut hasher = Hasher::new();

    // Process 64 blocks of audio across all tape speeds (4096 samples)
    for block_idx in 0..64 {
        let ctx = ProcessContext::from_transport(&transport);

        for i in 0..BLOCK_SIZE {
            let n = (block_idx * BLOCK_SIZE + i) as f32;
            let low = (2.0 * PI * 60.0 * n / 48000.0).sin() * 0.5;
            let mid = (2.0 * PI * 1000.0 * n / 48000.0).sin() * 0.4;
            let high = (2.0 * PI * 8000.0 * n / 48000.0).cos() * 0.3;
            in_l[i] = low + mid + high;
            in_r[i] = low * 0.9 + mid * 1.1 + high * 0.8;
        }

        {
            let _guard = AllocGuard::new();
            tape_7_5.process_block(
                &[&in_l[..], &in_r[..]],
                &mut [&mut out_l_75[..], &mut out_r_75[..]],
                &ctx,
            );
            tape_15.process_block(
                &[&in_l[..], &in_r[..]],
                &mut [&mut out_l_15[..], &mut out_r_15[..]],
                &ctx,
            );
            tape_30.process_block(
                &[&in_l[..], &in_r[..]],
                &mut [&mut out_l_30[..], &mut out_r_30[..]],
                &ctx,
            );
        }

        for i in 0..BLOCK_SIZE {
            assert!(out_l_75[i].is_finite());
            assert!(out_r_75[i].is_finite());
            assert!(out_l_15[i].is_finite());
            assert!(out_r_15[i].is_finite());
            assert!(out_l_30[i].is_finite());
            assert!(out_r_30[i].is_finite());

            hasher.update(&out_l_75[i].to_le_bytes());
            hasher.update(&out_r_75[i].to_le_bytes());
            hasher.update(&out_l_15[i].to_le_bytes());
            hasher.update(&out_r_15[i].to_le_bytes());
            hasher.update(&out_l_30[i].to_le_bytes());
            hasher.update(&out_r_30[i].to_le_bytes());
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_tape_saturation.hash", &digest);
}

#[test]
fn test_golden_generative_markov_sequence_mutator_deterministic() {
    let mut mutator = MarkovSequenceMutator::new()
        .with_order(MarkovOrder::Order3)
        .with_temperature(0.7)
        .with_division(PolyrhythmicDivision::Quintuplet16th);

    let training_phrase = vec![
        60.0, 62.0, 64.0, 67.0, 69.0, 67.0, 64.0, 62.0,
        60.0, 63.0, 65.0, 68.0, 70.0, 68.0, 65.0, 63.0,
    ];
    mutator.train_from_pitches(&training_phrase);

    let mut hasher = Hasher::new();

    // Generate multiple deterministic sequences with fixed seed
    let seed = 0x5EED_C0DE_1234_5678u64;
    let sequence_order3 = mutator.generate_sequence(&[60.0, 62.0, 64.0], 32, seed);
    assert_eq!(sequence_order3.len(), 32);

    for ev in &sequence_order3 {
        hasher.update(&ev.pitch.to_le_bytes());
        hasher.update(&ev.velocity.to_le_bytes());
        hasher.update(&ev.gate.to_le_bytes());
        hasher.update(&ev.probability.to_le_bytes());
        hasher.update(&[ev.ratchet]);
    }

    // Polyrhythmic clip generation test
    let clip = mutator.generate_polyrhythmic_clip("Markov Lead", 2, &[60.0, 63.0], seed);
    assert_eq!(clip.steps.len(), 40); // 2 bars = 8 beats / 0.2 step_beats = 40 steps
    for step in &clip.steps {
        hasher.update(&step.note.to_le_bytes());
        hasher.update(&step.velocity.to_le_bytes());
    }

    // Timeline arranger dispatch
    let mut arranger = TimelineArranger::new();
    mutator.dispatch_to_timeline(&mut arranger, 0, 0.0, 20, &[60.0, 62.0], seed);
    let eval = arranger.evaluate(0.0, 4.0);
    for n in &eval.note_events {
        hasher.update(&n.note.to_le_bytes());
        hasher.update(&n.beat_offset.to_le_bytes());
    }

    // ParamBus parameter modulation dispatch
    let mut param_bus = ParamBus::new();
    let p_pitch = param_bus.register(ParamId(10), 0.0);
    let p_vel = param_bus.register(ParamId(11), 0.0);
    let step_ev = MarkovStepEvent::new(67.5, 0.88, 0.75);
    mutator.dispatch_to_param_bus(&param_bus, ParamId(10), ParamId(11), &step_ev);

    assert_eq!(p_pitch.get(), 67.5);
    assert_eq!(p_vel.get(), 0.88);

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_markov_mutator.hash", &digest);
}

#[cfg(feature = "gui")]
#[test]
fn test_golden_tape_saturation_view_snapshot_and_telemetry() {
    let mut view = TapeSaturationView::new();
    view.drive = 2.4;
    view.saturation = 0.7;
    view.bias = 0.1;
    view.hysteresis = 0.6;
    view.speed = summoner_gui::views::tape_saturation_view::TapeViewSpeed::Ips15;

    let ascii = view.render_ascii(80, 24);
    assert!(ascii.contains("ANALOG TAPE SATURATION & HYSTERESIS HUD"));
    assert!(ascii.contains("SPEED: 15 IPS"));

    let render_path = "scratch/renders/tape_saturation.png";
    let res = view.render_snapshot_png(render_path, 800, 520);
    let _ = view.render_snapshot_png("../../scratch/renders/tape_saturation.png", 800, 520);
    assert!(res.is_ok(), "Tape saturation snapshot render must succeed: {:?}", res);

    let mut hasher = Hasher::new();
    hasher.update(ascii.as_bytes());
    hasher.update(&view.drive.to_le_bytes());
    hasher.update(&view.saturation.to_le_bytes());
    hasher.update(&view.bias.to_le_bytes());
    hasher.update(&view.hysteresis.to_le_bytes());

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_tape_saturation_view.hash", &digest);
}
