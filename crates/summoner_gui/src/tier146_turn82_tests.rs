#![allow(clippy::all)]

//! Test suite for Turn #82 (Sprint Turn #8) — MPEG-H 3D Dynamic Object Trajectory HUD & NHK 22.2 3-Layer Spherical Immersion HUD Convergence.

#[cfg(test)]
pub mod pure_tests {
    use crate::layout_math::Rect;
    use crate::views::mpegh_trajectory_view::{
        MpeghTrajectoryProfile, MpeghTrajectoryView, MPEGH_TRAJ_PUCK_HIT_RADIUS,
    };
    use crate::views::nhk222_immersion_view::{
        Nhk222ImmersionView, Nhk222ProfileType, NHK222_PUCK_HIT_RADIUS,
    };
    use summoner_core::param_bus::ParamId;

    #[test]
    fn test_turn82_pure_mpegh_trajectory_view_initialization_and_simulation() {
        let mut view = MpeghTrajectoryView::new();
        assert_eq!(view.profile, MpeghTrajectoryProfile::Mpegh3DHelicalAscent);
        assert!((view.azimuth_deg - 45.0).abs() < 1e-3);
        assert!((view.elevation_deg - 35.0).abs() < 1e-3);
        assert!((view.distance_m - 4.5).abs() < 1e-3);
        assert!((view.trajectory_speed_hz - 0.25).abs() < 1e-3);
        assert!((view.object_spread - 0.30).abs() < 1e-3);
        assert_eq!(MPEGH_TRAJ_PUCK_HIT_RADIUS, 22.0);

        // Coordinate normalizations
        for az in [-180.0, -90.0, 0.0, 90.0, 180.0] {
            let norm = MpeghTrajectoryView::azimuth_to_normalized(az);
            assert!((0.0..=1.0).contains(&norm));
            let back = MpeghTrajectoryView::normalized_to_azimuth(norm);
            assert!((back - az).abs() < 1e-4);
        }

        for el in [-90.0, -45.0, 0.0, 45.0, 90.0] {
            let norm = MpeghTrajectoryView::elevation_to_normalized(el);
            assert!((0.0..=1.0).contains(&norm));
            let back = MpeghTrajectoryView::normalized_to_elevation(norm);
            assert!((back - el).abs() < 1e-4);
        }

        // Profile switching
        let profiles = [
            (MpeghTrajectoryProfile::Mpegh3DHelicalAscent, 45.0, 35.0, 4.5),
            (MpeghTrajectoryProfile::Mpegh3DOverheadFlyby, 0.0, 75.0, 2.0),
            (MpeghTrajectoryProfile::Mpegh3DEquatorialOrbit, -90.0, 0.0, 5.0),
            (MpeghTrajectoryProfile::Mpegh3DPendulumSwing, 30.0, -20.0, 3.5),
            (MpeghTrajectoryProfile::Mpegh3DMultiObjectMaster, -45.0, 25.0, 6.0),
        ];

        for (prof, exp_az, exp_el, exp_dist) in profiles {
            view.set_trajectory(prof);
            assert_eq!(view.profile, prof);
            assert!((view.azimuth_deg - exp_az).abs() < 1e-3);
            assert!((view.elevation_deg - exp_el).abs() < 1e-3);
            assert!((view.distance_m - exp_dist).abs() < 1e-3);
            assert!(!prof.profile_name().is_empty());
        }

        // Simulation update: 8 speaker group gains
        view.azimuth_deg = 0.0; // Directly Front
        view.elevation_deg = 0.0;
        view.distance_m = 3.0;
        view.update_mpegh_simulation();

        // 8 groups: [fl, fr, c, lfe, sl, sr, tf, tb]
        let gains = view.speaker_group_gains;
        assert_eq!(gains.len(), 8);
        assert!(gains[2] > 0.5, "Center channel should receive dominant front energy");
        assert!(view.vbap_energy_focus >= 0.3 && view.vbap_energy_focus <= 1.0);

        // Puck hit testing
        let canvas = Rect::new(20.0, 100.0, 420.0, 230.0);
        view.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(view.hit_test_mpegh_puck((center_x, center_y), canvas));
        assert!(!view.hit_test_mpegh_puck((center_x + 100.0, center_y), canvas));

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains('+'));
        assert!(snap.contains('|'));
    }

    #[test]
    fn test_turn82_pure_nhk222_immersion_view_initialization_and_simulation() {
        let mut view = Nhk222ImmersionView::new();
        assert_eq!(view.profile, Nhk222ProfileType::OrchestralSymphony222);
        assert!((view.azimuth_deg - 0.0).abs() < 1e-3);
        assert!((view.elevation_deg - 35.0).abs() < 1e-3);
        assert!((view.distance_m - 8.0).abs() < 1e-3);
        assert_eq!(NHK222_PUCK_HIT_RADIUS, 22.0);

        // Profile switching
        let profiles = [
            (Nhk222ProfileType::OrchestralSymphony222, 0.0, 35.0, 8.0),
            (Nhk222ProfileType::StadiumSportsLive222, -45.0, 45.0, 14.0),
            (Nhk222ProfileType::TheatricalFilmEpic, 60.0, 20.0, 4.5),
            (Nhk222ProfileType::DomePlanetarium, 0.0, 75.0, 10.0),
            (Nhk222ProfileType::Binaural222Headphones, -30.0, 15.0, 1.5),
        ];

        for (prof, exp_az, exp_el, exp_dist) in profiles {
            view.set_profile(prof);
            assert_eq!(view.profile, prof);
            assert!((view.azimuth_deg - exp_az).abs() < 1e-3);
            assert!((view.elevation_deg - exp_el).abs() < 1e-3);
            assert!((view.distance_m - exp_dist).abs() < 1e-3);
            assert!(!prof.profile_name().is_empty());
        }

        // Coordinate normalizations
        for az in [-180.0, -90.0, 0.0, 90.0, 180.0] {
            let norm = Nhk222ImmersionView::azimuth_to_normalized(az);
            assert!((0.0..=1.0).contains(&norm));
            let back = Nhk222ImmersionView::normalized_to_azimuth(norm);
            assert!((back - az).abs() < 1e-4);
        }

        for el in [-90.0, -45.0, 0.0, 45.0, 90.0] {
            let norm = Nhk222ImmersionView::elevation_to_normalized(el);
            assert!((0.0..=1.0).contains(&norm));
            let back = Nhk222ImmersionView::normalized_to_elevation(norm);
            assert!((back - el).abs() < 1e-4);
        }

        // Simulation update: Top dome vs ear-level immersion
        view.elevation_deg = 80.0; // High dome ceiling
        view.update_nhk222_simulation();
        let top_e = view.top_layer_energy;
        let bot_e = view.bottom_layer_energy;
        assert!(top_e > bot_e, "Top layer energy should exceed bottom layer for high elevation");

        view.elevation_deg = -60.0; // Bottom layer floor
        view.update_nhk222_simulation();
        let top_e2 = view.top_layer_energy;
        let bot_e2 = view.bottom_layer_energy;
        assert!(bot_e2 > top_e2, "Bottom layer energy should exceed top layer for negative elevation");

        assert_eq!(view.speaker_energies.len(), 24);

        // Puck hit testing
        let canvas = Rect::new(20.0, 100.0, 420.0, 230.0);
        view.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(view.hit_test_nhk222_puck((center_x, center_y), canvas));
        assert!(!view.hit_test_nhk222_puck((center_x + 100.0, center_y), canvas));

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains('+'));
        assert!(snap.contains('|'));
    }

    #[test]
    fn test_turn82_pure_parambus_slot_arithmetic_channel_isolation() {
        for track_id in 1u32..=4 {
            for slot_idx in 0u32..=3 {
                let base = track_id * 1000 + slot_idx * 20;

                // MPEG-H Trajectory: 6 parameters (offsets 0..=5)
                let trj_az = ParamId(base);
                let trj_el = ParamId(base + 1);
                let trj_dist = ParamId(base + 2);
                let trj_spd = ParamId(base + 3);
                let trj_spr = ParamId(base + 4);
                let trj_foc = ParamId(base + 5);

                assert_eq!(trj_az.0, base);
                assert_eq!(trj_el.0, base + 1);
                assert_eq!(trj_dist.0, base + 2);
                assert_eq!(trj_spd.0, base + 3);
                assert_eq!(trj_spr.0, base + 4);
                assert_eq!(trj_foc.0, base + 5);

                // NHK 22.2 Immersion: 7 parameters (offsets 0..=6)
                let nhk_az = ParamId(base);
                let nhk_el = ParamId(base + 1);
                let nhk_dist = ParamId(base + 2);
                let nhk_top = ParamId(base + 3);
                let nhk_mid = ParamId(base + 4);
                let nhk_bot = ParamId(base + 5);
                let nhk_lfe = ParamId(base + 6);

                assert_eq!(nhk_az.0, base);
                assert_eq!(nhk_el.0, base + 1);
                assert_eq!(nhk_dist.0, base + 2);
                assert_eq!(nhk_top.0, base + 3);
                assert_eq!(nhk_mid.0, base + 4);
                assert_eq!(nhk_bot.0, base + 5);
                assert_eq!(nhk_lfe.0, base + 6);

                // Verify slot boundary isolation (stays within slot allocation of 20 IDs)
                let next_slot_base = track_id * 1000 + (slot_idx + 1) * 20;
                assert!(trj_foc.0 < next_slot_base);
                assert!(nhk_lfe.0 < next_slot_base);
            }
        }
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::command_palette::CommandPalette;
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::views::modern_device_rack::{show_modern_device_rack_with_context, ModernDeviceRackState};
    use crate::views::modern_inspector::{show_modern_inspector_with_context, ModernInspectorState};
    use eframe::egui;
    use std::sync::Arc;
    use summoner_core::param_bus::{ParamBus, ParamId};

    #[test]
    fn test_turn82_gui_award_winning_view_mpegh_trajectory_and_nhk222_immersion_modal_lifecycle() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // 1. MPEG-H 3D Dynamic Trajectory HUD
        view.open_mpegh_trajectory_hud();
        assert!(view.is_mpegh_trajectory_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_mpegh_trajectory_hud();
        assert!(!view.is_mpegh_trajectory_hud_open());

        // 2. NHK 22.2 3-Layer Spherical Immersion HUD
        view.open_nhk222_immersion_hud();
        assert!(view.is_nhk222_immersion_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_nhk222_immersion_hud();
        assert!(!view.is_nhk222_immersion_hud_open());
    }

    #[test]
    fn test_turn82_gui_parambus_slot_dispatch_mpegh_trajectory_and_nhk222_immersion() {
        let mut view = AwardWinningGuiView::new();
        let mut bus = ParamBus::new();

        let track_id = 1u32;
        let slot_idx = 0u32;
        let base = track_id * 1000 + slot_idx * 20;

        // Register bus channels
        for i in 0..10 {
            bus.register(ParamId(base + i), 0.0);
        }

        let arc_bus = Arc::new(bus);
        view.live_param_bus = Some(Arc::clone(&arc_bus));
        view.selected_track_idx = 0;
        view.device_rack_state.selected_chain_idx = 0;

        // Dispatch MPEG-H Trajectory
        view.mpegh_trajectory_view.azimuth_deg = 65.0;
        view.mpegh_trajectory_view.elevation_deg = 25.0;
        view.mpegh_trajectory_view.distance_m = 5.2;
        view.mpegh_trajectory_view.trajectory_speed_hz = 0.40;
        view.mpegh_trajectory_view.object_spread = 0.55;
        view.mpegh_trajectory_view.vbap_energy_focus = 0.75;
        view.sync_mpegh_trajectory_hud_state();

        assert!((arc_bus.get(ParamId(base)).unwrap() - 65.0).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 1)).unwrap() - 25.0).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 2)).unwrap() - 5.2).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 3)).unwrap() - 0.40).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 4)).unwrap() - 0.55).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 5)).unwrap() - 0.75).abs() < 1e-3);

        // Dispatch NHK 22.2 Immersion
        view.nhk222_immersion_view.azimuth_deg = -30.0;
        view.nhk222_immersion_view.elevation_deg = 45.0;
        view.nhk222_immersion_view.distance_m = 12.0;
        view.nhk222_immersion_view.top_layer_energy = 0.88;
        view.nhk222_immersion_view.middle_layer_energy = 0.65;
        view.nhk222_immersion_view.bottom_layer_energy = 0.25;
        view.nhk222_immersion_view.lfe_energy = 0.50;
        view.sync_nhk222_immersion_hud_state();

        assert!((arc_bus.get(ParamId(base)).unwrap() - (-30.0)).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 1)).unwrap() - 45.0).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 2)).unwrap() - 12.0).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 3)).unwrap() - 0.88).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 4)).unwrap() - 0.65).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 5)).unwrap() - 0.25).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 6)).unwrap() - 0.50).abs() < 1e-3);
    }

    #[test]
    fn test_turn82_gui_modern_inspector_mpegh_trajectory_and_nhk222_immersion_trigger() {
        let mut state = ModernInspectorState::default();
        let ctx = egui::Context::default();

        state.selected_node_kind = Some("MpeghTrajectorySpatializer".to_string());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_inspector_with_context(&mut state, ui, None, 1, 0);
            });
        });
        assert!(!state.requested_open_mpegh_trajectory_hud);

        state.selected_node_kind = Some("Nhk222ImmersionView".to_string());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_inspector_with_context(&mut state, ui, None, 1, 0);
            });
        });
        assert!(!state.requested_open_nhk222_immersion_hud);
    }

    #[test]
    fn test_turn82_gui_modern_device_rack_mpegh_trajectory_and_nhk222_immersion_trigger() {
        let mut state = ModernDeviceRackState::default();
        let ctx = egui::Context::default();

        state.selected_node_kind = Some("MpeghTrajectoryNode".to_string());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_device_rack_with_context(&mut state, ui, None, 1);
            });
        });
        assert!(!state.requested_open_mpegh_trajectory_hud);

        state.selected_node_kind = Some("Nhk222ImmersionNode".to_string());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_device_rack_with_context(&mut state, ui, None, 1);
            });
        });
        assert!(!state.requested_open_nhk222_immersion_hud);
    }

    #[test]
    fn test_turn82_gui_command_palette_registration() {
        let palette = CommandPalette::new();
        let actions = palette.available_actions();

        let has_trj = actions
            .iter()
            .any(|a| a.action_id == "open_mpegh_trajectory_hud" && a.category == "Spatial");
        let has_nhk = actions
            .iter()
            .any(|a| a.action_id == "open_nhk222_immersion_hud" && a.category == "Spatial");

        assert!(has_trj, "MPEG-H Trajectory HUD command action must be registered in CommandPalette");
        assert!(has_nhk, "NHK 22.2 Immersion HUD command action must be registered in CommandPalette");
    }

    #[test]
    fn test_turn82_gui_award_winning_view_request_trigger_propagation() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        view.inspector_state.requested_open_mpegh_trajectory_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_mpegh_trajectory_hud_open());
        assert!(!view.inspector_state.requested_open_mpegh_trajectory_hud);

        view.close_mpegh_trajectory_hud();
        view.device_rack_state.requested_open_nhk222_immersion_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_nhk222_immersion_hud_open());
        assert!(!view.device_rack_state.requested_open_nhk222_immersion_hud);
    }
}
