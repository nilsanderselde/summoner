// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.

use summoner_core::allocator::AllocGuard;
use summoner_core::audio::{FixedAudioBuffer, Frame, Sample};
use summoner_core::node::{AudioNode, GainNode, ProcessContext, SineOscillatorNode};
use summoner_core::transport::Transport;

#[test]
fn test_alloc_guard_detects_allocations() {
    let result = std::panic::catch_unwind(|| {
        let _guard = AllocGuard::new();
        // Heap allocation while AllocGuard is active should trigger panic
        let _v: Vec<i32> = vec![1, 2, 3];
    });
    assert!(
        result.is_err(),
        "AllocGuard failed to catch heap allocation panic!"
    );
}

#[test]
fn test_zero_alloc_dsp_processing() {
    let mut transport = Transport::new(44100, 120.0);
    transport.play();
    let ctx = ProcessContext::from_transport(&transport);

    let mut sine = SineOscillatorNode::new(440.0);
    let mut gain = GainNode::new(0.5);

    let mut mid_buf = FixedAudioBuffer::<2, 256>::new();
    let mut out_buf = FixedAudioBuffer::<2, 256>::new();

    // Verify DSP processing completes inside AllocGuard without any heap allocations
    {
        let _guard = AllocGuard::new();

        let dummy_in: [&[Sample]; 0] = [];
        let mut mid_slices = mid_buf.channels_mut_2();

        sine.process(&dummy_in, &mut mid_slices, &ctx);

        let mid_ref = mid_buf.channels_ref_2();
        let mut out_slices = out_buf.channels_mut_2();

        gain.process(&mid_ref, &mut out_slices, &ctx);
    }

    let energy: f32 = out_buf.channel(0).iter().map(|s| s.abs()).sum();
    assert!(
        energy > 0.0,
        "Expected non-zero signal output from DSP chain"
    );
}

#[test]
fn test_deterministic_rendering_identical_output() {
    let render_run = || -> Vec<f32> {
        let mut transport = Transport::new(44100, 120.0);
        transport.play();

        let mut sine = SineOscillatorNode::new(440.0);
        let mut gain = GainNode::new(0.75);

        let mut mid_buf = FixedAudioBuffer::<2, 128>::new();
        let mut out_buf = FixedAudioBuffer::<2, 128>::new();
        let mut output_samples = Vec::with_capacity(1024);

        for _ in 0..8 {
            let ctx = ProcessContext::from_transport(&transport);
            mid_buf.clear();
            out_buf.clear();

            let dummy_in: [&[Sample]; 0] = [];
            let mut mid_slices = mid_buf.channels_mut_2();

            sine.process(&dummy_in, &mut mid_slices, &ctx);

            let mid_ref = mid_buf.channels_ref_2();
            let mut out_slices = out_buf.channels_mut_2();

            gain.process(&mid_ref, &mut out_slices, &ctx);

            output_samples.extend_from_slice(out_buf.channel(0));
            transport.advance_frames(128);
        }

        output_samples
    };

    let run1 = render_run();
    let run2 = render_run();

    assert_eq!(run1.len(), run2.len());
    for (i, (s1, s2)) in run1.iter().zip(run2.iter()).enumerate() {
        assert_eq!(s1.to_bits(), s2.to_bits(), "Mismatch at sample {}", i);
    }
}

#[test]
fn test_frame_operations() {
    let mut f1 = Frame::<2> {
        channels: [0.5, -0.5],
    };
    let f2 = Frame::<2> {
        channels: [0.25, 0.25],
    };

    f1.mix(&f2);
    assert_eq!(f1.channels, [0.75, -0.25]);

    f1.scale(2.0);
    assert_eq!(f1.channels, [1.5, -0.5]);
}
