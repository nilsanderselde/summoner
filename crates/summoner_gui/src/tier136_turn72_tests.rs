#![allow(clippy::all)]

//! Test suite for Turn #72 (Turn #35 deliverable) — Neural Acoustic Wavefront Boundary Reflection & Continuous Latent Diffractive Spherical Wave Propagation HUDs Convergence.

#[cfg(test)]
pub mod pure_tests {
    use crate::layout_math::Rect;
    use crate::views::diffractive_propagation_view::{
        DiffractionModelType, DiffractivePropagationView, DIFFRACTION_PUCK_HIT_RADIUS,
    };
    use crate::views::wavefront_reflection_view::{
        RoomArchitectureType, WavefrontReflectionView, WAVEFRONT_PUCK_HIT_RADIUS,
    };
    use summoner_core::param_bus::ParamId;

    #[test]
    fn test_turn72_pure_wavefront_reflection_view_initialization_and_architectures() {
        let mut view = WavefrontReflectionView::new();
        assert_eq!(view.architecture, RoomArchitectureType::ConcertHallHorseshoe);
        assert!((view.surface_scattering_coeff - 0.45).abs() < 1e-3);
        assert!((view.boundary_absorption_alpha - 0.20).abs() < 1e-3);
        assert_eq!(view.ray_count, 2500);
        assert!((view.early_window_ms - 80.0).abs() < 1e-3);
        assert_eq!(WAVEFRONT_PUCK_HIT_RADIUS, 22.0);

        // Architecture profiles
        view.set_architecture(RoomArchitectureType::ShoeboxStudioControl);
        assert_eq!(view.architecture, RoomArchitectureType::ShoeboxStudioControl);
        assert!((view.surface_scattering_coeff - 0.25).abs() < 1e-3);
        assert!((view.boundary_absorption_alpha - 0.60).abs() < 1e-3);
        assert_eq!(view.ray_count, 1800);
        assert!((view.early_window_ms - 35.0).abs() < 1e-3);

        view.set_architecture(RoomArchitectureType::CathedralStoneVault);
        assert_eq!(view.architecture, RoomArchitectureType::CathedralStoneVault);
        assert!((view.surface_scattering_coeff - 0.15).abs() < 1e-3);
        assert!((view.boundary_absorption_alpha - 0.08).abs() < 1e-3);
        assert_eq!(view.ray_count, 4000);

        view.set_architecture(RoomArchitectureType::ModularVocalBooth);
        assert_eq!(view.architecture, RoomArchitectureType::ModularVocalBooth);
        assert!((view.surface_scattering_coeff - 0.85).abs() < 1e-3);
        assert!((view.boundary_absorption_alpha - 0.80).abs() < 1e-3);

        view.set_architecture(RoomArchitectureType::AsymmetricalGallery);
        assert_eq!(view.architecture, RoomArchitectureType::AsymmetricalGallery);
        assert!((view.surface_scattering_coeff - 0.55).abs() < 1e-3);
        assert!((view.boundary_absorption_alpha - 0.35).abs() < 1e-3);

        // Simulation update & reflection taps decay
        view.update_wavefront_simulation();
        assert!(view.reflection_energy_taps[0] > view.reflection_energy_taps[15]);
        assert_eq!(view.reflection_energy_taps.len(), 16);

        // Hit testing
        let canvas = Rect::new(20.0, 56.0, 420.0, 224.0);
        view.puck_pos = (0.5, 0.5);
        let px = canvas.x + 0.5 * canvas.width;
        let py = canvas.y + 0.5 * canvas.height;
        assert!(view.hit_test_wavefront_puck((px, py), canvas));
        assert!(!view.hit_test_wavefront_puck((px + 100.0, py), canvas));

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains('W'));
    }

    #[test]
    fn test_turn72_pure_diffractive_propagation_view_initialization_and_models() {
        let mut view = DiffractivePropagationView::new();
        assert_eq!(view.model, DiffractionModelType::SphericalHelmholtzKirchhoff);
        assert!((view.diffraction_angle_deg - 75.0).abs() < 1e-3);
        assert!((view.source_distance_m - 4.5).abs() < 1e-3);
        assert_eq!(DIFFRACTION_PUCK_HIT_RADIUS, 22.0);

        // Models
        view.set_model(DiffractionModelType::BiotTolstoyMedwinEdge);
        assert_eq!(view.model, DiffractionModelType::BiotTolstoyMedwinEdge);
        assert!((view.diffraction_angle_deg - 95.0).abs() < 1e-3);
        assert!((view.source_distance_m - 3.2).abs() < 1e-3);

        view.set_model(DiffractionModelType::NeuralLatentWavefield);
        assert_eq!(view.model, DiffractionModelType::NeuralLatentWavefield);
        assert!((view.diffraction_angle_deg - 60.0).abs() < 1e-3);
        assert!((view.source_distance_m - 5.0).abs() < 1e-3);

        view.set_model(DiffractionModelType::ThinWedgeShadow);
        assert_eq!(view.model, DiffractionModelType::ThinWedgeShadow);
        assert!((view.diffraction_angle_deg - 120.0).abs() < 1e-3);
        assert!((view.source_distance_m - 6.5).abs() < 1e-3);

        view.set_model(DiffractionModelType::CurvedPillarScattering);
        assert_eq!(view.model, DiffractionModelType::CurvedPillarScattering);
        assert!((view.diffraction_angle_deg - 45.0).abs() < 1e-3);
        assert!((view.source_distance_m - 2.8).abs() < 1e-3);

        // Conversions
        let norm_ang = DiffractivePropagationView::angle_to_normalized(90.0);
        assert!((norm_ang - 0.5).abs() < 1e-3);
        let ang_back = DiffractivePropagationView::normalized_to_angle(norm_ang);
        assert!((ang_back - 90.0).abs() < 1e-3);

        let norm_dist = DiffractivePropagationView::distance_to_normalized(12.75);
        assert!((norm_dist - 0.5).abs() < 1e-3);
        let dist_back = DiffractivePropagationView::normalized_to_distance(norm_dist);
        assert!((dist_back - 12.75).abs() < 1e-3);

        // Simulation update & frequency attenuation curve
        view.update_diffraction_simulation();
        assert_eq!(view.freq_attenuation_curve_db.len(), 8);
        assert!(view.shadow_attenuation_db <= 0.0);
        assert!(view.edge_delay_ms > 0.0);

        // Hit testing
        let canvas = Rect::new(20.0, 56.0, 420.0, 224.0);
        view.puck_pos = (0.5, 0.5);
        let px = canvas.x + 0.5 * canvas.width;
        let py = canvas.y + 0.5 * canvas.height;
        assert!(view.hit_test_diffraction_puck((px, py), canvas));
        assert!(!view.hit_test_diffraction_puck((px + 100.0, py), canvas));

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains('D'));
    }

    #[test]
    fn test_turn72_pure_slot_arithmetic_and_param_bus_isolation() {
        let track_id: u32 = 4;
        let slot_id: u32 = 3;
        let base_channel = track_id * 1000 + slot_id * 20;

        let pid_scat = ParamId(base_channel);
        let pid_abs = ParamId(base_channel + 1);
        let pid_neural = ParamId(base_channel + 2);
        let pid_rays = ParamId(base_channel + 3);

        assert_eq!(pid_scat.0, 4060);
        assert_eq!(pid_abs.0, 4061);
        assert_eq!(pid_neural.0, 4062);
        assert_eq!(pid_rays.0, 4063);

        let next_slot_pid = ParamId(track_id * 1000 + (slot_id + 1) * 20);
        assert_eq!(next_slot_pid.0, 4080);
        assert!(pid_rays.0 < next_slot_pid.0);
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::command_palette::CommandPalette;
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use eframe::egui;
    use summoner_core::param_bus::{ParamBus, ParamId};
    use std::sync::Arc;

    #[test]
    fn test_turn72_gui_award_winning_view_headless_rendering_without_panic() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // Open Wavefront Reflection modal
        view.open_wavefront_reflection_hud();
        assert!(view.is_wavefront_reflection_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_wavefront_reflection_hud();
        assert!(!view.is_wavefront_reflection_hud_open());

        // Open Diffractive Propagation modal
        view.open_diffractive_propagation_hud();
        assert!(view.is_diffractive_propagation_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_diffractive_propagation_hud();
        assert!(!view.is_diffractive_propagation_hud_open());
    }

    #[test]
    fn test_turn72_gui_live_parambus_slot_dispatch() {
        let mut view = AwardWinningGuiView::new();
        let mut bus = ParamBus::new();
        let track_id = view.tracks.get(view.selected_track_idx).map(|t| t.id).unwrap_or(1);
        let slot = view.device_rack_state.selected_chain_idx;
        let base = track_id as u32 * 1000 + slot as u32 * 20;

        for i in 0..10 {
            bus.register(ParamId(base + i), 0.0);
        }

        let arc_bus = Arc::new(bus);
        view.live_param_bus = Some(Arc::clone(&arc_bus));

        // 1. Sync Wavefront Reflection HUD
        view.wavefront_reflection_view.surface_scattering_coeff = 0.65;
        view.wavefront_reflection_view.boundary_absorption_alpha = 0.35;
        view.wavefront_reflection_view.neural_kernel_depth = 0.85;
        view.wavefront_reflection_view.ray_count = 3200;
        view.sync_wavefront_reflection_hud_state();

        assert_eq!(arc_bus.get(ParamId(base)), Some(0.65));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(0.35));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(0.85));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(3200.0));

        assert_eq!(view.device_rack_state.node_param_values.get("surface_scattering_coeff"), Some(&0.65));
        assert_eq!(view.device_rack_state.node_param_values.get("boundary_absorption_alpha"), Some(&0.35));
        assert_eq!(view.inspector_state.node_param_values.get("surface_scattering_coeff"), Some(&0.65));

        // 2. Sync Diffractive Propagation HUD
        view.diffractive_propagation_view.diffraction_angle_deg = 110.0;
        view.diffractive_propagation_view.source_distance_m = 8.5;
        view.diffractive_propagation_view.boundary_absorption_alpha = 0.40;
        view.diffractive_propagation_view.edge_wedge_angle_deg = 75.0;
        view.sync_diffractive_propagation_hud_state();

        assert_eq!(arc_bus.get(ParamId(base)), Some(110.0));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(8.5));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(0.40));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(75.0));

        assert_eq!(view.device_rack_state.node_param_values.get("diffraction_angle_deg"), Some(&110.0));
        assert_eq!(view.device_rack_state.node_param_values.get("source_distance_m"), Some(&8.5));
        assert_eq!(view.inspector_state.node_param_values.get("diffraction_angle_deg"), Some(&110.0));
    }

    #[test]
    fn test_turn72_gui_rack_and_inspector_trigger_propagation() {
        let mut view = AwardWinningGuiView::new();

        view.device_rack_state.requested_open_wavefront_reflection_hud = true;
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_wavefront_reflection_hud_open());
        assert!(!view.device_rack_state.requested_open_wavefront_reflection_hud);

        view.close_wavefront_reflection_hud();
        assert!(!view.is_wavefront_reflection_hud_open());

        view.inspector_state.requested_open_diffractive_propagation_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_diffractive_propagation_hud_open());
        assert!(!view.inspector_state.requested_open_diffractive_propagation_hud);
    }

    #[test]
    fn test_turn72_gui_command_palette_action_registration() {
        let palette = CommandPalette::new();
        let wf_action = palette.actions.iter().find(|a| a.action_id == "open_wavefront_reflection_hud");
        assert!(wf_action.is_some());
        let wf = wf_action.unwrap();
        assert_eq!(wf.category, "Physical Modeling");
        assert!(wf.label.contains("Wavefront Reflection"));

        let diff_action = palette.actions.iter().find(|a| a.action_id == "open_diffractive_propagation_hud");
        assert!(diff_action.is_some());
        let diff = diff_action.unwrap();
        assert_eq!(diff.category, "Physical Modeling");
        assert!(diff.label.contains("Diffractive"));
    }
}
