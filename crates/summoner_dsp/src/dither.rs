// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Dithering algorithms (TPDF and Shaped Noise) for audio export.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DitherType {
    None,
    Tpdf,
    NoiseShaped,
}

/// Applies dithering to an f32 audio sample before quantizing to target bit depth.
pub fn apply_dither(sample: f32, bit_depth: u8, dither_type: DitherType, prng_state: &mut u32) -> f32 {
    if bit_depth >= 32 || dither_type == DitherType::None {
        return sample;
    }

    let scale = (1u64 << (bit_depth - 1)) as f32;
    let inv_scale = 1.0 / scale;

    let noise = match dither_type {
        DitherType::None => 0.0,
        DitherType::Tpdf => {
            let r1 = next_rand(prng_state);
            let r2 = next_rand(prng_state);
            (r1 - r2) * inv_scale
        }
        DitherType::NoiseShaped => {
            let r1 = next_rand(prng_state);
            let r2 = next_rand(prng_state);
            let tpdf = r1 - r2;
            tpdf * 0.75 * inv_scale
        }
    };

    let dithered = (sample * scale + noise).round() * inv_scale;
    dithered.clamp(-1.0, 1.0)
}

fn next_rand(state: &mut u32) -> f32 {
    *state = state.wrapping_mul(1664525).wrapping_add(1013904223);
    ((*state >> 9) as f32) / ((1 << 23) as f32) - 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_dither_tpdf() {
        let mut prng = 12345;
        let original = 0.12345;
        let dithered = apply_dither(original, 16, DitherType::Tpdf, &mut prng);
        assert!((dithered - original).abs() < 0.001);
    }
}
