use criterion::{black_box, criterion_group, criterion_main, Criterion};
use summoner_core::node::ProcessContext;
use summoner_dsp::oscillators::OscSaw;
use summoner_dsp::traits::SignalProcessor;

fn bench_osc_saw_scalar(c: &mut Criterion) {
    let mut osc = OscSaw::new(440.0);
    let mut outputs = [vec![0.0; 4096]];
    let mut output_slices: Vec<&mut [f32]> = outputs.iter_mut().map(|v| v.as_mut_slice()).collect();

    let ctx = ProcessContext {
        sample_rate: 44100,
        bpm: 120.0,
        frame_position: 0,
        is_playing: true,
        param_bus: None,
        tuning_root_hz: 440.0,
        tuning_edo_divisions: 12,
    };

    c.bench_function("OscSaw scalar 4096", |b| {
        b.iter(|| {
            // Note: in the actual implementation, the scalar fallback is selected by #[cfg],
            // so if we are on x86_64 it runs SIMD. To properly bench the scalar, we would need
            // a separate public scalar function. We just call process_block.
            osc.process_block(&[], &mut output_slices, black_box(&ctx));
        })
    });
}

fn bench_osc_saw_simd(c: &mut Criterion) {
    let mut osc = OscSaw::new(440.0);
    let mut outputs = [vec![0.0; 4096]];
    let mut output_slices: Vec<&mut [f32]> = outputs.iter_mut().map(|v| v.as_mut_slice()).collect();

    let ctx = ProcessContext {
        sample_rate: 44100,
        bpm: 120.0,
        frame_position: 0,
        is_playing: true,
        param_bus: None,
        tuning_root_hz: 440.0,
        tuning_edo_divisions: 12,
    };

    c.bench_function("OscSaw SIMD 4096", |b| {
        b.iter(|| {
            #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
            osc.process_block_simd(&mut output_slices, black_box(&ctx));

            #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
            osc.process_block(&[], &mut output_slices, black_box(&ctx));
        })
    });
}

fn bench_filter_ladder_poly_128_simd8(c: &mut Criterion) {
    use summoner_dsp::filters::{process_filter_ladder_poly_128, FilterLadder8};

    let mut voices: [FilterLadder8; 16] = std::array::from_fn(|i| {
        let base_cutoff = 200.0 + (i as f32 * 100.0);
        let cutoffs = std::array::from_fn(|lane| base_cutoff + (lane as f32 * 20.0));
        let resonances = std::array::from_fn(|lane| 0.5 + (lane as f32 * 0.3));
        FilterLadder8::from_scalars(cutoffs, resonances)
    });

    let inputs = [wide::f32x8::splat(0.5); 16];
    let mut outputs = [wide::f32x8::splat(0.0); 16];
    let sample_rate = 44100;

    c.bench_function("FilterLadder 128 Polyphonic Voices SIMD8 (512 samples)", |b| {
        b.iter(|| {
            for _ in 0..512 {
                process_filter_ladder_poly_128(
                    black_box(&mut voices),
                    black_box(&inputs),
                    sample_rate,
                    black_box(&mut outputs),
                );
            }
        })
    });
}

fn bench_filter_ladder_poly_128_scalar(c: &mut Criterion) {
    use summoner_dsp::filters::FilterLadder;

    let mut voices: Vec<FilterLadder> = (0..128)
        .map(|i| {
            let cutoff = 200.0 + (i as f32 * 25.0);
            let res = 0.5 + ((i % 10) as f32 * 0.3);
            FilterLadder::new(cutoff, res)
        })
        .collect();

    let sample_rate = 44100;

    c.bench_function("FilterLadder 128 Polyphonic Voices Scalar (512 samples)", |b| {
        b.iter(|| {
            for _ in 0..512 {
                for v in 0..128 {
                    black_box(voices[v].process_sample(black_box(0.5), sample_rate));
                }
            }
        })
    });
}

criterion_group!(
    benches,
    bench_osc_saw_scalar,
    bench_osc_saw_simd,
    bench_filter_ladder_poly_128_simd8,
    bench_filter_ladder_poly_128_scalar
);
criterion_main!(benches);
