// Summoner - Deterministic Golden Microtonal Scale Synthesis & Stress Benchmark Suite
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

use blake3::Hasher;
use std::fs;
use std::path::Path;
use summoner_core::allocator::AllocGuard;
use summoner_core::audio::FixedAudioBuffer;
use summoner_core::node::{AudioNode, GainNode, ProcessContext, SineOscillatorNode};
use summoner_core::transport::Transport;
use summoner_harmony::edo::EdoTuning;
use summoner_harmony::scl::SclTuning;

const CHANNELS: usize = 2;
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
            "Golden microtonal hash mismatch for {}",
            hash_filename
        );
    } else {
        fs::write(&hash_file, digest).expect("Failed to write golden hash");
    }
}

#[test]
fn test_golden_19_edo_synthesis() {
    let sample_rate = 44100;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    let edo19 = EdoTuning::new(19, 440.0, 69.0);
    let mut mid_buf = FixedAudioBuffer::<CHANNELS, BLOCK_SIZE>::new();
    let mut out_buf = FixedAudioBuffer::<CHANNELS, BLOCK_SIZE>::new();
    let mut hasher = Hasher::new();

    // Render a 19-step microtonal scale sequence
    for step in 0..19 {
        let note = 69.0 + (step as f64);
        let freq = edo19.note_to_freq(note) as f32;
        let mut sine = SineOscillatorNode::new(freq);
        let mut gain = GainNode::new(0.5);

        for _ in 0..8 {
            // 512 frames per step
            mid_buf.set_active_frames(BLOCK_SIZE);
            out_buf.set_active_frames(BLOCK_SIZE);
            mid_buf.clear();
            out_buf.clear();

            let ctx = ProcessContext::from_transport(&transport);
            let dummy_in: [&[summoner_core::audio::Sample]; 0] = [];

            // Enforce zero allocations during inner audio block synthesis
            {
                let _guard = AllocGuard::new();
                let mut mid_slices = mid_buf.channels_mut_2();
                sine.process(&dummy_in, &mut mid_slices, &ctx);

                let mid_ref = mid_buf.channels_ref_2();
                let mut out_slices = out_buf.channels_mut_2();
                gain.process(&mid_ref, &mut out_slices, &ctx);
            }

            for ch in 0..CHANNELS {
                for &s in out_buf.channel(ch) {
                    assert!(s.is_finite());
                    assert!(!s.is_nan());
                    assert!((-1.0..=1.0).contains(&s));
                    hasher.update(&s.to_le_bytes());
                }
            }

            transport.advance_frames(BLOCK_SIZE as u64);
        }
    }

    let digest = hasher.finalize().to_hex().to_string();
    assert!(!digest.is_empty());
    check_or_save_golden("microtonal_19edo.blake3", &digest);
}

#[test]
fn test_golden_31_edo_synthesis() {
    let sample_rate = 44100;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    let edo31 = EdoTuning::new(31, 440.0, 69.0);
    let mut mid_buf = FixedAudioBuffer::<CHANNELS, BLOCK_SIZE>::new();
    let mut out_buf = FixedAudioBuffer::<CHANNELS, BLOCK_SIZE>::new();
    let mut hasher = Hasher::new();

    // Render 31 steps of the quarter-comma meantone system approximation
    for step in 0..31 {
        let note = 69.0 + (step as f64);
        let freq = edo31.note_to_freq(note) as f32;
        let mut sine = SineOscillatorNode::new(freq);
        let mut gain = GainNode::new(0.5);

        for _ in 0..8 {
            mid_buf.set_active_frames(BLOCK_SIZE);
            out_buf.set_active_frames(BLOCK_SIZE);
            mid_buf.clear();
            out_buf.clear();

            let ctx = ProcessContext::from_transport(&transport);
            let dummy_in: [&[summoner_core::audio::Sample]; 0] = [];

            {
                let _guard = AllocGuard::new();
                let mut mid_slices = mid_buf.channels_mut_2();
                sine.process(&dummy_in, &mut mid_slices, &ctx);

                let mid_ref = mid_buf.channels_ref_2();
                let mut out_slices = out_buf.channels_mut_2();
                gain.process(&mid_ref, &mut out_slices, &ctx);
            }

            for ch in 0..CHANNELS {
                for &s in out_buf.channel(ch) {
                    assert!(s.is_finite());
                    assert!(!s.is_nan());
                    assert!((-1.0..=1.0).contains(&s));
                    hasher.update(&s.to_le_bytes());
                }
            }

            transport.advance_frames(BLOCK_SIZE as u64);
        }
    }

    let digest = hasher.finalize().to_hex().to_string();
    assert!(!digest.is_empty());
    check_or_save_golden("microtonal_31edo.blake3", &digest);
}

#[test]
fn test_golden_53_edo_synthesis() {
    let sample_rate = 44100;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    let edo53 = EdoTuning::new(53, 440.0, 69.0);
    let mut mid_buf = FixedAudioBuffer::<CHANNELS, BLOCK_SIZE>::new();
    let mut out_buf = FixedAudioBuffer::<CHANNELS, BLOCK_SIZE>::new();
    let mut hasher = Hasher::new();

    // Render 53 Pythagorean commas / Turkish maqam steps
    for step in 0..53 {
        let note = 69.0 + (step as f64);
        let freq = edo53.note_to_freq(note) as f32;
        let mut sine = SineOscillatorNode::new(freq);
        let mut gain = GainNode::new(0.5);

        for _ in 0..4 {
            mid_buf.set_active_frames(BLOCK_SIZE);
            out_buf.set_active_frames(BLOCK_SIZE);
            mid_buf.clear();
            out_buf.clear();

            let ctx = ProcessContext::from_transport(&transport);
            let dummy_in: [&[summoner_core::audio::Sample]; 0] = [];

            {
                let _guard = AllocGuard::new();
                let mut mid_slices = mid_buf.channels_mut_2();
                sine.process(&dummy_in, &mut mid_slices, &ctx);

                let mid_ref = mid_buf.channels_ref_2();
                let mut out_slices = out_buf.channels_mut_2();
                gain.process(&mid_ref, &mut out_slices, &ctx);
            }

            for ch in 0..CHANNELS {
                for &s in out_buf.channel(ch) {
                    assert!(s.is_finite());
                    assert!(!s.is_nan());
                    assert!((-1.0..=1.0).contains(&s));
                    hasher.update(&s.to_le_bytes());
                }
            }

            transport.advance_frames(BLOCK_SIZE as u64);
        }
    }

    let digest = hasher.finalize().to_hex().to_string();
    assert!(!digest.is_empty());
    check_or_save_golden("microtonal_53edo.blake3", &digest);
}

#[test]
fn test_golden_bohlen_pierce_scala_synthesis() {
    let sample_rate = 44100;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    // Bohlen-Pierce 13-tone equal temperament tritave scale
    let scl_content = "! bohlen_pierce.scl\nBohlen-Pierce 13-tone scale\n13\n!\n146.304\n292.608\n438.913\n585.217\n731.521\n877.826\n1024.130\n1170.434\n1316.739\n1463.043\n1609.347\n1755.651\n3/1\n";
    let tuning = SclTuning::parse(scl_content).expect("Failed to parse Bohlen-Pierce SCL");
    assert_eq!(tuning.num_notes, 13);

    let base_freq = 220.0f64;
    let mut mid_buf = FixedAudioBuffer::<CHANNELS, BLOCK_SIZE>::new();
    let mut out_buf = FixedAudioBuffer::<CHANNELS, BLOCK_SIZE>::new();
    let mut hasher = Hasher::new();

    for &cents in &tuning.cents_or_ratios {
        let freq = (base_freq * 2.0_f64.powf(cents / 1200.0)) as f32;
        let mut sine = SineOscillatorNode::new(freq);
        let mut gain = GainNode::new(0.5);

        for _ in 0..8 {
            mid_buf.set_active_frames(BLOCK_SIZE);
            out_buf.set_active_frames(BLOCK_SIZE);
            mid_buf.clear();
            out_buf.clear();

            let ctx = ProcessContext::from_transport(&transport);
            let dummy_in: [&[summoner_core::audio::Sample]; 0] = [];

            {
                let _guard = AllocGuard::new();
                let mut mid_slices = mid_buf.channels_mut_2();
                sine.process(&dummy_in, &mut mid_slices, &ctx);

                let mid_ref = mid_buf.channels_ref_2();
                let mut out_slices = out_buf.channels_mut_2();
                gain.process(&mid_ref, &mut out_slices, &ctx);
            }

            for ch in 0..CHANNELS {
                for &s in out_buf.channel(ch) {
                    assert!(s.is_finite());
                    assert!(!s.is_nan());
                    assert!((-1.0..=1.0).contains(&s));
                    hasher.update(&s.to_le_bytes());
                }
            }

            transport.advance_frames(BLOCK_SIZE as u64);
        }
    }

    let digest = hasher.finalize().to_hex().to_string();
    assert!(!digest.is_empty());
    check_or_save_golden("microtonal_bohlen_pierce.blake3", &digest);
}

#[test]
fn test_golden_slendro_scala_synthesis() {
    let sample_rate = 44100;
    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    // Javanese Slendro 5-tone equidistant pentatonic system
    let scl_content = "! slendro.scl\nJavanese Slendro 5-tone scale\n5\n!\n240.0\n480.0\n720.0\n960.0\n1200.0\n";
    let tuning = SclTuning::parse(scl_content).expect("Failed to parse Slendro SCL");
    assert_eq!(tuning.num_notes, 5);

    let base_freq = 261.63f64; // Middle C
    let mut mid_buf = FixedAudioBuffer::<CHANNELS, BLOCK_SIZE>::new();
    let mut out_buf = FixedAudioBuffer::<CHANNELS, BLOCK_SIZE>::new();
    let mut hasher = Hasher::new();

    for &cents in &tuning.cents_or_ratios {
        let freq = (base_freq * 2.0_f64.powf(cents / 1200.0)) as f32;
        let mut sine = SineOscillatorNode::new(freq);
        let mut gain = GainNode::new(0.5);

        for _ in 0..16 {
            mid_buf.set_active_frames(BLOCK_SIZE);
            out_buf.set_active_frames(BLOCK_SIZE);
            mid_buf.clear();
            out_buf.clear();

            let ctx = ProcessContext::from_transport(&transport);
            let dummy_in: [&[summoner_core::audio::Sample]; 0] = [];

            {
                let _guard = AllocGuard::new();
                let mut mid_slices = mid_buf.channels_mut_2();
                sine.process(&dummy_in, &mut mid_slices, &ctx);

                let mid_ref = mid_buf.channels_ref_2();
                let mut out_slices = out_buf.channels_mut_2();
                gain.process(&mid_ref, &mut out_slices, &ctx);
            }

            for ch in 0..CHANNELS {
                for &s in out_buf.channel(ch) {
                    assert!(s.is_finite());
                    assert!(!s.is_nan());
                    assert!((-1.0..=1.0).contains(&s));
                    hasher.update(&s.to_le_bytes());
                }
            }

            transport.advance_frames(BLOCK_SIZE as u64);
        }
    }

    let digest = hasher.finalize().to_hex().to_string();
    assert!(!digest.is_empty());
    check_or_save_golden("microtonal_scala_slendro.blake3", &digest);
}

#[test]
fn test_microtonal_polyphonic_stress_and_alloc_guard() {
    let sample_rate = 48000;
    let mut transport = Transport::new(sample_rate, 140.0);
    transport.play();

    let edo53 = EdoTuning::new(53, 440.0, 69.0);
    const VOICES: usize = 32;

    // Create 32 polyphonic oscillator voices across 53-EDO microtonal intervals
    let mut oscs: Vec<SineOscillatorNode> = (0..VOICES)
        .map(|i| {
            let note = 50.0 + (i as f64 * (53.0 / 32.0));
            let freq = edo53.note_to_freq(note) as f32;
            SineOscillatorNode::new(freq)
        })
        .collect();

    let mut gain = GainNode::new(0.03); // Scaled for 32 voices to avoid clipping
    let mut voice_buf = FixedAudioBuffer::<CHANNELS, BLOCK_SIZE>::new();
    let mut mix_buf = FixedAudioBuffer::<CHANNELS, BLOCK_SIZE>::new();
    let mut out_buf = FixedAudioBuffer::<CHANNELS, BLOCK_SIZE>::new();

    // Stress test 128 audio blocks (8192 frames) under AllocGuard
    for _ in 0..128 {
        mix_buf.set_active_frames(BLOCK_SIZE);
        out_buf.set_active_frames(BLOCK_SIZE);
        mix_buf.clear();
        out_buf.clear();

        let ctx = ProcessContext::from_transport(&transport);
        let dummy_in: [&[summoner_core::audio::Sample]; 0] = [];

        {
            let _guard = AllocGuard::new();

            for osc in &mut oscs {
                voice_buf.set_active_frames(BLOCK_SIZE);
                voice_buf.clear();
                let mut v_slices = voice_buf.channels_mut_2();
                osc.process(&dummy_in, &mut v_slices, &ctx);

                // Accumulate into mix buffer
                for ch in 0..CHANNELS {
                    for f in 0..BLOCK_SIZE {
                        mix_buf.channel_mut(ch)[f] += voice_buf.channel(ch)[f];
                    }
                }
            }

            let mix_ref = mix_buf.channels_ref_2();
            let mut out_slices = out_buf.channels_mut_2();
            gain.process(&mix_ref, &mut out_slices, &ctx);
        }

        for ch in 0..CHANNELS {
            for &s in out_buf.channel(ch) {
                assert!(s.is_finite(), "Microtonal polyphonic sample must be finite");
                assert!(!s.is_nan(), "Sample must not be NaN");
                assert!(
                    (-1.5..=1.5).contains(&s),
                    "Polyphonic mix output must remain bounded: {}",
                    s
                );
            }
        }

        transport.advance_frames(BLOCK_SIZE as u64);
    }
}
