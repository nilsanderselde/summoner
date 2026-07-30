// Summoner DAW - Track Strip DSP Processing & Correlation Tools (Steps 687, 691-694)
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

/// Step 687: Phase Flip -- inverts signal polarity.
pub fn apply_phase_flip(samples: &mut [f32]) {
    for sample in samples.iter_mut() {
        *sample = -*sample;
    }
}

/// Step 691: Spectrum Matching -- computes gain adjustment offsets (in dB) per frequency band to match target spectrum.
pub fn compute_spectrum_matching_gain_offsets(
    source_spectrum: &[f32],
    target_spectrum: &[f32],
) -> Vec<f32> {
    let len = source_spectrum.len().min(target_spectrum.len());
    let mut gain_offsets = Vec::with_capacity(len);

    for i in 0..len {
        let src = source_spectrum[i].max(1e-6);
        let tgt = target_spectrum[i].max(1e-6);
        let ratio = tgt / src;
        let gain_db = (20.0 * ratio.log10()).clamp(-24.0, 24.0);
        gain_offsets.push(gain_db);
    }

    gain_offsets
}

/// Step 692: Correlation Meter -- calculates Pearson correlation coefficient (-1.0 to +1.0) between L and R channels for mono compatibility.
pub fn compute_stereo_correlation(l_samples: &[f32], r_samples: &[f32]) -> f32 {
    let len = l_samples.len().min(r_samples.len());
    if len == 0 {
        return 1.0;
    }

    let mut dot_product = 0.0f64;
    let mut sum_sq_l = 0.0f64;
    let mut sum_sq_r = 0.0f64;

    for i in 0..len {
        let l = l_samples[i] as f64;
        let r = r_samples[i] as f64;
        dot_product += l * r;
        sum_sq_l += l * l;
        sum_sq_r += r * r;
    }

    let denom = (sum_sq_l * sum_sq_r).sqrt();
    if denom < 1e-9 {
        1.0
    } else {
        (dot_product / denom).clamp(-1.0, 1.0) as f32
    }
}

/// Step 693: Master Bus Gain Trim -- applies separate gain trim (in dB) on master bus.
pub fn apply_master_trim(samples: &mut [f32], master_trim_db: f32) {
    if master_trim_db.abs() < 1e-4 {
        return;
    }
    let linear_gain = 10.0f32.powf(master_trim_db / 20.0);
    for s in samples.iter_mut() {
        *s *= linear_gain;
    }
}

/// Step 694: Input Gain Trim per track.
pub fn apply_input_gain(samples: &mut [f32], input_gain_db: f32) {
    if input_gain_db.abs() < 1e-4 {
        return;
    }
    let linear_gain = 10.0f32.powf(input_gain_db / 20.0);
    for s in samples.iter_mut() {
        *s *= linear_gain;
    }
}

/// Step 694: Output Gain Trim per track.
pub fn apply_output_gain(samples: &mut [f32], output_gain_db: f32) {
    if output_gain_db.abs() < 1e-4 {
        return;
    }
    let linear_gain = 10.0f32.powf(output_gain_db / 20.0);
    for s in samples.iter_mut() {
        *s *= linear_gain;
    }
}
