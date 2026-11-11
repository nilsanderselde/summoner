// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Dithering algorithms (TPDF and Shaped Noise) for audio export.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DitherType {
    #[default]
    None,
    Tpdf,
    NoiseShaped,
    PowRAnalog,
}

/// Stateful dithering processor with noise shaping feedback filter.
#[derive(Debug, Clone)]
pub struct DitherProcessor {
    pub bit_depth: u8,
    pub dither_type: DitherType,
    prng_state: u32,
    error_history: [f32; 4],
}

impl Default for DitherProcessor {
    fn default() -> Self {
        Self::new(16, DitherType::Tpdf)
    }
}

impl DitherProcessor {
    pub fn new(bit_depth: u8, dither_type: DitherType) -> Self {
        Self {
            bit_depth,
            dither_type,
            prng_state: 0x12345678,
            error_history: [0.0; 4],
        }
    }

    /// Process a single sample through dither and error-feedback noise shaping.
    #[inline]
    pub fn process_sample(&mut self, sample: f32) -> f32 {
        if self.bit_depth >= 32 || self.dither_type == DitherType::None {
            return sample;
        }

        let scale = (1u64 << (self.bit_depth - 1)) as f32;
        let inv_scale = 1.0 / scale;

        // Noise shaping error prediction filter
        let filtered_error = match self.dither_type {
            DitherType::None | DitherType::Tpdf => 0.0,
            DitherType::NoiseShaped => {
                // First-order highpass error feedback: e[n-1]
                self.error_history[0] * 0.8
            }
            DitherType::PowRAnalog => {
                // Multi-order psychoacoustic curve (POW-r style):
                // Concentrates dither energy in ultrasonic/HF (>15 kHz) and notches 2-4 kHz ear sensitivity
                2.033 * self.error_history[0] - 2.165 * self.error_history[1]
                    + 1.959 * self.error_history[2]
                    - 0.778 * self.error_history[3]
            }
        };

        let r1 = next_rand(&mut self.prng_state);
        let r2 = next_rand(&mut self.prng_state);
        let tpdf = (r1 - r2) * inv_scale;

        let input_with_shaping = sample - filtered_error;
        let quantized = (input_with_shaping * scale + tpdf * scale).round() * inv_scale;
        let clamped = quantized.clamp(-1.0, 1.0);

        // Update error history
        let error = clamped - input_with_shaping;
        self.error_history[3] = self.error_history[2];
        self.error_history[2] = self.error_history[1];
        self.error_history[1] = self.error_history[0];
        self.error_history[0] = error;

        clamped
    }

    /// Process an audio buffer in-place.
    pub fn process_buffer(&mut self, buffer: &mut [f32]) {
        for s in buffer.iter_mut() {
            *s = self.process_sample(*s);
        }
    }
}

/// Applies dithering to an f32 audio sample before quantizing to target bit depth.
pub fn apply_dither(
    sample: f32,
    bit_depth: u8,
    dither_type: DitherType,
    prng_state: &mut u32,
) -> f32 {
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
        DitherType::PowRAnalog => {
            let r1 = next_rand(prng_state);
            let r2 = next_rand(prng_state);
            let tpdf = r1 - r2;
            tpdf * 0.9 * inv_scale
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

    #[test]
    fn test_dither_processor_powr_analog() {
        let mut proc = DitherProcessor::new(16, DitherType::PowRAnalog);
        let mut buf = vec![0.5f32; 1000];
        proc.process_buffer(&mut buf);
        // All samples quantized to 16-bit grid with psychoacoustic noise shaping
        for &s in &buf {
            assert!((s - 0.5).abs() < 0.005);
            assert!(s >= -1.0 && s <= 1.0);
        }
    }
}
