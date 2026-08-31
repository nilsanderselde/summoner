// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling Hurdy-Gurdy Chien (Buzzing Dog Bridge) & Soundbox Body (Milestone 30).
//!
//! Provides a high-precision acoustic modeling system for the loose rocking *chien* (buzzing dog)
//! bridge and resonant vielle soundbox body of the Hurdy-Gurdy (Vielle à roue) featuring:
//! - Non-linear unilateral obstacle contact collision dynamics against the bone/ivory striking plate
//! - Dynamic *coup de poignet* wrist acceleration burst response generating percussive buzzing snarls
//! - Adjustable clearance gap $h_0 \in [0.05, 1.2]\text{ mm}$ via virtual tirant peg string tension
//! - Multi-pole modal Paulownia/Maple/Spruce soundbox body cavity acoustic resonator (8 modes)
//! - Energy-conserving restitution and contact damping
//!
//! Enforces zero heap allocations in audio render loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

/// Number of modal resonator filters for the Hurdy-Gurdy resonant wooden soundbox.
pub const NUM_HURDY_GURDY_BODY_MODES: usize = 8;

/// Material choices for the Chien (buzzing dog) bridge and striking plate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ChienMaterialProfile {
    /// Traditional Boxwood / Maple Dog on Bone Plate (Crisp, snappy snarl).
    #[default]
    MapleOnBone,
    /// Dense Antique Ivory Plate (Sharp metallic attack with high harmonic brightness).
    IvoryPlate,
    /// Hard Ebony Wood (Deep, dark woody rattle with fast transient damping).
    EbonyHardwood,
    /// Modern Delrin / Synthetic Polymer (Even, stable buzzing across all crank speeds).
    SyntheticDelrin,
    /// Walnut / Fruitwood Vintage (Warm, mellow acoustic chatter).
    VintageFruitwood,
}

impl ChienMaterialProfile {
    pub fn name(&self) -> &'static str {
        match self {
            Self::MapleOnBone => "Maple Dog on Bone Plate (Traditional)",
            Self::IvoryPlate => "Ivory Striking Plate (Bright Attack)",
            Self::EbonyHardwood => "Ebony Hardwood (Dark Woody Buzz)",
            Self::SyntheticDelrin => "Synthetic Delrin (Clean & Stable)",
            Self::VintageFruitwood => "Vintage Fruitwood (Warm & Mellow)",
        }
    }

    /// Returns nominal $(K_{\text{contact\_stiffness}}, C_{\text{contact\_damping}}, \text{restitution}, \text{buzz\_gain})$.
    pub fn nominal_physics(&self) -> (f32, f32, f32, f32) {
        match self {
            Self::MapleOnBone => (8500.0, 0.45, 0.65, 1.20),
            Self::IvoryPlate => (12000.0, 0.30, 0.75, 1.35),
            Self::EbonyHardwood => (7000.0, 0.60, 0.55, 1.05),
            Self::SyntheticDelrin => (9000.0, 0.40, 0.70, 1.15),
            Self::VintageFruitwood => (6500.0, 0.50, 0.60, 1.10),
        }
    }
}

/// Unilateral obstacle collision junction modeling the rattling Chien (buzzing dog bridge).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TrompetteChienJunction {
    /// Material and contact hardness profile.
    pub profile: ChienMaterialProfile,
    /// Resting clearance gap in millimeters $[0.05 ..= 1.20]\text{ mm}$.
    pub clearance_gap_mm: f32,
    /// Instantaneous dog displacement relative to rest position in mm.
    pub displacement_mm: f32,
    /// Instantaneous dog velocity in mm/s.
    pub velocity_mm_s: f32,
    /// Contact collision stiffness $k_c$.
    pub contact_stiffness: f32,
    /// Contact collision damping coefficient $c_c$.
    pub contact_damping: f32,
    /// Coefficient of restitution $e \in [0.0 ..= 1.0]$.
    pub restitution: f32,
    /// Output buzz gain multiplier.
    pub buzz_gain: f32,
    /// Last computed contact force (N).
    pub last_contact_force: f32,
    /// Cumulative chatter collision events counter.
    pub collision_count: u32,
}

impl Default for TrompetteChienJunction {
    fn default() -> Self {
        Self::new(ChienMaterialProfile::MapleOnBone, 0.35)
    }
}

impl TrompetteChienJunction {
    pub fn new(profile: ChienMaterialProfile, clearance_gap_mm: f32) -> Self {
        let (stiff, damp, rest, bgain) = profile.nominal_physics();
        Self {
            profile,
            clearance_gap_mm: clearance_gap_mm.clamp(0.05, 1.20),
            displacement_mm: 0.0,
            velocity_mm_s: 0.0,
            contact_stiffness: stiff,
            contact_damping: damp,
            restitution: rest,
            buzz_gain: bgain,
            last_contact_force: 0.0,
            collision_count: 0,
        }
    }

    /// Set material profile and update contact physics parameters.
    pub fn set_profile(&mut self, profile: ChienMaterialProfile) {
        self.profile = profile;
        let (stiff, damp, rest, bgain) = profile.nominal_physics();
        self.contact_stiffness = stiff;
        self.contact_damping = damp;
        self.restitution = rest;
        self.buzz_gain = bgain;
    }

    /// Set resting clearance gap $h_0 \in [0.05 ..= 1.20]\text{ mm}$.
    pub fn set_clearance_gap(&mut self, gap_mm: f32) {
        self.clearance_gap_mm = gap_mm.clamp(0.05, 1.20);
    }

    /// Process a single time step of the buzzing dog bridge collision dynamics.
    ///
    /// The dog is driven by string displacement $y_{\text{string}}$ and wrist acceleration pulse $\Delta \alpha$.
    /// When total downward deflection exceeds clearance $h_0$, unilateral obstacle contact occurs:
    /// $$F_c = k_c \cdot (-(h_0 + y))^{1.5} + c_c \cdot \dot{y}$$
    ///
    /// Returns `(contact_force, buzzing_audio_sample, body_excitation)`.
    #[inline]
    pub fn process_collision(
        &mut self,
        string_displacement: f32,
        string_velocity: f32,
        wrist_acceleration: f32,
        sample_rate: u32,
    ) -> (f32, f32, f32) {
        let dt = 1.0 / (sample_rate as f32).max(1000.0);

        // String drives dog displacement with amplification from wrist acceleration impulse
        let drive_disp = string_displacement * (1.0 + 2.5 * wrist_acceleration.clamp(0.0, 5.0));
        let target_disp = drive_disp * 1.5; // mm scale factor

        // Mass-spring dog dynamics with damping
        let omega_dog = 2.0 * PI * 440.0; // Natural dog rocking frequency
        let accel = -omega_dog * omega_dog * (self.displacement_mm - target_disp)
            - 2.0 * 0.15 * omega_dog * self.velocity_mm_s
            + string_velocity * 12.0;

        self.velocity_mm_s += accel * dt;
        self.displacement_mm += self.velocity_mm_s * dt;

        // Unilateral obstacle contact check: striking plate is at -clearance_gap_mm
        let penetration = -self.clearance_gap_mm - self.displacement_mm;
        let mut contact_force = 0.0f32;
        let mut buzz_out = 0.0f32;

        if penetration > 0.0 {
            // Hertzian-like non-linear compression $F \propto \delta^{1.5}$
            let k_term = self.contact_stiffness * penetration.powf(1.5);
            let d_term = self.contact_damping * (-self.velocity_mm_s).max(0.0);
            contact_force = (k_term + d_term).max(0.0);

            // Rebound velocity reflection
            if self.velocity_mm_s < 0.0 {
                self.velocity_mm_s *= -self.restitution;
                self.displacement_mm = -self.clearance_gap_mm + 0.001;
                self.collision_count = self.collision_count.wrapping_add(1);
            }

            // Buzzing audio output pulse proportional to contact force
            buzz_out = (contact_force * 0.02 * self.buzz_gain).clamp(-2.0, 2.0);
        }

        self.last_contact_force = contact_force;
        let body_excitation = (contact_force * 0.05 + self.displacement_mm.abs() * 0.1).clamp(0.0, 2.0);

        (contact_force, buzz_out, body_excitation)
    }

    pub fn reset(&mut self) {
        self.displacement_mm = 0.0;
        self.velocity_mm_s = 0.0;
        self.last_contact_force = 0.0;
        self.collision_count = 0;
    }
}

/// A 2nd-order Biquad resonant filter for Hurdy-Gurdy soundbox modal modeling.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HurdyGurdyBodyMode {
    pub freq_hz: f32,
    pub q: f32,
    pub gain: f32,
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

impl HurdyGurdyBodyMode {
    pub fn new(freq_hz: f32, q: f32, gain: f32, sample_rate: u32) -> Self {
        let mut mode = Self {
            freq_hz,
            q: q.max(0.5),
            gain,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        };
        mode.recalculate(sample_rate);
        mode
    }

    pub fn recalculate(&mut self, sample_rate: u32) {
        let sr = sample_rate as f32;
        let omega = 2.0 * PI * (self.freq_hz / sr).clamp(0.001, 0.49);
        let sin_w = omega.sin();
        let cos_w = omega.cos();
        let alpha = sin_w / (2.0 * self.q);

        // Constant skirt gain bandpass filter
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
        let y = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2 - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }

    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
}

/// Hurdy-Gurdy guitar/lute-shaped wooden soundbox body cavity acoustic modal resonator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HurdyGurdySoundboxBody {
    pub modes: [HurdyGurdyBodyMode; NUM_HURDY_GURDY_BODY_MODES],
    pub master_resonance_gain: f32,
}

impl HurdyGurdySoundboxBody {
    pub fn new(sample_rate: u32) -> Self {
        // Authentic Hurdy-Gurdy spruce top & maple rib cavity resonant modes
        let modal_specs: [(f32, f32, f32); NUM_HURDY_GURDY_BODY_MODES] = [
            (145.0, 10.0, 1.25), // Mode 1: Main body Helmholtz resonance (C-soundholes air cavity)
            (215.0, 14.0, 1.10), // Mode 2: First top arched soundboard longitudinal flexure
            (290.0, 18.0, 0.95), // Mode 3: Back plate ribs breathing mode
            (380.0, 20.0, 0.85), // Mode 4: Second longitudinal soundboard mode
            (520.0, 24.0, 0.75), // Mode 5: Transverse soundbox flexure
            (740.0, 26.0, 0.65), // Mode 6: Upper soundboard harmonic mode
            (1050.0, 30.0, 0.50),// Mode 7: Friction wheel contact bridge peak
            (1550.0, 32.0, 0.40),// Mode 8: Chien buzzing transient harmonic ring
        ];

        let mut modes = [HurdyGurdyBodyMode::new(100.0, 10.0, 1.0, sample_rate); NUM_HURDY_GURDY_BODY_MODES];
        for (i, &(f, q, g)) in modal_specs.iter().enumerate() {
            modes[i] = HurdyGurdyBodyMode::new(f, q, g, sample_rate);
        }

        Self {
            modes,
            master_resonance_gain: 0.85,
        }
    }

    #[inline]
    pub fn process(&mut self, input: f32) -> f32 {
        let mut out = 0.0;
        for mode in self.modes.iter_mut() {
            out += mode.step(input);
        }
        out * self.master_resonance_gain
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        for mode in self.modes.iter_mut() {
            mode.recalculate(sample_rate);
        }
    }

    pub fn reset(&mut self) {
        for mode in self.modes.iter_mut() {
            mode.reset();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chien_clearance_and_obstacle_collision() {
        let mut junction = TrompetteChienJunction::new(ChienMaterialProfile::MapleOnBone, 0.30);
        assert_eq!(junction.clearance_gap_mm, 0.30);

        // Small displacement (< gap) -> No contact force
        let (force_small, buzz_small, _) = junction.process_collision(0.05, 0.0, 0.0, 48000);
        assert_eq!(force_small, 0.0);
        assert_eq!(buzz_small, 0.0);

        // Large negative displacement (> gap) -> Non-linear collision
        let mut has_collided = false;
        for _ in 0..100 {
            let (force_large, buzz_large, _) = junction.process_collision(-0.80, -2.0, 2.0, 48000);
            if force_large > 0.0 {
                has_collided = true;
                assert!(buzz_large.is_finite());
                break;
            }
        }
        assert!(has_collided, "Chien should collide under large negative displacement");
    }

    #[test]
    fn test_chien_material_profiles() {
        for prof in [
            ChienMaterialProfile::MapleOnBone,
            ChienMaterialProfile::IvoryPlate,
            ChienMaterialProfile::EbonyHardwood,
            ChienMaterialProfile::SyntheticDelrin,
            ChienMaterialProfile::VintageFruitwood,
        ] {
            let junction = TrompetteChienJunction::new(prof, 0.25);
            assert_eq!(junction.profile, prof);
            let (stiff, damp, rest, _) = prof.nominal_physics();
            assert!(stiff > 0.0);
            assert!(damp > 0.0);
            assert!((0.0..=1.0).contains(&rest));
        }
    }

    #[test]
    fn test_soundbox_modal_resonance_decay() {
        let mut body = HurdyGurdySoundboxBody::new(48000);
        let resp = body.process(1.0);
        assert!(resp.is_finite());
        assert!(resp.abs() > 0.0);

        for _ in 0..1000 {
            body.process(0.0);
        }

        let decayed = body.process(0.0);
        assert!(decayed.abs() < resp.abs(), "Modal body resonance must decay over time");
    }
}
