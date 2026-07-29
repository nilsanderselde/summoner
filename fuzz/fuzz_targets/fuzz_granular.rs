#![no_main]
use libfuzzer_sys::fuzz_target;
use std::sync::Arc;
use summoner_core::node::ProcessContext;
use summoner_dsp::granular::GranularSynthNode;
use summoner_dsp::sampler::SampleBuffer;
use summoner_dsp::traits::SignalProcessor;

fuzz_target!(|data: &[u8]| {
    if data.len() < 8 {
        return;
    }
    let density = (data[0] as f32 / 255.0 * 50.0).max(0.1);
    let grain_size_ms = (data[1] as f32 / 255.0 * 200.0).max(5.0);
    let spray = data[2] as f32 / 255.0;

    let sample_data: Vec<f32> = data[3..]
        .iter()
        .map(|b| (*b as f32 / 128.0) - 1.0)
        .collect();

    let mut granular = GranularSynthNode::new(44100);
    granular.density = density;
    granular.grain_size_ms = grain_size_ms;
    granular.spray = spray;
    granular.load_buffer(Arc::new(SampleBuffer::new(sample_data, 44100, 1)));

    let mut out_buf = vec![0.0f32; 128];
    let ctx = ProcessContext::new(44100, 120.0, 0);

    let empty_in: [&[f32]; 0] = [];
    granular.process_block(&empty_in, &mut [&mut out_buf[..]], &ctx);
});
