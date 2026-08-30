// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Discrete Tonehole Lattice Acoustic Radiation & Scattering Junctions (Milestone 17).
//!
//! Provides an acoustic model of discrete tonehole lattices along a woodwind waveguide bore,
//! calculating acoustic impedance discontinuities, scattering junctions (reflection and transmission
//! coefficients), highpass radiation characteristics, and tonehole lattice cutoff frequencies.
//!
//! Enforces zero heap allocations in audio processing loops under `AllocGuard`.

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

/// Number of discrete toneholes modeled along the woodwind acoustic bore.
pub const NUM_TONEHOLES: usize = 6;

/// Speed of sound in air in meters per second (at 20°C).
pub const SPEED_OF_SOUND_MPS: f32 = 343.2;

/// Ambient air density in kg/m^3 (at 20°C, 1 atm).
pub const AIR_DENSITY_KG_M3: f32 = 1.204;

/// Physical geometry and acoustic parameters for a single tonehole.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ToneholeParams {
    /// Relative fractional position along the acoustic bore [0.0 ..= 1.0].
    pub position_ratio: f32,
    /// Tonehole inner radius in meters (typical 2.0 .. 7.0 mm).
    pub hole_radius_m: f32,
    /// Tonehole chimney height / wall thickness in meters (typical 1.5 .. 5.0 mm).
    pub chimney_height_m: f32,
    /// Acoustic bore inner radius at this tonehole location in meters (typical 6.0 .. 12.0 mm).
    pub bore_radius_m: f32,
}

impl Default for ToneholeParams {
    fn default() -> Self {
        Self {
            position_ratio: 0.5,
            hole_radius_m: 0.0045,      // 4.5 mm
            chimney_height_m: 0.0025,   // 2.5 mm wall thickness
            bore_radius_m: 0.0095,      // 9.5 mm bore radius (Flute C)
        }
    }
}

/// Instantaneous acoustic scattering result for a single tonehole junction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToneholeScatteringResult {
    /// Reflected pressure wave travelling backward upstream toward mouthpiece $p_1^-$.
    pub reflected_upstream: f32,
    /// Transmitted pressure wave travelling forward downstream toward bell $p_2^-$.
    pub transmitted_downstream: f32,
    /// Radiated acoustic pressure emitted into free air $p_{rad}$.
    pub radiated_pressure: f32,
    /// Current shunt reflection coefficient $r_s$.
    pub reflection_coeff: f32,
    /// Current shunt transmission coefficient $t_s$.
    pub transmission_coeff: f32,
}

/// Acoustic 3-port scattering junction representing a single tonehole on the bore.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ToneholeJunction {
    /// Physical geometric parameters.
    pub params: ToneholeParams,
    /// Discrete open/closed state (true = closed, false = open).
    pub is_closed: bool,
    /// Continuous opening fraction [0.0 = fully closed ..= 1.0 = fully open] for half-holing.
    pub open_fraction: f32,
    /// Previous state for radiation differentiation filter.
    prev_shunt_flow: f32,
    /// Previous radiated sample for highpass radiation filter.
    prev_rad_sample: f32,
}

impl Default for ToneholeJunction {
    fn default() -> Self {
        Self {
            params: ToneholeParams::default(),
            is_closed: true,
            open_fraction: 0.0,
            prev_shunt_flow: 0.0,
            prev_rad_sample: 0.0,
        }
    }
}

impl ToneholeJunction {
    /// Creates a new tonehole junction with specified geometry and position.
    pub fn new(position_ratio: f32, hole_radius_m: f32, chimney_height_m: f32, bore_radius_m: f32) -> Self {
        Self {
            params: ToneholeParams {
                position_ratio: position_ratio.clamp(0.05, 0.95),
                hole_radius_m: hole_radius_m.clamp(0.001, 0.020),
                chimney_height_m: chimney_height_m.clamp(0.0005, 0.015),
                bore_radius_m: bore_radius_m.clamp(0.003, 0.030),
            },
            is_closed: true,
            open_fraction: 0.0,
            prev_shunt_flow: 0.0,
            prev_rad_sample: 0.0,
        }
    }

    /// Sets the discrete open/closed state.
    #[inline]
    pub fn set_closed(&mut self, closed: bool) {
        self.is_closed = closed;
        self.open_fraction = if closed { 0.0 } else { 1.0 };
    }

    /// Sets continuous opening fraction for microtonal shading / half-holing [0.0 ..= 1.0].
    #[inline]
    pub fn set_open_fraction(&mut self, fraction: f32) {
        let f = fraction.clamp(0.0, 1.0);
        self.open_fraction = f;
        self.is_closed = f < 0.01;
    }

    /// Calculates tonehole acoustic shunt impedance ratio $z_s = Z_s / Z_0$.
    #[inline]
    pub fn compute_shunt_impedance_ratio(&self) -> f32 {
        if self.open_fraction < 0.001 {
            // Closed tonehole: high acoustic impedance with capacitive shunting (small negative inertance)
            50.0
        } else {
            // Open tonehole: impedance proportional to chimney length and inversely proportional to open area
            let area_ratio = (self.params.hole_radius_m / self.params.bore_radius_m).powi(2);
            let effective_chimney = self.params.chimney_height_m + 0.8 * self.params.hole_radius_m;
            let z_open = (effective_chimney / (self.params.hole_radius_m * 2.0 * area_ratio.max(0.05)))
                / self.open_fraction.max(0.01);
            z_open.clamp(0.05, 50.0)
        }
    }

    /// Evaluates 3-port acoustic scattering for incoming forward wave $p_1^+$ and backward wave $p_2^+$.
    #[inline]
    pub fn process_scattering(&mut self, p1_fwd_in: f32, p2_rev_in: f32, damping: f32) -> ToneholeScatteringResult {
        let z_ratio = self.compute_shunt_impedance_ratio();

        // 3-port acoustic scattering matrix coefficients:
        // r_s = -1 / (2 * z_ratio + 1) [negative reflection when open]
        // t_s = 2 * z_ratio / (2 * z_ratio + 1) = 1 + r_s [transmission]
        let denom = 2.0 * z_ratio + 1.0;
        let r_s = -1.0 / denom;
        let t_s = (2.0 * z_ratio) / denom;

        // Wave scattering equations:
        // p_1^- = r_s * p_1^+ + t_s * p_2^+
        // p_2^- = t_s * p_1^+ + r_s * p_2^+
        let reflected_upstream = (r_s * p1_fwd_in + t_s * p2_rev_in) * damping;
        let transmitted_downstream = (t_s * p1_fwd_in + r_s * p2_rev_in) * damping;

        // Acoustic shunt flow $U_{hole} = (p_1^+ + p_2^+ - (p_1^- + p_2^-)) / (2 * Z_0)$
        let junction_pressure = (p1_fwd_in + p2_rev_in) * (1.0 + r_s);
        let shunt_flow = junction_pressure / (z_ratio + 0.5);

        // Radiation differentiator (spherical acoustic dipole radiation into free space):
        // p_rad(t) ~ d/dt (U_hole(t))
        let rad_diff = shunt_flow - self.prev_shunt_flow;
        self.prev_shunt_flow = shunt_flow;

        // Highpass radiation filter
        let alpha = 0.88;
        let radiated = alpha * (self.prev_rad_sample + rad_diff);
        self.prev_rad_sample = radiated;

        let radiated_pressure = if self.open_fraction > 0.001 {
            (radiated * self.open_fraction * 1.5).clamp(-1.0, 1.0)
        } else {
            0.0
        };

        ToneholeScatteringResult {
            reflected_upstream,
            transmitted_downstream,
            radiated_pressure,
            reflection_coeff: r_s,
            transmission_coeff: t_s,
        }
    }

    /// Resets internal filter states to zero.
    pub fn reset(&mut self) {
        self.prev_shunt_flow = 0.0;
        self.prev_rad_sample = 0.0;
    }
}

/// A complete 6-tonehole lattice acoustic network along a woodwind waveguide bore.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToneholeLattice {
    /// Array of 6 discrete tonehole scattering junctions.
    pub toneholes: [ToneholeJunction; NUM_TONEHOLES],
    /// Nominal total bore length in meters.
    pub total_bore_length_m: f32,
    /// Nominal acoustic bore radius in meters.
    pub bore_radius_m: f32,
    /// Lattice radiation cutoff frequency in Hz.
    pub lattice_cutoff_hz: f32,
    /// Wall viscous-thermal damping loss per junction.
    pub junction_loss: f32,
}

impl Default for ToneholeLattice {
    fn default() -> Self {
        Self::new_standard_flute(0.60)
    }
}

impl ToneholeLattice {
    /// Creates a standard 6-tonehole lattice configuration for a Concert C Flute (~0.60 m).
    pub fn new_standard_flute(bore_length_m: f32) -> Self {
        let length = bore_length_m.clamp(0.20, 1.50);
        let bore_r = 0.0095; // 9.5 mm

        // Standard tonehole spacing fractions along the bore (holes 1 to 6)
        let positions = [0.45, 0.52, 0.60, 0.68, 0.76, 0.85];
        let hole_radii = [0.0040, 0.0042, 0.0045, 0.0048, 0.0050, 0.0052];
        let chimneys = [0.0022, 0.0024, 0.0025, 0.0026, 0.0028, 0.0030];

        let mut holes = [ToneholeJunction::default(); NUM_TONEHOLES];
        for i in 0..NUM_TONEHOLES {
            holes[i] = ToneholeJunction::new(positions[i], hole_radii[i], chimneys[i], bore_r);
        }

        let mut lattice = Self {
            toneholes: holes,
            total_bore_length_m: length,
            bore_radius_m: bore_r,
            lattice_cutoff_hz: 2200.0,
            junction_loss: 0.998,
        };
        lattice.recalculate_lattice_cutoff();
        lattice
    }

    /// Sets the entire 6-tonehole state from a bitmask (bits 0..5: 1 = closed, 0 = open).
    pub fn set_fingering_mask(&mut self, mask: u8) {
        for i in 0..NUM_TONEHOLES {
            let closed = (mask & (1 << i)) != 0;
            self.toneholes[i].set_closed(closed);
        }
        self.recalculate_lattice_cutoff();
    }

    /// Sets discrete boolean states for all 6 toneholes.
    pub fn set_tonehole_states(&mut self, states: [bool; NUM_TONEHOLES]) {
        for (i, &state) in states.iter().enumerate().take(NUM_TONEHOLES) {
            self.toneholes[i].set_closed(state);
        }
        self.recalculate_lattice_cutoff();
    }

    /// Sets fractional openings for all 6 toneholes for microtonal fingerings.
    pub fn set_open_fractions(&mut self, fractions: [f32; NUM_TONEHOLES]) {
        for (i, &frac) in fractions.iter().enumerate().take(NUM_TONEHOLES) {
            self.toneholes[i].set_open_fraction(frac);
        }
        self.recalculate_lattice_cutoff();
    }

    /// Returns the index of the first open tonehole closest to the embouchure (if any).
    pub fn first_open_tonehole_index(&self) -> Option<usize> {
        (0..NUM_TONEHOLES).find(|&i| self.toneholes[i].open_fraction > 0.05)
    }

    /// Computes effective acoustic tube length based on active fingering.
    pub fn effective_acoustic_length_m(&self) -> f32 {
        if let Some(first_open_idx) = self.first_open_tonehole_index() {
            let hole = &self.toneholes[first_open_idx];
            let raw_pos = hole.params.position_ratio * self.total_bore_length_m;
            // End-correction for open tonehole: delta_L ~ (bore_radius / hole_radius)^2 * chimney_height
            let area_ratio = (self.bore_radius_m / hole.params.hole_radius_m).powi(2);
            let end_corr = (hole.params.chimney_height_m * area_ratio * 0.4).clamp(0.005, 0.060);
            (raw_pos + end_corr).clamp(0.10, self.total_bore_length_m)
        } else {
            // All toneholes closed: full bore length plus open bell end correction
            self.total_bore_length_m + 0.6 * self.bore_radius_m
        }
    }

    /// Recalculates the tonehole lattice cutoff frequency $f_c$ in Hz (Keefe 1990).
    pub fn recalculate_lattice_cutoff(&mut self) {
        // Lattice cutoff formula: f_c = (c / 2pi) * sqrt( (2 * b / (a^2 * 2s * t_e)) )
        // Typical woodwind cutoff ranges from 1.5 kHz to 4.5 kHz
        let avg_hole_r = self.toneholes.iter().map(|h| h.params.hole_radius_m).sum::<f32>() / NUM_TONEHOLES as f32;
        let avg_chimney = self.toneholes.iter().map(|h| h.params.chimney_height_m).sum::<f32>() / NUM_TONEHOLES as f32;
        let s_spacing = (self.total_bore_length_m * 0.4) / (NUM_TONEHOLES as f32);

        let t_e = avg_chimney + 0.8 * avg_hole_r;
        let numerator = avg_hole_r.powi(2);
        let denominator = self.bore_radius_m.powi(2) * s_spacing * t_e;

        if denominator > 1e-9 {
            let fc = (SPEED_OF_SOUND_MPS / (2.0 * PI)) * (numerator / denominator).sqrt();
            self.lattice_cutoff_hz = fc.clamp(800.0, 6000.0);
        } else {
            self.lattice_cutoff_hz = 2200.0;
        }
    }

    /// Resets internal filter states for all toneholes.
    pub fn reset(&mut self) {
        for hole in self.toneholes.iter_mut() {
            hole.reset();
        }
    }
}
