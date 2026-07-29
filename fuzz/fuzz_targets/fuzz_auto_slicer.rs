#![no_main]
use libfuzzer_sys::fuzz_target;
use summoner_dsp::sampler::SampleBuffer;
use summoner_dsp::slicer::{AutoSlicer, SliceAlgorithm};

fuzz_target!(|data: &[u8]| {
    if data.len() < 5 {
        return;
    }
    let threshold = (data[0] as f32 / 255.0 * 10.0).max(0.01);
    let algo = if data[1] % 2 == 0 {
        SliceAlgorithm::EnergyDerivative
    } else {
        SliceAlgorithm::SpectralFlux
    };

    let sample_data: Vec<f32> = data[2..]
        .iter()
        .map(|b| (*b as f32 / 128.0) - 1.0)
        .collect();

    let buffer = SampleBuffer::new(sample_data, 44100, 1);
    let slicer = AutoSlicer::new(threshold, algo);

    let _slices = slicer.detect_slices(&buffer);
});
