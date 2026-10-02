#![allow(clippy::all)]

//! Test suite for Turn #73 (Turn #36 deliverable) — Physical Modeling Acoustic Membrane Percussion & Coupled 2D Membrane/Plate Resonator HUDs Convergence.

#[cfg(test)]
pub mod pure_tests {
    use crate::layout_math::Rect;
    use crate::views::membrane_plate_view::{
        BoundaryClamping, MembranePlateView, PlateProfile, MEMBRANE_PLATE_PUCK_HIT_RADIUS,
    };
    use crate::views::membrane_resonator_view::{
        MembraneMaterial, MembraneResonatorView, MEMBRANE_PUCK_HIT_RADIUS,
    };
    use summoner_core::param_bus::ParamId;

    #[test]
    fn test_turn73_pure_membrane_resonator_view_initialization_and_materials() {
        let mut view = MembraneResonatorView::new();
        assert_eq!(view.material, MembraneMaterial::MylarDrumhead);
        assert!((view.membrane_radius_m - 0.178).abs() < 1e-3);
        assert!((view.membrane_tension_nm - 3500.0).abs() < 1e-3);
        assert_eq!(MEMBRANE_PUCK_HIT_RADIUS, 22.0);

        // Material profiles
        view.set_material(MembraneMaterial::CalfskinVintage);
        assert_eq!(view.material, MembraneMaterial::CalfskinVintage);
        assert!((view.material.surface_density_kg_m2() - 0.45).abs() < 1e-3);
        assert!((view.material.internal_damping_coeff() - 0.042).abs() < 1e-3);

        view.set_material(MembraneMaterial::TitaniumFoil);
        assert_eq!(view.material, MembraneMaterial::TitaniumFoil);
        assert!((view.material.surface_density_kg_m2() - 0.72).abs() < 1e-3);

        view.set_material(MembraneMaterial::SiliconeElastic);
        assert_eq!(view.material, MembraneMaterial::SiliconeElastic);
        assert!((view.material.surface_density_kg_m2() - 0.58).abs() < 1e-3);

        view.set_material(MembraneMaterial::CarbonComposite);
        assert_eq!(view.material, MembraneMaterial::CarbonComposite);
        assert!((view.material.surface_density_kg_m2() - 0.32).abs() < 1e-3);

        // Normalization conversions
        let norm_t = MembraneResonatorView::tension_to_normalized(4250.0);
        assert!((norm_t - 0.5).abs() < 1e-3);
        let t_back = MembraneResonatorView::normalized_to_tension(norm_t);
        assert!((t_back - 4250.0).abs() < 1e-3);

        // Physics simulation
        view.update_physics_simulation();
        assert!(view.fundamental_freq_hz >= 20.0 && view.fundamental_freq_hz <= 2000.0);

        // Displacement evaluation
        let disp_center = view.evaluate_membrane_displacement(0.0, 0.0);
        let disp_edge = view.evaluate_membrane_displacement(1.0, 0.0);
        assert!(disp_center > disp_edge);

        // Hit testing
        let canvas = Rect::new(20.0, 56.0, 420.0, 224.0);
        let puck_x = canvas.x + view.strike_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.strike_puck_pos.1) * canvas.height;
        assert!(view.hit_test_strike_puck((puck_x, puck_y), canvas));
        assert!(!view.hit_test_strike_puck((puck_x + 100.0, puck_y), canvas));

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains('+'));
        assert!(snap.contains('O'));
    }

    #[test]
    fn test_turn73_pure_membrane_plate_view_initialization_and_profiles() {
        let mut view = MembranePlateView::new();
        assert_eq!(view.profile, PlateProfile::CircularTympanum);
        assert_eq!(view.boundary, BoundaryClamping::ClampedRigid);
        assert_eq!(MEMBRANE_PLATE_PUCK_HIT_RADIUS, 22.0);

        // Profile tests
        view.set_profile(PlateProfile::RectangularSteelPlate);
        assert_eq!(view.profile, PlateProfile::RectangularSteelPlate);
        assert!((view.thickness_mm - 3.2).abs() < 1e-3);

        view.set_profile(PlateProfile::GongTamTam);
        assert_eq!(view.profile, PlateProfile::GongTamTam);
        assert!((view.thickness_mm - 1.8).abs() < 1e-3);

        view.set_profile(PlateProfile::MarimbaRosewoodBar);
        assert_eq!(view.profile, PlateProfile::MarimbaRosewoodBar);
        assert!((view.thickness_mm - 18.5).abs() < 1e-3);

        // Boundary tests
        view.set_boundary(BoundaryClamping::FreeEdge);
        assert_eq!(view.boundary, BoundaryClamping::FreeEdge);
        assert!((view.boundary.impedance_factor() - 0.15).abs() < 1e-3);

        view.set_boundary(BoundaryClamping::SimplySupported);
        assert_eq!(view.boundary, BoundaryClamping::SimplySupported);
        assert!((view.boundary.impedance_factor() - 0.65).abs() < 1e-3);

        // Conversions
        let norm_thick = MembranePlateView::thickness_to_normalized(12.75);
        assert!((norm_thick - 0.5).abs() < 1e-3);
        let thick_back = MembranePlateView::normalized_to_thickness(norm_thick);
        assert!((thick_back - 12.75).abs() < 1e-3);

        let norm_t = MembranePlateView::tension_to_normalized(5050.0);
        assert!((norm_t - 0.5).abs() < 1e-3);
        let t_back = MembranePlateView::normalized_to_tension(norm_t);
        assert!((t_back - 5050.0).abs() < 1e-3);

        let norm_asp = MembranePlateView::aspect_to_normalized(1.25);
        assert!((norm_asp - 0.5).abs() < 1e-3);
        let asp_back = MembranePlateView::normalized_to_aspect(norm_asp);
        assert!((asp_back - 1.25).abs() < 1e-3);

        // Simulation update & modal frequencies
        view.update_physics_simulation();
        assert_eq!(view.modal_frequencies.len(), 6);
        assert_eq!(view.modal_amplitudes.len(), 6);
        assert!(view.modal_frequencies[0] < view.modal_frequencies[5]);
        assert!(view.contact_time_ms > 0.0);

        // Hit testing
        let canvas = Rect::new(20.0, 56.0, 420.0, 224.0);
        let puck_x = canvas.x + view.strike_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.strike_puck_pos.1) * canvas.height;
        assert!(view.hit_test_strike_puck((puck_x, puck_y), canvas));
        assert!(!view.hit_test_strike_puck((puck_x + 100.0, puck_y), canvas));

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains('+'));
        assert!(snap.contains('@'));
    }

    #[test]
    fn test_turn73_pure_slot_arithmetic_and_param_bus_isolation() {
        let track_id: u32 = 5;
        let slot_id: u32 = 2;
        let base_channel = track_id * 1000 + slot_id * 20;

        let pid_pos = ParamId(base_channel);
        let pid_ten = ParamId(base_channel + 1);
        let pid_vel = ParamId(base_channel + 2);
        let pid_rim = ParamId(base_channel + 3);

        assert_eq!(pid_pos.0, 5040);
        assert_eq!(pid_ten.0, 5041);
        assert_eq!(pid_vel.0, 5042);
        assert_eq!(pid_rim.0, 5043);

        let next_slot_pid = ParamId(track_id * 1000 + (slot_id + 1) * 20);
        assert_eq!(next_slot_pid.0, 5060);
        assert!(pid_rim.0 < next_slot_pid.0);
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::command_palette::CommandPalette;
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use eframe::egui;
    use std::sync::Arc;
    use summoner_core::param_bus::{ParamBus, ParamId};

    #[test]
    fn test_turn73_gui_award_winning_view_headless_rendering_without_panic() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // Open Membrane Resonator modal
        view.open_membrane_resonator_hud();
        assert!(view.is_membrane_resonator_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_membrane_resonator_hud();
        assert!(!view.is_membrane_resonator_hud_open());

        // Open Membrane Plate modal
        view.open_membrane_plate_hud();
        assert!(view.is_membrane_plate_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_membrane_plate_hud();
        assert!(!view.is_membrane_plate_hud_open());
    }

    #[test]
    fn test_turn73_gui_live_parambus_slot_dispatch() {
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

        // 1. Sync Membrane Resonator HUD
        view.membrane_resonator_view.strike_pos_norm = (0.4, 0.3);
        view.membrane_resonator_view.membrane_tension_nm = 4500.0;
        view.membrane_resonator_view.strike_velocity = 0.90;
        view.membrane_resonator_view.rim_shot_coupling = 0.25;
        view.sync_membrane_resonator_hud_state();

        let expected_pos = (0.4_f32.powi(2) + 0.3_f32.powi(2)).sqrt().min(1.0);
        assert_eq!(arc_bus.get(ParamId(base)), Some(expected_pos));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(4500.0));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(0.90));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(0.25));

        assert_eq!(
            view.device_rack_state.node_param_values.get("membrane_tension_n"),
            Some(&4500.0)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("membrane_tension_n"),
            Some(&4500.0)
        );

        // 2. Sync Membrane Plate HUD
        view.membrane_plate_view.thickness_mm = 8.5;
        view.membrane_plate_view.tension_nm = 6200.0;
        view.membrane_plate_view.mallet_hardness = 0.75;
        view.membrane_plate_view.aspect_ratio = 1.40;
        view.sync_membrane_plate_hud_state();

        assert_eq!(arc_bus.get(ParamId(base)), Some(8.5));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(6200.0));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(0.75));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(1.40));

        assert_eq!(
            view.device_rack_state.node_param_values.get("thickness_mm"),
            Some(&8.5)
        );
        assert_eq!(
            view.device_rack_state.node_param_values.get("tension_nm"),
            Some(&6200.0)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("thickness_mm"),
            Some(&8.5)
        );
    }

    #[test]
    fn test_turn73_gui_rack_and_inspector_trigger_propagation() {
        let mut view = AwardWinningGuiView::new();

        view.device_rack_state.requested_open_membrane_resonator_hud = true;
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_membrane_resonator_hud_open());
        assert!(!view.device_rack_state.requested_open_membrane_resonator_hud);

        view.close_membrane_resonator_hud();
        assert!(!view.is_membrane_resonator_hud_open());

        view.inspector_state.requested_open_membrane_plate_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_membrane_plate_hud_open());
        assert!(!view.inspector_state.requested_open_membrane_plate_hud);
    }

    #[test]
    fn test_turn73_gui_command_palette_action_registration() {
        let palette = CommandPalette::new();
        let mr_action = palette
            .actions
            .iter()
            .find(|a| a.action_id == "open_membrane_resonator_hud");
        assert!(mr_action.is_some());
        let mr = mr_action.unwrap();
        assert_eq!(mr.category, "Physical Modeling");
        assert!(mr.label.contains("Membrane Resonator"));

        let mp_action = palette
            .actions
            .iter()
            .find(|a| a.action_id == "open_membrane_plate_hud");
        assert!(mp_action.is_some());
        let mp = mp_action.unwrap();
        assert_eq!(mp.category, "Physical Modeling");
        assert!(mp.label.contains("Membrane & Plate"));
    }
}
