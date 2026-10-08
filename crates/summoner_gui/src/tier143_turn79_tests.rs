#![allow(clippy::all)]

//! Test suite for Turn #79 (Turn #42 deliverable / Sprint Turn #5) — Continuous 3D Doppler Kinematic Spatializer & 3D Shoebox Room Acoustics HUDs Convergence.

#[cfg(test)]
pub mod pure_tests {
    use crate::layout_math::Rect;
    use crate::views::continuous_doppler_view::{
        ContinuousDopplerView, HermiteInterpolationOrder, DOPPLER_PUCK_HIT_RADIUS,
        MAX_POS_RANGE_M, MIN_POS_RANGE_M,
    };
    use crate::views::spatial_room_view::{
        SpatialRoomView, WallMaterialPreset, SPATIAL_ROOM_PUCK_HIT_RADIUS,
    };
    use summoner_core::param_bus::ParamId;

    #[test]
    fn test_turn79_pure_continuous_doppler_view_initialization_and_kinematics() {
        let mut view = ContinuousDopplerView::new();
        assert_eq!(view.interpolation_order, HermiteInterpolationOrder::CubicHermite4Pt);
        assert!((view.pos_x - 0.0).abs() < 1e-3);
        assert!((view.pos_y - 3.5).abs() < 1e-3);
        assert!((view.pos_z - 1.0).abs() < 1e-3);
        assert!((view.speed_of_sound_mps - 343.0).abs() < 1e-3);
        assert!((view.air_absorption_coeff - 0.35).abs() < 1e-3);
        assert_eq!(DOPPLER_PUCK_HIT_RADIUS, 22.0);

        // Distance and angles
        let dist = view.distance();
        assert!(dist > 3.0);
        let az = view.azimuth_deg();
        assert!(az.abs() < 1e-3); // Facing front (+Y)
        let el = view.elevation_deg();
        assert!(el > 0.0); // Above listener (+Z)

        // Interpolation orders
        view.interpolation_order = HermiteInterpolationOrder::Linear2Pt;
        assert_eq!(view.interpolation_order, HermiteInterpolationOrder::Linear2Pt);
        view.interpolation_order = HermiteInterpolationOrder::WindowedSinc;
        assert_eq!(view.interpolation_order, HermiteInterpolationOrder::WindowedSinc);

        // Doppler kinematics formulas
        // Approaching source: source at +Y (0, 10, 0), moving towards origin (0, -20, 0)
        let (v_rad_app, ratio_app, cents_app) = ContinuousDopplerView::calculate_doppler_ratio(
            0.0, 10.0, 0.0, 0.0, -20.0, 0.0, 343.0,
        );
        assert!(v_rad_app > 0.0);
        assert!(ratio_app > 1.0);
        assert!(cents_app > 0.0);

        // Receding source: source at +Y (0, 10, 0), moving away from origin (0, +20, 0)
        let (v_rad_rec, ratio_rec, cents_rec) = ContinuousDopplerView::calculate_doppler_ratio(
            0.0, 10.0, 0.0, 0.0, 20.0, 0.0, 343.0,
        );
        assert!(v_rad_rec < 0.0);
        assert!(ratio_rec < 1.0);
        assert!(cents_rec < 0.0);

        // Stationary source
        let (v_rad_stat, ratio_stat, cents_stat) = ContinuousDopplerView::calculate_doppler_ratio(
            0.0, 10.0, 0.0, 0.0, 0.0, 0.0, 343.0,
        );
        assert!(v_rad_stat.abs() < 1e-3);
        assert!((ratio_stat - 1.0).abs() < 1e-3);
        assert!(cents_stat.abs() < 1e-3);

        // Coordinate normalizations
        let norm_x = ContinuousDopplerView::pos_to_normalized(0.0, MIN_POS_RANGE_M, MAX_POS_RANGE_M);
        assert!((norm_x - 0.5).abs() < 1e-3);
        let back_x = ContinuousDopplerView::normalized_to_pos(norm_x, MIN_POS_RANGE_M, MAX_POS_RANGE_M);
        assert!((back_x - 0.0).abs() < 1e-3);

        // Hit testing
        let canvas = Rect::new(0.0, 0.0, 400.0, 400.0);
        let cx = canvas.x + canvas.width * 0.5;
        let cy = canvas.y + canvas.height * 0.5;
        let max_r = canvas.width.min(canvas.height) * 0.44;
        let px = cx + (view.pos_x / MAX_POS_RANGE_M) * max_r;
        let py = cy - (view.pos_y / MAX_POS_RANGE_M) * max_r;

        assert!(view.hit_test_source_puck((px, py), canvas));
        assert!(!view.hit_test_source_puck((px + 80.0, py + 80.0), canvas));

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("DOPPLER 3D"));
        assert!(snap.contains("SoundSpeed"));
    }

    #[test]
    fn test_turn79_pure_spatial_room_view_initialization_and_acoustics() {
        let mut view = SpatialRoomView::new();
        assert_eq!(view.wall_material, WallMaterialPreset::StudioAcoustic);
        assert!((view.room_width_m - 8.0).abs() < 1e-3);
        assert!((view.room_depth_m - 12.0).abs() < 1e-3);
        assert!((view.room_height_m - 3.5).abs() < 1e-3);
        assert_eq!(SPATIAL_ROOM_PUCK_HIT_RADIUS, 22.0);

        // Volume and Surface Area
        let vol = view.room_volume();
        assert!((vol - (8.0 * 12.0 * 3.5)).abs() < 1e-2);
        let area = view.room_surface_area();
        assert!(area > 200.0);

        // Material Presets
        for mat in [
            WallMaterialPreset::StudioAcoustic,
            WallMaterialPreset::WoodPaneling,
            WallMaterialPreset::Concrete,
            WallMaterialPreset::Glass,
            WallMaterialPreset::HeavyCurtains,
            WallMaterialPreset::Plaster,
        ] {
            view.wall_material = mat;
            view.update_acoustics();
            assert!(view.active_absorption() > 0.0);
            assert!(view.rt60_seconds > 0.0);
            assert!(!mat.display_name().is_empty());
        }

        // Custom absorption
        view.wall_material = WallMaterialPreset::Custom;
        view.custom_absorption = 0.50;
        view.update_acoustics();
        assert!((view.active_absorption() - 0.50).abs() < 1e-3);

        // Sabine calculation
        let rt60 = SpatialRoomView::calculate_sabine_rt60(8.0, 12.0, 3.5, 0.35);
        assert!(rt60 > 0.2 && rt60 < 2.5);

        // Direct distance & delay
        let d_dist = view.direct_distance();
        let d_delay = view.direct_delay_ms();
        assert!(d_dist > 1.0);
        assert!(d_delay > 1.0);

        // 6 Early Reflections
        let rays = view.evaluate_early_reflections();
        assert_eq!(rays.len(), 6);
        for ray in &rays {
            assert!(ray.distance_m > 0.0);
            assert!(ray.delay_ms > 0.0);
            assert!(ray.attenuation_db <= 0.0);
            assert!(!ray.name.is_empty());
        }

        // Hit testing
        let dummy_rect = eframe::egui::Rect::from_min_size(
            eframe::egui::pos2(50.0, 50.0),
            eframe::egui::vec2(300.0, 300.0),
        );
        let s_canvas = view.room_to_canvas_pos(view.source_pos.0, view.source_pos.1, dummy_rect);
        let l_canvas = view.room_to_canvas_pos(view.listener_pos.0, view.listener_pos.1, dummy_rect);

        assert!(view.hit_test_source_puck((s_canvas.x, s_canvas.y), dummy_rect));
        assert!(view.hit_test_listener_puck((l_canvas.x, l_canvas.y), dummy_rect));
        assert!(!view.hit_test_source_puck((s_canvas.x + 80.0, s_canvas.y + 80.0), dummy_rect));

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("SHOEBOX ROOM"));
        assert!(snap.contains('+'));
        assert!(snap.contains('|'));
    }

    #[test]
    fn test_turn79_pure_slot_arithmetic_channel_isolation() {
        for track_id in [1u32, 2u32, 9u32] {
            for slot_idx in [0u32, 1u32, 4u32] {
                let base = track_id * 1000 + slot_idx * 20;

                // Continuous Doppler channels
                let dop_x = ParamId(base);
                let dop_y = ParamId(base + 1);
                let dop_z = ParamId(base + 2);
                let dop_c = ParamId(base + 3);
                let dop_air = ParamId(base + 4);

                assert_ne!(dop_x, dop_y);
                assert_ne!(dop_y, dop_z);
                assert_ne!(dop_z, dop_c);
                assert_ne!(dop_c, dop_air);

                // Spatial Room channels
                let rm_w = ParamId(base);
                let rm_d = ParamId(base + 1);
                let rm_h = ParamId(base + 2);
                let rm_sx = ParamId(base + 3);
                let rm_sy = ParamId(base + 4);
                let rm_sz = ParamId(base + 5);
                let rm_abs = ParamId(base + 6);

                assert_ne!(rm_w, rm_d);
                assert_ne!(rm_d, rm_h);
                assert_ne!(rm_h, rm_sx);
                assert_ne!(rm_sx, rm_sy);
                assert_ne!(rm_sy, rm_sz);
                assert_ne!(rm_sz, rm_abs);

                // Verify within slot boundary
                let next_slot_base = track_id * 1000 + (slot_idx + 1) * 20;
                assert!(rm_abs.0 < next_slot_base);
            }
        }
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
    fn test_turn79_gui_award_winning_view_continuous_doppler_and_spatial_room_modal_lifecycle() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // 1. Continuous 3D Doppler HUD
        view.open_continuous_doppler_hud();
        assert!(view.is_continuous_doppler_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_continuous_doppler_hud();
        assert!(!view.is_continuous_doppler_hud_open());

        // 2. 3D Shoebox Room HUD
        view.open_spatial_room_hud();
        assert!(view.is_spatial_room_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_spatial_room_hud();
        assert!(!view.is_spatial_room_hud_open());
    }

    #[test]
    fn test_turn79_gui_parambus_slot_dispatch_continuous_doppler_and_spatial_room() {
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

        // Dispatch Continuous Doppler
        view.continuous_doppler_view.pos_x = 14.5;
        view.continuous_doppler_view.pos_y = -8.2;
        view.continuous_doppler_view.pos_z = 2.4;
        view.continuous_doppler_view.speed_of_sound_mps = 345.0;
        view.continuous_doppler_view.air_absorption_coeff = 0.42;

        view.sync_continuous_doppler_hud_state();

        assert!((arc_bus.get(ParamId(base)).unwrap() - 14.5).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 1)).unwrap() - -8.2).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 2)).unwrap() - 2.4).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 3)).unwrap() - 345.0).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 4)).unwrap() - 0.42).abs() < 1e-3);

        // Dispatch Spatial Room
        view.spatial_room_view.room_width_m = 10.0;
        view.spatial_room_view.room_depth_m = 15.0;
        view.spatial_room_view.room_height_m = 4.2;
        view.spatial_room_view.source_pos = (2.5, 3.5, 1.2);
        view.spatial_room_view.wall_material = crate::views::spatial_room_view::WallMaterialPreset::WoodPaneling;

        view.sync_spatial_room_hud_state();

        assert!((arc_bus.get(ParamId(base)).unwrap() - 10.0).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 1)).unwrap() - 15.0).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 2)).unwrap() - 4.2).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 3)).unwrap() - 2.5).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 4)).unwrap() - 3.5).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 5)).unwrap() - 1.2).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 6)).unwrap() - 0.15).abs() < 1e-3);
    }

    #[test]
    fn test_turn79_gui_command_palette_registration_and_dispatch() {
        let palette = CommandPalette::new();
        let actions = &palette.actions;

        let has_dop = actions.iter().any(|a| a.action_id == "open_continuous_doppler_hud" && a.category == "Spatial");
        let has_room = actions.iter().any(|a| a.action_id == "open_spatial_room_hud" && a.category == "Spatial");

        assert!(has_dop);
        assert!(has_room);
    }

    #[test]
    fn test_turn79_gui_device_rack_and_inspector_trigger_propagation() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Trigger Continuous Doppler from Inspector
        view.inspector_state.requested_open_continuous_doppler_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_continuous_doppler_hud_open());
        assert!(!view.inspector_state.requested_open_continuous_doppler_hud);
        view.close_continuous_doppler_hud();

        // 2. Trigger Spatial Room from Device Rack
        view.device_rack_state.requested_open_spatial_room_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_spatial_room_hud_open());
        assert!(!view.device_rack_state.requested_open_spatial_room_hud);
        view.close_spatial_room_hud();
    }
}
