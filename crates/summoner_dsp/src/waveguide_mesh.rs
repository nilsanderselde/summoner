// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! 2D Physical Waveguide Resonator Mesh Modeler (Milestone 13).
//!
//! Provides a 2D finite-difference / scattering junction physical waveguide mesh
//! supporting membrane, plate, and acoustic bar geometries with configurable boundary
//! conditions (Clamped, Free, DampedAbsorption), Courant-stable wave propagation,
//! spatial strike excitation, and bilinear pickup tapping.
//!
//! Enforces zero heap allocations in audio processing loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use summoner_core::audio::Sample;
use summoner_core::node::{AudioNode, ProcessContext};
use crate::traits::SignalProcessor;

/// Fixed dimension for the 2D waveguide mesh grid (16x16 = 256 nodes).
pub const MESH_DIM: usize = 16;

/// Boundary edge condition for the 2D physical mesh.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeshBoundaryType {
    /// Zero displacement at edges (drum head rim / clamped plate).
    Clamped,
    /// Free displacement with zero spatial normal gradient (free plate / chime).
    Free,
    /// Absorptive boundary damping wave reflections.
    DampedAbsorption,
}

impl Default for MeshBoundaryType {
    fn default() -> Self {
        Self::Clamped
    }
}

/// Geometry and physical material profile of the resonator mesh.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeshMaterialProfile {
    /// Elastic circular/rectangular membrane (e.g. tympani, tom-tom).
    Membrane,
    /// Stiff metallic plate with inharmonic modal dispersion (e.g. gong, cymbal).
    Plate,
    /// 1D acoustic bar / marimba bar geometry.
    AcousticBar,
}

impl Default for MeshMaterialProfile {
    fn default() -> Self {
        Self::Membrane
    }
}

/// 2D Physical Waveguide Resonator Mesh.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveguideMesh2D {
    #[serde(skip)]
    u_curr: [[f32; MESH_DIM]; MESH_DIM],
    #[serde(skip)]
    u_prev: [[f32; MESH_DIM]; MESH_DIM],
    #[serde(skip)]
    u_next: [[f32; MESH_DIM]; MESH_DIM],
    /// Courant wave propagation speed parameter (strictly <= 0.7071 for 2D CFL stability).
    pub courant: f32,
    /// Damping factor per simulation step.
    pub damping: f32,
    /// Boundary reflection condition.
    pub boundary: MeshBoundaryType,
    /// Material profile.
    pub material: MeshMaterialProfile,
    /// Normalized strike excitation coordinate X [0.0 ..= 1.0].
    pub strike_x: f32,
    /// Normalized strike excitation coordinate Y [0.0 ..= 1.0].
    pub strike_y: f32,
    /// Normalized pickup tapping coordinate X [0.0 ..= 1.0].
    pub pickup_x: f32,
    /// Normalized pickup tapping coordinate Y [0.0 ..= 1.0].
    pub pickup_y: f32,
    /// Sample rate.
    pub sample_rate: u32,
}

impl Default for WaveguideMesh2D {
    fn default() -> Self {
        Self::new(48000)
    }
}

impl WaveguideMesh2D {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            u_curr: [[0.0; MESH_DIM]; MESH_DIM],
            u_prev: [[0.0; MESH_DIM]; MESH_DIM],
            u_next: [[0.0; MESH_DIM]; MESH_DIM],
            courant: 0.50,
            damping: 0.002,
            boundary: MeshBoundaryType::Clamped,
            material: MeshMaterialProfile::Membrane,
            strike_x: 0.50,
            strike_y: 0.50,
            pickup_x: 0.40,
            pickup_y: 0.60,
            sample_rate: sample_rate.max(8000),
        }
    }

    /// Sets material profile and updates physical constants.
    pub fn set_material(&mut self, material: MeshMaterialProfile) {
        self.material = material;
        match material {
            MeshMaterialProfile::Membrane => {
                self.courant = 0.50;
                self.damping = 0.0015;
                self.boundary = MeshBoundaryType::Clamped;
            }
            MeshMaterialProfile::Plate => {
                self.courant = 0.65;
                self.damping = 0.0008;
                self.boundary = MeshBoundaryType::Free;
            }
            MeshMaterialProfile::AcousticBar => {
                self.courant = 0.45;
                self.damping = 0.003;
                self.boundary = MeshBoundaryType::DampedAbsorption;
            }
        }
    }

    /// Strikes the mesh with an impulse force at normalized coordinates (x, y).
    pub fn strike(&mut self, x_norm: f32, y_norm: f32, amplitude: f32, radius: f32) {
        let cx = x_norm.clamp(0.05, 0.95) * (MESH_DIM - 1) as f32;
        let cy = y_norm.clamp(0.05, 0.95) * (MESH_DIM - 1) as f32;
        let rad = radius.max(0.5);

        for y in 0..MESH_DIM {
            for x in 0..MESH_DIM {
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist <= rad {
                    let env = (1.0 - dist / rad).clamp(0.0, 1.0);
                    let force = amplitude * env * env;
                    self.u_curr[y][x] += force;
                    self.u_prev[y][x] += force * 0.5;
                }
            }
        }
    }

    /// Resets all displacement states to zero.
    pub fn reset(&mut self) {
        self.u_curr = [[0.0; MESH_DIM]; MESH_DIM];
        self.u_prev = [[0.0; MESH_DIM]; MESH_DIM];
        self.u_next = [[0.0; MESH_DIM]; MESH_DIM];
    }

    /// Computes one discrete time step of the 2D finite-difference wave equation.
    #[inline]
    pub fn step_simulation(&mut self) {
        // Courant squared must not exceed 0.5 for 2D standard 5-point stencil stability
        let c_clamped = self.courant.clamp(0.05, 0.7071);
        let c2 = c_clamped * c_clamped;
        let d = self.damping.clamp(0.00001, 0.1);
        let decay = (1.0 - d).max(0.0);

        for y in 0..MESH_DIM {
            for x in 0..MESH_DIM {
                // Determine neighboring values according to boundary conditions
                let left = if x == 0 {
                    match self.boundary {
                        MeshBoundaryType::Clamped => 0.0,
                        MeshBoundaryType::Free => self.u_curr[y][1],
                        MeshBoundaryType::DampedAbsorption => self.u_curr[y][0] * 0.7,
                    }
                } else {
                    self.u_curr[y][x - 1]
                };

                let right = if x == MESH_DIM - 1 {
                    match self.boundary {
                        MeshBoundaryType::Clamped => 0.0,
                        MeshBoundaryType::Free => self.u_curr[y][MESH_DIM - 2],
                        MeshBoundaryType::DampedAbsorption => self.u_curr[y][MESH_DIM - 1] * 0.7,
                    }
                } else {
                    self.u_curr[y][x + 1]
                };

                let top = if y == 0 {
                    match self.boundary {
                        MeshBoundaryType::Clamped => 0.0,
                        MeshBoundaryType::Free => self.u_curr[1][x],
                        MeshBoundaryType::DampedAbsorption => self.u_curr[0][x] * 0.7,
                    }
                } else {
                    self.u_curr[y - 1][x]
                };

                let bottom = if y == MESH_DIM - 1 {
                    match self.boundary {
                        MeshBoundaryType::Clamped => 0.0,
                        MeshBoundaryType::Free => self.u_curr[MESH_DIM - 2][x],
                        MeshBoundaryType::DampedAbsorption => self.u_curr[MESH_DIM - 1][x] * 0.7,
                    }
                } else {
                    self.u_curr[y + 1][x]
                };

                let laplacian = left + right + top + bottom - 4.0 * self.u_curr[y][x];
                let next_val = (2.0 * self.u_curr[y][x] - self.u_prev[y][x] + c2 * laplacian) * decay;
                self.u_next[y][x] = next_val.clamp(-10.0, 10.0);
            }
        }

        // Rotate state buffers
        self.u_prev = self.u_curr;
        self.u_curr = self.u_next;
    }

    /// Reads output sample via bilinear interpolation at normalized position (px, py).
    #[inline]
    pub fn sample_pickup(&self, px_norm: f32, py_norm: f32) -> f32 {
        let gx = (px_norm.clamp(0.0, 1.0) * (MESH_DIM - 1) as f32).clamp(0.0, (MESH_DIM - 1) as f32);
        let gy = (py_norm.clamp(0.0, 1.0) * (MESH_DIM - 1) as f32).clamp(0.0, (MESH_DIM - 1) as f32);

        let x0 = (gx.floor() as usize).min(MESH_DIM - 1);
        let x1 = (x0 + 1).min(MESH_DIM - 1);
        let y0 = (gy.floor() as usize).min(MESH_DIM - 1);
        let y1 = (y0 + 1).min(MESH_DIM - 1);

        let fx = gx - x0 as f32;
        let fy = gy - y0 as f32;

        let v00 = self.u_curr[y0][x0];
        let v10 = self.u_curr[y0][x1];
        let v01 = self.u_curr[y1][x0];
        let v11 = self.u_curr[y1][x1];

        let top = v00 * (1.0 - fx) + v10 * fx;
        let bot = v01 * (1.0 - fx) + v11 * fx;

        (top * (1.0 - fy) + bot * fy).clamp(-1.0, 1.0)
    }

    /// Computes and returns the next output sample.
    #[inline]
    pub fn process_sample(&mut self, excitation: f32) -> Sample {
        if excitation.abs() > 1e-5 {
            self.strike(self.strike_x, self.strike_y, excitation, 1.5);
        }
        self.step_simulation();
        self.sample_pickup(self.pickup_x, self.pickup_y)
    }

    /// Gets current 2D displacement grid reference for visual inspection.
    pub fn displacement_grid(&self) -> &[[f32; MESH_DIM]; MESH_DIM] {
        &self.u_curr
    }

    /// Calculates total instantaneous mechanical kinetic and potential energy of the mesh.
    pub fn calculate_total_energy(&self) -> f32 {
        let mut total_energy = 0.0f32;
        for y in 0..MESH_DIM {
            for x in 0..MESH_DIM {
                let disp = self.u_curr[y][x];
                let vel = self.u_curr[y][x] - self.u_prev[y][x];
                total_energy += disp * disp + vel * vel;
            }
        }
        total_energy
    }
}

impl SignalProcessor for WaveguideMesh2D {
    fn name(&self) -> &str {
        "WaveguideMesh2D"
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

            let out_sample = self.process_sample(excitation);
            for ch in outputs.iter_mut() {
                if i < ch.len() {
                    ch[i] = out_sample;
                }
            }
        }
    }
}

/// AudioNode wrapper for 2D Waveguide Resonator Mesh.
#[derive(Debug)]
pub struct WaveguideMeshNode {
    pub mesh: WaveguideMesh2D,
}

impl WaveguideMeshNode {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            mesh: WaveguideMesh2D::new(sample_rate),
        }
    }
}

impl AudioNode for WaveguideMeshNode {
    fn name(&self) -> &str {
        "WaveguideMeshNode"
    }

    fn process(
        &mut self,
        input: &[&[Sample]],
        output: &mut [&mut [Sample]],
        ctx: &ProcessContext,
    ) {
        self.mesh.process_block(input, output, ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use summoner_core::allocator::AllocGuard;
    use summoner_core::transport::Transport;

    #[test]
    fn test_waveguide_mesh_stability_and_energy_decay() {
        let mut mesh = WaveguideMesh2D::new(48000);
        mesh.strike(0.5, 0.5, 1.0, 1.5);

        let initial_energy = mesh.calculate_total_energy();
        assert!(initial_energy > 0.0);

        for _ in 0..1000 {
            mesh.step_simulation();
            let sample = mesh.sample_pickup(0.5, 0.5);
            assert!(sample.is_finite());
            assert!(sample >= -1.0 && sample <= 1.0);
        }

        let decayed_energy = mesh.calculate_total_energy();
        assert!(decayed_energy < initial_energy);
    }

    #[test]
    fn test_waveguide_mesh_zero_allocation_in_process_block() {
        let mut mesh = WaveguideMesh2D::new(48000);
        mesh.strike(0.4, 0.6, 0.8, 1.2);

        let transport = Transport::new(48000, 120.0);
        let ctx = ProcessContext::from_transport(&transport);

        let in_buf = [0.0f32; 64];
        let mut out_l = [0.0f32; 64];
        let mut out_r = [0.0f32; 64];

        {
            let _guard = AllocGuard::new();
            mesh.process_block(
                &[&in_buf[..]],
                &mut [&mut out_l[..], &mut out_r[..]],
                &ctx,
            );
        }

        assert!(out_l.iter().any(|s| *s != 0.0));
        assert!(out_l.iter().all(|s| s.is_finite()));
    }
}
