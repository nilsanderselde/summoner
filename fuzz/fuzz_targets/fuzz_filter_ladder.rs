#![no_main]
use libfuzzer_sys::fuzz_target;
use summoner_core::node::ProcessContext;
use summoner_dsp::filters::FilterLadder;
use summoner_dsp::traits::SignalProcessor;

fuzz_target!(|data: &[u8]| {
    if data.len() < 8 {
        return;
    }
    let cutoff = f32::from_le_bytes([data[0], data[1], data[2], data[3]]).abs() % 20000.0;
    let res = (data[4] as f32 / 255.0).clamp(0.0, 1.0);

    let mut filter = FilterLadder::new(cutoff, res);
    let mut in_buf = vec![0.0f32; 128];
    for (i, byte) in data.iter().skip(5).take(128).enumerate() {
        in_buf[i] = (*byte as f32 / 128.0) - 1.0;
    }
    let mut out_buf = vec![0.0f32; 128];
    let ctx = ProcessContext::new(44100, 120.0, 0);

    filter.process_block(&[&in_buf[..]], &mut [&mut out_buf[..]], &ctx);
});
