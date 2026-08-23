// Summoner - Deterministic Golden True-Peak Limiter & Multitrack Looper Regression Suite
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

use blake3::Hasher;
use std::f32::consts::PI;
use std::fs;
use std::path::Path;
use summoner_core::allocator::AllocGuard;
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext, SineOscillatorNode};
use summoner_core::transport::Transport;
use summoner_dsp::meter::EbuR128LoudnessMeter;
use summoner_dsp::oscillators::OscSaw;
use summoner_dsp::traits::SignalProcessor;
use summoner_dsp::true_peak_limiter::{TruePeakDetector, TruePeakLimiter};
use summoner_gui::views::mastering_meter::{MasteringMeterView, MasteringStandard};
use summoner_sequencer::session_looper::{
    LooperCommand, LooperQuantize, LooperState, MultitrackSessionLooper,
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
            "Golden mastering & looper hash mismatch for {}",
            hash_filename
        );
    } else {
        fs::write(&hash_file, digest).expect("Failed to write golden hash");
    }
}

#[test]
fn test_golden_true_peak_isp_limiter_brickwall() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    let mut limiter = TruePeakLimiter::new(-0.2, -0.2, 1.0, 40.0, 64);
    let mut detector_out_l = TruePeakDetector::new();
    let mut detector_out_r = TruePeakDetector::new();

    let mut saw_osc = OscSaw::new(110.0);
    let mut sine_high = SineOscillatorNode::new(12000.0); // Near 1/4 Nyquist for high ISP

    let mut saw_buf = [0.0f32; BLOCK_SIZE];
    let mut sine_buf = [0.0f32; BLOCK_SIZE];
    let mut in_l = [0.0f32; BLOCK_SIZE];
    let mut in_r = [0.0f32; BLOCK_SIZE];
    let mut out_l = [0.0f32; BLOCK_SIZE];
    let mut out_r = [0.0f32; BLOCK_SIZE];

    let mut hasher = Hasher::new();

    // Process 64 consecutive audio blocks (4096 samples)
    for block_idx in 0..64 {
        let ctx = ProcessContext::from_transport(&transport);

        let dummy_in: [&[Sample]; 0] = [];
        let mut saw_slice = [&mut saw_buf[..]];
        saw_osc.process_block(&dummy_in, &mut saw_slice, &ctx);

        let mut sine_slice = [&mut sine_buf[..]];
        sine_high.process(&dummy_in, &mut sine_slice, &ctx);

        // Mix hot input signal (+4.5 dBFS transient spikes)
        for i in 0..BLOCK_SIZE {
            let burst = if block_idx % 4 == 0 { 2.2f32 } else { 1.1f32 };
            in_l[i] = (saw_buf[i] * 1.5 + sine_buf[i] * 0.8) * burst;
            in_r[i] = (saw_buf[i] * 0.9 + sine_buf[i] * 1.4) * burst;
        }

        // Process through True-Peak Limiter with strict zero-allocation enforcement
        {
            let _guard = AllocGuard::new();
            let in_slices: [&[Sample]; 2] = [&in_l[..], &in_r[..]];
            let mut out_slices: [&mut [Sample]; 2] = [&mut out_l[..], &mut out_r[..]];
            limiter.process_block(&in_slices, &mut out_slices, &ctx);
        }

        // Verify that limited output never violates the -0.2 dBTP ceiling
        let ceiling_lin = 10.0f32.powf(-0.2 / 20.0);
        for i in 0..BLOCK_SIZE {
            let (tp_l, _) = detector_out_l.process_sample(out_l[i]);
            let (tp_r, _) = detector_out_r.process_sample(out_r[i]);

            assert!(
                tp_l <= ceiling_lin + 1e-3,
                "ISP overshoot on left channel at block {}: {} > {}",
                block_idx,
                tp_l,
                ceiling_lin
            );
            assert!(
                tp_r <= ceiling_lin + 1e-3,
                "ISP overshoot on right channel at block {}: {} > {}",
                block_idx,
                tp_r,
                ceiling_lin
            );

            hasher.update(&out_l[i].to_le_bytes());
            hasher.update(&out_r[i].to_le_bytes());
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_true_peak_limiter.hash", &digest);
}

#[test]
fn test_golden_session_looper_multitrack_overdub() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    // 1 bar at 120 BPM = 2.0 seconds = 96,000 samples @ 48kHz
    let mut looper = MultitrackSessionLooper::new(2, 48000);

    // Setup Track 0 for 256-sample loop recording
    looper.tracks[0].quantize = LooperQuantize::None;
    looper.tracks[0].execute_command(LooperCommand::TriggerRecord);

    let mut in_l = [0.0f32; BLOCK_SIZE];
    let mut in_r = [0.0f32; BLOCK_SIZE];
    let mut out_l = [0.0f32; BLOCK_SIZE];
    let mut out_r = [0.0f32; BLOCK_SIZE];

    let mut hasher = Hasher::new();

    // Phase 1: Record 4 blocks (256 samples)
    for _ in 0..4 {
        for i in 0..BLOCK_SIZE {
            in_l[i] = 0.6f32;
            in_r[i] = 0.4f32;
        }
        looper.process_block(&in_l, &in_r, &mut out_l, &mut out_r, &transport);
        transport.advance_frames(BLOCK_SIZE as u64);
    }

    // Switch Track 0 to Play mode
    looper.tracks[0].execute_command(LooperCommand::TriggerPlay);
    assert_eq!(looper.tracks[0].state, LooperState::Playing);
    assert_eq!(looper.tracks[0].loop_length_samples, 256);

    // Phase 2: Overdub harmonic layer on top with feedback decay
    looper.tracks[0].feedback = 0.8;
    looper.tracks[0].execute_command(LooperCommand::TriggerOverdub);
    assert_eq!(looper.tracks[0].state, LooperState::Overdubbing);

    for block_idx in 0..4 {
        for i in 0..BLOCK_SIZE {
            let phase = (block_idx * BLOCK_SIZE + i) as f32 / 256.0 * 2.0 * PI;
            in_l[i] = phase.sin() * 0.3;
            in_r[i] = phase.cos() * 0.3;
        }
        {
            let _guard = AllocGuard::new();
            looper.process_block(&in_l, &in_r, &mut out_l, &mut out_r, &transport);
        }
        for i in 0..BLOCK_SIZE {
            hasher.update(&out_l[i].to_le_bytes());
            hasher.update(&out_r[i].to_le_bytes());
        }
        transport.advance_frames(BLOCK_SIZE as u64);
    }

    // Phase 3: Verify Undo restores pre-overdub layer
    assert!(looper.tracks[0].undo());
    assert_eq!(looper.tracks[0].buffer_l[100], 0.6);
    assert_eq!(looper.tracks[0].buffer_r[100], 0.4);

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_session_looper.hash", &digest);
}

#[test]
fn test_golden_mastering_radar_and_true_peak_meter() {
    let mut meter = EbuR128LoudnessMeter::new(-23.0);
    let mut view = MasteringMeterView::new();
    view.standard = MasteringStandard::EbuR128Broadcast;

    let mut samples = [0.0f32; 1024];
    for (i, s) in samples.iter_mut().enumerate() {
        let phase = (i as f32 / 1024.0) * 8.0 * PI;
        *s = phase.sin() * 0.5;
    }

    meter.process_block(&samples);
    assert!(meter.momentary_lufs < 0.0);

    view.state.momentary_lufs = meter.momentary_lufs;
    view.state.short_term_lufs = meter.short_term_lufs;
    view.state.integrated_lufs = meter.integrated_lufs;

    let ascii = view.render_ascii(80, 24);
    assert!(ascii.contains("EBU R128 (-23 LUFS)"));

    let mut hasher = Hasher::new();
    hasher.update(&view.state.momentary_lufs.to_le_bytes());
    hasher.update(&view.state.short_term_lufs.to_le_bytes());
    hasher.update(&view.state.integrated_lufs.to_le_bytes());
    hasher.update(&view.standard.target_integrated_lufs().to_le_bytes());

    let digest = hasher.finalize().to_hex().to_string();
    check_or_save_golden("golden_mastering_meter.hash", &digest);
}
