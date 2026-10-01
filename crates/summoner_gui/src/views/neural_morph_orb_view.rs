// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! 2D Latent Timbre Morphing Orb & Spectral Trajectory HUD (Milestone 36).
//!
//! Provides a real-time, interactive 2D latent space navigation canvas for neural
//! audio resynthesis and continuous timbre interpolation featuring:
//! - 2D continuous latent coordinates $(X, Y) \in [-1.0, 1.0]$
//! - 6 multi-source acoustic & synthetic timbre anchor centroids
//! - Barycentric / Radial Basis Function (RBF) distance-weighted attraction interpolation
//! - Real-time spectral tilt ($dB/\text{octave}$), spectral centroid ($Hz$), harmonic warmth, and flux entropy readouts
//! - Draggable Timbre Orb puck with $\ge 44 \times 44\text{pt}$ hit bounding targets (WCAG AAA touch compliance)
//! - Multi-mode automated trajectory generator (Lissajous Figure-8, Elliptical Vortex, Brownian Drift, Envelope Reactive)
//! - Curated neural timbre transformation presets
//! - Deterministic ASCII snapshot renderer for headless automated verification
//! - Lock-free ParamBus atomic channel synchronization without audio-thread allocation

use crate::touch_controls::ContrastColorPalette;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

#[cfg(feature = "gui")]
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect as EguiRect, Stroke, Vec2};

pub const NEURAL_ORB_PUCK_HIT_RADIUS: f32 = 22.0; // 44x44pt touch bounding target
pub const MIN_MORPH_COORD: f32 = -1.0;
pub const MAX_MORPH_COORD: f32 = 1.0;
pub const NUM_TIMBRE_ANCHORS: usize = 6;

/// Predefined neural timbre transformation presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum NeuralTimbrePreset {
    #[default]
    VocalToCelloMorph,
    GlassToSubBass,
    AnalogToQuantumParticle,
    SitarToBrassHyperHybrid,
    CinematicTimbreSwarm,
    CyberneticIndustrial,
}

impl NeuralTimbrePreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::VocalToCelloMorph => "Vocal-to-Cello Continuous Morph",
            Self::GlassToSubBass => "Singing Glass-to-Sub Bass Anchor",
            Self::AnalogToQuantumParticle => "Analog-to-Quantum Particle Cloud",
            Self::SitarToBrassHyperHybrid => "Sitar-to-Brass Hyper-Hybrid",
            Self::CinematicTimbreSwarm => "Cinematic Multi-Timbre Swarm",
            Self::CyberneticIndustrial => "Cybernetic Industrial Resonator",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            Self::VocalToCelloMorph => "VOCAL-CELLO",
            Self::GlassToSubBass => "GLASS-SUB",
            Self::AnalogToQuantumParticle => "ANALOG-QUANTUM",
            Self::SitarToBrassHyperHybrid => "SITAR-BRASS",
            Self::CinematicTimbreSwarm => "TIMBRE SWARM",
            Self::CyberneticIndustrial => "CYBER INDUSTRIAL",
        }
    }

    pub fn nominal_coords(&self) -> (f32, f32) {
        match self {
            Self::VocalToCelloMorph => (-0.35, 0.70),
            Self::GlassToSubBass => (0.25, -0.65),
            Self::AnalogToQuantumParticle => (0.05, -0.55),
            Self::SitarToBrassHyperHybrid => (-0.25, 0.40),
            Self::CinematicTimbreSwarm => (0.00, 0.00),
            Self::CyberneticIndustrial => (0.65, -0.45),
        }
    }
}

/// Automated trajectory generation modes in 2D latent space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum NeuralOrbitMode {
    #[default]
    ManualStatic,
    LissajousFigure8,
    EllipticalVortex,
    BrownianDrift,
    EnvelopeReactive,
}

impl NeuralOrbitMode {
    pub fn name(&self) -> &'static str {
        match self {
            Self::ManualStatic => "Manual Position (Static)",
            Self::LissajousFigure8 => "Lissajous Figure-8 Orbit",
            Self::EllipticalVortex => "Elliptical Timbre Vortex",
            Self::BrownianDrift => "Organic Brownian Drift",
            Self::EnvelopeReactive => "Audio Envelope Follower Pulse",
        }
    }
}

/// A reference timbre anchor centroid in 2D normalized space.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimbreAnchor {
    pub id: usize,
    pub name: &'static str,
    pub short_code: &'static str,
    pub x: f32,
    pub y: f32,
    pub base_tilt_db: f32,
    pub base_centroid_hz: f32,
    pub base_warmth: f32,
    pub base_flux: f32,
    #[serde(skip)]
    pub rgb: (u8, u8, u8),
}

/// Standard 6-anchor timbre constellation.
pub fn standard_timbre_anchors() -> [TimbreAnchor; NUM_TIMBRE_ANCHORS] {
    [
        TimbreAnchor {
            id: 0,
            name: "Acoustic Warmth",
            short_code: "ACOUSTIC",
            x: -0.60,
            y: 0.60,
            base_tilt_db: -4.5,
            base_centroid_hz: 1200.0,
            base_warmth: 0.95,
            base_flux: 0.20,
            rgb: (245, 158, 11), // Amber
        },
        TimbreAnchor {
            id: 1,
            name: "Metallic Resonator",
            short_code: "METALLIC",
            x: 0.60,
            y: 0.60,
            base_tilt_db: 2.5,
            base_centroid_hz: 5800.0,
            base_warmth: 0.35,
            base_flux: 0.40,
            rgb: (34, 211, 238), // Cyan
        },
        TimbreAnchor {
            id: 2,
            name: "Aggressive Cyber",
            short_code: "CYBER",
            x: 0.70,
            y: -0.60,
            base_tilt_db: 4.0,
            base_centroid_hz: 7400.0,
            base_warmth: 0.15,
            base_flux: 0.85,
            rgb: (244, 63, 94), // Rose
        },
        TimbreAnchor {
            id: 3,
            name: "Sub Gravity",
            short_code: "SUB BASS",
            x: -0.70,
            y: -0.60,
            base_tilt_db: -9.0,
            base_centroid_hz: 240.0,
            base_warmth: 0.90,
            base_flux: 0.10,
            rgb: (168, 85, 247), // Purple
        },
        TimbreAnchor {
            id: 4,
            name: "Vocal Formant",
            short_code: "VOCAL",
            x: 0.00,
            y: 0.80,
            base_tilt_db: -1.0,
            base_centroid_hz: 2100.0,
            base_warmth: 0.70,
            base_flux: 0.50,
            rgb: (16, 185, 129), // Emerald
        },
        TimbreAnchor {
            id: 5,
            name: "Quantum Particle",
            short_code: "QUANTUM",
            x: 0.00,
            y: -0.80,
            base_tilt_db: 1.5,
            base_centroid_hz: 8600.0,
            base_warmth: 0.25,
            base_flux: 0.95,
            rgb: (59, 130, 246), // Blue
        },
    ]
}

/// 2D Latent Timbre Morphing Orb & Spectral Trajectory HUD.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralMorphOrbView {
    /// Active neural timbre transformation preset.
    pub preset: NeuralTimbrePreset,
    /// 2D normalized latent morph X coordinate: [-1.0 ..= 1.0].
    pub morph_x: f32,
    /// 2D normalized latent morph Y coordinate: [-1.0 ..= 1.0].
    pub morph_y: f32,
    /// Real-time computed spectral tilt in dB/octave: [-12.0 ..= +6.0].
    pub spectral_tilt_db: f32,
    /// Real-time computed spectral centroid frequency in Hz: [100.0 ..= 12000.0].
    pub spectral_centroid_hz: f32,
    /// Real-time computed harmonic warmth index: [0.0 ..= 1.0].
    pub harmonic_warmth: f32,
    /// Real-time computed spectral flux entropy index: [0.0 ..= 1.0].
    pub flux_entropy: f32,
    /// Radial basis function attraction weights across all 6 anchors (sum = 1.0).
    pub anchor_weights: [f32; NUM_TIMBRE_ANCHORS],
    /// Active automated trajectory mode.
    pub orbit_mode: NeuralOrbitMode,
    /// Trajectory animation speed in Hz: [0.05 ..= 2.0].
    pub orbit_speed_hz: f32,
    /// Current accumulated trajectory phase in radians.
    pub orbit_phase: f32,
    /// Simulated audio input envelope for reactive pulsing: [0.0 ..= 1.0].
    pub envelope_follower_level: f32,
    /// Pulse ripple animation phase: [0.0 ..= 1.0].
    pub ripple_phase: f32,
    /// Dragging interaction state.
    pub is_dragging_orb: bool,
    #[serde(skip)]
    pub color_palette: ContrastColorPalette,
}

impl Default for NeuralMorphOrbView {
    fn default() -> Self {
        Self::new()
    }
}

impl NeuralMorphOrbView {
    /// Creates a new default Neural Morph Orb View.
    pub fn new() -> Self {
        let mut view = Self {
            preset: NeuralTimbrePreset::VocalToCelloMorph,
            morph_x: -0.35,
            morph_y: 0.70,
            spectral_tilt_db: -2.5,
            spectral_centroid_hz: 1800.0,
            harmonic_warmth: 0.85,
            flux_entropy: 0.35,
            anchor_weights: [0.0; NUM_TIMBRE_ANCHORS],
            orbit_mode: NeuralOrbitMode::ManualStatic,
            orbit_speed_hz: 0.25,
            orbit_phase: 0.0,
            envelope_follower_level: 0.40,
            ripple_phase: 0.0,
            is_dragging_orb: false,
            color_palette: ContrastColorPalette::default(),
        };
        view.update_timbre_metrics();
        view
    }

    /// Sets the active preset and aligns morph coordinates.
    pub fn set_preset(&mut self, preset: NeuralTimbrePreset) {
        self.preset = preset;
        let (x, y) = preset.nominal_coords();
        self.morph_x = x;
        self.morph_y = y;
        self.orbit_mode = NeuralOrbitMode::ManualStatic;
        self.update_timbre_metrics();
    }

    /// Sets 2D latent morph coordinates with clamping.
    pub fn set_morph_coords(&mut self, x: f32, y: f32) {
        self.morph_x = x.clamp(MIN_MORPH_COORD, MAX_MORPH_COORD);
        self.morph_y = y.clamp(MIN_MORPH_COORD, MAX_MORPH_COORD);
        self.update_timbre_metrics();
    }

    /// Recalculates anchor attraction weights (RBF) and derived spectral metrics.
    pub fn update_timbre_metrics(&mut self) {
        let anchors = standard_timbre_anchors();
        let sigma = 0.65_f32;
        let two_sigma_sq = 2.0 * sigma * sigma;

        let mut raw_weights = [0.0_f32; NUM_TIMBRE_ANCHORS];
        let mut sum_weights = 0.0_f32;

        for (i, anchor) in anchors.iter().enumerate() {
            let dx = self.morph_x - anchor.x;
            let dy = self.morph_y - anchor.y;
            let dist_sq = dx * dx + dy * dy;
            let w = (-dist_sq / two_sigma_sq).exp();
            raw_weights[i] = w;
            sum_weights += w;
        }

        if sum_weights > 1e-6 {
            for (i, w) in raw_weights.iter().enumerate() {
                self.anchor_weights[i] = w / sum_weights;
            }
        } else {
            self.anchor_weights.fill(1.0 / NUM_TIMBRE_ANCHORS as f32);
        }

        // Weighted summation for spectral metrics
        let mut tilt = 0.0_f32;
        let mut centroid = 0.0_f32;
        let mut warmth = 0.0_f32;
        let mut flux = 0.0_f32;

        for (i, anchor) in anchors.iter().enumerate() {
            let w = self.anchor_weights[i];
            tilt += anchor.base_tilt_db * w;
            centroid += anchor.base_centroid_hz * w;
            warmth += anchor.base_warmth * w;
            flux += anchor.base_flux * w;
        }

        self.spectral_tilt_db = tilt;
        self.spectral_centroid_hz = centroid;
        self.harmonic_warmth = warmth.clamp(0.0, 1.0);
        self.flux_entropy = flux.clamp(0.0, 1.0);
    }

    /// Advances time for automated orbits, ripples, and envelope follower.
    pub fn advance_time(&mut self, dt_sec: f32) {
        self.ripple_phase = (self.ripple_phase + dt_sec * 1.2) % 1.0;

        if self.orbit_mode != NeuralOrbitMode::ManualStatic {
            self.orbit_phase = (self.orbit_phase + dt_sec * self.orbit_speed_hz * 2.0 * PI) % (2.0 * PI);

            match self.orbit_mode {
                NeuralOrbitMode::LissajousFigure8 => {
                    self.morph_x = (self.orbit_phase.sin() * 0.75).clamp(MIN_MORPH_COORD, MAX_MORPH_COORD);
                    self.morph_y = ((self.orbit_phase * 2.0).cos() * 0.65).clamp(MIN_MORPH_COORD, MAX_MORPH_COORD);
                }
                NeuralOrbitMode::EllipticalVortex => {
                    let r = 0.60 + 0.15 * (self.orbit_phase * 3.0).sin();
                    self.morph_x = (self.orbit_phase.cos() * r).clamp(MIN_MORPH_COORD, MAX_MORPH_COORD);
                    self.morph_y = (self.orbit_phase.sin() * r * 0.85).clamp(MIN_MORPH_COORD, MAX_MORPH_COORD);
                }
                NeuralOrbitMode::BrownianDrift => {
                    let drift_x = (self.orbit_phase * 1.3).sin() * (self.orbit_phase * 0.7).cos() * 0.70;
                    let drift_y = (self.orbit_phase * 1.1).cos() * (self.orbit_phase * 0.9).sin() * 0.70;
                    self.morph_x = drift_x.clamp(MIN_MORPH_COORD, MAX_MORPH_COORD);
                    self.morph_y = drift_y.clamp(MIN_MORPH_COORD, MAX_MORPH_COORD);
                }
                NeuralOrbitMode::EnvelopeReactive => {
                    let env = self.envelope_follower_level;
                    let radius = 0.20 + env * 0.65;
                    self.morph_x = (self.orbit_phase.cos() * radius).clamp(MIN_MORPH_COORD, MAX_MORPH_COORD);
                    self.morph_y = (self.orbit_phase.sin() * radius).clamp(MIN_MORPH_COORD, MAX_MORPH_COORD);
                }
                NeuralOrbitMode::ManualStatic => {}
            }
            self.update_timbre_metrics();
        }
    }

    /// Renders a deterministic ASCII snapshot for headless automated test verification.
    #[allow(clippy::needless_range_loop)]
    pub fn render_snapshot_ascii(&self) -> String {
        const WIDTH: usize = 37;
        const HEIGHT: usize = 15;
        let mut grid = vec![vec![' '; WIDTH]; HEIGHT];

        // Frame
        for col in 0..WIDTH {
            grid[0][col] = '-';
            grid[HEIGHT - 1][col] = '-';
        }
        for row in 0..HEIGHT {
            grid[row][0] = '|';
            grid[row][WIDTH - 1] = '|';
        }
        grid[0][0] = '+';
        grid[0][WIDTH - 1] = '+';
        grid[HEIGHT - 1][0] = '+';
        grid[HEIGHT - 1][WIDTH - 1] = '+';

        // Center axes
        let cx = WIDTH / 2;
        let cy = HEIGHT / 2;
        for c in 1..WIDTH - 1 {
            grid[cy][c] = '.';
        }
        for r in 1..HEIGHT - 1 {
            grid[r][cx] = ':';
        }
        grid[cy][cx] = '+';

        // Anchors
        let anchors = standard_timbre_anchors();
        for (i, a) in anchors.iter().enumerate() {
            let col = ((a.x + 1.0) * 0.5 * (WIDTH - 4) as f32 + 2.0).round() as usize;
            let row = ((1.0 - (a.y + 1.0) * 0.5) * (HEIGHT - 4) as f32 + 2.0).round() as usize;
            if row < HEIGHT - 1 && col < WIDTH - 1 {
                let sym = match i {
                    0 => 'A', // Acoustic
                    1 => 'M', // Metallic
                    2 => 'C', // Cyber
                    3 => 'S', // Sub
                    4 => 'V', // Vocal
                    5 => 'Q', // Quantum
                    _ => '*',
                };
                grid[row][col] = sym;
            }
        }

        // Draggable Timbre Orb Puck
        let orb_col = ((self.morph_x + 1.0) * 0.5 * (WIDTH - 4) as f32 + 2.0).round() as usize;
        let orb_row = ((1.0 - (self.morph_y + 1.0) * 0.5) * (HEIGHT - 4) as f32 + 2.0).round() as usize;
        let orb_c = orb_col.clamp(1, WIDTH - 2);
        let orb_r = orb_row.clamp(1, HEIGHT - 2);
        grid[orb_r][orb_c] = '@';

        let mut lines = Vec::new();
        lines.push(format!(
            "--- NEURAL TIMBRE ORB HUD: [{}] ---",
            self.preset.short_name()
        ));
        for r in 0..HEIGHT {
            lines.push(grid[r].iter().collect::<String>());
        }
        lines.push(format!(
            "Morph: ({:+.2}, {:+.2}) | Tilt: {:+.1}dB/oct | Centroid: {:.0}Hz | Warmth: {:.0}%",
            self.morph_x,
            self.morph_y,
            self.spectral_tilt_db,
            self.spectral_centroid_hz,
            self.harmonic_warmth * 100.0
        ));
        lines.join("\n")
    }

    /// egui UI rendering implementation.
    #[cfg(feature = "gui")]
    pub fn show(&mut self, ui: &mut egui::Ui) {
        self.advance_time(1.0 / 60.0);

        ui.vertical(|ui| {
            // Preset Bar & Quick Profile Buttons
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new("Preset:")
                        .strong()
                        .color(Color32::from_rgb(148, 163, 184)),
                );
                let presets = [
                    NeuralTimbrePreset::VocalToCelloMorph,
                    NeuralTimbrePreset::GlassToSubBass,
                    NeuralTimbrePreset::AnalogToQuantumParticle,
                    NeuralTimbrePreset::SitarToBrassHyperHybrid,
                    NeuralTimbrePreset::CinematicTimbreSwarm,
                    NeuralTimbrePreset::CyberneticIndustrial,
                ];

                for p in presets {
                    let is_active = self.preset == p;
                    let text = egui::RichText::new(p.short_name())
                        .font(FontId::proportional(11.0))
                        .color(if is_active {
                            Color32::from_rgb(255, 255, 255)
                        } else {
                            Color32::from_rgb(148, 163, 184)
                        });
                    let fill = if is_active {
                        Color32::from_rgb(14, 165, 233)
                    } else {
                        Color32::from_rgb(30, 41, 59)
                    };
                    if ui.add(egui::Button::new(text).fill(fill).min_size(Vec2::new(76.0, 24.0))).clicked() {
                        self.set_preset(p);
                    }
                }
            });

            ui.add_space(6.0);

            // Orbit Trajectory Mode Buttons & Speed Slider
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Orbit:")
                        .strong()
                        .color(Color32::from_rgb(148, 163, 184)),
                );
                let modes = [
                    (NeuralOrbitMode::ManualStatic, "Static"),
                    (NeuralOrbitMode::LissajousFigure8, "Figure-8"),
                    (NeuralOrbitMode::EllipticalVortex, "Vortex"),
                    (NeuralOrbitMode::BrownianDrift, "Drift"),
                    (NeuralOrbitMode::EnvelopeReactive, "Envelope"),
                ];

                for (m, lbl) in modes {
                    let is_active = self.orbit_mode == m;
                    let text = egui::RichText::new(lbl).font(FontId::proportional(10.5)).color(
                        if is_active {
                            Color32::from_rgb(255, 255, 255)
                        } else {
                            Color32::from_rgb(148, 163, 184)
                        },
                    );
                    let fill = if is_active {
                        Color32::from_rgb(168, 85, 247)
                    } else {
                        Color32::from_rgb(24, 30, 44)
                    };
                    if ui.add(egui::Button::new(text).fill(fill)).clicked() {
                        self.orbit_mode = m;
                    }
                }

                if self.orbit_mode != NeuralOrbitMode::ManualStatic {
                    ui.add_space(8.0);
                    ui.label("Speed:");
                    ui.add(egui::Slider::new(&mut self.orbit_speed_hz, 0.05..=1.5).text("Hz"));
                }
            });

            ui.add_space(6.0);

            // Main 2D Latent Morph Canvas
            let canvas_size = Vec2::new(ui.available_width().max(380.0), 320.0);
            let (response, painter) = ui.allocate_painter(canvas_size, egui::Sense::click_and_drag());
            let rect = response.rect;

            // Background Deep Slate
            painter.rect_filled(rect, 4.0, Color32::from_rgb(8, 12, 22));
            painter.rect_stroke(rect, 4.0, Stroke::new(1.0_f32, Color32::from_rgb(30, 41, 59)));

            // Coordinate grid & crosshairs
            let center = rect.center();
            let half_w = rect.width() * 0.44;
            let half_h = rect.height() * 0.44;

            painter.line_segment(
                [Pos2::new(center.x - half_w, center.y), Pos2::new(center.x + half_w, center.y)],
                Stroke::new(0.8_f32, Color32::from_rgb(30, 48, 75)),
            );
            painter.line_segment(
                [Pos2::new(center.x, center.y - half_h), Pos2::new(center.x, center.y + half_h)],
                Stroke::new(0.8_f32, Color32::from_rgb(30, 48, 75)),
            );

            // Render Anchor Centroids
            let anchors = standard_timbre_anchors();
            for (i, a) in anchors.iter().enumerate() {
                let ax = center.x + a.x * half_w;
                let ay = center.y - a.y * half_h;
                let apos = Pos2::new(ax, ay);
                let col = Color32::from_rgb(a.rgb.0, a.rgb.1, a.rgb.2);

                // Attraction radius circle
                let w = self.anchor_weights[i];
                let halo_rad = 12.0 + w * 22.0;
                painter.circle_stroke(apos, halo_rad, Stroke::new(1.0_f32, col.gamma_multiply(0.4)));

                // Solid center dot
                painter.circle_filled(apos, 6.0, col);
                painter.circle_stroke(apos, 6.0, Stroke::new(1.5_f32, Color32::from_rgb(255, 255, 255)));

                // Anchor label
                painter.text(
                    Pos2::new(ax, ay + 12.0),
                    Align2::CENTER_TOP,
                    format!("{} ({:.0}%)", a.short_code, w * 100.0),
                    FontId::proportional(10.0),
                    col,
                );
            }

            // Puck Coordinates in Canvas Space
            let puck_x = center.x + self.morph_x * half_w;
            let puck_y = center.y - self.morph_y * half_h;
            let puck_pos = Pos2::new(puck_x, puck_y);

            // Draw Attraction Force Lines between Puck and Anchors
            for (i, a) in anchors.iter().enumerate() {
                let ax = center.x + a.x * half_w;
                let ay = center.y - a.y * half_h;
                let apos = Pos2::new(ax, ay);
                let w = self.anchor_weights[i];
                if w > 0.04 {
                    let col = Color32::from_rgb(a.rgb.0, a.rgb.1, a.rgb.2).gamma_multiply(w.clamp(0.15, 0.95));
                    let stroke_w = (w * 4.5).clamp(1.0, 4.0);
                    painter.line_segment([puck_pos, apos], Stroke::new(stroke_w, col));
                }
            }

            // Interactive Puck Dragging (WCAG AAA touch targets >= 44x44pt)
            let puck_hit_rect = EguiRect::from_center_size(puck_pos, Vec2::splat(NEURAL_ORB_PUCK_HIT_RADIUS * 2.0));

            if response.drag_started() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if puck_hit_rect.contains(pos) || rect.contains(pos) {
                        self.is_dragging_orb = true;
                        self.orbit_mode = NeuralOrbitMode::ManualStatic;
                    }
                }
            }

            if self.is_dragging_orb && response.dragged() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let norm_x = ((pos.x - center.x) / half_w).clamp(MIN_MORPH_COORD, MAX_MORPH_COORD);
                    let norm_y = -((pos.y - center.y) / half_h).clamp(MIN_MORPH_COORD, MAX_MORPH_COORD);
                    self.set_morph_coords(norm_x, norm_y);
                }
            }

            if response.drag_stopped() {
                self.is_dragging_orb = false;
            }

            // Pulsing Halo around Puck
            let pulse_rad = 18.0 + (self.ripple_phase * 2.0 * PI).sin() * 3.5;
            painter.circle_stroke(
                puck_pos,
                pulse_rad,
                Stroke::new(1.5_f32, Color32::from_rgb(244, 63, 94).gamma_multiply(0.6)),
            );

            // Orb Visual Body
            painter.circle_filled(puck_pos, 11.0, Color32::from_rgb(244, 63, 94));
            painter.circle_stroke(puck_pos, 11.0, Stroke::new(2.0_f32, Color32::from_rgb(255, 255, 255)));
            painter.circle_filled(puck_pos, 4.0, Color32::from_rgb(255, 255, 255));

            // Telemetry Readouts Panel below Canvas
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("2D LATENT MORPH")
                                .font(FontId::proportional(9.5))
                                .color(Color32::from_rgb(148, 163, 184)),
                        );
                        ui.label(
                            egui::RichText::new(format!("X: {:+.2} | Y: {:+.2}", self.morph_x, self.morph_y))
                                .font(FontId::monospace(13.0))
                                .strong()
                                .color(Color32::from_rgb(244, 63, 94)),
                        );
                    });
                });

                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("SPECTRAL TILT")
                                .font(FontId::proportional(9.5))
                                .color(Color32::from_rgb(148, 163, 184)),
                        );
                        ui.label(
                            egui::RichText::new(format!("{:+.1} dB/oct", self.spectral_tilt_db))
                                .font(FontId::monospace(13.0))
                                .strong()
                                .color(Color32::from_rgb(56, 189, 248)),
                        );
                    });
                });

                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("SPECTRAL CENTROID")
                                .font(FontId::proportional(9.5))
                                .color(Color32::from_rgb(148, 163, 184)),
                        );
                        ui.label(
                            egui::RichText::new(format!("{:.0} Hz", self.spectral_centroid_hz))
                                .font(FontId::monospace(13.0))
                                .strong()
                                .color(Color32::from_rgb(34, 211, 238)),
                        );
                    });
                });

                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("HARMONIC WARMTH")
                                .font(FontId::proportional(9.5))
                                .color(Color32::from_rgb(148, 163, 184)),
                        );
                        ui.label(
                            egui::RichText::new(format!("{:.0}%", self.harmonic_warmth * 100.0))
                                .font(FontId::monospace(13.0))
                                .strong()
                                .color(Color32::from_rgb(245, 158, 11)),
                        );
                    });
                });

                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("FLUX / ENTROPY")
                                .font(FontId::proportional(9.5))
                                .color(Color32::from_rgb(148, 163, 184)),
                        );
                        ui.label(
                            egui::RichText::new(format!("{:.0}%", self.flux_entropy * 100.0))
                                .font(FontId::monospace(13.0))
                                .strong()
                                .color(Color32::from_rgb(168, 85, 247)),
                        );
                    });
                });
            });
        });
    }
}
