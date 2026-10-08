#![allow(clippy::all)]

//! Test suite for Turn #78 (Turn #41 deliverable / Sprint Turn #4) — Spatial Binaural HRTF 3D Orbit & Multi-Source Binaural BRIR Room Impulse Response HUDs Convergence.

#[cfg(test)]
pub mod pure_tests {
    use crate::layout_math::Rect;
    use crate::views::binaural_panner_view::{
        BinauralPannerView, HrtfModel, BINAURAL_PUCK_HIT_RADIUS,
    };
    use crate::views::binaural_brir_view::{
        BinauralBrirView, RoomAcousticProfile, BRIR_SOURCE_PUCK_HIT_RADIUS,
    };
    use summoner_core::param_bus::ParamId;

    #[test]
    fn test_turn78_pure_binaural_panner_view_initialization_and_cues() {
        let mut view = BinauralPannerView::new();
        assert_eq!(view.model, HrtfModel::KemarStandardDummy);
        assert!((view.azimuth_deg - 45.0).abs() < 1e-3);
        assert!((view.elevation_deg - 10.0).abs() < 1e-3);
        assert!((view.distance_m - 1.5).abs() < 1e-3);
        assert!(view.room_reflections);
        assert!((view.crossfeed_percent - 65.0).abs() < 1e-3);
        assert_eq!(BINAURAL_PUCK_HIT_RADIUS, 22.0);

        // Model profiles
        view.model = HrtfModel::CustomSubjectModel;
        assert_eq!(view.model, HrtfModel::CustomSubjectModel);
        view.model = HrtfModel::SphericalHeadRayTraced;
        assert_eq!(view.model, HrtfModel::SphericalHeadRayTraced);
        view.model = HrtfModel::NearFieldBinaural;
        assert_eq!(view.model, HrtfModel::NearFieldBinaural);

        // Coordinate normalizations
        let norm_az = BinauralPannerView::azimuth_to_normalized(0.0);
        assert!((norm_az - 0.5).abs() < 1e-3);
        let back_az = BinauralPannerView::normalized_to_azimuth(norm_az);
        assert!((back_az - 0.0).abs() < 1e-3);

        let norm_el = BinauralPannerView::elevation_to_normalized(0.0);
        assert!((norm_el - 0.5).abs() < 1e-3);
        let back_el = BinauralPannerView::normalized_to_elevation(norm_el);
        assert!((back_el - 0.0).abs() < 1e-3);

        let norm_dist = BinauralPannerView::distance_to_normalized(1.5);
        let back_dist = BinauralPannerView::normalized_to_distance(norm_dist);
        assert!((back_dist - 1.5).abs() < 1e-2);

        // Acoustic ITD and ILD cues
        let (itd_center, ild_center) = BinauralPannerView::calculate_interaural_cues(0.0, 0.0, 1.0);
        assert!(itd_center.abs() < 1e-3);
        assert!(ild_center.abs() < 1e-3);

        let (itd_right, ild_right) = BinauralPannerView::calculate_interaural_cues(90.0, 0.0, 1.0);
        assert!(itd_right > 500.0);
        assert!(ild_right > 10.0);

        let (itd_left, ild_left) = BinauralPannerView::calculate_interaural_cues(-90.0, 0.0, 1.0);
        assert!(itd_left < -500.0);
        assert!(ild_left < -10.0);

        // Hit testing
        let canvas = Rect::new(20.0, 56.0, 430.0, 224.0);
        let cx = canvas.x + canvas.width * 0.5;
        let cy = canvas.y + canvas.height * 0.5;
        let max_r = (canvas.width.min(canvas.height) * 0.42).max(10.0);
        let dist_norm = BinauralPannerView::distance_to_normalized(view.distance_m);
        let r = 25.0 + dist_norm * (max_r - 25.0);
        let az_rad = (view.azimuth_deg - 90.0).to_radians();
        let px = cx + az_rad.cos() * r;
        let py = cy + az_rad.sin() * r;

        assert!(view.hit_test_orbit_puck((px, py), canvas));
        assert!(!view.hit_test_orbit_puck((px + 80.0, py + 80.0), canvas));

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("BINAURAL"));
        assert!(snap.contains("NearFieldBinaural"));
    }

    #[test]
    fn test_turn78_pure_binaural_brir_view_initialization_and_profiles() {
        let mut view = BinauralBrirView::new();
        assert_eq!(view.room_profile, RoomAcousticProfile::ScoringStage);
        assert!((view.azimuth_deg - 45.0).abs() < 1e-3);
        assert!((view.elevation_deg - 0.0).abs() < 1e-3);
        assert!((view.distance_m - 2.50).abs() < 1e-3);
        assert!((view.head_radius_m - 0.0875).abs() < 1e-4);
        assert!((view.speed_of_sound_mps - 343.0).abs() < 1e-3);
        assert!((view.hrtf_pinna_notch_hz - 8025.0).abs() < 1e-3);
        assert_eq!(BRIR_SOURCE_PUCK_HIT_RADIUS, 22.0);

        // Room profile metrics
        assert!((view.room_profile.rt60_seconds() - 1.45).abs() < 1e-3);
        assert!((view.room_profile.direct_to_reverberant_ratio_db() - 5.2).abs() < 1e-3);
        assert!((view.room_profile.early_reflection_delay_ms() - 18.0).abs() < 1e-3);

        view.room_profile = RoomAcousticProfile::ConcertHall;
        assert!((view.room_profile.rt60_seconds() - 2.10).abs() < 1e-3);
        assert!((view.room_profile.direct_to_reverberant_ratio_db() - 2.5).abs() < 1e-3);
        assert!((view.room_profile.early_reflection_delay_ms() - 28.0).abs() < 1e-3);

        view.room_profile = RoomAcousticProfile::ScoringStage;
        assert!((view.room_profile.rt60_seconds() - 1.45).abs() < 1e-3);
        assert!((view.room_profile.direct_to_reverberant_ratio_db() - 5.2).abs() < 1e-3);

        view.room_profile = RoomAcousticProfile::CathedralSpace;
        assert!((view.room_profile.rt60_seconds() - 4.80).abs() < 1e-3);
        assert!((view.room_profile.direct_to_reverberant_ratio_db() - -3.0).abs() < 1e-3);

        view.room_profile = RoomAcousticProfile::DryStudio;
        assert!((view.room_profile.rt60_seconds() - 0.35).abs() < 1e-3);
        assert!((view.room_profile.direct_to_reverberant_ratio_db() - 14.0).abs() < 1e-3);

        view.room_profile = RoomAcousticProfile::IntimateChamber;
        assert!((view.room_profile.rt60_seconds() - 0.85).abs() < 1e-3);
        assert!((view.room_profile.direct_to_reverberant_ratio_db() - 8.5).abs() < 1e-3);

        // Coordinate normalization
        let norm_az = BinauralBrirView::azimuth_to_normalized(0.0);
        assert!((norm_az - 0.5).abs() < 1e-3);
        let back_az = BinauralBrirView::normalized_to_azimuth(norm_az);
        assert!((back_az - 0.0).abs() < 1e-3);

        let norm_dist = BinauralBrirView::distance_to_normalized(2.50);
        let back_dist = BinauralBrirView::normalized_to_distance(norm_dist);
        assert!((back_dist - 2.50).abs() < 1e-2);

        // Early reflection evaluation (6 rays)
        for i in 0..6 {
            let (delay, amp) = view.evaluate_early_reflection(i);
            assert!(delay > 0.0);
            assert!(amp > 0.0);
        }

        // Hit testing
        let canvas = Rect::new(0.0, 0.0, 400.0, 400.0);
        let puck_x = canvas.x + view.source_puck_pos.0 * canvas.width;
        let puck_y = canvas.y + (1.0 - view.source_puck_pos.1) * canvas.height;
        assert!(view.hit_test_source_puck((puck_x, puck_y), canvas));
        assert!(!view.hit_test_source_puck((puck_x + 80.0, puck_y + 80.0), canvas));

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains('+'));
        assert!(snap.contains('|'));
    }

    #[test]
    fn test_turn78_pure_slot_arithmetic_channel_isolation() {
        for track_id in [1u32, 2u32, 9u32] {
            for slot_idx in [0u32, 1u32, 3u32] {
                let base = track_id * 1000 + slot_idx * 20;

                let bin_az = ParamId(base);
                let bin_el = ParamId(base + 1);
                let bin_dist = ParamId(base + 2);
                let bin_cross = ParamId(base + 3);

                assert_ne!(bin_az, bin_el);
                assert_ne!(bin_el, bin_dist);
                assert_ne!(bin_dist, bin_cross);

                let next_slot_base = track_id * 1000 + (slot_idx + 1) * 20;
                assert!(bin_cross.0 < next_slot_base);
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
    fn test_turn78_gui_award_winning_view_headless_rendering_without_panic() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // Open Binaural HRTF 3D Orbit modal
        view.open_binaural_panner_hud();
        assert!(view.is_binaural_panner_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_binaural_panner_hud();
        assert!(!view.is_binaural_panner_hud_open());

        // Open Binaural BRIR Room Impulse Response modal
        view.open_binaural_brir_hud();
        assert!(view.is_binaural_brir_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_binaural_brir_hud();
        assert!(!view.is_binaural_brir_hud_open());
    }

    #[test]
    fn test_turn78_gui_live_parambus_slot_dispatch() {
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

        // 1. Sync Binaural Panner HUD
        view.binaural_panner_view.azimuth_deg = 75.0;
        view.binaural_panner_view.elevation_deg = 15.0;
        view.binaural_panner_view.distance_m = 2.2;
        view.binaural_panner_view.crossfeed_percent = 80.0;
        view.sync_binaural_panner_hud_state();

        assert_eq!(arc_bus.get(ParamId(base)), Some(75.0));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(15.0));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(2.2));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(80.0));

        assert_eq!(
            view.device_rack_state.node_param_values.get("azimuth_deg"),
            Some(&75.0)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("azimuth_deg"),
            Some(&75.0)
        );

        // 2. Sync Binaural BRIR HUD
        view.binaural_brir_view.azimuth_deg = -45.0;
        view.binaural_brir_view.elevation_deg = 5.0;
        view.binaural_brir_view.distance_m = 3.2;
        view.binaural_brir_view.hrtf_pinna_notch_hz = 6800.0;
        view.sync_binaural_brir_hud_state();

        assert_eq!(arc_bus.get(ParamId(base)), Some(-45.0));
        assert_eq!(arc_bus.get(ParamId(base + 1)), Some(5.0));
        assert_eq!(arc_bus.get(ParamId(base + 2)), Some(3.2));
        assert_eq!(arc_bus.get(ParamId(base + 3)), Some(6800.0));

        assert_eq!(
            view.device_rack_state.node_param_values.get("azimuth_deg"),
            Some(&-45.0)
        );
        assert_eq!(
            view.inspector_state.node_param_values.get("azimuth_deg"),
            Some(&-45.0)
        );
    }

    #[test]
    fn test_turn78_gui_rack_and_inspector_trigger_propagation() {
        let mut view = AwardWinningGuiView::new();

        view.device_rack_state.requested_open_binaural_panner_hud = true;
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_binaural_panner_hud_open());
        assert!(!view.device_rack_state.requested_open_binaural_panner_hud);

        view.close_binaural_panner_hud();
        assert!(!view.is_binaural_panner_hud_open());

        view.inspector_state.requested_open_binaural_brir_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_binaural_brir_hud_open());
        assert!(!view.inspector_state.requested_open_binaural_brir_hud);
    }

    #[test]
    fn test_turn78_gui_command_palette_action_registration() {
        let palette = CommandPalette::new();
        let panner_action = palette
            .actions
            .iter()
            .find(|a| a.action_id == "open_binaural_panner_hud");
        assert!(panner_action.is_some());
        let panner = panner_action.unwrap();
        assert_eq!(panner.category, "Spatial");
        assert!(panner.label.contains("Binaural HRTF"));

        let brir_action = palette
            .actions
            .iter()
            .find(|a| a.action_id == "open_binaural_brir_hud");
        assert!(brir_action.is_some());
        let brir = brir_action.unwrap();
        assert_eq!(brir.category, "Spatial");
        assert!(brir.label.contains("Binaural BRIR"));
    }
}
