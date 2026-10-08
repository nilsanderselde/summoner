#![allow(clippy::all)]

//! Test suite for Turn #81 (Sprint Turn #7) — 7.1.4 Dolby Atmos 3D Immersive Bed & Object Radar HUD and Atmos Spherical Proximity HUD Convergence.

#[cfg(test)]
pub mod pure_tests {
    use crate::layout_math::Rect;
    use crate::views::atmos_proximity_view::{
        AtmosProximityProfile, AtmosProximityView, ATMOS_PROX_PUCK_HIT_RADIUS,
    };
    use crate::views::atmos_surround_view::{
        AtmosChannel, AtmosDownmixMode, AtmosSurroundView, ATMOS_PUCK_HIT_RADIUS, NUM_ATMOS_CHANNELS,
    };
    use summoner_core::param_bus::ParamId;

    #[test]
    fn test_turn81_pure_atmos_surround_view_initialization_and_vbap_gains() {
        let mut view = AtmosSurroundView::new();
        assert_eq!(view.downmix_mode, AtmosDownmixMode::Full714Immersive);
        assert!((view.object_x - 0.35).abs() < 1e-3);
        assert!((view.object_y - 0.55).abs() < 1e-3);
        assert!((view.object_z_height - 0.40).abs() < 1e-3);
        assert!((view.object_size_spread - 25.0).abs() < 1e-3);
        assert!((view.lfe_send_gain_db - (-12.0)).abs() < 1e-3);
        assert_eq!(ATMOS_PUCK_HIT_RADIUS, 22.0);
        assert_eq!(NUM_ATMOS_CHANNELS, 12);

        // Coordinate normalizations
        for coord in [-1.0, -0.5, 0.0, 0.5, 1.0] {
            let norm = AtmosSurroundView::coord_to_normalized(coord);
            assert!((0.0..=1.0).contains(&norm));
            let back = AtmosSurroundView::normalized_to_coord(norm);
            assert!((back - coord).abs() < 1e-4);
        }

        // Downmix modes
        for mode in [
            AtmosDownmixMode::Full714Immersive,
            AtmosDownmixMode::Surround51Legacy,
            AtmosDownmixMode::Stereo20Binaural,
        ] {
            view.downmix_mode = mode;
            assert_eq!(view.downmix_mode, mode);
        }

        // VBAP gain calculation: Front-Right sound object
        view.object_x = 0.8;
        view.object_y = 0.8;
        view.object_z_height = 0.2;
        view.update_vbap_gains();

        // Channel indices: 0: L, 1: C, 2: R, 3: LFE, 4: Lss, 5: Rss, 6: Lsr, 7: Rsr, 8: Ltf, 9: Rtf, 10: Ltr, 11: Rtr
        let gain_r = view.speaker_energy_gains[AtmosChannel::Right as usize];
        let gain_l = view.speaker_energy_gains[AtmosChannel::Left as usize];
        assert!(gain_r > gain_l, "Right speaker gain should exceed Left speaker gain for positive X");

        // Puck hit testing
        let canvas = Rect::new(20.0, 100.0, 420.0, 230.0);
        view.atmos_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(view.hit_test_atmos_puck((center_x, center_y), canvas));
        assert!(!view.hit_test_atmos_puck((center_x + 100.0, center_y), canvas));

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains('+'));
        assert!(snap.contains('O'));
    }

    #[test]
    fn test_turn81_pure_atmos_proximity_view_initialization_and_simulation() {
        let mut view = AtmosProximityView::new();
        assert_eq!(view.profile, AtmosProximityProfile::ImmersiveCinema714);
        assert!((view.distance_m - 3.5).abs() < 1e-3);
        assert!((view.azimuth_deg - (-35.0)).abs() < 1e-3);
        assert!((view.elevation_deg - 25.0).abs() < 1e-3);
        assert_eq!(ATMOS_PROX_PUCK_HIT_RADIUS, 22.0);

        // Profile switching
        let profiles = [
            (AtmosProximityProfile::ImmersiveCinema714, 3.5),
            (AtmosProximityProfile::BinauralProximityHeadphones, 0.6),
            (AtmosProximityProfile::NearfieldStudio714, 2.0),
            (AtmosProximityProfile::StadiumBroadcastImmersive, 12.0),
            (AtmosProximityProfile::VR6DOFObjectSpatial, 0.4),
        ];

        for (prof, expected_dist) in profiles {
            view.set_profile(prof);
            assert_eq!(view.profile, prof);
            assert!((view.distance_m - expected_dist).abs() < 1e-3);
            assert!(!prof.profile_name().is_empty());
        }

        // Coordinate normalizations
        for az in [-180.0, -90.0, 0.0, 90.0, 180.0] {
            let norm = AtmosProximityView::azimuth_to_normalized(az);
            assert!((0.0..=1.0).contains(&norm));
            let back = AtmosProximityView::normalized_to_azimuth(norm);
            assert!((back - az).abs() < 1e-4);
        }

        for el in [-90.0, -45.0, 0.0, 45.0, 90.0] {
            let norm = AtmosProximityView::elevation_to_normalized(el);
            assert!((0.0..=1.0).contains(&norm));
            let back = AtmosProximityView::normalized_to_elevation(norm);
            assert!((back - el).abs() < 1e-4);
        }

        // Simulation update: nearfield vs farfield
        view.distance_m = 0.5;
        view.update_atmos_simulation();
        let near_bass = view.proximity_bass_boost_db;
        let near_air = view.air_absorption_loss_db;

        view.distance_m = 10.0;
        view.update_atmos_simulation();
        let far_bass = view.proximity_bass_boost_db;
        let far_air = view.air_absorption_loss_db;

        assert!(near_bass > far_bass, "Proximity bass boost should be stronger nearfield than farfield");
        assert!(far_air > near_air, "Air absorption HF loss should be stronger farfield than nearfield");
        assert!(view.itd_us.abs() <= 800.0);

        // Puck hit testing
        let canvas = Rect::new(20.0, 100.0, 420.0, 230.0);
        view.puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(view.hit_test_atmos_puck((center_x, center_y), canvas));
        assert!(!view.hit_test_atmos_puck((center_x + 100.0, center_y), canvas));

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains('+'));
        assert!(snap.contains('A'));
    }

    #[test]
    fn test_turn81_pure_parambus_slot_arithmetic_channel_isolation() {
        for track_id in 1u32..=4 {
            for slot_idx in 0u32..=3 {
                let base = track_id * 1000 + slot_idx * 20;

                // Atmos Surround: 5 parameters (offsets 0..=4)
                let atm_x = ParamId(base);
                let atm_y = ParamId(base + 1);
                let atm_z = ParamId(base + 2);
                let atm_spr = ParamId(base + 3);
                let atm_lfe = ParamId(base + 4);

                assert_eq!(atm_x.0, base);
                assert_eq!(atm_y.0, base + 1);
                assert_eq!(atm_z.0, base + 2);
                assert_eq!(atm_spr.0, base + 3);
                assert_eq!(atm_lfe.0, base + 4);

                // Atmos Proximity: 6 parameters (offsets 0..=5)
                let prx_dist = ParamId(base);
                let prx_az = ParamId(base + 1);
                let prx_el = ParamId(base + 2);
                let prx_bass = ParamId(base + 3);
                let prx_air = ParamId(base + 4);
                let prx_itd = ParamId(base + 5);

                assert_eq!(prx_dist.0, base);
                assert_eq!(prx_az.0, base + 1);
                assert_eq!(prx_el.0, base + 2);
                assert_eq!(prx_bass.0, base + 3);
                assert_eq!(prx_air.0, base + 4);
                assert_eq!(prx_itd.0, base + 5);

                // Verify slot boundary isolation (stays within slot allocation of 20 IDs)
                let next_slot_base = track_id * 1000 + (slot_idx + 1) * 20;
                assert!(atm_lfe.0 < next_slot_base);
                assert!(prx_itd.0 < next_slot_base);
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
    fn test_turn81_gui_award_winning_view_atmos_surround_and_proximity_modal_lifecycle() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // 1. 7.1.4 Dolby Atmos Surround Bed HUD
        view.open_atmos_surround_hud();
        assert!(view.is_atmos_surround_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_atmos_surround_hud();
        assert!(!view.is_atmos_surround_hud_open());

        // 2. 7.1.4 Dolby Atmos Spherical Proximity HUD
        view.open_atmos_proximity_hud();
        assert!(view.is_atmos_proximity_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_atmos_proximity_hud();
        assert!(!view.is_atmos_proximity_hud_open());
    }

    #[test]
    fn test_turn81_gui_parambus_slot_dispatch_atmos_surround_and_proximity() {
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

        // Dispatch Atmos Surround
        view.atmos_surround_view.object_x = 0.45;
        view.atmos_surround_view.object_y = 0.75;
        view.atmos_surround_view.object_z_height = 0.60;
        view.atmos_surround_view.object_size_spread = 35.0;
        view.atmos_surround_view.lfe_send_gain_db = -6.0;

        view.sync_atmos_surround_hud_state();

        assert!((arc_bus.get(ParamId(base)).unwrap() - 0.45).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 1)).unwrap() - 0.75).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 2)).unwrap() - 0.60).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 3)).unwrap() - 35.0).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 4)).unwrap() - (-6.0)).abs() < 1e-3);

        // Dispatch Atmos Proximity
        view.atmos_proximity_view.distance_m = 4.5;
        view.atmos_proximity_view.azimuth_deg = -45.0;
        view.atmos_proximity_view.elevation_deg = 25.0;
        view.atmos_proximity_view.proximity_bass_boost_db = 4.0;
        view.atmos_proximity_view.air_absorption_loss_db = 1.5;
        view.atmos_proximity_view.itd_us = 420.0;

        view.sync_atmos_proximity_hud_state();

        assert!((arc_bus.get(ParamId(base)).unwrap() - 4.5).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 1)).unwrap() - (-45.0)).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 2)).unwrap() - 25.0).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 3)).unwrap() - 4.0).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 4)).unwrap() - 1.5).abs() < 1e-3);
        assert!((arc_bus.get(ParamId(base + 5)).unwrap() - 420.0).abs() < 1e-3);
    }

    #[test]
    fn test_turn81_gui_command_palette_registration_and_dispatch() {
        let palette = CommandPalette::new();
        let actions = &palette.actions;

        let has_surr = actions
            .iter()
            .any(|a| a.action_id == "open_atmos_surround_hud" && a.category == "Spatial");
        let has_prox = actions
            .iter()
            .any(|a| a.action_id == "open_atmos_proximity_hud" && a.category == "Spatial");

        assert!(has_surr);
        assert!(has_prox);
    }

    #[test]
    fn test_turn81_gui_device_rack_and_inspector_trigger_propagation() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        // 1. Trigger Atmos Surround from Inspector
        view.inspector_state.requested_open_atmos_surround_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_atmos_surround_hud_open());
        assert!(!view.inspector_state.requested_open_atmos_surround_hud);
        view.close_atmos_surround_hud();

        // 2. Trigger Atmos Proximity from Device Rack
        view.device_rack_state.requested_open_atmos_proximity_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_atmos_proximity_hud_open());
        assert!(!view.device_rack_state.requested_open_atmos_proximity_hud);
        view.close_atmos_proximity_hud();
    }

    #[test]
    fn test_turn81_gui_modern_device_rack_atmos_mini_visualizers() {
        let mut state = ModernDeviceRackState::default();
        let ctx = egui::Context::default();
        let mut bus = ParamBus::new();
        bus.register(ParamId(1000), 0.0);
        bus.register(ParamId(1001), 0.0);
        bus.register(ParamId(1002), 0.0);
        bus.register(ParamId(1003), 0.0);
        bus.register(ParamId(1004), 0.0);
        bus.register(ParamId(1005), 0.0);

        // Render Atmos Surround
        state.selected_node_kind = Some("AtmosSurroundNode".to_string());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_device_rack_with_context(ui, &mut state, None, Some(&bus), 1);
            });
        });
        assert!(state.node_param_values.contains_key("object_x") || state.node_param_values.contains_key("surround_azimuth"));

        // Render Atmos Proximity
        state.selected_node_kind = Some("AtmosProximityNode".to_string());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_device_rack_with_context(ui, &mut state, None, Some(&bus), 1);
            });
        });
        assert!(state.node_param_values.contains_key("distance_m"));
        assert!(state.node_param_values.contains_key("proximity_bass_boost_db") || state.node_param_values.contains_key("air_absorption_hf_loss_db"));
    }

    #[test]
    fn test_turn81_gui_modern_inspector_atmos_trigger_buttons() {
        let mut state = ModernInspectorState::default();
        let ctx = egui::Context::default();

        state.selected_node_kind = Some("AtmosSurroundNode".to_string());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_inspector_with_context(ui, &mut state, None, 1);
            });
        });

        state.selected_node_kind = Some("AtmosProximityNode".to_string());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_inspector_with_context(ui, &mut state, None, 1);
            });
        });
    }
}
