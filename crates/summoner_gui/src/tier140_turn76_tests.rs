#![allow(clippy::all)]

//! Test suite for Turn #76 (Turn #39 deliverable) — Broadcast Mastering Dolby Atmos 9.1.6 3D Room Raytracing & Auro-3D 13.1 Tri-Level Spatializer HUDs Convergence.

#[cfg(test)]
pub mod pure_tests {
    use crate::layout_math::Rect;
    use crate::views::atmos_916_spatializer_view::{
        Atmos916SpatializerView, AtmosRoomType, ATMOS_PUCK_HIT_RADIUS,
    };
    use crate::views::auro3d_spatializer_view::{
        Auro3dSpatializerView, AuroFormat, AURO3D_PUCK_HIT_RADIUS,
    };
    use summoner_core::param_bus::ParamId;

    #[test]
    fn test_turn76_pure_atmos_916_spatializer_view_initialization_and_profiles() {
        let mut view = Atmos916SpatializerView::new();
        assert_eq!(view.room_type, AtmosRoomType::MasteringStage916);
        assert!((view.source_x_m - 1.5).abs() < 1e-3);
        assert!((view.source_y_m - 2.0).abs() < 1e-3);
        assert!((view.source_z_m - 1.2).abs() < 1e-3);
        assert_eq!(view.ray_count, 128);
        assert!((view.rt60_decay_s - 0.35).abs() < 1e-3);
        assert_eq!(ATMOS_PUCK_HIT_RADIUS, 22.0);

        // Room profile testing
        view.set_room_type(AtmosRoomType::CinemaAuditoriumDolby);
        assert_eq!(view.room_type, AtmosRoomType::CinemaAuditoriumDolby);
        assert_eq!(view.room_type.room_name(), "CINEMA THEATRICAL DUB");
        assert_eq!(view.room_type.nominal_channels(), 32);
        assert_eq!(view.room_type.nominal_ray_count(), 256);
        assert!((view.rt60_decay_s - 0.85).abs() < 1e-3);

        view.set_room_type(AtmosRoomType::NearfieldMixingStudio);
        assert_eq!(view.room_type, AtmosRoomType::NearfieldMixingStudio);
        assert_eq!(view.room_type.nominal_channels(), 16);
        assert_eq!(view.ray_count, 64);
        assert!((view.rt60_decay_s - 0.22).abs() < 1e-3);

        view.set_room_type(AtmosRoomType::BinauralAtmosSpatializer);
        assert_eq!(view.room_type, AtmosRoomType::BinauralAtmosSpatializer);
        assert_eq!(view.room_type.nominal_channels(), 2);
        assert_eq!(view.ray_count, 128);

        view.set_room_type(AtmosRoomType::CarAudio16ChannelArray);
        assert_eq!(view.room_type, AtmosRoomType::CarAudio16ChannelArray);
        assert_eq!(view.room_type.nominal_channels(), 16);
        assert_eq!(view.ray_count, 96);
        assert!((view.rt60_decay_s - 0.15).abs() < 1e-3);

        // Coordinate normalization
        let norm_x = Atmos916SpatializerView::coord_to_normalized(0.0);
        assert!((norm_x - 0.5).abs() < 1e-3);
        let back_x = Atmos916SpatializerView::normalized_to_coord(norm_x);
        assert!((back_x - 0.0).abs() < 1e-3);

        let norm_max = Atmos916SpatializerView::coord_to_normalized(8.0);
        assert!((norm_max - 1.0).abs() < 1e-3);
        let norm_min = Atmos916SpatializerView::coord_to_normalized(-8.0);
        assert!((norm_min - 0.0).abs() < 1e-3);

        // Speaker energy simulation check (16 channels)
        assert_eq!(view.speaker_energy_levels.len(), 16);
        for &lvl in &view.speaker_energy_levels {
            assert!(lvl >= 0.0 && lvl <= 1.0);
        }

        // Hit testing
        let canvas = Rect::new(20.0, 56.0, 420.0, 224.0);
        let px = canvas.x + view.puck_pos.0 * canvas.width;
        let py = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;
        assert!(view.hit_test_atmos_puck((px, py), canvas));
        assert!(!view.hit_test_atmos_puck((px + 80.0, py + 80.0), canvas));

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains('+'));
        assert!(snap.contains('|'));
    }

    #[test]
    fn test_turn76_pure_auro3d_spatializer_view_initialization_and_formats() {
        let mut view = Auro3dSpatializerView::new();
        assert_eq!(view.format, AuroFormat::Auro131);
        assert!((view.azimuth_deg - 35.0).abs() < 1e-3);
        assert!((view.elevation_deg - 28.0).abs() < 1e-3);
        assert!((view.distance_m - 2.8).abs() < 1e-3);
        assert!((view.height_layer_delay_ms - 12.5).abs() < 1e-3);
        assert_eq!(view.format.channel_count(), 14);
        assert!(view.format.has_top_ceiling_channel());
        assert_eq!(AURO3D_PUCK_HIT_RADIUS, 22.0);

        // Format tests
        view.set_format(AuroFormat::Auro91);
        assert_eq!(view.format, AuroFormat::Auro91);
        assert_eq!(view.format.channel_count(), 10);
        assert!(!view.format.has_top_ceiling_channel());

        view.set_format(AuroFormat::Auro101);
        assert_eq!(view.format, AuroFormat::Auro101);
        assert_eq!(view.format.channel_count(), 11);
        assert!(view.format.has_top_ceiling_channel());

        view.set_format(AuroFormat::Auro111);
        assert_eq!(view.format, AuroFormat::Auro111);
        assert_eq!(view.format.channel_count(), 12);
        assert!(!view.format.has_top_ceiling_channel());

        view.set_format(AuroFormat::AuroMaxBinaural);
        assert_eq!(view.format, AuroFormat::AuroMaxBinaural);
        assert_eq!(view.format.channel_count(), 2);

        // Coordinate normalization
        let norm_az = Auro3dSpatializerView::azimuth_to_normalized(0.0);
        assert!((norm_az - 0.5).abs() < 1e-3);
        let az_back = Auro3dSpatializerView::normalized_to_azimuth(norm_az);
        assert!((az_back - 0.0).abs() < 1e-3);

        let norm_el = Auro3dSpatializerView::elevation_to_normalized(30.0);
        assert!((norm_el - 0.5).abs() < 1e-3);
        let el_back = Auro3dSpatializerView::normalized_to_elevation(norm_el);
        assert!((el_back - 30.0).abs() < 1e-3);

        // Cartesian coordinates
        let (cx, cy, cz) = view.evaluate_cartesian_position();
        assert!(cx.is_finite() && cy.is_finite() && cz.is_finite());

        // Tri-level energy panning distribution
        assert!(view.bed_layer_energy >= 0.0 && view.bed_layer_energy <= 1.0);
        assert!(view.height_layer_energy >= 0.0 && view.height_layer_energy <= 1.0);
        assert!(view.top_layer_energy >= 0.0 && view.top_layer_energy <= 1.0);

        // Hit testing
        let canvas = Rect::new(20.0, 56.0, 420.0, 224.0);
        let puck_x = canvas.x + view.auro_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.auro_puck_pos.1) * canvas.height;
        assert!(view.hit_test_auro_puck((puck_x, puck_y), canvas));
        assert!(!view.hit_test_auro_puck((puck_x + 90.0, puck_y), canvas));

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains('+'));
        assert!(snap.contains('|'));
    }

    #[test]
    fn test_turn76_pure_slot_arithmetic_isolation() {
        let track_id = 6u32;
        let slot_id = 3u32;
        let base_pid = ParamId(track_id * 1000 + slot_id * 20);

        let pid_p0 = ParamId(base_pid.0);
        let pid_p1 = ParamId(base_pid.0 + 1);
        let pid_p2 = ParamId(base_pid.0 + 2);
        let pid_p3 = ParamId(base_pid.0 + 3);

        assert_eq!(pid_p0.0, 6060);
        assert_eq!(pid_p1.0, 6061);
        assert_eq!(pid_p2.0, 6062);
        assert_eq!(pid_p3.0, 6063);

        let next_slot_pid = ParamId(track_id * 1000 + (slot_id + 1) * 20);
        assert_eq!(next_slot_pid.0, 6080);
        assert!(pid_p3.0 < next_slot_pid.0);
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
    fn test_turn76_gui_award_winning_view_headless_rendering_without_panic() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // Open Dolby Atmos 9.1.6 modal
        view.open_atmos_916_spatializer_hud();
        assert!(view.is_atmos_916_spatializer_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_atmos_916_spatializer_hud();
        assert!(!view.is_atmos_916_spatializer_hud_open());

        // Open Auro-3D 13.1 modal
        view.open_auro3d_spatializer_hud();
        assert!(view.is_auro3d_spatializer_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_auro3d_spatializer_hud();
        assert!(!view.is_auro3d_spatializer_hud_open());
    }

    #[test]
    fn test_turn76_gui_live_parambus_slot_dispatch() {
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

        // 1. Sync Atmos 9.1.6 Spatializer HUD
        view.atmos_916_spatializer_view.source_x_m = 2.5;
        view.atmos_916_spatializer_view.source_y_m = 3.5;
        view.atmos_916_spatializer_view.source_z_m = 1.8;
        view.atmos_916_spatializer_view.rt60_decay_s = 0.42;
        view.sync_atmos_916_spatializer_hud_state();

        assert_eq!(arc_bus.get(ParamId(base)), Some(2.5));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(3.5));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(1.8));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(0.42));

        assert_eq!(
            view.device_rack_state.node_param_values.get("source_x_m"),
            Some(&2.5)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("source_x_m"),
            Some(&2.5)
        );

        // 2. Sync Auro-3D 13.1 Spatializer HUD
        view.auro3d_spatializer_view.azimuth_deg = 45.0;
        view.auro3d_spatializer_view.elevation_deg = 30.0;
        view.auro3d_spatializer_view.distance_m = 3.5;
        view.auro3d_spatializer_view.height_layer_delay_ms = 15.0;
        view.sync_auro3d_spatializer_hud_state();

        assert_eq!(arc_bus.get(ParamId(base)), Some(45.0));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(30.0));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(3.5));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(15.0));

        assert_eq!(
            view.device_rack_state.node_param_values.get("height_layer_delay_ms"),
            Some(&15.0)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("height_layer_delay_ms"),
            Some(&15.0)
        );
    }

    #[test]
    fn test_turn76_gui_rack_and_inspector_trigger_propagation() {
        let mut view = AwardWinningGuiView::new();

        view.device_rack_state.requested_open_atmos_916_hud = true;
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_atmos_916_spatializer_hud_open());
        assert!(!view.device_rack_state.requested_open_atmos_916_hud);

        view.close_atmos_916_spatializer_hud();
        assert!(!view.is_atmos_916_spatializer_hud_open());

        view.inspector_state.requested_open_auro3d_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_auro3d_spatializer_hud_open());
        assert!(!view.inspector_state.requested_open_auro3d_hud);
    }

    #[test]
    fn test_turn76_gui_command_palette_action_registration() {
        let palette = CommandPalette::new();
        let atmos_action = palette
            .actions
            .iter()
            .find(|a| a.action_id == "open_atmos_916_spatializer_hud");
        assert!(atmos_action.is_some());
        let atmos = atmos_action.unwrap();
        assert_eq!(atmos.category, "Spatial");
        assert!(atmos.label.contains("Dolby Atmos 9.1.6"));

        let auro_action = palette
            .actions
            .iter()
            .find(|a| a.action_id == "open_auro3d_spatializer_hud");
        assert!(auro_action.is_some());
        let auro = auro_action.unwrap();
        assert_eq!(auro.category, "Spatial");
        assert!(auro.label.contains("Auro-3D 13.1"));
    }
}
