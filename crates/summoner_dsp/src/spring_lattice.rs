// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Physical Modeling 2D/3D Non-Linear Spring-Mass Lattice Synthesizer & Resonator (Milestone 20).
//!
//! Provides a physical 2D/3D interconnected grid of point masses coupled by non-linear Duffing
//! springs ($F = -k \Delta x - \beta (\Delta x)^3$), internal and grounded viscous damping
//! dissipation matrices, corner/edge boundary reflections, electromagnetic voice-coil driver
//! transducer excitation, and directional pickup transducers.
//!
//! Enforces zero heap allocations in audio processing loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};
use crate::traits::SignalProcessor;

/// Grid dimension for the 2D spring-mass lattice ($6 \times 6 = 36$ nodes).
pub const LATTICE_DIM: usize = 6;
/// Total number of point masses in the lattice grid.
pub const NUM_LATTICE_NODES: usize = LATTICE_DIM * LATTICE_DIM;

/// Preset mechanical spring-mass lattice profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SpringLatticeProfile {
    /// Classic Accutronics 2-spring / 3-spring helical reverb tank with chirpy dispersion and metallic boing.
    #[default]
    VintageAccutronicsSpring,
    /// High-tension studio suspension plate lattice with dense modal ringing and smooth decay.
    StudioPlateSuspension,
    /// Dual helical spring pair coupled with 180° out-of-phase transducers for wide stereo mechanical resonance.
    DualHelicalSpringTank,
    /// High-Q resonant helical spring coil with emphasized non-linear harmonic generation.
    ResonantHelicalCoil,
    /// High-displacement non-linear shaker table with strong Duffing cubic stiffness and sub-harmonic bifurcation.
    NonlinearShakerTable,
    /// Multi-axis 3D interconnected mass-spring lattice for complex structural acoustic modeling.
    MultiAxisSpringLattice,
}

impl SpringLatticeProfile {
    /// Returns default excitation fundamental frequency in Hz.
    pub fn default_fundamental_hz(&self) -> f32 {
        match self {
            Self::VintageAccutronicsSpring => 98.0,  // G2
            Self::StudioPlateSuspension => 164.81,  // E3
            Self::DualHelicalSpringTank => 110.0,   // A2
            Self::ResonantHelicalCoil => 220.0,     // A3
            Self::NonlinearShakerTable => 65.41,    // C2
            Self::MultiAxisSpringLattice => 130.81, // C3
        }
    }

    /// Returns nominal $(k_{\text{linear}}, \beta_{\text{duffing}}, \gamma_{\text{damping}}, \text{drive\_gain}, \text{boundary\_tension})$.
    pub fn nominal_physics(&self) -> (f32, f32, f32, f32, f32) {
        match self {
            Self::VintageAccutronicsSpring => (2800.0, 0.45, 0.008, 1.2, 0.80),
            Self::StudioPlateSuspension => (5500.0, 0.05, 0.003, 1.0, 0.95),
            Self::DualHelicalSpringTank => (3200.0, 0.35, 0.006, 1.1, 0.85),
            Self::ResonantHelicalCoil => (4200.0, 0.65, 0.002, 1.3, 0.90),
            Self::NonlinearShakerTable => (1800.0, 0.85, 0.015, 1.5, 0.60),
            Self::MultiAxisSpringLattice => (3800.0, 0.25, 0.005, 1.0, 0.75),
        }
    }

    /// Returns human-readable name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::VintageAccutronicsSpring => "Vintage Accutronics Spring Tank",
            Self::StudioPlateSuspension => "Studio Plate Suspension Lattice",
            Self::DualHelicalSpringTank => "Dual Helical Spring Reverb",
            Self::ResonantHelicalCoil => "Resonant Helical Coil Resonator",
            Self::NonlinearShakerTable => "Non-Linear Shaker Table",
            Self::MultiAxisSpringLattice => "Multi-Axis Spring-Mass Lattice",
        }
    }
}

/// A point mass node in the 2D/3D elastic lattice.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LatticePointMass {
    /// Displacement along the vibration axis $z$.
    pub displacement: f32,
    /// Instantaneous velocity $v = \dot{z}$.
    pub velocity: f32,
    /// Net restoring and external force $F_{\text{net}}$.
    pub force: f32,
    /// Point mass value $m > 0$.
    pub mass: f32,
    /// Grounded dissipation damping coefficient $\gamma$.
    pub damping: f32,
    /// Whether this node is clamped / fixed to the boundary frame.
    pub is_boundary: bool,
}

impl Default for LatticePointMass {
    fn default() -> Self {
        Self {
            displacement: 0.0,
            velocity: 0.0,
            force: 0.0,
            mass: 1.0,
            damping: 0.005,
            is_boundary: false,
        }
    }
}

fn default_lattice_nodes() -> [LatticePointMass; NUM_LATTICE_NODES] {
    [LatticePointMass {
        displacement: 0.0,
        velocity: 0.0,
        force: 0.0,
        mass: 1.0,
        damping: 0.005,
        is_boundary: false,
    }; NUM_LATTICE_NODES]
}

/// Physical modeling non-linear spring-mass lattice synthesizer & resonator engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpringLattice {
    pub sample_rate: u32,
    pub profile: SpringLatticeProfile,
    pub fundamental_hz: f32,
    /// Linear spring stiffness coefficient $k > 0$.
    pub linear_stiffness: f32,
    /// Non-linear Duffing cubic stiffness coefficient $\beta \ge 0$.
    pub duffing_nonlinearity: f32,
    /// Inter-node viscous damping ratio.
    pub inter_node_damping: f32,
    /// Grounded dissipation damping ratio.
    pub ground_damping: f32,
    /// Boundary reflection tension ratio $[0.0 ..= 1.0]$.
    pub boundary_tension: f32,
    /// Electromagnetic driver force scaling.
    pub drive_force: f32,
    /// Driver input node coordinates $(x, y)$.
    pub driver_pos: (usize, usize),
    /// Left pickup transducer node coordinates $(x, y)$.
    pub pickup_l_pos: (usize, usize),
    /// Right pickup transducer node coordinates $(x, y)$.
    pub pickup_r_pos: (usize, usize),
    /// Transducer pickup angle in radians $[0.0 ..= 2\pi]$.
    pub pickup_angle: f32,
    /// Point mass nodes on the $6 \times 6$ lattice grid.
    #[serde(skip, default = "default_lattice_nodes")]
    pub nodes: [LatticePointMass; NUM_LATTICE_NODES],
    // Mechanical strike impulse state
    strike_pulse_timer: f32,
    strike_pulse_duration: f32,
    strike_pulse_amplitude: f32,
    is_striking: bool,
    // Output DC blocking filter states
    dc_block_x1: f32,
    dc_block_y1: f32,
}

impl SpringLattice {
    /// Creates a new SpringLattice physical modeling synthesizer voice.
    pub fn new(sample_rate: u32) -> Self {
        let sr = sample_rate.max(8000);
        let prof = SpringLatticeProfile::VintageAccutronicsSpring;
        let fund = prof.default_fundamental_hz();
        let (k, beta, gamma, drive, tension) = prof.nominal_physics();

        let mut nodes = [LatticePointMass::default(); NUM_LATTICE_NODES];
        for y in 0..LATTICE_DIM {
            for x in 0..LATTICE_DIM {
                let idx = y * LATTICE_DIM + x;
                let is_edge = x == 0 || x == LATTICE_DIM - 1 || y == 0 || y == LATTICE_DIM - 1;
                nodes[idx].is_boundary = is_edge;
                nodes[idx].damping = gamma;
            }
        }

        let mut lattice = Self {
            sample_rate: sr,
            profile: prof,
            fundamental_hz: fund,
            linear_stiffness: k,
            duffing_nonlinearity: beta,
            inter_node_damping: 0.002,
            ground_damping: gamma,
            boundary_tension: tension,
            drive_force: drive,
            driver_pos: (1, 1),
            pickup_l_pos: (4, 4),
            pickup_r_pos: (4, 1),
            pickup_angle: 0.0,
            nodes,
            strike_pulse_timer: 0.0,
            strike_pulse_duration: 0.0025,
            strike_pulse_amplitude: 0.0,
            is_striking: false,
            dc_block_x1: 0.0,
            dc_block_y1: 0.0,
        };

        lattice.update_physics();
        lattice
    }

    /// Sets the active profile and updates nominal physical parameters.
    pub fn set_profile(&mut self, profile: SpringLatticeProfile) {
        self.profile = profile;
        self.fundamental_hz = profile.default_fundamental_hz();
        let (k, beta, gamma, drive, tension) = profile.nominal_physics();
        self.linear_stiffness = k;
        self.duffing_nonlinearity = beta;
        self.ground_damping = gamma;
        self.drive_force = drive;
        self.boundary_tension = tension;
        self.update_physics();
    }

    /// Sets fundamental pitch frequency in Hz and scales lattice linear stiffness.
    pub fn set_frequency(&mut self, freq_hz: f32) {
        self.fundamental_hz = freq_hz.clamp(20.0, 4000.0);
        self.update_physics();
    }

    /// Updates internal lattice stiffness and node parameters.
    pub fn update_physics(&mut self) {
        // Linear stiffness scales with fundamental frequency squared: k ~ (2 * pi * f0)^2 * m_eff
        let omega = 2.0 * PI * self.fundamental_hz;
        let base_k = (omega * omega * 0.012).clamp(200.0, 18000.0);
        self.linear_stiffness = base_k * (0.8 + 0.4 * self.boundary_tension);

        for y in 0..LATTICE_DIM {
            for x in 0..LATTICE_DIM {
                let idx = y * LATTICE_DIM + x;
                let is_edge = x == 0 || x == LATTICE_DIM - 1 || y == 0 || y == LATTICE_DIM - 1;
                self.nodes[idx].is_boundary = is_edge && (self.boundary_tension > 0.85);
                self.nodes[idx].damping = self.ground_damping;
            }
        }
    }

    /// Triggers an external mechanical strike / excitation impulse.
    pub fn trigger_strike(&mut self, velocity: f32, hardness: f32) {
        let vel = velocity.clamp(0.01, 1.0);
        let hard = hardness.clamp(0.0, 1.0);

        self.strike_pulse_duration = (0.0040 * (1.0 - 0.75 * hard)).clamp(0.0002, 0.010);
        self.strike_pulse_timer = 0.0;
        self.strike_pulse_amplitude = vel * self.drive_force * 2500.0;
        self.is_striking = true;
    }

    /// Resets all internal node states.
    pub fn reset(&mut self) {
        for node in &mut self.nodes {
            node.displacement = 0.0;
            node.velocity = 0.0;
            node.force = 0.0;
        }
        self.strike_pulse_timer = 0.0;
        self.is_striking = false;
        self.dc_block_x1 = 0.0;
        self.dc_block_y1 = 0.0;
    }

    /// Evaluates one audio sample time-step through the non-linear spring-mass grid.
    pub fn process_sample(&mut self, external_excitation: f32) -> (f32, f32) {
        let dt = 1.0 / (self.sample_rate as f32);

        // Sub-step count for numerical stability with high stiffness
        const SUB_STEPS: usize = 2;
        let sub_dt = dt / (SUB_STEPS as f32);

        // 1. Generate mechanical strike pulse
        let strike_force = if self.is_striking {
            self.strike_pulse_timer += dt;
            if self.strike_pulse_timer >= self.strike_pulse_duration {
                self.is_striking = false;
                0.0
            } else {
                let phase = (PI * self.strike_pulse_timer / self.strike_pulse_duration).clamp(0.0, PI);
                phase.sin().powf(1.6) * self.strike_pulse_amplitude
            }
        } else {
            0.0
        };

        let total_drive = strike_force + external_excitation * self.drive_force * 1500.0;

        // 2. Multi-step symplectic numerical integration
        for _ in 0..SUB_STEPS {
            // Reset net forces
            for node in &mut self.nodes {
                node.force = 0.0;
            }

            // Apply driver transducer force to driver node
            let driver_idx = self.driver_pos.1 * LATTICE_DIM + self.driver_pos.0;
            if driver_idx < NUM_LATTICE_NODES {
                self.nodes[driver_idx].force += total_drive;
            }

            // Compute inter-node non-linear spring and damping forces
            for y in 0..LATTICE_DIM {
                for x in 0..LATTICE_DIM {
                    let idx = y * LATTICE_DIM + x;
                    let z_curr = self.nodes[idx].displacement;
                    let v_curr = self.nodes[idx].velocity;

                    // Right neighbor (x + 1)
                    if x + 1 < LATTICE_DIM {
                        let r_idx = y * LATTICE_DIM + (x + 1);
                        let dz = z_curr - self.nodes[r_idx].displacement;
                        let dv = v_curr - self.nodes[r_idx].velocity;

                        // Duffing non-linear restoring force: F = -k * dz - beta * dz^3
                        let f_spring = -self.linear_stiffness * dz - self.duffing_nonlinearity * 12000.0 * dz.powi(3);
                        let f_damping = -self.inter_node_damping * 80.0 * dv;
                        let f_total = f_spring + f_damping;

                        self.nodes[idx].force += f_total;
                        self.nodes[r_idx].force -= f_total;
                    }

                    // Bottom neighbor (y + 1)
                    if y + 1 < LATTICE_DIM {
                        let b_idx = (y + 1) * LATTICE_DIM + x;
                        let dz = z_curr - self.nodes[b_idx].displacement;
                        let dv = v_curr - self.nodes[b_idx].velocity;

                        let f_spring = -self.linear_stiffness * dz - self.duffing_nonlinearity * 12000.0 * dz.powi(3);
                        let f_damping = -self.inter_node_damping * 80.0 * dv;
                        let f_total = f_spring + f_damping;

                        self.nodes[idx].force += f_total;
                        self.nodes[b_idx].force -= f_total;
                    }

                    // Grounded viscous dissipation damping
                    self.nodes[idx].force -= self.nodes[idx].damping * 150.0 * v_curr;
                }
            }

            // Update velocities and displacements (Symplectic Euler)
            for node in &mut self.nodes {
                if !node.is_boundary {
                    let accel = node.force / node.mass.max(0.1);
                    node.velocity += accel * sub_dt;
                    // Velocity clamping for numerical safety under extreme chaos
                    node.velocity = node.velocity.clamp(-150.0, 150.0);
                    node.displacement += node.velocity * sub_dt;
                    node.displacement = node.displacement.clamp(-1.0, 1.0);
                } else {
                    node.displacement = 0.0;
                    node.velocity = 0.0;
                }
            }
        }

        // 3. Readout pickup transducers
        let p_l_idx = self.pickup_l_pos.1 * LATTICE_DIM + self.pickup_l_pos.0;
        let p_r_idx = self.pickup_r_pos.1 * LATTICE_DIM + self.pickup_r_pos.0;

        let raw_l = if p_l_idx < NUM_LATTICE_NODES {
            self.nodes[p_l_idx].velocity * 0.12 + self.nodes[p_l_idx].displacement * 0.88
        } else {
            0.0
        };

        let raw_r = if p_r_idx < NUM_LATTICE_NODES {
            self.nodes[p_r_idx].velocity * 0.12 + self.nodes[p_r_idx].displacement * 0.88
        } else {
            0.0
        };

        // Directional pickup angle rotation
        let cos_a = self.pickup_angle.cos();
        let sin_a = self.pickup_angle.sin();
        let out_l = raw_l * cos_a - raw_r * sin_a;
        let out_r = raw_l * sin_a + raw_r * cos_a;

        // DC block filter on Left output
        let dc_blocked = out_l - self.dc_block_x1 + 0.995 * self.dc_block_y1;
        self.dc_block_x1 = out_l;
        self.dc_block_y1 = if dc_blocked.is_finite() { dc_blocked } else { 0.0 };

        (self.dc_block_y1.clamp(-1.0, 1.0), out_r.clamp(-1.0, 1.0))
    }
}

impl SignalProcessor for SpringLattice {
    fn name(&self) -> &str {
        "SpringLattice"
    }

    fn process_block(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        _ctx: &ProcessContext,
    ) {
        if outputs.is_empty() {
            return;
        }

        let num_samples = outputs[0].len();
        for i in 0..num_samples {
            let excitation = if !inputs.is_empty() && !inputs[0].is_empty() && i < inputs[0].len() {
                inputs[0][i]
            } else {
                0.0
            };

            let (out_l, out_r) = self.process_sample(excitation);
            if !outputs.is_empty() && i < outputs[0].len() {
                outputs[0][i] = out_l;
            }
            if outputs.len() > 1 && i < outputs[1].len() {
                outputs[1][i] = out_r;
            }
        }
    }
}

/// AudioNode wrapper for Spring-Mass Lattice synthesis & resonance node.
#[derive(Debug)]
pub struct SpringLatticeNode {
    pub lattice: SpringLattice,
}

impl SpringLatticeNode {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            lattice: SpringLattice::new(sample_rate),
        }
    }
}

impl AudioNode for SpringLatticeNode {
    fn name(&self) -> &str {
        "SpringLatticeNode"
    }

    fn process(
        &mut self,
        inputs: &[&[Sample]],
        outputs: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        self.lattice.process_block(inputs, outputs, ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::allocator::AllocGuard;

    #[test]
    fn test_spring_lattice_strike_decay_and_sound() {
        let mut spring = SpringLattice::new(48000);
        spring.set_profile(SpringLatticeProfile::VintageAccutronicsSpring);
        spring.set_frequency(98.0);
        spring.trigger_strike(0.90, 0.50);

        let mut peak_amp = 0.0f32;
        for _ in 0..12000 {
            let (l, r) = spring.process_sample(0.0);
            assert!(l.is_finite());
            assert!(r.is_finite());
            assert!((-1.0..=1.0).contains(&l));
            assert!((-1.0..=1.0).contains(&r));
            peak_amp = peak_amp.max(l.abs().max(r.abs()));
        }

        for _ in 0..48000 {
            let (l, r) = spring.process_sample(0.0);
            assert!(l.is_finite());
            assert!(r.is_finite());
        }

        let mut late_amp = 0.0f32;
        for _ in 0..2000 {
            let (l, r) = spring.process_sample(0.0);
            assert!(l.is_finite());
            assert!(r.is_finite());
            late_amp = late_amp.max(l.abs().max(r.abs()));
        }

        assert!(peak_amp > 0.01, "Spring lattice strike must produce sound");
        assert!(late_amp < peak_amp, "Spring lattice oscillation must naturally decay");
    }

    #[test]
    fn test_spring_lattice_zero_allocation_in_loop() {
        let mut spring = SpringLattice::new(48000);
        spring.trigger_strike(0.85, 0.50);

        let mut out_l = [0.0f32; 256];
        let mut out_r = [0.0f32; 256];
        let in_buf = [0.0f32; 256];
        let ctx = ProcessContext::new(48000, 120.0, 0);

        let _guard = AllocGuard::new();
        for _ in 0..10 {
            spring.process_block(&[&in_buf[..]], &mut [&mut out_l[..], &mut out_r[..]], &ctx);
        }
    }
}
