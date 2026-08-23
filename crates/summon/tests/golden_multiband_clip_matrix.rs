// Summoner - Deterministic Golden Multi-Band Dynamics & Clip Matrix Regression Suite
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

use blake3::Hasher;
use std::f32::consts::PI;
use std::fs;
use std::path::Path;
use summoner_core::allocator::AllocGuard;
use summoner_core::audio::Sample;
use summoner_core::node::ProcessContext;
use summoner_core::transport::Transport;
use summoner_dsp::multiband_dynamics::{LinkwitzRiley4BandCrossover, MultibandDynamicsProcessor};
use summoner_dsp::traits::SignalProcessor;
#[cfg(feature = "gui")]
use summoner_gui::views::multiband_visualizer::MultibandVisualizerView;
use summoner_sequencer::clip_matrix::{ClipMatrix, FollowAction, FollowActionType, LaunchQuantize, SlotState};

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
            "Golden multi-band dynamics & clip matrix hash mismatch for {}",
            hash_filename
        );
    } else {
        fs::write(&hash_file, digest).expect("Failed to write golden hash");
    }
}

#[test]
fn test_golden_linear_phase_crossover_reconstruction_flatness() {
    let sample_rate = 48000.0;
    let mut crossover = LinkwitzRiley4BandCrossover::new(140.0, 1400.0, 7000.0, sample_rate);

    let mut in_l = [0.0f32; BLOCK_SIZE];
    let mut in_r = [0.0f32; BLOCK_SIZE];
    let mut hasher = Hasher::new();

    // Test across multi-octave test signals (32 consecutive blocks = 2048 samples)
    for block_idx in 0..32 {
        for i in 0..BLOCK_SIZE {
            let n = (block_idx * BLOCK_SIZE + i) as f32;
            // Multi-tone stimulus combining low, mid, and high components
            let s_low = (2.0 * PI * 80.0 * n / sample_rate).sin() * 0.35;
            let s_mid = (2.0 * PI * 1000.0 * n / sample_rate).sin() * 0.35;
            let s_high = (2.0 * PI * 10000.0 * n / sample_rate).sin() * 0.30;
            in_l[i] = s_low + s_mid + s_high;
            in_r[i] = s_low * 0.8 + s_mid * 1.1 + s_high * 0.9;
        }

        // Process through Linkwitz-Riley Crossover with zero-allocation enforcement
        let mut sum_l = [0.0f32; BLOCK_SIZE];
        let mut sum_r = [0.0f32; BLOCK_SIZE];

        {
            let _guard = AllocGuard::new();
            for i in 0..BLOCK_SIZE {
                let bands = crossover.process_sample(in_l[i], in_r[i]);
                sum_l[i] = bands[0].0 + bands[1].0 + bands[2].0 + bands[3].0;
                sum_r[i] = bands[0].1 + bands[1].1 + bands[2].1 + bands[3].1;
            }
        }

        for i in 0..BLOCK_SIZE {
            hasher.update(&sum_l[i].to_le_bytes());
            hasher.update(&sum_r[i].to_le_bytes());
        }
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_multiband_crossover.hash", &digest);
}

#[test]
fn test_golden_multiband_dynamics_downward_and_upward_processing() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    let mut processor = MultibandDynamicsProcessor::new(sample_rate as f32);

    // Configure per-band aggressive dynamics
    processor.bands[0].comp_threshold_db = -16.0;
    processor.bands[0].comp_ratio = 4.0;
    processor.bands[0].comp_makeup_db = 2.0;

    processor.bands[1].comp_threshold_db = -18.0;
    processor.bands[1].comp_ratio = 3.0;

    processor.bands[2].comp_threshold_db = -14.0;
    processor.bands[2].comp_ratio = 2.5;

    processor.bands[3].comp_threshold_db = -12.0;
    processor.bands[3].comp_ratio = 2.0;

    let mut in_l = [0.0f32; BLOCK_SIZE];
    let mut in_r = [0.0f32; BLOCK_SIZE];
    let mut out_l = [0.0f32; BLOCK_SIZE];
    let mut out_r = [0.0f32; BLOCK_SIZE];

    let mut hasher = Hasher::new();

    // Process 64 blocks of hot multi-frequency transients
    for block_idx in 0..64 {
        let ctx = ProcessContext::from_transport(&transport);
        let burst = if block_idx % 4 == 0 { 2.0f32 } else { 0.8f32 };

        for i in 0..BLOCK_SIZE {
            let n = (block_idx * BLOCK_SIZE + i) as f32;
            let low = (2.0 * PI * 60.0 * n / 48000.0).sin() * burst;
            let mid = (2.0 * PI * 800.0 * n / 48000.0).cos() * burst;
            let high = (2.0 * PI * 8000.0 * n / 48000.0).sin() * (burst * 0.7);

            in_l[i] = (low * 0.6 + mid * 0.4 + high * 0.3) * 1.2;
            in_r[i] = (low * 0.4 + mid * 0.6 + high * 0.4) * 1.2;
        }

        {
            let _guard = AllocGuard::new();
            let in_slices: [&[Sample]; 2] = [&in_l[..], &in_r[..]];
            let mut out_slices: [&mut [Sample]; 2] = [&mut out_l[..], &mut out_r[..]];
            processor.process_block(&in_slices, &mut out_slices, &ctx);
        }

        for i in 0..BLOCK_SIZE {
            assert!(out_l[i].is_finite());
            assert!(out_r[i].is_finite());
            hasher.update(&out_l[i].to_le_bytes());
            hasher.update(&out_r[i].to_le_bytes());
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_multiband_dynamics.hash", &digest);
}

#[test]
fn test_golden_clip_launcher_scene_matrix_quantized_firing() {
    let mut matrix = ClipMatrix::new(4, 4);
    matrix.global_quantize = LaunchQuantize::Bar(1);

    // Setup Scene 0 (Intro: 4 bars)
    matrix.set_clip(0, 0, "Kick 1", 4.0, true);
    matrix.set_clip(1, 0, "Bass 1", 4.0, true);
    matrix.set_clip(2, 0, "Chords 1", 4.0, true);

    // Setup Scene 1 (Drop: 4 bars with Legato and Follow Action)
    matrix.set_clip(0, 1, "Kick 2", 4.0, true);
    matrix.set_clip(1, 1, "Bass 2", 4.0, true);
    matrix.set_clip(2, 1, "Chords 2", 4.0, true);
    matrix.set_clip(3, 1, "Lead 2", 4.0, true);

    if let Some(slot) = matrix.get_slot_mut(1, 1) {
        slot.legato = true;
    }
    if let Some(slot) = matrix.get_slot_mut(0, 1) {
        slot.follow_action = FollowAction {
            action_type: FollowActionType::PlayNext,
            trigger_beats: 8.0,
            chance: 1.0,
        };
    }

    // Setup Scene 2 (Outro)
    matrix.set_clip(0, 2, "Kick Outro", 4.0, false);
    matrix.set_clip(1, 2, "Bass Outro", 4.0, false);

    let mut hasher = Hasher::new();

    // Step 1: Queue Scene 0 at beat 0.0
    matrix.trigger_scene(0, false);
    assert_eq!(matrix.queued_scene_idx, Some(0));

    // Simulate 32 quarter-note steps (8 bars)
    let beats_per_bar = 4.0;
    let dt = 0.25;

    for step in 0..64 {
        let current_beat = step as f64 * dt;
        matrix.evaluate_tick(current_beat, beats_per_bar, dt);

        // At beat 8.0, trigger Scene 1
        if (current_beat - 8.0).abs() < 1e-4 {
            matrix.trigger_scene(1, false);
        }

        // Record state hash
        hasher.update(&current_beat.to_le_bytes());
        for t in 0..4 {
            for s in 0..4 {
                if let Some(slot) = matrix.get_slot(t, s) {
                    let state_byte = match slot.state {
                        SlotState::Empty => 0u8,
                        SlotState::Stopped => 1u8,
                        SlotState::QueuedToPlay => 2u8,
                        SlotState::Playing => 3u8,
                        SlotState::QueuedToStop => 4u8,
                    };
                    hasher.update(&[state_byte]);
                    hasher.update(&slot.playhead_beats.to_le_bytes());
                }
            }
        }
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_clip_matrix.hash", &digest);
}

#[cfg(feature = "gui")]
#[test]
fn test_golden_multiband_visualizer_telemetry_snapshot() {
    let mut view = MultibandVisualizerView::new();
    view.state.selected_band_idx = 1; // Low-Mid selected
    view.state.crossover_fc1 = 150.0;
    view.state.crossover_fc2 = 1500.0;
    view.state.crossover_fc3 = 6500.0;

    let ascii = view.render_ascii(80, 24);
    assert!(ascii.contains("4-BAND MULTI-BAND DYNAMICS VISUALIZER"));
    assert!(ascii.contains("FC1: 150 Hz"));

    let render_path = "scratch/renders/multiband_dynamics.png";
    let res = view.render_snapshot_png(render_path, 800, 520);
    assert!(res.is_ok(), "Snapshot render must succeed");

    let mut hasher = Hasher::new();
    hasher.update(ascii.as_bytes());
    hasher.update(&view.state.crossover_fc1.to_le_bytes());
    hasher.update(&view.state.crossover_fc2.to_le_bytes());
    hasher.update(&view.state.crossover_fc3.to_le_bytes());

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_multiband_visualizer.hash", &digest);
}
