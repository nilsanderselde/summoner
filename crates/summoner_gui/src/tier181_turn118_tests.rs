#![allow(clippy::all)]

//! Test suite for Turn #118 (Sprint Turn #44) — Quantum Bloch Sphere & Qubit State Tomography HUD
//! and Chaotic Fractal Attractor & Dynamic Systems Modulation HUD Convergence (Milestones 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::quantum_bloch_sphere_view::{
        QuantumStatePreset, QuantumBlochSphereView, QUANTUM_PUCK_HIT_RADIUS,
        NUM_BLOCH_ORBIT_POINTS,
    };
    use crate::views::chaotic_attractor_view::{
        AttractorSystemType, ChaoticAttractorPreset, ChaoticAttractorView,
        ATTRACTOR_PUCK_HIT_RADIUS, NUM_TRAJECTORY_POINTS,
    };
    use crate::layout_math::Rect;
    use std::f32::consts::PI;

    #[test]
    fn test_turn118_pure_quantum_bloch_sphere_initialization_and_physics() {
        let mut view = QuantumBlochSphereView::new();
        assert_eq!(view.preset, QuantumStatePreset::HadamardSuperposition);
        assert_eq!(QUANTUM_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt touch bounding target

        // Presets and expected parameters
        let presets = [
            (QuantumStatePreset::HadamardSuperposition, "Hadamard Superposition |+⟩ (Equator)", 0.50, 0.0, 440.0, 0.02),
            (QuantumStatePreset::ExcitedStatePauliX, "Excited State Pauli-X |1⟩ (South Pole)", 0.00, 0.0, 220.0, 0.01),
            (QuantumStatePreset::BellPhaseEntangled, "Bell Phase Entangled |+i⟩ (Y Axis)", 0.50, PI * 0.5, 550.0, 0.05),
            (QuantumStatePreset::GroundCoherentState, "Ground Coherent State |0⟩ (North Pole)", 1.00, 0.0, 440.0, 0.00),
        ];

        for (p, exp_name, exp_alpha, exp_phi, exp_freq, exp_decoh) in presets {
            view.set_preset(p);
            assert_eq!(p.name(), exp_name);
            let (alpha, phi, freq, decoh) = p.nominal_parameters();
            assert_eq!(alpha, exp_alpha);
            assert_eq!(phi, exp_phi);
            assert_eq!(freq, exp_freq);
            assert_eq!(decoh, exp_decoh);

            assert_eq!(view.alpha_prob, exp_alpha);
            assert_eq!(view.relative_phase_rad, exp_phi);
            assert_eq!(view.frequency_hz, exp_freq);
            assert_eq!(view.decoherence_rate, exp_decoh);
            assert!(view.state_purity >= 0.0 && view.state_purity <= 1.0);
            assert!(view.polar_angle_theta >= 0.0 && view.polar_angle_theta <= PI);
        }

        // Normalization round-trips for Alpha probability [0.0 ..= 1.0]
        let alpha_test = 0.725;
        let alpha_norm = QuantumBlochSphereView::alpha_prob_to_normalized(alpha_test);
        let alpha_back = QuantumBlochSphereView::normalized_to_alpha_prob(alpha_norm);
        assert!((alpha_back - alpha_test).abs() < 1e-4);

        // Normalization round-trips for Relative Phase [0.0 ..= 2*PI]
        let phase_test = 3.14159;
        let phase_norm = QuantumBlochSphereView::phase_to_normalized(phase_test);
        let phase_back = QuantumBlochSphereView::normalized_to_phase(phase_norm);
        assert!((phase_back - phase_test).abs() < 1e-4);

        // Quantum Gate Transformations:
        // 1. Pauli-X Bit Flip on Ground State (|0> -> |1>)
        view.set_preset(QuantumStatePreset::GroundCoherentState);
        assert_eq!(view.alpha_prob, 1.0);
        view.apply_pauli_x();
        assert_eq!(view.alpha_prob, 0.0);
        assert!(view.bloch_z < -0.9); // South pole

        // 2. Hadamard Gate on |1> -> equal superposition
        view.apply_hadamard();
        assert!((view.alpha_prob - 0.50).abs() < 0.05);

        // 3. Phase Shift delta
        let p_before = view.relative_phase_rad;
        view.apply_phase_shift(PI * 0.5);
        assert!(((view.relative_phase_rad - p_before) - PI * 0.5).abs() < 1e-3);

        // Bloch Orbit evaluation (48 points)
        let orbit = view.evaluate_bloch_orbit();
        assert_eq!(orbit.len(), NUM_BLOCH_ORBIT_POINTS);
        for pt in &orbit {
            assert!(pt.0.is_finite() && pt.1.is_finite() && pt.2.is_finite());
        }

        // Touch puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(20.0, 20.0, 320.0, 180.0);
        let puck_x = canvas.x + 15.0 + view.puck_pos.0 * (canvas.width - 30.0);
        let puck_y = canvas.y + canvas.height - 15.0 - view.puck_pos.1 * (canvas.height - 30.0);

        assert!(view.hit_test_quantum_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_quantum_puck((puck_x + 14.0, puck_y), canvas));
        assert!(view.hit_test_quantum_puck((puck_x, puck_y + 14.0), canvas));
        assert!(!view.hit_test_quantum_puck((puck_x + 35.0, puck_y), canvas));

        // Deterministic ASCII snapshot string
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("QUANTUM BLOCH SPHERE & STATE TOMOGRAPHY HUD"));
        assert!(snap.contains("[PASS: >=44pt]"));
    }

    #[test]
    fn test_turn118_pure_chaotic_attractor_initialization_and_physics() {
        let mut view = ChaoticAttractorView::new();
        assert_eq!(view.preset, ChaoticAttractorPreset::ClassicLorenzButterfly);
        assert_eq!(ATTRACTOR_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt touch bounding target

        // Presets and expected parameters
        let presets = [
            (ChaoticAttractorPreset::ClassicLorenzButterfly, "Classic Lorenz Butterfly (ρ=28.0, σ=10.0)", AttractorSystemType::LorenzSystem, 28.0, 10.0, 2.667, 0.010),
            (ChaoticAttractorPreset::RosslerStrangeAttractor, "Rössler Strange Attractor (Folded Band)", AttractorSystemType::RosslerSystem, 15.0, 6.0, 2.000, 0.015),
            (ChaoticAttractorPreset::HyperchaoticConvection, "Hyperchaotic Convection (ρ=45.0, σ=18.0)", AttractorSystemType::LorenzSystem, 45.0, 18.0, 3.200, 0.008),
            (ChaoticAttractorPreset::SubcriticalLimitCycle, "Subcritical Limit Cycle (ρ=22.0, σ=8.0)", AttractorSystemType::LorenzSystem, 22.0, 8.0, 2.000, 0.012),
        ];

        for (p, exp_name, exp_type, exp_rho, exp_sigma, exp_beta, exp_dt) in presets {
            view.set_preset(p);
            assert_eq!(p.name(), exp_name);
            let (st, rho, sigma, beta, dt) = p.nominal_parameters();
            assert_eq!(st, exp_type);
            assert_eq!(rho, exp_rho);
            assert_eq!(sigma, exp_sigma);
            assert_eq!(beta, exp_beta);
            assert_eq!(dt, exp_dt);

            assert_eq!(view.system_type, exp_type);
            assert_eq!(view.rayleigh_rho, exp_rho);
            assert_eq!(view.prandtl_sigma, exp_sigma);
            assert_eq!(view.geometric_beta, exp_beta);
            assert_eq!(view.time_step_dt, exp_dt);
            assert!(view.orbit_velocity > 0.0);
        }

        // Normalization round-trips for Rayleigh rho [10.0 ..= 50.0]
        let rho_test = 34.5;
        let rho_norm = ChaoticAttractorView::rho_to_normalized(rho_test);
        let rho_back = ChaoticAttractorView::normalized_to_rho(rho_norm);
        assert!((rho_back - rho_test).abs() < 1e-4);

        // Normalization round-trips for Prandtl sigma [2.0 ..= 25.0]
        let sigma_test = 14.2;
        let sigma_norm = ChaoticAttractorView::sigma_to_normalized(sigma_test);
        let sigma_back = ChaoticAttractorView::normalized_to_sigma(sigma_norm);
        assert!((sigma_back - sigma_test).abs() < 1e-4);

        // Dynamic step integration: coordinates stay finite
        let (x, y, z) = view.step_dynamics();
        assert!(x.is_finite() && y.is_finite() && z.is_finite());

        // Orbit perturbation
        let x_before = view.current_x;
        view.perturb_orbit();
        assert_ne!(view.current_x, x_before);

        // Trajectory evaluation (96 points)
        let traj = view.evaluate_trajectory();
        assert_eq!(traj.len(), NUM_TRAJECTORY_POINTS);
        for pt in &traj {
            assert!(pt.0.is_finite() && pt.1.is_finite() && pt.2.is_finite());
        }

        // Touch puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(20.0, 20.0, 300.0, 180.0);
        let puck_x = canvas.x + 15.0 + view.puck_pos.0 * (canvas.width - 30.0);
        let puck_y = canvas.y + canvas.height - 15.0 - view.puck_pos.1 * (canvas.height - 30.0);

        assert!(view.hit_test_attractor_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_attractor_puck((puck_x + 14.0, puck_y), canvas));
        assert!(view.hit_test_attractor_puck((puck_x, puck_y + 14.0), canvas));
        assert!(!view.hit_test_attractor_puck((puck_x + 35.0, puck_y), canvas));

        // Deterministic ASCII snapshot string
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("CHAOTIC FRACTAL ATTRACTOR MODULATION HUD"));
        assert!(snap.contains("[PASS: >=44pt]"));
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::command_palette::CommandPalette;
    use eframe::egui;

    #[test]
    fn test_turn118_gui_quantum_and_chaotic_modal_lifecycle() {
        let mut view = AwardWinningGuiView::new();

        // 1. Quantum Bloch Sphere Modal Lifecycle
        assert!(!view.is_quantum_bloch_sphere_hud_open());
        view.open_quantum_bloch_sphere_hud();
        assert!(view.is_quantum_bloch_sphere_hud_open());
        view.close_quantum_bloch_sphere_hud();
        assert!(!view.is_quantum_bloch_sphere_hud_open());

        // 2. Chaotic Attractor Modal Lifecycle
        assert!(!view.is_chaotic_attractor_hud_open());
        view.open_chaotic_attractor_hud();
        assert!(view.is_chaotic_attractor_hud_open());
        view.close_chaotic_attractor_hud();
        assert!(!view.is_chaotic_attractor_hud_open());

        // 3. Quantum Parameter Sync Bridge
        view.quantum_bloch_sphere_view.alpha_prob = 0.65;
        view.quantum_bloch_sphere_view.relative_phase_rad = 1.57;
        view.quantum_bloch_sphere_view.frequency_hz = 520.0;
        view.quantum_bloch_sphere_view.decoherence_rate = 0.04;
        view.sync_quantum_bloch_sphere_hud_state();

        assert_eq!(
            view.device_rack_state.node_param_values.get("alpha_prob").copied(),
            Some(0.65)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("relative_phase_rad").copied(),
            Some(1.57)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("alpha_prob").copied(),
            Some(0.65)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("relative_phase_rad").copied(),
            Some(1.57)
        );

        // 4. Chaotic Attractor Parameter Sync Bridge
        view.chaotic_attractor_view.rayleigh_rho = 35.0;
        view.chaotic_attractor_view.prandtl_sigma = 12.0;
        view.chaotic_attractor_view.geometric_beta = 2.8;
        view.chaotic_attractor_view.time_step_dt = 0.012;
        view.sync_chaotic_attractor_hud_state();

        assert_eq!(
            view.device_rack_state.node_param_values.get("rayleigh_rho").copied(),
            Some(35.0)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("prandtl_sigma").copied(),
            Some(12.0)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("rayleigh_rho").copied(),
            Some(35.0)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("prandtl_sigma").copied(),
            Some(12.0)
        );

        // 5. Headless Egui Rendering
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show_quantum_bloch_sphere_modal_window(ui);
                view.show_chaotic_attractor_modal_window(ui);
            });
        });

        // 6. Command Palette Registration & Action Verification
        let palette = CommandPalette::new();
        let quantum_action = palette.actions.iter().find(|a| a.action_id == "open_quantum_bloch_sphere_hud");
        assert!(quantum_action.is_some(), "open_quantum_bloch_sphere_hud should be registered");
        assert_eq!(quantum_action.unwrap().category, "Quantum & Spectral Synthesis");

        let chaos_action = palette.actions.iter().find(|a| a.action_id == "open_chaotic_attractor_hud");
        assert!(chaos_action.is_some(), "open_chaotic_attractor_hud should be registered");
        assert_eq!(chaos_action.unwrap().category, "Modulation & Dynamics");
    }
}
