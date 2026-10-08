#![allow(clippy::all)]

//! Test suite for Turn #75 (Turn #38 deliverable) — Broadcast Mastering 5th-Order Ambisonics HOA5 & MPEG-H 3D Immersive Spatializer HUDs Convergence.

#[cfg(test)]
pub mod pure_tests {
    use crate::layout_math::Rect;
    use crate::views::hoa5_binaural_view::{
        Hoa5BinauralView, Hoa5Profile, HOA5_PUCK_HIT_RADIUS, HOA5_TOTAL_CHANNELS,
    };
    use crate::views::mpegh_3d_spatializer_view::{
        Mpegh3DSpatializerView, MpeghProfile, MPEGH_3D_PUCK_HIT_RADIUS,
    };
    use summoner_core::param_bus::ParamId;

    #[test]
    fn test_turn75_pure_hoa5_binaural_view_initialization_and_profiles() {
        let mut view = Hoa5BinauralView::new();
        assert_eq!(view.profile, Hoa5Profile::Hoa5thOrderSpherical36Ch);
        assert!((view.azimuth_deg - 45.0).abs() < 1e-3);
        assert!((view.elevation_deg - 20.0).abs() < 1e-3);
        assert!((view.distance_m - 2.5).abs() < 1e-3);
        assert_eq!(view.order, 5);
        assert_eq!(HOA5_PUCK_HIT_RADIUS, 22.0);
        assert_eq!(HOA5_TOTAL_CHANNELS, 36);

        // Profiles test
        view.set_profile(Hoa5Profile::MaxReSphericalEnergy);
        assert_eq!(view.profile, Hoa5Profile::MaxReSphericalEnergy);
        assert_eq!(view.profile.profile_name(), "MAX-rE ENERGY OPTIMIZED");
        assert!((view.energy_focus - 0.92).abs() < 1e-3);
        assert!((view.azimuth_deg - (-30.0)).abs() < 1e-3);

        view.set_profile(Hoa5Profile::InPhaseOptimalDecoder);
        assert_eq!(view.profile, Hoa5Profile::InPhaseOptimalDecoder);
        assert_eq!(view.profile.profile_name(), "IN-PHASE BINAURAL DECODER");
        assert!((view.energy_focus - 0.85).abs() < 1e-3);

        view.set_profile(Hoa5Profile::BinauralKEMAR5thOrder);
        assert_eq!(view.profile, Hoa5Profile::BinauralKEMAR5thOrder);
        assert_eq!(view.profile.profile_name(), "KEMAR 5TH-ORDER HRIR");
        assert!((view.distance_m - 1.5).abs() < 1e-3);

        view.set_profile(Hoa5Profile::DolbyAtmosBed36Virtual);
        assert_eq!(view.profile, Hoa5Profile::DolbyAtmosBed36Virtual);
        assert_eq!(view.profile.profile_name(), "HOA36 TO ATMOS 9.1.6 BED");
        assert!((view.distance_m - 3.2).abs() < 1e-3);

        // Coordinate normalization
        let norm_az = Hoa5BinauralView::azimuth_to_normalized(90.0);
        assert!((norm_az - 0.75).abs() < 1e-3);
        let az_back = Hoa5BinauralView::normalized_to_azimuth(norm_az);
        assert!((az_back - 90.0).abs() < 1e-3);

        let norm_el = Hoa5BinauralView::elevation_to_normalized(0.0);
        assert!((norm_el - 0.5).abs() < 1e-3);
        let el_back = Hoa5BinauralView::normalized_to_elevation(norm_el);
        assert!((el_back - 0.0).abs() < 1e-3);

        // Spherical harmonics simulation
        for &lvl in &view.spherical_harmonics_levels {
            assert!(lvl >= 0.0);
        }

        // Hit testing
        let canvas = Rect::new(20.0, 56.0, 420.0, 224.0);
        let px = canvas.x + view.puck_pos.0 * canvas.width;
        let py = canvas.y + (1.0 - view.puck_pos.1) * canvas.height;
        assert!(view.hit_test_hoa5_puck((px, py), canvas));
        assert!(!view.hit_test_hoa5_puck((px + 80.0, py + 80.0), canvas));

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains('+'));
        assert!(snap.contains('|'));
    }

    #[test]
    fn test_turn75_pure_mpegh_3d_spatializer_view_initialization_and_profiles() {
        let mut view = Mpegh3DSpatializerView::new();
        assert_eq!(view.profile, MpeghProfile::Level4_7_1_4);
        assert!((view.azimuth_deg - 35.0).abs() < 1e-3);
        assert!((view.elevation_deg - 20.0).abs() < 1e-3);
        assert!((view.distance_m - 2.8).abs() < 1e-3);
        assert!((view.spread_divergence_pct - 30.0).abs() < 1e-3);
        assert_eq!(MPEGH_3D_PUCK_HIT_RADIUS, 22.0);

        // Profile tests
        view.set_profile(MpeghProfile::Level5_22_2);
        assert_eq!(view.profile, MpeghProfile::Level5_22_2);
        assert_eq!(view.profile.profile_name(), "22.2 BROADCAST (L5)");
        assert_eq!(view.profile.speaker_count(), 24);
        assert_eq!(view.profile.metadata_bitrate_kbps(), 1024);

        view.set_profile(MpeghProfile::BinauralHeadTrack);
        assert_eq!(view.profile, MpeghProfile::BinauralHeadTrack);
        assert_eq!(view.profile.speaker_count(), 2);
        assert_eq!(view.profile.metadata_bitrate_kbps(), 384);

        view.set_profile(MpeghProfile::Dynamic3DObject);
        assert_eq!(view.profile, MpeghProfile::Dynamic3DObject);
        assert_eq!(view.profile.speaker_count(), 16);
        assert_eq!(view.profile.metadata_bitrate_kbps(), 640);

        view.set_profile(MpeghProfile::AdvancedDownmix);
        assert_eq!(view.profile, MpeghProfile::AdvancedDownmix);
        assert_eq!(view.profile.speaker_count(), 6);
        assert_eq!(view.profile.metadata_bitrate_kbps(), 256);

        // Coordinate normalization
        let norm_az = Mpegh3DSpatializerView::azimuth_to_normalized(0.0);
        assert!((norm_az - 0.5).abs() < 1e-3);
        let az_back = Mpegh3DSpatializerView::normalized_to_azimuth(norm_az);
        assert!((az_back - 0.0).abs() < 1e-3);

        let norm_el = Mpegh3DSpatializerView::elevation_to_normalized(45.0);
        assert!((norm_el - 0.75).abs() < 1e-3);
        let el_back = Mpegh3DSpatializerView::normalized_to_elevation(norm_el);
        assert!((el_back - 45.0).abs() < 1e-3);

        // Cartesian 3D coordinates
        let (cx, cy, cz) = view.evaluate_cartesian_position();
        assert!(cx.is_finite() && cy.is_finite() && cz.is_finite());

        // Bed energy evaluation
        for &e in &view.bed_energy {
            assert!(e >= 0.0);
        }

        // Hit testing
        let canvas = Rect::new(20.0, 56.0, 420.0, 224.0);
        let puck_x = canvas.x + view.mpegh_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.mpegh_puck_pos.1) * canvas.height;
        assert!(view.hit_test_mpegh_puck((puck_x, puck_y), canvas));
        assert!(!view.hit_test_mpegh_puck((puck_x + 90.0, puck_y), canvas));

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains('+'));
        assert!(snap.contains('|'));
    }

    #[test]
    fn test_turn75_pure_slot_arithmetic_isolation() {
        let track_id = 5u32;
        let slot_id = 2u32;
        let base_pid = ParamId(track_id * 1000 + slot_id * 20);

        let pid_az = ParamId(base_pid.0);
        let pid_el = ParamId(base_pid.0 + 1);
        let pid_dst = ParamId(base_pid.0 + 2);
        let pid_foc = ParamId(base_pid.0 + 3);

        assert_eq!(pid_az.0, 5040);
        assert_eq!(pid_el.0, 5041);
        assert_eq!(pid_dst.0, 5042);
        assert_eq!(pid_foc.0, 5043);

        let next_slot_pid = ParamId(track_id * 1000 + (slot_id + 1) * 20);
        assert_eq!(next_slot_pid.0, 5060);
        assert!(pid_foc.0 < next_slot_pid.0);
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
    fn test_turn75_gui_award_winning_view_headless_rendering_without_panic() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // Open HOA5 Binaural modal
        view.open_hoa5_binaural_hud();
        assert!(view.is_hoa5_binaural_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_hoa5_binaural_hud();
        assert!(!view.is_hoa5_binaural_hud_open());

        // Open MPEG-H 3D Spatializer modal
        view.open_mpegh_3d_hud();
        assert!(view.is_mpegh_3d_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_mpegh_3d_hud();
        assert!(!view.is_mpegh_3d_hud_open());
    }

    #[test]
    fn test_turn75_gui_live_parambus_slot_dispatch() {
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

        // 1. Sync HOA5 Binaural HUD
        view.hoa5_binaural_view.azimuth_deg = 60.0;
        view.hoa5_binaural_view.elevation_deg = 30.0;
        view.hoa5_binaural_view.distance_m = 3.0;
        view.hoa5_binaural_view.energy_focus = 0.85;
        view.sync_hoa5_binaural_hud_state();

        assert_eq!(arc_bus.get(ParamId(base)), Some(60.0));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(30.0));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(3.0));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(0.85));

        assert_eq!(
            view.device_rack_state.node_param_values.get("azimuth_deg"),
            Some(&60.0)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("azimuth_deg"),
            Some(&60.0)
        );

        // 2. Sync MPEG-H 3D Spatializer HUD
        view.mpegh_3d_spatializer_view.azimuth_deg = -45.0;
        view.mpegh_3d_spatializer_view.elevation_deg = 15.0;
        view.mpegh_3d_spatializer_view.distance_m = 4.2;
        view.mpegh_3d_spatializer_view.spread_divergence_pct = 50.0;
        view.sync_mpegh_3d_hud_state();

        assert_eq!(arc_bus.get(ParamId(base)), Some(-45.0));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(15.0));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(4.2));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(50.0));

        assert_eq!(
            view.device_rack_state.node_param_values.get("spread_divergence_pct"),
            Some(&50.0)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("spread_divergence_pct"),
            Some(&50.0)
        );
    }

    #[test]
    fn test_turn75_gui_rack_and_inspector_trigger_propagation() {
        let mut view = AwardWinningGuiView::new();

        view.device_rack_state.requested_open_hoa5_binaural_hud = true;
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_hoa5_binaural_hud_open());
        assert!(!view.device_rack_state.requested_open_hoa5_binaural_hud);

        view.close_hoa5_binaural_hud();
        assert!(!view.is_hoa5_binaural_hud_open());

        view.inspector_state.requested_open_mpegh_3d_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_mpegh_3d_hud_open());
        assert!(!view.inspector_state.requested_open_mpegh_3d_hud);
    }

    #[test]
    fn test_turn75_gui_command_palette_action_registration() {
        let palette = CommandPalette::new();
        let hoa_action = palette
            .actions
            .iter()
            .find(|a| a.action_id == "open_hoa5_binaural_hud");
        assert!(hoa_action.is_some());
        let hoa = hoa_action.unwrap();
        assert_eq!(hoa.category, "Spatial");
        assert!(hoa.label.contains("5th-Order Ambisonics HOA5"));

        let mp_action = palette
            .actions
            .iter()
            .find(|a| a.action_id == "open_mpegh_3d_hud");
        assert!(mp_action.is_some());
        let mp = mp_action.unwrap();
        assert_eq!(mp.category, "Spatial");
        assert!(mp.label.contains("MPEG-H 3D Immersive Spatializer"));
    }
}
