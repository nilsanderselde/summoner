// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Cassotto Tone Chamber & Soundbox Acoustic Resonator (Milestone 28).
//!
//! Provides a wooden Cassotto tone chamber and accordion/bandoneon soundbox body resonator model.
//! In high-end free-reed aerophones, reeds mounted inside an enclosed wooden tone chamber (Cassotto)
//! undergo acoustic low-pass filtering and characteristic formant resonance (warmth around 400-900 Hz,
//! attenuation of harsh overtones above 2.5 kHz). Reeds mounted outside (direct) radiate directly into the soundbox.
//!
//! Enforces zero heap allocations in audio render loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

/// Maximum number of resonant cavity modal filters in the cassotto soundbox.
pub const NUM_CASSOTTO_MODES: usize = 4;

/// Preset acoustic profiles for the Cassotto tone chamber and soundbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CassottoProfile {
    /// Doble Cassotto Bandoneon (Premier 2-rank chamber for 16' Bassoon & 8' Clarinet, deep velvety resonance).
    #[default]
    DobleCassottoBandoneon,
    /// Standard Double Cassotto Accordion (Italian solid maple chamber, warm singing mids).
    StandardAccordionChamber,
    /// Vintage Concertina Spruce Cavity (Compact hexagonal body, crisp fast acoustic reflection).
    ConcertinaSpruceCavity,
    /// Harmonium Resonant Soundbox (Large pine pumping box with rich low-mid acoustic sustain).
    HarmoniumResonantBox,
    /// Open-Air Russian Bayan (Direct open reedblock with bright cutting projection).
    OpenAirBayan,
}

impl CassottoProfile {
    pub fn name(&self) -> &'static str {
        match self {
            Self::DobleCassottoBandoneon => "Doble Cassotto Bandoneon (Premier Chamber)",
            Self::StandardAccordionChamber => "Standard Double Cassotto (Italian Maple)",
            Self::ConcertinaSpruceCavity => "Vintage Concertina Spruce Body",
            Self::HarmoniumResonantBox => "Harmonium Resonant Soundbox",
            Self::OpenAirBayan => "Open-Air Russian Bayan (Direct)",
        }
    }

    /// Returns nominal $(f_{\text{mode1}}, f_{\text{mode2}}, f_{\text{cutoff\_base}}, Q_{\text{cavity}}, \text{wood\_warmth})$.
    pub fn nominal_acoustics(&self) -> (f32, f32, f32, f32, f32) {
        match self {
            Self::DobleCassottoBandoneon => (460.0, 1420.0, 2600.0, 3.8, 0.85),
            Self::StandardAccordionChamber => (520.0, 1680.0, 3200.0, 3.2, 0.75),
            Self::ConcertinaSpruceCavity => (680.0, 2100.0, 4800.0, 2.4, 0.50),
            Self::HarmoniumResonantBox => (340.0, 1150.0, 2100.0, 4.5, 0.95),
            Self::OpenAirBayan => (850.0, 2800.0, 8000.0, 1.5, 0.25),
        }
    }
}

/// A second-order biquad filter section for zero-heap acoustic resonance filtering.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct CassottoBiquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl CassottoBiquad {
    pub fn new() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    pub fn set_lowpass(&mut self, sample_rate: u32, cutoff_hz: f32, q: f32) {
        let sr = sample_rate.max(8000) as f32;
        let fc = cutoff_hz.clamp(20.0, sr * 0.48);
        let q = q.max(0.1);
        let omega = 2.0 * PI * fc / sr;
        let sn = omega.sin();
        let cs = omega.cos();
        let alpha = sn / (2.0 * q);

        let a0 = 1.0 + alpha;
        self.b0 = ((1.0 - cs) * 0.5) / a0;
        self.b1 = (1.0 - cs) / a0;
        self.b2 = ((1.0 - cs) * 0.5) / a0;
        self.a1 = (-2.0 * cs) / a0;
        self.a2 = (1.0 - alpha) / a0;
    }

    pub fn set_bandpass(&mut self, sample_rate: u32, center_hz: f32, q: f32) {
        let sr = sample_rate.max(8000) as f32;
        let fc = center_hz.clamp(20.0, sr * 0.48);
        let q = q.max(0.1);
        let omega = 2.0 * PI * fc / sr;
        let sn = omega.sin();
        let cs = omega.cos();
        let alpha = sn / (2.0 * q);

        let a0 = 1.0 + alpha;
        self.b0 = (sn * 0.5) / a0;
        self.b1 = 0.0;
        self.b2 = (-sn * 0.5) / a0;
        self.a1 = (-2.0 * cs) / a0;
        self.a2 = (1.0 - alpha) / a0;
    }

    pub fn set_peak(&mut self, sample_rate: u32, center_hz: f32, gain_db: f32, q: f32) {
        let sr = sample_rate.max(8000) as f32;
        let fc = center_hz.clamp(20.0, sr * 0.48);
        let q = q.max(0.1);
        let a = 10.0f32.powf(gain_db / 40.0);
        let omega = 2.0 * PI * fc / sr;
        let sn = omega.sin();
        let cs = omega.cos();
        let alpha = sn / (2.0 * q);

        let a0 = 1.0 + alpha / a;
        self.b0 = (1.0 + alpha * a) / a0;
        self.b1 = (-2.0 * cs) / a0;
        self.b2 = (1.0 - alpha * a) / a0;
        self.a1 = (-2.0 * cs) / a0;
        self.a2 = (1.0 - alpha / a) / a0;
    }

    #[inline(always)]
    pub fn process_sample(&mut self, input: f32) -> f32 {
        let out = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2 - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = if out.is_finite() { out } else { 0.0 };
        self.y1
    }

    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
}

/// Physical acoustic Cassotto Tone Chamber & Soundbox Resonator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CassottoChamber {
    pub profile: CassottoProfile,
    pub sample_rate: u32,
    /// Aperture shutter opening fraction $[0.0 ..= 1.0]$ (1.0 = fully open chamber, 0.0 = dark closed chamber).
    pub aperture: f32,
    /// Wooden warmth resonance gain factor $[0.0 ..= 2.0]$.
    pub wood_warmth: f32,
    /// Tone chamber modal filter bank.
    pub chamber_modes: [CassottoBiquad; NUM_CASSOTTO_MODES],
    /// Acoustic lowpass transmission loss filter for cassotto channel.
    pub cassotto_lowpass: CassottoBiquad,
    /// Soundbox wooden body resonance filter.
    pub body_resonance: CassottoBiquad,
}

impl CassottoChamber {
    pub fn new(sample_rate: u32) -> Self {
        let mut chamber = Self {
            profile: CassottoProfile::DobleCassottoBandoneon,
            sample_rate: sample_rate.max(8000),
            aperture: 0.85,
            wood_warmth: 0.85,
            chamber_modes: [CassottoBiquad::new(); NUM_CASSOTTO_MODES],
            cassotto_lowpass: CassottoBiquad::new(),
            body_resonance: CassottoBiquad::new(),
        };
        chamber.reconfigure_filters();
        chamber
    }

    pub fn set_profile(&mut self, profile: CassottoProfile) {
        self.profile = profile;
        let (_, _, _, _, warmth) = profile.nominal_acoustics();
        self.wood_warmth = warmth;
        self.reconfigure_filters();
    }

    pub fn set_aperture(&mut self, aperture: f32) {
        self.aperture = aperture.clamp(0.0, 1.0);
        self.reconfigure_filters();
    }

    pub fn set_wood_warmth(&mut self, warmth: f32) {
        self.wood_warmth = warmth.clamp(0.0, 2.0);
        self.reconfigure_filters();
    }

    pub fn reconfigure_filters(&mut self) {
        let (f1, f2, fc_base, q, _) = self.profile.nominal_acoustics();
        let sr = self.sample_rate;

        // Mode 1: Primary chamber resonance
        self.chamber_modes[0].set_peak(sr, f1, 6.0 * self.wood_warmth, q);
        // Mode 2: Secondary tone chamber harmonic formant
        self.chamber_modes[1].set_peak(sr, f2, 4.5 * self.wood_warmth, q * 1.2);
        // Mode 3: Soundbox lower cavity warmth mode (~240 Hz)
        self.chamber_modes[2].set_peak(sr, 240.0, 3.5 * self.wood_warmth, 2.5);
        // Mode 4: High air cavity damping notch / peak (~3400 Hz)
        self.chamber_modes[3].set_peak(sr, 3400.0, -4.0 * (1.0 - self.aperture), 2.0);

        // Cassotto low-pass filter tracking aperture opening
        let effective_cutoff = (fc_base * (0.35 + 0.65 * self.aperture)).clamp(100.0, sr as f32 * 0.45);
        self.cassotto_lowpass.set_lowpass(sr, effective_cutoff, 0.707);

        // Soundbox body resonance (birch/spruce cavity)
        self.body_resonance.set_bandpass(sr, 580.0, 1.8);
    }

    /// Process a single stereo/split sample pair:
    /// `cassotto_in`: signal from reed ranks mounted inside the Cassotto chamber (e.g. Bassoon 16', Clarinet 8').
    /// `direct_in`: signal from reed ranks mounted directly in the soundbox (e.g. Musette 8'+/-, Piccolo 4').
    #[inline(always)]
    pub fn process_sample(&mut self, cassotto_in: f32, direct_in: f32) -> f32 {
        // 1. Process Cassotto channel through tone chamber low-pass and formant modes
        let filtered_cassotto = self.cassotto_lowpass.process_sample(cassotto_in);
        let mut cassotto_shaped = filtered_cassotto;
        for mode in &mut self.chamber_modes {
            cassotto_shaped = mode.process_sample(cassotto_shaped);
        }

        // 2. Mix Cassotto chamber output with direct reed output
        let mixed = cassotto_shaped * 0.95 + direct_in * 0.85;

        // 3. Couple mixed signal into soundbox body resonance
        let body_refl = self.body_resonance.process_sample(mixed) * (0.15 * self.wood_warmth);

        // 4. Return total acoustic pressure
        mixed + body_refl
    }

    /// Reset internal filter state buffers.
    pub fn reset(&mut self) {
        for mode in &mut self.chamber_modes {
            mode.reset();
        }
        self.cassotto_lowpass.reset();
        self.body_resonance.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cassotto_filter_stability_and_profiles() {
        let mut chamber = CassottoChamber::new(48000);

        for prof in [
            CassottoProfile::DobleCassottoBandoneon,
            CassottoProfile::StandardAccordionChamber,
            CassottoProfile::ConcertinaSpruceCavity,
            CassottoProfile::HarmoniumResonantBox,
            CassottoProfile::OpenAirBayan,
        ] {
            chamber.set_profile(prof);
            assert!(!prof.name().is_empty());
            let (f1, f2, fc, q, warmth) = prof.nominal_acoustics();
            assert!(f1 > 100.0 && f1 < 2000.0);
            assert!(f2 > f1 && f2 < 5000.0);
            assert!(fc > 500.0);
            assert!(q > 0.5);
            assert!(warmth >= 0.0);

            // Process test impulse
            let out_impulse = chamber.process_sample(1.0, 0.5);
            assert!(out_impulse.is_finite());
            assert!(out_impulse != 0.0);

            // Verify decay over 200 samples
            let mut last_sample = 0.0;
            for _ in 0..200 {
                last_sample = chamber.process_sample(0.0, 0.0);
                assert!(last_sample.is_finite());
            }
            assert!(last_sample.abs() < 1.0);
        }
    }

    #[test]
    fn test_cassotto_aperture_attenuation() {
        let mut chamber = CassottoChamber::new(48000);
        chamber.set_profile(CassottoProfile::DobleCassottoBandoneon);

        // Closed aperture (dark)
        chamber.set_aperture(0.0);
        let mut energy_closed = 0.0f32;
        for i in 0..1000 {
            let high_freq_sine = (i as f32 * 0.6).sin(); // ~4.5 kHz tone
            let out = chamber.process_sample(high_freq_sine, 0.0);
            energy_closed += out * out;
        }

        chamber.reset();

        // Open aperture (bright)
        chamber.set_aperture(1.0);
        let mut energy_open = 0.0f32;
        for i in 0..1000 {
            let high_freq_sine = (i as f32 * 0.6).sin();
            let out = chamber.process_sample(high_freq_sine, 0.0);
            energy_open += out * out;
        }

        assert!(
            energy_open > energy_closed,
            "Open cassotto aperture must pass more high-frequency energy than closed aperture (open: {}, closed: {})",
            energy_open,
            energy_closed
        );
    }
}
