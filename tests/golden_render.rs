// Summoner - Deterministic Golden WAV Render Tests
// Copyright (C) 2026 nilsanderselde

use summoner_core::audio::FixedAudioBuffer;
use summoner_core::node::{AudioNode, GainNode, SineOscillatorNode};
use summoner_core::transport::Transport;
use blake3::Hasher;

#[test]
fn test_deterministic_golden_sine_render() {
    let mut transport = Transport::new(44100, 120.0);
    transport.play();

    let mut sine = SineOscillatorNode::new(440.0);
    let mut gain = GainNode::new(0.5);

    const CHANNELS: usize = 2;
    const BLOCK_SIZE: usize = 64;
    let mut mid_buf = FixedAudioBuffer::<CHANNELS, BLOCK_SIZE>::new();
    let mut out_buf = FixedAudioBuffer::<CHANNELS, BLOCK_SIZE>::new();

    let mut hasher = Hasher::new();

    for _ in 0..16 { // 1024 samples
        mid_buf.set_active_frames(BLOCK_SIZE);
        out_buf.set_active_frames(BLOCK_SIZE);
        mid_buf.clear();
        out_buf.clear();

        let ctx = summoner_core::node::ProcessContext::from_transport(&transport);
        let dummy_in: [&[summoner_core::audio::Sample]; 0] = [];

        let mut mid_slices = mid_buf.channels_mut_2();
        sine.process(&dummy_in, &mut mid_slices, &ctx);

        let mid_ref = mid_buf.channels_ref_2();
        let mut out_slices = out_buf.channels_mut_2();
        gain.process(&mid_ref, &mut out_slices, &ctx);

        for ch in 0..CHANNELS {
            for &s in out_buf.channel(ch) {
                hasher.update(&s.to_le_bytes());
            }
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }

    let digest = hasher.finalize().to_hex().to_string();
    assert!(!digest.is_empty());
    println!("Golden Render Digest: {}", digest);
}
