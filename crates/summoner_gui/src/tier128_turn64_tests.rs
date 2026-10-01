#![allow(clippy::all)]

//! Test suite for Turn #64 (Turn #27 Deliverable) — 2D Physical Waveguide Resonator Mesh & Karplus-Strong Plucked String HUDs Convergence (Milestones 13, 18 & 33).

#[cfg(test)]
pub mod pure_tests {
    use crate::layout_math::Rect;
    use crate::views::waveguide_mesh_view::{
        WaveguideMeshView, WAVEGUIDE_PUCK_HIT_RADIUS, WAVEGUIDE_PUCK_VISUAL_RADIUS,
    };
    use crate::views::plucked_string_view::{
        PluckedStringView, PluckedInstrumentViewProfile, PLUCK_PUCK_HIT_RADIUS,
        MIN_PLUCK_POSITION_BETA, MAX_PLUCK_POSITION_BETA, MIN_STRIKE_VELOCITY, MAX_STRIKE_VELOCITY,
    };
    use summoner_dsp::waveguide_mesh::{MeshBoundaryType, MeshMaterialProfile, MESH_DIM};

    #[test]
    fn test_turn64_pure_waveguide_mesh_view_initialization_materials_and_boundaries() {
        let mut view = WaveguideMeshView::new();
        assert!((view.strike_puck_pos.0 - 0.50).abs() < 1e-3);
        assert!((view.strike_puck_pos.1 - 0.50).abs() < 1e-3);
        assert!(!view.is_dragging_puck);
        assert!((view.view_elevation_deg - 35.0).abs() < 1e-3);
        assert!((view.view_azimuth_deg - 45.0).abs() < 1e-3);
        assert_eq!(view.wireframe_density, 16);
        assert!(view.show_energy_meter);
        assert_eq!(view.mesh.material, MeshMaterialProfile::Membrane);
        assert_eq!(view.mesh.boundary, MeshBoundaryType::Clamped);
        assert_eq!(WAVEGUIDE_PUCK_HIT_RADIUS, 22.0);
        assert_eq!(WAVEGUIDE_PUCK_VISUAL_RADIUS, 14.0);

        // Test material transitions and Courant factors
        view.set_material(MeshMaterialProfile::Plate);
        assert_eq!(view.mesh.material, MeshMaterialProfile::Plate);
        assert!((view.mesh.courant - 0.65).abs() < 1e-3);

        view.set_material(MeshMaterialProfile::AcousticBar);
        assert_eq!(view.mesh.material, MeshMaterialProfile::AcousticBar);
        assert!((view.mesh.courant - 0.45).abs() < 1e-3);

        view.set_material(MeshMaterialProfile::Membrane);
        assert_eq!(view.mesh.material, MeshMaterialProfile::Membrane);
        assert!((view.mesh.courant - 0.50).abs() < 1e-3);

        // Test boundary conditions
        view.set_boundary(MeshBoundaryType::Free);
        assert_eq!(view.mesh.boundary, MeshBoundaryType::Free);
        view.set_boundary(MeshBoundaryType::DampedAbsorption);
        assert_eq!(view.mesh.boundary, MeshBoundaryType::DampedAbsorption);
        view.set_boundary(MeshBoundaryType::Clamped);
        assert_eq!(view.mesh.boundary, MeshBoundaryType::Clamped);

        // Simulation stepping and strike excitation
        let initial_energy = view.mesh.calculate_total_energy();
        assert!(initial_energy.is_finite());
        view.step_simulation();
        let energy_after_step = view.mesh.calculate_total_energy();
        assert!(energy_after_step.is_finite());

        view.trigger_strike(0.95);
        let energy_after_strike = view.mesh.calculate_total_energy();
        assert!(energy_after_strike > 0.0);
        assert!(energy_after_strike >= energy_after_step);
    }

    #[test]
    fn test_turn64_pure_waveguide_mesh_3d_projection_and_hit_testing() {
        let view = WaveguideMeshView::new();
        let center = (300.0, 250.0);
        let scale = 120.0;

        // Center grid point (norm_x = 0, norm_y = 0) with gz = 0
        let mid_grid = (MESH_DIM - 1) as f32 * 0.5;
        let (proj_x, proj_y) = view.project_3d_to_canvas(mid_grid, mid_grid, 0.0, center, scale);
        assert!((proj_x - center.0).abs() < 1e-2);
        assert!((proj_y - center.1).abs() < 1e-2);

        // Hit testing strike puck at center
        assert!(view.hit_test_strike_puck(center, center, scale));
        assert!(view.hit_test_strike_puck(
            (center.0 + WAVEGUIDE_PUCK_HIT_RADIUS * 0.8, center.1),
            center,
            scale
        ));
        assert!(!view.hit_test_strike_puck(
            (center.0 + WAVEGUIDE_PUCK_HIT_RADIUS * 1.5, center.1),
            center,
            scale
        ));
    }

    #[test]
    fn test_turn64_pure_plucked_string_view_profiles_and_normalization() {
        let mut view = PluckedStringView::new();
        assert_eq!(view.instrument, PluckedInstrumentViewProfile::AcousticSteel);
        assert!((view.pluck_position_beta - 0.15).abs() < 1e-3);
        assert!((view.strike_velocity - 0.80).abs() < 1e-3);
        assert!((view.hammer_hardness - 0.50).abs() < 1e-3);
        assert_eq!(PLUCK_PUCK_HIT_RADIUS, 22.0);

        // Test normalization transforms
        let beta_norm = PluckedStringView::beta_to_normalized(0.15);
        let recon_beta = PluckedStringView::normalized_to_beta(beta_norm);
        assert!((recon_beta - 0.15).abs() < 1e-3);

        let vel_norm = PluckedStringView::velocity_to_normalized(0.80);
        let recon_vel = PluckedStringView::normalized_to_velocity(vel_norm);
        assert!((recon_vel - 0.80).abs() < 1e-3);

        // Clamp boundaries
        assert_eq!(PluckedStringView::beta_to_normalized(0.0), 0.0);
        assert_eq!(PluckedStringView::beta_to_normalized(1.0), 1.0);
        assert_eq!(PluckedStringView::normalized_to_beta(0.0), MIN_PLUCK_POSITION_BETA);
        assert_eq!(PluckedStringView::normalized_to_beta(1.0), MAX_PLUCK_POSITION_BETA);

        assert_eq!(PluckedStringView::velocity_to_normalized(0.0), 0.0);
        assert_eq!(PluckedStringView::velocity_to_normalized(2.0), 1.0);
        assert_eq!(PluckedStringView::normalized_to_velocity(0.0), MIN_STRIKE_VELOCITY);
        assert_eq!(PluckedStringView::normalized_to_velocity(1.0), MAX_STRIKE_VELOCITY);

        // Test all instrument profiles and nominal physics
        let profiles = [
            (PluckedInstrumentViewProfile::AcousticSteel, "ACOUSTIC GUITAR", 0.25, 4.5, 3.5),
            (PluckedInstrumentViewProfile::ClassicalNylon, "CLASSICAL NYLON", 0.08, 3.2, 2.0),
            (PluckedInstrumentViewProfile::GrandPiano, "GRAND PIANO", 0.45, 8.0, 1.8),
            (PluckedInstrumentViewProfile::Harpsichord, "HARPSICHORD", 0.18, 2.5, 4.0),
            (PluckedInstrumentViewProfile::Harp, "CONCERT HARP", 0.05, 9.5, 1.5),
            (PluckedInstrumentViewProfile::SitarKoto, "SITAR / KOTO", 0.30, 3.8, 6.0),
        ];

        for (prof, expected_name, exp_stiff, exp_t60, exp_detune) in profiles {
            assert_eq!(prof.name(), expected_name);
            let (stiff, t60, detune) = prof.nominal_physics();
            assert!((stiff - exp_stiff).abs() < 1e-3);
            assert!((t60 - exp_t60).abs() < 1e-3);
            assert!((detune - exp_detune).abs() < 1e-3);

            view.set_instrument(prof);
            assert_eq!(view.instrument, prof);
            assert!((view.string_stiffness - exp_stiff).abs() < 1e-3);
            assert!(view.inharmonicity_factor_b > 0.0);
            assert!(view.peak_contact_force_n > 0.0);
            assert!(view.contact_duration_ms >= 0.4 && view.contact_duration_ms <= 4.0);
        }
    }

    #[test]
    fn test_turn64_pure_plucked_string_deflection_and_polarization_orbits() {
        let mut view = PluckedStringView::new();
        view.pluck_position_beta = 0.20;
        view.strike_velocity = 0.90;

        // Spatial deflection tests
        let peak_defl = view.evaluate_string_deflection(0.20);
        assert!((peak_defl - 0.90).abs() < 1e-3);

        let left_slope = view.evaluate_string_deflection(0.10);
        assert!((left_slope - 0.45).abs() < 1e-3);

        let right_slope = view.evaluate_string_deflection(0.60);
        // (1.0 - 0.6) / (1.0 - 0.2) = 0.4 / 0.8 = 0.5 * 0.90 = 0.45
        assert!((right_slope - 0.45).abs() < 1e-3);

        assert_eq!(view.evaluate_string_deflection(0.0), 0.0);
        assert_eq!(view.evaluate_string_deflection(1.0), 0.0);

        // Polarization orbits evaluation
        for s in 0..16 {
            let phase = s as f32 / 16.0 * 2.0 * std::f32::consts::PI;
            let (yh, yv) = view.evaluate_polarization_orbit(phase);
            assert!(yh.is_finite());
            assert!(yv.is_finite());
            assert!(yh.abs() <= 1.5);
            assert!(yv.abs() <= 1.0);
        }

        // Hit testing pluck puck
        let canvas = Rect::new(100.0, 150.0, 300.0, 200.0);
        let puck_x = canvas.x + view.puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;

        assert!(view.hit_test_pluck_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_pluck_puck((puck_x + PLUCK_PUCK_HIT_RADIUS * 0.8, puck_y), canvas));
        assert!(!view.hit_test_pluck_puck((puck_x + PLUCK_PUCK_HIT_RADIUS * 1.5, puck_y), canvas));
    }

    #[test]
    fn test_turn64_pure_ascii_snapshots_deterministic() {
        let wave_view = WaveguideMeshView::new();
        let wave_ascii = wave_view.render_ascii(60, 16);
        assert_eq!(wave_ascii.len(), 16);
        assert!(wave_ascii[0].contains("WAVEGUIDE 3D MESH"));
        assert!(wave_ascii.iter().any(|line| line.contains('@')));

        let wave_snapshot = wave_view.render_ascii_snapshot_str();
        assert!(wave_snapshot.contains("WAVEGUIDE 3D MESH"));
        assert!(wave_snapshot.contains("Membrane"));

        let pluck_view = PluckedStringView::new();
        let pluck_ascii = pluck_view.render_ascii(60, 16);
        assert_eq!(pluck_ascii.len(), 16);
        assert!(pluck_ascii.iter().any(|line| line.contains('#')));
        assert!(pluck_ascii.iter().any(|line| line.contains('O')));

        let pluck_snapshot = pluck_view.render_ascii_snapshot_str();
        assert!(!pluck_snapshot.is_empty());
        assert!(pluck_snapshot.contains('#'));
        assert!(pluck_snapshot.contains('O'));
    }
}

#[cfg(feature = "gui")]
#[cfg(test)]
pub mod gui_tests {
    use std::sync::Arc;
    use eframe::egui;
    use summoner_core::param_bus::{ParamBus, ParamId};
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::command_palette::CommandPalette;

    #[test]
    fn test_turn64_gui_waveguide_and_plucked_string_modal_lifecycles() {
        let mut view = AwardWinningGuiView::new();

        // Waveguide Mesh HUD Lifecycle
        assert!(!view.is_waveguide_mesh_hud_open());
        view.open_waveguide_mesh_hud();
        assert!(view.is_waveguide_mesh_hud_open());
        view.close_waveguide_mesh_hud();
        assert!(!view.is_waveguide_mesh_hud_open());

        // Plucked String HUD Lifecycle
        assert!(!view.is_plucked_string_hud_open());
        view.open_plucked_string_hud();
        assert!(view.is_plucked_string_hud_open());
        view.close_plucked_string_hud();
        assert!(!view.is_plucked_string_hud_open());
    }

    #[test]
    fn test_turn64_gui_headless_render_without_panic() {
        let mut view = AwardWinningGuiView::new();
        view.open_waveguide_mesh_hud();
        view.open_plucked_string_hud();

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        assert!(view.is_waveguide_mesh_hud_open());
        assert!(view.is_plucked_string_hud_open());
    }

    #[test]
    fn test_turn64_gui_live_param_bus_atomic_dispatch() {
        let mut view = AwardWinningGuiView::new();
        let mut bus = ParamBus::new();

        let track_id = view.tracks.get(view.selected_track_idx).map(|t| t.id).unwrap_or(1);
        let slot = view.device_rack_state.selected_chain_idx;
        let base = track_id as u32 * 1000 + slot as u32 * 20;

        for i in 0..8 {
            bus.register(ParamId(base + i), 0.0);
        }

        let arc_bus = Arc::new(bus);
        view.live_param_bus = Some(Arc::clone(&arc_bus));

        // Test Waveguide Mesh HUD sync
        view.waveguide_mesh_view.strike_puck_pos = (0.72, 0.44);
        view.waveguide_mesh_view.view_elevation_deg = 48.0;
        view.waveguide_mesh_view.view_azimuth_deg = 62.0;
        view.waveguide_mesh_view.wireframe_density = 24;
        let expected_energy = view.waveguide_mesh_view.mesh.calculate_total_energy();
        view.sync_waveguide_mesh_hud_state();

        assert_eq!(arc_bus.get(ParamId(base)), Some(0.72));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(0.44));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(48.0));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(62.0));
        assert_eq!(arc_bus.get(ParamId(base + 4)), Some(24.0));
        assert_eq!(arc_bus.get(ParamId(base + 5)), Some(expected_energy));

        assert_eq!(view.device_rack_state.node_param_values.get("strike_pos_x"), Some(&0.72));
        assert_eq!(view.device_rack_state.node_param_values.get("strike_pos_y"), Some(&0.44));
        assert_eq!(view.inspector_state.node_param_values.get("strike_pos_x"), Some(&0.72));
        assert_eq!(view.inspector_state.node_param_values.get("strike_pos_y"), Some(&0.44));

        // Test Plucked String HUD sync
        view.plucked_string_view.pluck_position_beta = 0.32;
        view.plucked_string_view.strike_velocity = 0.88;
        view.plucked_string_view.string_stiffness = 0.35;
        view.plucked_string_view.hammer_hardness = 0.65;
        view.plucked_string_view.palm_mute_damping = 0.15;
        view.plucked_string_view.polarization_coupling = 0.42;
        view.plucked_string_view.peak_contact_force_n = 22.4;
        view.plucked_string_view.contact_duration_ms = 1.25;
        view.sync_plucked_string_hud_state();

        assert_eq!(arc_bus.get(ParamId(base)), Some(0.32));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(0.88));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(0.35));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(0.65));
        assert_eq!(arc_bus.get(ParamId(base + 4)), Some(0.15));
        assert_eq!(arc_bus.get(ParamId(base + 5)), Some(0.42));
        assert_eq!(arc_bus.get(ParamId(base + 6)), Some(22.4));
        assert_eq!(arc_bus.get(ParamId(base + 7)), Some(1.25));

        assert_eq!(view.device_rack_state.node_param_values.get("pluck_position_beta"), Some(&0.32));
        assert_eq!(view.device_rack_state.node_param_values.get("strike_velocity"), Some(&0.88));
        assert_eq!(view.inspector_state.node_param_values.get("pluck_position_beta"), Some(&0.32));
        assert_eq!(view.inspector_state.node_param_values.get("strike_velocity"), Some(&0.88));
    }

    #[test]
    fn test_turn64_gui_rack_and_inspector_trigger_propagation() {
        let mut view = AwardWinningGuiView::new();

        // 1. Trigger Waveguide Mesh from Inspector
        view.inspector_state.requested_open_waveguide_mesh_hud = true;
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.inspector_state.requested_open_waveguide_mesh_hud);
        assert!(view.is_waveguide_mesh_hud_open());
        view.close_waveguide_mesh_hud();

        // 2. Trigger Plucked String from Inspector
        view.inspector_state.requested_open_plucked_string_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.inspector_state.requested_open_plucked_string_hud);
        assert!(view.is_plucked_string_hud_open());
        view.close_plucked_string_hud();

        // 3. Trigger Waveguide Mesh from Device Rack
        view.device_rack_state.requested_open_waveguide_mesh_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.device_rack_state.requested_open_waveguide_mesh_hud);
        assert!(view.is_waveguide_mesh_hud_open());
        view.close_waveguide_mesh_hud();

        // 4. Trigger Plucked String from Device Rack
        view.device_rack_state.requested_open_plucked_string_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.device_rack_state.requested_open_plucked_string_hud);
        assert!(view.is_plucked_string_hud_open());
    }

    #[test]
    fn test_turn64_gui_command_palette_actions_registered() {
        let palette = CommandPalette::new();
        let has_waveguide = palette.actions.iter().any(|a| a.action_id == "open_waveguide_mesh_hud");
        let has_plucked_string = palette.actions.iter().any(|a| a.action_id == "open_plucked_string_hud");

        assert!(has_waveguide, "Expected open_waveguide_mesh_hud in command palette");
        assert!(has_plucked_string, "Expected open_plucked_string_hud in command palette");
    }
}
