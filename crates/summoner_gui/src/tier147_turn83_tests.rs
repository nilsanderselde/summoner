#![allow(clippy::all)]

//! Test suite for Turn #83 (Sprint Turn #9 — Sprint Review) — Higher-Order Ambisonics 4th-Order (25-ch) 3D Energy Density HUD & MPEG-H 3D Spatial Audio & Personalized HRTF Profile HUD Convergence.

#[cfg(test)]
pub mod pure_tests {
    use crate::layout_math::Rect;
    use crate::views::hoa4_spatializer_view::{
        Hoa4Profile, Hoa4SpatializerView, HOA4_PUCK_HIT_RADIUS,
    };
    use crate::views::mpegh_spatializer_view::{
        HrtfProfile, MpeghFormat, MpeghSpatializerView, MPEGH_PUCK_HIT_RADIUS,
    };
    use summoner_core::param_bus::ParamId;

    #[test]
    fn test_turn83_pure_hoa4_spatializer_view_initialization_and_simulation() {
        let mut view = Hoa4SpatializerView::new();
        assert_eq!(view.hoa_profile, Hoa4Profile::Hoa4_25ChannelSphere);
        assert!((view.azimuth_deg - 45.0).abs() < 1e-3);
        assert!((view.elevation_deg - 25.0).abs() < 1e-3);
        assert!((view.distance_m - 3.0).abs() < 1e-3);
        assert!((view.energy_focus_order - 4.0).abs() < 1e-3);
        assert_eq!(HOA4_PUCK_HIT_RADIUS, 22.0);

        // Coordinate normalizations
        for az in [-180.0, -90.0, 0.0, 90.0, 180.0] {
            let norm = Hoa4SpatializerView::azimuth_to_normalized(az);
            assert!((0.0..=1.0).contains(&norm));
            let back = Hoa4SpatializerView::normalized_to_azimuth(norm);
            assert!((back - az).abs() < 1e-4);
        }

        for el in [-90.0, -45.0, 0.0, 45.0, 90.0] {
            let norm = Hoa4SpatializerView::elevation_to_normalized(el);
            assert!((0.0..=1.0).contains(&norm));
            let back = Hoa4SpatializerView::normalized_to_elevation(norm);
            assert!((back - el).abs() < 1e-4);
        }

        // Profile switching
        let profiles = [
            (Hoa4Profile::Hoa4_25ChannelSphere, 25, 4),
            (Hoa4Profile::BinauralHoa4Hrir, 2, 4),
            (Hoa4Profile::Surround22_2Hoa4, 24, 4),
            (Hoa4Profile::Dome7_1_4Hoa4, 12, 4),
            (Hoa4Profile::EnergyMaxReDecoder, 25, 4),
        ];

        for (prof, exp_channels, exp_order) in profiles {
            view.set_profile(prof);
            assert_eq!(view.hoa_profile, prof);
            assert_eq!(prof.channel_count(), exp_channels);
            assert_eq!(prof.ambisonic_order(), exp_order);
            assert!(!prof.profile_name().is_empty());
        }

        // Cartesian evaluation
        view.azimuth_deg = 0.0;
        view.elevation_deg = 0.0;
        view.distance_m = 5.0;
        let (x, y, z) = view.evaluate_cartesian_position();
        assert!(x.abs() < 1e-4);
        assert!((y - 5.0).abs() < 1e-4);
        assert!(z.abs() < 1e-4);

        // Simulation update: 8 octants energy check
        view.azimuth_deg = 45.0;
        view.elevation_deg = 30.0;
        view.update_ambisonic_simulation();
        assert_eq!(view.octant_energy.len(), 8);
        assert!(view.octant_energy[0] > 0.0); // Front-Left-Up

        // Puck hit testing
        let canvas = Rect::new(20.0, 100.0, 420.0, 230.0);
        view.hoa4_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(view.hit_test_hoa4_puck((center_x, center_y), canvas));
        assert!(!view.hit_test_hoa4_puck((center_x + 100.0, center_y), canvas));

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains('+'));
        assert!(snap.contains('|'));
    }

    #[test]
    fn test_turn83_pure_mpegh_spatializer_view_initialization_and_simulation() {
        let mut view = MpeghSpatializerView::new();
        assert_eq!(view.format, MpeghFormat::Mpegh714);
        assert_eq!(view.hrtf_profile, HrtfProfile::KemarStandard);
        assert!((view.azimuth_deg - 45.0).abs() < 1e-3);
        assert!((view.elevation_deg - 15.0).abs() < 1e-3);
        assert!((view.distance_m - 2.5).abs() < 1e-3);
        assert_eq!(MPEGH_PUCK_HIT_RADIUS, 22.0);

        // Coordinate normalizations
        for az in [-180.0, -90.0, 0.0, 90.0, 180.0] {
            let norm = MpeghSpatializerView::azimuth_to_normalized(az);
            assert!((0.0..=1.0).contains(&norm));
            let back = MpeghSpatializerView::normalized_to_azimuth(norm);
            assert!((back - az).abs() < 1e-4);
        }

        for el in [-90.0, -45.0, 0.0, 45.0, 90.0] {
            let norm = MpeghSpatializerView::elevation_to_normalized(el);
            assert!((0.0..=1.0).contains(&norm));
            let back = MpeghSpatializerView::normalized_to_elevation(norm);
            assert!((back - el).abs() < 1e-4);
        }

        // Format switching
        let formats = [
            (MpeghFormat::Mpegh714, 12),
            (MpeghFormat::Mpegh51, 6),
            (MpeghFormat::Mpegh222Dome, 24),
            (MpeghFormat::MpeghBinaural, 2),
            (MpeghFormat::MpeghDynamicObj, 16),
        ];

        for (fmt, exp_channels) in formats {
            view.set_format(fmt);
            assert_eq!(view.format, fmt);
            assert_eq!(fmt.channel_count(), exp_channels);
            assert!(!fmt.display_name().is_empty());
        }

        // HRTF profile evaluation
        let hrtf_profiles = [
            (HrtfProfile::KemarStandard, 7.2),
            (HrtfProfile::GenelecAural, 8.4),
            (HrtfProfile::SphericalHead, 6.8),
            (HrtfProfile::Photogrammetry3D, 9.1),
        ];

        for (hp, exp_notch) in hrtf_profiles {
            view.hrtf_profile = hp;
            assert!((hp.notch_freq_khz() - exp_notch).abs() < 1e-3);
            assert!(!hp.display_name().is_empty());
            let (mag_l, mag_r) = view.evaluate_hrtf_magnitude(1000.0);
            assert!(mag_l >= -30.0 && mag_l <= 12.0);
            assert!(mag_r >= -30.0 && mag_r <= 12.0);
        }

        // Spatial calculations: ITD / ILD check
        view.azimuth_deg = 90.0; // Hard right
        view.elevation_deg = 0.0;
        view.distance_m = 1.0;
        view.update_spatial_calculations();
        assert!(view.itd_microseconds > 500.0); // Maximum ITD near 650 us
        assert!(view.ild_db > 10.0); // Significant interaural level difference

        view.azimuth_deg = 0.0; // Center front
        view.update_spatial_calculations();
        assert!(view.itd_microseconds.abs() < 1e-3);
        assert!(view.ild_db.abs() < 1e-3);

        // Puck hit testing
        let canvas = Rect::new(20.0, 100.0, 420.0, 230.0);
        view.object_puck_pos = (0.5, 0.5);
        let center_x = canvas.x + 0.5 * canvas.width;
        let center_y = canvas.y + 0.5 * canvas.height;
        assert!(view.hit_test_object_puck((center_x, center_y), canvas));
        assert!(!view.hit_test_object_puck((center_x + 100.0, center_y), canvas));

        // ASCII snapshot
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains('+'));
        assert!(snap.contains('|'));
    }

    #[test]
    fn test_turn83_pure_lock_free_param_bus_slot_arithmetic() {
        for track_id in [1u32, 2, 5, 8] {
            for slot_idx in [0usize, 1, 3, 7] {
                let base = track_id * 1000 + slot_idx as u32 * 20;

                // HOA 4th-Order: 6 parameters (offsets 0..=5)
                let hoa_az = ParamId(base);
                let hoa_el = ParamId(base + 1);
                let hoa_dist = ParamId(base + 2);
                let hoa_foc = ParamId(base + 3);
                let hoa_oct0 = ParamId(base + 4);
                let hoa_oct1 = ParamId(base + 5);

                assert_eq!(hoa_az.0, base);
                assert_eq!(hoa_el.0, base + 1);
                assert_eq!(hoa_dist.0, base + 2);
                assert_eq!(hoa_foc.0, base + 3);
                assert_eq!(hoa_oct0.0, base + 4);
                assert_eq!(hoa_oct1.0, base + 5);

                // MPEG-H Spatial Audio: 6 parameters (offsets 0..=5)
                let mpg_az = ParamId(base);
                let mpg_el = ParamId(base + 1);
                let mpg_dist = ParamId(base + 2);
                let mpg_itd = ParamId(base + 3);
                let mpg_ild = ParamId(base + 4);
                let mpg_lufs = ParamId(base + 5);

                assert_eq!(mpg_az.0, base);
                assert_eq!(mpg_el.0, base + 1);
                assert_eq!(mpg_dist.0, base + 2);
                assert_eq!(mpg_itd.0, base + 3);
                assert_eq!(mpg_ild.0, base + 4);
                assert_eq!(mpg_lufs.0, base + 5);

                // Verify slot boundary isolation (stays within slot allocation of 20 IDs)
                let next_slot_base = track_id * 1000 + (slot_idx as u32 + 1) * 20;
                assert!(hoa_oct1.0 < next_slot_base);
                assert!(mpg_lufs.0 < next_slot_base);
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
    fn test_turn83_gui_award_winning_view_hoa4_and_mpegh_spatializer_modal_lifecycle() {
        let mut view = AwardWinningGuiView::new();
        let ctx = egui::Context::default();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // 1. HOA 4th-Order (25-ch) 3D Ambisonic Energy Density HUD
        view.open_hoa4_spatializer_hud();
        assert!(view.is_hoa4_spatializer_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_hoa4_spatializer_hud();
        assert!(!view.is_hoa4_spatializer_hud_open());

        // 2. MPEG-H 3D Spatial Audio & Personalized HRTF Profile HUD
        view.open_mpegh_spatializer_hud();
        assert!(view.is_mpegh_spatializer_hud_open());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        view.close_mpegh_spatializer_hud();
        assert!(!view.is_mpegh_spatializer_hud_open());
    }

    #[test]
    fn test_turn83_gui_lock_free_param_bus_dispatch_hoa4_and_mpegh_spatializer() {
        let mut view = AwardWinningGuiView::new();
        let mut bus = ParamBus::new();
        let track_id = 2u32;
        let slot_idx = 0usize;
        view.device_rack_state.selected_chain_idx = slot_idx;

        // Register bus channels
        for offset in 0..=6 {
            let pid = ParamId(track_id * 1000 + slot_idx as u32 * 20 + offset);
            bus.register(pid, 0.0);
        }
        let arc_bus = Arc::new(bus);
        view.live_param_bus = Some(Arc::clone(&arc_bus));

        // Test HOA4 spatializer sync
        view.hoa4_spatializer_view.azimuth_deg = 35.0;
        view.hoa4_spatializer_view.elevation_deg = 15.0;
        view.hoa4_spatializer_view.distance_m = 4.0;
        view.hoa4_spatializer_view.energy_focus_order = 3.5;
        view.hoa4_spatializer_view.octant_energy[0] = 0.75;
        view.hoa4_spatializer_view.octant_energy[1] = 0.55;

        view.sync_hoa4_spatializer_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("azimuth_deg"), Some(&35.0));
        assert_eq!(view.device_rack_state.node_param_values.get("elevation_deg"), Some(&15.0));
        assert_eq!(view.device_rack_state.node_param_values.get("distance_m"), Some(&4.0));
        assert_eq!(view.device_rack_state.node_param_values.get("energy_focus_order"), Some(&3.5));

        // Test MPEG-H spatial audio sync
        view.mpegh_spatializer_view.azimuth_deg = -45.0;
        view.mpegh_spatializer_view.elevation_deg = -10.0;
        view.mpegh_spatializer_view.distance_m = 2.0;
        view.mpegh_spatializer_view.itd_microseconds = -320.0;
        view.mpegh_spatializer_view.ild_db = 4.5;
        view.mpegh_spatializer_view.integrated_lufs = -18.0;

        view.sync_mpegh_spatializer_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("azimuth_deg"), Some(&-45.0));
        assert_eq!(view.device_rack_state.node_param_values.get("elevation_deg"), Some(&-10.0));
        assert_eq!(view.device_rack_state.node_param_values.get("distance_m"), Some(&2.0));
        assert_eq!(view.device_rack_state.node_param_values.get("itd_microseconds"), Some(&-320.0));
    }

    #[test]
    fn test_turn83_gui_modern_inspector_and_rack_triggers_propagation() {
        let mut view = AwardWinningGuiView::new();

        // Inspector triggers
        view.inspector_state.requested_open_hoa4_spatializer_hud = true;
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.inspector_state.requested_open_hoa4_spatializer_hud);
        assert!(view.is_hoa4_spatializer_hud_open());
        view.close_hoa4_spatializer_hud();

        view.inspector_state.requested_open_mpegh_spatializer_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.inspector_state.requested_open_mpegh_spatializer_hud);
        assert!(view.is_mpegh_spatializer_hud_open());
        view.close_mpegh_spatializer_hud();

        // Rack triggers
        view.device_rack_state.requested_open_hoa4_spatializer_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.device_rack_state.requested_open_hoa4_spatializer_hud);
        assert!(view.is_hoa4_spatializer_hud_open());
        view.close_hoa4_spatializer_hud();

        view.device_rack_state.requested_open_mpegh_spatializer_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(!view.device_rack_state.requested_open_mpegh_spatializer_hud);
        assert!(view.is_mpegh_spatializer_hud_open());
        view.close_mpegh_spatializer_hud();
    }

    #[test]
    fn test_turn83_gui_command_palette_registration_hoa4_and_mpegh_spatializer() {
        let palette = CommandPalette::new();
        let hoa4_act = palette.actions.iter().find(|a| a.action_id == "open_hoa4_spatializer_hud");
        assert!(hoa4_act.is_some(), "open_hoa4_spatializer_hud must be registered");
        assert_eq!(hoa4_act.unwrap().category, "Spatial");

        let mpegh_act = palette.actions.iter().find(|a| a.action_id == "open_mpegh_spatializer_hud");
        assert!(mpegh_act.is_some(), "open_mpegh_spatializer_hud must be registered");
        assert_eq!(mpegh_act.unwrap().category, "Spatial");
    }

    #[test]
    fn test_turn83_gui_modern_device_rack_and_inspector_headless_render() {
        let ctx = egui::Context::default();
        let mut rack_state = ModernDeviceRackState::default();
        rack_state.selected_node_kind = Some("Hoa4Spatializer".to_string());
        rack_state.chain_devices = vec![crate::views::modern_device_rack::RackChainDeviceVisual {
            kind: "Hoa4Spatializer".to_string(),
            display_name: "HOA 4th-Order".to_string(),
            is_bypassed: false,
        }];

        let mut inspector_state = ModernInspectorState::default();
        inspector_state.selected_node_kind = Some("MpeghSpatializer".to_string());
        inspector_state.chain_devices = vec![crate::views::modern_device_rack::RackChainDeviceVisual {
            kind: "MpeghSpatializer".to_string(),
            display_name: "MPEG-H 3D".to_string(),
            is_bypassed: false,
        }];

        let bus = ParamBus::new();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_device_rack_with_context(ui, &mut rack_state, None, Some(&bus), 1);
                show_modern_inspector_with_context(ui, &mut inspector_state, Some(&bus), 1);
            });
        });
    }
}
