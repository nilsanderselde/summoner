#![no_main]
use libfuzzer_sys::fuzz_target;
use summoner_dsp::waveguide::KarplusStrongString;

fuzz_target!(|data: &[u8]| {
    if data.len() < 9 {
        return;
    }
    let freq = f32::from_le_bytes([data[0], data[1], data[2], data[3]]).abs() % 10000.0;
    let feedback = (data[4] as f32 / 255.0).clamp(0.1, 0.999);
    let pluck_amp = (data[5] as f32 / 255.0).clamp(0.0, 2.0);

    let mut ks = KarplusStrongString::new(freq.max(20.0), 44100, feedback);
    ks.pluck(pluck_amp);

    for _ in 0..128 {
        let _s = ks.process_sample();
    }
});
