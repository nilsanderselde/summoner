#![no_main]
use libfuzzer_sys::fuzz_target;
use summoner_core::node::ProcessContext;
use summoner_dsp::oscillators::OscSaw;
use summoner_dsp::traits::SignalProcessor;

fuzz_target!(|data: &[u8]| {
    if data.len() < 4 {
        return;
    }
    let freq = f32::from_le_bytes([data[0], data[1], data[2], data[3]]).abs() % 20000.0;
    let mut osc = OscSaw::new(freq);
    let mut out_buf = vec![0.0f32; 128];
    let ctx = ProcessContext::new(44100, 120.0, 0);

    let empty_in: [&[f32]; 0] = [];
    osc.process_block(&empty_in, &mut [&mut out_buf[..]], &ctx);
});
