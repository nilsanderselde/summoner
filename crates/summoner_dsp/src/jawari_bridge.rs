// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Sitar Non-Linear Curved Jawari Bridge & Tarab Resonator (Milestone 27).
//!
//! Provides a non-linear unilateral curved obstacle contact boundary condition modeling the
//! deer-horn/camel bone/ebony Jawari bridge (generating dynamic time-varying spectral buzz and
//! harmonic enrichment), coupled with a 13-string sympathetic Tarab modal resonator bank tuned
//! to traditional Indian Raga scales, and a Kaddu gourd/Tabli soundboard acoustic body resonator.
//!
//! Enforces zero heap allocations in audio render loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

/// Maximum number of sympathetic Tarab strings on a traditional sitar.
pub const NUM_TARAB_STRINGS: usize = 13;

/// Maximum number of body acoustic resonator modes (Tumba gourd + Tabli soundboard).
pub const NUM_GOURD_MODES: usize = 6;

/// Traditional Indian Raga scale tuning systems for sympathetic Tarab strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RagaScale {
    /// Raga Yaman (Kalyan Thaat: Sa, Re, Ga, tivra Ma, Pa, Dha, Ni -> 1, 9/8, 5/4, 45/32, 3/2, 27/16, 15/8).
    #[default]
    Yaman,
    /// Raga Bhairav (Bhairav Thaat: Sa, komal Re, Ga, Ma, Pa, komal Dha, Ni -> 1, 16/15, 5/4, 4/3, 3/2, 8/5, 15/8).
    Bhairav,
    /// Raga Bilawal (Bilawal Thaat: Natural Shuddha notes -> 1, 9/8, 5/4, 4/3, 3/2, 5/3, 15/8).
    Bilawal,
    /// Raga Darbari Kanhada (Asavari Thaat: Sa, Re, komal Ga, Ma, Pa, komal Dha, komal Ni with slow meend oscillation).
    DarbariKanhada,
    /// Raga Todi (Todi Thaat: Sa, komal Re, komal Ga, tivra Ma, Pa, komal Dha, Ni).
    Todi,
    /// Raga Kafi (Kafi Thaat: Sa, Re, komal Ga, Ma, Pa, Dha, komal Ni).
    Kafi,
    /// Raga Bhairavi (Bhairavi Thaat: All komal notes -> Sa, komal Re, komal Ga, Ma, Pa, komal Dha, komal Ni).
    Bhairavi,
}

impl RagaScale {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Yaman => "Raga Yaman (Evening - Kalyan Thaat)",
            Self::Bhairav => "Raga Bhairav (Dawn - Bhairav Thaat)",
            Self::Bilawal => "Raga Bilawal (Morning - Bilawal Thaat)",
            Self::DarbariKanhada => "Raga Darbari Kanhada (Midnight - Asavari Thaat)",
            Self::Todi => "Raga Miyan Ki Todi (Morning - Todi Thaat)",
            Self::Kafi => "Raga Kafi (Spring / Folk - Kafi Thaat)",
            Self::Bhairavi => "Raga Bhairavi (Universal Finale - Bhairavi Thaat)",
        }
    }

    /// Frequency ratios for all 13 Tarab strings spanning ~2 octaves relative to tonic root frequency $f_0$.
    pub fn tarab_ratios(&self) -> [f32; NUM_TARAB_STRINGS] {
        match self {
            Self::Yaman => [
                // Octave 1 (Tonic to Leading tone)
                1.0, 1.125, 1.25, 1.40625, 1.5, 1.6875, 1.875,
                // Octave 2
                2.0, 2.25, 2.5, 2.8125, 3.0, 3.75,
            ],
            Self::Bhairav => [
                // Sa, komal Re, Ga, Ma, Pa, komal Dha, Ni
                1.0, 1.06667, 1.25, 1.33333, 1.5, 1.60, 1.875,
                2.0, 2.13333, 2.5, 2.66667, 3.0, 3.20,
            ],
            Self::Bilawal => [
                1.0, 1.125, 1.25, 1.33333, 1.5, 1.66667, 1.875,
                2.0, 2.25, 2.5, 2.66667, 3.0, 3.33333,
            ],
            Self::DarbariKanhada => [
                // Sa, Re, komal Ga, Ma, Pa, komal Dha, komal Ni
                1.0, 1.125, 1.20, 1.33333, 1.5, 1.60, 1.77778,
                2.0, 2.25, 2.40, 2.66667, 3.0, 3.20,
            ],
            Self::Todi => [
                // Sa, komal Re, komal Ga, tivra Ma, Pa, komal Dha, Ni
                1.0, 1.06667, 1.20, 1.40625, 1.5, 1.60, 1.875,
                2.0, 2.13333, 2.40, 2.8125, 3.0, 3.20,
            ],
            Self::Kafi => [
                // Sa, Re, komal Ga, Ma, Pa, Dha, komal Ni
                1.0, 1.125, 1.20, 1.33333, 1.5, 1.66667, 1.77778,
                2.0, 2.25, 2.40, 2.66667, 3.0, 3.33333,
            ],
            Self::Bhairavi => [
                // Sa, komal Re, komal Ga, Ma, Pa, komal Dha, komal Ni
                1.0, 1.06667, 1.20, 1.33333, 1.5, 1.60, 1.77778,
                2.0, 2.13333, 2.40, 2.66667, 3.0, 3.20,
            ],
        }
    }
}

/// Sitar Jawari bridge material and curvature preset profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum JawariProfile {
    /// Classic Deer-Horn Curved Jawari (Ravi Shankar style - open buzz, long shimmering sustain).
    #[default]
    DeerHornCurved,
    /// Camel-Bone Sharp Jawari (Vilayat Khan Gayaki style - bright, crisp articulation).
    CamelBoneSharp,
    /// Ebony Hardwood Mellow Jawari (Surbahar bass sitar style - dark, deep woody purr).
    EbonyWoodMellow,
    /// Synthetic Delrin Precision Jawari (consistent climate-stable buzz).
    SyntheticDelrin,
    /// Electric Sitar Flat Metal Jawari (aggressive buzzing sizzle).
    ElectricFlatJawari,
    /// Open Resonant Gourd Jawari (light cotton touch, extended high-frequency cascade).
    OpenGourdAcoustic,
}

impl JawariProfile {
    pub fn name(&self) -> &'static str {
        match self {
            Self::DeerHornCurved => "Deer-Horn Curved Jawari (Ravi Shankar Style)",
            Self::CamelBoneSharp => "Camel-Bone Sharp Jawari (Vilayat Khan Style)",
            Self::EbonyWoodMellow => "Ebony Wood Mellow Jawari (Surbahar Style)",
            Self::SyntheticDelrin => "Synthetic Delrin Precision Jawari",
            Self::ElectricFlatJawari => "Electric Sitar Flat Metal Jawari",
            Self::OpenGourdAcoustic => "Open Resonant Gourd Jawari",
        }
    }

    /// Returns nominal $(h_0 \text{ gap mm}, \text{curvature } c, \text{contact stiffness } K_c, \text{jiva thread damping } \mu)$.
    pub fn nominal_physics(&self) -> (f32, f32, f32, f32) {
        match self {
            Self::DeerHornCurved => (0.18, 120.0, 4.5e7, 0.35),
            Self::CamelBoneSharp => (0.08, 240.0, 8.0e7, 0.25),
            Self::EbonyWoodMellow => (0.35, 60.0, 2.2e7, 0.50),
            Self::SyntheticDelrin => (0.15, 160.0, 5.0e7, 0.30),
            Self::ElectricFlatJawari => (0.05, 320.0, 1.2e8, 0.15),
            Self::OpenGourdAcoustic => (0.22, 90.0, 3.5e7, 0.40),
        }
    }
}

/// Non-linear unilateral curved Jawari obstacle contact boundary model.
///
/// Simulates the dynamic contact of a vibrating string against a wide, gently curved bridge profile:
/// $y_{\text{obs}}(x) = -h_0 + c x^2$
///
/// When string displacement drops below the obstacle profile, unilateral contact occurs,
/// generating non-linear restoring penalty forces:
/// $F_c = K_c \cdot \max(0, y_{\text{obs}} - y)^p \cdot (1 + \mu \dot{y})$
///
/// which introduces dynamic string shortening $\Delta L \propto \sqrt{\max(0, -y / h_0)}$ and rich harmonic buzz.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JawariBridge {
    pub profile: JawariProfile,
    /// Dynamic clearance gap $h_0$ at the bridge apex in mm $[0.01 ..= 1.5]$.
    pub clearance_gap_mm: f32,
    /// Curvature parameter $c$ in $\text{m}^{-1}$.
    pub curvature_c: f32,
    /// Unilateral obstacle contact stiffness $K_c$.
    pub contact_stiffness_k: f32,
    /// Jiva cotton thread position $[0.0 ..= 1.0]$ under the string.
    pub jiva_thread_pos: f32,
    /// Jiva cotton thread thickness / damping factor $\mu$.
    pub jiva_damping: f32,
    /// Previous string displacement for velocity estimation.
    prev_disp: f32,
    /// Accumulated contact force history.
    pub instantaneous_force: f32,
    /// Effective string shortening factor due to obstacle contact.
    pub string_shortening: f32,
    /// Contact engagement flag.
    pub is_in_contact: bool,
}

impl Default for JawariBridge {
    fn default() -> Self {
        Self::new(JawariProfile::DeerHornCurved)
    }
}

impl JawariBridge {
    pub fn new(profile: JawariProfile) -> Self {
        let (h0, c, k, mu) = profile.nominal_physics();
        Self {
            profile,
            clearance_gap_mm: h0,
            curvature_c: c,
            contact_stiffness_k: k,
            jiva_thread_pos: 0.45,
            jiva_damping: mu,
            prev_disp: 0.0,
            instantaneous_force: 0.0,
            string_shortening: 0.0,
            is_in_contact: false,
        }
    }

    pub fn set_profile(&mut self, profile: JawariProfile) {
        self.profile = profile;
        let (h0, c, k, mu) = profile.nominal_physics();
        self.clearance_gap_mm = h0;
        self.curvature_c = c;
        self.contact_stiffness_k = k;
        self.jiva_damping = mu;
    }

    /// Step the unilateral Jawari contact solver for a given string displacement $y_s$ in meters.
    ///
    /// Returns `(contact_force_n, string_shortening_ratio)`.
    #[inline]
    pub fn step_contact(&mut self, string_disp_m: f32, sample_rate: u32) -> (f32, f32) {
        let dt = 1.0 / sample_rate.max(8000) as f32;
        let string_vel = (string_disp_m - self.prev_disp) / dt;
        self.prev_disp = string_disp_m;

        // Obstacle clearance boundary in meters
        let h0_m = (self.clearance_gap_mm * 1e-3).max(1e-5);
        let jiva_offset = (self.jiva_thread_pos - 0.5) * 0.4 * h0_m;
        let y_boundary = -(h0_m + jiva_offset);

        // Penetration depth below curved bridge obstacle
        let penetration = (y_boundary - string_disp_m).max(0.0);

        if penetration > 0.0 {
            self.is_in_contact = true;

            // Non-linear power-law contact force (p ≈ 2.0..2.4 for curved bone/horn surface)
            let p = 2.2;
            let static_f = self.contact_stiffness_k * penetration.powf(p);
            let dynamic_mu = (1.0 + self.jiva_damping * (-string_vel * 0.02).clamp(-10.0, 10.0)).max(0.0);
            let force = (static_f * dynamic_mu).clamp(0.0, 50.0);

            // Effective string length shortening due to rolling contact on curved surface
            // x_contact ≈ sqrt(penetration / c)
            let x_contact = (penetration / self.curvature_c.max(1.0)).sqrt();
            let shortening = (x_contact * 0.05).clamp(0.0, 0.08);

            self.instantaneous_force = force;
            self.string_shortening = shortening;
            (force, shortening)
        } else {
            self.is_in_contact = false;
            self.instantaneous_force = 0.0;
            self.string_shortening = 0.0;
            (0.0, 0.0)
        }
    }

    pub fn reset(&mut self) {
        self.prev_disp = 0.0;
        self.instantaneous_force = 0.0;
        self.string_shortening = 0.0;
        self.is_in_contact = false;
    }
}

/// A 2nd-order resonant bandpass resonator representing a single sympathetic Tarab string.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TarabStringResonator {
    pub freq_hz: f32,
    pub q: f32,
    pub gain: f32,
    pub energy_level: f32,
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    w1: f32,
    w2: f32,
}

impl Default for TarabStringResonator {
    fn default() -> Self {
        Self {
            freq_hz: 261.63,
            q: 60.0,
            gain: 0.15,
            energy_level: 0.0,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            w1: 0.0,
            w2: 0.0,
        }
    }
}

impl TarabStringResonator {
    pub fn new(sample_rate: u32, freq_hz: f32, q: f32, gain: f32) -> Self {
        let mut resonator = Self {
            freq_hz,
            q: q.max(1.0),
            gain,
            energy_level: 0.0,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            w1: 0.0,
            w2: 0.0,
        };
        resonator.update_coefficients(sample_rate);
        resonator
    }

    pub fn update_coefficients(&mut self, sample_rate: u32) {
        let sr = sample_rate.max(8000) as f32;
        let omega = (2.0 * PI * self.freq_hz.clamp(20.0, sr * 0.48) / sr).min(PI * 0.98);
        let alpha = omega.sin() / (2.0 * self.q.max(1.0));
        let cos_w = omega.cos();

        let b0 = alpha * self.gain;
        let b1 = 0.0;
        let b2 = -alpha * self.gain;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_w;
        let a2 = 1.0 - alpha;

        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }

    #[inline]
    pub fn step(&mut self, input: f32) -> f32 {
        let w0 = input - self.a1 * self.w1 - self.a2 * self.w2;
        let y = self.b0 * w0 + self.b1 * self.w1 + self.b2 * self.w2;
        self.w2 = self.w1;
        self.w1 = w0;

        // Envelope follower for HUD energy visualization
        let env_alpha = 0.005;
        self.energy_level = (1.0 - env_alpha) * self.energy_level + env_alpha * y.abs();
        y
    }

    pub fn reset(&mut self) {
        self.w1 = 0.0;
        self.w2 = 0.0;
        self.energy_level = 0.0;
    }
}

/// 13-String Sympathetic Tarab Resonator Bank tuned dynamically to active Raga scale.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TarabResonatorBank {
    pub scale: RagaScale,
    pub root_freq_hz: f32,
    pub strings: [TarabStringResonator; NUM_TARAB_STRINGS],
    pub coupling_bleed: f32,
    pub sample_rate: u32,
}

impl TarabResonatorBank {
    pub fn new(sample_rate: u32, scale: RagaScale, root_freq_hz: f32) -> Self {
        let ratios = scale.tarab_ratios();
        let mut strings = [TarabStringResonator::default(); NUM_TARAB_STRINGS];

        for i in 0..NUM_TARAB_STRINGS {
            let f = (root_freq_hz * ratios[i]).clamp(50.0, 6000.0);
            let q = 45.0 + 15.0 * (i as f32 / NUM_TARAB_STRINGS as f32);
            strings[i] = TarabStringResonator::new(sample_rate, f, q, 0.25);
        }

        Self {
            scale,
            root_freq_hz,
            strings,
            coupling_bleed: 0.35,
            sample_rate,
        }
    }

    pub fn set_scale(&mut self, scale: RagaScale) {
        self.scale = scale;
        self.retune_all();
    }

    pub fn set_root_freq(&mut self, root_hz: f32) {
        self.root_freq_hz = root_hz.clamp(55.0, 880.0);
        self.retune_all();
    }

    pub fn retune_all(&mut self) {
        let ratios = self.scale.tarab_ratios();
        for (i, ratio) in ratios.iter().enumerate().take(NUM_TARAB_STRINGS) {
            let f = (self.root_freq_hz * ratio).clamp(50.0, 6000.0);
            self.strings[i].freq_hz = f;
            self.strings[i].update_coefficients(self.sample_rate);
        }
    }

    /// Process sympathetic excitation from main playing string & Chikari drones through the 13 Tarab strings.
    #[inline]
    pub fn process_sympathetic(&mut self, excitation: f32) -> f32 {
        let mut total_tarab = 0.0;
        let scaled_in = excitation * self.coupling_bleed;

        for s in &mut self.strings {
            total_tarab += s.step(scaled_in);
        }
        total_tarab
    }

    /// Instantaneous energy levels of all 13 Tarab strings for GUI HUD visualization.
    pub fn energy_levels(&self) -> [f32; NUM_TARAB_STRINGS] {
        let mut levels = [0.0; NUM_TARAB_STRINGS];
        for (i, s) in self.strings.iter().enumerate() {
            levels[i] = s.energy_level.clamp(0.0, 1.0);
        }
        levels
    }

    pub fn reset(&mut self) {
        for s in &mut self.strings {
            s.reset();
        }
    }
}

/// Sitar acoustic body resonator modeling the dried Kaddu Tumba gourd & Tun wood Tabli soundboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitarGourdBody {
    pub modes: [TarabStringResonator; NUM_GOURD_MODES],
    pub decay_scale: f32,
    pub sample_rate: u32,
}

impl SitarGourdBody {
    pub fn new(sample_rate: u32) -> Self {
        // Modal frequencies of Kaddu gourd cavity + flat wooden tabli plate:
        // F0: ~92 Hz (Tumba air Helmholtz cavity breathing)
        // F1: ~185 Hz (Main tumba shell vibration)
        // F2: ~310 Hz (Tabli soundboard longitudinal flexure)
        // F3: ~580 Hz (Upper bridge rib resonance)
        // F4: ~1150 Hz (High-frequency body projection)
        // F5: ~2400 Hz (Neck dandi coupling)
        let freqs = [92.0, 185.0, 310.0, 580.0, 1150.0, 2400.0];
        let qs = [12.0, 18.0, 22.0, 26.0, 32.0, 38.0];
        let gains = [0.45, 0.40, 0.35, 0.30, 0.25, 0.18];

        let mut modes = [TarabStringResonator::default(); NUM_GOURD_MODES];
        for i in 0..NUM_GOURD_MODES {
            modes[i] = TarabStringResonator::new(sample_rate, freqs[i], qs[i], gains[i]);
        }

        Self {
            modes,
            decay_scale: 1.0,
            sample_rate,
        }
    }

    pub fn set_decay_scale(&mut self, scale: f32) {
        self.decay_scale = scale.clamp(0.1, 4.0);
        let base_qs = [12.0, 18.0, 22.0, 26.0, 32.0, 38.0];
        for (i, base_q) in base_qs.iter().enumerate().take(NUM_GOURD_MODES) {
            self.modes[i].q = base_q * self.decay_scale;
            self.modes[i].update_coefficients(self.sample_rate);
        }
    }

    /// Process excitation force through the body resonator modal matrix and return (Left, Right) stereo acoustic radiation.
    #[inline]
    pub fn process_body(&mut self, input_force: f32) -> (f32, f32) {
        let mut out_l = 0.0;
        let mut out_r = 0.0;

        // Stereo spatial pan coefficients for each body mode
        let pans: [f32; 6] = [-0.15, 0.10, -0.20, 0.25, -0.05, 0.15];

        for (i, mode) in self.modes.iter_mut().enumerate() {
            let m_out = mode.step(input_force);
            let pan_norm: f32 = (pans[i] + 1.0) * 0.5;
            out_l += m_out * (1.0 - pan_norm).sqrt();
            out_r += m_out * pan_norm.sqrt();
        }

        (out_l, out_r)
    }

    pub fn reset(&mut self) {
        for m in &mut self.modes {
            m.reset();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jawari_obstacle_contact_non_linear_dynamics() {
        let mut bridge = JawariBridge::new(JawariProfile::DeerHornCurved);
        // Small displacement above clearance -> no contact
        let (f0, s0) = bridge.step_contact(0.0005, 48000);
        assert_eq!(f0, 0.0);
        assert_eq!(s0, 0.0);
        assert!(!bridge.is_in_contact);

        // Deep negative displacement below obstacle curve -> contact force and shortening
        let (f1, s1) = bridge.step_contact(-0.002, 48000);
        assert!(f1 > 0.0);
        assert!(s1 > 0.0);
        assert!(bridge.is_in_contact);
    }

    #[test]
    fn test_tarab_resonator_bank_raga_tunings() {
        let mut tarab = TarabResonatorBank::new(48000, RagaScale::Yaman, 138.59);
        assert_eq!(tarab.strings.len(), 13);
        assert!((tarab.strings[0].freq_hz - 138.59).abs() < 1e-2);

        // Switch to Bhairav
        tarab.set_scale(RagaScale::Bhairav);
        assert_eq!(tarab.scale, RagaScale::Bhairav);
        let out = tarab.process_sympathetic(1.0);
        assert!(out.is_finite());
        let energy = tarab.energy_levels();
        assert_eq!(energy.len(), 13);
    }

    #[test]
    fn test_sitar_gourd_body_resonance() {
        let mut body = SitarGourdBody::new(48000);
        let (l, r) = body.process_body(1.0);
        assert!(l.is_finite() && r.is_finite());
        assert!(l.abs() > 0.0 || r.abs() > 0.0);
    }
}
