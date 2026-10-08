#![allow(clippy::all)]

//! Test suite for Turn #77 (Turn #40 deliverable - Sprint Review) — Broadcast Mastering NHK 22.2 Super Hi-Vision Hemispherical Spatializer & Wave Field Synthesis (WFS) Holographic Linear Array HUDs Convergence.

#[cfg(test)]
pub mod pure_tests {
    use crate::layout_math::Rect;
    use crate::views::nhk222_spatializer_view::{
        Nhk222SpatializerView, NhkFormat, NHK222_PUCK_HIT_RADIUS,
    };
    use crate::views::wfs_array_spatializer_view::{
        WfsArraySpatializerView, WfsGeometry, WFS_PUCK_HIT_RADIUS,
    };
    use summoner_core::param_bus::ParamId;

    #[test]
    fn test_turn77_pure_nhk222_spatializer_view_initialization_and_formats() {
        let mut view = Nhk222SpatializerView::new();
        assert_eq!(view.format, NhkFormat::NHK222FullDome);
        assert!((view.azimuth_deg - 45.0).abs() < 1e-3);
        assert!((view.elevation_deg - 25.0).abs() < 1e-3);
        assert!((view.distance_m - 3.5).abs() < 1e-3);
        assert!((view.divergence_spread_pct - 35.0).abs() < 1e-3);
        assert_eq!(view.format.channel_count(), 24);
        assert!(view.format.is_hemispherical());
        assert_eq!(NHK222_PUCK_HIT_RADIUS, 22.0);

        // Format profiles
        view.set_format(NhkFormat::NHK222Downmix91);
        assert_eq!(view.format, NhkFormat::NHK222Downmix91);
        assert_eq!(view.format.channel_count(), 10);
        assert!(!view.format.is_hemispherical());

        view.set_format(NhkFormat::NHK222Downmix51);
        assert_eq!(view.format, NhkFormat::NHK222Downmix51);
        assert_eq!(view.format.channel_count(), 6);
        assert!(!view.format.is_hemispherical());

        view.set_format(NhkFormat::NHK222BinauralDome);
        assert_eq!(view.format, NhkFormat::NHK222BinauralDome);
        assert_eq!(view.format.channel_count(), 2);
        assert!(view.format.is_hemispherical());

        view.set_format(NhkFormat::NHK222ObjectMaster);
        assert_eq!(view.format, NhkFormat::NHK222ObjectMaster);
        assert_eq!(view.format.channel_count(), 24);
        assert!(view.format.is_hemispherical());

        // Coordinate normalization
        let norm_az = Nhk222SpatializerView::azimuth_to_normalized(0.0);
        assert!((norm_az - 0.5).abs() < 1e-3);
        let back_az = Nhk222SpatializerView::normalized_to_azimuth(norm_az);
        assert!((back_az - 0.0).abs() < 1e-3);

        let norm_el = Nhk222SpatializerView::elevation_to_normalized(0.0);
        assert!((norm_el - 0.25).abs() < 1e-3);
        let back_el = Nhk222SpatializerView::normalized_to_elevation(norm_el);
        assert!((back_el - 0.0).abs() < 1e-3);

        // 3D Cartesian coordinates
        let (x, y, z) = view.evaluate_cartesian_position();
        let computed_dist = (x * x + y * y + z * z).sqrt();
        assert!((computed_dist - view.distance_m).abs() < 1e-2);

        // Hit testing
        let canvas = Rect::new(20.0, 56.0, 420.0, 224.0);
        let px = canvas.x + view.nhk_puck_pos.0 * canvas.width;
        let py = canvas.y + (1.0 - view.nhk_puck_pos.1) * canvas.height;
        assert!(view.hit_test_nhk_puck((px, py), canvas));
        assert!(!view.hit_test_nhk_puck((px + 80.0, py + 80.0), canvas));

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains('+'));
        assert!(snap.contains('|'));
    }

    #[test]
    fn test_turn77_pure_wfs_array_spatializer_view_initialization_and_geometries() {
        let mut view = WfsArraySpatializerView::new();
        assert_eq!(view.array_geometry, WfsGeometry::LinearFrontArray64);
        assert!((view.source_x_m - 0.0).abs() < 1e-3);
        assert!((view.source_y_m - 3.5).abs() < 1e-3);
        assert_eq!(view.channel_count, 64);
        assert!((view.spatial_aliasing_cutoff_hz - 2200.0).abs() < 1e-3);
        assert!(!view.is_focused_source);
        assert_eq!(WFS_PUCK_HIT_RADIUS, 22.0);

        // Geometry profiles
        view.set_array_geometry(WfsGeometry::RectangularRoomArray128);
        assert_eq!(view.array_geometry, WfsGeometry::RectangularRoomArray128);
        assert_eq!(view.channel_count, 128);
        assert_eq!(view.array_geometry.geometry_name(), "RECTANGULAR (128-CH)");
        assert!((view.spatial_aliasing_cutoff_hz - 1800.0).abs() < 1e-3);

        view.set_array_geometry(WfsGeometry::CurvedStageProscenium32);
        assert_eq!(view.array_geometry, WfsGeometry::CurvedStageProscenium32);
        assert_eq!(view.channel_count, 32);
        assert!((view.spatial_aliasing_cutoff_hz - 1400.0).abs() < 1e-3);

        view.set_array_geometry(WfsGeometry::DualLinearMasteringDesk48);
        assert_eq!(view.array_geometry, WfsGeometry::DualLinearMasteringDesk48);
        assert_eq!(view.channel_count, 48);
        assert!((view.spatial_aliasing_cutoff_hz - 4500.0).abs() < 1e-3);

        view.set_array_geometry(WfsGeometry::HexagonalHolographicArray96);
        assert_eq!(view.array_geometry, WfsGeometry::HexagonalHolographicArray96);
        assert_eq!(view.channel_count, 96);
        assert!((view.spatial_aliasing_cutoff_hz - 2600.0).abs() < 1e-3);

        // Coordinate normalization
        let norm_x = WfsArraySpatializerView::x_to_normalized(0.0);
        assert!((norm_x - 0.5).abs() < 1e-3);
        let back_x = WfsArraySpatializerView::normalized_to_x(norm_x);
        assert!((back_x - 0.0).abs() < 1e-3);

        let norm_y = WfsArraySpatializerView::y_to_normalized(0.0);
        assert!((norm_y - (5.0 / 15.0)).abs() < 1e-3);
        let back_y = WfsArraySpatializerView::normalized_to_y(norm_y);
        assert!((back_y - 0.0).abs() < 1e-3);

        // Delay profile simulation check (16 delays)
        assert_eq!(view.array_delays_ms.len(), 16);
        for &del in &view.array_delays_ms {
            assert!(del >= 0.0);
        }

        // Hit testing
        let canvas = Rect::new(20.0, 56.0, 420.0, 224.0);
        let px = canvas.x + view.puck_pos.0 * canvas.width;
        let py = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;
        assert!(view.hit_test_wfs_puck((px, py), canvas));
        assert!(!view.hit_test_wfs_puck((px + 80.0, py + 80.0), canvas));

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains('+'));
        assert!(snap.contains('|'));
    }

    #[test]
    fn test_turn77_pure_slot_arithmetic_channel_isolation() {
        for track_id in [1u32, 2u32, 9u32] {
            for slot_idx in [0u32, 1u32, 3u32] {
                let base = track_id * 1000 + slot_idx * 20;

                let nhk_az = ParamId(base);
                let nhk_el = ParamId(base + 1);
                let nhk_dist = ParamId(base + 2);
                let nhk_spread = ParamId(base + 3);

                assert_ne!(nhk_az, nhk_el);
                assert_ne!(nhk_el, nhk_dist);
                assert_ne!(nhk_dist, nhk_spread);

                let next_slot_base = track_id * 1000 + (slot_idx + 1) * 20;
                assert!(nhk_spread.0 < next_slot_base);
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
    fn test_turn77_gui_award_winning_view_headless_rendering_without_panic() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // Open NHK 22.2 Super Hi-Vision modal
        view.open_nhk222_spatializer_hud();
        assert!(view.is_nhk222_spatializer_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_nhk222_spatializer_hud();
        assert!(!view.is_nhk222_spatializer_hud_open());

        // Open WFS Holographic Linear Array modal
        view.open_wfs_array_spatializer_hud();
        assert!(view.is_wfs_array_spatializer_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_wfs_array_spatializer_hud();
        assert!(!view.is_wfs_array_spatializer_hud_open());
    }

    #[test]
    fn test_turn77_gui_live_parambus_slot_dispatch() {
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

        // 1. Sync NHK 22.2 Spatializer HUD
        view.nhk222_spatializer_view.azimuth_deg = 60.0;
        view.nhk222_spatializer_view.elevation_deg = 35.0;
        view.nhk222_spatializer_view.distance_m = 4.2;
        view.nhk222_spatializer_view.divergence_spread_pct = 40.0;
        view.sync_nhk222_spatializer_hud_state();

        assert_eq!(arc_bus.get(ParamId(base)), Some(60.0));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(35.0));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(4.2));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(40.0));

        assert_eq!(
            view.device_rack_state.node_param_values.get("azimuth_deg"),
            Some(&60.0)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("azimuth_deg"),
            Some(&60.0)
        );

        // 2. Sync WFS Array Spatializer HUD
        view.wfs_array_spatializer_view.source_x_m = -2.5;
        view.wfs_array_spatializer_view.source_y_m = 4.8;
        view.wfs_array_spatializer_view.channel_count = 64;
        view.wfs_array_spatializer_view.spatial_aliasing_cutoff_hz = 2200.0;
        view.sync_wfs_array_spatializer_hud_state();

        assert_eq!(arc_bus.get(ParamId(base)), Some(-2.5));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(4.8));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(64.0));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(2200.0));

        assert_eq!(
            view.device_rack_state.node_param_values.get("source_x_m"),
            Some(&-2.5)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("source_x_m"),
            Some(&-2.5)
        );
    }

    #[test]
    fn test_turn77_gui_rack_and_inspector_trigger_propagation() {
        let mut view = AwardWinningGuiView::new();

        view.device_rack_state.requested_open_nhk222_hud = true;
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_nhk222_spatializer_hud_open());
        assert!(!view.device_rack_state.requested_open_nhk222_hud);

        view.close_nhk222_spatializer_hud();
        assert!(!view.is_nhk222_spatializer_hud_open());

        view.inspector_state.requested_open_wfs_array_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_wfs_array_spatializer_hud_open());
        assert!(!view.inspector_state.requested_open_wfs_array_hud);
    }

    #[test]
    fn test_turn77_gui_command_palette_action_registration() {
        let palette = CommandPalette::new();
        let nhk_action = palette
            .actions
            .iter()
            .find(|a| a.action_id == "open_nhk222_spatializer_hud");
        assert!(nhk_action.is_some());
        let nhk = nhk_action.unwrap();
        assert_eq!(nhk.category, "Spatial");
        assert!(nhk.label.contains("NHK 22.2"));

        let wfs_action = palette
            .actions
            .iter()
            .find(|a| a.action_id == "open_wfs_array_spatializer_hud");
        assert!(wfs_action.is_some());
        let wfs = wfs_action.unwrap();
        assert_eq!(wfs.category, "Spatial");
        assert!(wfs.label.contains("Wave Field Synthesis"));
    }
}
