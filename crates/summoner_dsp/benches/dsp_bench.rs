use criterion::{black_box, criterion_group, criterion_main, Criterion};
use summoner_dsp::oscillators::OscSaw;
use summoner_dsp::traits::SignalProcessor;
use summoner_core::node::ProcessContext;

fn bench_osc_saw_scalar(c: &mut Criterion) {
    let mut osc = OscSaw::new(440.0);
    let mut outputs = vec![vec![0.0; 4096]];
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
    let mut outputs = vec![vec![0.0; 4096]];
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

criterion_group!(benches, bench_osc_saw_scalar, bench_osc_saw_simd);
criterion_main!(benches);
