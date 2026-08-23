// Summoner DAW - End-to-End Stress & Fuzz Testing Suite
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Continuous fuzzing of topological graph mutations, MPE burst streams,
//! non-standard audio buffer chunk sizes, and zero-allocation enforcement under AllocGuard.

use summoner_core::allocator::AllocGuard;
use summoner_core::graph::NodeGraph;
use summoner_core::mpe::{ExpressionCurveType, MpeEvent, MpeExpressionCurveEditor, MpeRouter};
use summoner_core::node::{AudioNode, GainNode, ProcessContext, SineOscillatorNode};
use summoner_core::transport::Transport;

/// PRNG helper for deterministic, reproducible fuzz testing without external dependencies.
struct FuzzRng {
    state: u64,
}

impl FuzzRng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x853c49e6748fea9b } else { seed },
        }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.state >> 32) as u32
    }

    fn next_f32(&mut self) -> f32 {
        (self.next_u32() as f32) / (u32::MAX as f32)
    }

    fn next_range(&mut self, min: usize, max: usize) -> usize {
        if min >= max {
            return min;
        }
        min + (self.next_u32() as usize % (max - min))
    }
}

#[test]
fn test_fuzz_graph_topological_mutations_and_realtime_safety() {
    let mut rng = FuzzRng::new(0xDEADBEEF_CAFE1337);
    let sample_rate = 48000;
    let max_block_size = 512;
    let mut graph = NodeGraph::new("FuzzTopologyGraph", max_block_size, 2);

    // Populate graph with diverse nodes
    let mut node_indices = Vec::new();
    for i in 0..16 {
        let freq = 100.0 + (i as f32 * 50.0);
        let node_idx = graph.add_node(Box::new(SineOscillatorNode::new(freq)));
        node_indices.push(node_idx);
    }
    for _ in 0..8 {
        let gain_val = 0.2 + (rng.next_f32() * 0.6);
        let node_idx = graph.add_node(Box::new(GainNode::new(gain_val)));
        node_indices.push(node_idx);
    }

    let in_l = vec![0.0f32; max_block_size];
    let in_r = vec![0.0f32; max_block_size];
    let mut out_l = vec![0.0f32; max_block_size];
    let mut out_r = vec![0.0f32; max_block_size];

    let transport = Transport::new(sample_rate, 120.0);
    let ctx = ProcessContext::from_transport(&transport);

    // Fuzz 100 topological mutation rounds
    for round in 0..100 {
        // Randomly mutate connections
        let from_idx = node_indices[rng.next_range(0, 16)];
        let to_idx = node_indices[rng.next_range(16, node_indices.len())];

        if rng.next_f32() > 0.4 {
            let _ = graph.connect(from_idx, 0, to_idx, 0);
        } else {
            let _ = graph.disconnect(from_idx, 0, to_idx, 0);
        }

        // Compile and hot-swap schedule
        let schedule = graph.compile_schedule();
        assert!(
            !schedule.has_cycle,
            "Topological mutations must maintain acyclic DAG or handle cycles"
        );
        graph.hot_swap_schedule(schedule);

        // Process audio across varied block sizes
        let chunk_sizes = [1, 7, 16, 33, 64, 127, 256, 512];
        let block_len = chunk_sizes[round % chunk_sizes.len()];

        let inputs: [&[f32]; 2] = [&in_l[..block_len], &in_r[..block_len]];
        let mut outputs: [&mut [f32]; 2] = [&mut out_l[..block_len], &mut out_r[..block_len]];

        // Real-Time Safety Verification: AllocGuard asserts 0 heap allocations in audio loop
        {
            let _guard = AllocGuard::new();
            graph.process(&inputs, &mut outputs, &ctx);
        }

        // Verify zero NaN propagations, finiteness, and bounds
        for output_ch in &outputs {
            for &sample in &output_ch[..block_len] {
                assert!(
                    sample.is_finite(),
                    "Output sample must be finite during fuzzing"
                );
                assert!(!sample.is_nan(), "Output sample must not be NaN");
                assert!(
                    (-10.0..=10.0).contains(&sample),
                    "Output sample bounded within reasonable DSP limits"
                );
            }
        }
    }
}

#[test]
fn test_fuzz_mpe_burst_streams_and_curve_mapping() {
    let mut rng = FuzzRng::new(0x12345678_9ABCDEF0);
    let mut router = MpeRouter::new();
    let curve_editor = MpeExpressionCurveEditor::new(48.0);

    let curve_types = [
        ExpressionCurveType::Linear,
        ExpressionCurveType::Logarithmic,
        ExpressionCurveType::Exponential,
        ExpressionCurveType::Sigmoid,
    ];

    // Fuzz 5000 MPE event bursts with AllocGuard
    for burst in 0..5000 {
        let voice_id = (burst % 16) as u32 + 1;
        let channel = (rng.next_range(1, 16)) as u8;
        let note = 24.0 + (rng.next_f32() * 84.0);
        let velocity = rng.next_f32();
        let bend_raw = (rng.next_u32() % 16384) as i16 - 8192;
        let pressure_raw = rng.next_f32();
        let timbre_raw = rng.next_f32();

        let bend_semitones = curve_editor.map_pitch_bend(bend_raw);
        assert!(bend_semitones.is_finite());
        assert!(!bend_semitones.is_nan());
        assert!((-48.0..=48.0).contains(&bend_semitones));

        let curve_type = curve_types[burst % curve_types.len()];
        let mapped_pressure = curve_editor.map_expression_value(pressure_raw, curve_type);
        assert!(mapped_pressure.is_finite());
        assert!(!mapped_pressure.is_nan());
        assert!((0.0..=1.0).contains(&mapped_pressure));

        let events = [
            MpeEvent::NoteOn {
                voice_id,
                channel,
                note,
                velocity,
            },
            MpeEvent::PitchBend {
                voice_id,
                semitones: bend_semitones,
            },
            MpeEvent::Pressure {
                voice_id,
                pressure: mapped_pressure,
            },
            MpeEvent::Timbre {
                voice_id,
                timbre: timbre_raw,
            },
            MpeEvent::NoteOff {
                voice_id,
                channel,
                release_velocity: 0.5,
            },
        ];

        // Dispatch burst under AllocGuard
        {
            let _guard = AllocGuard::new();
            for ev in &events {
                router.dispatch(ev);
            }
        }
    }

    // Verify voice state invariants
    for voice in &router.voices {
        let eff_note = voice.effective_note();
        assert!(eff_note.is_finite());
        assert!(!eff_note.is_nan());
    }
}

#[test]
fn test_fuzz_non_standard_buffer_chunk_sizes_and_anti_pop() {
    let mut rng = FuzzRng::new(0xFEEDBEEF_01234567);
    let sample_rate = 44100;
    let max_buf = 2048;
    let mut graph = NodeGraph::new("NonStandardBufferGraph", max_buf, 2);

    let osc_idx = graph.add_node(Box::new(SineOscillatorNode::new(440.0)));
    let gain_idx = graph.add_node(Box::new(GainNode::new(0.5)));
    let _ = graph.connect(osc_idx, 0, gain_idx, 0);

    let schedule = graph.compile_schedule();
    graph.hot_swap_schedule(schedule);

    let in_l = vec![0.0f32; max_buf];
    let in_r = vec![0.0f32; max_buf];
    let mut out_l = vec![0.0f32; max_buf];
    let mut out_r = vec![0.0f32; max_buf];

    let mut transport = Transport::new(sample_rate, 120.0);
    transport.play();

    // Arbitrary non-power-of-2 and prime buffer sizes
    let chunk_sizes = [
        1, 2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83,
        89, 97, 113, 127, 131, 137, 139, 149, 151, 157, 163, 167, 173, 179, 181, 191, 193, 197,
        199, 211, 223, 227, 229, 233, 239, 241, 251, 257, 384, 513, 769, 1021, 1024, 1536,
    ];

    for (i, &chunk_len) in chunk_sizes.iter().enumerate() {
        let ctx = ProcessContext::from_transport(&transport);
        let inputs: [&[f32]; 2] = [&in_l[..chunk_len], &in_r[..chunk_len]];
        let mut outputs: [&mut [f32]; 2] = [&mut out_l[..chunk_len], &mut out_r[..chunk_len]];

        // Trigger occasional schedule hot swap to test anti-pop crossfade across odd chunk boundaries
        if i % 10 == 0 {
            let gain = 0.1 + (rng.next_f32() * 0.8);
            let new_gain_idx = graph.add_node(Box::new(GainNode::new(gain)));
            let _ = graph.connect(osc_idx, 0, new_gain_idx, 0);
            let new_sched = graph.compile_schedule();
            graph.hot_swap_schedule(new_sched);
        }

        // Process under AllocGuard
        {
            let _guard = AllocGuard::new();
            graph.process(&inputs, &mut outputs, &ctx);
        }

        for output_ch in &outputs {
            for &sample in &output_ch[..chunk_len] {
                assert!(sample.is_finite(), "Sample must be finite");
                assert!(!sample.is_nan(), "Sample must not be NaN");
                assert!((-2.0..=2.0).contains(&sample), "Sample bounded");
            }
        }

        transport.advance_frames(chunk_len as u64);
    }
}
